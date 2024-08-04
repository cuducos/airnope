FROM rust:1-slim-bookworm
WORKDIR /usr/src/airnope

COPY Cargo.toml Cargo.lock ./
RUN apt-get update && \
    apt-get install -y ca-certificates g++ libssl-dev pkg-config && \
    mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    cargo build --release && \
    rm -rf src/*

COPY src ./src
RUN cargo build --release && \
    cargo run -- --download

CMD ["sh", "-c", "cargo run --release"]
