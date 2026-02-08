FROM docker.io/ubuntu:25.04

RUN \
    apt-get update && \
    apt install -y --no-install-recommends \
        xvfb \
        libopengl0 \
        libglu1-mesa \
        libpq5 \
        xauth \
        libglfw3 \
        mesa-utils \
        libgl1-mesa-dri \
    && \
    rm -rf /var/lib/apt/lists/*

ARG PROFILE
RUN test -n "$PROFILE"

COPY bwrender/xvfb-run.sh /usr/bin/xvfb-run.sh
COPY target/x86_64-unknown-linux-gnu/$PROFILE/bwrender /bin/bwrender

ENTRYPOINT ["/usr/bin/xvfb-run.sh", "--auto-servernum", "-s", "-screen 0 4096x4096x24", "/bin/bwrender"]
