# Trivium Demo App

The Trivium demo is based on the [example implementation on TFHE-rs](https://github.com/zama-ai/tfhe-rs/tree/main/apps/trivium), put into a wrapper to make it a visually nicer application.

The demo expects a FPGA compatible TFHE-rs version in an adjacent folder. The easiest way is to follow the steps from the [Hello FPGA repository](https://github.com/belfortlabs/hello-fpga)

- [`src/demo`](src/demo) is the directory for the wrapper application to take arguments and print results

- [`src/trivium`](src/trivium) is the directory for the trivium implementations.

  - [`src/trivium/trivium_shortint_fpga.rs`](src/trivium/trivium_shortint_fpga.rs) is the main body for the FPGA implementation. This implementation operates on shortints. It creates `packs` to perform shortint operations in `batches`. A pack serves as an abstraction for the batch computation. The hardware processes packs as batches, with this batching handled internally at the FPGA's integration with TFHE-rs. Users are not required to understand the details of batching. **The key takeaway for end-users is that larger pack sizes result in better performance**.

The rest of the folders under [`src`](src) are software implementation of Trivium, copied from TFHE-rs.

## Execution

The FPGA acceleration is enabled through the feature flag. If the flag isn't specified it is executed on the CPU with optimizations for parallel processing.

```bash
# With FPGA acceleration
cargo run --release --package tfhe-trivium --bin demo-shortint --features fpga

# Without FPGA acceleration
cargo run --release --package tfhe-trivium --bin demo-shortint
```
