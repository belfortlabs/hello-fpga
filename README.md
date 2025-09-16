# BELFORT FHE Accelerator

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

:warning: This is the early access version of the Belfort FHE Accelerator, demonstrating its functionality on AWS.

## How to run a demo?

### Setup your AWS Account

AWS accounts do not have access to F2 instances by default. You need to file [quota increase request](https://aws.amazon.com/getting-started/hands-on/request-service-quota-increase/) for the `Running On-Demand F instances` service, which you can search for under `Service Quotas > Amazon Elastic Compute Cloud (Amazon EC2)`. Make sure to combine your request with **at least 24 vCPU cores**, as `f2.6xlarge` requires 24 vCPUs. The quota increase may take up to a few days to process.

In your communication to AWS, please pay attention that the F2 access permissions are tied to a region. **The FPGA image is available in all the F2 instance regions of today, which are `us-east-1`, `us-west-2`, `ap-southeast-2` and `eu-west-2`**.

### Launch an F2 instance

Open the [Amazon EC2 Console](https://console.aws.amazon.com/ec2/) and launch an AWS EC2 F2 instance based on Belfort public AMI (Amazon Machine Image).

Upon need a guide for AWS console, check this [tutorial](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/tutorial-launch-a-test-ec2-instance.html#tut2-task-4-launch-test-instance).

AMI: [Belfort FPGA Acceleration AMI](https://aws.amazon.com/marketplace/pp/prodview-imfiyzy7svjgu) on the AWS Marketplace.
  - This AMI by Belfort is ready-to-use.
  - It's free of charge, though using AWS EC2 comes with its own fees.

Instance types: pick one below depending on the FPGA acceleration you demand
  - `f2.6xlarge` for 1 FPGA (requires access to 24 vCPUs)
  - `f2.12xlarge` for 2 FPGAs (requires access to 48 vCPUs)
  - `f2.48xlarge` for 8 FPGAs (requires access to 192 vCPUs)

### Prepare execution environment

1. SSH into your instance with your credentials

```bash
ssh -i <id.pem> ubuntu@<instance_public_dns>
```

2. Clone this repo into your AWS instance

3. Run `prepare_env.sh` for cloning TFHE-rs and patching it with the Belfort extensions

```bash
cd hello-fpga && ./scripts/prepare_env.sh
```

### Run the weighted-sum tutorial

You can run both CPU and FPGA version of the application and compare the execution time differences;

```bash
cargo run --release --package example --bin weighted-sum
```

```bash
cargo run --release --package example --bin weighted-sum --features fpga
```

You should see the result of the weighted-sum complete much faster with the FPGA feature! In case you run into any issues, please open an issue in this repo.

### Other demos

This repository also contains more comprehensive demo applications. Below you can find the applications and the related commands. They should be run from the root repository and expects an [initialized environment](#prepare-execution-environment).

#### AES

[AES demo](/demos/fhe-aes/README.md) implements the transciphering of AES into FHE. You can run the interactive demo with:

```bash
cargo run --release --package fhe-aes --bin demo --features fpga
```

#### Leuvenshtein

[Leuvenshtein demo](/demos/leuvenshtein/README.md) implements the fuzzy matching algorithm to compare two strings while allowing a limited amount of mistakes. You can run the interactive demo with:

```bash
cargo run --release --package leuvenshtein --bin demo --features fpga
```

#### Other Demos:

- [ERC20 demo](/demos/erc20/README.md) is a terminal-based Rust demo that visualizes encrypted ERC20-like token transactions.
- [Trivium demo](/demos/trivium/README.md) for the transciphering of trivium into FHE.

## How to migrate your code for FPGA acceleration?

The acceleration requires a `BelfortServerKey` created from the `server_key`, which connects to the FPGA cores. You can find a weighted-sum example with the code differences for both CPU and FPGA execution below.

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

- Additional commands are available to interact with the FPGA's:
  - `fpga-program`: programs the fpga's with the Belfort FPGA image released on AWS.
                    This command is only required if the FPGA's were reset.
  - `fpga-reset`:   Resets the fpga images. This is useful if you kill your app with `Ctrl+C` while it interacts with the FPGAs,
                    and the FPGAs are in a bad state.
- **In case you run your programs without the fpga's programmed, you will get segmentation faults.**
- Lesser used operations are stubbed out with a software implementation. Our team is continuously replacing them with HW optimized versions.
- Enabling the logger ([as in env_logger::init(); in the tutorial](./tutorials/src/main.rs#L12)) gives you runtime warnings if a non-accelerated function is used. Contact us if you would like priority support for a function that emits a warning.
- Current implementations use FFT, but NTT support is under development.
- Development for a specialized cloud environment with optimized performance is ongoing.

### Contributors

- [Wouter Legiest](https://github.com/woutlegiest), developer of the Leuvenshtein demo
- [Beren Aydoğan](https://github.com/berenaydogan), developer of the AES demo


### License

Belfort's AMI is free to use only for development, research, prototyping, and experimentation purposes. However, for any commercial use of Belfort's AMI, companies must purchase Belfort’s commercial AMI license.

This software is distributed under the **BSD 3-Clause Clear** license. Read [the license](LICENSE) for more details.

Each demo contributed by independent developers includes its own license file in the corresponding folder.
