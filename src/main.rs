use clap::Parser;
use colored::Colorize;
use similar::TextDiff;
use std::process;

mod ast;
mod formatter;
mod parser;

#[derive(Parser)]
#[command(name = "nasfmt", about = "NASM x86-64 assembly formatter")]
struct Cli {
    files: Vec<std::path::PathBuf>,
    #[arg(long, help = "Check formatting; exit 1 with diff if changes needed")]
    check: bool,
    #[arg(
        long,
        help = "Normalise mnemonics, registers, and size prefixes to UPPER CASE"
    )]
    upper: bool,
}

fn main() {
    let cli = Cli::parse();
    let mut exit_code = 0;
    let in_place = cli.files.len() > 1;
    for file in &cli.files {
        let source = std::fs::read_to_string(file).unwrap_or_else(|e| {
            eprintln!("nasfmt: {}: {}", file.display(), e);
            process::exit(2);
        });
        let lines = parser::parse(&source);
        let formatted = formatter::format(&lines, cli.upper);
        if cli.check {
            if formatted != source {
                print!(
                    "{}",
                    unified_diff(&source, &formatted, &file.to_string_lossy())
                );
                exit_code = 1;
            }
        } else if in_place {
            if formatted != source {
                std::fs::write(file, &formatted).unwrap_or_else(|e| {
                    eprintln!("nasfmt: {}: {}", file.display(), e);
                    process::exit(2);
                });
            }
        } else {
            print!("{formatted}");
        }
    }
    process::exit(exit_code);
}

fn unified_diff(original: &str, formatted: &str, filename: &str) -> String {
    let diff = TextDiff::from_lines(original, formatted)
        .unified_diff()
        .header(filename, filename)
        .to_string();
    let mut output = diff
        .lines()
        .map(|line| {
            if line.starts_with('+') && !line.starts_with("+++") {
                line.green().to_string()
            } else if line.starts_with('-') && !line.starts_with("---") {
                line.red().to_string()
            } else if line.starts_with('@') {
                line.cyan().to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    output.push('\n');
    output
}
