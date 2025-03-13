#!/bin/bash

export REPO_DIR="$(realpath "$(dirname "$(realpath "$0")")")"

################################################################################
# Clone TFHE-rs and patch it with Belfort FPGA integration

export TFHERS_DIR="$HOME/tfhe-rs"

export TFHERS_URL=https://github.com/zama-ai/tfhe-rs.git
export TFHERS_TAG=tfhe-rs-0.11.3
git clone --no-checkout $TFHERS_URL $TFHERS_DIR

pushd $TFHERS_DIR
git checkout tags/$TFHERS_TAG -b $TFHERS_TAG
git apply $REPO_DIR/belfort.patch

# Cleanup git history
git add .
git commit -m "Belfort Release"


make install_rs_check_toolchain
make install_rs_build_toolchain
popd


# Program FPGA through .bashrc alias
fpga-setup

