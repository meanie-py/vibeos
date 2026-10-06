//! User-mode APCs whose routine is guest code: `QueueUserAPC`, the completion
//! routine of `ReadFileEx`/`WriteFileEx`, any `NtQueueApcThread` the guest
//! makes with one of its own functions.
//!
//! The kernel delivers every user APC through the NATIVE `KiUserApcDispatcher`,
//! which would `blr` straight into x86-64 code. So the native dispatcher asks
//! us first (its `WowX64UserApcHook` slot, dispatch.S). A native routine —
//! the native loader's own thread-start APC, anything in an arm64 image —
//! goes back and runs as before. A guest one runs the way `callback.rs` runs
//! a win32k callback: the guest's state is parked where the APC found it, a
//! `UAPC_FRAME` is built below the guest's rsp exactly as the amd64 kernel's
//! `KiInitializeUserApc` builds one, and the guest's OWN `KiUserApcDispatcher`
//! runs in a nested simulate. It ends in `NtContinue(&Frame->Context, TRUE)`,
//! which `syscall.rs` brings to [`Continue`]: restore the guest to where the
//! APC found it, then the native NtContinue of the context the kernel gave the
//! native dispatcher — which abandons every native frame of the nested run and
//! resumes the guest syscall the APC interrupted (returning its result and
//! draining any further APCs first), as the amd64 kernel's NtContinue abandons
//! the APC's user frames.
//!
//! WHERE THE GUEST WAS. A user APC is delivered on the way out of the kernel,
//! and for a guest thread the only way into the kernel is a guest syscall, so
//! the guest is parked on one: rip ON the `syscall` (the CPU layer advances it
//! when the call returns), rcx its return address. The frame's CONTEXT must be
//! that syscall COMPLETED — rip past it, rax its status, as the amd64 trap
//! frame says — because that is what the guest NtContinues to. The one other
//! way in is a guest `NtContinue(..., TestAlert = TRUE)`, which we turn into a
//! native NtTestAlert after installing the new context; there the guest is not
//! in a syscall at all, the installed context is exactly what the APC
//! interrupts, and a [`KIND_CONTINUE`] marker on the chain says so.

use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};

use ntdll_sys::nt::*;

use crate::contract::*;
use crate::host::{die, Cpu, CURRENT_PROCESS, CURRENT_THREAD};

const KIND_APC: u32 = 0;
const KIND_CONTINUE: u32 = 1;

/// One guest APC in progress on this thread (or a TestAlert marker), on the
/// native stack of the frame that pushed it, chained from the thread's record
/// (thread.rs).
struct Pending {
    Kind: u32,
    /// The guest CONTEXT the guest's dispatcher will NtContinue to.
    FrameContext: u64,
    /// The native CONTEXT the kernel delivered the APC with. Resuming it is
    /// what actually continues the guest (the CPU layer's syscall path writes
    /// the result back and returns to the guest) — so this, not a SetContext,
    /// is the resume.
    Native: PCONTEXT,
    /// The guest's state as the APC found it. The kernel drains several APCs at
    /// one syscall boundary, re-entering the native dispatcher per APC with the
    /// SAME interrupted `Native` — so each must build its frame from the same
    /// guest context. We restore this before handing back to `Native`, so the
    /// next APC's `GetContext` sees the interrupted state, not this APC's
    /// dispatcher-end one.
    Saved: WowX64Context,
    Prev: *mut Pending,
}

/// `UAPC_FRAME` (ntoskrnl ke/amd64): what the guest's `KiUserApcDispatcher`
/// finds at rsp — a CONTEXT whose P1-P4 homes carry the APC, then the
/// machine frame its unwind info describes.
#[repr(C)]
struct GuestApcFrame {
    Context: WowX64Context,
    Rip: u64,
    SegCs: u64,
    EFlags: u64,
    Rsp: u64,
    SegSs: u64,
}
// 0x4D0 + 0x28, padded to the CONTEXT's 16-byte alignment — as the kernel's is.
const _: () = assert!(core::mem::size_of::<GuestApcFrame>() == 0x500);

unsafe fn Head() -> *mut *mut Pending {
    addr_of_mut!((*crate::thread::Record()).Apcs) as *mut *mut Pending
}

/// Point the native dispatcher at us. Dies if the native ntdll has no slot: a
/// guest completion routine would otherwise be called as arm64 code.
pub unsafe fn Install() {
    let Slot = crate::host::NativeNtdllExport(b"WowX64UserApcHook\0");
    *(Slot as *mut Option<unsafe extern "system" fn(u64, u64, u64, u64, PCONTEXT)>) = Some(WowX64UserApc);
}

/// Is `Address` code the guest runs? Everything except the modules the NATIVE
/// loader loaded (its own list — two loaders share this process, see
/// ntdll-rust `LdrpLdrData`): ntdll, wowx64.dll, the CPU layer. The guest's
/// images, which only its own loader knows, and its runtime-generated code
/// are all the guest's. (Not the region's image type and machine: the native
/// ntdll's own thread-start APC did not read back as an arm64 image, and was
/// run as x86-64 code.)
unsafe fn IsGuestCode(Address: u64) -> bool {
    let mut Base: PVOID = null_mut();
    RtlPcToFileHeader(Address as PVOID, &mut Base).is_null()
}

/// The native `KiUserApcDispatcher`'s first call. Returns only when the routine
/// is native, for the dispatcher to run as usual.
unsafe extern "system" fn WowX64UserApc(NormalContext: u64, Argument1: u64, Argument2: u64, Routine: u64, Native: PCONTEXT) {
    if Routine == 0 || !IsGuestCode(Routine) {
        return;
    }
    let mut Guest: WowX64Context = core::mem::zeroed();
    if (Cpu().GetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), &mut Guest) < 0 {
        // A guest thread that has not entered the guest yet: an APC queued to a
        // new suspended thread, delivered by the native loader's thread-init
        // NtTestAlert. Queue it again; the guest loader's own NtTestAlert, made
        // once this thread has guest state, delivers it where it can run.
        let Status = NtQueueApcThread(CURRENT_THREAD, core::mem::transmute(Routine), NormalContext as PVOID, Argument1 as PVOID, Argument2 as PVOID);
        if Status < 0 {
            crate::trace!("guest APC %p before guest entry: requeue failed %08lx", Routine as *const c_void, Status);
        }
        NtContinue(Native, 0);
        die(b"NtContinue of a deferred APC returned\0");
    }

    let Saved = Guest;
    let InSyscall = (*Head()).is_null() || (**Head()).Kind != KIND_CONTINUE;
    if InSyscall {
        Guest.Rip = Guest.Rcx;
        Guest.Rax = (*Native).__bindgen_anon_1.X[0];
    }
    let Frame = ((Guest.Rsp - core::mem::size_of::<GuestApcFrame>() as u64) & !15) as *mut GuestApcFrame;
    core::ptr::write_bytes(Frame as *mut u8, 0, core::mem::size_of::<GuestApcFrame>());
    (*Frame).Context = Guest;
    (*Frame).Context.ContextFlags = WOWX64_CONTEXT_ALL;
    (*Frame).Context.P1Home = NormalContext;
    (*Frame).Context.P2Home = Argument1;
    (*Frame).Context.P3Home = Argument2;
    (*Frame).Context.P4Home = Routine;
    (*Frame).Rip = Guest.Rip;
    (*Frame).SegCs = Guest.SegCs as u64;
    (*Frame).EFlags = Guest.EFlags as u64;
    (*Frame).Rsp = Guest.Rsp;
    (*Frame).SegSs = Guest.SegSs as u64;

    let mut P = Pending { Kind: KIND_APC, FrameContext: &(*Frame).Context as *const _ as u64, Native, Saved, Prev: *Head() };
    *Head() = &mut P;

    let mut Run = Guest;
    Run.ContextFlags = WOWX64_CONTEXT_CONTROL | WOWX64_CONTEXT_INTEGER;
    Run.Rip = crate::host::GuestApcDispatcher();
    Run.Rsp = Frame as u64;
    (Cpu().SetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), &Run);
    (Cpu().Simulate)();
    die(b"guest APC returned without NtContinue\0");
}

/// The guest's `NtContinue(Context, TestAlert)`.
pub unsafe fn Continue(Args: &[u64; crate::syscall::WOWX64_SERVICE_ARGS]) -> u64 {
    let Context = Args[0] as *const WowX64Context;
    let TestAlert = Args[1] as u8;
    let P = *Head();
    if !P.is_null() && (*P).Kind == KIND_APC && (*P).FrameContext == Args[0] {
        // The end of a guest APC we started. Restore the guest to where the APC
        // found it — not to this frame's dispatcher-end `Context` — so the next
        // APC of the same drain reads the interrupted state. The actual resume
        // is `Native`: the CPU layer's syscall path returns the result to the
        // guest, and if more APCs are pending TestAlert delivers them first.
        let _ = Context;
        *Head() = (*P).Prev;
        (Cpu().SetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), &(*P).Saved);
        let Status = NtContinue((*P).Native, TestAlert);
        crate::trace!("NtContinue out of a guest APC failed %08lx", Status);
        die(b"NtContinue out of a guest APC returned\0");
    }

    // The guest's KiUserApcDispatcher / exception dispatcher resuming a saved
    // x86-64 context. The CPU layer installs it; the handler sees rip changed
    // and does not overwrite it with this call's return.
    let Status = (Cpu().SetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), Context);
    if Status >= 0 && TestAlert != 0 {
        let mut Marker = Pending {
            Kind: KIND_CONTINUE,
            FrameContext: 0,
            Native: null_mut(),
            Saved: core::mem::zeroed(),
            Prev: *Head(),
        };
        *Head() = &mut Marker;
        NtTestAlert();
        *Head() = Marker.Prev;
    }
    Status as u32 as u64
}
