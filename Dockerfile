FROM rust:1-slim-bookworm
WORKDIR /usr/src/airnope

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN apt-get clean && \
    apt-get update && \
    apt-get install -y ca-certificates g++ libssl-dev pkg-config && \
    cargo build --release && \
    cargo run -- --download

CMD ["sh", "-c", "cargo run --release"]
