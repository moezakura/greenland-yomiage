# syntax=docker/dockerfile:1

# ===== ビルドステージ =====
FROM rust:1-slim-trixie AS builder

# songbird の音声コーデック（opus2）のビルドに cmake / C コンパイラが必要。
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake build-essential pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

# レジストリと target を BuildKit キャッシュに乗せる。
# target はキャッシュマウント上にあるため、同一 RUN 内でバイナリを取り出す。
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release \
    && cp target/release/yomiage /usr/local/bin/yomiage

# ===== ランタイムステージ =====
FROM debian:trixie-slim

# rustls を使うため OpenSSL は不要。TLS 検証用に CA 証明書のみ入れる。
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/local/bin/yomiage /usr/local/bin/yomiage

CMD ["yomiage"]
