// Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::cpu_config::templates::{
    CpuTemplateType, GetCpuTemplate, GetCpuTemplateError, KvmCapability,
};

impl GetCpuTemplate for Option<CpuTemplateType> {
    fn get_cpu_template(&self) -> Result<Cow<'_, CustomCpuTemplate>, GetCpuTemplateError> {
        match self {
            Some(CpuTemplateType::Custom(template)) => Ok(Cow::Borrowed(template)),
            Some(CpuTemplateType::Static(template)) => {
                Err(GetCpuTemplateError::InvalidStaticCpuTemplate(*template))
            }
            None => Ok(Cow::Owned(CustomCpuTemplate::default())),
        }
    }
}

/// Wrapper type containing RISC-V CPU config modifiers.
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomCpuTemplate {
    /// Additional kvm capabilities to check before configuring vcpus.
    #[serde(default)]
    pub kvm_capabilities: Vec<KvmCapability>,
}

impl CustomCpuTemplate {
    /// Validate the correctness of the template.
    pub fn validate(&self) -> Result<(), serde_json::Error> {
        Ok(())
    }
}
