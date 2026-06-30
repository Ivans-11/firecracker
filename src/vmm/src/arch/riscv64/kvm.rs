// Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::convert::Infallible;

use kvm_ioctls::Kvm as KvmFd;

use crate::cpu_config::templates::KvmCapability;

/// KVM initialization does not currently need RISC-V specific fallible setup.
pub type KvmArchError = Infallible;

/// Optional RISC-V KVM capabilities.
#[derive(Debug, Default)]
pub struct OptionalCapabilities;

/// Struct with kvm fd and KVM associated parameters.
#[derive(Debug)]
pub struct Kvm {
    /// KVM fd.
    pub fd: KvmFd,
    /// Additional capabilities that were specified in CPU template.
    pub kvm_cap_modifiers: Vec<KvmCapability>,
}

impl Kvm {
    /// Minimal KVM capabilities required by the architecture-independent VMM.
    pub(crate) const DEFAULT_CAPABILITIES: [u32; 3] = [
        kvm_bindings::KVM_CAP_IOEVENTFD,
        kvm_bindings::KVM_CAP_USER_MEMORY,
        kvm_bindings::KVM_CAP_ONE_REG,
    ];

    /// Initialize [`Kvm`] type for RISC-V.
    pub fn init_arch(
        fd: KvmFd,
        kvm_cap_modifiers: Vec<KvmCapability>,
    ) -> Result<Self, KvmArchError> {
        Ok(Self {
            fd,
            kvm_cap_modifiers,
        })
    }

    /// Returns optional capability statuses.
    pub fn optional_capabilities(&self) -> OptionalCapabilities {
        OptionalCapabilities
    }
}
