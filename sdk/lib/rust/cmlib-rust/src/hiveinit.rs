use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CELL_DATA, CM_KEY_NODE, CmIsSelfHealEnabled, FILE_STANDARD_INFORMATION, FileStandardInformation, HBASE_BLOCK, HBIN, HCELL_INDEX, HHIVE, HMAP_ENTRY, PCELL_DATA, PCMHIVE, PCUNICODE_STRING, PFILE_FLUSH_ROUTINE, PFILE_READ_ROUTINE, PFILE_SET_SIZE_ROUTINE, PFILE_WRITE_ROUTINE, PFREE_ROUTINE, PHBASE_BLOCK, PHBIN, PHHIVE, PLARGE_INTEGER, Stable, TimeStamp, Volatile, _HHIVE, _IO_STATUS_BLOCK, _UNICODE_STRING, LARGE_INTEGER, PALLOCATE_ROUTINE};
use crate::glue::*;
use core::ffi::c_void;

extern "C" {
    fn DPRINT1(fmt: *const i8, ...);
    unsafe fn HvpAllocBaseBlockAligned(Hive: *mut HHIVE, Paged: i32, Tag: u32) -> *mut HBASE_BLOCK;
    unsafe fn HvpInitFileName(BaseBlock: *mut HBASE_BLOCK, FileName: *const _UNICODE_STRING) ;
    fn HvpHiveHeaderChecksum(BaseBlock: *mut HBASE_BLOCK) -> u32;
    fn HvpCreateHiveFreeCellList(Hive: *mut HHIVE) -> i32;
    fn HvpFreeHiveBins(Hive: *mut HHIVE);
}
extern "C" {
    fn ZwQueryInformationFile( FileHandle: HANDLE, IoStatusBlock: *mut _IO_STATUS_BLOCK, FileInformation: *mut core::ffi::c_void, Length: u32, FileInformationClass: FILE_INFORMATION_CLASS, ) -> NTSTATUS;
    fn HvpVerifyHiveHeader(BaseBlock: *mut HBASE_BLOCK, FileType: u32) -> u8;
    fn HvpAllocBaseBlockAligned(Hive: *mut HHIVE, Paged: i32, Tag: u32) -> *mut HBASE_BLOCK;
    fn HvpInitFileName(BaseBlock: *mut HBASE_BLOCK, FileName: *const _UNICODE_STRING);
}
#[no_mangle]
pub unsafe extern "C" fn HvpCreateHive(
    Hive: *mut HHIVE,
    ChunkBase: *mut HBASE_BLOCK,
    FileName: PCUNICODE_STRING,
) -> i32 { let mut BlockIndex: usize;
    let Bin: *mut HBIN;
    let NewBin: *mut HBIN;
    let mut i: u32;
    let BitmapSize: u32;
    let BitmapBuffer: *mut u32;
    let ChunkSize: usize;
    ChunkSize = (*ChunkBase).Length as usize;
    // DPRINT("ChunkSize: %zx\n", ChunkSize);
    if ChunkSize < core::mem::size_of::<HBASE_BLOCK>() || HvpVerifyHiveHeader(ChunkBase, HFILE_TYPE_PRIMARY) == 0 {
        // DPRINT1("Registry is corrupt: ChunkSize 0x%zx < sizeof(HBASE_BLOCK) 0x%zx, or HvpVerifyHiveHeader() failed\n", ChunkSize, core::mem::size_of::<HBASE_BLOCK>());
        return STATUS_REGISTRY_CORRUPT;
    }
    // Allocate the base block
    (*Hive).BaseBlock = HvpAllocBaseBlockAligned(Hive, 0, TAG_CM);
    if (*Hive).BaseBlock.is_null() { return STATUS_NO_MEMORY;
    } core::ptr::copy_nonoverlapping( ChunkBase as *const u8, (*Hive).BaseBlock as *mut u8, core::mem::size_of::<HBASE_BLOCK>(), );
    // Setup hive data
    (*Hive).Version = (*ChunkBase).Minor;
    // Build a block list from the in-memory chunk and copy the data as we go
    (*Hive).Storage[Stable].Length = (ChunkSize / HBLOCK_SIZE) as u32;
    (*Hive).Storage[Stable].BlockList = ((*Hive).Allocate.unwrap())( ((*Hive).Storage[Stable].Length as usize) * core::mem::size_of::<HMAP_ENTRY>(), 0, TAG_CM, ) as *mut HMAP_ENTRY;
    if (*Hive).Storage[Stable].BlockList.is_null() { println!("Allocating block list failed");
    ((*Hive).Free.unwrap())((*Hive).BaseBlock as *mut c_void, (*Hive).BaseBlockAlloc);
    return STATUS_NO_MEMORY;
    } BlockIndex = 0;
    while BlockIndex < ((*Hive).Storage[Stable].Length as usize) { Bin = (ChunkBase as usize + (BlockIndex + 1) * HBLOCK_SIZE) as *mut HBIN;
    if (*Bin).Signature != HV_HBIN_SIGNATURE || ((*Bin).Size as usize % HBLOCK_SIZE) != 0 || ((*Bin).FileOffset / (HBLOCK_SIZE as u32)) != (BlockIndex as u32) { if CmIsSelfHealEnabled(0) == 0 { println!( "Invalid bin at BlockIndex {}, Signature 0x{:x}, Size 0x{:x}. Self-heal not possible!", BlockIndex, (*Bin).Signature, (*Bin).Size );
    ((*Hive).Free.unwrap())((*Hive).Storage[Stable].BlockList as *mut c_void, 0);
    ((*Hive).Free.unwrap())((*Hive).BaseBlock as *mut c_void, (*Hive).BaseBlockAlloc);
    return STATUS_REGISTRY_CORRUPT;
    } // Fix this bin (*Bin).Signature = HV_HBIN_SIGNATURE;
    (*Bin).Size = HBLOCK_SIZE as u32;
    (*Bin).FileOffset = (BlockIndex * HBLOCK_SIZE) as u32;
    (*ChunkBase).BootType |= HBOOT_TYPE_SELF_HEAL;
    println!( "Bin at index {} is corrupt and it has been repaired!", BlockIndex );
    } NewBin = ((*Hive).Allocate.unwrap())((*Bin).Size as usize, 1, TAG_CM) as *mut HBIN;
    if NewBin.is_null() { ((*Hive).Free.unwrap())((*Hive).Storage[Stable].BlockList as *mut c_void, 0);
    ((*Hive).Free.unwrap())((*Hive).BaseBlock as *mut c_void, (*Hive).BaseBlockAlloc);
    return STATUS_NO_MEMORY;
    } (*Hive).Storage[Stable].BlockList[BlockIndex].BinAddress = NewBin as usize;
    (*Hive).Storage[Stable].BlockList[BlockIndex].BlockAddress = NewBin as usize;
    core::ptr::copy_nonoverlapping( Bin as *const u8, NewBin as *mut u8, (*Bin).Size as usize, );
    if (*Bin).Size > (HBLOCK_SIZE as u32) { let bin_count = (*Bin).Size / (HBLOCK_SIZE as u32);
    for i in 1..bin_count as usize {
        (*Hive).Storage[Stable].BlockList[BlockIndex + i].BinAddress = NewBin as usize;
        (*Hive).Storage[Stable].BlockList[BlockIndex + i].BlockAddress = NewBin as usize + (i * HBLOCK_SIZE);
    } } BlockIndex += ((*Bin).Size as usize) / HBLOCK_SIZE;
    }
    if !NT_SUCCESS(HvpCreateHiveFreeCellList(Hive)) {
        HvpFreeHiveBins(Hive);
        ((*Hive).Free.unwrap())((*Hive).BaseBlock as *mut c_void, (*Hive).BaseBlockAlloc);
        return STATUS_NO_MEMORY;
    }
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn HvpInitializeFlatHive(
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
    let mut Result: u32 = 0; // ULONG
    #[cfg(not(target_arch = "x86_64"))]
    let mut Result2: u32 = 0;
    let mut TimeStamp: LARGE_INTEGER = core::mem::zeroed();
    let mut Offset: u32 = 0;
    let mut HiveData: *mut core::ffi::c_void;
    let mut FileSize: u32;
    let mut HiveSelfHeal: BOOLEAN = 0;
    /* Get the hive header */ Result = HvpGetHiveHeader(Hive, &mut BaseBlock, &mut TimeStamp);
    match Result { RESULT::NoMemory => { /* Fail */ // DPRINT removed return STATUS_INSUFFICIENT_RESOURCES;
    } RESULT::NotHive => { /* Fail */ // DPRINT removed return STATUS_NOT_REGISTRY_FILE;
    } RESULT::RecoverData => { /* * FIXME: We must be handling this status * case if the header isn't corrupt but * the counter sequences do not match but * due to a hack in HvLoadHive we have * to do both a header + data recovery. * RecoverHeader also implies RecoverData * anyway. When HvLoadHive gets rid of * that hack, data recovery must be done * after we read the hive block by block. */ /* break; */ } RESULT::RecoverHeader => { /* FIXME: See the comment above (near HvpQueryHiveSize) */ #[cfg(target_arch = "x86_64")] { return STATUS_REGISTRY_CORRUPT;
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
    HiveData = ((*Hive).Allocate.unwrap())(FileSize, 1, TAG_CM); // TRUE -> 1
    if HiveData.is_null() {
        ((*Hive).Free.unwrap())(BaseBlock as *mut core::ffi::c_void, (*Hive).BaseBlockAlloc);
        // DPRINT removed
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    // HACK: Now read the whole hive
    Success = ((*Hive).FileRead.unwrap())(Hive, HFILE_TYPE_PRIMARY, &mut Offset, HiveData, FileSize);
    if Success == 0 {
        // FALSE // DPRINT removed
        ((*Hive).Free.unwrap())(HiveData, FileSize);
        ((*Hive).Free.unwrap())(BaseBlock as *mut core::ffi::c_void, (*Hive).BaseBlockAlloc);
        return STATUS_NOT_REGISTRY_FILE;
    }
    // HACK (FIXME): Free our base block...
    ((*Hive).Free.unwrap())(BaseBlock as *mut core::ffi::c_void, (*Hive).BaseBlockAlloc);
    Status = HvpInitializeMemoryHive(Hive, HiveData, FileName);
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
    Allocate: Option<unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> *mut core::ffi::c_void>,
    Free: Option<unsafe extern "system" fn(*mut core::ffi::c_void)>,
    FileSetSize: Option<unsafe extern "system" fn(*mut HHIVE, i32, *mut i64) -> i32>,
    FileWrite: Option<unsafe extern "system" fn(*mut HHIVE, u32, *mut u32, *mut u8, u32) -> bool>,
    FileRead: Option<unsafe extern "system" fn(*mut HHIVE, u32, *mut u32, *mut u8, u32) -> bool>,
    FileFlush: Option<unsafe extern "system" fn(*mut HHIVE, u32) -> bool>,
    Cluster: u32,
    FileName: *const _UNICODE_STRING,
) -> NTSTATUS {
    // Local constants
    const HIVE_NOLAZYFLUSH: u32 = 0x10000;
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
    (*Hive).GetCellRoutine = HvpGetCellData as usize;
    (*Hive).ReleaseCellRoutine = 0 as *mut (); // NULL
    Status = match OperationType {
        HINIT_CREATE => {
            Status = HvpCreateHive(Hive, FileName);
            Status
        }
        HINIT_MEMORY => {
            Status = HvpInitializeMemoryHive(Hive, HiveData, FileName);
            Status
        }
        HINIT_FLAT => {
            Status = HvpInitializeFlatHive(Hive, HiveData);
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
            if !HvSyncHiveFromRecover(Hive) {
                // DPRINT1("Fail to write healthy data back to hive\n");
                return STATUS_REGISTRY_IO_FAILED;
            }
            // FIXME: clear log and reset size to 0
            RtlClearAllBits(core::ptr::addr_of_mut!((*Hive).DirtyVector) as *mut RTL_BITMAP);
        }
    }
            Status
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn HvFree(RegistryHive: *mut HHIVE) {
    // if (!RegistryHive->ReadOnly)
    if (*RegistryHive).ReadOnly == 0 {
        // Release hive bitmap
        if !(*RegistryHive).DirtyVector.Buffer.is_null() {
            ((*RegistryHive).Free.unwrap())((*RegistryHive).DirtyVector.Buffer, 0);
        }
        HvpFreeHiveBins(RegistryHive);
        // Free the BaseBlock
        if !(*RegistryHive).BaseBlock.is_null() {
            ((*RegistryHive).Free.unwrap())((*RegistryHive).BaseBlock, (*RegistryHive).BaseBlockAlloc);
            (*RegistryHive).BaseBlock = core::ptr::null_mut();
        }
    }
}