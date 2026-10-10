//! The arm64 host half of wowx64.dll (the only arch it is built for — see
//! `dll/win32/wowx64/CMakeLists.txt`; elsewhere the crate is just its contract
//! types, so the workspace still `cargo check`s on every target).

use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntdll_sys::nt::*;

use crate::contract::*;

pub const CURRENT_PROCESS: HANDLE = -1isize as HANDLE;
pub const CURRENT_THREAD: HANDLE = -2isize as HANDLE;

const MEM_COMMIT: ULONG = 0x1000;
const MEM_RESERVE: ULONG = 0x2000;
const PAGE_READWRITE: ULONG = 0x04;

/// Exit status when the guest cannot even be started. Distinct from any
/// guest-chosen code and from an exception code, so a launch failure is never
/// mistaken for the guest's own verdict.
const WOWX64_LAUNCH_FAILED: NTSTATUS = 0xBAD0_0064u32 as NTSTATUS;

/// The CPU layer's verbs, resolved once at process init.
pub struct CpuLayer {
    pub GetContext: unsafe extern "system" fn(HANDLE, HANDLE, *mut c_void, *mut WowX64Context) -> NTSTATUS,
    pub SetContext: unsafe extern "system" fn(HANDLE, HANDLE, *mut c_void, *const WowX64Context) -> NTSTATUS,
    pub ResetToConsistentState: unsafe extern "system" fn(*mut _EXCEPTION_POINTERS) -> NTSTATUS,
    pub NotifyMemoryAlloc: unsafe extern "system" fn(PVOID, SIZE_T, ULONG, ULONG, BOOL, ULONG),
    pub NotifyMemoryProtect: unsafe extern "system" fn(PVOID, SIZE_T, ULONG, BOOL, ULONG),
    pub NotifyMemoryFree: unsafe extern "system" fn(PVOID, SIZE_T, ULONG, BOOL, ULONG),
    pub NotifyMapViewOfSection: unsafe extern "system" fn(PVOID, PVOID, PVOID, SIZE_T, ULONG, ULONG) -> NTSTATUS,
    pub NotifyUnmapViewOfSection: unsafe extern "system" fn(PVOID, BOOL, ULONG),
    pub FlushInstructionCache2: unsafe extern "system" fn(*const c_void, SIZE_T),
    pub NotifyMemoryDirty: unsafe extern "system" fn(PVOID, SIZE_T),
    pub NotifyReadFile: unsafe extern "system" fn(HANDLE, PVOID, SIZE_T, BOOL, NTSTATUS),
    pub ThreadInit: unsafe extern "system" fn() -> NTSTATUS,
    pub ThreadTerm: unsafe extern "system" fn(HANDLE, NTSTATUS),
    pub ProcessTerm: unsafe extern "system" fn(HANDLE, BOOL, NTSTATUS),
    pub Simulate: unsafe extern "system" fn(),
}

/// The guest ntdll's entry points every guest thread starts through.
struct GuestEntry {
    ApcDispatcher: u64,
    LdrInitializeThunk: u64,
}
static mut GUEST: GuestEntry = GuestEntry { ApcDispatcher: 0, LdrInitializeThunk: 0 };

static mut CPU: Option<CpuLayer> = None;

/// Set once the guest can take a thread: the CPU layer loaded and initialized,
/// the guest ntdll's entry points known. A thread another process creates in
/// this one can arrive as soon as the native loader has published its process
/// init (`LdrpEnterGuestLayer`), which is BEFORE `WowX64LdrpInitialize` has run.
static GUEST_READY: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

pub fn GuestReady() -> bool {
    GUEST_READY.load(core::sync::atomic::Ordering::Acquire)
}

/// The CPU layer. Only reachable once it is loaded: every caller is a guest
/// syscall or a fault taken after `WowX64LdrpInitialize` resolved it.
pub unsafe fn CpuIfLoaded() -> Option<&'static CpuLayer> {
    (*addr_of!(CPU)).as_ref()
}

pub unsafe fn Cpu() -> &'static CpuLayer {
    match &*addr_of!(CPU) {
        Some(c) => c,
        None => die(b"CPU layer used before it was loaded\0"),
    }
}

type BTCpu64ProcessInitFn = unsafe extern "system" fn(*const WowX64HostServices) -> NTSTATUS;

static HOST_SERVICES: WowX64HostServices = WowX64HostServices {
    Size: core::mem::size_of::<WowX64HostServices>() as u32,
    Version: WOWX64_CONTRACT_VERSION,
    SystemService: crate::syscall::WowX64SystemService,
};

pub unsafe fn die(Why: &[u8]) -> ! {
    trace!("FAIL: %s", Why.as_ptr());
    let Status = NtTerminateProcess(CURRENT_PROCESS, WOWX64_LAUNCH_FAILED);
    // Terminating the current process does not return; if it ever does, say so.
    panic!("wowx64: NtTerminateProcess(self) returned {:#x}", Status);
}

/// Issue native system service `Service` with `Args[0..17)`: x0-x7 from the
/// first eight, the remaining nine on the stack where `KiSvcDispatch` reads
/// them (`TrapFrame->Sp`), service number in x8.
///
/// The slot count is baked into the asm below (8 registers + 9 stack words at
/// offsets 64..136, in an 80-byte area to keep sp 16-aligned), so pin the
/// constant to it.
const _: () = assert!(crate::syscall::WOWX64_SERVICE_ARGS == 17);
#[unsafe(naked)]
pub unsafe extern "C" fn NativeSyscall(Service: u32, Args: *const u64) -> u64 {
    core::arch::naked_asm!(
        "stp x29, x30, [sp, #-16]!",
        "mov x29, sp",
        "sub sp, sp, #80",
        "mov w8, w0",
        "mov x9, x1",
        "ldp x10, x11, [x9, #64]",
        "stp x10, x11, [sp, #0]",
        "ldp x10, x11, [x9, #80]",
        "stp x10, x11, [sp, #16]",
        "ldp x10, x11, [x9, #96]",
        "stp x10, x11, [sp, #32]",
        "ldp x10, x11, [x9, #112]",
        "stp x10, x11, [sp, #48]",
        "ldr x10, [x9, #128]",
        "str x10, [sp, #64]",
        "ldp x0, x1, [x9, #0]",
        "ldp x2, x3, [x9, #16]",
        "ldp x4, x5, [x9, #32]",
        "ldp x6, x7, [x9, #48]",
        "svc #0",
        "mov sp, x29",
        "ldp x29, x30, [sp], #16",
        "ret",
    );
}

/// The address of an export of the NATIVE ntdll (a hook slot). Dies if it is
/// missing: each slot is how guest code avoids being called as arm64 code.
pub unsafe fn NativeNtdllExport(Name: &[u8]) -> PVOID {
    let mut Dll: UNICODE_STRING = core::mem::zeroed();
    let Wide: [u16; 10] = [
        b'n' as u16, b't' as u16, b'd' as u16, b'l' as u16, b'l' as u16, b'.' as u16, b'd' as u16,
        b'l' as u16, b'l' as u16, 0,
    ];
    RtlInitUnicodeString(&mut Dll, Wide.as_ptr());
    let mut Ntdll: PVOID = null_mut();
    let mut Ansi: ANSI_STRING = core::mem::zeroed();
    RtlInitAnsiString(&mut Ansi, Name.as_ptr() as PCSZ);
    let mut Slot: PVOID = null_mut();
    if LdrGetDllHandle(null_mut(), null_mut(), &mut Dll, &mut Ntdll) < 0
        || LdrGetProcedureAddress(Ntdll, &mut Ansi, 0, &mut Slot) < 0
        || Slot.is_null()
    {
        trace!("FAIL: native ntdll.dll has no %s", Name.as_ptr());
        die(b"native ntdll.dll is missing a wowx64 hook slot\0");
    }
    Slot
}

/// The guest ntdll's `KiUserApcDispatcher`.
pub unsafe fn GuestApcDispatcher() -> u64 {
    (*addr_of!(GUEST)).ApcDispatcher
}

unsafe fn Resolve(Module: PVOID, Name: &[u8]) -> PVOID {
    let mut Ansi: ANSI_STRING = core::mem::zeroed();
    RtlInitAnsiString(&mut Ansi, Name.as_ptr() as PCSZ);
    let mut Addr: PVOID = null_mut();
    if LdrGetProcedureAddress(Module, &mut Ansi, 0, &mut Addr) < 0 || Addr.is_null() {
        DbgPrint(b"[WOWX64] FAIL: libwowx64fex.dll has no %s\n\0".as_ptr() as PCSTR, Name.as_ptr());
        NtTerminateProcess(CURRENT_PROCESS, WOWX64_LAUNCH_FAILED);
    }
    Addr
}

/// An x86-64 user context with the flat long-mode selectors and the reset
/// FPU/SSE state, ready for `Rip`/`Rsp`/argument registers to be filled in.
pub fn InitialContext() -> WowX64Context {
    let mut Ctx: WowX64Context = unsafe { core::mem::zeroed() };
    Ctx.ContextFlags = WOWX64_CONTEXT_ALL;
    Ctx.EFlags = 0x202; // IF | the always-one bit
    Ctx.SegCs = WOWX64_SEG_CS;
    Ctx.SegDs = WOWX64_SEG_DS;
    Ctx.SegEs = WOWX64_SEG_DS;
    Ctx.SegSs = WOWX64_SEG_DS;
    Ctx.SegGs = WOWX64_SEG_DS;
    Ctx.SegFs = WOWX64_SEG_FS;
    Ctx.MxCsr = 0x1F80;
    Ctx.FltSave.ControlWord = 0x27F;
    Ctx.FltSave.MxCsr = 0x1F80;
    Ctx
}

/// This thread's CPU-layer state. Every guest thread, before its first guest
/// instruction.
pub unsafe fn ThreadInit() {
    let Status = (Cpu().ThreadInit)();
    if Status < 0 {
        DbgPrint(b"[WOWX64] FAIL: BTCpu64ThreadInit %08lx\n\0".as_ptr() as PCSTR, Status);
        NtTerminateProcess(CURRENT_PROCESS, WOWX64_LAUNCH_FAILED);
    }
}

/// Start the current thread in the guest the way an amd64 thread starts on
/// amd64: in the guest ntdll's `KiUserApcDispatcher`, on an APC frame naming
/// `LdrInitializeThunk`. That runs the guest loader's process or thread init
/// and ends in `NtContinue` of the frame's CONTEXT — `Run` — which
/// `syscall.rs` hands to the CPU layer. Never returns.
pub unsafe fn EnterGuest(Run: &WowX64Context, SystemArgument1: u64) -> ! {
    let Guest = &*addr_of!(GUEST);
    // The frame: an x86-64 CONTEXT whose P1-P4 homes are KiUserApcDispatcher's
    // arguments, just below where `Run` will resume. The loader's own frames
    // go below it; all of it is dead once the loader NtContinues to `Run`.
    let Frame = (Run.Rsp - core::mem::size_of::<WowX64Context>() as u64 - 0x40) & !0xF;
    let mut F = *Run;
    F.ContextFlags = WOWX64_CONTEXT_ALL;
    F.P1Home = 0; // NormalContext
    F.P2Home = SystemArgument1;
    F.P3Home = 0; // SystemArgument2
    F.P4Home = Guest.LdrInitializeThunk; // NormalRoutine
    core::ptr::write(Frame as *mut WowX64Context, F);

    let mut Start = InitialContext();
    Start.Rip = Guest.ApcDispatcher;
    Start.Rsp = Frame;
    let Status = (Cpu().SetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), &Start);
    if Status < 0 {
        DbgPrint(b"[WOWX64] FAIL: BTCpu64SetContext %08lx\n\0".as_ptr() as PCSTR, Status);
        NtTerminateProcess(CURRENT_PROCESS, WOWX64_LAUNCH_FAILED);
    }
    (Cpu().Simulate)();
    die(b"BTCpu64Simulate returned\0");
}

/// ntdll's divert target for an x86-64 process on arm64 (ldrinit.rs, the
/// `IsX64GuestProcess` handoff). Runs after the native loader finished its own
/// process init; never returns — the guest owns the thread from here. The
/// guest's own amd64 loader then does ITS process init: the image's import
/// closure out of `sysamd64`, DLL init routines, and the jump to the entry.
#[no_mangle]
pub unsafe extern "system" fn WowX64LdrpInitialize(_Context: PCONTEXT) {
    let Peb = NtCurrentPeb();
    let ImageBase = (*Peb).ImageBaseAddress;
    let Nt = RtlImageNtHeader(ImageBase);
    if Nt.is_null() {
        die(b"guest image has no NT headers\0");
    }
    let Entry = ImageBase as u64 + (*Nt).OptionalHeader.AddressOfEntryPoint as u64;
    trace!("x86-64 guest process: image %p entry %p", ImageBase, Entry as *const c_void);

    // Everything the guest owns is set up BEFORE BTCpu64ProcessInit, which is
    // when FEX takes its picture of the address space; later changes reach it
    // only through the notifications `syscall.rs` sends.
    //
    // The stack: the size the image asked for (clamped), committed outright
    // where Windows reserves and grows a guard page — no growth path needed,
    // and it cannot fault where the real thing would have grown.
    let mut StackSize = (*Nt).OptionalHeader.SizeOfStackReserve as SIZE_T;
    StackSize = StackSize.clamp(0x10000, 0x100_0000);
    StackSize = (StackSize + 0xFFFF) & !0xFFFF;
    let mut Stack: PVOID = null_mut();
    if NtAllocateVirtualMemory(CURRENT_PROCESS, &mut Stack, 0, &mut StackSize, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE) < 0 {
        die(b"guest stack allocation\0");
    }
    let Ntdll = crate::image::MapGuestNtdll();

    // The CPU layer.
    let mut Name: UNICODE_STRING = core::mem::zeroed();
    let Wide: [u16; 17] = [
        b'l' as u16, b'i' as u16, b'b' as u16, b'w' as u16, b'o' as u16, b'w' as u16, b'x' as u16,
        b'6' as u16, b'4' as u16, b'f' as u16, b'e' as u16, b'x' as u16, b'.' as u16, b'd' as u16,
        b'l' as u16, b'l' as u16, 0,
    ];
    RtlInitUnicodeString(&mut Name, Wide.as_ptr());
    let mut Fex: PVOID = null_mut();
    let Status = LdrLoadDll(null_mut(), null_mut(), &mut Name, &mut Fex);
    if Status < 0 || Fex.is_null() {
        DbgPrint(b"[WOWX64] FAIL: libwowx64fex.dll load %08lx\n\0".as_ptr() as PCSTR, Status);
        NtTerminateProcess(CURRENT_PROCESS, WOWX64_LAUNCH_FAILED);
    }
    let ProcessInit: BTCpu64ProcessInitFn = core::mem::transmute(Resolve(Fex, b"BTCpu64ProcessInit\0"));
    *addr_of_mut!(CPU) = Some(CpuLayer {
        GetContext: core::mem::transmute(Resolve(Fex, b"BTCpu64GetContext\0")),
        SetContext: core::mem::transmute(Resolve(Fex, b"BTCpu64SetContext\0")),
        ResetToConsistentState: core::mem::transmute(Resolve(Fex, b"BTCpu64ResetToConsistentState\0")),
        NotifyMemoryAlloc: core::mem::transmute(Resolve(Fex, b"BTCpu64NotifyMemoryAlloc\0")),
        NotifyMemoryProtect: core::mem::transmute(Resolve(Fex, b"BTCpu64NotifyMemoryProtect\0")),
        NotifyMemoryFree: core::mem::transmute(Resolve(Fex, b"BTCpu64NotifyMemoryFree\0")),
        NotifyMapViewOfSection: core::mem::transmute(Resolve(Fex, b"BTCpu64NotifyMapViewOfSection\0")),
        NotifyUnmapViewOfSection: core::mem::transmute(Resolve(Fex, b"BTCpu64NotifyUnmapViewOfSection\0")),
        FlushInstructionCache2: core::mem::transmute(Resolve(Fex, b"BTCpu64FlushInstructionCache2\0")),
        NotifyMemoryDirty: core::mem::transmute(Resolve(Fex, b"BTCpu64NotifyMemoryDirty\0")),
        NotifyReadFile: core::mem::transmute(Resolve(Fex, b"BTCpu64NotifyReadFile\0")),
        ThreadInit: core::mem::transmute(Resolve(Fex, b"BTCpu64ThreadInit\0")),
        ThreadTerm: core::mem::transmute(Resolve(Fex, b"BTCpu64ThreadTerm\0")),
        ProcessTerm: core::mem::transmute(Resolve(Fex, b"BTCpu64ProcessTerm\0")),
        Simulate: core::mem::transmute(Resolve(Fex, b"BTCpu64Simulate\0")),
    });
    let Cpu = Cpu();
    trace!("CPU layer libwowx64fex.dll @ %p", Fex);

    // First refusal on every native fault, before FEX can take one: InitCore
    // already relies on on-demand commit of its reserved caches.
    if RtlAddVectoredExceptionHandler(1, Some(crate::except::WowX64Veh)).is_null() {
        die(b"VEH registration\0");
    }

    let Status = ProcessInit(&HOST_SERVICES);
    if Status < 0 {
        DbgPrint(b"[WOWX64] FAIL: BTCpu64ProcessInit %08lx\n\0".as_ptr() as PCSTR, Status);
        NtTerminateProcess(CURRENT_PROCESS, WOWX64_LAUNCH_FAILED);
    }
    ThreadInit();
    crate::thread::InitialThreadRecord();
    // ProcessInit registered the main image itself; the guest ntdll was mapped
    // by us, not through a guest syscall, so nothing has told FEX it is code.
    (Cpu.NotifyMapViewOfSection)(null_mut(), Ntdll.Base as PVOID, null_mut(), Ntdll.Size as SIZE_T, 0, 0x20);

    let Guest = &mut *addr_of_mut!(GUEST);
    Guest.ApcDispatcher = crate::image::FindExport(&Ntdll, b"KiUserApcDispatcher\0")
        .unwrap_or_else(|| die(b"guest ntdll.dll has no KiUserApcDispatcher\0"));
    Guest.LdrInitializeThunk = crate::image::FindExport(&Ntdll, b"LdrInitializeThunk\0")
        .unwrap_or_else(|| die(b"guest ntdll.dll has no LdrInitializeThunk\0"));
    crate::except::SetGuestDispatcher(
        crate::image::FindExport(&Ntdll, b"KiUserExceptionDispatcher\0")
            .unwrap_or_else(|| die(b"guest ntdll.dll has no KiUserExceptionDispatcher\0")),
    );
    crate::callback::Install(
        crate::image::FindExport(&Ntdll, b"KiUserCallbackDispatcher\0")
            .unwrap_or_else(|| die(b"guest ntdll.dll has no KiUserCallbackDispatcher\0")),
    );
    crate::apc::Install();

    // The TEB now describes the guest stack (thread.rs says why), so the
    // native one this code runs on must never need guard-page growth again.
    let Teb = NtCurrentTeb();
    crate::thread::CommitWholeStack((*Teb).DeallocationStack as u64, (*Teb).NtTib.StackBase as u64);
    let Top = Stack as u64 + StackSize as u64;
    crate::thread::PublishGuestStack(Stack as u64, Top);

    // What the guest loader resumes when it is done: the image entry, entered
    // as if CALLed (rsp = 8 mod 16) with a zero return address — returning from
    // the entry is a loud fault at 0, not a silent wander — and the PEB in rcx
    // as amd64 hands it to an exe.
    let RunRsp = ((Top - 0x100) & !0xF) - 8;
    *(RunRsp as *mut u64) = 0;
    let mut Run = InitialContext();
    Run.Rip = Entry;
    Run.Rsp = RunRsp;
    Run.Rcx = Peb as u64;

    trace!(
        "entering the guest loader: KiUserApcDispatcher %p -> LdrInitializeThunk %p, then entry %p (stack %p+%p)",
        Guest.ApcDispatcher as *const c_void,
        Guest.LdrInitializeThunk as *const c_void,
        Entry as *const c_void,
        Stack,
        StackSize as *const c_void,
    );
    GUEST_READY.store(true, core::sync::atomic::Ordering::Release);
    // SystemArgument1 = the guest ntdll's base: its LdrpInitializeProcess
    // builds ntdll's own loader entry from it, as on amd64.
    EnterGuest(&Run, Ntdll.Base as u64);
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(_Instance: PVOID, _Reason: ULONG, _Reserved: PVOID) -> BOOL {
    1
}
