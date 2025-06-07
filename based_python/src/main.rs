mod parser;
mod ast;
mod codegen;

use std::{fs, io::{self, Write}, path::PathBuf};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input Bython file
    #[arg(short, long)]
    input: PathBuf,

    /// Output Python file
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input_code = fs::read_to_string(&args.input)
        .map_err(|e| format!("Could not read input file {}: {}", args.input.display(), e))?;

    let program_ast = parser::parse_bython_code(&input_code)?;

    let python_code = codegen::generate_python_code(&program_ast);

    match args.output {
        Some(output_path) => {
            fs::write(&output_path, python_code.as_bytes())
                .map_err(|e| format!("Could not write to output file {}: {}", output_path.display(), e))?;
            println!("Successfully processed '{}' to '{}'", args.input.display(), output_path.display());
        }
        None => {
            io::stdout().write_all(python_code.as_bytes())?;
        }
    }

    Ok(())
}