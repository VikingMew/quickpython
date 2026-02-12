# QuickPython API 设计

## Rust 集成 API

### 核心 API 概览

```rust
use quickpython::{Context, Value, Result};

// 创建上下文
let mut ctx = Context::new();

// 执行 Python 代码
ctx.eval("x = 1 + 2")?;

// 获取变量
let x: i64 = ctx.get("x")?.try_into()?;

// 调用 Python 函数
let result = ctx.call("my_function", &[Value::Int(42)])?;

// 注册 Rust 函数
ctx.register_function("rust_add", |a: i64, b: i64| a + b)?;
```

## Context API

### 创建和配置

```rust
pub struct Context {
    // 内部实现
}

impl Context {
    /// 创建新的执行上下文
    pub fn new() -> Self;
    
    /// 创建带配置的上下文
    pub fn with_config(config: ContextConfig) -> Self;
}

pub struct ContextConfig {
    /// 最大堆栈深度
    pub max_stack_depth: usize,
    /// GC 触发阈值
    pub gc_threshold: usize,
    /// 是否启用调试信息
    pub debug: bool,
}
```

### 代码执行

```rust
impl Context {
    /// 执行 Python 代码字符串
    pub fn eval(&mut self, source: &str) -> Result<Value>;
    
    /// 执行 Python 代码文件
    pub fn eval_file(&mut self, path: &str) -> Result<Value>;
    
    /// 编译代码（不执行）
    pub fn compile(&self, source: &str) -> Result<CompiledCode>;
    
    /// 执行已编译的代码
    pub fn run(&mut self, code: &CompiledCode) -> Result<Value>;
}
```

### 变量操作

```rust
impl Context {
    /// 获取全局变量
    pub fn get(&self, name: &str) -> Result<Value>;
    
    /// 设置全局变量
    pub fn set(&mut self, name: &str, value: Value) -> Result<()>;
    
    /// 删除全局变量
    pub fn delete(&mut self, name: &str) -> Result<()>;
    
    /// 检查变量是否存在
    pub fn contains(&self, name: &str) -> bool;
}
```

### 函数调用

```rust
impl Context {
    /// 调用 Python 函数
    pub fn call(&mut self, name: &str, args: &[Value]) -> Result<Value>;
    
    /// 调用方法
    pub fn call_method(
        &mut self, 
        object: &Value, 
        method: &str, 
        args: &[Value]
    ) -> Result<Value>;
}
```

## Value API

### Value 类型定义

```rust
#[derive(Clone, Debug)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Rc<String>),
    List(Rc<RefCell<Vec<Value>>>),
    Dict(Rc<RefCell<HashMap<Value, Value>>>),
    Function(FunctionRef),
    NativeFunction(NativeFn),
    Object(ObjectRef),
}
```

### Value 转换

```rust
impl Value {
    /// 转换为 Rust 类型
    pub fn try_into<T: FromValue>(self) -> Result<T>;
    
    /// 从 Rust 类型创建
    pub fn from<T: IntoValue>(value: T) -> Self;
    
    /// 类型检查
    pub fn is_none(&self) -> bool;
    pub fn is_bool(&self) -> bool;
    pub fn is_int(&self) -> bool;
    pub fn is_float(&self) -> bool;
    pub fn is_string(&self) -> bool;
    pub fn is_list(&self) -> bool;
    pub fn is_dict(&self) -> bool;
    pub fn is_function(&self) -> bool;
    
    /// 获取类型名称
    pub fn type_name(&self) -> &str;
}
```

### Trait 实现

```rust
/// 从 Value 转换为 Rust 类型
pub trait FromValue: Sized {
    fn from_value(value: Value) -> Result<Self>;
}

/// 从 Rust 类型转换为 Value
pub trait IntoValue {
    fn into_value(self) -> Value;
}

// 为基础类型实现 Trait
impl FromValue for i64 { /* ... */ }
impl FromValue for f64 { /* ... */ }
impl FromValue for String { /* ... */ }
impl FromValue for bool { /* ... */ }

impl IntoValue for i64 { /* ... */ }
impl IntoValue for f64 { /* ... */ }
impl IntoValue for String { /* ... */ }
impl IntoValue for bool { /* ... */ }
```

## 函数注册 API

### 注册 Rust 函数

```rust
impl Context {
    /// 注册简单函数
    pub fn register_function<F, Args, Ret>(
        &mut self,
        name: &str,
        func: F
    ) -> Result<()>
    where
        F: Fn(Args) -> Ret + 'static,
        Args: FromValue,
        Ret: IntoValue;
    
    /// 注册带错误处理的函数
    pub fn register_function_with_error<F, Args, Ret>(
        &mut self,
        name: &str,
        func: F
    ) -> Result<()>
    where
        F: Fn(Args) -> Result<Ret> + 'static,
        Args: FromValue,
        Ret: IntoValue;
}
```

### 使用示例

```rust
// 注册简单函数
ctx.register_function("add", |a: i64, b: i64| a + b)?;

// 注册多参数函数
ctx.register_function("greet", |name: String| {
    format!("Hello, {}!", name)
})?;

// 注册带错误处理的函数
ctx.register_function_with_error("divide", |a: f64, b: f64| {
    if b == 0.0 {
        Err(Error::Runtime("Division by zero".into()))
    } else {
        Ok(a / b)
    }
})?;

// 在 Python 中调用
ctx.eval("result = add(1, 2)")?;
ctx.eval("msg = greet('Alice')")?;
ctx.eval("x = divide(10.0, 2.0)")?;
```

## 类型注册 API

### 注册自定义类型

```rust
/// 类型构建器
pub struct TypeBuilder {
    // 内部实现
}

impl Context {
    /// 开始注册类型
    pub fn register_type(&mut self, name: &str) -> TypeBuilder;
}

impl TypeBuilder {
    /// 添加方法
    pub fn method<F, Args, Ret>(
        mut self,
        name: &str,
        func: F
    ) -> Self
    where
        F: Fn(&mut Value, Args) -> Ret + 'static,
        Args: FromValue,
        Ret: IntoValue;
    
    /// 添加属性 getter
    pub fn getter<F, Ret>(
        mut self,
        name: &str,
        func: F
    ) -> Self
    where
        F: Fn(&Value) -> Ret + 'static,
        Ret: IntoValue;
    
    /// 添加属性 setter
    pub fn setter<F, Arg>(
        mut self,
        name: &str,
        func: F
    ) -> Self
    where
        F: Fn(&mut Value, Arg) + 'static,
        Arg: FromValue;
    
    /// 完成注册
    pub fn build(self) -> Result<()>;
}
```

### 使用示例

```rust
// 在 Rust 中定义类型
#[derive(Clone)]
struct Point {
    x: f64,
    y: f64,
}

// 注册到 Python
ctx.register_type("Point")
    .method("distance", |p: &Point| {
        (p.x * p.x + p.y * p.y).sqrt()
    })
    .method("move_to", |p: &mut Point, x: f64, y: f64| {
        p.x = x;
        p.y = y;
    })
    .getter("x", |p: &Point| p.x)
    .getter("y", |p: &Point| p.y)
    .setter("x", |p: &mut Point, x: f64| p.x = x)
    .setter("y", |p: &mut Point, y: f64| p.y = y)
    .build()?;

// 在 Python 中使用
ctx.eval(r#"
p = Point()
p.x = 3.0
p.y = 4.0
dist = p.distance()  # 5.0
"#)?;
```

## 错误处理

```rust
#[derive(Debug)]
pub enum Error {
    /// 语法错误
    Syntax(String),
    /// 运行时错误
    Runtime(String),
    /// 类型错误
    Type(String),
    /// 名称错误
    Name(String),
    /// 自定义错误
    Custom(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Syntax(msg) => write!(f, "SyntaxError: {}", msg),
            Error::Runtime(msg) => write!(f, "RuntimeError: {}", msg),
            Error::Type(msg) => write!(f, "TypeError: {}", msg),
            Error::Name(msg) => write!(f, "NameError: {}", msg),
            Error::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}
```

## 高级 API

### 模块系统

```rust
pub struct Module {
    name: String,
    functions: HashMap<String, NativeFn>,
}

impl Module {
    pub fn new(name: &str) -> Self;
    pub fn add_function<F, Args, Ret>(&mut self, name: &str, func: F);
}

impl Context {
    /// 注册模块
    pub fn register_module(&mut self, module: Module) -> Result<()>;
}
```

### 使用示例

```rust
// 创建模块
let mut math_module = Module::new("math");
math_module.add_function("sqrt", |x: f64| x.sqrt());
math_module.add_function("pow", |x: f64, y: f64| x.powf(y));

// 注册模块
ctx.register_module(math_module)?;

// Python 中使用
ctx.eval("import math; result = math.sqrt(16)")?;
```

## 性能和安全

### 执行限制

```rust
impl Context {
    /// 设置最大执行步数
    pub fn set_max_steps(&mut self, max: usize);
    
    /// 设置超时时间
    pub fn set_timeout(&mut self, duration: std::time::Duration);
    
    /// 中断执行
    pub fn interrupt(&mut self);
}
```

### 沙箱模式

```rust
impl Context {
    /// 创建沙箱上下文（限制危险操作）
    pub fn sandbox() -> Self;
    
    /// 禁用某些功能
    pub fn disable_io(&mut self);
    pub fn disable_imports(&mut self);
}
```
