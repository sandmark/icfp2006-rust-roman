use std::fs::read_to_string;

use anyhow::Result;
use clap::Parser;
use icfp2006_rust_roman::{ConvertMode, convert};

#[derive(Parser, Debug)]
#[clap(author = "sandmark", version, about)]
/// Application configuration
struct Args {
    /// convert to roman
    #[arg(short, long)]
    roman: bool,

    /// convert to decimal
    #[arg(short, long)]
    decimal: bool,

    /// a path to the file to be converted
    #[arg()]
    path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.roman && args.decimal {
        anyhow::bail!("Both --roman and --decimal can't be specified at the same time.");
    }

    let mode = if args.decimal {
        ConvertMode::Decimal
    } else {
        ConvertMode::Roman
    };

    let code = read_to_string(args.path)?;
    let converted = convert(code, mode);
    println!("{}", converted);
    Ok(())
}
