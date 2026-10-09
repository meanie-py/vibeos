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
use ntkrnl_sys::ddk::{CELL_DATA, CM_KEY_NODE, CmIsSelfHealEnabled, HBASE_BLOCK, HBIN, HCELL_INDEX, HHIVE, HMAP_ENTRY, PCELL_DATA, PCMHIVE, PCUNICODE_STRING, PFILE_FLUSH_ROUTINE, PFILE_READ_ROUTINE, PFILE_SET_SIZE_ROUTINE, PFILE_WRITE_ROUTINE, PFREE_ROUTINE, PHBASE_BLOCK, PHBIN, PHHIVE, PLARGE_INTEGER, HSTORAGE_TYPE_Stable as Stable, TimeStamp, HSTORAGE_TYPE_Volatile as Volatile, _HHIVE, _UNICODE_STRING, LARGE_INTEGER, PALLOCATE_ROUTINE};
#[cfg(not(feature = "bldr"))]
use ntkrnl_sys::ddk::{FILE_STANDARD_INFORMATION, ZwQueryInformationFile, _FILE_INFORMATION_CLASS_FileStandardInformation, _IO_STATUS_BLOCK};
use crate::glue::*;
use crate::hivewrt::HvSyncHiveFromRecover;
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(format: *const i8, ...) -> u32;
}

// Pointer-width integer matching ULONG_PTR/SIZE_T on all arches.
// (Kept for reference; bodies use `usize` directly to match SIZE_T/ULONG_PTR.)
#[cfg(target_pointer_width = "32")]
#[allow(dead_code)]
type PtrInt = u32;
#[cfg(target_pointer_width = "64")]
#[allow(dead_code)]
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

// ── hivedata.h constants used by the translated bodies below ────────────────
const HSECTOR_SIZE: u32 = 0x200;
const HIVE_FILENAME_MAXLEN: usize = 31;
const HV_LOG_DIRTY_BLOCK: u8 = 0xFF;
const HV_LOG_DIRTY_SIGNATURE: u32 = 0x5452_4944; // "DIRT"
const HBOOT_BOOT_RECOVERED_BY_ALTERNATE_HIVE: u32 = 2;
// HV_LOG_HEADER_SIZE = FIELD_OFFSET(HBASE_BLOCK, Reserved2)
const HV_LOG_HEADER_SIZE: u32 = core::mem::offset_of!(HBASE_BLOCK, Reserved2) as u32;

// Real cmlib functions defined IN this crate (correct ABI). HvpHiveHeaderChecksum
// lives in hivesum.rs; HvpCreateHiveFreeCellList lives in its owning module.
// HvpRecoverHeaderFromLog / HvpQueryHiveSize are defined below in THIS file (the
// non-amd64 self-heal pair), so they're no longer declared extern here.
// Declared "system" to match CMAPI=NTAPI=stdcall on i386.
extern "system" {
    fn HvpHiveHeaderChecksum(BaseBlock: PHBASE_BLOCK) -> u32;
    fn HvpCreateHiveFreeCellList(Hive: PHHIVE) -> NTSTATUS;
    fn HvpGetCellData(Hive: PHHIVE, Cell: HCELL_INDEX) -> *mut core::ffi::c_void;
}

/// Validates the base block header of a registry file (hive or log). C: HvpVerifyHiveHeader.
#[no_mangle]
pub unsafe extern "system" fn HvpVerifyHiveHeader(
    BaseBlock: PHBASE_BLOCK,
    FileType: u32,
) -> BOOLEAN {
    if (*BaseBlock).Signature != HV_HBLOCK_SIGNATURE
        || (*BaseBlock).Major != HSYS_MAJOR
        || (*BaseBlock).Minor < HSYS_MINOR
        || (*BaseBlock).Type != FileType
        || (*BaseBlock).Format != HBASE_FORMAT_MEMORY
        || (*BaseBlock).Cluster != 1
        || (*BaseBlock).Sequence1 != (*BaseBlock).Sequence2
        || HvpHiveHeaderChecksum(BaseBlock) != (*BaseBlock).CheckSum
    {
        DbgPrint(b"Verify Hive Header failed:\n\0".as_ptr() as *const i8);
        DbgPrint(
            b"    Signature: 0x%x, expected 0x%x; Major: 0x%x, expected 0x%x\n\0".as_ptr() as *const i8,
            (*BaseBlock).Signature, HV_HBLOCK_SIGNATURE, (*BaseBlock).Major, HSYS_MAJOR,
        );
        DbgPrint(
            b"    Minor: 0x%x expected to be >= 0x%x; Type: 0x%x, expected 0x%x\n\0".as_ptr() as *const i8,
            (*BaseBlock).Minor, HSYS_MINOR, (*BaseBlock).Type, FileType,
        );
        DbgPrint(
            b"    Format: 0x%x, expected 0x%x; Cluster: 0x%x, expected 1\n\0".as_ptr() as *const i8,
            (*BaseBlock).Format, HBASE_FORMAT_MEMORY, (*BaseBlock).Cluster,
        );
        DbgPrint(
            b"    Sequence: 0x%x, expected 0x%x; Checksum: 0x%x, expected 0x%x\n\0".as_ptr() as *const i8,
            (*BaseBlock).Sequence1, (*BaseBlock).Sequence2,
            HvpHiveHeaderChecksum(BaseBlock), (*BaseBlock).CheckSum,
        );
        return 0; // FALSE
    }

    1 // TRUE
}

/// Frees all the bins within storage space associated with a hive descriptor.
/// C: HvpFreeHiveBins.
#[no_mangle]
pub unsafe extern "system" fn HvpFreeHiveBins(Hive: PHHIVE) {
    let free = (*Hive).Free.unwrap();

    let mut Storage: u32 = 0;
    while Storage < (*Hive).StorageTypeCount {
        let mut Bin: PHBIN = core::ptr::null_mut();
        let storage = &mut (*Hive).Storage[Storage as usize];
        let mut i: u32 = 0;
        while i < storage.Length {
            // storage.BlockList is a PHMAP_ENTRY array; index via pointer arith.
            let entry = storage.BlockList.add(i as usize);
            if (*entry).BinAddress == 0 {
                i += 1;
                continue;
            }
            if (*entry).BinAddress != Bin as _ {
                Bin = (*entry).BinAddress as PHBIN;
                free((*entry).BinAddress as *mut c_void, 0);
            }
            (*entry).BinAddress = 0;
            (*entry).BlockAddress = 0;
            i += 1;
        }

        if storage.Length != 0 {
            free(storage.BlockList as *mut c_void, 0);
        }
        Storage += 1;
    }
}

/// Allocates a cluster-aligned hive base header block. C: static __inline
/// HvpAllocBaseBlockAligned — private here (no #[no_mangle]).
unsafe fn HvpAllocBaseBlockAligned(
    Hive: PHHIVE,
    Paged: BOOLEAN,
    Tag: u32,
) -> PHBASE_BLOCK {
    let allocate = (*Hive).Allocate.unwrap();
    let free = (*Hive).Free.unwrap();

    // ASSERT(sizeof(HBASE_BLOCK) >= (HSECTOR_SIZE * Hive->Cluster));
    ASSERT(core::mem::size_of::<HBASE_BLOCK>() as u32 >= HSECTOR_SIZE * (*Hive).Cluster);

    /* Allocate the buffer */
    let mut BaseBlock: PHBASE_BLOCK =
        allocate((*Hive).BaseBlockAlloc as _, Paged, Tag) as PHBASE_BLOCK;
    if BaseBlock.is_null() {
        return core::ptr::null_mut();
    }

    /* Check for, and enforce, alignment */
    let Alignment: usize = (*Hive).Cluster as usize * HSECTOR_SIZE as usize - 1;
    if (BaseBlock as usize) & Alignment != 0 {
        /* Free the old header and reallocate a new one, always paged */
        free(BaseBlock as *mut c_void, (*Hive).BaseBlockAlloc);
        BaseBlock = allocate(PAGE_SIZE as _, 1u8 /* TRUE */, Tag) as PHBASE_BLOCK;
        if BaseBlock.is_null() {
            return core::ptr::null_mut();
        }

        (*Hive).BaseBlockAlloc = PAGE_SIZE as u32;
    }

    BaseBlock
}

/// Initializes a NULL-terminated Unicode hive file name of a hive header by
/// copying the last 31 characters of the hive file name. C: static
/// HvpInitFileName — private here (no #[no_mangle]).
unsafe fn HvpInitFileName(
    BaseBlock: PHBASE_BLOCK,
    FileName: PCUNICODE_STRING,
) {
    /* Always NULL-initialize */
    RtlZeroMemory(
        core::ptr::addr_of_mut!((*BaseBlock).FileName) as *mut c_void,
        (HIVE_FILENAME_MAXLEN + 1) * core::mem::size_of::<u16>(),
    );

    /* Copy the 31 last characters of the hive file name if any */
    if FileName.is_null() {
        return;
    }

    let fn_length = (*FileName).Length as usize; // bytes
    let Offset: usize;
    let Length: usize; // bytes
    if fn_length / core::mem::size_of::<u16>() <= HIVE_FILENAME_MAXLEN {
        Offset = 0;
        Length = fn_length;
    } else {
        Offset = fn_length / core::mem::size_of::<u16>() - HIVE_FILENAME_MAXLEN;
        Length = HIVE_FILENAME_MAXLEN * core::mem::size_of::<u16>();
    }

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*BaseBlock).FileName) as *mut c_void,
        (*FileName).Buffer.add(Offset) as *const c_void,
        Length,
    );
}

/// Initializes a hive descriptor from an already loaded registry hive in
/// memory; data is copied and prepared for read/write access. C:
/// HvpInitializeMemoryHive.
#[no_mangle]
pub unsafe extern "system" fn HvpInitializeMemoryHive(
    Hive: PHHIVE,
    ChunkBase: PHBASE_BLOCK,
    FileName: PCUNICODE_STRING,
) -> NTSTATUS {
    let allocate = (*Hive).Allocate.unwrap();
    let free = (*Hive).Free.unwrap();

    let ChunkSize: usize = (*ChunkBase).Length as usize;

    if ChunkSize < core::mem::size_of::<HBASE_BLOCK>()
        || HvpVerifyHiveHeader(ChunkBase, HFILE_TYPE_PRIMARY) == 0
    {
        return STATUS_REGISTRY_CORRUPT;
    }

    /* Allocate the base block */
    (*Hive).BaseBlock = HvpAllocBaseBlockAligned(Hive, 0 /* FALSE */, TAG_CM);
    if (*Hive).BaseBlock.is_null() {
        return STATUS_NO_MEMORY;
    }

    RtlCopyMemory(
        (*Hive).BaseBlock as *mut c_void,
        ChunkBase as *const c_void,
        core::mem::size_of::<HBASE_BLOCK>(),
    );
    /* Setup hive data */
    (*Hive).Version = (*ChunkBase).Minor;

    /*
     * Build a block list from the in-memory chunk and copy the data as we go.
     */
    (*Hive).Storage[Stable as usize].Length = (ChunkSize / HBLOCK_SIZE as usize) as u32;
    (*Hive).Storage[Stable as usize].BlockList = allocate(
        ((*Hive).Storage[Stable as usize].Length as usize
            * core::mem::size_of::<HMAP_ENTRY>()) as _,
        0, /* FALSE */
        TAG_CM,
    ) as *mut HMAP_ENTRY;
    if (*Hive).Storage[Stable as usize].BlockList.is_null() {
        free((*Hive).BaseBlock as *mut c_void, (*Hive).BaseBlockAlloc);
        return STATUS_NO_MEMORY;
    }

    let block_list = (*Hive).Storage[Stable as usize].BlockList;

    let mut BlockIndex: usize = 0;
    while BlockIndex < (*Hive).Storage[Stable as usize].Length as usize {
        let Bin: PHBIN = ((ChunkBase as usize)
            + (BlockIndex + 1) * HBLOCK_SIZE as usize)
            as PHBIN;
        if (*Bin).Signature != HV_HBIN_SIGNATURE
            || ((*Bin).Size % HBLOCK_SIZE) != 0
            || ((*Bin).FileOffset / HBLOCK_SIZE) != BlockIndex as u32
        {
            /*
             * Bin is toast but luckily either the signature, size or offset is
             * out of order. Self-heal it if enabled, otherwise bail.
             */
            if CmIsSelfHealEnabled(0 /* FALSE */) == 0 {
                free((*Hive).Storage[Stable as usize].BlockList as *mut c_void, 0);
                free((*Hive).BaseBlock as *mut c_void, (*Hive).BaseBlockAlloc);
                return STATUS_REGISTRY_CORRUPT;
            }

            /* Fix this bin */
            (*Bin).Signature = HV_HBIN_SIGNATURE;
            (*Bin).Size = HBLOCK_SIZE;
            (*Bin).FileOffset = BlockIndex as u32 * HBLOCK_SIZE;
            (*ChunkBase).BootType |= HBOOT_TYPE_SELF_HEAL;
        }

        let NewBin: PHBIN = allocate((*Bin).Size as _, 1 /* TRUE */, TAG_CM) as PHBIN;
        if NewBin.is_null() {
            free((*Hive).Storage[Stable as usize].BlockList as *mut c_void, 0);
            free((*Hive).BaseBlock as *mut c_void, (*Hive).BaseBlockAlloc);
            return STATUS_NO_MEMORY;
        }

        let entry0 = block_list.add(BlockIndex);
        (*entry0).BinAddress = NewBin as _;
        (*entry0).BlockAddress = NewBin as _;

        RtlCopyMemory(
            NewBin as *mut c_void,
            Bin as *const c_void,
            (*Bin).Size as usize,
        );

        if (*Bin).Size > HBLOCK_SIZE {
            let mut i: u32 = 1;
            while i < (*Bin).Size / HBLOCK_SIZE {
                let entry = block_list.add(BlockIndex + i as usize);
                (*entry).BinAddress = NewBin as _;
                (*entry).BlockAddress =
                    ((NewBin as usize) + (i as usize * HBLOCK_SIZE as usize)) as _;
                i += 1;
            }
        }

        BlockIndex += ((*Bin).Size / HBLOCK_SIZE) as usize;
    }

    if HvpCreateHiveFreeCellList(Hive) < 0 {
        HvpFreeHiveBins(Hive);
        free((*Hive).BaseBlock as *mut c_void, (*Hive).BaseBlockAlloc);
        return STATUS_NO_MEMORY;
    }
    // BitmapSize = ROUND_UP(Length, sizeof(ULONG)*8) / 8;
    let BitmapSize: u32 =
        round_up((*Hive).Storage[Stable as usize].Length, core::mem::size_of::<u32>() as u32 * 8)
            / 8;
    let BitmapBuffer = allocate(BitmapSize as _, 1 /* TRUE */, TAG_CM) as *mut u32;
    if BitmapBuffer.is_null() {
        HvpFreeHiveBins(Hive);
        free((*Hive).BaseBlock as *mut c_void, (*Hive).BaseBlockAlloc);
        return STATUS_NO_MEMORY;
    }

    RtlInitializeBitMap(
        core::ptr::addr_of_mut!((*Hive).DirtyVector) as *mut RTL_BITMAP,
        BitmapBuffer,
        BitmapSize * 8,
    );
    RtlClearAllBits(core::ptr::addr_of_mut!((*Hive).DirtyVector) as *mut RTL_BITMAP);
    /*
     * Mark the entire hive as dirty if we charged up the alternate variant of
     * the primary hive (e.g. SYSTEM.ALT).
     */
    if (*ChunkBase).BootRecover == HBOOT_BOOT_RECOVERED_BY_ALTERNATE_HIVE {
        RtlSetAllBits(core::ptr::addr_of_mut!((*Hive).DirtyVector) as *mut RTL_BITMAP);
        (*Hive).DirtyCount = (*Hive).DirtyVector.SizeOfBitMap;
    }

    HvpInitFileName((*Hive).BaseBlock, FileName);

    STATUS_SUCCESS
}

/// Retrieves the base block hive header from the primary hive file in backing
/// storage. C: HvpGetHiveHeader.
#[no_mangle]
pub unsafe extern "system" fn HvpGetHiveHeader(
    Hive: PHHIVE,
    HiveBaseBlock: *mut PHBASE_BLOCK,
    TimeStamp: *mut LARGE_INTEGER,
) -> RESULT {
    let free = (*Hive).Free.unwrap();
    let file_read = (*Hive).FileRead.unwrap();

    ASSERT(core::mem::size_of::<HBASE_BLOCK>() as u32 >= HSECTOR_SIZE * (*Hive).Cluster);

    /* Assume failure and allocate the base block */
    // write_volatile: these are #[no_mangle]-reachable output params; a plain
    // store is captures(none)-elidable and gets DROPPED under a shifted link
    // layout → caller reads an uninit BaseBlock → garbage Storage.Length →
    // HvpWriteHive block-loop hangs (the latent layout-sensitive Hv* hang).
    HiveBaseBlock.write_volatile(core::ptr::null_mut());
    let BaseBlock: PHBASE_BLOCK = HvpAllocBaseBlockAligned(Hive, 1 /* TRUE */, TAG_CM);
    if BaseBlock.is_null() {
        return RESULT::NoMemory;
    }

    /* Clear it */
    RtlZeroMemory(
        BaseBlock as *mut c_void,
        core::mem::size_of::<HBASE_BLOCK>(),
    );

    /* Now read it from disk */
    let mut FileOffset: u32 = 0;
    let mut Result = file_read(
        Hive,
        HFILE_TYPE_PRIMARY,
        &mut FileOffset,
        BaseBlock as *mut c_void,
        ((*Hive).Cluster * HSECTOR_SIZE) as _,
    );
    if Result == 0 {
        /*
         * Try to read the first block of the first bin hive so we can somewhat
         * recover this hive.
         */
        FileOffset = HBLOCK_SIZE;
        Result = file_read(
            Hive,
            HFILE_TYPE_PRIMARY,
            &mut FileOffset,
            BaseBlock as *mut c_void,
            ((*Hive).Cluster * HSECTOR_SIZE) as _,
        );
        if Result == 0 {
            free(BaseBlock as *mut c_void, (*Hive).BaseBlockAlloc);
            return RESULT::NotHive;
        }

        /*
         * Deconstruct the casted buffer into a hive bin. The first bin's offset
         * must be 0 and it must have a sane signature.
         */
        let FirstBin: PHBIN = BaseBlock as PHBIN;
        if (*FirstBin).Signature != HV_HBIN_SIGNATURE || (*FirstBin).FileOffset != 0 {
            free(BaseBlock as *mut c_void, (*Hive).BaseBlockAlloc);
            return RESULT::NotHive;
        }

        /* There's still hope: acknowledge the header needs recovery. */
        TimeStamp.write_volatile((*BaseBlock).TimeStamp);
        return RESULT::RecoverHeader;
    }

    /*
     * The base block isn't maimed but is the header data valid?
     */
    if HvpVerifyHiveHeader(BaseBlock, HFILE_TYPE_PRIMARY) == 0 {
        TimeStamp.write_volatile((*BaseBlock).TimeStamp);
        free(BaseBlock as *mut c_void, (*Hive).BaseBlockAlloc);
        return RESULT::RecoverHeader;
    }

    /* Return information */
    HiveBaseBlock.write_volatile(BaseBlock);
    TimeStamp.write_volatile((*BaseBlock).TimeStamp);
    RESULT::HiveSuccess
}

/// Recovers the registry data by obtaining it from a log associated with the
/// hive. C: HvpRecoverDataFromLog (only compiled when !defined(_M_AMD64)).
#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn HvpRecoverDataFromLog(
    Hive: PHHIVE,
    BaseBlock: PHBASE_BLOCK,
) -> RESULT {
    let file_read = (*Hive).FileRead.unwrap();
    let file_write = (*Hive).FileWrite.unwrap();

    let mut DirtyVector: [u8; HSECTOR_SIZE as usize] = [0; HSECTOR_SIZE as usize];
    let mut Buffer: [u8; HBLOCK_SIZE as usize] = [0; HBLOCK_SIZE as usize];

    /* Read the dirty data from the log */
    let mut FileOffset: u32 = HV_LOG_HEADER_SIZE;
    let mut Success = file_read(
        Hive,
        HFILE_TYPE_LOG,
        &mut FileOffset,
        DirtyVector.as_mut_ptr() as *mut c_void,
        HSECTOR_SIZE as _,
    );
    if Success == 0 {
        if CmIsSelfHealEnabled(0 /* FALSE */) == 0 {
            return RESULT::Fail;
        }
        /* Nothing else to do: trigger self-heal, accept potential data loss. */
        return RESULT::SelfHeal;
    }

    /* Check the dirty vector */
    if (DirtyVector.as_ptr() as *const u32).read_unaligned() != HV_LOG_DIRTY_SIGNATURE {
        if CmIsSelfHealEnabled(0 /* FALSE */) == 0 {
            return RESULT::Fail;
        }
        return RESULT::SelfHeal;
    }

    /* Now read each dirty block individually and write it back to hive */
    let mut LogIndex: u32 = 0;
    let StorageLength: u32 = (*BaseBlock).Length / HBLOCK_SIZE;
    let mut BlockIndex: u32 = 0;
    while BlockIndex < StorageLength {
        /* Skip this block if it's not dirty */
        if DirtyVector
            [(BlockIndex as usize) + core::mem::size_of::<u32>()]
            != HV_LOG_DIRTY_BLOCK
        {
            BlockIndex += 1;
            continue;
        }

        FileOffset = HSECTOR_SIZE + HSECTOR_SIZE + LogIndex * HBLOCK_SIZE;
        Success = file_read(
            Hive,
            HFILE_TYPE_LOG,
            &mut FileOffset,
            Buffer.as_mut_ptr() as *mut c_void,
            HBLOCK_SIZE as _,
        );
        if Success == 0 {
            return RESULT::Fail;
        }

        FileOffset = HBLOCK_SIZE + BlockIndex * HBLOCK_SIZE;
        Success = file_write(
            Hive,
            HFILE_TYPE_PRIMARY,
            &mut FileOffset,
            Buffer.as_mut_ptr() as *mut c_void,
            HBLOCK_SIZE as _,
        );
        if Success == 0 {
            return RESULT::Fail;
        }

        /* Increment the index in log as we continue further */
        LogIndex += 1;
        BlockIndex += 1;
    }

    RESULT::HiveSuccess
}

/// Computes the hive space size by querying the file size of the associated hive
/// file. C: HvpQueryHiveSize (only compiled when !defined(_M_AMD64); kernel build
/// path, i.e. !CMLIB_HOST && !_BLDR_).
#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn HvpQueryHiveSize(Hive: PHHIVE) -> u32 {
    #[cfg(not(feature = "bldr"))]
    {
    let mut FileStandard: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut HiveSize: u32 = 0;

    /*
     * Query the file size of the physical hive file. We need that information in
     * order to ensure how big the hive actually is.
     */
    let Status = ZwQueryInformationFile(
        (*(Hive as PCMHIVE)).FileHandles[HFILE_TYPE_PRIMARY as usize],
        &mut IoStatusBlock,
        core::ptr::addr_of_mut!(FileStandard) as *mut c_void,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
        _FILE_INFORMATION_CLASS_FileStandardInformation as _,
    );
    if Status < 0 {
        return HiveSize;
    }

    /* Now compute the hive size */
    HiveSize = FileStandard.EndOfFile.u.LowPart - HBLOCK_SIZE;
    HiveSize
    }
    #[cfg(feature = "bldr")]
    {
        let _ = Hive;
        0
    }
}

/// Recovers the base block header by obtaining it from a log file associated with
/// the hive. C: HvpRecoverHeaderFromLog (only compiled when !defined(_M_AMD64)).
#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn HvpRecoverHeaderFromLog(
    Hive: PHHIVE,
    TimeStamp: *mut LARGE_INTEGER,
    BaseBlock: *mut PHBASE_BLOCK,
) -> RESULT {
    let free = (*Hive).Free.unwrap();
    let file_read = (*Hive).FileRead.unwrap();
    let file_write = (*Hive).FileWrite.unwrap();

    /*
     * The cluster must not be greater than what the base block can permit.
     */
    ASSERT(core::mem::size_of::<HBASE_BLOCK>() as u32 >= HSECTOR_SIZE * (*Hive).Cluster);

    /* Assume we haven't resuscitated the header */
    let mut HeaderResuscitated: BOOLEAN = 0;

    /* Allocate an aligned buffer for the log header */
    let LogHeader: PHBASE_BLOCK = HvpAllocBaseBlockAligned(Hive, 1 /* TRUE */, TAG_CM);
    if LogHeader.is_null() {
        return RESULT::NoMemory;
    }

    /* Zero out our header buffer */
    RtlZeroMemory(LogHeader as *mut c_void, HSECTOR_SIZE as usize);

    /* Get the base header from the log */
    let mut FileOffset: u32 = 0;
    let mut Success = file_read(
        Hive,
        HFILE_TYPE_LOG,
        &mut FileOffset,
        LogHeader as *mut c_void,
        ((*Hive).Cluster * HSECTOR_SIZE) as _,
    );
    if Success == 0
        || HvpVerifyHiveHeader(LogHeader, HFILE_TYPE_LOG) == 0
        || (*TimeStamp).u.HighPart != (*LogHeader).TimeStamp.u.HighPart
        || (*TimeStamp).u.LowPart != (*LogHeader).TimeStamp.u.LowPart
    {
        /*
         * We failed to read the base block header from the log, or the header
         * itself or timestamp is invalid. Check if self healing is enabled.
         */
        if CmIsSelfHealEnabled(0 /* FALSE */) == 0 {
            free(LogHeader as *mut c_void, (*Hive).BaseBlockAlloc);
            return RESULT::Fail;
        }

        /*
         * Determine the size of this hive so that we can estabilish the length of
         * the base block we are trying to resuscitate.
         */
        let HiveSize: u32 = HvpQueryHiveSize(Hive);
        if HiveSize == 0 {
            free(LogHeader as *mut c_void, (*Hive).BaseBlockAlloc);
            return RESULT::Fail;
        }

        /*
         * We can still resuscitate the base header if we could not grab one from
         * the log by reconstructing the header internals by hand (this assumes the
         * root cell is not NIL nor damaged).
         */
        (*LogHeader).Signature = HV_HBLOCK_SIGNATURE;
        (*LogHeader).Sequence1 = 1;
        (*LogHeader).Sequence2 = 1;
        (*LogHeader).Cluster = 1;
        (*LogHeader).Length = HiveSize;
        (*LogHeader).CheckSum = HvpHiveHeaderChecksum(LogHeader);

        /* Acknowledge that we have resuscitated the header. */
        HeaderResuscitated = 1;
    }

    /*
     * Tag this log header as a primary hive before writing it to the hive.
     */
    (*LogHeader).Type = HFILE_TYPE_PRIMARY;

    /*
     * If we have not made attempts of recovering the header due to log corruption
     * then we have to compute the checksum. This is already done when the header
     * has been resuscitated so don't try to do it twice.
     */
    if HeaderResuscitated == 0 {
        (*LogHeader).CheckSum = HvpHiveHeaderChecksum(LogHeader);
    }

    /* Write the header back to hive now */
    Success = file_write(
        Hive,
        HFILE_TYPE_PRIMARY,
        &mut FileOffset,
        LogHeader as *mut c_void,
        ((*Hive).Cluster * HSECTOR_SIZE) as _,
    );
    if Success == 0 {
        free(LogHeader as *mut c_void, (*Hive).BaseBlockAlloc);
        return RESULT::Fail;
    }

    BaseBlock.write_volatile(LogHeader);
    if HeaderResuscitated != 0 {
        RESULT::SelfHeal
    } else {
        RESULT::HiveSuccess
    }
}

#[no_mangle]
pub unsafe extern "system" fn HvpCreateHive(
    RegistryHive: PHHIVE,
    FileName: PCUNICODE_STRING,
) -> NTSTATUS {
    let BaseBlock: PHBASE_BLOCK;
    let mut Index: u32;

    /* Allocate the base block */
    BaseBlock = HvpAllocBaseBlockAligned(RegistryHive, 0 /* FALSE */, TAG_CM);
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
    }
    // Setup hive data
    (*Hive).BaseBlock = ChunkBase;
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
    let Result: RESULT;
    #[cfg(not(target_arch = "x86_64"))]
    let mut Result2: RESULT;
    let mut TimeStamp: LARGE_INTEGER = core::mem::zeroed();
    let mut Offset: u32 = 0;
    let mut HiveData: *mut core::ffi::c_void;
    let mut FileSize: u32;
    let mut HiveSelfHeal: BOOLEAN = 0;
    // Get the hive header. (This block was collapsed onto a few lines with each DPRINT1
    // replaced by `// DPRINT removed`, which commented out the `return` after it: NoMemory
    // and NotHive fell through and carried on with no valid header.)
    Result = HvpGetHiveHeader(Hive, &mut BaseBlock, &mut TimeStamp);
    match Result {
        RESULT::NoMemory => {
            DbgPrint(b"There's no enough memory to get the header\n\0".as_ptr() as *const i8);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        RESULT::Fail => return STATUS_REGISTRY_CORRUPT,
        RESULT::NotHive => {
            DbgPrint(b"The hive is not an actual registry hive file\n\0".as_ptr() as *const i8);
            return STATUS_NOT_REGISTRY_FILE;
        }
        RESULT::HiveSuccess => {}
        RESULT::SelfHeal => HiveSelfHeal = 1,
        // FIXME (C): RecoverHeader implies RecoverData until HvLoadHive loses its hack.
        RESULT::RecoverData => {}
        RESULT::RecoverHeader => {
            // FIXME (C): see the comment near HvpQueryHiveSize.
            #[cfg(target_arch = "x86_64")]
            {
                DbgPrint(b"The hive base header needs recovery, unsupported on amd64\n\0".as_ptr() as *const i8);
                return STATUS_REGISTRY_CORRUPT;
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                DbgPrint(b"Attempting to heal the header...\n\0".as_ptr() as *const i8);
                Result2 = HvpRecoverHeaderFromLog(Hive, &mut TimeStamp, &mut BaseBlock);
                if Result2 == RESULT::NoMemory {
                    DbgPrint(b"There's no enough memory to recover header from log\n\0".as_ptr() as *const i8);
                    return STATUS_INSUFFICIENT_RESOURCES;
                }
                if Result2 == RESULT::Fail {
                    DbgPrint(b"Failed to recover the hive header\n\0".as_ptr() as *const i8);
                    return STATUS_REGISTRY_CORRUPT;
                }
                if Result2 == RESULT::SelfHeal {
                    HiveSelfHeal = 1;
                }
                Result2 = HvpRecoverDataFromLog(Hive, BaseBlock);
                if Result2 == RESULT::Fail {
                    DbgPrint(b"Failed to recover the hive data\n\0".as_ptr() as *const i8);
                    return STATUS_REGISTRY_CORRUPT;
                }
                if Result2 == RESULT::SelfHeal && HiveSelfHeal == 0 {
                    HiveSelfHeal = 1;
                }
            }
        }
    }
    /* Set the boot type */ (*BaseBlock).BootType = if HiveSelfHeal != 0 { HBOOT_TYPE_SELF_HEAL } else { HBOOT_TYPE_REGULAR };
    /* Setup hive data */ (*Hive).BaseBlock = BaseBlock;
    (*Hive).Version = (*BaseBlock).Minor;
    /* Allocate a buffer large enough to hold the hive */ FileSize = HBLOCK_SIZE + (*BaseBlock).Length; // == sizeof(HBASE_BLOCK) + BaseBlock->Length;
    HiveData = ((*Hive).Allocate.unwrap())(FileSize as _, 1u8, TAG_CM); // TRUE -> 1
    if HiveData.is_null() {
        ((*Hive).Free.unwrap())(BaseBlock as *mut core::ffi::c_void, (*Hive).BaseBlockAlloc);
        // DPRINT removed
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    // HACK: Now read the whole hive
    Success = ((*Hive).FileRead.unwrap())(Hive, HFILE_TYPE_PRIMARY, &mut Offset, HiveData, FileSize as _);
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
    // RtlZeroMemory(Hive, sizeof(HHIVE)).
    core::ptr::write_bytes(Hive as *mut u8, 0, core::mem::size_of::<HHIVE>());
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
