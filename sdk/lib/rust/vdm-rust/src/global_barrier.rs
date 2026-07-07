//! Per-crate write barrier for vdm-rust's `pub static mut` globals (crate-split
//! Phase 1b). NEVER CALLED; #[no_mangle]. 1 entry. (Whole crate is i386-only.)
#[allow(dead_code)]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _vdm_global_write_barrier() {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    write_volatile(addr_of_mut!(crate::vdmexec::VdmBopCount), read_volatile(addr_of!(crate::vdmexec::VdmBopCount)));
}
