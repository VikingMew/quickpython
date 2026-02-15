// 待实现功能的测试
// 这些测试目前会失败，因为相关功能尚未实现
// 当功能实现后，这些测试应该能通过

#[cfg(test)]
mod tests_pending {
    use crate::context::Context;

    // ============================================
    // is / is not 操作符 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_is_operator() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = None
x is None
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    #[ignore]
    fn test_is_not_operator() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = 5
x is not None
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    #[ignore]
    fn test_is_identity_check() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
a = [1, 2, 3]
b = a
c = [1, 2, 3]
b is a and not (c is a)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    // ============================================
    // re.compile 返回对象的方法 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_re_compile_match() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
pattern = re.compile(r"\d+")
m = pattern.match("123abc")
m.group(0)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("123"));
    }

    #[test]
    #[ignore]
    fn test_re_compile_search() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
pattern = re.compile(r"world")
m = pattern.search("hello world")
m.group(0)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("world"));
    }

    #[test]
    #[ignore]
    fn test_re_compile_findall() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
pattern = re.compile(r"\d+")
matches = pattern.findall("a1b22c333")
len(matches)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(3));
    }

    // ============================================
    // re.finditer (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_re_finditer() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
matches = re.finditer(r"\d+", "a1b22c333")
results = []
for m in matches:
    results.append(m.group(0))
len(results)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(3));
    }

    // ============================================
    // re.escape (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_re_escape() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
escaped = re.escape("a.b*c?")
"." in escaped or "*" in escaped
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    // ============================================
    // re.sub/split 的 count 参数 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_re_sub_with_count() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
result = re.sub(r"\d", "X", "a1b2c3d4", 2)
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("aXbXc3d4"));
    }

    #[test]
    #[ignore]
    fn test_re_split_with_limit() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import re
parts = re.split(r"\s+", "a b c d e", 2)
len(parts)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(3));
    }

    // ============================================
    // Tuple 的 len() (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_tuple_len() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = (1, 2, 3)
len(x)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(3));
    }

    #[test]
    #[ignore]
    fn test_empty_tuple_len() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = ()
len(x)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(0));
    }

    // ============================================
    // JSON 特殊字符处理 (可能未完全实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_json_escape_sequences() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import json
data = json.loads('{"text": "hello\\nworld\\ttab"}')
"\\n" in data["text"] and "\\t" in data["text"]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    // ============================================
    // os.remove 实际文件操作 (需要文件 I/O)
    // ============================================

    #[test]
    #[ignore]
    fn test_os_remove_file() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
# Create a test file (需要 open() 和 write())
# f = open("test_remove.txt", "w")
# f.write("test")
# f.close()
# os.remove("test_remove.txt")
# not os.path.exists("test_remove.txt")
True  # Placeholder
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    // ============================================
    // dir() 内置函数 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_dir_builtin() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
attrs = dir(os)
"getcwd" in attrs
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    // ============================================
    // hasattr() 内置函数 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_hasattr_builtin() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
import os
hasattr(os, "getcwd")
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    // ============================================
    // open() 文件操作 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_open_read_file() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
f = open("Cargo.toml", "r")
content = f.read()
f.close()
len(content) > 0
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    #[ignore]
    fn test_open_write_file() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
f = open("test_write.txt", "w")
f.write("hello world")
f.close()
# Read it back
f = open("test_write.txt", "r")
content = f.read()
f.close()
import os
os.remove("test_write.txt")
content == "hello world"
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    // ============================================
    // with 语句 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_with_statement() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
with open("Cargo.toml", "r") as f:
    content = f.read()
len(content) > 0
        "#,
            )
            .unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    // ============================================
    // 类和对象 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_class_definition() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def distance(self):
        return (self.x ** 2 + self.y ** 2) ** 0.5

p = Point(3, 4)
p.distance()
        "#,
            )
            .unwrap();
        assert_eq!(result.as_float(), Some(5.0));
    }

    // ============================================
    // lambda 表达式 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_lambda_expression() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
add = lambda x, y: x + y
add(3, 4)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(7));
    }

    // ============================================
    // 生成器表达式 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_generator_expression() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
gen = (x * 2 for x in range(5))
result = []
for val in gen:
    result.append(val)
len(result)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(5));
    }

    // ============================================
    // 字典推导式 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_dict_comprehension() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
d = {x: x*2 for x in range(5)}
d[3]
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(6));
    }

    // ============================================
    // 集合 set (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_set_operations() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
s = {1, 2, 3, 4, 5}
len(s)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(5));
    }

    // ============================================
    // *args 和 **kwargs (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_varargs() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
def sum_all(*args):
    total = 0
    for x in args:
        total += x
    return total

sum_all(1, 2, 3, 4, 5)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(15));
    }

    // ============================================
    // 装饰器 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_decorator() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
def double(func):
    def wrapper(*args):
        return func(*args) * 2
    return wrapper

@double
def add(a, b):
    return a + b

add(3, 4)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(14));
    }

    // ============================================
    // yield 和生成器函数 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_generator_function() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
def count_up_to(n):
    i = 1
    while i <= n:
        yield i
        i += 1

result = []
for x in count_up_to(5):
    result.append(x)
len(result)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(5));
    }

    // ============================================
    // 多重赋值和解包 (部分未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_starred_unpacking() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
a, *b, c = [1, 2, 3, 4, 5]
len(b)
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(3));
    }

    // ============================================
    // 全局变量和 global 关键字 (可能未完全实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_global_keyword() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = 10
def modify():
    global x
    x = 20

modify()
x
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(20));
    }

    // ============================================
    // nonlocal 关键字 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_nonlocal_keyword() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
def outer():
    x = 10
    def inner():
        nonlocal x
        x = 20
    inner()
    return x

outer()
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(20));
    }

    // ============================================
    // assert 语句 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_assert_statement() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
assert 1 + 1 == 2
assert True
"passed"
        "#,
        );
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_assert_failure() {
        let mut ctx = Context::new();
        let result = ctx.eval("assert False");
        assert!(result.is_err());
    }

    // ============================================
    // del 语句 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_del_statement() {
        let mut ctx = Context::new();
        let result = ctx.eval(
            r#"
x = 10
del x
# x should be undefined now
        "#,
        );
        // Should succeed in deleting
        assert!(result.is_ok());
    }

    // ============================================
    // pass 语句 (可能已实现但未测试)
    // ============================================

    #[test]
    #[ignore]
    fn test_pass_statement() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
def empty_function():
    pass

empty_function()
"ok"
        "#,
            )
            .unwrap();
        assert_eq!(result.as_string(), Some("ok"));
    }

    // ============================================
    // 三元运算符 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_ternary_operator() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
x = 10
y = 20 if x > 5 else 30
y
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(20));
    }

    // ============================================
    // 海象运算符 := (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_walrus_operator() {
        let mut ctx = Context::new();
        let result = ctx
            .eval(
                r#"
if (n := 10) > 5:
    result = n * 2
result
        "#,
            )
            .unwrap();
        assert_eq!(result.as_int(), Some(20));
    }

    // ============================================
    // 高级内置函数 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_any_function() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result1 = any([False, False, True])
result2 = any([False, False, False])
result3 = any([True, True, True])
result4 = any([])
"#,
        )
        .unwrap();
        use crate::value::Value;
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(false)));
        assert_eq!(ctx.get("result3"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result4"), Some(Value::Bool(false)));
    }

    #[test]
    #[ignore]
    fn test_all_function() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result1 = all([True, True, True])
result2 = all([True, False, True])
result3 = all([False, False, False])
result4 = all([])
"#,
        )
        .unwrap();
        use crate::value::Value;
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(false)));
        assert_eq!(ctx.get("result3"), Some(Value::Bool(false)));
        assert_eq!(ctx.get("result4"), Some(Value::Bool(true)));
    }

    #[test]
    #[ignore]
    fn test_sorted_function() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [3, 1, 4, 1, 5, 9, 2, 6]
result = sorted(x)
"#,
        )
        .unwrap();
        use crate::value::Value;
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            let items = &l.borrow().items;
            assert_eq!(items[0], Value::Int(1));
            assert_eq!(items[7], Value::Int(9));
        }
    }

    #[test]
    #[ignore]
    fn test_sorted_reverse() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [3, 1, 4, 1, 5]
result = sorted(x, reverse=True)
"#,
        )
        .unwrap();
        use crate::value::Value;
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            let items = &l.borrow().items;
            assert_eq!(items[0], Value::Int(5));
            assert_eq!(items[4], Value::Int(1));
        }
    }

    #[test]
    #[ignore]
    fn test_reversed_function() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [1, 2, 3, 4, 5]
result = list(reversed(x))
"#,
        )
        .unwrap();
        use crate::value::Value;
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            let items = &l.borrow().items;
            assert_eq!(items[0], Value::Int(5));
            assert_eq!(items[4], Value::Int(1));
        }
    }

    #[test]
    #[ignore]
    fn test_map_function() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
def double(x):
    return x * 2

numbers = [1, 2, 3, 4, 5]
result = list(map(double, numbers))
"#,
        )
        .unwrap();
        use crate::value::Value;
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            let items = &l.borrow().items;
            assert_eq!(items.len(), 5);
            assert_eq!(items[0], Value::Int(2));
            assert_eq!(items[4], Value::Int(10));
        }
    }

    #[test]
    #[ignore]
    fn test_filter_function() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
def is_even(x):
    return x % 2 == 0

numbers = [1, 2, 3, 4, 5, 6]
result = list(filter(is_even, numbers))
"#,
        )
        .unwrap();
        use crate::value::Value;
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            let items = &l.borrow().items;
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Int(2));
            assert_eq!(items[2], Value::Int(6));
        }
    }

    #[test]
    #[ignore]
    fn test_zip_function() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
a = [1, 2, 3]
b = ["a", "b", "c"]
result = list(zip(a, b))
"#,
        )
        .unwrap();
        use crate::value::Value;
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            assert_eq!(l.borrow().items.len(), 3);
        }
    }

    #[test]
    #[ignore]
    fn test_zip_unequal_length() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
a = [1, 2, 3, 4, 5]
b = ["a", "b"]
result = list(zip(a, b))
"#,
        )
        .unwrap();
        use crate::value::Value;
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            assert_eq!(l.borrow().items.len(), 2);
        }
    }

    // ============================================
    // 列表和字典高级方法 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_list_count() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [1, 2, 3, 2, 2, 4]
result = x.count(2)
"#,
        )
        .unwrap();
        use crate::value::Value;
        assert_eq!(ctx.get("result"), Some(Value::Int(3)));
    }

    #[test]
    #[ignore]
    fn test_list_reverse() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [1, 2, 3, 4, 5]
x.reverse()
"#,
        )
        .unwrap();
        use crate::value::Value;
        let x = ctx.get("x").unwrap();
        if let Value::List(l) = x {
            let items = &l.borrow().items;
            assert_eq!(items[0], Value::Int(5));
            assert_eq!(items[4], Value::Int(1));
        }
    }

    #[test]
    #[ignore]
    fn test_list_sort() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [3, 1, 4, 1, 5, 9, 2, 6]
x.sort()
"#,
        )
        .unwrap();
        use crate::value::Value;
        let x = ctx.get("x").unwrap();
        if let Value::List(l) = x {
            let items = &l.borrow().items;
            assert_eq!(items[0], Value::Int(1));
            assert_eq!(items[7], Value::Int(9));
        }
    }

    #[test]
    #[ignore]
    fn test_list_sort_reverse() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
x = [3, 1, 4, 1, 5]
x.sort(reverse=True)
"#,
        )
        .unwrap();
        use crate::value::Value;
        let x = ctx.get("x").unwrap();
        if let Value::List(l) = x {
            let items = &l.borrow().items;
            assert_eq!(items[0], Value::Int(5));
            assert_eq!(items[4], Value::Int(1));
        }
    }

    #[test]
    #[ignore]
    fn test_list_multiplication() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = [1, 2] * 3
"#,
        )
        .unwrap();
        use crate::value::Value;
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            assert_eq!(l.borrow().items.len(), 6);
        }
    }

    #[test]
    #[ignore]
    fn test_dict_values() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
d = {"a": 1, "b": 2, "c": 3}
values = list(d.values())
"#,
        )
        .unwrap();
        use crate::value::Value;
        let values = ctx.get("values").unwrap();
        if let Value::List(l) = values {
            assert_eq!(l.borrow().items.len(), 3);
        }
    }

    #[test]
    #[ignore]
    fn test_dict_items() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
d = {"a": 1, "b": 2}
items = list(d.items())
"#,
        )
        .unwrap();
        use crate::value::Value;
        let items = ctx.get("items").unwrap();
        if let Value::List(l) = items {
            assert_eq!(l.borrow().items.len(), 2);
        }
    }

    #[test]
    #[ignore]
    fn test_dict_pop_with_default() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
d = {"a": 1}
result = d.pop("b", 99)
"#,
        )
        .unwrap();
        use crate::value::Value;
        assert_eq!(ctx.get("result"), Some(Value::Int(99)));
    }

    #[test]
    #[ignore]
    fn test_nested_list_comprehension() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
matrix = [[1, 2], [3, 4], [5, 6]]
result = [x for row in matrix for x in row]
"#,
        )
        .unwrap();
        use crate::value::Value;
        let result = ctx.get("result").unwrap();
        if let Value::List(l) = result {
            let items = &l.borrow().items;
            assert_eq!(items.len(), 6);
            assert_eq!(items[0], Value::Int(1));
            assert_eq!(items[5], Value::Int(6));
        }
    }

    // ============================================
    // 字符串高级方法 (未实现)
    // ============================================

    #[test]
    #[ignore]
    fn test_string_count() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
s = "hello hello world"
result = s.count("hello")
"#,
        )
        .unwrap();
        use crate::value::Value;
        assert_eq!(ctx.get("result"), Some(Value::Int(2)));
    }

    #[test]
    #[ignore]
    fn test_string_isdigit() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result1 = "123".isdigit()
result2 = "12a".isdigit()
result3 = "".isdigit()
"#,
        )
        .unwrap();
        use crate::value::Value;
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(false)));
        assert_eq!(ctx.get("result3"), Some(Value::Bool(false)));
    }

    #[test]
    #[ignore]
    fn test_string_isalpha() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result1 = "abc".isalpha()
result2 = "abc123".isalpha()
result3 = "".isalpha()
"#,
        )
        .unwrap();
        use crate::value::Value;
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(false)));
        assert_eq!(ctx.get("result3"), Some(Value::Bool(false)));
    }

    #[test]
    #[ignore]
    fn test_string_isalnum() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result1 = "abc123".isalnum()
result2 = "abc 123".isalnum()
result3 = "".isalnum()
"#,
        )
        .unwrap();
        use crate::value::Value;
        assert_eq!(ctx.get("result1"), Some(Value::Bool(true)));
        assert_eq!(ctx.get("result2"), Some(Value::Bool(false)));
        assert_eq!(ctx.get("result3"), Some(Value::Bool(false)));
    }

    #[test]
    #[ignore]
    fn test_string_multiplication() {
        let mut ctx = Context::new();
        ctx.eval(
            r#"
result = "ab" * 3
"#,
        )
        .unwrap();
        use crate::value::Value;
        assert_eq!(ctx.get("result"), Some(Value::String("ababab".to_string())));
    }
}
