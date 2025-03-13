# BELFORT FPGA Acceleration

:warning: This is the early access version of the Belfort FHE Accelerator, demonstrating its functionality on Belfort development server `flipflop`. This early access version enables users to verify the Belfort FPGA integration.

## What FPGA acceleration requires?

You can find a weighted-sum example in this repo for both CPU and FPGA execution. Use `diff` to see how minimal the changes are for FPGA acceleration.

```bash
diff -y hello-fpga/src/weighted_sum_on_cpu.rs hello-fpga/src/weighted_sum_on_fpga.rs
```

**Change of only 3 lines of code:**

```Rust
// Create Keys                                                  // Create Keys

let config = ConfigBuilder::default().build();                  let config = ConfigBuilder::default().build();
let client_key = ClientKey::generate(config);                   let client_key = ClientKey::generate(config);
let server_key = client_key.generate_server_key();              let server_key = client_key.generate_server_key();

                                                          |     let mut fpga_key = BelfortServerKey::from(&server_key);
                                                          |     fpga_key.connect();
set_server_key(server_key);                               |     set_server_key(fpga_key.clone());

// Encrypt Values                                               // Encrypt Values
```

## How to run the demo?

### Prepare execution environment

1. SSH into Belfort development server `flipflop` by hopping over ESAT gateway `ssh.esat.kuleuven.be`

```bash
ssh -J <username>@ssh.esat.kuleuven.be <username>@flipflop.esat.kuleuven.be
```

2. Get a copy of this repo.

*Clone* - You might need to set your ssh-keys for being able to clone:

```bash
git clone --branch flipflop git@github.com:belfort-labs/hello-fpga.git
```

*SCP* - If you have a local clone, you can `scp` it to `flipflop`:

```bash
scp -r -J <username>@ssh.esat.kuleuven.be ./ <username>@flipflop.esat.kuleuven.be:~/hello-fpga
```

3. Source `prepare_env.sh` for cloning `TFHE-rs` and patching it with the Belfort extensions

```bash
cd hello-fpga && source ./scripts/prepare_env.sh
```

4. Set up the execution environment, which will make FPGAs discoverable to TFHE-rs

```bash
source /tools/source_tools
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
