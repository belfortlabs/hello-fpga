# Leuvenshtein demo: Approximated DB Lookup

Created by [Wouter Legiest](https://github.com/wouterlegiest)

Leuvenshtein is a fuzzy matching algorithm. Fuzzy matching is a technology to compare two strings while allowing a limited amount of mistakes. For example, it matches "Bilba Biggins" to "Bilbo Baggins". [Leuvenshtein](https://lirias.kuleuven.be/retrieve/797861) is a hand-crafted version of Levenshtein optimising its implementation for the FHE domain, enabling comparison of encrypted strings. This can be useful for applications, such as banking, government, health care, where data sensitivity is high. For further details, please check out the [publication](https://eprint.iacr.org/2025/012).

The demo application showcases the efficient processing of encrypted searches on the Belfort FPGAs. The demo implements following aspects:

- Encrypted query search in an encrypted database of names
- Plaintext query search in an encrypted database using preprocessing techniques
- Execution of above situations, both on CPU and FPGA

## Video Overview

[![Watch the video](https://github.com/user-attachments/assets/645045e5-52b6-4613-91d2-3506ddada15f)](https://youtu.be/6p6BDZx0ps0)

## Try Yourself

The demo is a terminal based application, to be run on Belfort FPGA enabled servers, e.g. AWS F2 as instructed on the main [readme](../../README.md).


**To run the demo yourself:**

Start up the program by running the following command:

```bash
cargo run --release --package leuvenshtein --bin demo --features fpga
```

The app will start with a cleared screen, which will remain empty for a period of time. During this period, the database is being preprocessed. 

After that, the main menu will appear. As instructed on the menu:

- `e` is to start a queary to be executed on multi-core CPU 
- `f` is to start a queary to be executed on FPGAs
- `q` quits the program
- `p:` is to start a plaintext query on un-encrypted database, resulting in an execution that is a third faster.
