use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    HANDLE, LANGID, LCID, MmGetSessionLocaleId, MmSetSessionLocaleId, _OBJECT_ATTRIBUTES,
    _UNICODE_STRING, ObCloseHandle, PKEY_VALUE_PARTIAL_INFORMATION, PLCID, RtlInitUnicodeString,
    RtlOpenCurrentUser, RtlUnicodeStringToInteger, ZwClose, ZwOpenKey, ZwQueryValueKey,
    ZwSetValueKey, _KEY_VALUE_PARTIAL_INFORMATION, KPROCESSOR_MODE, ExSystemExceptionFilter,
};
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


#[no_mangle]
pub static mut PsDefaultSystemLocaleId: LCID = 0x00000409;

#[no_mangle]
pub static mut PsInstallUILanguageId: LANGID = 0x0409;

#[no_mangle]
pub static mut PsDefaultThreadLocaleId: LCID = 0x00000409;

#[no_mangle]
pub static mut PsDefaultUILanguageId: LANGID = 0x0409;

const BOGUS_LOCALE_ID: ULONG = 0xFFFF0000;

const REG_SZ: ULONG = 1;
const REG_DWORD: ULONG = 4;
const KEY_QUERY_VALUE: u32 = 0x0001;
const KEY_SET_VALUE: u32 = 0x0002;
const KEY_READ: u32 = 0x00020019;
const KEY_WRITE: u32 = 0x00020006;

extern "C" {
    fn ProbeForWriteLangId(Address: *mut LANGID);
}
extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
}

#[inline]
fn LANGIDFROMLCID(lcid: LCID) -> LANGID {
    lcid as LANGID
}

#[inline]
fn MAKELCID(lgid: LANGID, srtid: u16) -> LCID {
    ((srtid as LCID) << 16) | (lgid as LCID)
}

const SORT_DEFAULT: u16 = 0;

unsafe fn ExpValidateNlsLocaleData(
    LocaleData: PKEY_VALUE_PARTIAL_INFORMATION,
) -> BOOLEAN {
    let Data: *mut u16;

    if (*LocaleData).Type != REG_SZ {
        return 0u8;
    }

    if (*LocaleData).DataLength < core::mem::size_of::<u16>() as ULONG {
        return 0u8;
    }

    Data = (*LocaleData).Data.as_ptr() as *mut u16;
    if *Data.add(0) != b'1' as u16 || *Data.add(1) != 0 {
        return 0u8;
    }

    1u8
}

unsafe fn ExpValidateNlsLocaleId(LocaleId: LCID) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut NlsLocaleKey: HANDLE = core::ptr::null_mut();
    let mut AltSortKey: HANDLE = core::ptr::null_mut();
    let mut LangGroupKey: HANDLE = core::ptr::null_mut();
    let mut NlsLocalKeyAttrs: _OBJECT_ATTRIBUTES;
    let mut AltSortKeyAttrs: _OBJECT_ATTRIBUTES;
    let mut LangGroupKeyAttrs: _OBJECT_ATTRIBUTES;
    let BufferKey: PKEY_VALUE_PARTIAL_INFORMATION;
    let mut ValueBuffer: [u16; 20] = [0; 20];
    let mut LocaleIdBuffer: [u16; 20] = [0; 20];
    let mut ReturnedLength: ULONG = 0;
    let mut LocaleIdString: _UNICODE_STRING = core::mem::zeroed();
    // L"\Registry\Machine\SYSTEM\CurrentControlSet\Control\Nls\Locale"
    static NLS_LOCALE_KEY_PATH_BUF: [u16; 62] = [
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16,
        b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16, b'S' as u16, b'Y' as u16,
        b'S' as u16, b'T' as u16, b'E' as u16, b'M' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
        b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16,
        b'e' as u16, b't' as u16, b'\\' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16,
        b'\\' as u16, b'N' as u16, b'l' as u16, b's' as u16, b'\\' as u16, b'L' as u16, b'o' as u16, b'c' as u16, b'a' as u16, b'l' as u16,
        b'e' as u16, 0,
    ];
    static mut NlsLocaleKeyPath: _UNICODE_STRING = _UNICODE_STRING {
        Length: 122,
        MaximumLength: 124,
        Buffer: NLS_LOCALE_KEY_PATH_BUF.as_ptr() as *mut u16,
    };
    // L"\Registry\Machine\SYSTEM\CurrentControlSet\Control\Nls\Locale\Alternate Sorts"
    static ALT_SORT_KEY_PATH_BUF: [u16; 78] = [
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16,
        b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16, b'S' as u16, b'Y' as u16,
        b'S' as u16, b'T' as u16, b'E' as u16, b'M' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
        b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16,
        b'e' as u16, b't' as u16, b'\\' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16,
        b'\\' as u16, b'N' as u16, b'l' as u16, b's' as u16, b'\\' as u16, b'L' as u16, b'o' as u16, b'c' as u16, b'a' as u16, b'l' as u16,
        b'e' as u16, b'\\' as u16, b'A' as u16, b'l' as u16, b't' as u16, b'e' as u16, b'r' as u16, b'n' as u16, b'a' as u16, b't' as u16,
        b'e' as u16, b' ' as u16, b'S' as u16, b'o' as u16, b'r' as u16, b't' as u16, b's' as u16, 0,
    ];
    static mut AltSortKeyPath: _UNICODE_STRING = _UNICODE_STRING {
        Length: 154,
        MaximumLength: 156,
        Buffer: ALT_SORT_KEY_PATH_BUF.as_ptr() as *mut u16,
    };
    // L"\Registry\Machine\SYSTEM\CurrentControlSet\Control\Nls\Language Groups"
    static LANG_GROUP_PATH_BUF: [u16; 71] = [
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16,
        b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16, b'S' as u16, b'Y' as u16,
        b'S' as u16, b'T' as u16, b'E' as u16, b'M' as u16, b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
        b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16,
        b'e' as u16, b't' as u16, b'\\' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16,
        b'\\' as u16, b'N' as u16, b'l' as u16, b's' as u16, b'\\' as u16, b'L' as u16, b'a' as u16, b'n' as u16, b'g' as u16, b'u' as u16,
        b'a' as u16, b'g' as u16, b'e' as u16, b' ' as u16, b'G' as u16, b'r' as u16, b'o' as u16, b'u' as u16, b'p' as u16, b's' as u16,
        0,
    ];
    static mut LangGroupPath: _UNICODE_STRING = _UNICODE_STRING {
        Length: 140,
        MaximumLength: 142,
        Buffer: LANG_GROUP_PATH_BUF.as_ptr() as *mut u16,
    };

    NlsLocalKeyAttrs = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: &raw mut NlsLocaleKeyPath as *mut _UNICODE_STRING,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };

    AltSortKeyAttrs = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: &raw mut AltSortKeyPath as *mut _UNICODE_STRING,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };

    LangGroupKeyAttrs = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: &raw mut LangGroupPath as *mut _UNICODE_STRING,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };

    crate::swprintf!(
        LocaleIdBuffer.as_mut_ptr(),
        LocaleIdBuffer.len(),
        crate::wstr!("%08lx"),
        LocaleId as ULONG,
    );

    RtlInitUnicodeString(&mut LocaleIdString, LocaleIdBuffer.as_ptr());

    Status = ZwOpenKey(
        &mut NlsLocaleKey,
        KEY_QUERY_VALUE,
        &mut NlsLocalKeyAttrs,
    );
    if Status < 0 {
        return Status;
    }

    Status = ZwOpenKey(&mut AltSortKey, KEY_QUERY_VALUE, &mut AltSortKeyAttrs);
    if Status < 0 {
        ZwClose(NlsLocaleKey);
        return Status;
    }

    Status = ZwOpenKey(&mut LangGroupKey, KEY_QUERY_VALUE, &mut LangGroupKeyAttrs);
    if Status < 0 {
        ZwClose(AltSortKey);
        ZwClose(NlsLocaleKey);
        return Status;
    }

    BufferKey = ValueBuffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;
    Status = ZwQueryValueKey(
        NlsLocaleKey,
        &mut LocaleIdString,
        ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as _,
        BufferKey as PVOID,
        core::mem::size_of_val(&ValueBuffer) as ULONG,
        &mut ReturnedLength,
    );
    if Status < 0 {
        Status = ZwQueryValueKey(
            AltSortKey,
            &mut LocaleIdString,
            ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as _,
            BufferKey as PVOID,
            core::mem::size_of_val(&ValueBuffer) as ULONG,
            &mut ReturnedLength,
        );
        if Status < 0 {
            ZwClose(LangGroupKey);
            ZwClose(AltSortKey);
            ZwClose(NlsLocaleKey);
            return Status;
        }
    }

    if (*BufferKey).Type != REG_SZ
        || (*BufferKey).DataLength < core::mem::size_of::<u16>() as ULONG
    {
        Status = STATUS_OBJECT_NAME_NOT_FOUND;
        ZwClose(LangGroupKey);
        ZwClose(AltSortKey);
        ZwClose(NlsLocaleKey);
        return Status;
    }

    RtlInitUnicodeString(&mut LocaleIdString, (*BufferKey).Data.as_ptr() as *mut u16);
    Status = ZwQueryValueKey(
        LangGroupKey,
        &mut LocaleIdString,
        ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as _,
        BufferKey as PVOID,
        core::mem::size_of_val(&ValueBuffer) as ULONG,
        &mut ReturnedLength,
    );
    if Status < 0 {
        ZwClose(LangGroupKey);
        ZwClose(AltSortKey);
        ZwClose(NlsLocaleKey);
        return Status;
    }

    if ExpValidateNlsLocaleData(BufferKey) == 0 {
        Status = STATUS_OBJECT_NAME_NOT_FOUND;
    }

    ZwClose(LangGroupKey);
    ZwClose(AltSortKey);
    ZwClose(NlsLocaleKey);

    Status
}

unsafe fn ExpGetCurrentUserUILanguage(
    MuiName: PCWSTR,
    LanguageId: *mut LANGID,
) -> NTSTATUS {
    let mut ValueBuffer: [u8; 256] = [0; 256];
    let ValueInfo: PKEY_VALUE_PARTIAL_INFORMATION;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES;
    static mut KeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 42,
        MaximumLength: 44,
        Buffer: w!("Control Panel\\Desktop").as_ptr() as *mut u16,
    };
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueString: _UNICODE_STRING;
    let mut ValueLength: ULONG = 0;
    let mut Value: ULONG = 0;
    let mut UserKey: HANDLE = core::ptr::null_mut();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(&mut ValueName, MuiName);

    Status = RtlOpenCurrentUser(KEY_READ, &mut UserKey);
    if Status < 0 {
        return Status;
    }

    ObjectAttributes = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: UserKey,
        ObjectName: &raw mut KeyName as *mut _UNICODE_STRING,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };

    Status = ZwOpenKey(&mut KeyHandle, KEY_QUERY_VALUE, &mut ObjectAttributes);
    if Status >= 0 {
        ValueInfo = ValueBuffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;
        Status = ZwQueryValueKey(
            KeyHandle,
            &mut ValueName,
            ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as _,
            ValueBuffer.as_mut_ptr() as PVOID,
            core::mem::size_of_val(&ValueBuffer) as ULONG,
            &mut ValueLength,
        );
        if Status >= 0 {
            if (*ValueInfo).Type == REG_SZ {
                ValueString = _UNICODE_STRING {
                    Buffer: (*ValueInfo).Data.as_ptr() as *mut u16,
                    Length: 0,
                    MaximumLength: 0,
                };
                RtlInitUnicodeString(&mut ValueString, ValueString.Buffer);
                Status = RtlUnicodeStringToInteger(&mut ValueString, 16, &mut Value);
                if Status >= 0 {
                    *LanguageId = Value as LANGID;
                }
            } else {
                Status = STATUS_UNSUCCESSFUL;
            }
        }

        ZwClose(KeyHandle);
    }

    ZwClose(UserKey);
    Status
}

unsafe fn ExpSetCurrentUserUILanguage(
    MuiName: PCWSTR,
    LanguageId: LANGID,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES;
    static mut KeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 42,
        MaximumLength: 44,
        Buffer: w!("Control Panel\\Desktop").as_ptr() as *mut u16,
    };
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueBuffer: [u16; 8] = [0; 8];
    let mut ValueLength: ULONG;
    let mut UserHandle: HANDLE = core::ptr::null_mut();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if (LanguageId as ULONG & BOGUS_LOCALE_ID) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    RtlInitUnicodeString(&mut ValueName, MuiName);

    Status = RtlOpenCurrentUser(KEY_WRITE, &mut UserHandle);
    if Status < 0 {
        return Status;
    }

    ObjectAttributes = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: UserHandle,
        ObjectName: &raw mut KeyName as *mut _UNICODE_STRING,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };

    Status = ExpValidateNlsLocaleId(MAKELCID(LanguageId, SORT_DEFAULT));
    if Status >= 0 {
        Status = ZwOpenKey(&mut KeyHandle, KEY_SET_VALUE, &mut ObjectAttributes);
        if Status >= 0 {
            ValueLength = crate::swprintf!(
                ValueBuffer.as_mut_ptr(),
                ValueBuffer.len(),
                crate::wstr!("%04lX"),
                LanguageId as ULONG,
            ) as ULONG;

            ValueLength = (ValueLength + 1) * core::mem::size_of::<u16>() as ULONG;
            Status = ZwSetValueKey(
                KeyHandle,
                &mut ValueName,
                0,
                REG_SZ,
                ValueBuffer.as_mut_ptr() as PVOID,
                ValueLength,
            );

            ZwClose(KeyHandle);
        }
    }

    ZwClose(UserHandle);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryDefaultLocale(
    UserProfile: BOOLEAN,
    DefaultLocaleId: PLCID,
) -> NTSTATUS {
    let Status = 'seh: {
        if KeGetPreviousMode() != 0i8 {
            if let Err(e) = probe_user_buffer_write(
                DefaultLocaleId as *const core::ffi::c_void,
                core::mem::size_of::<LCID>(),
                core::mem::align_of::<LCID>(),
                KeGetPreviousMode(),
            ) {
                break 'seh e;
            }
        }

        if UserProfile != 0 {
            if let Err(e) = write_user(DefaultLocaleId, MmGetSessionLocaleId(), KeGetPreviousMode())
            {
                break 'seh e;
            }
        } else {
            if let Err(e) = write_user(
                DefaultLocaleId,
                PsDefaultSystemLocaleId,
                KeGetPreviousMode(),
            ) {
                break 'seh e;
            }
        }

        STATUS_SUCCESS
    };

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetDefaultLocale(
    UserProfile: BOOLEAN,
    DefaultLocaleId: LCID,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES;
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut LocaleString: _UNICODE_STRING;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ValueLength: ULONG = 0;
    let mut ValueBuffer: [u16; 20] = [0; 20];
    let mut UserKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut KeyValueBuffer: [u8; 256] = [0; 256];
    let KeyValueInformation: PKEY_VALUE_PARTIAL_INFORMATION;
    let mut LocalDefaultLocaleId = DefaultLocaleId;

    if (DefaultLocaleId & BOGUS_LOCALE_ID) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if UserProfile != 0 {
        Status = RtlOpenCurrentUser(KEY_WRITE, &mut UserKey);
        if Status < 0 {
            return Status;
        }

        RtlInitUnicodeString(
            &mut KeyName,
            w!("Control Panel\\International").as_ptr() as *const u16,
        );
        RtlInitUnicodeString(&mut ValueName, w!("Locale").as_ptr() as *const u16);
    } else {
        RtlInitUnicodeString(
            &mut KeyName,
            win32macros_rust::wide_bytes!(b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0N\0l\0s\0\\\0L\0a\0n\0g\0u\0a\0g\0e\0\0\0") as *const u16,
        );
        RtlInitUnicodeString(&mut ValueName, w!("Default").as_ptr() as *const u16);
    }

    ObjectAttributes = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: UserKey,
        ObjectName: &mut KeyName as *mut _UNICODE_STRING,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };

    if LocalDefaultLocaleId == 0 {
        Status = ZwOpenKey(&mut KeyHandle, KEY_QUERY_VALUE, &mut ObjectAttributes);
        if Status < 0 {
            if !UserKey.is_null() {
                ObCloseHandle(UserKey, 0i8);
            }
            return Status;
        }

        KeyValueInformation = KeyValueBuffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;
        Status = ZwQueryValueKey(
            KeyHandle,
            &mut ValueName,
            ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as _,
            KeyValueInformation as PVOID,
            core::mem::size_of_val(&KeyValueBuffer) as ULONG,
            &mut ValueLength,
        );
        if Status < 0 {
            if !KeyHandle.is_null() {
                ObCloseHandle(KeyHandle, 0i8);
            }
            if !UserKey.is_null() {
                ObCloseHandle(UserKey, 0i8);
            }
            return Status;
        }

        if (*KeyValueInformation).Type == REG_DWORD
            && (*KeyValueInformation).DataLength == core::mem::size_of::<ULONG>() as ULONG
        {
            LocalDefaultLocaleId = *((*KeyValueInformation).Data.as_ptr() as *const ULONG);
        } else if (*KeyValueInformation).Type == REG_SZ {
            LocaleString = _UNICODE_STRING {
                Buffer: (*KeyValueInformation).Data.as_ptr() as *mut u16,
                Length: (*KeyValueInformation).DataLength as u16,
                MaximumLength: (*KeyValueInformation).DataLength as u16,
            };

            RtlUnicodeStringToInteger(&mut LocaleString, 16, &mut LocalDefaultLocaleId);
        } else {
            Status = STATUS_UNSUCCESSFUL;
        }
    } else {
        Status = ExpValidateNlsLocaleId(LocalDefaultLocaleId);
        if Status >= 0 {
            Status = ZwOpenKey(&mut KeyHandle, KEY_SET_VALUE, &mut ObjectAttributes);
            if Status >= 0 {
                if UserProfile != 0 {
                    ValueLength = crate::swprintf!(
                        ValueBuffer.as_mut_ptr(),
                        ValueBuffer.len(),
                        crate::wstr!("%08lx"),
                        LocalDefaultLocaleId as ULONG,
                    ) as ULONG;
                } else {
                    ValueLength = crate::swprintf!(
                        ValueBuffer.as_mut_ptr(),
                        ValueBuffer.len(),
                        crate::wstr!("%04lx"),
                        (LocalDefaultLocaleId as ULONG) & 0xFFFF,
                    ) as ULONG;
                }

                ValueLength = (ValueLength + 1) * core::mem::size_of::<u16>() as ULONG;

                Status = ZwSetValueKey(
                    KeyHandle,
                    &mut ValueName,
                    0,
                    REG_SZ,
                    ValueBuffer.as_mut_ptr() as PVOID,
                    ValueLength,
                );
            }
        }
    }

    if !KeyHandle.is_null() {
        ObCloseHandle(KeyHandle, 0i8);
    }

    if !UserKey.is_null() {
        ObCloseHandle(UserKey, 0i8);
    }

    if Status >= 0 {
        if UserProfile != 0 {
            MmSetSessionLocaleId(LocalDefaultLocaleId);
        } else {
            PsDefaultSystemLocaleId = LocalDefaultLocaleId;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryInstallUILanguage(LanguageId: *mut LANGID) -> NTSTATUS {
    let LanguageId = opaque_ptr(LanguageId);
    let Status = 'seh: {
        if KeGetPreviousMode() != 0i8 {
            if let Err(e) = probe_user_buffer_write(
                LanguageId as *const core::ffi::c_void,
                core::mem::size_of::<LANGID>(),
                core::mem::align_of::<LANGID>(),
                KeGetPreviousMode(),
            ) {
                break 'seh e;
            }
        }

        if let Err(e) = write_user(LanguageId, PsInstallUILanguageId, KeGetPreviousMode()) {
            break 'seh e;
        }

        STATUS_SUCCESS
    };

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryDefaultUILanguage(LanguageId: *mut LANGID) -> NTSTATUS {
    let LanguageId = opaque_ptr(LanguageId);
    let mut SafeLanguageId: LANGID = 0;

    let Status = ExpGetCurrentUserUILanguage(
        w!("MultiUILanguageId").as_ptr() as *const u16,
        &mut SafeLanguageId,
    );

    let FinalStatus = 'seh: {
        if KeGetPreviousMode() != 0i8 {
            if let Err(e) = probe_user_buffer_write(
                LanguageId as *const core::ffi::c_void,
                core::mem::size_of::<LANGID>(),
                core::mem::align_of::<LANGID>(),
                KeGetPreviousMode(),
            ) {
                break 'seh e;
            }
        }

        if Status >= 0 {
            if let Err(e) = write_user(LanguageId, SafeLanguageId, KeGetPreviousMode()) {
                break 'seh e;
            }
        } else {
            if let Err(e) = write_user(LanguageId, PsInstallUILanguageId, KeGetPreviousMode()) {
                break 'seh e;
            }
        }

        STATUS_SUCCESS
    };

    FinalStatus
}

#[no_mangle]
pub unsafe extern "system" fn NtSetDefaultUILanguage(LanguageId: LANGID) -> NTSTATUS {
    let Status: NTSTATUS;

    if LanguageId != 0 {
        Status = ExpSetCurrentUserUILanguage(
            w!("MUILanguagePending").as_ptr() as *const u16,
            LanguageId,
        );
    } else {
        let mut LocalLanguageId: LANGID = 0;
        let mut TempStatus = ExpGetCurrentUserUILanguage(
            w!("MUILanguagePending").as_ptr() as *const u16,
            &mut LocalLanguageId,
        );
        if TempStatus < 0 {
            return TempStatus;
        }

        TempStatus = ExpSetCurrentUserUILanguage(
            w!("MultiUILanguageId").as_ptr() as *const u16,
            LocalLanguageId,
        );
        Status = TempStatus;
    }

    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ex/locale.c
//  * PURPOSE:         Locale (Language) Support for the Executive
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Eric Kohl
//  *                  Thomas Weidenmueller (w3seek@reactos.org
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// /* System IDs: EN_US */
// LCID PsDefaultSystemLocaleId = 0x00000409;
// LANGID PsInstallUILanguageId = LANGIDFROMLCID(0x00000409);
//
// /* UI/Thread IDs: Same as system */
// LCID PsDefaultThreadLocaleId = 0x00000409;
// LANGID PsDefaultUILanguageId = LANGIDFROMLCID(0x00000409);
//
// /* DEFINES *******************************************************************/
//
// #define BOGUS_LOCALE_ID 0xFFFF0000
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// /**
//  * @brief
//  * Validates the registry data of a NLS locale.
//  *
//  * @param[in] LocaleData
//  * A pointer to partial information that contains
//  * the NLS locale data.
//  *
//  * @return
//  * Returns TRUE if the following conditions are met,
//  * otherwise FALSE is returned.
//  */
// static
// __inline
// BOOLEAN
// ExpValidateNlsLocaleData(
//     _In_ PKEY_VALUE_PARTIAL_INFORMATION LocaleData)
// {
//     PWCHAR Data;
//
//     /* Is this a null-terminated string type? */
//     if (LocaleData->Type != REG_SZ)
//     {
//         return FALSE;
//     }
//
//     /* Does it have a consistent length? */
//     if (LocaleData->DataLength < sizeof(WCHAR))
//     {
//         return FALSE;
//     }
//
//     /* Is the locale set and null-terminated? */
//     Data = (PWSTR)LocaleData->Data;
//     if (Data[0] != L'1' || Data[1] != UNICODE_NULL)
//     {
//         return FALSE;
//     }
//
//     /* All of the conditions above are met */
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Validates a NLS locale. Whether a locale is valid
//  * or not depends on the following conditions:
//  *
//  * - The locale must exist in the Locale key, otherwise
//  *   in the Alternate Sorts key;
//  *
//  * - The locale must exist in the Language Groups key, and
//  *   the queried value must be readable;
//  *
//  * - The locale registry data must be of REG_SIZE type,
//  *   has a consistent length and the locale belongs to
//  *   a supported language group that is set.
//  *
//  * @param[in] LocaleId
//  * A locale identifier that corresponds to a specific
//  * locale to be validated.
//  *
//  * @return
//  * Returns STATUS_SUCCESS if the function has successfully
//  * validated the locale and it is valid. STATUS_OBJECT_NAME_NOT_FOUND
//  * is returned if the following locale does not exist on the system.
//  * A failure NTSTATUS code is returned otherwise.
//  */
// static
// NTSTATUS
// ExpValidateNlsLocaleId(
//     _In_ LCID LocaleId)
// {
//     NTSTATUS Status;
//     HANDLE NlsLocaleKey = NULL, AltSortKey = NULL, LangGroupKey = NULL;
//     _OBJECT_ATTRIBUTES NlsLocalKeyAttrs, AltSortKeyAttrs, LangGroupKeyAttrs;
//     PKEY_VALUE_PARTIAL_INFORMATION BufferKey;
//     WCHAR ValueBuffer[20], LocaleIdBuffer[20];
//     ULONG ReturnedLength;
//     _UNICODE_STRING LocaleIdString;
//     static _UNICODE_STRING NlsLocaleKeyPath = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control\\Nls\\Locale");
//     static _UNICODE_STRING AltSortKeyPath = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control\\Nls\\Locale\\Alternate Sorts");
//     static _UNICODE_STRING LangGroupPath = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Control\\Nls\\Language Groups");
//
//     /* Initialize the registry path attributes */
//     InitializeObjectAttributes(&NlsLocalKeyAttrs,
//                                &NlsLocaleKeyPath,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     InitializeObjectAttributes(&AltSortKeyAttrs,
//                                &AltSortKeyPath,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     InitializeObjectAttributes(&LangGroupKeyAttrs,
//                                &LangGroupPath,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//
//     /* Copy the locale ID into a buffer */
//     swprintf(LocaleIdBuffer,
//              L"%08lx",
//              (ULONG)LocaleId);
//
//     /* And build the LCID string */
//     RtlInitUnicodeString(&LocaleIdString, LocaleIdBuffer);
//
//     /* Open the NLS locale key */
//     Status = ZwOpenKey(&NlsLocaleKey,
//                        KEY_QUERY_VALUE,
//                        &NlsLocalKeyAttrs);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open %wZ (Status 0x%lx)\n", NlsLocaleKeyPath, Status);
//         return Status;
//     }
//
//     /* Open the NLS alternate sort locales key */
//     Status = ZwOpenKey(&AltSortKey,
//                        KEY_QUERY_VALUE,
//                        &AltSortKeyAttrs);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open %wZ (Status 0x%lx)\n", AltSortKeyPath, Status);
//         goto Quit;
//     }
//
//     /* Open the NLS language groups key */
//     Status = ZwOpenKey(&LangGroupKey,
//                        KEY_QUERY_VALUE,
//                        &LangGroupKeyAttrs);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open %wZ (Status 0x%lx)\n", LangGroupPath, Status);
//         goto Quit;
//     }
//
//     /* Check if the captured locale ID exists in the list of other locales */
//     BufferKey = (PKEY_VALUE_PARTIAL_INFORMATION)ValueBuffer;
//     Status = ZwQueryValueKey(NlsLocaleKey,
//                              &LocaleIdString,
//                              KeyValuePartialInformation,
//                              BufferKey,
//                              sizeof(ValueBuffer),
//                              &ReturnedLength);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed, retry by looking at the alternate sorts locales */
//         Status = ZwQueryValueKey(AltSortKey,
//                                  &LocaleIdString,
//                                  KeyValuePartialInformation,
//                                  BufferKey,
//                                  sizeof(ValueBuffer),
//                                  &ReturnedLength);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to query value from Alternate Sorts key (Status 0x%lx)\n", Status);
//             goto Quit;
//         }
//     }
//
//     /* Ensure the queried locale is of the right key type with a sane length */
//     if (BufferKey->Type != REG_SZ ||
//         BufferKey->DataLength < sizeof(WCHAR))
//     {
//         DPRINT1("The queried locale is of bad value type or length (Type %lu, DataLength %lu)\n",
//                 BufferKey->Type, BufferKey->DataLength);
//         Status = STATUS_OBJECT_NAME_NOT_FOUND;
//         goto Quit;
//     }
//
//     /* We got what we need, now query the locale from the language groups */
//     RtlInitUnicodeString(&LocaleIdString, (PWSTR)BufferKey->Data);
//     Status = ZwQueryValueKey(LangGroupKey,
//                              &LocaleIdString,
//                              KeyValuePartialInformation,
//                              BufferKey,
//                              sizeof(ValueBuffer),
//                              &ReturnedLength);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to query value from Language Groups key (Status 0x%lx)\n", Status);
//         goto Quit;
//     }
//
//     /*
//      * We have queried the locale with its data. However we are not finished here yet,
//      * because the locale data could be malformed or the locale itself was not set
//      * so ensure all of these conditions are met.
//      */
//     if (!ExpValidateNlsLocaleData(BufferKey))
//     {
//         DPRINT1("The locale data is not valid!\n");
//         Status = STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//
// Quit:
//     if (LangGroupKey != NULL)
//     {
//         ZwClose(LangGroupKey);
//     }
//
//     if (AltSortKey != NULL)
//     {
//         ZwClose(AltSortKey);
//     }
//
//     if (NlsLocaleKey != NULL)
//     {
//         ZwClose(NlsLocaleKey);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// ExpGetCurrentUserUILanguage(IN PCWSTR MuiName,
//                             OUT LANGID* LanguageId)
// {
//     UCHAR ValueBuffer[256];
//     PKEY_VALUE_PARTIAL_INFORMATION ValueInfo;
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//     _UNICODE_STRING KeyName =
//         RTL_CONSTANT_STRING(L"Control Panel\\Desktop");
//     _UNICODE_STRING ValueName;
//     _UNICODE_STRING ValueString;
//     ULONG ValueLength;
//     ULONG Value;
//     HANDLE UserKey;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Setup the key name */
//     RtlInitUnicodeString(&ValueName, MuiName);
//
//     /* Open the use key */
//     Status = RtlOpenCurrentUser(KEY_READ, &UserKey);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Initialize the attributes and open the key */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                UserKey,
//                                NULL);
//     Status = ZwOpenKey(&KeyHandle, KEY_QUERY_VALUE,&ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         /* Set buffer and query the current value */
//         ValueInfo = (PKEY_VALUE_PARTIAL_INFORMATION)ValueBuffer;
//         Status = ZwQueryValueKey(KeyHandle,
//                                  &ValueName,
//                                  KeyValuePartialInformation,
//                                  ValueBuffer,
//                                  sizeof(ValueBuffer),
//                                  &ValueLength);
//         if (NT_SUCCESS(Status))
//         {
//             /* Success, is the value the right type? */
//             if (ValueInfo->Type == REG_SZ)
//             {
//                 /* It is. Initialize the data and convert it */
//                 RtlInitUnicodeString(&ValueString, (PWSTR)ValueInfo->Data);
//                 Status = RtlUnicodeStringToInteger(&ValueString, 16, &Value);
//                 if (NT_SUCCESS(Status))
//                 {
//                     /* Return the language */
//                     *LanguageId = (USHORT)Value;
//                 }
//             }
//             else
//             {
//                 /* Fail */
//                 Status = STATUS_UNSUCCESSFUL;
//             }
//         }
//
//         /* Close the key */
//         ZwClose(KeyHandle);
//     }
//
//     /* Close the user key and return */
//     ZwClose(UserKey);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// ExpSetCurrentUserUILanguage(IN PCWSTR MuiName,
//                             IN LANGID LanguageId)
// {
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//     _UNICODE_STRING KeyName = RTL_CONSTANT_STRING(L"Control Panel\\Desktop");
//     _UNICODE_STRING ValueName;
//     WCHAR ValueBuffer[8];
//     ULONG ValueLength;
//     HANDLE UserHandle;
//     HANDLE KeyHandle;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check that the passed language ID is not bogus */
//     if (LanguageId & BOGUS_LOCALE_ID)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Setup the key name */
//     RtlInitUnicodeString(&ValueName, MuiName);
//
//     /* Open the use key */
//     Status = RtlOpenCurrentUser(KEY_WRITE, &UserHandle);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Initialize the attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                UserHandle,
//                                NULL);
//
//     /* Validate the language ID */
//     Status = ExpValidateNlsLocaleId(MAKELCID(LanguageId, SORT_DEFAULT));
//     if (NT_SUCCESS(Status))
//     {
//         /* Open the key */
//         Status = ZwOpenKey(&KeyHandle, KEY_SET_VALUE, &ObjectAttributes);
//         if (NT_SUCCESS(Status))
//         {
//             /* Setup the value name */
//             ValueLength = swprintf(ValueBuffer,
//                                    L"%04lX",
//                                    (ULONG)LanguageId);
//
//             /* Set the length for the call and set the value */
//             ValueLength = (ValueLength + 1) * sizeof(WCHAR);
//             Status = ZwSetValueKey(KeyHandle,
//                                    &ValueName,
//                                    0,
//                                    REG_SZ,
//                                    ValueBuffer,
//                                    ValueLength);
//
//             /* Close the handle for this key */
//             ZwClose(KeyHandle);
//         }
//     }
//
//     /* Close the user key and return status */
//     ZwClose(UserHandle);
//     return Status;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// NTSTATUS
// NTAPI
// NtQueryDefaultLocale(IN BOOLEAN UserProfile,
//                      OUT PLCID DefaultLocaleId)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PAGED_CODE();
//
//     /* Enter SEH for probing */
//     _SEH2_TRY
//     {
//         /* Check if we came from user mode */
//         if (KeGetPreviousMode() != KernelMode)
//         {
//             /* Probe the language ID */
//             ProbeForWriteLangId(DefaultLocaleId);
//         }
//
//         /* Check if we have a user profile */
//         if (UserProfile)
//         {
//             /* Return session wide thread locale */
//             *DefaultLocaleId = MmGetSessionLocaleId();
//         }
//         else
//         {
//             /* Return system locale */
//             *DefaultLocaleId = PsDefaultSystemLocaleId;
//         }
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Get exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtSetDefaultLocale(IN BOOLEAN UserProfile,
//                    IN LCID DefaultLocaleId)
// {
//     _OBJECT_ATTRIBUTES ObjectAttributes;
//     _UNICODE_STRING KeyName;
//     _UNICODE_STRING ValueName;
//     _UNICODE_STRING LocaleString;
//     HANDLE KeyHandle = NULL;
//     ULONG ValueLength;
//     WCHAR ValueBuffer[20];
//     HANDLE UserKey = NULL;
//     NTSTATUS Status;
//     UCHAR KeyValueBuffer[256];
//     PKEY_VALUE_PARTIAL_INFORMATION KeyValueInformation;
//     PAGED_CODE();
//
//     /* Check that the passed locale ID is not bogus */
//     if (DefaultLocaleId & BOGUS_LOCALE_ID)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check if we have a profile */
//     if (UserProfile)
//     {
//         /* Open the user's key */
//         Status = RtlOpenCurrentUser(KEY_WRITE, &UserKey);
//         if (!NT_SUCCESS(Status)) return Status;
//
//         /* Initialize the registry location */
//         RtlInitUnicodeString(&KeyName, L"Control Panel\\International");
//         RtlInitUnicodeString(&ValueName, L"Locale");
//     }
//     else
//     {
//         /* Initialize the system registry location */
//         RtlInitUnicodeString(&KeyName,
//                              L"\\Registry\\Machine\\System\\CurrentControlSet"
//                              L"\\Control\\Nls\\Language");
//         RtlInitUnicodeString(&ValueName, L"Default");
//     }
//
//     /* Initialize the object attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                UserKey,
//                                NULL);
//
//     /* Check if we don't have a default locale yet */
//     if (!DefaultLocaleId)
//     {
//         /* Open the key for reading */
//         Status = ZwOpenKey(&KeyHandle, KEY_QUERY_VALUE, &ObjectAttributes);
//         if (!NT_SUCCESS(Status))
//         {
//             goto Cleanup;
//         }
//
//         /* Query the key value */
//         KeyValueInformation = (PKEY_VALUE_PARTIAL_INFORMATION)KeyValueBuffer;
//         Status = ZwQueryValueKey(KeyHandle,
//                                  &ValueName,
//                                  KeyValuePartialInformation,
//                                  KeyValueInformation,
//                                  sizeof(KeyValueBuffer),
//                                  &ValueLength);
//         if (!NT_SUCCESS(Status))
//         {
//             goto Cleanup;
//         }
//
//         /* Check if this is a REG_DWORD */
//         if ((KeyValueInformation->Type == REG_DWORD) &&
//             (KeyValueInformation->DataLength == sizeof(ULONG)))
//         {
//             /* It contains the LCID as a DWORD */
//             DefaultLocaleId = *((ULONG*)KeyValueInformation->Data);
//         }
//         /* Otherwise check for a REG_SZ */
//         else if (KeyValueInformation->Type == REG_SZ)
//         {
//             /* Initialize a unicode string from the value data */
//             LocaleString.Buffer = (PWCHAR)KeyValueInformation->Data;
//             LocaleString.Length = (USHORT)KeyValueInformation->DataLength;
//             LocaleString.MaximumLength = LocaleString.Length;
//
//             /* Convert the hex string to a number */
//             RtlUnicodeStringToInteger(&LocaleString, 16, &DefaultLocaleId);
//         }
//         else
//         {
//             Status = STATUS_UNSUCCESSFUL;
//         }
//     }
//     else
//     {
//         /* We have a locale, validate it */
//         Status = ExpValidateNlsLocaleId(DefaultLocaleId);
//         if (NT_SUCCESS(Status))
//         {
//             /* Open the key now */
//             Status = ZwOpenKey(&KeyHandle, KEY_SET_VALUE, &ObjectAttributes);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Check if we had a profile */
//                 if (UserProfile)
//                 {
//                     /* Fill in the buffer */
//                     ValueLength = swprintf(ValueBuffer,
//                                            L"%08lx",
//                                            (ULONG)DefaultLocaleId);
//                 }
//                 else
//                 {
//                     /* Fill in the buffer */
//                     ValueLength = swprintf(ValueBuffer,
//                                            L"%04lx",
//                                            (ULONG)DefaultLocaleId & 0xFFFF);
//                 }
//
//                 /* Set the length for the registry call */
//                 ValueLength = (ValueLength + 1) * sizeof(WCHAR);
//
//                 /* Now write the actual value */
//                 Status = ZwSetValueKey(KeyHandle,
//                                        &ValueName,
//                                        0,
//                                        REG_SZ,
//                                        ValueBuffer,
//                                        ValueLength);
//             }
//         }
//     }
//
// Cleanup:
//
//     /* Close the locale key */
//     if (KeyHandle)
//     {
//         ObCloseHandle(KeyHandle, KernelMode);
//     }
//
//     /* Close the user key */
//     if (UserKey)
//     {
//         ObCloseHandle(UserKey, KernelMode);
//     }
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Check if it was for a user */
//         if (UserProfile)
//         {
//             /* Set the session wide thread locale */
//             MmSetSessionLocaleId(DefaultLocaleId);
//         }
//         else
//         {
//             /* Set system locale */
//             PsDefaultSystemLocaleId = DefaultLocaleId;
//         }
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtQueryInstallUILanguage(OUT LANGID* LanguageId)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PAGED_CODE();
//
//     /* Enter SEH for probing */
//     _SEH2_TRY
//     {
//         /* Check if we came from user mode */
//         if (KeGetPreviousMode() != KernelMode)
//         {
//             /* Probe the Language ID */
//             ProbeForWriteLangId(LanguageId);
//         }
//
//         /* Return it */
//         *LanguageId = PsInstallUILanguageId;
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Get exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Return status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtQueryDefaultUILanguage(OUT LANGID* LanguageId)
// {
//     NTSTATUS Status;
//     LANGID SafeLanguageId;
//     PAGED_CODE();
//
//     /* Call the executive helper routine */
//     Status = ExpGetCurrentUserUILanguage(L"MultiUILanguageId", &SafeLanguageId);
//
//     /* Enter SEH for probing */
//     _SEH2_TRY
//     {
//         /* Check if we came from user mode */
//         if (KeGetPreviousMode() != KernelMode)
//         {
//             /* Probe the Language ID */
//             ProbeForWriteLangId(LanguageId);
//         }
//
//         if (NT_SUCCESS(Status))
//         {
//             /* Success, return the language */
//             *LanguageId = SafeLanguageId;
//         }
//         else
//         {
//             /* Failed, use fallback value */
//             // NOTE: Windows doesn't use PsDefaultUILanguageId.
//             *LanguageId = PsInstallUILanguageId;
//         }
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Return exception code */
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtSetDefaultUILanguage(IN LANGID LanguageId)
// {
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if the caller specified a language id */
//     if (LanguageId)
//     {
//         /* Set the pending MUI language id */
//         Status = ExpSetCurrentUserUILanguage(L"MUILanguagePending", LanguageId);
//     }
//     else
//     {
//         /* Otherwise get the pending MUI language id */
//         Status = ExpGetCurrentUserUILanguage(L"MUILanguagePending", &LanguageId);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//
//         /* And apply it as actual */
//         Status = ExpSetCurrentUserUILanguage(L"MultiUILanguageId", LanguageId);
//     }
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
