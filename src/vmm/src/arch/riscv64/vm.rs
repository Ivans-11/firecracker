// Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Mutex;

use kvm_bindings::{
    KVM_DEV_RISCV_AIA_ADDR_APLIC, KVM_DEV_RISCV_AIA_CONFIG_IDS, KVM_DEV_RISCV_AIA_CONFIG_MODE,
    KVM_DEV_RISCV_AIA_CONFIG_SRCS, KVM_DEV_RISCV_AIA_CTRL_INIT, KVM_DEV_RISCV_AIA_GRP_ADDR,
    KVM_DEV_RISCV_AIA_GRP_CONFIG, KVM_DEV_RISCV_AIA_GRP_CTRL, KVM_DEV_RISCV_AIA_MODE_AUTO,
    kvm_create_device, kvm_device_attr, kvm_device_type_KVM_DEV_TYPE_RISCV_AIA,
};
use kvm_ioctls::{Cap, DeviceFd};
use serde::{Deserialize, Serialize};

use super::layout;
use crate::Kvm;
use crate::snapshot::Persist;
use crate::vstate::memory::{GuestMemoryExtension, GuestMemoryState};
use crate::vstate::resources::{ResourceAllocator, ResourceAllocatorState};
use crate::vstate::vm::{VmCommon, VmError};

/// RISC-V KVM VM wrapper.
#[derive(Debug)]
pub struct KvmVm {
    /// Architecture independent VM state.
    pub common: VmCommon,
    aia_device: Option<DeviceFd>,
}

/// Error type for RISC-V VM setup and restore.
#[derive(Debug, PartialEq, Eq, thiserror::Error, displaydoc::Display)]
pub enum KvmVmError {
    /// RISC-V in-kernel irqchip support is not wired up yet.
    UnsupportedIrqchip,
    /// Failed to create RISC-V AIA device: {0}
    CreateAia(kvm_ioctls::Error),
    /// Failed to configure RISC-V AIA device attribute: {0}
    AiaDeviceAttribute(kvm_ioctls::Error),
    /// Failed to restore resource allocator: {0}
    ResourceAllocator(#[from] vm_allocator::Error),
}

impl KvmVm {
    /// Create a new `KvmVm` struct.
    pub fn new(kvm: Kvm) -> Result<KvmVm, VmError> {
        let common = Self::create_common(kvm)?;
        Ok(KvmVm {
            common,
            aia_device: None,
        })
    }

    /// Pre-vCPU creation setup.
    pub fn arch_pre_create_vcpus(&mut self, _: u8) -> Result<(), KvmVmError> {
        Ok(())
    }

    /// Post-vCPU creation setup.
    pub fn arch_post_create_vcpus(&mut self, vcpu_count: u8) -> Result<(), KvmVmError> {
        if self.common.fd.check_extension(Cap::DeviceCtrl) {
            self.create_aia_device(vcpu_count)?;
        }
        Ok(())
    }

    /// Saves and returns the KVM VM state.
    pub fn save_state(&self, _hart_ids: &[u64]) -> Result<VmState, KvmVmError> {
        Ok(VmState {
            memory: self.common.guest_memory.describe(),
            resource_allocator: self.resource_allocator().save(),
        })
    }

    /// Restore the KVM VM state.
    pub fn restore_state(&mut self, _hart_ids: &[u64], state: &VmState) -> Result<(), KvmVmError> {
        self.common.resource_allocator =
            Mutex::new(ResourceAllocator::restore((), &state.resource_allocator)?);
        Ok(())
    }

    fn create_aia_device(&mut self, vcpu_count: u8) -> Result<(), KvmVmError> {
        let mut aia_device = kvm_create_device {
            type_: kvm_device_type_KVM_DEV_TYPE_RISCV_AIA,
            fd: 0,
            flags: 0,
        };
        let aia_device = self
            .common
            .fd
            .create_device(&mut aia_device)
            .map_err(KvmVmError::CreateAia)?;

        let mode = KVM_DEV_RISCV_AIA_MODE_AUTO;
        set_device_attr(
            &aia_device,
            KVM_DEV_RISCV_AIA_GRP_CONFIG,
            KVM_DEV_RISCV_AIA_CONFIG_MODE.into(),
            &mode,
        )?;

        let ids = layout::AIA_IMSIC_NUM_IDS;
        set_device_attr(
            &aia_device,
            KVM_DEV_RISCV_AIA_GRP_CONFIG,
            KVM_DEV_RISCV_AIA_CONFIG_IDS.into(),
            &ids,
        )?;

        let sources = layout::GSI_LEGACY_NUM;
        set_device_attr(
            &aia_device,
            KVM_DEV_RISCV_AIA_GRP_CONFIG,
            KVM_DEV_RISCV_AIA_CONFIG_SRCS.into(),
            &sources,
        )?;

        let aplic_addr = layout::AIA_APLIC_MEM_START;
        set_device_attr(
            &aia_device,
            KVM_DEV_RISCV_AIA_GRP_ADDR,
            KVM_DEV_RISCV_AIA_ADDR_APLIC.into(),
            &aplic_addr,
        )?;

        for vcpu_id in 0..vcpu_count {
            let imsic_addr =
                layout::AIA_IMSIC_MEM_START + layout::AIA_IMSIC_MEM_SIZE * u64::from(vcpu_id);
            let imsic_attr = KVM_DEV_RISCV_AIA_ADDR_APLIC + 1 + u32::from(vcpu_id);
            set_device_attr(
                &aia_device,
                KVM_DEV_RISCV_AIA_GRP_ADDR,
                imsic_attr.into(),
                &imsic_addr,
            )?;
        }

        let init_attr = kvm_device_attr {
            group: KVM_DEV_RISCV_AIA_GRP_CTRL,
            attr: KVM_DEV_RISCV_AIA_CTRL_INIT.into(),
            addr: 0,
            flags: 0,
        };
        aia_device
            .set_device_attr(&init_attr)
            .map_err(KvmVmError::AiaDeviceAttribute)?;

        self.aia_device = Some(aia_device);
        Ok(())
    }
}

fn set_device_attr<T>(
    device: &DeviceFd,
    group: u32,
    attr: u64,
    value: &T,
) -> Result<(), KvmVmError> {
    let device_attr = kvm_device_attr {
        group,
        attr,
        addr: value as *const T as u64,
        flags: 0,
    };
    device
        .set_device_attr(&device_attr)
        .map_err(KvmVmError::AiaDeviceAttribute)
}

/// RISC-V VM state.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VmState {
    /// Guest memory state.
    pub memory: GuestMemoryState,
    /// Resource allocator state.
    pub resource_allocator: ResourceAllocatorState,
}
