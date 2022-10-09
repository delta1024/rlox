mod chunk;
mod compiler;
mod error;
mod scanner;
mod value;
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
        let mut input = String::new();
        print!("> ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        if input.is_empty() {
            print!("\n");
            exit(0);
        }
        if let Err(err) = vm::Vm::interpret(&input) {
            eprintln!("{}", err);
        }
    }
}

fn run_file(path: &str) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    match vm::Vm::interpret(&file_contents) {
        Err(err) => {
            exit(match err {
                vm::Error::Compile => 65,
                vm::Error::Runtime(err) => {
                    eprintln!("{}", err);
                    70
                }
            });
        }
        Ok(()) => Ok(()),
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
