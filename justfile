
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

# run all tests with nextest
test:
    cargo nextest run

# run nix checks
check:
	nix flake check -L

# run clippy on all targets
clippy:
	cargo clippy --all-targets

# run surrealdb
surreal:
	surreal start file:/tmp/nika_surreal_data --log=info --auth
# nuke surreal data in /tmp/surreal_data
wipe-surreal:
	rm -rf /tmp/nika_surreal_data
# run surrealdb migrations -- surreal must be running
apply-surreal:
	surrealdb-migrations apply

# run redis
redis: 
	redis-server
