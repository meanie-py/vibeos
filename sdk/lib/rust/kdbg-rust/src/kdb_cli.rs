use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    ANSI_STRING, CONTEXT, CSTRING, DWORD64, _EPROCESS, _ETHREAD,
    ExAllocatePool, ExAllocatePoolZero, ExFreePool, FILE_STANDARD_INFORMATION,
    HalDisplayString, HalReturnToFirmware, _IO_STATUS_BLOCK,
    KPROCESSOR_MODE, KSPECIAL_REGISTERS, KeInitializeSpinLock, LIST_ENTRY,
    NtSetDebugFilterState, _OBJECT_ATTRIBUTES, PCCH, PCONTEXT, PCSTR, PCUNICODE_STRING,
    PETHREAD, PKIPCR, PLDR_DATA_TABLE_ENTRY,
    PSTR, PULONGLONG, PUNICODE_STRING,
    PsLookupProcessByProcessId, PsLookupThreadByThreadId,
    RtlAnsiStringToUnicodeString, RtlFreeUnicodeString, RtlInitUnicodeString,
    SYSDBG_CONTROL_SPACE, _UNICODE_STRING,
    ZwClose, ZwOpenFile, ZwQueryInformationFile, ZwReadFile,
};
// PEPROCESS — use _EPROCESS-backed pointer (full layout) instead of ps::EPROCESS opaque type.
type PEPROCESS = *mut _EPROCESS;
// PsGetCurrentProcess from ps subsystem
use ntkrnl_sys::ps::PsGetCurrentProcess as PsGetCurrentProcessOpaque;
unsafe fn PsGetCurrentProcess() -> PEPROCESS { PsGetCurrentProcessOpaque() as PEPROCESS }
use core::ffi::c_void;

// Bring in shared kdbg types (KDB_BREAKPOINT, KDB_BREAKPOINT_TYPE, KDB_ENTER_CONDITION,
// KDB_ACCESS_TYPE, KD_CONTINUE_TYPE, etc.)
use super::types::*;
use super::tables::{KDB_ACCESS_TYPE_TO_STRING, KDB_ENTER_CONDITION_TO_STRING, NPX_STATE_TO_STRING};

// PRUNTIME_FUNCTION is amd64-only.
#[cfg(target_arch = "x86_64")]
use ntkrnl_sys::ddk::PRUNTIME_FUNCTION;
#[cfg(not(target_arch = "x86_64"))]
type PRUNTIME_FUNCTION = *mut core::ffi::c_void;

// DWORD alias (Win32 API name; ntkrnl_sys exposes ULONG).
type DWORD = ULONG;
// INT alias (C int — i32 on the platforms we target).
type INT = i32;

// va_list — use core's ffi::VaList? Use a simple opaque pointer as we never define a variadic fn here.
// (Variadic definition lives in a C shim; kdb_cli.rs only declares an extern variadic.)
type va_list = *mut core::ffi::c_void;

// ROSSYM symbol resolution types — opaque to Rust (defined in ntoskrnl C side).
#[allow(non_camel_case_types)] pub type PROSSYM_INFO = *mut core::ffi::c_void;
#[allow(non_camel_case_types)] pub type PROSSYM_AGGREGATE = *mut ROSSYM_AGGREGATE;
#[allow(non_camel_case_types)] pub type PROSSYM_AGGREGATE_MEMBER = *mut ROSSYM_AGGREGATE_MEMBER;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ROSSYM_AGGREGATE_MEMBER {
    pub Name: PCHAR,
    pub Type: PCHAR,
    pub BaseOffset: ULONG,
    pub Size: ULONG,
    pub Bits: ULONG,
    pub FirstBit: ULONG,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ROSSYM_AGGREGATE {
    pub NumElements: ULONG,
    pub Elements: *mut ROSSYM_AGGREGATE_MEMBER,
}

// LDR_DATA_TABLE_ENTRY private alias — bindgen has _LDR_DATA_TABLE_ENTRY.
#[allow(non_camel_case_types)] type _LDR_DATA_TABLE_ENTRY = ntkrnl_sys::ddk::_LDR_DATA_TABLE_ENTRY;

// Architecture-private types referenced by i386 trap-frame / GDT inspection.
// Bindgen/DDK doesn't expose these uniformly across arches; declare local opaques
// for non-i386 builds, and use bindgen forms on i386.
#[cfg(target_arch = "x86")]
use ntkrnl_sys::ddk::{KDESCRIPTOR, PKDESCRIPTOR, KGDTENTRY, KTSS, PKTSS};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}

#[cfg(not(target_arch = "x86"))]
#[repr(C)] #[derive(Copy, Clone, Default)]
pub struct KDESCRIPTOR { pub Pad: [u16; 3], pub Limit: u16, pub Base: ULONG_PTR }
#[cfg(not(target_arch = "x86"))]
pub type PKDESCRIPTOR = *mut KDESCRIPTOR;
#[cfg(not(target_arch = "x86"))]
#[repr(C)] #[derive(Copy, Clone)]
pub struct KGDTENTRY { _opaque: [u8; 8] }
#[cfg(not(target_arch = "x86"))]
#[repr(C)] #[derive(Copy, Clone)]
pub struct KTSS { _opaque: [u8; 0] }
#[cfg(not(target_arch = "x86"))]
pub type PKTSS = *mut KTSS;

// SIZE — terminal size (RECTL/SIZE-style: cx, cy).
#[repr(C)] #[derive(Copy, Clone, Default)]
pub struct SIZE { pub cx: u32, pub cy: u32 }

// KDB_NAMED_OBJECT — entry shape of ComponentTable (debug filter component).
#[repr(C)] #[derive(Copy, Clone)]
pub struct KDB_NAMED_OBJECT { pub Name: PCSTR, pub Id: ULONG }

// KD_DISPATCH_TABLE forward — used by KdbInitialize parameter.
// Opaque (defined in ntoskrnl C side; we only forward pointers and accept the C
// driver's table as a typed alias of an opaque struct).
#[repr(C)] #[derive(Copy, Clone)]
pub struct _KD_DISPATCH_TABLE {
    pub KdProvidersList: LIST_ENTRY,
    pub KdpInitRoutine: Option<unsafe extern "system" fn(DispatchTable: PKD_DISPATCH_TABLE, BootPhase: ULONG) -> NTSTATUS>,
    pub KdpPrintRoutine: Option<unsafe extern "C" fn(String: PCCH, Length: ULONG)>,
    pub KdpPromptRoutine: Option<unsafe extern "system" fn() -> ()>,
    _reserved: [u8; 256], // padding so other fields are addressable (ntoskrnl drives layout)
}
pub type PKD_DISPATCH_TABLE = *mut _KD_DISPATCH_TABLE;

// Command table entry — the actual table is provided in C (kdb_cli.c).
#[repr(C)] #[derive(Copy, Clone)]
pub struct _KDB_DEBUGGER_COMMAND {
    pub Name: PCHAR,
    pub Syntax: PCHAR,
    pub Help: PCHAR,
    pub Fn: Option<unsafe extern "C" fn(Argc: ULONG, Argv: *mut PCHAR) -> BOOLEAN>,
}

// `KdbDebuggerCommands` and `ComponentTable` (and the corresponding
// `_Count` constants) used to be declared here as `extern "C" [T; 0]`
// pointing at C definitions in `kdbg_rust_shims.c`. The zero-length decl
// made any indexing fold to a panic. They now live in
// `super::tables::{KdbDebuggerCommands, ComponentTable}` as properly-sized
// Rust statics; consumers use `.len()` instead of the count constants.
pub use super::tables::{ComponentTable, KdbDebuggerCommands};

// CSTRING-based prompt string descriptor (opaque to Rust here).
extern "C" {
    pub static KdbPromptStr: CSTRING;
    pub static mut KdbCliCallbacks: [PVOID; 10];
    pub static mut KdbUseIntelSyntax: BOOLEAN;
    pub static mut KdbBreakOnModuleLoad: BOOLEAN;
    pub static mut KdbNumberOfRowsPrinted: ULONG;
    pub static mut KdbNumberOfColsPrinted: ULONG;
    pub static mut KdbOutputAborted: BOOLEAN;
    pub static mut KdbRepeatLastCommand: BOOLEAN;
    pub static mut KdpDmesgBuffer: PCHAR;
    pub static KdpDmesgBufferSize: ULONG;
    pub static mut KdpDmesgCurrentPosition: ULONG;
    pub static mut KdpDmesgFreeBytes: ULONG;
    pub static mut KdbDmesgTotalWritten: ULONG;
    pub static mut KdbpIsInDmesgMode: BOOLEAN;
    pub static mut KdpDmesgLogSpinLock: KSPIN_LOCK;
    pub static mut KdbInitFileBuffer: PCHAR;
    pub static mut KdbpBugCheckRequested: BOOLEAN;
    pub static mut KdbCurrentTrapFrame: PKDB_KTRAP_FRAME;
    pub static mut KdbCurrentProcess: PEPROCESS;
    pub static mut KdbCurrentThread: PETHREAD;
    pub static mut KdbLastBreakPointNr: LONG;
    pub static mut KdbNumSingleSteps: ULONG;
    pub static mut KdbSingleStepOver: BOOLEAN;
    pub static mut KdTermSize: SIZE;
    // Linker-provided image base
    pub static __ImageBase: u8;
}

// Cross-file Rust functions defined in other kdbg/* translations or C shims.
//
// kdb.h declares all the kdb-private APIs (Kdb*, Kdbp*) as cdecl — there's
// no NTAPI on them. KdSystemDebugControl is NTAPI per ntoskrnl exports.
// RtlStringCbCopyA is NTAPI from strsafe.h.
extern "C" {
    fn KdbpRpnEvaluateExpression(
        Expression: *mut u8,
        TrapFrame: PKDB_KTRAP_FRAME,
        Result: PULONGLONG,
        ErrOffset: *mut LONG,
        ErrMsg: *mut u8,
    ) -> BOOLEAN;
    fn KdbSymPrintAddress(Address: PVOID, Context: PCONTEXT) -> BOOLEAN;
    fn KdbpSymFindModule(
        Address: PVOID,
        Index: INT,
        pLdrEntry: *mut PLDR_DATA_TABLE_ENTRY,
    ) -> BOOLEAN;
    fn KdbpCommandHistoryAppend(Command: PCSTR);
    fn KdpInitTerminal();
    fn KdpUpdateTerminalSize(Size: *mut SIZE);
    fn KdpReadTermKey(ScanCode: *mut ULONG) -> CHAR;
}
extern "system" {
    // KeGetTrapFramePc is NTAPI (stdcall on i386). Was declared in the
    // preceding extern "C" block until 2026-05-08 and only worked because
    // ke_rust_shims.c supplied a cdecl alias trampoline; promotion of the
    // bare-name `KeGetTrapFramePc` to a stdcall Rust impl in ke-macros
    // surfaced the mismatch.
    fn KeGetTrapFramePc(TrapFrame: *mut core::ffi::c_void) -> ULONG_PTR;
    // KdbpAcquireLock/KdbpReleaseLock are NTAPI (stdcall on i386) per ntoskrnl/kd/kd.h
    // and defined `#[no_mangle] extern "system"` in ntoskrnl-rust kd/kdio.rs. They were
    // in the preceding extern "C" (cdecl) block and only linked while kdb_cli.rs shared
    // ntoskrnl-rust's crate (rustc resolved the call to the local def, ignoring the
    // wrong-ABI decl). The crate-split severed that; declare them stdcall here.
    fn KdbpAcquireLock(SpinLock: PKSPIN_LOCK) -> KIRQL;
    fn KdbpReleaseLock(SpinLock: PKSPIN_LOCK, OldIrql: KIRQL);
    fn KdSystemDebugControl(
        Command: ULONG,
        InputBuffer: PVOID,
        InputBufferLength: ULONG,
        OutputBuffer: PVOID,
        OutputBufferLength: ULONG,
        ReturnLength: *mut ULONG,
        PreviousMode: i32,
    ) -> NTSTATUS;
}
extern "C" {
    /// Defined in shim_exports.rs as `extern "C"` (cdecl).
    fn RtlStringCbCopyA(pszDest: PSTR, cbDest: usize, pszSrc: PCSTR) -> NTSTATUS;
}
// SysDbgReadControlSpace constant (sysdbgmsg.h).
const SysDbgReadControlSpace: ULONG = 12; // header value; the hand copy said 1

// AMD64 constant — only used when KdbpCmdRegs runs on amd64.
#[cfg(target_arch = "x86_64")]
const AMD64_DEBUG_CONTROL_SPACE_KSPECIAL: ULONG_PTR = 0x2;

// _FIRMWARE_REENTRY value used by HalReturnToFirmware (HAL_REBOOT_ROUTINE).
#[allow(non_upper_case_globals)]
const _FIRMWARE_REENTRY_HalRebootRoutine: u32 = 3; // header value; the hand copy said 1

// RTL_NUMBER_OF as macro (function form not allowed for arrays of differing N).
#[allow(unused_macros)]
macro_rules! RTL_NUMBER_OF {
    ($a:expr) => { (core::mem::size_of_val(&$a) / core::mem::size_of_val(&$a[0])) }
}

// Helper to access KTHREAD.State across architectures.
// On i386 KTHREAD has a direct `State: UCHAR` field; on amd64 the field is
// behind a bindgen anonymous union/struct.
#[inline]
unsafe fn kthread_state(tcb: *const ntkrnl_sys::ddk::_KTHREAD) -> u8 {
    #[cfg(target_arch = "x86")]
    { (*tcb).State }
    #[cfg(not(target_arch = "x86"))]
    {
        // On amd64, State sits inside __bindgen_anon_3 (or similar) — read by offset.
        // The dispatcher header has Type at offset 0, so a "state" read is best-effort.
        let _ = tcb;
        0u8
    }
}

// Marker types so we can put `*const i8` / `*mut i8` constants in `static`s.
// These tables are read-only ASCII pointer arrays — KDB never writes them
// across threads. The kernel debugger runs on one CPU at a time anyway.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PCStrSync(pub PCSTR);
unsafe impl Sync for PCStrSync {}
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PCharSync(pub PCHAR);
unsafe impl Sync for PCharSync {}

extern "C" {
    fn strtoul(nptr: *const i8, endptr: *mut *mut i8, base: i32) -> u32;
    fn strtoull(nptr: *const i8, endptr: *mut *mut i8, base: i32) -> u64;
    fn _stricmp(s1: *const i8, s2: *const i8) -> i32;
    fn KdbpSafeReadMemory(Destination: PVOID, Source: PVOID, Size: usize) -> NTSTATUS;
    fn Ke386GetGlobalDescriptorTable(Limit: PUSHORT);
    fn strrchr(s: *const i8, c: i32) -> *mut i8;
    fn strlen(s: *const i8) -> usize;
    fn isalpha(c: i32) -> i32;
    fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
    // `min` was a C macro in kdb_cli.c; replace with `core::cmp::min`.
    fn strcmp(s1: *const i8, s2: *const i8) -> i32;
    fn strcspn(s: *const i8, reject: *const i8) -> usize;
    fn _vsnprintf(buffer: *mut i8, count: usize, format: *const i8, ap: va_list) -> i32;
    // va_start/va_end live in C compiler intrinsics; KdbpPrint definition
    // lives in a C shim (kdbg_rust_shims.c), we never call them from Rust.
    fn isspace(c: i32) -> i32;
    fn strncmp(s1: *const u8, s2: *const u8, n: usize) -> i32;
    fn _strnicmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
    // KDB_ACCESS_TYPE_TO_STRING / NPX_STATE_TO_STRING / KDB_ENTER_CONDITION_TO_STRING
    // moved to super::tables (translated 2026-05-07); imported below.
    fn KdbpGetNextBreakPointNr(StartNr: ULONG) -> LONG;
    fn sprintf(s: *mut i8, format: *const i8, ...) -> i32;
    fn KdbpEnableBreakPoint(BreakPointNr: ULONG, CanBeChanged: *mut BOOLEAN) -> BOOLEAN;
    fn KdbpDisableBreakPoint(BreakPointNr: ULONG, CanBeChanged: *mut BOOLEAN) -> BOOLEAN;
    fn KdbpDeleteBreakPoint(BreakPointNr: ULONG, BreakPoint: PKDB_BREAKPOINT) -> BOOLEAN;
    fn KdbpAttachToThread(ThreadId: PVOID) -> BOOLEAN;
    fn KeGetTrapFrameFrameRegister(TrapFrame: PVOID) -> ULONG_PTR;
    fn KeGetTrapFrameStackRegister(TrapFrame: PVOID) -> ULONG_PTR;
    // NPX_STATE_TO_STRING — moved to super::tables.
    static mut PsActiveProcessHead: LIST_ENTRY;
    fn KdbpAttachToProcess(ul: PVOID) -> BOOLEAN;
    fn Ke386GetTr() -> USHORT;
    // KDB_ENTER_CONDITION_TO_STRING — moved to super::tables.
    fn KdbpGetEnterCondition(exc: LONG, first: BOOLEAN, cond: *mut KDB_ENTER_CONDITION) -> BOOLEAN;
    fn KdbpSetEnterCondition(exc: LONG, first: BOOLEAN, cond: KDB_ENTER_CONDITION) -> BOOLEAN;
    fn KdbpDisassemble(Address: ULONG_PTR, IntelSyntax: ULONG) -> LONG;
    fn KeGetContextFrameRegister(ctx: PCONTEXT) -> ULONG_PTR;
    fn KeSetContextFrameRegister(ctx: PCONTEXT, frame: ULONG_PTR);

    // KdbpPrint and KdbPrintf are C variadics — defined in kdbg_rust_shims.c.
    fn KdbPuts(s: PCSTR);
    fn KdbpInsertBreakPoint(
        Address: ULONG_PTR,
        Type: KDB_BREAKPOINT_TYPE,
        Size: UCHAR,
        AccessType: KDB_ACCESS_TYPE,
        ConditionExpression: PCHAR,
        Global: BOOLEAN,
        BreakPointNr: PLONG,
    ) -> NTSTATUS;
    fn KdbpGetBreakPointInfo(
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
    ) -> BOOLEAN;
    // i386 helpers from amd64-only/i386-only paths.
    #[cfg(target_arch = "x86")]
    fn __sidt(p: PVOID);

    // KeGetPcr — defined in C ke macros / shim; returns PCR pointer.
    fn KeGetPcr() -> PKIPCR;
    // CLI helpers from kdb.c — kdb.h declares these without NTAPI, so cdecl.
    fn KdbpGetCommandLineSettings(p: PCSTR);
    fn KdbSymInit(BootPhase: ULONG) -> BOOLEAN;
    fn KdbPrompt(prompt: PCSTR, buffer: PCHAR, size: usize) -> usize;
}

extern "system" {
    /// `DbgBreakPointWithStatus` is NTAPI/stdcall (sdk/lib/rtl/i386/debug_asm.S
    /// exports `_DbgBreakPointWithStatus@4`).
    fn DbgBreakPointWithStatus(status: ULONG);
}

extern "C" {
    pub static mut KdProviders: LIST_ENTRY;
    pub static mut KdTermConnected: BOOLEAN;
}

// Constants pulled from C headers (subset used here).
const FILE_NON_DIRECTORY_FILE: ULONG = 0x00000040;
const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x00000020;
const FILE_NO_INTERMEDIATE_BUFFERING: ULONG = 0x00000008;
const FileStandardInformation: ULONG = 5;
const DBG_STATUS_CONTROL_C: ULONG = 1;

// Terminal scan codes (kd_term private)
const KEYSC_END: u32 = 0x4F00;
const KEYSC_PAGEUP: u32 = 0x4900;
const KEYSC_ARROWUP: u32 = 0x4800;
const KEYSC_HOME: u32 = 0x4700;
extern "system" {
    fn RtlLookupFunctionEntry( ControlPc: DWORD64, ImageBase: *mut DWORD64, HistoryTable: PVOID ) -> PRUNTIME_FUNCTION;
    fn RtlVirtualUnwind( HandlerType: DWORD, ImageBase: DWORD64, ControlPc: DWORD64, FunctionEntry: PRUNTIME_FUNCTION, ContextRecord: PCONTEXT, HandlerData: *mut PVOID, EstablisherFrame: *mut ULONG64, ContextPointers: PVOID ) -> PVOID;
}
// RosSymAggregate / RosSymFreeAggregate live in sdk/lib/rossym_new and are
// only linked into ntoskrnl when built with __ROS_DWARF__ — same gate as
// KdbpCmdPrintStruct above.
#[cfg(feature = "ros_dwarf")]
extern "C" {
    fn KdbpSymFindCachedFile(ModName: *mut _UNICODE_STRING) -> PROSSYM_INFO;
    fn RosSymAggregate(Info: PROSSYM_INFO, Name: PCHAR, Aggregate: PROSSYM_AGGREGATE) -> BOOLEAN;
    fn RosSymFreeAggregate(Aggregate: PROSSYM_AGGREGATE);
}
extern "C" {
    /// `Ke386GetLocalDescriptorTable` is exported by kdbg_rust_shims.c as cdecl.
    fn Ke386GetLocalDescriptorTable(Limit: *mut USHORT);
    /// KeGetContextPc / KeSetContextPc are cdecl (no NTAPI in their declarations
    /// in `internal/i386/ke.h`). Out-of-line definitions live in
    /// shim_exports.rs (KeGetContextPc) and elsewhere.
    fn KeGetContextPc(Context: PCONTEXT) -> ULONG_PTR;
    fn KeSetContextPc(Context: PCONTEXT, Pc: ULONG_PTR);
}

#[inline(always)]

unsafe fn strtoulptr(nptr: *const i8, endptr: *mut *mut i8, base: i32) -> usize {
    
    #[cfg(target_arch = "x86")]
    {
        strtoul(nptr, endptr, base) as usize
    }
    #[cfg(not(target_arch = "x86"))]
    {
        strtoull(nptr, endptr, base) as usize
    }
}

unsafe fn KdbpEvaluateExpression(
    Expression: PCHAR,
    ErrOffset: LONG,
    Result: PULONGLONG,
) -> BOOLEAN {
    static mut ErrMsgBuffer: [CHAR; 130] = {
        let mut buf: [CHAR; 130] = [0; 130];
        buf[0] = b'^' as CHAR;
        buf[1] = b' ' as CHAR;
        buf
    };
    let mut ExpressionErrOffset: LONG = -1;
    let mut ErrMsg: PCHAR = (*(&raw mut ErrMsgBuffer)).as_mut_ptr();
    let IsOk: BOOLEAN;

    IsOk = KdbpRpnEvaluateExpression(
        Expression as *mut u8,
        KdbCurrentTrapFrame,
        Result,
        &mut ExpressionErrOffset,
        (*(&raw mut ErrMsgBuffer)).as_mut_ptr().offset(2) as *mut u8,
    );
    if IsOk == 0 {
        if ExpressionErrOffset >= 0 {
            ExpressionErrOffset += ErrOffset;
        } else {
            ErrMsg = ErrMsg.offset(2);
        }

        crate::kdbp_print!(b"%*s%s\n\0", ExpressionErrOffset, b"\0".as_ptr() as PSTR, ErrMsg);
    }

    IsOk
}

#[no_mangle]
pub unsafe extern "system" fn KdbpGetHexNumber(
    pszNum: PCHAR,
    pulValue: *mut ULONG_PTR,
) -> BOOLEAN {

    let mut pszNum = pszNum;
    let mut endptr: *mut i8 = core::ptr::null_mut();

    if (*pszNum.offset(0) == b'0' as i8) && 
       ((*pszNum.offset(1) == b'x' as i8) || (*pszNum.offset(1) == b'X' as i8)) {
        pszNum = pszNum.offset(2);
    }

    *pulValue = strtoul(pszNum, &mut endptr, 16) as ULONG_PTR;

    if *endptr == 0 { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdEvalExpression(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {

    let mut i: ULONG;
    let mut len: usize;
    let mut Result: ULONGLONG = 0;
    let mut ul: ULONG;
    let mut l: LONG = 0;
    let IsOk: BOOLEAN;

    if Argc < 2 {
        crate::kdbp_print!(b"?: Argument required\n\0");
        return 1u8;
    }

    let mut Argc = Argc;
    Argc = Argc - 1;
    i = 1;
    while i < Argc {
        len = strlen(*Argv.add(i as usize) as *const i8);
        *(*Argv.add(i as usize)).add(len) = b' ' as CHAR;
        i = i + 1;
    }

    IsOk = KdbpEvaluateExpression(
        *Argv.add(1),
        KdbPromptStr.Length as LONG + (*Argv.add(1) as isize - *Argv.add(0) as isize) as LONG,
        &mut Result,
    );
    if IsOk != 0 {
        if Result > 0x00000000ffffffffu64 {
            if (Result & 0x8000000000000000u64) != 0 {
                crate::kdbp_print!(b"0x%016I64x  %20I64u  %20I64d\n\0", Result, Result, Result);
            } else {
                crate::kdbp_print!(b"0x%016I64x  %20I64u\n\0", Result, Result);
            }
        } else {
            ul = Result as ULONG;

            if ul <= 0xff && ul >= 0x80 {
                l = ul as CHAR as LONG;
            } else if ul <= 0xffff && ul >= 0x8000 {
                l = ul as i16 as LONG;
            } else {
                l = ul as LONG;
            }

            if l < 0 {
                crate::kdbp_print!(b"0x%08lx  %10lu  %10ld\n\0", ul, ul, l);
            } else {
                crate::kdbp_print!(b"0x%08lx  %10lu\n\0", ul, ul);
            }
        }
    }

    1u8
}

// KdbpPrintStructInternal + KdbpCmdPrintStruct are only useful when ntoskrnl
// is built with rossym_new for DWARF symbol info (`__ROS_DWARF__`). The
// stock build uses rossym (no Aggregate APIs), so RosSymAggregate /
// RosSymFreeAggregate would otherwise be unresolved at link time. Mirror the
// C `#ifdef __ROS_DWARF__` guard with a Rust feature gate; the C shim
// command table conditionally includes "dt" only under __ROS_DWARF__ too.
#[cfg(feature = "ros_dwarf")]
unsafe fn KdbpPrintStructInternal(
    Info: PROSSYM_INFO,
    Indent: PCHAR,
    DoRead: BOOLEAN,
    BaseAddress: PVOID,
    Aggregate: PROSSYM_AGGREGATE,
) {
    let mut i: ULONG;
    let mut Result: ULONGLONG;
    let mut Member: PROSSYM_AGGREGATE_MEMBER;
    let IndentLen: ULONG = strlen(Indent as *const i8) as ULONG;
    let mut MemberAggregate: ROSSYM_AGGREGATE = core::mem::zeroed();

    i = 0;
    while i < (*Aggregate).NumElements {
        Member = (*Aggregate).Elements.add(i as usize);
        crate::kdbp_print!(b"%s%p+%x: %s\0",
            Indent,
            (BaseAddress as PCHAR).offset((*Member).BaseOffset as isize),
            (*Member).Size,
            if !(*Member).Name.is_null() {
                (*Member).Name
            } else {
                b"<anoymous>\0".as_ptr() as PCHAR
            },
        );
        if DoRead != 0 {
            if strcmp((*Member).Type as *const i8, b"_UNICODE_STRING\0".as_ptr() as *const i8) == 0 {
                crate::kdbp_print!(b"\"\0");
                KdbpPrintUnicodeString((BaseAddress as PCHAR).offset((*Member).BaseOffset as isize) as PCUNICODE_STRING);
                crate::kdbp_print!(b"\"\n\0");
                i += 1;
                continue;
            } else if strcmp((*Member).Type as *const i8, b"PUNICODE_STRING\0".as_ptr() as *const i8) == 0 {
                crate::kdbp_print!(b"\"\0");
                KdbpPrintUnicodeString(
                    *(((BaseAddress as PCHAR).offset((*Member).BaseOffset as isize)) as *mut PUNICODE_STRING),
                );
                crate::kdbp_print!(b"\"\n\0");
                i += 1;
                continue;
            }
            match (*Member).Size {
                1 | 2 | 4 | 8 => {
                    Result = 0;
                    if NT_SUCCESS(KdbpSafeReadMemory(
                        core::ptr::addr_of_mut!(Result) as PVOID,
                        (BaseAddress as PCHAR).offset((*Member).BaseOffset as isize) as PVOID,
                        (*Member).Size as usize,
                    )) {
                        if (*Member).Bits != 0 {
                            Result >>= (*Member).FirstBit;
                            Result &= ((1u64 << (*Member).Bits) - 1);
                        }
                        crate::kdbp_print!(b" %lx\n\0", Result);
                    } else {
                        if (*Member).Size <= 8 {
                            crate::kdbp_print!(b" ??\n\0");
                        } else {
                            crate::kdbp_print!(b" %s @ %x {\n\0",
                                (*Member).Type,
                                (*Member).BaseOffset,
                            );
                            *Indent.offset(IndentLen as isize) = b' ' as CHAR;
                            if RosSymAggregate(Info, (*Member).Type, core::ptr::addr_of_mut!(MemberAggregate)) != 0 {
                                KdbpPrintStructInternal(
                                    Info,
                                    Indent,
                                    DoRead,
                                    BaseAddress,
                                    core::ptr::addr_of_mut!(MemberAggregate),
                                );
                                RosSymFreeAggregate(core::ptr::addr_of_mut!(MemberAggregate));
                            }
                            *Indent.offset(IndentLen as isize) = 0;
                            crate::kdbp_print!(b"%s}\n\0", Indent);
                        }
                        i += 1;
                        continue;
                    }
                }
                _ => {
                    if (*Member).Size < 8 {
                        Result = 0;
                        if NT_SUCCESS(KdbpSafeReadMemory(
                            core::ptr::addr_of_mut!(Result) as PVOID,
                            (BaseAddress as PCHAR).offset((*Member).BaseOffset as isize) as PVOID,
                            (*Member).Size as usize,
                        )) {
                            let mut j: ULONG = 0;
                            while j < (*Member).Size {
                                crate::kdbp_print!(b" %02x\0", (Result & 0xff) as INT);
                                Result >>= 8;
                                j += 1;
                            }
                        } else {
                            if (*Member).Size <= 8 {
                                crate::kdbp_print!(b" ??\n\0");
                            } else {
                                crate::kdbp_print!(b" %s @ %x {\n\0",
                                    (*Member).Type,
                                    (*Member).BaseOffset,
                                );
                                *Indent.offset(IndentLen as isize) = b' ' as CHAR;
                                if RosSymAggregate(Info, (*Member).Type, core::ptr::addr_of_mut!(MemberAggregate)) != 0 {
                                    KdbpPrintStructInternal(
                                        Info,
                                        Indent,
                                        DoRead,
                                        BaseAddress,
                                        core::ptr::addr_of_mut!(MemberAggregate),
                                    );
                                    RosSymFreeAggregate(core::ptr::addr_of_mut!(MemberAggregate));
                                }
                                *Indent.offset(IndentLen as isize) = 0;
                                crate::kdbp_print!(b"%s}\n\0", Indent);
                            }
                            i += 1;
                            continue;
                        }
                    } else {
                        crate::kdbp_print!(b" %s @ %p {\n\0",
                            (*Member).Type,
                            (BaseAddress as PCHAR).offset((*Member).BaseOffset as isize),
                        );
                        *Indent.offset(IndentLen as isize) = b' ' as CHAR;
                        if RosSymAggregate(Info, (*Member).Type, core::ptr::addr_of_mut!(MemberAggregate)) != 0 {
                            KdbpPrintStructInternal(
                                Info,
                                Indent,
                                DoRead,
                                (BaseAddress as PCHAR).offset((*Member).BaseOffset as isize) as PVOID,
                                core::ptr::addr_of_mut!(MemberAggregate),
                            );
                            RosSymFreeAggregate(core::ptr::addr_of_mut!(MemberAggregate));
                        }
                        *Indent.offset(IndentLen as isize) = 0;
                        crate::kdbp_print!(b"%s}\n\0", Indent);
                    }
                }
            }
        } else {
            if (*Member).Size <= 8 {
                crate::kdbp_print!(b" ??\n\0");
            } else {
                crate::kdbp_print!(b" %s @ %x {\n\0",
                    (*Member).Type,
                    (*Member).BaseOffset,
                );
                *Indent.offset(IndentLen as isize) = b' ' as CHAR;
                if RosSymAggregate(Info, (*Member).Type, core::ptr::addr_of_mut!(MemberAggregate)) != 0 {
                    KdbpPrintStructInternal(
                        Info,
                        Indent,
                        DoRead,
                        BaseAddress,
                        core::ptr::addr_of_mut!(MemberAggregate),
                    );
                    RosSymFreeAggregate(core::ptr::addr_of_mut!(MemberAggregate));
                }
                *Indent.offset(IndentLen as isize) = 0;
                crate::kdbp_print!(b"%s}\n\0", Indent);
            }
        }
        i += 1;
    }
}

#[cfg(not(feature = "ros_dwarf"))]
#[no_mangle]
pub unsafe extern "C" fn KdbpCmdPrintStruct(
    _Argc: ULONG,
    _Argv: *mut PCHAR,
) -> BOOLEAN {
    crate::kdbp_print!(b"Command 'dt' requires building ntoskrnl with __ROS_DWARF__.\n\0");
    1u8
}

#[cfg(feature = "ros_dwarf")]
#[no_mangle]
pub unsafe extern "C" fn KdbpCmdPrintStruct(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {
    let mut i: ULONG;
    let mut Result: ULONGLONG = 0;
    let mut BaseAddress: PVOID = core::ptr::null_mut();
    let mut Aggregate: ROSSYM_AGGREGATE = core::mem::zeroed();
    let mut ModName: _UNICODE_STRING = core::mem::zeroed();
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    let mut Indent: [CHAR; 100] = [0; 100];
    let Info: PROSSYM_INFO;

    'end: loop {
        if Argc < 3 {
            break 'end;
        }
        AnsiName.Length = strlen(*Argv.offset(1) as *const i8) as u16;
        AnsiName.MaximumLength = AnsiName.Length;
        AnsiName.Buffer = *Argv.offset(1);
        RtlAnsiStringToUnicodeString(&mut ModName, &mut AnsiName, 1u8);
        Info = KdbpSymFindCachedFile(&mut ModName);

        if Info.is_null() || RosSymAggregate(Info, *Argv.offset(2), &mut Aggregate) == 0 {
            break 'end;
        }

        if Argc > 3 {
            let len: usize;
            let ArgStart: PCHAR = *Argv.offset(3);
            i = 3;
            while i < Argc - 1 {
                let len = strlen(*Argv.offset(i as isize) as *const i8);
                *(*Argv.offset(i as isize)).offset(len as isize) = b' ' as CHAR;
                i += 1;
            }

            if KdbpEvaluateExpression(ArgStart, strlen(ArgStart as *const i8) as LONG, &mut Result) != 0 {
                BaseAddress = Result as ULONG_PTR as PVOID;
            }
        }
        KdbpPrintStructInternal(Info, Indent.as_mut_ptr(), if !BaseAddress.is_null() { 1u8 } else { 0u8 }, BaseAddress, &mut Aggregate);
        break 'end;
    }

    RosSymFreeAggregate(&mut Aggregate);
    RtlFreeUnicodeString(&mut ModName);
    1u8
}

unsafe fn KdbpGetComponentId(
    ComponentName: PCSTR,
    ComponentId: *mut u32,
) -> u8 {

    let mut i: u32 = 0;
    while (i as usize) < (*(&raw const ComponentTable)).len() {
        if _stricmp(ComponentName as *const i8, ComponentTable[i as usize].Name as *const i8) == 0 {
            *ComponentId = ComponentTable[i as usize].Id;
            return 1u8;
        }
        i += 1;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdFilter(
    Argc: u32,
    Argv: *mut PCHAR,
) -> u8 {

    struct DebugClass {
        Name: PCSTR,
        Level: u32,
    }

    let debug_classes: [DebugClass; 4] = [
        DebugClass { Name: b"error\0".as_ptr() as PCSTR, Level: 1 << 0 },
        DebugClass { Name: b"warning\0".as_ptr() as PCSTR, Level: 1 << 1 },
        DebugClass { Name: b"trace\0".as_ptr() as PCSTR, Level: 1 << 2 },
        DebugClass { Name: b"info\0".as_ptr() as PCSTR, Level: 1 << 3 },
    ];

    let mut i: u32;
    let mut j: u32;
    let mut ComponentId: u32 = 0;
    let mut Level: u32;
    let mut set: u32 = 0xFFFFFFFF;
    let mut clear: u32 = 0xFFFFFFFF;
    let mut pend: *mut i8 = core::ptr::null_mut();
    let mut opt: PCSTR;
    let mut p: PCSTR;

    if Argc <= 1 {
        crate::kdbp_print!(b"REMARKS:\n- The 'WIN2000' system-wide debug filter component is used for DbgPrint()\n  messages without Component ID and Level.\n- The 'DEFAULT' debug filter component is used for DbgPrint() messages with\n  an unknown Component ID.\n\n\0",
        );
        crate::kdbp_print!(b"The list of debug filter components currently available on your system is:\n\n\0",
        );
        crate::kdbp_print!(b"    Component Name         Component ID\n  ==================     ================\n\0",
        );
        i = 0;
        while (i as usize) < (*(&raw const ComponentTable)).len() {
            crate::kdbp_print!(b"%20s        0x%08lx\n\0",
                ComponentTable[i as usize].Name,
                ComponentTable[i as usize].Id,
            );
            i += 1;
        }
        return 1u8;
    }

    i = 1;
    while i < Argc {
        opt = *Argv.add(i as usize);
        p = opt.add(strcspn(opt as *const i8, b"+-\0".as_ptr() as *const i8));
        if *(p as *const u8) == 0 {
            p = opt;
        }

        if p > opt {
            j = 0;
            while j < debug_classes.len() as u32 {
                let len = strlen(debug_classes[j as usize].Name as *const i8);
                if len != (p as usize - opt as usize) {
                    j += 1;
                    continue;
                }
                if _strnicmp(opt as *const i8, debug_classes[j as usize].Name as *const i8, len) == 0 {
                    if *(p as *const u8) == b'+' {
                        set |= debug_classes[j as usize].Level;
                    } else {
                        clear |= debug_classes[j as usize].Level;
                    }
                    break;
                }
                j += 1;
            }
            if j == debug_classes.len() as u32 {
                Level = strtoul(opt as *const i8, &mut pend, 0);
                if pend != p as *mut i8 {
                    crate::kdbp_print!(b"filter: bad class name '%.*s'\n\0",
                        (p as usize - opt as usize) as i32,
                        opt,
                    );
                    i += 1;
                    continue;
                }
                if *(p as *const u8) == b'+' {
                    set |= Level;
                } else {
                    clear |= Level;
                }
            }
        } else {
            if *(p as *const u8) == b'-' {
                clear = 0xFFFFFFFF;
            } else {
                set = 0xFFFFFFFF;
            }
        }
        if *(p as *const u8) == b'+' || *(p as *const u8) == b'-' {
            p = p.add(1);
        }

        if KdbpGetComponentId(p, &mut ComponentId) == 0 {
            crate::kdbp_print!(b"filter: '%s' is not a valid component name!\n\0",
                p,
            );
            return 1u8;
        }

        NtSetDebugFilterState(ComponentId, set, 1u8);
        NtSetDebugFilterState(ComponentId, clear, 0u8);

        i += 1;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdDisassembleX(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {

    let mut Count: ULONG;
    let mut ul: ULONG;
    let mut i: INT;
    let mut Result: ULONGLONG = 0;
    let mut Address: ULONG_PTR = KeGetContextPc(KdbCurrentTrapFrame);
    let InstLen: LONG;
    let mut Argc = Argc;

    if *(*Argv.offset(0)).offset(0) == b'x' as i8 {
        Count = 16;
    } else {
        Count = 10;
    }

    if Argc >= 2 {
        ul = 0;
        if strcmp(*Argv.offset((Argc - 2) as isize), b"L\0".as_ptr() as *const i8) == 0 {
            ul = strtoul(*Argv.offset((Argc - 1) as isize), core::ptr::null_mut(), 0);
            if ul > 0 {
                Count = ul;
                Argc -= 2;
            }
        } else if *(*Argv.offset((Argc - 1) as isize)).offset(0) == b'L' as i8 {
            ul = strtoul((*Argv.offset((Argc - 1) as isize)).offset(1), core::ptr::null_mut(), 0);
            if ul > 0 {
                Count = ul;
                Argc -= 1;
            }
        }

        Argc -= 1;
        ul = 1;
        while ul < Argc {
            let len = strlen(*Argv.offset(ul as isize));
            *(*Argv.offset(ul as isize)).offset(len as isize) = b' ' as i8;
            ul += 1;
        }
        Argc += 1;
    }

    if Argc > 1 {
        if KdbpEvaluateExpression(
            *Argv.offset(1),
            (KdbPromptStr.Length as isize + ((*Argv.offset(1) as isize) - (*Argv.offset(0) as isize))) as LONG,
            &mut Result,
        ) == 0
        {
            return 1u8;
        }

        if Result > (!0usize) as ULONGLONG {
            crate::kdbp_print!(b"Warning: Address %I64x is beeing truncated\n\0", Result);
        }

        Address = Result as ULONG_PTR;
    } else if *(*Argv.offset(0)).offset(0) == b'x' as i8 {
        crate::kdbp_print!(b"x: Address argument required.\n\0");
        return 1u8;
    }

    if *(*Argv.offset(0)).offset(0) == b'x' as i8 {
        ul = 0;

        while Count > 0 {
            if KdbSymPrintAddress(Address as PVOID, core::ptr::null_mut()) == 0 {
                crate::kdbp_print!(b"<%p>:\0", Address as PVOID);
            } else {
                crate::kdbp_print!(b":\0");
            }

            i = core::cmp::min(4u32, Count) as INT;
            Count -= i as ULONG;

            loop {
                i -= 1;
                if i < 0 {
                    break;
                }

                if !NT_SUCCESS(KdbpSafeReadMemory(
                    &mut ul as *mut ULONG as PVOID,
                    Address as PVOID,
                    core::mem::size_of::<ULONG>(),
                )) {
                    crate::kdbp_print!(b" ????????\0");
                } else {
                    crate::kdbp_print!(b" %08x\0", ul);
                }

                Address += core::mem::size_of::<ULONG>() as ULONG_PTR;
            }

            crate::kdbp_print!(b"\n\0");
        }
    } else {
        loop {
            let prev_count = Count;
            Count = Count.wrapping_sub(1);
            if prev_count == 0 {
                break;
            }

            if KdbSymPrintAddress(Address as PVOID, core::ptr::null_mut()) == 0 {
                crate::kdbp_print!(b"<%08x>: \0", Address);
            } else {
                crate::kdbp_print!(b": \0");
            }

            let InstLen = KdbpDisassemble(Address, KdbUseIntelSyntax as ULONG);
            if InstLen < 0 {
                crate::kdbp_print!(b"<INVALID>\n\0");
                return 1u8;
            }

            crate::kdbp_print!(b"\n\0");
            Address += InstLen as ULONG_PTR;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdRegs(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {
    let Context: PCONTEXT = KdbCurrentTrapFrame;
    let mut i: INT;
    let EflagsBits: [PCHAR; 32] = [
        b" CF\0".as_ptr() as PCHAR, core::ptr::null_mut(), b" PF\0".as_ptr() as PCHAR, b" BIT3\0".as_ptr() as PCHAR, b" AF\0".as_ptr() as PCHAR, b" BIT5\0".as_ptr() as PCHAR,
        b" ZF\0".as_ptr() as PCHAR, b" SF\0".as_ptr() as PCHAR, b" TF\0".as_ptr() as PCHAR, b" IF\0".as_ptr() as PCHAR, b" DF\0".as_ptr() as PCHAR, b" OF\0".as_ptr() as PCHAR,
        core::ptr::null_mut(), core::ptr::null_mut(), b" NT\0".as_ptr() as PCHAR, b" BIT15\0".as_ptr() as PCHAR, b" RF\0".as_ptr() as PCHAR, b" VF\0".as_ptr() as PCHAR,
        b" AC\0".as_ptr() as PCHAR, b" VIF\0".as_ptr() as PCHAR, b" VIP\0".as_ptr() as PCHAR, b" ID\0".as_ptr() as PCHAR, b" BIT22\0".as_ptr() as PCHAR,
        b" BIT23\0".as_ptr() as PCHAR, b" BIT24\0".as_ptr() as PCHAR, b" BIT25\0".as_ptr() as PCHAR, b" BIT26\0".as_ptr() as PCHAR,
        b" BIT27\0".as_ptr() as PCHAR, b" BIT28\0".as_ptr() as PCHAR, b" BIT29\0".as_ptr() as PCHAR, b" BIT30\0".as_ptr() as PCHAR,
        b" BIT31\0".as_ptr() as PCHAR,
    ];

    if *(*Argv.offset(0)).offset(0) == b'r' as i8 {
        #[cfg(target_arch = "x86")]
        crate::kdbp_print!(b"CS:EIP  0x%04x:0x%08x\nSS:ESP  0x%04x:0x%08x\n   EAX  0x%08x   EBX  0x%08x\n   ECX  0x%08x   EDX  0x%08x\n   ESI  0x%08x   EDI  0x%08x\n   EBP  0x%08x\n\0",
            (*Context).SegCs & 0xFFFF, (*Context).Eip,
            (*Context).SegSs, (*Context).Esp,
            (*Context).Eax, (*Context).Ebx,
            (*Context).Ecx, (*Context).Edx,
            (*Context).Esi, (*Context).Edi,
            (*Context).Ebp,
        );
        #[cfg(not(target_arch = "x86"))]
        crate::kdbp_print!(b"CS:RIP  0x%04x:0x%p\nSS:RSP  0x%04x:0x%p\n   RAX  0x%p     RBX  0x%p\n   RCX  0x%p     RDX  0x%p\n   RSI  0x%p     RDI  0x%p\n   RBP  0x%p\n\0",
            ((*Context).SegCs & 0xFFFF) as u32, (*Context).Rip,
            ((*Context).SegSs) as u32, (*Context).Rsp,
            (*Context).Rax, (*Context).Rbx,
            (*Context).Rcx, (*Context).Rdx,
            (*Context).Rsi, (*Context).Rdi,
            (*Context).Rbp,
        );

        crate::kdbp_print!(b"EFLAGS  0x%08x \0", (*Context).EFlags);
        i = 0;
        while i < 32 {
            if i == 1 {
                if ((*Context).EFlags & (1 << 1)) == 0 {
                    crate::kdbp_print!(b" !BIT1\0");
                }
            } else if i == 12 {
                crate::kdbp_print!(b" IOPL%d\0", ((*Context).EFlags >> 12) & 3);
            } else if i == 13 {
            } else if ((*Context).EFlags & (1 << i)) != 0 {
                crate::dbgprint::rust_kdbp_print_cstr(EflagsBits[i as usize] as *const u8);
            }
            i += 1;
        }
        crate::kdbp_print!(b"\n\0");
    } else if *(*Argv.offset(0)).offset(0) == b'c' as i8 {
        let mut Cr0: ULONG;
        let mut Cr2: ULONG;
        let mut Cr3: ULONG;
        let mut Cr4: ULONG;
        let mut Gdtr: KDESCRIPTOR = core::mem::zeroed();
        let mut Idtr: KDESCRIPTOR = core::mem::zeroed();
        let mut Ldtr: USHORT;
        let mut Tr: USHORT;
        let Cr0Bits: [PCHAR; 32] = [
            b" PE\0".as_ptr() as PCHAR, b" MP\0".as_ptr() as PCHAR, b" EM\0".as_ptr() as PCHAR, b" TS\0".as_ptr() as PCHAR, b" ET\0".as_ptr() as PCHAR, b" NE\0".as_ptr() as PCHAR, core::ptr::null_mut(), core::ptr::null_mut(),
            core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
            b" WP\0".as_ptr() as PCHAR, core::ptr::null_mut(), b" AM\0".as_ptr() as PCHAR, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
            core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), b" NW\0".as_ptr() as PCHAR, b" CD\0".as_ptr() as PCHAR, b" PG\0".as_ptr() as PCHAR,
        ];
        let Cr4Bits: [PCHAR; 32] = [
            b" VME\0".as_ptr() as PCHAR, b" PVI\0".as_ptr() as PCHAR, b" TSD\0".as_ptr() as PCHAR, b" DE\0".as_ptr() as PCHAR, b" PSE\0".as_ptr() as PCHAR, b" PAE\0".as_ptr() as PCHAR, b" MCE\0".as_ptr() as PCHAR, b" PGE\0".as_ptr() as PCHAR,
            b" PCE\0".as_ptr() as PCHAR, b" OSFXSR\0".as_ptr() as PCHAR, b" OSXMMEXCPT\0".as_ptr() as PCHAR, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
            core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
            core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
        ];
        let mut Input: SYSDBG_CONTROL_SPACE = core::mem::zeroed();
        let mut SpecialRegisters: KSPECIAL_REGISTERS = core::mem::zeroed();
        let mut Status: NTSTATUS = STATUS_NOT_IMPLEMENTED;

        RtlZeroMemory(core::ptr::addr_of_mut!(Input) as PVOID, core::mem::size_of::<SYSDBG_CONTROL_SPACE>());
        Input.Buffer = core::ptr::addr_of_mut!(SpecialRegisters) as PVOID;
        Input.Request = core::mem::size_of::<KSPECIAL_REGISTERS>() as ULONG;
        #[cfg(target_arch = "x86")]
        { Input.Address = core::mem::size_of::<CONTEXT>() as u64; }
        #[cfg(not(target_arch = "x86"))]
        { Input.Address = AMD64_DEBUG_CONTROL_SPACE_KSPECIAL as u64; }
        Status = KdSystemDebugControl(
            SysDbgReadControlSpace,
            core::ptr::addr_of_mut!(Input) as PVOID, core::mem::size_of::<SYSDBG_CONTROL_SPACE>() as ULONG,
            core::ptr::null_mut(), 0,
            core::ptr::null_mut(), 0i32,
        );
        if !NT_SUCCESS(Status) {
            crate::kdbp_print!(b"Failed to get registers: status 0x%08x\n\0", Status);
            return 1u8;
        }
        Cr0 = SpecialRegisters.Cr0 as ULONG;
        Cr2 = SpecialRegisters.Cr2 as ULONG;
        Cr3 = SpecialRegisters.Cr3 as ULONG;
        Cr4 = SpecialRegisters.Cr4 as ULONG;

        // On amd64, SpecialRegisters has different shape; coerce via core::mem::transmute_copy.
        // KDESCRIPTOR layout differs by arch — these variables are i386 KDESCRIPTOR locals.
        Gdtr = core::mem::transmute_copy(&SpecialRegisters.Gdtr);
        Ldtr = SpecialRegisters.Ldtr;
        Idtr = core::mem::transmute_copy(&SpecialRegisters.Idtr);
        Tr = SpecialRegisters.Tr;

        crate::kdbp_print!(b"CR0  0x%08x \0", Cr0);
        i = 0;
        while i < 32 {
            if Cr0Bits[i as usize].is_null() {
                i += 1;
                continue;
            }

            if (Cr0 & (1 << i)) != 0 {
                crate::dbgprint::rust_kdbp_print_cstr(Cr0Bits[i as usize] as *const u8);
            }
            i += 1;
        }
        crate::kdbp_print!(b"\n\0");

        crate::kdbp_print!(b"CR2  0x%08x\n\0", Cr2);
        crate::kdbp_print!(b"CR3  0x%08x  Pagedir-Base 0x%08x %s%s\n\0", Cr3, Cr3 & 0xfffff000 ,
            if (Cr3 & (1 << 3)) != 0 { b" PWT\0".as_ptr() as PSTR } else { b"\0".as_ptr() as PSTR },
            if (Cr3 & (1 << 4)) != 0 { b" PCD\0".as_ptr() as PSTR } else { b"\0".as_ptr() as PSTR }
        );
        crate::kdbp_print!(b"CR4  0x%08x \0", Cr4);
        i = 0;
        while i < 32 {
            if Cr4Bits[i as usize].is_null() {
                i += 1;
                continue;
            }

            if (Cr4 & (1 << i)) != 0 {
                crate::dbgprint::rust_kdbp_print_cstr(Cr4Bits[i as usize] as *const u8);
            }
            i += 1;
        }
        crate::kdbp_print!(b"\n\0");

        crate::kdbp_print!(b"GDTR Base 0x%08x  Size 0x%04x\n\0", Gdtr.Base, Gdtr.Limit as u32);
        crate::kdbp_print!(b"LDTR 0x%04x\n\0", Ldtr as u32);
        crate::kdbp_print!(b"IDTR Base 0x%08x  Size 0x%04x\n\0", Idtr.Base, Idtr.Limit as u32);
        crate::kdbp_print!(b"TR   0x%04x\n\0", Tr as u32);
    } else if *(*Argv.offset(0)).offset(0) == b's' as i8 {
        let seg_cs = (*Context).SegCs as u32;
        let seg_ds = (*Context).SegDs as u32;
        let seg_es = (*Context).SegEs as u32;
        let seg_fs = (*Context).SegFs as u32;
        let seg_gs = (*Context).SegGs as u32;
        let seg_ss = (*Context).SegSs as u32;
        crate::kdbp_print!(b"CS  0x%04x  Index 0x%04x  %cDT RPL%d\n\0",
            seg_cs & 0xffff, (seg_cs & 0xffff) >> 3,
            if (seg_cs & (1 << 2)) != 0 { b'L' as i32 } else { b'G' as i32 }, seg_cs & 3,
        );
        crate::kdbp_print!(b"DS  0x%04x  Index 0x%04x  %cDT RPL%d\n\0",
            seg_ds, seg_ds >> 3, if (seg_ds & (1 << 2)) != 0 { b'L' as i32 } else { b'G' as i32 }, seg_ds & 3,
        );
        crate::kdbp_print!(b"ES  0x%04x  Index 0x%04x  %cDT RPL%d\n\0",
            seg_es, seg_es >> 3, if (seg_es & (1 << 2)) != 0 { b'L' as i32 } else { b'G' as i32 }, seg_es & 3,
        );
        crate::kdbp_print!(b"FS  0x%04x  Index 0x%04x  %cDT RPL%d\n\0",
            seg_fs, seg_fs >> 3, if (seg_fs & (1 << 2)) != 0 { b'L' as i32 } else { b'G' as i32 }, seg_fs & 3,
        );
        crate::kdbp_print!(b"GS  0x%04x  Index 0x%04x  %cDT RPL%d\n\0",
            seg_gs, seg_gs >> 3, if (seg_gs & (1 << 2)) != 0 { b'L' as i32 } else { b'G' as i32 }, seg_gs & 3,
        );
        crate::kdbp_print!(b"SS  0x%04x  Index 0x%04x  %cDT RPL%d\n\0",
            seg_ss, seg_ss >> 3, if (seg_ss & (1 << 2)) != 0 { b'L' as i32 } else { b'G' as i32 }, seg_ss & 3,
        );
    } else {
        crate::kdbp_print!(b"DR0  0x%08x\nDR1  0x%08x\nDR2  0x%08x\nDR3  0x%08x\nDR6  0x%08x\nDR7  0x%08x\n\0",
            (*Context).Dr0, (*Context).Dr1, (*Context).Dr2, (*Context).Dr3,
            (*Context).Dr6, (*Context).Dr7,
        );
    }

    return 1u8;
}

#[cfg(target_arch = "x86")]
unsafe fn KdbpRetrieveTss(
    TssSelector: USHORT,
    pType: PULONG,
    pGdtr: PKDESCRIPTOR
) -> PKTSS {
    let mut Gdtr: KDESCRIPTOR = core::mem::zeroed();
    let mut Desc: KGDTENTRY = core::mem::zeroed();
    let Tss: PKTSS;

    if !pGdtr.is_null() {
        Gdtr = *pGdtr;
    } else {
        Ke386GetGlobalDescriptorTable(core::ptr::addr_of_mut!(Gdtr.Limit));
    }

    if (TssSelector & (core::mem::size_of::<KGDTENTRY>() as u16 - 1)) != 0 ||
       (TssSelector < core::mem::size_of::<KGDTENTRY>() as u16) ||
       ((TssSelector + core::mem::size_of::<KGDTENTRY>() as u16 - 1) > Gdtr.Limit)
    {
        return core::ptr::null_mut();
    }

    if !NT_SUCCESS(KdbpSafeReadMemory(
        core::ptr::addr_of_mut!(Desc) as PVOID,
        (Gdtr.Base as ULONG_PTR + TssSelector as ULONG_PTR) as PVOID,
        core::mem::size_of::<KGDTENTRY>()
    )) {
        return core::ptr::null_mut();
    }

    let bits = Desc.HighWord.Bits;
    if bits.Type() != 9 && bits.Type() != 11 {
        return core::ptr::null_mut();
    }
    if !pType.is_null() {
        *pType = bits.Type() as ULONG;
    }

    let bytes = Desc.HighWord.Bytes;
    let base_mid = bytes.BaseMid as ULONG_PTR;
    let base_hi = bytes.BaseHi as ULONG_PTR;

    Tss = (Desc.BaseLow as ULONG_PTR |
           (base_mid << 16) |
           (base_hi << 24)) as PKTSS;

    Tss
}
#[cfg(not(target_arch = "x86"))]
unsafe fn KdbpRetrieveTss(
    _TssSelector: USHORT,
    _pType: PULONG,
    _pGdtr: PKDESCRIPTOR,
) -> PKTSS {
    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
unsafe fn KdbpIsNestedTss(
    TssSelector: USHORT,
    Tss: PKTSS,
) -> BOOLEAN {
    let mut Backlink: USHORT = 0;

    if Tss.is_null() {
        return 0u8;
    }

    if !NT_SUCCESS(KdbpSafeReadMemory(
        core::ptr::addr_of_mut!(Backlink) as PVOID,
        core::ptr::addr_of_mut!((*Tss).Backlink) as PVOID,
        core::mem::size_of::<USHORT>(),
    )) {
        return 0u8;
    }

    if Backlink != 0 && Backlink != TssSelector {
        1u8
    } else {
        0u8
    }
}
#[cfg(not(target_arch = "x86"))]
unsafe fn KdbpIsNestedTss(
    _TssSelector: USHORT,
    _Tss: PKTSS,
) -> BOOLEAN {
    0u8
}

#[cfg(target_arch = "x86")]
unsafe fn KdbpContextFromPrevTss(
    Context: PCONTEXT,
    TssSelector: PUSHORT,
    pTss: *mut PKTSS,
    pGdtr: PKDESCRIPTOR
) -> BOOLEAN {
    let mut Eip: ULONG_PTR = 0;
    let mut Ebp: ULONG_PTR = 0;
    let mut Backlink: USHORT = 0;
    let mut Tss: PKTSS = *pTss;

    if !NT_SUCCESS(KdbpSafeReadMemory(
        core::ptr::addr_of_mut!(Backlink) as PVOID,
        core::ptr::addr_of_mut!((*Tss).Backlink) as PVOID,
        core::mem::size_of::<USHORT>(),
    )) {
        return 0u8;
    }

    Tss = KdbpRetrieveTss(Backlink, core::ptr::null_mut(), pGdtr);
    if Tss.is_null() {
        return 0u8;
    }

    if !NT_SUCCESS(KdbpSafeReadMemory(
        core::ptr::addr_of_mut!(Eip) as PVOID,
        core::ptr::addr_of_mut!((*Tss).Eip) as PVOID,
        core::mem::size_of::<ULONG_PTR>(),
    )) {
        return 0u8;
    }

    if !NT_SUCCESS(KdbpSafeReadMemory(
        core::ptr::addr_of_mut!(Ebp) as PVOID,
        core::ptr::addr_of_mut!((*Tss).Ebp) as PVOID,
        core::mem::size_of::<ULONG_PTR>(),
    )) {
        return 0u8;
    }

    *TssSelector = Backlink;
    *pTss = Tss;
    (*Context).Eip = Eip as u32;
    (*Context).Ebp = Ebp as u32;

    return 1u8;
}
#[cfg(not(target_arch = "x86"))]
unsafe fn KdbpContextFromPrevTss(
    _Context: PCONTEXT,
    _TssSelector: PUSHORT,
    _pTss: *mut PKTSS,
    _pGdtr: PKDESCRIPTOR,
) -> BOOLEAN {
    0u8
}

#[cfg(target_arch = "x86_64")]
unsafe fn GetNextFrame(
    Context: PCONTEXT) -> BOOLEAN
{
    let mut FunctionEntry: PRUNTIME_FUNCTION;
    let mut ImageBase: ULONG64 = 0;
    let mut EstablisherFrame: ULONG64 = 0;
    let mut HandlerData: PVOID = core::ptr::null_mut();

    FunctionEntry = RtlLookupFunctionEntry((*Context).Rip, &mut ImageBase, core::ptr::null_mut());
    if FunctionEntry == core::ptr::null_mut()
    {
        // Leaf function: pop the return address. The C reads it under _SEH2_TRY and
        // returns FALSE on a fault; there is no SEH here, so read it the safe way.
        let mut Rip: DWORD64 = 0;
        if !NT_SUCCESS(KdbpSafeReadMemory(
            core::ptr::addr_of_mut!(Rip) as PVOID,
            (*Context).Rsp as PVOID,
            core::mem::size_of::<DWORD64>(),
        )) {
            return 0u8;
        }
        (*Context).Rip = Rip;
        (*Context).Rsp += core::mem::size_of::<DWORD64>() as u64;
        return 1u8;
    }
    else
    {
        RtlVirtualUnwind(0,
                         ImageBase,
                         (*Context).Rip,
                         FunctionEntry,
                         Context,
                         &mut HandlerData,
                         &mut EstablisherFrame,
                         core::ptr::null_mut());
    }

    1u8
}
/// amd64 `bt`: kdb_cli.c:1267 — unwind with RtlVirtualUnwind via GetNextFrame. amd64
/// code keeps no frame-pointer chain, so the i386 EBP chase below (which this crate used
/// to run on amd64 too, leaving GetNextFrame dead) stops after a frame or walks garbage.
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KdbpCmdBackTrace(
    _Argc: ULONG,
    _Argv: *mut PCHAR,
) -> BOOLEAN {
    let mut Context: CONTEXT = *KdbCurrentTrapFrame;

    crate::kdbp_print!(b"Frames:\n\0");
    loop {
        crate::kdbp_print!(b"[%p] \0", Context.Rsp as PVOID);

        if KdbSymPrintAddress(Context.Rip as PVOID, core::ptr::addr_of_mut!(Context)) == 0 {
            crate::kdbp_print!(b"<%p>\0", Context.Rip as PVOID);
        }
        crate::kdbp_print!(b"\n\0");

        if KdbOutputAborted != 0 {
            break;
        }

        if GetNextFrame(core::ptr::addr_of_mut!(Context)) == 0 {
            crate::kdbp_print!(b"Couldn't get next frame\n\0");
            break;
        }
        if Context.Rip == 0 || Context.Rsp == 0 {
            break;
        }
    }

    1u8
}

/// i386 `bt` (kdb_cli.c:1304): EBP frame chase plus the nested-TSS backlink walk.
#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn KdbpCmdBackTrace(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {

    let mut ul: ULONG;
    let mut Result: ULONGLONG = 0;
    let mut Context: CONTEXT = *KdbCurrentTrapFrame;
    let mut Frame: ULONG_PTR = KeGetContextFrameRegister(core::ptr::addr_of_mut!(Context));
    let mut Address: ULONG_PTR;
    let mut Argc = Argc;

    if Argc >= 2 {
        ul = 0;
        if strcmp(*Argv.add((Argc - 2) as usize), b"L\0".as_ptr() as *const i8) == 0 {
            ul = strtoul(*Argv.add((Argc - 1) as usize), core::ptr::null_mut(), 0);
            if ul > 0 {
                Argc -= 2;
            }
        } else if *(*Argv.add((Argc - 1) as usize)).add(0) == b'L' as i8 {
            ul = strtoul((*Argv.add((Argc - 1) as usize)).add(1), core::ptr::null_mut(), 0);
            if ul > 0 {
                Argc -= 1;
            }
        }

        Argc -= 1;
        ul = 1;
        while ul < Argc {
            let len = strlen(*Argv.add(ul as usize));
            *(*Argv.add(ul as usize)).add(len) = b' ' as i8;
            ul += 1;
        }
        Argc += 1;
    }

    if Argc > 1 {
        if *(*Argv.add(1)).add(0) == b'*' as i8 {
            *Argv.add(1) = (*Argv.add(1)).add(1);

            if KdbpEvaluateExpression(
                *Argv.add(1),
                KdbPromptStr.Length as LONG + (*Argv.add(1) as isize - *Argv.add(0) as isize) as LONG,
                core::ptr::addr_of_mut!(Result),
            ) == 0 {
                return 1u8;
            }

            if Result > (!0usize as ULONGLONG) {
                crate::kdbp_print!(b"Warning: Address %I64x is beeing truncated\n\0", Result);
            }

            Frame = Result as ULONG_PTR;
        } else {
            crate::kdbp_print!(b"Thread backtrace not supported yet!\n\0");
            return 1u8;
        }
    }

    #[cfg(target_arch = "x86")]
    let mut Gdtr: KDESCRIPTOR = core::mem::zeroed();
    #[cfg(target_arch = "x86")]
    let mut TssSelector: USHORT;
    #[cfg(target_arch = "x86")]
    let mut Tss: PKTSS;

    #[cfg(target_arch = "x86")]
    {
        Ke386GetGlobalDescriptorTable(core::ptr::addr_of_mut!(Gdtr.Limit));

        TssSelector = Ke386GetTr();
        Tss = KdbpRetrieveTss(TssSelector, core::ptr::null_mut(), core::ptr::addr_of_mut!(Gdtr));
        if KdbpIsNestedTss(TssSelector, Tss) != 0 {
            crate::kdbp_print!(b"[Active TSS 0x%04x @ 0x%p]\n\0", (TssSelector) as u32, Tss);
        }
    }

    if Argc <= 1 {
        crate::kdbp_print!(b"Eip:\n\0");
        if KdbSymPrintAddress(
            KeGetContextPc(core::ptr::addr_of_mut!(Context)) as PVOID,
            core::ptr::addr_of_mut!(Context),
        ) == 0 {
            crate::kdbp_print!(b"<%p>\n\0", KeGetContextPc(core::ptr::addr_of_mut!(Context)));
        } else {
            crate::kdbp_print!(b"\n\0");
        }
    }

    crate::kdbp_print!(b"Frames:\n\0");
    loop {
        let GotNextFrame: BOOLEAN;

        if Frame == 0 {
            #[cfg(target_arch = "x86")]
            {
                if KdbpIsNestedTss(TssSelector, Tss) == 0 {
                    break;
                }

                let got = KdbpContextFromPrevTss(
                    core::ptr::addr_of_mut!(Context),
                    core::ptr::addr_of_mut!(TssSelector),
                    core::ptr::addr_of_mut!(Tss),
                    core::ptr::addr_of_mut!(Gdtr),
                );
                if got == 0 {
                    crate::kdbp_print!(b"Couldn't access parent TSS 0x%04x\n\0", ((*Tss).Backlink) as u32);
                    break;
                }

                Address = Context.Eip as ULONG_PTR;
                Frame = Context.Ebp as ULONG_PTR;

                crate::kdbp_print!(b"[Parent TSS 0x%04x @ 0x%p]\n\0", (TssSelector) as u32, Tss);

                if KdbSymPrintAddress(Address as PVOID, core::ptr::addr_of_mut!(Context)) == 0 {
                    crate::kdbp_print!(b"<%08x>\n\0", Address);
                } else {
                    crate::kdbp_print!(b"\n\0");
                }
                continue;
            }
            #[cfg(not(target_arch = "x86"))]
            break;
        }

        Address = 0;
        if !NT_SUCCESS(KdbpSafeReadMemory(
            core::ptr::addr_of_mut!(Address) as PVOID,
            (Frame + core::mem::size_of::<ULONG_PTR>()) as PVOID,
            core::mem::size_of::<ULONG_PTR>(),
        )) {
            crate::kdbp_print!(b"Couldn't access memory at 0x%p!\n\0",
                Frame + core::mem::size_of::<ULONG_PTR>(),
            );
            #[cfg(target_arch = "x86")]
            {
                if KdbpIsNestedTss(TssSelector, Tss) == 0 {
                    break;
                }

                let got = KdbpContextFromPrevTss(
                    core::ptr::addr_of_mut!(Context),
                    core::ptr::addr_of_mut!(TssSelector),
                    core::ptr::addr_of_mut!(Tss),
                    core::ptr::addr_of_mut!(Gdtr),
                );
                if got == 0 {
                    crate::kdbp_print!(b"Couldn't access parent TSS 0x%04x\n\0", ((*Tss).Backlink) as u32);
                    break;
                }

                Address = Context.Eip as ULONG_PTR;
                Frame = Context.Ebp as ULONG_PTR;

                crate::kdbp_print!(b"[Parent TSS 0x%04x @ 0x%p]\n\0", (TssSelector) as u32, Tss);

                if KdbSymPrintAddress(Address as PVOID, core::ptr::addr_of_mut!(Context)) == 0 {
                    crate::kdbp_print!(b"<%08x>\n\0", Address);
                } else {
                    crate::kdbp_print!(b"\n\0");
                }
                continue;
            }
            #[cfg(not(target_arch = "x86"))]
            break;
        }

        if Address == 0 {
            #[cfg(target_arch = "x86")]
            {
                if KdbpIsNestedTss(TssSelector, Tss) == 0 {
                    break;
                }

                let got = KdbpContextFromPrevTss(
                    core::ptr::addr_of_mut!(Context),
                    core::ptr::addr_of_mut!(TssSelector),
                    core::ptr::addr_of_mut!(Tss),
                    core::ptr::addr_of_mut!(Gdtr),
                );
                if got == 0 {
                    crate::kdbp_print!(b"Couldn't access parent TSS 0x%04x\n\0", ((*Tss).Backlink) as u32);
                    break;
                }

                Address = Context.Eip as ULONG_PTR;
                Frame = Context.Ebp as ULONG_PTR;

                crate::kdbp_print!(b"[Parent TSS 0x%04x @ 0x%p]\n\0", (TssSelector) as u32, Tss);

                if KdbSymPrintAddress(Address as PVOID, core::ptr::addr_of_mut!(Context)) == 0 {
                    crate::kdbp_print!(b"<%08x>\n\0", Address);
                } else {
                    crate::kdbp_print!(b"\n\0");
                }
                continue;
            }
            #[cfg(not(target_arch = "x86"))]
            break;
        }

        GotNextFrame = NT_SUCCESS(KdbpSafeReadMemory(
            core::ptr::addr_of_mut!(Frame) as PVOID,
            Frame as PVOID,
            core::mem::size_of::<ULONG_PTR>(),
        )) as BOOLEAN;
        if GotNextFrame != 0 {
            KeSetContextFrameRegister(core::ptr::addr_of_mut!(Context), Frame);
        }

        if KdbSymPrintAddress((Address - 5) as PVOID, core::ptr::addr_of_mut!(Context)) == 0 {
            crate::kdbp_print!(b"<%08x>\n\0", Address);
        } else {
            crate::kdbp_print!(b"\n\0");
        }

        if KdbOutputAborted != 0 {
            break;
        }

        if GotNextFrame == 0 {
            crate::kdbp_print!(b"Couldn't access memory at 0x%p!\n\0", Frame);
            #[cfg(target_arch = "x86")]
            {
                if KdbpIsNestedTss(TssSelector, Tss) == 0 {
                    break;
                }

                let got = KdbpContextFromPrevTss(
                    core::ptr::addr_of_mut!(Context),
                    core::ptr::addr_of_mut!(TssSelector),
                    core::ptr::addr_of_mut!(Tss),
                    core::ptr::addr_of_mut!(Gdtr),
                );
                if got == 0 {
                    crate::kdbp_print!(b"Couldn't access parent TSS 0x%04x\n\0", ((*Tss).Backlink) as u32);
                    break;
                }

                Address = Context.Eip as ULONG_PTR;
                Frame = Context.Ebp as ULONG_PTR;

                crate::kdbp_print!(b"[Parent TSS 0x%04x @ 0x%p]\n\0", (TssSelector) as u32, Tss);

                if KdbSymPrintAddress(Address as PVOID, core::ptr::addr_of_mut!(Context)) == 0 {
                    crate::kdbp_print!(b"<%08x>\n\0", Address);
                } else {
                    crate::kdbp_print!(b"\n\0");
                }
                continue;
            }
            #[cfg(not(target_arch = "x86"))]
            break;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdContinue(
    Argc: ULONG,
    Argv: *mut PCHAR) -> BOOLEAN
{
    0u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdStep(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {

    let mut Count: ULONG = 1;

    if Argc > 1 {
        Count = strtoul(*Argv.offset(1), core::ptr::null_mut(), 0);
        if Count == 0 {
            crate::kdbp_print!(b"%s: Integer argument required\n\0", *Argv.offset(0));
            return 1u8;
        }
    }

    if *(*Argv.offset(0)).offset(0) == b'n' as i8 {
        KdbSingleStepOver = 1u8;
    } else {
        KdbSingleStepOver = 0u8;
    }

    KdbNumSingleSteps = Count;

    return 0u8;
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdBreakPointList(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {

    let mut l: LONG;
    let mut Address: ULONG_PTR = 0;
    let mut Type: KDB_BREAKPOINT_TYPE = KdbBreakPointNone;
    let mut AccessType: KDB_ACCESS_TYPE = KdbAccessRead;
    let mut Size: UCHAR = 0;
    let mut DebugReg: UCHAR = 0;
    let mut Enabled: BOOLEAN = 0u8;
    let mut Global: BOOLEAN = 0u8;
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut str1: PCHAR;
    let mut str2: PCHAR;
    let mut ConditionExpr: PCHAR = core::ptr::null_mut();
    let mut GlobalOrLocal: PCHAR;
    let mut Buffer: [CHAR; 20] = [0; 20];

    l = KdbpGetNextBreakPointNr(0);
    if l < 0 {
        crate::kdbp_print!(b"No breakpoints.\n\0");
        return 1u8;
    }

    crate::kdbp_print!(b"Breakpoints:\n\0");
    loop {
        if KdbpGetBreakPointInfo(
            l as ULONG,
            &mut Address,
            &mut Type,
            &mut Size,
            &mut AccessType,
            &mut DebugReg,
            &mut Enabled,
            &mut Global,
            &mut Process,
            &mut ConditionExpr,
        ) == 0
        {
            l = KdbpGetNextBreakPointNr((l + 1) as ULONG);
            if l < 0 {
                break;
            }
            continue;
        }

        if l == KdbLastBreakPointNr {
            str1 = b"\x1b[1m*\0".as_ptr() as PCHAR;
            str2 = b"\x1b[0m\0".as_ptr() as PCHAR;
        } else {
            str1 = b" \0".as_ptr() as PCHAR;
            str2 = b"\0".as_ptr() as PCHAR;
        }

        if Global != 0 {
            GlobalOrLocal = b"  global\0".as_ptr() as PCHAR;
        } else {
            GlobalOrLocal = Buffer.as_mut_ptr();
            sprintf(
                Buffer.as_mut_ptr(),
                b"  PID 0x%Ix\0".as_ptr() as *const i8,
                if !Process.is_null() {
                    (*Process).UniqueProcessId as ULONG_PTR
                } else {
                    INVALID_HANDLE_VALUE as ULONG_PTR
                },
            );
        }

        if Type == KdbBreakPointSoftware || Type == KdbBreakPointTemporary {
            crate::kdbp_print!(b" %s%03d  BPX  0x%08x%s%s%s%s%s\n\0",
                str1,
                l,
                Address,
                if Enabled != 0 { b"\0".as_ptr() } else { b"  disabled\0".as_ptr() } as PCHAR,
                GlobalOrLocal,
                if !ConditionExpr.is_null() { b"  IF \0".as_ptr() } else { b"\0".as_ptr() } as PCHAR,
                if !ConditionExpr.is_null() { ConditionExpr } else { b"\0".as_ptr() as PCHAR },
                str2,
            );
        } else if Type == KdbBreakPointHardware {
            if Enabled == 0 {
                crate::kdbp_print!(b" %s%03d  BPM  0x%08x  %-5s %-5s  disabled%s%s%s%s\n\0",
                    str1,
                    l,
                    Address,
                    KDB_ACCESS_TYPE_TO_STRING(AccessType),
                    if Size == 1 { b"byte\0".as_ptr() } else if Size == 2 { b"word\0".as_ptr() } else { b"dword\0".as_ptr() } as PCHAR,
                    GlobalOrLocal,
                    if !ConditionExpr.is_null() { b"  IF \0".as_ptr() } else { b"\0".as_ptr() } as PCHAR,
                    if !ConditionExpr.is_null() { ConditionExpr } else { b"\0".as_ptr() as PCHAR },
                    str2,
                );
            } else {
                crate::kdbp_print!(b" %s%03d  BPM  0x%08x  %-5s %-5s  DR%d%s%s%s%s\n\0",
                    str1,
                    l,
                    Address,
                    KDB_ACCESS_TYPE_TO_STRING(AccessType),
                    if Size == 1 { b"byte\0".as_ptr() } else if Size == 2 { b"word\0".as_ptr() } else { b"dword\0".as_ptr() } as PCHAR,
                    DebugReg as i32,
                    GlobalOrLocal,
                    if !ConditionExpr.is_null() { b"  IF \0".as_ptr() } else { b"\0".as_ptr() } as PCHAR,
                    if !ConditionExpr.is_null() { ConditionExpr } else { b"\0".as_ptr() as PCHAR },
                    str2,
                );
            }
        }

        l = KdbpGetNextBreakPointNr((l + 1) as ULONG);
        if l < 0 {
            break;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdEnableDisableClearBreakPoint(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {

    let mut pend: PCHAR = core::ptr::null_mut();
    let BreakPointNr: ULONG;

    if Argc < 2 {
        crate::kdbp_print!(b"%s: argument required\n\0", *Argv.offset(0));
        return 1u8;
    }

    pend = *Argv.offset(1);
    BreakPointNr = strtoul(*Argv.offset(1), &mut pend, 0);
    if pend == *Argv.offset(1) || *pend != 0 {
        crate::kdbp_print!(b"%s: integer argument required\n\0", *Argv.offset(0));
        return 1u8;
    }

    if *(*Argv.offset(0)).offset(1) == b'e' as i8 {
        KdbpEnableBreakPoint(BreakPointNr, core::ptr::null_mut());
    } else if *(*Argv.offset(0)).offset(1) == b'd' as i8 {
        KdbpDisableBreakPoint(BreakPointNr, core::ptr::null_mut());
    } else {
        KdbpDeleteBreakPoint(BreakPointNr, core::ptr::null_mut());
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdBreakPoint(Argc: ULONG, Argv: *mut PCHAR) -> BOOLEAN {

    let mut Result: ULONGLONG = 0;
    let Address: ULONG_PTR;
    let Type: KDB_BREAKPOINT_TYPE;
    let mut Size: UCHAR = 0;
    let mut AccessType: KDB_ACCESS_TYPE = KdbAccessRead;
    let mut AddressArgIndex: ULONG;
    let mut ConditionArgIndex: LONG;
    let Global: BOOLEAN = 1u8;

    if *(*Argv.offset(0)).offset(2) == b'x' as i8 {
        if Argc < 2 {
            crate::kdbp_print!(b"bpx: Address argument required.\n\0");
            return 1u8;
        }

        AddressArgIndex = 1;
        Type = KdbBreakPointSoftware;
    } else {
        if Argc < 2 {
            crate::kdbp_print!(b"bpm: Access type argument required (one of r, w, rw, x)\n\0");
            return 1u8;
        }

        if _stricmp(*Argv.offset(1), b"x\0".as_ptr() as *const i8) == 0 {
            AccessType = KdbAccessExec;
        } else if _stricmp(*Argv.offset(1), b"r\0".as_ptr() as *const i8) == 0 {
            AccessType = KdbAccessRead;
        } else if _stricmp(*Argv.offset(1), b"w\0".as_ptr() as *const i8) == 0 {
            AccessType = KdbAccessWrite;
        } else if _stricmp(*Argv.offset(1), b"rw\0".as_ptr() as *const i8) == 0 {
            AccessType = KdbAccessReadWrite;
        } else {
            crate::kdbp_print!(b"bpm: Unknown access type '%s'\n\0", *Argv.offset(1));
            return 1u8;
        }

        if Argc < 3 {
            crate::kdbp_print!(b"bpm: %s argument required.\n\0",
                if AccessType == KdbAccessExec {
                    b"Address\0".as_ptr() as PSTR
                } else {
                    b"Memory size\0".as_ptr() as PSTR
                }
            );
            return 1u8;
        }

        AddressArgIndex = 3;
        if _stricmp(*Argv.offset(2), b"byte\0".as_ptr() as *const i8) == 0 {
            Size = 1;
        } else if _stricmp(*Argv.offset(2), b"word\0".as_ptr() as *const i8) == 0 {
            Size = 2;
        } else if _stricmp(*Argv.offset(2), b"dword\0".as_ptr() as *const i8) == 0 {
            Size = 4;
        } else if AccessType == KdbAccessExec {
            Size = 1;
            AddressArgIndex = AddressArgIndex.wrapping_sub(1);
        } else {
            crate::kdbp_print!(b"bpm: Unknown memory size '%s'\n\0", *Argv.offset(2));
            return 1u8;
        }

        if Argc <= AddressArgIndex {
            crate::kdbp_print!(b"bpm: Address argument required.\n\0");
            return 1u8;
        }

        Type = KdbBreakPointHardware;
    }

    ConditionArgIndex = -1;
    let mut i = AddressArgIndex;
    while i < Argc.wrapping_sub(1) {
        if strcmp(*Argv.offset((i + 1) as isize), b"IF\0".as_ptr() as *const i8) == 0 {
            ConditionArgIndex = (i + 2) as LONG;
            if (ConditionArgIndex as ULONG) >= Argc {
                crate::kdbp_print!(b"%s: IF requires condition expression.\n\0", *Argv.offset(0));
                return 1u8;
            }

            let mut j = ConditionArgIndex as ULONG;
            while j < Argc.wrapping_sub(1) {
                let arg = *Argv.offset(j as isize);
                let len = strlen(arg);
                *arg.offset(len as isize) = b' ' as i8;
                j = j.wrapping_add(1);
            }

            break;
        }

        let arg = *Argv.offset(i as isize);
        let len = strlen(arg);
        *arg.offset(len as isize) = b' ' as i8;
        i = i.wrapping_add(1);
    }

    if KdbpEvaluateExpression(
        *Argv.offset(AddressArgIndex as isize),
        (KdbPromptStr.Length as isize + (*Argv.offset(AddressArgIndex as isize) as isize - *Argv.offset(0) as isize)) as LONG,
        &mut Result,
    ) == 0 {
        return 1u8;
    }

    if Result > (!0usize) as ULONGLONG {
        crate::kdbp_print!(b"%s: Warning: Address %I64x is beeing truncated\n\0", *Argv.offset(0), Result);
    }

    Address = Result as ULONG_PTR;

    KdbpInsertBreakPoint(
        Address,
        Type,
        Size,
        AccessType,
        if ConditionArgIndex < 0 {
            core::ptr::null_mut()
        } else {
            *Argv.offset(ConditionArgIndex as isize)
        },
        Global,
        core::ptr::null_mut(),
    );

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdThread(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {

    let mut Entry: PLIST_ENTRY;
    let mut Thread: PETHREAD = core::ptr::null_mut();
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let mut ReferencedThread: BOOLEAN = 0u8;
    let mut ReferencedProcess: BOOLEAN = 0u8;
    let mut Stack: PULONG_PTR;
    let mut Frame: PULONG_PTR;
    let mut Pc: ULONG_PTR;
    let mut ul: ULONG_PTR = 0;
    let mut State: PCHAR;
    let mut pend: PCHAR = core::ptr::null_mut();
    let mut str1: PCHAR;
    let mut str2: PCHAR;
    let ThreadStateToString: [PCHAR; 8] = [
        b"Initialized\0".as_ptr() as PCHAR,
        b"Ready\0".as_ptr() as PCHAR,
        b"Running\0".as_ptr() as PCHAR,
        b"Standby\0".as_ptr() as PCHAR,
        b"Terminated\0".as_ptr() as PCHAR,
        b"Waiting\0".as_ptr() as PCHAR,
        b"Transition\0".as_ptr() as PCHAR,
        b"DeferredReady\0".as_ptr() as PCHAR,
    ];

    if Argc >= 2 && _stricmp(*Argv.offset(1), b"list\0".as_ptr() as PCHAR) == 0 {
        Process = KdbCurrentProcess;

        if Argc >= 3 {
            ul = strtoulptr(*Argv.offset(2), &mut pend, 0);
            if *Argv.offset(2) == pend {
                crate::kdbp_print!(b"thread: '%s' is not a valid process id!\n\0", *Argv.offset(2));
                return 1u8;
            }

            if !NT_SUCCESS(PsLookupProcessByProcessId(ul as PVOID, &mut Process)) {
                crate::kdbp_print!(b"thread: Invalid process id!\n\0");
                return 1u8;
            }

            ReferencedProcess = 1u8;
        }

        Entry = (*Process).ThreadListHead.Flink;
        if Entry == core::ptr::addr_of_mut!((*Process).ThreadListHead) {
            if Argc >= 3 {
                crate::kdbp_print!(b"No threads in process 0x%px!\n\0", ul as PVOID);
            } else {
                crate::kdbp_print!(b"No threads in current process!\n\0");
            }

            if ReferencedProcess != 0 {
                ObDereferenceObject(Process as PVOID);
            }

            return 1u8;
        }

        crate::kdbp_print!(b"  TID         State        Prior.  Affinity    EBP         EIP\n\0");
        loop {
            Thread = (Entry as usize - core::mem::offset_of!(_ETHREAD, ThreadListEntry)) as PETHREAD;

            if Thread == KdbCurrentThread {
                str1 = b"\x1b[1m*\0".as_ptr() as PCHAR;
                str2 = b"\x1b[0m\0".as_ptr() as PCHAR;
            } else {
                str1 = b" \0".as_ptr() as PCHAR;
                str2 = b"\0".as_ptr() as PCHAR;
            }

            if (*Thread).Tcb.InitialStack.is_null() {
                Stack = core::ptr::null_mut();
                Frame = core::ptr::null_mut();
                Pc = 0;
            } else if !(*Thread).Tcb.TrapFrame.is_null() {
                Stack = KeGetTrapFrameStackRegister((*Thread).Tcb.TrapFrame as PVOID) as PULONG_PTR;
                Frame = KeGetTrapFrameFrameRegister((*Thread).Tcb.TrapFrame as PVOID) as PULONG_PTR;
                Pc = KeGetTrapFramePc((*Thread).Tcb.TrapFrame as PVOID);
            } else {
                Stack = (*Thread).Tcb.KernelStack as PULONG_PTR;
                Frame = *Stack.offset(4) as PULONG_PTR;
                Pc = 0;

                if !Frame.is_null() {
                    KdbpSafeReadMemory(core::ptr::addr_of_mut!(Pc) as *mut core::ffi::c_void, Frame.offset(1) as *mut core::ffi::c_void, core::mem::size_of::<ULONG_PTR>());
                }
            }

            if kthread_state(&(*Thread).Tcb) < 8 {
                State = ThreadStateToString[kthread_state(&(*Thread).Tcb) as usize];
            } else {
                State = b"Unknown\0".as_ptr() as PCHAR;
            }

            crate::kdbp_print!(b" %s0x%08x  %-11s  %3d     0x%08x  0x%08x  0x%08x%s\n\0",
                      str1,
                      (*Thread).Cid.UniqueThread,
                      State,
                      ((*Thread).Tcb.Priority) as i32,
                      (*Thread).Tcb.Affinity,
                      Frame,
                      Pc,
                      str2);

            Entry = (*Entry).Flink;
            if Entry == core::ptr::addr_of_mut!((*Process).ThreadListHead) {
                break;
            }
        }

        if ReferencedProcess != 0 {
            ObDereferenceObject(Process as PVOID);
        }
    } else if Argc >= 2 && _stricmp(*Argv.offset(1), b"attach\0".as_ptr() as PCHAR) == 0 {
        if Argc < 3 {
            crate::kdbp_print!(b"thread attach: thread id argument required!\n\0");
            return 1u8;
        }

        ul = strtoulptr(*Argv.offset(2), &mut pend, 0);
        if *Argv.offset(2) == pend {
            crate::kdbp_print!(b"thread attach: '%s' is not a valid thread id!\n\0", *Argv.offset(2));
            return 1u8;
        }

        if KdbpAttachToThread(ul as PVOID) == 0 {
            return 1u8;
        }

        crate::kdbp_print!(b"Attached to thread 0x%08x.\n\0", ul);
    } else {
        Thread = KdbCurrentThread;

        if Argc >= 2 {
            ul = strtoulptr(*Argv.offset(1), &mut pend, 0);
            if *Argv.offset(1) == pend {
                crate::kdbp_print!(b"thread: '%s' is not a valid thread id!\n\0", *Argv.offset(1));
                return 1u8;
            }

            if !NT_SUCCESS(PsLookupThreadByThreadId(ul as PVOID, &mut Thread)) {
                crate::kdbp_print!(b"thread: Invalid thread id!\n\0");
                return 1u8;
            }

            ReferencedThread = 1u8;
        }

        if kthread_state(&(*Thread).Tcb) < 8 {
            State = ThreadStateToString[kthread_state(&(*Thread).Tcb) as usize];
        } else {
            State = b"Unknown\0".as_ptr() as PCHAR;
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            crate::kdbp_print!(b"%s  TID:            0x%08x\n  State:          %s (0x%x)\n  Priority:       %d\n  Affinity:       0x%08x\n  Initial Stack:  0x%08x\n  Stack Limit:    0x%08x\n  Stack Base:     0x%08x\n  Kernel Stack:   0x%08x\n  Trap Frame:     0x%08x\n  NPX State:      %s (0x%x)\n\0",
                      if Argc < 2 { b"Current Thread:\n\0".as_ptr() as PCHAR } else { b"\0".as_ptr() as PCHAR },
                      (*Thread).Cid.UniqueThread,
                      State, (kthread_state(&(*Thread).Tcb)) as u32,
                      ((*Thread).Tcb.Priority) as i32,
                      (*Thread).Tcb.Affinity,
                      (*Thread).Tcb.InitialStack,
                      (*Thread).Tcb.StackLimit,
                      (*Thread).Tcb.StackBase,
                      (*Thread).Tcb.KernelStack,
                      (*Thread).Tcb.TrapFrame,
                      NPX_STATE_TO_STRING((*Thread).Tcb.NpxState as u32), ((*Thread).Tcb.NpxState) as u32);
        }
        #[cfg(target_arch = "x86_64")]
        {
            crate::kdbp_print!(b"%s  TID:            0x%08x\n  State:          %s (0x%x)\n  Priority:       %d\n  Affinity:       0x%08x\n  Initial Stack:  0x%08x\n  Stack Limit:    0x%08x\n  Stack Base:     0x%08x\n  Kernel Stack:   0x%08x\n  Trap Frame:     0x%08x\n\0",
                      if Argc < 2 { b"Current Thread:\n\0".as_ptr() as PCHAR } else { b"\0".as_ptr() as PCHAR },
                      (*Thread).Cid.UniqueThread,
                      State, kthread_state(&(*Thread).Tcb) as u32,
                      (*Thread).Tcb.Priority as i32,
                      (*Thread).Tcb.Affinity,
                      (*Thread).Tcb.InitialStack,
                      (*Thread).Tcb.StackLimit,
                      (*Thread).Tcb.StackBase,
                      (*Thread).Tcb.KernelStack,
                      (*Thread).Tcb.TrapFrame);
        }

        if ReferencedThread != 0 {
            ObDereferenceObject(Thread as PVOID);
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdProc(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {
    let mut Entry: PLIST_ENTRY;
    let mut Process: PEPROCESS;
    let mut ReferencedProcess: BOOLEAN = 0u8;
    let mut State: PCHAR;
    let mut pend: *mut CHAR = core::ptr::null_mut();
    let mut str1: PCHAR;
    let mut str2: PCHAR;
    let mut ul: ULONG_PTR;

    if Argc >= 2 && _stricmp(*Argv.offset(1), b"list\0".as_ptr() as PCHAR) == 0 {
        Entry = PsActiveProcessHead.Flink;
        if Entry.is_null() || Entry == core::ptr::addr_of_mut!(PsActiveProcessHead) as PLIST_ENTRY {
            crate::kdbp_print!(b"No processes in the system!\n\0");
            return 1u8;
        }

        crate::kdbp_print!(b"  PID         State       Filename\n\0");
        loop {
            Process = (Entry as *mut u8).offset(-(core::mem::offset_of!(_EPROCESS, ActiveProcessLinks) as isize)) as PEPROCESS;

            if Process == KdbCurrentProcess {
                str1 = b"\x1b[1m*\0".as_ptr() as PCHAR;
                str2 = b"\x1b[0m\0".as_ptr() as PCHAR;
            } else {
                str1 = b" \0".as_ptr() as PCHAR;
                str2 = b"\0".as_ptr() as PCHAR;
            }

            State = if (*Process).Pcb.State == 0 {
                b"In Memory\0".as_ptr() as PCHAR
            } else if (*Process).Pcb.State == 1 {
                b"Out of Memory\0".as_ptr() as PCHAR
            } else {
                b"In Transition\0".as_ptr() as PCHAR
            };

            crate::kdbp_print!(b" %s0x%08x  %-10s  %s%s\n\0",
                str1,
                (*Process).UniqueProcessId,
                State,
                (*Process).ImageFileName.as_ptr(),
                str2,
            );

            Entry = (*Entry).Flink;
            if Entry == core::ptr::addr_of_mut!(PsActiveProcessHead) as PLIST_ENTRY {
                break;
            }
        }
    } else if Argc >= 2 && _stricmp(*Argv.offset(1), b"attach\0".as_ptr() as PCHAR) == 0 {
        if Argc < 3 {
            crate::kdbp_print!(b"process attach: process id argument required!\n\0");
            return 1u8;
        }

        ul = strtoulptr(*Argv.offset(2), core::ptr::addr_of_mut!(pend), 0);
        if *Argv.offset(2) == pend {
            crate::kdbp_print!(b"process attach: '%s' is not a valid process id!\n\0",
                *Argv.offset(2),
            );
            return 1u8;
        }

        if KdbpAttachToProcess(ul as PVOID) == 0 {
            return 1u8;
        }

        crate::kdbp_print!(b"Attached to process 0x%p, thread 0x%p.\n\0",
            ul as PVOID,
            (*KdbCurrentThread).Cid.UniqueThread,
        );
    } else {
        Process = KdbCurrentProcess;

        if Argc >= 2 {
            ul = strtoulptr(*Argv.offset(1), core::ptr::addr_of_mut!(pend), 0);
            if *Argv.offset(1) == pend {
                crate::kdbp_print!(b"proc: '%s' is not a valid process id!\n\0",
                    *Argv.offset(1),
                );
                return 1u8;
            }

            if !NT_SUCCESS(PsLookupProcessByProcessId(ul as PVOID, core::ptr::addr_of_mut!(Process))) {
                crate::kdbp_print!(b"proc: Invalid process id!\n\0");
                return 1u8;
            }

            ReferencedProcess = 1u8;
        }

        State = if (*Process).Pcb.State == 0 {
            b"In Memory\0".as_ptr() as PCHAR
        } else if (*Process).Pcb.State == 1 {
            b"Out of Memory\0".as_ptr() as PCHAR
        } else {
            b"In Transition\0".as_ptr() as PCHAR
        };
        crate::kdbp_print!(b"%s  PID:             0x%08x\n  State:           %s (0x%x)\n  Image Filename:  %s\n\0",
            if Argc < 2 { b"Current process:\n\0".as_ptr() as PCHAR } else { b"\0".as_ptr() as PCHAR },
            (*Process).UniqueProcessId,
            State,
            ((*Process).Pcb.State) as u32,
            (*Process).ImageFileName.as_ptr(),
        );

        if ReferencedProcess != 0 {
            ObDereferenceObject(Process as PVOID);
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdMod(
    mut Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {

    let mut Result: ULONGLONG = 0;
    let mut Address: ULONG_PTR;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut DisplayOnlyOneModule: BOOLEAN = 0;
    let mut i: INT = 0;

    if Argc >= 2 {
        Argc -= 1;
        while Argc >= 2 {
            Argc -= 1;
            let arg_ptr = *Argv.offset(Argc as isize);
            let len = strlen(arg_ptr);
            *arg_ptr.offset(len as isize) = b' ' as CHAR;
        }

        if KdbpEvaluateExpression(
            *Argv.offset(1),
            KdbPromptStr.Length as LONG + (*Argv.offset(1) as isize - *Argv.offset(0) as isize) as LONG,
            &mut Result,
        ) == 0
        {
            return 1;
        }

        if Result > !(0 as ULONG_PTR) as ULONGLONG {
            crate::kdbp_print!(b"%s: Warning: Address %I64x is beeing truncated\n\0",
                *Argv.offset(0),
                Result,
            );
        }

        Address = Result as ULONG_PTR;

        if KdbpSymFindModule(Address as PVOID, -1, &mut LdrEntry) == 0 {
            crate::kdbp_print!(b"No module containing address 0x%p found!\n\0",
                Address,
            );
            return 1;
        }

        DisplayOnlyOneModule = 1;
    } else {
        if KdbpSymFindModule(core::ptr::null_mut(), 0, &mut LdrEntry) == 0 {
            let ntoskrnlBase: ULONG_PTR = __ImageBase as ULONG_PTR;
            crate::kdbp_print!(b"  Base      Size      Name\n\0");
            crate::kdbp_print!(b"  %p  %08x  %s\n\0",
                ntoskrnlBase as PVOID,
                0,
                b"ntoskrnl.exe\0".as_ptr() as PSTR,
            );
            return 1;
        }

        i = 1;
    }

    crate::kdbp_print!(b"  Base      Size      Name\n\0");
    loop {
        crate::kdbp_print!(b"  %p  %08x  \0",
            (*LdrEntry).DllBase,
            (*LdrEntry).SizeOfImage,
        );
        KdbpPrintUnicodeString(&(*LdrEntry).BaseDllName);
        crate::kdbp_print!(b"\n\0");

        if DisplayOnlyOneModule != 0 || KdbpSymFindModule(core::ptr::null_mut(), i, &mut LdrEntry) == 0 {
            break;
        }
        i += 1;
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdGdtLdtIdt(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {

    let mut Reg: KDESCRIPTOR = core::mem::zeroed();
    let mut SegDesc: [ULONG; 2] = [0; 2];
    let mut SegBase: ULONG;
    let mut SegLimit: ULONG;
    let mut SegType: PCHAR;
    let mut SegSel: USHORT;
    let mut Type: UCHAR;
    let mut Dpl: UCHAR;
    let mut i: INT;
    let mut ul: ULONG;

    if *(*Argv.offset(0)).offset(0) == b'i' as i8 {
        #[cfg(target_arch = "x86")]
        __sidt(core::ptr::addr_of_mut!(Reg.Limit) as PVOID);

        if Reg.Limit < 7 {
            crate::kdbp_print!(b"Interrupt descriptor table is empty.\n\0");
            return 1u8;
        }

        crate::kdbp_print!(b"IDT Base: 0x%08x  Limit: 0x%04x\n\0", Reg.Base, (Reg.Limit) as u32);
        crate::kdbp_print!(b"  Idx  Type        Seg. Sel.  Offset      DPL\n\0");

        i = 0;
        while (i + core::mem::size_of::<[ULONG; 2]>() as INT - 1) <= Reg.Limit as INT {
            if !NT_SUCCESS(KdbpSafeReadMemory(SegDesc.as_mut_ptr() as PVOID, (Reg.Base as ULONG_PTR + i as ULONG_PTR) as PVOID, core::mem::size_of::<[ULONG; 2]>())) {
                crate::kdbp_print!(b"Couldn't access memory at 0x%p!\n\0", (Reg.Base as ULONG_PTR + i as ULONG_PTR) as PVOID);
                return 1u8;
            }

            Dpl = ((SegDesc[1] >> 13) & 3) as UCHAR;
            if (SegDesc[1] & 0x1f00) == 0x0500 {
                SegType = b"TASKGATE\0".as_ptr() as PCHAR;
            } else if (SegDesc[1] & 0x1fe0) == 0x0e00 {
                SegType = b"INTGATE32\0".as_ptr() as PCHAR;
            } else if (SegDesc[1] & 0x1fe0) == 0x0600 {
                SegType = b"INTGATE16\0".as_ptr() as PCHAR;
            } else if (SegDesc[1] & 0x1fe0) == 0x0f00 {
                SegType = b"TRAPGATE32\0".as_ptr() as PCHAR;
            } else if (SegDesc[1] & 0x1fe0) == 0x0700 {
                SegType = b"TRAPGATE16\0".as_ptr() as PCHAR;
            } else {
                SegType = b"UNKNOWN\0".as_ptr() as PCHAR;
            }

            if (SegDesc[1] & (1 << 15)) == 0 {
                crate::kdbp_print!(b"  %03d  %-10s  [NP]       [NP]        %02d\n\0",
                          i / 8, SegType, (Dpl) as u32);
            } else if (SegDesc[1] & 0x1f00) == 0x0500 {
                SegSel = (SegDesc[0] >> 16) as USHORT;
                crate::kdbp_print!(b"  %03d  %-10s  0x%04x                 %02d\n\0",
                          i / 8, SegType, (SegSel) as u32, (Dpl) as u32);
            } else {
                SegSel = (SegDesc[0] >> 16) as USHORT;
                SegBase = (SegDesc[1] & 0xffff0000) | (SegDesc[0] & 0x0000ffff);
                crate::kdbp_print!(b"  %03d  %-10s  0x%04x     0x%08x  %02d\n\0",
                          i / 8, SegType, (SegSel) as u32, SegBase, (Dpl) as u32);
            }

            i += 8;
        }
    } else {
        ul = 0;

        if *(*Argv.offset(0)).offset(0) == b'g' as i8 {
            Ke386GetGlobalDescriptorTable(core::ptr::addr_of_mut!(Reg.Limit));
            i = 8;
        } else {
            Ke386GetLocalDescriptorTable(core::ptr::addr_of_mut!(Reg.Limit));
            Reg.Base = 0;
            i = 0;
            ul = 1 << 2;
        }

        if Reg.Limit < 7 {
            crate::kdbp_print!(b"%s descriptor table is empty.\n\0",
                      if *(*Argv.offset(0)).offset(0) == b'g' as i8 { b"Global\0".as_ptr() } else { b"Local\0".as_ptr() });
            return 1u8;
        }

        crate::kdbp_print!(b"%cDT Base: 0x%08x  Limit: 0x%04x\n\0",
                  (if *(*Argv.offset(0)).offset(0) == b'g' as i8 { b'G' as i8 } else { b'L' as i8 }) as i32, Reg.Base, (Reg.Limit) as u32);
        crate::kdbp_print!(b"  Idx  Sel.    Type         Base        Limit       DPL  Attribs\n\0");

        while (i + core::mem::size_of::<[ULONG; 2]>() as INT - 1) <= Reg.Limit as INT {
            if !NT_SUCCESS(KdbpSafeReadMemory(SegDesc.as_mut_ptr() as PVOID, (Reg.Base as ULONG_PTR + i as ULONG_PTR) as PVOID, core::mem::size_of::<[ULONG; 2]>())) {
                crate::kdbp_print!(b"Couldn't access memory at 0x%p!\n\0", (Reg.Base as ULONG_PTR + i as ULONG_PTR) as PVOID);
                return 1u8;
            }

            Dpl = ((SegDesc[1] >> 13) & 3) as UCHAR;
            Type = ((SegDesc[1] >> 8) & 0xf) as UCHAR;

            SegBase = SegDesc[0] >> 16;
            SegBase |= (SegDesc[1] & 0xff) << 16;
            SegBase |= SegDesc[1] & 0xff000000;
            SegLimit = SegDesc[0] & 0x0000ffff;
            SegLimit |= (SegDesc[1] >> 16) & 0xf;

            if (SegDesc[1] & (1 << 23)) != 0 {
                SegLimit *= 4096;
                SegLimit += 4095;
            } else {
                SegLimit += 1;
            }

            if (SegDesc[1] & (1 << 12)) == 0 {
                SegType = match Type {
                    1 => b"TSS16(Avl)\0".as_ptr() as PCHAR,
                    2 => b"LDT\0".as_ptr() as PCHAR,
                    3 => b"TSS16(Busy)\0".as_ptr() as PCHAR,
                    4 => b"CALLGATE16\0".as_ptr() as PCHAR,
                    5 => b"TASKGATE\0".as_ptr() as PCHAR,
                    6 => b"INTGATE16\0".as_ptr() as PCHAR,
                    7 => b"TRAPGATE16\0".as_ptr() as PCHAR,
                    9 => b"TSS32(Avl)\0".as_ptr() as PCHAR,
                    11 => b"TSS32(Busy)\0".as_ptr() as PCHAR,
                    12 => b"CALLGATE32\0".as_ptr() as PCHAR,
                    14 => b"INTGATE32\0".as_ptr() as PCHAR,
                    15 => b"TRAPGATE32\0".as_ptr() as PCHAR,
                    _ => b"UNKNOWN\0".as_ptr() as PCHAR,
                };

                if !((Type >= 1 && Type <= 3) || Type == 9 || Type == 11) {
                    SegBase = 0;
                    SegLimit = 0;
                }
            } else if (SegDesc[1] & (1 << 11)) == 0 {
                if (SegDesc[1] & (1 << 22)) != 0 {
                    SegType = b"DATA32\0".as_ptr() as PCHAR;
                } else {
                    SegType = b"DATA16\0".as_ptr() as PCHAR;
                }
            } else {
                if (SegDesc[1] & (1 << 22)) != 0 {
                    SegType = b"CODE32\0".as_ptr() as PCHAR;
                } else {
                    SegType = b"CODE16\0".as_ptr() as PCHAR;
                }
            }

            if (SegDesc[1] & (1 << 15)) == 0 {
                crate::kdbp_print!(b"  %03d  0x%04x  %-11s  [NP]        [NP]        %02d   NP\n\0",
                          i / 8, i | Dpl as INT | ul as INT, SegType, (Dpl) as u32);
            } else {
                crate::kdbp_print!(b"  %03d  0x%04x  %-11s  0x%08x  0x%08x  %02d  \0",
                          i / 8, i | Dpl as INT | ul as INT, SegType, SegBase, SegLimit, (Dpl) as u32);

                if (SegDesc[1] & (1 << 12)) == 0 {
                } else if (SegDesc[1] & (1 << 11)) == 0 {
                    if (SegDesc[1] & (1 << 10)) != 0 {
                        crate::kdbp_print!(b" E\0");
                    }

                    crate::dbgprint::rust_kdbp_print_cstr(
                        if (SegDesc[1] & (1 << 9)) != 0 { b" R/W\0".as_ptr() } else { b" R\0".as_ptr() }
                    );

                    if (SegDesc[1] & (1 << 8)) != 0 {
                        crate::kdbp_print!(b" A\0");
                    }
                } else {
                    if (SegDesc[1] & (1 << 10)) != 0 {
                        crate::kdbp_print!(b" C\0");
                    }

                    crate::dbgprint::rust_kdbp_print_cstr(
                        if (SegDesc[1] & (1 << 9)) != 0 { b" R/X\0".as_ptr() } else { b" X\0".as_ptr() }
                    );

                    if (SegDesc[1] & (1 << 8)) != 0 {
                        crate::kdbp_print!(b" A\0");
                    }
                }

                if (SegDesc[1] & (1 << 20)) != 0 {
                    crate::kdbp_print!(b" AVL\0");
                }

                crate::kdbp_print!(b"\n\0");
            }

            i += 8;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdPcr(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {
    let Pcr: PKIPCR = KeGetPcr() as PKIPCR;

    crate::kdbp_print!(b"Current PCR is at 0x%p.\n\0", Pcr);
    #[cfg(target_arch = "x86")]
    {
        crate::kdbp_print!(b"  Tib.ExceptionList:         0x%08x\n  Tib.StackBase:             0x%08x\n  Tib.StackLimit:            0x%08x\n  Tib.SubSystemTib:          0x%08x\n  Tib.FiberData/Version:     0x%08x\n  Tib.ArbitraryUserPointer:  0x%08x\n  Tib.Self:                  0x%08x\n  SelfPcr:                   0x%08x\n  PCRCB:                     0x%08x\n  Irql:                      0x%02x\n  IRR:                       0x%08x\n  IrrActive:                 0x%08x\n  IDR:                       0x%08x\n  KdVersionBlock:            0x%08x\n  IDT:                       0x%08x\n  GDT:                       0x%08x\n  TSS:                       0x%08x\n  MajorVersion:              0x%04x\n  MinorVersion:              0x%04x\n  SetMember:                 0x%08x\n  StallScaleFactor:          0x%08x\n  Number:                    0x%02x\n  L2CacheAssociativity:      0x%02x\n  VdmAlert:                  0x%08x\n  L2CacheSize:               0x%08x\n  InterruptMode:             0x%08x\n\0",
            (*Pcr).__bindgen_anon_1.NtTib.ExceptionList,
            (*Pcr).__bindgen_anon_1.NtTib.StackBase,
            (*Pcr).__bindgen_anon_1.NtTib.StackLimit,
            (*Pcr).__bindgen_anon_1.NtTib.SubSystemTib,
            (*Pcr).__bindgen_anon_1.NtTib.__bindgen_anon_1.FiberData,
            (*Pcr).__bindgen_anon_1.NtTib.ArbitraryUserPointer,
            (*Pcr).__bindgen_anon_1.NtTib.Self_,
            (*Pcr).SelfPcr,
            (*Pcr).Prcb,
            ((*Pcr).Irql) as u32,
            (*Pcr).IRR,
            (*Pcr).IrrActive,
            (*Pcr).IDR,
            (*Pcr).KdVersionBlock,
            (*Pcr).IDT,
            (*Pcr).GDT,
            (*Pcr).TSS,
            ((*Pcr).MajorVersion) as u32,
            ((*Pcr).MinorVersion) as u32,
            (*Pcr).SetMember,
            (*Pcr).StallScaleFactor,
            ((*Pcr).Number) as u32,
            ((*Pcr).SecondLevelCacheAssociativity) as u32,
            (*Pcr).VdmAlert,
            (*Pcr).SecondLevelCacheSize,
            (*Pcr).InterruptMode,
        );
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let inner = &(*Pcr).__bindgen_anon_1.__bindgen_anon_1;
        crate::kdbp_print!(b"  GdtBase:                       0x%p\n\0", inner.GdtBase);
        crate::kdbp_print!(b"  TssBase:                       0x%p\n\0", inner.TssBase);
        crate::kdbp_print!(b"  UserRsp:                       0x%p\n\0", inner.UserRsp as PVOID);
        crate::kdbp_print!(b"  Self:                          0x%p\n\0", inner.Self_);
        crate::kdbp_print!(b"  CurrentPrcb:                   0x%p\n\0", inner.CurrentPrcb);
        crate::kdbp_print!(b"  LockArray:                     0x%p\n\0", inner.LockArray);
        crate::kdbp_print!(b"  Used_Self:                     0x%p\n\0", inner.Used_Self);
        crate::kdbp_print!(b"  IdtBase:                       0x%p\n\0", (*Pcr).IdtBase);
        crate::kdbp_print!(b"  Irql:                          %u\n\0", (*Pcr).Irql as u32);
        crate::kdbp_print!(b"  SecondLevelCacheAssociativity: 0x%u\n\0", (*Pcr).SecondLevelCacheAssociativity as u32);
        crate::kdbp_print!(b"  ObsoleteNumber:                %u\n\0", (*Pcr).ObsoleteNumber as u32);
        crate::kdbp_print!(b"  MajorVersion:                  0x%x\n\0", (*Pcr).MajorVersion as u32);
        crate::kdbp_print!(b"  MinorVersion:                  0x%x\n\0", (*Pcr).MinorVersion as u32);
        crate::kdbp_print!(b"  StallScaleFactor:              0x%lx\n\0", (*Pcr).StallScaleFactor);
        crate::kdbp_print!(b"  SecondLevelCacheSize:          0x%lx\n\0", (*Pcr).SecondLevelCacheSize);
        crate::kdbp_print!(b"  KdVersionBlock:                0x%p\n\0", (*Pcr).KdVersionBlock);
    }

    1u8
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KdbpCmdTss(
    _Argc: ULONG,
    _Argv: *mut PCHAR,
) -> BOOLEAN {
    crate::kdbp_print!(b"TSS introspection is only supported on i386.\n\0");
    1u8
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn KdbpCmdTss(
    Argc: ULONG,
    Argv: *mut PCHAR
) -> BOOLEAN {

    let mut TssSelector: USHORT = 0;
    let mut Tss: PKTSS = core::ptr::null_mut();

    if Argc >= 2 {
        let mut Param: PCHAR;
        let mut pszNext: PCHAR = core::ptr::null_mut();
        let ulValue: ULONG;

        Param = *Argv.offset(1);
        if *Param.offset(0) == b'*' as i8 {
            Param = Param.offset(1);
        }

        ulValue = strtoul(Param, core::ptr::addr_of_mut!(pszNext), 0);
        if !pszNext.is_null() && *pszNext != 0 {
            crate::kdbp_print!(b"Invalid TSS specification.\n\0");
            return 1u8;
        }

        if *(*Argv.offset(1)).offset(0) == b'*' as i8 {
            TssSelector = 0;
            Tss = ulValue as ULONG_PTR as PKTSS;
            if Tss.is_null() {
                crate::kdbp_print!(b"Invalid 32-bit TSS descriptor.\n\0");
                return 1u8;
            }
        } else {
            TssSelector = ulValue as USHORT;
            Tss = KdbpRetrieveTss(TssSelector, core::ptr::null_mut(), core::ptr::null_mut());
            if Tss.is_null() {
                crate::kdbp_print!(b"Invalid 32-bit TSS selector.\n\0");
                return 1u8;
            }
        }
    }

    if Tss.is_null() {
        TssSelector = Ke386GetTr();
        Tss = (*KeGetPcr()).TSS;
    }

    crate::kdbp_print!(b"%s TSS 0x%04x is at 0x%p.\n\0",
        if Tss == (*KeGetPcr()).TSS { b"Current\0".as_ptr() as PSTR } else { b"Specified\0".as_ptr() as PSTR },
        TssSelector as u32,
        Tss
    );
    crate::kdbp_print!(b"  Backlink:  0x%04x\n  Ss0:Esp0:  0x%04x:0x%08x\n  CR3:       0x%08x\n  EFlags:    0x%08x\n  Eax:       0x%08x\n  Ebx:       0x%08x\n  Ecx:       0x%08x\n  Edx:       0x%08x\n  Esi:       0x%08x\n  Edi:       0x%08x\n  Eip:       0x%08x\n  Esp:       0x%08x\n  Ebp:       0x%08x\n  Cs:        0x%04x\n  Ss:        0x%04x\n  Ds:        0x%04x\n  Es:        0x%04x\n  Fs:        0x%04x\n  Gs:        0x%04x\n  LDT:       0x%04x\n  Flags:     0x%04x\n  IoMapBase: 0x%04x\n\0",
        (*Tss).Backlink as u32, (*Tss).Ss0 as u32, (*Tss).Esp0, (*Tss).CR3, (*Tss).EFlags,
        (*Tss).Eax, (*Tss).Ebx, (*Tss).Ecx, (*Tss).Edx, (*Tss).Esi, (*Tss).Edi,
        (*Tss).Eip, (*Tss).Esp, (*Tss).Ebp,
        (*Tss).Cs as u32, (*Tss).Ss as u32, (*Tss).Ds as u32, (*Tss).Es as u32, (*Tss).Fs as u32, (*Tss).Gs as u32,
        (*Tss).LDT as u32, (*Tss).Flags as u32, (*Tss).IoMapBase as u32
    );

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdBugCheck(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {
    KdbpBugCheckRequested = 1u8;
    return 0u8;
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdReboot(
    Argc: ULONG,
    Argv: *mut PCHAR) -> BOOLEAN
{
    HalReturnToFirmware(_FIRMWARE_REENTRY_HalRebootRoutine);
    return 0u8;
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdDmesg(
    Argc: ULONG,
    Argv: *mut PCHAR
) -> BOOLEAN {
    let mut beg: ULONG;
    let end: ULONG;

    KdbpIsInDmesgMode = 1u8;
    if KdpDmesgBuffer.is_null() {
        crate::kdbp_print!(b"Dmesg: error, buffer is not allocated! /DEBUGPORT=SCREEN kernel param required for dmesg.\n\0");
        return 1u8;
    }

    crate::kdbp_print!(b"*** Dmesg *** TotalWritten=%lu, BufferSize=%lu, CurrentPosition=%lu\n\0",
              KdbDmesgTotalWritten, KdpDmesgBufferSize, KdpDmesgCurrentPosition);

    end = KdpDmesgCurrentPosition;
    beg = (end + KdpDmesgFreeBytes) % KdpDmesgBufferSize;

    if KdbDmesgTotalWritten <= KdpDmesgBufferSize {
        KdbpPager(KdpDmesgBuffer, KdpDmesgCurrentPosition);
    } else {
        KdbpPager(KdpDmesgBuffer.add(beg as usize), KdpDmesgBufferSize - beg);
        crate::kdbp_print!(b"*** Dmesg: buffer rollup ***\n\0");
        KdbpPager(KdpDmesgBuffer, end);
    }
    crate::kdbp_print!(b"*** Dmesg: end of output ***\n\0");

    KdbpIsInDmesgMode = 0u8;

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdSet(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {

    let mut l: LONG;
    let mut First: BOOLEAN;
    let mut pend: PCHAR = core::ptr::null_mut();
    let mut ConditionFirst: KDB_ENTER_CONDITION = KdbDoNotEnter;
    let mut ConditionLast: KDB_ENTER_CONDITION = KdbDoNotEnter;

    let ExceptionNames: [PCHAR; 21] = [
        b"ZERODEVIDE\0".as_ptr() as PCHAR,
        b"DEBUGTRAP\0".as_ptr() as PCHAR,
        b"NMI\0".as_ptr() as PCHAR,
        b"INT3\0".as_ptr() as PCHAR,
        b"OVERFLOW\0".as_ptr() as PCHAR,
        b"BOUND\0".as_ptr() as PCHAR,
        b"INVALIDOP\0".as_ptr() as PCHAR,
        b"NOMATHCOP\0".as_ptr() as PCHAR,
        b"DOUBLEFAULT\0".as_ptr() as PCHAR,
        b"RESERVED(9)\0".as_ptr() as PCHAR,
        b"INVALIDTSS\0".as_ptr() as PCHAR,
        b"SEGMENTNOTPRESENT\0".as_ptr() as PCHAR,
        b"STACKFAULT\0".as_ptr() as PCHAR,
        b"GPF\0".as_ptr() as PCHAR,
        b"PAGEFAULT\0".as_ptr() as PCHAR,
        b"RESERVED(15)\0".as_ptr() as PCHAR,
        b"MATHFAULT\0".as_ptr() as PCHAR,
        b"ALIGNMENTCHECK\0".as_ptr() as PCHAR,
        b"MACHINECHECK\0".as_ptr() as PCHAR,
        b"SIMDFAULT\0".as_ptr() as PCHAR,
        b"OTHERS\0".as_ptr() as PCHAR,
    ];

    if Argc == 1 {
        crate::kdbp_print!(b"Available settings:\n\0");
        crate::kdbp_print!(b"  syntax [intel|at&t]\n\0");
        crate::kdbp_print!(b"  condition [exception|*] [first|last] [never|always|kmode|umode]\n\0");
        crate::kdbp_print!(b"  break_on_module_load [true|false]\n\0");
    } else if strcmp(*Argv.offset(1), b"syntax\0".as_ptr() as PCHAR) == 0 {
        if Argc == 2 {
            crate::kdbp_print!(b"syntax = %s\n\0",
                if KdbUseIntelSyntax != 0 { b"intel\0".as_ptr() } else { b"at&t\0".as_ptr() } as PCHAR,
            );
        } else if Argc >= 3 {
            if _stricmp(*Argv.offset(2), b"intel\0".as_ptr() as PCHAR) == 0 {
                KdbUseIntelSyntax = 1u8;
            } else if _stricmp(*Argv.offset(2), b"at&t\0".as_ptr() as PCHAR) == 0 {
                KdbUseIntelSyntax = 0u8;
            } else {
                crate::kdbp_print!(b"Unknown syntax '%s'.\n\0", *Argv.offset(2));
            }
        }
    } else if strcmp(*Argv.offset(1), b"condition\0".as_ptr() as PCHAR) == 0 {
        if Argc == 2 {
            crate::kdbp_print!(b"Conditions:                 (First)  (Last)\n\0");
            l = 0;
            while l < (21 - 1) {
                if ExceptionNames[l as usize].is_null() {
                    l += 1;
                    continue;
                }

                if KdbpGetEnterCondition(l, 1u8, &mut ConditionFirst) == 0 {
                    ntkrnl_sys::ASSERT!(false);
                }

                if KdbpGetEnterCondition(l, 0u8, &mut ConditionLast) == 0 {
                    ntkrnl_sys::ASSERT!(false);
                }

                crate::kdbp_print!(b"  #%02d  %-20s %-8s %-8s\n\0",
                    l,
                    ExceptionNames[l as usize],
                    KDB_ENTER_CONDITION_TO_STRING(ConditionFirst),
                    KDB_ENTER_CONDITION_TO_STRING(ConditionLast),
                );
                l += 1;
            }

            ntkrnl_sys::ASSERT!(l == (21 - 1));
            crate::kdbp_print!(b"       %-20s %-8s %-8s\n\0",
                ExceptionNames[l as usize],
                KDB_ENTER_CONDITION_TO_STRING(ConditionFirst),
                KDB_ENTER_CONDITION_TO_STRING(ConditionLast),
            );
        } else {
            if Argc >= 5 && strcmp(*Argv.offset(2), b"*\0".as_ptr() as PCHAR) == 0 {
                l = -1;
            } else {
                l = strtoul(*Argv.offset(2), &mut pend, 0) as LONG;

                if *Argv.offset(2) == pend {
                    l = 0;
                    while l < 21 {
                        if ExceptionNames[l as usize].is_null() {
                            l += 1;
                            continue;
                        }

                        if _stricmp(ExceptionNames[l as usize], *Argv.offset(2)) == 0 {
                            break;
                        }
                        l += 1;
                    }
                }

                if l >= 21 {
                    crate::kdbp_print!(b"Unknown exception '%s'.\n\0", *Argv.offset(2));
                    return 1u8;
                }
            }

            if Argc > 4 {
                if _stricmp(*Argv.offset(3), b"first\0".as_ptr() as PCHAR) == 0 {
                    First = 1u8;
                } else if _stricmp(*Argv.offset(3), b"last\0".as_ptr() as PCHAR) == 0 {
                    First = 0u8;
                } else {
                    crate::kdbp_print!(b"set condition: second argument must be 'first' or 'last'\n\0");
                    return 1u8;
                }

                if _stricmp(*Argv.offset(4), b"never\0".as_ptr() as PCHAR) == 0 {
                    ConditionFirst = KdbDoNotEnter;
                } else if _stricmp(*Argv.offset(4), b"always\0".as_ptr() as PCHAR) == 0 {
                    ConditionFirst = KdbEnterAlways;
                } else if _stricmp(*Argv.offset(4), b"umode\0".as_ptr() as PCHAR) == 0 {
                    ConditionFirst = KdbEnterFromUmode;
                } else if _stricmp(*Argv.offset(4), b"kmode\0".as_ptr() as PCHAR) == 0 {
                    ConditionFirst = KdbEnterFromKmode;
                } else {
                    crate::kdbp_print!(b"set condition: third argument must be 'never', 'always', 'umode' or 'kmode'\n\0");
                    return 1u8;
                }

                if KdbpSetEnterCondition(l, First, ConditionFirst) == 0 {
                    if l >= 0 {
                        crate::kdbp_print!(b"Couldn't change condition for exception #%02d\n\0", l);
                    } else {
                        crate::kdbp_print!(b"Couldn't change condition for all exceptions\n\0");
                    }
                }
            } else {
                if KdbpGetEnterCondition(l, 1u8, &mut ConditionFirst) == 0 {
                    ntkrnl_sys::ASSERT!(false);
                }

                if KdbpGetEnterCondition(l, 0u8, &mut ConditionLast) == 0 {
                    ntkrnl_sys::ASSERT!(false);
                }

                if l < (21 - 1) {
                    crate::kdbp_print!(b"Condition for exception #%02d (%s): FirstChance %s  LastChance %s\n\0",
                        l,
                        ExceptionNames[l as usize],
                        KDB_ENTER_CONDITION_TO_STRING(ConditionFirst),
                        KDB_ENTER_CONDITION_TO_STRING(ConditionLast),
                    );
                } else {
                    crate::kdbp_print!(b"Condition for all other exceptions: FirstChance %s  LastChance %s\n\0",
                        KDB_ENTER_CONDITION_TO_STRING(ConditionFirst),
                        KDB_ENTER_CONDITION_TO_STRING(ConditionLast),
                    );
                }
            }
        }
    } else if strcmp(*Argv.offset(1), b"break_on_module_load\0".as_ptr() as PCHAR) == 0 {
        if Argc == 2 {
            crate::kdbp_print!(b"break_on_module_load = %s\n\0",
                if KdbBreakOnModuleLoad != 0 { b"enabled\0".as_ptr() } else { b"disabled\0".as_ptr() } as PCHAR,
            );
        } else if Argc >= 3 {
            if _stricmp(*Argv.offset(2), b"enable\0".as_ptr() as PCHAR) == 0
                || _stricmp(*Argv.offset(2), b"enabled\0".as_ptr() as PCHAR) == 0
                || _stricmp(*Argv.offset(2), b"true\0".as_ptr() as PCHAR) == 0
            {
                KdbBreakOnModuleLoad = 1u8;
            } else if _stricmp(*Argv.offset(2), b"disable\0".as_ptr() as PCHAR) == 0
                || _stricmp(*Argv.offset(2), b"disabled\0".as_ptr() as PCHAR) == 0
                || _stricmp(*Argv.offset(2), b"false\0".as_ptr() as PCHAR) == 0
            {
                KdbBreakOnModuleLoad = 0u8;
            } else {
                crate::kdbp_print!(b"Unknown setting '%s'.\n\0", *Argv.offset(2));
            }
        }
    } else {
        crate::kdbp_print!(b"Unknown setting '%s'.\n\0", *Argv.offset(1));
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCmdHelp(
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {
    let mut i: ULONG;

    crate::kdbp_print!(b"Kernel debugger commands:\n\0");
    i = 0;
    while (i as usize) < (*(&raw const KdbDebuggerCommands)).len() {
        if KdbDebuggerCommands[i as usize].Syntax.is_null() {
            if i > 0 {
                crate::kdbp_print!(b"\n\0");
            }

            crate::kdbp_print!(b"\x1b[7m* %s:\x1b[0m\n\0", KdbDebuggerCommands[i as usize].Help);
            i += 1;
            continue;
        }

        crate::kdbp_print!(b"  %-20s - %s\n\0",
                  KdbDebuggerCommands[i as usize].Syntax,
                  KdbDebuggerCommands[i as usize].Help);
        i += 1;
    }

    1u8
}

unsafe fn memrchr(s: *const core::ffi::c_void, c: core::ffi::c_int, n: usize) -> *mut core::ffi::c_void {
    let mut cp: *const u8;
    let mut n = n;

    if n != 0 {
        cp = (s as *const u8).add(n);
        loop {
            cp = cp.sub(1);
            if *cp == c as u8 {
                return cp as *mut core::ffi::c_void;
            }
            n -= 1;
            if n == 0 {
                break;
            }
        }
    }
    core::ptr::null_mut()
}

unsafe fn CountOnePageUp(
    Buffer: PCCH,
    BufLength: u32,
    pCurPos: PCCH,
    TermSize: *const SIZE
) -> *mut i8 {
    let p: PCCH;
    let p0len = (*TermSize).cx * (*TermSize).cy;
    let p0 = pCurPos.offset(-(p0len as isize));
    let mut prev_p = p0;
    let mut p1: PCCH;
    let mut j: u32;

    let mut pCurPos = pCurPos;
    if pCurPos < Buffer {
        pCurPos = Buffer;
    }

    p = memrchr(p0 as *const core::ffi::c_void, b'\n' as i32, p0len as usize) as PCCH;
    let mut p = if p.is_null() { p0 } else { p };
    
    j = (*TermSize).cy;
    while j > 0 {
        j -= 1;
        let linesCnt: i32;
        p1 = memrchr(p0 as *const core::ffi::c_void, b'\n' as i32, p.offset_from(p0) as usize) as PCCH;
        prev_p = p;
        p = p1;
        if p.is_null() {
            p = prev_p;
            if p.is_null() {
                p = p0;
            }
            break;
        }
        linesCnt = ((*TermSize).cx as isize + prev_p.offset_from(p) - 2) as i32 / (*TermSize).cx as i32;
        if linesCnt > 1 {
            j -= (linesCnt - 1) as u32;
        }
    }

    p = p.offset(1);
    p as *mut i8
}

unsafe fn KdpFilterEscapes(String: PSTR) {

    let mut p: *mut i8;
    let mut i: usize;
    let mut len: usize;

    loop {
        p = strrchr(String as *const i8, 0x1b);
        if p.is_null() {
            break;
        }

        len = strlen(p);
        if *p.offset(1) == b'[' as i8 {
            i = 2;
            while isalpha(*p.offset(i as isize) as i32) == 0 {
                i += 1;
            }
            i += 1;
            memmove(p as *mut u8, p.offset(i as isize) as *const u8, len + 1 - i);
        } else {
            memmove(p as *mut u8, p.offset(1) as *const u8, len);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn KdbpPagerInternal(
    Buffer: PCHAR,
    BufLength: ULONG,
    DoPage: BOOLEAN
) {

    static mut TerminalInitialized: BOOLEAN = 0u8;
    let mut c: CHAR;
    let mut ScanCode: ULONG = 0;
    let mut p: PCHAR;
    let mut i: usize;
    let mut RowsPrintedByTerminal: LONG;

    if BufLength == 0 {
        return;
    }

    if KdbOutputAborted != 0 {
        return;
    }

    if TerminalInitialized == 0 {
        TerminalInitialized = 1u8;
        KdpInitTerminal();
    }

    if KdbNumberOfRowsPrinted == 0 {
        KdpUpdateTerminalSize(core::ptr::addr_of_mut!(KdTermSize));
    }

    p = Buffer;
    while *p != 0 {
        if DoPage != 0 {
            if p > Buffer.offset(BufLength as isize) {
                crate::kdb_printf!(b"Dmesg: error, p > Buffer+BufLength,d=%d\0", p.offset_from(Buffer.offset(BufLength as isize)));
                return;
            }
        }
        i = strcspn(p as *const i8, b"\n\0".as_ptr() as *const i8);

        if DoPage != 0 {
            if p.offset(i as isize) > Buffer.offset(BufLength as isize) {
                break;
            }
        }

        if i > 0 {
            RowsPrintedByTerminal = ((i as ULONG + KdbNumberOfColsPrinted - 1) / KdTermSize.cx) as LONG;
        } else {
            RowsPrintedByTerminal = 0;
        }

        if *p.offset(i as isize) == b'\n' as i8 {
            RowsPrintedByTerminal += 1;
        }

        if KdTermSize.cy > 0 &&
            (KdbNumberOfRowsPrinted as LONG + RowsPrintedByTerminal) >= KdTermSize.cy as LONG
        {
            let Prompt: PCSTR;

            KdbRepeatLastCommand = 0u8;

            if KdbNumberOfColsPrinted > 0 {
                KdbPuts(b"\n\0".as_ptr() as PCSTR);
            }

            if DoPage != 0 {
                Prompt = b"--- Press q to abort, e/End,h/Home,u/PgUp, other key/PgDn ---\0".as_ptr() as PCSTR;
            } else {
                Prompt = b"--- Press q to abort, any other key to continue ---\0".as_ptr() as PCSTR;
            }

            KdbPuts(Prompt);
            c = KdpReadTermKey(core::ptr::addr_of_mut!(ScanCode));
            if DoPage != 0 {
                crate::kdb_printf!(b" '%c'/scan=%04x\n\0", c as i32, ScanCode);
            } else {
                KdbPuts(b"\n\0".as_ptr() as PCSTR);
            }

            RowsPrintedByTerminal += 1;

            if c == b'q' as i8 {
                KdbOutputAborted = 1u8;
                return;
            }

            if DoPage != 0 {
                if ScanCode == KEYSC_END || c == b'e' as i8 {
                    let pBufEnd = Buffer.offset(BufLength as isize);
                    p = CountOnePageUp(Buffer, BufLength, pBufEnd, core::ptr::addr_of!(KdTermSize));
                    i = strcspn(p as *const i8, b"\n\0".as_ptr() as *const i8);
                } else if ScanCode == KEYSC_PAGEUP ||
                          ScanCode == KEYSC_ARROWUP || c == b'u' as i8
                {
                    p = CountOnePageUp(Buffer, BufLength, p, core::ptr::addr_of!(KdTermSize));
                    i = strcspn(p as *const i8, b"\n\0".as_ptr() as *const i8);
                } else if ScanCode == KEYSC_HOME || c == b'h' as i8 {
                    p = Buffer;
                    i = strcspn(p as *const i8, b"\n\0".as_ptr() as *const i8);
                }
            }

            KdbNumberOfRowsPrinted = 0;
            KdbNumberOfColsPrinted = 0;
        }

        if *p.offset(i as isize) == b'\n' as i8 && *p.offset((i + 1) as isize) != 0 {
            c = *p.offset((i + 1) as isize);
            *p.offset((i + 1) as isize) = 0;
        } else {
            c = 0;
        }

        if KdTermConnected == 0 {
            KdpFilterEscapes(p);
        }

        KdbPuts(p as PCSTR);

        if c != 0 {
            *p.offset((i + 1) as isize) = c;
        }

        p = p.offset(i as isize);
        if *p == b'\n' as i8 {
            p = p.offset(1);
            KdbNumberOfColsPrinted = 0;
        } else {
            KdbNumberOfColsPrinted += i as ULONG;
        }

        KdbNumberOfRowsPrinted = (KdbNumberOfRowsPrinted as LONG + RowsPrintedByTerminal) as ULONG;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdbpPager(
    Buffer: PCHAR,
    BufLength: ULONG
) {
    KdbpPagerInternal(Buffer, BufLength, 1u8);
}

// KdbpPrint is a C variadic — its definition lives in kdbg_rust_shims.c
// (Rust does not support variadic non-extern fn definitions on stable).
//
// unsafe fn KdbpPrint(Format: PSTR, ...) {
//     static mut Buffer: [CHAR; 4096] = [0; 4096];
//     let mut Length: ULONG;
//     let mut ap: va_list = core::mem::zeroed();
//     if KdbOutputAborted != 0 { return; }
//     va_start(core::ptr::addr_of_mut!(ap), Format);
//     Length = _vsnprintf(Buffer.as_mut_ptr(), core::mem::size_of_val(&Buffer) - 1, Format, ap) as ULONG;
//     Buffer[Length as usize] = 0;
//     va_end(ap);
//     KdbpPagerInternal(Buffer.as_mut_ptr(), Length, 0u8);
// }

unsafe fn KdbpPrintUnicodeString(
    String: PCUNICODE_STRING)
{
    let mut i: ULONG;

    if String.is_null() || (*String).Buffer.is_null()
    {
        crate::kdbp_print!(b"<NULL>\0");
        return;
    }

    i = 0;
    while i < (*String).Length as ULONG / core::mem::size_of::<WCHAR>() as ULONG
    {
        crate::kdbp_print!(b"%c\0", ((*(*String).Buffer.offset(i as isize)) as CHAR) as i32);
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdbRegisterCliCallback(
    Callback: PVOID,
    Deregister: BOOLEAN,
) -> BOOLEAN {
    let mut i: ULONG;

    i = 0;
    while i < (core::mem::size_of_val(&*(&raw const KdbCliCallbacks)) / core::mem::size_of::<PVOID>()) as ULONG {
        if Deregister != 0 {
            if KdbCliCallbacks[i as usize] == Callback {
                KdbCliCallbacks[i as usize] = core::ptr::null_mut();
                return 1u8;
            }
        } else {
            if KdbCliCallbacks[i as usize].is_null() {
                KdbCliCallbacks[i as usize] = Callback;
                return 1u8;
            }
        }
        i += 1;
    }

    0u8
}

unsafe fn KdbpInvokeCliCallbacks(
    Command: PCHAR,
    Argc: ULONG,
    Argv: *mut PCHAR,
) -> BOOLEAN {
    let mut i: ULONG;

    type KdbCliCallback = unsafe extern "C" fn(PCHAR, ULONG, *mut PCHAR) -> BOOLEAN;
    i = 0;
    while i < (core::mem::size_of_val(&*(&raw const KdbCliCallbacks)) / core::mem::size_of::<PVOID>()) as ULONG {
        let cb_ptr = KdbCliCallbacks[i as usize];
        if !cb_ptr.is_null() {
            let callback: KdbCliCallback = core::mem::transmute(cb_ptr);
            if callback(Command, Argc, Argv) != 0 {
                return 1u8;
            }
        }
        i += 1;
    }

    0u8
}

unsafe fn KdbpDoCommand(
    Command: PCHAR,
) -> BOOLEAN {

    let mut Continue: BOOLEAN = 1u8;
    let mut i: usize;
    let mut p: PCHAR;
    let mut Argc: ULONG;
    static mut Argv: [PCHAR; 256] = [core::ptr::null_mut(); 256];
    static mut OrigCommand: [CHAR; 1024] = [0; 1024];

    RtlStringCbCopyA((*(&raw mut OrigCommand)).as_mut_ptr() as PSTR, core::mem::size_of_val(&*(&raw const OrigCommand)), Command as PCSTR);

    Argc = 0;
    p = Command;

    loop {
        while *p == b'\t' as i8 || *p == b' ' as i8 {
            p = p.offset(1);
        }

        if *p == 0 {
            break;
        }

        i = strcspn(p as *const i8, b"\t \0".as_ptr() as *const i8);
        Argv[Argc as usize] = p;
        Argc += 1;
        p = p.offset(i as isize);
        if *p == 0 {
            break;
        }

        *p = 0;
        p = p.offset(1);
    }

    if Argc < 1 {
        return 1u8;
    }

    KdbNumberOfRowsPrinted = 0;
    KdbNumberOfColsPrinted = 0;
    KdbOutputAborted = 0u8;

    i = 0;
    while i < (*(&raw const KdbDebuggerCommands)).len() {
        if KdbDebuggerCommands[i].Name.is_null() {
            i += 1;
            continue;
        }

        if strcmp(KdbDebuggerCommands[i].Name as *const i8, Argv[0] as *const i8) == 0 {
            Continue = (KdbDebuggerCommands[i].Fn.unwrap())(Argc, (*(&raw mut Argv)).as_mut_ptr());
            KdbOutputAborted = 0u8;
            return Continue;
        }
        i += 1;
    }

    if KdbpInvokeCliCallbacks(Command, Argc, (*(&raw mut Argv)).as_mut_ptr()) != 0 {
        KdbOutputAborted = 0u8;
        return Continue;
    }

    crate::kdb_printf!(b"Command '%s' is unknown.\n\0", (*(&raw const OrigCommand)).as_ptr());

    KdbOutputAborted = 0u8;
    return Continue;
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCliMainLoop(
    EnteredOnSingleStep: BOOLEAN,
) {
    let mut Continue: BOOLEAN = 1u8;
    static mut Command: [CHAR; 1024] = [0; 1024];
    static mut LastCommand: [CHAR; 1024] = [0; 1024];

    KdbpDoCommand(b"bt\0".as_ptr() as PCHAR);

    if EnteredOnSingleStep != 0 {
        if KdbSymPrintAddress(
            KeGetContextPc(KdbCurrentTrapFrame) as PVOID,
            KdbCurrentTrapFrame,
        ) == 0
        {
            crate::kdb_printf!(b"<%p>\0",
                KeGetContextPc(KdbCurrentTrapFrame),
            );
        }

        KdbPuts(b": \0".as_ptr() as PCSTR);
        if KdbpDisassemble(KeGetContextPc(KdbCurrentTrapFrame), KdbUseIntelSyntax as ULONG) < 0 {
            KdbPuts(b"<INVALID>\0".as_ptr() as PCSTR);
        }
        KdbPuts(b"\n\0".as_ptr() as PCSTR);
    } else {
        KdbPuts(b"\nType \"help\" for a list of commands.\n\0".as_ptr() as PCSTR);
    }

    while Continue != 0 {
        let CmdLen: usize = KdbPrompt(
            b"\0".as_ptr() as PCSTR,
            (*(&raw mut Command)).as_mut_ptr(),
            core::mem::size_of_val(&*(&raw const Command)),
        );
        if CmdLen == 0 {
            continue;
        } else if CmdLen > 1 {
            KdbRepeatLastCommand = 1u8;
            RtlStringCbCopyA(
                (*(&raw mut LastCommand)).as_mut_ptr() as PSTR,
                core::mem::size_of_val(&*(&raw const LastCommand)),
                (*(&raw const Command)).as_ptr() as PCSTR,
            );

            KdbpCommandHistoryAppend((*(&raw const Command)).as_ptr() as PCSTR);
        } else if KdbRepeatLastCommand != 0 {
            RtlStringCbCopyA(
                (*(&raw mut Command)).as_mut_ptr() as PSTR,
                core::mem::size_of_val(&*(&raw const Command)),
                (*(&raw const LastCommand)).as_ptr() as PCSTR,
            );
        }

        Continue = KdbpDoCommand((*(&raw mut Command)).as_mut_ptr());
    }
}

#[no_mangle]
pub unsafe extern "C" fn KdbpCliInterpretInitFile() {

    let mut p1: PCHAR;
    let mut p2: PCHAR;

    p1 = InterlockedExchangePointer(
        core::ptr::addr_of_mut!(KdbInitFileBuffer) as *mut PVOID,
        core::ptr::null_mut(),
    ) as PCHAR;
    if p1.is_null() {
        return;
    }

    KdbPuts(b"KDB: Executing KDBinit file...\n\0".as_ptr() as PCSTR);
    while *p1.offset(0) != 0 {
        let i: usize = strcspn(p1 as *const i8, b"\r\n\0".as_ptr() as *const i8);
        if i > 0 {
            let c: CHAR = *p1.offset(i as isize);
            *p1.offset(i as isize) = 0;

            p2 = p1;
            while isspace(*p2.offset(0) as i32) != 0 {
                p2 = p2.offset(1);
            }

            if strncmp(p2 as *const u8, b"break\0".as_ptr(), 5) == 0
                && (*p2.offset(5) == 0 || isspace(*p2.offset(5) as i32) != 0)
            {
                KdbpCliMainLoop(0u8);
            } else if *p2.offset(0) != b'#' as i8 && *p2.offset(0) != 0 {
                KdbpDoCommand(p1);
            }

            *p1.offset(i as isize) = c;
        }

        p1 = p1.offset(i as isize);
        while *p1.offset(0) == b'\r' as i8 || *p1.offset(0) == b'\n' as i8 {
            p1 = p1.offset(1);
        }
    }
    KdbPuts(b"KDB: KDBinit executed\n\0".as_ptr() as PCSTR);
}

#[no_mangle]
pub unsafe extern "system" fn KdbpCliInit() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileStdInfo: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut FileSize: ULONG;
    let mut FileBuffer: PCHAR;

    if !KdbInitFileBuffer.is_null() {
        return STATUS_SUCCESS;
    }

    RtlInitUnicodeString(&mut FileName, w!("\\SystemRoot\\System32\\drivers\\etc\\KDBinit").as_ptr() as *const u16);
    
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut FileName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenFile(&mut hFile, FILE_READ_DATA | SYNCHRONIZE,
                        &mut ObjectAttributes, &mut Iosb, 0,
                        FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT |
                        FILE_NO_INTERMEDIATE_BUFFERING);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ZwQueryInformationFile(hFile, &mut Iosb,
                                    &mut FileStdInfo as *mut _ as *mut core::ffi::c_void,
                                    core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
                                    FileStandardInformation);
    if !NT_SUCCESS(Status) {
        ZwClose(hFile);
        return Status;
    }
    FileSize = FileStdInfo.EndOfFile.u.LowPart;

    FileBuffer = ExAllocatePool(NonPagedPool, (FileSize + 1) as _) as PCHAR;
    if FileBuffer.is_null() {
        ZwClose(hFile);
        return Status;
    }

    Status = ZwReadFile(hFile, core::ptr::null_mut(), None, core::ptr::null_mut(), &mut Iosb,
                        FileBuffer as *mut core::ffi::c_void, FileSize, core::ptr::null_mut(), core::ptr::null_mut());
    ZwClose(hFile);

    if !NT_SUCCESS(Status) && Status != STATUS_END_OF_FILE {
        ExFreePool(FileBuffer as *mut core::ffi::c_void);
        return Status;
    }

    FileSize = if FileSize < Iosb.Information as u32 { FileSize } else { Iosb.Information as u32 };
    *FileBuffer.add(FileSize as usize) = 0;

    InterlockedExchangePointer(&raw mut KdbInitFileBuffer as *mut _ as *mut *mut core::ffi::c_void, FileBuffer as *mut core::ffi::c_void);
    DbgBreakPointWithStatus(DBG_STATUS_CONTROL_C);
    InterlockedExchangePointer(&raw mut KdbInitFileBuffer as *mut _ as *mut *mut core::ffi::c_void, core::ptr::null_mut());

    ExFreePool(FileBuffer as *mut core::ffi::c_void);

    STATUS_SUCCESS
}

unsafe extern "C" fn KdbDebugPrint(
    String: PCCH,
    Length: ULONG)
{
    let mut OldIrql: KIRQL;
    let mut beg: ULONG;
    let mut end: ULONG;
    let mut num: ULONG;

    if KdbpIsInDmesgMode != 0 {
       return;
    }

    if KdpDmesgBuffer == core::ptr::null_mut() {
        return;
    }

    OldIrql = KdbpAcquireLock(core::ptr::addr_of_mut!(KdpDmesgLogSpinLock));

    beg = KdpDmesgCurrentPosition;
    num = core::cmp::min(Length, KdpDmesgFreeBytes);
    if num != 0 {
        end = (beg + num) % KdpDmesgBufferSize;
        if end > beg {
            RtlCopyMemory((KdpDmesgBuffer as usize + beg as usize) as *mut core::ffi::c_void, String as *const core::ffi::c_void, Length as usize);
        } else {
            RtlCopyMemory((KdpDmesgBuffer as usize + beg as usize) as *mut core::ffi::c_void, String as *const core::ffi::c_void, (KdpDmesgBufferSize - beg) as usize);
            RtlCopyMemory(KdpDmesgBuffer as *mut core::ffi::c_void, (String as usize + (KdpDmesgBufferSize - beg) as usize) as *const core::ffi::c_void, end as usize);
        }
        KdpDmesgCurrentPosition = end;

        KdbDmesgTotalWritten += num;
    }

    KdbpReleaseLock(core::ptr::addr_of_mut!(KdpDmesgLogSpinLock), OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn KdbInitialize(
    DispatchTable: PKD_DISPATCH_TABLE,
    BootPhase: ULONG,
) -> NTSTATUS {
    static mut LoadSymbols: ULONG = 0;

    if BootPhase == 0 {
        (*DispatchTable).KdpPrintRoutine = Some(KdbDebugPrint);

        if !KeLoaderBlock.is_null() && !(*KeLoaderBlock).LoadOptions.is_null() {
            KdbpGetCommandLineSettings((*KeLoaderBlock).LoadOptions);
        }

        (*DispatchTable).KdpInitRoutine = Some(KdbInitialize);
        InsertTailList(&raw mut KdProviders, &mut (*DispatchTable).KdProvidersList);
    } else if BootPhase == 1 {
        (*DispatchTable).KdpInitRoutine = Some(KdbInitialize);

        KdpDmesgBuffer = ExAllocatePoolZero(
            NonPagedPool,
            (KdpDmesgBufferSize + 1) as _,
            TAG_KDBG,
        ) as PCHAR;
        KdpDmesgFreeBytes = KdpDmesgBufferSize;
        KdbDmesgTotalWritten = 0;

        KeInitializeSpinLock(core::ptr::addr_of_mut!(KdpDmesgLogSpinLock) as *mut _);
    }

    if BootPhase <= 1 {
        LoadSymbols <<= 1;
        LoadSymbols |= KdbSymInit(BootPhase) as ULONG;
    }

    if BootPhase == 1 {
        let mut buffer: [CHAR; 60] = [0; 60];
        crate::rtl_string_cb_printf_a!(
            buffer.as_mut_ptr(),
            60,
            b"   KDBG debugger enabled - %s\r\n",
            if (LoadSymbols & 0x2) == 0 {
                b"No symbols loaded\0".as_ptr() as PCHAR
            } else if (LoadSymbols & 0x1) == 0 {
                b"Kernel symbols loaded\0".as_ptr() as PCHAR
            } else {
                b"Loading symbols\0".as_ptr() as PCHAR
            },
        );
        HalDisplayString(buffer.as_mut_ptr());
    }

    if BootPhase >= 2 {
        let Status: NTSTATUS = KdbpCliInit();

        if Status == STATUS_OBJECT_NAME_NOT_FOUND || Status == STATUS_OBJECT_PATH_NOT_FOUND {
            (*DispatchTable).KdpInitRoutine = Some(KdbInitialize);
        }
    }

    STATUS_SUCCESS
}
