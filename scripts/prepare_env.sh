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

# Set default rust version
rustup default stable