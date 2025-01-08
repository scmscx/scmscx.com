
ARG PG_MAJOR=15

FROM docker.io/postgres:15.6-bullseye

COPY --chown=postgres:postgres postgresql.conf /etc/postgresql/postgresql.conf

RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends build-essential clang-13 llvm-13 postgresql-server-dev-$PG_MAJOR git ca-certificates && \
    mkdir -p /tmp/pgvector && \
    cd /tmp/pgvector && \
    git clone --depth 1 --branch v0.7.1 https://github.com/pgvector/pgvector.git && \
    apt-mark hold locales && \
    cd pgvector && \
    make clean && \
    make OPTFLAGS="" && \
    make install && \
    mkdir /usr/share/doc/pgvector && \
    cp LICENSE README.md /usr/share/doc/pgvector && \
    rm -r /tmp/pgvector && \
    apt-get remove -y build-essential postgresql-server-dev-$PG_MAJOR git ca-certificates && \
    apt-get autoremove -y && \
    apt-mark unhold locales && \
    rm -rf /var/lib/apt/lists/*

COPY 00-init.sh /docker-entrypoint-initdb.d/00-init.sh
# COPY schema.sql /docker-entrypoint-initdb.d/01-schema.sql
