#!/bin/bash
set -e

# Source build environment variables
source .env

DISTRO=${1:-ubuntu}
VERSION=${2:-24.04}
ARCH=${3:-amd64}
PACKAGE_NAME=${4:-kvaser-remotivebus-plugin}

# Source version from Cargo.toml
PACKAGE_VERSION=$(grep '^version' Cargo.toml | cut -d'"' -f2)

# Dependency mapping based on distro version
case "$DISTRO-$VERSION" in
    ubuntu-20.04)
        DEPENDENCIES="remotivebusd, can-utils, systemd"
        ;;
    ubuntu-22.04|ubuntu-24.04)
        DEPENDENCIES="remotivebusd, can-utils, systemd"
        ;;
    debian-11|debian-12)
        DEPENDENCIES="remotivebusd, can-utils, systemd"
        ;;
    *)
        echo "Warning: Unknown distro version $DISTRO-$VERSION, using default dependencies"
        DEPENDENCIES="remotivebusd, can-utils, systemd"
        ;;
esac

echo "Building DEB package for $DISTRO $VERSION $ARCH"

# Create build directories
BUILD_DIR="$OUT_INTERMEDIATE_BASE/deb/$ARCH"
INSTALL_DIR="$OUT_PACKAGES_BASE/deb"

mkdir -p "$BUILD_DIR/usr/bin"
mkdir -p "$BUILD_DIR/usr/lib/systemd/system"
mkdir -p "$BUILD_DIR/DEBIAN"
mkdir -p "$INSTALL_DIR"

# Copy files
cp "$OUT_BIN_BASE/$ARCH/kvaser-remotivebus-plugin" "$BUILD_DIR/usr/bin/"
cp "$SRC_PKG_COMMON/kvaser-remotivebus-plugin.service" "$BUILD_DIR/usr/lib/systemd/system/"

# Process templates
sed -e "s|{{PACKAGE_NAME}}|$PACKAGE_NAME|g" \
    -e "s|{{VERSION}}|$PACKAGE_VERSION|g" \
    -e "s|{{ARCH}}|$ARCH|g" \
    -e "s|{{DEPENDENCIES}}|$DEPENDENCIES|g" \
    "distribution/templates/ubuntu/control.template" > "$BUILD_DIR/DEBIAN/control"

# Copy and make executable post/pre scripts
cp "distribution/templates/ubuntu/postinst.template" "$BUILD_DIR/DEBIAN/postinst"
cp "distribution/templates/ubuntu/prerm.template" "$BUILD_DIR/DEBIAN/prerm"
chmod 755 "$BUILD_DIR/DEBIAN/postinst" "$BUILD_DIR/DEBIAN/prerm"

# Build package
PACKAGE_FILE="${PACKAGE_NAME}_${PACKAGE_VERSION}_${ARCH}.deb"
dpkg-deb --build "$BUILD_DIR" "$INSTALL_DIR/$PACKAGE_FILE"

echo "Package built: $INSTALL_DIR/$PACKAGE_FILE"
