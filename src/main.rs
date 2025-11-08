// Main entry point for sharp-dpapi-rs
// Rust port of SharpDPAPI - A tool for DPAPI (Data Protection API) operations

use std::time::Instant;
use std::fs::File;

mod domain;
mod lib_dpapi;
mod commands;

use domain::{ArgumentParser, CommandCollection, Info};

fn file_execute(command_name: &str, parsed_args: &std::collections::HashMap<String, String>) {
    // Execute with stdout/err redirected to a file
    if let Some(file_path) = parsed_args.get("/consoleoutfile") {
        let file = File::create(file_path).expect("Failed to create output file");
        // Note: Redirecting stdout/stderr in Rust requires more complex handling
        // For now, we'll write directly to the file
        main_execute(command_name, parsed_args, Some(file));
    }
}

fn main_execute(
    command_name: &str,
    parsed_args: &std::collections::HashMap<String, String>,
    _output_file: Option<File>,
) {
    let start = Instant::now();

    Info::show_logo();

    match CommandCollection::execute_command(command_name, parsed_args) {
        Ok(found) => {
            if !found {
                Info::show_usage();
            }
        }
        Err(e) => {
            eprintln!("\r\n[!] Unhandled SharpDPAPI exception:\r\n");
            eprintln!("{}", e);
        }
    }

    let duration = start.elapsed();
    println!("\n\nSharpDPAPI completed in {:?}", duration);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match ArgumentParser::parse(&args[1..]) {
        Ok(parsed) => {
            let command_name = if !args.is_empty() && args.len() > 1 {
                args[1].to_lowercase()
            } else {
                String::new()
            };

            if parsed.contains_key("/consoleoutfile") {
                file_execute(&command_name, &parsed);
            } else {
                main_execute(&command_name, &parsed, None);
            }
        }
        Err(_) => {
            Info::show_logo();
            Info::show_usage();
        }
    }
}
