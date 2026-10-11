use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ps::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ERESOURCE, ExDeleteNPagedLookasideList, ExDeleteResourceLite, ExInitializeFastMutex, ExInitializeNPagedLookasideList, ExInitializeResourceLite, FAST_IO_DISPATCH, FS_FILTER_CALLBACKS, IoAllocateWorkItem, IoCreateDevice, IoDeleteDevice, IoFreeWorkItem, IoRegisterFileSystem, KeInitializeEvent, KeInitializeSpinLock, MmQuerySystemSize, _OBJECT_ATTRIBUTES, PDRIVER_DISPATCH, PDRIVER_OBJECT, PKEY_VALUE_FULL_INFORMATION, PUNICODE_STRING, RtlInitUnicodeString, _EVENT_TYPE_SynchronizationEvent};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::cleanup::{FatFsdCleanup};
use crate::close::{FatFsdClose};
use crate::create::{FatFsdCreate};
use crate::devctrl::{FatFsdDeviceControl};
use crate::dirctrl::{FatFsdDirectoryControl};
use crate::ea::{FatFsdSetEa, FatFsdQueryEa};
use crate::fatdata::{FatFastIoCheckIfPossible, FatFastQueryBasicInfo, FatFastQueryNetworkOpenInfo, FatFastQueryStdInfo};
use crate::fileinfo::{FatFsdSetInformation, FatFsdQueryInformation};
use crate::flush::{FatFsdFlushBuffers};
use crate::lockctrl::{FatFastLock, FatFastUnlockAll, FatFastUnlockAllByKey, FatFastUnlockSingle, FatFsdLockControl};
use crate::pnp::{FatFsdPnp};
use crate::read::{FatFsdRead};
use crate::resrcsup::{FatAcquireFcbForLazyWrite, FatAcquireFcbForReadAhead, FatAcquireForCcFlush, FatFilterCallbackAcquireForCreateSection, FatNoOpAcquire, FatNoOpRelease, FatReleaseFcbFromLazyWrite, FatReleaseFcbFromReadAhead, FatReleaseForCcFlush};
use crate::shutdown::{FatFsdShutdown};
use crate::types::{FAT_DATA, FatCdromFileSystemDeviceObject, FatCloseContextSList, FatCloseQueueMutex, FatData, FatDiskFileSystemDeviceObject, FatEResourceLookasideList, FatFastIoDispatch, FatIrpContextLookasideList, FatMaxDelayedCloseCount, FatNonPagedFcbLookasideList, FatReserveEvent, IRP_CONTEXT, NODE_BYTE_SIZE, NON_PAGED_FCB, PVOLUME_DEVICE_OBJECT};
use crate::volinfo::{FatFsdSetVolumeInformation, FatFsdQueryVolumeInformation};
use crate::write::{FatFsdWrite};
use crate::fsctrl::{FatFsdFileSystemControl};
use ntkrnl_sys::ddk::{KEY_VALUE_FULL_INFORMATION, PACQUIRE_FOR_LAZY_WRITE, PACQUIRE_FOR_READ_AHEAD, PDEVICE_OBJECT, PFAST_IO_ACQUIRE_FOR_CCFLUSH, PFAST_IO_CHECK_IF_POSSIBLE, PFAST_IO_LOCK, PFAST_IO_MDL_READ, PFAST_IO_MDL_READ_COMPLETE, PFAST_IO_MDL_WRITE_COMPLETE, PFAST_IO_PREPARE_MDL_WRITE, PFAST_IO_QUERY_BASIC_INFO, PFAST_IO_QUERY_NETWORK_OPEN_INFO, PFAST_IO_QUERY_STANDARD_INFO, PFAST_IO_READ, PFAST_IO_RELEASE_FOR_CCFLUSH, PFAST_IO_UNLOCK_ALL, PFAST_IO_UNLOCK_ALL_BY_KEY, PFAST_IO_UNLOCK_SINGLE, PFAST_IO_WRITE, PFILE_BASIC_INFORMATION, PFILE_NETWORK_OPEN_INFORMATION, PFILE_OBJECT, PFILE_STANDARD_INFORMATION, PFS_FILTER_CALLBACK, PFS_FILTER_CALLBACK_DATA, PIO_STATUS_BLOCK, PIRP, PLARGE_INTEGER, PMDL, PRELEASE_FROM_LAZY_WRITE, PRELEASE_FROM_READ_AHEAD, _DRIVER_OBJECT, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{FAT_MAX_DELAYED_CLOSES, FAT_NTC_DATA_HEADER, IRP_MJ_DIRECTORY_CONTROL, IRP_MJ_QUERY_EA, IRP_MJ_QUERY_VOLUME_INFORMATION, IRP_MJ_SET_EA, IRP_MJ_SET_VOLUME_INFORMATION, TAG_ERESOURCE, TAG_FCB_NONPAGED, TAG_IRP_CONTEXT};
// resolve_imports.py: explicit kernel/private-type imports
use crate::ddk_shims::{FsRtlCopyRead, FsRtlCopyWrite, FsRtlMdlReadCompleteDev, FsRtlMdlReadDev, FsRtlMdlWriteCompleteDev, FsRtlPrepareMdlWriteDev, FsRtlRegisterFileSystemFilterCallbacks};

// KeNumberProcessors is a kernel DATA export — raw-dylib routes the reference
// through the __imp_ thunk instead of expecting a local definition.
// NOTE: the kernel exports it as CCHAR (1 byte), NOT ULONG. Declaring it ULONG
// reads 3 adjacent garbage bytes (e.g. count 1 -> 0x000C0001), which blew up the
// per-processor stats allocation to 0x6000080 bytes -> STATUS_INSUFFICIENT_RESOURCES.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static KeNumberProcessors: core::ffi::c_char;
}
extern "C" { fn DbgPrint(format: *const u8, ...) -> i32; }
extern "system" {
    fn ZwOpenKey( KeyHandle: *mut *mut core::ffi::c_void, DesiredAccess: u32, ObjectAttributes: *mut _OBJECT_ATTRIBUTES, ) -> i32;
    fn ZwQueryValueKey( KeyHandle: *mut core::ffi::c_void, ValueName: *mut _UNICODE_STRING, KeyValueInformationClass: u32, KeyValueInformation: *mut core::ffi::c_void, Length: u32, ResultLength: *mut u32, ) -> i32;
    fn ExAllocatePoolWithTag( PoolType: u32, NumberOfBytes: usize, Tag: u32, ) -> *mut core::ffi::c_void;
    fn ExFreePool(P: *mut core::ffi::c_void);
    fn ZwClose(Handle: *mut core::ffi::c_void);
    fn RtlCompareMemory( Buffer1: *const core::ffi::c_void, Buffer2: *const core::ffi::c_void, Length: usize, ) -> usize;
}
// ObfDereferenceObject is fastcall on i386, C on amd64/arm64
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ObfDereferenceObject(p: *mut core::ffi::c_void) -> isize;
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn ObfDereferenceObject(p: *mut core::ffi::c_void) -> isize;
}
// ExInitializeSListHead is FORCEINLINE — implement inline
#[inline]
unsafe fn ExInitializeSListHead(slist_head: *mut SLIST_HEADER) {
    core::ptr::write_bytes(slist_head as *mut u8, 0, core::mem::size_of::<SLIST_HEADER>());
}
// __int2c is a compiler intrinsic (INT 0x2C — DbgRaiseAssertionFailure, raises
// STATUS_ASSERTION_FAILURE) — implement inline. INT 0x2C is a valid instruction on
// BOTH x86 arches (i386 + amd64); arm64 uses `brk`. The old non-x86 body was
// unreachable_unchecked() — UB — which is wrong because __int2c IS reachable
// (assertion paths in fatinit.rs:532 / workque.rs:195,205), so on amd64/arm64 it
// invoked UB instead of a defined assertion trap.
#[inline(never)]
unsafe fn __int2c() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("int 0x2c", options(nostack, nomem));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("brk #0xf000", options(nostack, nomem));
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const COMPATIBILITY_MODE_KEY_NAME: [::core::ffi::c_ushort; 62] = unsafe {
    ::core::mem::transmute::<
        [u8; 124],
        [::core::ffi::c_ushort; 62],
    >(
        *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0F\0i\0l\0e\0S\0y\0s\0t\0e\0m\0\0\0",
    )
};

pub const COMPATIBILITY_MODE_VALUE_NAME: [::core::ffi::c_ushort; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_ushort; 16]>(
        *b"W\0i\0n\x003\x001\0F\0i\0l\0e\0S\0y\0s\0t\0e\0m\0\0\0",
    )
};

pub const CODE_PAGE_INVARIANCE_VALUE_NAME: [::core::ffi::c_ushort; 29] = unsafe {
    ::core::mem::transmute::<
        [u8; 58],
        [::core::ffi::c_ushort; 29],
    >(
        *b"F\0a\0t\0D\0i\0s\0a\0b\0l\0e\0C\0o\0d\0e\0P\0a\0g\0e\0I\0n\0v\0a\0r\0i\0a\0n\0c\0e\0\0\0",
    )
};

pub const REGISTRY_HARDWARE_DESCRIPTION_W: [::core::ffi::c_ushort; 46] = unsafe {
    ::core::mem::transmute::<
        [u8; 92],
        [::core::ffi::c_ushort; 46],
    >(
        *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0H\0a\0r\0d\0w\0a\0r\0e\0\\\0D\0E\0S\0C\0R\0I\0P\0T\0I\0O\0N\0\\\0S\0y\0s\0t\0e\0m\0\0\0",
    )
};

pub const REGISTRY_MACHINE_IDENTIFIER_W: [::core::ffi::c_ushort; 11] = unsafe {
    ::core::mem::transmute::<[u8; 22], [::core::ffi::c_ushort; 11]>(
        *b"I\0d\0e\0n\0t\0i\0f\0i\0e\0r\0\0\0",
    )
};

pub const FUJITSU_FMR_NAME_W: [::core::ffi::c_ushort; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_ushort; 13]>(
        *b"F\0U\0J\0I\0T\0S\0U\0 \0F\0M\0R\0-\0\0\0",
    )
};

pub const KEY_WORK_AREA: usize = core::mem::size_of::<KEY_VALUE_FULL_INFORMATION>() + 4 + 64 ;



type PULONG = *mut u32;
type PWCH = *mut u16;

const KEY_VALUE_FULL_INFORMATION_CLASS_FULL: u32 = 1;

const FILE_DEVICE_DISK_FILE_SYSTEM: ULONG = 0x0008;
const FILE_DEVICE_CD_ROM_FILE_SYSTEM: ULONG = 0x0003;
static FAT_DISK_DEVICE_NAME: [u16; 5] = [0x5C, 0x46, 0x61, 0x74, 0x00]; // L"\\Fat"
static FAT_CDROM_DEVICE_NAME: [u16; 10] = [0x5C, 0x46, 0x61, 0x74, 0x43, 0x64, 0x72, 0x6F, 0x6D, 0x00]; // L"\\FatCdrom"
// fatprocs.h:47: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNx: ULONG = 0;
const POOL_NX_ALLOCATION: ULONG = 0; // fatprocs.h downgrades it to 0 under __REACTOS__
const POOL_RAISE_IF_ALLOCATION_FAILURE: ULONG = 0x10;
const PAGE_SIZE: SIZE_T = 0x1000;
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut MaxDepth: USHORT;
    let mut Status: NTSTATUS;
    let mut UnicodeString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut FilterCallbacks: FS_FILTER_CALLBACKS = FS_FILTER_CALLBACKS {
        SizeOfFsFilterCallbacks: 0,
        Reserved: 0,
        PreAcquireForSectionSynchronization: None,
        PostAcquireForSectionSynchronization: None,
        PreReleaseForSectionSynchronization: None,
        PostReleaseForSectionSynchronization: None,
        PreAcquireForCcFlush: None,
        PostAcquireForCcFlush: None,
        PreReleaseForCcFlush: None,
        PostReleaseForCcFlush: None,
        PreAcquireForModifiedPageWriter: None,
        PostAcquireForModifiedPageWriter: None,
        PreReleaseForModifiedPageWriter: None,
        PostReleaseForModifiedPageWriter: None,
    };
    let mut ValueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Value: ULONG = 0;

    // UNREFERENCED_PARAMETER(RegistryPath)
    let _ = RegistryPath;

    // Create device object for disks
    RtlInitUnicodeString(
        &mut UnicodeString,
        FAT_DISK_DEVICE_NAME.as_ptr(),
    );
    Status = IoCreateDevice(
        DriverObject,
        0,
        &mut UnicodeString,
        FILE_DEVICE_DISK_FILE_SYSTEM,
        0,
        0, // FALSE
        &raw mut FatDiskFileSystemDeviceObject,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // Create device object for cdroms
    RtlInitUnicodeString(
        &mut UnicodeString,
        FAT_CDROM_DEVICE_NAME.as_ptr(),
    );
    Status = IoCreateDevice(
        DriverObject,
        0,
        &mut UnicodeString,
        FILE_DEVICE_CD_ROM_FILE_SYSTEM,
        0,
        0, // FALSE
        &raw mut FatCdromFileSystemDeviceObject,
    );
    if !NT_SUCCESS(Status) {
        IoDeleteDevice(FatDiskFileSystemDeviceObject);
        return Status;
    }

    // Driver unload and dispatch setup
    (*DriverObject).DriverUnload =
        core::mem::transmute::<*const (), ntkrnl_sys::ddk::PDRIVER_UNLOAD>(FatUnload as *const ());

    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdCreate as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdClose as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdRead as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_WRITE as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdWrite as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_INFORMATION as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdQueryInformation as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_SET_INFORMATION as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdSetInformation as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_EA as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdQueryEa as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_SET_EA as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdSetEa as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_FLUSH_BUFFERS as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdFlushBuffers as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_VOLUME_INFORMATION as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdQueryVolumeInformation as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_SET_VOLUME_INFORMATION as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdSetVolumeInformation as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_CLEANUP as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdCleanup as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_DIRECTORY_CONTROL as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdDirectoryControl as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_FILE_SYSTEM_CONTROL as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdFileSystemControl as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_LOCK_CONTROL as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdLockControl as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdDeviceControl as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_SHUTDOWN as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdShutdown as *const ());
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] =
        core::mem::transmute::<*const (), PDRIVER_DISPATCH>(FatFsdPnp as *const ());

    // Fast I/O dispatch
    (*DriverObject).FastIoDispatch = core::ptr::addr_of_mut!(FatFastIoDispatch);
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(FatFastIoDispatch) as *mut u8,
        0,
        core::mem::size_of::<FAST_IO_DISPATCH>(),
    );
    FatFastIoDispatch.SizeOfFastIoDispatch = core::mem::size_of::<FAST_IO_DISPATCH>() as ULONG;

    FatFastIoDispatch.FastIoCheckIfPossible =
        core::mem::transmute::<*const (), PFAST_IO_CHECK_IF_POSSIBLE>(FatFastIoCheckIfPossible as *const ());
    FatFastIoDispatch.FastIoRead =
        core::mem::transmute::<*const (), PFAST_IO_READ>(FsRtlCopyRead as *const ());
    FatFastIoDispatch.FastIoWrite =
        core::mem::transmute::<*const (), PFAST_IO_WRITE>(FsRtlCopyWrite as *const ());
    FatFastIoDispatch.FastIoQueryBasicInfo =
        core::mem::transmute::<*const (), PFAST_IO_QUERY_BASIC_INFO>(FatFastQueryBasicInfo as *const ());
    FatFastIoDispatch.FastIoQueryStandardInfo =
        core::mem::transmute::<*const (), PFAST_IO_QUERY_STANDARD_INFO>(FatFastQueryStdInfo as *const ());
    FatFastIoDispatch.FastIoLock =
        core::mem::transmute::<*const (), PFAST_IO_LOCK>(FatFastLock as *const ());
    FatFastIoDispatch.FastIoUnlockSingle =
        core::mem::transmute::<*const (), PFAST_IO_UNLOCK_SINGLE>(FatFastUnlockSingle as *const ());
    FatFastIoDispatch.FastIoUnlockAll =
        core::mem::transmute::<*const (), PFAST_IO_UNLOCK_ALL>(FatFastUnlockAll as *const ());
    FatFastIoDispatch.FastIoUnlockAllByKey =
        core::mem::transmute::<*const (), PFAST_IO_UNLOCK_ALL_BY_KEY>(FatFastUnlockAllByKey as *const ());
    FatFastIoDispatch.FastIoQueryNetworkOpenInfo =
        core::mem::transmute::<*const (), PFAST_IO_QUERY_NETWORK_OPEN_INFO>(FatFastQueryNetworkOpenInfo as *const ());
    FatFastIoDispatch.AcquireForCcFlush =
        core::mem::transmute::<*const (), PFAST_IO_ACQUIRE_FOR_CCFLUSH>(FatAcquireForCcFlush as *const ());
    FatFastIoDispatch.ReleaseForCcFlush =
        core::mem::transmute::<*const (), PFAST_IO_RELEASE_FOR_CCFLUSH>(FatReleaseForCcFlush as *const ());
    FatFastIoDispatch.MdlRead =
        core::mem::transmute::<*const (), PFAST_IO_MDL_READ>(FsRtlMdlReadDev as *const ());
    FatFastIoDispatch.MdlReadComplete =
        core::mem::transmute::<*const (), PFAST_IO_MDL_READ_COMPLETE>(FsRtlMdlReadCompleteDev as *const ());
    FatFastIoDispatch.PrepareMdlWrite =
        core::mem::transmute::<*const (), PFAST_IO_PREPARE_MDL_WRITE>(FsRtlPrepareMdlWriteDev as *const ());
    FatFastIoDispatch.MdlWriteComplete =
        core::mem::transmute::<*const (), PFAST_IO_MDL_WRITE_COMPLETE>(FsRtlMdlWriteCompleteDev as *const ());

    // Initialize filter callbacks
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(FilterCallbacks) as *mut u8,
        0,
        core::mem::size_of::<FS_FILTER_CALLBACKS>(),
    );
    FilterCallbacks.SizeOfFsFilterCallbacks = core::mem::size_of::<FS_FILTER_CALLBACKS>() as ULONG;
    FilterCallbacks.PreAcquireForSectionSynchronization =
        core::mem::transmute::<*const (), PFS_FILTER_CALLBACK>(FatFilterCallbackAcquireForCreateSection as *const ());

    Status = FsRtlRegisterFileSystemFilterCallbacks(DriverObject as *mut _DRIVER_OBJECT, &mut FilterCallbacks);
    if !NT_SUCCESS(Status) {
        IoDeleteDevice(FatDiskFileSystemDeviceObject);
        IoDeleteDevice(FatCdromFileSystemDeviceObject);
        return Status;
    }

    // Initialize global data structures
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(FatData) as *mut u8,
        0,
        core::mem::size_of::<FAT_DATA>(),
    );
    FatData.NodeTypeCode = (FAT_NTC_DATA_HEADER) as u16;
    FatData.NodeByteSize = core::mem::size_of::<FAT_DATA>() as NODE_BYTE_SIZE;
    InitializeListHead(&raw mut FatData.VcbQueue);
    FatData.DriverObject = DriverObject;
    FatData.DiskFileSystemDeviceObject = FatDiskFileSystemDeviceObject as PVOID;
    FatData.CdromFileSystemDeviceObject = FatCdromFileSystemDeviceObject as PVOID;

    InitializeListHead(&raw mut FatData.AsyncCloseList);
    InitializeListHead(&raw mut FatData.DelayedCloseList);

    FatData.FatCloseItem = IoAllocateWorkItem(FatDiskFileSystemDeviceObject);
    if FatData.FatCloseItem.is_null() {
        IoDeleteDevice(FatDiskFileSystemDeviceObject);
        IoDeleteDevice(FatCdromFileSystemDeviceObject);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    FatData.ZeroPage = ExAllocatePoolWithTag(NonPagedPoolNx, PAGE_SIZE as SIZE_T, 1517576518i32 as ULONG);
    if FatData.ZeroPage.is_null() {
        IoDeleteDevice(FatDiskFileSystemDeviceObject);
        IoDeleteDevice(FatCdromFileSystemDeviceObject);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::write_bytes(FatData.ZeroPage as *mut u8, 0, 0x1000);

    KeInitializeSpinLock(&raw mut FatData.GeneralSpinLock);

    match MmQuerySystemSize() as u32 {
        0 => {
            MaxDepth = 4;
            FatMaxDelayedCloseCount = (FAT_MAX_DELAYED_CLOSES) as u32;
        }
        1 => {
            MaxDepth = 8;
            FatMaxDelayedCloseCount = (4 as ULONG).wrapping_mul((FAT_MAX_DELAYED_CLOSES) as u32);
        }
        _ => {
            MaxDepth = 16;
            FatMaxDelayedCloseCount = (16 as ULONG).wrapping_mul((FAT_MAX_DELAYED_CLOSES) as u32);
        }
    }

    FatData.CacheManagerCallbacks.AcquireForLazyWrite =
        core::mem::transmute::<*const (), PACQUIRE_FOR_LAZY_WRITE>(FatAcquireFcbForLazyWrite as *const ());
    FatData.CacheManagerCallbacks.ReleaseFromLazyWrite =
        core::mem::transmute::<*const (), PRELEASE_FROM_LAZY_WRITE>(FatReleaseFcbFromLazyWrite as *const ());
    FatData.CacheManagerCallbacks.AcquireForReadAhead =
        core::mem::transmute::<*const (), PACQUIRE_FOR_READ_AHEAD>(FatAcquireFcbForReadAhead as *const ());
    FatData.CacheManagerCallbacks.ReleaseFromReadAhead =
        core::mem::transmute::<*const (), PRELEASE_FROM_READ_AHEAD>(FatReleaseFcbFromReadAhead as *const ());

    FatData.CacheManagerNoOpCallbacks.AcquireForLazyWrite =
        core::mem::transmute::<*const (), PACQUIRE_FOR_LAZY_WRITE>(FatNoOpAcquire as *const ());
    FatData.CacheManagerNoOpCallbacks.ReleaseFromLazyWrite =
        core::mem::transmute::<*const (), PRELEASE_FROM_LAZY_WRITE>(FatNoOpRelease as *const ());
    FatData.CacheManagerNoOpCallbacks.AcquireForReadAhead =
        core::mem::transmute::<*const (), PACQUIRE_FOR_READ_AHEAD>(FatNoOpAcquire as *const ());
    FatData.CacheManagerNoOpCallbacks.ReleaseFromReadAhead =
        core::mem::transmute::<*const (), PRELEASE_FROM_READ_AHEAD>(FatNoOpRelease as *const ());

    FatData.OurProcess = (PsGetCurrentProcess()).cast();
    FatData.NumberProcessors = (KeNumberProcessors as u8) as ULONG;

    // Registry: ChicagoMode
    ValueName.Buffer = COMPATIBILITY_MODE_VALUE_NAME.as_ptr() as PWCH;
    ValueName.Length = (core::mem::size_of::<[u16; 16]>().wrapping_sub(core::mem::size_of::<WCHAR>())) as USHORT;
    ValueName.MaximumLength = core::mem::size_of::<[u16; 16]>() as USHORT;

    Status = FatGetCompatibilityModeValue(&mut ValueName, &mut Value);
    if NT_SUCCESS(Status) && (Value & 1) != 0 {
        (*&raw mut FatData).set_ChicagoMode(0); // FALSE
    } else {
        (*&raw mut FatData).set_ChicagoMode(1); // TRUE
    }

    // Registry: CodePageInvariance
    ValueName.Buffer = CODE_PAGE_INVARIANCE_VALUE_NAME.as_ptr() as PWCH;
    ValueName.Length = (core::mem::size_of::<[u16; 29]>().wrapping_sub(core::mem::size_of::<WCHAR>())) as USHORT;
    ValueName.MaximumLength = core::mem::size_of::<[u16; 29]>() as USHORT;

    Status = FatGetCompatibilityModeValue(&mut ValueName, &mut Value);
    if NT_SUCCESS(Status) && (Value & 1) != 0 {
        (*&raw mut FatData).set_CodePageInvariant(0); // FALSE
    } else {
        (*&raw mut FatData).set_CodePageInvariant(1); // TRUE
    }

    ExInitializeResourceLite(&raw mut FatData.Resource);

    ExInitializeNPagedLookasideList(
        &raw mut FatIrpContextLookasideList,
        None,
        None,
        POOL_NX_ALLOCATION | POOL_RAISE_IF_ALLOCATION_FAILURE,
        core::mem::size_of::<IRP_CONTEXT>() as _,
        TAG_IRP_CONTEXT as ULONG,
        MaxDepth,
    );
    ExInitializeNPagedLookasideList(
        &raw mut FatNonPagedFcbLookasideList,
        None,
        None,
        POOL_NX_ALLOCATION | POOL_RAISE_IF_ALLOCATION_FAILURE,
        core::mem::size_of::<NON_PAGED_FCB>() as _,
        TAG_FCB_NONPAGED as ULONG,
        MaxDepth,
    );
    ExInitializeNPagedLookasideList(
        &raw mut FatEResourceLookasideList,
        None,
        None,
        POOL_NX_ALLOCATION | POOL_RAISE_IF_ALLOCATION_FAILURE,
        core::mem::size_of::<ERESOURCE>() as _,
        TAG_ERESOURCE as ULONG,
        MaxDepth,
    );

    ExInitializeSListHead(core::ptr::addr_of_mut!(FatCloseContextSList) as *mut SLIST_HEADER);
    ExInitializeFastMutex(&raw mut FatCloseQueueMutex);
    KeInitializeEvent(&raw mut FatReserveEvent, _EVENT_TYPE_SynchronizationEvent, 1); // TRUE

    IoRegisterFileSystem(FatDiskFileSystemDeviceObject);
    ObReferenceObject(FatDiskFileSystemDeviceObject as PVOID);
    IoRegisterFileSystem(FatCdromFileSystemDeviceObject);
    ObReferenceObject(FatCdromFileSystemDeviceObject as PVOID);

    (*&raw mut FatData).set_FujitsuFMR(FatIsFujitsuFMR() as BOOLEAN);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FatUnload(DriverObject: PDRIVER_OBJECT) {
    unsafe {
        ExDeleteNPagedLookasideList(&raw mut FatEResourceLookasideList);
        ExDeleteNPagedLookasideList(&raw mut FatNonPagedFcbLookasideList);
        ExDeleteNPagedLookasideList(&raw mut FatIrpContextLookasideList);
        ExDeleteResourceLite(&raw mut FatData.Resource);
        IoFreeWorkItem(FatData.FatCloseItem);
        ObfDereferenceObject(FatDiskFileSystemDeviceObject as *mut core::ffi::c_void);
        ObfDereferenceObject(FatCdromFileSystemDeviceObject as *mut core::ffi::c_void);
    }
}
#[no_mangle]
pub unsafe extern "system" fn FatGetCompatibilityModeValue(
    ValueName: PUNICODE_STRING,
    Value: PULONG,
) -> NTSTATUS {
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut RequestLength: ULONG = 0;
    let mut ResultLength: ULONG = 0;
    let mut Buffer: [u8; KEY_WORK_AREA] = [0u8; KEY_WORK_AREA];
    let mut KeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut KeyValueInformation: *mut KEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();

    KeyName.Buffer = COMPATIBILITY_MODE_KEY_NAME.as_ptr() as PWCH;
    KeyName.Length = (core::mem::size_of::<[u16; 62]>() as u32)
        .wrapping_sub(core::mem::size_of::<u16>() as u32) as u16;
    KeyName.MaximumLength = core::mem::size_of::<[u16; 62]>() as u16;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &raw mut KeyName as PUNICODE_STRING;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenKey(
        &raw mut Handle,
        KEY_READ as u32,
        &raw mut ObjectAttributes,
    );
    if !(Status >= 0) {
        return Status;
    }

    RequestLength = KEY_WORK_AREA as ULONG;
    KeyValueInformation = &raw mut Buffer as *mut u8 as *mut KEY_VALUE_FULL_INFORMATION;

    loop {
        Status = ZwQueryValueKey(
            Handle,
            ValueName,
            KEY_VALUE_FULL_INFORMATION_CLASS_FULL,
            KeyValueInformation as *mut core::ffi::c_void,
            RequestLength,
            &raw mut ResultLength,
        );
        if !(Status != STATUS_BUFFER_OVERFLOW) {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const u8,
                b"drivers/filesystems/fastfat/fatinit.c\0"
                    .as_ptr() as *const u8,
                553 as core::ffi::c_int,
                b"Status != STATUS_BUFFER_OVERFLOW\0".as_ptr() as *const u8,
            );
            __int2c();
        }
        if !(Status == STATUS_BUFFER_OVERFLOW) {
            break;
        }
        if KeyValueInformation != &raw mut Buffer as *mut u8 as *mut KEY_VALUE_FULL_INFORMATION {
            ExFreePool(KeyValueInformation as *mut core::ffi::c_void);
        }
        RequestLength = (RequestLength as u32).wrapping_add(256 as u32);
        KeyValueInformation = ExAllocatePoolWithTag(
            PagedPool,
            RequestLength as usize,
            (0x20746146i32) as u32,
        ) as *mut KEY_VALUE_FULL_INFORMATION;
        if KeyValueInformation.is_null() {
            ZwClose(Handle);
            return STATUS_NO_MEMORY;
        }
    }

    ZwClose(Handle);

    if Status >= 0 {
        if (*KeyValueInformation).DataLength != 0 {
            let DataPtr: *mut u32 = ((KeyValueInformation as *mut u8)
                .offset((*KeyValueInformation).DataOffset as isize)) as *mut u32;
            *Value = *DataPtr;
        } else {
            Status = STATUS_OBJECT_NAME_NOT_FOUND;
        }
    }

    if KeyValueInformation != &raw mut Buffer as *mut u8 as *mut KEY_VALUE_FULL_INFORMATION {
        ExFreePool(KeyValueInformation as *mut core::ffi::c_void);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn FatIsFujitsuFMR() -> i32 {
    let mut Result: i32 = 0;
    let mut Handle: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut Status: i32 = 0;
    let mut RequestLength: u32 = 0;
    let mut ResultLength: u32 = 0;
    let mut Buffer: [u8; 92] = [0; 92];
    let mut KeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ValueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut KeyValueInformation: *mut KEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();

    // L"\\Registry\\Machine\\Hardware\\DESCRIPTION\\System"
    let reg_hw_desc: [u16; 46] = [
        0x5C, 0x52, 0x65, 0x67, 0x69, 0x73, 0x74, 0x72, 0x79,
        0x5C, 0x4D, 0x61, 0x63, 0x68, 0x69, 0x6E, 0x65,
        0x5C, 0x48, 0x61, 0x72, 0x64, 0x77, 0x61, 0x72, 0x65,
        0x5C, 0x44, 0x45, 0x53, 0x43, 0x52, 0x49, 0x50, 0x54, 0x49, 0x4F, 0x4E,
        0x5C, 0x53, 0x79, 0x73, 0x74, 0x65, 0x6D,
        0,
    ];
    KeyName.Buffer = reg_hw_desc.as_ptr() as *mut u16;
    KeyName.Length = (core::mem::size_of_val(&reg_hw_desc) - core::mem::size_of::<u16>()) as u16;
    KeyName.MaximumLength = core::mem::size_of_val(&reg_hw_desc) as u16;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &raw mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenKey(
        &mut Handle,
        KEY_READ as u32,
        &mut ObjectAttributes,
    );

    if !NT_SUCCESS(Status) {
        return 0;
    }

    // L"Identifier"
    let machine_id: [u16; 11] = [
        0x49, 0x64, 0x65, 0x6E, 0x74, 0x69, 0x66, 0x69, 0x65, 0x72,
        0,
    ];
    ValueName.Buffer = machine_id.as_ptr() as *mut u16;
    ValueName.Length = (core::mem::size_of_val(&machine_id) - core::mem::size_of::<u16>()) as u16;
    ValueName.MaximumLength = core::mem::size_of_val(&machine_id) as u16;

    RequestLength = 92u32; // KEY_WORK_AREA
    KeyValueInformation = &mut Buffer as *mut u8 as *mut KEY_VALUE_FULL_INFORMATION;

    loop {
        Status = ZwQueryValueKey(
            Handle,
            &mut ValueName,
            KEY_VALUE_FULL_INFORMATION_CLASS_FULL,
            KeyValueInformation as *mut core::ffi::c_void,
            RequestLength,
            &mut ResultLength,
        );

        if Status != STATUS_BUFFER_OVERFLOW {
            break;
        }

        if KeyValueInformation != (&mut Buffer as *mut u8 as *mut KEY_VALUE_FULL_INFORMATION) {
            ExFreePool(KeyValueInformation as *mut core::ffi::c_void);
        }

        RequestLength = RequestLength.wrapping_add(256);
        KeyValueInformation = ExAllocatePoolWithTag(
            PagedPool,
            RequestLength as usize,
            0x20746146u32, // ' taF'
        ) as *mut KEY_VALUE_FULL_INFORMATION;

        if KeyValueInformation.is_null() {
            ZwClose(Handle);
            return 0;
        }
    }

    ZwClose(Handle);

    // L"FUJITSU FMR-"
    let fujitsu_fmr: [u16; 13] = [
        0x46, 0x55, 0x4A, 0x49, 0x54, 0x53, 0x55, 0x20, 0x46, 0x4D, 0x52, 0x2D,
        0,
    ];

    if NT_SUCCESS(Status)
        && (*KeyValueInformation).DataLength as usize >= core::mem::size_of::<[u16; 13]>()
        && RtlCompareMemory(
            (KeyValueInformation as *mut u8).add((*KeyValueInformation).DataOffset as usize)
                as *const core::ffi::c_void,
            fujitsu_fmr.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of::<[u16; 13]>() - core::mem::size_of::<u16>(),
        ) == core::mem::size_of::<[u16; 13]>() - core::mem::size_of::<u16>()
    {
        Result = 1;
    } else {
        Result = 0;
    }

    if KeyValueInformation != (&mut Buffer as *mut u8 as *mut KEY_VALUE_FULL_INFORMATION) {
        ExFreePool(KeyValueInformation as *mut core::ffi::c_void);
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
//     FatInit.c
//
// Abstract:
//
//     This module implements the DRIVER_INITIALIZATION routine for Fat
//
//
// --*/
//
// #include "fatprocs.h"
//
// DRIVER_INITIALIZE DriverEntry;
//
// NTSTATUS
// NTAPI
// DriverEntry(
//      PDRIVER_OBJECT DriverObject,
//      PUNICODE_STRING RegistryPath
//     );
//
// _Function_class_(DRIVER_UNLOAD)
// VOID
// NTAPI
// FatUnload(
//      _Unreferenced_parameter_ PDRIVER_OBJECT DriverObject
//     );
//
// NTSTATUS
// FatGetCompatibilityModeValue(
//     IN PUNICODE_STRING ValueName,
//     IN OUT PULONG Value
//     );
//
// BOOLEAN
// FatIsFujitsuFMR (
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(INIT, DriverEntry)
// #pragma alloc_text(INIT, FatGetCompatibilityModeValue)
// #pragma alloc_text(INIT, FatIsFujitsuFMR)
// //#pragma alloc_text(PAGE, FatUnload)
// #endif
//
// #define COMPATIBILITY_MODE_KEY_NAME L"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\FileSystem"
// #define COMPATIBILITY_MODE_VALUE_NAME L"Win31FileSystem"
// #define CODE_PAGE_INVARIANCE_VALUE_NAME L"FatDisableCodePageInvariance"
//
//
// #define KEY_WORK_AREA ((sizeof(KEY_VALUE_FULL_INFORMATION) + \
//                         sizeof(ULONG)) + 64)
//
// #define REGISTRY_HARDWARE_DESCRIPTION_W \
//         L"\\Registry\\Machine\\Hardware\\DESCRIPTION\\System"
//
// #define REGISTRY_MACHINE_IDENTIFIER_W   L"Identifier"
//
// #define FUJITSU_FMR_NAME_W  L"FUJITSU FMR-"
//
//
//
// NTSTATUS
// NTAPI
// DriverEntry(
//      PDRIVER_OBJECT DriverObject,
//      PUNICODE_STRING RegistryPath
//     )
//
// /*++
//
// Routine Description:
//
//     This is the initialization routine for the Fat file system
//     device driver.  This routine creates the device object for the FileSystem
//     device and performs all other driver initialization.
//
// Arguments:
//
//     DriverObject - Pointer to driver object created by the system.
//
// Return Value:
//
//     NTSTATUS - The function value is the final status from the initialization
//         operation.
//
// --*/
//
// {
//     USHORT MaxDepth;
//     NTSTATUS Status;
//     UNICODE_STRING UnicodeString;
//     FS_FILTER_CALLBACKS FilterCallbacks;
//     UNICODE_STRING ValueName;
//     ULONG Value;
//
//     UNREFERENCED_PARAMETER( RegistryPath );
//
//     //
//     // Create the device object for disks.  To avoid problems with filters who
//     // know this name, we must keep it.
//     //
//
//     RtlInitUnicodeString( &UnicodeString, L"\\Fat" );
//     Status = IoCreateDevice( DriverObject,
//                              0,
//                              &UnicodeString,
//                              FILE_DEVICE_DISK_FILE_SYSTEM,
//                              0,
//                              FALSE,
//                              &FatDiskFileSystemDeviceObject );
//
//     if (!NT_SUCCESS( Status )) {
//         return Status;
//     }
//
//     //
//     // Create the device object for "cdroms".
//     //
//
//     RtlInitUnicodeString( &UnicodeString, L"\\FatCdrom" );
//     Status = IoCreateDevice( DriverObject,
//                              0,
//                              &UnicodeString,
//                              FILE_DEVICE_CD_ROM_FILE_SYSTEM,
//                              0,
//                              FALSE,
//                              &FatCdromFileSystemDeviceObject );
//
//     if (!NT_SUCCESS( Status )) {
//         IoDeleteDevice( FatDiskFileSystemDeviceObject);
//         return Status;
//     }
//
// #ifdef _MSC_VER
// #pragma prefast( push )
// #pragma prefast( disable:28155, "these are all correct" )
// #pragma prefast( disable:28169, "these are all correct" )
// #pragma prefast( disable:28175, "this is a filesystem, touching FastIoDispatch is allowed" )
// #endif
//
//     DriverObject->DriverUnload = FatUnload;
//
//     //
//     //  Note that because of the way data caching is done, we set neither
//     //  the Direct I/O or Buffered I/O bit in DeviceObject->Flags.  If
//     //  data is not in the cache, or the request is not buffered, we may,
//     //  set up for Direct I/O by hand.
//     //
//
//     //
//     // Initialize the driver object with this driver's entry points.
//     //
//
//     DriverObject->MajorFunction[IRP_MJ_CREATE]                   = (PDRIVER_DISPATCH)FatFsdCreate;
//     DriverObject->MajorFunction[IRP_MJ_CLOSE]                    = (PDRIVER_DISPATCH)FatFsdClose;
//     DriverObject->MajorFunction[IRP_MJ_READ]                     = (PDRIVER_DISPATCH)FatFsdRead;
//     DriverObject->MajorFunction[IRP_MJ_WRITE]                    = (PDRIVER_DISPATCH)FatFsdWrite;
//     DriverObject->MajorFunction[IRP_MJ_QUERY_INFORMATION]        = (PDRIVER_DISPATCH)FatFsdQueryInformation;
//     DriverObject->MajorFunction[IRP_MJ_SET_INFORMATION]          = (PDRIVER_DISPATCH)FatFsdSetInformation;
//     DriverObject->MajorFunction[IRP_MJ_QUERY_EA]                 = (PDRIVER_DISPATCH)FatFsdQueryEa;
//     DriverObject->MajorFunction[IRP_MJ_SET_EA]                   = (PDRIVER_DISPATCH)FatFsdSetEa;
//     DriverObject->MajorFunction[IRP_MJ_FLUSH_BUFFERS]            = (PDRIVER_DISPATCH)FatFsdFlushBuffers;
//     DriverObject->MajorFunction[IRP_MJ_QUERY_VOLUME_INFORMATION] = (PDRIVER_DISPATCH)FatFsdQueryVolumeInformation;
//     DriverObject->MajorFunction[IRP_MJ_SET_VOLUME_INFORMATION]   = (PDRIVER_DISPATCH)FatFsdSetVolumeInformation;
//     DriverObject->MajorFunction[IRP_MJ_CLEANUP]                  = (PDRIVER_DISPATCH)FatFsdCleanup;
//     DriverObject->MajorFunction[IRP_MJ_DIRECTORY_CONTROL]        = (PDRIVER_DISPATCH)FatFsdDirectoryControl;
//     DriverObject->MajorFunction[IRP_MJ_FILE_SYSTEM_CONTROL]      = (PDRIVER_DISPATCH)FatFsdFileSystemControl;
//     DriverObject->MajorFunction[IRP_MJ_LOCK_CONTROL]             = (PDRIVER_DISPATCH)FatFsdLockControl;
//     DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL]           = (PDRIVER_DISPATCH)FatFsdDeviceControl;
//     DriverObject->MajorFunction[IRP_MJ_SHUTDOWN]                 = (PDRIVER_DISPATCH)FatFsdShutdown;
//     DriverObject->MajorFunction[IRP_MJ_PNP]                      = (PDRIVER_DISPATCH)FatFsdPnp;
//
//     DriverObject->FastIoDispatch = &FatFastIoDispatch;
//
//     RtlZeroMemory(&FatFastIoDispatch, sizeof(FatFastIoDispatch));
//
//     FatFastIoDispatch.SizeOfFastIoDispatch =    sizeof(FAST_IO_DISPATCH);
//     FatFastIoDispatch.FastIoCheckIfPossible =   FatFastIoCheckIfPossible;  //  CheckForFastIo
//     FatFastIoDispatch.FastIoRead =              FsRtlCopyRead;             //  Read
//     FatFastIoDispatch.FastIoWrite =             FsRtlCopyWrite;            //  Write
//     FatFastIoDispatch.FastIoQueryBasicInfo =    FatFastQueryBasicInfo;     //  QueryBasicInfo
//     FatFastIoDispatch.FastIoQueryStandardInfo = FatFastQueryStdInfo;       //  QueryStandardInfo
//     FatFastIoDispatch.FastIoLock =              FatFastLock;               //  Lock
//     FatFastIoDispatch.FastIoUnlockSingle =      FatFastUnlockSingle;       //  UnlockSingle
//     FatFastIoDispatch.FastIoUnlockAll =         FatFastUnlockAll;          //  UnlockAll
//     FatFastIoDispatch.FastIoUnlockAllByKey =    FatFastUnlockAllByKey;     //  UnlockAllByKey
//     FatFastIoDispatch.FastIoQueryNetworkOpenInfo = FatFastQueryNetworkOpenInfo;
//     FatFastIoDispatch.AcquireForCcFlush =       FatAcquireForCcFlush;
//     FatFastIoDispatch.ReleaseForCcFlush =       FatReleaseForCcFlush;
//     FatFastIoDispatch.MdlRead =                 FsRtlMdlReadDev;
//     FatFastIoDispatch.MdlReadComplete =         FsRtlMdlReadCompleteDev;
//     FatFastIoDispatch.PrepareMdlWrite =         FsRtlPrepareMdlWriteDev;
//     FatFastIoDispatch.MdlWriteComplete =        FsRtlMdlWriteCompleteDev;
//
// #ifdef _MSC_VER
// #pragma prefast( pop )
// #endif
//
//     //
//     //  Initialize the filter callbacks we use.
//     //
//
//     RtlZeroMemory( &FilterCallbacks,
//                    sizeof(FS_FILTER_CALLBACKS) );
//
//     FilterCallbacks.SizeOfFsFilterCallbacks = sizeof(FS_FILTER_CALLBACKS);
//     FilterCallbacks.PreAcquireForSectionSynchronization = FatFilterCallbackAcquireForCreateSection;
//
//     Status = FsRtlRegisterFileSystemFilterCallbacks( DriverObject,
//                                                      &FilterCallbacks );
//
//     if (!NT_SUCCESS( Status )) {
//
//         IoDeleteDevice( FatDiskFileSystemDeviceObject );
//         IoDeleteDevice( FatCdromFileSystemDeviceObject );
//         return Status;
//     }
//
//     //
//     //  Initialize the global data structures
//     //
//
//     //
//     //  The FatData record
//     //
//
//     RtlZeroMemory( &FatData, sizeof(FAT_DATA));
//
//     FatData.NodeTypeCode = FAT_NTC_DATA_HEADER;
//     FatData.NodeByteSize = sizeof(FAT_DATA);
//
//     InitializeListHead(&FatData.VcbQueue);
//
//     FatData.DriverObject = DriverObject;
//     FatData.DiskFileSystemDeviceObject = FatDiskFileSystemDeviceObject;
//     FatData.CdromFileSystemDeviceObject = FatCdromFileSystemDeviceObject;
//
//     //
//     //  This list head keeps track of closes yet to be done.
//     //
//
//     InitializeListHead( &FatData.AsyncCloseList );
//     InitializeListHead( &FatData.DelayedCloseList );
//
//     FatData.FatCloseItem = IoAllocateWorkItem( FatDiskFileSystemDeviceObject);
//
//     if (FatData.FatCloseItem == NULL) {
//         IoDeleteDevice (FatDiskFileSystemDeviceObject);
//         IoDeleteDevice (FatCdromFileSystemDeviceObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     //  Allocate the zero page
//     //
//
//     FatData.ZeroPage = ExAllocatePoolWithTag( NonPagedPoolNx, PAGE_SIZE, 'ZtaF' );
//     if (FatData.ZeroPage == NULL) {
//         IoDeleteDevice (FatDiskFileSystemDeviceObject);
//         IoDeleteDevice (FatCdromFileSystemDeviceObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     RtlZeroMemory( FatData.ZeroPage, PAGE_SIZE );
//
//
//     //
//     //  Now initialize our general purpose spinlock (gag) and figure out how
//     //  deep and wide we want our delayed lists (along with fooling ourselves
//     //  about the lookaside depths).
//     //
//
//     KeInitializeSpinLock( &FatData.GeneralSpinLock );
//
//     switch ( MmQuerySystemSize() ) {
//
//     case MmSmallSystem:
//
//         MaxDepth = 4;
//         FatMaxDelayedCloseCount = FAT_MAX_DELAYED_CLOSES;
//         break;
//
//     case MmMediumSystem:
//
//         MaxDepth = 8;
//         FatMaxDelayedCloseCount = 4 * FAT_MAX_DELAYED_CLOSES;
//         break;
//
//     case MmLargeSystem:
//     default:
//
//         MaxDepth = 16;
//         FatMaxDelayedCloseCount = 16 * FAT_MAX_DELAYED_CLOSES;
//         break;
//     }
//
//
//     //
//     //  Initialize the cache manager callback routines
//     //
//
//     FatData.CacheManagerCallbacks.AcquireForLazyWrite  = &FatAcquireFcbForLazyWrite;
//     FatData.CacheManagerCallbacks.ReleaseFromLazyWrite = &FatReleaseFcbFromLazyWrite;
//     FatData.CacheManagerCallbacks.AcquireForReadAhead  = &FatAcquireFcbForReadAhead;
//     FatData.CacheManagerCallbacks.ReleaseFromReadAhead = &FatReleaseFcbFromReadAhead;
//
//     FatData.CacheManagerNoOpCallbacks.AcquireForLazyWrite  = &FatNoOpAcquire;
//     FatData.CacheManagerNoOpCallbacks.ReleaseFromLazyWrite = &FatNoOpRelease;
//     FatData.CacheManagerNoOpCallbacks.AcquireForReadAhead  = &FatNoOpAcquire;
//     FatData.CacheManagerNoOpCallbacks.ReleaseFromReadAhead = &FatNoOpRelease;
//
//     //
//     //  Set up global pointer to our process.
//     //
//
//     FatData.OurProcess = PsGetCurrentProcess();
//
//     //
//     //  Setup the number of processors we support for statistics as the current number
//     //  running.
//     //
//
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//     FatData.NumberProcessors = KeQueryActiveProcessorCount( NULL );
// #else
//     FatData.NumberProcessors = KeNumberProcessors;
// #endif
//
//
//     //
//     //  Read the registry to determine if we are in ChicagoMode.
//     //
//
//     ValueName.Buffer = COMPATIBILITY_MODE_VALUE_NAME;
//     ValueName.Length = sizeof(COMPATIBILITY_MODE_VALUE_NAME) - sizeof(WCHAR);
//     ValueName.MaximumLength = sizeof(COMPATIBILITY_MODE_VALUE_NAME);
//
//     Status = FatGetCompatibilityModeValue( &ValueName, &Value );
//
//     if (NT_SUCCESS(Status) && FlagOn(Value, 1)) {
//
//         FatData.ChicagoMode = FALSE;
//
//     } else {
//
//         FatData.ChicagoMode = TRUE;
//     }
//
//     //
//     //  Read the registry to determine if we are going to generate LFNs
//     //  for valid 8.3 names with extended characters.
//     //
//
//     ValueName.Buffer = CODE_PAGE_INVARIANCE_VALUE_NAME;
//     ValueName.Length = sizeof(CODE_PAGE_INVARIANCE_VALUE_NAME) - sizeof(WCHAR);
//     ValueName.MaximumLength = sizeof(CODE_PAGE_INVARIANCE_VALUE_NAME);
//
//     Status = FatGetCompatibilityModeValue( &ValueName, &Value );
//
//     if (NT_SUCCESS(Status) && FlagOn(Value, 1)) {
//
//         FatData.CodePageInvariant = FALSE;
//
//     } else {
//
//         FatData.CodePageInvariant = TRUE;
//     }
//
//     //
//     //  Initialize our global resource and fire up the lookaside lists.
//     //
//
//     ExInitializeResourceLite( &FatData.Resource );
//
//     ExInitializeNPagedLookasideList( &FatIrpContextLookasideList,
//                                      NULL,
//                                      NULL,
//                                      POOL_NX_ALLOCATION | POOL_RAISE_IF_ALLOCATION_FAILURE,
//                                      sizeof(IRP_CONTEXT),
//                                      TAG_IRP_CONTEXT,
//                                      MaxDepth );
//
//     ExInitializeNPagedLookasideList( &FatNonPagedFcbLookasideList,
//                                      NULL,
//                                      NULL,
//                                      POOL_NX_ALLOCATION | POOL_RAISE_IF_ALLOCATION_FAILURE,
//                                      sizeof(NON_PAGED_FCB),
//                                      TAG_FCB_NONPAGED,
//                                      MaxDepth );
//
//     ExInitializeNPagedLookasideList( &FatEResourceLookasideList,
//                                      NULL,
//                                      NULL,
//                                      POOL_NX_ALLOCATION | POOL_RAISE_IF_ALLOCATION_FAILURE,
//                                      sizeof(ERESOURCE),
//                                      TAG_ERESOURCE,
//                                      MaxDepth );
//
//     ExInitializeSListHead( &FatCloseContextSList );
//     ExInitializeFastMutex( &FatCloseQueueMutex );
//     KeInitializeEvent( &FatReserveEvent, SynchronizationEvent, TRUE );
//
//     //
//     //  Register the file system with the I/O system
//     //
//
//     IoRegisterFileSystem(FatDiskFileSystemDeviceObject);
//     ObReferenceObject (FatDiskFileSystemDeviceObject);
//     IoRegisterFileSystem(FatCdromFileSystemDeviceObject);
//     ObReferenceObject (FatCdromFileSystemDeviceObject);
//
//     //
//     //  Find out if we are running an a FujitsuFMR machine.
//     //
//
//     FatData.FujitsuFMR = FatIsFujitsuFMR();
//
// #if (NTDDI_VERSION >= NTDDI_WIN8)
//
//     //
//     //  Find out global disk accounting state, cache the result
//     //
//
//     FatDiskAccountingEnabled = PsIsDiskCountersEnabled();
//
// #endif
//
//     //
//     //  And return to our caller
//     //
//
//     return( STATUS_SUCCESS );
// }
//
//
// _Function_class_(DRIVER_UNLOAD)
// VOID
// NTAPI
// FatUnload(
//      _Unreferenced_parameter_ PDRIVER_OBJECT DriverObject
//     )
//
// /*++
//
// Routine Description:
//
//     This is the unload routine for the filesystem
//
// Arguments:
//
//     DriverObject - Pointer to driver object created by the system.
//
// Return Value:
//
//     None
//
// --*/
//
// {
//     UNREFERENCED_PARAMETER( DriverObject );
//
//
//     ExDeleteNPagedLookasideList (&FatEResourceLookasideList);
//     ExDeleteNPagedLookasideList (&FatNonPagedFcbLookasideList);
//     ExDeleteNPagedLookasideList (&FatIrpContextLookasideList);
//     ExDeleteResourceLite( &FatData.Resource );
//     IoFreeWorkItem (FatData.FatCloseItem);
//     ObDereferenceObject( FatDiskFileSystemDeviceObject);
//     ObDereferenceObject( FatCdromFileSystemDeviceObject);
// }
//
//
// //
// //  Local Support routine
// //
//
// NTSTATUS
// FatGetCompatibilityModeValue (
//     IN PUNICODE_STRING ValueName,
//     IN OUT PULONG Value
//     )
//
// /*++
//
// Routine Description:
//
//     Given a unicode value name this routine will go into the registry
//     location for the Chicago compatibilitymode information and get the
//     value.
//
// Arguments:
//
//     ValueName - the unicode name for the registry value located in the registry.
//     Value   - a pointer to the ULONG for the result.
//
// Return Value:
//
//     NTSTATUS
//
//     If STATUS_SUCCESSFUL is returned, the location *Value will be
//     updated with the DWORD value from the registry.  If any failing
//     status is returned, this value is untouched.
//
// --*/
//
// {
//     HANDLE Handle;
//     NTSTATUS Status;
//     ULONG RequestLength;
//     ULONG ResultLength;
//     UCHAR Buffer[KEY_WORK_AREA];
//     UNICODE_STRING KeyName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PKEY_VALUE_FULL_INFORMATION KeyValueInformation;
//
//     KeyName.Buffer = COMPATIBILITY_MODE_KEY_NAME;
//     KeyName.Length = sizeof(COMPATIBILITY_MODE_KEY_NAME) - sizeof(WCHAR);
//     KeyName.MaximumLength = sizeof(COMPATIBILITY_MODE_KEY_NAME);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = ZwOpenKey(&Handle,
//                        KEY_READ,
//                        &ObjectAttributes);
//
//     if (!NT_SUCCESS(Status)) {
//
//         return Status;
//     }
//
//     RequestLength = KEY_WORK_AREA;
//
//     KeyValueInformation = (PKEY_VALUE_FULL_INFORMATION)Buffer;
//
//     while (1) {
//
//         Status = ZwQueryValueKey(Handle,
//                                  ValueName,
//                                  KeyValueFullInformation,
//                                  KeyValueInformation,
//                                  RequestLength,
//                                  &ResultLength);
//
//         NT_ASSERT( Status != STATUS_BUFFER_OVERFLOW );
//
//         if (Status == STATUS_BUFFER_OVERFLOW) {
//
//             //
//             // Try to get a buffer big enough.
//             //
//
//             if (KeyValueInformation != (PKEY_VALUE_FULL_INFORMATION)Buffer) {
//
//                 ExFreePool(KeyValueInformation);
//             }
//
//             RequestLength += 256;
//
//             KeyValueInformation = (PKEY_VALUE_FULL_INFORMATION)
//                                   ExAllocatePoolWithTag(PagedPool,
//                                                         RequestLength,
//                                                         ' taF');
//
//             if (!KeyValueInformation) {
//
//                 ZwClose(Handle);
//                 return STATUS_NO_MEMORY;
//             }
//
//         } else {
//
//             break;
//         }
//     }
//
//     ZwClose(Handle);
//
//     if (NT_SUCCESS(Status)) {
//
//         if (KeyValueInformation->DataLength != 0) {
//
//             PULONG DataPtr;
//
//             //
//             // Return contents to the caller.
//             //
//
//             DataPtr = (PULONG)
//               ((PUCHAR)KeyValueInformation + KeyValueInformation->DataOffset);
//             *Value = *DataPtr;
//
//         } else {
//
//             //
//             // Treat as if no value was found
//             //
//
//             Status = STATUS_OBJECT_NAME_NOT_FOUND;
//         }
//     }
//
//     if (KeyValueInformation != (PKEY_VALUE_FULL_INFORMATION)Buffer) {
//
//         ExFreePool(KeyValueInformation);
//     }
//
//     return Status;
// }
//
// //
// //  Local Support routine
// //
//
// BOOLEAN
// FatIsFujitsuFMR (
//     )
//
// /*++
//
// Routine Description:
//
//     This routine tells us if we are running on a FujitsuFMR machine.
//
// Arguments:
//
//
// Return Value:
//
//     BOOLEAN - TRUE if we are and FALSE otherwise
//
// --*/
//
// {
//     BOOLEAN Result;
//     HANDLE Handle;
//     NTSTATUS Status;
//     ULONG RequestLength;
//     ULONG ResultLength;
//     UCHAR Buffer[KEY_WORK_AREA];
//     UNICODE_STRING KeyName;
//     UNICODE_STRING ValueName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PKEY_VALUE_FULL_INFORMATION KeyValueInformation;
//
//     //
//     // Set default as PC/AT
//     //
//
//     KeyName.Buffer = REGISTRY_HARDWARE_DESCRIPTION_W;
//     KeyName.Length = sizeof(REGISTRY_HARDWARE_DESCRIPTION_W) - sizeof(WCHAR);
//     KeyName.MaximumLength = sizeof(REGISTRY_HARDWARE_DESCRIPTION_W);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = ZwOpenKey(&Handle,
//                        KEY_READ,
//                        &ObjectAttributes);
//
//     if (!NT_SUCCESS(Status)) {
//
//         return FALSE;
//     }
//
//     ValueName.Buffer = REGISTRY_MACHINE_IDENTIFIER_W;
//     ValueName.Length = sizeof(REGISTRY_MACHINE_IDENTIFIER_W) - sizeof(WCHAR);
//     ValueName.MaximumLength = sizeof(REGISTRY_MACHINE_IDENTIFIER_W);
//
//     RequestLength = KEY_WORK_AREA;
//
//     KeyValueInformation = (PKEY_VALUE_FULL_INFORMATION)Buffer;
//
//     while (1) {
//
//         Status = ZwQueryValueKey(Handle,
//                                  &ValueName,
//                                  KeyValueFullInformation,
//                                  KeyValueInformation,
//                                  RequestLength,
//                                  &ResultLength);
//
//         // NT_ASSERT( Status != STATUS_BUFFER_OVERFLOW );
//
//         if (Status == STATUS_BUFFER_OVERFLOW) {
//
//             //
//             // Try to get a buffer big enough.
//             //
//
//             if (KeyValueInformation != (PKEY_VALUE_FULL_INFORMATION)Buffer) {
//
//                 ExFreePool(KeyValueInformation);
//             }
//
//             RequestLength += 256;
//
//             KeyValueInformation = (PKEY_VALUE_FULL_INFORMATION)
//                                   ExAllocatePoolWithTag(PagedPool, RequestLength, ' taF');
//
//             if (!KeyValueInformation) {
//
//                 ZwClose(Handle);
//                 return FALSE;
//             }
//
//         } else {
//
//             break;
//         }
//     }
//
//     ZwClose(Handle);
//
//     if (NT_SUCCESS(Status) &&
//         (KeyValueInformation->DataLength >= sizeof(FUJITSU_FMR_NAME_W)) &&
//         (RtlCompareMemory((PUCHAR)KeyValueInformation + KeyValueInformation->DataOffset,
//                           FUJITSU_FMR_NAME_W,
//                           sizeof(FUJITSU_FMR_NAME_W) - sizeof(WCHAR)) ==
//          sizeof(FUJITSU_FMR_NAME_W) - sizeof(WCHAR))) {
//
//         Result = TRUE;
//
//     } else {
//
//         Result = FALSE;
//     }
//
//     if (KeyValueInformation != (PKEY_VALUE_FULL_INFORMATION)Buffer) {
//
//         ExFreePool(KeyValueInformation);
//     }
//
//     return Result;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
