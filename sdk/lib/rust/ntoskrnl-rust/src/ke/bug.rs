use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::NonPagedPool;
// mm symbols imported from ddk below (MmIsSpecialPoolAddress etc.)
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _LDR_DATA_TABLE_ENTRY, PLDR_DATA_TABLE_ENTRY,
    _CONTEXT, CONTEXT,
    _UNICODE_STRING, PUNICODE_STRING,
    _KBUGCHECK_CALLBACK_RECORD, PKBUGCHECK_CALLBACK_RECORD,
    _KBUGCHECK_REASON_CALLBACK_RECORD, PKBUGCHECK_REASON_CALLBACK_RECORD,
    _KNMI_HANDLER_CALLBACK, PKNMI_HANDLER_CALLBACK,
    _KTRAP_FRAME, PKTRAP_FRAME,
    _IMAGE_NT_HEADERS, PIMAGE_NT_HEADERS,
    _IMAGE_RESOURCE_DATA_ENTRY, PIMAGE_RESOURCE_DATA_ENTRY,
    _LDR_RESOURCE_INFO,
    _MESSAGE_RESOURCE_DATA, PMESSAGE_RESOURCE_DATA,
    _MESSAGE_RESOURCE_ENTRY, PMESSAGE_RESOURCE_ENTRY,
    _MESSAGE_RESOURCE_BLOCK,
    PKBUGCHECK_CALLBACK_ROUTINE, PKBUGCHECK_REASON_CALLBACK_ROUTINE,
    PNMI_CALLBACK,
    KBUGCHECK_CALLBACK_REASON,
    HalReturnToFirmware, InbvAcquireDisplayOwnership, InbvDisplayString,
    InbvEnableDisplayString, InbvInstallDisplayStringFilter, InbvIsBootDriverInstalled,
    InbvResetDisplay, InbvSetScrollRegion, InbvSetTextColor, InbvSolidColorFill,
    KeBugCheckUnicodeToAnsi as KeBugCheckUnicodeToAnsiExtern,
    KeBugCheckWithTf as KeBugCheckWithTfExtern,
    KiAcquireNmiListLock, KiHandleNmi as KiHandleNmiExtern,
    KiInitializeBugCheck as KiInitializeBugCheckExtern, KiPcToFileHeader as KiPcToFileHeaderExtern,
    KiReleaseNmiListLock, KiRosPcToUserFileHeader as KiRosPcToUserFileHeaderExtern,
    KiSaveProcessorControlState, LdrAccessResource, LdrFindResource_U,
    MmIsAddressValid, MmIsSpecialPoolAddress, MmIsSpecialPoolAddressFree, MmMakeKernelResourceSectionWritable,
    PKE_BUGCHECK_UNICODE_TO_ANSI, PANSI_STRING,
};

#[no_mangle]
pub static mut KeBugcheckCallbackListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut KeBugcheckReasonCallbackListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut BugCheckCallbackLock: KSPIN_LOCK = 0;
#[no_mangle]
pub static mut KeBugCheckActive: ULONG = 0;
#[no_mangle]
pub static mut KeBugCheckOwner: ULONG = 0;
static mut KeBugCheckOwnerRecursionCount: LONG = 0;
static mut KiBugCodeMessages: PMESSAGE_RESOURCE_DATA = core::ptr::null_mut();
static mut KeBugCheckCount: ULONG = 1;
#[no_mangle]
pub static mut KiHardwareTrigger: ULONG = 0;
static mut KiBugCheckDriver: PUNICODE_STRING = core::ptr::null_mut();
#[no_mangle]
pub static mut KiBugCheckData: [ULONG_PTR; 5] = [0; 5];

#[no_mangle]
pub static mut KiNmiCallbackListHead: PKNMI_HANDLER_CALLBACK = core::ptr::null_mut();
#[no_mangle]
pub static mut KiNmiCallbackListLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut KeRosProcessorName: _UNICODE_STRING = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut KeRosBiosDate: _UNICODE_STRING = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut KeRosBiosVersion: _UNICODE_STRING = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut KeRosVideoBiosDate: _UNICODE_STRING = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut KeRosVideoBiosVersion: _UNICODE_STRING = unsafe { core::mem::zeroed() };

use super::krnlinit::KeLoaderBlock;
extern "system" {
    // NTAPI/stdcall per ntoskrnl.spec — matches the extern "system" shim def
    // in shim_exports.rs and the canonical mem-rust export.
    fn RtlFillMemoryUlong(Destination: PVOID, Length: SIZE_T, Pattern: ULONG);
}
extern "system" {
    static mut PsLoadedModuleList: LIST_ENTRY;
    static mut MmHighestUserAddress: PVOID;
    static KdDebuggerEnabled: BOOLEAN;
    static KdPitchDebugger: BOOLEAN;
    static mut KdDebuggerNotPresent: BOOLEAN;
    static mut KdDebuggerDataBlock: PVOID;
    static mut IopAutoReboot: BOOLEAN;
    #[cfg(target_arch = "aarch64")] static Arm64UartKernelBase: PVOID;
    fn RtlWalkFrameChain(Callers: *mut PVOID, Count: ULONG, Flags: ULONG) -> ULONG;
    fn RtlCaptureStackBackTrace(FramesToSkip: ULONG, FramesToCapture: ULONG, BackTrace: *mut PVOID, BackTraceHash: PULONG) -> USHORT;
    fn RtlCaptureContext(ContextRecord: *mut CONTEXT);
    fn KdInitSystem(BootPhase: ULONG, LoaderBlock: PVOID);
    fn DbgUnLoadImageSymbols(FileName: PANSI_STRING, BaseOfDll: PVOID, ProcessId: ULONG_PTR);
}
extern "system" {

}
extern "system" {
    fn RtlImageNtHeader(Base: PVOID) -> PIMAGE_NT_HEADERS;
    fn DbgBreakPointWithStatus(Status: ULONG);
    fn HalHaltSystem();
    fn HeadlessDispatch(Command: ULONG, InputBuffer: PVOID, InputBufferSize: SIZE_T, OutputBuffer: PVOID, OutputBufferSize: PSIZE_T);
    fn KeGetTrapFramePc(TrapFrame: PKTRAP_FRAME) -> ULONG_PTR;
    #[cfg(feature = "smp")] fn KxFreezeExecution();
}
extern "C" {
    fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32;
    fn strlen(s: *const i8) -> SIZE_T;
}

const TAG_KNMI: ULONG = 0x494D4E4B;

const RT_MESSAGETABLE: ULONG = 11;
const RESOURCE_DATA_LEVEL: ULONG = 3; // ndk/ldrtypes.h:33 (type/name/language -> data entry)
const LANG_NEUTRAL: USHORT = 0;
const SUBLANG_NEUTRAL: USHORT = 0;

fn MAKELANGID(p: USHORT, s: USHORT) -> USHORT {
    (s << 10) | p
}

const MESSAGE_RESOURCE_UNICODE: USHORT = 0x0001;
const ANSI_NULL: u8 = 0;

const BUGCHECK_MESSAGE_INTRO: ULONG = 0x8000007F; // FIXED 2026-07-05: translation invented wrong value (verified vs bugcodes.h)
const BUGCODE_ID_DRIVER: ULONG = 0x80000080; // FIXED 2026-07-05: translation invented wrong value (verified vs bugcodes.h)
const PSS_MESSAGE_INTRO: ULONG = 0x80000081; // FIXED 2026-07-05: translation invented wrong value (verified vs bugcodes.h)
const BUGCHECK_TECH_INFO: ULONG = 0x80000083; // FIXED 2026-07-05: translation invented wrong value (verified vs bugcodes.h)
const BUGCODE_PSS_MESSAGE: ULONG = 0x80000082; // FIXED 2026-07-05: translation invented wrong value (verified vs bugcodes.h)
const BUGCODE_PSS_MESSAGE_SIGNATURE: ULONG = 0x000000C3; // FIXED 2026-07-05: translation invented wrong value (verified vs bugcodes.h)

const BufferEmpty: UCHAR = 0;
const BufferInserted: UCHAR = 1;
const BufferStarted: UCHAR = 2;
const BufferFinished: UCHAR = 3;
const BufferIncomplete: UCHAR = 4;

const DBG_STATUS_BUGCHECK_FIRST: ULONG = 0x03;
const DBG_STATUS_BUGCHECK_SECOND: ULONG = 0x04;
const DBG_STATUS_FATAL: ULONG = 0x05;

const SCREEN_WIDTH: ULONG = 640;
const SCREEN_HEIGHT: ULONG = 480;
const BV_COLOR_RED: ULONG = 0x1;
const BV_COLOR_BLUE: ULONG = 0x4;
const BV_COLOR_WHITE: ULONG = 15;

/// panic_common::RUST_PANIC — a Rust panic routed into KeBugCheckEx.
/// P1 = &NUL-terminated report (message + file:line + raw-PC backtrace,
/// formatted by panic-common into its static buffer), P2 = buffer cap,
/// P4 = source line. Crash-value lane: Rust panics render a RED screen
/// (logic/translation bug) vs the classic blue stop — the color is the
/// category at a glance.
const RUST_PANIC_BUGCODE: ULONG = 0xE052_5354;

const HeadlessCmdStartBugCheck: ULONG = 0;
const HeadlessCmdEnableTerminal: ULONG = 1;
const HeadlessCmdSendBlueScreenData: ULONG = 2;

const HalRebootRoutine: ULONG = 1;

const POWER_FAILURE_SIMULATE: ULONG = 0x000000E5; // FIXED 2026-07-05: translation invented wrong value (verified vs bugcodes.h)
const UNEXPECTED_KERNEL_MODE_TRAP: ULONG = 0x0000007F;
const DRIVER_CORRUPTED_EXPOOL: ULONG = 0x000000C5;
const ACPI_BIOS_ERROR: ULONG = 0x000000A5;
const ACPI_BIOS_FATAL_ERROR: ULONG = 0x000000E0;
const THREAD_STUCK_IN_DEVICE_DRIVER: ULONG = 0x000000EA;
const DATA_BUS_ERROR: ULONG = 0x0000002E;
const FAT_FILE_SYSTEM: ULONG = 0x00000023;
const NO_MORE_SYSTEM_PTES: ULONG = 0x0000003F;
const INACCESSIBLE_BOOT_DEVICE: ULONG = 0x0000007B;
const KERNEL_MODE_EXCEPTION_NOT_HANDLED: ULONG = 0x0000008E; // FIXED 2026-07-05: translation had 0x1E (dup of KMODE_) -> unreachable OR-pattern; real value per bugcodes.h (master converged on same value)
const SYSTEM_THREAD_EXCEPTION_NOT_HANDLED: ULONG = 0x0000007E;
const KMODE_EXCEPTION_NOT_HANDLED: ULONG = 0x0000001E;
const NTFS_FILE_SYSTEM: ULONG = 0x00000024;
const DRIVER_CORRUPTED_MMPOOL: ULONG = 0x000000D0;
const STATUS_SYSTEM_IMAGE_BAD_SIGNATURE: ULONG = 0xC00002D1;
const ATTEMPTED_WRITE_TO_READONLY_MEMORY: ULONG = 0x000000BE;
const ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY: ULONG = 0x000000FC;
const IRQL_NOT_LESS_OR_EQUAL: ULONG = 0x0000000A;
const DRIVER_PORTION_MUST_BE_NONPAGED: ULONG = 0x000000D3; // FIXED 2026-07-05: translation invented wrong value (verified vs bugcodes.h)
const SYSTEM_SCAN_AT_RAISED_IRQL_CAUGHT_IMPROPER_DRIVER_UNLOAD: ULONG = 0x000000D4; // FIXED 2026-07-05: translation invented wrong value (verified vs bugcodes.h)
const DRIVER_IRQL_NOT_LESS_OR_EQUAL: ULONG = 0x000000D1;
const FATAL_UNHANDLED_HARD_ERROR: ULONG = 0x0000004C;
const PAGE_FAULT_IN_NONPAGED_AREA: ULONG = 0x00000050;
const PAGE_FAULT_IN_FREED_SPECIAL_POOL: ULONG = 0x000000CC; // FIXED 2026-07-05: translation invented wrong value (verified vs bugcodes.h)
const DRIVER_PAGE_FAULT_IN_FREED_SPECIAL_POOL: ULONG = 0x000000D5; // FIXED 2026-07-05: translation invented wrong value (verified vs bugcodes.h)
const PAGE_FAULT_BEYOND_END_OF_ALLOCATION: ULONG = 0x000000CD; // FIXED 2026-07-05: translation invented wrong value (verified vs bugcodes.h)
const DRIVER_PAGE_FAULT_BEYOND_END_OF_ALLOCATION: ULONG = 0x000000D6; // FIXED 2026-07-05: translation invented wrong value (verified vs bugcodes.h)
const DRIVER_UNLOADED_WITHOUT_CANCELLING_PENDING_OPERATIONS: ULONG = 0x000000CE;
const DRIVER_LEFT_LOCKED_PAGES_IN_PROCESS: ULONG = 0x000000CB;
const DRIVER_USED_EXCESSIVE_PTES: ULONG = 0x000000D8;
const MANUALLY_INITIATED_CRASH: ULONG = 0x000000E2;

#[no_mangle]
pub unsafe extern "system" fn KiPcToFileHeader(
    Pc: PVOID,
    LdrEntry: *mut PLDR_DATA_TABLE_ENTRY,
    DriversOnly: BOOLEAN,
    InKernel: PBOOLEAN,
) -> PVOID {
    let Pc = opaque_ptr(Pc);
    let LdrEntry = opaque_ptr(LdrEntry);
    let InKernel = opaque_ptr(InKernel);
    let mut i: ULONG = 0;
    let mut PcBase: PVOID = core::ptr::null_mut();
    let mut ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;

    ListHead = if !KeLoaderBlock.is_null() {
        (KeLoaderBlock as *mut LIST_ENTRY).offset(0)
    } else {
        &mut PsLoadedModuleList as PLIST_ENTRY
    };

    *InKernel = 0u8;

    NextEntry = (*ListHead).Flink;
    if !NextEntry.is_null() {
        while NextEntry != ListHead {
            i += 1;

            if (i <= 2) && (DriversOnly != 0) {
                NextEntry = (*NextEntry).Flink;
                continue;
            }

            let Entry = ((NextEntry as *mut u8).offset(-(core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks) as isize))) as PLDR_DATA_TABLE_ENTRY;

            NextEntry = (*NextEntry).Flink;
            let ImageBase = (*Entry).DllBase;

            if ((Pc as ULONG_PTR) >= ((*Entry).DllBase as ULONG_PTR)) &&
               ((Pc as ULONG_PTR) < (((*Entry).DllBase as ULONG_PTR) + (*Entry).SizeOfImage as ULONG_PTR))
            {
                *LdrEntry = Entry;
                PcBase = ImageBase;

                if i <= 2 {
                    *InKernel = 1u8;
                }
                break;
            }
        }
    }

    PcBase
}

unsafe fn KiRosPcToUserFileHeader(
    Pc: PVOID,
    LdrEntry: *mut PLDR_DATA_TABLE_ENTRY,
) -> PVOID {
    let mut PcBase: PVOID = core::ptr::null_mut();
    let ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;

    // C: ListHead = &KeGetCurrentThread()->Teb->ProcessEnvironmentBlock->Ldr->
    //               InLoadOrderModuleList;
    // Typed bindgen field access — a previous version hand-rolled these hops with a
    // hard-coded KTHREAD.Teb offset of 0x1e8 (real-Windows x64), but this port's
    // _KTHREAD has Teb at 176 (amd64/arm64) / 116 (i386), so it read garbage on
    // every arch. (Caller guarantees validity per the C comment: only reached after
    // a successful UserMode RtlWalkFrameChain.)
    let CurrentThread = KeGetCurrentThread() as *mut ntkrnl_sys::ddk::_KTHREAD;
    let Teb = (*CurrentThread).Teb;
    let Peb = (*Teb).ProcessEnvironmentBlock;
    let Ldr = (*Peb).Ldr;
    ListHead = core::ptr::addr_of_mut!((*Ldr).InLoadOrderModuleList);

    NextEntry = (*ListHead).Flink;
    if !NextEntry.is_null() {
        while NextEntry != ListHead {
            let Entry = ((NextEntry as *mut u8).offset(-(core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks) as isize))) as PLDR_DATA_TABLE_ENTRY;

            NextEntry = (*NextEntry).Flink;
            let ImageBase = (*Entry).DllBase;

            if ((Pc as ULONG_PTR) >= ((*Entry).DllBase as ULONG_PTR)) &&
               ((Pc as ULONG_PTR) < (((*Entry).DllBase as ULONG_PTR) + (*Entry).SizeOfImage as ULONG_PTR))
            {
                *LdrEntry = Entry;
                PcBase = ImageBase;
                break;
            }
        }
    }

    PcBase
}

#[no_mangle]
pub unsafe extern "system" fn KeRosCaptureUserStackBackTrace(
    FramesToSkip: ULONG,
    FramesToCapture: ULONG,
    BackTrace: *mut PVOID,
    BackTraceHash: PULONG,
) -> USHORT {
    let BackTrace = opaque_ptr(BackTrace);
    let BackTraceHash = opaque_ptr(BackTraceHash);
    let mut Frames: [PVOID; 128] = [core::ptr::null_mut(); 128];
    let mut FrameCount: ULONG;
    let mut Hash: ULONG = 0;
    let mut i: ULONG;

    let FramesToSkip = FramesToSkip + 1;

    if (FramesToCapture + FramesToSkip) >= 128 {
        return 0;
    }

    FrameCount = RtlWalkFrameChain(Frames.as_mut_ptr(), FramesToCapture + FramesToSkip, 1);

    if FrameCount <= FramesToSkip {
        return 0;
    }

    i = 0;
    while i < FramesToCapture {
        if (FramesToSkip + i) >= FrameCount {
            break;
        }

        *BackTrace.offset(i as isize) = Frames[(FramesToSkip + i) as usize];
        Hash = Hash.wrapping_add(*BackTrace.offset(i as isize) as ULONG);
        i += 1;
    }

    if !BackTraceHash.is_null() {
        *BackTraceHash = Hash;
    }

    RtlFillMemoryUlong(Frames.as_mut_ptr() as PVOID, 128, 0);
    i as USHORT
}

unsafe fn KeRosDumpStackFrameArray(Frames: PULONG_PTR, FrameCount: ULONG) {
    let mut i: ULONG;
    let mut Addr: ULONG_PTR;
    let mut InSystem: BOOLEAN = 0;
    let mut p: PVOID;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();

    i = 0;
    while i < FrameCount {
        Addr = *Frames.offset(i as isize);
        if Addr == 0 {
            break;
        }

        if Addr > (MmHighestUserAddress as ULONG_PTR) {
            p = KiPcToFileHeader(Addr as PVOID, &mut LdrEntry, 0u8, &mut InSystem);
        } else {
            p = KiRosPcToUserFileHeader(Addr as PVOID, &mut LdrEntry);
        }

        if !p.is_null() {
            let mut AnsiName: [i8; 64] = [0; 64];
            KeBugCheckUnicodeToAnsi(&(*LdrEntry).BaseDllName as *const _ as PUNICODE_STRING, AnsiName.as_mut_ptr(), 64);
            let Addr = Addr - ((*LdrEntry).DllBase as ULONG_PTR);
            crate::dbg_print!(b"<%s: %p>\0", AnsiName.as_ptr(), Addr as PVOID);
        } else {
            crate::dbg_print!(b"<%p>\0", Addr as PVOID);
        }

        crate::dbg_print!(b"\n\0");
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeRosDumpStackFrames(Frame: PULONG_PTR, FrameCount: ULONG) {
    let Frame = opaque_ptr(Frame);
    let mut Frames: [ULONG_PTR; 32] = [0; 32];
    let mut RealFrameCount: ULONG;
    let FrameCount = if FrameCount == 0 || FrameCount > 32 { 32 } else { FrameCount };

    if !Frame.is_null() {
        KeRosDumpStackFrameArray(Frame, FrameCount);
    } else {
        RealFrameCount = RtlCaptureStackBackTrace(2, FrameCount, Frames.as_mut_ptr() as *mut PVOID, core::ptr::null_mut()) as ULONG;

        KeRosDumpStackFrameArray(Frames.as_mut_ptr(), RealFrameCount);

        if FrameCount > RealFrameCount {
            RealFrameCount = KeRosCaptureUserStackBackTrace(
                u32::MAX,
                FrameCount - RealFrameCount,
                Frames.as_mut_ptr() as *mut PVOID,
                core::ptr::null_mut(),
            ) as ULONG;

            KeRosDumpStackFrameArray(Frames.as_mut_ptr(), RealFrameCount);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn KiInitializeBugCheck() {
    let mut BugCheckData: PMESSAGE_RESOURCE_DATA = core::ptr::null_mut();
    let mut ResourceInfo: _LDR_RESOURCE_INFO = core::mem::zeroed();
    let mut ResourceDataEntry: PIMAGE_RESOURCE_DATA_ENTRY = core::ptr::null_mut();
    let Status: NTSTATUS;
    let LdrEntry: PLDR_DATA_TABLE_ENTRY;

    let LoadOrderHead = KeLoaderBlock as *mut LIST_ENTRY;
    LdrEntry = (((*LoadOrderHead).Flink as *mut u8).offset(-(core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks) as isize))) as PLDR_DATA_TABLE_ENTRY;

    ResourceInfo.Type = RT_MESSAGETABLE as _;
    ResourceInfo.Name = 1;
    ResourceInfo.Language = MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL) as _;

    Status = LdrFindResource_U(
        (*LdrEntry).DllBase,
        &mut ResourceInfo,
        RESOURCE_DATA_LEVEL,
        &mut ResourceDataEntry,
    );

    if Status >= 0 {
        let Status = LdrAccessResource(
            (*LdrEntry).DllBase,
            ResourceDataEntry,
            &mut BugCheckData as *mut _ as *mut PVOID,
            core::ptr::null_mut(),
        );
        if Status >= 0 {
            KiBugCodeMessages = BugCheckData;
        }
    }
}

/// TRUE iff every page of [p, p+len) is mapped — the crash path must never
/// itself fault (field manual §9.35 class; an early-boot bugcheck used to
/// take STATUS_IN_PAGE_ERROR right here in the message-table walk and cascade
/// into a second 0x1E, burying the real report).
unsafe fn KiBugMsgRangeValid(p: *const u8, len: usize) -> bool {
    if p.is_null() {
        return false;
    }
    let start = p as usize & !0xFFF;
    let end = (p as usize).saturating_add(len.max(1) - 1) & !0xFFF;
    let mut page = start;
    loop {
        if MmIsAddressValid(page as PVOID) == 0 {
            return false;
        }
        if page >= end {
            return true;
        }
        page += 0x1000;
    }
}

unsafe fn KeGetBugMessageText(BugCheckCode: ULONG, OutputString: PANSI_STRING) -> BOOLEAN {
    let mut i: ULONG;
    let mut IdOffset: ULONG;
    let mut MessageEntry: PMESSAGE_RESOURCE_ENTRY;
    let mut BugCode: *mut i8;
    let mut Length: USHORT;
    let mut Result: BOOLEAN = 0u8;

    if KiBugCodeMessages.is_null() {
        return Result;
    }

    // Early boot (KeLoaderBlock still live = pre-Phase1-complete): the
    // resource-section access path is not fault-safe yet — a Phase-0 bugcheck
    // took STATUS_IN_PAGE_ERROR in this walk even with page-validity checks
    // (the write-enable path itself faults), cascading into a secondary 0x1E
    // that buried the real report. Degrade to no message text — stop code,
    // parameters, and the Rust panic report (P1) still print.
    if !(KeLoaderBlock as PVOID).is_null() {
        return Result;
    }

    // Guard every hop through the resource section anyway (corrupted-image
    // case): a fault HERE means the crash reporter crashes.
    if !KiBugMsgRangeValid(
        KiBugCodeMessages as *const u8,
        core::mem::size_of::<_MESSAGE_RESOURCE_DATA>(),
    ) {
        return Result;
    }
    if !KiBugMsgRangeValid(
        (*KiBugCodeMessages).Blocks.as_ptr() as *const u8,
        ((*KiBugCodeMessages).NumberOfBlocks as usize)
            .saturating_mul(core::mem::size_of::<_MESSAGE_RESOURCE_BLOCK>()),
    ) {
        return Result;
    }

    MmMakeKernelResourceSectionWritable();

    i = 0;
    while i < (*KiBugCodeMessages).NumberOfBlocks {
        // Blocks is a C flexible array member ([_MESSAGE_RESOURCE_BLOCK; 1] in bindgen).
        // Use pointer arithmetic to avoid Rust bounds check on the [1]-length array.
        let block = &*((*KiBugCodeMessages).Blocks.as_ptr().add(i as usize));
        if (BugCheckCode >= block.LowId) && (BugCheckCode <= block.HighId) {
            MessageEntry = ((KiBugCodeMessages as ULONG_PTR) + block.OffsetToEntries as ULONG_PTR) as PMESSAGE_RESOURCE_ENTRY;
            IdOffset = BugCheckCode - block.LowId;

            // Every hop trusts an in-section Length field. Validate BEFORE
            // each dereference: a 2026-07-11 arm64 crash walked this into an
            // unmapped level-1 region, and the nested 0x1E buried the real
            // stop code (the harness then reports the mask, not the bug).
            while IdOffset > 0 {
                if !KiBugMsgRangeValid(
                    MessageEntry as *const u8,
                    core::mem::size_of::<_MESSAGE_RESOURCE_ENTRY>(),
                ) {
                    return Result;
                }
                let Hop = (*MessageEntry).Length as ULONG_PTR;
                if Hop == 0 {
                    return Result;
                }
                MessageEntry = ((MessageEntry as ULONG_PTR) + Hop) as PMESSAGE_RESOURCE_ENTRY;
                IdOffset -= 1;
            }

            if !KiBugMsgRangeValid(
                MessageEntry as *const u8,
                core::mem::size_of::<_MESSAGE_RESOURCE_ENTRY>(),
            ) {
                return Result;
            }
            let EntryLen = (*MessageEntry).Length as usize;
            if EntryLen < core::mem::size_of::<_MESSAGE_RESOURCE_ENTRY>()
                || !KiBugMsgRangeValid(MessageEntry as *const u8, EntryLen)
            {
                return Result;
            }

            BugCode = (*MessageEntry).Text.as_ptr() as *mut i8;
            // Bounded strlen: garbage text data must not scan past the
            // validated entry span.
            let TextCap = EntryLen - core::mem::offset_of!(_MESSAGE_RESOURCE_ENTRY, Text);
            let mut TextLen: usize = 0;
            while TextLen < TextCap && *BugCode.add(TextLen) != 0 {
                TextLen += 1;
            }
            Length = TextLen as USHORT;

            while (Length > 0) &&
                  ((*BugCode.offset((Length - 1) as isize) == b'\n' as i8) ||
                   (*BugCode.offset((Length - 1) as isize) == b'\r' as i8) ||
                   (*BugCode.offset((Length - 1) as isize) == ANSI_NULL as i8))
            {
                if OutputString.is_null() {
                    *BugCode.offset((Length - 1) as isize) = ANSI_NULL as i8;
                }
                Length -= 1;
            }

            if !OutputString.is_null() {
                (*OutputString).Buffer = BugCode;
                (*OutputString).Length = Length;
                (*OutputString).MaximumLength = Length;
            } else {
                InbvDisplayString(BugCode);
                InbvDisplayString(b"\r\0".as_ptr() as *const i8);
            }

            Result = 1u8;
            break;
        }
        i += 1;
    }

    Result
}

unsafe fn KiDoBugCheckCallbacks() {
    let mut CurrentRecord: PKBUGCHECK_CALLBACK_RECORD;
    let ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut LastEntry: PLIST_ENTRY;
    let mut Checksum: ULONG_PTR;

    ListHead = &mut KeBugcheckCallbackListHead as PLIST_ENTRY;
    if (*ListHead).Flink.is_null() || (*ListHead).Blink.is_null() {
        return;
    }

    LastEntry = ListHead;
    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        CurrentRecord = ((NextEntry as *mut u8).offset(-(core::mem::offset_of!(_KBUGCHECK_CALLBACK_RECORD, Entry) as isize))) as PKBUGCHECK_CALLBACK_RECORD;

        if (*CurrentRecord).Entry.Blink != LastEntry {
            return;
        }

        Checksum = (*CurrentRecord).CallbackRoutine.unwrap() as ULONG_PTR;
        Checksum = Checksum.wrapping_add((*CurrentRecord).Buffer as ULONG_PTR);
        Checksum = Checksum.wrapping_add((*CurrentRecord).Length as ULONG_PTR);
        Checksum = Checksum.wrapping_add((*CurrentRecord).Component as ULONG_PTR);

        if ((*CurrentRecord).State == BufferInserted) && ((*CurrentRecord).Checksum == Checksum as _) {
            (*CurrentRecord).State = BufferStarted;
            ((*CurrentRecord).CallbackRoutine.unwrap())((*CurrentRecord).Buffer, (*CurrentRecord).Length);
            (*CurrentRecord).State = BufferFinished;
        }

        LastEntry = NextEntry;
        NextEntry = (*NextEntry).Flink;
    }
}

unsafe fn KiBugCheckDebugBreak(StatusCode: ULONG) {
    loop {
        DbgBreakPointWithStatus(StatusCode);
        HalHaltSystem();
        if StatusCode != DBG_STATUS_BUGCHECK_FIRST {
            continue;
        }
        break;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeBugCheckUnicodeToAnsi(
    Unicode: PUNICODE_STRING,
    Ansi: *mut i8,
    Length: ULONG,
) -> *mut i8 {
    let Unicode = opaque_ptr(Unicode);
    let Ansi = opaque_ptr(Ansi);
    let mut p: *mut i8;
    let mut pw: *mut u16;
    let mut i: ULONG;

    i = (*Unicode).Length as ULONG / core::mem::size_of::<u16>() as ULONG;
    if i > Length - 1 {
        i = Length - 1;
    }

    pw = (*Unicode).Buffer;
    p = Ansi;
    while i > 0 {
        *p = (*pw & 0xFF) as i8;
        p = p.offset(1);
        pw = pw.offset(1);
        i -= 1;
    }

    *p = ANSI_NULL as i8;
    Ansi
}

unsafe fn KiDumpParameterImages(
    Message: *mut i8,
    Parameters: PULONG_PTR,
    ParameterCount: ULONG,
    ConversionRoutine: PKE_BUGCHECK_UNICODE_TO_ANSI,
) {
    let mut i: ULONG;
    let mut InSystem: BOOLEAN = 0;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut ImageBase: PVOID;
    let mut DriverName: PUNICODE_STRING;
    let mut AnsiName: [i8; 32] = [0; 32];
    let mut NtHeader: PIMAGE_NT_HEADERS;
    let mut TimeStamp: ULONG;
    let mut FirstRun: BOOLEAN = 1u8;

    i = 0;
    while i < ParameterCount {
        ImageBase = KiPcToFileHeader(
            *Parameters.offset(i as isize) as PVOID,
            &mut LdrEntry,
            0u8,
            &mut InSystem,
        );
        if ImageBase.is_null() {
            i += 1;
            continue;
        } else {
            NtHeader = RtlImageNtHeader((*LdrEntry).DllBase);
            TimeStamp = (*NtHeader).FileHeader.TimeDateStamp;

            DriverName = &mut (*LdrEntry).BaseDllName;
            (ConversionRoutine.unwrap())(
                &(*LdrEntry).BaseDllName as *const _ as PUNICODE_STRING,
                AnsiName.as_mut_ptr(),
                core::mem::size_of_val(&AnsiName) as ULONG,
            );
        }

        sprintf(
            Message,
            b"%s**  %12s - Address %p base at %p, DateStamp %08lx\r\n\0".as_ptr() as *const i8,
            if FirstRun != 0 { b"\r\n*\0".as_ptr() as *const i8 } else { b"*\0".as_ptr() as *const i8 },
            AnsiName.as_ptr(),
            *Parameters.offset(i as isize) as PVOID,
            ImageBase,
            TimeStamp,
        );

        if ParameterCount <= 1 {
            KiBugCheckDriver = DriverName;
        } else {
            InbvDisplayString(Message);
        }

        FirstRun = 0u8;
        i += 1;
    }
}

unsafe fn KiDisplayBlueScreen(
    MessageId: ULONG,
    IsHardError: BOOLEAN,
    HardErrCaption: *mut i8,
    HardErrMessage: *mut i8,
    Message: *mut i8,
) {
    let BugCheckCode = KiBugCheckData[0] as ULONG;
    let mut Enable: BOOLEAN = 1u8;
    let mut AnsiName: [i8; 107] = [0; 107];

    HeadlessDispatch(
        HeadlessCmdStartBugCheck,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    HeadlessDispatch(
        HeadlessCmdEnableTerminal,
        &mut Enable as *mut _ as PVOID,
        core::mem::size_of::<BOOLEAN>(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    HeadlessDispatch(
        HeadlessCmdSendBlueScreenData,
        &BugCheckCode as *const _ as PVOID,
        core::mem::size_of::<ULONG>(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if InbvIsBootDriverInstalled() != 0 {
        InbvAcquireDisplayOwnership();
        InbvResetDisplay();

        // Rust panic → RED screen; C/classic stop → blue. See RUST_PANIC_BUGCODE.
        let fill = if BugCheckCode == RUST_PANIC_BUGCODE {
            BV_COLOR_RED
        } else {
            BV_COLOR_BLUE
        };
        InbvSolidColorFill(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, fill);
        InbvSetTextColor(BV_COLOR_WHITE);
        InbvInstallDisplayStringFilter(None);
        InbvEnableDisplayString(1u8);
        InbvSetScrollRegion(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1);
    }

    if IsHardError != 0 {
        if !HardErrCaption.is_null() {
            InbvDisplayString(HardErrCaption);
        }
        if !HardErrMessage.is_null() {
            InbvDisplayString(HardErrMessage);
        }
    }

    InbvDisplayString(b"\r\n\0".as_ptr() as *const i8);

    KeGetBugMessageText(BUGCHECK_MESSAGE_INTRO, core::ptr::null_mut());
    InbvDisplayString(b"\r\n\r\n\0".as_ptr() as *const i8);

    if !KiBugCheckDriver.is_null() {
        KeGetBugMessageText(BUGCODE_ID_DRIVER, core::ptr::null_mut());

        KeBugCheckUnicodeToAnsi(KiBugCheckDriver, AnsiName.as_mut_ptr(), core::mem::size_of_val(&AnsiName) as ULONG);
        InbvDisplayString(b" \0".as_ptr() as *const i8);
        InbvDisplayString(AnsiName.as_ptr());
        InbvDisplayString(b"\r\n\r\n\0".as_ptr() as *const i8);
    }

    if MessageId == BUGCODE_PSS_MESSAGE {
        KeGetBugMessageText(BugCheckCode, core::ptr::null_mut());
        InbvDisplayString(b"\r\n\r\n\0".as_ptr() as *const i8);
    }

    KeGetBugMessageText(PSS_MESSAGE_INTRO, core::ptr::null_mut());
    InbvDisplayString(b"\r\n\r\n\0".as_ptr() as *const i8);

    KeGetBugMessageText(MessageId, core::ptr::null_mut());
    InbvDisplayString(b"\r\n\r\n\0".as_ptr() as *const i8);

    KeGetBugMessageText(BUGCHECK_TECH_INFO, core::ptr::null_mut());

    crate::rtl_string_cb_printf_a!(
        AnsiName.as_mut_ptr(),
        core::mem::size_of_val(&AnsiName),
        b"\r\n\r\n*** STOP: 0x%08lX (0x%p,0x%p,0x%p,0x%p)\r\n\r\n",
        BugCheckCode,
        KiBugCheckData[1] as PVOID,
        KiBugCheckData[2] as PVOID,
        KiBugCheckData[3] as PVOID,
        KiBugCheckData[4] as PVOID,
    );
    InbvDisplayString(AnsiName.as_ptr());

    // Rust panic: P1 points at panic-common's static report buffer — the
    // panic message, source file:line, and raw-PC backtrace. Surface it all
    // on screen (the Rust wins Windows structurally cannot show).
    if BugCheckCode == RUST_PANIC_BUGCODE && KiBugCheckData[1] != 0 {
        InbvDisplayString(b"\r\n\0".as_ptr() as *const i8);
        InbvDisplayString(KiBugCheckData[1] as *const i8);
        InbvDisplayString(b"\r\n\0".as_ptr() as *const i8);
    }

    if !KiBugCheckDriver.is_null() {
        InbvDisplayString(Message);
    } else {
        KiDumpParameterImages(
            Message,
            &mut KiBugCheckData[1] as *mut _ as PVOID as PULONG_PTR,
            4,
            core::mem::transmute::<_, PKE_BUGCHECK_UNICODE_TO_ANSI>(KeBugCheckUnicodeToAnsi as usize),
        );
    }
}

// Set once the bugcheck banner has printed (by Ex or WithTf) — prevents a
// double banner on the Ex->WithTf path.
static mut KiBannerDone: BOOLEAN = 0;

// Flight-recorder sidecar prints exactly once per crash, independent of the
// banner guard (which the KeBugCheckEx path pre-sets before reaching WithTf).
static mut KiFlightrecDumped: BOOLEAN = 0;

#[no_mangle]
pub unsafe extern "system" fn KeBugCheckWithTf(
    BugCheckCode: ULONG,
    BugCheckParameter1: ULONG_PTR,
    BugCheckParameter2: ULONG_PTR,
    BugCheckParameter3: ULONG_PTR,
    BugCheckParameter4: ULONG_PTR,
    TrapFrame: PKTRAP_FRAME,
) -> ! {
    let Prcb = KeGetCurrentPrcb();
    let mut Context: CONTEXT = core::mem::zeroed();
    let mut MessageId: ULONG;
    let mut AnsiName: [i8; 128] = [0; 128];
    let mut IsSystem: BOOLEAN = 0;
    let mut IsHardError: BOOLEAN = 0u8;
    let mut Reboot: BOOLEAN = 0u8;
    let mut HardErrCaption: *mut i8 = core::ptr::null_mut();
    let mut HardErrMessage: *mut i8 = core::ptr::null_mut();
    let mut Pc: PVOID = core::ptr::null_mut();
    let mut Memory: PVOID;
    let mut DriverBase: PVOID;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut HardErrorParameters: PULONG_PTR;
    let mut OldIrql: KIRQL = 0;

    KeBugCheckActive = 1;
    KiBugCheckDriver = core::ptr::null_mut();

    // arm64: park the HAL IRQL tripwire — this path runs at HIGH_LEVEL by
    // design, so the display path's DISPATCH-level lock acquires would spam
    // raise-DOWN/lower-UP violations between the banner and the diagnostics.
    #[cfg(target_arch = "aarch64")]
    {
        extern "system" {
            fn HalVibeosQuiesceIrqlTripwire();
        }
        HalVibeosQuiesceIrqlTripwire();
    }

    // Direct callers (KiSystemFatalException on double fault, trap handlers)
    // reach WithTf WITHOUT going through KeBugCheckEx — historically NO banner
    // printed, and when the BSOD display path then faulted (KeGetBugMessageText
    // in-page, 2026-07-06), the nested 0x1E banner MASKED the primary bugcheck
    // entirely. Print the banner here when Ex hasn't already (KiBannerDone).
    if KiBannerDone == 0 {
        KiBannerDone = 1;
        crate::dbg_print!(b"\n*** BUGCHECK: 0x%08lX (0x%p, 0x%p, 0x%p, 0x%p) [via WithTf]\n\0",
            BugCheckCode,
            BugCheckParameter1 as PVOID,
            BugCheckParameter2 as PVOID,
            BugCheckParameter3 as PVOID,
            BugCheckParameter4 as PVOID,
        );
        // i386: symbolicate TrapFrame.Eip + walk the EBP chain — a PC=0 bugcheck
        // is otherwise unattributable (nothing to symbolicate at the fault).
        #[cfg(target_arch = "x86")] // nolint: cfg-vs-c (Rust-only diagnostic; not the C's CONFIG_SMP block)
        if !TrapFrame.is_null() {
            i386_trap_frame_backtrace(TrapFrame as ULONG_PTR);
        }
        #[cfg(target_arch = "x86_64")] // nolint: cfg-vs-c (Rust-only diagnostic; not the C's CONFIG_SMP block)
        if !TrapFrame.is_null() {
            let rip = (*(TrapFrame as *const ntkrnl_sys::ddk::_KTRAP_FRAME)).Rip as u64;
            if rip > 0xFFFF800000000000 {
                let mut le: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
                let mut in_kernel: BOOLEAN = 0;
                let base = KiPcToFileHeader(rip as PVOID, &mut le, 0u8, &mut in_kernel);
                if !base.is_null() && !le.is_null() {
                    let mut ansi = [0i8; 64];
                    KeBugCheckUnicodeToAnsi(
                        &(*le).BaseDllName as *const _ as PUNICODE_STRING,
                        ansi.as_mut_ptr(),
                        64,
                    );
                    crate::dbg_print!(
                        b"***   TrapFrame.Rip = 0x%p = %s+0x%lX\n\0",
                        rip as PVOID,
                        ansi.as_ptr(),
                        (rip as ULONG_PTR - base as ULONG_PTR) as ULONG,
                    );
                }
            }
            // Mini-backtrace off the trap frame's Rsp (same scan as the P3 path
            // below). Critical for 0x7F double faults: the recursion cycle that
            // blew the stack is only visible on the overflowed stack itself.
            let rsp = (*(TrapFrame as *const ntkrnl_sys::ddk::_KTRAP_FRAME)).Rsp as u64;
            if rsp > 0xFFFF800000000000 {
                // Innermost frames (the repeating cycle on a blown stack)...
                let mut printed = 0u32;
                let mut qi = 0u64;
                while qi < 256 && printed < 12 {
                    let sp = rsp + qi * 8;
                    let val = *(sp as *const u64);
                    if val > 0xFFFF800000000000 {
                        let mut le2: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
                        let mut in_kernel2: BOOLEAN = 0;
                        let base2 = KiPcToFileHeader(val as PVOID, &mut le2, 0u8, &mut in_kernel2);
                        if !base2.is_null() && !le2.is_null() {
                            let mut ansi2 = [0i8; 64];
                            KeBugCheckUnicodeToAnsi(
                                &(*le2).BaseDllName as *const _ as PUNICODE_STRING,
                                ansi2.as_mut_ptr(),
                                64,
                            );
                            crate::dbg_print!(
                                b"***   tfstack[+0x%03lX] = %s+0x%lX\n\0",
                                (qi * 8) as ULONG,
                                ansi2.as_ptr(),
                                (val as ULONG_PTR - base2 as ULONG_PTR) as ULONG,
                            );
                            printed += 1;
                        }
                    }
                    qi += 1;
                }
                // ...then the OUTERMOST frames (original context that started the
                // recursion) — ring-buffer the last 16 resolvable entries across
                // the whole plausible stack extent and dump them.
                let mut ring: [(u64, u64); 64] = [(0, 0); 64];
                let mut nhits: u64 = 0;
                let mut qj = 0u64;
                while qj < 16384 {
                    let sp = rsp + qj * 8;
                    // Guard each page: walking past StackBase into unmapped VA
                    // would nest-fault inside the bugcheck path.
                    if sp & 0xFFF == 0 || qj == 0 {
                        if ntkrnl_sys::ddk::MmIsAddressValid(sp as PVOID) == 0 {
                            break;
                        }
                    }
                    let val = *(sp as *const u64);
                    if val > 0xFFFF800000000000 {
                        let mut le3: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
                        let mut in_kernel3: BOOLEAN = 0;
                        let base3 = KiPcToFileHeader(val as PVOID, &mut le3, 0u8, &mut in_kernel3);
                        if !base3.is_null() && !le3.is_null() {
                            ring[(nhits % 64) as usize] = (qj * 8, val);
                            nhits += 1;
                        }
                    }
                    qj += 1;
                }
                // Oldest nested KTRAP_FRAME = the FIRST fault of the recursion.
                // Each KiTrap0E invocation leaves the same return address on the
                // stack (the recursion is a fixed cycle); the highest-offset match
                // is the first fault, and its KTRAP_FRAME sits just above the slot.
                {
                    let ret0 = *((rsp + 0x18) as *const u64);
                    let mut best_sp: u64 = 0;
                    let mut qk = 0u64;
                    while qk < 16384 {
                        let sp = rsp + qk * 8;
                        if sp & 0xFFF == 0 || qk == 0 {
                            if ntkrnl_sys::ddk::MmIsAddressValid(sp as PVOID) == 0 {
                                break;
                            }
                        }
                        if *(sp as *const u64) == ret0 {
                            best_sp = sp;
                        }
                        qk += 1;
                    }
                    if best_sp != 0 {
                        let tf0 = (best_sp + 8) as *const ntkrnl_sys::ddk::_KTRAP_FRAME;
                        crate::dbg_print!(
                            b"***   FIRST-FAULT KTRAP_FRAME @0x%p: Rip=0x%p FaultAddress=0x%p Rsp=0x%p\n\0",
                            tf0 as PVOID,
                            (*tf0).Rip as PVOID,
                            (*tf0).__bindgen_anon_2.FaultAddress as PVOID,
                            (*tf0).Rsp as PVOID,
                        );
                        // Callers of the faulting leaf: symbolicate the first
                        // resolvable slots above the faulting frame's Rsp.
                        let frsp = (*tf0).Rsp as u64;
                        let mut fp = 0u32;
                        let mut fq = 0u64;
                        while fq < 64 && fp < 8 {
                            let sp = frsp + fq * 8;
                            if ntkrnl_sys::ddk::MmIsAddressValid(sp as PVOID) != 0 {
                                let val = *(sp as *const u64);
                                if val > 0xFFFF800000000000 {
                                    let mut le5: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
                                    let mut ik5: BOOLEAN = 0;
                                    let base5 = KiPcToFileHeader(val as PVOID, &mut le5, 0u8, &mut ik5);
                                    if !base5.is_null() && !le5.is_null() {
                                        let mut ansi5 = [0i8; 64];
                                        KeBugCheckUnicodeToAnsi(
                                            &(*le5).BaseDllName as *const _ as PUNICODE_STRING,
                                            ansi5.as_mut_ptr(),
                                            64,
                                        );
                                        crate::dbg_print!(
                                            b"***   ffstack[+0x%03lX] = %s+0x%lX\n\0",
                                            (fq * 8) as ULONG,
                                            ansi5.as_ptr(),
                                            (val as ULONG_PTR - base5 as ULONG_PTR) as ULONG,
                                        );
                                        fp += 1;
                                    }
                                }
                            }
                            fq += 1;
                        }
                    }
                }
                if nhits > 12 {
                    crate::dbg_print!(b"***   --- outermost frames (of %lu hits) ---\n\0", nhits as ULONG);
                    let start = if nhits >= 64 { nhits - 64 } else { 0 };
                    let mut k = start;
                    while k < nhits {
                        let (soff, val) = ring[(k % 64) as usize];
                        let mut le4: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
                        let mut in_kernel4: BOOLEAN = 0;
                        let base4 = KiPcToFileHeader(val as PVOID, &mut le4, 0u8, &mut in_kernel4);
                        if !base4.is_null() && !le4.is_null() {
                            let mut ansi4 = [0i8; 64];
                            KeBugCheckUnicodeToAnsi(
                                &(*le4).BaseDllName as *const _ as PUNICODE_STRING,
                                ansi4.as_mut_ptr(),
                                64,
                            );
                            crate::dbg_print!(
                                b"***   tfstack[+0x%04lX] = %s+0x%lX\n\0",
                                soff as ULONG,
                                ansi4.as_ptr(),
                                (val as ULONG_PTR - base4 as ULONG_PTR) as ULONG,
                            );
                        }
                        k += 1;
                    }
                }
            }
        }
    }

    // Flight-recorder sidecar (Layer 3): dump the tail of the sync-event ring —
    // the last gate/pushlock/WS-lock/lockdep operations before the crash, with
    // their threads. For lost-wakeup / self-deadlock / ABBA bugchecks this is
    // the smoking gun that a post-mortem backtrace alone can't show. OUTSIDE the
    // KiBannerDone guard: the KeBugCheckEx path sets KiBannerDone before calling
    // here, so anything gated on it never runs on a Rust-panic/Ex bugcheck. Own
    // once-guard (KiFlightrecDumped) so it prints exactly once per crash.
    if KiFlightrecDumped == 0 {
        KiFlightrecDumped = 1;
        super::flightrec::flightrec_dump_recent(32);
        // ...and this thread's OWN history, which the tail window cannot show.
        // The unfiltered tail is dominated by whatever lock traffic happened to
        // run last -- on the #39 capture it was two park events on an unrelated
        // thread -- so an earlier kernel fault that THIS thread recovered from
        // (OP_KERNEL_FAULT_PC / OP_KERNEL_RESUME_PC) falls outside it and is
        // neither shown nor excluded. Scans the full 16K ring.
        super::flightrec::flightrec_dump_thread(
            crate::ke::ke_defs::KeGetCurrentThread() as usize,
            64,
        );
    }

    if BugCheckCode == POWER_FAILURE_SIMULATE {
        KiDoBugCheckCallbacks();
        HalReturnToFirmware(HalRebootRoutine);
    }

    (*Prcb).DebuggerSavedIRQL = KeGetCurrentIrql();
    InterlockedIncrement(&mut KiHardwareTrigger as *mut _ as *mut LONG);

    RtlCaptureContext(&mut (*Prcb).ProcessorState.ContextFrame);
    KiSaveProcessorControlState(&mut (*Prcb).ProcessorState);
    Context = (*Prcb).ProcessorState.ContextFrame;

    MessageId = match BugCheckCode {
        UNEXPECTED_KERNEL_MODE_TRAP |
        DRIVER_CORRUPTED_EXPOOL |
        ACPI_BIOS_ERROR |
        ACPI_BIOS_FATAL_ERROR |
        THREAD_STUCK_IN_DEVICE_DRIVER |
        DATA_BUS_ERROR |
        FAT_FILE_SYSTEM |
        NO_MORE_SYSTEM_PTES |
        INACCESSIBLE_BOOT_DEVICE => BugCheckCode,
        KERNEL_MODE_EXCEPTION_NOT_HANDLED |
        SYSTEM_THREAD_EXCEPTION_NOT_HANDLED |
        KMODE_EXCEPTION_NOT_HANDLED => KMODE_EXCEPTION_NOT_HANDLED,
        NTFS_FILE_SYSTEM => FAT_FILE_SYSTEM,
        DRIVER_CORRUPTED_MMPOOL => DRIVER_CORRUPTED_EXPOOL,
        STATUS_SYSTEM_IMAGE_BAD_SIGNATURE => BUGCODE_PSS_MESSAGE_SIGNATURE,
        _ => BUGCODE_PSS_MESSAGE,
    };

    KiBugCheckData[0] = BugCheckCode as ULONG_PTR;
    KiBugCheckData[1] = BugCheckParameter1;
    KiBugCheckData[2] = BugCheckParameter2;
    KiBugCheckData[3] = BugCheckParameter3;
    KiBugCheckData[4] = BugCheckParameter4;

    let mut TrapFrame = TrapFrame;

    match BugCheckCode {
        KERNEL_MODE_EXCEPTION_NOT_HANDLED |
        ATTEMPTED_WRITE_TO_READONLY_MEMORY |
        ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY => {
            if TrapFrame.is_null() {
                if BugCheckParameter3 != 0 {
                    TrapFrame = BugCheckParameter3 as PVOID as PKTRAP_FRAME;
                }
            }

            if !TrapFrame.is_null() && BugCheckCode != KERNEL_MODE_EXCEPTION_NOT_HANDLED {
                Pc = KeGetTrapFramePc(TrapFrame) as PVOID;
            }
        }
        IRQL_NOT_LESS_OR_EQUAL => {
            Pc = BugCheckParameter4 as PVOID;

            DriverBase = KiPcToFileHeader(Pc, &mut LdrEntry, 0u8, &mut IsSystem);
            if IsSystem != 0 {
                Memory = BugCheckParameter1 as PVOID;

                DriverBase = KiPcToFileHeader(Memory, &mut LdrEntry, 1u8, &mut IsSystem);
                if !DriverBase.is_null() {
                    KiBugCheckDriver = &mut (*LdrEntry).BaseDllName;
                    KiBugCheckData[0] = DRIVER_PORTION_MUST_BE_NONPAGED as ULONG_PTR;
                } else {
                    KiBugCheckDriver = core::ptr::null_mut();

                    if !KiBugCheckDriver.is_null() {
                        KiBugCheckData[0] = SYSTEM_SCAN_AT_RAISED_IRQL_CAUGHT_IMPROPER_DRIVER_UNLOAD as ULONG_PTR;
                    }
                }
            } else {
                KiBugCheckData[0] = DRIVER_IRQL_NOT_LESS_OR_EQUAL as ULONG_PTR;
            }

            Pc = core::ptr::null_mut();
        }
        FATAL_UNHANDLED_HARD_ERROR => {
            HardErrorParameters = BugCheckParameter2 as PULONG_PTR;
            KiBugCheckData[0] = BugCheckParameter1;
            KiBugCheckData[1] = *HardErrorParameters.offset(0);
            KiBugCheckData[2] = *HardErrorParameters.offset(1);
            KiBugCheckData[3] = *HardErrorParameters.offset(2);
            KiBugCheckData[4] = *HardErrorParameters.offset(3);

            IsHardError = 1u8;
            HardErrCaption = BugCheckParameter3 as *mut i8;
            HardErrMessage = BugCheckParameter4 as *mut i8;
        }
        PAGE_FAULT_IN_NONPAGED_AREA => {
            DriverBase = core::ptr::null_mut();

            if TrapFrame.is_null() {
                if BugCheckParameter3 != 0 {
                    TrapFrame = BugCheckParameter3 as PVOID as PKTRAP_FRAME;
                }
            }

            if !TrapFrame.is_null() {
                Pc = KeGetTrapFramePc(TrapFrame) as PVOID;
                KiBugCheckData[3] = Pc as ULONG_PTR;

                DriverBase = KiPcToFileHeader(Pc, &mut LdrEntry, 0u8, &mut IsSystem);
            } else {
                IsSystem = 1u8;
            }

            if MmIsSpecialPoolAddress(BugCheckParameter1 as PVOID) != 0 {
                if MmIsSpecialPoolAddressFree(BugCheckParameter1 as PVOID) != 0 {
                    KiBugCheckData[0] = if IsSystem != 0 {
                        PAGE_FAULT_IN_FREED_SPECIAL_POOL
                    } else {
                        DRIVER_PAGE_FAULT_IN_FREED_SPECIAL_POOL
                    } as ULONG_PTR;
                } else {
                    KiBugCheckData[0] = if IsSystem != 0 {
                        PAGE_FAULT_BEYOND_END_OF_ALLOCATION
                    } else {
                        DRIVER_PAGE_FAULT_BEYOND_END_OF_ALLOCATION
                    } as ULONG_PTR;
                }
            } else if DriverBase.is_null() {
                KiBugCheckDriver = core::ptr::null_mut();

                if !KiBugCheckDriver.is_null() {
                    KiBugCheckData[0] = DRIVER_UNLOADED_WITHOUT_CANCELLING_PENDING_OPERATIONS as ULONG_PTR;
                }
            }
        }
        DRIVER_LEFT_LOCKED_PAGES_IN_PROCESS => {
            Pc = BugCheckParameter1 as PVOID;
        }
        DRIVER_USED_EXCESSIVE_PTES => {
            LdrEntry = BugCheckParameter1 as PVOID as PLDR_DATA_TABLE_ENTRY;
            KiBugCheckDriver = &mut (*LdrEntry).BaseDllName;
        }
        THREAD_STUCK_IN_DEVICE_DRIVER => {
            KiBugCheckDriver = BugCheckParameter3 as PVOID as PUNICODE_STRING;
        }
        _ => {}
    }

    if !KiBugCheckDriver.is_null() {
        KeBugCheckUnicodeToAnsi(KiBugCheckDriver, AnsiName.as_mut_ptr(), core::mem::size_of_val(&AnsiName) as ULONG);
    } else {
        if !Pc.is_null() {
            KiDumpParameterImages(
                AnsiName.as_mut_ptr(),
                &mut Pc as *mut _ as PULONG_PTR,
                1,
                core::mem::transmute::<_, PKE_BUGCHECK_UNICODE_TO_ANSI>(KeBugCheckUnicodeToAnsi as usize),
            );
        }
    }

    if KdPitchDebugger == 0 {
        KdDebuggerDataBlock = &Context as *const _ as ULONG_PTR as PVOID;
    }

    if (BugCheckCode != MANUALLY_INITIATED_CRASH) && (KdDebuggerEnabled != 0) {
        crate::dbg_print!(b"\n*** Fatal System Error: 0x%08lX\n                       (0x%p,0x%p,0x%p,0x%p)\n\n\0",
            KiBugCheckData[0],
            KiBugCheckData[1] as PVOID,
            KiBugCheckData[2] as PVOID,
            KiBugCheckData[3] as PVOID,
            KiBugCheckData[4] as PVOID,
        );

        if KdDebuggerNotPresent == 0 {
            if !KiBugCheckDriver.is_null() {
                crate::dbg_print!(b"Driver at fault: %s.\n\0", AnsiName.as_ptr());
            }

            if IsHardError != 0 {
                if !HardErrCaption.is_null() {
                    crate::dbgprint::rust_dbg_print_cstr(HardErrCaption, &[]);
                }
                if !HardErrMessage.is_null() {
                    crate::dbgprint::rust_dbg_print_cstr(HardErrMessage, &[]);
                }
            }

            KiBugCheckDebugBreak(DBG_STATUS_BUGCHECK_FIRST);
        }
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("cli", options(nomem, nostack));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("msr daifset, #2", options(nomem, nostack));
    KeRaiseIrql(HIGH_LEVEL, &mut OldIrql);

    if InterlockedDecrement(&mut KeBugCheckCount as *mut _ as *mut LONG) == 0 {
        // bug.c: `#ifdef CONFIG_SMP` — the SMP kernel on every arch, never the UP one.
        // This was gated on x86/x86_64, which froze in UP builds and never on arm64 SMP.
        #[cfg(feature = "smp")]
        {
            KeBugCheckOwner = (*Prcb).Number as ULONG;
            KxFreezeExecution();
        }

        KiDisplayBlueScreen(MessageId, IsHardError, HardErrCaption, HardErrMessage, AnsiName.as_mut_ptr());

        if (KdDebuggerEnabled == 0) && (KdPitchDebugger == 0) {
            KdInitSystem(0, core::ptr::null_mut());
        } else {
            InbvDisplayString(b"\r\n\0".as_ptr() as *const i8);
        }

        (*Prcb).ProcessorState.ContextFrame = Context;

        Reboot = if IopAutoReboot != 0 { 1u8 } else { 0u8 };
    } else {
        KeBugCheckOwnerRecursionCount += 1;
        if KeBugCheckOwnerRecursionCount == 2 {
            KiBugCheckDebugBreak(DBG_STATUS_BUGCHECK_SECOND);
        } else if KeBugCheckOwnerRecursionCount > 2 {
            loop {}
        }
    }

    KiDoBugCheckCallbacks();

    if Reboot != 0 {
        DbgUnLoadImageSymbols(core::ptr::null_mut(), usize::MAX as PVOID, 0);
        HalReturnToFirmware(HalRebootRoutine);
    }

    KiBugCheckDebugBreak(DBG_STATUS_BUGCHECK_SECOND);

    loop {}
}

#[no_mangle]
pub unsafe extern "system" fn KiHandleNmi() -> BOOLEAN {
    let mut Handled: BOOLEAN = 0u8;
    let mut NmiData: PKNMI_HANDLER_CALLBACK;

    NmiData = KiNmiCallbackListHead;
    while !NmiData.is_null() {
        Handled |= ((*NmiData).Callback.unwrap())((*NmiData).Context, Handled);
        NmiData = (*NmiData).Next;
    }

    Handled
}

#[no_mangle]
pub unsafe extern "system" fn KeInitializeCrashDumpHeader(
    Type: ULONG,
    Flags: ULONG,
    Buffer: PVOID,
    BufferSize: ULONG,
    BufferNeeded: ULONG,
) -> NTSTATUS {
    let Buffer = opaque_ptr(Buffer);
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn KeDeregisterBugCheckCallback(
    CallbackRecord: PKBUGCHECK_CALLBACK_RECORD,
) -> BOOLEAN {
    let mut OldIrql: KIRQL = 0;
    let mut Status: BOOLEAN = 0u8;

    // [guard] KeRaiseIrql(HIGH_LEVEL, &mut OldIrql);
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(HIGH_LEVEL) };

    if (*CallbackRecord).State == BufferInserted {
        (*CallbackRecord).State = BufferEmpty;
        RemoveEntryList(&mut (*CallbackRecord).Entry);
        Status = 1u8;
    }

    drop(irql_guard);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn KeDeregisterBugCheckReasonCallback(
    CallbackRecord: PKBUGCHECK_REASON_CALLBACK_RECORD,
) -> BOOLEAN {
    let mut OldIrql: KIRQL = 0;
    let mut Status: BOOLEAN = 0u8;

    // [guard] KeRaiseIrql(HIGH_LEVEL, &mut OldIrql);
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(HIGH_LEVEL) };

    if (*CallbackRecord).State == BufferInserted {
        (*CallbackRecord).State = BufferEmpty;
        RemoveEntryList(&mut (*CallbackRecord).Entry);
        Status = 1u8;
    }

    drop(irql_guard);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn KeRegisterBugCheckCallback(
    CallbackRecord: PKBUGCHECK_CALLBACK_RECORD,
    CallbackRoutine: PKBUGCHECK_CALLBACK_ROUTINE,
    Buffer: PVOID,
    Length: ULONG,
    Component: *mut u8,
) -> BOOLEAN {
    let Buffer = opaque_ptr(Buffer);
    let Component = opaque_ptr(Component);
    let mut OldIrql: KIRQL = 0;
    let mut Status: BOOLEAN = 0u8;

    // [guard] KeRaiseIrql(HIGH_LEVEL, &mut OldIrql);
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(HIGH_LEVEL) };

    if (*CallbackRecord).State == BufferEmpty {
        (*CallbackRecord).Length = Length;
        (*CallbackRecord).Buffer = Buffer;
        (*CallbackRecord).Component = Component;
        (*CallbackRecord).CallbackRoutine = CallbackRoutine;
        (*CallbackRecord).State = BufferInserted;
        InsertTailList(&mut KeBugcheckCallbackListHead, &mut (*CallbackRecord).Entry);
        Status = 1u8;
    }

    drop(irql_guard);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn KeRegisterBugCheckReasonCallback(
    CallbackRecord: PKBUGCHECK_REASON_CALLBACK_RECORD,
    CallbackRoutine: PKBUGCHECK_REASON_CALLBACK_ROUTINE,
    Reason: KBUGCHECK_CALLBACK_REASON,
    Component: *mut u8,
) -> BOOLEAN {
    let Component = opaque_ptr(Component);
    let mut OldIrql: KIRQL = 0;
    let mut Status: BOOLEAN = 0u8;

    // [guard] KeRaiseIrql(HIGH_LEVEL, &mut OldIrql);
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(HIGH_LEVEL) };

    if (*CallbackRecord).State == BufferEmpty {
        (*CallbackRecord).Component = Component;
        (*CallbackRecord).CallbackRoutine = CallbackRoutine;
        (*CallbackRecord).State = BufferInserted;
        (*CallbackRecord).Reason = Reason;
        InsertTailList(&mut KeBugcheckReasonCallbackListHead, &mut (*CallbackRecord).Entry);
        Status = 1u8;
    }

    drop(irql_guard);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn KeRegisterNmiCallback(
    CallbackRoutine: PNMI_CALLBACK,
    Context: PVOID,
) -> PVOID {
    let Context = opaque_ptr(Context);
    let mut OldIrql: KIRQL = 0;
    let NmiData: PKNMI_HANDLER_CALLBACK;
    let Next: PKNMI_HANDLER_CALLBACK;

    NmiData = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<_KNMI_HANDLER_CALLBACK>(), TAG_KNMI) as PKNMI_HANDLER_CALLBACK;
    if NmiData.is_null() {
        return core::ptr::null_mut();
    }

    (*NmiData).Callback = CallbackRoutine;
    (*NmiData).Context = Context;
    (*NmiData).Handle = NmiData as PVOID;

    KiAcquireNmiListLock(&mut OldIrql);
    (*NmiData).Next = KiNmiCallbackListHead;
    Next = InterlockedCompareExchangePointer(
        &mut KiNmiCallbackListHead as *mut _ as *mut PVOID,
        NmiData as PVOID,
        (*NmiData).Next as PVOID,
    ) as PKNMI_HANDLER_CALLBACK;
    KiReleaseNmiListLock(OldIrql);

    (*NmiData).Handle
}

#[no_mangle]
pub unsafe extern "system" fn KeDeregisterNmiCallback(Handle: PVOID) -> NTSTATUS {
    let Handle = opaque_ptr(Handle);
    let mut OldIrql: KIRQL = 0;
    let mut NmiData: PKNMI_HANDLER_CALLBACK;
    let mut Previous: *mut PKNMI_HANDLER_CALLBACK;

    KiAcquireNmiListLock(&mut OldIrql);
    Previous = &mut KiNmiCallbackListHead;
    NmiData = *Previous;
    while !NmiData.is_null() {
        if (*NmiData).Handle == Handle {
            *Previous = (*NmiData).Next;
            break;
        }

        Previous = &mut (*NmiData).Next;
        NmiData = *Previous;
    }
    KiReleaseNmiListLock(OldIrql);

    if !NmiData.is_null() {
        ExFreePoolWithTag(NmiData as PVOID, TAG_KNMI);
        return STATUS_SUCCESS;
    }

    STATUS_INVALID_HANDLE
}

/// i386: symbolicate a trap frame's Eip and walk its EBP chain (frame-pointer builds; bails on
/// a bad link). Shared by KeBugCheckWithTf and KeBugCheckEx's 0x1E, whose P3 on i386 IS the
/// trap frame (KiDispatchException) — that crash used to print no stack at all.
#[cfg(target_arch = "x86")]
unsafe fn i386_trap_frame_backtrace(TrapFrame: ULONG_PTR) {
    if MmIsAddressValid(TrapFrame as PVOID) == 0 {
        return;
    }
    let tf = TrapFrame as *const ntkrnl_sys::ddk::_KTRAP_FRAME;
    let eip = (*tf).Eip as u32;
    let ebp = (*tf).Ebp as u32;
    crate::dbg_print!(b"***   TrapFrame Eip=%p Ebp=%p Esp=%p\n\0",
        eip as ULONG_PTR as PVOID, ebp as ULONG_PTR as PVOID,
        core::ptr::addr_of!((*tf).HardwareEsp) as PVOID);
    unsafe fn ksym(pc: u32) -> bool {
        let mut le: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
        let mut in_kernel: BOOLEAN = 0;
        let base = KiPcToFileHeader(pc as ULONG_PTR as PVOID, &mut le, 0u8, &mut in_kernel);
        if !base.is_null() && !le.is_null() {
            let mut ansi = [0i8; 64];
            KeBugCheckUnicodeToAnsi(
                &(*le).BaseDllName as *const _ as PUNICODE_STRING,
                ansi.as_mut_ptr(), 64);
            crate::dbg_print!(b"***     %p = %s+0x%lX\n\0",
                pc as ULONG_PTR as PVOID, ansi.as_ptr(),
                (pc as ULONG_PTR - base as ULONG_PTR) as ULONG);
            true
        } else {
            crate::dbg_print!(b"***     %p = <no module>\n\0", pc as ULONG_PTR as PVOID);
            false
        }
    }
    if !ksym(eip) {
        // EIP outside every module means a call through a bad pointer, and then
        // EBP is whatever the garbage "function" left -- the chain below walks
        // nothing. The CALLER's return address is still on the stack, at or near
        // the trap's ESP (kernel-mode: the address of HardwareEsp). Print every
        // word there that lands in a module; the first is usually the bad call.
        crate::dbg_print!(b"***   EIP is in no module; stack words that are code addresses:\n\0");
        let sp = core::ptr::addr_of!((*tf).HardwareEsp) as u32;
        let mut i = 0u32;
        while i < 32 {
            let at = sp + i * 4;
            if MmIsAddressValid(at as ULONG_PTR as PVOID) == 0 {
                break;
            }
            let w = core::ptr::read_volatile(at as *const u32);
            let mut le: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
            let mut in_kernel: BOOLEAN = 0;
            if w > 0x1000
                && !KiPcToFileHeader(w as ULONG_PTR as PVOID, &mut le, 0u8, &mut in_kernel).is_null()
            {
                crate::dbg_print!(b"***   [esp+%02lx]\n\0", (i * 4) as ULONG);
                ksym(w);
            }
            i += 1;
        }
    }
    // EBP-chain backtrace (frame-pointer builds; bail on bad links)
    let mut bp = ebp;
    let mut n = 0;
    while n < 16 && bp > 0x1000 {
        if MmIsAddressValid(bp as ULONG_PTR as PVOID) == 0
            || MmIsAddressValid((bp + 7) as ULONG_PTR as PVOID) == 0 {
            break;
        }
        let ret = core::ptr::read_volatile((bp + 4) as *const u32);
        if ret < 0x1000 { break; }
        let _ = ksym(ret);
        let next = core::ptr::read_volatile(bp as *const u32);
        if next <= bp { break; }
        bp = next;
        n += 1;
    }

}

#[no_mangle]
pub unsafe extern "system" fn KeBugCheckEx(
    BugCheckCode: ULONG,
    BugCheckParameter1: ULONG_PTR,
    BugCheckParameter2: ULONG_PTR,
    BugCheckParameter3: ULONG_PTR,
    BugCheckParameter4: ULONG_PTR,
) -> ! {
    // Capture the caller's return address (x30) as the VERY FIRST thing, before
    // dbg_print!/any bl clobbers it — that's the bugcheck raise site. The later
    // `mov x30` read (below) was post-dbg_print! so it printed garbage.
    #[cfg(target_arch = "aarch64")]
    let entry_lr: ULONG_PTR = {
        let lr: ULONG_PTR;
        core::arch::asm!("mov {}, x30", out(reg) lr, options(nomem, nostack));
        lr
    };
    crate::dbg_print!(b"\n*** BUGCHECK: 0x%08lX (0x%p, 0x%p, 0x%p, 0x%p)\n\0",
        BugCheckCode,
        BugCheckParameter1 as PVOID,
        BugCheckParameter2 as PVOID,
        BugCheckParameter3 as PVOID,
        BugCheckParameter4 as PVOID,
    );
    KiBannerDone = 1; // WithTf must not re-print (Ex -> WithTf path)
    // 0x50/0xBE pass MmAccessFault's TrapInformation as P3 too (ARM3/pagfault.rs). Without
    // this an i386 0x50 printed four bare numbers: P3 is a stack address, in no module.
    #[cfg(target_arch = "x86")] // nolint: cfg-vs-c (bugcheck backtrace diagnostic, Rust-only)
    if matches!(BugCheckCode, 0x1E | 0x50 | 0xBE) && BugCheckParameter3 > 0x80000000 {
        i386_trap_frame_backtrace(BugCheckParameter3);
    }
    // Self-symbolicate: any parameter that lands inside a loaded image prints
    // module+offset right in the banner, so post-boot-marker crashes are
    // self-reporting on serial (no GDB round-trip; module bases vary per boot,
    // making post-mortem cross-boot symbolication unreliable). Best-effort —
    // KiPcToFileHeader only reads PsLoadedModuleList, safe at bugcheck time.
    {
        let params = [
            BugCheckParameter1,
            BugCheckParameter2,
            BugCheckParameter3,
            BugCheckParameter4,
        ];
        let mut idx = 0usize;
        while idx < 4 {
            let p = params[idx];
            if p > 0x10000 {
                let mut le: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
                let mut in_kernel: BOOLEAN = 0;
                let base = KiPcToFileHeader(p as PVOID, &mut le, 0u8, &mut in_kernel);
                if !base.is_null() && !le.is_null() {
                    let mut ansi = [0i8; 64];
                    KeBugCheckUnicodeToAnsi(
                        &(*le).BaseDllName as *const _ as PUNICODE_STRING,
                        ansi.as_mut_ptr(),
                        64,
                    );
                    crate::dbg_print!(
                        b"***   P%u = %s+0x%lX\n\0",
                        (idx + 1) as u32,
                        ansi.as_ptr(),
                        (p - base as ULONG_PTR) as ULONG,
                    );
                }
            }
            idx += 1;
        }
        // Mm bugchecks (0x50 PAGE_FAULT_IN_NONPAGED_AREA, 0xBE
        // ATTEMPTED_WRITE_TO_READONLY_MEMORY, 0x1E-family) pass TrapInformation
        // as P3, not a PC — the real faulting PC is inside that KTRAP_FRAME.
        // Extract and symbolicate it too (bindgen field access, not a raw offset).
        #[cfg(target_arch = "x86_64")] // nolint: cfg-vs-c (bugcheck backtrace diagnostic, Rust-only)
        if (BugCheckCode == 0x50 || BugCheckCode == 0xBE || BugCheckCode == 0x7E || BugCheckCode == 0x1E)
            && BugCheckParameter3 > 0xFFFF800000000000
        {
            let rip = (*(BugCheckParameter3 as *const ntkrnl_sys::ddk::_KTRAP_FRAME)).Rip as u64;
            if rip > 0xFFFF800000000000 {
                let mut le: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
                let mut in_kernel: BOOLEAN = 0;
                let base = KiPcToFileHeader(rip as PVOID, &mut le, 0u8, &mut in_kernel);
                if !base.is_null() && !le.is_null() {
                    let mut ansi = [0i8; 64];
                    KeBugCheckUnicodeToAnsi(
                        &(*le).BaseDllName as *const _ as PUNICODE_STRING,
                        ansi.as_mut_ptr(),
                        64,
                    );
                    crate::dbg_print!(
                        b"***   TrapFrame.Rip = 0x%p = %s+0x%lX\n\0",
                        rip as PVOID,
                        ansi.as_ptr(),
                        (rip as ULONG_PTR - base as ULONG_PTR) as ULONG,
                    );
                } else {
                    crate::dbg_print!(b"***   TrapFrame.Rip = 0x%p (no module)\n\0", rip as PVOID);
                }
            }
            // Mini-backtrace: scan the faulting stack for return addresses that
            // land inside loaded images (scan-based, frame-pointer-free — noisy
            // but names the caller chain without a debugger).
            let rsp = (*(BugCheckParameter3 as *const ntkrnl_sys::ddk::_KTRAP_FRAME)).Rsp as u64;
            if rsp > 0xFFFF800000000000 {
                let mut printed = 0u32;
                let mut qi = 0u64;
                while qi < 64 && printed < 8 {
                    let sp = rsp + qi * 8;
                    let val = *(sp as *const u64);
                    if val > 0xFFFF800000000000 {
                        let mut le: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
                        let mut in_kernel: BOOLEAN = 0;
                        let base = KiPcToFileHeader(val as PVOID, &mut le, 0u8, &mut in_kernel);
                        if !base.is_null() && !le.is_null() {
                            let mut ansi = [0i8; 64];
                            KeBugCheckUnicodeToAnsi(
                                &(*le).BaseDllName as *const _ as PUNICODE_STRING,
                                ansi.as_mut_ptr(),
                                64,
                            );
                            crate::dbg_print!(
                                b"***   stack[+0x%03lX] = %s+0x%lX\n\0",
                                (qi * 8) as ULONG,
                                ansi.as_ptr(),
                                (val as ULONG_PTR - base as ULONG_PTR) as ULONG,
                            );
                            printed += 1;
                        }
                    }
                    qi += 1;
                }
            }
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        let u = Arm64UartKernelBase as *mut ULONG;
        let mut s: &[u8] = b"\r\n*** BUGCHECK: code=0x\0";
        let mut i = 0;
        while s[i] != 0 {
            crate::arm64_dbg::uart_putc_default(s[i] as u8);
            i += 1;
        }
        let lr: ULONG_PTR = entry_lr;
        { let mut shift: i32 = 28; while shift >= 0 {
            let d = ((BugCheckCode >> shift) & 0xF) as u8;
            crate::arm64_dbg::uart_putc_default(if d < 10 { b'0' + d } else { b'A' + d - 10 } as u8);
            shift -= 4;
        } }
        s = b" LR=0x\0";
        i = 0;
        while s[i] != 0 {
            crate::arm64_dbg::uart_putc_default(s[i] as u8);
            i += 1;
        }
        { let mut shift: i32 = 60; while shift >= 0 {
            let d = ((lr >> shift) & 0xF) as u8;
            crate::arm64_dbg::uart_putc_default(if d < 10 { b'0' + d } else { b'A' + d - 10 } as u8);
            shift -= 4;
        } }
        s = b" P1=0x\0";
        i = 0;
        while s[i] != 0 {
            crate::arm64_dbg::uart_putc_default(s[i] as u8);
            i += 1;
        }
        { let mut shift: i32 = 60; while shift >= 0 {
            let d = ((BugCheckParameter1 >> shift) & 0xF) as u8;
            crate::arm64_dbg::uart_putc_default(if d < 10 { b'0' + d } else { b'A' + d - 10 } as u8);
            shift -= 4;
        } }
        s = b" P2=0x\0";
        i = 0;
        while s[i] != 0 {
            crate::arm64_dbg::uart_putc_default(s[i] as u8);
            i += 1;
        }
        { let mut shift: i32 = 60; while shift >= 0 {
            let d = ((BugCheckParameter2 >> shift) & 0xF) as u8;
            crate::arm64_dbg::uart_putc_default(if d < 10 { b'0' + d } else { b'A' + d - 10 } as u8);
            shift -= 4;
        } }
        s = b" P3=0x\0";
        i = 0;
        while s[i] != 0 {
            crate::arm64_dbg::uart_putc_default(s[i] as u8);
            i += 1;
        }
        { let mut shift: i32 = 60; while shift >= 0 {
            let d = ((BugCheckParameter3 >> shift) & 0xF) as u8;
            crate::arm64_dbg::uart_putc_default(if d < 10 { b'0' + d } else { b'A' + d - 10 } as u8);
            shift -= 4;
        } }
        s = b" P4=0x\0";
        i = 0;
        while s[i] != 0 {
            crate::arm64_dbg::uart_putc_default(s[i] as u8);
            i += 1;
        }
        { let mut shift: i32 = 60; while shift >= 0 {
            let d = ((BugCheckParameter4 >> shift) & 0xF) as u8;
            crate::arm64_dbg::uart_putc_default(if d < 10 { b'0' + d } else { b'A' + d - 10 } as u8);
            shift -= 4;
        } }
        s = b"\r\n\0";
        i = 0;
        while s[i] != 0 {
            crate::arm64_dbg::uart_putc_default(s[i] as u8);
            i += 1;
        }
    }

    // For KMODE_EXCEPTION_NOT_HANDLED (0x1E), P3 is the TrapFrame pointer
    let tf = if BugCheckCode == 0x1E {
        BugCheckParameter3 as *mut _
    } else {
        core::ptr::null_mut()
    };
    KeBugCheckWithTf(
        BugCheckCode,
        BugCheckParameter1,
        BugCheckParameter2,
        BugCheckParameter3,
        BugCheckParameter4,
        tf,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KeBugCheck(BugCheckCode: ULONG) -> ! {
    // Capture caller x30 first, before any bl clobbers it (the raise site).
    #[cfg(target_arch = "aarch64")]
    let entry_lr: ULONG_PTR = {
        let lr: ULONG_PTR;
        core::arch::asm!("mov {}, x30", out(reg) lr, options(nomem, nostack));
        lr
    };
    // Debugcon: dump return address so we can identify the caller
    #[cfg(target_arch = "x86")] // nolint: cfg-vs-c (bugcheck backtrace diagnostic, Rust-only)
    {
        let retaddr: u32;
        core::arch::asm!(
            "mov {}, [ebp+4]",
            out(reg) retaddr,
            options(nomem, nostack, preserves_flags)
        );
        let mut shift: i32 = 28;
        while shift >= 0 {
            let d = ((retaddr >> shift) & 0xF) as u8;
            let ch = if d < 10 { b'0' + d } else { b'a' + d - 10 };
            shift -= 4;
        }
    }
    crate::dbg_print!(b"\n*** BUGCHECK: 0x%08lX\n\0", BugCheckCode);
    #[cfg(target_arch = "aarch64")]
    {
        let u = Arm64UartKernelBase as *mut ULONG;
        let mut s: &[u8] = b"\r\n*** BUGCHECK: code=0x\0";
        let mut i = 0;
        while s[i] != 0 {
            crate::arm64_dbg::uart_putc_default(s[i] as u8);
            i += 1;
        }
        let lr: ULONG_PTR = entry_lr;
        { let mut shift: i32 = 28; while shift >= 0 {
            let d = ((BugCheckCode >> shift) & 0xF) as u8;
            crate::arm64_dbg::uart_putc_default(if d < 10 { b'0' + d } else { b'A' + d - 10 } as u8);
            shift -= 4;
        } }
        s = b" LR=0x\0";
        i = 0;
        while s[i] != 0 {
            crate::arm64_dbg::uart_putc_default(s[i] as u8);
            i += 1;
        }
        { let mut shift: i32 = 60; while shift >= 0 {
            let d = ((lr >> shift) & 0xF) as u8;
            crate::arm64_dbg::uart_putc_default(if d < 10 { b'0' + d } else { b'A' + d - 10 } as u8);
            shift -= 4;
        } }
        s = b"\r\n\0";
        i = 0;
        while s[i] != 0 {
            crate::arm64_dbg::uart_putc_default(s[i] as u8);
            i += 1;
        }
    }

    KeBugCheckWithTf(BugCheckCode, 0, 0, 0, 0, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn KeEnterKernelDebugger() {
    KiHardwareTrigger = 1;
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("cli", options(nomem, nostack));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("msr daifset, #2", options(nomem, nostack));

    if InterlockedDecrement(&mut KeBugCheckCount as *mut _ as *mut LONG) == 0 {
        if (KdDebuggerEnabled == 0) && (KdPitchDebugger == 0) {
            KdInitSystem(0, core::ptr::null_mut());
        }
    }

    KiBugCheckDebugBreak(DBG_STATUS_FATAL);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/bug.c
//  * PURPOSE:         Bugcheck Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
//
// #ifdef KDBG
// #include <kdbg/kdb.h>
// #endif
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// LIST_ENTRY KeBugcheckCallbackListHead;
// LIST_ENTRY KeBugcheckReasonCallbackListHead;
// KSPIN_LOCK BugCheckCallbackLock;
// ULONG KeBugCheckActive, KeBugCheckOwner;
// LONG KeBugCheckOwnerRecursionCount;
// PMESSAGE_RESOURCE_DATA KiBugCodeMessages;
// ULONG KeBugCheckCount = 1;
// ULONG KiHardwareTrigger;
// PUNICODE_STRING KiBugCheckDriver;
// ULONG_PTR KiBugCheckData[5];
//
// PKNMI_HANDLER_CALLBACK KiNmiCallbackListHead = NULL;
// KSPIN_LOCK KiNmiCallbackListLock;
//
// /* Jira Reporting */
// UNICODE_STRING KeRosProcessorName, KeRosBiosDate, KeRosBiosVersion;
// UNICODE_STRING KeRosVideoBiosDate, KeRosVideoBiosVersion;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// PVOID
// NTAPI
// KiPcToFileHeader(IN PVOID Pc,
//                  OUT PLDR_DATA_TABLE_ENTRY *LdrEntry,
//                  IN BOOLEAN DriversOnly,
//                  OUT PBOOLEAN InKernel)
// {
//     ULONG i = 0;
//     PVOID ImageBase, PcBase = NULL;
//     PLDR_DATA_TABLE_ENTRY Entry;
//     PLIST_ENTRY ListHead, NextEntry;
//
//     /* Check which list we should use */
//     ListHead = (KeLoaderBlock) ? &KeLoaderBlock->LoadOrderListHead :
//                                  &PsLoadedModuleList;
//
//     /* Assume no */
//     *InKernel = FALSE;
//
//     /* Set list pointers and make sure it's valid */
//     NextEntry = ListHead->Flink;
//     if (NextEntry)
//     {
//         /* Start loop */
//         while (NextEntry != ListHead)
//         {
//             /* Increase entry */
//             i++;
//
//             /* Check if this is a kernel entry and we only want drivers */
//             if ((i <= 2) && (DriversOnly != FALSE))
//             {
//                 /* Skip it */
//                 NextEntry = NextEntry->Flink;
//                 continue;
//             }
//
//             /* Get the loader entry */
//             Entry = CONTAINING_RECORD(NextEntry,
//                                       LDR_DATA_TABLE_ENTRY,
//                                       InLoadOrderLinks);
//
//             /* Move to the next entry */
//             NextEntry = NextEntry->Flink;
//             ImageBase = Entry->DllBase;
//
//             /* Check if this is the right one */
//             if (((ULONG_PTR)Pc >= (ULONG_PTR)Entry->DllBase) &&
//                 ((ULONG_PTR)Pc < ((ULONG_PTR)Entry->DllBase + Entry->SizeOfImage)))
//             {
//                 /* Return this entry */
//                 *LdrEntry = Entry;
//                 PcBase = ImageBase;
//
//                 /* Check if this was a kernel or HAL entry */
//                 if (i <= 2) *InKernel = TRUE;
//                 break;
//             }
//         }
//     }
//
//     /* Return the base address */
//     return PcBase;
// }
//
// PVOID
// NTAPI
// KiRosPcToUserFileHeader(IN PVOID Pc,
//                         OUT PLDR_DATA_TABLE_ENTRY *LdrEntry)
// {
//     PVOID ImageBase, PcBase = NULL;
//     PLDR_DATA_TABLE_ENTRY Entry;
//     PLIST_ENTRY ListHead, NextEntry;
//
//     /*
//      * We know this is valid because we should only be called after a
//      * succesfull address from RtlWalkFrameChain for UserMode, which
//      * validates everything for us.
//      */
//     ListHead = &KeGetCurrentThread()->
//                Teb->ProcessEnvironmentBlock->Ldr->InLoadOrderModuleList;
//
//     /* Set list pointers and make sure it's valid */
//     NextEntry = ListHead->Flink;
//     if (NextEntry)
//     {
//         /* Start loop */
//         while (NextEntry != ListHead)
//         {
//             /* Get the loader entry */
//             Entry = CONTAINING_RECORD(NextEntry,
//                                       LDR_DATA_TABLE_ENTRY,
//                                       InLoadOrderLinks);
//
//             /* Move to the next entry */
//             NextEntry = NextEntry->Flink;
//             ImageBase = Entry->DllBase;
//
//             /* Check if this is the right one */
//             if (((ULONG_PTR)Pc >= (ULONG_PTR)Entry->DllBase) &&
//                 ((ULONG_PTR)Pc < ((ULONG_PTR)Entry->DllBase + Entry->SizeOfImage)))
//             {
//                 /* Return this entry */
//                 *LdrEntry = Entry;
//                 PcBase = ImageBase;
//                 break;
//             }
//         }
//     }
//
//     /* Return the base address */
//     return PcBase;
// }
//
// USHORT
// NTAPI
// KeRosCaptureUserStackBackTrace(IN ULONG FramesToSkip,
//                                IN ULONG FramesToCapture,
//                                OUT PVOID *BackTrace,
//                                OUT PULONG BackTraceHash OPTIONAL)
// {
//     PVOID Frames[2 * 64];
//     ULONG FrameCount;
//     ULONG Hash = 0, i;
//
//     /* Skip a frame for the caller */
//     FramesToSkip++;
//
//     /* Don't go past the limit */
//     if ((FramesToCapture + FramesToSkip) >= 128) return 0;
//
//     /* Do the back trace */
//     FrameCount = RtlWalkFrameChain(Frames, FramesToCapture + FramesToSkip, 1);
//
//     /* Make sure we're not skipping all of them */
//     if (FrameCount <= FramesToSkip) return 0;
//
//     /* Loop all the frames */
//     for (i = 0; i < FramesToCapture; i++)
//     {
//         /* Don't go past the limit */
//         if ((FramesToSkip + i) >= FrameCount) break;
//
//         /* Save this entry and hash it */
//         BackTrace[i] = Frames[FramesToSkip + i];
//         Hash += PtrToUlong(BackTrace[i]);
//     }
//
//     /* Write the hash */
//     if (BackTraceHash) *BackTraceHash = Hash;
//
//     /* Clear the other entries and return count */
//     RtlFillMemoryUlong(Frames, 128, 0);
//     return (USHORT)i;
// }
//
//
// VOID
// FASTCALL
// KeRosDumpStackFrameArray(IN PULONG_PTR Frames,
//                          IN ULONG FrameCount)
// {
//     ULONG i;
//     ULONG_PTR Addr;
//     BOOLEAN InSystem;
//     PVOID p;
//
//     /* GCC complaints that it may be used uninitialized */
//     PLDR_DATA_TABLE_ENTRY LdrEntry = NULL;
//
//     /* Loop them */
//     for (i = 0; i < FrameCount; i++)
//     {
//         /* Get the EIP */
//         Addr = Frames[i];
//         if (!Addr)
//         {
//         	break;
//         }
//
//         /* Get the base for this file */
//         if (Addr > (ULONG_PTR)MmHighestUserAddress)
//         {
//             /* We are in kernel */
//             p = KiPcToFileHeader((PVOID)Addr, &LdrEntry, FALSE, &InSystem);
//         }
//         else
//         {
//             /* We are in user land */
//             p = KiRosPcToUserFileHeader((PVOID)Addr, &LdrEntry);
//         }
//         if (p)
//         {
// #ifdef KDBG
//             if (!KdbSymPrintAddress((PVOID)Addr, NULL))
// #endif
//             {
//                 CHAR AnsiName[64];
//
//                 /* Convert module name to ANSI and print it */
//                 KeBugCheckUnicodeToAnsi(&LdrEntry->BaseDllName,
//                                         AnsiName,
//                                         sizeof(AnsiName));
//                 Addr -= (ULONG_PTR)LdrEntry->DllBase;
//                 DbgPrint("<%s: %p>", AnsiName, (PVOID)Addr);
//             }
//         }
//         else
//         {
//             /* Print only the address */
//             DbgPrint("<%p>", (PVOID)Addr);
//         }
//
//         /* Go to the next frame */
//         DbgPrint("\n");
//     }
// }
//
// VOID
// NTAPI
// KeRosDumpStackFrames(IN PULONG_PTR Frame OPTIONAL,
//                      IN ULONG FrameCount OPTIONAL)
// {
//     ULONG_PTR Frames[32];
//     ULONG RealFrameCount;
//
//     /* If the caller didn't ask, assume 32 frames */
//     if (!FrameCount || FrameCount > 32) FrameCount = 32;
//
//     if (Frame)
//     {
//         /* Dump them */
//         KeRosDumpStackFrameArray(Frame, FrameCount);
//     }
//     else
//     {
//         /* Get the current frames (skip the two. One for the dumper, one for the caller) */
//         RealFrameCount = RtlCaptureStackBackTrace(2, FrameCount, (PVOID*)Frames, NULL);
//         DPRINT1("RealFrameCount =%lu\n", RealFrameCount);
//
//         /* Dump them */
//         KeRosDumpStackFrameArray(Frames, RealFrameCount);
//
//         /* Count left for user mode? */
//         if (FrameCount - RealFrameCount > 0)
//         {
//             /* Get the current frames */
//             RealFrameCount = KeRosCaptureUserStackBackTrace(-1, FrameCount - RealFrameCount, (PVOID*)Frames, NULL);
//
//             /* Dump them */
//             KeRosDumpStackFrameArray(Frames, RealFrameCount);
//         }
//     }
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// KiInitializeBugCheck(VOID)
// {
//     PMESSAGE_RESOURCE_DATA BugCheckData;
//     LDR_RESOURCE_INFO ResourceInfo;
//     PIMAGE_RESOURCE_DATA_ENTRY ResourceDataEntry;
//     NTSTATUS Status;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//
//     /* Get the kernel entry */
//     LdrEntry = CONTAINING_RECORD(KeLoaderBlock->LoadOrderListHead.Flink,
//                                  LDR_DATA_TABLE_ENTRY,
//                                  InLoadOrderLinks);
//
//     /* Cache the bugcheck message strings. Prepare the lookup data. */
//     ResourceInfo.Type = RT_MESSAGETABLE;
//     ResourceInfo.Name = 1;
//     ResourceInfo.Language = MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL);
//
//     /* Do the lookup */
//     Status = LdrFindResource_U(LdrEntry->DllBase,
//                                &ResourceInfo,
//                                RESOURCE_DATA_LEVEL,
//                                &ResourceDataEntry);
//
//     /* Make sure it worked */
//     if (NT_SUCCESS(Status))
//     {
//         /* Now actually get a pointer to it */
//         Status = LdrAccessResource(LdrEntry->DllBase,
//                                    ResourceDataEntry,
//                                    (PVOID*)&BugCheckData,
//                                    NULL);
//         if (NT_SUCCESS(Status)) KiBugCodeMessages = BugCheckData;
//     }
// }
//
// BOOLEAN
// NTAPI
// KeGetBugMessageText(IN ULONG BugCheckCode,
//                     OUT PANSI_STRING OutputString OPTIONAL)
// {
//     ULONG i;
//     ULONG IdOffset;
//     PMESSAGE_RESOURCE_ENTRY MessageEntry;
//     PCHAR BugCode;
//     USHORT Length;
//     BOOLEAN Result = FALSE;
//
//     /* Make sure we're not bugchecking too early */
//     if (!KiBugCodeMessages) return Result;
//
//     /*
//      * Globally protect in SEH as we are trying to access data in
//      * dire situations, and potentially going to patch it (see below).
//      */
//     _SEH2_TRY
//     {
//
//     /*
//      * Make the kernel resource section writable, as we are going to manually
//      * trim the trailing newlines in the bugcheck resource message in place,
//      * when OutputString is NULL and before displaying it on screen.
//      */
//     MmMakeKernelResourceSectionWritable();
//
//     /* Find the message. This code is based on RtlFindMesssage */
//     for (i = 0; i < KiBugCodeMessages->NumberOfBlocks; i++)
//     {
//         /* Check if the ID matches */
//         if ((BugCheckCode >= KiBugCodeMessages->Blocks[i].LowId) &&
//             (BugCheckCode <= KiBugCodeMessages->Blocks[i].HighId))
//         {
//             /* Get offset to entry */
//             MessageEntry = (PMESSAGE_RESOURCE_ENTRY)
//                 ((ULONG_PTR)KiBugCodeMessages + KiBugCodeMessages->Blocks[i].OffsetToEntries);
//             IdOffset = BugCheckCode - KiBugCodeMessages->Blocks[i].LowId;
//
//             /* Advance in the entries until finding it */
//             while (IdOffset--)
//             {
//                 MessageEntry = (PMESSAGE_RESOURCE_ENTRY)
//                     ((ULONG_PTR)MessageEntry + MessageEntry->Length);
//             }
//
//             /* Make sure it's not Unicode */
//             ASSERT(!(MessageEntry->Flags & MESSAGE_RESOURCE_UNICODE));
//
//             /* Get the final code */
//             BugCode = (PCHAR)MessageEntry->Text;
//             Length = (USHORT)strlen(BugCode);
//
//             /* Handle trailing newlines */
//             while ((Length > 0) && ((BugCode[Length - 1] == '\n') ||
//                                     (BugCode[Length - 1] == '\r') ||
//                                     (BugCode[Length - 1] == ANSI_NULL)))
//             {
//                 /* Directly trim the newline in place if we don't return the string */
//                 if (!OutputString) BugCode[Length - 1] = ANSI_NULL;
//
//                 /* Skip the trailing newline */
//                 Length--;
//             }
//
//             /* Check if caller wants an output string */
//             if (OutputString)
//             {
//                 /* Return it in the OutputString */
//                 OutputString->Buffer = BugCode;
//                 OutputString->Length = Length;
//                 OutputString->MaximumLength = Length;
//             }
//             else
//             {
//                 /* Direct output to screen */
//                 InbvDisplayString(BugCode);
//                 InbvDisplayString("\r");
//             }
//
//             /* We're done */
//             Result = TRUE;
//             break;
//         }
//     }
//
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//     }
//     _SEH2_END;
//
//     /* Return the result */
//     return Result;
// }
//
// VOID
// NTAPI
// KiDoBugCheckCallbacks(VOID)
// {
//     PKBUGCHECK_CALLBACK_RECORD CurrentRecord;
//     PLIST_ENTRY ListHead, NextEntry, LastEntry;
//     ULONG_PTR Checksum;
//
//     /* First make sure that the list is initialized... it might not be */
//     ListHead = &KeBugcheckCallbackListHead;
//     if ((!ListHead->Flink) || (!ListHead->Blink))
//         return;
//
//     /* Loop the list */
//     LastEntry = ListHead;
//     NextEntry = ListHead->Flink;
//     while (NextEntry != ListHead)
//     {
//         /* Get the record */
//         CurrentRecord = CONTAINING_RECORD(NextEntry,
//                                           KBUGCHECK_CALLBACK_RECORD,
//                                           Entry);
//
//         /* Validate it */
//         // TODO/FIXME: Check whether the memory CurrentRecord points to
//         // is still accessible and valid!
//         if (CurrentRecord->Entry.Blink != LastEntry) return;
//         Checksum = (ULONG_PTR)CurrentRecord->CallbackRoutine;
//         Checksum += (ULONG_PTR)CurrentRecord->Buffer;
//         Checksum += (ULONG_PTR)CurrentRecord->Length;
//         Checksum += (ULONG_PTR)CurrentRecord->Component;
//
//         /* Make sure it's inserted and validated */
//         if ((CurrentRecord->State == BufferInserted) &&
//             (CurrentRecord->Checksum == Checksum))
//         {
//             /* Call the routine */
//             CurrentRecord->State = BufferStarted;
//             _SEH2_TRY
//             {
//                 (CurrentRecord->CallbackRoutine)(CurrentRecord->Buffer,
//                                                  CurrentRecord->Length);
//                 CurrentRecord->State = BufferFinished;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 CurrentRecord->State = BufferIncomplete;
//             }
//             _SEH2_END;
//         }
//
//         /* Go to the next entry */
//         LastEntry = NextEntry;
//         NextEntry = NextEntry->Flink;
//     }
// }
//
// VOID
// NTAPI
// KiBugCheckDebugBreak(IN ULONG StatusCode)
// {
//     /*
//      * Wrap this in SEH so we don't crash if
//      * there is no debugger or if it disconnected
//      */
// DoBreak:
//     _SEH2_TRY
//     {
//         /* Breakpoint */
//         DbgBreakPointWithStatus(StatusCode);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         /* No debugger, halt the CPU */
//         HalHaltSystem();
//     }
//     _SEH2_END;
//
//     /* Break again if this wasn't first try */
//     if (StatusCode != DBG_STATUS_BUGCHECK_FIRST) goto DoBreak;
// }
//
// PCHAR
// NTAPI
// KeBugCheckUnicodeToAnsi(IN PUNICODE_STRING Unicode,
//                         OUT PCHAR Ansi,
//                         IN ULONG Length)
// {
//     PCHAR p;
//     PWCHAR pw;
//     ULONG i;
//
//     /* Set length and normalize it */
//     i = Unicode->Length / sizeof(WCHAR);
//     i = min(i, Length - 1);
//
//     /* Set source and destination, and copy */
//     pw = Unicode->Buffer;
//     p = Ansi;
//     while (i--) *p++ = (CHAR)*pw++;
//
//     /* Null terminate and return */
//     *p = ANSI_NULL;
//     return Ansi;
// }
//
// VOID
// NTAPI
// KiDumpParameterImages(IN PCHAR Message,
//                       IN PULONG_PTR Parameters,
//                       IN ULONG ParameterCount,
//                       IN PKE_BUGCHECK_UNICODE_TO_ANSI ConversionRoutine)
// {
//     ULONG i;
//     BOOLEAN InSystem;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PVOID ImageBase;
//     PUNICODE_STRING DriverName;
//     CHAR AnsiName[32];
//     PIMAGE_NT_HEADERS NtHeader;
//     ULONG TimeStamp;
//     BOOLEAN FirstRun = TRUE;
//
//     /* Loop parameters */
//     for (i = 0; i < ParameterCount; i++)
//     {
//         /* Get the base for this parameter */
//         ImageBase = KiPcToFileHeader((PVOID)Parameters[i],
//                                      &LdrEntry,
//                                      FALSE,
//                                      &InSystem);
//         if (!ImageBase)
//         {
//             /* FIXME: Add code to check for unloaded drivers */
//             DPRINT1("Potentially unloaded driver!\n");
//             continue;
//         }
//         else
//         {
//             /* Get the NT Headers and Timestamp */
//             NtHeader = RtlImageNtHeader(LdrEntry->DllBase);
//             TimeStamp = NtHeader->FileHeader.TimeDateStamp;
//
//             /* Convert the driver name */
//             DriverName = &LdrEntry->BaseDllName;
//             ConversionRoutine(&LdrEntry->BaseDllName,
//                               AnsiName,
//                               sizeof(AnsiName));
//         }
//
//         /* Format driver name */
//         sprintf(Message,
//                 "%s**  %12s - Address %p base at %p, DateStamp %08lx\r\n",
//                 FirstRun ? "\r\n*":"*",
//                 AnsiName,
//                 (PVOID)Parameters[i],
//                 ImageBase,
//                 TimeStamp);
//
//         /* Check if we only had one parameter */
//         if (ParameterCount <= 1)
//         {
//             /* Then just save the name */
//             KiBugCheckDriver = DriverName;
//         }
//         else
//         {
//             /* Otherwise, display the message */
//             InbvDisplayString(Message);
//         }
//
//         /* Loop again */
//         FirstRun = FALSE;
//     }
// }
//
// VOID
// NTAPI
// KiDisplayBlueScreen(IN ULONG MessageId,
//                     IN BOOLEAN IsHardError,
//                     IN PCHAR HardErrCaption OPTIONAL,
//                     IN PCHAR HardErrMessage OPTIONAL,
//                     IN PCHAR Message)
// {
//     ULONG BugCheckCode = (ULONG)KiBugCheckData[0];
//     BOOLEAN Enable = TRUE;
//     CHAR AnsiName[107];
//
//     /* Enable headless support for bugcheck */
//     HeadlessDispatch(HeadlessCmdStartBugCheck,
//                      NULL, 0, NULL, NULL);
//     HeadlessDispatch(HeadlessCmdEnableTerminal,
//                      &Enable, sizeof(Enable),
//                      NULL, NULL);
//     HeadlessDispatch(HeadlessCmdSendBlueScreenData,
//                      &BugCheckCode, sizeof(BugCheckCode),
//                      NULL, NULL);
//
//     /* Check if bootvid is installed */
//     if (InbvIsBootDriverInstalled())
//     {
//         /* Acquire ownership and reset the display */
//         InbvAcquireDisplayOwnership();
//         InbvResetDisplay();
//
//         /* Display blue screen */
//         InbvSolidColorFill(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, BV_COLOR_BLUE);
//         InbvSetTextColor(BV_COLOR_WHITE);
//         InbvInstallDisplayStringFilter(NULL);
//         InbvEnableDisplayString(TRUE);
//         InbvSetScrollRegion(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1);
//     }
//
//     /* Check if this is a hard error */
//     if (IsHardError)
//     {
//         /* Display caption and message */
//         if (HardErrCaption) InbvDisplayString(HardErrCaption);
//         if (HardErrMessage) InbvDisplayString(HardErrMessage);
//     }
//
//     /* Begin the display */
//     InbvDisplayString("\r\n");
//
//     /* Print out initial message */
//     KeGetBugMessageText(BUGCHECK_MESSAGE_INTRO, NULL);
//     InbvDisplayString("\r\n\r\n");
//
//     /* Check if we have a driver */
//     if (KiBugCheckDriver)
//     {
//         /* Print out into to driver name */
//         KeGetBugMessageText(BUGCODE_ID_DRIVER, NULL);
//
//         /* Convert and print out driver name */
//         KeBugCheckUnicodeToAnsi(KiBugCheckDriver, AnsiName, sizeof(AnsiName));
//         InbvDisplayString(" ");
//         InbvDisplayString(AnsiName);
//         InbvDisplayString("\r\n\r\n");
//     }
//
//     /* Check if this is the generic message */
//     if (MessageId == BUGCODE_PSS_MESSAGE)
//     {
//         /* It is, so get the bug code string as well */
//         KeGetBugMessageText(BugCheckCode, NULL);
//         InbvDisplayString("\r\n\r\n");
//     }
//
//     /* Print second introduction message */
//     KeGetBugMessageText(PSS_MESSAGE_INTRO, NULL);
//     InbvDisplayString("\r\n\r\n");
//
//     /* Get the bug code string */
//     KeGetBugMessageText(MessageId, NULL);
//     InbvDisplayString("\r\n\r\n");
//
//     /* Print message for technical information */
//     KeGetBugMessageText(BUGCHECK_TECH_INFO, NULL);
//
//     /* Show the technical Data */
//     RtlStringCbPrintfA(AnsiName,
//                        sizeof(AnsiName),
//                        "\r\n\r\n*** STOP: 0x%08lX (0x%p,0x%p,0x%p,0x%p)\r\n\r\n",
//                        BugCheckCode,
//                        (PVOID)KiBugCheckData[1],
//                        (PVOID)KiBugCheckData[2],
//                        (PVOID)KiBugCheckData[3],
//                        (PVOID)KiBugCheckData[4]);
//     InbvDisplayString(AnsiName);
//
//     /* Check if we have a driver*/
//     if (KiBugCheckDriver)
//     {
//         /* Display technical driver data */
//         InbvDisplayString(Message);
//     }
//     else
//     {
//         /* Dump parameter information */
//         KiDumpParameterImages(Message,
//                               (PVOID)&KiBugCheckData[1],
//                               4,
//                               KeBugCheckUnicodeToAnsi);
//     }
// }
//
// DECLSPEC_NORETURN
// VOID
// NTAPI
// KeBugCheckWithTf(IN ULONG BugCheckCode,
//                  IN ULONG_PTR BugCheckParameter1,
//                  IN ULONG_PTR BugCheckParameter2,
//                  IN ULONG_PTR BugCheckParameter3,
//                  IN ULONG_PTR BugCheckParameter4,
//                  IN PKTRAP_FRAME TrapFrame)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     CONTEXT Context;
//     ULONG MessageId;
//     CHAR AnsiName[128];
//     BOOLEAN IsSystem, IsHardError = FALSE, Reboot = FALSE;
//     PCHAR HardErrCaption = NULL, HardErrMessage = NULL;
//     PVOID Pc = NULL, Memory;
//     PVOID DriverBase;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PULONG_PTR HardErrorParameters;
//     KIRQL OldIrql;
//
//     /* Set active bugcheck */
//     KeBugCheckActive = TRUE;
//     KiBugCheckDriver = NULL;
//
//     /* Check if this is power failure simulation */
//     if (BugCheckCode == POWER_FAILURE_SIMULATE)
//     {
//         /* Call the Callbacks and reboot */
//         KiDoBugCheckCallbacks();
//         HalReturnToFirmware(HalRebootRoutine);
//     }
//
//     /* Save the IRQL and set hardware trigger */
//     Prcb->DebuggerSavedIRQL = KeGetCurrentIrql();
//     InterlockedIncrement((PLONG)&KiHardwareTrigger);
//
//     /* Capture the CPU Context */
//     RtlCaptureContext(&Prcb->ProcessorState.ContextFrame);
//     KiSaveProcessorControlState(&Prcb->ProcessorState);
//     Context = Prcb->ProcessorState.ContextFrame;
//
//     /* FIXME: Call the Watchdog if it's registered */
//
//     /* Check which bugcode this is */
//     switch (BugCheckCode)
//     {
//         /* These bug checks already have detailed messages, keep them */
//         case UNEXPECTED_KERNEL_MODE_TRAP:
//         case DRIVER_CORRUPTED_EXPOOL:
//         case ACPI_BIOS_ERROR:
//         case ACPI_BIOS_FATAL_ERROR:
//         case THREAD_STUCK_IN_DEVICE_DRIVER:
//         case DATA_BUS_ERROR:
//         case FAT_FILE_SYSTEM:
//         case NO_MORE_SYSTEM_PTES:
//         case INACCESSIBLE_BOOT_DEVICE:
//
//             /* Keep the same code */
//             MessageId = BugCheckCode;
//             break;
//
//         /* Check if this is a kernel-mode exception */
//         case KERNEL_MODE_EXCEPTION_NOT_HANDLED:
//         case SYSTEM_THREAD_EXCEPTION_NOT_HANDLED:
//         case KMODE_EXCEPTION_NOT_HANDLED:
//
//             /* Use the generic text message */
//             MessageId = KMODE_EXCEPTION_NOT_HANDLED;
//             break;
//
//         /* File-system errors */
//         case NTFS_FILE_SYSTEM:
//
//             /* Use the generic message for FAT */
//             MessageId = FAT_FILE_SYSTEM;
//             break;
//
//         /* Check if this is a coruption of the Mm's Pool */
//         case DRIVER_CORRUPTED_MMPOOL:
//
//             /* Use generic corruption message */
//             MessageId = DRIVER_CORRUPTED_EXPOOL;
//             break;
//
//         /* Check if this is a signature check failure */
//         case STATUS_SYSTEM_IMAGE_BAD_SIGNATURE:
//
//             /* Use the generic corruption message */
//             MessageId = BUGCODE_PSS_MESSAGE_SIGNATURE;
//             break;
//
//         /* All other codes */
//         default:
//
//             /* Use the default bugcheck message */
//             MessageId = BUGCODE_PSS_MESSAGE;
//             break;
//     }
//
//     /* Save bugcheck data */
//     KiBugCheckData[0] = BugCheckCode;
//     KiBugCheckData[1] = BugCheckParameter1;
//     KiBugCheckData[2] = BugCheckParameter2;
//     KiBugCheckData[3] = BugCheckParameter3;
//     KiBugCheckData[4] = BugCheckParameter4;
//
//     /* Now check what bugcheck this is */
//     switch (BugCheckCode)
//     {
//         /* Invalid access to R/O memory or Unhandled KM Exception */
//         case KERNEL_MODE_EXCEPTION_NOT_HANDLED:
//         case ATTEMPTED_WRITE_TO_READONLY_MEMORY:
//         case ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY:
//         {
//             /* Check if we have a trap frame */
//             if (!TrapFrame)
//             {
//                 /* Use parameter 3 as a trap frame, if it exists */
//                 if (BugCheckParameter3) TrapFrame = (PVOID)BugCheckParameter3;
//             }
//
//             /* Check if we got one now and if we need to get the Program Counter */
//             if ((TrapFrame) &&
//                 (BugCheckCode != KERNEL_MODE_EXCEPTION_NOT_HANDLED))
//             {
//                 /* Get the Program Counter */
//                 Pc = (PVOID)KeGetTrapFramePc(TrapFrame);
//             }
//             break;
//         }
//
//         /* Wrong IRQL */
//         case IRQL_NOT_LESS_OR_EQUAL:
//         {
//             /*
//              * The NT kernel has 3 special sections:
//              * MISYSPTE, POOLMI and POOLCODE. The bug check code can
//              * determine in which of these sections this bugcode happened
//              * and provide a more detailed analysis. For now, we don't.
//              */
//
//             /* Program Counter is in parameter 4 */
//             Pc = (PVOID)BugCheckParameter4;
//
//             /* Get the driver base */
//             DriverBase = KiPcToFileHeader(Pc,
//                                           &LdrEntry,
//                                           FALSE,
//                                           &IsSystem);
//             if (IsSystem)
//             {
//                 /*
//                  * The error happened inside the kernel or HAL.
//                  * Get the memory address that was being referenced.
//                  */
//                 Memory = (PVOID)BugCheckParameter1;
//
//                 /* Find to which driver it belongs */
//                 DriverBase = KiPcToFileHeader(Memory,
//                                               &LdrEntry,
//                                               TRUE,
//                                               &IsSystem);
//                 if (DriverBase)
//                 {
//                     /* Get the driver name and update the bug code */
//                     KiBugCheckDriver = &LdrEntry->BaseDllName;
//                     KiBugCheckData[0] = DRIVER_PORTION_MUST_BE_NONPAGED;
//                 }
//                 else
//                 {
//                     /* Find the driver that unloaded at this address */
//                     KiBugCheckDriver = NULL; // FIXME: ROS can't locate
//
//                     /* Check if the cause was an unloaded driver */
//                     if (KiBugCheckDriver)
//                     {
//                         /* Update bug check code */
//                         KiBugCheckData[0] =
//                             SYSTEM_SCAN_AT_RAISED_IRQL_CAUGHT_IMPROPER_DRIVER_UNLOAD;
//                     }
//                 }
//             }
//             else
//             {
//                 /* Update the bug check code */
//                 KiBugCheckData[0] = DRIVER_IRQL_NOT_LESS_OR_EQUAL;
//             }
//
//             /* Clear Pc so we don't look it up later */
//             Pc = NULL;
//             break;
//         }
//
//         /* Hard error */
//         case FATAL_UNHANDLED_HARD_ERROR:
//         {
//             /* Copy bug check data from hard error */
//             HardErrorParameters = (PULONG_PTR)BugCheckParameter2;
//             KiBugCheckData[0] = BugCheckParameter1;
//             KiBugCheckData[1] = HardErrorParameters[0];
//             KiBugCheckData[2] = HardErrorParameters[1];
//             KiBugCheckData[3] = HardErrorParameters[2];
//             KiBugCheckData[4] = HardErrorParameters[3];
//
//             /* Remember that this is hard error and set the caption/message */
//             IsHardError = TRUE;
//             HardErrCaption = (PCHAR)BugCheckParameter3;
//             HardErrMessage = (PCHAR)BugCheckParameter4;
//             break;
//         }
//
//         /* Page fault */
//         case PAGE_FAULT_IN_NONPAGED_AREA:
//         {
//             /* Assume no driver */
//             DriverBase = NULL;
//
//             /* Check if we have a trap frame */
//             if (!TrapFrame)
//             {
//                 /* We don't, use parameter 3 if possible */
//                 if (BugCheckParameter3) TrapFrame = (PVOID)BugCheckParameter3;
//             }
//
//             /* Check if we have a frame now */
//             if (TrapFrame)
//             {
//                 /* Get the Program Counter */
//                 Pc = (PVOID)KeGetTrapFramePc(TrapFrame);
//                 KiBugCheckData[3] = (ULONG_PTR)Pc;
//
//                 /* Find out if was in the kernel or drivers */
//                 DriverBase = KiPcToFileHeader(Pc,
//                                               &LdrEntry,
//                                               FALSE,
//                                               &IsSystem);
//             }
//             else
//             {
//                 /* Can't blame a driver, assume system */
//                 IsSystem = TRUE;
//             }
//
//             /* FIXME: Check for session pool in addition to special pool */
//
//             /* Special pool has its own bug check codes */
//             if (MmIsSpecialPoolAddress((PVOID)BugCheckParameter1))
//             {
//                 if (MmIsSpecialPoolAddressFree((PVOID)BugCheckParameter1))
//                 {
//                     KiBugCheckData[0] = IsSystem
//                         ? PAGE_FAULT_IN_FREED_SPECIAL_POOL
//                         : DRIVER_PAGE_FAULT_IN_FREED_SPECIAL_POOL;
//                 }
//                 else
//                 {
//                     KiBugCheckData[0] = IsSystem
//                         ? PAGE_FAULT_BEYOND_END_OF_ALLOCATION
//                         : DRIVER_PAGE_FAULT_BEYOND_END_OF_ALLOCATION;
//                 }
//             }
//             else if (!DriverBase)
//             {
//                 /* Find the driver that unloaded at this address */
//                 KiBugCheckDriver = NULL; // FIXME: ROS can't locate
//
//                 /* Check if the cause was an unloaded driver */
//                 if (KiBugCheckDriver)
//                 {
//                     KiBugCheckData[0] =
//                         DRIVER_UNLOADED_WITHOUT_CANCELLING_PENDING_OPERATIONS;
//                 }
//             }
//             break;
//         }
//
//         /* Check if the driver forgot to unlock pages */
//         case DRIVER_LEFT_LOCKED_PAGES_IN_PROCESS:
//
//             /* Program Counter is in parameter 1 */
//             Pc = (PVOID)BugCheckParameter1;
//             break;
//
//         /* Check if the driver consumed too many PTEs */
//         case DRIVER_USED_EXCESSIVE_PTES:
//
//             /* Loader entry is in parameter 1 */
//             LdrEntry = (PVOID)BugCheckParameter1;
//             KiBugCheckDriver = &LdrEntry->BaseDllName;
//             break;
//
//         /* Check if the driver has a stuck thread */
//         case THREAD_STUCK_IN_DEVICE_DRIVER:
//
//             /* The name is in Parameter 3 */
//             KiBugCheckDriver = (PVOID)BugCheckParameter3;
//             break;
//
//         /* Anything else */
//         default:
//             break;
//     }
//
//     /* Do we have a driver name? */
//     if (KiBugCheckDriver)
//     {
//         /* Convert it to ANSI */
//         KeBugCheckUnicodeToAnsi(KiBugCheckDriver, AnsiName, sizeof(AnsiName));
//     }
//     else
//     {
//         /* Do we have a Program Counter? */
//         if (Pc)
//         {
//             /* Dump image name */
//             KiDumpParameterImages(AnsiName,
//                                   (PULONG_PTR)&Pc,
//                                   1,
//                                   KeBugCheckUnicodeToAnsi);
//         }
//     }
//
//     /* Check if we need to save the context for KD */
//     if (!KdPitchDebugger) KdDebuggerDataBlock.SavedContext = (ULONG_PTR)&Context;
//
//     /* Check if a debugger is connected */
//     if ((BugCheckCode != MANUALLY_INITIATED_CRASH) && (KdDebuggerEnabled))
//     {
//         /* Crash on the debugger console */
//         DbgPrint("\n*** Fatal System Error: 0x%08lx\n"
//                  "                       (0x%p,0x%p,0x%p,0x%p)\n\n",
//                  KiBugCheckData[0],
//                  KiBugCheckData[1],
//                  KiBugCheckData[2],
//                  KiBugCheckData[3],
//                  KiBugCheckData[4]);
//
//         /* Check if the debugger isn't currently connected */
//         if (!KdDebuggerNotPresent)
//         {
//             /* Check if we have a driver to blame */
//             if (KiBugCheckDriver)
//             {
//                 /* Dump it */
//                 DbgPrint("Driver at fault: %s.\n", AnsiName);
//             }
//
//             /* Check if this was a hard error */
//             if (IsHardError)
//             {
//                 /* Print caption and message */
//                 if (HardErrCaption) DbgPrint(HardErrCaption);
//                 if (HardErrMessage) DbgPrint(HardErrMessage);
//             }
//
//             /* Break in the debugger */
//             KiBugCheckDebugBreak(DBG_STATUS_BUGCHECK_FIRST);
//         }
//     }
//
//     /* Raise IRQL to HIGH_LEVEL */
//     _disable();
//     KeRaiseIrql(HIGH_LEVEL, &OldIrql);
//
//     /* Avoid recursion */
//     if (!InterlockedDecrement((PLONG)&KeBugCheckCount))
//     {
// #ifdef CONFIG_SMP
//         /* Set CPU that is bug checking now */
//         KeBugCheckOwner = Prcb->Number;
//
//         /* Freeze the other CPUs */
//         KxFreezeExecution();
// #endif
//
//         /* Display the BSOD */
//         KiDisplayBlueScreen(MessageId,
//                             IsHardError,
//                             HardErrCaption,
//                             HardErrMessage,
//                             AnsiName);
//
//         // TODO/FIXME: Run the registered reason-callbacks from
//         // the KeBugcheckReasonCallbackListHead list with the
//         // KbCallbackReserved1 reason.
//
//         /* Check if the debugger is disabled but we can enable it */
//         if (!(KdDebuggerEnabled) && !(KdPitchDebugger))
//         {
//             /* Enable it */
//             KdEnableDebuggerWithLock(FALSE);
//         }
//         else
//         {
//             /* Otherwise, print the last line */
//             InbvDisplayString("\r\n");
//         }
//
//         /* Save the context */
//         Prcb->ProcessorState.ContextFrame = Context;
//
//         /* FIXME: Support Triage Dump */
//
//         /* FIXME: Write the crash dump */
//         // TODO: The crash-dump helper must set the Reboot variable.
//         Reboot = !!IopAutoReboot;
//     }
//     else
//     {
//         /* Increase recursion count */
//         KeBugCheckOwnerRecursionCount++;
//         if (KeBugCheckOwnerRecursionCount == 2)
//         {
//             /* Break in the debugger */
//             KiBugCheckDebugBreak(DBG_STATUS_BUGCHECK_SECOND);
//         }
//         else if (KeBugCheckOwnerRecursionCount > 2)
//         {
//             /* Halt execution */
//             while (TRUE);
//         }
//     }
//
//     /* Call the Callbacks */
//     KiDoBugCheckCallbacks();
//
//     /* FIXME: Call Watchdog if enabled */
//
//     /* Check if we have to reboot */
//     if (Reboot)
//     {
//         /* Unload symbols */
//         DbgUnLoadImageSymbols(NULL, (PVOID)MAXULONG_PTR, 0);
//         HalReturnToFirmware(HalRebootRoutine);
//     }
//
//     /* Attempt to break in the debugger (otherwise halt CPU) */
//     KiBugCheckDebugBreak(DBG_STATUS_BUGCHECK_SECOND);
//
//     /* Shouldn't get here */
//     ASSERT(FALSE);
//     while (TRUE);
// }
//
// BOOLEAN
// NTAPI
// KiHandleNmi(VOID)
// {
//     BOOLEAN Handled = FALSE;
//     PKNMI_HANDLER_CALLBACK NmiData;
//
//     /* Parse the list of callbacks */
//     NmiData = KiNmiCallbackListHead;
//     while (NmiData)
//     {
//         /* Save if this callback has handled it -- all it takes is one */
//         Handled |= NmiData->Callback(NmiData->Context, Handled);
//         NmiData = NmiData->Next;
//     }
//
//     /* Has anyone handled this? */
//     return Handled;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// KeInitializeCrashDumpHeader(IN ULONG Type,
//                             IN ULONG Flags,
//                             OUT PVOID Buffer,
//                             IN ULONG BufferSize,
//                             OUT ULONG BufferNeeded OPTIONAL)
// {
//     UNIMPLEMENTED;
//     return STATUS_UNSUCCESSFUL;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeDeregisterBugCheckCallback(IN PKBUGCHECK_CALLBACK_RECORD CallbackRecord)
// {
//     KIRQL OldIrql;
//     BOOLEAN Status = FALSE;
//
//     /* Raise IRQL to High */
//     KeRaiseIrql(HIGH_LEVEL, &OldIrql);
//
//     /* Check the Current State */
//     if (CallbackRecord->State == BufferInserted)
//     {
//         /* Reset state and remove from list */
//         CallbackRecord->State = BufferEmpty;
//         RemoveEntryList(&CallbackRecord->Entry);
//         Status = TRUE;
//     }
//
//     /* Lower IRQL and return */
//     KeLowerIrql(OldIrql);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeDeregisterBugCheckReasonCallback(
//     IN PKBUGCHECK_REASON_CALLBACK_RECORD CallbackRecord)
// {
//     KIRQL OldIrql;
//     BOOLEAN Status = FALSE;
//
//     /* Raise IRQL to High */
//     KeRaiseIrql(HIGH_LEVEL, &OldIrql);
//
//     /* Check the Current State */
//     if (CallbackRecord->State == BufferInserted)
//     {
//         /* Reset state and remove from list */
//         CallbackRecord->State = BufferEmpty;
//         RemoveEntryList(&CallbackRecord->Entry);
//         Status = TRUE;
//     }
//
//     /* Lower IRQL and return */
//     KeLowerIrql(OldIrql);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeRegisterBugCheckCallback(IN PKBUGCHECK_CALLBACK_RECORD CallbackRecord,
//                            IN PKBUGCHECK_CALLBACK_ROUTINE CallbackRoutine,
//                            IN PVOID Buffer,
//                            IN ULONG Length,
//                            IN PUCHAR Component)
// {
//     KIRQL OldIrql;
//     BOOLEAN Status = FALSE;
//
//     /* Raise IRQL to High */
//     KeRaiseIrql(HIGH_LEVEL, &OldIrql);
//
//     /* Check the Current State first so we don't double-register */
//     if (CallbackRecord->State == BufferEmpty)
//     {
//         /* Set the Callback Settings and insert into the list */
//         CallbackRecord->Length = Length;
//         CallbackRecord->Buffer = Buffer;
//         CallbackRecord->Component = Component;
//         CallbackRecord->CallbackRoutine = CallbackRoutine;
//         CallbackRecord->State = BufferInserted;
//         InsertTailList(&KeBugcheckCallbackListHead, &CallbackRecord->Entry);
//         Status = TRUE;
//     }
//
//     /* Lower IRQL and return */
//     KeLowerIrql(OldIrql);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeRegisterBugCheckReasonCallback(
//     IN PKBUGCHECK_REASON_CALLBACK_RECORD CallbackRecord,
//     IN PKBUGCHECK_REASON_CALLBACK_ROUTINE CallbackRoutine,
//     IN KBUGCHECK_CALLBACK_REASON Reason,
//     IN PUCHAR Component)
// {
//     KIRQL OldIrql;
//     BOOLEAN Status = FALSE;
//
//     /* Raise IRQL to High */
//     KeRaiseIrql(HIGH_LEVEL, &OldIrql);
//
//     /* Check the Current State first so we don't double-register */
//     if (CallbackRecord->State == BufferEmpty)
//     {
//         /* Set the Callback Settings and insert into the list */
//         CallbackRecord->Component = Component;
//         CallbackRecord->CallbackRoutine = CallbackRoutine;
//         CallbackRecord->State = BufferInserted;
//         CallbackRecord->Reason = Reason;
//         InsertTailList(&KeBugcheckReasonCallbackListHead,
//                        &CallbackRecord->Entry);
//         Status = TRUE;
//     }
//
//     /* Lower IRQL and return */
//     KeLowerIrql(OldIrql);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// PVOID
// NTAPI
// KeRegisterNmiCallback(IN PNMI_CALLBACK CallbackRoutine,
//                       IN PVOID Context)
// {
//     KIRQL OldIrql;
//     PKNMI_HANDLER_CALLBACK NmiData, Next;
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Allocate NMI callback data */
//     NmiData = ExAllocatePoolWithTag(NonPagedPool, sizeof(*NmiData), TAG_KNMI);
//     if (!NmiData) return NULL;
//
//     /* Fill in the information */
//     NmiData->Callback = CallbackRoutine;
//     NmiData->Context = Context;
//     NmiData->Handle = NmiData;
//
//     /* Insert it into NMI callback list */
//     KiAcquireNmiListLock(&OldIrql);
//     NmiData->Next = KiNmiCallbackListHead;
//     Next = InterlockedCompareExchangePointer((PVOID*)&KiNmiCallbackListHead,
//                                              NmiData,
//                                              NmiData->Next);
//     ASSERT(Next == NmiData->Next);
//     KiReleaseNmiListLock(OldIrql);
//
//     /* Return the opaque "handle" */
//     return NmiData->Handle;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// KeDeregisterNmiCallback(IN PVOID Handle)
// {
//     KIRQL OldIrql;
//     PKNMI_HANDLER_CALLBACK NmiData;
//     PKNMI_HANDLER_CALLBACK* Previous;
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Find in the list the NMI callback corresponding to the handle */
//     KiAcquireNmiListLock(&OldIrql);
//     Previous = &KiNmiCallbackListHead;
//     NmiData = *Previous;
//     while (NmiData)
//     {
//         if (NmiData->Handle == Handle)
//         {
//             /* The handle is the pointer to the callback itself */
//             ASSERT(Handle == NmiData);
//
//             /* Found it, remove from the list */
//             *Previous = NmiData->Next;
//             break;
//         }
//
//         /* Not found; try again */
//         Previous = &NmiData->Next;
//         NmiData = *Previous;
//     }
//     KiReleaseNmiListLock(OldIrql);
//
//     /* If we have found the entry, free it */
//     if (NmiData)
//     {
//         ExFreePoolWithTag(NmiData, TAG_KNMI);
//         return STATUS_SUCCESS;
//     }
//
//     return STATUS_INVALID_HANDLE;
// }
//
// /*
//  * @implemented
//  */
// DECLSPEC_NORETURN
// VOID
// NTAPI
// KeBugCheckEx(IN ULONG BugCheckCode,
//              IN ULONG_PTR BugCheckParameter1,
//              IN ULONG_PTR BugCheckParameter2,
//              IN ULONG_PTR BugCheckParameter3,
//              IN ULONG_PTR BugCheckParameter4)
// {
//     DbgPrint("\n*** BUGCHECK: 0x%08lX (0x%p, 0x%p, 0x%p, 0x%p)\n",
//              BugCheckCode,
//              (PVOID)BugCheckParameter1,
//              (PVOID)BugCheckParameter2,
//              (PVOID)BugCheckParameter3,
//              (PVOID)BugCheckParameter4);
// #ifdef _M_ARM64
//     /* UART trace for bugcheck on ARM64 */
//     {
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "\r\n*** BUGCHECK: code=0x";
//         int _i;
//         ULONG_PTR _lr;
//         __asm__ __volatile__("mov %0, x30" : "=r" (_lr));
//         while (*_s) *_u = (ULONG)*_s++;
//         for (_i = 28; _i >= 0; _i -= 4) {
//             int _d = (BugCheckCode >> _i) & 0xF;
//             *_u = (ULONG)(_d < 10 ? '0' + _d : 'A' + _d - 10);
//         }
//         _s = " LR=0x";
//         while (*_s) *_u = (ULONG)*_s++;
//         for (_i = 60; _i >= 0; _i -= 4) {
//             int _d = (_lr >> _i) & 0xF;
//             *_u = (ULONG)(_d < 10 ? '0' + _d : 'A' + _d - 10);
//         }
//         _s = " P1=0x";
//         while (*_s) *_u = (ULONG)*_s++;
//         for (_i = 60; _i >= 0; _i -= 4) {
//             int _d = (BugCheckParameter1 >> _i) & 0xF;
//             *_u = (ULONG)(_d < 10 ? '0' + _d : 'A' + _d - 10);
//         }
//         _s = " P2=0x";
//         while (*_s) *_u = (ULONG)*_s++;
//         for (_i = 60; _i >= 0; _i -= 4) {
//             int _d = (BugCheckParameter2 >> _i) & 0xF;
//             *_u = (ULONG)(_d < 10 ? '0' + _d : 'A' + _d - 10);
//         }
//         _s = " P3=0x";
//         while (*_s) *_u = (ULONG)*_s++;
//         for (_i = 60; _i >= 0; _i -= 4) {
//             int _d = (BugCheckParameter3 >> _i) & 0xF;
//             *_u = (ULONG)(_d < 10 ? '0' + _d : 'A' + _d - 10);
//         }
//         _s = " P4=0x";
//         while (*_s) *_u = (ULONG)*_s++;
//         for (_i = 60; _i >= 0; _i -= 4) {
//             int _d = (BugCheckParameter4 >> _i) & 0xF;
//             *_u = (ULONG)(_d < 10 ? '0' + _d : 'A' + _d - 10);
//         }
//         _s = "\r\n";
//         while (*_s) *_u = (ULONG)*_s++;
//     }
// #endif
//     /* Call the internal API */
//     KeBugCheckWithTf(BugCheckCode,
//                      BugCheckParameter1,
//                      BugCheckParameter2,
//                      BugCheckParameter3,
//                      BugCheckParameter4,
//                      NULL);
// }
//
// /*
//  * @implemented
//  */
// DECLSPEC_NORETURN
// VOID
// NTAPI
// KeBugCheck(ULONG BugCheckCode)
// {
//     DbgPrint("\n*** BUGCHECK: 0x%08lX\n", BugCheckCode);
// #ifdef _M_ARM64
//     {
//         volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//         const char *_s = "\r\n*** BUGCHECK: code=0x";
//         int _i;
//         ULONG_PTR _lr;
//         __asm__ __volatile__("mov %0, x30" : "=r" (_lr));
//         while (*_s) *_u = (ULONG)*_s++;
//         for (_i = 28; _i >= 0; _i -= 4) {
//             int _d = (BugCheckCode >> _i) & 0xF;
//             *_u = (ULONG)(_d < 10 ? '0' + _d : 'A' + _d - 10);
//         }
//         _s = " LR=0x";
//         while (*_s) *_u = (ULONG)*_s++;
//         for (_i = 60; _i >= 0; _i -= 4) {
//             int _d = (_lr >> _i) & 0xF;
//             *_u = (ULONG)(_d < 10 ? '0' + _d : 'A' + _d - 10);
//         }
//         _s = "\r\n";
//         while (*_s) *_u = (ULONG)*_s++;
//     }
// #endif
//     /* Call the internal API */
//     KeBugCheckWithTf(BugCheckCode, 0, 0, 0, 0, NULL);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeEnterKernelDebugger(VOID)
// {
//     /* Disable interrupts */
//     KiHardwareTrigger = 1;
//     _disable();
//
//     /* Check the bugcheck count */
//     if (!InterlockedDecrement((PLONG)&KeBugCheckCount))
//     {
//         /* There was only one, is the debugger disabled? */
//         if (!(KdDebuggerEnabled) && !(KdPitchDebugger))
//         {
//             /* Enable the debugger */
//             KdInitSystem(0, NULL);
//         }
//     }
//
//     /* Break in the debugger */
//     KiBugCheckDebugBreak(DBG_STATUS_FATAL);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
