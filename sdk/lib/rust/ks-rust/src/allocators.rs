use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{ExDeleteNPagedLookasideList, ExDeletePagedLookasideList, ExInitializeNPagedLookasideList, ExInitializePagedLookasideList, GUID, IoGetCurrentIrpStackLocation, PDEVICE_OBJECT, PFILE_OBJECT, PHANDLE, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const KSSTRING_Allocator: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x006\x004\x002\0F\x005\0D\x000\x000\0-\x004\x007\09\x001\0-\x001\x001\0D\x000\0-\0A\x005\0D\x006\0-\x002\08\0D\0B\x000\x004\0C\x001\x000\x000\x000\x000\0}\0\0\0",
    )
};

pub static mut IID_IKsAllocator: GUID = _GUID {
    Data1: 0x642f5d00 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x4791 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut KSPROPSETID_StreamAllocator: GUID = _GUID {
    Data1: 0xcf6e4342 as ::core::ffi::c_uint,
    Data2: 0xec87 as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x30 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x56 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut vt_IKsAllocator: IKsAllocatorVtbl = unsafe {
    IKsAllocatorVtbl {
        QueryInterface: Some(
            IKsAllocator_fnQueryInterface
                as unsafe extern "C" fn(*mut IKsAllocator, *const IID, *mut PVOID) -> NTSTATUS,
        ),
        AddRef: Some(IKsAllocator_fnAddRef as unsafe extern "C" fn(*mut IKsAllocator) -> ULONG),
        Release: Some(IKsAllocator_fnRelease as unsafe extern "C" fn(*mut IKsAllocator) -> ULONG),
        DispatchDeviceIoControl: Some(
            IKsAllocator_fnDeviceIoControl
                as unsafe extern "C" fn(*mut IKsAllocator, PDEVICE_OBJECT, PIRP) -> NTSTATUS,
        ),
        Close: Some(IKsAllocator_fnClose as unsafe extern "C" fn(*mut IKsAllocator) -> NTSTATUS),
        AllocateFrame: Some(
            IKsAllocator_fnAllocateFrame
                as unsafe extern "C" fn(*mut IKsAllocator, *mut PVOID) -> NTSTATUS,
        ),
        FreeFrame: Some(
            IKsAllocator_fnFreeFrame as unsafe extern "C" fn(*mut IKsAllocator, PVOID) -> (),
        ),
    }
};

static mut DispatchTable: KSDISPATCH_TABLE = unsafe {
    KSDISPATCH_TABLE {
        DeviceIoControl: Some(
            IKsAllocator_DispatchDeviceIoControl
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
            IKsAllocator_DispatchClose as unsafe extern "C" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
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

#[repr(C)]
pub enum ALLOCATOR_TYPE {
    ALLOCATOR_NPAGED_LOOKASIDE = 0,
    ALLOCATOR_PAGED_LOOKASIDE = 1,
    ALLOCATOR_CUSTOM = 2,
}

#[repr(C)]
pub enum ALLOC_REQUEST {
    ALLOCATOR_DEVICE_CONTROL = 0,
    ALLOCATOR_DEVICE_CLOSE = 1,
    ALLOCATOR_ALLOCATE = 2,
    ALLOCATOR_FREE = 3,
}

pub type PFNKSPAGEDPOOLALLOCATE = Option<unsafe extern "system" fn(Lookaside: PPAGED_LOOKASIDE_LIST) -> PVOID>;
pub type PFNKSNPAGEDPOOLALLOCATE = Option<unsafe extern "system" fn(Lookaside: PNPAGED_LOOKASIDE_LIST) -> PVOID>;
pub type PFNKSPAGEDPOOLFREE = Option<unsafe extern "system" fn(Lookaside: PPAGED_LOOKASIDE_LIST, Entry: PVOID)>;
pub type PFNKSNPAGEDPOOLFREE = Option<unsafe extern "system" fn(Lookaside: PNPAGED_LOOKASIDE_LIST, Entry: PVOID)>;
pub type PFNKSNPAGEDPOOLDELETE = Option<unsafe extern "system" fn(Lookaside: PNPAGED_LOOKASIDE_LIST)>;
pub type PFNKSPAGEDPOOLDELETE = Option<unsafe extern "system" fn(Lookaside: PPAGED_LOOKASIDE_LIST)>;

#[repr(C)]
pub struct ALLOCATOR {
    pub lpVtbl: *mut IKsAllocatorVtbl,
    pub ref: LONG,
    pub Header: PKSIOBJECT_HEADER,
    pub Type: ALLOCATOR_TYPE,
    pub Status: KSSTREAMALLOCATOR_STATUS,
    pub u: ALLOCATOR__bindgen_ty_1,
    pub Allocate: ALLOCATOR__bindgen_ty_2,
    pub Free: ALLOCATOR__bindgen_ty_3,
    pub Delete: ALLOCATOR__bindgen_ty_4,
}

#[repr(C)]
pub union ALLOCATOR__bindgen_ty_1 {
    pub NPagedList: NPAGED_LOOKASIDE_LIST,
    pub PagedList: PAGED_LOOKASIDE_LIST,
    pub CustomList: PVOID,
}

#[repr(C)]
pub union ALLOCATOR__bindgen_ty_2 {
    pub DefaultAllocate: PFNKSDEFAULTALLOCATE,
    pub PagedPool: PFNKSPAGEDPOOLALLOCATE,
    pub NPagedPool: PFNKSNPAGEDPOOLALLOCATE,
}

#[repr(C)]
pub union ALLOCATOR__bindgen_ty_3 {
    pub DefaultFree: PFNKSDEFAULTFREE,
    pub PagedPool: PFNKSPAGEDPOOLFREE,
    pub NPagedPool: PFNKSNPAGEDPOOLFREE,
}

#[repr(C)]
pub union ALLOCATOR__bindgen_ty_4 {
    pub DefaultDelete: PFNKSDELETEALLOCATOR,
    pub NPagedPool: PFNKSNPAGEDPOOLDELETE,
    pub PagedPool: PFNKSPAGEDPOOLDELETE,
}

pub type PALLOCATOR = *mut ALLOCATOR;

#[no_mangle]
pub unsafe extern "system" fn IKsAllocator_fnQueryInterface(
    iface: *mut IKsAllocator,
    refiid: *const GUID,
    Output: *mut PVOID,
) -> NTSTATUS {
    let This: PALLOCATOR = CONTAINING_RECORD(iface, ALLOCATOR, lpVtbl);

    if IsEqualGUIDAligned(refiid, &IID_IUnknown) ||
       IsEqualGUIDAligned(refiid, &IID_IKsAllocator)
    {
        *Output = &mut (*This).lpVtbl as *mut _ as PVOID;
        _InterlockedIncrement(&mut (*This).ref);
        return STATUS_SUCCESS;
    }
    return STATUS_UNSUCCESSFUL;
}

#[no_mangle]
pub unsafe extern "system" fn IKsAllocator_fnAddRef(iface: *mut IKsAllocator) -> ULONG {
    let This: PALLOCATOR = CONTAINING_RECORD(iface, ALLOCATOR, lpVtbl);
    InterlockedIncrement(&mut (*This).ref_0) as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn IKsAllocator_fnRelease(
    iface: *mut IKsAllocator,
) -> ULONG {
    let This: PALLOCATOR = CONTAINING_RECORD(iface, ALLOCATOR, lpVtbl);
    InterlockedDecrement(&mut (*This).ref);
    if (*This).ref == 0 {
        FreeItem(This as PVOID);
        return 0;
    }
    (*This).ref as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn IKsAllocator_fnDeviceIoControl(
    iface: *mut IKsAllocator,
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let This: PALLOCATOR = core::mem::transmute(
        (iface as usize)
            .wrapping_sub(core::mem::offset_of!(ALLOCATOR, lpVtbl))
    );
    let IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).Parameters.DeviceIoControl.IoControlCode != IOCTL_KS_PROPERTY {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
        CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        return STATUS_NOT_IMPLEMENTED;
    }

    if ((*IoStack).Parameters.DeviceIoControl.InputBufferLength as usize) < core::mem::size_of::<KSPROPERTY>() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;
        CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    let Property = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as PKSPROPERTY;

    if IsEqualGUIDAligned(&(*Property).Set, &KSPROPSETID_StreamAllocator) != 0 {
        if (*Property).Id == KSPROPERTY_STREAMALLOCATOR_FUNCTIONTABLE {
            if ((*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize) < core::mem::size_of::<KSSTREAMALLOCATOR_FUNCTIONTABLE>() {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
                (*Irp).IoStatus.Information = core::mem::size_of::<KSSTREAMALLOCATOR_FUNCTIONTABLE>() as ULONG_PTR;
                CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
                return STATUS_BUFFER_TOO_SMALL;
            }
            if ((*Property).Flags & KSPROPERTY_TYPE_GET) == 0 {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
                CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
                return STATUS_UNSUCCESSFUL;
            }

            let FunctionTable = (*Irp).UserBuffer as PKSSTREAMALLOCATOR_FUNCTIONTABLE;
            (*FunctionTable).AllocateFrame = Some(IKsAllocator_Allocate as PFNALLOCATOR_ALLOCATEFRAME);
            (*FunctionTable).FreeFrame = Some(IKsAllocator_FreeFrame as PFNALLOCATOR_FREEFRAME);

            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            (*Irp).IoStatus.Information = core::mem::size_of::<KSSTREAMALLOCATOR_FUNCTIONTABLE>() as ULONG_PTR;
            CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
            return STATUS_SUCCESS;
        } else if (*Property).Id == KSPROPERTY_STREAMALLOCATOR_STATUS {
            if ((*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize) < core::mem::size_of::<KSSTREAMALLOCATOR_STATUS>() {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
                (*Irp).IoStatus.Information = core::mem::size_of::<KSSTREAMALLOCATOR_STATUS>() as ULONG_PTR;
                CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
                return STATUS_BUFFER_TOO_SMALL;
            }
            if ((*Property).Flags & KSPROPERTY_TYPE_GET) == 0 {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
                CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
                return STATUS_UNSUCCESSFUL;
            }

            let State = (*Irp).UserBuffer as PKSSTREAMALLOCATOR_STATUS;
            core::ptr::copy_nonoverlapping(
                &raw const (*This).__bindgen_anon_1.Status as *const u8,
                State as *mut u8,
                core::mem::size_of::<KSSTREAMALLOCATOR_STATUS>(),
            );

            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            (*Irp).IoStatus.Information = core::mem::size_of::<KSSTREAMALLOCATOR_STATUS>() as ULONG_PTR;
            CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
            return STATUS_SUCCESS;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsAllocator_fnClose(
    iface: *mut IKsAllocator,
) -> NTSTATUS {
    let This: PALLOCATOR = CONTAINING_RECORD(iface, ALLOCATOR, lpVtbl);

    if (*This).Type == ALLOCATOR_CUSTOM {
        let delete = (*This).Delete.DefaultDelete;
        delete((*This).u.CustomList);
    } else if (*This).Type == ALLOCATOR_NPAGED_LOOKASIDE {
        let delete = (*This).Delete.NPagedPool;
        delete(&mut (*This).u.NPagedList);
    } else if (*This).Type == ALLOCATOR_PAGED_LOOKASIDE {
        let delete = (*This).Delete.PagedPool;
        delete(&mut (*This).u.PagedList);
    }

    KsFreeObjectHeader(&mut (*This).Header as *mut _ as PVOID);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IKsAllocator_fnAllocateFrame(
    iface: *mut IKsAllocator,
    OutFrame: *mut PVOID,
) -> NTSTATUS {
    let mut Frame: PVOID = core::ptr::null_mut();
    let This: PALLOCATOR = core::mem::transmute(
        (iface as usize)
            .wrapping_sub(core::mem::size_of::<IKsAllocatorVtbl>()),
    );

    if (*This).Type == ALLOCATOR_CUSTOM {
        Frame = (*This)
            .Allocate
            .DefaultAllocate
            .expect("non-null function pointer")((*This).u.CustomList);
    } else if (*This).Type == ALLOCATOR_NPAGED_LOOKASIDE {
        Frame = (*This)
            .Allocate
            .NPagedPool
            .expect("non-null function pointer")(&mut (*This).u.NPagedList);
    } else if (*This).Type == ALLOCATOR_PAGED_LOOKASIDE {
        Frame = (*This)
            .Allocate
            .PagedPool
            .expect("non-null function pointer")(&mut (*This).u.PagedList);
    }

    if !Frame.is_null() {
        *OutFrame = Frame;
        InterlockedIncrement(&mut (*This).__bindgen_anon_1.Status.AllocatedFrames as *mut LONG);
        return STATUS_SUCCESS;
    }

    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn IKsAllocator_fnFreeFrame(
    iface: *mut IKsAllocator,
    Frame: PVOID,
) {
    let This: PALLOCATOR = CONTAINING_RECORD(iface, ALLOCATOR, lpVtbl);

    if (*This).Type == ALLOCATOR_CUSTOM {
        let free = (*This).Free.DefaultFree;
        free((*This).u.CustomList, Frame);
    } else if (*This).Type == ALLOCATOR_NPAGED_LOOKASIDE {
        let free = (*This).Free.NPagedPool;
        free(&mut (*This).u.NPagedList, Frame);
    } else if (*This).Type == ALLOCATOR_PAGED_LOOKASIDE {
        let free = (*This).Free.PagedPool;
        free(&mut (*This).u.PagedList, Frame);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KsCreateAllocator(
    ConnectionHandle: HANDLE,
    AllocatorFraming: PKSALLOCATOR_FRAMING,
    AllocatorHandle: PHANDLE,
) -> NTSTATUS {
    return KspCreateObjectType(
        ConnectionHandle,
        KSSTRING_Allocator.as_ptr() as LPWSTR,
        AllocatorFraming as PVOID,
        core::mem::size_of::<KSALLOCATOR_FRAMING>() as UINT,
        GENERIC_READ,
        AllocatorHandle,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsCreateDefaultAllocator(
    Irp: PIRP,
) -> NTSTATUS {
    return KsCreateDefaultAllocatorEx(
        Irp,
        core::ptr::null_mut(),
        None,
        None,
        None,
        None,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsValidateAllocatorCreateRequest(
    Irp: PIRP,
    OutAllocatorFraming: *mut PKSALLOCATOR_FRAMING,
) -> NTSTATUS {
    let mut AllocatorFraming: PKSALLOCATOR_FRAMING = core::ptr::null_mut();
    let mut Size: ULONG = core::mem::size_of::<KSALLOCATOR_FRAMING>() as ULONG;
    let mut Status: NTSTATUS;
    let SupportedFlags: ULONG;

    Status = KspCopyCreateRequest(
        Irp,
        KSSTRING_Allocator.as_ptr() as LPWSTR,
        &mut Size as *mut ULONG,
        &mut AllocatorFraming as *mut PKSALLOCATOR_FRAMING as *mut PVOID,
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    SupportedFlags = (KSALLOCATOR_OPTIONF_COMPATIBLE
        | KSALLOCATOR_OPTIONF_SYSTEM_MEMORY
        | KSALLOCATOR_REQUIREMENTF_INPLACE_MODIFIER
        | KSALLOCATOR_REQUIREMENTF_SYSTEM_MEMORY
        | KSALLOCATOR_REQUIREMENTF_FRAME_INTEGRITY
        | KSALLOCATOR_REQUIREMENTF_MUST_ALLOCATE) as ULONG;

    if (*AllocatorFraming).FrameSize == 0
        || ((*AllocatorFraming).__bindgen_anon_1.OptionsFlags & !SupportedFlags) != 0
    {
        FreeItem(AllocatorFraming as PVOID);
        return STATUS_INVALID_PARAMETER;
    }

    *OutAllocatorFraming = AllocatorFraming;

    Status
}
#[no_mangle]
pub unsafe extern "C" fn IKsAllocator_DispatchRequest(
    DeviceObject: PDEVICE_OBJECT,
    FileObject: PFILE_OBJECT,
    Irp: PIRP,
    Frame: PVOID,
    Request: ALLOC_REQUEST,
) -> NTSTATUS {
    let mut Header: PKSIOBJECT_HEADER;
    let mut Status: NTSTATUS;
    let mut Allocator: *mut IKsAllocator;

    Header = (*FileObject).FsContext2 as PKSIOBJECT_HEADER;

    Status = (*(*(*Header).Unknown).lpVtbl).QueryInterface.unwrap()(
        (*Header).Unknown,
        &IID_IKsAllocator as *const GUID,
        &mut Allocator as *mut *mut IKsAllocator as *mut PVOID,
    );

    if !NT_SUCCESS(Status) {
        return STATUS_UNSUCCESSFUL;
    }

    if Request == ALLOCATOR_DEVICE_CONTROL {
        Status = (*(*Allocator).lpVtbl).DispatchDeviceIoControl.unwrap()(
            Allocator,
            DeviceObject,
            Irp,
        );
    } else if Request == ALLOCATOR_DEVICE_CLOSE {
        Status = (*(*Allocator).lpVtbl).Close.unwrap()(Allocator);
    } else if Request == ALLOCATOR_ALLOCATE {
        Status = (*(*Allocator).lpVtbl).AllocateFrame.unwrap()(
            Allocator,
            Frame as *mut PVOID,
        );
    } else if Request == ALLOCATOR_FREE {
        (*(*Allocator).lpVtbl).FreeFrame.unwrap()(Allocator, Frame);
        Status = STATUS_SUCCESS;
    }

    (*(*Allocator).lpVtbl).Release.unwrap()(Allocator);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsAllocator_DispatchDeviceIoControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let Status: NTSTATUS = IKsAllocator_DispatchRequest(
        DeviceObject,
        (*IoStack).FileObject,
        Irp,
        core::ptr::null_mut(),
        ALLOCATOR_DEVICE_CONTROL,
    );
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    CompleteRequest(Irp, IO_NO_INCREMENT as i8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsAllocator_DispatchClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let Status: NTSTATUS = IKsAllocator_DispatchRequest(
        DeviceObject,
        (*IoStack).FileObject,
        Irp,
        core::ptr::null_mut(),
        ALLOCATOR_DEVICE_CLOSE,
    );
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsAllocator_Allocate(
    FileObject: PFILE_OBJECT,
    Frame: *mut PVOID,
) -> NTSTATUS {
    let Status: NTSTATUS;
    Status = IKsAllocator_DispatchRequest(
        core::ptr::null_mut(),
        FileObject,
        core::ptr::null_mut(),
        Frame as PVOID,
        ALLOCATOR_ALLOCATE,
    );
    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn IKsAllocator_FreeFrame(
    FileObject: PFILE_OBJECT,
    Frame: PVOID,
) {
    IKsAllocator_DispatchRequest(
        core::ptr::null_mut(),
        FileObject,
        core::ptr::null_mut(),
        Frame,
        ALLOCATOR_FREE,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsCreateDefaultAllocatorEx(
    Irp: PIRP,
    InitializeContext: PVOID,
    DefaultAllocate: PFNKSDEFAULTALLOCATE,
    DefaultFree: PFNKSDEFAULTFREE,
    InitializeAllocator: PFNKSINITIALIZEALLOCATOR,
    DeleteAllocator: PFNKSDELETEALLOCATOR,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut AllocatorFraming: PKSALLOCATOR_FRAMING;
    let mut Allocator: PALLOCATOR;
    let mut Ctx: PVOID;

    Status = KsValidateAllocatorCreateRequest(Irp, &mut AllocatorFraming);
    if !NT_SUCCESS(Status) {
        return STATUS_INVALID_PARAMETER;
    }

    if (*AllocatorFraming).FileAlignment > (PAGE_SIZE - 1) as u32 {
        FreeItem(AllocatorFraming as PVOID);
        return STATUS_INVALID_PARAMETER;
    }

    Allocator = AllocateItem(NonPagedPool, core::mem::size_of::<ALLOCATOR>()) as PALLOCATOR;
    if Allocator.is_null() {
        FreeItem(AllocatorFraming as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = KsAllocateObjectHeader(
        &mut (*Allocator).Header as *mut _ as *mut KSOBJECT_HEADER,
        0,
        core::ptr::null_mut(),
        Irp,
        &DispatchTable as *const _ as PVOID,
    );
    if !NT_SUCCESS(Status) {
        FreeItem(AllocatorFraming as PVOID);
        FreeItem(Allocator as PVOID);
        return Status;
    }

    (*Allocator).lpVtbl = &vt_IKsAllocator;
    (*(*Allocator).Header).Unknown = &mut (*Allocator).lpVtbl as *mut _ as PUNKNOWN;
    (*Allocator).ref = 1;

    if !DefaultAllocate.is_null() {
        (*Allocator).Type = ALLOCATOR_CUSTOM;
        (*Allocator).Allocate.DefaultAllocate = DefaultAllocate;
        (*Allocator).Free.DefaultFree = DefaultFree;
        (*Allocator).Delete.DefaultDelete = DeleteAllocator;
        Ctx = InitializeAllocator.unwrap()(
            InitializeContext,
            AllocatorFraming,
            &mut (*Allocator).u.CustomList,
        );
        if Ctx.is_null() {
            KsFreeObjectHeader((*Allocator).Header as PVOID);
            FreeItem(Allocator as PVOID);
            return Status;
        }
    } else if (*AllocatorFraming).PoolType as u32 == NonPagedPool as u32 {
        (*Allocator).Type = ALLOCATOR_NPAGED_LOOKASIDE;
        (*Allocator).Allocate.NPagedPool = Some(
            ExAllocateFromNPagedLookasideList
                as unsafe extern "system" fn(PNPAGED_LOOKASIDE_LIST) -> PVOID,
        ) as PFNKSNPAGEDPOOLALLOCATE;
        (*Allocator).Free.NPagedPool = Some(
            ExFreeToNPagedLookasideList
                as unsafe extern "system" fn(PNPAGED_LOOKASIDE_LIST, PVOID),
        ) as PFNKSNPAGEDPOOLFREE;
        (*Allocator).Delete.NPagedPool = Some(
            ExDeleteNPagedLookasideList
                as unsafe extern "system" fn(PNPAGED_LOOKASIDE_LIST),
        ) as PFNKSNPAGEDPOOLDELETE;
        ExInitializeNPagedLookasideList(
            &mut (*Allocator).u.NPagedList,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            (*AllocatorFraming).FrameSize,
            0,
            0,
        );
    } else if (*AllocatorFraming).PoolType as u32 == PagedPool as u32 {
        (*Allocator).Type = ALLOCATOR_PAGED_LOOKASIDE;
        (*Allocator).Allocate.PagedPool = Some(
            ExAllocateFromPagedLookasideList
                as unsafe extern "system" fn(PPAGED_LOOKASIDE_LIST) -> PVOID,
        ) as PFNKSPAGEDPOOLALLOCATE;
        (*Allocator).Free.PagedPool = Some(
            ExFreeToPagedLookasideList
                as unsafe extern "system" fn(PPAGED_LOOKASIDE_LIST, PVOID),
        ) as PFNKSPAGEDPOOLFREE;
        (*Allocator).Delete.PagedPool = Some(
            ExDeletePagedLookasideList
                as unsafe extern "system" fn(PPAGED_LOOKASIDE_LIST),
        ) as PFNKSPAGEDPOOLDELETE;
        ExInitializePagedLookasideList(
            &mut (*Allocator).u.PagedList,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            (*AllocatorFraming).FrameSize,
            0,
            0,
        );
    }

    core::ptr::copy_nonoverlapping(
        AllocatorFraming as *const u8,
        &mut (*Allocator).__bindgen_anon_1.Status.Framing as *mut _ as *mut u8,
        core::mem::size_of::<KSALLOCATOR_FRAMING>(),
    );
    FreeItem(AllocatorFraming as PVOID);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsValidateAllocatorFramingEx(
    Framing: PKSALLOCATOR_FRAMING_EX,
    BufferSize: ULONG,
    PinFraming: *const KSALLOCATOR_FRAMING_EX,
) -> NTSTATUS {
    if (BufferSize as usize) < core::mem::size_of::<KSALLOCATOR_FRAMING_EX>() {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    if ((*Framing).FramingItem[0].Flags as ULONG & KSALLOCATOR_FLAG_PARTIAL_READ_SUPPORT as ULONG) != 0
        && (*Framing).OutputCompression.RatioNumerator != MAXULONG
        && (*Framing).OutputCompression.RatioDenominator != 0
        && (*Framing).OutputCompression.RatioDenominator < (*Framing).OutputCompression.RatioNumerator
    {
        return STATUS_SUCCESS;
    }

    STATUS_INVALID_DEVICE_REQUEST
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/allocators.c
//  * PURPOSE:         KS Allocator functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// typedef enum
// {
//     ALLOCATOR_NPAGED_LOOKASIDE,
//     ALLOCATOR_PAGED_LOOKASIDE,
//     ALLOCATOR_CUSTOM
// }ALLOCATOR_TYPE;
//
// typedef enum
// {
//     ALLOCATOR_DEVICE_CONTROL,
//     ALLOCATOR_DEVICE_CLOSE,
//     ALLOCATOR_ALLOCATE,
//     ALLOCATOR_FREE
//
// }ALLOC_REQUEST;
//
// typedef PVOID (*PFNKSPAGEDPOOLALLOCATE)(IN PPAGED_LOOKASIDE_LIST  Lookaside);
// typedef PVOID (*PFNKSNPAGEDPOOLALLOCATE)(IN PNPAGED_LOOKASIDE_LIST  Lookaside);
//
// typedef VOID (*PFNKSPAGEDPOOLFREE)(IN PPAGED_LOOKASIDE_LIST  Lookaside, IN PVOID  Entry);
// typedef VOID (*PFNKSNPAGEDPOOLFREE)(IN PNPAGED_LOOKASIDE_LIST  Lookaside, IN PVOID  Entry);
//
// typedef VOID (NTAPI *PFNKSNPAGEDPOOLDELETE)(IN PNPAGED_LOOKASIDE_LIST  Lookaside);
// typedef VOID (NTAPI *PFNKSPAGEDPOOLDELETE)(IN PPAGED_LOOKASIDE_LIST  Lookaside);
//
// typedef struct
// {
//     IKsAllocatorVtbl *lpVtbl;
//     LONG ref;
//     PKSIOBJECT_HEADER Header;
//     ALLOCATOR_TYPE Type;
//
//     KSSTREAMALLOCATOR_STATUS Status;
//
//     union
//     {
//         NPAGED_LOOKASIDE_LIST NPagedList;
//         PAGED_LOOKASIDE_LIST PagedList;
//         PVOID CustomList;
//     }u;
//
//     union
//     {
//          PFNKSDEFAULTALLOCATE DefaultAllocate;
//          PFNKSPAGEDPOOLALLOCATE PagedPool;
//          PFNKSNPAGEDPOOLALLOCATE NPagedPool;
//     }Allocate;
//
//    union
//     {
//          PFNKSDEFAULTFREE DefaultFree;
//          PFNKSPAGEDPOOLFREE PagedPool;
//          PFNKSNPAGEDPOOLFREE NPagedPool;
//     }Free;
//
//     union
//     {
//         PFNKSDELETEALLOCATOR DefaultDelete;
//         PFNKSNPAGEDPOOLDELETE NPagedPool;
//         PFNKSPAGEDPOOLDELETE PagedPool;
//     }Delete;
//
// }ALLOCATOR, *PALLOCATOR;
//
// /* use KSNAME_Allocator for IID_IKsAllocator */
// const GUID IID_IKsAllocator =            {0x642F5D00L, 0x4791, 0x11D0, {0xA5, 0xD6, 0x28, 0xDB, 0x04, 0xC1, 0x00, 0x00}};
// const GUID KSPROPSETID_StreamAllocator = {0x0cf6e4342, 0xec87, 0x11cf, {0xa1, 0x30, 0x00, 0x20, 0xaf, 0xd1, 0x56, 0xe4}};
//
//
// NTSTATUS
// NTAPI
// IKsAllocator_Allocate(
//     IN PFILE_OBJECT FileObject,
//     PVOID *Frame);
//
// VOID
// NTAPI
// IKsAllocator_FreeFrame(
//     IN PFILE_OBJECT FileObject,
//     PVOID Frame);
//
//
// NTSTATUS
// NTAPI
// IKsAllocator_fnQueryInterface(
//     IKsAllocator * iface,
//     IN  REFIID refiid,
//     OUT PVOID* Output)
// {
//     PALLOCATOR This = (PALLOCATOR)CONTAINING_RECORD(iface, ALLOCATOR, lpVtbl);
//
//     if (IsEqualGUIDAligned(refiid, &IID_IUnknown) ||
//         IsEqualGUIDAligned(refiid, &IID_IKsAllocator))
//     {
//         *Output = &This->lpVtbl;
//         _InterlockedIncrement(&This->ref);
//         return STATUS_SUCCESS;
//     }
//     return STATUS_UNSUCCESSFUL;
// }
//
// ULONG
// NTAPI
// IKsAllocator_fnAddRef(
//     IKsAllocator * iface)
// {
//     PALLOCATOR This = (PALLOCATOR)CONTAINING_RECORD(iface, ALLOCATOR, lpVtbl);
//
//     return InterlockedIncrement(&This->ref);
// }
//
// ULONG
// NTAPI
// IKsAllocator_fnRelease(
//     IKsAllocator * iface)
// {
//     PALLOCATOR This = (PALLOCATOR)CONTAINING_RECORD(iface, ALLOCATOR, lpVtbl);
//
//     InterlockedDecrement(&This->ref);
//
//     if (This->ref == 0)
//     {
//         FreeItem(This);
//         return 0;
//     }
//     /* Return new reference count */
//     return This->ref;
// }
//
// NTSTATUS
// NTAPI
// IKsAllocator_fnDeviceIoControl(
//     IKsAllocator *iface,
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PALLOCATOR This = (PALLOCATOR)CONTAINING_RECORD(iface, ALLOCATOR, lpVtbl);
//     PIO_STACK_LOCATION IoStack;
//     PKSSTREAMALLOCATOR_FUNCTIONTABLE FunctionTable;
//     PKSSTREAMALLOCATOR_STATUS State;
//     PKSPROPERTY Property;
//
//     /* FIXME locks */
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (IoStack->Parameters.DeviceIoControl.IoControlCode  != IOCTL_KS_PROPERTY)
//     {
//         /* only KSPROPERTY requests are supported */
//         UNIMPLEMENTED;
//
//         /* complete and forget irps */
//         Irp->IoStatus.Status = STATUS_NOT_IMPLEMENTED;
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//
//         return STATUS_NOT_IMPLEMENTED;
//    }
//
//     if (IoStack->Parameters.DeviceIoControl.InputBufferLength < sizeof(KSPROPERTY))
//     {
//         /* invalid request */
//         Irp->IoStatus.Status = STATUS_INVALID_DEVICE_REQUEST;
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     /* check the request */
//     Property = (PKSPROPERTY)IoStack->Parameters.DeviceIoControl.Type3InputBuffer;
//
//     if (IsEqualGUIDAligned(&Property->Set, &KSPROPSETID_StreamAllocator))
//     {
//         if (Property->Id == KSPROPERTY_STREAMALLOCATOR_FUNCTIONTABLE)
//         {
//             if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(KSSTREAMALLOCATOR_FUNCTIONTABLE))
//             {
//                 /* buffer too small */
//                 Irp->IoStatus.Status = STATUS_BUFFER_TOO_SMALL;
//                 Irp->IoStatus.Information = sizeof(KSSTREAMALLOCATOR_FUNCTIONTABLE);
//                 /* complete and forget irp */
//                 CompleteRequest(Irp, IO_NO_INCREMENT);
//                 return STATUS_BUFFER_TOO_SMALL;
//             }
//             if (!(Property->Flags & KSPROPERTY_TYPE_GET))
//             {
//                 /* only support retrieving the property */
//                 Irp->IoStatus.Status = STATUS_UNSUCCESSFUL;
//                 /* complete and forget irp */
//                 CompleteRequest(Irp, IO_NO_INCREMENT);
//                 return STATUS_UNSUCCESSFUL;
//             }
//
//             /* get output buffer */
//             FunctionTable = (PKSSTREAMALLOCATOR_FUNCTIONTABLE)Irp->UserBuffer;
//
//             FunctionTable->AllocateFrame = IKsAllocator_Allocate;
//             FunctionTable->FreeFrame = IKsAllocator_FreeFrame;
//
//             /* save result */
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             Irp->IoStatus.Information = sizeof(KSSTREAMALLOCATOR_FUNCTIONTABLE);
//             /* complete request */
//             CompleteRequest(Irp, IO_NO_INCREMENT);
//             return STATUS_SUCCESS;
//         }
//         else if (Property->Id == KSPROPERTY_STREAMALLOCATOR_STATUS)
//         {
//             if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < sizeof(KSPROPERTY_STREAMALLOCATOR_STATUS))
//             {
//                 /* buffer too small */
//                 Irp->IoStatus.Status = STATUS_BUFFER_TOO_SMALL;
//                 Irp->IoStatus.Information = sizeof(KSPROPERTY_STREAMALLOCATOR_STATUS);
//                 /* complete and forget irp */
//                 CompleteRequest(Irp, IO_NO_INCREMENT);
//                 return STATUS_BUFFER_TOO_SMALL;
//             }
//             if (!(Property->Flags & KSPROPERTY_TYPE_GET))
//             {
//                 /* only support retrieving the property */
//                 Irp->IoStatus.Status = STATUS_UNSUCCESSFUL;
//                 /* complete and forget irp */
//                 CompleteRequest(Irp, IO_NO_INCREMENT);
//                 return STATUS_UNSUCCESSFUL;
//             }
//
//             /* get output buffer */
//             State = (PKSSTREAMALLOCATOR_STATUS)Irp->UserBuffer;
//
//             /* copy allocator status */
//             RtlMoveMemory(State, &This->Status, sizeof(KSSTREAMALLOCATOR_STATUS));
//
//             /* save result */
//             Irp->IoStatus.Status = STATUS_SUCCESS;
//             Irp->IoStatus.Information = sizeof(KSSTREAMALLOCATOR_STATUS);
//
//             /* complete request */
//             CompleteRequest(Irp, IO_NO_INCREMENT);
//             return STATUS_SUCCESS;
//         }
//     }
//
//     /* unhandled request */
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return STATUS_NOT_SUPPORTED;
// }
//
// NTSTATUS
// NTAPI
// IKsAllocator_fnClose(
//     IKsAllocator *iface)
// {
//     PALLOCATOR This = (PALLOCATOR)CONTAINING_RECORD(iface, ALLOCATOR, lpVtbl);
//
//     /* FIXME locks */
//
//     /* now close allocator */
//     if (This->Type == ALLOCATOR_CUSTOM)
//     {
//         This->Delete.DefaultDelete(This->u.CustomList);
//     }
//     else if (This->Type == ALLOCATOR_NPAGED_LOOKASIDE)
//     {
//         This->Delete.NPagedPool(&This->u.NPagedList);
//     }
//     else if (This->Type == ALLOCATOR_PAGED_LOOKASIDE)
//     {
//         This->Delete.PagedPool(&This->u.PagedList);
//     }
//
//     /* free object header */
//     KsFreeObjectHeader(&This->Header);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// IKsAllocator_fnAllocateFrame(
//     IKsAllocator *iface,
//     IN PVOID * OutFrame)
// {
//     PVOID Frame = NULL;
//     PALLOCATOR This = (PALLOCATOR)CONTAINING_RECORD(iface, ALLOCATOR, lpVtbl);
//
//     /* FIXME locks */
//
//     /* now allocate frame */
//     if (This->Type == ALLOCATOR_CUSTOM)
//     {
//         Frame = This->Allocate.DefaultAllocate(This->u.CustomList);
//     }
//     else if (This->Type == ALLOCATOR_NPAGED_LOOKASIDE)
//     {
//         Frame = This->Allocate.NPagedPool(&This->u.NPagedList);
//     }
//     else if (This->Type == ALLOCATOR_PAGED_LOOKASIDE)
//     {
//         Frame = This->Allocate.PagedPool(&This->u.PagedList);
//     }
//
//     if (Frame)
//     {
//         *OutFrame = Frame;
//         InterlockedIncrement((PLONG)&This->Status.AllocatedFrames);
//         return STATUS_SUCCESS;
//     }
//
//     return STATUS_UNSUCCESSFUL;
// }
//
// VOID
// NTAPI
// IKsAllocator_fnFreeFrame(
//     IKsAllocator *iface,
//     IN PVOID Frame)
// {
//     PALLOCATOR This = (PALLOCATOR)CONTAINING_RECORD(iface, ALLOCATOR, lpVtbl);
//
//     /* now allocate frame */
//     if (This->Type == ALLOCATOR_CUSTOM)
//     {
//         This->Free.DefaultFree(This->u.CustomList, Frame);
//     }
//     else if (This->Type == ALLOCATOR_NPAGED_LOOKASIDE)
//     {
//         This->Free.NPagedPool(&This->u.NPagedList, Frame);
//     }
//     else if (This->Type == ALLOCATOR_PAGED_LOOKASIDE)
//     {
//         This->Free.PagedPool(&This->u.PagedList, Frame);
//     }
// }
//
//
// static IKsAllocatorVtbl vt_IKsAllocator =
// {
//     IKsAllocator_fnQueryInterface,
//     IKsAllocator_fnAddRef,
//     IKsAllocator_fnRelease,
//     IKsAllocator_fnDeviceIoControl,
//     IKsAllocator_fnClose,
//     IKsAllocator_fnAllocateFrame,
//     IKsAllocator_fnFreeFrame
// };
//
//
// /*
//     @implemented
// */
// KSDDKAPI NTSTATUS NTAPI
// KsCreateAllocator(
//     IN  HANDLE ConnectionHandle,
//     IN  PKSALLOCATOR_FRAMING AllocatorFraming,
//     OUT PHANDLE AllocatorHandle)
// {
//     return KspCreateObjectType(ConnectionHandle,
//                                KSSTRING_Allocator,
//                                (PVOID)AllocatorFraming,
//                                sizeof(KSALLOCATOR_FRAMING),
//                                GENERIC_READ,
//                                AllocatorHandle);
// }
//
// /*
//     @implemented
// */
// KSDDKAPI NTSTATUS NTAPI
// KsCreateDefaultAllocator(
//     IN  PIRP Irp)
// {
//     return KsCreateDefaultAllocatorEx(Irp, NULL, NULL, NULL, NULL, NULL);
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsValidateAllocatorCreateRequest(
//     IN  PIRP Irp,
//     OUT PKSALLOCATOR_FRAMING* OutAllocatorFraming)
// {
//     PKSALLOCATOR_FRAMING AllocatorFraming;
//     ULONG Size;
//     NTSTATUS Status;
//     ULONG SupportedFlags;
//
//     /* set minimum request size */
//     Size = sizeof(KSALLOCATOR_FRAMING);
//
//     Status = KspCopyCreateRequest(Irp,
//                                   KSSTRING_Allocator,
//                                   &Size,
//                                   (PVOID*)&AllocatorFraming);
//
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* allowed supported flags */
//     SupportedFlags = (KSALLOCATOR_OPTIONF_COMPATIBLE | KSALLOCATOR_OPTIONF_SYSTEM_MEMORY |
//                       KSALLOCATOR_REQUIREMENTF_INPLACE_MODIFIER | KSALLOCATOR_REQUIREMENTF_SYSTEM_MEMORY | KSALLOCATOR_REQUIREMENTF_FRAME_INTEGRITY |
//                       KSALLOCATOR_REQUIREMENTF_MUST_ALLOCATE);
//
//
//     if (!AllocatorFraming->FrameSize || (AllocatorFraming->OptionsFlags & (~SupportedFlags)))
//     {
//         FreeItem(AllocatorFraming);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* store result */
//     *OutAllocatorFraming = AllocatorFraming;
//
//     return Status;
// }
//
// NTSTATUS
// IKsAllocator_DispatchRequest(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PFILE_OBJECT FileObject,
//     IN PIRP Irp,
//     IN PVOID Frame,
//     IN ALLOC_REQUEST Request)
// {
//     PKSIOBJECT_HEADER Header;
//     NTSTATUS Status;
//     IKsAllocator * Allocator;
//
//     /* sanity check */
//     ASSERT(FileObject);
//
//     /* get object header */
//     Header = (PKSIOBJECT_HEADER)FileObject->FsContext2;
//
//     /* get real allocator */
//     Status = Header->Unknown->lpVtbl->QueryInterface(Header->Unknown, &IID_IKsAllocator, (PVOID*)&Allocator);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* misbehaving object */
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     if (Request == ALLOCATOR_DEVICE_CONTROL)
//     {
//         /* dispatch request allocator */
//         Status = Allocator->lpVtbl->DispatchDeviceIoControl(Allocator, DeviceObject, Irp);
//     }
//     else if (Request == ALLOCATOR_DEVICE_CLOSE)
//     {
//         /* delete allocator */
//         Status = Allocator->lpVtbl->Close(Allocator);
//     }
//     else if (Request == ALLOCATOR_ALLOCATE)
//     {
//         /* allocate frame */
//         Status = Allocator->lpVtbl->AllocateFrame(Allocator, (PVOID*)Frame);
//
//     }else if (Request == ALLOCATOR_FREE)
//     {
//         /* allocate frame */
//         Allocator->lpVtbl->FreeFrame(Allocator, Frame);
//         Status = STATUS_SUCCESS;
//     }
//
//     /* release interface */
//     Allocator->lpVtbl->Release(Allocator);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsAllocator_DispatchDeviceIoControl(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* dispatch request */
//     Status = IKsAllocator_DispatchRequest(DeviceObject, IoStack->FileObject, Irp, NULL, ALLOCATOR_DEVICE_CONTROL);
//
//     /* complete request */
//     Irp->IoStatus.Status = Status;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsAllocator_DispatchClose(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* dispatch request */
//     Status = IKsAllocator_DispatchRequest(DeviceObject, IoStack->FileObject, Irp, NULL, ALLOCATOR_DEVICE_CLOSE);
//
//     /* complete request */
//     Irp->IoStatus.Status = Status;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsAllocator_Allocate(
//     IN PFILE_OBJECT FileObject,
//     PVOID *Frame)
// {
//     NTSTATUS Status;
//
//     /* dispatch request */
//     Status = IKsAllocator_DispatchRequest(NULL, FileObject, NULL, (PVOID)Frame, ALLOCATOR_ALLOCATE);
//
//     return Status;
// }
//
// VOID
// NTAPI
// IKsAllocator_FreeFrame(
//     IN PFILE_OBJECT FileObject,
//     PVOID Frame)
// {
//     /* dispatch request */
//     IKsAllocator_DispatchRequest(NULL, FileObject, NULL, Frame, ALLOCATOR_FREE);
// }
//
//
// static KSDISPATCH_TABLE DispatchTable =
// {
//     IKsAllocator_DispatchDeviceIoControl,
//     KsDispatchInvalidDeviceRequest,
//     KsDispatchInvalidDeviceRequest,
//     KsDispatchInvalidDeviceRequest,
//     IKsAllocator_DispatchClose,
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
// KSDDKAPI NTSTATUS NTAPI
// KsCreateDefaultAllocatorEx(
//     IN  PIRP Irp,
//     IN  PVOID InitializeContext OPTIONAL,
//     IN  PFNKSDEFAULTALLOCATE DefaultAllocate OPTIONAL,
//     IN  PFNKSDEFAULTFREE DefaultFree OPTIONAL,
//     IN  PFNKSINITIALIZEALLOCATOR InitializeAllocator OPTIONAL,
//     IN  PFNKSDELETEALLOCATOR DeleteAllocator OPTIONAL)
// {
//     NTSTATUS Status;
//     PKSALLOCATOR_FRAMING AllocatorFraming;
//     PALLOCATOR Allocator;
//     PVOID Ctx;
//
//     /* first validate connect request */
//     Status = KsValidateAllocatorCreateRequest(Irp, &AllocatorFraming);
//     if (!NT_SUCCESS(Status))
//         return STATUS_INVALID_PARAMETER;
//
//     /* check the valid file alignment */
//     if (AllocatorFraming->FileAlignment > (PAGE_SIZE-1))
//     {
//         FreeItem(AllocatorFraming);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* allocate allocator struct */
//     Allocator = AllocateItem(NonPagedPool, sizeof(ALLOCATOR));
//     if (!Allocator)
//     {
//         FreeItem(AllocatorFraming);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* allocate object header */
//
//     Status = KsAllocateObjectHeader((KSOBJECT_HEADER*)&Allocator->Header, 0, NULL, Irp, &DispatchTable);
//     if (!NT_SUCCESS(Status))
//     {
//         FreeItem(AllocatorFraming);
//         FreeItem(Allocator);
//         return Status;
//     }
//
//     /* set allocator type in object header */
//     Allocator->lpVtbl = &vt_IKsAllocator;
//     Allocator->Header->Unknown = (PUNKNOWN)&Allocator->lpVtbl;
//     Allocator->ref = 1;
//
//     if (DefaultAllocate)
//     {
//         /* use external allocator */
//         Allocator->Type  = ALLOCATOR_CUSTOM;
//         Allocator->Allocate.DefaultAllocate = DefaultAllocate;
//         Allocator->Free.DefaultFree = DefaultFree;
//         Allocator->Delete.DefaultDelete = DeleteAllocator;
//         Ctx = InitializeAllocator(InitializeContext, AllocatorFraming, &Allocator->u.CustomList);
//         /* check for success */
//         if (!Ctx)
//         {
//             KsFreeObjectHeader(Allocator->Header);
//             FreeItem(Allocator);
//             return Status;
//         }
//     }
//     else if (AllocatorFraming->PoolType == NonPagedPool)
//     {
//         /* use non-paged pool allocator */
//         Allocator->Type  = ALLOCATOR_NPAGED_LOOKASIDE;
//         Allocator->Allocate.NPagedPool = ExAllocateFromNPagedLookasideList;
//         Allocator->Free.NPagedPool = ExFreeToNPagedLookasideList;
//         Allocator->Delete.NPagedPool = ExDeleteNPagedLookasideList;
//         ExInitializeNPagedLookasideList(&Allocator->u.NPagedList, NULL, NULL, 0, AllocatorFraming->FrameSize, 0, 0);
//     }
//     else if (AllocatorFraming->PoolType == PagedPool)
//     {
//         /* use paged pool allocator */
//         Allocator->Allocate.PagedPool = ExAllocateFromPagedLookasideList;
//         Allocator->Free.PagedPool = ExFreeToPagedLookasideList;
//         Allocator->Delete.PagedPool = ExDeletePagedLookasideList;
//         Allocator->Type  = ALLOCATOR_PAGED_LOOKASIDE;
//         ExInitializePagedLookasideList(&Allocator->u.PagedList, NULL, NULL, 0, AllocatorFraming->FrameSize, 0, 0);
//
//     }
//
//     /* backup allocator framing */
//     RtlMoveMemory(&Allocator->Status.Framing, AllocatorFraming, sizeof(KSALLOCATOR_FRAMING));
//     FreeItem(AllocatorFraming);
//
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI NTSTATUS NTAPI
// KsValidateAllocatorFramingEx(
//     IN  PKSALLOCATOR_FRAMING_EX Framing,
//     IN  ULONG BufferSize,
//     IN  const KSALLOCATOR_FRAMING_EX* PinFraming)
// {
//     if (BufferSize < sizeof(KSALLOCATOR_FRAMING_EX))
//        return  STATUS_INVALID_DEVICE_REQUEST;
//
//     /* verify framing */
//     if ((Framing->FramingItem[0].Flags & KSALLOCATOR_FLAG_PARTIAL_READ_SUPPORT) &&
//          Framing->OutputCompression.RatioNumerator != MAXULONG &&
//          Framing->OutputCompression.RatioDenominator != 0 &&
//          Framing->OutputCompression.RatioDenominator < Framing->OutputCompression.RatioNumerator)
//     {
//         /* framing request is ok */
//         return STATUS_SUCCESS;
//     }
//
//     return STATUS_INVALID_DEVICE_REQUEST;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
