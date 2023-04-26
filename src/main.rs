#![forbid(unsafe_code)]
#![warn(clippy::nursery, clippy::pedantic)]

mod expression;
mod method;
mod object;
mod parse;
mod program;
mod resolve;
mod typ;
mod value;
mod vm;

use std::process::ExitCode;

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}

fn real_main() -> Result<(), ()> {
    let mut args = std::env::args_os().skip(1);
    if args.len() > 1 {
        eprintln!("Error: too many command line arguments");
        return Err(());
    }
    let source_path = args
        .next()
        .ok_or_else(|| eprintln!("Error: no file provided"))?;
    let source_code = std::fs::read_to_string(source_path)
        .map_err(|err| eprintln!("Error: failed to read source file: {err}"))?;
    let (_, program) = parse::program(&source_code)
        .map_err(|err| eprintln!("Error: {err}"))?;
    let mut vm = vm::VM::new();
    let class_ids = vm.load_program(program);
    vm.run(
        *class_ids
            .get("Main")
            .ok_or_else(|| eprintln!("Error: program has no `Main` class"))?,
    );

    Ok(())
}
