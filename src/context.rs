use crate::compiler::Compiler;
use crate::value::Value;
use crate::vm::VM;

pub struct Context {
    vm: VM,
}

impl Context {
    pub fn new() -> Self {
        Context { vm: VM::new() }
    }

    pub fn eval(&mut self, source: &str) -> Result<Value, String> {
        let bytecode = Compiler::compile(source)?;
        self.vm.execute(&bytecode)
    }
}
