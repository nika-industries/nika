
run-fetcher:
    cargo run -p fetcher

build:
    cargo build

test:
    cargo nextest run

# run surrealdb
surreal:
	surreal start file:/tmp/nika_surreal_data --log=info --auth
# nuke surreal data in /tmp/surreal_data
wipe-surreal:
	rm -rf /tmp/nika_surreal_data
# run surrealdb migrations -- surreal must be running
apply-surreal:
	surrealdb-migrations apply
