#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::globals::*;
// One declaration per external symbol: externs.rs.
use crate::externs::FatBugCheck;
use crate::externs::FatCompleteRequest;
use crate::externs::FatIsFat32;
use crate::externs::FatRootDirectoryLbo;
use crate::externs::FatUnpinBcb;
use crate::externs::FatVcbAcquiredExclusive;
use crate::externs::CcSetDirtyPinnedData;
use crate::externs::ExIsResourceAcquiredExclusiveLite;
use crate::externs::KeCancelTimer;
use crate::externs::KeRemoveQueueDpc;
// fastfat's own functions: the definition is the declaration.
use crate::fatdata::FatData;
use crate::filobsup::FatDecodeFileObject;
use crate::verfysup::FatMarkVolume;
use crate::seh::FatNormalizeAndRaiseStatus;
use crate::seh::FatRaiseStatus;
use crate::fsctrl::FatRemoveMcbEntry;
use crate::filobsup::FatSetFileObject;
use crate::deviosup::FatToggleMediaEjectDisable;
use crate::allocsup::FatTruncateFileAllocation;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::LARGE_INTEGER;
use ntkrnl_sys::ddk::{CACHE_UNINITIALIZE_EVENT, CcFlushCache, CcGetFileObjectFromBcb, CcInitializeCacheMap, CcMapData, CcMdlReadComplete, CcMdlWriteComplete, CcPinMappedData, CcPinRead, CcPurgeCacheSection, CcRepinBcb, CcSetAdditionalCacheAttributes, CcSetFileSizes, CcUninitializeCacheMap, CcUnpinRepinnedBcb, CcZeroData, ExFreePool, _KWAIT_REASON_Executive, FILE_SEGMENT_ELEMENT, _IO_STATUS_BLOCK, IoCreateStreamFileObject, IoSetHardErrorOrVerifyDevice, KeInitializeEvent, KeWaitForSingleObject, MM_PREFETCH_FLAGS, MmPrefetchPages, PCACHE_MANAGER_CALLBACKS, PCC_FILE_SIZES, PDEVICE_OBJECT, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PNTSTATUS, PREAD_LIST, READ_LIST, _EVENT_TYPE_SynchronizationEvent};
use core::ffi::c_void;
use crate::types::{BIOS_PARAMETER_BLOCK, CCB, DCB, FAT_BUG_CHECK_CACHESUP, FAT_DATA, FCB, FatLargeZero, FatMaxLarge, IRP_CONTEXT, IRP_CONTEXT_FLAG_DISABLE_RAISE, PBCB, PDCB, PIRP_CONTEXT, PVCB, REPINNED_BCBS, VBO, VCB, _TYPE_OF_OPEN_DirectoryFile as DirectoryFile, _TYPE_OF_OPEN_EaFile as EaFile};
use ntkrnl_sys::ddk::{CACHE_MANAGER_CALLBACKS, CC_FILE_SIZES, DEVICE_OBJECT, ERESOURCE, FILE_OBJECT, IO_STACK_LOCATION, IRP, _PUBLIC_BCB as BCB, KDPC, KEVENT, KSPIN_LOCK, KTIMER};
use crate::ddk_shims::{FsRtlAllocatePoolWithTag};

extern "system" {
    static mut FatDisableParentCheck: i32;
    } extern "system" {
    fn FatAcquireDirectoryFileMutex(Vcb: *mut VCB);
    fn FatCheckFreeDirentBitmap(IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB);
    fn FatLookupFileAllocationSize(IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB);
    fn FatPreallocateCloseContext(Vcb: *mut VCB);
    fn FatReleaseDirectoryFileMutex(Vcb: *mut VCB);
    fn FatIsFat12(Vcb: *mut VCB) -> u8;
    fn KeQuerySystemTime(Time: *mut LARGE_INTEGER);
    fn KeAcquireSpinLock(SpinLock: *mut KSPIN_LOCK, OldIrql: *mut u8);
    fn KeReleaseSpinLock(SpinLock: *mut KSPIN_LOCK, OldIrql: u8);
    fn KeSetTimer(Timer: *mut KTIMER, DueTime: LARGE_INTEGER, Dpc: *mut KDPC) -> u8;
}
extern "system" {
    static FatDiskAccountingEnabled: BOOLEAN;
    fn CcSetAdditionalCacheAttributesEx(FileObject: PFILE_OBJECT, Flags: u32);
// LBO
    fn PagedCode();
    fn ExIsResourceAcquiredSharedLite(resource: *const ERESOURCE) -> u8;
    fn IoInitializePriorityInfo(PriorityInfo: *mut _IO_PRIORITY_INFO);
    fn IoRetrievePriorityInfo( Irp: *mut IRP, FileObject: *mut FILE_OBJECT, Thread: *mut core::ffi::c_void, PriorityInfo: *mut _IO_PRIORITY_INFO, ) -> NTSTATUS;
    fn FatAddFileAllocation( IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB, FileObject: PFILE_OBJECT, NewAllocation: u32, );
}
extern "C" {
    // wdm FORCEINLINE helper: cdecl body (`_IoGetCurrentIrpStackLocation`),
    // matches the ntkrnl-sys bindgen class — "system" decl was a §9.99 fold.
    fn IoGetCurrentIrpStackLocation(irp: *mut IRP) -> *mut IO_STACK_LOCATION;
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_CACHESUP;
pub const Dbg: u32 = DEBUG_TRACE_CACHESUP;

// IO_PRIORITY_INFO (from wdm.h / ntifs.h; not in ntkrnl-sys bindgen)
#[repr(C)]
struct _IO_PRIORITY_INFO {
    Size: u32,
    ThreadPriority: u32,
    PagePriority: u32,
    IoPriority: u32, // IO_PRIORITY_HINT enum, stored as u32
}


const CC_ENABLE_DISK_IO_ACCOUNTING: u32 = 0x1;

const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02; // placeholder  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
const STATUS_CANT_WAIT: i32 = 0xC00000D8_u32 as i32; // placeholder
const DEBUG_TRACE_CACHESUP: u32 = 0x04000000; // placeholder

const IRP_MN_MOUNT_VOLUME: u32 = 1;
const IRP_CONTEXT_FLAG_PARENT_BY_CHILD: u32 = 0x80000000;
const VACB_MAPPING_GRANULARITY: u32 = 0x40000; // from sdk/include/xdk/cctypes.h


// Unknown constants (placeholder values)

// External functions not in provided imports

const SYSTEM_PAGE_PRIORITY_LEVELS: u32 = 8;

// EaFile and FatMaxLarge come from crate::types (extern statics, type LARGE_INTEGER)

// Forward declarations for unknown externals

// Forward declare local functions not yet translated

// Constants (values assumed for compilation)
const FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64 = -1;
const FAT_NTC_ROOT_DCB: u32 = 0x0504;

// Constants with placeholder values
const IRP_CONTEXT_FLAG_DISABLE_DIRTY: u32 = 0x01;
const VCB_STATE_FLAG_DEFERRED_FLUSH: u32 = 0x1000;
const VCB_STATE_FLAG_VOLUME_DIRTY: u32 = 0x04;
const VCB_STATE_FLAG_MOUNTED_DIRTY: u32 = 0x10;
const VCB_STATE_FLAG_REMOVABLE_MEDIA: u32 = 0x02;
const VolumeDirty: u32 = 1;

// Placeholder structs for unknown types
// Types come from crate::types (types_amd64.rs) and ntkrnl_sys

const STATUS_END_OF_FILE: i32 = 0xC0000011_u32 as i32;



const STATUS_DISK_FULL: i32 = 0xC000007F_u32 as i32;
#[no_mangle]
pub unsafe extern "system" fn FatInitializeCacheMap(
    FileObject: PFILE_OBJECT,
    FileSizes: PCC_FILE_SIZES,
    PinAccess: BOOLEAN,
    Callbacks: PCACHE_MANAGER_CALLBACKS,
    LazyWriteContext: *mut core::ffi::c_void,
) {
    CcInitializeCacheMap(
        FileObject,
        FileSizes,
        PinAccess,
        Callbacks,
        LazyWriteContext,
    );

    // NTDDI_VERSION >= NTDDI_WIN8
    if FatDiskAccountingEnabled != 0 {
        CcSetAdditionalCacheAttributesEx(FileObject, CC_ENABLE_DISK_IO_ACCOUNTING);
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatReadVolumeFile(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    StartingVbo: u32,
    ByteCount: u32,
    Bcb: *mut *mut BCB,
    Buffer: *mut *mut core::ffi::c_void,
) {
    unsafe {
        PAGED_CODE!();

        let condition = (StartingVbo == 0) ||
            (StartingVbo.wrapping_add(ByteCount) as i64 <=
                (FatRootDirectoryLbo(&raw const (*Vcb).Bpb as *const _) + PAGE_SIZE as i64));
        NT_ASSERT!(condition as i32);

        DebugTrace!(1, Dbg as u32, b"FatReadVolumeFile\n\0".as_ptr() as *const i8, (0u32) as usize);
        DebugTrace!(0, Dbg as u32, b"Vcb         = %p\n\0".as_ptr() as *const i8, Vcb as usize);
        DebugTrace!(0, Dbg as u32, b"StartingVbo = %08lx\n\0".as_ptr() as *const i8, (StartingVbo as u64) as usize);
        DebugTrace!(0, Dbg as u32, b"ByteCount   = %08lx\n\0".as_ptr() as *const i8, (ByteCount as u64) as usize);

        let mut Vbo: LARGE_INTEGER = core::mem::zeroed();
        Vbo.QuadPart = StartingVbo as i64;

        if CcMapData(
            (*Vcb).VirtualVolumeFile,
            &mut Vbo,
            ByteCount,
            BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) as u32,
            Bcb as *mut *mut c_void,
            Buffer,
        ) == 0
        {
            let flag_on_result = FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT as u32);
            NT_ASSERT!((flag_on_result == 0) as i32);

            FatRaiseStatus(IrpContext, STATUS_CANT_WAIT as i32);
        }

        (*IrpContext).PinCount += 1;
        DbgDoit!(((*IrpContext).PinCount) as u32);

        DebugTrace!(-1, Dbg as u32, b"FatReadVolumeFile -> VOID, *BCB = %p\n\0".as_ptr() as *const i8, *Bcb as usize);
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatPrepareWriteVolumeFile(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    // C signature is `VBO StartingVbo` (32-bit!). This was translated as i64,
    // while every caller declares+passes u32 — on amd64 the callee then read
    // 32 bits of register GARBAGE in the upper half of the VBO, pinning the
    // virtual-volume file at a wild offset (FAT-corrupting write class).
    StartingVbo: u32,
    ByteCount: u32,
    Bcb: *mut *mut BCB,
    Buffer: *mut *mut core::ffi::c_void,
    Reversible: i32,
    Zero: i32,
) {
    let mut Vbo: LARGE_INTEGER = core::mem::zeroed();
    Vbo.QuadPart = StartingVbo as i64;

    NT_ASSERT!((((StartingVbo as u64) + (ByteCount as u64)) <= FatRootDirectoryLbo(&raw const (*Vcb).Bpb) as u64) as i32);

    DebugTrace!(1, 0, b"FatPrepareWriteVolumeFile\n\0".as_ptr() as *const i8, 0);
    DebugTrace!(0, 0, b"Vcb         = %p\n\0".as_ptr() as *const i8, Vcb as usize);
    DebugTrace!(
        0,
        0,
        b"StartingVbo = %08lx\n\0".as_ptr() as *const i8,
        StartingVbo as usize,
    );
    DebugTrace!(0, 0, b"ByteCount   = %08lx\n\0".as_ptr() as *const i8, ByteCount as usize);
    DebugTrace!(0, 0, b"Zero        = %08lx\n\0".as_ptr() as *const i8, Zero as usize);

    if CcPinRead(
        (*Vcb).VirtualVolumeFile,
        &mut Vbo,
        ByteCount,
        BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) as u32,
        Bcb as *mut *mut c_void,
        Buffer,
    ) == 0
    {
        NT_ASSERT!((FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) == 0) as i32);
        FatRaiseStatus(IrpContext, STATUS_CANT_WAIT);
    }

    (*IrpContext).PinCount += 1;
    DbgDoit!(((*IrpContext).PinCount) as u32);

    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        if Zero != 0 {
            RtlZeroMemory(*Buffer, ByteCount as usize);
        }
        FatSetDirtyBcb(IrpContext, *Bcb as *mut c_void, Vcb, Reversible as u8);
    }, |_, abnormal| {
        if abnormal {
            FatUnpinBcb(IrpContext, Bcb as *mut *mut core::ffi::c_void);
        }
    });

    DebugTrace!(
        -1i32,
        0u32,
        b"FatPrepareWriteVolumeFile -> VOID, *Bcb = %p\n\0".as_ptr() as *const i8,
        *Bcb as usize,
    );
}
#[no_mangle]
pub unsafe extern "system" fn FatReadDirectoryFile(
    IrpContext: *mut IRP_CONTEXT,
    Dcb: *mut DCB,
    StartingVbo: VBO,
    ByteCount: ULONG,
    Pin: i32,
    Bcb: *mut *mut BCB,
    Buffer: *mut *mut core::ffi::c_void,
    Status: *mut NTSTATUS,
) {
    let mut ByteCount = ByteCount;
    let mut Vbo: ntkrnl_sys::ddk::LARGE_INTEGER = core::mem::zeroed();

    PAGED_CODE!();

    DebugTrace!(1, Dbg, b"FatReadDirectoryFile\n\0".as_ptr() as *const i8, 0);
    DebugTrace!(0, Dbg, b"Dcb         = %p\n\0".as_ptr() as *const i8, Dcb as usize);
    DebugTrace!(0, Dbg, b"StartingVbo = %08lx\n\0".as_ptr() as *const i8, StartingVbo as usize);
    DebugTrace!(0, Dbg, b"ByteCount   = %08lx\n\0".as_ptr() as *const i8, ByteCount as usize);

    if ByteCount == 0 {
        DebugTrace!(0, Dbg, b"Nothing to read\n\0".as_ptr() as *const i8, 0);

        *Bcb = core::ptr::null_mut::<BCB>();
        *Buffer = core::ptr::null_mut::<core::ffi::c_void>();
        *Status = ntkrnl_sys::base::STATUS_SUCCESS;

        DebugTrace!(-1, Dbg, b"FatReadDirectoryFile -> VOID\n\0".as_ptr() as *const i8, 0);
        return;
    }

    FatOpenDirectoryFile(IrpContext, Dcb);

    if StartingVbo >= (*Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart {
        DebugTrace!(0, Dbg, b"End of file read for directory\n\0".as_ptr() as *const i8, 0);

        *Bcb = core::ptr::null_mut::<BCB>();
        *Buffer = core::ptr::null_mut::<core::ffi::c_void>();
        *Status = STATUS_END_OF_FILE;

        DebugTrace!(-1, Dbg, b"FatReadDirectoryFile -> VOID\n\0".as_ptr() as *const i8, 0);
        return;
    }

    let diff = (*Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart.wrapping_sub(StartingVbo);
    if diff < ByteCount {
        ByteCount = diff;
    } else {
        ByteCount = ByteCount;
    }

    NT_ASSERT!((ByteCount != 0) as i32);

    Vbo.QuadPart = StartingVbo as i64;

    if if Pin != 0 {
        CcPinRead(
            (*Dcb).Specific.Dcb.as_ref().DirectoryFile,
            &mut Vbo,
            ByteCount,
            BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) as u32,
            Bcb as *mut *mut c_void,
            Buffer,
        )
    } else {
        CcMapData(
            (*Dcb).Specific.Dcb.as_ref().DirectoryFile,
            &mut Vbo,
            ByteCount,
            BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) as u32,
            Bcb as *mut *mut c_void,
            Buffer,
        )
    } == 0
    {
        *Bcb = core::ptr::null_mut::<BCB>();
        *Buffer = core::ptr::null_mut::<core::ffi::c_void>();
        FatRaiseStatus(IrpContext, STATUS_CANT_WAIT);
    }

    {
        let new_pincount = {
            (*IrpContext).PinCount += 1;
            (*IrpContext).PinCount
        };
        DbgDoit!(new_pincount as u32);
    }

    *Status = ntkrnl_sys::base::STATUS_SUCCESS;

    DebugTrace!(-1, Dbg, b"FatReadDirectoryFile -> VOID, *BCB = %p\n\0".as_ptr() as *const i8, *Bcb as usize);

    return;
}

#[no_mangle]
pub unsafe extern "system" fn FatPrepareWriteDirectoryFile(
    IrpContextArg: PIRP_CONTEXT,
    DcbArg: PDCB,
    StartingVbo: u32,      // VBO
    ByteCount: u32,
    Bcb: *mut PBCB,
    Buffer: *mut PVOID,
    Zero: u8,
    Reversible: u8,
    Status: *mut NTSTATUS,
) {
    // C prototype takes single pointers (PIRP_CONTEXT, PDCB); all callers pass single
    // pointers. This body was mistranslated with an extra indirection (*IrpContext,
    // *Dcb, **Dcb), so a single Dcb was dereferenced as **Dcb → the DCB's first 8 bytes
    // were used as a pointer → garbage Dcb into FatOpenDirectoryFile → BUGCHECK 0x1E.
    // Rebind the body's double-pointer names to the addresses of the single-ptr args so
    // the translated body is unchanged and correct. (Bcb/Buffer remain genuine outs.)
    let mut IrpContextLocal = IrpContextArg;
    let mut DcbLocal = DcbArg;
    let IrpContext: *mut PIRP_CONTEXT = &mut IrpContextLocal;
    let Dcb: *mut PDCB = &mut DcbLocal;
    // PAGED_CODE();
    DebugTrace!(
        1,
        Dbg,
        b"FatPrepareWriteDirectoryFile\n\0".as_ptr() as *const i8,
        0,
    );
    DebugTrace!(
        0,
        Dbg,
        b"Dcb         = %p\n\0".as_ptr() as *const i8,
        Dcb as usize,
    );
    DebugTrace!(
        0,
        Dbg,
        b"StartingVbo = %08lx\n\0".as_ptr() as *const i8,
        StartingVbo as usize,
    );
    DebugTrace!(
        0,
        Dbg,
        b"ByteCount   = %08lx\n\0".as_ptr() as *const i8,
        ByteCount as usize,
    );
    DebugTrace!(
        0,
        Dbg,
        b"Zero        = %08lx\n\0".as_ptr() as *const i8,
        Zero as usize,
    );

    *Bcb = core::ptr::null_mut();
    *Buffer = core::ptr::null_mut();

    FatOpenDirectoryFile(*IrpContext, *Dcb);

    let mut vbo: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    vbo.QuadPart = StartingVbo as i64;
    let mut initial_allocation: u32 = 0;
    let mut unwind_we_allocated_disk_space: u8 = 0;
    let mut local_bcb: PBCB = core::ptr::null_mut();
    let mut local_buffer: PVOID = core::ptr::null_mut();
    let initial_request: u32 = ByteCount;
    let mut mapping_granularity: u32 = (PAGE_SIZE) as u32;
    let mut byte_count: u32 = ByteCount;
    let mut zero_flag: u8 = Zero;

    // _SEH2_TRY / _SEH2_FINALLY
    let p_local_bcb = core::ptr::addr_of_mut!(local_bcb);
    let p_unwind_alloc = core::ptr::addr_of_mut!(unwind_we_allocated_disk_space);
    let p_initial_allocation = core::ptr::addr_of_mut!(initial_allocation);
    crate::seh::try_finally(&mut (), |_| {
        if (StartingVbo as i64 + byte_count as i64) > (**Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart as i64 {
            if NodeType!((*Dcb).cast()) == (FAT_NTC_ROOT_DCB as u16) && FatIsFat32((**Dcb).Vcb) == 0 {
                FatRaiseStatus(*IrpContext, STATUS_DISK_FULL);
            }

            DebugTrace!(
                0,
                Dbg,
                b"Try extending normal directory\n\0".as_ptr() as *const i8,
                0,
            );

            *p_initial_allocation = (**Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart;

            FatAddFileAllocation(
                *IrpContext,
                *Dcb,
                (**Dcb).Specific.Dcb.as_ref().DirectoryFile,
                StartingVbo + byte_count,
            );

            *p_unwind_alloc = 1;

            (**Dcb).Header.FileSize.__bindgen_anon_1.LowPart = (**Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart;

            CcSetFileSizes(
                (**Dcb).Specific.Dcb.as_ref().DirectoryFile,
                core::ptr::addr_of!((**Dcb).Header.AllocationSize) as *const CC_FILE_SIZES as PCC_FILE_SIZES,
            );

            FatCheckFreeDirentBitmap(*IrpContext, *Dcb);

            zero_flag = 1;
            vbo.QuadPart = *p_initial_allocation as i64;
            byte_count = (**Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart - *p_initial_allocation;
        }

        while byte_count > 0 {
            let bytes_to_pin: u32;

            *p_local_bcb = core::ptr::null_mut();

            if vbo.QuadPart > (StartingVbo as i64 + initial_request as i64) {
                mapping_granularity = VACB_MAPPING_GRANULARITY;
            }

            if (vbo.QuadPart / mapping_granularity as i64)
                == ((vbo.QuadPart + byte_count as i64 - 1) / mapping_granularity as i64)
            {
                bytes_to_pin = byte_count;
            } else {
                bytes_to_pin =
                    mapping_granularity - ((vbo.QuadPart as u32) & (mapping_granularity - 1));
            }

            if CcPinRead(
                (**Dcb).Specific.Dcb.as_ref().DirectoryFile,
                &mut vbo,
                bytes_to_pin,
                BooleanFlagOn((**IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) as u32,
                p_local_bcb,
                &mut local_buffer,
            ) == 0 {
                FatRaiseStatus(*IrpContext, STATUS_CANT_WAIT);
            }

            if (*Buffer).is_null() {
                *Buffer = local_buffer;
                *Bcb = *p_local_bcb;
            }

            (**IrpContext).PinCount += 1;
            DbgDoit!((**IrpContext).PinCount as u32);

            if zero_flag != 0 {
                core::ptr::write_bytes(local_buffer as *mut u8, 0, bytes_to_pin as usize);
                CcSetDirtyPinnedData(*p_local_bcb, core::ptr::null_mut());
            }

            byte_count -= bytes_to_pin;
            vbo.QuadPart += bytes_to_pin as i64;

            if *Bcb != *p_local_bcb {
                FatRepinBcb(*IrpContext, *p_local_bcb);
                FatUnpinBcb(*IrpContext, p_local_bcb as *mut *mut core::ffi::c_void);
            }
        }

        FatSetDirtyBcb(*IrpContext, *Bcb, (**Dcb).Vcb, Reversible);
        *Status = STATUS_SUCCESS as i32;
    }, |_, abnormal| {
        DebugUnwind!(b"FatPrepareWriteDirectoryFile\0".as_ptr() as *const i8);

        if abnormal {
            if *Bcb != *p_local_bcb {
                FatUnpinBcb(*IrpContext, p_local_bcb as *mut *mut core::ffi::c_void);
            }
            FatUnpinBcb(*IrpContext, Bcb as *mut *mut core::ffi::c_void);

            if *p_unwind_alloc != 0 {
                FatTruncateFileAllocation(*IrpContext, *Dcb, *p_initial_allocation);
                (**Dcb).Header.FileSize.__bindgen_anon_1.LowPart = (**Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart;
                CcSetFileSizes(
                    (**Dcb).Specific.Dcb.as_ref().DirectoryFile,
                    core::ptr::addr_of!((**Dcb).Header.AllocationSize) as *const CC_FILE_SIZES as PCC_FILE_SIZES,
                );
            }
        }
    });

    DebugTrace!(
        -1,
        Dbg,
        b"FatPrepareWriteDirectoryFile -> (VOID), *Bcb = %p\n\0".as_ptr() as *const i8,
        *Bcb as usize,
    );
}

#[no_mangle]
//  The C takes `PIRP_CONTEXT` and `PDCB` — one level of indirection each, not two.
//  The body was written half to each signature (`(**dcb).Vcb` but `(*dcb).cast()`),
//  which is what a spurious extra `*` looks like once it propagates.
pub unsafe extern "system" fn FatIsCurrentOperationSynchedForDcbTeardown(
    irp_context: PIRP_CONTEXT,
    dcb: PDCB,
) -> i32 {
    let irp = (*irp_context).OriginatingIrp;
    let stack = IoGetCurrentIrpStackLocation(irp);
    let mut vcb: *mut VCB = core::ptr::null_mut();
    let mut fcb: *mut FCB = core::ptr::null_mut();
    let mut ccb: *mut CCB = core::ptr::null_mut();

    let mut to_check: [*mut FILE_OBJECT; 3] = [core::ptr::null_mut(); 3];
    let mut index: u32 = 0;

    PagedCode();

    // While mounting, we're OK without having to own anything.
    if (*stack).MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL
        && (*stack).MinorFunction == (IRP_MN_MOUNT_VOLUME) as u8
    {
        return 1;
    }

    // With the Vcb held, the close path is blocked out.
    if ExIsResourceAcquiredSharedLite(
        core::ptr::addr_of!((*(*dcb).Vcb).Resource)
    ) != 0
        || ExIsResourceAcquiredExclusiveLite(
            core::ptr::addr_of_mut!((*(*dcb).Vcb).Resource)
        ) != 0
    {
        return 1;
    }

    // Accept this assertion at face value.  It comes from GetDirentForFcbOrDcb,
    // and is reliable.
    if FlagOn((*irp_context).Flags as u32, IRP_CONTEXT_FLAG_PARENT_BY_CHILD as u32) != 0 {
        return 1;
    }

    // Determine which fileobjects are around on this operation.
    if (*stack).MajorFunction == IRP_MJ_SET_INFORMATION
        && !(*stack).Parameters.SetFile.FileObject.is_null()
    {
        to_check[index as usize] = (*stack).Parameters.SetFile.FileObject;
        index += 1;
    }

    if !(*stack).FileObject.is_null() {
        to_check[index as usize] = (*stack).FileObject;
        index += 1;
    }

    to_check[index as usize] = core::ptr::null_mut();

    // If the fileobjects we have are for this dcb or a child of it, we are
    // also guaranteed that this dcb isn't going anywhere (even without
    // the Vcb).
    index = 0;
    while !to_check[index as usize].is_null() {
        FatDecodeFileObject(to_check[index as usize], &mut vcb, &mut fcb, &mut ccb);

        while !fcb.is_null() {
            if fcb == dcb.cast() {
                return 1;
            }
            fcb = (*fcb).ParentDcb;
        }

        index += 1;
    }

    return FatDisableParentCheck;
}


#[no_mangle]
pub unsafe extern "system" fn FatOpenDirectoryFile(
    IrpContext: *mut IRP_CONTEXT,
    Dcb: *mut DCB,
) {
    // PAGED_CODE(); ignored
    let mut IrpContext = IrpContext;
    let mut Dcb = Dcb;

    DebugTrace!(1, Dbg, b"FatOpenDirectoryFile\0".as_ptr() as *const i8, 0);
    DebugTrace!(0, Dbg, b"Dcb = %p\0".as_ptr() as *const i8, Dcb as usize);

    // C: NT_ASSERT( FatIsCurrentOperationSynchedForDcbTeardown( IrpContext, Dcb )).
    // NT_ASSERT does NOT evaluate its argument in release/DUMMY_PSEH builds (it is a
    // checked-build-only invariant). The Rust NT_ASSERT used to be an extern fn that
    // evaluated it eagerly, and NT_ASSERT! still would under debug-assertions.
    // FatIsCurrentOperationSynchedForDcbTeardown reads the create
    // IRP's current stack location, which faults when that location has been consumed
    // during a nested directory-file open (e.g. FatCreateNewFile). The result is
    // discarded by the assert, so match C release semantics and do not evaluate it.
    let _ = FatIsCurrentOperationSynchedForDcbTeardown; // keep the translation referenced

    if (*Dcb).Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT {
        FatLookupFileAllocationSize(IrpContext, Dcb);
        (*Dcb).Header.FileSize.__bindgen_anon_1.LowPart = (*Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart;
    }

    FatCheckFreeDirentBitmap(IrpContext, Dcb);

    if (*Dcb).Specific.Dcb.as_ref().DirectoryFile.is_null() {
        let mut DirectoryFileObject: *mut FILE_OBJECT = core::ptr::null_mut();
        FatAcquireDirectoryFileMutex((*Dcb).Vcb);

        // _SEH2_TRY / _SEH2_FINALLY
        let p_dfo = core::ptr::addr_of_mut!(DirectoryFileObject);
        crate::seh::try_finally(&mut (), |_| {
            if (*Dcb).Specific.Dcb.as_ref().DirectoryFile.is_null() {
                let RealDevice: *mut DEVICE_OBJECT = (*(*Dcb).Vcb).CurrentDevice;

                *p_dfo = IoCreateStreamFileObject(core::ptr::null_mut(), RealDevice);
                FatPreallocateCloseContext((*Dcb).Vcb);

                FatSetFileObject(
                    *p_dfo,
                    DirectoryFile,
                    Dcb as *mut c_void,
                    core::ptr::null_mut(),
                );

                // InterlockedIncrement on InternalOpenCount
                InterlockedIncrement(
                    &raw mut (*(*Dcb).Vcb).InternalOpenCount as *mut i32,
                );

                if NodeType!((Dcb).cast()) == (FAT_NTC_ROOT_DCB as u16) {
                    InterlockedIncrement(
                        &raw mut (*(*Dcb).Vcb).ResidualOpenCount as *mut i32,
                    );
                }

                (**p_dfo).SectionObjectPointer =
                    &raw mut (*(*Dcb).NonPaged).SectionObjectPointers as *mut _;

                (**p_dfo).ReadAccess = 1;
                (**p_dfo).WriteAccess = 1;
                (**p_dfo).DeleteAccess = 1;

                InterlockedIncrement(
                    &raw mut (*Dcb).Specific.Dcb.as_mut().DirectoryFileOpenCount as *mut i32,
                );

                (*Dcb).Specific.Dcb.as_mut().DirectoryFile = *p_dfo;
                // Success - clear the pointer so finally doesn't dereference
                *p_dfo = core::ptr::null_mut();
            }
        }, |_, _abnormal| {
            FatReleaseDirectoryFileMutex((*Dcb).Vcb);
            if !(*p_dfo).is_null() {
                ObDereferenceObject(*p_dfo as *mut core::ffi::c_void);
            }
        });
    }

    if (*(*Dcb).Specific.Dcb.as_ref().DirectoryFile).PrivateCacheMap.is_null() {
        (*Dcb).Header.ValidDataLength = FatMaxLarge;
        (*Dcb).ValidDataToDisk = ntkrnl_sys::constants::MAXULONG;

        FatInitializeCacheMap(
            (*Dcb).Specific.Dcb.as_ref().DirectoryFile,
            &raw const (*Dcb).Header.AllocationSize as *const CC_FILE_SIZES as *mut CC_FILE_SIZES,
            1,
            &raw const FatData.CacheManagerNoOpCallbacks as *const CACHE_MANAGER_CALLBACKS as *mut CACHE_MANAGER_CALLBACKS,
            (*Dcb).Vcb as *mut c_void,
        );
    }

    DebugTrace!(-1, Dbg, b"FatOpenDirectoryFile -> VOID\0".as_ptr() as *const i8, 0);
}

#[no_mangle]
pub unsafe extern "system" fn FatOpenEaFile(
    IrpContext: *mut IRP_CONTEXT,
    EaFcb: *mut FCB,
) -> *mut FILE_OBJECT {
    let mut EaFileObject: *mut FILE_OBJECT = core::ptr::null_mut();
    let RealDevice: *mut DEVICE_OBJECT;

    // PAGED_CODE();

    DebugTrace!(1, 0, b"FatOpenEaFile\n\0".as_ptr() as *const i8, 0);
    DebugTrace!(0, 0, b"EaFcb = %p\n\0".as_ptr() as *const i8, EaFcb as usize);

    RealDevice = (*(*EaFcb).Vcb).CurrentDevice;

    EaFileObject = IoCreateStreamFileObject(core::ptr::null_mut(), RealDevice);

    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        FatPreallocateCloseContext((*IrpContext).Vcb);

        FatSetFileObject(EaFileObject, EaFile, EaFcb as *mut c_void, core::ptr::null_mut());

        InterlockedIncrement(
            core::ptr::addr_of_mut!((*(*EaFcb).Vcb).InternalOpenCount) as *mut i32,
        );
        InterlockedIncrement(
            core::ptr::addr_of_mut!((*(*EaFcb).Vcb).ResidualOpenCount) as *mut i32,
        );

        (*EaFileObject).SectionObjectPointer =
            core::ptr::addr_of_mut!((*(*EaFcb).NonPaged).SectionObjectPointers);
        (*EaFileObject).ReadAccess = 1;
        (*EaFileObject).WriteAccess = 1;

        (*EaFcb).Header.ValidDataLength = FatMaxLarge;

        FatInitializeCacheMap(
            EaFileObject,
            core::ptr::addr_of!((*EaFcb).Header.AllocationSize) as *const CC_FILE_SIZES as *mut CC_FILE_SIZES,
            1,
            // C: &FatData.CacheManagerCallbacks with the EaFcb as the lazy-write context
            // (was mistranslated as the NoOp callbacks + the Vcb).
            core::ptr::addr_of!(FatData.CacheManagerCallbacks) as *const CACHE_MANAGER_CALLBACKS as *mut CACHE_MANAGER_CALLBACKS,
            EaFcb as *mut c_void,
        );

        CcSetAdditionalCacheAttributes(EaFileObject, 1, 1);
    }, |_, abnormal| {
        // _SEH2_FINALLY
        if abnormal {
            ObDereferenceObject(EaFileObject as *mut c_void);
        }
    });
    // _SEH2_END

    DebugTrace!(
        -1,
        0,
        b"FatOpenEaFile -> %p\n\0".as_ptr() as *const i8,
        EaFileObject as usize,
    );

    let _ = IrpContext;

    EaFileObject
}


#[no_mangle]
pub unsafe extern "system" fn FatCloseEaFile(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    FlushFirst: u8,
) {
    // PAGED_CODE();
    PAGED_CODE!();

    // DebugTrace(+1, Dbg, "FatCloseEaFile\n", 0);
    DebugTrace!(
        1i32,
        Dbg,
        b"FatCloseEaFile\n\0".as_ptr() as *const i8,
        0usize,
    );

    // DebugTrace( 0, Dbg, "Vcb = %p\n", Vcb);
    DebugTrace!(
        0i32,
        Dbg,
        b"Vcb = %p\n\0".as_ptr() as *const i8,
        Vcb as usize,
    );

    // NT_ASSERT( FatVcbAcquiredExclusive(IrpContext, Vcb) );
    let cond: i32 = FatVcbAcquiredExclusive(IrpContext, Vcb) as i32;
    NT_ASSERT!(cond);

    // PFILE_OBJECT EaFileObject = Vcb->VirtualEaFile;
    let mut EaFileObject: *mut FILE_OBJECT = (*Vcb).VirtualEaFile;

    // if (EaFileObject != NULL) {
    if !EaFileObject.is_null() {
        // EaFileObject = Vcb->VirtualEaFile;
        EaFileObject = (*Vcb).VirtualEaFile;

        // if (FlushFirst) {
        if FlushFirst != 0 {
            // CcFlushCache( Vcb->VirtualEaFile->SectionObjectPointer, NULL, 0, NULL );
            CcFlushCache(
                (*((*Vcb).VirtualEaFile)).SectionObjectPointer,
                core::ptr::null_mut::<LARGE_INTEGER>(),
                0u32,
                core::ptr::null_mut::<_IO_STATUS_BLOCK>(),
            );
        }

        // Vcb->VirtualEaFile = NULL;
        (*Vcb).VirtualEaFile = core::ptr::null_mut();

        // FatRemoveMcbEntry( Vcb, &Vcb->EaFcb->Mcb, 0, 0xFFFFFFFF );
        FatRemoveMcbEntry(
            Vcb,
            (core::ptr::addr_of_mut!((*((*Vcb).EaFcb)).Mcb) as *mut core::ffi::c_void).cast(),
            0u32,
            0xFFFFFFFFu32,
        );

        // FatSyncUninitializeCacheMap( IrpContext, EaFileObject );
        FatSyncUninitializeCacheMap(IrpContext, EaFileObject);

        // ObDereferenceObject( EaFileObject );
        ObDereferenceObject(EaFileObject as *mut core::ffi::c_void);
    }

    // DebugTrace(-1, Dbg, "FatCloseEaFile -> %p\n", EaFileObject);
    DebugTrace!(
        -1i32,
        Dbg,
        b"FatCloseEaFile -> %p\n\0".as_ptr() as *const i8,
        EaFileObject as usize,
    );
}

const REPINNED_BCBS_ARRAY_SIZE: usize = 4; // REPINNED_BCBS_ARRAY_SIZE
const TAG_REPINNED_BCB: u32 = 0x52746146; // TAG_REPINNED_BCB
#[no_mangle]
pub unsafe extern "system" fn FatRepinBcb(
    IrpContext: *mut IRP_CONTEXT,
    Bcb: *mut core::ffi::c_void,
) {
    // PAGED_CODE();
    DebugTrace!(1, Dbg, b"FatRepinBcb\n\0".as_ptr() as *const i8, 0);
    DebugTrace!(0, Dbg, b"IrpContext = %p\n\0".as_ptr() as *const i8, IrpContext as usize);
    DebugTrace!(0, Dbg, b"Bcb        = %p\n\0".as_ptr() as *const i8, Bcb as usize);

    let mut repinned: *mut REPINNED_BCBS = core::ptr::addr_of_mut!((*IrpContext).Repinned);

    loop {
        for i in 0..REPINNED_BCBS_ARRAY_SIZE {
            let entry = (*repinned).Bcb[i];
            if entry == Bcb {
                DebugTrace!(-1, Dbg, b"FatRepinBcb -> VOID\n\0".as_ptr() as *const i8, 0);
                return;
            }
            if entry.is_null() {
                (*repinned).Bcb[i] = Bcb;
                CcRepinBcb(Bcb);
                DebugTrace!(-1, Dbg, b"FatRepinBcb -> VOID\n\0".as_ptr() as *const i8, 0);
                return;
            }
        }

        if (*repinned).Next.is_null() {
            let new_node = FsRtlAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<REPINNED_BCBS>() as u32,
                TAG_REPINNED_BCB,
            ) as *mut REPINNED_BCBS;
            (*repinned).Next = new_node;
            RtlZeroMemory(
                new_node as *mut core::ffi::c_void,
                core::mem::size_of::<REPINNED_BCBS>(),
            );
        }

        repinned = (*repinned).Next;
    }
}

const IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH: u32 = 0x08; // placeholder
const IRP_CONTEXT_FLAG_WRITE_THROUGH: u32 = 0x04; // placeholder
const DO_VERIFY_VOLUME: u32 = 0x2; // placeholder
const STATUS_VERIFY_REQUIRED: i32 = 0x80000016_u32 as i32;
const FAT_NTC_FCB: u32 = 0x0502; // placeholder, must be correct value

unsafe extern "system" {
}

// REPINNED_BCBS comes from crate::types (types_amd64.rs): _REPINNED_BCBS { Next, Bcb[4] }
#[no_mangle]
pub unsafe extern "system" fn FatUnpinRepinnedBcbs(irp_context: *mut IRP_CONTEXT) {
    // PAGED_CODE();
    DebugTrace!(1, Dbg, b"FatUnpinRepinnedBcbs\n\0".as_ptr() as *const i8, 0);
    DebugTrace!(0, Dbg, b"IrpContext = %p\n\0".as_ptr() as *const i8, (irp_context as i32) as usize);

    let mut raise_iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    raise_iosb.__bindgen_anon_1.Status = (STATUS_SUCCESS) as i32;
    raise_iosb.Information = 0;

    let repinned: *mut REPINNED_BCBS = core::ptr::addr_of_mut!((*irp_context).Repinned);

    let mut file_object: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut force_verify: i32 = 0;
    let mut fcb_or_dcb: *mut FCB = core::ptr::null_mut();

    // Detect file creation for special handling
    if (*irp_context).MajorFunction == IRP_MJ_CREATE && !(*irp_context).OriginatingIrp.is_null() {
        let irp_sp: *mut IO_STACK_LOCATION =
            IoGetCurrentIrpStackLocation((*irp_context).OriginatingIrp);

        if !irp_sp.is_null()
            && !(*irp_sp).FileObject.is_null()
            && !(*(*irp_sp).FileObject).FsContext.is_null()
        {
            fcb_or_dcb = (*(*irp_sp).FileObject).FsContext as *mut FCB;
        }
    }

    // C `!FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH)`. Was
    // `!FlagOn(..) != 0`, a BITWISE not of the flag word that is never zero, so
    // DISABLE_WRITE_THROUGH was ignored.
    let write_through_to_disk: i32 = (FlagOn(
        (*irp_context).Flags,
        IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH,
    ) == 0
        && !(*irp_context).Vcb.is_null()
        && (FlagOn((*irp_context).Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH) != 0
            || FlagOn(
                (*(*irp_context).Vcb).VcbState,
                VCB_STATE_FLAG_DEFERRED_FLUSH,
            ) != 0)) as i32;

    let mut repinned_ptr: *mut REPINNED_BCBS = repinned;

    while !repinned_ptr.is_null() {
        let mut i: usize = 0;
        while i < REPINNED_BCBS_ARRAY_SIZE {
            let bcb = (*repinned_ptr).Bcb[i];
            if !bcb.is_null() {
                let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();

                if write_through_to_disk != 0
                    && FlagOn(
                        (*(*irp_context).Vcb).VcbState,
                        VCB_STATE_FLAG_DEFERRED_FLUSH,
                    ) != 0
                {
                    file_object = CcGetFileObjectFromBcb(bcb);
                }

                CcUnpinRepinnedBcb(bcb, (write_through_to_disk) as u8, &mut iosb);

                if !NT_SUCCESS(iosb.__bindgen_anon_1.Status) {
                    if raise_iosb.__bindgen_anon_1.Status == (STATUS_SUCCESS) as i32 {
                        raise_iosb = iosb;
                    }

                    if !file_object.is_null()
                        && (*irp_context).MajorFunction != IRP_MJ_CLEANUP
                        && (*irp_context).MajorFunction != IRP_MJ_FLUSH_BUFFERS
                        && (*irp_context).MajorFunction != IRP_MJ_SET_INFORMATION
                        && !((*irp_context).MajorFunction == IRP_MJ_CREATE
                            && iosb.__bindgen_anon_1.Status == STATUS_VERIFY_REQUIRED
                            && !fcb_or_dcb.is_null()
                            && (NodeType!((fcb_or_dcb).cast()) == (FAT_NTC_FCB as u16)))
                    {
                        let mut k: usize = i + 1;
                        let mut repinned_to_purge: *mut REPINNED_BCBS = repinned_ptr;

                        while !repinned_to_purge.is_null() {
                            let mut j: usize = k;
                            while j < REPINNED_BCBS_ARRAY_SIZE {
                                let bcb2 = (*repinned_to_purge).Bcb[j];
                                if !bcb2.is_null() {
                                    if CcGetFileObjectFromBcb(bcb2) == file_object {
                                        CcUnpinRepinnedBcb(bcb2, 0, &mut iosb);
                                        (*repinned_to_purge).Bcb[j] = core::ptr::null_mut();
                                    }
                                }
                                j += 1;
                            }
                            repinned_to_purge = (*repinned_to_purge).Next;
                            k = 0;
                        }

                        CcPurgeCacheSection(
                            (*file_object).SectionObjectPointer,
                            core::ptr::null_mut(),
                            0u32,
                            0u8,
                        );
                        force_verify = 1;
                    }
                }

                (*repinned_ptr).Bcb[i] = core::ptr::null_mut();
            }
            i += 1;
        }

        if repinned_ptr != repinned {
            let saved: *mut REPINNED_BCBS = (*repinned_ptr).Next;
            ExFreePool(repinned_ptr as *mut core::ffi::c_void);
            repinned_ptr = saved;
        } else {
            repinned_ptr = (*repinned_ptr).Next;
            (*irp_context).Repinned.Next = core::ptr::null_mut();
        }
    }

    if !NT_SUCCESS(raise_iosb.__bindgen_anon_1.Status) {
        if force_verify != 0 && !file_object.is_null() {
            SetFlag!((*(*file_object).DeviceObject).Flags, DO_VERIFY_VOLUME);
            IoSetHardErrorOrVerifyDevice(
                (*irp_context).OriginatingIrp,
                (*file_object).DeviceObject,
            );
        }

        if FlagOn((*irp_context).Flags, IRP_CONTEXT_FLAG_DISABLE_RAISE) == 0 {
            if !(*irp_context).OriginatingIrp.is_null() {
                (*(*irp_context).OriginatingIrp).IoStatus = raise_iosb;
            }
            FatNormalizeAndRaiseStatus(irp_context, raise_iosb.__bindgen_anon_1.Status);
        }
    }

    DebugTrace!(-1, Dbg, b"FatUnpinRepinnedBcbs -> VOID\n\0".as_ptr() as *const i8, 0);
}


#[no_mangle]
pub unsafe extern "system" fn FatZeroData(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    FileObject: *mut FILE_OBJECT,
    StartingZero: u32,
    ByteCount: u32,
) -> u8 {
    let sector_size: u32;

    let mut zero_start: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    let mut beyond_zero_end: LARGE_INTEGER = unsafe { core::mem::zeroed() };

    let finished: u8;

    unsafe {
        PAGED_CODE!();
    }

    sector_size = (*Vcb).Bpb.BytesPerSector as u32;

    zero_start.__bindgen_anon_1.LowPart = (StartingZero + (sector_size - 1)) & !(sector_size - 1);

    //
    //  Detect overflow if we were asked to zero in the last sector of the file,
    //  which must be "zeroed" already (or we're in trouble).
    //

    if StartingZero != 0 && zero_start.__bindgen_anon_1.LowPart == 0 {
        return 1;
    }

    //
    //  Note that BeyondZeroEnd can take the value 4gb.
    //

    beyond_zero_end.QuadPart = (((StartingZero as u64 + ByteCount as u64 + (sector_size as u64 - 1))
                                 & !((sector_size as u64) - 1))) as i64;

    //
    //  If we were called to just zero part of a sector we are in trouble.
    //

    if zero_start.QuadPart == beyond_zero_end.QuadPart {
        return 1;
    }

    finished = CcZeroData(
        FileObject,
        core::ptr::addr_of_mut!(zero_start),
        core::ptr::addr_of_mut!(beyond_zero_end),
        BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) as u8,
    );

    finished
}
#[no_mangle]
pub unsafe extern "system" fn FatCompleteMdl(
    IrpContext: *mut IRP_CONTEXT,
    Irp: *mut IRP,
) -> NTSTATUS {
    // PAGED_CODE(); // pageable code marker, ignored

    DebugTrace!(1, 0 /* Dbg (DEBUG_TRACE_CACHESUP) */, b"FatCompleteMdl\n\0".as_ptr() as *const i8, 0);
    DebugTrace!(0, 0 /* Dbg */, b"IrpContext = %p\n\0".as_ptr() as *const i8, IrpContext as usize);
    DebugTrace!(0, 0 /* Dbg */, b"Irp        = %p\n\0".as_ptr() as *const i8, Irp as usize);

    // Do completion processing.
    let FileObject = (*(IoGetCurrentIrpStackLocation(Irp))).FileObject;

    match (*IrpContext).MajorFunction {
        IRP_MJ_READ => {
            CcMdlReadComplete(FileObject, (*Irp).MdlAddress);
        },
        IRP_MJ_WRITE => {
            let IrpSp = IoGetCurrentIrpStackLocation(Irp);
            NT_ASSERT!(FlagOn((*IrpContext).Flags as u32, 0x2 /* IRP_CONTEXT_FLAG_WAIT */) as i32);
            CcMdlWriteComplete(
                FileObject,
                core::ptr::addr_of_mut!((*IrpSp).Parameters.Write.ByteOffset),
                (*Irp).MdlAddress,
            );
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS as i32;
        },
        _ => {
            DebugTrace!(0 /* DEBUG_TRACE_ERROR */, 0, b"Illegal Mdl Complete.\n\0".as_ptr() as *const i8, 0);
            // #pragma prefast suppress omitted
            FatBugCheck(((*IrpContext).MajorFunction) as ULONG_PTR, 0, 0);
        }
    }

    // Mdl is now deallocated.
    (*Irp).MdlAddress = core::ptr::null_mut();

    // Complete the request and exit right away.
    FatCompleteRequest(IrpContext, Irp, (STATUS_SUCCESS) as i32);

    DebugTrace!(-1, 0 /* Dbg */, b"FatCompleteMdl -> STATUS_SUCCESS\n\0".as_ptr() as *const i8, 0);

    return (STATUS_SUCCESS) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn FatSyncUninitializeCacheMap(
    IrpContext: PIRP_CONTEXT,
    FileObject: PFILE_OBJECT,
) {
    // UNREFERENCED_PARAMETER
    _ = IrpContext;

    // Local variables
    let mut uninitialize_complete_event: CACHE_UNINITIALIZE_EVENT = core::mem::zeroed();
    let wait_status: NTSTATUS;

    // PAGED_CODE();  (platform assertion macro, elided for mechanical translation)

    KeInitializeEvent(
        core::ptr::addr_of_mut!(uninitialize_complete_event.Event),
        _EVENT_TYPE_SynchronizationEvent,
        0u8, // FALSE
    );

    CcUninitializeCacheMap(
        FileObject,
        core::ptr::addr_of!(FatLargeZero) as *mut LARGE_INTEGER,
        &mut uninitialize_complete_event as *mut CACHE_UNINITIALIZE_EVENT,
    );

    wait_status = KeWaitForSingleObject(
        &mut uninitialize_complete_event.Event as *mut KEVENT as *mut c_void,
        _KWAIT_REASON_Executive,
        0i8, // KernelMode
        0u8, // FALSE
        core::ptr::null_mut(), // NULL
    );

    NT_ASSERT!((wait_status == (STATUS_SUCCESS) as i32) as i32);
}

#[no_mangle]
pub unsafe extern "system" fn FatPinMappedData(
    IrpContext: PIRP_CONTEXT,
    Dcb: PDCB,
    StartingVbo: VBO,
    ByteCount: ULONG,
    Bcb: *mut *mut c_void,
) {
    // PAGED_CODE();
    let mut Vbo: LARGE_INTEGER = core::mem::zeroed();
    Vbo.QuadPart = StartingVbo as i64;

    DebugTrace!(
        1,
        Dbg,
        b"FatPinMappedData\n\0".as_ptr() as *const i8,
        0,
    );
    DebugTrace!(0, Dbg, b"Dcb         = %p\n\0".as_ptr() as *const i8, Dcb as usize);
    DebugTrace!(
        0,
        Dbg,
        b"StartingVbo = %08lx\n\0".as_ptr() as *const i8,
        (StartingVbo) as usize,
    );
    DebugTrace!(
        0,
        Dbg,
        b"ByteCount   = %08lx\n\0".as_ptr() as *const i8,
        (ByteCount) as usize,
    );

        if CcPinMappedData(
        (*Dcb).Specific.Dcb.as_ref().DirectoryFile,
        &mut Vbo,
        ByteCount,
        BooleanFlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_WAIT) as u32,
        Bcb,
    ) == 0
    {
        FatRaiseStatus(IrpContext, STATUS_CANT_WAIT);
    }

    DebugTrace!(
        -1,
        Dbg,
        b"FatReadDirectoryFile -> VOID, *BCB = %p\n\0".as_ptr() as *const i8,
        *Bcb as usize,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FatPrefetchPages(
    IrpContext: *mut IRP_CONTEXT,
    FileObject: *mut FILE_OBJECT,
    StartingPage: u32,
    PageCount: u32,
) -> NTSTATUS {
    let mut read_list: *mut READ_LIST = core::ptr::null_mut();
    let mut priority_information: _IO_PRIORITY_INFO = core::mem::zeroed();
    let mut prefetch_flags: u32;
    let mut page_no: u32;

    // UNREFERENCED_PARAMETER(IrpContext);
    let _ = IrpContext;

    // PAGED_CODE();

    if PageCount == 0 {
        return (STATUS_SUCCESS) as i32;
    }

    // ASSERT(PageCount < (PFN_NUMBER)MAXULONG);
    ntkrnl_sys::ASSERT!(PageCount < MAXULONG as u32);

    IoInitializePriorityInfo(&mut priority_information);

    let originating_irp = (*IrpContext).OriginatingIrp;

    let status = 'cleanup: loop {
        let status_local = IoRetrievePriorityInfo(
            originating_irp,
            FileObject,
            (*originating_irp).Tail.Overlay.Thread as *mut core::ffi::c_void,
            &mut priority_information,
        );

        if !NT_SUCCESS(status_local) {
            break status_local;
        }

        read_list = ExAllocatePoolWithTag(
            PagedPool,
            ((core::mem::offset_of!(READ_LIST, List) + PageCount as usize * core::mem::size_of::<FILE_SEGMENT_ELEMENT>()) as u64) as usize,
            u32::from_ne_bytes([b' ', b't', b'a', b'F']),
        ) as *mut READ_LIST;

        if read_list.is_null() {
            break STATUS_INSUFFICIENT_RESOURCES;
        }

        (*read_list).FileObject = FileObject;
        (*read_list).IsImage = 0; // FALSE
        (*read_list).NumberOfEntries = PageCount;

        // Build PrefetchFlags bitfield equivalent
        prefetch_flags = 0;
        // PrefetchFlags.Flags.Priority = PriorityInformation.PagePriority;
        prefetch_flags |= (priority_information.PagePriority as u32 & 0x7) << 0;
        // PrefetchFlags.Flags.RepurposePriority = SYSTEM_PAGE_PRIORITY_LEVELS - 1;
        prefetch_flags |= ((SYSTEM_PAGE_PRIORITY_LEVELS - 1) as u32 & 0x7) << 3;
        // PrefetchFlags.Flags.PriorityProtection = 1;
        prefetch_flags |= 1u32 << 6;

        // READ_LIST.List is a C flexible-array member (List[ANYSIZE_ARRAY=1]); the
        // allocation above is sized for PageCount entries. Index it via raw pointer
        // arithmetic — array indexing would hit Rust's bounds check (compile-time len 1)
        // and panic_bounds_check for any PageCount > 1.
        let list = core::ptr::addr_of_mut!((*read_list).List) as *mut FILE_SEGMENT_ELEMENT;
        (*list).Alignment = (StartingPage as u64) * (PAGE_SIZE as u64);
        (*list).Alignment |= prefetch_flags as u64;

        page_no = 1;
        while page_no < PageCount {
            (*list.add(page_no as usize)).Alignment =
                (*list.add((page_no - 1) as usize)).Alignment + (PAGE_SIZE as u64);
            page_no += 1;
        }

        break MmPrefetchPages(1u32, &mut read_list);
    };

    // Cleanup
    if !read_list.is_null() {
        ExFreePoolWithTag(read_list as *mut core::ffi::c_void, u32::from_ne_bytes([b' ', b't', b'a', b'F']));
    }

    status
}




// ==== re-stitched (genuine drop, authoritative) ====
// extern "system": no_mangle WITHOUT an extern ABI keeps the (unstable) Rust CC
// while exporting the raw symbol — the crate's 8 extern "system" decls then fold
// their i386 call sites to ud2 (2026-07-18 fastfat bomb set, FatSetFileSizeInDirent
// & friends). The decl consensus is the ABI contract; match it.
#[no_mangle]
pub unsafe extern "system" fn FatSetDirtyBcb(
    mut IrpContext: *mut IRP_CONTEXT,
    Bcb: *mut core::ffi::c_void,
    Vcb: *mut VCB,
    Reversible: u8,
) {
    const IRP_CONTEXT_FLAG_DISABLE_DIRTY: u32 = 0x01; // fatstruc.h: was wrongly 0x100 (placeholder) → guard never fired → FatSetDirtyBcb↔FatMarkVolume↔FatSetFatEntry↔FatPrepareWriteVolumeFile infinite recursion → arm64 stack overflow 0x50
    const VCB_STATE_FLAG_DEFERRED_FLUSH: u32 = 0x1000;
    const VCB_STATE_FLAG_VOLUME_DIRTY: u32 = 0x04;
    const VCB_STATE_FLAG_MOUNTED_DIRTY: u32 = 0x10;
    const VCB_STATE_FLAG_REMOVABLE_MEDIA: u32 = 0x02;
    const VolumeDirty: u32 = 1;
    const Dbg: u32 = 0;
    // DebugTrace calls preserved
    DebugTrace!(
        1,
        Dbg,
        b"FatSetDirtyBcb\n\0".as_ptr() as *const i8,
        0usize,
    );
    DebugTrace!(
        0,
        Dbg,
        b"IrpContext = %p\n\0".as_ptr() as *const i8,
        IrpContext as usize,
    );
    DebugTrace!(
        0,
        Dbg,
        b"Bcb        = %p\n\0".as_ptr() as *const i8,
        Bcb as usize,
    );
    DebugTrace!(
        0,
        Dbg,
        b"Vcb        = %p\n\0".as_ptr() as *const i8,
        Vcb as usize,
    );

    // Repin the bcb as required
    if Reversible != 0 {
        FatRepinBcb(IrpContext, Bcb);
    }

    // Set the bcb dirty
    CcSetDirtyPinnedData(Bcb, core::ptr::null_mut());

    // If volume dirtying isn't disabled...
    if ((*IrpContext).Flags & IRP_CONTEXT_FLAG_DISABLE_DIRTY) == 0
        && !Vcb.is_null()
        && FatIsFat12(Vcb) == 0
    {
        let mut SavedIrql: u8 = 0;
        let mut SetTimer: u8;

        let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();
        let mut TimeSincePreviousCall: LARGE_INTEGER = core::mem::zeroed();

        // "Borrow" the irp context spinlock
        KeQuerySystemTime(core::ptr::addr_of_mut!(CurrentTime));

        KeAcquireSpinLock(
            core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(FatData)).GeneralSpinLock),
            core::ptr::addr_of_mut!(SavedIrql),
        );

        TimeSincePreviousCall.QuadPart =
            CurrentTime.QuadPart - (*Vcb).LastFatMarkVolumeDirtyCall.QuadPart;

        // If more than one second has elapsed...
        if (TimeSincePreviousCall.u.HighPart != 0)
            || (TimeSincePreviousCall.u.LowPart > (1000 * 1000 * 10))
        {
            SetTimer = 1;
        } else {
            SetTimer = 0;
        }

        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(FatData)).GeneralSpinLock),
            SavedIrql,
        );

        if SetTimer != 0 {
            let mut CleanVolumeTimer: LARGE_INTEGER = core::mem::zeroed();

            // Use shorter timer for deferred flush (hot plug) volumes
            CleanVolumeTimer.QuadPart = if ((*Vcb).VcbState & VCB_STATE_FLAG_DEFERRED_FLUSH) != 0 {
                -1500i64 * 1000 * 10
            } else {
                -8i64 * 1000 * 1000 * 10
            };

            KeCancelTimer(core::ptr::addr_of_mut!((*Vcb).CleanVolumeTimer));
            KeRemoveQueueDpc(core::ptr::addr_of_mut!((*Vcb).CleanVolumeDpc));

            // Synchronized; now check if we really need to mark dirty physically
            if ((*Vcb).VcbState & VCB_STATE_FLAG_VOLUME_DIRTY) == 0 {
                // Mark volume dirty for real
                if ((*Vcb).VcbState & VCB_STATE_FLAG_MOUNTED_DIRTY) == 0 {
                    FatMarkVolume(IrpContext, Vcb, (VolumeDirty) as i32);
                }

                (*Vcb).VcbState |= VCB_STATE_FLAG_VOLUME_DIRTY;

                // Lock the volume if removable
                if ((*Vcb).VcbState & VCB_STATE_FLAG_REMOVABLE_MEDIA) != 0 {
                    FatToggleMediaEjectDisable(IrpContext, Vcb, 1);
                }
            }

            KeAcquireSpinLock(
                core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(FatData)).GeneralSpinLock),
                core::ptr::addr_of_mut!(SavedIrql),
            );

            KeQuerySystemTime(core::ptr::addr_of_mut!((*Vcb).LastFatMarkVolumeDirtyCall));

            KeReleaseSpinLock(
                core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(FatData)).GeneralSpinLock),
                SavedIrql,
            );

            KeSetTimer(
                core::ptr::addr_of_mut!((*Vcb).CleanVolumeTimer),
                CleanVolumeTimer,
                core::ptr::addr_of_mut!((*Vcb).CleanVolumeDpc),
            );
        }
    }

    DebugTrace!(
        -1,
        Dbg,
        b"FatSetDirtyBcb -> VOID\n\0".as_ptr() as *const i8,
        0usize,
    );
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1990-2000 Microsoft Corporation
//
// Module Name:
//
//     cache.c
//
// Abstract:
//
//     This module implements the cache management routines for the Fat
//     FSD and FSP, by calling the Common Cache Manager.
//
//
// --*/
//
// #include "fatprocs.h"
//
// //
// //  The Bug check file id for this module
// //
//
// #define BugCheckFileId                   (FAT_BUG_CHECK_CACHESUP)
//
// //
// //  Local debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_CACHESUP)
//
// #if DBG
//
// BOOLEAN
// FatIsCurrentOperationSynchedForDcbTeardown (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Dcb
//     );
//
// #endif
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatCloseEaFile)
// #pragma alloc_text(PAGE, FatCompleteMdl)
// #pragma alloc_text(PAGE, FatOpenDirectoryFile)
// #pragma alloc_text(PAGE, FatOpenEaFile)
// #pragma alloc_text(PAGE, FatPinMappedData)
// #pragma alloc_text(PAGE, FatPrepareWriteDirectoryFile)
// #pragma alloc_text(PAGE, FatPrepareWriteVolumeFile)
// #pragma alloc_text(PAGE, FatReadDirectoryFile)
// #pragma alloc_text(PAGE, FatReadVolumeFile)
// #pragma alloc_text(PAGE, FatRepinBcb)
// #pragma alloc_text(PAGE, FatSyncUninitializeCacheMap)
// #pragma alloc_text(PAGE, FatUnpinRepinnedBcbs)
// #pragma alloc_text(PAGE, FatZeroData)
// #pragma alloc_text(PAGE, FatPrefetchPages)
// #if DBG
// #pragma alloc_text(PAGE, FatIsCurrentOperationSynchedForDcbTeardown)
// #endif
// #endif
//
// VOID
// FatInitializeCacheMap (
//      PFILE_OBJECT FileObject,
//      PCC_FILE_SIZES FileSizes,
//      BOOLEAN PinAccess,
//      PCACHE_MANAGER_CALLBACKS Callbacks,
//      PVOID LazyWriteContext
//     )
// /*++
//
// Routine Description:
//
//     Wrapper over CcInitializeCacheMap and CcSetAdditionalCacheAttributesEx to initialize
//     caching and enable IO accounting on a file.
//
// --*/
//
// {
//     //
//     //  Initialize caching
//     //
//
//     CcInitializeCacheMap( FileObject,
//                           FileSizes,
//                           PinAccess,
//                           Callbacks,
//                           LazyWriteContext );
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//     //
//     //  Enable Disk IO Accounting for this file
//     //
//
//     if (FatDiskAccountingEnabled) {
//
//         CcSetAdditionalCacheAttributesEx( FileObject, CC_ENABLE_DISK_IO_ACCOUNTING );
//     }
// #endif
// }
//
// VOID
// FatReadVolumeFile (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN VBO StartingVbo,
//     IN ULONG ByteCount,
//     OUT PBCB *Bcb,
//     OUT PVOID *Buffer
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called when the specified range of sectors is to be
//     read into the cache.  In fat, the volume file only contains the boot
//     sector, reserved sectors, and the "fat(s)."  Thus the volume file is
//     of fixed size and only extends up to (but not not including) the root
//     directory entry, and will never move or change size.
//
//     The fat volume file is also peculiar in that, since it starts at the
//     logical beginning of the disk, Vbo == Lbo.
//
// Arguments:
//
//     Vcb - Pointer to the VCB for the volume
//
//     StartingVbo - The virtual offset of the first desired byte
//
//     ByteCount - Number of bytes desired
//
//     Bcb - Returns a pointer to the BCB which is valid until unpinned
//
//     Buffer - Returns a pointer to the sectors, which is valid until unpinned
//
// --*/
//
// {
//     LARGE_INTEGER Vbo;
//
//     PAGED_CODE();
//
//     //
//     //  Check to see that all references are within the Bios Parameter Block
//     //  or the fat(s).  A special case is made when StartingVbo == 0 at
//     //  mounting time since we do not know how big the fat is.
//     //
//
//     NT_ASSERT( ((StartingVbo == 0) || ((StartingVbo + ByteCount) <= (ULONG)
//             (FatRootDirectoryLbo( &Vcb->Bpb ) + PAGE_SIZE))));
//
//     DebugTrace(+1, Dbg, "FatReadVolumeFile\n", 0);
//     DebugTrace( 0, Dbg, "Vcb         = %p\n", Vcb);
//     DebugTrace( 0, Dbg, "StartingVbo = %08lx\n", StartingVbo);
//     DebugTrace( 0, Dbg, "ByteCount   = %08lx\n", ByteCount);
//
//     //
//     //  Call the Cache manager to attempt the transfer.
//     //
//
//     Vbo.QuadPart = StartingVbo;
//
//     if (!CcMapData( Vcb->VirtualVolumeFile,
//                     &Vbo,
//                     ByteCount,
//                     BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT),
//                     Bcb,
//                     Buffer )) {
//
//         NT_ASSERT( !FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) );
//
//         //
//         // Could not read the data without waiting (cache miss).
//         //
//
//         FatRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//     }
//
//
//     DbgDoit( IrpContext->PinCount += 1 )
//
//     DebugTrace(-1, Dbg, "FatReadVolumeFile -> VOID, *BCB = %p\n", *Bcb);
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatPrepareWriteVolumeFile (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN VBO StartingVbo,
//     IN ULONG ByteCount,
//     OUT PBCB *Bcb,
//     OUT PVOID *Buffer,
//     IN BOOLEAN Reversible,
//     IN BOOLEAN Zero
//     )
//
// /*++
//
// Routine Description:
//
//     This routine first looks to see if the specified range of sectors,
//     is already in the cache.  If so, it increments the BCB PinCount,
//     sets the BCB dirty, and returns with the location of the sectors.
//
//     If the sectors are not in the cache and Wait is TRUE, it finds a
//     free BCB (potentially causing a flush), and clears out the entire
//     buffer.  Once this is done, it increments the BCB PinCount, sets the
//     BCB dirty, and returns with the location of the sectors.
//
//     If the sectors are not in the cache and Wait is FALSE, this routine
//     raises STATUS_CANT_WAIT.
//
// Arguments:
//
//     Vcb - Pointer to the VCB for the volume
//
//     StartingVbo - The virtual offset of the first byte to be written
//
//     ByteCount - Number of bytes to be written
//
//     Bcb - Returns a pointer to the BCB which is valid until unpinned
//
//     Buffer - Returns a pointer to the sectors, which is valid until unpinned
//
//     Reversible - Supplies TRUE if the specified range of modification should
//         be repinned so that the operation can be reversed in a controlled
//         fashion if errors are encountered.
//
//     Zero - Supplies TRUE if the specified range of bytes should be zeroed
//
// --*/
//
// {
//     LARGE_INTEGER Vbo;
//
//     PAGED_CODE();
//
//     //
//     //  Check to see that all references are within the Bios Parameter Block
//     //  or the fat(s).
//     //
//
//     NT_ASSERT( ((StartingVbo + ByteCount) <= (ULONG)
//             (FatRootDirectoryLbo( &Vcb->Bpb ))));
//
//     DebugTrace(+1, Dbg, "FatPrepareWriteVolumeFile\n", 0);
//     DebugTrace( 0, Dbg, "Vcb         = %p\n", Vcb);
//     DebugTrace( 0, Dbg, "StartingVbo = %08lx\n", (ULONG)StartingVbo);
//     DebugTrace( 0, Dbg, "ByteCount   = %08lx\n", ByteCount);
//     DebugTrace( 0, Dbg, "Zero        = %08lx\n", Zero);
//
//     //
//     //  Call the Cache manager to attempt the transfer.
//     //
//
//     Vbo.QuadPart = StartingVbo;
//
//     if (!CcPinRead( Vcb->VirtualVolumeFile,
//                     &Vbo,
//                     ByteCount,
//                     BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT),
//                     Bcb,
//                     Buffer )) {
//
//         NT_ASSERT( !FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) );
//
//         //
//         // Could not read the data without waiting (cache miss).
//         //
//
//         FatRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//     }
//
//     //
//     //  This keeps the data pinned until we complete the request
//     //  and writes the dirty bit through to the disk.
//     //
//
//     DbgDoit( IrpContext->PinCount += 1 )
//
//     _SEH2_TRY {
//
//         if (Zero) {
//
//             RtlZeroMemory( *Buffer, ByteCount );
//         }
//
//         FatSetDirtyBcb( IrpContext, *Bcb, Vcb, Reversible );
//
//     } _SEH2_FINALLY {
//
//         if (_SEH2_AbnormalTermination()) {
//
//             FatUnpinBcb(IrpContext, *Bcb);
//         }
//     } _SEH2_END;
//
//     DebugTrace(-1, Dbg, "FatPrepareWriteVolumeFile -> VOID, *Bcb = %p\n", *Bcb);
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatReadDirectoryFile (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Dcb,
//     IN VBO StartingVbo,
//     IN ULONG ByteCount,
//     IN BOOLEAN Pin,
//     OUT PBCB *Bcb,
//     OUT PVOID *Buffer,
//     OUT PNTSTATUS Status
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called when the specified range of sectors is to be
//     read into the cache.  If the desired range falls beyond the current
//     cache mapping, the fat will be searched, and if the desired range can
//     be satisfied, the cache mapping will be extended and the MCB updated
//     accordingly.
//
// Arguments:
//
//     Dcb - Pointer to the DCB for the directory
//
//     StartingVbo - The virtual offset of the first desired byte
//
//     ByteCount - Number of bytes desired
//
//     Pin - Tells us if we should pin instead of just mapping.
//
//     Bcb - Returns a pointer to the BCB which is valid until unpinned
//
//     Buffer - Returns a pointer to the sectors, which is valid until unpinned
//
//     Status - Returns the status of the operation.
//
// --*/
//
// {
//     LARGE_INTEGER Vbo;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatReadDirectoryFile\n", 0);
//     DebugTrace( 0, Dbg, "Dcb         = %p\n", Dcb);
//     DebugTrace( 0, Dbg, "StartingVbo = %08lx\n", StartingVbo);
//     DebugTrace( 0, Dbg, "ByteCount   = %08lx\n", ByteCount);
//
//     //
//     //  Check for the zero case
//     //
//
//     if (ByteCount == 0) {
//
//         DebugTrace(0, Dbg, "Nothing to read\n", 0);
//
//         *Bcb = NULL;
//         *Buffer = NULL;
//         *Status = STATUS_SUCCESS;
//
//         DebugTrace(-1, Dbg, "FatReadDirectoryFile -> VOID\n", 0);
//         return;
//     }
//
//     //
//     //  If we need to create a directory file and initialize the
//     //  cachemap, do so.
//     //
//
//     FatOpenDirectoryFile( IrpContext, Dcb );
//
//     //
//     //  Now if the transfer is beyond the allocation size return EOF.
//     //
//
//     if (StartingVbo >= Dcb->Header.AllocationSize.LowPart) {
//
//         DebugTrace(0, Dbg, "End of file read for directory\n", 0);
//
//         *Bcb = NULL;
//         *Buffer = NULL;
//         *Status = STATUS_END_OF_FILE;
//
//         DebugTrace(-1, Dbg, "FatReadDirectoryFile -> VOID\n", 0);
//         return;
//     }
//
//     //
//     // If the caller is trying to read past the EOF, truncate the
//     // read.
//     //
//
//     ByteCount = (Dcb->Header.AllocationSize.LowPart - StartingVbo < ByteCount) ?
//                  Dcb->Header.AllocationSize.LowPart - StartingVbo : ByteCount;
//
//     NT_ASSERT( ByteCount != 0 );
//
//     //
//     //  Call the Cache manager to attempt the transfer.
//     //
//
//     Vbo.QuadPart = StartingVbo;
//
//     if (Pin ?
//
//         !CcPinRead( Dcb->Specific.Dcb.DirectoryFile,
//                     &Vbo,
//                     ByteCount,
//                     BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT),
//                     Bcb,
//                     Buffer )
//         :
//
//         !CcMapData( Dcb->Specific.Dcb.DirectoryFile,
//                     &Vbo,
//                     ByteCount,
//                     BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT),
//                     Bcb,
//                     Buffer ) ) {
//
//         //
//         // Could not read the data without waiting (cache miss).
//         //
//
//         *Bcb = NULL;
//         *Buffer = NULL;
//         FatRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//     }
//
//
//     DbgDoit( IrpContext->PinCount += 1 )
//
//     *Status = STATUS_SUCCESS;
//
//     DebugTrace(-1, Dbg, "FatReadDirectoryFile -> VOID, *BCB = %p\n", *Bcb);
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatPrepareWriteDirectoryFile (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Dcb,
//     IN VBO StartingVbo,
//     IN ULONG ByteCount,
//     OUT PBCB *Bcb,
//     OUT PVOID *Buffer,
//     IN BOOLEAN Zero,
//     IN BOOLEAN Reversible,
//     OUT PNTSTATUS Status
//     )
//
// /*++
//
// Routine Description:
//
//     This routine first looks to see if the specified range of sectors
//     is already in the cache.  If so, it increments the BCB PinCount,
//     sets the BCB dirty, and returns TRUE with the location of the sectors.
//
//     The IrpContext->Flags .. Wait == TRUE/FALSE actions of this routine are identical to
//     FatPrepareWriteVolumeFile() above.
//
// Arguments:
//
//     Dcb - Pointer to the DCB for the directory
//
//     StartingVbo - The virtual offset of the first byte to be written
//
//     ByteCount - Number of bytes to be written
//
//     Bcb - Returns a pointer to the BCB which is valid until unpinned
//
//     Buffer - Returns a pointer to the sectors, which is valid until unpinned
//
//     Zero - Supplies TRUE if the specified range of bytes should be zeroed
//
//     Reversible - Supplies TRUE if the specified range of modification should
//         be repinned so that the operation can be reversed in a controlled
//         fashion if errors are encountered.
//
//     Status - Returns the status of the operation.
//
// --*/
//
// {
//     LARGE_INTEGER Vbo;
//     ULONG InitialAllocation = 0;
//     BOOLEAN UnwindWeAllocatedDiskSpace = FALSE;
//     PBCB    LocalBcb = NULL;
//     PVOID   LocalBuffer = NULL;
//     ULONG   InitialRequest = ByteCount;
//     ULONG   MappingGranularity = PAGE_SIZE;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatPrepareWriteDirectoryFile\n", 0);
//     DebugTrace( 0, Dbg, "Dcb         = %p\n", Dcb);
//     DebugTrace( 0, Dbg, "StartingVbo = %08lx\n", (ULONG)StartingVbo);
//     DebugTrace( 0, Dbg, "ByteCount   = %08lx\n", ByteCount);
//     DebugTrace( 0, Dbg, "Zero        = %08lx\n", Zero);
//
//     *Bcb = NULL;
//     *Buffer = NULL;
//
//     //
//     //  If we need to create a directory file and initialize the
//     //  cachemap, do so.
//     //
//
//     FatOpenDirectoryFile( IrpContext, Dcb );
//
//     //
//     //  If the transfer is beyond the allocation size we need to
//     //  extend the directory's allocation.  The call to
//     //  AddFileAllocation will raise a condition if
//     //  it runs out of disk space.  Note that the root directory
//     //  cannot be extended.
//     //
//
//     Vbo.QuadPart = StartingVbo;
//
//     _SEH2_TRY {
//
//         if (StartingVbo + ByteCount > Dcb->Header.AllocationSize.LowPart) {
//
//             if (NodeType(Dcb) == FAT_NTC_ROOT_DCB &&
//                 !FatIsFat32(Dcb->Vcb)) {
//
//                 FatRaiseStatus( IrpContext, STATUS_DISK_FULL );
//             }
//
//             DebugTrace(0, Dbg, "Try extending normal directory\n", 0);
//
//             InitialAllocation = Dcb->Header.AllocationSize.LowPart;
//
//             FatAddFileAllocation( IrpContext,
//                                   Dcb,
//                                   Dcb->Specific.Dcb.DirectoryFile,
//                                   StartingVbo + ByteCount );
//
//             UnwindWeAllocatedDiskSpace = TRUE;
//
//             //
//             //  Inform the cache manager of the new allocation
//             //
//
//             Dcb->Header.FileSize.LowPart =
//                 Dcb->Header.AllocationSize.LowPart;
//
//             CcSetFileSizes( Dcb->Specific.Dcb.DirectoryFile,
//                             (PCC_FILE_SIZES)&Dcb->Header.AllocationSize );
//
//             //
//             //  Set up the Bitmap buffer if it is not big enough already
//             //
//
//             FatCheckFreeDirentBitmap( IrpContext, Dcb );
//
//             //
//             //  The newly allocated clusters should be zeroed starting at
//             //  the previous allocation size
//             //
//
//             Zero = TRUE;
//             Vbo.QuadPart = InitialAllocation;
//             ByteCount = Dcb->Header.AllocationSize.LowPart - InitialAllocation;
//         }
//
//         while (ByteCount > 0) {
//
//             ULONG BytesToPin;
//
//             LocalBcb = NULL;
//
//             //
//             //  We must pin in terms of pages below the boundary of the initial request.
//             //  Once we pass the end of the request, we are free to expand the pin size to
//             //  VACB_MAPPING_GRANULARITY. This will prevent Cc from returning OBCBs
//             //  and hence will prevent bugchecks when we then attempt to repin one, yet
//             //  allow us to be more efficient by pinning in 256KB chunks instead of 4KB pages.
//             //
//
//             if (Vbo.QuadPart > StartingVbo + InitialRequest) {
//
//                 MappingGranularity = VACB_MAPPING_GRANULARITY;
//             }
//
//             //
//             //  If the first and final byte are both described by the same page, pin
//             //  the entire range. Note we pin in pages to prevent cache manager from
//             //  returning OBCBs, which would result in a bugcheck on CcRepinBcb.
//             //
//
//             if ((Vbo.QuadPart / MappingGranularity) ==
//                 ((Vbo.QuadPart + ByteCount - 1) / MappingGranularity)) {
//
//                 BytesToPin = ByteCount;
//
//             } else {
//
//                 BytesToPin = MappingGranularity -
//                              ((ULONG)Vbo.QuadPart & (MappingGranularity - 1));
//             }
//
//             if (!CcPinRead( Dcb->Specific.Dcb.DirectoryFile,
//                             &Vbo,
//                             BytesToPin,
//                             BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT),
//                             &LocalBcb,
//                             &LocalBuffer )) {
//
//                 //
//                 // Could not read the data without waiting (cache miss).
//                 //
//
//                 FatRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//             }
//
//             //
//             //  Update our caller with the beginning of their request.
//             //
//
//             if (*Buffer == NULL) {
//
//                 *Buffer = LocalBuffer;
//                 *Bcb = LocalBcb;
//             }
//
//             DbgDoit( IrpContext->PinCount += 1 )
//
//             if (Zero) {
//
//                 //
//                 //  We set this guy dirty right now so that we can raise CANT_WAIT when
//                 //  it needs to be done.  It'd be beautiful if we could noop the read IO
//                 //  since we know we don't care about it.
//                 //
//
//                 RtlZeroMemory( LocalBuffer, BytesToPin );
//                 CcSetDirtyPinnedData( LocalBcb, NULL );
//             }
//
//             ByteCount -= BytesToPin;
//             Vbo.QuadPart += BytesToPin;
//
//             if (*Bcb != LocalBcb) {
//
//                 FatRepinBcb( IrpContext, LocalBcb );
//                 FatUnpinBcb( IrpContext, LocalBcb );
//             }
//         }
//
//         //
//         //  This lets us get the data pinned until we complete the request
//         //  and writes the dirty bit through to the disk.
//         //
//
//         FatSetDirtyBcb( IrpContext, *Bcb, Dcb->Vcb, Reversible );
//
//         *Status = STATUS_SUCCESS;
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatPrepareWriteDirectoryFile );
//
//         if (_SEH2_AbnormalTermination()) {
//
//             //
//             //  Make sure we unpin the buffers.
//             //
//
//             if (*Bcb != LocalBcb) {
//
//                 FatUnpinBcb( IrpContext, LocalBcb );
//             }
//
//             FatUnpinBcb(IrpContext, *Bcb);
//
//             //
//             //  These steps are carefully arranged - FatTruncateFileAllocation can raise.
//             //  Make sure we unpin the buffer.  If FTFA raises, the effect should be benign.
//             //
//
//             if (UnwindWeAllocatedDiskSpace == TRUE) {
//
//                 //
//                 //  Inform the cache manager of the change.
//                 //
//
//                 FatTruncateFileAllocation( IrpContext, Dcb, InitialAllocation );
//
//                 Dcb->Header.FileSize.LowPart =
//                     Dcb->Header.AllocationSize.LowPart;
//
//                 CcSetFileSizes( Dcb->Specific.Dcb.DirectoryFile,
//                                 (PCC_FILE_SIZES)&Dcb->Header.AllocationSize );
//             }
//         }
//
//         DebugTrace(-1, Dbg, "FatPrepareWriteDirectoryFile -> (VOID), *Bcb = %p\n", *Bcb);
//     } _SEH2_END;
//
//     return;
// }
//
//
// #if DBG
// BOOLEAN FatDisableParentCheck = 0;
//
// BOOLEAN
// FatIsCurrentOperationSynchedForDcbTeardown (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Dcb
//     )
// {
//     PIRP Irp = IrpContext->OriginatingIrp;
//     PIO_STACK_LOCATION Stack = IoGetCurrentIrpStackLocation( Irp ) ;
//     PVCB Vcb;
//     PFCB Fcb;
//     PCCB Ccb;
//
//     PFILE_OBJECT ToCheck[3];
//     ULONG Index = 0;
//
//     PAGED_CODE();
//
//     //
//     //  While mounting, we're OK without having to own anything.
//     //
//
//     if (Stack->MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL &&
//         Stack->MinorFunction == IRP_MN_MOUNT_VOLUME) {
//
//         return TRUE;
//     }
//
//     //
//     //  With the Vcb held, the close path is blocked out.
//     //
//
//     if (ExIsResourceAcquiredSharedLite( &Dcb->Vcb->Resource ) ||
//         ExIsResourceAcquiredExclusiveLite( &Dcb->Vcb->Resource )) {
//
//         return TRUE;
//     }
//
//     //
//     //  Accept this assertion at face value.  It comes from GetDirentForFcbOrDcb,
//     //  and is reliable.
//     //
//
//     if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_PARENT_BY_CHILD )) {
//
//         return TRUE;
//     }
//
//     //
//     //  Determine which fileobjects are around on this operation.
//     //
//
//     if (Stack->MajorFunction == IRP_MJ_SET_INFORMATION &&
//         Stack->Parameters.SetFile.FileObject) {
//
//         ToCheck[Index++] = Stack->Parameters.SetFile.FileObject;
//     }
//
//     if (Stack->FileObject) {
//
//         ToCheck[Index++] = Stack->FileObject;
//     }
//
//     ToCheck[Index] = NULL;
//
//     //
//     //  If the fileobjects we have are for this dcb or a child of it, we are
//     //  also guaranteed that this dcb isn't going anywhere (even without
//     //  the Vcb).
//     //
//
//     for (Index = 0; ToCheck[Index] != NULL; Index++) {
//
//         (VOID) FatDecodeFileObject( ToCheck[Index], &Vcb, &Fcb, &Ccb );
//
//         while ( Fcb ) {
//
//             if (Fcb == Dcb) {
//
//                 return TRUE;
//             }
//
//             Fcb = Fcb->ParentDcb;
//         }
//     }
//
//     return FatDisableParentCheck;
// }
// #endif // DBG
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatOpenDirectoryFile (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Dcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine opens a new directory file if one is not already open.
//
// Arguments:
//
//     Dcb - Pointer to the DCB for the directory
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatOpenDirectoryFile\n", 0);
//     DebugTrace( 0, Dbg, "Dcb = %p\n", Dcb);
//
//     //
//     //  If we don't have some hold on this Dcb (there are several ways), there is nothing
//     //  to prevent child files from closing and tearing this branch of the tree down in the
//     //  midst of our slapping this reference onto it.
//     //
//     //  I really wish we had a proper Fcb synchronization model (like CDFS/UDFS/NTFS).
//     //
//
//     NT_ASSERT( FatIsCurrentOperationSynchedForDcbTeardown( IrpContext, Dcb ));
//
//     //
//     //  If we haven't yet set the correct AllocationSize, do so.
//     //
//
//     if (Dcb->Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT) {
//
//         FatLookupFileAllocationSize( IrpContext, Dcb );
//
//         Dcb->Header.FileSize.LowPart =
//         Dcb->Header.AllocationSize.LowPart;
//     }
//
//     //
//     //  Setup the Bitmap buffer if it is not big enough already
//     //
//
//     FatCheckFreeDirentBitmap( IrpContext, Dcb );
//
//     //
//     //  Check if we need to create a directory file.
//     //
//     //  We first do a spot check and then synchronize and check again.
//     //
//
//     if (Dcb->Specific.Dcb.DirectoryFile == NULL) {
//
//         PFILE_OBJECT DirectoryFileObject = NULL;
//
//         FatAcquireDirectoryFileMutex( Dcb->Vcb );
//
//         _SEH2_TRY {
//
//             if (Dcb->Specific.Dcb.DirectoryFile == NULL) {
//
//                 PDEVICE_OBJECT RealDevice;
//
//                 //
//                 //  Create the special file object for the directory file, and set
//                 //  up its pointers back to the Dcb and the section object pointer.
//                 //  Note that setting the DirectoryFile pointer in the Dcb has
//                 //  to be the last thing done.
//                 //
//                 //  Preallocate a close context since we have no Ccb for this object.
//                 //
//
//                 RealDevice = Dcb->Vcb->CurrentDevice;
//
//                 DirectoryFileObject = IoCreateStreamFileObject( NULL, RealDevice );
//                 FatPreallocateCloseContext( Dcb->Vcb);
//
//                 FatSetFileObject( DirectoryFileObject,
//                                   DirectoryFile,
//                                   Dcb,
//                                   NULL );
//
//                 //
//                 //  Remember this internal open.
//                 //
//
//                 InterlockedIncrement( (LONG*)&(Dcb->Vcb->InternalOpenCount) );
//
//                 //
//                 //  If this is the root directory, it is also a residual open.
//                 //
//
//                 if (NodeType( Dcb ) == FAT_NTC_ROOT_DCB) {
//
//                     InterlockedIncrement( (LONG*)&(Dcb->Vcb->ResidualOpenCount) );
//                 }
//
//                 DirectoryFileObject->SectionObjectPointer = &Dcb->NonPaged->SectionObjectPointers;
//
//                 DirectoryFileObject->ReadAccess = TRUE;
//                 DirectoryFileObject->WriteAccess = TRUE;
//                 DirectoryFileObject->DeleteAccess = TRUE;
//
//                 InterlockedIncrement( (LONG*)&Dcb->Specific.Dcb.DirectoryFileOpenCount );
//
//                 Dcb->Specific.Dcb.DirectoryFile = DirectoryFileObject;
//
//                 //
//                 //  Indicate we're happy with the fileobject now.
//                 //
//
//                 DirectoryFileObject = NULL;
//             }
//
//         } _SEH2_FINALLY {
//
//             FatReleaseDirectoryFileMutex( Dcb->Vcb );
//
//             //
//             //  Rip the object up if we couldn't get the close context.
//             //
//
//             if (DirectoryFileObject) {
//
//                 ObDereferenceObject( DirectoryFileObject );
//             }
//         } _SEH2_END;
//     }
//
//     //
//     //  Finally check if we need to initialize the Cache Map for the
//     //  directory file.  The size of the section we are going to map
//     //  the current allocation size for the directory.  Note that the
//     //  cache manager will provide syncronization for us.
//     //
//
//     if ( Dcb->Specific.Dcb.DirectoryFile->PrivateCacheMap == NULL ) {
//
//         Dcb->Header.ValidDataLength = FatMaxLarge;
//         Dcb->ValidDataToDisk = MAXULONG;
//
//         FatInitializeCacheMap( Dcb->Specific.Dcb.DirectoryFile,
//                                (PCC_FILE_SIZES)&Dcb->Header.AllocationSize,
//                                TRUE,
//                                &FatData.CacheManagerNoOpCallbacks,
//                                Dcb );
//     }
//
//     DebugTrace(-1, Dbg, "FatOpenDirectoryFile -> VOID\n", 0);
//
//     return;
// }
//
//
//
//
// PFILE_OBJECT
// FatOpenEaFile (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB EaFcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine opens the Ea file.
//
// Arguments:
//
//     EaFcb - Pointer to the Fcb for the Ea file.
//
// Return Value:
//
//     Pointer to the new file object.
//
// --*/
//
// {
//     PFILE_OBJECT EaFileObject = NULL;
//     PDEVICE_OBJECT RealDevice;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatOpenEaFile\n", 0);
//     DebugTrace( 0, Dbg, "EaFcb = %p\n", EaFcb);
//
//     //
//     //  Create the special file object for the ea file, and set
//     //  up its pointers back to the Fcb and the section object pointer
//     //
//
//     RealDevice = EaFcb->Vcb->CurrentDevice;
//
//     EaFileObject = IoCreateStreamFileObject( NULL, RealDevice );
//
//     _SEH2_TRY {
//
//         FatPreallocateCloseContext( IrpContext->Vcb);
//
//         FatSetFileObject( EaFileObject,
//                           EaFile,
//                           EaFcb,
//                           NULL );
//
//         //
//         //  Remember this internal, residual open.
//         //
//
//         InterlockedIncrement( (LONG*)&(EaFcb->Vcb->InternalOpenCount) );
//         InterlockedIncrement( (LONG*)&(EaFcb->Vcb->ResidualOpenCount) );
//
//         EaFileObject->SectionObjectPointer = &EaFcb->NonPaged->SectionObjectPointers;
//
//         EaFileObject->ReadAccess = TRUE;
//         EaFileObject->WriteAccess = TRUE;
//
//         //
//         //  Finally check if we need to initialize the Cache Map for the
//         //  ea file.  The size of the section we are going to map
//         //  the current allocation size for the Fcb.
//         //
//
//         EaFcb->Header.ValidDataLength = FatMaxLarge;
//
//         FatInitializeCacheMap( EaFileObject,
//                                (PCC_FILE_SIZES)&EaFcb->Header.AllocationSize,
//                                TRUE,
//                                &FatData.CacheManagerCallbacks,
//                                EaFcb );
//
//         CcSetAdditionalCacheAttributes( EaFileObject, TRUE, TRUE );
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Drop the fileobject if we're raising.  Two cases: couldn't get
//         //  the close context, and it is still an UnopenedFileObject, or
//         //  we lost trying to build the cache map - in which case we're
//         //  OK for the close context if we have to.
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             ObDereferenceObject( EaFileObject );
//         }
//     } _SEH2_END;
//
//     DebugTrace(-1, Dbg, "FatOpenEaFile -> %p\n", EaFileObject);
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return EaFileObject;
// }
//
//
// VOID
// FatCloseEaFile (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN BOOLEAN FlushFirst
//     )
//
// /*++
//
// Routine Description:
//
//     This routine shuts down the ea file.  Usually this is required when the volume
//     begins to leave the system: after verify, dismount, deletion, pnp.
//
// Arguments:
//
//     Vcb - the volume to close the ea file on
//
//     FlushFirst - whether the file should be flushed
//
// Return Value:
//
//     None. As a side effect, the EA fileobject in the Vcb is cleared.
//
//     Caller must have the Vcb exclusive.
//
// --*/
//
// {
//     PFILE_OBJECT EaFileObject = Vcb->VirtualEaFile;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatCloseEaFile\n", 0);
//     DebugTrace( 0, Dbg, "Vcb = %p\n", Vcb);
//
//     NT_ASSERT( FatVcbAcquiredExclusive(IrpContext, Vcb) );
//
//     if (EaFileObject != NULL) {
//
//         EaFileObject = Vcb->VirtualEaFile;
//
//         if (FlushFirst) {
//
//             CcFlushCache( Vcb->VirtualEaFile->SectionObjectPointer, NULL, 0, NULL );
//         }
//
//         Vcb->VirtualEaFile = NULL;
//
//         //
//         //  Empty the Mcb for the Ea file.
//         //
//
//         FatRemoveMcbEntry( Vcb, &Vcb->EaFcb->Mcb, 0, 0xFFFFFFFF );
//
//         //
//         //  Uninitialize the cache for this file object and dereference it.
//         //
//
//         FatSyncUninitializeCacheMap( IrpContext, EaFileObject );
//
//         ObDereferenceObject( EaFileObject );
//     }
//
//     DebugTrace(-1, Dbg, "FatCloseEaFile -> %p\n", EaFileObject);
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatSetDirtyBcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PBCB Bcb,
//     IN PVCB Vcb OPTIONAL,
//     IN BOOLEAN Reversible
//     )
//
// /*++
//
// Routine Description:
//
//     This routine saves a reference to the bcb in the irp context and
//     sets the bcb dirty.  This will have the affect of keeping the page in
//     memory until we complete the request
//
//     In addition, a DPC is set to fire in 5 seconds (or if one is pending,
//     pushed back 5 seconds) to mark the volume clean.
//
// Arguments:
//
//     Bcb - Supplies the Bcb being set dirty
//
//     Vcb - Supplies the volume being marked dirty
//
//     Reversible - Supplies TRUE if the specified range of bcb should be repinned
//         so that the changes can be reversed in a controlled fashion if errors
//         are encountered.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     DebugTrace(+1, Dbg, "FatSetDirtyBcb\n", 0 );
//     DebugTrace( 0, Dbg, "IrpContext = %p\n", IrpContext );
//     DebugTrace( 0, Dbg, "Bcb        = %p\n", Bcb );
//     DebugTrace( 0, Dbg, "Vcb        = %p\n", Vcb );
//
//     //
//     //  Repin the bcb as required
//     //
//
//     if (Reversible) {
//
//         FatRepinBcb( IrpContext, Bcb );
//     }
//
//     //
//     //  Set the bcb dirty
//     //
//
//     CcSetDirtyPinnedData( Bcb, NULL );
//
//     //
//     //  If volume dirtying isn't disabled for this operation (for
//     //  instance, when we're changing the dirty state), set the
//     //  volume dirty if we were given a Vcb that we want to perform
//     //  clean volume processing on, and return.
//     //
//     //  As a historical note, we used to key off of the old floppy
//     //  (now deferred flush) bit to disable dirtying behavior.  Since
//     //  hotpluggable media can still be yanked while operations are
//     //  in flight, recognize that its really the case that FAT12
//     //  doesn't have the dirty bit.
//     //
//
//     if ( !FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_DIRTY) &&
//          ARGUMENT_PRESENT(Vcb) &&
//          !FatIsFat12(Vcb)) {
//
//         KIRQL SavedIrql;
//
//         BOOLEAN SetTimer;
//
//         LARGE_INTEGER TimeSincePreviousCall;
//         LARGE_INTEGER CurrentTime;
//
//         //
//         //  "Borrow" the irp context spinlock.
//         //
//
//         KeQuerySystemTime( &CurrentTime );
//
//         KeAcquireSpinLock( &FatData.GeneralSpinLock, &SavedIrql );
//
//         TimeSincePreviousCall.QuadPart =
//                 CurrentTime.QuadPart - Vcb->LastFatMarkVolumeDirtyCall.QuadPart;
//
//         //
//         //  If more than one second has elapsed since the prior call
//         //  to here, bump the timer up again and see if we need to
//         //  physically mark the volume dirty.
//         //
//
//         if ( (TimeSincePreviousCall.HighPart != 0) ||
//              (TimeSincePreviousCall.LowPart > (1000 * 1000 * 10)) ) {
//
//             SetTimer = TRUE;
//
//         } else {
//
//             SetTimer = FALSE;
//         }
//
//         KeReleaseSpinLock( &FatData.GeneralSpinLock, SavedIrql );
//
//         if ( SetTimer ) {
//
//             LARGE_INTEGER CleanVolumeTimer;
//
//             //
//             //  We use a shorter volume clean timer for hot plug volumes.
//             //
//
//             CleanVolumeTimer.QuadPart = FlagOn( Vcb->VcbState, VCB_STATE_FLAG_DEFERRED_FLUSH)
//                                            ? (LONG)-1500*1000*10
//                                            : (LONG)-8*1000*1000*10;
//
//             (VOID)KeCancelTimer( &Vcb->CleanVolumeTimer );
//             (VOID)KeRemoveQueueDpc( &Vcb->CleanVolumeDpc );
//
//             //
//             //  We have now synchronized with anybody clearing the dirty
//             //  flag, so we can now see if we really have to actually write
//             //  out the physical bit.
//             //
//
//             if ( !FlagOn(Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DIRTY) ) {
//
//                 //
//                 //  We want to really mark the volume dirty now.
//                 //
//
//                 if (!FlagOn(Vcb->VcbState, VCB_STATE_FLAG_MOUNTED_DIRTY)) {
//
//                     FatMarkVolume( IrpContext, Vcb, VolumeDirty );
//                 }
//
//                 SetFlag( Vcb->VcbState, VCB_STATE_FLAG_VOLUME_DIRTY );
//
//                 //
//                 //  Lock the volume if it is removable.
//                 //
//
//                 if (FlagOn( Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA)) {
//
//                     FatToggleMediaEjectDisable( IrpContext, Vcb, TRUE );
//                 }
//             }
//
//             KeAcquireSpinLock( &FatData.GeneralSpinLock, &SavedIrql );
//
//             KeQuerySystemTime( &Vcb->LastFatMarkVolumeDirtyCall );
//
//             KeReleaseSpinLock( &FatData.GeneralSpinLock, SavedIrql );
//
//             KeSetTimer( &Vcb->CleanVolumeTimer,
//                         CleanVolumeTimer,
//                         &Vcb->CleanVolumeDpc );
//         }
//     }
//
//     DebugTrace(-1, Dbg, "FatSetDirtyBcb -> VOID\n", 0 );
// }
//
//
// VOID
// FatRepinBcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PBCB Bcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine saves a reference to the bcb in the irp context. This will
//     have the affect of keeping the page in memory until we complete the
//     request
//
// Arguments:
//
//     Bcb - Supplies the Bcb being referenced
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PREPINNED_BCBS Repinned;
//     ULONG i;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatRepinBcb\n", 0 );
//     DebugTrace( 0, Dbg, "IrpContext = %p\n", IrpContext );
//     DebugTrace( 0, Dbg, "Bcb        = %p\n", Bcb );
//
//     //
//     //  The algorithm is to search the list of repinned records until
//     //  we either find a match for the bcb or we find a null slot.
//     //
//
//     Repinned = &IrpContext->Repinned;
//
//     while (TRUE) {
//
//         //
//         //  For every entry in the repinned record check if the bcb's
//         //  match or if the entry is null.  If the bcb's match then
//         //  we've done because we've already repinned this bcb, if
//         //  the entry is null then we know, because it's densely packed,
//         //  that the bcb is not in the list so add it to the repinned
//         //  record and repin it.
//         //
//
//         for (i = 0; i < REPINNED_BCBS_ARRAY_SIZE; i += 1) {
//
//             if (Repinned->Bcb[i] == Bcb) {
//
//                 DebugTrace(-1, Dbg, "FatRepinBcb -> VOID\n", 0 );
//                 return;
//             }
//
//             if (Repinned->Bcb[i] == NULL) {
//
//                 Repinned->Bcb[i] = Bcb;
//                 CcRepinBcb( Bcb );
//
//                 DebugTrace(-1, Dbg, "FatRepinBcb -> VOID\n", 0 );
//                 return;
//             }
//         }
//
//         //
//         //  We finished checking one repinned record so now locate the next
//         //  repinned record,  If there isn't one then allocate and zero out
//         //  a new one.
//         //
//
//         if (Repinned->Next == NULL) {
//
//             Repinned->Next = FsRtlAllocatePoolWithTag( PagedPool,
//                                                        sizeof(REPINNED_BCBS),
//                                                        TAG_REPINNED_BCB );
//
//             RtlZeroMemory( Repinned->Next, sizeof(REPINNED_BCBS) );
//         }
//
//         Repinned = Repinned->Next;
//     }
// }
//
//
// VOID
// FatUnpinRepinnedBcbs (
//     IN PIRP_CONTEXT IrpContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine frees all of the repinned bcbs, stored in an IRP context.
//
// Arguments:
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     IO_STATUS_BLOCK RaiseIosb;
//     PREPINNED_BCBS Repinned;
//     BOOLEAN WriteThroughToDisk;
//     PFILE_OBJECT FileObject = NULL;
//     BOOLEAN ForceVerify = FALSE;
//     ULONG i;
//     PFCB FcbOrDcb = NULL;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatUnpinRepinnedBcbs\n", 0 );
//     DebugTrace( 0, Dbg, "IrpContext = %p\n", IrpContext );
//
//     //
//     //  The algorithm for this procedure is to scan the entire list of
//     //  repinned records unpinning any repinned bcbs.  We start off
//     //  with the first record in the irp context, and while there is a
//     //  record to scan we do the following loop.
//     //
//
//     Repinned = &IrpContext->Repinned;
//     RaiseIosb.Status = STATUS_SUCCESS;
//
//     //
//     //  WinSE bug #307418 "Occasional data corruption when
//     //  standby/resume while copying files to removable FAT
//     //  formatted media".
//     //  Extract main FCB pointer from the irp context - we
//     //  will need it later to detect new file creation operation.
//     //
//
//     if (IrpContext->MajorFunction == IRP_MJ_CREATE &&
//         IrpContext->OriginatingIrp != NULL) {
//         PIO_STACK_LOCATION IrpSp;
//
//         IrpSp = IoGetCurrentIrpStackLocation( IrpContext->OriginatingIrp );
//
//         if (IrpSp != NULL &&
//             IrpSp->FileObject != NULL &&
//             IrpSp->FileObject->FsContext != NULL) {
//
//             FcbOrDcb = IrpSp->FileObject->FsContext;
//         }
//     }
//
//     //
//     //  If the request is write through or the media is deferred flush,
//     //  unpin the bcb's write through.
//     //
//
//     WriteThroughToDisk = (BOOLEAN) (!FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_WRITE_THROUGH) &&
//                                     IrpContext->Vcb != NULL &&
//                                     (FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH) ||
//                                      FlagOn(IrpContext->Vcb->VcbState, VCB_STATE_FLAG_DEFERRED_FLUSH)));
//
//     while (Repinned != NULL) {
//
//         //
//         //  For every non-null entry in the repinned record unpin the
//         //  repinned entry.
//         //
//         //  If the this is removable media (therefore all requests write-
//         //  through) and the write fails, purge the cache so that we throw
//         //  away the modifications as we will be returning an error to the
//         //  user.
//         //
//
//         for (i = 0; i < REPINNED_BCBS_ARRAY_SIZE; i += 1) {
//
//             if (Repinned->Bcb[i] != NULL) {
//
//                 IO_STATUS_BLOCK Iosb;
//
//                 if (WriteThroughToDisk &&
//                     FlagOn(IrpContext->Vcb->VcbState, VCB_STATE_FLAG_DEFERRED_FLUSH)) {
//
//                     FileObject = CcGetFileObjectFromBcb( Repinned->Bcb[i] );
//                 }
//
//                 CcUnpinRepinnedBcb( Repinned->Bcb[i],
//                                     WriteThroughToDisk,
//                                     &Iosb );
//
//                 if (!NT_SUCCESS(Iosb.Status)) {
//
//                     if (RaiseIosb.Status == STATUS_SUCCESS) {
//
//                         RaiseIosb = Iosb;
//                     }
//
//                     //
//                     //  If this was a writethrough device, purge the cache,
//                     //  except for Irp major codes that either don't handle
//                     //  the error paths correctly or are simple victims like
//                     //  cleanup.c.
//                     //
//
//                     if (FileObject &&
//                         (IrpContext->MajorFunction != IRP_MJ_CLEANUP) &&
//                         (IrpContext->MajorFunction != IRP_MJ_FLUSH_BUFFERS) &&
//                         (IrpContext->MajorFunction != IRP_MJ_SET_INFORMATION)
//
//                                         &&
//
//                             //
//                             //  WinSE bug #307418 "Occasional data corruption when
//                             //  standby/resume while copying files to removable FAT
//                             //  formatted media".
//                             //  Buffer unpinning for new file creation operation can
//                             //  be interrupted by system syspend. As a result some BCBs
//                             //  will be successfully written to the disk while others will
//                             //  be kicked back with STATUS_VERIFY_REQUIRED. Since there is
//                             //  is still a chance for the failed BCBs to reach the disk
//                             //  after the volume verification we'll not purge them.
//                             //  Instead FatCommonCreate() will unroll the file creation
//                             //  changes for these pages.
//                             //
//
//                         !(IrpContext->MajorFunction == IRP_MJ_CREATE &&
//                           Iosb.Status == STATUS_VERIFY_REQUIRED &&
//                           FcbOrDcb != NULL &&
//                           NodeType( FcbOrDcb ) == FAT_NTC_FCB)) {
//
//                         //
//                         //  The call to CcPurgeCacheSection() below will
//                         //  purge the entire file from memory.  It will also
//                         //  block until all the file's BCB's are pinned.
//                         //
//                         //  We end up in a deadlock situation of there
//                         //  are any other pinned BCB's in this IRP context
//                         //  so the first thing we do is search the list
//                         //  for BCB's pinned in the same file and unpin
//                         //  them.
//                         //
//                         //  We are probably not going to lose data because
//                         //  it's safe to assume that all flushes will
//                         //  fail after the first one fails.
//                         //
//
//                         ULONG j;
//                         ULONG k = i + 1;
//                         PREPINNED_BCBS RepinnedToPurge = Repinned;
//
//                         while( RepinnedToPurge != NULL ) {
//
//                             for (j = k; j < REPINNED_BCBS_ARRAY_SIZE; j++) {
//
//                                 if (RepinnedToPurge->Bcb[j] != NULL) {
//
//                                     if (CcGetFileObjectFromBcb( RepinnedToPurge->Bcb[j] ) == FileObject) {
//
//                                         CcUnpinRepinnedBcb( RepinnedToPurge->Bcb[j],
//                                                             FALSE,
//                                                             &Iosb );
//
//                                         RepinnedToPurge->Bcb[j] = NULL;
//                                     }
//                                 }
//                             }
//
//                             RepinnedToPurge = RepinnedToPurge->Next;
//                             k = 0;
//                         }
//
//                         CcPurgeCacheSection( FileObject->SectionObjectPointer,
//                                              NULL,
//                                              0,
//                                              FALSE );
//
//                         //
//                         //  Force a verify operation here since who knows
//                         //  what state things are in.
//                         //
//
//                         ForceVerify = TRUE;
//                     }
//                 }
//
//                 Repinned->Bcb[i] = NULL;
//
//             }
//         }
//
//         //
//         //  Now find the next repinned record in the list, and possibly
//         //  delete the one we've just processed.
//         //
//
//         if (Repinned != &IrpContext->Repinned) {
//
//             PREPINNED_BCBS Saved;
//
//             Saved = Repinned->Next;
//             ExFreePool( Repinned );
//             Repinned = Saved;
//
//         } else {
//
//             Repinned = Repinned->Next;
//             IrpContext->Repinned.Next = NULL;
//         }
//     }
//
//     //
//     //  Now if we weren't completely successful in the our unpin
//     //  then raise the iosb we got
//     //
//
//     if (!NT_SUCCESS(RaiseIosb.Status)) {
//
//         if (ForceVerify && FileObject) {
//
//             SetFlag(FileObject->DeviceObject->Flags, DO_VERIFY_VOLUME);
//
//             IoSetHardErrorOrVerifyDevice( IrpContext->OriginatingIrp,
//                                           FileObject->DeviceObject );
//         }
//
//         if (!FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_DISABLE_RAISE )) {
//             if (IrpContext->OriginatingIrp) {
//                 IrpContext->OriginatingIrp->IoStatus = RaiseIosb;
//             }
//             FatNormalizeAndRaiseStatus( IrpContext, RaiseIosb.Status );
//         }
//     }
//
//     DebugTrace(-1, Dbg, "FatUnpinRepinnedBcbs -> VOID\n", 0 );
//
//     return;
// }
//
//
// FINISHED
// FatZeroData (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PFILE_OBJECT FileObject,
//     IN ULONG StartingZero,
//     IN ULONG ByteCount
//     )
//
// /*++
//
//     **** Temporary function - Remove when CcZeroData is capable of handling
//     non sector aligned requests.
//
// --*/
// {
// #ifndef __REACTOS__
//     LARGE_INTEGER ZeroStart = {0,0};
//     LARGE_INTEGER BeyondZeroEnd = {0,0};
// #else
//     LARGE_INTEGER ZeroStart = {{0,0}};
//     LARGE_INTEGER BeyondZeroEnd = {{0,0}};
// #endif
//
//     ULONG SectorSize;
//
//     BOOLEAN Finished;
//
//     PAGED_CODE();
//
//     SectorSize = (ULONG)Vcb->Bpb.BytesPerSector;
//
//     ZeroStart.LowPart = (StartingZero + (SectorSize - 1)) & ~(SectorSize - 1);
//
//     //
//     //  Detect overflow if we were asked to zero in the last sector of the file,
//     //  which must be "zeroed" already (or we're in trouble).
//     //
//
//     if (StartingZero != 0 && ZeroStart.LowPart == 0) {
//
//         return TRUE;
//     }
//
//     //
//     //  Note that BeyondZeroEnd can take the value 4gb.
//     //
//
//     BeyondZeroEnd.QuadPart = ((ULONGLONG) StartingZero + ByteCount + (SectorSize - 1))
//                              & (~((LONGLONG) SectorSize - 1));
//
//     //
//     //  If we were called to just zero part of a sector we are in trouble.
//     //
//
//     if ( ZeroStart.QuadPart == BeyondZeroEnd.QuadPart ) {
//
//         return TRUE;
//     }
//
//     Finished = CcZeroData( FileObject,
//                            &ZeroStart,
//                            &BeyondZeroEnd,
//                            BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) );
//
//     return Finished;
// }
//
//
// NTSTATUS
// FatCompleteMdl (
//     IN PIRP_CONTEXT IrpContext,
//     IN PIRP Irp
//     )
//
// /*++
//
// Routine Description:
//
//     This routine performs the function of completing Mdl read and write
//     requests.  It should be called only from FatFsdRead and FatFsdWrite.
//
// Arguments:
//
//     Irp - Supplies the originating Irp.
//
// Return Value:
//
//     NTSTATUS - Will always be STATUS_PENDING or STATUS_SUCCESS.
//
// --*/
//
// {
//     PFILE_OBJECT FileObject;
//     PIO_STACK_LOCATION IrpSp;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatCompleteMdl\n", 0 );
//     DebugTrace( 0, Dbg, "IrpContext = %p\n", IrpContext );
//     DebugTrace( 0, Dbg, "Irp        = %p\n", Irp );
//
//     //
//     // Do completion processing.
//     //
//
//     FileObject = IoGetCurrentIrpStackLocation( Irp )->FileObject;
//
//     switch( IrpContext->MajorFunction ) {
//
//     case IRP_MJ_READ:
//
//         CcMdlReadComplete( FileObject, Irp->MdlAddress );
//         break;
//
//     case IRP_MJ_WRITE:
//
//         IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//         NT_ASSERT( FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT ));
//
//         CcMdlWriteComplete( FileObject, &IrpSp->Parameters.Write.ByteOffset, Irp->MdlAddress );
//
//         Irp->IoStatus.Status = STATUS_SUCCESS;
//
//         break;
//
//     default:
//
//         DebugTrace( DEBUG_TRACE_ERROR, 0, "Illegal Mdl Complete.\n", 0);
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28159, "we're very broken if we get here" )
// #endif
//         FatBugCheck( IrpContext->MajorFunction, 0, 0 );
//     }
//
//     //
//     // Mdl is now deallocated.
//     //
//
//     Irp->MdlAddress = NULL;
//
//     //
//     // Complete the request and exit right away.
//     //
//
//     FatCompleteRequest( IrpContext, Irp, STATUS_SUCCESS );
//
//     DebugTrace(-1, Dbg, "FatCompleteMdl -> STATUS_SUCCESS\n", 0 );
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// FatSyncUninitializeCacheMap (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFILE_OBJECT FileObject
//     )
//
// /*++
//
// Routine Description:
//
//     The routine performs a CcUnitializeCacheMap to LargeZero synchronously.  That
//     is it waits on the Cc event.  This call is useful when we want to be certain
//     when a close will actually some in.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     CACHE_UNINITIALIZE_EVENT UninitializeCompleteEvent;
//     NTSTATUS WaitStatus;
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     PAGED_CODE();
//
//     KeInitializeEvent( &UninitializeCompleteEvent.Event,
//                        SynchronizationEvent,
//                        FALSE);
//
//     CcUninitializeCacheMap( FileObject,
//                             &FatLargeZero,
//                             &UninitializeCompleteEvent );
//
//     //
//     //  Now wait for the cache manager to finish purging the file.
//     //  This will garentee that Mm gets the purge before we
//     //  delete the Vcb.
//     //
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28931, "we use WaitStatus in the debug assert, in fre builds prefast complains it's unused" )
// #endif
//     WaitStatus = KeWaitForSingleObject( &UninitializeCompleteEvent.Event,
//                                         Executive,
//                                         KernelMode,
//                                         FALSE,
//                                         NULL);
//
//     NT_ASSERT(WaitStatus == STATUS_SUCCESS);
// }
//
// VOID
// FatPinMappedData (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Dcb,
//     IN VBO StartingVbo,
//     IN ULONG ByteCount,
//     OUT PBCB *Bcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine pins data that was previously mapped before setting it dirty.
//
// Arguments:
//
//     Dcb - Pointer to the DCB for the directory
//
//     StartingVbo - The virtual offset of the first desired byte
//
//     ByteCount - Number of bytes desired
//
//     Bcb - Returns a pointer to the BCB which is valid until unpinned
//
// --*/
//
// {
//     LARGE_INTEGER Vbo;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatPinMappedData\n", 0);
//     DebugTrace( 0, Dbg, "Dcb         = %p\n", Dcb);
//     DebugTrace( 0, Dbg, "StartingVbo = %08lx\n", StartingVbo);
//     DebugTrace( 0, Dbg, "ByteCount   = %08lx\n", ByteCount);
//
//     //
//     //  Call the Cache manager to perform the operation.
//     //
//
//     Vbo.QuadPart = StartingVbo;
//
//     if (!CcPinMappedData( Dcb->Specific.Dcb.DirectoryFile,
//                           &Vbo,
//                           ByteCount,
//                           BooleanFlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT),
//                           Bcb )) {
//
//         //
//         // Could not pin the data without waiting (cache miss).
//         //
//
//         FatRaiseStatus( IrpContext, STATUS_CANT_WAIT );
//     }
//
//     DebugTrace(-1, Dbg, "FatReadDirectoryFile -> VOID, *BCB = %p\n", *Bcb);
//
//     return;
// }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
// NTSTATUS
// FatPrefetchPages (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFILE_OBJECT FileObject,
//     IN ULONG StartingPage,
//     IN ULONG PageCount
//     )
// {
//     IO_PRIORITY_INFO PriorityInformation = {0};
//     MM_PREFETCH_FLAGS PrefetchFlags;
//     ULONG PageNo;
//     NTSTATUS Status;
//
//     PREAD_LIST ReadList = NULL;
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     PAGED_CODE();
//
//     //
//     //  Succeed zero page prefetch requests.
//     //
//
//     if (PageCount == 0) {
//
//         return STATUS_SUCCESS;
//     }
//
//     //
//     //  Mm's prefetch API's "only" support fetching a ULONG worth of pages.
//     //  Make sure we don't overflow.
//     //
//
//     ASSERT( PageCount < (PFN_NUMBER)MAXULONG );
//
//     IoInitializePriorityInfo( &PriorityInformation );
//
//     Status = IoRetrievePriorityInfo( IrpContext->OriginatingIrp,
//                                      FileObject,
//                                      IrpContext->OriginatingIrp->Tail.Overlay.Thread,
//                                      &PriorityInformation );
//
//     if (!NT_SUCCESS( Status)) {
//
//         goto Cleanup;
//     }
//
//     ReadList = ExAllocatePoolWithTag( PagedPool,
//                                       FIELD_OFFSET( READ_LIST, List ) + PageCount * sizeof( FILE_SEGMENT_ELEMENT ),
//                                       ' taF' );
//
//     if (ReadList == NULL) {
//
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto Cleanup;
//     }
//
//     //
//     //  Call Mm to prefetch data.
//     //
//
//     ReadList->FileObject = FileObject;
//     ReadList->IsImage = FALSE;
//     ReadList->NumberOfEntries = PageCount;
//
//     PrefetchFlags.AllFlags = 0;
//     PrefetchFlags.Flags.Priority = PriorityInformation.PagePriority;
//     PrefetchFlags.Flags.RepurposePriority = SYSTEM_PAGE_PRIORITY_LEVELS - 1;
//     PrefetchFlags.Flags.PriorityProtection = 1;
//     ReadList->List[0].Alignment = StartingPage * PAGE_SIZE;
//     ReadList->List[0].Alignment |= PrefetchFlags.AllFlags;
//
//     for (PageNo = 1; PageNo < PageCount; PageNo++) {
//
//         ReadList->List[PageNo].Alignment = ReadList->List[PageNo-1].Alignment + PAGE_SIZE;
//     }
//
//     Status = MmPrefetchPages( 1, &ReadList );
//
// Cleanup:
//
//     if (ReadList != NULL) {
//
//         ExFreePoolWithTag( ReadList, ' taF' );
//     }
//
//     return Status;
// }
// #endif
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// fatprocs.h flag macros, inline at the declared widths (were C stubs taking
// ULONG_PTR, which read garbage upper bits from a narrower argument).
#[inline(always)]
unsafe fn BooleanFlagOn(flags: u32, flag: u32) -> u8 {
    ((flags as u64 & flag as u64) != 0) as u8
}

#[inline(always)]
unsafe fn FlagOn(flags: u32, mask: u32) -> i32 {
    (flags as u64 & mask as u64) as i32
}
