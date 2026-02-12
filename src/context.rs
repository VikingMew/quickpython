use crate::compiler::Compiler;
use crate::value::Value;
use crate::vm::VM;
use std::collections::HashMap;

pub struct Context {
    vm: VM,
    globals: HashMap<String, Value>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            vm: VM::new(),
            globals: HashMap::new(),
        }
    }

    pub fn eval(&mut self, source: &str) -> Result<Value, String> {
        let bytecode = Compiler::compile(source)?;
        self.vm.execute(&bytecode, &mut self.globals)
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.globals.get(name).cloned()
    }

    pub fn set(&mut self, name: &str, value: Value) {
        self.globals.insert(name.to_string(), value);
    }
}
