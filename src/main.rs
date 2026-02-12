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

    #[test]
    fn test_while_loop() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
i = 0
sum = 0
while i < 10:
    sum = sum + i
    i = i + 1
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(45));
    }

    #[test]
    fn test_fibonacci_iterative() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
def fib(n):
    if n <= 1:
        return n
    a = 0
    b = 1
    i = 2
    while i <= n:
        temp = a + b
        a = b
        b = temp
        i = i + 1
    return b
        "#,
        )
        .unwrap();

        let result = ctx.eval("fib(10)").unwrap();
        assert_eq!(result.as_int(), Some(55));
    }

    #[test]
    fn test_string_literal() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""hello""#).unwrap();
        assert_eq!(result.as_string(), Some("hello"));
    }

    #[test]
    fn test_string_concatenation() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""hello" + " " + "world""#).unwrap();
        assert_eq!(result.as_string(), Some("hello world"));
    }

    #[test]
    fn test_string_variable() {
        let mut ctx = Context::new();
        ctx.eval(r#"s = "test""#).unwrap();
        let result = ctx.eval("s").unwrap();
        assert_eq!(result.as_string(), Some("test"));
    }

    #[test]
    fn test_print_string() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"print("hello")"#).unwrap();
        assert_eq!(result, Value::None);
    }

    #[test]
    fn test_print_int() {
        let mut ctx = Context::new();
        let result = ctx.eval("print(42)").unwrap();
        assert_eq!(result, Value::None);
    }

    #[test]
    fn test_float_literal() {
        let mut ctx = Context::new();
        let result = ctx.eval("3.14").unwrap();
        assert_eq!(result.as_float(), Some(3.14));
    }

    #[test]
    fn test_float_arithmetic() {
        let mut ctx = Context::new();
        let result = ctx.eval("3.14 * 2.0").unwrap();
        assert_eq!(result.as_float(), Some(6.28));
    }

    #[test]
    fn test_mixed_int_float() {
        let mut ctx = Context::new();
        let result = ctx.eval("10 + 3.5").unwrap();
        assert_eq!(result.as_float(), Some(13.5));

        let result = ctx.eval("10 / 3.0").unwrap();
        assert!((result.as_float().unwrap() - 3.333333333333333).abs() < 0.0001);
    }

    #[test]
    fn test_int_conversion() {
        let mut ctx = Context::new();
        ctx.eval("x = 3.14").unwrap();
        let result = ctx.eval("int(x)").unwrap();
        assert_eq!(result.as_int(), Some(3));

        let result = ctx.eval("int(3.9)").unwrap();
        assert_eq!(result.as_int(), Some(3));
    }

    #[test]
    fn test_float_conversion() {
        let mut ctx = Context::new();
        let result = ctx.eval("float(42)").unwrap();
        assert_eq!(result.as_float(), Some(42.0));

        ctx.eval("x = 10").unwrap();
        let result = ctx.eval("float(x)").unwrap();
        assert_eq!(result.as_float(), Some(10.0));
    }
}
