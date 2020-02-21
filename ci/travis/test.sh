#!/bin/sh

set -ex

cargo build --verbose
cargo test --verbose