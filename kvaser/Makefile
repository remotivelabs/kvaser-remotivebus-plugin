# Use environment variables from .env
# The download URL always serves the latest linuxcan release, so the upstream
# file name changes with every release. Save it under a fixed name instead.
LINUXCAN_URL := https://www.kvaser.com/downloads-kvaser/?utm_source=software&utm_ean=7330130980754&utm_status=latest
LINUXCAN_TARBALL := linuxcan.tar.gz
OUT_BIN_BASE ?= build/bin
OUT_LIB_BASE ?= build/lib
OUT_INTERMEDIATE_BASE ?= build/intermediate

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
