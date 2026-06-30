// Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::device_manager::mmio::MMIO_LEN;

/// Start of RAM on the RISC-V `virt` machine.
pub const DRAM_MEM_START: u64 = 0x8000_0000;
/// The maximum RAM size initially supported by the RISC-V layout.
pub const DRAM_MEM_MAX_SIZE: usize = 0x80_0000_0000;

/// Start of system memory.
pub const SYSTEM_MEM_START: u64 = DRAM_MEM_START;
/// Reserved space before the kernel image.
pub const SYSTEM_MEM_SIZE: u64 = 0x20_0000;

/// Kernel command line maximum size.
pub const CMDLINE_MAX_SIZE: usize = 2048;
/// Maximum size reserved for a generated device tree.
pub const FDT_MAX_SIZE: usize = 0x20_0000;

/// RISC-V interrupt IDs exposed through the platform interrupt controller.
pub const GSI_LEGACY_START: u32 = 1;
/// Number of legacy GSIs reserved for MMIO devices.
pub const GSI_LEGACY_NUM: u32 = 256;
/// Last legacy GSI.
pub const GSI_LEGACY_END: u32 = GSI_LEGACY_START + GSI_LEGACY_NUM - 1;
/// First GSI used by MSI after legacy GSI.
pub const GSI_MSI_START: u32 = GSI_LEGACY_END + 1;
/// The highest available GSI in KVM.
pub const GSI_MSI_END: u32 = 4095;
/// Number of GSI available for MSI.
pub const GSI_MSI_NUM: u32 = GSI_MSI_END - GSI_MSI_START + 1;

/// Start of the RISC-V AIA APLIC MMIO range.
pub const AIA_APLIC_MEM_START: u64 = 0x0d00_0000;
/// Size of the RISC-V AIA APLIC MMIO range.
pub const AIA_APLIC_MEM_SIZE: u64 = 0x8000;
/// Start of the RISC-V AIA IMSIC MMIO range.
pub const AIA_IMSIC_MEM_START: u64 = 0x2800_0000;
/// Size of one RISC-V AIA IMSIC MMIO page.
pub const AIA_IMSIC_MEM_SIZE: u64 = 0x1000;

/// Start of 32-bit MMIO space, matching the conventional RISC-V virt map.
pub const MMIO32_MEM_START: u64 = 0x1000_0000;
/// Size of 32-bit MMIO space below DRAM.
pub const MMIO32_MEM_SIZE: u64 = DRAM_MEM_START - MMIO32_MEM_START;

/// Memory region start for boot device.
pub const BOOT_DEVICE_MEM_START: u64 = MMIO32_MEM_START;
/// Memory region start for Serial device.
pub const SERIAL_MEM_START: u64 = BOOT_DEVICE_MEM_START + MMIO_LEN;
/// Beginning of memory region for device MMIO 32-bit accesses.
pub const MEM_32BIT_DEVICES_START: u64 = SERIAL_MEM_START + MMIO_LEN;
/// Size of MMIO region reserved for PCIe configuration accesses.
pub const PCI_MMCONFIG_SIZE: u64 = 256 << 20;
/// Start of MMIO region reserved for PCIe configuration accesses.
pub const PCI_MMCONFIG_START: u64 = DRAM_MEM_START - PCI_MMCONFIG_SIZE;
/// MMIO space per PCIe segment.
pub const PCI_MMIO_CONFIG_SIZE_PER_SEGMENT: u64 = 4096 * 256;
/// Size of memory region for device MMIO 32-bit accesses.
pub const MEM_32BIT_DEVICES_SIZE: u64 = PCI_MMCONFIG_START - MEM_32BIT_DEVICES_START;

/// The start of the memory area reserved for MMIO 64-bit accesses.
pub const MMIO64_MEM_START: u64 = 256 << 30;
/// The size of the memory area reserved for MMIO 64-bit accesses.
pub const MMIO64_MEM_SIZE: u64 = 256 << 30;
/// Beginning of memory region for device MMIO 64-bit accesses.
pub const MEM_64BIT_DEVICES_START: u64 = MMIO64_MEM_START;
/// Size of memory region for device MMIO 64-bit accesses.
pub const MEM_64BIT_DEVICES_SIZE: u64 = MMIO64_MEM_SIZE;
/// First address past the 64-bit MMIO gap.
pub const FIRST_ADDR_PAST_64BITS_MMIO: u64 = MMIO64_MEM_START + MMIO64_MEM_SIZE;
/// Size of the memory past 64-bit MMIO gap.
pub const PAST_64BITS_MMIO_SIZE: u64 = 512 << 30;
