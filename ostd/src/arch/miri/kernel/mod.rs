// SPDX-License-Identifier: MPL-2.0
//! Minimal ACPI view for the Miri platform. No firmware tables are present.

use spin::Once;

#[derive(Debug)]
pub struct AcpiInfo {
    pub reset_port_and_val: Option<(u16, u8)>,
}

pub static ACPI_INFO: Once<AcpiInfo> = Once::new();

pub(crate) fn init() {
    ACPI_INFO.call_once(|| AcpiInfo { reset_port_and_val: None });
}
