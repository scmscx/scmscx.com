RUST_SOURCE = Cargo.toml Cargo.lock $(shell find crates/ -name "*.rs" -or -name "*.toml" | sed 's/ /\\ /g')
RUST_TARGET_DIR := target/x86_64-unknown-linux-gnu

$(RUST_TARGET_DIR)/debug/scmscx-com: $(RUST_SOURCE)
	cargo build --bin scmscx-com
$(RUST_TARGET_DIR)/release/scmscx-com: $(RUST_SOURCE)
	cargo build --release --bin scmscx-com

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
	podman compose build --build-arg PROFILE="debug"
image-release: dist/release
	podman compose build --build-arg PROFILE="release"

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
	podman compose down
	podman compose up

run-release: image-release
	podman compose down
	podman compose up

push: image-release
	podman push oni.zxcv.io/scmscx.com

dev:
	npm run dev

deploy:
	ssh -i~/.ssh/stan -C stan@urmom.zxcv.io sudo podman pull \
		oni.zxcv.io/scmscx.com
	ssh -i~/.ssh/stan -C stan@urmom.zxcv.io sudo systemctl restart \
		container-zxcv.io

.PHONY: check build test fmt clippy ci run push dev deploy run-release
