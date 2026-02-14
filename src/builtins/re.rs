use crate::value::{ExceptionType, MatchObject, Module, Value};
use regex::Regex;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_module() -> Module {
    let mut module = Module::new("re");

    // 基础匹配函数
    module.add_function("match", re_match);
    module.add_function("search", re_search);
    module.add_function("findall", re_findall);

    // 替换函数
    module.add_function("sub", re_sub);
    module.add_function("subn", re_subn);

    // 分割函数
    module.add_function("split", re_split);

    // 编译函数
    module.add_function("compile", re_compile);

    module
}

fn re_match(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() < 2 {
        return Err(Value::error(
            ExceptionType::TypeError,
            "match() requires 2 arguments: pattern and string",
        ));
    }

    let pattern = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "pattern must be a string"))?;

    let text = args[1]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "string must be a string"))?;

    let regex = Regex::new(pattern).map_err(|e| {
        Value::error(
            ExceptionType::ValueError,
            format!("Invalid regex pattern: {}", e),
        )
    })?;

    // match 只匹配字符串开头
    if let Some(captures) = regex.captures(text) {
        let m = captures.get(0).unwrap();
        if m.start() == 0 {
            let groups: Vec<Option<String>> = captures
                .iter()
                .map(|g| g.map(|m| m.as_str().to_string()))
                .collect();

            let match_obj = MatchObject::new(text.to_string(), m.start(), m.end(), groups);

            return Ok(Value::Match(Rc::new(match_obj)));
        }
    }

    Ok(Value::None)
}

fn re_search(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() < 2 {
        return Err(Value::error(
            ExceptionType::TypeError,
            "search() requires 2 arguments: pattern and string",
        ));
    }

    let pattern = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "pattern must be a string"))?;

    let text = args[1]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "string must be a string"))?;

    let regex = Regex::new(pattern).map_err(|e| {
        Value::error(
            ExceptionType::ValueError,
            format!("Invalid regex pattern: {}", e),
        )
    })?;

    if let Some(captures) = regex.captures(text) {
        let m = captures.get(0).unwrap();
        let groups: Vec<Option<String>> = captures
            .iter()
            .map(|g| g.map(|m| m.as_str().to_string()))
            .collect();

        let match_obj = MatchObject::new(text.to_string(), m.start(), m.end(), groups);

        return Ok(Value::Match(Rc::new(match_obj)));
    }

    Ok(Value::None)
}

fn re_findall(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() < 2 {
        return Err(Value::error(
            ExceptionType::TypeError,
            "findall() requires 2 arguments: pattern and string",
        ));
    }

    let pattern = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "pattern must be a string"))?;

    let text = args[1]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "string must be a string"))?;

    let regex = Regex::new(pattern).map_err(|e| {
        Value::error(
            ExceptionType::ValueError,
            format!("Invalid regex pattern: {}", e),
        )
    })?;

    let mut matches = Vec::new();
    for captures in regex.captures_iter(text) {
        let m = captures.get(0).unwrap();
        matches.push(Value::String(m.as_str().to_string()));
    }

    Ok(Value::List(Rc::new(RefCell::new(
        crate::value::ListValue::with_items(matches),
    ))))
}

fn re_sub(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() < 3 {
        return Err(Value::error(
            ExceptionType::TypeError,
            "sub() requires 3 arguments: pattern, repl, and string",
        ));
    }

    let pattern = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "pattern must be a string"))?;

    let repl = args[1]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "repl must be a string"))?;

    let text = args[2]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "string must be a string"))?;

    let regex = Regex::new(pattern).map_err(|e| {
        Value::error(
            ExceptionType::ValueError,
            format!("Invalid regex pattern: {}", e),
        )
    })?;

    let result = regex.replace_all(text, repl).to_string();

    Ok(Value::String(result))
}

fn re_subn(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() < 3 {
        return Err(Value::error(
            ExceptionType::TypeError,
            "subn() requires 3 arguments: pattern, repl, and string",
        ));
    }

    let pattern = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "pattern must be a string"))?;

    let repl = args[1]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "repl must be a string"))?;

    let text = args[2]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "string must be a string"))?;

    let regex = Regex::new(pattern).map_err(|e| {
        Value::error(
            ExceptionType::ValueError,
            format!("Invalid regex pattern: {}", e),
        )
    })?;

    let count = regex.find_iter(text).count();
    let result = regex.replace_all(text, repl).to_string();

    // 返回 (result, count) 元组
    Ok(Value::List(Rc::new(RefCell::new(
        crate::value::ListValue::with_items(vec![Value::String(result), Value::Int(count as i32)]),
    ))))
}

fn re_split(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() < 2 {
        return Err(Value::error(
            ExceptionType::TypeError,
            "split() requires 2 arguments: pattern and string",
        ));
    }

    let pattern = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "pattern must be a string"))?;

    let text = args[1]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "string must be a string"))?;

    let regex = Regex::new(pattern).map_err(|e| {
        Value::error(
            ExceptionType::ValueError,
            format!("Invalid regex pattern: {}", e),
        )
    })?;

    let parts: Vec<Value> = regex
        .split(text)
        .map(|s| Value::String(s.to_string()))
        .collect();

    Ok(Value::List(Rc::new(RefCell::new(
        crate::value::ListValue::with_items(parts),
    ))))
}

fn re_compile(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "compile() missing required argument: 'pattern'",
        ));
    }

    let pattern = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "pattern must be a string"))?;

    let regex = Regex::new(pattern).map_err(|e| {
        Value::error(
            ExceptionType::ValueError,
            format!("Invalid regex pattern: {}", e),
        )
    })?;

    Ok(Value::Regex(Rc::new(regex)))
}
