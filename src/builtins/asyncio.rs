use crate::value::{ExceptionType, Value};

fn asyncio_sleep(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() != 1 {
        return Err(Value::error(
            ExceptionType::TypeError,
            format!("sleep() takes exactly 1 argument ({} given)", args.len()),
        ));
    }

    let seconds = match &args[0] {
        Value::Int(i) => *i as f64,
        Value::Float(f) => *f,
        _ => {
            return Err(Value::error(
                ExceptionType::TypeError,
                "sleep() argument must be a number",
            ));
        }
    };

    if seconds < 0.0 {
        return Err(Value::error(
            ExceptionType::ValueError,
            "sleep() argument must be non-negative",
        ));
    }

    // 创建一个异步睡眠协程
    // 注意：这里返回一个特殊的标记，VM 会识别并异步执行
    Ok(Value::AsyncSleep(seconds))
}

pub fn create_asyncio_module() -> crate::value::Module {
    let mut module = crate::value::Module {
        name: "asyncio".to_string(),
        attributes: std::collections::HashMap::new(),
    };

    // asyncio.sleep(seconds) - 异步睡眠
    module
        .attributes
        .insert("sleep".to_string(), Value::NativeFunction(asyncio_sleep));

    module
}
