.PHONY: build fmt format test install
fmt:
	cargo fmt
build:
	cargo build
disposable-pg:
	bash ./scripts/disposable_pg.sh
docker-compose-images:
	./scripts/docker_compose_build.sh
install:
	cargo build --release && cp ./target/release/pg-to-sqlite3 ~/.cargo/bin
