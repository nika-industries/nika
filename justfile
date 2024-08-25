
_:
	@just -l

# run the fetcher binary
fetcher:
    cargo run --bin fetcher

# run the fetcher binary and watch for changes
watch-fetcher:
	bacon -j run -- --bin fetcher

# run the api binary
api:
	cargo run --bin api

# run the api binary and watch for changes
watch-api:
	bacon -j run -- --bin api

# run the daemon binary
daemon:
	cargo run --bin daemon

# run the daemon binary and watch for changes
watch-daemon:
	bacon -j run -- --bin daemon

# run tests with nextest
test:
    cargo nextest run

# run all tests, including ones that require a running redis instance
test-all:
	cargo nextest run --run-ignored all

# run nix checks
check:
	nix flake check -L

# run clippy on all targets
clippy:
	cargo clippy --all-targets

# run redis
redis: 
	redis-server

tikv:
	docker compose -f tikv_compose.yaml up

migrate:
    cargo run --bin migrator
