#!/bin/bash
set -e

# Source build environment variables
source .env

DISTRO=${1:-fedora}
VERSION=${2:-41}
ARCH=${3:-amd64}
PACKAGE_NAME=${4:-kvaser-remotivebus-plugin}

# Source version from Cargo.toml
PACKAGE_VERSION=$(grep '^version' Cargo.toml | cut -d'"' -f2)

# Architecture mapping for RPM
case "$ARCH" in
    amd64) RPM_ARCH="x86_64" ;;
    arm64) RPM_ARCH="aarch64" ;;
    *) RPM_ARCH="$ARCH" ;;
esac

# Dependency mapping based on distro version
case "$DISTRO-$VERSION" in
    fedora-40|fedora-41)
        DEPENDENCIES="remotivebusd, can-utils, systemd"
        ;;
    centos-9|rocky-9)
        DEPENDENCIES="remotivebusd, can-utils, systemd"
        ;;
    *)
        echo "Warning: Unknown distro version $DISTRO-$VERSION, using default dependencies"
        DEPENDENCIES="remotivebusd, can-utils, systemd"
        ;;
esac

echo "Building RPM package for $DISTRO $VERSION $ARCH"

# Create build directories
BUILD_DIR="$OUT_INTERMEDIATE_BASE/rpm/$ARCH"
INSTALL_DIR="$OUT_PACKAGES_BASE/rpm"

mkdir -p "$BUILD_DIR/SOURCES"
mkdir -p "$BUILD_DIR/SPECS"
mkdir -p "$BUILD_DIR/BUILD"
mkdir -p "$BUILD_DIR/RPMS"
mkdir -p "$BUILD_DIR/SRPMS"
mkdir -p "$INSTALL_DIR"

# Copy source files
cp "$OUT_BIN_BASE/$ARCH/kvaser-remotivebus-plugin" "$BUILD_DIR/SOURCES/"
cp "$SRC_PKG_COMMON/kvaser-remotivebus-plugin.service" "$BUILD_DIR/SOURCES/"

# Process template
sed -e "s|{{PACKAGE_NAME}}|$PACKAGE_NAME|g" \
    -e "s|{{VERSION}}|$PACKAGE_VERSION|g" \
    -e "s|{{DEPENDENCIES}}|$DEPENDENCIES|g" \
    "distribution/templates/fedora/spec.template" > "$BUILD_DIR/SPECS/$PACKAGE_NAME.spec"

# Build RPM
QA_RPATHS=0x0002 rpmbuild \
    --define "_topdir $(pwd)/$BUILD_DIR" \
    --define "_sourcedir $(pwd)/$BUILD_DIR/SOURCES" \
    --target "$RPM_ARCH" \
    -bb "$BUILD_DIR/SPECS/$PACKAGE_NAME.spec"

# Copy built RPM to install directory
find "$BUILD_DIR/RPMS" -name "*.rpm" -exec cp {} "$INSTALL_DIR/" \;

echo "Package built: $INSTALL_DIR/${PACKAGE_NAME}-${PACKAGE_VERSION}-1*.rpm"
