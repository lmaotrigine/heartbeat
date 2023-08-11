#![forbid(unsafe_code)]
#![deny(clippy::pedantic, clippy::nursery)]

use clap::Parser;
use regex::Regex;

#[derive(Parser)]
struct Arguments {
    #[arg(long)]
    reference: String,
}

fn main() {
    let args = Arguments::parse();
    let regex = Regex::new("^refs/tags/v?[[:digit:]]+[.][[:digit:]]+[.][[:digit:]]+$")
        .expect("Failed to compile release regex");
    let value = if regex.is_match(&args.reference) {
        "release"
    } else {
        "other"
    };
    eprintln!("ref: {}", args.reference);
    eprintln!("value: {value}");
    println!("value={value}");
}
