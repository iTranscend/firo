//! CLI for the contract runner

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[clap(short = 'p', long)]
    pub contract_path: String,
}
