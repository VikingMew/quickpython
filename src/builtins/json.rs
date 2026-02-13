use crate::value::{DictKey, ExceptionType, Module, Value};
use serde_json;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn create_module() -> Module {
    let mut module = Module::new("json");
    module.add_function("loads", json_loads);
    module.add_function("dumps", json_dumps);
    module
}

fn json_loads(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "loads() missing required argument: 's'",
        ));
    }

    let json_str = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "argument must be a string"))?;

    let json_value: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| Value::error(ExceptionType::ValueError, &format!("Invalid JSON: {}", e)))?;

    json_to_value(&json_value)
}

fn json_dumps(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "dumps() missing required argument: 'obj'",
        ));
    }

    let json_value = value_to_json(&args[0])?;
    let json_str = serde_json::to_string(&json_value).map_err(|e| {
        Value::error(
            ExceptionType::RuntimeError,
            &format!("Failed to serialize: {}", e),
        )
    })?;

    Ok(Value::String(json_str))
}

fn json_to_value(json: &serde_json::Value) -> Result<Value, Value> {
    match json {
        serde_json::Value::Null => Ok(Value::None),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i as i32))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(Value::error(
                    ExceptionType::ValueError,
                    "Number out of range",
                ))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let mut items = Vec::new();
            for item in arr {
                items.push(json_to_value(item)?);
            }
            Ok(Value::List(Rc::new(RefCell::new(
                crate::value::ListValue::with_items(items),
            ))))
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (key, val) in obj {
                let py_val = json_to_value(val)?;
                map.insert(DictKey::String(key.clone()), py_val);
            }
            Ok(Value::Dict(Rc::new(RefCell::new(map))))
        }
    }
}

fn value_to_json(value: &Value) -> Result<serde_json::Value, Value> {
    match value {
        Value::None => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Int(i) => Ok(serde_json::Value::Number((*i).into())),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .ok_or_else(|| Value::error(ExceptionType::ValueError, "Float value is not finite")),
        Value::String(s) => Ok(serde_json::Value::String(s.clone())),
        Value::List(list) => {
            let mut arr = Vec::new();
            for item in &list.borrow().items {
                arr.push(value_to_json(item)?);
            }
            Ok(serde_json::Value::Array(arr))
        }
        Value::Dict(dict) => {
            let mut obj = serde_json::Map::new();
            for (key, val) in dict.borrow().iter() {
                let key_str = match key {
                    DictKey::String(s) => s.clone(),
                    DictKey::Int(i) => i.to_string(),
                };
                obj.insert(key_str, value_to_json(val)?);
            }
            Ok(serde_json::Value::Object(obj))
        }
        _ => Err(Value::error(
            ExceptionType::TypeError,
            "Object is not JSON serializable",
        )),
    }
}
