//! Per-crate write barrier for inbv-rust's `pub static mut` globals (crate-split
//! Phase 1b). NEVER CALLED; #[no_mangle]. 15 entries.
#[allow(dead_code)]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _inbv_global_write_barrier() {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    macro_rules! b { ($p:path) => { write_volatile(addr_of_mut!($p), read_volatile(addr_of!($p))); } }
    b!(crate::bootanim::RotBarThreadActive); b!(crate::bootanim::DotHack);
    b!(crate::inbv::BootDriverLock); b!(crate::inbv::InbvOldIrql); b!(crate::inbv::InbvDisplayState);
    b!(crate::inbv::InbvBootDriverInstalled); b!(crate::inbv::InbvResetDisplayParameters);
    b!(crate::inbv::InbvDisplayDebugStrings); b!(crate::inbv::InbvDisplayFilter);
    b!(crate::inbv::ProgressBarLeft); b!(crate::inbv::ProgressBarTop); b!(crate::inbv::ShowProgressBar);
    b!(crate::inbv::ResourceCount); b!(crate::inbv::InbvTerminalTextColor); b!(crate::inbv::InbvTerminalBkgdColor);
}
