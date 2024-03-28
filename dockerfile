FROM rust:1.77.0-bookworm

WORKDIR /app

COPY ./ ./

CMD ["bash"]
