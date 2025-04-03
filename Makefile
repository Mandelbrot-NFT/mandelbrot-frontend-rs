#!make
-include .env
export $(shell sed 's/=.*//' .env)

dirs:
	mkdir -p ./mandelbrot-explorer-rs/target

build: dirs
	cargo build
	trunk build

build_render: dirs
	npm install -g sass
	cargo install trunk wasm-bindgen-cli
	cargo build --release
	trunk build --release

run: dirs
	trunk serve

run_release:
	(cd dist; http-server)
