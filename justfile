
_:
	@just -l

# run the fetcher binary
fetcher:
    cargo run --bin fetcher

# run the fetcher binary and watch for changes
watch-fetcher:
	bacon -j run -- --bin fetcher

# run the fetcher binary in release mode
fetcher-release:
	cargo run --bin fetcher --release

# run the api binary
api:
	cargo run --bin api

# run the api binary and watch for changes
watch-api:
	bacon -j run -- --bin api

# run the api binary in release mode
api-release:
    cargo run --bin api --release

# run the daemon binary
daemon:
	cargo run --bin daemon

# run the daemon binary and watch for changes
watch-daemon:
	bacon -j run -- --bin daemon

# run the daemon binary in release mode
daemon-release:
    cargo run --bin daemon --release

# run the whole stack
stack:
    mprocs "just run-tikv" "just run-pd" "just redis" "just fetcher" "just api" # "just daemon"

# run the whole stack in release mode
stack-release:
    mprocs "just run-tikv" "just run-pd" "just redis" "just fetcher-release" "just api-release" # "just daemon"

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

# run the tikv stack
tikv:
	mprocs "just run-tikv" "just run-pd"

# run the tikv server in a container
run-tikv:
	cat $(nix build .#tikv-image --print-out-paths) | docker load
	docker run --rm --network host tikv-server:8.1.1
# run the pd server in a container
run-pd:
	cat $(nix build .#pd-image --print-out-paths) | docker load
	docker run --rm --network host pd-server:8.1.1

migrate:
    cargo run --bin migrator
