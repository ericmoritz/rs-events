all: src/schema.rs


src/schema.rs:
	diesel print-schema users > src/schema.rs