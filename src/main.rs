mod value {
    pub type Value = f64;
}
mod chunk;
mod compiler;
mod error;
mod scanner;
#[allow(dead_code)]
mod vm;
use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    process::exit,
};
fn repl() -> io::Result<()> {
    loop {
        let mut string = String::new();
        print!("> ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut string)?;
        if let Err(err) = vm::Vm::interpret(&string) {
            eprintln!("{}", err);
        }
    }
}

fn run_file(path: &str) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    match vm::Vm::interpret(&file_contents) {
        Err(err) if err == vm::Error::Compile => exit(65),
        Err(err) if err == vm::Error::Runtime => exit(70),
        _ => Ok(()),
    }
}
fn main() -> io::Result<()> {
    vm::Vm::init_vm();
    let (argv, argc) = {
        let args = env::args().collect::<Vec<String>>();
        let n = args.len();
        (args, n)
    };
    if argc == 1 {
        repl()
    } else if argc == 2 {
        run_file(&argv[1])
    } else {
        eprintln!("Usage: rlox [path]");
        exit(64);
    }
}
