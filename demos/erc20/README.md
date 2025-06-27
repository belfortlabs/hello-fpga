# Hello-FPGA ERC20 Demo

This is a terminal-based Rust demo that visualizes encrypted ERC20-like token transactions using Fully Homomorphic Encryption (FHE) and optional FPGA acceleration via [TFHE-rs](https://github.com/zama-ai/tfhe-rs). The demo shows how FPGA accelerates secure ERC20 transfers.

## Interactive controls

Compile and run with

```bash
cargo run --release --package tfhe-erc20 --bin demo --features fpga
```

You will see the demo running, after introductory frames:

- Frame 1: Illustrates the ERC20 
- Frame 2: Illustrates the wish to hide the amounts
- Frame 3: Illustrates the use of FHE to encrypt the amounts
- Next frame: In a loop creates random transfers:
    - Press `f` to switch to FPGA execution  
    - Press `c` to switch to CPU execution  
    - Press `q` to quit
