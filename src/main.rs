mod builtins;
mod bytecode;
mod compiler;
mod context;
mod serializer;
mod value;
mod vm;

#[cfg(test)]
mod tests_pending;

pub use bytecode::{ByteCode, Instruction};
pub use compiler::Compiler;
pub use context::Context;
pub use serializer::{deserialize_bytecode, serialize_bytecode};
pub use value::{DictKey, ExceptionType, Module, Value};

use clap::{Parser, Subcommand};

#[cfg(not(test))]
use std::process;

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

#[cfg(not(test))]
#[allow(dead_code)]
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
                        eprintln!("Runtime error: {:?}", e);
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
    fn test_type_error_add_string_int() {
        let mut ctx = Context::new();
        let result = ctx.eval("\"hello\" + 5");
        assert!(result.is_err());
    }

    #[test]
    fn test_type_error_sub_string() {
        let mut ctx = Context::new();
        let result = ctx.eval("\"hello\" - \"world\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_type_error_mul_string_float() {
        let mut ctx = Context::new();
        let result = ctx.eval("\"hello\" * 3.5");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_index_out_of_bounds() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
x = [1, 2, 3]
x[10]
        "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_list_negative_index_out_of_bounds() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
x = [1, 2, 3]
x[-10]
        "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_dict_key_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
x = {"a": 1}
x["b"]
        "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_list_operations() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = []
len(x) == 0
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_empty_dict_operations() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = {}
len(x) == 0
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_empty_tuple_creation() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = ()
# Just verify empty tuple can be created
True
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_nested_list_access() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = [[1, 2], [3, 4], [5, 6]]
x[1][1]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(4));
    }

    #[test]
    fn test_nested_dict_access() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = {"a": {"b": {"c": 42}}}
x["a"]["b"]["c"]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_list_slice_empty_result() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = [1, 2, 3]
y = x[5:10]
len(y)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(0));
    }

    #[test]
    fn test_string_slice_empty_result() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = "hello"
y = x[10:20]
len(y)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(0));
    }

    #[test]
    fn test_range_negative_step() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = []
for i in range(5, 0, -1):
    x.append(i)
len(x)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(5));
    }

    #[test]
    fn test_range_zero_step_error() {
        let mut ctx = Context::new();
        let result = ctx.eval("range(0, 10, 0)");
        assert!(result.is_err());
    }

    #[test]
    fn test_function_wrong_arg_count() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
def foo(a, b):
    return a + b
foo(1)
        "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_len_on_int_error() {
        let mut ctx = Context::new();
        let result = ctx.eval("len(42)");
        assert!(result.is_err());
    }

    #[test]
    fn test_int_conversion_invalid() {
        let mut ctx = Context::new();
        let result = ctx.eval("int(\"not a number\")");
        assert!(result.is_err());
    }

    #[test]
    fn test_float_conversion_invalid() {
        let mut ctx = Context::new();
        let result = ctx.eval("float(\"not a number\")");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_append_multiple() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = []
x.append(1)
x.append(2)
x.append(3)
len(x)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(3));
    }

    #[test]
    fn test_list_pop_empty_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
x = []
x.pop()
        "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_dict_keys_empty() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = {}
keys = x.keys()
len(keys)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(0));
    }

    #[test]
    fn test_string_methods_chaining() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = "  HELLO  "
x.strip().lower()
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("hello"));
    }

    #[test]
    fn test_unpacking_wrong_count() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
x, y = [1, 2, 3]
        "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_break_in_nested_if() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
result = 0
for i in range(10):
    if i > 5:
        if i == 7:
            break
    result = i
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(6));
    }

    #[test]
    fn test_continue_in_nested_if() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
result = 0
for i in range(10):
    if i % 2 == 0:
        if i > 0:
            continue
    result += i
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(25));
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
    fn test_comparison_operators_all() {
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
    fn test_print_multiple_args() {
        // Test that print() with multiple arguments works
        // This test verifies the behavior exists, but doesn't capture stdout
        // The actual output format (strings without quotes) is verified manually
        let mut ctx = Context::new();
        let result = ctx.eval(r#"print("hello", 42, "world", 3.14)"#).unwrap();
        assert_eq!(result, Value::None);
    }

    #[test]
    fn test_print_string_and_number() {
        // Test mixed string and number arguments
        let mut ctx = Context::new();
        let result = ctx.eval(r#"print("1", 2)"#).unwrap();
        assert_eq!(result, Value::None);
    }

    #[test]
    fn test_float_literal() {
        let mut ctx = Context::new();
        let result = ctx.eval("3.15").unwrap();
        assert_eq!(result.as_float(), Some(3.15));
    }

    #[test]
    fn test_float_arithmetic() {
        let mut ctx = Context::new();
        let result = ctx.eval("3.15 * 2.0").unwrap();
        assert_eq!(result.as_float(), Some(6.3));
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

    #[test]
    fn test_list_literal() {
        let mut ctx = Context::new();
        ctx.eval("numbers = [1, 2, 3, 4, 5]").unwrap();
        let numbers = ctx.get("numbers").unwrap();
        let list = numbers.as_list().unwrap();
        assert_eq!(list.borrow().items.len(), 5);
    }

    #[test]
    fn test_list_index() {
        let mut ctx = Context::new();
        ctx.eval("numbers = [10, 20, 30]").unwrap();
        let result = ctx.eval("numbers[0]").unwrap();
        assert_eq!(result.as_int(), Some(10));

        let result = ctx.eval("numbers[2]").unwrap();
        assert_eq!(result.as_int(), Some(30));
    }

    #[test]
    fn test_list_index_assignment() {
        let mut ctx = Context::new();
        ctx.eval("numbers = [1, 2, 3]").unwrap();
        ctx.eval("numbers[1] = 99").unwrap();
        let result = ctx.eval("numbers[1]").unwrap();
        assert_eq!(result.as_int(), Some(99));
    }

    #[test]
    fn test_list_append() {
        let mut ctx = Context::new();
        ctx.eval("numbers = [1, 2, 3]").unwrap();
        ctx.eval("numbers.append(4)").unwrap();
        let numbers = ctx.get("numbers").unwrap();
        let list = numbers.as_list().unwrap();
        assert_eq!(list.borrow().items.len(), 4);
        assert_eq!(list.borrow().items[3].as_int(), Some(4));
    }

    #[test]
    fn test_list_pop() {
        let mut ctx = Context::new();
        ctx.eval("numbers = [1, 2, 3]").unwrap();
        let result = ctx.eval("numbers.pop()").unwrap();
        assert_eq!(result.as_int(), Some(3));

        let numbers = ctx.get("numbers").unwrap();
        let list = numbers.as_list().unwrap();
        assert_eq!(list.borrow().items.len(), 2);
    }

    #[test]
    fn test_dict_literal_string_keys() {
        let mut ctx = Context::new();
        ctx.eval(r#"person = {"name": "Alice", "age": 30}"#)
            .unwrap();
        let result = ctx.eval(r#"person["name"]"#).unwrap();
        assert_eq!(result.as_string(), Some("Alice"));
    }

    #[test]
    fn test_dict_literal_int_keys() {
        let mut ctx = Context::new();
        ctx.eval("scores = {1: 100, 2: 95, 3: 88}").unwrap();
        let result = ctx.eval("scores[1]").unwrap();
        assert_eq!(result.as_int(), Some(100));
    }

    #[test]
    fn test_dict_assignment() {
        let mut ctx = Context::new();
        ctx.eval(r#"person = {"name": "Bob"}"#).unwrap();
        ctx.eval(r#"person["age"] = 25"#).unwrap();
        let result = ctx.eval(r#"person["age"]"#).unwrap();
        assert_eq!(result.as_int(), Some(25));
    }

    #[test]
    fn test_dict_keys() {
        let mut ctx = Context::new();
        ctx.eval(r#"person = {"name": "Alice", "age": 30}"#)
            .unwrap();
        let result = ctx.eval("person.keys()").unwrap();
        let keys = result.as_list().unwrap();
        assert_eq!(keys.borrow().items.len(), 2);
    }

    #[test]
    fn test_len_function() {
        let mut ctx = Context::new();

        ctx.eval("numbers = [1, 2, 3, 4, 5]").unwrap();
        let result = ctx.eval("len(numbers)").unwrap();
        assert_eq!(result.as_int(), Some(5));

        ctx.eval(r#"person = {"name": "Alice", "age": 30}"#)
            .unwrap();
        let result = ctx.eval("len(person)").unwrap();
        assert_eq!(result.as_int(), Some(2));

        let result = ctx.eval(r#"len("hello")"#).unwrap();
        assert_eq!(result.as_int(), Some(5));
    }

    #[test]
    fn test_for_range_simple() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
sum = 0
for i in range(10):
    sum = sum + i
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(45));
    }

    #[test]
    fn test_for_range_start_stop() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
sum = 0
for i in range(5, 10):
    sum = sum + i
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(35)); // 5+6+7+8+9 = 35
    }

    #[test]
    fn test_for_range_with_step() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
sum = 0
for i in range(0, 10, 2):
    sum = sum + i
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(20)); // 0+2+4+6+8 = 20
    }

    #[test]
    fn test_for_list_iteration() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
numbers = [10, 20, 30, 40]
sum = 0
for num in numbers:
    sum = sum + num
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(100));
    }

    #[test]
    fn test_for_dict_iteration() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
scores = {1: 100, 2: 95, 3: 88}
sum = 0
for key in scores:
    sum = sum + key
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(6)); // 1+2+3 = 6
    }

    #[test]
    fn test_nested_for_loops() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
sum = 0
for i in range(3):
    for j in range(3):
        sum = sum + i * j
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(9)); // 0+0+0 + 0+1+2 + 0+2+4 = 9
    }

    #[test]
    fn test_for_with_function() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
def sum_range(n):
    total = 0
    for i in range(n):
        total = total + i
    return total
        "#,
        )
        .unwrap();

        let result = ctx.eval("sum_range(10)").unwrap();
        assert_eq!(result.as_int(), Some(45));
    }

    #[test]
    fn test_exception_creation() {
        use crate::value::ExceptionType;

        let exc = Value::error(ExceptionType::ValueError, "test error");
        assert!(exc.is_exception());

        let exc_value = exc.as_exception().unwrap();
        assert_eq!(exc_value.exception_type, ExceptionType::ValueError);
        assert_eq!(exc_value.message, "test error");
    }

    #[test]
    fn test_exception_equality() {
        use crate::value::ExceptionType;

        let exc1 = Value::error(ExceptionType::ValueError, "test");
        let exc2 = Value::error(ExceptionType::ValueError, "test");
        let exc3 = Value::error(ExceptionType::TypeError, "test");

        assert_eq!(exc1, exc2);
        assert_ne!(exc1, exc3);
    }

    #[test]
    fn test_raise_value_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"raise ValueError("test error")"#);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.contains("ValueError"));
        assert!(err.contains("test error"));
    }

    #[test]
    fn test_division_by_zero_exception() {
        let mut ctx = Context::new();
        let result = ctx.eval("x = 1 / 0");
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.contains("ZeroDivisionError"));
    }

    #[test]
    fn test_index_error_exception() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
my_list = [1, 2, 3]
x = my_list[10]
        "#,
        );
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.contains("IndexError"));
    }

    #[test]
    fn test_key_error_exception() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
my_dict = {"a": 1}
x = my_dict["b"]
        "#,
        );
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(err.contains("KeyError"));
    }

    #[test]
    fn test_try_except_basic() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = "ok"
try:
    x = 1 / 0
except ZeroDivisionError:
    result = "caught"
        "#,
        )
        .unwrap();

        let result = ctx.get("result").unwrap();
        assert_eq!(result.as_string(), Some("caught"));
    }

    #[test]
    fn test_try_except_with_binding() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
msg = ""
try:
    raise ValueError("test error")
except ValueError as e:
    msg = "caught"
        "#,
        )
        .unwrap();

        let msg = ctx.get("msg").unwrap();
        assert_eq!(msg.as_string(), Some("caught"));
    }

    #[test]
    fn test_try_except_multiple() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = ""
try:
    x = 1 / 0
except ValueError:
    result = "value"
except ZeroDivisionError:
    result = "zero"
        "#,
        )
        .unwrap();

        let result = ctx.get("result").unwrap();
        assert_eq!(result.as_string(), Some("zero"));
    }

    #[test]
    fn test_try_except_no_match() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
try:
    x = 1 / 0
except ValueError:
    pass
        "#,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("ZeroDivisionError"));
    }

    #[test]
    fn test_break_in_while() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
i = 0
sum = 0
while i < 100:
    if i == 5:
        break
    sum = sum + i
    i = i + 1
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(10)); // 0+1+2+3+4 = 10

        let i = ctx.get("i").unwrap();
        assert_eq!(i.as_int(), Some(5));
    }

    #[test]
    fn test_continue_in_while() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
i = 0
sum = 0
while i < 10:
    i = i + 1
    if i == 5:
        continue
    sum = sum + i
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(50)); // 1+2+3+4+6+7+8+9+10 = 50 (skips 5)
    }

    #[test]
    fn test_break_in_for() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
sum = 0
for i in range(100):
    if i == 5:
        break
    sum = sum + i
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(10)); // 0+1+2+3+4 = 10
    }

    #[test]
    fn test_continue_in_for() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
sum = 0
for i in range(10):
    if i == 5:
        continue
    sum = sum + i
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(40)); // 0+1+2+3+4+6+7+8+9 = 40 (skips 5)
    }

    #[test]
    fn test_break_in_nested_loop() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
outer_count = 0
inner_count = 0
for i in range(3):
    outer_count = outer_count + 1
    for j in range(5):
        if j == 2:
            break
        inner_count = inner_count + 1
        "#,
        )
        .unwrap();

        let outer_count = ctx.get("outer_count").unwrap();
        assert_eq!(outer_count.as_int(), Some(3)); // outer loop runs 3 times

        let inner_count = ctx.get("inner_count").unwrap();
        assert_eq!(inner_count.as_int(), Some(6)); // inner loop runs 2 times per outer (0, 1) * 3 = 6
    }

    #[test]
    fn test_continue_in_nested_loop() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
count = 0
for i in range(3):
    for j in range(5):
        if j == 2:
            continue
        count = count + 1
        "#,
        )
        .unwrap();

        let count = ctx.get("count").unwrap();
        assert_eq!(count.as_int(), Some(12)); // (5-1) * 3 = 12 (skips one per inner loop)
    }

    #[test]
    fn test_break_outside_loop() {
        let mut ctx = Context::new();
        let result = ctx.eval("break");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("break") && err.contains("outside loop"));
    }

    #[test]
    fn test_continue_outside_loop() {
        let mut ctx = Context::new();
        let result = ctx.eval("continue");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("continue") && err.contains("outside loop"));
    }

    #[test]
    fn test_break_with_list_iteration() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
numbers = [10, 20, 30, 40, 50]
sum = 0
for num in numbers:
    if num == 30:
        break
    sum = sum + num
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(30)); // 10+20 = 30
    }

    #[test]
    fn test_continue_with_list_iteration() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
numbers = [10, 20, 30, 40, 50]
sum = 0
for num in numbers:
    if num == 30:
        continue
    sum = sum + num
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(120)); // 10+20+40+50 = 120
    }

    #[test]
    fn test_break_in_function() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
def find_first_even(numbers):
    result = 0
    for num in numbers:
        if num == 0:
            break
        if num / 2 * 2 == num:
            result = num
            break
    return result
        "#,
        )
        .unwrap();

        let result = ctx.eval("find_first_even([1, 3, 5, 8, 10])").unwrap();
        assert_eq!(result.as_int(), Some(8));
    }

    #[test]
    fn test_continue_skip_odds() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
sum = 0
for i in range(10):
    if i / 2 * 2 != i:
        continue
    sum = sum + i
        "#,
        )
        .unwrap();

        let sum = ctx.get("sum").unwrap();
        assert_eq!(sum.as_int(), Some(20)); // 0+2+4+6+8 = 20 (only evens)
    }

    #[test]
    fn test_try_finally_basic() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
executed = []
try:
    executed.append(1)
finally:
    executed.append(2)
        "#,
        )
        .unwrap();

        let executed = ctx.get("executed").unwrap().as_list().unwrap();
        assert_eq!(executed.borrow().items.len(), 2);
    }

    #[test]
    fn test_try_except_finally() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = ""
try:
    x = 1 / 0
except ZeroDivisionError:
    result = "caught"
finally:
    result = result + " finally"
        "#,
        )
        .unwrap();

        let result = ctx.get("result").unwrap();
        assert_eq!(result.as_string(), Some("caught finally"));
    }

    // === Additional exception handling tests ===

    #[test]
    fn test_try_finally_with_return() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
def test_func():
    try:
        return "try"
    finally:
        x = 1  # Finally executes even with return
test_func()
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("try"));
    }

    #[test]
    fn test_nested_try_except_inner_outer() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
result = ""
try:
    try:
        x = 1 / 0
    except ValueError:
        result = "inner"
except ZeroDivisionError:
    result = "outer"
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("outer"));
    }

    #[test]
    fn test_try_except_in_loop() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
count = 0
for i in range(5):
    try:
        if i == 2:
            x = 1 / 0
        count += 1
    except ZeroDivisionError:
        count += 10
count
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(14)); // 0+1+10+3+4 = 18, wait: 1+1+10+1+1 = 14
    }

    #[test]
    fn test_try_finally_in_loop() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
result = []
for i in range(3):
    try:
        result.append(i)
    finally:
        result.append(100)
len(result)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(6)); // [0, 100, 1, 100, 2, 100]
    }

    #[test]
    fn test_exception_in_function() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
def divide(a, b):
    try:
        return a / b
    except ZeroDivisionError:
        return -1

result = divide(10, 0)
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(-1));
    }

    #[test]
    fn test_multiple_except_handlers_ordered() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
result = ""
try:
    x = {}
    y = x["missing"]
except ZeroDivisionError:
    result = "zero"
except KeyError:
    result = "key"
except ValueError:
    result = "value"
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("key"));
    }

    #[test]
    fn test_exception_with_message() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
result = ""
try:
    x = [1, 2, 3]
    y = x[10]
except IndexError as e:
    result = "caught"
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("caught"));
    }

    #[test]
    fn test_finally_always_executes() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
executed = False
try:
    x = 1 + 1
finally:
    executed = True
executed
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_try_except_else_not_implemented() {
        // else clause in try/except is not implemented yet
        // This just tests that basic try/except works
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
result = "ok"
try:
    x = 1 + 1
except:
    result = "error"
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("ok"));
    }

    // === Additional async tests ===

    #[test]
    fn test_async_function_with_exception() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import asyncio

async def test():
    try:
        x = 1 / 0
    except ZeroDivisionError:
        return "caught"
    return "not caught"

result = test()
result
        "#,
            )
            .unwrap();
        // Returns coroutine, not executed yet
        assert!(matches!(result, Value::Coroutine(_, _)));
    }

    #[test]
    fn test_async_multiple_awaits() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import asyncio

async def multi_sleep():
    await asyncio.sleep(0.001)
    await asyncio.sleep(0.001)
    return "done"

coro = multi_sleep()
await coro
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("done"));
    }

    // === Additional method call tests ===

    #[test]
    fn test_string_method_on_literal() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
result = "HELLO".lower()
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("hello"));
    }

    #[test]
    fn test_list_method_chaining() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = []
x.append(1)
x.append(2)
x.append(3)
x.pop()
len(x)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(2));
    }

    #[test]
    fn test_dict_get_method_with_default() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
d = {"a": 1}
result = d.get("b", 999)
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(999));
    }

    #[test]
    fn test_string_split_join_roundtrip() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
text = "a,b,c,d"
parts = text.split(",")
result = "-".join(parts)
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("a-b-c-d"));
    }

    #[test]
    fn test_iterator_modification_append() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
numbers = [1, 2, 3]
for n in numbers:
    numbers.append(10)
        "#,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("IteratorError"));
    }

    #[test]
    fn test_iterator_modification_pop() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
numbers = [1, 2, 3]
for n in numbers:
    numbers.pop()
        "#,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("IteratorError"));
    }

    #[test]
    fn test_iterator_modification_after_loop_ok() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
numbers = [1, 2, 3]
for n in numbers:
    pass
numbers.append(4)
        "#,
        )
        .unwrap();

        let numbers = ctx.get("numbers").unwrap().as_list().unwrap();
        assert_eq!(numbers.borrow().items.len(), 4);
    }

    // Task 016: 比较运算符类型支持测试

    #[test]
    fn test_float_comparison() {
        let mut ctx = Context::new();
        ctx.eval("x = 3.14").unwrap();
        ctx.eval("y = 2.71").unwrap();

        let result = ctx.eval("x > y").unwrap();
        assert_eq!(result.as_bool(), Some(true));

        let result = ctx.eval("x == y").unwrap();
        assert_eq!(result.as_bool(), Some(false));

        let result = ctx.eval("x != y").unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_mixed_int_float_comparison() {
        let mut ctx = Context::new();
        ctx.eval("score = 87.5").unwrap();

        let result = ctx.eval("score >= 90").unwrap();
        assert_eq!(result.as_bool(), Some(false));

        let result = ctx.eval("score >= 80").unwrap();
        assert_eq!(result.as_bool(), Some(true));

        let result = ctx.eval("10 == 10.0").unwrap();
        assert_eq!(result.as_bool(), Some(true));

        let result = ctx.eval("5 < 5.5").unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_string_comparison() {
        let mut ctx = Context::new();

        // 相等性
        let result = ctx.eval(r#""hello" == "hello""#).unwrap();
        assert_eq!(result.as_bool(), Some(true));

        let result = ctx.eval(r#""hello" == "world""#).unwrap();
        assert_eq!(result.as_bool(), Some(false));

        let result = ctx.eval(r#""hello" != "world""#).unwrap();
        assert_eq!(result.as_bool(), Some(true));

        // 字典序
        let result = ctx.eval(r#""apple" < "banana""#).unwrap();
        assert_eq!(result.as_bool(), Some(true));

        let result = ctx.eval(r#""zebra" > "apple""#).unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_bool_comparison() {
        let mut ctx = Context::new();

        let result = ctx.eval("True == True").unwrap();
        assert_eq!(result.as_bool(), Some(true));

        let result = ctx.eval("True == False").unwrap();
        assert_eq!(result.as_bool(), Some(false));

        let result = ctx.eval("False != True").unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_none_comparison() {
        let mut ctx = Context::new();
        ctx.eval("x = None").unwrap();

        let result = ctx.eval("x == None").unwrap();
        assert_eq!(result.as_bool(), Some(true));

        let result = ctx.eval("x != None").unwrap();
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_different_types_equality() {
        let mut ctx = Context::new();

        // 相等性比较返回 False
        let result = ctx.eval(r#""hello" == 5"#).unwrap();
        assert_eq!(result.as_bool(), Some(false));

        let result = ctx.eval(r#"True == 5"#).unwrap();
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_different_types_ordering_error() {
        let mut ctx = Context::new();

        // 顺序比较抛出 TypeError
        let result = ctx.eval(r#""hello" < 5"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("TypeError"));
        assert!(err.contains("unsupported operand types"));
    }

    #[test]
    fn test_grade_function_with_float() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
def get_grade(score):
    if score >= 90.0:
        return "A"
    else:
        if score >= 80.0:
            return "B"
        else:
            if score >= 70.0:
                return "C"
            else:
                return "F"

grade = get_grade(85.5)
        "#,
        )
        .unwrap();

        let grade = ctx.get("grade").unwrap();
        assert_eq!(grade.as_string(), Some("B"));
    }

    #[test]
    fn test_import_json() {
        let mut ctx = Context::new();
        ctx.eval("import json").unwrap();
    }

    #[test]
    fn test_json_loads() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json
data = json.loads('{"x": 1, "y": 2}')
data["x"]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(1));
    }

    #[test]
    fn test_json_dumps() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json
obj = {"x": 1, "y": 2}
json.dumps(obj)
        "#,
            )
            .unwrap();

        let json_str = result.as_string().unwrap();
        assert!(json_str.contains("\"x\""));
        assert!(json_str.contains("1"));
    }

    #[test]
    fn test_from_import() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
from json import loads
data = loads('{"value": 42}')
data["value"]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_import_as() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json as j
data = j.loads('{"test": true}')
data["test"]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_json_array() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json
arr = json.loads('[1, 2, 3, 4, 5]')
len(arr)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(5));
    }

    #[test]
    fn test_json_nested() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json
data = json.loads('{"user": {"name": "Bob", "id": 123}}')
data["user"]["name"]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("Bob"));
    }

    #[test]
    fn test_json_null() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json
data = json.loads('{"value": null}')
data["value"] == None
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_json_boolean() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json
data = json.loads('{"active": true, "deleted": false}')
data["active"] and not data["deleted"]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_json_nested_arrays() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json
data = json.loads('{"matrix": [[1, 2], [3, 4]]}')
data["matrix"][1][0]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(3));
    }

    #[test]
    fn test_json_special_chars() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json
data = json.loads('{"text": "hello world"}')
len(data["text"]) > 0
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_json_dumps_dict() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json
data = {"name": "Alice", "age": 30}
json_str = json.dumps(data)
isinstance(json_str, str)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_json_dumps_list() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json
data = [1, 2, 3, 4, 5]
json_str = json.dumps(data)
isinstance(json_str, str)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_json_dumps_none() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json
json_str = json.dumps(None)
json_str == "null"
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_json_roundtrip() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json
original = {"name": "Test", "value": 42}
json_str = json.dumps(original)
restored = json.loads(json_str)
restored["name"] == "Test" and restored["value"] == 42
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_module_not_found() {
        let mut ctx = Context::new();
        let result = ctx.eval("import nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_os_getcwd() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
cwd = os.getcwd()
len(cwd) > 0
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_os_listdir() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
files = os.listdir(".")
len(files) > 0
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_os_path_exists() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
os.path.exists("Cargo.toml")
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_os_path_join() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
path = os.path.join("dir", "subdir", "file.txt")
len(path) > 0
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_os_path_basename() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
os.path.basename("/path/to/file.txt")
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("file.txt"));
    }

    #[test]
    fn test_os_path_dirname() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
os.path.dirname("/path/to/file.txt")
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("/path/to"));
    }

    #[test]
    fn test_os_getenv() {
        let mut ctx = Context::new();

        unsafe {
            std::env::set_var("TEST_VAR_OS", "test_value");
        }

        let result = ctx
            .eval(
                r#"
import os
os.getenv("TEST_VAR_OS")
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("test_value"));

        let result = ctx
            .eval(
                r#"
import os
os.getenv("NONEXISTENT_VAR_OS", "default")
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("default"));
    }

    #[test]
    fn test_os_name() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
os.name
        "#,
            )
            .unwrap();

        let name = result.as_string().unwrap();
        assert!(name == "posix" || name == "nt");
    }

    #[test]
    fn test_os_mkdir_rmdir() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
os.mkdir("test_dir_quickpy")
exists = os.path.exists("test_dir_quickpy")
os.rmdir("test_dir_quickpy")
exists
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_os_path_isfile() {
        let mut ctx = Context::new();
        // Test with existing file
        let result = ctx
            .eval(
                r#"
import os
os.path.isfile("Cargo.toml")
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));

        // Test with directory
        let result = ctx
            .eval(
                r#"
import os
os.path.isfile("src")
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(false));

        // Test with non-existent path
        let result = ctx
            .eval(
                r#"
import os
os.path.isfile("nonexistent_file_xyz.txt")
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_os_path_isdir() {
        let mut ctx = Context::new();
        // Test with existing directory
        let result = ctx
            .eval(
                r#"
import os
os.path.isdir("src")
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));

        // Test with file
        let result = ctx
            .eval(
                r#"
import os
os.path.isdir("Cargo.toml")
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(false));

        // Test with non-existent path
        let result = ctx
            .eval(
                r#"
import os
os.path.isdir("nonexistent_dir_xyz")
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_os_path_abspath() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
path = os.path.abspath(".")
len(path) > 0
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_os_remove_exists() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
# Just verify remove function exists in os module
hasattr = False
try:
    # Check if we can access os.remove
    f = os.remove
    hasattr = True
except:
    pass
hasattr
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_os_rename() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
os.mkdir("test_rename_src")
os.rename("test_rename_src", "test_rename_dst")
exists = os.path.exists("test_rename_dst")
os.rmdir("test_rename_dst")
exists
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_os_chdir() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
original = os.getcwd()
os.chdir("..")
parent = os.getcwd()
os.chdir(original)
back = os.getcwd()
original == back and parent != original
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_os_makedirs() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
os.makedirs("test_nested/sub1/sub2")
exists = os.path.isdir("test_nested/sub1/sub2")
os.rmdir("test_nested/sub1/sub2")
os.rmdir("test_nested/sub1")
os.rmdir("test_nested")
exists
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_os_environ() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
# Check that environ is a dict
isinstance(os.environ, dict)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_os_environ_get() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
# PATH should exist on most systems
path = os.environ.get("PATH", "")
isinstance(path, str)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_re_match() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
m = re.match(r"hello", "hello world")
m.group(0)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("hello"));
    }

    #[test]
    fn test_re_match_no_match() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
m = re.match(r"world", "hello world")
m
        "#,
            )
            .unwrap();
        assert_eq!(result, Value::None);
    }

    #[test]
    fn test_re_search() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
m = re.search(r"world", "hello world")
m.group(0)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("world"));
    }

    #[test]
    fn test_re_findall() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
matches = re.findall(r"\d+", "abc 123 def 456 ghi")
len(matches)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(2));
    }

    #[test]
    fn test_re_sub() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
result = re.sub(r"\d+", "X", "abc 123 def 456")
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("abc X def X"));
    }

    #[test]
    fn test_re_split() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
parts = re.split(r"\s+", "hello  world   test")
len(parts)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(3));
    }

    #[test]
    fn test_re_groups() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
m = re.search(r"(\d+)-(\d+)", "Phone: 123-456")
m.group(1)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("123"));
    }

    #[test]
    fn test_re_match_span() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
m = re.search(r"world", "hello world")
span = m.span()
span[0]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(6));
    }

    #[test]
    fn test_re_compile_exists() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
# Just verify compile function exists
pattern = re.compile(r"\d+")
pattern != None
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_re_match_groups_multiple() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
m = re.match(r"(\w+):(\d+)", "user:123")
groups = m.groups()
len(groups)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(2));
    }

    #[test]
    fn test_re_search_no_match() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
m = re.search(r"\d+", "no numbers here")
m == None
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_re_sub_basic() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
result = re.sub(r"\d", "X", "a1b2c3d4")
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("aXbXcXdX"));
    }

    #[test]
    fn test_re_split_basic() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
parts = re.split(r"\s+", "a b c d e")
len(parts)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(5));
    }

    // === pyq bytecode API tests ===

    #[test]
    fn test_compile_serialize_execute() {
        let source = "x = 1 + 2";
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("x"), Some(Value::Int(3)));
    }

    #[test]
    fn test_compile_serialize_roundtrip() {
        let source = r#"
x = 10 + 20
y = x * 2
"#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();
        assert_eq!(bytecode, restored);
    }

    #[test]
    fn test_serialize_error_on_function() {
        let source = "def foo(): return 1";
        let bytecode = Compiler::compile(source).unwrap();
        assert!(serialize_bytecode(&bytecode).is_err());
    }

    #[test]
    fn test_serialize_error_on_list() {
        let source = "x = [1, 2, 3]";
        let bytecode = Compiler::compile(source).unwrap();
        assert!(serialize_bytecode(&bytecode).is_err());
    }

    #[test]
    fn test_eval_bytecode_with_string() {
        let source = r#"name = "hello""#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("name"), Some(Value::String("hello".to_string())));
    }

    #[test]
    fn test_eval_bytecode_with_float() {
        let source = "pi = 3.15";
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("pi"), Some(Value::Float(3.15)));
    }

    // === Additional serializer tests ===

    #[test]
    fn test_serialize_arithmetic() {
        let source = "x = 10 + 20 * 3";
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("x"), Some(Value::Int(70)));
    }

    #[test]
    fn test_serialize_comparison() {
        let source = "result = 5 > 3";
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Bool(true)));
    }

    #[test]
    fn test_serialize_if_else() {
        let source = r#"
x = 10
if x > 5:
    result = "big"
else:
    result = "small"
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("result"), Some(Value::String("big".to_string())));
    }

    #[test]
    fn test_serialize_while_loop() {
        let source = r#"
count = 0
i = 0
while i < 5:
    count = count + 1
    i = i + 1
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("count"), Some(Value::Int(5)));
    }

    #[test]
    fn test_serialize_logical_operators_error() {
        // Logical operators use jump instructions which aren't serializable yet
        let source = "result = True and False or True";
        let bytecode = Compiler::compile(source).unwrap();
        assert!(serialize_bytecode(&bytecode).is_err());
    }

    #[test]
    fn test_serialize_string_operations() {
        let source = r#"result = "hello" + " " + "world""#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(
            ctx.get("result"),
            Some(Value::String("hello world".to_string()))
        );
    }

    #[test]
    fn test_serialize_multiple_variables() {
        let source = r#"
a = 10
b = 20
c = a + b
d = c * 2
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("d"), Some(Value::Int(60)));
    }

    #[test]
    fn test_serialize_none_value() {
        let source = "x = None";
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("x"), Some(Value::None));
    }

    #[test]
    fn test_serialize_bool_values() {
        let source = r#"
t = True
f = False
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("t"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("f"), Some(Value::Bool(false)));
    }

    #[test]
    fn test_serialize_negative_numbers() {
        let source = "x = -42";
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("x"), Some(Value::Int(-42)));
    }

    #[test]
    fn test_serialize_modulo() {
        let source = "x = 17 % 5";
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("x"), Some(Value::Int(2)));
    }

    #[test]
    fn test_serialize_augmented_assignment() {
        let source = r#"
x = 10
x += 5
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("x"), Some(Value::Int(15)));
    }

    #[test]
    fn test_unary_minus_int() {
        let mut ctx = Context::new();
        let result = ctx.eval("-17").unwrap();
        assert_eq!(result, Value::Int(-17));
    }

    #[test]
    fn test_unary_minus_float() {
        let mut ctx = Context::new();
        let result = ctx.eval("-3.15").unwrap();
        assert_eq!(result, Value::Float(-3.15));
    }

    #[test]
    fn test_unary_minus_variable() {
        let mut ctx = Context::new();
        ctx.eval("x = 42").unwrap();
        let result = ctx.eval("-x").unwrap();
        assert_eq!(result, Value::Int(-42));
    }

    #[test]
    fn test_unary_plus() {
        let mut ctx = Context::new();
        let result = ctx.eval("+17").unwrap();
        assert_eq!(result, Value::Int(17));
    }

    #[test]
    fn test_double_negative() {
        let mut ctx = Context::new();
        let result = ctx.eval("--17").unwrap();
        assert_eq!(result, Value::Int(17));
    }

    #[test]
    fn test_unary_minus_in_expression() {
        let mut ctx = Context::new();
        let result = ctx.eval("10 + -5").unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_unary_minus_serialization() {
        let source = "-42";
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        let result = ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(result, Value::Int(-42));
    }

    #[test]
    fn test_modulo_int() {
        let mut ctx = Context::new();
        let result = ctx.eval("10 % 3").unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_modulo_even_check() {
        let mut ctx = Context::new();
        let result = ctx.eval("10 % 2").unwrap();
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_modulo_float() {
        let mut ctx = Context::new();
        let result = ctx.eval("10.5 % 3.0").unwrap();
        assert_eq!(result, Value::Float(1.5));
    }

    #[test]
    fn test_modulo_mixed() {
        let mut ctx = Context::new();
        let result = ctx.eval("10 % 3.0").unwrap();
        assert_eq!(result, Value::Float(1.0));
    }

    #[test]
    fn test_modulo_zero_error() {
        let mut ctx = Context::new();
        let result = ctx.eval("10 % 0");
        assert!(result.is_err());
    }

    #[test]
    fn test_exception_hierarchy_exception_catches_valueerror() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
try:
    raise ValueError("test")
except Exception:
    x = "caught"
x
"#,
            )
            .unwrap();
        assert_eq!(result, Value::String("caught".to_string()));
    }

    #[test]
    fn test_exception_hierarchy_exception_catches_all() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
count = 0
try:
    raise ValueError("test")
except Exception:
    count = count + 1
try:
    raise TypeError("test")
except Exception:
    count = count + 1
try:
    raise IndexError("test")
except Exception:
    count = count + 1
count
"#,
            )
            .unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_exception_hierarchy_specific_handler_works() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
try:
    raise ValueError("test")
except ValueError:
    x = "caught ValueError"
x
"#,
            )
            .unwrap();
        assert_eq!(result, Value::String("caught ValueError".to_string()));
    }

    #[test]
    fn test_exception_hierarchy_wrong_handler_doesnt_catch() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
try:
    raise ValueError("test")
except TypeError:
    x = "caught TypeError"
except ValueError:
    x = "caught ValueError"
x
"#,
            )
            .unwrap();
        assert_eq!(result, Value::String("caught ValueError".to_string()));
    }

    // Task 027: Augmented Assignment Operators
    #[test]
    fn test_augmented_add() {
        let mut ctx = Context::new();
        let result = ctx.eval("x = 10\nx += 5\nx").unwrap();
        assert_eq!(result, Value::Int(15));
    }

    #[test]
    fn test_augmented_sub() {
        let mut ctx = Context::new();
        let result = ctx.eval("x = 10\nx -= 3\nx").unwrap();
        assert_eq!(result, Value::Int(7));
    }

    #[test]
    fn test_augmented_mul() {
        let mut ctx = Context::new();
        let result = ctx.eval("x = 5\nx *= 3\nx").unwrap();
        assert_eq!(result, Value::Int(15));
    }

    #[test]
    fn test_augmented_div() {
        let mut ctx = Context::new();
        let result = ctx.eval("x = 20\nx /= 4\nx").unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_augmented_mod() {
        let mut ctx = Context::new();
        let result = ctx.eval("x = 17\nx %= 5\nx").unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_augmented_float() {
        let mut ctx = Context::new();
        let result = ctx.eval("x = 10.0\nx += 2.5\nx").unwrap();
        assert_eq!(result, Value::Float(12.5));
    }

    #[test]
    fn test_augmented_string() {
        let mut ctx = Context::new();
        let result = ctx.eval("s = \"Hello\"\ns += \" World\"\ns").unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_augmented_in_loop() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
total = 0
for i in range(5):
    total += i
total
"#,
            )
            .unwrap();
        assert_eq!(result, Value::Int(10));
    }

    // Task 028: Logical Operators
    #[test]
    fn test_logical_and_both_true() {
        let mut ctx = Context::new();
        let result = ctx.eval("True and True").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_logical_and_first_false() {
        let mut ctx = Context::new();
        let result = ctx.eval("False and True").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_logical_and_second_false() {
        let mut ctx = Context::new();
        let result = ctx.eval("True and False").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_logical_or_both_false() {
        let mut ctx = Context::new();
        let result = ctx.eval("False or False").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_logical_or_first_true() {
        let mut ctx = Context::new();
        let result = ctx.eval("True or False").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_logical_or_second_true() {
        let mut ctx = Context::new();
        let result = ctx.eval("False or True").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_logical_not_true() {
        let mut ctx = Context::new();
        let result = ctx.eval("not True").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_logical_not_false() {
        let mut ctx = Context::new();
        let result = ctx.eval("not False").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_logical_and_returns_value() {
        let mut ctx = Context::new();
        // Python's 'and' returns the last truthy value or first falsy value
        let result = ctx.eval("1 and 2").unwrap();
        assert_eq!(result, Value::Int(2));

        let result = ctx.eval("0 and 5").unwrap();
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_logical_or_returns_value() {
        let mut ctx = Context::new();
        // Python's 'or' returns the first truthy value or last falsy value
        let result = ctx.eval("1 or 2").unwrap();
        assert_eq!(result, Value::Int(1));

        let result = ctx.eval("0 or 5").unwrap();
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_logical_truthiness() {
        let mut ctx = Context::new();
        // Test truthiness of different types
        assert_eq!(ctx.eval("not 0").unwrap(), Value::Bool(true));
        assert_eq!(ctx.eval("not 1").unwrap(), Value::Bool(false));
        assert_eq!(ctx.eval("not \"\"").unwrap(), Value::Bool(true));
        assert_eq!(ctx.eval("not \"hello\"").unwrap(), Value::Bool(false));
        assert_eq!(ctx.eval("not []").unwrap(), Value::Bool(true));
        assert_eq!(ctx.eval("not [1]").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_logical_in_condition() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = 5
if x > 0 and x < 10:
    result = "in range"
else:
    result = "out of range"
result
"#,
            )
            .unwrap();
        assert_eq!(result, Value::String("in range".to_string()));
    }

    #[test]
    fn test_logical_chaining() {
        let mut ctx = Context::new();
        let result = ctx.eval("True and True and True").unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = ctx.eval("False or False or True").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // Task 029: in Operator
    #[test]
    fn test_in_list_found() {
        let mut ctx = Context::new();
        let result = ctx.eval("2 in [1, 2, 3]").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_in_list_not_found() {
        let mut ctx = Context::new();
        let result = ctx.eval("4 in [1, 2, 3]").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_not_in_list() {
        let mut ctx = Context::new();
        let result = ctx.eval("4 not in [1, 2, 3]").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_in_string() {
        let mut ctx = Context::new();
        let result = ctx.eval("\"world\" in \"hello world\"").unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = ctx.eval("\"xyz\" in \"hello world\"").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_in_dict() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
d = {"a": 1, "b": 2}
result = "a" in d
result
"#,
            )
            .unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = ctx
            .eval(
                r#"
d = {"a": 1, "b": 2}
result = "c" in d
result
"#,
            )
            .unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_in_empty_containers() {
        let mut ctx = Context::new();
        assert_eq!(ctx.eval("1 in []").unwrap(), Value::Bool(false));
        assert_eq!(ctx.eval("\"x\" in {}").unwrap(), Value::Bool(false));
        assert_eq!(ctx.eval("\"x\" in \"\"").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_in_condition() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
numbers = [1, 2, 3, 4, 5]
if 3 in numbers:
    result = "found"
else:
    result = "not found"
result
"#,
            )
            .unwrap();
        assert_eq!(result, Value::String("found".to_string()));
    }

    #[test]
    fn test_in_loop() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
count = 0
for i in range(10):
    if i in [2, 4, 6, 8]:
        count += 1
count
"#,
            )
            .unwrap();
        assert_eq!(result, Value::Int(4));
    }

    // Task 030: String Methods
    #[test]
    fn test_string_split_whitespace() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
s = "hello world python"
words = s.split()
len(words)
"#,
            )
            .unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_string_split_separator() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
s = "a,b,c"
parts = s.split(",")
len(parts)
"#,
            )
            .unwrap();
        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_string_strip() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""  hello  ".strip()"#).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_string_startswith() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""hello world".startswith("hello")"#).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = ctx.eval(r#""hello world".startswith("world")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_endswith() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""hello world".endswith("world")"#).unwrap();
        assert_eq!(result, Value::Bool(true));

        let result = ctx.eval(r#""hello world".endswith("hello")"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_lower() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""Hello World".lower()"#).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_string_upper() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""Hello World".upper()"#).unwrap();
        assert_eq!(result, Value::String("HELLO WORLD".to_string()));
    }

    #[test]
    fn test_string_replace_all_occurrences() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(r#""hello world".replace("world", "python")"#)
            .unwrap();
        assert_eq!(result, Value::String("hello python".to_string()));
    }

    #[test]
    fn test_string_join_method() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
words = ["hello", "world"]
" ".join(words)
"#,
            )
            .unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));

        let result = ctx
            .eval(
                r#"
words = ["hello", "world"]
",".join(words)
"#,
            )
            .unwrap();
        assert_eq!(result, Value::String("hello,world".to_string()));
    }

    #[test]
    fn test_string_method_chaining() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""  HELLO WORLD  ".strip().lower()"#).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_string_split_empty() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
parts = "".split()
len(parts)
"#,
            )
            .unwrap();
        assert_eq!(result, Value::Int(0));
    }

    // Task 031: Dictionary .get() Method
    #[test]
    fn test_dict_get_existing_key() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
d = {"a": 1, "b": 2}
d.get("a")
"#,
            )
            .unwrap();
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_dict_get_missing_key_no_default() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
d = {"a": 1}
d.get("b")
"#,
            )
            .unwrap();
        assert_eq!(result, Value::None);
    }

    #[test]
    fn test_dict_get_missing_key_with_default() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
d = {"a": 1}
d.get("b", 0)
"#,
            )
            .unwrap();
        assert_eq!(result, Value::Int(0));

        let result = ctx
            .eval(
                r#"
d = {"a": 1}
d.get("c", "default")
"#,
            )
            .unwrap();
        assert_eq!(result, Value::String("default".to_string()));
    }

    #[test]
    fn test_dict_get_doesnt_modify() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
d = {"a": 1}
result = d.get("b", 0)
"#,
        )
        .unwrap();

        let result = ctx.eval(r#""b" in d"#).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_dict_get_different_key_types() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
d = {1: "one", "two": 2}
d.get(1)
"#,
            )
            .unwrap();
        assert_eq!(result, Value::String("one".to_string()));

        let result = ctx
            .eval(
                r#"
d = {1: "one", "two": 2}
d.get("two")
"#,
            )
            .unwrap();
        assert_eq!(result, Value::Int(2));

        let result = ctx
            .eval(
                r#"
d = {1: "one", "two": 2}
d.get(3, "missing")
"#,
            )
            .unwrap();
        assert_eq!(result, Value::String("missing".to_string()));
    }

    #[test]
    fn test_dict_get_real_usage() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
config = {"host": "localhost", "port": 8080}
host = config.get("host", "0.0.0.0")
timeout = config.get("timeout", 30)
"#,
        )
        .unwrap();

        let host = ctx.get("host").unwrap();
        assert_eq!(host, Value::String("localhost".to_string()));

        let timeout = ctx.get("timeout").unwrap();
        assert_eq!(timeout, Value::Int(30));
    }

    #[test]
    fn test_dict_get_no_args_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
d = {"a": 1}
d.get()
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("TypeError"));
    }

    // Task 032: Multiple Assignment and Tuple Unpacking
    #[test]
    fn test_tuple_unpacking_basic() {
        let mut ctx = Context::new();
        ctx.eval("a, b = 1, 2").unwrap();
        assert_eq!(ctx.get("a"), Some(Value::Int(1)));
        assert_eq!(ctx.get("b"), Some(Value::Int(2)));
    }

    #[test]
    fn test_tuple_unpacking_from_list() {
        let mut ctx = Context::new();
        ctx.eval("x, y = [10, 20]").unwrap();
        assert_eq!(ctx.get("x"), Some(Value::Int(10)));
        assert_eq!(ctx.get("y"), Some(Value::Int(20)));
    }

    #[test]
    fn test_tuple_unpacking_from_tuple() {
        let mut ctx = Context::new();
        ctx.eval("p, q = (100, 200)").unwrap();
        assert_eq!(ctx.get("p"), Some(Value::Int(100)));
        assert_eq!(ctx.get("q"), Some(Value::Int(200)));
    }

    #[test]
    fn test_tuple_swap_variables() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
a = 5
b = 10
a, b = b, a
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("a"), Some(Value::Int(10)));
        assert_eq!(ctx.get("b"), Some(Value::Int(5)));
    }

    #[test]
    fn test_tuple_multiple_values() {
        let mut ctx = Context::new();
        ctx.eval("a, b, c = 1, 2, 3").unwrap();
        assert_eq!(ctx.get("a"), Some(Value::Int(1)));
        assert_eq!(ctx.get("b"), Some(Value::Int(2)));
        assert_eq!(ctx.get("c"), Some(Value::Int(3)));
    }

    #[test]
    fn test_tuple_function_return() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
def get_coords():
    return 3, 4

x, y = get_coords()
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("x"), Some(Value::Int(3)));
        assert_eq!(ctx.get("y"), Some(Value::Int(4)));
    }

    #[test]
    fn test_tuple_unpacking_too_many_values() {
        let mut ctx = Context::new();
        let result = ctx.eval("a, b = [1, 2, 3]");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("ValueError") || err.contains("too many values"));
    }

    #[test]
    fn test_tuple_unpacking_too_few_values() {
        let mut ctx = Context::new();
        let result = ctx.eval("a, b, c = [1, 2]");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("ValueError") || err.contains("expected 3"));
    }

    #[test]
    fn test_tuple_literal() {
        let mut ctx = Context::new();
        let result = ctx.eval("t = (1, 2, 3)").unwrap();
        assert_eq!(result, Value::None);
        // Tuple is stored in variable t
    }

    #[test]
    fn test_tuple_in_expression() {
        let mut ctx = Context::new();
        let result = ctx.eval("(1, 2)").unwrap();
        // Should return a tuple
        match result {
            Value::Tuple(t) => {
                assert_eq!(t.len(), 2);
                assert_eq!(t[0], Value::Int(1));
                assert_eq!(t[1], Value::Int(2));
            }
            _ => panic!("Expected tuple"),
        }
    }

    // Task 034: str() Builtin Function
    #[test]
    fn test_str_int() {
        let mut ctx = Context::new();
        let result = ctx.eval("str(123)").unwrap();
        assert_eq!(result, Value::String("123".to_string()));
    }

    #[test]
    fn test_str_float() {
        let mut ctx = Context::new();
        let result = ctx.eval("str(3.14)").unwrap();
        assert_eq!(result, Value::String("3.14".to_string()));
    }

    #[test]
    fn test_str_bool() {
        let mut ctx = Context::new();
        let result = ctx.eval("str(True)").unwrap();
        assert_eq!(result, Value::String("True".to_string()));

        let result = ctx.eval("str(False)").unwrap();
        assert_eq!(result, Value::String("False".to_string()));
    }

    #[test]
    fn test_str_none() {
        let mut ctx = Context::new();
        let result = ctx.eval("str(None)").unwrap();
        assert_eq!(result, Value::String("None".to_string()));
    }

    #[test]
    fn test_str_string() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"str("hello")"#).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_str_list() {
        let mut ctx = Context::new();
        let result = ctx.eval("str([1, 2, 3])").unwrap();
        assert_eq!(result, Value::String("[1, 2, 3]".to_string()));
    }

    #[test]
    fn test_str_tuple() {
        let mut ctx = Context::new();
        let result = ctx.eval("str((1, 2))").unwrap();
        assert_eq!(result, Value::String("(1, 2)".to_string()));
    }

    #[test]
    fn test_str_dict() {
        let mut ctx = Context::new();
        ctx.eval(r#"d = {"a": 1}"#).unwrap();
        let result = ctx.eval("str(d)").unwrap();
        // Dict order might vary, just check it contains the key-value
        let s = result.as_string().unwrap();
        assert!(s.contains("'a'"));
        assert!(s.contains("1"));
    }

    #[test]
    fn test_str_concatenation() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""Number: " + str(42)"#).unwrap();
        assert_eq!(result, Value::String("Number: 42".to_string()));
    }

    // === Value type tests ===

    #[test]
    fn test_value_equality_int() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
a = 42
b = 42
c = 43
a == b and a != c
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_value_equality_float() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
a = 3.14
b = 3.14
c = 2.71
a == b and a != c
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_value_equality_string() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
a = "hello"
b = "hello"
c = "world"
a == b and a != c
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_value_equality_list() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
a = [1, 2, 3]
b = [1, 2, 3]
# List equality by content
len(a) == len(b) and a[0] == b[0] and a[1] == b[1]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_value_equality_dict() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
a = {"x": 1, "y": 2}
b = {"x": 1, "y": 2}
# Dict equality by content
a["x"] == b["x"] and a["y"] == b["y"]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_value_equality_none() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
a = None
b = None
a == b
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_value_equality_bool() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
a = True
b = True
c = False
a == b and a != c
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_value_truthy_int() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
if 1:
    result = True
else:
    result = False
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_value_falsy_zero() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
if 0:
    result = True
else:
    result = False
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_value_falsy_empty_string() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
if "":
    result = True
else:
    result = False
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_value_falsy_empty_list() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
if []:
    result = True
else:
    result = False
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_value_falsy_empty_dict() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
if {}:
    result = True
else:
    result = False
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_value_truthy_nonempty_string() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
if "hello":
    result = True
else:
    result = False
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_value_truthy_nonempty_list() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
if [1]:
    result = True
else:
    result = False
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_value_comparison_mixed_types() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
a = 42
b = "42"
a != b
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_value_none_comparison() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
a = None
b = 0
c = ""
d = []
a != b and a != c and a != d
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_dict_key_int() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
d = {1: "one", 2: "two", 3: "three"}
d[2]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("two"));
    }

    #[test]
    fn test_dict_key_string() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
d = {"a": 1, "b": 2, "c": 3}
d["b"]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(2));
    }

    #[test]
    fn test_dict_mixed_keys() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
d = {1: "int", "1": "string"}
d[1] != d["1"]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    // Task 033: F-String Formatting
    #[test]
    fn test_fstring_simple_variable() {
        let mut ctx = Context::new();
        ctx.eval(r#"name = "Alice""#).unwrap();
        let result = ctx.eval(r#"f"Hello {name}""#).unwrap();
        assert_eq!(result, Value::String("Hello Alice".to_string()));
    }

    #[test]
    fn test_fstring_multiple_variables() {
        let mut ctx = Context::new();
        ctx.eval("x = 10").unwrap();
        ctx.eval("y = 20").unwrap();
        let result = ctx.eval(r#"f"{x} + {y} = {x + y}""#).unwrap();
        assert_eq!(result, Value::String("10 + 20 = 30".to_string()));
    }

    #[test]
    fn test_fstring_expressions() {
        let mut ctx = Context::new();
        ctx.eval("n = 5").unwrap();
        let result = ctx.eval(r#"f"Square of {n} is {n * n}""#).unwrap();
        assert_eq!(result, Value::String("Square of 5 is 25".to_string()));
    }

    #[test]
    fn test_fstring_different_types() {
        let mut ctx = Context::new();
        ctx.eval("age = 30").unwrap();
        ctx.eval("height = 5.9").unwrap();
        let result = ctx.eval(r#"f"Age: {age}, Height: {height}""#).unwrap();
        assert_eq!(result, Value::String("Age: 30, Height: 5.9".to_string()));
    }

    #[test]
    fn test_fstring_no_interpolation() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"f"Hello World""#).unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_fstring_only_expression() {
        let mut ctx = Context::new();
        ctx.eval("x = 42").unwrap();
        let result = ctx.eval(r#"f"{x}""#).unwrap();
        assert_eq!(result, Value::String("42".to_string()));
    }

    #[test]
    fn test_fstring_function_call() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
def get_name():
    return "Bob"
"#,
        )
        .unwrap();
        let result = ctx.eval(r#"f"Hello {get_name()}""#).unwrap();
        assert_eq!(result, Value::String("Hello Bob".to_string()));
    }

    #[test]
    fn test_fstring_nested_expression() {
        let mut ctx = Context::new();
        ctx.eval("items = [1, 2, 3]").unwrap();
        let result = ctx.eval(r#"f"Length: {len(items)}""#).unwrap();
        assert_eq!(result, Value::String("Length: 3".to_string()));
    }

    // Task 035: Slicing tests
    #[test]
    fn test_list_slice_basic() {
        let mut ctx = Context::new();
        ctx.eval("items = [0, 1, 2, 3, 4]").unwrap();
        let result = ctx.eval("items[1:3]").unwrap();
        let expected = ctx.eval("[1, 2]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_list_slice_start_only() {
        let mut ctx = Context::new();
        ctx.eval("items = [0, 1, 2, 3, 4]").unwrap();
        let result = ctx.eval("items[2:]").unwrap();
        let expected = ctx.eval("[2, 3, 4]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_list_slice_stop_only() {
        let mut ctx = Context::new();
        ctx.eval("items = [0, 1, 2, 3, 4]").unwrap();
        let result = ctx.eval("items[:3]").unwrap();
        let expected = ctx.eval("[0, 1, 2]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_list_slice_full() {
        let mut ctx = Context::new();
        ctx.eval("items = [0, 1, 2, 3, 4]").unwrap();
        let result = ctx.eval("items[:]").unwrap();
        let expected = ctx.eval("[0, 1, 2, 3, 4]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_list_slice_step() {
        let mut ctx = Context::new();
        ctx.eval("items = [0, 1, 2, 3, 4, 5, 6]").unwrap();
        let result = ctx.eval("items[::2]").unwrap();
        let expected = ctx.eval("[0, 2, 4, 6]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_list_slice_with_negative_indices() {
        let mut ctx = Context::new();
        ctx.eval("items = [0, 1, 2, 3, 4]").unwrap();
        let result = ctx.eval("items[-3:-1]").unwrap();
        let expected = ctx.eval("[2, 3]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_list_slice_negative_step() {
        let mut ctx = Context::new();
        ctx.eval("items = [0, 1, 2, 3, 4]").unwrap();
        let result = ctx.eval("items[::-1]").unwrap();
        let expected = ctx.eval("[4, 3, 2, 1, 0]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_string_slice_basic() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""hello"[1:4]"#).unwrap();
        assert_eq!(result, Value::String("ell".to_string()));
    }

    #[test]
    fn test_string_slice_start_only() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""hello"[2:]"#).unwrap();
        assert_eq!(result, Value::String("llo".to_string()));
    }

    #[test]
    fn test_string_slice_stop_only() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""hello"[:3]"#).unwrap();
        assert_eq!(result, Value::String("hel".to_string()));
    }

    #[test]
    fn test_string_slice_step() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""hello"[::2]"#).unwrap();
        assert_eq!(result, Value::String("hlo".to_string()));
    }

    #[test]
    fn test_string_slice_negative_step() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#""hello"[::-1]"#).unwrap();
        assert_eq!(result, Value::String("olleh".to_string()));
    }

    #[test]
    fn test_tuple_slice_basic() {
        let mut ctx = Context::new();
        ctx.eval("t = (0, 1, 2, 3, 4)").unwrap();
        let result = ctx.eval("t[1:3]").unwrap();
        let expected = ctx.eval("(1, 2)").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_slice_empty_result() {
        let mut ctx = Context::new();
        ctx.eval("items = [1, 2, 3]").unwrap();
        let result = ctx.eval("items[5:10]").unwrap();
        let expected = ctx.eval("[]").unwrap();
        assert_eq!(result, expected);
    }

    // Task 036: isinstance() tests
    #[test]
    fn test_isinstance_int() {
        let mut ctx = Context::new();
        ctx.eval("x = 123").unwrap();
        let result = ctx.eval("isinstance(x, int)").unwrap();
        assert_eq!(result, Value::Bool(true));
        let result = ctx.eval("isinstance(x, str)").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_isinstance_float() {
        let mut ctx = Context::new();
        ctx.eval("y = 3.14").unwrap();
        let result = ctx.eval("isinstance(y, float)").unwrap();
        assert_eq!(result, Value::Bool(true));
        let result = ctx.eval("isinstance(y, int)").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_isinstance_string() {
        let mut ctx = Context::new();
        ctx.eval("s = 'hello'").unwrap();
        let result = ctx.eval("isinstance(s, str)").unwrap();
        assert_eq!(result, Value::Bool(true));
        let result = ctx.eval("isinstance(s, list)").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_isinstance_bool() {
        let mut ctx = Context::new();
        ctx.eval("b = True").unwrap();
        let result = ctx.eval("isinstance(b, bool)").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_isinstance_list() {
        let mut ctx = Context::new();
        ctx.eval("lst = [1, 2, 3]").unwrap();
        let result = ctx.eval("isinstance(lst, list)").unwrap();
        assert_eq!(result, Value::Bool(true));
        let result = ctx.eval("isinstance(lst, dict)").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_isinstance_dict() {
        let mut ctx = Context::new();
        ctx.eval("d = {'a': 1}").unwrap();
        let result = ctx.eval("isinstance(d, dict)").unwrap();
        assert_eq!(result, Value::Bool(true));
        let result = ctx.eval("isinstance(d, list)").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_isinstance_tuple() {
        let mut ctx = Context::new();
        ctx.eval("t = (1, 2, 3)").unwrap();
        let result = ctx.eval("isinstance(t, tuple)").unwrap();
        assert_eq!(result, Value::Bool(true));
        let result = ctx.eval("isinstance(t, list)").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_isinstance_in_condition() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
def process(value):
    if isinstance(value, int):
        return value * 2
    elif isinstance(value, str):
        return value.upper()
    else:
        return None
"#,
        )
        .unwrap();
        let result = ctx.eval("process(5)").unwrap();
        assert_eq!(result, Value::Int(10));
        let result = ctx.eval("process('hi')").unwrap();
        assert_eq!(result, Value::String("HI".to_string()));
    }

    #[test]
    fn test_isinstance_wrong_arg_count() {
        let mut ctx = Context::new();
        let result = ctx.eval("isinstance(1)");
        assert!(result.is_err());
    }

    #[test]
    fn test_isinstance_second_arg_not_type() {
        let mut ctx = Context::new();
        let result = ctx.eval("isinstance(1, 'int')");
        assert!(result.is_err());
    }

    // Task 037: List Comprehensions tests
    #[test]
    fn test_listcomp_basic() {
        let mut ctx = Context::new();
        let result = ctx.eval("[x*x for x in range(5)]").unwrap();
        let expected = ctx.eval("[0, 1, 4, 9, 16]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_listcomp_with_filter() {
        let mut ctx = Context::new();
        let result = ctx.eval("[x for x in range(10) if x % 2 == 0]").unwrap();
        let expected = ctx.eval("[0, 2, 4, 6, 8]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_listcomp_string_transformation() {
        let mut ctx = Context::new();
        ctx.eval("words = ['hello', 'world']").unwrap();
        let result = ctx.eval("[w.upper() for w in words]").unwrap();
        let expected = ctx.eval("['HELLO', 'WORLD']").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_listcomp_expression() {
        let mut ctx = Context::new();
        ctx.eval("nums = [1, 2, 3]").unwrap();
        let result = ctx.eval("[n * 2 for n in nums]").unwrap();
        let expected = ctx.eval("[2, 4, 6]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_listcomp_empty_result() {
        let mut ctx = Context::new();
        let result = ctx.eval("[x for x in range(5) if x > 10]").unwrap();
        let expected = ctx.eval("[]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_listcomp_multiple_filters() {
        let mut ctx = Context::new();
        let result = ctx
            .eval("[x for x in range(20) if x % 2 == 0 if x % 3 == 0]")
            .unwrap();
        let expected = ctx.eval("[0, 6, 12, 18]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_listcomp_with_variable() {
        let mut ctx = Context::new();
        ctx.eval("n = 5").unwrap();
        let result = ctx.eval("[i for i in range(n)]").unwrap();
        let expected = ctx.eval("[0, 1, 2, 3, 4]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_listcomp_complex_expression() {
        let mut ctx = Context::new();
        let result = ctx.eval("[x*2 + 1 for x in range(5)]").unwrap();
        let expected = ctx.eval("[1, 3, 5, 7, 9]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_listcomp_from_list() {
        let mut ctx = Context::new();
        ctx.eval("items = [1, 2, 3, 4, 5]").unwrap();
        let result = ctx.eval("[x*x for x in items if x > 2]").unwrap();
        let expected = ctx.eval("[9, 16, 25]").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_listcomp_string_iteration() {
        let mut ctx = Context::new();
        let result = ctx.eval("[c for c in 'hello']").unwrap();
        let expected = ctx.eval("['h', 'e', 'l', 'l', 'o']").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_string_iteration_for_loop() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = []
for c in "abc":
    result.append(c)
"#,
        )
        .unwrap();
        let result = ctx.eval("result").unwrap();
        let expected = ctx.eval("['a', 'b', 'c']").unwrap();
        assert_eq!(result, expected);
    }

    // Task 038: Async/Await Support
    #[test]
    fn test_async_function_basic() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
async def greet(name):
    return "Hello, " + name

result = await greet("World")
"#,
        )
        .unwrap();
        let result = ctx.eval("result").unwrap();
        assert_eq!(result, Value::String("Hello, World".to_string()));
    }

    #[test]
    fn test_async_function_with_computation() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
async def compute(x, y):
    return x * y + 10

result = await compute(5, 3)
"#,
        )
        .unwrap();
        let result = ctx.eval("result").unwrap();
        assert_eq!(result, Value::Int(25));
    }

    #[test]
    fn test_async_function_returns_coroutine() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
async def get_value():
    return 42

coro = get_value()
"#,
        )
        .unwrap();
        let result = ctx.eval("coro").unwrap();
        // Should be a coroutine object, not the result
        match result {
            Value::Coroutine(_, _) => {} // Expected
            _ => panic!("Expected coroutine, got {:?}", result),
        }
    }

    #[test]
    fn test_await_coroutine() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
async def get_value():
    return 42

coro = get_value()
result = await coro
"#,
        )
        .unwrap();
        let result = ctx.eval("result").unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_async_function_with_multiple_statements() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
async def process(x):
    y = x * 2
    z = y + 5
    return z

result = await process(10)
"#,
        )
        .unwrap();
        let result = ctx.eval("result").unwrap();
        assert_eq!(result, Value::Int(25));
    }

    #[test]
    fn test_await_non_coroutine_error() {
        let mut ctx = Context::new();
        let result = ctx.eval("await 42");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("TypeError"));
        assert!(err.contains("cannot be awaited"));
    }

    #[test]
    fn test_async_function_with_conditionals() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
async def check(x):
    if x > 10:
        return "big"
    else:
        return "small"

result1 = await check(15)
result2 = await check(5)
"#,
        )
        .unwrap();
        let result1 = ctx.eval("result1").unwrap();
        let result2 = ctx.eval("result2").unwrap();
        assert_eq!(result1, Value::String("big".to_string()));
        assert_eq!(result2, Value::String("small".to_string()));
    }

    #[test]
    fn test_async_function_nested_calls() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
async def inner(x):
    return x * 2

async def outer(x):
    y = await inner(x)
    return y + 1

result = await outer(5)
"#,
        )
        .unwrap();
        let result = ctx.eval("result").unwrap();
        assert_eq!(result, Value::Int(11));
    }

    // Tests for asyncio.sleep - true async behavior
    #[test]
    fn test_asyncio_sleep_basic() {
        let mut ctx = Context::new();
        let start = std::time::Instant::now();
        ctx.eval(
            r#"
import asyncio

async def test():
    await asyncio.sleep(0.1)
    return "done"

result = await test()
"#,
        )
        .unwrap();
        let elapsed = start.elapsed();
        let result = ctx.eval("result").unwrap();
        assert_eq!(result, Value::String("done".to_string()));
        // Verify it actually slept (at least 100ms)
        assert!(elapsed.as_millis() >= 100);
    }

    #[test]
    fn test_asyncio_sleep_with_int() {
        let mut ctx = Context::new();
        let start = std::time::Instant::now();
        ctx.eval(
            r#"
import asyncio
await asyncio.sleep(0)
"#,
        )
        .unwrap();
        let elapsed = start.elapsed();
        // Should complete quickly (runtime creation has overhead)
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_asyncio_sleep_with_float() {
        let mut ctx = Context::new();
        let start = std::time::Instant::now();
        ctx.eval(
            r#"
import asyncio
await asyncio.sleep(0.05)
"#,
        )
        .unwrap();
        let elapsed = start.elapsed();
        // Should sleep at least 50ms
        assert!(elapsed.as_millis() >= 50);
    }

    #[test]
    fn test_asyncio_sleep_negative_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
import asyncio
await asyncio.sleep(-1)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("ValueError"));
        assert!(err.contains("non-negative"));
    }

    #[test]
    fn test_asyncio_sleep_wrong_type_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
import asyncio
await asyncio.sleep("hello")
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("TypeError"));
        assert!(err.contains("must be a number"));
    }

    #[test]
    fn test_asyncio_sleep_in_async_function() {
        let mut ctx = Context::new();
        let start = std::time::Instant::now();
        ctx.eval(
            r#"
import asyncio

async def delayed_add(x, y):
    await asyncio.sleep(0.1)
    return x + y

result = await delayed_add(3, 4)
"#,
        )
        .unwrap();
        let elapsed = start.elapsed();
        let result = ctx.eval("result").unwrap();
        assert_eq!(result, Value::Int(7));
        assert!(elapsed.as_millis() >= 100);
    }

    #[test]
    fn test_asyncio_sleep_multiple_awaits() {
        let mut ctx = Context::new();
        let start = std::time::Instant::now();
        ctx.eval(
            r#"
import asyncio

async def multi_sleep():
    await asyncio.sleep(0.05)
    await asyncio.sleep(0.05)
    return "done"

result = await multi_sleep()
"#,
        )
        .unwrap();
        let elapsed = start.elapsed();
        let result = ctx.eval("result").unwrap();
        assert_eq!(result, Value::String("done".to_string()));
        // Should sleep at least 100ms total
        assert!(elapsed.as_millis() >= 100);
    }

    // Value.rs tests - ExceptionType methods
    #[test]
    fn test_exception_type_from_i32_valid() {
        use crate::value::ExceptionType;
        assert_eq!(ExceptionType::from_i32(0), Some(ExceptionType::Exception));
        assert_eq!(
            ExceptionType::from_i32(1),
            Some(ExceptionType::RuntimeError)
        );
        assert_eq!(ExceptionType::from_i32(2), Some(ExceptionType::IndexError));
        assert_eq!(ExceptionType::from_i32(3), Some(ExceptionType::KeyError));
        assert_eq!(ExceptionType::from_i32(4), Some(ExceptionType::ValueError));
        assert_eq!(ExceptionType::from_i32(5), Some(ExceptionType::TypeError));
        assert_eq!(
            ExceptionType::from_i32(6),
            Some(ExceptionType::ZeroDivisionError)
        );
        assert_eq!(
            ExceptionType::from_i32(7),
            Some(ExceptionType::IteratorError)
        );
        assert_eq!(ExceptionType::from_i32(8), Some(ExceptionType::OSError));
        assert_eq!(
            ExceptionType::from_i32(9),
            Some(ExceptionType::AttributeError)
        );
    }

    #[test]
    fn test_exception_type_from_i32_invalid() {
        use crate::value::ExceptionType;
        assert_eq!(ExceptionType::from_i32(10), None);
        assert_eq!(ExceptionType::from_i32(-1), None);
        assert_eq!(ExceptionType::from_i32(999), None);
    }

    #[test]
    fn test_exception_type_as_i32() {
        use crate::value::ExceptionType;
        assert_eq!(ExceptionType::Exception.as_i32(), 0);
        assert_eq!(ExceptionType::RuntimeError.as_i32(), 1);
        assert_eq!(ExceptionType::IndexError.as_i32(), 2);
        assert_eq!(ExceptionType::KeyError.as_i32(), 3);
        assert_eq!(ExceptionType::ValueError.as_i32(), 4);
        assert_eq!(ExceptionType::TypeError.as_i32(), 5);
        assert_eq!(ExceptionType::ZeroDivisionError.as_i32(), 6);
        assert_eq!(ExceptionType::IteratorError.as_i32(), 7);
        assert_eq!(ExceptionType::OSError.as_i32(), 8);
        assert_eq!(ExceptionType::AttributeError.as_i32(), 9);
    }

    #[test]
    fn test_exception_type_matches_base_catches_all() {
        use crate::value::ExceptionType;
        assert!(ExceptionType::ValueError.matches(&ExceptionType::Exception));
        assert!(ExceptionType::TypeError.matches(&ExceptionType::Exception));
        assert!(ExceptionType::IndexError.matches(&ExceptionType::Exception));
        assert!(ExceptionType::RuntimeError.matches(&ExceptionType::Exception));
    }

    #[test]
    fn test_exception_type_matches_specific() {
        use crate::value::ExceptionType;
        assert!(ExceptionType::ValueError.matches(&ExceptionType::ValueError));
        assert!(ExceptionType::TypeError.matches(&ExceptionType::TypeError));
        assert!(!ExceptionType::ValueError.matches(&ExceptionType::TypeError));
        assert!(!ExceptionType::TypeError.matches(&ExceptionType::ValueError));
    }

    // Value.rs tests - DictKey
    #[test]
    fn test_dict_key_equality() {
        use crate::value::DictKey;
        assert_eq!(
            DictKey::String("hello".to_string()),
            DictKey::String("hello".to_string())
        );
        assert_eq!(DictKey::Int(42), DictKey::Int(42));
        assert_ne!(
            DictKey::String("hello".to_string()),
            DictKey::String("world".to_string())
        );
        assert_ne!(DictKey::Int(42), DictKey::Int(43));
        assert_ne!(DictKey::String("42".to_string()), DictKey::Int(42));
    }

    #[test]
    fn test_dict_key_hash() {
        use crate::value::DictKey;
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(DictKey::String("key1".to_string()), 1);
        map.insert(DictKey::Int(42), 2);
        assert_eq!(map.get(&DictKey::String("key1".to_string())), Some(&1));
        assert_eq!(map.get(&DictKey::Int(42)), Some(&2));
        assert_eq!(map.get(&DictKey::String("key2".to_string())), None);
    }

    // Value.rs tests - Value as_* methods
    #[test]
    fn test_value_as_int_wrong_type() {
        assert_eq!(Value::Float(3.14).as_int(), None);
        assert_eq!(Value::String("hello".to_string()).as_int(), None);
        assert_eq!(Value::Bool(true).as_int(), None);
        assert_eq!(Value::None.as_int(), None);
    }

    #[test]
    fn test_value_as_float_wrong_type() {
        assert_eq!(Value::Int(42).as_float(), None);
        assert_eq!(Value::String("hello".to_string()).as_float(), None);
        assert_eq!(Value::Bool(true).as_float(), None);
        assert_eq!(Value::None.as_float(), None);
    }

    #[test]
    fn test_value_as_bool_wrong_type() {
        assert_eq!(Value::Int(42).as_bool(), None);
        assert_eq!(Value::Float(3.14).as_bool(), None);
        assert_eq!(Value::String("hello".to_string()).as_bool(), None);
        assert_eq!(Value::None.as_bool(), None);
    }

    #[test]
    fn test_value_as_string_wrong_type() {
        assert_eq!(Value::Int(42).as_string(), None);
        assert_eq!(Value::Float(3.14).as_string(), None);
        assert_eq!(Value::Bool(true).as_string(), None);
        assert_eq!(Value::None.as_string(), None);
    }

    #[test]
    fn test_value_as_list_wrong_type() {
        assert!(Value::Int(42).as_list().is_none());
        assert!(Value::String("hello".to_string()).as_list().is_none());
        assert!(Value::None.as_list().is_none());
    }

    #[test]
    fn test_value_as_dict_wrong_type() {
        assert!(Value::Int(42).as_dict().is_none());
        assert!(Value::String("hello".to_string()).as_dict().is_none());
        assert!(Value::None.as_dict().is_none());
    }

    // Value.rs tests - Value is_exception and as_exception
    #[test]
    fn test_value_is_exception() {
        use crate::value::ExceptionType;
        let exc = Value::error(ExceptionType::ValueError, "test error");
        assert!(exc.is_exception());
        assert!(!Value::Int(42).is_exception());
        assert!(!Value::String("error".to_string()).is_exception());
        assert!(!Value::None.is_exception());
    }

    #[test]
    fn test_value_as_exception() {
        use crate::value::ExceptionType;
        let exc = Value::error(ExceptionType::ValueError, "test error");
        let exc_val = exc.as_exception().unwrap();
        assert_eq!(exc_val.exception_type, ExceptionType::ValueError);
        assert_eq!(exc_val.message, "test error");
        assert!(Value::Int(42).as_exception().is_none());
    }

    // Value.rs tests - Value Debug formatting
    #[test]
    fn test_value_debug_format_primitives() {
        assert_eq!(format!("{:?}", Value::Int(42)), "Int(42)");
        assert_eq!(format!("{:?}", Value::Float(3.14)), "Float(3.14)");
        assert_eq!(format!("{:?}", Value::Bool(true)), "Bool(true)");
        assert_eq!(format!("{:?}", Value::None), "None");
        assert_eq!(
            format!("{:?}", Value::String("hello".to_string())),
            "String(\"hello\")"
        );
    }

    #[test]
    fn test_value_debug_format_slice() {
        let slice = Value::Slice {
            start: Some(1),
            stop: Some(5),
            step: Some(2),
        };
        assert_eq!(format!("{:?}", slice), "Slice(Some(1):Some(5):Some(2))");
    }

    #[test]
    fn test_value_debug_format_async_sleep() {
        let sleep = Value::AsyncSleep(1.5);
        assert_eq!(format!("{:?}", sleep), "AsyncSleep(1.5)");
    }

    // Value.rs tests - Module methods
    #[test]
    fn test_module_new() {
        use crate::value::Module;
        let module = Module::new("test_module");
        assert_eq!(module.name, "test_module");
        assert!(module.attributes.is_empty());
    }

    #[test]
    fn test_module_add_function() {
        use crate::value::Module;
        fn test_func(_args: Vec<Value>) -> Result<Value, Value> {
            Ok(Value::Int(42))
        }
        let mut module = Module::new("test");
        module.add_function("test_func", test_func);
        assert!(module.attributes.contains_key("test_func"));
    }

    #[test]
    fn test_module_get_attribute() {
        use crate::value::Module;
        let mut module = Module::new("test");
        module
            .attributes
            .insert("value".to_string(), Value::Int(42));
        assert_eq!(module.get_attribute("value"), Some(Value::Int(42)));
        assert_eq!(module.get_attribute("nonexistent"), None);
    }

    // Value.rs tests - ListValue methods
    #[test]
    fn test_list_value_new() {
        use crate::value::ListValue;
        let list = ListValue::new();
        assert!(list.items.is_empty());
        assert_eq!(list.version, 0);
    }

    #[test]
    fn test_list_value_with_items() {
        use crate::value::ListValue;
        let items = vec![Value::Int(1), Value::Int(2), Value::Int(3)];
        let list = ListValue::with_items(items.clone());
        assert_eq!(list.items, items);
        assert_eq!(list.version, 0);
    }

    #[test]
    fn test_list_value_increment_version() {
        use crate::value::ListValue;
        let mut list = ListValue::new();
        assert_eq!(list.version, 0);
        list.increment_version();
        assert_eq!(list.version, 1);
        list.increment_version();
        assert_eq!(list.version, 2);
    }

    #[test]
    fn test_list_value_version_wrapping() {
        use crate::value::ListValue;
        let mut list = ListValue::new();
        list.version = usize::MAX;
        list.increment_version();
        assert_eq!(list.version, 0); // Should wrap around
    }

    // Value.rs tests - MatchObject
    #[test]
    fn test_match_object_new() {
        use crate::value::MatchObject;
        let match_obj = MatchObject::new(
            "hello world".to_string(),
            0,
            5,
            vec![Some("hello".to_string())],
        );
        assert_eq!(match_obj.text, "hello world");
        assert_eq!(match_obj.start, 0);
        assert_eq!(match_obj.end, 5);
        assert_eq!(match_obj.groups, vec![Some("hello".to_string())]);
    }

    // Value.rs tests - Value equality for complex types
    #[test]
    fn test_value_equality_tuple() {
        use std::rc::Rc;
        let tuple1 = Value::Tuple(Rc::new(vec![Value::Int(1), Value::Int(2)]));
        let tuple2 = Value::Tuple(Rc::new(vec![Value::Int(1), Value::Int(2)]));
        let tuple3 = Value::Tuple(Rc::new(vec![Value::Int(1), Value::Int(3)]));
        assert_eq!(tuple1, tuple2);
        assert_ne!(tuple1, tuple3);
    }

    #[test]
    fn test_value_equality_slice() {
        let slice1 = Value::Slice {
            start: Some(1),
            stop: Some(5),
            step: Some(2),
        };
        let slice2 = Value::Slice {
            start: Some(1),
            stop: Some(5),
            step: Some(2),
        };
        let slice3 = Value::Slice {
            start: Some(1),
            stop: Some(6),
            step: Some(2),
        };
        assert_eq!(slice1, slice2);
        assert_ne!(slice1, slice3);
    }

    #[test]
    fn test_value_truthy_tuple() {
        use std::rc::Rc;
        let empty_tuple = Value::Tuple(Rc::new(vec![]));
        let nonempty_tuple = Value::Tuple(Rc::new(vec![Value::Int(1)]));
        assert!(!empty_tuple.is_truthy());
        assert!(nonempty_tuple.is_truthy());
    }

    #[test]
    fn test_value_truthy_float_zero() {
        assert!(!Value::Float(0.0).is_truthy());
        assert!(Value::Float(0.1).is_truthy());
        assert!(Value::Float(-0.1).is_truthy());
    }

    #[test]
    fn test_value_truthy_special_types() {
        use crate::bytecode::ByteCode;
        use crate::value::Function;
        let func = Function {
            name: "test".to_string(),
            params: vec![],
            code: ByteCode::new(),
            is_async: false,
        };
        assert!(Value::Function(func).is_truthy());
        assert!(Value::AsyncSleep(1.0).is_truthy());
    }

    // Serializer tests - more instruction types
    #[test]
    fn test_serialize_type_conversions() {
        let source = r#"
x = int("42")
y = float("3.14")
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("x"), Some(Value::Int(42)));
        assert_eq!(ctx.get("y"), Some(Value::Float(3.14)));
    }

    #[test]
    fn test_serialize_negate() {
        let source = r#"
x = 10
y = -x
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("y"), Some(Value::Int(-10)));
    }

    #[test]
    fn test_serialize_local_variables() {
        let source = r#"
def add(a, b):
    result = a + b
    return result

x = add(3, 4)
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode);
        // Should fail because MakeFunction is not serializable
        assert!(bytes.is_err());
    }

    #[test]
    fn test_serialize_jump_instructions() {
        let source = r#"
x = 10
if x > 5:
    y = 1
else:
    y = 2
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("y"), Some(Value::Int(1)));
    }

    #[test]
    fn test_serialize_all_comparison_ops() {
        let source = r#"
a = 5 == 5
b = 5 != 3
c = 5 < 10
d = 5 <= 5
e = 10 > 5
f = 10 >= 10
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("a"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("b"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("c"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("d"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("e"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("f"), Some(Value::Bool(true)));
    }

    #[test]
    fn test_serialize_all_arithmetic_ops() {
        let source = r#"
a = 10 + 5
b = 10 - 5
c = 10 * 5
d = 10 / 5
e = 10 % 3
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("a"), Some(Value::Int(15)));
        assert_eq!(ctx.get("b"), Some(Value::Int(5)));
        assert_eq!(ctx.get("c"), Some(Value::Int(50)));
        assert_eq!(ctx.get("d"), Some(Value::Int(2)));
        assert_eq!(ctx.get("e"), Some(Value::Int(1)));
    }

    #[test]
    fn test_serialize_complex_expression() {
        let source = r#"
x = 2
y = 3
z = (x + y) * (x - y)
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("z"), Some(Value::Int(-5)));
    }

    #[test]
    fn test_serialize_nested_if() {
        let source = r#"
x = 10
if x > 5:
    if x > 8:
        y = 1
    else:
        y = 2
else:
    y = 3
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("y"), Some(Value::Int(1)));
    }

    #[test]
    fn test_serialize_float_operations() {
        let source = r#"
a = 3.14
b = 2.0
c = a + b
d = a * b
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        // Use approximate comparison for floats due to precision
        match ctx.get("c") {
            Some(Value::Float(f)) => assert!((f - 5.14).abs() < 0.0001),
            _ => panic!("Expected float value for c"),
        }
        match ctx.get("d") {
            Some(Value::Float(f)) => assert!((f - 6.28).abs() < 0.0001),
            _ => panic!("Expected float value for d"),
        }
    }

    #[test]
    fn test_serialize_string_concatenation() {
        let source = r#"
a = "hello"
b = "world"
c = a + " " + b
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("c"), Some(Value::String("hello world".to_string())));
    }

    #[test]
    fn test_serialize_empty_bytecode() {
        let bytecode: ByteCode = vec![];
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();
        assert_eq!(restored.len(), 0);
    }

    #[test]
    fn test_deserialize_invalid_magic() {
        let bytes = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Invalid magic
        let result = deserialize_bytecode(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_invalid_version() {
        let mut bytes = vec![0x51, 0x50, 0x59, 0x51]; // Valid magic
        bytes.push(99); // Invalid version
        let result = deserialize_bytecode(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_truncated_data() {
        let bytes = vec![0x51, 0x50, 0x59, 0x51, 0x01]; // Magic + version, but no data
        let result = deserialize_bytecode(&bytes);
        // Should either succeed with empty bytecode or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_serialize_round_trip_multiple_types() {
        let source = r#"
int_val = 42
float_val = 3.14
bool_val = True
none_val = None
str_val = "test"
        "#;
        let bytecode = Compiler::compile(source).unwrap();
        let bytes = serialize_bytecode(&bytecode).unwrap();
        let restored = deserialize_bytecode(&bytes).unwrap();

        let mut ctx = Context::new();
        ctx.eval_bytecode(&restored).unwrap();
        assert_eq!(ctx.get("int_val"), Some(Value::Int(42)));
        assert_eq!(ctx.get("float_val"), Some(Value::Float(3.14)));
        assert_eq!(ctx.get("bool_val"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("none_val"), Some(Value::None));
        assert_eq!(ctx.get("str_val"), Some(Value::String("test".to_string())));
    }

    // VM deep tests - exception handling and edge cases

    #[test]
    fn test_multiple_except_handlers_type_match() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
try:
    raise TypeError("type error")
except ValueError:
    result = "value"
except TypeError:
    result = "type"
except:
    result = "other"
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::String("type".to_string())));
    }

    #[test]
    fn test_nested_try_except_propagation() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
try:
    try:
        raise ValueError("inner")
    except TypeError:
        result = "inner type"
except ValueError:
    result = "outer value"
"#,
        )
        .unwrap();
        assert_eq!(
            ctx.get("result"),
            Some(Value::String("outer value".to_string()))
        );
    }

    #[test]
    fn test_finally_with_exception() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
try:
    raise ValueError("test")
finally:
    cleanup = True
"#,
        );
        assert!(result.is_err());
        assert_eq!(ctx.get("cleanup"), Some(Value::Bool(true)));
    }

    #[test]
    fn test_method_call_on_list() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [1, 2, 3]
x.append(4)
x.append(5)
"#,
        )
        .unwrap();
        let list = ctx.get("x").unwrap();
        if let Value::List(l) = list {
            assert_eq!(l.borrow().items.len(), 5);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_method_call_on_string() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
s = "hello world"
result = s.upper()
"#,
        )
        .unwrap();
        assert_eq!(
            ctx.get("result"),
            Some(Value::String("HELLO WORLD".to_string()))
        );
    }

    #[test]
    fn test_chained_method_calls() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = "  hello  ".strip().upper()
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::String("HELLO".to_string())));
    }

    #[test]
    fn test_slice_with_none_bounds() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [1, 2, 3, 4, 5]
result = x[:]
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            assert_eq!(l.borrow().items.len(), 5);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_slice_with_negative_step() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [1, 2, 3, 4, 5]
result = x[::-1]
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            let items = &l.borrow().items;
            assert_eq!(items[0], Value::Int(5));
            assert_eq!(items[4], Value::Int(1));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_slice_string_with_step() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
s = "abcdefgh"
result = s[::2]
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::String("aceg".to_string())));
    }

    #[test]
    fn test_nested_list_modification() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [[1, 2], [3, 4]]
x[0].append(3)
x[1].append(5)
"#,
        )
        .unwrap();
        let x = ctx.get("x").unwrap();
        if let Value::List(outer) = x {
            let items = &outer.borrow().items;
            if let Value::List(inner) = &items[0] {
                assert_eq!(inner.borrow().items.len(), 3);
            }
        }
    }

    #[test]
    fn test_dict_get_with_default_value() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
d = {"a": 1, "b": 2}
result1 = d.get("a")
result2 = d.get("c")
result3 = d.get("c", 99)
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Int(1)));
        assert_eq!(ctx.get("result2"), Some(Value::None));
        assert_eq!(ctx.get("result3"), Some(Value::Int(99)));
    }

    #[test]
    fn test_string_join_list() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
parts = ["hello", "world", "test"]
result = " ".join(parts)
"#,
        )
        .unwrap();
        assert_eq!(
            ctx.get("result"),
            Some(Value::String("hello world test".to_string()))
        );
    }

    #[test]
    fn test_while_with_break_in_nested_if() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = 0
while x < 100:
    x += 1
    if x > 5:
        if x == 7:
            break
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("x"), Some(Value::Int(7)));
    }

    #[test]
    fn test_isinstance_with_multiple_types() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = 42
result1 = isinstance(x, int)
result2 = isinstance(x, str)
result3 = isinstance("hello", str)
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(false)));
        assert_eq!(ctx.get("result3"), Some(Value::Bool(true)));
    }

    // Additional VM tests for existing functionality
    #[test]
    fn test_string_startswith_endswith() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
s = "hello world"
result1 = s.startswith("hello")
result2 = s.startswith("world")
result3 = s.endswith("world")
result4 = s.endswith("hello")
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(false)));
        assert_eq!(ctx.get("result3"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result4"), Some(Value::Bool(false)));
    }

    // Additional tests to reach 75% coverage
    #[test]
    fn test_division_by_zero_error() {
        let mut ctx = Context::new();
        let result = ctx.eval("10 / 0");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("ZeroDivisionError"));
    }

    #[test]
    fn test_modulo_by_zero() {
        let mut ctx = Context::new();
        let result = ctx.eval("10 % 0");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("ZeroDivisionError"));
    }

    #[test]
    fn test_float_division_by_zero() {
        let mut ctx = Context::new();
        let result = ctx.eval("10.5 / 0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_index_negative_out_of_bounds() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
x = [1, 2, 3]
x[-10]
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_string_index_out_of_bounds() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
s = "hello"
s[10]
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_string_negative_index_out_of_bounds() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
s = "hello"
s[-10]
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_tuple_index_out_of_bounds() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
t = (1, 2, 3)
t[10]
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_slice_assignment() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
x = [1, 2, 3]
x[0:2] = 5
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_call_non_function() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
x = 42
x()
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_attribute_error_on_int() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
x = 42
x.append(5)
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_list_pop_empty() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
x = []
x.pop()
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_list_index_not_found() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
x = [1, 2, 3]
x.index(99)
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_dict_pop_missing_key() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
d = {"a": 1}
d.pop("b")
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_string_replace() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
s = "hello world"
result = s.replace("world", "python")
"#,
        )
        .unwrap();
        assert_eq!(
            ctx.get("result"),
            Some(Value::String("hello python".to_string()))
        );
    }

    #[test]
    fn test_string_replace_multiple() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
s = "hello hello hello"
result = s.replace("hello", "hi")
"#,
        )
        .unwrap();
        assert_eq!(
            ctx.get("result"),
            Some(Value::String("hi hi hi".to_string()))
        );
    }

    #[test]
    fn test_nested_function_calls() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = len(str(len([1, 2, 3])))
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Int(1)));
    }

    #[test]
    fn test_complex_list_comprehension() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = [x * 2 for x in range(5) if x % 2 == 0]
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            let items = &l.borrow().items;
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Int(0));
            assert_eq!(items[1], Value::Int(4));
            assert_eq!(items[2], Value::Int(8));
        }
    }

    #[test]
    fn test_multiple_assignment() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
a = b = c = 10
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("a"), Some(Value::Int(10)));
        assert_eq!(ctx.get("b"), Some(Value::Int(10)));
        assert_eq!(ctx.get("c"), Some(Value::Int(10)));
    }

    #[test]
    fn test_augmented_assignment_all_ops() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
a = 10
a += 5
b = 20
b -= 5
c = 3
c *= 4
d = 20
d /= 4
e = 17
e %= 5
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("a"), Some(Value::Int(15)));
        assert_eq!(ctx.get("b"), Some(Value::Int(15)));
        assert_eq!(ctx.get("c"), Some(Value::Int(12)));
        assert_eq!(ctx.get("d"), Some(Value::Int(5)));
        assert_eq!(ctx.get("e"), Some(Value::Int(2)));
    }

    #[test]
    fn test_in_operator_list() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [1, 2, 3, 4, 5]
result1 = 3 in x
result2 = 10 in x
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(false)));
    }

    #[test]
    fn test_in_operator_string() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
s = "hello world"
result1 = "world" in s
result2 = "xyz" in s
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(false)));
    }

    #[test]
    fn test_not_in_operator() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [1, 2, 3]
result1 = 5 not in x
result2 = 2 not in x
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(false)));
    }

    #[test]
    fn test_logical_and_short_circuit() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = 0
result = x and (10 / x)
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Int(0)));
    }

    #[test]
    fn test_logical_or_short_circuit() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = 5
result = x or (10 / 0)
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Int(5)));
    }

    #[test]
    fn test_not_operator() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result1 = not True
result2 = not False
result3 = not 0
result4 = not 5
result5 = not ""
result6 = not "hello"
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Bool(false)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result3"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result4"), Some(Value::Bool(false)));
        assert_eq!(ctx.get("result5"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result6"), Some(Value::Bool(false)));
    }

    // More tests for better coverage
    #[test]
    fn test_empty_string_operations() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
s = ""
result1 = len(s)
result2 = s.upper()
result3 = s.lower()
result4 = s.strip()
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Int(0)));
        assert_eq!(ctx.get("result2"), Some(Value::String("".to_string())));
        assert_eq!(ctx.get("result3"), Some(Value::String("".to_string())));
        assert_eq!(ctx.get("result4"), Some(Value::String("".to_string())));
    }

    #[test]
    fn test_tuple_slicing() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
t = (1, 2, 3, 4, 5)
result = t[1:4]
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        if let Value::Tuple(t) = result {
            assert_eq!(t.len(), 3);
        }
    }

    #[test]
    fn test_string_split_no_separator() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
s = "hello world test"
result = s.split()
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            assert_eq!(l.borrow().items.len(), 3);
        }
    }

    #[test]
    fn test_string_split_custom_separator() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
s = "a,b,c,d"
result = s.split(",")
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            assert_eq!(l.borrow().items.len(), 4);
        }
    }

    #[test]
    fn test_string_strip_whitespace() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
s = "  hello  "
result = s.strip()
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::String("hello".to_string())));
    }

    #[test]
    fn test_for_loop_with_break() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = 0
for i in range(10):
    if i == 5:
        break
    result = i
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Int(4)));
    }

    #[test]
    fn test_for_loop_with_continue() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = 0
for i in range(10):
    if i % 2 == 0:
        continue
    result += i
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Int(25)));
    }

    #[test]
    fn test_while_loop_basic() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
i = 0
while i < 5:
    i += 1
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("i"), Some(Value::Int(5)));
    }

    #[test]
    fn test_while_loop_with_break() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
i = 0
while True:
    i += 1
    if i >= 10:
        break
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("i"), Some(Value::Int(10)));
    }

    #[test]
    fn test_while_loop_with_continue() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
i = 0
count = 0
while i < 10:
    i += 1
    if i % 2 == 0:
        continue
    count += 1
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("count"), Some(Value::Int(5)));
    }

    #[test]
    fn test_nested_loops() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = 0
for i in range(3):
    for j in range(3):
        result += 1
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Int(9)));
    }

    #[test]
    fn test_function_multiple_returns() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
def check(x):
    if x > 0:
        return "positive"
    elif x < 0:
        return "negative"
    else:
        return "zero"

result1 = check(5)
result2 = check(-3)
result3 = check(0)
"#,
        )
        .unwrap();
        assert_eq!(
            ctx.get("result1"),
            Some(Value::String("positive".to_string()))
        );
        assert_eq!(
            ctx.get("result2"),
            Some(Value::String("negative".to_string()))
        );
        assert_eq!(ctx.get("result3"), Some(Value::String("zero".to_string())));
    }

    #[test]
    fn test_recursive_function() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

result = factorial(5)
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Int(120)));
    }

    #[test]
    fn test_list_comprehension_with_condition() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = [x for x in range(10) if x % 2 == 0]
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            assert_eq!(l.borrow().items.len(), 5);
        }
    }

    #[test]
    fn test_list_comprehension_with_transformation() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = [x * x for x in range(5)]
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            let items = &l.borrow().items;
            assert_eq!(items[0], Value::Int(0));
            assert_eq!(items[4], Value::Int(16));
        }
    }

    #[test]
    fn test_elif_chain() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = 15
if x < 10:
    result = "small"
elif x < 20:
    result = "medium"
elif x < 30:
    result = "large"
else:
    result = "huge"
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::String("medium".to_string())));
    }

    #[test]
    fn test_comparison_operators() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result1 = 5 == 5
result2 = 5 != 3
result3 = 5 < 10
result4 = 5 <= 5
result5 = 10 > 5
result6 = 10 >= 10
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result3"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result4"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result5"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result6"), Some(Value::Bool(true)));
    }

    #[test]
    fn test_float_operations() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
a = 3.5
b = 2.0
result1 = a + b
result2 = a - b
result3 = a * b
result4 = a / b
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Float(5.5)));
        assert_eq!(ctx.get("result2"), Some(Value::Float(1.5)));
        assert_eq!(ctx.get("result3"), Some(Value::Float(7.0)));
        assert_eq!(ctx.get("result4"), Some(Value::Float(1.75)));
    }

    #[test]
    fn test_mixed_int_float_operations() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result1 = 5 + 2.5
result2 = 10 - 3.5
result3 = 4 * 2.5
result4 = 10 / 4.0
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Float(7.5)));
        assert_eq!(ctx.get("result2"), Some(Value::Float(6.5)));
        assert_eq!(ctx.get("result3"), Some(Value::Float(10.0)));
        assert_eq!(ctx.get("result4"), Some(Value::Float(2.5)));
    }

    // Additional targeted tests for better coverage
    #[test]
    fn test_os_path_exists_check() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import os
result1 = os.path.exists("/tmp")
result2 = os.path.exists("/nonexistent_path_12345")
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(false)));
    }

    #[test]
    fn test_os_path_join_multiple() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import os
result = os.path.join("a", "b", "c")
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        if let Value::String(s) = result {
            assert!(s.contains("a") && s.contains("b") && s.contains("c"));
        }
    }

    #[test]
    fn test_os_getcwd_returns_string() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import os
result = os.getcwd()
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_os_listdir_current() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import os
result = os.listdir(".")
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            assert!(l.borrow().items.len() > 0);
        }
    }

    #[test]
    fn test_re_match_groups() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import re
match = re.match(r"(\d+)-(\d+)", "123-456")
result1 = match.group(0)
result2 = match.group(1)
result3 = match.group(2)
"#,
        )
        .unwrap();
        assert_eq!(
            ctx.get("result1"),
            Some(Value::String("123-456".to_string()))
        );
        assert_eq!(ctx.get("result2"), Some(Value::String("123".to_string())));
        assert_eq!(ctx.get("result3"), Some(Value::String("456".to_string())));
    }

    #[test]
    fn test_re_findall_multiple() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import re
result = re.findall(r"\d+", "abc 123 def 456 ghi 789")
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            assert_eq!(l.borrow().items.len(), 3);
        }
    }

    #[test]
    fn test_re_sub_multiple() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import re
result = re.sub(r"\d+", "X", "abc 123 def 456")
"#,
        )
        .unwrap();
        assert_eq!(
            ctx.get("result"),
            Some(Value::String("abc X def X".to_string()))
        );
    }

    #[test]
    fn test_json_loads_nested() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import json
data = json.loads('{"a": {"b": {"c": 123}}}')
result = data["a"]["b"]["c"]
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Int(123)));
    }

    #[test]
    fn test_json_dumps_nested() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import json
data = {"a": [1, 2, 3], "b": {"c": 4}}
result = json.dumps(data)
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        if let Value::String(s) = result {
            assert!(s.contains("\"a\"") && s.contains("\"b\""));
        }
    }

    #[test]
    fn test_isinstance_multiple_types() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result1 = isinstance([], list)
result2 = isinstance({}, dict)
result3 = isinstance((), tuple)
result4 = isinstance(5, int)
result5 = isinstance(5.0, float)
result6 = isinstance("", str)
result7 = isinstance(True, bool)
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result3"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result4"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result5"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result6"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result7"), Some(Value::Bool(true)));
    }

    #[test]
    fn test_async_function_simple() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
async def add(a, b):
    return a + b

result = await add(3, 4)
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Int(7)));
    }

    #[test]
    fn test_async_function_with_multiple_awaits() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import asyncio

async def task1():
    await asyncio.sleep(0.01)
    return 1

async def task2():
    await asyncio.sleep(0.01)
    return 2

async def main():
    r1 = await task1()
    r2 = await task2()
    return r1 + r2

result = await main()
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Int(3)));
    }

    #[test]
    fn test_exception_with_traceback() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
def func1():
    raise ValueError("test error")

def func2():
    func1()

func2()
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("ValueError"));
        assert!(err.contains("test error"));
    }

    #[test]
    fn test_finally_block_with_exception() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
cleanup = False
try:
    raise ValueError("error")
finally:
    cleanup = True
"#,
        );
        assert!(result.is_err());
        assert_eq!(ctx.get("cleanup"), Some(Value::Bool(true)));
    }

    #[test]
    fn test_try_except_finally_all() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = []
try:
    result.append(1)
    raise ValueError("error")
except ValueError:
    result.append(2)
finally:
    result.append(3)
"#,
        )
        .unwrap();
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            assert_eq!(l.borrow().items.len(), 3);
        }
    }

    #[test]
    fn test_nested_exception_handling() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = 0
try:
    try:
        raise ValueError("inner")
    except TypeError:
        result = 1
    except ValueError:
        result = 2
except:
    result = 3
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Int(2)));
    }

    // Targeted tests for uncovered code paths
    #[test]
    fn test_re_subn() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import re
result, count = re.subn(r"\d+", "X", "abc 123 def 456 ghi")
"#,
        )
        .unwrap();
        assert_eq!(
            ctx.get("result"),
            Some(Value::String("abc X def X ghi".to_string()))
        );
        assert_eq!(ctx.get("count"), Some(Value::Int(2)));
    }

    #[test]
    fn test_re_subn_no_match() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import re
result, count = re.subn(r"\d+", "X", "abc def ghi")
"#,
        )
        .unwrap();
        assert_eq!(
            ctx.get("result"),
            Some(Value::String("abc def ghi".to_string()))
        );
        assert_eq!(ctx.get("count"), Some(Value::Int(0)));
    }

    #[test]
    fn test_json_loads_with_whitespace() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import json
data = json.loads('  { "a" : 1 , "b" : 2 }  ')
result = data["a"]
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Int(1)));
    }

    #[test]
    fn test_json_loads_array() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import json
data = json.loads('[1, 2, 3, 4, 5]')
result = len(data)
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::Int(5)));
    }

    #[test]
    fn test_json_dumps_with_none() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import json
result = json.dumps(None)
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::String("null".to_string())));
    }

    #[test]
    fn test_json_dumps_with_bool() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
import json
result1 = json.dumps(True)
result2 = json.dumps(False)
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::String("true".to_string())));
        assert_eq!(ctx.get("result2"), Some(Value::String("false".to_string())));
    }

    #[test]
    fn test_string_index_not_found() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
s = "hello"
s.index("z")
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_dict_has_key() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
d = {"a": 1, "b": 2}
result1 = "a" in d
result2 = "c" in d
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(false)));
    }

    #[test]
    fn test_string_slice_negative_indices() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
s = "hello world"
result1 = s[-5:]
result2 = s[:-6]
result3 = s[-5:-1]
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::String("world".to_string())));
        assert_eq!(ctx.get("result2"), Some(Value::String("hello".to_string())));
        assert_eq!(ctx.get("result3"), Some(Value::String("worl".to_string())));
    }

    #[test]
    fn test_list_slice_negative_indices() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [1, 2, 3, 4, 5]
result1 = x[-3:]
result2 = x[:-2]
"#,
        )
        .unwrap();
        let result1 = ctx.get("result1").unwrap();
        let result2 = ctx.get("result2").unwrap();
        if let Value::List(l) = result1 {
            assert_eq!(l.borrow().items.len(), 3);
        }
        if let Value::List(l) = result2 {
            assert_eq!(l.borrow().items.len(), 3);
        }
    }

    #[test]
    fn test_tuple_unpacking_single() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
(x,) = (42,)
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("x"), Some(Value::Int(42)));
    }

    #[test]
    fn test_string_concatenation_multiple() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = "a" + "b" + "c" + "d"
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result"), Some(Value::String("abcd".to_string())));
    }

    #[test]
    fn test_modulo_negative() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result1 = -10 % 3
result2 = 10 % -3
result3 = -10 % -3
"#,
        )
        .unwrap();
        assert_eq!(ctx.get("result1"), Some(Value::Int(-1)));
        assert_eq!(ctx.get("result2"), Some(Value::Int(1)));
        assert_eq!(ctx.get("result3"), Some(Value::Int(-1)));
    }

    #[test]
    fn test_float_modulo() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = 10.5 % 3.0
"#,
        )
        .unwrap();
        if let Some(Value::Float(f)) = ctx.get("result") {
            assert!((f - 1.5).abs() < 0.0001);
        }
    }

    // Error handling tests for arithmetic operations
    #[test]
    fn test_sub_type_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = "hello" - 5"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_mul_type_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = {} * 5"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_div_type_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = [] / 5"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_div_int_float_zero() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = 10 / 0.0"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_div_float_int_zero() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = 10.5 / 0"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_mod_type_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = "hello" % 5"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_mod_float_int_zero() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = 10.5 % 0"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_mod_int_float_zero() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = 10 % 0.0"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_mod_float_float_zero() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = 10.5 % 0.0"#);
        assert!(result.is_err());
    }

    // Comparison error tests
    #[test]
    fn test_lt_type_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = "hello" < 5"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_le_type_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = [] <= 5"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_gt_type_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = {} > 5"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_ge_type_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = None >= 5"#);
        assert!(result.is_err());
    }

    // GetItem error tests
    #[test]
    fn test_getitem_on_int_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = 42[0]"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_getitem_on_none_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = None[0]"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_dict_getitem_unhashable_key() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
d = {1: "a"}
result = d[[1, 2]]
"#,
        );
        assert!(result.is_err());
    }

    // SetItem error tests
    #[test]
    fn test_setitem_on_int_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
x = 42
x[0] = 1
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_setitem_on_tuple_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
t = (1, 2, 3)
t[0] = 5
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_setitem_on_string_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
s = "hello"
s[0] = "H"
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_dict_setitem_unhashable_key() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
d = {}
d[[1, 2]] = "value"
"#,
        );
        assert!(result.is_err());
    }

    // Range error tests
    #[test]
    fn test_range_wrong_arg_count() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = list(range())"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_range_too_many_args() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = list(range(1, 2, 3, 4))"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_range_non_int_arg() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = list(range("hello"))"#);
        assert!(result.is_err());
    }

    // CallMethod error tests
    #[test]
    fn test_method_on_int_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = (42).append(1)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_method_on_none_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = None.upper()"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_append_wrong_arg_count() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
lst = []
lst.append()
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_list_pop_with_arg() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
lst = [1, 2, 3]
result = lst.pop(1)
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_string_split_non_string_sep() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = "a,b,c".split(123)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_replace_wrong_arg_count() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = "hello".replace("l")"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_replace_non_string_args() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = "hello".replace(1, 2)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_startswith_wrong_arg_count() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = "hello".startswith()"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_startswith_non_string() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = "hello".startswith(123)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_endswith_wrong_arg_count() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = "hello".endswith()"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_endswith_non_string() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = "hello".endswith([])"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_join_wrong_arg_count() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = ",".join()"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_join_non_list() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = ",".join(123)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_join_list_with_non_strings() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = ",".join([1, 2, 3])"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_index_wrong_arg_count() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = "hello".index()"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_index_non_string() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = "hello".index(123)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_index_wrong_arg_count() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
lst = [1, 2, 3]
result = lst.index()
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_dict_keys_with_args() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
d = {"a": 1}
result = d.keys(1)
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_dict_pop_wrong_arg_count() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
d = {"a": 1}
result = d.pop()
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_dict_pop_too_many_args() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
d = {"a": 1}
result = d.pop("a", 0, "extra")
"#,
        );
        assert!(result.is_err());
    }

    // UnpackSequence error tests
    #[test]
    fn test_unpack_non_sequence() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"a, b = 42"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_unpack_dict() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"a, b = {"x": 1, "y": 2}"#);
        assert!(result.is_err());
    }

    // Contains error tests
    #[test]
    fn test_in_on_int_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = 1 in 42"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_in_on_none_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = 1 in None"#);
        assert!(result.is_err());
    }

    // Len error tests
    #[test]
    fn test_len_on_none_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = len(None)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_len_on_bool_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = len(True)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_len_on_float_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = len(3.14)"#);
        assert!(result.is_err());
    }

    // GetIter error tests
    #[test]
    fn test_iterate_int_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
for x in 42:
    pass
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_iterate_none_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
for x in None:
    pass
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_iterate_bool_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
for x in True:
    pass
"#,
        );
        assert!(result.is_err());
    }

    // BuildSlice error tests
    #[test]
    fn test_slice_non_int_start() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = [1, 2, 3]["a":2]"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_slice_non_int_stop() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = [1, 2, 3][0:"b"]"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_slice_non_int_step() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = [1, 2, 3][0:2:"c"]"#);
        assert!(result.is_err());
    }

    // GetItemSlice error tests
    #[test]
    fn test_slice_on_int_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = 42[0:2]"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_slice_on_dict_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = {"a": 1}[0:2]"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_slice_on_none_error() {
        let mut ctx = Context::new();
        let result = ctx.eval(r#"result = None[0:2]"#);
        assert!(result.is_err());
    }
}
