// SPDX-License-Identifier: MPL-2.0

//! Handle keyboard input.
#![no_std]
#![deny(unsafe_code)]
#![cfg(target_arch = "x86_64")]

extern crate alloc;

use component::{ComponentInitError, init_component};
use ostd::power;

#[cfg(not(miri))]
use self::controller::I8042_CONTROLLER;

// Set this crate's log prefix for `ostd::log`.
macro_rules! __log_prefix {
    () => {
        "i8042: "
    };
}

#[cfg(not(miri))]
mod controller;
#[cfg(not(miri))]
mod keyboard;
#[cfg(not(miri))]
mod mouse;
#[cfg(not(miri))]
mod ps2;

#[init_component]
fn init() -> Result<(), ComponentInitError> {
    #[cfg(not(miri))]
    if let Err(err) = controller::init() {
        ostd::warn!("i8042 controller initialization failed: {:?}", err);
    }
    #[cfg(miri)]
    ostd::warn!("KMiri experiment: no i8042 controller configured");
    Ok(())
}

/// Attempts to reset the CPU via the i8042 PS/2 controller.
pub fn try_cpu_reset(_code: power::ExitCode) {
    // If possible, keep this method panic-free because it may be called by the panic handler.
    #[cfg(not(miri))]
    if let Some(controller) = I8042_CONTROLLER.get() {
        controller.lock().reset_cpu();
    }
}
