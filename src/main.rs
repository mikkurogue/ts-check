use anyhow::Result;
use clap::Parser;
use colored::*;

mod formatter;
mod message_parser;
mod parser;
mod suggestion;
mod token_utils;
mod tokenizer;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional file to read TSC error output from. If not provided, runs `tsc` in the current directory.
    input: Option<String>,

    /// Format a diagnostic from LSP instead of running tsc
    #[arg(long)]
    from_lsp: bool,

    /// Error code (e.g., TS2322) - required for --from-lsp
    #[arg(long, requires = "from_lsp")]
    code: Option<String>,

    /// Error message - required for --from-lsp
    #[arg(long, requires = "from_lsp")]
    message: Option<String>,

    /// Line number (1-indexed) - required for --from-lsp
    #[arg(long, requires = "from_lsp")]
    line: Option<usize>,

    /// Column number (1-indexed) - required for --from-lsp
    #[arg(long, requires = "from_lsp")]
    column: Option<usize>,

    /// File path - required for --from-lsp
    #[arg(long, requires = "from_lsp")]
    file: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.from_lsp {
        // LSP mode: format a single diagnostic
        format_lsp_diagnostic(
            cli.file.expect("--file required"),
            cli.line.expect("--line required"),
            cli.column.expect("--column required"),
            cli.code.expect("--code required"),
            cli.message.expect("--message required"),
        )?;
    } else {
        // Default behavior: parse tsc output
        parse_tsc_output(cli.input)?;
    }

    Ok(())
}

fn format_lsp_diagnostic(
    file: String,
    line: usize,
    column: usize,
    code: String,
    message: String,
) -> Result<()> {
    let parsed = parser::TsError {
        file,
        line,
        column,
        code: parser::CommonErrors::from_code(&code),
        message,
    };

    println!("{}", formatter::fmt(&parsed));
    Ok(())
}

fn parse_tsc_output(input: Option<String>) -> Result<()> {
    let buf: String;

    if let Some(input_file) = input {
        // Execute tsc on a specific file
        // Note: When tsc is run with a file argument, it doesn't use tsconfig.json
        // So we need to pass compiler options explicitly
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
                "--noUnusedLocals",
                "--noUnusedParameters",
                "--strict",
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
