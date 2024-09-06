cargo +nightly fmt --quiet
cargo clippy -- -W clippy::pedantic -W clippy::style -W clippy::unwrap_used -W clippy::expect_used -W clippy::allow_attributes
