# QuickPython 架构设计

## 核心优化技术（参考 QuickJS）

QuickPython 借鉴 QuickJS 的成熟设计，采用以下关键优化：

### 1. NaN Boxing / Tagged Pointers
- **原理**: 在 64 位值中直接编码小整数、布尔、None，无需堆分配
- **收益**: 
  - 基础类型操作零堆分配
  - Value 只占 8 字节，高效传递
  - 类型检查仅需位运算
- **对比**: Python 中 `1` 也是对象需要堆分配，这是性能瓶颈之一

### 2. 原子化字符串（String Interning）
- **原理**: 相同字符串在内存中只存一份，通过 ID 引用
- **收益**:
  - 内存占用大幅减少
  - 字符串比较从 O(n) 降为 O(1)
  - 变量查找更快（整数 key）
- **对比**: 避免 Python 中重复字符串占用大量内存

### 3. 引用计数（确定性垃圾回收）
- **原理**: 使用引用计数实时回收，简单的循环检测器处理循环引用
- **收益**:
  - 确定性析构，对象立即释放
  - 无 GC 暂停，无 stop-the-world
  - 内存使用可预测
  - 实现简单，易于维护
- **对比**: 避免复杂的分代 GC 和 Python GIL 瓶颈

### 4. 紧凑字节码
- **原理**: 变长指令编码，常用指令 1-2 字节
- **收益**:
  - 字节码体积减少 30-50%
  - 更好的缓存局部性
  - 支持预编译和 AOT
- **对比**: Python .pyc 相对臃肿

### 5. 预编译与字节码缓存
- **原理**: 源码编译为字节码，可序列化保存和复用
- **收益**:
  - 启动时间微秒级
  - 编译一次多次执行
  - 保护源码知识产权
- **对比**: 类似 QuickJS 的 `qjsc` 工具

### 6. 原生异步支持
- **原理**: 异步功能直接集成到 VM 和运行时
- **收益**:
  - 无需复杂的事件循环库
  - 与 Rust 的 async/await 无缝集成
  - 高性能协程调度
- **特点**: 不依赖外部库，原生支持

**性能目标**: 
- 启动速度: < 1ms
- 内存占用: < 1MB (基础运行时)
- 执行性能: 达到 QuickJS 的 70-90%

**设计原则**:
- ✅ 基于栈的虚拟机
- ✅ Tagged Pointers 值类型（借鉴 QuickJS NaN Boxing）
- ✅ 简单的引用计数 GC（借鉴 QuickJS）
- ✅ 确定性垃圾回收
- ✅ 原生异步集成（Rust async/await，不是 Promise）
- ✅ Slot 和静态查找（不使用 `__dict__`）
- ✅ Rust 函数绑定（不是 C API）
- ❌ 不支持元编程
- ❌ 不提供完整标准库
- ❌ 不兼容 Python C API
- ❌ 不兼容 C 函数绑定（只支持 Rust）

## 整体架构

```
┌─────────────────────────────────────────┐
│         Rust Application                │
├─────────────────────────────────────────┤
│         QuickPython API                 │
│  (Context, Value, Function Binding)     │
├─────────────────────────────────────────┤
│            Runtime Layer                │
│  ┌──────────┬──────────┬────────────┐  │
│  │   VM     │  RefCnt  │  Builtin   │  │
│  │ Executor │  Values  │  Functions │  │
│  └──────────┴──────────┴────────────┘  │
├─────────────────────────────────────────┤
│          Compiler Layer                 │
│  ┌──────────┬──────────┬────────────┐  │
│  │  Parser  │   AST    │  Bytecode  │  │
│  │          │          │  Generator │  │
│  └──────────┴──────────┴────────────┘  │
└─────────────────────────────────────────┘
```

## 模块划分

### 1. Frontend (编译前端)
- **Parser** - 使用 `rustpython_parser` 解析 Python 代码
- **AST** - 复用 RustPython 的 AST 定义
- **注**: 不自己实现 Lexer 和 Parser，直接使用成熟的 RustPython 解析器

### 2. Compiler (编译器)
- **Bytecode Generator** - 将 AST 编译为字节码
- **Optimizer** - 字节码优化器（可选）
- **Constant Pool** - 常量池管理

### 3. Runtime (运行时)
- **VM** - 基于栈的虚拟机执行器
- **Value System** - Tagged Pointers 值类型系统
- **String Pool** - 原子化字符串池
- **RefCount GC** - 简单的引用计数垃圾回收
- **Async Runtime** - 原生异步运行时
- **Builtin Functions** - 精简内置函数库（不包含完整标准库）

### 4. API Layer (接口层)
- **Context** - 执行上下文管理
- **Value Binding** - Rust ↔ Python 值转换
- **Function Registration** - 函数注册和调用
- **Error Handling** - 错误处理机制

## 核心数据结构

### Value 类型 - NaN Boxing / Tagged Pointers

参考 QuickJS 的 NaN Boxing 技术，在 64 位指针中直接编码小值：

```rust
/// 值表示 - 使用 NaN Boxing 优化
/// 
/// 64 位布局：
/// - NaN 浮点数: 0x7FF8_0000_0000_0000 - 0x7FFF_FFFF_FFFF_FFFF
/// - 小整数:     0xFFF0_0000_0000_0000 | (i32 as u64)
/// - Bool True:  0xFFF1_0000_0000_0001
/// - Bool False: 0xFFF1_0000_0000_0000
/// - None:       0xFFF2_0000_0000_0000
/// - 指针:       其他值（低 48 位存指针）
#[repr(transparent)]
pub struct Value(u64);

impl Value {
    // 类型标签定义
    const TAG_FLOAT:   u64 = 0x7FF8_0000_0000_0000;
    const TAG_INT:     u64 = 0xFFF0_0000_0000_0000;
    const TAG_BOOL:    u64 = 0xFFF1_0000_0000_0000;
    const TAG_NONE:    u64 = 0xFFF2_0000_0000_0000;
    const TAG_PTR:     u64 = 0x0000_0000_0000_0000;
    
    /// 创建小整数（-2^31 到 2^31-1）- 无堆分配
    pub fn int(n: i32) -> Self {
        Value(Self::TAG_INT | (n as u32 as u64))
    }
    
    /// 创建浮点数 - 无堆分配（利用 NaN 空间）
    pub fn float(f: f64) -> Self {
        if f.is_nan() {
            Value(Self::TAG_FLOAT)  // 标准化 NaN
        } else {
            Value(f.to_bits())
        }
    }
    
    /// 创建布尔值 - 无堆分配
    pub fn bool(b: bool) -> Self {
        Value(Self::TAG_BOOL | (b as u64))
    }
    
    /// 创建 None - 无堆分配
    pub fn none() -> Self {
        Value(Self::TAG_NONE)
    }
    
    /// 检查是否是小整数
    pub fn is_int(&self) -> bool {
        self.0 & 0xFFFF_0000_0000_0000 == Self::TAG_INT
    }
    
    /// 提取整数值
    pub fn as_int(&self) -> Option<i32> {
        if self.is_int() {
            Some(self.0 as i32)
        } else {
            None
        }
    }
    
    // ... 其他类型的方法
}

/// 堆分配对象 - 用于大整数、字符串、列表等
struct RefCounted {
    ref_count: u32,      // 引用计数
    kind: u8,            // 对象类型
    // ... 实际数据
}

pub enum HeapObjectKind {
    BigInt,              // 大整数（超过 i32 范围）
    String,              // 原子化字符串
    List,
    Dict,
    Function,
    Object,              // 对象（使用 slot，不使用 __dict__）
}

/// 对象结构 - 使用固定 slot 而非动态 __dict__
pub struct Object {
    ref_count: u32,
    class_id: u32,       // 类 ID
    slots: Vec<Value>,   // 固定 slot 数组（编译时确定）
}

/// 类定义 - 编译时确定所有属性位置
pub struct Class {
    name: AtomId,
    slot_count: u32,                    // slot 数量
    slot_names: Vec<AtomId>,            // slot 名称（编译时确定）
    methods: HashMap<AtomId, FunctionRef>, // 方法表
}
```

**Slot 查找优势**：
- ✅ 属性访问 O(1)，直接索引数组
- ✅ 编译时确定所有属性位置
- ✅ 内存布局紧凑，缓存友好
- ✅ 无需动态 `__dict__` 查找
- ❌ 不支持动态添加属性（这是设计取舍）

### 字节码指令集 - 紧凑编码

参考 QuickJS，使用变长编码节省空间：

```rust
/// 字节码采用紧凑编码
/// - 常用指令 1 字节
/// - 带小参数指令 2 字节
/// - 带大参数指令 3-5 字节
pub enum OpCode {
    // === 零参数指令 (1 字节) ===
    Add,                 // 0x01: 加法
    Sub,                 // 0x02: 减法
    Mul,                 // 0x03: 乘法
    Div,                 // 0x04: 除法
    Mod,                 // 0x05: 取模
    
    Eq,                  // 0x10: 相等
    Ne,                  // 0x11: 不等
    Lt,                  // 0x12: 小于
    Le,                  // 0x13: 小于等于
    Gt,                  // 0x14: 大于
    Ge,                  // 0x15: 大于等于
    
    Return,              // 0x20: 返回
    Pop,                 // 0x21: 弹出栈顶
    
    // === 异步指令 ===
    Await,               // 0x22: 等待异步操作
    Yield,               // 0x23: 协程让出
    
    // === 带小参数指令 (2 字节) ===
    LoadConst8(u8),      // 0x30: 加载常量（索引 < 256）
    LoadName8(u8),       // 0x31: 加载变量（索引 < 256）
    StoreName8(u8),      // 0x32: 存储变量（索引 < 256）
    Call8(u8),           // 0x33: 调用函数（参数 < 256）
    
    // === 带大参数指令 (5 字节) ===
    LoadConst32(u32),    // 0x40: 加载常量（大索引）
    LoadName32(u32),     // 0x41: 加载变量（大索引）
    StoreName32(u32),    // 0x42: 存储变量（大索引）
    
    // === 控制流 (变长) ===
    JumpShort(i8),       // 0x50: 短跳转 (-128 ~ 127)
    Jump(i32),           // 0x51: 跳转
    JumpIfFalseShort(i8),// 0x52: 条件短跳转
    JumpIfFalse(i32),    // 0x53: 条件跳转
    
    // === 复合操作 ===
    BuildList(u16),      // 0x60: 构建列表
    BuildDict(u16),      // 0x61: 构建字典
}

/// 字节码生成器智能选择最短指令
impl Compiler {
    fn emit_load_const(&mut self, index: u32) {
        if index < 256 {
            self.emit(OpCode::LoadConst8(index as u8));  // 2 字节
        } else {
            self.emit(OpCode::LoadConst32(index));       // 5 字节
        }
    }
    
    fn emit_jump(&mut self, offset: i32) {
        if offset >= -128 && offset <= 127 {
            self.emit(OpCode::JumpShort(offset as i8)); // 2 字节
        } else {
            self.emit(OpCode::Jump(offset));            // 5 字节
        }
    }
}
```

**紧凑字节码优势**：
- ✅ 常见指令只占 1-2 字节（vs Python 的 2-6 字节）
- ✅ 字节码体积减少 30-50%
- ✅ 更好的缓存局部性，执行更快
- ✅ 网络传输和磁盘 I/O 更快
- ✅ 原生支持异步指令（Await, Yield）

### 执行上下文
```rust
pub struct Context {
    globals: HashMap<String, Value>,
    call_stack: Vec<Frame>,
    
    // 字符串池 - 所有字符串都原子化
    string_pool: StringPool,
}

/// 原子化字符串池
/// 
/// 参考 QuickJS，所有字符串在内存中只存一份
pub struct StringPool {
    // 字符串到 ID 的映射
    strings: HashMap<String, AtomId>,
    // ID 到字符串的映射（用于反查）
    atoms: Vec<Rc<String>>,
}

pub struct AtomId(u32);

impl StringPool {
    /// 原子化字符串 - 如果已存在则返回现有的
    pub fn intern(&mut self, s: &str) -> AtomId {
        if let Some(&id) = self.strings.get(s) {
            id  // 复用现有字符串
        } else {
            let id = AtomId(self.atoms.len() as u32);
            let rc_str = Rc::new(s.to_string());
            self.atoms.push(rc_str.clone());
            self.strings.insert(s.to_string(), id);
            id
        }
    }
    
    /// 通过 ID 获取字符串
    pub fn get(&self, id: AtomId) -> &str {
        &self.atoms[id.0 as usize]
    }
    
    /// 比较两个原子字符串只需比较 ID
    pub fn equals(&self, a: AtomId, b: AtomId) -> bool {
        a.0 == b.0  // O(1) 比较！
    }
}
```

**原子字符串优势**：
- ✅ 相同字符串只存一份，大幅节省内存
- ✅ 字符串比较从 O(n) 降为 O(1)（比较 ID）
- ✅ 变量名查找更快（HashMap key 是整数）
- ✅ 字典 key 为字符串时性能优异

pub struct Frame {
    locals: HashMap<String, Value>,
    stack: Vec<Value>,
    ip: usize,          // 指令指针
    code: Rc<ByteCode>,
}
```

## 内存管理策略

采用简单的引用计数机制，确保确定性垃圾回收：

### 引用计数机制
```rust
impl Value {
    // 增加引用计数
    pub fn dup(&self) -> Self {
        if let Some(obj) = self.as_object() {
            obj.inc_ref();
        }
        self.clone()
    }
    
    // 减少引用计数，为 0 时立即释放
    pub fn free(&mut self) {
        if let Some(obj) = self.as_object_mut() {
            obj.dec_ref();
            if obj.ref_count() == 0 {
                obj.destroy();  // 递归释放子对象
            }
        }
    }
}
```

### 循环引用处理（简单策略）
- **弱引用**: 某些场景使用弱引用打破循环
- **手动管理**: 开发者可以手动打破循环引用
- **可选检测**: 提供简单的循环检测工具（非自动）

### 设计原则
- ✅ **简单实现**: 避免复杂的 GC 算法
- ✅ **确定性**: 对象在引用计数归零时立即释放
- ✅ **无暂停**: 没有 stop-the-world 的 GC 停顿
- ✅ **可预测**: 内存使用量精确可控
- ❌ **不支持**: 不实现分代 GC、增量 GC 等复杂机制

## 字节码设计

参考 QuickJS 的字节码系统，支持预编译和缓存：

### 字节码文件格式
```rust
pub struct BytecodeModule {
    magic: u32,              // 魔数标识
    version: u32,            // 版本号
    flags: u32,              // 编译标志
    
    // 常量池
    constants: Vec<Value>,
    
    // 字符串表（用于变量名等）
    strings: Vec<String>,
    
    // 函数表
    functions: Vec<FunctionBytecode>,
    
    // 调试信息（可选）
    debug_info: Option<DebugInfo>,
}

pub struct FunctionBytecode {
    name: u32,               // 函数名（索引到字符串表）
    arg_count: u8,           // 参数个数
    local_count: u16,        // 局部变量个数
    stack_size: u16,         // 所需栈大小
    code: Vec<u8>,           // 字节码指令
    line_info: Vec<u16>,     // 行号映射（调试用）
}
```

### 预编译 API
```rust
impl Context {
    /// 编译源码为字节码（不执行）
    pub fn compile(&self, source: &str) -> Result<BytecodeModule>;
    
    /// 将字节码序列化为二进制
    pub fn serialize_bytecode(&self, module: &BytecodeModule) -> Vec<u8>;
    
    /// 从二进制反序列化字节码
    pub fn deserialize_bytecode(&self, data: &[u8]) -> Result<BytecodeModule>;
    
    /// 执行字节码模块
    pub fn execute_bytecode(&mut self, module: &BytecodeModule) -> Result<Value>;
    
    /// 从文件加载并执行预编译字节码
    pub fn load_bytecode(&mut self, path: &str) -> Result<Value>;
}
```

### 使用场景

1. **AOT 编译**
   ```rust
   // 开发时预编译
   let bytecode = ctx.compile(source)?;
   let binary = ctx.serialize_bytecode(&bytecode);
   std::fs::write("script.pyq", binary)?;
   
   // 运行时加载
   ctx.load_bytecode("script.pyq")?;
   ```

2. **字节码缓存**
   ```rust
   // 编译一次，多次执行
   let module = ctx.compile(source)?;
   for _ in 0..1000 {
       ctx.execute_bytecode(&module)?;
   }
   ```

3. **代码保护**
   - 分发字节码而非源码，保护知识产权
   - 字节码更难逆向工程

## 异步运行时设计

原生集成异步支持，与 Rust 的 async/await 无缝对接：

### 异步值类型
```rust
pub enum Value {
    // ... 其他类型
    
    // 异步相关类型
    Future(FutureRef),      // 异步 Future
    Coroutine(CoroutineRef), // 协程
}

pub struct Future {
    state: FutureState,
    waker: Option<Waker>,
}

pub enum FutureState {
    Pending,
    Ready(Value),
    Error(Error),
}
```

### 异步函数定义
```python
# Python 代码
async def fetch_data(url):
    response = await http_get(url)
    return response.json()

# 调用
result = await fetch_data("https://api.example.com")
```

### VM 异步执行
```rust
impl VM {
    /// 执行异步函数
    pub async fn execute_async(&mut self, code: &ByteCode) -> Result<Value> {
        loop {
            let opcode = self.fetch_opcode();
            
            match opcode {
                OpCode::Await => {
                    // 遇到 await 指令
                    let future = self.pop_stack()?;
                    let result = self.await_future(future).await?;
                    self.push_stack(result);
                }
                
                OpCode::Yield => {
                    // 协程让出控制权
                    tokio::task::yield_now().await;
                }
                
                // ... 其他指令
            }
        }
    }
    
    /// 等待 Future 完成
    async fn await_future(&mut self, future: Value) -> Result<Value> {
        match future {
            Value::Future(f) => f.await,
            _ => Err(Error::Type("Expected Future".into())),
        }
    }
}
```

### Rust 集成
```rust
// 从 Rust 注册异步函数
ctx.register_async_function("http_get", |url: String| async move {
    let response = reqwest::get(&url).await?;
    let text = response.text().await?;
    Ok(text)
})?;

// 在 Rust 中调用 Python 异步函数
let result = ctx.call_async("fetch_data", &[Value::from("https://...")]).await?;
```

### 异步运行时特性
- ✅ **原生集成**: 直接内置在 VM 中，无需外部库
- ✅ **Rust 互操作**: 与 Rust 的 async/await 无缝对接
- ✅ **轻量级**: 不依赖复杂的事件循环
- ✅ **高性能**: 利用 Rust 的 tokio 运行时
- ✅ **简单**: 开发者使用标准的 async/await 语法

### 执行模型
```
Python async function
        ↓
    Bytecode (含 Await 指令)
        ↓
    VM 执行到 Await
        ↓
    转换为 Rust Future
        ↓
    Tokio 运行时调度
        ↓
    返回结果到 Python
```

## 执行流程

1. **直接执行模式**
   ```
   Source Code → rustpython_parser (AST) → Compiler → ByteCode → VM → Result
   ```

2. **预编译模式**
   ```
   编译阶段: Source Code (.py) → rustpython_parser → Compiler → ByteCode → Serialize → .pyq 文件
   执行阶段: .pyq 文件 → Deserialize → ByteCode → VM → Result
   ```

3. **Rust 互操作**
   ```
   Rust Function → Value Conversion → Python Call
   Python Return → Value Conversion → Rust Type
   ```

## 性能优化点

### 编译时优化
1. **常量折叠** - 编译时计算常量表达式
2. **死代码消除** - 移除不可达代码
3. **窥孔优化** - 优化指令序列
4. **字节码压缩** - 减小字节码体积
5. **静态属性查找** - 编译时确定对象属性位置（slot）

### 运行时优化
1. **字节码缓存** - 避免重复编译（预编译支持）
2. **Slot 查找** - 对象属性直接索引，O(1) 访问
3. **快速路径** - 常见操作的优化路径（如整数加法）
4. **引用计数优化** - 减少不必要的引用计数操作
5. **栈分配优化** - 小对象栈上分配

### 未来优化
1. **JIT 编译** - 热点代码即时编译
2. **类型特化** - 基于运行时类型信息特化代码
3. **向量化** - SIMD 指令优化数值计算
