use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, KeInitializeSpinLock, KeInitializeTimer, PDEVICE_OBJECT, PFILE_OBJECT, PHANDLE, PIO_STACK_LOCATION, PIRP, PLONGLONG, RtlFreeUnicodeString, RtlStringFromGUID};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const KSSTRING_Clock: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x005\x003\x001\x007\x002\x004\08\x000\0-\x004\x007\09\x001\0-\x001\x001\0D\x000\0-\0A\x005\0D\x006\0-\x002\08\0D\0B\x000\x004\0C\x001\x000\x000\x000\x000\0}\0\0\0",
    )
};

pub static mut ClockPropertyTable: [KSPROPERTY_ITEM; 7] = unsafe {
    [
        KSPROPERTY_ITEM {
            PropertyId: KSPROPERTY_CLOCK_TIME as ::core::ffi::c_int as ULONG,
            c2rust_unnamed: C2RustUnnamed_95 {
                GetPropertyHandler: ::core::mem::transmute::<
                    Option<unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS>,
                    PFNKSHANDLER,
                >(Some(
                    ClockPropertyTime
                        as unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS,
                )),
            },
            MinProperty: ::core::mem::size_of::<KSPROPERTY>() as ULONG,
            MinData: ::core::mem::size_of::<LONGLONG>() as ULONG,
            c2rust_unnamed_0: C2RustUnnamed_94 {
                SetPropertyHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
                ),
            },
            Values: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY_VALUES as *const KSPROPERTY_VALUES,
            RelationsCount: 0 as ULONG,
            Relations: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY as *const KSPROPERTY,
            SupportHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
            ),
            SerializedSize: 0 as ::core::ffi::c_int as ULONG,
        },
        KSPROPERTY_ITEM {
            PropertyId: KSPROPERTY_CLOCK_PHYSICALTIME as ::core::ffi::c_int as ULONG,
            c2rust_unnamed: C2RustUnnamed_95 {
                GetPropertyHandler: ::core::mem::transmute::<
                    Option<unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS>,
                    PFNKSHANDLER,
                >(Some(
                    ClockPropertyPhysicalTime
                        as unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS,
                )),
            },
            MinProperty: ::core::mem::size_of::<KSPROPERTY>() as ULONG,
            MinData: ::core::mem::size_of::<LONGLONG>() as ULONG,
            c2rust_unnamed_0: C2RustUnnamed_94 {
                SetPropertyHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
                ),
            },
            Values: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY_VALUES as *const KSPROPERTY_VALUES,
            RelationsCount: 0 as ULONG,
            Relations: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY as *const KSPROPERTY,
            SupportHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
            ),
            SerializedSize: 0 as ::core::ffi::c_int as ULONG,
        },
        KSPROPERTY_ITEM {
            PropertyId: KSPROPERTY_CLOCK_CORRELATEDTIME as ::core::ffi::c_int as ULONG,
            c2rust_unnamed: C2RustUnnamed_95 {
                GetPropertyHandler: ::core::mem::transmute::<
                    Option<unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS>,
                    PFNKSHANDLER,
                >(Some(
                    ClockPropertyCorrelatedTime
                        as unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS,
                )),
            },
            MinProperty: ::core::mem::size_of::<KSPROPERTY>() as ULONG,
            MinData: ::core::mem::size_of::<KSCORRELATED_TIME>() as ULONG,
            c2rust_unnamed_0: C2RustUnnamed_94 {
                SetPropertyHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
                ),
            },
            Values: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY_VALUES as *const KSPROPERTY_VALUES,
            RelationsCount: 0 as ULONG,
            Relations: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY as *const KSPROPERTY,
            SupportHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
            ),
            SerializedSize: 0 as ::core::ffi::c_int as ULONG,
        },
        KSPROPERTY_ITEM {
            PropertyId: KSPROPERTY_CLOCK_CORRELATEDPHYSICALTIME as ::core::ffi::c_int as ULONG,
            c2rust_unnamed: C2RustUnnamed_95 {
                GetPropertyHandler: ::core::mem::transmute::<
                    Option<unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS>,
                    PFNKSHANDLER,
                >(Some(
                    ClockPropertyCorrelatedPhysicalTime
                        as unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS,
                )),
            },
            MinProperty: ::core::mem::size_of::<KSPROPERTY>() as ULONG,
            MinData: ::core::mem::size_of::<KSCORRELATED_TIME>() as ULONG,
            c2rust_unnamed_0: C2RustUnnamed_94 {
                SetPropertyHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
                ),
            },
            Values: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY_VALUES as *const KSPROPERTY_VALUES,
            RelationsCount: 0 as ULONG,
            Relations: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY as *const KSPROPERTY,
            SupportHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
            ),
            SerializedSize: 0 as ::core::ffi::c_int as ULONG,
        },
        KSPROPERTY_ITEM {
            PropertyId: KSPROPERTY_CLOCK_RESOLUTION as ::core::ffi::c_int as ULONG,
            c2rust_unnamed: C2RustUnnamed_95 {
                GetPropertyHandler: ::core::mem::transmute::<
                    Option<unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS>,
                    PFNKSHANDLER,
                >(Some(
                    ClockPropertyResolution
                        as unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS,
                )),
            },
            MinProperty: ::core::mem::size_of::<KSPROPERTY>() as ULONG,
            MinData: ::core::mem::size_of::<KSRESOLUTION>() as ULONG,
            c2rust_unnamed_0: C2RustUnnamed_94 {
                SetPropertyHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
                ),
            },
            Values: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY_VALUES as *const KSPROPERTY_VALUES,
            RelationsCount: 0 as ULONG,
            Relations: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY as *const KSPROPERTY,
            SupportHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
            ),
            SerializedSize: 0 as ::core::ffi::c_int as ULONG,
        },
        KSPROPERTY_ITEM {
            PropertyId: KSPROPERTY_CLOCK_STATE as ::core::ffi::c_int as ULONG,
            c2rust_unnamed: C2RustUnnamed_95 {
                GetPropertyHandler: ::core::mem::transmute::<
                    Option<unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS>,
                    PFNKSHANDLER,
                >(Some(
                    ClockPropertyState
                        as unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS,
                )),
            },
            MinProperty: ::core::mem::size_of::<KSPROPERTY>() as ULONG,
            MinData: ::core::mem::size_of::<KSSTATE>() as ULONG,
            c2rust_unnamed_0: C2RustUnnamed_94 {
                SetPropertyHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
                ),
            },
            Values: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY_VALUES as *const KSPROPERTY_VALUES,
            RelationsCount: 0 as ULONG,
            Relations: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY as *const KSPROPERTY,
            SupportHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
            ),
            SerializedSize: 0 as ::core::ffi::c_int as ULONG,
        },
        KSPROPERTY_ITEM {
            PropertyId: KSPROPERTY_CLOCK_FUNCTIONTABLE as ::core::ffi::c_int as ULONG,
            c2rust_unnamed: C2RustUnnamed_95 {
                GetPropertyHandler: ::core::mem::transmute::<
                    Option<unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS>,
                    PFNKSHANDLER,
                >(Some(
                    ClockPropertyFunctionTable
                        as unsafe extern "C" fn(PIRP, PKSIDENTIFIER, PVOID) -> NTSTATUS,
                )),
            },
            MinProperty: ::core::mem::size_of::<KSPROPERTY>() as ULONG,
            MinData: ::core::mem::size_of::<KSCLOCK_FUNCTIONTABLE>() as ULONG,
            c2rust_unnamed_0: C2RustUnnamed_94 {
                SetPropertyHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
                ),
            },
            Values: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY_VALUES as *const KSPROPERTY_VALUES,
            RelationsCount: 0 as ULONG,
            Relations: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void
                as PKSPROPERTY as *const KSPROPERTY,
            SupportHandler: ::core::mem::transmute::<*mut ::core::ffi::c_void, PFNKSHANDLER>(
                ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
            ),
            SerializedSize: 0 as ::core::ffi::c_int as ULONG,
        },
    ]
};

pub static mut ClockPropertySet: [KSPROPERTY_SET; 1] = [KSPROPERTY_SET {
    Set: ::core::ptr::null::<GUID>(),
    PropertiesCount: 0,
    PropertyItem: ::core::ptr::null::<KSPROPERTY_ITEM>(),
    FastIoCount: 0,
    FastIoTable: ::core::ptr::null::<KSFASTPROPERTY_ITEM>(),
}; 1];

static mut DispatchTable: KSDISPATCH_TABLE = unsafe {
    KSDISPATCH_TABLE {
        DeviceIoControl: Some(
            IKsClock_DispatchDeviceIoControl
                as unsafe extern "C" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
        ),
        Read: Some(
            KsDispatchInvalidDeviceRequest
                as unsafe extern "C" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
        ),
        Write: Some(
            KsDispatchInvalidDeviceRequest
                as unsafe extern "C" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
        ),
        Flush: Some(
            KsDispatchInvalidDeviceRequest
                as unsafe extern "C" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
        ),
        Close: Some(
            IKsClock_DispatchClose as unsafe extern "C" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
        ),
        QuerySecurity: Some(
            KsDispatchQuerySecurity as unsafe extern "C" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
        ),
        SetSecurity: Some(
            KsDispatchSetSecurity as unsafe extern "C" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
        ),
        FastDeviceIoControl: Some(
            KsDispatchFastIoDeviceControlFailure
                as unsafe extern "C" fn(
                    PFILE_OBJECT,
                    BOOLEAN,
                    PVOID,
                    ULONG,
                    PVOID,
                    ULONG,
                    ULONG,
                    PIO_STATUS_BLOCK,
                    PDEVICE_OBJECT,
                ) -> BOOLEAN,
        ),
        FastRead: Some(
            KsDispatchFastReadFailure
                as unsafe extern "C" fn(
                    PFILE_OBJECT,
                    PLARGE_INTEGER,
                    ULONG,
                    BOOLEAN,
                    ULONG,
                    PVOID,
                    PIO_STATUS_BLOCK,
                    PDEVICE_OBJECT,
                ) -> BOOLEAN,
        ),
        FastWrite: Some(
            KsDispatchFastReadFailure
                as unsafe extern "C" fn(
                    PFILE_OBJECT,
                    PLARGE_INTEGER,
                    ULONG,
                    BOOLEAN,
                    ULONG,
                    PVOID,
                    PIO_STATUS_BLOCK,
                    PDEVICE_OBJECT,
                ) -> BOOLEAN,
        ),
    }
};

static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];

#[repr(C)]
pub struct KSIDEFAULTCLOCK {
    pub Time: LONGLONG,
    pub TimeLock: KSPIN_LOCK,
    pub State: KSSTATE,
    pub Timer: KTIMER,
    pub ReferenceCount: LONG,
    pub Context: PVOID,
    pub SetTimer: PFNKSSETTIMER,
    pub CancelTimer: PFNKSCANCELTIMER,
    pub CorrelatedTime: PFNKSCORRELATEDTIME,
    pub Granularity: LONGLONG,
    pub Error: LONGLONG,
    pub Flags: ULONG,
}
pub type PKSIDEFAULTCLOCK = *mut KSIDEFAULTCLOCK;

#[repr(C)]
pub struct KSICLOCK {
    pub ref: LONG,
    pub ClockCreate: PKSCLOCK_CREATE,
    pub DefaultClock: PKSIDEFAULTCLOCK,
    pub ObjectHeader: PKSIOBJECT_HEADER,
}
pub type PKSICLOCK = *mut KSICLOCK;
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ClockGetPhysicalTime(
    FileObject: PFILE_OBJECT,
) -> LONGLONG {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"ClockGetPhysicalTime\0".as_ptr() as *const i8,
        b"drivers/ksfilter/ks/clocks.c\0".as_ptr() as *const i8,
        66i32,
    );
    0i64
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ClockGetPhysicalTime(
    FileObject: PFILE_OBJECT,
) -> LONGLONG {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"ClockGetPhysicalTime\0".as_ptr() as *const i8,
        b"drivers/ksfilter/ks/clocks.c\0".as_ptr() as *const i8,
        66i32,
    );
    0i64
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ClockGetCorrelatedTime(
    FileObject: PFILE_OBJECT,
    SystemTime: PLONGLONG,
) -> LONGLONG {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"ClockGetCorrelatedTime\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/ksfilter/ks/clocks.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        76i32,
    );
    return 0i64;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ClockGetCorrelatedTime(
    FileObject: PFILE_OBJECT,
    SystemTime: PLONGLONG,
) -> LONGLONG {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"ClockGetCorrelatedTime\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/ksfilter/ks/clocks.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        76i32,
    );
    return 0i64;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ClockGetTime(
    FileObject: PFILE_OBJECT,
) -> LONGLONG {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"ClockGetTime\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/ksfilter/ks/clocks.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        85i32,
    );
    return 0i64;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ClockGetTime(
    FileObject: PFILE_OBJECT,
) -> LONGLONG {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"ClockGetTime\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/ksfilter/ks/clocks.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        85i32,
    );
    return 0i64;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ClockGetCorrelatedPhysicalTime(
    FileObject: PFILE_OBJECT,
    SystemTime: PLONGLONG,
) -> LONGLONG {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"ClockGetCorrelatedPhysicalTime\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/ksfilter/ks/clocks.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        95i32,
    );
    0i64
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ClockGetCorrelatedPhysicalTime(
    FileObject: PFILE_OBJECT,
    SystemTime: PLONGLONG,
) -> LONGLONG {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"ClockGetCorrelatedPhysicalTime\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/ksfilter/ks/clocks.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        95i32,
    );
    0i64
}
#[no_mangle]
pub unsafe extern "system" fn ClockPropertyTime(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let Time: PLONGLONG = Data as PLONGLONG;
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    *Time = ClockGetTime((*IoStack).FileObject);

    (*Irp).IoStatus.Information = core::mem::size_of::<LONGLONG>() as ULONG_PTR;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn ClockPropertyPhysicalTime(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let Time: PLONGLONG = Data as PLONGLONG;
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    *Time = ClockGetPhysicalTime((*IoStack).FileObject);

    (*Irp).IoStatus.Information = core::mem::size_of::<LONGLONG>() as ULONG_PTR;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn ClockPropertyCorrelatedTime(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let Time = Data as PKSCORRELATED_TIME;
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    (*Time).Time = ClockGetCorrelatedTime((*IoStack).FileObject, &mut (*Time).SystemTime);

    (*Irp).IoStatus.Information = core::mem::size_of::<KSCORRELATED_TIME>() as ULONG_PTR;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ClockPropertyCorrelatedPhysicalTime(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let Time: PKSCORRELATED_TIME = Data as PKSCORRELATED_TIME;
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    (*Time).Time = ClockGetCorrelatedPhysicalTime(
        (*IoStack).FileObject,
        core::ptr::addr_of_mut!((*Time).SystemTime),
    );

    (*Irp).IoStatus.Information = core::mem::size_of::<KSCORRELATED_TIME>() as ULONG_PTR;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ClockPropertyResolution(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let ObjectHeader = (*((*IoStack).FileObject)).FsContext2 as PKSIOBJECT_HEADER;
    let Clock = (*ObjectHeader).ObjectType as PKSICLOCK;
    let Resolution = Data as PKSRESOLUTION;

    (*Resolution).Error = (*(*Clock).DefaultClock).Error;
    (*Resolution).Granularity = (*(*Clock).DefaultClock).Granularity;

    (*Irp).IoStatus.Information = core::mem::size_of::<KSRESOLUTION>() as ULONG_PTR;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ClockPropertyState(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let State = Data as PKSSTATE;
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let ObjectHeader = (*((*IoStack).FileObject)).FsContext2 as PKSIOBJECT_HEADER;
    let Clock = (*ObjectHeader).ObjectType as PKSICLOCK;
    *State = (*(*Clock).DefaultClock).State;
    (*Irp).IoStatus.Information = core::mem::size_of::<KSSTATE>() as ULONG_PTR;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ClockPropertyFunctionTable(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let Table = Data as PKSCLOCK_FUNCTIONTABLE;

    (*Table).GetCorrelatedPhysicalTime = Some(
        ClockGetCorrelatedPhysicalTime as unsafe extern "system" fn(PFILE_OBJECT, PLONGLONG) -> LONGLONG,
    ) as PFNKSCLOCK_CORRELATEDTIME;
    (*Table).GetCorrelatedTime =
        Some(ClockGetCorrelatedTime as unsafe extern "system" fn(PFILE_OBJECT, PLONGLONG) -> LONGLONG)
            as PFNKSCLOCK_CORRELATEDTIME;
    (*Table).GetPhysicalTime =
        Some(ClockGetPhysicalTime as unsafe extern "system" fn(PFILE_OBJECT) -> LONGLONG)
            as PFNKSCLOCK_GETTIME;
    (*Table).GetTime =
        Some(ClockGetTime as unsafe extern "system" fn(PFILE_OBJECT) -> LONGLONG) as PFNKSCLOCK_GETTIME;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsCreateClock(
    ConnectionHandle: HANDLE,
    ClockCreate: PKSCLOCK_CREATE,
    ClockHandle: PHANDLE,
) -> NTSTATUS {
    return KspCreateObjectType(
        ConnectionHandle,
        KSSTRING_Clock.as_ptr() as LPWSTR,
        ClockCreate as PVOID,
        core::mem::size_of::<KSCLOCK_CREATE>() as UINT,
        GENERIC_READ,
        ClockHandle,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsValidateClockCreateRequest(
    Irp: PIRP,
    OutClockCreate: *mut PKSCLOCK_CREATE,
) -> NTSTATUS {
    let mut ClockCreate: PKSCLOCK_CREATE = core::ptr::null_mut();
    let mut Size: ULONG = core::mem::size_of::<KSCLOCK_CREATE>() as ULONG;

    let Status = KspCopyCreateRequest(
        Irp,
        KSSTRING_Clock.as_ptr() as LPWSTR,
        &mut Size,
        &mut ClockCreate as *mut PKSCLOCK_CREATE as *mut PVOID,
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*ClockCreate).CreateFlags != 0 {
        FreeItem(ClockCreate as PVOID);
        return STATUS_INVALID_PARAMETER;
    }

    *OutClockCreate = ClockCreate;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IKsClock_DispatchDeviceIoControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let mut GuidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let Property: PKSPROPERTY;
    let Status: NTSTATUS;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    debug_assert!((*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_PROPERTY);
    debug_assert!((*IoStack).Parameters.DeviceIoControl.InputBufferLength as usize >= core::mem::size_of::<KSPROPERTY>());

    Status = KsPropertyHandler(Irp, 1, ClockPropertySet.as_mut_ptr());

    Property = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as PKSPROPERTY;

    RtlStringFromGUID(&raw mut (*Property).Set, &raw mut GuidString);
    RtlFreeUnicodeString(&raw mut GuidString);

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IKsClock_DispatchClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsCreateDefaultClock(
    Irp: PIRP,
    DefaultClock: PKSDEFAULTCLOCK,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ClockCreate: PKSCLOCK_CREATE;
    let mut Clock: PKSICLOCK;

    Status = KsValidateClockCreateRequest(Irp, &mut ClockCreate);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Clock = AllocateItem(NonPagedPool, core::mem::size_of::<KSICLOCK>()) as PKSICLOCK;
    if Clock.is_null() {
        FreeItem(ClockCreate as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = KsAllocateObjectHeader(
        &mut (*Clock).ObjectHeader as *mut _ as PVOID,
        0,
        core::ptr::null_mut(),
        Irp,
        &mut DispatchTable,
    );

    if !NT_SUCCESS(Status) {
        FreeItem(ClockCreate as PVOID);
        FreeItem(Clock as PVOID);
        return Status;
    }

    (*(*Clock).ObjectHeader).ObjectType = Clock as PVOID;
    (*Clock).ref = 1;
    (*Clock).ClockCreate = ClockCreate;
    (*Clock).DefaultClock = DefaultClock as PKSIDEFAULTCLOCK;

    InterlockedIncrement(&mut (*((*Clock).DefaultClock)).ReferenceCount);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsAllocateDefaultClock(
    DefaultClock: *mut PKSDEFAULTCLOCK,
) -> NTSTATUS {
    return KsAllocateDefaultClockEx(
        DefaultClock,
        core::ptr::null_mut(),
        None,
        None,
        None,
        core::ptr::null(),
        0,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsAllocateDefaultClockEx(
    DefaultClock: *mut PKSDEFAULTCLOCK,
    Context: PVOID,
    SetTimer: PFNKSSETTIMER,
    CancelTimer: PFNKSCANCELTIMER,
    CorrelatedTime: PFNKSCORRELATEDTIME,
    Resolution: *const KSRESOLUTION,
    Flags: ULONG,
) -> NTSTATUS {
    let mut Clock: PKSIDEFAULTCLOCK;

    if DefaultClock.is_null() {
        return STATUS_INVALID_PARAMETER_1;
    }

    Clock = AllocateItem(NonPagedPool, core::mem::size_of::<KSIDEFAULTCLOCK>()) as PKSIDEFAULTCLOCK;
    if Clock.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KeInitializeSpinLock(&mut (*Clock).TimeLock);
    KeInitializeTimer(&mut (*Clock).Timer);
    (*Clock).ReferenceCount = 1;
    (*Clock).Context = Context;
    (*Clock).SetTimer = SetTimer;
    (*Clock).CancelTimer = CancelTimer;
    (*Clock).CorrelatedTime = CorrelatedTime;
    (*Clock).Flags = Flags;

    if !Resolution.is_null() {
        if !SetTimer.is_null() {
            (*Clock).Error = (*Resolution).Error;
        }

        if !CorrelatedTime.is_null() {
            (*Clock).Granularity = (*Resolution).Granularity;
        }
    }

    *DefaultClock = Clock as PKSDEFAULTCLOCK;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsFreeDefaultClock(
    DefaultClock: PKSDEFAULTCLOCK,
) {
    let Clock: PKSIDEFAULTCLOCK = DefaultClock as PKSIDEFAULTCLOCK;

    InterlockedDecrement(&mut (*Clock).ReferenceCount);

    if (*Clock).ReferenceCount == 0 {
        FreeItem(Clock as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KsGetDefaultClockState(DefaultClock: PKSDEFAULTCLOCK) -> KSSTATE {
    let Clock: PKSIDEFAULTCLOCK = DefaultClock as PKSIDEFAULTCLOCK;
    return (*Clock).State;
}

#[no_mangle]
pub unsafe extern "system" fn KsSetDefaultClockState(
    DefaultClock: PKSDEFAULTCLOCK,
    State: KSSTATE,
) {
    let Clock: PKSIDEFAULTCLOCK = DefaultClock as PKSIDEFAULTCLOCK;
    if State as u32 != (*Clock).State as u32 {
        (*Clock).State = State;
    }
}

#[no_mangle]
pub unsafe extern "system" fn KsGetDefaultClockTime(
    DefaultClock: PKSDEFAULTCLOCK,
) -> LONGLONG {
    let mut Time: LONGLONG = 0i64;
    let Clock: PKSIDEFAULTCLOCK = DefaultClock as PKSIDEFAULTCLOCK;
    Time = ExInterlockedCompareExchange64(
        &mut (*Clock).Time,
        &mut Time,
        &mut Time,
        &mut (*Clock).TimeLock,
    );
    Time
}

#[no_mangle]
pub unsafe extern "system" fn KsSetDefaultClockTime(
    DefaultClock: PKSDEFAULTCLOCK,
    Time: LONGLONG,
) {
    let Clock: PKSIDEFAULTCLOCK = DefaultClock as PKSIDEFAULTCLOCK;
    ExInterlockedCompareExchange64(
        &mut (*Clock).Time,
        &Time,
        &(*Clock).Time,
        &mut (*Clock).TimeLock,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/clocks.c
//  * PURPOSE:         KS Clocks functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// typedef struct
// {
//     LONGLONG Time;
//     KSPIN_LOCK TimeLock;
//     KSSTATE State;
//     KTIMER Timer;
//     LONG ReferenceCount;
//
//     PVOID Context;
//     PFNKSSETTIMER SetTimer;
//     PFNKSCANCELTIMER CancelTimer;
//     PFNKSCORRELATEDTIME CorrelatedTime;
//     LONGLONG Granularity;
//     LONGLONG Error;
//     ULONG Flags;
//
// }KSIDEFAULTCLOCK, *PKSIDEFAULTCLOCK;
//
// typedef struct
// {
//     LONG ref;
//     PKSCLOCK_CREATE ClockCreate;
//     PKSIDEFAULTCLOCK DefaultClock;
//     PKSIOBJECT_HEADER ObjectHeader;
// }KSICLOCK, *PKSICLOCK;
//
// NTSTATUS NTAPI ClockPropertyTime(IN PIRP Irp, IN PKSIDENTIFIER Request, IN OUT PVOID Data);
// NTSTATUS NTAPI ClockPropertyPhysicalTime(IN PIRP Irp, IN PKSIDENTIFIER Request, IN OUT PVOID Data);
// NTSTATUS NTAPI ClockPropertyCorrelatedTime(IN PIRP Irp, IN PKSIDENTIFIER Request, IN OUT PVOID Data);
// NTSTATUS NTAPI ClockPropertyCorrelatedPhysicalTime(IN PIRP Irp, IN PKSIDENTIFIER Request, IN OUT PVOID Data);
// NTSTATUS NTAPI ClockPropertyResolution(IN PIRP Irp, IN PKSIDENTIFIER Request, IN OUT PVOID Data);
// NTSTATUS NTAPI ClockPropertyState(IN PIRP Irp, IN PKSIDENTIFIER Request, IN OUT PVOID Data);
// NTSTATUS NTAPI ClockPropertyFunctionTable(IN PIRP Irp, IN PKSIDENTIFIER Request, IN OUT PVOID Data);
//
// DEFINE_KSPROPERTY_CLOCKSET(ClockPropertyTable, ClockPropertyTime, ClockPropertyPhysicalTime, ClockPropertyCorrelatedTime, ClockPropertyCorrelatedPhysicalTime, ClockPropertyResolution, ClockPropertyState, ClockPropertyFunctionTable);
//
// KSPROPERTY_SET ClockPropertySet[] =
// {
//     {
//         &KSPROPSETID_Clock,
//         sizeof(ClockPropertyTable) / sizeof(KSPROPERTY_ITEM),
//         (const KSPROPERTY_ITEM*)&ClockPropertyTable,
//         0,
//         NULL
//     }
// };
//
// LONGLONG
// FASTCALL
// ClockGetPhysicalTime(
//     IN PFILE_OBJECT FileObject)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// LONGLONG
// FASTCALL
// ClockGetCorrelatedTime(
//     IN PFILE_OBJECT FileObject,
//     OUT PLONGLONG SystemTime)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// LONGLONG
// FASTCALL
// ClockGetTime(
//     IN PFILE_OBJECT FileObject)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// LONGLONG
// FASTCALL
// ClockGetCorrelatedPhysicalTime(
//     IN PFILE_OBJECT FileObject,
//     OUT PLONGLONG SystemTime)
// {
//     UNIMPLEMENTED;
//     return 0;
// }
//
// NTSTATUS
// NTAPI
// ClockPropertyTime(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER Request,
//     IN OUT PVOID Data)
// {
//     PLONGLONG Time = (PLONGLONG)Data;
//     PIO_STACK_LOCATION IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     DPRINT("ClockPropertyTime\n");
//
//     *Time = ClockGetTime(IoStack->FileObject);
//
//     Irp->IoStatus.Information = sizeof(LONGLONG);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// ClockPropertyPhysicalTime(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER Request,
//     IN OUT PVOID Data)
// {
//     PLONGLONG Time = (PLONGLONG)Data;
//     PIO_STACK_LOCATION IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     DPRINT("ClockPropertyPhysicalTime\n");
//
//     *Time = ClockGetPhysicalTime(IoStack->FileObject);
//
//     Irp->IoStatus.Information = sizeof(LONGLONG);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// ClockPropertyCorrelatedTime(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER Request,
//     IN OUT PVOID Data)
// {
//     PKSCORRELATED_TIME Time = (PKSCORRELATED_TIME)Data;
//     PIO_STACK_LOCATION IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     DPRINT("ClockPropertyCorrelatedTime\n");
//
//     Time->Time = ClockGetCorrelatedTime(IoStack->FileObject, &Time->SystemTime);
//
//     Irp->IoStatus.Information = sizeof(KSCORRELATED_TIME);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// ClockPropertyCorrelatedPhysicalTime(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER Request,
//     IN OUT PVOID Data)
// {
//     PKSCORRELATED_TIME Time = (PKSCORRELATED_TIME)Data;
//     PIO_STACK_LOCATION IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     DPRINT("ClockPropertyCorrelatedPhysicalTime\n");
//
//     Time->Time = ClockGetCorrelatedPhysicalTime(IoStack->FileObject, &Time->SystemTime);
//
//     Irp->IoStatus.Information = sizeof(KSCORRELATED_TIME);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// ClockPropertyResolution(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER Request,
//     IN OUT PVOID Data)
// {
//     PKSICLOCK Clock;
//     PKSIOBJECT_HEADER ObjectHeader;
//     PIO_STACK_LOCATION IoStack;
//     PKSRESOLUTION Resolution = (PKSRESOLUTION)Data;
//
//     DPRINT("ClockPropertyResolution\n");
//
//     /* get stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get the object header */
//     ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->FsContext2;
//
//     /* sanity check */
//     ASSERT(ObjectHeader);
//
//     /* locate ks pin implementation from KSPIN offset */
//     Clock = (PKSICLOCK)ObjectHeader->ObjectType;
//
//     Resolution->Error = Clock->DefaultClock->Error;
//     Resolution->Granularity = Clock->DefaultClock->Granularity;
//
//     Irp->IoStatus.Information = sizeof(KSRESOLUTION);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// ClockPropertyState(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER Request,
//     IN OUT PVOID Data)
// {
//     PKSICLOCK Clock;
//     PKSIOBJECT_HEADER ObjectHeader;
//     PKSSTATE State = (PKSSTATE)Data;
//     PIO_STACK_LOCATION IoStack;
//
//     DPRINT("ClockPropertyState\n");
//
//     /* get stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get the object header */
//     ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->FsContext2;
//
//     /* sanity check */
//     ASSERT(ObjectHeader);
//
//     /* locate ks pin implementation from KSPIN offset */
//     Clock = (PKSICLOCK)ObjectHeader->ObjectType;
//
//     *State = Clock->DefaultClock->State;
//     Irp->IoStatus.Information = sizeof(KSSTATE);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// ClockPropertyFunctionTable(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER Request,
//     IN OUT PVOID Data)
// {
//     PKSCLOCK_FUNCTIONTABLE Table = (PKSCLOCK_FUNCTIONTABLE)Data;
//
//     DPRINT("ClockPropertyFunctionTable\n");
//
//     Table->GetCorrelatedPhysicalTime = ClockGetCorrelatedPhysicalTime;
//     Table->GetCorrelatedTime = ClockGetCorrelatedTime;
//     Table->GetPhysicalTime = ClockGetPhysicalTime;
//     Table->GetTime = ClockGetTime;
//
//     return STATUS_SUCCESS;
// }
//
//
// /*
//     @implemented
// */
// KSDDKAPI NTSTATUS NTAPI
// KsCreateClock(
//     IN  HANDLE ConnectionHandle,
//     IN  PKSCLOCK_CREATE ClockCreate,
//     OUT PHANDLE ClockHandle)
// {
//     return KspCreateObjectType(ConnectionHandle,
//                                KSSTRING_Clock,
//                                ClockCreate,
//                                sizeof(KSCLOCK_CREATE),
//                                GENERIC_READ,
//                                ClockHandle);
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsValidateClockCreateRequest(
//     IN  PIRP Irp,
//     OUT PKSCLOCK_CREATE* OutClockCreate)
// {
//     PKSCLOCK_CREATE ClockCreate;
//     NTSTATUS Status;
//     ULONG Size;
//
//     /* minimum request size */
//     Size = sizeof(KSCLOCK_CREATE);
//
//     /* copy create request */
//     Status = KspCopyCreateRequest(Irp,
//                                   KSSTRING_Clock,
//                                   &Size,
//                                   (PVOID*)&ClockCreate);
//
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if (ClockCreate->CreateFlags != 0)
//     {
//         /* flags must be zero */
//         FreeItem(ClockCreate);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     *OutClockCreate = ClockCreate;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// IKsClock_DispatchDeviceIoControl(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN  PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     UNICODE_STRING GuidString;
//     PKSPROPERTY Property;
//     NTSTATUS Status;
//
//     DPRINT("IKsClock_DispatchDeviceIoControl\n");
//
//     /* get current io stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* FIXME support events */
//     ASSERT(IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_PROPERTY);
//
//     /* sanity check */
//     ASSERT(IoStack->Parameters.DeviceIoControl.InputBufferLength >= sizeof(KSPROPERTY));
//
//     /* call property handler */
//     Status = KsPropertyHandler(Irp, 1, ClockPropertySet);
//
//     /* get property from input buffer */
//     Property = (PKSPROPERTY)IoStack->Parameters.DeviceIoControl.Type3InputBuffer;
//
//     RtlStringFromGUID(&Property->Set, &GuidString);
//     DPRINT("IKsClock_DispatchDeviceIoControl property Set |%S| Id %u Flags %x Status %lx ResultLength %lu\n", GuidString.Buffer, Property->Id, Property->Flags, Status, Irp->IoStatus.Information);
//     RtlFreeUnicodeString(&GuidString);
//
//
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// IKsClock_DispatchClose(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN  PIRP Irp)
// {
//     UNIMPLEMENTED;
//
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return STATUS_SUCCESS;
// }
//
// static KSDISPATCH_TABLE DispatchTable =
// {
//     IKsClock_DispatchDeviceIoControl,
//     KsDispatchInvalidDeviceRequest,
//     KsDispatchInvalidDeviceRequest,
//     KsDispatchInvalidDeviceRequest,
//     IKsClock_DispatchClose,
//     KsDispatchQuerySecurity,
//     KsDispatchSetSecurity,
//     KsDispatchFastIoDeviceControlFailure,
//     KsDispatchFastReadFailure,
//     KsDispatchFastReadFailure,
// };
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsCreateDefaultClock(
//     IN  PIRP Irp,
//     IN  PKSDEFAULTCLOCK DefaultClock)
// {
//     NTSTATUS Status;
//     PKSCLOCK_CREATE ClockCreate;
//     PKSICLOCK Clock;
//
//     Status = KsValidateClockCreateRequest(Irp, &ClockCreate);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* let's allocate the clock struct */
//     Clock = AllocateItem(NonPagedPool, sizeof(KSICLOCK));
//     if (!Clock)
//     {
//         FreeItem(ClockCreate);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* now allocate the object header */
//     Status = KsAllocateObjectHeader((PVOID*)&Clock->ObjectHeader, 0, NULL, Irp, &DispatchTable);
//
//     /* did it work */
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         FreeItem(ClockCreate);
//         FreeItem(Clock);
//         return Status;
//     }
//
//     /* initialize clock */
//     /* FIXME IKsClock */
//     Clock->ObjectHeader->ObjectType = (PVOID)Clock;
//     Clock->ref = 1;
//     Clock->ClockCreate = ClockCreate;
//     Clock->DefaultClock = (PKSIDEFAULTCLOCK)DefaultClock;
//
//     /* increment reference count */
//     InterlockedIncrement(&Clock->DefaultClock->ReferenceCount);
//
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsAllocateDefaultClock(
//     OUT PKSDEFAULTCLOCK* DefaultClock)
// {
//     return KsAllocateDefaultClockEx(DefaultClock, NULL, NULL, NULL, NULL, NULL, 0);
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsAllocateDefaultClockEx(
//     OUT PKSDEFAULTCLOCK* DefaultClock,
//     IN  PVOID Context OPTIONAL,
//     IN  PFNKSSETTIMER SetTimer OPTIONAL,
//     IN  PFNKSCANCELTIMER CancelTimer OPTIONAL,
//     IN  PFNKSCORRELATEDTIME CorrelatedTime OPTIONAL,
//     IN  const KSRESOLUTION* Resolution OPTIONAL,
//     IN  ULONG Flags)
// {
//     PKSIDEFAULTCLOCK Clock;
//
//     if (!DefaultClock)
//        return STATUS_INVALID_PARAMETER_1;
//
//     /* allocate default clock */
//     Clock = AllocateItem(NonPagedPool, sizeof(KSIDEFAULTCLOCK));
//     if (!Clock)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* initialize default clock */
//     KeInitializeSpinLock(&Clock->TimeLock);
//     KeInitializeTimer(&Clock->Timer);
//     Clock->ReferenceCount = 1;
//     Clock->Context = Context;
//     Clock->SetTimer = SetTimer;
//     Clock->CancelTimer = CancelTimer;
//     Clock->CorrelatedTime = CorrelatedTime;
//     Clock->Flags = Flags;
//
//     if (Resolution)
//     {
//         if (SetTimer)
//         {
//             Clock->Error = Resolution->Error;
//         }
//
//         if (CorrelatedTime)
//         {
//             Clock->Granularity = Resolution->Granularity;
//         }
//     }
//
//     *DefaultClock = (PKSDEFAULTCLOCK)Clock;
//     return STATUS_SUCCESS;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsFreeDefaultClock(
//     IN  PKSDEFAULTCLOCK DefaultClock)
// {
//     PKSIDEFAULTCLOCK Clock = (PKSIDEFAULTCLOCK)DefaultClock;
//
//     InterlockedDecrement(&Clock->ReferenceCount);
//
//     if (Clock->ReferenceCount == 0)
//     {
//         /* free default clock */
//         FreeItem(Clock);
//     }
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// KSSTATE
// NTAPI
// KsGetDefaultClockState(
//     IN  PKSDEFAULTCLOCK DefaultClock)
// {
//     PKSIDEFAULTCLOCK Clock = (PKSIDEFAULTCLOCK)DefaultClock;
//     return Clock->State;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsSetDefaultClockState(
//     IN  PKSDEFAULTCLOCK DefaultClock,
//     IN  KSSTATE State)
// {
//     PKSIDEFAULTCLOCK Clock = (PKSIDEFAULTCLOCK)DefaultClock;
//
//     if (State != Clock->State)
//     {
//          /* FIXME set time etc */
//          Clock->State = State;
//     }
//
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// LONGLONG
// NTAPI
// KsGetDefaultClockTime(
//     IN  PKSDEFAULTCLOCK DefaultClock)
// {
//     LONGLONG Time = 0LL;
//     PKSIDEFAULTCLOCK Clock = (PKSIDEFAULTCLOCK)DefaultClock;
//
//     Time = ExInterlockedCompareExchange64(&Clock->Time, &Time, &Time, &Clock->TimeLock);
//
//     return Time;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsSetDefaultClockTime(
//     IN  PKSDEFAULTCLOCK DefaultClock,
//     IN  LONGLONG Time)
// {
//     PKSIDEFAULTCLOCK Clock = (PKSIDEFAULTCLOCK)DefaultClock;
//
//     /* set the time safely */
//     ExInterlockedCompareExchange64(&Clock->Time, &Time, &Clock->Time, &Clock->TimeLock);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
