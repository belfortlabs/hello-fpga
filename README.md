# BELFORT FPGA Acceleration

This repo provides a weighted-sum demo application implemented on TFHE-rs, and enables FPGA acceleration on it.

The following teaser shows the simple code changes (`Cargo.toml` changes follow later) to enable Belfort FPGA acceleration of your THFE-rs code:

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

## How to run the demo?

### Setup your AWS Account

:exclamation: Your AWS account may not have the required vCPU quota allowance to launch F2 instances. In this case, file a [quota increase request](https://aws.amazon.com/getting-started/hands-on/request-service-quota-increase/) for the `Running On-Demand F instances` service, which you can search for under `Service Quotas > Amazon Elastic Compute Cloud (Amazon EC2)`. Make sure to combine your request with **at least 24 vCPU cores**, as `f2.6xlarge` requires 24 vCPUs. The quota increase may take up to a few days to process.

In your communication to AWS, please pay attention that the F2 access permissions are tied to a region. **The FPGA image is publicly available in all the F2 instance regions of today, which are `us-east-1`, `us-west-2`, `ap-southeast-2` and `eu-west-2`**. If more regions with F2 instances appear in future, we will publish the image in those regions too. Feel free to create an issue if you notice that we are late with this.

### Get access permissions

For running the demo, you need access permissions to the Belfort AMI and FPGA accelerator. To receive access, you can send us a message with your AWS ID on [belfortlabs.com](https://belfortlabs.com/), create an issue on GitHub, or [post/dm on X](https://x.com/belfort_eu).

### Launch an F2 instance

Launch an AWS EC2 F2 instance.

- Instance types: `f2.6xlarge` / `f2.12xlarge` / `f2.48xlarge`
- AMI: Belfort FPGA Acceleration AMI - `ami-012d786b8acdd9c72`.
  - This AMI is prepared by Belfort, free of charge, and ready-to-use, based on Ubuntu 24.04 LTS.

Pick the instance type depending on how much FPGA acceleration you want;
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

### Run the example applications

You can run both CPU and FPGA version of the application and compare the execution time differences;

```bash
cargo run --release --package hello-fpga --bin weighted-sum-on-cpu
```

```bash
cargo run --release --package hello-fpga --bin weighted-sum-on-fpga --features fpga
```

## How to migrate your code for FPGA acceleration?

You can find a weighted-sum example in this repo for both CPU and FPGA execution. Use `diff` to see the minimal changes for FPGA acceleration.

```bash
diff -y hello-fpga/src/weighted_sum_on_cpu.rs hello-fpga/src/weighted_sum_on_fpga.rs
```

**Change 5 lines of code:**

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

```Cargo.toml
[dependencies]
tfhe = { path = "../../tfhe-rs/tfhe", features = [
    "shortint",
    "integer",
    "experimental-force_fft_algo_dif4",
] }
```

2. Add the `fpga` feature to your application's `Cargo.toml`:

```Cargo.toml
[features]
fpga = ["tfhe/fpga"]
emulate_fpga = ["tfhe/emulate_fpga"]
```

### Specify the number of FPGA cores

If you want to specify the number of FPGA cores to use, you can use the alternative `connect_to()` instead of the `connect()` function.
This can be useful for development purposes or distributing access of the resources to multiple users. 

```Rust
let mut fpga_key = BelfortServerKey::from(&server_key);
fpga_key.connect_to(4); // Specifies connection to 4 FPGA cores
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
- Enabling the logger gives you runtime warnings if a non-accelerated function is used. Contact us if you would like priority support for a function that emits a warning.
- Current implementations use FFT, but NTT support is under development.
- Development for a specialized cloud environment with optimized performance is ongoing.
