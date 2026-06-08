use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DRIVER_INITIALIZE, DRIVER_UNLOAD, ExDeleteResourceLite, ExInitializeFastMutex, ExInitializeResourceLite, FAST_IO_DISPATCH, FS_FILTER_CALLBACKS, IoAllocateWorkItem, IoCreateDevice, IoDeleteDevice, IoFreeWorkItem, IoRegisterFileSystem, MmQuerySystemSize, PDEVICE_OBJECT, PDRIVER_DISPATCH, PDRIVER_OBJECT, PUNICODE_STRING, PopEntryList, RtlInitUnicodeString};
use core::ffi::c_void;

extern "system" {
    fn CdFreePool(IrpContext: *mut *mut IRP_CONTEXT);
    fn CdFastIoCheckIfPossible();
    fn FsRtlCopyRead();
    fn CdFastQueryBasicInfo();
    fn CdFastQueryStdInfo();
    fn CdFastLock();
    fn CdFastUnlockSingle();
    fn CdFastUnlockAll();
    fn CdFastUnlockAllByKey();
    fn CdReleaseForCreateSection();
    fn CdFastQueryNetworkInfo();
    fn FsRtlMdlReadDev();
    fn FsRtlMdlReadCompleteDev();
    fn FsRtlPrepareMdlWriteDev();
    fn FsRtlMdlWriteCompleteDev();
    fn CdAcquireForCache();
    fn CdReleaseFromCache();
    fn CdNoopAcquire();
    fn CdNoopRelease();
    fn CdInitializeTelemetry();
}

// ---- file-scope preamble (translated) ----
pub const BugCheckFileId: u32 = CDFS_BUG_CHECK_CDINIT;

// Manually define missing constants
const FILE_DEVICE_CD_ROM_FILE_SYSTEM: u32 = 0x00000009;
const FILE_DEVICE_DISK_FILE_SYSTEM: u32 = 0x00000008;
const DO_LOW_PRIORITY_FILESYSTEM: u32 = 0x00020000;
const IRP_MJ_QUERY_VOLUME_INFORMATION: i32 = 0x0a;
const IRP_MJ_DIRECTORY_CONTROL: i32 = 0x0c;

// Declare external items not available in the given imports
#[no_mangle]
pub unsafe extern "C" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    // UNREFERENCED_PARAMETER
    let _ = RegistryPath;

    let mut Status: NTSTATUS;
    let mut UnicodeString: _UNICODE_STRING;
    let mut CdfsFileSystemDeviceObject: PDEVICE_OBJECT;
    #[cfg(feature = "__REACTOS__")]
    let mut HddFileSystemDeviceObject: PDEVICE_OBJECT;
    let mut FilterCallbacks: FS_FILTER_CALLBACKS;

    // Create the device object.
    RtlInitUnicodeString(&mut UnicodeString, b"\\Cdfs\0".as_ptr() as *const u16);

    Status = IoCreateDevice(
        DriverObject,
        0,
        &UnicodeString,
        FILE_DEVICE_CD_ROM_FILE_SYSTEM,
        0,
        0, // FALSE
        &mut CdfsFileSystemDeviceObject,
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    #[cfg(feature = "__REACTOS__")]
    {
        // Create the HDD device object.
        RtlInitUnicodeString(&mut UnicodeString, b"\\CdfsHdd\0".as_ptr() as *const u16);

        Status = IoCreateDevice(
            DriverObject,
            0,
            &UnicodeString,
            FILE_DEVICE_DISK_FILE_SYSTEM,
            0,
            0, // FALSE
            &mut HddFileSystemDeviceObject,
        );

        if !NT_SUCCESS(Status) {
            IoDeleteDevice(CdfsFileSystemDeviceObject);
            return Status;
        }
    }

    (*DriverObject).DriverUnload = Some(CdUnload);

    // Initialize dispatch table.
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );
    (*DriverObject).MajorFunction[IRP_MJ_WRITE as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_INFORMATION as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );
    (*DriverObject).MajorFunction[IRP_MJ_SET_INFORMATION as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_VOLUME_INFORMATION as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );
    (*DriverObject).MajorFunction[IRP_MJ_DIRECTORY_CONTROL as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );
    (*DriverObject).MajorFunction[IRP_MJ_FILE_SYSTEM_CONTROL as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );
    (*DriverObject).MajorFunction[IRP_MJ_LOCK_CONTROL as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );
    (*DriverObject).MajorFunction[IRP_MJ_CLEANUP as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );
    (*DriverObject).MajorFunction[IRP_MJ_SHUTDOWN as usize] = Some(
        core::mem::transmute(CdFsdDispatch as *const ())
    );

    (*DriverObject).FastIoDispatch = core::ptr::addr_of_mut!(CdFastIoDispatch) as *mut FAST_IO_DISPATCH;

    // Initialize the filter callbacks we use.
    RtlZeroMemory(
        &mut FilterCallbacks as *mut FS_FILTER_CALLBACKS as *mut u8,
        core::mem::size_of::<FS_FILTER_CALLBACKS>(),
    );

    FilterCallbacks.SizeOfFsFilterCallbacks = core::mem::size_of::<FS_FILTER_CALLBACKS>() as u32;
    FilterCallbacks.PreAcquireForSectionSynchronization = Some(CdFilterCallbackAcquireForCreateSection);

    Status = FsRtlRegisterFileSystemFilterCallbacks(
        DriverObject,
        &mut FilterCallbacks,
    );

    if !NT_SUCCESS(Status) {
        IoDeleteDevice(CdfsFileSystemDeviceObject);
        #[cfg(feature = "__REACTOS__")]
        IoDeleteDevice(HddFileSystemDeviceObject);
        return Status;
    }

    // Initialize the global data structures
    #[cfg(not(feature = "__REACTOS__"))]
    {
        Status = CdInitializeGlobalData(DriverObject, CdfsFileSystemDeviceObject);
    }
    #[cfg(feature = "__REACTOS__")]
    {
        Status = CdInitializeGlobalData(DriverObject, CdfsFileSystemDeviceObject, HddFileSystemDeviceObject);
    }
    if !NT_SUCCESS(Status) {
        IoDeleteDevice(CdfsFileSystemDeviceObject);
        #[cfg(feature = "__REACTOS__")]
        IoDeleteDevice(HddFileSystemDeviceObject);
        return Status;
    }

    // Register the file system as low priority.
    (*CdfsFileSystemDeviceObject).Flags |= DO_LOW_PRIORITY_FILESYSTEM;
    #[cfg(feature = "__REACTOS__")]
    {
        (*HddFileSystemDeviceObject).Flags |= DO_LOW_PRIORITY_FILESYSTEM;
    }

    IoRegisterFileSystem(CdfsFileSystemDeviceObject);
    ObReferenceObject(CdfsFileSystemDeviceObject);

    #[cfg(feature = "__REACTOS__")]
    {
        IoRegisterFileSystem(HddFileSystemDeviceObject);
        ObReferenceObject(HddFileSystemDeviceObject);
    }

    #[cfg(feature = "CDFS_TELEMETRY_DATA")]
    CdInitializeTelemetry();

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn CdUnload(DriverObject: *mut DRIVER_OBJECT) {
    // PAGED_CODE();
    // UNREFERENCED_PARAMETER(DriverObject)
    let _ = DriverObject;

    let cd_data = &mut *core::ptr::addr_of_mut!(CdData);
    let mut IrpContext: *mut IRP_CONTEXT;

    // Free any IRP contexts
    loop {
        IrpContext = PopEntryList(core::ptr::addr_of_mut!(cd_data.IrpContextList)) as *mut IRP_CONTEXT;
        if IrpContext.is_null() {
            break;
        }
        CdFreePool(core::ptr::addr_of_mut!(IrpContext));
    }

    IoFreeWorkItem(cd_data.CloseItem);
    ExDeleteResourceLite(core::ptr::addr_of_mut!(cd_data.DataResource));
    ObDereferenceObject(cd_data.FileSystemDeviceObject as *mut c_void);

    #[cfg(reactos)]
    ObDereferenceObject(cd_data.HddFileSystemDeviceObject as *mut c_void);
}
#[no_mangle]
pub unsafe extern "C" fn CdInitializeGlobalData(
    DriverObject: *mut DRIVER_OBJECT,
    FileSystemDeviceObject: *mut DEVICE_OBJECT,
    #[cfg(feature = "reactos")] HddFileSystemDeviceObject: *mut DEVICE_OBJECT,
) -> NTSTATUS {
    RtlZeroMemory(
        core::ptr::addr_of_mut!(CdFastIoDispatch).cast(),
        core::mem::size_of::<FAST_IO_DISPATCH>(),
    );
    CdFastIoDispatch.SizeOfFastIoDispatch = core::mem::size_of::<FAST_IO_DISPATCH>() as u32;

    CdFastIoDispatch.FastIoCheckIfPossible = CdFastIoCheckIfPossible as _;
    CdFastIoDispatch.FastIoRead = FsRtlCopyRead as _;
    CdFastIoDispatch.FastIoQueryBasicInfo = CdFastQueryBasicInfo as _;
    CdFastIoDispatch.FastIoQueryStandardInfo = CdFastQueryStdInfo as _;
    CdFastIoDispatch.FastIoLock = CdFastLock as _;
    CdFastIoDispatch.FastIoUnlockSingle = CdFastUnlockSingle as _;
    CdFastIoDispatch.FastIoUnlockAll = CdFastUnlockAll as _;
    CdFastIoDispatch.FastIoUnlockAllByKey = CdFastUnlockAllByKey as _;
    CdFastIoDispatch.AcquireFileForNtCreateSection = core::ptr::null_mut() as _;
    CdFastIoDispatch.ReleaseFileForNtCreateSection = CdReleaseForCreateSection as _;
    CdFastIoDispatch.FastIoQueryNetworkOpenInfo = CdFastQueryNetworkInfo as _;
    CdFastIoDispatch.MdlRead = FsRtlMdlReadDev as _;
    CdFastIoDispatch.MdlReadComplete = FsRtlMdlReadCompleteDev as _;
    CdFastIoDispatch.PrepareMdlWrite = FsRtlPrepareMdlWriteDev as _;
    CdFastIoDispatch.MdlWriteComplete = FsRtlMdlWriteCompleteDev as _;

    RtlZeroMemory(
        core::ptr::addr_of_mut!(CdData).cast(),
        core::mem::size_of::<CD_DATA>(),
    );
    CdData.NodeTypeCode = 0x2603i16; // CDFS_NTC_DATA_HEADER
    CdData.NodeByteSize = core::mem::size_of::<CD_DATA>() as u32;
    CdData.DriverObject = DriverObject;
    CdData.FileSystemDeviceObject = FileSystemDeviceObject;
    #[cfg(feature = "reactos")]
    {
        CdData.HddFileSystemDeviceObject = HddFileSystemDeviceObject;
    }
    InitializeListHead(core::ptr::addr_of_mut!(CdData.VcbQueue));
    ExInitializeResourceLite(core::ptr::addr_of_mut!(CdData.DataResource));

    CdData.CacheManagerCallbacks.AcquireForLazyWrite =
        core::mem::transmute::<*mut core::ffi::c_void, _>(CdAcquireForCache as *mut core::ffi::c_void);
    CdData.CacheManagerCallbacks.ReleaseFromLazyWrite =
        core::mem::transmute::<*mut core::ffi::c_void, _>(CdReleaseFromCache as *mut core::ffi::c_void);
    CdData.CacheManagerCallbacks.AcquireForReadAhead =
        core::mem::transmute::<*mut core::ffi::c_void, _>(CdAcquireForCache as *mut core::ffi::c_void);
    CdData.CacheManagerCallbacks.ReleaseFromReadAhead =
        core::mem::transmute::<*mut core::ffi::c_void, _>(CdReleaseFromCache as *mut core::ffi::c_void);

    CdData.CacheManagerVolumeCallbacks.AcquireForLazyWrite = CdNoopAcquire;
    CdData.CacheManagerVolumeCallbacks.ReleaseFromLazyWrite = CdNoopRelease;
    CdData.CacheManagerVolumeCallbacks.AcquireForReadAhead = CdNoopAcquire;
    CdData.CacheManagerVolumeCallbacks.ReleaseFromReadAhead = CdNoopRelease;

    ExInitializeFastMutex(core::ptr::addr_of_mut!(CdData.CdDataMutex));
    InitializeListHead(core::ptr::addr_of_mut!(CdData.AsyncCloseQueue));
    InitializeListHead(core::ptr::addr_of_mut!(CdData.DelayedCloseQueue));

    CdData.CloseItem = IoAllocateWorkItem(FileSystemDeviceObject);
    if CdData.CloseItem.is_null() {
        ExDeleteResourceLite(core::ptr::addr_of_mut!(CdData.DataResource));
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    match MmQuerySystemSize() {
        MmSmallSystem => {
            CdData.IrpContextMaxDepth = 4;
            CdData.MaxDelayedCloseCount = 8;
            CdData.MinDelayedCloseCount = 2;
        }
        MmMediumSystem => {
            CdData.IrpContextMaxDepth = 8;
            CdData.MaxDelayedCloseCount = 24;
            CdData.MinDelayedCloseCount = 6;
        }
        MmLargeSystem => {
            CdData.IrpContextMaxDepth = 32;
            CdData.MaxDelayedCloseCount = 72;
            CdData.MinDelayedCloseCount = 18;
        }
        _ => {}
    }
    STATUS_SUCCESS
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
//     CdInit.c
//
// Abstract:
//
//     This module implements the DRIVER_INITIALIZATION routine for Cdfs
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
// #define BugCheckFileId                   (CDFS_BUG_CHECK_CDINIT)
//
// //  Tell prefast the function type.
// DRIVER_INITIALIZE DriverEntry;
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DriverEntry(
//      PDRIVER_OBJECT DriverObject,
//      PUNICODE_STRING RegistryPath
//     );
//
//
// // tell prefast this is a driver unload function
// DRIVER_UNLOAD CdUnload;
//
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdUnload(
//      PDRIVER_OBJECT DriverObject
//     );
//
// NTSTATUS
// CdInitializeGlobalData (
//      PDRIVER_OBJECT DriverObject,
//      PDEVICE_OBJECT FileSystemDeviceObject
// #ifdef __REACTOS__
//     ,
//     IN PDEVICE_OBJECT HddFileSystemDeviceObject
// #endif
//     );
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(INIT, DriverEntry)
// #pragma alloc_text(PAGE, CdUnload)
// #pragma alloc_text(INIT, CdInitializeGlobalData)
// #endif
//
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// DriverEntry(
//      PDRIVER_OBJECT DriverObject,
//      PUNICODE_STRING RegistryPath
//     )
//
// /*++
//
// Routine Description:
//
//     This is the initialization routine for the Cdrom file system
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
//     NTSTATUS Status;
//     UNICODE_STRING UnicodeString;
//     PDEVICE_OBJECT CdfsFileSystemDeviceObject;
//     FS_FILTER_CALLBACKS FilterCallbacks;
// #ifdef __REACTOS__
//     PDEVICE_OBJECT HddFileSystemDeviceObject;
// #endif
//
//     UNREFERENCED_PARAMETER( RegistryPath );
//
//     //
//     // Create the device object.
//     //
//
//     RtlInitUnicodeString( &UnicodeString, L"\\Cdfs" );
//
//     Status = IoCreateDevice( DriverObject,
//                              0,
//                              &UnicodeString,
//                              FILE_DEVICE_CD_ROM_FILE_SYSTEM,
//                              0,
//                              FALSE,
//                              &CdfsFileSystemDeviceObject );
//
//     if (!NT_SUCCESS( Status )) {
//         return Status;
//     }
//
// #ifdef __REACTOS__
//     //
//     // Create the HDD device object.
//     //
//
//     RtlInitUnicodeString( &UnicodeString, L"\\CdfsHdd" );
//
//     Status = IoCreateDevice( DriverObject,
//                              0,
//                              &UnicodeString,
//                              FILE_DEVICE_DISK_FILE_SYSTEM,
//                              0,
//                              FALSE,
//                              &HddFileSystemDeviceObject );
//
//     if (!NT_SUCCESS( Status )) {
//         IoDeleteDevice (CdfsFileSystemDeviceObject);
//         return Status;
//     }
// #endif
//
// #ifdef _MSC_VER
// #pragma prefast(push)
// #pragma prefast(disable: 28155, "the dispatch routine has the correct type, prefast is just being paranoid.")
// #pragma prefast(disable: 28168, "the dispatch routine has the correct type, prefast is just being paranoid.")
// #pragma prefast(disable: 28169, "the dispatch routine has the correct type, prefast is just being paranoid.")
// #pragma prefast(disable: 28175, "we're allowed to change these.")
// #endif
//
//     DriverObject->DriverUnload = CdUnload;
//
//     //
//     //  Note that because of the way data caching is done, we set neither
//     //  the Direct I/O or Buffered I/O bit in DeviceObject->Flags.  If
//     //  data is not in the cache, or the request is not buffered, we may,
//     //  set up for Direct I/O by hand.
//     //
//
//     //
//     //  Initialize the driver object with this driver's entry points.
//     //
//     //  NOTE - Each entry in the dispatch table must have an entry in
//     //  the Fsp/Fsd dispatch switch statements.
//     //
//
//     DriverObject->MajorFunction[IRP_MJ_CREATE]                  =
//     DriverObject->MajorFunction[IRP_MJ_CLOSE]                   =
//     DriverObject->MajorFunction[IRP_MJ_READ]                    =
//     DriverObject->MajorFunction[IRP_MJ_WRITE]                   =
//     DriverObject->MajorFunction[IRP_MJ_QUERY_INFORMATION]       =
//     DriverObject->MajorFunction[IRP_MJ_SET_INFORMATION]         =
//     DriverObject->MajorFunction[IRP_MJ_QUERY_VOLUME_INFORMATION]=
//     DriverObject->MajorFunction[IRP_MJ_DIRECTORY_CONTROL]       =
//     DriverObject->MajorFunction[IRP_MJ_FILE_SYSTEM_CONTROL]     =
//     DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL]          =
//     DriverObject->MajorFunction[IRP_MJ_LOCK_CONTROL]            =
//     DriverObject->MajorFunction[IRP_MJ_CLEANUP]                 =
//     DriverObject->MajorFunction[IRP_MJ_PNP]                     =
//     DriverObject->MajorFunction[IRP_MJ_SHUTDOWN]                = (PDRIVER_DISPATCH) CdFsdDispatch;
// #ifdef _MSC_VER
// #pragma prefast(pop)
//
// #pragma prefast(suppress: 28175, "this is a file system driver, we're allowed to touch FastIoDispatch.")
// #endif
//     DriverObject->FastIoDispatch = &CdFastIoDispatch;
//
//     //
//     //  Initialize the filter callbacks we use.
//     //
//
//     RtlZeroMemory( &FilterCallbacks,
//                    sizeof(FS_FILTER_CALLBACKS) );
//
//     FilterCallbacks.SizeOfFsFilterCallbacks = sizeof(FS_FILTER_CALLBACKS);
//     FilterCallbacks.PreAcquireForSectionSynchronization = CdFilterCallbackAcquireForCreateSection;
//
//     Status = FsRtlRegisterFileSystemFilterCallbacks( DriverObject,
//                                                      &FilterCallbacks );
//
//     if (!NT_SUCCESS( Status )) {
//
//         IoDeleteDevice( CdfsFileSystemDeviceObject );
// #ifdef __REACTOS__
//         IoDeleteDevice (HddFileSystemDeviceObject);
// #endif
//         return Status;
//     }
//
//     //
//     //  Initialize the global data structures
//     //
//
// #ifndef __REACTOS__
//     Status = CdInitializeGlobalData( DriverObject, CdfsFileSystemDeviceObject );
// #else
//     Status = CdInitializeGlobalData( DriverObject, CdfsFileSystemDeviceObject, HddFileSystemDeviceObject );
// #endif
//     if (!NT_SUCCESS (Status)) {
//         IoDeleteDevice (CdfsFileSystemDeviceObject);
// #ifdef __REACTOS__
//         IoDeleteDevice (HddFileSystemDeviceObject);
// #endif
//         return Status;
//     }
//
//     //
//     //  Register the file system as low priority with the I/O system.  This will cause
//     //  CDFS to receive mount requests after a) other filesystems currently registered
//     //  and b) other normal priority filesystems that may be registered later.
//     //
//
//     CdfsFileSystemDeviceObject->Flags |= DO_LOW_PRIORITY_FILESYSTEM;
// #ifdef __REACTOS__
//     HddFileSystemDeviceObject->Flags |= DO_LOW_PRIORITY_FILESYSTEM;
// #endif
//
//     IoRegisterFileSystem( CdfsFileSystemDeviceObject );
//     ObReferenceObject (CdfsFileSystemDeviceObject);
// #ifdef __REACTOS__
//     IoRegisterFileSystem( HddFileSystemDeviceObject );
//     ObReferenceObject (HddFileSystemDeviceObject);
// #endif
//
// #ifdef CDFS_TELEMETRY_DATA
//     //
//     //  Initialize Telemetry
//     //
//
//     CdInitializeTelemetry();
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
// VOID
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// CdUnload(
//      PDRIVER_OBJECT DriverObject
//     )
// /*++
//
// Routine Description:
//
//     This routine unload routine for CDFS.
//
// Arguments:
//
//     DriverObject - Supplies the driver object for CDFS.
//
// Return Value:
//
//     None.
//
// --*/
// {
//     PIRP_CONTEXT IrpContext;
//
//     PAGED_CODE();
//
//     UNREFERENCED_PARAMETER( DriverObject );
//
//     //
//     // Free any IRP contexts
//     //
//     while (1) {
//         IrpContext = (PIRP_CONTEXT) PopEntryList( &CdData.IrpContextList) ;
//         if (IrpContext == NULL) {
//             break;
//         }
//         CdFreePool(&IrpContext);
//     }
//
//     IoFreeWorkItem (CdData.CloseItem);
//     ExDeleteResourceLite( &CdData.DataResource );
//     ObDereferenceObject (CdData.FileSystemDeviceObject);
// #ifdef __REACTOS__
//     ObDereferenceObject (CdData.HddFileSystemDeviceObject);
// #endif
// }
//
// //
// //  Local support routine
// //
//
// NTSTATUS
// CdInitializeGlobalData (
//      PDRIVER_OBJECT DriverObject,
//      PDEVICE_OBJECT FileSystemDeviceObject
// #ifdef __REACTOS__
//     ,
//     IN PDEVICE_OBJECT HddFileSystemDeviceObject
// #endif
//     )
//
// /*++
//
// Routine Description:
//
//     This routine initializes the global cdfs data structures.
//
// Arguments:
//
//     DriverObject - Supplies the driver object for CDFS.
//
//     FileSystemDeviceObject - Supplies the device object for CDFS.
//
// Return Value:
//
//     None.
//
// --*/
//
// {
//     //
//     //  Start by initializing the FastIoDispatch Table.
//     //
//
//     RtlZeroMemory( &CdFastIoDispatch, sizeof( FAST_IO_DISPATCH ));
//
//     CdFastIoDispatch.SizeOfFastIoDispatch =    sizeof(FAST_IO_DISPATCH);
//
// #ifdef _MSC_VER
// #pragma prefast(push)
// #pragma prefast(disable:28155, "these are all correct")
// #endif
//
//     CdFastIoDispatch.FastIoCheckIfPossible =   CdFastIoCheckIfPossible;  //  CheckForFastIo
//     CdFastIoDispatch.FastIoRead =              FsRtlCopyRead;            //  Read
//     CdFastIoDispatch.FastIoQueryBasicInfo =    CdFastQueryBasicInfo;     //  QueryBasicInfo
//     CdFastIoDispatch.FastIoQueryStandardInfo = CdFastQueryStdInfo;       //  QueryStandardInfo
//     CdFastIoDispatch.FastIoLock =              CdFastLock;               //  Lock
//     CdFastIoDispatch.FastIoUnlockSingle =      CdFastUnlockSingle;       //  UnlockSingle
//     CdFastIoDispatch.FastIoUnlockAll =         CdFastUnlockAll;          //  UnlockAll
//     CdFastIoDispatch.FastIoUnlockAllByKey =    CdFastUnlockAllByKey;     //  UnlockAllByKey
//     //
//     //  This callback has been replaced by CdFilterCallbackAcquireForCreateSection.
//     //
//
//     CdFastIoDispatch.AcquireFileForNtCreateSection =  NULL;
//     CdFastIoDispatch.ReleaseFileForNtCreateSection =  CdReleaseForCreateSection;
//     CdFastIoDispatch.FastIoQueryNetworkOpenInfo =     CdFastQueryNetworkInfo;   //  QueryNetworkInfo
//
//     CdFastIoDispatch.MdlRead = FsRtlMdlReadDev;
//     CdFastIoDispatch.MdlReadComplete = FsRtlMdlReadCompleteDev;
//     CdFastIoDispatch.PrepareMdlWrite = FsRtlPrepareMdlWriteDev;
//     CdFastIoDispatch.MdlWriteComplete = FsRtlMdlWriteCompleteDev;
//
// #ifdef _MSC_VER
// #pragma prefast(pop)
// #endif
//
//     //
//     //  Initialize the CdData structure.
//     //
//
//     RtlZeroMemory( &CdData, sizeof( CD_DATA ));
//
//     CdData.NodeTypeCode = CDFS_NTC_DATA_HEADER;
//     CdData.NodeByteSize = sizeof( CD_DATA );
//
//     CdData.DriverObject = DriverObject;
//     CdData.FileSystemDeviceObject = FileSystemDeviceObject;
// #ifdef __REACTOS__
//     CdData.HddFileSystemDeviceObject = HddFileSystemDeviceObject;
// #endif
//
//     InitializeListHead( &CdData.VcbQueue );
//
//     ExInitializeResourceLite( &CdData.DataResource );
//
//     //
//     //  Initialize the cache manager callback routines
//     //
//
//     CdData.CacheManagerCallbacks.AcquireForLazyWrite  = (PVOID)&CdAcquireForCache;/* ReactOS Change: GCC "assignment from incompatible pointer type" */
//     CdData.CacheManagerCallbacks.ReleaseFromLazyWrite = (PVOID)&CdReleaseFromCache;/* ReactOS Change: GCC "assignment from incompatible pointer type" */
//     CdData.CacheManagerCallbacks.AcquireForReadAhead  = (PVOID)&CdAcquireForCache;/* ReactOS Change: GCC "assignment from incompatible pointer type" */
//     CdData.CacheManagerCallbacks.ReleaseFromReadAhead = (PVOID)&CdReleaseFromCache;/* ReactOS Change: GCC "assignment from incompatible pointer type" */
//
//     CdData.CacheManagerVolumeCallbacks.AcquireForLazyWrite  = &CdNoopAcquire;
//     CdData.CacheManagerVolumeCallbacks.ReleaseFromLazyWrite = &CdNoopRelease;
//     CdData.CacheManagerVolumeCallbacks.AcquireForReadAhead  = &CdNoopAcquire;
//     CdData.CacheManagerVolumeCallbacks.ReleaseFromReadAhead = &CdNoopRelease;
//
//     //
//     //  Initialize the lock mutex and the async and delay close queues.
//     //
//
//     ExInitializeFastMutex( &CdData.CdDataMutex );
//     InitializeListHead( &CdData.AsyncCloseQueue );
//     InitializeListHead( &CdData.DelayedCloseQueue );
//
//     CdData.CloseItem = IoAllocateWorkItem (FileSystemDeviceObject);
//     if (CdData.CloseItem == NULL) {
//
//         ExDeleteResourceLite( &CdData.DataResource );
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     //
//     //  Do the initialization based on the system size.
//     //
//
//     switch (MmQuerySystemSize()) {
//
//     case MmSmallSystem:
//
//         CdData.IrpContextMaxDepth = 4;
//         CdData.MaxDelayedCloseCount = 8;
//         CdData.MinDelayedCloseCount = 2;
//         break;
//
//     case MmMediumSystem:
//
//         CdData.IrpContextMaxDepth = 8;
//         CdData.MaxDelayedCloseCount = 24;
//         CdData.MinDelayedCloseCount = 6;
//         break;
//
//     case MmLargeSystem:
//
//         CdData.IrpContextMaxDepth = 32;
//         CdData.MaxDelayedCloseCount = 72;
//         CdData.MinDelayedCloseCount = 18;
//         break;
//     }
//     return STATUS_SUCCESS;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
