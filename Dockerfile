FROM golang:1.20

WORKDIR /app

RUN apt-get update \
    && apt-get install curl ffmpeg -y

RUN curl -OL https://github.com/VOICEVOX/voicevox_core/releases/download/0.14.4/download-linux-x64 && \
    chmod +x download-linux-x64

RUN ./download-linux-x64 -o voicevox_core

RUN ln -fs "/app/voicevox_core/libvoicevox_core.so" "/app/voicevox_core/libonnxruntime.so.1.13.1" /usr/lib && \
    ln -fs "/app/voicevox_core/voicevox_core.h" /usr/local/include

RUN cp -r /app/voicevox_core/open_jtalk_dic_utf_8-1.11 .

COPY ./app/go.mod .
COPY ./app/go.sum .

RUN go mod download

COPY ./app .

RUN go build -o bot cmd/main.go

RUN cp -r /app/voicevox_core/model /usr/lib/

CMD ["./bot"]