#!/bin/bash

export REPO_DIR="$(realpath "$(dirname "$(realpath "$0")")"/..)"

################################################################################
# Clone TFHE-rs and patch it with Belfort FPGA integration

export TFHERS_DIR="$REPO_DIR/tfhe-rs"

export TFHERS_URL=https://github.com/zama-ai/tfhe-rs.git
export TFHERS_TAG=tfhe-rs-0.8.3

git clone --no-checkout $TFHERS_URL $TFHERS_DIR

pushd $TFHERS_DIR
git checkout tags/$TFHERS_TAG -b $TFHERS_TAG
git apply $REPO_DIR/belfort.patch

cat <<EOL > "$TFHERS_DIR/tfhe/src/core_crypto/fpga/accelerators/accelerators.rs"
pub const PARAM_MESSAGE_2_CARRY_2: ContainerParameters = ContainerParameters {
  fpga_image: "$REPO_DIR/accel.awsxclbin",
  batch_size: 12,
  streaming_size: 8,
  bsk_num_kernels: 1,
  ksk_num_kernels: 1,
  bsk_bits: 54,
  bsk_frac_bits: 46,
  ksk_bits: 34,
};

pub const DEFAULT_PARAMETERS_KS_PBS: ContainerParameters = ContainerParameters {
  fpga_image: "/path/to/boolean.awsxclbin",
  batch_size: 1,
  streaming_size: 1,
  bsk_num_kernels: 1,
  ksk_num_kernels: 1,
  bsk_bits: 1,
  bsk_frac_bits: 1,
  ksk_bits: 1,
};
EOL

git add .
git checkout -b "Belfort-Release"
git commit -m "Belfort Release"
popd

################################################################################
# Install Rust

export ENV_DIR="$REPO_DIR/.env"
export RUSTUP_HOME=$ENV_DIR/rust/rustup
export CARGO_HOME=$ENV_DIR/rust/cargo

export RUST_SETUP_SH="$REPO_DIR/scripts/rust_setup.sh"

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > $RUST_SETUP_SH

chmod +x $RUST_SETUP_SH
$RUST_SETUP_SH -y --no-modify-path
rm -f $RUST_SETUP_SH

source "$CARGO_HOME/env"

rustup toolchain install nightly-x86_64-unknown-linux-gnu
rustup default nightly

pushd $TFHERS_DIR
make install_rs_check_toolchain
make install_rs_build_toolchain
popd

################################################################################
# Create setup file

export ENV_SETUP_SH="$REPO_DIR/scripts/source_env.sh"

echo ""
echo "export RUSTUP_HOME=$RUSTUP_HOME" >> $ENV_SETUP_SH
echo "export CARGO_HOME=$CARGO_HOME" >> $ENV_SETUP_SH
echo "export RUST_LOG=warn" >> $ENV_SETUP_SH
echo ""
echo "source $CARGO_HOME/env" >> $ENV_SETUP_SH
echo ""

chmod +x $ENV_SETUP_SH
