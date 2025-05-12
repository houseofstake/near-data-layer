.PHONY: build protogen stream_db_out run_sink setup clean

build:
	cargo build --target wasm32-unknown-unknown --release

protogen:
	substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google/"

stream_db_out:
	substreams run -e $(ENDPOINT) substreams.yaml db_out -s -10

setup_sink:
	substreams-sink-sql setup $(DSN) ./sink/substreams.dev.yaml

run_sink:
	RUST_LOG=debug substreams-sink-sql run $(DSN) sink/substreams.dev.yaml --endpoint $(ENDPOINT) --on-module-hash-mistmatch=ignore

clean:
	cargo clean
