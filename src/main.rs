mod chunk;
mod compiler;
mod error;
mod memory;
mod objects;
mod scanner;
mod value;
mod vm;
cfg_if::cfg_if! {
    if #[cfg(feature = "trace_execution")] {
        // Do nothing
    } else if #[cfg(feature = "print_code")] {
        // Do nothing
    } else {
        use memory::GarbageCollector;
        #[global_allocator]
        static GLOBAL: GarbageCollector = GarbageCollector;
    }
}
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
            return Ok(());
        }
        if let Err(_) = vm::Vm::interpret(&input) {
            continue;
        }
    }
}

fn run_file(path: &str) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    if let Err(err) = vm::Vm::interpret(&file_contents) {
        let code = match err {
            vm::Error::Compile(_) => 65,
            vm::Error::Runtime(_) => 70,
        };
        exit(code);
    }
    Ok(())
}
fn main() -> io::Result<()> {
    vm::Vm::init_vm();
    let (argv, argc) = {
        let args = env::args().collect::<Vec<String>>();
        let n = args.len();
        (args, n)
    };
    if argc == 1 {
        repl()?;
    } else if argc == 2 {
        run_file(&argv[1])?;
    } else {
        eprintln!("Usage: rlox [path]");
        exit(64);
    }
    vm::Vm::free_vm();
    Ok(())
}
