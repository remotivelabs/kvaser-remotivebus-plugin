# Use environment variables from .env
OUT_BIN_BASE ?= build/bin
OUT_LIB_BASE ?= build/lib
OUT_INTERMEDIATE_BASE ?= build/intermediate

# Pin the linuxcan release. Kvaser's "latest" download link moves with every
# release, which changes the tarball name and breaks kvaser/linuxcan/canlib.patch.
# When bumping, update the patch to match the new release.
LINUXCAN_VERSION := 5.51.461
LINUXCAN_TARBALL := linuxcan_$(subst .,_,$(LINUXCAN_VERSION)).tar.gz
LINUXCAN_URL := https://pim.kvaser.com/var/assets/Product_Resources/7330130980754/$(LINUXCAN_VERSION)/$(LINUXCAN_TARBALL)

libs-%:
	@echo "Building Kvaser libraries for $*..."
	@mkdir -p $(OUT_LIB_BASE)/$*
	@mkdir -p $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$*
	@if [ ! -f $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$*/$(LINUXCAN_TARBALL) ]; then \
		cd $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$* && \
		wget -O $(LINUXCAN_TARBALL).tmp "$(LINUXCAN_URL)" && \
		mv $(LINUXCAN_TARBALL).tmp $(LINUXCAN_TARBALL); \
	fi
	@if [ ! -f $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$*/linuxcan/.unpacked ]; then \
		cd $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$* && \
		tar -zxf $(LINUXCAN_TARBALL) && \
		touch linuxcan/.unpacked; \
	fi
	@if [ ! -f $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$*/linuxcan/.patched ]; then \
		cd $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$*/linuxcan && \
		patch -p1 < /workspace/kvaser/linuxcan/canlib.patch && \
		touch .patched; \
	fi
	@echo "Building user libraries for $*..."
	@cd $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$*/linuxcan && \
		if [ "$*" = "amd64" ]; then \
			CC=x86_64-linux-gnu-gcc CXX=x86_64-linux-gnu-g++ make -j$(shell nproc) canlib linlib; \
		else \
			CC=aarch64-linux-gnu-gcc CXX=aarch64-linux-gnu-g++ make -j$(shell nproc) canlib linlib; \
		fi
	@cp $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$*/linuxcan/canlib/libcanlib.so* $(OUT_LIB_BASE)/$*/
	@cp $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$*/linuxcan/linlib/liblinlib.so* $(OUT_LIB_BASE)/$*/
	@mkdir -p $(OUT_LIB_BASE)/$*/include
	@cp $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$*/linuxcan/include/*.h $(OUT_LIB_BASE)/$*/include/
	@echo "Built Kvaser libraries: $(OUT_LIB_BASE)/$*/"

.PHONY: clean
clean:
	@rm -rf $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan
