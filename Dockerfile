FROM debian:bookworm-slim AS libtorch
WORKDIR /usr/src

ENV BUILD_PKGS="ca-certificates curl unzip"
ENV LIBTORCH_ZIP=libtorch-cxx11-abi-shared-with-deps-2.1.0%2Bcpu.zip

RUN apt-get clean && \
    apt-get update && \
    apt-get install -y ${BUILD_PKGS} && \
    curl -LO https://download.pytorch.org/libtorch/cpu/${LIBTORCH_ZIP} && \
    unzip ${LIBTORCH_ZIP} && \
    rm ${LIBTORCH_ZIP} && \
    apt-get -y purge ${BUILD_PKGS} && \
    apt-get -y autoremove && \
    rm -rf /var/lib/apt/lists/*

FROM rust:1-slim-bookworm AS build

WORKDIR /usr/src/airnope
ENV LIBTORCH=/usr/local/lib/libtorch
ENV LD_LIBRARY_PATH=${LIBTORCH}/lib
ENV BUILD_PKGS="build-essential ca-certificates g++ libssl-dev pkg-config"

COPY --from=libtorch /usr/src/libtorch ${LIBTORCH}
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN apt-get clean && \
    apt-get update && \
    apt-get install -y ${BUILD_PKGS} && \
    cargo install --path . && \
    cargo clean && \
    airnope --download && \
    apt-get -y purge ${BUILD_PKGS} && \
    apt-get -y autoremove && \
    rm -rf /var/lib/apt/lists/*

FROM debian:bookworm-slim

ENV LIBTORCH=/usr/local/lib/libtorch
ENV LD_LIBRARY_PATH=${LIBTORCH}/lib

RUN apt-get clean && \
    apt-get update && \
    apt-get install -y ca-certificates libgomp1 libssl-dev && \
    apt-get -y autoremove && \
    rm -rf /var/lib/apt/lists/*

COPY --from=libtorch /usr/src/libtorch ${LIBTORCH}
COPY --from=build /usr/local/cargo/bin/airnope /usr/local/bin/airnope
COPY --from=build /root/.cache/.rustbert /root/.cache/.rustbert

CMD ["airnope"]
