# See https://github.com/LukeMathWalker/cargo-chef?tab=readme-ov-file#without-the-pre-built-image

FROM rustlang/rust:1 as chef

# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN apt-get update
RUN apt-get install -y build-essential
RUN apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev
RUN apt-get install -y lld clang
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-bindgen-cli

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release  --package networking-server --bin server_console
RUN cargo build --profile wasm-release --target wasm32-unknown-unknown --package client-single-player --package networking-client --bin client_single_player --bin client_graphical
RUN wasm-bindgen --out-name client_single_player --out-dir static/wasm-build --target web target/wasm32-unknown-unknown/wasm-release/client_single_player.wasm
RUN wasm-bindgen --out-name client_graphical --out-dir static/wasm-build --target web target/wasm32-unknown-unknown/wasm-release/client_graphical.wasm
RUN cp -r assets static

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
EXPOSE 5000/tcp
EXPOSE 8080/tcp
WORKDIR app
COPY --from=builder /app/target/release/server_console /app
COPY --from=builder /app/static /app/static
ENTRYPOINT ["/app/server_console"]