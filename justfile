
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

tikv-docker:
	docker run \
		--rm \
		--name tikv1 \
		--network host \
		-v /etc/localtime:/etc/localtime:ro \
		--mount type=bind,source=./tikv_data,target=/data \
		pingcap/tikv:latest \
		--addr="0.0.0.0:20160" \
		--advertise-addr="127.0.0.1:20160" \
		--data-dir="/data/tikv1" \
		--pd="127.0.0.1:2379"

pd-docker:
	docker run \
		--rm \
		--name pd1 \
		--network host \
		-v /etc/localtime:/etc/localtime:ro \
		--mount type=bind,source=./tikv_data,target=/data \
		pingcap/pd:latest \
		--name="pd1" \
		--data-dir="/data/pd1" \
		--client-urls="http://0.0.0.0:2379" \
		--advertise-client-urls="http://127.0.0.1:2379" \
		--peer-urls="http://0.0.0.0:2380" \
		--advertise-peer-urls="http://127.0.0.1:2380" \
		--initial-cluster="pd1=http://127.0.0.1:2380"
