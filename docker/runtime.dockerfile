FROM rust:1.76-slim

RUN apt-get update && \
	apt-get upgrade -y && \
	apt-get install --no-install-recommends -y \
	protobuf-compiler libprotobuf-dev && \
	rustup target add aarch64-unknown-linux-musl

ENV CC_aarch64_unknown_linux_musl="aarch64-linux-musl-gcc"  \
	AR_aarch64_unknown_linux_musl="aarch64-linux-musl-ar"  \ 
	CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld"

COPY runtime/hybrid-shim/target/aarch64-unknown-linux-musl/debug/containerd-shim-containerd-hybrid /github/workspace
