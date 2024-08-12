# syntax=docker/dockerfile:1.4
FROM rust:1.80-slim-bullseye AS base

WORKDIR /code
RUN cargo init
COPY Cargo.toml /code/Cargo.toml
RUN cargo fetch
COPY . /code

FROM base AS builder

RUN apt update && apt-get install -y pkg-config libudev-dev libssl-dev cmake gcc g++
RUN cargo build --release --offline

FROM debian:11.10-slim

COPY --from=builder /code/target/release/owshen /owshen
COPY --from=builder /code/GENESIS.json /GENESIS.json
