use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;



mod formatter;
mod lsp;
mod message_parser;
mod parser;
mod suggestion;
mod token_utils;
mod tokenizer;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Optional file to read TSC error output from. If not provided, runs `tsc` in the current directory.
    input: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Starts the TypeScript Analyzer LSP proxy.
    Lsp {
        /// Specify which underlying LSP server to use.
        #[arg(long, value_enum, default_value_t = LspServer::TsServer)]
        server: LspServer,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum LspServer {
    Vtsls,
    TsServer,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // If a subcommand is present, run it. Otherwise, run the default tsc parser.
    if let Some(command) = cli.command {
        match command {
            Commands::Lsp { server } => {
                let ts_lsp = match server {
                    LspServer::Vtsls => lsp::proxy::TsLsp::Vtsls,
                    LspServer::TsServer => lsp::proxy::TsLsp::TsServer,
                };
                let proxy = lsp::proxy::LspProxy::new(ts_lsp);
                // The start_as_proxy method is annotated with #[tokio::main],
                // so it will run its own async runtime.
                proxy.start_as_proxy();
            }
        }
        return Ok(());
    }

    // Default behavior: parse tsc output
    parse_tsc_output(cli.input)?;

    Ok(())
}

fn parse_tsc_output(input: Option<String>) -> Result<()> {
    let buf: String;

    if let Some(input_file) = input {
        // Execute tsc on a specific file
        let output = std::process::Command::new("tsc")
            .arg(&input_file)
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
        // Execute tsc in the current directory
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
