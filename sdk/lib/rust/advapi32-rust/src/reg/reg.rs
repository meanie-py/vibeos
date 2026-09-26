use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_DEFAULT_HANDLES: usize = 6;

pub const REG_MAX_NAME_SIZE: u32 = 256;

pub const REG_MAX_DATA_SIZE: u32 = 2048;

static mut HandleTableCS: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };
static mut DefaultHandleTable: [HANDLE; MAX_DEFAULT_HANDLES] = [core::ptr::null_mut(); MAX_DEFAULT_HANDLES];
static mut ProcessHeap: HANDLE = core::ptr::null_mut();
static mut DefaultHandlesDisabled: BOOLEAN = 0u8;
static mut DefaultHandleHKUDisabled: BOOLEAN = 0u8;
static mut DllInitialized: BOOLEAN = 0u8;

unsafe fn is_string(r#type: DWORD) -> core::ffi::c_int {
    ((r#type == REG_SZ) || (r#type == REG_EXPAND_SZ) || (r#type == REG_MULTI_SZ)) as core::ffi::c_int
}
#[no_mangle]
pub unsafe extern "C" fn RegInitialize() -> BOOL {
    if DllInitialized == 0 {
        ProcessHeap = RtlGetProcessHeap();
        RtlZeroMemory(
            DefaultHandleTable.as_mut_ptr() as PVOID,
            MAX_DEFAULT_HANDLES as usize * core::mem::size_of::<HANDLE>(),
        );
        RtlInitializeCriticalSection(core::ptr::addr_of_mut!(HandleTableCS));

        DllInitialized = 1;
    }

    1
}
#[no_mangle]
pub unsafe extern "C" fn RegCleanup() -> BOOL {
    CloseDefaultKeys();
    RtlDeleteCriticalSection(core::ptr::addr_of_mut!(HandleTableCS));

    1
}

unsafe fn OpenPredefinedKey(Index: ULONG, Handle: HANDLE) -> NTSTATUS {
    let Status: NTSTATUS;

    match Index {
        0 => {
            Status = OpenClassesRootKey(Handle as PHANDLE);
        }
        1 => {
            Status = RtlOpenCurrentUser(MAXIMUM_ALLOWED as ACCESS_MASK, Handle as PHANDLE);
        }
        2 => {
            Status = OpenLocalMachineKey(Handle as PHANDLE);
        }
        3 => {
            Status = OpenUsersKey(Handle as PHANDLE);
        }
        5 => {
            Status = OpenCurrentConfigKey(Handle as PHANDLE);
        }
        6 => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
        _ => {
            Status = STATUS_INVALID_PARAMETER;
        }
    }

    Status
}

unsafe fn MapDefaultKey(RealKey: PHANDLE, Key: HKEY) -> NTSTATUS {
    let Handle: PHANDLE;
    let Index: ULONG;
    let DoOpen: BOOLEAN;
    let DefDisabled: BOOLEAN;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if IsPredefKey(Key) == 0 {
        *RealKey = ((Key as ULONG_PTR) & !0x1) as HANDLE;
        return STATUS_SUCCESS;
    }

    /* Handle special cases here */
    Index = GetPredefKeyIndex(Key) as DWORD;
    if Index >= MAX_DEFAULT_HANDLES as DWORD {
        return STATUS_INVALID_PARAMETER;
    }
    RegInitialize();
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(HandleTableCS));

    if SamePredefKey(Key, HKEY_CURRENT_USER) {
        DefDisabled = DefaultHandleHKUDisabled;
    } else {
        DefDisabled = DefaultHandlesDisabled;
    }

    if DefDisabled == 0 {
        Handle = core::ptr::addr_of_mut!(DefaultHandleTable[Index as usize]);
        DoOpen = if *Handle == core::ptr::null_mut() { 1u8 } else { 0u8 };
    } else {
        Handle = RealKey;
        DoOpen = 1u8;
    }

    if DoOpen != 0 {
        /* create/open the default handle */
        Status = OpenPredefinedKey(Index, Handle as *mut HANDLE as HANDLE);
    }

    if NT_SUCCESS(Status) {
        if DefDisabled == 0 {
            *RealKey = *Handle;
        } else {
            *(Handle as PULONG_PTR) |= 0x1;
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(HandleTableCS));

    Status
}

unsafe fn CloseDefaultKeys() {
    let mut i: ULONG;
    RegInitialize();
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(HandleTableCS));

    i = 0;
    while i < 6 {
        if DefaultHandleTable[i as usize] != core::ptr::null_mut() {
            NtClose(DefaultHandleTable[i as usize]);
            DefaultHandleTable[i as usize] = core::ptr::null_mut();
        }
        i += 1;
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(HandleTableCS));
}

unsafe fn OpenClassesRootKey(KeyHandle: PHANDLE) -> NTSTATUS {
    let mut Attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let KeyNameBuffer: &[u16] = &[
        0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
        0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x006F,
        0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x0043, 0x004C, 0x0041,
        0x0053, 0x0053, 0x0045, 0x0053, 0x0000,
    ];
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    KeyName.Length = (34 * 2) as u16;
    KeyName.MaximumLength = (35 * 2) as u16;
    KeyName.Buffer = KeyNameBuffer.as_ptr() as *mut u16;

    let Status: NTSTATUS;

    Attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    Attributes.RootDirectory = core::ptr::null_mut();
    Attributes.ObjectName = core::ptr::addr_of_mut!(KeyName);
    Attributes.Attributes = OBJ_CASE_INSENSITIVE;
    Attributes.SecurityDescriptor = core::ptr::null_mut();
    Attributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(KeyHandle, MAXIMUM_ALLOWED, core::ptr::addr_of_mut!(Attributes));

    if !NT_SUCCESS(Status) {
        return Status;
    }

    MakeHKCRKey(KeyHandle as *mut HKEY);

    Status
}

unsafe fn OpenLocalMachineKey(KeyHandle: PHANDLE) -> NTSTATUS {
    let mut Attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let name: [u16; 18] = [
        0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072,
        0x0079, 0x005C, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E,
        0x0065, 0x0000,
    ];
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    KeyName.Length = (17 * core::mem::size_of::<u16>()) as u16;
    KeyName.MaximumLength = (18 * core::mem::size_of::<u16>()) as u16;
    KeyName.Buffer = name.as_ptr() as *mut u16;

    let Status;

    Attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    Attributes.RootDirectory = core::ptr::null_mut();
    Attributes.ObjectName = core::ptr::addr_of_mut!(KeyName);
    Attributes.Attributes = OBJ_CASE_INSENSITIVE;
    Attributes.SecurityDescriptor = core::ptr::null_mut();
    Attributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(KeyHandle,
                       MAXIMUM_ALLOWED,
                       core::ptr::addr_of_mut!(Attributes));

    Status
}

unsafe fn OpenUsersKey(KeyHandle: PHANDLE) -> NTSTATUS {
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let key_name_buf: [u16; 15] = [
        0x5C, 0x52, 0x65, 0x67, 0x69, 0x73, 0x74, 0x72, 0x79, 0x5C, 0x55, 0x73, 0x65, 0x72, 0x00,
    ];
    KeyName.Buffer = key_name_buf.as_ptr() as *mut u16;
    KeyName.Length = (14 * core::mem::size_of::<u16>()) as u16;
    KeyName.MaximumLength = (15 * core::mem::size_of::<u16>()) as u16;

    let mut Attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    Attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    Attributes.RootDirectory = core::ptr::null_mut();
    Attributes.ObjectName = core::ptr::addr_of_mut!(KeyName);
    Attributes.Attributes = OBJ_CASE_INSENSITIVE;
    Attributes.SecurityDescriptor = core::ptr::null_mut();
    Attributes.SecurityQualityOfService = core::ptr::null_mut();

    NtOpenKey(KeyHandle, MAXIMUM_ALLOWED, core::ptr::addr_of_mut!(Attributes))
}

unsafe fn OpenCurrentConfigKey(KeyHandle: PHANDLE) -> NTSTATUS {
    let KeyNameBuffer: &[u16] = &[
        0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, // \Registry
        0x005C, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065,         // \Machine
        0x005C, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D,                 // \System
        0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074,         // \Current
        0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,                 // Control
        0x0053, 0x0065, 0x0074,                                                 // Set
        0x005C, 0x0048, 0x0061, 0x0072, 0x0064, 0x0077, 0x0061, 0x0072, 0x0065, // \Hardware
        0x0020, 0x0050, 0x0072, 0x006F, 0x0066, 0x0069, 0x006C, 0x0065, 0x0073, //  Profiles
        0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074,         // \Current
        0x0000,
    ];

    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    KeyName.Length = ((KeyNameBuffer.len() - 1) * core::mem::size_of::<u16>()) as u16;
    KeyName.MaximumLength = (KeyNameBuffer.len() * core::mem::size_of::<u16>()) as u16;
    KeyName.Buffer = KeyNameBuffer.as_ptr() as *mut u16;

    let mut Attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    Attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    Attributes.RootDirectory = core::ptr::null_mut();
    Attributes.ObjectName = core::ptr::addr_of_mut!(KeyName);
    Attributes.Attributes = OBJ_CASE_INSENSITIVE;
    Attributes.SecurityDescriptor = core::ptr::null_mut();
    Attributes.SecurityQualityOfService = core::ptr::null_mut();

    NtOpenKey(KeyHandle, MAXIMUM_ALLOWED, core::ptr::addr_of_mut!(Attributes))
}

#[no_mangle]
pub unsafe extern "system" fn RegDisablePredefinedCache() -> LONG {
    RegInitialize();
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(HandleTableCS));
    DefaultHandleHKUDisabled = 1u8;
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(HandleTableCS));
    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegDisablePredefinedCacheEx() -> LONG {
    RegInitialize();
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(HandleTableCS));
    DefaultHandlesDisabled = 1u8;
    DefaultHandleHKUDisabled = 1u8;
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(HandleTableCS));
    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegOverridePredefKey(hKey: HKEY, mut hNewHKey: HKEY) -> LONG {
    let mut ErrorCode: LONG = ERROR_SUCCESS as LONG;

    if (SamePredefKey(hKey, HKEY_CLASSES_ROOT)
        || SamePredefKey(hKey, HKEY_CURRENT_CONFIG)
        || SamePredefKey(hKey, HKEY_CURRENT_USER)
        || SamePredefKey(hKey, HKEY_LOCAL_MACHINE)
        || SamePredefKey(hKey, HKEY_PERFORMANCE_DATA)
        || SamePredefKey(hKey, HKEY_USERS))
        && (((hNewHKey as ULONG_PTR) & 0xF0000000) != 0x80000000)
    {
        let Handle: PHANDLE;
        let Index: ULONG;

        Index = ((hKey as ULONG_PTR) & 0x0FFFFFFF) as ULONG;
        Handle = core::ptr::addr_of_mut!(DefaultHandleTable[Index as usize]);

        if hNewHKey.is_null() {
            let Status: NTSTATUS = OpenPredefinedKey(Index, core::ptr::addr_of_mut!(hNewHKey) as HANDLE);
            if !NT_SUCCESS(Status) {
                return RtlNtStatusToDosError(Status) as LONG;
            }
        }

        RegInitialize();
        RtlEnterCriticalSection(core::ptr::addr_of_mut!(HandleTableCS));

        if !(*Handle).is_null() {
            NtClose(*Handle);
        }

        *Handle = hNewHKey as HANDLE;

        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(HandleTableCS));
    } else {
        ErrorCode = ERROR_INVALID_HANDLE as LONG;
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegCloseKey(hKey: HKEY) -> LONG {
    let Status: NTSTATUS;

    if hKey.is_null() {
        return ERROR_INVALID_HANDLE as LONG;
    }

    if ((hKey as ULONG_PTR) & 0xF0000000) == 0x80000000 {
        return ERROR_SUCCESS as LONG;
    }

    Status = NtClose(hKey as HANDLE);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    ERROR_SUCCESS as LONG
}

unsafe fn RegpCopyTree(hKeySrc: HKEY, hKeyDest: HKEY) -> NTSTATUS {
    #[repr(C)]
    struct REGP_COPY_KEYS {
        ListEntry: LIST_ENTRY,
        hKeySrc: HANDLE,
        hKeyDest: HANDLE,
    }

    let mut copyQueueHead: LIST_ENTRY = core::mem::zeroed();
    let mut copyKeys: *mut REGP_COPY_KEYS;
    let mut newCopyKeys: *mut REGP_COPY_KEYS;

    let mut InfoBuffer: PVOID;
    let mut Index: ULONG;
    let mut BufferSizeRequired: ULONG = 0;
    let mut BufferSize: ULONG = 0x200;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Status2: NTSTATUS;

    InitializeListHead(&mut copyQueueHead);

    InfoBuffer = RtlAllocateHeap(ProcessHeap, 0, BufferSize as SIZE_T);
    if InfoBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    copyKeys = RtlAllocateHeap(ProcessHeap, 0, core::mem::size_of::<REGP_COPY_KEYS>() as SIZE_T) as *mut REGP_COPY_KEYS;
    if !copyKeys.is_null() {
        (*copyKeys).hKeySrc = hKeySrc as HANDLE;
        (*copyKeys).hKeyDest = hKeyDest as HANDLE;
        InsertHeadList(&mut copyQueueHead, core::ptr::addr_of_mut!((*copyKeys).ListEntry));

        loop {
            copyKeys = (copyQueueHead.Flink as *mut u8).offset(-(core::mem::offset_of!(REGP_COPY_KEYS, ListEntry) as isize)) as *mut REGP_COPY_KEYS;

            /* enumerate all values and copy them */
            Index = 0;
            loop {
                let KeyValue = InfoBuffer as *mut KEY_VALUE_FULL_INFORMATION;
                Status2 = NtEnumerateValueKey(
                    (*copyKeys).hKeySrc,
                    Index,
                    KeyValueFullInformation,
                    InfoBuffer,
                    BufferSize,
                    &mut BufferSizeRequired,
                );
                if NT_SUCCESS(Status2) {
                    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();

                    ValueName.Length = (*KeyValue).NameLength as u16;
                    ValueName.MaximumLength = ValueName.Length;
                    ValueName.Buffer = core::ptr::addr_of_mut!((*KeyValue).Name) as PWSTR;

                    let Data = (KeyValue as usize + (*KeyValue).DataOffset as usize) as PVOID;

                    Status2 = NtSetValueKey(
                        (*copyKeys).hKeyDest,
                        &mut ValueName,
                        (*KeyValue).TitleIndex,
                        (*KeyValue).Type,
                        Data,
                        (*KeyValue).DataLength,
                    );

                    if !NT_SUCCESS(Status2) && NT_SUCCESS(Status) {
                        Status = Status2;
                    }

                    Index += 1;
                } else if Status2 == STATUS_BUFFER_OVERFLOW {
                    let Buffer = RtlReAllocateHeap(ProcessHeap, 0, InfoBuffer, BufferSizeRequired as SIZE_T);
                    if !Buffer.is_null() {
                        InfoBuffer = Buffer;
                        BufferSize = BufferSizeRequired;
                    } else {
                        Status2 = STATUS_INSUFFICIENT_RESOURCES;
                        Index += 1;
                        if NT_SUCCESS(Status) {
                            Status = Status2;
                        }
                    }
                } else {
                    if Status2 != STATUS_NO_MORE_ENTRIES && NT_SUCCESS(Status) {
                        Status = Status2;
                    }
                    break;
                }
            }

            /* enumerate all subkeys and open and enqueue them */
            Index = 0;
            loop {
                let KeyNode = InfoBuffer as *mut KEY_NODE_INFORMATION;
                Status2 = NtEnumerateKey(
                    (*copyKeys).hKeySrc,
                    Index,
                    KeyNodeInformation,
                    InfoBuffer,
                    BufferSize,
                    &mut BufferSizeRequired,
                );
                if NT_SUCCESS(Status2) {
                    let mut KeyHandle: HANDLE = core::ptr::null_mut();
                    let mut NewKeyHandle: HANDLE = core::ptr::null_mut();
                    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
                    let mut SubKeyName: _UNICODE_STRING = core::mem::zeroed();
                    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();

                    SubKeyName.Length = (*KeyNode).NameLength as u16;
                    SubKeyName.MaximumLength = SubKeyName.Length;
                    SubKeyName.Buffer = core::ptr::addr_of_mut!((*KeyNode).Name) as PWSTR;
                    ClassName.Length = (*KeyNode).ClassLength as u16;
                    ClassName.MaximumLength = ClassName.Length;
                    ClassName.Buffer = (KeyNode as usize + (*KeyNode).ClassOffset as usize) as PWSTR;

                    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
                    ObjectAttributes.RootDirectory = (*copyKeys).hKeySrc;
                    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
                    ObjectAttributes.ObjectName = &mut SubKeyName;
                    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                    Status2 = NtOpenKey(
                        &mut KeyHandle,
                        KEY_ENUMERATE_SUB_KEYS | KEY_QUERY_VALUE,
                        &mut ObjectAttributes,
                    );
                    if NT_SUCCESS(Status2) {
                        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
                        ObjectAttributes.RootDirectory = (*copyKeys).hKeyDest;
                        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
                        ObjectAttributes.ObjectName = &mut SubKeyName;
                        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                        Status2 = NtCreateKey(
                            &mut NewKeyHandle,
                            KEY_ALL_ACCESS,
                            &mut ObjectAttributes,
                            (*KeyNode).TitleIndex,
                            &mut ClassName,
                            0,
                            core::ptr::null_mut(),
                        );
                        if NT_SUCCESS(Status2) {
                            newCopyKeys = RtlAllocateHeap(ProcessHeap, 0, core::mem::size_of::<REGP_COPY_KEYS>() as SIZE_T) as *mut REGP_COPY_KEYS;
                            if !newCopyKeys.is_null() {
                                (*newCopyKeys).hKeySrc = KeyHandle;
                                (*newCopyKeys).hKeyDest = NewKeyHandle;
                                InsertTailList(&mut copyQueueHead, core::ptr::addr_of_mut!((*newCopyKeys).ListEntry));
                            } else {
                                NtClose(KeyHandle);
                                NtClose(NewKeyHandle);
                                Status2 = STATUS_INSUFFICIENT_RESOURCES;
                            }
                        } else {
                            NtClose(KeyHandle);
                        }
                    }

                    if !NT_SUCCESS(Status2) && NT_SUCCESS(Status) {
                        Status = Status2;
                    }

                    Index += 1;
                } else if Status2 == STATUS_BUFFER_OVERFLOW {
                    let Buffer = RtlReAllocateHeap(ProcessHeap, 0, InfoBuffer, BufferSizeRequired as SIZE_T);
                    if !Buffer.is_null() {
                        InfoBuffer = Buffer;
                        BufferSize = BufferSizeRequired;
                    } else {
                        Status2 = STATUS_INSUFFICIENT_RESOURCES;
                        Index += 1;
                        if NT_SUCCESS(Status) {
                            Status = Status2;
                        }
                    }
                } else {
                    if Status2 != STATUS_NO_MORE_ENTRIES && NT_SUCCESS(Status) {
                        Status = Status2;
                    }
                    break;
                }
            }

            if (*copyKeys).hKeySrc != hKeySrc as HANDLE {
                NtClose((*copyKeys).hKeySrc);
            }
            if (*copyKeys).hKeyDest != hKeyDest as HANDLE {
                NtClose((*copyKeys).hKeyDest);
            }

            RemoveEntryList(core::ptr::addr_of_mut!((*copyKeys).ListEntry));

            RtlFreeHeap(ProcessHeap, 0, copyKeys as PVOID);

            if IsListEmpty(&mut copyQueueHead)  {
                break;
            }
        }
    } else {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlFreeHeap(ProcessHeap, 0, InfoBuffer);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RegCopyTreeW(
    hKeySrc: HKEY,
    lpSubKey: LPCWSTR,
    hKeyDest: HKEY,
) -> LONG {
    let mut DestKeyHandle: HANDLE = core::ptr::null_mut();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let CurKey: HANDLE;
    let mut SubKeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = MapDefaultKey(&mut KeyHandle, hKeySrc);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    'cleanup2: {
        Status = MapDefaultKey(&mut DestKeyHandle, hKeyDest);
        if !NT_SUCCESS(Status) {
            break 'cleanup2;
        }

        'cleanup: {
            if lpSubKey != core::ptr::null_mut() {
                let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
                let mut SubKeyName: _UNICODE_STRING = core::mem::zeroed();

                RtlInitUnicodeString(&mut SubKeyName, lpSubKey);

                ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
                ObjectAttributes.RootDirectory = KeyHandle;
                ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
                ObjectAttributes.ObjectName = &mut SubKeyName;
                ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                Status = NtOpenKey(
                    &mut SubKeyHandle,
                    KEY_ENUMERATE_SUB_KEYS | KEY_QUERY_VALUE,
                    &mut ObjectAttributes,
                );
                if !NT_SUCCESS(Status) {
                    break 'cleanup;
                }

                CurKey = SubKeyHandle;
            } else {
                CurKey = KeyHandle;
            }

            Status = RegpCopyTree(CurKey as HKEY, hKeyDest);

            if SubKeyHandle != core::ptr::null_mut() {
                NtClose(SubKeyHandle);
            }
        }

        if (DestKeyHandle as ULONG_PTR) & 0x1 != 0 {
            NtClose(DestKeyHandle);
        }
    }

    if (KeyHandle as ULONG_PTR) & 0x1 != 0 {
        NtClose(KeyHandle);
    }

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegCopyTreeA(
    hKeySrc: HKEY,
    lpSubKey: LPCSTR,
    hKeyDest: HKEY,
) -> LONG {
    let mut SubKeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let Ret: LONG;

    if !lpSubKey.is_null()
        && RtlCreateUnicodeStringFromAsciiz(&mut SubKeyName, lpSubKey) == 0
    {
        return ERROR_NOT_ENOUGH_MEMORY as LONG;
    }

    Ret = RegCopyTreeW(hKeySrc, SubKeyName.Buffer, hKeyDest);

    RtlFreeUnicodeString(&mut SubKeyName);

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn RegConnectRegistryA(
    lpMachineName: LPCSTR,
    hKey: HKEY,
    phkResult: PHKEY,
) -> LONG {
    let mut MachineName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let Ret: LONG;

    if !lpMachineName.is_null()
        && RtlCreateUnicodeStringFromAsciiz(&mut MachineName, lpMachineName) == 0
    {
        return ERROR_NOT_ENOUGH_MEMORY as LONG;
    }

    Ret = RegConnectRegistryW(MachineName.Buffer, hKey, phkResult);

    RtlFreeUnicodeString(&mut MachineName);

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn RegConnectRegistryW(
    mut lpMachineName: LPCWSTR,
    hKey: HKEY,
    phkResult: PHKEY,
) -> LONG {
    let ret: LONG;

    if lpMachineName.is_null() || *lpMachineName == 0 {
        ret = RegOpenKeyW(hKey, core::ptr::null_mut(), phkResult);
    } else {
        let mut compName: [u16; (MAX_COMPUTERNAME_LENGTH + 1) as usize] =
            [0; (MAX_COMPUTERNAME_LENGTH + 1) as usize];
        let mut len: DWORD =
            (core::mem::size_of::<[u16; (MAX_COMPUTERNAME_LENGTH + 1) as usize]>()
                / core::mem::size_of::<u16>()) as DWORD;

        if *lpMachineName.offset(0) == b'\\' as u16
            && *lpMachineName.offset(1) == b'\\' as u16
        {
            lpMachineName = lpMachineName.offset(2);
        }

        if GetComputerNameW(compName.as_mut_ptr(), &mut len) != 0 {
            if _wcsicmp(lpMachineName, compName.as_ptr()) == 0 {
                ret = RegOpenKeyW(hKey, core::ptr::null_mut(), phkResult);
            } else {
                ret = ERROR_BAD_NETPATH as LONG;
            }
        } else {
            ret = GetLastError() as LONG;
        }
    }

    ret
}

unsafe fn CreateNestedKey(
    KeyHandle: PHKEY,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    ClassString: PUNICODE_STRING,
    dwOptions: DWORD,
    samDesired: REGSAM,
    lpdwDisposition: *mut DWORD,
) -> NTSTATUS {
    let mut LocalObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut LocalKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut Disposition: ULONG = 0;
    let mut Status: NTSTATUS;
    let FullNameLength: ULONG;
    let mut Length: ULONG;
    let mut Ptr: PWCHAR;
    let mut LocalKeyHandle: HANDLE;

    Status = NtCreateKey(
        KeyHandle as PHANDLE,
        samDesired,
        ObjectAttributes,
        0,
        ClassString,
        dwOptions,
        lpdwDisposition as PULONG,
    );
    if Status != STATUS_OBJECT_NAME_NOT_FOUND {
        return Status;
    }

    RtlCopyMemory(
        core::ptr::addr_of_mut!(LocalObjectAttributes) as *mut core::ffi::c_void,
        ObjectAttributes as *const core::ffi::c_void,
        core::mem::size_of::<_OBJECT_ATTRIBUTES>() as _,
    );
    RtlCreateUnicodeString(
        core::ptr::addr_of_mut!(LocalKeyName),
        (*(*ObjectAttributes).ObjectName).Buffer,
    );
    LocalObjectAttributes.ObjectName = core::ptr::addr_of_mut!(LocalKeyName);
    FullNameLength = (LocalKeyName.Length as ULONG) / core::mem::size_of::<u16>() as ULONG;

    LocalKeyHandle = core::ptr::null_mut();

    while Status == STATUS_OBJECT_NAME_NOT_FOUND {
        Ptr = wcsrchr(LocalKeyName.Buffer, '\\' as u16);
        if Ptr.is_null() || Ptr == LocalKeyName.Buffer {
            Status = STATUS_UNSUCCESSFUL;
            break;
        }

        *Ptr = 0u16;
        LocalKeyName.Length = (wcslen(LocalKeyName.Buffer) as u16) * core::mem::size_of::<u16>() as u16;

        Status = NtCreateKey(
            core::ptr::addr_of_mut!(LocalKeyHandle),
            KEY_CREATE_SUB_KEY,
            core::ptr::addr_of_mut!(LocalObjectAttributes),
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(Disposition),
        );
    }

    if !NT_SUCCESS(Status) {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(LocalKeyName));
        return Status;
    }

    Length = wcslen(LocalKeyName.Buffer) as ULONG;
    loop {
        if !LocalKeyHandle.is_null() {
            NtClose(LocalKeyHandle);
        }

        *LocalKeyName.Buffer.offset(Length as isize) = '\\' as u16;
        Length = wcslen(LocalKeyName.Buffer) as ULONG;
        LocalKeyName.Length = (Length * core::mem::size_of::<u16>() as ULONG) as u16;

        if Length == FullNameLength {
            Status = NtCreateKey(
                KeyHandle as PHANDLE,
                samDesired,
                ObjectAttributes,
                0,
                ClassString,
                dwOptions,
                lpdwDisposition as PULONG,
            );
            break;
        }

        Status = NtCreateKey(
            core::ptr::addr_of_mut!(LocalKeyHandle),
            KEY_CREATE_SUB_KEY,
            core::ptr::addr_of_mut!(LocalObjectAttributes),
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(Disposition),
        );
        if !NT_SUCCESS(Status) {
            break;
        }
    }

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(LocalKeyName));

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RegCreateKeyExA(
    hKey: HKEY,
    lpSubKey: LPCSTR,
    Reserved: DWORD,
    lpClass: LPSTR,
    dwOptions: DWORD,
    samDesired: REGSAM,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
    phkResult: PHKEY,
    lpdwDisposition: LPDWORD,
) -> LONG {
    let mut SubKeyString: _UNICODE_STRING = core::mem::zeroed();
    let mut ClassString: _UNICODE_STRING = core::mem::zeroed();
    let mut ErrorCode: DWORD;

    ClassString.Length = 0;
    ClassString.MaximumLength = 0;
    ClassString.Buffer = core::ptr::null_mut();
    SubKeyString.Length = 0;
    SubKeyString.MaximumLength = 0;
    SubKeyString.Buffer = core::ptr::null_mut();

    'exit: {
        if !lpClass.is_null() {
            if RtlCreateUnicodeStringFromAsciiz(
                core::ptr::addr_of_mut!(ClassString),
                lpClass,
            ) == 0
            {
                ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'exit;
            }
        }

        if !lpSubKey.is_null() {
            if RtlCreateUnicodeStringFromAsciiz(
                core::ptr::addr_of_mut!(SubKeyString),
                lpSubKey,
            ) == 0
            {
                ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
                break 'exit;
            }
        }

        ErrorCode = RegCreateKeyExW(
            hKey,
            SubKeyString.Buffer,
            Reserved,
            ClassString.Buffer,
            dwOptions,
            samDesired,
            lpSecurityAttributes,
            phkResult,
            lpdwDisposition,
        ) as DWORD;
    }

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(SubKeyString));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(ClassString));

    ErrorCode as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegCreateKeyExW(
    hKey: HKEY,
    lpSubKey: LPCWSTR,
    Reserved: DWORD,
    lpClass: LPWSTR,
    dwOptions: DWORD,
    samDesired: REGSAM,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
    phkResult: PHKEY,
    lpdwDisposition: LPDWORD,
) -> LONG {
    let mut SubKeyString: _UNICODE_STRING = core::mem::zeroed();
    let mut ClassString: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ParentKey: HANDLE = core::ptr::null_mut();
    let mut Attributes: ULONG = OBJ_CASE_INSENSITIVE;
    let mut Status: NTSTATUS;

    Status = MapDefaultKey(&mut ParentKey, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    if IsHKCRKey(ParentKey as HKEY) != 0 {
        let ErrorCode = CreateHKCRKey(
            ParentKey as HKEY,
            lpSubKey,
            Reserved,
            lpClass,
            dwOptions,
            samDesired,
            lpSecurityAttributes,
            phkResult,
            lpdwDisposition,
        );
        ClosePredefKey(ParentKey);
        return ErrorCode;
    }

    if dwOptions & REG_OPTION_OPEN_LINK != 0 {
        Attributes |= OBJ_OPENLINK;
    }

    RtlInitUnicodeString(&mut ClassString, lpClass);
    RtlInitUnicodeString(&mut SubKeyString, lpSubKey);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = ParentKey as HANDLE;
    ObjectAttributes.Attributes = Attributes;
    ObjectAttributes.ObjectName = &mut SubKeyString;
    ObjectAttributes.SecurityDescriptor = if !lpSecurityAttributes.is_null() {
        (*lpSecurityAttributes).lpSecurityDescriptor
    } else {
        core::ptr::null_mut()
    };
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = CreateNestedKey(
        phkResult,
        &mut ObjectAttributes,
        if lpClass.is_null() {
            core::ptr::null_mut()
        } else {
            &mut ClassString
        },
        dwOptions,
        samDesired,
        lpdwDisposition,
    );

    ClosePredefKey(ParentKey);

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegCreateKeyA(
    hKey: HKEY,
    lpSubKey: LPCSTR,
    phkResult: PHKEY,
) -> LONG {
    RegCreateKeyExA(
        hKey,
        lpSubKey,
        0,
        core::ptr::null_mut(),
        0,
        MAXIMUM_ALLOWED,
        core::ptr::null_mut(),
        phkResult,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn RegCreateKeyW(
    hKey: HKEY,
    lpSubKey: LPCWSTR,
    phkResult: PHKEY,
) -> LONG {
    RegCreateKeyExW(
        hKey,
        lpSubKey,
        0,
        core::ptr::null_mut(),
        0,
        MAXIMUM_ALLOWED as REGSAM,
        core::ptr::null_mut(),
        phkResult,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn RegDeleteKeyA(hKey: HKEY, lpSubKey: LPCSTR) -> LONG {
    RegDeleteKeyExA(hKey, lpSubKey, 0, 0)
}

#[no_mangle]
pub unsafe extern "system" fn RegDeleteKeyW(
    hKey: HKEY,
    lpSubKey: LPCWSTR,
) -> LONG {
    RegDeleteKeyExW(hKey, lpSubKey, 0, 0)
}

#[no_mangle]
pub unsafe extern "system" fn RegDeleteKeyExA(
    hKey: HKEY,
    lpSubKey: LPCSTR,
    samDesired: REGSAM,
    Reserved: DWORD,
) -> LONG {
    let ErrorCode: LONG;
    let mut SubKeyName: _UNICODE_STRING = core::mem::zeroed();

    if !lpSubKey.is_null() {
        if RtlCreateUnicodeStringFromAsciiz(&mut SubKeyName, lpSubKey) == 0 {
            return ERROR_NOT_ENOUGH_MEMORY as LONG;
        }
    } else {
        SubKeyName.Length = 0;
        SubKeyName.MaximumLength = 0;
        SubKeyName.Buffer = core::ptr::null_mut();
    }

    ErrorCode = RegDeleteKeyExW(hKey, SubKeyName.Buffer, samDesired, Reserved);

    RtlFreeUnicodeString(&mut SubKeyName);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegDeleteKeyExW(
    hKey: HKEY,
    lpSubKey: LPCWSTR,
    samDesired: REGSAM,
    Reserved: DWORD,
) -> LONG {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SubKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ParentKey: HANDLE = core::ptr::null_mut();
    let mut TargetKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    /* Make sure we got a subkey */
    if lpSubKey.is_null() {
        /* Fail */
        return ERROR_INVALID_PARAMETER as LONG;
    }

    Status = MapDefaultKey(&mut ParentKey, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    if IsHKCRKey(ParentKey as HKEY) != 0 {
        let ErrorCode = DeleteHKCRKey(ParentKey as HKEY, lpSubKey, samDesired, Reserved);
        ClosePredefKey(ParentKey);
        return ErrorCode;
    }

    /* KEY_WOW64_32KEY / KEY_WOW64_64KEY are VIEW SELECTORS, not access bits, and
     * they are honoured in the CM parse (config-rust/src/cmwow64.rs): a native
     * process naming KEY_WOW64_32KEY gets the 32-bit view, and CM clears both
     * before the access check so they never reach a DACL. NtOpenKey stopped
     * refusing them in E21 (911b1e1d82b). So the only thing left to do here is
     * pass them through -- this used to log "Wow64 not yet supported" and drop
     * them, which silently deleted from the NATIVE hive when the caller had
     * explicitly asked for the other view. */
    let Wow64View = samDesired & (KEY_WOW64_32KEY | KEY_WOW64_64KEY);

    RtlInitUnicodeString(&mut SubKeyName, lpSubKey);
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut SubKeyName,
        OBJ_CASE_INSENSITIVE,
        ParentKey,
        core::ptr::null_mut(),
    );
    Status = NtOpenKey(&mut TargetKey, DELETE | Wow64View, &mut ObjectAttributes);

    'cleanup: {
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        Status = NtDeleteKey(TargetKey);
        NtClose(TargetKey);
    }

    ClosePredefKey(ParentKey);

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegDeleteKeyValueW(
    hKey: HKEY,
    lpSubKey: LPCWSTR,
    lpValueName: LPCWSTR,
) -> LONG {
    let mut hSubKey: HKEY = hKey;
    let mut ErrorCode: LONG;

    if !lpSubKey.is_null() {
        ErrorCode = RegOpenKeyExW(hKey, lpSubKey, 0, KEY_SET_VALUE as REGSAM, &mut hSubKey);
        if ErrorCode != 0 {
            return ErrorCode;
        }
    }
    ErrorCode = RegDeleteValueW(hSubKey, lpValueName);

    if hSubKey != hKey {
        RegCloseKey(hSubKey);
    }
    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegDeleteKeyValueA(
    hKey: HKEY,
    lpSubKey: LPCSTR,
    lpValueName: LPCSTR,
) -> LONG {
    let mut SubKey: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ValueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let Ret: LONG;

    if !lpSubKey.is_null()
        && RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(SubKey), lpSubKey) == 0
    {
        return ERROR_NOT_ENOUGH_MEMORY as LONG;
    }

    if !lpValueName.is_null()
        && RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(ValueName), lpValueName) == 0
    {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(SubKey));
        return ERROR_NOT_ENOUGH_MEMORY as LONG;
    }

    Ret = RegDeleteKeyValueW(hKey, SubKey.Buffer, ValueName.Buffer);

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(SubKey));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(ValueName));

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn RegDeleteTreeW(hKey: HKEY, lpszSubKey: LPCWSTR) -> LSTATUS {
    let mut ret: LONG;
    let mut dwMaxSubkeyLen: DWORD = 0;
    let mut dwMaxValueLen: DWORD = 0;
    let mut dwMaxLen: DWORD;
    let mut dwSize: DWORD;
    let Status: NTSTATUS;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut hSubKey: HKEY;
    let mut szNameBuf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut lpszName: *mut u16 = szNameBuf.as_mut_ptr();

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LSTATUS;
    }

    hSubKey = KeyHandle as HKEY;

    if !lpszSubKey.is_null() {
        ret = RegOpenKeyExW(KeyHandle as HKEY, lpszSubKey, 0, KEY_READ, &mut hSubKey);
        if ret != 0 {
            if (KeyHandle as usize) & 0x1 != 0 {
                NtClose(KeyHandle);
            }
            return ret as LSTATUS;
        }
    }

    ret = 'cleanup: {
        ret = RegQueryInfoKeyW(
            hSubKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut dwMaxSubkeyLen,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut dwMaxValueLen,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if ret != 0 {
            break 'cleanup ret;
        }

        dwMaxSubkeyLen += 1;
        dwMaxValueLen += 1;
        dwMaxLen = if dwMaxSubkeyLen > dwMaxValueLen { dwMaxSubkeyLen } else { dwMaxValueLen };
        if dwMaxLen as usize > core::mem::size_of::<[u16; MAX_PATH as usize]>() / core::mem::size_of::<u16>() {
            lpszName = RtlAllocateHeap(RtlGetProcessHeap(), 0, ((dwMaxLen as usize) * core::mem::size_of::<u16>()) as SIZE_T) as *mut u16;
            if lpszName.is_null() {
                ret = ERROR_NOT_ENOUGH_MEMORY as LONG;
                break 'cleanup ret;
            }
        }

        loop {
            dwSize = dwMaxLen;
            if RegEnumKeyExW(
                hSubKey,
                0,
                lpszName,
                &mut dwSize,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) != 0 {
                break;
            }

            ret = RegDeleteTreeW(hSubKey, lpszName);
            if ret != 0 {
                break 'cleanup ret;
            }
        }

        if !lpszSubKey.is_null() {
            ret = RegDeleteKeyW(KeyHandle as HKEY, lpszSubKey);
        } else {
            loop {
                dwSize = dwMaxLen;
                if RegEnumValueW(
                    KeyHandle as HKEY,
                    0,
                    lpszName,
                    &mut dwSize,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) != 0 {
                    break;
                }

                ret = RegDeleteValueW(KeyHandle as HKEY, lpszName);
                if ret != 0 {
                    break 'cleanup ret;
                }
            }
        }

        ret
    };

    if lpszName != szNameBuf.as_mut_ptr() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, lpszName as PVOID);
    }
    if !lpszSubKey.is_null() {
        RegCloseKey(hSubKey);
    }

    if (KeyHandle as usize) & 0x1 != 0 {
        NtClose(KeyHandle);
    }

    ret as LSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn RegDeleteTreeA(hKey: HKEY, lpSubKey: LPCSTR) -> LONG {
    let mut SubKeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let Ret: LONG;

    if !lpSubKey.is_null()
        && RtlCreateUnicodeStringFromAsciiz(&mut SubKeyName, lpSubKey) == 0
    {
        return ERROR_NOT_ENOUGH_MEMORY as LONG;
    }

    Ret = RegDeleteTreeW(hKey, SubKeyName.Buffer);

    RtlFreeUnicodeString(&mut SubKeyName);

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn RegDisableReflectionKey(hBase: HKEY) -> LONG {
    ERROR_CALL_NOT_IMPLEMENTED as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegEnableReflectionKey(hBase: HKEY) -> LONG {
    ERROR_CALL_NOT_IMPLEMENTED as LONG
}

unsafe fn RegpApplyRestrictions(dwFlags: DWORD, dwType: DWORD, cbData: DWORD, ret: PLONG) {
    if *ret == ERROR_SUCCESS as LONG || *ret == ERROR_MORE_DATA as LONG {
        let mut dwMask: DWORD = 0;

        match dwType {
            REG_NONE => dwMask = RRF_RT_REG_NONE,
            REG_SZ => dwMask = RRF_RT_REG_SZ,
            REG_EXPAND_SZ => dwMask = RRF_RT_REG_EXPAND_SZ,
            REG_MULTI_SZ => dwMask = RRF_RT_REG_MULTI_SZ,
            REG_BINARY => dwMask = RRF_RT_REG_BINARY,
            REG_DWORD => dwMask = RRF_RT_REG_DWORD,
            REG_QWORD => dwMask = RRF_RT_REG_QWORD,
            _ => {}
        }

        if dwFlags & dwMask != 0 {
            if dwType == REG_BINARY {
                let mut cbExpect: DWORD = 0;

                if (dwFlags & RRF_RT_ANY) == RRF_RT_DWORD {
                    cbExpect = 4;
                } else if (dwFlags & RRF_RT_ANY) == RRF_RT_QWORD {
                    cbExpect = 8;
                }

                if cbExpect != 0 && cbData != cbExpect {
                    *ret = ERROR_DATATYPE_MISMATCH as LONG;
                }
            }
        } else {
            *ret = ERROR_UNSUPPORTED_TYPE as LONG;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RegGetValueW(
    mut hKey: HKEY,
    pszSubKey: LPCWSTR,
    pszValue: LPCWSTR,
    dwFlags: DWORD,
    pdwType: LPDWORD,
    pvData: PVOID,
    pcbData: LPDWORD,
) -> LSTATUS {
    let mut dwType: DWORD;
    let mut cbData: DWORD = if !pcbData.is_null() { *pcbData } else { 0 };
    let mut pvBuf: PVOID = core::ptr::null_mut();
    let mut ret: LONG;

    if !pvData.is_null() && pcbData.is_null() {
        return ERROR_INVALID_PARAMETER as LSTATUS;
    }
    if (dwFlags & RRF_RT_REG_EXPAND_SZ) != 0
        && (dwFlags & RRF_NOEXPAND) == 0
        && ((dwFlags & RRF_RT_ANY) != RRF_RT_ANY)
    {
        return ERROR_INVALID_PARAMETER as LSTATUS;
    }

    if !pszSubKey.is_null() && *pszSubKey != 0 {
        ret = RegOpenKeyExW(hKey, pszSubKey, 0, KEY_QUERY_VALUE as REGSAM, &mut hKey);
        if ret != ERROR_SUCCESS as LONG {
            return ret;
        }
    }

    dwType = REG_NONE;
    ret = RegQueryValueExW(
        hKey,
        pszValue,
        core::ptr::null_mut(),
        &mut dwType,
        pvData as LPBYTE,
        &mut cbData,
    );

    if (ret == ERROR_SUCCESS as LONG || ret == ERROR_MORE_DATA as LONG)
        && dwType == REG_EXPAND_SZ
        && (dwFlags & RRF_NOEXPAND) == 0
    {
        loop {
            HeapFree(GetProcessHeap(), 0, pvBuf);

            pvBuf = HeapAlloc(GetProcessHeap(), 0, cbData as SIZE_T);
            if pvBuf.is_null() {
                ret = ERROR_NOT_ENOUGH_MEMORY as LONG;
                break;
            }

            if ret == ERROR_MORE_DATA as LONG || pvData.is_null() {
                ret = RegQueryValueExW(
                    hKey,
                    pszValue,
                    core::ptr::null_mut(),
                    &mut dwType,
                    pvBuf as LPBYTE,
                    &mut cbData,
                );
            } else {
                CopyMemory(pvBuf, pvData, cbData as usize);
            }

            if !(dwType == REG_EXPAND_SZ && ret == ERROR_MORE_DATA as LONG) {
                break;
            }
        }

        if ret == ERROR_SUCCESS as LONG {
            if dwType == REG_EXPAND_SZ {
                cbData = ExpandEnvironmentStringsW(
                    pvBuf as LPCWSTR,
                    pvData as LPWSTR,
                    if !pcbData.is_null() { *pcbData } else { 0 },
                ) * core::mem::size_of::<u16>() as DWORD;
                dwType = REG_SZ;
                if !pvData.is_null() && !pcbData.is_null() && cbData > *pcbData {
                    ret = ERROR_MORE_DATA as LONG;
                }
            } else if !pvData.is_null() {
                CopyMemory(pvData, pvBuf, *pcbData as usize);
            }
        }

        HeapFree(GetProcessHeap(), 0, pvBuf);
    }

    if !pszSubKey.is_null() && *pszSubKey != 0 {
        RegCloseKey(hKey);
    }

    RegpApplyRestrictions(dwFlags, dwType, cbData, &mut ret);

    if !pvData.is_null() && ret != ERROR_SUCCESS as LONG && (dwFlags & RRF_ZEROONFAILURE) != 0 {
        ZeroMemory(pvData, *pcbData as usize);
    }

    if !pdwType.is_null() {
        *pdwType = dwType;
    }

    if !pcbData.is_null() {
        *pcbData = cbData;
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn RegGetValueA(
    mut hKey: HKEY,
    pszSubKey: LPCSTR,
    pszValue: LPCSTR,
    dwFlags: DWORD,
    pdwType: LPDWORD,
    pvData: PVOID,
    pcbData: LPDWORD,
) -> LSTATUS {
    let mut dwType: DWORD = 0;
    let mut cbData: DWORD = if !pcbData.is_null() { *pcbData } else { 0 };
    let mut pvBuf: PVOID = core::ptr::null_mut();
    let mut ret: LONG;

    if !pvData.is_null() && pcbData.is_null() {
        return ERROR_INVALID_PARAMETER as LSTATUS;
    }
    if (dwFlags & RRF_RT_REG_EXPAND_SZ) != 0
        && (dwFlags & RRF_NOEXPAND) == 0
        && ((dwFlags & RRF_RT_ANY) != RRF_RT_ANY)
    {
        return ERROR_INVALID_PARAMETER as LSTATUS;
    }

    if !pszSubKey.is_null() && *pszSubKey != 0 {
        ret = RegOpenKeyExA(hKey, pszSubKey, 0, KEY_QUERY_VALUE as REGSAM, &mut hKey);
        if ret != ERROR_SUCCESS as LONG {
            return ret;
        }
    }

    ret = RegQueryValueExA(
        hKey,
        pszValue,
        core::ptr::null_mut(),
        &mut dwType,
        pvData as LPBYTE,
        &mut cbData,
    );

    if (ret == ERROR_SUCCESS as LONG || ret == ERROR_MORE_DATA as LONG)
        && (dwType == REG_EXPAND_SZ && (dwFlags & RRF_NOEXPAND) == 0)
    {
        loop {
            HeapFree(GetProcessHeap(), 0, pvBuf);

            pvBuf = HeapAlloc(GetProcessHeap(), 0, cbData as SIZE_T);
            if pvBuf.is_null() {
                ret = ERROR_NOT_ENOUGH_MEMORY as LONG;
                break;
            }

            if ret == ERROR_MORE_DATA as LONG || pvData.is_null() {
                ret = RegQueryValueExA(
                    hKey,
                    pszValue,
                    core::ptr::null_mut(),
                    &mut dwType,
                    pvBuf as LPBYTE,
                    &mut cbData,
                );
            } else {
                CopyMemory(pvBuf, pvData, cbData as usize);
            }

            if !(dwType == REG_EXPAND_SZ && ret == ERROR_MORE_DATA as LONG) {
                break;
            }
        }

        if ret == ERROR_SUCCESS as LONG {
            if dwType == REG_EXPAND_SZ {
                cbData = ExpandEnvironmentStringsA(
                    pvBuf as LPCSTR,
                    pvData as LPSTR,
                    if !pcbData.is_null() { *pcbData } else { 0 },
                );
                dwType = REG_SZ;
                if !pvData.is_null() && !pcbData.is_null() && cbData > *pcbData {
                    ret = ERROR_MORE_DATA as LONG;
                }
            } else if !pvData.is_null() {
                CopyMemory(pvData, pvBuf, *pcbData as usize);
            }
        }

        HeapFree(GetProcessHeap(), 0, pvBuf);
    }

    if !pszSubKey.is_null() && *pszSubKey != 0 {
        RegCloseKey(hKey);
    }

    RegpApplyRestrictions(dwFlags, dwType, cbData, &mut ret);

    if !pvData.is_null() && ret != ERROR_SUCCESS as LONG && (dwFlags & RRF_ZEROONFAILURE) != 0 {
        ZeroMemory(pvData, *pcbData as usize);
    }

    if !pdwType.is_null() {
        *pdwType = dwType;
    }
    if !pcbData.is_null() {
        *pcbData = cbData;
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn RegSetKeyValueW(
    hKey: HKEY,
    lpSubKey: LPCWSTR,
    lpValueName: LPCWSTR,
    dwType: DWORD,
    lpData: LPCVOID,
    cbData: DWORD,
) -> LONG {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut CurKey: HANDLE;
    let mut SubKeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Ret: LONG;

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    'cleanup: {
        if !lpSubKey.is_null() {
            let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
            let mut SubKeyName: _UNICODE_STRING = core::mem::zeroed();

            RtlInitUnicodeString(&mut SubKeyName, lpSubKey);

            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = KeyHandle;
            ObjectAttributes.ObjectName = &mut SubKeyName;
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = NtOpenKey(&mut SubKeyHandle, KEY_SET_VALUE, &mut ObjectAttributes);
            if !NT_SUCCESS(Status) {
                Ret = RtlNtStatusToDosError(Status) as LONG;
                break 'cleanup;
            }

            CurKey = SubKeyHandle;
        } else {
            CurKey = KeyHandle;
        }

        Ret = RegSetValueExW(
            CurKey as HKEY,
            lpValueName,
            0,
            dwType,
            lpData as *const BYTE,
            cbData,
        );

        if !SubKeyHandle.is_null() {
            NtClose(SubKeyHandle);
        }
    }

    if (KeyHandle as ULONG_PTR) & 0x1 != 0 {
        NtClose(KeyHandle);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn RegSetKeyValueA(
    hKey: HKEY,
    lpSubKey: LPCSTR,
    lpValueName: LPCSTR,
    dwType: DWORD,
    lpData: LPCVOID,
    cbData: DWORD,
) -> LONG {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut CurKey: HANDLE;
    let mut SubKeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let Ret: LONG;

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    'cleanup: {
        if lpSubKey != core::ptr::null_mut() {
            let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
            let mut SubKeyName: _UNICODE_STRING = core::mem::zeroed();

            if RtlCreateUnicodeStringFromAsciiz(&mut SubKeyName, lpSubKey) == 0 {
                Ret = ERROR_NOT_ENOUGH_MEMORY as LONG;
                break 'cleanup;
            }

            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = KeyHandle;
            ObjectAttributes.ObjectName = &mut SubKeyName;
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = NtOpenKey(&mut SubKeyHandle, KEY_SET_VALUE, &mut ObjectAttributes);

            RtlFreeUnicodeString(&mut SubKeyName);

            if !NT_SUCCESS(Status) {
                Ret = RtlNtStatusToDosError(Status) as LONG;
                break 'cleanup;
            }

            CurKey = SubKeyHandle;
        } else {
            CurKey = KeyHandle;
        }

        Ret = RegSetValueExA(
            CurKey as HKEY,
            lpValueName,
            0,
            dwType,
            lpData as *const BYTE,
            cbData,
        );

        if SubKeyHandle != core::ptr::null_mut() {
            NtClose(SubKeyHandle);
        }
    }

    if (KeyHandle as ULONG_PTR) & 0x1 != 0 {
        NtClose(KeyHandle);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn RegDeleteValueA(hKey: HKEY, lpValueName: LPCSTR) -> LONG {
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ErrorCode: LONG = ERROR_SUCCESS as LONG;

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    if RtlCreateUnicodeStringFromAsciiz(&mut ValueName, lpValueName) == 0 {
        if (KeyHandle as ULONG_PTR) & 0x1 != 0 {
            NtClose(KeyHandle);
        }
        return ERROR_NOT_ENOUGH_MEMORY as LONG;
    }

    if IsHKCRKey(KeyHandle as HKEY) != 0 {
        ErrorCode = DeleteHKCRValue(KeyHandle as HKEY, &mut ValueName);
    } else {
        Status = NtDeleteValueKey(KeyHandle, &mut ValueName);
        if !NT_SUCCESS(Status) {
            ErrorCode = RtlNtStatusToDosError(Status) as LONG;
        }
    }
    RtlFreeUnicodeString(&mut ValueName);
    if (KeyHandle as ULONG_PTR) & 0x1 != 0 {
        NtClose(KeyHandle);
    }
    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegDeleteValueW(hKey: HKEY, lpValueName: LPCWSTR) -> LONG {
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ErrorCode: LONG = ERROR_SUCCESS as LONG;

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    RtlInitUnicodeString(&mut ValueName, lpValueName);

    if IsHKCRKey(KeyHandle as HKEY) != 0 {
        ErrorCode = DeleteHKCRValue(KeyHandle as HKEY, &mut ValueName);
    } else {
        Status = NtDeleteValueKey(KeyHandle, &mut ValueName);
        if !NT_SUCCESS(Status) {
            ErrorCode = RtlNtStatusToDosError(Status) as LONG;
        }
    }

    if (KeyHandle as ULONG_PTR) & 0x1 != 0 {
        NtClose(KeyHandle);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegEnumKeyA(
    hKey: HKEY,
    dwIndex: DWORD,
    lpName: LPSTR,
    cbName: DWORD,
) -> LONG {
    let mut dwLength: DWORD;

    dwLength = cbName;
    RegEnumKeyExA(
        hKey,
        dwIndex,
        lpName,
        &mut dwLength,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn RegEnumKeyW(
    hKey: HKEY,
    dwIndex: DWORD,
    lpName: LPWSTR,
    cbName: DWORD,
) -> LONG {
    let mut dwLength: DWORD;

    dwLength = cbName;
    RegEnumKeyExW(
        hKey,
        dwIndex,
        lpName,
        &mut dwLength,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn RegEnumKeyExA(
    hKey: HKEY,
    dwIndex: DWORD,
    lpName: LPSTR,
    lpcbName: LPDWORD,
    lpReserved: LPDWORD,
    lpClass: LPSTR,
    lpcbClass: LPDWORD,
    lpftLastWriteTime: PFILETIME,
) -> LONG {
    let mut NameBuffer: *mut u16 = core::ptr::null_mut();
    let mut ClassBuffer: *mut u16 = core::ptr::null_mut();
    let mut NameLength: DWORD = 0;
    let mut ClassLength: DWORD = 0;
    let mut ErrorCode: LONG = ERROR_SUCCESS as LONG;

    'Exit: {
        /* Allocate our buffers */
        if *lpcbName > 0 {
            NameLength = *lpcbName;
            NameBuffer = RtlAllocateHeap(
                RtlGetProcessHeap(),
                0,
                ((*lpcbName as usize) * core::mem::size_of::<u16>()) as SIZE_T,
            ) as *mut u16;
            if NameBuffer.is_null() {
                ErrorCode = ERROR_NOT_ENOUGH_MEMORY as LONG;
                break 'Exit;
            }
        }

        if !lpClass.is_null() {
            if *lpcbClass > 0 {
                ClassLength = *lpcbClass;
                ClassBuffer = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    ((*lpcbClass as usize) * core::mem::size_of::<u16>()) as SIZE_T,
                ) as *mut u16;
                if ClassBuffer.is_null() {
                    ErrorCode = ERROR_NOT_ENOUGH_MEMORY as LONG;
                    break 'Exit;
                }
            }
        }

        /* Do the actual call */
        ErrorCode = RegEnumKeyExW(
            hKey,
            dwIndex,
            NameBuffer,
            lpcbName,
            lpReserved,
            ClassBuffer,
            lpcbClass,
            lpftLastWriteTime,
        );

        if ErrorCode != ERROR_SUCCESS as LONG {
            break 'Exit;
        }

        /* Convert the strings */
        RtlUnicodeToMultiByteN(
            lpName,
            *lpcbName,
            core::ptr::null_mut(),
            NameBuffer,
            *lpcbName * core::mem::size_of::<u16>() as DWORD,
        );
        /* NULL terminate if we can */
        if NameLength > *lpcbName {
            *lpName.add(*lpcbName as usize) = 0;
        }

        if !lpClass.is_null() {
            RtlUnicodeToMultiByteN(
                lpClass,
                *lpcbClass,
                core::ptr::null_mut(),
                NameBuffer,
                *lpcbClass * core::mem::size_of::<u16>() as DWORD,
            );
            if ClassLength > *lpcbClass {
                *lpClass.add(*lpcbClass as usize) = 0;
            }
        }
    }

    if !NameBuffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NameBuffer as PVOID);
    }
    if !ClassBuffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, ClassBuffer as PVOID);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegEnumKeyExW(
    hKey: HKEY,
    dwIndex: DWORD,
    lpName: LPWSTR,
    lpcbName: LPDWORD,
    lpReserved: LPDWORD,
    lpClass: LPWSTR,
    lpcbClass: LPDWORD,
    lpftLastWriteTime: PFILETIME,
) -> LONG {
    let mut BufferSize: ULONG;
    let mut ResultSize: ULONG = 0;
    let NameLength: ULONG;
    let mut ClassLength: ULONG = 0;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ErrorCode: LONG = ERROR_SUCCESS as LONG;
    let mut Status: NTSTATUS;

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    if IsHKCRKey(KeyHandle as HKEY) != 0 {
        ErrorCode = EnumHKCRKey(
            KeyHandle as HKEY,
            dwIndex,
            lpName,
            lpcbName,
            lpReserved,
            lpClass,
            lpcbClass,
            lpftLastWriteTime,
        );
        ClosePredefKey(KeyHandle);
        return ErrorCode;
    }

    if *lpcbName > 0 {
        NameLength = min(*lpcbName - 1, REG_MAX_NAME_SIZE) * core::mem::size_of::<u16>() as ULONG;
    } else {
        NameLength = 0;
    }

    if !lpClass.is_null() {
        if *lpcbClass > 0 {
            ClassLength = min(*lpcbClass - 1, REG_MAX_NAME_SIZE) * core::mem::size_of::<u16>() as ULONG;
        } else {
            ClassLength = 0;
        }

        BufferSize = ((core::mem::size_of::<KEY_NODE_INFORMATION>() as ULONG + NameLength + 3) & !3) + ClassLength;
    } else {
        BufferSize = core::mem::size_of::<KEY_BASIC_INFORMATION>() as ULONG + NameLength;
    }

    let KeyInfo = RtlAllocateHeap(ProcessHeap, 0, BufferSize as SIZE_T);
    if KeyInfo.is_null() {
        ErrorCode = ERROR_OUTOFMEMORY as LONG;
    } else {
        let Node = KeyInfo as *mut KEY_NODE_INFORMATION;
        let Basic = KeyInfo as *mut KEY_BASIC_INFORMATION;

        Status = NtEnumerateKey(
            KeyHandle,
            dwIndex as ULONG,
            if !lpClass.is_null() { KeyNodeInformation } else { KeyBasicInformation },
            KeyInfo,
            BufferSize,
            &mut ResultSize,
        );
        if !NT_SUCCESS(Status) {
            ErrorCode = RtlNtStatusToDosError(Status) as LONG;
        } else {
            if lpClass.is_null() {
                if (*Basic).NameLength > NameLength {
                    ErrorCode = ERROR_MORE_DATA as LONG;
                } else {
                    RtlCopyMemory(
                        lpName as *mut core::ffi::c_void,
                        core::ptr::addr_of!((*Basic).Name) as *const core::ffi::c_void,
                        (*Basic).NameLength as usize,
                    );
                    *lpcbName = (*Basic).NameLength / core::mem::size_of::<u16>() as ULONG;
                    *lpName.add(*lpcbName as usize) = 0;
                }
            } else {
                if (*Node).NameLength > NameLength || (*Node).ClassLength > ClassLength {
                    ErrorCode = ERROR_MORE_DATA as LONG;
                } else {
                    RtlCopyMemory(
                        lpName as *mut core::ffi::c_void,
                        core::ptr::addr_of!((*Node).Name) as *const core::ffi::c_void,
                        (*Node).NameLength as usize,
                    );
                    *lpcbName = (*Node).NameLength / core::mem::size_of::<u16>() as ULONG;
                    *lpName.add(*lpcbName as usize) = 0;
                    RtlCopyMemory(
                        lpClass as *mut core::ffi::c_void,
                        (core::ptr::addr_of!((*Node).Name) as ULONG_PTR + (*Node).ClassOffset as ULONG_PTR) as *const core::ffi::c_void,
                        (*Node).ClassLength as usize,
                    );
                    *lpcbClass = (*Node).ClassLength / core::mem::size_of::<u16>() as ULONG;
                    *lpClass.add(*lpcbClass as usize) = 0;
                }
            }

            if ErrorCode == ERROR_SUCCESS as LONG && !lpftLastWriteTime.is_null() {
                if lpClass.is_null() {
                    (*lpftLastWriteTime).dwLowDateTime = (*Basic).LastWriteTime.u.LowPart;
                    (*lpftLastWriteTime).dwHighDateTime = (*Basic).LastWriteTime.u.HighPart as u32;
                } else {
                    (*lpftLastWriteTime).dwLowDateTime = (*Node).LastWriteTime.u.LowPart;
                    (*lpftLastWriteTime).dwHighDateTime = (*Node).LastWriteTime.u.HighPart as u32;
                }
            }
        }

        RtlFreeHeap(ProcessHeap, 0, KeyInfo);
    }

    ClosePredefKey(KeyHandle);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegEnumValueA(
    hKey: HKEY,
    dwIndex: DWORD,
    lpName: LPSTR,
    lpcbName: LPDWORD,
    lpdwReserved: LPDWORD,
    lpdwType: LPDWORD,
    lpData: LPBYTE,
    lpcbData: LPDWORD,
) -> LONG {
    let mut NameBuffer: *mut u16;
    let mut NameBufferSize: DWORD;
    let mut NameLength: DWORD;
    let mut ErrorCode: LONG;
    let mut LocalType: DWORD = REG_NONE;
    let mut NameOverflow: u8 = 0u8;

    if lpName.is_null() || lpcbName.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    if (!lpData.is_null() && lpcbData.is_null()) || !lpdwReserved.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    NameBufferSize = 0;
    ErrorCode = RegQueryInfoKeyW(
        hKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut NameBufferSize,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if ErrorCode != ERROR_SUCCESS as LONG {
        return ErrorCode;
    }

    NameBufferSize += 1;

    NameBuffer = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (NameBufferSize as usize * core::mem::size_of::<u16>()) as SIZE_T,
    ) as *mut u16;
    if NameBuffer.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY as LONG;
    }

    'exit: {
        NameLength = *lpcbName;
        ErrorCode = RegEnumValueW(
            hKey,
            dwIndex,
            NameBuffer,
            &mut NameLength,
            core::ptr::null_mut(),
            &mut LocalType,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if ErrorCode != ERROR_SUCCESS as LONG {
            if ErrorCode == ERROR_MORE_DATA as LONG {
                NameOverflow = 1u8;
            } else {
                break 'exit;
            }
        }

        if is_string(LocalType) != 0 && !lpcbData.is_null() {
            let mut DataBufferSize: DWORD = *lpcbData * core::mem::size_of::<u16>() as DWORD;
            let mut DataBuffer: *mut u16 = core::ptr::null_mut();
            let DataLength: DWORD = *lpcbData;
            let DataStr: LPSTR = lpData as LPSTR;

            if !lpData.is_null() {
                DataBuffer = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    (*lpcbData as usize * core::mem::size_of::<u16>()) as SIZE_T,
                ) as *mut u16;
            }

            ErrorCode = RegEnumValueW(
                hKey,
                dwIndex,
                NameBuffer,
                &mut NameBufferSize,
                lpdwReserved,
                lpdwType,
                DataBuffer as LPBYTE,
                &mut DataBufferSize,
            );

            *lpcbData = DataBufferSize / core::mem::size_of::<u16>() as DWORD;

            if ErrorCode != ERROR_SUCCESS as LONG {
                RtlFreeHeap(RtlGetProcessHeap(), 0, DataBuffer as PVOID);
                break 'exit;
            }

            if !lpData.is_null() {
                RtlUnicodeToMultiByteN(
                    DataStr,
                    DataLength,
                    core::ptr::null_mut(),
                    DataBuffer,
                    DataBufferSize,
                );
                if DataLength > *lpcbData
                    && *DataStr.offset((*lpcbData - 1) as isize) != 0
                {
                    *DataStr.offset(*lpcbData as isize) = 0;
                }
            }

            RtlFreeHeap(RtlGetProcessHeap(), 0, DataBuffer as PVOID);
        } else {
            ErrorCode = RegEnumValueW(
                hKey,
                dwIndex,
                NameBuffer,
                &mut NameBufferSize,
                lpdwReserved,
                lpdwType,
                lpData,
                lpcbData,
            );

            if ErrorCode != ERROR_SUCCESS as LONG {
                break 'exit;
            }
        }

        if NameOverflow != 0 {
            ErrorCode = ERROR_MORE_DATA as LONG;
            break 'exit;
        }

        RtlUnicodeToMultiByteN(
            lpName,
            *lpcbName,
            lpcbName,
            NameBuffer,
            NameBufferSize * core::mem::size_of::<u16>() as DWORD,
        );
        *lpName.offset(*lpcbName as isize) = 0;
    }

    if !NameBuffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, NameBuffer as PVOID);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegEnumValueW(
    hKey: HKEY,
    index: DWORD,
    value: LPWSTR,
    val_count: PDWORD,
    reserved: PDWORD,
    type_: PDWORD,
    data: LPBYTE,
    count: PDWORD,
) -> LONG {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut total_size: ULONG;
    let mut buffer: [core::ffi::c_char; 256] = [0; 256];
    let mut buf_ptr: *mut core::ffi::c_char = buffer.as_mut_ptr();
    let mut info: *mut KEY_VALUE_FULL_INFORMATION =
        buffer.as_mut_ptr() as *mut KEY_VALUE_FULL_INFORMATION;
    let info_size: core::ffi::c_int =
        core::mem::offset_of!(KEY_VALUE_FULL_INFORMATION, Name) as core::ffi::c_int;

    if value.is_null() || val_count.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    if (!data.is_null() && count.is_null()) || !reserved.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(status) {
        return RtlNtStatusToDosError(status) as LONG;
    }

    if IsHKCRKey(KeyHandle as HKEY) != 0 {
        let ErrorCode = EnumHKCRValue(
            KeyHandle as HKEY, index, value, val_count, reserved, type_, data, count,
        );
        ClosePredefKey(KeyHandle);
        return ErrorCode;
    }

    total_size = info_size as ULONG
        + (MAX_PATH + 1) as ULONG * core::mem::size_of::<u16>() as ULONG;
    if !data.is_null() {
        total_size += *count;
    }
    let buf_size = core::mem::size_of::<[core::ffi::c_char; 256]>() as ULONG;
    total_size = if buf_size < total_size { buf_size } else { total_size };

    status = NtEnumerateValueKey(
        KeyHandle,
        index,
        KeyValueFullInformation,
        buffer.as_mut_ptr() as *mut core::ffi::c_void,
        total_size,
        &mut total_size,
    );

    'done: loop {
        if status != 0
            && status != STATUS_BUFFER_OVERFLOW
            && status != STATUS_BUFFER_TOO_SMALL
        {
            break 'done;
        }

        'overflow: {
            if !value.is_null() || !data.is_null() {
                while status == STATUS_BUFFER_OVERFLOW || status == STATUS_BUFFER_TOO_SMALL {
                    if buf_ptr != buffer.as_mut_ptr() {
                        HeapFree(GetProcessHeap(), 0, buf_ptr as *mut core::ffi::c_void);
                    }
                    buf_ptr = HeapAlloc(GetProcessHeap(), 0, total_size as SIZE_T)
                        as *mut core::ffi::c_char;
                    if buf_ptr.is_null() {
                        status = ERROR_NOT_ENOUGH_MEMORY as NTSTATUS;
                        break 'done;
                    }
                    info = buf_ptr as *mut KEY_VALUE_FULL_INFORMATION;
                    status = NtEnumerateValueKey(
                        KeyHandle,
                        index,
                        KeyValueFullInformation,
                        buf_ptr as *mut core::ffi::c_void,
                        total_size,
                        &mut total_size,
                    );
                }

                if status != 0 {
                    break 'done;
                }

                if !value.is_null() {
                    if (*info).NameLength as usize / core::mem::size_of::<u16>()
                        >= *val_count as usize
                    {
                        status = STATUS_BUFFER_OVERFLOW;
                        break 'overflow;
                    }
                    memcpy(
                        value as *mut core::ffi::c_void,
                        core::ptr::addr_of!((*info).Name) as *const core::ffi::c_void,
                        (*info).NameLength as usize,
                    );
                    *val_count = (*info).NameLength / core::mem::size_of::<u16>() as ULONG;
                    *value.offset(*val_count as isize) = 0;
                }

                if !data.is_null() {
                    if (*info).DataLength > *count {
                        status = STATUS_BUFFER_OVERFLOW;
                        break 'overflow;
                    }
                    memcpy(
                        data as *mut core::ffi::c_void,
                        buf_ptr.offset((*info).DataOffset as isize) as *const core::ffi::c_void,
                        (*info).DataLength as usize,
                    );
                    if is_string((*info).Type) != 0
                        && (*info).DataLength as usize
                            <= *count as usize - core::mem::size_of::<u16>()
                    {
                        let ptr = data.offset((*info).DataLength as isize) as *mut u16;
                        if ptr > data as *mut u16 && *ptr.offset(-1) != 0 {
                            *ptr = 0;
                        }
                    }
                }
            } else {
                status = STATUS_SUCCESS;
            }
        }

        if !type_.is_null() {
            *type_ = (*info).Type;
        }
        if !count.is_null() {
            *count = (*info).DataLength;
        }

        break 'done;
    }

    if buf_ptr != buffer.as_mut_ptr() {
        HeapFree(GetProcessHeap(), 0, buf_ptr as *mut core::ffi::c_void);
    }
    ClosePredefKey(KeyHandle);
    RtlNtStatusToDosError(status) as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegFlushKey(hKey: HKEY) -> LONG {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if SamePredefKey(hKey, HKEY_PERFORMANCE_DATA) {
        return ERROR_SUCCESS as LONG;
    }

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    Status = NtFlushKey(KeyHandle);

    if (KeyHandle as ULONG_PTR) & 0x1 != 0 {
        NtClose(KeyHandle);
    }

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegGetKeySecurity(
    hKey: HKEY,
    SecurityInformation: SECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    lpcbSecurityDescriptor: LPDWORD,
) -> LONG {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if SamePredefKey(hKey, HKEY_PERFORMANCE_DATA) {
        return ERROR_INVALID_HANDLE as LONG;
    }

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    Status = NtQuerySecurityObject(
        KeyHandle,
        SecurityInformation,
        pSecurityDescriptor,
        *lpcbSecurityDescriptor,
        lpcbSecurityDescriptor,
    );

    ClosePredefKey(KeyHandle);

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegLoadKeyA(
    hKey: HKEY,
    lpSubKey: LPCSTR,
    lpFile: LPCSTR,
) -> LONG {
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let ErrorCode: LONG;

    RtlInitEmptyUnicodeString(&mut KeyName, core::ptr::null_mut(), 0);
    RtlInitEmptyUnicodeString(&mut FileName, core::ptr::null_mut(), 0);

    'exit: {
        if !lpSubKey.is_null() {
            if RtlCreateUnicodeStringFromAsciiz(&mut KeyName, lpSubKey) == 0 {
                ErrorCode = ERROR_NOT_ENOUGH_MEMORY as LONG;
                break 'exit;
            }
        }

        if !lpFile.is_null() {
            if RtlCreateUnicodeStringFromAsciiz(&mut FileName, lpFile) == 0 {
                ErrorCode = ERROR_NOT_ENOUGH_MEMORY as LONG;
                break 'exit;
            }
        }

        ErrorCode = RegLoadKeyW(hKey, KeyName.Buffer, FileName.Buffer);
    }

    RtlFreeUnicodeString(&mut FileName);
    RtlFreeUnicodeString(&mut KeyName);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegLoadKeyW(
    hKey: HKEY,
    lpSubKey: LPCWSTR,
    lpFile: LPCWSTR,
) -> LONG {
    let mut FileObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ErrorCode: LONG = ERROR_SUCCESS as LONG;

    if SamePredefKey(hKey, HKEY_PERFORMANCE_DATA) {
        return ERROR_INVALID_HANDLE as LONG;
    }

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    'cleanup: {
        if RtlDosPathNameToNtPathName_U(
            lpFile,
            &mut FileName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            ErrorCode = ERROR_BAD_PATHNAME as LONG;
            break 'cleanup;
        }

        InitializeObjectAttributes(
            &mut FileObjectAttributes,
            &mut FileName,
            OBJ_CASE_INSENSITIVE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        RtlInitUnicodeString(&mut KeyName, lpSubKey);

        InitializeObjectAttributes(
            &mut KeyObjectAttributes,
            &mut KeyName,
            OBJ_CASE_INSENSITIVE,
            KeyHandle,
            core::ptr::null_mut(),
        );

        Status = NtLoadKey(&mut KeyObjectAttributes, &mut FileObjectAttributes);

        RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);

        if !NT_SUCCESS(Status) {
            ErrorCode = RtlNtStatusToDosError(Status) as LONG;
            break 'cleanup;
        }
    }

    ClosePredefKey(KeyHandle);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegNotifyChangeKeyValue(
    hKey: HKEY,
    bWatchSubtree: BOOL,
    dwNotifyFilter: DWORD,
    hEvent: HANDLE,
    fAsynchronous: BOOL,
) -> LONG {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ErrorCode: LONG = ERROR_SUCCESS as LONG;

    if SamePredefKey(hKey, HKEY_PERFORMANCE_DATA) {
        return ERROR_INVALID_HANDLE as LONG;
    }

    if (fAsynchronous != 0) && (hEvent.is_null()) {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    Status = NtNotifyChangeKey(
        KeyHandle,
        hEvent,
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        dwNotifyFilter,
        bWatchSubtree as BOOLEAN,
        core::ptr::null_mut(),
        0,
        fAsynchronous as BOOLEAN,
    );
    if !NT_SUCCESS(Status) && Status != STATUS_TIMEOUT {
        ErrorCode = RtlNtStatusToDosError(Status) as LONG;
    }

    if (KeyHandle as ULONG_PTR) & 0x1 != 0 {
        NtClose(KeyHandle);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegOpenCurrentUser(samDesired: REGSAM, phkResult: PHKEY) -> LONG {
    let Status: NTSTATUS = RtlOpenCurrentUser(samDesired as ACCESS_MASK, phkResult as PHANDLE);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegOpenKeyA(
    hKey: HKEY,
    lpSubKey: LPCSTR,
    phkResult: PHKEY,
) -> LONG {
    if phkResult.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    if hKey.is_null() && lpSubKey.is_null() {
        *phkResult = hKey;
        return ERROR_SUCCESS as LONG;
    }

    RegOpenKeyExA(hKey, lpSubKey, 0, MAXIMUM_ALLOWED as REGSAM, phkResult)
}

#[no_mangle]
pub unsafe extern "system" fn RegOpenKeyW(
    hKey: HKEY,
    lpSubKey: LPCWSTR,
    phkResult: PHKEY,
) -> LONG {
    if phkResult.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    if hKey.is_null() && lpSubKey.is_null() {
        *phkResult = hKey;
        return ERROR_SUCCESS as LONG;
    }

    RegOpenKeyExW(hKey, lpSubKey, 0, MAXIMUM_ALLOWED as REGSAM, phkResult)
}

#[no_mangle]
pub unsafe extern "system" fn RegOpenKeyExA(
    hKey: HKEY,
    lpSubKey: LPCSTR,
    ulOptions: DWORD,
    samDesired: REGSAM,
    phkResult: PHKEY,
) -> LONG {
    let mut SubKeyString: _UNICODE_STRING = core::mem::zeroed();
    let ErrorCode: LONG;

    if !lpSubKey.is_null() {
        if RtlCreateUnicodeStringFromAsciiz(&mut SubKeyString, lpSubKey) == 0 {
            return ERROR_NOT_ENOUGH_MEMORY as LONG;
        }
    } else {
        SubKeyString.Length = 0;
        SubKeyString.MaximumLength = 0;
        SubKeyString.Buffer = core::ptr::null_mut();
    }

    ErrorCode = RegOpenKeyExW(hKey, SubKeyString.Buffer, ulOptions, samDesired, phkResult);

    RtlFreeUnicodeString(&mut SubKeyString);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegOpenKeyExW(
    hKey: HKEY,
    lpSubKey: LPCWSTR,
    ulOptions: DWORD,
    samDesired: REGSAM,
    phkResult: PHKEY,
) -> LONG {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SubKeyString: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Attributes: ULONG = OBJ_CASE_INSENSITIVE;
    let mut ErrorCode: LONG = ERROR_SUCCESS as LONG;
    let mut SubKeyStringAllocated: BOOLEAN = 0u8;

    if phkResult.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    if hKey.is_null() && !lpSubKey.is_null() && !phkResult.is_null() {
        return ERROR_INVALID_HANDLE as LONG;
    }

    if IsPredefKey(hKey) != 0 && (lpSubKey.is_null() || *lpSubKey == 0) {
        *phkResult = hKey;
        return ERROR_SUCCESS as LONG;
    }

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    if IsHKCRKey(KeyHandle as HKEY) != 0 {
        ErrorCode = OpenHKCRKey(KeyHandle as HKEY, lpSubKey, ulOptions, samDesired, phkResult);
        ClosePredefKey(KeyHandle);
        return ErrorCode;
    }

    if ulOptions & REG_OPTION_OPEN_LINK != 0 {
        Attributes |= OBJ_OPENLINK;
    }

    if lpSubKey.is_null()
        || wcscmp(lpSubKey, crate::wide!("\\").as_ptr()) == 0
    {
        RtlInitUnicodeString(&mut SubKeyString, [0u16].as_ptr());
    } else {
        RtlInitUnicodeString(&mut SubKeyString, lpSubKey);

        if (lpSubKey as usize) & 1 != 0 {
            let mut AlignedString: _UNICODE_STRING = core::mem::zeroed();

            Status = RtlDuplicateUnicodeString(
                RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE as ULONG,
                &mut SubKeyString,
                &mut AlignedString,
            );
            if !NT_SUCCESS(Status) {
                'exit: {
                    break 'exit;
                }
                if SubKeyStringAllocated != 0 {
                    RtlFreeUnicodeString(&mut SubKeyString);
                }
                if !NT_SUCCESS(Status) {
                    ErrorCode = RtlNtStatusToDosError(Status) as LONG;
                }
                ClosePredefKey(KeyHandle);
                return ErrorCode;
            }

            SubKeyString = AlignedString;
            SubKeyStringAllocated = 1u8;
        }
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = KeyHandle;
    ObjectAttributes.ObjectName = &mut SubKeyString;
    ObjectAttributes.Attributes = Attributes;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(phkResult as PHANDLE, samDesired, &mut ObjectAttributes);

    if SubKeyStringAllocated != 0 {
        RtlFreeUnicodeString(&mut SubKeyString);
    }

    if !NT_SUCCESS(Status) {
        ErrorCode = RtlNtStatusToDosError(Status) as LONG;
    }

    ClosePredefKey(KeyHandle);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegOpenUserClassesRoot(
    hToken: HANDLE,
    dwOptions: DWORD,
    samDesired: REGSAM,
    phkResult: PHKEY,
) -> LONG {
    let UserClassesKeyPrefix: [u16; 15] = [
        0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C, 0x0055,
        0x0073, 0x0065, 0x0072, 0x005C,
    ];
    let UserClassesKeySuffix: [u16; 9] = [
        0x005F, 0x0043, 0x006C, 0x0061, 0x0073, 0x0073, 0x0065, 0x0073, 0x0000,
    ];
    let mut TokenUserData: PTOKEN_USER;
    let mut RequiredLength: ULONG = 0;
    let mut UserSidString: _UNICODE_STRING = core::mem::zeroed();
    let mut UserClassesKeyRoot: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if hToken.is_null() || dwOptions != 0 || phkResult.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    loop {
        Status = NtQueryInformationToken(
            hToken,
            TokenUser,
            core::ptr::null_mut(),
            0,
            &mut RequiredLength,
        );
        if !NT_SUCCESS(Status) && Status != STATUS_BUFFER_TOO_SMALL {
            return RtlNtStatusToDosError(Status) as LONG;
        }

        RegInitialize();

        TokenUserData = RtlAllocateHeap(ProcessHeap, 0, RequiredLength as SIZE_T) as PTOKEN_USER;
        if TokenUserData.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY as LONG;
        }

        Status = NtQueryInformationToken(
            hToken,
            TokenUser,
            TokenUserData as *mut _,
            RequiredLength,
            &mut RequiredLength,
        );
        if !NT_SUCCESS(Status) {
            RtlFreeHeap(ProcessHeap, 0, TokenUserData as *mut _);
            if Status == STATUS_BUFFER_TOO_SMALL {
                continue;
            }
            return RtlNtStatusToDosError(Status) as LONG;
        }

        break;
    }

    Status = RtlConvertSidToUnicodeString(
        &mut UserSidString,
        (*TokenUserData).User.Sid,
        1u8,
    );

    RtlFreeHeap(ProcessHeap, 0, TokenUserData as *mut _);

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    UserClassesKeyRoot.Length = 0;
    UserClassesKeyRoot.MaximumLength = (UserSidString.Length as usize
        + core::mem::size_of_val(&UserClassesKeyPrefix)
        + core::mem::size_of_val(&UserClassesKeySuffix)) as u16;
    UserClassesKeyRoot.Buffer =
        RtlAllocateHeap(ProcessHeap, 0, UserClassesKeyRoot.MaximumLength as SIZE_T) as PWSTR;
    if UserClassesKeyRoot.Buffer.is_null() {
        RtlFreeUnicodeString(&mut UserSidString);
        return RtlNtStatusToDosError(Status) as LONG;
    }

    RtlAppendUnicodeToString(&mut UserClassesKeyRoot, UserClassesKeyPrefix.as_ptr());
    RtlAppendUnicodeStringToString(&mut UserClassesKeyRoot, &mut UserSidString);
    RtlAppendUnicodeToString(&mut UserClassesKeyRoot, UserClassesKeySuffix.as_ptr());

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut UserClassesKeyRoot;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(
        phkResult as *mut HANDLE,
        samDesired,
        &mut ObjectAttributes,
    );

    RtlFreeUnicodeString(&mut UserSidString);
    RtlFreeUnicodeString(&mut UserClassesKeyRoot);

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegQueryInfoKeyA(
    hKey: HKEY,
    lpClass: LPSTR,
    lpcClass: LPDWORD,
    lpReserved: LPDWORD,
    lpcSubKeys: LPDWORD,
    lpcMaxSubKeyLen: LPDWORD,
    lpcMaxClassLen: LPDWORD,
    lpcValues: LPDWORD,
    lpcMaxValueNameLen: LPDWORD,
    lpcMaxValueLen: LPDWORD,
    lpcbSecurityDescriptor: LPDWORD,
    lpftLastWriteTime: PFILETIME,
) -> LONG {
    let mut ClassName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut ErrorCode: LONG;
    let Status: NTSTATUS;
    let mut cClass: u32 = 0;

    if !lpClass.is_null() && lpcClass.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    RtlInitUnicodeString(&mut UnicodeString, core::ptr::null_mut());
    if !lpClass.is_null() {
        RtlInitEmptyUnicodeString(
            &mut UnicodeString,
            ClassName.as_mut_ptr(),
            core::mem::size_of::<[u16; MAX_PATH as usize]>() as u16,
        );
        cClass = (core::mem::size_of::<[u16; MAX_PATH as usize]>() / core::mem::size_of::<u16>()) as u32;
    }

    ErrorCode = RegQueryInfoKeyW(
        hKey,
        UnicodeString.Buffer,
        &mut cClass,
        lpReserved,
        lpcSubKeys,
        lpcMaxSubKeyLen,
        lpcMaxClassLen,
        lpcValues,
        lpcMaxValueNameLen,
        lpcMaxValueLen,
        lpcbSecurityDescriptor,
        lpftLastWriteTime,
    );

    if ErrorCode == ERROR_SUCCESS as LONG && !lpClass.is_null() {
        if *lpcClass == 0 {
            return ErrorCode;
        }

        RtlInitEmptyAnsiString(&mut AnsiString, lpClass, *lpcClass as u16);
        UnicodeString.Length = (cClass * core::mem::size_of::<u16>() as u32) as u16;
        Status = RtlUnicodeStringToAnsiString(&mut AnsiString, &mut UnicodeString, 0u8);
        ErrorCode = RtlNtStatusToDosError(Status) as LONG;
        cClass = AnsiString.Length as u32;
        *lpClass.add(cClass as usize) = 0;
    }

    if !lpcClass.is_null() {
        *lpcClass = cClass;
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegQueryInfoKeyW(
    hKey: HKEY,
    lpClass: LPWSTR,
    lpcClass: LPDWORD,
    lpReserved: LPDWORD,
    lpcSubKeys: LPDWORD,
    lpcMaxSubKeyLen: LPDWORD,
    lpcMaxClassLen: LPDWORD,
    lpcValues: LPDWORD,
    lpcMaxValueNameLen: LPDWORD,
    lpcMaxValueLen: LPDWORD,
    lpcbSecurityDescriptor: LPDWORD,
    lpftLastWriteTime: PFILETIME,
) -> LONG {
    let mut FullInfoBuffer: KEY_FULL_INFORMATION = core::mem::zeroed();
    let FullInfo: PKEY_FULL_INFORMATION;
    let FullInfoSize: ULONG;
    let mut ClassLength: ULONG = 0;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Length: ULONG = 0;
    let mut ErrorCode: LONG = ERROR_SUCCESS as LONG;

    if !lpClass.is_null() && lpcClass.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    if IsHKCRKey(KeyHandle as HKEY) != 0 {
        ErrorCode = QueryInfoHKCRKey(
            KeyHandle as HKEY,
            lpClass,
            lpcClass,
            lpReserved,
            lpcSubKeys,
            lpcMaxSubKeyLen,
            lpcMaxClassLen,
            lpcValues,
            lpcMaxValueNameLen,
            lpcMaxValueLen,
            lpcbSecurityDescriptor,
            lpftLastWriteTime,
        );
        ClosePredefKey(KeyHandle);
        return ErrorCode;
    }

    if !lpClass.is_null() {
        if *lpcClass > 0 {
            let m = if *lpcClass - 1 < REG_MAX_NAME_SIZE as DWORD {
                *lpcClass - 1
            } else {
                REG_MAX_NAME_SIZE as DWORD
            };
            ClassLength = m * core::mem::size_of::<u16>() as ULONG;
        } else {
            ClassLength = 0;
        }

        FullInfoSize = core::mem::size_of::<KEY_FULL_INFORMATION>() as ULONG
            + ((ClassLength + 3) & !3);
        FullInfo = RtlAllocateHeap(ProcessHeap, 0, FullInfoSize as SIZE_T)
            as PKEY_FULL_INFORMATION;
        if FullInfo.is_null() {
            ErrorCode = ERROR_OUTOFMEMORY as LONG;
            if !lpClass.is_null() {
                RtlFreeHeap(ProcessHeap, 0, FullInfo as *mut _);
            }
            ClosePredefKey(KeyHandle);
            return ErrorCode;
        }
    } else {
        FullInfoSize = core::mem::size_of::<KEY_FULL_INFORMATION>() as ULONG;
        FullInfo = &mut FullInfoBuffer;
    }

    'cleanup: {
        if !lpcbSecurityDescriptor.is_null() {
            *lpcbSecurityDescriptor = 0;
        }

        Status = NtQueryKey(
            KeyHandle,
            KeyFullInformation,
            FullInfo as *mut _,
            FullInfoSize,
            &mut Length,
        );
        if !NT_SUCCESS(Status) && Status != STATUS_BUFFER_OVERFLOW {
            ErrorCode = RtlNtStatusToDosError(Status) as LONG;
            break 'cleanup;
        }

        if !lpcSubKeys.is_null() {
            *lpcSubKeys = (*FullInfo).SubKeys;
        }

        if !lpcMaxSubKeyLen.is_null() {
            *lpcMaxSubKeyLen = (*FullInfo).MaxNameLen / core::mem::size_of::<u16>() as ULONG;
        }

        if !lpcMaxClassLen.is_null() {
            *lpcMaxClassLen = (*FullInfo).MaxClassLen / core::mem::size_of::<u16>() as ULONG;
        }

        if !lpcValues.is_null() {
            *lpcValues = (*FullInfo).Values;
        }

        if !lpcMaxValueNameLen.is_null() {
            *lpcMaxValueNameLen =
                (*FullInfo).MaxValueNameLen / core::mem::size_of::<u16>() as ULONG;
        }

        if !lpcMaxValueLen.is_null() {
            *lpcMaxValueLen = (*FullInfo).MaxValueDataLen;
        }

        if !lpcbSecurityDescriptor.is_null() {
            Status = NtQuerySecurityObject(
                KeyHandle,
                OWNER_SECURITY_INFORMATION
                    | GROUP_SECURITY_INFORMATION
                    | DACL_SECURITY_INFORMATION,
                core::ptr::null_mut(),
                0,
                lpcbSecurityDescriptor,
            );
        }

        if !lpftLastWriteTime.is_null() {
            (*lpftLastWriteTime).dwLowDateTime = (*FullInfo).LastWriteTime.u.LowPart;
            (*lpftLastWriteTime).dwHighDateTime = (*FullInfo).LastWriteTime.u.HighPart as DWORD;
        }

        if !lpClass.is_null() {
            if *lpcClass == 0 {
                break 'cleanup;
            }

            if (*FullInfo).ClassLength > ClassLength {
                ErrorCode = ERROR_INSUFFICIENT_BUFFER as LONG;
            } else {
                RtlCopyMemory(
                    lpClass as *mut _,
                    core::ptr::addr_of!((*FullInfo).Class) as *const _,
                    (*FullInfo).ClassLength as usize,
                );
                *lpClass.add(((*FullInfo).ClassLength / core::mem::size_of::<u16>() as ULONG) as usize) = 0;
            }
        }

        if !lpcClass.is_null() {
            *lpcClass = (*FullInfo).ClassLength / core::mem::size_of::<u16>() as ULONG;
        }
    }

    if !lpClass.is_null() {
        RtlFreeHeap(ProcessHeap, 0, FullInfo as *mut _);
    }

    ClosePredefKey(KeyHandle);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegQueryMultipleValuesA(
    hKey: HKEY,
    val_list: PVALENTA,
    num_vals: DWORD,
    lpValueBuf: LPSTR,
    ldwTotsize: LPDWORD,
) -> LONG {
    let maxBytes: DWORD = *ldwTotsize;
    let mut bufptr: LPSTR = lpValueBuf;
    let mut ErrorCode: LONG;

    if maxBytes >= (1024 * 1024) {
        return ERROR_MORE_DATA as LONG;
    }

    *ldwTotsize = 0;

    let mut i: ULONG = 0;
    while i < num_vals {
        (*val_list.add(i as usize)).ve_valuelen = 0;
        ErrorCode = RegQueryValueExA(
            hKey,
            (*val_list.add(i as usize)).ve_valuename,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*val_list.add(i as usize)).ve_valuelen),
        );
        if ErrorCode != ERROR_SUCCESS as LONG {
            return ErrorCode;
        }

        if !lpValueBuf.is_null()
            && *ldwTotsize + (*val_list.add(i as usize)).ve_valuelen <= maxBytes
        {
            ErrorCode = RegQueryValueExA(
                hKey,
                (*val_list.add(i as usize)).ve_valuename,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*val_list.add(i as usize)).ve_type),
                bufptr as LPBYTE,
                core::ptr::addr_of_mut!((*val_list.add(i as usize)).ve_valuelen),
            );
            if ErrorCode != ERROR_SUCCESS as LONG {
                return ErrorCode;
            }

            (*val_list.add(i as usize)).ve_valueptr = bufptr as DWORD;

            bufptr = bufptr.add((*val_list.add(i as usize)).ve_valuelen as usize);
        }

        *ldwTotsize += (*val_list.add(i as usize)).ve_valuelen;

        i += 1;
    }

    if !lpValueBuf.is_null() && *ldwTotsize <= maxBytes {
        ERROR_SUCCESS as LONG
    } else {
        ERROR_MORE_DATA as LONG
    }
}

#[no_mangle]
pub unsafe extern "system" fn RegQueryMultipleValuesW(
    hKey: HKEY,
    val_list: PVALENTW,
    num_vals: DWORD,
    lpValueBuf: LPWSTR,
    ldwTotsize: LPDWORD,
) -> LONG {
    let maxBytes: DWORD = *ldwTotsize;
    let mut bufptr: LPSTR = lpValueBuf as LPSTR;
    let mut ErrorCode: LONG;

    if maxBytes >= (1024 * 1024) {
        return ERROR_MORE_DATA as LONG;
    }

    *ldwTotsize = 0;

    let mut i: ULONG = 0;
    while i < num_vals {
        (*val_list.offset(i as isize)).ve_valuelen = 0;
        ErrorCode = RegQueryValueExW(
            hKey,
            (*val_list.offset(i as isize)).ve_valuename,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*val_list.offset(i as isize)).ve_valuelen),
        );
        if ErrorCode != ERROR_SUCCESS as LONG {
            return ErrorCode;
        }

        if !lpValueBuf.is_null()
            && *ldwTotsize + (*val_list.offset(i as isize)).ve_valuelen <= maxBytes
        {
            ErrorCode = RegQueryValueExW(
                hKey,
                (*val_list.offset(i as isize)).ve_valuename,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*val_list.offset(i as isize)).ve_type),
                bufptr as LPBYTE,
                core::ptr::addr_of_mut!((*val_list.offset(i as isize)).ve_valuelen),
            );
            if ErrorCode != ERROR_SUCCESS as LONG {
                return ErrorCode;
            }

            (*val_list.offset(i as isize)).ve_valueptr = bufptr as DWORD;

            bufptr = bufptr.offset((*val_list.offset(i as isize)).ve_valuelen as isize);
        }

        *ldwTotsize += (*val_list.offset(i as isize)).ve_valuelen;

        i += 1;
    }

    if !lpValueBuf.is_null() && *ldwTotsize <= maxBytes {
        ERROR_SUCCESS as LONG
    } else {
        ERROR_MORE_DATA as LONG
    }
}

#[no_mangle]
pub unsafe extern "system" fn RegQueryReflectionKey(
    hBase: HKEY,
    bIsReflectionDisabled: *mut BOOL,
) -> LONG {
    let _ = hBase;
    let _ = bIsReflectionDisabled;
    ERROR_CALL_NOT_IMPLEMENTED as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegQueryValueExA(
    hkeyorg: HKEY,
    name: LPCSTR,
    reserved: LPDWORD,
    type_: LPDWORD,
    data: LPBYTE,
    count: LPDWORD,
) -> LONG {
    let mut nameW: _UNICODE_STRING = core::mem::zeroed();
    let DataLength: DWORD;
    let mut ErrorCode: DWORD;
    let mut BufferSize: DWORD = 0;
    let Buffer: *mut u16;
    let DataStr: *mut core::ffi::c_char = data as *mut core::ffi::c_char;
    let mut LocalType: DWORD = 0;

    if (!data.is_null() && count.is_null()) || !reserved.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    if !name.is_null() {
        if RtlCreateUnicodeStringFromAsciiz(&mut nameW, name) == 0 {
            return ERROR_NOT_ENOUGH_MEMORY as LONG;
        }
    } else {
        nameW.Length = 0;
        nameW.MaximumLength = 0;
        nameW.Buffer = core::ptr::null_mut();
    }

    ErrorCode = RegQueryValueExW(
        hkeyorg,
        nameW.Buffer,
        core::ptr::null_mut(),
        &mut LocalType,
        core::ptr::null_mut(),
        &mut BufferSize,
    ) as DWORD;
    if ErrorCode != ERROR_SUCCESS {
        if data.is_null() && !count.is_null() {
            *count = 0;
        }
        RtlFreeUnicodeString(&mut nameW);
        return ErrorCode as LONG;
    }

    if is_string(LocalType) == 0 || count.is_null() {
        ErrorCode = RegQueryValueExW(hkeyorg, nameW.Buffer, reserved, type_, data, count) as DWORD;
        RtlFreeUnicodeString(&mut nameW);
        return ErrorCode as LONG;
    }

    Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferSize as SIZE_T) as *mut u16;
    if Buffer.is_null() {
        RtlFreeUnicodeString(&mut nameW);
        return ERROR_NOT_ENOUGH_MEMORY as LONG;
    }

    ErrorCode = RegQueryValueExW(
        hkeyorg,
        nameW.Buffer,
        reserved,
        type_,
        Buffer as LPBYTE,
        &mut BufferSize,
    ) as DWORD;
    if ErrorCode != ERROR_SUCCESS {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer as PVOID);
        RtlFreeUnicodeString(&mut nameW);
        return ErrorCode as LONG;
    }

    RtlFreeUnicodeString(&mut nameW);

    DataLength = *count;
    RtlUnicodeToMultiByteSize(count, Buffer as *const u16, BufferSize);

    if data.is_null() || DataLength < *count {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer as PVOID);
        return if !data.is_null() {
            ERROR_MORE_DATA as LONG
        } else {
            ERROR_SUCCESS as LONG
        };
    }

    RtlUnicodeToMultiByteN(
        DataStr,
        DataLength,
        core::ptr::null_mut(),
        Buffer,
        BufferSize,
    );

    if DataLength > *count {
        *DataStr.offset(*count as isize) = 0;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer as PVOID);

    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegQueryValueExW(
    hkeyorg: HKEY,
    name: LPCWSTR,
    reserved: LPDWORD,
    type_: LPDWORD,
    data: LPBYTE,
    count: LPDWORD,
) -> LONG {
    let mut hkey: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut name_str: _UNICODE_STRING = core::mem::zeroed();
    let mut total_size: DWORD;
    let mut buffer: [core::ffi::c_char; 256] = [0; 256];
    let mut buf_ptr: *mut core::ffi::c_char = buffer.as_mut_ptr();
    let mut info: *mut KEY_VALUE_PARTIAL_INFORMATION =
        buffer.as_mut_ptr() as *mut KEY_VALUE_PARTIAL_INFORMATION;
    let info_size: core::ffi::c_int =
        core::mem::offset_of!(KEY_VALUE_PARTIAL_INFORMATION, Data) as core::ffi::c_int;

    if (!data.is_null() && count.is_null()) || !reserved.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    status = MapDefaultKey(&mut hkey, hkeyorg);
    if !NT_SUCCESS(status) {
        return RtlNtStatusToDosError(status) as LONG;
    }

    if IsHKCRKey(hkey as HKEY) != 0 {
        let ErrorCode = QueryHKCRValue(hkey as HKEY, name, reserved, type_, data, count);
        ClosePredefKey(hkey);
        return ErrorCode;
    }

    RtlInitUnicodeString(&mut name_str, name);

    if !data.is_null() {
        let a = core::mem::size_of_val(&buffer) as DWORD;
        let b = (*count).wrapping_add(info_size as DWORD);
        total_size = if a < b { a } else { b };
    } else {
        total_size = info_size as DWORD;
    }

    status = NtQueryValueKey(
        hkey,
        &mut name_str,
        KeyValuePartialInformation,
        buffer.as_mut_ptr() as *mut core::ffi::c_void,
        total_size,
        &mut total_size,
    );

    'done: {
        if !NT_SUCCESS(status) && status != STATUS_BUFFER_OVERFLOW {
            if status == STATUS_INVALID_HANDLE && !hkey.is_null() {
                if !type_.is_null() {
                    *type_ = REG_NONE;
                }
                if !count.is_null() {
                    *count = 0;
                }
            } else if status == STATUS_OBJECT_NAME_NOT_FOUND {
                if !type_.is_null() {
                    *type_ = REG_NONE;
                }
                if data.is_null() && !count.is_null() {
                    *count = 0;
                }
            }
            break 'done;
        }

        if !data.is_null() {
            while status == STATUS_BUFFER_OVERFLOW
                && total_size - info_size as DWORD <= *count
            {
                if buf_ptr != buffer.as_mut_ptr() {
                    HeapFree(GetProcessHeap(), 0, buf_ptr as *mut core::ffi::c_void);
                }
                buf_ptr = HeapAlloc(GetProcessHeap(), 0, total_size as SIZE_T)
                    as *mut core::ffi::c_char;
                if buf_ptr.is_null() {
                    ClosePredefKey(hkey);
                    return ERROR_NOT_ENOUGH_MEMORY as LONG;
                }
                info = buf_ptr as *mut KEY_VALUE_PARTIAL_INFORMATION;
                status = NtQueryValueKey(
                    hkey,
                    &mut name_str,
                    KeyValuePartialInformation,
                    buf_ptr as *mut core::ffi::c_void,
                    total_size,
                    &mut total_size,
                );
            }

            if NT_SUCCESS(status) {
                memcpy(
                    data as *mut core::ffi::c_void,
                    buf_ptr.add(info_size as usize) as *const core::ffi::c_void,
                    (total_size - info_size as DWORD) as usize,
                );
                if is_string((*info).Type) != 0
                    && total_size - info_size as DWORD
                        <= *count - core::mem::size_of::<u16>() as DWORD
                {
                    let ptr = data.add((total_size - info_size as DWORD) as usize)
                        as *mut u16;
                    if ptr > data as *mut u16 && *ptr.offset(-1) != 0 {
                        *ptr = 0;
                    }
                }
            } else if status != STATUS_BUFFER_OVERFLOW {
                break 'done;
            }
        } else {
            status = STATUS_SUCCESS;
        }

        if !type_.is_null() {
            *type_ = (*info).Type;
        }
        if !count.is_null() {
            *count = total_size - info_size as DWORD;
        }
    }

    if buf_ptr != buffer.as_mut_ptr() {
        HeapFree(GetProcessHeap(), 0, buf_ptr as *mut core::ffi::c_void);
    }
    ClosePredefKey(hkey);
    RtlNtStatusToDosError(status) as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegQueryValueA(
    hkey: HKEY,
    name: LPCSTR,
    data: LPSTR,
    count: LPLONG,
) -> LSTATUS {
    let mut ret: DWORD;
    let mut subkey: HKEY = hkey;

    if !name.is_null() && *name != 0 {
        ret = RegOpenKeyA(hkey, name, &mut subkey) as DWORD;
        if ret != ERROR_SUCCESS {
            return ret as LSTATUS;
        }
    }

    ret = RegQueryValueExA(
        subkey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        data as LPBYTE,
        count as LPDWORD,
    ) as DWORD;

    if subkey != hkey {
        RegCloseKey(subkey);
    }

    if ret == ERROR_FILE_NOT_FOUND {
        if !data.is_null() {
            *data = 0;
        }
        if !count.is_null() {
            *count = 1;
        }
        ret = ERROR_SUCCESS;
    }

    ret as LSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn RegQueryValueW(
    hkey: HKEY,
    name: LPCWSTR,
    data: LPWSTR,
    count: LPLONG,
) -> LSTATUS {
    let mut ret: DWORD;
    let mut subkey: HKEY = hkey;

    if hkey.is_null() {
        return ERROR_INVALID_HANDLE as LSTATUS;
    }

    if !name.is_null() && *name != 0 {
        ret = RegOpenKeyW(hkey, name, &mut subkey) as DWORD;
        if ret != ERROR_SUCCESS {
            return ret as LSTATUS;
        }
    }

    ret = RegQueryValueExW(
        subkey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        data as LPBYTE,
        count as LPDWORD,
    ) as DWORD;

    if subkey != hkey {
        RegCloseKey(subkey);
    }

    if ret == ERROR_FILE_NOT_FOUND {
        if !data.is_null() {
            *data = 0;
        }
        if !count.is_null() {
            *count = core::mem::size_of::<u16>() as LONG;
        }
        ret = ERROR_SUCCESS;
    }

    ret as LSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn RegReplaceKeyA(
    hKey: HKEY,
    lpSubKey: LPCSTR,
    lpNewFile: LPCSTR,
    lpOldFile: LPCSTR,
) -> LONG {
    let mut SubKey: _UNICODE_STRING = core::mem::zeroed();
    let mut NewFile: _UNICODE_STRING = core::mem::zeroed();
    let mut OldFile: _UNICODE_STRING = core::mem::zeroed();
    let mut ErrorCode: LONG;

    SubKey.Length = 0;
    SubKey.MaximumLength = 0;
    SubKey.Buffer = core::ptr::null_mut();
    OldFile.Length = 0;
    OldFile.MaximumLength = 0;
    OldFile.Buffer = core::ptr::null_mut();
    NewFile.Length = 0;
    NewFile.MaximumLength = 0;
    NewFile.Buffer = core::ptr::null_mut();

    'block: {
        if !lpSubKey.is_null() {
            if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(SubKey), lpSubKey) == 0 {
                ErrorCode = ERROR_NOT_ENOUGH_MEMORY as LONG;
                break 'block;
            }
        }

        if !lpOldFile.is_null() {
            if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(OldFile), lpOldFile) == 0 {
                ErrorCode = ERROR_NOT_ENOUGH_MEMORY as LONG;
                break 'block;
            }
        }

        if !lpNewFile.is_null() {
            if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(NewFile), lpNewFile) == 0 {
                ErrorCode = ERROR_NOT_ENOUGH_MEMORY as LONG;
                break 'block;
            }
        }

        ErrorCode = RegReplaceKeyW(hKey, SubKey.Buffer, NewFile.Buffer, OldFile.Buffer);
    }

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(OldFile));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(NewFile));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(SubKey));

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegReplaceKeyW(
    hKey: HKEY,
    lpSubKey: LPCWSTR,
    lpNewFile: LPCWSTR,
    lpOldFile: LPCWSTR,
) -> LONG {
    let mut KeyObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut NewObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut OldObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SubKeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut NewFileName: _UNICODE_STRING = core::mem::zeroed();
    let mut OldFileName: _UNICODE_STRING = core::mem::zeroed();
    let CloseRealKey: BOOLEAN;
    let mut RealKeyHandle: HANDLE;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ErrorCode: LONG = ERROR_SUCCESS as LONG;

    if SamePredefKey(hKey, HKEY_PERFORMANCE_DATA) {
        return ERROR_INVALID_HANDLE as LONG;
    }

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    'cleanup: {
        /* Open the real key */
        if !lpSubKey.is_null() && *lpSubKey != 0 {
            RtlInitUnicodeString(&mut SubKeyName, lpSubKey);
            InitializeObjectAttributes(
                &mut KeyObjectAttributes,
                &mut SubKeyName,
                OBJ_CASE_INSENSITIVE,
                KeyHandle,
                core::ptr::null_mut(),
            );
            RealKeyHandle = core::ptr::null_mut();
            Status = NtOpenKey(&mut RealKeyHandle, MAXIMUM_ALLOWED, &mut KeyObjectAttributes);
            if !NT_SUCCESS(Status) {
                ErrorCode = RtlNtStatusToDosError(Status) as LONG;
                break 'cleanup;
            }

            CloseRealKey = 1u8;
        } else {
            RealKeyHandle = KeyHandle;
            CloseRealKey = 0u8;
        }

        /* Convert new file name */
        if RtlDosPathNameToNtPathName_U(
            lpNewFile,
            &mut NewFileName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            if CloseRealKey != 0 {
                NtClose(RealKeyHandle);
            }

            ErrorCode = ERROR_INVALID_PARAMETER as LONG;
            break 'cleanup;
        }

        InitializeObjectAttributes(
            &mut NewObjectAttributes,
            &mut NewFileName,
            OBJ_CASE_INSENSITIVE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        /* Convert old file name */
        if RtlDosPathNameToNtPathName_U(
            lpOldFile,
            &mut OldFileName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            RtlFreeHeap(RtlGetProcessHeap(), 0, NewFileName.Buffer as PVOID);
            if CloseRealKey != 0 {
                NtClose(RealKeyHandle);
            }

            ErrorCode = ERROR_INVALID_PARAMETER as LONG;
            break 'cleanup;
        }

        InitializeObjectAttributes(
            &mut OldObjectAttributes,
            &mut OldFileName,
            OBJ_CASE_INSENSITIVE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        Status = NtReplaceKey(&mut NewObjectAttributes, RealKeyHandle, &mut OldObjectAttributes);

        RtlFreeHeap(RtlGetProcessHeap(), 0, OldFileName.Buffer as PVOID);
        RtlFreeHeap(RtlGetProcessHeap(), 0, NewFileName.Buffer as PVOID);

        if CloseRealKey != 0 {
            NtClose(RealKeyHandle);
        }

        if !NT_SUCCESS(Status) {
            return RtlNtStatusToDosError(Status) as LONG;
        }
    }

    ClosePredefKey(KeyHandle);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegRestoreKeyA(
    hKey: HKEY,
    lpFile: LPCSTR,
    dwFlags: DWORD,
) -> LONG {
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let ErrorCode: LONG;

    if !lpFile.is_null() {
        if RtlCreateUnicodeStringFromAsciiz(&mut FileName, lpFile) == 0 {
            return ERROR_NOT_ENOUGH_MEMORY as LONG;
        }
    } else {
        FileName.Length = 0;
        FileName.MaximumLength = 0;
        FileName.Buffer = core::ptr::null_mut();
    }

    ErrorCode = RegRestoreKeyW(hKey, FileName.Buffer, dwFlags);

    RtlFreeUnicodeString(&mut FileName);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegRestoreKeyW(
    hKey: HKEY,
    lpFile: LPCWSTR,
    dwFlags: DWORD,
) -> LONG {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if SamePredefKey(hKey, HKEY_PERFORMANCE_DATA) {
        return ERROR_INVALID_HANDLE as LONG;
    }

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    'Cleanup: {
        if RtlDosPathNameToNtPathName_U(
            lpFile,
            &mut FileName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            Status = STATUS_INVALID_PARAMETER;
            break 'Cleanup;
        }

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.ObjectName = &mut FileName;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenFile(
            &mut FileHandle,
            FILE_GENERIC_READ,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            FILE_SHARE_READ,
            FILE_SYNCHRONOUS_IO_NONALERT,
        );
        RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);
        if !NT_SUCCESS(Status) {
            break 'Cleanup;
        }

        Status = NtRestoreKey(KeyHandle, FileHandle, dwFlags as ULONG);
        NtClose(FileHandle);
    }

    if (KeyHandle as ULONG_PTR) & 0x1 != 0 {
        NtClose(KeyHandle);
    }

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegSaveKeyA(
    hKey: HKEY,
    lpFile: LPCSTR,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
) -> LONG {
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let ErrorCode: LONG;

    if !lpFile.is_null() {
        if RtlCreateUnicodeStringFromAsciiz(&mut FileName, lpFile) == 0 {
            return ERROR_NOT_ENOUGH_MEMORY as LONG;
        }
    } else {
        FileName.Length = 0;
        FileName.MaximumLength = 0;
        FileName.Buffer = core::ptr::null_mut();
    }

    ErrorCode = RegSaveKeyW(hKey, FileName.Buffer, lpSecurityAttributes);
    RtlFreeUnicodeString(&mut FileName);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegSaveKeyW(
    hKey: HKEY,
    lpFile: LPCWSTR,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
) -> LONG {
    let mut SecurityDescriptor: PVOID = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    'cleanup: {
        if RtlDosPathNameToNtPathName_U(
            lpFile,
            &mut FileName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            Status = STATUS_INVALID_PARAMETER;
            break 'cleanup;
        }

        if !lpSecurityAttributes.is_null() {
            SecurityDescriptor = (*lpSecurityAttributes).lpSecurityDescriptor;
        }

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.ObjectName = &mut FileName;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.SecurityDescriptor = SecurityDescriptor;
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtCreateFile(
            &mut FileHandle,
            GENERIC_WRITE | SYNCHRONIZE,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            core::ptr::null_mut(),
            FILE_ATTRIBUTE_NORMAL,
            FILE_SHARE_READ,
            FILE_CREATE,
            FILE_OPEN_FOR_BACKUP_INTENT | FILE_SYNCHRONOUS_IO_NONALERT,
            core::ptr::null_mut(),
            0,
        );
        RtlFreeHeap(RtlGetProcessHeap(), 0, FileName.Buffer as PVOID);
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        Status = NtSaveKey(KeyHandle, FileHandle);
        NtClose(FileHandle);
    }

    if (KeyHandle as ULONG_PTR) & 0x1 != 0 {
        NtClose(KeyHandle);
    }

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegSaveKeyExA(
    hKey: HKEY,
    lpFile: LPCSTR,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
    Flags: DWORD,
) -> LONG {
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let ErrorCode: LONG;

    if !lpFile.is_null() {
        if RtlCreateUnicodeStringFromAsciiz(&mut FileName, lpFile) == 0 {
            return ERROR_NOT_ENOUGH_MEMORY as LONG;
        }
    } else {
        FileName.Length = 0;
        FileName.MaximumLength = 0;
        FileName.Buffer = core::ptr::null_mut();
    }

    ErrorCode = RegSaveKeyExW(hKey, FileName.Buffer, lpSecurityAttributes, Flags);

    RtlFreeUnicodeString(&mut FileName);

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegSaveKeyExW(
    hKey: HKEY,
    lpFile: LPCWSTR,
    lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
    Flags: DWORD,
) -> LONG {
    match Flags {
        REG_STANDARD_FORMAT | REG_LATEST_FORMAT | REG_NO_COMPRESSION => {}
        _ => return ERROR_INVALID_PARAMETER as LONG,
    }

    RegSaveKeyW(hKey, lpFile, lpSecurityAttributes)
}

#[no_mangle]
pub unsafe extern "system" fn RegSetKeySecurity(
    hKey: HKEY,
    SecurityInformation: SECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> LONG {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if SamePredefKey(hKey, HKEY_PERFORMANCE_DATA) {
        return ERROR_INVALID_HANDLE as LONG;
    }

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    Status = NtSetSecurityObject(KeyHandle, SecurityInformation, pSecurityDescriptor);

    if (KeyHandle as ULONG_PTR) & 0x1 != 0 {
        NtClose(KeyHandle);
    }

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegSetValueExA(
    hKey: HKEY,
    lpValueName: LPCSTR,
    Reserved: DWORD,
    dwType: DWORD,
    lpData: *const BYTE,
    cbData: DWORD,
) -> LONG {
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let pValueName: LPWSTR;
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut Data: _UNICODE_STRING = core::mem::zeroed();
    let ErrorCode: LONG;
    let pData: LPBYTE;
    let DataSize: DWORD;
    let Status: NTSTATUS;

    if !lpValueName.is_null() && *lpValueName != 0 {
        if RtlCreateUnicodeStringFromAsciiz(&mut ValueName, lpValueName) == 0 {
            return ERROR_NOT_ENOUGH_MEMORY as LONG;
        }
    } else {
        ValueName.Buffer = core::ptr::null_mut();
    }

    pValueName = ValueName.Buffer;

    if is_string(dwType) != 0 && cbData != 0 {
        let bNoNulledStr: LONG = if *lpData.add((cbData - 1) as usize) != 0 { 1 } else { 0 };
        AnsiString.Buffer = lpData as PSTR;
        AnsiString.Length = (cbData as i32 + bNoNulledStr) as u16;
        AnsiString.MaximumLength = (cbData as i32 + bNoNulledStr) as u16;
        Status = RtlAnsiStringToUnicodeString(&mut Data, &mut AnsiString, 1u8);

        if !NT_SUCCESS(Status) {
            if !pValueName.is_null() {
                RtlFreeUnicodeString(&mut ValueName);
            }
            return RtlNtStatusToDosError(Status) as LONG;
        }
        pData = Data.Buffer as LPBYTE;
        DataSize = cbData * core::mem::size_of::<u16>() as DWORD;
    } else {
        Data.Buffer = core::ptr::null_mut();
        pData = lpData as LPBYTE;
        DataSize = cbData;
    }

    ErrorCode = RegSetValueExW(hKey, pValueName, Reserved, dwType, pData, DataSize);

    if !pValueName.is_null() {
        RtlFreeUnicodeString(&mut ValueName);
    }

    if !Data.Buffer.is_null() {
        RtlFreeUnicodeString(&mut Data);
    }

    ErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn RegSetValueExW(
    hKey: HKEY,
    lpValueName: LPCWSTR,
    Reserved: DWORD,
    dwType: DWORD,
    lpData: *const BYTE,
    mut cbData: DWORD,
) -> LONG {
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    if IsHKCRKey(KeyHandle as HKEY) != 0 {
        let ErrorCode = SetHKCRValue(KeyHandle as HKEY, lpValueName, Reserved, dwType, lpData, cbData);
        ClosePredefKey(KeyHandle);
        return ErrorCode;
    }

    if is_string(dwType) != 0 && cbData != 0 {
        let pwsData = lpData as PWSTR;

        let idx = (cbData as usize) / core::mem::size_of::<u16>();
        if *pwsData.add(idx - 1) != 0 && *pwsData.add(idx) == 0 {
            cbData += core::mem::size_of::<u16>() as DWORD;
        }
    }

    RtlInitUnicodeString(&mut ValueName, lpValueName);

    Status = NtSetValueKey(
        KeyHandle,
        &mut ValueName,
        0,
        dwType,
        lpData as PVOID,
        cbData as ULONG,
    );

    ClosePredefKey(KeyHandle);

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    ERROR_SUCCESS as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegSetValueA(
    hKeyOriginal: HKEY,
    lpSubKey: LPCSTR,
    dwType: DWORD,
    lpData: LPCSTR,
    cbData: DWORD,
) -> LONG {
    let mut subkey: HKEY;
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut ret: DWORD;
    let Status: NTSTATUS;

    if dwType != REG_SZ || lpData.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    Status = MapDefaultKey(&mut hKey as *mut HANDLE as PHANDLE, hKeyOriginal);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }
    subkey = hKey as HKEY;

    'cleanup: {
        if !lpSubKey.is_null() && *lpSubKey != 0 {
            ret = RegCreateKeyA(hKey as HKEY, lpSubKey, &mut subkey) as DWORD;
            if ret != ERROR_SUCCESS {
                break 'cleanup;
            }
        }

        ret = RegSetValueExA(
            subkey,
            core::ptr::null_mut(),
            0,
            REG_SZ,
            lpData as *const BYTE,
            (strlen(lpData) + 1) as DWORD,
        ) as DWORD;
        if subkey != hKey as HKEY {
            RegCloseKey(subkey);
        }
    }

    if (hKey as ULONG_PTR) & 0x1 != 0 {
        NtClose(hKey);
    }

    ret as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegSetValueW(
    hKeyOriginal: HKEY,
    lpSubKey: LPCWSTR,
    dwType: DWORD,
    lpData: LPCWSTR,
    cbData: DWORD,
) -> LONG {
    let mut subkey: HKEY;
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut ret: DWORD;
    let Status: NTSTATUS;

    if dwType != REG_SZ || lpData.is_null() {
        return ERROR_INVALID_PARAMETER as LONG;
    }

    Status = MapDefaultKey(&mut hKey, hKeyOriginal);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }
    subkey = hKey as HKEY;

    'cleanup: loop {
        if !lpSubKey.is_null() && *lpSubKey != 0 {
            ret = RegCreateKeyW(hKey as HKEY, lpSubKey, &mut subkey) as DWORD;
            if ret != ERROR_SUCCESS {
                break 'cleanup;
            }
        }

        ret = RegSetValueExW(
            subkey,
            core::ptr::null_mut(),
            0,
            REG_SZ,
            lpData as *const BYTE,
            ((wcslen(lpData) + 1) * core::mem::size_of::<WCHAR>()) as DWORD,
        ) as DWORD;
        if subkey != hKey as HKEY {
            RegCloseKey(subkey);
        }

        break 'cleanup;
    }

    ClosePredefKey(hKey);

    ret as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegUnLoadKeyA(hKey: HKEY, lpSubKey: LPCSTR) -> LONG {
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let ErrorCode: DWORD;

    if !lpSubKey.is_null() {
        if RtlCreateUnicodeStringFromAsciiz(&mut KeyName, lpSubKey) == 0 {
            return ERROR_NOT_ENOUGH_MEMORY as LONG;
        }
    } else {
        KeyName.Length = 0;
        KeyName.MaximumLength = 0;
        KeyName.Buffer = core::ptr::null_mut();
    }

    ErrorCode = RegUnLoadKeyW(hKey, KeyName.Buffer) as DWORD;

    RtlFreeUnicodeString(&mut KeyName);

    ErrorCode as LONG
}

#[no_mangle]
pub unsafe extern "system" fn RegUnLoadKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LONG {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if SamePredefKey(hKey, HKEY_PERFORMANCE_DATA) {
        return ERROR_INVALID_HANDLE as LONG;
    }

    Status = MapDefaultKey(&mut KeyHandle, hKey);
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    RtlInitUnicodeString(&mut KeyName, lpSubKey);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = KeyHandle;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtUnloadKey(&mut ObjectAttributes);

    ClosePredefKey(KeyHandle);

    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status) as LONG;
    }

    ERROR_SUCCESS as LONG
}

unsafe fn load_string(hModule: HINSTANCE, mut resId: UINT, pwszBuffer: LPWSTR, mut cMaxChars: INT) -> i32 {
    let hMemory: HGLOBAL;
    let hResource: HRSRC;
    let mut pString: *mut WCHAR;
    let mut idxString: i32;

    if HIWORD(resId) == 0xffff {
        resId = (-((resId as INT))) as UINT;
    }

    hResource = FindResourceW(hModule, MAKEINTRESOURCEW(LOWORD(resId >> 4) + 1), RT_STRING as LPWSTR) as HRSRC;
    if hResource.is_null() {
        return 0;
    }
    hMemory = LoadResource(hModule, hResource as HANDLE);
    if hMemory.is_null() {
        return 0;
    }
    pString = LockResource(hMemory) as *mut WCHAR;

    idxString = (resId & 0xf) as i32;
    while {
        let old = idxString;
        idxString -= 1;
        old != 0
    } {
        pString = pString.add((*pString as usize) + 1);
    }

    if pwszBuffer.is_null() {
        return *pString as i32;
    }

    cMaxChars = if (*pString as INT) < cMaxChars {
        *pString as INT
    } else {
        cMaxChars - 1
    };
    if cMaxChars >= 0 {
        memcpy(
            pwszBuffer as *mut core::ffi::c_void,
            pString.add(1) as *const core::ffi::c_void,
            (cMaxChars as usize) * core::mem::size_of::<WCHAR>(),
        );
        *pwszBuffer.add(cMaxChars as usize) = 0;
    }

    cMaxChars
}

#[no_mangle]
pub unsafe extern "system" fn RegLoadMUIStringW(
    hKey: HKEY,
    pszValue: LPCWSTR,
    pszOutBuf: LPWSTR,
    cbOutBuf: DWORD,
    pcbData: LPDWORD,
    Flags: DWORD,
    pszDirectory: LPCWSTR,
) -> LONG {
    let mut dwValueType: DWORD = 0;
    let mut cbData: DWORD = 0;
    let mut pwszTempBuffer: LPWSTR = core::ptr::null_mut();
    let mut pwszExpandedBuffer: LPWSTR = core::ptr::null_mut();
    let mut result: LONG;

    'cleanup: loop {
        /* Parameter sanity checks. */
        if hKey.is_null() || pszOutBuf.is_null() {
            return ERROR_INVALID_PARAMETER as LONG;
        }

        if !pszDirectory.is_null() && *pszDirectory != 0 {
            FIXME!("BaseDir parameter not yet supported!\n");
            return ERROR_INVALID_PARAMETER as LONG;
        }

        /* Check for value existence and correctness of it's type, allocate a buffer and load it. */
        result = RegQueryValueExW(
            hKey,
            pszValue,
            core::ptr::null_mut(),
            &mut dwValueType,
            core::ptr::null_mut(),
            &mut cbData,
        );
        if result != ERROR_SUCCESS as LONG {
            break 'cleanup;
        }
        if !(dwValueType == REG_SZ || dwValueType == REG_EXPAND_SZ) || cbData == 0 {
            result = ERROR_FILE_NOT_FOUND as LONG;
            break 'cleanup;
        }
        pwszTempBuffer = HeapAlloc(GetProcessHeap(), 0, cbData as SIZE_T) as LPWSTR;
        if pwszTempBuffer.is_null() {
            result = ERROR_NOT_ENOUGH_MEMORY as LONG;
            break 'cleanup;
        }
        result = RegQueryValueExW(
            hKey,
            pszValue,
            core::ptr::null_mut(),
            &mut dwValueType,
            pwszTempBuffer as LPBYTE,
            &mut cbData,
        );
        if result != ERROR_SUCCESS as LONG {
            break 'cleanup;
        }

        /* Expand environment variables, if appropriate, or copy the original string over. */
        if dwValueType == REG_EXPAND_SZ {
            cbData = ExpandEnvironmentStringsW(pwszTempBuffer, core::ptr::null_mut(), 0)
                * core::mem::size_of::<u16>() as DWORD;
            if cbData == 0 {
                break 'cleanup;
            }
            pwszExpandedBuffer = HeapAlloc(GetProcessHeap(), 0, cbData as SIZE_T) as LPWSTR;
            if pwszExpandedBuffer.is_null() {
                result = ERROR_NOT_ENOUGH_MEMORY as LONG;
                break 'cleanup;
            }
            ExpandEnvironmentStringsW(pwszTempBuffer, pwszExpandedBuffer, cbData);
        } else {
            pwszExpandedBuffer = HeapAlloc(GetProcessHeap(), 0, cbData as SIZE_T) as LPWSTR;
            memcpy(
                pwszExpandedBuffer as *mut core::ffi::c_void,
                pwszTempBuffer as *const core::ffi::c_void,
                cbData as usize,
            );
        }

        /* If the value references a resource based string, parse the value and load the string.
         * Else just copy over the original value. */
        result = ERROR_SUCCESS as LONG;
        if *pwszExpandedBuffer != b'@' as u16 {
            lstrcpynW(
                pszOutBuf,
                pwszExpandedBuffer,
                (cbOutBuf / core::mem::size_of::<u16>() as DWORD) as INT,
            );
        } else {
            let pComma: *mut u16 = wcsrchr(pwszExpandedBuffer, b',' as u16);
            let uiStringId: UINT;
            let hModule: HMODULE;

            /* Format of the expanded value is 'path_to_dll,-resId' */
            if pComma.is_null() || *pComma.add(1) != b'-' as u16 {
                result = ERROR_BADKEY as LONG;
                break 'cleanup;
            }

            uiStringId = _wtoi(pComma.add(2)) as UINT;
            *pComma = 0;

            hModule = LoadLibraryExW(
                pwszExpandedBuffer.add(1),
                core::ptr::null_mut(),
                LOAD_LIBRARY_AS_DATAFILE,
            );
            if hModule.is_null()
                || load_string(
                    hModule as HINSTANCE,
                    uiStringId,
                    pszOutBuf,
                    (cbOutBuf / core::mem::size_of::<u16>() as DWORD) as INT,
                ) == 0
            {
                result = ERROR_BADKEY as LONG;
            }
            FreeLibrary(hModule);
        }

        break 'cleanup;
    }

    HeapFree(GetProcessHeap(), 0, pwszTempBuffer as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, pwszExpandedBuffer as *mut core::ffi::c_void);
    result
}

#[no_mangle]
pub unsafe extern "system" fn RegLoadMUIStringA(
    hKey: HKEY,
    pszValue: LPCSTR,
    pszOutBuf: LPSTR,
    cbOutBuf: DWORD,
    pcbData: LPDWORD,
    Flags: DWORD,
    pszDirectory: LPCSTR,
) -> LONG {
    let mut valueW: _UNICODE_STRING = core::mem::zeroed();
    let mut baseDirW: _UNICODE_STRING = core::mem::zeroed();
    let mut pwszBuffer: *mut WCHAR;
    let mut cbData: DWORD = cbOutBuf * core::mem::size_of::<WCHAR>() as DWORD;
    let result: LONG;

    valueW.Buffer = core::ptr::null_mut();
    baseDirW.Buffer = core::ptr::null_mut();
    pwszBuffer = core::ptr::null_mut();

    'cleanup: {
        if RtlCreateUnicodeStringFromAsciiz(&mut valueW, pszValue) == 0
            || RtlCreateUnicodeStringFromAsciiz(&mut baseDirW, pszDirectory) == 0
            || {
                pwszBuffer = HeapAlloc(GetProcessHeap(), 0, cbData as SIZE_T) as *mut WCHAR;
                pwszBuffer.is_null()
            }
        {
            result = ERROR_NOT_ENOUGH_MEMORY as LONG;
            break 'cleanup;
        }

        result = RegLoadMUIStringW(
            hKey,
            valueW.Buffer,
            pwszBuffer,
            cbData,
            core::ptr::null_mut(),
            Flags,
            baseDirW.Buffer,
        );

        if result == ERROR_SUCCESS as LONG {
            cbData = WideCharToMultiByte(
                CP_ACP,
                0,
                pwszBuffer,
                -1,
                pszOutBuf,
                cbOutBuf as i32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as DWORD;
            if !pcbData.is_null() {
                *pcbData = cbData;
            }
        }
    }

    HeapFree(GetProcessHeap(), 0, pwszBuffer as *mut core::ffi::c_void);
    RtlFreeUnicodeString(&mut baseDirW);
    RtlFreeUnicodeString(&mut valueW);

    result
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            lib/advapi32/reg/reg.c
//  * PURPOSE:         Registry functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  *                  Thomas Weidenmueller <w3seek@reactos.com>
//  * UPDATE HISTORY:
//  *                  Created 01/11/98
//  *                  19990309 EA Stubs
//  *                  20050502 Fireball imported some stuff from WINE
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <advapi32.h>
//
// #include <ndk/cmfuncs.h>
// #include <pseh/pseh2.h>
//
// #include "reg.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(reg);
//
// /* DEFINES ******************************************************************/
//
// #define MAX_DEFAULT_HANDLES   6
// #define REG_MAX_NAME_SIZE     256
// #define REG_MAX_DATA_SIZE     2048
//
// /* GLOBALS ******************************************************************/
//
// static RTL_CRITICAL_SECTION HandleTableCS;
// static HANDLE DefaultHandleTable[MAX_DEFAULT_HANDLES];
// static HANDLE ProcessHeap;
// static BOOLEAN DefaultHandlesDisabled = FALSE;
// static BOOLEAN DefaultHandleHKUDisabled = FALSE;
// static BOOLEAN DllInitialized = FALSE; /* HACK */
//
// /* PROTOTYPES ***************************************************************/
//
// static NTSTATUS MapDefaultKey (PHANDLE ParentKey, HKEY Key);
// static VOID CloseDefaultKeys(VOID);
// #define ClosePredefKey(Handle)                                                 \
//     if ((ULONG_PTR)Handle & 0x1) {                                             \
//         NtClose(Handle);                                                       \
//     }
// #define IsPredefKey(HKey)                                                      \
//     (((ULONG_PTR)(HKey) & 0xF0000000) == 0x80000000)
// #define GetPredefKeyIndex(HKey)                                                \
//     ((ULONG_PTR)(HKey) & 0x0FFFFFFF)
//
// static NTSTATUS OpenClassesRootKey(PHANDLE KeyHandle);
// static NTSTATUS OpenLocalMachineKey (PHANDLE KeyHandle);
// static NTSTATUS OpenUsersKey (PHANDLE KeyHandle);
// static NTSTATUS OpenCurrentConfigKey(PHANDLE KeyHandle);
//
//
// /* FUNCTIONS ****************************************************************/
// /* check if value type needs string conversion (Ansi<->Unicode) */
// __inline static int is_string( DWORD type )
// {
//     return (type == REG_SZ) || (type == REG_EXPAND_SZ) || (type == REG_MULTI_SZ);
// }
//
// /************************************************************************
//  *  RegInitDefaultHandles
//  */
// BOOL
// RegInitialize(VOID)
// {
//     TRACE("RegInitialize()\n");
//
//     /* Lazy init hack */
//     if (!DllInitialized)
//     {
//         ProcessHeap = RtlGetProcessHeap();
//         RtlZeroMemory(DefaultHandleTable,
//                       MAX_DEFAULT_HANDLES * sizeof(HANDLE));
//         RtlInitializeCriticalSection(&HandleTableCS);
//
//         DllInitialized = TRUE;
//     }
//
//     return TRUE;
// }
//
//
// /************************************************************************
//  *  RegInit
//  */
// BOOL
// RegCleanup(VOID)
// {
//     TRACE("RegCleanup()\n");
//
//     CloseDefaultKeys();
//     RtlDeleteCriticalSection(&HandleTableCS);
//
//     return TRUE;
// }
//
//
// static NTSTATUS
// OpenPredefinedKey(IN ULONG Index,
//                   OUT HANDLE Handle)
// {
//     NTSTATUS Status;
//
//     switch (Index)
//     {
//         case 0: /* HKEY_CLASSES_ROOT */
//             Status = OpenClassesRootKey (Handle);
//             break;
//
//         case 1: /* HKEY_CURRENT_USER */
//             Status = RtlOpenCurrentUser (MAXIMUM_ALLOWED,
//                                          Handle);
//             break;
//
//         case 2: /* HKEY_LOCAL_MACHINE */
//             Status = OpenLocalMachineKey (Handle);
//             break;
//
//         case 3: /* HKEY_USERS */
//             Status = OpenUsersKey (Handle);
//             break;
//
//         case 5: /* HKEY_CURRENT_CONFIG */
//             Status = OpenCurrentConfigKey (Handle);
//             break;
//
//         case 6: /* HKEY_DYN_DATA */
//             Status = STATUS_NOT_IMPLEMENTED;
//             break;
//
//         default:
//             WARN("MapDefaultHandle() no handle creator\n");
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//     }
//
//     return Status;
// }
//
//
// static NTSTATUS
// MapDefaultKey(OUT PHANDLE RealKey,
//               IN HKEY Key)
// {
//     PHANDLE Handle;
//     ULONG Index;
//     BOOLEAN DoOpen, DefDisabled;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     TRACE("MapDefaultKey (Key %x)\n", Key);
//
//     if (!IsPredefKey(Key))
//     {
//         *RealKey = (HANDLE)((ULONG_PTR)Key & ~0x1);
//         return STATUS_SUCCESS;
//     }
//
//     /* Handle special cases here */
//     Index = GetPredefKeyIndex(Key);
//     if (Index >= MAX_DEFAULT_HANDLES)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//     RegInitialize(); /* HACK until delay-loading is implemented */
//     RtlEnterCriticalSection (&HandleTableCS);
//
//     if (Key == HKEY_CURRENT_USER)
//         DefDisabled = DefaultHandleHKUDisabled;
//     else
//         DefDisabled = DefaultHandlesDisabled;
//
//     if (!DefDisabled)
//     {
//         Handle = &DefaultHandleTable[Index];
//         DoOpen = (*Handle == NULL);
//     }
//     else
//     {
//         Handle = RealKey;
//         DoOpen = TRUE;
//     }
//
//     if (DoOpen)
//     {
//         /* create/open the default handle */
//         Status = OpenPredefinedKey(Index,
//                                    Handle);
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         if (!DefDisabled)
//             *RealKey = *Handle;
//         else
//             *(PULONG_PTR)Handle |= 0x1;
//     }
//
//     RtlLeaveCriticalSection (&HandleTableCS);
//
//     return Status;
// }
//
//
// static VOID
// CloseDefaultKeys(VOID)
// {
//     ULONG i;
//     RegInitialize(); /* HACK until delay-loading is implemented */
//     RtlEnterCriticalSection(&HandleTableCS);
//
//     for (i = 0; i < MAX_DEFAULT_HANDLES; i++)
//     {
//         if (DefaultHandleTable[i] != NULL)
//         {
//             NtClose(DefaultHandleTable[i]);
//             DefaultHandleTable[i] = NULL;
//         }
//     }
//
//     RtlLeaveCriticalSection(&HandleTableCS);
// }
//
//
// static NTSTATUS
// OpenClassesRootKey( PHANDLE KeyHandle)
// {
//     OBJECT_ATTRIBUTES Attributes;
//     UNICODE_STRING KeyName = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\Software\\CLASSES");
//     NTSTATUS Status;
//
//     TRACE("OpenClassesRootKey()\n");
//
//     InitializeObjectAttributes(&Attributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(KeyHandle,
//                        MAXIMUM_ALLOWED,
//                        &Attributes);
//
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Mark it as HKCR */
//     MakeHKCRKey((HKEY*)KeyHandle);
//
//     return Status;
// }
//
//
// static NTSTATUS
// OpenLocalMachineKey(PHANDLE KeyHandle)
// {
//     OBJECT_ATTRIBUTES Attributes;
//     UNICODE_STRING KeyName = RTL_CONSTANT_STRING(L"\\Registry\\Machine");
//     NTSTATUS Status;
//
//     TRACE("OpenLocalMachineKey()\n");
//
//     InitializeObjectAttributes(&Attributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(KeyHandle,
//                        MAXIMUM_ALLOWED,
//                        &Attributes);
//
//     TRACE("NtOpenKey(%wZ) => %08x\n", &KeyName, Status);
//
//     return Status;
// }
//
//
// static NTSTATUS
// OpenUsersKey(PHANDLE KeyHandle)
// {
//     OBJECT_ATTRIBUTES Attributes;
//     UNICODE_STRING KeyName = RTL_CONSTANT_STRING(L"\\Registry\\User");
//
//     TRACE("OpenUsersKey()\n");
//
//     InitializeObjectAttributes(&Attributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     return NtOpenKey(KeyHandle,
//                      MAXIMUM_ALLOWED,
//                      &Attributes);
// }
//
//
// static NTSTATUS
// OpenCurrentConfigKey (PHANDLE KeyHandle)
// {
//     OBJECT_ATTRIBUTES Attributes;
//     UNICODE_STRING KeyName =
//         RTL_CONSTANT_STRING(L"\\Registry\\Machine\\System\\CurrentControlSet\\Hardware Profiles\\Current");
//
//     TRACE("OpenCurrentConfigKey()\n");
//
//     InitializeObjectAttributes(&Attributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     return NtOpenKey(KeyHandle,
//                      MAXIMUM_ALLOWED,
//                      &Attributes);
// }
//
// #ifndef _ADVAPI32_VISTA_
//
// /************************************************************************
//  *  RegDisablePredefinedCache
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegDisablePredefinedCache(VOID)
// {
//     RegInitialize(); /* HACK until delay-loading is implemented */
//     RtlEnterCriticalSection(&HandleTableCS);
//     DefaultHandleHKUDisabled = TRUE;
//     RtlLeaveCriticalSection(&HandleTableCS);
//     return ERROR_SUCCESS;
// }
//
//
// /************************************************************************
//  *  RegDisablePredefinedCacheEx
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegDisablePredefinedCacheEx(VOID)
// {
//     RegInitialize(); /* HACK until delay-loading is implemented */
//     RtlEnterCriticalSection(&HandleTableCS);
//     DefaultHandlesDisabled = TRUE;
//     DefaultHandleHKUDisabled = TRUE;
//     RtlLeaveCriticalSection(&HandleTableCS);
//     return ERROR_SUCCESS;
// }
//
//
// /************************************************************************
//  *  RegOverridePredefKey
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegOverridePredefKey(IN HKEY hKey,
//                      IN HKEY hNewHKey  OPTIONAL)
// {
//     LONG ErrorCode = ERROR_SUCCESS;
//
//     if ((hKey == HKEY_CLASSES_ROOT ||
//          hKey == HKEY_CURRENT_CONFIG ||
//          hKey == HKEY_CURRENT_USER ||
//          hKey == HKEY_LOCAL_MACHINE ||
//          hKey == HKEY_PERFORMANCE_DATA ||
//          hKey == HKEY_USERS) &&
//         !IsPredefKey(hNewHKey))
//     {
//         PHANDLE Handle;
//         ULONG Index;
//
//         Index = GetPredefKeyIndex(hKey);
//         Handle = &DefaultHandleTable[Index];
//
//         if (hNewHKey == NULL)
//         {
//             /* restore the default mapping */
//             NTSTATUS Status = OpenPredefinedKey(Index,
//                                                 &hNewHKey);
//             if (!NT_SUCCESS(Status))
//             {
//                 return RtlNtStatusToDosError(Status);
//             }
//
//             ASSERT(hNewHKey != NULL);
//         }
//         RegInitialize(); /* HACK until delay-loading is implemented */
//         RtlEnterCriticalSection(&HandleTableCS);
//
//         /* close the currently mapped handle if existing */
//         if (*Handle != NULL)
//         {
//             NtClose(*Handle);
//         }
//
//         /* update the mapping */
//         *Handle = hNewHKey;
//
//         RtlLeaveCriticalSection(&HandleTableCS);
//     }
//     else
//         ErrorCode = ERROR_INVALID_HANDLE;
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegCloseKey
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegCloseKey(HKEY hKey)
// {
//     NTSTATUS Status;
//
//     /* don't close null handle or a pseudo handle */
//     if (!hKey)
//     {
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (((ULONG_PTR)hKey & 0xF0000000) == 0x80000000)
//     {
//         return ERROR_SUCCESS;
//     }
//
//     Status = NtClose(hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//   return ERROR_SUCCESS;
// }
//
// #endif // _ADVAPI32_VISTA_
//
// static NTSTATUS
// RegpCopyTree(IN HKEY hKeySrc,
//              IN HKEY hKeyDest)
// {
//     typedef struct
//     {
//         LIST_ENTRY ListEntry;
//         HANDLE hKeySrc;
//         HANDLE hKeyDest;
//     } REGP_COPY_KEYS, *PREGP_COPY_KEYS;
//
//     LIST_ENTRY copyQueueHead;
//     PREGP_COPY_KEYS copyKeys, newCopyKeys;
//     union
//     {
//         KEY_VALUE_FULL_INFORMATION *KeyValue;
//         KEY_NODE_INFORMATION *KeyNode;
//         PVOID Buffer;
//     } Info;
//     ULONG Index, BufferSizeRequired, BufferSize = 0x200;
//     NTSTATUS Status = STATUS_SUCCESS;
//     NTSTATUS Status2 = STATUS_SUCCESS;
//
//     InitializeListHead(&copyQueueHead);
//
//     Info.Buffer = RtlAllocateHeap(ProcessHeap,
//                                   0,
//                                   BufferSize);
//     if (Info.Buffer == NULL)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     copyKeys = RtlAllocateHeap(ProcessHeap,
//                                0,
//                                sizeof(REGP_COPY_KEYS));
//     if (copyKeys != NULL)
//     {
//         copyKeys->hKeySrc = hKeySrc;
//         copyKeys->hKeyDest = hKeyDest;
//         InsertHeadList(&copyQueueHead,
//                        &copyKeys->ListEntry);
//
//         /* FIXME - copy security from hKeySrc to hKeyDest or just for the subkeys? */
//
//         do
//         {
//             copyKeys = CONTAINING_RECORD(copyQueueHead.Flink,
//                                          REGP_COPY_KEYS,
//                                          ListEntry);
//
//             /* enumerate all values and copy them */
//             Index = 0;
//             for (;;)
//             {
//                 Status2 = NtEnumerateValueKey(copyKeys->hKeySrc,
//                                               Index,
//                                               KeyValueFullInformation,
//                                               Info.KeyValue,
//                                               BufferSize,
//                                               &BufferSizeRequired);
//                 if (NT_SUCCESS(Status2))
//                 {
//                     UNICODE_STRING ValueName;
//                     PVOID Data;
//
//                     /* don't use RtlInitUnicodeString as the string is not NULL-terminated! */
//                     ValueName.Length = Info.KeyValue->NameLength;
//                     ValueName.MaximumLength = ValueName.Length;
//                     ValueName.Buffer = Info.KeyValue->Name;
//
//                     Data = (PVOID)((ULONG_PTR)Info.KeyValue + Info.KeyValue->DataOffset);
//
//                     Status2 = NtSetValueKey(copyKeys->hKeyDest,
//                                             &ValueName,
//                                             Info.KeyValue->TitleIndex,
//                                             Info.KeyValue->Type,
//                                             Data,
//                                             Info.KeyValue->DataLength);
//
//                     /* don't break, let's try to copy as many values as possible */
//                     if (!NT_SUCCESS(Status2) && NT_SUCCESS(Status))
//                     {
//                         Status = Status2;
//                     }
//
//                     Index++;
//                 }
//                 else if (Status2 == STATUS_BUFFER_OVERFLOW)
//                 {
//                     PVOID Buffer;
//
//                     ASSERT(BufferSize < BufferSizeRequired);
//
//                     Buffer = RtlReAllocateHeap(ProcessHeap,
//                                                0,
//                                                Info.Buffer,
//                                                BufferSizeRequired);
//                     if (Buffer != NULL)
//                     {
//                         Info.Buffer = Buffer;
//                         BufferSize = BufferSizeRequired;
//                         /* try again */
//                     }
//                     else
//                     {
//                         /* don't break, let's try to copy as many values as possible */
//                         Status2 = STATUS_INSUFFICIENT_RESOURCES;
//                         Index++;
//
//                         if (NT_SUCCESS(Status))
//                         {
//                             Status = Status2;
//                         }
//                     }
//                 }
//                 else
//                 {
//                     /* break to avoid an infinite loop in case of denied access or
//                        other errors! */
//                     if (Status2 != STATUS_NO_MORE_ENTRIES && NT_SUCCESS(Status))
//                     {
//                         Status = Status2;
//                     }
//
//                     break;
//                 }
//             }
//
//             /* enumerate all subkeys and open and enqueue them */
//             Index = 0;
//             for (;;)
//             {
//                 Status2 = NtEnumerateKey(copyKeys->hKeySrc,
//                                          Index,
//                                          KeyNodeInformation,
//                                          Info.KeyNode,
//                                          BufferSize,
//                                          &BufferSizeRequired);
//                 if (NT_SUCCESS(Status2))
//                 {
//                     HANDLE KeyHandle, NewKeyHandle;
//                     OBJECT_ATTRIBUTES ObjectAttributes;
//                     UNICODE_STRING SubKeyName, ClassName;
//
//                     /* don't use RtlInitUnicodeString as the string is not NULL-terminated! */
//                     SubKeyName.Length = Info.KeyNode->NameLength;
//                     SubKeyName.MaximumLength = SubKeyName.Length;
//                     SubKeyName.Buffer = Info.KeyNode->Name;
//                     ClassName.Length = Info.KeyNode->ClassLength;
//                     ClassName.MaximumLength = ClassName.Length;
//                     ClassName.Buffer = (PWSTR)((ULONG_PTR)Info.KeyNode + Info.KeyNode->ClassOffset);
//
//                     /* open the subkey with sufficient rights */
//
//                     InitializeObjectAttributes(&ObjectAttributes,
//                                                &SubKeyName,
//                                                OBJ_CASE_INSENSITIVE,
//                                                copyKeys->hKeySrc,
//                                                NULL);
//
//                     Status2 = NtOpenKey(&KeyHandle,
//                                         KEY_ENUMERATE_SUB_KEYS | KEY_QUERY_VALUE,
//                                         &ObjectAttributes);
//                     if (NT_SUCCESS(Status2))
//                     {
//                         /* FIXME - attempt to query the security information */
//
//                         InitializeObjectAttributes(&ObjectAttributes,
//                                                &SubKeyName,
//                                                OBJ_CASE_INSENSITIVE,
//                                                copyKeys->hKeyDest,
//                                                NULL);
//
//                         Status2 = NtCreateKey(&NewKeyHandle,
//                                               KEY_ALL_ACCESS,
//                                               &ObjectAttributes,
//                                               Info.KeyNode->TitleIndex,
//                                               &ClassName,
//                                               0,
//                                               NULL);
//                         if (NT_SUCCESS(Status2))
//                         {
//                             newCopyKeys = RtlAllocateHeap(ProcessHeap,
//                                                           0,
//                                                           sizeof(REGP_COPY_KEYS));
//                             if (newCopyKeys != NULL)
//                             {
//                                 /* save the handles and enqueue the subkey */
//                                 newCopyKeys->hKeySrc = KeyHandle;
//                                 newCopyKeys->hKeyDest = NewKeyHandle;
//                                 InsertTailList(&copyQueueHead,
//                                                &newCopyKeys->ListEntry);
//                             }
//                             else
//                             {
//                                 NtClose(KeyHandle);
//                                 NtClose(NewKeyHandle);
//
//                                 Status2 = STATUS_INSUFFICIENT_RESOURCES;
//                             }
//                         }
//                         else
//                         {
//                             NtClose(KeyHandle);
//                         }
//                     }
//
//                     if (!NT_SUCCESS(Status2) && NT_SUCCESS(Status))
//                     {
//                         Status = Status2;
//                     }
//
//                     Index++;
//                 }
//                 else if (Status2 == STATUS_BUFFER_OVERFLOW)
//                 {
//                     PVOID Buffer;
//
//                     ASSERT(BufferSize < BufferSizeRequired);
//
//                     Buffer = RtlReAllocateHeap(ProcessHeap,
//                                                0,
//                                                Info.Buffer,
//                                                BufferSizeRequired);
//                     if (Buffer != NULL)
//                     {
//                         Info.Buffer = Buffer;
//                         BufferSize = BufferSizeRequired;
//                         /* try again */
//                     }
//                     else
//                     {
//                         /* don't break, let's try to copy as many keys as possible */
//                         Status2 = STATUS_INSUFFICIENT_RESOURCES;
//                         Index++;
//
//                         if (NT_SUCCESS(Status))
//                         {
//                             Status = Status2;
//                         }
//                     }
//                 }
//                 else
//                 {
//                     /* break to avoid an infinite loop in case of denied access or
//                        other errors! */
//                     if (Status2 != STATUS_NO_MORE_ENTRIES && NT_SUCCESS(Status))
//                     {
//                         Status = Status2;
//                     }
//
//                     break;
//                 }
//             }
//
//             /* close the handles and remove the entry from the list */
//             if (copyKeys->hKeySrc != hKeySrc)
//             {
//                 NtClose(copyKeys->hKeySrc);
//             }
//             if (copyKeys->hKeyDest != hKeyDest)
//             {
//                 NtClose(copyKeys->hKeyDest);
//             }
//
//             RemoveEntryList(&copyKeys->ListEntry);
//
//             RtlFreeHeap(ProcessHeap,
//                         0,
//                         copyKeys);
//         } while (!IsListEmpty(&copyQueueHead));
//     }
//     else
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//
//     RtlFreeHeap(ProcessHeap,
//                 0,
//                 Info.Buffer);
//
//     return Status;
// }
//
//
// /************************************************************************
//  *  RegCopyTreeW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegCopyTreeW(IN HKEY hKeySrc,
//              IN LPCWSTR lpSubKey  OPTIONAL,
//              IN HKEY hKeyDest)
// {
//     HANDLE DestKeyHandle, KeyHandle, CurKey, SubKeyHandle = NULL;
//     NTSTATUS Status;
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKeySrc);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     Status = MapDefaultKey(&DestKeyHandle,
//                            hKeyDest);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup2;
//     }
//
//     if (lpSubKey != NULL)
//     {
//         OBJECT_ATTRIBUTES ObjectAttributes;
//         UNICODE_STRING SubKeyName;
//
//         RtlInitUnicodeString(&SubKeyName, lpSubKey);
//
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &SubKeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    KeyHandle,
//                                    NULL);
//
//         Status = NtOpenKey(&SubKeyHandle,
//                            KEY_ENUMERATE_SUB_KEYS | KEY_QUERY_VALUE,
//                            &ObjectAttributes);
//         if (!NT_SUCCESS(Status))
//         {
//             goto Cleanup;
//         }
//
//         CurKey = SubKeyHandle;
//     }
//     else
//         CurKey = KeyHandle;
//
//     Status = RegpCopyTree(CurKey,
//                           hKeyDest);
//
//     if (SubKeyHandle != NULL)
//     {
//         NtClose(SubKeyHandle);
//     }
//
// Cleanup:
//     ClosePredefKey(DestKeyHandle);
// Cleanup2:
//     ClosePredefKey(KeyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     return ERROR_SUCCESS;
// }
//
// #ifndef _ADVAPI32_VISTA_
//
// /************************************************************************
//  *  RegCopyTreeA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegCopyTreeA(IN HKEY hKeySrc,
//              IN LPCSTR lpSubKey  OPTIONAL,
//              IN HKEY hKeyDest)
// {
//     UNICODE_STRING SubKeyName = { 0, 0, NULL };
//     LONG Ret;
//
//     if (lpSubKey != NULL &&
//         !RtlCreateUnicodeStringFromAsciiz(&SubKeyName, lpSubKey))
//     {
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     Ret = RegCopyTreeW(hKeySrc,
//                        SubKeyName.Buffer,
//                        hKeyDest);
//
//     RtlFreeUnicodeString(&SubKeyName);
//
//     return Ret;
// }
//
//
// /************************************************************************
//  *  RegConnectRegistryA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegConnectRegistryA(IN LPCSTR lpMachineName,
//                     IN HKEY hKey,
//                     OUT PHKEY phkResult)
// {
//     UNICODE_STRING MachineName = { 0, 0, NULL };
//     LONG Ret;
//
//     if (lpMachineName != NULL &&
//         !RtlCreateUnicodeStringFromAsciiz(&MachineName, lpMachineName))
//     {
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     Ret = RegConnectRegistryW(MachineName.Buffer,
//                               hKey,
//                               phkResult);
//
//     RtlFreeUnicodeString(&MachineName);
//
//     return Ret;
// }
//
//
// /************************************************************************
//  *  RegConnectRegistryW
//  *
//  * @unimplemented
//  */
// LONG WINAPI
// RegConnectRegistryW(LPCWSTR lpMachineName,
//                     HKEY hKey,
//                     PHKEY phkResult)
// {
//     LONG ret;
//
//     TRACE("(%s,%p,%p): stub\n",debugstr_w(lpMachineName),hKey,phkResult);
//
//     if (!lpMachineName || !*lpMachineName)
//     {
//         /* Use the local machine name */
//         ret = RegOpenKeyW( hKey, NULL, phkResult );
//     }
//     else
//     {
//         WCHAR compName[MAX_COMPUTERNAME_LENGTH + 1];
//         DWORD len = sizeof(compName) / sizeof(WCHAR);
//
//         /* MSDN says lpMachineName must start with \\ : not so */
//         if( lpMachineName[0] == '\\' &&  lpMachineName[1] == '\\')
//             lpMachineName += 2;
//
//         if (GetComputerNameW(compName, &len))
//         {
//             if (!_wcsicmp(lpMachineName, compName))
//                 ret = RegOpenKeyW(hKey, NULL, phkResult);
//             else
//             {
//                 FIXME("Connect to %s is not supported.\n",debugstr_w(lpMachineName));
//                 ret = ERROR_BAD_NETPATH;
//             }
//         }
//         else
//             ret = GetLastError();
//     }
//
//     return ret;
// }
//
//
// /************************************************************************
//  *  CreateNestedKey
//  *
//  *  Create key and all necessary intermediate keys
//  */
// static NTSTATUS
// CreateNestedKey(PHKEY KeyHandle,
//                 POBJECT_ATTRIBUTES ObjectAttributes,
//                 PUNICODE_STRING ClassString,
//                 DWORD dwOptions,
//                 REGSAM samDesired,
//                 DWORD *lpdwDisposition)
// {
//     OBJECT_ATTRIBUTES LocalObjectAttributes;
//     UNICODE_STRING LocalKeyName;
//     ULONG Disposition;
//     NTSTATUS Status;
//     ULONG FullNameLength;
//     ULONG Length;
//     PWCHAR Ptr;
//     HANDLE LocalKeyHandle;
//
//     Status = NtCreateKey((PHANDLE) KeyHandle,
//                          samDesired,
//                          ObjectAttributes,
//                          0,
//                          ClassString,
//                          dwOptions,
//                          (PULONG)lpdwDisposition);
//     TRACE("NtCreateKey(%wZ) called (Status %lx)\n", ObjectAttributes->ObjectName, Status);
//     if (Status != STATUS_OBJECT_NAME_NOT_FOUND)
//         return Status;
//
//     /* Copy object attributes */
//     RtlCopyMemory(&LocalObjectAttributes,
//                   ObjectAttributes,
//                   sizeof(OBJECT_ATTRIBUTES));
//     RtlCreateUnicodeString(&LocalKeyName,
//                            ObjectAttributes->ObjectName->Buffer);
//     LocalObjectAttributes.ObjectName = &LocalKeyName;
//     FullNameLength = LocalKeyName.Length / sizeof(WCHAR);
//
//     LocalKeyHandle = NULL;
//
//     /* Remove the last part of the key name and try to create the key again. */
//     while (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//     {
//         Ptr = wcsrchr(LocalKeyName.Buffer, '\\');
//         if (Ptr == NULL || Ptr == LocalKeyName.Buffer)
//         {
//             Status = STATUS_UNSUCCESSFUL;
//             break;
//         }
//
//         *Ptr = (WCHAR)0;
//         LocalKeyName.Length = (USHORT)wcslen(LocalKeyName.Buffer) * sizeof(WCHAR);
//
//         Status = NtCreateKey(&LocalKeyHandle,
//                              KEY_CREATE_SUB_KEY,
//                              &LocalObjectAttributes,
//                              0,
//                              NULL,
//                              0,
//                              &Disposition);
//         TRACE("NtCreateKey(%wZ) called (Status %lx)\n", &LocalKeyName, Status);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeUnicodeString(&LocalKeyName);
//         return Status;
//     }
//
//     /* Add removed parts of the key name and create them too. */
//     Length = wcslen(LocalKeyName.Buffer);
//     while (TRUE)
//     {
//         if (LocalKeyHandle)
//             NtClose (LocalKeyHandle);
//
//         LocalKeyName.Buffer[Length] = L'\\';
//         Length = wcslen (LocalKeyName.Buffer);
//         LocalKeyName.Length = Length * sizeof(WCHAR);
//
//         if (Length == FullNameLength)
//         {
//             Status = NtCreateKey((PHANDLE) KeyHandle,
//                                  samDesired,
//                                  ObjectAttributes,
//                                  0,
//                                  ClassString,
//                                  dwOptions,
//                                  (PULONG)lpdwDisposition);
//             break;
//         }
//
//         Status = NtCreateKey(&LocalKeyHandle,
//                              KEY_CREATE_SUB_KEY,
//                              &LocalObjectAttributes,
//                              0,
//                              NULL,
//                              0,
//                              &Disposition);
//         TRACE("NtCreateKey(%wZ) called (Status %lx)\n", &LocalKeyName, Status);
//         if (!NT_SUCCESS(Status))
//             break;
//     }
//
//     RtlFreeUnicodeString(&LocalKeyName);
//
//     return Status;
// }
//
//
// /************************************************************************
//  *  RegCreateKeyExA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegCreateKeyExA(
//      HKEY hKey,
//      LPCSTR lpSubKey,
//      DWORD Reserved,
//      LPSTR lpClass,
//      DWORD dwOptions,
//      REGSAM samDesired,
//      LPSECURITY_ATTRIBUTES lpSecurityAttributes,
//      PHKEY phkResult,
//      LPDWORD lpdwDisposition)
// {
//     UNICODE_STRING SubKeyString;
//     UNICODE_STRING ClassString;
//     DWORD ErrorCode;
//
//     RtlInitEmptyUnicodeString(&ClassString, NULL, 0);
//     RtlInitEmptyUnicodeString(&SubKeyString, NULL, 0);
//
//     if (lpClass)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&ClassString, lpClass))
//         {
//             ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//     }
//
//     if (lpSubKey)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&SubKeyString, lpSubKey))
//         {
//             ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//     }
//
//     ErrorCode = RegCreateKeyExW(
//         hKey,
//         SubKeyString.Buffer,
//         Reserved,
//         ClassString.Buffer,
//         dwOptions,
//         samDesired,
//         lpSecurityAttributes,
//         phkResult,
//         lpdwDisposition);
//
// Exit:
//     RtlFreeUnicodeString(&SubKeyString);
//     RtlFreeUnicodeString(&ClassString);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegCreateKeyExW
//  *
//  * @implemented
//  */
// LONG
// WINAPI
// RegCreateKeyExW(
//      HKEY hKey,
//      LPCWSTR lpSubKey,
//      DWORD Reserved,
//      LPWSTR lpClass,
//      DWORD dwOptions,
//      REGSAM samDesired,
//      LPSECURITY_ATTRIBUTES lpSecurityAttributes,
//      PHKEY phkResult,
//      LPDWORD lpdwDisposition)
// {
//     UNICODE_STRING SubKeyString;
//     UNICODE_STRING ClassString;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE ParentKey;
//     ULONG Attributes = OBJ_CASE_INSENSITIVE;
//     NTSTATUS Status;
//
//     TRACE("RegCreateKeyExW() called\n");
//
//     /* get the real parent key */
//     Status = MapDefaultKey(&ParentKey,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     TRACE("ParentKey %p\n", ParentKey);
//
//     if (IsHKCRKey(ParentKey))
//     {
//         LONG ErrorCode = CreateHKCRKey(
//             ParentKey,
//             lpSubKey,
//             Reserved,
//             lpClass,
//             dwOptions,
//             samDesired,
//             lpSecurityAttributes,
//             phkResult,
//             lpdwDisposition);
//         ClosePredefKey(ParentKey);
//         return ErrorCode;
//     }
//
//     if (dwOptions & REG_OPTION_OPEN_LINK)
//         Attributes |= OBJ_OPENLINK;
//
//     RtlInitUnicodeString(&ClassString,
//                          lpClass);
//     RtlInitUnicodeString(&SubKeyString,
//                          lpSubKey);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &SubKeyString,
//                                Attributes,
//                                (HANDLE)ParentKey,
//                                lpSecurityAttributes ? (PSECURITY_DESCRIPTOR)lpSecurityAttributes->lpSecurityDescriptor : NULL);
//     Status = CreateNestedKey(phkResult,
//                              &ObjectAttributes,
//                              (lpClass == NULL)? NULL : &ClassString,
//                              dwOptions,
//                              samDesired,
//                              lpdwDisposition);
//
//     ClosePredefKey(ParentKey);
//
//     TRACE("Status %x\n", Status);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /************************************************************************
//  *  RegCreateKeyA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegCreateKeyA(HKEY hKey,
//               LPCSTR lpSubKey,
//               PHKEY phkResult)
// {
//     return RegCreateKeyExA(hKey,
//                            lpSubKey,
//                            0,
//                            NULL,
//                            0,
//                            MAXIMUM_ALLOWED,
//                            NULL,
//                            phkResult,
//                            NULL);
// }
//
//
// /************************************************************************
//  *  RegCreateKeyW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegCreateKeyW(HKEY hKey,
//               LPCWSTR lpSubKey,
//               PHKEY phkResult)
// {
//     return RegCreateKeyExW(hKey,
//                            lpSubKey,
//                            0,
//                            NULL,
//                            0,
//                            MAXIMUM_ALLOWED,
//                            NULL,
//                            phkResult,
//                            NULL);
// }
//
//
// /************************************************************************
//  *  RegDeleteKeyA
//  *
//  * @implemented
//  */
// LONG
// WINAPI
// RegDeleteKeyA(
//      HKEY hKey,
//      LPCSTR lpSubKey)
// {
//     return RegDeleteKeyExA(hKey, lpSubKey, 0, 0);
// }
//
//
// /************************************************************************
//  *  RegDeleteKeyW
//  *
//  * @implemented
//  */
// LONG
// WINAPI
// RegDeleteKeyW(
//      HKEY hKey,
//      LPCWSTR lpSubKey)
// {
//     return RegDeleteKeyExW(hKey, lpSubKey, 0, 0);
// }
//
//
// /************************************************************************
//  *  RegDeleteKeyExA
//  *
//  * @implemented
//  */
// LONG
// WINAPI
// RegDeleteKeyExA(
//      HKEY hKey,
//      LPCSTR lpSubKey,
//      REGSAM samDesired,
//      DWORD Reserved)
// {
//     LONG ErrorCode;
//     UNICODE_STRING SubKeyName;
//
//     if (lpSubKey)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&SubKeyName, lpSubKey))
//             return ERROR_NOT_ENOUGH_MEMORY;
//     }
//     else
//         RtlInitEmptyUnicodeString(&SubKeyName, NULL, 0);
//
//     ErrorCode = RegDeleteKeyExW(hKey, SubKeyName.Buffer, samDesired, Reserved);
//
//     RtlFreeUnicodeString(&SubKeyName);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegDeleteKeyExW
//  *
//  * @implemented
//  */
// LONG
// WINAPI
// RegDeleteKeyExW(
//      HKEY hKey,
//      LPCWSTR lpSubKey,
//      REGSAM samDesired,
//      DWORD Reserved)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING SubKeyName;
//     HANDLE ParentKey;
//     HANDLE TargetKey;
//     NTSTATUS Status;
//
//     /* Make sure we got a subkey */
//     if (!lpSubKey)
//     {
//         /* Fail */
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     Status = MapDefaultKey(&ParentKey,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     if (IsHKCRKey(ParentKey))
//     {
//         LONG ErrorCode = DeleteHKCRKey(ParentKey, lpSubKey, samDesired, Reserved);
//         ClosePredefKey(ParentKey);
//         return ErrorCode;
//     }
//
//     if (samDesired & KEY_WOW64_32KEY)
//         ERR("Wow64 not yet supported!\n");
//
//     if (samDesired & KEY_WOW64_64KEY)
//         ERR("Wow64 not yet supported!\n");
//
//
//     RtlInitUnicodeString(&SubKeyName, lpSubKey);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &SubKeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                ParentKey,
//                                NULL);
//     Status = NtOpenKey(&TargetKey,
//                        DELETE,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     Status = NtDeleteKey(TargetKey);
//     NtClose(TargetKey);
//
// Cleanup:
//     ClosePredefKey(ParentKey);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /************************************************************************
//  *  RegDeleteKeyValueW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegDeleteKeyValueW(IN HKEY hKey,
//                    IN LPCWSTR lpSubKey  OPTIONAL,
//                    IN LPCWSTR lpValueName  OPTIONAL)
// {
//     HKEY hSubKey = hKey;
//     LONG ErrorCode;
//
//     if (lpSubKey)
//     {
//         ErrorCode = RegOpenKeyExW(hKey, lpSubKey, 0, KEY_SET_VALUE, &hSubKey);
//         if (ErrorCode)
//         {
//             return ErrorCode;
//         }
//     }
//     ErrorCode = RegDeleteValueW(hSubKey, lpValueName);
//
//     if (hSubKey != hKey)
//     {
//         RegCloseKey(hSubKey);
//     }
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegDeleteKeyValueA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegDeleteKeyValueA(IN HKEY hKey,
//                    IN LPCSTR lpSubKey  OPTIONAL,
//                    IN LPCSTR lpValueName  OPTIONAL)
// {
//     UNICODE_STRING SubKey = { 0, 0, NULL }, ValueName = { 0, 0, NULL };
//     LONG Ret;
//
//     if (lpSubKey != NULL &&
//         !RtlCreateUnicodeStringFromAsciiz(&SubKey, lpSubKey))
//     {
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     if (lpValueName != NULL &&
//         !RtlCreateUnicodeStringFromAsciiz(&ValueName, lpValueName))
//     {
//         RtlFreeUnicodeString(&SubKey);
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     Ret = RegDeleteKeyValueW(hKey,
//                              SubKey.Buffer,
//                              ValueName.Buffer);
//
//     RtlFreeUnicodeString(&SubKey);
//     RtlFreeUnicodeString(&ValueName);
//
//     return Ret;
// }
//
//
// #endif // _ADVAPI32_VISTA_
//
// /************************************************************************
//  *  RegDeleteTreeW
//  *
//  * @implemented
//  */
// LSTATUS
// WINAPI
// RegDeleteTreeW(HKEY hKey,
//                LPCWSTR lpszSubKey)
// {
//     LONG ret;
//     DWORD dwMaxSubkeyLen, dwMaxValueLen;
//     DWORD dwMaxLen, dwSize;
//     NTSTATUS Status;
//     HANDLE KeyHandle;
//     HKEY hSubKey;
//     WCHAR szNameBuf[MAX_PATH], *lpszName = szNameBuf;
//
//     TRACE("(hkey=%p,%p %s)\n", hKey, lpszSubKey, debugstr_w(lpszSubKey));
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     hSubKey = KeyHandle;
//
//     if(lpszSubKey)
//     {
//         ret = RegOpenKeyExW(KeyHandle, lpszSubKey, 0, KEY_READ, &hSubKey);
//         if (ret)
//         {
//             ClosePredefKey(KeyHandle);
//             return ret;
//         }
//     }
//
//     /* Get highest length for keys, values */
//     ret = RegQueryInfoKeyW(hSubKey, NULL, NULL, NULL, NULL,
//             &dwMaxSubkeyLen, NULL, NULL, &dwMaxValueLen, NULL, NULL, NULL);
//     if (ret) goto cleanup;
//
//     dwMaxSubkeyLen++;
//     dwMaxValueLen++;
//     dwMaxLen = max(dwMaxSubkeyLen, dwMaxValueLen);
//     if (dwMaxLen > sizeof(szNameBuf)/sizeof(WCHAR))
//     {
//         /* Name too big: alloc a buffer for it */
//         if (!(lpszName = RtlAllocateHeap( RtlGetProcessHeap(), 0, dwMaxLen*sizeof(WCHAR))))
//         {
//             ret = ERROR_NOT_ENOUGH_MEMORY;
//             goto cleanup;
//         }
//     }
//
//
//     /* Recursively delete all the subkeys */
//     while (TRUE)
//     {
//         dwSize = dwMaxLen;
//         if (RegEnumKeyExW(hSubKey, 0, lpszName, &dwSize, NULL,
//                           NULL, NULL, NULL)) break;
//
//         ret = RegDeleteTreeW(hSubKey, lpszName);
//         if (ret) goto cleanup;
//     }
//
//     if (lpszSubKey)
//         ret = RegDeleteKeyW(KeyHandle, lpszSubKey);
//     else
//         while (TRUE)
//         {
//             dwSize = dwMaxLen;
//             if (RegEnumValueW(KeyHandle, 0, lpszName, &dwSize,
//                   NULL, NULL, NULL, NULL)) break;
//
//             ret = RegDeleteValueW(KeyHandle, lpszName);
//             if (ret) goto cleanup;
//         }
//
// cleanup:
//     /* Free buffer if allocated */
//     if (lpszName != szNameBuf)
//         RtlFreeHeap( RtlGetProcessHeap(), 0, lpszName);
//     if(lpszSubKey)
//         RegCloseKey(hSubKey);
//
//     ClosePredefKey(KeyHandle);
//
//     return ret;
// }
//
//
// /************************************************************************
//  *  RegDeleteTreeA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegDeleteTreeA(IN HKEY hKey,
//                IN LPCSTR lpSubKey  OPTIONAL)
// {
//     UNICODE_STRING SubKeyName = { 0, 0, NULL };
//     LONG Ret;
//
//     if (lpSubKey != NULL &&
//         !RtlCreateUnicodeStringFromAsciiz(&SubKeyName, lpSubKey))
//     {
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     Ret = RegDeleteTreeW(hKey,
//                          SubKeyName.Buffer);
//
//     RtlFreeUnicodeString(&SubKeyName);
//
//     return Ret;
// }
//
// #ifndef _ADVAPI32_VISTA_
//
// /************************************************************************
//  *  RegDisableReflectionKey
//  *
//  * @unimplemented
//  */
// LONG WINAPI
// RegDisableReflectionKey(IN HKEY hBase)
// {
//     FIXME("RegDisableReflectionKey(0x%p) UNIMPLEMENTED!\n", hBase);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /************************************************************************
//  *  RegEnableReflectionKey
//  *
//  * @unimplemented
//  */
// LONG WINAPI
// RegEnableReflectionKey(IN HKEY hBase)
// {
//     FIXME("RegEnableReflectionKey(0x%p) UNIMPLEMENTED!\n", hBase);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /******************************************************************************
//  * RegpApplyRestrictions   [internal]
//  *
//  * Helper function for RegGetValueA/W.
//  */
// static VOID
// RegpApplyRestrictions(DWORD dwFlags,
//                       DWORD dwType,
//                       DWORD cbData,
//                       PLONG ret)
// {
//     /* Check if the type is restricted by the passed flags */
//     if (*ret == ERROR_SUCCESS || *ret == ERROR_MORE_DATA)
//     {
//         DWORD dwMask = 0;
//
//         switch (dwType)
//         {
//         case REG_NONE: dwMask = RRF_RT_REG_NONE; break;
//         case REG_SZ: dwMask = RRF_RT_REG_SZ; break;
//         case REG_EXPAND_SZ: dwMask = RRF_RT_REG_EXPAND_SZ; break;
//         case REG_MULTI_SZ: dwMask = RRF_RT_REG_MULTI_SZ; break;
//         case REG_BINARY: dwMask = RRF_RT_REG_BINARY; break;
//         case REG_DWORD: dwMask = RRF_RT_REG_DWORD; break;
//         case REG_QWORD: dwMask = RRF_RT_REG_QWORD; break;
//         }
//
//         if (dwFlags & dwMask)
//         {
//             /* Type is not restricted, check for size mismatch */
//             if (dwType == REG_BINARY)
//             {
//                 DWORD cbExpect = 0;
//
//                 if ((dwFlags & RRF_RT_ANY) == RRF_RT_DWORD)
//                     cbExpect = 4;
//                 else if ((dwFlags & RRF_RT_ANY) == RRF_RT_QWORD)
//                     cbExpect = 8;
//
//                 if (cbExpect && cbData != cbExpect)
//                     *ret = ERROR_DATATYPE_MISMATCH;
//             }
//         }
//         else *ret = ERROR_UNSUPPORTED_TYPE;
//     }
// }
//
//
// /******************************************************************************
//  * RegGetValueW   [ADVAPI32.@]
//  *
//  * Retrieves the type and data for a value name associated with a key,
//  * optionally expanding its content and restricting its type.
//  *
//  * PARAMS
//  *  hKey      [I] Handle to an open key.
//  *  pszSubKey [I] Name of the subkey of hKey.
//  *  pszValue  [I] Name of value under hKey/szSubKey to query.
//  *  dwFlags   [I] Flags restricting the value type to retrieve.
//  *  pdwType   [O] Destination for the values type, may be NULL.
//  *  pvData    [O] Destination for the values content, may be NULL.
//  *  pcbData   [I/O] Size of pvData, updated with the size in bytes required to
//  *                  retrieve the whole content, including the trailing '\0'
//  *                  for strings.
//  *
//  * RETURNS
//  *  Success: ERROR_SUCCESS
//  *  Failure: nonzero error code from Winerror.h
//  *
//  * NOTES
//  *  - Unless RRF_NOEXPAND is specified, REG_EXPAND_SZ values are automatically
//  *    expanded and pdwType is set to REG_SZ instead.
//  *  - Restrictions are applied after expanding, using RRF_RT_REG_EXPAND_SZ
//  *    without RRF_NOEXPAND is thus not allowed.
//  *    An exception is the case where RRF_RT_ANY is specified, because then
//  *    RRF_NOEXPAND is allowed.
//  */
// LSTATUS WINAPI
// RegGetValueW(HKEY hKey,
//              LPCWSTR pszSubKey,
//              LPCWSTR pszValue,
//              DWORD dwFlags,
//              LPDWORD pdwType,
//              PVOID pvData,
//              LPDWORD pcbData)
// {
//     DWORD dwType, cbData = pcbData ? *pcbData : 0;
//     PVOID pvBuf = NULL;
//     LONG ret;
//
//     TRACE("(%p,%s,%s,%ld,%p,%p,%p=%ld)\n",
//           hKey, debugstr_w(pszSubKey), debugstr_w(pszValue), dwFlags, pdwType,
//           pvData, pcbData, cbData);
//
//     if (pvData && !pcbData)
//         return ERROR_INVALID_PARAMETER;
//     if ((dwFlags & RRF_RT_REG_EXPAND_SZ) && !(dwFlags & RRF_NOEXPAND) &&
//             ((dwFlags & RRF_RT_ANY) != RRF_RT_ANY))
//         return ERROR_INVALID_PARAMETER;
//
//     if (pszSubKey && pszSubKey[0])
//     {
//         ret = RegOpenKeyExW(hKey, pszSubKey, 0, KEY_QUERY_VALUE, &hKey);
//         if (ret != ERROR_SUCCESS) return ret;
//     }
//
//     ret = RegQueryValueExW(hKey, pszValue, NULL, &dwType, pvData, &cbData);
//
//     /* If we are going to expand we need to read in the whole the value even
//      * if the passed buffer was too small as the expanded string might be
//      * smaller than the unexpanded one and could fit into cbData bytes. */
//     if ((ret == ERROR_SUCCESS || ret == ERROR_MORE_DATA) &&
//         dwType == REG_EXPAND_SZ && !(dwFlags & RRF_NOEXPAND))
//     {
//         do
//         {
//             HeapFree(GetProcessHeap(), 0, pvBuf);
//
//             pvBuf = HeapAlloc(GetProcessHeap(), 0, cbData);
//             if (!pvBuf)
//             {
//                 ret = ERROR_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             if (ret == ERROR_MORE_DATA || !pvData)
//                 ret = RegQueryValueExW(hKey, pszValue, NULL,
//                                        &dwType, pvBuf, &cbData);
//             else
//             {
//                 /* Even if cbData was large enough we have to copy the
//                  * string since ExpandEnvironmentStrings can't handle
//                  * overlapping buffers. */
//                 CopyMemory(pvBuf, pvData, cbData);
//             }
//
//             /* Both the type or the value itself could have been modified in
//              * between so we have to keep retrying until the buffer is large
//              * enough or we no longer have to expand the value. */
//         }
//         while (dwType == REG_EXPAND_SZ && ret == ERROR_MORE_DATA);
//
//         if (ret == ERROR_SUCCESS)
//         {
//             /* Recheck dwType in case it changed since the first call */
//             if (dwType == REG_EXPAND_SZ)
//             {
//                 cbData = ExpandEnvironmentStringsW(pvBuf, pvData,
//                                                    pcbData ? *pcbData : 0) * sizeof(WCHAR);
//                 dwType = REG_SZ;
//                 if (pvData && pcbData && cbData > *pcbData)
//                     ret = ERROR_MORE_DATA;
//             }
//             else if (pvData)
//                 CopyMemory(pvData, pvBuf, *pcbData);
//         }
//
//         HeapFree(GetProcessHeap(), 0, pvBuf);
//     }
//
//     if (pszSubKey && pszSubKey[0])
//         RegCloseKey(hKey);
//
//     RegpApplyRestrictions(dwFlags, dwType, cbData, &ret);
//
//     if (pvData && ret != ERROR_SUCCESS && (dwFlags & RRF_ZEROONFAILURE))
//         ZeroMemory(pvData, *pcbData);
//
//     if (pdwType)
//         *pdwType = dwType;
//
//     if (pcbData)
//         *pcbData = cbData;
//
//     return ret;
// }
//
//
// /******************************************************************************
//  * RegGetValueA   [ADVAPI32.@]
//  *
//  * See RegGetValueW.
//  */
// LSTATUS WINAPI
// RegGetValueA(HKEY hKey,
//              LPCSTR pszSubKey,
//              LPCSTR pszValue,
//              DWORD dwFlags,
//              LPDWORD pdwType,
//              PVOID pvData,
//              LPDWORD pcbData)
// {
//     DWORD dwType, cbData = pcbData ? *pcbData : 0;
//     PVOID pvBuf = NULL;
//     LONG ret;
//
//     TRACE("(%p,%s,%s,%ld,%p,%p,%p=%ld)\n",
//           hKey, pszSubKey, pszValue, dwFlags, pdwType, pvData, pcbData,
//           cbData);
//
//     if (pvData && !pcbData)
//         return ERROR_INVALID_PARAMETER;
//     if ((dwFlags & RRF_RT_REG_EXPAND_SZ) && !(dwFlags & RRF_NOEXPAND) &&
//             ((dwFlags & RRF_RT_ANY) != RRF_RT_ANY))
//         return ERROR_INVALID_PARAMETER;
//
//     if (pszSubKey && pszSubKey[0])
//     {
//         ret = RegOpenKeyExA(hKey, pszSubKey, 0, KEY_QUERY_VALUE, &hKey);
//         if (ret != ERROR_SUCCESS) return ret;
//     }
//
//     ret = RegQueryValueExA(hKey, pszValue, NULL, &dwType, pvData, &cbData);
//
//     /* If we are going to expand we need to read in the whole the value even
//      * if the passed buffer was too small as the expanded string might be
//      * smaller than the unexpanded one and could fit into cbData bytes. */
//     if ((ret == ERROR_SUCCESS || ret == ERROR_MORE_DATA) &&
//         (dwType == REG_EXPAND_SZ && !(dwFlags & RRF_NOEXPAND)))
//     {
//         do {
//             HeapFree(GetProcessHeap(), 0, pvBuf);
//
//             pvBuf = HeapAlloc(GetProcessHeap(), 0, cbData);
//             if (!pvBuf)
//             {
//                 ret = ERROR_NOT_ENOUGH_MEMORY;
//                 break;
//             }
//
//             if (ret == ERROR_MORE_DATA || !pvData)
//                 ret = RegQueryValueExA(hKey, pszValue, NULL,
//                                        &dwType, pvBuf, &cbData);
//             else
//             {
//                 /* Even if cbData was large enough we have to copy the
//                  * string since ExpandEnvironmentStrings can't handle
//                  * overlapping buffers. */
//                 CopyMemory(pvBuf, pvData, cbData);
//             }
//
//             /* Both the type or the value itself could have been modified in
//              * between so we have to keep retrying until the buffer is large
//              * enough or we no longer have to expand the value. */
//         } while (dwType == REG_EXPAND_SZ && ret == ERROR_MORE_DATA);
//
//         if (ret == ERROR_SUCCESS)
//         {
//             /* Recheck dwType in case it changed since the first call */
//             if (dwType == REG_EXPAND_SZ)
//             {
//                 cbData = ExpandEnvironmentStringsA(pvBuf, pvData,
//                                                    pcbData ? *pcbData : 0);
//                 dwType = REG_SZ;
//                 if(pvData && pcbData && cbData > *pcbData)
//                     ret = ERROR_MORE_DATA;
//             }
//             else if (pvData)
//                 CopyMemory(pvData, pvBuf, *pcbData);
//         }
//
//         HeapFree(GetProcessHeap(), 0, pvBuf);
//     }
//
//     if (pszSubKey && pszSubKey[0])
//         RegCloseKey(hKey);
//
//     RegpApplyRestrictions(dwFlags, dwType, cbData, &ret);
//
//     if (pvData && ret != ERROR_SUCCESS && (dwFlags & RRF_ZEROONFAILURE))
//         ZeroMemory(pvData, *pcbData);
//
//     if (pdwType) *pdwType = dwType;
//     if (pcbData) *pcbData = cbData;
//
//     return ret;
// }
//
// #endif // _ADVAPI32_VISTA_
//
// /************************************************************************
//  *  RegSetKeyValueW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegSetKeyValueW(IN HKEY hKey,
//                 IN LPCWSTR lpSubKey  OPTIONAL,
//                 IN LPCWSTR lpValueName  OPTIONAL,
//                 IN DWORD dwType,
//                 IN LPCVOID lpData  OPTIONAL,
//                 IN DWORD cbData)
// {
//     HANDLE KeyHandle, CurKey, SubKeyHandle = NULL;
//     NTSTATUS Status;
//     LONG Ret;
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     if (lpSubKey != NULL)
//     {
//         OBJECT_ATTRIBUTES ObjectAttributes;
//         UNICODE_STRING SubKeyName;
//
//         RtlInitUnicodeString(&SubKeyName, lpSubKey);
//
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &SubKeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    KeyHandle,
//                                    NULL);
//
//         Status = NtOpenKey(&SubKeyHandle,
//                            KEY_SET_VALUE,
//                            &ObjectAttributes);
//         if (!NT_SUCCESS(Status))
//         {
//             Ret = RtlNtStatusToDosError(Status);
//             goto Cleanup;
//         }
//
//         CurKey = SubKeyHandle;
//     }
//     else
//         CurKey = KeyHandle;
//
//     Ret = RegSetValueExW(CurKey,
//                          lpValueName,
//                          0,
//                          dwType,
//                          lpData,
//                          cbData);
//
//     if (SubKeyHandle != NULL)
//     {
//         NtClose(SubKeyHandle);
//     }
//
// Cleanup:
//     ClosePredefKey(KeyHandle);
//
//     return Ret;
// }
//
// #ifndef _ADVAPI32_VISTA_
//
// /************************************************************************
//  *  RegSetKeyValueA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegSetKeyValueA(IN HKEY hKey,
//                 IN LPCSTR lpSubKey  OPTIONAL,
//                 IN LPCSTR lpValueName  OPTIONAL,
//                 IN DWORD dwType,
//                 IN LPCVOID lpData  OPTIONAL,
//                 IN DWORD cbData)
// {
//     HANDLE KeyHandle, CurKey, SubKeyHandle = NULL;
//     NTSTATUS Status;
//     LONG Ret;
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     if (lpSubKey != NULL)
//     {
//         OBJECT_ATTRIBUTES ObjectAttributes;
//         UNICODE_STRING SubKeyName;
//
//         if (!RtlCreateUnicodeStringFromAsciiz(&SubKeyName, lpSubKey))
//         {
//             Ret = ERROR_NOT_ENOUGH_MEMORY;
//             goto Cleanup;
//         }
//
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &SubKeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    KeyHandle,
//                                    NULL);
//
//         Status = NtOpenKey(&SubKeyHandle,
//                            KEY_SET_VALUE,
//                            &ObjectAttributes);
//
//         RtlFreeUnicodeString(&SubKeyName);
//
//         if (!NT_SUCCESS(Status))
//         {
//             Ret = RtlNtStatusToDosError(Status);
//             goto Cleanup;
//         }
//
//         CurKey = SubKeyHandle;
//     }
//     else
//         CurKey = KeyHandle;
//
//     Ret = RegSetValueExA(CurKey,
//                          lpValueName,
//                          0,
//                          dwType,
//                          lpData,
//                          cbData);
//
//     if (SubKeyHandle != NULL)
//     {
//         NtClose(SubKeyHandle);
//     }
//
// Cleanup:
//     ClosePredefKey(KeyHandle);
//
//     return Ret;
// }
//
//
// /************************************************************************
//  *  RegDeleteValueA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegDeleteValueA(HKEY hKey,
//                 LPCSTR lpValueName)
// {
//     UNICODE_STRING ValueName;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//     LONG ErrorCode = ERROR_SUCCESS;
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     if (!RtlCreateUnicodeStringFromAsciiz(&ValueName, lpValueName))
//     {
//         ClosePredefKey(KeyHandle);
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     if (IsHKCRKey(KeyHandle))
//     {
//         ErrorCode = DeleteHKCRValue(KeyHandle, &ValueName);
//     }
//     else
//     {
//         Status = NtDeleteValueKey(KeyHandle, &ValueName);
//         if (!NT_SUCCESS(Status))
//             ErrorCode = RtlNtStatusToDosError(Status);
//     }
//     RtlFreeUnicodeString(&ValueName);
//     ClosePredefKey(KeyHandle);
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegDeleteValueW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegDeleteValueW(HKEY hKey,
//                 LPCWSTR lpValueName)
// {
//     UNICODE_STRING ValueName;
//     NTSTATUS Status;
//     HANDLE KeyHandle;
//     LONG ErrorCode = ERROR_SUCCESS;
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     RtlInitUnicodeString(&ValueName, lpValueName);
//
//     if (IsHKCRKey(KeyHandle))
//     {
//         ErrorCode = DeleteHKCRValue(KeyHandle, &ValueName);
//     }
//     else
//     {
//         Status = NtDeleteValueKey(KeyHandle, &ValueName);
//         if (!NT_SUCCESS(Status))
//             ErrorCode = RtlNtStatusToDosError(Status);
//     }
//     ClosePredefKey(KeyHandle);
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegEnumKeyA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegEnumKeyA(HKEY hKey,
//             DWORD dwIndex,
//             LPSTR lpName,
//             DWORD cbName)
// {
//     DWORD dwLength;
//
//     dwLength = cbName;
//     return RegEnumKeyExA(hKey,
//                          dwIndex,
//                          lpName,
//                          &dwLength,
//                          NULL,
//                          NULL,
//                          NULL,
//                          NULL);
// }
//
//
// /************************************************************************
//  *  RegEnumKeyW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegEnumKeyW(HKEY hKey,
//             DWORD dwIndex,
//             LPWSTR lpName,
//             DWORD cbName)
// {
//     DWORD dwLength;
//
//     dwLength = cbName;
//     return RegEnumKeyExW(hKey,
//                          dwIndex,
//                          lpName,
//                          &dwLength,
//                          NULL,
//                          NULL,
//                          NULL,
//                          NULL);
// }
//
//
// /************************************************************************
//  *  RegEnumKeyExA
//  *
//  * @implemented
//  */
// LONG
// WINAPI
// RegEnumKeyExA(
//      HKEY hKey,
//      DWORD dwIndex,
//      LPSTR lpName,
//      LPDWORD lpcbName,
//      LPDWORD lpReserved,
//      LPSTR lpClass,
//      LPDWORD lpcbClass,
//      PFILETIME lpftLastWriteTime)
// {
//     WCHAR* NameBuffer = NULL;
//     WCHAR* ClassBuffer = NULL;
//     DWORD NameLength, ClassLength;
//     LONG ErrorCode;
//
//     /* Allocate our buffers */
//     if (*lpcbName > 0)
//     {
//         NameLength = *lpcbName;
//         NameBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, *lpcbName * sizeof(WCHAR));
//         if (NameBuffer == NULL)
//         {
//             ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//     }
//
//     if (lpClass)
//     {
//         if (*lpcbClass > 0)
//         {
//             ClassLength = *lpcbClass;
//             ClassBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, *lpcbClass * sizeof(WCHAR));
//             if (ClassBuffer == NULL)
//             {
//                 ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//                 goto Exit;
//             }
//         }
//     }
//
//     /* Do the actual call */
//     ErrorCode = RegEnumKeyExW(
//         hKey,
//         dwIndex,
//         NameBuffer,
//         lpcbName,
//         lpReserved,
//         ClassBuffer,
//         lpcbClass,
//         lpftLastWriteTime);
//
//     if (ErrorCode != ERROR_SUCCESS)
//         goto Exit;
//
//     /* Convert the strings */
//     RtlUnicodeToMultiByteN(lpName, *lpcbName, 0, NameBuffer, *lpcbName * sizeof(WCHAR));
//     /* NULL terminate if we can */
//     if (NameLength > *lpcbName)
//         lpName[*lpcbName] = '\0';
//
//     if (lpClass)
//     {
//         RtlUnicodeToMultiByteN(lpClass, *lpcbClass, 0, NameBuffer, *lpcbClass * sizeof(WCHAR));
//         if (ClassLength > *lpcbClass)
//             lpClass[*lpcbClass] = '\0';
//     }
//
// Exit:
//     if (NameBuffer)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NameBuffer);
//     if (ClassBuffer)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, ClassBuffer);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegEnumKeyExW
//  *
//  * @implemented
//  */
// LONG
// WINAPI
// RegEnumKeyExW(
//      HKEY hKey,
//      DWORD dwIndex,
//      LPWSTR lpName,
//      LPDWORD lpcbName,
//      LPDWORD lpReserved,
//      LPWSTR lpClass,
//      LPDWORD lpcbClass,
//      PFILETIME lpftLastWriteTime)
// {
//     union
//     {
//         KEY_NODE_INFORMATION Node;
//         KEY_BASIC_INFORMATION Basic;
//     } *KeyInfo;
//
//     ULONG BufferSize;
//     ULONG ResultSize;
//     ULONG NameLength;
//     ULONG ClassLength = 0;
//     HANDLE KeyHandle;
//     LONG ErrorCode = ERROR_SUCCESS;
//     NTSTATUS Status;
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     if (IsHKCRKey(KeyHandle))
//     {
//         ErrorCode = EnumHKCRKey(
//             KeyHandle,
//             dwIndex,
//             lpName,
//             lpcbName,
//             lpReserved,
//             lpClass,
//             lpcbClass,
//             lpftLastWriteTime);
//         ClosePredefKey(KeyHandle);
//         return ErrorCode;
//     }
//
//     if (*lpcbName > 0)
//     {
//         NameLength = min (*lpcbName - 1, REG_MAX_NAME_SIZE) * sizeof (WCHAR);
//     }
//     else
//     {
//         NameLength = 0;
//     }
//
//     if (lpClass)
//     {
//         if (*lpcbClass > 0)
//         {
//             ClassLength = min (*lpcbClass - 1, REG_MAX_NAME_SIZE) * sizeof(WCHAR);
//         }
//         else
//         {
//             ClassLength = 0;
//         }
//
//         BufferSize = ((sizeof(KEY_NODE_INFORMATION) + NameLength + 3) & ~3) + ClassLength;
//     }
//     else
//     {
//         BufferSize = sizeof(KEY_BASIC_INFORMATION) + NameLength;
//     }
//
//     KeyInfo = RtlAllocateHeap(ProcessHeap,
//                               0,
//                               BufferSize);
//     if (KeyInfo == NULL)
//     {
//         ErrorCode = ERROR_OUTOFMEMORY;
//         goto Cleanup;
//     }
//
//     Status = NtEnumerateKey(KeyHandle,
//                             (ULONG)dwIndex,
//                             lpClass ? KeyNodeInformation : KeyBasicInformation,
//                             KeyInfo,
//                             BufferSize,
//                             &ResultSize);
//     TRACE("NtEnumerateKey() returned status 0x%X\n", Status);
//     if (!NT_SUCCESS(Status))
//     {
//         ErrorCode = RtlNtStatusToDosError (Status);
//     }
//     else
//     {
//         if (lpClass == NULL)
//         {
//             if (KeyInfo->Basic.NameLength > NameLength)
//             {
//                 ErrorCode = ERROR_MORE_DATA;
//             }
//             else
//             {
//                 RtlCopyMemory(lpName,
//                               KeyInfo->Basic.Name,
//                               KeyInfo->Basic.NameLength);
//                 *lpcbName = (DWORD)(KeyInfo->Basic.NameLength / sizeof(WCHAR));
//                 lpName[*lpcbName] = 0;
//             }
//         }
//         else
//         {
//             if (KeyInfo->Node.NameLength > NameLength ||
//                 KeyInfo->Node.ClassLength > ClassLength)
//             {
//                 ErrorCode = ERROR_MORE_DATA;
//             }
//             else
//             {
//                 RtlCopyMemory(lpName,
//                               KeyInfo->Node.Name,
//                               KeyInfo->Node.NameLength);
//                 *lpcbName = KeyInfo->Node.NameLength / sizeof(WCHAR);
//                 lpName[*lpcbName] = 0;
//                 RtlCopyMemory(lpClass,
//                               (PVOID)((ULONG_PTR)KeyInfo->Node.Name + KeyInfo->Node.ClassOffset),
//                               KeyInfo->Node.ClassLength);
//                 *lpcbClass = (DWORD)(KeyInfo->Node.ClassLength / sizeof(WCHAR));
//                 lpClass[*lpcbClass] = 0;
//             }
//         }
//
//         if (ErrorCode == ERROR_SUCCESS && lpftLastWriteTime != NULL)
//         {
//             if (lpClass == NULL)
//             {
//                 lpftLastWriteTime->dwLowDateTime = KeyInfo->Basic.LastWriteTime.u.LowPart;
//                 lpftLastWriteTime->dwHighDateTime = KeyInfo->Basic.LastWriteTime.u.HighPart;
//             }
//             else
//             {
//                 lpftLastWriteTime->dwLowDateTime = KeyInfo->Node.LastWriteTime.u.LowPart;
//                 lpftLastWriteTime->dwHighDateTime = KeyInfo->Node.LastWriteTime.u.HighPart;
//             }
//         }
//     }
//
//     RtlFreeHeap(ProcessHeap,
//                 0,
//                 KeyInfo);
//
// Cleanup:
//     ClosePredefKey(KeyHandle);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegEnumValueA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegEnumValueA(
//      HKEY hKey,
//      DWORD dwIndex,
//      LPSTR lpName,
//      LPDWORD lpcbName,
//      LPDWORD lpdwReserved,
//      LPDWORD lpdwType,
//      LPBYTE lpData,
//      LPDWORD lpcbData)
// {
//     WCHAR* NameBuffer;
//     DWORD NameBufferSize, NameLength;
//     LONG ErrorCode;
//     DWORD LocalType = REG_NONE;
//     BOOL NameOverflow = FALSE;
//
//     /* Do parameter checks now, once and for all. */
//     if (!lpName || !lpcbName)
//         return ERROR_INVALID_PARAMETER;
//
//     if ((lpData && !lpcbData) || lpdwReserved)
//         return ERROR_INVALID_PARAMETER;
//
//     /* Get the size of the buffer we must use for the first call to RegEnumValueW */
//     ErrorCode = RegQueryInfoKeyW(
//         hKey, NULL, NULL, NULL, NULL, NULL, NULL, NULL, &NameBufferSize, NULL, NULL, NULL);
//     if (ErrorCode != ERROR_SUCCESS)
//         return ErrorCode;
//
//     /* Add space for the null terminator */
//     NameBufferSize++;
//
//     /* Allocate the buffer for the unicode name */
//     NameBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, NameBufferSize * sizeof(WCHAR));
//     if (NameBuffer == NULL)
//     {
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     /*
//      * This code calls RegEnumValueW twice, because we need to know the type of the enumerated value.
//      * So for the first call, we check if we overflow on the name, as we have no way of knowing if this
//      * is an overflow on the data or on the name during the the second call. So the first time, we make the
//      * call with the supplied value. This is merdique, but this is how it is.
//      */
//     NameLength = *lpcbName;
//     ErrorCode = RegEnumValueW(
//         hKey,
//         dwIndex,
//         NameBuffer,
//         &NameLength,
//         NULL,
//         &LocalType,
//         NULL,
//         NULL);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         if (ErrorCode == ERROR_MORE_DATA)
//             NameOverflow = TRUE;
//         else
//             goto Exit;
//     }
//
//     if (is_string(LocalType) && lpcbData)
//     {
//         /* We must allocate a buffer to get the unicode data */
//         DWORD DataBufferSize = *lpcbData * sizeof(WCHAR);
//         WCHAR* DataBuffer = NULL;
//         DWORD DataLength = *lpcbData;
//         LPSTR DataStr = (LPSTR)lpData;
//
//         if (lpData)
//             DataBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, *lpcbData * sizeof(WCHAR));
//
//         /* Do the real call */
//         ErrorCode = RegEnumValueW(
//             hKey,
//             dwIndex,
//             NameBuffer,
//             &NameBufferSize,
//             lpdwReserved,
//             lpdwType,
//             (LPBYTE)DataBuffer,
//             &DataBufferSize);
//
//         *lpcbData = DataBufferSize / sizeof(WCHAR);
//
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             RtlFreeHeap(RtlGetProcessHeap(), 0, DataBuffer);
//             goto Exit;
//         }
//
//         /* Copy the data whatever the error code is */
//         if (lpData)
//         {
//             /* Do the data conversion */
//             RtlUnicodeToMultiByteN(DataStr, DataLength, 0, DataBuffer, DataBufferSize);
//             /* NULL-terminate if there is enough room */
//             if ((DataLength > *lpcbData) && (DataStr[*lpcbData - 1] != '\0'))
//                 DataStr[*lpcbData] = '\0';
//         }
//
//         RtlFreeHeap(RtlGetProcessHeap(), 0, DataBuffer);
//     }
//     else
//     {
//         /* No data conversion needed. Do the call with provided buffers */
//         ErrorCode = RegEnumValueW(
//             hKey,
//             dwIndex,
//             NameBuffer,
//             &NameBufferSize,
//             lpdwReserved,
//             lpdwType,
//             lpData,
//             lpcbData);
//
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             goto Exit;
//         }
//     }
//
//     if (NameOverflow)
//     {
//         ErrorCode = ERROR_MORE_DATA;
//         goto Exit;
//     }
//
//     /* Convert the name string */
//     RtlUnicodeToMultiByteN(lpName, *lpcbName, lpcbName, NameBuffer, NameBufferSize * sizeof(WCHAR));
//     lpName[*lpcbName] = ANSI_NULL;
//
// Exit:
//     if (NameBuffer)
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NameBuffer);
//
//     return ErrorCode;
// }
//
//
// /******************************************************************************
//  * RegEnumValueW   [ADVAPI32.@]
//  * @implemented
//  *
//  * PARAMS
//  *  hkey       [I] Handle to key to query
//  *  index      [I] Index of value to query
//  *  value      [O] Value string
//  *  val_count  [I/O] Size of value buffer (in wchars)
//  *  reserved   [I] Reserved
//  *  type       [O] Type code
//  *  data       [O] Value data
//  *  count      [I/O] Size of data buffer (in bytes)
//  *
//  * RETURNS
//  *  Success: ERROR_SUCCESS
//  *  Failure: nonzero error code from Winerror.h
//  */
// LONG
// WINAPI
// RegEnumValueW(
//      HKEY hKey,
//      DWORD index,
//      LPWSTR value,
//      PDWORD val_count,
//      PDWORD reserved,
//      PDWORD type,
//      LPBYTE data,
//      PDWORD count)
// {
//     HANDLE KeyHandle;
//     NTSTATUS status;
//     ULONG total_size;
//     char buffer[256], *buf_ptr = buffer;
//     KEY_VALUE_FULL_INFORMATION *info = (KEY_VALUE_FULL_INFORMATION *)buffer;
//     static const int info_size = FIELD_OFFSET( KEY_VALUE_FULL_INFORMATION, Name );
//
//     TRACE("(%p,%ld,%p,%p,%p,%p,%p,%p)\n",
//           hKey, index, value, val_count, reserved, type, data, count );
//
//     if (!value || !val_count)
//         return ERROR_INVALID_PARAMETER;
//
//     if ((data && !count) || reserved)
//         return ERROR_INVALID_PARAMETER;
//
//     status = MapDefaultKey(&KeyHandle, hKey);
//     if (!NT_SUCCESS(status))
//     {
//         return RtlNtStatusToDosError(status);
//     }
//
//     if (IsHKCRKey(KeyHandle))
//     {
//         LONG ErrorCode = EnumHKCRValue(
//             KeyHandle,
//             index,
//             value,
//             val_count,
//             reserved,
//             type,
//             data,
//             count);
//         ClosePredefKey(KeyHandle);
//         return ErrorCode;
//     }
//
//     total_size = info_size + (MAX_PATH + 1) * sizeof(WCHAR);
//     if (data) total_size += *count;
//     total_size = min( sizeof(buffer), total_size );
//
//     status = NtEnumerateValueKey( KeyHandle, index, KeyValueFullInformation,
//                                   buffer, total_size, &total_size );
//     if (status && (status != STATUS_BUFFER_OVERFLOW) && (status != STATUS_BUFFER_TOO_SMALL)) goto done;
//
//     if (value || data)
//     {
//         /* retry with a dynamically allocated buffer */
//         while ((status == STATUS_BUFFER_OVERFLOW) || (status == STATUS_BUFFER_TOO_SMALL))
//         {
//             if (buf_ptr != buffer) HeapFree( GetProcessHeap(), 0, buf_ptr );
//             if (!(buf_ptr = HeapAlloc( GetProcessHeap(), 0, total_size )))
//             {
//                 status = ERROR_NOT_ENOUGH_MEMORY;
//                 goto done;
//             }
//             info = (KEY_VALUE_FULL_INFORMATION *)buf_ptr;
//             status = NtEnumerateValueKey( KeyHandle, index, KeyValueFullInformation,
//                                           buf_ptr, total_size, &total_size );
//         }
//
//         if (status) goto done;
//
//         if (value)
//         {
//             if (info->NameLength/sizeof(WCHAR) >= *val_count)
//             {
//                 status = STATUS_BUFFER_OVERFLOW;
//                 goto overflow;
//             }
//             memcpy( value, info->Name, info->NameLength );
//             *val_count = info->NameLength / sizeof(WCHAR);
//             value[*val_count] = 0;
//         }
//
//         if (data)
//         {
//             if (info->DataLength > *count)
//             {
//                 status = STATUS_BUFFER_OVERFLOW;
//                 goto overflow;
//             }
//             memcpy( data, buf_ptr + info->DataOffset, info->DataLength );
//             if (is_string(info->Type) && info->DataLength <= *count - sizeof(WCHAR))
//             {
//                 /* if the type is REG_SZ and data is not 0-terminated
//                  * and there is enough space in the buffer NT appends a \0 */
//                 WCHAR *ptr = (WCHAR *)(data + info->DataLength);
//                 if (ptr > (WCHAR *)data && ptr[-1]) *ptr = 0;
//             }
//         }
//     }
//     else status = STATUS_SUCCESS;
//
//  overflow:
//     if (type) *type = info->Type;
//     if (count) *count = info->DataLength;
//
//  done:
//     if (buf_ptr != buffer) HeapFree( GetProcessHeap(), 0, buf_ptr );
//     ClosePredefKey(KeyHandle);
//     return RtlNtStatusToDosError(status);
// }
//
//
// /************************************************************************
//  *  RegFlushKey
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegFlushKey(HKEY hKey)
// {
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//
//     if (hKey == HKEY_PERFORMANCE_DATA)
//     {
//         return ERROR_SUCCESS;
//     }
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     Status = NtFlushKey(KeyHandle);
//
//     ClosePredefKey(KeyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /************************************************************************
//  *  RegGetKeySecurity
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegGetKeySecurity(HKEY hKey,
//                   SECURITY_INFORMATION SecurityInformation,
//                   PSECURITY_DESCRIPTOR pSecurityDescriptor,
//                   LPDWORD lpcbSecurityDescriptor)
// {
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//
//     if (hKey == HKEY_PERFORMANCE_DATA)
//     {
//         return ERROR_INVALID_HANDLE;
//     }
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("MapDefaultKey() failed (Status %lx)\n", Status);
//         return RtlNtStatusToDosError(Status);
//     }
//
//     Status = NtQuerySecurityObject(KeyHandle,
//                                    SecurityInformation,
//                                    pSecurityDescriptor,
//                                    *lpcbSecurityDescriptor,
//                                    lpcbSecurityDescriptor);
//
//     ClosePredefKey(KeyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//          WARN("NtQuerySecurityObject() failed (Status %lx)\n", Status);
//          return RtlNtStatusToDosError(Status);
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /************************************************************************
//  *  RegLoadKeyA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegLoadKeyA(HKEY hKey,
//             LPCSTR lpSubKey,
//             LPCSTR lpFile)
// {
//     UNICODE_STRING FileName;
//     UNICODE_STRING KeyName;
//     LONG ErrorCode;
//
//     RtlInitEmptyUnicodeString(&KeyName, NULL, 0);
//     RtlInitEmptyUnicodeString(&FileName, NULL, 0);
//
//     if (lpSubKey)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&KeyName, lpSubKey))
//         {
//             ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//     }
//
//     if (lpFile)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&FileName, lpFile))
//         {
//             ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//     }
//
//     ErrorCode = RegLoadKeyW(hKey,
//                             KeyName.Buffer,
//                             FileName.Buffer);
//
// Exit:
//     RtlFreeUnicodeString(&FileName);
//     RtlFreeUnicodeString(&KeyName);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegLoadKeyW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegLoadKeyW(HKEY hKey,
//             LPCWSTR lpSubKey,
//             LPCWSTR lpFile)
// {
//     OBJECT_ATTRIBUTES FileObjectAttributes;
//     OBJECT_ATTRIBUTES KeyObjectAttributes;
//     UNICODE_STRING FileName;
//     UNICODE_STRING KeyName;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//     LONG ErrorCode = ERROR_SUCCESS;
//
//     if (hKey == HKEY_PERFORMANCE_DATA)
//     {
//         return ERROR_INVALID_HANDLE;
//     }
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     if (!RtlDosPathNameToNtPathName_U(lpFile,
//                                       &FileName,
//                                       NULL,
//                                       NULL))
//     {
//       ErrorCode = ERROR_BAD_PATHNAME;
//       goto Cleanup;
//     }
//
//     InitializeObjectAttributes(&FileObjectAttributes,
//                                &FileName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     RtlInitUnicodeString(&KeyName, lpSubKey);
//
//     InitializeObjectAttributes(&KeyObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                KeyHandle,
//                                NULL);
//
//     Status = NtLoadKey(&KeyObjectAttributes,
//                        &FileObjectAttributes);
//
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 FileName.Buffer);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ErrorCode = RtlNtStatusToDosError(Status);
//         goto Cleanup;
//     }
//
// Cleanup:
//     ClosePredefKey(KeyHandle);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegNotifyChangeKeyValue
//  *
//  * @unimplemented
//  */
// LONG WINAPI
// RegNotifyChangeKeyValue(HKEY hKey,
//                         BOOL bWatchSubtree,
//                         DWORD dwNotifyFilter,
//                         HANDLE hEvent,
//                         BOOL fAsynchronous)
// {
//     IO_STATUS_BLOCK IoStatusBlock;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//     LONG ErrorCode = ERROR_SUCCESS;
//
//     if (hKey == HKEY_PERFORMANCE_DATA)
//     {
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if ((fAsynchronous != FALSE) && (hEvent == NULL))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     /* FIXME: Remote key handles must fail */
//
//     Status = NtNotifyChangeKey(KeyHandle,
//                                hEvent,
//                                0,
//                                0,
//                                &IoStatusBlock,
//                                dwNotifyFilter,
//                                bWatchSubtree,
//                                0,
//                                0,
//                                fAsynchronous);
//     if (!NT_SUCCESS(Status) && Status != STATUS_TIMEOUT)
//     {
//         ErrorCode = RtlNtStatusToDosError(Status);
//     }
//
//     ClosePredefKey(KeyHandle);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegOpenCurrentUser
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegOpenCurrentUser(IN REGSAM samDesired,
//                    OUT PHKEY phkResult)
// {
//     NTSTATUS Status;
//
//     Status = RtlOpenCurrentUser((ACCESS_MASK)samDesired,
//                                 (PHANDLE)phkResult);
//     if (!NT_SUCCESS(Status))
//     {
//         /* NOTE - don't set the last error code! just return the error! */
//         return RtlNtStatusToDosError(Status);
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /************************************************************************
//  *  RegOpenKeyA
//  *
//  *  20050503 Fireball - imported from WINE
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegOpenKeyA(HKEY hKey,
//             LPCSTR lpSubKey,
//             PHKEY phkResult)
// {
//     TRACE("RegOpenKeyA hKey 0x%x lpSubKey %s phkResult %p\n",
//           hKey, lpSubKey, phkResult);
//
//     if (!phkResult)
//         return ERROR_INVALID_PARAMETER;
//
//     if (!hKey && !lpSubKey)
//     {
//         *phkResult = hKey;
//         return ERROR_SUCCESS;
//     }
//
//     return RegOpenKeyExA(hKey,
//                          lpSubKey,
//                          0,
//                          MAXIMUM_ALLOWED,
//                          phkResult);
// }
//
//
// /************************************************************************
//  *  RegOpenKeyW
//  *
//  *  19981101 Ariadne
//  *  19990525 EA
//  *  20050503 Fireball - imported from WINE
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegOpenKeyW(HKEY hKey,
//             LPCWSTR lpSubKey,
//             PHKEY phkResult)
// {
//     TRACE("RegOpenKeyW hKey 0x%x lpSubKey %S phkResult %p\n",
//           hKey, lpSubKey, phkResult);
//
//     if (!phkResult)
//         return ERROR_INVALID_PARAMETER;
//
//     if (!hKey && !lpSubKey)
//     {
//         *phkResult = hKey;
//         return ERROR_SUCCESS;
//     }
//
//     return RegOpenKeyExW(hKey,
//                          lpSubKey,
//                          0,
//                          MAXIMUM_ALLOWED,
//                          phkResult);
// }
//
//
// /************************************************************************
//  *  RegOpenKeyExA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegOpenKeyExA(
//      HKEY hKey,
//      LPCSTR lpSubKey,
//      DWORD ulOptions,
//      REGSAM samDesired,
//      PHKEY phkResult)
// {
//     UNICODE_STRING SubKeyString;
//     LONG ErrorCode;
//
//     TRACE("RegOpenKeyExA hKey 0x%x lpSubKey %s ulOptions 0x%x samDesired 0x%x phkResult %p\n",
//           hKey, lpSubKey, ulOptions, samDesired, phkResult);
//
//     if (lpSubKey)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&SubKeyString, lpSubKey))
//             return ERROR_NOT_ENOUGH_MEMORY;
//     }
//     else
//         RtlInitEmptyUnicodeString(&SubKeyString, NULL, 0);
//
//     ErrorCode = RegOpenKeyExW(hKey, SubKeyString.Buffer, ulOptions, samDesired, phkResult);
//
//     RtlFreeUnicodeString(&SubKeyString);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegOpenKeyExW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegOpenKeyExW(HKEY hKey,
//               LPCWSTR lpSubKey,
//               DWORD ulOptions,
//               REGSAM samDesired,
//               PHKEY phkResult)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING SubKeyString;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//     ULONG Attributes = OBJ_CASE_INSENSITIVE;
//     LONG ErrorCode = ERROR_SUCCESS;
//     BOOLEAN SubKeyStringAllocated = FALSE;
//
//     TRACE("RegOpenKeyExW hKey 0x%x lpSubKey %S ulOptions 0x%x samDesired 0x%x phkResult %p\n",
//           hKey, lpSubKey, ulOptions, samDesired, phkResult);
//     if (!phkResult)
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     if (!hKey && lpSubKey && phkResult)
//     {
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (IsPredefKey(hKey) && (!lpSubKey || !*lpSubKey))
//     {
//         *phkResult = hKey;
//         return ERROR_SUCCESS;
//     }
//
//     Status = MapDefaultKey(&KeyHandle, hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     if (IsHKCRKey(KeyHandle))
//     {
//         ErrorCode = OpenHKCRKey(KeyHandle, lpSubKey, ulOptions, samDesired, phkResult);
//         ClosePredefKey(KeyHandle);
//         return ErrorCode;
//     }
//
//     if (ulOptions & REG_OPTION_OPEN_LINK)
//         Attributes |= OBJ_OPENLINK;
//
//     if (lpSubKey == NULL || wcscmp(lpSubKey, L"\\") == 0)
//     {
//         RtlInitUnicodeString(&SubKeyString, L"");
//     }
//     else
//     {
//         RtlInitUnicodeString(&SubKeyString, lpSubKey);
//
//         /* Handle unaligned lpSubKey */
//         if ((ULONG_PTR)lpSubKey & 1)
//         {
//             UNICODE_STRING AlignedString;
//
//             Status = RtlDuplicateUnicodeString(RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
//                                                &SubKeyString,
//                                                &AlignedString);
//             if (!NT_SUCCESS(Status))
//             {
//                 goto Exit;
//             }
//
//             SubKeyString = AlignedString;
//             SubKeyStringAllocated = TRUE;
//         }
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &SubKeyString,
//                                Attributes,
//                                KeyHandle,
//                                NULL);
//
//     Status = NtOpenKey((PHANDLE)phkResult,
//                        samDesired,
//                        &ObjectAttributes);
//
// Exit:
//
//     if (SubKeyStringAllocated)
//     {
//         RtlFreeUnicodeString(&SubKeyString);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         ErrorCode = RtlNtStatusToDosError(Status);
//     }
//
//     ClosePredefKey(KeyHandle);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegOpenUserClassesRoot
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegOpenUserClassesRoot(IN HANDLE hToken,
//                        IN DWORD dwOptions,
//                        IN REGSAM samDesired,
//                        OUT PHKEY phkResult)
// {
//     const WCHAR UserClassesKeyPrefix[] = L"\\Registry\\User\\";
//     const WCHAR UserClassesKeySuffix[] = L"_Classes";
//     PTOKEN_USER TokenUserData;
//     ULONG RequiredLength;
//     UNICODE_STRING UserSidString, UserClassesKeyRoot;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//
//     /* check parameters */
//     if (hToken == NULL || dwOptions != 0 || phkResult == NULL)
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /*
//      * Get the user sid from the token
//      */
//
// ReadTokenSid:
//     /* determine how much memory we need */
//     Status = NtQueryInformationToken(hToken,
//                                      TokenUser,
//                                      NULL,
//                                      0,
//                                      &RequiredLength);
//     if (!NT_SUCCESS(Status) && (Status != STATUS_BUFFER_TOO_SMALL))
//     {
//         /* NOTE - as opposed to all other registry functions windows does indeed
//                   change the last error code in case the caller supplied a invalid
//                   handle for example! */
//         return RtlNtStatusToDosError(Status);
//     }
//     RegInitialize(); /* HACK until delay-loading is implemented */
//     TokenUserData = RtlAllocateHeap(ProcessHeap,
//                                     0,
//                                     RequiredLength);
//     if (TokenUserData == NULL)
//     {
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     /* attempt to read the information */
//     Status = NtQueryInformationToken(hToken,
//                                      TokenUser,
//                                      TokenUserData,
//                                      RequiredLength,
//                                      &RequiredLength);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeHeap(ProcessHeap,
//                     0,
//                     TokenUserData);
//         if (Status == STATUS_BUFFER_TOO_SMALL)
//         {
//             /* the information appears to have changed?! try again */
//             goto ReadTokenSid;
//         }
//
//         /* NOTE - as opposed to all other registry functions windows does indeed
//                   change the last error code in case the caller supplied a invalid
//                   handle for example! */
//         return RtlNtStatusToDosError(Status);
//     }
//
//     /*
//      * Build the absolute path for the user's registry in the form
//      * "\Registry\User\<SID>_Classes"
//      */
//     Status = RtlConvertSidToUnicodeString(&UserSidString,
//                                           TokenUserData->User.Sid,
//                                           TRUE);
//
//     /* we don't need the user data anymore, free it */
//     RtlFreeHeap(ProcessHeap,
//                 0,
//                 TokenUserData);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     /* allocate enough memory for the entire key string */
//     UserClassesKeyRoot.Length = 0;
//     UserClassesKeyRoot.MaximumLength = UserSidString.Length +
//                                        sizeof(UserClassesKeyPrefix) +
//                                        sizeof(UserClassesKeySuffix);
//     UserClassesKeyRoot.Buffer = RtlAllocateHeap(ProcessHeap,
//                                                 0,
//                                                 UserClassesKeyRoot.MaximumLength);
//     if (UserClassesKeyRoot.Buffer == NULL)
//     {
//         RtlFreeUnicodeString(&UserSidString);
//         return RtlNtStatusToDosError(Status);
//     }
//
//     /* build the string */
//     RtlAppendUnicodeToString(&UserClassesKeyRoot,
//                              UserClassesKeyPrefix);
//     RtlAppendUnicodeStringToString(&UserClassesKeyRoot,
//                                    &UserSidString);
//     RtlAppendUnicodeToString(&UserClassesKeyRoot,
//                              UserClassesKeySuffix);
//
//     TRACE("RegOpenUserClassesRoot: Absolute path: %wZ\n", &UserClassesKeyRoot);
//
//     /*
//      * Open the key
//      */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &UserClassesKeyRoot,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenKey((PHANDLE)phkResult,
//                        samDesired,
//                        &ObjectAttributes);
//
//     RtlFreeUnicodeString(&UserSidString);
//     RtlFreeUnicodeString(&UserClassesKeyRoot);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /************************************************************************
//  *  RegQueryInfoKeyA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegQueryInfoKeyA(HKEY hKey,
//                  LPSTR lpClass,
//                  LPDWORD lpcClass,
//                  LPDWORD lpReserved,
//                  LPDWORD lpcSubKeys,
//                  LPDWORD lpcMaxSubKeyLen,
//                  LPDWORD lpcMaxClassLen,
//                  LPDWORD lpcValues,
//                  LPDWORD lpcMaxValueNameLen,
//                  LPDWORD lpcMaxValueLen,
//                  LPDWORD lpcbSecurityDescriptor,
//                  PFILETIME lpftLastWriteTime)
// {
//     WCHAR ClassName[MAX_PATH];
//     UNICODE_STRING UnicodeString;
//     ANSI_STRING AnsiString;
//     LONG ErrorCode;
//     NTSTATUS Status;
//     DWORD cClass = 0;
//
//     if ((lpClass) && (!lpcClass))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     RtlInitUnicodeString(&UnicodeString,
//                          NULL);
//     if (lpClass != NULL)
//     {
//         RtlInitEmptyUnicodeString(&UnicodeString,
//                                   ClassName,
//                                   sizeof(ClassName));
//         cClass = sizeof(ClassName) / sizeof(WCHAR);
//     }
//
//     ErrorCode = RegQueryInfoKeyW(hKey,
//                                  UnicodeString.Buffer,
//                                  &cClass,
//                                  lpReserved,
//                                  lpcSubKeys,
//                                  lpcMaxSubKeyLen,
//                                  lpcMaxClassLen,
//                                  lpcValues,
//                                  lpcMaxValueNameLen,
//                                  lpcMaxValueLen,
//                                  lpcbSecurityDescriptor,
//                                  lpftLastWriteTime);
//     if ((ErrorCode == ERROR_SUCCESS) && (lpClass != NULL))
//     {
//         if (*lpcClass == 0)
//         {
//             return ErrorCode;
//         }
//
//         RtlInitEmptyAnsiString(&AnsiString, lpClass, *lpcClass);
//         UnicodeString.Length = cClass * sizeof(WCHAR);
//         Status = RtlUnicodeStringToAnsiString(&AnsiString,
//                                               &UnicodeString,
//                                               FALSE);
//         ErrorCode = RtlNtStatusToDosError(Status);
//         cClass = AnsiString.Length;
//         lpClass[cClass] = ANSI_NULL;
//     }
//
//     if (lpcClass != NULL)
//     {
//         *lpcClass = cClass;
//     }
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegQueryInfoKeyW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegQueryInfoKeyW(HKEY hKey,
//                  LPWSTR lpClass,
//                  LPDWORD lpcClass,
//                  LPDWORD lpReserved,
//                  LPDWORD lpcSubKeys,
//                  LPDWORD lpcMaxSubKeyLen,
//                  LPDWORD lpcMaxClassLen,
//                  LPDWORD lpcValues,
//                  LPDWORD lpcMaxValueNameLen,
//                  LPDWORD lpcMaxValueLen,
//                  LPDWORD lpcbSecurityDescriptor,
//                  PFILETIME lpftLastWriteTime)
// {
//     KEY_FULL_INFORMATION FullInfoBuffer;
//     PKEY_FULL_INFORMATION FullInfo;
//     ULONG FullInfoSize;
//     ULONG ClassLength = 0;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//     ULONG Length;
//     LONG ErrorCode = ERROR_SUCCESS;
//
//     if ((lpClass) && (!lpcClass))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     if (IsHKCRKey(KeyHandle))
//     {
//         ErrorCode = QueryInfoHKCRKey(KeyHandle, lpClass, lpcClass, lpReserved,
//                                      lpcSubKeys, lpcMaxSubKeyLen, lpcMaxClassLen,
//                                      lpcValues, lpcMaxValueNameLen, lpcMaxValueLen,
//                                      lpcbSecurityDescriptor, lpftLastWriteTime);
//         ClosePredefKey(KeyHandle);
//         return ErrorCode;
//     }
//
//     if (lpClass != NULL)
//     {
//         if (*lpcClass > 0)
//         {
//             ClassLength = min(*lpcClass - 1, REG_MAX_NAME_SIZE) * sizeof(WCHAR);
//         }
//         else
//         {
//             ClassLength = 0;
//         }
//
//         FullInfoSize = sizeof(KEY_FULL_INFORMATION) + ((ClassLength + 3) & ~3);
//         FullInfo = RtlAllocateHeap(ProcessHeap,
//                                    0,
//                                    FullInfoSize);
//         if (FullInfo == NULL)
//         {
//             ErrorCode = ERROR_OUTOFMEMORY;
//             goto Cleanup;
//         }
//     }
//     else
//     {
//         FullInfoSize = sizeof(KEY_FULL_INFORMATION);
//         FullInfo = &FullInfoBuffer;
//     }
//
//     if (lpcbSecurityDescriptor != NULL)
//         *lpcbSecurityDescriptor = 0;
//
//     Status = NtQueryKey(KeyHandle,
//                         KeyFullInformation,
//                         FullInfo,
//                         FullInfoSize,
//                         &Length);
//     TRACE("NtQueryKey() returned status 0x%X\n", Status);
//     if (!NT_SUCCESS(Status) && Status != STATUS_BUFFER_OVERFLOW)
//     {
//         ErrorCode = RtlNtStatusToDosError(Status);
//         goto Cleanup;
//     }
//
//     TRACE("SubKeys %d\n", FullInfo->SubKeys);
//     if (lpcSubKeys != NULL)
//     {
//         *lpcSubKeys = FullInfo->SubKeys;
//     }
//
//     TRACE("MaxNameLen %lu\n", FullInfo->MaxNameLen);
//     if (lpcMaxSubKeyLen != NULL)
//     {
//         *lpcMaxSubKeyLen = FullInfo->MaxNameLen / sizeof(WCHAR);
//     }
//
//     TRACE("MaxClassLen %lu\n", FullInfo->MaxClassLen);
//     if (lpcMaxClassLen != NULL)
//     {
//         *lpcMaxClassLen = FullInfo->MaxClassLen / sizeof(WCHAR);
//     }
//
//     TRACE("Values %lu\n", FullInfo->Values);
//     if (lpcValues != NULL)
//     {
//         *lpcValues = FullInfo->Values;
//     }
//
//     TRACE("MaxValueNameLen %lu\n", FullInfo->MaxValueNameLen);
//     if (lpcMaxValueNameLen != NULL)
//     {
//         *lpcMaxValueNameLen = FullInfo->MaxValueNameLen / sizeof(WCHAR);
//     }
//
//     TRACE("MaxValueDataLen %lu\n", FullInfo->MaxValueDataLen);
//     if (lpcMaxValueLen != NULL)
//     {
//         *lpcMaxValueLen = FullInfo->MaxValueDataLen;
//     }
//
//     if (lpcbSecurityDescriptor != NULL)
//     {
//         Status = NtQuerySecurityObject(KeyHandle,
//                                        OWNER_SECURITY_INFORMATION |
//                                        GROUP_SECURITY_INFORMATION |
//                                        DACL_SECURITY_INFORMATION,
//                                        NULL,
//                                        0,
//                                        lpcbSecurityDescriptor);
//         TRACE("NtQuerySecurityObject() returned status 0x%X\n", Status);
//     }
//
//     if (lpftLastWriteTime != NULL)
//     {
//         lpftLastWriteTime->dwLowDateTime = FullInfo->LastWriteTime.u.LowPart;
//         lpftLastWriteTime->dwHighDateTime = FullInfo->LastWriteTime.u.HighPart;
//     }
//
//     if (lpClass != NULL)
//     {
//         if (*lpcClass == 0)
//         {
//             goto Cleanup;
//         }
//
//         if (FullInfo->ClassLength > ClassLength)
//         {
//             ErrorCode = ERROR_INSUFFICIENT_BUFFER;
//         }
//         else
//         {
//             RtlCopyMemory(lpClass,
//                           FullInfo->Class,
//                           FullInfo->ClassLength);
//             lpClass[FullInfo->ClassLength / sizeof(WCHAR)] = UNICODE_NULL;
//         }
//     }
//
//     if (lpcClass != NULL)
//     {
//         *lpcClass = FullInfo->ClassLength / sizeof(WCHAR);
//     }
//
// Cleanup:
//     if (lpClass != NULL)
//     {
//         RtlFreeHeap(ProcessHeap,
//                     0,
//                     FullInfo);
//     }
//
//     ClosePredefKey(KeyHandle);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegQueryMultipleValuesA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegQueryMultipleValuesA(HKEY hKey,
//                         PVALENTA val_list,
//                         DWORD num_vals,
//                         LPSTR lpValueBuf,
//                         LPDWORD ldwTotsize)
// {
//     ULONG i;
//     DWORD maxBytes = *ldwTotsize;
//     LPSTR bufptr = lpValueBuf;
//     LONG ErrorCode;
//
//     if (maxBytes >= (1024*1024))
//         return ERROR_MORE_DATA;
//
//     *ldwTotsize = 0;
//
//     TRACE("RegQueryMultipleValuesA(%p,%p,%ld,%p,%p=%ld)\n",
//           hKey, val_list, num_vals, lpValueBuf, ldwTotsize, *ldwTotsize);
//
//     for (i = 0; i < num_vals; i++)
//     {
//         val_list[i].ve_valuelen = 0;
//         ErrorCode = RegQueryValueExA(hKey,
//                                      val_list[i].ve_valuename,
//                                      NULL,
//                                      NULL,
//                                      NULL,
//                                      &val_list[i].ve_valuelen);
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             return ErrorCode;
//         }
//
//         if (lpValueBuf != NULL && *ldwTotsize + val_list[i].ve_valuelen <= maxBytes)
//         {
//             ErrorCode = RegQueryValueExA(hKey,
//                                          val_list[i].ve_valuename,
//                                          NULL,
//                                          &val_list[i].ve_type,
//                                          (LPBYTE)bufptr,
//                                          &val_list[i].ve_valuelen);
//             if (ErrorCode != ERROR_SUCCESS)
//             {
//                 return ErrorCode;
//             }
//
//             val_list[i].ve_valueptr = (DWORD_PTR)bufptr;
//
//             bufptr += val_list[i].ve_valuelen;
//         }
//
//         *ldwTotsize += val_list[i].ve_valuelen;
//     }
//
//     return (lpValueBuf != NULL && *ldwTotsize <= maxBytes) ? ERROR_SUCCESS : ERROR_MORE_DATA;
// }
//
//
// /************************************************************************
//  *  RegQueryMultipleValuesW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegQueryMultipleValuesW(HKEY hKey,
//                         PVALENTW val_list,
//                         DWORD num_vals,
//                         LPWSTR lpValueBuf,
//                         LPDWORD ldwTotsize)
// {
//     ULONG i;
//     DWORD maxBytes = *ldwTotsize;
//     LPSTR bufptr = (LPSTR)lpValueBuf;
//     LONG ErrorCode;
//
//     if (maxBytes >= (1024*1024))
//         return ERROR_MORE_DATA;
//
//     *ldwTotsize = 0;
//
//     TRACE("RegQueryMultipleValuesW(%p,%p,%ld,%p,%p=%ld)\n",
//           hKey, val_list, num_vals, lpValueBuf, ldwTotsize, *ldwTotsize);
//
//     for (i = 0; i < num_vals; i++)
//     {
//         val_list[i].ve_valuelen = 0;
//         ErrorCode = RegQueryValueExW(hKey,
//                                      val_list[i].ve_valuename,
//                                      NULL,
//                                      NULL,
//                                      NULL,
//                                      &val_list[i].ve_valuelen);
//         if (ErrorCode != ERROR_SUCCESS)
//         {
//             return ErrorCode;
//          }
//
//         if (lpValueBuf != NULL && *ldwTotsize + val_list[i].ve_valuelen <= maxBytes)
//         {
//             ErrorCode = RegQueryValueExW(hKey,
//                                          val_list[i].ve_valuename,
//                                          NULL,
//                                          &val_list[i].ve_type,
//                                          (LPBYTE)bufptr,
//                                          &val_list[i].ve_valuelen);
//             if (ErrorCode != ERROR_SUCCESS)
//             {
//                 return ErrorCode;
//             }
//
//             val_list[i].ve_valueptr = (DWORD_PTR)bufptr;
//
//             bufptr += val_list[i].ve_valuelen;
//         }
//
//         *ldwTotsize += val_list[i].ve_valuelen;
//     }
//
//     return (lpValueBuf != NULL && *ldwTotsize <= maxBytes) ? ERROR_SUCCESS : ERROR_MORE_DATA;
// }
//
//
// /************************************************************************
//  *  RegQueryReflectionKey
//  *
//  * @unimplemented
//  */
// LONG WINAPI
// RegQueryReflectionKey(IN HKEY hBase,
//                       OUT BOOL* bIsReflectionDisabled)
// {
//     FIXME("RegQueryReflectionKey(0x%p, 0x%p) UNIMPLEMENTED!\n",
//           hBase, bIsReflectionDisabled);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /******************************************************************************
//  * RegQueryValueExA   [ADVAPI32.@]
//  *
//  * Get the type and contents of a specified value under with a key.
//  *
//  * PARAMS
//  *  hkey      [I]   Handle of the key to query
//  *  name      [I]   Name of value under hkey to query
//  *  reserved  [I]   Reserved - must be NULL
//  *  type      [O]   Destination for the value type, or NULL if not required
//  *  data      [O]   Destination for the values contents, or NULL if not required
//  *  count     [I/O] Size of data, updated with the number of bytes returned
//  *
//  * RETURNS
//  *  Success: ERROR_SUCCESS. *count is updated with the number of bytes copied to data.
//  *  Failure: ERROR_INVALID_HANDLE, if hkey is invalid.
//  *           ERROR_INVALID_PARAMETER, if any other parameter is invalid.
//  *           ERROR_MORE_DATA, if on input *count is too small to hold the contents.
//  *
//  * NOTES
//  *   MSDN states that if data is too small it is partially filled. In reality
//  *   it remains untouched.
//  */
// LONG
// WINAPI
// RegQueryValueExA(
//      HKEY hkeyorg,
//      LPCSTR name,
//      LPDWORD reserved,
//      LPDWORD type,
//      LPBYTE data,
//      LPDWORD count)
// {
//     UNICODE_STRING nameW;
//     DWORD DataLength;
//     DWORD ErrorCode;
//     DWORD BufferSize = 0;
//     WCHAR* Buffer;
//     CHAR* DataStr = (CHAR*)data;
//     DWORD LocalType;
//
//     /* Validate those parameters, the rest will be done with the first RegQueryValueExW call */
//     if ((data && !count) || reserved)
//         return ERROR_INVALID_PARAMETER;
//
//     if (name)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&nameW, name))
//             return ERROR_NOT_ENOUGH_MEMORY;
//     }
//     else
//         RtlInitEmptyUnicodeString(&nameW, NULL, 0);
//
//     ErrorCode = RegQueryValueExW(hkeyorg, nameW.Buffer, NULL, &LocalType, NULL, &BufferSize);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         if ((!data) && count)
//             *count = 0;
//         RtlFreeUnicodeString(&nameW);
//         return ErrorCode;
//     }
//
//     /* See if we can directly handle the call without caring for conversion */
//     if (!is_string(LocalType) || !count)
//     {
//         ErrorCode = RegQueryValueExW(hkeyorg, nameW.Buffer, reserved, type, data, count);
//         RtlFreeUnicodeString(&nameW);
//         return ErrorCode;
//     }
//
//     /* Allocate a unicode string to get the data */
//     Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferSize);
//     if (!Buffer)
//     {
//         RtlFreeUnicodeString(&nameW);
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     ErrorCode = RegQueryValueExW(hkeyorg, nameW.Buffer, reserved, type, (LPBYTE)Buffer, &BufferSize);
//     if (ErrorCode != ERROR_SUCCESS)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//         RtlFreeUnicodeString(&nameW);
//         return ErrorCode;
//     }
//
//     /* We don't need this anymore */
//     RtlFreeUnicodeString(&nameW);
//
//     /* Get the length for the multi-byte string (without the terminating NULL!) */
//     DataLength = *count;
//     RtlUnicodeToMultiByteSize(count, Buffer, BufferSize);
//
//     if ((!data) || (DataLength < *count))
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//         return  data ? ERROR_MORE_DATA : ERROR_SUCCESS;
//     }
//
//     /* We can finally do the conversion */
//     RtlUnicodeToMultiByteN(DataStr, DataLength, NULL, Buffer, BufferSize);
//
//     /* NULL-terminate if there is enough room */
//     if (DataLength > *count)
//         DataStr[*count] = '\0';
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//
//     return ERROR_SUCCESS;
// }
//
//
// /************************************************************************
//  *  RegQueryValueExW
//  *
//  * @implemented
//  */
// LONG
// WINAPI
// RegQueryValueExW(
//      HKEY hkeyorg,
//      LPCWSTR name,
//      LPDWORD reserved,
//      LPDWORD type,
//      LPBYTE data,
//      LPDWORD count)
// {
//     HANDLE hkey;
//     NTSTATUS status;
//     UNICODE_STRING name_str;
//     DWORD total_size;
//     char buffer[256], *buf_ptr = buffer;
//     KEY_VALUE_PARTIAL_INFORMATION *info = (KEY_VALUE_PARTIAL_INFORMATION *)buffer;
//     static const int info_size = offsetof( KEY_VALUE_PARTIAL_INFORMATION, Data );
//
//     TRACE("(%p,%s,%p,%p,%p,%p=%d)\n",
//           hkeyorg, debugstr_w(name), reserved, type, data, count,
//           (count && data) ? *count : 0 );
//
//     if ((data && !count) || reserved) return ERROR_INVALID_PARAMETER;
//
//     status = MapDefaultKey(&hkey, hkeyorg);
//     if (!NT_SUCCESS(status))
//     {
//         return RtlNtStatusToDosError(status);
//     }
//
//     if (IsHKCRKey(hkey))
//     {
//         LONG ErrorCode = QueryHKCRValue(hkey, name, reserved, type, data, count);
//         ClosePredefKey(hkey);
//         return ErrorCode;
//     }
//
//     RtlInitUnicodeString( &name_str, name );
//
//     if (data)
//         total_size = min( sizeof(buffer), *count + info_size );
//     else
//         total_size = info_size;
//
//
//     status = NtQueryValueKey( hkey, &name_str, KeyValuePartialInformation,
//                               buffer, total_size, &total_size );
//
//     if (!NT_SUCCESS(status) && status != STATUS_BUFFER_OVERFLOW)
//     {
//         // NT: Valid handles with inexistant/null values or invalid (but not NULL) handles sets type to REG_NONE
//         // On windows these conditions are likely to be side effects of the implementation...
//         if (status == STATUS_INVALID_HANDLE && hkey)
//         {
//             if (type) *type = REG_NONE;
//             if (count) *count = 0;
//         }
//         else if (status == STATUS_OBJECT_NAME_NOT_FOUND)
//         {
//             if (type) *type = REG_NONE;
//             if (data == NULL && count) *count = 0;
//         }
//         goto done;
//     }
//
//     if (data)
//     {
//         /* retry with a dynamically allocated buffer */
//         while (status == STATUS_BUFFER_OVERFLOW && total_size - info_size <= *count)
//         {
//             if (buf_ptr != buffer) HeapFree( GetProcessHeap(), 0, buf_ptr );
//             if (!(buf_ptr = HeapAlloc( GetProcessHeap(), 0, total_size )))
//             {
//                 ClosePredefKey(hkey);
//                 return ERROR_NOT_ENOUGH_MEMORY;
//             }
//             info = (KEY_VALUE_PARTIAL_INFORMATION *)buf_ptr;
//             status = NtQueryValueKey( hkey, &name_str, KeyValuePartialInformation,
//                                       buf_ptr, total_size, &total_size );
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             memcpy( data, buf_ptr + info_size, total_size - info_size );
//             /* if the type is REG_SZ and data is not 0-terminated
//              * and there is enough space in the buffer NT appends a \0 */
//             if (is_string(info->Type) && total_size - info_size <= *count-sizeof(WCHAR))
//             {
//                 WCHAR *ptr = (WCHAR *)(data + total_size - info_size);
//                 if (ptr > (WCHAR *)data && ptr[-1]) *ptr = 0;
//             }
//         }
//         else if (status != STATUS_BUFFER_OVERFLOW) goto done;
//     }
//     else status = STATUS_SUCCESS;
//
//     if (type) *type = info->Type;
//     if (count) *count = total_size - info_size;
//
//  done:
//     if (buf_ptr != buffer) HeapFree( GetProcessHeap(), 0, buf_ptr );
//     ClosePredefKey(hkey);
//     return RtlNtStatusToDosError(status);
// }
//
//
// /************************************************************************
//  *  RegQueryValueA
//  *
//  * @implemented
//  */
// LSTATUS WINAPI RegQueryValueA( HKEY hkey, LPCSTR name, LPSTR data, LPLONG count )
// {
//     DWORD ret;
//     HKEY subkey = hkey;
//
//     TRACE("(%p,%s,%p,%d)\n", hkey, debugstr_a(name), data, count ? *count : 0 );
//
//     if (name && name[0])
//     {
//     if ((ret = RegOpenKeyA( hkey, name, &subkey )) != ERROR_SUCCESS) return ret;
//     }
//     ret = RegQueryValueExA( subkey, NULL, NULL, NULL, (LPBYTE)data, (LPDWORD)count );
//     if (subkey != hkey) RegCloseKey( subkey );
//     if (ret == ERROR_FILE_NOT_FOUND)
//     {
//     /* return empty string if default value not found */
//     if (data) *data = 0;
//     if (count) *count = 1;
//     ret = ERROR_SUCCESS;
//     }
//     return ret;
// }
//
//
// /************************************************************************
//  *  RegQueryValueW
//  *
//  * @implemented
//  */
// LSTATUS WINAPI RegQueryValueW( HKEY hkey, LPCWSTR name, LPWSTR data, LPLONG count )
// {
//     DWORD ret;
//     HKEY subkey = hkey;
//
//     TRACE("(%p,%s,%p,%d)\n", hkey, debugstr_w(name), data, count ? *count : 0 );
//     if (hkey == NULL)
//     {
//        return ERROR_INVALID_HANDLE;
//     }
//     if (name && name[0])
//     {
//         ret = RegOpenKeyW( hkey, name, &subkey);
//         if (ret != ERROR_SUCCESS)
//         {
//             return ret;
//         }
//     }
//
//     ret = RegQueryValueExW( subkey, NULL, NULL, NULL, (LPBYTE)data, (LPDWORD)count );
//
//     if (subkey != hkey)
//     {
//         RegCloseKey( subkey );
//     }
//
//     if (ret == ERROR_FILE_NOT_FOUND)
//     {
//         /* return empty string if default value not found */
//         if (data)
//             *data = 0;
//         if (count)
//             *count = sizeof(WCHAR);
//         ret = ERROR_SUCCESS;
//     }
//     return ret;
// }
//
//
// /************************************************************************
//  *  RegReplaceKeyA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegReplaceKeyA(HKEY hKey,
//                LPCSTR lpSubKey,
//                LPCSTR lpNewFile,
//                LPCSTR lpOldFile)
// {
//     UNICODE_STRING SubKey;
//     UNICODE_STRING NewFile;
//     UNICODE_STRING OldFile;
//     LONG ErrorCode;
//
//     RtlInitEmptyUnicodeString(&SubKey, NULL, 0);
//     RtlInitEmptyUnicodeString(&OldFile, NULL, 0);
//     RtlInitEmptyUnicodeString(&NewFile, NULL, 0);
//
//     if (lpSubKey)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&SubKey, lpSubKey))
//         {
//             ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//     }
//
//     if (lpOldFile)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&OldFile, lpOldFile))
//         {
//             ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//     }
//
//     if (lpNewFile)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&NewFile, lpNewFile))
//         {
//             ErrorCode = ERROR_NOT_ENOUGH_MEMORY;
//             goto Exit;
//         }
//     }
//
//     ErrorCode = RegReplaceKeyW(hKey,
//                                SubKey.Buffer,
//                                NewFile.Buffer,
//                                OldFile.Buffer);
//
// Exit:
//     RtlFreeUnicodeString(&OldFile);
//     RtlFreeUnicodeString(&NewFile);
//     RtlFreeUnicodeString(&SubKey);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegReplaceKeyW
//  *
//  * @unimplemented
//  */
// LONG WINAPI
// RegReplaceKeyW(HKEY hKey,
//                LPCWSTR lpSubKey,
//                LPCWSTR lpNewFile,
//                LPCWSTR lpOldFile)
// {
//     OBJECT_ATTRIBUTES KeyObjectAttributes;
//     OBJECT_ATTRIBUTES NewObjectAttributes;
//     OBJECT_ATTRIBUTES OldObjectAttributes;
//     UNICODE_STRING SubKeyName;
//     UNICODE_STRING NewFileName;
//     UNICODE_STRING OldFileName;
//     BOOLEAN CloseRealKey;
//     HANDLE RealKeyHandle;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//     LONG ErrorCode = ERROR_SUCCESS;
//
//     if (hKey == HKEY_PERFORMANCE_DATA)
//     {
//         return ERROR_INVALID_HANDLE;
//     }
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     /* Open the real key */
//     if (lpSubKey != NULL && *lpSubKey != (WCHAR)0)
//     {
//         RtlInitUnicodeString(&SubKeyName, lpSubKey);
//         InitializeObjectAttributes(&KeyObjectAttributes,
//                                    &SubKeyName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    KeyHandle,
//                                    NULL);
//         Status = NtOpenKey(&RealKeyHandle,
//                            MAXIMUM_ALLOWED,
//                            &KeyObjectAttributes);
//         if (!NT_SUCCESS(Status))
//         {
//             ErrorCode = RtlNtStatusToDosError(Status);
//             goto Cleanup;
//         }
//
//         CloseRealKey = TRUE;
//     }
//     else
//     {
//         RealKeyHandle = KeyHandle;
//         CloseRealKey = FALSE;
//     }
//
//     /* Convert new file name */
//     if (!RtlDosPathNameToNtPathName_U(lpNewFile,
//                                       &NewFileName,
//                                       NULL,
//                                       NULL))
//     {
//         if (CloseRealKey)
//         {
//             NtClose(RealKeyHandle);
//         }
//
//         ErrorCode = ERROR_INVALID_PARAMETER;
//         goto Cleanup;
//     }
//
//     InitializeObjectAttributes(&NewObjectAttributes,
//                                &NewFileName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     /* Convert old file name */
//     if (!RtlDosPathNameToNtPathName_U(lpOldFile,
//                                       &OldFileName,
//                                       NULL,
//                                       NULL))
//     {
//         RtlFreeHeap(RtlGetProcessHeap (),
//                     0,
//                     NewFileName.Buffer);
//         if (CloseRealKey)
//         {
//             NtClose(RealKeyHandle);
//         }
//
//         ErrorCode = ERROR_INVALID_PARAMETER;
//         goto Cleanup;
//     }
//
//     InitializeObjectAttributes(&OldObjectAttributes,
//                                &OldFileName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = NtReplaceKey(&NewObjectAttributes,
//                           RealKeyHandle,
//                           &OldObjectAttributes);
//
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 OldFileName.Buffer);
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 NewFileName.Buffer);
//
//     if (CloseRealKey)
//     {
//         NtClose(RealKeyHandle);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
// Cleanup:
//     ClosePredefKey(KeyHandle);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegRestoreKeyA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegRestoreKeyA(HKEY hKey,
//                LPCSTR lpFile,
//                DWORD dwFlags)
// {
//     UNICODE_STRING FileName;
//     LONG ErrorCode;
//
//     if (lpFile)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&FileName, lpFile))
//             return ERROR_NOT_ENOUGH_MEMORY;
//     }
//     else
//         RtlInitEmptyUnicodeString(&FileName, NULL, 0);
//
//     ErrorCode = RegRestoreKeyW(hKey,
//                                FileName.Buffer,
//                                dwFlags);
//
//     RtlFreeUnicodeString(&FileName);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegRestoreKeyW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegRestoreKeyW(HKEY hKey,
//                LPCWSTR lpFile,
//                DWORD dwFlags)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     UNICODE_STRING FileName;
//     HANDLE FileHandle;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//
//     if (hKey == HKEY_PERFORMANCE_DATA)
//     {
//         return ERROR_INVALID_HANDLE;
//     }
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     if (!RtlDosPathNameToNtPathName_U(lpFile,
//                                       &FileName,
//                                       NULL,
//                                       NULL))
//     {
//         Status = STATUS_INVALID_PARAMETER;
//         goto Cleanup;
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &FileName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenFile(&FileHandle,
//                         FILE_GENERIC_READ,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ,
//                         FILE_SYNCHRONOUS_IO_NONALERT);
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 FileName.Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     Status = NtRestoreKey(KeyHandle,
//                           FileHandle,
//                           (ULONG)dwFlags);
//     NtClose (FileHandle);
//
// Cleanup:
//     ClosePredefKey(KeyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /************************************************************************
//  *  RegSaveKeyA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegSaveKeyA(HKEY hKey,
//             LPCSTR lpFile,
//             LPSECURITY_ATTRIBUTES lpSecurityAttributes)
// {
//     UNICODE_STRING FileName;
//     LONG ErrorCode;
//
//     if (lpFile)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&FileName, lpFile))
//             return ERROR_NOT_ENOUGH_MEMORY;
//     }
//     else
//         RtlInitEmptyUnicodeString(&FileName, NULL, 0);
//
//     ErrorCode = RegSaveKeyW(hKey,
//                             FileName.Buffer,
//                             lpSecurityAttributes);
//     RtlFreeUnicodeString(&FileName);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegSaveKeyW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegSaveKeyW(HKEY hKey,
//             LPCWSTR lpFile,
//             LPSECURITY_ATTRIBUTES lpSecurityAttributes)
// {
//     PSECURITY_DESCRIPTOR SecurityDescriptor = NULL;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING FileName;
//     IO_STATUS_BLOCK IoStatusBlock;
//     HANDLE FileHandle;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     if (!RtlDosPathNameToNtPathName_U(lpFile,
//                                       &FileName,
//                                       NULL,
//                                       NULL))
//     {
//         Status = STATUS_INVALID_PARAMETER;
//         goto Cleanup;
//     }
//
//     if (lpSecurityAttributes != NULL)
//     {
//         SecurityDescriptor = lpSecurityAttributes->lpSecurityDescriptor;
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &FileName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                SecurityDescriptor);
//     Status = NtCreateFile(&FileHandle,
//                           GENERIC_WRITE | SYNCHRONIZE,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           FILE_ATTRIBUTE_NORMAL,
//                           FILE_SHARE_READ,
//                           FILE_CREATE,
//                           FILE_OPEN_FOR_BACKUP_INTENT | FILE_SYNCHRONOUS_IO_NONALERT,
//                           NULL,
//                           0);
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 FileName.Buffer);
//     if (!NT_SUCCESS(Status))
//     {
//         goto Cleanup;
//     }
//
//     Status = NtSaveKey(KeyHandle,
//                        FileHandle);
//     NtClose (FileHandle);
//
// Cleanup:
//     ClosePredefKey(KeyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /************************************************************************
//  *  RegSaveKeyExA
//  *
//  * @implemented
//  */
// LONG
// WINAPI
// RegSaveKeyExA(HKEY hKey,
//               LPCSTR lpFile,
//               LPSECURITY_ATTRIBUTES lpSecurityAttributes,
//               DWORD Flags)
// {
//     UNICODE_STRING FileName;
//     LONG ErrorCode;
//
//     if (lpFile)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&FileName, lpFile))
//             return ERROR_NOT_ENOUGH_MEMORY;
//     }
//     else
//         RtlInitEmptyUnicodeString(&FileName, NULL, 0);
//
//     ErrorCode = RegSaveKeyExW(hKey,
//                               FileName.Buffer,
//                               lpSecurityAttributes,
//                               Flags);
//     RtlFreeUnicodeString(&FileName);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegSaveKeyExW
//  *
//  * @unimplemented
//  */
// LONG
// WINAPI
// RegSaveKeyExW(HKEY hKey,
//               LPCWSTR lpFile,
//               LPSECURITY_ATTRIBUTES lpSecurityAttributes,
//               DWORD Flags)
// {
//     switch (Flags)
//     {
//         case REG_STANDARD_FORMAT:
//         case REG_LATEST_FORMAT:
//         case REG_NO_COMPRESSION:
//             break;
//         default:
//             return ERROR_INVALID_PARAMETER;
//     }
//
//     FIXME("RegSaveKeyExW(): Flags ignored!\n");
//
//     return RegSaveKeyW(hKey,
//                        lpFile,
//                        lpSecurityAttributes);
// }
//
//
// /************************************************************************
//  *  RegSetKeySecurity
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegSetKeySecurity(HKEY hKey,
//                   SECURITY_INFORMATION SecurityInformation,
//                   PSECURITY_DESCRIPTOR pSecurityDescriptor)
// {
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//
//     if (hKey == HKEY_PERFORMANCE_DATA)
//     {
//         return ERROR_INVALID_HANDLE;
//     }
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     Status = NtSetSecurityObject(KeyHandle,
//                                  SecurityInformation,
//                                  pSecurityDescriptor);
//
//     ClosePredefKey(KeyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /************************************************************************
//  *  RegSetValueExA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegSetValueExA(HKEY hKey,
//                LPCSTR lpValueName,
//                DWORD Reserved,
//                DWORD dwType,
//                CONST BYTE* lpData,
//                DWORD cbData)
// {
//     UNICODE_STRING ValueName;
//     LPWSTR pValueName;
//     ANSI_STRING AnsiString;
//     UNICODE_STRING Data;
//     LONG ErrorCode;
//     LPBYTE pData;
//     DWORD DataSize;
//     NTSTATUS Status;
//
//     /* Convert SubKey name to Unicode */
//     if (lpValueName != NULL && lpValueName[0] != '\0')
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&ValueName, lpValueName))
//             return ERROR_NOT_ENOUGH_MEMORY;
//     }
//     else
//     {
//         ValueName.Buffer = NULL;
//     }
//
//     pValueName = (LPWSTR)ValueName.Buffer;
//
//
//     if (is_string(dwType) && (cbData != 0))
//     {
//         /* Convert ANSI string Data to Unicode */
//         /* If last character NOT zero then increment length */
//         LONG bNoNulledStr = ((lpData[cbData-1] != '\0') ? 1 : 0);
//         AnsiString.Buffer = (PSTR)lpData;
//         AnsiString.Length = cbData + bNoNulledStr;
//         AnsiString.MaximumLength = cbData + bNoNulledStr;
//         Status = RtlAnsiStringToUnicodeString(&Data,
//                                      &AnsiString,
//                                      TRUE);
//
//         if (!NT_SUCCESS(Status))
//         {
//             if (pValueName != NULL)
//                 RtlFreeUnicodeString(&ValueName);
//
//             return RtlNtStatusToDosError(Status);
//         }
//         pData = (LPBYTE)Data.Buffer;
//         DataSize = cbData * sizeof(WCHAR);
//     }
//     else
//     {
//         Data.Buffer = NULL;
//         pData = (LPBYTE)lpData;
//         DataSize = cbData;
//     }
//
//     ErrorCode = RegSetValueExW(hKey,
//                                pValueName,
//                                Reserved,
//                                dwType,
//                                pData,
//                                DataSize);
//
//     if (pValueName != NULL)
//         RtlFreeUnicodeString(&ValueName);
//
//     if (Data.Buffer != NULL)
//         RtlFreeUnicodeString(&Data);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegSetValueExW
//  *
//  * @implemented
//  */
// LONG
// WINAPI
// RegSetValueExW(
//      HKEY hKey,
//      LPCWSTR lpValueName,
//      DWORD Reserved,
//      DWORD dwType,
//      CONST BYTE* lpData,
//      DWORD cbData)
// {
//     UNICODE_STRING ValueName;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//
//     Status = MapDefaultKey(&KeyHandle,
//                            hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     if (IsHKCRKey(KeyHandle))
//     {
//         LONG ErrorCode = SetHKCRValue(KeyHandle, lpValueName, Reserved, dwType, lpData, cbData);
//         ClosePredefKey(KeyHandle);
//         return ErrorCode;
//     }
//
//     if (is_string(dwType) && (cbData != 0))
//     {
//         PWSTR pwsData = (PWSTR)lpData;
//
//         _SEH2_TRY
//         {
//             if((pwsData[cbData / sizeof(WCHAR) - 1] != L'\0') &&
//                 (pwsData[cbData / sizeof(WCHAR)] == L'\0'))
//             {
//                 /* Increment length if last character is not zero and next is zero */
//                 cbData += sizeof(WCHAR);
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Do not fail if we fault where we were told not to go */
//         }
//         _SEH2_END;
//     }
//
//     RtlInitUnicodeString(&ValueName, lpValueName);
//
//     Status = NtSetValueKey(KeyHandle,
//                            &ValueName,
//                            0,
//                            dwType,
//                            (PVOID)lpData,
//                            (ULONG)cbData);
//
//     ClosePredefKey(KeyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /************************************************************************
//  *  RegSetValueA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegSetValueA(HKEY hKeyOriginal,
//              LPCSTR lpSubKey,
//              DWORD dwType,
//              LPCSTR lpData,
//              DWORD cbData)
// {
//     HKEY subkey;
//     HANDLE hKey;
//     DWORD ret;
//     NTSTATUS Status;
//
//     TRACE("(%p,%s,%d,%s,%d)\n", hKeyOriginal, debugstr_a(lpSubKey), dwType, debugstr_a(lpData), cbData );
//
//     if (dwType != REG_SZ || !lpData) return ERROR_INVALID_PARAMETER;
//
//     Status = MapDefaultKey(&hKey, hKeyOriginal);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError (Status);
//     }
//     subkey = hKey;
//
//     if (lpSubKey && lpSubKey[0])  /* need to create the subkey */
//     {
//         ret = RegCreateKeyA(hKey, lpSubKey, &subkey);
//         if (ret != ERROR_SUCCESS)
//             goto Cleanup;
//     }
//
//     ret = RegSetValueExA( subkey, NULL, 0, REG_SZ, (const BYTE*)lpData, strlen(lpData)+1 );
//     if (subkey != hKey)
//         RegCloseKey(subkey);
//
// Cleanup:
//     ClosePredefKey(hKey);
//
//     return ret;
// }
//
//
// /************************************************************************
//  *  RegSetValueW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegSetValueW(HKEY hKeyOriginal,
//              LPCWSTR lpSubKey,
//              DWORD dwType,
//              LPCWSTR lpData,
//              DWORD cbData)
// {
//     HKEY subkey;
//     HANDLE hKey;
//     DWORD ret;
//     NTSTATUS Status;
//
//     TRACE("(%p,%s,%d,%s,%d)\n", hKeyOriginal, debugstr_w(lpSubKey), dwType, debugstr_w(lpData), cbData );
//
//     if (dwType != REG_SZ || !lpData)
//         return ERROR_INVALID_PARAMETER;
//
//     Status = MapDefaultKey(&hKey,
//                            hKeyOriginal);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//     subkey = hKey;
//
//     if (lpSubKey && lpSubKey[0])  /* need to create the subkey */
//     {
//         ret = RegCreateKeyW(hKey, lpSubKey, &subkey);
//         if (ret != ERROR_SUCCESS)
//             goto Cleanup;
//     }
//
//     ret = RegSetValueExW( subkey, NULL, 0, REG_SZ, (const BYTE*)lpData,
//                           (wcslen( lpData ) + 1) * sizeof(WCHAR) );
//     if (subkey != hKey)
//         RegCloseKey(subkey);
//
// Cleanup:
//     ClosePredefKey(hKey);
//
//     return ret;
// }
//
//
// /************************************************************************
//  *  RegUnLoadKeyA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegUnLoadKeyA(HKEY hKey,
//               LPCSTR lpSubKey)
// {
//     UNICODE_STRING KeyName;
//     DWORD ErrorCode;
//
//     if (lpSubKey)
//     {
//         if (!RtlCreateUnicodeStringFromAsciiz(&KeyName, lpSubKey))
//             return ERROR_NOT_ENOUGH_MEMORY;
//     }
//     else
//         RtlInitEmptyUnicodeString(&KeyName, NULL, 0);
//
//     ErrorCode = RegUnLoadKeyW(hKey,
//                               KeyName.Buffer);
//
//     RtlFreeUnicodeString (&KeyName);
//
//     return ErrorCode;
// }
//
//
// /************************************************************************
//  *  RegUnLoadKeyW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegUnLoadKeyW(HKEY hKey,
//               LPCWSTR lpSubKey)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING KeyName;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//
//     if (hKey == HKEY_PERFORMANCE_DATA)
//     {
//       return ERROR_INVALID_HANDLE;
//     }
//
//     Status = MapDefaultKey(&KeyHandle, hKey);
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     RtlInitUnicodeString(&KeyName, lpSubKey);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                KeyHandle,
//                                NULL);
//
//     Status = NtUnloadKey(&ObjectAttributes);
//
//     ClosePredefKey(KeyHandle);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return RtlNtStatusToDosError(Status);
//     }
//
//     return ERROR_SUCCESS;
// }
//
// #endif // _ADVAPI32_VISTA_
//
// /******************************************************************************
//  * load_string [Internal]
//  *
//  * This is basically a copy of user32/resource.c's LoadStringW. Necessary to
//  * avoid importing user32, which is higher level than advapi32. Helper for
//  * RegLoadMUIString.
//  */
// static int load_string(HINSTANCE hModule, UINT resId, LPWSTR pwszBuffer, INT cMaxChars)
// {
//     HGLOBAL hMemory;
//     HRSRC hResource;
//     WCHAR* pString;
//     int idxString;
//
//     /* Negative values have to be inverted. */
//     if (HIWORD(resId) == 0xffff)
//         resId = (UINT)(-((INT)resId));
//
//     /* Load the resource into memory and get a pointer to it. */
//     hResource = FindResourceW(hModule, MAKEINTRESOURCEW(LOWORD(resId >> 4) + 1), (LPWSTR)RT_STRING);
//     if (!hResource) return 0;
//     hMemory = LoadResource(hModule, hResource);
//     if (!hMemory) return 0;
//     pString = LockResource(hMemory);
//
//     /* Strings are length-prefixed. Lowest nibble of resId is an index. */
//     idxString = resId & 0xf;
//     while (idxString--) pString += *pString + 1;
//
//     /* If no buffer is given, return length of the string. */
//     if (!pwszBuffer) return *pString;
//
//     /* Else copy over the string, respecting the buffer size. */
//     cMaxChars = (*pString < cMaxChars) ? *pString : (cMaxChars - 1);
//     if (cMaxChars >= 0)
//     {
//         memcpy(pwszBuffer, pString + 1, cMaxChars * sizeof(WCHAR));
//         pwszBuffer[cMaxChars] = L'\0';
//     }
//
//     return cMaxChars;
// }
//
// /************************************************************************
//  *  RegLoadMUIStringW
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegLoadMUIStringW(
//     IN HKEY hKey,
//     IN LPCWSTR pszValue  OPTIONAL,
//     OUT LPWSTR pszOutBuf,
//     IN DWORD cbOutBuf,
//     OUT LPDWORD pcbData OPTIONAL,
//     IN DWORD Flags,
//     IN LPCWSTR pszDirectory  OPTIONAL)
// {
//     DWORD dwValueType, cbData;
//     LPWSTR pwszTempBuffer = NULL, pwszExpandedBuffer = NULL;
//     LONG result;
//
//     /* Parameter sanity checks. */
//     if (!hKey || !pszOutBuf)
//         return ERROR_INVALID_PARAMETER;
//
//     if (pszDirectory && *pszDirectory)
//     {
//         FIXME("BaseDir parameter not yet supported!\n");
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Check for value existence and correctness of it's type, allocate a buffer and load it. */
//     result = RegQueryValueExW(hKey, pszValue, NULL, &dwValueType, NULL, &cbData);
//     if (result != ERROR_SUCCESS) goto cleanup;
//     if (!(dwValueType == REG_SZ || dwValueType == REG_EXPAND_SZ) || !cbData)
//     {
//         result = ERROR_FILE_NOT_FOUND;
//         goto cleanup;
//     }
//     pwszTempBuffer = HeapAlloc(GetProcessHeap(), 0, cbData);
//     if (!pwszTempBuffer)
//     {
//         result = ERROR_NOT_ENOUGH_MEMORY;
//         goto cleanup;
//     }
//     result = RegQueryValueExW(hKey, pszValue, NULL, &dwValueType, (LPBYTE)pwszTempBuffer, &cbData);
//     if (result != ERROR_SUCCESS) goto cleanup;
//
//     /* Expand environment variables, if appropriate, or copy the original string over. */
//     if (dwValueType == REG_EXPAND_SZ)
//     {
//         cbData = ExpandEnvironmentStringsW(pwszTempBuffer, NULL, 0) * sizeof(WCHAR);
//         if (!cbData) goto cleanup;
//         pwszExpandedBuffer = HeapAlloc(GetProcessHeap(), 0, cbData);
//         if (!pwszExpandedBuffer)
//         {
//             result = ERROR_NOT_ENOUGH_MEMORY;
//             goto cleanup;
//         }
//         ExpandEnvironmentStringsW(pwszTempBuffer, pwszExpandedBuffer, cbData);
//     }
//     else
//     {
//         pwszExpandedBuffer = HeapAlloc(GetProcessHeap(), 0, cbData);
//         memcpy(pwszExpandedBuffer, pwszTempBuffer, cbData);
//     }
//
//     /* If the value references a resource based string, parse the value and load the string.
//      * Else just copy over the original value. */
//     result = ERROR_SUCCESS;
//     if (*pwszExpandedBuffer != L'@') /* '@' is the prefix for resource based string entries. */
//     {
//         lstrcpynW(pszOutBuf, pwszExpandedBuffer, cbOutBuf / sizeof(WCHAR));
//     }
//     else
//     {
//         WCHAR* pComma = wcsrchr(pwszExpandedBuffer, L',');
//         UINT uiStringId;
//         HMODULE hModule;
//
//         /* Format of the expanded value is 'path_to_dll,-resId' */
//         if (!pComma || pComma[1] != L'-')
//         {
//             result = ERROR_BADKEY;
//             goto cleanup;
//         }
//
//         uiStringId = _wtoi(pComma + 2);
//         *pComma = L'\0';
//
//         hModule = LoadLibraryExW(pwszExpandedBuffer + 1, NULL, LOAD_LIBRARY_AS_DATAFILE);
//         if (!hModule || !load_string(hModule, uiStringId, pszOutBuf, cbOutBuf / sizeof(WCHAR)))
//             result = ERROR_BADKEY;
//         FreeLibrary(hModule);
//     }
//
// cleanup:
//     HeapFree(GetProcessHeap(), 0, pwszTempBuffer);
//     HeapFree(GetProcessHeap(), 0, pwszExpandedBuffer);
//     return result;
// }
//
// /************************************************************************
//  *  RegLoadMUIStringA
//  *
//  * @implemented
//  */
// LONG WINAPI
// RegLoadMUIStringA(
//     IN HKEY hKey,
//     IN LPCSTR pszValue  OPTIONAL,
//     OUT LPSTR pszOutBuf,
//     IN DWORD cbOutBuf,
//     OUT LPDWORD pcbData OPTIONAL,
//     IN DWORD Flags,
//     IN LPCSTR pszDirectory  OPTIONAL)
// {
//     UNICODE_STRING valueW, baseDirW;
//     WCHAR* pwszBuffer;
//     DWORD cbData = cbOutBuf * sizeof(WCHAR);
//     LONG result;
//
//     valueW.Buffer = baseDirW.Buffer = pwszBuffer = NULL;
//     if (!RtlCreateUnicodeStringFromAsciiz(&valueW, pszValue) ||
//         !RtlCreateUnicodeStringFromAsciiz(&baseDirW, pszDirectory) ||
//         !(pwszBuffer = HeapAlloc(GetProcessHeap(), 0, cbData)))
//     {
//         result = ERROR_NOT_ENOUGH_MEMORY;
//         goto cleanup;
//     }
//
//     result = RegLoadMUIStringW(hKey, valueW.Buffer, pwszBuffer, cbData, NULL, Flags,
//         baseDirW.Buffer);
//
//     if (result == ERROR_SUCCESS)
//     {
//         cbData = WideCharToMultiByte(CP_ACP, 0, pwszBuffer, -1, pszOutBuf, cbOutBuf, NULL, NULL);
//         if (pcbData)
//             *pcbData = cbData;
//     }
//
// cleanup:
//     HeapFree(GetProcessHeap(), 0, pwszBuffer);
//     RtlFreeUnicodeString(&baseDirW);
//     RtlFreeUnicodeString(&valueW);
//
//     return result;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
