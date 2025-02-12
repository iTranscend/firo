# firo

WASM Smart contract runner leveraging the [wasmtime](https://wasmtime.dev/) runtime

# Usage

### Compile sample rust contract to wasm

```sh
cargo build --release -p sample-contract --target wasm32-unknown-unknown
```

### Run wasm contract

```sh
cargo run
```
