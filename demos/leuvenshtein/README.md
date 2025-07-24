# Leuvenshtein demo: Approximated db lookup

Created by Wouter Legiest

Demo application to showcase the efficient processing of the Leuvenshtein algorithm on the Belfort FPGA's. The demo implements following aspects:

- Encrypted query search in an encrypted database of names
- Plaintext query search in an encrypted database using preprocessing techniques
- Execution of above situations, both on CPU and FPGA

Citation:
> Wouter Legiest, Jan-Pieter D'Anvers, Bojan Spasic, Nam-Luc Tran, & Ingrid Verbauwhede. (2025). Leuvenshtein: Efficient FHE-based Edit Distance Computation with Single Bootstrap per Cell.

Current versions:
tfhe-rs: 0.11.3
branch: dev-wout

## General overview

The program completely runs in the terminal, hence it is OS independent and can be showcased anywhere in the world, while running on Belfort servers.

Once the program starts, the screen will be cleared and will remain black for a period of time. During this time, the database is being preprocessed. After a while, the main menu will be displayed.

## Usages

Start up the program by running the following command:

```bash
cargo run --release --package leuvenshtein --bin demo --features fpga
```

The main menu will start to flicker when it starts. You can now choose `e` to execute a query on the CPU. Pressing `f` will have the same effect, but now the query is processed on the FPGAs. `q` quits the program.

If the query starts with `p:`, it is NOT encrypted and send in plaintext to the database. Resulting in an execution that is a third faster.

