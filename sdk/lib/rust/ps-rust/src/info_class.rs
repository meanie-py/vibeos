//! `PsProcessInfoClass` and `PsThreadInfoClass` info-class tables.
//!
//! Direct translation of `ntoskrnl/include/internal/ps_i.h`. These are the
//! per-class metadata arrays consumed by `DefaultQueryInfoBufferCheck` and
//! `DefaultSetInfoBufferCheck` in the `Nt{Query,Set}InformationProcess` /
//! `Nt{Query,Set}InformationThread` dispatchers (`ps/query.rs`).
//!
//! Each entry mirrors the C `IQS` / `IQS_SAME` / `IQS_NO_TYPE_LENGTH` /
//! `IQS_NONE` macro expansions:
//!
//!   IQS_SAME(T, A, F)        = { sizeof T, sizeof A, sizeof T, sizeof A, F }
//!   IQS(TQ, AQ, TS, AS, F)   = { sizeof TQ, sizeof AQ, sizeof TS, sizeof AS, F }
//!   IQS_NO_TYPE_LENGTH(A, F) = { 0, sizeof A, 0, sizeof A, F }
//!   IQS_NONE                 = { 0, 1, 0, 1, ICIF_NONE }
//!
//! The `Alignment{QUERY,SET}` UCHAR fields are populated with `size_of` of
//! the alignment-marker type (matching the C macro's `sizeof(Alignment)`),
//! not `align_of`. We mirror that with `core::mem::size_of` so the
//! translation is byte-identical to the C original on each architecture.

use ntkrnl_sys::base::{UCHAR, USHORT, ULONG_PTR, BOOLEAN, ULONG, LONG};
// KAFFINITY = ULONG_PTR per Windows DDK convention.
type KAFFINITY = ULONG_PTR;
use ntkrnl_sys::ddk::{
    HANDLE, KPRIORITY, _LARGE_INTEGER as LARGE_INTEGER,
    _INFORMATION_CLASS_INFO as INFORMATION_CLASS_INFO,
    _PROCESS_BASIC_INFORMATION as PROCESS_BASIC_INFORMATION,
    _QUOTA_LIMITS as QUOTA_LIMITS,
    _IO_COUNTERS as IO_COUNTERS,
    _VM_COUNTERS as VM_COUNTERS,
    _KERNEL_USER_TIMES as KERNEL_USER_TIMES,
    _PROCESS_ACCESS_TOKEN as PROCESS_ACCESS_TOKEN,
    _PROCESS_LDT_INFORMATION as PROCESS_LDT_INFORMATION,
    _PROCESS_LDT_SIZE as PROCESS_LDT_SIZE,
    _POOLED_USAGE_AND_LIMITS as POOLED_USAGE_AND_LIMITS,
    _PROCESS_WS_WATCH_INFORMATION as PROCESS_WS_WATCH_INFORMATION,
    _PROCESS_PRIORITY_CLASS as PROCESS_PRIORITY_CLASS,
    _PROCESS_DEVICEMAP_INFORMATION__bindgen_ty_1__bindgen_ty_1
        as PROCESS_DEVICEMAP_INFORMATION_SET,
    _PROCESS_DEVICEMAP_INFORMATION__bindgen_ty_1__bindgen_ty_2
        as PROCESS_DEVICEMAP_INFORMATION_QUERY,
    _PROCESS_SESSION_INFORMATION as PROCESS_SESSION_INFORMATION,
    _UNICODE_STRING as UNICODE_STRING,
    _PROCESS_HANDLE_TRACING_QUERY as PROCESS_HANDLE_TRACING_QUERY,
    _SECTION_IMAGE_INFORMATION as SECTION_IMAGE_INFORMATION,
    _THREAD_BASIC_INFORMATION as THREAD_BASIC_INFORMATION,
};

use core::mem::size_of;

// Mirror of `internal/icif.h` (each .rs that uses this currently redeclares
// these locally; matching the existing pattern keeps the swap mechanical).
const ICIF_NONE:                u16 = 0x0;
const ICIF_QUERY:               u16 = 0x1;
const ICIF_SET:                 u16 = 0x2;
const ICIF_QUERY_SIZE_VARIABLE: u16 = 0x4;
const ICIF_SET_SIZE_VARIABLE:   u16 = 0x8;
const ICIF_SIZE_VARIABLE:       u16 = ICIF_QUERY_SIZE_VARIABLE | ICIF_SET_SIZE_VARIABLE;

const fn iqs<TQ: Sized, AQ: Sized, TS: Sized, AS: Sized>(flags: u16) -> INFORMATION_CLASS_INFO {
    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: size_of::<TQ>() as USHORT,
        AlignmentQUERY:    size_of::<AQ>() as UCHAR,
        RequiredSizeSET:   size_of::<TS>() as USHORT,
        AlignmentSET:      size_of::<AS>() as UCHAR,
        Flags:             flags as USHORT,
    }
}

const fn iqs_same<T: Sized, A: Sized>(flags: u16) -> INFORMATION_CLASS_INFO {
    iqs::<T, A, T, A>(flags)
}

const fn iqs_no_type_length<A: Sized>(flags: u16) -> INFORMATION_CLASS_INFO {
    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: 0,
        AlignmentQUERY:    size_of::<A>() as UCHAR,
        RequiredSizeSET:   0,
        AlignmentSET:      size_of::<A>() as UCHAR,
        Flags:             flags as USHORT,
    }
}

const fn iqs_none() -> INFORMATION_CLASS_INFO {
    INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: 0,
        AlignmentQUERY:    1,    // sizeof(CHAR)
        RequiredSizeSET:   0,
        AlignmentSET:      1,
        Flags:             ICIF_NONE as USHORT,
    }
}

// CHAR/u8 marker for IQS expansions that use sizeof(CHAR).
type C_CHAR    = u8;
type C_BOOLEAN = BOOLEAN;

// ---------------------------------------------------------------------------
// PsProcessInfoClass — 47 entries.
// ---------------------------------------------------------------------------

#[no_mangle]
pub static PsProcessInfoClass: [INFORMATION_CLASS_INFO; 47] = [
    /*  0 ProcessBasicInformation              */ iqs_same::<PROCESS_BASIC_INFORMATION, ULONG>(ICIF_QUERY),
    /*  1 ProcessQuotaLimits                   */ iqs_same::<QUOTA_LIMITS, ULONG>(ICIF_QUERY | ICIF_SET | ICIF_SIZE_VARIABLE),
    /*  2 ProcessIoCounters                    */ iqs_same::<IO_COUNTERS, ULONG>(ICIF_QUERY),
    /*  3 ProcessVmCounters                    */ iqs_same::<VM_COUNTERS, ULONG>(ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE),
    /*  4 ProcessTimes                         */ iqs_same::<KERNEL_USER_TIMES, ULONG>(ICIF_QUERY),
    /*  5 ProcessBasePriority                  */ iqs_same::<KPRIORITY, ULONG>(ICIF_SET),
    /*  6 ProcessRaisePriority                 */ iqs_same::<ULONG, ULONG>(ICIF_SET),
    /*  7 ProcessDebugPort                     */ iqs_same::<HANDLE, ULONG>(ICIF_QUERY),
    /*  8 ProcessExceptionPort                 */ iqs_same::<HANDLE, HANDLE>(ICIF_SET),
    /*  9 ProcessAccessToken                   */ iqs_same::<PROCESS_ACCESS_TOKEN, ULONG>(ICIF_SET),
    /* 10 ProcessLdtInformation                */ iqs_same::<PROCESS_LDT_INFORMATION, ULONG>(ICIF_QUERY | ICIF_SET),
    /* 11 ProcessLdtSize                       */ iqs_same::<PROCESS_LDT_SIZE, ULONG>(ICIF_SET),
    /* 12 ProcessDefaultHardErrorMode          */ iqs_same::<ULONG, ULONG>(ICIF_QUERY | ICIF_SET),
    /* 13 ProcessIoPortHandlers                */ iqs_same::<UCHAR, ULONG>(ICIF_SET),
    /* 14 ProcessPooledUsageAndLimits          */ iqs_same::<POOLED_USAGE_AND_LIMITS, ULONG>(ICIF_QUERY),
    /* 15 ProcessWorkingSetWatch               */ iqs_same::<PROCESS_WS_WATCH_INFORMATION, ULONG>(ICIF_QUERY | ICIF_SET | ICIF_SET_SIZE_VARIABLE),

    /* 16 ProcessUserModeIOPL — i386 only      */
    #[cfg(target_arch = "x86")]    iqs_no_type_length::<ULONG>(ICIF_SET),
    #[cfg(not(target_arch = "x86"))] iqs_none(),

    /* 17 ProcessEnableAlignmentFaultFixup     */ iqs::<C_BOOLEAN, ULONG, C_BOOLEAN, C_CHAR>(ICIF_SET),
    /* 18 ProcessPriorityClass                 */ iqs::<PROCESS_PRIORITY_CLASS, ULONG, PROCESS_PRIORITY_CLASS, C_CHAR>(ICIF_QUERY | ICIF_SET),
    /* 19 ProcessWx86Information               */ iqs_same::<ULONG, ULONG>(ICIF_QUERY | ICIF_SET),
    /* 20 ProcessHandleCount                   */ iqs_same::<ULONG, ULONG>(ICIF_QUERY),
    /* 21 ProcessAffinityMask                  */ iqs_same::<KAFFINITY, KAFFINITY>(ICIF_SET),
    /* 22 ProcessPriorityBoost                 */ iqs_same::<ULONG, ULONG>(ICIF_QUERY | ICIF_SET),
    /* 23 ProcessDeviceMap                     */ iqs::<PROCESS_DEVICEMAP_INFORMATION_QUERY, ULONG,
                                                          PROCESS_DEVICEMAP_INFORMATION_SET,   ULONG>(ICIF_QUERY | ICIF_SET),
    /* 24 ProcessSessionInformation            */ iqs_same::<PROCESS_SESSION_INFORMATION, ULONG>(ICIF_QUERY | ICIF_SET),
    /* 25 ProcessForegroundInformation         */ iqs::<C_CHAR, ULONG, C_BOOLEAN, C_CHAR>(ICIF_SET),
    /* 26 ProcessWow64Information              */ iqs_same::<ULONG_PTR, ULONG>(ICIF_QUERY),
    /* 27 ProcessImageFileName                 */ iqs_same::<UNICODE_STRING, ULONG>(ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE),
    /* 28 ProcessLUIDDeviceMapsEnabled         */ iqs_same::<ULONG, ULONG>(ICIF_QUERY),
    /* 29 ProcessBreakOnTermination            */ iqs_same::<ULONG, ULONG>(ICIF_QUERY | ICIF_SET),
    /* 30 ProcessDebugObjectHandle             */ iqs_same::<HANDLE, ULONG>(ICIF_QUERY),
    /* 31 ProcessDebugFlags                    */ iqs_same::<ULONG, ULONG>(ICIF_QUERY | ICIF_SET),
    /* 32 ProcessHandleTracing                 */ iqs::<PROCESS_HANDLE_TRACING_QUERY, ULONG, ULONG, ULONG>(ICIF_QUERY | ICIF_SET),
    /* 33 ProcessIoPriority                    */ iqs_none(),
    /* 34 ProcessExecuteFlags                  */ iqs_same::<ULONG, ULONG>(ICIF_QUERY | ICIF_SET),
    /* 35 ProcessTlsInformation                */ iqs_none(),
    /* 36 ProcessCookie                        */ iqs_same::<ULONG, ULONG>(ICIF_QUERY),
    /* 37 ProcessImageInformation              */ iqs_same::<SECTION_IMAGE_INFORMATION, ULONG>(ICIF_QUERY),
    /* 38 ProcessCycleTime                     */ iqs_none(),
    /* 39 ProcessPagePriority                  */ iqs_none(),
    /* 40 ProcessInstrumentationCallback       */ iqs_none(),
    /* 41 ProcessThreadStackAllocation         */ iqs_none(),
    /* 42 ProcessWorkingSetWatchEx             */ iqs_none(),
    /* 43 ProcessImageFileNameWin32            */ iqs_same::<UNICODE_STRING, ULONG_PTR>(ICIF_QUERY | ICIF_QUERY_SIZE_VARIABLE),
    /* 44 ProcessImageFileMapping              */ iqs_none(),
    /* 45 ProcessAffinityUpdateMode            */ iqs_none(),
    /* 46 ProcessMemoryAllocationMode          */ iqs_none(),
];

// ---------------------------------------------------------------------------
// PsThreadInfoClass — 28 entries.
// ---------------------------------------------------------------------------

// On i386 the C source uses DESCRIPTOR_TABLE_ENTRY (defined only there).
#[cfg(target_arch = "x86")]
use ntkrnl_sys::ddk::_DESCRIPTOR_TABLE_ENTRY as DESCRIPTOR_TABLE_ENTRY;

#[no_mangle]
pub static PsThreadInfoClass: [INFORMATION_CLASS_INFO; 28] = [
    /*  0 ThreadBasicInformation               */ iqs_same::<THREAD_BASIC_INFORMATION, ULONG>(ICIF_QUERY),
    /*  1 ThreadTimes                          */ iqs_same::<KERNEL_USER_TIMES, ULONG>(ICIF_QUERY),
    /*  2 ThreadPriority                       */ iqs_same::<KPRIORITY, ULONG>(ICIF_SET),
    /*  3 ThreadBasePriority                   */ iqs_same::<LONG, ULONG>(ICIF_SET),
    /*  4 ThreadAffinityMask                   */ iqs_same::<KAFFINITY, ULONG>(ICIF_SET),
    /*  5 ThreadImpersonationToken             */ iqs_same::<HANDLE, ULONG>(ICIF_SET),

    /*  6 ThreadDescriptorTableEntry — i386    */
    #[cfg(target_arch = "x86")]    iqs_same::<DESCRIPTOR_TABLE_ENTRY, ULONG>(ICIF_QUERY),
    #[cfg(not(target_arch = "x86"))] iqs_none(),

    /*  7 ThreadEnableAlignmentFaultFixup      */ iqs::<C_CHAR, C_CHAR, C_BOOLEAN, UCHAR>(ICIF_SET),
    /*  8 ThreadEventPair_Reusable             */ iqs_none(),
    /*  9 ThreadQuerySetWin32StartAddress      */ iqs::<*const u8, ULONG, ULONG_PTR, ULONG>(ICIF_QUERY | ICIF_SET),
    /* 10 ThreadZeroTlsCell                    */ iqs_same::<ULONG, ULONG>(ICIF_SET),
    /* 11 ThreadPerformanceCount               */ iqs_same::<LARGE_INTEGER, ULONG>(ICIF_QUERY),
    /* 12 ThreadAmILastThread                  */ iqs_same::<ULONG, ULONG>(ICIF_QUERY),
    // 13 and 14 SET a ULONG (the C table said ULONG_PTR): Windows 11 accepts exactly 4
    // bytes for both, on x64 and x86, and kernel32 passes a ULONG.
    /* 13 ThreadIdealProcessor                 */ iqs_same::<ULONG, ULONG>(ICIF_SET),
    /* 14 ThreadPriorityBoost                  */ iqs_same::<ULONG, ULONG>(ICIF_QUERY | ICIF_SET),
    /* 15 ThreadSetTlsArrayAddress             */ iqs_same::<*const u8, ULONG>(ICIF_SET | ICIF_SIZE_VARIABLE),
    /* 16 ThreadIsIoPending                    */ iqs_same::<ULONG, ULONG>(ICIF_QUERY),
    /* 17 ThreadHideFromDebugger               */ iqs_same::<C_CHAR, ULONG>(ICIF_SET | ICIF_SET_SIZE_VARIABLE),
    /* 18 ThreadBreakOnTermination             */ iqs_same::<ULONG, ULONG>(ICIF_QUERY | ICIF_SET),
    /* 19 ThreadSwitchLegacyState              */ iqs_same::<ULONG, ULONG>(ICIF_SET),
    /* 20 ThreadIsTerminated                   */ iqs_same::<ULONG, ULONG>(ICIF_QUERY),
    /* 21 ThreadLastSystemCall                 */ iqs_none(),
    /* 22 ThreadIoPriority                     */ iqs_none(),
    /* 23 ThreadCycleTime                      */ iqs_none(),
    /* 24 ThreadPagePriority                   */ iqs_none(),
    /* 25 ThreadActualBasePriority             */ iqs_none(),
    /* 26 ThreadTebInformation                 */ iqs_none(),
    /* 27 ThreadCSwitchMon                     */ iqs_none(),
];

// ---------------------------------------------------------------------------
// Status (2026-05-08)
// ---------------------------------------------------------------------------
//
// Rust supplies the canonical `PsProcessInfoClass` / `PsThreadInfoClass`
// linker symbols. The C-side `kernel_c_helpers.c` ps_i.h re-include trick
// is retired.
//
// The earlier "i386 BUGCHECK 0x1E STATUS_DATATYPE_MISALIGNMENT at WINLOGON"
// regression turned out to be a pre-existing latent bug in `ps/query.rs`,
// not in the translation: the `PS_{PROCESS,THREAD}_INFO_CLASS_COUNT`
// constants were transcribed from the Vista+ `Max*InfoClass` enums (51 / 56
// minus 0 / 1 → 48 / 57) instead of the actual NT 5.x array entry counts
// (47 / 28). Passing the larger counts to `DefaultQueryInfoBufferCheck`
// skipped the bounds check for past-end indices, reading garbage
// `AlignmentQUERY` bytes from whatever followed the array in `.rdata`.
// The C-side `kernel_c_helpers.c` re-include happened to leave "safe"
// power-of-2 trailing bytes; the Rust translation didn't, surfacing the
// bug only after the symbol-supplier swap. Fix landed in `ps/query.rs`
// alongside this rename — see the comment block above the constants.
