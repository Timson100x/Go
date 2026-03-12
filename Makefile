.PHONY: all build test lint fmt up down

all: build

build:
	cargo build --release

test:
	cargo test --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt -- --check

fmt:
	cargo fmt

up:
	docker compose up -d

down:
	docker compose down
