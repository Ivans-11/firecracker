// Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use vmm::cpu_config::templates::CustomCpuTemplate;

use super::VerifyError;

pub fn verify(
    _cpu_template: CustomCpuTemplate,
    _cpu_config: CustomCpuTemplate,
) -> Result<(), VerifyError> {
    Ok(())
}
