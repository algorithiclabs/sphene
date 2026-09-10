# Stage 1: Build
FROM rust:1.98-slim-bookworm AS builder
WORKDIR /app

# Enable bookworm-backports for newer libheif, then install dependencies
RUN echo "deb http://deb.debian.org/debian bookworm-backports main" > /etc/apt/sources.list.d/backports.list && \
    apt-get update && \
    apt-get install -y clang libclang-dev nasm pkg-config cmake && \
    apt-get install -y -t bookworm-backports libheif-dev

COPY . .
RUN cargo build --release

# Stage 2: Runtime Micro-Image
FROM debian:bookworm-slim

# Enable bookworm-backports for the runtime library
RUN echo "deb http://deb.debian.org/debian bookworm-backports main" > /etc/apt/sources.list.d/backports.list && \
    apt-get update && \
    apt-get install -y imagemagick && \
    apt-get install -y -t bookworm-backports libheif1 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sphene /usr/local/bin/sphene

# The Drop-In Symlinks
RUN ln -sf /usr/local/bin/sphene /usr/local/bin/magick && \
    ln -sf /usr/local/bin/sphene /usr/local/bin/convert

ENTRYPOINT ["sphene"]
