FROM alpine:3.17 as base_image
MAINTAINER Mark Tuddenham	<mark@tudders.com>
RUN apk add wget python3
RUN wget -P /usr/bin https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp \
		&& chmod +x /usr/bin/yt-dlp

# --- ---

FROM rust:1.67-alpine3.17 as build_image
MAINTAINER Mark Tuddenham	<mark@tudders.com>
RUN apk add gcc musl-dev
ENV CARGO_TARGET_DIR=/target
COPY . /app
WORKDIR /app
RUN cargo build --release

# --- ---

FROM base_image
MAINTAINER Mark Tuddenham	<mark@tudders.com>
COPY --from=build_image /target/release/yt-backup /app/yt-backup
WORKDIR /app
ENTRYPOINT ["./yt-backup"]

