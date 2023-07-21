#!make
-include .env
export $(shell sed 's/=.*//' .env)

build:
	npm install
	trunk build

build_render:
	cargo install trunk wasm-bindgen-cli
	npm install
	trunk build --release

run:
	trunk serve

run_release:
	(cd dist; http-server)
