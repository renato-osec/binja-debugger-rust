#!/bin/sh

RUST_LOG=trace cargo run --example headless_debug -- './test/traits.bndb'
