// SPDX-License-Identifier: MPL-2.0

//! Explicit no-device configuration for KMiri bring-up experiments.

pub(super) use ostd::irq::IrqLine as MappedIrqLine;

pub(super) fn probe_for_device() {
    // No MMIO devices are described by the Miri boot environment. Do not
    // fabricate a device or claim to exercise its native interrupt mapping.
    ostd::warn!("KMiri experiment: no VirtIO-MMIO devices configured");
}
