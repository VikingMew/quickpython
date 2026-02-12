mod bytecode;
mod compiler;
mod context;
mod serializer;
mod value;
mod vm;

pub use context::Context;
pub use value::Value;

use clap::{Parser, Subcommand};
use std::process;

use compiler::Compiler;
use serializer::{deserialize_bytecode, serialize_bytecode};

#[derive(Parser)]
#[command(name = "quickpython")]
#[command(about = "A simple Python interpreter", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Python file
    Run { file: String },
    /// Compile a Python file to bytecode
    Compile {
        file: String,
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => {
            // 检测文件类型
            if file.ends_with(".pyq") {
                // 运行字节码文件
                let data = match std::fs::read(&file) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("Error reading file '{}': {}", file, e);
                        process::exit(1);
                    }
                };

                let bytecode = match deserialize_bytecode(&data) {
                    Ok(bc) => bc,
                    Err(e) => {
                        eprintln!("Error deserializing bytecode: {}", e);
                        process::exit(1);
                    }
                };

                let mut vm = vm::VM::new();
                let mut globals = std::collections::HashMap::new();
                match vm.execute(&bytecode, &mut globals) {
                    Ok(result) => {
                        if let Some(i) = result.as_int() {
                            println!("{}", i);
                        }
                    }
                    Err(e) => {
                        eprintln!("Runtime error: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                // 运行 Python 文件
                let source = match std::fs::read_to_string(&file) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error reading file '{}': {}", file, e);
                        process::exit(1);
                    }
                };

                let mut ctx = Context::new();
                match ctx.eval(&source) {
                    Ok(result) => {
                        if let Some(i) = result.as_int() {
                            println!("{}", i);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
        Commands::Compile { file, output } => {
            // 读取源文件
            let source = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file '{}': {}", file, e);
                    process::exit(1);
                }
            };

            // 编译
            let bytecode = match Compiler::compile(&source) {
                Ok(bc) => bc,
                Err(e) => {
                    eprintln!("Compile error: {}", e);
                    process::exit(1);
                }
            };

            // 序列化
            let data = match serialize_bytecode(&bytecode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Serialization error: {}", e);
                    process::exit(1);
                }
            };

            // 确定输出文件名
            let output_file = output
                .unwrap_or_else(|| file.strip_suffix(".py").unwrap_or(&file).to_string() + ".pyq");

            // 写入文件
            match std::fs::write(&output_file, data) {
                Ok(_) => {
                    println!("Compiled successfully: {}", output_file);
                }
                Err(e) => {
                    eprintln!("Error writing file '{}': {}", output_file, e);
                    process::exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_add() {
        let mut ctx = Context::new();
        let result = ctx.eval("1 + 2").unwrap();
        assert_eq!(result.as_int(), Some(3));
    }

    #[test]
    fn test_simple_mul() {
        let mut ctx = Context::new();
        let result = ctx.eval("10 * 5").unwrap();
        assert_eq!(result.as_int(), Some(50));
    }

    #[test]
    fn test_simple_div() {
        let mut ctx = Context::new();
        let result = ctx.eval("100 / 4").unwrap();
        assert_eq!(result.as_int(), Some(25));
    }

    #[test]
    fn test_simple_sub() {
        let mut ctx = Context::new();
        let result = ctx.eval("7 - 3").unwrap();
        assert_eq!(result.as_int(), Some(4));
    }

    #[test]
    fn test_complex_expr() {
        let mut ctx = Context::new();
        let result = ctx.eval("(10 + 5) * 2").unwrap();
        assert_eq!(result.as_int(), Some(30));
    }

    #[test]
    fn test_another_complex_expr() {
        let mut ctx = Context::new();
        let result = ctx.eval("(1 + 2) * 3").unwrap();
        assert_eq!(result.as_int(), Some(9));
    }

    #[test]
    fn test_division_by_zero() {
        let mut ctx = Context::new();
        let result = ctx.eval("10 / 0");
        assert!(result.is_err());
    }

    #[test]
    fn test_variable_assignment() {
        let mut ctx = Context::new();
        ctx.eval("x = 42").unwrap();
        let x = ctx.eval("x").unwrap();
        assert_eq!(x.as_int(), Some(42));
    }

    #[test]
    fn test_variable_expr() {
        let mut ctx = Context::new();
        ctx.eval("x = 10").unwrap();
        ctx.eval("y = x * 2").unwrap();
        let y = ctx.eval("y").unwrap();
        assert_eq!(y.as_int(), Some(20));
    }

    #[test]
    fn test_get_set_api() {
        let mut ctx = Context::new();
        ctx.set("z", Value::Int(100));
        let z = ctx.get("z").unwrap();
        assert_eq!(z.as_int(), Some(100));
    }

    #[test]
    fn test_undefined_variable() {
        let mut ctx = Context::new();
        let result = ctx.eval("undefined_var");
        assert!(result.is_err());
    }

    #[test]
    fn test_function_def_and_call() {
        let mut ctx = Context::new();
        ctx.eval("def add(a, b):\n    return a + b").unwrap();
        let result = ctx.eval("add(1, 2)").unwrap();
        assert_eq!(result.as_int(), Some(3));
    }

    #[test]
    fn test_factorial() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)
        "#,
        )
        .unwrap();

        let result = ctx.eval("factorial(5)").unwrap();
        assert_eq!(result.as_int(), Some(120));
    }

    #[test]
    fn test_if_else() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
def max(a, b):
    if a > b:
        return a
    else:
        return b
        "#,
        )
        .unwrap();

        let result = ctx.eval("max(10, 5)").unwrap();
        assert_eq!(result.as_int(), Some(10));

        let result = ctx.eval("max(3, 8)").unwrap();
        assert_eq!(result.as_int(), Some(8));
    }

    #[test]
    fn test_comparison_operators() {
        let mut ctx = Context::new();

        let result = ctx.eval("5 > 3").unwrap();
        assert_eq!(result.as_bool(), Some(true));

        let result = ctx.eval("5 < 3").unwrap();
        assert_eq!(result.as_bool(), Some(false));

        let result = ctx.eval("5 == 5").unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }
}
