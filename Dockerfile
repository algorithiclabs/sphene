# Stage 1: Build
FROM rust:1.98-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && \
    apt-get install -y clang libclang-dev nasm pkg-config cmake && \
    rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release

# Stage 2: Runtime Micro-Image
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y imagemagick && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sphene /usr/local/bin/sphene

# The Drop-In Symlinks
RUN ln -sf /usr/local/bin/sphene /usr/local/bin/magick && \
    ln -sf /usr/local/bin/sphene /usr/local/bin/convert

RUN useradd --system --create-home --shell /usr/sbin/nologin sphene && \
    mkdir /work && chown sphene:sphene /work
WORKDIR /work
USER sphene
ENTRYPOINT ["sphene"]
