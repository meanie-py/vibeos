use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag, RtlAppendUnicodeStringToString, RtlAppendUnicodeToString, RtlFormatCurrentUserKeyPath, RtlFreeUnicodeString, RtlInitUnicodeString, ZwClose, ZwCreateKey, ZwDeleteValueKey, ZwEnumerateValueKey, ZwOpenKey, ZwQueryValueKey, ZwSetValueKey};
use ntkrnl_sys::rtl::RtlCopyMemory;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NO_MEMORY};
use ntkrnl_sys::constants::{STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL, STATUS_OBJECT_TYPE_MISMATCH};

// ---- file-scope preamble (translated) ----
// tags.h: TAG_TEMP 'K0hG' (C multichar big-endian -> reversed bytes).
const TAG_TEMP: u32 = u32::from_le_bytes(*b"Gh0K");

// RtlInitEmptyUnicodeString is a macro (no kernel symbol): set Buffer/Length/Max.
#[inline(always)]
unsafe fn rtl_init_empty_unicode_string(s: *mut ntkrnl_sys::ddk::_UNICODE_STRING, buf: *mut u16, sz: u16) {
    (*s).Buffer = buf;
    (*s).Length = 0;
    (*s).MaximumLength = sz;
}

#[no_mangle]
pub unsafe extern "system" fn RegOpenKey(
    pwszKeyName: LPCWSTR,
    phkey: PHKEY,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ustrKeyName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut hkey: HKEY = core::ptr::null_mut();

    RtlInitUnicodeString(&mut ustrKeyName, pwszKeyName);

    ObjectAttributes.Length = core::mem::size_of::<ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
    ObjectAttributes.ObjectName = &mut ustrKeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwOpenKey(
        &mut hkey as *mut HKEY as PHANDLE,
        KEY_READ,
        &mut ObjectAttributes,
    );
    if NT_SUCCESS(Status) {
        *phkey = hkey;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RegQueryValue(
    hkey: HKEY,
    pwszValueName: PCWSTR,
    ulType: u32,
    pvData: *mut core::ffi::c_void,
    pcbValue: *mut u32,
) -> i32 {
    let mut Status: i32;
    let mut ustrValueName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut ajBuffer: [u8; 100] = [0; 100];
    let pInfo: PKEY_VALUE_PARTIAL_INFORMATION;
    let mut cbInfoSize: u32;
    let cbDataSize: u32;

    let dataOffset =
        core::mem::offset_of!(KEY_VALUE_PARTIAL_INFORMATION, Data) as u32;

    cbInfoSize = dataOffset + *pcbValue;
    if cbInfoSize as usize <= core::mem::size_of::<[u8; 100]>() {
        pInfo = ajBuffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;
    } else {
        pInfo = ExAllocatePoolWithTag(PagedPool, cbInfoSize as SIZE_T, TAG_TEMP)
            as PKEY_VALUE_PARTIAL_INFORMATION;
        if pInfo.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    RtlInitUnicodeString(&mut ustrValueName, pwszValueName);
    Status = ZwQueryValueKey(
        hkey as _,
        &mut ustrValueName,
        KeyValuePartialInformation,
        pInfo as *mut core::ffi::c_void,
        cbInfoSize,
        &mut cbInfoSize,
    );

    if NT_SUCCESS(Status) {
        cbDataSize = (*pInfo).DataLength;

        if (*pInfo).Type != ulType {
            Status = STATUS_OBJECT_TYPE_MISMATCH;
        } else if cbDataSize > *pcbValue {
            Status = STATUS_BUFFER_TOO_SMALL;
        } else {
            RtlCopyMemory(
                pvData,
                core::ptr::addr_of!((*pInfo).Data) as *const core::ffi::c_void,
                cbDataSize as usize,
            );
        }
    } else if Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL {
        cbDataSize = cbInfoSize - dataOffset;
    } else {
        cbDataSize = 0;
    }

    *pcbValue = cbDataSize; // nolint: dropped-null-guard (retired registry.c derefs it unconditionally; the pairing is freeldr's)

    if pInfo != ajBuffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION {
        ExFreePoolWithTag(pInfo as *mut core::ffi::c_void, TAG_TEMP);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RegWriteSZ(hkey: HKEY, pwszValue: PCWSTR, pwszData: PWSTR) {
    let mut ustrValue: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut ustrData: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut ustrValue, pwszValue);
    RtlInitUnicodeString(&mut ustrData, pwszData);
    ZwSetValueKey(
        hkey as _,
        &mut ustrValue,
        0,
        REG_SZ,
        ustrData.Buffer as *mut core::ffi::c_void,
        (ustrData.Length as u32) + core::mem::size_of::<u16>() as u32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn RegWriteDWORD(hkey: HKEY, pwszValue: PCWSTR, mut dwData: u32) {
    let mut ustrValue: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut ustrValue, pwszValue);
    ZwSetValueKey(
        hkey as _,
        &mut ustrValue,
        0,
        REG_DWORD,
        core::ptr::addr_of_mut!(dwData) as *mut core::ffi::c_void,
        core::mem::size_of::<u32>() as u32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn RegReadDWORD(
    hkey: HKEY,
    pwszValue: PCWSTR,
    pdwData: PDWORD,
) -> i32 {
    let mut cbSize: u32 = core::mem::size_of::<u32>() as u32;
    let Status = RegQueryValue(
        hkey as _,
        pwszValue,
        REG_DWORD,
        pdwData as *mut core::ffi::c_void,
        &mut cbSize,
    );
    NT_SUCCESS(Status) as i32
}

// RtlStringCchCatW is a strsafe.h inline (no kernel symbol) — bounded wide cat.
unsafe fn rtl_string_cch_cat_w(dst: *mut u16, cch: usize, src: PCWSTR) {
    let mut d = 0usize;
    while d < cch && *dst.add(d) != 0 { d += 1; }
    let mut si = 0usize;
    while d + 1 < cch && *src.add(si) != 0 {
        *dst.add(d) = *src.add(si);
        d += 1; si += 1;
    }
    if d < cch { *dst.add(d) = 0; }
}

#[no_mangle]
pub unsafe extern "system" fn RegOpenSectionKey(
    pszSection: LPCWSTR,
    phkey: PHKEY,
) -> NTSTATUS {
    let mut szKey: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let init: &[u16] = &[
        0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079,
        0x005C, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065,
        0x005C, 0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065,
        0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074,
        0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020,
        0x004E, 0x0054,
        0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E,
        0x005C,
    ];
    core::ptr::copy_nonoverlapping(init.as_ptr(), szKey.as_mut_ptr(), init.len());

    rtl_string_cch_cat_w(szKey.as_mut_ptr(), szKey.len(), pszSection);
    RegOpenKey(szKey.as_ptr(), phkey)
}

#[no_mangle]
pub unsafe extern "system" fn RegGetSectionDWORD(
    pszSection: LPCWSTR,
    pszValue: PCWSTR,
    dwDefault: DWORD,
) -> DWORD {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwValue: DWORD = 0;

    let Status = RegOpenSectionKey(pszSection, &mut hKey);
    if (Status as i32) < 0 && (Status as u32 >> 30) == 3 {
        return dwDefault;
    }

    if RegReadDWORD(hKey, pszValue, &mut dwValue) == 0 {
        dwValue = dwDefault;
    }

    ZwClose(hKey as _);
    dwValue
}

#[no_mangle]
pub unsafe extern "system" fn RegReadUserSetting(
    pwszKeyName: PCWSTR,
    pwszValueName: PCWSTR,
    ulType: ULONG,
    pvData: *mut core::ffi::c_void,
    cbDataSize: ULONG,
) -> u32 {
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut usCurrentUserKey: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut usKeyName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut usValueName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut awcBuffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut hkey: HKEY = core::ptr::null_mut();
    let pInfo: PKEY_VALUE_PARTIAL_INFORMATION;
    let cbInfoSize: ULONG;
    let mut cbReqSize: ULONG = 0;

    /* Get the path of the current user's profile */
    Status = RtlFormatCurrentUserKeyPath(&mut usCurrentUserKey);
    if !NT_SUCCESS(Status) {
        return 0;
    }

    /* Initialize empty key name */
    rtl_init_empty_unicode_string(
        &mut usKeyName,
        awcBuffer.as_mut_ptr(),
        core::mem::size_of_val(&awcBuffer) as u16,
    );

    /* Append the current user key name */
    Status = RtlAppendUnicodeStringToString(&mut usKeyName, &mut usCurrentUserKey);

    /* Free the current user key name */
    RtlFreeUnicodeString(&mut usCurrentUserKey);

    /* Check for success */
    if !NT_SUCCESS(Status) {
        return 0;
    }

    /* Append a '\', we can trust in enough space left. */
    *usKeyName
        .Buffer
        .add((usKeyName.Length as usize) / core::mem::size_of::<u16>()) = b'\\' as u16;
    usKeyName.Length += core::mem::size_of::<u16>() as u16;

    /* Append the subkey name */
    Status = RtlAppendUnicodeToString(&mut usKeyName, pwszKeyName);
    if !NT_SUCCESS(Status) {
        return 0;
    }

    /* Initialize object attributes */
    ObjectAttributes.Length = core::mem::size_of::<ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
    ObjectAttributes.ObjectName = &mut usKeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    /* Open the key */
    Status = ZwOpenKey(
        &mut hkey as *mut HKEY as PHANDLE,
        KEY_READ,
        &mut ObjectAttributes,
    );
    if !NT_SUCCESS(Status) {
        return 0;
    }

    /* Check if the local buffer is sufficient */
    cbInfoSize =
        core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as u32 + cbDataSize;
    if cbInfoSize as usize <= core::mem::size_of_val(&awcBuffer) {
        pInfo = awcBuffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;
    } else {
        /* It's not, allocate a sufficient buffer */
        pInfo = ExAllocatePoolWithTag(PagedPool, cbInfoSize as SIZE_T, TAG_TEMP)
            as PKEY_VALUE_PARTIAL_INFORMATION;
        if pInfo.is_null() {
            ZwClose(hkey as _);
            return 0;
        }
    }

    /* Query the value */
    RtlInitUnicodeString(&mut usValueName, pwszValueName);
    Status = ZwQueryValueKey(
        hkey as _,
        &mut usValueName,
        KeyValuePartialInformation,
        pInfo as *mut core::ffi::c_void,
        cbInfoSize,
        &mut cbReqSize,
    );
    if NT_SUCCESS(Status) {
        /* Did we get the right type */
        if (*pInfo).Type == ulType {
            /* Copy the contents to the caller */
            RtlCopyMemory(
                pvData,
                core::ptr::addr_of!((*pInfo).Data) as *const core::ffi::c_void,
                cbDataSize as usize,
            );
        }
    }

    /* Cleanup */
    ZwClose(hkey as _);
    if pInfo != awcBuffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION {
        ExFreePoolWithTag(pInfo as *mut core::ffi::c_void, TAG_TEMP);
    }

    NT_SUCCESS(Status) as u32
}

#[no_mangle]
pub unsafe extern "system" fn RegWriteUserSetting(
    pwszKeyName: PCWSTR,
    pwszValueName: PCWSTR,
    ulType: u32,
    pvData: *const core::ffi::c_void,
    cbDataSize: u32,
) -> i32 {
    let mut Status;
    let mut ObjectAttributes: ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut usCurrentUserKey: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut usKeyName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut usValueName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    let mut awcBuffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut hkey: HKEY = core::ptr::null_mut();

    Status = RtlFormatCurrentUserKeyPath(&mut usCurrentUserKey);
    if !NT_SUCCESS(Status) {
        return 0;
    }

    rtl_init_empty_unicode_string(
        &mut usKeyName,
        awcBuffer.as_mut_ptr(),
        core::mem::size_of_val(&awcBuffer) as u16,
    );

    Status = RtlAppendUnicodeStringToString(&mut usKeyName, &mut usCurrentUserKey);
    if !NT_SUCCESS(Status) {
        return 0;
    }

    RtlFreeUnicodeString(&mut usCurrentUserKey);

    *usKeyName.Buffer.offset((usKeyName.Length as usize / core::mem::size_of::<u16>()) as isize) =
        b'\\' as u16;
    usKeyName.Length += core::mem::size_of::<u16>() as u16;

    Status = RtlAppendUnicodeToString(&mut usKeyName, pwszKeyName);
    if !NT_SUCCESS(Status) {
        return 0;
    }

    ObjectAttributes.Length = core::mem::size_of::<ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut usKeyName;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateKey(
        &mut hkey as *mut HKEY as PHANDLE,
        (KEY_READ | KEY_WRITE) as u32,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return 0;
    }

    RtlInitUnicodeString(&mut usValueName, pwszValueName);

    Status = ZwSetValueKey(
        hkey as _,
        &mut usValueName,
        0,
        ulType,
        pvData as *mut core::ffi::c_void,
        cbDataSize,
    );

    ZwClose(hkey as _);

    NT_SUCCESS(Status) as i32
}

unsafe fn IsStringType(Type: ULONG) -> BOOL {
    ((Type == REG_SZ) || (Type == REG_EXPAND_SZ) || (Type == REG_MULTI_SZ)) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn RegEnumValueW(
    hKey: HKEY,
    Index: ULONG,
    Name: LPWSTR,
    NameLength: PULONG,
    Type: PULONG,
    Data: PVOID,
    DataLength: PULONG,
) -> NTSTATUS {
    let mut ValueInfo: PKEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut BufferLength: ULONG;
    let mut ReturnedLength: ULONG = 0;
    let mut Status: NTSTATUS;

    /* Calculate the required buffer length */
    BufferLength =
        core::mem::offset_of!(KEY_VALUE_FULL_INFORMATION, Name) as ULONG;
    BufferLength += (MAX_PATH + 1) * core::mem::size_of::<u16>() as ULONG;
    if Data != core::ptr::null_mut() {
        BufferLength += *DataLength;
    }

    /* Allocate the value buffer */
    ValueInfo = ExAllocatePoolWithTag(PagedPool, BufferLength as SIZE_T, TAG_TEMP)
        as PKEY_VALUE_FULL_INFORMATION;
    if ValueInfo == core::ptr::null_mut() {
        return STATUS_NO_MEMORY;
    }

    /* Enumerate the value*/
    Status = ZwEnumerateValueKey(
        hKey as _,
        Index,
        KeyValueFullInformation,
        ValueInfo as PVOID,
        BufferLength,
        &mut ReturnedLength,
    );
    if NT_SUCCESS(Status) {
        'done: {
            if Name != core::ptr::null_mut() {
                /* Check if the name fits */
                if ((*ValueInfo).NameLength + core::mem::size_of::<u16>() as ULONG)
                    <= (*NameLength * core::mem::size_of::<u16>() as ULONG)
                {
                    /* Copy it */
                    core::ptr::copy(
                        core::ptr::addr_of!((*ValueInfo).Name) as *const u8,
                        Name as *mut u8,
                        (*ValueInfo).NameLength as usize,
                    );

                    /* Terminate the string */
                    *Name.add(
                        ((*ValueInfo).NameLength / core::mem::size_of::<u16>() as ULONG)
                            as usize,
                    ) = 0;
                } else {
                    /* Otherwise, we ran out of buffer space */
                    Status = STATUS_BUFFER_OVERFLOW;
                    break 'done;
                }
            }

            if Data != core::ptr::null_mut() {
                /* Check if the data fits */
                if (*ValueInfo).DataLength <= *DataLength {
                    /* Copy it */
                    core::ptr::copy(
                        ((ValueInfo as usize) + (*ValueInfo).DataOffset as usize)
                            as *const u8,
                        Data as *mut u8,
                        (*ValueInfo).DataLength as usize,
                    );

                    if IsStringType((*ValueInfo).Type) != 0
                        && (*ValueInfo).DataLength
                            <= *DataLength - core::mem::size_of::<u16>() as ULONG
                    {
                        let ptr = ((Data as usize) + (*ValueInfo).DataLength as usize)
                            as *mut u16;
                        if ptr > (Data as *mut u16) && *ptr.offset(-1) != 0 {
                            *ptr = 0;
                        }
                    }
                } else {
                    Status = STATUS_BUFFER_OVERFLOW;
                    break 'done;
                }
            }
        }
    }

    if NT_SUCCESS(Status) || Status == STATUS_BUFFER_OVERFLOW {
        if Type != core::ptr::null_mut() {
            *Type = (*ValueInfo).Type;
        }

        if NameLength != core::ptr::null_mut() {
            *NameLength = (*ValueInfo).NameLength;
        }

        if DataLength != core::ptr::null_mut() {
            *DataLength = (*ValueInfo).DataLength;
        }
    }

    /* Free the buffer and return status */
    if ValueInfo != core::ptr::null_mut() {
        ExFreePoolWithTag(ValueInfo as PVOID, TAG_TEMP);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn RegDeleteValueW(hKey: HKEY, pszValueName: LPCWSTR) -> NTSTATUS {
    let mut ustrName: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut ustrName, pszValueName);
    ZwDeleteValueKey(hKey as _, &mut ustrName)
}
