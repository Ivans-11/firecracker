// Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

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
}

/// Error type for RISC-V VM setup and restore.
#[derive(Debug, PartialEq, Eq, thiserror::Error, displaydoc::Display)]
pub enum KvmVmError {
    /// RISC-V in-kernel irqchip support is not wired up yet.
    UnsupportedIrqchip,
    /// Failed to restore resource allocator: {0}
    ResourceAllocator(#[from] vm_allocator::Error),
}

impl KvmVm {
    /// Create a new `KvmVm` struct.
    pub fn new(kvm: Kvm) -> Result<KvmVm, VmError> {
        let common = Self::create_common(kvm)?;
        Ok(KvmVm { common })
    }

    /// Pre-vCPU creation setup.
    pub fn arch_pre_create_vcpus(&mut self, _: u8) -> Result<(), KvmVmError> {
        Ok(())
    }

    /// Post-vCPU creation setup.
    pub fn arch_post_create_vcpus(&mut self, _: u8) -> Result<(), KvmVmError> {
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
}

/// RISC-V VM state.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VmState {
    /// Guest memory state.
    pub memory: GuestMemoryState,
    /// Resource allocator state.
    pub resource_allocator: ResourceAllocatorState,
}
