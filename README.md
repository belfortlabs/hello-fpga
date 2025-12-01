<p align="center">
<!-- product name logo -->
<picture>
  <source media="(prefers-color-scheme: light)" srcset="https://github.com/user-attachments/assets/a81f0598-59b1-4160-95d3-661cab99f6a8">
  <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/a6abe4d5-e849-435c-8319-70029f08d201">
  <img width=600 alt="Belfort FHE Accelerator">
</picture>
</p>

---

# Belfort FHE Accelerator

This repo provides demo applications implemented on TFHE-rs, and enables FPGA acceleration on it.

Check out the [How to migrate your code for FPGA acceleration?](#how-to-migrate-your-code-for-fpga-acceleration) section below to migrate your application. The following steps enable Belfort FPGA acceleration of your THFE-rs code:

```Rust
// Import the Belfort dependency
use tfhe::integer::fpga::BelfortServerKey;

// Generates the FPGA key from server_key
let mut fpga_key = BelfortServerKey::from(&server_key);

// Connect to the FPGAs
fpga_key.connect();

// Accelerates operations with FPGA
set_server_key(fpga_key.clone());

// The rest of your code stays unchanged
```

:warning: This is the early access version of the Belfort FHE Accelerator.

## How to run a demo?

### Prepare execution environment

1. SSH into the `bologna` Belfort development server

```bash
ssh -i <ssh_key> <username>@bologna.belfortlabs.cloud
```

2. Clone this repo into your home directory

```bash
git clone https://github.com/belfortlabs/hello-fpga.git
```

3. Run the setup script

```bash
cd hello-fpga && ./scripts/prepare_env.sh
```

4. Set the environment variables

```bash
source ~/.cargo/cargo/env
source /opt/belfort/source_tools
```

Note: add the above lines to your `.bashrc` file to automatically source the environment variables when you log in.

### Run the weighted-sum tutorial

You can run both CPU and FPGA version of the application and compare the execution time differences;

```bash
cargo run --release --package example --bin weighted-sum
```

```bash
cargo run --release --package example --bin weighted-sum --features fpga
```

You should see the result of the weighted-sum complete much faster with the FPGA feature!

### Other demos

This repository also contains more comprehensive demo applications. Below you can find the applications and the related commands. They should be run from the root repository and expects an [initialized environment](#prepare-execution-environment).

#### AES

[AES demo](/demos/fhe-aes/README.md) implements the transciphering of AES into FHE. You can run the interactive demo with:

```bash
cargo run --release --package fhe-aes --bin demo --features fpga
```

#### Other Demos:

- [ERC20 demo](/demos/erc20/README.md) is a terminal-based Rust demo that visualizes encrypted ERC20-like token transactions.
- [Trivium demo](/demos/trivium/README.md) for the transciphering of trivium into FHE.

## How to migrate your code for FPGA acceleration?

The acceleration requires `BelfortServerKey` created from the `server_key`, which connects to the FPGA cores. You can find below the weighted-sum example with the code differences for both CPU and FPGA execution.

**Change 5 lines of code:**

The changes focus solely on key creation, which is standard for every TFHE application. The modifications are limited to using `fpga_key` as your `server_key`. All other computation code remains unchanged and will automatically benefit from FPGA acceleration.


```Rust
/// Import dependencies                                         // Import dependencies
                                                          |     use tfhe::integer::fpga::BelfortServerKey;

/// Create Keys                                                 // Create Keys
let config = ConfigBuilder::default().build();                  let config = ConfigBuilder::default().build();
let client_key = ClientKey::generate(config);                   let client_key = ClientKey::generate(config);
let server_key = client_key.generate_server_key();              let server_key = client_key.generate_server_key();

                                                          |     let mut fpga_key = BelfortServerKey::from(&server_key);
                                                          |     fpga_key.connect();
set_server_key(server_key);                               |     set_server_key(fpga_key.clone());

// Compute on encrypted data                                    // Compute on encrypted data

                                                                // Disconnect from FPGA
                                                          |     fpga_key.disconnect();
```

**Update your `Cargo.toml`:**

1. Change the `tfhe` dependency to use your local fpga-enabled `tfhe-rs` repo:

```toml
[dependencies]
tfhe = { path = "../../tfhe-rs/tfhe", features = [
    "shortint",
    "integer",
    "experimental-force_fft_algo_dif4",
] }
```

2. Add the `fpga` feature to your application's `Cargo.toml`:

```toml
[features]
fpga = ["tfhe/fpga"]
emulate_fpga = ["tfhe/emulate_fpga"]
```

These are the only changes to your code to enable FPGA acceleration.

### Specify FPGA cores

If you want to specify the number of FPGA cores to use, you can use the alternative `connect_to()` instead of the `connect()` function.
This can be useful for development purposes or distributing access of the resources to multiple users.

```Rust
let mut fpga_key = BelfortServerKey::from(&server_key);
fpga_key.connect_to(vec![0,1,2,3]); // Specifies connection to FPGA cores with indices 0,1,2 and 3
set_server_key(fpga_key);
```

### Caveats

- Lesser used operations are stubbed out with a software implementation. Our team is continuously replacing them with HW optimized versions.
- Enabling the logger ([as in env_logger::init(); in the tutorial](./tutorials/src/main.rs#L12)) gives you runtime warnings if a non-accelerated function is used. Contact us if you would like priority support for a function that emits a warning.
- Current implementations use FFT, but NTT support is under development.
- Development for a specialized cloud environment with optimized performance is ongoing.
- The FPGAs can also be emulated while running the demos (useful when hardware access is unavailable) by replacing:
    ```bash
    --features fpga
    ```
    with:
    ```bash
    --features "fpga,emulate_fpga"
    ```


### Contributors

- [Beren Aydoğan](https://github.com/berenaydogan), developer of the AES demo

### License

Belfort's AMI is free to use only for development, research, prototyping, and experimentation purposes. However, for any commercial use of Belfort's AMI, companies must purchase Belfort’s commercial AMI license.

This software is distributed under the **BSD 3-Clause Clear** license. Read [the license](LICENSE) for more details.

Each demo contributed by independent developers includes its own license file in the corresponding folder.
