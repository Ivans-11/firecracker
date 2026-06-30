// Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use vmm::cpu_config::templates::{CpuConfiguration, CustomCpuTemplate};

pub fn config_to_template(_cpu_config: &CpuConfiguration) -> CustomCpuTemplate {
    CustomCpuTemplate::default()
}
