use byte_code::Chunk;
use compiler::{CompilerError, Parser};
use lexer::Lexer;
mod byte_code;
mod compiler;
mod frame;
mod lexer;

mod value {
    pub(crate) type Value = i64;
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = Parser::new(Lexer::new("1 + 2")).collect::<Result<Chunk, CompilerError>>();

    dbg!(parser);

    Ok(())
}
