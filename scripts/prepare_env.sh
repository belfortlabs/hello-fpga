#!/bin/bash

export REPO_DIR="$(realpath "$(dirname "$(realpath "$0")")"/..)"

################################################################################
# Clone TFHE-rs and patch it with Belfort FPGA integration

export TFHERS_DIR="$HOME/tfhe-rs"

echo "============="
echo "Clone TFHE-rs"

export TFHERS_URL=https://github.com/zama-ai/tfhe-rs.git
export TFHERS_TAG=tfhe-rs-0.11.3
git clone --no-checkout $TFHERS_URL $TFHERS_DIR

pushd $TFHERS_DIR
echo "==========================================="
echo "Patch TFHE-rs for Belfort FPGA acceleration"

git checkout tags/$TFHERS_TAG -b $TFHERS_TAG
git apply $REPO_DIR/belfort.patch

echo "================================="
echo "Group all changes into one commit"

git add .
git commit -m "Belfort Release"

echo "====================="
echo "Update rust if needed"

make install_rs_check_toolchain
make install_rs_build_toolchain
popd
