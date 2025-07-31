# Hello-FPGA HEIR ML Hello-World Demo

This is a terminal-based Rust demo that calculates a tiny one-layer neural network. The code is compiled by the Google [HEIR compiler](https://heir.dev).
The MLIR input of the compiler is given at the [Github of HEIR](https://github.com/google/heir/blob/main/tests/Examples/tfhe_rust_hl/cpu/hello_world_clean_xsmall.mlir). 
Through this demo, the inputs of the neural network are encrypted. 
The program uses Fully Homomorphic Encryption (FHE) and optional FPGA acceleration via [TFHE-rs](https://github.com/zama-ai/tfhe-rs).

## Interactive controls

Compile and run with

```bash
cargo run --release --package tfhe-heir-ml --bin demo --features fpga
```

