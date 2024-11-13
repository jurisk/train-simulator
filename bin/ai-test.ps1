set RUST_LOG=info
cargo test --package game-integration-tests --test ai ai_until_final_goods_built_oct2025 -- --exact --nocapture