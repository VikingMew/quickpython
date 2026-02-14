use crate::bytecode::{ByteCode, Instruction};
use std::io::Write;

const MAGIC: &[u8; 4] = b"QPY\0";
const VERSION: u32 = 5;

pub fn serialize_bytecode(bytecode: &ByteCode) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();

    buffer.write_all(MAGIC).map_err(|e| e.to_string())?;
    buffer
        .write_all(&VERSION.to_le_bytes())
        .map_err(|e| e.to_string())?;

    let count = bytecode.len() as u32;
    buffer
        .write_all(&count.to_le_bytes())
        .map_err(|e| e.to_string())?;

    for instruction in bytecode {
        serialize_instruction(&mut buffer, instruction)?;
    }

    Ok(buffer)
}

pub fn deserialize_bytecode(data: &[u8]) -> Result<ByteCode, String> {
    if data.len() < 12 {
        return Err("Invalid bytecode: too short".to_string());
    }

    if &data[0..4] != MAGIC {
        return Err("Invalid bytecode: wrong magic number".to_string());
    }

    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if version > VERSION {
        return Err(format!("Unsupported bytecode version: {}", version));
    }

    let count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;

    let mut bytecode = Vec::with_capacity(count);
    let mut offset = 12;

    for _ in 0..count {
        let (instruction, bytes_read) = deserialize_instruction(&data[offset..])?;
        bytecode.push(instruction);
        offset += bytes_read;
    }

    Ok(bytecode)
}

fn serialize_instruction(buffer: &mut Vec<u8>, instruction: &Instruction) -> Result<(), String> {
    match instruction {
        Instruction::PushInt(value) => {
            buffer.push(0x01);
            buffer
                .write_all(&value.to_le_bytes())
                .map_err(|e| e.to_string())?;
        }
        Instruction::PushFloat(value) => {
            buffer.push(0x06);
            buffer
                .write_all(&value.to_le_bytes())
                .map_err(|e| e.to_string())?;
        }
        Instruction::PushBool(b) => {
            buffer.push(0x02);
            buffer.push(if *b { 1 } else { 0 });
        }
        Instruction::PushNone => buffer.push(0x03),
        Instruction::PushString(s) => {
            buffer.push(0x05);
            let bytes = s.as_bytes();
            buffer
                .write_all(&(bytes.len() as u32).to_le_bytes())
                .map_err(|e| e.to_string())?;
            buffer.write_all(bytes).map_err(|e| e.to_string())?;
        }
        Instruction::Pop => buffer.push(0x04),
        Instruction::Add => buffer.push(0x10),
        Instruction::Sub => buffer.push(0x11),
        Instruction::Mul => buffer.push(0x12),
        Instruction::Div => buffer.push(0x13),
        Instruction::Mod => buffer.push(0x1B),
        Instruction::Negate => buffer.push(0x1A),
        Instruction::Eq => buffer.push(0x14),
        Instruction::Ne => buffer.push(0x15),
        Instruction::Lt => buffer.push(0x16),
        Instruction::Le => buffer.push(0x17),
        Instruction::Gt => buffer.push(0x18),
        Instruction::Ge => buffer.push(0x19),
        Instruction::GetGlobal(name) => {
            buffer.push(0x20);
            let bytes = name.as_bytes();
            buffer
                .write_all(&(bytes.len() as u16).to_le_bytes())
                .map_err(|e| e.to_string())?;
            buffer.write_all(bytes).map_err(|e| e.to_string())?;
        }
        Instruction::SetGlobal(name) => {
            buffer.push(0x21);
            let bytes = name.as_bytes();
            buffer
                .write_all(&(bytes.len() as u16).to_le_bytes())
                .map_err(|e| e.to_string())?;
            buffer.write_all(bytes).map_err(|e| e.to_string())?;
        }
        Instruction::GetLocal(index) => {
            buffer.push(0x22);
            buffer
                .write_all(&(*index as u16).to_le_bytes())
                .map_err(|e| e.to_string())?;
        }
        Instruction::SetLocal(index) => {
            buffer.push(0x23);
            buffer
                .write_all(&(*index as u16).to_le_bytes())
                .map_err(|e| e.to_string())?;
        }
        Instruction::Jump(offset) => {
            buffer.push(0x30);
            buffer
                .write_all(&(*offset as u32).to_le_bytes())
                .map_err(|e| e.to_string())?;
        }
        Instruction::JumpIfFalse(offset) => {
            buffer.push(0x31);
            buffer
                .write_all(&(*offset as u32).to_le_bytes())
                .map_err(|e| e.to_string())?;
        }
        Instruction::MakeFunction { .. } | Instruction::Call(_) | Instruction::Return => {
            return Err("Function instructions cannot be serialized yet".to_string());
        }
        Instruction::Print => buffer.push(0x40),
        Instruction::Int => buffer.push(0x41),
        Instruction::Float => buffer.push(0x42),
        Instruction::Len => buffer.push(0x43),
        Instruction::BuildList(_)
        | Instruction::BuildDict(_)
        | Instruction::GetItem
        | Instruction::SetItem
        | Instruction::CallMethod(_, _) => {
            return Err("List/Dict instructions cannot be serialized yet".to_string());
        }
        Instruction::Range => buffer.push(0x50),
        Instruction::GetIter => buffer.push(0x51),
        Instruction::ForIter(offset) => {
            buffer.push(0x52);
            buffer
                .write_all(&(*offset as u32).to_le_bytes())
                .map_err(|e| e.to_string())?;
        }
        Instruction::Break => {
            return Err(
                "Break instructions cannot be serialized (should be compiled to Jump)".to_string(),
            );
        }
        Instruction::Continue => {
            return Err(
                "Continue instructions cannot be serialized (should be compiled to Jump)"
                    .to_string(),
            );
        }
        Instruction::MakeException(_) | Instruction::Raise => {
            return Err("Exception instructions cannot be serialized yet".to_string());
        }
        Instruction::SetupTry(_)
        | Instruction::PopTry
        | Instruction::GetExceptionType
        | Instruction::Dup
        | Instruction::SetupFinally(_)
        | Instruction::PopFinally
        | Instruction::EndFinally => {
            return Err("Try-except-finally instructions cannot be serialized yet".to_string());
        }
        Instruction::Import(_) | Instruction::GetAttr(_) => {
            return Err("Import instructions cannot be serialized yet".to_string());
        }
    }
    Ok(())
}

fn deserialize_instruction(data: &[u8]) -> Result<(Instruction, usize), String> {
    if data.is_empty() {
        return Err("Unexpected end of bytecode".to_string());
    }

    let opcode = data[0];
    match opcode {
        0x01 => {
            if data.len() < 5 {
                return Err("Invalid PushInt instruction".to_string());
            }
            let value = i32::from_le_bytes([data[1], data[2], data[3], data[4]]);
            Ok((Instruction::PushInt(value), 5))
        }
        0x02 => {
            if data.len() < 2 {
                return Err("Invalid PushBool instruction".to_string());
            }
            Ok((Instruction::PushBool(data[1] != 0), 2))
        }
        0x03 => Ok((Instruction::PushNone, 1)),
        0x04 => Ok((Instruction::Pop, 1)),
        0x05 => {
            if data.len() < 5 {
                return Err("Invalid PushString instruction".to_string());
            }
            let len = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
            if data.len() < 5 + len {
                return Err("Invalid PushString instruction: string too short".to_string());
            }
            let s = String::from_utf8(data[5..5 + len].to_vec())
                .map_err(|_| "Invalid UTF-8 in string".to_string())?;
            Ok((Instruction::PushString(s), 5 + len))
        }
        0x06 => {
            if data.len() < 9 {
                return Err("Invalid PushFloat instruction".to_string());
            }
            let value = f64::from_le_bytes([
                data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            ]);
            Ok((Instruction::PushFloat(value), 9))
        }
        0x10 => Ok((Instruction::Add, 1)),
        0x11 => Ok((Instruction::Sub, 1)),
        0x12 => Ok((Instruction::Mul, 1)),
        0x13 => Ok((Instruction::Div, 1)),
        0x1B => Ok((Instruction::Mod, 1)),
        0x1A => Ok((Instruction::Negate, 1)),
        0x14 => Ok((Instruction::Eq, 1)),
        0x15 => Ok((Instruction::Ne, 1)),
        0x16 => Ok((Instruction::Lt, 1)),
        0x17 => Ok((Instruction::Le, 1)),
        0x18 => Ok((Instruction::Gt, 1)),
        0x19 => Ok((Instruction::Ge, 1)),
        0x20 => {
            if data.len() < 3 {
                return Err("Invalid GetGlobal instruction".to_string());
            }
            let len = u16::from_le_bytes([data[1], data[2]]) as usize;
            if data.len() < 3 + len {
                return Err("Invalid GetGlobal instruction: string too short".to_string());
            }
            let name = String::from_utf8(data[3..3 + len].to_vec())
                .map_err(|_| "Invalid UTF-8 in variable name".to_string())?;
            Ok((Instruction::GetGlobal(name), 3 + len))
        }
        0x21 => {
            if data.len() < 3 {
                return Err("Invalid SetGlobal instruction".to_string());
            }
            let len = u16::from_le_bytes([data[1], data[2]]) as usize;
            if data.len() < 3 + len {
                return Err("Invalid SetGlobal instruction: string too short".to_string());
            }
            let name = String::from_utf8(data[3..3 + len].to_vec())
                .map_err(|_| "Invalid UTF-8 in variable name".to_string())?;
            Ok((Instruction::SetGlobal(name), 3 + len))
        }
        0x22 => {
            if data.len() < 3 {
                return Err("Invalid GetLocal instruction".to_string());
            }
            let index = u16::from_le_bytes([data[1], data[2]]) as usize;
            Ok((Instruction::GetLocal(index), 3))
        }
        0x23 => {
            if data.len() < 3 {
                return Err("Invalid SetLocal instruction".to_string());
            }
            let index = u16::from_le_bytes([data[1], data[2]]) as usize;
            Ok((Instruction::SetLocal(index), 3))
        }
        0x30 => {
            if data.len() < 5 {
                return Err("Invalid Jump instruction".to_string());
            }
            let offset = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
            Ok((Instruction::Jump(offset), 5))
        }
        0x31 => {
            if data.len() < 5 {
                return Err("Invalid JumpIfFalse instruction".to_string());
            }
            let offset = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
            Ok((Instruction::JumpIfFalse(offset), 5))
        }
        0x40 => Ok((Instruction::Print, 1)),
        0x41 => Ok((Instruction::Int, 1)),
        0x42 => Ok((Instruction::Float, 1)),
        0x43 => Ok((Instruction::Len, 1)),
        0x50 => Ok((Instruction::Range, 1)),
        0x51 => Ok((Instruction::GetIter, 1)),
        0x52 => {
            if data.len() < 5 {
                return Err("Invalid ForIter instruction".to_string());
            }
            let offset = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
            Ok((Instruction::ForIter(offset), 5))
        }
        _ => Err(format!("Unknown opcode: 0x{:02x}", opcode)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let bytecode = vec![
            Instruction::PushInt(42),
            Instruction::PushInt(10),
            Instruction::Add,
        ];

        let serialized = serialize_bytecode(&bytecode).unwrap();
        let deserialized = deserialize_bytecode(&serialized).unwrap();

        assert_eq!(bytecode, deserialized);
    }

    #[test]
    fn test_magic_number() {
        let bytecode = vec![Instruction::PushInt(1)];
        let serialized = serialize_bytecode(&bytecode).unwrap();

        assert_eq!(&serialized[0..4], b"QPY\0");
    }

    #[test]
    fn test_variable_instructions() {
        let bytecode = vec![
            Instruction::PushInt(42),
            Instruction::SetGlobal("x".to_string()),
            Instruction::GetGlobal("x".to_string()),
        ];

        let serialized = serialize_bytecode(&bytecode).unwrap();
        let deserialized = deserialize_bytecode(&serialized).unwrap();

        assert_eq!(bytecode, deserialized);
    }

    #[test]
    fn test_bool_and_comparison() {
        let bytecode = vec![
            Instruction::PushInt(5),
            Instruction::PushInt(3),
            Instruction::Lt,
        ];

        let serialized = serialize_bytecode(&bytecode).unwrap();
        let deserialized = deserialize_bytecode(&serialized).unwrap();

        assert_eq!(bytecode, deserialized);
    }
}
