#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("failed to create engine: {0}")]
    EngineCreation(#[from] wasmtime::Error),
    #[error("Function `{name}` not found in contract")]
    FunctionNotFound { name: String },
    #[error("Memory not found in contract")]
    MemoryNotFound,
    #[error("Invalid memory access: {0}")]
    InvalidMemoryAccess(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
