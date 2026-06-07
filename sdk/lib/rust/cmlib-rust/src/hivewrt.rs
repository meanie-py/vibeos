use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CELL_DATA, CM_KEY_NODE, HBASE_BLOCK, HBIN, HCELL, HCELL_INDEX, HHIVE, IoSetThreadHardErrorMode, PCELL_DATA, PHBASE_BLOCK, PHBIN, PHHIVE, RtlFindSetBits, HSTORAGE_TYPE_Stable as Stable, UINT32, _HHIVE, _UNICODE_STRING};
use crate::glue::{HCELL_NIL, HBLOCK_SIZE, TAG_CM, HV_HHIVE_SIGNATURE, HFILE_TYPE_ALTERNATE, HFILE_TYPE_LOG, HFILE_TYPE_PRIMARY, HBOOT_TYPE_SELF_HEAL, STATUS_REGISTRY_CORRUPT, STATUS_REGISTRY_IO_FAILED, STATUS_NOT_REGISTRY_FILE, STATUS_REGISTRY_RECOVERED, HIVE_VOLATILE};
use core::ffi::c_void;
use core::ptr::addr_of_mut;

extern "C" {
    unsafe fn HvpValidateBaseHeader(RegistryHive: *mut HHIVE) ;
    fn HvpHiveHeaderChecksum(BaseBlock: PHBASE_BLOCK) -> u32;
    fn DbgPrint(format: *const i8, ...);
    unsafe fn HvpWriteHive(RegistryHive: *mut HHIVE, OnlyDirty: i32, FileType: u32) -> i32;
}
extern "C" {
    fn KeQuerySystemTime(SystemTime: *mut i64);
    fn HvpWriteHive(RegistryHive: *mut HHIVE, OnlyDirty: u8, FileType: u32) -> u8;
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: i32 = 1;

const HV_HBLOCK_SIGNATURE: u32 = 0xbee0bee0;
const HBASE_FORMAT_MEMORY: u32 = 1;
const HSYS_MAJOR: u32 = 1;

const HV_LOG_HEADER_SIZE: u32 = 4096;
const HSECTOR_SIZE: u32 = 512;
const HV_LOG_DIRTY_SIGNATURE: u32 = 0x4c445646;
const HV_LOG_DIRTY_BLOCK: u32 = 1;
const HV_CLEAN_BLOCK: u32 = 0;

unsafe extern "C" fn HvpWriteLog(RegistryHive: *mut HHIVE) -> i32 {
    let mut Success: i32;
    let mut FileOffset: u32;
    let mut BlockIndex: u32;
    let mut LastIndex: u32;
    let mut Block: *mut u8;
    let mut BitmapSize: u32;
    let mut BufferSize: u32;
    let mut HeaderBuffer: *mut u8;
    let mut Ptr: *mut u8;

    HvpValidateBaseHeader(RegistryHive);

    if (*(*RegistryHive).BaseBlock).Sequence1 != (*(*RegistryHive).BaseBlock).Sequence2 {
        DbgPrint(
            b"The sequences DO NOT MATCH (Sequence1 == 0x%x, Sequence2 == 0x%x)\n\0".as_ptr() as *const i8,
            (*(*RegistryHive).BaseBlock).Sequence1,
            (*(*RegistryHive).BaseBlock).Sequence2,
        );
        return 0;
    }

    BitmapSize = ROUND_UP(
        core::mem::size_of::<u32>() as u32 + (*(*RegistryHive).DirtyVector).SizeOfBitMap,
        HSECTOR_SIZE,
    );
    BufferSize = HV_LOG_HEADER_SIZE + BitmapSize;

    HeaderBuffer = ((*RegistryHive).Allocate as unsafe extern "C" fn(u32, i32, u32) -> *mut u8)(
        BufferSize,
        1,
        TAG_CM,
    );
    if HeaderBuffer.is_null() {
        DbgPrint(b"Couldn't allocate buffer for base header block\n\0".as_ptr() as *const i8);
        return 0;
    }

    RtlZeroMemory(HeaderBuffer, BufferSize as _);

    (*((*RegistryHive).BaseBlock)).Type = HFILE_TYPE_LOG;
    (*((*RegistryHive).BaseBlock)).Sequence1 = (*((*RegistryHive).BaseBlock)).Sequence1.wrapping_add(1);
    (*((*RegistryHive).BaseBlock)).CheckSum = HvpHiveHeaderChecksum((*RegistryHive).BaseBlock);

    RtlCopyMemory(
        HeaderBuffer as _,
        (*RegistryHive).BaseBlock as _,
        HV_LOG_HEADER_SIZE as _,
    );
    Ptr = HeaderBuffer.add(HV_LOG_HEADER_SIZE as usize);

    *(Ptr as *mut u32) = HV_LOG_DIRTY_SIGNATURE;
    Ptr = Ptr.add(core::mem::size_of::<u32>());

    BlockIndex = 0;
    while BlockIndex < (*(*RegistryHive).Storage[Stable as usize]).Length {
        LastIndex = BlockIndex;
        BlockIndex = RtlFindSetBits(
            &((*RegistryHive).DirtyVector as *const RTL_BITMAP),
            1,
            BlockIndex,
        );
        if BlockIndex == !HV_CLEAN_BLOCK || BlockIndex < LastIndex {
            break;
        }

        *Ptr.add(BlockIndex as usize) = HV_LOG_DIRTY_BLOCK as u8;
        BlockIndex = BlockIndex.wrapping_add(1);
    }

    FileOffset = 0;
    Success = ((*RegistryHive).FileWrite as unsafe extern "C" fn(
        *mut HHIVE,
        u32,
        *mut u32,
        *mut u8,
        u32,
    ) -> i32)(
        RegistryHive,
        HFILE_TYPE_LOG,
        &mut FileOffset as *mut u32,
        HeaderBuffer,
        BufferSize,
    );
    ((*RegistryHive).Free as unsafe extern "C" fn(*mut u8, u32))(HeaderBuffer, 0);
    if Success == 0 {
        DbgPrint(b"Failed to write the hive header block to log (primary sequence)\n\0".as_ptr() as *const i8);
        return 0;
    }

    FileOffset = BufferSize;
    BlockIndex = 0;
    while BlockIndex < (*(*RegistryHive).Storage[Stable as usize]).Length {
        LastIndex = BlockIndex;
        BlockIndex = RtlFindSetBits(
            &((*RegistryHive).DirtyVector as *const RTL_BITMAP),
            1,
            BlockIndex,
        );
        if BlockIndex == !HV_CLEAN_BLOCK || BlockIndex < LastIndex {
            break;
        }

        Block = (*(*RegistryHive).Storage[Stable as usize]).BlockList[BlockIndex as usize].BlockAddress as *mut u8;

        Success = ((*RegistryHive).FileWrite as unsafe extern "C" fn(
            *mut HHIVE,
            u32,
            *mut u32,
            *mut u8,
            u32,
        ) -> i32)(
            RegistryHive,
            HFILE_TYPE_LOG,
            &mut FileOffset as *mut u32,
            Block,
            HBLOCK_SIZE,
        );
        if Success == 0 {
            DbgPrint(
                b"Failed to write dirty block to log (block 0x%p, block index 0x%x)\n\0".as_ptr() as *const i8,
                Block as *mut core::ffi::c_void,
                BlockIndex,
            );
            return 0;
        }

        BlockIndex = BlockIndex.wrapping_add(1);
        FileOffset = FileOffset.wrapping_add(HBLOCK_SIZE);
    }

    Success = ((*RegistryHive).FileFlush as unsafe extern "C" fn(
        *mut HHIVE,
    ) -> i32)(RegistryHive);
    return 1;
}

#[no_mangle]
pub unsafe extern "C" fn HvSyncHive(RegistryHive: *mut HHIVE) -> u8 {
    let HardErrors: u8;

    assert!(!(*RegistryHive).ReadOnly);
    assert!((*RegistryHive).Signature == HV_HHIVE_SIGNATURE);

    if (*RegistryHive).HiveFlags & HIVE_VOLATILE != 0 {
        DbgPrint(
            b"Hive 0x%p is volatile\n\0".as_ptr() as *const i8,
            RegistryHive as *const core::ffi::c_void,
        );
        return 1u8;
    }

    if RtlFindSetBits(core::ptr::addr_of_mut!((*RegistryHive).DirtyVector), 1, 0) == !HV_CLEAN_BLOCK {
        DbgPrint(
            b"The dirty vector has clean data, nothing to do\n\0".as_ptr() as *const i8,
        );
        return 1u8;
    }

    HardErrors = IoSetThreadHardErrorMode(0u8);

    KeQuerySystemTime(core::ptr::addr_of_mut!((*(*RegistryHive).BaseBlock).TimeStamp));

    if (*RegistryHive).Log != 0 {
        if HvpWriteLog(RegistryHive) == 0 {
            DbgPrint(
                b"Failed to write a log whilst syncing the hive\n\0".as_ptr() as *const i8,
            );
            IoSetThreadHardErrorMode(HardErrors);
            return 0u8;
        }
    }

    if HvpWriteHive(RegistryHive, 1u8, HFILE_TYPE_PRIMARY) == 0 {
        DbgPrint(
            b"Failed to write the primary hive\n\0".as_ptr() as *const i8,
        );
        IoSetThreadHardErrorMode(HardErrors);
        return 0u8;
    }

    if (*RegistryHive).Alternate != 0 {
        if HvpWriteHive(RegistryHive, 1u8, HFILE_TYPE_ALTERNATE) == 0 {
            DbgPrint(
                b"Failed to write the alternate hive\n\0".as_ptr() as *const i8,
            );
            IoSetThreadHardErrorMode(HardErrors);
            return 0u8;
        }
    }

    RtlClearAllBits(core::ptr::addr_of_mut!((*RegistryHive).DirtyVector));
    (*RegistryHive).DirtyCount = 0;

    IoSetThreadHardErrorMode(HardErrors);
    return 1u8;
}
#[no_mangle]
pub unsafe extern "C" fn HvHiveWillShrink(
    RegistryHive: *mut HHIVE,
) -> i32 {
    // No shrinking yet
    DbgPrint(b"HvHiveWillShrink is unimplemented\n\0".as_ptr() as *const i8);
    return 0;
}

#[cfg(not(bldr))]

#[no_mangle]
pub unsafe extern "C" fn HvWriteHive(RegistryHive: *mut HHIVE) -> i32 {
    debug_assert!((*RegistryHive).ReadOnly == 0);
    debug_assert!((*RegistryHive).Signature == HV_HHIVE_SIGNATURE);

    #[cfg(not(bldr))]
    {
        // Update hive header modification time
        KeQuerySystemTime(addr_of_mut!((*(*RegistryHive).BaseBlock).TimeStamp));
    }

    // Update hive file
    if HvpWriteHive(RegistryHive, 0, HFILE_TYPE_PRIMARY) == 0 {
        DbgPrint(b"Failed to write the hive\n\0".as_ptr() as *const u8);
        return 0;
    }

    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn HvWriteAlternateHive(RegistryHive: *mut HHIVE) -> i32 {
    core::assert!((*RegistryHive).ReadOnly == 0);
    core::assert!((*RegistryHive).Signature == HV_HHIVE_SIGNATURE);
    core::assert!((*RegistryHive).Alternate != 0);
    #[cfg(not(_BLDR_))]
    {
        KeQuerySystemTime(core::ptr::addr_of_mut!((*(*RegistryHive).BaseBlock).TimeStamp));
    }
    if HvpWriteHive(RegistryHive, 0, HFILE_TYPE_ALTERNATE) == 0 {
        DbgPrint(b"Failed to write the alternate hive\n\0".as_ptr() as *const i8);
        return 0;
    }
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn HvSyncHiveFromRecover(RegistryHive: *mut HHIVE) -> i32 {
    unsafe {
        debug_assert!((*RegistryHive).ReadOnly == 0);
        debug_assert!((*RegistryHive).Signature == HV_HHIVE_SIGNATURE);

        HvpWriteHive(RegistryHive, 1, HFILE_TYPE_PRIMARY);
    }
}
