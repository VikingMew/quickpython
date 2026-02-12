/// Bytecode instructions for the VM
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    PushInt(i32),
    Add,
    Sub,
    Mul,
    Div,
    GetGlobal(String),
    SetGlobal(String),
    Pop,
}

pub type ByteCode = Vec<Instruction>;
