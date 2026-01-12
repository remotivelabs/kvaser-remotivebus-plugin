# Use environment variables from .env
OUT_BIN_BASE ?= build/bin

# Map toolchain to architecture
TOOLCHAIN_amd64=x86_64-unknown-linux-gnu
TOOLCHAIN_arm64=aarch64-unknown-linux-gnu

# Cross-compilation setup
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

.PHONY: build-%
build-%:
	@mkdir -p $(OUT_BIN_BASE)/$*
	cargo build --release --target $(TOOLCHAIN_$*) --target-dir target/$*
	@cp target/$*/$(TOOLCHAIN_$*)/release/kvaser-remotivebus-plugin $(OUT_BIN_BASE)/$*/kvaser-remotivebus-plugin
	@echo "Built: $(OUT_BIN_BASE)/$*/kvaser-remotivebus-plugin"

.PHONY: clean
clean:
	@cargo clean
