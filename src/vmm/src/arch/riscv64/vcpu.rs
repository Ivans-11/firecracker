// Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use kvm_bindings::{KVM_REG_RISCV, KVM_REG_RISCV_CORE, KVM_REG_SIZE_U64};
use kvm_ioctls::{VcpuExit, VcpuFd};
use serde::{Deserialize, Serialize};

use crate::arch::EntryPoint;
use crate::cpu_config::riscv64::CpuConfiguration;
use crate::logger::{IncMetric, METRICS, error, info};
use crate::vcpu::{VcpuConfig, VcpuError};
use crate::vstate::bus::Bus;
use crate::vstate::memory::{Address, GuestMemoryMmap};
use crate::vstate::vcpu::VcpuEmulation;
use crate::vstate::vm::KvmVm;

const RISCV_CORE_REG_PC: u64 = 0;
const RISCV_CORE_REG_A0: u64 = 10;
const RISCV_CORE_REG_A1: u64 = 11;

fn riscv_core_reg_id(index: u64) -> u64 {
    KVM_REG_RISCV as u64 | KVM_REG_SIZE_U64 | u64::from(KVM_REG_RISCV_CORE) | index
}

/// Errors thrown while setting RISC-V registers.
#[derive(Debug, PartialEq, Eq, thiserror::Error, displaydoc::Display)]
pub enum VcpuArchError {
    /// Failed to set register {0:#x} to value {1}: {2}
    SetOneReg(u64, String, kvm_ioctls::Error),
}

/// Errors associated with the wrappers over KVM ioctls.
#[derive(Debug, PartialEq, Eq, thiserror::Error, displaydoc::Display)]
pub enum KvmVcpuError {
    /// Error creating vcpu: {0}
    CreateVcpu(kvm_ioctls::Error),
    /// Error configuring the vcpu registers: {0}
    ConfigureRegisters(VcpuArchError),
    /// RISC-V vCPU snapshot state is not supported yet.
    SnapshotUnsupported,
}

/// Error type for [`KvmVcpu::configure`].
pub type KvmVcpuConfigureError = KvmVcpuError;

/// A wrapper around creating and using a KVM RISC-V vcpu.
#[derive(Debug)]
pub struct KvmVcpu {
    /// Index of vcpu.
    pub index: u8,
    /// KVM vcpu fd.
    pub fd: VcpuFd,
    /// Vcpu peripherals, such as buses.
    pub peripherals: Peripherals,
}

/// Vcpu peripherals.
#[derive(Default, Debug)]
pub struct Peripherals {
    /// MMIO bus.
    pub mmio_bus: Option<Arc<Bus>>,
}

impl KvmVcpu {
    /// Constructs a new KVM vcpu with RISC-V specific functionality.
    pub fn new(index: u8, vm: &KvmVm) -> Result<Self, KvmVcpuError> {
        let fd = vm
            .fd()
            .create_vcpu(index.into())
            .map_err(KvmVcpuError::CreateVcpu)?;

        Ok(Self {
            index,
            fd,
            peripherals: Default::default(),
        })
    }

    /// Configures a RISC-V vcpu for booting Linux.
    pub fn configure(
        &mut self,
        guest_mem: &GuestMemoryMmap,
        kernel_entry_point: EntryPoint,
        _vcpu_config: &VcpuConfig,
    ) -> Result<(), KvmVcpuError> {
        self.setup_boot_regs(kernel_entry_point.entry_addr.raw_value(), guest_mem)
            .map_err(KvmVcpuError::ConfigureRegisters)
    }

    /// Save the KVM internal state.
    pub fn save_state(&self) -> Result<VcpuState, KvmVcpuError> {
        Err(KvmVcpuError::SnapshotUnsupported)
    }

    /// Use provided state to populate KVM internal state.
    pub fn restore_state(&mut self, _state: &VcpuState) -> Result<(), KvmVcpuError> {
        Err(KvmVcpuError::SnapshotUnsupported)
    }

    /// Dumps CPU configuration.
    pub fn dump_cpu_config(&self) -> Result<CpuConfiguration, KvmVcpuError> {
        Ok(CpuConfiguration::default())
    }

    /// Configure relevant boot registers for a given vCPU.
    pub fn setup_boot_regs(
        &self,
        boot_ip: u64,
        _mem: &GuestMemoryMmap,
    ) -> Result<(), VcpuArchError> {
        if self.index == 0 {
            self.set_u64_reg(RISCV_CORE_REG_PC, boot_ip)?;
            self.set_u64_reg(RISCV_CORE_REG_A0, 0)?;
            self.set_u64_reg(RISCV_CORE_REG_A1, super::get_fdt_addr(_mem))?;
        }
        Ok(())
    }

    fn set_u64_reg(&self, index: u64, value: u64) -> Result<(), VcpuArchError> {
        let id = riscv_core_reg_id(index);
        self.fd
            .set_one_reg(id, &value.to_le_bytes())
            .map(|_| ())
            .map_err(|err| VcpuArchError::SetOneReg(id, format!("{value:#x}"), err))
    }
}

impl Peripherals {
    /// Runs architecture-specific vCPU emulation.
    pub fn run_arch_emulation(&self, exit: VcpuExit) -> Result<VcpuEmulation, VcpuError> {
        if matches!(exit, VcpuExit::Shutdown) {
            info!("Received KVM_EXIT_SHUTDOWN");
            return Ok(VcpuEmulation::Stopped);
        }

        METRICS.vcpu.failures.inc();
        error!("Unexpected exit reason on vcpu run: {:?}", exit);
        Err(VcpuError::UnhandledKvmExit(format!("{:?}", exit)))
    }
}

/// RISC-V vCPU state.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VcpuState;
