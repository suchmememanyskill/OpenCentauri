FROM ghcr.io/cross-rs/armv7-unknown-linux-musleabihf:latest

RUN apt-get update && \
    apt-get install -y libfontconfig1-dev libssl-dev openssl && \
    rm -rf /var/lib/apt/lists/*
