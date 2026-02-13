# Task 017: 实现 import json 支持

## 状态
- **状态**: DONE
- **优先级**: P0
- **预计工作量**: 中等
- **完成日期**: 2026-02-12

## 目标

实现最基础的模块导入系统，支持 `import json` 和 `from json import loads, dumps`。

## 范围

### 包含
1. 基础的 import 语句编译和执行
2. json 内置模块实现（使用 serde_json）
3. 模块缓存机制
4. 基本的 from...import 支持

### 不包含
- 包路径导入（`from package.subpackage import module`）
- 相对导入
- Python wheel 文件支持
- 扩展模块注册 API

## 实现步骤

### 1. 添加依赖

```toml
# Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 2. 扩展 Value 枚举

```rust
// src/value.rs
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

pub enum Value {
    // ... 现有类型
    Module(Rc<RefCell<Module>>),
    NativeFunction(NativeFunction),
}

pub type NativeFunction = fn(Vec<Value>) -> Result<Value, Value>;

pub struct Module {
    pub name: String,
    pub attributes: HashMap<String, Value>,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Module {
            name: name.to_string(),
            attributes: HashMap::new(),
        }
    }
    
    pub fn add_function(&mut self, name: &str, func: NativeFunction) {
        self.attributes.insert(
            name.to_string(),
            Value::NativeFunction(func)
        );
    }
    
    pub fn get_attribute(&self, name: &str) -> Option<Value> {
        self.attributes.get(name).cloned()
    }
}
```

### 3. 添加字节码指令

```rust
// src/bytecode.rs
pub enum Instruction {
    // ... 现有指令
    
    // 导入指令
    Import(String),                    // import module
    ImportFrom(String, Vec<String>),   // from module import names
    GetAttr(String),                   // 获取模块属性
}
```

### 4. 实现 json 模块

```rust
// src/builtins/mod.rs
pub mod json;

const BUILTIN_MODULES: &[&str] = &["json"];

pub fn is_builtin_module(name: &str) -> bool {
    BUILTIN_MODULES.contains(&name)
}

pub fn get_builtin_module(name: &str) -> Module {
    match name {
        "json" => json::create_module(),
        _ => panic!("Unknown builtin module: {}", name),
    }
}
```

```rust
// src/builtins/json.rs
use crate::value::{Value, Module, DictKey};
use crate::value::ExceptionType;
use serde_json;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

pub fn create_module() -> Module {
    let mut module = Module::new("json");
    module.add_function("loads", json_loads);
    module.add_function("dumps", json_dumps);
    module
}

fn json_loads(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "loads() missing required argument: 's'"
        ));
    }
    
    let json_str = args[0].as_string()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "argument must be a string"
        ))?;
    
    let json_value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| Value::error(
            ExceptionType::ValueError,
            &format!("Invalid JSON: {}", e)
        ))?;
    
    json_to_value(&json_value)
}

fn json_dumps(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "dumps() missing required argument: 'obj'"
        ));
    }
    
    let json_value = value_to_json(&args[0])?;
    let json_str = serde_json::to_string(&json_value)
        .map_err(|e| Value::error(
            ExceptionType::RuntimeError,
            &format!("Failed to serialize: {}", e)
        ))?;
    
    Ok(Value::String(json_str))
}

fn json_to_value(json: &serde_json::Value) -> Result<Value, Value> {
    match json {
        serde_json::Value::Null => Ok(Value::None),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i as i32))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(Value::error(
                    ExceptionType::ValueError,
                    "Number out of range"
                ))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let mut items = Vec::new();
            for item in arr {
                items.push(json_to_value(item)?);
            }
            Ok(Value::new_list(items))
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (key, val) in obj {
                let py_val = json_to_value(val)?;
                map.insert(DictKey::String(key.clone()), py_val);
            }
            Ok(Value::Dict(Rc::new(RefCell::new(map))))
        }
    }
}

fn value_to_json(value: &Value) -> Result<serde_json::Value, Value> {
    match value {
        Value::None => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Int(i) => Ok(serde_json::Value::Number((*i).into())),
        Value::Float(f) => {
            serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .ok_or_else(|| Value::error(
                    ExceptionType::ValueError,
                    "Float value is not finite"
                ))
        }
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::List(list) => {
            let mut arr = Vec::new();
            for item in &list.borrow().items {
                arr.push(value_to_json(item)?);
            }
            Ok(serde_json::Value::Array(arr))
        }
        Value::Dict(dict) => {
            let mut obj = serde_json::Map::new();
            for (key, val) in dict.borrow().iter() {
                let key_str = match key {
                    DictKey::String(s) => s.clone(),
                    DictKey::Int(i) => i.to_string(),
                    DictKey::Bool(b) => b.to_string(),
                };
                obj.insert(key_str, value_to_json(val)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        _ => Err(Value::error(
            ExceptionType::TypeError,
            &format!("Object of type '{}' is not JSON serializable", value.type_name())
        ))
    }
}
```

### 5. 编译器支持

```rust
// src/compiler.rs
use rustpython_parser::ast;

impl Compiler {
    fn compile_import(&mut self, import: &ast::StmtImport) -> Result<(), String> {
        for alias in &import.names {
            let module_name = alias.name.to_string();
            let as_name = alias.asname.as_ref()
                .map(|n| n.to_string())
                .unwrap_or_else(|| module_name.clone());
            
            // import module
            self.bytecode.push(Instruction::Import(module_name));
            // 绑定到变量
            self.bytecode.push(Instruction::SetGlobal(as_name));
        }
        Ok(())
    }
    
    fn compile_import_from(&mut self, import_from: &ast::StmtImportFrom) -> Result<(), String> {
        let module_name = import_from.module.as_ref()
            .ok_or("from import without module name")?
            .to_string();
        
        // import module
        self.bytecode.push(Instruction::Import(module_name.clone()));
        
        // 对每个导入的名称
        for alias in &import_from.names {
            let name = alias.name.to_string();
            let as_name = alias.asname.as_ref()
                .map(|n| n.to_string())
                .unwrap_or_else(|| name.clone());
            
            // 复制模块到栈顶
            self.bytecode.push(Instruction::LoadGlobal(module_name.clone()));
            // 获取属性
            self.bytecode.push(Instruction::GetAttr(name));
            // 绑定到变量
            self.bytecode.push(Instruction::SetGlobal(as_name));
        }
        
        // 弹出模块
        self.bytecode.push(Instruction::Pop);
        
        Ok(())
    }
    
    fn compile_stmt(&mut self, stmt: &ast::Stmt) -> Result<(), String> {
        match stmt {
            ast::Stmt::Import(import) => self.compile_import(import),
            ast::Stmt::ImportFrom(import_from) => self.compile_import_from(import_from),
            // ... 其他语句
        }
    }
}
```

### 6. VM 执行

```rust
// src/vm.rs
use std::collections::HashMap;

pub struct VM {
    // ... 现有字段
    loaded_modules: HashMap<String, Rc<RefCell<Module>>>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            // ... 现有初始化
            loaded_modules: HashMap::new(),
        }
    }
    
    fn load_module(&mut self, name: &str) -> Result<Rc<RefCell<Module>>, Value> {
        // 1. 检查缓存
        if let Some(module) = self.loaded_modules.get(name) {
            return Ok(module.clone());
        }
        
        // 2. 加载内置模块
        if builtins::is_builtin_module(name) {
            let module = builtins::get_builtin_module(name);
            let module_rc = Rc::new(RefCell::new(module));
            self.loaded_modules.insert(name.to_string(), module_rc.clone());
            return Ok(module_rc);
        }
        
        // 3. 未找到
        Err(Value::error(
            ExceptionType::ImportError,
            &format!("No module named '{}'", name)
        ))
    }
    
    pub fn execute(&mut self) -> Result<Value, Value> {
        let mut ip = 0;
        
        while ip < self.bytecode.len() {
            match &self.bytecode[ip] {
                Instruction::Import(module_name) => {
                    let module = self.load_module(module_name)?;
                    self.stack.push(Value::Module(module));
                    ip += 1;
                }
                
                Instruction::GetAttr(name) => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Value::error(
                            ExceptionType::RuntimeError,
                            "Stack underflow"
                        ))?;
                    
                    match value {
                        Value::Module(module) => {
                            let attr = module.borrow().get_attribute(name)
                                .ok_or_else(|| Value::error(
                                    ExceptionType::AttributeError,
                                    &format!("module has no attribute '{}'", name)
                                ))?;
                            self.stack.push(attr);
                        }
                        _ => {
                            return Err(Value::error(
                                ExceptionType::TypeError,
                                "getattr on non-module"
                            ));
                        }
                    }
                    ip += 1;
                }
                
                // ... 其他指令
            }
        }
        
        // ...
    }
}
```

### 7. 调用 NativeFunction

```rust
// src/vm.rs
impl VM {
    fn call_function(&mut self, func: &Value, args: Vec<Value>) -> Result<Value, Value> {
        match func {
            Value::NativeFunction(native_fn) => {
                // 直接调用 Rust 函数
                native_fn(args)
            }
            Value::Function(py_fn) => {
                // 调用 Python 函数（现有逻辑）
                // ...
            }
            _ => Err(Value::error(
                ExceptionType::TypeError,
                "object is not callable"
            ))
        }
    }
}
```

## 测试用例

```rust
// src/main.rs

#[test]
fn test_import_json() {
    let mut ctx = Context::new();
    
    // 测试 import json
    ctx.eval("import json").unwrap();
    
    // 测试 json.loads
    let result = ctx.eval(r#"
import json
data = json.loads('{"x": 1, "y": 2}')
data["x"]
    "#).unwrap();
    assert_eq!(result, Value::Int(1));
}

#[test]
fn test_json_loads() {
    let mut ctx = Context::new();
    
    let result = ctx.eval(r#"
import json
data = json.loads('{"name": "Alice", "age": 30}')
data["name"]
    "#).unwrap();
    assert_eq!(result, Value::String("Alice".to_string()));
}

#[test]
fn test_json_dumps() {
    let mut ctx = Context::new();
    
    let result = ctx.eval(r#"
import json
obj = {"x": 1, "y": 2}
json.dumps(obj)
    "#).unwrap();
    
    let json_str = result.as_string().unwrap();
    assert!(json_str.contains("\"x\""));
    assert!(json_str.contains("1"));
}

#[test]
fn test_from_import() {
    let mut ctx = Context::new();
    
    let result = ctx.eval(r#"
from json import loads
data = loads('{"value": 42}')
data["value"]
    "#).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_import_as() {
    let mut ctx = Context::new();
    
    let result = ctx.eval(r#"
import json as j
data = j.loads('{"test": true}')
data["test"]
    "#).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_json_array() {
    let mut ctx = Context::new();
    
    let result = ctx.eval(r#"
import json
arr = json.loads('[1, 2, 3, 4, 5]')
len(arr)
    "#).unwrap();
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_json_nested() {
    let mut ctx = Context::new();
    
    let result = ctx.eval(r#"
import json
data = json.loads('{"user": {"name": "Bob", "id": 123}}')
data["user"]["name"]
    "#).unwrap();
    assert_eq!(result, Value::String("Bob".to_string()));
}

#[test]
fn test_module_not_found() {
    let mut ctx = Context::new();
    
    let result = ctx.eval("import nonexistent");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("No module named"));
}
```

## 示例代码

```python
# 基本使用
import json

data = json.loads('{"name": "Alice", "age": 30}')
print(data["name"])  # Alice

obj = {"x": 1, "y": 2}
text = json.dumps(obj)
print(text)  # {"x":1,"y":2}

# from import
from json import loads, dumps

data = loads('{"value": 42}')
print(data["value"])  # 42

# import as
import json as j
data = j.loads('{"test": true}')
```

## 验收标准

- [x] 可以 `import json`
- [x] 可以调用 `json.loads()` 解析 JSON 字符串
- [x] 可以调用 `json.dumps()` 序列化为 JSON
- [x] 支持 `from json import loads, dumps`
- [x] 支持 `import json as j`
- [x] 模块缓存正常工作（重复 import 返回同一个模块）
- [x] 所有测试通过
- [x] 错误处理正确（模块不存在、JSON 格式错误等）

## 注意事项

1. **模块缓存**：确保同一个模块只创建一次，使用 `Rc<RefCell<Module>>` 共享
2. **错误处理**：JSON 解析错误要转换为 Python 异常
3. **类型转换**：注意 JSON 数字可能是整数或浮点数
4. **内存管理**：Module 使用 Rc 包装，避免所有权问题
5. **NativeFunction 调用**：确保参数传递正确

## 后续任务

- Task 018: 实现 math 模块
- Task 019: 实现扩展模块注册 API
- Task 020: 实现 Python wheel 文件加载
