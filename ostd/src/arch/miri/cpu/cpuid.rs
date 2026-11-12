// SPDX-License-Identifier: MPL-2.0
//! CPUID is not executed by the Miri platform model.

use core::arch::x86_64::CpuidResult;

pub fn cpuid(_leaf: u32, _subleaf: u32) -> Option<CpuidResult> {
    None
}
