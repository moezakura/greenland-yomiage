# syntax=docker/dockerfile:1

# ===== cargo-chef ベース =====
# 依存ビルドを独立レイヤに切り出すための cargo-chef を用意する。
FROM rust:1-slim-trixie AS chef
# songbird の音声コーデック（opus2）と rustls の aws-lc-rs のビルドに必要。
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake build-essential pkg-config \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /app

# ===== 依存レシピの生成 =====
# Cargo.toml / Cargo.lock を正規化した recipe.json を作る。
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# ===== ビルド =====
FROM chef AS builder
# 依存だけを先にビルドする。recipe.json（＝Cargo.toml/Cargo.lock）が変わらない
# 限りこの層はキャッシュにヒットし、依存の再コンパイルを丸ごと省ける。
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
# アプリ本体のソースはここで初めて投入する（変更してもこの層から下だけ再ビルド）。
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# ===== ランタイム =====
FROM debian:trixie-slim
# rustls は aws-lc-rs を使うため OpenSSL は不要。TLS 検証用に CA 証明書のみ。
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/yomiage /usr/local/bin/yomiage
CMD ["yomiage"]
