use clap::Parser;
use csv_cleaner::{clean, Rules};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::ExitCode;

/// Trim, validate and de-duplicate a CSV file, and report what changed.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Input CSV file (first line must be the header).
    input: PathBuf,

    /// Where to write the cleaned CSV.
    #[arg(short, long)]
    output: PathBuf,

    /// TOML file with column rules (see examples/rules.toml).
    #[arg(long)]
    rules: Option<PathBuf>,

    /// Comma-separated columns that must not be empty (used when --rules is not given).
    #[arg(long, value_delimiter = ',')]
    required: Vec<String>,

    /// Comma-separated name=type pairs, e.g. age=integer,price=float.
    #[arg(long = "type", value_delimiter = ',')]
    types: Vec<String>,

    /// Also write the report as JSON to this file.
    #[arg(long)]
    report_json: Option<PathBuf>,
}

fn run(args: &Args) -> Result<(), String> {
    let rules = match &args.rules {
        Some(path) => {
            let text = fs::read_to_string(path)
                .map_err(|e| format!("cannot read rules file {}: {e}", path.display()))?;
            Rules::from_toml_str(&text).map_err(|e| format!("invalid rules file: {e}"))?
        }
        None => Rules::from_flags(&args.required, &args.types)?,
    };

    let input = File::open(&args.input)
        .map_err(|e| format!("cannot open {}: {e}", args.input.display()))?;
    let output = File::create(&args.output)
        .map_err(|e| format!("cannot create {}: {e}", args.output.display()))?;

    let report = clean(input, BufWriter::new(output), &rules).map_err(|e| e.to_string())?;

    print!("{}", report.to_text());
    if let Some(path) = &args.report_json {
        fs::write(path, report.to_json())
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}
