# RISC-V Firecracker

本文说明如何获取、构建和运行实验性的 RISC-V Firecracker 二进制程序。

## 从 Release 下载

可以从 Release 页面下载 RISC-V 二进制和校验文件。例如：

```bash
curl -LO https://github.com/Ivans-11/firecracker/releases/download/firecracker-riscv64-v0.1.1/firecracker-riscv64gc-unknown-linux-musl
curl -LO https://github.com/Ivans-11/firecracker/releases/download/firecracker-riscv64-v0.1.1/SHA256SUMS
sha256sum -c SHA256SUMS
chmod +x firecracker-riscv64gc-unknown-linux-musl
```

启动 guest 时，将下载得到的二进制作为 `FC_BIN` 使用。

## 从源码构建

在安装了 RISC-V Rust target 和 RISC-V musl linker 的机器上克隆仓库：

```bash
git clone https://github.com/Ivans-11/firecracker.git
cd firecracker
rustup target add riscv64gc-unknown-linux-musl
tools/riscv64-release-firecracker.sh
```

构建产物会放在：

```text
release-riscv64gc-unknown-linux-musl/
```

默认二进制文件名是：

```text
firecracker-riscv64gc-unknown-linux-musl
```

脚本还会在同一目录下生成 `SHA256SUMS`。
从仓库根目录运行时，`tools/riscv64-run-basic.sh` 默认会使用这个 release 二进制。

## 运行 guest

RISC-V Firecracker 需要运行在支持 KVM 的 RISC-V host 上。当前用户需要具备 `/dev/kvm` 的读写权限。

辅助脚本会以无 API server 的方式启动 Firecracker，并生成临时 VM 配置文件：

```bash
KERNEL_IMAGE=/path/to/riscv64/Image \
INITRD_IMAGE=/path/to/initramfs.cpio.gz \
ROOTFS_IMAGE=/path/to/rootfs.ext4 \
FC_BIN=/path/to/firecracker-riscv64gc-unknown-linux-musl \
tools/riscv64-run-basic.sh
```

`ROOTFS_IMAGE` 和 `INITRD_IMAGE` 都是可选项，但要求至少设置其中一个。

常用其他可选变量：

```text
VCPUS=1
MEM_MIB=128
BOOT_ARGS='console=ttyS0 reboot=k panic=1'
```

如果设置了 `ROOTFS_IMAGE`，且 `BOOT_ARGS` 中还没有 `root=`，脚本会追加 `root=/dev/vda ro`。

## 手动配置

也可以直接传入 Firecracker JSON 配置文件运行：

```bash
/path/to/firecracker-riscv64gc-unknown-linux-musl \
  --no-api \
  --no-seccomp \
  --config-file /path/to/vm.json \
  --level Debug
```

guest 配置通常可以包含 RISC-V kernel image、initrd、一个 block device 和 machine configuration：

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

如果不使用 initrd，可以省略 `initrd_path`。如果不使用 root filesystem drive，可以省略 `drives`，并按 initrd 的启动方式调整 `boot_args`。
