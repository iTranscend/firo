//! CLI for the contract runner

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[clap(short = 'p', long = "contract-paths", value_delimiter = ',')]
    pub contract_paths: Vec<String>,
}
