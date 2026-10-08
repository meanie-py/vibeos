//! Message-signaled interrupts (MSI-X): the HAL's half.
//!
//! The flow, Windows-shaped:
//!   1. pci.sys adds one `CmResourceTypeInterrupt` requirement per MSI-X message,
//!      flagged `CM_RESOURCE_INTERRUPT_MESSAGE`, whose vector is a TOKEN rather than
//!      a line: message k asks for raw vector `CM_RESOURCE_INTERRUPT_MESSAGE_TOKEN - k`.
//!      Distinct tokens keep the PnP manager's match-by-vector bookkeeping from
//!      folding two messages into one resource.
//!   2. The PnP manager translates each through [`HalVibeosMessageAllocate`], which
//!      hands out a vector the CPU can take a message on, its IRQL, and the one CPU it
//!      is delivered to, chosen from the processors the device's affinity policy
//!      allows. (`HalGetInterruptVector` spots a token too, and allocates with no
//!      preference.)
//!   3. At START pci.sys asks [`HalVibeosMessageTarget`] for the address/data pair that
//!      raises that vector, naming the device that will write it, and writes the pair
//!      into the device's MSI-X table.
//!   4. `IoConnectInterruptEx(CONNECT_MESSAGE_BASED)` connects each translated vector
//!      like any other.
//!   5. When the device is removed the PnP manager gives each vector back through
//!      [`HalVibeosMessageRelease`].
//!
//! Windows keeps steps 2-3 behind a private HAL interface; ours are two vibeos-named
//! exports, the HalVibeos* convention for HAL entry points with no Windows original.
//!
//! Backends: x86 hands out APIC vectors with no IOAPIC pin behind them
//! (x86/apic.rs); arm64 hands out SPIs behind a GICv2m MSI frame (arm64/v2m.rs) on a
//! GICv2, and LPIs translated by an ITS (arm64/its.rs) on a GICv3. Only the ITS cares
//! who writes a message: it keys its translation on the device. The PIC HAL and an
//! arm64 GICv2 without a v2m frame (the Pi 4's GIC-400) have none, so
//! [`HalVibeosMessageCapacity`] reports 0 and pci.sys never asks.

use ntkrnl_sys::ddk::{BOOLEAN, KAFFINITY, PKAFFINITY, PKIRQL, ULONG};

/// `CM_RESOURCE_INTERRUPT_MESSAGE_TOKEN` (cmtypes.h).
pub const MESSAGE_TOKEN: u32 = 0xFFFF_FFFE;
/// MSI-X tables hold at most 2048 entries.
pub const MAX_MESSAGES: u32 = 2048;
/// [`HalVibeosMessageTarget`]'s requester when the caller does not know the device:
/// only a vector already bound to one (by pci.sys) can be answered.
pub const REQUESTER_UNKNOWN: u32 = u32::MAX;

/// Is this raw interrupt vector a message token rather than a line?
pub fn is_message_token(vector: u32) -> bool {
    vector <= MESSAGE_TOKEN && vector > MESSAGE_TOKEN - MAX_MESSAGES
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), hal_apic))]
mod imp {
    pub(super) use crate::x86::apic::{allocate_message_vector, message_capacity, message_target, release_message_vector};
}

/// The GIC the firmware described decides: an ITS on a GICv3, a v2m frame on a GICv2.
#[cfg(target_arch = "aarch64")]
mod imp {
    use crate::arm64::{gic3, its, v2m};
    use ntkrnl_sys::ddk::{KAFFINITY, KIRQL};
    pub(super) unsafe fn message_capacity() -> u32 {
        unsafe { if gic3::present() { its::message_capacity() } else { v2m::message_capacity() } }
    }
    pub(super) unsafe fn message_target(vector: u32, requester: u32) -> Option<(u64, u32)> {
        unsafe { if gic3::present() { its::message_target(vector, requester) } else { v2m::message_target(vector, requester) } }
    }
    pub(super) unsafe fn allocate_message_vector(requested: KAFFINITY) -> Option<(u32, KIRQL, KAFFINITY)> {
        unsafe { if gic3::present() { its::allocate_message_vector(requested) } else { v2m::allocate_message_vector(requested) } }
    }
    pub(super) unsafe fn release_message_vector(vector: u32) -> bool {
        unsafe { if gic3::present() { its::release_message_vector(vector) } else { v2m::release_message_vector(vector) } }
    }
}

/// No message-capable interrupt controller (the PIC HAL).
#[cfg(not(any(all(any(target_arch = "x86", target_arch = "x86_64"), hal_apic), target_arch = "aarch64")))]
mod imp {
    use ntkrnl_sys::ddk::{KAFFINITY, KIRQL};
    pub(super) unsafe fn message_capacity() -> u32 {
        0
    }
    pub(super) unsafe fn message_target(_vector: u32, _requester: u32) -> Option<(u64, u32)> {
        None
    }
    pub(super) unsafe fn allocate_message_vector(_requested: KAFFINITY) -> Option<(u32, KIRQL, KAFFINITY)> {
        None
    }
    pub(super) unsafe fn release_message_vector(_vector: u32) -> bool {
        false
    }
}

/// `HalGetInterruptVector` for a message token: a fresh vector, or 0 (the "no vector"
/// answer every caller already checks) when the controller is out of them.
pub unsafe fn allocate(irql: PKIRQL, affinity: PKAFFINITY) -> ULONG {
    unsafe { allocate_for(!0, irql, affinity) }
}

unsafe fn allocate_for(requested: KAFFINITY, irql: PKIRQL, affinity: PKAFFINITY) -> ULONG {
    match unsafe { imp::allocate_message_vector(requested) } {
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

/// Translate message token `Token` for a device that may take it on the processors in
/// `Requested` (its affinity policy's set; all ones for no preference): a vector, its
/// IRQL, and the single processor it is delivered to. 0 when `Token` is not a message
/// token or the controller has no vector left.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalVibeosMessageAllocate(
    Token: ULONG,
    Requested: KAFFINITY,
    Irql: PKIRQL,
    Affinity: PKAFFINITY,
) -> ULONG {
    if !is_message_token(Token) {
        return 0;
    }
    unsafe { allocate_for(Requested, Irql, Affinity) }
}

/// Give back a vector [`HalVibeosMessageAllocate`] handed out: the device it was for
/// is gone. Its interrupt is disabled and the vector may be handed out again. FALSE
/// for a vector that was not handed out.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalVibeosMessageRelease(Vector: ULONG) -> BOOLEAN {
    unsafe { imp::release_message_vector(Vector) as BOOLEAN }
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
///
/// `RequesterId` is the PCI requester ID (bus << 8 | device << 3 | function) of the
/// device that will write it, or [`REQUESTER_UNKNOWN`]. The first ask for a vector
/// must name it where the backend translates per device (the ITS); later asks get the
/// same pair back without it.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalVibeosMessageTarget(
    Vector: ULONG,
    RequesterId: ULONG,
    Address: *mut u64,
    Data: *mut ULONG,
) -> BOOLEAN {
    match unsafe { imp::message_target(Vector, RequesterId) } {
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
