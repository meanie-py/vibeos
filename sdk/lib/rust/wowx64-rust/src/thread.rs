//! Threads in an x86-64 guest process: who owns the stack, and how a thread the
//! guest creates becomes a guest thread.
//!
//! STACKS. Every guest thread has two: the guest's (x86-64 frames) and a
//! native one (the CPU layer, its JIT, the syscall path). The TEB's
//! `NtTib.StackBase/StackLimit` can describe only one, and it describes the
//! GUEST's: the guest's amd64 `RtlDispatchException` rejects any frame outside
//! those bounds, so a TEB that describes the native stack silently disables
//! every guest exception handler. The native stack is then outside the
//! bounds the kernel's guard-page growth checks against, so it is committed
//! in full up front and never needs to grow.
//!
//! THREAD CREATION. amd64 kernel32 creates a thread with
//! `NtCreateThread(..., CONTEXT*, INITIAL_TEB*, Suspended)`: the CONTEXT is an
//! x86-64 one (rip = guest `BaseThreadStartup`, rcx/rdx = routine/parameter)
//! and the INITIAL_TEB describes the guest stack kernel32 allocated. Neither
//! may reach the kernel as is — it would start an arm64 thread from x86-64
//! registers. So the guest CONTEXT is parked in a page of its own, the kernel
//! gets a native CONTEXT that starts [`WowX64GuestThreadStart`] on a native
//! stack we allocate, and the INITIAL_TEB goes through unchanged — so the new
//! TEB describes the guest stack, as above. The native loader's thread init
//! runs first (native DLL thread-attach); then the guest's, entered exactly as
//! the initial thread enters it (see `host::EnterGuest`).

use core::ffi::c_void;
use core::ptr::null_mut;

use ntdll_sys::nt::*;

use crate::contract::*;
use crate::host::{die, CURRENT_PROCESS};

const MEM_COMMIT: ULONG = 0x1000;
const MEM_RESERVE: ULONG = 0x2000;
const MEM_RELEASE: ULONG = 0x8000;
const PAGE_READWRITE: ULONG = 0x04;
const STATUS_NO_MEMORY: u64 = 0xC000_0017;

/// The native stack each guest thread's CPU layer runs on. Fully committed (see
/// the module docs); FEX's deepest native frames are its block compiler's.
const NATIVE_STACK_SIZE: SIZE_T = 0x10_0000;

/// Commit the whole of a reserved stack region and drop its guard page, so it
/// never depends on guard-page growth (which checks the TEB's bounds, and those
/// are about to describe the guest's stack instead).
pub unsafe fn CommitWholeStack(Bottom: u64, Top: u64) {
    let mut Base = Bottom as PVOID;
    let mut Size = (Top - Bottom) as SIZE_T;
    if NtAllocateVirtualMemory(CURRENT_PROCESS, &mut Base, 0, &mut Size, MEM_COMMIT, PAGE_READWRITE) < 0 {
        die(b"committing the native stack\0");
    }
    let mut Old: ULONG = 0;
    let (mut Base, mut Size) = (Bottom as PVOID, (Top - Bottom) as SIZE_T);
    NtProtectVirtualMemory(CURRENT_PROCESS, &mut Base, &mut Size, PAGE_READWRITE, &mut Old);
}

/// Publish the guest's stack as this thread's TEB stack (see the module docs).
pub unsafe fn PublishGuestStack(Bottom: u64, Top: u64) {
    let Teb = NtCurrentTeb();
    (*Teb).NtTib.StackBase = Top as PVOID;
    (*Teb).NtTib.StackLimit = Bottom as PVOID;
    (*Teb).DeallocationStack = Bottom as PVOID;
}

/// What a new guest thread starts from, parked between the guest's
/// NtCreateThread and the thread's first native instruction.
#[repr(C, align(16))]
struct ThreadStart {
    Context: WowX64Context,
}

const STATUS_NOT_SUPPORTED: u64 = 0xC000_00BB;
const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;

/// What we can learn about a thread's target process from outside it: the
/// machine of its main image, and whether its loader has run yet (PEB->Ldr).
unsafe fn TargetProcess(Process: HANDLE) -> Option<(u16, bool)> {
    let mut Info: PROCESS_BASIC_INFORMATION = core::mem::zeroed();
    if NtQueryInformationProcess(
        Process,
        ProcessBasicInformation,
        &mut Info as *mut _ as PVOID,
        core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
        null_mut(),
    ) < 0
    {
        return None;
    }
    let Peb = Info.PebBaseAddress as u64;
    let Read = |At: u64, Out: *mut c_void, Len: usize| -> bool {
        NtReadVirtualMemory(Process, At as PVOID, Out, Len as SIZE_T, null_mut()) >= 0
    };
    // 64-bit PEB: Ldr at +0x18, ImageBaseAddress at +0x10.
    let (mut Image, mut Ldr, mut Lfanew, mut Machine) = (0u64, 0u64, 0u32, 0u16);
    if !Read(Peb + 0x10, &mut Image as *mut _ as _, 8)
        || !Read(Peb + 0x18, &mut Ldr as *mut _ as _, 8)
        || !Read(Image + 0x3C, &mut Lfanew as *mut _ as _, 4)
        || !Read(Image + Lfanew as u64 + 4, &mut Machine as *mut _ as _, 2)
    {
        return None;
    }
    Some((Machine, Ldr != 0))
}

unsafe fn Free(Process: HANDLE, Base: PVOID) {
    let (mut B, mut S) = (Base, 0 as SIZE_T);
    NtFreeVirtualMemory(Process, &mut B, &mut S, MEM_RELEASE);
}

/// The guest's `NtCreateThread(Handle, Access, ObjectAttributes, Process,
/// ClientId, Context, InitialTeb, CreateSuspended)`.
///
/// Three cases, by target:
///   * this process, or a RUNNING x86-64 guest (CreateRemoteThread): park the
///     guest CONTEXT in the target and start the thread in the target's
///     [`WowX64GuestThreadStart`] on a native stack allocated there. wowx64.dll
///     sits at its preferred base in every guest process, so our address of
///     that function is the target's too.
///   * a NEW x86-64 process (its loader has not run): this is CreateProcess
///     making the initial thread. Its x86-64 CONTEXT names the guest kernel32's
///     process-start routine, but an x86-64 process never uses its initial
///     context — the native loader diverts to WowX64LdrpInitialize, which
///     enters the image entry itself — so the kernel gets a native context that
///     could only run if that divert ever returned, and then exits the thread.
///   * anything else (a native arm64 or a 32-bit child): would need that
///     machine's start context, which the guest cannot name. Refused, loudly.
pub unsafe fn CreateThread(Args: &mut [u64; crate::syscall::WOWX64_SERVICE_ARGS]) -> u64 {
    let GuestContext = Args[5] as *const WowX64Context;
    let Process = Args[3] as HANDLE;
    if GuestContext.is_null() {
        return crate::host::NativeSyscall(crate::syscall::SVC_NtCreateThread, Args.as_ptr());
    }

    let mut Native: CONTEXT = core::mem::zeroed();
    Native.ContextFlags = CONTEXT_FULL;

    let SelfProcess = Process == CURRENT_PROCESS;
    if !SelfProcess {
        match TargetProcess(Process) {
            Some((IMAGE_FILE_MACHINE_AMD64, true)) => {} // a running guest: below
            Some((IMAGE_FILE_MACHINE_AMD64, false)) => {
                Native.Pc = RtlExitUserThread as usize as u64;
                Native.__bindgen_anon_1.X[0] = 0xC000_007B; // STATUS_INVALID_IMAGE_FORMAT
                Native.Sp = ((*GuestContext).Rsp & !15).wrapping_sub(0x100);
                Args[5] = &Native as *const CONTEXT as u64;
                return crate::host::NativeSyscall(crate::syscall::SVC_NtCreateThread, Args.as_ptr());
            }
            Some((Machine, _)) => {
                crate::trace!("guest NtCreateThread into a machine-%04x process: not supported yet", Machine as u32);
                return STATUS_NOT_SUPPORTED;
            }
            None => {
                crate::trace!("guest NtCreateThread: cannot inspect the target process");
                return STATUS_NOT_SUPPORTED;
            }
        }
    }

    // The parked context and the native stack, in the TARGET.
    let mut Page: PVOID = null_mut();
    let mut Size = core::mem::size_of::<ThreadStart>() as SIZE_T;
    if NtAllocateVirtualMemory(Process, &mut Page, 0, &mut Size, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE) < 0 {
        return STATUS_NO_MEMORY;
    }
    let Start = ThreadStart { Context: *GuestContext };
    if NtWriteVirtualMemory(
        Process,
        Page,
        &Start as *const ThreadStart as PVOID,
        core::mem::size_of::<ThreadStart>() as SIZE_T,
        null_mut(),
    ) < 0
    {
        Free(Process, Page);
        return STATUS_NO_MEMORY;
    }
    let mut Stack: PVOID = null_mut();
    let mut StackSize = NATIVE_STACK_SIZE;
    if NtAllocateVirtualMemory(Process, &mut Stack, 0, &mut StackSize, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE) < 0 {
        Free(Process, Page);
        return STATUS_NO_MEMORY;
    }

    Native.Pc = WowX64GuestThreadStart as usize as u64;
    Native.Sp = (Stack as u64 + StackSize as u64) & !15;
    Native.__bindgen_anon_1.X[0] = Page as u64;
    Args[5] = &Native as *const CONTEXT as u64;
    let Status = crate::host::NativeSyscall(crate::syscall::SVC_NtCreateThread, Args.as_ptr());
    if (Status as NTSTATUS) < 0 {
        Free(Process, Page);
        Free(Process, Stack);
    }
    Status
}

/// A guest-created thread's first native frame, after the native loader's
/// thread init. Never returns: the guest exits the thread with
/// NtTerminateThread.
unsafe extern "C" fn WowX64GuestThreadStart(Start: *mut ThreadStart) -> ! {
    let Context = (*Start).Context;
    let (mut B, mut S) = (Start as PVOID, 0 as SIZE_T);
    NtFreeVirtualMemory(CURRENT_PROCESS, &mut B, &mut S, MEM_RELEASE);

    let Teb = NtCurrentTeb();
    crate::trace!(
        "guest thread %p: entry %p (routine %p), guest stack %p..%p",
        (*Teb).ClientId.UniqueThread,
        Context.Rip as *const c_void,
        Context.Rcx as *const c_void,
        (*Teb).NtTib.StackLimit,
        (*Teb).NtTib.StackBase,
    );
    crate::host::ThreadInit();
    // Not the initial thread: SystemArgument1 is the ntdll base only for that one.
    crate::host::EnterGuest(&Context, 0);
}
