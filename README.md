# BELFORT FPGA Acceleration

:warning: This is the early access version of the Belfort FHE Accelerator, demonstrating its functionality on AWS. While the full acceleration capabilities are limited by AWS FPGAs' restrictions, this early access version enables users to verify the Belfort FPGA integration.

## What FPGA acceleration requires?

You can find a weighted-sum example in this repo for both CPU and FPGA execution. Use `diff` to see how minimal the changes are for FPGA acceleration.

```bash
diff -y hello-fpga/src/weighted_sum_on_cpu.rs hello-fpga/src/weighted_sum_on_fpga.rs
```

**Change of only 3 lines of code:**

```Rust
// Create keys                                        // Create keys
let client_key = ClientKey::generate(config);         let client_key = ClientKey::generate(config);
let server_key = client_key.generate_server_key();    let server_key = client_key.generate_server_key();

                                                    | let mut fpga_key = BelfortServerKey::from(&server_key);
set_server_key(server_key);                         | set_server_key(fpga_key.clone());
                                                    | fpga_key.connect(1);

// Prepare your encrypted data                        // Prepare your encrypted data
```

## How to run the demo?

### Setup your AWS Account

:exclamation: A new AWS account may not have the required quota allowance to launch a `f1.2xlarge` type instance. In this case, file a [quota increase request](https://aws.amazon.com/getting-started/hands-on/request-service-quota-increase/) for the `Running On-Demand F instances` service, which you can search for under `Service Quotas > Amazon Elastic Compute Cloud (Amazon EC2)`. Make sure to combine your request with **at least 8 vCPU cores**, as `f1.2xlarge` requires 8 vCPUs. The quota increase may take up to a few days to process.

In your communication to AWS, please pay attention that the F1 access permissions are tied to a given region. Though we provide the demo application in this repository, the actual FPGA image is hosted by AWS. We make that image publicly available in all the F1 instance regions of today, which are `us-east-1`, `us-west-2`, `eu-west-1`, `ap-southeast-2`, `eu-central-1` and `eu-west-2`. If more regions with F1 instances appear in future, we will make the image available in those regions too. If you notice that we are late to do this, you can create an issue.

### Get access permissions

For running the demo, you need access permissions to the Belfort AMI and FPGA accelerator. We will grant you access, if you drop us a message with your AWS ID over [Belfort.eu](https://belfort.eu/contact/), create an issue here on GitHub, or [post/dm on X](https://x.com/belfort_eu).

### Launch an F1 instance

Launch an AWS EC2 F1 instance.

- Instance type: `f1.2xlarge`
- AMI: Belfort FPGA Acceleration AMI - `ami-06810d664ae1d2325`.
  - This AMI is prepared by Belfort, free of charge, and ready-to-use, based on Ubuntu 20.04 LTS.

### Prepare execution environment

1. SSH into your instance with your credentials

```bash
ssh -i <id.pem> ubuntu@<instance_public_dns>
```

2. Clone this repo into your AWS instance, or `scp` your existing clone (clean!) to it

```bash
scp -i <id.pem> -r . ubuntu@<instance_public_dns>:~/.
```

3. Run `prepare_env.sh` for setting up the execution environment; cloning `TFHE-rs`, patching it with the Belfort extensions, installing Rust, ...

```bash
cd hello-fpga && ./scripts/prepare_env.sh
```

4. Source the created `source_env.sh` file for setting the environment vars for Rust development

```bash
source ./scripts/source_env.sh
```

5. You are ready to go

### Run the example applications

You can run both CPU and FPGA version of the application and compare the execution time differences;

```bash
cargo run --release --package hello-fpga --bin weighted-sum-on-cpu
```

```bash
cargo run --release --package hello-fpga --bin weighted-sum-on-fpga --features fpga
```

### Caveats

- Lesser used operations are stubbed out with a software implementation. Our team is continuously replacing them with HW optimized versions.
- Enabling the logger gives you runtime warnings if a software function is used. Contact us if you would like priority support for a function that emits a warning.
- Current implementations use FFT, but NTT support is under development.
- Development for a specialized cloud environment with optimized performance is ongoing.
