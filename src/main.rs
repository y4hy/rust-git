mod cli;
mod commands;
mod index;

use clap::Parser;
use std::io::Result;

use crate::commands::commit::commit;

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
            if args.args.is_empty() {
                eprintln!("Error: 'add' command requires at least one path argument.");
            } else {
                if let Some(flag) = args.args.first() {
                    if flag != "-m" {
                        eprintln!("Error: 'commit' command requires '-m' flag to point commit messsage.");
                    } else {
                        if let Some(message) = args.args.get(1) {
                            commands::commit::commit(message)?;
                        }
                    }
                }
            }
        }
        "diff" => {
            if args.args.is_empty() {
                commands::diff::diff()?;
            } else {
                eprintln!("Error: 'diff' command doesn't expect any arguments");
            }
        }
        _ => {
            eprintln!("Unknown util: {}", args.util);
        }
    }

    Ok(())
}
