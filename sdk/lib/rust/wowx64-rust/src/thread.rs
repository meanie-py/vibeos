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
//!
//! THREAD EXIT. What a guest thread owns beyond its guest stack — the CPU
//! layer's state for it and the native stack we allocated — is ours to free,
//! and the place to do it is the guest's `NtTerminateThread` ([`TerminateThread`]).
//! The native stack is freed the way the kernel already frees a thread's stack:
//! the TEB's `DeallocationStack` + `FreeStackOnTermination`, which kernel32's
//! thread start sets for the guest stack. A thread ending itself frees the
//! guest stack on the spot (nothing of the guest runs again) and hands the
//! kernel the native one instead.

use core::ffi::c_void;
use core::ptr::null_mut;

use ntdll_sys::nt::*;

use crate::contract::*;
use crate::host::{die, Cpu, CURRENT_PROCESS};

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

/// Per-thread wowx64 state, found through the TEB's `DeallocationBStore` (an
/// IA-64 register-stack field nothing on amd64 or arm64 reads). For a thread
/// we created it sits at the top of the native stack we allocated; the initial
/// thread's is static.
#[repr(C)]
pub struct ThreadRecord {
    /// The chain of active win32k callbacks (callback.rs).
    pub Callouts: *mut c_void,
    /// The chain of active guest APCs (apc.rs).
    pub Apcs: *mut c_void,
    /// Allocation base of the native stack, if we allocated it; 0 for the
    /// initial thread, whose native stack is the process's own.
    pub NativeStack: u64,
}

static mut INITIAL_THREAD: ThreadRecord = ThreadRecord { Callouts: null_mut(), Apcs: null_mut(), NativeStack: 0 };

unsafe fn SetRecord(R: *mut ThreadRecord) {
    (*NtCurrentTeb()).DeallocationBStore = R as PVOID;
}

/// The initial thread's record, before its first guest instruction.
pub unsafe fn InitialThreadRecord() {
    SetRecord(core::ptr::addr_of_mut!(INITIAL_THREAD));
}

/// This thread's record. Every guest thread has one before it runs guest code.
pub unsafe fn Record() -> *mut ThreadRecord {
    let R = (*NtCurrentTeb()).DeallocationBStore as *mut ThreadRecord;
    if R.is_null() {
        die(b"guest thread with no wowx64 thread record\0");
    }
    R
}

/// What a new guest thread starts from, parked between the guest's
/// NtCreateThread and the thread's first native instruction.
#[repr(C, align(16))]
struct ThreadStart {
    Context: WowX64Context,
}

const STATUS_NOT_SUPPORTED: u64 = 0xC000_00BB;
const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const IMAGE_FILE_MACHINE_I386: u16 = 0x014C;

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

/// `DllBase` of `Name` (lower-case ASCII base name) in `Process`'s 64-bit
/// loader list, PEB->Ldr: in a native or WoW64 process the native loader's,
/// in this one the GUEST's (the guest ntdll owns PEB->Ldr; the native loader
/// keeps its own list, see ntdll-rust `LdrpLdrData`). Offsets as wow64.c's
/// cross-process walk measured them against the bindgen structs.
unsafe fn RemoteModuleBase(Process: HANDLE, Name: &[u8]) -> Option<u64> {
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
    let Read = |At: u64, Out: *mut c_void, Len: usize| -> bool {
        NtReadVirtualMemory(Process, At as PVOID, Out, Len as SIZE_T, null_mut()) >= 0
    };
    let mut Ldr = 0u64;
    if !Read(Info.PebBaseAddress as u64 + 0x18, &mut Ldr as *mut _ as _, 8) || Ldr == 0 {
        return None;
    }
    let Head = Ldr + 0x10; // PEB_LDR_DATA.InLoadOrderModuleList
    let mut Cur = 0u64;
    if !Read(Head, &mut Cur as *mut _ as _, 8) {
        return None;
    }
    // A corrupt list must not spin forever; no process has this many modules.
    for _ in 0..512 {
        if Cur == 0 || Cur == Head {
            break;
        }
        // InLoadOrderLinks sits at offset 0, so the link IS the entry.
        let (mut DllBase, mut Len, mut Buf) = (0u64, 0u16, 0u64);
        let mut Wide = [0u16; 32];
        if Read(Cur + 0x30, &mut DllBase as *mut _ as _, 8)
            && Read(Cur + 0x58, &mut Len as *mut _ as _, 2)
            && Read(Cur + 0x60, &mut Buf as *mut _ as _, 8)
            && Len as usize == Name.len() * 2
            && Name.len() <= Wide.len()
            && Read(Buf, Wide.as_mut_ptr() as _, Len as usize)
            && Wide[..Name.len()]
                .iter()
                .zip(Name)
                .all(|(&W, &C)| W < 0x80 && (W as u8).to_ascii_lowercase() == C)
        {
            return Some(DllBase);
        }
        if !Read(Cur, &mut Cur as *mut _ as _, 8) {
            break;
        }
    }
    None
}

/// The address of export `Name` (no NUL) of the PE32+ image at `Base` in
/// `Process`. The export directory -- its tables and their name strings, which
/// the linker emits together -- is read once into a local buffer, and any RVA
/// pointing outside it (a forwarder, a malformed table) is refused, so this
/// never dereferences an address it did not bound-check.
unsafe fn RemoteExport(Process: HANDLE, Base: u64, Name: &[u8]) -> Option<u64> {
    let Read = |At: u64, Out: *mut c_void, Len: usize| -> bool {
        NtReadVirtualMemory(Process, At as PVOID, Out, Len as SIZE_T, null_mut()) >= 0
    };
    let (mut Lfanew, mut Magic, mut Dir) = (0u32, 0u16, [0u32; 2]);
    // OptionalHeader at e_lfanew + 24; PE32+ DataDirectory[EXPORT] at +112.
    if !Read(Base + 0x3C, &mut Lfanew as *mut _ as _, 4)
        || !Read(Base + Lfanew as u64 + 24, &mut Magic as *mut _ as _, 2)
        || Magic != 0x20B
        || !Read(Base + Lfanew as u64 + 24 + 112, Dir.as_mut_ptr() as _, 8)
    {
        return None;
    }
    let (Rva, Size) = (Dir[0] as usize, Dir[1] as usize);
    if Rva == 0 || Size < 40 || Size > 0x40_0000 {
        return None;
    }
    let mut Block: PVOID = null_mut();
    let mut BlockSize = Size as SIZE_T;
    if NtAllocateVirtualMemory(CURRENT_PROCESS, &mut Block, 0, &mut BlockSize, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE) < 0 {
        return None;
    }
    let Found = (|| {
        if !Read(Base + Rva as u64, Block, Size) {
            return None;
        }
        let B = Block as *const u8;
        // The bytes [At, At+Len) of the image, if they lie inside the block.
        let At = |R: usize, Len: usize| -> Option<*const u8> {
            (R >= Rva && R.checked_add(Len)? <= Rva + Size).then(|| B.add(R - Rva))
        };
        let U32 = |R: usize| At(R, 4).map(|P| core::ptr::read_unaligned(P as *const u32) as usize);
        let NumberOfFunctions = U32(Rva + 0x14)?;
        let NumberOfNames = U32(Rva + 0x18)?;
        let (Functions, Names, Ordinals) = (U32(Rva + 0x1C)?, U32(Rva + 0x20)?, U32(Rva + 0x24)?);
        for i in 0..NumberOfNames {
            let NameRva = U32(Names + i * 4)?;
            let Candidate = match At(NameRva, Name.len() + 1) {
                Some(P) => core::slice::from_raw_parts(P, Name.len() + 1),
                None => continue,
            };
            if &Candidate[..Name.len()] != Name || Candidate[Name.len()] != 0 {
                continue;
            }
            let Ord = core::ptr::read_unaligned(At(Ordinals + i * 2, 2)? as *const u16) as usize;
            if Ord >= NumberOfFunctions {
                return None;
            }
            let FnRva = U32(Functions + Ord * 4)?;
            // An RVA inside the export directory is a forwarder string.
            if FnRva == 0 || (FnRva >= Rva && FnRva < Rva + Size) {
                return None;
            }
            return Some(Base + FnRva as u64);
        }
        None
    })();
    Free(CURRENT_PROCESS, Block);
    Found
}

/// The guest kernel32's Win32 thread start (its `BaseThreadStart` export, of
/// `BaseThreadStartup`): the `rip` of every x86-64 CONTEXT kernel32 builds for
/// a new thread, with `rcx`/`rdx` = routine/parameter. 0 until the guest has a
/// kernel32. Cached once found: the guest kernel32 never moves.
unsafe fn GuestThreadStart() -> u64 {
    static mut CACHED: u64 = 0;
    if CACHED == 0 {
        if let Some(Base) = RemoteModuleBase(CURRENT_PROCESS, b"kernel32.dll") {
            CACHED = RemoteExport(CURRENT_PROCESS, Base, b"BaseThreadStart").unwrap_or(0);
        }
    }
    CACHED
}

/// The guest's `NtCreateThread` into a RUNNING process that is not an x86-64
/// guest -- a native one, or a 32-bit (WoW64) one. Nothing of ours runs over
/// there, and nothing has to: the thread's routine is already the target's
/// code, named by an address the caller got from it. All that is x86-64 about
/// the request is the CONTEXT's shape, so it is translated into the one a
/// native creator would have built, and the TARGET runs it as its own:
///
///   * kernel32's Win32 shape (rip = guest `BaseThreadStart`, rcx/rdx =
///     routine/parameter) becomes the target's native kernel32
///     `BaseThreadStart(routine, parameter)`: the thread exits through
///     ExitThread with the routine's return value, as any CreateThread's does;
///   * the RtlCreateUserThread shape (rip = routine, rcx = parameter) becomes
///     routine(parameter), returning into the native `RtlExitUserThread`
///     (ntdll is at one base in every process).
///
/// A native target then just runs it. A WoW64 target's native loader sees a
/// start that is not native code and hands the thread to its wow64.dll, which
/// runs it as a 32-bit thread (ntdll-rust `LdrpDivertWow64Thread`) -- the same
/// path a native creator's CreateRemoteThread into it takes.
///
/// The guest kernel32 allocated the thread's stack in the target and the
/// INITIAL_TEB describes it; it is the thread's native stack as is.
unsafe fn CreateForeignThread(Args: &mut [u64; crate::syscall::WOWX64_SERVICE_ARGS], Process: HANDLE) -> u64 {
    let Guest = *(Args[5] as *const WowX64Context);
    let mut Native: CONTEXT = core::mem::zeroed();
    Native.ContextFlags = CONTEXT_FULL;
    Native.Sp = Guest.Rsp & !15;

    let ThreadStart = GuestThreadStart();
    let TargetThreadStart = if ThreadStart != 0 && Guest.Rip == ThreadStart {
        RemoteModuleBase(Process, b"kernel32.dll").and_then(|Base| RemoteExport(Process, Base, b"BaseThreadStart"))
    } else {
        None
    };
    match TargetThreadStart {
        Some(Entry) => {
            Native.Pc = Entry;
            Native.__bindgen_anon_1.X[0] = Guest.Rcx;
            Native.__bindgen_anon_1.X[1] = Guest.Rdx;
        }
        None => {
            // The Rtl shape, or the Win32 one into a target without a
            // kernel32 (then routine(parameter) is all that is left of it).
            let Win32 = ThreadStart != 0 && Guest.Rip == ThreadStart;
            Native.Pc = if Win32 { Guest.Rcx } else { Guest.Rip };
            Native.__bindgen_anon_1.X[0] = if Win32 { Guest.Rdx } else { Guest.Rcx };
            Native.__bindgen_anon_1.X[30] = RtlExitUserThread as usize as u64; // lr
        }
    }
    crate::trace!(
        "guest NtCreateThread into a foreign process: pc %p x0 %p x1 %p sp %p",
        Native.Pc as *const c_void,
        Native.__bindgen_anon_1.X[0] as *const c_void,
        Native.__bindgen_anon_1.X[1] as *const c_void,
        Native.Sp as *const c_void,
    );
    Args[5] = &Native as *const CONTEXT as u64;
    crate::host::NativeSyscall(crate::syscall::svc::SVC_NtCreateThread, Args.as_ptr())
}

unsafe fn Free(Process: HANDLE, Base: PVOID) {
    let (mut B, mut S) = (Base, 0 as SIZE_T);
    NtFreeVirtualMemory(Process, &mut B, &mut S, MEM_RELEASE);
}

/// The guest's `NtCreateThread(Handle, Access, ObjectAttributes, Process,
/// ClientId, Context, InitialTeb, CreateSuspended)`.
///
/// By target:
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
///   * a NEW 32-bit child: the same, for the same reason (its wow64.dll
///     starts it from its image).
///   * a RUNNING native or 32-bit process: [`CreateForeignThread`].
///   * a NEW native arm64 child: unreachable — the x86-64 guest's redirected
///     system32 cannot name an arm64 exe to spawn (image.rs). Refused.
pub unsafe fn CreateThread(Args: &mut [u64; crate::syscall::WOWX64_SERVICE_ARGS]) -> u64 {
    let GuestContext = Args[5] as *const WowX64Context;
    let Process = Args[3] as HANDLE;
    if GuestContext.is_null() {
        return crate::host::NativeSyscall(crate::syscall::svc::SVC_NtCreateThread, Args.as_ptr());
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
                return crate::host::NativeSyscall(crate::syscall::svc::SVC_NtCreateThread, Args.as_ptr());
            }
            Some((IMAGE_FILE_MACHINE_I386, false)) => {
                // A NEW 32-bit (WoW64) child: like a new x86-64 one, its
                // initial context is never used — its native loader diverts to
                // wow64.dll, which takes the entry from the image itself. A
                // 64-bit parent's process parameters are exactly what that
                // path reads (winlogon starts every 32-bit ladder guest so).
                Native.Pc = RtlExitUserThread as usize as u64;
                Native.__bindgen_anon_1.X[0] = 0xC000_007B; // STATUS_INVALID_IMAGE_FORMAT
                Native.Sp = ((*GuestContext).Rsp & !15).wrapping_sub(0x100);
                Args[5] = &Native as *const CONTEXT as u64;
                return crate::host::NativeSyscall(crate::syscall::svc::SVC_NtCreateThread, Args.as_ptr());
            }
            // A RUNNING native or 32-bit process (CreateRemoteThread).
            Some((_, true)) => return CreateForeignThread(Args, Process),
            Some((Machine, false)) => {
                // A NEW native arm64 child, whose initial context its loader
                // DOES run. An x86-64 guest cannot reach this: its system32 is
                // redirected to sysamd64, where an arm64 image reads back as
                // not-found (image.rs / syscall.rs), so it cannot name an arm64
                // exe to spawn -- WoW64's own-machine view.
                crate::trace!("guest NtCreateThread: initial thread of a new machine-%04x process", Machine as u32);
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

    let Top = Stack as u64 + StackSize as u64;
    let Rec = (Top - core::mem::size_of::<ThreadRecord>() as u64) & !15;
    Native.Pc = WowX64GuestThreadStart as usize as u64;
    Native.Sp = Rec;
    Native.__bindgen_anon_1.X[0] = Page as u64;
    Native.__bindgen_anon_1.X[1] = Rec;
    Args[5] = &Native as *const CONTEXT as u64;
    let Status = crate::host::NativeSyscall(crate::syscall::svc::SVC_NtCreateThread, Args.as_ptr());
    if (Status as NTSTATUS) < 0 {
        Free(Process, Page);
        Free(Process, Stack);
    }
    Status
}

/// A guest-created thread's first native frame, after the native loader's
/// thread init. Never returns: the guest exits the thread with
/// NtTerminateThread.
unsafe extern "C" fn WowX64GuestThreadStart(Start: *mut ThreadStart, Rec: *mut ThreadRecord) -> ! {
    let Context = (*Start).Context;
    let (mut B, mut S) = (Start as PVOID, 0 as SIZE_T);
    NtFreeVirtualMemory(CURRENT_PROCESS, &mut B, &mut S, MEM_RELEASE);

    // The record: just above the first native frame, where CreateThread left room.
    let mut Info: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
    if NtQueryVirtualMemory(
        CURRENT_PROCESS,
        Rec as PVOID,
        MemoryBasicInformation,
        &mut Info as *mut _ as PVOID,
        core::mem::size_of::<MEMORY_BASIC_INFORMATION>() as SIZE_T,
        null_mut(),
    ) < 0
    {
        die(b"guest thread: its native stack is not mapped\0");
    }
    Rec.write(ThreadRecord { Callouts: null_mut(), Apcs: null_mut(), NativeStack: Info.AllocationBase as u64 });
    SetRecord(Rec);

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

const THREAD_AM_I_LAST_THREAD: THREADINFOCLASS = 12;
const SYNCHRONIZE: ACCESS_MASK = 0x0010_0000;
const DUPLICATE_SAME_ATTRIBUTES: ULONG = 0x4;

/// Which thread `Handle` names: (process id, thread id, TEB).
unsafe fn ThreadIdentity(Handle: HANDLE) -> Option<(u64, u64, *mut TEB)> {
    let mut Info: THREAD_BASIC_INFORMATION = core::mem::zeroed();
    if NtQueryInformationThread(
        Handle,
        ThreadBasicInformation,
        &mut Info as *mut _ as PVOID,
        core::mem::size_of::<THREAD_BASIC_INFORMATION>() as ULONG,
        null_mut(),
    ) < 0
    {
        return None;
    }
    Some((Info.ClientId.UniqueProcess as u64, Info.ClientId.UniqueThread as u64, Info.TebBaseAddress as *mut TEB))
}

/// The guest's `NtTerminateThread(Thread, ExitStatus)`.
pub unsafe fn TerminateThread(Args: &mut [u64; crate::syscall::WOWX64_SERVICE_ARGS]) -> u64 {
    let Handle = Args[0] as HANDLE;
    let Me = &(*NtCurrentTeb()).ClientId;
    let Target = if Handle.is_null() || Handle == crate::host::CURRENT_THREAD {
        Some((Me.UniqueProcess as u64, Me.UniqueThread as u64, NtCurrentTeb()))
    } else {
        ThreadIdentity(Handle)
    };
    let Forward = |Args: &[u64; crate::syscall::WOWX64_SERVICE_ARGS]| {
        crate::host::NativeSyscall(crate::syscall::svc::SVC_NtTerminateThread, Args.as_ptr())
    };
    let (Pid, Tid, Teb) = match Target {
        Some(t) => t,
        None => return Forward(Args), // a bad handle: the kernel says so
    };
    if Pid != Me.UniqueProcess as u64 {
        // A thread of another process. Its wowx64 cannot see this, so if that
        // process is an x86-64 guest its CPU-layer state for the thread and
        // native stack stay until that process exits.
        return Forward(Args);
    }

    if Tid == Me.UniqueThread as u64 {
        let mut Last: ULONG = 0;
        NtQueryInformationThread(
            crate::host::CURRENT_THREAD,
            THREAD_AM_I_LAST_THREAD,
            &mut Last as *mut _ as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
            null_mut(),
        );
        if Last != 0 {
            // The process ends with it, and takes everything along.
            return Forward(Args);
        }
        let Rec = Record();
        (Cpu().ThreadTerm)(crate::host::CURRENT_THREAD, Args[1] as NTSTATUS);
        if (*Rec).NativeStack != 0 {
            let Teb = NtCurrentTeb();
            if (*Teb).FreeStackOnTermination != 0 {
                Free(CURRENT_PROCESS, (*Teb).DeallocationStack);
            }
            (*Teb).DeallocationStack = (*Rec).NativeStack as PVOID;
            (*Teb).FreeStackOnTermination = 1;
        }
        let Status = Forward(Args);
        crate::trace!("NtTerminateThread(self) returned %08lx after its state was freed", Status as u32);
        die(b"NtTerminateThread(self) returned\0");
    }

    // Another thread of this process. Its record is in its TEB, which dies
    // with it, so read what we need first; free its state only once it is
    // gone (BTCpu64ThreadTerm's contract).
    let Rec = (*Teb).DeallocationBStore as *const ThreadRecord;
    let NativeStack = if Rec.is_null() { 0 } else { (*Rec).NativeStack };
    let Status = Forward(Args);
    if (Status as NTSTATUS) < 0 || Rec.is_null() {
        return Status;
    }
    let mut Waitable: HANDLE = null_mut();
    if NtDuplicateObject(CURRENT_PROCESS, Handle, CURRENT_PROCESS, &mut Waitable, SYNCHRONIZE, 0, DUPLICATE_SAME_ATTRIBUTES) < 0 {
        crate::trace!("terminated guest thread %p: cannot wait for it, its CPU state stays", Tid as *const c_void);
        return Status;
    }
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    Timeout.QuadPart = -5 * 10_000_000; // 5 s, relative
    let Waited = NtWaitForSingleObject(Waitable, 0, &mut Timeout);
    if Waited == 0 {
        // ThreadTerm (its NtQueryInformationThread) needs query rights; the
        // guest's own handle has them and still resolves a dead thread's id,
        // whereas `Waitable` is SYNCHRONIZE-only.
        (Cpu().ThreadTerm)(Handle, Args[1] as NTSTATUS);
        if NativeStack != 0 {
            Free(CURRENT_PROCESS, NativeStack as PVOID);
        }
    } else {
        crate::trace!("terminated guest thread %p did not exit (%08lx): its CPU state stays", Tid as *const c_void, Waited);
    }
    NtClose(Waitable);
    Status
}
