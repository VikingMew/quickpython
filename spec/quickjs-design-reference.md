# QuickJS 设计借鉴分析

基于 QuickJS 头文件的深入分析，以下是 QuickPython 可以直接借鉴的核心设计。

## 1. NaN Boxing 实现 ⭐⭐⭐⭐⭐

QuickJS 提供了三种 Value 表示方式，QuickPython 应该直接采用：

### 方案选择
```rust
// 32 位系统：使用 NaN Boxing
#[cfg(target_pointer_width = "32")]
pub type Value = u64;

// 64 位系统：可选 NaN Boxing 或 Tagged Union
#[cfg(target_pointer_width = "64")]
pub type Value = u64;  // 或 TaggedUnion
```

### Tag 定义（直接借鉴）
```rust
pub const JS_TAG_FIRST: i32 = -9;
pub const JS_TAG_BIG_INT: i32 = -9;
pub const JS_TAG_STRING: i32 = -7;
pub const JS_TAG_OBJECT: i32 = -1;

pub const JS_TAG_INT: i32 = 0;
pub const JS_TAG_BOOL: i32 = 1;
pub const JS_TAG_NULL: i32 = 2;
pub const JS_TAG_UNDEFINED: i32 = 3;
pub const JS_TAG_EXCEPTION: i32 = 6;
pub const JS_TAG_FLOAT64: i32 = 8;
```

### NaN Boxing 实现
```rust
// 64 位 NaN Boxing
const JS_FLOAT64_TAG_ADDEND: u64 = 0x7ff80000 - JS_TAG_FIRST as u64 + 1;
const JS_NAN: u64 = 0x7ff8000000000000 - (JS_FLOAT64_TAG_ADDEND << 32);

impl Value {
    #[inline]
    pub fn new_float64(d: f64) -> Self {
        let mut u = d.to_bits();
        // 标准化 NaN
        if (u & 0x7fffffffffffffff) > 0x7ff0000000000000 {
            Value(JS_NAN)
        } else {
            Value(u - (JS_FLOAT64_TAG_ADDEND << 32))
        }
    }
    
    #[inline]
    pub fn get_float64(&self) -> f64 {
        let u = self.0 + (JS_FLOAT64_TAG_ADDEND << 32);
        f64::from_bits(u)
    }
    
    #[inline]
    pub fn get_tag(&self) -> i32 {
        (self.0 >> 32) as i32
    }
    
    #[inline]
    pub fn get_int(&self) -> i32 {
        self.0 as i32
    }
}
```

**优势**：
- ✅ 经过实战验证的设计
- ✅ 性能优异
- ✅ 内存高效

## 2. 引用计数机制 ⭐⭐⭐⭐⭐

QuickJS 的引用计数设计非常简洁：

```rust
// 检查是否需要引用计数
#[inline]
pub fn has_ref_count(v: Value) -> bool {
    v.get_tag() < JS_TAG_FIRST
}

impl Value {
    pub fn dup(&self, ctx: &Context) -> Self {
        if has_ref_count(*self) {
            // 增加引用计数
            unsafe {
                let ptr = self.get_ptr() as *mut RefCounted;
                (*ptr).ref_count += 1;
            }
        }
        *self
    }
    
    pub fn free(&self, ctx: &mut Context) {
        if has_ref_count(*self) {
            unsafe {
                let ptr = self.get_ptr() as *mut RefCounted;
                (*ptr).ref_count -= 1;
                if (*ptr).ref_count == 0 {
                    // 释放对象
                    self.destroy(ctx);
                }
            }
        }
    }
}

struct RefCounted {
    ref_count: u32,
    // ... 其他字段
}
```

**关键点**：
- 只有堆分配对象需要引用计数
- 小整数、布尔、None 等不需要引用计数
- 引用计数为 0 时立即释放

## 3. Atom（原子化字符串）系统 ⭐⭐⭐⭐⭐

QuickJS 的 Atom 设计非常值得借鉴：

```rust
pub type JSAtom = u32;
pub const JS_ATOM_NULL: JSAtom = 0;

pub struct AtomTable {
    // 字符串 -> Atom ID
    string_to_atom: HashMap<String, JSAtom>,
    // Atom ID -> 字符串
    atom_to_string: Vec<Rc<String>>,
    // 引用计数
    ref_counts: Vec<u32>,
}

impl AtomTable {
    pub fn new_atom(&mut self, s: &str) -> JSAtom {
        if let Some(&atom) = self.string_to_atom.get(s) {
            self.ref_counts[atom as usize] += 1;
            return atom;
        }
        
        let atom = self.atom_to_string.len() as JSAtom;
        let rc_str = Rc::new(s.to_string());
        self.atom_to_string.push(rc_str);
        self.string_to_atom.insert(s.to_string(), atom);
        self.ref_counts.push(1);
        atom
    }
    
    pub fn dup_atom(&mut self, atom: JSAtom) {
        if atom != JS_ATOM_NULL {
            self.ref_counts[atom as usize] += 1;
        }
    }
    
    pub fn free_atom(&mut self, atom: JSAtom) {
        if atom != JS_ATOM_NULL {
            self.ref_counts[atom as usize] -= 1;
            if self.ref_counts[atom as usize] == 0 {
                // 可以选择延迟删除或立即删除
            }
        }
    }
    
    pub fn atom_to_string(&self, atom: JSAtom) -> &str {
        &self.atom_to_string[atom as usize]
    }
}
```

**用途**：
- 属性名
- 变量名
- 函数名
- 字符串常量

## 4. Runtime 和 Context 分离 ⭐⭐⭐⭐

QuickJS 的两层架构值得借鉴：

```rust
// Runtime - 全局资源管理
pub struct Runtime {
    // 内存分配器
    malloc_functions: MallocFunctions,
    // 内存限制
    memory_limit: usize,
    // GC 阈值
    gc_threshold: usize,
    // 栈大小限制
    max_stack_size: usize,
    // Atom 表（全局共享）
    atom_table: AtomTable,
    // 类注册表
    class_registry: HashMap<ClassID, ClassDef>,
    // 调试标志
    dump_flags: u64,
}

// Context - 执行上下文
pub struct Context {
    // 所属 Runtime
    runtime: *mut Runtime,
    // 全局对象
    global_object: Value,
    // 类原型
    class_protos: HashMap<ClassID, Value>,
    // 异常状态
    current_exception: Option<Value>,
    // 调用栈
    call_stack: Vec<Frame>,
}
```

**优势**：
- 多个 Context 可以共享一个 Runtime
- 资源隔离更清晰
- 便于实现沙箱

## 5. 内存分配接口 ⭐⭐⭐⭐

QuickJS 提供了可定制的内存分配器：

```rust
pub struct MallocFunctions {
    pub calloc: fn(opaque: *mut c_void, count: usize, size: usize) -> *mut c_void,
    pub malloc: fn(opaque: *mut c_void, size: usize) -> *mut c_void,
    pub free: fn(opaque: *mut c_void, ptr: *mut c_void),
    pub realloc: fn(opaque: *mut c_void, ptr: *mut c_void, size: usize) -> *mut c_void,
    pub malloc_usable_size: fn(ptr: *const c_void) -> usize,
}

impl Runtime {
    pub fn new_with_allocator(mf: MallocFunctions, opaque: *mut c_void) -> Self {
        // 使用自定义分配器
    }
}
```

**用途**：
- 内存跟踪
- 内存限制
- 自定义分配策略

## 6. 类系统 ⭐⭐⭐⭐

QuickJS 的类系统设计：

```rust
pub type ClassID = u32;
pub const JS_INVALID_CLASS_ID: ClassID = 0;

pub struct ClassDef {
    pub class_name: &'static str,
    pub finalizer: Option<fn(*mut Runtime, Value)>,
    pub gc_mark: Option<fn(*mut Runtime, Value, MarkFunc)>,
    pub call: Option<fn(*mut Context, Value, Value, &[Value], i32) -> Value>,
    pub exotic: Option<*const ExoticMethods>,
}

impl Runtime {
    pub fn new_class_id(&mut self) -> ClassID {
        let id = self.next_class_id;
        self.next_class_id += 1;
        id
    }
    
    pub fn new_class(&mut self, class_id: ClassID, class_def: ClassDef) -> Result<()> {
        self.class_registry.insert(class_id, class_def);
        Ok(())
    }
}
```

**QuickPython 适配**：
- 简化 exotic methods（不需要完整的 Proxy 支持）
- 使用 Slot 而非动态属性

## 7. 异常处理 ⭐⭐⭐⭐

QuickJS 的异常机制：

```rust
impl Context {
    pub fn throw(&mut self, obj: Value) -> Value {
        self.current_exception = Some(obj);
        Value::EXCEPTION
    }
    
    pub fn get_exception(&mut self) -> Value {
        self.current_exception.take().unwrap_or(Value::UNDEFINED)
    }
    
    pub fn has_exception(&self) -> bool {
        self.current_exception.is_some()
    }
}

// 特殊值
pub const JS_EXCEPTION: Value = Value::new_tag(JS_TAG_EXCEPTION, 0);
```

**使用模式**：
```rust
let result = some_operation(ctx);
if result == Value::EXCEPTION {
    let exc = ctx.get_exception();
    // 处理异常
}
```

## 8. 属性标志 ⭐⭐⭐

QuickJS 的属性标志系统：

```rust
pub const JS_PROP_CONFIGURABLE: i32 = 1 << 0;
pub const JS_PROP_WRITABLE: i32 = 1 << 1;
pub const JS_PROP_ENUMERABLE: i32 = 1 << 2;
pub const JS_PROP_C_W_E: i32 = JS_PROP_CONFIGURABLE | JS_PROP_WRITABLE | JS_PROP_ENUMERABLE;
```

**QuickPython 简化**：
- 只保留基础标志
- Slot 属性默认可写、可配置

## 9. 模块系统接口 ⭐⭐⭐

QuickJS 的模块加载器设计：

```rust
pub type ModuleNormalizeFunc = fn(*mut Context, &str, &str, *mut c_void) -> *mut c_char;
pub type ModuleLoaderFunc = fn(*mut Context, &str, *mut c_void) -> *mut ModuleDef;

impl Runtime {
    pub fn set_module_loader(
        &mut self,
        normalize: Option<ModuleNormalizeFunc>,
        loader: ModuleLoaderFunc,
        opaque: *mut c_void,
    ) {
        self.module_normalize = normalize;
        self.module_loader = Some(loader);
        self.module_loader_opaque = opaque;
    }
}
```

## 10. C 函数绑定 ⭐⭐⭐⭐

QuickJS 的函数绑定非常灵活：

```rust
pub enum CFunctionEnum {
    Generic,
    GenericMagic,
    Constructor,
    ConstructorMagic,
    Getter,
    Setter,
    GetterMagic,
    SetterMagic,
}

pub struct CFunctionListEntry {
    pub name: &'static str,
    pub prop_flags: u8,
    pub def_type: u8,
    pub magic: i16,
    // ... union 字段
}
```

**QuickPython 适配**：
```rust
ctx.register_function("add", |a: i64, b: i64| a + b)?;
ctx.register_async_function("fetch", |url: String| async move {
    // ...
})?;
```

## 11. 调试支持 ⭐⭐⭐

QuickJS 的调试标志：

```rust
pub const JS_DUMP_BYTECODE_FINAL: u64 = 0x01;
pub const JS_DUMP_BYTECODE_HEX: u64 = 0x10;
pub const JS_DUMP_GC: u64 = 0x400;
pub const JS_DUMP_LEAKS: u64 = 0x4000;
pub const JS_DUMP_MEM: u64 = 0x10000;

impl Runtime {
    pub fn set_dump_flags(&mut self, flags: u64) {
        self.dump_flags = flags;
    }
}
```

## 12. 内存使用统计 ⭐⭐⭐

QuickJS 提供详细的内存统计：

```rust
pub struct MemoryUsage {
    pub malloc_size: i64,
    pub malloc_limit: i64,
    pub memory_used_size: i64,
    pub atom_count: i64,
    pub atom_size: i64,
    pub str_count: i64,
    pub str_size: i64,
    pub obj_count: i64,
    pub obj_size: i64,
    // ...
}

impl Runtime {
    pub fn compute_memory_usage(&self) -> MemoryUsage {
        // 统计内存使用
    }
}
```

## 总结：QuickPython 应该借鉴的核心设计

### 必须借鉴 ⭐⭐⭐⭐⭐
1. **NaN Boxing** - 完整的实现方案
2. **引用计数** - 简洁高效的内存管理
3. **Atom 系统** - 字符串原子化
4. **Runtime/Context 分离** - 清晰的架构
5. **紧凑字节码** - 变长编码

### 强烈推荐 ⭐⭐⭐⭐
1. **内存分配接口** - 可定制的分配器
2. **异常处理** - 简单的异常机制
3. **调试支持** - 完善的调试工具

### 可选借鉴 ⭐⭐⭐
1. **模块系统** - 模块加载器接口
2. **属性标志** - 属性元数据
3. **内存统计** - 内存使用分析

### 不需要借鉴 ❌
1. **Exotic Methods** - 太复杂，QuickPython 用 Slot
2. **Proxy 支持** - 不需要
3. **Promise** - QuickPython 用 Rust async/await
4. **WeakRef/WeakMap** - 暂不需要
5. **C 函数绑定** - QuickPython 用 Rust 原生绑定

## QuickPython 的核心差异

### 1. 异步模型
```rust
// QuickJS: Promise
let promise = JS_NewPromise(ctx);

// QuickPython: Rust async/await
async fn fetch(url: String) -> Result<String> {
    reqwest::get(&url).await?.text().await
}
```

### 2. FFI 绑定
```rust
// QuickJS: C 函数
JSValue my_func(JSContext *ctx, JSValueConst this_val, 
                int argc, JSValueConst *argv);

// QuickPython: Rust 函数（类型安全）
ctx.register_function("add", |a: i64, b: i64| a + b)?;
ctx.register_async_function("fetch", |url: String| async move {
    // Rust async 代码
})?;
```

### 3. 属性访问
```rust
// QuickJS: 动态属性表
obj.prop_table[hash(name)]

// QuickPython: Slot 静态查找
obj.slots[compile_time_index]  // O(1) 数组访问
```

### 4. 依赖
```
QuickJS:  需要 C 编译器
QuickPython: 纯 Rust，cargo build 即可
```

## 设计哲学

**QuickJS**: JavaScript 引擎，兼容 ES2020，支持完整的 JS 语义

**QuickPython**: 
- Python 风格的胶水语言
- 性能优先，简化语义
- Rust 生态原生集成
- 不追求完全兼容 Python
- 专注嵌入式场景
