use crate::value::Module;
use std::collections::HashMap;
use std::sync::Mutex;

/// 扩展模块注册函数类型
pub type ExtensionModuleFactory = fn() -> Module;

/// 扩展模块注册表（线程安全）
static EXTENSION_MODULES: Mutex<Option<HashMap<String, ExtensionModuleFactory>>> = Mutex::new(None);

/// 注册扩展模块
pub fn register_extension_module(name: &str, factory: ExtensionModuleFactory) {
    let mut modules = EXTENSION_MODULES.lock().unwrap();
    if modules.is_none() {
        *modules = Some(HashMap::new());
    }
    modules.as_mut().unwrap().insert(name.to_string(), factory);
}

/// 检查是否是扩展模块
pub fn is_extension_module(name: &str) -> bool {
    let modules = EXTENSION_MODULES.lock().unwrap();
    modules
        .as_ref()
        .map(|m| m.contains_key(name))
        .unwrap_or(false)
}

/// 获取扩展模块
pub fn get_extension_module(name: &str) -> Option<Module> {
    let modules = EXTENSION_MODULES.lock().unwrap();
    modules
        .as_ref()
        .and_then(|m| m.get(name))
        .map(|factory| factory())
}
