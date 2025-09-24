use serde::{Deserialize, Serialize};

#[repr(C)]
pub struct HandleResult {
    pub ptr: u32,
    pub len: u32,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ContractOutput {
    pub message: String,
}
