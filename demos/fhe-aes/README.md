# Transciphering: A Homomorphic AES Evaluation using TFHE
Created by [Beren Aydoğan](https://github.com/wouterlegiest)

Transciphering is a technique that allows symmetrically encrypted data to be processed using fully homomorphic encryption (FHE), without revealing the underlying plaintext. 

In this workflow, a symmetric encryption algorithm first encrypts the data. When computation is required, the ciphertext is homomorphically decrypted: the plaintext remains hidden, but becomes processable within the FHE domain.

Transciphering is useful for performing secure computations on encrypted data in scenarios where the data is initially protected by efficient symmetric encryption, but later needs to be processed homomorphically without exposing the plaintext. Another advantage is that it helps preserve memory bandwidth, particularly during client-server data transfers, by keeping ciphertexts compact. Since symmetric encryption produces much smaller ciphertexts compared to FHE, transmitting symmetrically encrypted data and then transciphering it on the server side significantly reduces communication overhead and improves performance in bandwidth-constrained environments.

This repository contains a hardware-accelerated transciphering implementation that combines AES (Advanced Encryption Standard) with TFHE (Fully Homomorphic Encryption over the Torus), specifically optimized for the Belfort FPGA platform.

## Implementation Details

### AES Configuration

There are three types of AES, each defined by the length of the encryption key: AES-128, AES-192, and AES-256. All three use a fixed 128-bit block size but differ in security level and computational cost. This implementation focuses on AES-128, as it offers a balance between performance and security and is widely adopted in practice.

Block ciphers like AES can be used in various modes of operation, which defines how blocks are processed and how secure the overall encryption is in different contexts. For simplicity in our implementation, we focus on the electronic codebook (ECB) mode, where each 128-bit block is encrypted independently, directly using the key. However, other modes can easily be introduced on top of our implementation, as they operate at a higher level by defining how individual AES block encryptions are chained. 

### FHE Optimization

The optimizations in our implementation are specifically tailored for the Belfort FPGA platform, which supports programmable bootstrapping (PBS) batching through a patched version of the `tfhe-rs` library by Zama. We use the `Shortint v0.11` parameter set (`PARAM_MESSAGE_2_CARRY_2_KS_PBS`), which allows a 2-bit message and 2-bit carry space. The optimizations are designed to exploit the structure and constraints of shortint ciphertexts.

One key optimization involves the use of a tower field construction, a mathematical technique that enables breaking down inversion in a field into operations over smaller fields. This structure allows the use of compact lookup tables to compute the S-box substitute, compatible with the parameter set we are using. The mathematical foundations of this approach are detailed in [Fan and Paar, On Efficient Inversion in Tower Fields of Characteristic Two](https://doi.org/10.1109/ISIT.1997.612935) and further applied to S-box optimization in [Satoh et al., A Compact Rijndael Hardware Architecture with S-Box Optimization](https://doi.org/10.1007/3-540-45682-1_15).

## Try Yourself

A terminal-based application showcases transciphering from AES to FHE, after which a homomorphic addition takes place. The application is designed to run on Belfort FPGA–enabled servers, such as AWS F2 instances. For setup instructions, please refer to the main [readme](../../README.md).

### What It Does:

- Accepts two plaintext numbers which are the operands.
- Generates a random AES key of 128 bits.
- Encrypts each operand using AES-128.
- Transforms the AES ciphertexts into TFHE domain using trivial encryption.
- Homomorphically AES-decrypts the ciphertexts in the TFHE domain.
- Homomorphically adds the TFHE-encrypted values.
- TFHE-decrypts and displays the sum.

## Running the Demo

Make sure you are on a server with FPGA support (e.g., AWS F2). Then start the program by running:

```bash
cargo run --release --package aes --bin demo --features fpga
```

Follow the on-screen logs for:
- Instructions to enter your operands,
- Observe the homomorphic addition,
- And view the final decrypted result.