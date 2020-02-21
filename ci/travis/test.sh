#!/bin/sh

set -ex

cargo +nightly build --verbose
cargo +nightly test --verbose