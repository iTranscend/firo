use std::sync::Arc;

use sdk::{ContractOutput, HandleResult};
use serde::Serialize;
use wasmtime::{Caller, Engine, Linker, Module, Store, Val};

use crate::errors::ContractError;

type GasUsed = u64;
type ContractExecutionResult = Result<(ContractOutput, GasUsed), wasmtime::Error>;

#[derive(Serialize, Clone)]
pub struct InputData {
    pub name: String,
}

pub struct ContractEngine {
    contract_path: String,
}

impl ContractEngine {
    pub fn new(contract_path: String) -> Arc<Self> {
        Arc::new(Self { contract_path })
    }

    pub async fn run(
        self: Arc<Self>,
        input: InputData,
    ) -> Result<ContractExecutionResult, wasmtime::Error> {
        // Run contract exec in separate task
        let this = self.clone();
        let handle = tokio::spawn(async move { this.execute_contract(input).await });
        let result = handle.await.unwrap();

        Ok(result)
    }

    pub async fn execute_contract(
        &self,
        input: InputData,
    ) -> Result<(ContractOutput, GasUsed), wasmtime::Error> {
        let mut config = wasmtime::Config::new();
        config.consume_fuel(true);
        let engine = Engine::new(&config)?;
        let module = Module::from_file(&engine, &self.contract_path)?;

        let mut linker = Linker::new(&engine);
        let mut store = Store::new(&engine, ());
        store.set_fuel(1_000_000)?;

        // Host function for memory allocation
        linker.func_wrap(
            "host",
            "allocate",
            |mut caller: Caller<'_, ()>, size: i32| -> i32 {
                let pages = ((size + 0xfff) / 0x10000) as u64;
                let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
                let ptr = memory.data_size(&caller);
                memory
                    .grow(caller, pages)
                    .map_err(|_| wasmtime::Trap::MemoryOutOfBounds)
                    .unwrap();
                ptr as i32
            },
        )?;

        // Instantiate wasm module
        let instance = linker.instantiate(&mut store, &module)?;

        // get handle function
        let func = "handle";
        let handle =
            instance
                .get_func(&mut store, func)
                .ok_or_else(|| ContractError::FunctionNotFound {
                    name: func.to_owned(),
                })?;

        let input_json = serde_json::to_vec(&input).map_err(|e| ContractError::Serialization(e))?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| "")
            .map_err(|_| ContractError::MemoryNotFound)?;

        memory
            .write(&mut store, 0, &input_json)
            .map_err(|e| ContractError::InvalidMemoryAccess(e.to_string()))?;

        // Record initial fuel
        let initial_gas = store.get_fuel()?;

        // Call 'handle' function on guest
        let params: Vec<Val> = vec![0.into(), (input_json.len() as i32).into()];
        let mut results = vec![Val::I32(0)];
        handle.call(&mut store, &params, &mut results)?;

        // Get fuel consumed
        let gas_used = initial_gas - store.get_fuel()?;

        // Parse result
        let output_ptr = results[0].unwrap_i32() as usize;
        let memory_data = memory.data(&store);

        let handle_func_result_size = std::mem::size_of::<HandleResult>();

        if output_ptr + handle_func_result_size < memory_data.len() {
            // throw error
            // panic!("invalid memory access");
        }

        let output = &memory.data(&store)[output_ptr..output_ptr + handle_func_result_size];
        let int = output as *const _ as *const HandleResult;
        let handle_result = unsafe { &*int };

        let ptr = handle_result.ptr as usize;
        let len = handle_result.len as usize;
        let handle_result_slice = &memory_data[ptr..ptr + len];
        let contract_output: ContractOutput = serde_json::from_slice(handle_result_slice)?;

        Ok((contract_output, gas_used))
    }
}
