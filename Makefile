RUST_SOURCE = Cargo.toml Cargo.lock $(shell find crates/ -name "*.rs" -or -name "*.toml" -or -name "*.cpp" -or -name "*.h" | sed 's/ /\\ /g')
RUST_TARGET_DIR := target/x86_64-unknown-linux-gnu


# SHELL=/bin/bash

GIT_VERSION := $(shell git log -1 --format=%H)

$(RUST_TARGET_DIR)/debug/scmscx-com: $(RUST_SOURCE)
	cargo build --bin scmscx-com
$(RUST_TARGET_DIR)/release/scmscx-com: $(RUST_SOURCE)
	cargo build --release --bin scmscx-com

$(RUST_TARGET_DIR)/debug/bwrender: $(RUST_SOURCE)
	cargo build -p bwrender
$(RUST_TARGET_DIR)/release/bwrender: $(RUST_SOURCE)
	cargo build --release -p bwrender

package-lock.json node_modules &: package.json
	npm ci
	touch node_modules
	touch package-lock.json

dist/vite: node_modules vite.config.ts tsconfig.json $(shell find app | sed 's/ /\\ /g')
	mkdir -p dist/vite
	npm run build
	touch $@

dist/assets: dist/vite
	mkdir -p dist/assets

	cp -pr app/web/uiv2 dist/assets
	cp -pr app/web/public dist/assets
	cp -a dist/vite dist/assets/dist

	touch $@

scmscx.com-image-debug: $(RUST_TARGET_DIR)/debug/scmscx-com dist/assets
	podman build --build-arg PROFILE="debug" -t "registry.zxcv.io/scmscx.com:$(GIT_VERSION)-debug" -t registry.zxcv.io/scmscx.com:latest-debug -f Dockerfile .
scmscx.com-image: $(RUST_TARGET_DIR)/release/scmscx-com dist/assets
	podman build --build-arg PROFILE="release" -t "registry.zxcv.io/scmscx.com:$(GIT_VERSION)" -t registry.zxcv.io/scmscx.com:latest -f Dockerfile .

bwrender-image-debug: $(RUST_TARGET_DIR)/debug/bwrender
	podman build --build-arg PROFILE="debug" -t "registry.zxcv.io/bwrender:$(GIT_VERSION)-debug" -t registry.zxcv.io/bwrender:latest-debug -f bwrender.Dockerfile .
bwrender-image: $(RUST_TARGET_DIR)/release/bwrender
	podman build --build-arg PROFILE="release" -t "registry.zxcv.io/bwrender:$(GIT_VERSION)" -t registry.zxcv.io/bwrender:latest -f bwrender.Dockerfile .

postgres-image:
	podman build -t "registry.zxcv.io/postgres:$(GIT_VERSION)" -t registry.zxcv.io/postgres:latest -f postgres/Dockerfile

check: $(RUST_SOURCE)
	cargo check --workspace --all-targets

build: $(RUST_SOURCE)
	cargo build --workspace --all-targets

test: $(RUST_SOURCE)
	cargo test --no-fail-fast --workspace --all-targets
	cargo test --no-fail-fast --workspace --doc

E2E_PG_CONTAINER := scmscx-e2e-pg
E2E_PG_PORT ?= 55432
E2E_PG_PASSWORD ?= anotverysecurepassword

# End-to-end tests: start ONE shared Postgres (the project's postgres/ image) and
# hand its address to the tests via E2E_PG_*; each test clones its own isolated
# database from it (CREATE DATABASE ... TEMPLATE). The container lifecycle lives
# here, not in the tests. Needs podman + the built frontend manifest (dist/vite)
# and the postgres-image (both Make prerequisites, so Make builds them as needed).
e2e: $(RUST_SOURCE) dist/vite postgres-image
	podman rm -f $(E2E_PG_CONTAINER) >/dev/null 2>&1 || true
	podman run -d --name $(E2E_PG_CONTAINER) -p 127.0.0.1:$(E2E_PG_PORT):5432 \
		-e POSTGRES_PASSWORD=$(E2E_PG_PASSWORD) registry.zxcv.io/postgres:latest \
		-c max_connections=500
	@echo "waiting for postgres schema (bounding.net) ..."
	@for i in $$(seq 1 60); do \
		podman exec $(E2E_PG_CONTAINER) psql -U bounding.net -d bounding.net -tAc 'select 1' >/dev/null 2>&1 && break; \
		sleep 1; \
	done
	E2E_PG_HOST=127.0.0.1 E2E_PG_PORT=$(E2E_PG_PORT) E2E_PG_PASSWORD=$(E2E_PG_PASSWORD) \
		cargo test -p scmscx-com --features e2e --test e2e -- --nocapture; \
		status=$$?; \
		podman rm -f $(E2E_PG_CONTAINER) >/dev/null; \
		exit $$status

fmt: $(RUST_SOURCE)
	cargo fmt --all -- --check

clippy: $(RUST_SOURCE)
	cargo clippy --workspace --all-targets -- -D warnings

ci: fmt clippy test e2e scmscx.com-image-debug

# Mutation testing — measures how well the tests pin down behavior. Scope and
# rationale live in .cargo/mutants.toml. Requires cargo-mutants
# (`cargo install cargo-mutants`).
#
# Like `make e2e`, this starts the shared Postgres and hands its address to the
# tests: cargo-mutants gates each mutation on the whole `scmscx-com` test suite,
# INCLUDING the E2E integration tests (so handler/middleware mutations are caught
# by the black-box HTTP tests, not just unit tests). The E2E harness needs the
# same environment as `make e2e` — a running Postgres and wireguard access to the
# map origin (10.99.99.5:5000) for the map fixture. Running bare `cargo mutants`
# (no container / E2E_PG_*) makes the baseline E2E run panic, so go through here.
#
# Runs `--in-place` (mutating the real tree, restored afterward) rather than
# cargo-mutants' default copy-to-tmp: the C-library submodules
# (StormLib/zlib/bzip2/compact_enc_det) build in-tree via cmake/configure and
# leave non-relocatable CMake caches that fail to build once copied to /tmp, and
# the gitignored `dist/` the E2E server needs wouldn't be copied either. Building
# in place sidesteps both, at the cost of per-mutant parallelism.
#
# Kept out of `ci`: slower than the unit suite, and it exits non-zero while real
# gaps remain (that's the point). The E2E harness drops each per-test database on
# teardown so one long-lived container survives a whole run's thousands of tests.
mutants: $(RUST_SOURCE) dist/vite postgres-image
	podman rm -f $(E2E_PG_CONTAINER) >/dev/null 2>&1 || true
	podman run -d --name $(E2E_PG_CONTAINER) -p 127.0.0.1:$(E2E_PG_PORT):5432 \
		-e POSTGRES_PASSWORD=$(E2E_PG_PASSWORD) registry.zxcv.io/postgres:latest \
		-c max_connections=500
	@echo "waiting for postgres schema (bounding.net) ..."
	@for i in $$(seq 1 60); do \
		podman exec $(E2E_PG_CONTAINER) psql -U bounding.net -d bounding.net -tAc 'select 1' >/dev/null 2>&1 && break; \
		sleep 1; \
	done
	E2E_PG_HOST=127.0.0.1 E2E_PG_PORT=$(E2E_PG_PORT) E2E_PG_PASSWORD=$(E2E_PG_PASSWORD) \
		cargo mutants --in-place --features e2e $(MUTANTS_ARGS); \
		status=$$?; \
		podman rm -f $(E2E_PG_CONTAINER) >/dev/null; \
		exit $$status

run: scmscx.com-image-debug bwrender-image-debug postgres-image
	GIT_VERSION=$(GIT_VERSION) podman-compose down
	GIT_VERSION=$(GIT_VERSION) podman-compose up

push: scmscx.com-image bwrender-image postgres-image
	podman push "registry.zxcv.io/scmscx.com:$(GIT_VERSION)"
	podman push "registry.zxcv.io/postgres:$(GIT_VERSION)"
	podman push "registry.zxcv.io/bwrender:$(GIT_VERSION)"

dev:
	npm run dev

deploy:
	ssh -i~/.ssh/stan -C root@10.70.23.1 podman pull registry.zxcv.io/scmscx.com
	ssh -i~/.ssh/stan -C root@10.70.23.1 systemctl restart scmscx.com

.PHONY: .phony check build test e2e mutants fmt clippy ci run push dev deploy scmscx.com-image-debug scmscx.com-image bwrender-image-debug bwrender-image postgres-image
