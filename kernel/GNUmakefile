# Nuke built-in rules and variables.
override MAKEFLAGS += -rR

# Default target.
.PHONY: debug
debug:
	cargo build --target x86_64-unknown-none
	cp target/x86_64-unknown-none/debug/ferrum_os kernel.elf

.PHONY: release
release:
	cargo build -r --target x86_64-unknown-none
	cp target/x86_64-unknown-none/release/ferrum_os kernel.elf


.PHONY: check
check:
	cargo check --target x86_64-unknown-none

.PHONY: doc
doc:
	cargo doc --target x86_64-unknown-none
.PHONY: test
test:
	cargo build --target x86_64-unknown-none --features test
	cp target/x86_64-unknown-none/debug/ferrum_os kernel.elf

# Remove object files and the final executable.
.PHONY: clean
clean:
	cargo clean
	rm -rf kernel.elf

.PHONY: distclean
distclean: clean
