#!/usr/bin/env zsh
cargo +nightly fmt --quiet
cargo clippy --quiet -- -W clippy::pedantic -W clippy::style -W clippy::unwrap_used -W clippy::expect_used
cargo +nightly llvm-cov nextest --branch