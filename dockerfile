FROM rust:1.77.0-slim-bullseye as build

WORKDIR /build

RUN rustup target add x86_64-unknown-linux-gnu && \
    update-ca-certificates

COPY ./src ./src
COPY ./migrations ./migrations
COPY ./build.rs .
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
ENV ORT_DYLIB_LOCATION=/lib/libonnxruntime.so

RUN apt-get update && apt-get -y install python ffmpeg pip
RUN pip install -U yt-dlp gallery-dl mutagen

WORKDIR /app

COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group

ENV DATABASE_URL=/app/data/database.sqlite3
RUN mkdir -p /app/data && touch "${DATABASE_URL}" && chown -R "mediamon:mediamon" /app/data

USER mediamon:mediamon

COPY --chown=mediamon:mediamon libonnxruntime.so /lib/libonnxruntime.so
COPY --from=build --chown=mediamon:mediamon /build/mediamon /app/mediamon

CMD [ "/app/mediamon" ]
