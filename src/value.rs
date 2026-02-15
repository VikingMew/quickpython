use crate::bytecode::ByteCode;
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Native function type - Rust functions callable from Python
pub type NativeFunction = fn(Vec<Value>) -> Result<Value, Value>;

/// Module structure
#[derive(Clone)]
pub struct Module {
    pub name: String,
    pub attributes: HashMap<String, Value>,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Module {
            name: name.to_string(),
            attributes: HashMap::new(),
        }
    }

    pub fn add_function(&mut self, name: &str, func: NativeFunction) {
        self.attributes
            .insert(name.to_string(), Value::NativeFunction(func));
    }

    pub fn get_attribute(&self, name: &str) -> Option<Value> {
        self.attributes.get(name).cloned()
    }
}

impl std::fmt::Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("name", &self.name)
            .field("attributes", &self.attributes.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Match object for regex matches
#[derive(Debug, Clone)]
pub struct MatchObject {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub groups: Vec<Option<String>>,
}

impl MatchObject {
    pub fn new(text: String, start: usize, end: usize, groups: Vec<Option<String>>) -> Self {
        MatchObject {
            text,
            start,
            end,
            groups,
        }
    }
}

/// List value with version tracking for iterator modification detection
#[derive(Debug, Clone)]
pub struct ListValue {
    pub items: Vec<Value>,
    pub version: usize,
}

impl ListValue {
    pub fn new() -> Self {
        ListValue {
            items: Vec::new(),
            version: 0,
        }
    }

    pub fn with_items(items: Vec<Value>) -> Self {
        ListValue { items, version: 0 }
    }

    pub fn increment_version(&mut self) {
        self.version = self.version.wrapping_add(1);
    }
}

/// Dictionary key type - only String and Int are supported
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DictKey {
    String(String),
    Int(i32),
}

/// Iterator state for different types
#[derive(Debug, Clone)]
pub enum IteratorState {
    Range {
        current: i32,
        stop: i32,
        step: i32,
    },
    List {
        list: Rc<RefCell<ListValue>>,
        index: usize,
        version: usize, // Version at iterator creation time
    },
    DictKeys {
        keys: Vec<DictKey>,
        index: usize,
    },
    String {
        chars: Vec<char>,
        index: usize,
    },
}

/// Exception type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ExceptionType {
    Exception,         // 基础异常
    RuntimeError,      // 运行时错误
    IndexError,        // 索引越界
    KeyError,          // 键不存在
    ValueError,        // 值错误
    TypeError,         // 类型错误
    ZeroDivisionError, // 除零错误
    IteratorError,     // 迭代器错误（自定义）
    OSError,           // 操作系统错误
    AttributeError,    // 属性错误
}

impl ExceptionType {
    pub fn as_i32(&self) -> i32 {
        match self {
            ExceptionType::Exception => 0,
            ExceptionType::RuntimeError => 1,
            ExceptionType::IndexError => 2,
            ExceptionType::KeyError => 3,
            ExceptionType::ValueError => 4,
            ExceptionType::TypeError => 5,
            ExceptionType::ZeroDivisionError => 6,
            ExceptionType::IteratorError => 7,
            ExceptionType::OSError => 8,
            ExceptionType::AttributeError => 9,
        }
    }

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(ExceptionType::Exception),
            1 => Some(ExceptionType::RuntimeError),
            2 => Some(ExceptionType::IndexError),
            3 => Some(ExceptionType::KeyError),
            4 => Some(ExceptionType::ValueError),
            5 => Some(ExceptionType::TypeError),
            6 => Some(ExceptionType::ZeroDivisionError),
            7 => Some(ExceptionType::IteratorError),
            8 => Some(ExceptionType::OSError),
            9 => Some(ExceptionType::AttributeError),
            _ => None,
        }
    }

    /// Check if this exception type matches the handler type
    /// Exception matches all types (it's the base class)
    pub fn matches(&self, handler: &ExceptionType) -> bool {
        match handler {
            ExceptionType::Exception => true, // Exception catches everything
            _ => self == handler,             // Otherwise exact match
        }
    }
}

/// Traceback frame for exception
#[derive(Debug, Clone)]
pub struct TracebackFrame {
    pub function_name: String,
    pub line_number: usize,
}

/// Exception value structure
#[derive(Debug, Clone)]
pub struct ExceptionValue {
    pub exception_type: ExceptionType,
    pub message: String,
    pub traceback: Option<Vec<TracebackFrame>>,
}

/// Value type for QuickPython runtime
#[derive(Clone)]
pub enum Value {
    Int(i32),
    Float(f64),
    Bool(bool),
    None,
    String(String),
    List(Rc<RefCell<ListValue>>),
    Dict(Rc<RefCell<HashMap<DictKey, Value>>>),
    Tuple(Rc<Vec<Value>>), // Immutable sequence
    Slice {
        start: Option<i32>,
        stop: Option<i32>,
        step: Option<i32>,
    },
    Iterator(Rc<RefCell<IteratorState>>),
    Function(Function),
    Exception(ExceptionValue),
    Module(Rc<RefCell<Module>>),
    NativeFunction(NativeFunction),
    BoundMethod(Box<Value>, String), // (receiver, method_name)
    Regex(Rc<Regex>),
    Match(Rc<MatchObject>),
    Type(TypeObject),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeObject {
    Int,
    Float,
    Bool,
    Str,
    List,
    Dict,
    Tuple,
    NoneType,
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "Int({})", i),
            Value::Float(fl) => write!(f, "Float({})", fl),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::None => write!(f, "None"),
            Value::String(s) => write!(f, "String({:?})", s),
            Value::List(l) => write!(f, "List({:?})", l),
            Value::Dict(d) => write!(f, "Dict({:?})", d),
            Value::Tuple(t) => write!(f, "Tuple({:?})", t),
            Value::Slice { start, stop, step } => {
                write!(f, "Slice({:?}:{:?}:{:?})", start, stop, step)
            }
            Value::Iterator(i) => write!(f, "Iterator({:?})", i),
            Value::Function(func) => write!(f, "Function({:?})", func),
            Value::Exception(e) => write!(f, "Exception({:?})", e),
            Value::Module(m) => write!(f, "Module({:?})", m),
            Value::NativeFunction(_) => write!(f, "NativeFunction(<native>)"),
            Value::BoundMethod(_, method_name) => write!(f, "BoundMethod(<{}>)", method_name),
            Value::Regex(_) => write!(f, "Regex(<pattern>)"),
            Value::Match(m) => write!(f, "Match({:?})", m),
            Value::Type(t) => write!(f, "Type({:?})", t),
        }
    }
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

    pub fn as_list(&self) -> Option<Rc<RefCell<ListValue>>> {
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

    /// Create an exception value
    pub fn error(exception_type: ExceptionType, message: impl Into<String>) -> Value {
        Value::Exception(ExceptionValue {
            exception_type,
            message: message.into(),
            traceback: None,
        })
    }

    /// Check if value is an exception
    pub fn is_exception(&self) -> bool {
        matches!(self, Value::Exception(_))
    }

    /// Get exception value
    pub fn as_exception(&self) -> Option<&ExceptionValue> {
        match self {
            Value::Exception(exc) => Some(exc),
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
            Value::List(list) => !list.borrow().items.is_empty(),
            Value::Dict(dict) => !dict.borrow().is_empty(),
            Value::Tuple(tuple) => !tuple.is_empty(),
            Value::Slice { .. } => true,
            Value::Iterator(_) => true,
            Value::Function(_) => true,
            Value::Exception(_) => true,
            Value::Module(_) => true,
            Value::NativeFunction(_) => true,
            Value::BoundMethod(_, _) => true,
            Value::Regex(_) => true,
            Value::Match(_) => true,
            Value::Type(_) => true,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::None, Value::None) => true,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::List(a), Value::List(b)) => {
                let a_items = &a.borrow().items;
                let b_items = &b.borrow().items;
                a_items == b_items
            }
            (Value::Dict(a), Value::Dict(b)) => Rc::ptr_eq(a, b),
            (Value::Tuple(a), Value::Tuple(b)) => a.as_ref() == b.as_ref(),
            (
                Value::Slice {
                    start: s1,
                    stop: st1,
                    step: step1,
                },
                Value::Slice {
                    start: s2,
                    stop: st2,
                    step: step2,
                },
            ) => s1 == s2 && st1 == st2 && step1 == step2,
            (Value::Iterator(a), Value::Iterator(b)) => Rc::ptr_eq(a, b),
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::Exception(a), Value::Exception(b)) => {
                a.exception_type == b.exception_type && a.message == b.message
            }
            (Value::Module(a), Value::Module(b)) => Rc::ptr_eq(a, b),
            (Value::NativeFunction(a), Value::NativeFunction(b)) => std::ptr::eq(a, b),
            (Value::BoundMethod(a1, m1), Value::BoundMethod(a2, m2)) => a1 == a2 && m1 == m2,
            (Value::Regex(a), Value::Regex(b)) => Rc::ptr_eq(a, b),
            (Value::Match(a), Value::Match(b)) => Rc::ptr_eq(a, b),
            (Value::Type(a), Value::Type(b)) => a == b,
            _ => false,
        }
    }
}
