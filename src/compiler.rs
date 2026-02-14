use crate::bytecode::{ByteCode, Instruction};
use crate::value::ExceptionType;
use rustpython_parser::{Parse, ast};
use std::collections::HashMap;

struct LoopContext {
    start: usize,            // 循环开始位置（用于 continue）
    break_jumps: Vec<usize>, // 需要回填的 break 跳转位置
}

pub struct Compiler {
    local_vars: HashMap<String, usize>,
    local_count: usize,
    loop_stack: Vec<LoopContext>, // 循环栈
}

impl Compiler {
    pub fn compile(source: &str) -> Result<ByteCode, String> {
        let mut compiler = Compiler {
            local_vars: HashMap::new(),
            local_count: 0,
            loop_stack: Vec::new(),
        };
        let mut bytecode = Vec::new();

        // 尝试解析为模块（语句）
        match ast::Suite::parse(source, "<eval>") {
            Ok(stmts) => {
                if stmts.is_empty() {
                    return Ok(bytecode);
                }

                // 编译除最后一条之外的所有语句
                for stmt in &stmts[..stmts.len() - 1] {
                    compiler.compile_stmt(stmt, &mut bytecode)?;
                }

                // 最后一条语句特殊处理：如果是表达式语句，不添加 Pop
                let last_stmt = &stmts[stmts.len() - 1];
                match last_stmt {
                    ast::Stmt::Expr(expr_stmt) => {
                        compiler.compile_expr(&expr_stmt.value, &mut bytecode)?;
                    }
                    _ => {
                        compiler.compile_stmt(last_stmt, &mut bytecode)?;
                    }
                }

                Ok(bytecode)
            }
            Err(_) => {
                // 回退到表达式解析
                let parsed = ast::Expr::parse(source, "<eval>")
                    .map_err(|e| format!("Parse error: {}", e))?;
                compiler.compile_expr(&parsed, &mut bytecode)?;
                Ok(bytecode)
            }
        }
    }

    fn compile_stmt(&mut self, stmt: &ast::Stmt, bytecode: &mut ByteCode) -> Result<(), String> {
        match stmt {
            ast::Stmt::Assign(assign) => {
                // 编译右侧表达式
                self.compile_expr(&assign.value, bytecode)?;

                // 处理每个目标
                for target in &assign.targets {
                    match target {
                        ast::Expr::Name(name) => {
                            let var_name = name.id.to_string();
                            // 检查是否是局部变量
                            if let Some(&index) = self.local_vars.get(&var_name) {
                                bytecode.push(Instruction::SetLocal(index));
                            } else {
                                bytecode.push(Instruction::SetGlobal(var_name));
                            }
                        }
                        ast::Expr::Subscript(subscript) => {
                            // 下标赋值: obj[index] = value
                            // 编译对象
                            self.compile_expr(&subscript.value, bytecode)?;
                            // 编译索引
                            self.compile_expr(&subscript.slice, bytecode)?;
                            // 值已经在栈顶
                            bytecode.push(Instruction::SetItem);
                        }
                        _ => return Err("Unsupported assignment target".to_string()),
                    }
                }
                // 赋值语句不应该留值在栈上
                bytecode.push(Instruction::Pop);
                Ok(())
            }
            ast::Stmt::FunctionDef(func_def) => {
                let func_name = func_def.name.to_string();
                let params: Vec<String> = func_def
                    .args
                    .args
                    .iter()
                    .map(|arg| arg.def.arg.to_string())
                    .collect();

                // 创建新的编译器上下文用于函数体
                let mut func_compiler = Compiler {
                    local_vars: HashMap::new(),
                    local_count: 0,
                    loop_stack: Vec::new(),
                };

                // 将参数注册为局部变量
                for (i, param) in params.iter().enumerate() {
                    func_compiler.local_vars.insert(param.clone(), i);
                    func_compiler.local_count = i + 1;
                }

                // 编译函数体
                let mut func_bytecode = Vec::new();
                for stmt in &func_def.body {
                    func_compiler.compile_stmt(stmt, &mut func_bytecode)?;
                }

                // 如果最后没有 return，添加 return None
                if !matches!(func_bytecode.last(), Some(Instruction::Return)) {
                    func_bytecode.push(Instruction::PushNone);
                    func_bytecode.push(Instruction::Return);
                }

                let code_len = func_bytecode.len();
                bytecode.push(Instruction::MakeFunction {
                    name: func_name,
                    params,
                    code_len,
                });
                bytecode.extend(func_bytecode);

                // 函数定义不产生值，添加 None 到栈
                bytecode.push(Instruction::PushNone);
                Ok(())
            }
            ast::Stmt::Return(ret) => {
                if let Some(value) = &ret.value {
                    self.compile_expr(value, bytecode)?;
                } else {
                    bytecode.push(Instruction::PushNone);
                }
                bytecode.push(Instruction::Return);
                Ok(())
            }
            ast::Stmt::If(if_stmt) => {
                // 编译条件
                self.compile_expr(&if_stmt.test, bytecode)?;

                // JumpIfFalse 到 else 分支或结束
                let jump_to_else = bytecode.len();
                bytecode.push(Instruction::JumpIfFalse(0)); // 占位符

                // 编译 if 分支
                for stmt in &if_stmt.body {
                    self.compile_stmt(stmt, bytecode)?;
                }

                // 如果有 else 分支，需要跳过它
                let jump_to_end = if !if_stmt.orelse.is_empty() {
                    let pos = bytecode.len();
                    bytecode.push(Instruction::Jump(0)); // 占位符
                    Some(pos)
                } else {
                    None
                };

                // 回填 JumpIfFalse 的目标
                let else_start = bytecode.len();
                bytecode[jump_to_else] = Instruction::JumpIfFalse(else_start);

                // 编译 else 分支
                if !if_stmt.orelse.is_empty() {
                    for stmt in &if_stmt.orelse {
                        self.compile_stmt(stmt, bytecode)?;
                    }
                }

                // 回填跳转到结束的位置
                if let Some(jump_pos) = jump_to_end {
                    let end_pos = bytecode.len();
                    bytecode[jump_pos] = Instruction::Jump(end_pos);
                }

                Ok(())
            }
            ast::Stmt::While(while_stmt) => {
                // 循环开始位置
                let loop_start = bytecode.len();

                // 进入循环上下文
                self.loop_stack.push(LoopContext {
                    start: loop_start,
                    break_jumps: Vec::new(),
                });

                // 编译条件
                self.compile_expr(&while_stmt.test, bytecode)?;

                // JumpIfFalse 到循环结束
                let jump_to_end = bytecode.len();
                bytecode.push(Instruction::JumpIfFalse(0)); // 占位符

                // 编译循环体
                for stmt in &while_stmt.body {
                    self.compile_stmt(stmt, bytecode)?;
                }

                // 跳回循环开始
                bytecode.push(Instruction::Jump(loop_start));

                // 回填跳转到结束的位置
                let end_pos = bytecode.len();
                bytecode[jump_to_end] = Instruction::JumpIfFalse(end_pos);

                // 循环结束，回填所有 break 跳转
                let loop_ctx = self.loop_stack.pop().unwrap();
                for jump_pos in loop_ctx.break_jumps {
                    bytecode[jump_pos] = Instruction::Jump(end_pos);
                }

                Ok(())
            }
            ast::Stmt::For(for_stmt) => {
                // for target in iter: body
                // 编译迭代对象
                self.compile_expr(&for_stmt.iter, bytecode)?;

                // 获取迭代器
                bytecode.push(Instruction::GetIter);

                // 循环开始位置
                let loop_start = bytecode.len();

                // 进入循环上下文
                self.loop_stack.push(LoopContext {
                    start: loop_start,
                    break_jumps: Vec::new(),
                });

                // ForIter: 获取下一个元素，如果结束则跳转
                // ForIter 会将迭代器保留在栈上，并将下一个值压入栈
                let jump_to_end = bytecode.len();
                bytecode.push(Instruction::ForIter(0)); // 占位符

                // 将迭代值赋给目标变量
                // 栈状态: [iterator, value]
                match &*for_stmt.target {
                    ast::Expr::Name(name) => {
                        let var_name = name.id.to_string();
                        if let Some(&index) = self.local_vars.get(&var_name) {
                            bytecode.push(Instruction::SetLocal(index));
                        } else {
                            bytecode.push(Instruction::SetGlobal(var_name));
                        }
                        bytecode.push(Instruction::Pop); // 清理赋值后的值
                    }
                    _ => {
                        return Err(
                            "Only simple variable names are supported in for loops".to_string()
                        );
                    }
                }

                // 编译循环体
                // 栈状态: [iterator]
                for stmt in &for_stmt.body {
                    self.compile_stmt(stmt, bytecode)?;
                }

                // 跳回循环开始
                bytecode.push(Instruction::Jump(loop_start));

                // 回填跳转到结束的位置
                let end_pos = bytecode.len();
                bytecode[jump_to_end] = Instruction::ForIter(end_pos);

                // 循环结束，回填所有 break 跳转
                let loop_ctx = self.loop_stack.pop().unwrap();
                for jump_pos in loop_ctx.break_jumps {
                    bytecode[jump_pos] = Instruction::Jump(end_pos);
                }

                // 清理迭代器
                bytecode.push(Instruction::Pop);

                Ok(())
            }
            ast::Stmt::Expr(expr_stmt) => {
                self.compile_expr(&expr_stmt.value, bytecode)?;
                bytecode.push(Instruction::Pop);
                Ok(())
            }
            ast::Stmt::Raise(raise) => {
                use crate::value::ExceptionType;

                if let Some(exc) = &raise.exc {
                    // 检查是否是简单的异常调用
                    if let ast::Expr::Call(call) = &**exc
                        && let ast::Expr::Name(name) = &*call.func
                    {
                        let exc_name = name.id.to_string();
                        let exc_type = match exc_name.as_str() {
                            "ValueError" => ExceptionType::ValueError,
                            "TypeError" => ExceptionType::TypeError,
                            "IndexError" => ExceptionType::IndexError,
                            "KeyError" => ExceptionType::KeyError,
                            "ZeroDivisionError" => ExceptionType::ZeroDivisionError,
                            "RuntimeError" => ExceptionType::RuntimeError,
                            "IteratorError" => ExceptionType::IteratorError,
                            "Exception" => ExceptionType::Exception,
                            _ => return Err(format!("Unknown exception type: {}", exc_name)),
                        };

                        // 编译消息参数
                        if call.args.len() != 1 {
                            return Err("Exception requires exactly one argument".to_string());
                        }
                        self.compile_expr(&call.args[0], bytecode)?;

                        // 创建异常对象
                        bytecode.push(Instruction::MakeException(exc_type));
                        bytecode.push(Instruction::Raise);
                        return Ok(());
                    }

                    // 其他情况：编译表达式，应该得到一个异常对象
                    self.compile_expr(exc, bytecode)?;
                    bytecode.push(Instruction::Raise);
                } else {
                    // bare raise（重新抛出当前异常）
                    return Err("bare raise not supported yet".to_string());
                }
                Ok(())
            }
            ast::Stmt::Try(try_stmt) => {
                if !try_stmt.finalbody.is_empty() {
                    // Has finally block - wrap everything in SetupFinally
                    self.compile_try_except_finally(try_stmt, bytecode)?;
                } else {
                    // No finally block - use simple try-except
                    self.compile_try_except(try_stmt, bytecode)?;
                }
                Ok(())
            }
            ast::Stmt::Pass(_) => {
                // Pass statement does nothing
                Ok(())
            }
            ast::Stmt::Break(_) => {
                if let Some(loop_ctx) = self.loop_stack.last_mut() {
                    // 添加占位跳转，稍后回填
                    let jump_pos = bytecode.len();
                    bytecode.push(Instruction::Jump(0)); // 占位符
                    loop_ctx.break_jumps.push(jump_pos);
                    Ok(())
                } else {
                    Err("'break' outside loop".to_string())
                }
            }
            ast::Stmt::Continue(_) => {
                if let Some(loop_ctx) = self.loop_stack.last() {
                    // 直接跳转到循环开始
                    bytecode.push(Instruction::Jump(loop_ctx.start));
                    Ok(())
                } else {
                    Err("'continue' outside loop".to_string())
                }
            }
            ast::Stmt::Import(import) => {
                // import module [as alias]
                for alias in &import.names {
                    let module_name = alias.name.to_string();
                    let as_name = alias
                        .asname
                        .as_ref()
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| module_name.clone());

                    // 导入模块
                    bytecode.push(Instruction::Import(module_name));
                    // 绑定到变量
                    bytecode.push(Instruction::SetGlobal(as_name));
                    bytecode.push(Instruction::Pop);
                }
                Ok(())
            }
            ast::Stmt::ImportFrom(import_from) => {
                // from module import name1, name2 [as alias]
                let module_name = import_from
                    .module
                    .as_ref()
                    .ok_or("from import without module name")?
                    .to_string();

                // 导入模块
                bytecode.push(Instruction::Import(module_name.clone()));

                // 对每个导入的名称
                for alias in &import_from.names {
                    let name = alias.name.to_string();
                    let as_name = alias
                        .asname
                        .as_ref()
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| name.clone());

                    // 复制模块到栈顶
                    bytecode.push(Instruction::Dup);
                    // 获取属性
                    bytecode.push(Instruction::GetAttr(name));
                    // 绑定到变量
                    bytecode.push(Instruction::SetGlobal(as_name));
                    bytecode.push(Instruction::Pop);
                }

                // 弹出模块
                bytecode.push(Instruction::Pop);

                Ok(())
            }
            _ => Err(format!("Unsupported statement: {:?}", stmt)),
        }
    }

    fn compile_expr(&mut self, expr: &ast::Expr, bytecode: &mut ByteCode) -> Result<(), String> {
        match expr {
            ast::Expr::Constant(constant) => {
                match &constant.value {
                    ast::Constant::Int(int_val) => {
                        let value: i32 = int_val
                            .try_into()
                            .map_err(|_| "Integer overflow".to_string())?;
                        bytecode.push(Instruction::PushInt(value));
                    }
                    ast::Constant::Float(f) => {
                        bytecode.push(Instruction::PushFloat(*f));
                    }
                    ast::Constant::Bool(b) => {
                        bytecode.push(Instruction::PushBool(*b));
                    }
                    ast::Constant::None => {
                        bytecode.push(Instruction::PushNone);
                    }
                    ast::Constant::Str(s) => {
                        bytecode.push(Instruction::PushString(s.to_string()));
                    }
                    _ => return Err("Unsupported constant type".to_string()),
                }
                Ok(())
            }
            ast::Expr::Name(name) => {
                let var_name = name.id.to_string();
                // 检查是否是局部变量
                if let Some(&index) = self.local_vars.get(&var_name) {
                    bytecode.push(Instruction::GetLocal(index));
                } else {
                    bytecode.push(Instruction::GetGlobal(var_name));
                }
                Ok(())
            }
            ast::Expr::BinOp(binop) => {
                self.compile_expr(&binop.left, bytecode)?;
                self.compile_expr(&binop.right, bytecode)?;

                match binop.op {
                    ast::Operator::Add => bytecode.push(Instruction::Add),
                    ast::Operator::Sub => bytecode.push(Instruction::Sub),
                    ast::Operator::Mult => bytecode.push(Instruction::Mul),
                    ast::Operator::Div => bytecode.push(Instruction::Div),
                    ast::Operator::Mod => bytecode.push(Instruction::Mod),
                    _ => return Err(format!("Unsupported operator: {:?}", binop.op)),
                }
                Ok(())
            }
            ast::Expr::Compare(compare) => {
                if compare.ops.len() != 1 || compare.comparators.len() != 1 {
                    return Err("Only simple comparisons are supported".to_string());
                }

                self.compile_expr(&compare.left, bytecode)?;
                self.compile_expr(&compare.comparators[0], bytecode)?;

                match &compare.ops[0] {
                    ast::CmpOp::Eq => bytecode.push(Instruction::Eq),
                    ast::CmpOp::NotEq => bytecode.push(Instruction::Ne),
                    ast::CmpOp::Lt => bytecode.push(Instruction::Lt),
                    ast::CmpOp::LtE => bytecode.push(Instruction::Le),
                    ast::CmpOp::Gt => bytecode.push(Instruction::Gt),
                    ast::CmpOp::GtE => bytecode.push(Instruction::Ge),
                    _ => return Err(format!("Unsupported comparison: {:?}", compare.ops[0])),
                }
                Ok(())
            }
            ast::Expr::Call(call) => {
                // 检查是否是内置函数
                if let ast::Expr::Name(name) = &*call.func {
                    match name.id.as_str() {
                        "print" => {
                            // 编译参数
                            let arg_count = call.args.len();
                            for arg in &call.args {
                                self.compile_expr(arg, bytecode)?;
                            }
                            bytecode.push(Instruction::Print(arg_count));
                            return Ok(());
                        }
                        "int" => {
                            if call.args.len() != 1 {
                                return Err("int() takes exactly one argument".to_string());
                            }
                            self.compile_expr(&call.args[0], bytecode)?;
                            bytecode.push(Instruction::Int);
                            return Ok(());
                        }
                        "float" => {
                            if call.args.len() != 1 {
                                return Err("float() takes exactly one argument".to_string());
                            }
                            self.compile_expr(&call.args[0], bytecode)?;
                            bytecode.push(Instruction::Float);
                            return Ok(());
                        }
                        "len" => {
                            if call.args.len() != 1 {
                                return Err("len() takes exactly one argument".to_string());
                            }
                            self.compile_expr(&call.args[0], bytecode)?;
                            bytecode.push(Instruction::Len);
                            return Ok(());
                        }
                        "range" => {
                            // range(stop) or range(start, stop) or range(start, stop, step)
                            if call.args.is_empty() || call.args.len() > 3 {
                                return Err("range() takes 1 to 3 arguments".to_string());
                            }
                            // 先压入参数数量
                            bytecode.push(Instruction::PushInt(call.args.len() as i32));
                            // 再压入参数
                            for arg in &call.args {
                                self.compile_expr(arg, bytecode)?;
                            }
                            bytecode.push(Instruction::Range);
                            return Ok(());
                        }
                        _ => {}
                    }
                }

                // 检查是否是方法调用
                if let ast::Expr::Attribute(attr) = &*call.func {
                    let method_name = attr.attr.to_string();
                    // 编译对象
                    self.compile_expr(&attr.value, bytecode)?;
                    // 编译参数
                    for arg in &call.args {
                        self.compile_expr(arg, bytecode)?;
                    }
                    bytecode.push(Instruction::CallMethod(method_name, call.args.len()));
                    return Ok(());
                }

                // 编译函数表达式
                self.compile_expr(&call.func, bytecode)?;

                // 编译参数
                for arg in &call.args {
                    self.compile_expr(arg, bytecode)?;
                }

                bytecode.push(Instruction::Call(call.args.len()));
                Ok(())
            }
            ast::Expr::List(list) => {
                // 编译列表元素
                for elt in &list.elts {
                    self.compile_expr(elt, bytecode)?;
                }
                bytecode.push(Instruction::BuildList(list.elts.len()));
                Ok(())
            }
            ast::Expr::Dict(dict) => {
                // 编译字典键值对
                for i in 0..dict.keys.len() {
                    if let Some(key) = &dict.keys[i] {
                        self.compile_expr(key, bytecode)?;
                    } else {
                        return Err("Dictionary unpacking not supported".to_string());
                    }
                    self.compile_expr(&dict.values[i], bytecode)?;
                }
                bytecode.push(Instruction::BuildDict(dict.keys.len()));
                Ok(())
            }
            ast::Expr::Subscript(subscript) => {
                // 编译对象
                self.compile_expr(&subscript.value, bytecode)?;
                // 编译索引
                self.compile_expr(&subscript.slice, bytecode)?;
                bytecode.push(Instruction::GetItem);
                Ok(())
            }
            ast::Expr::Attribute(attribute) => {
                // obj.attr
                // 编译对象
                self.compile_expr(&attribute.value, bytecode)?;
                // 获取属性
                let attr_name = attribute.attr.to_string();
                bytecode.push(Instruction::GetAttr(attr_name));
                Ok(())
            }
            ast::Expr::UnaryOp(unary) => {
                match unary.op {
                    ast::UnaryOp::USub => {
                        // 一元负号：编译操作数，然后取反
                        self.compile_expr(&unary.operand, bytecode)?;
                        bytecode.push(Instruction::Negate);
                    }
                    ast::UnaryOp::UAdd => {
                        // 一元正号：直接编译操作数（无操作）
                        self.compile_expr(&unary.operand, bytecode)?;
                    }
                    _ => return Err(format!("Unsupported unary operator: {:?}", unary.op)),
                }
                Ok(())
            }
            _ => Err(format!("Unsupported expression: {:?}", expr)),
        }
    }

    fn compile_try_except(
        &mut self,
        try_stmt: &ast::StmtTry,
        bytecode: &mut ByteCode,
    ) -> Result<(), String> {
        // 设置 try 块
        let handler_offset_placeholder = bytecode.len();
        bytecode.push(Instruction::SetupTry(0)); // 占位符

        // 编译 try 块
        for stmt in &try_stmt.body {
            self.compile_stmt(stmt, bytecode)?;
        }

        // 正常结束，移除 try 块
        bytecode.push(Instruction::PopTry);
        let end_offset_placeholder = bytecode.len();
        bytecode.push(Instruction::Jump(0)); // 跳过 except 块

        // except 块开始位置
        let except_start = bytecode.len();
        bytecode[handler_offset_placeholder] = Instruction::SetupTry(except_start);

        let mut handler_end_placeholders = Vec::new();

        // 编译每个 except 子句
        for handler in &try_stmt.handlers {
            match handler {
                ast::ExceptHandler::ExceptHandler(eh) => {
                    if let Some(exc_type) = &eh.type_ {
                        // 复制异常对象
                        bytecode.push(Instruction::Dup);

                        // 压入期望的异常类型
                        let expected_type = self.parse_exception_type(exc_type)?;
                        bytecode.push(Instruction::PushInt(expected_type.as_i32()));

                        // 检查类型匹配（支持继承）
                        bytecode.push(Instruction::MatchException);

                        // 如果不匹配，跳到下一个 handler
                        let next_handler_placeholder = bytecode.len();
                        bytecode.push(Instruction::JumpIfFalse(0));

                        // 类型匹配，弹出比较结果
                        bytecode.push(Instruction::Pop);

                        // 绑定到变量（如果有）
                        if let Some(name) = &eh.name {
                            let var_name = name.to_string();
                            if let Some(&index) = self.local_vars.get(&var_name) {
                                bytecode.push(Instruction::SetLocal(index));
                            } else {
                                bytecode.push(Instruction::SetGlobal(var_name));
                            }
                            bytecode.push(Instruction::Pop);
                        } else {
                            // 没有绑定变量，弹出异常对象
                            bytecode.push(Instruction::Pop);
                        }

                        // 编译 except 块体
                        for stmt in &eh.body {
                            self.compile_stmt(stmt, bytecode)?;
                        }

                        // 跳到 try-except 结束
                        let handler_end_placeholder = bytecode.len();
                        bytecode.push(Instruction::Jump(0));
                        handler_end_placeholders.push(handler_end_placeholder);

                        // 回填"跳到下一个 handler"的地址
                        let next_handler_pos = bytecode.len();
                        bytecode[next_handler_placeholder] =
                            Instruction::JumpIfFalse(next_handler_pos);

                        // 弹出比较结果（类型不匹配）
                        bytecode.push(Instruction::Pop);
                    } else {
                        // 捕获所有异常
                        if let Some(name) = &eh.name {
                            let var_name = name.to_string();
                            if let Some(&index) = self.local_vars.get(&var_name) {
                                bytecode.push(Instruction::SetLocal(index));
                            } else {
                                bytecode.push(Instruction::SetGlobal(var_name));
                            }
                            bytecode.push(Instruction::Pop);
                        } else {
                            bytecode.push(Instruction::Pop);
                        }

                        // 编译 except 块体
                        for stmt in &eh.body {
                            self.compile_stmt(stmt, bytecode)?;
                        }

                        let handler_end_placeholder = bytecode.len();
                        bytecode.push(Instruction::Jump(0));
                        handler_end_placeholders.push(handler_end_placeholder);
                    }
                }
            }
        }

        // 如果所有 except 都不匹配，重新抛出
        bytecode.push(Instruction::Raise);

        // 回填跳转地址
        let after_except = bytecode.len();
        bytecode[end_offset_placeholder] = Instruction::Jump(after_except);
        for placeholder in handler_end_placeholders {
            bytecode[placeholder] = Instruction::Jump(after_except);
        }

        Ok(())
    }

    fn compile_try_except_finally(
        &mut self,
        try_stmt: &ast::StmtTry,
        bytecode: &mut ByteCode,
    ) -> Result<(), String> {
        // SetupFinally - wraps the entire try-except block
        let finally_offset_placeholder = bytecode.len();
        bytecode.push(Instruction::SetupFinally(0)); // Placeholder

        // If there are except handlers, compile the try-except block
        if !try_stmt.handlers.is_empty() {
            self.compile_try_except(try_stmt, bytecode)?;
        } else {
            // No except handlers, just compile the try body
            for stmt in &try_stmt.body {
                self.compile_stmt(stmt, bytecode)?;
            }
        }

        // Normal completion - pop finally block and push None
        bytecode.push(Instruction::PopFinally);
        bytecode.push(Instruction::PushNone);

        // Jump to finally block
        let jump_to_finally_placeholder = bytecode.len();
        bytecode.push(Instruction::Jump(0)); // Placeholder

        // Finally block starts here
        let finally_start = bytecode.len();
        bytecode[finally_offset_placeholder] = Instruction::SetupFinally(finally_start);

        // Compile finally body
        for stmt in &try_stmt.finalbody {
            self.compile_stmt(stmt, bytecode)?;
        }

        // EndFinally - check if we need to re-raise
        bytecode.push(Instruction::EndFinally);

        // Backfill jump to finally
        bytecode[jump_to_finally_placeholder] = Instruction::Jump(finally_start);

        Ok(())
    }

    fn parse_exception_type(&self, expr: &ast::Expr) -> Result<ExceptionType, String> {
        if let ast::Expr::Name(name) = expr {
            match name.id.to_string().as_str() {
                "ValueError" => Ok(ExceptionType::ValueError),
                "TypeError" => Ok(ExceptionType::TypeError),
                "IndexError" => Ok(ExceptionType::IndexError),
                "KeyError" => Ok(ExceptionType::KeyError),
                "ZeroDivisionError" => Ok(ExceptionType::ZeroDivisionError),
                "RuntimeError" => Ok(ExceptionType::RuntimeError),
                "IteratorError" => Ok(ExceptionType::IteratorError),
                "Exception" => Ok(ExceptionType::Exception),
                _ => Err(format!("Unknown exception type: {}", name.id)),
            }
        } else {
            Err("Exception type must be a name".to_string())
        }
    }
}
