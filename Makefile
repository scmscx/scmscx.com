RUST_SOURCE = Cargo.toml Cargo.lock $(shell find crates/ -name "*.rs" -or -name "*.toml" | sed 's/ /\\ /g')
RUST_TARGET_DIR := target/x86_64-unknown-linux-gnu


# FEATURES ?= ""
# /tmp/scmscx.com/FEATURES: .phony
# 	mkdir -p "$(@D)"
# 	@if [[ `cat ARGS 2>&1` != '$(FEATURES)' ]]; then echo -n $(FEATURES) >$@; fi
SHELL=/bin/bash

ENABLE_GSFS ?= ""

CONFIG ?= ""
/tmp/scmscx.com/ENABLE_GSFS: .phony
	@mkdir -p "$(@D)"
	@if [[ `cat $@ 2>&1` != '$(ENABLE_GSFS)' ]]; then echo -n "$(ENABLE_GSFS)" >$@; fi

$(RUST_TARGET_DIR)/debug/scmscx-com: $(RUST_SOURCE) /tmp/scmscx.com/ENABLE_GSFS
ifeq ($(ENABLE_GSFS), 1)
	cargo build --bin scmscx-com --config 'patch.crates-io.gsfs.git="ssh://git@github.com/zzlk/gsfs.git"' --config 'patch.crates-io.gsfs.branch="main"' --features "gsfs"
else ifeq ($(ENABLE_GSFS), 2)
	cargo build --bin scmscx-com --config 'patch.crates-io.gsfs.path="../gsfs/rust/gsfs"' --features "gsfs"
else
	cargo build --bin scmscx-com
endif

$(RUST_TARGET_DIR)/release/scmscx-com: $(RUST_SOURCE) /tmp/scmscx.com/ENABLE_GSFS
ifeq ($(shell cat /tmp/scmscx.com/ENABLE_GSFS), 1)
	cargo build --release --bin scmscx-com --config 'patch.crates-io.gsfs.git="ssh://git@github.com/zzlk/gsfs.git"' --config 'patch.crates-io.gsfs.branch="main"' --features "gsfs"
else ifeq ($(shell cat /tmp/scmscx.com/ENABLE_GSFS), 2)
	cargo build --release --bin scmscx-com --config 'patch.crates-io.gsfs.path="../gsfs/rust/gsfs"' --features "gsfs"
else
	cargo build --release --bin scmscx-com
endif

package-lock.json node_modules &: package.json
	npm i
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

image-debug: dist/debug
	podman-compose build --build-arg PROFILE="debug"
image-release: dist/release
	podman-compose build --build-arg PROFILE="release"

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
	cargo clippy --workspace --all-targets -- \
		-A clippy::all \
		-D clippy::correctness \
		-D clippy::suspicious \
		-D clippy::complexity \
		-A clippy::clone-on-copy # I think .clone() is much clearer than deref + implcit copy

ci: check build test fmt clippy image-debug

run: image-debug
	podman-compose down
	podman-compose up

run-release: image-release
	podman-compose down
	podman-compose up

push: image-release
	podman push oni.zxcv.io/scmscx.com

dev:
	npm run dev

deploy:
	ssh -i~/.ssh/stan -C stan@urmom.zxcv.io sudo podman pull \
		oni.zxcv.io/scmscx.com
	ssh -i~/.ssh/stan -C stan@urmom.zxcv.io sudo systemctl restart \
		container-scmscx.com-S1

update: /tmp/scmscx.com/ENABLE_GSFS
ifeq ($(ENABLE_GSFS), 1)
	cargo update --config 'patch.crates-io.gsfs.git="ssh://git@github.com/zzlk/gsfs.git"' --config 'patch.crates-io.gsfs.tag="scmscx.com"'
else ifeq ($(ENABLE_GSFS), 2)
	cargo update --config 'patch.crates-io.gsfs.path="../gsfs/rust/gsfs"'
else
	cargo update
endif

.PHONY: .phony check build test fmt clippy ci run run-release push dev deploy update
