use crate::bytecode::{ByteCode, Instruction};
use rustpython_parser::{Parse, ast};
use std::collections::HashMap;

pub struct Compiler {
    local_vars: HashMap<String, usize>,
    local_count: usize,
}

impl Compiler {
    pub fn compile(source: &str) -> Result<ByteCode, String> {
        let mut compiler = Compiler {
            local_vars: HashMap::new(),
            local_count: 0,
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

                return Ok(bytecode);
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
                        _ => return Err("Only simple variable assignment is supported".to_string()),
                    }
                }
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

                Ok(())
            }
            ast::Stmt::Expr(expr_stmt) => {
                self.compile_expr(&expr_stmt.value, bytecode)?;
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
                    ast::Constant::Bool(b) => {
                        bytecode.push(Instruction::PushBool(*b));
                    }
                    ast::Constant::None => {
                        bytecode.push(Instruction::PushNone);
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
                // 编译函数表达式
                self.compile_expr(&call.func, bytecode)?;

                // 编译参数
                for arg in &call.args {
                    self.compile_expr(arg, bytecode)?;
                }

                bytecode.push(Instruction::Call(call.args.len()));
                Ok(())
            }
            _ => Err(format!("Unsupported expression: {:?}", expr)),
        }
    }
}
