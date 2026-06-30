// Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use vmm::cpu_config::templates::CustomCpuTemplate;

use crate::template::strip::StripError;

pub fn strip(templates: Vec<CustomCpuTemplate>) -> Result<Vec<CustomCpuTemplate>, StripError> {
    if templates.len() < 2 {
        return Err(StripError::NumberOfInputs);
    }

    Ok(templates)
}
