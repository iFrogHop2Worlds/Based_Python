mod parser;
mod ast;
mod codegen;

use std::{fs, io::{self, Write}, path::PathBuf};
use std::process::Command;
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

    /// Run the generated Python code immediately
    #[arg(short, long)]
    run: bool,

    /// Python interpreter to use (default: python)
    #[arg(long, default_value = "python")]
    python_interpreter: String,
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

            if args.run {
                println!("Running generated Python code...");
                println!("~~~ Output ~~~");
                let output = Command::new(&args.python_interpreter)
                    .arg(&output_path)
                    .output()
                    .map_err(|e| format!("Failed to execute Python interpreter '{}': {}", args.python_interpreter, e))?;

                io::stdout().write_all(&output.stdout)?;
                io::stderr().write_all(&output.stderr)?;

                if !output.status.success() {
                    return Err(format!("Python execution failed with exit code: {}",
                                       output.status.code().unwrap_or(-1)).into());
                }
            }
        }
        None => {
            if args.run {
                println!("Running generated Python code...");
                println!("~~~ Output ~~~");
                let mut child = Command::new(&args.python_interpreter)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to start Python interpreter '{}': {}", args.python_interpreter, e))?;

                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(python_code.as_bytes())?;
                }

                let output = child.wait_with_output()?;
                io::stdout().write_all(&output.stdout)?;
                io::stderr().write_all(&output.stderr)?;

                if !output.status.success() {
                    return Err(format!("Python execution failed with exit code: {}",
                                       output.status.code().unwrap_or(-1)).into());
                }
            } else {
                io::stdout().write_all(python_code.as_bytes())?;
            }
        }
    }

    Ok(())
}
