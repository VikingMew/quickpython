/// Bytecode instructions for the VM
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // 栈操作
    PushInt(i32),
    PushFloat(f64),
    PushBool(bool),
    PushNone,
    PushString(String),
    Pop,

    // 算术运算
    Add,
    Sub,
    Mul,
    Div,

    // 比较运算
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // 变量
    GetGlobal(String),
    SetGlobal(String),
    GetLocal(usize),
    SetLocal(usize),

    // 控制流
    Jump(usize),
    JumpIfFalse(usize),

    // 函数
    MakeFunction {
        name: String,
        params: Vec<String>,
        code_len: usize,
    },
    Call(usize), // 参数数量
    Return,

    // 内置函数
    Print,
    Int,   // int() 类型转换
    Float, // float() 类型转换
    Len,   // len() 函数

    // 列表和字典
    BuildList(usize),          // 从栈顶取 n 个元素构建列表
    BuildDict(usize),          // 从栈顶取 n*2 个元素构建字典（键值对）
    GetItem,                   // 索引访问 list[i] 或 dict[key]
    SetItem,                   // 索引赋值 list[i] = x 或 dict[key] = x
    CallMethod(String, usize), // 方法调用 obj.method(args)
}

pub type ByteCode = Vec<Instruction>;
