#!/bin/sh

set -ex

cargo build --verbose --features $FEATURES
cargo test --verbose --features $FEATURES -- --test-threads=1