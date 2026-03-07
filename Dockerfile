# ステージ1: libdave ビルド
FROM golang:1.24 AS dave-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake make git pkg-config g++ curl zip unzip tar ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN git clone https://github.com/sh1ma/discordgo.git /discordgo

WORKDIR /discordgo
RUN bash scripts/setup-dave.sh

# ステージ2: Go アプリビルド
FROM golang:1.24 AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    libopus-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY ./app/go.mod ./app/go.sum ./
COPY --from=dave-builder /discordgo /discordgo

RUN go mod edit -replace github.com/bwmarrin/discordgo=/discordgo
RUN go mod download

COPY ./app .

RUN go build -o bot general/cmd/main.go

# ステージ3: ランタイム
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg libopus0 ca-certificates libstdc++6 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/bot .

CMD ["./bot"]
