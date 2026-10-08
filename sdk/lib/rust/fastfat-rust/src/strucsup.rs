use ntkrnl_sys::globals::*;
// The real definitions, so the compiler checks every call against them.
use crate::timesup::{FatFatDateToNtTime, FatFatTimeToNtTime};
use ntkrnl_sys::ddk::PRTL_SPLAY_LINKS;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CC_FILE_SIZES, CcUninitializeCacheMap, ERESOURCE, ExDeleteResourceLite, ExFreePool, ExInitializeFastMutex, ExInitializeResourceLite, _IO_STATUS_BLOCK, IoAcquireVpbSpinLock, IoCreateStreamFileObject, IoDeleteDevice, IoFreeMdl, IoGetCurrentIrpStackLocation, IoGetStackLimits, IoGetTopLevelIrp, IoReleaseVpbSpinLock, KeCancelTimer, KeInitializeDpc, KeInitializeTimer, KeRemoveQueueDpc, LARGE_INTEGER, OEM_STRING, PDEVICE_OBJECT, PERESOURCE, PFILE_LOCK, PFILE_OBJECT, PIO_STACK_LOCATION, PIRP, PLARGE_MCB, POEM_STRING, POPLOCK, PSTRING, PUNICODE_STRING, PVPB, RtlDowncaseUnicodeString, RtlFreeOemString, RtlFreeUnicodeString, RtlUpcaseUnicodeString, RtlUpcaseUnicodeStringToCountedOemString, STORAGE_DEVICE_NUMBER, STORAGE_HOTPLUG_INFO, VPB};
#[cfg(target_arch = "x86_64")]
#[allow(unused_imports)]
use ntkrnl_sys::inline::ke_amd64::RtlFillMemoryUlong;
#[cfg(target_arch = "x86")]
#[allow(unused_imports)]
use ntkrnl_sys::inline::ke_i386::RtlFillMemoryUlong;
#[cfg(target_arch = "aarch64")]
#[allow(dead_code)]
unsafe fn RtlFillMemoryUlong(dest: *mut core::ffi::c_void, length: usize, pattern: u32) {
    // Fill length/4 ULONGs with `pattern` — RtlFillMemoryUlong semantics, matching
    // the x86 ke_amd64/ke_i386 inline. The old body zero-filled and dropped the
    // pattern, which on arm64 would blind the FAT_FILL_FREE poison-fill's UAF
    // detection (dormant today: the caller is gated on FAT_FILL_FREE != 0).
    let d = dest as *mut u32;
    let count = length / 4;
    let mut i = 0;
    while i < count {
        core::ptr::write(d.add(i), pattern);
        i += 1;
    }
}
use ntkrnl_sys::inline::ex::ExAllocateFromNPagedLookasideList;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut, copy_nonoverlapping, null_mut};
use core::ptr::{write_bytes};
use core::ptr;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{BIOS_PARAMETER_BLOCK, CCB, CLOSE_CONTEXT, DCB, DIRENT, FAT_BUG_CHECK_STRUCSUP, FAT_DATA, FAT_DIRENT_NT_BYTE_3_LOWER_CASE, FAT_DIRENT_NT_BYTE_8_LOWER_CASE, FCB, FCB_STATE_3_LOWER_CASE, FCB_STATE_8_LOWER_CASE, FCB_STATE_HAS_OEM_LONG_NAME, FCB_STATE_HAS_UNICODE_LONG_NAME, FCB_STATE_PAGING_FILE, FCB_STATE_SYSTEM_FILE, FILE_SYSTEM_STATISTICS, FatCloseContextSList, FatEResourceLookasideList, FatIrpContextLookasideList, FatNonPagedFcbLookasideList, IRP_CONTEXT, IRP_CONTEXT_STACK_IO_CONTEXT, NON_PAGED_FCB, PACKED_BOOT_SECTOR, PFCB, PIRP_CONTEXT, PVCB, VCB, VCB_CONDITION, VCB_STATE_FLAG_LOCKED, VOLUME_DEVICE_OBJECT};
use crate::types::{_FCB__bindgen_ty_1__bindgen_ty_1, _FCB__bindgen_ty_1__bindgen_ty_2, _CCB__bindgen_ty_1__bindgen_ty_1, FILE_NAME_NODE};
use ntkrnl_sys::ddk::{DEVICE_OBJECT, FAST_MUTEX, FILE_LOCK, FILE_OBJECT, FSRTL_ADVANCED_FCB_HEADER, IO_STACK_LOCATION, IRP, KDPC, LARGE_MCB, NPAGED_LOOKASIDE_LIST, OPLOCK, STRING, _CREATE_FILE_TYPE as FILE_TYPE, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{FSRTL_FLAG2_IS_PAGING_FILE, FSRTL_FLAG2_SUPPORTS_FILTER_CONTEXTS, TAG_CCB, TAG_FILENAME_BUFFER};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlAllocatePoolWithTag, FsRtlDeleteTunnelCache, FsRtlInitializeFileLock, FsRtlInitializeLargeMcb, FsRtlInitializeOplock, FsRtlInitializeTunnelCache, FsRtlNotifyInitializeSync, FsRtlNotifyUninitializeSync, FsRtlTeardownPerStreamContexts, FsRtlUninitializeFileLock, FsRtlUninitializeLargeMcb};
use ntkrnl_sys::ddk::{RTL_SPLAY_LINKS};

extern "system" {
    fn RtlAssert( FailedAssertion: *const u8, FileName: *const u8, LineNumber: u32, Message: *const u8, );
    fn UNREFERENCED_PARAMETER(_: *const core::ffi::c_void);
    fn NT_ASSERT(cond: i32);
    fn FatVcbAcquiredExclusive(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB) -> u8;
    fn FlagOn(Flags: u32, Mask: u32) -> i32;
    fn NodeType(Fcb: *mut FCB) -> u32;
    fn FatGetFirstChild(Fcb: *mut FCB) -> *mut FCB;
    fn FatGetNextSibling(Fcb: *mut FCB) -> *mut FCB;
    } extern "C" { fn DebugTrace(level: i32, component: u32, message: *const i8, ...); } extern "system" {
    fn DebugUnwind(name: *const i8);
    fn ClearFlag(flags: *mut u32, bits: u32);
    fn SetFlag(flags: *mut u32, bits: u32);
    fn FatGetFcbOplock(Fcb: *mut FCB) -> *mut OPLOCK;
    fn FatIsFat32(Vcb: *mut VCB) -> u8;
    fn FsRtlSetupAdvancedHeader( Header: *mut FSRTL_ADVANCED_FCB_HEADER, Mutex: *mut FAST_MUTEX, );
    static mut FatData: FAT_DATA;
    static FatJanOne1980: LARGE_INTEGER;
    fn FatFspClose(Vcb: *mut VCB);
    } extern "C" { fn KdPrintEx(Filter: u32, Level: u32, Format: *const i8, ...); } extern "system" {
}
extern "system" {
    fn ExFreeToNPagedLookasideList( lookaside: *mut NPAGED_LOOKASIDE_LIST, entry: *mut core::ffi::c_void, );
    fn FatCloseEaFile(IrpContext: *mut IRP_CONTEXT, Vcb: *mut VCB, Flag: i32);
    fn FatSetVcbCondition(Vcb: *mut VCB, Condition: u32);
    static mut FatLargeZero: LARGE_INTEGER;
    fn Fat8dot3ToString( IrpContext: *mut IRP_CONTEXT, Dirent: *mut DIRENT, flag: u8, ShortName: *mut OEM_STRING, );
    fn FatAreNamesEqual( IrpContext: *mut IRP_CONTEXT, name1: OEM_STRING, name2: OEM_STRING, ) -> u8;
    fn FatFindFcb( IrpContext: *mut IRP_CONTEXT, RootNode: *mut PRTL_SPLAY_LINKS, Name: *const OEM_STRING, FileNameDos: PBOOLEAN, ) -> *mut core::ffi::c_void;
    fn FatInsertName( IrpContext: *mut IRP_CONTEXT, RootNode: *mut PRTL_SPLAY_LINKS, Name: *mut FILE_NAME_NODE, );
    fn RtlOemStringToCountedUnicodeSize(OemString: *const OEM_STRING) -> u32;
    fn ExInterlockedPopEntrySList( ListHead: *mut core::ffi::c_void, Lock: *mut core::ffi::c_void, ) -> *mut core::ffi::c_void;
    fn ExInterlockedPushEntrySList( ListHead: *mut core::ffi::c_void, ListEntry: *mut core::ffi::c_void, Lock: *mut u32, ) -> *mut core::ffi::c_void;
    fn DbgDoit(val: i32);
    fn FatPerformDevIoCtrl( IrpContext: *mut IRP_CONTEXT, IoctlCode: u32, TargetDeviceObject: *mut DEVICE_OBJECT, InputBuffer: *mut core::ffi::c_void, InputBufferLength: u32, OutputBuffer: *mut core::ffi::c_void, OutputBufferLength: u32, InternalDeviceIoControl: i32, RemapStatus: i32, Iosb: *mut _IO_STATUS_BLOCK, ) -> i32;
    fn BooleanFlagOn(Flags: u32, Bit: u32) -> u8;
    fn DebugDump(format: *const u8, arg: i32, fcb: *mut FCB);
    fn FatBugCheck(arg1: u32, arg2: u32, arg3: u32) -> !;
    fn FatRemoveNames(IrpContext: *mut IRP_CONTEXT, Fcb: *mut FCB);
    fn DebugDoit(value: u32);
    fn FatDeviceIsFatFsdo(device_object: *mut DEVICE_OBJECT) -> u8;
    fn IsFileWriteThrough(file_object: *mut FILE_OBJECT, vcb: *mut VCB) -> u8;
    static mut FatFsdEntryCount: u32;
    fn ExLocalTimeToSystemTime(LocalTime: *const LARGE_INTEGER, SystemTime: *mut LARGE_INTEGER);
    fn FsRtlUninitializeOplock(Oplock: *mut OPLOCK);
    // See the note in fileinfo.rs: `Lbo` is LBO = LONGLONG, widened in the
    // fsctrl.rs definition; this decl was left at u32.
    fn FatAddMcbEntry( Vcb: *mut VCB, Mcb: *mut ntkrnl_sys::ddk::LARGE_MCB, Vbo: u32, Lbo: i64, SectorCount: u32, ) -> i32;
    fn FatRootDirectorySize(Bpb: *const BIOS_PARAMETER_BLOCK) -> u32;
    fn FatRootDirectoryLbo(Bpb: *const BIOS_PARAMETER_BLOCK) -> u32;
    fn FatLookupFileAllocationSize( IrpContext: *mut IRP_CONTEXT, Dcb: *mut DCB, );
    fn FatAcquireExclusiveGlobal(irp_context: *mut IRP_CONTEXT) -> u8;
    fn FatDeleteResource(resource: *mut ERESOURCE) -> NTSTATUS;
    fn FatInitializeCacheMap( file_object: *mut FILE_OBJECT, file_sizes: *const CC_FILE_SIZES, read_ahead: i32, callbacks: *const core::ffi::c_void, vcb: *mut VCB, ) -> NTSTATUS;
    fn FatReleaseGlobal(irp_context: *mut IRP_CONTEXT) -> NTSTATUS;
    fn FatSetFileObject( file_object: *mut FILE_OBJECT, file_type: FILE_TYPE, vcb: *mut VCB, fcb: *mut FCB, ) -> NTSTATUS;
    fn FatSyncUninitializeCacheMap( irp_context: *mut IRP_CONTEXT, file_object: *mut FILE_OBJECT, ) -> NTSTATUS;
    fn FatToggleMediaEjectDisable( irp_context: *mut IRP_CONTEXT, vcb: *mut VCB, eject: i32, ) -> NTSTATUS;
    fn ObReferenceObject(object: *mut core::ffi::c_void);
    fn ObDereferenceObject(object: *mut core::ffi::c_void);
    fn FatCleanVolumeDpc( dpc: *mut KDPC, context: *mut core::ffi::c_void, arg1: *mut core::ffi::c_void, arg2: *mut core::ffi::c_void, );
    fn FatTearDownAllocationSupport(IrpContext: PIRP_CONTEXT, Vcb: PVCB);
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = FAT_BUG_CHECK_STRUCSUP;
pub const Dbg: u32 = DEBUG_TRACE_STRUCSUP;
pub const IRP_CONTEXT_HEADER: usize = core::mem::size_of::<IRP_CONTEXT>().wrapping_mul(0x10000).wrapping_add((FAT_NTC_IRP_CONTEXT) as usize);
pub const FAT_FILL_FREE: u32 = 0;

const CCB_FLAG_CLOSE_CONTEXT: u32 = 0x8000;
const CCB_FLAG_FREE_UNICODE: u32 = 0x08;
const CCB_FLAG_FREE_OEM_BEST_FIT: u32 = 0x04;

const VCB_STATE_FLAG_VPB_MUST_BE_FREED: u32 = 0x00040000;
const VcbBad: u32 = crate::types::_VCB_CONDITION_VcbBad; // 3 — bindgen ground truth (was 2, off by one)
const VPB_REMOVE_PENDING: u32 = 0x08;
const IO_TYPE_VPB: u16 = 0x0A;
const IRP_MJ_FILE_SYSTEM_CONTROL: u32 = 13;
const IRP_MN_MOUNT_VOLUME: u32 = 1;

const TAG_IO_BUFFER: u32 = 0x4F746146; // nolint: const-contradiction ('OtaF' -- fastfat/nodetype.h; cdfs' is 'fbdC')
const IOCTL_CDROM_READ_TOC: u32 = 0x0024000; // placeholder
const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185_u32 as i32; // placeholder
const STATUS_INVALID_DEVICE_REQUEST: i32 = 0xC0000010_u32 as i32; // placeholder

// Forward declarations for unknown extern functions

// Constants (placeholder values – adjust to actual definitions)
const FAT_NTC_DCB: u16 = 0x0503;
const FAT_NTC_ROOT_DCB: u16 = 0x0504;
const FAT_NTC_FCB: u16 = 0x0502;
const NTC_UNDEFINED: u16 = 0x0000;
const FCB_STATE_NAMES_IN_SPLAY_TREE: u32 = 0x0100; // Example mask

const FAT_NTC_IRP_CONTEXT: u32 = 0x0508;
const IRP_NOCACHE: u32 = 0x00000001;
const SL_WRITE_THROUGH: u32 = 0x04;
const IRP_CONTEXT_FLAG_WRITE_THROUGH: u32 = 0x04;
const IRP_CONTEXT_FLAG_WAIT: u32 = 0x0002;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
const IRP_CONTEXT_FLAG_RECURSIVE_CALL: u32 = 0x10;
const IRP_MN_USER_FS_REQUEST: u8 = 0x00;
const FSCTL_INVALIDATE_VOLUMES: u32 = 0x00090054;
const DEBUG_TRACE_STRUCSUP: u32 = 0x20000000;

// Constant definitions needed (these would be provided by the assembly step)
const FcbGood: u32 = 1;          // Placeholder
const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
const TAG_FCB: u32 = 0x46746146; // 'T' 'A' 'C' 'F'? placeholder
// fatprocs.h:47: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNx: ntkrnl_sys::mm::POOL_TYPE = 0;

// Forward declarations for external functions not already available

const FAT_NTC_VCB: u16 = 0x0501; // placeholder
const IOCTL_STORAGE_GET_HOTPLUG_INFO: u32 = 0x002D0C14;
const IOCTL_STORAGE_GET_DEVICE_NUMBER: u32 = 0x2D1080;
const FILE_FLOPPY_DISKETTE: u32 = 0x04; // placeholder
const VCB_STATE_FLAG_REMOVABLE_MEDIA: u32 = 0x02;
const VCB_STATE_FLAG_HOTPLUGGABLE: u32 = 0x00200000;
const VCB_STATE_FLAG_DEFERRED_FLUSH: u32 = 0x1000;
const FILESYSTEM_STATISTICS_TYPE_FAT: u16 = 0x02;
const PACKED_BOOT_SECTOR_SIZE: usize = core::mem::size_of::<PACKED_BOOT_SECTOR>();
// fatdata.c:63 `LARGE_INTEGER FatMaxLarge = {MAXULONG,MAXLONG}` -- LowPart 0xFFFFFFFF,
// HighPart 0x7FFFFFFF. Verified, not a placeholder.
const FAT_MAX_LARGE_QUADPART: u64 = 0x7FFFFFFF_FFFFFFFF;
const VCB_GOOD: VCB_CONDITION = crate::types::_VCB_CONDITION_VcbGood; // 1 — bindgen ground truth (was 0)
const VIRTUAL_VOLUME_FILE: FILE_TYPE = 5; // fatprocs.h:53 _TYPE_OF_OPEN, which starts at 1:
                                          // 1 is UnopenedFileObject, so every later decode of
                                          // the virtual volume file said "not open"
// fatprocs.h:47: `#define NON_PAGED_POOL_NX NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NON_PAGED_POOL_NX: POOL_TYPE = 0;
                                        // NonPagedPool on ReactOS
const TAG_VCB_STATS: u32 = 0x56746146; // placeholder
const TAG_VPB: u32 = u32::from_be_bytes(*b"vtaF"); // nolint: const-contradiction (per-driver pool tag, each verified against its own C)
                                                  // nodetype.h:174 'vtaF'; 0x20425056 was
                                                  // an invented " BPV" that no free would match
const FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64 = -1;

// ==== recovered (stitcher-drop): FatFreeStringBuffer ====
#[no_mangle]
pub unsafe extern "system" fn FatFreeStringBuffer(String: *mut core::ffi::c_void) {
    let mut High: usize = 0;
    let mut Low: usize = 0;
    let LocalString: *mut STRING = String as *mut STRING;

    PAGED_CODE!();

    if !(*LocalString).Buffer.is_null() {
        IoGetStackLimits(&mut Low as *mut usize as *mut _, &mut High as *mut usize as *mut _);
        let buffer_addr: usize = (*LocalString).Buffer as usize;
        if buffer_addr < Low || buffer_addr > High {
            ExFreePool((*LocalString).Buffer as *mut core::ffi::c_void);
        }
        (*LocalString).Buffer = core::ptr::null_mut();
    }

    (*LocalString).MaximumLength = 0;
    (*LocalString).Length = 0;
}

const TAG_DYNAMIC_NAME_BUFFER: u32 = 0x63746146; // TAG('D','n','y','B')

// (inner attr removed — covered by lib.rs crate-level allow)


// Forward declarations for unknown extern functions

// Local C functions not yet translated

// Constants (guessed values)
const VcbGood: u32 = crate::types::_VCB_CONDITION_VcbGood; // 1 — bindgen ground truth (was 0)
const VPB_MOUNTED: u32 = 0x01;
const VPB_LOCKED: u32 = 0x02;
const VPB_DIRECT_WRITES_ALLOWED: u32 = 0x0020;
const VCB_STATE_FLAG_DISMOUNT_IN_PROGRESS: u32 = 0x00080000;
const DPFLTR_FASTFAT_ID: u32 = 33;
const DPFLTR_INFO_LEVEL: u32 = 3;
const MAXIMUM_VOLUME_LABEL_LENGTH: u16 = 64; // in bytes, assuming 32 WCHARs
#[no_mangle]
pub unsafe extern "system" fn FatAllocateCcb() -> *mut CCB {
    return FsRtlAllocatePoolWithTag(PagedPool, core::mem::size_of::<CCB>() as u32, TAG_CCB) as *mut CCB;
}
#[no_mangle]
pub unsafe extern "system" fn FatFreeCcb(Ccb: *mut CCB) {
    // #if FAT_FILL_FREE
    // (FAT_FILL_FREE is 0, so this block is omitted by the preprocessor)
    // #endif

    ExFreePool(Ccb as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "system" fn FatAllocateFcb() -> *mut FCB {
    FsRtlAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<FCB>() as u32,
        0x46434220u32, // TAG_FCB
    ) as *mut FCB
}
#[no_mangle]
pub unsafe extern "system" fn FatFreeFcb(Fcb: *mut FCB) {
    ExFreePool(Fcb as *mut _);
}
#[no_mangle]
pub unsafe extern "system" fn FatAllocateNonPagedFcb() -> *mut NON_PAGED_FCB {
    ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(FatNonPagedFcbLookasideList)) as *mut NON_PAGED_FCB
}

#[no_mangle]
pub unsafe extern "system" fn FatFreeNonPagedFcb(
    NonPagedFcb: *mut NON_PAGED_FCB,
) {
    ExFreeToNPagedLookasideList(
        &raw mut FatNonPagedFcbLookasideList as *mut NPAGED_LOOKASIDE_LIST,
        NonPagedFcb as *mut core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "system" fn FatAllocateResource() -> PERESOURCE {
    let Resource: PERESOURCE;
    Resource = ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(FatEResourceLookasideList)) as PERESOURCE;
    ExInitializeResourceLite(Resource);
    return Resource;
}

#[no_mangle]
pub unsafe extern "system" fn FatFreeResource(Resource: *mut ERESOURCE) {
    ExDeleteResourceLite(Resource);
    if FAT_FILL_FREE != 0 {
        RtlFillMemoryUlong(
            Resource.cast(),
            core::mem::size_of::<ERESOURCE>(),
            FAT_FILL_FREE,
        );
    }
    ExFreeToNPagedLookasideList(
        core::ptr::addr_of_mut!(FatEResourceLookasideList),
        Resource.cast(),
    );
}
#[no_mangle]
pub unsafe extern "system" fn FatAllocateIrpContext() -> PIRP_CONTEXT {
    return ExAllocateFromNPagedLookasideList(core::ptr::addr_of_mut!(FatIrpContextLookasideList))
        as PIRP_CONTEXT;
}
#[no_mangle]
pub unsafe extern "system" fn FatFreeIrpContext(IrpContext: *mut IRP_CONTEXT) {
    ExFreeToNPagedLookasideList(
        core::ptr::addr_of_mut!(FatIrpContextLookasideList),
        IrpContext as *mut core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "system" fn FatInitializeVcb(
    irp_context: *mut IRP_CONTEXT,
    vcb: *mut VCB,
    target_device_object: *mut DEVICE_OBJECT,
    vpb: *mut VPB,
    fs_device_object: *mut DEVICE_OBJECT,
) {

    let _ = fs_device_object; // UNREFERENCED_PARAMETER

    DebugTrace(1, DEBUG_TRACE_STRUCSUP, b"FatInitializeVcb, Vcb = %p\n\0".as_ptr() as *const i8, vcb);

    let mut unwind_entry_list: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut unwind_resource: *mut ERESOURCE = core::ptr::null_mut();
    let mut unwind_resource2: *mut ERESOURCE = core::ptr::null_mut();
    let mut unwind_file_object: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut unwind_cache_map: *mut FILE_OBJECT = core::ptr::null_mut();
    let mut unwind_we_allocated_mcb: bool = false;
    let mut unwind_statistics: *mut FILE_SYSTEM_STATISTICS = core::ptr::null_mut();
    let mut unwind_we_allocated_bad_block_map: bool = false;
    let mut close_context_allocated: bool = false;

    let mut file_sizes: CC_FILE_SIZES = core::mem::zeroed();
    let mut hotplug_info: STORAGE_HOTPLUG_INFO = core::mem::zeroed();
    let mut stor_device_number: STORAGE_DEVICE_NUMBER = core::mem::zeroed();
    let mut status: NTSTATUS = 0;
    let mut real_device: *mut DEVICE_OBJECT = core::ptr::null_mut();

    let p_unwind_entry_list = addr_of_mut!(unwind_entry_list);
    let p_unwind_resource = addr_of_mut!(unwind_resource);
    let p_unwind_resource2 = addr_of_mut!(unwind_resource2);
    let p_unwind_file_object = addr_of_mut!(unwind_file_object);
    let p_unwind_cache_map = addr_of_mut!(unwind_cache_map);
    let p_unwind_we_allocated_mcb = addr_of_mut!(unwind_we_allocated_mcb);
    let p_unwind_statistics = addr_of_mut!(unwind_statistics);
    let p_unwind_we_allocated_bad_block_map = addr_of_mut!(unwind_we_allocated_bad_block_map);
    let p_close_context_allocated = addr_of_mut!(close_context_allocated);

    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        // Try block
        write_bytes(vcb as *mut u8, 0, core::mem::size_of::<VCB>());

        // Set node type code and node byte size
        (*vcb).VolumeFileHeader.NodeTypeCode = (FAT_NTC_VCB) as i16;
        (*vcb).VolumeFileHeader.NodeByteSize = (core::mem::size_of::<VCB>() as u16) as i16;

        // Initialize the tunneling cache
        FsRtlInitializeTunnelCache(&raw mut (*vcb).Tunnel);

        // NT_ASSERT( FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) ) -- an assert, not an exit

        // Insert this Vcb record on the FatData.VcbQueue
        let _ = FatAcquireExclusiveGlobal(irp_context);
        InsertTailList(&raw mut FatData.VcbQueue, &raw mut (*vcb).VcbLinks);
        let _ = FatReleaseGlobal(irp_context);
        *p_unwind_entry_list = &raw mut (*vcb).VcbLinks;

        // Set the Target Device Object, Vpb, and Vcb State fields
        ObReferenceObject(target_device_object as *mut core::ffi::c_void);
        (*vcb).TargetDeviceObject = target_device_object;
        (*vcb).Vpb = vpb;
        (*vcb).CurrentDevice = (*vpb).RealDevice;

        // Set the removable media and defflush flags
        status = FatPerformDevIoCtrl(
            irp_context,
            IOCTL_STORAGE_GET_HOTPLUG_INFO,
            target_device_object,
            core::ptr::null_mut(),
            0,
            addr_of_mut!(hotplug_info) as *mut core::ffi::c_void,
            core::mem::size_of::<STORAGE_HOTPLUG_INFO>() as u32,
            0, 1, core::ptr::null_mut(),
        );

        if status >= 0 {
            if hotplug_info.MediaRemovable != 0 {
                (*vcb).VcbState |= VCB_STATE_FLAG_REMOVABLE_MEDIA;
            }

            if (hotplug_info.MediaHotplug != 0) || (hotplug_info.DeviceHotplug != 0) {
                (*vcb).VcbState |= VCB_STATE_FLAG_HOTPLUGGABLE;
            }

            if hotplug_info.WriteCacheEnableOverride == 0 {
                if (hotplug_info.MediaHotplug != 0) || (hotplug_info.DeviceHotplug != 0) {
                    // NT_ASSERT( FlagOn( Vcb->VcbState, VCB_STATE_FLAG_HOTPLUGGABLE )) -- an assert, not an exit
                    (*vcb).VcbState |= VCB_STATE_FLAG_DEFERRED_FLUSH;
                } else if hotplug_info.MediaRemovable != 0 && hotplug_info.MediaHotplug == 0 {
                    status = FatToggleMediaEjectDisable(irp_context, vcb, 1);
                    if status < 0 {
                        (*vcb).VcbState |= VCB_STATE_FLAG_DEFERRED_FLUSH;
                    }
                    let _ = FatToggleMediaEjectDisable(irp_context, vcb, 0);
                }
            }
        }

        if ((*(*vpb).RealDevice).Characteristics & FILE_REMOVABLE_MEDIA) != 0 {
            (*vcb).VcbState |= VCB_STATE_FLAG_REMOVABLE_MEDIA;
        }

        if ((*(*vpb).RealDevice).Characteristics & FILE_FLOPPY_DISKETTE) != 0 {
            (*vcb).VcbState |= VCB_STATE_FLAG_DEFERRED_FLUSH;
        }

        // Query the storage device number
        status = FatPerformDevIoCtrl(
            irp_context,
            IOCTL_STORAGE_GET_DEVICE_NUMBER,
            target_device_object,
            core::ptr::null_mut(),
            0,
            addr_of_mut!(stor_device_number) as *mut core::ffi::c_void,
            core::mem::size_of::<STORAGE_DEVICE_NUMBER>() as u32,
            0, 1, core::ptr::null_mut(),
        );

        if status >= 0 {
            (*vcb).DeviceNumber = stor_device_number.DeviceNumber;
        } else {
            (*vcb).DeviceNumber = core::u32::MAX;
        }

        // Set Vcb condition to good
        let _ = FatSetVcbCondition(vcb, VCB_GOOD);

        // Initialize the resource variables
        ExInitializeResourceLite(&raw mut (*vcb).Resource);
        *p_unwind_resource = &raw mut (*vcb).Resource;

        ExInitializeResourceLite(&raw mut (*vcb).ChangeBitMapResource);
        *p_unwind_resource2 = &raw mut (*vcb).ChangeBitMapResource;

        // Initialize the free cluster bitmap mutex
        ExInitializeFastMutex(&raw mut (*vcb).FreeClusterBitMapMutex);

        // Create the special file object for the virtual volume file
        real_device = (*vcb).CurrentDevice;
        FatPreallocateCloseContext(vcb);
        *p_close_context_allocated = true;

        *p_unwind_file_object = IoCreateStreamFileObject(core::ptr::null_mut(), real_device);
        (*vcb).VirtualVolumeFile = unwind_file_object;

        let _ = FatSetFileObject(
            (*vcb).VirtualVolumeFile,
            VIRTUAL_VOLUME_FILE,
            (vcb as *mut DCB).cast(),
            core::ptr::null_mut(),
        );

        // Increment open counts
        InterlockedIncrement(addr_of_mut!((*vcb).InternalOpenCount) as *mut i32);
        InterlockedIncrement(addr_of_mut!((*vcb).ResidualOpenCount) as *mut i32);

        let vol_file = (*vcb).VirtualVolumeFile;
        (*vol_file).SectionObjectPointer = addr_of_mut!((*vcb).SectionObjectPointers);
        (*vol_file).ReadAccess = 1;
        (*vol_file).WriteAccess = 1;
        (*vol_file).DeleteAccess = 1;

        // Initialize the notify structures
        InitializeListHead(&raw mut (*vcb).DirNotifyList);
        FsRtlNotifyInitializeSync(addr_of_mut!((*vcb).NotifySync));

        // Initialize the Cache Map for the volume file
        file_sizes.AllocationSize.QuadPart = (PACKED_BOOT_SECTOR_SIZE as u64) as i64;
        file_sizes.FileSize.QuadPart = (PACKED_BOOT_SECTOR_SIZE as u64) as i64;
        file_sizes.ValidDataLength.QuadPart = (FAT_MAX_LARGE_QUADPART) as i64;

        let _ = FatInitializeCacheMap(
            (*vcb).VirtualVolumeFile,
            addr_of!(file_sizes),
            1,
            addr_of!(FatData.CacheManagerNoOpCallbacks) as *const core::ffi::c_void,
            vcb,
        );
        *p_unwind_cache_map = (*vcb).VirtualVolumeFile;

        // Initialize the structure that will keep track of dirty fat sectors
        FsRtlInitializeLargeMcb(&raw mut (*vcb).DirtyFatMcb, PagedPool);
        *p_unwind_we_allocated_mcb = true;

        // Initialize the structure that will keep track of bad clusters
        FsRtlInitializeLargeMcb(&raw mut (*vcb).BadBlockMcb, PagedPool);
        *p_unwind_we_allocated_bad_block_map = true;

        // Set the cluster index hint to the first valid cluster of a fat: 2
        (*vcb).ClusterHint = 2;

        // Initialize the directory stream file object creation event
        ExInitializeFastMutex(&raw mut (*vcb).DirectoryFileCreationMutex);

        // Initialize the clean volume callback Timer and DPC
        KeInitializeTimer(&raw mut (*vcb).CleanVolumeTimer);
        KeInitializeDpc(
            &raw mut (*vcb).CleanVolumeDpc,
            Some(core::mem::transmute(FatCleanVolumeDpc as unsafe extern "system" fn(*mut KDPC, *mut c_void, *mut c_void, *mut c_void))),
            vcb as *mut core::ffi::c_void,
        );

        // Initialize the performance counters
        let stats = FsRtlAllocatePoolWithTag(
            NON_PAGED_POOL_NX,
            core::mem::size_of::<FILE_SYSTEM_STATISTICS>() as u32 * FatData.NumberProcessors,
            TAG_VCB_STATS,
        ) as *mut FILE_SYSTEM_STATISTICS;
        (*vcb).Statistics = stats;
        *p_unwind_statistics = stats;

        write_bytes(
            stats as *mut u8,
            0,
            core::mem::size_of::<FILE_SYSTEM_STATISTICS>() * FatData.NumberProcessors as usize,
        );

        {
            let mut i: u32 = 0;
            while i < FatData.NumberProcessors {
                (*stats.add(i as usize)).Common.FileSystemType = FILESYSTEM_STATISTICS_TYPE_FAT;
                (*stats.add(i as usize)).Common.Version = 1;
                (*stats.add(i as usize)).Common.SizeOfCompleteStructure =
                    (core::mem::size_of::<FILE_SYSTEM_STATISTICS>() as u16) as u32;
                i = i + 1;
            }
        }

        // Pick up a VPB right now
        let swap_vpb = FsRtlAllocatePoolWithTag(
            NON_PAGED_POOL_NX,
            core::mem::size_of::<VPB>() as u32,
            TAG_VPB,
        ) as *mut VPB;
        (*vcb).SwapVpb = swap_vpb;
        write_bytes(swap_vpb as *mut u8, 0, core::mem::size_of::<VPB>());

        // Initialize the close queue listheads
        InitializeListHead(&raw mut (*vcb).AsyncCloseList);
        InitializeListHead(&raw mut (*vcb).DelayedCloseList);

        // Initialize the Advanced FCB Header
        ExInitializeFastMutex(&raw mut (*vcb).AdvancedFcbHeaderMutex);
        FsRtlSetupAdvancedHeader(
            &raw mut (*vcb).VolumeFileHeader,
            &raw mut (*vcb).AdvancedFcbHeaderMutex,
        );

        // With the Vcb now set up, set the IrpContext Vcb field
        (*irp_context).Vcb = vcb;

    }, |_, abnormal| {
        // Finally block
        DebugUnwind(core::ptr::null()); // DebugUnwind( FatInitializeVcb ) approximated

        if abnormal {
            if !(*p_unwind_cache_map).is_null() {
                let _ = FatSyncUninitializeCacheMap(irp_context, (*p_unwind_cache_map));
            }
            if !(*p_unwind_file_object).is_null() {
                ObDereferenceObject((*p_unwind_file_object) as *mut core::ffi::c_void);
            }
            if !(*p_unwind_resource).is_null() {
                let _ = FatDeleteResource(*p_unwind_resource);
            }
            if !(*p_unwind_resource2).is_null() {
                let _ = FatDeleteResource(*p_unwind_resource2);
            }
            if *p_unwind_we_allocated_mcb {
                FsRtlUninitializeLargeMcb(&raw mut (*vcb).DirtyFatMcb);
            }
            if *p_unwind_we_allocated_bad_block_map {
                FsRtlUninitializeLargeMcb(&raw mut (*vcb).BadBlockMcb);
            }
            if !(*p_unwind_entry_list).is_null() {
                let _ = FatAcquireExclusiveGlobal(irp_context);
                RemoveEntryList(*p_unwind_entry_list);
                let _ = FatReleaseGlobal(irp_context);
            }
            if !(*p_unwind_statistics).is_null() {
                ExFreePool((*p_unwind_statistics) as *mut core::ffi::c_void);
            }

            // Cleanup the close context we preallocated above
            if *p_close_context_allocated && (*vcb).VirtualVolumeFile.is_null() {
                let close_context: *mut CLOSE_CONTEXT = (FatAllocateCloseContext(vcb)).cast();
                ExFreePool(close_context as *mut core::ffi::c_void);
                *p_close_context_allocated = false;
            }
        }

        DebugTrace(-1, DEBUG_TRACE_STRUCSUP, b"FatInitializeVcb -> VOID\n\0".as_ptr() as *const i8, core::ptr::null::<core::ffi::c_void>());
    });
}

#[no_mangle]
pub unsafe extern "system" fn FatTearDownVcb(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
) {
    let mut DirectoryFileObject: *mut FILE_OBJECT;

    if (*Vcb).VirtualVolumeFile != core::ptr::null_mut() {
        CcUninitializeCacheMap(
            (*Vcb).VirtualVolumeFile,
            core::ptr::addr_of_mut!(FatLargeZero) as *mut LARGE_INTEGER,
            core::ptr::null_mut(),
        );
        FsRtlTeardownPerStreamContexts(
            core::ptr::addr_of_mut!((*Vcb).VolumeFileHeader) as *mut FSRTL_ADVANCED_FCB_HEADER,
        );
        ObDereferenceObject((*Vcb).VirtualVolumeFile as *mut core::ffi::c_void);
        (*Vcb).VirtualVolumeFile = core::ptr::null_mut();
    }

    FatCloseEaFile(IrpContext, Vcb, 0i32);

    if (*Vcb).RootDcb != core::ptr::null_mut() {
        let root_dcb_inner = (*(*Vcb).RootDcb).Specific.Dcb.as_mut() as *mut _FCB__bindgen_ty_1__bindgen_ty_1;
        DirectoryFileObject = (*root_dcb_inner).DirectoryFile;

        if DirectoryFileObject != core::ptr::null_mut() {
            CcUninitializeCacheMap(
                DirectoryFileObject,
                core::ptr::addr_of_mut!(FatLargeZero) as *mut LARGE_INTEGER,
                core::ptr::null_mut(),
            );
            (*root_dcb_inner).DirectoryFile = core::ptr::null_mut();
            ObDereferenceObject(DirectoryFileObject as *mut core::ffi::c_void);
        }
    }

    FatSetVcbCondition(Vcb, VcbBad);
}

#[no_mangle]
pub unsafe extern "system" fn FatDeleteVcb(
    IrpContext: PIRP_CONTEXT,
    Vcb: PVCB,
) {
    let mut Fcb: PFCB;

    PAGED_CODE!();

    DebugTrace(1i32, Dbg, b"FatDeleteVcb, Vcb = %p\n\0".as_ptr() as *const i8, Vcb);

    if (*IrpContext).Vcb == Vcb {
        (*IrpContext).Vcb = core::ptr::null_mut();
    }

    if !(*Vcb).SwapVpb.is_null() {
        ExFreePool((*Vcb).SwapVpb as *mut core::ffi::c_void);
    }

    if FlagOn((*Vcb).VcbState, 0x40000 /* VCB_STATE_FLAG_VPB_MUST_BE_FREED */) != 0 {
        ExFreePool((*Vcb).Vpb as *mut core::ffi::c_void);
    }

    if !(*Vcb).VolumeGuidPath.Buffer.is_null() {
        ExFreePool((*Vcb).VolumeGuidPath.Buffer as *mut core::ffi::c_void);
        (*Vcb).VolumeGuidPath.Buffer = core::ptr::null_mut();
    }

    RemoveEntryList(core::ptr::addr_of_mut!((*Vcb).VcbLinks) as PLIST_ENTRY);

    if (*Vcb).DirectAccessOpenCount != 0 || (*Vcb).OpenFileCount != 0 {
        FatBugCheck(0, 0, 0);
    }

    if !(*Vcb).EaFcb.is_null() {
        (*(*Vcb).EaFcb).OpenCount = 0;
        FatDeleteFcb(IrpContext, core::ptr::addr_of_mut!((*Vcb).EaFcb));
    }

    if !(*Vcb).RootDcb.is_null() {
        loop {
            Fcb = FatGetNextFcbBottomUp(IrpContext, core::ptr::null_mut(), (*Vcb).RootDcb);
            if Fcb == (*Vcb).RootDcb {
                break;
            }
            FatDeleteFcb(IrpContext, core::ptr::addr_of_mut!(Fcb));
        }
        FatDeleteFcb(IrpContext, core::ptr::addr_of_mut!((*Vcb).RootDcb));
    }

    FsRtlNotifyUninitializeSync(core::ptr::addr_of_mut!((*Vcb).NotifySync));

    FatDeleteResource(core::ptr::addr_of_mut!((*Vcb).Resource));
    FatDeleteResource(core::ptr::addr_of_mut!((*Vcb).ChangeBitMapResource));

    if !(*Vcb).FreeClusterBitMap.__bindgen_anon_1.__bindgen_anon_1.as_ref().Buffer.is_null() {
        FatTearDownAllocationSupport(IrpContext, Vcb);
    }

    FsRtlUninitializeLargeMcb(core::ptr::addr_of_mut!((*Vcb).DirtyFatMcb));
    FsRtlUninitializeLargeMcb(core::ptr::addr_of_mut!((*Vcb).BadBlockMcb));

    if !(*Vcb).First0x24BytesOfBootSector.is_null() {
        ExFreePool((*Vcb).First0x24BytesOfBootSector as *mut core::ffi::c_void);
        (*Vcb).First0x24BytesOfBootSector = core::ptr::null_mut();
    }

    KeCancelTimer(core::ptr::addr_of_mut!((*Vcb).CleanVolumeTimer));
    KeRemoveQueueDpc(core::ptr::addr_of_mut!((*Vcb).CleanVolumeDpc));

    ExFreePool((*Vcb).Statistics as *mut core::ffi::c_void);

    FsRtlDeleteTunnelCache(core::ptr::addr_of_mut!((*Vcb).Tunnel));

    ObDereferenceObject((*Vcb).TargetDeviceObject as *mut core::ffi::c_void);

    NT_ASSERT(if (*Vcb).CloseContextCount <= 1 { 1 } else { 0 });

    core::ptr::write_bytes(Vcb as *mut u8, 0, core::mem::size_of::<VCB>());

    DebugTrace(-1i32, Dbg, b"FatDeleteVcb -> VOID\n\0".as_ptr() as *const i8, 0i32);
}
#[no_mangle]
pub unsafe extern "system" fn FatCreateRootDcb(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
) {
    let mut Dcb: *mut DCB = core::ptr::null_mut();
    let mut UnwindStorage: [*mut core::ffi::c_void; 2] = [core::ptr::null_mut(); 2];
    let mut UnwindResource: *mut ntkrnl_sys::ddk::ERESOURCE = core::ptr::null_mut();
    let mut UnwindResource2: *mut ntkrnl_sys::ddk::ERESOURCE = core::ptr::null_mut();
    let mut UnwindMcb: *mut ntkrnl_sys::ddk::LARGE_MCB = core::ptr::null_mut();
    let mut UnwindFileObject: *mut FILE_OBJECT = core::ptr::null_mut();

    // PAGED_CODE(); -- ignore annotation

    DebugTrace(1, 0, b"FatCreateRootDcb, Vcb = %p\n\0".as_ptr() as *const i8, Vcb);

    let p_Dcb = core::ptr::addr_of_mut!(Dcb);
    let p_UnwindStorage = core::ptr::addr_of_mut!(UnwindStorage);
    let p_UnwindResource = core::ptr::addr_of_mut!(UnwindResource);
    let p_UnwindResource2 = core::ptr::addr_of_mut!(UnwindResource2);
    let p_UnwindMcb = core::ptr::addr_of_mut!(UnwindMcb);
    let p_UnwindFileObject = core::ptr::addr_of_mut!(UnwindFileObject);
    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        // --- _SEH2_TRY block ---

        if !(*Vcb).RootDcb.is_null() {
            DebugDump(b"Error trying to create multiple root dcbs\n\0".as_ptr(), 0, Vcb as *mut FCB);
            FatBugCheck(0, 0, 0);
        }

        // Allocate and zero DCB
        let alloc = FsRtlAllocatePoolWithTag(
            NonPagedPoolNx,
            core::mem::size_of::<DCB>() as u32,
            TAG_FCB,
        );
        *p_Dcb = alloc as *mut DCB;
        (*Vcb).RootDcb = (*p_Dcb);
        (*p_UnwindStorage)[0] = alloc;

        RtlZeroMemory((*p_Dcb) as *mut core::ffi::c_void, core::mem::size_of::<DCB>());

        // Allocate NonPagedFcb
        let non_paged = FatAllocateNonPagedFcb();
        (**p_Dcb).NonPaged = non_paged;
        (*p_UnwindStorage)[1] = non_paged as *mut core::ffi::c_void;

        RtlZeroMemory(
            non_paged as *mut core::ffi::c_void,
            core::mem::size_of::<NON_PAGED_FCB>(),
        );

        // Set header fields
        (**p_Dcb).Header.NodeTypeCode = (FAT_NTC_ROOT_DCB) as i16;
        (**p_Dcb).Header.NodeByteSize = (core::mem::size_of::<DCB>() as u16) as i16;
        (**p_Dcb).FcbCondition = FcbGood;

        // Initialize resources
        let resource = FatAllocateResource();
        (**p_Dcb).Header.Resource = resource;
        *p_UnwindResource = resource;

        let paging_io_resource = FatAllocateResource();
        (**p_Dcb).Header.PagingIoResource = paging_io_resource;
        *p_UnwindResource2 = paging_io_resource;

        // Initialize parent (*p_Dcb) links
        InitializeListHead(core::ptr::addr_of_mut!((**p_Dcb).ParentDcbLinks));

        (**p_Dcb).Vcb = Vcb;

        {
            let dcb_inner = (**p_Dcb).Specific.Dcb.as_mut() as *mut _FCB__bindgen_ty_1__bindgen_ty_1;
            InitializeListHead(core::ptr::addr_of_mut!((*dcb_inner).ParentDcbQueue));
        }

        // Set full file name (wide "\\")
        static WIDE_BACKSLASH: [u16; 2] = [0x5C, 0x0000];
        (**p_Dcb).FullFileName.Buffer = WIDE_BACKSLASH.as_ptr() as *mut u16;
        (**p_Dcb).FullFileName.Length = 2u16;
        (**p_Dcb).FullFileName.MaximumLength = 4u16;

        // Set short OEM name (ASCII "\\")
        static OEM_BACKSLASH: [u8; 2] = [0x5C, 0];
        (**p_Dcb).ShortName.Name.Oem.as_mut().Buffer = OEM_BACKSLASH.as_ptr() as *const i8 as *mut i8;
        (**p_Dcb).ShortName.Name.Oem.as_mut().Length = 1u16;
        (**p_Dcb).ShortName.Name.Oem.as_mut().MaximumLength = 2u16;

        (**p_Dcb).DirentFatFlags = (FILE_ATTRIBUTE_DIRECTORY) as u8;

        // Initialize advanced FCB header
        ExInitializeFastMutex(
            core::ptr::addr_of_mut!((*(**p_Dcb).NonPaged).AdvancedFcbHeaderMutex),
        );
        FsRtlSetupAdvancedHeader(
            core::ptr::addr_of_mut!((**p_Dcb).Header),
            core::ptr::addr_of_mut!((*(**p_Dcb).NonPaged).AdvancedFcbHeaderMutex),
        );

        // Initialize MCB
        FsRtlInitializeLargeMcb(core::ptr::addr_of_mut!((**p_Dcb).Mcb), NonPagedPoolNx);
        *p_UnwindMcb = core::ptr::addr_of_mut!((**p_Dcb).Mcb);

        if FatIsFat32(Vcb) != 0 {
            (**p_Dcb).FirstClusterOfFile = (*Vcb).Bpb.RootDirFirstCluster;
        } else {
            FatAddMcbEntry(
                Vcb,
                core::ptr::addr_of_mut!((**p_Dcb).Mcb),
                0,
                FatRootDirectoryLbo(core::ptr::addr_of!((*Vcb).Bpb)) as i64,
                FatRootDirectorySize(core::ptr::addr_of!((*Vcb).Bpb)),
            );
        }

        if FatIsFat32(Vcb) != 0 {
            (**p_Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart = 0xFFFFFFFFu32;
            FatLookupFileAllocationSize(IrpContext, (*p_Dcb));
            (**p_Dcb).Header.FileSize.QuadPart = (**p_Dcb).Header.AllocationSize.QuadPart;
        } else {
            let dir_size = FatRootDirectorySize(core::ptr::addr_of!((*Vcb).Bpb)) as u64;
            (**p_Dcb).Header.AllocationSize.QuadPart = (dir_size) as i64;
            (**p_Dcb).Header.FileSize.QuadPart = (dir_size) as i64;
        }

        {
            let dcb_u = (**p_Dcb).Specific.Dcb.as_mut() as *mut _FCB__bindgen_ty_1__bindgen_ty_1;
            (*dcb_u).UnusedDirentVbo = 0xFFFFFFFFu32;
            (*dcb_u).DeletedDirentHint = 0xFFFFFFFFu32;
        }

        {
            let dcb_inner = (**p_Dcb).Specific.Dcb.as_mut() as *mut _FCB__bindgen_ty_1__bindgen_ty_1;
            RtlInitializeBitMap(
                addr_of_mut!((*dcb_inner).FreeDirentBitmap) as *mut RTL_BITMAP,
                core::ptr::null_mut(),
                0,
            );
        }
        FatCheckFreeDirentBitmap(IrpContext, (*p_Dcb));

        #[cfg(ntddi_ge_ntddi_win8)]
        {
            FsRtlInitializeOplock(FatGetFcbOplock(*p_Dcb));
        }
        // #endif

    }, |_, abnormal| {
        DebugUnwind(b"FatCreateRootDcb\0".as_ptr() as *const i8);

        if abnormal {
            // Abnormal termination cleanup
            if !(*p_UnwindFileObject).is_null() {
                ntkrnl_sys::ob::ObDereferenceObject((*p_UnwindFileObject) as *mut core::ffi::c_void);
            }
            if !(*p_UnwindMcb).is_null() {
                FsRtlUninitializeLargeMcb(*p_UnwindMcb);
            }
            if !(*p_UnwindResource).is_null() {
                FatFreeResource(*p_UnwindResource);
            }
            if !(*p_UnwindResource2).is_null() {
                FatFreeResource(*p_UnwindResource2);
            }
            for i in 0..(*p_UnwindStorage).len() {
                if !(*p_UnwindStorage)[i].is_null() {
                    ExFreePool((*p_UnwindStorage)[i]);
                }
            }
            (*Vcb).RootDcb = core::ptr::null_mut();
        }

        DebugTrace(-1, 0, b"FatCreateRootDcb -> %p\n\0".as_ptr() as *const i8, (*p_Dcb));
    });
}
#[no_mangle]
pub unsafe extern "system" fn FatCreateFcb(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    ParentDcb: *mut DCB,
    LfnOffsetWithinDirectory: u32,
    DirentOffsetWithinDirectory: u32,
    Dirent: *mut DIRENT,
    Lfn: *mut _UNICODE_STRING,
    OrigLfn: *mut _UNICODE_STRING,
    IsPagingFile: BOOLEAN,
    SingleResource: BOOLEAN,
) -> *mut FCB {
    // Forward declarations for unknown symbols

    // Locals (must match C exactly)
    let mut fcb: *mut FCB = core::ptr::null_mut();
    let mut pool_type: POOL_TYPE = core::mem::zeroed();

    // Unwind storage
    let mut unwind_storage: [*mut core::ffi::c_void; 2] = [
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ];
    let mut unwind_resource: *mut ERESOURCE = core::ptr::null_mut();
    let mut unwind_resource2: *mut ERESOURCE = core::ptr::null_mut();
    let mut unwind_entry_list: *mut core::ffi::c_void = core::ptr::null_mut(); // PLIST_ENTRY
    let mut unwind_mcb: *mut LARGE_MCB = core::ptr::null_mut();
    let mut unwind_file_lock: *mut FILE_LOCK = core::ptr::null_mut();
    let mut unwind_oplock: *mut OPLOCK = core::ptr::null_mut();

    // PAGED_CODE();  (omitted)

    // UNREFERENCED_PARAMETER( OrigLfn );
    let _ = OrigLfn;

    DebugTrace(1, DEBUG_TRACE_STRUCSUP, b"FatCreateFcb\n\0".as_ptr() as *const i8, 0);

    let p_fcb = core::ptr::addr_of_mut!(fcb);
    let p_unwind_storage = core::ptr::addr_of_mut!(unwind_storage);
    let p_unwind_resource = core::ptr::addr_of_mut!(unwind_resource);
    let p_unwind_resource2 = core::ptr::addr_of_mut!(unwind_resource2);
    let p_unwind_entry_list = core::ptr::addr_of_mut!(unwind_entry_list);
    let p_unwind_mcb = core::ptr::addr_of_mut!(unwind_mcb);
    let p_unwind_file_lock = core::ptr::addr_of_mut!(unwind_file_lock);
    let p_unwind_oplock = core::ptr::addr_of_mut!(unwind_oplock);
    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        // Determine pool type
        if IsPagingFile != 0 {
            pool_type = NonPagedPoolNx;
            *p_fcb = FsRtlAllocatePoolWithTag(NonPagedPoolNx, core::mem::size_of::<FCB>() as u32, TAG_FCB) as *mut FCB;
            (*p_unwind_storage)[0] = (*p_fcb) as *mut core::ffi::c_void;
        } else {
            pool_type = PagedPool;
            *p_fcb = FatAllocateFcb();
            (*p_unwind_storage)[0] = (*p_fcb) as *mut core::ffi::c_void;
        }

        // Zero out FCB
        core::ptr::write_bytes((*p_fcb) as *mut u8, 0, core::mem::size_of::<FCB>());

        // Allocate NonPaged part
        (*p_unwind_storage)[1] = (FatAllocateNonPagedFcb()).cast();
        (**p_fcb).NonPaged = (*p_unwind_storage)[1] as *mut NON_PAGED_FCB;
        core::ptr::write_bytes(
            (**p_fcb).NonPaged as *mut u8,
            0,
            core::mem::size_of::<NON_PAGED_FCB>(),
        );

        // Set node type code and byte size
        (**p_fcb).Header.NodeTypeCode = (FAT_NTC_FCB) as i16;
        (**p_fcb).Header.NodeByteSize = (core::mem::size_of::<FCB>() as u16) as i16; // Note: C uses sizeof(FCB) which may be u16

        // Set FcbCondition
        (**p_fcb).FcbCondition = FcbGood;

        // If paging file, set flags
        if IsPagingFile != 0 {
            SetFlag(
                core::ptr::addr_of_mut!((**p_fcb).FcbState),
                FCB_STATE_PAGING_FILE | FCB_STATE_SYSTEM_FILE,
            );
        }

        // Allocate main resource
        *p_unwind_resource = FatAllocateResource();
        (**p_fcb).Header.Resource = (*p_unwind_resource);

        // Allocate paging I/O resource (or share)
        if SingleResource != 0 {
            (**p_fcb).Header.PagingIoResource = (**p_fcb).Header.Resource;
        } else {
            *p_unwind_resource2 = FatAllocateResource();
            (**p_fcb).Header.PagingIoResource = (*p_unwind_resource2);
        }

        // Insert into parent's DCB queue
        {
            let pdcb_inner = (*ParentDcb).Specific.Dcb.as_mut() as *mut _FCB__bindgen_ty_1__bindgen_ty_1;
            InsertTailList(
                core::ptr::addr_of_mut!((*pdcb_inner).ParentDcbQueue) as *mut LIST_ENTRY,
                core::ptr::addr_of_mut!((**p_fcb).ParentDcbLinks) as *mut LIST_ENTRY,
            );
        }
        *p_unwind_entry_list = core::ptr::addr_of_mut!((**p_fcb).ParentDcbLinks) as *mut core::ffi::c_void;

        // Set parent DCB, VCB
        (**p_fcb).ParentDcb = ParentDcb;
        (**p_fcb).Vcb = Vcb;

        // Store directory offsets
        (**p_fcb).LfnOffsetWithinDirectory = LfnOffsetWithinDirectory;
        (**p_fcb).DirentOffsetWithinDirectory = DirentOffsetWithinDirectory;

        // Copy DirentFatFlags and compute LastWriteTime
        (**p_fcb).DirentFatFlags = (*Dirent).Attributes;

        (**p_fcb).LastWriteTime = FatFatTimeToNtTime(
            IrpContext,
            core::ptr::read(core::ptr::addr_of!((*Dirent).LastWriteTime)),
            0u8,
        );

        // Chicago-specific time fields
        if (*&raw const FatData).ChicagoMode() != 0 {
            let mut fat_system_jan_one_1980: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
            let fat_jan_one_1980_ptr: *const LARGE_INTEGER = &FatJanOne1980 as *const LARGE_INTEGER;

            // If either date field could be zero, get system 1/1/80
            if ((Dirent as *const u16).offset(9).read() & (Dirent as *const u16).offset(8).read()) == 0 {
                ExLocalTimeToSystemTime(
                    fat_jan_one_1980_ptr as *mut LARGE_INTEGER,
                    &mut fat_system_jan_one_1980 as *mut LARGE_INTEGER,
                );
            }

            if (Dirent as *const u16).offset(9).read() != 0 {
                (**p_fcb).LastAccessTime = FatFatDateToNtTime(
                    IrpContext,
                    core::ptr::read(core::ptr::addr_of!((*Dirent).LastAccessDate)),
                );
            } else {
                (**p_fcb).LastAccessTime = fat_system_jan_one_1980;
            }

            if (Dirent as *const u16).offset(8).read() != 0 {
                (**p_fcb).CreationTime = FatFatTimeToNtTime(
                    IrpContext,
                    core::ptr::read(core::ptr::addr_of!((*Dirent).CreationTime)),
                    (*Dirent).CreationMSec,
                );
            } else {
                (**p_fcb).CreationTime = fat_system_jan_one_1980;
            }
        }

        // Initialize advanced header
        ExInitializeFastMutex(&mut (*(**p_fcb).NonPaged).AdvancedFcbHeaderMutex);
        FsRtlSetupAdvancedHeader(
            &mut (**p_fcb).Header as *mut FSRTL_ADVANCED_FCB_HEADER,
            &mut (*(**p_fcb).NonPaged).AdvancedFcbHeaderMutex,
        );

        // Setup flags for paging file
        if IsPagingFile != 0 {
            (**p_fcb).Header.Flags2 |= FSRTL_FLAG2_IS_PAGING_FILE as u8;
            (**p_fcb).Header.Flags2 &= !(FSRTL_FLAG2_SUPPORTS_FILTER_CONTEXTS as u8);
        }

        // Initialize MCB
        FsRtlInitializeLargeMcb(&mut (**p_fcb).Mcb as *mut LARGE_MCB, pool_type);
        *p_unwind_mcb = &mut (**p_fcb).Mcb;

        // Copy file size info from dirent
        (**p_fcb).Header.FileSize.__bindgen_anon_1.LowPart = (*Dirent).FileSize;
        (**p_fcb).Header.ValidDataLength.__bindgen_anon_1.LowPart = (*Dirent).FileSize;
        (**p_fcb).ValidDataToDisk = (*Dirent).FileSize;

        // First cluster calculation
        (**p_fcb).FirstClusterOfFile = (*Dirent).FirstClusterOfFile as u32;

        if FatIsFat32(Vcb) != 0 {
            (**p_fcb).FirstClusterOfFile += ((*(*Dirent).__bindgen_anon_1.FirstClusterOfFileHi.as_ref()) as u32) << 16;
        }

        if (**p_fcb).FirstClusterOfFile == 0 {
            (**p_fcb).Header.AllocationSize.QuadPart = 0;
        } else {
            (**p_fcb).Header.AllocationSize.QuadPart = FCB_LOOKUP_ALLOCATIONSIZE_HINT;
        }

        // Initialize file lock
        {
            let fcb_inner = (**p_fcb).Specific.Fcb.as_mut() as *mut _FCB__bindgen_ty_1__bindgen_ty_2;
            FsRtlInitializeFileLock(
                core::ptr::addr_of_mut!((*fcb_inner).FileLock),
                None,
                None,
            );
            *p_unwind_file_lock = core::ptr::addr_of_mut!((*fcb_inner).FileLock);
        }

        // Initialize oplock
        FsRtlInitializeOplock(FatGetFcbOplock(*p_fcb));
        *p_unwind_oplock = FatGetFcbOplock(*p_fcb);

        // Fast I/O possible
        (**p_fcb).Header.IsFastIoPossible = 1; // TRUE

        // Build names (must be last)
        FatConstructNamesInFcb(
            IrpContext,
            (*p_fcb),
            Dirent,
            Lfn,
        );

        // Drop shortname hint
        (**p_fcb).ShortName.FileNameDos = 1; // TRUE
    }, |_, abnormal| {
        DebugUnwind(b"FatCreateFcb\0".as_ptr() as *const i8);

        if abnormal {
            if !(*p_unwind_oplock).is_null() {
                FsRtlUninitializeOplock(*p_unwind_oplock);
            }
            if !(*p_unwind_file_lock).is_null() {
                FsRtlUninitializeFileLock(*p_unwind_file_lock);
            }
            if !(*p_unwind_mcb).is_null() {
                FsRtlUninitializeLargeMcb(*p_unwind_mcb);
            }
            if !(*p_unwind_entry_list).is_null() {
                RemoveEntryList((*p_unwind_entry_list) as *mut LIST_ENTRY);
            }
            if !(*p_unwind_resource).is_null() {
                FatFreeResource(*p_unwind_resource);
            }
            if !(*p_unwind_resource2).is_null() {
                FatFreeResource(*p_unwind_resource2);
            }

            for i in 0..2 {
                if !(*p_unwind_storage)[i].is_null() {
                    ExFreePool((*p_unwind_storage)[i]);
                }
            }
        }

        DebugTrace(-1, DEBUG_TRACE_STRUCSUP, b"FatCreateFcb -> %p\n\0".as_ptr() as *const i8, (*p_fcb));
    });

    fcb
}

#[no_mangle]
pub unsafe extern "system" fn FatCreateDcb(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    ParentDcb: *mut DCB,
    LfnOffsetWithinDirectory: u32,
    DirentOffsetWithinDirectory: u32,
    Dirent: *mut DIRENT,
    Lfn: *mut _UNICODE_STRING,
) -> *mut DCB {
    const IRP_CONTEXT_FLAG_WAIT: u32 = 0x02;  // nolint: const-contradiction (fastfat's own flag set, fatstruc.h:1563)
    const FAT_NTC_DCB: u32 = 0x0503;
    const FcbGood: u32 = 1;
    const FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64 = -1;
    const Dbg: u32 = 0;

    let mut Dcb: *mut DCB = core::ptr::null_mut();
    let mut UnwindStorage: [*mut core::ffi::c_void; 2] = [core::ptr::null_mut(), core::ptr::null_mut()];
    let mut UnwindResource: *mut ERESOURCE = core::ptr::null_mut();
    let mut UnwindResource2: *mut ERESOURCE = core::ptr::null_mut();
    let mut UnwindEntryList: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut UnwindMcb: *mut LARGE_MCB = core::ptr::null_mut();
    let mut UnwindOplock: *mut OPLOCK = core::ptr::null_mut();

    DebugTrace(1, Dbg, b"FatCreateDcb\n\0".as_ptr() as *const i8, 0);

    let p_Dcb = core::ptr::addr_of_mut!(Dcb);
    let p_UnwindStorage = core::ptr::addr_of_mut!(UnwindStorage);
    let p_UnwindResource = core::ptr::addr_of_mut!(UnwindResource);
    let p_UnwindResource2 = core::ptr::addr_of_mut!(UnwindResource2);
    let p_UnwindEntryList = core::ptr::addr_of_mut!(UnwindEntryList);
    let p_UnwindMcb = core::ptr::addr_of_mut!(UnwindMcb);
    let p_UnwindOplock = core::ptr::addr_of_mut!(UnwindOplock);
    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        // NT_ASSERT( FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) ) -- an assert, not an exit.
        // (The translation's invented NULL checks below were dropped: they left the unwound,
        // freed Dcb as the return value. C relies on the allocators raising.)
        *p_Dcb = FatAllocateFcb();
        (*p_UnwindStorage)[0] = (*p_Dcb) as *mut core::ffi::c_void;
        core::ptr::write_bytes((*p_Dcb) as *mut u8, 0, core::mem::size_of::<DCB>());

        let non_paged = FatAllocateNonPagedFcb();
        (**p_Dcb).NonPaged = non_paged;
        (*p_UnwindStorage)[1] = non_paged as *mut core::ffi::c_void;
        core::ptr::write_bytes(non_paged as *mut u8, 0, core::mem::size_of::<NON_PAGED_FCB>());

        (**p_Dcb).Header.NodeTypeCode = (FAT_NTC_DCB) as i16;
        (**p_Dcb).Header.NodeByteSize = (core::mem::size_of::<DCB>() as u16) as i16;
        (**p_Dcb).FcbCondition = FcbGood;

        let res = FatAllocateResource();
        (**p_Dcb).Header.Resource = res;
        *p_UnwindResource = res;

        let pio_res = FatAllocateResource();
        (**p_Dcb).Header.PagingIoResource = pio_res;
        *p_UnwindResource2 = pio_res;

        {
            let parent_dcb_inner = (*ParentDcb).Specific.Dcb.as_mut() as *mut _FCB__bindgen_ty_1__bindgen_ty_1;
            InsertHeadList(
                core::ptr::addr_of_mut!((*parent_dcb_inner).ParentDcbQueue),
                core::ptr::addr_of_mut!((**p_Dcb).ParentDcbLinks),
            );
        }
        *p_UnwindEntryList = core::ptr::addr_of_mut!((**p_Dcb).ParentDcbLinks);

        (**p_Dcb).ParentDcb = ParentDcb;
        (**p_Dcb).Vcb = Vcb;

        (**p_Dcb).LfnOffsetWithinDirectory = LfnOffsetWithinDirectory;
        (**p_Dcb).DirentOffsetWithinDirectory = DirentOffsetWithinDirectory;

        (**p_Dcb).DirentFatFlags = (*Dirent).Attributes;
        (**p_Dcb).LastWriteTime = FatFatTimeToNtTime(IrpContext, core::ptr::read(core::ptr::addr_of!((*Dirent).LastWriteTime)), 0);

        let fat_data = &mut *core::ptr::addr_of_mut!(FatData);
        if fat_data.ChicagoMode() != 0 {
            let mut fat_system_jan_one_1980 = LARGE_INTEGER { QuadPart: 0 };
            let dirent_as_u16: *const u16 = Dirent as *const u16;
            if ((*dirent_as_u16.add(9)) & (*dirent_as_u16.add(8))) == 0 {
                ExLocalTimeToSystemTime(core::ptr::addr_of!(FatJanOne1980), core::ptr::addr_of_mut!(fat_system_jan_one_1980));
            }
            if *dirent_as_u16.add(9) != 0 {
                (**p_Dcb).LastAccessTime = FatFatDateToNtTime(IrpContext, core::ptr::read(core::ptr::addr_of!((*Dirent).LastAccessDate)));
            } else {
                (**p_Dcb).LastAccessTime = fat_system_jan_one_1980;
            }
            if *dirent_as_u16.add(8) != 0 {
                (**p_Dcb).CreationTime = FatFatTimeToNtTime(
                    IrpContext,
                    core::ptr::read(core::ptr::addr_of!((*Dirent).CreationTime)),
                    (*Dirent).CreationMSec,
                );
            } else {
                (**p_Dcb).CreationTime = fat_system_jan_one_1980;
            }
        }

        ExInitializeFastMutex(core::ptr::addr_of_mut!(
            (*(**p_Dcb).NonPaged).AdvancedFcbHeaderMutex
        ));
        FsRtlSetupAdvancedHeader(
            core::ptr::addr_of_mut!((**p_Dcb).Header),
            core::ptr::addr_of_mut!((*(**p_Dcb).NonPaged).AdvancedFcbHeaderMutex),
        );

        FsRtlInitializeLargeMcb(core::ptr::addr_of_mut!((**p_Dcb).Mcb), PagedPool);
        *p_UnwindMcb = core::ptr::addr_of_mut!((**p_Dcb).Mcb);

        (**p_Dcb).FirstClusterOfFile = (*Dirent).FirstClusterOfFile as u32;
        if FatIsFat32((**p_Dcb).Vcb) != 0 {
            (**p_Dcb).FirstClusterOfFile += (*(*Dirent).__bindgen_anon_1.FirstClusterOfFileHi.as_ref() as u32) << 16;
        }
        if (**p_Dcb).FirstClusterOfFile == 0 {
            (**p_Dcb).Header.AllocationSize.QuadPart = 0;
        } else {
            (**p_Dcb).Header.AllocationSize.QuadPart = FCB_LOOKUP_ALLOCATIONSIZE_HINT;
        }

        {
            let dcb_inner = (**p_Dcb).Specific.Dcb.as_mut() as *mut _FCB__bindgen_ty_1__bindgen_ty_1;
            InitializeListHead(core::ptr::addr_of_mut!((*dcb_inner).ParentDcbQueue));
            RtlInitializeBitMap(
                core::ptr::addr_of_mut!((*dcb_inner).FreeDirentBitmap) as *mut RTL_BITMAP,
                core::ptr::null_mut(),
                0,
            );
            (*dcb_inner).UnusedDirentVbo = 0xffffffff;
            (*dcb_inner).DeletedDirentHint = 0xffffffff;
        }

        // NTDDI >= WIN8 only (strucsup.c 1890-1897). Pre-WIN8 this "oplock" is a word of the
        // DCB's in-union free-dirent bitmap, and the unwind would uninitialize it.
        #[cfg(ntddi_ge_ntddi_win8)]
        {
            let oplock = FatGetFcbOplock(*p_Dcb);
            FsRtlInitializeOplock(oplock);
            *p_UnwindOplock = oplock;
        }

        FatConstructNamesInFcb(IrpContext, (*p_Dcb), Dirent, Lfn);
        (**p_Dcb).ShortName.FileNameDos = 1;
    }, |_, abnormal| {
        if abnormal {
            if !(*p_UnwindOplock).is_null() {
                FsRtlUninitializeOplock(*p_UnwindOplock);
            }
            if !(*p_UnwindMcb).is_null() {
                FsRtlUninitializeLargeMcb(*p_UnwindMcb);
            }
            if !(*p_UnwindEntryList).is_null() {
                RemoveEntryList(*p_UnwindEntryList);
            }
            if !(*p_UnwindResource).is_null() {
                FatFreeResource(*p_UnwindResource);
            }
            if !(*p_UnwindResource2).is_null() {
                FatFreeResource(*p_UnwindResource2);
            }
            let mut i = 0;
            while i < (*p_UnwindStorage).len() {
                if !(*p_UnwindStorage)[i].is_null() {
                    ExFreePool((*p_UnwindStorage)[i]);
                }
                i += 1;
            }
        }

        DebugTrace(-1, Dbg, b"FatCreateDcb -> %p\n\0".as_ptr() as *const i8, (*p_Dcb) as usize);
    });
    Dcb
}

#[no_mangle]
pub unsafe extern "system" fn FatDeleteFcb(IrpContext: *mut IRP_CONTEXT, FcbPtr: *mut *mut FCB) {
    let Fcb: *mut FCB = *FcbPtr;

    // PAGED_CODE(); - ignored

    DebugTrace(1, (Dbg as i32) as u32, b"FatDeleteFcb, Fcb = %p\n\0".as_ptr() as *const _, Fcb);

    // We can only delete this record if the open count is zero.
    if (*Fcb).OpenCount != 0 {
        DebugDump(b"Error deleting Fcb, Still Open\n\0".as_ptr() as *const _, 0, Fcb);
        FatBugCheck(0, 0, 0);
    }

    // Better be an FCB/DCB.
    if ((*Fcb).Header.NodeTypeCode != (FAT_NTC_DCB) as i16) &&
       ((*Fcb).Header.NodeTypeCode != (FAT_NTC_ROOT_DCB) as i16) &&
       ((*Fcb).Header.NodeTypeCode != (FAT_NTC_FCB) as i16) {
        FatBugCheck(0, 0, 0);
    }

    // If this is a DCB then remove every Notify record from the two notify queues
    if ((*Fcb).Header.NodeTypeCode == (FAT_NTC_DCB) as i16) || ((*Fcb).Header.NodeTypeCode == (FAT_NTC_ROOT_DCB) as i16) {
        let dcb_inner = (*Fcb).Specific.Dcb.as_ref() as *const _FCB__bindgen_ty_1__bindgen_ty_1;
        // If we allocated a free dirent bitmap buffer, free it.
        {
            let bitmap_buf = (*dcb_inner).FreeDirentBitmap.__bindgen_anon_1.__bindgen_anon_1.as_ref().Buffer;
            if (!bitmap_buf.is_null()) &&
               (bitmap_buf != core::ptr::addr_of!((*dcb_inner).FreeDirentBitmapBuffer) as *const u32 as *mut u32) {
                ExFreePool(bitmap_buf as *mut core::ffi::c_void);
            }
        }

        // Uninitialize the oplock -- NTDDI >= WIN8 only (strucsup.c 2028-2034): pre-WIN8 a
        // DCB has none, and this nulled a word of its free-dirent bitmap and freed "it".
        #[cfg(ntddi_ge_ntddi_win8)]
        FsRtlUninitializeOplock(FatGetFcbOplock(Fcb));

        NT_ASSERT(((*dcb_inner).DirectoryFileOpenCount == 0) as i32);
        NT_ASSERT(IsListEmpty(core::ptr::addr_of!((*dcb_inner).ParentDcbQueue) as *mut LIST_ENTRY) as i32);
        NT_ASSERT((core::ptr::null_mut::<FILE_OBJECT>() == (*dcb_inner).DirectoryFile) as i32);
    } else {
        // Uninitialize the byte range file locks and opportunistic locks
        let fcb_inner = (*Fcb).Specific.Fcb.as_ref() as *const _FCB__bindgen_ty_1__bindgen_ty_2;
        FsRtlUninitializeFileLock(core::ptr::addr_of!((*fcb_inner).FileLock) as *mut FILE_LOCK);
        FsRtlUninitializeOplock(FatGetFcbOplock(Fcb));
    }

    // Release any Filter Context structures associated with this FCB
    FsRtlTeardownPerStreamContexts(core::ptr::addr_of!((*Fcb).Header) as *mut FSRTL_ADVANCED_FCB_HEADER);

    // Uninitialize the Mcb
    FsRtlUninitializeLargeMcb(core::ptr::addr_of!((*Fcb).Mcb) as *mut LARGE_MCB);

    // If this is not the root dcb then we need to remove ourselves from our parents Dcb queue
    if (*Fcb).Header.NodeTypeCode != (FAT_NTC_ROOT_DCB) as i16 {
        RemoveEntryList(core::ptr::addr_of!((*Fcb).ParentDcbLinks) as *mut LIST_ENTRY);
    }

    // Remove the entry from the splay table if there is still one.
    if FlagOn((*Fcb).FcbState, FCB_STATE_NAMES_IN_SPLAY_TREE) != 0 {
        FatRemoveNames(IrpContext, Fcb);
    }

    // Free the file name pool if allocated.
    if (*Fcb).Header.NodeTypeCode != (FAT_NTC_ROOT_DCB) as i16 {
        // If we blew up at inconvenient times, the shortname could be null...
        if !(*(*Fcb).ShortName.Name.Oem.as_ref()).Buffer.is_null() {
            ExFreePool((*(*Fcb).ShortName.Name.Oem.as_ref()).Buffer as *mut core::ffi::c_void);
        }
        if !(*Fcb).FullFileName.Buffer.is_null() {
            ExFreePool((*Fcb).FullFileName.Buffer as *mut core::ffi::c_void);
        }
    }

    if !(*Fcb).ExactCaseLongName.Buffer.is_null() {
        ExFreePool((*Fcb).ExactCaseLongName.Buffer as *mut core::ffi::c_void);
    }

    // SYSCACHE_COMPILE: WriteMask is conditionally compiled; skip if absent

    // Finally deallocate the Fcb and non-paged fcb records
    FatFreeResource((*Fcb).Header.Resource);

    if (*Fcb).Header.PagingIoResource != (*Fcb).Header.Resource {
        FatFreeResource((*Fcb).Header.PagingIoResource);
    }

    // If an Event was allocated, get rid of it.
    if !(*(*Fcb).NonPaged).OutstandingAsyncEvent.is_null() {
        ExFreePool((*(*Fcb).NonPaged).OutstandingAsyncEvent as *mut core::ffi::c_void);
    }

    FatFreeNonPagedFcb((*Fcb).NonPaged);

    (*Fcb).Header.NodeTypeCode = (NTC_UNDEFINED) as i16;

    FatFreeFcb(Fcb);
    *FcbPtr = core::ptr::null_mut();

    // and return to our caller
    DebugTrace(-1, (Dbg as i32) as u32, b"FatDeleteFcb -> VOID\n\0".as_ptr() as *const _, 0);
}

#[no_mangle]
pub unsafe extern "system" fn FatCreateCcb(
    IrpContext: *mut IRP_CONTEXT,
) -> *mut CCB {
    // External declarations for unknown symbols

    // Unknown constants; values taken from typical NT definitions
    const DEBUG_TRACE_STRUCSUP: u32 = 0x20000000;
    const Dbg: u32 = DEBUG_TRACE_STRUCSUP;
    const FAT_NTC_CCB: u16 = 0x0507;

    let Ccb: *mut CCB;

    PAGED_CODE!();

    DebugTrace(
        1,
        Dbg,
        b"FatCreateCcb\n\0".as_ptr() as *const i8,
        0i32,
    );

    Ccb = FatAllocateCcb();

    RtlZeroMemory(Ccb as *mut core::ffi::c_void, core::mem::size_of::<CCB>());

    (*Ccb).NodeTypeCode = FAT_NTC_CCB;
    (*Ccb).NodeByteSize = (core::mem::size_of::<CCB>()) as i16;

    DebugTrace(
        -1,
        Dbg,
        b"FatCreateCcb -> %p\n\0".as_ptr() as *const i8,
        Ccb,
    );

    let _ = IrpContext;

    Ccb
}
#[no_mangle]
pub unsafe extern "system" fn FatDeallocateCcbStrings(Ccb: *mut CCB) {
    PAGED_CODE!();
    if ((*Ccb).Flags & CCB_FLAG_FREE_UNICODE) != 0 {
        let ccb_inner = (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut() as *mut _CCB__bindgen_ty_1__bindgen_ty_1;
        NT_ASSERT(if (*ccb_inner).UnicodeQueryTemplate.Buffer.is_null() { 0 } else { 1 });
        NT_ASSERT(if ((*Ccb).Flags & CCB_FLAG_CLOSE_CONTEXT) != 0 { 0 } else { 1 });
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*ccb_inner).UnicodeQueryTemplate));
    }
    if ((*Ccb).Flags & CCB_FLAG_FREE_OEM_BEST_FIT) != 0 {
        let ccb_inner = (*Ccb).__bindgen_anon_1.__bindgen_anon_1.as_mut() as *mut _CCB__bindgen_ty_1__bindgen_ty_1;
        NT_ASSERT(if (*ccb_inner).OemQueryTemplate.Wild.as_ref().Buffer.is_null() { 0 } else { 1 });
        NT_ASSERT(if ((*Ccb).Flags & CCB_FLAG_CLOSE_CONTEXT) != 0 { 0 } else { 1 });
        RtlFreeOemString((*ccb_inner).OemQueryTemplate.Wild.as_mut() as *mut OEM_STRING);
    }
    (*Ccb).Flags &= !(CCB_FLAG_FREE_OEM_BEST_FIT | CCB_FLAG_FREE_UNICODE);
}

#[no_mangle]
pub unsafe extern "system" fn FatDeleteCcb(
    irp_context: *mut IRP_CONTEXT,
    ccb: *mut *mut CCB,
) {
    // PAGED_CODE();

    DebugTrace(
        1,
        0x1c, // DEBUG_TRACE_STRUCSUP
        b"FatDeleteCcb, Ccb = %p\n\0".as_ptr() as *const i8,
        *ccb,
    );

    FatDeallocateCcbStrings(*ccb);

    FatFreeCcb(*ccb);
    *ccb = core::ptr::null_mut();

    DebugTrace(
        -1,
        0x1c, // DEBUG_TRACE_STRUCSUP
        b"FatDeleteCcb -> VOID\n\0".as_ptr() as *const i8,
        0,
    );

    _ = irp_context;
}
#[no_mangle]
pub unsafe extern "system" fn FatCreateIrpContext(
    irp: *mut IRP,
    wait: i32, // BOOLEAN
) -> *mut IRP_CONTEXT {
    // PAGED_CODE();
    DebugTrace(1, Dbg, b"FatCreateIrpContext\n\0".as_ptr() as *const i8, 0);

    let irp_sp = IoGetCurrentIrpStackLocation(irp);

    //
    //  The only operations a filesystem device object should ever receive
    //  are create/teardown of fsdo handles and operations which do not
    //  occur in the context of fileobjects (i.e., mount).
    //
    if FatDeviceIsFatFsdo((*irp_sp).DeviceObject) != 0 {
        if (*irp_sp).FileObject.is_null() == false
            && (*irp_sp).MajorFunction != IRP_MJ_CREATE
            && (*irp_sp).MajorFunction != IRP_MJ_CLEANUP
            && (*irp_sp).MajorFunction != IRP_MJ_CLOSE
        {
            {
                extern "C" {
                    fn DbgPrint(fmt: *const i8, ...) -> u32;
                }
                DbgPrint(
                    b"[FASTFAT] FatCreateIrpContext RAISE on FSDO: Mj=%u Mn=%u FileObj=%p Dev=%p Irp=%p IrpSp=%p\n\0"
                        .as_ptr() as *const i8,
                    (*irp_sp).MajorFunction as u32,
                    (*irp_sp).MinorFunction as u32,
                    (*irp_sp).FileObject,
                    (*irp_sp).DeviceObject,
                    irp,
                    irp_sp,
                );
                let fo = (*irp_sp).FileObject;
                if !fo.is_null() {
                    let fo_dev = (*fo).DeviceObject;
                    let fo_vpb = (*fo).Vpb;
                    let dev_vpb = if !fo_dev.is_null() { (*fo_dev).Vpb } else { core::ptr::null_mut() };
                    let dev_vpb_dev = if !dev_vpb.is_null() { (*dev_vpb).DeviceObject } else { core::ptr::null_mut() };
                    DbgPrint(
                        b"[FASTFAT]   chain: FO->Vpb=%p FO->Dev=%p FO->Dev->Vpb=%p ->Vpb->Dev=%p FO->Flags=%08x FatDiskFsdo=%p FatCdFsdo=%p\n\0"
                            .as_ptr() as *const i8,
                        fo_vpb,
                        fo_dev,
                        dev_vpb,
                        dev_vpb_dev,
                        (*fo).Flags,
                        FatData.DiskFileSystemDeviceObject,
                        FatData.CdromFileSystemDeviceObject,
                    );
                }
            }
            ExRaiseStatus(STATUS_INVALID_DEVICE_REQUEST);
        }

        NT_ASSERT(
            (((*irp_sp).FileObject.is_null() == false)
                || ((*irp_sp).MajorFunction == (IRP_MJ_FILE_SYSTEM_CONTROL) as u8
                    && (*irp_sp).MinorFunction == IRP_MN_USER_FS_REQUEST as u8
                    && (*irp_sp).Parameters.FileSystemControl.FsControlCode == FSCTL_INVALIDATE_VOLUMES)
                || ((*irp_sp).MajorFunction == (IRP_MJ_FILE_SYSTEM_CONTROL) as u8
                    && (*irp_sp).MinorFunction == (IRP_MN_MOUNT_VOLUME) as u8)
                || (*irp_sp).MajorFunction == (IRP_MJ_SHUTDOWN
                as i32) as u8) as i32,
        );
    }

    //
    //  Attempt to allocate from the region first and failing that allocate
    //  from pool.
    //
    FatFsdEntryCount += 1;
    DebugDoit(FatFsdEntryCount);

    let mut irp_context: *mut IRP_CONTEXT = FatAllocateIrpContext();

    //
    //  Zero out the irp context.
    //
    RtlZeroMemory(irp_context as *mut c_void, core::mem::size_of::<IRP_CONTEXT>());

    //
    //  Set the proper node type code and node byte size
    //
    (*irp_context).NodeTypeCode = (FAT_NTC_IRP_CONTEXT) as u16;
    (*irp_context).NodeByteSize = (core::mem::size_of::<IRP_CONTEXT>() as u16) as i16; // or u32? assume field is DWORD

    //
    //  Set the originating Irp field
    //
    (*irp_context).OriginatingIrp = irp;

    //
    //  Major/Minor Function codes
    //
    (*irp_context).MajorFunction = (*irp_sp).MajorFunction;
    (*irp_context).MinorFunction = (*irp_sp).MinorFunction;

    //
    //  Copy RealDevice for workque algorithms, and also set Write Through
    //  and Removable Media if there is a file object.  Only file system
    //  control Irps won't have a file object, and they should all have
    //  a Vpb as the first IrpSp location.
    //
    if (*irp_sp).FileObject.is_null() == false {
        let file_object: *mut FILE_OBJECT = (*irp_sp).FileObject;

        (*irp_context).RealDevice = (*file_object).DeviceObject;
        let vdo = (*irp_sp).DeviceObject as *mut VOLUME_DEVICE_OBJECT;
        (*irp_context).Vcb = &(*vdo).Vcb as *const VCB as *mut VCB;

        //
        //  See if the request is Write Through.
        //
        if IsFileWriteThrough(file_object, (*irp_context).Vcb) != 0
            || ((*irp_sp).MajorFunction == IRP_MJ_WRITE
                && BooleanFlagOn((*irp).Flags, IRP_NOCACHE) != 0
                && BooleanFlagOn(((*irp_sp).Flags) as u32, SL_WRITE_THROUGH) != 0)
        {
            SetFlag(&mut (*irp_context).Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH);
        }
    } else if (*irp_context).MajorFunction == (IRP_MJ_FILE_SYSTEM_CONTROL) as u8 {
        (*irp_context).RealDevice = (*(*irp_sp).Parameters.MountVolume.Vpb).RealDevice;
    }

    //
    //  Set the wait parameter
    //
    if wait != 0 {
        SetFlag(&mut (*irp_context).Flags, IRP_CONTEXT_FLAG_WAIT);
    }

    //
    //  Set the recursive file system call parameter.
    //
    if IoGetTopLevelIrp() != irp {
        SetFlag(&mut (*irp_context).Flags, IRP_CONTEXT_FLAG_RECURSIVE_CALL);
    }

    //
    //  return and tell the caller
    //
    DebugTrace(-1, Dbg, b"FatCreateIrpContext -> %p\n\0".as_ptr() as *const i8, irp_context);

    irp_context
}
#[no_mangle]
pub unsafe extern "system" fn FatDeleteIrpContext_Real(IrpContext: *mut IRP_CONTEXT) {
    PAGED_CODE!();

    DebugTrace(1, Dbg, b"FatDeleteIrpContext, IrpContext = %p\n\0".as_ptr() as *const i8, IrpContext);

    NT_ASSERT(((*IrpContext).NodeTypeCode == (FAT_NTC_IRP_CONTEXT) as u16) as i32);
    NT_ASSERT(((*IrpContext).PinCount == 0) as i32);

    if !(*IrpContext).FatIoContext.is_null() {
        if ((*IrpContext).Flags & IRP_CONTEXT_STACK_IO_CONTEXT) == 0 {
            let fat_io_ctx = (*IrpContext).FatIoContext;
            if !(*fat_io_ctx).ZeroMdl.is_null() {
                IoFreeMdl( (*fat_io_ctx).ZeroMdl );
            }
            ExFreePool(fat_io_ctx as *mut core::ffi::c_void);
        }
    }

    FatFreeIrpContext( IrpContext );

    DebugTrace(-1, Dbg, b"FatDeleteIrpContext -> VOID\n\0".as_ptr() as *const i8, 0);
    return;
}

#[no_mangle]
pub unsafe extern "system" fn FatGetNextFcbBottomUp(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    TerminationFcb: *mut FCB,
) -> *mut FCB {
    let mut NextFcb: *mut FCB = core::ptr::null_mut();

    PAGED_CODE!();
    UNREFERENCED_PARAMETER(IrpContext as *mut core::ffi::c_void);

    NT_ASSERT(
        ((FatVcbAcquiredExclusive(IrpContext, (*TerminationFcb).Vcb) != 0)
            || (FlagOn((*(*TerminationFcb).Vcb).VcbState, VCB_STATE_FLAG_LOCKED) != 0))
            as i32,
    );

    if !Fcb.is_null() {
        if Fcb == TerminationFcb {
            return core::ptr::null_mut();
        }

        NextFcb = FatGetNextSibling(Fcb);
        if NextFcb.is_null() {
            return (*Fcb).ParentDcb;
        }
    } else {
        NextFcb = TerminationFcb;
    }

    while NodeType(NextFcb) != (FAT_NTC_FCB) as u32 && !FatGetFirstChild(NextFcb).is_null() {
        NextFcb = FatGetFirstChild(NextFcb);
    }

    NextFcb
}

#[no_mangle]
pub unsafe extern "system" fn FatGetNextFcbTopDown(
    IrpContext: *mut IRP_CONTEXT,
    mut Fcb: *mut FCB,
    TerminationFcb: *mut FCB,
) -> *mut FCB {
    PAGED_CODE!();
    UNREFERENCED_PARAMETER(IrpContext as *const core::ffi::c_void);

    let valid = if FatVcbAcquiredExclusive(IrpContext, (*Fcb).Vcb) != 0
        || FlagOn((*((*Fcb).Vcb)).VcbState, VCB_STATE_FLAG_LOCKED) != 0
    {
        1
    } else {
        0
    };
    NT_ASSERT(valid);

    if (NodeType(Fcb) == (FAT_NTC_DCB) as u32 || NodeType(Fcb) == (FAT_NTC_ROOT_DCB) as u32)
        && !IsListEmpty({
            let dcb_i = (*Fcb).Specific.Dcb.as_ref() as *const _FCB__bindgen_ty_1__bindgen_ty_1;
            core::ptr::addr_of!((*dcb_i).ParentDcbQueue)
        })
    {
        return FatGetFirstChild(Fcb);
    }

    if Fcb == TerminationFcb {
        return core::ptr::null_mut();
    }

    let mut Sibling = FatGetNextSibling(Fcb);

    loop {
        if !Sibling.is_null() {
            return if Sibling != TerminationFcb {
                Sibling
            } else {
                core::ptr::null_mut()
            };
        }

        if (*Fcb).ParentDcb == TerminationFcb {
            return core::ptr::null_mut();
        }

        Fcb = (*Fcb).ParentDcb;
        Sibling = FatGetNextSibling(Fcb);
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatSwapVpb(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
) -> i32 {
    unsafe {
        let mut Result: i32 = 0; // FALSE
        let OldVpb: *mut VPB;
        let mut IrpSp: *mut IO_STACK_LOCATION;

        // Make sure we have not already swapped it.
        OldVpb = (*Vcb).Vpb;

        // (prefast annotations omitted – mechanical translation)
        if FlagOn((*Vcb).VcbState, VCB_STATE_FLAG_VPB_MUST_BE_FREED) == 0
            && (*(*OldVpb).RealDevice).Vpb == OldVpb
        {
            // If not the final reference and we are forcing the disconnect,
            // then swap out the Vpb.  We must preserve the REMOVE_PENDING flag
            // so that the device is not remounted in the middle of a PnP remove
            // operation.
            NT_ASSERT( ((*Vcb).SwapVpb != core::ptr::null_mut()) as i32 );

            let swap_vpb = (*Vcb).SwapVpb;

            (*swap_vpb).Type = (IO_TYPE_VPB) as i16;
            (*swap_vpb).Size = (core::mem::size_of::<VPB>() as u16) as i16;
            (*swap_vpb).RealDevice = (*OldVpb).RealDevice;
            (*(*swap_vpb).RealDevice).Vpb = swap_vpb;
            (*swap_vpb).Flags = FlagOn(((*OldVpb).Flags) as u32, VPB_REMOVE_PENDING) as u16;

            // If we are working on a mount request, we need to make sure we update
            // the VPB in the IRP, since the one it points to may no longer be valid.
            if (*IrpContext).MajorFunction == (IRP_MJ_FILE_SYSTEM_CONTROL) as u8
                && (*IrpContext).MinorFunction == (IRP_MN_MOUNT_VOLUME) as u8
            {
                // Get the IRP stack.
                IrpSp = IoGetCurrentIrpStackLocation((*IrpContext).OriginatingIrp);

                NT_ASSERT( ((*IrpSp).FileObject).is_null() as i32 );

                // Check the VPB in the IRP to see if it is the one we are swapping.
                if (*IrpSp).Parameters.MountVolume.Vpb == OldVpb {
                    // Change the IRP to point to the swap VPB.
                    (*IrpSp).Parameters.MountVolume.Vpb = swap_vpb;
                }
            }

            // We place the volume in the Bad state (as opposed to NotMounted) so
            // that it is not eligible for a remount.  Also indicate we used up
            // the swap.
            (*Vcb).SwapVpb = core::ptr::null_mut();
            FatSetVcbCondition(Vcb, VcbBad);
            SetFlag(
                core::ptr::addr_of_mut!((*Vcb).VcbState),
                VCB_STATE_FLAG_VPB_MUST_BE_FREED,
            );

            Result = 1; // TRUE
        }

        Result
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatCheckForDismount(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
    Force: BOOLEAN,
) -> BOOLEAN {
    let mut SavedIrql: KIRQL = 0;
    let mut VcbDeleted: BOOLEAN = 0;

    // If the VCB condition is good and we are not forcing, just return.
    if (*Vcb).VcbCondition == VcbGood && Force == 0 {
        return 0;
    }

    // Acquire VPB spinlock to check reference counts.
    IoAcquireVpbSpinLock(&mut SavedIrql as *mut KIRQL);

    if (*(*Vcb).Vpb).ReferenceCount == (*Vcb).ResidualOpenCount && (*Vcb).OpenFileCount == 0 {
        let Vpb: *mut VPB = (*Vcb).Vpb;

        #[cfg(debug_assertions)]
        {
            let mut VolumeLabel: _UNICODE_STRING = core::mem::zeroed();
            VolumeLabel.Length = (*Vpb).VolumeLabelLength;
            VolumeLabel.MaximumLength = MAXIMUM_VOLUME_LABEL_LENGTH;
            VolumeLabel.Buffer = &mut (*Vpb).VolumeLabel[0] as *mut u16;

            KdPrintEx(
                DPFLTR_FASTFAT_ID,
                DPFLTR_INFO_LEVEL,
                b"FASTFAT: Dismounting Volume %Z\n\0".as_ptr() as *const i8,
                &VolumeLabel as *const _UNICODE_STRING,
            );
        }

        // Swap this VCB's VPB.
        FatSwapVpb(IrpContext, Vcb);

        // Clear the VPB_MOUNTED bit.
        ClearFlag((&mut (*Vpb).Flags as *mut u16) as *mut u32, VPB_MOUNTED);

        // Clear VPB_LOCKED and VPB_DIRECT_WRITES_ALLOWED.
        ClearFlag(
            (&mut (*Vpb).Flags as *mut u16) as *mut u32,
            VPB_LOCKED | VPB_DIRECT_WRITES_ALLOWED,
        );

        IoReleaseVpbSpinLock(SavedIrql);

        // Mark dismount in progress.
        NT_ASSERT(((FlagOn(
            ((*Vcb).VcbState as u8) as u32,
            VCB_STATE_FLAG_DISMOUNT_IN_PROGRESS,
        ) == 0) as BOOLEAN) as i32);
        SetFlag(
            core::ptr::addr_of_mut!((*Vcb).VcbState),
            VCB_STATE_FLAG_DISMOUNT_IN_PROGRESS,
        );

        // Close down internal opens.
        FatTearDownVcb(IrpContext, Vcb);

        // Process delayed closes.
        FatFspClose(Vcb);

        // Reacquire spinlock to recheck counts.
        IoAcquireVpbSpinLock(&mut SavedIrql as *mut KIRQL);

        if (*(*Vcb).Vpb).ReferenceCount == 0 && (*Vcb).InternalOpenCount == 0 {
            (*Vpb).DeviceObject = core::ptr::null_mut();
            IoReleaseVpbSpinLock(SavedIrql);

            FatDeleteVcb(IrpContext, Vcb);

            // Compute CONTAINING_RECORD: address of VOLUME_DEVICE_OBJECT containing Vcb.
            let device_object = (Vcb as *mut u8).offset(-(core::mem::offset_of!(VOLUME_DEVICE_OBJECT, Vcb) as isize))
                as *mut DEVICE_OBJECT;
            IoDeleteDevice(device_object);

            VcbDeleted = 1;
        } else {
            IoReleaseVpbSpinLock(SavedIrql);

            NT_ASSERT((FlagOn(
                ((*Vcb).VcbState as u8) as u32,
                VCB_STATE_FLAG_DISMOUNT_IN_PROGRESS,
            ) as BOOLEAN) as i32);
            ClearFlag(
                &mut (*Vcb).VcbState as *mut u32,
                VCB_STATE_FLAG_DISMOUNT_IN_PROGRESS,
            );
        }
    } else if Force != 0 {
        // Forcing the dismount; swap the VPB.
        FatSwapVpb(IrpContext, Vcb);
        IoReleaseVpbSpinLock(SavedIrql);
    } else {
        // Just drop the spinlock.
        IoReleaseVpbSpinLock(SavedIrql);
    }

    VcbDeleted
}

#[no_mangle]
pub unsafe extern "system" fn FatConstructNamesInFcb(
    IrpContext: *mut IRP_CONTEXT,
    Fcb: *mut FCB,
    Dirent: *mut DIRENT,
    Lfn: *const _UNICODE_STRING,
) {
    let mut i: usize = 0;
    let mut Unicode: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: null_mut(),
    };
    let ShortName: *mut OEM_STRING = (*Fcb).ShortName.Name.Oem.as_mut() as *mut OEM_STRING;
    let mut LongOemName: *mut OEM_STRING = null_mut();
    let mut LongUniName: *mut _UNICODE_STRING = null_mut();

    PAGED_CODE!();

    NT_ASSERT((((*ShortName).Buffer).is_null() as u8) as i32);

    // _SEH2_TRY / _SEH2_FINALLY
    crate::seh::try_finally(&mut (), |_| {
        // Short name case flags
        if ((*Dirent).NtByte & (FAT_DIRENT_NT_BYTE_8_LOWER_CASE) as u8) != 0 {
            (*Fcb).FcbState |= FCB_STATE_8_LOWER_CASE;
        } else {
            (*Fcb).FcbState &= !FCB_STATE_8_LOWER_CASE;
        }
        if ((*Dirent).NtByte & (FAT_DIRENT_NT_BYTE_3_LOWER_CASE) as u8) != 0 {
            (*Fcb).FcbState |= FCB_STATE_3_LOWER_CASE;
        } else {
            (*Fcb).FcbState &= !FCB_STATE_3_LOWER_CASE;
        }

        (*ShortName).MaximumLength = 16;
        (*ShortName).Buffer = (FsRtlAllocatePoolWithTag(PagedPool, 16, TAG_FILENAME_BUFFER)
            as *mut u8).cast();

        Fat8dot3ToString(IrpContext, Dirent, 0u8, ShortName);

        (*Fcb).ShortName.FileNameDos = 1;

        NT_ASSERT((((*Fcb).ExactCaseLongName.Buffer).is_null() as u8) as i32);

        if Lfn.is_null() || (*Lfn).Length == 0 {
            let size = RtlOemStringToCountedUnicodeSize(ShortName) as usize;
            (*Fcb).FinalNameLength = size as u16;
            (*Fcb).ExactCaseLongName.Length = 0;
            (*Fcb).ExactCaseLongName.MaximumLength = 0;
            return; // try_return( NOTHING )
        }

        NT_ASSERT((((*Fcb).FullFileName.Buffer).is_null() as u8) as i32);

        (*Fcb).FinalNameLength = (*Lfn).Length;
        (*Fcb).ExactCaseLongName.Length = (*Lfn).Length;
        (*Fcb).ExactCaseLongName.MaximumLength = (*Lfn).Length;
        (*Fcb).ExactCaseLongName.Buffer =
            (FsRtlAllocatePoolWithTag(PagedPool, (*Lfn).Length as u32, TAG_FILENAME_BUFFER)
                as *mut u8).cast();
        copy_nonoverlapping(
            (*Lfn).Buffer as *const u8,
            (*Fcb).ExactCaseLongName.Buffer as *mut u8,
            (*Lfn).Length as usize,
        );

        let chars = ((*Lfn).Length as usize) / core::mem::size_of::<u16>();
        i = 0;
        while i < chars {
            let c = *(*Lfn).Buffer.add(i);
            if c >= 0x80 {
                break;
            }
            i += 1;
        }

        if i == chars {
            // No extended characters – use OEM fast path
            {
                let oem_node = (*Fcb).LongName.Oem.as_mut() as *mut FILE_NAME_NODE;
                LongOemName = (*oem_node).Name.Oem.as_mut() as *mut OEM_STRING;
            }
            (*LongOemName).Buffer =
                (FsRtlAllocatePoolWithTag(PagedPool, chars as u32, TAG_FILENAME_BUFFER)
                    as *mut u8).cast();
            (*LongOemName).Length = chars as u16;
            (*LongOemName).MaximumLength = chars as u16;

            i = 0;
            while i < chars {
                let c = *(*Lfn).Buffer.add(i);
                let byte = if (c as u32) < ('a' as u32) {
                    c as u8
                } else if (c as u32) <= ('z' as u32) {
                    (c as u32 - ('a' as u32 - 'A' as u32)) as u8
                } else {
                    c as u8
                };
                *(*LongOemName).Buffer.add(i) = (byte) as i8;
                i += 1;
            }

            let equal = FatAreNamesEqual(IrpContext, *ShortName, *LongOemName) != 0;
            let parent_dcb_inner = (*(*Fcb).ParentDcb).Specific.Dcb.as_mut() as *mut _FCB__bindgen_ty_1__bindgen_ty_1;
            let found = !FatFindFcb(
                IrpContext,
                core::ptr::addr_of_mut!((*parent_dcb_inner).RootOemNode).cast(),
                LongOemName as *const OEM_STRING,
                null_mut(),
            )
            .is_null();

            if equal || found {
                ExFreePool((*LongOemName).Buffer as *mut core::ffi::c_void);
                (*LongOemName).Buffer = null_mut();
                (*LongOemName).Length = 0;
                (*LongOemName).MaximumLength = 0;
            } else {
                (*Fcb).FcbState |= FCB_STATE_HAS_OEM_LONG_NAME;
            }

            return; // try_return( NOTHING )
        }

        // Unicode path – extended characters present
        Unicode.Length = (*Lfn).Length;
        Unicode.MaximumLength = (*Lfn).Length;
        Unicode.Buffer = FsRtlAllocatePoolWithTag(
            PagedPool,
            (*Lfn).Length as u32,
            TAG_FILENAME_BUFFER,
        ) as *mut u16;
        copy_nonoverlapping(
            (*Lfn).Buffer as *const u8,
            Unicode.Buffer as *mut u8,
            (*Lfn).Length as usize,
        );

        RtlDowncaseUnicodeString(&mut Unicode as *mut _UNICODE_STRING, &Unicode as *const _UNICODE_STRING, 0u8);
        RtlUpcaseUnicodeString(&mut Unicode as *mut _UNICODE_STRING, &Unicode as *const _UNICODE_STRING, 0u8);

        {
            let uni_node = (*Fcb).LongName.Unicode.as_mut() as *mut FILE_NAME_NODE;
            LongUniName = (*uni_node).Name.Unicode.as_mut() as *mut _UNICODE_STRING;
        }
        (*LongUniName).Length = Unicode.Length;
        (*LongUniName).MaximumLength = Unicode.MaximumLength;
        (*LongUniName).Buffer = Unicode.Buffer;

        (*Fcb).FcbState |= FCB_STATE_HAS_UNICODE_LONG_NAME;
    }, |_, abnormal| {
        if abnormal {
            if !(*ShortName).Buffer.is_null() {
                ExFreePool((*ShortName).Buffer as *mut core::ffi::c_void);
                (*ShortName).Buffer = null_mut();
            }
        } else {
            let parent_dcb_inner = (*(*Fcb).ParentDcb).Specific.Dcb.as_mut() as *mut _FCB__bindgen_ty_1__bindgen_ty_1;
            FatInsertName(
                IrpContext,
                core::ptr::addr_of_mut!((*parent_dcb_inner).RootOemNode).cast(),
                core::ptr::addr_of_mut!((*Fcb).ShortName),
            );
            (*Fcb).ShortName.Fcb = Fcb;

            if ((*Fcb).FcbState & FCB_STATE_HAS_OEM_LONG_NAME) != 0 {
                let long_oem_node = (*Fcb).LongName.Oem.as_mut() as *mut FILE_NAME_NODE;
                FatInsertName(
                    IrpContext,
                    core::ptr::addr_of_mut!((*parent_dcb_inner).RootOemNode).cast(),
                    long_oem_node,
                );
                (*long_oem_node).Fcb = Fcb;
            }

            if ((*Fcb).FcbState & FCB_STATE_HAS_UNICODE_LONG_NAME) != 0 {
                let long_uni_node = (*Fcb).LongName.Unicode.as_mut() as *mut FILE_NAME_NODE;
                FatInsertName(
                    IrpContext,
                    core::ptr::addr_of_mut!((*parent_dcb_inner).RootUnicodeNode).cast(),
                    long_uni_node,
                );
                (*long_uni_node).Fcb = Fcb;
            }

            (*Fcb).FcbState |= FCB_STATE_NAMES_IN_SPLAY_TREE;
        }
    });
}

#[no_mangle]
pub unsafe extern "system" fn FatCheckFreeDirentBitmap(
    IrpContext: *mut IRP_CONTEXT,
    Dcb: *mut DCB,
) {
    // Gap function declarations
    unsafe extern "system" {
        fn UNREFERENCED_PARAMETER(param: *mut core::ffi::c_void);
        fn FatAcquireDirectoryFileMutex(Vcb: *mut VCB);
        fn FatReleaseDirectoryFileMutex(Vcb: *mut VCB);
        // External constants
        static DCB_UNION_SLACK_SPACE: u32;
        static TAG_DIRENT_BITMAP: u32;
        static FCB_LOOKUP_ALLOCATIONSIZE_HINT: i64;
    }

    PAGED_CODE!();
    // Macro UNREFERENCED_PARAMETER(IrpContext)
    let _ = IrpContext;

    // NT_ASSERT
    if (*Dcb).Header.AllocationSize.QuadPart == FCB_LOOKUP_ALLOCATIONSIZE_HINT {
        RtlAssert(
            b"Dcb->Header.AllocationSize.QuadPart != FCB_LOOKUP_ALLOCATIONSIZE_HINT\0".as_ptr(),
            b"strucsup.rs\0".as_ptr(),
            line!(),
            core::ptr::null(),
        );
    }

    let dir_ent_size = core::mem::size_of::<DIRENT>() as u32;
    let dcb_inner0 = (*Dcb).Specific.Dcb.as_ref() as *const _FCB__bindgen_ty_1__bindgen_ty_1;
    let old_number_of_dirents = (*dcb_inner0).FreeDirentBitmap.__bindgen_anon_1.__bindgen_anon_1.as_ref().SizeOfBitMap;
    let new_number_of_dirents = (*Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart / dir_ent_size;

    if new_number_of_dirents > old_number_of_dirents {
        FatAcquireDirectoryFileMutex((*Dcb).Vcb);

        // _SEH2_TRY / _SEH2_FINALLY
        crate::seh::try_finally(&mut (), |_| {
            let old_bitmap_buffer: *mut u32;
            let bitmap_buffer: *mut u32;
            let bytes_in_bitmap_buffer: u32;
            let bytes_in_old_bitmap_buffer: u32;

            // Recheck inside the lock
            let dcb_inner = (*Dcb).Specific.Dcb.as_mut() as *mut _FCB__bindgen_ty_1__bindgen_ty_1;
            let old_number_of_dirents = (*dcb_inner).FreeDirentBitmap.__bindgen_anon_1.__bindgen_anon_1.as_ref().SizeOfBitMap;
            let new_number_of_dirents = (*Dcb).Header.AllocationSize.__bindgen_anon_1.LowPart / dir_ent_size;

            if new_number_of_dirents > old_number_of_dirents {
                // Remember the old bitmap
                old_bitmap_buffer = (*dcb_inner).FreeDirentBitmap.__bindgen_anon_1.__bindgen_anon_1.as_ref().Buffer;

                // Compute sizes
                bytes_in_bitmap_buffer = new_number_of_dirents / 8;
                bytes_in_old_bitmap_buffer = old_number_of_dirents / 8;

                // Choose the buffer for the new bitmap
                let embedded_buf = core::ptr::addr_of_mut!((*dcb_inner).FreeDirentBitmapBuffer[0]);
                if DCB_UNION_SLACK_SPACE >= bytes_in_bitmap_buffer {
                    bitmap_buffer = embedded_buf;
                } else {
                    bitmap_buffer = FsRtlAllocatePoolWithTag(
                        PagedPool,
                        bytes_in_bitmap_buffer,
                        TAG_DIRENT_BITMAP,
                    ) as *mut u32;
                }

                // Copy old data if we had a bitmap and we are not reusing the embedded buffer
                if (old_number_of_dirents != 0) && (bitmap_buffer != embedded_buf)
                {
                    RtlCopyMemory(
                        bitmap_buffer as *mut core::ffi::c_void,
                        old_bitmap_buffer as *const core::ffi::c_void,
                        (bytes_in_old_bitmap_buffer as u32) as usize,
                    );

                    // Free the old buffer if it was dynamically allocated
                    if old_bitmap_buffer != embedded_buf {
                        ExFreePool(old_bitmap_buffer as *mut core::ffi::c_void);
                    }
                }

                // NT_ASSERT: new buffer must be larger
                if !(bytes_in_bitmap_buffer > bytes_in_old_bitmap_buffer) {
                    RtlAssert(
                        b"BytesInBitmapBuffer > BytesInOldBitmapBuffer\0".as_ptr(),
                        b"strucsup.rs\0".as_ptr(),
                        line!(),
                        core::ptr::null(),
                    );
                }

                // Zero the tail of the new buffer
                RtlZeroMemory(
                    (bitmap_buffer as *mut u8).offset(bytes_in_old_bitmap_buffer as isize) as *mut core::ffi::c_void,
                    ((bytes_in_bitmap_buffer - bytes_in_old_bitmap_buffer) as u32) as usize,
                );

                // Initialize the bitmap header
                RtlInitializeBitMap(
                    core::ptr::addr_of_mut!((*dcb_inner).FreeDirentBitmap) as *mut RTL_BITMAP,
                    bitmap_buffer,
                    new_number_of_dirents,
                );
            }
        }, |_, _abnormal| {
            FatReleaseDirectoryFileMutex((*Dcb).Vcb);
        });
    }
}

#[no_mangle]
pub unsafe extern "system" fn FatIsHandleCountZero(
    IrpContext: *mut IRP_CONTEXT,
    Vcb: *mut VCB,
) -> u8 {
    PAGED_CODE!();

    let mut Fcb: *mut FCB;
    Fcb = (*Vcb).RootDcb;

    while !Fcb.is_null() {
        if (*Fcb).UncleanCount != 0 {
            return 0u8;
        }
        Fcb = FatGetNextFcbTopDown(IrpContext, Fcb, (*Vcb).RootDcb);
    }

    return 1u8;
}
#[no_mangle]
pub unsafe extern "system" fn FatAllocateCloseContext(
    Vcb: *mut VCB,
) -> *mut core::ffi::c_void {
    // PAGED_CODE();

    // UNREFERENCED_PARAMETER( Vcb );

    #[cfg(debug_assertions)]
    {
        if !Vcb.is_null() {
            ntkrnl_sys::ASSERT!(!(0 == (*Vcb).CloseContextCount)); // NT_ASSERT( 0 != Vcb->CloseContextCount)
            InterlockedDecrement(
                core::ptr::addr_of_mut!((*Vcb).CloseContextCount) as *mut i32
            );
        }
    }

    let result = ExInterlockedPopEntrySList(
        core::ptr::addr_of_mut!(FatCloseContextSList) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(FatData)).GeneralSpinLock)
            as *mut core::ffi::c_void,
    );

    result as *mut core::ffi::c_void
}
#[no_mangle]
pub unsafe extern "system" fn FatPreallocateCloseContext(
    Vcb: *mut VCB,
) {
    // PAGED_CODE();
    _ = Vcb; // UNREFERENCED_PARAMETER

    let mut CloseContext: *mut CLOSE_CONTEXT;

    CloseContext = FsRtlAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<CLOSE_CONTEXT>() as u32,
        // nodetype.h TAG_FAT_CLOSE_CONTEXT ('xtaF'). The translation had 0, which is
        // PoolHitTag's default, so every close-context allocation hit the pool
        // tracker's DbgBreakPoint.
        u32::from_le_bytes(*b"Fatx"),
    ) as *mut CLOSE_CONTEXT;

    ExInterlockedPushEntrySList(
        &raw mut FatCloseContextSList as *mut core::ffi::c_void,
        CloseContext as *mut core::ffi::c_void,
        &raw mut FatData.GeneralSpinLock as *mut _,
    );

    DbgDoit((InterlockedIncrement(
        &raw mut (*Vcb).CloseContextCount as *mut i32,
    ) as u32) as i32);
}

#[no_mangle]
pub unsafe extern "system" fn FatEnsureStringBufferEnough(
    String: *mut core::ffi::c_void,
    DesiredBufferSize: u16,
) {
    let LocalString: *mut STRING = String as *mut STRING;

    // PAGED_CODE();

    if (*LocalString).MaximumLength < DesiredBufferSize {
        FatFreeStringBuffer(LocalString as *mut core::ffi::c_void);

        (*LocalString).Buffer = (FsRtlAllocatePoolWithTag(
            ntkrnl_sys::mm::PagedPool,
            DesiredBufferSize as u32,
            TAG_DYNAMIC_NAME_BUFFER,
        ) as *mut u8).cast();

        // NT_ASSERT((*LocalString).Buffer != NULL) — in C an assert is
        // ((void)0) in retail and RtlAssert under DBG; it is NEVER __assume(0).
        // This had been translated to unreachable_unchecked(), i.e. UB, which
        // lets LLVM assume the pointer is non-null and drop null checks on it
        // downstream. FsRtlAllocatePoolWithTag raises on failure so the branch
        // should be unreachable in practice — but "should be" is exactly what
        // an assert exists to catch, so report it rather than assume it.
        if (*LocalString).Buffer.is_null() {
            extern "C" {
                fn DbgPrint(fmt: *const i8, ...) -> u32;
            }
            DbgPrint(
                b"[FASTFAT] FatInitializeStringBuffer: allocation returned NULL (%u bytes)\n\0"
                    .as_ptr() as *const i8,
                DesiredBufferSize as u32,
            );
        }

        (*LocalString).MaximumLength = DesiredBufferSize;
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatScanForDataTrack(
    IrpContext: *mut IRP_CONTEXT,
    TargetDeviceObject: *mut DEVICE_OBJECT,
) -> i32 {

    let mut Status: i32 = 0;
    let mut Iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut LocalTrackCount: u32 = 0;
    let mut LocalTocLength: u32 = 0;
    let CdromToc: *mut crate::CDROM_TOC;
    let mut Result: i32 = 0;

    let pool_ptr = FsRtlAllocatePoolWithTag(
        PagedPool,
        size_of::<crate::CDROM_TOC>() as u32,
        TAG_IO_BUFFER,
    );
    CdromToc = pool_ptr as *mut crate::CDROM_TOC;

    RtlZeroMemory(
        CdromToc as *mut core::ffi::c_void,
        (size_of::<crate::CDROM_TOC>() as u32) as usize,
    );

    // _SEH2_TRY / _SEH2_FINALLY
    let p_Result = core::ptr::addr_of_mut!(Result);
    crate::seh::try_finally(&mut (), |_| {
        Status = FatPerformDevIoCtrl(
            IrpContext,
            IOCTL_CDROM_READ_TOC,
            TargetDeviceObject,
            core::ptr::null_mut(),
            0,
            CdromToc as *mut core::ffi::c_void,
            size_of::<crate::CDROM_TOC>() as u32,
            0,
            1,
            &mut Iosb,
        );

        if Status != STATUS_SUCCESS {
            if Status == STATUS_IO_DEVICE_ERROR || Status == STATUS_INVALID_DEVICE_REQUEST {
                *p_Result = 1;
            }
            return; // try_leave( NOTHING )
        }

        // C promotes the UCHARs to int: a bogus TOC (Last < First) must not overflow-panic.
        LocalTrackCount = (((*CdromToc).LastTrack as i32 - (*CdromToc).FirstTrack as i32) + 1) as u32;

        let track_data_ptr = ptr::addr_of_mut!((*CdromToc).TrackData) as *mut crate::CDROM_TOC_TRACK_DATA;
        let field_addr = track_data_ptr.add((LocalTrackCount + 1) as usize) as *const u8;
        let base_addr = CdromToc as *const u8;
        LocalTocLength = (field_addr as usize - base_addr as usize) as u32;

        if (LocalTocLength > Iosb.Information as u32)
            || ((*CdromToc).FirstTrack > (*CdromToc).LastTrack)
            || (LocalTrackCount != 1)
        {
            return; // try_leave( NOTHING )
        }

        *p_Result = if BooleanFlagOn((*CdromToc).TrackData[0].Control as u32, 0x04u32) != 0 { 1 } else { 0 };
    }, |_, _abnormal| {
        ExFreePool(CdromToc as *mut core::ffi::c_void);
    });

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
//     This module implements the Fat in-memory data structure manipulation
//     routines
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
// #define BugCheckFileId                   (FAT_BUG_CHECK_STRUCSUP)
//
// //
// //  The debug trace level
// //
//
// #define Dbg                              (DEBUG_TRACE_STRUCSUP)
//
// #define FillMemory(BUF,SIZ,MASK) {                          \
//     ULONG i;                                                \
//     for (i = 0; i < (((SIZ)/4) - 1); i += 2) {              \
//         ((PULONG)(BUF))[i] = (MASK);                        \
//         ((PULONG)(BUF))[i+1] = (ULONG)PsGetCurrentThread(); \
//     }                                                       \
// }
//
// #define IRP_CONTEXT_HEADER (sizeof( IRP_CONTEXT ) * 0x10000 + FAT_NTC_IRP_CONTEXT)
//
// //
// //  Local macros.
// //
// //  Define our lookaside list allocators.  For the time being, and perhaps
// //  permanently, the paged structures don't come off of lookasides.  This
// //  is due to complications with clean unload as FAT can be in the paging
// //  path, making it really hard to find the right time to empty them.
// //
// //  Fortunately, the hit rates on the Fcb/Ccb lists weren't stunning.
// //
//
// #define FAT_FILL_FREE 0
//
// #ifdef __REACTOS__
// static
// #endif
// INLINE
// PCCB
// FatAllocateCcb (
//     )
// {
//     return (PCCB) FsRtlAllocatePoolWithTag( PagedPool, sizeof(CCB), TAG_CCB );
// }
//
// #ifdef __REACTOS__
// static
// #endif
// INLINE
// VOID
// FatFreeCcb (
//     IN PCCB Ccb
//     )
// {
// #if FAT_FILL_FREE
//     RtlFillMemoryUlong(Ccb, sizeof(CCB), FAT_FILL_FREE);
// #endif
//
//     ExFreePool( Ccb );
// }
//
// #ifdef __REACTOS__
// static
// #endif
// INLINE
// PFCB
// FatAllocateFcb (
//     )
// {
//     return (PFCB) FsRtlAllocatePoolWithTag( PagedPool, sizeof(FCB), TAG_FCB );
// }
//
// #ifdef __REACTOS__
// static
// #endif
// INLINE
// VOID
// FatFreeFcb (
//     IN PFCB Fcb
//     )
// {
// #if FAT_FILL_FREE
//     RtlFillMemoryUlong(Fcb, sizeof(FCB), FAT_FILL_FREE);
// #endif
//
//     ExFreePool( Fcb );
// }
//
// #ifdef __REACTOS__
// static
// #endif
// INLINE
// PNON_PAGED_FCB
// FatAllocateNonPagedFcb (
//     )
// {
//     return (PNON_PAGED_FCB) ExAllocateFromNPagedLookasideList( &FatNonPagedFcbLookasideList );
// }
//
// #ifdef __REACTOS__
// static
// #endif
// INLINE
// VOID
// FatFreeNonPagedFcb (
//     PNON_PAGED_FCB NonPagedFcb
//     )
// {
// #if FAT_FILL_FREE
//     RtlFillMemoryUlong(NonPagedFcb, sizeof(NON_PAGED_FCB), FAT_FILL_FREE);
// #endif
//
//     ExFreeToNPagedLookasideList( &FatNonPagedFcbLookasideList, (PVOID) NonPagedFcb );
// }
//
// #ifdef __REACTOS__
// static
// #endif
// INLINE
// PERESOURCE
// FatAllocateResource (
//     )
// {
//     PERESOURCE Resource;
//
//     Resource = (PERESOURCE) ExAllocateFromNPagedLookasideList( &FatEResourceLookasideList );
//
//     ExInitializeResourceLite( Resource );
//
//     return Resource;
// }
//
// #ifdef __REACTOS__
// static
// #endif
// INLINE
// VOID
// FatFreeResource (
//     IN PERESOURCE Resource
//     )
// {
//     ExDeleteResourceLite( Resource );
//
// #if FAT_FILL_FREE
//     RtlFillMemoryUlong(Resource, sizeof(ERESOURCE), FAT_FILL_FREE);
// #endif
//
//     ExFreeToNPagedLookasideList( &FatEResourceLookasideList, (PVOID) Resource );
// }
//
// #ifdef __REACTOS__
// static
// #endif
// INLINE
// PIRP_CONTEXT
// FatAllocateIrpContext (
//     )
// {
//     return (PIRP_CONTEXT) ExAllocateFromNPagedLookasideList( &FatIrpContextLookasideList );
// }
//
// #ifdef __REACTOS__
// static
// #endif
// INLINE
// VOID
// FatFreeIrpContext (
//     IN PIRP_CONTEXT IrpContext
//     )
// {
// #if FAT_FILL_FREE
//     RtlFillMemoryUlong(IrpContext, sizeof(IRP_CONTEXT), FAT_FILL_FREE);
// #endif
//
//     ExFreeToNPagedLookasideList( &FatIrpContextLookasideList, (PVOID) IrpContext );
// }
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, FatInitializeVcb)
// #pragma alloc_text(PAGE, FatTearDownVcb)
// #pragma alloc_text(PAGE, FatDeleteVcb)
// #pragma alloc_text(PAGE, FatCreateRootDcb)
// #pragma alloc_text(PAGE, FatCreateFcb)
// #pragma alloc_text(PAGE, FatCreateDcb)
// #pragma alloc_text(PAGE, FatDeleteFcb)
// #pragma alloc_text(PAGE, FatCreateCcb)
// #pragma alloc_text(PAGE, FatDeallocateCcbStrings)
// #pragma alloc_text(PAGE, FatDeleteCcb)
// #pragma alloc_text(PAGE, FatGetNextFcbTopDown)
// #pragma alloc_text(PAGE, FatGetNextFcbBottomUp)
// #pragma alloc_text(PAGE, FatConstructNamesInFcb)
// #pragma alloc_text(PAGE, FatCheckFreeDirentBitmap)
// #pragma alloc_text(PAGE, FatCreateIrpContext)
// #pragma alloc_text(PAGE, FatDeleteIrpContext_Real)
// #pragma alloc_text(PAGE, FatIsHandleCountZero)
// #pragma alloc_text(PAGE, FatAllocateCloseContext)
// #pragma alloc_text(PAGE, FatPreallocateCloseContext)
// #pragma alloc_text(PAGE, FatEnsureStringBufferEnough)
// #pragma alloc_text(PAGE, FatFreeStringBuffer)
// #pragma alloc_text(PAGE, FatScanForDataTrack)
// #endif
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatInitializeVcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN OUT PVCB Vcb,
//     IN PDEVICE_OBJECT TargetDeviceObject,
//     IN PVPB Vpb,
//     IN PDEVICE_OBJECT FsDeviceObject
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
//     FsDeviceObject - The filesystem device object that the mount was directed
//                      too.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     CC_FILE_SIZES FileSizes;
//     PDEVICE_OBJECT RealDevice;
//     ULONG i;
//
//     STORAGE_HOTPLUG_INFO HotplugInfo;
//     STORAGE_DEVICE_NUMBER StorDeviceNumber;
//     NTSTATUS Status;
//
//     //
//     //  The following variables are used for abnormal unwind
//     //
//
//     PLIST_ENTRY UnwindEntryList = NULL;
//     PERESOURCE UnwindResource = NULL;
//     PERESOURCE UnwindResource2 = NULL;
//     PFILE_OBJECT UnwindFileObject = NULL;
//     PFILE_OBJECT UnwindCacheMap = NULL;
//     BOOLEAN UnwindWeAllocatedMcb = FALSE;
//     PFILE_SYSTEM_STATISTICS UnwindStatistics = NULL;
//     BOOLEAN UnwindWeAllocatedBadBlockMap = FALSE;
//     BOOLEAN CloseContextAllocated = FALSE;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( FsDeviceObject );
//
//     DebugTrace(+1, Dbg, "FatInitializeVcb, Vcb = %p\n", Vcb);
//
//     _SEH2_TRY {
//
//         //
//         //  We start by first zeroing out all of the VCB, this will guarantee
//         //  that any stale data is wiped clean
//         //
//
//         RtlZeroMemory( Vcb, sizeof(VCB) );
//
//         //
//         //  Set the proper node type code and node byte size
//         //
//
//         Vcb->VolumeFileHeader.NodeTypeCode = FAT_NTC_VCB;
//         Vcb->VolumeFileHeader.NodeByteSize = sizeof(VCB);
//
//         //
//         //  Initialize the tunneling cache
//         //
//
//         FsRtlInitializeTunnelCache(&Vcb->Tunnel);
//
//         //
//         //  Insert this Vcb record on the FatData.VcbQueue
//         //
//
//         NT_ASSERT( FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) );
//
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28137, "prefast wants the wait to be a constant, but that isn't possible for the way fastfat is designed" )
// #pragma prefast( disable: 28193, "this will always wait" )
// #endif
//
//         (VOID)FatAcquireExclusiveGlobal( IrpContext );
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//         InsertTailList( &FatData.VcbQueue, &Vcb->VcbLinks );
//         FatReleaseGlobal( IrpContext );
//         UnwindEntryList = &Vcb->VcbLinks;
//
//         //
//         //  Set the Target Device Object, Vpb, and Vcb State fields
//         //
//
//
//         ObReferenceObject( TargetDeviceObject );
//         Vcb->TargetDeviceObject = TargetDeviceObject;
//         Vcb->Vpb = Vpb;
//
//         Vcb->CurrentDevice = Vpb->RealDevice;
//
//         //
//         //  Set the removable media and defflush flags based on the storage
//         //  inquiry and the old characteristic bits.
//         //
//
//         Status = FatPerformDevIoCtrl( IrpContext,
//                                       IOCTL_STORAGE_GET_HOTPLUG_INFO,
//                                       TargetDeviceObject,
//                                       NULL,
//                                       0,
//                                       &HotplugInfo,
//                                       sizeof(HotplugInfo),
//                                       FALSE,
//                                       TRUE,
//                                       NULL );
//
//         if (NT_SUCCESS( Status )) {
//
//             if (HotplugInfo.MediaRemovable) {
//
//                 SetFlag( Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA );
//             }
//
//             //
//             //  If the media or device is hot-pluggable, then set this flag.
//             //
//
//             if (HotplugInfo.MediaHotplug || HotplugInfo.DeviceHotplug) {
//
//                 SetFlag( Vcb->VcbState, VCB_STATE_FLAG_HOTPLUGGABLE );
//             }
//
//             if (!HotplugInfo.WriteCacheEnableOverride) {
//
//                 //
//                 //  If the device or media is hotplug and the override is not
//                 //  set, force defflush behavior for the device.
//                 //
//
//                 if (HotplugInfo.MediaHotplug || HotplugInfo.DeviceHotplug) {
//
//                     NT_ASSERT( FlagOn( Vcb->VcbState, VCB_STATE_FLAG_HOTPLUGGABLE ));
//
//                     SetFlag( Vcb->VcbState, VCB_STATE_FLAG_DEFERRED_FLUSH );
//
//                 //
//                 //  Now, for removables that claim to be lockable, lob a lock
//                 //  request and see if it works.  There can unfortunately be
//                 //  transient, media dependent reasons that it can fail.  If
//                 //  it does not, we must force defflush on.
//                 //
//
//                 } else if (HotplugInfo.MediaRemovable &&
//                            !HotplugInfo.MediaHotplug) {
//
//                     Status = FatToggleMediaEjectDisable( IrpContext, Vcb, TRUE );
//
//                     if (!NT_SUCCESS( Status )) {
//
//                         SetFlag( Vcb->VcbState, VCB_STATE_FLAG_DEFERRED_FLUSH );
//
//                     }
//
//                     (VOID)FatToggleMediaEjectDisable( IrpContext, Vcb, FALSE );
//                 }
//             }
//         }
//
//         if (FlagOn(Vpb->RealDevice->Characteristics, FILE_REMOVABLE_MEDIA)) {
//
//             SetFlag( Vcb->VcbState, VCB_STATE_FLAG_REMOVABLE_MEDIA );
//         }
//
//         //
//         //  Make sure we turn on deferred flushing for floppies like we always
//         //  have.
//         //
//
//         if (FlagOn(Vpb->RealDevice->Characteristics, FILE_FLOPPY_DISKETTE)) {
//
//             SetFlag( Vcb->VcbState, VCB_STATE_FLAG_DEFERRED_FLUSH );
//         }
//
//         //
//         //  Query the storage device number.
//         //
//
//         Status = FatPerformDevIoCtrl( IrpContext,
//                                       IOCTL_STORAGE_GET_DEVICE_NUMBER,
//                                       TargetDeviceObject,
//                                       NULL,
//                                       0,
//                                       &StorDeviceNumber,
//                                       sizeof(StorDeviceNumber),
//                                       FALSE,
//                                       TRUE,
//                                       NULL );
//
//         if (NT_SUCCESS( Status )) {
//
//             Vcb->DeviceNumber = StorDeviceNumber.DeviceNumber;
//
//         } else {
//
//             Vcb->DeviceNumber = (ULONG)(-1);
//         }
//
//         FatSetVcbCondition( Vcb, VcbGood);
//
//         //
//         //  Initialize the resource variable for the Vcb
//         //
//
//         ExInitializeResourceLite( &Vcb->Resource );
//         UnwindResource = &Vcb->Resource;
//
//         ExInitializeResourceLite( &Vcb->ChangeBitMapResource );
//         UnwindResource2 = &Vcb->ChangeBitMapResource;
//
//         //
//         //  Initialize the free cluster bitmap mutex.
//         //
//
//         ExInitializeFastMutex( &Vcb->FreeClusterBitMapMutex );
//
//         //
//         //  Create the special file object for the virtual volume file with a close
//         //  context, its pointers back to the Vcb and the section object pointer.
//         //
//         //  We don't have to unwind the close context.  That will happen in the close
//         //  path automatically.
//         //
//
//         RealDevice = Vcb->CurrentDevice;
//
//         FatPreallocateCloseContext(Vcb);
//         CloseContextAllocated = TRUE;
//
//         Vcb->VirtualVolumeFile = UnwindFileObject = IoCreateStreamFileObject( NULL, RealDevice );
//
//         FatSetFileObject( Vcb->VirtualVolumeFile,
//                           VirtualVolumeFile,
//                           Vcb,
//                           NULL );
//
//         //
//         //  Remember this internal, residual open.
//         //
//
//         InterlockedIncrement( (LONG*)&(Vcb->InternalOpenCount) );
//         InterlockedIncrement( (LONG*)&(Vcb->ResidualOpenCount) );
//
//         Vcb->VirtualVolumeFile->SectionObjectPointer = &Vcb->SectionObjectPointers;
//
//         Vcb->VirtualVolumeFile->ReadAccess = TRUE;
//         Vcb->VirtualVolumeFile->WriteAccess = TRUE;
//         Vcb->VirtualVolumeFile->DeleteAccess = TRUE;
//
//         //
//         //  Initialize the notify structures.
//         //
//
//         InitializeListHead( &Vcb->DirNotifyList );
//
//         FsRtlNotifyInitializeSync( &Vcb->NotifySync );
//
//         //
//         //  Initialize the Cache Map for the volume file.  The size is
//         //  initially set to that of our first read.  It will be extended
//         //  when we know how big the Fat is.
//         //
//
//         FileSizes.AllocationSize.QuadPart =
//         FileSizes.FileSize.QuadPart = sizeof(PACKED_BOOT_SECTOR);
//         FileSizes.ValidDataLength = FatMaxLarge;
//
//         FatInitializeCacheMap( Vcb->VirtualVolumeFile,
//                                &FileSizes,
//                                TRUE,
//                                &FatData.CacheManagerNoOpCallbacks,
//                                Vcb );
//
//         UnwindCacheMap = Vcb->VirtualVolumeFile;
//
//         //
//         //  Initialize the structure that will keep track of dirty fat sectors.
//         //  The largest possible Mcb structures are less than 1K, so we use
//         //  non paged pool.
//         //
//
//         FsRtlInitializeLargeMcb( &Vcb->DirtyFatMcb, PagedPool );
//
//         UnwindWeAllocatedMcb = TRUE;
//
//         //
//         //  Initialize the structure that will keep track of bad clusters on the volume.
//         //
//         //  It will be empty until it is populated by FSCTL_GET_RETRIEVAL_POINTERS with a volume handle.
//         //
//
//         FsRtlInitializeLargeMcb( &Vcb->BadBlockMcb, PagedPool );
//         UnwindWeAllocatedBadBlockMap = TRUE;
//
//         //
//         //  Set the cluster index hint to the first valid cluster of a fat: 2
//         //
//
//         Vcb->ClusterHint = 2;
//
//         //
//         //  Initialize the directory stream file object creation event.
//         //  This event is also "borrowed" for async non-cached writes.
//         //
//
//         ExInitializeFastMutex( &Vcb->DirectoryFileCreationMutex );
//
//         //
//         //  Initialize the clean volume callback Timer and DPC.
//         //
//
//         KeInitializeTimer( &Vcb->CleanVolumeTimer );
//
//         KeInitializeDpc( &Vcb->CleanVolumeDpc, FatCleanVolumeDpc, Vcb );
//
//         //
//         //  Initialize the performance counters.
//         //
//
//         Vcb->Statistics = FsRtlAllocatePoolWithTag( NonPagedPoolNx,
//                                                     sizeof(FILE_SYSTEM_STATISTICS) * FatData.NumberProcessors,
//                                                     TAG_VCB_STATS );
//         UnwindStatistics = Vcb->Statistics;
//
//         RtlZeroMemory( Vcb->Statistics, sizeof(FILE_SYSTEM_STATISTICS) * FatData.NumberProcessors );
//
//         for (i = 0; i < FatData.NumberProcessors; i += 1) {
//             Vcb->Statistics[i].Common.FileSystemType = FILESYSTEM_STATISTICS_TYPE_FAT;
//             Vcb->Statistics[i].Common.Version = 1;
//             Vcb->Statistics[i].Common.SizeOfCompleteStructure =
//                 sizeof(FILE_SYSTEM_STATISTICS);
//         }
//
//         //
//         //  Pick up a VPB right now so we know we can pull this filesystem stack off
//         //  of the storage stack on demand.
//         //
//
//         Vcb->SwapVpb = FsRtlAllocatePoolWithTag( NonPagedPoolNx,
//                                                  sizeof( VPB ),
//                                                  TAG_VPB );
//
//         RtlZeroMemory( Vcb->SwapVpb, sizeof( VPB ) );
//
//         //
//         //  Initialize the close queue listheads.
//         //
//
//         InitializeListHead( &Vcb->AsyncCloseList );
//         InitializeListHead( &Vcb->DelayedCloseList );
//
//         //
//         //  Initialize the Advanced FCB Header
//         //
//
//         ExInitializeFastMutex( &Vcb->AdvancedFcbHeaderMutex );
//         FsRtlSetupAdvancedHeader( &Vcb->VolumeFileHeader,
//                                   &Vcb->AdvancedFcbHeaderMutex );
//
//
//         //
//         //  With the Vcb now set up, set the IrpContext Vcb field.
//         //
//
//         IrpContext->Vcb = Vcb;
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatInitializeVcb );
//
//         //
//         //  If this is an abnormal termination then undo our work
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             if (UnwindCacheMap != NULL) { FatSyncUninitializeCacheMap( IrpContext, UnwindCacheMap ); }
//             if (UnwindFileObject != NULL) { ObDereferenceObject( UnwindFileObject ); }
//             if (UnwindResource != NULL) { FatDeleteResource( UnwindResource ); }
//             if (UnwindResource2 != NULL) { FatDeleteResource( UnwindResource2 ); }
//             if (UnwindWeAllocatedMcb) { FsRtlUninitializeLargeMcb( &Vcb->DirtyFatMcb ); }
//             if (UnwindWeAllocatedBadBlockMap) { FsRtlUninitializeLargeMcb(&Vcb->BadBlockMcb ); }
//             if (UnwindEntryList != NULL) {
// #ifdef _MSC_VER
// #pragma prefast( suppress: 28137, "prefast wants the wait to be a constant, but that isn't possible for the way fastfat is designed" )
// #endif
//                 (VOID)FatAcquireExclusiveGlobal( IrpContext );
//                 RemoveEntryList( UnwindEntryList );
//                 FatReleaseGlobal( IrpContext );
//             }
//             if (UnwindStatistics != NULL) { ExFreePool( UnwindStatistics ); }
//
//             //
//             // Cleanup the close context we preallocated above.
//             //
//
//             if (CloseContextAllocated && (Vcb->VirtualVolumeFile == NULL)) {
//
//                 //
//                 // FatAllocateCloseContext does not allocate memory, it
//                 // pulls a close context off the preallocated slist queue.
//                 //
//                 // Doing this here is necessary to balance out the one we
//                 // preallocated for the Vcb earlier in this function, but
//                 // only if we failed to create the virtual volume file.
//                 //
//                 // If VirtualVolumeFile is not NULL, then this CloseContext
//                 // will get cleaned up when the close comes in for it during
//                 // Vcb teardown.
//                 //
//
//                 PCLOSE_CONTEXT CloseContext = FatAllocateCloseContext(Vcb);
//
//                 ExFreePool( CloseContext );
//                 CloseContextAllocated = FALSE;
//             }
//         }
//
//         DebugTrace(-1, Dbg, "FatInitializeVcb -> VOID\n", 0);
//     } _SEH2_END;
//
//     //
//     //  and return to our caller
//     //
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return;
// }
//
//
// VOID
// FatTearDownVcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine tries to remove all internal opens from the volume.
//
// Arguments:
//
//     IrpContext - Supplies the context for the overall request.
//
//     Vcb - Supplies the Vcb to be torn down.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PFILE_OBJECT DirectoryFileObject;
//
//
//     PAGED_CODE();
//
//     //
//     //  Get rid of the virtual volume file, if we need to.
//     //
//
//     if (Vcb->VirtualVolumeFile != NULL) {
//
//         //
//         //  Uninitialize the cache
//         //
//
//         CcUninitializeCacheMap( Vcb->VirtualVolumeFile,
//                                 &FatLargeZero,
//                                 NULL );
//
//         FsRtlTeardownPerStreamContexts( &Vcb->VolumeFileHeader );
//
//         ObDereferenceObject( Vcb->VirtualVolumeFile );
//
//         Vcb->VirtualVolumeFile = NULL;
//     }
//
//     //
//     //  Close down the EA file.
//     //
//
//     FatCloseEaFile( IrpContext, Vcb, FALSE );
//
//     //
//     //  Close down the root directory stream..
//     //
//
//     if (Vcb->RootDcb != NULL) {
//
//         DirectoryFileObject = Vcb->RootDcb->Specific.Dcb.DirectoryFile;
//
//         if (DirectoryFileObject != NULL) {
//
//             //
//             //  Tear down this directory file.
//             //
//
//             CcUninitializeCacheMap( DirectoryFileObject,
//                                     &FatLargeZero,
//                                     NULL );
//
//             Vcb->RootDcb->Specific.Dcb.DirectoryFile = NULL;
//             ObDereferenceObject( DirectoryFileObject );
//         }
//     }
//
//     //
//     //  The VCB can no longer be used.
//     //
//
//     FatSetVcbCondition( Vcb, VcbBad );
// }
//
//
// VOID
// FatDeleteVcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine removes the Vcb record from Fat's in-memory data
//     structures.  It also will remove all associated underlings
//     (i.e., FCB records).
//
// Arguments:
//
//     Vcb - Supplies the Vcb to be removed
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PFCB Fcb;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatDeleteVcb, Vcb = %p\n", Vcb);
//
//     //
//     //  If the IrpContext points to the VCB being deleted NULL out the stail
//     //  pointer.
//     //
//
//     if (IrpContext->Vcb == Vcb) {
//
//         IrpContext->Vcb = NULL;
//
//     }
//
//     //
//     //  Chuck the backpocket Vpb we kept just in case.
//     //
//
//     if (Vcb->SwapVpb) {
//
//         ExFreePool( Vcb->SwapVpb );
//
//     }
//
//     //
//     //  Free the VPB, if we need to.
//     //
//
//     if (FlagOn( Vcb->VcbState, VCB_STATE_FLAG_VPB_MUST_BE_FREED )) {
//
//         //
//         //  We swapped the VPB, so we need to free the main one.
//         //
//
//         ExFreePool( Vcb->Vpb );
//     }
//
//     if (Vcb->VolumeGuidPath.Buffer) {
//         ExFreePool( Vcb->VolumeGuidPath.Buffer );
//         Vcb->VolumeGuidPath.Buffer = NULL;
//     }
//
//     //
//     //  Remove this record from the global list of all Vcb records.
//     //  Note that the global lock must already be held when calling
//     //  this function.
//     //
//
//     RemoveEntryList( &(Vcb->VcbLinks) );
//
//     //
//     //  Make sure the direct access open count is zero, and the open file count
//     //  is also zero.
//     //
//
//     if ((Vcb->DirectAccessOpenCount != 0) || (Vcb->OpenFileCount != 0)) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//         FatBugCheck( 0, 0, 0 );
//     }
//
//     //
//     //  Remove the EaFcb and dereference the Fcb for the Ea file if it
//     //  exists.
//     //
//
//     if (Vcb->EaFcb != NULL) {
//
//         Vcb->EaFcb->OpenCount = 0;
//         FatDeleteFcb( IrpContext, &Vcb->EaFcb );
//     }
//
//     //
//     //  Remove the Root Dcb
//     //
//
//     if (Vcb->RootDcb != NULL) {
//
//         //
//         //  Rundown stale child Fcbs that may be hanging around.  Yes, this
//         //  can happen.  No, the create path isn't perfectly defensive about
//         //  tearing down branches built up on creates that don't wind up
//         //  succeeding.  Normal system operation usually winds up having
//         //  cleaned them out through re-visiting, but ...
//         //
//         //  Just pick off Fcbs from the bottom of the tree until we run out.
//         //  Then we delete the root Dcb.
//         //
//
//         while( (Fcb = FatGetNextFcbBottomUp( IrpContext, NULL, Vcb->RootDcb )) != Vcb->RootDcb ) {
//
//             FatDeleteFcb( IrpContext, &Fcb );
//         }
//
//         FatDeleteFcb( IrpContext, &Vcb->RootDcb );
//     }
//
//     //
//     //  Uninitialize the notify sychronization object.
//     //
//
//     FsRtlNotifyUninitializeSync( &Vcb->NotifySync );
//
//     //
//     //  Uninitialize the resource variable for the Vcb
//     //
//
//     FatDeleteResource( &Vcb->Resource );
//     FatDeleteResource( &Vcb->ChangeBitMapResource );
//
//     //
//     //  If allocation support has been setup, free it.
//     //
//
//     if (Vcb->FreeClusterBitMap.Buffer != NULL) {
//
//         FatTearDownAllocationSupport( IrpContext, Vcb );
//     }
//
//     //
//     //  UnInitialize the Mcb structure that kept track of dirty fat sectors.
//     //
//
//     FsRtlUninitializeLargeMcb( &Vcb->DirtyFatMcb );
//
//     //
//     //  Uninitialize the Mcb structure that kept track of bad sectors.
//     //
//
//     FsRtlUninitializeLargeMcb( &Vcb->BadBlockMcb );
//
//     //
//     //  Free the pool for the stached copy of the boot sector
//     //
//
//     if ( Vcb->First0x24BytesOfBootSector ) {
//
//         ExFreePool( Vcb->First0x24BytesOfBootSector );
//         Vcb->First0x24BytesOfBootSector = NULL;
//     }
//
//     //
//     //  Cancel the CleanVolume Timer and Dpc
//     //
//
//     (VOID)KeCancelTimer( &Vcb->CleanVolumeTimer );
//
//     (VOID)KeRemoveQueueDpc( &Vcb->CleanVolumeDpc );
//
//     //
//     //  Free the performance counters memory
//     //
//
//     ExFreePool( Vcb->Statistics );
//
//     //
//     //  Clean out the tunneling cache
//     //
//
//     FsRtlDeleteTunnelCache(&Vcb->Tunnel);
//
//     //
//     // Dereference the target device object.
//     //
//
//     ObDereferenceObject( Vcb->TargetDeviceObject );
//
//     //
//     //  We better have used all the close contexts we allocated. There could be
//     //  one remaining if we're doing teardown due to a final close coming in on
//     //  a directory file stream object.  It will be freed on the way back up.
//     //
//
//     NT_ASSERT( Vcb->CloseContextCount <= 1);
//
//     //
//     //  And zero out the Vcb, this will help ensure that any stale data is
//     //  wiped clean
//     //
//
//     RtlZeroMemory( Vcb, sizeof(VCB) );
//
//     //
//     //  return and tell the caller
//     //
//
//     DebugTrace(-1, Dbg, "FatDeleteVcb -> VOID\n", 0);
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatCreateRootDcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine allocates, initializes, and inserts a new root DCB record
//     into the in memory data structure.
//
// Arguments:
//
//     Vcb - Supplies the Vcb to associate the new DCB under
//
// Return Value:
//
//     None. The Vcb is modified in-place.
//
// --*/
//
// {
//     PDCB Dcb = NULL;
//
//     //
//     //  The following variables are used for abnormal unwind
//     //
//
//     PVOID UnwindStorage[2] = { NULL, NULL };
//     PERESOURCE UnwindResource = NULL;
//     PERESOURCE UnwindResource2 = NULL;
//     PLARGE_MCB UnwindMcb = NULL;
//     PFILE_OBJECT UnwindFileObject = NULL;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatCreateRootDcb, Vcb = %p\n", Vcb);
//
//     _SEH2_TRY {
//
//         //
//         //  Make sure we don't already have a root dcb for this vcb
//         //
//
//         if (Vcb->RootDcb != NULL) {
//
//             DebugDump("Error trying to create multiple root dcbs\n", 0, Vcb);
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//             FatBugCheck( 0, 0, 0 );
//         }
//
//         //
//         //  Allocate a new DCB and zero it out, we use Dcb locally so we don't
//         //  have to continually reference through the Vcb
//         //
//
//         UnwindStorage[0] = Dcb = Vcb->RootDcb = FsRtlAllocatePoolWithTag( NonPagedPoolNx,
//                                                                           sizeof(DCB),
//                                                                           TAG_FCB );
//
//         RtlZeroMemory( Dcb, sizeof(DCB));
//
//         UnwindStorage[1] =
//         Dcb->NonPaged = FatAllocateNonPagedFcb();
//
//         RtlZeroMemory( Dcb->NonPaged, sizeof( NON_PAGED_FCB ) );
//
//         //
//         //  Set the proper node type code, node byte size, and call backs
//         //
//
//         Dcb->Header.NodeTypeCode = FAT_NTC_ROOT_DCB;
//         Dcb->Header.NodeByteSize = sizeof(DCB);
//
//         Dcb->FcbCondition = FcbGood;
//
//         //
//         //  The parent Dcb, initial state, open count, dirent location
//         //  information, and directory change count fields are already zero so
//         //  we can skip setting them
//         //
//
//         //
//         //  Initialize the resource variable
//         //
//
//         UnwindResource =
//         Dcb->Header.Resource = FatAllocateResource();
//
//         //
//         //  Initialize the PagingIo Resource.  We no longer use the FsRtl common
//         //  shared pool because this led to a) deadlocks due to cases where files
//         //  and their parent directories shared a resource and b) there is no way
//         //  to anticipate inter-driver induced deadlock via recursive operation.
//         //
//
//         UnwindResource2 =
//         Dcb->Header.PagingIoResource = FatAllocateResource();
//
//         //
//         //  The root Dcb has an empty parent dcb links field
//         //
//
//         InitializeListHead( &Dcb->ParentDcbLinks );
//
//         //
//         //  Set the Vcb
//         //
//
//         Dcb->Vcb = Vcb;
//
//         //
//         //  initialize the parent dcb queue.
//         //
//
//         InitializeListHead( &Dcb->Specific.Dcb.ParentDcbQueue );
//
//         //
//         //  Set the full file name up.
//         //
//
//         Dcb->FullFileName.Buffer = L"\\";
//         Dcb->FullFileName.Length = (USHORT)2;
//         Dcb->FullFileName.MaximumLength = (USHORT)4;
//
//         Dcb->ShortName.Name.Oem.Buffer = "\\";
//         Dcb->ShortName.Name.Oem.Length = (USHORT)1;
//         Dcb->ShortName.Name.Oem.MaximumLength = (USHORT)2;
//
//         //
//         //  Construct a lie about file properties since we don't
//         //  have a proper "." entry to look at.
//         //
//
//         Dcb->DirentFatFlags = FILE_ATTRIBUTE_DIRECTORY;
//
//         //
//         //  Initialize Advanced FCB Header fields
//         //
//
//         ExInitializeFastMutex( &Dcb->NonPaged->AdvancedFcbHeaderMutex );
//         FsRtlSetupAdvancedHeader( &Dcb->Header,
//                                   &Dcb->NonPaged->AdvancedFcbHeaderMutex );
//
//         //
//         //  Initialize the Mcb, and setup its mapping.  Note that the root
//         //  directory is a fixed size so we can set it everything up now.
//         //
//
//         FsRtlInitializeLargeMcb( &Dcb->Mcb, NonPagedPoolNx );
//         UnwindMcb = &Dcb->Mcb;
//
//         if (FatIsFat32(Vcb)) {
//
//             //
//             //  The first cluster of fat32 roots comes from the BPB
//             //
//
//             Dcb->FirstClusterOfFile = Vcb->Bpb.RootDirFirstCluster;
//
//         } else {
//
//             FatAddMcbEntry( Vcb, &Dcb->Mcb,
//                             0,
//                             FatRootDirectoryLbo( &Vcb->Bpb ),
//                             FatRootDirectorySize( &Vcb->Bpb ));
//         }
//
//         if (FatIsFat32(Vcb)) {
//
//             //
//             //  Find the size of the fat32 root. As a side-effect, this will create
//             //  MCBs for the entire root. In the process of doing this, we may
//             //  discover that the FAT chain is bogus and raise corruption.
//             //
//
//             Dcb->Header.AllocationSize.LowPart = 0xFFFFFFFF;
//             FatLookupFileAllocationSize( IrpContext, Dcb);
//
//             Dcb->Header.FileSize.QuadPart =
//                     Dcb->Header.AllocationSize.QuadPart;
//         } else {
//
//             //
//             //  set the allocation size to real size of the root directory
//             //
//
//             Dcb->Header.FileSize.QuadPart =
//             Dcb->Header.AllocationSize.QuadPart = FatRootDirectorySize( &Vcb->Bpb );
//
//         }
//
//         //
//         //  Set our two create dirent aids to represent that we have yet to
//         //  enumerate the directory for never used or deleted dirents.
//         //
//
//         Dcb->Specific.Dcb.UnusedDirentVbo = 0xffffffff;
//         Dcb->Specific.Dcb.DeletedDirentHint = 0xffffffff;
//
//         //
//         //  Setup the free dirent bitmap buffer.
//         //
//
//         RtlInitializeBitMap( &Dcb->Specific.Dcb.FreeDirentBitmap,
//                              NULL,
//                              0 );
//
//         FatCheckFreeDirentBitmap( IrpContext, Dcb );
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         //
//         //  Initialize the oplock structure.
//         //
//
//         FsRtlInitializeOplock( FatGetFcbOplock(Dcb) );
// #endif
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCreateRootDcb );
//
//         //
//         //  If this is an abnormal termination then undo our work
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             ULONG i;
//
//             if (UnwindFileObject != NULL) { ObDereferenceObject( UnwindFileObject ); }
//             if (UnwindMcb != NULL) { FsRtlUninitializeLargeMcb( UnwindMcb ); }
//             if (UnwindResource != NULL) { FatFreeResource( UnwindResource ); }
//             if (UnwindResource2 != NULL) { FatFreeResource( UnwindResource2 ); }
//
//             for (i = 0; i < sizeof(UnwindStorage)/sizeof(PVOID); i += 1) {
//                 if (UnwindStorage[i] != NULL) { ExFreePool( UnwindStorage[i] ); }
//             }
//
//             //
//             //  Re-zero the entry in the Vcb.
//             //
//
//             Vcb->RootDcb = NULL;
//         }
//
//         DebugTrace(-1, Dbg, "FatCreateRootDcb -> %p\n", Dcb);
//     } _SEH2_END;
//
//     return;
// }
//
//
// PFCB
// FatCreateFcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PDCB ParentDcb,
//     IN ULONG LfnOffsetWithinDirectory,
//     IN ULONG DirentOffsetWithinDirectory,
//     IN PDIRENT Dirent,
//     IN PUNICODE_STRING Lfn OPTIONAL,
//     IN PUNICODE_STRING OrigLfn OPTIONAL,
//     IN BOOLEAN IsPagingFile,
//     IN BOOLEAN SingleResource
//     )
//
// /*++
//
// Routine Description:
//
//     This routine allocates, initializes, and inserts a new Fcb record into
//     the in-memory data structures.
//
// Arguments:
//
//     Vcb - Supplies the Vcb to associate the new FCB under.
//
//     ParentDcb - Supplies the parent dcb that the new FCB is under.
//
//     LfnOffsetWithinDirectory - Supplies the offset of the LFN.  If there is
//         no LFN associated with this file then this value is same as
//         DirentOffsetWithinDirectory.
//
//     DirentOffsetWithinDirectory - Supplies the offset, in bytes from the
//         start of the directory file where the dirent for the fcb is located
//
//     Dirent - Supplies the dirent for the fcb being created
//
//     Lfn - Supplies a long UNICODE name associated with this file.
//
//     IsPagingFile - Indicates if we are creating an FCB for a paging file
//         or some other type of file.
//
//     SingleResource - Indicates if this Fcb should share a single resource
//         as both main and paging.
//
// Return Value:
//
//     PFCB - Returns a pointer to the newly allocated FCB
//
// --*/
//
// {
//     PFCB Fcb = NULL;
//     POOL_TYPE PoolType;
//
//     //
//     //  The following variables are used for abnormal unwind
//     //
//
//     PVOID UnwindStorage[2] = { NULL, NULL };
//     PERESOURCE UnwindResource = NULL;
//     PERESOURCE UnwindResource2 = NULL;
//     PLIST_ENTRY UnwindEntryList = NULL;
//     PLARGE_MCB UnwindMcb = NULL;
//     PFILE_LOCK UnwindFileLock = NULL;
//     POPLOCK UnwindOplock = NULL;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( OrigLfn );
//
//     DebugTrace(+1, Dbg, "FatCreateFcb\n", 0);
//
//     _SEH2_TRY {
//
//         //
//         //  Determine the pool type we should be using for the fcb and the
//         //  mcb structure
//         //
//
//         if (IsPagingFile) {
//
//             PoolType = NonPagedPoolNx;
//             Fcb = UnwindStorage[0] = FsRtlAllocatePoolWithTag( NonPagedPoolNx,
//                                                                sizeof(FCB),
//                                                                TAG_FCB );
//         } else {
//
//             PoolType = PagedPool;
//             Fcb = UnwindStorage[0] = FatAllocateFcb();
//
//         }
//
//         //
//         //  ... and zero it out
//         //
//
//         RtlZeroMemory( Fcb, sizeof(FCB) );
//
//         UnwindStorage[1] =
//         Fcb->NonPaged = FatAllocateNonPagedFcb();
//
//         RtlZeroMemory( Fcb->NonPaged, sizeof( NON_PAGED_FCB ) );
//
//         //
//         //  Set the proper node type code, node byte size, and call backs
//         //
//
//         Fcb->Header.NodeTypeCode = FAT_NTC_FCB;
//         Fcb->Header.NodeByteSize = sizeof(FCB);
//
//         Fcb->FcbCondition = FcbGood;
//
//         //
//         //  Check to see if we need to set the Fcb state to indicate that this
//         //  is a paging/system file.  This will prevent it from being opened
//         //  again.
//         //
//
//         if (IsPagingFile) {
//
//             SetFlag( Fcb->FcbState, FCB_STATE_PAGING_FILE | FCB_STATE_SYSTEM_FILE );
//         }
//
//         //
//         //  The initial state, open count, and segment objects fields are already
//         //  zero so we can skip setting them
//         //
//
//         //
//         //  Initialize the resource variable
//         //
//
//
//         UnwindResource =
//         Fcb->Header.Resource = FatAllocateResource();
//
//         //
//         //  Initialize the PagingIo Resource.  We no longer use the FsRtl common
//         //  shared pool because this led to a) deadlocks due to cases where files
//         //  and their parent directories shared a resource and b) there is no way
//         //  to anticipate inter-driver induced deadlock via recursive operation.
//         //
//
//         if (SingleResource) {
//
//             Fcb->Header.PagingIoResource = Fcb->Header.Resource;
//
//         } else {
//
//             UnwindResource2 =
//             Fcb->Header.PagingIoResource = FatAllocateResource();
//         }
//
//         //
//         //  Insert this fcb into our parent dcb's queue.
//         //
//         //  There is a deep reason why this goes on the tail, to allow us
//         //  to easily enumerate all child directories before child files.
//         //  This is important to let us maintain whole-volume lockorder
//         //  via BottomUp enumeration.
//         //
//
//         InsertTailList( &ParentDcb->Specific.Dcb.ParentDcbQueue,
//                         &Fcb->ParentDcbLinks );
//         UnwindEntryList = &Fcb->ParentDcbLinks;
//
//         //
//         //  Point back to our parent dcb
//         //
//
//         Fcb->ParentDcb = ParentDcb;
//
//         //
//         //  Set the Vcb
//         //
//
//         Fcb->Vcb = Vcb;
//
//         //
//         //  Set the dirent offset within the directory
//         //
//
//         Fcb->LfnOffsetWithinDirectory = LfnOffsetWithinDirectory;
//         Fcb->DirentOffsetWithinDirectory = DirentOffsetWithinDirectory;
//
//         //
//         //  Set the DirentFatFlags and LastWriteTime
//         //
//
//         Fcb->DirentFatFlags = Dirent->Attributes;
//
//         Fcb->LastWriteTime = FatFatTimeToNtTime( IrpContext,
//                                                  Dirent->LastWriteTime,
//                                                  0 );
//
//         //
//         //  These fields are only non-zero when in Chicago mode.
//         //
//
//         if (FatData.ChicagoMode) {
//
//             LARGE_INTEGER FatSystemJanOne1980 = {0};
//
//             //
//             //  If either date is possibly zero, get the system
//             //  version of 1/1/80.
//             //
//
//             if ((((PUSHORT)Dirent)[9] & ((PUSHORT)Dirent)[8]) == 0) {
//
//                 ExLocalTimeToSystemTime( &FatJanOne1980,
//                                          &FatSystemJanOne1980 );
//             }
//
//             //
//             //  Only do the really hard work if this field is non-zero.
//             //
//
//             if (((PUSHORT)Dirent)[9] != 0) {
//
//                 Fcb->LastAccessTime =
//                     FatFatDateToNtTime( IrpContext,
//                                         Dirent->LastAccessDate );
//
//             } else {
//
//                 Fcb->LastAccessTime = FatSystemJanOne1980;
//             }
//
//             //
//             //  Only do the really hard work if this field is non-zero.
//             //
//
//             if (((PUSHORT)Dirent)[8] != 0) {
//
//                 Fcb->CreationTime =
//                     FatFatTimeToNtTime( IrpContext,
//                                         Dirent->CreationTime,
//                                         Dirent->CreationMSec );
//
//             } else {
//
//                 Fcb->CreationTime = FatSystemJanOne1980;
//             }
//         }
//
//         //
//         //  Initialize Advanced FCB Header fields
//         //
//
//         ExInitializeFastMutex( &Fcb->NonPaged->AdvancedFcbHeaderMutex );
//         FsRtlSetupAdvancedHeader( &Fcb->Header,
//                                   &Fcb->NonPaged->AdvancedFcbHeaderMutex );
//
//         //
//         //  To make FAT match the present functionality of NTFS, disable
//         //  stream contexts on paging files
//         //
//
//         if (IsPagingFile) {
//
//             SetFlag( Fcb->Header.Flags2, FSRTL_FLAG2_IS_PAGING_FILE );
//             ClearFlag( Fcb->Header.Flags2, FSRTL_FLAG2_SUPPORTS_FILTER_CONTEXTS );
//         }
//
//         //
//         //  Initialize the Mcb
//         //
//
//         FsRtlInitializeLargeMcb( &Fcb->Mcb, PoolType );
//         UnwindMcb = &Fcb->Mcb;
//
//         //
//         //  Set the file size, valid data length, first cluster of file,
//         //  and allocation size based on the information stored in the dirent
//         //
//
//         Fcb->Header.FileSize.LowPart = Dirent->FileSize;
//
//         Fcb->Header.ValidDataLength.LowPart = Dirent->FileSize;
//
//         Fcb->ValidDataToDisk = Dirent->FileSize;
//
//         Fcb->FirstClusterOfFile = (ULONG)Dirent->FirstClusterOfFile;
//
//         if ( FatIsFat32(Vcb) ) {
//
//             Fcb->FirstClusterOfFile += Dirent->FirstClusterOfFileHi << 16;
//         }
//
//         if ( Fcb->FirstClusterOfFile == 0 ) {
//
//             Fcb->Header.AllocationSize.QuadPart = 0;
//
//         } else {
//
//             Fcb->Header.AllocationSize.QuadPart = FCB_LOOKUP_ALLOCATIONSIZE_HINT;
//         }
//
//
//         //
//         //  Initialize the Fcb's file lock record
//         //
//
//         FsRtlInitializeFileLock( &Fcb->Specific.Fcb.FileLock, NULL, NULL );
//         UnwindFileLock = &Fcb->Specific.Fcb.FileLock;
//
//         //
//         //  Initialize the oplock structure.
//         //
//
//         FsRtlInitializeOplock( FatGetFcbOplock(Fcb) );
//         UnwindOplock = FatGetFcbOplock(Fcb);
//
//         //
//         //  Indicate that Fast I/O is possible
//         //
//
//         Fcb->Header.IsFastIoPossible = TRUE;
//
//         //
//         //  Set the file names.  This must be the last thing we do.
//         //
//
//         FatConstructNamesInFcb( IrpContext,
//                                 Fcb,
//                                 Dirent,
//                                 Lfn );
//
//         //
//         //  Drop the shortname hint so prefix searches can figure out
//         //  what they found
//         //
//
//         Fcb->ShortName.FileNameDos = TRUE;
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCreateFcb );
//
//         //
//         //  If this is an abnormal termination then undo our work
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             ULONG i;
//
//             if (UnwindOplock != NULL) { FsRtlUninitializeOplock( UnwindOplock ); }
//             if (UnwindFileLock != NULL) { FsRtlUninitializeFileLock( UnwindFileLock ); }
//             if (UnwindMcb != NULL) { FsRtlUninitializeLargeMcb( UnwindMcb ); }
//             if (UnwindEntryList != NULL) { RemoveEntryList( UnwindEntryList ); }
//             if (UnwindResource != NULL) { FatFreeResource( UnwindResource ); }
//             if (UnwindResource2 != NULL) { FatFreeResource( UnwindResource2 ); }
//
//             for (i = 0; i < sizeof(UnwindStorage)/sizeof(PVOID); i += 1) {
//                 if (UnwindStorage[i] != NULL) { ExFreePool( UnwindStorage[i] ); }
//             }
//         }
//
//         DebugTrace(-1, Dbg, "FatCreateFcb -> %p\n", Fcb);
//     } _SEH2_END;
//
//     //
//     //  return and tell the caller
//     //
//
//     return Fcb;
// }
//
//
// PDCB
// FatCreateDcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb,
//     IN PDCB ParentDcb,
//     IN ULONG LfnOffsetWithinDirectory,
//     IN ULONG DirentOffsetWithinDirectory,
//     IN PDIRENT Dirent,
//     IN PUNICODE_STRING Lfn OPTIONAL
//     )
//
// /*++
//
// Routine Description:
//
//     This routine allocates, initializes, and inserts a new Dcb record into
//     the in memory data structures.
//
// Arguments:
//
//     Vcb - Supplies the Vcb to associate the new DCB under.
//
//     ParentDcb - Supplies the parent dcb that the new DCB is under.
//
//     LfnOffsetWithinDirectory - Supplies the offset of the LFN.  If there is
//         no LFN associated with this file then this value is same as
//         DirentOffsetWithinDirectory.
//
//     DirentOffsetWithinDirectory - Supplies the offset, in bytes from the
//         start of the directory file where the dirent for the fcb is located
//
//     Dirent - Supplies the dirent for the dcb being created
//
//     FileName - Supplies the file name of the file relative to the directory
//         it's in (e.g., the file \config.sys is called "CONFIG.SYS" without
//         the preceding backslash).
//
//     Lfn - Supplies a long UNICODE name associated with this directory.
//
// Return Value:
//
//     PDCB - Returns a pointer to the newly allocated DCB
//
// --*/
//
// {
//     PDCB Dcb = NULL;
//
//     //
//     //  The following variables are used for abnormal unwind
//     //
//
//     PVOID UnwindStorage[2] = { NULL, NULL  };
//     PERESOURCE UnwindResource = NULL;
//     PERESOURCE UnwindResource2 = NULL;
//     PLIST_ENTRY UnwindEntryList = NULL;
//     PLARGE_MCB UnwindMcb = NULL;
//     POPLOCK UnwindOplock = NULL;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatCreateDcb\n", 0);
//
//     _SEH2_TRY {
//
//         //
//         //  assert that the only time we are called is if wait is true
//         //
//
//         NT_ASSERT( FlagOn(IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT) );
//
//         //
//         //  Allocate a new DCB, and zero it out
//         //
//
//         UnwindStorage[0] = Dcb = FatAllocateFcb();
//
//         RtlZeroMemory( Dcb, sizeof(DCB) );
//
//         UnwindStorage[1] =
//         Dcb->NonPaged = FatAllocateNonPagedFcb();
//
//         RtlZeroMemory( Dcb->NonPaged, sizeof( NON_PAGED_FCB ) );
//
//         //
//         //  Set the proper node type code, node byte size and call backs
//         //
//
//         Dcb->Header.NodeTypeCode = FAT_NTC_DCB;
//         Dcb->Header.NodeByteSize = sizeof(DCB);
//
//         Dcb->FcbCondition = FcbGood;
//
//         //
//         //  The initial state, open count, and directory change count fields are
//         //  already zero so we can skip setting them
//         //
//
//         //
//         //  Initialize the resource variable
//         //
//
//
//         UnwindResource =
//         Dcb->Header.Resource = FatAllocateResource();
//
//         //
//         //  Initialize the PagingIo Resource.  We no longer use the FsRtl common
//         //  shared pool because this led to a) deadlocks due to cases where files
//         //  and their parent directories shared a resource and b) there is no way
//         //  to anticipate inter-driver induced deadlock via recursive operation.
//         //
//
//         UnwindResource2 =
//         Dcb->Header.PagingIoResource = FatAllocateResource();
//
//         //
//         //  Insert this Dcb into our parent dcb's queue
//         //
//         //  There is a deep reason why this goes on the head, to allow us
//         //  to easily enumerate all child directories before child files.
//         //  This is important to let us maintain whole-volume lockorder
//         //  via BottomUp enumeration.
//         //
//
//         InsertHeadList( &ParentDcb->Specific.Dcb.ParentDcbQueue,
//                         &Dcb->ParentDcbLinks );
//         UnwindEntryList = &Dcb->ParentDcbLinks;
//
//         //
//         //  Point back to our parent dcb
//         //
//
//         Dcb->ParentDcb = ParentDcb;
//
//         //
//         //  Set the Vcb
//         //
//
//         Dcb->Vcb = Vcb;
//
//         //
//         //  Set the dirent offset within the directory
//         //
//
//         Dcb->LfnOffsetWithinDirectory = LfnOffsetWithinDirectory;
//         Dcb->DirentOffsetWithinDirectory = DirentOffsetWithinDirectory;
//
//         //
//         //  Set the DirentFatFlags and LastWriteTime
//         //
//
//         Dcb->DirentFatFlags = Dirent->Attributes;
//
//         Dcb->LastWriteTime = FatFatTimeToNtTime( IrpContext,
//                                                  Dirent->LastWriteTime,
//                                                  0 );
//
//         //
//         //  These fields are only non-zero when in Chicago mode.
//         //
//
//         if (FatData.ChicagoMode) {
//
//             LARGE_INTEGER FatSystemJanOne1980 = {0};
//
//             //
//             //  If either date is possibly zero, get the system
//             //  version of 1/1/80.
//             //
//
//             if ((((PUSHORT)Dirent)[9] & ((PUSHORT)Dirent)[8]) == 0) {
//
//                 ExLocalTimeToSystemTime( &FatJanOne1980,
//                                          &FatSystemJanOne1980 );
//             }
//
//             //
//             //  Only do the really hard work if this field is non-zero.
//             //
//
//             if (((PUSHORT)Dirent)[9] != 0) {
//
//                 Dcb->LastAccessTime =
//                     FatFatDateToNtTime( IrpContext,
//                                         Dirent->LastAccessDate );
//
//             } else {
//
//                 Dcb->LastAccessTime = FatSystemJanOne1980;
//             }
//
//             //
//             //  Only do the really hard work if this field is non-zero.
//             //
//
//             if (((PUSHORT)Dirent)[8] != 0) {
//
//                 Dcb->CreationTime =
//                     FatFatTimeToNtTime( IrpContext,
//                                         Dirent->CreationTime,
//                                         Dirent->CreationMSec );
//
//             } else {
//
//                 Dcb->CreationTime = FatSystemJanOne1980;
//             }
//         }
//
//         //
//         //  Initialize Advanced FCB Header fields
//         //
//
//         ExInitializeFastMutex( &Dcb->NonPaged->AdvancedFcbHeaderMutex );
//         FsRtlSetupAdvancedHeader( &Dcb->Header,
//                                   &Dcb->NonPaged->AdvancedFcbHeaderMutex );
//
//         //
//         //  Initialize the Mcb
//         //
//
//         FsRtlInitializeLargeMcb( &Dcb->Mcb, PagedPool );
//         UnwindMcb = &Dcb->Mcb;
//
//         //
//         //  Set the file size, first cluster of file, and allocation size
//         //  based on the information stored in the dirent
//         //
//
//         Dcb->FirstClusterOfFile = (ULONG)Dirent->FirstClusterOfFile;
//
//         if ( FatIsFat32(Dcb->Vcb) ) {
//
//             Dcb->FirstClusterOfFile += Dirent->FirstClusterOfFileHi << 16;
//         }
//
//         if ( Dcb->FirstClusterOfFile == 0 ) {
//
//             Dcb->Header.AllocationSize.QuadPart = 0;
//
//         } else {
//
//             Dcb->Header.AllocationSize.QuadPart = FCB_LOOKUP_ALLOCATIONSIZE_HINT;
//         }
//
//
//         //  initialize the notify queues, and the parent dcb queue.
//         //
//
//         InitializeListHead( &Dcb->Specific.Dcb.ParentDcbQueue );
//
//         //
//         //  Setup the free dirent bitmap buffer.  Since we don't know the
//         //  size of the directory, leave it zero for now.
//         //
//
//         RtlInitializeBitMap( &Dcb->Specific.Dcb.FreeDirentBitmap,
//                              NULL,
//                              0 );
//
//         //
//         //  Set our two create dirent aids to represent that we have yet to
//         //  enumerate the directory for never used or deleted dirents.
//         //
//
//         Dcb->Specific.Dcb.UnusedDirentVbo = 0xffffffff;
//         Dcb->Specific.Dcb.DeletedDirentHint = 0xffffffff;
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         //
//         //  Initialize the oplock structure.
//         //
//
//         FsRtlInitializeOplock( FatGetFcbOplock(Dcb) );
//         UnwindOplock = FatGetFcbOplock(Dcb);
// #endif
//
//         //
//         //  Postpone initializing the cache map until we need to do a read/write
//         //  of the directory file.
//
//
//         //
//         //  set the file names.  This must be the last thing we do.
//         //
//
//         FatConstructNamesInFcb( IrpContext,
//                                 Dcb,
//                                 Dirent,
//                                 Lfn );
//
//         Dcb->ShortName.FileNameDos = TRUE;
//
//     } _SEH2_FINALLY {
//
//         DebugUnwind( FatCreateDcb );
//
//         //
//         //  If this is an abnormal termination then undo our work
//         //
//
//         if (_SEH2_AbnormalTermination()) {
//
//             ULONG i;
//
//             if (UnwindOplock != NULL) { FsRtlUninitializeOplock( UnwindOplock ); }
//             if (UnwindMcb != NULL) { FsRtlUninitializeLargeMcb( UnwindMcb ); }
//             if (UnwindEntryList != NULL) { RemoveEntryList( UnwindEntryList ); }
//             if (UnwindResource != NULL) { FatFreeResource( UnwindResource ); }
//             if (UnwindResource2 != NULL) { FatFreeResource( UnwindResource2 ); }
//
//             for (i = 0; i < sizeof(UnwindStorage)/sizeof(PVOID); i += 1) {
//                 if (UnwindStorage[i] != NULL) { ExFreePool( UnwindStorage[i] ); }
//             }
//         }
//
//         DebugTrace(-1, Dbg, "FatCreateDcb -> %p\n", Dcb);
//     } _SEH2_END;
//
//     //
//     //  return and tell the caller
//     //
//
//     DebugTrace(-1, Dbg, "FatCreateDcb -> %p\n", Dcb);
//
//     return Dcb;
// }
//
//
// VOID
// FatDeleteFcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB *FcbPtr
//     )
//
// /*++
//
// Routine Description:
//
//     This routine deallocates and removes an FCB, DCB, or ROOT DCB record
//     from Fat's in-memory data structures.  It also will remove all
//     associated underlings (i.e., Notify irps, and child FCB/DCB records).
//
// Arguments:
//
//     Fcb - Supplies the FCB/DCB/ROOT DCB to be removed
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     PFCB Fcb = *FcbPtr;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatDeleteFcb, Fcb = %p\n", Fcb);
//
//     //
//     //  We can only delete this record if the open count is zero.
//     //
//
//     if (Fcb->OpenCount != 0) {
//
//         DebugDump("Error deleting Fcb, Still Open\n", 0, Fcb);
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//         FatBugCheck( 0, 0, 0 );
//     }
//
//     //
//     //  Better be an FCB/DCB.
//     //
//
//     if ((Fcb->Header.NodeTypeCode != FAT_NTC_DCB) &&
//         (Fcb->Header.NodeTypeCode != FAT_NTC_ROOT_DCB) &&
//         (Fcb->Header.NodeTypeCode != FAT_NTC_FCB)) {
//
// #ifdef _MSC_VER
// #pragma prefast( suppress:28159, "things are seriously wrong if we get here" )
// #endif
//         FatBugCheck( 0, 0, 0 );
//     }
//
//     //
//     //  If this is a DCB then remove every Notify record from the two
//     //  notify queues
//     //
//
//     if ((Fcb->Header.NodeTypeCode == FAT_NTC_DCB) ||
//         (Fcb->Header.NodeTypeCode == FAT_NTC_ROOT_DCB)) {
//
//         //
//         //  If we allocated a free dirent bitmap buffer, free it.
//         //
//
//         if ((Fcb->Specific.Dcb.FreeDirentBitmap.Buffer != NULL) &&
//             (Fcb->Specific.Dcb.FreeDirentBitmap.Buffer !=
//              &Fcb->Specific.Dcb.FreeDirentBitmapBuffer[0])) {
//
//             ExFreePool(Fcb->Specific.Dcb.FreeDirentBitmap.Buffer);
//         }
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//         //
//         //  Uninitialize the oplock.
//         //
//
//         FsRtlUninitializeOplock( FatGetFcbOplock(Fcb) );
// #endif
//
//         NT_ASSERT( Fcb->Specific.Dcb.DirectoryFileOpenCount == 0 );
//         NT_ASSERT( IsListEmpty(&Fcb->Specific.Dcb.ParentDcbQueue) );
//         NT_ASSERT( NULL == Fcb->Specific.Dcb.DirectoryFile);
//
//     } else {
//
//         //
//         //  Uninitialize the byte range file locks and opportunistic locks
//         //
//
//         FsRtlUninitializeFileLock( &Fcb->Specific.Fcb.FileLock );
//         FsRtlUninitializeOplock( FatGetFcbOplock(Fcb) );
//     }
//
//
//     //
//     //  Release any Filter Context structures associated with this FCB
//     //
//
//     FsRtlTeardownPerStreamContexts( &Fcb->Header );
//
//     //
//     //  Uninitialize the Mcb
//     //
//
//     FsRtlUninitializeLargeMcb( &Fcb->Mcb );
//
//     //
//     //  If this is not the root dcb then we need to remove ourselves from
//     //  our parents Dcb queue
//     //
//
//     if (Fcb->Header.NodeTypeCode != FAT_NTC_ROOT_DCB) {
//
//         RemoveEntryList( &(Fcb->ParentDcbLinks) );
//     }
//
//     //
//     //  Remove the entry from the splay table if there is still is one.
//     //
//
//     if (FlagOn( Fcb->FcbState, FCB_STATE_NAMES_IN_SPLAY_TREE )) {
//
//         FatRemoveNames( IrpContext, Fcb );
//     }
//
//     //
//     //  Free the file name pool if allocated.
//     //
//
//     if (Fcb->Header.NodeTypeCode != FAT_NTC_ROOT_DCB) {
//
//         //
//         //  If we blew up at inconvenient times, the shortname
//         //  could be null even though you will *never* see this
//         //  normally.  Rename is a good example of this case.
//         //
//
//         if (Fcb->ShortName.Name.Oem.Buffer) {
//
//             ExFreePool( Fcb->ShortName.Name.Oem.Buffer );
//         }
//
//         if (Fcb->FullFileName.Buffer) {
//
//             ExFreePool( Fcb->FullFileName.Buffer );
//         }
//     }
//
//     if (Fcb->ExactCaseLongName.Buffer) {
//
//         ExFreePool(Fcb->ExactCaseLongName.Buffer);
//     }
//
// #ifdef SYSCACHE_COMPILE
//
//     if (Fcb->WriteMask) {
//
//         ExFreePool( Fcb->WriteMask );
//     }
//
// #endif
//
//     //
//     //  Finally deallocate the Fcb and non-paged fcb records
//     //
//
//     FatFreeResource( Fcb->Header.Resource );
//
//     if (Fcb->Header.PagingIoResource != Fcb->Header.Resource) {
//
//         FatFreeResource( Fcb->Header.PagingIoResource );
//     }
//
//     //
//     //  If an Event was allocated, get rid of it.
//     //
//
//     if (Fcb->NonPaged->OutstandingAsyncEvent) {
//
//         ExFreePool( Fcb->NonPaged->OutstandingAsyncEvent );
//     }
//
//     FatFreeNonPagedFcb( Fcb->NonPaged );
//
//     Fcb->Header.NodeTypeCode = NTC_UNDEFINED;
//
//     FatFreeFcb( Fcb );
//     *FcbPtr = NULL;
//
//     //
//     //  and return to our caller
//     //
//
//     DebugTrace(-1, Dbg, "FatDeleteFcb -> VOID\n", 0);
// }
//
//
// PCCB
// FatCreateCcb (
//     IN PIRP_CONTEXT IrpContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine creates a new CCB record
//
// Arguments:
//
// Return Value:
//
//     CCB - returns a pointer to the newly allocate CCB
//
// --*/
//
// {
//     PCCB Ccb;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatCreateCcb\n", 0);
//
//     //
//     //  Allocate a new CCB Record
//     //
//
//     Ccb = FatAllocateCcb();
//
//     RtlZeroMemory( Ccb, sizeof(CCB) );
//
//     //
//     //  Set the proper node type code and node byte size
//     //
//
//     Ccb->NodeTypeCode = FAT_NTC_CCB;
//     Ccb->NodeByteSize = sizeof(CCB);
//
//     //
//     //  return and tell the caller
//     //
//
//     DebugTrace(-1, Dbg, "FatCreateCcb -> %p\n", Ccb);
//
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     return Ccb;
// }
//
//
//
// VOID
// FatDeallocateCcbStrings(
//     IN PCCB Ccb
//     )
// /*++
//
// Routine Description:
//
//     This routine deallocates CCB query templates
//
// Arguments:
//
//     Ccb - Supplies the CCB
//
// Return Value:
//
//     None
//
// --*/
// {
//     PAGED_CODE();
//
//     //
//     //  If we allocated query template buffers, deallocate them now.
//     //
//
//     if (FlagOn(Ccb->Flags, CCB_FLAG_FREE_UNICODE)) {
//
//         NT_ASSERT( Ccb->UnicodeQueryTemplate.Buffer);
//         NT_ASSERT( !FlagOn( Ccb->Flags, CCB_FLAG_CLOSE_CONTEXT));
//         RtlFreeUnicodeString( &Ccb->UnicodeQueryTemplate );
//     }
//
//     if (FlagOn(Ccb->Flags, CCB_FLAG_FREE_OEM_BEST_FIT)) {
//
//         NT_ASSERT( Ccb->OemQueryTemplate.Wild.Buffer );
//         NT_ASSERT( !FlagOn( Ccb->Flags, CCB_FLAG_CLOSE_CONTEXT));
//         RtlFreeOemString( &Ccb->OemQueryTemplate.Wild );
//     }
//
//     ClearFlag( Ccb->Flags, CCB_FLAG_FREE_OEM_BEST_FIT | CCB_FLAG_FREE_UNICODE);
// }
//
//
//
// VOID
// FatDeleteCcb (
//     IN PIRP_CONTEXT IrpContext,
//     IN PCCB *Ccb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine deallocates and removes the specified CCB record
//     from the Fat in memory data structures
//
// Arguments:
//
//     Ccb - Supplies the CCB to remove
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
//     DebugTrace(+1, Dbg, "FatDeleteCcb, Ccb = %p\n", *Ccb);
//
//     FatDeallocateCcbStrings( *Ccb);
//
//     //
//     //  Deallocate the Ccb record
//     //
//
//     FatFreeCcb( *Ccb );
//     *Ccb = NULL;
//
//     //
//     //  return and tell the caller
//     //
//
//     DebugTrace(-1, Dbg, "FatDeleteCcb -> VOID\n", 0);
//
//     UNREFERENCED_PARAMETER( IrpContext );
// }
//
//
// PIRP_CONTEXT
// FatCreateIrpContext (
//     IN PIRP Irp,
//     IN BOOLEAN Wait
//     )
//
// /*++
//
// Routine Description:
//
//     This routine creates a new IRP_CONTEXT record
//
// Arguments:
//
//     Irp - Supplies the originating Irp.
//
//     Wait - Supplies the wait value to store in the context
//
// Return Value:
//
//     PIRP_CONTEXT - returns a pointer to the newly allocate IRP_CONTEXT Record
//
// --*/
//
// {
//     PIRP_CONTEXT IrpContext;
//     PIO_STACK_LOCATION IrpSp;
//
//     PAGED_CODE();
//
//     DebugTrace(+1, Dbg, "FatCreateIrpContext\n", 0);
//
//     IrpSp = IoGetCurrentIrpStackLocation( Irp );
//
//     //
//     //  The only operations a filesystem device object should ever receive
//     //  are create/teardown of fsdo handles and operations which do not
//     //  occur in the context of fileobjects (i.e., mount).
//     //
//
//     if (FatDeviceIsFatFsdo( IrpSp->DeviceObject))  {
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
//     //  Attemtp to allocate from the region first and failing that allocate
//     //  from pool.
//     //
//
//     DebugDoit( FatFsdEntryCount += 1);
//
//     IrpContext = FatAllocateIrpContext();
//
//     //
//     //  Zero out the irp context.
//     //
//
//     RtlZeroMemory( IrpContext, sizeof(IRP_CONTEXT) );
//
//     //
//     //  Set the proper node type code and node byte size
//     //
//
//     IrpContext->NodeTypeCode = FAT_NTC_IRP_CONTEXT;
//     IrpContext->NodeByteSize = sizeof(IRP_CONTEXT);
//
//     //
//     //  Set the originating Irp field
//     //
//
//     IrpContext->OriginatingIrp = Irp;
//
//     //
//     //  Major/Minor Function codes
//     //
//
//     IrpContext->MajorFunction = IrpSp->MajorFunction;
//     IrpContext->MinorFunction = IrpSp->MinorFunction;
//
//     //
//     //  Copy RealDevice for workque algorithms, and also set Write Through
//     //  and Removable Media if there is a file object.  Only file system
//     //  control Irps won't have a file object, and they should all have
//     //  a Vpb as the first IrpSp location.
//     //
//
//     if (IrpSp->FileObject != NULL) {
//
//         PFILE_OBJECT FileObject = IrpSp->FileObject;
//
//         IrpContext->RealDevice = FileObject->DeviceObject;
//         IrpContext->Vcb = &((PVOLUME_DEVICE_OBJECT)(IrpSp->DeviceObject))->Vcb;
//
//         //
//         //  See if the request is Write Through. Look for both FileObjects opened
//         //  as write through, and non-cached requests with the SL_WRITE_THROUGH flag set.
//         //
//         //  The latter can only originate from kernel components. (Note - NtWriteFile()
//         //  does redundantly set the SL_W_T flag for all requests it issues on write
//         //  through file objects)
//         //
//
//         if (IsFileWriteThrough( FileObject, IrpContext->Vcb ) ||
//             ( (IrpSp->MajorFunction == IRP_MJ_WRITE) &&
//               BooleanFlagOn( Irp->Flags, IRP_NOCACHE) &&
//               BooleanFlagOn( IrpSp->Flags, SL_WRITE_THROUGH))) {
//
//             SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_WRITE_THROUGH);
//         }
//     } else if (IrpContext->MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL) {
//
//         IrpContext->RealDevice = IrpSp->Parameters.MountVolume.Vpb->RealDevice;
//     }
//
//     //
//     //  Set the wait parameter
//     //
//
//     if (Wait) { SetFlag( IrpContext->Flags, IRP_CONTEXT_FLAG_WAIT); }
//
//     //
//     //  Set the recursive file system call parameter.  We set it true if
//     //  the TopLevelIrp field in the thread local storage is not the current
//     //  irp, otherwise we leave it as FALSE.
//     //
//
//     if ( IoGetTopLevelIrp() != Irp) {
//
//         SetFlag(IrpContext->Flags, IRP_CONTEXT_FLAG_RECURSIVE_CALL);
//     }
//
//     //
//     //  return and tell the caller
//     //
//
//     DebugTrace(-1, Dbg, "FatCreateIrpContext -> %p\n", IrpContext);
//
//     return IrpContext;
// }
//
//
//
// VOID
// FatDeleteIrpContext_Real (
//     IN PIRP_CONTEXT IrpContext
//     )
//
// /*++
//
// Routine Description:
//
//     This routine deallocates and removes the specified IRP_CONTEXT record
//     from the Fat in memory data structures.  It should only be called
//     by FatCompleteRequest.
//
// Arguments:
//
//     IrpContext - Supplies the IRP_CONTEXT to remove
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
//     DebugTrace(+1, Dbg, "FatDeleteIrpContext, IrpContext = %p\n", IrpContext);
//
//     NT_ASSERT( IrpContext->NodeTypeCode == FAT_NTC_IRP_CONTEXT );
//     NT_ASSERT( IrpContext->PinCount == 0 );
//
//
//     //
//     //  If there is a FatIoContext that was allocated, free it.
//     //
//
//     if (IrpContext->FatIoContext != NULL) {
//
//         if (!FlagOn(IrpContext->Flags, IRP_CONTEXT_STACK_IO_CONTEXT)) {
//
//             if (IrpContext->FatIoContext->ZeroMdl) {
//                 IoFreeMdl( IrpContext->FatIoContext->ZeroMdl );
//             }
//
//             ExFreePool( IrpContext->FatIoContext );
//         }
//     }
//
//     //
//     //  Drop the IrpContext.
//     //
//
//     FatFreeIrpContext( IrpContext );
//
//     //
//     //  return and tell the caller
//     //
//
//     DebugTrace(-1, Dbg, "FatDeleteIrpContext -> VOID\n", 0);
//
//     return;
// }
//
//
// PFCB
// FatGetNextFcbBottomUp (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb OPTIONAL,
//     IN PFCB TerminationFcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is used to iterate through Fcbs in a tree.  In order to match
//     the lockorder for getting multiple Fcbs (so this can be used for acquiring
//     all Fcbs), this version does a bottom-up enumeration.
//
//     This is different than the old one, now called TopDown. The problem with
//     lockorder was very well hidden.
//
//     The transition rule is still pretty simple:
//
//         A) If you have an adjacent sibling, go to it
//             1) Descend to its leftmost child
//         B) Else go to your parent
//
//     If this routine is called with in invalid TerminationFcb it will fail,
//     badly.
//
//     The TerminationFcb is the last Fcb returned in the enumeration.
//
//     This method is incompatible with the possibility that ancestors may vanish
//     based on operations done on the last returned node.  For instance,
//     FatPurgeReferencedFileObjects cannot use BottomUp enumeration.
//
// Arguments:
//
//     Fcb - Supplies the current Fcb.  This is NULL if enumeration is starting.
//
//     TerminationFcb - The root Fcb of the tree in which the enumeration starts
//         and at which it inclusively stops.
//
// Return Value:
//
//     The next Fcb in the enumeration, or NULL if Fcb was the final one.
//
// --*/
//
// {
//     PFCB NextFcb;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     NT_ASSERT( FatVcbAcquiredExclusive( IrpContext, TerminationFcb->Vcb ) ||
//             FlagOn( TerminationFcb->Vcb->VcbState, VCB_STATE_FLAG_LOCKED ) );
//
//     //
//     //  Do we need to begin the enumeration?
//     //
//
//     if (Fcb != NULL) {
//
//         //
//         //  Did we complete?
//         //
//
//         if (Fcb == TerminationFcb) {
//
//             return NULL;
//         }
//
//         //
//         //  Do we have a sibling to return?
//         //
//
//         NextFcb = FatGetNextSibling( Fcb );
//
//         //
//         //  If not, return our parent.  We are done with this branch.
//         //
//
//         if (NextFcb == NULL) {
//
//             return Fcb->ParentDcb;
//         }
//
//     } else {
//
//         NextFcb = TerminationFcb;
//     }
//
//     //
//     //  Decend to its furthest child (if it exists) and return it.
//     //
//
//     for (;
//          NodeType( NextFcb ) != FAT_NTC_FCB && FatGetFirstChild( NextFcb ) != NULL;
//          NextFcb = FatGetFirstChild( NextFcb )) {
//     }
//
//     return NextFcb;
// }
//
// PFCB
// FatGetNextFcbTopDown (
//     IN PIRP_CONTEXT IrpContext,
//     IN PFCB Fcb,
//     IN PFCB TerminationFcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is used to iterate through Fcbs in a tree, from the top down.
//
//     The rule is very simple:
//
//         A) If you have a child, go to it, else
//         B) If you have an older sibling, go to it, else
//         C) Go to your parent's older sibling.
//
//     If this routine is called with in invalid TerminationFcb it will fail,
//     badly.
//
//     The Termination Fcb is never returned.  If it is the root of the tree you
//     are traversing, visit it first.
//
//     This routine never returns direct ancestors of Fcb, and thus is useful when
//     making Fcb's go away (which may tear up the tree).
//
// Arguments:
//
//     Fcb - Supplies the current Fcb
//
//     TerminationFcb - The Fcb at which the enumeration should (non-inclusivly)
//         stop.  Assumed to be a directory.
//
// Return Value:
//
//     The next Fcb in the enumeration, or NULL if Fcb was the final one.
//
// --*/
//
// {
//     PFCB Sibling;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     NT_ASSERT( FatVcbAcquiredExclusive( IrpContext, Fcb->Vcb ) ||
//             FlagOn( Fcb->Vcb->VcbState, VCB_STATE_FLAG_LOCKED ) );
//
//     //
//     //  If this was a directory (ie. not a file), get the child.  If
//     //  there aren't any children and this is our termination Fcb,
//     //  return NULL.
//     //
//
//     if ( ((NodeType(Fcb) == FAT_NTC_DCB) ||
//           (NodeType(Fcb) == FAT_NTC_ROOT_DCB)) &&
//          !IsListEmpty(&Fcb->Specific.Dcb.ParentDcbQueue) ) {
//
//         return FatGetFirstChild( Fcb );
//     }
//
//     //
//     //  Were we only meant to do one iteration?
//     //
//
//     if ( Fcb == TerminationFcb ) {
//
//         return NULL;
//     }
//
//     Sibling = FatGetNextSibling(Fcb);
//
//     while (TRUE) {
//
//         //
//         //  Do we still have an "older" sibling in this directory who is
//         //  not the termination Fcb?
//         //
//
//         if ( Sibling != NULL ) {
//
//             return (Sibling != TerminationFcb) ? Sibling : NULL;
//         }
//
//         //
//         //  OK, let's move on to out parent and see if he is the termination
//         //  node or has any older siblings.
//         //
//
//         if ( Fcb->ParentDcb == TerminationFcb ) {
//
//             return NULL;
//         }
//
//         Fcb = Fcb->ParentDcb;
//
//         Sibling = FatGetNextSibling(Fcb);
//     }
// }
//
//
// BOOLEAN
// FatSwapVpb (
//     IN PIRP_CONTEXT IrpContext,
//     PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine swaps the VPB for this VCB if it has not been done already.
//     This means the device object will get our spare VPB and we will cleanup
//     the one used while the volume was mounted.
//
// Arguments:
//
//     IrpContext - Supplies the context for the overall request.
//
//     Vcb - Supplies the VCB to swap the VPB on.
//
// Return Value:
//
//     TRUE - If the VPB was actually swapped.
//
//     FALSE - If the VPB was already swapped.
//
// --*/
//
// {
//     BOOLEAN Result = FALSE;
//     PVPB OldVpb;
//     PIO_STACK_LOCATION IrpSp;
//
//
//     //
//     //  Make sure we have not already swapped it.
//     //
//
//     OldVpb = Vcb->Vpb;
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable: 28175, "touching Vpb is ok for a filesystem" )
// #endif
//
//     if (!FlagOn( Vcb->VcbState, VCB_STATE_FLAG_VPB_MUST_BE_FREED ) && OldVpb->RealDevice->Vpb == OldVpb) {
//
//         //
//         //  If not the final reference and we are forcing the disconnect,
//         //  then swap out the Vpb.  We must preserve the REMOVE_PENDING flag
//         //  so that the device is not remounted in the middle of a PnP remove
//         //  operation.
//         //
//
//         NT_ASSERT( Vcb->SwapVpb != NULL );
//
//         Vcb->SwapVpb->Type = IO_TYPE_VPB;
//         Vcb->SwapVpb->Size = sizeof( VPB );
//         Vcb->SwapVpb->RealDevice = OldVpb->RealDevice;
//
//         Vcb->SwapVpb->RealDevice->Vpb = Vcb->SwapVpb;
//
//         Vcb->SwapVpb->Flags = FlagOn( OldVpb->Flags, VPB_REMOVE_PENDING );
//
//         //
//         //  If we are working on a mount request, we need to make sure we update
//         //  the VPB in the IRP, since the one it points to may no longer be valid.
//         //
//
//         if (IrpContext->MajorFunction == IRP_MJ_FILE_SYSTEM_CONTROL && IrpContext->MinorFunction == IRP_MN_MOUNT_VOLUME) {
//
//             //
//             //  Get the IRP stack.
//             //
//
//             IrpSp = IoGetCurrentIrpStackLocation( IrpContext->OriginatingIrp );
//
//             NT_ASSERT( IrpSp->FileObject == NULL );
//
//             //
//             //  Check the VPB in the IRP to see if it is the one we are swapping.
//             //
//
//             if (IrpSp->Parameters.MountVolume.Vpb == OldVpb) {
//
//                 //
//                 //  Change the IRP to point to the swap VPB.
//                 //
//
//                 IrpSp->Parameters.MountVolume.Vpb = Vcb->SwapVpb;
//             }
//         }
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//         //
//         //  We place the volume in the Bad state (as opposed to NotMounted) so
//         //  that it is not eligible for a remount.  Also indicate we used up
//         //  the swap.
//         //
//
//         Vcb->SwapVpb = NULL;
//         FatSetVcbCondition( Vcb, VcbBad);
//         SetFlag( Vcb->VcbState, VCB_STATE_FLAG_VPB_MUST_BE_FREED );
//
//         Result = TRUE;
//     }
//
//     return Result;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// BOOLEAN
// FatCheckForDismount (
//     IN PIRP_CONTEXT IrpContext,
//     PVCB Vcb,
//     IN BOOLEAN Force
//     )
//
// /*++
//
// Routine Description:
//
//     This routine determines if a volume is ready for deletion.  It
//     correctly synchronizes with creates en-route to the file system.
//
// Arguments:
//
//     Vcb - Supplies the volume to examine
//
//     Force - Specifies whether we want this Vcb forcibly disconnected
//         from the driver stack if it will not be deleted (a new vpb will
//         be installed if neccesary).  Caller is responsible for making
//         sure that the volume has been marked in such a way that attempts
//         to operate through the realdevice are blocked (i.e., move the vcb
//         out of the mounted state).
//
// Return Value:
//
//     BOOLEAN - TRUE if the volume was deleted, FALSE otherwise.
//
// --*/
//
// {
//     KIRQL SavedIrql;
//     BOOLEAN VcbDeleted = FALSE;
//
//
//     //
//     //  If the VCB condition is good and we are not forcing, just return.
//     //
//
//     if (Vcb->VcbCondition == VcbGood && !Force) {
//
//         return FALSE;
//     }
//
//     //
//     //  Now check for a zero Vpb count on an unmounted volume.  These
//     //  volumes will be deleted as they now have no file objects and
//     //  there are no creates en route to this volume.
//     //
//
//     IoAcquireVpbSpinLock( &SavedIrql );
//
//     if (Vcb->Vpb->ReferenceCount == Vcb->ResidualOpenCount && Vcb->OpenFileCount == 0) {
//
//         PVPB Vpb = Vcb->Vpb;
//
// #if DBG
//         UNICODE_STRING VolumeLabel;
//
//         //
//         //  Setup the VolumeLabel string
//         //
//
//         VolumeLabel.Length = Vcb->Vpb->VolumeLabelLength;
//         VolumeLabel.MaximumLength = MAXIMUM_VOLUME_LABEL_LENGTH;
//         VolumeLabel.Buffer = &Vcb->Vpb->VolumeLabel[0];
//
//         KdPrintEx((DPFLTR_FASTFAT_ID,
//                    DPFLTR_INFO_LEVEL,
//                    "FASTFAT: Dismounting Volume %Z\n",
//                    &VolumeLabel));
// #endif // DBG
//
//         //
//         //  Swap this VCB's VPB.
//         //
//
//         FatSwapVpb( IrpContext,
//                     Vcb );
//
//         //
//         //  Clear the VPB_MOUNTED bit.  New opens should not come in due
//         //  to the swapped VPB, but having the flag cleared helps debugging.
//         //  Note that we must leave the Vpb->DeviceObject field set until
//         //  after the FatTearDownVcb call as closes will have to make their
//         //  way back to us.
//         //
//
//         ClearFlag( Vpb->Flags, VPB_MOUNTED );
//
//         //
//         //  If this Vpb was locked, clear this flag now.
//         //
//
//         ClearFlag( Vpb->Flags, (VPB_LOCKED | VPB_DIRECT_WRITES_ALLOWED) );
//
//         IoReleaseVpbSpinLock( SavedIrql );
//
//         //
//         //  We are going to attempt the dismount, so mark the VCB as having
//         //  a dismount in progress.
//         //
//
//         NT_ASSERT( !FlagOn( Vcb->VcbState, VCB_STATE_FLAG_DISMOUNT_IN_PROGRESS ) );
//         SetFlag( Vcb->VcbState, VCB_STATE_FLAG_DISMOUNT_IN_PROGRESS );
//
//         //
//         //  Close down our internal opens.
//         //
//
//         FatTearDownVcb( IrpContext,
//                         Vcb );
//
//         //
//         //  Process any delayed closes.
//         //
//
//         FatFspClose( Vcb );
//
//         //
//         //  Grab the VPB lock again so that we can recheck our counts.
//         //
//
//         IoAcquireVpbSpinLock( &SavedIrql );
//
//         //
//         //  See if we can delete this VCB.
//         //
//
//         if (Vcb->Vpb->ReferenceCount == 0 && Vcb->InternalOpenCount == 0) {
//
//             Vpb->DeviceObject = NULL;
//
//             IoReleaseVpbSpinLock( SavedIrql );
//
//             FatDeleteVcb( IrpContext, Vcb );
//
//             IoDeleteDevice( (PDEVICE_OBJECT)
//                             CONTAINING_RECORD( Vcb,
//                                                VOLUME_DEVICE_OBJECT,
//                                                Vcb ) );
//
//             VcbDeleted = TRUE;
//
//         } else {
//
//             IoReleaseVpbSpinLock( SavedIrql );
//
//             NT_ASSERT( FlagOn( Vcb->VcbState, VCB_STATE_FLAG_DISMOUNT_IN_PROGRESS ) );
//             ClearFlag( Vcb->VcbState, VCB_STATE_FLAG_DISMOUNT_IN_PROGRESS );
//         }
//
//     } else if (Force) {
//
//         //
//         //  The requester is forcing the issue.  We need to swap the VPB with our spare.
//         //
//
//         FatSwapVpb( IrpContext,
//                     Vcb );
//
//         IoReleaseVpbSpinLock( SavedIrql );
//
//     } else {
//
//         //
//         //  Just drop the Vpb spinlock.
//         //
//
//         IoReleaseVpbSpinLock( SavedIrql );
//     }
//
//     return VcbDeleted;
// }
//
//
// VOID
// FatConstructNamesInFcb (
//     IN PIRP_CONTEXT IrpContext,
//     PFCB Fcb,
//     PDIRENT Dirent,
//     PUNICODE_STRING Lfn OPTIONAL
//     )
//
// /*++
//
// Routine Description:
//
//     This routine places the short name in the dirent in the first set of
//     STRINGs in the Fcb.  If a long file name (Lfn) was specified, then
//     we must decide whether we will store its Oem equivolent in the same
//     prefix table as the short name, or rather just save the upcased
//     version of the UNICODE string in the FCB.
//
//     For looking up Fcbs, the first approach will be faster, so we want to
//     do this as much as possible.  Here are the rules that I have thought
//     through extensively to determine when it is safe to store only Oem
//     version of the UNICODE name.
//
//     - If the UNICODE name contains no extended characters (>0x80), use Oem.
//
//     - Let U be the upcased version of the UNICODE name.
//       Let Up(x) be the function that upcases a UNICODE string.
//       Let Down(x) be the function that upcases a UNICODE string.
//       Let OemToUni(x) be the function that converts an Oem string to Unicode.
//       Let UniToOem(x) be the function that converts a Unicode string to Oem.
//       Let BestOemFit(x) be the function that creates the Best uppercase Oem
//         fit for the UNICODE string x.
//
//       BestOemFit(x) = UniToOem(Up(OemToUni(UniToOem(x))))   <1>
//
//       if (BestOemFit(U) == BestOemFit(Down(U))              <2>
//
//           then I know that there exists no UNICODE string Y such that:
//
//               Up(Y) == Up(U)                                <3>
//
//               AND
//
//               BestOemFit(U) != BestOemFit(Y)                <4>
//
//       Consider string U as a collection of one character strings.  The
//       conjecture is clearly true for each sub-string, thus it is true
//       for the entire string.
//
//       Equation <1> is what we use to convert an incoming unicode name in
//       FatCommonCreate() to Oem.  The double conversion is done to provide
//       better visual best fitting for characters in the Ansi code page but
//       not in the Oem code page.  A single Nls routine is provided to do
//       this conversion efficiently.
//
//       The idea is that with U, I only have to worry about a case varient Y
//       matching it in a unicode compare, and I have shown that any case varient
//       of U (the set Y defined in equation <3>), when filtered through <1>
//       (as in create), will match the Oem string defined in <1>.
//
//       Thus I do not have to worry about another UNICODE string missing in
//       the prefix lookup, but matching when comparing LFNs in the directory.
//
// Arguments:
//
//     Fcb - The Fcb we are supposed to fill in.  Note that ParentDcb must
//         already be filled in.
//
//     Dirent - The gives up the short name.
//
//     Lfn - If provided, this gives us the long name.
//
// Return Value:
//
//     None
//
// --*/
//
// {
// #ifndef __REACTOS__
//     NTSTATUS Status;
// #endif
//     ULONG i;
//
// #ifndef __REACTOS__
//     OEM_STRING OemA;
//     OEM_STRING OemB;
// #endif
//     UNICODE_STRING Unicode;
//     POEM_STRING ShortName;
//     POEM_STRING LongOemName;
//     PUNICODE_STRING LongUniName;
//
//     PAGED_CODE();
//
//     ShortName = &Fcb->ShortName.Name.Oem;
//
//     NT_ASSERT( ShortName->Buffer == NULL );
//
//     _SEH2_TRY {
//
//         //
//         //  First do the short name.
//         //
//
//         //
//         //  Copy over the case flags for the short name of the file
//         //
//
//         if (FlagOn(Dirent->NtByte, FAT_DIRENT_NT_BYTE_8_LOWER_CASE)) {
//
//             SetFlag(Fcb->FcbState, FCB_STATE_8_LOWER_CASE);
//
//         } else {
//
//             ClearFlag(Fcb->FcbState, FCB_STATE_8_LOWER_CASE);
//         }
//
//         if (FlagOn(Dirent->NtByte, FAT_DIRENT_NT_BYTE_3_LOWER_CASE)) {
//
//             SetFlag(Fcb->FcbState, FCB_STATE_3_LOWER_CASE);
//
//         } else {
//
//             ClearFlag(Fcb->FcbState, FCB_STATE_3_LOWER_CASE);
//         }
//
//         ShortName->MaximumLength = 16;
//         ShortName->Buffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                       16,
//                                                       TAG_FILENAME_BUFFER );
//
//         Fat8dot3ToString( IrpContext, Dirent, FALSE, ShortName );
//
//         Fcb->ShortName.FileNameDos = TRUE;
//
//         //
//         //  If no Lfn was specified, we are done.  In either case, set the
//         //  final name length.
//         //
//
//         NT_ASSERT( Fcb->ExactCaseLongName.Buffer == NULL );
//
//         if (!ARGUMENT_PRESENT(Lfn) || (Lfn->Length == 0)) {
//
//             Fcb->FinalNameLength = (USHORT) RtlOemStringToCountedUnicodeSize( ShortName );
//             Fcb->ExactCaseLongName.Length = Fcb->ExactCaseLongName.MaximumLength = 0;
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  If we already set up the full filename, we could be in trouble.  If the fast
//         //  path for doing it already fired, FatSetFullFileNameInFcb, it will have missed
//         //  this and could have built the full filename out of the shortname of the file.
//         //
//         //  At that point, disaster could be inevitable since the final name length will not
//         //  match.  We use this to tell the notify package what to do - FatNotifyReportChange.
//         //
//
//         NT_ASSERT( Fcb->FullFileName.Buffer == NULL );
//
//         //
//         //  We know now we have an Lfn, save away a copy.
//         //
//
//         Fcb->FinalNameLength = Lfn->Length;
//
//         Fcb->ExactCaseLongName.Length = Fcb->ExactCaseLongName.MaximumLength = Lfn->Length;
//         Fcb->ExactCaseLongName.Buffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                                   Lfn->Length,
//                                                                   TAG_FILENAME_BUFFER );
//         RtlCopyMemory(Fcb->ExactCaseLongName.Buffer, Lfn->Buffer, Lfn->Length);
//
//         //
//         //  First check for no extended characters.
//         //
//
//         for (i=0; i < Lfn->Length/sizeof(WCHAR); i++) {
//
//             if (Lfn->Buffer[i] >= 0x80) {
//
//                 break;
//             }
//         }
//
//         if (i == Lfn->Length/sizeof(WCHAR)) {
//
//             //
//             //  Cool, I can go with the Oem, upcase it fast by hand.
//             //
//
//             LongOemName = &Fcb->LongName.Oem.Name.Oem;
//
//
//             LongOemName->Buffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                             Lfn->Length/sizeof(WCHAR),
//                                                             TAG_FILENAME_BUFFER );
//             LongOemName->Length =
//             LongOemName->MaximumLength = Lfn->Length/sizeof(WCHAR);
//
//             for (i=0; i < Lfn->Length/sizeof(WCHAR); i++) {
//
//                 WCHAR c;
//
//                 c = Lfn->Buffer[i];
//
// #ifdef _MSC_VER
// #pragma warning( push )
// #pragma warning( disable:4244 )
// #endif
//                 LongOemName->Buffer[i] = c < 'a' ?
//                                          (UCHAR)c :
//                                          c <= 'z' ?
//                                          c - (UCHAR)('a'-'A') :
//                                          (UCHAR)c;
// #ifdef _MSC_VER
// #pragma warning( pop )
// #endif
//             }
//
//             //
//             //  If this name happens to be exactly the same as the short
//             //  name, don't add it to the splay table.
//             //
//
//             if (FatAreNamesEqual(IrpContext, *ShortName, *LongOemName) ||
//                 (FatFindFcb( IrpContext,
//                              &Fcb->ParentDcb->Specific.Dcb.RootOemNode,
//                              LongOemName,
//                              NULL) != NULL)) {
//
//                 ExFreePool( LongOemName->Buffer );
//
//                 LongOemName->Buffer = NULL;
//                 LongOemName->Length =
//                 LongOemName->MaximumLength = 0;
//
//             } else {
//
//                 SetFlag( Fcb->FcbState, FCB_STATE_HAS_OEM_LONG_NAME );
//             }
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  Now we have the fun part.  Make a copy of the Lfn.
//         //
//
// #ifndef __REACTOS__
//         OemA.Buffer = NULL;
//         OemB.Buffer = NULL;
// #endif
//         Unicode.Buffer = NULL;
//
//         Unicode.Length =
//         Unicode.MaximumLength = Lfn->Length;
//         Unicode.Buffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                    Lfn->Length,
//                                                    TAG_FILENAME_BUFFER );
//
//         RtlCopyMemory( Unicode.Buffer, Lfn->Buffer, Lfn->Length );
//
// #ifndef __REACTOS__
//         Status = STATUS_SUCCESS;
// #endif
//
// #if TRUE
//         //
//         //  Unfortunately, this next block of code breaks down when you have
//         //  two long Unicode filenames that both map to the same Oem (and are,
//         //  well, long, i.e. are not the short names). In this case, with one
//         //  in the prefix table first, the other will hit the common Oem
//         //  representation.  This leads to several forms of user astonishment.
//         //
//         //  It isn't worth it, or probably even possible, to try to figure out
//         //  when this is really safe to go through.  Simply omit the attempt.
//         //
//         //  Ex: ANSI 0x82 and 0x84 in the 1252 ANSI->UNI and 437 UNI->OEM codepages.
//         //
//         //      0x82 => 0x201a => 0x2c
//         //      0x84 => 0x201e => 0x2c
//         //
//         //  0x2c is comma, so is FAT Oem illegal and forces shortname generation.
//         //  Since it is otherwise well-formed by the rules articulated previously,
//         //  we would have put 0x2c in the Oem prefix tree.  In terms of the
//         //  argument given above, even though there exist no Y and U s.t.
//         //
//         //  Up(Y) == Up(U) && BestOemFit(U) != BestOemFit(Y)
//         //
//         //  there most certainly exist Y and U s.t.
//         //
//         //  Up(Y) != Up(U) && BestOemFit(U) == BestOemFit(Y)
//         //
//         //  and that is enough to keep us from doing this.  Note that the < 0x80
//         //  case is OK since we know that the mapping in the OEM codepages are
//         //  the identity in that range.
//         //
//         //  We still need to monocase it, though.  Do this through a full down/up
//         //  transition.
//         //
//
//         (VOID)RtlDowncaseUnicodeString( &Unicode, &Unicode, FALSE );
//         (VOID)RtlUpcaseUnicodeString( &Unicode, &Unicode, FALSE );
// #else
//         //
//         //  Downcase and convert to upcased Oem.  Only continue if we can
//         //  convert without error.  Any error other than UNMAPPABLE_CHAR
//         //  is a fatal error and we raise.
//         //
//         //  Note that even if the conversion fails, we must leave Unicode
//         //  in an upcased state.
//         //
//         //  NB: The Rtl doesn't NULL .Buffer on error.
//         //
//
//         (VOID)RtlDowncaseUnicodeString( &Unicode, &Unicode, FALSE );
//         Status = RtlUpcaseUnicodeStringToCountedOemString( &OemA, &Unicode, TRUE );
//         (VOID)RtlUpcaseUnicodeString( &Unicode, &Unicode, FALSE );
//
//         if (!NT_SUCCESS(Status)) {
//
//             if (Status != STATUS_UNMAPPABLE_CHARACTER) {
//
//                 NT_ASSERT( Status == STATUS_NO_MEMORY );
//                 ExFreePool(Unicode.Buffer);
//                 FatNormalizeAndRaiseStatus( IrpContext, Status );
//             }
//
//         } else {
//
//             //
//             //  The same as above except upcase.
//             //
//
//             Status = RtlUpcaseUnicodeStringToCountedOemString( &OemB, &Unicode, TRUE );
//
//             if (!NT_SUCCESS(Status)) {
//
//                 RtlFreeOemString( &OemA );
//
//                 if (Status != STATUS_UNMAPPABLE_CHARACTER) {
//
//                     NT_ASSERT( Status == STATUS_NO_MEMORY );
//                     ExFreePool(Unicode.Buffer);
//                     FatNormalizeAndRaiseStatus( IrpContext, Status );
//                 }
//             }
//         }
//
//         //
//         //  If the final OemNames are equal, I can use save only the Oem
//         //  name.  If the name did not map, then I have to go with the UNICODE
//         //  name because I could get a case varient that didn't convert
//         //  in create, but did match the LFN.
//         //
//
//         if (NT_SUCCESS(Status) && FatAreNamesEqual( IrpContext, OemA, OemB )) {
//
//             //
//             //  Cool, I can go with the Oem.  If we didn't convert correctly,
//             //  get a fresh convert from the original LFN.
//             //
//
//             ExFreePool(Unicode.Buffer);
//
//             RtlFreeOemString( &OemB );
//
//             Fcb->LongName.Oem.Name.Oem = OemA;
//
//             //
//             //  If this name happens to be exactly the same as the short
//             //  name, or a similar short name already exists don't add it
//             //  to the splay table (note the final condition implies a
//             //  corrupt disk.
//             //
//
//             if (FatAreNamesEqual(IrpContext, *ShortName, OemA) ||
//                 (FatFindFcb( IrpContext,
//                              &Fcb->ParentDcb->Specific.Dcb.RootOemNode,
//                              &OemA,
//                              NULL) != NULL)) {
//
//                 RtlFreeOemString( &OemA );
//
//             } else {
//
//                 SetFlag( Fcb->FcbState, FCB_STATE_HAS_OEM_LONG_NAME );
//             }
//
//             try_return( NOTHING );
//         }
//
//         //
//         //  The long name must be left in UNICODE.  Free the two Oem strings
//         //  if we got here just because they weren't equal.
//         //
//
//         if (NT_SUCCESS(Status)) {
//
//             RtlFreeOemString( &OemA );
//             RtlFreeOemString( &OemB );
//         }
// #endif
//
//         LongUniName = &Fcb->LongName.Unicode.Name.Unicode;
//
//         LongUniName->Length =
//         LongUniName->MaximumLength = Unicode.Length;
//         LongUniName->Buffer = Unicode.Buffer;
//
//         SetFlag(Fcb->FcbState, FCB_STATE_HAS_UNICODE_LONG_NAME);
//
//     try_exit: NOTHING;
//     } _SEH2_FINALLY {
//
//         if (_SEH2_AbnormalTermination()) {
//
//             if (ShortName->Buffer != NULL) {
//
//                 ExFreePool( ShortName->Buffer );
//                 ShortName->Buffer = NULL;
//             }
//
//         } else {
//
//             //
//             //  Creating all the names worked, so add all the names
//             //  to the splay tree.
//             //
//
//             FatInsertName( IrpContext,
//                            &Fcb->ParentDcb->Specific.Dcb.RootOemNode,
//                            &Fcb->ShortName );
//
//             Fcb->ShortName.Fcb = Fcb;
//
//             if (FlagOn(Fcb->FcbState, FCB_STATE_HAS_OEM_LONG_NAME)) {
//
//                 FatInsertName( IrpContext,
//                                &Fcb->ParentDcb->Specific.Dcb.RootOemNode,
//                                &Fcb->LongName.Oem );
//
//                 Fcb->LongName.Oem.Fcb = Fcb;
//             }
//
//             if (FlagOn(Fcb->FcbState, FCB_STATE_HAS_UNICODE_LONG_NAME)) {
//
//                 FatInsertName( IrpContext,
//                                &Fcb->ParentDcb->Specific.Dcb.RootUnicodeNode,
//                                &Fcb->LongName.Unicode );
//
//                 Fcb->LongName.Unicode.Fcb = Fcb;
//             }
//
//             SetFlag(Fcb->FcbState, FCB_STATE_NAMES_IN_SPLAY_TREE);
//         }
//     } _SEH2_END;
//
//     return;
// }
//
//
// _Requires_lock_held_(_Global_critical_region_)
// VOID
// FatCheckFreeDirentBitmap (
//     IN PIRP_CONTEXT IrpContext,
//     IN PDCB Dcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine checks if the size of the free dirent bitmap is
//     sufficient to for the current directory size.  It is called
//     whenever we grow a directory.
//
// Arguments:
//
//     Dcb -  Supplies the directory in question.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     ULONG OldNumberOfDirents;
//     ULONG NewNumberOfDirents;
//
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( IrpContext );
//
//     //
//     //  Setup the Bitmap buffer if it is not big enough already
//     //
//
//     NT_ASSERT( Dcb->Header.AllocationSize.QuadPart != FCB_LOOKUP_ALLOCATIONSIZE_HINT );
//
//     OldNumberOfDirents = Dcb->Specific.Dcb.FreeDirentBitmap.SizeOfBitMap;
//     NewNumberOfDirents = Dcb->Header.AllocationSize.LowPart / sizeof(DIRENT);
//
//     //
//     //  Do the usual unsync/sync check.
//     //
//
//     if (NewNumberOfDirents > OldNumberOfDirents) {
//
//         FatAcquireDirectoryFileMutex( Dcb->Vcb );
//
//         _SEH2_TRY {
//
//             PULONG OldBitmapBuffer;
//             PULONG BitmapBuffer;
//
//             ULONG BytesInBitmapBuffer;
//             ULONG BytesInOldBitmapBuffer;
//
//             OldNumberOfDirents = Dcb->Specific.Dcb.FreeDirentBitmap.SizeOfBitMap;
//             NewNumberOfDirents = Dcb->Header.AllocationSize.LowPart / sizeof(DIRENT);
//
//             if (NewNumberOfDirents > OldNumberOfDirents) {
//
//                 //
//                 //  Remember the old bitmap
//                 //
//
//                 OldBitmapBuffer = Dcb->Specific.Dcb.FreeDirentBitmap.Buffer;
//
//                 //
//                 //  Now make a new bitmap bufffer
//                 //
//
//                 BytesInBitmapBuffer = NewNumberOfDirents / 8;
//
//                 BytesInOldBitmapBuffer = OldNumberOfDirents / 8;
//
//                 if (DCB_UNION_SLACK_SPACE >= BytesInBitmapBuffer) {
//
//                     BitmapBuffer = &Dcb->Specific.Dcb.FreeDirentBitmapBuffer[0];
//
//                 } else {
//
//                     BitmapBuffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                              BytesInBitmapBuffer,
//                                                              TAG_DIRENT_BITMAP );
//                 }
//
//                 //
//                 //  Copy the old buffer to the new buffer, free the old one, and zero
//                 //  the rest of the new one.  Only do the first two steps though if
//                 //  we moved out of the initial buffer.
//                 //
//
//                 if ((OldNumberOfDirents != 0) &&
//                     (BitmapBuffer != &Dcb->Specific.Dcb.FreeDirentBitmapBuffer[0])) {
//
//                     RtlCopyMemory( BitmapBuffer,
//                                    OldBitmapBuffer,
//                                    BytesInOldBitmapBuffer );
//
//                     if (OldBitmapBuffer != &Dcb->Specific.Dcb.FreeDirentBitmapBuffer[0]) {
//
//                         ExFreePool( OldBitmapBuffer );
//                     }
//                 }
//
//                 NT_ASSERT( BytesInBitmapBuffer > BytesInOldBitmapBuffer );
//
//                 RtlZeroMemory( (PUCHAR)BitmapBuffer + BytesInOldBitmapBuffer,
//                                BytesInBitmapBuffer - BytesInOldBitmapBuffer );
//
//                 //
//                 //  Now initialize the new bitmap.
//                 //
//
//                 RtlInitializeBitMap( &Dcb->Specific.Dcb.FreeDirentBitmap,
//                                      BitmapBuffer,
//                                      NewNumberOfDirents );
//             }
//
//         } _SEH2_FINALLY {
//
//             FatReleaseDirectoryFileMutex( Dcb->Vcb );
//         } _SEH2_END;
//     }
// }
//
//
// BOOLEAN
// FatIsHandleCountZero (
//     IN PIRP_CONTEXT IrpContext,
//     IN PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine decides if the handle count on the volume is zero.
//
// Arguments:
//
//     Vcb - The volume in question
//
// Return Value:
//
//     BOOLEAN - TRUE if there are no open handles on the volume, FALSE
//               otherwise.
//
// --*/
//
// {
//     PFCB Fcb;
//
//     PAGED_CODE();
//
//     Fcb = Vcb->RootDcb;
//
//     while (Fcb != NULL) {
//
//         if (Fcb->UncleanCount != 0) {
//
//             return FALSE;
//         }
//
//         Fcb = FatGetNextFcbTopDown(IrpContext, Fcb, Vcb->RootDcb);
//     }
//
//     return TRUE;
// }
//
//
// PCLOSE_CONTEXT
//
// FatAllocateCloseContext(
//     OPTIONAL PVCB Vcb
//     )
// /*++
//
// Routine Description:
//
//     This routine preallocates a close context, presumeably on behalf
//     of a fileobject which does not have a structure we can embed one
//     in.
//
// Arguments:
//
//     None.
//
// Return Value:
//
//     None.
//
// --*/
// {
//     PAGED_CODE();
//     UNREFERENCED_PARAMETER( Vcb );
//
// #if DBG
//     if (ARGUMENT_PRESENT(Vcb)) {
//
//         NT_ASSERT( 0 != Vcb->CloseContextCount);
//         InterlockedDecrement( (LONG*)&Vcb->CloseContextCount);
//     }
// #endif
//     return (PCLOSE_CONTEXT)ExInterlockedPopEntrySList( &FatCloseContextSList,
//                                                        &FatData.GeneralSpinLock );
// }
//
//
// VOID
// FatPreallocateCloseContext (
//     PVCB Vcb
//     )
//
// /*++
//
// Routine Description:
//
//     This routine preallocates a close context, presumeably on behalf
//     of a fileobject which does not have a structure we can embed one
//     in.
//
// Arguments:
//
//     None.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     PCLOSE_CONTEXT CloseContext;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( Vcb );
//
//     CloseContext = FsRtlAllocatePoolWithTag( PagedPool,
//                                              sizeof(CLOSE_CONTEXT),
//                                              TAG_FAT_CLOSE_CONTEXT );
//
//     ExInterlockedPushEntrySList( &FatCloseContextSList,
//                                  (PSLIST_ENTRY) CloseContext,
//                                  &FatData.GeneralSpinLock );
//
//     DbgDoit( InterlockedIncrement( (LONG*)&Vcb->CloseContextCount));
// }
//
//
//
// VOID
// FatEnsureStringBufferEnough (
//      PVOID String,
//      USHORT DesiredBufferSize
//     )
//
// /*++
//
// Routine Description:
//
//     Ensures that a string string (STRING, UNICODE_STRING, ANSI_STRING, OEM_STRING)
//     has a buffer >= DesiredBufferSize,  allocating from pool if neccessary.  Any
//     existing pool buffer will be freed if a new one is allocated.
//
//     NOTE: No copy of old buffer contents is performed on reallocation.
//
//     Will raise on allocation failure.
//
// Arguments:
//
//     String - pointer to string structure
//
//     DesiredBufferSize - (bytes) minimum required buffer size
//
// --*/
//
// {
//     PSTRING LocalString = String;
//
//     PAGED_CODE();
//
//     if (LocalString->MaximumLength < DesiredBufferSize)  {
//
//         FatFreeStringBuffer( LocalString);
//
//         LocalString->Buffer = FsRtlAllocatePoolWithTag( PagedPool,
//                                                         DesiredBufferSize,
//                                                         TAG_DYNAMIC_NAME_BUFFER);
//         NT_ASSERT( LocalString->Buffer);
//
//         LocalString->MaximumLength = DesiredBufferSize;
//     }
// }
//
//
// VOID
// FatFreeStringBuffer (
//      PVOID String
//     )
//
// /*++
//
// Routine Description:
//
//     Frees the buffer of an string (STRING, UNICODE_STRING, ANSI_STRING, OEM_STRING)
//     structure if it is not within the current thread's stack limits.
//
//     Regardless of action performed,  on exit String->Buffer will be set to NULL and
//     String->MaximumLength to zero.
//
// Arguments:
//
//     String - pointer to string structure
//
// --*/
//
// {
//     ULONG_PTR High, Low;
//     PSTRING LocalString = String;
//
//     PAGED_CODE();
//
//     if (NULL != LocalString->Buffer)  {
//
//         IoGetStackLimits( &Low, &High );
//
//         if (((ULONG_PTR)(LocalString->Buffer) < Low) ||
//             ((ULONG_PTR)(LocalString->Buffer) > High))  {
//
//             ExFreePool( LocalString->Buffer);
//         }
//
//         LocalString->Buffer = NULL;
//     }
//
//     LocalString->MaximumLength = LocalString->Length = 0;
// }
//
//
// BOOLEAN
// FatScanForDataTrack(
//     IN PIRP_CONTEXT IrpContext,
//     IN PDEVICE_OBJECT TargetDeviceObject
//     )
//
// /*++
//
// Routine Description:
//
//     This routine is called to verify and process the TOC for this disk.
//
//     FAT queries for the TOC to avoid trying to mount on CD-DA/CD-E media,  Doing data reads on
//     audio/leadin of that media sends a lot of drives into what could charitably be called
//     "conniptions" which take a couple seconds to clear and would also convince FAT that the
//     device was busted, and fail the mount (not letting CDFS get its crack).
//
//     There is special handling of PD media.  These things fail the TOC read, but return
//     a special error code so FAT knows to continue to try the mount anyway.
//
// Arguments:
//
//     TargetDeviceObject - Device object to send TOC request to.
//
// Return Value:
//
//     BOOLEAN - TRUE if we found a TOC with a single data track.
//
// --*/
//
// {
//     NTSTATUS Status;
//     IO_STATUS_BLOCK Iosb;
//
//     ULONG LocalTrackCount;
//     ULONG LocalTocLength;
//
//     PCDROM_TOC CdromToc;
//     BOOLEAN Result = FALSE;
//
//     PAGED_CODE();
//
//     CdromToc = FsRtlAllocatePoolWithTag( PagedPool,
//                                          sizeof( CDROM_TOC ),
//                                          TAG_IO_BUFFER );
//
//     RtlZeroMemory( CdromToc, sizeof( CDROM_TOC ));
//
//     _SEH2_TRY {
//
//         //
//         //  Go ahead and read the table of contents
//         //
//
//         Status = FatPerformDevIoCtrl( IrpContext,
//                                      IOCTL_CDROM_READ_TOC,
//                                      TargetDeviceObject,
//                                      NULL,
//                                      0,
//                                      CdromToc,
//                                      sizeof( CDROM_TOC ),
//                                      FALSE,
//                                      TRUE,
//                                      &Iosb );
//
//         //
//         //  Nothing to process if this request fails.
//         //
//
//         if (Status != STATUS_SUCCESS) {
//
//             //
//             //  If we get the special error indicating a failed TOC read on PD media just
//             //  plow ahead with the mount (see comments above).
//             //
//
//             if ((Status == STATUS_IO_DEVICE_ERROR) || (Status == STATUS_INVALID_DEVICE_REQUEST)) {
//
//                 Result = TRUE;
//
//             }
//
//             try_leave( NOTHING );
//         }
//
//         //
//         //  Get the number of tracks and stated size of this structure.
//         //
//
//         LocalTrackCount = CdromToc->LastTrack - CdromToc->FirstTrack + 1;
//         LocalTocLength = PtrOffset( CdromToc, &CdromToc->TrackData[LocalTrackCount + 1] );
//
//         //
//         //  Get out if there is an immediate problem with the TOC,  or more than
//         //  one track.
//         //
//
//         if ((LocalTocLength > Iosb.Information) ||
//             (CdromToc->FirstTrack > CdromToc->LastTrack) ||
//             (LocalTrackCount != 1)) {
//
//             try_leave( NOTHING);
//         }
//
//         //
//         //  Is it a data track?  DVD-RAM reports single,  data,  track.
//         //
//
//         Result = BooleanFlagOn( CdromToc->TrackData[ 0].Control, 0x04 );
//     }
//     _SEH2_FINALLY {
//
//         ExFreePool( CdromToc);
//     } _SEH2_END;
//
//     return Result;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
