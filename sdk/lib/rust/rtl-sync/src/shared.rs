//! Shared substrate for the rtl-sync cluster: helpers + constants/types the
//! per-file translations reference but ntkrnl-sys doesn't surface directly.
use core::ffi::c_void;
use ntkrnl_sys::ddk::PTEB;

// NtCurrentTeb — ddk declares it out-of-line (`_NtCurrentTeb`) but it's really an
// fs/gs/x18 intrinsic with no defined symbol, so provide the inline-asm version.
#[cfg(target_arch = "x86")]
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn NtCurrentTeb() -> PTEB {
    let teb: PTEB;
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb, options(nostack, pure, readonly));
    teb
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn NtCurrentTeb() -> PTEB {
    let teb: PTEB;
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb, options(nostack, pure, readonly));
    teb
}
#[cfg(target_arch = "aarch64")]
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn NtCurrentTeb() -> PTEB {
    let teb: PTEB;
    core::arch::asm!("mov {}, x18", out(reg) teb, options(nostack, pure, nomem));
    teb
}

pub type LPVOID = *mut c_void;

// winbase.h / ntstatus.h / winnt.h
pub const INFINITE: u32 = 0xFFFF_FFFF;
pub const STATUS_USER_APC: i32 = 0x0000_00C0; // NTSTATUS
pub const EVENT_ALL_ACCESS: u32 = 0x001F_0003;

// winnt.h WT_* thread-pool wait flags
pub const WT_EXECUTEDEFAULT: u32 = 0x0000_0000;
pub const WT_EXECUTEINIOTHREAD: u32 = 0x0000_0001;
pub const WT_EXECUTEINUITHREAD: u32 = 0x0000_0002;
pub const WT_EXECUTEINWAITTHREAD: u32 = 0x0000_0004;
pub const WT_EXECUTEONLYONCE: u32 = 0x0000_0008;
pub const WT_EXECUTELONGFUNCTION: u32 = 0x0000_0010;
pub const WT_EXECUTEINTIMERTHREAD: u32 = 0x0000_0020;
pub const WT_EXECUTEINPERSISTENTIOTHREAD: u32 = 0x0000_0040;
pub const WT_EXECUTEINPERSISTENTTHREAD: u32 = 0x0000_0080;
pub const WT_TRANSFER_IMPERSONATION: u32 = 0x0000_0100;

// WAIT_TYPE::WaitAny (NtWaitForMultipleObjects WaitType arg)
pub use ntkrnl_sys::ddk::_WAIT_TYPE_WaitAny as WaitAny;

// C `#define RtlGetProcessHeap() NtCurrentPeb()->ProcessHeap` — crate-private inline
// (NOT #[no_mangle]; ntdll-rust owns the RtlGetProcessHeap export symbol).
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn RtlGetProcessHeap() -> *mut c_void {
    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap as *mut c_void
}

// C `#define GetProcessHeap() RtlGetProcessHeap()`
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn GetProcessHeap() -> *mut c_void {
    RtlGetProcessHeap()
}

// winnt.h LongToPtr(l) = (PVOID)(LONG_PTR)(LONG)l
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn LongToPtr(l: i32) -> *mut c_void {
    l as isize as *mut c_void
}
