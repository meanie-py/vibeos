use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static KSSTRING_Pin: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x001\x004\x006\0F\x001\0A\08\x000\0-\x004\x007\09\x001\0-\x001\x001\0D\x000\0-\0A\x005\0D\x006\0-\x002\08\0D\0B\x000\x004\0C\x001\x000\x000\x000\x000\0}\0\0\0",
    )
};

pub static KSSTRING_Clock: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x005\x003\x001\x007\x002\x004\08\x000\0-\x004\x007\09\x001\0-\x001\x001\0D\x000\0-\0A\x005\0D\x006\0-\x002\08\0D\0B\x000\x004\0C\x001\x000\x000\x000\x000\0}\0\0\0",
    )
};

pub static KSSTRING_Allocator: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x006\x004\x002\0F\x005\0D\x000\x000\0-\x004\x007\09\x001\0-\x001\x001\0D\x000\0-\0A\x005\0D\x006\0-\x002\08\0D\0B\x000\x004\0C\x001\x000\x000\x000\x000\0}\0\0\0",
    )
};

pub static KSSTRING_TopologyNode: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x000\x006\x002\x001\x000\x006\x001\0A\0-\0E\0E\x007\x005\0-\x001\x001\0D\x000\0-\0B\09\x001\x005\0-\x000\x000\0A\x000\0C\09\x002\x002\x003\x001\09\x006\0}\0\0\0",
    )
};

#[no_mangle]
pub unsafe extern "system" fn KsiCreateObjectType(
    hHandle: HANDLE,
    ObjectType: LPWSTR,
    Buffer: PVOID,
    BufferSize: ULONG,
    DesiredAccess: ACCESS_MASK,
    phHandle: PHANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Length: SIZE_T;
    let mut TotalSize: SIZE_T;
    let mut pStr: LPWSTR;
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    /* get length of object type */
    Length = wcslen(ObjectType as *const u16) as SIZE_T;

    /* get length for request */
    TotalSize = (Length * core::mem::size_of::<WCHAR>() as SIZE_T) + BufferSize as SIZE_T;

    /* append space for '\\'*/
    TotalSize += core::mem::size_of::<WCHAR>() as SIZE_T;

    /* allocate buffer */
    pStr = HeapAlloc(GetProcessHeap(), 0 as DWORD, TotalSize) as LPWSTR;
    if pStr.is_null() {
        /* out of memory */
        return ERROR_NOT_ENOUGH_MEMORY as NTSTATUS;
    }

    /* copy object type */
    wcscpy(pStr as *mut u16, ObjectType as *const u16);

    /* append slash */
    *pStr.offset(Length as isize) = 0x005Cu16;

    /* append parameters */
    memcpy(
        pStr.offset((Length + 1 as SIZE_T) as isize) as *mut core::ffi::c_void,
        Buffer as *const core::ffi::c_void,
        BufferSize as usize,
    );

    /* initialize object name */
    ObjectName.Buffer = pStr;
    ObjectName.MaximumLength = TotalSize as USHORT;
    ObjectName.Length = ObjectName.MaximumLength;

    /* initialize object attributes */
    // InitializeObjectAttributes(&ObjectAttributes, &ObjectName, FACT_0003, hHandle, NULL) expanded inline
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = hHandle;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(ObjectName);
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    /* create the object */
    Status = NtCreateFile(
        phHandle,
        DesiredAccess,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL as ULONG,
        0 as ULONG,
        1 as ULONG,
        0 as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
    );

    /* free buffer */
    HeapFree(GetProcessHeap(), 0 as DWORD, pStr as PVOID);

    /* check for success */
    if !NT_SUCCESS!(Status) {
        /* failed zero handle */
        *phHandle = INVALID_HANDLE_VALUE as HANDLE;

        /* convert error code */
        Status = RtlNtStatusToDosError(Status) as NTSTATUS;
    }

    /* done */
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn KsCreateAllocator(
    ConnectionHandle: HANDLE,
    AllocatorFraming: PKSALLOCATOR_FRAMING,
    AllocatorHandle: PHANDLE,
) -> NTSTATUS {
    return KsiCreateObjectType(
        ConnectionHandle,
        KSSTRING_Allocator.as_ptr() as LPWSTR,
        AllocatorFraming as PVOID,
        core::mem::size_of::<KSALLOCATOR_FRAMING>() as ULONG,
        GENERIC_READ,
        AllocatorHandle,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsCreateClock(
    ConnectionHandle: HANDLE,
    ClockCreate: PKSCLOCK_CREATE,
    ClockHandle: PHANDLE,
) -> NTSTATUS {
    return KsiCreateObjectType(
        ConnectionHandle,
        KSSTRING_Clock.as_ptr() as LPWSTR,
        ClockCreate as PVOID,
        core::mem::size_of::<KSCLOCK_CREATE>() as ULONG,
        GENERIC_READ,
        ClockHandle,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsCreatePin(
    mut FilterHandle: HANDLE,
    mut Connect: PKSPIN_CONNECT,
    mut DesiredAccess: ACCESS_MASK,
    mut ConnectionHandle: PHANDLE,
) -> DWORD {
    let mut BufferSize: ULONG = core::mem::size_of::<KSPIN_CONNECT>() as ULONG;
    let mut DataFormat: PKSDATAFORMAT =
        (Connect.add(1)) as PKSDATAFORMAT;

    BufferSize = BufferSize.wrapping_add((*DataFormat).__bindgen_anon_1.FormatSize);

    // KSSTRING_Pin is the pin create item's GUID string, which ks.sys matches the create
    // against. This once passed a local L"Pin" -- the macro's NAME, not its value -- so every
    // pin create from user mode failed ERROR_GEN_FAILURE and no waveOut device could open.
    return KsiCreateObjectType(
        FilterHandle,
        KSSTRING_Pin.as_ptr() as LPWSTR,
        Connect as PVOID,
        BufferSize,
        DesiredAccess,
        ConnectionHandle,
    ) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn KsCreateTopologyNode(
    ParentHandle: HANDLE,
    NodeCreate: PKSNODE_CREATE,
    DesiredAccess: ACCESS_MASK,
    NodeHandle: PHANDLE,
) -> NTSTATUS {
    return KsiCreateObjectType(
        ParentHandle,
        KSSTRING_TopologyNode.as_ptr() as LPWSTR,
        NodeCreate as PVOID,
        core::mem::size_of::<KSNODE_CREATE>() as ULONG,
        DesiredAccess,
        NodeHandle,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hModule: HANDLE, ulreason: DWORD, lpReserved: LPVOID) -> BOOL {
    match ulreason {
        DLL_PROCESS_ATTACH | DLL_THREAD_ATTACH | DLL_THREAD_DETACH | DLL_PROCESS_DETACH => {}
        _ => {}
    }

    return (TRUE) as BOOL;
}
