# syntax=docker/dockerfile:1
FROM docker.io/debian:bookworm-slim

RUN \
    dpkg --add-architecture arm64 && \
    apt-get update && apt-get -y --no-install-recommends install \
        software-properties-common \
        perl \
        sudo \
        build-essential \
        git \
        clang \
        zip \
        unzip \
        cmake \
        libsqlite3-dev \
        jq \
        ssh \
        libssl-dev \
        pkg-config \
        npm \
        podman \
        ca-certificates \
        gnupg \
        uidmap \
        fuse-overlayfs \
        zstd \
        lld \
        fontconfig \
        libfontconfig-dev \
        curl \
        slirp4netns \
        python3-dev \
        python3-yaml \
        python3-dotenv \
        libc6-dev-arm64-cross \
        libc6-dev:arm64 \
        libc-dev:arm64 \
        gcc-aarch64-linux-gnu \
        libxinerama-dev \
        libxcursor-dev \
        xorg-dev \
        libglu1-mesa-dev \
        autoconf \
        automake \
        libtool \
        gperf \
        autoconf-archive \
    && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

RUN useradd -m runner
WORKDIR /home/runner
RUN \
    curl -o actions-runner-linux-x64-2.328.0.tar.gz -L https://github.com/actions/runner/releases/download/v2.328.0/actions-runner-linux-x64-2.328.0.tar.gz && \
    echo "01066fad3a2893e63e6ca880ae3a1fad5bf9329d60e77ee15f2b97c148c3cd4e  actions-runner-linux-x64-2.328.0.tar.gz" | shasum -a 256 -c && \
    tar xzf ./actions-runner-linux-x64-2.328.0.tar.gz && \
    rm actions-runner-linux-x64-2.328.0.tar.gz

COPY startup.sh .

RUN \
    git clone https://github.com/microsoft/vcpkg.git /opt/vcpkg && \
    /opt/vcpkg/bootstrap-vcpkg.sh -disableMetrics && \
    chown -R runner:runner /opt/vcpkg
ENV VCPKG_ROOT=/opt/vcpkg

RUN \
    curl -o /usr/bin/podman-compose https://raw.githubusercontent.com/containers/podman-compose/84f1fbd6226a412494d9751622139022803bc7c0/podman_compose.py && \
    chmod a+rx /usr/bin/podman-compose

ENTRYPOINT ["./startup.sh"]
