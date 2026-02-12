use crate::bytecode::{ByteCode, Instruction};
use rustpython_parser::{Parse, ast};

pub struct Compiler;

impl Compiler {
    pub fn compile(source: &str) -> Result<ByteCode, String> {
        let mut bytecode = Vec::new();

        // 尝试解析为模块（语句）
        match ast::Suite::parse(source, "<eval>") {
            Ok(stmts) => {
                if stmts.is_empty() {
                    return Ok(bytecode);
                }

                // 编译除最后一条之外的所有语句
                for stmt in &stmts[..stmts.len() - 1] {
                    Self::compile_stmt(stmt, &mut bytecode)?;
                }

                // 最后一条语句特殊处理：如果是表达式语句，不添加 Pop
                let last_stmt = &stmts[stmts.len() - 1];
                match last_stmt {
                    ast::Stmt::Expr(expr_stmt) => {
                        Self::compile_expr(&expr_stmt.value, &mut bytecode)?;
                    }
                    _ => {
                        Self::compile_stmt(last_stmt, &mut bytecode)?;
                    }
                }

                return Ok(bytecode);
            }
            Err(_) => {
                // 回退到表达式解析
                let parsed = ast::Expr::parse(source, "<eval>")
                    .map_err(|e| format!("Parse error: {}", e))?;
                Self::compile_expr(&parsed, &mut bytecode)?;
                Ok(bytecode)
            }
        }
    }

    fn compile_stmt(stmt: &ast::Stmt, bytecode: &mut ByteCode) -> Result<(), String> {
        match stmt {
            ast::Stmt::Assign(assign) => {
                // 编译右侧表达式
                Self::compile_expr(&assign.value, bytecode)?;

                // 处理每个目标（支持多重赋值）
                for target in &assign.targets {
                    match target {
                        ast::Expr::Name(name) => {
                            bytecode.push(Instruction::SetGlobal(name.id.to_string()));
                        }
                        _ => return Err("Only simple variable assignment is supported".to_string()),
                    }
                }
                Ok(())
            }
            ast::Stmt::Expr(expr_stmt) => {
                Self::compile_expr(&expr_stmt.value, bytecode)?;
                bytecode.push(Instruction::Pop);
                Ok(())
            }
            _ => Err(format!("Unsupported statement: {:?}", stmt)),
        }
    }

    fn compile_expr(expr: &ast::Expr, bytecode: &mut ByteCode) -> Result<(), String> {
        match expr {
            ast::Expr::Constant(constant) => {
                if let ast::Constant::Int(int_val) = &constant.value {
                    let value: i32 = int_val
                        .try_into()
                        .map_err(|_| "Integer overflow".to_string())?;
                    bytecode.push(Instruction::PushInt(value));
                    Ok(())
                } else {
                    Err("Only integers are supported".to_string())
                }
            }
            ast::Expr::Name(name) => {
                bytecode.push(Instruction::GetGlobal(name.id.to_string()));
                Ok(())
            }
            ast::Expr::BinOp(binop) => {
                Self::compile_expr(&binop.left, bytecode)?;
                Self::compile_expr(&binop.right, bytecode)?;

                match binop.op {
                    ast::Operator::Add => bytecode.push(Instruction::Add),
                    ast::Operator::Sub => bytecode.push(Instruction::Sub),
                    ast::Operator::Mult => bytecode.push(Instruction::Mul),
                    ast::Operator::Div => bytecode.push(Instruction::Div),
                    _ => return Err(format!("Unsupported operator: {:?}", binop.op)),
                }
                Ok(())
            }
            _ => Err(format!("Unsupported expression: {:?}", expr)),
        }
    }
}
