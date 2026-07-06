//! Per-crate write barrier for wmi-rust's `pub static mut` globals — a visible,
//! un-eliminable write site so LLVM can't const-fold a global that's 0-initialized
//! in Rust but written from C/another crate. NEVER CALLED; `#[no_mangle]` + pulled
//! into the link via kernel-rust's `extern crate wmi_rust`. Hand-written for the
//! crate-split; see docs/engineering/ntoskrnl-crate-split-plan.md.
#[allow(unused_unsafe)]
#[allow(dead_code)]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _wmi_global_write_barrier() {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    write_volatile(addr_of_mut!(crate::guidobj::WmipGuidObjectType), read_volatile(addr_of!(crate::guidobj::WmipGuidObjectType)));
    write_volatile(addr_of_mut!(crate::guidobj::WmipGenericMapping), read_volatile(addr_of!(crate::guidobj::WmipGenericMapping)));
    write_volatile(addr_of_mut!(crate::wmidrv::WmipServiceDeviceObject), read_volatile(addr_of!(crate::wmidrv::WmipServiceDeviceObject)));
    write_volatile(addr_of_mut!(crate::wmidrv::WmipAdminDeviceObject), read_volatile(addr_of!(crate::wmidrv::WmipAdminDeviceObject)));
    write_volatile(addr_of_mut!(crate::wmidrv::WmipFastIoDispatch), read_volatile(addr_of!(crate::wmidrv::WmipFastIoDispatch)));
}
