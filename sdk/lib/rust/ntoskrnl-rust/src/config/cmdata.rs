//! cmdata.c → Rust translation (Phase A: verbatim).
//!
//! Configuration Manager — global data definitions:
//!  - Simple globals (DummyData, CmNtGlobalFlag, language IDs, alloc state, …)
//!  - CmpMachineHiveList[] — list of hives loaded at boot
//!  - CmClassName[]/CmTypeName[]/CmpMultifunctionTypes[] — registry hardware enumeration tables
//!  - CmControlVector[] — registry-key → variable-pointer dispatch table (~200 entries)
//!
//! Phase B (planned) will chop CmControlVector into per-subsystem slices —
//! see docs/engineering/cmdata-chopping-plan.md.
//!
//! Cross-subsystem variables are referenced via `core::ptr::addr_of_mut!(crate::xxx::yyy::Var)`
//! when the variable is in another Rust module, or via `extern "C" { static mut ... }` blocks
//! when it's still in C.

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use core::ffi::c_void;
use ntkrnl_sys::ddk::{
    _CM_SYSTEM_CONTROL_VECTOR, _HIVE_LIST_ENTRY, _CMP_MF_TYPE,
    _UNICODE_STRING, _CMHIVE, _LIST_ENTRY, _KGUARDED_MUTEX, _EX_PUSH_LOCK,
    PCM_FULL_RESOURCE_DESCRIPTOR, PWCHAR, PVOID, PULONG,
    HANDLE, USHORT, ULONG, BOOLEAN, WCHAR,
};

// _CONFIGURATION_TYPE_MaximumType=41 — gives 42 entries in CmTypeName/CmpTypeCount.
// _CONFIGURATION_CLASS_MaximumClass=7 — gives 8 entries in CmClassName.
const MaximumType: usize = 41;
const MaximumClass: usize = 7;

// `_CM_SYSTEM_CONTROL_VECTOR` contains raw `*mut` pointers, so it isn't `Sync`.
// All access goes through `CmGetSystemControlValues` which is single-threaded
// at boot, so a manual Sync impl is sound. Subsystem files declare
// `pub static FOO_CONTROL_VECTOR: SubVector<N> = SubVector(unsafe { [...] });`
// using this wrapper.
#[repr(transparent)]
pub struct SubVector<const N: usize>(pub [_CM_SYSTEM_CONTROL_VECTOR; N]);
unsafe impl<const N: usize> Sync for SubVector<N> {}

// Wide-string literal helper. Returns &'static [u16] terminated with a NUL.
// Pattern lifted from config/cmconfig.rs. Exported so subsystem files can
// build their own CONTROL_VECTOR slices (see Phase B).
#[macro_export]
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

// CM_SYSTEM_CONTROL_VECTOR entry constructor — covers the common 3-arg form
// (NULL length+type) plus the full 5-arg form. `$buf`/`$len`/`$ty` are raw
// pointer expressions (e.g. `core::ptr::addr_of_mut!(crate::ob::obinit::Foo)`).
// Exported so subsystem files can build their own CONTROL_VECTOR slices.
// Uses absolute paths for type names so callers don't need extra `use`s.
#[macro_export]
macro_rules! cv {
    ($key:literal, $val:literal, $buf:expr) => {
        ntkrnl_sys::ddk::_CM_SYSTEM_CONTROL_VECTOR {
            KeyPath: $crate::w!($key).as_ptr() as ntkrnl_sys::ddk::PWCHAR,
            ValueName: $crate::w!($val).as_ptr() as ntkrnl_sys::ddk::PWCHAR,
            Buffer: $buf as ntkrnl_sys::ddk::PVOID,
            BufferLength: core::ptr::null_mut(),
            Type: core::ptr::null_mut(),
        }
    };
    ($key:literal, $val:literal, $buf:expr, $len:expr) => {
        ntkrnl_sys::ddk::_CM_SYSTEM_CONTROL_VECTOR {
            KeyPath: $crate::w!($key).as_ptr() as ntkrnl_sys::ddk::PWCHAR,
            ValueName: $crate::w!($val).as_ptr() as ntkrnl_sys::ddk::PWCHAR,
            Buffer: $buf as ntkrnl_sys::ddk::PVOID,
            BufferLength: $len as ntkrnl_sys::ddk::PULONG,
            Type: core::ptr::null_mut(),
        }
    };
    ($key:literal, $val:literal, $buf:expr, $len:expr, $ty:expr $(,)?) => {
        ntkrnl_sys::ddk::_CM_SYSTEM_CONTROL_VECTOR {
            KeyPath: $crate::w!($key).as_ptr() as ntkrnl_sys::ddk::PWCHAR,
            ValueName: $crate::w!($val).as_ptr() as ntkrnl_sys::ddk::PWCHAR,
            Buffer: $buf as ntkrnl_sys::ddk::PVOID,
            BufferLength: $len as ntkrnl_sys::ddk::PULONG,
            Type: $ty as ntkrnl_sys::ddk::PULONG,
        }
    };
}

// CM_DEBUG_PRINT_FILTER expansion — references Kd_<NAME>_Mask in kd64::kddata.
// Caller passes the literal name and full path (paste! crate not available).
macro_rules! kd_filter {
    ($name:literal, $sym:path) => {
        _CM_SYSTEM_CONTROL_VECTOR {
            KeyPath: w!("Session Manager\\Debug Print Filter").as_ptr() as PWCHAR,
            ValueName: w!($name).as_ptr() as PWCHAR,
            Buffer: unsafe { core::ptr::addr_of_mut!($sym) } as PVOID,
            BufferLength: core::ptr::null_mut(),
            Type: core::ptr::null_mut(),
        }
    };
}

// ============================================================================
// Simple globals (cmdata.c lines 17-80)
// ============================================================================

#[no_mangle] pub static mut DummyData: ULONG = 0;
// CmNtGlobalFlag is also defined in ex/init.rs? No — only declared there; we own it.

// MmProductType is `extern ULONG` — defined in ARM3::mminit. Don't redeclare.

#[no_mangle] pub static mut CmDefaultLanguageId: [WCHAR; 12] = [0; 12];
#[no_mangle] pub static mut CmDefaultLanguageIdLength: ULONG =
    (core::mem::size_of::<[WCHAR; 12]>()) as ULONG;
#[no_mangle] pub static mut CmDefaultLanguageIdType: ULONG = 0;

#[no_mangle] pub static mut CmInstallUILanguageId: [WCHAR; 12] = [0; 12];
#[no_mangle] pub static mut CmInstallUILanguageIdLength: ULONG =
    (core::mem::size_of::<[WCHAR; 12]>()) as ULONG;
#[no_mangle] pub static mut CmInstallUILanguageIdType: ULONG = 0;

#[no_mangle] pub static mut CmSuiteBuffer: [WCHAR; 128] = [0; 128];
#[no_mangle] pub static mut CmSuiteBufferLength: ULONG =
    (core::mem::size_of::<[WCHAR; 128]>()) as ULONG;
#[no_mangle] pub static mut CmSuiteBufferType: ULONG = 0;

#[no_mangle] pub static mut CmControlHive: _CMHIVE = unsafe { core::mem::zeroed() };

#[no_mangle] pub static mut CmpConfigurationAreaSize: ULONG = 4096 * 4; // PAGE_SIZE * 4
#[no_mangle] pub static mut CmpConfigurationData: PCM_FULL_RESOURCE_DESCRIPTOR =
    core::ptr::null_mut();

#[no_mangle] pub static mut CmpHiveListHeadLock: _EX_PUSH_LOCK =
    unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpLoadHiveLock: _EX_PUSH_LOCK =
    unsafe { core::mem::zeroed() };

// Globals moved here from cmalloc.c when it was translated to Rust — kept here
// so they're declared once for cross-subsystem use. (Original C-side comment
// preserved.)
#[no_mangle] pub static mut CmpAllocInited: BOOLEAN = 0;
#[no_mangle] pub static mut CmpAllocBucketLock: _KGUARDED_MUTEX =
    unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpFreeKCBListHead: _LIST_ENTRY =
    unsafe { core::mem::zeroed() };

#[no_mangle] pub static mut CmpDelayAllocBucketLock: _KGUARDED_MUTEX =
    unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut CmpFreeDelayItemsListHead: _LIST_ENTRY =
    unsafe { core::mem::zeroed() };

// ============================================================================
// CmSymbolicLinkValueName / CmpLoadOptions — UNICODE_STRINGs
// ============================================================================

const CM_SYMBOLIC_LINK_VALUE_NAME_BUF: &[u16] = w!("SymbolicLinkValue");

#[no_mangle] pub static mut CmSymbolicLinkValueName: _UNICODE_STRING = _UNICODE_STRING {
    // RTL_CONSTANT_STRING: Length = byte len excluding NUL (17 chars × 2 = 34),
    // MaximumLength = byte len including NUL (18 × 2 = 36).
    Length: ("SymbolicLinkValue".len() * 2) as u16,
    MaximumLength: (("SymbolicLinkValue".len() + 1) * 2) as u16,
    Buffer: CM_SYMBOLIC_LINK_VALUE_NAME_BUF.as_ptr() as PWCHAR,
};

#[no_mangle] pub static mut CmpLoadOptions: _UNICODE_STRING = unsafe { core::mem::zeroed() };

#[no_mangle] pub static mut CmpVolatileBoot: ULONG = 0;
#[no_mangle] pub static mut CmpShareSystemHives: BOOLEAN = 0;
#[no_mangle] pub static mut CmpMiniNTBoot: BOOLEAN = 0;

#[no_mangle] pub static mut CmpBootType: ULONG = 0;
#[no_mangle] pub static mut CmSelfHeal: ULONG = 1; // TRUE
#[no_mangle] pub static mut CmpSelfHeal: BOOLEAN = 1; // TRUE

#[no_mangle] pub static mut CmpUnknownBusCount: USHORT = 0;
#[no_mangle] pub static mut CmpTypeCount: [ULONG; MaximumType + 1] = [0; MaximumType + 1];

#[no_mangle] pub static mut CmpRegistryRootHandle: HANDLE = core::ptr::null_mut();

// CmNtGlobalFlag is defined in ex/init.rs — we just declare a shadow here for
// access by CmControlVector. Wait — actually the C cmdata.c DEFINES it. So we
// own it.
#[no_mangle] pub static mut CmNtGlobalFlag: ULONG = 0;

// ============================================================================
// CmpMachineHiveList — list of hives to load at boot.
// HIVE_LIST_ENTRY layout: { Name, BaseName, CmHive, HHiveFlags, CmHiveFlags,
//                          CmHive2, ThreadFinished, ThreadStarted, Allocate }
// ============================================================================

const HIVE_VOLATILE: ULONG = 1;
const HIVE_NOLAZYFLUSH: ULONG = 8;

const HL_HARDWARE_NAME: &[u16] = w!("HARDWARE");
const HL_SECURITY_NAME: &[u16] = w!("SECURITY");
const HL_SOFTWARE_NAME: &[u16] = w!("SOFTWARE");
const HL_SYSTEM_NAME: &[u16] = w!("SYSTEM");
const HL_DEFAULT_NAME: &[u16] = w!("DEFAULT");
const HL_SAM_NAME: &[u16] = w!("SAM");
const HL_MACHINE_PATH: &[u16] = w!("MACHINE\\");
const HL_USER_DEFAULT_PATH: &[u16] = w!("USER\\.DEFAULT");

const fn hive_entry(
    name: *mut WCHAR,
    base: *mut WCHAR,
    flags: ULONG,
) -> _HIVE_LIST_ENTRY {
    _HIVE_LIST_ENTRY {
        Name: name,
        BaseName: base,
        CmHive: core::ptr::null_mut(),
        HHiveFlags: flags,
        CmHiveFlags: 0,
        CmHive2: core::ptr::null_mut(),
        ThreadFinished: 0, // FALSE
        ThreadStarted: 0,
        Allocate: 0,
    }
}

#[no_mangle]
pub static mut CmpMachineHiveList: [_HIVE_LIST_ENTRY; 7] = [
    hive_entry(HL_HARDWARE_NAME.as_ptr() as *mut WCHAR,
               HL_MACHINE_PATH.as_ptr() as *mut WCHAR, HIVE_VOLATILE),
    hive_entry(HL_SECURITY_NAME.as_ptr() as *mut WCHAR,
               HL_MACHINE_PATH.as_ptr() as *mut WCHAR, 0),
    hive_entry(HL_SOFTWARE_NAME.as_ptr() as *mut WCHAR,
               HL_MACHINE_PATH.as_ptr() as *mut WCHAR, 0),
    hive_entry(HL_SYSTEM_NAME.as_ptr() as *mut WCHAR,
               HL_MACHINE_PATH.as_ptr() as *mut WCHAR, 0),
    hive_entry(HL_DEFAULT_NAME.as_ptr() as *mut WCHAR,
               HL_USER_DEFAULT_PATH.as_ptr() as *mut WCHAR, 0),
    hive_entry(HL_SAM_NAME.as_ptr() as *mut WCHAR,
               HL_MACHINE_PATH.as_ptr() as *mut WCHAR, HIVE_NOLAZYFLUSH),
    hive_entry(core::ptr::null_mut(), core::ptr::null_mut(), 0),
];

// ============================================================================
// CmClassName / CmTypeName / CmpMultifunctionTypes
// ============================================================================

// Wide-string buffers backing the UNICODE_STRINGs below.
const CN_SYSTEM: &[u16] = w!("System");
const CN_PROCESSOR: &[u16] = w!("Processor");
const CN_CACHE: &[u16] = w!("Cache");
const CN_ADAPTER: &[u16] = w!("Adapter");
const CN_CONTROLLER: &[u16] = w!("Controller");
const CN_PERIPHERAL: &[u16] = w!("Peripheral");
const CN_MEMORYCLASS: &[u16] = w!("MemoryClass");
const CN_UNDEFINED: &[u16] = w!("Undefined");

const fn ustr(buf: &'static [u16]) -> _UNICODE_STRING {
    // RTL_CONSTANT_STRING: byte length excluding the NUL terminator we emit.
    let chars = buf.len() - 1;
    _UNICODE_STRING {
        Length: (chars * 2) as u16,
        MaximumLength: ((chars + 1) * 2) as u16,
        Buffer: buf.as_ptr() as PWCHAR,
    }
}

#[no_mangle]
pub static mut CmClassName: [_UNICODE_STRING; MaximumClass + 1] = [
    ustr(CN_SYSTEM),
    ustr(CN_PROCESSOR),
    ustr(CN_CACHE),
    ustr(CN_ADAPTER),
    ustr(CN_CONTROLLER),
    ustr(CN_PERIPHERAL),
    ustr(CN_MEMORYCLASS),
    ustr(CN_UNDEFINED),
];

// CmTypeName backing strings — 42 entries.
const TN_SYSTEM: &[u16] = w!("System");
const TN_CENTRALPROCESSOR: &[u16] = w!("CentralProcessor");
const TN_FLOATINGPOINTPROCESSOR: &[u16] = w!("FloatingPointProcessor");
const TN_PRIMARYICACHE: &[u16] = w!("PrimaryICache");
const TN_PRIMARYDCACHE: &[u16] = w!("PrimaryDCache");
const TN_SECONDARYICACHE: &[u16] = w!("SecondaryICache");
const TN_SECONDARYDCACHE: &[u16] = w!("SecondaryDCache");
const TN_SECONDARYCACHE: &[u16] = w!("SecondaryCache");
const TN_EISAADAPTER: &[u16] = w!("EisaAdapter");
const TN_TCADAPTER: &[u16] = w!("TcAdapter");
const TN_SCSIADAPTER: &[u16] = w!("ScsiAdapter");
const TN_DTIADAPTER: &[u16] = w!("DtiAdapter");
const TN_MULTIFUNCTIONADAPTER: &[u16] = w!("MultifunctionAdapter");
const TN_DISKCONTROLLER: &[u16] = w!("DiskController");
const TN_TAPECONTROLLER: &[u16] = w!("TapeController");
const TN_CDROMCONTROLLER: &[u16] = w!("CdRomController");
const TN_WORMCONTROLLER: &[u16] = w!("WormController");
const TN_SERIALCONTROLLER: &[u16] = w!("SerialController");
const TN_NETWORKCONTROLLER: &[u16] = w!("NetworkController");
const TN_DISPLAYCONTROLLER: &[u16] = w!("DisplayController");
const TN_PARALLELCONTROLLER: &[u16] = w!("ParallelController");
const TN_POINTERCONTROLLER: &[u16] = w!("PointerController");
const TN_KEYBOARDCONTROLLER: &[u16] = w!("KeyboardController");
const TN_AUDIOCONTROLLER: &[u16] = w!("AudioController");
const TN_OTHERCONTROLLER: &[u16] = w!("OtherController");
const TN_DISKPERIPHERAL: &[u16] = w!("DiskPeripheral");
const TN_FLOPPYDISKPERIPHERAL: &[u16] = w!("FloppyDiskPeripheral");
const TN_TAPEPERIPHERAL: &[u16] = w!("TapePeripheral");
const TN_MODEMPERIPHERAL: &[u16] = w!("ModemPeripheral");
const TN_MONITORPERIPHERAL: &[u16] = w!("MonitorPeripheral");
const TN_PRINTERPERIPHERAL: &[u16] = w!("PrinterPeripheral");
const TN_POINTERPERIPHERAL: &[u16] = w!("PointerPeripheral");
const TN_KEYBOARDPERIPHERAL: &[u16] = w!("KeyboardPeripheral");
const TN_TERMINALPERIPHERAL: &[u16] = w!("TerminalPeripheral");
const TN_OTHERPERIPHERAL: &[u16] = w!("OtherPeripheral");
const TN_LINEPERIPHERAL: &[u16] = w!("LinePeripheral");
const TN_NETWORKPERIPHERAL: &[u16] = w!("NetworkPeripheral");
const TN_SYSTEMMEMORY: &[u16] = w!("SystemMemory");
const TN_DOCKINGINFORMATION: &[u16] = w!("DockingInformation");
const TN_REALMODEIRQROUTINGTABLE: &[u16] = w!("RealModeIrqRoutingTable");
const TN_REALMODEPCIENUMERATION: &[u16] = w!("RealModePCIEnumeration");
const TN_UNDEFINED: &[u16] = w!("Undefined");

#[no_mangle]
pub static mut CmTypeName: [_UNICODE_STRING; MaximumType + 1] = [
    ustr(TN_SYSTEM),
    ustr(TN_CENTRALPROCESSOR),
    ustr(TN_FLOATINGPOINTPROCESSOR),
    ustr(TN_PRIMARYICACHE),
    ustr(TN_PRIMARYDCACHE),
    ustr(TN_SECONDARYICACHE),
    ustr(TN_SECONDARYDCACHE),
    ustr(TN_SECONDARYCACHE),
    ustr(TN_EISAADAPTER),
    ustr(TN_TCADAPTER),
    ustr(TN_SCSIADAPTER),
    ustr(TN_DTIADAPTER),
    ustr(TN_MULTIFUNCTIONADAPTER),
    ustr(TN_DISKCONTROLLER),
    ustr(TN_TAPECONTROLLER),
    ustr(TN_CDROMCONTROLLER),
    ustr(TN_WORMCONTROLLER),
    ustr(TN_SERIALCONTROLLER),
    ustr(TN_NETWORKCONTROLLER),
    ustr(TN_DISPLAYCONTROLLER),
    ustr(TN_PARALLELCONTROLLER),
    ustr(TN_POINTERCONTROLLER),
    ustr(TN_KEYBOARDCONTROLLER),
    ustr(TN_AUDIOCONTROLLER),
    ustr(TN_OTHERCONTROLLER),
    ustr(TN_DISKPERIPHERAL),
    ustr(TN_FLOPPYDISKPERIPHERAL),
    ustr(TN_TAPEPERIPHERAL),
    ustr(TN_MODEMPERIPHERAL),
    ustr(TN_MONITORPERIPHERAL),
    ustr(TN_PRINTERPERIPHERAL),
    ustr(TN_POINTERPERIPHERAL),
    ustr(TN_KEYBOARDPERIPHERAL),
    ustr(TN_TERMINALPERIPHERAL),
    ustr(TN_OTHERPERIPHERAL),
    ustr(TN_LINEPERIPHERAL),
    ustr(TN_NETWORKPERIPHERAL),
    ustr(TN_SYSTEMMEMORY),
    ustr(TN_DOCKINGINFORMATION),
    ustr(TN_REALMODEIRQROUTINGTABLE),
    ustr(TN_REALMODEPCIENUMERATION),
    ustr(TN_UNDEFINED),
];

// CmpMultifunctionTypes — 9 entries with NULL terminator.
// _INTERFACE_TYPE values from bindgen (Isa=0, MicroChannel=1, PCIBus=5, …).
const IT_ISA: USHORT = 0;
const IT_MCA: USHORT = 1;
const IT_PCI: USHORT = 5;
const IT_VME: USHORT = 6;
const IT_PCMCIA: USHORT = 8;
const IT_CBUS: USHORT = 9;
const IT_MPIPI: USHORT = 11;
const IT_MPSA: USHORT = 12;
const IT_INTERNAL: USHORT = 1; // _INTERFACE_TYPE_Internal=1 (actually same enum value as MCA — yes, see C)

#[no_mangle]
pub static mut CmpMultifunctionTypes: [_CMP_MF_TYPE; 9] = [
    _CMP_MF_TYPE { Identifier: b"ISA\0".as_ptr() as *mut i8, InterfaceType: IT_ISA, Count: 0 },
    _CMP_MF_TYPE { Identifier: b"MCA\0".as_ptr() as *mut i8, InterfaceType: IT_MCA, Count: 0 },
    _CMP_MF_TYPE { Identifier: b"PCI\0".as_ptr() as *mut i8, InterfaceType: IT_PCI, Count: 0 },
    _CMP_MF_TYPE { Identifier: b"VME\0".as_ptr() as *mut i8, InterfaceType: IT_VME, Count: 0 },
    _CMP_MF_TYPE { Identifier: b"PCMCIA\0".as_ptr() as *mut i8, InterfaceType: IT_PCMCIA, Count: 0 },
    _CMP_MF_TYPE { Identifier: b"CBUS\0".as_ptr() as *mut i8, InterfaceType: IT_CBUS, Count: 0 },
    _CMP_MF_TYPE { Identifier: b"MPIPI\0".as_ptr() as *mut i8, InterfaceType: IT_MPIPI, Count: 0 },
    _CMP_MF_TYPE { Identifier: b"MPSA\0".as_ptr() as *mut i8, InterfaceType: IT_MPSA, Count: 0 },
    _CMP_MF_TYPE { Identifier: core::ptr::null_mut(), InterfaceType: IT_INTERNAL, Count: 0 },
];

// ============================================================================
// extern declarations for variables NOT in Rust (still C-side)
// ============================================================================

extern "C" {
    static mut MmConsumedPoolPercentage: ULONG;
}

// ============================================================================
// CmControlVector — registry-key → variable-pointer dispatch table.
//
// Phase B in progress: ob entries (5) live in ob/{obinit,devicemap,obname}.rs
// and are concatenated below. Remaining entries (244) are still local. As more
// subsystems get chopped, more entries will move out and SLICES grows.
// ============================================================================

// Local entries — entries that reference cmdata-owned globals or are still
// awaiting Phase B chop. Excludes the NULL terminator (added separately by
// the assembler below).
const CMDATA_LOCAL_CV: [_CM_SYSTEM_CONTROL_VECTOR; 243] = unsafe { [
    cv!("LSA", "AuditBaseDirectories", core::ptr::addr_of_mut!(DummyData)),
    cv!("LSA", "AuditBaseObjects", core::ptr::addr_of_mut!(DummyData)),
    cv!("LSA\\audit", "ProcessAccessesToAudit", core::ptr::addr_of_mut!(DummyData)),
    cv!("TimeZoneInformation", "ActiveTimeBias", core::ptr::addr_of_mut!(DummyData)),
    cv!("TimeZoneInformation", "Bias", core::ptr::addr_of_mut!(DummyData)),
    cv!("TimeZoneInformation", "RealTimeIsUniversal", core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager", "GlobalFlag", core::ptr::addr_of_mut!(CmNtGlobalFlag)),
    cv!("Session Manager\\Memory Management", "PagedPoolQuota",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "NonPagedPoolQuota",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "PagingFileQuota",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "AllocationPreference",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "DynamicMemory",
        core::ptr::addr_of_mut!(crate::ARM3::pfnlist::MmDynamicPfn)),
    cv!("Session Manager\\Memory Management", "Mirroring",
        core::ptr::addr_of_mut!(crate::ARM3::pfnlist::MmMirroring)),
    cv!("Session Manager\\Memory Management", "SystemViewSize",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "SessionImageSize",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "SessionPoolSize",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "PoolUsageMaximum",
        core::ptr::addr_of_mut!(MmConsumedPoolPercentage)),
    cv!("Session Manager\\Memory Management", "MapAllocationFragment",
        core::ptr::addr_of_mut!(crate::ARM3::mminit::MmAllocationFragment)),
    cv!("Session Manager\\Memory Management", "PagedPoolSize",
        core::ptr::addr_of_mut!(crate::ARM3::mminit::MmSizeOfPagedPoolInBytes)),
    cv!("Session Manager\\Memory Management", "NonPagedPoolSize",
        core::ptr::addr_of_mut!(crate::ARM3::mminit::MmSizeOfNonPagedPoolInBytes)),
    cv!("Session Manager\\Memory Management", "NonPagedPoolMaximumPercent",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "LargeSystemCache",
        core::ptr::addr_of_mut!(crate::ARM3::mminit::MmLargeSystemCache)),
    cv!("Session Manager\\Memory Management", "LargeStackSize",
        core::ptr::addr_of_mut!(crate::ARM3::helpers::MmLargeStackSize)),
    cv!("Session Manager\\Memory Management", "SystemPages",
        core::ptr::addr_of_mut!(crate::ARM3::mminit::MmNumberOfSystemPtes)),
    cv!("Session Manager\\Memory Management", "LowMemoryThreshold",
        core::ptr::addr_of_mut!(crate::ARM3::mminit::MmLowMemoryThreshold)),
    cv!("Session Manager\\Memory Management", "HighMemoryThreshold",
        core::ptr::addr_of_mut!(crate::ARM3::helpers::MmHighMemoryThreshold)),
    cv!("Session Manager\\Memory Management", "DisablePagingExecutive",
        core::ptr::addr_of_mut!(crate::mm::mminit::MmDisablePagingExecutive)),
    cv!("Session Manager\\Memory Management", "ModifiedPageLife",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "SecondLevelDataCache",
        core::ptr::addr_of_mut!(crate::ARM3::mminit::MmSecondaryColors)),
    cv!("Session Manager\\Memory Management", "ClearPageFileAtShutdown",
        core::ptr::addr_of_mut!(crate::mm::pagefile::MmZeroPageFile)),
    cv!("Session Manager\\Memory Management", "PoolTagSmallTableSize",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "PoolTagBigTableSize",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "PoolTag",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "PoolTagOverruns",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "SnapUnloads",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "ProtectNonPagedPool",
        core::ptr::addr_of_mut!(crate::ARM3::helpers::MmProtectFreedNonPagedPool)),
    cv!("Session Manager\\Memory Management", "TrackLockedPages",
        core::ptr::addr_of_mut!(crate::ARM3::mdlsup::MmTrackLockedPages)),
    cv!("Session Manager\\Memory Management", "TrackPtes",
        core::ptr::addr_of_mut!(crate::ARM3::mdlsup::MmTrackPtes)),
    cv!("Session Manager\\Memory Management", "VerifyDrivers",
        core::ptr::addr_of_mut!(crate::ARM3::drvmgmt::MmVerifyDriverBuffer),
        core::ptr::addr_of_mut!(crate::ARM3::drvmgmt::MmVerifyDriverBufferLength),
        core::ptr::addr_of_mut!(crate::ARM3::drvmgmt::MmVerifyDriverBufferType)),
    cv!("Session Manager\\Memory Management", "VerifyDriverLevel",
        core::ptr::addr_of_mut!(crate::ARM3::drvmgmt::MmVerifyDriverLevel)),
    cv!("Session Manager\\Memory Management", "VerifyMode",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "LargePageMinimum",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "EnforceWriteProtection",
        core::ptr::addr_of_mut!(crate::ARM3::sysldr::MmEnforceWriteProtection)),
    cv!("Session Manager\\Memory Management", "MakeLowMemory",
        core::ptr::addr_of_mut!(crate::ARM3::sysldr::MmMakeLowMemory)),
    cv!("Session Manager\\Memory Management", "WriteWatch",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "MinimumStackCommitInBytes",
        core::ptr::addr_of_mut!(crate::ARM3::mminit::MmMinimumStackCommitInBytes)),
    cv!("Session Manager\\Executive", "AdditionalCriticalWorkerThreads",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Executive", "AdditionalDelayedWorkerThreads",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Executive", "PriorityQuantumMatrix",
        core::ptr::addr_of_mut!(DummyData),
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Kernel", "DpcQueueDepth",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Kernel", "MinimumDpcRate",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Kernel", "AdjustDpcThreshold",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Kernel", "IdealDpcRate",
        core::ptr::addr_of_mut!(DummyData)),
    // ObUnsecureGlobalNames + ObCaseInsensitive moved to ob/obname.rs::OBNAME_CONTROL_VECTOR
    cv!("Session Manager\\I/O System", "CountOperations",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\I/O System", "LargeIrpStackLocations",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\I/O System", "IoVerifierLevel",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager", "ResourceTimeoutCount",
        core::ptr::addr_of_mut!(crate::ex::resource::ExpResourceTimeoutCount)),
    cv!("Session Manager", "CriticalSectionTimeout",
        core::ptr::addr_of_mut!(crate::ARM3::mminit::MmCritsectTimeoutSeconds)),
    cv!("Session Manager", "HeapSegmentReserve",
        core::ptr::addr_of_mut!(crate::ARM3::helpers::MmHeapSegmentReserve)),
    cv!("Session Manager", "HeapSegmentCommit",
        core::ptr::addr_of_mut!(crate::ARM3::helpers::MmHeapSegmentCommit)),
    cv!("Session Manager", "HeapDeCommitTotalFreeThreshold",
        core::ptr::addr_of_mut!(crate::ARM3::mminit::MmHeapDeCommitTotalFreeThreshold)),
    cv!("Session Manager", "HeapDeCommitFreeBlockThreshold",
        core::ptr::addr_of_mut!(crate::ARM3::mminit::MmHeapDeCommitFreeBlockThreshold)),
    cv!("ProductOptions", "ProductType",
        core::ptr::addr_of_mut!(crate::ARM3::mminit::MmProductType)),
    cv!("Terminal Server", "TSEnabled", core::ptr::addr_of_mut!(DummyData)),
    cv!("Terminal Server", "TSAppCompat", core::ptr::addr_of_mut!(DummyData)),
    cv!("ProductOptions", "ProductSuite",
        core::ptr::addr_of_mut!(CmSuiteBuffer),
        core::ptr::addr_of_mut!(CmSuiteBufferLength),
        core::ptr::addr_of_mut!(CmSuiteBufferType)),
    cv!("Windows", "CSDVersion",
        core::ptr::addr_of_mut!(crate::ex::init::CmNtCSDVersion)),
    cv!("Windows", "CSDReleaseType",
        core::ptr::addr_of_mut!(crate::ex::init::CmNtCSDReleaseType)),
    cv!("Nls\\Language", "Default",
        core::ptr::addr_of_mut!(CmDefaultLanguageId),
        core::ptr::addr_of_mut!(CmDefaultLanguageIdLength),
        core::ptr::addr_of_mut!(CmDefaultLanguageIdType)),
    cv!("Nls\\Language", "InstallLanguage",
        core::ptr::addr_of_mut!(CmInstallUILanguageId),
        core::ptr::addr_of_mut!(CmInstallUILanguageIdLength),
        core::ptr::addr_of_mut!(CmInstallUILanguageIdType)),
    cv!("\0\0", "RegistrySizeLimit",
        core::ptr::addr_of_mut!(DummyData),
        core::ptr::addr_of_mut!(DummyData),
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Configuration Manager", "SelfHealingEnabled",
        core::ptr::addr_of_mut!(CmSelfHeal)),
    cv!("Session Manager\\Configuration Manager", "RegistryLazyFlushInterval",
        core::ptr::addr_of_mut!(crate::config::cmlazy::CmpLazyFlushIntervalInSeconds)),
    cv!("Session Manager\\Configuration Manager", "RegistryLazyFlushHiveCount",
        core::ptr::addr_of_mut!(crate::config::cmlazy::CmpLazyFlushHiveCount)),
    cv!("Session Manager\\Configuration Manager", "DelayCloseSize",
        core::ptr::addr_of_mut!(crate::config::cmdelay::CmpDelayedCloseSize)),
    cv!("Session Manager\\Configuration Manager", "VolatileBoot",
        core::ptr::addr_of_mut!(CmpVolatileBoot)),
    cv!("Session Manager", "ForceNpxEmulation", core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager", "PowerPolicySimulate", core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Executive", "MaxTimeSeparationBeforeCorrect",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Windows", "ShutdownTime",
        core::ptr::addr_of_mut!(DummyData),
        core::ptr::addr_of_mut!(DummyData)),
    cv!("PriorityControl", "Win32PrioritySeparation",
        core::ptr::addr_of_mut!(crate::ps::process::PsRawPrioritySeparation)),
    cv!("Session Manager", "EnableTimerWatchdog", core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager", "Debugger Retries",
        core::ptr::addr_of_mut!(crate::kd64::kddata::KdpContext.KdpDefaultRetries)),

    // === KD debug print filter masks (156 entries — see kd64::kddata::Kd_*_Mask) ===
    kd_filter!("WIN2000", crate::kd64::kddata::Kd_WIN2000_Mask),
    kd_filter!("SYSTEM", crate::kd64::kddata::Kd_SYSTEM_Mask),
    kd_filter!("SMSS", crate::kd64::kddata::Kd_SMSS_Mask),
    kd_filter!("SETUP", crate::kd64::kddata::Kd_SETUP_Mask),
    kd_filter!("NTFS", crate::kd64::kddata::Kd_NTFS_Mask),
    kd_filter!("FSTUB", crate::kd64::kddata::Kd_FSTUB_Mask),
    kd_filter!("CRASHDUMP", crate::kd64::kddata::Kd_CRASHDUMP_Mask),
    kd_filter!("CDAUDIO", crate::kd64::kddata::Kd_CDAUDIO_Mask),
    kd_filter!("CDROM", crate::kd64::kddata::Kd_CDROM_Mask),
    kd_filter!("CLASSPNP", crate::kd64::kddata::Kd_CLASSPNP_Mask),
    kd_filter!("DISK", crate::kd64::kddata::Kd_DISK_Mask),
    kd_filter!("REDBOOK", crate::kd64::kddata::Kd_REDBOOK_Mask),
    kd_filter!("STORPROP", crate::kd64::kddata::Kd_STORPROP_Mask),
    kd_filter!("SCSIPORT", crate::kd64::kddata::Kd_SCSIPORT_Mask),
    kd_filter!("SCSIMINIPORT", crate::kd64::kddata::Kd_SCSIMINIPORT_Mask),
    kd_filter!("CONFIG", crate::kd64::kddata::Kd_CONFIG_Mask),
    kd_filter!("I8042PRT", crate::kd64::kddata::Kd_I8042PRT_Mask),
    kd_filter!("SERMOUSE", crate::kd64::kddata::Kd_SERMOUSE_Mask),
    kd_filter!("LSERMOUS", crate::kd64::kddata::Kd_LSERMOUS_Mask),
    kd_filter!("KBDHID", crate::kd64::kddata::Kd_KBDHID_Mask),
    kd_filter!("MOUHID", crate::kd64::kddata::Kd_MOUHID_Mask),
    kd_filter!("KBDCLASS", crate::kd64::kddata::Kd_KBDCLASS_Mask),
    kd_filter!("MOUCLASS", crate::kd64::kddata::Kd_MOUCLASS_Mask),
    kd_filter!("TWOTRACK", crate::kd64::kddata::Kd_TWOTRACK_Mask),
    kd_filter!("WMILIB", crate::kd64::kddata::Kd_WMILIB_Mask),
    kd_filter!("ACPI", crate::kd64::kddata::Kd_ACPI_Mask),
    kd_filter!("AMLI", crate::kd64::kddata::Kd_AMLI_Mask),
    kd_filter!("HALIA64", crate::kd64::kddata::Kd_HALIA64_Mask),
    kd_filter!("VIDEO", crate::kd64::kddata::Kd_VIDEO_Mask),
    kd_filter!("SVCHOST", crate::kd64::kddata::Kd_SVCHOST_Mask),
    kd_filter!("VIDEOPRT", crate::kd64::kddata::Kd_VIDEOPRT_Mask),
    kd_filter!("TCPIP", crate::kd64::kddata::Kd_TCPIP_Mask),
    kd_filter!("DMSYNTH", crate::kd64::kddata::Kd_DMSYNTH_Mask),
    kd_filter!("NTOSPNP", crate::kd64::kddata::Kd_NTOSPNP_Mask),
    kd_filter!("FASTFAT", crate::kd64::kddata::Kd_FASTFAT_Mask),
    kd_filter!("SAMSS", crate::kd64::kddata::Kd_SAMSS_Mask),
    kd_filter!("PNPMGR", crate::kd64::kddata::Kd_PNPMGR_Mask),
    kd_filter!("NETAPI", crate::kd64::kddata::Kd_NETAPI_Mask),
    kd_filter!("SCSERVER", crate::kd64::kddata::Kd_SCSERVER_Mask),
    kd_filter!("SCCLIENT", crate::kd64::kddata::Kd_SCCLIENT_Mask),
    kd_filter!("SERIAL", crate::kd64::kddata::Kd_SERIAL_Mask),
    kd_filter!("SERENUM", crate::kd64::kddata::Kd_SERENUM_Mask),
    kd_filter!("UHCD", crate::kd64::kddata::Kd_UHCD_Mask),
    kd_filter!("RPCPROXY", crate::kd64::kddata::Kd_RPCPROXY_Mask),
    kd_filter!("AUTOCHK", crate::kd64::kddata::Kd_AUTOCHK_Mask),
    kd_filter!("DCOMSS", crate::kd64::kddata::Kd_DCOMSS_Mask),
    kd_filter!("UNIMODEM", crate::kd64::kddata::Kd_UNIMODEM_Mask),
    kd_filter!("SIS", crate::kd64::kddata::Kd_SIS_Mask),
    kd_filter!("FLTMGR", crate::kd64::kddata::Kd_FLTMGR_Mask),
    kd_filter!("WMICORE", crate::kd64::kddata::Kd_WMICORE_Mask),
    kd_filter!("BURNENG", crate::kd64::kddata::Kd_BURNENG_Mask),
    kd_filter!("IMAPI", crate::kd64::kddata::Kd_IMAPI_Mask),
    kd_filter!("SXS", crate::kd64::kddata::Kd_SXS_Mask),
    kd_filter!("FUSION", crate::kd64::kddata::Kd_FUSION_Mask),
    kd_filter!("IDLETASK", crate::kd64::kddata::Kd_IDLETASK_Mask),
    kd_filter!("SOFTPCI", crate::kd64::kddata::Kd_SOFTPCI_Mask),
    kd_filter!("TAPE", crate::kd64::kddata::Kd_TAPE_Mask),
    kd_filter!("MCHGR", crate::kd64::kddata::Kd_MCHGR_Mask),
    kd_filter!("IDEP", crate::kd64::kddata::Kd_IDEP_Mask),
    kd_filter!("PCIIDE", crate::kd64::kddata::Kd_PCIIDE_Mask),
    kd_filter!("FLOPPY", crate::kd64::kddata::Kd_FLOPPY_Mask),
    kd_filter!("FDC", crate::kd64::kddata::Kd_FDC_Mask),
    kd_filter!("TERMSRV", crate::kd64::kddata::Kd_TERMSRV_Mask),
    kd_filter!("W32TIME", crate::kd64::kddata::Kd_W32TIME_Mask),
    kd_filter!("PREFETCHER", crate::kd64::kddata::Kd_PREFETCHER_Mask),
    kd_filter!("RSFILTER", crate::kd64::kddata::Kd_RSFILTER_Mask),
    kd_filter!("FCPORT", crate::kd64::kddata::Kd_FCPORT_Mask),
    kd_filter!("PCI", crate::kd64::kddata::Kd_PCI_Mask),
    kd_filter!("DMIO", crate::kd64::kddata::Kd_DMIO_Mask),
    kd_filter!("DMCONFIG", crate::kd64::kddata::Kd_DMCONFIG_Mask),
    kd_filter!("DMADMIN", crate::kd64::kddata::Kd_DMADMIN_Mask),
    kd_filter!("WSOCKTRANSPORT", crate::kd64::kddata::Kd_WSOCKTRANSPORT_Mask),
    kd_filter!("VSS", crate::kd64::kddata::Kd_VSS_Mask),
    kd_filter!("PNPMEM", crate::kd64::kddata::Kd_PNPMEM_Mask),
    kd_filter!("PROCESSOR", crate::kd64::kddata::Kd_PROCESSOR_Mask),
    kd_filter!("DMSERVER", crate::kd64::kddata::Kd_DMSERVER_Mask),
    kd_filter!("SR", crate::kd64::kddata::Kd_SR_Mask),
    kd_filter!("INFINIBAND", crate::kd64::kddata::Kd_INFINIBAND_Mask),
    kd_filter!("IHVDRIVER", crate::kd64::kddata::Kd_IHVDRIVER_Mask),
    kd_filter!("IHVVIDEO", crate::kd64::kddata::Kd_IHVVIDEO_Mask),
    kd_filter!("IHVAUDIO", crate::kd64::kddata::Kd_IHVAUDIO_Mask),
    kd_filter!("IHVNETWORK", crate::kd64::kddata::Kd_IHVNETWORK_Mask),
    kd_filter!("IHVSTREAMING", crate::kd64::kddata::Kd_IHVSTREAMING_Mask),
    kd_filter!("IHVBUS", crate::kd64::kddata::Kd_IHVBUS_Mask),
    kd_filter!("HPS", crate::kd64::kddata::Kd_HPS_Mask),
    kd_filter!("RTLTHREADPOOL", crate::kd64::kddata::Kd_RTLTHREADPOOL_Mask),
    kd_filter!("LDR", crate::kd64::kddata::Kd_LDR_Mask),
    kd_filter!("TCPIP6", crate::kd64::kddata::Kd_TCPIP6_Mask),
    kd_filter!("ISAPNP", crate::kd64::kddata::Kd_ISAPNP_Mask),
    kd_filter!("SHPC", crate::kd64::kddata::Kd_SHPC_Mask),
    kd_filter!("STORPORT", crate::kd64::kddata::Kd_STORPORT_Mask),
    kd_filter!("STORMINIPORT", crate::kd64::kddata::Kd_STORMINIPORT_Mask),
    kd_filter!("PRINTSPOOLER", crate::kd64::kddata::Kd_PRINTSPOOLER_Mask),
    kd_filter!("VSSDYNDISK", crate::kd64::kddata::Kd_VSSDYNDISK_Mask),
    kd_filter!("VERIFIER", crate::kd64::kddata::Kd_VERIFIER_Mask),
    kd_filter!("VDS", crate::kd64::kddata::Kd_VDS_Mask),
    kd_filter!("VDSBAS", crate::kd64::kddata::Kd_VDSBAS_Mask),
    kd_filter!("VDSDYN", crate::kd64::kddata::Kd_VDSDYN_Mask),
    kd_filter!("VDSDYNDR", crate::kd64::kddata::Kd_VDSDYNDR_Mask),
    kd_filter!("VDSLDR", crate::kd64::kddata::Kd_VDSLDR_Mask),
    kd_filter!("VDSUTIL", crate::kd64::kddata::Kd_VDSUTIL_Mask),
    kd_filter!("DFRGIFC", crate::kd64::kddata::Kd_DFRGIFC_Mask),
    kd_filter!("DEFAULT", crate::kd64::kddata::Kd_DEFAULT_Mask),
    kd_filter!("MM", crate::kd64::kddata::Kd_MM_Mask),
    kd_filter!("DFSC", crate::kd64::kddata::Kd_DFSC_Mask),
    kd_filter!("WOW64", crate::kd64::kddata::Kd_WOW64_Mask),
    kd_filter!("ALPC", crate::kd64::kddata::Kd_ALPC_Mask),
    kd_filter!("WDI", crate::kd64::kddata::Kd_WDI_Mask),
    kd_filter!("PERFLIB", crate::kd64::kddata::Kd_PERFLIB_Mask),
    kd_filter!("KTM", crate::kd64::kddata::Kd_KTM_Mask),
    kd_filter!("IOSTRESS", crate::kd64::kddata::Kd_IOSTRESS_Mask),
    kd_filter!("HEAP", crate::kd64::kddata::Kd_HEAP_Mask),
    kd_filter!("WHEA", crate::kd64::kddata::Kd_WHEA_Mask),
    kd_filter!("USERGDI", crate::kd64::kddata::Kd_USERGDI_Mask),
    kd_filter!("MMCSS", crate::kd64::kddata::Kd_MMCSS_Mask),
    kd_filter!("TPM", crate::kd64::kddata::Kd_TPM_Mask),
    kd_filter!("THREADORDER", crate::kd64::kddata::Kd_THREADORDER_Mask),
    kd_filter!("ENVIRON", crate::kd64::kddata::Kd_ENVIRON_Mask),
    kd_filter!("EMS", crate::kd64::kddata::Kd_EMS_Mask),
    kd_filter!("WDT", crate::kd64::kddata::Kd_WDT_Mask),
    kd_filter!("FVEVOL", crate::kd64::kddata::Kd_FVEVOL_Mask),
    kd_filter!("NDIS", crate::kd64::kddata::Kd_NDIS_Mask),
    kd_filter!("NVCTRACE", crate::kd64::kddata::Kd_NVCTRACE_Mask),
    kd_filter!("LUAFV", crate::kd64::kddata::Kd_LUAFV_Mask),
    kd_filter!("APPCOMPAT", crate::kd64::kddata::Kd_APPCOMPAT_Mask),
    kd_filter!("USBSTOR", crate::kd64::kddata::Kd_USBSTOR_Mask),
    kd_filter!("SBP2PORT", crate::kd64::kddata::Kd_SBP2PORT_Mask),
    kd_filter!("COVERAGE", crate::kd64::kddata::Kd_COVERAGE_Mask),
    kd_filter!("CACHEMGR", crate::kd64::kddata::Kd_CACHEMGR_Mask),
    kd_filter!("MOUNTMGR", crate::kd64::kddata::Kd_MOUNTMGR_Mask),
    kd_filter!("CFR", crate::kd64::kddata::Kd_CFR_Mask),
    kd_filter!("TXF", crate::kd64::kddata::Kd_TXF_Mask),
    kd_filter!("KSECDD", crate::kd64::kddata::Kd_KSECDD_Mask),
    kd_filter!("FLTREGRESS", crate::kd64::kddata::Kd_FLTREGRESS_Mask),
    kd_filter!("MPIO", crate::kd64::kddata::Kd_MPIO_Mask),
    kd_filter!("MSDSM", crate::kd64::kddata::Kd_MSDSM_Mask),
    kd_filter!("UDFS", crate::kd64::kddata::Kd_UDFS_Mask),
    kd_filter!("PSHED", crate::kd64::kddata::Kd_PSHED_Mask),
    kd_filter!("STORVSP", crate::kd64::kddata::Kd_STORVSP_Mask),
    kd_filter!("LSASS", crate::kd64::kddata::Kd_LSASS_Mask),
    kd_filter!("SSPICLI", crate::kd64::kddata::Kd_SSPICLI_Mask),
    kd_filter!("CNG", crate::kd64::kddata::Kd_CNG_Mask),
    kd_filter!("EXFAT", crate::kd64::kddata::Kd_EXFAT_Mask),
    kd_filter!("FILETRACE", crate::kd64::kddata::Kd_FILETRACE_Mask),
    kd_filter!("XSAVE", crate::kd64::kddata::Kd_XSAVE_Mask),
    kd_filter!("SE", crate::kd64::kddata::Kd_SE_Mask),
    kd_filter!("DRIVEEXTENDER", crate::kd64::kddata::Kd_DRIVEEXTENDER_Mask),
    kd_filter!("POWER", crate::kd64::kddata::Kd_POWER_Mask),
    kd_filter!("CRASHDUMPXHCI", crate::kd64::kddata::Kd_CRASHDUMPXHCI_Mask),
    kd_filter!("GPIO", crate::kd64::kddata::Kd_GPIO_Mask),
    kd_filter!("REFS", crate::kd64::kddata::Kd_REFS_Mask),
    kd_filter!("WER", crate::kd64::kddata::Kd_WER_Mask),
    kd_filter!("CAPIMG", crate::kd64::kddata::Kd_CAPIMG_Mask),
    kd_filter!("VPCI", crate::kd64::kddata::Kd_VPCI_Mask),
    kd_filter!("STORAGECLASSMEMORY", crate::kd64::kddata::Kd_STORAGECLASSMEMORY_Mask),
    kd_filter!("FSLIB", crate::kd64::kddata::Kd_FSLIB_Mask),

    // === Trailing entries (after the debug-print block) ===
    cv!("WMI", "MaxEventSize", core::ptr::addr_of_mut!(DummyData)),
    cv!("WMI\\Trace", "UsePerformanceClock", core::ptr::addr_of_mut!(DummyData)),
    cv!("WMI\\Trace", "TraceAlignment", core::ptr::addr_of_mut!(DummyData)),
    cv!("CrashControl", "AutoReboot",
        core::ptr::addr_of_mut!(crate::iomgr::iomgr::IopAutoReboot)),
] };

// ============================================================================
// CmControlVector — assembled at compile time from sub-vectors + cmdata-local.
//
// Each subsystem owns its own `pub static FOO_CONTROL_VECTOR` slice (Phase B
// chop). cmdata.rs assembles them all + its remaining local entries + a NULL
// terminator into the single `CmControlVector` symbol that
// `CmGetSystemControlValues` consumes.
//
// To add a subsystem: define `pub static FOO_CONTROL_VECTOR: [_CM_..; N]` in
// the owning file (using `crate::cv!` macro), add `&FOO_CONTROL_VECTOR` to
// SLICES below, and remove the entries from CMDATA_LOCAL_CV. Adjust
// CMDATA_LOCAL_CV's array size accordingly. CV_TOTAL recomputes automatically.
// ============================================================================

const NULL_ENTRY: _CM_SYSTEM_CONTROL_VECTOR = _CM_SYSTEM_CONTROL_VECTOR {
    KeyPath: core::ptr::null_mut(),
    ValueName: core::ptr::null_mut(),
    Buffer: core::ptr::null_mut(),
    BufferLength: core::ptr::null_mut(),
    Type: core::ptr::null_mut(),
};

const TERMINATOR: [_CM_SYSTEM_CONTROL_VECTOR; 1] = [NULL_ENTRY];

const SLICES: &[&[_CM_SYSTEM_CONTROL_VECTOR]] = &[
    &crate::ob::obinit::OBINIT_CONTROL_VECTOR.0,
    &crate::ob::devicemap::DEVICEMAP_CONTROL_VECTOR.0,
    &crate::ob::obname::OBNAME_CONTROL_VECTOR.0,
    &CMDATA_LOCAL_CV,
    &TERMINATOR,
];

const CV_TOTAL: usize = {
    let mut t = 0;
    let mut i = 0;
    while i < SLICES.len() {
        t += SLICES[i].len();
        i += 1;
    }
    t
};

#[no_mangle]
pub static mut CmControlVector: [_CM_SYSTEM_CONTROL_VECTOR; CV_TOTAL] = {
    let mut out = [NULL_ENTRY; CV_TOTAL];
    let mut dst = 0;
    let mut s = 0;
    while s < SLICES.len() {
        let mut i = 0;
        while i < SLICES[s].len() {
            out[dst] = SLICES[s][i];
            i += 1;
            dst += 1;
        }
        s += 1;
    }
    out
};
