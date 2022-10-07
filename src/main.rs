mod value {
    pub type Value = f64;
}

mod chunk;
mod vm;
use chunk::{Chunk, OpCode};
fn main() {
    vm::Vm::init_vm();
    let mut chunk = Chunk::new();

    let mut constant = chunk.constant(1.2);
    chunk.write(OpCode::Constant, 123);
    chunk.write(constant, 123);
    
    constant = chunk.constant(3.4);
    chunk.write(OpCode::Constant, 123);
    chunk.write(constant, 123);
    
    chunk.write(OpCode::Add, 123);

    constant = chunk.constant(5.6);
    chunk.write(OpCode::Constant, 123);
    chunk.write(constant, 123);

    chunk.write(OpCode::Divide, 123);

    chunk.write(OpCode::Negate, 123);
    chunk.write(OpCode::Return, 123);
    println!("{:?}", chunk);
    match vm::Vm::interpret(chunk) {
        Err(err) => {
            eprintln!("{:?}", err);
        }
        Ok(()) => (),
    }
}
