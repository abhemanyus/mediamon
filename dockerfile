FROM rust:1.77.0-slim-bullseye as build

WORKDIR /build

RUN rustup target add x86_64-unknown-linux-gnu && \
    update-ca-certificates

# ENV ORT_STRATEGY=system
# ENV ORT_LIB_LOCATION=/lib
COPY libonnxruntime.so* /lib/
COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    "mediamon"

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --target x86_64-unknown-linux-gnu --release \
    && cp /build/target/x86_64-unknown-linux-gnu/release/mediamon .


FROM debian:11-slim
ENV ORT_STRATEGY=system
ENV ORT_LIB_LOCATION=./libonnxruntime.so

RUN apt-get update && apt-get -y install python ffmpeg pip
RUN pip install -U yt-dlp gallery-dl mutagen

COPY ./gallery-dl/config.json /etc/gallery-dl.conf

WORKDIR /app

COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group

USER mediamon:mediamon

COPY tags.txt /app/
# COPY ./target/debug/libonnx* /app/
COPY migrations /app/

COPY --chown=mediamon:mediamon deepdanbooru.onnx /app/
ENV ORT_STRATEGY=system
ENV ORT_LIB_LOCATION=/lib
COPY --chown=mediamon:mediamon libonnxruntime.so* /lib/
COPY --from=build --chown=mediamon:mediamon /build/mediamon /app/mediamon

ENTRYPOINT [ "/app/mediamon" ]
