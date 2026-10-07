//! Message-signaled interrupts (MSI-X): the HAL's half.
//!
//! The flow, Windows-shaped:
//!   1. pci.sys adds one `CmResourceTypeInterrupt` requirement per MSI-X message,
//!      flagged `CM_RESOURCE_INTERRUPT_MESSAGE`, whose vector is a TOKEN rather than
//!      a line: message k asks for raw vector `CM_RESOURCE_INTERRUPT_MESSAGE_TOKEN - k`.
//!      Distinct tokens keep the PnP manager's match-by-vector bookkeeping from
//!      folding two messages into one resource.
//!   2. The PnP manager translates each through `HalGetInterruptVector`, which spots
//!      the token range and routes here instead of to a line: [`allocate`] hands out a
//!      vector the CPU can take a message on, and its IRQL.
//!   3. At START pci.sys asks [`HalVibeosMessageTarget`] for the address/data pair that
//!      raises that vector, and writes it into the device's MSI-X table.
//!   4. `IoConnectInterruptEx(CONNECT_MESSAGE_BASED)` connects each translated vector
//!      like any other.
//!
//! Windows keeps steps 2-3 behind a private HAL interface; ours are two vibeos-named
//! exports, the HalVibeos* convention for HAL entry points with no Windows original.
//!
//! Backends: x86 hands out APIC vectors with no IOAPIC pin behind them
//! (x86/apic.rs); arm64 hands out SPIs behind a GICv2m MSI frame (arm64/v2m.rs). The
//! PIC HAL and an arm64 GIC without a v2m frame (the Pi 4's GIC-400) have none, so
//! [`HalVibeosMessageCapacity`] reports 0 and pci.sys never asks.

use ntkrnl_sys::ddk::{BOOLEAN, PKAFFINITY, PKIRQL, ULONG};

/// `CM_RESOURCE_INTERRUPT_MESSAGE_TOKEN` (cmtypes.h).
pub const MESSAGE_TOKEN: u32 = 0xFFFF_FFFE;
/// MSI-X tables hold at most 2048 entries.
pub const MAX_MESSAGES: u32 = 2048;

/// Is this raw interrupt vector a message token rather than a line?
pub fn is_message_token(vector: u32) -> bool {
    vector <= MESSAGE_TOKEN && vector > MESSAGE_TOKEN - MAX_MESSAGES
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), hal_apic))]
use crate::x86::apic as backend;
#[cfg(target_arch = "aarch64")]
use crate::arm64::v2m as backend;

#[cfg(any(all(any(target_arch = "x86", target_arch = "x86_64"), hal_apic), target_arch = "aarch64"))]
mod imp {
    pub(super) use super::backend::{message_capacity, message_target, allocate_message_vector};
}

/// No message-capable interrupt controller (the PIC HAL).
#[cfg(not(any(all(any(target_arch = "x86", target_arch = "x86_64"), hal_apic), target_arch = "aarch64")))]
mod imp {
    use ntkrnl_sys::ddk::{KAFFINITY, KIRQL};
    pub(super) unsafe fn message_capacity() -> u32 {
        0
    }
    pub(super) unsafe fn message_target(_vector: u32) -> Option<(u64, u32)> {
        None
    }
    pub(super) unsafe fn allocate_message_vector() -> Option<(u32, KIRQL, KAFFINITY)> {
        None
    }
}

/// `HalGetInterruptVector` for a message token: a fresh vector, or 0 (the "no vector"
/// answer every caller already checks) when the controller is out of them.
pub unsafe fn allocate(irql: PKIRQL, affinity: PKAFFINITY) -> ULONG {
    match unsafe { imp::allocate_message_vector() } {
        Some((vector, level, mask)) => {
            unsafe {
                if !irql.is_null() {
                    *irql = level;
                }
                if !affinity.is_null() {
                    *affinity = mask;
                }
            }
            vector
        }
        None => 0,
    }
}

/// How many more message vectors this HAL can hand out (0: no MSI support at all).
/// pci.sys asks before requesting messages, so a device on a HAL without them keeps
/// its line interrupt instead of failing translation.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalVibeosMessageCapacity() -> ULONG {
    unsafe { imp::message_capacity() }
}

/// The address/data pair a device writes to raise `Vector`, which must have come from
/// a message token's translation. FALSE for any other vector.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalVibeosMessageTarget(
    Vector: ULONG,
    Address: *mut u64,
    Data: *mut ULONG,
) -> BOOLEAN {
    match unsafe { imp::message_target(Vector) } {
        Some((address, data)) => {
            unsafe {
                *Address = address;
                *Data = data;
            }
            1
        }
        None => 0,
    }
}
