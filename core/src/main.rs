use clap::Parser;

use crate::engine::{ContractEngine, InputData};

mod cli;
mod engine;
mod errors;

#[tokio::main]
async fn main() -> Result<(), wasmtime::Error> {
    let args = cli::Args::parse();
    let input = InputData {
        name: "Manny".to_string(),
    };

    for (index, contract) in args.contract_paths.iter().enumerate() {
        let engine = ContractEngine::new(contract.clone());
        let exec_result = engine.run(input.clone()).await?;
        match exec_result {
            Ok((output, gas)) => {
                println!("Contract {} result -> {:#?}", index, output);

                println!("Contract {} gas consumed -> {}", index, gas);
            }
            Err(e) => panic!("{}", e),
        }
    }

    Ok(())
}
