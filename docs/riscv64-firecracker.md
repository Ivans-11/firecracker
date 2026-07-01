# RISC-V Firecracker

This document describes how to get, build, and run the experimental RISC-V
Firecracker binary.

## Download a release binary

Download the RISC-V binary and checksum file from a release page. For example:

```bash
curl -LO https://github.com/Ivans-11/firecracker/releases/download/firecracker-riscv64-v0.1.1/firecracker-riscv64gc-unknown-linux-musl
curl -LO https://github.com/Ivans-11/firecracker/releases/download/firecracker-riscv64-v0.1.1/SHA256SUMS
sha256sum -c SHA256SUMS
chmod +x firecracker-riscv64gc-unknown-linux-musl
```

Use the downloaded binary as `FC_BIN` when starting a guest.

## Build from source

Clone this repository on a machine that has the RISC-V Rust target and a
RISC-V musl linker available:

```bash
git clone https://github.com/Ivans-11/firecracker.git
cd firecracker
rustup target add riscv64gc-unknown-linux-musl
tools/riscv64-release-firecracker.sh
```

The build output is placed under:

```text
release-riscv64gc-unknown-linux-musl/
```

The default binary name is:

```text
firecracker-riscv64gc-unknown-linux-musl
```

The script also writes a `SHA256SUMS` file in the same directory.
When run from the repository root, `tools/riscv64-run-basic.sh` uses this release binary by default.

## Run a guest

RISC-V Firecracker must run on a RISC-V host with KVM support. The current user must be able to read and write `/dev/kvm`.

The helper script starts Firecracker without the API server and generates a
temporary VM configuration file:

```bash
KERNEL_IMAGE=/path/to/riscv64/Image \
INITRD_IMAGE=/path/to/initramfs.cpio.gz \
ROOTFS_IMAGE=/path/to/rootfs.ext4 \
FC_BIN=/path/to/firecracker-riscv64gc-unknown-linux-musl \
tools/riscv64-run-basic.sh
```

`ROOTFS_IMAGE` and `INITRD_IMAGE` are both optional, but at least one of them is required.

Other useful optional variables:

```text
VCPUS=1
MEM_MIB=128
BOOT_ARGS='console=ttyS0 reboot=k panic=1'
```

When `ROOTFS_IMAGE` is set and `BOOT_ARGS` does not already contain `root=`, the script appends `root=/dev/vda ro`.

## Manual config

You can also run the binary directly with a Firecracker JSON configuration:

```bash
/path/to/firecracker-riscv64gc-unknown-linux-musl \
  --no-api \
  --no-seccomp \
  --config-file /path/to/vm.json \
  --level Debug
```

A guest config usually can contain a RISC-V kernel image, an initrd, one block device, and a machine configuration:

```json
{
  "boot-source": {
    "kernel_image_path": "/path/to/riscv64/Image",
    "initrd_path": "/path/to/initramfs.cpio.gz",
    "boot_args": "console=ttyS0 reboot=k panic=1 init=/init root=/dev/vda rw"
  },
  "drives": [
    {
      "drive_id": "rootfs",
      "path_on_host": "/path/to/rootfs.ext4",
      "is_root_device": true,
      "is_read_only": false
    }
  ],
  "machine-config": {
    "vcpu_count": 1,
    "mem_size_mib": 128,
    "smt": false
  }
}
```

If you do not use an initrd, omit `initrd_path`. If you do not use a root filesystem drive, omit `drives` and adjust `boot_args` for your initrd.
