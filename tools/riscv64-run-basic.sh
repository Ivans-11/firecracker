#!/usr/bin/env bash
# Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

usage() {
    cat <<'EOF'
Run a minimal RISC-V Firecracker guest on a RISC-V KVM host.

Required:
  KERNEL_IMAGE=/path/to/riscv64/Image

Optional:
  ROOTFS_IMAGE=/path/to/rootfs.ext4
  INITRD_IMAGE=/path/to/initramfs.cpio
  FC_BIN=/path/to/firecracker
  VCPUS=1
  MEM_MIB=128
  BOOT_ARGS='console=ttyS0 reboot=k panic=1'

Example:
  KERNEL_IMAGE=/opt/riscv/Image \
  ROOTFS_IMAGE=/opt/riscv/rootfs.ext4 \
  FC_BIN=build/cargo_target/riscv64gc-unknown-linux-musl/debug/firecracker \
  tools/riscv64-run-basic.sh
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
    exit 0
fi

if [[ -z "${KERNEL_IMAGE:-}" ]]; then
    usage >&2
    exit 2
fi

FC_BIN="${FC_BIN:-build/cargo_target/riscv64gc-unknown-linux-musl/debug/firecracker}"
VCPUS="${VCPUS:-1}"
MEM_MIB="${MEM_MIB:-128}"
BOOT_ARGS="${BOOT_ARGS:-console=ttyS0 reboot=k panic=1}"

if [[ "$(uname -m)" != "riscv64" ]]; then
    echo "This script must run on a riscv64 host with KVM." >&2
    exit 1
fi

if [[ ! -r /dev/kvm || ! -w /dev/kvm ]]; then
    echo "/dev/kvm is not readable and writable by the current user." >&2
    exit 1
fi

if [[ ! -x "$FC_BIN" ]]; then
    echo "Firecracker binary is not executable: $FC_BIN" >&2
    exit 1
fi

if [[ ! -f "$KERNEL_IMAGE" ]]; then
    echo "Kernel image does not exist: $KERNEL_IMAGE" >&2
    exit 1
fi

tmpdir="$(mktemp -d /tmp/fc-riscv64.XXXXXX)"
trap 'rm -rf "$tmpdir"' EXIT
config="$tmpdir/vm.json"

if [[ -n "${ROOTFS_IMAGE:-}" ]]; then
    if [[ ! -f "$ROOTFS_IMAGE" ]]; then
        echo "Rootfs image does not exist: $ROOTFS_IMAGE" >&2
        exit 1
    fi
    BOOT_ARGS="${BOOT_ARGS} root=/dev/vda ro"
    cat >"$config" <<EOF
{
  "boot-source": {
    "kernel_image_path": "$KERNEL_IMAGE",
    "boot_args": "$BOOT_ARGS"
  },
  "drives": [
    {
      "drive_id": "rootfs",
      "path_on_host": "$ROOTFS_IMAGE",
      "is_root_device": true,
      "is_read_only": false
    }
  ],
  "machine-config": {
    "vcpu_count": $VCPUS,
    "mem_size_mib": $MEM_MIB,
    "smt": false
  }
}
EOF
elif [[ -n "${INITRD_IMAGE:-}" ]]; then
    if [[ ! -f "$INITRD_IMAGE" ]]; then
        echo "Initrd image does not exist: $INITRD_IMAGE" >&2
        exit 1
    fi
    cat >"$config" <<EOF
{
  "boot-source": {
    "kernel_image_path": "$KERNEL_IMAGE",
    "initrd_path": "$INITRD_IMAGE",
    "boot_args": "$BOOT_ARGS"
  },
  "machine-config": {
    "vcpu_count": $VCPUS,
    "mem_size_mib": $MEM_MIB,
    "smt": false
  }
}
EOF
else
    echo "Set either ROOTFS_IMAGE or INITRD_IMAGE." >&2
    exit 2
fi

exec "$FC_BIN" --no-api --no-seccomp --config-file "$config" --level Debug
