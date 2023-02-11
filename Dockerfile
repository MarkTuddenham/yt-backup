# --- wasm-rust image ---
FROM rust:1.63-alpine3.15 as rust_build
MAINTAINER Mark Tuddenham	<mark@tudders.com>
# RUN apk add gcc

ENV CARGO_TARGET_DIR=/target
COPY . /app

WORKDIR /app

RUN cargo build --release
# --- ---

# --- Production run server ---
FROM alpine:3.15
MAINTAINER Mark Tuddenham	<mark@tudders.com>
COPY --from=rust_build /app/target/release/yt-backup /app/yt-backup
WORKDIR /app/
CMD ["./yt-backup"]
# --- ---
