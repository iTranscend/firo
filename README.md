# firo

Concurrent WASM smart contract runner, leveraging the [wasmtime](https://wasmtime.dev/) runtime

# Usage

### Compile sample rust contract to wasm

```sh
cargo build --release -p sample-contract --target wasm32-unknown-unknown
```

### Run wasm contract

```sh
cargo run -- -p "./target/wasm32-unknown-unknown/release/sample_contract.wasm"
```

### Run multiple wasm contracts

```sh
cargo run -- -p "sample_contract.wasm","sample_contract2.wasm"
```
