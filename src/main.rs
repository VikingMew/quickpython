mod builtins;
mod bytecode;
mod compiler;
mod context;
mod serializer;
mod value;
mod vm;

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
    fn test_string_replace() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(r#""hello world".replace("world", "python")"#)
            .unwrap();
        assert_eq!(result, Value::String("hello python".to_string()));
    }

    #[test]
    fn test_string_join() {
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
    fn test_list_slice_negative_indices() {
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
}
