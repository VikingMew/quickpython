# 模块实现示例

本文档提供三个具体的模块实现示例，展示 QuickPython 支持的三种模块类型。

## 1. JSON 模块（内置模块 - 使用 serde）

JSON 模块是内置模块，使用 Rust 的 serde 库实现高性能的 JSON 解析和序列化。

### 依赖配置

```toml
# Cargo.toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Rust 实现

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

/// 将 JSON 字符串解析为 Python 值
fn json_loads(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "loads() missing required argument: 's' (pos 1)"
        ));
    }
    
    let json_str = args[0].as_string()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "the JSON object must be str, not other type"
        ))?;
    
    // 使用 serde_json 解析
    let json_value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| Value::error(
            ExceptionType::ValueError,
            &format!("Invalid JSON: {}", e)
        ))?;
    
    // 转换为 Python Value
    json_to_value(&json_value)
}

/// 将 Python 值序列化为 JSON 字符串
fn json_dumps(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "dumps() missing required argument: 'obj' (pos 1)"
        ));
    }
    
    // 转换为 serde_json::Value
    let json_value = value_to_json(&args[0])?;
    
    // 序列化为字符串
    let json_str = serde_json::to_string(&json_value)
        .map_err(|e| Value::error(
            ExceptionType::RuntimeError,
            &format!("Failed to serialize: {}", e)
        ))?;
    
    Ok(Value::String(json_str))
}

/// 将 serde_json::Value 转换为 Python Value
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

/// 将 Python Value 转换为 serde_json::Value
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

### 注册模块

```rust
// src/builtins/mod.rs
pub mod json;

pub fn get_builtin_module(name: &str) -> Option<Module> {
    match name {
        "json" => Some(json::create_module()),
        _ => None,
    }
}
```

### Python 使用示例

```python
import json

# 解析 JSON
data = json.loads('{"name": "Alice", "age": 30, "active": true}')
print(data["name"])  # Alice
print(data["age"])   # 30

# 序列化为 JSON
obj = {
    "users": ["Alice", "Bob"],
    "count": 2,
    "enabled": True
}
json_str = json.dumps(obj)
print(json_str)  # {"users":["Alice","Bob"],"count":2,"enabled":true}

# 嵌套结构
nested = {
    "config": {
        "host": "localhost",
        "port": 8080,
        "options": [1, 2, 3]
    }
}
print(json.dumps(nested))
```

### 测试

```rust
// src/main.rs
#[test]
fn test_json_module() {
    let mut ctx = Context::new();
    
    // 测试 loads
    let result = ctx.eval(r#"
import json
data = json.loads('{"x": 1, "y": 2}')
data["x"] + data["y"]
    "#).unwrap();
    assert_eq!(result, Value::Int(3));
    
    // 测试 dumps
    let result = ctx.eval(r#"
import json
obj = {"name": "test", "value": 42}
json.dumps(obj)
    "#).unwrap();
    let json_str = result.as_string().unwrap();
    assert!(json_str.contains("name"));
    assert!(json_str.contains("test"));
    
    // 测试数组
    let result = ctx.eval(r#"
import json
arr = json.loads('[1, 2, 3, 4, 5]')
len(arr)
    "#).unwrap();
    assert_eq!(result, Value::Int(5));
}
```

## 2. tqdm 模块（Pure Python 包）

tqdm 是一个进度条库，使用纯 Python 实现。

### Python 实现

```python
# python_modules/tqdm/__init__.py
"""
tqdm - 简单的进度条库
"""

__version__ = "1.0.0"

from .tqdm import tqdm

__all__ = ["tqdm"]
```

```python
# python_modules/tqdm/tqdm.py
"""
tqdm 进度条实现
"""

import sys

class tqdm:
    """
    简单的进度条
    
    用法:
        for i in tqdm(range(100)):
            # 处理
            pass
    """
    
    def __init__(self, iterable=None, desc=None, total=None):
        self.iterable = iterable
        self.desc = desc
        self.total = total
        self.n = 0
        
        if iterable is not None and total is None:
            try:
                self.total = len(iterable)
            except:
                self.total = None
    
    def __iter__(self):
        if self.iterable is None:
            raise ValueError("iterable is required")
        
        for item in self.iterable:
            yield item
            self.update(1)
        
        self.close()
    
    def update(self, n=1):
        """更新进度"""
        self.n = self.n + n
        self._display()
    
    def _display(self):
        """显示进度条"""
        if self.total is None:
            # 未知总数，只显示计数
            bar = f"{self.desc}: {self.n}it" if self.desc else f"{self.n}it"
        else:
            # 已知总数，显示百分比和进度条
            percent = int(self.n * 100 / self.total)
            filled = int(self.n * 30 / self.total)
            bar_str = "=" * filled + "-" * (30 - filled)
            
            if self.desc:
                bar = f"{self.desc}: {percent}% |{bar_str}| {self.n}/{self.total}"
            else:
                bar = f"{percent}% |{bar_str}| {self.n}/{self.total}"
        
        # 输出进度条（使用 \r 回到行首）
        print(f"\r{bar}", end="")
    
    def close(self):
        """完成进度条"""
        print()  # 换行
    
    def set_description(self, desc):
        """设置描述"""
        self.desc = desc
```

### Rust 配置

```rust
// src/main.rs
use quickpython::Context;

fn main() {
    let mut ctx = Context::new();
    
    // 添加 tqdm wheel 文件
    ctx.add_python_path("./wheels/tqdm-1.0.0-py3-none-any.whl");
    
    // 现在可以使用 tqdm
    ctx.eval(r#"
from tqdm import tqdm
import time

for i in tqdm(range(100), desc="Processing"):
    # 模拟处理
    pass
    "#).unwrap();
}
```

### Python 使用示例

```python
from tqdm import tqdm
import time

# 基本使用
for i in tqdm(range(100)):
    time.sleep(0.01)

# 带描述
for i in tqdm(range(50), desc="Downloading"):
    time.sleep(0.02)

# 手动更新
pbar = tqdm(total=100, desc="Training")
for epoch in range(10):
    for batch in range(10):
        # 训练代码
        pbar.update(1)
pbar.close()

# 处理列表
items = [1, 2, 3, 4, 5]
for item in tqdm(items, desc="Processing items"):
    result = item * 2
```

### 目录结构

```
wheels/
└── tqdm-1.0.0-py3-none-any.whl

# wheel 文件内部结构：
tqdm-1.0.0-py3-none-any.whl
├── tqdm/
│   ├── __init__.py
│   └── tqdm.py
└── tqdm-1.0.0.dist-info/
    └── METADATA
```

## 3. reqwest 模块（Rust 扩展模块）

reqwest 模块展示如何将 Rust 函数注册为 Python 模块，以及如何处理 Python 和 Rust 之间的类型转换。

### 核心概念

1. **注册 Rust 函数**：通过 `module.add_function()` 将 Rust 函数暴露给 Python
2. **类型转换**：处理 Python Value 和 Rust 原生类型之间的转换
3. **错误处理**：将 Rust 错误转换为 Python 异常

### 依赖配置

```toml
# Cargo.toml
[dependencies]
reqwest = { version = "0.11", features = ["blocking"] }
```

### 1. 基础：注册简单函数

```rust
// src/builtins/reqwest.rs
use crate::value::{Value, Module};
use crate::value::ExceptionType;

pub fn create_module() -> Module {
    let mut module = Module::new("reqwest");
    
    // 注册函数：函数名 -> Rust 函数
    module.add_function("get", reqwest_get);
    module.add_function("post", reqwest_post);
    
    module
}

/// 最简单的函数：接收 Vec<Value>，返回 Result<Value, Value>
fn reqwest_get(args: Vec<Value>) -> Result<Value, Value> {
    // 1. 参数验证
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "get() missing required argument: 'url' (pos 1)"
        ));
    }
    
    // 2. 类型转换：Python Value -> Rust String
    let url = args[0].as_string()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "url must be a string"
        ))?;
    
    // 3. 调用 Rust 代码
    let response_text = perform_get_request(&url)?;
    
    // 4. 类型转换：Rust String -> Python Value
    Ok(Value::String(response_text))
}

fn perform_get_request(url: &str) -> Result<String, Value> {
    use reqwest::blocking::Client;
    
    let client = Client::new();
    let response = client.get(url)
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
    
    Ok(text)
}
```

### 2. 类型转换：处理多种参数类型

```rust
/// 处理可选参数和多种类型
fn reqwest_post(args: Vec<Value>) -> Result<Value, Value> {
    // 必需参数：url (string)
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "post() missing required argument: 'url'"
        ));
    }
    
    let url = args[0].as_string()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "url must be a string"
        ))?;
    
    // 可选参数：data (string)
    let data = if args.len() > 1 {
        Some(args[1].as_string()
            .ok_or_else(|| Value::error(
                ExceptionType::TypeError,
                "data must be a string"
            ))?)
    } else {
        None
    };
    
    // 可选参数：headers (dict)
    let headers = if args.len() > 2 {
        Some(extract_headers(&args[2])?)
    } else {
        None
    };
    
    // 可选参数：timeout (int)
    let timeout = if args.len() > 3 {
        Some(args[3].as_int()
            .ok_or_else(|| Value::error(
                ExceptionType::TypeError,
                "timeout must be an integer"
            ))? as u64)
    } else {
        None
    };
    
    // 调用实际的 HTTP 请求
    perform_post_request(&url, data.as_deref(), headers, timeout)
}
```

### 3. 复杂类型转换：Python dict -> Rust HashMap

```rust
use std::collections::HashMap;

/// 从 Python 字典提取 headers
/// 展示如何处理 dict 类型
fn extract_headers(value: &Value) -> Result<HashMap<String, String>, Value> {
    // 1. 验证类型是 dict
    let dict = value.as_dict()
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            "headers must be a dict"
        ))?;
    
    // 2. 遍历字典，转换每个键值对
    let mut headers = HashMap::new();
    for (key, val) in dict.borrow().iter() {
        // 3. 转换 key：DictKey -> String
        let key_str = match key {
            DictKey::String(s) => s.clone(),
            DictKey::Int(i) => i.to_string(),
            DictKey::Bool(b) => b.to_string(),
        };
        
        // 4. 转换 value：Value -> String
        let val_str = val.as_string()
            .ok_or_else(|| Value::error(
                ExceptionType::TypeError,
                &format!("header value for '{}' must be a string", key_str)
            ))?;
        
        headers.insert(key_str, val_str);
    }
    
    Ok(headers)
}
```

### 4. 复杂类型转换：Rust struct -> Python dict

```rust
use std::rc::Rc;
use std::cell::RefCell;
use crate::value::DictKey;

/// HTTP 响应结构
struct HttpResponse {
    status_code: u16,
    headers: HashMap<String, String>,
    body: String,
}

/// 将 Rust 结构体转换为 Python 字典
fn response_to_dict(response: HttpResponse) -> Value {
    let mut dict = HashMap::new();
    
    // 1. 基本类型转换
    dict.insert(
        DictKey::String("status_code".to_string()),
        Value::Int(response.status_code as i32)
    );
    
    dict.insert(
        DictKey::String("body".to_string()),
        Value::String(response.body)
    );
    
    // 2. 计算属性
    dict.insert(
        DictKey::String("ok".to_string()),
        Value::Bool(response.status_code >= 200 && response.status_code < 300)
    );
    
    // 3. 嵌套字典：headers
    let mut headers_dict = HashMap::new();
    for (key, value) in response.headers {
        headers_dict.insert(
            DictKey::String(key),
            Value::String(value)
        );
    }
    dict.insert(
        DictKey::String("headers".to_string()),
        Value::Dict(Rc::new(RefCell::new(headers_dict)))
    );
    
    // 4. 返回字典
    Value::Dict(Rc::new(RefCell::new(dict)))
}

/// 完整的 POST 请求实现
fn perform_post_request(
    url: &str,
    data: Option<&str>,
    headers: Option<HashMap<String, String>>,
    timeout: Option<u64>,
) -> Result<Value, Value> {
    use reqwest::blocking::Client;
    use std::time::Duration;
    
    // 构建客户端
    let mut client_builder = Client::builder();
    if let Some(t) = timeout {
        client_builder = client_builder.timeout(Duration::from_secs(t));
    }
    
    let client = client_builder.build()
        .map_err(|e| Value::error(
            ExceptionType::RuntimeError,
            &format!("Failed to create client: {}", e)
        ))?;
    
    // 构建请求
    let mut request = client.post(url);
    
    // 添加 headers
    if let Some(hdrs) = headers {
        for (key, value) in hdrs {
            request = request.header(key, value);
        }
    }
    
    // 添加 body
    if let Some(body) = data {
        request = request.body(body.to_string());
    }
    
    // 发送请求
    let response = request.send()
        .map_err(|e| Value::error(
            ExceptionType::RuntimeError,
            &format!("Request failed: {}", e)
        ))?;
    
    // 提取响应信息
    let status_code = response.status().as_u16();
    let response_headers: HashMap<String, String> = response.headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let body = response.text()
        .map_err(|e| Value::error(
            ExceptionType::RuntimeError,
            &format!("Failed to read response: {}", e)
        ))?;
    
    // 构建响应对象
    let http_response = HttpResponse {
        status_code,
        headers: response_headers,
        body,
    };
    
    // 转换为 Python 字典
    Ok(response_to_dict(http_response))
}
```

### 5. 类型转换辅助函数

```rust
/// 辅助函数：检查参数数量
fn check_arg_count(args: &[Value], expected: usize, func_name: &str) -> Result<(), Value> {
    if args.len() != expected {
        return Err(Value::error(
            ExceptionType::TypeError,
            &format!("{}() takes exactly {} argument(s), got {}", 
                     func_name, expected, args.len())
        ));
    }
    Ok(())
}

/// 辅助函数：提取字符串参数
fn get_string_arg(args: &[Value], index: usize, name: &str) -> Result<String, Value> {
    args.get(index)
        .and_then(|v| v.as_string())
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            &format!("argument '{}' must be a string", name)
        ))
}

/// 辅助函数：提取整数参数
fn get_int_arg(args: &[Value], index: usize, name: &str) -> Result<i32, Value> {
    args.get(index)
        .and_then(|v| v.as_int())
        .ok_or_else(|| Value::error(
            ExceptionType::TypeError,
            &format!("argument '{}' must be an integer", name)
        ))
}

/// 辅助函数：提取可选字典参数
fn get_optional_dict(args: &[Value], index: usize) -> Option<Rc<RefCell<HashMap<DictKey, Value>>>> {
    args.get(index).and_then(|v| v.as_dict())
}

/// 使用辅助函数简化代码
fn reqwest_get_simplified(args: Vec<Value>) -> Result<Value, Value> {
    check_arg_count(&args, 1, "get")?;
    let url = get_string_arg(&args, 0, "url")?;
    
    perform_get_request(&url)
}
```

### 注册模块

```rust
// src/builtins/mod.rs
pub mod json;
pub mod reqwest;

pub fn get_builtin_module(name: &str) -> Option<Module> {
    match name {
        "json" => Some(json::create_module()),
        "reqwest" => Some(reqwest::create_module()),
        _ => None,
    }
}
```

### Python 使用示例

```python
import reqwest

# 简单 GET 请求
response_text = reqwest.get("https://httpbin.org/get")
print(response_text)

# POST 请求（带数据和 headers）
import json

data = json.dumps({"name": "Alice", "age": 30})
headers = {"Content-Type": "application/json"}
response = reqwest.post("https://httpbin.org/post", data, headers)

# 响应是字典
print(response["status_code"])  # 200
print(response["ok"])            # True
print(response["body"])          # 响应体
print(response["headers"])       # 响应头字典

# 带超时的请求
response = reqwest.post("https://httpbin.org/delay/2", None, None, 5)  # 5 秒超时
```

### 测试

```rust
// src/main.rs
#[test]
fn test_reqwest_module() {
    let mut ctx = Context::new();
    
    // 测试简单 GET 请求
    let result = ctx.eval(r#"
import reqwest
response = reqwest.get("https://httpbin.org/get")
type(response)
    "#).unwrap();
    // 返回字符串
    assert!(matches!(result, Value::String(_)));
    
    // 测试 POST 请求返回字典
    let result = ctx.eval(r#"
import reqwest
response = reqwest.post("https://httpbin.org/post", "test data")
response["status_code"]
    "#).unwrap();
    assert_eq!(result, Value::Int(200));
}
```

### 关键要点总结

1. **函数签名固定**：所有 Rust 函数必须是 `fn(Vec<Value>) -> Result<Value, Value>`
2. **类型转换模式**：
   - Python → Rust：使用 `.as_xxx()` 方法 + `ok_or_else` 错误处理
   - Rust → Python：构造 `Value::Xxx()` 枚举
3. **错误处理**：使用 `Value::error(ExceptionType, message)` 创建 Python 异常
4. **复杂类型**：
   - dict：使用 `HashMap<DictKey, Value>` + `Rc<RefCell<>>`
   - list：使用 `Vec<Value>` + `Rc<RefCell<>>`
5. **辅助函数**：创建类型转换辅助函数简化代码

## 综合使用示例

```python
import json
import reqwest
from tqdm import tqdm

# 场景：批量下载数据并显示进度

urls = [
    "https://api.example.com/data/1",
    "https://api.example.com/data/2",
    "https://api.example.com/data/3",
    "https://api.example.com/data/4",
    "https://api.example.com/data/5",
]

results = []

for url in tqdm(urls, desc="Downloading"):
    # 使用 reqwest（Rust 扩展）发送请求
    response = reqwest.get(url)
    
    if response["ok"]:
        # 使用 json（内置模块）解析响应
        data = json.loads(response["text"])
        results.append(data)
    else:
        print(f"Failed to download {url}")

# 保存结果
output = json.dumps(results)
print(f"Downloaded {len(results)} items")
```

## 模块对比总结

| 特性 | json (内置) | tqdm (Pure Python) | reqwest (Rust 扩展) |
|------|------------|-------------------|---------------------|
| 实现语言 | Rust | Python | Rust |
| 性能 | 高（serde） | 中等 | 高（reqwest） |
| 注册方式 | `get_builtin_module()` | `ctx.add_python_path()` | `get_builtin_module()` |
| 依赖 | serde, serde_json | 无 | reqwest, tokio |
| 可移植性 | 编译到二进制 | 需要 .py 文件 | 编译到二进制 |
| 开发难度 | 中等 | 简单 | 中等 |
| 适用场景 | 核心功能 | 辅助工具 | 性能关键功能 |
