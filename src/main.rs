mod cli;
mod commands;
mod index;

use clap::Parser;
use std::io::Result;

fn main() -> Result<()> {
    let args = cli::Args::parse();

    // Call each functionality in a separate branch
    match args.util.as_str() {
        "init" => {
            if let Some(path_str) = args.args.first() {
                commands::init::init(path_str)?;
                let abs_path = std::fs::canonicalize(path_str)?;
                println!("Initialized empty Git repository in {}", abs_path.display());
            } else {
                eprintln!("Error: 'init' command requires a path argument.")
            }
        }
        "add" => {
            if args.args.is_empty() {
                eprintln!("Error: 'add' command requires at least one path argument.");
            } else {
                for path_str in &args.args {
                    commands::add::add(path_str)?;
                    println!("Added '{}' to the index.", path_str);
                }
            }
        }
        "status" => {
            if args.args.is_empty() {
                commands::status::status()?;
            } else {
                eprintln!("Error: 'status' command doesn't expect any arguments");
            }
        }
        "commit" => {
            todo!("Implement commit function");
        }
        "diff" => {
            todo!("Implement diff function");
        }
        _ => {
            eprintln!("Unknown util: {}", args.util);
        }
    }

    Ok(())
}
