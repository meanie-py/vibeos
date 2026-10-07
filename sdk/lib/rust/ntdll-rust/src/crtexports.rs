//! C-runtime exports of arm64 ntdll, for the FEX CPU layers (`libwow64fex`,
//! `libwowx64fex`). Their mini-CRT takes `_errno`, `atan2` and `tan` from
//! ntdll, as Wine's ntdll provides them. Those were forwarders to msvcrt, and
//! resolving a forwarder LOADS its target: so every WoW64 and x86-64 guest
//! process loaded the native msvcrt, and through msvcrt's imports the native
//! kernel32 -- whose DllMain registers the process with csrss and writes the
//! PEB, which in an x86-64 guest process is the guest's (the guest kernel32
//! then does both again). `atan2`/`tan` are now the CRT math linked into
//! ntdll like `sin`/`cos` (ntdll.spec); `_errno` is here.

use ntdll_sys::nt::*;

/// The TLS slot that holds every thread's `errno`, as Wine's ntdll keeps it:
/// a slot of the TEB's own `TlsSlots[]`, reserved in `PEB->TlsBitmap` at
/// process init (ldrinit.rs) so `TlsAlloc` -- kernel32's, a guest kernel32's
/// sharing this PEB, or FEX's own -- can never hand it out. Above the WoW64
/// range `0..WOW64_TLS_MAX_NUMBER`, which a WoW64 process reserves as well.
///
/// It must be a RESERVED slot, not merely a quiet-looking TEB field. The first
/// attempt at this used `ReservedForPerf`, and the 32-bit ladder crashed in
/// libwow64fex's rpmalloc; that crash was the unreserved WoW64 slots below,
/// not `ReservedForPerf` -- but an unreserved field is exactly the kind of
/// home that ends that way, so `errno` does not get one.
pub const NTDLL_TLS_ERRNO: u32 = 19;

/// The calling thread's `errno`: an `int` at the start of its pointer-sized slot.
#[no_mangle]
pub unsafe extern "C" fn _errno() -> *mut i32 {
    core::ptr::addr_of_mut!((*NtCurrentTeb()).TlsSlots[NTDLL_TLS_ERRNO as usize]) as *mut i32
}
