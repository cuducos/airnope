FROM rust:1-slim-trixie AS build

WORKDIR /usr/src/airnope
ENV BUILD_PKGS="ca-certificates g++ gcc libc6-dev libssl-dev make pkg-config"

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN apt-get clean && \
    apt-get update && \
    apt-get install -y ${BUILD_PKGS} && \
    cargo install --path . && \
    cargo clean && \
    airnope download && \
    apt-get -y purge ${BUILD_PKGS} && \
    apt-get -y autoremove && \
    rm -rf /var/lib/apt/lists/*

FROM debian:trixie-slim

RUN apt-get clean && \
    apt-get update && \
    apt-get install -y ca-certificates && \
    apt-get -y autoremove && \
    rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/local/cargo/bin/airnope* /usr/local/bin/
COPY --from=build /root/.cache/huggingface /root/.cache/huggingface

CMD ["airnope", "bot"]
