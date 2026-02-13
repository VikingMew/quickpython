use quickpython::Context;
use quickpython_llm;
use std::env;
use std::process;

fn main() {
    // 注册 llm 扩展模块
    quickpython_llm::init();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("QuickPython Demo with LLM extension");
        println!("Usage: quickpython-demo <python_file>");
        println!("Examples:");
        println!("  quickpython-demo examples/llm_chat.py");
        println!("  quickpython-demo test/test_llm_basic.py");
        process::exit(1);
    }

    let file_path = &args[1];

    // 读取 Python 文件
    let source = match std::fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            process::exit(1);
        }
    };

    // 执行 Python 代码
    let mut context = Context::new();
    if let Err(e) = context.eval(&source) {
        eprintln!("Runtime error: {}", e);
        process::exit(1);
    }
}
