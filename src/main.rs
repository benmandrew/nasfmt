use clap::Parser;
use similar::TextDiff;
use std::process;

mod ast;
mod formatter;
mod parser;

#[derive(Parser)]
#[command(name = "nasmlint", about = "NASM x86-64 assembly formatter")]
struct Cli {
    file: std::path::PathBuf,
    #[arg(long, help = "Check formatting; exit 1 with diff if changes needed")]
    check: bool,
}

fn main() {
    let cli = Cli::parse();

    let source = std::fs::read_to_string(&cli.file).unwrap_or_else(|e| {
        eprintln!("nasmlint: {}: {}", cli.file.display(), e);
        process::exit(2);
    });

    let lines = parser::parse(&source);
    let formatted = formatter::format(&lines);

    if cli.check {
        if formatted == source {
            process::exit(0);
        }
        print!(
            "{}",
            unified_diff(&source, &formatted, &cli.file.to_string_lossy())
        );
        process::exit(1);
    } else {
        print!("{}", formatted);
    }
}

fn unified_diff(original: &str, formatted: &str, filename: &str) -> String {
    TextDiff::from_lines(original, formatted)
        .unified_diff()
        .header(filename, filename)
        .to_string()
}
