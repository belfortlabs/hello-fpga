#!/bin/bash
set -e

export REPO_DIR="$(realpath "$(dirname "$(realpath "$0")")"/..)"

################################################################################
# Clone TFHE-rs and patch it with Belfort FPGA integration

export TFHERS_DIR="$(realpath "$REPO_DIR/../tfhe-rs")"
export TFHERS_TAG=tfhe-rs-1.4.2
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
    git -c user.name="Hello FPGA" -c user.email="hello-fpga@belfortlabs.com" commit -m "$PATCH_COMMIT_MSG"
fi

separator
echo "Update rust if needed"

export ENV_DIR="$HOME/.cargo"
export RUSTUP_HOME=$ENV_DIR/rustup
export CARGO_HOME=$ENV_DIR/cargo

export RUST_SETUP_SH="$REPO_DIR/scripts/rust_setup.sh"

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > $RUST_SETUP_SH

chmod +x $RUST_SETUP_SH
$RUST_SETUP_SH -y --no-modify-path
rm -f $RUST_SETUP_SH

source "$CARGO_HOME/env"

pushd $TFHERS_DIR
make install_rs_check_toolchain
make install_rs_build_toolchain
popd
