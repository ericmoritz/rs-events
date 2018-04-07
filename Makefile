all: src/schema.rs test lint acceptance

doc:
	cargo doc

test: 
	cargo test

lint:
	cargo +nightly clippy

acceptance:
	@make -C tests bdd

up:
	@make -C tests up

## Files
src/schema.rs:
	diesel print-schema users > src/schema.rs
