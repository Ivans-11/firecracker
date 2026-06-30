#!/usr/bin/env bash

# Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

FC_TOOLS_DIR=$(dirname "$(realpath "$0")")
FC_ROOT_DIR=$FC_TOOLS_DIR/..

TARGET=${TARGET:-riscv64gc-unknown-linux-musl}
PROFILE=${PROFILE:-release}
ASSET_DIR=${ASSET_DIR:-release-$TARGET}
ASSET_NAME=${ASSET_NAME:-firecracker-$TARGET}

cd "$FC_ROOT_DIR"

CARGO_OPTS=(build -p firecracker --target "$TARGET")
if [[ "$PROFILE" == "release" ]]; then
    CARGO_OPTS+=(--release)
elif [[ "$PROFILE" != "debug" ]]; then
    echo "unsupported PROFILE=$PROFILE; expected release or debug" >&2
    exit 1
fi

RUSTFLAGS="${RUSTFLAGS:-} -C target-feature=+crt-static" cargo "${CARGO_OPTS[@]}"

profile_dir=$PROFILE
if [[ "$PROFILE" == "debug" ]]; then
    profile_dir=debug
fi

binary="build/cargo_target/$TARGET/$profile_dir/firecracker"
mkdir -p "$ASSET_DIR"
cp "$binary" "$ASSET_DIR/$ASSET_NAME"
chmod +x "$ASSET_DIR/$ASSET_NAME"

(
    cd "$ASSET_DIR"
    sha256sum "$ASSET_NAME" > SHA256SUMS
)

file "$ASSET_DIR/$ASSET_NAME"
