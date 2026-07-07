//! Per-crate write barrier for po-rust's `pub static mut` globals (crate-split
//! Phase 1b). NEVER CALLED; #[no_mangle]. 16 entries.
#[allow(dead_code)]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _po_global_write_barrier() {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    macro_rules! b { ($p:path) => { write_volatile(addr_of_mut!($p), read_volatile(addr_of!($p))); } }
    b!(crate::events::PopEventCallout);
    b!(crate::poshtdwn::PopShutdownPowerOffPolicy); b!(crate::poshtdwn::PopShutdownEvent);
    b!(crate::poshtdwn::PopShutdownThreadList); b!(crate::poshtdwn::PopShutdownQueue);
    b!(crate::poshtdwn::PopShutdownListMutex); b!(crate::poshtdwn::PopShutdownListAvailable);
    b!(crate::povolume::PopFlushPolicy); b!(crate::povolume::PopVolumeLock);
    b!(crate::povolume::PopVolumeDevices); b!(crate::povolume::PopDopeGlobalLock);
    b!(crate::power::PopSystemPowerDeviceNode); b!(crate::power::PopAcpiPresent);
    b!(crate::power::PopAction); b!(crate::power::PopShutdownWorkItem); b!(crate::power::PopCapabilities);
}
