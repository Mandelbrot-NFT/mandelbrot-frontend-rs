#!make
-include .env
export $(shell sed 's/=.*//' .env)

build:
	npm install
	RUSTFLAGS=--cfg=web_sys_unstable_apis trunk build

build_render:
	cargo install trunk wasm-bindgen-cli
	npm install -g
	RUSTFLAGS=--cfg=web_sys_unstable_apis trunk build --release

run:
	RUSTFLAGS=--cfg=web_sys_unstable_apis trunk serve

run_release:
	(cd dist; http-server)
