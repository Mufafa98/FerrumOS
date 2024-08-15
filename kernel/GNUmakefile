# Nuke built-in rules and variables.
override MAKEFLAGS += -rR

# Default target.
.PHONY: all
all:
	cargo build --target x86_64-unknown-none
	cp target/x86_64-unknown-none/debug/ferrum_os kernel.elf

.PHONY: check
check:
	cargo check --target x86_64-unknown-none

# Remove object files and the final executable.
.PHONY: clean
clean:
	cargo clean
	rm -rf kernel.elf

.PHONY: distclean
distclean: clean
