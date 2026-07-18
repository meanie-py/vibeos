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

// `SubVector`, `cv!`, and `w!` live in `ntkrnl-sys::cm_control` so other crates
// (e.g. the pool crate) can declare their own `FOO_CONTROL_VECTOR` slices
// without depending on ntoskrnl-rust. See cm_control.rs.
use ntkrnl_sys::{cv, w, SubVector};

// IopAutoReboot: registry CrashControl\AutoReboot flag. Relocated here from iomgr
// (crate-split Phase 1b): the const CmControlVector table below must reference its
// address as an IN-CRATE relocation — const-eval cannot point at an extern
// (severed-crate) static. iomgr had no runtime use of it; it's read by ke/bug.rs
// (extern decl) and populated by the registry via CmControlVector.
#[no_mangle]
pub static mut IopAutoReboot: ULONG = 0;

// iomgr's registry slice (was IOMGR_CONTROL_VECTOR in iomgr.rs). Local const so the
// const CmControlVector concatenation can reference it. crate-split Phase 1b.
const IOMGR_CONTROL_VECTOR: SubVector<1> = SubVector({
    [cv!("CrashControl", "AutoReboot", core::ptr::addr_of_mut!(IopAutoReboot))]
});

// (CM_DEBUG_PRINT_FILTER entries moved to kd64::kddata::KD_FILTER_CONTROL_VECTOR)

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
// (No remaining extern "C" blocks — every variable referenced by CmControlVector
// is now owned by a Rust module. MmConsumedPoolPercentage moved to the pool
// crate; its CmControlVector entry lives in pool/src/globals.rs as
// POOL_CONTROL_VECTOR and is concatenated into SLICES below.)
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
const CMDATA_LOCAL_CV: [_CM_SYSTEM_CONTROL_VECTOR; 50] = { [
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
    // Mm-owned entries moved to ARM3/{pfnlist,mminit,helpers,mdlsup,drvmgmt,sysldr}.rs
    // and mm/{mminit,pagefile}.rs. PoolUsageMaximum -> MmConsumedPoolPercentage moved
    // to crate::ex::pool::globals::POOL_CONTROL_VECTOR.
    cv!("Session Manager\\Memory Management", "SystemViewSize",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "SessionImageSize",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "SessionPoolSize",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "NonPagedPoolMaximumPercent",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "ModifiedPageLife",
        core::ptr::addr_of_mut!(DummyData)),
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
    cv!("Session Manager\\Memory Management", "VerifyMode",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "LargePageMinimum",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Memory Management", "WriteWatch",
        core::ptr::addr_of_mut!(DummyData)),
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
    // ResourceTimeoutCount moved to ex/resource.rs::RESOURCE_CONTROL_VECTOR
    // CriticalSectionTimeout, HeapSegment*, HeapDeCommit* moved to ARM3/{mminit,helpers}.rs
    // ProductType moved to ARM3/mminit.rs::MMINIT_CONTROL_VECTOR
    cv!("Terminal Server", "TSEnabled", core::ptr::addr_of_mut!(DummyData)),
    cv!("Terminal Server", "TSAppCompat", core::ptr::addr_of_mut!(DummyData)),
    cv!("ProductOptions", "ProductSuite",
        core::ptr::addr_of_mut!(CmSuiteBuffer),
        core::ptr::addr_of_mut!(CmSuiteBufferLength),
        core::ptr::addr_of_mut!(CmSuiteBufferType)),
    // CSDVersion + CSDReleaseType moved to ex/init.rs::INIT_CONTROL_VECTOR
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
    // RegistryLazyFlush* moved to config/cmlazy.rs::CMLAZY_CONTROL_VECTOR
    // DelayCloseSize moved to config/cmdelay.rs::CMDELAY_CONTROL_VECTOR
    cv!("Session Manager\\Configuration Manager", "VolatileBoot",
        core::ptr::addr_of_mut!(CmpVolatileBoot)),
    cv!("Session Manager", "ForceNpxEmulation", core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager", "PowerPolicySimulate", core::ptr::addr_of_mut!(DummyData)),
    cv!("Session Manager\\Executive", "MaxTimeSeparationBeforeCorrect",
        core::ptr::addr_of_mut!(DummyData)),
    cv!("Windows", "ShutdownTime",
        core::ptr::addr_of_mut!(DummyData),
        core::ptr::addr_of_mut!(DummyData)),
    // Win32PrioritySeparation moved to ps/process.rs::PROCESS_CONTROL_VECTOR
    cv!("Session Manager", "EnableTimerWatchdog", core::ptr::addr_of_mut!(DummyData)),
    // Debugger Retries moved to kd64/kddata.rs::KDDATA_TUNABLES_CV

    // === Trailing entries (after the debug-print block) ===
    cv!("WMI", "MaxEventSize", core::ptr::addr_of_mut!(DummyData)),
    cv!("WMI\\Trace", "UsePerformanceClock", core::ptr::addr_of_mut!(DummyData)),
    cv!("WMI\\Trace", "TraceAlignment", core::ptr::addr_of_mut!(DummyData)),
    // CrashControl/AutoReboot moved to iomgr/iomgr.rs::IOMGR_CONTROL_VECTOR
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
// the owning file (using `ntkrnl_sys::cv!` macro), add `&FOO_CONTROL_VECTOR` to
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

// The per-subsystem control vectors that live in OTHER crates (ob/mm/ARM3/ex/ps/kd64,
// which stay in ntoskrnl-rust once config is severed) — referenced as LINK SYMBOLS,
// read at runtime by cm_build_control_vector() below. Each is `#[no_mangle] pub static
// FOO_CONTROL_VECTOR: SubVector<N>` in its owner. The two Mm-init vectors were renamed
// (ARM3_/MM_) at their defs to disambiguate the old MMINIT_CONTROL_VECTOR name clash.
//
// IMPORTANT: the `SubVector<N>` sizes here MUST match each owner's def — an extern
// static carries no length, so a stale N would over-/under-read at assembly. They're
// stable registry-table definitions; the boot-time bound check below catches an
// undersize. (Const-eval can't take the address of these, which is exactly why
// CmControlVector is assembled at runtime, not compile time — the config crate-split.)
extern "C" {
    static OBINIT_CONTROL_VECTOR: SubVector<2>;
    static DEVICEMAP_CONTROL_VECTOR: SubVector<1>;
    static OBNAME_CONTROL_VECTOR: SubVector<2>;
    static KD_FILTER_CONTROL_VECTOR: SubVector<156>;
    static KDDATA_TUNABLES_CV: SubVector<1>;
    static PFNLIST_CONTROL_VECTOR: SubVector<2>;
    static ARM3_MMINIT_CONTROL_VECTOR: SubVector<12>;
    static HELPERS_CONTROL_VECTOR: SubVector<5>;
    static MDLSUP_CONTROL_VECTOR: SubVector<2>;
    static DRVMGMT_CONTROL_VECTOR: SubVector<2>;
    static SYSLDR_CONTROL_VECTOR: SubVector<2>;
    static MM_MMINIT_CONTROL_VECTOR: SubVector<1>;
    static PAGEFILE_CONTROL_VECTOR: SubVector<1>;
    static INIT_CONTROL_VECTOR: SubVector<2>;
    static RESOURCE_CONTROL_VECTOR: SubVector<1>;
    static PROCESS_CONTROL_VECTOR: SubVector<1>;
    static POOL_CONTROL_VECTOR: SubVector<1>;
}

// Total entry count: 194 (the 17 cross-crate CVs: 2+1+2+156+1+2+12+5+2+2+2+1+1+2+1+1+1)
// + 1 IOMGR + 2 CMLAZY + 1 CMDELAY + 50 CMDATA_LOCAL + 1 NULL terminator = 249.
const CV_TOTAL: usize = 249;

// CmControlVector is RUNTIME-assembled (was a compile-time const concatenation).
// WHY: the crate-split severs config into config-rust, and const-eval cannot
// reference an extern (other-crate) static's address. The slices below aggregate
// control vectors owned by ob/mm/ARM3/ex/ps/kd64 (staying in ntoskrnl-rust) AND by
// config (moving out), so no single crate can const-build this table. Assembling at
// boot from runtime reads sidesteps that entirely (runtime extern access is legal).
// Consumed once, at ex/init.rs's CmGetSystemControlValues call, which runs
// cm_build_control_vector() immediately before. Zero-init = NULL_ENTRY, so the tail
// (incl. the terminator slot) is already null; the fill copies the same entries the
// old const did, in the same order, leaving the last slot NULL as terminator.
#[no_mangle]
pub static mut CmControlVector: [_CM_SYSTEM_CONTROL_VECTOR; CV_TOTAL] = [NULL_ENTRY; CV_TOTAL];

/// Populate `CmControlVector` at boot from the per-subsystem control vectors. MUST run
/// before the single consumer (ex/init.rs → CmGetSystemControlValues). Idempotent
/// (a re-run rewrites the same bytes). Runtime reads of the sources let them live in
/// a severed crate (config-rust) — the crux of the config split.
///
/// `#[no_mangle] extern "C"` so ex/init.rs (ntoskrnl-rust) calls it across the crate
/// boundary as a link symbol.
#[no_mangle]
pub unsafe extern "C" fn cm_build_control_vector() {
    // Cross-crate CVs (link symbols) first, then config-local ones, in the exact order
    // the old const SLICES used. CMDATA_LOCAL_CV last; the NULL terminator is implicit
    // (zero-init tail). Keep this order stable — it's the registry apply order.
    let slices: [&[_CM_SYSTEM_CONTROL_VECTOR]; 21] = [
        &OBINIT_CONTROL_VECTOR.0,
        &DEVICEMAP_CONTROL_VECTOR.0,
        &OBNAME_CONTROL_VECTOR.0,
        &KD_FILTER_CONTROL_VECTOR.0,
        &KDDATA_TUNABLES_CV.0,
        &PFNLIST_CONTROL_VECTOR.0,
        &ARM3_MMINIT_CONTROL_VECTOR.0,
        &HELPERS_CONTROL_VECTOR.0,
        &MDLSUP_CONTROL_VECTOR.0,
        &DRVMGMT_CONTROL_VECTOR.0,
        &SYSLDR_CONTROL_VECTOR.0,
        &MM_MMINIT_CONTROL_VECTOR.0,
        &PAGEFILE_CONTROL_VECTOR.0,
        &INIT_CONTROL_VECTOR.0,
        &RESOURCE_CONTROL_VECTOR.0,
        &PROCESS_CONTROL_VECTOR.0,
        &POOL_CONTROL_VECTOR.0,
        &IOMGR_CONTROL_VECTOR.0,
        &crate::cmlazy::CMLAZY_CONTROL_VECTOR.0,
        &crate::cmdelay::CMDELAY_CONTROL_VECTOR.0,
        &CMDATA_LOCAL_CV,
    ];
    let cv = core::ptr::addr_of_mut!(CmControlVector) as *mut _CM_SYSTEM_CONTROL_VECTOR;
    let mut dst = 0usize;
    let mut s = 0;
    while s < slices.len() {
        let sl = slices[s];
        let mut i = 0;
        while i < sl.len() {
            // Fail loud if a stale extern SubVector<N> pushed us past the table
            // (leaves one slot for the NULL terminator).
            assert!(dst + 1 < CV_TOTAL);
            *cv.add(dst) = sl[i];
            i += 1;
            dst += 1;
        }
        s += 1;
    }
}
