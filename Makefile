.PHONY: fmt lint test run infra-up infra-down

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

run:
	cargo run -p api-server

infra-up:
	docker compose up -d postgres

infra-down:
	docker compose down
