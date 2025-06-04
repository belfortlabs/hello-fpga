#!/bin/bash

export REPO_DIR="$(realpath "$(dirname "$(realpath "$0")")"/..)"

################################################################################
# Clone TFHE-rs and patch it with Belfort FPGA integration

export TFHERS_DIR="$HOME/tfhe-rs"
export TFHERS_TAG=tfhe-rs-0.11.3
export TFHERS_URL=https://github.com/zama-ai/tfhe-rs.git
export PATCH_COMMIT_MSG="Belfort Patch Applied"

get_patch_commit() {
    git log --grep="$PATCH_COMMIT_MSG" --format="%H" | head -n 1
}

echo "============="
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

echo "==========================================="
echo "Checkout TFHE-rs for Belfort FPGA acceleration"
PATCH_COMMIT=$(get_patch_commit)

if [ -n "$PATCH_COMMIT" ]; then
    git checkout $PATCH_COMMIT
else
    git checkout tags/$TFHERS_TAG -B $TFHERS_TAG
    
    echo "Applying Belfort patch..."
    git apply $REPO_DIR/belfort.patch

    echo "================================="
    git add .
    echo "Group all changes into one commit"
    git commit -m "$PATCH_COMMIT_MSG"
fi

echo "====================="
echo "Update rust if needed"

make install_rs_check_toolchain
make install_rs_build_toolchain
popd
