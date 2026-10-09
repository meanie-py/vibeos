use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{


    _LUID,
    _UNICODE_STRING,
    KPROCESSOR_MODE,
    PCSTR,
    PUNICODE_STRING,
    PLOADER_PARAMETER_BLOCK,
    PLDR_DATA_TABLE_ENTRY,
    LDR_DATA_TABLE_ENTRY,
    PIMAGE_RESOURCE_DATA_ENTRY,
    INBV_DISPLAY_STRING_FILTER,
    INBV_RESET_DISPLAY_PARAMETERS,
    PLIST_ENTRY,
    OEM_STRING,
    PANSI_STRING,
    KeInitializeSpinLock,
    KeGetCurrentIrql,
    KeTestSpinLock,
    KiAcquireSpinLock,
    KiReleaseSpinLock,
    RtlEqualUnicodeString,
    LdrFindResource_U,
    LdrAccessResource,
    RtlComputeCrc32,
    ExGetPreviousMode,
    RtlInitEmptyAnsiString,
    RtlUnicodeStringToOemString,
};

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


type LUID = _LUID;

extern "system" {
    fn VidInitialize(ResetMode: BOOLEAN) -> BOOLEAN;
    fn VidCleanUp();
    fn VidResetDisplay(HalReset: BOOLEAN);
    fn VidSetScrollRegion(Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG);
    fn VidSetTextColor(Color: ULONG);
    fn VidDisplayString(String: PCSTR);
    fn VidSolidColorFill(Left: ULONG, Top: ULONG, Right: ULONG, Bottom: ULONG, Color: UCHAR);
    fn VidBitBlt(Buffer: PUCHAR, X: ULONG, Y: ULONG);
    fn VidBufferToScreenBlt(Buffer: PUCHAR, X: ULONG, Y: ULONG, Width: ULONG, Height: ULONG, Delta: ULONG);
    fn VidScreenToBufferBlt(Buffer: PUCHAR, X: ULONG, Y: ULONG, Width: ULONG, Height: ULONG, Delta: ULONG);
    fn BootAnimTickProgressBar(TotalProgress: ULONG);
    fn HeadlessDispatch(Command: ULONG, InputBuffer: PVOID, InputBufferSize: usize, OutputBuffer: PVOID, OutputBufferSize: PVOID) -> BOOLEAN;
    fn SeSinglePrivilegeCheck(PrivilegeValue: LUID, PreviousMode: KPROCESSOR_MODE) -> BOOLEAN;

    static mut KiBugCheckData: [ULONG_PTR; 5];
    static SeTcbPrivilege: LUID;
}
extern "C" {
    fn ProbeAndCaptureUnicodeString(CapturedString: PUNICODE_STRING, PreviousMode: KPROCESSOR_MODE, DisplayString: PUNICODE_STRING) -> NTSTATUS;
    fn ReleaseCapturedUnicodeString(CapturedString: PUNICODE_STRING, PreviousMode: KPROCESSOR_MODE);
}
unsafe extern "system" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
}
extern "system" {
    fn RtlUnicodeStringToOemSize(UnicodeString: PUNICODE_STRING) -> ULONG;
}

extern "C" {
    fn strlen(s: PCSTR) -> usize;
    fn _strupr(s: PCHAR) -> PCHAR;
    fn strstr(haystack: PCSTR, needle: PCSTR) -> PCSTR;
}

const ANSI_NULL: u8 = 0;
const TAG_OSTR: ULONG = 0x5254534F;
const MAXUSHORT: ULONG = 0xFFFF;

const LANG_NEUTRAL: USHORT = 0x00;
const SUBLANG_NEUTRAL: USHORT = 0x00;
const RESOURCE_DATA_LEVEL: ULONG = 0x3;

const IDB_MAX_RESOURCES: usize = 5;

const HeadlessCmdPutString: ULONG = 3; // header value (was 1, invented)
const HeadlessCmdClearDisplay: ULONG = 4; // header value (was 3, invented)
const HeadlessCmdSetColor: ULONG = 9; // header value (was 4, invented)

#[repr(C)]
struct INBV_PROGRESS_STATE {
    Floor: ULONG,
    Ceiling: ULONG,
    Bias: ULONG,
}

#[repr(C)]
struct BT_PROGRESS_INDICATOR {
    Count: ULONG,
    Expected: ULONG,
    Percentage: ULONG,
}

#[repr(C)]
struct LDR_RESOURCE_INFO {
    Type: ULONG_PTR,
    Name: ULONG_PTR,
    Language: ULONG,
}

#[repr(C)]
struct HEADLESS_CMD_SET_COLOR {
    TextColor: ULONG,
    BkgdColor: ULONG,
}

// From the header, not retyped: the translation had OWNED and DISABLED swapped (OWNED is 0),
// so InbvGetDisplayState disagreed with bootanim.rs, which had OWNED right.
const INBV_DISPLAY_STATE_DISABLED: ULONG =
    ntkrnl_sys::ddk::_INBV_DISPLAY_STATE_INBV_DISPLAY_STATE_DISABLED as ULONG;
const INBV_DISPLAY_STATE_OWNED: ULONG =
    ntkrnl_sys::ddk::_INBV_DISPLAY_STATE_INBV_DISPLAY_STATE_OWNED as ULONG;
const INBV_DISPLAY_STATE_LOST: ULONG =
    ntkrnl_sys::ddk::_INBV_DISPLAY_STATE_INBV_DISPLAY_STATE_LOST as ULONG;

type INBV_DISPLAY_STATE = ULONG;

#[no_mangle]
pub static mut BootDriverLock: KSPIN_LOCK = 0;
#[no_mangle]
pub static mut InbvOldIrql: KIRQL = 0;
#[no_mangle]
pub static mut InbvDisplayState: INBV_DISPLAY_STATE = INBV_DISPLAY_STATE_DISABLED;
#[no_mangle]
pub static mut InbvBootDriverInstalled: BOOLEAN = 0u8;
#[no_mangle]
pub static mut InbvResetDisplayParameters: INBV_RESET_DISPLAY_PARAMETERS = None;

#[no_mangle]
pub static mut InbvDisplayDebugStrings: BOOLEAN = 0u8;
#[no_mangle]
pub static mut InbvDisplayFilter: INBV_DISPLAY_STRING_FILTER = None;

#[no_mangle]
pub static mut ProgressBarLeft: ULONG = 0;
#[no_mangle]
pub static mut ProgressBarTop: ULONG = 0;
#[no_mangle]
pub static mut ShowProgressBar: BOOLEAN = 0u8;
#[no_mangle]
pub static mut InbvProgressState: INBV_PROGRESS_STATE = INBV_PROGRESS_STATE {
    Floor: 0,
    Ceiling: 0,
    Bias: 0,
};
#[no_mangle]
pub static mut InbvProgressIndicator: BT_PROGRESS_INDICATOR = BT_PROGRESS_INDICATOR {
    Count: 0,
    Expected: 25,
    Percentage: 0,
};

#[no_mangle]
pub static mut ResourceCount: ULONG = 0;
#[no_mangle]
pub static mut ResourceList: [PUCHAR; 1 + IDB_MAX_RESOURCES] = [core::ptr::null_mut(); 1 + IDB_MAX_RESOURCES];

#[no_mangle]
pub static mut InbvTerminalTextColor: ULONG = 37;
#[no_mangle]
pub static mut InbvTerminalBkgdColor: ULONG = 40;

fn MAKEINTRESOURCE(id: ULONG) -> ULONG_PTR {
    id as ULONG_PTR
}

fn MAKELANGID(primary: USHORT, sublang: USHORT) -> ULONG {
    ((sublang as ULONG) << 10) | (primary as ULONG)
}

fn RTL_NUMBER_OF<T>(arr: &[T]) -> usize {
    arr.len()
}

fn min(a: ULONG, b: ULONG) -> ULONG {
    if a < b { a } else { b }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn FindBitmapResource(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    ResourceId: ULONG,
) -> PVOID {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let mut UpString = _UNICODE_STRING {
        Length: 24,
        MaximumLength: 26,
        Buffer: w!("ntoskrnl.exe").as_ptr() as *mut u16,
    };
    let mut MpString = _UNICODE_STRING {
        Length: 26,
        MaximumLength: 28,
        Buffer: w!("ntkrnlmp.exe").as_ptr() as *mut u16,
    };
    let mut NextEntry: PLIST_ENTRY;
    let ListHead: PLIST_ENTRY;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
    let mut ResourceDataEntry: PIMAGE_RESOURCE_DATA_ENTRY = core::ptr::null_mut();
    let mut ResourceInfo: LDR_RESOURCE_INFO;
    let mut Status: NTSTATUS;
    let mut Data: PVOID = core::ptr::null_mut();

    ListHead = core::ptr::addr_of_mut!((*LoaderBlock).LoadOrderListHead) as PLIST_ENTRY;
    NextEntry = (*ListHead).Flink;

    while NextEntry != ListHead {
        LdrEntry = (NextEntry as usize - core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)) as PLDR_DATA_TABLE_ENTRY;

        if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
            &mut UpString,
            1u8,
        ) != 0 || RtlEqualUnicodeString(
            core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
            &mut MpString,
            1u8,
        ) != 0 {
            break;
        }

        NextEntry = (*NextEntry).Flink;
    }

    if NextEntry != ListHead {
        ResourceInfo = LDR_RESOURCE_INFO {
            Type: MAKEINTRESOURCE(2),
            Name: ResourceId as ULONG_PTR,
            Language: MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL),
        };

        Status = LdrFindResource_U(
            (*LdrEntry).DllBase,
            &mut ResourceInfo as *mut LDR_RESOURCE_INFO as *mut _,
            RESOURCE_DATA_LEVEL,
            &mut ResourceDataEntry,
        );
        if Status >= 0 {
            let mut Size: ULONG = 0;
            Status = LdrAccessResource(
                (*LdrEntry).DllBase,
                ResourceDataEntry,
                &mut Data,
                &mut Size,
            );
            if !Data.is_null() && ResourceId < 3 {
                KiBugCheckData[4] ^= RtlComputeCrc32(0, Data as PUCHAR, Size) as ULONG_PTR;
            }
            if Status < 0 {
                Data = core::ptr::null_mut();
            }
        }
    }

    Data
}

#[no_mangle]
pub unsafe extern "system" fn InbvGetResourceAddress(
    ResourceNumber: ULONG,
) -> PUCHAR {
    if ResourceNumber > ResourceCount {
        return core::ptr::null_mut();
    }

    ResourceList[ResourceNumber as usize]
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn InbvDriverInitialize(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    Count: ULONG,
) -> BOOLEAN {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let mut CommandLine: PCHAR;
    let mut ResetMode: BOOLEAN = 0u8;
    let mut i: ULONG;

    if InbvBootDriverInstalled != 0 {
        return 1u8;
    }

    KeInitializeSpinLock(core::ptr::addr_of_mut!(BootDriverLock) as *mut _);
    if InbvDisplayState == INBV_DISPLAY_STATE_OWNED {
        CommandLine = if !(*LoaderBlock).LoadOptions.is_null() {
            _strupr((*LoaderBlock).LoadOptions)
        } else {
            core::ptr::null_mut()
        };
        ResetMode = if CommandLine.is_null() || strstr(CommandLine as PCSTR, b"BOOTLOGO\0".as_ptr() as PCSTR).is_null() {
            1u8
        } else {
            0u8
        };
    }

    InbvBootDriverInstalled = VidInitialize(ResetMode);
    if InbvBootDriverInstalled != 0 {
        ResourceCount = min(Count, RTL_NUMBER_OF(&*(&raw const ResourceList)) as ULONG - 1);
        i = 1;
        while i <= ResourceCount {
            ResourceList[i as usize] = FindBitmapResource(LoaderBlock, i) as PUCHAR;
            i += 1;
        }

        InbvSetProgressBarSubset(0, 100);
    }

    InbvBootDriverInstalled
}

#[no_mangle]
pub unsafe extern "system" fn InbvAcquireLock() {
    let mut OldIrql: KIRQL;

    OldIrql = KeGetCurrentIrql();
    if OldIrql <= DISPATCH_LEVEL {
        while KeTestSpinLock(core::ptr::addr_of_mut!(BootDriverLock) as *mut _) == 0 {}

        KeRaiseIrql(DISPATCH_LEVEL, &mut OldIrql);
    }

    KiAcquireSpinLock(core::ptr::addr_of_mut!(BootDriverLock) as *mut _);
    InbvOldIrql = OldIrql;
}

#[no_mangle]
pub unsafe extern "system" fn InbvReleaseLock() {
    let OldIrql: KIRQL;

    OldIrql = InbvOldIrql;

    KiReleaseSpinLock(core::ptr::addr_of_mut!(BootDriverLock) as *mut _);

    if InbvOldIrql <= DISPATCH_LEVEL {
        KeLowerIrql(OldIrql);
    }
}

#[no_mangle]
pub unsafe extern "system" fn InbvEnableBootDriver(
    Enable: BOOLEAN,
) {
    if InbvBootDriverInstalled != 0 {
        if InbvDisplayState >= INBV_DISPLAY_STATE_LOST {
            return;
        }

        InbvAcquireLock();

        if InbvDisplayState == INBV_DISPLAY_STATE_OWNED {
            VidCleanUp();
        }

        InbvDisplayState = if Enable != 0 {
            INBV_DISPLAY_STATE_OWNED
        } else {
            INBV_DISPLAY_STATE_DISABLED
        };

        InbvReleaseLock();
    } else {
        InbvDisplayState = if Enable != 0 {
            INBV_DISPLAY_STATE_OWNED
        } else {
            INBV_DISPLAY_STATE_DISABLED
        };
    }
}

#[no_mangle]
pub unsafe extern "system" fn InbvAcquireDisplayOwnership() {
    if (*(&raw const InbvResetDisplayParameters)).is_some() &&
        InbvDisplayState == INBV_DISPLAY_STATE_LOST {
        (InbvResetDisplayParameters.unwrap())(80, 50);
    }

    InbvDisplayState = INBV_DISPLAY_STATE_OWNED;
}

#[no_mangle]
pub unsafe extern "system" fn InbvSetDisplayOwnership(
    DisplayOwned: BOOLEAN,
) {
    InbvDisplayState = if DisplayOwned != 0 {
        INBV_DISPLAY_STATE_OWNED
    } else {
        INBV_DISPLAY_STATE_LOST
    };
}

#[no_mangle]
pub unsafe extern "system" fn InbvCheckDisplayOwnership() -> BOOLEAN {
    if InbvDisplayState != INBV_DISPLAY_STATE_LOST { 1u8 } else { 0u8 }
}

#[no_mangle]
pub unsafe extern "system" fn InbvGetDisplayState() -> INBV_DISPLAY_STATE {
    InbvDisplayState
}

#[no_mangle]
pub unsafe extern "system" fn InbvDisplayString(
    String: PCSTR,
) -> BOOLEAN {
    if InbvDisplayState == INBV_DISPLAY_STATE_OWNED {
        if InbvDisplayDebugStrings == 0 {
            return 1u8;
        }

        if (*(&raw const InbvDisplayFilter)).is_some() {
            (InbvDisplayFilter.unwrap())(core::ptr::addr_of!(String) as *mut PCSTR);
        }

        InbvAcquireLock();

        if InbvBootDriverInstalled != 0 {
            VidDisplayString(String);
        }

        HeadlessDispatch(
            HeadlessCmdPutString,
            String as PVOID,
            strlen(String) + core::mem::size_of::<u8>(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        InbvReleaseLock();

        return 1u8;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn InbvEnableDisplayString(
    Enable: BOOLEAN,
) -> BOOLEAN {
    let OldSetting: BOOLEAN;

    OldSetting = InbvDisplayDebugStrings;

    InbvDisplayDebugStrings = Enable;

    OldSetting
}

#[no_mangle]
pub unsafe extern "system" fn InbvInstallDisplayStringFilter(
    DisplayFilter: INBV_DISPLAY_STRING_FILTER,
) {
    InbvDisplayFilter = DisplayFilter;
}

#[no_mangle]
pub unsafe extern "system" fn InbvIsBootDriverInstalled() -> BOOLEAN {
    InbvBootDriverInstalled
}

#[no_mangle]
pub unsafe extern "system" fn InbvNotifyDisplayOwnershipLost(
    Callback: INBV_RESET_DISPLAY_PARAMETERS,
) {
    if InbvBootDriverInstalled != 0 {
        InbvAcquireLock();
        if InbvDisplayState != INBV_DISPLAY_STATE_LOST {
            VidCleanUp();
        }

        InbvResetDisplayParameters = Callback;
        InbvDisplayState = INBV_DISPLAY_STATE_LOST;

        InbvReleaseLock();
    } else {
        InbvResetDisplayParameters = Callback;
        InbvDisplayState = INBV_DISPLAY_STATE_LOST;
    }
}

#[no_mangle]
pub unsafe extern "system" fn InbvResetDisplay() -> BOOLEAN {
    if InbvBootDriverInstalled != 0 &&
        InbvDisplayState == INBV_DISPLAY_STATE_OWNED {
        VidResetDisplay(1u8);
        return 1u8;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn InbvSetScrollRegion(
    Left: ULONG,
    Top: ULONG,
    Right: ULONG,
    Bottom: ULONG,
) {
    VidSetScrollRegion(Left, Top, Right, Bottom);
}

#[no_mangle]
pub unsafe extern "system" fn InbvSetTextColor(
    Color: ULONG,
) {
    let mut HeadlessSetColor: HEADLESS_CMD_SET_COLOR;

    InbvTerminalTextColor = 37;
    HeadlessSetColor = HEADLESS_CMD_SET_COLOR {
        TextColor: InbvTerminalTextColor,
        BkgdColor: InbvTerminalBkgdColor,
    };
    HeadlessDispatch(
        HeadlessCmdSetColor,
        core::ptr::addr_of_mut!(HeadlessSetColor) as PVOID,
        core::mem::size_of::<HEADLESS_CMD_SET_COLOR>(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    VidSetTextColor(Color);
}

#[no_mangle]
pub unsafe extern "system" fn InbvSolidColorFill(
    Left: ULONG,
    Top: ULONG,
    Right: ULONG,
    Bottom: ULONG,
    Color: ULONG,
) {
    let mut HeadlessSetColor: HEADLESS_CMD_SET_COLOR;

    if InbvDisplayState == INBV_DISPLAY_STATE_OWNED {
        InbvAcquireLock();

        if InbvBootDriverInstalled != 0 {
            VidSolidColorFill(Left, Top, Right, Bottom, Color as UCHAR);
        }

        InbvTerminalBkgdColor = 40;
        HeadlessSetColor = HEADLESS_CMD_SET_COLOR {
            TextColor: InbvTerminalTextColor,
            BkgdColor: InbvTerminalBkgdColor,
        };
        HeadlessDispatch(
            HeadlessCmdSetColor,
            core::ptr::addr_of_mut!(HeadlessSetColor) as PVOID,
            core::mem::size_of::<HEADLESS_CMD_SET_COLOR>(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        HeadlessDispatch(
            HeadlessCmdClearDisplay,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        InbvReleaseLock();
    }
}

#[no_mangle]
pub unsafe extern "system" fn InbvBitBlt(
    Buffer: PUCHAR,
    X: ULONG,
    Y: ULONG,
) {
    let Buffer = opaque_ptr(Buffer);
    if InbvBootDriverInstalled != 0 &&
        InbvDisplayState == INBV_DISPLAY_STATE_OWNED {
        InbvAcquireLock();

        VidBitBlt(Buffer, X, Y);

        InbvReleaseLock();
    }
}

#[no_mangle]
pub unsafe extern "system" fn InbvBufferToScreenBlt(
    Buffer: PUCHAR,
    X: ULONG,
    Y: ULONG,
    Width: ULONG,
    Height: ULONG,
    Delta: ULONG,
) {
    let Buffer = opaque_ptr(Buffer);
    if InbvBootDriverInstalled != 0 &&
        InbvDisplayState == INBV_DISPLAY_STATE_OWNED {
        VidBufferToScreenBlt(Buffer, X, Y, Width, Height, Delta);
    }
}

#[no_mangle]
pub unsafe extern "system" fn InbvScreenToBufferBlt(
    Buffer: PUCHAR,
    X: ULONG,
    Y: ULONG,
    Width: ULONG,
    Height: ULONG,
    Delta: ULONG,
) {
    let Buffer = opaque_ptr(Buffer);
    if InbvBootDriverInstalled != 0 &&
        InbvDisplayState == INBV_DISPLAY_STATE_OWNED {
        VidScreenToBufferBlt(Buffer, X, Y, Width, Height, Delta);
    }
}

#[no_mangle]
pub unsafe extern "system" fn InbvSetProgressBarCoordinates(
    Left: ULONG,
    Top: ULONG,
) {
    ProgressBarLeft = Left;
    ProgressBarTop = Top;

    ShowProgressBar = 1u8;
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn InbvIndicateProgress() {
    let mut Percentage: ULONG;

    InbvProgressIndicator.Count += 1;

    Percentage = 100 * InbvProgressIndicator.Count / InbvProgressIndicator.Expected;
    Percentage = min(Percentage, 99);

    if Percentage != InbvProgressIndicator.Percentage {
        InbvProgressIndicator.Percentage = Percentage;
        InbvUpdateProgressBar(Percentage);
    }
}

#[no_mangle]
pub unsafe extern "system" fn InbvSetProgressBarSubset(
    Floor: ULONG,
    Ceiling: ULONG,
) {
    InbvProgressState.Floor = Floor * 100;
    InbvProgressState.Ceiling = Ceiling * 100;
    InbvProgressState.Bias = Ceiling - Floor;
}

#[no_mangle]
pub unsafe extern "system" fn InbvUpdateProgressBar(
    Percentage: ULONG,
) {
    let mut TotalProgress: ULONG;

    if ShowProgressBar != 0 &&
        InbvBootDriverInstalled != 0 &&
        InbvDisplayState == INBV_DISPLAY_STATE_OWNED {
        TotalProgress = InbvProgressState.Floor + (Percentage * InbvProgressState.Bias);

        BootAnimTickProgressBar(TotalProgress);
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtDisplayString(
    DisplayString: PUNICODE_STRING,
) -> NTSTATUS {
    let DisplayString = opaque_ptr(DisplayString);
    let mut Status: NTSTATUS;
    let mut CapturedString: _UNICODE_STRING = core::mem::zeroed();
    let mut OemString: OEM_STRING = core::mem::zeroed();
    let mut OemLength: ULONG;
    let PreviousMode: KPROCESSOR_MODE;

    PreviousMode = ExGetPreviousMode();

    if SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode) == 0 {
        return STATUS_PRIVILEGE_NOT_HELD;
    }

    Status = ProbeAndCaptureUnicodeString(
        &mut CapturedString,
        PreviousMode,
        DisplayString,
    );
    if Status < 0 {
        return Status;
    }

    if CapturedString.Length == 0 || CapturedString.Buffer.is_null() {
        Status = STATUS_SUCCESS;
        ReleaseCapturedUnicodeString(&mut CapturedString, PreviousMode);
        return Status;
    }

    OemLength = RtlUnicodeStringToOemSize(&mut CapturedString);
    if OemLength > MAXUSHORT {
        Status = STATUS_BUFFER_OVERFLOW;
        ReleaseCapturedUnicodeString(&mut CapturedString, PreviousMode);
        return Status;
    }
    RtlInitEmptyAnsiString(
        core::ptr::addr_of_mut!(OemString) as PANSI_STRING,
        core::ptr::null_mut(),
        OemLength as USHORT,
    );
    OemString.Buffer = ExAllocatePoolWithTag(NonPagedPool, OemLength as SIZE_T, TAG_OSTR) as PCHAR;
    if OemString.Buffer.is_null() {
        Status = STATUS_NO_MEMORY;
        ReleaseCapturedUnicodeString(&mut CapturedString, PreviousMode);
        return Status;
    }
    Status = RtlUnicodeStringToOemString(&mut OemString, &mut CapturedString, 0u8);
    if Status < 0 {
        ExFreePoolWithTag(OemString.Buffer as PVOID, TAG_OSTR);
        ReleaseCapturedUnicodeString(&mut CapturedString, PreviousMode);
        return Status;
    }

    InbvDisplayString(OemString.Buffer as PCSTR);

    ExFreePoolWithTag(OemString.Buffer as PVOID, TAG_OSTR);

    Status = STATUS_SUCCESS;

    ReleaseCapturedUnicodeString(&mut CapturedString, PreviousMode);

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     BSD - See COPYING.ARM in the top level directory
//  * PURPOSE:     Boot Video Driver support
//  * COPYRIGHT:   Copyright 2007 Alex Ionescu (alex.ionescu@reactos.org)
//  *              Copyright 2010 Aleksey Bragin (aleksey@reactos.org)
//  *              Copyright 2015-2022 Hermès Bélusca-Maïto
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include "inbv/logo.h"
//
// /* GLOBALS *******************************************************************/
//
// /*
//  * Enable this define if you want Inbv to use coloured headless mode.
//  */
// // #define INBV_HEADLESS_COLORS
//
// typedef struct _INBV_PROGRESS_STATE
// {
//     ULONG Floor;
//     ULONG Ceiling;
//     ULONG Bias;
// } INBV_PROGRESS_STATE;
//
// typedef struct _BT_PROGRESS_INDICATOR
// {
//     ULONG Count;
//     ULONG Expected;
//     ULONG Percentage;
// } BT_PROGRESS_INDICATOR, *PBT_PROGRESS_INDICATOR;
//
// static KSPIN_LOCK BootDriverLock;
// static KIRQL InbvOldIrql;
// static INBV_DISPLAY_STATE InbvDisplayState = INBV_DISPLAY_STATE_DISABLED;
// BOOLEAN InbvBootDriverInstalled = FALSE;
// static INBV_RESET_DISPLAY_PARAMETERS InbvResetDisplayParameters = NULL;
//
// static BOOLEAN InbvDisplayDebugStrings = FALSE;
// static INBV_DISPLAY_STRING_FILTER InbvDisplayFilter = NULL;
//
// ULONG ProgressBarLeft = 0, ProgressBarTop = 0;
// BOOLEAN ShowProgressBar = FALSE;
// static INBV_PROGRESS_STATE InbvProgressState;
// static BT_PROGRESS_INDICATOR InbvProgressIndicator = {0, 25, 0};
//
// static ULONG ResourceCount = 0;
// static PUCHAR ResourceList[1 + IDB_MAX_RESOURCES]; // First entry == NULL, followed by 'ResourceCount' entries.
//
//
// /*
//  * Headless terminal text colors
//  */
//
// #ifdef INBV_HEADLESS_COLORS
//
// // Conversion table CGA to ANSI color index
// static const UCHAR CGA_TO_ANSI_COLOR_TABLE[16] =
// {
//     0,  // Black
//     4,  // Blue
//     2,  // Green
//     6,  // Cyan
//     1,  // Red
//     5,  // Magenta
//     3,  // Brown/Yellow
//     7,  // Grey/White
//
//     60, // Bright Black
//     64, // Bright Blue
//     62, // Bright Green
//     66, // Bright Cyan
//     61, // Bright Red
//     65, // Bright Magenta
//     63, // Bright Yellow
//     67  // Bright Grey (White)
// };
//
// #define CGA_TO_ANSI_COLOR(CgaColor) \
//     CGA_TO_ANSI_COLOR_TABLE[CgaColor & 0x0F]
//
// #endif
//
// // Default colors: text in white, background in black
// static ULONG InbvTerminalTextColor = 37;
// static ULONG InbvTerminalBkgdColor = 40;
//
//
// /* FUNCTIONS *****************************************************************/
//
// #define RT_BITMAP   MAKEINTRESOURCE(2)
//
// CODE_SEG("INIT")
// static
// PVOID
// FindBitmapResource(
//     _In_ PLOADER_PARAMETER_BLOCK LoaderBlock,
//     _In_ ULONG ResourceId)
// {
//     UNICODE_STRING UpString = RTL_CONSTANT_STRING(L"ntoskrnl.exe");
//     UNICODE_STRING MpString = RTL_CONSTANT_STRING(L"ntkrnlmp.exe");
//     PLIST_ENTRY NextEntry, ListHead;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PIMAGE_RESOURCE_DATA_ENTRY ResourceDataEntry;
//     LDR_RESOURCE_INFO ResourceInfo;
//     NTSTATUS Status;
//     PVOID Data = NULL;
//
//     /* Loop the driver list */
//     ListHead = &LoaderBlock->LoadOrderListHead;
//     for (NextEntry = ListHead->Flink;
//          NextEntry != ListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         /* Get the entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry,
//                                      LDR_DATA_TABLE_ENTRY,
//                                      InLoadOrderLinks);
//
//         /* Check for a match */
//         if (RtlEqualUnicodeString(&LdrEntry->BaseDllName, &UpString, TRUE) ||
//             RtlEqualUnicodeString(&LdrEntry->BaseDllName, &MpString, TRUE))
//         {
//             /* Break out */
//             break;
//         }
//     }
//
//     /* Check if we found it */
//     if (NextEntry != ListHead)
//     {
//         /* Try to find the resource */
//         ResourceInfo.Type = RT_BITMAP;
//         ResourceInfo.Name = ResourceId;
//         ResourceInfo.Language = MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL);
//
//         Status = LdrFindResource_U(LdrEntry->DllBase,
//                                    &ResourceInfo,
//                                    RESOURCE_DATA_LEVEL,
//                                    &ResourceDataEntry);
//         if (NT_SUCCESS(Status))
//         {
//             /* Access the resource */
//             ULONG Size = 0;
//             Status = LdrAccessResource(LdrEntry->DllBase,
//                                        ResourceDataEntry,
//                                        &Data,
//                                        &Size);
//             if ((Data) && (ResourceId < 3))
//             {
//                 KiBugCheckData[4] ^= RtlComputeCrc32(0, Data, Size);
//             }
//             if (!NT_SUCCESS(Status)) Data = NULL;
//         }
//     }
//
//     /* Return the pointer */
//     return Data;
// }
//
// PUCHAR
// NTAPI
// InbvGetResourceAddress(
//     _In_ ULONG ResourceNumber)
// {
//     /* Validate the resource number */
//     if (ResourceNumber > ResourceCount) return NULL;
//
//     /* Return the address */
//     return ResourceList[ResourceNumber];
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// InbvDriverInitialize(
//     _In_ PLOADER_PARAMETER_BLOCK LoaderBlock,
//     _In_ ULONG Count)
// {
//     PCHAR CommandLine;
//     BOOLEAN ResetMode = FALSE; // By default do not reset the video mode
//     ULONG i;
//
//     /* Quit if we're already installed */
//     if (InbvBootDriverInstalled) return TRUE;
//
//     /* Initialize the lock and check the current display state */
//     KeInitializeSpinLock(&BootDriverLock);
//     if (InbvDisplayState == INBV_DISPLAY_STATE_OWNED)
//     {
//         /* Reset the video mode in case we do not have a custom boot logo */
//         CommandLine = (LoaderBlock->LoadOptions ? _strupr(LoaderBlock->LoadOptions) : NULL);
//         ResetMode   = (CommandLine == NULL) || (strstr(CommandLine, "BOOTLOGO") == NULL);
//     }
//
//     /* Initialize the video */
//     InbvBootDriverInstalled = VidInitialize(ResetMode);
//     if (InbvBootDriverInstalled)
//     {
//         /* Find bitmap resources in the kernel */
//         ResourceCount = min(Count, RTL_NUMBER_OF(ResourceList) - 1);
//         for (i = 1; i <= ResourceCount; i++)
//         {
//             /* Do the lookup */
//             ResourceList[i] = FindBitmapResource(LoaderBlock, i);
//         }
//
//         /* Set the progress bar ranges */
//         InbvSetProgressBarSubset(0, 100);
//
//         // BootAnimInitialize(LoaderBlock, Count);
//     }
//
//     /* Return install state */
//     return InbvBootDriverInstalled;
// }
//
// VOID
// NTAPI
// InbvAcquireLock(VOID)
// {
//     KIRQL OldIrql;
//
//     /* Check if we're at dispatch level or lower */
//     OldIrql = KeGetCurrentIrql();
//     if (OldIrql <= DISPATCH_LEVEL)
//     {
//         /* Loop until the lock is free */
//         while (!KeTestSpinLock(&BootDriverLock));
//
//         /* Raise IRQL to dispatch level */
//         KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//     }
//
//     /* Acquire the lock */
//     KiAcquireSpinLock(&BootDriverLock);
//     InbvOldIrql = OldIrql;
// }
//
// VOID
// NTAPI
// InbvReleaseLock(VOID)
// {
//     KIRQL OldIrql;
//
//     /* Capture the old IRQL */
//     OldIrql = InbvOldIrql;
//
//     /* Release the driver lock */
//     KiReleaseSpinLock(&BootDriverLock);
//
//     /* If we were at dispatch level or lower, restore the old IRQL */
//     if (InbvOldIrql <= DISPATCH_LEVEL) KeLowerIrql(OldIrql);
// }
//
// VOID
// NTAPI
// InbvEnableBootDriver(
//     _In_ BOOLEAN Enable)
// {
//     /* Check if we're installed */
//     if (InbvBootDriverInstalled)
//     {
//         /* Check for lost state */
//         if (InbvDisplayState >= INBV_DISPLAY_STATE_LOST) return;
//
//         /* Acquire the lock */
//         InbvAcquireLock();
//
//         /* Cleanup the screen if we own it */
//         if (InbvDisplayState == INBV_DISPLAY_STATE_OWNED) VidCleanUp();
//
//         /* Set the new display state */
//         InbvDisplayState = Enable ? INBV_DISPLAY_STATE_OWNED :
//                                     INBV_DISPLAY_STATE_DISABLED;
//
//         /* Release the lock */
//         InbvReleaseLock();
//     }
//     else
//     {
//         /* Set the new display state */
//         InbvDisplayState = Enable ? INBV_DISPLAY_STATE_OWNED :
//                                     INBV_DISPLAY_STATE_DISABLED;
//     }
// }
//
// VOID
// NTAPI
// InbvAcquireDisplayOwnership(VOID)
// {
//     /* Check if we have a callback and we're just acquiring it now */
//     if ((InbvResetDisplayParameters) &&
//         (InbvDisplayState == INBV_DISPLAY_STATE_LOST))
//     {
//         /* Call the callback */
//         InbvResetDisplayParameters(80, 50);
//     }
//
//     /* Acquire the display */
//     InbvDisplayState = INBV_DISPLAY_STATE_OWNED;
// }
//
// VOID
// NTAPI
// InbvSetDisplayOwnership(
//     _In_ BOOLEAN DisplayOwned)
// {
//     /* Set the new display state */
//     InbvDisplayState = DisplayOwned ? INBV_DISPLAY_STATE_OWNED :
//                                       INBV_DISPLAY_STATE_LOST;
// }
//
// BOOLEAN
// NTAPI
// InbvCheckDisplayOwnership(VOID)
// {
//     /* Return if we own it or not */
//     return InbvDisplayState != INBV_DISPLAY_STATE_LOST;
// }
//
// INBV_DISPLAY_STATE
// NTAPI
// InbvGetDisplayState(VOID)
// {
//     /* Return the actual state */
//     return InbvDisplayState;
// }
//
// BOOLEAN
// NTAPI
// InbvDisplayString(
//     _In_ PCSTR String)
// {
//     /* Make sure we own the display */
//     if (InbvDisplayState == INBV_DISPLAY_STATE_OWNED)
//     {
//         /* If we're not allowed, return success anyway */
//         if (!InbvDisplayDebugStrings)
//             return TRUE;
//
//         /* Check if a filter is installed */
//         if (InbvDisplayFilter)
//             InbvDisplayFilter(&String);
//
//         /* Acquire the lock */
//         InbvAcquireLock();
//
//         /* Make sure we're installed and display the string */
//         if (InbvBootDriverInstalled)
//             VidDisplayString(String);
//
//         /* Print the string on the EMS port */
//         HeadlessDispatch(HeadlessCmdPutString,
//                          (PVOID)String,
//                          strlen(String) + sizeof(ANSI_NULL),
//                          NULL,
//                          NULL);
//
//         /* Release the lock */
//         InbvReleaseLock();
//
//         /* All done */
//         return TRUE;
//     }
//
//     /* We don't own it, fail */
//     return FALSE;
// }
//
// BOOLEAN
// NTAPI
// InbvEnableDisplayString(
//     _In_ BOOLEAN Enable)
// {
//     BOOLEAN OldSetting;
//
//     /* Get the old setting */
//     OldSetting = InbvDisplayDebugStrings;
//
//     /* Update it */
//     InbvDisplayDebugStrings = Enable;
//
//     /* Return the old setting */
//     return OldSetting;
// }
//
// VOID
// NTAPI
// InbvInstallDisplayStringFilter(
//     _In_ INBV_DISPLAY_STRING_FILTER DisplayFilter)
// {
//     /* Save the filter */
//     InbvDisplayFilter = DisplayFilter;
// }
//
// BOOLEAN
// NTAPI
// InbvIsBootDriverInstalled(VOID)
// {
//     /* Return driver state */
//     return InbvBootDriverInstalled;
// }
//
// VOID
// NTAPI
// InbvNotifyDisplayOwnershipLost(
//     _In_ INBV_RESET_DISPLAY_PARAMETERS Callback)
// {
//     /* Check if we're installed */
//     if (InbvBootDriverInstalled)
//     {
//         /* Acquire the lock and cleanup if we own the screen */
//         InbvAcquireLock();
//         if (InbvDisplayState != INBV_DISPLAY_STATE_LOST) VidCleanUp();
//
//         /* Set the reset callback and display state */
//         InbvResetDisplayParameters = Callback;
//         InbvDisplayState = INBV_DISPLAY_STATE_LOST;
//
//         /* Release the lock */
//         InbvReleaseLock();
//     }
//     else
//     {
//         /* Set the reset callback and display state */
//         InbvResetDisplayParameters = Callback;
//         InbvDisplayState = INBV_DISPLAY_STATE_LOST;
//     }
// }
//
// BOOLEAN
// NTAPI
// InbvResetDisplay(VOID)
// {
//     /* Check if we're installed and we own it */
//     if (InbvBootDriverInstalled &&
//         (InbvDisplayState == INBV_DISPLAY_STATE_OWNED))
//     {
//         /* Do the reset */
//         VidResetDisplay(TRUE);
//         return TRUE;
//     }
//
//     /* Nothing to reset */
//     return FALSE;
// }
//
// VOID
// NTAPI
// InbvSetScrollRegion(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ ULONG Bottom)
// {
//     /* Just call bootvid */
//     VidSetScrollRegion(Left, Top, Right, Bottom);
// }
//
// VOID
// NTAPI
// InbvSetTextColor(
//     _In_ ULONG Color)
// {
//     HEADLESS_CMD_SET_COLOR HeadlessSetColor;
//
//     /* Set color for EMS port */
// #ifdef INBV_HEADLESS_COLORS
//     InbvTerminalTextColor = 30 + CGA_TO_ANSI_COLOR(Color);
// #else
//     InbvTerminalTextColor = 37;
// #endif
//     HeadlessSetColor.TextColor = InbvTerminalTextColor;
//     HeadlessSetColor.BkgdColor = InbvTerminalBkgdColor;
//     HeadlessDispatch(HeadlessCmdSetColor,
//                      &HeadlessSetColor,
//                      sizeof(HeadlessSetColor),
//                      NULL,
//                      NULL);
//
//     /* Update the text color */
//     VidSetTextColor(Color);
// }
//
// VOID
// NTAPI
// InbvSolidColorFill(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ ULONG Bottom,
//     _In_ ULONG Color)
// {
//     HEADLESS_CMD_SET_COLOR HeadlessSetColor;
//
//     /* Make sure we own it */
//     if (InbvDisplayState == INBV_DISPLAY_STATE_OWNED)
//     {
//         /* Acquire the lock */
//         InbvAcquireLock();
//
//         /* Check if we're installed */
//         if (InbvBootDriverInstalled)
//         {
//             /* Call bootvid */
//             VidSolidColorFill(Left, Top, Right, Bottom, (UCHAR)Color);
//         }
//
//         /* Set color for EMS port and clear display */
// #ifdef INBV_HEADLESS_COLORS
//         InbvTerminalBkgdColor = 40 + CGA_TO_ANSI_COLOR(Color);
// #else
//         InbvTerminalBkgdColor = 40;
// #endif
//         HeadlessSetColor.TextColor = InbvTerminalTextColor;
//         HeadlessSetColor.BkgdColor = InbvTerminalBkgdColor;
//         HeadlessDispatch(HeadlessCmdSetColor,
//                          &HeadlessSetColor,
//                          sizeof(HeadlessSetColor),
//                          NULL,
//                          NULL);
//         HeadlessDispatch(HeadlessCmdClearDisplay,
//                          NULL, 0,
//                          NULL, NULL);
//
//         /* Release the lock */
//         InbvReleaseLock();
//     }
// }
//
// VOID
// NTAPI
// InbvBitBlt(
//     _In_ PUCHAR Buffer,
//     _In_ ULONG X,
//     _In_ ULONG Y)
// {
//     /* Check if we're installed and we own it */
//     if (InbvBootDriverInstalled &&
//         (InbvDisplayState == INBV_DISPLAY_STATE_OWNED))
//     {
//         /* Acquire the lock */
//         InbvAcquireLock();
//
//         /* Do the blit */
//         VidBitBlt(Buffer, X, Y);
//
//         /* Release the lock */
//         InbvReleaseLock();
//     }
// }
//
// VOID
// NTAPI
// InbvBufferToScreenBlt(
//     _In_reads_bytes_(Delta * Height) PUCHAR Buffer,
//     _In_ ULONG X,
//     _In_ ULONG Y,
//     _In_ ULONG Width,
//     _In_ ULONG Height,
//     _In_ ULONG Delta)
// {
//     /* Check if we're installed and we own it */
//     if (InbvBootDriverInstalled &&
//         (InbvDisplayState == INBV_DISPLAY_STATE_OWNED))
//     {
//         /* Do the blit */
//         VidBufferToScreenBlt(Buffer, X, Y, Width, Height, Delta);
//     }
// }
//
// VOID
// NTAPI
// InbvScreenToBufferBlt(
//     _Out_writes_bytes_all_(Delta * Height) PUCHAR Buffer,
//     _In_ ULONG X,
//     _In_ ULONG Y,
//     _In_ ULONG Width,
//     _In_ ULONG Height,
//     _In_ ULONG Delta)
// {
//     /* Check if we're installed and we own it */
//     if (InbvBootDriverInstalled &&
//         (InbvDisplayState == INBV_DISPLAY_STATE_OWNED))
//     {
//         /* Do the blit */
//         VidScreenToBufferBlt(Buffer, X, Y, Width, Height, Delta);
//     }
// }
//
// /**
//  * @brief
//  * Sets the screen coordinates of the loading progress bar and enable it.
//  *
//  * @param[in]   Left
//  * @param[in]   Top
//  * The left/top coordinates.
//  *
//  * @return None.
//  **/
// VOID
// NTAPI
// InbvSetProgressBarCoordinates(
//     _In_ ULONG Left,
//     _In_ ULONG Top)
// {
//     /* Update the coordinates */
//     ProgressBarLeft = Left;
//     ProgressBarTop  = Top;
//
//     /* Enable the progress bar */
//     ShowProgressBar = TRUE;
// }
//
// /**
//  * @brief
//  * Gives some progress feedback, without specifying any explicit number
//  * of progress steps or percentage.
//  * The corresponding percentage is derived from the progress indicator's
//  * current count, capped to the number of expected calls to be made to
//  * this function (default: 25, see @b InbvProgressIndicator.Expected).
//  *
//  * @return None.
//  **/
// CODE_SEG("INIT")
// VOID
// NTAPI
// InbvIndicateProgress(VOID)
// {
//     ULONG Percentage;
//
//     /* Increase progress */
//     InbvProgressIndicator.Count++;
//
//     /* Compute the new percentage - Don't go over 100% */
//     Percentage = 100 * InbvProgressIndicator.Count /
//                        InbvProgressIndicator.Expected;
//     Percentage = min(Percentage, 99);
//
//     if (Percentage != InbvProgressIndicator.Percentage)
//     {
//         /* Percentage has changed, update the progress bar */
//         InbvProgressIndicator.Percentage = Percentage;
//         InbvUpdateProgressBar(Percentage);
//     }
// }
//
// /**
//  * @brief
//  * Specifies a progress percentage sub-range.
//  * Further calls to InbvIndicateProgress() or InbvUpdateProgressBar()
//  * will update the progress percentage relative to this sub-range.
//  * In particular, the percentage provided to InbvUpdateProgressBar()
//  * is relative to this sub-range.
//  *
//  * @param[in]   Floor
//  * The lower bound percentage of the sub-range (default: 0).
//  *
//  * @param[in]   Ceiling
//  * The upper bound percentage of the sub-range (default: 100).
//  *
//  * @return None.
//  **/
// VOID
// NTAPI
// InbvSetProgressBarSubset(
//     _In_ ULONG Floor,
//     _In_ ULONG Ceiling)
// {
//     /* Sanity checks */
//     ASSERT(Floor < Ceiling);
//     ASSERT(Ceiling <= 100);
//
//     /* Update the progress bar state */
//     InbvProgressState.Floor = Floor * 100;
//     InbvProgressState.Ceiling = Ceiling * 100;
//     InbvProgressState.Bias = Ceiling - Floor;
// }
//
// /**
//  * @brief
//  * Updates the progress bar percentage, relative to the current
//  * percentage sub-range previously set by InbvSetProgressBarSubset().
//  *
//  * @param[in]   Percentage
//  * The progress percentage, relative to the current sub-range.
//  *
//  * @return None.
//  **/
// VOID
// NTAPI
// InbvUpdateProgressBar(
//     _In_ ULONG Percentage)
// {
//     ULONG TotalProgress;
//
//     /* Make sure the progress bar is enabled, that we own and are installed */
//     if (ShowProgressBar &&
//         InbvBootDriverInstalled &&
//         (InbvDisplayState == INBV_DISPLAY_STATE_OWNED))
//     {
//         /* Compute the total progress and tick the progress bar */
//         TotalProgress = InbvProgressState.Floor + (Percentage * InbvProgressState.Bias);
//         // TotalProgress /= (100 * 100);
//
//         BootAnimTickProgressBar(TotalProgress);
//     }
// }
//
// NTSTATUS
// NTAPI
// NtDisplayString(IN PUNICODE_STRING DisplayString)
// {
//     NTSTATUS Status;
//     UNICODE_STRING CapturedString;
//     OEM_STRING OemString;
//     ULONG OemLength;
//     KPROCESSOR_MODE PreviousMode;
//
//     PAGED_CODE();
//
//     PreviousMode = ExGetPreviousMode();
//
//     /* We require the TCB privilege */
//     if (!SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode))
//         return STATUS_PRIVILEGE_NOT_HELD;
//
//     /* Capture the string */
//     Status = ProbeAndCaptureUnicodeString(&CapturedString, PreviousMode, DisplayString);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Do not display the string if it is empty */
//     if (CapturedString.Length == 0 || CapturedString.Buffer == NULL)
//     {
//         Status = STATUS_SUCCESS;
//         goto Quit;
//     }
//
//     /*
//      * Convert the string since INBV understands only ANSI/OEM. Allocate the
//      * string buffer in non-paged pool because INBV passes it down to BOOTVID.
//      * We cannot perform the allocation using RtlUnicodeStringToOemString()
//      * since its allocator uses PagedPool.
//      */
//     OemLength = RtlUnicodeStringToOemSize(&CapturedString);
//     if (OemLength > MAXUSHORT)
//     {
//         Status = STATUS_BUFFER_OVERFLOW;
//         goto Quit;
//     }
//     RtlInitEmptyAnsiString((PANSI_STRING)&OemString, NULL, (USHORT)OemLength);
//     OemString.Buffer = ExAllocatePoolWithTag(NonPagedPool, OemLength, TAG_OSTR);
//     if (OemString.Buffer == NULL)
//     {
//         Status = STATUS_NO_MEMORY;
//         goto Quit;
//     }
//     Status = RtlUnicodeStringToOemString(&OemString, &CapturedString, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(OemString.Buffer, TAG_OSTR);
//         goto Quit;
//     }
//
//     /* Display the string */
//     InbvDisplayString(OemString.Buffer);
//
//     /* Free the string buffer */
//     ExFreePoolWithTag(OemString.Buffer, TAG_OSTR);
//
//     Status = STATUS_SUCCESS;
//
// Quit:
//     /* Free the captured string */
//     ReleaseCapturedUnicodeString(&CapturedString, PreviousMode);
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<INBV_PROGRESS_STATE>() == 12);
    assert!(core::mem::offset_of!(INBV_PROGRESS_STATE, Floor) == 0);
    assert!(core::mem::offset_of!(INBV_PROGRESS_STATE, Ceiling) == 4);
    assert!(core::mem::offset_of!(INBV_PROGRESS_STATE, Bias) == 8);
    assert!(core::mem::size_of::<BT_PROGRESS_INDICATOR>() == 12);
    assert!(core::mem::offset_of!(BT_PROGRESS_INDICATOR, Count) == 0);
    assert!(core::mem::offset_of!(BT_PROGRESS_INDICATOR, Expected) == 4);
    assert!(core::mem::offset_of!(BT_PROGRESS_INDICATOR, Percentage) == 8);
    assert!(core::mem::size_of::<LDR_RESOURCE_INFO>() == 24);
    assert!(core::mem::offset_of!(LDR_RESOURCE_INFO, Type) == 0);
    assert!(core::mem::offset_of!(LDR_RESOURCE_INFO, Name) == 8);
    assert!(core::mem::offset_of!(LDR_RESOURCE_INFO, Language) == 16);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<INBV_PROGRESS_STATE>() == 12);
    assert!(core::mem::offset_of!(INBV_PROGRESS_STATE, Floor) == 0);
    assert!(core::mem::offset_of!(INBV_PROGRESS_STATE, Ceiling) == 4);
    assert!(core::mem::offset_of!(INBV_PROGRESS_STATE, Bias) == 8);
    assert!(core::mem::size_of::<BT_PROGRESS_INDICATOR>() == 12);
    assert!(core::mem::offset_of!(BT_PROGRESS_INDICATOR, Count) == 0);
    assert!(core::mem::offset_of!(BT_PROGRESS_INDICATOR, Expected) == 4);
    assert!(core::mem::offset_of!(BT_PROGRESS_INDICATOR, Percentage) == 8);
    assert!(core::mem::size_of::<LDR_RESOURCE_INFO>() == 12);
    assert!(core::mem::offset_of!(LDR_RESOURCE_INFO, Type) == 0);
    assert!(core::mem::offset_of!(LDR_RESOURCE_INFO, Name) == 4);
    assert!(core::mem::offset_of!(LDR_RESOURCE_INFO, Language) == 8);
};
// END OFFSET CHECKS
