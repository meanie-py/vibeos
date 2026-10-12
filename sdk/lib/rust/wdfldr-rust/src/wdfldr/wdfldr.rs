// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_7276DD3725DD: [u16; 72] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,
    0x005C, 0x0057, 0x0064, 0x0066, 0x005C, 0x004B, 0x006D, 0x0064, 0x0066, 0x005C,
    0x0044, 0x0069, 0x0061, 0x0067, 0x006E, 0x006F, 0x0073, 0x0074, 0x0069, 0x0063,
    0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_76F43129F4E9: [u16; 11] = [
    0x0044u16, 0x0062, 0x0067, 0x0050, 0x0072, 0x0069, 0x006E, 0x0074, 0x004F, 0x006E,
    0x0000,
];
pub static RUSTCONV_WSTR_78945956DEA4: [u16; 15] = [
    0x0056u16, 0x0065, 0x0072, 0x0062, 0x006F, 0x0073, 0x0065, 0x004C, 0x006F, 0x0067,
    0x0067, 0x0069, 0x006E, 0x0067, 0x0000,
];
pub static RUSTCONV_WSTR_A0602B8678A9: [u16; 11] = [
    0x0054u16, 0x0072, 0x0061, 0x0063, 0x0065, 0x0045, 0x006E, 0x0074, 0x0072, 0x0079,
    0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{ExDeleteResourceLite, ExInitializeResourceLite, _OBJECT_ATTRIBUTES, PCUNICODE_STRING, PDRIVER_OBJECT, POSVERSIONINFOW, PUNICODE_STRING, RtlFreeUnicodeString, RtlGetVersion, ZwClose, ZwOpenKey};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_7276DD3725DD: [u16; 72] = [
    0x005Cu16, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
    0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
    0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
    0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,
    0x005C, 0x0057, 0x0064, 0x0066, 0x005C, 0x004B, 0x006D, 0x0064, 0x0066, 0x005C,
    0x0044, 0x0069, 0x0061, 0x0067, 0x006E, 0x006F, 0x0073, 0x0074, 0x0069, 0x0063,
    0x0073, 0x0000,
];

static mut diagPath: _UNICODE_STRING = _UNICODE_STRING {
        Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 72]>() as usize)
            .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
            as USHORT,
        MaximumLength: (::core::mem::size_of::<[::core::ffi::c_ushort; 72]>() as usize)
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>() as usize)
            as USHORT,
        Buffer: RUSTCONV_C2R_WSTR_7276DD3725DD.as_ptr() as PWCH,
    };

#[no_mangle]
pub static mut GUID_WDF_LOADER_INTERFACE_STANDARD: GUID = _GUID {
    Data1: 0x49215dff as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xf5ac as ::core::ffi::c_ushort,
    Data3: 0x4901 as ::core::ffi::c_ushort,
    Data4: [
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x54 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut GUID_WDF_LOADER_INTERFACE_DIAGNOSTIC: GUID = _GUID {
    Data1: 0x55905ba4 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x1dd2 as ::core::ffi::c_ushort,
    Data3: 0x45d3 as ::core::ffi::c_ushort,
    Data4: [
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xea as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut GUID_WDF_LOADER_INTERFACE_CLASS_BIND: GUID = _GUID {
    Data1: 0xfa4838cb as ::core::ffi::c_uint,
    Data2: 0x1d08 as ::core::ffi::c_ushort,
    Data3: 0x41e1 as ::core::ffi::c_ushort,
    Data4: [
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xea as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut gAlreadyInitialized: BOOLEAN = FALSE as BOOLEAN;

#[no_mangle]
pub static mut gAlreadyUnloaded: BOOLEAN = FALSE as BOOLEAN;

#[no_mangle]
pub static mut WdfLdrDiags: WDFLDR_DIAGS = _WDFLDR_DIAGS {
    __bindgen_anon_1: _WDFLDR_DIAGS__bindgen_ty_1 { DiagFlags: 0 as ::core::ffi::c_int as UINT32,
    },
};

#[no_mangle]
pub static mut WdfLdrGlobals: WDF_LDR_GLOBALS = _WDF_LDR_GLOBALS {
    OsVersion: _OSVERSIONINFOEXW {
        dwOSVersionInfoSize: 0,
        dwMajorVersion: 0,
        dwMinorVersion: 0,
        dwBuildNumber: 0,
        dwPlatformId: 0,
        szCSDVersion: [0; 128],
        wServicePackMajor: 0,
        wServicePackMinor: 0,
        wSuiteMask: 0,
        wProductType: 0,
        wReserved: 0,
    },
    // All zeros, as the C's zero-initialized global. ERESOURCE's fields differ per arch
    // (no Reserved2 on i386), and c2rust spelled them from one arch.
    LoadedModulesListLock: unsafe { core::mem::zeroed() },
    LoadedModulesList: _LIST_ENTRY {
        Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
        Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    },
};

#[no_mangle]
pub unsafe extern "system" fn WdfLdrUnload(DriverObject: PDRIVER_OBJECT) {
    DllUnload();
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    (*DriverObject).DriverUnload =
        Some(WdfLdrUnload);
    return DllInitialize(RegistryPath);
}
#[no_mangle]
pub unsafe extern "system" fn WdfLdrDiagnosticsValueByNameAsULONG(
    ValueName: PUNICODE_STRING,
    Value: PULONG,
) -> NTSTATUS {
    let mut keyHandle: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;

    if ValueName.is_null() || Value.is_null() {
        __DBGPRINT!(b"ERROR: Invalid Input Parameter\n\0".as_ptr() as *const i8);
        return STATUS_INVALID_PARAMETER;
    }

    *Value = 0; // for compatibility

    if (KeGetCurrentIrql() as i32) > (PASSIVE_LEVEL as i32) {
        __DBGPRINT!(b"Not at PASSIVE_LEVEL\n\0".as_ptr() as *const i8);
        return STATUS_INVALID_PARAMETER;
    }

    // open framework diagnostics key
    let mut attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    // `diagPath` is the C's function-local `static` (lifetime storage), hoisted to file scope.

    // InitializeObjectAttributes expanded inline
    attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    attributes.RootDirectory = core::ptr::null_mut();
    attributes.ObjectName = core::ptr::addr_of_mut!(diagPath);
    attributes.Attributes = (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as ULONG;
    attributes.SecurityDescriptor = core::ptr::null_mut();
    attributes.SecurityQualityOfService = core::ptr::null_mut();

    status = ZwOpenKey(
        core::ptr::addr_of_mut!(keyHandle),
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(attributes),
    );

    if NT_SUCCESS!(status) {
        status = FxLdrQueryUlong(keyHandle, ValueName, Value);

        __DBGPRINT!(
            b"Status %x, value 0x%x\n\0".as_ptr() as *const i8,
            status,
            *Value,
        );

        ZwClose(keyHandle);
    } else {
        __DBGPRINT!(
            b"ERROR: ZwOpenKey failed with Status 0x%x\n\0".as_ptr() as *const i8,
            status,
        );
    }

    return status;
}

#[no_mangle]
pub unsafe extern "system" fn DllInitialize(RegistryPath: PUNICODE_STRING) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut dbgPrintOn: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_76F43129F4E9.as_ptr());
    let mut verboseLogging: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_78945956DEA4.as_ptr());
    let mut traceEntry: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_A0602B8678A9.as_ptr());
    let mut diagValue: ULONG = 0;

    UNREFERENCED_PARAMETER!(RegistryPath);

    if gAlreadyInitialized != 0 {
        return STATUS_SUCCESS;
    }

    gAlreadyInitialized = TRUE;
    RtlZeroMemory!(
        &raw mut WdfLdrGlobals as *mut core::ffi::c_void,
        core::mem::size_of::<WDF_LDR_GLOBALS>(),
    );
    RtlZeroMemory!(
        &raw mut WdfLdrDiags as *mut core::ffi::c_void,
        core::mem::size_of::<WDFLDR_DIAGS>(),
    );

    InitializeListHead(&raw mut WdfLdrGlobals.LoadedModulesList);
    status = ExInitializeResourceLite(&raw mut WdfLdrGlobals.LoadedModulesListLock);
    if !NT_SUCCESS!(status) {
        DPRINT_ERROR!("ExInitializeResourceLite failed with Status 0x%x\n", status);
        return status;
    }

    status = WdfLdrDiagnosticsValueByNameAsULONG(&raw mut dbgPrintOn, &raw mut diagValue);
    if NT_SUCCESS!(status) && diagValue != 0 {
        WdfLdrDiags.__bindgen_anon_1.DiagFlags |= DIAGFLAG_ENABLED;
    }

    status = WdfLdrDiagnosticsValueByNameAsULONG(&raw mut verboseLogging, &raw mut diagValue);
    if NT_SUCCESS!(status) && diagValue != 0 {
        WdfLdrDiags.__bindgen_anon_1.DiagFlags |= DIAGFLAG_VERBOSE_LOGGING;
    }

    status = WdfLdrDiagnosticsValueByNameAsULONG(&raw mut traceEntry, &raw mut diagValue);
    if NT_SUCCESS!(status) && diagValue != 0 {
        WdfLdrDiags.__bindgen_anon_1.DiagFlags |= (DIAGFLAG_TRACE_FUNCTION_ENTRY | DIAGFLAG_TRACE_FUNCTION_EXIT);
    }

    status = AuxKlibInitialize();
    if NT_SUCCESS!(status) {
        RtlGetVersion(&raw mut WdfLdrGlobals.OsVersion as POSVERSIONINFOW);
        DPRINT!(
            "Initialized WdfLdr - OS Version %d.%d.%d\n",
            WdfLdrGlobals.OsVersion.dwMajorVersion,
            WdfLdrGlobals.OsVersion.dwMinorVersion,
            WdfLdrGlobals.OsVersion.dwBuildNumber
        );
    } else {
        DPRINT_ERROR!("AuxKlibInitialize failed with Status 0x%x\n", status);
        ExDeleteResourceLite(&raw mut WdfLdrGlobals.LoadedModulesListLock);
        gAlreadyInitialized = FALSE;
        return status;
    }

    DPRINT!("WDF Loader initialization completed successfully\n");
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DllUnload() {
    if gAlreadyUnloaded != 0 {
        return;
    }

    DPRINT_TRACE_ENTRY!();
    gAlreadyUnloaded = TRUE;

    //
    // Unload all loaded libraries - CRITICAL: proper cleanup sequence
    //
    while IsListEmpty(core::ptr::addr_of_mut!(WdfLdrGlobals.LoadedModulesList)) == 0 {
        let mut entry: PLIST_ENTRY = RemoveHeadList(core::ptr::addr_of_mut!(WdfLdrGlobals.LoadedModulesList));
        let mut module: PLIBRARY_MODULE = CONTAINING_RECORD!(entry, LIBRARY_MODULE, LibraryListEntry);

        DPRINT!(("Unloading module %p (%wZ)\n", module, core::ptr::addr_of_mut!((*module).ServicePath)));

        InitializeListHead(core::ptr::addr_of_mut!((*module).LibraryListEntry));
        LibraryUnloadClasses(module);

        if !(*module).LibraryInfo.is_null() {
            LibraryUnload(module);
        } else {
            LibraryClose(module);
            LibraryFree(module);
        }
    }

    ExDeleteResourceLite(core::ptr::addr_of_mut!(WdfLdrGlobals.LoadedModulesListLock));
    DPRINT_TRACE_EXIT!();
}

#[no_mangle]
pub unsafe extern "system" fn WdfLdrQueryInterface(
    LoaderInterface: PWDF_INTERFACE_HEADER,
) -> NTSTATUS {
    DPRINT_TRACE_ENTRY!();

    if LoaderInterface.is_null() {
        DPRINT_ERROR!("LoaderInterface is NULL\n");
        return STATUS_INVALID_PARAMETER;
    }

    if (*LoaderInterface).InterfaceType.is_null() {
        DPRINT_ERROR!("InterfaceType is NULL\n");
        return STATUS_INVALID_PARAMETER;
    }

    if IsEqualGUID!(
        (*LoaderInterface).InterfaceType,
        core::ptr::addr_of!(GUID_WDF_LOADER_INTERFACE_STANDARD),
    ) != false
    {
        if (*LoaderInterface).InterfaceSize < (core::mem::size_of::<WDF_LOADER_INTERFACE>()) as _ {
            DPRINT_ERROR!(
                "Interface size too small: %u, expected: %u\n",
                (*LoaderInterface).InterfaceSize,
                core::mem::size_of::<WDF_LOADER_INTERFACE>()
            );
            return STATUS_INVALID_PARAMETER;
        }

        let lInterface: PWDF_LOADER_INTERFACE = LoaderInterface as PWDF_LOADER_INTERFACE;

        (*lInterface).RegisterLibrary = Some(WdfRegisterLibrary);
        (*lInterface).VersionBind = Some(WdfVersionBind);
        (*lInterface).VersionUnbind = Some(WdfVersionUnbind);
        (*lInterface).DiagnosticsValueByNameAsULONG = Some(WdfLdrDiagnosticsValueByNameAsULONG);

        DPRINT_VERBOSE!("Provided standard WDF loader interface\n");
        return STATUS_SUCCESS;
    } else if IsEqualGUID!(
        (*LoaderInterface).InterfaceType,
        core::ptr::addr_of!(GUID_WDF_LOADER_INTERFACE_DIAGNOSTIC),
    ) != false
    {
        if (*LoaderInterface).InterfaceSize < (core::mem::size_of::<WDF_LOADER_INTERFACE_DIAGNOSTIC>()) as _ {
            DPRINT_ERROR!(
                "Diagnostic interface size too small: %u, expected: %u\n",
                (*LoaderInterface).InterfaceSize,
                core::mem::size_of::<WDF_LOADER_INTERFACE_DIAGNOSTIC>()
            );
            return STATUS_INVALID_PARAMETER;
        }

        let lInterface: PWDF_LOADER_INTERFACE_DIAGNOSTIC =
            LoaderInterface as PWDF_LOADER_INTERFACE_DIAGNOSTIC;

        (*lInterface).DiagnosticsValueByNameAsULONG = Some(WdfLdrDiagnosticsValueByNameAsULONG);

        DPRINT_VERBOSE!("Provided diagnostic interface\n");
        return STATUS_SUCCESS;
    } else if IsEqualGUID!(
        (*LoaderInterface).InterfaceType,
        core::ptr::addr_of!(GUID_WDF_LOADER_INTERFACE_CLASS_BIND),
    ) != false
    {
        if (*LoaderInterface).InterfaceSize < (core::mem::size_of::<WDF_LOADER_INTERFACE_CLASS_BIND>()) as _ {
            DPRINT_ERROR!(
                "Class bind interface size too small: %u, expected: %u\n",
                (*LoaderInterface).InterfaceSize,
                core::mem::size_of::<WDF_LOADER_INTERFACE_CLASS_BIND>()
            );

            return STATUS_INVALID_PARAMETER;
        }

        let lInterface: PWDF_LOADER_INTERFACE_CLASS_BIND =
            LoaderInterface as PWDF_LOADER_INTERFACE_CLASS_BIND;

        (*lInterface).ClassBind = Some(WdfVersionBindClass);
        (*lInterface).ClassUnbind = Some(WdfVersionUnbindClass);

        DPRINT_VERBOSE!("Provided class bind interface\n");
        return STATUS_SUCCESS;
    }

    DPRINT_ERROR!("Unknown interface type requested\n");
    DPRINT_TRACE_EXIT!();
    return STATUS_NOINTERFACE;
}

#[no_mangle]
pub unsafe extern "system" fn WdfRegisterLibrary(
    LibraryInfo: PWDF_LIBRARY_INFO,
    ServicePath: PUNICODE_STRING,
    LibraryDeviceName: PCUNICODE_STRING,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;
    let mut pLibModule: PLIBRARY_MODULE = core::ptr::null_mut();

    PAGED_CODE!();

    /*
     * Wdf01000 may be loaded as a boot-start import consumer before wdfldr's
     * own DriverEntry has run. Initialize the loader globals on this path too.
     */
    DllInitialize(core::ptr::null_mut());

    FxLdrAcquireLoadedModuleLock();
    pLibModule = FindLibraryByServicePathLocked(ServicePath as PCUNICODE_STRING);

    if !pLibModule.is_null() {
        // This happens when a driver which uses KMDF library is loaded before the library itself.
        // In such case, LibraryCreate(NULL, ...) was called in from WdfVersionBind

        ASSERT!((*pLibModule).ImplicitlyLoaded == TRUE);
        ASSERT!((*pLibModule).LibraryInfo.is_null());

        (*pLibModule).Version = (*LibraryInfo).Version;
        (*pLibModule).LibraryInfo = LibraryInfo;

        status = GetImageInfo(
            core::ptr::addr_of_mut!((*pLibModule).ImageName) as PCUNICODE_STRING,
            core::ptr::addr_of_mut!((*pLibModule).ImageAddress),
            core::ptr::addr_of_mut!((*pLibModule).ImageSize),
        );
        if !NT_SUCCESS!(status) {
            __DBGPRINT!((
                "ERROR: GetImageInfo(%wZ) failed with status 0x%x\n",
                (*pLibModule).ImageName,
                status
            ));
        }
    } else {
        status = LibraryCreate(
            LibraryInfo,
            ServicePath as PCUNICODE_STRING,
            core::ptr::addr_of_mut!(pLibModule),
        );
    }

    // Reference the library while we're working on it
    if !pLibModule.is_null() {
        InterlockedIncrement(core::ptr::addr_of_mut!((*pLibModule).LibraryRefCount));
    }

    FxLdrReleaseLoadedModuleLock();

    if pLibModule.is_null() {
        __DBGPRINT!((
            "WdfRegisterLibrary: LibraryCreate(%wZ) failed, status %X\n",
            ServicePath,
            status
        ));
        return status;
    }

    __DBGPRINT!((
        "Registering library(%p) %wZ\n",
        pLibModule,
        core::ptr::addr_of_mut!((*pLibModule).ImageName)
    ));

    // WdfRegisterLibrary is called from the library itself so we're sure it is loaded
    // Now reference the library's device object
    'failure: {
        status = LibraryOpen(pLibModule, LibraryDeviceName);
        if !NT_SUCCESS!(status) {
            __DBGPRINT!((
                "ERROR: LibraryOpen(%wZ) failed with status %x\n",
                LibraryDeviceName,
                status
            ));
            break 'failure;
        }

        // Now call the custom library callback
        status = (*(*pLibModule).LibraryInfo)
            .LibraryCommission
            .unwrap_unchecked()();
        if !NT_SUCCESS!(status) {
            __DBGPRINT!((
                "ERROR: WdfRegisterLibrary: LibraryCommission failed status 0x%X\n",
                status
            ));
            break 'failure;
        }

        // The work is finished
        LibraryDereference(pLibModule);
        return status;
    }

    // Failure:
    // Dereference two times - the library should be cleaned up then
    // Setting LibraryInfo to NULL so ZwUnloadDriver is not called
    (*pLibModule).LibraryInfo = core::ptr::null_mut();
    LibraryDereference(pLibModule);

    // If the library was created in WdfVersionBind, it will be dereferenced there
    if (*pLibModule).ImplicitlyLoaded != 0 {
        LibraryDereference(pLibModule);
    }
    return status;
}

#[no_mangle]
pub unsafe extern "system" fn WdfVersionBind(
    DriverObject: PDRIVER_OBJECT,
    ServicePath: PUNICODE_STRING,
    BindInfo: PWDF_BIND_INFO,
    ComponentGlobals: *mut PWDF_COMPONENT_GLOBALS,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut pLibModule: PLIBRARY_MODULE = core::ptr::null_mut();
    let mut clientModule: PCLIENT_MODULE = core::ptr::null_mut();
    let mut libraryServicePath: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut clientInfo: CLIENT_INFO = CLIENT_INFO {
        Size: 0,
        RegistryPath: core::ptr::null_mut(),
    };
    let mut context: PVOID = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(DriverObject);
    PAGED_CODE!();

    DPRINT_TRACE_ENTRY!();

    /* Lazy init: WdfLdr is loaded on demand (no Boot-start service entry),
     * so its own DriverEntry/DllInitialize may not have run before the
     * first client driver calls in here.  Without this, WdfLdrGlobals
     * stays zero-initialized and the LoadedModulesList walks crash. */
    let _ = DllInitialize(core::ptr::null_mut());

    'exit: {
        if ComponentGlobals.is_null() || BindInfo.is_null() || (*BindInfo).FuncTable.is_null() {
            DPRINT_ERROR!(
                "Invalid parameters: ComponentGlobals=%p, BindInfo=%p\n",
                ComponentGlobals,
                BindInfo,
            );
            status = STATUS_INVALID_PARAMETER;
            break 'exit;
        }

        *ComponentGlobals = core::ptr::null_mut();

        status = ReferenceVersion(BindInfo, core::ptr::addr_of_mut!(pLibModule));
        if !NT_SUCCESS!(status) {
            DPRINT_ERROR!("ReferenceVersion failed with status 0x%x\n", status);
            break 'exit;
        }

        DPRINT_VERBOSE!(
            "Referenced library version for %wZ\n",
            core::ptr::addr_of_mut!((*pLibModule).ServicePath),
        );

        clientInfo.Size = core::mem::size_of::<CLIENT_INFO>() as ULONG;
        clientInfo.RegistryPath = ServicePath;

        status = LibraryLinkInClient(
            pLibModule,
            ServicePath,
            BindInfo,
            core::ptr::addr_of_mut!(clientInfo) as PVOID,
            core::ptr::addr_of_mut!(clientModule),
        );
        if !NT_SUCCESS!(status) {
            DPRINT_ERROR!("LibraryLinkInClient failed with status 0x%x\n", status);
            LibraryDereference(pLibModule);
            break 'exit;
        }

        /* Dereference the library as LibraryLinkInClient has its own reference */
        LibraryDereference(pLibModule);

        (*BindInfo).Module = pLibModule;

        'cleanup: {
            if (*pLibModule).LibraryInfo.is_null() {
                DPRINT_ERROR!("Library not properly initialized - LibraryInfo is NULL\n");
                status = STATUS_INVALID_DEVICE_STATE;
                break 'cleanup;
            }

            if (*(*pLibModule).LibraryInfo).LibraryRegisterClient.is_none() {
                DPRINT_ERROR!("Library missing LibraryRegisterClient function pointer\n");
                status = STATUS_INVALID_DEVICE_STATE;
                break 'cleanup;
            }

            DPRINT_VERBOSE!(
                "Calling LibraryRegisterClient at %p for library %wZ\n",
                (*(*pLibModule).LibraryInfo).LibraryRegisterClient,
                core::ptr::addr_of_mut!((*pLibModule).ServicePath),
            );

            status = (*(*pLibModule).LibraryInfo)
                .LibraryRegisterClient
                .unwrap_unchecked()(
                BindInfo,
                ComponentGlobals,
                core::ptr::addr_of_mut!(context),
            );
            if NT_SUCCESS!(status) {
                (*clientModule).Globals = *ComponentGlobals as PVOID;
                (*clientModule).Context = context; // Pointer to FX_DRIVER_GLOBALS

                DPRINT!(
                    "Successfully bound client %wZ to library %wZ\n",
                    ServicePath,
                    core::ptr::addr_of_mut!(libraryServicePath),
                );
                break 'exit;
            }

            DPRINT_ERROR!("LibraryRegisterClient failed with status 0x%x\n", status);
        }

        /* Cleanup: */
        WdfVersionUnbind(ServicePath, BindInfo, *ComponentGlobals);
    }

    /* Exit: */
    if !libraryServicePath.Buffer.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(libraryServicePath));
    }

    DPRINT_TRACE_EXIT!();
    return status;
}

#[no_mangle]
pub unsafe extern "system" fn WdfVersionUnbind(
    RegistryPath: PUNICODE_STRING,
    BindInfo: PWDF_BIND_INFO,
    ComponentGlobals: PWDF_COMPONENT_GLOBALS,
) -> NTSTATUS {
    let mut pLibModule: PLIBRARY_MODULE = core::ptr::null_mut();
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut unregisterStatus: NTSTATUS = STATUS_SUCCESS;

    PAGED_CODE!();
    DPRINT_TRACE_ENTRY!();

    'exit: {
        if BindInfo.is_null() || (*BindInfo).Module.is_null() {
            DPRINT_ERROR!("Invalid BindInfo or Module is NULL\n");
            status = STATUS_INVALID_PARAMETER;
            break 'exit;
        }

        pLibModule = (*BindInfo).Module as PLIBRARY_MODULE;

        /* Reference the module while working on it and unregister if globals are valid. */
        LibraryReference(pLibModule);
        if !ComponentGlobals.is_null()
            && !(*pLibModule).LibraryInfo.is_null()
            && (*(*pLibModule).LibraryInfo).LibraryUnregisterClient.is_some()
        {
            unregisterStatus = (*(*pLibModule).LibraryInfo)
                .LibraryUnregisterClient
                .unwrap_unchecked()(BindInfo, ComponentGlobals);
            if !NT_SUCCESS!(unregisterStatus) {
                DPRINT_ERROR!("LibraryUnregisterClient failed with status 0x%x\n", unregisterStatus);
            }
        }

        if LibraryUnlinkClient(pLibModule, BindInfo) == 0 {
            DPRINT_ERROR!("LibraryUnlinkClient failed for BindInfo %p\n", BindInfo);
        }

        LibraryDereference(pLibModule);
        (*BindInfo).Module = core::ptr::null_mut();

        DPRINT_VERBOSE!("Successfully unbound client %wZ\n", RegistryPath);
    }

    DPRINT_TRACE_EXIT!();
    return if NT_SUCCESS!(unregisterStatus) {
        status
    } else {
        unregisterStatus
    };
}
#[no_mangle]
pub unsafe extern "system" fn ReferenceVersion(
    Info: PWDF_BIND_INFO,
    Module: *mut PLIBRARY_MODULE,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;
    let mut libraryServicePath: _UNICODE_STRING = core::mem::zeroed();

    DPRINT_TRACE_ENTRY!();

    if Info.is_null() || Module.is_null() {
        DPRINT_ERROR!("Invalid parameters: Info=%p, Module=%p\n", Info, Module);
        return STATUS_INVALID_PARAMETER;
    }

    *Module = core::ptr::null_mut();
    'exit: {
        status = GetVersionServicePath(Info, core::ptr::addr_of_mut!(libraryServicePath));
        if !NT_SUCCESS!(status) {
            DPRINT_ERROR!("GetVersionServicePath failed with status 0x%x\n", status);
            break 'exit;
        }

        status = LibraryFindOrLoad(core::ptr::addr_of_mut!(libraryServicePath), Module);
        if NT_SUCCESS!(status) {
            (*Info).Module = *Module;
            DPRINT_VERBOSE!("Referenced version for library %wZ\n", core::ptr::addr_of_mut!(libraryServicePath));
        } else {
            DPRINT_ERROR!("LibraryFindOrLoad failed with status 0x%x\n", status);
        }
    }

    if !libraryServicePath.Buffer.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(libraryServicePath));
    }

    DPRINT_TRACE_EXIT!();
    return status;
}
#[no_mangle]
pub unsafe extern "system" fn DereferenceVersion(
    Info: PWDF_BIND_INFO,
    Globals: PWDF_COMPONENT_GLOBALS,
) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut unregisterStatus: NTSTATUS = STATUS_SUCCESS;
    let mut pLibModule: PLIBRARY_MODULE;

    DPRINT_TRACE_ENTRY!();

    if Info.is_null() || (*Info).Module.is_null() {
        DPRINT_ERROR!("Invalid Info or Module is NULL\n");
        return STATUS_INVALID_PARAMETER;
    }

    pLibModule = (*Info).Module as PLIBRARY_MODULE;

    // Unregister the client if we have globals
    if !Globals.is_null()
        && !(*pLibModule).LibraryInfo.is_null()
        && (*(*pLibModule).LibraryInfo).LibraryUnregisterClient.is_some()
    {
        unregisterStatus = (*(*pLibModule).LibraryInfo)
            .LibraryUnregisterClient
            .unwrap_unchecked()(Info, Globals);
        if !NT_SUCCESS!(unregisterStatus) {
            DPRINT_ERROR!(
                "LibraryUnregisterClient failed with status 0x%x\n",
                unregisterStatus
            );
            if WdfLdrDiags.__bindgen_anon_1.DiagFlags & DIAGFLAG_ENABLED != 0 {
                DPRINT_VERBOSE!("LibraryUnregisterClient failed\n");
            }
        }
    }

    // Unlink the client
    if LibraryUnlinkClient(pLibModule, Info) == 0 {
        DPRINT_ERROR!("LibraryUnlinkClient failed\n");
        if WdfLdrDiags.__bindgen_anon_1.DiagFlags & DIAGFLAG_ENABLED != 0 {
            DPRINT_VERBOSE!("LibraryUnlinkClient failed\n");
        }
    }

    // Release the reference
    LibraryReleaseReference(pLibModule);

    // Clear the module reference
    (*Info).Module = core::ptr::null_mut();

    DPRINT_TRACE_EXIT!();
    return if NT_SUCCESS!(unregisterStatus) {
        status
    } else {
        unregisterStatus
    };
}

#[no_mangle]
pub unsafe extern "system" fn WdfRegisterClassLibrary(
    ClassLibInfo: PWDF_CLASS_LIBRARY_INFO,
    SourceString: PUNICODE_STRING,
    ObjectName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut pClassModule: PCLASS_MODULE;
    let mut fnClassLibInit: PFN_CLASS_LIBRARY_INIT;
    let mut libModule: PLIBRARY_MODULE = core::ptr::null_mut();

    PAGED_CODE!();

    status = STATUS_SUCCESS;
    FxLdrAcquireLoadedModuleLock();

    // First, try to find an existing class module
    pClassModule = FindClassByServiceNameLocked(SourceString, core::ptr::addr_of_mut!(libModule));

    if !pClassModule.is_null() {
        /* Another behavior you can observe by running UCX */
        (*pClassModule).ClassLibraryInfo = ClassLibInfo;
    } else {
        status = FindModuleByClientService(SourceString, core::ptr::addr_of_mut!(libModule));
        if !NT_SUCCESS!(status) {
            if WdfLdrDiags.__bindgen_anon_1.DiagFlags & DIAGFLAG_ENABLED != 0 {
                DPRINT_VERBOSE!(
                    b"No library found for class module %wZ, status 0x%x\n\0".as_ptr() as PCSTR,
                    SourceString,
                    status,
                );
            }

            /*
             * This ensures class modules are always searchable through library class lists
             * The behavior here can be observed from UCX
             */
            if IsListEmpty(core::ptr::addr_of_mut!(WdfLdrGlobals.LoadedModulesList)) == 0 {
                let mut entry: PLIST_ENTRY = WdfLdrGlobals.LoadedModulesList.Flink;
                libModule = CONTAINING_RECORD!(entry, LIBRARY_MODULE, LibraryListEntry);
                DPRINT!(
                    b"Using default WDF library %wZ for class module %wZ\n\0".as_ptr() as PCSTR,
                    core::ptr::addr_of_mut!((*libModule).ServicePath),
                    SourceString,
                );
            } else {
                libModule = core::ptr::null_mut();
            }
        }

        pClassModule = ClassCreate(ClassLibInfo, libModule, SourceString);
        if !pClassModule.is_null() {
            if !libModule.is_null() {
                LibraryAddToClassListLocked(libModule, pClassModule);
            }
            ClassAddReference(pClassModule);
        }
    }
    FxLdrReleaseLoadedModuleLock();

    if pClassModule.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    __DBGPRINT!(b"Class Library (%p)\n\0".as_ptr() as PCSTR, pClassModule);

    status = ClassOpen(pClassModule, ObjectName);

    if !NT_SUCCESS!(status) {
        __DBGPRINT!(
            b"ERROR: ClassOpen(%wZ) failed, status 0x%x\n\0".as_ptr() as PCSTR,
            ObjectName,
            status,
        );
        ClassRemoveFromLibraryList(pClassModule);
        return status;
    }

    fnClassLibInit = (*(*pClassModule).ClassLibraryInfo).ClassLibraryInitialize;
    if fnClassLibInit.is_some() {
        status = (fnClassLibInit.unwrap_unchecked())();
        if !NT_SUCCESS!(status) {
            __DBGPRINT!(
                b"ERROR: WdfRegisterClassLibrary: ClassLibraryInitialize failed, status 0x%x\n\0"
                    .as_ptr() as PCSTR,
                status,
            );
            (*pClassModule).ClassLibraryInfo = core::ptr::null_mut();
            ClassClose(pClassModule);
        }
    }

    /* Classes always stay loaded after registration - (UCX, NDIS) */
    return status;
}

#[no_mangle]
pub unsafe extern "system" fn WdfVersionBindClass(
    BindInfo: PWDF_BIND_INFO,
    ClientGlobals: *mut PWDF_COMPONENT_GLOBALS,
    ClassBindInfo: PWDF_CLASS_BIND_INFO,
) -> NTSTATUS {
    let mut pClassClientModule: PCLASS_CLIENT_MODULE = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut pClassModule: PCLASS_MODULE = core::ptr::null_mut();

    DPRINT_TRACE_ENTRY!();

    pClassClientModule = ClassClientCreate();
    if pClassClientModule.is_null() {
        DPRINT_ERROR!("Could not create class client struct\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = ReferenceClassVersion(ClassBindInfo, BindInfo, core::ptr::addr_of_mut!(pClassModule));
    if !NT_SUCCESS!(status) {
        ExFreePoolWithTag((pClassClientModule).cast(), WDFLDR_TAG);
        return status;
    }

    status = ClassLinkInClient(pClassModule, ClassBindInfo, BindInfo, pClassClientModule);
    if !NT_SUCCESS!(status) {
        DPRINT_ERROR!("ClassLinkInClient failed 0x%x\n", status);
    } else {
        // Success - clear pClassClientModule so we don't free it
        pClassClientModule = core::ptr::null_mut();
        status = (*(*pClassModule).ClassLibraryInfo)
            .ClassLibraryBindClient
            .unwrap_unchecked()(ClassBindInfo, ClientGlobals);
        if NT_SUCCESS!(status) {
            DPRINT_TRACE_EXIT!();
            return status;
        }

        DPRINT_ERROR!("ClassLibraryBindClient failed, status 0x%x\n", status);
    }

    if !pClassModule.is_null() {
        DereferenceClassVersion(ClassBindInfo, BindInfo, *ClientGlobals);
    }
    if !pClassClientModule.is_null() {
        ExFreePoolWithTag((pClassClientModule).cast(), WDFLDR_TAG);
    }

    DPRINT_TRACE_EXIT!();
    return status;
}

#[no_mangle]
pub unsafe extern "system" fn WdfVersionUnbindClass(
    BindInfo: PWDF_BIND_INFO,
    Globals: PWDF_COMPONENT_GLOBALS,
    ClassBindInfo: PWDF_CLASS_BIND_INFO,
) {
    DereferenceClassVersion(ClassBindInfo, BindInfo, Globals);
}
