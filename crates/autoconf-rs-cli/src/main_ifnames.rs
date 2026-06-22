//! ifnames binary — extract #if/#ifdef names from source files.
//! Receipt family: AC.CLI.IFNAMES.* Status: Phase 3

use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let input_path = args.get(1).map(|s| s.as_str()).unwrap_or("configure.ac");

    if input_path == "--help" || input_path == "-h" {
        println!("ifnames-rs {}", env!("CARGO_PKG_VERSION"));
        println!("Extract #if/#ifdef/#ifndef names from source files");
        println!("Usage: ifnames [file]");
        println!("  -h, --help    Show this help");
        println!("  --version     Show version");
        return ExitCode::SUCCESS;
    }
    if input_path == "--version" {
        println!("ifnames-rs {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    match fs::read_to_string(input_path) {
        Ok(content) => {
            println!("/* ifnames: scanning {} */", input_path);
            for line in content.lines() {
                if line.trim().starts_with("#if")
                    || line.trim().starts_with("#ifdef")
                    || line.trim().starts_with("#ifndef")
                {
                    println!("{}", line.trim());
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("ifnames: {}", e);
            ExitCode::from(2)
        }
    }
}
