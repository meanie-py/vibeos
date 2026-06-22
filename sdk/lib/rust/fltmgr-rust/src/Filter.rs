use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ERESOURCE, ExInitializeResourceLite, KeEnterCriticalRegion, KeLeaveCriticalRegion, LIST_ENTRY, _OBJECT_ATTRIBUTES, PCUNICODE_STRING, PDRIVER_OBJECT, PDRIVER_UNLOAD, PUNICODE_STRING, PWCH, RtlAppendUnicodeStringToString, RtlCopyUnicodeString, ZwClose, ZwLoadDriver, ZwOpenKey, ZwUnloadDriver, _UNICODE_STRING, _LIST_ENTRY, PCSTR, ACCESS_MASK};
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;

// ntoskrnl NTAPI exports — import from ntkrnl_sys::ddk (bindgen: correct per-arch
// ABI — stdcall/`_Name@N` on i386, cdecl on amd64/arm64).
use ntkrnl_sys::ddk::{
    RtlInitUnicodeString, ExAcquireResourceSharedLite, ExReleaseResourceLite, ExDeleteResourceLite,
};
// spec-exported in-crate Flt* defs (extern "system"); import the real defs.
use crate::Object::{FltObjectReference, FltObjectDereference};

// crate-internal Fltp* helpers: defined `extern "C"` in their modules; the decl
// and def agree on cdecl, so they resolve in-crate on all arches. DbgPrint is the
// variadic cdecl ntoskrnl export (left extern "C").
extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> u32;
    fn FltpOpenFilterServicesKey( Filter: PFLT_FILTER, DesiredAccess: ACCESS_MASK, KeyName: PUNICODE_STRING, RootHandle: *mut HANDLE, ) -> NTSTATUS;
    fn FltpReadRegistryValue( KeyHandle: HANDLE, ValueName: PUNICODE_STRING, Type: ULONG, Data: PVOID, DataSize: ULONG, BytesRequired: *mut ULONG, ) -> NTSTATUS;
    fn FltpObjectRundownWait(RundownRef: *mut core::ffi::c_void);
    fn FltpExRundownCompleted(RundownRef: *mut core::ffi::c_void);
    fn FltpExInitializeRundownProtection(RunRef: PVOID);
    fn FltpRegisterContexts( Filter: PFLT_FILTER, Contexts: *const FLT_CONTEXT_REGISTRATION, ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const SERVICES_KEY: [::core::ffi::c_ushort; 53] = unsafe {
    ::core::mem::transmute::<
        [u8; 106],
        [::core::ffi::c_ushort; 53],
    >(
        *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0\0\0",
    )
};

pub static mut FilterList: LIST_ENTRY = LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

// C: `ERESOURCE FilterListLock;` — file-scope global, zero-initialized in BSS.
// (c2rust spelled out the ERESOURCE zero-init with its own anonymous-union type
// names which don't match the bindgen layout; zeroed() is identical semantics.)
pub static mut FilterListLock: ERESOURCE = unsafe { core::mem::zeroed() };

pub const MAX_KEY_LENGTH: u32 = 0x200;

#[no_mangle]
pub unsafe extern "system" fn FltLoadFilter(mut FilterName: PCUNICODE_STRING) -> NTSTATUS {
    let mut DriverServiceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut ServicesKey: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut Buffer: [CHAR; 512] = [0; 512];

    /* Setup the base services key */
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(ServicesKey),
        SERVICES_KEY.as_ptr() as PCWSTR,
    );

    /* Initialize the string data */
    DriverServiceName.Length = 0 as USHORT;
    DriverServiceName.Buffer = core::ptr::addr_of_mut!(Buffer) as *mut CHAR as PWCH;
    DriverServiceName.MaximumLength = MAX_KEY_LENGTH as USHORT;

    /* Create the full service key for this filter */
    RtlCopyUnicodeString(
        core::ptr::addr_of_mut!(DriverServiceName),
        core::ptr::addr_of_mut!(ServicesKey) as PCUNICODE_STRING,
    );
    RtlAppendUnicodeStringToString(core::ptr::addr_of_mut!(DriverServiceName), FilterName);

    /* Ask the kernel to load it for us */
    return ZwLoadDriver(core::ptr::addr_of_mut!(DriverServiceName)) as NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn FltUnloadFilter(mut FilterName: PCUNICODE_STRING) -> NTSTATUS {
    const MAX_KEY_LENGTH: u32 = 0x200;
    let ServicesKeyData: &[u16] = &[
        0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C, 0x004D,
        0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079, 0x0073, 0x0074,
        0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0043,
        0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053,
        0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073, 0x005C, 0x0000,
    ];

    let mut DriverServiceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ServicesKey: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Buffer: [CHAR; 512] = [0; 512];

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(ServicesKey),
        ServicesKeyData.as_ptr() as PCWSTR,
    );

    DriverServiceName.Length = 0 as USHORT;
    DriverServiceName.Buffer = core::ptr::addr_of_mut!(Buffer) as *mut CHAR as PWCH;
    DriverServiceName.MaximumLength = MAX_KEY_LENGTH as USHORT;

    RtlCopyUnicodeString(
        core::ptr::addr_of_mut!(DriverServiceName),
        core::ptr::addr_of_mut!(ServicesKey) as PCUNICODE_STRING,
    );
    RtlAppendUnicodeStringToString(core::ptr::addr_of_mut!(DriverServiceName), FilterName);
    return ZwUnloadDriver(core::ptr::addr_of_mut!(DriverServiceName)) as NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn FltRegisterFilter(
    DriverObject: PDRIVER_OBJECT,
    Registration: *const FLT_REGISTRATION,
    RetFilter: *mut PFLT_FILTER,
) -> NTSTATUS {

    let mut Callbacks: PFLT_OPERATION_REGISTRATION;
    let Filter: PFLT_FILTER;
    let mut Frame: PFLTP_FRAME = core::ptr::null_mut();
    let CallbackBufferSize: ULONG;
    let FilterBufferSize: ULONG;
    let mut Count: ULONG = 0;
    let mut Ptr: PCHAR;
    let mut Status: NTSTATUS;

    *RetFilter = core::ptr::null_mut();

    /* Make sure we're targeting the correct major revision */
    if ((*Registration).Version as i32 & 0xFF00) != FLT_MAJOR_VERSION {
        return STATUS_INVALID_PARAMETER;
    }

    /* Make sure our namespace callbacks are valid */
    if ((*Registration).GenerateFileNameCallback.is_none()
        && (*Registration).NormalizeNameComponentCallback.is_some())
        || ((*Registration).NormalizeNameComponentCallback.is_none()
            && (*Registration).NormalizeContextCleanupCallback.is_some())
    {
        return STATUS_INVALID_PARAMETER;
    }

    /* Count the number of operations that were requested */
    Callbacks = (*Registration).OperationRegistration as PFLT_OPERATION_REGISTRATION;
    while !Callbacks.is_null() {
        Count = Count.wrapping_add(1);

        /* Bail when we find the last one */
        if (*Callbacks).MajorFunction as i32 == IRP_MJ_OPERATION_END as i32 {
            break;
        }

        /* Move to the next item */
        Callbacks = Callbacks.offset(1);
    }

    /* Calculate the buffer sizes */
    CallbackBufferSize = (Count as usize)
        .wrapping_mul(core::mem::size_of::<FLT_OPERATION_REGISTRATION>()) as ULONG;
    FilterBufferSize = (core::mem::size_of::<FLT_FILTER>())
        .wrapping_add(CallbackBufferSize as usize)
        .wrapping_add((*(*DriverObject).DriverExtension).ServiceKeyName.Length as usize)
        as ULONG;

    /* Allocate a buffer to hold our filter data */
    Filter = ExAllocatePoolWithTag(
        NonPagedPool,
        FilterBufferSize as SIZE_T,
        FM_TAG_FILTER as ULONG,
    ) as PFLT_FILTER;
    if Filter.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    RtlZeroMemory(Filter as PVOID, FilterBufferSize as SIZE_T);

    /* Find the end of the fixed struct */
    Ptr = Filter.offset(1) as PCHAR;

    /* Store a copy of the driver object of this filter */
    (*Filter).DriverObject = DriverObject;

    /* Initialize the base object data */
    (*Filter).Base.Flags = FLT_OBFL_TYPE_FILTER;
    (*Filter).Base.PointerCount = 1;
    FltpExInitializeRundownProtection(core::ptr::addr_of_mut!((*Filter).Base.RundownRef) as PVOID);
    FltObjectReference(core::ptr::addr_of_mut!((*Filter).Base) as PVOID);

    /* Set the callback addresses */
    (*Filter).FilterUnload = (*Registration).FilterUnloadCallback;
    (*Filter).InstanceSetup = (*Registration).InstanceSetupCallback;
    (*Filter).InstanceQueryTeardown = (*Registration).InstanceQueryTeardownCallback;
    (*Filter).InstanceTeardownStart = (*Registration).InstanceTeardownStartCallback;
    (*Filter).InstanceTeardownComplete = (*Registration).InstanceTeardownCompleteCallback;
    (*Filter).GenerateFileName = (*Registration).GenerateFileNameCallback;
    (*Filter).NormalizeNameComponent = (*Registration).NormalizeNameComponentCallback;
    (*Filter).NormalizeContextCleanup = (*Registration).NormalizeContextCleanupCallback;

    /* Initialize the instance list */
    ExInitializeResourceLite(core::ptr::addr_of_mut!((*Filter).InstanceList.rLock));
    InitializeListHead(core::ptr::addr_of_mut!((*Filter).InstanceList.rList));
    (*Filter).InstanceList.rCount = 0;

    ExInitializeFastMutex(core::ptr::addr_of_mut!((*Filter).ActiveOpens.mLock));
    InitializeListHead(core::ptr::addr_of_mut!((*Filter).ActiveOpens.mList));
    (*Filter).ActiveOpens.mCount = 0;

    ExInitializeFastMutex(core::ptr::addr_of_mut!((*Filter).ConnectionList.mLock));
    InitializeListHead(core::ptr::addr_of_mut!((*Filter).ConnectionList.mList));
    (*Filter).ConnectionList.mCount = 0;

    /* Initialize the usermode port list */
    ExInitializeFastMutex(core::ptr::addr_of_mut!((*Filter).PortList.mLock));
    InitializeListHead(core::ptr::addr_of_mut!((*Filter).PortList.mList));
    (*Filter).PortList.mCount = 0;

    /* We got this far, assume success from here */
    Status = STATUS_SUCCESS;

    let mut goto_quit = false;

    /* Check if the caller requested any context data */
    if !(*Registration).ContextRegistration.is_null() {
        /* Register the contexts for this filter */
        Status = FltpRegisterContexts(Filter, (*Registration).ContextRegistration);
        if NT_SUCCESS(Status) {
            goto_quit = true;
        }
    }

    if !goto_quit {
        /* Check if the caller is registering any callbacks */
        if !(*Registration).OperationRegistration.is_null() {
            /* The callback data comes after the fixed struct */
            (*Filter).Operations = Ptr as PFLT_OPERATION_REGISTRATION;
            Ptr = Ptr.offset(
                ((Count as usize)
                    .wrapping_mul(core::mem::size_of::<FLT_OPERATION_REGISTRATION>()))
                    as isize,
            );

            /* Tag the operation data onto the end of the filter data */
            RtlCopyMemory(
                (*Filter).Operations as PVOID,
                (*Registration).OperationRegistration as PVOID,
                CallbackBufferSize as SIZE_T,
            );

            /* walk through the requested callbacks */
            Callbacks = (*Filter).Operations;
            while (*Callbacks).MajorFunction as i32 != IRP_MJ_OPERATION_END as i32 {
                /* Check if this is an attach to a volume */
                if (*Callbacks).MajorFunction as i32 == IRP_MJ_VOLUME_MOUNT as i32 {
                    (*Filter).PreVolumeMount =
                        core::mem::transmute((*Callbacks).PreOperation);
                    (*Filter).PostVolumeMount =
                        core::mem::transmute((*Callbacks).PostOperation);
                } else if (*Callbacks).MajorFunction as i32 == IRP_MJ_SHUTDOWN as i32 {
                    (*Callbacks).PostOperation = None;
                }
                Callbacks = Callbacks.offset(1);
            }
        }

        /* Add the filter name buffer onto the end of the data and fill in the string */
        (*Filter).Name.Length = 0;
        (*Filter).Name.MaximumLength = (*(*DriverObject).DriverExtension).ServiceKeyName.Length;
        (*Filter).Name.Buffer = Ptr as PWCH;
        RtlCopyUnicodeString(
            core::ptr::addr_of_mut!((*Filter).Name),
            core::ptr::addr_of_mut!((*(*DriverObject).DriverExtension).ServiceKeyName)
                as PCUNICODE_STRING,
        );

        /* Lookup the altitude of the mini-filter */
        Status = GetFilterAltitude(Filter, core::ptr::addr_of_mut!((*Filter).DefaultAltitude));
        if !NT_SUCCESS(Status) {
            goto_quit = true;
        }

        if !goto_quit {
            /* Lookup the filter frame */
            Status = GetFilterFrame(
                Filter,
                core::ptr::addr_of_mut!((*Filter).DefaultAltitude),
                core::ptr::addr_of_mut!(Frame),
            );
            if Status == STATUS_NOT_FOUND {
                /* Store the frame this mini-filter's main struct */
                (*Filter).Frame = Frame;

                Status = FltpAttachFrame(
                    core::ptr::addr_of_mut!((*Filter).DefaultAltitude),
                    core::ptr::addr_of_mut!(Frame),
                );
            }

            if !NT_SUCCESS(Status) {
                goto_quit = true;
            }

            if !goto_quit {
                /* Store any existing driver unload routine before we make any changes */
                (*Filter).OldDriverUnload =
                    core::mem::transmute((*DriverObject).DriverUnload);

                /* Check we opted not to have an unload routine, or if we want to stop the driver from being unloaded */
                if (*Registration).FilterUnloadCallback.is_some()
                    && FlagOn((*Filter).Flags, FLTFL_REGISTRATION_DO_NOT_SUPPORT_SERVICE_STOP) == 0
                {
                    (*DriverObject).DriverUnload =
                        core::mem::transmute(Some(FltpMiniFilterDriverUnload as unsafe extern "C" fn()));
                } else {
                    (*DriverObject).DriverUnload =
                        core::mem::transmute(core::ptr::null_mut::<core::ffi::c_void>());
                }
            }
        }
    }

    /* Quit: */
    if NT_SUCCESS(Status) {
        DbgPrint(
            b"Loaded FS mini-filter %wZ\n\0".as_ptr() as *const i8,
            core::ptr::addr_of_mut!((*(*DriverObject).DriverExtension).ServiceKeyName),
        );
        *RetFilter = Filter;
    } else {
        DbgPrint(
            b"Failed to load FS mini-filter %wZ : 0x%X\n\0".as_ptr() as *const i8,
            core::ptr::addr_of_mut!((*(*DriverObject).DriverExtension).ServiceKeyName),
            Status,
        );

        ExDeleteResourceLite(core::ptr::addr_of_mut!((*Filter).InstanceList.rLock));
        ExFreePoolWithTag(Filter as PVOID, FM_TAG_FILTER as ULONG);
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn FltUnregisterFilter(Filter: PFLT_FILTER) {

    let mut Instance: PFLT_INSTANCE;
    let mut CurrentEntry: PLIST_ENTRY;
    let mut Status: NTSTATUS;

    /* Set the draining flag */
    Status = FltpStartingToDrainObject(core::ptr::addr_of_mut!((*Filter).Base) as PVOID);
    if !NT_SUCCESS(Status) {
        /* Someone already unregistered us, just remove our ref and bail */
        FltObjectDereference(core::ptr::addr_of_mut!((*Filter).Base) as PVOID);
        return;
    }

    /* Lock the instance list */
    KeEnterCriticalRegion();
    ExAcquireResourceSharedLite(core::ptr::addr_of_mut!((*Filter).InstanceList.rLock), 1 as BOOLEAN);

    /* Set the first entry in the list */
    CurrentEntry = (*Filter).InstanceList.rList.Flink as PLIST_ENTRY;

    /* Free all instances referenced by the filter */
    while CurrentEntry != core::ptr::addr_of_mut!((*Filter).InstanceList.rList) {
        /* Get the record pointer */
        Instance = (CurrentEntry as usize).wrapping_sub(
            core::mem::offset_of!(FLT_INSTANCE, FilterLink),
        ) as *mut FLT_INSTANCE as PFLT_INSTANCE;

        // FIXME: implement
        let _ = Instance;

        /* Reset the pointer and move to next entry */
        Instance = core::ptr::null_mut();
        let _ = Instance;
        CurrentEntry = (*CurrentEntry).Flink as PLIST_ENTRY;
    }

    /* We're done with instances now */
    ExReleaseResourceLite(core::ptr::addr_of_mut!((*Filter).InstanceList.rLock));
    KeLeaveCriticalRegion();

    /* Remove the reference from the base object */
    FltObjectDereference(core::ptr::addr_of_mut!((*Filter).Base) as PVOID);

    /* Wait until we're sure nothing is using the filter */
    FltpObjectRundownWait(core::ptr::addr_of_mut!((*Filter).Base.RundownRef) as *mut core::ffi::c_void);

    /* Delete the instance list lock */
    ExDeleteResourceLite(core::ptr::addr_of_mut!((*Filter).InstanceList.rLock));

    /* We're finished cleaning up now */
    FltpExRundownCompleted(core::ptr::addr_of_mut!((*Filter).Base.RundownRef) as *mut core::ffi::c_void);

    /* Hand the memory back */
    ExFreePoolWithTag(Filter as PVOID, FM_TAG_FILTER);
}

#[no_mangle]
pub unsafe extern "system" fn FltStartFiltering(mut Filter: PFLT_FILTER) -> NTSTATUS {
    let mut Status: NTSTATUS;

    /* Grab a ref to the filter */
    Status = FltObjectReference(core::ptr::addr_of_mut!((*Filter).Base) as PVOID);
    if NT_SUCCESS(Status) {
        /* Make sure we aren't already starting up */
        if (*Filter).Flags as u32 & FLTFL_FILTERING_INITIATED as u32 == 0 {
            // Startup
        } else {
            Status = STATUS_INVALID_PARAMETER;
        }

        FltObjectDereference(core::ptr::addr_of_mut!((*Filter).Base) as PVOID);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn FltGetFilterFromName(
    FilterName: PCUNICODE_STRING,
    RetFilter: *mut PFLT_FILTER,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"FltGetFilterFromName\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/filters/fltmgr/Filter.c\0".as_ptr() as *const ::core::ffi::c_char,
        406 as ::core::ffi::c_int,
    );
    let _ = FilterName;
    *RetFilter = core::ptr::null_mut();
    return STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "C" fn FltpMiniFilterDriverUnload() {
    __debugbreak();
}
#[no_mangle]
pub unsafe extern "C" fn FltpAttachFrame(
    Altitude: PUNICODE_STRING,
    Frame: *mut PFLTP_FRAME,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"FltpAttachFrame\0".as_ptr() as *const i8,
        b"drivers/filters/fltmgr/Filter.c\0"
            .as_ptr() as *const i8,
        443 as i32,
    );
    let _ = Altitude;
    *Frame = core::ptr::null_mut();
    return STATUS_SUCCESS;
}

// drivers/filters/fltmgr/Filter.c — INTERNAL FUNCTIONS
// NTSTATUS FltpStartingToDrainObject(_Inout_ PFLT_OBJECT Object)
// (caller passes &Filter->Base cast to PVOID; param is PVOID to match the call site.)
#[no_mangle]
pub unsafe extern "C" fn FltpStartingToDrainObject(Object: PVOID) -> NTSTATUS {
    const FLT_OBFL_DRAINING: core::ffi::c_long = 1;
    const STATUS_FLT_DELETING_OBJECT: NTSTATUS = 0xC01C0009u32 as NTSTATUS;
    let Object = Object as PFLT_OBJECT;
    /*
     * Set the draining flag for the filter. This let's us force
     * a post op callback for minifilters currently awaiting one.
     */
    if InterlockedOr(
        core::ptr::addr_of_mut!((*Object).Flags) as *mut core::ffi::c_long,
        FLT_OBFL_DRAINING,
    ) & 1
        != 0
    {
        /* We've been called once, we're already being deleted */
        return STATUS_FLT_DELETING_OBJECT;
    }

    return STATUS_SUCCESS;
}

const FM_TAG_UNICODE_STRING: ULONG = 0x6D754D46; // 'FMum'
unsafe fn GetFilterAltitude(
    Filter: PFLT_FILTER,
    AltitudeString: PUNICODE_STRING,
) -> NTSTATUS {
    let mut InstancesKey: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 10]>())
            .wrapping_sub(core::mem::size_of::<u16>()) as u16,
        MaximumLength: (core::mem::size_of::<[u16; 10]>())
            .wrapping_div(core::mem::size_of::<i8>()) as u16,
        Buffer: core::ptr::addr_of!(
            *core::mem::transmute::<&[u8; 20], &[u16; 10]>(b"I\0n\0s\0t\0a\0n\0c\0e\0s\0\0\0")
        ) as *const u16 as PWCH,
    };
    let mut DefaultInstance: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 16]>())
            .wrapping_sub(core::mem::size_of::<u16>()) as u16,
        MaximumLength: (core::mem::size_of::<[u16; 16]>())
            .wrapping_div(core::mem::size_of::<i8>()) as u16,
        Buffer: core::ptr::addr_of!(
            *core::mem::transmute::<&[u8; 32], &[u16; 16]>(
                b"D\0e\0f\0a\0u\0l\0t\0I\0n\0s\0t\0a\0n\0c\0e\0\0\0"
            )
        ) as *const u16 as PWCH,
    };
    let mut Altitude: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[u16; 9]>())
            .wrapping_sub(core::mem::size_of::<u16>()) as u16,
        MaximumLength: (core::mem::size_of::<[u16; 9]>())
            .wrapping_div(core::mem::size_of::<i8>()) as u16,
        Buffer: core::ptr::addr_of!(
            *core::mem::transmute::<&[u8; 18], &[u16; 9]>(b"A\0l\0t\0i\0t\0u\0d\0e\0\0\0")
        ) as *const u16 as PWCH,
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut FilterInstancePath: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut BytesRequired: ULONG = 0;
    let mut InstHandle: HANDLE = core::ptr::null_mut();
    let mut RootHandle: HANDLE = core::ptr::null_mut();
    let mut InstBuffer: PWCH = core::ptr::null_mut();
    let mut AltBuffer: PWCH = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    /* Get a handle to the instances key in the filter's services key */
    Status = FltpOpenFilterServicesKey(
        Filter,
        KEY_QUERY_VALUE as ACCESS_MASK,
        core::ptr::addr_of_mut!(InstancesKey),
        core::ptr::addr_of_mut!(RootHandle),
    );
    if !(Status >= 0) {
        return Status;
    }

    /* Read the size 'default instances' string value */
    Status = FltpReadRegistryValue(
        RootHandle,
        core::ptr::addr_of_mut!(DefaultInstance),
        REG_SZ as ULONG,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(BytesRequired),
    );

    let mut to_quit = false;

    /* We should get a buffer too small error */
    if Status == STATUS_BUFFER_TOO_SMALL {
        /* Allocate the buffer we need to hold the string */
        InstBuffer = ExAllocatePoolWithTag(
            PagedPool,
            BytesRequired as SIZE_T,
            FM_TAG_UNICODE_STRING,
        ) as PWCH;
        if InstBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            to_quit = true;
        } else {
            /* Now read the string value */
            Status = FltpReadRegistryValue(
                RootHandle,
                core::ptr::addr_of_mut!(DefaultInstance),
                REG_SZ as ULONG,
                InstBuffer as PVOID,
                BytesRequired,
                core::ptr::addr_of_mut!(BytesRequired),
            );
        }
    }

    if !to_quit {
        if !(Status >= 0) {
            to_quit = true;
        }
    }

    if !to_quit {
        /* Convert the string to a unicode_string */
        RtlInitUnicodeString(core::ptr::addr_of_mut!(FilterInstancePath), InstBuffer as PCWSTR);

        /* Setup the attributes using the root key handle */
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = RootHandle;
        ObjectAttributes.ObjectName =
            core::ptr::addr_of_mut!(FilterInstancePath) as PUNICODE_STRING;
        ObjectAttributes.Attributes = (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE) as ULONG;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        /* Now open the key name which was stored in the default instance */
        Status = ZwOpenKey(
            core::ptr::addr_of_mut!(InstHandle),
            KEY_QUERY_VALUE as ACCESS_MASK,
            core::ptr::addr_of_mut!(ObjectAttributes),
        ) as NTSTATUS;
        if Status >= 0 {
            /* Get the size of the buffer that holds the altitude */
            Status = FltpReadRegistryValue(
                InstHandle,
                core::ptr::addr_of_mut!(Altitude),
                REG_SZ as ULONG,
                core::ptr::null_mut(),
                0,
                core::ptr::addr_of_mut!(BytesRequired),
            );
            if Status == STATUS_BUFFER_TOO_SMALL {
                /* Allocate the required buffer */
                AltBuffer = ExAllocatePoolWithTag(
                    PagedPool,
                    BytesRequired as SIZE_T,
                    FM_TAG_UNICODE_STRING,
                ) as PWCH;
                if AltBuffer.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    to_quit = true;
                } else {
                    /* And now finally read in the actual altitude string */
                    Status = FltpReadRegistryValue(
                        InstHandle,
                        core::ptr::addr_of_mut!(Altitude),
                        REG_SZ as ULONG,
                        AltBuffer as PVOID,
                        BytesRequired,
                        core::ptr::addr_of_mut!(BytesRequired),
                    );
                    if Status >= 0 {
                        /* We made it, setup the return buffer */
                        (*AltitudeString).Length = BytesRequired as u16;
                        (*AltitudeString).MaximumLength = BytesRequired as u16;
                        (*AltitudeString).Buffer = AltBuffer;
                    }
                }
            }
        }
    }

    // Quit:
    if !(Status >= 0) {
        if !AltBuffer.is_null() {
            ExFreePoolWithTag(AltBuffer as PVOID, FM_TAG_UNICODE_STRING);
        }
    }

    if !InstBuffer.is_null() {
        ExFreePoolWithTag(InstBuffer as PVOID, FM_TAG_UNICODE_STRING);
    }

    if !InstHandle.is_null() {
        ZwClose(InstHandle);
    }
    ZwClose(RootHandle);

    return Status;
}
unsafe fn GetFilterFrame(
    mut Filter: PFLT_FILTER,
    mut Altitude: PUNICODE_STRING,
    mut Frame: *mut PFLTP_FRAME,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"GetFilterFrame\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/filters/fltmgr/Filter.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        588 as ::core::ffi::c_int,
    );
    *Frame = core::ptr::null_mut();
    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         Filesystem Filter Manager
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            drivers/filters/fltmgr/Filter.c
// * PURPOSE:         Handles registration of mini filters
// * PROGRAMMERS:     Ged Murphy (gedmurphy@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include "fltmgr.h"
// #include "fltmgrint.h"
// #include "Registry.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* DATA *********************************************************************/
//
// #define SERVICES_KEY L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\"
// #define MAX_KEY_LENGTH  0x200
//
// LIST_ENTRY FilterList;
// ERESOURCE FilterListLock;
//
// NTSTATUS
// FltpStartingToDrainObject(
//      PFLT_OBJECT Object
// );
//
// VOID
// FltpMiniFilterDriverUnload(
// );
//
// NTSTATUS
// FltpAttachFrame(
//      PUNICODE_STRING Altitude,
//      PFLTP_FRAME *Frame
// );
//
// static
// NTSTATUS
// GetFilterAltitude(
//      PFLT_FILTER Filter,
//      PUNICODE_STRING AltitudeString
// );
//
// static
// NTSTATUS
// GetFilterFrame(
//      PFLT_FILTER Filter,
//      PUNICODE_STRING Altitude,
//      PFLTP_FRAME *Frame
// );
//
//
// /* EXPORTED FUNCTIONS ******************************************************/
//
// NTSTATUS
// NTAPI
// FltLoadFilter( PCUNICODE_STRING FilterName)
// {
//     UNICODE_STRING DriverServiceName;
//     UNICODE_STRING ServicesKey;
//     CHAR Buffer[MAX_KEY_LENGTH];
//
//     /* Setup the base services key */
//     RtlInitUnicodeString(&ServicesKey, SERVICES_KEY);
//
//     /* Initialize the string data */
//     DriverServiceName.Length = 0;
//     DriverServiceName.Buffer = (PWCH)Buffer;
//     DriverServiceName.MaximumLength = MAX_KEY_LENGTH;
//
//     /* Create the full service key for this filter */
//     RtlCopyUnicodeString(&DriverServiceName, &ServicesKey);
//     RtlAppendUnicodeStringToString(&DriverServiceName, FilterName);
//
//     /* Ask the kernel to load it for us */
//     return ZwLoadDriver(&DriverServiceName);
// }
//
// NTSTATUS
// NTAPI
// FltUnloadFilter( PCUNICODE_STRING FilterName)
// {
//     //
//     //FIXME: This is a temp hack, it needs properly implementing
//     //
//
//     UNICODE_STRING DriverServiceName;
//     UNICODE_STRING ServicesKey;
//     CHAR Buffer[MAX_KEY_LENGTH];
//
//     /* Setup the base services key */
//     RtlInitUnicodeString(&ServicesKey, SERVICES_KEY);
//
//     /* Initialize the string data */
//     DriverServiceName.Length = 0;
//     DriverServiceName.Buffer = (PWCH)Buffer;
//     DriverServiceName.MaximumLength = MAX_KEY_LENGTH;
//
//     /* Create the full service key for this filter */
//     RtlCopyUnicodeString(&DriverServiceName, &ServicesKey);
//     RtlAppendUnicodeStringToString(&DriverServiceName, FilterName);
//     return ZwUnloadDriver(&DriverServiceName);
// }
//
// NTSTATUS
// NTAPI
// FltRegisterFilter( PDRIVER_OBJECT DriverObject,
//                    const FLT_REGISTRATION *Registration,
//                    PFLT_FILTER *RetFilter)
// {
//     PFLT_OPERATION_REGISTRATION Callbacks;
//     PFLT_FILTER Filter;
//     PFLTP_FRAME Frame;
//     ULONG CallbackBufferSize;
//     ULONG FilterBufferSize;
//     ULONG Count = 0;
//     PCHAR Ptr;
//     NTSTATUS Status;
//
//     *RetFilter = NULL;
//
//     /* Make sure we're targeting the correct major revision */
//     if ((Registration->Version & 0xFF00) != FLT_MAJOR_VERSION)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Make sure our namespace callbacks are valid */
//     if ((!Registration->GenerateFileNameCallback && Registration->NormalizeNameComponentCallback) ||
//         (!Registration->NormalizeNameComponentCallback && Registration->NormalizeContextCleanupCallback))
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Count the number of operations that were requested */
//     Callbacks = (PFLT_OPERATION_REGISTRATION)Registration->OperationRegistration;
//     while (Callbacks)
//     {
//         Count++;
//
//         /* Bail when we find the last one */
//         if (Callbacks->MajorFunction == IRP_MJ_OPERATION_END)
//             break;
//
//         /* Move to the next item */
//         Callbacks++;
//     }
//
//     /* Calculate the buffer sizes */
//     CallbackBufferSize = Count * sizeof(FLT_OPERATION_REGISTRATION);
//     FilterBufferSize = sizeof(FLT_FILTER) +
//                        CallbackBufferSize +
//                        DriverObject->DriverExtension->ServiceKeyName.Length;
//
//     /* Allocate a buffer to hold our filter data */
//     Filter = ExAllocatePoolWithTag(NonPagedPool,
//                                    FilterBufferSize,
//                                    FM_TAG_FILTER);
//     if (Filter == NULL) return STATUS_INSUFFICIENT_RESOURCES;
//     RtlZeroMemory(Filter, FilterBufferSize);
//
//     /* Find the end of the fixed struct */
//     Ptr = (PCHAR)(Filter + 1);
//
//     /* Store a copy of the driver object of this filter */
//     Filter->DriverObject = DriverObject;
//
//     /* Initialize the base object data */
//     Filter->Base.Flags = FLT_OBFL_TYPE_FILTER;
//     Filter->Base.PointerCount = 1;
//     FltpExInitializeRundownProtection(&Filter->Base.RundownRef);
//     FltObjectReference(&Filter->Base);
//
//     /* Set the callback addresses */
//     Filter->FilterUnload = Registration->FilterUnloadCallback;
//     Filter->InstanceSetup = Registration->InstanceSetupCallback;
//     Filter->InstanceQueryTeardown = Registration->InstanceQueryTeardownCallback;
//     Filter->InstanceTeardownStart = Registration->InstanceTeardownStartCallback;
//     Filter->InstanceTeardownComplete = Registration->InstanceTeardownCompleteCallback;
//     Filter->GenerateFileName = Registration->GenerateFileNameCallback;
//     Filter->NormalizeNameComponent = Registration->NormalizeNameComponentCallback;
//     Filter->NormalizeContextCleanup = Registration->NormalizeContextCleanupCallback;
//
//     /* Initialize the instance list */
//     ExInitializeResourceLite(&Filter->InstanceList.rLock);
//     InitializeListHead(&Filter->InstanceList.rList);
//     Filter->InstanceList.rCount = 0;
//
//     ExInitializeFastMutex(&Filter->ActiveOpens.mLock);
//     InitializeListHead(&Filter->ActiveOpens.mList);
//     Filter->ActiveOpens.mCount = 0;
//
//     ExInitializeFastMutex(&Filter->ConnectionList.mLock);
//     InitializeListHead(&Filter->ConnectionList.mList);
//     Filter->ConnectionList.mCount = 0;
//
//     /* Initialize the usermode port list */
//     ExInitializeFastMutex(&Filter->PortList.mLock);
//     InitializeListHead(&Filter->PortList.mList);
//     Filter->PortList.mCount = 0;
//
//     /* We got this far, assume success from here */
//     Status = STATUS_SUCCESS;
//
//     /* Check if the caller requested any context data */
//     if (Registration->ContextRegistration)
//     {
//         /* Register the contexts for this filter */
//         Status = FltpRegisterContexts(Filter, Registration->ContextRegistration);
//         if (NT_SUCCESS(Status))
//         {
//             goto Quit;
//         }
//     }
//
//     /* Check if the caller is registering any callbacks */
//     if (Registration->OperationRegistration)
//     {
//         /* The callback data comes after the fixed struct */
//         Filter->Operations = (PFLT_OPERATION_REGISTRATION)Ptr;
//         Ptr += (Count * sizeof(FLT_OPERATION_REGISTRATION));
//
//         /* Tag the operation data onto the end of the filter data */
//         RtlCopyMemory(Filter->Operations, Registration->OperationRegistration, CallbackBufferSize);
//
//         /* walk through the requested callbacks */
//         for (Callbacks = Filter->Operations;
//              Callbacks->MajorFunction != IRP_MJ_OPERATION_END;
//              Callbacks++)
//         {
//             // http://fsfilters.blogspot.co.uk/2011/03/how-file-system-filters-attach-to_17.html
//             /* Check if this is an attach to a volume */
//             if (Callbacks->MajorFunction == IRP_MJ_VOLUME_MOUNT)
//             {
//                 Filter->PreVolumeMount = Callbacks->PreOperation;
//                 Filter->PostVolumeMount = Callbacks->PostOperation;
//             }
//             else if (Callbacks->MajorFunction == IRP_MJ_SHUTDOWN)
//             {
//                 Callbacks->PostOperation = NULL;
//             }
//         }
//     }
//
//     /* Add the filter name buffer onto the end of the data and fill in the string */
//     Filter->Name.Length = 0;
//     Filter->Name.MaximumLength = DriverObject->DriverExtension->ServiceKeyName.Length;
//     Filter->Name.Buffer = (PWCH)Ptr;
//     RtlCopyUnicodeString(&Filter->Name, &DriverObject->DriverExtension->ServiceKeyName);
//
//     /* Lookup the altitude of the mini-filter */
//     Status = GetFilterAltitude(Filter, &Filter->DefaultAltitude);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Quit;
//     }
//
//     /* Lookup the filter frame */
//     Status = GetFilterFrame(Filter, &Filter->DefaultAltitude, &Frame);
//     if (Status == STATUS_NOT_FOUND)
//     {
//         /* Store the frame this mini-filter's main struct */
//         Filter->Frame = Frame;
//
//         Status = FltpAttachFrame(&Filter->DefaultAltitude, &Frame);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         goto Quit;
//     }
//
//     //
//     // - Slot the filter into the correct altitude location
//     // - More stuff??
//     //
//
//     /* Store any existing driver unload routine before we make any changes */
//     Filter->OldDriverUnload = (PFLT_FILTER_UNLOAD_CALLBACK)DriverObject->DriverUnload;
//
//     /* Check we opted not to have an unload routine, or if we want to stop the driver from being unloaded */
//     if (Registration->FilterUnloadCallback && !FlagOn(Filter->Flags, FLTFL_REGISTRATION_DO_NOT_SUPPORT_SERVICE_STOP))
//     {
//         DriverObject->DriverUnload = (PDRIVER_UNLOAD)FltpMiniFilterDriverUnload;
//     }
//     else
//     {
//         DriverObject->DriverUnload = (PDRIVER_UNLOAD)NULL;
//     }
//
//
// Quit:
//
//     if (NT_SUCCESS(Status))
//     {
//         DPRINT1("Loaded FS mini-filter %wZ\n", &DriverObject->DriverExtension->ServiceKeyName);
//         *RetFilter = Filter;
//     }
//     else
//     {
//         DPRINT1("Failed to load FS mini-filter %wZ : 0x%X\n", &DriverObject->DriverExtension->ServiceKeyName, Status);
//
//         // Add cleanup for context resources
//
//         ExDeleteResourceLite(&Filter->InstanceList.rLock);
//         ExFreePoolWithTag(Filter, FM_TAG_FILTER);
//     }
//
//     return Status;
// }
//
// VOID
// FLTAPI
// FltUnregisterFilter( PFLT_FILTER Filter)
// {
//     PFLT_INSTANCE Instance;
//     PLIST_ENTRY CurrentEntry;
//     NTSTATUS Status;
//
//     /* Set the draining flag */
//     Status = FltpStartingToDrainObject(&Filter->Base);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Someone already unregistered us, just remove our ref and bail */
//         FltObjectDereference(&Filter->Base);
//         return;
//     }
//
//     /* Lock the instance list */
//     KeEnterCriticalRegion();
//     ExAcquireResourceSharedLite(&Filter->InstanceList.rLock, TRUE);
//
//     /* Set the first entry in the list */
//     CurrentEntry = Filter->InstanceList.rList.Flink;
//
//     /* Free all instances referenced by the filter */
//     while (CurrentEntry != &Filter->InstanceList.rList)
//     {
//         /* Get the record pointer */
//         Instance = CONTAINING_RECORD(CurrentEntry, FLT_INSTANCE, FilterLink);
//
//         // FIXME: implement
//         (void)Instance;
//
//         /* Reset the pointer and move to next entry */
//         Instance = NULL;
//         CurrentEntry = CurrentEntry->Flink;
//     }
//
//     /* We're done with instances now */
//     ExReleaseResourceLite(&Filter->InstanceList.rLock);
//     KeLeaveCriticalRegion();
//
//     /* Remove the reference from the base object */
//     FltObjectDereference(&Filter->Base);
//
//     /* Wait until we're sure nothing is using the filter */
//     FltpObjectRundownWait(&Filter->Base.RundownRef);
//
//     /* Delete the instance list lock */
//     ExDeleteResourceLite(&Filter->InstanceList.rLock);
//
//     /* We're finished cleaning up now */
//     FltpExRundownCompleted(&Filter->Base.RundownRef);
//
//     /* Hand the memory back */
//     ExFreePoolWithTag(Filter, FM_TAG_FILTER);
// }
//
// NTSTATUS
// NTAPI
// FltStartFiltering( PFLT_FILTER Filter)
// {
//     NTSTATUS Status;
//
//     /* Grab a ref to the filter */
//     Status = FltObjectReference(&Filter->Base);
//     if (NT_SUCCESS(Status))
//     {
//         /* Make sure we aren't already starting up */
//         if (!(Filter->Flags & FLTFL_FILTERING_INITIATED))
//         {
//             // Startup
//         }
//         else
//         {
//             Status = STATUS_INVALID_PARAMETER;
//         }
//
//         FltObjectDereference(&Filter->Base);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// FltGetFilterFromName( PCUNICODE_STRING FilterName,
//                       PFLT_FILTER *RetFilter)
// {
//    UNIMPLEMENTED;
//     UNREFERENCED_PARAMETER(FilterName);
//     *RetFilter = NULL;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /* INTERNAL FUNCTIONS ******************************************************/
//
// NTSTATUS
// FltpStartingToDrainObject( PFLT_OBJECT Object)
// {
//     /*
//      * Set the draining flag for the filter. This let's us force
//      * a post op callback for minifilters currently awaiting one.
//      */
//     if (InterlockedOr((PLONG)&Object->Flags, FLT_OBFL_DRAINING) & 1)
//     {
//         /* We've been called once, we're already being deleted */
//         return STATUS_FLT_DELETING_OBJECT;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// FltpMiniFilterDriverUnload()
// {
//     __debugbreak();
// }
//
//
// NTSTATUS
// FltpAttachFrame(
//      PUNICODE_STRING Altitude,
//      PFLTP_FRAME *Frame)
// {
//     UNIMPLEMENTED;
//     UNREFERENCED_PARAMETER(Altitude);
//     *Frame = NULL;
//     return STATUS_SUCCESS;
// }
//
// /* PRIVATE FUNCTIONS ******************************************************/
//
// static
// NTSTATUS
// GetFilterAltitude( PFLT_FILTER Filter,
//                    PUNICODE_STRING AltitudeString)
// {
//     UNICODE_STRING InstancesKey = RTL_CONSTANT_STRING(L"Instances");
//     UNICODE_STRING DefaultInstance = RTL_CONSTANT_STRING(L"DefaultInstance");
//     UNICODE_STRING Altitude = RTL_CONSTANT_STRING(L"Altitude");
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING FilterInstancePath;
//     ULONG BytesRequired;
//     HANDLE InstHandle = NULL;
//     HANDLE RootHandle;
//     PWCH InstBuffer = NULL;
//     PWCH AltBuffer = NULL;
//     NTSTATUS Status;
//
//     /* Get a handle to the instances key in the filter's services key */
//     Status = FltpOpenFilterServicesKey(Filter,
//                                        KEY_QUERY_VALUE,
//                                        &InstancesKey,
//                                        &RootHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Read the size 'default instances' string value */
//     Status = FltpReadRegistryValue(RootHandle,
//                                    &DefaultInstance,
//                                    REG_SZ,
//                                    NULL,
//                                    0,
//                                    &BytesRequired);
//
//     /* We should get a buffer too small error */
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//         /* Allocate the buffer we need to hold the string */
//         InstBuffer = ExAllocatePoolWithTag(PagedPool, BytesRequired, FM_TAG_UNICODE_STRING);
//         if (InstBuffer == NULL)
//         {
//             Status = STATUS_INSUFFICIENT_RESOURCES;
//             goto Quit;
//         }
//
//         /* Now read the string value */
//         Status = FltpReadRegistryValue(RootHandle,
//                                        &DefaultInstance,
//                                        REG_SZ,
//                                        InstBuffer,
//                                        BytesRequired,
//                                        &BytesRequired);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         goto Quit;
//     }
//
//     /* Convert the string to a unicode_string */
//     RtlInitUnicodeString(&FilterInstancePath, InstBuffer);
//
//     /* Setup the attributes using the root key handle */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &FilterInstancePath,
//                                OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
//                                RootHandle,
//                                NULL);
//
//     /* Now open the key name which was stored in the default instance */
//     Status = ZwOpenKey(&InstHandle, KEY_QUERY_VALUE, &ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         /* Get the size of the buffer that holds the altitude */
//         Status = FltpReadRegistryValue(InstHandle,
//                                        &Altitude,
//                                        REG_SZ,
//                                        NULL,
//                                        0,
//                                        &BytesRequired);
//         if (Status == STATUS_BUFFER_TOO_SMALL)
//         {
//             /* Allocate the required buffer */
//             AltBuffer = ExAllocatePoolWithTag(PagedPool, BytesRequired, FM_TAG_UNICODE_STRING);
//             if (AltBuffer == NULL)
//             {
//                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                 goto Quit;
//             }
//
//             /* And now finally read in the actual altitude string */
//             Status = FltpReadRegistryValue(InstHandle,
//                                            &Altitude,
//                                            REG_SZ,
//                                            AltBuffer,
//                                            BytesRequired,
//                                            &BytesRequired);
//             if (NT_SUCCESS(Status))
//             {
//                 /* We made it, setup the return buffer */
//                 AltitudeString->Length = BytesRequired;
//                 AltitudeString->MaximumLength = BytesRequired;
//                 AltitudeString->Buffer = AltBuffer;
//             }
//         }
//     }
//
// Quit:
//     if (!NT_SUCCESS(Status))
//     {
//         if (AltBuffer)
//         {
//             ExFreePoolWithTag(AltBuffer, FM_TAG_UNICODE_STRING);
//         }
//     }
//
//     if (InstBuffer)
//     {
//         ExFreePoolWithTag(InstBuffer, FM_TAG_UNICODE_STRING);
//     }
//
//     if (InstHandle)
//     {
//         ZwClose(InstHandle);
//     }
//     ZwClose(RootHandle);
//
//     return Status;
// }
//
// static
// NTSTATUS
// GetFilterFrame( PFLT_FILTER Filter,
//                 PUNICODE_STRING Altitude,
//                 PFLTP_FRAME *Frame)
// {
//     UNIMPLEMENTED;
//     UNREFERENCED_PARAMETER(Filter);
//     UNREFERENCED_PARAMETER(Altitude);
//
//     //
//     // Try to find a frame from our existing filter list (see FilterList)
//     // If none exists, create a new frame, add it and return it
//     //
//
//     *Frame = NULL;
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
