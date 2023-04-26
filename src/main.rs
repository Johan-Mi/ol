#![forbid(unsafe_code, clippy::unwrap_used)]
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

use anyhow::{ensure, Context, Result};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    ensure!(args.len() < 2, "too many command line arguments");
    let source_path = args.next().context("no file provided")?;
    let source_code = std::fs::read_to_string(source_path)
        .context("failed to read source file")?;
    let program = parse::program(&source_code).context("syntax error")?;
    let mut vm = vm::VM::new();
    let class_ids = vm.load_program(program)?;
    vm.run(
        *class_ids
            .get("Main")
            .context("program has no `Main` class")?,
    )?;

    Ok(())
}
