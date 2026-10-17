#[allow(unused_imports)] use crate::{NT_ASSERT};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CLONG, ExDeleteResourceLite, ExInitializeFastMutex, ExInitializeResourceLite, _IO_STATUS_BLOCK, IoDeleteDevice, IoFreeIrp, IoGetCurrentIrpStackLocation, PDEVICE_OBJECT, PFILE_LOCK, PIO_STACK_LOCATION, PLARGE_INTEGER, PRTL_GENERIC_ALLOCATE_ROUTINE, PRTL_GENERIC_COMPARE_ROUTINE, PRTL_GENERIC_FREE_ROUTINE, PRTL_GENERIC_TABLE, PVPB, RtlInitializeGenericTable, RtlLookupElementGenericTable, VPB};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CCB, CDROM_TOC_LARGE, CD_MCB_ENTRY, COMPOUND_DIRENT, DIRENT, DIRENT_ENUM_CONTEXT, FCB, FCB_DATA, FCB_INDEX, FCB_NONPAGED, FILE_ENUM_CONTEXT, FILE_ID, IRP_CONTEXT, IRP_CONTEXT_FLAG_IN_TEARDOWN, IRP_CONTEXT_LITE, NODE_TYPE_CODE, PATH_ENTRY, PFCB, TRACK_DATA, VCB, VCB_STATE_AUDIO_DISK, VCB_STATE_CDXA, VCB_STATE_REMOVABLE_MEDIA, VOLUME_DEVICE_OBJECT, _VCB_CONDITION_VcbMountInProgress as VcbMountInProgress};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, ExFreePoolWithTag, FAST_MUTEX, FSRTL_ADVANCED_FCB_HEADER, OPLOCK, RTL_GENERIC_TABLE, _RTL_GENERIC_COMPARE_RESULTS_GenericEqual as GenericEqual, _RTL_GENERIC_COMPARE_RESULTS_GenericGreaterThan as GenericGreaterThan, _RTL_GENERIC_COMPARE_RESULTS_GenericLessThan as GenericLessThan, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlInitializeOplock, FsRtlNotifyInitializeSync, FsRtlNotifyUninitializeSync, FsRtlTeardownPerStreamContexts};
use crate::glue::{CDFS_NTC_FCB_DATA, CDFS_NTC_FCB_INDEX, CDFS_NTC_FCB_NONPAGED, CDFS_NTC_FCB_PATH_TABLE, CDFS_NTC_VCB, CDFS_RESIDUAL_REFERENCE, TAG_VPB};
use crate::types::{RAW_DIRENT, RAW_PATH_ISO};
use ntkrnl_sys::ddk::{PopEntryList, PIRP};
// resolve_imports.py: explicit kernel/private-type imports
use crate::cddata::{CdData};
use crate::ddk_shims::{FsRtlFreeFileLock, FsRtlUninitializeOplock};
use crate::glue::{CdNonPagedPool};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{CDROM_READ_TOC_EX};

extern "C" {
    fn SetFlag(Flags: *mut u32, Flag: u32);
}
extern "C" {
    fn CdAllocateFcbIndex(IrpContext: *mut IRP_CONTEXT) -> *mut FCB;
}
extern "C" {
    fn CdAllocateFcbData(IrpContext: *mut IRP_CONTEXT) -> *mut FCB;
}
extern "C" {
    fn CdBugCheck(a: usize, b: usize, c: usize);
}
extern "C" {
    fn CdFreePool(P: *mut *mut core::ffi::c_void);
}
extern "C" {
    fn CdGetFcbOplock(Fcb: *mut FCB) -> *mut OPLOCK;
}
extern "C" {
    fn CdInitializeMcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
    fn CdRaiseStatus(IrpContext: *mut IRP_CONTEXT, Status: NTSTATUS);
}
extern "C" {
    fn FsRtlSetupAdvancedHeader( Header: *mut FSRTL_ADVANCED_FCB_HEADER, Mutex: *mut FAST_MUTEX, );
}
extern "C" {
    fn _SEH2_AbnormalTermination() -> i32;
}
extern "C" {
    fn CdUpdateMediaChangeCount(Vcb: *mut VCB, MediaChangeCount: ULONG);
}
extern "C" {
    fn CdUpdateVcbCondition(Vcb: *mut VCB, condition: u32); // assuming VcbMountInProgress is u32 enum fn FlagOn(Flags: ULONG, Flag: ULONG) -> i32;
}
extern "C" {
    fn BytesFromBlocks(Vcb: *mut VCB, BlockOffset: u32) -> u32;
}
extern "C" {
    fn SectorBlockOffset(Vcb: *mut VCB, DiskOffset: u32) -> u32;
}
extern "C" {
    // 4th arg DataLength is i64 in the definition (allocsup.rs), not u32 — a u32 here makes
    // the i386 callee read 4 garbage stack bytes for the high dword (see cachesup note).
    fn CdAddInitialAllocation(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, StartingBlock: u32, DataLength: i64);
}
extern "C" {
    fn CdIncrementReferenceCounts(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, RefInc: i32, UserRefInc: i32);
}
extern "C" {
    fn InsertTailList(ListHead: *mut LIST_ENTRY, Entry: *mut LIST_ENTRY);
}
extern "system" {
    fn RtlInsertElementGenericTable( Table: *mut RTL_GENERIC_TABLE, Buffer: *mut core::ffi::c_void, BufferSize: u32, NewElement: *mut core::ffi::c_void, ) -> *mut core::ffi::c_void;
}
extern "C" {
    fn CdAddAllocationFromDirent(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, McbEntryOffset: u32, FileOffset: i64, Dirent: *mut DIRENT);
}
extern "C" {
    fn CdConvertCdTimeToNtTime(IrpContext: *mut IRP_CONTEXT, CdTime: *mut i8, Time: *mut LARGE_INTEGER);
}
extern "C" {
    fn CdLockFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
    fn CdLookupNextDirent(IrpContext: *mut IRP_CONTEXT, ParentFcb: *mut FCB, CurrentDirContext: *mut DIRENT_ENUM_CONTEXT, PreviousDirContext: *const DIRENT_ENUM_CONTEXT) -> i32;
}
extern "C" {
    fn CdUnlockFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
    fn CdUpdateDirentFromRawDirent(IrpContext: *mut IRP_CONTEXT, ParentFcb: *mut FCB, DirContext: *const DIRENT_ENUM_CONTEXT, Dirent: *mut DIRENT);
}
extern "C" {
    fn LlBlockAlign(Vcb: *mut VCB, FileSize: i64) -> i64;
}
extern "C" {
    fn CdAllocateCcb(IrpContext: *mut IRP_CONTEXT) -> *mut CCB;
}
extern "C" {
    fn FlagOn(Flags: u32, Flag: u32) -> i32;
}
extern "C" {
    fn ClearFlag(Flags: *mut u32, Mask: u32);
}
extern "C" {
    fn CdRestoreThreadContext(IrpContext: *mut IRP_CONTEXT);
}
extern "C" {
    fn CdFreeIoContext(IoContext: *mut c_void);
}
extern "C" {
    fn CdLockCdData();
}
extern "C" {
    fn CdUnlockCdData();
}
extern "C" {
    fn PushEntryList(ListHead: *mut SINGLE_LIST_ENTRY, Entry: *mut SINGLE_LIST_ENTRY);
}
extern "system" {
    fn RtlEnumerateGenericTableWithoutSplaying( Table: *mut RTL_GENERIC_TABLE, RestartKey: *mut *mut core::ffi::c_void, ) -> *mut core::ffi::c_void;
}
extern "C" {
    // CdPerformDevIoCtrlEx is plain cdecl (no FASTCALL in cdprocs.h).
    fn CdPerformDevIoCtrlEx( IrpContext: *mut IRP_CONTEXT, IoControlCode: u32, DeviceObject: *mut DEVICE_OBJECT, InputBuffer: *mut u8, InputBufferLength: usize, OutputBuffer: *mut u8, OutputBufferLength: usize, InternalDeviceIoControl: u8, Wait: u8, IoStatusBlock: *mut _IO_STATUS_BLOCK, ) -> i32;
}
extern "C" {
    fn CdAllocateFcbNonpaged(IrpContext: *mut IRP_CONTEXT) -> *mut FCB_NONPAGED;
}
extern "system" {
    fn FsRtlAllocatePoolWithTag(PoolType: u32, NumberOfBytes: u32, Tag: u32) -> *mut core::ffi::c_void;
}
extern "C" {
    fn SwapCopyUchar4(dest: *mut u32, src: *const u8);
}
extern "C" {
    fn CdLbnToMmSsFf(lbn: u32, dest: *mut u8);
}
extern "C" {
}
extern "C" {
    fn CdUninitializeMcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "system" {
    fn CdDeallocateFcbNonpaged(IrpContext: *mut IRP_CONTEXT, FcbNonpaged: *mut FCB_NONPAGED);
}
extern "C" {
    fn CdDeallocateFcbIndex(IrpContext: *mut IRP_CONTEXT, fcb: *mut core::ffi::c_void);
}
extern "C" {
    fn CdDeallocateFcbData(IrpContext: *mut IRP_CONTEXT, fcb: *mut core::ffi::c_void);
}
extern "C" {
    fn CdCreateInternalStream( IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Fcb: *mut FCB, Name: *const _UNICODE_STRING, );
}
extern "C" {
    fn CdFidSetDirectory(FileId: *mut FILE_ID);
}
extern "C" {
    fn CdLockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
}
extern "C" {
    fn CdRvdBlkSz(RawIsoVd: *mut i8, VcbState: u32) -> u32;
}
extern "C" {
    fn CdRvdDirent(RawIsoVd: *mut i8, VcbState: u32) -> *mut core::ffi::c_void;
}
extern "C" {
    fn CdRvdPtLoc(RawIsoVd: *mut i8, VcbState: u32) -> u32;
}
extern "C" {
    fn CdRvdPtSz(RawIsoVd: *mut i8, VcbState: u32) -> u32;
}
extern "C" {
    fn CdRvdVolSz(RawIsoVd: *mut i8, VcbState: u32) -> u32;
}
extern "C" {
    fn CdSetFidPathTableOffset(FileId: *mut FILE_ID, Offset: LONGLONG);
}
extern "C" {
    fn CdUnlockVcb(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB);
}
extern "C" {
    fn CopyUchar4(Dst: *mut u32, Src: *const u8);
}
extern "C" {
    fn RtlEqualMemory(B1: *const core::ffi::c_void, B2: *const core::ffi::c_void, Length: usize) -> i32;
}
extern "C" {
    fn LlBytesFromBlocks(Vcb: *mut VCB, Block: u32) -> LONGLONG;
}
extern "C" {
    fn LlSectorAlign(Value: LONGLONG) -> LONGLONG;
}
extern "C" {
    fn SectorAlign(Value: u32) -> u32;
}
extern "system" {
    static CdXaId: [u8; 8];
}
extern "system" {
    static CdAudioLabel: [u8; 32];
}
extern "C" {
    fn CdAcquireFcbExclusive(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, Wait: u8) -> u32;
}
extern "C" {
    fn CdDecrementReferenceCounts(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB, RefCount: u32, UserRefCount: u32);
}
extern "C" {
    fn CdDeleteFcbTable(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
    fn CdDeleteInternalStream(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
    fn CdReleaseFcb(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
    fn CdRemovePrefix(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
}
extern "C" {
    fn SafeNodeType(Node: *mut FCB) -> u32;
}
extern "system" {
    // PFILE_LOCK NTAPI FsRtlAllocateFileLock(PCOMPLETE_LOCK_IRP_ROUTINE, PUNLOCK_ROUTINE);
    fn FsRtlAllocateFileLock(
        CompleteLockIrpRoutine: *mut core::ffi::c_void,
        UnlockRoutine: *mut core::ffi::c_void,
    ) -> PFILE_LOCK;
}

// ---- file-scope preamble (translated) ----
// NOTE: the real FCB / VCB / FCB_NONPAGED / IRP_CONTEXT / FILE_ID layouts come from
// crate::types (exact bindgen substrate), pulled in by resolve_imports.py. The grind's
// ZST placeholders (`pub struct FCB;` etc.) were struct-shadow stubs — the same class
// the fastfat lane killed in 4e736e07d1f — and have been removed so field access resolves
// to the real structs instead of cascading E0609/E0610.

// The C FCB has two anonymous unions that bindgen collapsed to opaque storage:
//   __bindgen_anon_1 = { FSRTL_ADVANCED_FCB_HEADER Header; <unnamed header> }
//   __bindgen_anon_2 = { ULONG FcbType; FCB_DATA; FCB_INDEX }
// These helpers reinterpret that storage as the real substrate structs so the
// C field accesses (Fcb->NodeTypeCode, Fcb->FileSize, Fcb->FcbQueue, ...) resolve.
#[inline(always)]
unsafe fn fcb_hdr(f: *mut FCB) -> *mut FSRTL_ADVANCED_FCB_HEADER {
    core::ptr::addr_of_mut!((*f).__bindgen_anon_1) as *mut FSRTL_ADVANCED_FCB_HEADER
}
#[inline(always)]
unsafe fn fcb_idx(f: *mut FCB) -> *mut FCB_INDEX {
    core::ptr::addr_of_mut!((*f).__bindgen_anon_2) as *mut FCB_INDEX
}
#[inline(always)]
unsafe fn fcb_data(f: *mut FCB) -> *mut FCB_DATA {
    core::ptr::addr_of_mut!((*f).__bindgen_anon_2) as *mut FCB_DATA
}

// SIZEOF_FCB_DATA / SIZEOF_FCB_INDEX: C macros =
//   FIELD_OFFSET(FCB, FcbType) + sizeof(FCB_DATA|FCB_INDEX)
// FcbType is the first member of __bindgen_anon_2, so its offset == offset_of anon_2.
const SIZEOF_FCB_DATA: u32 =
    (core::mem::offset_of!(FCB, __bindgen_anon_2) + core::mem::size_of::<FCB_DATA>()) as u32;
const SIZEOF_FCB_INDEX: u32 =
    (core::mem::offset_of!(FCB, __bindgen_anon_2) + core::mem::size_of::<FCB_INDEX>()) as u32;

#[repr(C)]
pub struct FCB_TABLE_ELEMENT {
    pub FileId: FILE_ID,
    pub Fcb: PFCB,
}

pub type PFCB_TABLE_ELEMENT = *mut FCB_TABLE_ELEMENT;

pub static mut CdInternalStreamNames: [_UNICODE_STRING; 2] = [
    _UNICODE_STRING {
        Length: 24,
        MaximumLength: 24,
        Buffer: CD_INTERNAL_STREAM_NAMES_0.as_ptr() as *mut u16,
    },
    _UNICODE_STRING {
        Length: 2,
        MaximumLength: 2,
        Buffer: CD_INTERNAL_STREAM_NAMES_1.as_ptr() as *mut u16,
    },
];

static CD_INTERNAL_STREAM_NAMES_0: [u16; 13] = [
    '$' as u16, 'P' as u16, 'A' as u16, 'T' as u16, 'H' as u16, '_' as u16,
    'T' as u16, 'A' as u16, 'B' as u16, 'L' as u16, 'E' as u16, '$' as u16,
    0u16,
];

static CD_INTERNAL_STREAM_NAMES_1: [u16; 2] = ['\\' as u16, 0u16];

const FILE_ATTRIBUTE_READONLY: u32 = 0x00000001;
const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;
const CD_ATTRIBUTE_HIDDEN: u32 = 0x01;
const Form1Data: u32 = 0;
const Mode2Form2Data: u32 = 1;
const FCB_STATE_MODE2FORM2_FILE: u32 = 0x04;
const FCB_STATE_DA_FILE: u32 = 0x10;
const FCB_STATE_INITIALIZED: u32 = 0x01;
const FCB_STATE_IN_FCB_TABLE: u32 = 0x02;
const CD_ATTRIBUTE_MULTI: u32 = 0x80;
const STATUS_FILE_CORRUPT_ERROR: i32 = 0xC0000102u32 as i32;

const IRP_CONTEXT_FLAG_MORE_PROCESSING: u32 = 0x02;
const IRP_CONTEXT_FLAG_ALLOC_IO: u32 = 0x0100;
const IRP_CONTEXT_FLAG_ON_STACK: u32 = 0x01;
const IRP_CONTEXT_FLAGS_CLEAR_ON_POST: u32 = 0x00000008;
const IRP_CONTEXT_FLAGS_CLEAR_ON_RETRY: u32 = 0x00000010;

const IOCTL_CDROM_READ_TOC: u32 = 0x00024000;
const IOCTL_CDROM_READ_TOC_EX: u32 = 0x24054;
const STATUS_INVALID_DEVICE_REQUEST: i32 = 0xC0000010u32 as i32;
const STATUS_NOT_IMPLEMENTED: i32 = 0xC0000002u32 as i32;
const STATUS_INVALID_PARAMETER: i32 = 0xC000000Du32 as i32;
const STATUS_DISK_CORRUPT_ERROR: i32 = 0xC0000032u32 as i32;
const TOC_DATA_TRACK: u32 = 0x04;
const CDROM_DISK_AUDIO_TRACK: u32 = 0x00000001;
const CDROM_DISK_DATA_TRACK: u32 = 0x02;
const TOC_LAST_TRACK: u32 = 0xAA;

#[repr(C)]
union BiasedTocLength {
    BigEndian: [u8; 2],
    Length: u16,
}

// Pool tags: cdprocs.h multichar constants, used directly as their u32 value.
const TAG_FCB_NONPAGED: u32 = 0x6e666443; // nolint: const-contradiction ('nfdC' -- cdfs/cdprocs.h; fastfat's is 'NtaF')

const CdPagedPool: u32 = 1; // PagedPool
const TAG_FCB_TABLE: u32 = 0x74666443; // 'tfdC'  (TAG_FCB_TABLE)

// cdprocs.h: TAG_IRP_CONTEXT = 'cidC' = 0x63696443 (same multichar scheme as TAG_VPB).
const TAG_IRP_CONTEXT: u32 = 0x63696443; // nolint: const-contradiction ('cidC' -- cdfs/cdprocs.h; fastfat's is 'ItaF')
// cdstruc.h IRP_CONTEXT_FLAG_FORCE_POST = 0x00000008 (types_i386.rs).
const IRP_CONTEXT_FLAG_FORCE_POST: u32 = 0x00000008;
// nodetype.h CDFS_NTC_IRP_CONTEXT (also in glue.rs).
const CDFS_NTC_IRP_CONTEXT_LOCAL: u16 = crate::glue::CDFS_NTC_IRP_CONTEXT as u16;
// iotypes.h IRP_MJ codes used by the fsdo sanity assert.
const IRP_MJ_CREATE_LOCAL: u8 = 0x00;
const IRP_MJ_CLOSE_LOCAL: u8 = 0x02;
const IRP_MJ_CLEANUP_LOCAL: u8 = 0x12;

// CdCreateIrpContext — translated from drivers/filesystems/cdfs/strucsup.c
// (the grind dropped it). Allocates+initializes an IRP_CONTEXT from the Irp.
// cdecl to match its C original (no NTAPI on the C definition); lld bridges the
// stdcall-decorated call-site decls on i386.
#[no_mangle]
pub unsafe extern "C" fn CdCreateIrpContext(Irp: PIRP, Wait: BOOLEAN) -> *mut IRP_CONTEXT {
    let mut NewIrpContext: *mut IRP_CONTEXT = core::ptr::null_mut();
    let IrpSp: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    PAGED_CODE!();

    // The only operations a filesystem device object should receive are
    // create/teardown of fsdo handles and non-fileobject operations (mount).
    if (*IrpSp).DeviceObject == CdData.FileSystemDeviceObject
        || (*IrpSp).DeviceObject == CdData.HddFileSystemDeviceObject
    {
        if !(*IrpSp).FileObject.is_null()
            && (*IrpSp).MajorFunction != IRP_MJ_CREATE_LOCAL
            && (*IrpSp).MajorFunction != IRP_MJ_CLEANUP_LOCAL
            && (*IrpSp).MajorFunction != IRP_MJ_CLOSE_LOCAL
        {
            ntkrnl_sys::ddk::ExRaiseStatus(STATUS_INVALID_DEVICE_REQUEST);
        }
        // NT_ASSERT(...) — retail no-op.
    }

    // Look in our lookaside list for an IrpContext.
    if CdData.IrpContextDepth != 0 {
        CdLockCdData();
        NewIrpContext =
            PopEntryList(core::ptr::addr_of_mut!(CdData.IrpContextList)) as *mut IRP_CONTEXT;
        if !NewIrpContext.is_null() {
            CdData.IrpContextDepth -= 1;
        }
        CdUnlockCdData();
    }

    if NewIrpContext.is_null() {
        // Not on the private list; allocate from pool.
        NewIrpContext = FsRtlAllocatePoolWithTag(
            CdNonPagedPool as u32,
            core::mem::size_of::<IRP_CONTEXT>() as u32,
            TAG_IRP_CONTEXT,
        ) as *mut IRP_CONTEXT;
    }

    RtlZeroMemory(NewIrpContext as PVOID, core::mem::size_of::<IRP_CONTEXT>() as SIZE_T);

    (*NewIrpContext).NodeTypeCode = CDFS_NTC_IRP_CONTEXT_LOCAL;
    (*NewIrpContext).NodeByteSize = core::mem::size_of::<IRP_CONTEXT>() as crate::types::NODE_BYTE_SIZE;
    (*NewIrpContext).Irp = Irp;

    // Copy RealDevice for workque algorithms (updated later in Mount/Verify).
    if !(*IrpSp).FileObject.is_null() {
        (*NewIrpContext).RealDevice = (*(*IrpSp).FileObject).DeviceObject;
    }

    // Locate the volume device object and Vcb; skip if this is the fsdo.
    if (*IrpSp).DeviceObject != CdData.FileSystemDeviceObject
        && (*IrpSp).DeviceObject != CdData.HddFileSystemDeviceObject
    {
        (*NewIrpContext).Vcb =
            core::ptr::addr_of_mut!((*((*IrpSp).DeviceObject as *mut VOLUME_DEVICE_OBJECT)).Vcb);
    }

    (*NewIrpContext).MajorFunction = (*IrpSp).MajorFunction;
    (*NewIrpContext).MinorFunction = (*IrpSp).MinorFunction;

    if Wait != 0 {
        SetFlag(core::ptr::addr_of_mut!((*NewIrpContext).Flags), IRP_CONTEXT_FLAG_WAIT);
    } else {
        SetFlag(core::ptr::addr_of_mut!((*NewIrpContext).Flags), IRP_CONTEXT_FLAG_FORCE_POST);
    }

    NewIrpContext
}

#[no_mangle]
pub unsafe extern "C" fn CdInitializeVcb(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    TargetDeviceObject: *mut DEVICE_OBJECT,
    Vpb: *mut VPB,
    CdromToc: *mut CDROM_TOC_LARGE,
    TocLength: ULONG,
    TocTrackCount: ULONG,
    TocDiskFlags: ULONG,
    BlockFactor: ULONG,
    MediaChangeCount: ULONG,
) {
    // PAGED_CODE();
    // UNREFERENCED_PARAMETER( IrpContext );
    UNREFERENCED_PARAMETER!(IrpContext);

    // RtlZeroMemory( Vcb, sizeof( VCB ));
    core::ptr::write_bytes(Vcb as *mut u8, 0, core::mem::size_of::<VCB>());

    // Vcb->NodeTypeCode = CDFS_NTC_VCB;
    // Vcb->NodeByteSize = sizeof( VCB );
    (*Vcb).NodeTypeCode = CDFS_NTC_VCB as u16;
    (*Vcb).NodeByteSize = core::mem::size_of::<VCB>() as i16;

    // InitializeListHead( &Vcb->DirNotifyList );
    InitializeListHead(core::ptr::addr_of_mut!((*Vcb).DirNotifyList));

    // FsRtlNotifyInitializeSync( &Vcb->NotifySync );
    FsRtlNotifyInitializeSync(core::ptr::addr_of_mut!((*Vcb).NotifySync));

    // try { allocate the swap VPB (raises on failure) } finally { if (AbnormalTermination())
    // undo the notify sync } — a real __finally (ntkrnl_sys::seh), so the abnormal arm runs.
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            (*Vcb).SwapVpb = FsRtlAllocatePoolWithTag(
                CdNonPagedPool,
                core::mem::size_of::<VPB>() as ULONG,
                TAG_VPB,
            ) as *mut _;
        },
        |_, abnormal| {
            if abnormal {
                FsRtlNotifyUninitializeSync(core::ptr::addr_of_mut!((*Vcb).NotifySync));
            }
        },
    );

    // RtlZeroMemory( Vcb->SwapVpb, sizeof( VPB ) );
    core::ptr::write_bytes(
        (*Vcb).SwapVpb as *mut u8,
        0,
        core::mem::size_of::<VPB>(),
    );

    // ExInitializeResourceLite( &Vcb->VcbResource );
    ExInitializeResourceLite(core::ptr::addr_of_mut!((*Vcb).VcbResource));
    // ExInitializeResourceLite( &Vcb->FileResource );
    ExInitializeResourceLite(core::ptr::addr_of_mut!((*Vcb).FileResource));
    // ExInitializeFastMutex( &Vcb->VcbMutex );
    ExInitializeFastMutex(core::ptr::addr_of_mut!((*Vcb).VcbMutex));

    // InsertHeadList( &CdData.VcbQueue, &Vcb->VcbLinks );
    InsertHeadList(
        core::ptr::addr_of_mut!(CdData.VcbQueue) as *mut _,
        (*Vcb).__bindgen_anon_1.VcbLinks.as_mut() as *mut _,
    );

    // ObReferenceObject( TargetDeviceObject );
    ObReferenceObject(TargetDeviceObject as *mut core::ffi::c_void);
    (*Vcb).TargetDeviceObject = TargetDeviceObject;
    (*Vcb).Vpb = Vpb;

    // if (FlagOn( Vpb->RealDevice->Characteristics, FILE_REMOVABLE_MEDIA )) {
    if FlagOn(
        (*(*Vpb).RealDevice).Characteristics,
        FILE_REMOVABLE_MEDIA,
    ) != 0
    {
        SetFlag(&mut (*Vcb).VcbState, VCB_STATE_REMOVABLE_MEDIA);
    }

    // RtlInitializeGenericTable( &Vcb->FcbTable, ... )
    RtlInitializeGenericTable(
        core::ptr::addr_of_mut!((*Vcb).FcbTable),
        // The Rtl generic-table callback ABI is stdcall on i386; the cdfs callbacks are
        // declared extern "C". Transmute the fn items to the bindgen Option<fn> type.
        core::mem::transmute::<_, PRTL_GENERIC_COMPARE_ROUTINE>(CdFcbTableCompare as *const ()),
        core::mem::transmute::<_, PRTL_GENERIC_ALLOCATE_ROUTINE>(CdAllocateFcbTable as *const ()),
        core::mem::transmute::<_, PRTL_GENERIC_FREE_ROUTINE>(CdDeallocateFcbTable as *const ()),
        core::ptr::null_mut(),
    );

    // CdUpdateVcbCondition( Vcb, VcbMountInProgress);
    CdUpdateVcbCondition(Vcb, VcbMountInProgress);

    // Vcb->VcbReference = 1 + CDFS_RESIDUAL_REFERENCE;
    (*Vcb).VcbReference = (1 + CDFS_RESIDUAL_REFERENCE) as i32;

    // Vcb->CdromToc = CdromToc; ...
    (*Vcb).CdromToc = CdromToc;
    (*Vcb).TocLength = TocLength;
    (*Vcb).TrackCount = TocTrackCount;
    (*Vcb).DiskFlags = TocDiskFlags;

    // if (TocDiskFlags == CDROM_DISK_AUDIO_TRACK) { SetFlag(...) }
    if TocDiskFlags == CDROM_DISK_AUDIO_TRACK {
        SetFlag(
            &mut (*Vcb).VcbState,
            VCB_STATE_AUDIO_DISK | VCB_STATE_CDXA,
        );
    }

    // Vcb->BlockFactor = BlockFactor;
    (*Vcb).BlockFactor = BlockFactor;

    // CdUpdateMediaChangeCount( Vcb, MediaChangeCount);
    CdUpdateMediaChangeCount(Vcb, MediaChangeCount);
}

// Forward declarations of unknown externs
#[no_mangle]
pub unsafe extern "C" fn CdUpdateVcbFromVolDescriptor(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    RawIsoVd: *mut i8,
) {
    const SECTOR_SIZE: u32 = 2048;
    const SECTOR_SHIFT: u32 = 11;
    const FILE_ATTRIBUTE_READONLY: u32 = 1;
    const CDFS_NTC_FCB_PATH_TABLE: NODE_TYPE_CODE = 0x0303;
    const CDFS_NTC_FCB_INDEX: NODE_TYPE_CODE = 0x0304;
    const CDFS_NTC_FCB_DATA: NODE_TYPE_CODE = 0x0305;
    const VCB_STATE_ISO: u32 = 0x02;
    const VCB_STATE_JOLIET: u32 = 0x04;
    const VCB_STATE_CDXA: u32 = 0x40;
    const VCB_STATE_AUDIO_DISK: u32 = 0x80;
    const FCB_STATE_INITIALIZED: u32 = 0x01;
    const STATUS_DISK_CORRUPT_ERROR: i32 = 0xC0000032u32 as i32;
    const CdAudioDirentSize: u32 = 32;
    const CdAudioLabelLength: u32 = 32;

    let mut UnlockVcb: bool = false;
    let mut StartingBlock: u32 = 0;
    let mut ByteCount: u32 = 0;
    let FileId: LONGLONG = 0;
    let mut RawDirent: *mut RAW_DIRENT = core::ptr::null_mut();
    let mut PathEntry: PATH_ENTRY = core::mem::zeroed();
    let mut McbEntry: *mut CD_MCB_ENTRY = core::ptr::null_mut();

    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    let p_UnlockVcb = core::ptr::addr_of_mut!(UnlockVcb);
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            // _SEH2_TRY
            {
                // Block size and masks
                (*Vcb).BlockSize = if !RawIsoVd.is_null() {
                    CdRvdBlkSz(RawIsoVd, (*Vcb).VcbState)
                } else {
                    SECTOR_SIZE
                };

                if (*Vcb).BlockSize != SECTOR_SIZE {
                    if UnlockVcb {
                        CdUnlockVcb(IrpContext, Vcb);
                        *p_UnlockVcb = false;
                    }
                    CdRaiseStatus(IrpContext, STATUS_DISK_CORRUPT_ERROR);
                }

                let block_size = (*Vcb).BlockSize;
                (*Vcb).BlocksPerSector = SECTOR_SIZE / block_size;
                (*Vcb).BlockMask = block_size - 1;
                (*Vcb).BlockInverseMask = !(*Vcb).BlockMask;
                (*Vcb).BlockToSectorShift = 0;
                (*Vcb).BlockToByteShift = SECTOR_SHIFT;

                if !RawIsoVd.is_null() {
                    // ISO volume descriptor present
                    CdLockVcb(IrpContext, Vcb);
                    *p_UnlockVcb = true;

                    let path_table_fcb = CdCreateFcb(
                        IrpContext,
                        *(core::ptr::addr_of!(FileId) as *const FILE_ID),
                        CDFS_NTC_FCB_PATH_TABLE,
                        core::ptr::null_mut(),
                    );
                    (*Vcb).PathTableFcb = path_table_fcb;
                    CdIncrementReferenceCounts(IrpContext, path_table_fcb, 1, 1);
                    CdUnlockVcb(IrpContext, Vcb);
                    *p_UnlockVcb = false;

                    StartingBlock = CdRvdPtLoc(RawIsoVd, (*Vcb).VcbState);
                    ByteCount = CdRvdPtSz(RawIsoVd, (*Vcb).VcbState);

                    let stream_offset = BytesFromBlocks(
                        Vcb,
                        SectorBlockOffset(Vcb, StartingBlock),
                    );
                    (*fcb_idx(path_table_fcb)).StreamOffset = stream_offset;
                    (*fcb_hdr(path_table_fcb)).FileSize.QuadPart = (stream_offset + ByteCount) as LONGLONG;
                    (*fcb_hdr(path_table_fcb)).ValidDataLength.QuadPart = (*fcb_hdr(path_table_fcb)).FileSize.QuadPart;
                    (*fcb_hdr(path_table_fcb)).AllocationSize.QuadPart =
                        LlSectorAlign((*fcb_hdr(path_table_fcb)).FileSize.QuadPart);

                    CdLockFcb(IrpContext, path_table_fcb);
                    CdAddInitialAllocation(
                        IrpContext,
                        path_table_fcb,
                        StartingBlock,
                        (*fcb_hdr(path_table_fcb)).AllocationSize.QuadPart, // DataLength: i64
                    );
                    CdUnlockFcb(IrpContext, path_table_fcb);

                    (*fcb_hdr(path_table_fcb)).Resource = &raw mut (*Vcb).FileResource as *mut _;
                    (*path_table_fcb).FcbState |= FCB_STATE_INITIALIZED;
                    CdCreateInternalStream(
                        IrpContext,
                        Vcb,
                        path_table_fcb,
                        &raw const CdInternalStreamNames[0],
                    );

                    // Root index
                    CdLockVcb(IrpContext, Vcb);
                    *p_UnlockVcb = true;
                    let root_index_fcb = CdCreateFcb(
                        IrpContext,
                        *(core::ptr::addr_of!(FileId) as *const FILE_ID),
                        CDFS_NTC_FCB_INDEX,
                        core::ptr::null_mut(),
                    );
                    (*Vcb).RootIndexFcb = root_index_fcb;
                    CdIncrementReferenceCounts(IrpContext, root_index_fcb, 1, 1);
                    CdUnlockVcb(IrpContext, Vcb);
                    *p_UnlockVcb = false;

                    CdSetFidPathTableOffset(
                        &raw mut (*root_index_fcb).FileId,
                        (*fcb_idx(path_table_fcb)).StreamOffset as i64,
                    );
                    CdFidSetDirectory(&raw mut (*root_index_fcb).FileId);

                    RawDirent = CdRvdDirent(RawIsoVd, (*Vcb).VcbState) as *mut RAW_DIRENT;
                    CopyUchar4(
                        &raw mut PathEntry.DiskOffset,
                        core::ptr::addr_of!((*RawDirent).FileLoc) as *const u8,
                    );
                    PathEntry.DiskOffset += (*RawDirent).XarLen as u32;
                    PathEntry.Ordinal = 1;
                    PathEntry.PathTableOffset = (*fcb_idx(path_table_fcb)).StreamOffset;

                    CdInitializeFcbFromPathEntry(
                        IrpContext,
                        root_index_fcb,
                        core::ptr::null_mut(),
                        &raw mut PathEntry,
                    );

                    CdCreateInternalStream(
                        IrpContext,
                        Vcb,
                        root_index_fcb,
                        &raw const CdInternalStreamNames[1],
                    );

                    // Volume dasd
                    CdLockVcb(IrpContext, Vcb);
                    *p_UnlockVcb = true;
                    let volume_dasd_fcb = CdCreateFcb(
                        IrpContext,
                        *(core::ptr::addr_of!(FileId) as *const FILE_ID),
                        CDFS_NTC_FCB_DATA,
                        core::ptr::null_mut(),
                    );
                    (*Vcb).VolumeDasdFcb = volume_dasd_fcb;
                    CdIncrementReferenceCounts(IrpContext, volume_dasd_fcb, 1, 1);
                    CdUnlockVcb(IrpContext, Vcb);
                    *p_UnlockVcb = false;

                    StartingBlock = CdRvdVolSz(RawIsoVd, (*Vcb).VcbState);
                    (*fcb_hdr(volume_dasd_fcb)).FileSize.QuadPart = LlBytesFromBlocks(Vcb, StartingBlock);
                    (*fcb_hdr(volume_dasd_fcb)).AllocationSize.QuadPart = (*fcb_hdr(volume_dasd_fcb)).FileSize.QuadPart;
                    (*fcb_hdr(volume_dasd_fcb)).ValidDataLength.QuadPart = (*fcb_hdr(volume_dasd_fcb)).FileSize.QuadPart;

                    CdLockFcb(IrpContext, volume_dasd_fcb);
                    McbEntry = core::ptr::addr_of!((*volume_dasd_fcb).Mcb.McbArray) as *mut CD_MCB_ENTRY;
                    (*McbEntry).FileOffset = 0;
                    (*McbEntry).DiskOffset = 0;
                    (*McbEntry).ByteCount = (*fcb_hdr(volume_dasd_fcb)).AllocationSize.QuadPart;
                    (*McbEntry).DataBlockByteCount = (*McbEntry).ByteCount;
                    (*McbEntry).TotalBlockByteCount = (*McbEntry).ByteCount;
                    (*volume_dasd_fcb).Mcb.CurrentEntryCount = 1;
                    CdUnlockFcb(IrpContext, volume_dasd_fcb);

                    (*fcb_hdr(volume_dasd_fcb)).Resource = &raw mut (*Vcb).FileResource as *mut _;
                    (*volume_dasd_fcb).FileAttributes = FILE_ATTRIBUTE_READONLY;
                    (*volume_dasd_fcb).FcbState |= FCB_STATE_INITIALIZED;

                    // Check for XA disc
                    if ((*Vcb).VcbState & (VCB_STATE_ISO | VCB_STATE_JOLIET)) != 0
                        && RtlEqualMemory(
                            core::ptr::addr_of!(CdXaId) as *const core::ffi::c_void,
                            ((RawIsoVd as usize) + 0x400) as *const core::ffi::c_void,
                            8,
                        ) != 0
                    {
                        (*Vcb).VcbState |= VCB_STATE_CDXA;
                    }
                } else if ((*Vcb).VcbState & VCB_STATE_AUDIO_DISK) != 0 {
                    // Audio disk branch
                    CdLockVcb(IrpContext, Vcb);
                    *p_UnlockVcb = true;

                    let path_table_fcb = CdCreateFcb(
                        IrpContext,
                        *(core::ptr::addr_of!(FileId) as *const FILE_ID),
                        CDFS_NTC_FCB_PATH_TABLE,
                        core::ptr::null_mut(),
                    );
                    (*Vcb).PathTableFcb = path_table_fcb;
                    CdIncrementReferenceCounts(IrpContext, path_table_fcb, 1, 1);
                    CdUnlockVcb(IrpContext, Vcb);
                    *p_UnlockVcb = false;

                    let dir_id_offset = core::mem::offset_of!(RAW_PATH_ISO, DirId) as LONGLONG;
                    (*fcb_hdr(path_table_fcb)).FileSize.QuadPart = dir_id_offset + 2;
                    (*fcb_hdr(path_table_fcb)).ValidDataLength.QuadPart = (*fcb_hdr(path_table_fcb)).FileSize.QuadPart;
                    (*fcb_hdr(path_table_fcb)).AllocationSize.QuadPart =
                        LlSectorAlign((*fcb_hdr(path_table_fcb)).FileSize.QuadPart);
                    (*fcb_hdr(path_table_fcb)).Resource = &raw mut (*Vcb).FileResource as *mut _;
                    (*path_table_fcb).FcbState |= FCB_STATE_INITIALIZED;
                    CdCreateInternalStream(
                        IrpContext,
                        Vcb,
                        path_table_fcb,
                        &raw const CdInternalStreamNames[0],
                    );

                    // Root index
                    CdLockVcb(IrpContext, Vcb);
                    *p_UnlockVcb = true;
                    let root_index_fcb = CdCreateFcb(
                        IrpContext,
                        *(core::ptr::addr_of!(FileId) as *const FILE_ID),
                        CDFS_NTC_FCB_INDEX,
                        core::ptr::null_mut(),
                    );
                    (*Vcb).RootIndexFcb = root_index_fcb;
                    CdIncrementReferenceCounts(IrpContext, root_index_fcb, 1, 1);
                    CdUnlockVcb(IrpContext, Vcb);
                    *p_UnlockVcb = false;

                    CdSetFidPathTableOffset(
                        &raw mut (*root_index_fcb).FileId,
                        (*fcb_idx(path_table_fcb)).StreamOffset as i64,
                    );
                    CdFidSetDirectory(&raw mut (*root_index_fcb).FileId);

                    core::ptr::write_bytes(
                        &raw mut PathEntry as *mut u8,
                        0,
                        core::mem::size_of::<PATH_ENTRY>(),
                    );
                    PathEntry.Ordinal = 1;
                    PathEntry.PathTableOffset = (*fcb_idx(path_table_fcb)).StreamOffset;

                    CdInitializeFcbFromPathEntry(
                        IrpContext,
                        root_index_fcb,
                        core::ptr::null_mut(),
                        &raw mut PathEntry,
                    );

                    let mut root_directory_size = ((*Vcb).TrackCount + 2) * CdAudioDirentSize;
                    root_directory_size = SectorAlign(root_directory_size);
                    let root_size = root_directory_size as LONGLONG;
                    (*fcb_hdr(root_index_fcb)).AllocationSize.QuadPart = root_size;
                    (*fcb_hdr(root_index_fcb)).ValidDataLength.QuadPart = root_size;
                    (*fcb_hdr(root_index_fcb)).FileSize.QuadPart = root_size;
                    (*root_index_fcb).FcbState |= FCB_STATE_INITIALIZED;

                    CdCreateInternalStream(
                        IrpContext,
                        Vcb,
                        root_index_fcb,
                        &raw const CdInternalStreamNames[1],
                    );

                    // Volume dasd
                    CdLockVcb(IrpContext, Vcb);
                    *p_UnlockVcb = true;
                    let volume_dasd_fcb = CdCreateFcb(
                        IrpContext,
                        *(core::ptr::addr_of!(FileId) as *const FILE_ID),
                        CDFS_NTC_FCB_DATA,
                        core::ptr::null_mut(),
                    );
                    (*Vcb).VolumeDasdFcb = volume_dasd_fcb;
                    CdIncrementReferenceCounts(IrpContext, volume_dasd_fcb, 1, 1);
                    CdUnlockVcb(IrpContext, Vcb);
                    *p_UnlockVcb = false;

                    (*fcb_hdr(volume_dasd_fcb)).Resource = &raw mut (*Vcb).FileResource as *mut _;
                    (*volume_dasd_fcb).FileAttributes = FILE_ATTRIBUTE_READONLY;
                    (*volume_dasd_fcb).FcbState |= FCB_STATE_INITIALIZED;

                    (*(*Vcb).Vpb).VolumeLabelLength = CdAudioLabelLength as u16;
                    core::ptr::copy_nonoverlapping(
                        core::ptr::addr_of!(CdAudioLabel) as *const u8,
                        (*(*Vcb).Vpb).VolumeLabel.as_mut_ptr() as *mut u8,
                        CdAudioLabelLength as usize,
                    );

                    let serial = CdTocSerial(IrpContext, (*Vcb).CdromToc);
                    (*(*Vcb).Vpb).SerialNumber = serial;

                    (*Vcb).VcbState |= VCB_STATE_ISO;
                }
            }
        },
        |_, _| {
            if *p_UnlockVcb  {
                CdUnlockVcb(IrpContext, Vcb);
            }
        },
    );

    // _SEH2_END
}
#[no_mangle]
pub unsafe extern "C" fn CdDeleteVcb(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
) {
    unsafe {
        PAGED_CODE!();
        ASSERT_EXCLUSIVE_CDDATA!();
        ASSERT_EXCLUSIVE_VCB!(Vcb);
        UNREFERENCED_PARAMETER!(IrpContext);

        CdFreePool(core::ptr::addr_of_mut!((*Vcb).SwapVpb) as *mut *mut core::ffi::c_void);
        CdFreePool(core::ptr::addr_of_mut!((*Vcb).Vpb) as *mut *mut core::ffi::c_void);

        if !(*Vcb).TargetDeviceObject.is_null() {
            ObDereferenceObject((*Vcb).TargetDeviceObject as *mut core::ffi::c_void);
        }

        CdFreePool(core::ptr::addr_of_mut!((*Vcb).XASector) as *mut *mut core::ffi::c_void);
        CdFreePool(core::ptr::addr_of_mut!((*Vcb).SectorCacheBuffer) as *mut *mut core::ffi::c_void);

        if !(*Vcb).SectorCacheIrp.is_null() {
            IoFreeIrp((*Vcb).SectorCacheIrp);
            (*Vcb).SectorCacheIrp = core::ptr::null_mut();
            ExDeleteResourceLite(core::ptr::addr_of_mut!((*Vcb).SectorCacheResource));
        }

        RemoveEntryList((*Vcb).__bindgen_anon_1.VcbLinks.as_mut());

        ExDeleteResourceLite(core::ptr::addr_of_mut!((*Vcb).VcbResource));
        ExDeleteResourceLite(core::ptr::addr_of_mut!((*Vcb).FileResource));

        CdFreePool(core::ptr::addr_of_mut!((*Vcb).CdromToc) as *mut *mut core::ffi::c_void);

        if !(*Vcb).NotifySync.is_null() {
            FsRtlNotifyUninitializeSync(core::ptr::addr_of_mut!((*Vcb).NotifySync));
        }

        let device_object = (Vcb as *mut u8)
            .sub(core::mem::offset_of!(VOLUME_DEVICE_OBJECT, Vcb))
            as *mut DEVICE_OBJECT;
        IoDeleteDevice(device_object);
    }
}

#[no_mangle]
pub unsafe extern "C" fn CdCreateFcb(
    IrpContext: *mut IRP_CONTEXT,
    FileId: FILE_ID,
    NodeTypeCode: NODE_TYPE_CODE,
    mut FcbExisted: *mut u8,
) -> *mut FCB {
    // PAGED_CODE() macro – emit the call
    PAGED_CODE!();

    let mut NewFcb: *mut FCB;
    let mut LocalFcbExisted: u8 = 0;

    if FcbExisted.is_null() {
        FcbExisted = core::ptr::addr_of_mut!(LocalFcbExisted);
    }

    NewFcb = CdLookupFcbTable(IrpContext, (*IrpContext).Vcb, FileId);

    if NewFcb.is_null() {
        match NodeTypeCode as u32 {
            CDFS_NTC_FCB_PATH_TABLE | CDFS_NTC_FCB_INDEX => {
                NewFcb = CdAllocateFcbIndex(IrpContext);
                RtlZeroMemory(NewFcb as *mut core::ffi::c_void, SIZEOF_FCB_INDEX as usize);
                (*fcb_hdr(NewFcb)).NodeByteSize = SIZEOF_FCB_INDEX as i16;
                InitializeListHead(core::ptr::addr_of_mut!((*fcb_idx(NewFcb)).FcbQueue));
            }
            CDFS_NTC_FCB_DATA => {
                NewFcb = CdAllocateFcbData(IrpContext);
                RtlZeroMemory(NewFcb as *mut core::ffi::c_void, SIZEOF_FCB_DATA as usize);
                (*fcb_hdr(NewFcb)).NodeByteSize = SIZEOF_FCB_DATA as i16;
            }
            _ => {
                CdBugCheck(0, 0, 0);
            }
        }

        (*fcb_hdr(NewFcb)).NodeTypeCode = NodeTypeCode as i16;
        (*NewFcb).Vcb = (*IrpContext).Vcb;
        (*NewFcb).FileId = FileId;

        CdInitializeMcb(IrpContext, NewFcb);

        (*NewFcb).FcbNonpaged = CdCreateFcbNonpaged(IrpContext);

        if (*NewFcb).FcbNonpaged.is_null() {
            CdFreePool(core::ptr::addr_of_mut!(NewFcb) as *mut *mut core::ffi::c_void);
            CdRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
        }

        *FcbExisted = 0; // FALSE

        ExInitializeFastMutex(core::ptr::addr_of_mut!(
            (*(*NewFcb).FcbNonpaged).AdvancedFcbHeaderMutex
        ));
        FsRtlSetupAdvancedHeader(
            fcb_hdr(NewFcb),
            core::ptr::addr_of_mut!((*(*NewFcb).FcbNonpaged).AdvancedFcbHeaderMutex),
        );

        if NodeTypeCode as u32 == CDFS_NTC_FCB_DATA {
            FsRtlInitializeOplock(CdGetFcbOplock(NewFcb));
        }
    } else {
        *FcbExisted = 1; // TRUE
    }

    NewFcb
}
#[no_mangle]
pub unsafe extern "C" fn CdInitializeFcbFromPathEntry(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    ParentFcb: *mut FCB,
    PathEntry: *mut PATH_ENTRY,
) {
    // PAGED_CODE();
    const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
    const FCB_STATE_IN_FCB_TABLE: u32 = 0x02;
    const SECTOR_SIZE: u32 = 2048;

    let fcb = &mut *Fcb;
    let path_entry = &*PathEntry;

    (*fcb_idx(Fcb)).StreamOffset = BytesFromBlocks(fcb.Vcb, SectorBlockOffset(fcb.Vcb, path_entry.DiskOffset));
    (*fcb_idx(Fcb)).Ordinal = path_entry.Ordinal;
    (*fcb_hdr(Fcb)).Resource = core::ptr::addr_of_mut!((*fcb.Vcb).FileResource);

    (*fcb_hdr(Fcb)).AllocationSize.QuadPart = SECTOR_SIZE as i64;
    (*fcb_hdr(Fcb)).FileSize.QuadPart = SECTOR_SIZE as i64;
    (*fcb_hdr(Fcb)).ValidDataLength.QuadPart = SECTOR_SIZE as i64;

    CdAddInitialAllocation(IrpContext, Fcb, path_entry.DiskOffset, SECTOR_SIZE as i64);

    fcb.FileAttributes |= FILE_ATTRIBUTE_DIRECTORY;

    if !ParentFcb.is_null() {
        fcb.ParentFcb = ParentFcb;
        InsertTailList(&mut (*fcb_idx(ParentFcb)).FcbQueue, &mut fcb.FcbLinks);
        CdIncrementReferenceCounts(IrpContext, ParentFcb, 1, 1);
    }

    // CdInsertFcbTable inline expansion
    {
        let mut key: FCB_TABLE_ELEMENT = core::mem::zeroed();
        key.Fcb = Fcb;
        key.FileId = fcb.FileId;
        RtlInsertElementGenericTable(
            &mut (*fcb.Vcb).FcbTable,
            &mut key as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<FCB_TABLE_ELEMENT>() as u32,
            core::ptr::null_mut(),
        );
    }

    fcb.FcbState |= FCB_STATE_IN_FCB_TABLE;
}
#[no_mangle]
pub unsafe extern "C" fn CdInitializeFcbFromFileContext(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    ParentFcb: *mut FCB,
    FileContext: *mut FILE_ENUM_CONTEXT
) {
    let ThisDirent: *mut DIRENT = core::ptr::addr_of_mut!((*(*FileContext).InitialDirent).Dirent);
    let mut CurrentCompoundDirent: *mut COMPOUND_DIRENT = core::mem::zeroed();
    let mut CurrentFileOffset: i64 = 0;
    let mut CurrentMcbEntryOffset: u32 = 0;

    CdLockFcb(IrpContext, Fcb);

    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            // try block start
            (*fcb_hdr(Fcb)).Resource = core::ptr::addr_of_mut!((*(*IrpContext).Vcb).FileResource);

            (*fcb_hdr(Fcb)).FileSize.QuadPart = (*FileContext).FileSize;
            (*fcb_hdr(Fcb)).ValidDataLength.QuadPart = (*FileContext).FileSize;

            (*fcb_hdr(Fcb)).AllocationSize.QuadPart = LlBlockAlign((*Fcb).Vcb, (*FileContext).FileSize);

            (*Fcb).FileAttributes |= FILE_ATTRIBUTE_READONLY;
            if ((*ThisDirent).DirentFlags & CD_ATTRIBUTE_HIDDEN as u8) != 0 {
                (*Fcb).FileAttributes |= FILE_ATTRIBUTE_HIDDEN;
            }

            CdConvertCdTimeToNtTime(
                IrpContext,
                (*ThisDirent).CdTime,
                core::ptr::addr_of_mut!((*Fcb).CreationTime) as *mut LARGE_INTEGER,
            );

            if (*ThisDirent).ExtentType != Form1Data {
                if (*ThisDirent).ExtentType == Mode2Form2Data {
                    (*Fcb).FcbState |= FCB_STATE_MODE2FORM2_FILE;
                } else {
                    (*Fcb).FcbState |= FCB_STATE_DA_FILE;
                }
                (*Fcb).XAAttributes = (*ThisDirent).XAAttributes;
                (*Fcb).XAFileNumber = (*ThisDirent).XAFileNumber;
            }

            CurrentCompoundDirent = (*FileContext).InitialDirent;

            loop {
                CdAddAllocationFromDirent(
                    IrpContext,
                    Fcb,
                    CurrentMcbEntryOffset,
                    CurrentFileOffset,
                    core::ptr::addr_of_mut!((*CurrentCompoundDirent).Dirent),
                );

                if ((*CurrentCompoundDirent).Dirent.DirentFlags & CD_ATTRIBUTE_MULTI as u8) == 0 {
                    break;
                }

                CurrentFileOffset += (*CurrentCompoundDirent).Dirent.DataLength as i64;
                CurrentMcbEntryOffset += 1;

                if CdLookupNextDirent(
                    IrpContext,
                    ParentFcb,
                    core::ptr::addr_of_mut!((*CurrentCompoundDirent).DirContext),
                    core::ptr::addr_of!((*(*FileContext).CurrentDirent).DirContext),
                ) == 0
                {
                    CdRaiseStatus(IrpContext, STATUS_FILE_CORRUPT_ERROR);
                }

                CurrentCompoundDirent = (*FileContext).CurrentDirent;

                CdUpdateDirentFromRawDirent(
                    IrpContext,
                    ParentFcb,
                    core::ptr::addr_of!((*CurrentCompoundDirent).DirContext),
                    core::ptr::addr_of_mut!((*CurrentCompoundDirent).Dirent),
                );
            }

            (*Fcb).FcbState |= FCB_STATE_INITIALIZED;

            (*Fcb).ParentFcb = ParentFcb;

            InsertTailList(
                core::ptr::addr_of_mut!((*fcb_idx(ParentFcb)).FcbQueue),
                core::ptr::addr_of_mut!((*Fcb).FcbLinks),
            );

            CdIncrementReferenceCounts(IrpContext, ParentFcb, 1, 1);

            {
                // CdInsertFcbTable expanded
                let mut key: FCB_TABLE_ELEMENT = core::mem::zeroed();
                key.Fcb = Fcb;
                key.FileId = (*Fcb).FileId;
                RtlInsertElementGenericTable(
                    core::ptr::addr_of_mut!((*(*Fcb).Vcb).FcbTable),
                    &mut key as *mut FCB_TABLE_ELEMENT as *mut core::ffi::c_void,
                    core::mem::size_of::<FCB_TABLE_ELEMENT>() as u32,
                    core::ptr::null_mut(),
                );
            }

            (*Fcb).FcbState |= FCB_STATE_IN_FCB_TABLE;
        },
        |_, _| {
            CdUnlockFcb(IrpContext, Fcb);
        },
    );

}

const CDFS_NTC_CCB: u32 = 0x0307; // nodetype.h CDFS_NTC_CCB
#[no_mangle]
pub unsafe extern "C" fn CdCreateCcb(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Flags: u32,
) -> *mut CCB {
    let NewCcb: *mut CCB;

    PAGED_CODE!();
    UNREFERENCED_PARAMETER!(IrpContext);

    NewCcb = CdAllocateCcb(IrpContext);
    RtlZeroMemory(NewCcb as *mut core::ffi::c_void, core::mem::size_of::<CCB>());
    (*NewCcb).NodeTypeCode = CDFS_NTC_CCB as u16;
    (*NewCcb).NodeByteSize = core::mem::size_of::<CCB>() as i16;
    (*NewCcb).Flags = Flags;
    (*NewCcb).Fcb = Fcb;

    NewCcb
}
#[no_mangle]
pub unsafe extern "C" fn CdDeleteCcb(IrpContext: *mut IRP_CONTEXT, mut Ccb: *mut CCB) {
    let _ = IrpContext;
    if !(*Ccb).SearchExpression.FileName.Buffer.is_null() {
        CdFreePool(core::ptr::addr_of_mut!((*Ccb).SearchExpression.FileName.Buffer) as *mut *mut core::ffi::c_void);
    }
    CdFreePool(core::ptr::addr_of_mut!(Ccb) as *mut *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn CdCleanupIrpContext(
    mut IrpContext: *mut IRP_CONTEXT,
    Post: BOOLEAN,
) {
    // PAGED_CODE();
    if FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_MORE_PROCESSING) == 0 {
        if !(*IrpContext).ThreadContext.is_null() {
            CdRestoreThreadContext(IrpContext);
        }
        if FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_ALLOC_IO) != 0 {
            CdFreeIoContext((*IrpContext).__bindgen_anon_1.IoContext as *mut c_void);
        }
        if FlagOn((*IrpContext).Flags, IRP_CONTEXT_FLAG_ON_STACK) == 0 {
            if CdData.IrpContextDepth < CdData.IrpContextMaxDepth {
                CdLockCdData();
                PushEntryList(
                    core::ptr::addr_of_mut!(CdData.IrpContextList) as *mut SINGLE_LIST_ENTRY,
                    IrpContext as *mut SINGLE_LIST_ENTRY,
                );
                CdData.IrpContextDepth += 1;
                CdUnlockCdData();
            } else {
                CdFreePool(&mut IrpContext as *mut *mut IRP_CONTEXT as *mut *mut core::ffi::c_void);
            }
        }
    } else if Post != 0 {
        if !(*IrpContext).ThreadContext.is_null() {
            CdRestoreThreadContext(IrpContext);
        }
        ClearFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAGS_CLEAR_ON_POST);
    } else {
        ClearFlag(&mut (*IrpContext).Flags, IRP_CONTEXT_FLAGS_CLEAR_ON_RETRY);
    }
    return;
}

const CDFS_NTC_IRP_CONTEXT: u32 = 0x0308; // nodetype.h CDFS_NTC_IRP_CONTEXT
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x00000004; // FIXED 2026-07-06: cdfs flag mismatch (verified vs cdstruc.h); WAIT=0x02 collided w/ MORE_PROCESSING and != the 0x04 CdAcquireResource checks -> WAIT flag never took -> every cdfs resource acquire ran non-blocking -> CANT_WAIT raise -> 0x1E under DUMMY_PSEH // cdstruc.h IRP_CONTEXT_FLAG_WAIT  // nolint: const-contradiction (cdfs's own flag set, cdstruc.h:1215)
#[no_mangle]
pub unsafe extern "C" fn CdInitializeStackIrpContext(IrpContext: *mut IRP_CONTEXT, IrpContextLite: *mut IRP_CONTEXT_LITE) {
    // PAGED_CODE();

    RtlZeroMemory(IrpContext as *mut core::ffi::c_void, core::mem::size_of::<IRP_CONTEXT>());

    (*IrpContext).NodeTypeCode = CDFS_NTC_IRP_CONTEXT as u16;
    (*IrpContext).NodeByteSize = core::mem::size_of::<IRP_CONTEXT>() as i16;

    SetFlag(core::ptr::addr_of_mut!((*IrpContext).Flags), IRP_CONTEXT_FLAG_ON_STACK);

    (*IrpContext).RealDevice = (*IrpContextLite).RealDevice;

    (*IrpContext).Vcb = (*(*IrpContextLite).Fcb).Vcb;

    (*IrpContext).MajorFunction = IRP_MJ_CLOSE as u8;

    SetFlag(core::ptr::addr_of_mut!((*IrpContext).Flags), IRP_CONTEXT_FLAG_WAIT);

    return;
}
#[no_mangle]
pub unsafe extern "C" fn CdTeardownStructures(
    IrpContext: *mut IRP_CONTEXT,
    StartingFcb: *mut FCB,
    RemovedStartingFcb: *mut u8,
) {
    let Vcb: *mut VCB = (*StartingFcb).Vcb;
    let mut CurrentFcb: *mut FCB = StartingFcb;
    let mut AcquiredCurrentFcb: u8 = 0;
    let mut ParentFcb: *mut FCB = core::mem::zeroed();

    PAGED_CODE!();

    *RemovedStartingFcb = 0;

    let top_level = (*IrpContext).TopLevel;
    if FlagOn((*top_level).Flags, IRP_CONTEXT_FLAG_IN_TEARDOWN) != 0 {
        return;
    }

    SetFlag(&mut (*top_level).Flags, IRP_CONTEXT_FLAG_IN_TEARDOWN);

    // _SEH2_TRY
    // try { ... } finally { ... }: a real __finally (ntkrnl_sys::seh), so a raise
    // inside the body still runs the cleanup on its way to the handler.
    let p_AcquiredCurrentFcb = core::ptr::addr_of_mut!(AcquiredCurrentFcb);
    let p_CurrentFcb = core::ptr::addr_of_mut!(CurrentFcb);
    ntkrnl_sys::seh::try_finally(
        &mut (),
        |_| {
            {
                loop {
                    if SafeNodeType(CurrentFcb) != CDFS_NTC_FCB_DATA
                        && (*CurrentFcb).FcbUserReference == 0
                        && !(*fcb_idx(CurrentFcb)).FileObject.is_null()
                    {
                        CdDeleteInternalStream(IrpContext, CurrentFcb);
                    }

                    if (*CurrentFcb).FcbReference != 0 {
                        break;
                    }

                    ParentFcb = (*CurrentFcb).ParentFcb;

                    if !ParentFcb.is_null() {
                        CdAcquireFcbExclusive(IrpContext, ParentFcb, 0);
                    }

                    CdLockVcb(IrpContext, Vcb);

                    if (*CurrentFcb).FcbReference != 0 {
                        CdUnlockVcb(IrpContext, Vcb);
                        if !ParentFcb.is_null() {
                            CdReleaseFcb(IrpContext, ParentFcb);
                        }
                        break;
                    }

                    if !ParentFcb.is_null() {
                        CdRemovePrefix(IrpContext, CurrentFcb);
                        RemoveEntryList(core::ptr::addr_of_mut!((*CurrentFcb).FcbLinks));
                        CdDecrementReferenceCounts(IrpContext, ParentFcb, 1, 1);
                    }

                    if FlagOn((*CurrentFcb).FcbState, FCB_STATE_IN_FCB_TABLE) != 0 {
                        CdDeleteFcbTable(IrpContext, CurrentFcb);
                        ClearFlag(&mut (*CurrentFcb).FcbState, FCB_STATE_IN_FCB_TABLE);
                    }

                    CdUnlockVcb(IrpContext, Vcb);
                    CdDeleteFcb(IrpContext, CurrentFcb);

                    CurrentFcb = ParentFcb;
                    AcquiredCurrentFcb = 1;

                    if CurrentFcb.is_null() {
                        break;
                    }
                }
            }
        },
        |_, _| {
            if (*p_AcquiredCurrentFcb) != 0 && !(*p_CurrentFcb).is_null() {
                CdReleaseFcb(IrpContext, *p_CurrentFcb );
            }

            ClearFlag(&mut (*(*IrpContext).TopLevel).Flags, IRP_CONTEXT_FLAG_IN_TEARDOWN);
        },
    );

    // _SEH2_END

    *RemovedStartingFcb = (CurrentFcb != StartingFcb) as u8;
}
#[no_mangle]
pub unsafe extern "C" fn CdLookupFcbTable(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    FileId: FILE_ID,
) -> *mut FCB {
    // PAGED_CODE();

    let mut Key: FCB_TABLE_ELEMENT = core::mem::zeroed();
    let Hit: *mut FCB_TABLE_ELEMENT;
    let mut ReturnFcb: *mut FCB = core::ptr::null_mut();

    Key.FileId = FileId;

    Hit = RtlLookupElementGenericTable(
        core::ptr::addr_of_mut!((*Vcb).FcbTable),
        core::ptr::addr_of_mut!(Key) as *mut core::ffi::c_void,
    ) as *mut FCB_TABLE_ELEMENT;

    if !Hit.is_null() {
        ReturnFcb = (*Hit).Fcb;
    }

    // UNREFERENCED_PARAMETER
    let _ = IrpContext;

    ReturnFcb
}
#[no_mangle]
pub unsafe extern "C" fn CdGetNextFcb(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    RestartKey: *mut *mut core::ffi::c_void,
) -> *mut FCB {
    let mut Fcb: *mut FCB;

    // PAGED_CODE();
    // UNREFERENCED_PARAMETER(IrpContext);  (omitted, Rust doesn't require unused suppression)

    Fcb = RtlEnumerateGenericTableWithoutSplaying(
        &((*Vcb).FcbTable) as *const _ as *mut RTL_GENERIC_TABLE,
        RestartKey,
    ) as *mut FCB;

    if !Fcb.is_null() {
        Fcb = (*(Fcb as *mut FCB_TABLE_ELEMENT)).Fcb;
    }

    return Fcb;
}
#[no_mangle]
pub unsafe extern "C" fn CdProcessToc(
    IrpContext: *mut IRP_CONTEXT,
    TargetDeviceObject: *mut DEVICE_OBJECT,
    CdromToc: *mut CDROM_TOC_LARGE,
    Length: *mut u32,
    TrackCount: *mut u32,
    DiskFlags: *mut u32,
) -> i32 {
    // PAGED_CODE();

    let mut status: i32;
    let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut command: CDROM_READ_TOC_EX = core::mem::zeroed();
    let mut current_track: u32;
    let local_track_count: u32;
    let local_toc_length: u32;
    let mut address: u32 = 0;
    let mut use_read_toc: u8 = 0;
    let mut biased_toc_length: BiasedTocLength = BiasedTocLength { Length: 0 };
    let mut track: *mut TRACK_DATA;

    RtlZeroMemory(
        &raw mut command as *mut core::ffi::c_void,
        core::mem::size_of::<CDROM_READ_TOC_EX>(),
    );

    'retry_read_toc: loop {
        status = CdPerformDevIoCtrlEx(
            IrpContext,
            if use_read_toc != 0 { IOCTL_CDROM_READ_TOC } else { IOCTL_CDROM_READ_TOC_EX },
            TargetDeviceObject,
            &raw mut command as *mut u8,
            core::mem::size_of::<CDROM_READ_TOC_EX>(),
            CdromToc as *mut u8,
            core::mem::size_of::<CDROM_TOC_LARGE>(),
            0,
            1,
            &raw mut iosb,
        );

        if !(status >= 0) {
            if use_read_toc == 0
                && (status == STATUS_INVALID_DEVICE_REQUEST
                    || status == STATUS_NOT_IMPLEMENTED
                    || status == STATUS_INVALID_PARAMETER)
            {
                use_read_toc = 1;
                continue 'retry_read_toc;
            }
            return status;
        }

        break 'retry_read_toc;
    }

    current_track = 0;
    local_track_count = (*CdromToc).LastTrack as u32 - (*CdromToc).FirstTrack as u32 + 1;
    {
        let base = CdromToc as *const u8;
        let field = core::ptr::addr_of!((*CdromToc).TrackData[(local_track_count + 1) as usize]) as *const u8;
        local_toc_length = (field as usize - base as usize) as u32;
    }

    if local_toc_length > iosb.Information as u32
        || (*CdromToc).FirstTrack > (*CdromToc).LastTrack
    {
        status = STATUS_DISK_CORRUPT_ERROR;
        return status;
    }

    loop {
        track = &raw mut (*CdromToc).TrackData[current_track as usize] as *mut TRACK_DATA;

        if ((*track).Control() as u32 & TOC_DATA_TRACK) != 0 {
            if ((*DiskFlags) & CDROM_DISK_AUDIO_TRACK) != 0
                && ((*DiskFlags) & CDROM_DISK_DATA_TRACK) == 0
            {
                (*CdromToc).LastTrack -= 1;

                {
                    let src: *const u8 = &raw const (*track).Address as *const u8;
                    let dst: *mut u8 = &raw mut address as *mut u8;
                    *dst.add(0) = *src.add(3);
                    *dst.add(1) = *src.add(2);
                    *dst.add(2) = *src.add(1);
                    *dst.add(3) = *src.add(0);
                }
                address -= 11250;
                {
                    let src: *const u8 = (&address) as *const u32 as *const u8;
                    let dst: *mut u8 = &raw mut (*track).Address as *mut u8;
                    *dst.add(0) = *src.add(3);
                    *dst.add(1) = *src.add(2);
                    *dst.add(2) = *src.add(1);
                    *dst.add(3) = *src.add(0);
                }

                (*track).TrackNumber = TOC_LAST_TRACK as u8;
                *DiskFlags |= CDROM_DISK_DATA_TRACK;
                break;
            }

            *DiskFlags |= CDROM_DISK_DATA_TRACK;
        } else {
            *DiskFlags |= CDROM_DISK_AUDIO_TRACK;
        }

        current_track += 1;

        if current_track >= local_track_count {
            break;
        }
    }

    *TrackCount = current_track;
    {
        let base = CdromToc as *const u8;
        let field = core::ptr::addr_of!((*CdromToc).TrackData[(current_track + 1) as usize]) as *const u8;
        *Length = (field as usize - base as usize) as u32;
    }
    biased_toc_length.Length = (*Length as u16) - 2;

    (*CdromToc).Length[0] = biased_toc_length.BigEndian[1];
    (*CdromToc).Length[1] = biased_toc_length.BigEndian[0];

    status
}
#[no_mangle]
pub unsafe extern "C" fn CdDeleteFcb(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
) {

    let mut Vcb: *mut VCB = core::ptr::null_mut();
    PAGED_CODE!();

    if (*Fcb).FcbCleanup != 0 {
        DbgPrint(
            b"CDFS: CdDeleteFcb FcbCleanup %lu != 0\n\0".as_ptr() as *const i8,
            (*Fcb).FcbCleanup as u32,
        );
    }
    if (*Fcb).FcbReference != 0 {
        DbgPrint(
            b"CDFS: CdDeleteFcb FcbReference %lu != 0\n\0".as_ptr() as *const i8,
            (*Fcb).FcbReference as u32,
        );
    }

    FsRtlTeardownPerStreamContexts(fcb_hdr(Fcb));

    CdUninitializeMcb(IrpContext, Fcb);
    CdDeleteFcbNonpaged(IrpContext, (*Fcb).FcbNonpaged);

    let buffer_ptr = (*Fcb).FileNamePrefix.ExactCaseName.FileName.Buffer;
    let file_name_buffer_ptr =
        core::ptr::addr_of!((*Fcb).FileNamePrefix.FileNameBuffer) as *mut u16;
    if buffer_ptr != file_name_buffer_ptr && !buffer_ptr.is_null() {
        CdFreePool(core::ptr::addr_of_mut!(
            (*Fcb).FileNamePrefix.ExactCaseName.FileName.Buffer
        ) as *mut *mut core::ffi::c_void);
    }

    if !(*Fcb).ShortNamePrefix.is_null() {
        CdFreePool(core::ptr::addr_of_mut!((*Fcb).ShortNamePrefix) as *mut *mut core::ffi::c_void);
    }

    match (*fcb_hdr(Fcb)).NodeTypeCode as u32 {
        CDFS_NTC_FCB_PATH_TABLE | CDFS_NTC_FCB_INDEX => {
            NT_ASSERT!(((*fcb_idx(Fcb)).FileObject.is_null()) as i32);
            NT_ASSERT!(IsListEmpty(
                core::ptr::addr_of!((*fcb_idx(Fcb)).FcbQueue) as *mut LIST_ENTRY,
            ) as i32);
            if Fcb == (*(*Fcb).Vcb).RootIndexFcb {
                Vcb = (*Fcb).Vcb;
                (*(*Fcb).Vcb).RootIndexFcb = core::ptr::null_mut();
            } else if Fcb == (*(*Fcb).Vcb).PathTableFcb {
                Vcb = (*Fcb).Vcb;
                (*(*Fcb).Vcb).PathTableFcb = core::ptr::null_mut();
            }
            CdDeallocateFcbIndex(IrpContext, Fcb as *mut core::ffi::c_void);
        }
        CDFS_NTC_FCB_DATA => {
            if !(*fcb_data(Fcb)).FileLock.is_null() {
                FsRtlFreeFileLock((*fcb_data(Fcb)).FileLock);
            }
            FsRtlUninitializeOplock(CdGetFcbOplock(Fcb));
            if Fcb == (*(*Fcb).Vcb).VolumeDasdFcb {
                Vcb = (*Fcb).Vcb;
                (*(*Fcb).Vcb).VolumeDasdFcb = core::ptr::null_mut();
            }
            CdDeallocateFcbData(IrpContext, Fcb as *mut core::ffi::c_void);
        }
        _ => {}
    }

    if !Vcb.is_null() {
        InterlockedDecrement(core::ptr::addr_of_mut!((*Vcb).VcbReference) as *mut i32);
        InterlockedDecrement(core::ptr::addr_of_mut!((*Vcb).VcbUserReference) as *mut i32);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CdCreateFcbNonpaged(IrpContext: *mut IRP_CONTEXT) -> *mut FCB_NONPAGED {
    // Unknown externs used by this function

    let FcbNonpaged: *mut FCB_NONPAGED;

    PAGED_CODE!();

    // UNREFERENCED_PARAMETER( IrpContext ) – suppress unused warning
    let _ = IrpContext;

    FcbNonpaged = CdAllocateFcbNonpaged(IrpContext);

    if !FcbNonpaged.is_null() {
        // RtlZeroMemory(FcbNonpaged, sizeof(FCB_NONPAGED)) — zero BYTES. write_bytes
        // counts ELEMENTS, so the pointer MUST be *mut u8 or it zeroes size_of² bytes
        // (a heap overflow that corrupted the adjacent VCB → BUGCHECK 0x1E).
        core::ptr::write_bytes(FcbNonpaged as *mut u8, 0u8, core::mem::size_of::<FCB_NONPAGED>());

        (*FcbNonpaged).NodeTypeCode = CDFS_NTC_FCB_NONPAGED as u16;
        (*FcbNonpaged).NodeByteSize = core::mem::size_of::<FCB_NONPAGED>() as i16;

        ExInitializeResourceLite(core::ptr::addr_of_mut!((*FcbNonpaged).FcbResource));
        ExInitializeFastMutex(core::ptr::addr_of_mut!((*FcbNonpaged).FcbMutex));
    }

    return FcbNonpaged;
}
#[no_mangle]
pub unsafe extern "C" fn CdDeleteFcbNonpaged(
    IrpContext: *mut IRP_CONTEXT,
    FcbNonpaged: *mut FCB_NONPAGED,
) {
    // PAGED_CODE();
    let _ = IrpContext; // UNREFERENCED_PARAMETER(IrpContext)

    // ExDeleteResourceLite(&FcbNonpaged->FcbResource)
    ExDeleteResourceLite(core::ptr::addr_of_mut!((*FcbNonpaged).FcbResource));

    // CdDeallocateFcbNonpaged( IrpContext, *(PVOID*)&FcbNonpaged )
    // The macro expands to: ExFreePoolWithTag(FcbNonpaged, TAG_FCB_NONPAGED)
    ExFreePoolWithTag(FcbNonpaged as *mut core::ffi::c_void, TAG_FCB_NONPAGED);
}
// NTAPI (stdcall on i386): the Rtl generic-table machinery calls these three
// callbacks as extern "system" with no caller-side stack cleanup. Declaring them
// extern "C" (cdecl) leaves the args on the stack after each call → ESP drift →
// stack corruption → eventual ret-to-NULL (BUGCHECK 0x1E at mount). i386-only;
// harmless on amd64/arm64 where system==C. The C cdfs declares these NTAPI.
#[no_mangle]
pub unsafe extern "system" fn CdFcbTableCompare(
    FcbTable: *mut ntkrnl_sys::ddk::RTL_GENERIC_TABLE,
    Fid1: *mut core::ffi::c_void,
    Fid2: *mut core::ffi::c_void,
) -> ntkrnl_sys::ddk::RTL_GENERIC_COMPARE_RESULTS {
    unsafe { /* PAGED_CODE!(); */ }

    let _ = FcbTable; // UNREFERENCED_PARAMETER(FcbTable)

    let Id1: FILE_ID = core::ptr::read_unaligned(Fid1 as *const FILE_ID);
    let Id2: FILE_ID = core::ptr::read_unaligned(Fid2 as *const FILE_ID);

    if Id1.QuadPart < Id2.QuadPart {
        return GenericLessThan;
    } else if Id1.QuadPart > Id2.QuadPart {
        return GenericGreaterThan;
    } else {
        return GenericEqual;
    }
}
#[no_mangle]
pub unsafe extern "system" fn CdAllocateFcbTable(
    FcbTable: *mut core::ffi::c_void,
    ByteSize: i32,
) -> *mut core::ffi::c_void {
    PAGED_CODE!();
    let _ = FcbTable; // UNREFERENCED_PARAMETER
    FsRtlAllocatePoolWithTag(CdPagedPool, ByteSize as u32, TAG_FCB_TABLE)
}
#[no_mangle]
pub unsafe extern "system" fn CdDeallocateFcbTable(
    FcbTable: *mut RTL_GENERIC_TABLE,
    mut Buffer: *mut core::ffi::c_void,
) {
    // PAGED_CODE(); // macro not expanded

    unsafe {
        CdFreePool(core::ptr::addr_of_mut!(Buffer) as *mut *mut core::ffi::c_void);
    }
    let _ = FcbTable;
}
#[no_mangle]
pub unsafe extern "C" fn CdTocSerial(
    IrpContext: *mut IRP_CONTEXT,
    CdromToc: *mut CDROM_TOC_LARGE,
) -> u32 {
    let mut SerialNumber: u32 = 0;
    let mut ThisTrack: *const TRACK_DATA;
    let LastTrack: *const TRACK_DATA;
    let mut Address: u32 = 0;
    let mut MsfAddress: u32 = 0;

    let _ = IrpContext;

    // Compute pointer to leadout track (LastTrack)
    let track_count = (*CdromToc).LastTrack.wrapping_sub((*CdromToc).FirstTrack) as usize + 1;
    LastTrack = ((*CdromToc).TrackData.as_ptr()).add(track_count);
    ThisTrack = (*CdromToc).TrackData.as_ptr();

    if (*CdromToc).LastTrack.wrapping_sub((*CdromToc).FirstTrack) <= 1 {
        SwapCopyUchar4(
            core::ptr::addr_of_mut!(Address),
            ((*LastTrack).Address).as_ptr(),
        );
        CdLbnToMmSsFf(Address, core::ptr::addr_of_mut!(SerialNumber) as *mut u8);
    } else {
        while ThisTrack != LastTrack {
            SwapCopyUchar4(
                core::ptr::addr_of_mut!(Address),
                ((*ThisTrack).Address).as_ptr(),
            );
            CdLbnToMmSsFf(Address, core::ptr::addr_of_mut!(MsfAddress) as *mut u8);
            SerialNumber = SerialNumber.wrapping_add(MsfAddress);
            ThisTrack = ThisTrack.add(1);
        }
    }

    SerialNumber
}

#[no_mangle]
pub unsafe extern "C" fn CdCreateFileLock(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    RaiseOnError: u8, // BOOLEAN
) -> u8 {
    let mut Result: u8 = 1; // TRUE

    PAGED_CODE!();

    // Lock the Fcb and check if there is really any work to do.
    CdLockFcb(IrpContext, Fcb);

    if !(*fcb_data(Fcb)).FileLock.is_null() {
        CdUnlockFcb(IrpContext, Fcb);
        return 1; // TRUE
    }

    let FileLock: PFILE_LOCK =
        FsRtlAllocateFileLock(core::ptr::null_mut(), core::ptr::null_mut());
    (*fcb_data(Fcb)).FileLock = FileLock;

    CdUnlockFcb(IrpContext, Fcb);

    // Return or raise as appropriate.
    if FileLock.is_null() {
        if RaiseOnError != 0 {
            NT_ASSERT!((!IrpContext.is_null()) as i32); // ARGUMENT_PRESENT( IrpContext )

            CdRaiseStatus(IrpContext, STATUS_INSUFFICIENT_RESOURCES);
        }

        Result = 0; // FALSE
    }

    Result
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (c) 1989-2000 Microsoft Corporation
//
// Module Name:
//
//     StrucSup.c
//
// Abstract:
//
//     This module implements the Cdfs in-memory data structure manipulation
//     routines
//
//
// --*/
//
// #include "cdprocs.h"
//
// //
// //  The Bug check file id for this module
// //
//
// #define BugCheckFileId                   (CDFS_BUG_CHECK_STRUCSUP)
//
// //
// //  Local macros
// //
//
// //
// //  PFCB
// //  CdAllocateFcbData (
// //       PIRP_CONTEXT IrpContext
// //      );
// //
// //  VOID
// //  CdDeallocateFcbData (
// //       PIRP_CONTEXT IrpContext,
// //       PFCB Fcb
// //      );
// //
// //  PFCB
// //  CdAllocateFcbIndex (
// //       PIRP_CONTEXT IrpContext
// //      );
// //
// //  VOID
// //  CdDeallocateFcbIndex (
// //       PIRP_CONTEXT IrpContext,
// //       PFCB Fcb
// //      );
// //
// //  PFCB_NONPAGED
// //  CdAllocateFcbNonpaged (
// //       PIRP_CONTEXT IrpContext
// //      );
// //
// //  VOID
// //  CdDeallocateFcbNonpaged (
// //       PIRP_CONTEXT IrpContext,
// //       PFCB_NONPAGED FcbNonpaged
// //      );
// //
// //  PCCB
// //  CdAllocateCcb (
// //       PIRP_CONTEXT IrpContext
// //      );
// //
// //  VOID
// //  CdDeallocateCcb (
// //       PIRP_CONTEXT IrpContext,
// //       PCCB Ccb
// //      );
// //
//
// #define CdAllocateFcbData(IC) \
//     FsRtlAllocatePoolWithTag( CdPagedPool, SIZEOF_FCB_DATA, TAG_FCB_DATA )
//
// #define CdDeallocateFcbData(IC,F) \
//     CdFreePool( &(F) )
//
// #define CdAllocateFcbIndex(IC) \
//     FsRtlAllocatePoolWithTag( CdPagedPool, SIZEOF_FCB_INDEX, TAG_FCB_INDEX )
//
// #define CdDeallocateFcbIndex(IC,F) \
//     CdFreePool( &(F) )
//
// #define CdAllocateFcbNonpaged(IC) \
//     ExAllocatePoolWithTag( CdNonPagedPool, sizeof( FCB_NONPAGED ), TAG_FCB_NONPAGED )
//
// #define CdDeallocateFcbNonpaged(IC,FNP) \
//     CdFreePool( &(FNP) )
//
// #define CdAllocateCcb(IC) \
//     FsRtlAllocatePoolWithTag( CdPagedPool, sizeof( CCB ), TAG_CCB )
//
// #define CdDeallocateCcb(IC,C) \
//     CdFreePool( &(C) )
//
// //
// //  Local structures
// //
//
// typedef struct _FCB_TABLE_ELEMENT {
//
//     FILE_ID FileId;
//     PFCB Fcb;
//
// } FCB_TABLE_ELEMENT, *PFCB_TABLE_ELEMENT;
//
// //
// //  Local macros
// //
//
// //
// //  VOID
// //  CdInsertFcbTable (
// //       PIRP_CONTEXT IrpContext,
// //       PFCB Fcb
// //      );
// //
// //  VOID
// //  CdDeleteFcbTable (
// //       PIRP_CONTEXT IrpContext,
// //       PFCB Fcb
// //      );
// //
//
//
// #define CdInsertFcbTable(IC,F) {                                    \
//     FCB_TABLE_ELEMENT _Key;                                         \
//     _Key.Fcb = (F);                                                 \
//     _Key.FileId = (F)->FileId;                                      \
//     RtlInsertElementGenericTable( &(F)->Vcb->FcbTable,              \
//                                   &_Key,                            \
//                                   sizeof( FCB_TABLE_ELEMENT ),      \
//                                   NULL );                           \
// }
//
// #define CdDeleteFcbTable(IC,F) {                                    \
//     FCB_TABLE_ELEMENT _Key;                                         \
//     _Key.FileId = (F)->FileId;                                      \
//     RtlDeleteElementGenericTable( &(F)->Vcb->FcbTable, &_Key );     \
// }
//
// //
// //  Local support routines
// //
//
// VOID
// CdDeleteFcb (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb
//     );
//
// PFCB_NONPAGED
// CdCreateFcbNonpaged (
//      PIRP_CONTEXT IrpContext
//     );
//
// VOID
// CdDeleteFcbNonpaged (
//      PIRP_CONTEXT IrpContext,
//      PFCB_NONPAGED FcbNonpaged
//     );
//
// //  Inform prefast that this is a compare routine.
// RTL_GENERIC_COMPARE_ROUTINE CdFcbTableCompare;
//
// RTL_GENERIC_COMPARE_RESULTS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdFcbTableCompare (
//      PRTL_GENERIC_TABLE FcbTable,
//      PVOID Fid1,
//      PVOID Fid2
//     );
//
// //  Inform prefast that this is an alloc reoutine.
// RTL_GENERIC_ALLOCATE_ROUTINE CdAllocateFcbTable;
//
// PVOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdAllocateFcbTable (
//      PRTL_GENERIC_TABLE FcbTable,
//      CLONG ByteSize
//     );
//
// //  Inform prefast that this is a free reoutine.
// RTL_GENERIC_FREE_ROUTINE CdDeallocateFcbTable;
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdDeallocateFcbTable (
//      PRTL_GENERIC_TABLE FcbTable,
//      __drv_freesMem(Mem)  PVOID Buffer
//     );
//
// ULONG
// CdTocSerial (
//      PIRP_CONTEXT IrpContext,
//      PCDROM_TOC_LARGE CdromToc
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, CdAllocateFcbTable)
// #pragma alloc_text(PAGE, CdCleanupIrpContext)
// #pragma alloc_text(PAGE, CdCreateCcb)
// #pragma alloc_text(PAGE, CdCreateFcb)
// #pragma alloc_text(PAGE, CdCreateFcbNonpaged)
// #pragma alloc_text(PAGE, CdCreateFileLock)
// #pragma alloc_text(PAGE, CdCreateIrpContext)
// #pragma alloc_text(PAGE, CdDeallocateFcbTable)
// #pragma alloc_text(PAGE, CdDeleteCcb)
// #pragma alloc_text(PAGE, CdDeleteFcb)
// #pragma alloc_text(PAGE, CdDeleteFcbNonpaged)
// #pragma alloc_text(PAGE, CdDeleteFileLock)
// #pragma alloc_text(PAGE, CdDeleteVcb)
// #pragma alloc_text(PAGE, CdFcbTableCompare)
// #pragma alloc_text(PAGE, CdGetNextFcb)
// #pragma alloc_text(PAGE, CdInitializeFcbFromFileContext)
// #pragma alloc_text(PAGE, CdInitializeFcbFromPathEntry)
// #pragma alloc_text(PAGE, CdInitializeStackIrpContext)
// #pragma alloc_text(PAGE, CdInitializeVcb)
// #pragma alloc_text(PAGE, CdLookupFcbTable)
// #pragma alloc_text(PAGE, CdProcessToc)
// #pragma alloc_text(PAGE, CdTeardownStructures)
// #pragma alloc_text(PAGE, CdTocSerial)
// #pragma alloc_text(PAGE, CdUpdateVcbFromVolDescriptor)
// #endif
//
// //
// //  Some static names for volume streams
// //
//
// UNICODE_STRING CdInternalStreamNames[] = {
//         { 24, 24, L"$PATH_TABLE$"},
//         { 2,  2,  L"\\"}
// };
//
//
// VOID
// CdInitializeVcb (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      __drv_aliasesMem  PDEVICE_OBJECT TargetDeviceObject,
//      __drv_aliasesMem PVPB Vpb,
//      __drv_aliasesMem PCDROM_TOC_LARGE CdromToc,
//      ULONG TocLength,
//      ULONG TocTrackCount,
//      ULONG TocDiskFlags,
//      ULONG BlockFactor,
//      ULONG MediaChangeCount
//     )
//
// /*++
//
// Routine Description:
//
//     This routine initializes and inserts a new Vcb record into the in-memory
//     data structure.  The Vcb record "hangs" off the end of the Volume device
//     object and must be allocated by our caller.
//
// Arguments:
//
//     Vcb - Supplies the address of the Vcb record being initialized.
//
//     TargetDeviceObject - Supplies the address of the target device object to
//         associate with the Vcb record.
//
//     Vpb - Supplies the address of the Vpb to associate with the Vcb record.
//
//     CdromToc - Buffer to hold table of contents.  NULL if TOC command not
//         supported.
//
//     TocLength - Byte count length of TOC.  We use this as the TOC length to
//         return on a user query.
//
//     TocTrackCount - Count of tracks in TOC.  Used to create pseudo files for
//         audio disks.
//
//     TocDiskFlags - Flag field to indicate the type of tracks on the disk.
//
//     BlockFactor - Used to decode any multi-session information.
//
//     MediaChangeCount - Initial media change count of the target device
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
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  We start by first zeroing out all of the VCB, this will guarantee
//     //  that any stale data is wiped clean.
//     //
//
//     RtlZeroMemory( Vcb, sizeof( VCB ));
//
//     //
//     //  Set the proper node type code and node byte size.
//     //
//
//     Vcb->NodeTypeCode = CDFS_NTC_VCB;
//     Vcb->NodeByteSize = sizeof( VCB );
//
//     //
//     //  Initialize the DirNotify structs.  FsRtlNotifyInitializeSync can raise.
//     //
//
//     InitializeListHead( &Vcb->DirNotifyList );
//     FsRtlNotifyInitializeSync( &Vcb->NotifySync );
//
//     //
//     //  Pick up a VPB right now so we know we can pull this filesystem stack
//     //  off of the storage stack on demand.  This can raise - if it does,
//     //  uninitialize the notify structures before returning.
//     //
//
//     _SEH2_TRY  {
//
//         Vcb->SwapVpb = FsRtlAllocatePoolWithTag( CdNonPagedPool,
//                                                  sizeof( VPB ),
//                                                  TAG_VPB );
//     }
//     _SEH2_FINALLY {
//
//         if (_SEH2_AbnormalTermination())  {
//
//             FsRtlNotifyUninitializeSync( &Vcb->NotifySync );
//         }
//     } _SEH2_END;
//
//     //
//     //  Nothing beyond this point should raise.
//     //
//
//     RtlZeroMemory( Vcb->SwapVpb, sizeof( VPB ) );
//
//     //
//     //  Initialize the resource variable for the Vcb and files.
//     //
//
//     ExInitializeResourceLite( &Vcb->VcbResource );
//     ExInitializeResourceLite( &Vcb->FileResource );
//     ExInitializeFastMutex( &Vcb->VcbMutex );
//
//     //
//     //  Insert this Vcb record on the CdData.VcbQueue.
//     //
//
//     InsertHeadList( &CdData.VcbQueue, &Vcb->VcbLinks );
//
//     //
//     //  Set the Target Device Object and Vpb fields, referencing the
//     //  Target device for the mount.
//     //
//
//     ObReferenceObject( TargetDeviceObject );
//     Vcb->TargetDeviceObject = TargetDeviceObject;
//     Vcb->Vpb = Vpb;
//
//     //
//     //  Set the removable media flag based on the real device's
//     //  characteristics
//     //
//
//     if (FlagOn( Vpb->RealDevice->Characteristics, FILE_REMOVABLE_MEDIA )) {
//
//         SetFlag( Vcb->VcbState, VCB_STATE_REMOVABLE_MEDIA );
//     }
//
//     //
//     //  Initialize the generic Fcb Table.
//     //
//
//     RtlInitializeGenericTable( &Vcb->FcbTable,
//                                (PRTL_GENERIC_COMPARE_ROUTINE) CdFcbTableCompare,
//                                (PRTL_GENERIC_ALLOCATE_ROUTINE) CdAllocateFcbTable,
//                                (PRTL_GENERIC_FREE_ROUTINE) CdDeallocateFcbTable,
//                                NULL );
//
//     //
//     //  Show that we have a mount in progress.
//     //
//
//     CdUpdateVcbCondition( Vcb, VcbMountInProgress);
//
//     //
//     //  Refererence the Vcb for two reasons.  The first is a reference
//     //  that prevents the Vcb from going away on the last close unless
//     //  dismount has already occurred.  The second is to make sure
//     //  we don't go into the dismount path on any error during mount
//     //  until we get to the Mount cleanup.
//     //
//
//     Vcb->VcbReference = 1 + CDFS_RESIDUAL_REFERENCE;
//
//     //
//     //  Update the TOC information in the Vcb.
//     //
//
//     Vcb->CdromToc = CdromToc;
//     Vcb->TocLength = TocLength;
//     Vcb->TrackCount = TocTrackCount;
//     Vcb->DiskFlags = TocDiskFlags;
//
//     //
//     //  If this disk contains audio tracks only then set the audio flag.
//     //
//
//     if (TocDiskFlags == CDROM_DISK_AUDIO_TRACK) {
//
//         SetFlag( Vcb->VcbState, VCB_STATE_AUDIO_DISK | VCB_STATE_CDXA );
//     }
//
//     //
//     //  Set the block factor.
//     //
//
//     Vcb->BlockFactor = BlockFactor;
//
//     //
//     //  Set the media change count on the device
//     //
//
//     CdUpdateMediaChangeCount( Vcb, MediaChangeCount);
// }
//
//
// VOID
// CdUpdateVcbFromVolDescriptor (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PCHAR RawIsoVd
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to perform the final initialization of a Vcb from the
//     volume descriptor on the disk.
//
// Arguments:
//
//     Vcb - Vcb for the volume being mounted.  We have already set the flags for the
//         type of descriptor.
//
//     RawIsoVd - If specified this is the volume descriptor to use to mount the
//         volume.  Not specified for a raw disk.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     ULONG StartingBlock;
//     ULONG ByteCount;
//
//     LONGLONG FileId = 0;
//
//     PRAW_DIRENT RawDirent;
//     PATH_ENTRY PathEntry;
//     PCD_MCB_ENTRY McbEntry;
//
//     BOOLEAN UnlockVcb = FALSE;
//
//     PAGED_CODE();
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Copy the block size and compute the various block masks.
//         //  Block size must not be larger than the sector size.  We will
//         //  use a default of the CD physical sector size if we are not
//         //  on a data-full disc.
//         //
//         //  This must always be set.
//         //
//
//         Vcb->BlockSize = ( ARGUMENT_PRESENT( RawIsoVd ) ?
//                             CdRvdBlkSz( RawIsoVd, Vcb->VcbState ) :
//                             SECTOR_SIZE );
//
//         //
//         //  We no longer accept media where blocksize != sector size.
//         //
//
//         if (Vcb->BlockSize != SECTOR_SIZE)  {
//
//             CdRaiseStatus( IrpContext, STATUS_DISK_CORRUPT_ERROR );
//         }
//
//         Vcb->BlocksPerSector = SECTOR_SIZE / Vcb->BlockSize;
//         Vcb->BlockMask = Vcb->BlockSize - 1;
//         Vcb->BlockInverseMask = ~Vcb->BlockMask;
//
//         Vcb->BlockToSectorShift = 0;
//         Vcb->BlockToByteShift = SECTOR_SHIFT;
//
//         //
//         //  If there is a volume descriptor then do the internal Fcb's and
//         //  other Vcb fields.
//         //
//
//         if (ARGUMENT_PRESENT( RawIsoVd )) {
//
//             //
//             //  Create the path table Fcb and refererence it and the Vcb.
//             //
//
//             CdLockVcb( IrpContext, Vcb );
//             UnlockVcb = TRUE;
//
//             Vcb->PathTableFcb = CdCreateFcb( IrpContext,
//                                              *((PFILE_ID) &FileId),
//                                              CDFS_NTC_FCB_PATH_TABLE,
//                                              NULL );
//
//             CdIncrementReferenceCounts( IrpContext, Vcb->PathTableFcb, 1, 1 );
//             CdUnlockVcb( IrpContext, Vcb );
//             UnlockVcb = FALSE;
//
//             //
//             //  Compute the stream offset and size of this path table.
//             //
//
//             StartingBlock = CdRvdPtLoc( RawIsoVd, Vcb->VcbState );
//
//             ByteCount = CdRvdPtSz( RawIsoVd, Vcb->VcbState );
//
//             Vcb->PathTableFcb->StreamOffset = BytesFromBlocks( Vcb,
//                                                                SectorBlockOffset( Vcb, StartingBlock ));
//
//             Vcb->PathTableFcb->FileSize.QuadPart = (LONGLONG) (Vcb->PathTableFcb->StreamOffset +
//                                                                ByteCount);
//
//             Vcb->PathTableFcb->ValidDataLength.QuadPart = Vcb->PathTableFcb->FileSize.QuadPart;
//
//             Vcb->PathTableFcb->AllocationSize.QuadPart = LlSectorAlign( Vcb->PathTableFcb->FileSize.QuadPart );
//
//             //
//             //  Now add the mapping information.
//             //
//
//             CdLockFcb( IrpContext, Vcb->PathTableFcb );
//
//             CdAddInitialAllocation( IrpContext,
//                                     Vcb->PathTableFcb,
//                                     StartingBlock,
//                                     Vcb->PathTableFcb->AllocationSize.QuadPart );
//
//             CdUnlockFcb( IrpContext, Vcb->PathTableFcb );
//
//             //
//             //  Point to the file resource.
//             //
//
//             Vcb->PathTableFcb->Resource = &Vcb->FileResource;
//
//             //
//             //  Mark the Fcb as initialized and create the stream file for this.
//             //
//
//             SetFlag( Vcb->PathTableFcb->FcbState, FCB_STATE_INITIALIZED );
//
//             CdCreateInternalStream( IrpContext, Vcb, Vcb->PathTableFcb, &CdInternalStreamNames[0]);
//
//             //
//             //  Create the root index and reference it in the Vcb.
//             //
//
//             CdLockVcb( IrpContext, Vcb );
//             UnlockVcb = TRUE;
//             Vcb->RootIndexFcb = CdCreateFcb( IrpContext,
//                                              *((PFILE_ID) &FileId),
//                                              CDFS_NTC_FCB_INDEX,
//                                              NULL );
//
//             CdIncrementReferenceCounts( IrpContext, Vcb->RootIndexFcb, 1, 1 );
//             CdUnlockVcb( IrpContext, Vcb );
//             UnlockVcb = FALSE;
//
//             //
//             //  Create the File id by hand for this Fcb.
//             //
//
//             CdSetFidPathTableOffset( Vcb->RootIndexFcb->FileId, Vcb->PathTableFcb->StreamOffset );
//             CdFidSetDirectory( Vcb->RootIndexFcb->FileId );
//
//             //
//             //  Create a pseudo path table entry so we can call the initialization
//             //  routine for the directory.
//             //
//
//             RawDirent = (PRAW_DIRENT) CdRvdDirent( RawIsoVd, Vcb->VcbState );
//
//             CopyUchar4( &PathEntry.DiskOffset, RawDirent->FileLoc );
//
//             PathEntry.DiskOffset += RawDirent->XarLen;
//             PathEntry.Ordinal = 1;
//             PathEntry.PathTableOffset = Vcb->PathTableFcb->StreamOffset;
//
//             CdInitializeFcbFromPathEntry( IrpContext,
//                                           Vcb->RootIndexFcb,
//                                           NULL,
//                                           &PathEntry );
//
//             //
//             //  Create the stream file for the root directory.
//             //
//
//             CdCreateInternalStream( IrpContext, Vcb, Vcb->RootIndexFcb, &CdInternalStreamNames[1] );
//
//             //
//             //  Now do the volume dasd Fcb.  Create this and reference it in the
//             //  Vcb.
//             //
//
//             CdLockVcb( IrpContext, Vcb );
//             UnlockVcb = TRUE;
//
//             Vcb->VolumeDasdFcb = CdCreateFcb( IrpContext,
//                                               *((PFILE_ID) &FileId),
//                                               CDFS_NTC_FCB_DATA,
//                                               NULL );
//
//             CdIncrementReferenceCounts( IrpContext, Vcb->VolumeDasdFcb, 1, 1 );
//             CdUnlockVcb( IrpContext, Vcb );
//             UnlockVcb = FALSE;
//
//             //
//             //  The file size is the full disk.
//             //
//
//             StartingBlock = CdRvdVolSz( RawIsoVd, Vcb->VcbState );
//
//             Vcb->VolumeDasdFcb->FileSize.QuadPart = LlBytesFromBlocks( Vcb, StartingBlock );
//
//             Vcb->VolumeDasdFcb->AllocationSize.QuadPart =
//             Vcb->VolumeDasdFcb->ValidDataLength.QuadPart = Vcb->VolumeDasdFcb->FileSize.QuadPart;
//
//             //
//             //  Now add the extent representing the volume 'by hand'.
//             //
//
//             CdLockFcb( IrpContext, Vcb->VolumeDasdFcb );
//
//             McbEntry = Vcb->VolumeDasdFcb->Mcb.McbArray;
//
//             McbEntry->FileOffset =
//             McbEntry->DiskOffset = 0;
//
//             McbEntry->ByteCount = Vcb->VolumeDasdFcb->AllocationSize.QuadPart;
//
//             McbEntry->DataBlockByteCount =
//             McbEntry->TotalBlockByteCount = McbEntry->ByteCount;
//
//             Vcb->VolumeDasdFcb->Mcb.CurrentEntryCount = 1;
//
//             CdUnlockFcb( IrpContext, Vcb->VolumeDasdFcb );
//
//             //
//             //  Point to the file resource.
//             //
//
//             Vcb->VolumeDasdFcb->Resource = &Vcb->FileResource;
//
//             Vcb->VolumeDasdFcb->FileAttributes = FILE_ATTRIBUTE_READONLY;
//
//             //
//             //  Mark the Fcb as initialized.
//             //
//
//             SetFlag( Vcb->VolumeDasdFcb->FcbState, FCB_STATE_INITIALIZED );
//
//             //
//             //  Check and see if this is an XA disk.
//             //
//
//             if (FlagOn( Vcb->VcbState, VCB_STATE_ISO | VCB_STATE_JOLIET)
//                 && RtlEqualMemory( CdXaId,
//                                    Add2Ptr( RawIsoVd, 0x400, PCHAR ),
//                                    8 )) {
//
//                 SetFlag( Vcb->VcbState, VCB_STATE_CDXA );
//             }
//
//         //
//         //  If this is a music disk then we want to mock this disk to make it
//         //  look like ISO disk.  We will create a pseudo root directory in
//         //  that case.
//         //
//
//         } else if (FlagOn( Vcb->VcbState, VCB_STATE_AUDIO_DISK )) {
//
//             ULONG RootDirectorySize;
//
//             //
//             //  Create the path table Fcb and refererence it and the Vcb.
//             //
//
//             CdLockVcb( IrpContext, Vcb );
//             UnlockVcb = TRUE;
//
//             Vcb->PathTableFcb = CdCreateFcb( IrpContext,
//                                              *((PFILE_ID) &FileId),
//                                              CDFS_NTC_FCB_PATH_TABLE,
//                                              NULL );
//
//             CdIncrementReferenceCounts( IrpContext, Vcb->PathTableFcb, 1, 1 );
//             CdUnlockVcb( IrpContext, Vcb );
//             UnlockVcb = FALSE;
//
//             //
//             //  We only create a pseudo entry for the root.
//             //
//
//             Vcb->PathTableFcb->FileSize.QuadPart = (LONGLONG) (FIELD_OFFSET( RAW_PATH_ISO, DirId ) + 2);
//
//             Vcb->PathTableFcb->ValidDataLength.QuadPart = Vcb->PathTableFcb->FileSize.QuadPart;
//
//             Vcb->PathTableFcb->AllocationSize.QuadPart = LlSectorAlign( Vcb->PathTableFcb->FileSize.QuadPart );
//
//             //
//             //  Point to the file resource.
//             //
//
//             Vcb->PathTableFcb->Resource = &Vcb->FileResource;
//
//             //
//             //  Mark the Fcb as initialized and create the stream file for this.
//             //
//
//             SetFlag( Vcb->PathTableFcb->FcbState, FCB_STATE_INITIALIZED );
//
//             CdCreateInternalStream( IrpContext, Vcb, Vcb->PathTableFcb, &CdInternalStreamNames[0]);
//
//             //
//             //  Create the root index and reference it in the Vcb.
//             //
//
//             CdLockVcb( IrpContext, Vcb );
//             UnlockVcb = TRUE;
//             Vcb->RootIndexFcb = CdCreateFcb( IrpContext,
//                                              *((PFILE_ID) &FileId),
//                                              CDFS_NTC_FCB_INDEX,
//                                              NULL );
//
//             CdIncrementReferenceCounts( IrpContext, Vcb->RootIndexFcb, 1, 1 );
//             CdUnlockVcb( IrpContext, Vcb );
//             UnlockVcb = FALSE;
//
//             //
//             //  Create the File id by hand for this Fcb.
//             //
//
//             CdSetFidPathTableOffset( Vcb->RootIndexFcb->FileId, Vcb->PathTableFcb->StreamOffset );
//             CdFidSetDirectory( Vcb->RootIndexFcb->FileId );
//
//             //
//             //  Create a pseudo path table entry so we can call the initialization
//             //  routine for the directory.
//             //
//
//             RtlZeroMemory( &PathEntry, sizeof( PATH_ENTRY ));
//
//
//             PathEntry.Ordinal = 1;
//             PathEntry.PathTableOffset = Vcb->PathTableFcb->StreamOffset;
//
//             CdInitializeFcbFromPathEntry( IrpContext,
//                                           Vcb->RootIndexFcb,
//                                           NULL,
//                                           &PathEntry );
//
//             //
//             //  Set the sizes by hand for this Fcb.  It should have an entry for each track plus an
//             //  entry for the root and parent.
//             //
//
//             RootDirectorySize = (Vcb->TrackCount + 2) * CdAudioDirentSize;
//             RootDirectorySize = SectorAlign( RootDirectorySize );
//
//             Vcb->RootIndexFcb->AllocationSize.QuadPart =
//             Vcb->RootIndexFcb->ValidDataLength.QuadPart =
//             Vcb->RootIndexFcb->FileSize.QuadPart = RootDirectorySize;
//
//             SetFlag( Vcb->RootIndexFcb->FcbState, FCB_STATE_INITIALIZED );
//
//             //
//             //  Create the stream file for the root directory.
//             //
//
//             CdCreateInternalStream( IrpContext, Vcb, Vcb->RootIndexFcb, &CdInternalStreamNames[1] );
//
//             //
//             //  Now do the volume dasd Fcb.  Create this and reference it in the
//             //  Vcb.
//             //
//
//             CdLockVcb( IrpContext, Vcb );
//             UnlockVcb = TRUE;
//
//             Vcb->VolumeDasdFcb = CdCreateFcb( IrpContext,
//                                               *((PFILE_ID) &FileId),
//                                               CDFS_NTC_FCB_DATA,
//                                               NULL );
//
//             CdIncrementReferenceCounts( IrpContext, Vcb->VolumeDasdFcb, 1, 1 );
//             CdUnlockVcb( IrpContext, Vcb );
//             UnlockVcb = FALSE;
//
//             //
//             //  We won't allow raw reads on this Fcb so leave the size at
//             //  zero.
//             //
//
//             //
//             //  Point to the file resource.
//             //
//
//             Vcb->VolumeDasdFcb->Resource = &Vcb->FileResource;
//
//             Vcb->VolumeDasdFcb->FileAttributes = FILE_ATTRIBUTE_READONLY;
//
//             //
//             //  Mark the Fcb as initialized.
//             //
//
//             SetFlag( Vcb->VolumeDasdFcb->FcbState, FCB_STATE_INITIALIZED );
//
//             //
//             //  We will store a hard-coded name in the Vpb and use the toc as
//             //  the serial number.
//             //
//
//             Vcb->Vpb->VolumeLabelLength = CdAudioLabelLength;
//
//             RtlCopyMemory( Vcb->Vpb->VolumeLabel,
//                            CdAudioLabel,
//                            CdAudioLabelLength );
//
//             //
//             //  Find the serial number for the audio disk.
//             //
//
//             Vcb->Vpb->SerialNumber = CdTocSerial( IrpContext, Vcb->CdromToc );
//
//             //
//             //  Set the ISO bit so we know how to treat the names.
//             //
//
//             SetFlag( Vcb->VcbState, VCB_STATE_ISO );
//         }
//
//     } _SEH2_FINALLY {
//
//         if (UnlockVcb) { CdUnlockVcb( IrpContext, Vcb ); }
//     } _SEH2_END;
// }
//
//
// VOID
// CdDeleteVcb (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to delete a Vcb which failed mount or has been
//     dismounted.  The dismount code should have already removed all of the
//     open Fcb's.  We do nothing here but clean up other auxilary structures.
//
// Arguments:
//
//     Vcb - Vcb to delete.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PAGED_CODE();
//
//     ASSERT_EXCLUSIVE_CDDATA;
//     ASSERT_EXCLUSIVE_VCB( Vcb );
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Chuck the backpocket Vpb we kept just in case.
//     //
//
//     CdFreePool( &Vcb->SwapVpb );
//
//     //
//     //  If there is a Vpb then we must delete it ourselves.
//     //
//
//     CdFreePool( &Vcb->Vpb );
//
//     //
//     //  Dereference our target if we haven't already done so.
//     //
//
//     if (Vcb->TargetDeviceObject != NULL) {
//
//         ObDereferenceObject( Vcb->TargetDeviceObject );
//     }
//
//     //
//     //  Delete the XA Sector and sector cache buffer if allocated.
//     //
//
//     CdFreePool( &Vcb->XASector );
//     CdFreePool( &Vcb->SectorCacheBuffer);
//
//     if (Vcb->SectorCacheIrp != NULL) {
//
//         IoFreeIrp( Vcb->SectorCacheIrp);
//         Vcb->SectorCacheIrp = NULL;
//
//         ExDeleteResourceLite( &Vcb->SectorCacheResource);
//     }
//
//     //
//     //  Remove this entry from the global queue.
//     //
//
//     RemoveEntryList( &Vcb->VcbLinks );
//
//     //
//     //  Delete the Vcb and File resources.
//     //
//
//     ExDeleteResourceLite( &Vcb->VcbResource );
//     ExDeleteResourceLite( &Vcb->FileResource );
//
//     //
//     //  Delete the TOC if present.
//     //
//
//     CdFreePool( &Vcb->CdromToc );
//
//     //
//     //  Uninitialize the notify structures.
//     //
//
//     if (Vcb->NotifySync != NULL) {
//
//         FsRtlNotifyUninitializeSync( &Vcb->NotifySync );
//     }
//
//     //
//     //  Now delete the volume device object.
//     //
// #ifdef _MSC_VER
// #pragma prefast( suppress: __WARNING_BUFFER_UNDERFLOW, "This is ok, the Vcb is embedded in our volume device object, and that is what we are really deleting." )
// #endif
//     IoDeleteDevice( (PDEVICE_OBJECT) CONTAINING_RECORD( Vcb,
//                                                         VOLUME_DEVICE_OBJECT,
//                                                         Vcb ));
//
//     return;
// }
//
//
// PFCB
// CdCreateFcb (
//      PIRP_CONTEXT IrpContext,
//      FILE_ID FileId,
//      NODE_TYPE_CODE NodeTypeCode,
//      PBOOLEAN FcbExisted
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to find the Fcb for the given FileId.  We will
//     look this up first in the Fcb table and if not found we will create
//     an Fcb.  We don't initialize it or insert it into the FcbTable in this
//     routine.
//
//     This routine is called while the Vcb is locked.
//
// Arguments:
//
//     FileId - This is the Id for the target Fcb.
//
//     NodeTypeCode - Node type for this Fcb if we need to create.
//
//     FcbExisted - If specified, we store whether the Fcb existed.
//
// Return Value:
//
//     PFCB - The Fcb found in the table or created if needed.
//
// --*/
//
// {
//     PFCB NewFcb;
//     BOOLEAN LocalFcbExisted;
//
//     PAGED_CODE();
//
//     //
//     //  Use the local boolean if one was not passed in.
//     //
//
//     if (!ARGUMENT_PRESENT( FcbExisted )) {
//
//         FcbExisted = &LocalFcbExisted;
//     }
//
//     //
//     //  Maybe this is already in the table.
//     //
//
//     NewFcb = CdLookupFcbTable( IrpContext, IrpContext->Vcb, FileId );
//
//     //
//     //  If not then create the Fcb is requested by our caller.
//     //
//
//     if (NewFcb == NULL) {
//
//         //
//         //  Allocate and initialize the structure depending on the
//         //  type code.
//         //
//
//         switch (NodeTypeCode) {
//
//         case CDFS_NTC_FCB_PATH_TABLE:
//         case CDFS_NTC_FCB_INDEX:
//
//             NewFcb = CdAllocateFcbIndex( IrpContext );
//
//             RtlZeroMemory( NewFcb, SIZEOF_FCB_INDEX );
//
//             NewFcb->NodeByteSize = SIZEOF_FCB_INDEX;
//
//             InitializeListHead( &NewFcb->FcbQueue );
//
//             break;
//
//         case CDFS_NTC_FCB_DATA :
//
//             NewFcb = CdAllocateFcbData( IrpContext );
//
//             RtlZeroMemory( NewFcb, SIZEOF_FCB_DATA );
//
//             NewFcb->NodeByteSize = SIZEOF_FCB_DATA;
//
//             break;
//
//         default:
//
// #ifdef _MSC_VER
// #pragma prefast( suppress: __WARNING_USE_OTHER_FUNCTION, "This is a bug." )
// #endif
//             CdBugCheck( 0, 0, 0 );
//         }
//
//         //
//         //  Now do the common initialization.
//         //
//
//         NewFcb->NodeTypeCode = NodeTypeCode;
//
//         NewFcb->Vcb = IrpContext->Vcb;
//         NewFcb->FileId = FileId;
//
//         CdInitializeMcb( IrpContext, NewFcb );
//
//         //
//         //  Now create the non-paged section object.
//         //
//
//         NewFcb->FcbNonpaged = CdCreateFcbNonpaged( IrpContext );
//
//         //
//         //  Deallocate the Fcb and raise if the allocation failed.
//         //
//
//         if (NewFcb->FcbNonpaged == NULL) {
//
//             CdFreePool( &NewFcb );
//
//             CdRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//         }
//
//         *FcbExisted = FALSE;
//
//         //
//         //  Initialize Advanced FCB Header fields
//         //
//
//         ExInitializeFastMutex( &NewFcb->FcbNonpaged->AdvancedFcbHeaderMutex );
//         FsRtlSetupAdvancedHeader( &NewFcb->Header,
//                                   &NewFcb->FcbNonpaged->AdvancedFcbHeaderMutex );
//
//         if (NodeTypeCode == CDFS_NTC_FCB_DATA) {
//
//             FsRtlInitializeOplock( CdGetFcbOplock(NewFcb) );
//         }
//
//     } else {
//
//         *FcbExisted = TRUE;
//     }
//
//     return NewFcb;
// }
//
//
// VOID
// CdInitializeFcbFromPathEntry (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFCB ParentFcb,
//      PPATH_ENTRY PathEntry
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to initialize an Fcb for a directory from
//     the path entry.  Since we only have a starting point for the directory,
//     not the length, we can only speculate on the sizes.
//
//     The general initialization is performed in CdCreateFcb.
//
// Arguments:
//
//     Fcb - Newly created Fcb for this stream.
//
//     ParentFcb - Parent Fcb for this stream.  It may not be present.
//
//     PathEntry - PathEntry for this Fcb in the Path Table.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PAGED_CODE();
//
//     //
//     //  Fill in the Index specific fields of the Fcb.
//     //
//
//     Fcb->StreamOffset = BytesFromBlocks( Fcb->Vcb,
//                                          SectorBlockOffset( Fcb->Vcb, PathEntry->DiskOffset ));
//
//     Fcb->Ordinal = PathEntry->Ordinal;
//
//     //
//     //  Initialize the common header in the Fcb.  The node type is already
//     //  present.
//     //
//
//     Fcb->Resource = &Fcb->Vcb->FileResource;
//
//     //
//     //  Always set the sizes to one sector until we read the self-entry.
//     //
//
//     Fcb->AllocationSize.QuadPart =
//     Fcb->FileSize.QuadPart =
//     Fcb->ValidDataLength.QuadPart = SECTOR_SIZE;
//
//     CdAddInitialAllocation( IrpContext,
//                             Fcb,
//                             PathEntry->DiskOffset,
//                             SECTOR_SIZE );
//     //
//     //  State flags for this Fcb.
//     //
//
//     SetFlag( Fcb->FileAttributes, FILE_ATTRIBUTE_DIRECTORY );
//
//     //
//     //  Link into the other in-memory structures and into the Fcb table.
//     //
//
//     if (ParentFcb != NULL) {
//
//         Fcb->ParentFcb = ParentFcb;
//
//         InsertTailList( &ParentFcb->FcbQueue, &Fcb->FcbLinks );
//
//         CdIncrementReferenceCounts( IrpContext, ParentFcb, 1, 1 );
//     }
//
//     CdInsertFcbTable( IrpContext, Fcb );
//     SetFlag( Fcb->FcbState, FCB_STATE_IN_FCB_TABLE );
//
//     return;
// }
//
//
// VOID
// CdInitializeFcbFromFileContext (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      PFCB ParentFcb,
//      PFILE_ENUM_CONTEXT FileContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to initialize an Fcb for a file from
//     the file context.  We have looked up all of the dirents for this
//     stream and have the full file size.  We will load the all of the allocation
//     for the file into the Mcb now.
//
//     The general initialization is performed in CdCreateFcb.
//
// Arguments:
//
//     Fcb - Newly created Fcb for this stream.
//
//     ParentFcb - Parent Fcb for this stream.
//
//     FileContext - FileContext for the file.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PDIRENT ThisDirent = &FileContext->InitialDirent->Dirent;
//     PCOMPOUND_DIRENT CurrentCompoundDirent;
//
//     LONGLONG CurrentFileOffset;
//     ULONG CurrentMcbEntryOffset;
//
//     PAGED_CODE();
//
//     //
//     //  Use a try-finally to facilitate cleanup.
//     //
//
//     CdLockFcb( IrpContext, Fcb );
//
//     _SEH2_TRY {
//
//         //
//         //  Initialize the common header in the Fcb.  The node type is already
//         //  present.
//         //
//
//         Fcb->Resource = &IrpContext->Vcb->FileResource;
//
//         //
//         //  Allocation occurs in block-sized units.
//         //
//
//         Fcb->FileSize.QuadPart =
//         Fcb->ValidDataLength.QuadPart = FileContext->FileSize;
//
//         Fcb->AllocationSize.QuadPart = LlBlockAlign( Fcb->Vcb, FileContext->FileSize );
//
//         //
//         //  Set the flags from the dirent.  We always start with the read-only bit.
//         //
//
//         SetFlag( Fcb->FileAttributes, FILE_ATTRIBUTE_READONLY );
//         if (FlagOn( ThisDirent->DirentFlags, CD_ATTRIBUTE_HIDDEN )) {
//
//             SetFlag( Fcb->FileAttributes, FILE_ATTRIBUTE_HIDDEN );
//         }
//
//         //
//         //  Convert the time to NT time.
//         //
//
//         CdConvertCdTimeToNtTime( IrpContext,
//                                  ThisDirent->CdTime,
//                                  (PLARGE_INTEGER) &Fcb->CreationTime );
//
//         //
//         //  Set the flag indicating the type of extent.
//         //
//
//         if (ThisDirent->ExtentType != Form1Data) {
//
//             if (ThisDirent->ExtentType == Mode2Form2Data) {
//
//                 SetFlag( Fcb->FcbState, FCB_STATE_MODE2FORM2_FILE );
//
//             } else {
//
//                 SetFlag( Fcb->FcbState, FCB_STATE_DA_FILE );
//             }
//
//             Fcb->XAAttributes = ThisDirent->XAAttributes;
//             Fcb->XAFileNumber = ThisDirent->XAFileNumber;
//         }
//
//         //
//         //  Read through all of the dirents for the file until we find the last
//         //  and add the allocation into the Mcb.
//         //
//
//         CurrentCompoundDirent = FileContext->InitialDirent;
//         CurrentFileOffset = 0;
//         CurrentMcbEntryOffset = 0;
//
//         while (TRUE) {
//
//             CdAddAllocationFromDirent( IrpContext,
//                                        Fcb,
//                                        CurrentMcbEntryOffset,
//                                        CurrentFileOffset,
//                                        &CurrentCompoundDirent->Dirent );
//
//             //
//             //  Break out if we are at the last dirent.
//             //
//
//             if (!FlagOn( CurrentCompoundDirent->Dirent.DirentFlags, CD_ATTRIBUTE_MULTI )) {
//
//                 break;
//             }
//
//             CurrentFileOffset += CurrentCompoundDirent->Dirent.DataLength;
//             CurrentMcbEntryOffset += 1;
//
//             //
//             //  We better be able to find the next dirent.
//             //
//
//             if (!CdLookupNextDirent( IrpContext,
//                                      ParentFcb,
//                                      &CurrentCompoundDirent->DirContext,
//                                      &FileContext->CurrentDirent->DirContext )) {
//
//                 CdRaiseStatus( IrpContext, STATUS_FILE_CORRUPT_ERROR );
//             }
//
//             CurrentCompoundDirent = FileContext->CurrentDirent;
//
//             CdUpdateDirentFromRawDirent( IrpContext,
//                                          ParentFcb,
//                                          &CurrentCompoundDirent->DirContext,
//                                          &CurrentCompoundDirent->Dirent );
//         }
//
//         //
//         //  Show that the Fcb is initialized.
//         //
//
//         SetFlag( Fcb->FcbState, FCB_STATE_INITIALIZED );
//
//         //
//         //  Link into the other in-memory structures and into the Fcb table.
//         //
//
//         Fcb->ParentFcb = ParentFcb;
//
//         InsertTailList( &ParentFcb->FcbQueue, &Fcb->FcbLinks );
//
//         CdIncrementReferenceCounts( IrpContext, ParentFcb, 1, 1 );
//
//         CdInsertFcbTable( IrpContext, Fcb );
//         SetFlag( Fcb->FcbState, FCB_STATE_IN_FCB_TABLE );
//
//     } _SEH2_FINALLY {
//
//         CdUnlockFcb( IrpContext, Fcb );
//     } _SEH2_END;
//
//     return;
// }
//
//
// PCCB
// CdCreateCcb (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      ULONG Flags
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to allocate and initialize the Ccb structure.
//
// Arguments:
//
//     Fcb - This is the Fcb for the file being opened.
//
//     Flags - User flags to set in this Ccb.
//
// Return Value:
//
//     PCCB - Pointer to the created Ccb.
//
// --*/
//
// {
//     PCCB NewCcb;
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Allocate and initialize the structure.
//     //
//
//     NewCcb = CdAllocateCcb( IrpContext );
//
//     RtlZeroMemory( NewCcb, sizeof( CCB ));
//
//     //
//     //  Set the proper node type code and node byte size
//     //
//
//     NewCcb->NodeTypeCode = CDFS_NTC_CCB;
//     NewCcb->NodeByteSize = sizeof( CCB );
//
//     //
//     //  Set the initial value for the flags and Fcb
//     //
//
//     NewCcb->Flags = Flags;
//     NewCcb->Fcb = Fcb;
//
//     return NewCcb;
// }
//
//
// VOID
// CdDeleteCcb (
//      PIRP_CONTEXT IrpContext,
//      __drv_freesMem( Pool ) PCCB Ccb
//     )
// /*++
//
// Routine Description:
//
//     This routine is called to cleanup and deallocate a Ccb structure.
//
// Arguments:
//
//     Ccb - This is the Ccb to delete.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     if (Ccb->SearchExpression.FileName.Buffer != NULL) {
//
//         CdFreePool( &Ccb->SearchExpression.FileName.Buffer );
//     }
//
//     CdDeallocateCcb( IrpContext, Ccb );
//     return;
// }
//
//
// (RaiseOnError || return, )
// (RaiseOnError, )
// BOOLEAN
// CdCreateFileLock (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb,
//      BOOLEAN RaiseOnError
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called when we want to attach a file lock structure to the
//     given Fcb.  It is possible the file lock is already attached.
//
//     This routine is sometimes called from the fast path and sometimes in the
//     Irp-based path.  We don't want to raise in the fast path, just return FALSE.
//
// Arguments:
//
//     Fcb - This is the Fcb to create the file lock for.
//
//     RaiseOnError - If TRUE, we will raise on an allocation failure.  Otherwise we
//         return FALSE on an allocation failure.
//
// Return Value:
//
//     BOOLEAN - TRUE if the Fcb has a filelock, FALSE otherwise.
//
// --*/
//
// {
//     BOOLEAN Result = TRUE;
//     PFILE_LOCK FileLock;
//
//     PAGED_CODE();
//
//     //
//     //  Lock the Fcb and check if there is really any work to do.
//     //
//
//     CdLockFcb( IrpContext, Fcb );
//
//     if (Fcb->FileLock != NULL) {
//
//         CdUnlockFcb( IrpContext, Fcb );
//         return TRUE;
//     }
//
//     Fcb->FileLock = FileLock =
//         FsRtlAllocateFileLock( NULL, NULL );
//
//     CdUnlockFcb( IrpContext, Fcb );
//
//     //
//     //  Return or raise as appropriate.
//     //
//
//     if (FileLock == NULL) {
//
//         if (RaiseOnError) {
//
//             NT_ASSERT( ARGUMENT_PRESENT( IrpContext ));
//
//             CdRaiseStatus( IrpContext, STATUS_INSUFFICIENT_RESOURCES );
//         }
//
//         Result = FALSE;
//     }
//
//     return Result;
// }
//
//
//  PIRP_CONTEXT
// CdCreateIrpContext (
//      PIRP Irp,
//      BOOLEAN Wait
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to initialize an IrpContext for the current
//     CDFS request.  We allocate the structure and then initialize it from
//     the given Irp.
//
// Arguments:
//
//     Irp - Irp for this request.
//
//     Wait - TRUE if this request is synchronous, FALSE otherwise.
//
// Return Value:
//
//     PIRP_CONTEXT - Allocated IrpContext.
//
// --*/
//
// {
//     PIRP_CONTEXT NewIrpContext = NULL;
//     PIO_STACK_LOCATION IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     PAGED_CODE();
//
//     //
//     //  The only operations a filesystem device object should ever receive
//     //  are create/teardown of fsdo handles and operations which do not
//     //  occur in the context of fileobjects (i.e., mount).
//     //
//
// #ifndef __REACTOS__
//     if (IrpSp->DeviceObject == CdData.FileSystemDeviceObject) {
// #else
//     if (IrpSp->DeviceObject == CdData.FileSystemDeviceObject ||
//         IrpSp->DeviceObject == CdData.HddFileSystemDeviceObject) {
// #endif
//
//         if (IrpSp->FileObject != NULL &&
//             IrpSp->MajorFunction != IRP_MJ_CREATE &&
//             IrpSp->MajorFunction != IRP_MJ_CLEANUP &&
//             IrpSp->MajorFunction != IRP_MJ_CLOSE) {
//
//             ExRaiseStatus( STATUS_INVALID_DEVICE_REQUEST );
//         }
//
//         NT_ASSERT( IrpSp->FileObject != NULL ||
//
//                 (IrpSp->MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL &&
//                  IrpSp->MinorFunction == IRP_MN_USER_FS_REQUEST &&
//                  IrpSp->Parameters.FileSystemControl.FsControlCode == FSCTL_INVALIDATE_VOLUMES) ||
//
//                 (IrpSp->MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL &&
//                  IrpSp->MinorFunction == IRP_MN_MOUNT_VOLUME ) ||
//
//                 IrpSp->MajorFunction == IRP_MJ_SHUTDOWN );
//     }
//
//     //
//     //  Look in our lookaside list for an IrpContext.
//     //
//
//     if (CdData.IrpContextDepth) {
//
//         CdLockCdData();
//         NewIrpContext = (PIRP_CONTEXT) PopEntryList( &CdData.IrpContextList );
//         if (NewIrpContext != NULL) {
//
//             CdData.IrpContextDepth--;
//         }
//
//         CdUnlockCdData();
//     }
//
//     if (NewIrpContext == NULL) {
//
//         //
//         //  We didn't get it from our private list so allocate it from pool.
//         //
//
//         NewIrpContext = FsRtlAllocatePoolWithTag( CdNonPagedPool, sizeof( IRP_CONTEXT ), TAG_IRP_CONTEXT );
//     }
//
//     RtlZeroMemory( NewIrpContext, sizeof( IRP_CONTEXT ));
//
//     //
//     //  Set the proper node type code and node byte size
//     //
//
//     NewIrpContext->NodeTypeCode = CDFS_NTC_IRP_CONTEXT;
//     NewIrpContext->NodeByteSize = sizeof( IRP_CONTEXT );
//
//     //
//     //  Set the originating Irp field
//     //
//
//     NewIrpContext->Irp = Irp;
//
//     //
//     //  Copy RealDevice for workque algorithms.  We will update this in the Mount or
//     //  Verify since they have no file objects to use here.
//     //
//
//     if (IrpSp->FileObject != NULL) {
//
//         NewIrpContext->RealDevice = IrpSp->FileObject->DeviceObject;
//     }
//
//     //
//     //  Locate the volume device object and Vcb that we are trying to access.
//     //  This may be our filesystem device object.  In that case don't initialize
//     //  the Vcb field.
//     //
//
// #ifndef __REACTOS__
//     if (IrpSp->DeviceObject != CdData.FileSystemDeviceObject) {
// #else
//     if (IrpSp->DeviceObject != CdData.FileSystemDeviceObject &&
//         IrpSp->DeviceObject != CdData.HddFileSystemDeviceObject) {
// #endif
//
//         NewIrpContext->Vcb =  &((PVOLUME_DEVICE_OBJECT) IrpSp->DeviceObject)->Vcb;
//
//     }
//
//     //
//     //  Major/Minor Function codes
//     //
//
//     NewIrpContext->MajorFunction = IrpSp->MajorFunction;
//     NewIrpContext->MinorFunction = IrpSp->MinorFunction;
//
//     //
//     //  Set the wait parameter
//     //
//
//     if (Wait) {
//
//         SetFlag( NewIrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     } else {
//
//         SetFlag( NewIrpContext->Flags, IRP_CONTEXT_FLAG_FORCE_POST );
//     }
//
//     //
//     //  return and tell the caller
//     //
//
//     return NewIrpContext;
// }
//
//
// VOID
// CdCleanupIrpContext (
//      PIRP_CONTEXT IrpContext,
//      BOOLEAN Post
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to cleanup and possibly deallocate the Irp Context.
//     If the request is being posted or this Irp Context is possibly on the
//     stack then we only cleanup any auxilary structures.
//
// Arguments:
//
//     Post - TRUE if we are posting this request, FALSE if we are deleting
//         or retrying this in the current thread.
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
//     //
//     //  If we aren't doing more processing then deallocate this as appropriate.
//     //
//
//     if (!FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_MORE_PROCESSING)) {
//
//         //
//         //  If this context is the top level CDFS context then we need to
//         //  restore the top level thread context.
//         //
//
//         if (IrpContext->ThreadContext != NULL) {
//
//             CdRestoreThreadContext( IrpContext );
//         }
//
//         //
//         //  Deallocate the Io context if allocated.
//         //
//
//         if (FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_ALLOC_IO )) {
//
//             CdFreeIoContext( IrpContext->IoContext );
//         }
//
//         //
//         //  Deallocate the IrpContext if not from the stack.
//         //
//
//         if (!FlagOn( IrpContext->Flags, IRP_CONTEXT_FLAG_ON_STACK )) {
//
//             if (CdData.IrpContextDepth < CdData.IrpContextMaxDepth) {
//
//                 CdLockCdData();
//
//                 PushEntryList( &CdData.IrpContextList, (PSINGLE_LIST_ENTRY) IrpContext );
//                 CdData.IrpContextDepth++;
//
//                 CdUnlockCdData();
//
//             } else {
//
//                 //
//                 //  We couldn't add this to our lookaside list so free it to
//                 //  pool.
//                 //
//
//                 CdFreePool( &IrpContext );
//             }
//         }
//
//     //
//     //  Clear the appropriate flags.
//     //
//
//     } else if (Post) {
//
//         //
//         //  If this context is the top level CDFS context then we need to
//         //  restore the top level thread context.
//         //
//
//         if (IrpContext->ThreadContext != NULL) {
//
//             CdRestoreThreadContext( IrpContext );
//         }
//
//         ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAGS_CLEAR_ON_POST );
//
//     } else {
//
//         ClearFlag( IrpContext->Flags, IRP_CONTEXT_FLAGS_CLEAR_ON_RETRY );
//     }
//
//     return;
// }
//
//
// VOID
// CdInitializeStackIrpContext (
//      PIRP_CONTEXT IrpContext,
//      PIRP_CONTEXT_LITE IrpContextLite
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to initialize an IrpContext for the current
//     CDFS request.  The IrpContext is on the stack and we need to initialize
//     it for the current request.  The request is a close operation.
//
// Arguments:
//
//     IrpContext - IrpContext to initialize.
//
//     IrpContextLite - Structure containing the details of this request.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PAGED_CODE();
//
//     //
//     //  Zero and then initialize the structure.
//     //
//
//     RtlZeroMemory( IrpContext, sizeof( IRP_CONTEXT ));
//
//     //
//     //  Set the proper node type code and node byte size
//     //
//
//     IrpContext->NodeTypeCode = CDFS_NTC_IRP_CONTEXT;
//     IrpContext->NodeByteSize = sizeof( IRP_CONTEXT );
//
//     //
//     //  Note that this is from the stack.
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_ON_STACK );
//
//     //
//     //  Copy RealDevice for workque algorithms.
//     //
//
//     IrpContext->RealDevice = IrpContextLite->RealDevice;
//
//     //
//     //  The Vcb is found in the Fcb.
//     //
//
//     IrpContext->Vcb = IrpContextLite->Fcb->Vcb;
//
//     //
//     //  Major/Minor Function codes
//     //
//
//     IrpContext->MajorFunction = IRP_MJ_CLOSE;
//
//     //
//     //  Set the wait parameter
//     //
//
//     SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT );
//
//     return;
// }
//
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// CdTeardownStructures (
//      PIRP_CONTEXT IrpContext,
//      PFCB StartingFcb,
//      PBOOLEAN RemovedStartingFcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is used to walk from some starting point in the Fcb tree towards
//     the root.  It will remove the Fcb and continue walking up the tree until
//     it finds a point where we can't remove an Fcb.
//
//     We look at the following fields in the Fcb to determine whether we can
//     remove this.
//
//         1 - Handle count must be zero.
//         2 - If directory then only the only reference can be for a stream file.
//         3 - Reference count must either be zero or go to zero here.
//
//     We return immediately if we are recursively entering this routine.
//
// Arguments:
//
//     StartingFcb - This is the Fcb node in the tree to begin with.  This Fcb
//         must currently be acquired exclusively.
//
//     RemovedStartingFcb - Address to store whether we removed the starting Fcb.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PVCB Vcb = StartingFcb->Vcb;
//     PFCB CurrentFcb = StartingFcb;
//     BOOLEAN AcquiredCurrentFcb = FALSE;
//     PFCB ParentFcb;
//
//     PAGED_CODE();
//
//     *RemovedStartingFcb = FALSE;
//
//     //
//     //  If this is a recursive call to TearDownStructures we return immediately
//     //  doing no operation.
//     //
//
//     if (FlagOn( IrpContext->TopLevel->Flags, IRP_CONTEXT_FLAG_IN_TEARDOWN )) {
//
//         return;
//     }
//
//     SetFlag( IrpContext->TopLevel->Flags, IRP_CONTEXT_FLAG_IN_TEARDOWN );
//
//     //
//     //  Use a try-finally to safely clear the top-level field.
//     //
//
//     _SEH2_TRY {
//
//         //
//         //  Loop until we find an Fcb we can't remove.
//         //
//
//         do {
//
//             //
//             //  See if there is an internal stream we should delete.
//             //  Only do this if it is the last reference on the Fcb.
//             //
//
//             if ((SafeNodeType( CurrentFcb ) != CDFS_NTC_FCB_DATA) &&
//                 (CurrentFcb->FcbUserReference == 0) &&
//                 (CurrentFcb->FileObject != NULL)) {
//
//                 //
//                 //  Go ahead and delete the stream file object.
//                 //
//
//                 CdDeleteInternalStream( IrpContext, CurrentFcb );
//             }
//
//             //
//             //  If the reference count is non-zero then break.
//             //
//
//             if (CurrentFcb->FcbReference != 0) {
//
//                 break;
//             }
//
//             //
//             //  It looks like we have a candidate for removal here.  We
//             //  will need to acquire the parent, if present, in order to
//             //  remove this from the parent prefix table.
//             //
//
//             ParentFcb = CurrentFcb->ParentFcb;
//
//             if (ParentFcb != NULL) {
//
//                 CdAcquireFcbExclusive( IrpContext, ParentFcb, FALSE );
//             }
//
//             //
//             //  Now lock the vcb.
//             //
//
//             CdLockVcb( IrpContext, Vcb );
//
//             //
//             //  Final check to see if the reference count is still zero.
//             //
//
//             if (CurrentFcb->FcbReference != 0) {
//
//                 CdUnlockVcb( IrpContext, Vcb );
//
//                 if (ParentFcb != NULL) {
//
//                     CdReleaseFcb( IrpContext, ParentFcb );
//                 }
//
//                 break;
//             }
//
//             //
//             //  If there is a parent then do the necessary cleanup for the parent.
//             //
//
//             if (ParentFcb != NULL) {
//
//                 CdRemovePrefix( IrpContext, CurrentFcb );
//                 RemoveEntryList( &CurrentFcb->FcbLinks );
//
//                 CdDecrementReferenceCounts( IrpContext, ParentFcb, 1, 1 );
//             }
//
//             if (FlagOn( CurrentFcb->FcbState, FCB_STATE_IN_FCB_TABLE )) {
//
//                 CdDeleteFcbTable( IrpContext, CurrentFcb );
//                 ClearFlag( CurrentFcb->FcbState, FCB_STATE_IN_FCB_TABLE );
//
//             }
//
//             //
//             //  Unlock the Vcb but hold the parent in order to walk up
//             //  the tree.
//             //
//
//             CdUnlockVcb( IrpContext, Vcb );
//             CdDeleteFcb( IrpContext, CurrentFcb );
//
//             //
//             //  Move to the parent Fcb.
//             //
//
//             CurrentFcb = ParentFcb;
//             AcquiredCurrentFcb = TRUE;
//
//         } while (CurrentFcb != NULL);
//
//     } _SEH2_FINALLY {
//
//         //
//         //  Release the current Fcb if we have acquired it.
//         //
//
//         if (AcquiredCurrentFcb && (CurrentFcb != NULL)) {
//
//             CdReleaseFcb( IrpContext, CurrentFcb );
//         }
//
//         //
//         //  Clear the teardown flag.
//         //
//
//         ClearFlag( IrpContext->TopLevel->Flags, IRP_CONTEXT_FLAG_IN_TEARDOWN );
//     } _SEH2_END;
//
//     *RemovedStartingFcb = (CurrentFcb != StartingFcb);
//     return;
// }
//
//
// PFCB
// CdLookupFcbTable (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      FILE_ID FileId
//     )
//
// /*++
//
// Routine Description:
//
//     This routine will look through the Fcb table looking for a matching
//     entry.
//
// Arguments:
//
//     Vcb - Vcb for this volume.
//
//     FileId - This is the key value to use for the search.
//
// Return Value:
//
//     PFCB - A pointer to the matching entry or NULL otherwise.
//
// --*/
//
// {
//     FCB_TABLE_ELEMENT Key;
//     PFCB_TABLE_ELEMENT Hit;
//     PFCB ReturnFcb = NULL;
//
//     PAGED_CODE();
//
//     Key.FileId = FileId;
//
//     Hit = (PFCB_TABLE_ELEMENT) RtlLookupElementGenericTable( &Vcb->FcbTable, &Key );
//
//     if (Hit != NULL) {
//
//         ReturnFcb = Hit->Fcb;
//     }
//
//     return ReturnFcb;
//
//     UNREFERENCED_PARAMETER( IrpContext );
// }
//
//
// PFCB
// CdGetNextFcb (
//      PIRP_CONTEXT IrpContext,
//      PVCB Vcb,
//      PVOID *RestartKey
//     )
//
// /*++
//
// Routine Description:
//
//     This routine will enumerate through all of the Fcb's in the Fcb table.
//
// Arguments:
//
//     Vcb - Vcb for this volume.
//
//     RestartKey - This value is used by the table package to maintain
//         its position in the enumeration.  It is initialized to NULL
//         for the first search.
//
// Return Value:
//
//     PFCB - A pointer to the next fcb or NULL if the enumeration is
//         completed
//
// --*/
//
// {
//     PFCB Fcb;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     Fcb = (PFCB) RtlEnumerateGenericTableWithoutSplaying( &Vcb->FcbTable, RestartKey );
//
//     if (Fcb != NULL) {
//
//         Fcb = ((PFCB_TABLE_ELEMENT)(Fcb))->Fcb;
//     }
//
//     return Fcb;
// }
//
//
// NTSTATUS
// CdProcessToc (
//      PIRP_CONTEXT IrpContext,
//      PDEVICE_OBJECT TargetDeviceObject,
//      PCDROM_TOC_LARGE CdromToc,
//      PULONG Length,
//      PULONG TrackCount,
//      PULONG DiskFlags
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to verify and process the TOC for this disk.
//     We hide a data track for a CD+ volume.
//
// Arguments:
//
//     TargetDeviceObject - Device object to send TOC request to.
//
//     CdromToc - Pointer to TOC structure.
//
//     Length - On input this is the length of the TOC.  On return is the TOC
//         length we will show to the user.
//
//     TrackCount - This is the count of tracks for the TOC.  We use this
//         when creating a pseudo directory for a music disk.
//
//     DiskFlags - We return flags indicating what we know about this disk.
//
// Return Value:
//
//     NTSTATUS - The result of trying to read the TOC.
//
// --*/
//
// {
//     NTSTATUS Status;
//     IO_STATUS_BLOCK Iosb;
//     CDROM_READ_TOC_EX Command;
//
//     ULONG CurrentTrack;
//     ULONG LocalTrackCount;
//     ULONG LocalTocLength;
//     ULONG Address = 0;
//     BOOLEAN UseReadToc = FALSE;
//
//     union {
//
//         UCHAR BigEndian[2];
//         USHORT Length;
//
//     } BiasedTocLength;
//
//     PTRACK_DATA Track;
//
//     PAGED_CODE();
//
//     //
//     //  Zero the command block.  This conveniently corresponds to an
//     //  LBA mode READ_TOC request.
//     //
//
//     RtlZeroMemory( &Command, sizeof( Command));
//
// RetryReadToc:
//
//     //
//     //  Go ahead and read the table of contents
//     //
//
//     Status = CdPerformDevIoCtrlEx( IrpContext,
//                                    UseReadToc ? IOCTL_CDROM_READ_TOC : IOCTL_CDROM_READ_TOC_EX,
//                                    TargetDeviceObject,
//                                    &Command,
//                                    sizeof( Command ),
//                                    CdromToc,
//                                    sizeof( CDROM_TOC_LARGE ),
//                                    FALSE,
//                                    TRUE,
//                                    &Iosb );
//
//     //
//     //  Nothing to process if this request fails.
//     //
//
//     if (!NT_SUCCESS( Status )) {
//
//         //
//         //  If the underlying device does not support READ_TOC_EX, try the old method.
//         //
//
//         if (!UseReadToc &&
//             ((Status == STATUS_INVALID_DEVICE_REQUEST) ||
//              (Status == STATUS_NOT_IMPLEMENTED) || /* ReactOS Change: we return STATUS_NOT_IMPLEMENTED for IOCTL_CDROM_READ_TOC_EX */
//              (Status == STATUS_INVALID_PARAMETER))) {
//
//             UseReadToc = TRUE;
//             goto RetryReadToc;
//         }
//
//         return Status;
//     }
//
//     //
//     //  Get the number of tracks and stated size of this structure.
//     //
//
//     CurrentTrack = 0;
//     LocalTrackCount = CdromToc->LastTrack - CdromToc->FirstTrack + 1;
//     LocalTocLength = PtrOffset( CdromToc, &CdromToc->TrackData[LocalTrackCount + 1] );
//
//     //
//     //  Get out if there is an immediate problem with the TOC.
//     //
//
//     if ((LocalTocLength > Iosb.Information) ||
//         (CdromToc->FirstTrack > CdromToc->LastTrack)) {
//
//         Status = STATUS_DISK_CORRUPT_ERROR;
//         return Status;
//     }
//
//     //
//     //  Walk through the individual tracks.  Stop at the first data track after
//     //  any lead-in audio tracks.
//     //
//
//     do {
//
//         //
//         //  Get the next track.
//         //
//
//         Track = &CdromToc->TrackData[CurrentTrack];
//
//         //
//         //  If this is a data track then check if we have only seen audio tracks
//         //  to this point.
//         //
//
//         if (FlagOn( Track->Control, TOC_DATA_TRACK )) {
//
//             //
//             //  If we have only seen audio tracks then assume this is a
//             //  CD+ disk.  Hide the current data track and only return
//             //  the previous audio tracks.  Set the disk type to be mixed
//             //  data/audio.
//             //
//
//             if (FlagOn( *DiskFlags, CDROM_DISK_AUDIO_TRACK ) &&
//                 !FlagOn( *DiskFlags, CDROM_DISK_DATA_TRACK )) {
//
//                 //
//                 //  Remove one track from the TOC.
//                 //
//
//                 CdromToc->LastTrack -= 1;
//
//                 //
//                 //  Knock 2.5 minutes off the current track to hide the final leadin.
//                 //  2.5 min = 150 sec = (x 75) 11250 frames (sectors).
//                 //
//
//                 SwapCopyUchar4( &Address, &Track->Address);
//                 Address -= 11250;
//                 SwapCopyUchar4( &Track->Address, &Address);
//
//                 Track->TrackNumber = TOC_LAST_TRACK;
//
//                 //
//                 //  Set the disk type to mixed data/audio.
//                 //
//
//                 SetFlag( *DiskFlags, CDROM_DISK_DATA_TRACK );
//
//                 break;
//             }
//
//             //
//             //  Set the flag to indicate data tracks present.
//             //
//
//             SetFlag( *DiskFlags, CDROM_DISK_DATA_TRACK );
//
//         //
//         //  If this is a audio track then set the flag indicating audio
//         //  tracks.
//         //
//
//         } else {
//
//             SetFlag( *DiskFlags, CDROM_DISK_AUDIO_TRACK );
//         }
//
//         //
//         //  Set our index for the next track.
//         //
//
//         CurrentTrack += 1;
//
//     } while (CurrentTrack < LocalTrackCount);
//
//     //
//     //  Set the length to point just past the last track we looked at.
//     //
//
//     *TrackCount = CurrentTrack;
//     *Length = PtrOffset( CdromToc, &CdromToc->TrackData[CurrentTrack + 1] );
//     BiasedTocLength.Length = (USHORT) *Length - 2;
//
//     CdromToc->Length[0] = BiasedTocLength.BigEndian[1];
//     CdromToc->Length[1] = BiasedTocLength.BigEndian[0];
//
//     return Status;
// }
//
//
// //
// //  Local support routine
// //
//
// VOID
// CdDeleteFcb (
//      PIRP_CONTEXT IrpContext,
//      PFCB Fcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to cleanup and deallocate an Fcb.  We know there
//     are no references remaining.  We cleanup any auxilary structures and
//     deallocate this Fcb.
//
// Arguments:
//
//     Fcb - This is the Fcb to deallcoate.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PVCB Vcb = NULL;
//     PAGED_CODE();
//
//     //
//     //  Sanity check the counts.
//     //
//
//     if (Fcb->FcbCleanup != 0) {
//         DbgPrint("CDFS: CdDeleteFcb FcbCleanup %lu != 0\n", Fcb->FcbCleanup);
//     }
//     if (Fcb->FcbReference != 0) {
//         DbgPrint("CDFS: CdDeleteFcb FcbReference %lu != 0\n", Fcb->FcbReference);
//     }
//
//     //
//     //  Release any Filter Context structures associated with this FCB
//     //
//
//     FsRtlTeardownPerStreamContexts( &Fcb->Header );
//
//     //
//     //  Start with the common structures.
//     //
//
//     CdUninitializeMcb( IrpContext, Fcb );
//
//     CdDeleteFcbNonpaged( IrpContext, Fcb->FcbNonpaged );
//
//     //
//     //  Check if we need to deallocate the prefix name buffer.
//     //
//
//     if ((Fcb->FileNamePrefix.ExactCaseName.FileName.Buffer != (PWCHAR) Fcb->FileNamePrefix.FileNameBuffer) &&
//         (Fcb->FileNamePrefix.ExactCaseName.FileName.Buffer != NULL)) {
//
//         CdFreePool( &Fcb->FileNamePrefix.ExactCaseName.FileName.Buffer );
//     }
//
//     //
//     //  Now look at the short name prefix.
//     //
//
//     if (Fcb->ShortNamePrefix != NULL) {
//
//         CdFreePool( &Fcb->ShortNamePrefix );
//     }
//
//     //
//     //  Now do the type specific structures.
//     //
//
//     switch (Fcb->NodeTypeCode) {
//
//     case CDFS_NTC_FCB_PATH_TABLE:
//     case CDFS_NTC_FCB_INDEX:
//
//         NT_ASSERT( Fcb->FileObject == NULL );
//         NT_ASSERT( IsListEmpty( &Fcb->FcbQueue ));
//
//         if (Fcb == Fcb->Vcb->RootIndexFcb) {
//
//             Vcb = Fcb->Vcb;
//             Vcb->RootIndexFcb = NULL;
//
//         } else if (Fcb == Fcb->Vcb->PathTableFcb) {
//
//             Vcb = Fcb->Vcb;
//             Vcb->PathTableFcb = NULL;
//         }
//
//         CdDeallocateFcbIndex( IrpContext, *(PVOID*)&Fcb );/* ReactOS Change: GCC "passing argument 1 from incompatible pointer type" */
//         break;
//
//     case CDFS_NTC_FCB_DATA :
//
//         if (Fcb->FileLock != NULL) {
//
//             FsRtlFreeFileLock( Fcb->FileLock );
//         }
//
//         FsRtlUninitializeOplock( CdGetFcbOplock(Fcb) );
//
//         if (Fcb == Fcb->Vcb->VolumeDasdFcb) {
//
//             Vcb = Fcb->Vcb;
//             Vcb->VolumeDasdFcb = NULL;
//         }
//
//         CdDeallocateFcbData( IrpContext, *(PVOID*)&Fcb );/* ReactOS Change: GCC "passing argument 1 from incompatible pointer type" */
//     }
//
//     //
//     //  Decrement the Vcb reference count if this is a system
//     //  Fcb.
//     //
//
//     if (Vcb != NULL) {
//
//         InterlockedDecrement( (LONG*)&Vcb->VcbReference );
//         InterlockedDecrement( (LONG*)&Vcb->VcbUserReference );
//     }
//
//     return;
// }
//
//
// //
// //  Local support routine
// //
//
// PFCB_NONPAGED
// CdCreateFcbNonpaged (
//      PIRP_CONTEXT IrpContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to create and initialize the non-paged portion
//     of an Fcb.
//
// Arguments:
//
// Return Value:
//
//     PFCB_NONPAGED - Pointer to the created nonpaged Fcb.  NULL if not created.
//
// --*/
//
// {
//     PFCB_NONPAGED FcbNonpaged;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Allocate the non-paged pool and initialize the various
//     //  synchronization objects.
//     //
//
//     FcbNonpaged = CdAllocateFcbNonpaged( IrpContext );
//
//     if (FcbNonpaged != NULL) {
//
//         RtlZeroMemory( FcbNonpaged, sizeof( FCB_NONPAGED ));
//
//         FcbNonpaged->NodeTypeCode = CDFS_NTC_FCB_NONPAGED;
//         FcbNonpaged->NodeByteSize = sizeof( FCB_NONPAGED );
//
//         ExInitializeResourceLite( &FcbNonpaged->FcbResource );
//         ExInitializeFastMutex( &FcbNonpaged->FcbMutex );
//     }
//
//     return FcbNonpaged;
// }
//
//
// //
// //  Local support routine
// //
//
// VOID
// CdDeleteFcbNonpaged (
//      PIRP_CONTEXT IrpContext,
//      PFCB_NONPAGED FcbNonpaged
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to cleanup the non-paged portion of an Fcb.
//
// Arguments:
//
//     FcbNonpaged - Structure to clean up.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     ExDeleteResourceLite( &FcbNonpaged->FcbResource );
//
//     CdDeallocateFcbNonpaged( IrpContext, *(PVOID*)&FcbNonpaged );/* ReactOS Change: GCC "passing argument 1 from incompatible pointer type" */
//
//     return;
// }
//
//
// //
// //  Local support routine
// //
//
// RTL_GENERIC_COMPARE_RESULTS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdFcbTableCompare (
//      PRTL_GENERIC_TABLE FcbTable,
//      PVOID Fid1,
//      PVOID Fid2
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is the Cdfs compare routine called by the generic table package.
//     If will compare the two File Id values and return a comparison result.
//
// Arguments:
//
//     FcbTable - This is the table being searched.
//
//     Fid1 - First key value.
//
//     Fid2 - Second key value.
//
// Return Value:
//
//     RTL_GENERIC_COMPARE_RESULTS - The results of comparing the two
//         input structures
//
// --*/
//
// {
//     FILE_ID Id1, Id2;
//     PAGED_CODE();
//
//     Id1 = *((FILE_ID UNALIGNED *) Fid1);
//     Id2 = *((FILE_ID UNALIGNED *) Fid2);
//
//     if (Id1.QuadPart < Id2.QuadPart) {
//
//         return GenericLessThan;
//
//     } else if (Id1.QuadPart > Id2.QuadPart) {
//
//         return GenericGreaterThan;
//
//     } else {
//
//         return GenericEqual;
//     }
//
//     UNREFERENCED_PARAMETER( FcbTable );
// }
//
//
// //
// //  Local support routine
// //
//
// PVOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdAllocateFcbTable (
//      PRTL_GENERIC_TABLE FcbTable,
//      CLONG ByteSize
//     )
//
// /*++
//
// Routine Description:
//
//     This is a generic table support routine to allocate memory
//
// Arguments:
//
//     FcbTable - Supplies the generic table being used
//
//     ByteSize - Supplies the number of bytes to allocate
//
// Return Value:
//
//     PVOID - Returns a pointer to the allocated data
//
// --*/
//
// {
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( FcbTable );
//
//     return( FsRtlAllocatePoolWithTag( CdPagedPool, ByteSize, TAG_FCB_TABLE ));
// }
//
//
// //
// //  Local support routine
// //
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdDeallocateFcbTable (
//      PRTL_GENERIC_TABLE FcbTable,
//      __drv_freesMem(Mem)  PVOID Buffer
//     )
// /*++
//
// Routine Description:
//
//     This is a generic table support routine that deallocates memory
//
// Arguments:
//
//     FcbTable - Supplies the generic table being used
//
//     Buffer - Supplies the buffer being deallocated
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
//     CdFreePool( &Buffer );
//
//     UNREFERENCED_PARAMETER( FcbTable );
// }
//
//
// //
// //  Local support routine
// //
//
// ULONG
// CdTocSerial (
//      PIRP_CONTEXT IrpContext,
//      PCDROM_TOC_LARGE CdromToc
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to generate a serial number for an audio disk.
//     The number is based on the starting positions of the tracks.
//     The following algorithm is used.
//
//     If the number of tracks is <= 2 then initialize the serial number to the
//     leadout block number.
//
//     Then add the starting address of each track (use 0x00mmssff format).
//
// Arguments:
//
//     CdromToc - Valid table of contents to use for track information.
//
// Return Value:
//
//     ULONG - 32 bit serial number based on TOC.
//
// --*/
//
// {
//     ULONG SerialNumber = 0;
//     PTRACK_DATA ThisTrack;
//     PTRACK_DATA LastTrack;
//     ULONG Address;
//     ULONG MsfAddress = 0;   // satisfy PREFIX
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Check if there are two tracks or fewer.
//     //
//
//     LastTrack = &CdromToc->TrackData[ CdromToc->LastTrack - CdromToc->FirstTrack + 1];
//     ThisTrack = &CdromToc->TrackData[0];
//
//     if (CdromToc->LastTrack - CdromToc->FirstTrack <= 1) {
//
//         SwapCopyUchar4( &Address, LastTrack->Address);
//         CdLbnToMmSsFf( Address, (PUCHAR)&SerialNumber);
//     }
//     else {
//
//         //
//         //  Add the starting offset of each track and add to the serial number.
//         //
//
//         while (ThisTrack != LastTrack) {
//
//             SwapCopyUchar4( &Address, ThisTrack->Address);
//             CdLbnToMmSsFf( Address, (PUCHAR)&MsfAddress);
//
//             SerialNumber += MsfAddress;
//             ThisTrack += 1;
//         }
//     }
//
//     return SerialNumber;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
