use anyhow::Result;
use clap::Parser;
use colored::*;

mod formatter;
mod parser;
mod suggestion;
mod tokenizer;

#[derive(Parser)]
struct Cli {
    /// Optional file to read TSC error output from.
    input: Option<String>,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let buf: String;

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
        let output = std::process::Command::new("tsc")
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
    }

    if buf.is_empty() {
        println!("No output from tsc.");
        return Ok(());
    }

    let mut found_error = false;
    let mut counter: usize = 0;
    let lines: Vec<&str> = buf.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if let Some(mut parsed) = parser::parse(lines[i]) {
            found_error = true;
            counter += 1;

            // Collect continuation lines (indented lines following the error)
            let mut indented_line = i + 1;
            while indented_line < lines.len() && lines[indented_line].starts_with("  ") {
                parsed.message.push('\n');
                parsed.message.push_str(lines[indented_line].trim());
                indented_line += 1;
            }

            println!("{}", formatter::fmt(&parsed));
            i = indented_line;
        } else {
            i += 1;
        }
    }
    if !found_error {
        println!("No errors were emitted.");
    }

    let counter_str = counter.to_string();

    println!("\nTotal errors: {}", counter_str.red().bold());

    Ok(())
}
