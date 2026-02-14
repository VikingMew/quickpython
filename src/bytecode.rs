/// Bytecode instructions for the VM
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // 栈操作
    PushInt(i32),
    PushFloat(f64),
    PushBool(bool),
    PushNone,
    PushString(String),
    Pop,

    // 算术运算
    Add,
    Sub,
    Mul,
    Div,
    Mod,    // 模运算
    Negate, // 一元负号

    // 比较运算
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // 变量
    GetGlobal(String),
    SetGlobal(String),
    GetLocal(usize),
    SetLocal(usize),

    // 控制流
    Jump(usize),
    JumpIfFalse(usize),

    // 函数
    MakeFunction {
        name: String,
        params: Vec<String>,
        code_len: usize,
    },
    Call(usize), // 参数数量
    Return,

    // 内置函数
    Print,
    Int,   // int() 类型转换
    Float, // float() 类型转换
    Len,   // len() 函数
    Range, // range() 函数，参数数量在栈上

    // 列表和字典
    BuildList(usize),          // 从栈顶取 n 个元素构建列表
    BuildDict(usize),          // 从栈顶取 n*2 个元素构建字典（键值对）
    GetItem,                   // 索引访问 list[i] 或 dict[key]
    SetItem,                   // 索引赋值 list[i] = x 或 dict[key] = x
    CallMethod(String, usize), // 方法调用 obj.method(args)

    // 迭代器和 for 循环
    GetIter,        // 获取对象的迭代器
    ForIter(usize), // 迭代下一个元素，如果结束则跳转到指定位置
    Break,          // 跳出循环
    Continue,       // 继续下一次循环迭代

    // 异常处理
    Raise,                                      // 抛出异常，栈顶是异常对象
    MakeException(crate::value::ExceptionType), // 创建异常，栈顶是消息字符串
    SetupTry(usize),                            // 设置 try 块，参数是 except 块的位置
    PopTry,                                     // 移除 try 块（正常结束时）
    GetExceptionType,                           // 获取异常类型（用于类型检查）
    MatchException,                             // 检查异常类型是否匹配（支持继承）
    Dup,                                        // 复制栈顶元素
    SetupFinally(usize),                        // 设置 finally 块，参数是 finally 块的位置
    PopFinally,                                 // 移除 finally 块
    EndFinally,                                 // 结束 finally 块，检查是否需要重新抛出异常

    // 模块导入
    Import(String),  // 导入模块，参数是模块名
    GetAttr(String), // 获取对象属性，参数是属性名
}

pub type ByteCode = Vec<Instruction>;
