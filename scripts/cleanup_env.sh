#!/bin/bash

export REPO_DIR="$(realpath "$(dirname "$(realpath "$0")")"/..)"

################################################################################
# Cleanup TFHE-rs

export TFHERS_DIR="$REPO_DIR/tfhe-rs"

cargo clean
rm -rf $REPO_DIR/Cargo.lock
rm -rf $TFHERS_DIR

################################################################################
# Cleanup .env

export ENV_DIR="$REPO_DIR/.env"

rm -rf $ENV_DIR

################################################################################
# Cleanup setup file

export SETUP_FILE="$REPO_DIR/scripts/source_env.sh"

rm -rf $SETUP_FILE
