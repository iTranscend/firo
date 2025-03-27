use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use wasmtime::{Caller, Engine, Linker, Module, Store, Val};

mod cli;

#[derive(Serialize)]
struct InputData {
    name: String,
}

#[derive(Deserialize, Debug)]
struct ContractResult {
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), wasmtime::Error> {
    let args = cli::Args::parse();

    let mut config = wasmtime::Config::new();
    config.consume_fuel(true);
    let engine = Engine::new(&config)?;
    let module = Module::from_file(&engine, args.contract_path)?;

    let mut linker: Linker<()> = Linker::new(&engine);
    let mut store = Store::new(&engine, ());
    store.set_fuel(1_000_000)?;

    linker.func_wrap(
        "host",
        "allocate",
        |mut caller: Caller<'_, ()>, size: i32| -> i32 {
            let pages = ((size + 0xffff) / 0x10000) as u64;

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

    let handle = instance.get_func(&mut store, "handle").unwrap();
    let input = InputData {
        name: "Danny".to_string(),
    };
    let input_json = serde_json::to_vec(&input)?;
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let _ = memory.write(&mut store, 0, &input_json);

    let initial_fuel = store.get_fuel()?;

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

    let fuel_after_exec = store.get_fuel()?;
    let fuel_consumed = initial_fuel - fuel_after_exec;
    println!("Gas consumed -> {}", fuel_consumed);

    Ok(())
}
