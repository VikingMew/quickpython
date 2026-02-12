use crate::bytecode::{ByteCode, Instruction};
use rustpython_parser::{Parse, ast};

pub struct Compiler;

impl Compiler {
    pub fn compile(source: &str) -> Result<ByteCode, String> {
        let parsed =
            ast::Expr::parse(source, "<eval>").map_err(|e| format!("Parse error: {}", e))?;

        let mut bytecode = Vec::new();
        Self::compile_expr(&parsed, &mut bytecode)?;
        Ok(bytecode)
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
