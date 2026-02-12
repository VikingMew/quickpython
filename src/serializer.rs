use crate::bytecode::{ByteCode, Instruction};
use std::io::Write;

const MAGIC: &[u8; 4] = b"QPY\0";
const VERSION: u32 = 1;

pub fn serialize_bytecode(bytecode: &ByteCode) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();

    // 写入魔数
    buffer.write_all(MAGIC).map_err(|e| e.to_string())?;

    // 写入版本号
    buffer
        .write_all(&VERSION.to_le_bytes())
        .map_err(|e| e.to_string())?;

    // 写入指令数量
    let count = bytecode.len() as u32;
    buffer
        .write_all(&count.to_le_bytes())
        .map_err(|e| e.to_string())?;

    // 写入每条指令
    for instruction in bytecode {
        serialize_instruction(&mut buffer, instruction)?;
    }

    Ok(buffer)
}

pub fn deserialize_bytecode(data: &[u8]) -> Result<ByteCode, String> {
    if data.len() < 12 {
        return Err("Invalid bytecode: too short".to_string());
    }

    // 验证魔数
    if &data[0..4] != MAGIC {
        return Err("Invalid bytecode: wrong magic number".to_string());
    }

    // 读取版本号
    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if version != VERSION {
        return Err(format!("Unsupported bytecode version: {}", version));
    }

    // 读取指令数量
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
        Instruction::Add => buffer.push(0x10),
        Instruction::Sub => buffer.push(0x11),
        Instruction::Mul => buffer.push(0x12),
        Instruction::Div => buffer.push(0x13),
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
        Instruction::Pop => buffer.push(0x30),
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
        0x10 => Ok((Instruction::Add, 1)),
        0x11 => Ok((Instruction::Sub, 1)),
        0x12 => Ok((Instruction::Mul, 1)),
        0x13 => Ok((Instruction::Div, 1)),
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
        0x30 => Ok((Instruction::Pop, 1)),
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
    fn test_version() {
        let bytecode = vec![Instruction::Add];
        let serialized = serialize_bytecode(&bytecode).unwrap();

        let version =
            u32::from_le_bytes([serialized[4], serialized[5], serialized[6], serialized[7]]);
        assert_eq!(version, 1);
    }

    #[test]
    fn test_invalid_magic() {
        let data = b"XXX\0\x01\x00\x00\x00\x00\x00\x00\x00";
        let result = deserialize_bytecode(data);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("magic"));
    }

    #[test]
    fn test_too_short() {
        let data = b"QPY\0\x01";
        let result = deserialize_bytecode(data);

        assert!(result.is_err());
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
}
