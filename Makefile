all: src/schema.rs

src/schema.rs:
	diesel print-schema users > src/schema.rs

bdd:
	@make -C tests bdd

up:
	@make -C tests up