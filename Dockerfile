FROM rust:1-slim-bookworm AS builder
ADD Cargo.* .
ADD src/* src/
RUN apt update && \
    apt install -y libssl-dev pkg-config && \
    cargo build --release && \
    cp target/release/airnope /usr/local/bin && \
    cargo clean && \
    apt purge -y libssl-dev pkg-config && \
    apt autoremove -y && \
    rm -rf /var/lib/apt/lists/*

FROM debian:bookworm-slim
COPY --from=builder /usr/local/bin/airnope /usr/local/bin/airnope
RUN apt update && \
    apt install -y ca-certificates openssl && \
    apt autoremove -y && \
    rm -rf /var/lib/apt/lists/*
CMD ["airnope"]

