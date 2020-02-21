#!/bin/sh

set -ex

cargo +nightly build --verbose

# Only test spaceindex - for some reason we get link errors testing spaceindex-py
cd ../../../spaceindex/
cargo +nightly test --verbose