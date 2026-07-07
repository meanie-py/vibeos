//! Per-crate write barrier for pnpmgr-rust's `pub static mut` globals — a visible,
//! un-eliminable write site so LLVM can't const-fold a global that's 0-initialized
//! in Rust but written from C or another crate. NEVER CALLED; `#[no_mangle]` +
//! pulled into the link via kernel-rust's `extern crate pnpmgr_rust`. Relocated from
//! ntoskrnl-rust/src/global_barrier.rs during the crate-split (Phase 1b).
//!
//! Hand-written for the crate-split. When `tools/gen_global_barrier.py` is made
//! crate-aware this becomes generated output — see
//! docs/engineering/ntoskrnl-crate-split-plan.md.
//!
//! 22 statics: devaction ×5, devnode ×3, pnpinit ×9, pnpmgr ×5 (no arb statics).
#[allow(unused_unsafe)]
#[allow(dead_code)]
/// Write barrier — touches pnpmgr-rust's pub static mut globals. NEVER CALLED.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _pnpmgr_global_write_barrier() {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    // devaction.rs
    write_volatile(addr_of_mut!(crate::devaction::IopDeviceActionRequestList), read_volatile(addr_of!(crate::devaction::IopDeviceActionRequestList)));
    write_volatile(addr_of_mut!(crate::devaction::IopDeviceActionWorkItem), read_volatile(addr_of!(crate::devaction::IopDeviceActionWorkItem)));
    write_volatile(addr_of_mut!(crate::devaction::IopDeviceActionInProgress), read_volatile(addr_of!(crate::devaction::IopDeviceActionInProgress)));
    write_volatile(addr_of_mut!(crate::devaction::IopDeviceActionLock), read_volatile(addr_of!(crate::devaction::IopDeviceActionLock)));
    write_volatile(addr_of_mut!(crate::devaction::PiEnumerationFinished), read_volatile(addr_of!(crate::devaction::PiEnumerationFinished)));
    // devnode.rs
    write_volatile(addr_of_mut!(crate::devnode::IopRootDeviceNode), read_volatile(addr_of!(crate::devnode::IopRootDeviceNode)));
    write_volatile(addr_of_mut!(crate::devnode::IopDeviceTreeLock), read_volatile(addr_of!(crate::devnode::IopDeviceTreeLock)));
    write_volatile(addr_of_mut!(crate::devnode::IopNumberDeviceNodes), read_volatile(addr_of!(crate::devnode::IopNumberDeviceNodes)));
    // pnpinit.rs
    write_volatile(addr_of_mut!(crate::pnpinit::PiInitGroupOrderTable), read_volatile(addr_of!(crate::pnpinit::PiInitGroupOrderTable)));
    write_volatile(addr_of_mut!(crate::pnpinit::PiInitGroupOrderTableCount), read_volatile(addr_of!(crate::pnpinit::PiInitGroupOrderTableCount)));
    write_volatile(addr_of_mut!(crate::pnpinit::PnPBootDriversLoaded), read_volatile(addr_of!(crate::pnpinit::PnPBootDriversLoaded)));
    write_volatile(addr_of_mut!(crate::pnpinit::PnPBootDriversInitialized), read_volatile(addr_of!(crate::pnpinit::PnPBootDriversInitialized)));
    write_volatile(addr_of_mut!(crate::pnpinit::IopRootBusNumberArbiter), read_volatile(addr_of!(crate::pnpinit::IopRootBusNumberArbiter)));
    write_volatile(addr_of_mut!(crate::pnpinit::IopRootIrqArbiter), read_volatile(addr_of!(crate::pnpinit::IopRootIrqArbiter)));
    write_volatile(addr_of_mut!(crate::pnpinit::IopRootDmaArbiter), read_volatile(addr_of!(crate::pnpinit::IopRootDmaArbiter)));
    write_volatile(addr_of_mut!(crate::pnpinit::IopRootMemArbiter), read_volatile(addr_of!(crate::pnpinit::IopRootMemArbiter)));
    write_volatile(addr_of_mut!(crate::pnpinit::IopRootPortArbiter), read_volatile(addr_of!(crate::pnpinit::IopRootPortArbiter)));
    // pnpmgr.rs
    write_volatile(addr_of_mut!(crate::pnpmgr::PpRegistryDeviceResource), read_volatile(addr_of!(crate::pnpmgr::PpRegistryDeviceResource)));
    write_volatile(addr_of_mut!(crate::pnpmgr::PpDeviceReferenceTableLock), read_volatile(addr_of!(crate::pnpmgr::PpDeviceReferenceTableLock)));
    write_volatile(addr_of_mut!(crate::pnpmgr::PpDeviceReferenceTable), read_volatile(addr_of!(crate::pnpmgr::PpDeviceReferenceTable)));
    write_volatile(addr_of_mut!(crate::pnpmgr::IopRootDriverObject), read_volatile(addr_of!(crate::pnpmgr::IopRootDriverObject)));
    write_volatile(addr_of_mut!(crate::pnpmgr::PnpBusTypeGuidList), read_volatile(addr_of!(crate::pnpmgr::PnpBusTypeGuidList)));
}
