FROM golang:1.20

WORKDIR /app

RUN apt-get update \
    && apt-get install ffmpeg -y

COPY ./app/go.mod .
COPY ./app/go.sum .

RUN go mod download

COPY ./app .

RUN go build -o bot general/cmd/main.go

CMD ["./bot"]