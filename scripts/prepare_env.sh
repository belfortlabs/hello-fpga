#!/bin/bash

export REPO_DIR="$(realpath "$(dirname "$(realpath "$0")")"/..)"

################################################################################
# Clone TFHE-rs and patch it with Belfort FPGA integration

export TFHERS_DIR="$HOME/tfhe-rs"
export TFHERS_TAG=tfhe-rs-0.11.3
export TFHERS_URL=https://github.com/zama-ai/tfhe-rs.git
export PATCH_COMMIT_MSG="Belfort Patch Applied"

separator() {
    printf "\n=========================================================\n\n"
}

if [ -d "$TFHERS_DIR" ]; then
    echo "Stash changes and set ZAMA GitHub as the origin"
    pushd $TFHERS_DIR
    git stash -m "Stashed changes"
    git remote set-url origin $TFHERS_URL
else
    echo "Fresh Clone of TFHE-rs"
    git clone --no-checkout $TFHERS_URL $TFHERS_DIR
    pushd $TFHERS_DIR
fi

separator
echo "Checkout TFHE-rs for Belfort FPGA acceleration"
PATCH_COMMIT=$(git log --grep="$PATCH_COMMIT_MSG" --format="%H" | head -n 1)

if [ -n "$PATCH_COMMIT" ]; then
    git checkout $PATCH_COMMIT
else
    git checkout tags/$TFHERS_TAG -B $TFHERS_TAG

    echo "Applying Belfort patch..."
    git apply $REPO_DIR/belfort.patch

    separator
    git add .
    echo "Group all changes into one commit"
    git commit -m "$PATCH_COMMIT_MSG"
fi

separator
echo "Update rust if needed"

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
