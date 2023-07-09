build:
	RUSTFLAGS=--cfg=web_sys_unstable_apis trunk build --release

run:
	RUSTFLAGS=--cfg=web_sys_unstable_apis trunk serve

run_release:
	(cd dist; http-server)