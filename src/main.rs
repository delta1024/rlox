mod value {
    pub type Value = f64;
}

mod chunk;
mod vm;
use chunk::{Chunk, OpCode};
fn main() {
    vm::Vm::init_vm();
    let mut chunk = Chunk::new();

    let pos = chunk.constant(23.3 as f64);
    chunk.write(OpCode::Constant, 123);
    chunk.write(pos, 123);
    
    chunk.write(OpCode::Negate, 123);
    chunk.write(OpCode::Return, 123);

    match vm::Vm::interpret(chunk) {
        Err(err) => {
            eprintln!("{:?}", err);
        }
        Ok(()) => (),
    }
}
