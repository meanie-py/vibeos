//! [correctness-review 2026-06-07 · sonnet] Reviewed vs sdk/lib/cmlib/hiveinit.c. Risk: HIGH.
//! Behavior: 5 real bugs found, code is largely non-functional for HINIT_CREATE.
//!
//! Findings:
//!   BUG-1 (CRITICAL) HvInitialize match @ line 237: HINIT_CREATE/HINIT_MEMORY/HINIT_FLAT/
//!     HINIT_FILE are not defined in any imported namespace (ntkrnl_sys::constants, glue::*).
//!     Rust silently treats each as a catch-all binding pattern; the first arm (HINIT_CREATE)
//!     swallows every OperationType value and calls HvpCreateHive(Hive, HiveData, FileName).
//!     HINIT_MEMORY_INPLACE, HINIT_MAPFILE, and the default→STATUS_INVALID_PARAMETER arms from
//!     the C switch are entirely absent. Fix: add HINIT_* constants to glue.rs (values 0–5 from
//!     hivedata.h) and add the missing arms; or use numeric literals. See memory
//!     feedback_undefined_const_in_match_silent_binding.md.
//!
//!   BUG-2 (HIGH) HvpCreateHive @ line 55: body is C's HvpInitializeMemoryHive, not C's
//!     HvpCreateHive. C's HvpCreateHive (2 params: RegistryHive, FileName) allocates a fresh
//!     zeroed BaseBlock, fills in Signature/Major/Minor/Type/Format/Cluster/RootCell/Length/
//!     Sequence[12]/TimeStamp/CheckSum/BootType, loops FreeDisplay[24]=HCELL_NIL for both
//!     storage types, and calls HvpInitFileName. That entire body is absent. What is here
//!     (3-param signature taking ChunkBase, validating header, iterating existing bins) is the
//!     body of C's HvpInitializeMemoryHive. Fix: implement the correct fresh-hive creation body;
//!     separately, the extern "system" declaration of HvpInitializeMemoryHive (which calls into
//!     C) can remain for the HINIT_MEMORY and HvLoadHive paths.
//!
//!   BUG-3 (HIGH) HIVE_NOLAZYFLUSH @ line 211: `const HIVE_NOLAZYFLUSH: u32 = 0x10000`.
//!     C hivedata.h defines this as 2. The Rust value (0x10000 = 65536) masks a completely
//!     wrong bit in HiveFlags (line 232), so the lazy-flush flag is never stripped and callers
//!     that set HIVE_NOLAZYFLUSH=2 will find the flag preserved in hive->HiveFlags. Fix:
//!     move the correct value (2) to glue.rs and import it.
//!
//!   BUG-4 (MED) HvInitialize HINIT_FILE recovery path @ line 264 (non-amd64 only): two
//!     statements from C are missing after RtlClearAllBits: (a) `Hive->DirtyCount = 0` and
//!     (b) `Status = STATUS_SUCCESS` (the "masquerade" the C comment documents). Without (b),
//!     STATUS_REGISTRY_RECOVERED leaks out as the return value of HvInitialize, which the
//!     C code notes is not a failure but not STATUS_SUCCESS either and can confuse callers.
//!
//!   BUG-5 (MED) ABI: HvpCreateHive (line 55), HvpInitializeFlatHive (line 116), and HvFree
//!     (line 273) are `extern "C"` but must be `extern "system"` to match CMAPI=NTAPI=stdcall
//!     on i386. HvFree is a public API called by external code. On amd64/arm64 there is no
//!     observable difference; on i386 the wrong decoration (_HvFree vs _HvFree@4) breaks the
//!     cohab link. lib.rs already documents this rule. Fix: change all three to extern "system".
//!
//! Unverified: HvpGetHiveHeader/HvpRecoverDataFromLog/HvpFreeHiveBins/HvpAllocBaseBlockAligned/
//!   HvpInitFileName are declared extern and not translated here — those call into C unchanged.
//!   HvLoadHive control flow (RecoverData/RecoverHeader) looks structurally correct (RESULT enum
//!   is local and properly ordered). ChunkSize source in HvpCreateHive (ChunkBase->Length not
//!   ChunkBase->Length+HBASE_BLOCK) is inherited from the body-confusion bug and not reviewed
//!   independently.
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CELL_DATA, CM_KEY_NODE, CmIsSelfHealEnabled, FILE_STANDARD_INFORMATION, HBASE_BLOCK, HBIN, HCELL_INDEX, HHIVE, HMAP_ENTRY, PCELL_DATA, PCMHIVE, PCUNICODE_STRING, PFILE_FLUSH_ROUTINE, PFILE_READ_ROUTINE, PFILE_SET_SIZE_ROUTINE, PFILE_WRITE_ROUTINE, PFREE_ROUTINE, PHBASE_BLOCK, PHBIN, PHHIVE, PLARGE_INTEGER, HSTORAGE_TYPE_Stable as Stable, TimeStamp, HSTORAGE_TYPE_Volatile as Volatile, _HHIVE, _IO_STATUS_BLOCK, _UNICODE_STRING, LARGE_INTEGER, PALLOCATE_ROUTINE, _FILE_INFORMATION_CLASS};
use crate::glue::*;
use crate::hivewrt::HvSyncHiveFromRecover;
use core::ffi::c_void;

// Pointer-width integer matching ULONG_PTR/SIZE_T on all arches
#[cfg(target_pointer_width = "32")]
type PtrInt = u32;
#[cfg(target_pointer_width = "64")]
type PtrInt = u64;

// File-local enum from hiveinit.c:17 (`typedef enum _RESULT`). Variant order is
// preserved EXACTLY so discriminants match the C (NotHive=0 .. SelfHeal=6) —
// the mechanical pass had dropped `HiveSuccess` and mis-ordered the rest.

// hivedata.h constants needed by HvpCreateHive
const HV_HBLOCK_SIGNATURE: u32 = 0x6667_6572; // "regf"
const HBASE_FORMAT_MEMORY: u32 = 1;
const HSYS_MAJOR: u32 = 1;

// HINIT operation types from hivedata.h — must match C exactly
const HINIT_CREATE: u32 = 0;
const HINIT_MEMORY: u32 = 1;
const HINIT_FILE: u32 = 2;
const HINIT_MEMORY_INPLACE: u32 = 3;
const HINIT_FLAT: u32 = 4;
const HINIT_MAPFILE: u32 = 5;

#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
enum RESULT {
    NotHive,
    Fail,
    NoMemory,
    HiveSuccess,
    RecoverHeader,
    RecoverData,
    SelfHeal,
}

extern "system" {
    fn HvpInitializeMemoryHive(Hive: PHHIVE, ChunkBase: PHBASE_BLOCK, FileName: PCUNICODE_STRING) -> NTSTATUS;
}
unsafe extern "C" {
    fn DPRINT1(fmt: *const i8, ...);
    unsafe fn HvpAllocBaseBlockAligned(Hive: *mut HHIVE, Paged: i32, Tag: u32) -> *mut HBASE_BLOCK;
    unsafe fn HvpInitFileName(BaseBlock: *mut HBASE_BLOCK, FileName: *const _UNICODE_STRING) ;
    fn HvpHiveHeaderChecksum(BaseBlock: *mut HBASE_BLOCK) -> u32;
    fn HvpCreateHiveFreeCellList(Hive: *mut HHIVE) -> i32;
    fn HvpFreeHiveBins(Hive: *mut HHIVE);
}
unsafe extern "C" {
    fn ZwQueryInformationFile( FileHandle: HANDLE, IoStatusBlock: *mut _IO_STATUS_BLOCK, FileInformation: *mut core::ffi::c_void, Length: u32, FileInformationClass: _FILE_INFORMATION_CLASS, ) -> NTSTATUS;
    fn HvpVerifyHiveHeader(BaseBlock: *mut HBASE_BLOCK, FileType: u32) -> u8;
}

extern "system" {
    fn HvpGetCellData(Hive: PHHIVE, Cell: HCELL_INDEX) -> *mut core::ffi::c_void;
    fn HvpGetHiveHeader(Hive: PHHIVE, BaseBlock: *mut PHBASE_BLOCK, TimeStamp: *mut LARGE_INTEGER) -> RESULT;
    fn HvpRecoverDataFromLog(Hive: PHHIVE, BaseBlock: PHBASE_BLOCK) -> RESULT;
}
#[no_mangle]
pub unsafe extern "system" fn HvpCreateHive(
    RegistryHive: PHHIVE,
    FileName: PCUNICODE_STRING,
) -> NTSTATUS {
    let BaseBlock: PHBASE_BLOCK;
    let mut Index: u32;

    /* Allocate the base block */
    BaseBlock = HvpAllocBaseBlockAligned(RegistryHive, 0i32, TAG_CM);
    if BaseBlock.is_null() {
        return STATUS_NO_MEMORY;
    }

    /* Clear it */
    RtlZeroMemory(BaseBlock as *mut _, (*RegistryHive).BaseBlockAlloc as usize);

    (*BaseBlock).Signature = HV_HBLOCK_SIGNATURE;
    (*BaseBlock).Major = HSYS_MAJOR;
    (*BaseBlock).Minor = HSYS_MINOR;
    (*BaseBlock).Type = HFILE_TYPE_PRIMARY;
    (*BaseBlock).Format = HBASE_FORMAT_MEMORY;
    (*BaseBlock).Cluster = 1;
    (*BaseBlock).RootCell = HCELL_NIL;
    (*BaseBlock).Length = 0;
    (*BaseBlock).Sequence1 = 1;
    (*BaseBlock).Sequence2 = 1;
    (*BaseBlock).TimeStamp.QuadPart = 0u64 as i64;

    /*
     * No need to compute the checksum since
     * the hive resides only in memory so far.
     */
    (*BaseBlock).CheckSum = 0;

    /* Set default boot type */
    (*BaseBlock).BootType = HBOOT_TYPE_REGULAR;

    /* Setup hive data */
    (*RegistryHive).BaseBlock = BaseBlock;
    (*RegistryHive).Version = (*BaseBlock).Minor; // == HSYS_MINOR

    Index = 0;
    while Index < 24 {
        (*RegistryHive).Storage[Stable as usize].FreeDisplay[Index as usize] = HCELL_NIL;
        (*RegistryHive).Storage[Volatile as usize].FreeDisplay[Index as usize] = HCELL_NIL;
        Index += 1;
    }

    HvpInitFileName(BaseBlock, FileName);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn HvpInitializeFlatHive(
    Hive: *mut HHIVE,
    ChunkBase: *mut HBASE_BLOCK,
) -> i32 {
    if HvpVerifyHiveHeader(ChunkBase, HFILE_TYPE_PRIMARY) == 0 {
        return STATUS_REGISTRY_CORRUPT;
    } // Setup hive data (*Hive).BaseBlock = ChunkBase;
    (*Hive).Version = (*ChunkBase).Minor;
    (*Hive).Flat = 1;
    (*Hive).ReadOnly = 1;
    (*Hive).StorageTypeCount = 1;
    // Set default boot type
    (*ChunkBase).BootType = HBOOT_TYPE_REGULAR;
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn HvLoadHive(
    Hive: PHHIVE,
    FileName: PCUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Success: BOOLEAN;
    let mut BaseBlock: PHBASE_BLOCK = core::ptr::null_mut();
    let mut Result: RESULT = RESULT::NotHive;
    #[cfg(not(target_arch = "x86_64"))]
    let mut Result2: RESULT = RESULT::NotHive;
    let mut TimeStamp: LARGE_INTEGER = core::mem::zeroed();
    let mut Offset: u32 = 0;
    let mut HiveData: *mut core::ffi::c_void;
    let mut FileSize: u32;
    let mut HiveSelfHeal: BOOLEAN = 0;
    /* Get the hive header */ Result = HvpGetHiveHeader(Hive, &mut BaseBlock, &mut TimeStamp);
    match Result { RESULT::NoMemory => { /* Fail */ // DPRINT removed return STATUS_INSUFFICIENT_RESOURCES;
    } RESULT::Fail => { /* Fail */ return STATUS_REGISTRY_CORRUPT;
    } RESULT::NotHive => { /* Fail */ // DPRINT removed return STATUS_NOT_REGISTRY_FILE;
    } RESULT::HiveSuccess => { /* Success, continue */ }
    RESULT::SelfHeal => { HiveSelfHeal = 1; }
    RESULT::RecoverData => { /* break; */ } RESULT::RecoverHeader => { /* FIXME: See the comment above (near HvpQueryHiveSize) */ #[cfg(target_arch = "x86_64")] { return STATUS_REGISTRY_CORRUPT;
    } #[cfg(not(target_arch = "x86_64"))] { /* The header needs to be recovered so do it */ // DPRINT removed Result2 = HvpRecoverHeaderFromLog(Hive, &mut TimeStamp, &mut BaseBlock);
    if Result2 == RESULT::NoMemory { // DPRINT removed return STATUS_INSUFFICIENT_RESOURCES;
    } /* Did we fail? */ if Result2 == RESULT::Fail { // DPRINT removed return STATUS_REGISTRY_CORRUPT;
    } /* Did we trigger the self-heal mode? */ if Result2 == RESULT::SelfHeal { HiveSelfHeal = 1;
    } /* Now recover the data */ Result2 = HvpRecoverDataFromLog(Hive, BaseBlock);
    if Result2 == RESULT::Fail { // DPRINT removed return STATUS_REGISTRY_CORRUPT;
    } /* Tag the boot as self heal if we haven't done it before */ if (Result2 == RESULT::SelfHeal) && (HiveSelfHeal == 0) { HiveSelfHeal = 1;
    } /* break; */ } } } /* Set the boot type */ (*BaseBlock).BootType = if HiveSelfHeal != 0 { HBOOT_TYPE_SELF_HEAL } else { HBOOT_TYPE_REGULAR };
    /* Setup hive data */ (*Hive).BaseBlock = BaseBlock;
    (*Hive).Version = (*BaseBlock).Minor;
    /* Allocate a buffer large enough to hold the hive */ FileSize = HBLOCK_SIZE + (*BaseBlock).Length; // == sizeof(HBASE_BLOCK) + BaseBlock->Length;
    HiveData = ((*Hive).Allocate.unwrap())(FileSize as PtrInt, 1u8, TAG_CM); // TRUE -> 1
    if HiveData.is_null() {
        ((*Hive).Free.unwrap())(BaseBlock as *mut core::ffi::c_void, (*Hive).BaseBlockAlloc);
        // DPRINT removed
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    // HACK: Now read the whole hive
    Success = ((*Hive).FileRead.unwrap())(Hive, HFILE_TYPE_PRIMARY, &mut Offset, HiveData, FileSize as PtrInt);
    if Success == 0 {
        // FALSE // DPRINT removed
        ((*Hive).Free.unwrap())(HiveData, FileSize);
        ((*Hive).Free.unwrap())(BaseBlock as *mut core::ffi::c_void, (*Hive).BaseBlockAlloc);
        return STATUS_NOT_REGISTRY_FILE;
    }
    // HACK (FIXME): Free our base block...
    ((*Hive).Free.unwrap())(BaseBlock as *mut core::ffi::c_void, (*Hive).BaseBlockAlloc);
    Status = HvpInitializeMemoryHive(Hive, HiveData as *mut HBASE_BLOCK, FileName);
    if Status < 0 {
        // DPRINT removed
        ((*Hive).Free.unwrap())(HiveData, FileSize);
        return Status;
    }
    // If we recovered, tell the caller
    if Result == RESULT::RecoverHeader {
        STATUS_REGISTRY_RECOVERED
    } else {
        Status
    }
}
#[no_mangle]
pub unsafe extern "system" fn HvInitialize(
    RegistryHive: *mut HHIVE,
    OperationType: u32,
    HiveFlags: u32,
    FileType: u32,
    HiveData: *mut core::ffi::c_void,
    Allocate: PALLOCATE_ROUTINE,
    Free: PFREE_ROUTINE,
    FileSetSize: PFILE_SET_SIZE_ROUTINE,
    FileWrite: PFILE_WRITE_ROUTINE,
    FileRead: PFILE_READ_ROUTINE,
    FileFlush: PFILE_FLUSH_ROUTINE,
    Cluster: u32,
    FileName: *const _UNICODE_STRING,
) -> NTSTATUS {
    // Local constants
    const HIVE_NOLAZYFLUSH: u32 = 2;
    const NTDDI_VERSION: u32 = 0x05010000; // assumed XP
    let Hive: *mut HHIVE = RegistryHive;
    let mut Status: NTSTATUS;
    // RtlZeroMemory(Hive, sizeof(HHIVE));
    core::ptr::write_bytes(Hive, 0, core::mem::size_of::<HHIVE>());
    (*Hive).Signature = HV_HHIVE_SIGNATURE;
    (*Hive).Allocate = Allocate;
    (*Hive).Free = Free;
    (*Hive).FileSetSize = FileSetSize;
    (*Hive).FileWrite = FileWrite;
    (*Hive).FileRead = FileRead;
    (*Hive).FileFlush = FileFlush;
    (*Hive).RefreshCount = 0;
    (*Hive).StorageTypeCount = HTYPE_COUNT;
    (*Hive).Cluster = Cluster;
    (*Hive).BaseBlockAlloc = core::mem::size_of::<HBASE_BLOCK>() as u32;
    (*Hive).Version = HSYS_MINOR;
    // #if (NTDDI_VERSION < NTDDI_VISTA)
    if NTDDI_VERSION < NTDDI_VISTA { (*Hive).Log = (FileType == HFILE_TYPE_LOG) as u8;
    (*Hive).Alternate = (FileType == HFILE_TYPE_ALTERNATE) as u8;
    } (*Hive).HiveFlags = HiveFlags & !HIVE_NOLAZYFLUSH;
    // TODO: The CellRoutines point to different callbacks depending on the OperationType.
    (*Hive).GetCellRoutine = core::mem::transmute(HvpGetCellData as *const ());
    (*Hive).ReleaseCellRoutine = None; // NULL
    Status = match OperationType {
        HINIT_CREATE => {
            Status = HvpCreateHive(Hive, FileName);
            Status
        }
        HINIT_MEMORY => {
            Status = HvpInitializeMemoryHive(Hive, HiveData as *mut HBASE_BLOCK, FileName);
            Status
        }
        HINIT_FLAT => {
            Status = HvpInitializeFlatHive(Hive, HiveData as *mut HBASE_BLOCK);
            Status
        }
        HINIT_FILE => {
            Status = HvLoadHive(Hive, FileName);
    if Status != STATUS_SUCCESS && Status != STATUS_REGISTRY_RECOVERED { // DPRINT1("Registry hive couldn't be initialized, it's corrupt (hive 0x%p)\n", Hive);
    // comment placeholder for DPRINT1
    return Status;
    }
    // #if !defined(_M_AMD64)
    #[cfg(not(target_arch = "x86_64"))]
    {
        if Status == STATUS_REGISTRY_RECOVERED {
            if HvSyncHiveFromRecover(Hive) == 0 {
                // DPRINT1("Fail to write healthy data back to hive\n");
                return STATUS_REGISTRY_IO_FAILED;
            }
            // FIXME: clear log and reset size to 0
            RtlClearAllBits(core::ptr::addr_of_mut!((*Hive).DirtyVector) as *mut RTL_BITMAP);
            (*Hive).DirtyCount = 0;
            Status = STATUS_SUCCESS;
        }
    }
            Status
        }
        HINIT_MEMORY_INPLACE => STATUS_NOT_IMPLEMENTED,
        HINIT_MAPFILE => STATUS_NOT_IMPLEMENTED,
        _ => STATUS_INVALID_PARAMETER,
    };
    Status
}
#[no_mangle]
pub unsafe extern "system" fn HvFree(RegistryHive: *mut HHIVE) {
    // if (!RegistryHive->ReadOnly)
    if (*RegistryHive).ReadOnly == 0 {
        // Release hive bitmap
        if !(*RegistryHive).DirtyVector.Buffer.is_null() {
            ((*RegistryHive).Free.unwrap())((*RegistryHive).DirtyVector.Buffer as *mut c_void, 0);
        }
        HvpFreeHiveBins(RegistryHive);
        // Free the BaseBlock
        if !(*RegistryHive).BaseBlock.is_null() {
            ((*RegistryHive).Free.unwrap())((*RegistryHive).BaseBlock as *mut c_void, (*RegistryHive).BaseBlockAlloc);
            (*RegistryHive).BaseBlock = core::ptr::null_mut();
        }
    }
}