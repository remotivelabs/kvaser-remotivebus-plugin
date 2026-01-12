# Load environment variables from .env automatically
SHELL := /bin/bash

# Default values (will be overridden by .env when sourced)
PACKAGE_NAME ?= kvaser-remotivebus-plugin
VERSION := $(shell grep ^version Cargo.toml | grep -Eo "[0-9]+\.[0-9]+\.[0-9]+")

OUT_BUILD_BASE ?= build
OUT_INTERMEDIATE_BASE ?= $(OUT_BUILD_BASE)/intermediate
OUT_BIN_BASE ?= $(OUT_BUILD_BASE)/bin
OUT_LIB_BASE ?= $(OUT_BUILD_BASE)/lib
OUT_PACKAGES_BASE ?= $(OUT_BUILD_BASE)/pkgs
OUT_INSTALL_BASE ?= $(OUT_BUILD_BASE)/installer

.PHONY: help
help:
	@echo "Kvaser RemotiveBus Plugin Build System"
	@echo "======================================"
	@echo
	@echo "Main targets:"
	@echo "  all                     - Build packages for all platforms (deb + rpm, amd64 + arm64)"
	@echo "  clean                   - Clean build artifacts (build/ directory)"
	@echo "  prune                   - Clean build artifacts and remove Docker images"
	@echo "  help                    - Show this help message"
	@echo "  version                 - Show current plugin version"
	@echo
	@echo "Package generation:"
	@echo "  pkg-amd64-deb           - Build .deb package for amd64"
	@echo "  pkg-arm64-deb           - Build .deb package for arm64"
	@echo "  pkg-amd64-rpm           - Build .rpm package for amd64"
	@echo "  pkg-arm64-rpm           - Build .rpm package for arm64"
	@echo
	@echo "Plugin binary builds:"
	@echo "  build-amd64-plugin      - Build plugin binary for amd64"
	@echo "  build-arm64-plugin      - Build plugin binary for arm64"
	@echo
	@echo "Kvaser library builds:"
	@echo "  build-amd64-kvaser-libs - Build Kvaser LinuxCAN libraries for amd64"
	@echo "  build-arm64-kvaser-libs - Build Kvaser LinuxCAN libraries for arm64"
	@echo "  install-kvaser-libs     - Install Kvaser libraries (default: /usr/lib, override: INSTALL_DIR=/path)"
	@echo
	@echo "Docker containers:"
	@echo "  .deb-docker             - Build deb-builder container"
	@echo "  .rpm-docker             - Build rpm-builder container"
	@echo "  .app-docker             - Build app-builder container"
	@echo "  .kvaser-docker          - Build kvaser-builder container"
	@echo
	@echo "Note: All builds use Docker containers with architecture isolation to prevent caching conflicts."
	@echo

.PHONY: version
version:
	@echo $(VERSION)

#
# Build targets
#

.PHONY: all
all: pkg-amd64-deb pkg-arm64-deb pkg-amd64-rpm pkg-arm64-rpm installer

# Install Kvaser libraries for current architecture to INSTALL_DIR (default: /usr/lib)
INSTALL_DIR ?= /usr/lib
CURRENT_ARCH := $(shell uname -m)
ARCH_DIR := $(shell case "$(CURRENT_ARCH)" in (x86_64) echo amd64;; (aarch64) echo arm64;; (*) echo "Unsupported: $(CURRENT_ARCH)" >&2; exit 1;; esac)

.PHONY: install-kvaser-libs
install-kvaser-libs: build-$(ARCH_DIR)-kvaser-libs
	@echo "Installing Kvaser libraries to $(INSTALL_DIR)..."
	@case "$(INSTALL_DIR)" in \
		/lib|/lib/*|/usr/lib|/usr/lib/*|/usr/local/lib|/usr/local/lib/*) \
			sudo cp -v $(OUT_LIB_BASE)/$(ARCH_DIR)/libcanlib.so* $(INSTALL_DIR)/; \
			sudo cp -v $(OUT_LIB_BASE)/$(ARCH_DIR)/liblinlib.so* $(INSTALL_DIR)/; \
			sudo ldconfig; \
			;; \
		*) \
			mkdir -p $(INSTALL_DIR); \
			cp -v $(OUT_LIB_BASE)/$(ARCH_DIR)/libcanlib.so* $(INSTALL_DIR)/; \
			cp -v $(OUT_LIB_BASE)/$(ARCH_DIR)/liblinlib.so* $(INSTALL_DIR)/; \
			;; \
	esac
	@echo "Installed to $(INSTALL_DIR). Use KVASER_LIB_PATH=$(INSTALL_DIR) for cargo."

.PHONY:
build-%-plugin: build-%-kvaser-libs .plugin-docker
	@[ -f .env ] && source .env; docker run --rm \
		-v "$$PWD:/workspace" \
		-w /workspace \
		-e OUT_BIN_BASE \
		-e OUT_LIB_BASE \
		-e OUT_INTERMEDIATE_BASE \
		-e OUT_PACKAGES_BASE \
		-e SRC_PKG_COMMON \
		-e PACKAGE_NAME \
		--user $(shell id -u):$(shell id -g) \
		kvaser-plugin-builder \
		make -f plugin.Makefile build-$*

build-%-kvaser-libs: .kvaser-docker
	@[ -f .env ] && source .env; docker run --rm \
		-v "$$PWD:/workspace" \
		-w /workspace \
		-e OUT_BIN_BASE \
		-e OUT_LIB_BASE \
		-e OUT_INTERMEDIATE_BASE \
		-e OUT_PACKAGES_BASE \
		-e SRC_PKG_COMMON \
		-e PACKAGE_NAME \
		--user $(shell id -u):$(shell id -g) \
		kvaser-libs-builder \
		make -f kvaser/Makefile libs-$*

pkg-%-deb: build-%-plugin .deb-docker
	@[ -f .env ] && source .env; docker run --rm \
		-v "$$PWD:/workspace" \
		-w /workspace \
		-e OUT_BIN_BASE \
		-e OUT_LIB_BASE \
		-e OUT_INTERMEDIATE_BASE \
		-e OUT_PACKAGES_BASE \
		-e SRC_PKG_COMMON \
		-e PACKAGE_NAME \
		--user $(shell id -u):$(shell id -g) \
		kvaser-deb-builder \
		distribution/scripts/build-deb.sh ubuntu 24.04 $*

pkg-%-rpm: build-%-plugin .rpm-docker
	@[ -f .env ] && source .env; docker run --rm \
		-v "$$PWD:/workspace" \
		-w /workspace \
		-e OUT_BIN_BASE \
		-e OUT_LIB_BASE \
		-e OUT_INTERMEDIATE_BASE \
		-e OUT_PACKAGES_BASE \
		-e SRC_PKG_COMMON \
		-e PACKAGE_NAME \
		--user $(shell id -u):$(shell id -g) \
		kvaser-rpm-builder \
		distribution/scripts/build-rpm.sh fedora 41 $*

.PHONY: installer
installer: pkg-amd64-deb pkg-arm64-deb pkg-amd64-rpm pkg-arm64-rpm
	@mkdir -p $(OUT_INSTALL_BASE)
	@cp distribution/installer/* $(OUT_INSTALL_BASE)
	@sed -i s/REPLACE_VERSION_NUMBER/$(VERSION)/g $(OUT_INSTALL_BASE)/Makefile
	@cp -R ${OUT_INSTALL_BASE}/* ${OUT_PACKAGES_BASE}
	@tar -czvf $(OUT_BUILD_BASE)/kvaser-remotivebus-plugin-$(VERSION).tar.gz -C $(OUT_INSTALL_BASE) .

#
# Docker container builds
#

.PHONY: .plugin-docker
.plugin-docker:
	docker build -t kvaser-plugin-builder --build-arg UID=$(shell id -u) --build-arg GID=$(shell id -g) -f docker/plugin.Dockerfile .

.PHONY: .deb-docker
.deb-docker:
	docker build -t kvaser-deb-builder --build-arg UID=$(shell id -u) --build-arg GID=$(shell id -g) -f docker/deb.Dockerfile .
.PHONY: .rpm-docker
.rpm-docker:
	docker build -t kvaser-rpm-builder --build-arg UID=$(shell id -u) --build-arg GID=$(shell id -g) -f docker/rpm.Dockerfile .

.PHONY: .kvaser-docker
.kvaser-docker:
	docker build -t kvaser-libs-builder --build-arg UID=$(shell id -u) --build-arg GID=$(shell id -g) -f kvaser/Dockerfile .
#
# Cleanup
#

.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf build

.PHONY: prune
prune: clean
	@echo "Removing Docker images..."
	@docker rmi -f \
		kvaser-plugin-builder \
		kvaser-deb-builder \
		kvaser-rpm-builder \
		kvaser-libs-builder \
		2>/dev/null || true
