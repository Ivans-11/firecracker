// Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::ffi::CString;

use vm_fdt::{Error as VmFdtError, FdtWriter};
use vm_memory::{GuestMemoryError, GuestMemoryRegion};

use super::layout;
use super::vcpu::RiscvIsaExtensions;
use crate::device_manager::DeviceManager;
use crate::device_manager::mmio::MMIODeviceInfo;
use crate::initrd::InitrdConfig;
use crate::vstate::memory::{Address, GuestMemory, GuestMemoryMmap, GuestRegionType};

const ADDRESS_CELLS: u32 = 2;
const SIZE_CELLS: u32 = 2;
const APLIC_PHANDLE: u32 = 1;
const IMSIC_PHANDLE: u32 = 2;
const CPU_INTC_PHANDLE_BASE: u32 = 0x100;
const IRQ_TYPE_EDGE_RISING: u32 = 1;
const RISCV_SUPERVISOR_EXTERNAL_IRQ: u32 = 9;
const TIMEBASE_FREQUENCY: u32 = 10_000_000;
const UART_CLOCK_FREQUENCY: u32 = 3_686_400;

/// RISC-V platform features that are safe to expose to the guest.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RiscvPlatformFeatures {
    /// Expose the AIA interrupt controller nodes.
    pub aia: bool,
    /// Expose CPU ISA extensions.
    pub isa: RiscvIsaExtensions,
}

/// Errors thrown while configuring the flattened device tree for riscv64.
#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum FdtError {
    /// Create FDT error: {0}
    CreateFdt(#[from] VmFdtError),
    /// Failure in writing FDT in memory.
    WriteFdtToMemory(#[from] GuestMemoryError),
}

/// Creates the flattened device tree for this riscv64 microVM.
pub fn create_fdt(
    guest_mem: &GuestMemoryMmap,
    vcpu_count: u8,
    cmdline: CString,
    device_manager: &DeviceManager,
    initrd: &Option<InitrdConfig>,
    features: RiscvPlatformFeatures,
) -> Result<Vec<u8>, FdtError> {
    let mut fdt = FdtWriter::new()?;

    let root = fdt.begin_node("")?;
    fdt.property_string("compatible", "linux,dummy-virt")?;
    fdt.property_string("model", "firecracker,riscv64-virt")?;
    fdt.property_u32("#address-cells", ADDRESS_CELLS)?;
    fdt.property_u32("#size-cells", SIZE_CELLS)?;
    create_cpu_nodes(&mut fdt, vcpu_count, features.isa)?;
    create_memory_node(&mut fdt, guest_mem)?;
    create_chosen_node(&mut fdt, cmdline, device_manager, initrd)?;
    if features.aia {
        create_aia_nodes(&mut fdt, vcpu_count)?;
    }
    create_devices_node(&mut fdt, device_manager, features.aia)?;

    fdt.end_node(root)?;
    Ok(fdt.finish()?)
}

fn cpu_intc_phandle(cpu_index: u8) -> u32 {
    CPU_INTC_PHANDLE_BASE + u32::from(cpu_index)
}

fn create_cpu_nodes(
    fdt: &mut FdtWriter,
    vcpu_count: u8,
    isa: RiscvIsaExtensions,
) -> Result<(), FdtError> {
    let cpus = fdt.begin_node("cpus")?;
    fdt.property_u32("#address-cells", 1)?;
    fdt.property_u32("#size-cells", 0)?;
    fdt.property_u32("timebase-frequency", TIMEBASE_FREQUENCY)?;

    for cpu_index in 0..vcpu_count {
        let cpu = fdt.begin_node(&format!("cpu@{cpu_index:x}"))?;
        fdt.property_string("device_type", "cpu")?;
        fdt.property_string("compatible", "riscv")?;
        fdt.property_string("riscv,isa", riscv_isa_string(isa).as_str())?;
        fdt.property_string("mmu-type", "riscv,sv48")?;
        fdt.property_string("status", "okay")?;
        fdt.property_u32("reg", u32::from(cpu_index))?;

        let intc = fdt.begin_node("interrupt-controller")?;
        fdt.property_null("interrupt-controller")?;
        fdt.property_u32("#interrupt-cells", 1)?;
        fdt.property_u32("phandle", cpu_intc_phandle(cpu_index))?;
        fdt.property_string("compatible", "riscv,cpu-intc")?;
        fdt.end_node(intc)?;

        fdt.end_node(cpu)?;
    }

    fdt.end_node(cpus)?;
    Ok(())
}

fn riscv_isa_string(isa: RiscvIsaExtensions) -> String {
    let mut isa_string = String::from("rv64imafdc_zicsr_zifencei");
    if isa.ssaia {
        isa_string.push_str("_ssaia");
    }
    if isa.sstc {
        isa_string.push_str("_sstc");
    }
    isa_string
}

fn create_memory_node(fdt: &mut FdtWriter, guest_mem: &GuestMemoryMmap) -> Result<(), FdtError> {
    let dram_region = guest_mem
        .iter()
        .find(|region| region.region_type == GuestRegionType::Dram)
        .unwrap();

    let start_addr = dram_region.start_addr();
    let mem_reg_prop = &[start_addr.raw_value(), dram_region.len()];
    let mem = fdt.begin_node(&format!("memory@{:x}", start_addr.raw_value()))?;
    fdt.property_string("device_type", "memory")?;
    fdt.property_array_u64("reg", mem_reg_prop)?;
    fdt.end_node(mem)?;
    Ok(())
}

fn create_chosen_node(
    fdt: &mut FdtWriter,
    cmdline: CString,
    device_manager: &DeviceManager,
    initrd: &Option<InitrdConfig>,
) -> Result<(), FdtError> {
    let chosen = fdt.begin_node("chosen")?;
    let cmdline_string = cmdline
        .into_string()
        .map_err(|_| VmFdtError::InvalidString)?;
    fdt.property_string("bootargs", cmdline_string.as_str())?;

    if let Some(serial_info) = device_manager.mmio_devices.serial_device_info() {
        fdt.property_string("stdout-path", &format!("/uart@{:x}", serial_info.addr))?;
    }

    if let Some(initrd_config) = initrd {
        fdt.property_u64("linux,initrd-start", initrd_config.address.raw_value())?;
        fdt.property_u64(
            "linux,initrd-end",
            initrd_config.address.raw_value() + initrd_config.size as u64,
        )?;
    }

    fdt.end_node(chosen)?;
    Ok(())
}

fn create_aia_nodes(fdt: &mut FdtWriter, vcpu_count: u8) -> Result<(), FdtError> {
    let imsic = fdt.begin_node(&format!(
        "interrupt-controller@{:x}",
        layout::AIA_IMSIC_MEM_START
    ))?;
    fdt.property_string_list(
        "compatible",
        vec!["qemu,imsics".to_string(), "riscv,imsics".to_string()],
    )?;
    fdt.property_null("interrupt-controller")?;
    fdt.property_null("msi-controller")?;
    fdt.property_u32("#interrupt-cells", 0)?;
    fdt.property_u32("phandle", IMSIC_PHANDLE)?;
    fdt.property_u32("riscv,guest-index-bits", layout::AIA_IMSIC_GUEST_INDEX_BITS)?;
    fdt.property_u32("riscv,num-ids", layout::AIA_IMSIC_NUM_IDS)?;
    fdt.property_array_u64(
        "reg",
        &[
            layout::AIA_IMSIC_MEM_START,
            layout::AIA_IMSIC_MEM_SIZE * u64::from(vcpu_count),
        ],
    )?;

    let mut interrupts_extended = Vec::with_capacity(vcpu_count as usize * 2);
    for cpu_index in 0..vcpu_count {
        let cpu_phandle = cpu_intc_phandle(cpu_index);
        interrupts_extended.push(cpu_phandle);
        interrupts_extended.push(RISCV_SUPERVISOR_EXTERNAL_IRQ);
    }
    fdt.property_array_u32("interrupts-extended", interrupts_extended.as_slice())?;
    fdt.end_node(imsic)?;

    let aplic = fdt.begin_node(&format!(
        "interrupt-controller@{:x}",
        layout::AIA_APLIC_MEM_START
    ))?;
    fdt.property_string_list(
        "compatible",
        vec!["qemu,aplic".to_string(), "riscv,aplic".to_string()],
    )?;
    fdt.property_null("interrupt-controller")?;
    fdt.property_u32("#interrupt-cells", 2)?;
    fdt.property_u32("#address-cells", 0)?;
    fdt.property_u32("phandle", APLIC_PHANDLE)?;
    fdt.property_u32("msi-parent", IMSIC_PHANDLE)?;
    fdt.property_u32("riscv,num-sources", layout::GSI_LEGACY_NUM)?;
    fdt.property_array_u64(
        "reg",
        &[layout::AIA_APLIC_MEM_START, layout::AIA_APLIC_MEM_SIZE],
    )?;
    fdt.end_node(aplic)?;
    Ok(())
}

fn create_virtio_node(
    fdt: &mut FdtWriter,
    dev_info: &MMIODeviceInfo,
    use_aia: bool,
) -> Result<(), FdtError> {
    let virtio_mmio = fdt.begin_node(&format!("virtio_mmio@{:x}", dev_info.addr))?;
    fdt.property_null("dma-coherent")?;
    fdt.property_string("compatible", "virtio,mmio")?;
    fdt.property_array_u64("reg", &[dev_info.addr, dev_info.len])?;
    if use_aia {
        fdt.property_u32("interrupt-parent", APLIC_PHANDLE)?;
        fdt.property_array_u32("interrupts", &[dev_info.gsi.unwrap(), IRQ_TYPE_EDGE_RISING])?;
    }
    fdt.end_node(virtio_mmio)?;
    Ok(())
}

fn create_serial_node(
    fdt: &mut FdtWriter,
    dev_info: &MMIODeviceInfo,
    use_aia: bool,
) -> Result<(), FdtError> {
    let serial = fdt.begin_node(&format!("uart@{:x}", dev_info.addr))?;
    fdt.property_string("compatible", "ns16550a")?;
    fdt.property_array_u64("reg", &[dev_info.addr, dev_info.len])?;
    fdt.property_u32("clock-frequency", UART_CLOCK_FREQUENCY)?;
    fdt.property_u32("current-speed", 115_200)?;
    if use_aia {
        fdt.property_u32("interrupt-parent", APLIC_PHANDLE)?;
        fdt.property_array_u32("interrupts", &[dev_info.gsi.unwrap(), IRQ_TYPE_EDGE_RISING])?;
    }
    fdt.end_node(serial)?;
    Ok(())
}

fn create_devices_node(
    fdt: &mut FdtWriter,
    device_manager: &DeviceManager,
    use_aia: bool,
) -> Result<(), FdtError> {
    if let Some(serial_info) = device_manager.mmio_devices.serial_device_info() {
        create_serial_node(fdt, serial_info, use_aia)?;
    }

    let mut virtio_mmio = device_manager.mmio_devices.virtio_device_info();
    virtio_mmio.sort_by_key(|info| info.addr);
    for device_info in virtio_mmio {
        create_virtio_node(fdt, device_info, use_aia)?;
    }

    Ok(())
}
