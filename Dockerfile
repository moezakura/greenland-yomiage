# ステージ1: ビルド
FROM golang:1.24-alpine AS builder

WORKDIR /app

RUN apk add --no-cache gcc musl-dev opus-dev

COPY ./app/go.mod .
COPY ./app/go.sum .

RUN go mod download

COPY ./app .

RUN go build -o bot general/cmd/main.go

# ステージ2: 実行
FROM alpine:3.21

RUN apk add --no-cache ffmpeg opus ca-certificates

WORKDIR /app
COPY --from=builder /app/bot .

CMD ["./bot"]
