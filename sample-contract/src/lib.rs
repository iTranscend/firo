use serde::{Deserialize};

use sdk::{ContractOutput, HandleResult};

#[derive(Deserialize)]
struct InputData {
    name: String,
}

#[no_mangle]
pub extern "C" fn handle(input: *const u8, input_len: u32) -> *const HandleResult {
    let input_data = unsafe { std::slice::from_raw_parts(input, input_len as usize) };
    let input_json: InputData = serde_json::from_slice(input_data).unwrap();

    let result = ContractResult {
        message: format!("Hello {}", input_json.name),
    };

    let output = serde_json::to_vec(&result).unwrap().leak();
    let pointer = output.as_mut_ptr();
    let len = output.len() as u32;

    let handle_result = Box::new(HandleResult {
        ptr: pointer as _,
        len,
    });

    Box::leak(handle_result) as _
}
