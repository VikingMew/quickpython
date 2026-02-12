/// Value type for QuickPython runtime
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i32),
}

impl Value {
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Value::Int(i) => Some(*i),
        }
    }
}
