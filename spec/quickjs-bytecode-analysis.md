# QuickJS 字节码借鉴分析

## QuickJS 字节码特点

QuickJS 使用了非常紧凑和高效的字节码设计，以下是可以借鉴的部分：

## 1. 可直接借鉴的字节码指令

### 基础栈操作
```rust
// QuickJS 风格的栈操作
Push            // 压栈
Pop             // 弹栈
Dup             // 复制栈顶
Swap            // 交换栈顶两个元素
Rot3            // 旋转栈顶三个元素
```

### 常量加载
```rust
// 小常量优化（QuickJS 特色）
PushI8(i8)      // 压入小整数 (-128 ~ 127)
PushI16(i16)    // 压入中等整数
PushI32(i32)    // 压入大整数
PushConst(u32)  // 从常量池加载

PushTrue        // 压入 true
PushFalse       // 压入 false
PushNone        // 压入 None
PushUndefined   // 可选：未定义值
```

### 变量操作
```rust
// 局部变量（QuickJS 使用索引）
GetLocal(u16)   // 获取局部变量
SetLocal(u16)   // 设置局部变量
GetArg(u16)     // 获取参数

// 全局变量
GetGlobal(u32)  // 获取全局变量
SetGlobal(u32)  // 设置全局变量

// 闭包变量
GetVar(u16)     // 获取闭包变量
SetVar(u16)     // 设置闭包变量
```

### 算术运算
```rust
// 二元运算
Add             // 加法
Sub             // 减法
Mul             // 乘法
Div             // 除法
Mod             // 取模
Pow             // 幂运算

// 一元运算
Neg             // 取负
Not             // 逻辑非
BitNot          // 按位取反

// 位运算
Shl             // 左移
Shr             // 右移（算术）
UShr            // 右移（逻辑）
And             // 按位与
Or              // 按位或
Xor             // 按位异或
```

### 比较运算
```rust
Eq              // ==
Ne              // !=
Lt              // <
Le              // <=
Gt              // >
Ge              // >=
StrictEq        // === (可选，Python 用 is)
StrictNe        // !== (可选)
```

### 控制流
```rust
// 跳转（QuickJS 使用相对偏移）
Jump(i32)           // 无条件跳转
JumpIfTrue(i32)     // 真则跳转
JumpIfFalse(i32)    // 假则跳转

// 短跳转优化
JumpShort(i8)       // 短跳转 (-128 ~ 127)
JumpIfTrueShort(i8)
JumpIfFalseShort(i8)

// 函数调用
Call(u16)           // 调用函数（参数个数）
TailCall(u16)       // 尾调用优化
Return              // 返回
```

### 对象和属性
```rust
// 对象创建
NewObject           // 创建空对象
NewArray(u32)       // 创建数组

// 属性访问（QuickJS 使用原子化字符串）
GetProp(u32)        // 获取属性（属性名索引）
SetProp(u32)        // 设置属性
GetPropByVal        // 通过值获取属性（如 obj[key]）
SetPropByVal        // 通过值设置属性

// 索引访问
GetIndex            // 获取索引（如 list[0]）
SetIndex            // 设置索引
```

## 2. 需要适配的字节码

### 迭代器（Python 特有）
```rust
GetIter             // 获取迭代器
ForIter(i32)        // for 循环迭代（失败则跳转）
```

### 异常处理（Python 特有）
```rust
SetupExcept(i32)    // 设置异常处理器
PopExcept           // 弹出异常处理器
Raise               // 抛出异常
```

### 异步（需要扩展）
```rust
Await               // 等待 Future
Yield               // 协程让出
YieldFrom           // yield from
```

### 解包操作（Python 特有）
```rust
UnpackSeq(u16)      // 解包序列
UnpackEx(u16, u16)  // 扩展解包
```

## 3. QuickJS 的优化技巧

### 变长编码
```rust
// QuickJS 根据操作数大小选择不同指令
enum OpCode {
    // 8 位操作数（1 字节指令 + 1 字节操作数）
    GetLocal8(u8),
    SetLocal8(u8),
    PushI8(i8),
    
    // 16 位操作数（1 字节指令 + 2 字节操作数）
    GetLocal16(u16),
    SetLocal16(u16),
    PushI16(i16),
    
    // 32 位操作数（1 字节指令 + 4 字节操作数）
    GetLocal32(u32),
    SetLocal32(u32),
    PushI32(i32),
}
```

### 组合指令（减少指令数）
```rust
// QuickJS 的组合指令
GetLocal_PushI8(u8, i8)     // 获取局部变量 + 压入整数
Add_SetLocal(u8)            // 加法 + 设置局部变量
GetProp_Call(u32, u16)      // 获取属性 + 调用
```

### 快速路径
```rust
// 常见操作的快速版本
AddI8(i8)           // 加上小整数
SubI8(i8)           // 减去小整数
MulI8(i8)           // 乘以小整数
IncLocal(u8)        // 局部变量自增
DecLocal(u8)        // 局部变量自减
```

## 4. QuickPython 字节码设计建议

### 核心指令集（约 60-80 条）
```rust
pub enum OpCode {
    // === 栈操作 (5 条) ===
    Pop,
    Dup,
    Swap,
    Rot3,
    
    // === 常量 (10 条) ===
    PushNone,
    PushTrue,
    PushFalse,
    PushI8(i8),
    PushI16(i16),
    PushI32(i32),
    PushConst8(u8),
    PushConst16(u16),
    PushConst32(u32),
    
    // === 变量 (12 条) ===
    GetLocal8(u8),
    GetLocal16(u16),
    SetLocal8(u8),
    SetLocal16(u16),
    GetGlobal8(u8),
    GetGlobal16(u16),
    SetGlobal8(u8),
    SetGlobal16(u16),
    GetArg8(u8),
    GetArg16(u16),
    
    // === 算术运算 (15 条) ===
    Add, Sub, Mul, Div, FloorDiv, Mod, Pow,
    Neg, Not,
    Shl, Shr,
    BitAnd, BitOr, BitXor, BitNot,
    
    // === 比较 (8 条) ===
    Eq, Ne, Lt, Le, Gt, Ge,
    Is, IsNot,
    
    // === 控制流 (10 条) ===
    Jump(i32),
    JumpShort(i8),
    JumpIfTrue(i32),
    JumpIfFalse(i32),
    JumpIfTrueShort(i8),
    JumpIfFalseShort(i8),
    Call8(u8),
    Call16(u16),
    Return,
    
    // === 对象和属性 (10 条) ===
    NewObject,
    NewList(u16),
    NewDict(u16),
    GetAttr8(u8),      // Slot 索引
    GetAttr16(u16),
    SetAttr8(u8),
    SetAttr16(u16),
    GetItem,
    SetItem,
    
    // === Python 特有 (8 条) ===
    GetIter,
    ForIter(i32),
    UnpackSeq(u16),
    BuildSlice,
    
    // === 异步 (3 条) ===
    Await,
    Yield,
    
    // === 异常 (4 条) ===
    SetupExcept(i32),
    PopExcept,
    Raise,
    
    // === 优化指令 (可选 10 条) ===
    AddI8(i8),          // 加小整数
    SubI8(i8),
    IncLocal8(u8),      // 局部变量自增
    DecLocal8(u8),
    GetLocal_Call8(u8, u8),  // 组合指令
}
```

## 5. 字节码大小对比

| 操作 | QuickJS | Python | QuickPython (建议) |
|------|---------|--------|-------------------|
| 加法 | 1 字节 | 2 字节 | 1 字节 |
| 加小整数 | 2 字节 | 4 字节 | 2 字节 |
| 获取局部变量 | 2 字节 | 2-4 字节 | 2 字节 |
| 跳转 | 2-5 字节 | 3 字节 | 2-5 字节 |
| 函数调用 | 2 字节 | 2 字节 | 2 字节 |

## 6. 实现优先级

### Phase 1（MVP）
- 基础栈操作
- 常量加载
- 变量操作（局部、全局）
- 算术和比较运算
- 基础控制流（跳转、调用、返回）
- 基础对象操作

### Phase 2（扩展）
- 对象和属性（Slot）
- 迭代器
- 异常处理
- 列表和字典操作

### Phase 3（优化）
- 异步指令
- 组合指令
- 快速路径优化
- 尾调用优化

## 7. 关键差异

| 特性 | QuickJS | QuickPython |
|------|---------|-------------|
| 属性访问 | 动态属性表 | 固定 Slot |
| 闭包 | 支持 | 简化或不支持 |
| 原型链 | 支持 | 不支持 |
| this 绑定 | 复杂 | 简化（self） |
| 异步 | Promise | async/await + Future |
| 类型系统 | 动态 | 动态 + Tagged Pointers |

## 总结

QuickPython 可以借鉴 QuickJS 的：
- ✅ 变长编码策略
- ✅ 紧凑的指令格式
- ✅ 栈操作指令
- ✅ 常量加载优化
- ✅ 短跳转优化
- ✅ 组合指令思想

需要适配的：
- 🔄 属性访问（Slot 而非动态属性）
- 🔄 迭代器协议
- 🔄 异常处理
- 🔄 异步模型（Future 而非 Promise）
- 🔄 Python 特有语法（解包、切片等）
