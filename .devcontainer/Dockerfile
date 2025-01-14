# syntax=docker/dockerfile:1
FROM --platform=linux/amd64 ubuntu:24.04
#RUN apt update

RUN \
    apt-get update && \
    apt install -y --no-install-recommends \
        software-properties-common \
        heaptrack \
        build-essential \
        git \
        rustup \
        podman \
        libssl-dev \
        pkg-config \
        cmake \
        clang \
        npm \
        curl \
        python3-dev \
        python3-yaml \
        python3-dotenv \
        ssh \
        nano \
        vim \
        unzip \
        less \
    && \
    rm -rf /var/lib/apt/lists/* \
    && \
    rustup toolchain add stable --no-self-update \
    && \
    curl -o /usr/bin/podman-compose https://raw.githubusercontent.com/containers/podman-compose/84f1fbd6226a412494d9751622139022803bc7c0/podman_compose.py && \
    chmod a+rx /usr/bin/podman-compose

ENTRYPOINT ["/bin/bash"]
