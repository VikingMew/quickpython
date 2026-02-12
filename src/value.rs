use crate::bytecode::ByteCode;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Dictionary key type - only String and Int are supported
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DictKey {
    String(String),
    Int(i32),
}

/// Value type for QuickPython runtime
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i32),
    Float(f64),
    Bool(bool),
    None,
    String(String),
    List(Rc<RefCell<Vec<Value>>>),
    Dict(Rc<RefCell<HashMap<DictKey, Value>>>),
    Function(Function),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub code: ByteCode,
}

impl Value {
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<Rc<RefCell<Vec<Value>>>> {
        match self {
            Value::List(list) => Some(list.clone()),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<Rc<RefCell<HashMap<DictKey, Value>>>> {
        match self {
            Value::Dict(dict) => Some(dict.clone()),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::None => false,
            Value::String(s) => !s.is_empty(),
            Value::List(list) => !list.borrow().is_empty(),
            Value::Dict(dict) => !dict.borrow().is_empty(),
            Value::Function(_) => true,
        }
    }
}
