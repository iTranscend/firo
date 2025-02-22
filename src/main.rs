use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;
use wasmtime::{Caller, Engine, Linker, Module, Store, Val};

#[derive(Serialize)]
struct InputData {
    name: String,
}

#[derive(Deserialize, Debug)]
struct ContractResult {
    message: String,
}

struct GasConfig {
    memory_page_cost: u64,
    memory_read_cost: u64,
    memory_write_cost: u64,
    compute_cost: u64,
}

pub struct HostState {
    gas_left: u64,
    gas_limit: u64,
    gas_config: GasConfig,
}

#[derive(Error, Debug)]
pub enum GasError {
    #[error("Insufficient gas: required {required}, but only {remaining} left")]
    OutOfGas { required: u64, remaining: u64 },
}

impl HostState {
    pub fn new(gas_limit: u64) -> Self {
        Self {
            gas_left: gas_limit,
            gas_limit,
            gas_config: GasConfig {
                memory_page_cost: 15,
                memory_read_cost: 2,
                memory_write_cost: 4,
                compute_cost: 6,
            },
        }
    }

    pub fn charge_gas(&mut self, amount: u64) -> Result<(), GasError> {
        if self.gas_left < amount {
            return Err(GasError::OutOfGas {
                required: amount,
                remaining: self.gas_left,
            });
        }
        self.gas_left -= amount;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), wasmtime::Error> {
    let engine = Engine::default();
    let module = Module::from_file(
        &engine,
        "./target/wasm32-unknown-unknown/release/sample_contract.wasm",
    )?;

    let state = HostState::new(2_000_000);
    let mut linker: Linker<HostState> = Linker::new(&engine);
    let mut store = Store::new(&engine, state);

    linker.func_wrap(
        "host",
        "allocate",
        |mut caller: Caller<'_, HostState>, size: i32| -> i32 {
            let pages = ((size + 0xffff) / 0x10000) as u64;
            caller
                .data_mut()
                .charge_gas(pages * 10)
                .map_err(|_| wasmtime::Trap::OutOfFuel)
                .unwrap();

            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
            let ptr = memory.data_size(&caller);
            memory
                .grow(caller, ((size + 0xffff) / 0x10000) as u64)
                .map_err(|_| wasmtime::Trap::MemoryOutOfBounds)
                .unwrap();
            ptr as i32
        },
    )?;

    // Instantiate wasm module
    let instance = linker.instantiate(&mut store, &module)?;

    let handle = instance.get_func(&mut store, "handle").unwrap();
    let input = InputData {
        name: "Danny".to_string(),
    };
    let input_json = serde_json::to_vec(&input)?;
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let _ = memory.write(&mut store, 0, &input_json);

    let mut results = vec![Val::I32(0)];
    let _ = handle.call(
        &mut store,
        &[0.into(), (input_json.len() as i32).into()],
        &mut results,
    )?;

    #[repr(C)]
    #[derive(Debug)]
    pub struct HandleResult {
        ptr: u32,
        len: u32,
    }
    dbg!(&results);

    // Retrieve output
    let output_ptr = results[0].unwrap_i32() as usize;
    let output = &memory.data(&store)[output_ptr..output_ptr + std::mem::size_of::<HandleResult>()];
    dbg!(output);
    let int = output as *const _ as *const HandleResult;
    let handle_result = unsafe { &*int };

    dbg!(handle_result);

    {
        let nu_handle_result = HandleResult {
            ptr: 69 as _,
            len: 120,
        };
        let ptr = std::ptr::addr_of!(nu_handle_result) as *const u8;
        let bytes = unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<HandleResult>()) };

        dbg!(bytes);
    }

    let ptr = handle_result.ptr as usize;
    let len = handle_result.len as usize;
    let handle_result_slice = &memory.data(&store)[ptr..ptr + len];
    let contract_result: ContractResult = serde_json::from_slice(handle_result_slice)?;

    println!("Contract result -> {:#?}", contract_result);

    Ok(())
}
