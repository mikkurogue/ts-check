use anyhow::Result;
use clap::Parser;

mod formatter;
mod parser;
mod suggestion;

#[derive(Parser)]
struct Cli {
    /// Optional file to read TSC error output from.
    input: Option<String>,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // Read input
    let mut buf = String::new();
    if let Some(input) = args.input {
        // Execute tsc and capture its output
        let output = std::process::Command::new("tsc")
            .arg(&input)
            .args([
                "--pretty",
                "false",
                "--diagnostics",
                "--extendedDiagnostics",
                "--noEmit",
                "--preserveWatchOutput",
                "false",
            ])
            .output()?;
        buf = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        // For now we can just return a rust error as we need an input file for now
        // we can later make sure we can check stdin for things like "no tsconfig" etc
        return Err(anyhow::anyhow!(
            "No input file provided. Please provide a TypeScript file as an argument."
        ));
    }

    let mut found_error = false;
    for line in buf.lines() {
        if let Some(parsed) = parser::parse(line) {
            found_error = true;
            println!("{}", formatter::fmt(&parsed));
        }
    }
    if !found_error {
        println!("No errors were emitted.");
    }

    Ok(())
}
