FROM ubuntu:24.04

# Build environment interface contract defaults (overridden by .env)
ENV PACKAGE_NAME=kvaser-remotivebus-plugin
ENV OUT_BIN_BASE=build/bin
ENV OUT_LIB_BASE=build/lib
ENV OUT_INTERMEDIATE_BASE=build/intermediate
ENV OUT_PACKAGES_BASE=build/packages
ENV SRC_PKG_COMMON=distribution/common

ARG UID=1000
ARG GID=1000

RUN apt-get update && apt-get install -y \
    passwd wget build-essential ca-certificates sudo \
    gcc-aarch64-linux-gnu gcc-x86-64-linux-gnu \
 && rm -rf /var/lib/apt/lists/*

# Create group if needed
RUN if ! getent group $GID >/dev/null; then \
        groupadd -g $GID builder; \
    else \
        groupname=$(getent group $GID | cut -d: -f1); \
        echo "Reusing existing group $groupname"; \
    fi

# Create builder user if needed (or re-use existing UID)
RUN if ! getent passwd builder >/dev/null; then \
        if getent passwd $UID >/dev/null; then \
            echo "UID $UID already exists, creating alias 'builder'"; \
            useradd -M -u $UID -g $GID -o -s /bin/bash builder; \
        else \
            useradd -m -u $UID -g $GID -s /bin/bash builder; \
        fi; \
    fi

USER builder
WORKDIR /home/builder
ENV HOME=/home/builder
