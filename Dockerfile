# TODO: Also build the single-player WASM app and serve it with a web server, so we can get rid of Google Pages

# See https://github.com/LukeMathWalker/cargo-chef?tab=readme-ov-file#without-the-pre-built-image

# FROM rust:1 AS chef <-- switching to nightly as some flags we use require it
FROM rustlang/rust:nightly as chef

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

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin server_renet_console

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
EXPOSE 5000
WORKDIR app
COPY --from=builder /app/target/release/server_renet_console /usr/local/bin
ENTRYPOINT ["/usr/local/bin/server_renet_console"]