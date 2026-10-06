//! Guest exceptions: delivered to the guest's OWN amd64 ntdll, which dispatches
//! them against the guest's `.pdata` exactly as on amd64.
//!
//! Two ways in, one way out. A hardware fault in guest code surfaces as a
//! native exception that the CPU layer restates in guest terms (the VEH below);
//! a software raise is the guest's `NtRaiseException` (amd64 RtlRaiseException
//! dispatches in user mode first, so that syscall means "deliver this" or, on
//! second chance, "nobody handled it"). Both end in `Deliver`, which builds
//! the frame the amd64 kernel's `KiDispatchExceptionToUser` builds — a
//! `KUSER_EXCEPTION_STACK` below the faulting rsp — and points the guest at its
//! `KiUserExceptionDispatcher`.

use core::ffi::c_void;
use core::ptr::null_mut;

use ntdll_sys::nt::*;

use crate::contract::*;
use crate::host::{Cpu, CURRENT_PROCESS, CURRENT_THREAD};

const EXCEPTION_CONTINUE_SEARCH: LONG = 0;
const EXCEPTION_CONTINUE_EXECUTION: LONG = -1;
const STATUS_BREAKPOINT: u32 = 0x8000_0003;
const BREAKPOINT_PRINT: u64 = 1;

/// The amd64 EXCEPTION_RECORD — the native one has the same layout (both are
/// the 64-bit shape), restated so the frame below is self-describing.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GuestExceptionRecord {
    pub ExceptionCode: u32,
    pub ExceptionFlags: u32,
    pub ExceptionRecord: u64,
    pub ExceptionAddress: u64,
    pub NumberParameters: u32,
    pub _Pad: u32,
    pub ExceptionInformation: [u64; 15],
}

#[repr(C)]
struct MachineFrame {
    Rip: u64,
    SegCs: u16,
    Fill1: [u16; 3],
    EFlags: u32,
    Fill2: u32,
    Rsp: u64,
    SegSs: u16,
    Fill3: [u16; 3],
}

/// `KUSER_EXCEPTION_STACK` (ndk/amd64/ketypes.h): what the guest's
/// KiUserExceptionDispatcher finds at rsp.
#[repr(C)]
struct UserExceptionStack {
    Context: WowX64Context,
    ExceptionRecord: GuestExceptionRecord,
    Alignment: u64,
    MachineFrame: MachineFrame,
}

const _: () = assert!(core::mem::size_of::<GuestExceptionRecord>() == 0x98);
const _: () = assert!(core::mem::size_of::<MachineFrame>() == 0x28);
const _: () = assert!(core::mem::offset_of!(UserExceptionStack, ExceptionRecord) == 0x4D0);

static mut KiUserExceptionDispatcher: u64 = 0;
pub static VehCount: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

pub unsafe fn SetGuestDispatcher(Address: u64) {
    KiUserExceptionDispatcher = Address;
}

/// Point the current guest thread at its exception dispatcher with `Rec`
/// raised in `Ctx`. Dies if the guest has no dispatcher: an exception with
/// nowhere to go must not be swallowed.
pub unsafe fn Deliver(Rec: &GuestExceptionRecord, Ctx: &WowX64Context) {
    if KiUserExceptionDispatcher == 0 {
        crate::host::die(b"guest exception before the guest ntdll was resolved\0");
    }
    // Rare by nature (each one is an exception in guest code), so always said:
    // a guest that dies of a handled-then-rethrown fault is otherwise silent.
    crate::trace!(
        "guest exception %08lx at %p (rsp %p, info %p %p) -> guest dispatcher",
        Rec.ExceptionCode,
        Rec.ExceptionAddress as *const c_void,
        Ctx.Rsp as *const c_void,
        Rec.ExceptionInformation[0] as *const c_void,
        Rec.ExceptionInformation[1] as *const c_void,
    );
    let Frame = ((Ctx.Rsp - core::mem::size_of::<UserExceptionStack>() as u64) & !15) as *mut UserExceptionStack;
    core::ptr::write_bytes(Frame as *mut u8, 0, core::mem::size_of::<UserExceptionStack>());
    (*Frame).Context = *Ctx;
    (*Frame).Context.ContextFlags = WOWX64_CONTEXT_ALL;
    (*Frame).ExceptionRecord = *Rec;
    (*Frame).MachineFrame.Rip = Ctx.Rip;
    (*Frame).MachineFrame.SegCs = Ctx.SegCs;
    (*Frame).MachineFrame.EFlags = Ctx.EFlags;
    (*Frame).MachineFrame.Rsp = Ctx.Rsp;
    (*Frame).MachineFrame.SegSs = Ctx.SegSs;

    let mut Next = *Ctx;
    Next.ContextFlags = WOWX64_CONTEXT_ALL;
    Next.Rip = KiUserExceptionDispatcher;
    Next.Rsp = Frame as u64;
    Next.Rcx = &(*Frame).Context as *const _ as u64;
    Next.Rdx = &(*Frame).ExceptionRecord as *const _ as u64;
    Next.EFlags &= !0x400; // the dispatcher runs with DF clear, as after the kernel's
    (Cpu().SetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), &Next);
}

/// The guest's `NtRaiseException(Record, Context, FirstChance)`.
pub unsafe fn RaiseException(Rec: *const GuestExceptionRecord, Ctx: *const WowX64Context, FirstChance: bool) -> u64 {
    if FirstChance {
        Deliver(&*Rec, &*Ctx);
        return 0;
    }
    // Second chance: the guest's own dispatch found no handler. With no
    // debugger that is the end of the process, as on amd64.
    crate::trace!(
        "unhandled guest exception %08lx at %p (second chance) — terminating",
        (*Rec).ExceptionCode,
        (*Rec).ExceptionAddress as *const c_void,
    );
    DumpGuest(&*Ctx);
    NtTerminateProcess(CURRENT_PROCESS, (*Rec).ExceptionCode as NTSTATUS);
    0
}

pub unsafe fn DumpGuest(Ctx: &WowX64Context) {
    crate::trace!(
        "  rip=%p rsp=%p rax=%p rcx=%p rdx=%p rbx=%p rsi=%p rdi=%p rbp=%p",
        Ctx.Rip as *const c_void,
        Ctx.Rsp as *const c_void,
        Ctx.Rax as *const c_void,
        Ctx.Rcx as *const c_void,
        Ctx.Rdx as *const c_void,
        Ctx.Rbx as *const c_void,
        Ctx.Rsi as *const c_void,
        Ctx.Rdi as *const c_void,
        Ctx.Rbp as *const c_void,
    );
    crate::trace!(
        "  r8=%p r9=%p r10=%p r11=%p r12=%p r13=%p r14=%p r15=%p",
        Ctx.R8 as *const c_void,
        Ctx.R9 as *const c_void,
        Ctx.R10 as *const c_void,
        Ctx.R11 as *const c_void,
        Ctx.R12 as *const c_void,
        Ctx.R13 as *const c_void,
        Ctx.R14 as *const c_void,
        Ctx.R15 as *const c_void,
    );
}

/// Is the guest stopped just past `int 2d` of the amd64 `DebugService`
/// (`int 2d; int 3`)? FEX reports that interrupt as a breakpoint with rip
/// already past it, i.e. on the trailing `int 3`.
unsafe fn AtDebugService(Rip: u64) -> bool {
    Rip >= 2 && *(Rip as *const u8) == 0xCC && *((Rip - 2) as *const u8) == 0xCD && *((Rip - 1) as *const u8) == 0x2D
}

/// Every native exception, first. The CPU layer resolves its own (overcommit,
/// SMC, unaligned atomics) and NtContinues without returning. If it comes
/// back having REWRITTEN the host context, the fault was the guest's: the CPU
/// layer has restated the record in guest terms and parked the host on the
/// context captured at Simulate entry, so resuming re-enters the guest from
/// whatever guest state we leave. Otherwise it is a host fault — not ours.
pub unsafe extern "system" fn WowX64Veh(Info: *mut _EXCEPTION_POINTERS) -> LONG {
    let Cpu = match crate::host::CpuIfLoaded() {
        Some(c) => c,
        None => return EXCEPTION_CONTINUE_SEARCH,
    };
    let HostPc = (*(*Info).ContextRecord).Pc;
    let HostSp = (*(*Info).ContextRecord).Sp;
    // Every exception, for the first few: the CPU layer resolves most of them
    // (overcommit, unaligned atomics, SMC) by NtContinue from inside Reset, so
    // the ones that END a guest are otherwise preceded by nothing at all.
    let N = VehCount.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    if N < 64 {
        crate::trace!(
            "veh #%u: %08lx at pc %p lr %p (address %p)",
            N + 1,
            (*(*Info).ExceptionRecord).ExceptionCode,
            HostPc as *const c_void,
            (*(*Info).ContextRecord).__bindgen_anon_1.X[30] as *const c_void,
            (*(*Info).ExceptionRecord).ExceptionInformation[1] as *const c_void,
        );
    }
    (Cpu.ResetToConsistentState)(Info);
    if (*(*Info).ContextRecord).Pc == HostPc && (*(*Info).ContextRecord).Sp == HostSp {
        // A host fault: passed on, and said, because in a guest process a host
        // fault the CPU layer did not claim is about to end it.
        crate::trace!(
            "host exception %08lx at pc %p (address %p), CPU-layer thread state %p — not the guest's",
            (*(*Info).ExceptionRecord).ExceptionCode,
            HostPc as *const c_void,
            (*(*Info).ExceptionRecord).ExceptionInformation[1] as *const c_void,
            (*NtCurrentTeb()).WOW32Reserved,
        );
        return EXCEPTION_CONTINUE_SEARCH;
    }

    let Rec = (*Info).ExceptionRecord;
    let mut Ctx: WowX64Context = core::mem::zeroed();
    if (Cpu.GetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), &mut Ctx) < 0 {
        crate::host::die(b"guest fault on a thread with no guest state\0");
    }

    if (*Rec).ExceptionCode as u32 == STATUS_BREAKPOINT && AtDebugService(Ctx.Rip) {
        // The guest's DbgPrint channel (its loader's diagnostics among
        // others): print it the way the amd64 kernel's debug service would,
        // and resume past the `int 3` with STATUS_SUCCESS in eax.
        if Ctx.Rax == BREAKPOINT_PRINT && Ctx.Rcx != 0 && Ctx.Rdx < 0x1000 {
            // The guest's DbgPrint set the shared InDbgPrint flag (lib.rs, trace!).
            let Teb = NtCurrentTeb();
            let Saved = (*Teb).InDbgPrint;
            (*Teb).InDbgPrint = 0;
            DbgPrint(b"[GUEST64] %.*s\0".as_ptr() as PCSTR, Ctx.Rdx as i32, Ctx.Rcx as *const i8);
            (*Teb).InDbgPrint = Saved;
        }
        Ctx.Rip += 1;
        Ctx.Rax = 0;
        (Cpu.SetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), &Ctx);
        return EXCEPTION_CONTINUE_EXECUTION;
    }

    let mut G: GuestExceptionRecord = core::mem::zeroed();
    G.ExceptionCode = (*Rec).ExceptionCode as u32;
    G.ExceptionFlags = (*Rec).ExceptionFlags;
    G.ExceptionAddress = (*Rec).ExceptionAddress as u64;
    G.NumberParameters = (*Rec).NumberParameters.min(15);
    for i in 0..G.NumberParameters as usize {
        G.ExceptionInformation[i] = (*Rec).ExceptionInformation[i] as u64;
    }
    if G.ExceptionCode == STATUS_BREAKPOINT {
        // amd64 reports an int3 AT the instruction: the kernel backs rip up.
        Ctx.Rip = G.ExceptionAddress;
    }
    Deliver(&G, &Ctx);
    EXCEPTION_CONTINUE_EXECUTION
}
