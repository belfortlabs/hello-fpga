# How to run a demo?

## Launch an F2 instance

Launch an AWS EC2 F2 instance based on our public Amazon Machine Image (AMI).

- AMI: [Belfort FPGA Acceleration AMI](https://aws.amazon.com/marketplace/pp/prodview-imfiyzy7svjgu) on the AWS Marketplace.
  - This AMI by Belfort is ready-to-use.
  - It's free of charge, but AWS EC2 fees apply
- Instance types: `f2.6xlarge` / `f2.12xlarge` / `f2.48xlarge`

Pick the instance type depending on how much FPGA acceleration you want;
  - `f2.6xlarge` for 1 FPGA (requires access to 24 vCPUs)
  - `f2.12xlarge` for 2 FPGAs (requires access to 48 vCPUs)
  - `f2.48xlarge` for 8 FPGAs (requires access to 192 vCPUs)

## Prepare execution environment

1. SSH into your instance with your credentials

```bash
ssh -i <id.pem> ubuntu@<instance_public_dns>
```

2. Clone this repo into your AWS instance

3. Run `prepare_env.sh` for cloning TFHE-rs and patching it with the Belfort extensions

```bash
cd hello-fpga && ./scripts/prepare_env.sh
```

## Run the weighted-sum tutorial

You can run both CPU and FPGA version of the application and compare the execution time differences;

```bash
cargo run --release --package example --bin weighted-sum
```

```bash
cargo run --release --package example --bin weighted-sum --features fpga
```

You should see the result of the weighted-sum complete much faster with the FPGA feature! In case you run into any issues, please open an issue in this repo.

## Other demos

This repository also contains more comprehensive demo applications. Below you can find the applications and the related commands. They should be run from the root repository and expects an [initialized environment](#prepare-execution-environment).

### Trivium

[The Trivium demo](/demos/trivium/README.md) contains the transciphering of trivium into FHE. Below you can find its execution command:

```bash
# With FPGA acceleration
cargo run --release --package tfhe-trivium --bin demo-shortint --features fpga

# Without FPGA acceleration
cargo run --release --package tfhe-trivium --bin demo-shortint
```