# BELFORT FPGA Acceleration

:warning: This is the early access version of the Belfort FHE Accelerator, demonstrating its functionality on AWS. While the full acceleration capabilities are limited by AWS FPGAs' restrictions, this early access version enables users to verify Belfort FPGA integration.

## What FPGA acceleration requires?

You can fine a weighted-sum example in this repo, prepared for execution on both CPU and FPGA. You can use `diff` to see how minimal the changes are for utilizing FPGA acceleration.

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

// Calculate on encrypted data                        // Calculate on encrypted data
let encypted_weighted_sum =                           let encypted_weighted_sum =
      encypted_value1 * encypted_weight1                     encypted_value1 * encypted_weight1
    + encypted_value2 * encypted_weight2                   + encypted_value2 * encypted_weight2
    + encypted_value3 * encypted_weight3;                  + encypted_value3 * encypted_weight3;
```

## How to run the demo?

### Setup your AWS Account

:exclamation: A new AWS account may not have the required quota allowance to launch a `f1.2xlarge` type instance. If that is your case, file a [quota increase request](https://aws.amazon.com/getting-started/hands-on/request-service-quota-increase/) for the `Running On-Demand F instances` service. Make sure to combine your request with at least 8 CPU cores, as `f1.2xlarge` contains 8 vCPUs. The quota increase may take up to a few days to process.

In your communication to AWS, please pay attention that the F1 access permissions are tied to a given region. Though we provide the demo application in this repository, the actual FPGA image is hosted by AWS. We make that image publicly available in all the F1 instance regions of today, which are `us-east-1`, `us-west-2`, `eu-west-1`, `ap-southeast-2`, `eu-central-1` and `eu-west-2`. If more regions with F1 instances appear in future, we will make the image available in those regions too. If you notice that we are late to do this, you can create an issue.

### Launch an F1 instance

Launch an AWS EC2 F1 instance.

- Instance type: `f1.2xlarge`
- AMI: Belfort FPGA Acceleration AMI - `ami-06810d664ae1d2325`.
  - This AMI is prepared by Belfort, free of charge, and comes ready-to-use, based on Ubuntu 20.04 LTS.

### Prepare execution environment

1. SSH into your instance with the credentials you set at launching it

  ```bash
  ssh -i <id.pem> ubuntu@<instance_public_dns>
  ```

2. Clone this repo into your AWS instance, or `scp` your existing clone to it

  ```bash
  scp -i <id.pem> -r . ubuntu@<instance_public_dns>:~/.
  ```

3. Run `prepare_env.sh` for setting up the execution environment; cloning `TFHE-rs`, patching it Belfort extensions, installing Rust, ...

  ```bash
  ./scripts/prepare_env.sh
  ```

4. Source the created `source_env.sh` file for setting the environment vars for Rust development

  ```bash
  source ./scripts/source_env.sh
  ```

5. You are ready to go

### Run the example applications

You can run the both CPU and FPGA version of the application and compare the execution time differences;

```bash
cargo run --release --package hello-fpga --bin weighted-sum-on-cpu
```

```bash
cargo run --release --package hello-fpga --bin weighted-sum-on-fpga
```
