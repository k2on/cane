build:
	cargo build --release
install:
	make build && cp ./target/release/cane /usr/local/bin