use lexer::Lexer;

mod compiler;
mod lexer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lexer = Lexer::new("( ( (\n (");
    for token in lexer {
	dbg!(token);
    }
    Ok(())
}
