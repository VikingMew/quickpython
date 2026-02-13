# 模块导入系统规范

## 概述

定义 QuickPython 的模块导入机制，支持三种类型的模块：
1. 内置模块（Built-in modules）- 用 Rust 实现的标准库模块
2. Pure Python 包（Pure wheels）- 纯 Python 实现的第三方包
3. Rust 扩展模块（Rust extensions）- 用 Rust 实现的自定义扩展模块

**注意**：Pure Python wheels 需要在 Rust 代码中显式指定路径，不支持自动搜索和安装。

## 导入语法

### 基本导入

```python
# 导入整个模块
import json
import math
import mymodule

# 使用模块
data = json.loads('{"key": "value"}')
result = math.sqrt(16)
```

### 选择性导入

```python
# 从模块导入特定名称
from json import loads, dumps
from math import sqrt, pi

# 直接使用
data = loads('{"key": "value"}')
result = sqrt(16)
```

### 别名导入

```python
# 模块别名
import json as j
import mymodule as mm

# 使用别名
data = j.loads('...')
result = mm.process()

# 名称别名
from json import loads as parse
data = parse('...')
```

## 模块类型

### 1. 内置模块（Built-in Modules）

内置模块是用 Rust 实现并编译到 QuickPython 二进制文件中的模块。

#### 示例：json 模块

```python
import json

# json.loads - 解析 JSON 字符串
data = json.loads('{"name": "Alice", "age": 30}')
print(data["name"])  # Alice

# json.dumps - 序列化为 JSON
obj = {"x": 1, "y": 2}
text = json.dumps(obj)
print(text)  # {"x": 1, "y": 2}
```

#### 示例：math 模块

```python
import math

# 数学函数
print(math.sqrt(16))      # 4.0
print(math.pow(2, 3))     # 8.0
print(math.floor(3.7))    # 3
print(math.ceil(3.2))     # 4

# 数学常量
print(math.pi)            # 3.141592653589793
print(math.e)             # 2.718281828459045
```

#### 实现方式

内置模块在 Rust 中实现为模块注册表：

```rust
// src/builtins/mod.rs
pub mod json;
pub mod math;
pub mod os;

pub fn get_builtin_module(name: &str) -> Option<Module> {
    match name {
        "json" => Some(json::create_module()),
        "math" => Some(math::create_module()),
        "os" => Some(os::create_module()),
        _ => None,
    }
}
```

每个内置模块提供一个创建函数：

```rust
// src/builtins/json.rs
pub fn create_module() -> Module {
    let mut module = Module::new("json");
    
    // 注册函数
    module.add_function("loads", json_loads);
    module.add_function("dumps", json_dumps);
    
    module
}

fn json_loads(args: Vec<Value>) -> Result<Value, Value> {
    // 实现 JSON 解析
    // ...
}

fn json_dumps(args: Vec<Value>) -> Result<Value, Value> {
    // 实现 JSON 序列化
    // ...
}
```

#### 内置模块列表

**计划支持的内置模块**：

| 模块名 | 功能 | 优先级 |
|--------|------|--------|
| json | JSON 解析和序列化 | P0 |
| math | 数学函数和常量 | P0 |
| re | 正则表达式 | P1 |
| os | 文件系统操作 | P1 |

### 2. Pure Python 包（Pure Wheels）

Pure Python 包是完全用 Python 编写的第三方包，通常以 `.whl` 文件分发。

#### 包结构

```
mypackage/
├── __init__.py
├── module1.py
├── module2.py
└── subpackage/
    ├── __init__.py
    └── module3.py
```

#### 导入示例

```python
# 导入包
import mypackage
mypackage.function1()

# 导入子模块
import mypackage.module1
mypackage.module1.function2()

# 从包导入
from mypackage import module1
module1.function2()

# 从子包导入
from mypackage.subpackage import module3
module3.function3()
```

#### 包的 __init__.py

```python
# mypackage/__init__.py
# 包初始化代码

__version__ = "1.0.0"

# 导出公共 API
from .module1 import function1
from .module2 import function2

__all__ = ["function1", "function2"]
```

#### 包路径配置

Pure Python 包需要在 Rust 代码中显式指定 wheel 文件路径。不支持自动搜索和 `pip install` 风格的包管理。

**在 Rust 中配置 wheel 文件**：

```rust
// src/main.rs 或应用初始化代码
use quickpython::Context;

fn main() {
    let mut ctx = Context::new();
    
    // 添加 Python wheel 文件
    ctx.add_python_path("./wheels/mypackage-1.0.0-py3-none-any.whl");
    ctx.add_python_path("./wheels/another-2.0.0-py3-none-any.whl");
    
    // 现在可以导入这些包
    ctx.eval("import mypackage").unwrap();
}
```

**Wheel 文件结构**：

```
wheels/
├── mypackage-1.0.0-py3-none-any.whl
└── another-2.0.0-py3-none-any.whl

# wheel 文件内部结构：
mypackage-1.0.0-py3-none-any.whl
├── mypackage/
│   ├── __init__.py
│   ├── module1.py
│   └── module2.py
└── mypackage-1.0.0.dist-info/
    └── METADATA
```

**环境变量支持（可选）**：

```bash
# 通过环境变量指定额外的 wheel 文件（用冒号分隔）
export QUICKPYTHONPATH=./wheels/pkg1.whl:./wheels/pkg2.whl

# 运行脚本
quickpython run script.py
```

在 Rust 中读取环境变量：

```rust
use std::env;

fn setup_python_paths(ctx: &mut Context) {
    // 读取 QUICKPYTHONPATH 环境变量
    if let Ok(paths) = env::var("QUICKPYTHONPATH") {
        for path in paths.split(':') {
            ctx.add_python_path(path);
        }
    }
    
    // 添加默认 wheel 文件
    ctx.add_python_path("./wheels/default-1.0.0-py3-none-any.whl");
}
```

### 3. Rust 模块（Rust Modules）

Rust 模块是直接编译到 QuickPython 中的 Rust 代码，通过注册机制暴露给 Python。

#### 使用场景

- 性能关键的计算（图像处理、数值计算）
- 系统级操作（文件系统、网络）
- 与 Rust 生态系统集成（使用现有的 Rust crates）
- 扩展 QuickPython 的核心功能

#### 导入示例

```python
# 导入 Rust 模块
import myrust

# 调用 Rust 实现的函数
result = myrust.fast_compute([1, 2, 3, 4, 5])
print(result)

# 使用 Rust 实现的类
obj = myrust.MyClass(10)
value = obj.get_value()
```

#### 实现方式：模块注册

Rust 模块通过注册机制在 QuickPython 启动时注册：

```rust
// src/builtins/myrust.rs
use crate::value::{Value, Module};
use crate::value::ExceptionType;

// 创建模块
pub fn create_module() -> Module {
    let mut module = Module::new("myrust");
    
    // 注册函数
    module.add_function("fast_compute", fast_compute);
    module.add_function("process_data", process_data);
    
    // 注册常量
    module.add_constant("VERSION", Value::String("1.0.0".to_string()));
    module.add_constant("MAX_SIZE", Value::Int(1000));
    
    module
}

// 实现函数
fn fast_compute(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::error(
            ExceptionType::TypeError,
            "fast_compute() takes exactly 1 argument"
        ));
    }
    
    let list = args[0].as_list()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "argument must be a list"
        ))?;
    
    // 高性能计算
    let sum: i32 = list.borrow().items.iter()
        .filter_map(|v| v.as_int())
        .sum();
    
    Ok(Value::Int(sum * 2))
}

fn process_data(args: Vec<Value>) -> Result<Value, Value> {
    // 实现数据处理逻辑
    // ...
    Ok(Value::None)
}
```

#### 注册到 QuickPython

内置模块在编译时自动注册，用户无需手动调用。扩展模块需要用户显式注册。

```rust
// src/builtins/mod.rs
pub mod json;
pub mod math;
pub mod re;
pub mod os;

/// 内置模块列表（编译时确定）
const BUILTIN_MODULES: &[&str] = &["json", "math", "re", "os"];

/// 检查是否是内置模块
pub fn is_builtin_module(name: &str) -> bool {
    BUILTIN_MODULES.contains(&name)
}

/// 获取内置模块
/// 内置模块是固定的，编译时确定，用户不需要手动注册
pub fn get_builtin_module(name: &str) -> Module {
    match name {
        "json" => json::create_module(),
        "math" => math::create_module(),
        "re" => re::create_module(),
        "os" => os::create_module(),
        _ => panic!("Unknown builtin module: {}", name),
    }
}
```

**用户自定义扩展模块**需要显式注册：

```rust
// 用户代码：src/main.rs 或应用初始化
use quickpython::{Context, Module};

fn main() {
    let mut ctx = Context::new();
    
    // 注册自定义 Rust 扩展模块
    ctx.register_extension_module("myrust", create_myrust_module());
    
    // 现在可以导入
    ctx.eval("import json").unwrap();      // 内置模块，自动可用
    ctx.eval("import myrust").unwrap();    // 扩展模块，需要注册
}

fn create_myrust_module() -> Module {
    let mut module = Module::new("myrust");
    module.add_function("my_function", my_function);
    module
}

fn my_function(args: Vec<Value>) -> Result<Value, Value> {
    // 实现
    Ok(Value::None)
}
```

在 VM 的模块加载器中：

```rust
// src/vm.rs
fn load_module(&mut self, name: &str) -> Result<Module, Value> {
    // 1. 检查是否已加载
    if let Some(module) = self.loaded_modules.get(name) {
        return Ok(module.clone());
    }
    
    // 2. 加载内置模块（自动可用，无需注册）
    if builtins::is_builtin_module(name) {
        let module = builtins::get_builtin_module(name);
        self.loaded_modules.insert(name.to_string(), module.clone());
        return Ok(module);
    }
    
    // 3. 加载用户注册的扩展模块
    if let Some(module) = self.extension_modules.get(name) {
        self.loaded_modules.insert(name.to_string(), module.clone());
        return Ok(module.clone());
    }
    
    // 4. 加载 Python 模块
    if let Some(module) = self.load_python_module(name)? {
        self.loaded_modules.insert(name.to_string(), module.clone());
        return Ok(module);
    }
    
    // 5. 未找到模块
    Err(Value::error(
        ExceptionType::ImportError,
        &format!("No module named '{}'", name)
    ))
}
```

Context API：

```rust
// src/context.rs
impl Context {
    /// 注册扩展模块
    pub fn register_extension_module(&mut self, name: &str, module: Module) {
        self.vm.extension_modules.insert(name.to_string(), module);
    }
    
    /// 添加 Python wheel 文件路径
    pub fn add_python_path(&mut self, path: &str) {
        self.vm.python_paths.push(path.to_string());
    }
}
```

#### 使用外部 Crate

Rust 模块可以使用任何 Rust crate：

```rust
// Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
image = "0.24"
reqwest = { version = "0.11", features = ["blocking"] }

// src/builtins/http.rs
use reqwest::blocking::Client;

pub fn create_module() -> Module {
    let mut module = Module::new("http");
    module.add_function("get", http_get);
    module
}

fn http_get(args: Vec<Value>) -> Result<Value, Value> {
    let url = args[0].as_string()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "url must be a string"
        ))?;
    
    // 使用 reqwest crate
    let client = Client::new();
    let response = client.get(&url)
        .send()
        .map_err(|e| Value::error(
            ExceptionType::RuntimeError,
            &format!("HTTP request failed: {}", e)
        ))?;
    
    let text = response.text()
        .map_err(|e| Value::error(
            ExceptionType::RuntimeError,
            &format!("Failed to read response: {}", e)
        ))?;
    
    Ok(Value::String(text))
}
```

在 Python 中使用：

```python
import http

# 使用 Rust 的 reqwest crate 发送 HTTP 请求
response = http.get("https://api.example.com/data")
print(response)
```

#### 高级特性：类和对象

Rust 模块可以暴露类和对象：

```rust
// src/builtins/myrust.rs
use std::sync::{Arc, Mutex};

// Rust 结构体
pub struct Counter {
    value: Arc<Mutex<i32>>,
}

impl Counter {
    fn new(initial: i32) -> Self {
        Counter {
            value: Arc::new(Mutex::new(initial)),
        }
    }
    
    fn increment(&self) -> i32 {
        let mut val = self.value.lock().unwrap();
        *val += 1;
        *val
    }
    
    fn get(&self) -> i32 {
        *self.value.lock().unwrap()
    }
}

// 创建模块
pub fn create_module() -> Module {
    let mut module = Module::new("myrust");
    
    // 注册类构造器
    module.add_function("Counter", counter_new);
    
    module
}

// 构造器
fn counter_new(args: Vec<Value>) -> Result<Value, Value> {
    let initial = if args.is_empty() {
        0
    } else {
        args[0].as_int()
            .ok_or_else(|| Value::error(
                ExceptionType::TypeError,
                "initial value must be an integer"
            ))?
    };
    
    let counter = Counter::new(initial);
    
    // 创建 Python 对象
    let mut obj = Object::new("Counter");
    obj.add_method("increment", move |_args| {
        Ok(Value::Int(counter.increment()))
    });
    obj.add_method("get", move |_args| {
        Ok(Value::Int(counter.get()))
    });
    
    Ok(Value::Object(Rc::new(RefCell::new(obj))))
}
```

在 Python 中使用：

```python
import myrust

# 创建对象
counter = myrust.Counter(10)

# 调用方法
print(counter.get())        # 10
print(counter.increment())  # 11
print(counter.increment())  # 12
print(counter.get())        # 12
```

#### 模块开发工作流

**1. 创建模块函数**

```rust
// 用户代码：src/my_extensions.rs
use quickpython::{Value, Module};
use quickpython::ExceptionType;

pub fn create_mymodule() -> Module {
    let mut module = Module::new("mymodule");
    module.add_function("my_function", my_function);
    module
}

fn my_function(args: Vec<Value>) -> Result<Value, Value> {
    // 实现逻辑
    Ok(Value::None)
}
```

**2. 注册模块**

```rust
// src/main.rs
mod my_extensions;

fn main() {
    let mut ctx = Context::new();
    
    // 注册扩展模块
    ctx.register_extension_module("mymodule", my_extensions::create_mymodule());
    
    // 使用模块
    ctx.eval("import mymodule; mymodule.my_function()").unwrap();
}
```

**3. 测试模块**

```bash
# 编译和运行
cargo build
cargo run
```

**4. 添加测试**

```rust
// src/main.rs
#[test]
fn test_mymodule() {
    let mut ctx = Context::new();
    ctx.register_extension_module("mymodule", my_extensions::create_mymodule());
    
    ctx.eval("import mymodule").unwrap();
    let result = ctx.eval("mymodule.my_function()").unwrap();
    // 断言结果
}
```

#### 模块 API 参考

```rust
// 模块结构
pub struct Module {
    pub name: String,
    pub attributes: HashMap<String, Value>,
    pub doc: Option<String>,
}

impl Module {
    // 创建新模块
    pub fn new(name: &str) -> Self;
    
    // 添加函数
    pub fn add_function(&mut self, name: &str, func: NativeFunction);
    
    // 添加常量
    pub fn add_constant(&mut self, name: &str, value: Value);
    
    // 添加子模块
    pub fn add_module(&mut self, name: &str, module: Module);
    
    // 设置文档字符串
    pub fn set_doc(&mut self, doc: &str);
}

// 函数签名
pub type NativeFunction = fn(Vec<Value>) -> Result<Value, Value>;

// 辅助宏
#[macro_export]
macro_rules! py_function {
    ($name:ident, $args:ident, $body:block) => {
        fn $name($args: Vec<Value>) -> Result<Value, Value> $body
    };
}
```

#### 类型对应关系

Python 类型和 Rust 类型之间的映射：

| Python 类型 | Rust Value 枚举 | Rust 原生类型 | 转换方法 |
|------------|----------------|--------------|---------|
| `int` | `Value::Int(i32)` | `i32` | `.as_int()` |
| `float` | `Value::Float(f64)` | `f64` | `.as_float()` |
| `bool` | `Value::Bool(bool)` | `bool` | `.as_bool()` |
| `str` | `Value::String(String)` | `String` | `.as_string()` |
| `None` | `Value::None` | - | - |
| `list` | `Value::List(Rc<RefCell<ListValue>>)` | `Vec<Value>` | `.as_list()` |
| `dict` | `Value::Dict(Rc<RefCell<HashMap<DictKey, Value>>>)` | `HashMap` | `.as_dict()` |
| `function` | `Value::Function(Function)` | - | - |
| `module` | `Value::Module(Rc<RefCell<Module>>)` | - | - |

**从 Python 到 Rust**：

```rust
fn my_function(args: Vec<Value>) -> Result<Value, Value> {
    // 获取整数参数
    let num = args[0].as_int()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "argument must be an integer"
        ))?;
    
    // 获取字符串参数
    let text = args[1].as_string()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "argument must be a string"
        ))?;
    
    // 获取列表参数
    let list = args[2].as_list()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "argument must be a list"
        ))?;
    
    // 访问列表元素
    let items = &list.borrow().items;
    for item in items {
        // 处理每个元素
    }
    
    // 获取字典参数
    let dict = args[3].as_dict()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "argument must be a dict"
        ))?;
    
    // 访问字典
    let dict_ref = dict.borrow();
    if let Some(value) = dict_ref.get(&DictKey::String("key".to_string())) {
        // 处理值
    }
    
    Ok(Value::None)
}
```

**从 Rust 到 Python**：

```rust
fn create_values() -> Result<Value, Value> {
    // 返回整数
    Ok(Value::Int(42))
    
    // 返回浮点数
    Ok(Value::Float(3.14))
    
    // 返回字符串
    Ok(Value::String("hello".to_string()))
    
    // 返回布尔值
    Ok(Value::Bool(true))
    
    // 返回 None
    Ok(Value::None)
    
    // 返回列表
    let items = vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
    ];
    Ok(Value::new_list(items))
    
    // 返回字典
    let mut map = HashMap::new();
    map.insert(DictKey::String("name".to_string()), Value::String("Alice".to_string()));
    map.insert(DictKey::Int(1), Value::Int(100));
    Ok(Value::Dict(Rc::new(RefCell::new(map))))
}
```

**类型检查辅助函数**：

```rust
// 检查参数数量
fn check_arg_count(args: &[Value], expected: usize) -> Result<(), Value> {
    if args.len() != expected {
        return Err(Value::error(
            ExceptionType::TypeError,
            &format!("expected {} arguments, got {}", expected, args.len())
        ));
    }
    Ok(())
}

// 检查参数类型
fn expect_int(value: &Value, arg_name: &str) -> Result<i32, Value> {
    value.as_int()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            &format!("{} must be an integer", arg_name)
        ))
}

fn expect_string(value: &Value, arg_name: &str) -> Result<String, Value> {
    value.as_string()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            &format!("{} must be a string", arg_name)
        ))
}

// 使用示例
fn my_function(args: Vec<Value>) -> Result<Value, Value> {
    check_arg_count(&args, 2)?;
    
    let x = expect_int(&args[0], "x")?;
    let name = expect_string(&args[1], "name")?;
    
    // 处理逻辑
    Ok(Value::None)
}
```

**复杂类型转换示例**：

```rust
// 将 Rust 结构体转换为 Python 字典
#[derive(Debug)]
struct Person {
    name: String,
    age: i32,
    email: String,
}

fn person_to_dict(person: &Person) -> Value {
    let mut map = HashMap::new();
    map.insert(
        DictKey::String("name".to_string()),
        Value::String(person.name.clone())
    );
    map.insert(
        DictKey::String("age".to_string()),
        Value::Int(person.age)
    );
    map.insert(
        DictKey::String("email".to_string()),
        Value::String(person.email.clone())
    );
    Value::Dict(Rc::new(RefCell::new(map)))
}

// 将 Python 字典转换为 Rust 结构体
fn dict_to_person(value: &Value) -> Result<Person, Value> {
    let dict = value.as_dict()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "expected a dict"
        ))?;
    
    let dict_ref = dict.borrow();
    
    let name = dict_ref.get(&DictKey::String("name".to_string()))
        .and_then(|v| v.as_string())
        .ok_or_else(|| Value::error(
            ExceptionType::KeyError,
            "missing or invalid 'name' field"
        ))?;
    
    let age = dict_ref.get(&DictKey::String("age".to_string()))
        .and_then(|v| v.as_int())
        .ok_or_else(|| Value::error(
            ExceptionType::KeyError,
            "missing or invalid 'age' field"
        ))?;
    
    let email = dict_ref.get(&DictKey::String("email".to_string()))
        .and_then(|v| v.as_string())
        .ok_or_else(|| Value::error(
            ExceptionType::KeyError,
            "missing or invalid 'email' field"
        ))?;
    
    Ok(Person { name, age, email })
}
```

**使用 serde 进行序列化**：

```rust
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct Config {
    host: String,
    port: u16,
    debug: bool,
}

// 从 Python 字典创建 Rust 结构体
fn config_from_dict(value: &Value) -> Result<Config, Value> {
    // 先转换为 JSON
    let dict = value.as_dict()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "expected a dict"
        ))?;
    
    // 构建 JSON 对象
    let mut json_map = serde_json::Map::new();
    for (key, val) in dict.borrow().iter() {
        if let DictKey::String(k) = key {
            let json_val = value_to_json(val)?;
            json_map.insert(k.clone(), json_val);
        }
    }
    
    // 反序列化
    let config: Config = serde_json::from_value(serde_json::Value::Object(json_map))
        .map_err(|e| Value::error(
            ExceptionType::ValueError,
            &format!("invalid config: {}", e)
        ))?;
    
    Ok(config)
}

// 将 Rust 结构体转换为 Python 字典
fn config_to_dict(config: &Config) -> Result<Value, Value> {
    let json = serde_json::to_value(config)
        .map_err(|e| Value::error(
            ExceptionType::RuntimeError,
            &format!("serialization failed: {}", e)
        ))?;
    
    json_to_value(&json)
}
```

#### 示例：图像处理模块

```rust
// src/builtins/image.rs
use image::{DynamicImage, ImageBuffer, Rgba};

pub fn create_module() -> Module {
    let mut module = Module::new("image");
    
    module.add_function("load", image_load);
    module.add_function("save", image_save);
    module.add_function("blur", image_blur);
    module.add_function("resize", image_resize);
    
    module
}

fn image_load(args: Vec<Value>) -> Result<Value, Value> {
    let path = args[0].as_string()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "path must be a string"
        ))?;
    
    let img = image::open(&path)
        .map_err(|e| Value::error(
            ExceptionType::RuntimeError,
            &format!("Failed to load image: {}", e)
        ))?;
    
    // 将图像包装为 Python 对象
    Ok(Value::RustObject(Box::new(img)))
}

fn image_blur(args: Vec<Value>) -> Result<Value, Value> {
    let img = args[0].as_rust_object::<DynamicImage>()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "first argument must be an image"
        ))?;
    
    let radius = args[1].as_float()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "radius must be a number"
        ))?;
    
    let blurred = img.blur(radius as f32);
    Ok(Value::RustObject(Box::new(blurred)))
}

// ... 其他函数
```

在 Python 中使用：

```python
import image

# 加载图像（使用 Rust 的 image crate）
img = image.load("photo.jpg")

# 应用模糊（Rust 实现，速度快）
img = image.blur(img, 5.0)

# 调整大小
img = image.resize(img, 800, 600)

# 保存
image.save(img, "output.jpg")
```

#### 与其他 Rust 项目集成

QuickPython 可以作为库被其他 Rust 项目使用：

```rust
// 其他 Rust 项目的 Cargo.toml
[dependencies]
quickpython = { path = "../quickpython" }

// main.rs
use quickpython::{Context, Module, Value};

fn main() {
    let mut ctx = Context::new();
    
    // 创建自定义模块
    let mut my_module = Module::new("myapp");
    my_module.add_function("native_function", |args| {
        println!("Called from Python!");
        Ok(Value::None)
    });
    
    // 注册模块
    ctx.register_extension_module("myapp", my_module);
    
    // 运行 Python 代码
    ctx.eval(r#"
import myapp
myapp.native_function()
    "#).unwrap();
}
```


## 导入机制实现

### 字节码指令

```rust
pub enum Instruction {
    // ... 现有指令
    
    // 导入指令
    Import(String),              // import module
    ImportFrom(String, Vec<String>),  // from module import names
    ImportAs(String, String),    // import module as alias
}
```

### 编译器实现

```rust
// src/compiler.rs
ast::Stmt::Import(import) => {
    for alias in &import.names {
        let module_name = alias.name.to_string();
        let as_name = alias.asname.as_ref()
            .map(|n| n.to_string())
            .unwrap_or_else(|| module_name.clone());
        
        bytecode.push(Instruction::Import(module_name));
        bytecode.push(Instruction::SetGlobal(as_name));
    }
    Ok(())
}

ast::Stmt::ImportFrom(import_from) => {
    let module_name = import_from.module.as_ref()
        .map(|m| m.to_string())
        .unwrap_or_else(|| String::new());
    
    let names: Vec<String> = import_from.names.iter()
        .map(|alias| alias.name.to_string())
        .collect();
    
    bytecode.push(Instruction::ImportFrom(module_name, names));
    
    // 绑定导入的名称
    for alias in &import_from.names {
        let name = alias.name.to_string();
        let as_name = alias.asname.as_ref()
            .map(|n| n.to_string())
            .unwrap_or_else(|| name.clone());
        
        bytecode.push(Instruction::SetGlobal(as_name));
    }
    
    Ok(())
}
```

### VM 实现

```rust
// src/vm.rs
Instruction::Import(module_name) => {
    let module = self.load_module(module_name)?;
    self.stack.push(Value::Module(module));
    *ip += 1;
}

Instruction::ImportFrom(module_name, names) => {
    let module = self.load_module(module_name)?;
    
    for name in names {
        let value = module.get_attribute(name)
            .ok_or_else(|| Value::error(
                ExceptionType::ImportError,
                &format!("cannot import name '{}' from '{}'", name, module_name)
            ))?;
        
        self.stack.push(value);
    }
    
    *ip += 1;
}

// 模块加载器
fn load_module(&mut self, name: &str) -> Result<Module, Value> {
    // 1. 检查是否已加载
    if let Some(module) = self.loaded_modules.get(name) {
        return Ok(module.clone());
    }
    
    // 2. 尝试加载内置模块
    if let Some(module) = builtins::get_builtin_module(name) {
        self.loaded_modules.insert(name.to_string(), module.clone());
        return Ok(module);
    }
    
    // 3. 尝试加载 Rust 扩展模块
    if let Some(module) = self.load_rust_extension(name)? {
        self.loaded_modules.insert(name.to_string(), module.clone());
        return Ok(module);
    }
    
    // 4. 尝试加载 Python 模块
    if let Some(module) = self.load_python_module(name)? {
        self.loaded_modules.insert(name.to_string(), module.clone());
        return Ok(module);
    }
    
    // 5. 未找到模块
    Err(Value::error(
        ExceptionType::ImportError,
        &format!("No module named '{}'", name)
    ))
}
```

### 模块对象

```rust
// src/value.rs
pub struct Module {
    pub name: String,
    pub attributes: HashMap<String, Value>,
    pub doc: Option<String>,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Module {
            name: name.to_string(),
            attributes: HashMap::new(),
            doc: None,
        }
    }
    
    pub fn add_function(&mut self, name: &str, func: NativeFunction) {
        self.attributes.insert(
            name.to_string(),
            Value::NativeFunction(func)
        );
    }
    
    pub fn add_constant(&mut self, name: &str, value: Value) {
        self.attributes.insert(name.to_string(), value);
    }
    
    pub fn get_attribute(&self, name: &str) -> Option<Value> {
        self.attributes.get(name).cloned()
    }
}

// 扩展 Value 枚举
pub enum Value {
    // ... 现有类型
    Module(Rc<RefCell<Module>>),
    NativeFunction(NativeFunction),
}

pub type NativeFunction = fn(Vec<Value>) -> Result<Value, Value>;
```

## 模块搜索算法

### 搜索顺序

```
1. sys.modules (已加载模块缓存)
   ↓
2. Built-in modules (内置模块)
   在 get_builtin_module() 中查找
   包括：json, math, re, os
   ↓
3. Extension modules (Rust 扩展模块)
   在 get_extension_module() 中查找
   用户自定义的 Rust 模块
   ↓
4. Python modules (Pure Python 包)
   在 Rust 中通过 add_python_path() 配置的 wheel 文件中搜索
   ↓
5. ImportError (未找到)
```

### 模块类型总结

| 模块类型 | 注册方式 | 示例 |
|---------|---------|------|
| 内置模块 | 在 `get_builtin_module()` 中硬编码 | json, math, re, os |
| Rust 扩展 | 在 `get_extension_module()` 中注册 | 用户自定义的 Rust 模块 |
| Pure Python | 通过 `ctx.add_python_path("path/to/package.whl")` 指定 wheel 文件 | 第三方 Python 包 |

### 路径解析

```python
# 绝对导入
import json
import mymodule
from mymodule import function

# 支持包路径
from package.subpackage import module
```

## 异常处理

### ImportError

```python
try:
    import nonexistent_module
except ImportError as e:
    print("Failed to import:", e)
```

### ModuleNotFoundError

```python
try:
    from mymodule import nonexistent_function
except ImportError as e:
    print("Cannot import:", e)
```

## 性能考虑

### 模块缓存

- 已加载的模块存储在 `sys.modules` 中
- 重复导入直接返回缓存的模块对象
- 避免重复解析和执行

### 延迟加载

```python
# 延迟导入（在函数内）
def process_data():
    import json  # 仅在需要时导入
    return json.loads(data)
```

### 预编译

```bash
# 预编译 Python 模块为字节码
quickpython compile mymodule.py -o mymodule.pyc

# 导入时优先使用 .pyc
import mymodule  # 自动使用 mymodule.pyc
```

## 与 Python 标准的差异

### 不支持的特性

1. **相对导入** - `from . import module` 不支持
2. **动态导入** - `importlib.import_module()` 不支持
3. **包资源** - `pkg_resources` 不支持
4. **命名空间包** - PEP 420 不支持
5. **导入钩子** - `sys.meta_path` 不支持
6. **__path__ 属性** - 包的 `__path__` 属性不支持

### 不支持的第三方库

QuickPython 不支持依赖 C 扩展的第三方库，包括但不限于：

- **numpy** - 依赖 C/Fortran 扩展，不支持
- **pandas** - 依赖 numpy 和 C 扩展，不支持
- **scipy** - 依赖 C/Fortran 扩展，不支持
- **tensorflow/pytorch** - 依赖 C++ 扩展，不支持
- **pillow** - 依赖 C 扩展，不支持

**替代方案**：
- 对于数值计算，使用 Rust 模块实现（性能更好）
- 对于图像处理，使用 Rust 的 `image` crate
- 对于数据处理，使用 Pure Python 实现或 Rust 模块

**支持的库类型**：
- ✅ Pure Python 包（无 C 扩展依赖）
- ✅ Rust 模块（直接编译到 QuickPython）
- ❌ C/C++ 扩展包（不支持 CFFI/ctypes）

### 简化的特性

1. **绝对导入** - 仅支持绝对导入，不支持相对导入
2. **__all__** - 支持但不强制

## 示例场景

### 场景 1：使用内置 JSON 模块

```python
import json

# 读取配置文件
config_text = '{"host": "localhost", "port": 8080}'
config = json.loads(config_text)

print("Host:", config["host"])
print("Port:", config["port"])

# 保存配置
new_config = {"host": "0.0.0.0", "port": 9000}
config_text = json.dumps(new_config)
print(config_text)
```

### 场景 2：使用 Pure Python 包

```rust
// 在 Rust 中配置 wheel 文件路径
let mut ctx = Context::new();
ctx.add_python_path("./wheels/mypackage-1.0.0-py3-none-any.whl");
```

```python
# 现在可以导入配置的包
import mypackage

result = mypackage.process_data([1, 2, 3])
print(result)
```

### 场景 3：使用 Rust 扩展

```python
# 使用高性能图像处理扩展
import quickimage

# 加载图像
img = quickimage.load("photo.jpg")

# 应用滤镜（Rust 实现，速度快）
img = quickimage.blur(img, radius=5)
img = quickimage.sharpen(img, amount=1.5)

# 保存结果
quickimage.save(img, "output.jpg")
```

### 场景 4：混合使用

```rust
// 在 Rust 中配置
let mut ctx = Context::new();
ctx.add_python_path("./wheels/mypackage-1.0.0-py3-none-any.whl");  // Pure Python 包
// 内置模块和 Rust 扩展自动可用
```

```python
import json              # 内置模块
import mypackage         # Pure Python 包（需在 Rust 中配置 wheel 文件）
import fastcompute       # Rust 扩展（需在 Rust 中注册）

# 读取数据
data = json.loads(text)

# 使用 Python 包处理
processed = mypackage.process(data)

# 使用 Rust 扩展加速计算
result = fastcompute.compute(processed)

# 输出结果
print(json.dumps(result))
```

## 参考实现

- **Python Import System**: PEP 302, PEP 451
- **Rust PyO3**: Rust-Python 绑定参考
- **Node.js require()**: 模块加载机制参考
