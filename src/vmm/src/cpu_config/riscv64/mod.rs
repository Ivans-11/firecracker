// Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

/// Module for custom CPU templates.
pub mod custom_cpu_template;
/// Module for static CPU templates.
pub mod static_cpu_templates;

/// Test utilities for RISC-V CPU templates.
pub mod test_utils {
    #[allow(unused_imports)]
    pub(crate) use super::custom_cpu_template::CustomCpuTemplate;
}

/// Errors thrown while configuring templates.
#[derive(Debug, PartialEq, Eq, thiserror::Error, displaydoc::Display)]
pub enum CpuConfigurationError {
    /// RISC-V CPU templates do not support register modifiers yet.
    UnsupportedTemplate,
}

/// CPU configuration for RISC-V.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CpuConfiguration;

impl CpuConfiguration {
    /// Create new CpuConfiguration.
    pub fn new(
        _cpu_template: &super::templates::CustomCpuTemplate,
    ) -> Result<Self, CpuConfigurationError> {
        Ok(Self)
    }

    /// Creates new guest CPU config based on the provided template.
    pub fn apply_template(self, _template: &super::templates::CustomCpuTemplate) -> Self {
        self
    }
}
