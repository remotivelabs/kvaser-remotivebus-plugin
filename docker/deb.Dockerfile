FROM ubuntu:24.04

# Build environment interface contract defaults (overridden by .env)
ENV PACKAGE_NAME=kvaser-remotivebus-plugin
ENV OUT_BIN_BASE=build/bin
ENV OUT_INTERMEDIATE_BASE=build/intermediate
ENV OUT_PACKAGES_BASE=build/pkgs
ENV OUT_PKG_INSTALL_BASE=build/pkgs
ENV SRC_PKG_COMMON=distribution/common

ARG UID=1000
ARG GID=1000

RUN getent group $GID || groupadd -g $GID builder
RUN id -u $UID || useradd -m -u $UID -g $GID builder

# Avoid interactive prompts
ENV DEBIAN_FRONTEND=noninteractive

# Install dpkg-deb (comes with dpkg) and other common tools
RUN apt-get update && apt-get install -y \
    dpkg \
    build-essential \
    fakeroot \
    devscripts \
    && rm -rf /var/lib/apt/lists/*
