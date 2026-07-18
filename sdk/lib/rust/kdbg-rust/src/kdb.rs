use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::MAXULONG;
use ntkrnl_sys::ex::ExAllocatePoolWithTag;
use ntkrnl_sys::interlocked::{InterlockedDecrement, InterlockedIncrement};
use ntkrnl_sys::mm::{NonPagedPool, PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_NOACCESS, PAGE_READONLY};
use ntkrnl_sys::ddk::{
    _EPROCESS, _ETHREAD, _KAPC_STATE, KeBugCheck, KeIsExecutingDpc,
    KeStackAttachProcess, KeUnstackDetachProcess, KPROCESSOR_MODE, MmGetPageProtect, MmSetPageProtect,
    PCONTEXT, PCSTR, PEPROCESS, PETHREAD, PEXCEPTION_RECORD64, PsGetCurrentThread,
    PsLookupProcessByProcessId, PsLookupThreadByThreadId,
};

extern "system" {
    /// Defined in shim_exports.rs as `extern "system"` — see comment there:
    /// "PsGetCurrentProcess (NTAPI)". The kernel function follows NTAPI/stdcall convention.
    fn PsGetCurrentProcess() -> PEPROCESS;
}

use super::types::*;

// Constants from ntoskrnl/kdbg/kdb.c & ndk
const KDB_STACK_SIZE: usize = 4096 * 3;
#[cfg(target_arch = "x86_64")]
const KDB_STACK_RESERVE: usize = 5 * core::mem::size_of::<PVOID>();
#[cfg(not(target_arch = "x86_64"))]
const KDB_STACK_RESERVE: usize = core::mem::size_of::<ULONG>();

const KDB_MAXIMUM_BREAKPOINT_COUNT: usize = 256;
const KDB_MAXIMUM_HW_BREAKPOINT_COUNT: usize = 4;
const KDB_MAXIMUM_SW_BREAKPOINT_COUNT: usize = 256;

// i386 GDT selectors (from ndk/i386/asm.h)
const KGDT_R0_CODE: ULONG = 0x08;
const KGDT_R0_DATA: ULONG = 0x10;

// Trap-flag / resume-flag bits in EFlags
const EFLAGS_TF: ULONG = 0x100;
const EFLAGS_RF: ULONG = 0x10000;

const MANUALLY_INITIATED_CRASH: ULONG = 0xE2;

// PAGE_ROUND_DOWN(x) — from ndk/mmtypes.h
macro_rules! PAGE_ROUND_DOWN {
    ($x:expr) => {
        (($x as ULONG_PTR) & !(PAGE_SIZE as ULONG_PTR - 1))
    };
}

// RTL_NUMBER_OF(arr) — number of elements in a fixed-size array.
macro_rules! RTL_NUMBER_OF {
    ($arr:expr) => {
        ($arr.len() as ULONG)
    };
}

// KDB_BREAKPOINT_SIZE — disambiguate (defined in both base & constants).
const KDB_BREAKPOINT_SIZE: ULONG_PTR = ntkrnl_sys::constants::KD_BREAKPOINT_SIZE as ULONG_PTR;

// KDPDEBUGMODE bitfield mirror — see ntoskrnl/kd/kd.h _KDP_DEBUG_MODE.
// We mirror the layout as a single ULONG (the C union has Screen/Serial/File
// at bits 0/1/2 and a Value alias). For our use, Screen lives at bit 0, so
// we test against bit 0 of the entire word — accessor via field name.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct KDPDEBUGMODE {
    pub Screen: ULONG,
}

// Common typedefs not in base.rs.
#[allow(non_camel_case_types)]
pub type INT = i32;
#[allow(non_camel_case_types)]
pub type ULONG64 = u64;

// KDESCRIPTOR (i386 IDTR layout) — see ndk/i386/ketypes.h
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct KDESCRIPTOR {
    pub Pad: USHORT,
    pub Limit: USHORT,
    pub Base: ULONG,
}

// Status codes used here that are not reexported via constants.
const STATUS_MEMORY_NOT_ALLOCATED: NTSTATUS = 0xC00000A0u32 as NTSTATUS;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001u32 as NTSTATUS;
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_BREAKPOINT: NTSTATUS = 0x80000003u32 as NTSTATUS;
const STATUS_SINGLE_STEP: NTSTATUS = 0x80000004u32 as NTSTATUS;
const STATUS_INTEGER_DIVIDE_BY_ZERO: NTSTATUS = 0xC0000094u32 as NTSTATUS;
const STATUS_INTEGER_OVERFLOW: NTSTATUS = 0xC0000095u32 as NTSTATUS;
const STATUS_ARRAY_BOUNDS_EXCEEDED: NTSTATUS = 0xC000008Cu32 as NTSTATUS;
const STATUS_ILLEGAL_INSTRUCTION: NTSTATUS = 0xC000001Du32 as NTSTATUS;
const STATUS_FLOAT_INVALID_OPERATION: NTSTATUS = 0xC0000090u32 as NTSTATUS;
const STATUS_STACK_OVERFLOW: NTSTATUS = 0xC00000FDu32 as NTSTATUS;
const STATUS_ACCESS_VIOLATION: NTSTATUS = 0xC0000005u32 as NTSTATUS;
const STATUS_DATATYPE_MISALIGNMENT: NTSTATUS = 0x80000002u32 as NTSTATUS;
const STATUS_FLOAT_MULTIPLE_TRAPS: NTSTATUS = 0xC00002B5u32 as NTSTATUS;
const STATUS_ASSERTION_FAILURE: NTSTATUS = 0xC0000420u32 as NTSTATUS;

const TAG_KDBG: ULONG = u32::from_be_bytes(*b"GBDK");

const MMDBG_COPY_UNSAFE: ULONG = 0x04;
const MMDBG_COPY_WRITE: ULONG = 0x01;

#[inline]
fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }

// =============================================================================
// Static globals (translated from kdb.c)
// =============================================================================

static mut KdbEntryCount: LONG = 0;

#[repr(align(16))]
struct AlignedStack([u8; KDB_STACK_SIZE]);
impl AlignedStack {
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0.as_mut_ptr()
    }
}
static mut KdbStack: AlignedStack = AlignedStack([0; KDB_STACK_SIZE]);

static mut KdbBreakPointCount: ULONG = 0;
static mut KdbBreakPoints: [KDB_BREAKPOINT; KDB_MAXIMUM_BREAKPOINT_COUNT] = [KDB_BREAKPOINT {
    Type: KdbBreakPointNone,
    Enabled: 0u8,
    Address: 0,
    Global: 0u8,
    Process: core::ptr::null_mut(),
    ConditionExpression: core::ptr::null_mut(),
    Condition: core::ptr::null_mut(),
    Data: KDB_BREAKPOINT_DATA { SavedInstruction: 0 },
}; KDB_MAXIMUM_BREAKPOINT_COUNT];
static mut KdbSwBreakPointCount: ULONG = 0;
static mut KdbHwBreakPointCount: ULONG = 0;
static mut KdbSwBreakPoints: [PKDB_BREAKPOINT; KDB_MAXIMUM_SW_BREAKPOINT_COUNT] =
    [core::ptr::null_mut(); KDB_MAXIMUM_SW_BREAKPOINT_COUNT];
static mut KdbHwBreakPoints: [PKDB_BREAKPOINT; KDB_MAXIMUM_HW_BREAKPOINT_COUNT] =
    [core::ptr::null_mut(); KDB_MAXIMUM_HW_BREAKPOINT_COUNT];
static mut KdbBreakPointToReenable: PKDB_BREAKPOINT = core::ptr::null_mut();
static mut KdbpEvenThoughWeHaveABreakPointToReenableWeAlsoHaveARealSingleStep: BOOLEAN = 0u8;

#[no_mangle]
pub static mut KdbLastBreakPointNr: LONG = -1;
#[no_mangle]
pub static mut KdbNumSingleSteps: ULONG = 0;
#[no_mangle]
pub static mut KdbSingleStepOver: BOOLEAN = 0u8;
static mut KdbEnteredOnSingleStep: BOOLEAN = 0u8;

#[no_mangle]
pub static mut KdbCurrentProcess: PEPROCESS = core::ptr::null_mut();
#[no_mangle]
pub static mut KdbOriginalProcess: PEPROCESS = core::ptr::null_mut();
#[no_mangle]
pub static mut KdbCurrentThread: PETHREAD = core::ptr::null_mut();
#[no_mangle]
pub static mut KdbOriginalThread: PETHREAD = core::ptr::null_mut();
#[no_mangle]
pub static mut KdbCurrentTrapFrame: PKDB_KTRAP_FRAME = core::ptr::null_mut();

static mut KdbTrapFrame: KDB_KTRAP_FRAME = unsafe { core::mem::zeroed() };
static mut KdbThreadTrapFrame: KDB_KTRAP_FRAME = unsafe { core::mem::zeroed() };
static mut KdbApcState: _KAPC_STATE = unsafe { core::mem::zeroed() };

static mut KdbEnterConditions: [[KDB_ENTER_CONDITION; 2]; 22] = [
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbEnterFromKmode, KdbDoNotEnter],
    [KdbDoNotEnter, KdbEnterAlways],
    [KdbEnterFromKmode, KdbDoNotEnter],
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbEnterAlways, KdbEnterAlways],
    [KdbEnterAlways, KdbEnterAlways],
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbEnterAlways, KdbEnterAlways],
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbDoNotEnter, KdbEnterFromKmode],
    [KdbEnterFromKmode, KdbDoNotEnter],
    [KdbDoNotEnter, KdbEnterFromKmode],
];

static ExceptionNrToString: [&[u8]; 21] = [
    b"Divide Error\0",
    b"Debug Trap\0",
    b"NMI\0",
    b"Breakpoint\0",
    b"Overflow\0",
    b"BOUND range exceeded\0",
    b"Invalid Opcode\0",
    b"No Math Coprocessor\0",
    b"Double Fault\0",
    b"Unknown(9)\0",
    b"Invalid TSS\0",
    b"Segment Not Present\0",
    b"Stack Segment Fault\0",
    b"General Protection\0",
    b"Page Fault\0",
    b"Reserved(15)\0",
    b"Math Fault\0",
    b"Alignment Check\0",
    b"Machine Check\0",
    b"SIMD Fault\0",
    b"Assertion Failure\0",
];

// =============================================================================
// Inline-asm replacements for x86 intrinsics (no exported symbol).
// =============================================================================
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn __readcr2() -> ULONG_PTR {
    let cr2: ULONG_PTR;
    core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack));
    cr2
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn __readeflags() -> ULONG_PTR {
    let flags: ULONG_PTR;
    core::arch::asm!("pushfd", "pop {}", out(reg) flags, options(nomem, nostack));
    flags
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn __writeeflags(value: ULONG_PTR) {
    core::arch::asm!("push {}", "popfd", in(reg) value, options(nomem, nostack));
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn __sidt(dest: PVOID) {
    core::arch::asm!("sidt [{}]", in(reg) dest, options(nostack));
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn _disable() {
    core::arch::asm!("cli", options(nomem, nostack));
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn __readcr2() -> ULONG_PTR {
    let cr2: ULONG_PTR;
    core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack));
    cr2
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn __readeflags() -> ULONG_PTR {
    let flags: ULONG_PTR;
    core::arch::asm!("pushfq", "pop {}", out(reg) flags, options(nomem, nostack));
    flags
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn __writeeflags(value: ULONG_PTR) {
    core::arch::asm!("push {}", "popfq", in(reg) value, options(nomem, nostack));
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn __sidt(dest: PVOID) {
    core::arch::asm!("sidt [{}]", in(reg) dest, options(nostack));
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn _disable() {
    core::arch::asm!("cli", options(nomem, nostack));
}

// =============================================================================
// External declarations — symbols defined in C or in other Rust files
// =============================================================================

extern "C" {
    fn KdbPuts(String: PCSTR);
    fn KdbpGetInstLength(Address: ULONG_PTR) -> LONG;
    // KdbpStackSwitchAndCall is declared in kdb.h as `VOID NTAPI`. Defined in
    // i386/kdb_help.S and amd64/kdb_help.S — both export `_*@8`.
    // Move to extern "system" below.
    fn KdbpRpnParseExpression(
        Expression: *mut u8,
        ErrOffset: *mut LONG,
        ErrMsg: *mut u8,
    ) -> PVOID;
    fn KdbpRpnEvaluateParsedExpression(
        Expression: PVOID,
        TrapFrame: PKDB_KTRAP_FRAME,
        Result: *mut ULONGLONG,
        ErrOffset: *mut LONG,
        ErrMsg: *mut u8,
    ) -> BOOLEAN;
    fn KdbpCliInterpretInitFile();
    fn KbdDisableMouse();
    fn KbdEnableMouse();
    fn KdpScreenAcquire();
    fn KdpScreenRelease();
    fn strchr(s: PCSTR, c: i32) -> PCSTR;
    fn strlen(s: PCSTR) -> usize;
    fn _strnicmp(s1: PCSTR, s2: PCSTR, n: usize) -> i32;
    fn KeGetContextPc(Context: PCONTEXT) -> ULONG_PTR;
    fn KeSetContextPc(Context: PCONTEXT, Pc: ULONG_PTR);
    static mut KdbInitFileBuffer: PCHAR;
    static mut KdbpBugCheckRequested: BOOLEAN;
    static mut KdpDebugMode: KDPDEBUGMODE;
}

extern "C" {
    /// Defined in kdb_cli.rs as `pub unsafe extern "C" fn KdbpCliMainLoop`.
    fn KdbpCliMainLoop(EnteredOnSingleStep: BOOLEAN);
}

extern "system" {
    /// `ObDereferenceObject` is exported by ob/obref.rs as `extern "system"`.
    fn ObDereferenceObject(Object: PVOID);
    /// Defined in i386/kdb_help.S / amd64/kdb_help.S as NTAPI.
    fn KdbpStackSwitchAndCall(NewStack: PVOID, Procedure: Option<unsafe extern "C" fn()>);
    /// `KdpCopyMemoryChunks` is NTAPI per ndk/kdfuncs.h. Defined in kd64/kdapi.rs.
    fn KdpCopyMemoryChunks(
        Address: u64,
        Buffer: PVOID,
        TotalSize: ULONG,
        ChunkSize: ULONG,
        Flags: ULONG,
        ActualSize: *mut ULONG,
    ) -> NTSTATUS;
    /// IRQL primitives — NTAPI on i386 (and bindgen declares stdcall on
    /// non-amd64). On amd64 these are FORCEINLINE in C; the Rust port has
    /// out-of-line wrappers in shim_exports.rs.
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
    fn KeGetCurrentIrql() -> KIRQL;
}

// KdbpDeleteBreakPoint and KdbpEnableBreakPoint are translated below
// (search for `pub unsafe extern "C" fn KdbpDeleteBreakPoint`).

unsafe fn KdbpKdbTrapFrameFromKernelStack(
    KernelStack: PVOID,
    KdbTrapFrame_p: PKDB_KTRAP_FRAME)
{
    let mut StackPtr: *mut ULONG_PTR;

    core::ptr::write_bytes(KdbTrapFrame_p, 0, 1);
    StackPtr = KernelStack as *mut ULONG_PTR;
    #[cfg(target_arch = "x86")]
    {
        (*KdbTrapFrame_p).Ebp = *StackPtr.add(3) as ULONG;
        (*KdbTrapFrame_p).Edi = *StackPtr.add(4) as ULONG;
        (*KdbTrapFrame_p).Esi = *StackPtr.add(5) as ULONG;
        (*KdbTrapFrame_p).Ebx = *StackPtr.add(6) as ULONG;
        (*KdbTrapFrame_p).Eip = *StackPtr.add(7) as ULONG;
        (*KdbTrapFrame_p).Esp = StackPtr.add(8) as ULONG;
        (*KdbTrapFrame_p).SegSs = KGDT_R0_DATA;
        (*KdbTrapFrame_p).SegCs = KGDT_R0_CODE;
        (*KdbTrapFrame_p).SegDs = KGDT_R0_DATA;
        (*KdbTrapFrame_p).SegEs = KGDT_R0_DATA;
        (*KdbTrapFrame_p).SegGs = KGDT_R0_DATA;
    }
}

unsafe fn KdbpOverwriteInstruction(
    Process: PEPROCESS,
    Address: ULONG_PTR,
    NewInst: UCHAR,
    OldInst: PUCHAR,
) -> NTSTATUS
{
    let mut Status: NTSTATUS;
    let mut Protect: ULONG;
    let CurrentProcess = PsGetCurrentProcess();
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();

    Protect = MmGetPageProtect(Process, PAGE_ROUND_DOWN!(Address) as PVOID);

    if (Protect & PAGE_NOACCESS) != 0
    {
        return STATUS_MEMORY_NOT_ALLOCATED;
    }

    if CurrentProcess != Process
    {
        KeStackAttachProcess(core::ptr::addr_of_mut!((*Process).Pcb), &mut ApcState);
    }

    if (Protect & (PAGE_READONLY | PAGE_EXECUTE | PAGE_EXECUTE_READ)) != 0
    {
        MmSetPageProtect(Process, PAGE_ROUND_DOWN!(Address) as PVOID,
                         (Protect & !(PAGE_READONLY | PAGE_EXECUTE | PAGE_EXECUTE_READ)) | PAGE_READWRITE);
    }

    if !OldInst.is_null()
    {
        Status = KdbpSafeReadMemory(OldInst as PVOID, Address as PUCHAR as PVOID, 1);
        if !NT_SUCCESS(Status)
        {
            if (Protect & (PAGE_READONLY | PAGE_EXECUTE | PAGE_EXECUTE_READ)) != 0
            {
                MmSetPageProtect(Process, PAGE_ROUND_DOWN!(Address) as PVOID, Protect);
            }

            if CurrentProcess != Process
            {
                KeUnstackDetachProcess(&mut ApcState);
            }

            return Status;
        }
    }

    Status = KdbpSafeWriteMemory(Address as PUCHAR as PVOID, core::ptr::addr_of!(NewInst) as PVOID, 1);

    if (Protect & (PAGE_READONLY | PAGE_EXECUTE | PAGE_EXECUTE_READ)) != 0
    {
        MmSetPageProtect(Process, PAGE_ROUND_DOWN!(Address) as PVOID, Protect);
    }

    if CurrentProcess != Process
    {
        KeUnstackDetachProcess(&mut ApcState);
    }

    Status
}

unsafe fn KdbpShouldStepOverInstruction(
    Eip: ULONG_PTR,
) -> BOOLEAN
{
    let mut Mem: [UCHAR; 3] = [0; 3];
    let mut i: ULONG = 0;

    if !NT_SUCCESS(KdbpSafeReadMemory(
        Mem.as_mut_ptr() as PVOID,
        Eip as PVOID,
        core::mem::size_of_val(&Mem) as ULONG,
    ))
    {
        crate::kdb_printf!(b"Couldn't access memory at 0x%p\n\0", Eip);
        return 0u8;
    }

    while (i < core::mem::size_of_val(&Mem) as ULONG) && (Mem[i as usize] == 0x66 || Mem[i as usize] == 0x67)
    {
        i += 1;
    }

    if i == core::mem::size_of_val(&Mem) as ULONG
    {
        return 0u8;
    }

    if Mem[i as usize] == 0xE8 || Mem[i as usize] == 0x9A || Mem[i as usize] == 0xF2 || Mem[i as usize] == 0xF3 ||
        (((i + 1) < core::mem::size_of_val(&Mem) as ULONG) && Mem[i as usize] == 0xFF && (Mem[(i+1) as usize] & 0x38) == 0x10)
    {
        return 1u8;
    }

    0u8
}

unsafe fn KdbpStepOverInstruction(
    Eip: ULONG_PTR,
) -> BOOLEAN
{
    let mut InstLen: LONG;

    if KdbpShouldStepOverInstruction(Eip) == 0u8 {
        return 0u8;
    }

    InstLen = KdbpGetInstLength(Eip);
    if InstLen < 1 {
        return 0u8;
    }

    if !NT_SUCCESS(KdbpInsertBreakPoint(
        Eip.wrapping_add(InstLen as ULONG_PTR),
        KdbBreakPointTemporary,
        0,
        KdbAccessRead,
        core::ptr::null_mut(),
        0u8,
        core::ptr::null_mut(),
    ))
    {
        return 0u8;
    }

    1u8
}

unsafe fn KdbpStepIntoInstruction(
    Eip: ULONG_PTR,
) -> BOOLEAN
{
    let mut Idtr: KDESCRIPTOR = core::mem::zeroed();
    let mut Mem: [UCHAR; 2] = [0; 2];
    let IntVect: INT;
    let mut IntDesc: [ULONG; 2] = [0; 2];
    let TargetEip: ULONG_PTR;

    if !NT_SUCCESS(KdbpSafeReadMemory(
        Mem.as_mut_ptr() as PVOID,
        Eip as PVOID,
        core::mem::size_of_val(&Mem) as ULONG,
    ))
    {
        return 0u8;
    }

    if Mem[0] == 0xcc {
        IntVect = 3;
    } else if Mem[0] == 0xcd {
        IntVect = Mem[1] as INT;
    } else if Mem[0] == 0xce && ((*KdbCurrentTrapFrame).EFlags & (1 << 11)) != 0 {
        IntVect = 4;
    } else {
        return 0u8;
    }

    if IntVect < 32 {
        return 0u8;
    }

    __sidt(core::ptr::addr_of_mut!(Idtr.Limit) as PVOID);
    if IntVect >= ((Idtr.Limit + 1) / 8) as INT {
        return 1u8;
    }

    if !NT_SUCCESS(KdbpSafeReadMemory(
        IntDesc.as_mut_ptr() as PVOID,
        (Idtr.Base as ULONG_PTR + (IntVect as ULONG_PTR * 8)) as PVOID,
        core::mem::size_of_val(&IntDesc) as ULONG,
    ))
    {
        return 0u8;
    }

    if (IntDesc[1] & (1 << 15)) == 0 {
        return 0u8;
    }
    if (IntDesc[1] & 0x1f00) == 0x0500 {
        return 0u8;
    } else if ((IntDesc[1] & 0x1fe0) == 0x0e00) || ((IntDesc[1] & 0x1fe0) == 0x0f00) {
        TargetEip = ((IntDesc[1] & 0xffff0000) | (IntDesc[0] & 0x0000ffff)) as ULONG_PTR;
    } else {
        return 0u8;
    }

    if !NT_SUCCESS(KdbpInsertBreakPoint(
        TargetEip,
        KdbBreakPointTemporary,
        0,
        KdbAccessRead,
        core::ptr::null_mut(),
        0u8,
        core::ptr::null_mut(),
    ))
    {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpGetNextBreakPointNr(
    Start: ULONG
) -> LONG {
    let mut Start = Start;
    while Start < (core::mem::size_of_val(&*(&raw const KdbBreakPoints)) / core::mem::size_of::<KDB_BREAKPOINT>()) as ULONG {
        if KdbBreakPoints[Start as usize].Type != KdbBreakPointNone {
            return Start as LONG;
        }
        Start += 1;
    }
    
    -1
}

#[no_mangle]
pub unsafe extern "C" fn KdbpGetBreakPointInfo(
    BreakPointNr: ULONG,
    Address: *mut ULONG_PTR,
    Type: *mut KDB_BREAKPOINT_TYPE,
    Size: *mut UCHAR,
    AccessType: *mut KDB_ACCESS_TYPE,
    DebugReg: *mut UCHAR,
    Enabled: *mut BOOLEAN,
    Global: *mut BOOLEAN,
    Process: *mut PEPROCESS,
    ConditionExpression: *mut PCHAR,
) -> BOOLEAN {
    let bp: PKDB_BREAKPOINT;

    if BreakPointNr >= RTL_NUMBER_OF!((*(&raw const KdbBreakPoints))) ||
        KdbBreakPoints[BreakPointNr as usize].Type == KdbBreakPointNone
    {
        return 0u8;
    }

    bp = (*(&raw mut KdbBreakPoints)).as_mut_ptr().add(BreakPointNr as usize);
    // All writes use write_volatile to defeat LLVM captures(none) elimination —
    // see project notes on `tools/lint_kernel.py` and the captures(none) bug.
    if !Address.is_null() {
        core::ptr::write_volatile(Address, (*bp).Address);
    }

    if !Type.is_null() {
        core::ptr::write_volatile(Type, (*bp).Type);
    }

    if (*bp).Type == KdbBreakPointHardware {
        if !Size.is_null() {
            core::ptr::write_volatile(Size, (*bp).Data.Hw.Size);
        }

        if !AccessType.is_null() {
            core::ptr::write_volatile(AccessType, (*bp).Data.Hw.AccessType);
        }

        if !DebugReg.is_null() && (*bp).Enabled != 0 {
            core::ptr::write_volatile(DebugReg, (*bp).Data.Hw.DebugReg);
        }
    }

    if !Enabled.is_null() {
        core::ptr::write_volatile(Enabled, (*bp).Enabled);
    }

    if !Global.is_null() {
        core::ptr::write_volatile(Global, (*bp).Global);
    }

    if !Process.is_null() {
        core::ptr::write_volatile(Process, (*bp).Process);
    }

    if !ConditionExpression.is_null() {
        core::ptr::write_volatile(ConditionExpression, (*bp).ConditionExpression);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpInsertBreakPoint(
    Address: ULONG_PTR,
    Type: KDB_BREAKPOINT_TYPE,
    Size: UCHAR,
    AccessType: KDB_ACCESS_TYPE,
    ConditionExpression: PCHAR,
    Global: BOOLEAN,
    BreakPointNr: PLONG,
) -> NTSTATUS {
    let mut i: LONG_PTR;
    let mut Condition: PVOID;
    let mut ConditionExpressionDup: PCHAR;
    let mut ErrOffset: LONG = 0;
    let mut ErrMsg: [CHAR; 128] = [0; 128];

    if Type == KdbBreakPointHardware {
        if (Address % Size as ULONG_PTR) != 0 {
            crate::kdb_printf!(b"Address (0x%p) must be aligned to a multiple of the size (%d)\n\0",
                Address,
                Size as i32,
            );
            return STATUS_UNSUCCESSFUL;
        }

        if AccessType == KdbAccessExec && Size != 1 {
            KdbPuts(b"Size must be 1 for execution breakpoints.\n\0".as_ptr() as PCSTR);
            return STATUS_UNSUCCESSFUL;
        }
    }

    if KdbBreakPointCount == KDB_MAXIMUM_BREAKPOINT_COUNT as ULONG {
        return STATUS_UNSUCCESSFUL;
    }

    if !ConditionExpression.is_null() {
        Condition = KdbpRpnParseExpression(
            ConditionExpression as *mut u8,
            &mut ErrOffset,
            ErrMsg.as_mut_ptr() as *mut u8,
        );
        if Condition.is_null() {
            if ErrOffset >= 0 {
                crate::kdb_printf!(b"Couldn't parse expression: %s at character %d\n\0",
                    ErrMsg.as_ptr(),
                    ErrOffset,
                );
            } else {
                crate::kdb_printf!(b"Couldn't parse expression: %s\0",
                    ErrMsg.as_ptr(),
                );
            }

            return STATUS_UNSUCCESSFUL;
        }

        i = strlen(ConditionExpression as PCSTR) as LONG_PTR;
        i += 1;
        ConditionExpressionDup = ExAllocatePoolWithTag(NonPagedPool, i as SIZE_T, TAG_KDBG) as PCHAR;
        core::ptr::copy_nonoverlapping(ConditionExpression, ConditionExpressionDup, i as usize);
    } else {
        Condition = core::ptr::null_mut();
        ConditionExpressionDup = core::ptr::null_mut();
    }

    if Type == KdbBreakPointTemporary {
        i = (RTL_NUMBER_OF!((*(&raw const KdbBreakPoints))) - 1) as LONG_PTR;
        while i >= 0 {
            if (*(*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(i as isize)).Type == KdbBreakPointNone {
                break;
            }
            i -= 1;
        }
    } else {
        i = 0;
        while i < RTL_NUMBER_OF!((*(&raw const KdbBreakPoints))) as LONG_PTR {
            if (*(*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(i as isize)).Type == KdbBreakPointNone {
                break;
            }
            i += 1;
        }
    }

    (*(*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(i as isize)).Type = Type;
    (*(*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(i as isize)).Address = Address;
    (*(*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(i as isize)).Enabled = 0u8;
    (*(*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(i as isize)).Global = Global;
    (*(*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(i as isize)).Process = KdbCurrentProcess;
    (*(*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(i as isize)).ConditionExpression = ConditionExpressionDup;
    (*(*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(i as isize)).Condition = Condition;

    if Type == KdbBreakPointHardware {
        (*(*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(i as isize)).Data.Hw.Size = Size;
        (*(*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(i as isize)).Data.Hw.AccessType = AccessType;
    }

    KdbBreakPointCount += 1;

    if Type != KdbBreakPointTemporary {
        crate::kdb_printf!(b"Breakpoint %d inserted.\n\0", i as i32);
    }

    KdbpEnableBreakPoint(i as LONG, core::ptr::null_mut());

    if !BreakPointNr.is_null() {
        // write_volatile to defeat LLVM captures(none).
        core::ptr::write_volatile(BreakPointNr, i as LONG);
    }

    STATUS_SUCCESS
}

unsafe fn KdbpIsBreakPointOurs(
    ExceptionCode: NTSTATUS,
    Context: PCONTEXT,
) -> LONG {
    let mut i: ULONG;

    if ExceptionCode == STATUS_BREAKPOINT {
        let BpPc: ULONG_PTR = KeGetContextPc(Context).wrapping_sub(1);
        i = 0;
        while i < KdbSwBreakPointCount {
            if (*KdbSwBreakPoints[i as usize]).Address == BpPc {
                return KdbSwBreakPoints[i as usize].offset_from((*(&raw const KdbBreakPoints)).as_ptr()) as LONG;
            }
            i = i.wrapping_add(1);
        }
    } else if ExceptionCode == STATUS_SINGLE_STEP {
        let mut DebugReg: UCHAR;

        i = 0;
        while i < KdbHwBreakPointCount {
            DebugReg = (*KdbHwBreakPoints[i as usize]).Data.Hw.DebugReg;

            if ((*Context).Dr6 & (1 << DebugReg as ULONG)) != 0 {
                return KdbHwBreakPoints[i as usize].offset_from((*(&raw const KdbBreakPoints)).as_ptr()) as LONG;
            }
            i = i.wrapping_add(1);
        }
    }

    -1
}

#[no_mangle]
pub unsafe extern "C" fn KdbpDisableBreakPoint(
    mut BreakPointNr: LONG,
    mut BreakPoint: PKDB_BREAKPOINT,
) -> BOOLEAN
{
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    if BreakPointNr < 0
    {
        BreakPointNr = BreakPoint.offset_from((*(&raw const KdbBreakPoints)).as_ptr()) as LONG;
    }

    if BreakPointNr < 0 || BreakPointNr >= KDB_MAXIMUM_BREAKPOINT_COUNT as LONG
    {
        crate::kdb_printf!(b"Invalid breakpoint: %d\n\0", BreakPointNr);
        return 0u8;
    }

    if BreakPoint.is_null()
    {
        BreakPoint = (*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(BreakPointNr as isize);
    }

    if (*BreakPoint).Type == KdbBreakPointNone
    {
        crate::kdb_printf!(b"Invalid breakpoint: %d\n\0", BreakPointNr);
        return 0u8;
    }

    if (*BreakPoint).Enabled == 0u8
    {
        crate::kdb_printf!(b"Breakpoint %d is not enabled.\n\0", BreakPointNr);
        return 1u8;
    }

    if (*BreakPoint).Type == KdbBreakPointSoftware ||
        (*BreakPoint).Type == KdbBreakPointTemporary
    {
        Status = KdbpOverwriteInstruction((*BreakPoint).Process, (*BreakPoint).Address,
                                          (*BreakPoint).Data.SavedInstruction, core::ptr::null_mut());

        if !NT_SUCCESS(Status)
        {
            KdbPuts(b"Couldn't restore original instruction.\n\0".as_ptr() as PCSTR);
            return 0u8;
        }

        i = 0;
        while i < KdbSwBreakPointCount
        {
            if KdbSwBreakPoints[i as usize] == BreakPoint
            {
                KdbSwBreakPointCount -= 1;
                KdbSwBreakPoints[i as usize] = KdbSwBreakPoints[KdbSwBreakPointCount as usize];
                i = MAXULONG;
                break;
            }
            i += 1;
        }

        if i != MAXULONG
        {
        }
    }
    else
    {
        KdbTrapFrame.Dr7 &= !(0x3 << ((*BreakPoint).Data.Hw.DebugReg * 2));
        if (KdbTrapFrame.Dr7 & 0xFF) == 0
        {
            KdbTrapFrame.Dr7 &= 0xFFFFFCFF;
        }

        i = 0;
        while i < KdbHwBreakPointCount
        {
            if KdbHwBreakPoints[i as usize] == BreakPoint
            {
                KdbHwBreakPointCount -= 1;
                KdbHwBreakPoints[i as usize] = KdbHwBreakPoints[KdbHwBreakPointCount as usize];
                i = MAXULONG;
                break;
            }
            i += 1;
        }

        if i != MAXULONG
        {
        }
    }

    (*BreakPoint).Enabled = 0u8;
    if (*BreakPoint).Type != KdbBreakPointTemporary
    {
        crate::kdb_printf!(b"Breakpoint %d disabled.\n\0", BreakPointNr);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpGetEnterCondition(
    ExceptionNr: LONG,
    FirstChance: BOOLEAN,
    Condition: *mut KDB_ENTER_CONDITION,
) -> BOOLEAN {
    
    if ExceptionNr >= (core::mem::size_of_val(&*(&raw const KdbEnterConditions)) / core::mem::size_of::<[KDB_ENTER_CONDITION; 2]>()) as LONG {
        return 0u8;
    }

    *Condition = KdbEnterConditions[ExceptionNr as usize][if FirstChance != 0 { 0 } else { 1 }];
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpSetEnterCondition(
    ExceptionNr: LONG,
    FirstChance: BOOLEAN,
    Condition: KDB_ENTER_CONDITION,
) -> BOOLEAN {
    if ExceptionNr < 0 {
        let mut ExceptionNr = 0i32;
        while ExceptionNr < RTL_NUMBER_OF!((*(&raw const KdbEnterConditions))) as LONG {
            if ExceptionNr == 1 || ExceptionNr == 8 ||
               ExceptionNr == 9 || ExceptionNr == 15 {
                ExceptionNr += 1;
                continue;
            }

            KdbEnterConditions[ExceptionNr as usize][if FirstChance != 0 { 0 } else { 1 }] = Condition;
            ExceptionNr += 1;
        }
    } else {
        if ExceptionNr >= RTL_NUMBER_OF!((*(&raw const KdbEnterConditions))) as LONG ||
           ExceptionNr == 1 || ExceptionNr == 8 ||
           ExceptionNr == 9 || ExceptionNr == 15 {
            return 0u8;
        }

        KdbEnterConditions[ExceptionNr as usize][if FirstChance != 0 { 0 } else { 1 }] = Condition;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpAttachToThread(
    ThreadId: PVOID) -> BOOLEAN
{
    let mut Thread: PETHREAD = core::ptr::null_mut();
    let mut Process: PEPROCESS;

    if !NT_SUCCESS(PsLookupThreadByThreadId(ThreadId, &mut Thread))
    {
        crate::kdbp_print!(b"Invalid thread id: 0x%08x\n\0", ThreadId as ULONG_PTR);
        return 0u8;
    }
    Process = (*Thread).ThreadsProcess;

    if KeIsExecutingDpc() != 0 && Process != KdbCurrentProcess
    {
        crate::kdbp_print!(b"Cannot attach to thread within another process while executing a DPC.\n\0");
        ObDereferenceObject(Thread as PVOID);
        return 0u8;
    }

    if KdbCurrentThread != KdbOriginalThread
    {
    }
    else
    {
    }

    if Thread != KdbOriginalThread
    {
        KdbpKdbTrapFrameFromKernelStack(
            (*Thread).Tcb.KernelStack,
            core::ptr::addr_of_mut!(KdbThreadTrapFrame),
        );
        KdbCurrentTrapFrame = core::ptr::addr_of_mut!(KdbThreadTrapFrame);
    }
    else
    {
        KdbCurrentTrapFrame = core::ptr::addr_of_mut!(KdbTrapFrame);
    }
    KdbCurrentThread = Thread;

    if KdbCurrentProcess != Process
    {
        if KdbCurrentProcess != KdbOriginalProcess
        {
            KeUnstackDetachProcess(core::ptr::addr_of_mut!(KdbApcState));
        }

        if KdbOriginalProcess != Process
        {
            KeStackAttachProcess(
                core::ptr::addr_of_mut!((*Process).Pcb),
                core::ptr::addr_of_mut!(KdbApcState),
            );
        }

        KdbCurrentProcess = Process;
    }

    ObDereferenceObject(Thread as PVOID);
    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn KdbpAttachToProcess(
    ProcessId: PVOID) -> BOOLEAN
{
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let Thread: PETHREAD;
    let Entry: PLIST_ENTRY;

    if !NT_SUCCESS(PsLookupProcessByProcessId(ProcessId, &mut Process))
    {
        crate::kdbp_print!(b"Invalid process id: 0x%08x\n\0", ProcessId as ULONG_PTR);
        return 0u8;
    }

    Entry = (*Process).ThreadListHead.Flink;
    ObDereferenceObject(Process as PVOID);
    if Entry == core::ptr::addr_of_mut!((*KdbCurrentProcess).ThreadListHead)
    {
        crate::kdbp_print!(b"No threads in process 0x%p, cannot attach to process!\n\0", ProcessId);
        return 0u8;
    }

    Thread = (Entry as *mut u8).offset(-(core::mem::offset_of!(_ETHREAD, ThreadListEntry) as isize)) as PETHREAD;

    return KdbpAttachToThread((*Thread).Cid.UniqueThread);
}

unsafe extern "C" fn KdbpCallMainLoop() {
    KdbpCliMainLoop(KdbEnteredOnSingleStep);
}

unsafe fn KdbpInternalEnter(Procedure: Option<unsafe extern "C" fn()>) {
    let Thread: PETHREAD;
    let SavedInitialStack: *mut core::ffi::c_void;
    let SavedStackBase: *mut core::ffi::c_void;
    let SavedKernelStack: *mut core::ffi::c_void;
    let SavedStackLimit: ULONG_PTR;

    KbdDisableMouse();

    if KdpDebugMode.Screen != 0 {
        KdpScreenAcquire();
    }

    Thread = PsGetCurrentThread();
    SavedInitialStack = (*Thread).Tcb.InitialStack;
    SavedStackBase = (*Thread).Tcb.StackBase;
    SavedStackLimit = (*Thread).Tcb.StackLimit as ULONG_PTR;
    SavedKernelStack = (*Thread).Tcb.KernelStack;
    (*Thread).Tcb.InitialStack = ((*(&raw mut KdbStack)).as_mut_ptr() as *mut i8).add(KDB_STACK_SIZE as usize) as *mut core::ffi::c_void;
    (*Thread).Tcb.StackBase = ((*(&raw mut KdbStack)).as_mut_ptr() as *mut i8).add(KDB_STACK_SIZE as usize) as *mut core::ffi::c_void;
    (*Thread).Tcb.StackLimit = (*(&raw mut KdbStack)).as_mut_ptr() as _;
    (*Thread).Tcb.KernelStack = ((*(&raw mut KdbStack)).as_mut_ptr() as *mut i8).add(KDB_STACK_SIZE as usize) as *mut core::ffi::c_void;

    KdbpStackSwitchAndCall(
        (*(&raw mut KdbStack)).as_mut_ptr().add((KDB_STACK_SIZE - KDB_STACK_RESERVE) as usize) as PVOID,
        Procedure,
    );

    (*Thread).Tcb.InitialStack = SavedInitialStack;
    (*Thread).Tcb.StackBase = SavedStackBase;
    (*Thread).Tcb.StackLimit = SavedStackLimit as _;
    (*Thread).Tcb.KernelStack = SavedKernelStack;

    if KdpDebugMode.Screen != 0 {
        KdpScreenRelease();
    }

    KbdEnableMouse();
}

unsafe fn KdbpGetExceptionNumberFromStatus(
    ExceptionCode: NTSTATUS,
) -> ULONG {
    let Ret: ULONG;

    match ExceptionCode {
        STATUS_INTEGER_DIVIDE_BY_ZERO => {
            Ret = 0;
        }
        STATUS_SINGLE_STEP => {
            Ret = 1;
        }
        STATUS_BREAKPOINT => {
            Ret = 3;
        }
        STATUS_INTEGER_OVERFLOW => {
            Ret = 4;
        }
        STATUS_ARRAY_BOUNDS_EXCEEDED => {
            Ret = 5;
        }
        STATUS_ILLEGAL_INSTRUCTION => {
            Ret = 6;
        }
        STATUS_FLOAT_INVALID_OPERATION => {
            Ret = 7;
        }
        STATUS_STACK_OVERFLOW => {
            Ret = 12;
        }
        STATUS_ACCESS_VIOLATION => {
            Ret = 14;
        }
        STATUS_DATATYPE_MISALIGNMENT => {
            Ret = 17;
        }
        STATUS_FLOAT_MULTIPLE_TRAPS => {
            Ret = 18;
        }
        STATUS_ASSERTION_FAILURE => {
            Ret = 20;
        }
        _ => {
            Ret = (core::mem::size_of_val(&*(&raw const KdbEnterConditions)) / core::mem::size_of::<KDB_ENTER_CONDITION>()) as ULONG - 1;
        }
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "C" fn KdbEnterDebuggerException(
    ExceptionRecord: PEXCEPTION_RECORD64,
    PreviousMode: KPROCESSOR_MODE,
    Context: PCONTEXT,
    FirstChance: BOOLEAN,
) -> KD_CONTINUE_TYPE {
    let mut EnterCondition: KDB_ENTER_CONDITION;
    let mut ContinueType: KD_CONTINUE_TYPE = kdHandleException;
    let mut BreakPoint: PKDB_BREAKPOINT;
    let ExpNr: ULONG;
    let mut ull: ULONGLONG;
    let mut Resume: BOOLEAN = 0u8;
    let mut EnterConditionMet: BOOLEAN = 1u8;
    let OldEflags: ULONG;
    let mut OldIrql: KIRQL;
    let ExceptionCode: NTSTATUS;
    let mut EntryPoint: Option<unsafe extern "C" fn()> = Some(KdbpCallMainLoop);

    ExceptionCode = if !ExceptionRecord.is_null() {
        (*ExceptionRecord).ExceptionCode
    } else {
        STATUS_BREAKPOINT
    };

    KdbCurrentProcess = PsGetCurrentProcess();

    if ExceptionCode == STATUS_SINGLE_STEP
        || ExceptionCode == STATUS_BREAKPOINT
        || ExceptionCode == STATUS_ASSERTION_FAILURE
    {
        ContinueType = kdContinue;
    }

    ExpNr = KdbpGetExceptionNumberFromStatus(ExceptionCode);
    EnterCondition = KdbEnterConditions[ExpNr as usize][if FirstChance != 0 { 0 } else { 1 }];
    if EnterCondition == KdbDoNotEnter
        || (EnterCondition == KdbEnterFromUmode && PreviousMode == 0)
        || (EnterCondition == KdbEnterFromKmode && PreviousMode != 0)
    {
        EnterConditionMet = 0u8;
    }

    KdbLastBreakPointNr = -1;
    KdbEnteredOnSingleStep = 0u8;

    if FirstChance != 0
        && (ExceptionCode == STATUS_SINGLE_STEP || ExceptionCode == STATUS_BREAKPOINT)
    {
        KdbLastBreakPointNr = KdbpIsBreakPointOurs(ExceptionCode, Context);
        if KdbLastBreakPointNr >= 0 {
            BreakPoint = (*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(KdbLastBreakPointNr as isize);

            if ExceptionCode == STATUS_BREAKPOINT {
                if !NT_SUCCESS(KdbpOverwriteInstruction(
                    KdbCurrentProcess,
                    (*BreakPoint).Address,
                    (*BreakPoint).Data.SavedInstruction,
                    core::ptr::null_mut(),
                )) {
                    KdbPuts(b"Couldn't restore original instruction after INT3! Cannot continue execution.\n\0".as_ptr() as PCSTR);
                    KeBugCheck(0);
                }

                KeSetContextPc(Context, KeGetContextPc(Context).wrapping_sub(1));
            }

            if (*BreakPoint).Type == KdbBreakPointHardware
                && (*BreakPoint).Data.Hw.AccessType == KdbAccessExec
            {
                Resume = 1u8;
            } else if (*BreakPoint).Type == KdbBreakPointTemporary
                && (*BreakPoint).Process == KdbCurrentProcess
            {
                KdbpDeleteBreakPoint(-1, BreakPoint);

                KdbNumSingleSteps = KdbNumSingleSteps.wrapping_sub(1);
                if KdbNumSingleSteps > 0 {
                    if (KdbSingleStepOver != 0 && KdbpStepOverInstruction(KeGetContextPc(Context)) == 0)
                        || (KdbSingleStepOver == 0 && KdbpStepIntoInstruction(KeGetContextPc(Context)) == 0)
                    {
                        (*Context).EFlags |= EFLAGS_TF;
                    }

                    'continue_execution: loop {
                        if ExceptionCode == STATUS_BREAKPOINT {
                            if Resume != 0 {
                                (*Context).EFlags |= EFLAGS_RF;
                            }

                            (*Context).Dr6 &= !0x0000e00f;

                            if !(KdbEnteredOnSingleStep != 0 && KdbSingleStepOver != 0) {
                                KeSetContextPc(Context, KeGetContextPc(Context).wrapping_add(KDB_BREAKPOINT_SIZE as ULONG_PTR));
                            }
                        }

                        return ContinueType;
                    }
                }

                KdbEnteredOnSingleStep = 1u8;
            } else if (*BreakPoint).Type == KdbBreakPointSoftware
                || (*BreakPoint).Type == KdbBreakPointTemporary
            {
                (*Context).EFlags |= EFLAGS_TF;
                KdbBreakPointToReenable = BreakPoint;
            }

            if (*BreakPoint).Global == 0 && (*BreakPoint).Process != KdbCurrentProcess {
                'continue_execution: loop {
                    if ExceptionCode == STATUS_BREAKPOINT {
                        if Resume != 0 {
                            (*Context).EFlags |= EFLAGS_RF;
                        }

                        (*Context).Dr6 &= !0x0000e00f;

                        if !(KdbEnteredOnSingleStep != 0 && KdbSingleStepOver != 0) {
                            KeSetContextPc(Context, KeGetContextPc(Context).wrapping_add(KDB_BREAKPOINT_SIZE as ULONG_PTR));
                        }
                    }

                    return ContinueType;
                }
            }

            if !(*BreakPoint).Condition.is_null() {
                KdbTrapFrame = *Context;

                ull = 0;
                if KdbpRpnEvaluateParsedExpression(
                    (*BreakPoint).Condition,
                    core::ptr::addr_of_mut!(KdbTrapFrame),
                    core::ptr::addr_of_mut!(ull),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) == 0
                    || ull == 0
                {
                    'continue_execution: loop {
                        if ExceptionCode == STATUS_BREAKPOINT {
                            if Resume != 0 {
                                (*Context).EFlags |= EFLAGS_RF;
                            }

                            (*Context).Dr6 &= !0x0000e00f;

                            if !(KdbEnteredOnSingleStep != 0 && KdbSingleStepOver != 0) {
                                KeSetContextPc(Context, KeGetContextPc(Context).wrapping_add(KDB_BREAKPOINT_SIZE as ULONG_PTR));
                            }
                        }

                        return ContinueType;
                    }
                }
            }

            if (*BreakPoint).Type == KdbBreakPointSoftware {
                crate::kdb_printf!(b"\nEntered debugger on breakpoint #%d: EXEC 0x%04x:0x%p\n\0",
                    KdbLastBreakPointNr,
                    ((*Context).SegCs & 0xffff) as ULONG,
                    KeGetContextPc(Context),
                );
            } else if (*BreakPoint).Type == KdbBreakPointHardware {
                crate::kdb_printf!(b"\nEntered debugger on breakpoint #%d: %s 0x%08x\n\0",
                    KdbLastBreakPointNr,
                    if (*BreakPoint).Data.Hw.AccessType == KdbAccessRead {
                        b"READ\0".as_ptr() as PCSTR
                    } else if (*BreakPoint).Data.Hw.AccessType == KdbAccessWrite {
                        b"WRITE\0".as_ptr() as PCSTR
                    } else if (*BreakPoint).Data.Hw.AccessType == KdbAccessReadWrite {
                        b"RDWR\0".as_ptr() as PCSTR
                    } else {
                        b"EXEC\0".as_ptr() as PCSTR
                    },
                    (*BreakPoint).Address,
                );
            }
        }
    } else if ExceptionCode == STATUS_SINGLE_STEP {
        if ((*Context).Dr6 & 0xf) == 0 && !KdbBreakPointToReenable.is_null() {
            BreakPoint = KdbBreakPointToReenable;
            KdbBreakPointToReenable = core::ptr::null_mut();

            if !NT_SUCCESS(KdbpOverwriteInstruction(
                KdbCurrentProcess,
                (*BreakPoint).Address,
                0xCC,
                core::ptr::addr_of_mut!((*BreakPoint).Data.SavedInstruction),
            )) {
                crate::kdb_printf!(b"Warning: Couldn't reenable breakpoint %d\n\0",
                    BreakPoint.offset_from((*(&raw const KdbBreakPoints)).as_ptr()),
                );
            }

            if KdbNumSingleSteps == 0 {
                (*Context).EFlags &= !EFLAGS_TF;
            }

            if KdbpEvenThoughWeHaveABreakPointToReenableWeAlsoHaveARealSingleStep == 0 {
                'continue_execution: loop {
                    if ExceptionCode == STATUS_BREAKPOINT {
                        if Resume != 0 {
                            (*Context).EFlags |= EFLAGS_RF;
                        }

                        (*Context).Dr6 &= !0x0000e00f;

                        if !(KdbEnteredOnSingleStep != 0 && KdbSingleStepOver != 0) {
                            KeSetContextPc(Context, KeGetContextPc(Context).wrapping_add(KDB_BREAKPOINT_SIZE as ULONG_PTR));
                        }
                    }

                    return ContinueType;
                }
            }
        }

        KdbpEvenThoughWeHaveABreakPointToReenableWeAlsoHaveARealSingleStep = 0u8;

        if ((*Context).Dr6 & 0xf) == 0 && KdbNumSingleSteps > 0 {
            KdbNumSingleSteps = KdbNumSingleSteps.wrapping_sub(1);
            if KdbNumSingleSteps > 0 {
                if (KdbSingleStepOver != 0 && KdbpStepOverInstruction(KeGetContextPc(Context)) != 0)
                    || (KdbSingleStepOver == 0 && KdbpStepIntoInstruction(KeGetContextPc(Context)) != 0)
                {
                    (*Context).EFlags &= !EFLAGS_TF;
                } else {
                    (*Context).EFlags |= EFLAGS_TF;
                }

                'continue_execution: loop {
                    if ExceptionCode == STATUS_BREAKPOINT {
                        if Resume != 0 {
                            (*Context).EFlags |= EFLAGS_RF;
                        }

                        (*Context).Dr6 &= !0x0000e00f;

                        if !(KdbEnteredOnSingleStep != 0 && KdbSingleStepOver != 0) {
                            KeSetContextPc(Context, KeGetContextPc(Context).wrapping_add(KDB_BREAKPOINT_SIZE as ULONG_PTR));
                        }
                    }

                    return ContinueType;
                }
            } else {
                (*Context).EFlags &= !EFLAGS_TF;
                KdbEnteredOnSingleStep = 1u8;
            }
        } else {
            if EnterConditionMet == 0 {
                return kdHandleException;
            }

            KdbPuts(b"\nEntered debugger on unexpected debug trap!\n\0".as_ptr() as PCSTR);
        }
    } else if ExceptionCode == STATUS_BREAKPOINT {
        if !KdbInitFileBuffer.is_null() {
            EntryPoint = Some(KdbpCliInterpretInitFile);
        } else {
            if EnterConditionMet == 0 {
                return kdHandleException;
            }

            crate::kdb_printf!(b"\nEntered debugger on embedded INT3 at 0x%04x:0x%p.\n\0",
                ((*Context).SegCs & 0xffff) as ULONG,
                KeGetContextPc(Context),
            );
        }
    } else {
        let ExceptionString: PCSTR = if (ExpNr as usize) < ExceptionNrToString.len() {
            ExceptionNrToString[ExpNr as usize].as_ptr() as PCSTR
        } else {
            b"Unknown/User defined exception\0".as_ptr() as PCSTR
        };

        if EnterConditionMet == 0 {
            return ContinueType;
        }

        crate::kdb_printf!(b"\nEntered debugger on %s-chance exception (Exception Code: 0x%x) (%s)\n\0",
            if FirstChance != 0 {
                b"first\0".as_ptr() as PCSTR
            } else {
                b"last\0".as_ptr() as PCSTR
            },
            ExceptionCode,
            ExceptionString,
        );

        if ExceptionCode == STATUS_ACCESS_VIOLATION
            && !ExceptionRecord.is_null()
            && (*ExceptionRecord).NumberParameters != 0
        {
            let TrapCr2: ULONG_PTR = __readcr2();
            crate::kdb_printf!(b"Memory at 0x%p could not be accessed\n\0",
                TrapCr2,
            );
        }
    }

    KdbNumSingleSteps = 0;

    KdbCurrentProcess = PsGetCurrentProcess();
    KdbOriginalProcess = KdbCurrentProcess;
    KdbCurrentThread = PsGetCurrentThread();
    KdbOriginalThread = KdbCurrentThread;
    KdbCurrentTrapFrame = core::ptr::addr_of_mut!(KdbTrapFrame);

    KdbTrapFrame = *Context;

    OldEflags = __readeflags() as ULONG;
    _disable();

    OldIrql = KeGetCurrentIrql();
    if OldIrql > DISPATCH_LEVEL {
        KeLowerIrql(DISPATCH_LEVEL);
    }

    if InterlockedIncrement(core::ptr::addr_of_mut!(KdbEntryCount)) > 1 {
        __writeeflags(OldEflags as ULONG_PTR);
        return kdHandleException;
    }

    KdbpInternalEnter(EntryPoint);

    if KdbNumSingleSteps > 0 {
        KdbpEvenThoughWeHaveABreakPointToReenableWeAlsoHaveARealSingleStep = 1u8;

        if (KdbSingleStepOver != 0 && KdbpStepOverInstruction(KeGetContextPc(KdbCurrentTrapFrame)) != 0)
            || (KdbSingleStepOver == 0 && KdbpStepIntoInstruction(KeGetContextPc(KdbCurrentTrapFrame)) != 0)
        {
        } else {
            KdbTrapFrame.EFlags |= EFLAGS_TF;
        }
    }

    if KdbCurrentProcess != KdbOriginalProcess {
        KeUnstackDetachProcess(core::ptr::addr_of_mut!(KdbApcState));
    }

    *Context = KdbTrapFrame;

    InterlockedDecrement(core::ptr::addr_of_mut!(KdbEntryCount));

    if OldIrql > DISPATCH_LEVEL {
        KeRaiseIrql(OldIrql, core::ptr::addr_of_mut!(OldIrql));
    }

    __writeeflags(OldEflags as ULONG_PTR);

    if KdbpBugCheckRequested != 0 {
        KdbpBugCheckRequested = 0u8;
        KeBugCheck(MANUALLY_INITIATED_CRASH);
    }

    'continue_execution: loop {
        if ExceptionCode == STATUS_BREAKPOINT {
            if Resume != 0 {
                (*Context).EFlags |= EFLAGS_RF;
            }

            (*Context).Dr6 &= !0x0000e00f;

            if !(KdbEnteredOnSingleStep != 0 && KdbSingleStepOver != 0) {
                KeSetContextPc(Context, KeGetContextPc(Context).wrapping_add(KDB_BREAKPOINT_SIZE as ULONG_PTR));
            }
        }

        return ContinueType;
    }
}

#[no_mangle]
pub unsafe extern "C" fn KdbpGetCommandLineSettings(
    p1: PCSTR,
) {

    const fn CONST_STR_LEN(x: &[u8]) -> usize {
        x.len() - 1
    }

    let mut p1 = p1;

    while !p1.is_null() && *p1 != 0 {
        while *p1 == b' ' as i8 {
            p1 = p1.offset(1);
        }

        if _strnicmp(p1, b"FIRSTCHANCE\0".as_ptr() as *const i8, CONST_STR_LEN(b"FIRSTCHANCE\0")) == 0 {
            p1 = p1.offset(CONST_STR_LEN(b"FIRSTCHANCE\0") as isize);
            KdbpSetEnterCondition(-1, 1u8, KdbEnterAlways);
        }

        p1 = strchr(p1, b' ' as i32);
    }
}

#[no_mangle]
pub unsafe extern "C" fn KdbpSafeReadMemory(
    Dest: PVOID,
    Src: PVOID,
    Bytes: ULONG,
) -> NTSTATUS
{
    KdpCopyMemoryChunks(
        Src as ULONG_PTR as ULONG64,
        Dest,
        Bytes,
        0,
        MMDBG_COPY_UNSAFE,
        core::ptr::null_mut(),
    )
}

unsafe fn KdbpSafeWriteMemory(
    Dest: PVOID,
    Src: PVOID,
    Bytes: ULONG,
) -> NTSTATUS
{
    KdpCopyMemoryChunks(
        Dest as ULONG64,
        Src,
        Bytes,
        0,
        MMDBG_COPY_UNSAFE | MMDBG_COPY_WRITE,
        core::ptr::null_mut(),
    )
}

/// Deletes a breakpoint. Translation of `ntoskrnl/kdbg/kdb.c:KdbpDeleteBreakPoint`.
#[no_mangle]
pub unsafe extern "C" fn KdbpDeleteBreakPoint(
    mut BreakPointNr: LONG,
    mut BreakPoint: PKDB_BREAKPOINT,
) -> BOOLEAN {
    if BreakPointNr < 0 {
        // ASSERT(BreakPoint != NULL)
        BreakPointNr = BreakPoint.offset_from((*(&raw const KdbBreakPoints)).as_ptr()) as LONG;
    }

    if BreakPointNr < 0 || BreakPointNr >= KDB_MAXIMUM_BREAKPOINT_COUNT as LONG {
        crate::kdb_printf!(b"Invalid breakpoint: %d\n\0", BreakPointNr);
        return 0u8;
    }

    if BreakPoint.is_null() {
        BreakPoint = (*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(BreakPointNr as isize);
    }

    if (*BreakPoint).Type == KdbBreakPointNone {
        crate::kdb_printf!(b"Invalid breakpoint: %d\n\0", BreakPointNr);
        return 0u8;
    }

    if (*BreakPoint).Enabled != 0
        && KdbpDisableBreakPoint(-1, BreakPoint) == 0
    {
        return 0u8;
    }

    if (*BreakPoint).Type != KdbBreakPointTemporary {
        crate::kdb_printf!(b"Breakpoint %d deleted.\n\0", BreakPointNr);
    }

    (*BreakPoint).Type = KdbBreakPointNone;
    KdbBreakPointCount = KdbBreakPointCount.wrapping_sub(1);

    1u8
}

/// Enables a breakpoint. Translation of `ntoskrnl/kdbg/kdb.c:KdbpEnableBreakPoint`.
#[no_mangle]
pub unsafe extern "C" fn KdbpEnableBreakPoint(
    mut BreakPointNr: LONG,
    mut BreakPoint: PKDB_BREAKPOINT,
) -> BOOLEAN {
    let Status: NTSTATUS;
    let mut i: INT;
    let ul: ULONG;

    if BreakPointNr < 0 {
        // ASSERT(BreakPoint != NULL)
        BreakPointNr = BreakPoint.offset_from((*(&raw const KdbBreakPoints)).as_ptr()) as LONG;
    }

    if BreakPointNr < 0 || BreakPointNr >= KDB_MAXIMUM_BREAKPOINT_COUNT as LONG {
        crate::kdb_printf!(b"Invalid breakpoint: %d\n\0", BreakPointNr);
        return 0u8;
    }

    if BreakPoint.is_null() {
        BreakPoint = (*(&raw mut KdbBreakPoints)).as_mut_ptr().offset(BreakPointNr as isize);
    }

    if (*BreakPoint).Type == KdbBreakPointNone {
        crate::kdb_printf!(b"Invalid breakpoint: %d\n\0", BreakPointNr);
        return 0u8;
    }

    if (*BreakPoint).Enabled != 0 {
        crate::kdb_printf!(b"Breakpoint %d is already enabled.\n\0", BreakPointNr);
        return 1u8;
    }

    if (*BreakPoint).Type == KdbBreakPointSoftware
        || (*BreakPoint).Type == KdbBreakPointTemporary
    {
        if KdbSwBreakPointCount as usize >= KDB_MAXIMUM_SW_BREAKPOINT_COUNT {
            crate::kdb_printf!(b"Maximum number of SW breakpoints (%d) used. Disable another breakpoint in order to enable this one.\n\0",
                KDB_MAXIMUM_SW_BREAKPOINT_COUNT as ULONG,
            );
            return 0u8;
        }

        Status = KdbpOverwriteInstruction(
            (*BreakPoint).Process,
            (*BreakPoint).Address,
            0xCCu8,
            core::ptr::addr_of_mut!((*BreakPoint).Data.SavedInstruction),
        );
        if !NT_SUCCESS(Status) {
            crate::kdb_printf!(b"Couldn't access memory at 0x%p\n\0",
                (*BreakPoint).Address,
            );
            return 0u8;
        }

        KdbSwBreakPoints[KdbSwBreakPointCount as usize] = BreakPoint;
        KdbSwBreakPointCount = KdbSwBreakPointCount.wrapping_add(1);
    } else {
        // Hardware breakpoint
        // (BreakPoint->Data.Hw.AccessType == KdbAccessExec) implies size==1 (asserted in C)

        // (BreakPoint->Address % BreakPoint->Data.Hw.Size) == 0 (asserted in C)

        if KdbHwBreakPointCount as usize >= KDB_MAXIMUM_HW_BREAKPOINT_COUNT {
            crate::kdb_printf!(b"Maximum number of HW breakpoints (%d) already used. Disable another breakpoint in order to enable this one.\n\0",
                KDB_MAXIMUM_HW_BREAKPOINT_COUNT as ULONG,
            );
            return 0u8;
        }

        // Dr7 width: u32 on i386, u64 on amd64. Use a local type alias so
        // the same code works on both. CONTEXT.Dr7 in bindgen is typed as
        // ULONG (u32) on i386 and ULONG64 (u64) on amd64.
        #[cfg(target_arch = "x86")]
        type Dr7 = u32;
        #[cfg(not(target_arch = "x86"))]
        type Dr7 = u64;

        // Find unused hw breakpoint slot.
        // (KDB_MAXIMUM_HW_BREAKPOINT_COUNT == 4 — asserted in C)
        i = 0;
        while (i as usize) < KDB_MAXIMUM_HW_BREAKPOINT_COUNT {
            if (KdbTrapFrame.Dr7 & ((0x3 as Dr7) << (i * 2))) == 0 {
                break;
            }
            i += 1;
        }

        // Set the breakpoint address.
        match i {
            0 => KdbTrapFrame.Dr0 = (*BreakPoint).Address as _,
            1 => KdbTrapFrame.Dr1 = (*BreakPoint).Address as _,
            2 => KdbTrapFrame.Dr2 = (*BreakPoint).Address as _,
            3 => KdbTrapFrame.Dr3 = (*BreakPoint).Address as _,
            _ => {}
        }

        // Enable the global breakpoint.
        KdbTrapFrame.Dr7 |= (0x2 as Dr7) << (i * 2);

        // Enable the exact match bits.
        KdbTrapFrame.Dr7 |= 0x00000300 as Dr7;

        // Clear existing state.
        KdbTrapFrame.Dr7 &= !((0xF as Dr7) << (16 + (i * 4)));

        // Set the breakpoint type.
        ul = match (*BreakPoint).Data.Hw.AccessType {
            KdbAccessExec => 0u32,
            KdbAccessWrite => 1u32,
            KdbAccessRead | KdbAccessReadWrite => 3u32,
        };

        KdbTrapFrame.Dr7 |= (ul as Dr7) << (16 + (i * 4));

        // Set the breakpoint length.
        KdbTrapFrame.Dr7 |= (((*BreakPoint).Data.Hw.Size as Dr7).wrapping_sub(1))
            << (18 + (i * 4));

        // Update KdbCurrentTrapFrame — values are taken from there by the CLI.
        if (core::ptr::addr_of_mut!(KdbTrapFrame) as PKDB_KTRAP_FRAME) != KdbCurrentTrapFrame {
            (*KdbCurrentTrapFrame).Dr0 = KdbTrapFrame.Dr0;
            (*KdbCurrentTrapFrame).Dr1 = KdbTrapFrame.Dr1;
            (*KdbCurrentTrapFrame).Dr2 = KdbTrapFrame.Dr2;
            (*KdbCurrentTrapFrame).Dr3 = KdbTrapFrame.Dr3;
            (*KdbCurrentTrapFrame).Dr6 = KdbTrapFrame.Dr6;
            (*KdbCurrentTrapFrame).Dr7 = KdbTrapFrame.Dr7;
        }

        (*BreakPoint).Data.Hw.DebugReg = i as UCHAR;
        KdbHwBreakPoints[KdbHwBreakPointCount as usize] = BreakPoint;
        KdbHwBreakPointCount = KdbHwBreakPointCount.wrapping_add(1);
    }

    (*BreakPoint).Enabled = 1u8;
    if (*BreakPoint).Type != KdbBreakPointTemporary {
        crate::kdb_printf!(b"Breakpoint %d enabled.\n\0", BreakPointNr);
    }

    1u8
}
