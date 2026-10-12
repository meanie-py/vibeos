use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAcquireResourceExclusiveLite, ExDeleteResourceLite, ExInitializeResourceLite, ExReleaseResourceLite, IoGetDeviceObjectPointer, KeEnterCriticalRegion, KeInitializeEvent, KeLeaveCriticalRegion, PCUNICODE_STRING, PDEVICE_OBJECT, PUNICODE_STRING, RtlEqualUnicodeString, RtlFreeUnicodeString, _EVENT_TYPE_SynchronizationEvent, ZwLoadDriver, ZwUnloadDriver};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations in this preamble: it contains only SAL annotations and a dangling `NTSTATUS` return type with no function following it, so nothing is emitted.
#[no_mangle]
pub unsafe extern "C" fn LibraryFree(LibModule: PLIBRARY_MODULE) {
    DPRINT_TRACE_ENTRY!();

    if LibModule.is_null() {
        DPRINT_ERROR!(("LibModule is NULL\n"));
        return;
    }

    if !(*LibModule).ServicePath.Buffer.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*LibModule).ServicePath));
        (*LibModule).ServicePath.Buffer = core::ptr::null_mut();
    }

    if !(*LibModule).ImageName.Buffer.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*LibModule).ImageName));
        (*LibModule).ImageName.Buffer = core::ptr::null_mut();
    }

    ExDeleteResourceLite(core::ptr::addr_of_mut!((*LibModule).ClientsListLock));
    RtlZeroMemory!(
        LibModule as *mut core::ffi::c_void,
        core::mem::size_of::<_LIBRARY_MODULE>(),
    );
    ExFreePoolWithTag(LibModule as PVOID, WDFLDR_TAG);

    DPRINT_TRACE_EXIT!();
}
#[no_mangle]
pub unsafe extern "C" fn LibraryCreate(
    LibraryInfo: PWDF_LIBRARY_INFO,
    ServicePath: PCUNICODE_STRING,
    OutLibraryModule: *mut PLIBRARY_MODULE,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;
    let mut LibModule: PLIBRARY_MODULE = core::ptr::null_mut();

    DPRINT_TRACE_ENTRY!();
    if ServicePath.is_null() || OutLibraryModule.is_null() {
        DPRINT_ERROR!(
            "Invalid parameters: ServicePath=%p, OutLibraryModule=%p\n",
            ServicePath,
            OutLibraryModule
        );
        return STATUS_INVALID_PARAMETER;
    }

    *OutLibraryModule = core::ptr::null_mut();

    LibModule = ExAllocatePoolZero(
        NonPagedPool,
        (core::mem::size_of::<LIBRARY_MODULE>() as SIZE_T) as _,
        WDFLDR_TAG,
    ) as PLIBRARY_MODULE;
    if LibModule.is_null() {
        DPRINT_ERROR!("Failed to allocate library module structure\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*LibModule).LibraryRefCount = 1;
    (*LibModule).ImplicitlyLoaded = (LibraryInfo.is_null()) as BOOLEAN;
    (*LibModule).IsBootDriver = ServiceCheckBootStart(ServicePath as PUNICODE_STRING);
    (*LibModule).LoaderThread = (KeGetCurrentThread()).cast();

    InitializeListHead(&raw mut (*LibModule).ClientsListHead);
    InitializeListHead(&raw mut (*LibModule).ClassListHead);

    status = ExInitializeResourceLite(&raw mut (*LibModule).ClientsListLock);
    'failure: {
        if !NT_SUCCESS!(status) {
            DPRINT_ERROR!(
                "ExInitializeResourceLite failed with status 0x%x\n",
                status
            );
            break 'failure;
        }

        KeInitializeEvent(
            &raw mut (*LibModule).LoaderEvent,
            _EVENT_TYPE_SynchronizationEvent,
            FALSE,
        );

        // Only set library information if provided
        if !LibraryInfo.is_null() {
            (*LibModule).LibraryInfo = LibraryInfo;
            (*LibModule).Version = (*LibraryInfo).Version;
            DPRINT_VERBOSE!(
                "Library info provided: Version %d.%d.%d\n",
                (*LibraryInfo).Version.Major,
                (*LibraryInfo).Version.Minor,
                (*LibraryInfo).Version.Build
            );
        }

        (*LibModule).ServicePath.Buffer = ExAllocatePoolWithTag(
            PagedPool,
            (*ServicePath).MaximumLength as SIZE_T,
            WDFLDR_TAG,
        ) as PWCH;
        if (*LibModule).ServicePath.Buffer.is_null() {
            DPRINT_ERROR!("Failed to allocate service path buffer\n");
            status = STATUS_INSUFFICIENT_RESOURCES;
            break 'failure;
        }

        (*LibModule).ServicePath.MaximumLength = (*ServicePath).MaximumLength;
        (*LibModule).ServicePath.Length = (*ServicePath).Length;
        RtlCopyMemory!(
            (*LibModule).ServicePath.Buffer as *mut core::ffi::c_void,
            (*ServicePath).Buffer as *const core::ffi::c_void,
            (*ServicePath).Length as usize,
        );

        status = GetImageName(ServicePath, &raw mut (*LibModule).ImageName);
        if !NT_SUCCESS!(status) {
            DPRINT_ERROR!("Failed to get image name from service path\n");
            break 'failure;
        }

        status = GetImageInfo(
            &raw mut (*LibModule).ImageName as PCUNICODE_STRING,
            &raw mut (*LibModule).ImageAddress,
            &raw mut (*LibModule).ImageSize,
        );
        if NT_SUCCESS!(status) {
            // Insert into loaded modules list. The LoadedModulesListLock is held here
            InsertHeadList(
                &raw mut WdfLdrGlobals.LoadedModulesList,
                &raw mut (*LibModule).LibraryListEntry,
            );

            *OutLibraryModule = LibModule;

            DPRINT_VERBOSE!(
                "Successfully created library module %wZ (Image: %wZ, Base: %p, Size: 0x%x)\n",
                &raw mut (*LibModule).ServicePath,
                &raw mut (*LibModule).ImageName,
                (*LibModule).ImageAddress,
                (*LibModule).ImageSize
            );

            DPRINT_TRACE_EXIT!();
            return STATUS_SUCCESS;
        }

        DPRINT_ERROR!("GetImageInfo failed with status 0x%x\n", status);
    }

    if !LibModule.is_null() {
        LibraryFree(LibModule);
    }

    DPRINT_TRACE_EXIT!();
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn LibraryOpen(
    LibModule: PLIBRARY_MODULE,
    ObjectName: PCUNICODE_STRING,
) -> NTSTATUS {
    let mut deviceObject: PDEVICE_OBJECT = core::ptr::null_mut();

    if ObjectName.is_null() {
        return STATUS_SUCCESS;
    }

    let mut status: NTSTATUS = IoGetDeviceObjectPointer(
        ObjectName as PUNICODE_STRING,
        FILE_READ_DATA as ACCESS_MASK,
        core::ptr::addr_of_mut!((*LibModule).LibraryFileObject),
        core::ptr::addr_of_mut!(deviceObject),
    );

    if NT_SUCCESS!(status) {
        (*LibModule).LibraryDriverObject = (*deviceObject).DriverObject;
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn LibraryClose(LibModule: PLIBRARY_MODULE) {
    if !(*LibModule).LibraryFileObject.is_null() {
        ObDereferenceObject((*LibModule).LibraryFileObject as PVOID);
        (*LibModule).LibraryFileObject = ::core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn LibraryFindOrLoad(mut ServicePath: PCUNICODE_STRING, mut LibModule: *mut PLIBRARY_MODULE) -> NTSTATUS {
    let mut loadNeeded: BOOLEAN = FALSE;
    let mut status: NTSTATUS = 0;

    FxLdrAcquireLoadedModuleLock();
    let mut pLibModule: PLIBRARY_MODULE = FindLibraryByServicePathLocked(ServicePath);

    if pLibModule.is_null() {
        // Client driver is loaded before the library, we need to create it here
        status = LibraryCreate(
            core::ptr::null_mut(),
            ServicePath,
            core::ptr::addr_of_mut!(pLibModule),
        );
        if NT_SUCCESS!(status) {
            loadNeeded = TRUE;
        }
    } else {
        status = STATUS_SUCCESS;
    }

    // Reference the library
    if !pLibModule.is_null() {
        InterlockedIncrement(core::ptr::addr_of_mut!((*pLibModule).LibraryRefCount));
    }

    FxLdrReleaseLoadedModuleLock();

    // We've just created a library, time to load the actual driver
    if loadNeeded != 0 {
        // This will call WdfRegisterLibrary (as part of KMDF's DriverEntry)
        status = ZwLoadDriver(ServicePath as PUNICODE_STRING);
        if NT_SUCCESS!(status) && (*pLibModule).LibraryInfo.is_null() {
            __DBGPRINT!((
                "ZwLoadDriver(%wZ) succeeded but Libray information was returned\n",
                ServicePath
            ));
            status = STATUS_NOT_SAFE_MODE_DRIVER;
        } else if !NT_SUCCESS!(status) {
            __DBGPRINT!((
                "ZwLoadDriver(%wZ) failed with status %x\n",
                ServicePath,
                status
            ));
        }

        if !NT_SUCCESS!(status) {
            LibraryDereference(pLibModule);
            // Cleanup created library
            ASSERT!((*pLibModule).LibraryInfo.is_null());
            LibraryDereference(pLibModule);
            pLibModule = core::ptr::null_mut();
        }
    }

    *LibModule = pLibModule;
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn LibraryReference(LibModule: PLIBRARY_MODULE) {
    FxLdrAcquireLoadedModuleLock();
    InterlockedIncrement(core::ptr::addr_of_mut!((*LibModule).LibraryRefCount) as *mut i32);
    FxLdrReleaseLoadedModuleLock();
}
#[no_mangle]
pub unsafe extern "system" fn LibraryReleaseReference(mut LibModule: PLIBRARY_MODULE) {
    let mut refCount: LONG;
    DPRINT_TRACE_ENTRY!();

    refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*LibModule).LibraryRefCount));
    DPRINT_VERBOSE!(
        "Released reference to library %wZ, RefCount=%d\n",
        core::ptr::addr_of_mut!((*LibModule).ServicePath),
        refCount
    );

    if refCount <= 0 {
        DPRINT!(
            "Library %wZ reference count reached zero, unloading\n",
            core::ptr::addr_of_mut!((*LibModule).ServicePath)
        );
        LibraryUnload(LibModule);
    }

    DPRINT_TRACE_EXIT!();
}
#[no_mangle]
pub unsafe extern "C" fn LibraryDereference(LibModule: PLIBRARY_MODULE) {
    let mut cleanupNeeded: BOOLEAN = FALSE;

    FxLdrAcquireLoadedModuleLock();
    if InterlockedDecrement(core::ptr::addr_of_mut!((*LibModule).LibraryRefCount) as *mut ::core::ffi::c_long) == 0 {
        if IsListEmpty(core::ptr::addr_of_mut!((*LibModule).LibraryListEntry)) == 0 {
            RemoveEntryList(core::ptr::addr_of_mut!((*LibModule).LibraryListEntry));
            InitializeListHead(core::ptr::addr_of_mut!((*LibModule).LibraryListEntry)); // Indicate removal process
            cleanupNeeded = TRUE;
        }
    }
    FxLdrReleaseLoadedModuleLock();

    if cleanupNeeded != 0 {
        if !(*LibModule).LibraryInfo.is_null() {
            LibraryUnload(LibModule);
        } else {
            LibraryClose(LibModule);
        }
        LibraryFree(LibModule);
    }
}
#[no_mangle]
pub unsafe extern "C" fn LibraryUnload(LibModule: PLIBRARY_MODULE) {
    let mut status: NTSTATUS;

    /* This occurs because of how WDF ClassDrivers behave. */
    if (*LibModule).IsBootDriver != 0 {
        return;
    }

    ASSERT!(!(*LibModule).LibraryInfo.is_null());

    status = (*(*LibModule).LibraryInfo)
        .LibraryDecommission
        .unwrap_unchecked()();
    if !NT_SUCCESS!(status) {
        __DBGPRINT!("LibraryDecommission failed %X\n", status);
    }

    (*LibModule).LibraryInfo = core::ptr::null_mut();

    __DBGPRINT!("Unloading library %wZ\n", core::ptr::addr_of_mut!((*LibModule).ServicePath));

    LibraryClose(LibModule);

    status = ZwUnloadDriver(core::ptr::addr_of_mut!((*LibModule).ServicePath));
    if !NT_SUCCESS!(status) {
        __DBGPRINT!(
            "unload of %wZ returned 0x%x (this may not be a true error if someone else attempted to stop the service first)\n",
            core::ptr::addr_of_mut!((*LibModule).ServicePath),
            status
        );
    }

    // Free the library module directly - caller handles the cleanup sequence
    LibraryFree(LibModule);
}
#[no_mangle]
pub unsafe extern "C" fn LibraryAcquireClientLock(LibModule: PLIBRARY_MODULE) -> BOOLEAN {
    KeEnterCriticalRegion();
    return ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!((*LibModule).ClientsListLock), TRUE);
}
#[no_mangle]
pub unsafe extern "C" fn LibraryReleaseClientLock(LibModule: PLIBRARY_MODULE) {
    ExReleaseResourceLite(core::ptr::addr_of_mut!((*LibModule).ClientsListLock));
    KeLeaveCriticalRegion();
}

unsafe extern "C" fn ClientFree(ClientModule: PCLIENT_MODULE) {
    if !(*ClientModule).ImageName.Buffer.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*ClientModule).ImageName));
    }
    ExFreePoolWithTag(ClientModule as PVOID, WDFLDR_TAG as ULONG);
}
#[no_mangle]
pub unsafe extern "C" fn LibraryLinkInClient(LibModule: PLIBRARY_MODULE, ServicePath: PUNICODE_STRING, BindInfo: PWDF_BIND_INFO, Context: PVOID, OutClientModule: *mut PCLIENT_MODULE) -> NTSTATUS {
    let mut clientModule: PCLIENT_MODULE;
    let mut status: NTSTATUS;

    clientModule = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<CLIENT_MODULE>() as SIZE_T,
        WDFLDR_TAG,
    ) as PCLIENT_MODULE;

    if clientModule.is_null() {
        __DBGPRINT!("ERROR: ExAllocatePoolWithTag failed\n");
        __DBGPRINT!("ERROR: Client module NOT linked\n");

        return STATUS_INSUFFICIENT_RESOURCES;
    }

    *clientModule = CLIENT_MODULE {
        Context: Context,
        Info: BindInfo,
        ..core::mem::zeroed::<CLIENT_MODULE>()
    };

    InitializeListHead(core::ptr::addr_of_mut!((*clientModule).ClassListHead));
    InitializeListHead(core::ptr::addr_of_mut!((*clientModule).LibListEntry));

    status = GetImageName(
        ServicePath,
        core::ptr::addr_of_mut!((*clientModule).ImageName),
    );
    if NT_SUCCESS!(status) {
        __DBGPRINT!(
            "Client Image Name: %wZ\n",
            core::ptr::addr_of_mut!((*clientModule).ImageName)
        );

        status = GetImageInfo(
            core::ptr::addr_of_mut!((*clientModule).ImageName),
            core::ptr::addr_of_mut!((*clientModule).ImageAddr),
            core::ptr::addr_of_mut!((*clientModule).ImageSize),
        );
        if !NT_SUCCESS!(status) {
            __DBGPRINT!("GetImageInfo failed with status 0x%x\n", status);
            __DBGPRINT!("ERROR: Client module NOT linked\n");
            ClientFree(clientModule);

            return status;
        }
    }

    LibraryAcquireClientLock(LibModule);
    InsertHeadList(
        core::ptr::addr_of_mut!((*LibModule).ClientsListHead),
        core::ptr::addr_of_mut!((*clientModule).LibListEntry),
    );
    InterlockedIncrement(core::ptr::addr_of_mut!((*LibModule).ClientRefCount));
    LibraryReleaseClientLock(LibModule);

    *OutClientModule = clientModule;
    return STATUS_SUCCESS;
}

unsafe extern "C" fn LibraryClientDereference(LibModule: PLIBRARY_MODULE) {
    let refs: LONG = InterlockedDecrement(core::ptr::addr_of_mut!((*LibModule).ClientRefCount)) as LONG;

    __DBGPRINT!(("Dereference module %wZ, %d references left\n", core::ptr::addr_of_mut!((*LibModule).ServicePath), refs));

    if refs <= 0 {
        LibraryUnload(LibModule);
    }
}
#[no_mangle]
pub unsafe extern "C" fn LibraryUnlinkClient(LibModule: PLIBRARY_MODULE, BindInfo: PWDF_BIND_INFO) -> BOOLEAN {
    LibraryAcquireClientLock(LibModule);

    // search in library clients entry to remove
    let mut found: BOOLEAN = FALSE;
    let mut clientModule: PCLIENT_MODULE = core::ptr::null_mut();
    let mut entry: PLIST_ENTRY = (*LibModule).ClientsListHead.Flink;
    while entry != core::ptr::addr_of_mut!((*LibModule).ClientsListHead) {
        clientModule = (entry as *mut u8)
            .sub(core::mem::offset_of!(CLIENT_MODULE, LibListEntry))
            as PCLIENT_MODULE;
        if (*clientModule).Info == BindInfo {
            found = TRUE;
            break;
        }
        entry = (*entry).Flink;
    }

    if found != 0 {
        RemoveEntryList(core::ptr::addr_of_mut!((*clientModule).LibListEntry));
        LibraryClientDereference(LibModule);
    }

    LibraryReleaseClientLock(LibModule);

    if found != 0 {
        InitializeListHead(core::ptr::addr_of_mut!((*clientModule).LibListEntry));
        ClientFree(clientModule);
        return TRUE;
    }

    __DBGPRINT!(
        b"ERROR: Client module %p, bind %p NOT found\n\0".as_ptr() as *const i8,
        LibModule,
        BindInfo,
    );
    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn FindLibraryByServicePathLocked(
    ServicePath: PCUNICODE_STRING,
) -> PLIBRARY_MODULE {
    let mut foundModule: PLIBRARY_MODULE = core::ptr::null_mut();
    let mut needleName: _UNICODE_STRING = core::mem::zeroed();

    GetNameFromPath(ServicePath, core::ptr::addr_of_mut!(needleName));

    let mut entry: PLIST_ENTRY =
        (*core::ptr::addr_of_mut!(WdfLdrGlobals.LoadedModulesList)).Flink;
    while entry != core::ptr::addr_of_mut!(WdfLdrGlobals.LoadedModulesList) as PLIST_ENTRY {
        let mut haystackName: _UNICODE_STRING = core::mem::zeroed();
        let mut currentLib: PLIBRARY_MODULE =
            CONTAINING_RECORD!(entry, LIBRARY_MODULE, LibraryListEntry);

        GetNameFromPath(
            core::ptr::addr_of_mut!((*currentLib).ServicePath) as PCUNICODE_STRING,
            core::ptr::addr_of_mut!(haystackName),
        );

        if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!(needleName),
            core::ptr::addr_of_mut!(haystackName),
            TRUE as BOOLEAN,
        ) != 0
        {
            foundModule = currentLib;
            break;
        }

        entry = (*entry).Flink;
    }

    return foundModule;
}

#[no_mangle]
pub unsafe extern "system" fn FindModuleByClientService(RegistryPath: PUNICODE_STRING, Library: *mut PLIBRARY_MODULE) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut imageName: _UNICODE_STRING = core::mem::zeroed();
    let mut foundModule: PLIBRARY_MODULE = core::ptr::null_mut();

    DPRINT_TRACE_ENTRY!();

    if RegistryPath.is_null() || Library.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *Library = core::ptr::null_mut();

    status = GetImageName(RegistryPath, core::ptr::addr_of_mut!(imageName));
    'exit: {
        if !NT_SUCCESS!(status) {
            if WdfLdrDiags.__bindgen_anon_1.DiagFlags & DIAGFLAG_ENABLED != 0 {
                DbgPrint(b"WdfLdr: FindModuleByClientService - \0".as_ptr() as *const i8);
                DbgPrint(
                    b"WdfLdr: FindModuleByClientService: GetImageName for %wZ failed with status 0x%x\n\0"
                        .as_ptr() as *const i8,
                    RegistryPath,
                    status,
                );
            }
            break 'exit;
        }

        /* Search through loaded modules by comparing image names only */
        let mut entry: PLIST_ENTRY = WdfLdrGlobals.LoadedModulesList.Flink;
        while entry != core::ptr::addr_of_mut!(WdfLdrGlobals.LoadedModulesList) {
            let currentLib: PLIBRARY_MODULE =
                CONTAINING_RECORD!(entry, LIBRARY_MODULE, LibraryListEntry);

            if RtlEqualUnicodeString(
                core::ptr::addr_of_mut!(imageName),
                core::ptr::addr_of_mut!((*currentLib).ImageName),
                TRUE,
            ) != 0
            {
                foundModule = currentLib;
                break;
            }
            entry = (*entry).Flink;
        }

        *Library = foundModule;
        status = if !foundModule.is_null() {
            STATUS_SUCCESS
        } else {
            STATUS_NOT_FOUND
        };
    }

    // Exit:
    if !imageName.Buffer.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(imageName));
    }

    DPRINT_TRACE_EXIT!();
    return status;
}
