#!/usr/bin/env zsh
cargo +nightly fmt
cargo clippy -- -W clippy::pedantic -W clippy::style -W clippy::unwrap_used -W clippy::expect_used
