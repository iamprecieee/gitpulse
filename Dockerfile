FROM rust:1.90-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml ./

COPY src/ ./src/

COPY system_prompt.txt ./system_prompt.txt

RUN cargo build --release --bin gitpulse

FROM gcr.io/distroless/cc-debian12

WORKDIR /app

COPY --from=builder /app/target/release/gitpulse /app/gitpulse

COPY --from=builder /app/system_prompt.txt /app/system_prompt.txt

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

EXPOSE 8000

ENV RUST_LOG=info

ENTRYPOINT [ "/app/gitpulse" ]