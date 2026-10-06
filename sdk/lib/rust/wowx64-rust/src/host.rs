//! The arm64 host half of wowx64.dll (the only arch it is built for — see
//! `dll/win32/wowx64/CMakeLists.txt`; elsewhere the crate is just its contract
//! types, so the workspace still `cargo check`s on every target).

use core::ffi::c_void;
use core::ptr::{addr_of_mut, null_mut};
use core::sync::atomic::{AtomicU32, Ordering};

use ntdll_sys::nt::*;

use crate::contract::*;

const CURRENT_PROCESS: HANDLE = -1isize as HANDLE;
const CURRENT_THREAD: HANDLE = -2isize as HANDLE;

const MEM_COMMIT: ULONG = 0x1000;
const MEM_RESERVE: ULONG = 0x2000;
const PAGE_READWRITE: ULONG = 0x04;
const EXCEPTION_CONTINUE_SEARCH: LONG = 0;

/// Exit status when the guest cannot even be started. Distinct from any
/// guest-chosen code and from an exception code, so a launch failure is never
/// mistaken for the guest's own verdict.
const WOWX64_LAUNCH_FAILED: NTSTATUS = 0xBAD0_0064u32 as NTSTATUS;

/// Guest syscalls traced to serial before going quiet. Bounded so a real
/// guest (thousands of syscalls) cannot flood the UART — whose cost on real
/// hardware is ~90ms/KB inside the trap path — yet the first ones, which are
/// where a bring-up breaks, are always visible.
const TRACED_SYSCALLS: u32 = 16;

type BTCpu64ProcessInitFn = unsafe extern "system" fn(*const WowX64HostServices) -> NTSTATUS;
type BTCpu64ThreadInitFn = unsafe extern "system" fn() -> NTSTATUS;
type BTCpu64GetContextFn =
    unsafe extern "system" fn(HANDLE, HANDLE, *mut c_void, *mut WowX64Context) -> NTSTATUS;
type BTCpu64SetContextFn =
    unsafe extern "system" fn(HANDLE, HANDLE, *mut c_void, *const WowX64Context) -> NTSTATUS;
type BTCpu64SimulateFn = unsafe extern "system" fn();
type BTCpu64ResetToConsistentStateFn = unsafe extern "system" fn(*mut _EXCEPTION_POINTERS) -> NTSTATUS;

static mut GetContext: Option<BTCpu64GetContextFn> = None;
static mut ResetToConsistentState: Option<BTCpu64ResetToConsistentStateFn> = None;

static SyscallCount: AtomicU32 = AtomicU32::new(0);

static HOST_SERVICES: WowX64HostServices = WowX64HostServices {
    Size: core::mem::size_of::<WowX64HostServices>() as u32,
    Version: WOWX64_CONTRACT_VERSION,
    SystemService: WowX64SystemService,
};

macro_rules! trace {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        DbgPrint(concat!("[WOWX64] ", $fmt, "\n\0").as_ptr() as PCSTR $(, $arg)*)
    };
}

unsafe fn die(Why: &[u8]) -> ! {
    DbgPrint(b"[WOWX64] FAIL: %s\n\0".as_ptr() as PCSTR, Why.as_ptr());
    let Status = NtTerminateProcess(CURRENT_PROCESS, WOWX64_LAUNCH_FAILED);
    // Terminating the current process does not return; if it ever does, say so.
    panic!("wowx64: NtTerminateProcess(self) returned {:#x}", Status);
}

/// Issue native system service `Service` with `Args[0..15)`: x0-x7 from the
/// first eight, the remaining seven on the stack where `KiSvcDispatch` reads
/// them (`TrapFrame->Sp`), service number in x8.
///
/// The slot count is baked into the asm below (8 registers + 7 stack words at
/// offsets 64..120), so pin the contract constant to it.
const _: () = assert!(WOWX64_SERVICE_ARGS == 15);
#[unsafe(naked)]
unsafe extern "C" fn NativeSyscall(Service: u32, Args: *const u64) -> u64 {
    core::arch::naked_asm!(
        "stp x29, x30, [sp, #-16]!",
        "mov x29, sp",
        "sub sp, sp, #64",
        "mov w8, w0",
        "mov x9, x1",
        "ldp x10, x11, [x9, #64]",
        "stp x10, x11, [sp, #0]",
        "ldp x10, x11, [x9, #80]",
        "stp x10, x11, [sp, #16]",
        "ldp x10, x11, [x9, #96]",
        "stp x10, x11, [sp, #32]",
        "ldr x10, [x9, #112]",
        "str x10, [sp, #48]",
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

/// `WOWX64_HOST_SERVICES::SystemService`: one guest `syscall`. The CPU layer
/// has already gathered the arguments in guest order (r10, rdx, r8, r9, then
/// [rsp+0x28...]), so this is the register shuffle and nothing else — no
/// per-service code exists, and none is needed (see the module docs).
unsafe extern "C" fn WowX64SystemService(Service: u32, Args: *const u64) -> u64 {
    let Ret = NativeSyscall(Service, Args);
    let N = SyscallCount.fetch_add(1, Ordering::Relaxed);
    if N < TRACED_SYSCALLS {
        trace!(
            "guest syscall #%u: service %u (%p, %p, %p, %p) -> %p",
            N + 1,
            Service,
            *Args.add(0) as *const c_void,
            *Args.add(1) as *const c_void,
            *Args.add(2) as *const c_void,
            *Args.add(3) as *const c_void,
            Ret as *const c_void,
        );
        if N + 1 == TRACED_SYSCALLS {
            trace!("(syscall trace capped at %u)", TRACED_SYSCALLS);
        }
    }
    Ret
}

/// Every native exception, first. The CPU layer resolves its own (overcommit,
/// SMC, unaligned atomics) and NtContinues without returning. What comes back
/// is either a host fault — passed on untouched — or a GUEST fault, restated in
/// guest terms with `ExceptionAddress` = the guest RIP. M1 has no guest
/// exception dispatch, so a guest fault is reported and the process ended with
/// the exception code: loud, and exactly what an unhandled exception does.
unsafe extern "system" fn WowX64Veh(Info: *mut _EXCEPTION_POINTERS) -> LONG {
    let Reset = match ResetToConsistentState {
        Some(f) => f,
        None => return EXCEPTION_CONTINUE_SEARCH,
    };
    Reset(Info);

    let Rec = (*Info).ExceptionRecord;
    let Get = match GetContext {
        Some(f) => f,
        None => return EXCEPTION_CONTINUE_SEARCH,
    };
    let mut Ctx: WowX64Context = core::mem::zeroed();
    if Get(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), &mut Ctx) < 0
        || (*Rec).ExceptionAddress as u64 != Ctx.Rip
    {
        return EXCEPTION_CONTINUE_SEARCH; // a host fault — not ours
    }

    trace!(
        "guest fault %08lx at rip=%p rsp=%p (info %p %p) — no guest SEH yet (M1), terminating",
        (*Rec).ExceptionCode,
        Ctx.Rip as *const c_void,
        Ctx.Rsp as *const c_void,
        (*Rec).ExceptionInformation[0] as *const c_void,
        (*Rec).ExceptionInformation[1] as *const c_void,
    );
    trace!(
        "  rax=%p rcx=%p rdx=%p rbx=%p rsi=%p rdi=%p rbp=%p",
        Ctx.Rax as *const c_void,
        Ctx.Rcx as *const c_void,
        Ctx.Rdx as *const c_void,
        Ctx.Rbx as *const c_void,
        Ctx.Rsi as *const c_void,
        Ctx.Rdi as *const c_void,
        Ctx.Rbp as *const c_void,
    );
    NtTerminateProcess(CURRENT_PROCESS, (*Rec).ExceptionCode as NTSTATUS);
    EXCEPTION_CONTINUE_SEARCH
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

/// ntdll's divert target for an x86-64 process on arm64 (ldrinit.rs, the
/// `IsX64GuestProcess` handoff). Runs after the native loader finished its own
/// process init; never returns — the guest owns the thread from here.
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

    // The guest's stack, the size its image asked for (clamped). Allocated
    // BEFORE BTCpu64ProcessInit, which is when FEX takes its picture of the
    // address space; regions made later are invisible to it until the memory
    // notifications are wired (owed, see module docs). Committed outright,
    // where Windows reserves and grows a guard page: no growth path needed, and
    // it cannot fault where the real thing would have grown.
    //
    // NOT published as the TEB's stack bounds — those still describe the
    // native stack this code runs on. A guest that probes or unwinds against
    // NtTib.StackBase/StackLimit (any CRT-linked one) needs that fixed first.
    let mut StackSize = (*Nt).OptionalHeader.SizeOfStackReserve as SIZE_T;
    StackSize = StackSize.clamp(0x10000, 0x100_0000);
    StackSize = (StackSize + 0xFFFF) & !0xFFFF;
    let mut Stack: PVOID = null_mut();
    if NtAllocateVirtualMemory(CURRENT_PROCESS, &mut Stack, 0, &mut StackSize, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE) < 0 {
        die(b"guest stack allocation\0");
    }

    // The CPU layer.
    let mut Name: UNICODE_STRING = core::mem::zeroed();
    let Wide: [u16; 17] = [
        b'l' as u16, b'i' as u16, b'b' as u16, b'w' as u16, b'o' as u16, b'w' as u16, b'x' as u16,
        b'6' as u16, b'4' as u16, b'f' as u16, b'e' as u16, b'x' as u16, b'.' as u16, b'd' as u16,
        b'l' as u16, b'l' as u16, 0,
    ];
    RtlInitUnicodeString(&mut Name, Wide.as_ptr());
    let mut Cpu: PVOID = null_mut();
    let Status = LdrLoadDll(null_mut(), null_mut(), &mut Name, &mut Cpu);
    if Status < 0 || Cpu.is_null() {
        DbgPrint(b"[WOWX64] FAIL: libwowx64fex.dll load %08lx\n\0".as_ptr() as PCSTR, Status);
        NtTerminateProcess(CURRENT_PROCESS, WOWX64_LAUNCH_FAILED);
    }
    let ProcessInit: BTCpu64ProcessInitFn = core::mem::transmute(Resolve(Cpu, b"BTCpu64ProcessInit\0"));
    let ThreadInit: BTCpu64ThreadInitFn = core::mem::transmute(Resolve(Cpu, b"BTCpu64ThreadInit\0"));
    let SetContext: BTCpu64SetContextFn = core::mem::transmute(Resolve(Cpu, b"BTCpu64SetContext\0"));
    let Simulate: BTCpu64SimulateFn = core::mem::transmute(Resolve(Cpu, b"BTCpu64Simulate\0"));
    *addr_of_mut!(GetContext) = Some(core::mem::transmute(Resolve(Cpu, b"BTCpu64GetContext\0")));
    *addr_of_mut!(ResetToConsistentState) =
        Some(core::mem::transmute(Resolve(Cpu, b"BTCpu64ResetToConsistentState\0")));
    trace!("CPU layer libwowx64fex.dll @ %p", Cpu);

    // First refusal on every native fault, before FEX can take one: InitCore
    // already relies on on-demand commit of its reserved caches.
    if RtlAddVectoredExceptionHandler(1, Some(WowX64Veh)).is_null() {
        die(b"VEH registration\0");
    }

    let Status = ProcessInit(&HOST_SERVICES);
    if Status < 0 {
        DbgPrint(b"[WOWX64] FAIL: BTCpu64ProcessInit %08lx\n\0".as_ptr() as PCSTR, Status);
        NtTerminateProcess(CURRENT_PROCESS, WOWX64_LAUNCH_FAILED);
    }
    let Status = ThreadInit();
    if Status < 0 {
        DbgPrint(b"[WOWX64] FAIL: BTCpu64ThreadInit %08lx\n\0".as_ptr() as PCSTR, Status);
        NtTerminateProcess(CURRENT_PROCESS, WOWX64_LAUNCH_FAILED);
    }

    // The initial context: the image entry, entered as if CALLed (rsp = 8 mod
    // 16, a zero return address — returning from the entry is a loud fault at
    // 0, not a silent wander), with the PEB in rcx as amd64 hands it to an
    // exe's entry point.
    let Top = Stack as u64 + StackSize as u64 - 0x100;
    let Rsp = Top - 8;
    *(Rsp as *mut u64) = 0;

    let mut Ctx: WowX64Context = core::mem::zeroed();
    Ctx.ContextFlags = WOWX64_CONTEXT_ALL;
    Ctx.Rip = Entry;
    Ctx.Rsp = Rsp;
    Ctx.Rcx = Peb as u64;
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
    let Status = SetContext(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), &Ctx);
    if Status < 0 {
        DbgPrint(b"[WOWX64] FAIL: BTCpu64SetContext %08lx\n\0".as_ptr() as PCSTR, Status);
        NtTerminateProcess(CURRENT_PROCESS, WOWX64_LAUNCH_FAILED);
    }

    trace!(
        "entering BTCpu64Simulate (rip=%p rsp=%p, stack %p+%p)",
        Entry as *const c_void,
        Rsp as *const c_void,
        Stack,
        StackSize as *const c_void,
    );
    Simulate();
    die(b"BTCpu64Simulate returned\0");
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(_Instance: PVOID, _Reason: ULONG, _Reserved: PVOID) -> BOOL {
    1
}
