use quickpython::{Compiler, Context, deserialize_bytecode, serialize_bytecode};

fn main() {
    // Step 1: Compile Python source to bytecode
    let source = r#"
x = 10 + 20
y = x * 2
print(y)
"#;
    let bytecode = Compiler::compile(source).unwrap();
    println!("Compiled {} instructions", bytecode.len());

    // Step 2: Serialize to bytes (could write to file / embed in binary)
    let bytes = serialize_bytecode(&bytecode).unwrap();
    println!("Serialized to {} bytes", bytes.len());

    // Step 3: Deserialize (could read from file / bundled resource)
    let restored = deserialize_bytecode(&bytes).unwrap();

    // Step 4: Execute the bytecode
    let mut ctx = Context::new();
    ctx.eval_bytecode(&restored).unwrap();

    // Step 5: Read results from context
    let y = ctx.get("y");
    println!("y = {:?}", y);
}
