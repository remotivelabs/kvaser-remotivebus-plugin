# Use environment variables from .env
OUT_BIN_BASE ?= build/bin
OUT_LIB_BASE ?= build/lib
OUT_INTERMEDIATE_BASE ?= build/intermediate

libs-%:
	@echo "Building Kvaser libraries for $*..."
	@mkdir -p $(OUT_LIB_BASE)/$*
	@mkdir -p $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$*
	@if [ ! -f $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$*/linuxcan-5.50.312.tar.gz ]; then \
		cd $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$* && \
		wget --content-disposition "https://www.kvaser.com/downloads-kvaser/?utm_source=software&utm_ean=7330130980754&utm_status=latest"; \
	fi
	@if [ ! -f $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$*/linuxcan/.unpacked ]; then \
		cd $(OUT_INTERMEDIATE_BASE)/kvaser-linuxcan/$* && \
		tar -zxf linuxcan-5.50.312.tar.gz && \
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
