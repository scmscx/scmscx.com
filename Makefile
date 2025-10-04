RUST_SOURCE = Cargo.toml Cargo.lock $(shell find crates/ -name "*.rs" -or -name "*.toml" | sed 's/ /\\ /g')
RUST_TARGET_DIR := target/x86_64-unknown-linux-gnu


SHELL=/bin/bash

GIT_VERSION := $(shell git log -1 --format=%H)

$(RUST_TARGET_DIR)/debug/scmscx-com: $(RUST_SOURCE)
	cargo build --bin scmscx-com

$(RUST_TARGET_DIR)/release/scmscx-com: $(RUST_SOURCE)
	cargo build --release --bin scmscx-com

package-lock.json node_modules &: package.json
	npm ci
	touch node_modules
	touch package-lock.json

dist/vite: node_modules vite.config.ts tsconfig.json $(shell find app | sed 's/ /\\ /g')
	mkdir -p dist/vite
	npm run build
	touch $@

dist/debug: $(RUST_TARGET_DIR)/debug/scmscx-com dist/vite
	mkdir -p dist/debug

	cp -pr $(RUST_TARGET_DIR)/debug/scmscx-com dist/debug
	cp -pr app/web/uiv2 dist/debug
	cp -pr app/web/public dist/debug
	cp -a dist/vite dist/debug/dist

	touch $@

dist/release: $(RUST_TARGET_DIR)/release/scmscx-com dist/vite
	mkdir -p dist/release

	cp -pr $(RUST_TARGET_DIR)/release/scmscx-com dist/release
	cp -pr app/web/uiv2 dist/release
	cp -pr app/web/public dist/release
	cp -a dist/vite dist/release/dist

	touch $@

scmscx.com-image-debug: dist/debug
	podman build --build-arg PROFILE="debug" -t "registry.zxcv.io/scmscx.com:$(GIT_VERSION)-debug" -t registry.zxcv.io/scmscx.com:latest-debug -f Dockerfile

scmscx.com-image: dist/release
	podman build --build-arg PROFILE="release" -t "registry.zxcv.io/scmscx.com:$(GIT_VERSION)" -t registry.zxcv.io/scmscx.com:latest -f Dockerfile
render-image:
	podman build -t "registry.zxcv.io/render:$(GIT_VERSION)" -t registry.zxcv.io/render:latest -f render/Dockerfile
postgres-image:
	podman build -t "registry.zxcv.io/postgres:$(GIT_VERSION)" -t registry.zxcv.io/postgres:latest -f postgres/Dockerfile

check: $(RUST_SOURCE)
	cargo check --workspace --all-targets

build: $(RUST_SOURCE)
	cargo build --workspace --all-targets

test: $(RUST_SOURCE)
	cargo test --no-fail-fast --workspace --all-targets
	cargo test --no-fail-fast --workspace --doc

fmt: $(RUST_SOURCE)
	cargo fmt --all -- --check

clippy: $(RUST_SOURCE)
	# cargo clippy -- -Dclippy::all -Dclippy::pedantic
	# __CARGO_FIX_YOLO=1 cargo clippy --fix to get it to apply risky changes
	# TODO: add -D clippy::style one day. in general add more clippy lints.
	# __CARGO_FIX_YOLO=1 cargo clippy --workspace --all-targets --fix --allow-dirty -- \
	cargo clippy --workspace --all-targets -- \
		-A clippy::all \
		-D clippy::correctness \
		-D clippy::suspicious \
		-D clippy::complexity \
		-D clippy::perf \
		-D clippy::or_fun_call \
		-A clippy::clone-on-copy \
		-A clippy::type-complexity

ci: check build test fmt clippy scmscx.com-image-debug

run: scmscx.com-image-debug render-image postgres-image
	GIT_VERSION=$(GIT_VERSION) podman-compose down
	GIT_VERSION=$(GIT_VERSION) podman-compose up

push: scmscx.com-image render-image postgres-image
	podman push "registry.zxcv.io/scmscx.com:$(GIT_VERSION)"
	podman push "registry.zxcv.io/postgres:$(GIT_VERSION)"
	podman push "registry.zxcv.io/render:$(GIT_VERSION)"

dev:
	npm run dev

deploy:
	ssh -i~/.ssh/stan -C root@10.70.23.1 podman pull registry.zxcv.io/scmscx.com
	ssh -i~/.ssh/stan -C root@10.70.23.1 systemctl restart scmscx.com

.PHONY: .phony check build test fmt clippy ci run push dev deploy scmscx.com-image-debug scmscx.com-image render-image postgres-image
