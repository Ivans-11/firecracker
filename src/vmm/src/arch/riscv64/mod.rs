// Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

/// Flattened device tree support.
mod fdt;
/// Architecture specific KVM-related code.
pub mod kvm;
/// Layout for this RISC-V system.
pub mod layout;
/// Architecture specific vCPU code.
pub mod vcpu;
/// Architecture specific VM state code.
pub mod vm;

use std::cmp::min;
use std::fmt::Debug;
use std::fs::File;

use kvm_ioctls::Cap;
use linux_loader::loader::pe::PE as Loader;
use linux_loader::loader::{Cmdline, KernelLoader};
use vm_memory::{GuestMemoryError, GuestMemoryRegion};

use crate::arch::{BootProtocol, EntryPoint, arch_memory_regions_with_gap};
use crate::cpu_config::riscv64::{CpuConfiguration, CpuConfigurationError};
use crate::cpu_config::templates::CustomCpuTemplate;
use crate::initrd::InitrdConfig;
use crate::utils::{align_up, u64_to_usize, usize_to_u64};
use crate::vmm_config::machine_config::MachineConfig;
use crate::vstate::memory::{
    Address, Bytes, GuestAddress, GuestMemory, GuestMemoryMmap, GuestRegionType,
};
use crate::vstate::vcpu::KvmVcpuError;
use crate::vstate::vm::KvmVm;
use crate::{DeviceManager, Vcpu, VcpuConfig, logger};

/// Errors thrown while configuring RISC-V system.
#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum ConfigurationError {
    /// Failed to write to guest memory.
    MemoryError(#[from] GuestMemoryError),
    /// Cannot copy kernel file fd.
    KernelFile,
    /// Cannot load kernel due to invalid memory configuration or invalid kernel image: {0}
    KernelLoader(#[from] linux_loader::loader::Error),
    /// Cannot configure the device tree: {0}
    SetupFdt(#[from] fdt::FdtError),
    /// Error creating vcpu configuration: {0}
    VcpuConfig(#[from] CpuConfigurationError),
    /// Error configuring the vcpu: {0}
    VcpuConfigure(#[from] KvmVcpuError),
}

/// Returns a Vec of the valid memory addresses for RISC-V.
pub fn arch_memory_regions(size: usize) -> Vec<(GuestAddress, usize)> {
    assert!(size > 0, "Attempt to allocate guest memory of length 0");

    let dram_size = min(size, layout::DRAM_MEM_MAX_SIZE);

    if dram_size != size {
        logger::warn!(
            "Requested memory size {} exceeds architectural maximum. Size has been truncated to {}",
            size,
            dram_size
        );
    }

    let mut regions = vec![];
    if let Some((offset, remaining)) = arch_memory_regions_with_gap(
        &mut regions,
        u64_to_usize(layout::DRAM_MEM_START),
        dram_size,
        u64_to_usize(layout::MMIO64_MEM_START),
        u64_to_usize(layout::MMIO64_MEM_SIZE),
    ) {
        regions.push((GuestAddress(offset as u64), remaining));
    }

    regions
}

/// Configures the system for booting Linux.
#[allow(clippy::too_many_arguments)]
pub fn configure_system_for_boot(
    _kvm: &crate::Kvm,
    vm: &KvmVm,
    device_manager: &mut DeviceManager,
    vcpus: &mut [Vcpu],
    machine_config: &MachineConfig,
    cpu_template: &CustomCpuTemplate,
    entry_point: EntryPoint,
    initrd: &Option<InitrdConfig>,
    boot_cmdline: Cmdline,
) -> Result<(), ConfigurationError> {
    let cpu_config = CpuConfiguration::new(cpu_template)?;
    let cpu_config = CpuConfiguration::apply_template(cpu_config, cpu_template);

    let vcpu_config = VcpuConfig {
        vcpu_count: machine_config.vcpu_count,
        smt: machine_config.smt,
        cpu_config,
    };

    for vcpu in vcpus.iter_mut() {
        vcpu.kvm_vcpu
            .configure(vm.guest_memory(), entry_point, &vcpu_config)?;
    }

    let cmdline = boot_cmdline
        .as_cstring()
        .expect("Cannot create cstring from cmdline string");
    let fdt = fdt::create_fdt(
        vm.guest_memory(),
        machine_config.vcpu_count,
        cmdline,
        device_manager,
        initrd,
        vm.common.fd.check_extension(Cap::DeviceCtrl),
    )?;
    vm.guest_memory().write_slice(
        fdt.as_slice(),
        GuestAddress(get_fdt_addr(vm.guest_memory())),
    )?;

    Ok(())
}

/// Returns the memory address where the kernel could be loaded.
pub fn get_kernel_start() -> u64 {
    layout::SYSTEM_MEM_START + layout::SYSTEM_MEM_SIZE
}

/// Returns the memory address where the initrd could be loaded.
pub fn initrd_load_addr(guest_mem: &GuestMemoryMmap, initrd_size: usize) -> Option<u64> {
    let rounded_size = align_up(
        usize_to_u64(initrd_size),
        usize_to_u64(super::GUEST_PAGE_SIZE),
    );
    GuestAddress(get_fdt_addr(guest_mem))
        .checked_sub(rounded_size)
        .filter(|&addr| guest_mem.address_in_range(addr))
        .map(|addr| addr.raw_value())
}

fn get_fdt_addr(mem: &GuestMemoryMmap) -> u64 {
    let dram_region = mem
        .iter()
        .find(|region| region.region_type == GuestRegionType::Dram)
        .unwrap();

    dram_region
        .last_addr()
        .checked_sub(layout::FDT_MAX_SIZE as u64 - 1)
        .filter(|&addr| mem.address_in_range(addr))
        .map(|addr| addr.raw_value())
        .unwrap_or(layout::DRAM_MEM_START)
}

/// Load Linux kernel into guest memory.
pub fn load_kernel(
    kernel: &File,
    guest_memory: &GuestMemoryMmap,
) -> Result<EntryPoint, ConfigurationError> {
    let mut kernel_file = kernel
        .try_clone()
        .map_err(|_| ConfigurationError::KernelFile)?;

    let entry_addr = Loader::load(
        guest_memory,
        Some(GuestAddress(get_kernel_start())),
        &mut kernel_file,
        None,
    )?;

    Ok(EntryPoint {
        entry_addr: entry_addr.kernel_load,
        protocol: BootProtocol::LinuxBoot,
    })
}
