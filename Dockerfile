# -- Base Image --
# Installs application dependencies
FROM rust:slim-buster as builder

ARG VERSION
ENV VERSION=$VERSION

# Set up application environment
WORKDIR /ord
RUN cargo init --bin \
 && mkdir ./src/bin \
 && mv ./src/main.rs ./src/bin \
 && touch ./src/lib.rs
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./test-bitcoincore-rpc ./test-bitcoincore-rpc
RUN cargo build --bin ord --release \
 && rm -r ./src
COPY ./src ./src
COPY ./templates ./templates
COPY ./static ./static
RUN rm ./target/release/deps/ord* ./target/release/deps/libord* \
 && cargo build --bin ord --release

# -- Test Image --
# Code to be mounted into /app
FROM builder AS test
# TODO
ENTRYPOINT ["./scripts/entry.sh", "test"]

# -- Production Image --
# Runs the service
FROM debian:buster-slim AS prod
WORKDIR /app
COPY --from=builder /ord/target/release/ord /usr/local/bin
ENTRYPOINT ["ord", "--data-dir", "/ord/index", "--bitcoin-data-dir", "/ord/bitcoin"]
