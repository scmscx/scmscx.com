# syntax=docker/dockerfile:1
FROM docker.io/ubuntu:24.04
#RUN apt update

RUN \
    apt-get update && \
    apt install -y --no-install-recommends \
        software-properties-common \ 
        heaptrack \
    && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN mkdir /app/pending

ENV RUST_LOG=info
ENV RUST_BACKTRACE=full

ENV DEV_MODE=false
ENV ROOT_DIR=/app
ENV DB_CONNECTIONS=64
ENV USE_SSL=false

ARG PROFILE
RUN test -n "$PROFILE"

COPY dist/assets .
COPY target/x86_64-unknown-linux-gnu/$PROFILE/scmscx-com .

VOLUME ["/app/pending"]

ENTRYPOINT ["/app/scmscx-com"]
# ENTRYPOINT ["heaptrack", "/app/bwmapserver"]
