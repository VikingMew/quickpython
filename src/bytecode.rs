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
}

pub type ByteCode = Vec<Instruction>;
