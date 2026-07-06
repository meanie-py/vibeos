use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{
    HAL_DISPATCH,
    HalDisplayString, IoCreateDriver, IoDeleteDriver, IoRegisterBootDriverReinitialization,
    IoRegisterDriverReinitialization, ObMakeTemporaryObject, ANSI_STRING, CPPORT, PCSTR,
    PDRIVER_OBJECT, PLOADER_PARAMETER_BLOCK, PUNICODE_STRING, UNICODE_STRING, USHORT,
    _LIST_ENTRY, pHalInitPnpDriver,
};

// ============================================================================
// kd.h types — kdmain.c-owned globals (kdio.c is excluded; defined here)
// ============================================================================

// Number of native KD providers (KdScreen=0, KdSerial=1, KdFile=2). KDBG=3 not
// translated yet, so KdMax = 3 on all arches.
pub const KdMax: usize = 3;

/// KDP_DEBUG_MODE — bitfield in C (Screen/Serial/File at bits 0/1/2 of Value).
/// Modeled here as 4 separate fields. The "Value == 0" check in C is replaced
/// with a check on all three mode flags.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct KDP_DEBUG_MODE {
    pub Screen: u8,
    pub Serial: u8,
    pub File: u8,
    pub Value: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _KD_DISPATCH_TABLE {
    pub KdProvidersList: _LIST_ENTRY,
    pub KdpInitRoutine: PKDP_INIT_ROUTINE,
    pub KdpPrintRoutine: PKDP_PRINT_ROUTINE,
    pub InitStatus: NTSTATUS,
}
pub type KD_DISPATCH_TABLE = _KD_DISPATCH_TABLE;
pub type PKD_DISPATCH_TABLE = *mut _KD_DISPATCH_TABLE;

// PKDP_INIT_ROUTINE / PKDP_PRINT_ROUTINE — NTAPI on the C side; per-arch ABI.
#[cfg(not(target_arch = "x86"))]
pub type PKDP_INIT_ROUTINE = Option<
    unsafe extern "C" fn(DispatchTable: PKD_DISPATCH_TABLE, BootPhase: ULONG) -> NTSTATUS,
>;
#[cfg(target_arch = "x86")]
pub type PKDP_INIT_ROUTINE = Option<
    unsafe extern "stdcall" fn(DispatchTable: PKD_DISPATCH_TABLE, BootPhase: ULONG) -> NTSTATUS,
>;

#[cfg(not(target_arch = "x86"))]
pub type PKDP_PRINT_ROUTINE = Option<unsafe extern "C" fn(String: *const i8, Length: ULONG)>;
#[cfg(target_arch = "x86")]
pub type PKDP_PRINT_ROUTINE = Option<unsafe extern "stdcall" fn(String: *const i8, Length: ULONG)>;

// ============================================================================
// kd globals (kdio.c excluded; canonical definitions live here)
// ============================================================================

// kdio.c-owned globals (kdio.rs has the canonical definitions): extern only.
// kdterminal.c-owned KdbDebugState: extern from kdterminal.rs.
extern "C" {
    pub static mut KdpDebugMode: KDP_DEBUG_MODE;
    pub static mut SerialPortNumber: ULONG;
    pub static mut SerialPortInfo: CPPORT;
    pub static mut KdpLogFileName: ANSI_STRING;
    pub static mut KdProviders: _LIST_ENTRY;
    pub static mut DispatchTable: [KD_DISPATCH_TABLE; KdMax];
    pub static mut InitRoutines: [PKDP_INIT_ROUTINE; KdMax];
    pub static mut KdbDebugState: ULONG;
}

pub const KD_DEBUG_KDSERIAL: ULONG = 0x1;
pub const KD_DEBUG_KDNOECHO: ULONG = 0x2;

// `HalInitPnpDriver` is a C macro: `#define HalInitPnpDriver HALDISPATCH->HalInitPnpDriver`
// — a field in the `HalDispatchTable` global (defined in fstub/halstub.rs).
// We access it directly via `crate::fstub::halstub::HalDispatchTable.HalInitPnpDriver`.
// HalDispatchTable now lives in fstub-rust (crate-split Phase 1b); reference the
// #[no_mangle] global by extern decl instead of a monolith crate:: path.
extern "C" {
    static mut HalDispatchTable: HAL_DISPATCH;
}

#[no_mangle]
pub static mut orgHalInitPnpDriver: pHalInitPnpDriver = None;

const DRVO_INITIALIZED: ULONG = 0x00000010;
const FLG_STOP_ON_EXCEPTION: ULONG = 0x1;

// ============================================================================
// libc-style helpers from msvcrt
// ============================================================================

extern "C" {
    fn _strnicmp(s1: PCSTR, s2: PCSTR, n: usize) -> i32;
    fn strchr(s: PCSTR, c: i32) -> PCSTR;
    fn atol(s: PCSTR) -> i32;
    fn strtoul(s: PCSTR, endptr: *mut PCSTR, base: i32) -> ULONG;
    fn _strupr(s: PCHAR) -> PCHAR;
    fn strstr(s1: PCSTR, s2: PCSTR) -> PCSTR;
}

#[inline]
unsafe fn UlongToPtr(value: ULONG) -> PVOID {
    value as usize as PVOID
}

// _InterlockedCompareExchange8 — atomic CAS on a single byte. Returns OLD value.
#[inline]
unsafe fn _InterlockedCompareExchange8(dest: *mut i8, exchange: i8, comparand: i8) -> i8 {
    use core::sync::atomic::{AtomicI8, Ordering};
    let atomic = &*(dest as *const AtomicI8);
    match atomic.compare_exchange(comparand, exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(old) => old,
        Err(old) => old,
    }
}

// ============================================================================
// kdmain.c translations
// ============================================================================

unsafe fn KdpGetTerminalSettings(mut p1: PCSTR) {
    while !p1.is_null() && *p1 != 0 {
        while *p1 == b' ' as i8 {
            p1 = p1.add(1);
        }

        if _strnicmp(p1, b"KDSERIAL\0".as_ptr() as PCSTR, 8) == 0 {
            p1 = p1.add(8);
            KdbDebugState |= KD_DEBUG_KDSERIAL;
            KdpDebugMode.Serial = 1u8;
        } else if _strnicmp(p1, b"KDNOECHO\0".as_ptr() as PCSTR, 8) == 0 {
            p1 = p1.add(8);
            KdbDebugState |= KD_DEBUG_KDNOECHO;
        }

        p1 = strchr(p1, b' ' as i32);
    }
}

unsafe fn KdpGetDebugMode(Currentp2: PCHAR) -> PCHAR {
    let mut p2 = Currentp2;
    let mut p1: PCHAR;
    let mut Value: ULONG;

    if _strnicmp(p2, b"SCREEN\0".as_ptr() as PCSTR, 6) == 0 {
        p2 = p2.add(6);
        KdpDebugMode.Screen = 1u8;
    } else if _strnicmp(p2, b"COM\0".as_ptr() as PCSTR, 3) == 0 {
        p2 = p2.add(3);
        if *p2 != b':' as i8 {
            Value = atol(p2) as ULONG;
            if Value > 0 && Value < 5 {
                KdpDebugMode.Serial = 1u8;
                SerialPortNumber = Value;
            }
        } else {
            Value = strtoul(p2.add(1), core::ptr::null_mut(), 0);
            if Value != 0 {
                KdpDebugMode.Serial = 1u8;
                SerialPortInfo.Address = UlongToPtr(Value) as *mut u8;
                SerialPortNumber = 0;
            }
        }
    } else if _strnicmp(p2, b"FILE\0".as_ptr() as PCSTR, 4) == 0 {
        p2 = p2.add(4);
        KdpDebugMode.File = 1u8;
        if *p2 == b':' as i8 {
            p2 = p2.add(1);
            p1 = p2;
            while *p2 != 0 && *p2 != b' ' as i8 {
                p2 = p2.add(1);
            }
            KdpLogFileName.MaximumLength = p2.offset_from(p1) as u16;
            KdpLogFileName.Length = p2.offset_from(p1) as u16;
            KdpLogFileName.Buffer = p1;
        }
    }

    p2
}

#[no_mangle]
pub unsafe extern "system" fn KdDebuggerInitialize0(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    // C-side initializer for KdProviders is `LIST_ENTRY KdProviders = {&KdProviders, &KdProviders};`
    // — a self-referencing empty list. Rust can't express that in const init, so
    // initialize at runtime here, before any InsertTailList(&KdProviders, ...) calls.
    InitializeListHead(core::ptr::addr_of_mut!(KdProviders));

    let mut CommandLine: PCHAR;
    let mut Port: PCHAR = core::ptr::null_mut();
    let mut i: ULONG;
    let mut Success: BOOLEAN = 0u8;

    if !LoaderBlock.is_null() {
        CommandLine = (*LoaderBlock).LoadOptions;
        if !CommandLine.is_null() {
            _strupr(CommandLine);
            KdpGetTerminalSettings(CommandLine);
            Port = strstr(CommandLine, b"DEBUGPORT\0".as_ptr() as PCSTR) as PCHAR;
        }
    }

    while !Port.is_null() {
        Port = Port.add(b"DEBUGPORT\0".len() - 1);
        while *Port == b' ' as i8 {
            Port = Port.add(1);
        }
        Port = Port.add(1);
        Port = KdpGetDebugMode(Port);
        Port = strstr(Port, b"DEBUGPORT\0".as_ptr() as PCSTR) as PCHAR;
    }

    // No mode selected → default to serial.
    if KdpDebugMode.Screen == 0 && KdpDebugMode.Serial == 0 && KdpDebugMode.File == 0
        && KdpDebugMode.Value == 0
    {
        KdpDebugMode.Serial = 1u8;
    }

    i = 0;
    while (i as usize) < KdMax {
        let entry = core::ptr::addr_of_mut!(DispatchTable[i as usize]);
        if let Some(routine) = InitRoutines[i as usize] {
            (*entry).InitStatus = routine(entry, 0);
        }
        Success = if Success != 0 || NT_SUCCESS((*entry).InitStatus) {
            1u8
        } else {
            0u8
        };
        i += 1;
    }

    if Success != 0 {
        STATUS_SUCCESS
    } else {
        STATUS_UNSUCCESSFUL
    }
}

// NTAPI on i386 = stdcall, on amd64/arm64 = cdecl/AAPCS64 (extern "C").
// We must match the bindgen function-pointer types exactly per arch.
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KdpDriverReinit(
    DriverObject: PDRIVER_OBJECT,
    _Context: PVOID,
    Count: ULONG,
) {
    KdpDriverReinit_impl(DriverObject, _Context, Count)
}

#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn KdpDriverReinit(
    DriverObject: PDRIVER_OBJECT,
    _Context: PVOID,
    Count: ULONG,
) {
    KdpDriverReinit_impl(DriverObject, _Context, Count)
}

unsafe fn KdpDriverReinit_impl(
    DriverObject: PDRIVER_OBJECT,
    _Context: PVOID,
    Count: ULONG,
) {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut CurrentTable: PKD_DISPATCH_TABLE;
    let mut KdpInitRoutine: PKDP_INIT_ROUTINE;
    let BootPhase = Count + 1;
    let mut ScheduleReinit: BOOLEAN = 0u8;

    CurrentEntry = (*core::ptr::addr_of_mut!(KdProviders)).Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(KdProviders) {
        CurrentTable = (CurrentEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_KD_DISPATCH_TABLE, KdProvidersList) as isize))
            as PKD_DISPATCH_TABLE;

        CurrentEntry = (*CurrentEntry).Flink;

        if (*CurrentTable).KdpInitRoutine.is_some() {
            KdpInitRoutine = (*CurrentTable).KdpInitRoutine;
            (*CurrentTable).KdpInitRoutine = None;
            (*CurrentTable).InitStatus = KdpInitRoutine.unwrap()(CurrentTable, BootPhase);

            if (*CurrentTable).KdpInitRoutine.is_some() {
                ScheduleReinit = 1u8;
            }
        }
    }

    if ScheduleReinit != 0 {
        if Count <= 1 {
            IoRegisterBootDriverReinitialization(
                DriverObject,
                Some(KdpDriverReinit),
                0 as PVOID,
            );
        } else if Count <= 3 {
            IoRegisterDriverReinitialization(
                DriverObject,
                Some(KdpDriverReinit),
                1 as PVOID,
            );
        } else {
            ScheduleReinit = 0u8;
        }
    }

    if ScheduleReinit == 0 {
        ObMakeTemporaryObject(DriverObject as *mut core::ffi::c_void);
        IoDeleteDriver(DriverObject);
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KdpDriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    KdpDriverEntry_impl(DriverObject, RegistryPath)
}

#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn KdpDriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    KdpDriverEntry_impl(DriverObject, RegistryPath)
}

unsafe fn KdpDriverEntry_impl(
    DriverObject: PDRIVER_OBJECT,
    _RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    IoRegisterBootDriverReinitialization(
        DriverObject,
        Some(KdpDriverReinit),
        0 as PVOID,
    );

    (*DriverObject).Flags |= DRVO_INITIALIZED;
    STATUS_SUCCESS
}

// L"\\Driver\\KdDriver" as a static u16 array.
static KDDRIVER_NAME_BUF: [u16; 17] = [
    b'\\' as u16, b'D' as u16, b'r' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'r' as u16,
    b'\\' as u16, b'K' as u16, b'd' as u16, b'D' as u16, b'r' as u16, b'i' as u16, b'v' as u16,
    b'e' as u16, b'r' as u16, 0,
];

unsafe fn KdpInitDriver() -> NTSTATUS {
    static mut InitCalled: i8 = 0i8;
    let Status: NTSTATUS;
    // RTL_CONSTANT_STRING(L"\\Driver\\KdDriver") — Length excludes NUL, MaximumLength includes it.
    let mut DriverName: UNICODE_STRING = UNICODE_STRING {
        Length: ((KDDRIVER_NAME_BUF.len() - 1) * 2) as USHORT,
        MaximumLength: (KDDRIVER_NAME_BUF.len() * 2) as USHORT,
        Buffer: KDDRIVER_NAME_BUF.as_ptr() as *mut u16,
    };

    if KeGetCurrentIrql() != PASSIVE_LEVEL as u8 {
        panic!("ASSERT failed");
    }

    if _InterlockedCompareExchange8(
        core::ptr::addr_of_mut!(InitCalled),
        1i8,
        0i8,
    ) != 0i8
    {
        return STATUS_SUCCESS;
    }

    Status = IoCreateDriver(core::ptr::addr_of_mut!(DriverName), Some(KdpDriverEntry));
    if !NT_SUCCESS(Status) {
        crate::dbg_print!(b"IoCreateDriver failed: 0x%08lx\n\0", Status);
    }

    InterlockedExchangePointer(
        core::ptr::addr_of_mut!(HalDispatchTable.HalInitPnpDriver) as *mut PVOID,
        orgHalInitPnpDriver
            .map(|f| f as *mut core::ffi::c_void)
            .unwrap_or(core::ptr::null_mut()),
    );
    if HalDispatchTable.HalInitPnpDriver.is_some() {
        HalDispatchTable.HalInitPnpDriver.unwrap()()
    } else {
        STATUS_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "system" fn KdDebuggerInitialize1(
    _LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut CurrentTable: PKD_DISPATCH_TABLE;
    let mut KdpInitRoutine: PKDP_INIT_ROUTINE;
    let mut Success: BOOLEAN = 0u8;
    let mut ReinitForPhase2: BOOLEAN = 0u8;

    // Defensively initialize KdProviders if KdDebuggerInitialize0 wasn't called
    // (e.g., enable_kd was false). Rust's static init can't express the C
    // self-referencing initializer `LIST_ENTRY KdProviders = {&KdProviders, &KdProviders}`.
    if KdProviders.Flink.is_null() {
        InitializeListHead(core::ptr::addr_of_mut!(KdProviders));
    }

    HalDisplayString(b"\r\n\0".as_ptr() as PCSTR);

    CurrentEntry = (*core::ptr::addr_of_mut!(KdProviders)).Flink;
    while CurrentEntry != core::ptr::addr_of_mut!(KdProviders) {
        CurrentTable = (CurrentEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_KD_DISPATCH_TABLE, KdProvidersList) as isize))
            as PKD_DISPATCH_TABLE;

        CurrentEntry = (*CurrentEntry).Flink;

        KdpInitRoutine = (*CurrentTable).KdpInitRoutine;
        (*CurrentTable).KdpInitRoutine = None;

        (*CurrentTable).InitStatus = KdpInitRoutine.unwrap()(CurrentTable, 1);

        if NT_SUCCESS((*CurrentTable).InitStatus) {
            Success = 1u8;
        }
        if (*CurrentTable).KdpInitRoutine.is_some() {
            ReinitForPhase2 = 1u8;
        }
    }

    HalDisplayString(b"\r\n\0".as_ptr() as PCSTR);

    NtGlobalFlag |= FLG_STOP_ON_EXCEPTION;

    if ReinitForPhase2 == 0u8 {
        return if Success != 0u8 {
            STATUS_SUCCESS
        } else {
            STATUS_UNSUCCESSFUL
        };
    }

    let init_driver_ptr: pHalInitPnpDriver = Some(KdpInitDriver_extern_c);
    let prev = InterlockedExchangePointer(
        core::ptr::addr_of_mut!(HalDispatchTable.HalInitPnpDriver) as *mut PVOID,
        init_driver_ptr
            .map(|f| f as *mut core::ffi::c_void)
            .unwrap_or(core::ptr::null_mut()),
    );
    orgHalInitPnpDriver = core::mem::transmute::<*mut core::ffi::c_void, pHalInitPnpDriver>(prev);
    STATUS_SUCCESS
}

// Wrapper: convert KdpInitDriver (Rust ABI, internal) to the per-arch C ABI
// for the pHalInitPnpDriver function-pointer slot. NTAPI = stdcall on i386,
// cdecl/AAPCS64 (extern "C") on amd64/arm64.
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn KdpInitDriver_extern_c() -> NTSTATUS {
    KdpInitDriver()
}

#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn KdpInitDriver_extern_c() -> NTSTATUS {
    KdpInitDriver()
}

// Public KD-protocol entry points. When KDBG is enabled, callers hit
// KdbgD0Transition / KdbgSendPacket / etc. (because <ntoskrnl.h> redefines
// `KdSendPacket` -> `KdbgSendPacket` and so on). The KDBG-side wrappers
// (kdbg/kdbg.rs) export the Kdbg* names and delegate back to these
// "underlying" names via plain function calls. So the names defined here
// are always present and always the real implementation.
#[no_mangle]
pub unsafe extern "system" fn KdD0Transition() -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdD3Transition() -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdSave(_SleepTransition: BOOLEAN) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KdRestore(_SleepTransition: BOOLEAN) -> NTSTATUS {
    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Kernel Debugger Initialization
//  * COPYRIGHT:   Copyright 2005 Alex Ionescu <alex.ionescu@reactos.org>
//  *              Copyright 2020 Hervé Poussineau <hpoussin@reactos.org>
//  *              Copyright 2023 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
//  */
//
// (See ntoskrnl/kd/kdmain.c for the original; trimmed here.)
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
