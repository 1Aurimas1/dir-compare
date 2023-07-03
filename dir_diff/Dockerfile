FROM rust:1.68 AS builder

WORKDIR /app_builder

COPY ./Cargo.toml /app_builder/Cargo.toml
COPY ./Cargo.lock /app_builder/Cargo.lock

COPY ./src/main.rs /app_builder/src/main.rs

RUN cargo build --release


# FROM scratch
FROM debian:buster-slim

WORKDIR /app

COPY --from=builder /app_builder/target/release/dir_diff /app

ENTRYPOINT ["/app/dir_diff"]
# CMD ["/usr/bin/sh", "/app/dir_diff"]
