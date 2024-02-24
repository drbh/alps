#![allow(special_module_name)]

use std::error::Error;
mod lib;
mod tests;
use clap::Parser;
use lib::{solve, UnoptimizedProblem};

/// App Configuration
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    // The name of the file to read from
    #[clap(short, long)]
    input: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let json_problem = std::fs::read_to_string(args.input)?;

    let problem: UnoptimizedProblem = serde_json::from_str(&json_problem).unwrap();

    let solution = solve(problem).unwrap();

    let solution = serde_json::to_string(&solution).unwrap();
    println!("{}", solution);

    Ok(())
}
