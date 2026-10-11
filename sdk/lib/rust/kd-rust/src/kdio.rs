//! kd/kdio.rs — NT Kernel Debugger I/O. Translated from ntoskrnl/kd/kdio.c.
//!
//! Owns globals: KdpDebugMode, KdProviders, DispatchTable, InitRoutines,
//! KdpLogFileName, SerialPortNumber, SerialPortInfo, plus file-private state.
//! Sibling-owned globals (KdbDebugState by kdterminal.rs) are referenced via
//! extern decl.

use va_slots_rust::format_slots;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::{PACKET_TYPE_KD_ACKNOWLEDGE, PACKET_TYPE_KD_RESET};
use ntkrnl_sys::globals::KdpContext;
use ntkrnl_sys::ddk::{
    self,
    HANDLE,
    ExAllocatePoolZero, ExFreePoolWithTag, HalDisplayString,
    InbvAcquireDisplayOwnership, InbvCheckDisplayOwnership, InbvEnableDisplayString,
    InbvInstallDisplayStringFilter, InbvIsBootDriverInstalled, InbvNotifyDisplayOwnershipLost,
    InbvResetDisplay, InbvSetScrollRegion, InbvSetTextColor, InbvSolidColorFill,
    KeInitializeEvent, KeInitializeSpinLock, KeSetEvent, KeTestSpinLock,
    KeTryToAcquireSpinLockAtDpcLevel, KeWaitForSingleObject, KiReleaseSpinLock,
    NtWriteFile, PsCreateSystemThread,
    RtlAnsiStringToUnicodeString, RtlFreeUnicodeString,
    ZwClose, ZwCreateFile, ZwQueryInformationFile, ZwSetInformationFile,
    ZwSetInformationThread,
    PCCH, PCSTR, PCONTEXT,
    PSTRING, _IO_STATUS_BLOCK, _OBJECT_ATTRIBUTES, _UNICODE_STRING,
    DBGKD_DEBUG_IO, PDBGKD_ANY_WAIT_STATE_CHANGE, PDBGKD_DEBUG_IO,
    PDBGKD_MANIPULATE_STATE64, PEXCEPTION_RECORD64, PKD_CONTEXT,
    KSPIN_LOCK, KEVENT, KPRIORITY, ANSI_STRING, CPPORT, _LIST_ENTRY,
    KeGetCurrentPrcb,
    _EVENT_TYPE_SynchronizationEvent,
    _FILE_INFORMATION_CLASS_FilePositionInformation,
    _FILE_INFORMATION_CLASS_FileStandardInformation,
    _THREADINFOCLASS_ThreadPriority,
    _POOL_TYPE_NonPagedPool,
};

// ============================================================================
// Constants
// ============================================================================

pub const KdpBufferSize: ULONG = 1024 * 512;
pub const KdpScreenLineLengthDefault: ULONG = 80;

pub const STATUS_PORT_DISCONNECTED: NTSTATUS = 0xC0000037u32 as i32;
pub const STATUS_NO_MEMORY: NTSTATUS = 0xC0000017u32 as i32;
pub const STATUS_DEVICE_DOES_NOT_EXIST: NTSTATUS = 0xC00000C0u32 as i32;
pub const STATUS_OBJECT_NAME_NOT_FOUND: NTSTATUS = 0xC0000034u32 as i32;
pub const STATUS_OBJECT_PATH_NOT_FOUND: NTSTATUS = 0xC000003Au32 as i32;
pub const STATUS_BREAKPOINT: NTSTATUS = 0x80000003u32 as i32;
pub const STATUS_SUCCESS: NTSTATUS = 0;
pub const DBG_EXCEPTION_NOT_HANDLED: NTSTATUS = 0x80010001u32 as i32;

pub const FILE_APPEND_DATA: ULONG = 0x00000004;
pub const SYNCHRONIZE: ULONG = 0x00100000;
pub const FILE_ATTRIBUTE_NORMAL: ULONG = 0x00000080;
pub const FILE_SHARE_READ: ULONG = 0x00000001;
pub const FILE_OPEN_IF: ULONG = 0x00000003;
pub const FILE_NON_DIRECTORY_FILE: ULONG = 0x00000040;
pub const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x00000020;
pub const FILE_SEQUENTIAL_ONLY: ULONG = 0x00000004;
pub const FILE_WRITE_THROUGH: ULONG = 0x00000002;

pub const OBJ_CASE_INSENSITIVE: ULONG = 0x00000040;
pub const OBJ_KERNEL_HANDLE: ULONG = 0x00000200;
pub const THREAD_ALL_ACCESS: ULONG = 0x001F03FF;
pub const HIGH_PRIORITY: KPRIORITY = 31;

pub const TAG_KDBG: ULONG = u32::from_le_bytes(*b"KDBG");

pub const PACKET_TYPE_KD_STATE_CHANGE32: ULONG = 1;
pub const PACKET_TYPE_KD_STATE_MANIPULATE: ULONG = 2;
pub const PACKET_TYPE_KD_DEBUG_IO: ULONG = 3;
pub const PACKET_TYPE_KD_STATE_CHANGE64: ULONG = 7;
pub const PACKET_TYPE_KD_POLL_BREAKIN: ULONG = 8;

pub const DbgKdExceptionStateChange: ULONG = 0x00003030;
pub const DbgKdLoadSymbolsStateChange: ULONG = 0x00003031;
pub const DbgKdContinueApi: ULONG = 0x00003136;
pub const DbgKdPrintStringApi: ULONG = 0x00003230;
pub const DbgKdGetStringApi: ULONG = 0x00003231;

pub const KdPacketReceived: KDSTATUS = 0;
pub const KdPacketTimedOut: KDSTATUS = 1;
pub const KdPacketNeedsResend: KDSTATUS = 2;

pub const HIGH_LEVEL_IRQL: KIRQL = {
    #[cfg(target_arch = "x86")]
    { 31u8 }
    #[cfg(target_arch = "x86_64")]
    { 15u8 }
    #[cfg(target_arch = "aarch64")]
    { 15u8 }
};

pub const DISPATCH_LEVEL_IRQL: KIRQL = 2;

// kdterminal.h: KD_DEBUG_KDSERIAL = 1, KD_DEBUG_KDNOECHO = 2
pub const KD_DEBUG_KDSERIAL: ULONG = 1;
pub const KD_DEBUG_KDNOECHO: ULONG = 2;

// inbv.h
pub const SCREEN_WIDTH: u32 = 640;
pub const SCREEN_HEIGHT: u32 = 480;
pub const BV_COLOR_BLACK: u32 = 0;
pub const BV_COLOR_WHITE: u32 = 15;

// kd.h serial defaults
pub const DEFAULT_DEBUG_PORT: ULONG = 2;
pub const DEFAULT_DEBUG_BAUD_RATE: ULONG = 115200;

// ============================================================================
// KD types — kdmain.rs is the canonical owner. Re-import from sibling.
// ============================================================================

pub type KDSTATUS = u32;

// kdmain.rs defines: KDP_DEBUG_MODE { Screen:u8, Serial:u8, File:u8, Value:u32 },
// _KD_DISPATCH_TABLE { KdProvidersList, KdpInitRoutine, KdpPrintRoutine, InitStatus },
// KD_DISPATCH_TABLE = _KD_DISPATCH_TABLE, PKD_DISPATCH_TABLE = *mut _KD_DISPATCH_TABLE,
// PKDP_INIT_ROUTINE = Option<extern "system" fn>, PKDP_PRINT_ROUTINE = Option<...>,
// KdMax = 3, plus the globals KdpDebugMode/SerialPortNumber/SerialPortInfo/
// KdpLogFileName/KdProviders/DispatchTable/InitRoutines/KdbDebugState.
pub use super::kdmain::{
    KDP_DEBUG_MODE, _KD_DISPATCH_TABLE, KD_DISPATCH_TABLE, PKD_DISPATCH_TABLE,
    PKDP_INIT_ROUTINE, PKDP_PRINT_ROUTINE, KdMax,
};

// ============================================================================
// Cross-file extern decls — globals owned by kdmain.rs / kdterminal.rs / HAL
// ============================================================================

// HAL exports `KdComPortInUse` via `__declspec(dllimport)`. PE
// cross-DLL imports go through the IAT — `__imp_<name>` is a slot
// holding the address of the imported symbol. Set the imported value
// by loading the IAT pointer and dereferencing it.
//
// The C header trick (`__declspec(dllimport)` + assignment) compiles
// to exactly that. Rust `extern "C" { static X }` with `#[link_name]`
// can't be used for the IAT entry on i386 because rustc auto-prefixes
// `_` to the symbol name (gives `___imp__X` instead of `__imp__X`).
// Inline asm references the IAT slot literally and dodges the rustc
// name-mangling rule — same pattern as `_KeRaiseIrqlToSynchLevel@0`
// in ke_rust_shims.c.
#[inline]
unsafe fn KdpRustSetComPortInUse(port: PUCHAR) {
    #[cfg(target_arch = "x86")]
    {
        core::arch::asm!(
            "mov eax, dword ptr [__imp__KdComPortInUse]",
            "mov dword ptr [eax], {port}",
            port = in(reg) port,
            out("eax") _,
            options(nostack, preserves_flags),
        );
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!(
            "mov rax, qword ptr [rip + __imp_KdComPortInUse]",
            "mov qword ptr [rax], {port}",
            port = in(reg) port,
            out("rax") _,
            options(nostack, preserves_flags),
        );
    }
    #[cfg(target_arch = "aarch64")]
    {
        let iat: *mut *mut u8;
        core::arch::asm!(
            "adrp {iat}, __imp_KdComPortInUse",
            "add  {iat}, {iat}, :lo12:__imp_KdComPortInUse",
            "ldr  {iat}, [{iat}]",
            iat = out(reg) iat,
            options(nostack, preserves_flags),
        );
        *iat = port;
    }
}

extern "C" {
    /// xdk/kdfuncs.h: BOOLEAN KdDebuggerNotPresent. KD_DEBUGGER_NOT_PRESENT alias.
    pub static mut KdDebuggerNotPresent: BOOLEAN;

    /// SharedUserData — KUSER_SHARED_DATA at fixed kernel VA.
    pub static mut SharedUserData: *mut ddk::_KUSER_SHARED_DATA;

    /// IRQL APIs — exported as extern "C" by Rust shim_exports on i386,
    /// real bindgen extern "C" on amd64/arm64.

    /// CRT-style: msvcrt._snprintf for inline format helpers.
    fn _snprintf(buffer: *mut CHAR, count: usize, format: PCSTR, ...) -> i32;
    fn strlen(s: *const CHAR) -> usize;
}
unsafe extern "system" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
}

// Per kdio.c, this file owns: KdpDebugMode, SerialPortNumber, SerialPortInfo,
// KdpLogFileName, KdProviders, DispatchTable, InitRoutines.
// kdterminal.rs owns KdbDebugState (extern decl below).

#[no_mangle]
pub static mut KdpDebugMode: KDP_DEBUG_MODE = KDP_DEBUG_MODE {
    Screen: 0,
    Serial: 0,
    File: 0,
    Value: 0,
};

#[no_mangle]
pub static mut SerialPortNumber: ULONG = DEFAULT_DEBUG_PORT;

#[no_mangle]
pub static mut SerialPortInfo: CPPORT = CPPORT {
    Address: core::ptr::null_mut(),
    BaudRate: DEFAULT_DEBUG_BAUD_RATE,
    Flags: 0,
};

#[no_mangle]
pub static mut KdpLogFileName: ANSI_STRING = ANSI_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: core::ptr::null_mut(),
};

#[no_mangle]
pub static mut KdProviders: _LIST_ENTRY = _LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[no_mangle]
pub static mut DispatchTable: [KD_DISPATCH_TABLE; KdMax] = [KD_DISPATCH_TABLE {
    KdProvidersList: _LIST_ENTRY {
        Flink: core::ptr::null_mut(),
        Blink: core::ptr::null_mut(),
    },
    KdpInitRoutine: None,
    KdpPrintRoutine: None,
    InitStatus: 0,
}; KdMax];

#[no_mangle]
pub static mut InitRoutines: [PKDP_INIT_ROUTINE; KdMax] = [
    Some(KdpScreenInit),
    Some(KdpSerialInit),
    Some(KdpDebugLogInit),
];

extern "C" {
    pub static mut KdbDebugState: ULONG;
}

// File-private statics (not exported; no #[no_mangle]) — owned by kdio.rs.
static mut KdpLoggingEnabled: BOOLEAN = 0;
static mut KdpDebugBuffer: PCHAR = core::ptr::null_mut();
static mut KdpCurrentPosition: ULONG = 0;
static mut KdpFreeBytes: ULONG = 0;
static mut KdpDebugLogSpinLock: KSPIN_LOCK = 0;
static mut KdpLoggerThreadEvent: KEVENT = unsafe { core::mem::zeroed() };
static mut KdpLogFileHandle: HANDLE = core::ptr::null_mut();
static mut KdpSerialSpinLock: KSPIN_LOCK = 0;

/// CHAR KdpScreenLineBuffer[KdpScreenLineLengthDefault + 1]
static mut KdpScreenLineBuffer: [CHAR; (KdpScreenLineLengthDefault as usize) + 1] =
    [0; (KdpScreenLineLengthDefault as usize) + 1];
static mut KdpScreenLineBufferPos: ULONG = 0;
static mut KdpScreenLineLength: ULONG = 0;

extern "system" {
    /// Implemented by per-arch i386/kdserial.rs + arm64/kdserial.rs.
    fn KdPortInitializeEx(PortInformation: *mut CPPORT, ComPortNumber: ULONG) -> BOOLEAN;
    fn KdPortPutByteEx(PortInformation: *mut CPPORT, ByteToSend: UCHAR);

    /// rtl: BREAK_WITH_STATUS instruction marker (per-arch). NTAPI on i386
    /// → stdcall (_RtlpBreakWithStatusInstruction@0).
    pub fn RtlpBreakWithStatusInstruction();
}

// InsertTailList / RemoveEntryList — FORCEINLINE list ops in ntdef.h. Use the
// inline implementations from ntkrnl_sys::base (already imported via use *).
// (kdio.rs callers pass ddk::_LIST_ENTRY* — same layout as base::LIST_ENTRY.)

/// Alias C `KD_DEBUGGER_NOT_PRESENT` macro → KdDebuggerNotPresent extern.
#[allow(non_upper_case_globals)]
const _UNUSED_KD_DEBUGGER_NOT_PRESENT_MACRO: u8 = 0;

#[inline]
unsafe fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

/// RtlCopyMemory — C macro. Wraps copy_nonoverlapping for raw pointers.
#[inline]
unsafe fn RtlCopyMemory(dst: PVOID, src: *const core::ffi::c_void, len: SIZE_T) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

/// InterlockedExchangeAddUL — atomic add returning previous value (ULONG).
#[inline]
unsafe fn InterlockedExchangeAddUL(addend: *mut ULONG, value: ULONG) -> ULONG {
    use core::sync::atomic::{AtomicU32, Ordering};
    let atomic = &*(addend as *const AtomicU32);
    atomic.fetch_add(value, Ordering::SeqCst)
}

/// kdio_printf!() — replacement for the variadic KdIoPrintf. Formats into
/// a 512-byte stack buffer using _snprintf, then dispatches via KdIoPrintString.
/// Defining a Rust C-variadic function requires nightly, so we route formatting
/// through msvcrt's _snprintf and keep KdIoPrintf as a non-variadic extern.
macro_rules! kdio_printf {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        let mut __buf: [CHAR; 512] = [0; 512];
        let __ret = _snprintf(__buf.as_mut_ptr(), 512, $fmt $(, $arg)*);
        let __len: ULONG = if __ret < 0 { 0 } else { __ret as ULONG };
        KdIoPrintString(__buf.as_ptr(), __len);
    }};
}

// ============================================================================
// LOCKING FUNCTIONS
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn KdbpAcquireLock(SpinLock: *mut KSPIN_LOCK) -> KIRQL {
    let mut OldIrql: KIRQL = 0;

    extern "C" {
        static KeBugCheckActive: ULONG;
    }
    loop {
        // Spin while the lock is held (KeTestSpinLock returns FALSE while held).
        //
        // ...but not forever once a bugcheck is under way. The bugcheck freezes the
        // other CPUs, and amd64 delivers that freeze by NMI, which HIGH_LEVEL does not
        // mask: a CPU can be parked FOR GOOD while holding this lock, and every crash
        // print after it would spin here with the evidence unwritten. After a bounded
        // wait the bugchecking CPU takes the lock; interleaved output beats none.
        let mut spins: u64 = 0;
        while KeTestSpinLock(SpinLock) == 0 {
            spins += 1;
            if spins > 20_000_000 && core::ptr::read_volatile(core::ptr::addr_of!(KeBugCheckActive)) != 0 {
                KeRaiseIrql(HIGH_LEVEL_IRQL, &mut OldIrql);
                return OldIrql;
            }
            core::hint::spin_loop();
        }

        KeRaiseIrql(HIGH_LEVEL_IRQL, &mut OldIrql);

        if KeTryToAcquireSpinLockAtDpcLevel(SpinLock) != 0 {
            break;
        }

        KeLowerIrql(OldIrql);
    }

    OldIrql
}

#[no_mangle]
pub unsafe extern "system" fn KdbpReleaseLock(SpinLock: *mut KSPIN_LOCK, OldIrql: KIRQL) {
    KiReleaseSpinLock(SpinLock);
    KeLowerIrql(OldIrql);
}

// ============================================================================
// FILE DEBUG LOG FUNCTIONS
// ============================================================================

// PKSTART_ROUTINE calling convention: i386 uses stdcall, amd64/arm64 use C.
unsafe extern "system" fn KdpLoggerThread(_Context: PVOID) {
    KdpLoggerThreadBody()
}

unsafe fn KdpLoggerThreadBody() {
    let mut beg: ULONG;
    let mut end: ULONG;
    let mut num: ULONG;
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

    KdpLoggingEnabled = 1u8;

    loop {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(KdpLoggerThreadEvent) as PVOID,
            0u32,                         // Executive
            0i8,                          // KernelMode
            0u8,                          // !Alertable
            core::ptr::null_mut(),        // No timeout
        );

        end = KdpCurrentPosition;
        num = KdpFreeBytes;

        beg = end.wrapping_add(num) % KdpBufferSize;
        num = KdpBufferSize.wrapping_sub(num);

        if num == 0 {
            continue;
        }

        if end > beg {
            NtWriteFile(
                KdpLogFileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Iosb),
                KdpDebugBuffer.add(beg as usize) as PVOID,
                num,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        } else {
            NtWriteFile(
                KdpLogFileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Iosb),
                KdpDebugBuffer.add(beg as usize) as PVOID,
                KdpBufferSize.wrapping_sub(beg),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            NtWriteFile(
                KdpLogFileHandle,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(Iosb),
                KdpDebugBuffer as PVOID,
                end,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }

        InterlockedExchangeAddUL(core::ptr::addr_of_mut!(KdpFreeBytes), num);
    }
}

unsafe extern "system" fn KdpPrintToLogFile(String: PCCH, Length: ULONG) {
    if KdpDebugBuffer.is_null() {
        return;
    }

    let OldIrql: KIRQL = KdbpAcquireLock(core::ptr::addr_of_mut!(KdpDebugLogSpinLock));

    let beg: ULONG = KdpCurrentPosition;
    let num: ULONG = core::cmp::min(Length, KdpFreeBytes);
    if num != 0 {
        let end: ULONG = (beg + num) % KdpBufferSize;
        KdpCurrentPosition = end;
        KdpFreeBytes -= num;

        if end > beg {
            RtlCopyMemory(
                KdpDebugBuffer.add(beg as usize) as PVOID,
                String as *const core::ffi::c_void,
                num as SIZE_T,
            );
        } else {
            RtlCopyMemory(
                KdpDebugBuffer.add(beg as usize) as PVOID,
                String as *const core::ffi::c_void,
                (KdpBufferSize - beg) as SIZE_T,
            );
            RtlCopyMemory(
                KdpDebugBuffer as PVOID,
                String.add((KdpBufferSize - beg) as usize) as *const core::ffi::c_void,
                end as SIZE_T,
            );
        }
    }

    KdbpReleaseLock(core::ptr::addr_of_mut!(KdpDebugLogSpinLock), OldIrql);

    if OldIrql <= DISPATCH_LEVEL_IRQL && KdpLoggingEnabled != 0 {
        KeSetEvent(core::ptr::addr_of_mut!(KdpLoggerThreadEvent) as ddk::PRKEVENT, 0i32, 0u8);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpDebugLogInit(
    dispatch_table: PKD_DISPATCH_TABLE,
    BootPhase: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if KdpDebugMode.File == 0 {
        return STATUS_PORT_DISCONNECTED;
    }

    if BootPhase == 0 {
        (*dispatch_table).KdpPrintRoutine = Some(KdpPrintToLogFile);
        (*dispatch_table).KdpInitRoutine = Some(KdpDebugLogInit);
        InsertTailList(
            core::ptr::addr_of_mut!(KdProviders),
            core::ptr::addr_of_mut!((*dispatch_table).KdProvidersList),
        );
    } else if BootPhase == 1 {
        KdpDebugBuffer = ExAllocatePoolZero(
            _POOL_TYPE_NonPagedPool,
            KdpBufferSize as ddk::SIZE_T,
            TAG_KDBG,
        ) as PCHAR;
        if KdpDebugBuffer.is_null() {
            KdpDebugMode.File = 0u8;
            RemoveEntryList(core::ptr::addr_of_mut!((*dispatch_table).KdProvidersList));
            return STATUS_NO_MEMORY;
        }
        KdpFreeBytes = KdpBufferSize;

        KeInitializeSpinLock(core::ptr::addr_of_mut!(KdpDebugLogSpinLock));

        (*dispatch_table).KdpInitRoutine = Some(KdpDebugLogInit);

        HalDisplayString(b"   File log debugging enabled\r\n\0".as_ptr() as PCHAR);
    } else if BootPhase >= 2 {
        let mut FileName: _UNICODE_STRING = core::mem::zeroed();
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut ThreadHandle: HANDLE = core::ptr::null_mut();

        if !KdpLogFileHandle.is_null() {
            return STATUS_SUCCESS;
        }

        Status = RtlAnsiStringToUnicodeString(
            &mut FileName,
            core::ptr::addr_of_mut!(KdpLogFileName),
            1u8,
        );
        if !NT_SUCCESS(Status) {
            // Failure: drop our buffer and bail out.
            KdpFreeBytes = 0;
            ExFreePoolWithTag(KdpDebugBuffer as PVOID, TAG_KDBG);
            KdpDebugBuffer = core::ptr::null_mut();
            KdpDebugMode.File = 0u8;
            RemoveEntryList(core::ptr::addr_of_mut!((*dispatch_table).KdProvidersList));
            return Status;
        }

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.ObjectName = &mut FileName;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = ZwCreateFile(
            core::ptr::addr_of_mut!(KdpLogFileHandle),
            FILE_APPEND_DATA | SYNCHRONIZE,
            &mut ObjectAttributes,
            &mut Iosb,
            core::ptr::null_mut(),
            FILE_ATTRIBUTE_NORMAL,
            FILE_SHARE_READ,
            FILE_OPEN_IF,
            FILE_NON_DIRECTORY_FILE
                | FILE_SYNCHRONOUS_IO_NONALERT
                | FILE_SEQUENTIAL_ONLY
                | FILE_WRITE_THROUGH,
            core::ptr::null_mut(),
            0,
        );

        RtlFreeUnicodeString(&mut FileName);

        if !NT_SUCCESS(Status) {
            if Status == STATUS_OBJECT_NAME_NOT_FOUND
                || Status == STATUS_OBJECT_PATH_NOT_FOUND
            {
                (*dispatch_table).KdpInitRoutine = Some(KdpDebugLogInit);
                return Status;
            }
            // Failure cleanup
            KdpFreeBytes = 0;
            ExFreePoolWithTag(KdpDebugBuffer as PVOID, TAG_KDBG);
            KdpDebugBuffer = core::ptr::null_mut();
            KdpDebugMode.File = 0u8;
            RemoveEntryList(core::ptr::addr_of_mut!((*dispatch_table).KdProvidersList));
            return Status;
        }

        // HACK for FILE_APPEND_DATA — seek to EOF.
        {
            let mut FileInfo: ddk::FILE_STANDARD_INFORMATION = core::mem::zeroed();
            let mut FilePosInfo: ddk::FILE_POSITION_INFORMATION = core::mem::zeroed();

            Status = ZwQueryInformationFile(
                KdpLogFileHandle,
                &mut Iosb,
                &mut FileInfo as *mut _ as PVOID,
                core::mem::size_of::<ddk::FILE_STANDARD_INFORMATION>() as ULONG,
                _FILE_INFORMATION_CLASS_FileStandardInformation,
            );

            Status = ZwQueryInformationFile(
                KdpLogFileHandle,
                &mut Iosb,
                &mut FilePosInfo as *mut _ as PVOID,
                core::mem::size_of::<ddk::FILE_POSITION_INFORMATION>() as ULONG,
                _FILE_INFORMATION_CLASS_FilePositionInformation,
            );

            FilePosInfo.CurrentByteOffset.QuadPart = FileInfo.EndOfFile.QuadPart;
            Status = ZwSetInformationFile(
                KdpLogFileHandle,
                &mut Iosb,
                &mut FilePosInfo as *mut _ as PVOID,
                core::mem::size_of::<ddk::FILE_POSITION_INFORMATION>() as ULONG,
                _FILE_INFORMATION_CLASS_FilePositionInformation,
            );
            // Suppress unused warning on Status (logging printfs were here in C).
            let _ = Status;
        }

        KeInitializeEvent(
            core::ptr::addr_of_mut!(KdpLoggerThreadEvent) as ddk::PRKEVENT,
            _EVENT_TYPE_SynchronizationEvent,
            1u8,
        );

        Status = PsCreateSystemThread(
            &mut ThreadHandle,
            THREAD_ALL_ACCESS,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            Some(KdpLoggerThread),
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            ZwClose(KdpLogFileHandle);
            // Failure cleanup
            KdpFreeBytes = 0;
            ExFreePoolWithTag(KdpDebugBuffer as PVOID, TAG_KDBG);
            KdpDebugBuffer = core::ptr::null_mut();
            KdpDebugMode.File = 0u8;
            RemoveEntryList(core::ptr::addr_of_mut!((*dispatch_table).KdProvidersList));
            return Status;
        }

        let mut Priority: KPRIORITY = HIGH_PRIORITY;
        ZwSetInformationThread(
            ThreadHandle,
            _THREADINFOCLASS_ThreadPriority,
            &mut Priority as *mut _ as PVOID,
            core::mem::size_of::<KPRIORITY>() as ULONG,
        );

        ZwClose(ThreadHandle);
        return Status;
    }

    Status
}

// ============================================================================
// SERIAL FUNCTIONS
// ============================================================================

unsafe extern "system" fn KdpSerialPrint(String: PCCH, Length: ULONG) {
    // rtl-diag's port lock as well as our own: kernel-mode DebugPrint writes the same
    // UART directly, and without one lock shared by both paths a user-mode print and a
    // kernel print came out interleaved byte by byte (the OptiPlex's late-boot logs).
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    extern "C" {
        fn RtlpAcquireDebugPort(Flags: *mut usize) -> u8;
        fn RtlpReleaseDebugPort(Flags: usize, Held: u8);
    }
    let mut pch: PCCH = String;
    let OldIrql: KIRQL = KdbpAcquireLock(core::ptr::addr_of_mut!(KdpSerialSpinLock));
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let mut port_flags = 0usize;
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let port_held = RtlpAcquireDebugPort(&mut port_flags);

    while pch < String.add(Length as usize) && *pch != 0 {
        if *pch == b'\n' as CHAR {
            KdPortPutByteEx(core::ptr::addr_of_mut!(SerialPortInfo), b'\r');
        }
        KdPortPutByteEx(core::ptr::addr_of_mut!(SerialPortInfo), *pch as UCHAR);
        pch = pch.add(1);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    RtlpReleaseDebugPort(port_flags, port_held);
    KdbpReleaseLock(core::ptr::addr_of_mut!(KdpSerialSpinLock), OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn KdpSerialInit(
    dispatch_table: PKD_DISPATCH_TABLE,
    BootPhase: ULONG,
) -> NTSTATUS {
    if KdpDebugMode.Serial == 0 {
        return STATUS_PORT_DISCONNECTED;
    }

    if BootPhase == 0 {
        (*dispatch_table).KdpPrintRoutine = Some(KdpSerialPrint);

        if KdPortInitializeEx(core::ptr::addr_of_mut!(SerialPortInfo), SerialPortNumber) == 0 {
            KdpDebugMode.Serial = 0u8;
            return STATUS_DEVICE_DOES_NOT_EXIST;
        }
        KdpRustSetComPortInUse(SerialPortInfo.Address);

        KeInitializeSpinLock(core::ptr::addr_of_mut!(KdpSerialSpinLock));

        (*dispatch_table).KdpInitRoutine = Some(KdpSerialInit);
        InsertTailList(
            core::ptr::addr_of_mut!(KdProviders),
            core::ptr::addr_of_mut!((*dispatch_table).KdProvidersList),
        );
    } else if BootPhase == 1 {
        HalDisplayString(b"   Serial debugging enabled\r\n\0".as_ptr() as PCHAR);
    }

    STATUS_SUCCESS
}

// ============================================================================
// SCREEN FUNCTIONS
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn KdpScreenAcquire() {
    if InbvIsBootDriverInstalled() != 0 {
        InbvAcquireDisplayOwnership();
        InbvResetDisplay();
        InbvSolidColorFill(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, BV_COLOR_BLACK);
        InbvSetTextColor(BV_COLOR_WHITE);
        InbvInstallDisplayStringFilter(None);
        InbvEnableDisplayString(1u8);
        InbvSetScrollRegion(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn KdpScreenRelease() {
    if InbvIsBootDriverInstalled() != 0 && InbvCheckDisplayOwnership() != 0 {
        InbvNotifyDisplayOwnershipLost(None);
    }
}

unsafe extern "system" fn KdpScreenPrint(String: PCCH, Length: ULONG) {
    let mut pch: PCCH = String;

    while pch < String.add(Length as usize) && *pch != 0 {
        if *pch == b'\x08' as CHAR {
            // Backspace: rewind buffer and reprint line.
            if KdpScreenLineLength > 0 {
                KdpScreenLineLength -= 1;
                KdpScreenLineBuffer[KdpScreenLineLength as usize] = 0;
                KdpScreenLineBufferPos = KdpScreenLineLength;

                HalDisplayString(b"\r\0".as_ptr() as PCHAR);
                HalDisplayString((*(&raw mut KdpScreenLineBuffer)).as_mut_ptr());
            }
        } else {
            KdpScreenLineBuffer[KdpScreenLineLength as usize] = *pch;
            KdpScreenLineLength += 1;
            KdpScreenLineBuffer[KdpScreenLineLength as usize] = 0;
        }

        if *pch == b'\n' as CHAR || KdpScreenLineLength == KdpScreenLineLengthDefault {
            if KdpScreenLineBufferPos != KdpScreenLineLength {
                HalDisplayString(
                    (*(&raw mut KdpScreenLineBuffer)).as_mut_ptr().add(KdpScreenLineBufferPos as usize),
                );
            }

            KdpScreenLineBuffer[0] = 0;
            KdpScreenLineLength = 0;
            KdpScreenLineBufferPos = 0;
        }

        pch = pch.add(1);
    }

    if KdpScreenLineBufferPos != KdpScreenLineLength {
        HalDisplayString(
            (*(&raw mut KdpScreenLineBuffer)).as_mut_ptr().add(KdpScreenLineBufferPos as usize),
        );
        KdpScreenLineBufferPos = KdpScreenLineLength;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdpScreenInit(
    dispatch_table: PKD_DISPATCH_TABLE,
    BootPhase: ULONG,
) -> NTSTATUS {
    if KdpDebugMode.Screen == 0 {
        return STATUS_PORT_DISCONNECTED;
    }

    if BootPhase == 0 {
        (*dispatch_table).KdpPrintRoutine = Some(KdpScreenPrint);
        (*dispatch_table).KdpInitRoutine = Some(KdpScreenInit);
        InsertTailList(
            core::ptr::addr_of_mut!(KdProviders),
            core::ptr::addr_of_mut!((*dispatch_table).KdProvidersList),
        );
    } else if BootPhase == 1 {
        KdpScreenAcquire();
        HalDisplayString(b"   Screen debugging enabled\r\n\0".as_ptr() as PCHAR);
    }

    STATUS_SUCCESS
}

// ============================================================================
// GENERAL FUNCTIONS
// ============================================================================

unsafe fn KdIoPrintString(String: PCCH, Length: ULONG) {
    let head = core::ptr::addr_of_mut!(KdProviders);
    let mut CurrentEntry: *mut ddk::_LIST_ENTRY = (*head).Flink;

    while !CurrentEntry.is_null() && CurrentEntry != head {
        let CurrentTable: PKD_DISPATCH_TABLE = (CurrentEntry as *mut u8).offset(
            -(core::mem::offset_of!(_KD_DISPATCH_TABLE, KdProvidersList) as isize),
        ) as PKD_DISPATCH_TABLE;
        if let Some(print_routine) = (*CurrentTable).KdpPrintRoutine {
            print_routine(String, Length);
        }
        CurrentEntry = (*CurrentEntry).Flink;
    }
}

#[no_mangle]
pub unsafe extern "C" fn KdIoPuts(String: PCSTR) {
    KdIoPrintString(String, strlen(String) as ULONG);
}

/// KdIoPrintf — kdio.c:574. Stable rustc cannot DEFINE a C-variadic, so the
/// trailing slots are declared as fixed parameters and replayed as the va_list
/// (see `va_slots_rust`). It used to forward the format string to KdIoPuts, so a
/// C consumer calling `KdIoPrintf("%s: exception 0x%08lx\n", …)` -- which is how
/// every site in kdio.c calls it -- got the specifiers printed literally, with
/// its arguments dropped. Internal Rust callers still use `kdio_printf!`.
#[no_mangle]
pub unsafe extern "C" fn KdIoPrintf(
    Format: PCSTR,
    a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize,
    a6: usize, a7: usize, a8: usize, a9: usize, a10: usize, a11: usize,
) {
    let mut Buffer: [i8; 512] = [0; 512];
    let Length = format_slots(
        Buffer.as_mut_ptr(),
        512,
        Format as *const i8,
        &[a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11],
    );
    // The C is `Length = (ULONG)_vsnprintf(...)` with no truncation check, so a
    // format overrunning 512 bytes hands KdIoPrintString a length of 0xFFFFFFFF
    // and walks off the buffer. Clamp instead, the way tui.c:52 does for the
    // identical idiom -- a 4GB overread inside the debug printer is the last
    // thing anyone would successfully debug.
    let Length = if Length < 0 { 512 } else { Length };
    KdIoPrintString(Buffer.as_ptr() as PCSTR, Length as ULONG);
}

// crash-value: when a boot opts into REAL WinDbg KD (/DEBUG /DEBUGPORT=COM2,
// see kd64 kdinit.rs), the embedded kdcom-serial-rust transport takes over
// KdSendPacket/KdReceivePacket wholesale — binary KD packet framing on the
// COM2 UART. Off (the default), behavior below is unchanged: text to COM1.
pub(crate) static mut KD_WIRE_ACTIVE: bool = false;

/// Called from kd64 `KdInitSystem` when the COM2 opt-in is present
/// (/DEBUGPORT=COM2 boot option or `--tripwire "KD=COM2"`). Pins the wire to
/// COM2 @115200 directly — deliberately NOT the transport's LoadOptions parse:
/// a tripwire-armed boot still carries the default `/DEBUGPORT=COM1` options,
/// and the wire must never seize COM1 (the console/smoke channel). Arms
/// delegation only on success.
#[no_mangle]
pub unsafe extern "system" fn KdpWireTransportArm(_LoaderBlock: PVOID) -> NTSTATUS {
    let status = kdcom_serial_rust::kdcom::KdpPortInitialize(2, 115200);
    if NT_SUCCESS(status) {
        KD_WIRE_ACTIVE = true;
        // Probe before any INT 2Dh trap frame exists. WinDbg answers a target
        // RESET with a RESET control packet; a dead wire simply times out.
        kdcom_serial_rust::kddll::KdpSendControlPacket(PACKET_TYPE_KD_RESET as u16, 0);
        let probe = kdcom_serial_rust::kddll::KdReceivePacket(
            PACKET_TYPE_KD_ACKNOWLEDGE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(KdpContext),
        );
        let debugger_waiting = probe != KdPacketTimedOut;
        KdDebuggerNotPresent = if debugger_waiting { 0u8 } else { 1u8 };
        if debugger_waiting {
            (*SharedUserData).KdDebuggerEnabled |= 0x00000002;
            kdio_printf!(b"KD: kdcom-serial wire transport ARMED (COM2, debugger reset)\n\0".as_ptr() as PCSTR);
        } else {
            (*SharedUserData).KdDebuggerEnabled &= !0x00000002;
            kdio_printf!(b"KD: kdcom-serial wire transport ARMED (COM2, awaiting debugger)\n\0".as_ptr() as PCSTR);
        }
    } else {
        kdio_printf!(
            b"KD: kdcom-serial wire transport init FAILED (0x%08lx)\n\0".as_ptr() as PCSTR,
            status,
        );
    }
    status
}

// `KdSendPacket` is the underlying KD-IO sender. When KDBG is enabled the
// callers go through `KdbgSendPacket` (kdbg/kdbg.rs), which delegates here
// for PACKET_TYPE_KD_DEBUG_IO. Same pattern as the C build (see kdbg.c).
#[no_mangle]
pub unsafe extern "system" fn KdSendPacket(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    MessageData: PSTRING,
    Context: PKD_CONTEXT,
) {
    if KD_WIRE_ACTIVE {
        // Tee DbgPrint traffic to COM1 before it becomes a DEBUG_IO packet on
        // the COM2 wire: the console/smoke-marker channel must stay alive on a
        // KD-armed boot (WinDbg additionally shows the same text via the wire).
        if PacketType == PACKET_TYPE_KD_DEBUG_IO
            && !MessageHeader.is_null()
            && !MessageData.is_null()
        {
            let DebugIo = (*MessageHeader).Buffer as PDBGKD_DEBUG_IO;
            if !DebugIo.is_null()
                && (*DebugIo).ApiNumber == DbgKdPrintStringApi
                && !(*MessageData).Buffer.is_null()
            {
                KdIoPrintString((*MessageData).Buffer, (*MessageData).Length as ULONG);
            }
        }
        return kdcom_serial_rust::kddll::KdSendPacket(
            PacketType,
            MessageHeader as *mut _,
            MessageData as *mut _,
            Context as *mut _,
        );
    }
    let _Context = Context;
    if PacketType == PACKET_TYPE_KD_STATE_CHANGE32
        || PacketType == PACKET_TYPE_KD_STATE_CHANGE64
    {
        let WaitStateChange: PDBGKD_ANY_WAIT_STATE_CHANGE =
            (*MessageHeader).Buffer as PDBGKD_ANY_WAIT_STATE_CHANGE;

        if (*WaitStateChange).NewState == DbgKdLoadSymbolsStateChange {
            return;
        }

        if (*WaitStateChange).NewState == DbgKdExceptionStateChange {
            let ExceptionRecord: PEXCEPTION_RECORD64 =
                core::ptr::addr_of_mut!((*WaitStateChange).u.Exception.ExceptionRecord);

            // Claim the debugger to be present (defensive — see C source).
            KdDebuggerNotPresent = 0u8;
            (*SharedUserData).KdDebuggerEnabled |= 0x00000002;

            let first_chance = (*WaitStateChange).u.Exception.FirstChance;
            let chance_str = if first_chance != 0 {
                b"FirstChance\0".as_ptr() as PCSTR
            } else {
                b"LastChance\0".as_ptr() as PCSTR
            };
            kdio_printf!(
                b"%s: Got exception 0x%08lx @ 0x%p, Flags 0x%08x, %s - Info[0]: 0x%p\n\0".as_ptr() as PCSTR,
                b"KdSendPacket\0".as_ptr() as PCSTR,
                (*ExceptionRecord).ExceptionCode,
                (*ExceptionRecord).ExceptionAddress as PVOID,
                (*ExceptionRecord).ExceptionFlags,
                chance_str,
                (*ExceptionRecord).ExceptionInformation[0] as PVOID,
            );

            if ((*ExceptionRecord).ExceptionCode as i32) == STATUS_BREAKPOINT
                && ((*ExceptionRecord).ExceptionAddress as usize)
                    == (RtlpBreakWithStatusInstruction as usize)
            {
                let ContextRecord: PCONTEXT =
                    core::ptr::addr_of_mut!((*KeGetCurrentPrcb()).ProcessorState.ContextFrame);

                #[cfg(target_arch = "x86")]
                let Status: ULONG = (*ContextRecord).Eax;
                #[cfg(target_arch = "x86_64")]
                let Status: ULONG = (*ContextRecord).Rcx as ULONG;
                // arm64 CONTEXT has X0 inside a nested anonymous union:
                //   _CONTEXT { __bindgen_anon_1: union { __bindgen_anon_1: { X0..X30 }, X[31] } }
                #[cfg(target_arch = "aarch64")]
                let Status: ULONG =
                    (*ContextRecord).__bindgen_anon_1.__bindgen_anon_1.X0 as ULONG;

                kdio_printf!(
                    b"STATUS_BREAKPOINT Status 0x%08lx\n\0".as_ptr() as PCSTR,
                    Status,
                );
            }
            return;
        }

        kdio_printf!(
            b"%s: PACKET_TYPE_KD_STATE_CHANGE32/64 NewState %d is UNIMPLEMENTED\n\0".as_ptr() as PCSTR,
            b"KdSendPacket\0".as_ptr() as PCSTR,
            (*WaitStateChange).NewState,
        );
        return;
    } else if PacketType == PACKET_TYPE_KD_STATE_MANIPULATE {
        let ManipulateState: PDBGKD_MANIPULATE_STATE64 =
            (*MessageHeader).Buffer as PDBGKD_MANIPULATE_STATE64;
        kdio_printf!(
            b"%s: PACKET_TYPE_KD_STATE_MANIPULATE for ApiNumber %lu\n\0".as_ptr() as PCSTR,
            b"KdSendPacket\0".as_ptr() as PCSTR,
            (*ManipulateState).ApiNumber,
        );
        return;
    }

    if PacketType != PACKET_TYPE_KD_DEBUG_IO {
        kdio_printf!(
            b"%s: PacketType %d is UNIMPLEMENTED\n\0".as_ptr() as PCSTR,
            b"KdSendPacket\0".as_ptr() as PCSTR,
            PacketType,
        );
        return;
    }

    let DebugIo: PDBGKD_DEBUG_IO = (*MessageHeader).Buffer as PDBGKD_DEBUG_IO;

    if (*MessageHeader).Length as usize != core::mem::size_of::<DBGKD_DEBUG_IO>() {
        return;
    }
    if (*DebugIo).ApiNumber != DbgKdPrintStringApi
        && (*DebugIo).ApiNumber != DbgKdGetStringApi
    {
        return;
    }
    if MessageData.is_null() {
        return;
    }

    if KdpDebugMode.Value == 0 {
        return;
    }

    KdIoPrintString((*MessageData).Buffer, (*MessageData).Length as ULONG);
}

#[no_mangle]
pub unsafe extern "system" fn KdReceivePacket(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    _MessageData: PSTRING,
    _DataLength: PULONG,
    _Context: PKD_CONTEXT,
) -> KDSTATUS {
    if KD_WIRE_ACTIVE {
        return kdcom_serial_rust::kddll::KdReceivePacket(
            PacketType,
            MessageHeader as *mut _,
            _MessageData as *mut _,
            _DataLength,
            _Context as *mut _,
        ) as KDSTATUS;
    }

    if PacketType == PACKET_TYPE_KD_POLL_BREAKIN {
        return KdPacketTimedOut;
    }

    if PacketType == PACKET_TYPE_KD_STATE_MANIPULATE {
        let ManipulateState = (*MessageHeader).Buffer as PDBGKD_MANIPULATE_STATE64;
        core::ptr::write_bytes(
            (*MessageHeader).Buffer as *mut u8,
            0,
            (*MessageHeader).MaximumLength as usize,
        );

        (*ManipulateState).ApiNumber = DbgKdContinueApi;
        (*ManipulateState).u.Continue.ContinueStatus = DBG_EXCEPTION_NOT_HANDLED;
        return KdPacketReceived;
    }

    if PacketType != PACKET_TYPE_KD_DEBUG_IO {
        kdio_printf!(
            b"%s: PacketType %d is UNIMPLEMENTED\n\0".as_ptr() as PCSTR,
            b"KdReceivePacket\0".as_ptr() as PCSTR,
            PacketType,
        );
        return KdPacketTimedOut;
    }

    // KDBG-only branch in C; we don't have a `KDBG` Cargo feature, so the
    // remaining KdbDebugState/KbdDisableMouse/KbdEnableMouse/KdbPromptStr/
    // KdIoReadLine path is left out. KdReceivePacket falls through and just
    // returns KdPacketReceived. (Equivalent to building C with KDBG undefined.)

    KdPacketReceived
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/kd/kdio.c
//  * PURPOSE:         NT Kernel Debugger Input/Output Functions
//  *
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include <reactos/buildno.h>
// #include "kd.h"
// #include "kdterminal.h"
// #ifdef KDBG
// #include "../kdbg/kdb.h"
// #endif
//
// #define NDEBUG
// #include <debug.h>
//
// #undef KdSendPacket
// #undef KdReceivePacket
//
// /* GLOBALS *******************************************************************/
//
// #define KdpBufferSize  (1024 * 512)
// static BOOLEAN KdpLoggingEnabled = FALSE;
// static PCHAR KdpDebugBuffer = NULL;
// static volatile ULONG KdpCurrentPosition = 0;
// static volatile ULONG KdpFreeBytes = 0;
// static KSPIN_LOCK KdpDebugLogSpinLock;
// static KEVENT KdpLoggerThreadEvent;
// static HANDLE KdpLogFileHandle;
// ANSI_STRING KdpLogFileName = RTL_CONSTANT_STRING("\\SystemRoot\\debug.log");
//
// static KSPIN_LOCK KdpSerialSpinLock;
// ULONG  SerialPortNumber = DEFAULT_DEBUG_PORT;
// CPPORT SerialPortInfo   = {0, DEFAULT_DEBUG_BAUD_RATE, 0};
//
// #define KdpScreenLineLengthDefault 80
// static CHAR KdpScreenLineBuffer[KdpScreenLineLengthDefault + 1] = "";
// static ULONG KdpScreenLineBufferPos = 0, KdpScreenLineLength = 0;
//
// KDP_DEBUG_MODE KdpDebugMode;
// LIST_ENTRY KdProviders = {&KdProviders, &KdProviders};
// KD_DISPATCH_TABLE DispatchTable[KdMax] = {0};
//
// PKDP_INIT_ROUTINE InitRoutines[KdMax] =
// {
//     KdpScreenInit,
//     KdpSerialInit,
//     KdpDebugLogInit,
// #ifdef KDBG // See kdb_cli.c
//     KdpKdbgInit
// #endif
// };
//
// /* LOCKING FUNCTIONS *********************************************************/
//
// KIRQL
// NTAPI
// KdbpAcquireLock(
//     _In_ PKSPIN_LOCK SpinLock)
// {
//     KIRQL OldIrql;
//
//     /* Acquire the spinlock without waiting at raised IRQL */
//     while (TRUE)
//     {
//         /* Loop until the spinlock becomes available */
//         while (!KeTestSpinLock(SpinLock));
//
//         /* Spinlock is free, raise IRQL to high level */
//         KeRaiseIrql(HIGH_LEVEL, &OldIrql);
//
//         /* Try to get the spinlock */
//         if (KeTryToAcquireSpinLockAtDpcLevel(SpinLock))
//             break;
//
//         /* Someone else got the spinlock, lower IRQL back */
//         KeLowerIrql(OldIrql);
//     }
//
//     return OldIrql;
// }
//
// VOID
// NTAPI
// KdbpReleaseLock(
//     _In_ PKSPIN_LOCK SpinLock,
//     _In_ KIRQL OldIrql)
// {
//     /* Release the spinlock */
//     KiReleaseSpinLock(SpinLock);
//
//     /* Restore the old IRQL */
//     KeLowerIrql(OldIrql);
// }
//
// /* FILE DEBUG LOG FUNCTIONS **************************************************/
//
// static VOID
// NTAPI
// KdpLoggerThread(PVOID Context)
// {
//     ULONG beg, end, num;
//     IO_STATUS_BLOCK Iosb;
//
//     ASSERT(ExGetPreviousMode() == KernelMode);
//
//     KdpLoggingEnabled = TRUE;
//
//     while (TRUE)
//     {
//         KeWaitForSingleObject(&KdpLoggerThreadEvent, Executive, KernelMode, FALSE, NULL);
//
//         /* Bug */
//         /* Keep KdpCurrentPosition and KdpFreeBytes values in local
//          * variables to avoid their possible change from Producer part,
//          * KdpPrintToLogFile function
//          */
//         end = KdpCurrentPosition;
//         num = KdpFreeBytes;
//
//         /* Now securely calculate values, based on local variables */
//         beg = (end + num) % KdpBufferSize;
//         num = KdpBufferSize - num;
//
//         /* Nothing to do? */
//         if (num == 0)
//             continue;
//
//         if (end > beg)
//         {
//             NtWriteFile(KdpLogFileHandle, NULL, NULL, NULL, &Iosb,
//                         KdpDebugBuffer + beg, num, NULL, NULL);
//         }
//         else
//         {
//             NtWriteFile(KdpLogFileHandle, NULL, NULL, NULL, &Iosb,
//                         KdpDebugBuffer + beg, KdpBufferSize - beg, NULL, NULL);
//
//             NtWriteFile(KdpLogFileHandle, NULL, NULL, NULL, &Iosb,
//                         KdpDebugBuffer, end, NULL, NULL);
//         }
//
//         (VOID)InterlockedExchangeAddUL(&KdpFreeBytes, num);
//     }
// }
//
// static VOID
// NTAPI
// KdpPrintToLogFile(
//     _In_ PCCH String,
//     _In_ ULONG Length)
// {
//     KIRQL OldIrql;
//     ULONG beg, end, num;
//
//     if (KdpDebugBuffer == NULL) return;
//
//     /* Acquire the printing spinlock without waiting at raised IRQL */
//     OldIrql = KdbpAcquireLock(&KdpDebugLogSpinLock);
//
//     beg = KdpCurrentPosition;
//     num = min(Length, KdpFreeBytes);
//     if (num != 0)
//     {
//         end = (beg + num) % KdpBufferSize;
//         KdpCurrentPosition = end;
//         KdpFreeBytes -= num;
//
//         if (end > beg)
//         {
//             RtlCopyMemory(KdpDebugBuffer + beg, String, num);
//         }
//         else
//         {
//             RtlCopyMemory(KdpDebugBuffer + beg, String, KdpBufferSize - beg);
//             RtlCopyMemory(KdpDebugBuffer, String + KdpBufferSize - beg, end);
//         }
//     }
//
//     /* Release the spinlock */
//     KdbpReleaseLock(&KdpDebugLogSpinLock, OldIrql);
//
//     /* Signal the logger thread */
//     if (OldIrql <= DISPATCH_LEVEL && KdpLoggingEnabled)
//         KeSetEvent(&KdpLoggerThreadEvent, IO_NO_INCREMENT, FALSE);
// }
//
// NTSTATUS
// NTAPI
// KdpDebugLogInit(
//     _In_ PKD_DISPATCH_TABLE DispatchTable,
//     _In_ ULONG BootPhase)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     if (!KdpDebugMode.File)
//         return STATUS_PORT_DISCONNECTED;
//
//     if (BootPhase == 0)
//     {
//         /* Write out the functions that we support for now */
//         DispatchTable->KdpPrintRoutine = KdpPrintToLogFile;
//
//         /* Register for BootPhase 1 initialization and as a Provider */
//         DispatchTable->KdpInitRoutine = KdpDebugLogInit;
//         InsertTailList(&KdProviders, &DispatchTable->KdProvidersList);
//     }
//     else if (BootPhase == 1)
//     {
//         /* Allocate a buffer for debug log */
//         KdpDebugBuffer = ExAllocatePoolZero(NonPagedPool,
//                                             KdpBufferSize,
//                                             TAG_KDBG);
//         if (!KdpDebugBuffer)
//         {
//             KdpDebugMode.File = FALSE;
//             RemoveEntryList(&DispatchTable->KdProvidersList);
//             return STATUS_NO_MEMORY;
//         }
//         KdpFreeBytes = KdpBufferSize;
//
//         /* Initialize spinlock */
//         KeInitializeSpinLock(&KdpDebugLogSpinLock);
//
//         /* Register for later BootPhase 2 reinitialization */
//         DispatchTable->KdpInitRoutine = KdpDebugLogInit;
//
//         /* Announce ourselves */
//         HalDisplayString("   File log debugging enabled\r\n");
//     }
//     else if (BootPhase >= 2)
//     {
//         UNICODE_STRING FileName;
//         OBJECT_ATTRIBUTES ObjectAttributes;
//         IO_STATUS_BLOCK Iosb;
//         HANDLE ThreadHandle;
//         KPRIORITY Priority;
//
//         /* If we have already successfully opened the log file, bail out */
//         if (KdpLogFileHandle != NULL)
//             return STATUS_SUCCESS;
//
//         /* Setup the log name */
//         Status = RtlAnsiStringToUnicodeString(&FileName, &KdpLogFileName, TRUE);
//         if (!NT_SUCCESS(Status))
//             goto Failure;
//
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &FileName,
//                                    OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                    NULL,
//                                    NULL);
//
//         /* Create the log file */
//         Status = ZwCreateFile(&KdpLogFileHandle,
//                               FILE_APPEND_DATA | SYNCHRONIZE,
//                               &ObjectAttributes,
//                               &Iosb,
//                               NULL,
//                               FILE_ATTRIBUTE_NORMAL,
//                               FILE_SHARE_READ,
//                               FILE_OPEN_IF,
//                               FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT |
//                                 FILE_SEQUENTIAL_ONLY | FILE_WRITE_THROUGH,
//                               NULL,
//                               0);
//
//         RtlFreeUnicodeString(&FileName);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to open log file: 0x%08lx\n", Status);
//
//             /* Schedule an I/O reinitialization if needed */
//             if (Status == STATUS_OBJECT_NAME_NOT_FOUND ||
//                 Status == STATUS_OBJECT_PATH_NOT_FOUND)
//             {
//                 DispatchTable->KdpInitRoutine = KdpDebugLogInit;
//                 return Status;
//             }
//             goto Failure;
//         }
//
//         /**    HACK for FILE_APPEND_DATA     **
//          ** Remove once CORE-18789 is fixed. **
//          ** Enforce to go to the end of file **/
//         {
//             FILE_STANDARD_INFORMATION FileInfo;
//             FILE_POSITION_INFORMATION FilePosInfo;
//
//             Status = ZwQueryInformationFile(KdpLogFileHandle,
//                                             &Iosb,
//                                             &FileInfo,
//                                             sizeof(FileInfo),
//                                             FileStandardInformation);
//             DPRINT("Status: 0x%08lx - EOF offset: %I64d\n",
//                     Status, FileInfo.EndOfFile.QuadPart);
//
//             Status = ZwQueryInformationFile(KdpLogFileHandle,
//                                             &Iosb,
//                                             &FilePosInfo,
//                                             sizeof(FilePosInfo),
//                                             FilePositionInformation);
//             DPRINT("Status: 0x%08lx - Position: %I64d\n",
//                     Status, FilePosInfo.CurrentByteOffset.QuadPart);
//
//             FilePosInfo.CurrentByteOffset.QuadPart = FileInfo.EndOfFile.QuadPart;
//             Status = ZwSetInformationFile(KdpLogFileHandle,
//                                           &Iosb,
//                                           &FilePosInfo,
//                                           sizeof(FilePosInfo),
//                                           FilePositionInformation);
//             DPRINT("ZwSetInformationFile(FilePositionInfo) returned: 0x%08lx\n", Status);
//         }
//         /** END OF HACK **/
//
//         KeInitializeEvent(&KdpLoggerThreadEvent, SynchronizationEvent, TRUE);
//
//         /* Create the logger thread */
//         Status = PsCreateSystemThread(&ThreadHandle,
//                                       THREAD_ALL_ACCESS,
//                                       NULL,
//                                       NULL,
//                                       NULL,
//                                       KdpLoggerThread,
//                                       NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to create log file thread: 0x%08lx\n", Status);
//             ZwClose(KdpLogFileHandle);
//             goto Failure;
//         }
//
//         Priority = HIGH_PRIORITY;
//         ZwSetInformationThread(ThreadHandle,
//                                ThreadPriority,
//                                &Priority,
//                                sizeof(Priority));
//
//         ZwClose(ThreadHandle);
//         return Status;
//
// Failure:
//         KdpFreeBytes = 0;
//         ExFreePoolWithTag(KdpDebugBuffer, TAG_KDBG);
//         KdpDebugBuffer = NULL;
//         KdpDebugMode.File = FALSE;
//         RemoveEntryList(&DispatchTable->KdProvidersList);
//     }
//
//     return Status;
// }
//
// /* SERIAL FUNCTIONS **********************************************************/
//
// static VOID
// NTAPI
// KdpSerialPrint(
//     _In_ PCCH String,
//     _In_ ULONG Length)
// {
//     PCCH pch = String;
//     KIRQL OldIrql;
//
//     /* Acquire the printing spinlock without waiting at raised IRQL */
//     OldIrql = KdbpAcquireLock(&KdpSerialSpinLock);
//
//     /* Output the string */
//     while (pch < String + Length && *pch)
//     {
//         if (*pch == '\n')
//         {
//             KdPortPutByteEx(&SerialPortInfo, '\r');
//         }
//         KdPortPutByteEx(&SerialPortInfo, *pch);
//         ++pch;
//     }
//
//     /* Release the spinlock */
//     KdbpReleaseLock(&KdpSerialSpinLock, OldIrql);
// }
//
// NTSTATUS
// NTAPI
// KdpSerialInit(
//     _In_ PKD_DISPATCH_TABLE DispatchTable,
//     _In_ ULONG BootPhase)
// {
//     if (!KdpDebugMode.Serial)
//         return STATUS_PORT_DISCONNECTED;
//
//     if (BootPhase == 0)
//     {
//         /* Write out the functions that we support for now */
//         DispatchTable->KdpPrintRoutine = KdpSerialPrint;
//
//         /* Initialize the Port */
//         if (!KdPortInitializeEx(&SerialPortInfo, SerialPortNumber))
//         {
//             KdpDebugMode.Serial = FALSE;
//             return STATUS_DEVICE_DOES_NOT_EXIST;
//         }
//         KdpRustSetComPortInUse(SerialPortInfo.Address);
//
//         /* Initialize spinlock */
//         KeInitializeSpinLock(&KdpSerialSpinLock);
//
//         /* Register for BootPhase 1 initialization and as a Provider */
//         DispatchTable->KdpInitRoutine = KdpSerialInit;
//         InsertTailList(&KdProviders, &DispatchTable->KdProvidersList);
//     }
//     else if (BootPhase == 1)
//     {
//         /* Announce ourselves */
//         HalDisplayString("   Serial debugging enabled\r\n");
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /* SCREEN FUNCTIONS **********************************************************/
//
// VOID
// KdpScreenAcquire(VOID)
// {
//     if (InbvIsBootDriverInstalled() /* &&
//         !InbvCheckDisplayOwnership() */)
//     {
//         /* Acquire ownership and reset the display */
//         InbvAcquireDisplayOwnership();
//         InbvResetDisplay();
//         InbvSolidColorFill(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, BV_COLOR_BLACK);
//         InbvSetTextColor(BV_COLOR_WHITE);
//         InbvInstallDisplayStringFilter(NULL);
//         InbvEnableDisplayString(TRUE);
//         InbvSetScrollRegion(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1);
//     }
// }
//
// // extern VOID NTAPI InbvSetDisplayOwnership(IN BOOLEAN DisplayOwned);
//
// VOID
// KdpScreenRelease(VOID)
// {
//     if (InbvIsBootDriverInstalled()&&
//         InbvCheckDisplayOwnership())
//     {
//         /* Release the display */
//         // InbvSetDisplayOwnership(FALSE);
//         InbvNotifyDisplayOwnershipLost(NULL);
//     }
// }
//
// static VOID
// NTAPI
// KdpScreenPrint(
//     _In_ PCCH String,
//     _In_ ULONG Length)
// {
//     PCCH pch = String;
//
//     while (pch < String + Length && *pch)
//     {
//         if (*pch == '\b')
//         {
//             /* HalDisplayString does not support '\b'. Workaround it and use '\r' */
//             if (KdpScreenLineLength > 0)
//             {
//                 /* Remove last character from buffer */
//                 KdpScreenLineBuffer[--KdpScreenLineLength] = '\0';
//                 KdpScreenLineBufferPos = KdpScreenLineLength;
//
//                 /* Clear row and print line again */
//                 HalDisplayString("\r");
//                 HalDisplayString(KdpScreenLineBuffer);
//             }
//         }
//         else
//         {
//             KdpScreenLineBuffer[KdpScreenLineLength++] = *pch;
//             KdpScreenLineBuffer[KdpScreenLineLength] = '\0';
//         }
//
//         if (*pch == '\n' || KdpScreenLineLength == KdpScreenLineLengthDefault)
//         {
//             /* Print buffered characters */
//             if (KdpScreenLineBufferPos != KdpScreenLineLength)
//                 HalDisplayString(KdpScreenLineBuffer + KdpScreenLineBufferPos);
//
//             /* Clear line buffer */
//             KdpScreenLineBuffer[0] = '\0';
//             KdpScreenLineLength = KdpScreenLineBufferPos = 0;
//         }
//
//         ++pch;
//     }
//
//     /* Print buffered characters */
//     if (KdpScreenLineBufferPos != KdpScreenLineLength)
//     {
//         HalDisplayString(KdpScreenLineBuffer + KdpScreenLineBufferPos);
//         KdpScreenLineBufferPos = KdpScreenLineLength;
//     }
// }
//
// NTSTATUS
// NTAPI
// KdpScreenInit(
//     _In_ PKD_DISPATCH_TABLE DispatchTable,
//     _In_ ULONG BootPhase)
// {
//     if (!KdpDebugMode.Screen)
//         return STATUS_PORT_DISCONNECTED;
//
//     if (BootPhase == 0)
//     {
//         /* Write out the functions that we support for now */
//         DispatchTable->KdpPrintRoutine = KdpScreenPrint;
//
//         /* Register for BootPhase 1 initialization and as a Provider */
//         DispatchTable->KdpInitRoutine = KdpScreenInit;
//         InsertTailList(&KdProviders, &DispatchTable->KdProvidersList);
//     }
//     else if (BootPhase == 1)
//     {
//         /* Take control of the display */
//         KdpScreenAcquire();
//
//         /* Announce ourselves */
//         HalDisplayString("   Screen debugging enabled\r\n");
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// /* GENERAL FUNCTIONS *********************************************************/
//
// static VOID
// KdIoPrintString(
//     _In_ PCCH String,
//     _In_ ULONG Length)
// {
//     PLIST_ENTRY CurrentEntry;
//     PKD_DISPATCH_TABLE CurrentTable;
//
//     /* Call the registered providers */
//     for (CurrentEntry = KdProviders.Flink;
//          CurrentEntry != &KdProviders;
//          CurrentEntry = CurrentEntry->Flink)
//     {
//         CurrentTable = CONTAINING_RECORD(CurrentEntry,
//                                          KD_DISPATCH_TABLE,
//                                          KdProvidersList);
//         CurrentTable->KdpPrintRoutine(String, Length);
//     }
// }
//
// VOID
// KdIoPuts(
//     _In_ PCSTR String)
// {
//     KdIoPrintString(String, (ULONG)strlen(String));
// }
//
// VOID
// __cdecl
// KdIoPrintf(
//     _In_ PCSTR Format,
//     ...)
// {
//     va_list ap;
//     ULONG Length;
//     CHAR Buffer[512];
//
//     /* Format the string */
//     va_start(ap, Format);
//     Length = (ULONG)_vsnprintf(Buffer,
//                                sizeof(Buffer),
//                                Format,
//                                ap);
//     va_end(ap);
//
//     /* Send it to the display providers */
//     KdIoPrintString(Buffer, Length);
// }
//
// #ifdef KDBG
// extern const CSTRING KdbPromptStr;
// #endif
//
// VOID
// NTAPI
// KdSendPacket(
//     _In_ ULONG PacketType,
//     _In_ PSTRING MessageHeader,
//     _In_opt_ PSTRING MessageData,
//     _Inout_ PKD_CONTEXT Context)
// {
//     PDBGKD_DEBUG_IO DebugIo;
//
//     if (PacketType == PACKET_TYPE_KD_STATE_CHANGE32 ||
//         PacketType == PACKET_TYPE_KD_STATE_CHANGE64)
//     {
//         PDBGKD_ANY_WAIT_STATE_CHANGE WaitStateChange = (PDBGKD_ANY_WAIT_STATE_CHANGE)MessageHeader->Buffer;
//
//         if (WaitStateChange->NewState == DbgKdLoadSymbolsStateChange)
//             return; // Ignore: invoked anytime a new module is loaded.
//
//         /* We should not get there, unless an exception has been raised */
//         if (WaitStateChange->NewState == DbgKdExceptionStateChange)
//         {
//             PEXCEPTION_RECORD64 ExceptionRecord = &WaitStateChange->u.Exception.ExceptionRecord;
//
//             /*
//              * Claim the debugger to be present, so that KdpSendWaitContinue()
//              * can call back KdReceivePacket(PACKET_TYPE_KD_STATE_MANIPULATE),
//              * which, in turn, informs KD that the exception cannot be handled.
//              */
//             KD_DEBUGGER_NOT_PRESENT = FALSE;
//             SharedUserData->KdDebuggerEnabled |= 0x00000002;
//
//             KdIoPrintf("%s: Got exception 0x%08lx @ 0x%p, Flags 0x%08x, %s - Info[0]: 0x%p\n",
//                        __FUNCTION__,
//                        ExceptionRecord->ExceptionCode,
//                        (PVOID)(ULONG_PTR)ExceptionRecord->ExceptionAddress,
//                        ExceptionRecord->ExceptionFlags,
//                        WaitStateChange->u.Exception.FirstChance ? "FirstChance" : "LastChance",
//                        ExceptionRecord->ExceptionInformation[0]);
// #if defined(_M_IX86) || defined(_M_AMD64) || defined(_M_ARM) || defined(_M_ARM64)
// extern VOID NTAPI RtlpBreakWithStatusInstruction(VOID);
//             if ((ExceptionRecord->ExceptionCode == STATUS_BREAKPOINT) &&
//                 ((PVOID)(ULONG_PTR)ExceptionRecord->ExceptionAddress == (PVOID)RtlpBreakWithStatusInstruction))
//             {
//                 PCONTEXT ContextRecord = &KeGetCurrentPrcb()->ProcessorState.ContextFrame;
//                 ULONG Status =
// #if defined(_M_IX86)
//                     ContextRecord->Eax;
// #elif defined(_M_AMD64)
//                     (ULONG)ContextRecord->Rcx;
// #elif defined(_M_ARM)
//                     ContextRecord->R0;
// #else // defined(_M_ARM64)
//                     (ULONG)ContextRecord->X0;
// #endif
//                 KdIoPrintf("STATUS_BREAKPOINT Status 0x%08lx\n", Status);
//             }
// // #else
// // #error Unknown architecture
// #endif
//             return;
//         }
//
//         KdIoPrintf("%s: PACKET_TYPE_KD_STATE_CHANGE32/64 NewState %d is UNIMPLEMENTED\n",
//                    __FUNCTION__, WaitStateChange->NewState);
//         return;
//     }
//     else
//     if (PacketType == PACKET_TYPE_KD_STATE_MANIPULATE)
//     {
//         PDBGKD_MANIPULATE_STATE64 ManipulateState = (PDBGKD_MANIPULATE_STATE64)MessageHeader->Buffer;
//         KdIoPrintf("%s: PACKET_TYPE_KD_STATE_MANIPULATE for ApiNumber %lu\n",
//                    __FUNCTION__, ManipulateState->ApiNumber);
//         return;
//     }
//
//     if (PacketType != PACKET_TYPE_KD_DEBUG_IO)
//     {
//         KdIoPrintf("%s: PacketType %d is UNIMPLEMENTED\n", __FUNCTION__, PacketType);
//         return;
//     }
//
//     DebugIo = (PDBGKD_DEBUG_IO)MessageHeader->Buffer;
//
//     /* Validate API call */
//     if (MessageHeader->Length != sizeof(DBGKD_DEBUG_IO))
//         return;
//     if ((DebugIo->ApiNumber != DbgKdPrintStringApi) &&
//         (DebugIo->ApiNumber != DbgKdGetStringApi))
//     {
//         return;
//     }
//     if (!MessageData)
//         return;
//
//     /* NOTE: MessageData->Length should be equal to
//      * DebugIo.u.PrintString.LengthOfString, or to
//      * DebugIo.u.GetString.LengthOfPromptString */
//
//     if (!KdpDebugMode.Value)
//         return;
//
//     /* Print the string proper */
//     KdIoPrintString(MessageData->Buffer, MessageData->Length);
// }
//
// KDSTATUS
// NTAPI
// KdReceivePacket(
//     _In_ ULONG PacketType,
//     _Out_ PSTRING MessageHeader,
//     _Out_ PSTRING MessageData,
//     _Out_ PULONG DataLength,
//     _Inout_ PKD_CONTEXT Context)
// {
// #ifdef KDBG
//     PDBGKD_DEBUG_IO DebugIo;
//     STRING ResponseString;
//     CHAR MessageBuffer[512];
// #endif
//
//     if (PacketType == PACKET_TYPE_KD_POLL_BREAKIN)
//     {
//         /* We don't support breaks-in */
//         return KdPacketTimedOut;
//     }
//
//     if (PacketType == PACKET_TYPE_KD_STATE_MANIPULATE)
//     {
//         PDBGKD_MANIPULATE_STATE64 ManipulateState = (PDBGKD_MANIPULATE_STATE64)MessageHeader->Buffer;
//         RtlZeroMemory(MessageHeader->Buffer, MessageHeader->MaximumLength);
//
//         /* The exception (notified via DbgKdExceptionStateChange in
//          * KdSendPacket()) cannot be handled: return a failure code */
//         ManipulateState->ApiNumber = DbgKdContinueApi;
//         ManipulateState->u.Continue.ContinueStatus = DBG_EXCEPTION_NOT_HANDLED;
//         return KdPacketReceived;
//     }
//
//     if (PacketType != PACKET_TYPE_KD_DEBUG_IO)
//     {
//         KdIoPrintf("%s: PacketType %d is UNIMPLEMENTED\n", __FUNCTION__, PacketType);
//         return KdPacketTimedOut;
//     }
//
// #ifdef KDBG
//     DebugIo = (PDBGKD_DEBUG_IO)MessageHeader->Buffer;
//
//     /* Validate API call */
//     if (MessageHeader->MaximumLength != sizeof(DBGKD_DEBUG_IO))
//         return KdPacketNeedsResend;
//     if (DebugIo->ApiNumber != DbgKdGetStringApi)
//         return KdPacketNeedsResend;
//
//     /* NOTE: We cannot use directly MessageData->Buffer here as it points
//      * to the temporary KdpMessageBuffer scratch buffer that is being
//      * shared with all the possible I/O KD operations that may happen. */
//     ResponseString.Buffer = MessageBuffer;
//     ResponseString.Length = 0;
//     ResponseString.MaximumLength = min(sizeof(MessageBuffer),
//                                        MessageData->MaximumLength);
//     ResponseString.MaximumLength = min(ResponseString.MaximumLength,
//                                        DebugIo->u.GetString.LengthOfStringRead);
//
//     /* The prompt string has been printed by KdSendPacket; go to
//      * new line and print the kdb prompt -- for SYSREG2 support. */
//     KdIoPrintString("\n", 1);
//     KdIoPuts(KdbPromptStr.Buffer); // Alternatively, use "Input> "
//
//     if (!(KdbDebugState & KD_DEBUG_KDSERIAL))
//         KbdDisableMouse();
//
//     /*
//      * Read a NULL-terminated line of user input and retrieve its length.
//      * Official documentation states that DbgPrompt() includes a terminating
//      * newline character but does not NULL-terminate. However, experiments
//      * show that this behaviour is left at the discretion of WinDbg itself.
//      * WinDbg NULL-terminates the string unless its buffer is too short,
//      * in which case the string is simply truncated without NULL-termination.
//      */
//     ResponseString.Length =
//         (USHORT)KdIoReadLine(ResponseString.Buffer,
//                              ResponseString.MaximumLength);
//
//     if (!(KdbDebugState & KD_DEBUG_KDSERIAL))
//         KbdEnableMouse();
//
//     /* Adjust and return the string length */
//     *DataLength = min(ResponseString.Length + sizeof(ANSI_NULL),
//                       DebugIo->u.GetString.LengthOfStringRead);
//     MessageData->Length = DebugIo->u.GetString.LengthOfStringRead = *DataLength;
//
//     /* Only now we can copy back the data into MessageData->Buffer */
//     RtlCopyMemory(MessageData->Buffer, ResponseString.Buffer, *DataLength);
// #endif
//
//     return KdPacketReceived;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
