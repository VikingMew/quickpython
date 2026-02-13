pub mod json;
pub mod os;
pub mod re;

use crate::value::Module;

/// 内置模块列表（编译时确定）
const BUILTIN_MODULES: &[&str] = &["json", "os", "re"];

/// 检查是否是内置模块
pub fn is_builtin_module(name: &str) -> bool {
    BUILTIN_MODULES.contains(&name)
}

/// 获取内置模块
pub fn get_builtin_module(name: &str) -> Module {
    match name {
        "json" => json::create_module(),
        "os" => os::create_module(),
        "re" => re::create_module(),
        _ => panic!("Unknown builtin module: {}", name),
    }
}
