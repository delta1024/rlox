mod value {
    pub type Value = f64;
}

mod chunk;
use chunk::{Chunk, OpCode};
fn main() {
    let mut chunk = Chunk::new();

    let pos = chunk.constant(123 as f64);
    chunk.write(OpCode::Constant, 123);
    chunk.write(pos, 123);
    chunk.write(OpCode::Return, 123);
    println!("{:?}", chunk);
}
