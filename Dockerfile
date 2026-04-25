# syntax=docker/dockerfile:1

# ステージ1: libdave ビルド
FROM golang:1.24 AS dave-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake make git pkg-config g++ curl zip unzip tar ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN git clone https://github.com/disgoorg/godave.git /godave-src
WORKDIR /godave-src
RUN NON_INTERACTIVE=1 bash scripts/libdave_install.sh v1.1.0/cpp

# ステージ2: Go アプリビルド
FROM golang:1.24 AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    libopus-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

# libdave ヘッダーとライブラリをコピー
COPY --from=dave-builder /root/.local/lib/ /usr/local/lib/
COPY --from=dave-builder /root/.local/include/ /usr/local/include/
COPY --from=dave-builder /root/.local/lib/pkgconfig/dave.pc /usr/local/lib/pkgconfig/
RUN ldconfig

# dave.pc の prefix を /usr/local に書き換え
RUN sed -i 's|prefix=.*|prefix=/usr/local|' /usr/local/lib/pkgconfig/dave.pc

WORKDIR /app

ENV GOPRIVATE=10.77.0.20/*
ENV GONOSUMCHECK=10.77.0.20/*
ENV GOINSECURE=10.77.0.20/*

COPY ./app/go.mod ./app/go.sum ./
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    go mod download

COPY ./app .

RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    go build -o bot general/cmd/main.go

# ステージ3: ランタイム
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg libopus0 ca-certificates libstdc++6 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/bot .
COPY --from=dave-builder /root/.local/lib/libdave* /usr/local/lib/
RUN ldconfig

CMD ["./bot"]
