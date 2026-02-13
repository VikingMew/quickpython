use crate::value::{DictKey, ExceptionType, Module, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::rc::Rc;

pub fn create_module() -> Module {
    let mut module = Module::new("os");

    // 文件和目录操作
    module.add_function("listdir", os_listdir);
    module.add_function("mkdir", os_mkdir);
    module.add_function("makedirs", os_makedirs);
    module.add_function("remove", os_remove);
    module.add_function("rmdir", os_rmdir);
    module.add_function("rename", os_rename);
    module.add_function("getcwd", os_getcwd);
    module.add_function("chdir", os_chdir);

    // 环境变量
    module.add_function("getenv", os_getenv);

    // 添加 os.environ 字典
    let environ = create_environ_dict();
    module.attributes.insert("environ".to_string(), environ);

    // 添加 os.name
    #[cfg(unix)]
    let os_name = "posix";
    #[cfg(windows)]
    let os_name = "nt";
    module
        .attributes
        .insert("name".to_string(), Value::String(os_name.to_string()));

    // 添加 os.path 子模块
    let path_module = create_path_module();
    module.attributes.insert(
        "path".to_string(),
        Value::Module(Rc::new(RefCell::new(path_module))),
    );

    module
}

fn create_path_module() -> Module {
    let mut module = Module::new("os.path");

    module.add_function("exists", path_exists);
    module.add_function("isfile", path_isfile);
    module.add_function("isdir", path_isdir);
    module.add_function("join", path_join);
    module.add_function("basename", path_basename);
    module.add_function("dirname", path_dirname);
    module.add_function("abspath", path_abspath);

    module
}

fn create_environ_dict() -> Value {
    let mut map = HashMap::new();

    for (key, value) in env::vars() {
        map.insert(DictKey::String(key), Value::String(value));
    }

    Value::Dict(Rc::new(RefCell::new(map)))
}

// 文件和目录操作函数

fn os_listdir(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "listdir() missing required argument: 'path'",
        ));
    }

    let path = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "path must be a string"))?;

    let entries = fs::read_dir(&path).map_err(|e| {
        Value::error(
            ExceptionType::OSError,
            &format!("Failed to read directory '{}': {}", path, e),
        )
    })?;

    let mut items = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| {
            Value::error(
                ExceptionType::OSError,
                &format!("Failed to read entry: {}", e),
            )
        })?;

        let name = entry.file_name().to_string_lossy().to_string();
        items.push(Value::String(name));
    }

    Ok(Value::List(Rc::new(RefCell::new(
        crate::value::ListValue::with_items(items),
    ))))
}

fn os_mkdir(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "mkdir() missing required argument: 'path'",
        ));
    }

    let path = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "path must be a string"))?;

    fs::create_dir(&path).map_err(|e| {
        Value::error(
            ExceptionType::OSError,
            &format!("Failed to create directory '{}': {}", path, e),
        )
    })?;

    Ok(Value::None)
}

fn os_makedirs(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "makedirs() missing required argument: 'path'",
        ));
    }

    let path = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "path must be a string"))?;

    fs::create_dir_all(&path).map_err(|e| {
        Value::error(
            ExceptionType::OSError,
            &format!("Failed to create directories '{}': {}", path, e),
        )
    })?;

    Ok(Value::None)
}

fn os_remove(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "remove() missing required argument: 'path'",
        ));
    }

    let path = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "path must be a string"))?;

    fs::remove_file(&path).map_err(|e| {
        Value::error(
            ExceptionType::OSError,
            &format!("Failed to remove file '{}': {}", path, e),
        )
    })?;

    Ok(Value::None)
}

fn os_rmdir(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "rmdir() missing required argument: 'path'",
        ));
    }

    let path = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "path must be a string"))?;

    fs::remove_dir(&path).map_err(|e| {
        Value::error(
            ExceptionType::OSError,
            &format!("Failed to remove directory '{}': {}", path, e),
        )
    })?;

    Ok(Value::None)
}

fn os_rename(args: Vec<Value>) -> Result<Value, Value> {
    if args.len() < 2 {
        return Err(Value::error(
            ExceptionType::TypeError,
            "rename() requires 2 arguments: old and new",
        ));
    }

    let old = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "old path must be a string"))?;

    let new = args[1]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "new path must be a string"))?;

    fs::rename(&old, &new).map_err(|e| {
        Value::error(
            ExceptionType::OSError,
            &format!("Failed to rename '{}' to '{}': {}", old, new, e),
        )
    })?;

    Ok(Value::None)
}

fn os_getcwd(_args: Vec<Value>) -> Result<Value, Value> {
    let cwd = env::current_dir().map_err(|e| {
        Value::error(
            ExceptionType::OSError,
            &format!("Failed to get current directory: {}", e),
        )
    })?;

    Ok(Value::String(cwd.to_string_lossy().to_string()))
}

fn os_chdir(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "chdir() missing required argument: 'path'",
        ));
    }

    let path = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "path must be a string"))?;

    env::set_current_dir(&path).map_err(|e| {
        Value::error(
            ExceptionType::OSError,
            &format!("Failed to change directory to '{}': {}", path, e),
        )
    })?;

    Ok(Value::None)
}

// 环境变量操作

fn os_getenv(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "getenv() missing required argument: 'key'",
        ));
    }

    let key = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "key must be a string"))?;

    let default = if args.len() > 1 {
        args[1].clone()
    } else {
        Value::None
    };

    match env::var(&key) {
        Ok(value) => Ok(Value::String(value)),
        Err(_) => Ok(default),
    }
}

// os.path 子模块函数

fn path_exists(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "exists() missing required argument: 'path'",
        ));
    }

    let path = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "path must be a string"))?;

    Ok(Value::Bool(Path::new(&path).exists()))
}

fn path_isfile(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "isfile() missing required argument: 'path'",
        ));
    }

    let path = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "path must be a string"))?;

    Ok(Value::Bool(Path::new(&path).is_file()))
}

fn path_isdir(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "isdir() missing required argument: 'path'",
        ));
    }

    let path = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "path must be a string"))?;

    Ok(Value::Bool(Path::new(&path).is_dir()))
}

fn path_join(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Ok(Value::String(String::new()));
    }

    let mut path = std::path::PathBuf::new();

    for arg in args {
        let part = arg.as_string().ok_or_else(|| {
            Value::error(ExceptionType::TypeError, "all arguments must be strings")
        })?;
        path.push(part);
    }

    Ok(Value::String(path.to_string_lossy().to_string()))
}

fn path_basename(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "basename() missing required argument: 'path'",
        ));
    }

    let path = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "path must be a string"))?;

    let basename = Path::new(&path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    Ok(Value::String(basename.to_string()))
}

fn path_dirname(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "dirname() missing required argument: 'path'",
        ));
    }

    let path = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "path must be a string"))?;

    let dirname = Path::new(&path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("");

    Ok(Value::String(dirname.to_string()))
}

fn path_abspath(args: Vec<Value>) -> Result<Value, Value> {
    if args.is_empty() {
        return Err(Value::error(
            ExceptionType::TypeError,
            "abspath() missing required argument: 'path'",
        ));
    }

    let path = args[0]
        .as_string()
        .ok_or_else(|| Value::error(ExceptionType::TypeError, "path must be a string"))?;

    let abs_path = if Path::new(&path).is_absolute() {
        std::path::PathBuf::from(path)
    } else {
        env::current_dir()
            .map_err(|e| {
                Value::error(
                    ExceptionType::OSError,
                    &format!("Failed to get current directory: {}", e),
                )
            })?
            .join(path)
    };

    Ok(Value::String(abs_path.to_string_lossy().to_string()))
}
