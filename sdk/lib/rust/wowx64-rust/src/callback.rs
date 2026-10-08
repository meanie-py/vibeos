//! Kernel->user callbacks (win32k: window procedures, hooks, ...) into an
//! x86-64 guest.
//!
//! The kernel enters user mode through the NATIVE `KiUserCallbackDispatcher`,
//! nested inside whatever guest syscall made win32k call back (so: inside
//! `syscall.rs`, inside the CPU layer's syscall handler, inside
//! `BTCpu64Simulate`). The table that dispatcher would index is the guest
//! user32's x86-64 code, so the native ntdll hands every callback to
//! [`WowX64UserCallback`] instead (its `WowX64UserCallbackHook` slot).
//!
//! That saves the guest's state — parked on the syscall the callback came out
//! of — runs the guest's own `KiUserCallbackDispatcher` in a NESTED simulate,
//! and lets the guest finish with `NtCallbackReturn`, which arrives back in
//! `syscall.rs` and comes here ([`CallbackReturn`]): restore the saved state,
//! then the native NtCallbackReturn, which unwinds the kernel callout and
//! every native frame of the nested run with it — exactly as on amd64, where
//! the callout's user frames are abandoned the same way. The outer syscall then
//! completes against the restored state as if nothing had run in between.

use core::ffi::c_void;
use core::ptr::null_mut;

use ntdll_sys::nt::*;

use crate::contract::*;
use crate::host::{die, Cpu, CURRENT_PROCESS, CURRENT_THREAD};

/// One active callback on this thread, on the native stack of its hook frame,
/// chained from the thread's record (thread.rs).
struct Callout {
    Saved: WowX64Context,
    /// The outer syscall's published guest rsp and service (thread.rs
    /// `SyscallRsp`): NtCallbackReturn abandons the frames that would restore
    /// them, so the callout does.
    SyscallRsp: u64,
    SyscallService: u64,
    Prev: *mut Callout,
}

/// `UCALLOUT_FRAME` (ndk/amd64/ketypes.h): what the guest's
/// `KiUserCallbackDispatcher` finds at rsp.
#[repr(C)]
struct GuestCalloutFrame {
    P1Home: u64,
    P2Home: u64,
    P3Home: u64,
    P4Home: u64,
    Buffer: u64,
    Length: u32,
    ApiNumber: u32,
    // MACHINE_FRAME
    Rip: u64,
    SegCs: u64,
    EFlags: u64,
    Rsp: u64,
    SegSs: u64,
}
const _: () = assert!(core::mem::size_of::<GuestCalloutFrame>() == 0x58);

static mut KiUserCallbackDispatcher: u64 = 0;

/// Point the native dispatcher at us. Dies if the native ntdll has no slot: a
/// guest window procedure would otherwise be called as arm64 code.
pub unsafe fn Install(GuestDispatcher: u64) {
    KiUserCallbackDispatcher = GuestDispatcher;
    let Slot = crate::host::NativeNtdllExport(b"WowX64UserCallbackHook\0");
    *(Slot as *mut Option<unsafe extern "system" fn(ULONG, *mut c_void, ULONG) -> NTSTATUS>) =
        Some(WowX64UserCallback);
}

unsafe fn Head() -> *mut *mut Callout {
    core::ptr::addr_of_mut!((*crate::thread::Record()).Callouts) as *mut *mut Callout
}

unsafe extern "system" fn WowX64UserCallback(Index: ULONG, Argument: *mut c_void, Length: ULONG) -> NTSTATUS {
    let Rec = crate::thread::Record();
    let mut C = Callout {
        Saved: core::mem::zeroed(),
        SyscallRsp: (*Rec).SyscallRsp,
        SyscallService: (*Rec).SyscallService,
        Prev: *Head(),
    };
    // Guest code runs from here until NtCallbackReturn: not parked in a
    // syscall, so a remote sampler is refused rather than shown the outer one.
    core::ptr::write_volatile(&mut (*Rec).SyscallRsp, 0);
    if (Cpu().GetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), &mut C.Saved) < 0 {
        die(b"win32k callback on a thread with no guest state\0");
    }
    *Head() = &mut C;

    // The callout frame goes below the guest's current stack, as the amd64
    // kernel's KiCallUserMode puts it. (The argument buffer stays where the
    // native kernel copied it, on this native stack: the same address space,
    // alive until NtCallbackReturn.)
    let Frame = ((C.Saved.Rsp - 0x100 - core::mem::size_of::<GuestCalloutFrame>() as u64) & !15) as *mut GuestCalloutFrame;
    core::ptr::write_bytes(Frame as *mut u8, 0, core::mem::size_of::<GuestCalloutFrame>());
    (*Frame).Buffer = Argument as u64;
    (*Frame).Length = Length;
    (*Frame).ApiNumber = Index;
    (*Frame).Rip = C.Saved.Rip;
    (*Frame).SegCs = C.Saved.SegCs as u64;
    (*Frame).EFlags = C.Saved.EFlags as u64;
    (*Frame).Rsp = C.Saved.Rsp;
    (*Frame).SegSs = C.Saved.SegSs as u64;

    let mut Run = C.Saved;
    Run.ContextFlags = WOWX64_CONTEXT_CONTROL | WOWX64_CONTEXT_INTEGER;
    Run.Rip = KiUserCallbackDispatcher;
    Run.Rsp = Frame as u64;
    (Cpu().SetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), &Run);
    (Cpu().Simulate)();
    die(b"guest callback returned without NtCallbackReturn\0");
}

/// The guest's `NtCallbackReturn(Result, ResultLength, Status)`.
pub unsafe fn CallbackReturn(Args: &[u64; crate::syscall::WOWX64_SERVICE_ARGS]) -> u64 {
    let C = *Head();
    if C.is_null() {
        // Not inside a callback we started: let the kernel say so.
        return crate::host::NativeSyscall(crate::syscall::svc::SVC_NtCallbackReturn, Args.as_ptr());
    }
    *Head() = (*C).Prev;
    let Rec = crate::thread::Record();
    (*Rec).SyscallService = (*C).SyscallService;
    core::ptr::write_volatile(&mut (*Rec).SyscallRsp, (*C).SyscallRsp);
    (Cpu().SetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), &(*C).Saved);
    let Status = crate::host::NativeSyscall(crate::syscall::svc::SVC_NtCallbackReturn, Args.as_ptr());
    // Only an error comes back. The guest is now parked on the OUTER syscall,
    // so this status is that syscall's result, which is as good an outcome as
    // exists for a callback return the kernel refused.
    crate::trace!("NtCallbackReturn failed %08lx", Status as u32);
    Status
}
