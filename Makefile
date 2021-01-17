.PHONY: build fmt format test
fmt:
	cargo fmt
build:
	cargo build
disposable-pg:
	bash ./scripts/disposable_pg.sh
docker-compose-images:
	./scripts/docker_compose_build.sh
