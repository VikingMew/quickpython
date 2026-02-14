use crate::bytecode::ByteCode;
use crate::compiler::Compiler;
use crate::value::{Module, Value};
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
        self.vm
            .execute(&bytecode, &mut self.globals)
            .map_err(|e| format!("{:?}", e))
    }

    pub fn eval_bytecode(&mut self, bytecode: &ByteCode) -> Result<Value, String> {
        self.vm
            .execute(bytecode, &mut self.globals)
            .map_err(|e| format!("{:?}", e))
    }

    pub fn register_extension_module(&mut self, name: &str, module: Module) {
        self.vm.register_extension_module(name, module);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.globals.get(name).cloned()
    }

    pub fn set(&mut self, name: &str, value: Value) {
        self.globals.insert(name.to_string(), value);
    }
}
