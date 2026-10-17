use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::reg_reg::RegCloseKey;
use crate::reg_reg::RegCreateKeyExW;
use crate::reg_reg::RegDeleteKeyW;
use crate::reg_reg::RegEnumKeyW;
use crate::reg_reg::RegOpenKeyExW;
use crate::reg_reg::RegQueryInfoKeyW;
use crate::reg_reg::RegQueryValueExW;
use crate::reg_reg::RegSetValueExW;
use core::ffi::c_void;

extern "system" {
    fn SystemFunction032(data: *mut ustring, key: *mut ustring) -> i32;
}

// Reg* error-code returns are used as DWORD throughout this file; DWORD-returning
// local decls shadow the prelude's LONG ones (link to the same advapi32 reg
// exports — ABI-identical, the return register width is unchanged).
extern "system" {
}

// wine/unicode.h string macros (cred.c) → CRT wide funcs (cdecl).
extern "C" {
    fn wcschr(s: *const u16, c: u16) -> *mut u16;
    fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16;
}
#[inline]
unsafe fn strlenW(s: *const u16) -> usize { wcslen(s) }
#[inline]
unsafe fn strcpyW(d: *mut u16, s: *const u16) -> *mut u16 { wcscpy(d, s) }
#[inline]
unsafe fn strcatW(d: *mut u16, s: *const u16) -> *mut u16 { wcscat(d, s) }
#[inline]
unsafe fn strchrW(s: *const u16, c: u16) -> *mut u16 { wcschr(s, c) }
#[inline]
unsafe fn strcmpiW(a: *const u16, b: *const u16) -> i32 { _wcsicmp(a, b) }

// CRED_MARSHAL_TYPE variants (module-local — see prelude.rs note).
const CertCredential: CRED_MARSHAL_TYPE = advapi32_sys::_CRED_MARSHAL_TYPE_CertCredential;
const UsernameTargetCredential: CRED_MARSHAL_TYPE =
    advapi32_sys::_CRED_MARSHAL_TYPE_UsernameTargetCredential;
const BinaryBlobCredential: CRED_MARSHAL_TYPE =
    advapi32_sys::_CRED_MARSHAL_TYPE_BinaryBlobCredential;

// ---- file-scope preamble (translated) ----
pub const KEY_SIZE: usize = 8;

pub static wszCredentialManagerKey: [u16; 33] = [
    'S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16, 'e' as u16, '\\' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'e' as u16, '\\' as u16,
    'C' as u16, 'r' as u16, 'e' as u16, 'd' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'i' as u16, 'a' as u16, 'l' as u16, ' ' as u16, 'M' as u16, 'a' as u16, 'n' as u16, 'a' as u16, 'g' as u16, 'e' as u16, 'r' as u16, 0,
];

pub static wszEncryptionKeyValue: [u16; 14] = [
    'E' as u16, 'n' as u16, 'c' as u16, 'r' as u16, 'y' as u16, 'p' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16, 'K' as u16, 'e' as u16, 'y' as u16, 0,
];

pub static wszFlagsValue: [u16; 6] = [
    'F' as u16, 'l' as u16, 'a' as u16, 'g' as u16, 's' as u16, 0,
];

pub static wszTypeValue: [u16; 5] = [
    'T' as u16, 'y' as u16, 'p' as u16, 'e' as u16, 0,
];

pub static wszCommentValue: [u16; 8] = [
    'C' as u16, 'o' as u16, 'm' as u16, 'm' as u16, 'e' as u16, 'n' as u16, 't' as u16, 0,
];

pub static wszLastWrittenValue: [u16; 12] = [
    'L' as u16, 'a' as u16, 's' as u16, 't' as u16, 'W' as u16, 'r' as u16, 'i' as u16, 't' as u16, 't' as u16, 'e' as u16, 'n' as u16, 0,
];

pub static wszPersistValue: [u16; 8] = [
    'P' as u16, 'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 's' as u16, 't' as u16, 0,
];

pub static wszTargetAliasValue: [u16; 12] = [
    'T' as u16, 'a' as u16, 'r' as u16, 'g' as u16, 'e' as u16, 't' as u16, 'A' as u16, 'l' as u16, 'i' as u16, 'a' as u16, 's' as u16, 0,
];

pub static wszUserNameValue: [u16; 9] = [
    'U' as u16, 's' as u16, 'e' as u16, 'r' as u16, 'N' as u16, 'a' as u16, 'm' as u16, 'e' as u16, 0,
];

pub static wszPasswordValue: [u16; 9] = [
    'P' as u16, 'a' as u16, 's' as u16, 's' as u16, 'w' as u16, 'o' as u16, 'r' as u16, 'd' as u16, 0,
];

// (translator note) The following helpers sit alongside the __APPLE__-guarded
// keychain path in wine/cred.c; translated faithfully for completeness.

unsafe fn read_credential_blob(hkey: HKEY, key_data: *const u8, credential_blob: LPBYTE, credential_blob_size: *mut u32) -> u32 {

    let mut ret: u32;
    let mut r#type: u32 = 0;

    *credential_blob_size = 0;
    ret = (RegQueryValueExW(hkey, wszPasswordValue.as_ptr(), 0 as LPDWORD, &mut r#type, core::ptr::null_mut(), credential_blob_size)) as u32;
    if ret != ERROR_SUCCESS {
        return ret;
    } else if r#type != REG_BINARY {
        return ERROR_REGISTRY_CORRUPT;
    }
    if !credential_blob.is_null() {
        let mut data: ustring = core::mem::zeroed();
        let mut key: ustring = core::mem::zeroed();

        ret = (RegQueryValueExW(hkey, wszPasswordValue.as_ptr(), 0 as LPDWORD, &mut r#type, credential_blob, credential_blob_size)) as u32;
        if ret != ERROR_SUCCESS {
            return ret;
        } else if r#type != REG_BINARY {
            return ERROR_REGISTRY_CORRUPT;
        }

        key.Length = KEY_SIZE as _;
        key.MaximumLength = KEY_SIZE as _;
        key.Buffer = key_data as *mut u8;

        data.Length = *credential_blob_size as _;
        data.MaximumLength = *credential_blob_size as _;
        data.Buffer = credential_blob;
        SystemFunction032(&mut data, &mut key);
    }
    ERROR_SUCCESS
}

unsafe fn registry_read_credential(hkey: HKEY, credential: PCREDENTIALW, key_data: *const u8, mut buffer: *mut i8, len: *mut u32) -> u32 {
    let mut type_: u32 = 0;
    let mut ret: u32;
    let mut count: u32 = 0;

    ret = (RegQueryValueExW(hkey, core::ptr::null(), core::ptr::null_mut(), &mut type_, core::ptr::null_mut(), &mut count)) as u32;
    if ret != ERROR_SUCCESS {
        return ret;
    } else if type_ != REG_SZ {
        return ERROR_REGISTRY_CORRUPT;
    }
    *len += count;
    if !credential.is_null() {
        (*credential).TargetName = buffer as LPWSTR;
        ret = (RegQueryValueExW(hkey, core::ptr::null(), core::ptr::null_mut(), &mut type_, (*credential).TargetName as LPBYTE, &mut count)) as u32;
        if ret != ERROR_SUCCESS {
            return ret;
        } else if type_ != REG_SZ {
            return ERROR_REGISTRY_CORRUPT;
        }
        buffer = buffer.add(count as usize);
    }

    ret = (RegQueryValueExW(hkey, wszCommentValue.as_ptr(), core::ptr::null_mut(), &mut type_, core::ptr::null_mut(), &mut count)) as u32;
    if ret != ERROR_FILE_NOT_FOUND {
        if ret != ERROR_SUCCESS {
            return ret;
        } else if type_ != REG_SZ {
            return ERROR_REGISTRY_CORRUPT;
        }
        *len += count;
    }
    if !credential.is_null() {
        (*credential).Comment = buffer as LPWSTR;
        ret = (RegQueryValueExW(hkey, wszCommentValue.as_ptr(), core::ptr::null_mut(), &mut type_, (*credential).Comment as LPBYTE, &mut count)) as u32;
        if ret == ERROR_FILE_NOT_FOUND {
            (*credential).Comment = core::ptr::null_mut();
        } else if ret != ERROR_SUCCESS {
            return ret;
        } else if type_ != REG_SZ {
            return ERROR_REGISTRY_CORRUPT;
        } else {
            buffer = buffer.add(count as usize);
        }
    }

    ret = (RegQueryValueExW(hkey, wszTargetAliasValue.as_ptr(), core::ptr::null_mut(), &mut type_, core::ptr::null_mut(), &mut count)) as u32;
    if ret != ERROR_FILE_NOT_FOUND {
        if ret != ERROR_SUCCESS {
            return ret;
        } else if type_ != REG_SZ {
            return ERROR_REGISTRY_CORRUPT;
        }
        *len += count;
    }
    if !credential.is_null() {
        (*credential).TargetAlias = buffer as LPWSTR;
        ret = (RegQueryValueExW(hkey, wszTargetAliasValue.as_ptr(), core::ptr::null_mut(), &mut type_, (*credential).TargetAlias as LPBYTE, &mut count)) as u32;
        if ret == ERROR_FILE_NOT_FOUND {
            (*credential).TargetAlias = core::ptr::null_mut();
        } else if ret != ERROR_SUCCESS {
            return ret;
        } else if type_ != REG_SZ {
            return ERROR_REGISTRY_CORRUPT;
        } else {
            buffer = buffer.add(count as usize);
        }
    }

    ret = (RegQueryValueExW(hkey, wszUserNameValue.as_ptr(), core::ptr::null_mut(), &mut type_, core::ptr::null_mut(), &mut count)) as u32;
    if ret != ERROR_FILE_NOT_FOUND {
        if ret != ERROR_SUCCESS {
            return ret;
        } else if type_ != REG_SZ {
            return ERROR_REGISTRY_CORRUPT;
        }
        *len += count;
    }
    if !credential.is_null() {
        (*credential).UserName = buffer as LPWSTR;
        ret = (RegQueryValueExW(hkey, wszUserNameValue.as_ptr(), core::ptr::null_mut(), &mut type_, (*credential).UserName as LPBYTE, &mut count)) as u32;
        if ret == ERROR_FILE_NOT_FOUND {
            (*credential).UserName = core::ptr::null_mut();
        } else if ret != ERROR_SUCCESS {
            return ret;
        } else if type_ != REG_SZ {
            return ERROR_REGISTRY_CORRUPT;
        } else {
            buffer = buffer.add(count as usize);
        }
    }

    ret = read_credential_blob(hkey, key_data, core::ptr::null_mut(), &mut count);
    if ret != ERROR_FILE_NOT_FOUND {
        if ret != ERROR_SUCCESS {
            return ret;
        }
        *len += count;
    }
    if !credential.is_null() {
        (*credential).CredentialBlob = buffer as LPBYTE;
        ret = read_credential_blob(hkey, key_data, (*credential).CredentialBlob, &mut count);
        if ret == ERROR_FILE_NOT_FOUND {
            (*credential).CredentialBlob = core::ptr::null_mut();
        } else if ret != ERROR_SUCCESS {
            return ret;
        }
        (*credential).CredentialBlobSize = count;
    }

    if !credential.is_null() {
        (*credential).AttributeCount = 0;
        (*credential).Attributes = core::ptr::null_mut();
    }

    if credential.is_null() {
        return ERROR_SUCCESS;
    }

    count = core::mem::size_of_val(&(*credential).Flags) as u32;
    ret = (RegQueryValueExW(hkey, wszFlagsValue.as_ptr(), core::ptr::null_mut(), &mut type_, core::ptr::addr_of_mut!((*credential).Flags) as LPBYTE, &mut count)) as u32;
    if ret != ERROR_SUCCESS {
        return ret;
    } else if type_ != REG_DWORD {
        return ERROR_REGISTRY_CORRUPT;
    }
    count = core::mem::size_of_val(&(*credential).Type) as u32;
    ret = (RegQueryValueExW(hkey, wszTypeValue.as_ptr(), core::ptr::null_mut(), &mut type_, core::ptr::addr_of_mut!((*credential).Type) as LPBYTE, &mut count)) as u32;
    if ret != ERROR_SUCCESS {
        return ret;
    } else if type_ != REG_DWORD {
        return ERROR_REGISTRY_CORRUPT;
    }

    count = core::mem::size_of_val(&(*credential).LastWritten) as u32;
    ret = (RegQueryValueExW(hkey, wszLastWrittenValue.as_ptr(), core::ptr::null_mut(), &mut type_, core::ptr::addr_of_mut!((*credential).LastWritten) as LPBYTE, &mut count)) as u32;
    if ret != ERROR_SUCCESS {
        return ret;
    } else if type_ != REG_BINARY {
        return ERROR_REGISTRY_CORRUPT;
    }
    count = core::mem::size_of_val(&(*credential).Persist) as u32;
    ret = (RegQueryValueExW(hkey, wszPersistValue.as_ptr(), core::ptr::null_mut(), &mut type_, core::ptr::addr_of_mut!((*credential).Persist) as LPBYTE, &mut count)) as u32;
    if ret == ERROR_SUCCESS && type_ != REG_DWORD {
        return ERROR_REGISTRY_CORRUPT;
    }
    ret
}

#[cfg(target_vendor = "apple")] // #ifdef __APPLE__ in wine/cred.c — not built on ReactOS
unsafe fn mac_read_credential_from_item(
    item: SecKeychainItemRef,
    require_password: BOOL,
    credential: PCREDENTIALW,
    mut buffer: *mut core::ffi::c_char,
    len: *mut DWORD,
) -> DWORD {
    let status: i32;
    let mut i: UInt32;
    let mut cred_blob_len: UInt32 = 0;
    let mut cred_blob: *mut core::ffi::c_void;
    let mut user: *mut WCHAR = core::ptr::null_mut();
    let mut user_name_present: BOOL = FALSE;
    let mut info: SecKeychainAttributeInfo = core::mem::zeroed();
    let mut attr_list: *mut SecKeychainAttributeList = core::ptr::null_mut();
    let mut info_tags: [UInt32; 4] = [
        kSecServiceItemAttr,
        kSecAccountItemAttr,
        kSecCommentItemAttr,
        kSecCreationDateItemAttr,
    ];
    info.count = (info_tags.len()) as UInt32;
    info.tag = info_tags.as_mut_ptr();
    info.format = core::ptr::null_mut();

    let mut status_local = SecKeychainItemCopyAttributesAndData(
        item,
        &mut info,
        core::ptr::null_mut(),
        &mut attr_list,
        &mut cred_blob_len,
        &mut cred_blob,
    );
    if status_local == errSecAuthFailed && require_password == FALSE {
        cred_blob_len = 0;
        cred_blob = core::ptr::null_mut();
        status_local = SecKeychainItemCopyAttributesAndData(
            item,
            &mut info,
            core::ptr::null_mut(),
            &mut attr_list,
            &mut cred_blob_len,
            core::ptr::null_mut(),
        );
    }
    status = status_local;
    if status != noErr {
        WARN!(b"SecKeychainItemCopyAttributesAndData returned status %d\n\0".as_ptr() as *const _, status);
        return ERROR_NOT_FOUND;
    }

    i = 0;
    while i < (*attr_list).count {
        let attr = (*attr_list).attr.offset(i as isize);
        if (*attr).tag == kSecAccountItemAttr && !(*attr).data.is_null() {
            user_name_present = TRUE;
            break;
        }
        i += 1;
    }
    if user_name_present == FALSE {
        WARN!(b"no kSecAccountItemAttr for item\n\0".as_ptr() as *const _);
        SecKeychainItemFreeAttributesAndData(attr_list, cred_blob);
        return ERROR_NOT_FOUND;
    }

    if !buffer.is_null() {
        (*credential).Flags = 0;
        (*credential).Type = CRED_TYPE_DOMAIN_PASSWORD;
        (*credential).TargetName = core::ptr::null_mut();
        (*credential).Comment = core::ptr::null_mut();
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!((*credential).LastWritten),
            0,
            1,
        );
        (*credential).CredentialBlobSize = 0;
        (*credential).CredentialBlob = core::ptr::null_mut();
        (*credential).Persist = CRED_PERSIST_LOCAL_MACHINE;
        (*credential).AttributeCount = 0;
        (*credential).Attributes = core::ptr::null_mut();
        (*credential).TargetAlias = core::ptr::null_mut();
        (*credential).UserName = core::ptr::null_mut();
    }

    i = 0;
    while i < (*attr_list).count {
        let attr = (*attr_list).attr.offset(i as isize);
        let tag = (*attr).tag;
        if tag == kSecServiceItemAttr {
            TRACE!(
                b"kSecServiceItemAttr: %.*s\n\0".as_ptr() as *const _,
                (*attr).length as i32,
                (*attr).data as *const core::ffi::c_char,
            );
            if (*attr).data.is_null() {
                i += 1;
                continue;
            }
            if !buffer.is_null() {
                (*credential).TargetName = buffer as LPWSTR;
                let str_len = MultiByteToWideChar(
                    CP_UTF8,
                    0,
                    (*attr).data as *const _,
                    (*attr).length as i32,
                    buffer as LPWSTR,
                    0xffff,
                );
                *(*credential).TargetName.offset(str_len as isize) = 0;
                buffer = buffer.offset(((str_len + 1) * core::mem::size_of::<WCHAR>() as i32) as isize);
                *len += ((str_len + 1) as DWORD) * core::mem::size_of::<WCHAR>() as DWORD;
            } else {
                let str_len = MultiByteToWideChar(
                    CP_UTF8,
                    0,
                    (*attr).data as *const _,
                    (*attr).length as i32,
                    core::ptr::null_mut(),
                    0,
                );
                *len += ((str_len + 1) as DWORD) * core::mem::size_of::<WCHAR>() as DWORD;
            }
        } else if tag == kSecAccountItemAttr {
            TRACE!(
                b"kSecAccountItemAttr: %.*s\n\0".as_ptr() as *const _,
                (*attr).length as i32,
                (*attr).data as *const core::ffi::c_char,
            );
            if (*attr).data.is_null() {
                i += 1;
                continue;
            }
            let str_len = MultiByteToWideChar(
                CP_UTF8,
                0,
                (*attr).data as *const _,
                (*attr).length as i32,
                core::ptr::null_mut(),
                0,
            );
            user = heap_alloc(((str_len + 1) as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
            MultiByteToWideChar(
                CP_UTF8,
                0,
                (*attr).data as *const _,
                (*attr).length as i32,
                user,
                str_len,
            );
            *user.offset(str_len as isize) = 0;
        } else if tag == kSecCommentItemAttr {
            TRACE!(
                b"kSecCommentItemAttr: %.*s\n\0".as_ptr() as *const _,
                (*attr).length as i32,
                (*attr).data as *const core::ffi::c_char,
            );
            if (*attr).data.is_null() {
                i += 1;
                continue;
            }
            if !buffer.is_null() {
                (*credential).Comment = buffer as LPWSTR;
                let str_len = MultiByteToWideChar(
                    CP_UTF8,
                    0,
                    (*attr).data as *const _,
                    (*attr).length as i32,
                    buffer as LPWSTR,
                    0xffff,
                );
                *(*credential).Comment.offset(str_len as isize) = 0;
                buffer = buffer.offset(((str_len + 1) * core::mem::size_of::<WCHAR>() as i32) as isize);
                *len += ((str_len + 1) as DWORD) * core::mem::size_of::<WCHAR>() as DWORD;
            } else {
                let str_len = MultiByteToWideChar(
                    CP_UTF8,
                    0,
                    (*attr).data as *const _,
                    (*attr).length as i32,
                    core::ptr::null_mut(),
                    0,
                );
                *len += ((str_len + 1) as DWORD) * core::mem::size_of::<WCHAR>() as DWORD;
            }
        } else if tag == kSecCreationDateItemAttr {
            TRACE!(
                b"kSecCreationDateItemAttr: %.*s\n\0".as_ptr() as *const _,
                (*attr).length as i32,
                (*attr).data as *const core::ffi::c_char,
            );
            if (*attr).data.is_null() {
                i += 1;
                continue;
            }
            if !buffer.is_null() {
                let mut win_time: LARGE_INTEGER = core::mem::zeroed();
                let mut tm: tm = core::mem::zeroed();
                let time: time_t;
                core::ptr::write_bytes(&mut tm as *mut tm, 0, 1);
                strptime(
                    (*attr).data as *const core::ffi::c_char,
                    b"%Y%m%d%H%M%SZ\0".as_ptr() as *const _,
                    &mut tm,
                );
                time = mktime(&mut tm);
                RtlSecondsSince1970ToTime(time as u32, &mut win_time as *mut _ as *mut _);
                (*credential).LastWritten.dwLowDateTime = win_time.u.__bindgen_anon_1.LowPart;
                (*credential).LastWritten.dwHighDateTime = win_time.u.__bindgen_anon_1.HighPart as u32;
            }
        } else {
            FIXME!("unhandled attribute %u\n", tag as u32);
        }
        i += 1;
    }

    if !user.is_null() {
        if !buffer.is_null() {
            (*credential).UserName = buffer as LPWSTR;
        }
        let str_len = strlenW(user) as i32;
        *len += ((str_len + 1) as DWORD) * core::mem::size_of::<WCHAR>() as DWORD;
        if !buffer.is_null() {
            memcpy(
                buffer as *mut _,
                user as *const _,
                ((str_len + 1) as usize) * core::mem::size_of::<WCHAR>(),
            );
            buffer = buffer.offset(((str_len + 1) * core::mem::size_of::<WCHAR>() as i32) as isize);
            TRACE!(b"UserName = %s\n\0".as_ptr() as *const _, debugstr_w((*credential).UserName));
        }
    }
    heap_free(user as *mut _);

    if !cred_blob.is_null() {
        if !buffer.is_null() {
            (*credential).CredentialBlob = buffer as LPBYTE;
            let str_len = MultiByteToWideChar(
                CP_UTF8,
                0,
                cred_blob as *const _,
                cred_blob_len as i32,
                buffer as LPWSTR,
                0xffff,
            );
            (*credential).CredentialBlobSize = (str_len as DWORD) * core::mem::size_of::<WCHAR>() as DWORD;
            *len += (str_len as DWORD) * core::mem::size_of::<WCHAR>() as DWORD;
        } else {
            let str_len = MultiByteToWideChar(
                CP_UTF8,
                0,
                cred_blob as *const _,
                cred_blob_len as i32,
                core::ptr::null_mut(),
                0,
            );
            *len += (str_len as DWORD) * core::mem::size_of::<WCHAR>() as DWORD;
        }
    }
    SecKeychainItemFreeAttributesAndData(attr_list, cred_blob);
    ERROR_SUCCESS
}

unsafe fn write_credential_blob(
    hkey: HKEY,
    target_name: LPCWSTR,
    type_: DWORD,
    key_data: *const BYTE,
    credential_blob: *const BYTE,
    credential_blob_size: DWORD,
) -> DWORD {
    let mut data: ustring = core::mem::zeroed();
    let mut key: ustring = core::mem::zeroed();

    key.Length = KEY_SIZE as DWORD;
    key.MaximumLength = KEY_SIZE as DWORD;
    key.Buffer = key_data as *mut core::ffi::c_uchar;

    let encrypted_credential_blob = heap_alloc(credential_blob_size as usize) as LPBYTE;
    if encrypted_credential_blob.is_null() {
        return ERROR_OUTOFMEMORY;
    }

    memcpy(
        encrypted_credential_blob as *mut core::ffi::c_void,
        credential_blob as *const core::ffi::c_void,
        credential_blob_size as usize,
    );
    data.Length = credential_blob_size;
    data.MaximumLength = credential_blob_size;
    data.Buffer = encrypted_credential_blob;
    SystemFunction032(
        core::ptr::addr_of_mut!(data),
        core::ptr::addr_of_mut!(key),
    );

    let ret = RegSetValueExW(
        hkey,
        wszPasswordValue.as_ptr(),
        0,
        REG_BINARY,
        encrypted_credential_blob,
        credential_blob_size,
    );
    heap_free(encrypted_credential_blob as *mut core::ffi::c_void);

    (ret) as u32
}

unsafe fn registry_write_credential(
    hkey: HKEY,
    credential: *const CREDENTIALW,
    key_data: *const BYTE,
    preserve_blob: BOOL,
) -> DWORD {
    let mut ret: DWORD;
    let mut LastWritten: FILETIME = core::mem::zeroed();

    GetSystemTimeAsFileTime(&mut LastWritten);

    ret = (RegSetValueExW(
        hkey,
        wszFlagsValue.as_ptr(),
        0,
        REG_DWORD,
        core::ptr::addr_of!((*credential).Flags) as *const BYTE,
        core::mem::size_of_val(&(*credential).Flags) as DWORD,
    )) as u32;
    if ret != ERROR_SUCCESS {
        return ret;
    }
    ret = (RegSetValueExW(
        hkey,
        wszTypeValue.as_ptr(),
        0,
        REG_DWORD,
        core::ptr::addr_of!((*credential).Type) as *const BYTE,
        core::mem::size_of_val(&(*credential).Type) as DWORD,
    )) as u32;
    if ret != ERROR_SUCCESS {
        return ret;
    }
    ret = (RegSetValueExW(
        hkey,
        core::ptr::null(),
        0,
        REG_SZ,
        (*credential).TargetName as LPVOID as *const BYTE,
        (core::mem::size_of::<WCHAR>() as DWORD) * ((strlenW((*credential).TargetName) + 1) as DWORD),
    )) as u32;
    if ret != ERROR_SUCCESS {
        return ret;
    }
    if !(*credential).Comment.is_null() {
        ret = (RegSetValueExW(
            hkey,
            wszCommentValue.as_ptr(),
            0,
            REG_SZ,
            (*credential).Comment as LPVOID as *const BYTE,
            (core::mem::size_of::<WCHAR>() as DWORD) * ((strlenW((*credential).Comment) + 1) as DWORD),
        )) as u32;
        if ret != ERROR_SUCCESS {
            return ret;
        }
    }
    ret = (RegSetValueExW(
        hkey,
        wszLastWrittenValue.as_ptr(),
        0,
        REG_BINARY,
        core::ptr::addr_of!(LastWritten) as LPVOID as *const BYTE,
        core::mem::size_of_val(&LastWritten) as DWORD,
    )) as u32;
    if ret != ERROR_SUCCESS {
        return ret;
    }
    ret = (RegSetValueExW(
        hkey,
        wszPersistValue.as_ptr(),
        0,
        REG_DWORD,
        core::ptr::addr_of!((*credential).Persist) as *const BYTE,
        core::mem::size_of_val(&(*credential).Persist) as DWORD,
    )) as u32;
    if ret != ERROR_SUCCESS {
        return ret;
    }
    if !(*credential).TargetAlias.is_null() {
        ret = (RegSetValueExW(
            hkey,
            wszTargetAliasValue.as_ptr(),
            0,
            REG_SZ,
            (*credential).TargetAlias as LPVOID as *const BYTE,
            (core::mem::size_of::<WCHAR>() as DWORD) * ((strlenW((*credential).TargetAlias) + 1) as DWORD),
        )) as u32;
        if ret != ERROR_SUCCESS {
            return ret;
        }
    }
    if !(*credential).UserName.is_null() {
        ret = (RegSetValueExW(
            hkey,
            wszUserNameValue.as_ptr(),
            0,
            REG_SZ,
            (*credential).UserName as LPVOID as *const BYTE,
            (core::mem::size_of::<WCHAR>() as DWORD) * ((strlenW((*credential).UserName) + 1) as DWORD),
        )) as u32;
        if ret != ERROR_SUCCESS {
            return ret;
        }
    }
    if preserve_blob == 0 {
        ret = write_credential_blob(
            hkey,
            (*credential).TargetName,
            (*credential).Type,
            key_data,
            (*credential).CredentialBlob,
            (*credential).CredentialBlobSize,
        );
    }
    ret
}

#[cfg(target_vendor = "apple")] // #ifdef __APPLE__ in wine/cred.c — not built on ReactOS
unsafe fn mac_write_credential(credential: *const CREDENTIALW, preserve_blob: BOOL) -> DWORD {
    let status: i32;
    let mut keychain_item: SecKeychainItemRef = core::ptr::null_mut();
    let username: *mut core::ffi::c_char;
    let password: *mut core::ffi::c_char;
    let servername: *mut core::ffi::c_char;
    let userlen: UInt32;
    let pwlen: UInt32;
    let serverlen: UInt32;
    let mut attrs: [SecKeychainAttribute; 1] = core::mem::zeroed();
    let mut attr_list: SecKeychainAttributeList = core::mem::zeroed();

    if (*credential).Flags != 0 {
        FIXME!("Flags 0x%x not written\n", (*credential).Flags);
    }
    if (*credential).Type != CRED_TYPE_DOMAIN_PASSWORD {
        FIXME!("credential type of %d not supported\n", (*credential).Type);
    }
    if (*credential).Persist != CRED_PERSIST_LOCAL_MACHINE {
        FIXME!("persist value of %d not supported\n", (*credential).Persist);
    }
    if (*credential).AttributeCount != 0 {
        FIXME!("custom attributes not supported\n");
    }

    userlen = WideCharToMultiByte(CP_UTF8, 0, (*credential).UserName, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
    username = heap_alloc((userlen as usize) * core::mem::size_of::<core::ffi::c_char>()) as *mut core::ffi::c_char;
    WideCharToMultiByte(CP_UTF8, 0, (*credential).UserName, -1, username, userlen, core::ptr::null_mut(), core::ptr::null_mut());

    serverlen = WideCharToMultiByte(CP_UTF8, 0, (*credential).TargetName, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
    servername = heap_alloc((serverlen as usize) * core::mem::size_of::<core::ffi::c_char>()) as *mut core::ffi::c_char;
    WideCharToMultiByte(CP_UTF8, 0, (*credential).TargetName, -1, servername, serverlen, core::ptr::null_mut(), core::ptr::null_mut());

    pwlen = WideCharToMultiByte(CP_UTF8, 0, (*credential).CredentialBlob as LPCWSTR,
                                ((*credential).CredentialBlobSize as usize / core::mem::size_of::<u16>()) as i32,
                                core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
    password = heap_alloc((pwlen as usize) * core::mem::size_of::<core::ffi::c_char>()) as *mut core::ffi::c_char;
    WideCharToMultiByte(CP_UTF8, 0, (*credential).CredentialBlob as LPCWSTR,
                        ((*credential).CredentialBlobSize as usize / core::mem::size_of::<u16>()) as i32,
                        password, pwlen, core::ptr::null_mut(), core::ptr::null_mut());

    TRACE!(b"adding server %s, username %s using Keychain\n\0".as_ptr() as *const core::ffi::c_char, servername, username);
    status = SecKeychainAddGenericPassword(core::ptr::null_mut(), strlen(servername), servername, strlen(username),
                                           username, strlen(password), password, core::ptr::addr_of_mut!(keychain_item));
    if status != noErr {
        ERR!("SecKeychainAddGenericPassword returned %d\n", status);
    }
    let mut status = status;
    if status == errSecDuplicateItem {
        status = SecKeychainFindGenericPassword(core::ptr::null_mut(), strlen(servername), servername, strlen(username),
                                                username, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(keychain_item));
        if status != noErr {
            ERR!("SecKeychainFindGenericPassword returned %d\n", status);
        }
    }
    heap_free(username as *mut core::ffi::c_void);
    heap_free(servername as *mut core::ffi::c_void);
    if status != noErr {
        heap_free(password as *mut core::ffi::c_void);
        return ERROR_GEN_FAILURE;
    }
    if !(*credential).Comment.is_null() {
        attr_list.count = 1;
        attr_list.attr = attrs.as_mut_ptr();
        attrs[0].tag = kSecCommentItemAttr;
        attrs[0].length = WideCharToMultiByte(CP_UTF8, 0, (*credential).Comment, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as UInt32;
        if attrs[0].length != 0 {
            attrs[0].length -= 1;
        }
        attrs[0].data = heap_alloc(attrs[0].length as usize);
        WideCharToMultiByte(CP_UTF8, 0, (*credential).Comment, -1, attrs[0].data as *mut core::ffi::c_char, attrs[0].length as i32, core::ptr::null_mut(), core::ptr::null_mut());
    } else {
        attr_list.count = 0;
        attr_list.attr = core::ptr::null_mut();
    }
    status = SecKeychainItemModifyAttributesAndData(keychain_item, core::ptr::addr_of_mut!(attr_list),
                                                    if preserve_blob != 0 { 0 } else { strlen(password) },
                                                    if preserve_blob != 0 { core::ptr::null_mut() } else { password as *mut core::ffi::c_void });
    if !(*credential).Comment.is_null() {
        heap_free(attrs[0].data);
    }
    heap_free(password as *mut core::ffi::c_void);
    CFRelease(keychain_item);
    if status != noErr {
        return ERROR_GEN_FAILURE;
    }
    ERROR_SUCCESS
}

unsafe fn open_cred_mgr_key(hkey: *mut HKEY, open_for_write: BOOL) -> DWORD {
    (RegCreateKeyExW(
        HKEY_CURRENT_USER,
        wszCredentialManagerKey.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_READ | (if open_for_write != 0 { KEY_WRITE } else { 0 }),
        core::ptr::null_mut(),
        hkey,
        core::ptr::null_mut(),
    )) as u32
}

unsafe fn get_cred_mgr_encryption_key(mut hkeyMgr: HKEY, key_data: *mut u8) -> DWORD {
    static my_key_data: [u8; 8] = [0; 8];
    let mut r#type: DWORD = 0;
    let mut count: DWORD;
    let mut ft: FILETIME = core::mem::zeroed();
    let mut seed: ULONG;
    let mut value: ULONG;
    let mut ret: DWORD;

    memcpy(key_data as *mut _, my_key_data.as_ptr() as *const _, 8);

    count = 8;
    ret = (RegQueryValueExW(hkeyMgr, wszEncryptionKeyValue.as_ptr(), core::ptr::null_mut(),
                           &mut r#type, key_data, &mut count)) as u32;
    if ret == ERROR_SUCCESS {
        if r#type != REG_BINARY {
            return ERROR_REGISTRY_CORRUPT;
        } else {
            return ERROR_SUCCESS;
        }
    }
    if ret != ERROR_FILE_NOT_FOUND {
        return ret;
    }

    GetSystemTimeAsFileTime(&mut ft);
    seed = ft.dwLowDateTime;
    value = RtlUniform(&mut seed);
    *(key_data as *mut DWORD) = value;
    seed = ft.dwHighDateTime;
    value = RtlUniform(&mut seed);
    *(key_data.add(4) as *mut DWORD) = value;

    ret = (RegSetValueExW(hkeyMgr, wszEncryptionKeyValue.as_ptr(), 0, REG_BINARY,
                         key_data, 8)) as u32;
    if ret == ERROR_ACCESS_DENIED {
        ret = open_cred_mgr_key(&mut hkeyMgr, 1);
        if ret == ERROR_SUCCESS {
            ret = (RegSetValueExW(hkeyMgr, wszEncryptionKeyValue.as_ptr(), 0, REG_BINARY,
                                 key_data, 8)) as u32;
            RegCloseKey(hkeyMgr);
        }
    }
    ret
}

unsafe fn get_key_name_for_target(target_name: LPCWSTR, type_: DWORD) -> LPWSTR {
    static wszGenericPrefix: [WCHAR; 10] = [
        'G' as WCHAR, 'e' as WCHAR, 'n' as WCHAR, 'e' as WCHAR, 'r' as WCHAR,
        'i' as WCHAR, 'c' as WCHAR, ':' as WCHAR, ' ' as WCHAR, 0,
    ];
    static wszDomPasswdPrefix: [WCHAR; 12] = [
        'D' as WCHAR, 'o' as WCHAR, 'm' as WCHAR, 'P' as WCHAR, 'a' as WCHAR,
        's' as WCHAR, 's' as WCHAR, 'w' as WCHAR, 'd' as WCHAR, ':' as WCHAR,
        ' ' as WCHAR, 0,
    ];

    let mut len: INT;
    let prefix: LPCWSTR;

    len = strlenW(target_name) as INT;
    if type_ == CRED_TYPE_GENERIC {
        prefix = wszGenericPrefix.as_ptr();
        len += (wszGenericPrefix.len()) as INT;
    } else {
        prefix = wszDomPasswdPrefix.as_ptr();
        len += (wszDomPasswdPrefix.len()) as INT;
    }

    let key_name: LPWSTR = heap_alloc((len as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
    if key_name.is_null() {
        return core::ptr::null_mut();
    }

    strcpyW(key_name, prefix);
    strcatW(key_name, target_name);

    let mut p: LPWSTR = key_name;
    while *p != 0 {
        if *p == '\\' as WCHAR {
            *p = '_' as WCHAR;
        }
        p = p.add(1);
    }

    key_name
}

unsafe fn registry_credential_matches_filter(hkeyCred: HKEY, filter: LPCWSTR) -> BOOL {
    let mut target_name: LPWSTR;
    let mut ret: DWORD;
    let mut type_: DWORD = 0;
    let mut count: DWORD = 0;
    let p: LPCWSTR;

    if filter.is_null() {
        return 1;
    }

    ret = RegQueryValueExW(hkeyCred, core::ptr::null_mut(), core::ptr::null_mut(), &mut type_, core::ptr::null_mut(), &mut count) as DWORD;
    if ret != ERROR_SUCCESS {
        return 0;
    } else if type_ != REG_SZ {
        return 0;
    }

    target_name = heap_alloc(count as usize) as LPWSTR;
    if target_name.is_null() {
        return 0;
    }
    ret = RegQueryValueExW(hkeyCred, core::ptr::null_mut(), core::ptr::null_mut(), &mut type_, target_name as LPBYTE, &mut count) as DWORD;
    if ret != ERROR_SUCCESS || type_ != REG_SZ {
        heap_free(target_name as LPVOID);
        return 0;
    }

    TRACE!(b"comparing filter %s to target name %s\n\0".as_ptr() as *const i8,
          debugstr_w(filter), debugstr_w(target_name));

    p = strchrW(filter, '*' as WCHAR);
    let cmp_len: core::ffi::c_int = if !p.is_null() && *p.offset(1) == 0 {
        p.offset_from(filter) as core::ffi::c_int
    } else {
        -1
    };
    ret = (CompareStringW(GetThreadLocale(), NORM_IGNORECASE, filter, cmp_len,
                          target_name, cmp_len) == CSTR_EQUAL) as DWORD;

    heap_free(target_name as LPVOID);
    ret as BOOL
}

unsafe fn registry_enumerate_credentials(
    hkeyMgr: HKEY,
    filter: LPCWSTR,
    target_name: LPWSTR,
    target_name_len: u32,
    key_data: *const u8,
    credentials: *mut PCREDENTIALW,
    buffer: *mut *mut i8,
    len: *mut u32,
    count: *mut u32,
) -> u32 {
    let mut i: u32 = 0;
    let mut ret: u32;
    loop {
        let mut hkeyCred: HKEY = core::ptr::null_mut();
        ret = (RegEnumKeyW(hkeyMgr, i, target_name, target_name_len + 1)) as u32;
        if ret == ERROR_NO_MORE_ITEMS {
            ret = ERROR_SUCCESS;
            break;
        } else if ret != ERROR_SUCCESS {
            i = i.wrapping_add(1);
            continue;
        }
        TRACE!(b"target_name = %s\n\0".as_ptr() as *const i8, debugstr_w(target_name));
        ret = (RegOpenKeyExW(
            hkeyMgr,
            target_name,
            0,
            KEY_QUERY_VALUE,
            &mut hkeyCred,
        )) as u32;
        if ret != ERROR_SUCCESS {
            i = i.wrapping_add(1);
            continue;
        }
        if registry_credential_matches_filter(hkeyCred, filter) == 0 {
            RegCloseKey(hkeyCred);
            i = i.wrapping_add(1);
            continue;
        }
        if !buffer.is_null() {
            *len = core::mem::size_of::<CREDENTIALW>() as u32;
            *credentials.offset(*count as isize) = *buffer as PCREDENTIALW;
        } else {
            *len += core::mem::size_of::<CREDENTIALW>() as u32;
        }
        ret = registry_read_credential(
            hkeyCred,
            if !buffer.is_null() {
                *credentials.offset(*count as isize)
            } else {
                core::ptr::null_mut()
            },
            key_data,
            if !buffer.is_null() {
                (*buffer).add(core::mem::size_of::<CREDENTIALW>())
            } else {
                core::ptr::null_mut()
            },
            len,
        );
        RegCloseKey(hkeyCred);
        if ret != ERROR_SUCCESS {
            break;
        }
        if !buffer.is_null() {
            *buffer = (*buffer).add(*len as usize);
        }
        *count += 1;
        i = i.wrapping_add(1);
    }
    ret
}

#[cfg(target_vendor = "apple")] // #ifdef __APPLE__ in wine/cred.c — not built on ReactOS
unsafe fn mac_credential_matches_filter(data: *mut core::ffi::c_void, data_len: UInt32, filter: *const WCHAR) -> BOOL {
    let len: core::ffi::c_int;
    let target_name: *mut WCHAR;
    let p: *const WCHAR;
    let ret: BOOL;

    if filter.is_null() {
        return TRUE;
    }

    len = MultiByteToWideChar(CP_UTF8, 0, data as LPCSTR, data_len as i32, core::ptr::null_mut(), 0);
    target_name = heap_alloc(((len + 1) as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
    if target_name.is_null() {
        return FALSE;
    }
    MultiByteToWideChar(CP_UTF8, 0, data as LPCSTR, data_len as i32, target_name, len);
    *target_name.offset(len as isize) = 0;

    p = strchrW(filter, '*' as WCHAR);

    let cmp_len: core::ffi::c_int = if !p.is_null() && *p.offset(1) == 0 {
        p.offset_from(filter) as core::ffi::c_int
    } else {
        -1
    };

    ret = (CompareStringW(GetThreadLocale(), NORM_IGNORECASE, filter, cmp_len, target_name, cmp_len) == CSTR_EQUAL) as BOOL;
    heap_free(target_name as *mut core::ffi::c_void);
    ret
}

#[cfg(target_vendor = "apple")] // #ifdef __APPLE__ in wine/cred.c — not built on ReactOS
unsafe fn mac_enumerate_credentials(
    filter: LPCWSTR,
    credentials: *mut PCREDENTIALW,
    mut buffer: *mut core::ffi::c_char,
    len: *mut DWORD,
    count: *mut DWORD,
) -> DWORD {
    let mut search: SecKeychainSearchRef = core::ptr::null_mut();
    let mut item: SecKeychainItemRef = core::ptr::null_mut();
    let mut status: core::ffi::c_int;
    let mut saved_user_interaction_allowed: Boolean = 0;

    SecKeychainGetUserInteractionAllowed(&mut saved_user_interaction_allowed);
    SecKeychainSetUserInteractionAllowed(false as Boolean);

    status = SecKeychainSearchCreateFromAttributes(
        core::ptr::null_mut(),
        kSecGenericPasswordItemClass,
        core::ptr::null_mut(),
        &mut search,
    );
    if status == noErr {
        while SecKeychainSearchCopyNext(search, &mut item) == noErr {
            let mut info: SecKeychainAttributeInfo = core::mem::zeroed();
            let mut attr_list: *mut SecKeychainAttributeList = core::ptr::null_mut();
            let mut info_tags: [UInt32; 1] = [kSecServiceItemAttr];

            info.count = (info_tags.len()) as UInt32;
            info.tag = info_tags.as_mut_ptr();
            info.format = core::ptr::null_mut();
            status = SecKeychainItemCopyAttributesAndData(
                item,
                &mut info,
                core::ptr::null_mut(),
                &mut attr_list,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if status != noErr {
                continue;
            }
            if !buffer.is_null() {
                *len = core::mem::size_of::<CREDENTIALW>() as DWORD;
                *credentials.offset(*count as isize) = buffer as PCREDENTIALW;
            } else {
                *len += core::mem::size_of::<CREDENTIALW>() as DWORD;
            }
            if (*attr_list).count != 1
                || (*(*attr_list).attr.offset(0)).tag != kSecServiceItemAttr
            {
                SecKeychainItemFreeAttributesAndData(attr_list, core::ptr::null_mut());
                continue;
            }
            let match_ = mac_credential_matches_filter(
                (*(*attr_list).attr.offset(0)).data,
                (*(*attr_list).attr.offset(0)).length,
                filter,
            );
            SecKeychainItemFreeAttributesAndData(attr_list, core::ptr::null_mut());
            if match_ == 0 {
                continue;
            }
            let ret = mac_read_credential_from_item(
                item,
                0,
                if !buffer.is_null() {
                    *credentials.offset(*count as isize)
                } else {
                    core::ptr::null_mut()
                },
                if !buffer.is_null() {
                    buffer.add(core::mem::size_of::<CREDENTIALW>())
                } else {
                    core::ptr::null_mut()
                },
                len,
            );
            CFRelease(item as *const core::ffi::c_void);
            if ret == ERROR_SUCCESS {
                *count += 1;
                if !buffer.is_null() {
                    buffer = buffer.add(*len as usize);
                }
            }
        }
        CFRelease(search as *const core::ffi::c_void);
    }
    SecKeychainSetUserInteractionAllowed(saved_user_interaction_allowed);
    ERROR_SUCCESS
}

#[cfg(target_vendor = "apple")] // #ifdef __APPLE__ in wine/cred.c — not built on ReactOS
unsafe fn mac_delete_credential(TargetName: LPCWSTR) -> DWORD {
    let mut status: i32;
    let mut search: SecKeychainSearchRef = core::ptr::null_mut();
    status = SecKeychainSearchCreateFromAttributes(
        core::ptr::null_mut(),
        kSecGenericPasswordItemClass,
        core::ptr::null_mut(),
        &mut search,
    );
    if status == noErr {
        let mut item: SecKeychainItemRef = core::ptr::null_mut();
        while SecKeychainSearchCopyNext(search, &mut item) == noErr {
            let mut info: SecKeychainAttributeInfo = core::mem::zeroed();
            let mut attr_list: *mut SecKeychainAttributeList = core::ptr::null_mut();
            let mut info_tags: [UInt32; 1] = [kSecServiceItemAttr];
            info.count = (core::mem::size_of::<[UInt32; 1]>() / core::mem::size_of::<UInt32>()) as UInt32;
            info.tag = info_tags.as_mut_ptr();
            info.format = core::ptr::null_mut();
            status = SecKeychainItemCopyAttributesAndData(
                item,
                &mut info,
                core::ptr::null_mut(),
                &mut attr_list,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if status != noErr {
                continue;
            }
            if (*attr_list).count != 1
                || (*(*attr_list).attr.offset(0)).tag != kSecServiceItemAttr
            {
                CFRelease(item as _);
                continue;
            }
            let str_len: INT = MultiByteToWideChar(
                CP_UTF8,
                0,
                (*(*attr_list).attr.offset(0)).data as _,
                (*(*attr_list).attr.offset(0)).length as i32,
                core::ptr::null_mut(),
                0,
            );
            let target_name: LPWSTR =
                heap_alloc(((str_len + 1) as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
            MultiByteToWideChar(
                CP_UTF8,
                0,
                (*(*attr_list).attr.offset(0)).data as _,
                (*(*attr_list).attr.offset(0)).length as i32,
                target_name,
                str_len,
            );
            *target_name.offset(str_len as isize) = 0;
            if strcmpiW(TargetName, target_name) != 0 {
                CFRelease(item as _);
                heap_free(target_name as _);
                continue;
            }
            heap_free(target_name as _);
            SecKeychainItemFreeAttributesAndData(attr_list, core::ptr::null_mut());
            SecKeychainItemDelete(item);
            CFRelease(item as _);
            CFRelease(search as _);

            return ERROR_SUCCESS;
        }
        CFRelease(search as _);
    }
    ERROR_NOT_FOUND
}

unsafe fn convert_PCREDENTIALW_to_PCREDENTIALA(CredentialW: *const CREDENTIALW, CredentialA: PCREDENTIALA, mut len: DWORD) -> INT {
    let mut buffer: *mut core::ffi::c_char;
    let mut string_len: INT;
    let mut needed: INT = core::mem::size_of::<CREDENTIALA>() as INT;

    if CredentialA.is_null() {
        if !(*CredentialW).TargetName.is_null() {
            needed += WideCharToMultiByte(CP_ACP, 0, (*CredentialW).TargetName, -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut());
        }
        if !(*CredentialW).Comment.is_null() {
            needed += WideCharToMultiByte(CP_ACP, 0, (*CredentialW).Comment, -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut());
        }
        needed += (*CredentialW).CredentialBlobSize as INT;
        if !(*CredentialW).TargetAlias.is_null() {
            needed += WideCharToMultiByte(CP_ACP, 0, (*CredentialW).TargetAlias, -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut());
        }
        if !(*CredentialW).UserName.is_null() {
            needed += WideCharToMultiByte(CP_ACP, 0, (*CredentialW).UserName, -1, core::ptr::null_mut(), 0, core::ptr::null(), core::ptr::null_mut());
        }

        return needed;
    }

    buffer = (CredentialA as *mut core::ffi::c_char).add(core::mem::size_of::<CREDENTIALA>());
    len -= core::mem::size_of::<CREDENTIALA>() as DWORD;
    (*CredentialA).Flags = (*CredentialW).Flags;
    (*CredentialA).Type = (*CredentialW).Type;

    if !(*CredentialW).TargetName.is_null() {
        (*CredentialA).TargetName = buffer as LPSTR;
        string_len = WideCharToMultiByte(CP_ACP, 0, (*CredentialW).TargetName, -1, buffer as LPSTR, len as i32, core::ptr::null(), core::ptr::null_mut());
        buffer = buffer.add(string_len as usize);
        needed += string_len;
        len -= string_len as DWORD;
    } else {
        (*CredentialA).TargetName = core::ptr::null_mut();
    }
    if !(*CredentialW).Comment.is_null() {
        (*CredentialA).Comment = buffer as LPSTR;
        string_len = WideCharToMultiByte(CP_ACP, 0, (*CredentialW).Comment, -1, buffer as LPSTR, len as i32, core::ptr::null(), core::ptr::null_mut());
        buffer = buffer.add(string_len as usize);
        needed += string_len;
        len -= string_len as DWORD;
    } else {
        (*CredentialA).Comment = core::ptr::null_mut();
    }
    (*CredentialA).LastWritten = (*CredentialW).LastWritten;
    (*CredentialA).CredentialBlobSize = (*CredentialW).CredentialBlobSize;
    if (*CredentialW).CredentialBlobSize != 0 && (*CredentialW).CredentialBlobSize <= len {
        (*CredentialA).CredentialBlob = buffer as LPBYTE;
        memcpy((*CredentialA).CredentialBlob as *mut core::ffi::c_void, (*CredentialW).CredentialBlob as *const core::ffi::c_void, (*CredentialW).CredentialBlobSize as usize);
        buffer = buffer.add((*CredentialW).CredentialBlobSize as usize);
        needed += (*CredentialW).CredentialBlobSize as INT;
        len -= (*CredentialW).CredentialBlobSize;
    } else {
        (*CredentialA).CredentialBlob = core::ptr::null_mut();
    }
    (*CredentialA).Persist = (*CredentialW).Persist;
    (*CredentialA).AttributeCount = 0;
    (*CredentialA).Attributes = core::ptr::null_mut();
    if !(*CredentialW).TargetAlias.is_null() {
        (*CredentialA).TargetAlias = buffer as LPSTR;
        string_len = WideCharToMultiByte(CP_ACP, 0, (*CredentialW).TargetAlias, -1, buffer as LPSTR, len as i32, core::ptr::null(), core::ptr::null_mut());
        buffer = buffer.add(string_len as usize);
        needed += string_len;
        len -= string_len as DWORD;
    } else {
        (*CredentialA).TargetAlias = core::ptr::null_mut();
    }
    if !(*CredentialW).UserName.is_null() {
        (*CredentialA).UserName = buffer as LPSTR;
        string_len = WideCharToMultiByte(CP_ACP, 0, (*CredentialW).UserName, -1, buffer as LPSTR, len as i32, core::ptr::null(), core::ptr::null_mut());
        needed += string_len;
    } else {
        (*CredentialA).UserName = core::ptr::null_mut();
    }

    needed
}

unsafe fn convert_PCREDENTIALA_to_PCREDENTIALW(CredentialA: *const CREDENTIALA, CredentialW: PCREDENTIALW, mut len: INT) -> INT {
    let mut buffer: *mut core::ffi::c_char;
    let mut string_len: INT;
    let mut needed: INT = core::mem::size_of::<CREDENTIALW>() as INT;

    if CredentialW.is_null() {
        if !(*CredentialA).TargetName.is_null() {
            needed += core::mem::size_of::<WCHAR>() as INT
                * MultiByteToWideChar(CP_ACP, 0, (*CredentialA).TargetName, -1, core::ptr::null_mut(), 0);
        }
        if !(*CredentialA).Comment.is_null() {
            needed += core::mem::size_of::<WCHAR>() as INT
                * MultiByteToWideChar(CP_ACP, 0, (*CredentialA).Comment, -1, core::ptr::null_mut(), 0);
        }
        needed += (*CredentialA).CredentialBlobSize as INT;
        if !(*CredentialA).TargetAlias.is_null() {
            needed += core::mem::size_of::<WCHAR>() as INT
                * MultiByteToWideChar(CP_ACP, 0, (*CredentialA).TargetAlias, -1, core::ptr::null_mut(), 0);
        }
        if !(*CredentialA).UserName.is_null() {
            needed += core::mem::size_of::<WCHAR>() as INT
                * MultiByteToWideChar(CP_ACP, 0, (*CredentialA).UserName, -1, core::ptr::null_mut(), 0);
        }

        return needed;
    }

    buffer = (CredentialW as *mut core::ffi::c_char).add(core::mem::size_of::<CREDENTIALW>());
    len -= core::mem::size_of::<CREDENTIALW>() as INT;
    (*CredentialW).Flags = (*CredentialA).Flags;
    (*CredentialW).Type = (*CredentialA).Type;
    if !(*CredentialA).TargetName.is_null() {
        (*CredentialW).TargetName = buffer as LPWSTR;
        string_len = MultiByteToWideChar(CP_ACP, 0, (*CredentialA).TargetName, -1, (*CredentialW).TargetName, len / core::mem::size_of::<WCHAR>() as INT);
        buffer = buffer.add(core::mem::size_of::<WCHAR>() * string_len as usize);
        needed += core::mem::size_of::<WCHAR>() as INT * string_len;
        len -= core::mem::size_of::<WCHAR>() as INT * string_len;
    } else {
        (*CredentialW).TargetName = core::ptr::null_mut();
    }
    if !(*CredentialA).Comment.is_null() {
        (*CredentialW).Comment = buffer as LPWSTR;
        string_len = MultiByteToWideChar(CP_ACP, 0, (*CredentialA).Comment, -1, (*CredentialW).Comment, len / core::mem::size_of::<WCHAR>() as INT);
        buffer = buffer.add(core::mem::size_of::<WCHAR>() * string_len as usize);
        needed += core::mem::size_of::<WCHAR>() as INT * string_len;
        len -= core::mem::size_of::<WCHAR>() as INT * string_len;
    } else {
        (*CredentialW).Comment = core::ptr::null_mut();
    }
    (*CredentialW).LastWritten = (*CredentialA).LastWritten;
    (*CredentialW).CredentialBlobSize = (*CredentialA).CredentialBlobSize;
    if (*CredentialA).CredentialBlobSize != 0 {
        (*CredentialW).CredentialBlob = buffer as LPBYTE;
        memcpy((*CredentialW).CredentialBlob as *mut core::ffi::c_void, (*CredentialA).CredentialBlob as *const core::ffi::c_void, (*CredentialA).CredentialBlobSize as usize);
        buffer = buffer.add((*CredentialA).CredentialBlobSize as usize);
        needed += (*CredentialA).CredentialBlobSize as INT;
        len -= (*CredentialA).CredentialBlobSize as INT;
    } else {
        (*CredentialW).CredentialBlob = core::ptr::null_mut();
    }
    (*CredentialW).Persist = (*CredentialA).Persist;
    (*CredentialW).AttributeCount = 0;
    (*CredentialW).Attributes = core::ptr::null_mut();
    if !(*CredentialA).TargetAlias.is_null() {
        (*CredentialW).TargetAlias = buffer as LPWSTR;
        string_len = MultiByteToWideChar(CP_ACP, 0, (*CredentialA).TargetAlias, -1, (*CredentialW).TargetAlias, len / core::mem::size_of::<WCHAR>() as INT);
        buffer = buffer.add(core::mem::size_of::<WCHAR>() * string_len as usize);
        needed += core::mem::size_of::<WCHAR>() as INT * string_len;
        len -= core::mem::size_of::<WCHAR>() as INT * string_len;
    } else {
        (*CredentialW).TargetAlias = core::ptr::null_mut();
    }
    if !(*CredentialA).UserName.is_null() {
        (*CredentialW).UserName = buffer as LPWSTR;
        string_len = MultiByteToWideChar(CP_ACP, 0, (*CredentialA).UserName, -1, (*CredentialW).UserName, len / core::mem::size_of::<WCHAR>() as INT);
        needed += core::mem::size_of::<WCHAR>() as INT * string_len;
    } else {
        (*CredentialW).UserName = core::ptr::null_mut();
    }

    needed
}

#[no_mangle]
pub unsafe extern "system" fn CredDeleteA(TargetName: LPCSTR, Type: DWORD, Flags: DWORD) -> BOOL {
    let TargetNameW: LPWSTR;
    let len: DWORD;
    let ret: BOOL;

    if TargetName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    len = MultiByteToWideChar(CP_ACP, 0, TargetName, -1, core::ptr::null_mut(), 0) as DWORD;
    TargetNameW = heap_alloc((len as usize) * core::mem::size_of::<u16>()) as LPWSTR;
    if TargetNameW.is_null() {
        SetLastError(ERROR_OUTOFMEMORY);
        return 0;
    }
    MultiByteToWideChar(CP_ACP, 0, TargetName, -1, TargetNameW, len as i32);

    ret = CredDeleteW(TargetNameW, Type, Flags);

    heap_free(TargetNameW as *mut core::ffi::c_void);

    ret
}

#[no_mangle]
pub unsafe extern "system" fn CredDeleteW(TargetName: LPCWSTR, Type: DWORD, Flags: DWORD) -> BOOL {
    let mut hkeyMgr: HKEY = core::ptr::null_mut();
    let mut ret: DWORD;
    let key_name: LPWSTR;

    if TargetName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if Type != CRED_TYPE_GENERIC && Type != CRED_TYPE_DOMAIN_PASSWORD {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if Flags != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    ret = open_cred_mgr_key(&mut hkeyMgr, 1);
    if ret != ERROR_SUCCESS {
        SetLastError(ERROR_NO_SUCH_LOGON_SESSION);
        return 0;
    }

    key_name = get_key_name_for_target(TargetName, Type);
    ret = (RegDeleteKeyW(hkeyMgr, key_name)) as u32;
    heap_free(key_name as *mut _);
    RegCloseKey(hkeyMgr);
    if ret != ERROR_SUCCESS {
        SetLastError(ERROR_NOT_FOUND);
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn CredEnumerateA(
    Filter: LPCSTR,
    Flags: DWORD,
    Count: *mut DWORD,
    Credentials: *mut *mut PCREDENTIALA,
) -> BOOL {
    let FilterW: LPWSTR;
    let mut CredentialsW: *mut PCREDENTIALW = core::ptr::null_mut();
    let mut i: DWORD;
    let mut len: INT;
    let mut needed: INT;
    let mut buffer: *mut core::ffi::c_char;

    if !Filter.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, Filter, -1, core::ptr::null_mut(), 0);
        FilterW = heap_alloc((len as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
        if FilterW.is_null() {
            SetLastError(ERROR_OUTOFMEMORY);
            return 0;
        }
        MultiByteToWideChar(CP_ACP, 0, Filter, -1, FilterW, len);
    } else {
        FilterW = core::ptr::null_mut();
    }

    if CredEnumerateW(FilterW, Flags, Count, &mut CredentialsW) == 0 {
        heap_free(FilterW as LPVOID);
        return 0;
    }
    heap_free(FilterW as LPVOID);

    len = (*Count as usize * core::mem::size_of::<PCREDENTIALA>()) as INT;
    i = 0;
    while i < *Count {
        len += convert_PCREDENTIALW_to_PCREDENTIALA(*CredentialsW.add(i as usize), core::ptr::null_mut(), 0);
        i += 1;
    }

    *Credentials = heap_alloc(len as usize) as *mut PCREDENTIALA;
    if (*Credentials).is_null() {
        CredFree(CredentialsW as PVOID);
        SetLastError(ERROR_OUTOFMEMORY);
        return 0;
    }

    buffer = (*Credentials).add(*Count as usize) as *mut core::ffi::c_char;
    len -= (*Count as usize * core::mem::size_of::<PCREDENTIALA>()) as INT;
    i = 0;
    while i < *Count {
        *(*Credentials).add(i as usize) = buffer as PCREDENTIALA;
        needed = convert_PCREDENTIALW_to_PCREDENTIALA(
            *CredentialsW.add(i as usize),
            *(*Credentials).add(i as usize),
            len as DWORD,
        );
        buffer = buffer.add(needed as usize);
        len -= needed;
        i += 1;
    }

    CredFree(CredentialsW as PVOID);

    1
}

#[no_mangle]
pub unsafe extern "system" fn CredEnumerateW(
    Filter: LPCWSTR,
    Flags: DWORD,
    Count: *mut DWORD,
    Credentials: *mut *mut PCREDENTIALW,
) -> BOOL {
    let mut hkeyMgr: HKEY = core::ptr::null_mut();
    let mut ret: DWORD;
    let target_name: LPWSTR;
    let mut target_name_len: DWORD = 0;
    let mut len: DWORD;
    let mut buffer: *mut core::ffi::c_char;
    let mut key_data: [u8; 8] = [0; 8];

    TRACE!(b"(%s, 0x%x, %p, %p)\n\0".as_ptr() as *const i8, debugstr_w(Filter), Flags, Count, Credentials);

    if Flags != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    ret = open_cred_mgr_key(&mut hkeyMgr, 0);
    if ret != ERROR_SUCCESS {
        WARN!(b"couldn't open/create manager key, error %d\n\0".as_ptr() as *const i8, ret);
        SetLastError(ERROR_NO_SUCH_LOGON_SESSION);
        return 0;
    }

    ret = get_cred_mgr_encryption_key(hkeyMgr, key_data.as_mut_ptr());
    if ret != ERROR_SUCCESS {
        RegCloseKey(hkeyMgr);
        SetLastError(ret);
        return 0;
    }

    ret = (RegQueryInfoKeyW(
        hkeyMgr,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut target_name_len,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )) as u32;
    if ret != ERROR_SUCCESS {
        RegCloseKey(hkeyMgr);
        SetLastError(ret);
        return 0;
    }

    target_name = heap_alloc(((target_name_len + 1) as usize) * core::mem::size_of::<u16>()) as LPWSTR;
    if target_name.is_null() {
        RegCloseKey(hkeyMgr);
        SetLastError(ERROR_OUTOFMEMORY);
        return 0;
    }

    *Count = 0;
    len = 0;
    ret = registry_enumerate_credentials(
        hkeyMgr,
        Filter,
        target_name,
        target_name_len,
        key_data.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut len,
        Count,
    );

    if ret == ERROR_SUCCESS && *Count == 0 {
        ret = ERROR_NOT_FOUND;
    }
    if ret != ERROR_SUCCESS {
        heap_free(target_name as *mut core::ffi::c_void);
        RegCloseKey(hkeyMgr);
        SetLastError(ret);
        return 0;
    }
    len += *Count * core::mem::size_of::<PCREDENTIALW>() as DWORD;

    if ret == ERROR_SUCCESS {
        buffer = heap_alloc(len as usize) as *mut core::ffi::c_char;
        *Credentials = buffer as *mut PCREDENTIALW;
        if !buffer.is_null() {
            buffer = buffer.add((*Count as usize) * core::mem::size_of::<PCREDENTIALW>());
            *Count = 0;
            ret = registry_enumerate_credentials(
                hkeyMgr,
                Filter,
                target_name,
                target_name_len,
                key_data.as_ptr(),
                *Credentials,
                &mut buffer,
                &mut len,
                Count,
            );
        } else {
            ret = ERROR_OUTOFMEMORY;
        }
    }

    heap_free(target_name as *mut core::ffi::c_void);
    RegCloseKey(hkeyMgr);

    if ret != ERROR_SUCCESS {
        SetLastError(ret);
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn CredFree(Buffer: PVOID) {
    heap_free(Buffer);
}

#[no_mangle]
pub unsafe extern "system" fn CredReadA(
    TargetName: LPCSTR,
    Type: DWORD,
    Flags: DWORD,
    Credential: *mut PCREDENTIALA,
) -> BOOL {
    let mut CredentialW: PCREDENTIALW = core::ptr::null_mut();
    let mut len: INT;

    if TargetName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    len = MultiByteToWideChar(CP_ACP, 0, TargetName, -1, core::ptr::null_mut(), 0);
    let TargetNameW = heap_alloc((len as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
    if TargetNameW.is_null() {
        SetLastError(ERROR_OUTOFMEMORY);
        return 0;
    }
    MultiByteToWideChar(CP_ACP, 0, TargetName, -1, TargetNameW, len);

    if CredReadW(TargetNameW, Type, Flags, &mut CredentialW) == 0 {
        heap_free(TargetNameW as *mut _);
        return 0;
    }
    heap_free(TargetNameW as *mut _);

    len = convert_PCREDENTIALW_to_PCREDENTIALA(CredentialW, core::ptr::null_mut(), 0);
    *Credential = heap_alloc(len as usize) as PCREDENTIALA;
    if (*Credential).is_null() {
        SetLastError(ERROR_OUTOFMEMORY);
        return 0;
    }
    convert_PCREDENTIALW_to_PCREDENTIALA(CredentialW, *Credential, len as DWORD);

    CredFree(CredentialW as PVOID);

    1
}

#[no_mangle]
pub unsafe extern "system" fn CredReadW(
    TargetName: LPCWSTR,
    Type: DWORD,
    Flags: DWORD,
    Credential: *mut PCREDENTIALW,
) -> BOOL {
    let mut hkeyMgr: HKEY = core::ptr::null_mut();
    let mut hkeyCred: HKEY = core::ptr::null_mut();
    let mut ret: DWORD;
    let key_name: LPWSTR;
    let mut len: DWORD;
    let mut key_data: [u8; KEY_SIZE as usize] = [0; KEY_SIZE as usize];

    if TargetName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if Type != CRED_TYPE_GENERIC && Type != CRED_TYPE_DOMAIN_PASSWORD {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if Flags != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    ret = open_cred_mgr_key(&mut hkeyMgr, 0);
    if ret != ERROR_SUCCESS {
        SetLastError(ERROR_NO_SUCH_LOGON_SESSION);
        return 0;
    }

    ret = get_cred_mgr_encryption_key(hkeyMgr, key_data.as_mut_ptr());
    if ret != ERROR_SUCCESS {
        RegCloseKey(hkeyMgr);
        SetLastError(ret);
        return 0;
    }

    key_name = get_key_name_for_target(TargetName, Type);
    ret = (RegOpenKeyExW(hkeyMgr, key_name, 0, KEY_QUERY_VALUE, &mut hkeyCred)) as u32;
    heap_free(key_name as LPVOID);
    if ret != ERROR_SUCCESS {
        SetLastError(ERROR_NOT_FOUND);
        return 0;
    }

    len = core::mem::size_of::<CREDENTIALW>() as DWORD;
    ret = registry_read_credential(
        hkeyCred,
        core::ptr::null_mut(),
        key_data.as_ptr(),
        core::ptr::null_mut(),
        &mut len,
    );
    if ret == ERROR_SUCCESS {
        *Credential = heap_alloc(len as usize) as PCREDENTIALW;
        if !(*Credential).is_null() {
            len = core::mem::size_of::<CREDENTIALW>() as DWORD;
            ret = registry_read_credential(
                hkeyCred,
                *Credential,
                key_data.as_ptr(),
                (*Credential).add(1) as *mut i8,
                &mut len,
            );
        } else {
            ret = ERROR_OUTOFMEMORY;
        }
    }

    RegCloseKey(hkeyCred);
    RegCloseKey(hkeyMgr);

    if ret != ERROR_SUCCESS {
        SetLastError(ret);
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn CredReadDomainCredentialsA(
    TargetInformation: PCREDENTIAL_TARGET_INFORMATIONA,
    Flags: DWORD,
    Size: *mut DWORD,
    Credentials: *mut *mut PCREDENTIALA,
) -> BOOL {
    let TargetInformationW: PCREDENTIAL_TARGET_INFORMATIONW;
    let mut len: INT;
    let mut i: DWORD;
    let mut buffer: *mut WCHAR;
    let end: *mut WCHAR;
    let ret: BOOL;
    let mut CredentialsW: *mut PCREDENTIALW = core::ptr::null_mut();

    *Size = 0;
    *Credentials = core::ptr::null_mut();

    if TargetInformation.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    len = core::mem::size_of::<CREDENTIAL_TARGET_INFORMATIONW>() as INT;
    if !(*TargetInformation).TargetName.is_null() {
        len += MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).TargetName, -1, core::ptr::null_mut(), 0) * core::mem::size_of::<WCHAR>() as INT;
    }
    if !(*TargetInformation).NetbiosServerName.is_null() {
        len += MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).NetbiosServerName, -1, core::ptr::null_mut(), 0) * core::mem::size_of::<WCHAR>() as INT;
    }
    if !(*TargetInformation).DnsServerName.is_null() {
        len += MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).DnsServerName, -1, core::ptr::null_mut(), 0) * core::mem::size_of::<WCHAR>() as INT;
    }
    if !(*TargetInformation).NetbiosDomainName.is_null() {
        len += MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).NetbiosDomainName, -1, core::ptr::null_mut(), 0) * core::mem::size_of::<WCHAR>() as INT;
    }
    if !(*TargetInformation).DnsDomainName.is_null() {
        len += MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).DnsDomainName, -1, core::ptr::null_mut(), 0) * core::mem::size_of::<WCHAR>() as INT;
    }
    if !(*TargetInformation).DnsTreeName.is_null() {
        len += MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).DnsTreeName, -1, core::ptr::null_mut(), 0) * core::mem::size_of::<WCHAR>() as INT;
    }
    if !(*TargetInformation).PackageName.is_null() {
        len += MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).PackageName, -1, core::ptr::null_mut(), 0) * core::mem::size_of::<WCHAR>() as INT;
    }

    TargetInformationW = heap_alloc(len as usize) as PCREDENTIAL_TARGET_INFORMATIONW;
    if TargetInformationW.is_null() {
        SetLastError(ERROR_OUTOFMEMORY);
        return 0;
    }
    buffer = TargetInformationW.offset(1) as *mut WCHAR;
    end = (TargetInformationW as *mut u8).offset(len as isize) as *mut WCHAR;

    if !(*TargetInformation).TargetName.is_null() {
        (*TargetInformationW).TargetName = buffer;
        buffer = buffer.offset(MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).TargetName, -1,
            (*TargetInformationW).TargetName, end.offset_from(buffer) as INT) as isize);
    } else {
        (*TargetInformationW).TargetName = core::ptr::null_mut();
    }

    if !(*TargetInformation).NetbiosServerName.is_null() {
        (*TargetInformationW).NetbiosServerName = buffer;
        buffer = buffer.offset(MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).NetbiosServerName, -1,
            (*TargetInformationW).NetbiosServerName, end.offset_from(buffer) as INT) as isize);
    } else {
        (*TargetInformationW).NetbiosServerName = core::ptr::null_mut();
    }

    if !(*TargetInformation).DnsServerName.is_null() {
        (*TargetInformationW).DnsServerName = buffer;
        buffer = buffer.offset(MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).DnsServerName, -1,
            (*TargetInformationW).DnsServerName, end.offset_from(buffer) as INT) as isize);
    } else {
        (*TargetInformationW).DnsServerName = core::ptr::null_mut();
    }

    if !(*TargetInformation).NetbiosDomainName.is_null() {
        (*TargetInformationW).NetbiosDomainName = buffer;
        buffer = buffer.offset(MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).NetbiosDomainName, -1,
            (*TargetInformationW).NetbiosDomainName, end.offset_from(buffer) as INT) as isize);
    } else {
        (*TargetInformationW).NetbiosDomainName = core::ptr::null_mut();
    }

    if !(*TargetInformation).DnsDomainName.is_null() {
        (*TargetInformationW).DnsDomainName = buffer;
        buffer = buffer.offset(MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).DnsDomainName, -1,
            (*TargetInformationW).DnsDomainName, end.offset_from(buffer) as INT) as isize);
    } else {
        (*TargetInformationW).DnsDomainName = core::ptr::null_mut();
    }

    if !(*TargetInformation).DnsTreeName.is_null() {
        (*TargetInformationW).DnsTreeName = buffer;
        buffer = buffer.offset(MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).DnsTreeName, -1,
            (*TargetInformationW).DnsTreeName, end.offset_from(buffer) as INT) as isize);
    } else {
        (*TargetInformationW).DnsTreeName = core::ptr::null_mut();
    }

    if !(*TargetInformation).PackageName.is_null() {
        (*TargetInformationW).PackageName = buffer;
        MultiByteToWideChar(CP_ACP, 0, (*TargetInformation).PackageName, -1,
            (*TargetInformationW).PackageName, end.offset_from(buffer) as INT);
    } else {
        (*TargetInformationW).PackageName = core::ptr::null_mut();
    }

    (*TargetInformationW).Flags = (*TargetInformation).Flags;
    (*TargetInformationW).CredTypeCount = (*TargetInformation).CredTypeCount;
    (*TargetInformationW).CredTypes = (*TargetInformation).CredTypes;

    ret = CredReadDomainCredentialsW(TargetInformationW, Flags, Size, &mut CredentialsW);

    heap_free(TargetInformationW as LPVOID);

    if ret != 0 {
        let mut buf: *mut u8;
        let mut needed: INT;

        len = (*Size * core::mem::size_of::<PCREDENTIALA>() as DWORD) as INT;
        i = 0;
        while i < *Size {
            len += convert_PCREDENTIALW_to_PCREDENTIALA(*CredentialsW.offset(i as isize), core::ptr::null_mut(), 0);
            i += 1;
        }

        *Credentials = heap_alloc(len as usize) as *mut PCREDENTIALA;
        if (*Credentials).is_null() {
            CredFree(CredentialsW as PVOID);
            SetLastError(ERROR_OUTOFMEMORY);
            return 0;
        }

        buf = (*Credentials).offset(*Size as isize) as *mut u8;
        len -= (*Size * core::mem::size_of::<PCREDENTIALA>() as DWORD) as INT;
        i = 0;
        while i < *Size {
            *(*Credentials).offset(i as isize) = buf as PCREDENTIALA;
            needed = convert_PCREDENTIALW_to_PCREDENTIALA(*CredentialsW.offset(i as isize), *(*Credentials).offset(i as isize), len as DWORD);
            buf = buf.offset(needed as isize);
            len -= needed;
            i += 1;
        }

        CredFree(CredentialsW as PVOID);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn CredReadDomainCredentialsW(
    TargetInformation: PCREDENTIAL_TARGET_INFORMATIONW,
    Flags: DWORD,
    Size: *mut DWORD,
    Credentials: *mut *mut PCREDENTIALW,
) -> BOOL {
    FIXME!("(%p, 0x%x, %p, %p) stub\n",
          TargetInformation, Flags, Size, Credentials);

    *Size = 0;
    *Credentials = core::ptr::null_mut();
    if TargetInformation.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    SetLastError(ERROR_NOT_FOUND);
    0
}

#[no_mangle]
pub unsafe extern "system" fn CredWriteA(Credential: PCREDENTIALA, Flags: DWORD) -> BOOL {
    let ret: BOOL;
    let len: INT;
    let CredentialW: PCREDENTIALW;

    if Credential.is_null() || (*Credential).TargetName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    len = convert_PCREDENTIALA_to_PCREDENTIALW(Credential, core::ptr::null_mut(), 0);
    CredentialW = heap_alloc(len as usize) as PCREDENTIALW;
    if CredentialW.is_null() {
        SetLastError(ERROR_OUTOFMEMORY);
        return 0;
    }

    convert_PCREDENTIALA_to_PCREDENTIALW(Credential, CredentialW, len);

    ret = CredWriteW(CredentialW, Flags);

    heap_free(CredentialW as *mut _);

    ret
}

#[no_mangle]
pub unsafe extern "system" fn CredWriteW(Credential: PCREDENTIALW, Flags: DWORD) -> BOOL {
    let hkeyMgr: HKEY = core::ptr::null_mut();
    let mut hkeyMgr = hkeyMgr;
    let mut hkeyCred: HKEY = core::ptr::null_mut();
    let mut ret: DWORD;
    let key_name: LPWSTR;
    let mut key_data: [BYTE; KEY_SIZE as usize] = [0; KEY_SIZE as usize];

    TRACE!(b"(%p, 0x%x)\n\0".as_ptr() as *const i8, Credential, Flags);

    if Credential.is_null() || (*Credential).TargetName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if Flags & !CRED_PRESERVE_CREDENTIAL_BLOB != 0 {
        FIXME!("unhandled flags 0x%x\n", Flags);
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    if (*Credential).Type != CRED_TYPE_GENERIC && (*Credential).Type != CRED_TYPE_DOMAIN_PASSWORD {
        FIXME!("unhandled type %d\n", (*Credential).Type);
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    TRACE!(b"Credential->Flags = 0x%08x\n\0".as_ptr() as *const i8, (*Credential).Flags);
    TRACE!(b"Credential->Type = %u\n\0".as_ptr() as *const i8, (*Credential).Type);
    TRACE!(b"Credential->TargetName = %s\n\0".as_ptr() as *const i8, debugstr_w((*Credential).TargetName));
    TRACE!(b"Credential->Comment = %s\n\0".as_ptr() as *const i8, debugstr_w((*Credential).Comment));
    TRACE!(b"Credential->Persist = %u\n\0".as_ptr() as *const i8, (*Credential).Persist);
    TRACE!(b"Credential->TargetAlias = %s\n\0".as_ptr() as *const i8, debugstr_w((*Credential).TargetAlias));
    TRACE!(b"Credential->UserName = %s\n\0".as_ptr() as *const i8, debugstr_w((*Credential).UserName));

    if (*Credential).Type == CRED_TYPE_DOMAIN_PASSWORD {
        if (*Credential).UserName.is_null()
            || ((*Credential).Persist == CRED_PERSIST_ENTERPRISE
                && strchrW((*Credential).UserName, '\\' as u16).is_null()
                && strchrW((*Credential).UserName, '@' as u16).is_null())
        {
            ERR!("bad username %S\n", (*Credential).UserName);
            SetLastError(ERROR_BAD_USERNAME);
            return 0;
        }
    }

    ret = open_cred_mgr_key(&mut hkeyMgr, 0);
    if ret != ERROR_SUCCESS {
        WARN!(b"couldn't open/create manager key, error %d\n\0".as_ptr() as *const i8, ret);
        SetLastError(ERROR_NO_SUCH_LOGON_SESSION);
        return 0;
    }

    ret = get_cred_mgr_encryption_key(hkeyMgr, key_data.as_mut_ptr());
    if ret != ERROR_SUCCESS {
        RegCloseKey(hkeyMgr);
        SetLastError(ret);
        return 0;
    }

    key_name = get_key_name_for_target((*Credential).TargetName, (*Credential).Type);
    ret = (RegCreateKeyExW(
        hkeyMgr,
        key_name,
        0,
        core::ptr::null_mut(),
        if (*Credential).Persist == CRED_PERSIST_SESSION { REG_OPTION_VOLATILE } else { REG_OPTION_NON_VOLATILE },
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        &mut hkeyCred,
        core::ptr::null_mut(),
    )) as u32;
    heap_free(key_name as LPVOID);
    if ret != ERROR_SUCCESS {
        TRACE!(b"credentials for target name %s not found\n\0".as_ptr() as *const i8, debugstr_w((*Credential).TargetName));
        SetLastError(ERROR_NOT_FOUND);
        return 0;
    }

    ret = registry_write_credential(hkeyCred, Credential, key_data.as_ptr(), (Flags & CRED_PRESERVE_CREDENTIAL_BLOB) as BOOL);

    RegCloseKey(hkeyCred);
    RegCloseKey(hkeyMgr);

    if ret != ERROR_SUCCESS {
        SetLastError(ret);
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn CredGetSessionTypes(persistCount: DWORD, persists: LPDWORD) -> BOOL {
    memset(
        persists as *mut core::ffi::c_void,
        CRED_PERSIST_NONE as i32,
        (persistCount as usize) * core::mem::size_of::<DWORD>(),
    );

    if (CRED_TYPE_GENERIC as DWORD) < persistCount {
        *persists.add(CRED_TYPE_GENERIC as usize) = CRED_PERSIST_ENTERPRISE;

        if (CRED_TYPE_DOMAIN_PASSWORD as DWORD) < persistCount {
            *persists.add(CRED_TYPE_DOMAIN_PASSWORD as usize) = CRED_PERSIST_ENTERPRISE;
        }
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn CredMarshalCredentialA(
    type_: CRED_MARSHAL_TYPE,
    cred: PVOID,
    out: *mut LPSTR,
) -> BOOL {
    let mut outW: LPWSTR = core::ptr::null_mut();

    let ret = CredMarshalCredentialW(type_, cred, &mut outW);
    if ret != 0 {
        let len = WideCharToMultiByte(
            CP_ACP,
            0,
            outW,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        *out = heap_alloc(len as usize) as LPSTR;
        if (*out).is_null() {
            heap_free(outW as LPVOID);
            return 0;
        }
        WideCharToMultiByte(
            CP_ACP,
            0,
            outW,
            -1,
            *out,
            len,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        heap_free(outW as LPVOID);
    }
    ret
}

unsafe fn cred_encode(mut bin: *const core::ffi::c_char, mut len: core::ffi::c_uint, cred: *mut u16) -> u32 {
    let enc: &[u8; 65] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789#-\0";
    let mut n: u32 = 0;
    let mut x: u32;

    while len > 0 {
        *cred.offset(n as isize) = enc[(*bin as u32 & 0x3f) as usize] as u16;
        n += 1;
        x = (*bin as u32 & 0xc0) >> 6;
        if len == 1 {
            *cred.offset(n as isize) = enc[x as usize] as u16;
            n += 1;
            break;
        }
        *cred.offset(n as isize) = enc[((((*bin.offset(1) as u32 & 0xf) << 2) | x)) as usize] as u16;
        n += 1;
        x = (*bin.offset(1) as u32 & 0xf0) >> 4;
        if len == 2 {
            *cred.offset(n as isize) = enc[x as usize] as u16;
            n += 1;
            break;
        }
        *cred.offset(n as isize) = enc[((((*bin.offset(2) as u32 & 0x3) << 4) | x)) as usize] as u16;
        n += 1;
        *cred.offset(n as isize) = enc[((*bin.offset(2) as u32 & 0xfc) >> 2) as usize] as u16;
        n += 1;
        bin = bin.offset(3);
        len -= 3;
    }
    n
}

#[no_mangle]
pub unsafe extern "system" fn CredMarshalCredentialW(
    type_: CRED_MARSHAL_TYPE,
    cred: PVOID,
    out: *mut LPWSTR,
) -> BOOL {
    let cert = cred as *mut CERT_CREDENTIAL_INFO;
    let target = cred as *mut USERNAME_TARGET_CREDENTIAL_INFO;
    let mut len: DWORD;
    let mut size: DWORD;
    let p: *mut WCHAR;

    if cred.is_null()
        || (type_ == CertCredential
            && (*cert).cbSize < core::mem::size_of::<CERT_CREDENTIAL_INFO>() as u32)
        || (type_ != CertCredential
            && type_ != UsernameTargetCredential
            && type_ != BinaryBlobCredential)
        || (type_ == UsernameTargetCredential
            && ((*target).UserName.is_null() || *(*target).UserName == 0))
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    match type_ {
        CertCredential => {
            size = (core::mem::size_of_val(&(*cert).rgbHashOfCert) as u32 + 2) * 4 / 3;
            p = heap_alloc(((size + 4) as usize * core::mem::size_of::<WCHAR>()) as _) as *mut WCHAR;
            if p.is_null() {
                return 0;
            }
            *p.offset(0) = b'@' as WCHAR;
            *p.offset(1) = b'@' as WCHAR;
            *p.offset(2) = (b'A' as u32 + type_ as u32) as WCHAR;
            len = cred_encode(
                core::ptr::addr_of!((*cert).rgbHashOfCert) as *const core::ffi::c_char,
                core::mem::size_of_val(&(*cert).rgbHashOfCert) as core::ffi::c_uint,
                p.offset(3),
            );
            *p.offset(len as isize + 3) = 0;
        }
        UsernameTargetCredential => {
            len = strlenW((*target).UserName) as u32;
            size = (core::mem::size_of::<DWORD>() as u32
                + len * core::mem::size_of::<WCHAR>() as u32
                + 2)
                * 4
                / 3;
            p = heap_alloc(((size + 4) as usize * core::mem::size_of::<WCHAR>()) as _) as *mut WCHAR;
            if p.is_null() {
                return 0;
            }
            *p.offset(0) = b'@' as WCHAR;
            *p.offset(1) = b'@' as WCHAR;
            *p.offset(2) = (b'A' as u32 + type_ as u32) as WCHAR;
            size = len * core::mem::size_of::<WCHAR>() as u32;
            len = cred_encode(
                core::ptr::addr_of!(size) as *const core::ffi::c_char,
                core::mem::size_of::<DWORD>() as core::ffi::c_uint,
                p.offset(3),
            );
            len += cred_encode(
                (*target).UserName as *const core::ffi::c_char,
                size as core::ffi::c_uint,
                p.offset(3 + len as isize),
            );
            *p.offset(len as isize + 3) = 0;
        }
        BinaryBlobCredential => {
            FIXME!("BinaryBlobCredential not implemented\n");
            return 0;
        }
        _ => {
            return 0;
        }
    }

    *out = p;
    1
}

#[no_mangle]
pub unsafe extern "system" fn CredUnmarshalCredentialA(
    cred: LPCSTR,
    type_: PCRED_MARSHAL_TYPE,
    out: *mut PVOID,
) -> BOOL {
    let ret: BOOL;
    let mut credW: *mut WCHAR = core::ptr::null_mut();

    if !cred.is_null() {
        let len = MultiByteToWideChar(CP_ACP, 0, cred, -1, core::ptr::null_mut(), 0);
        credW = heap_alloc((len as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
        if credW.is_null() {
            return 0;
        }
        MultiByteToWideChar(CP_ACP, 0, cred, -1, credW, len);
    }
    ret = CredUnmarshalCredentialW(credW, type_, out);
    heap_free(credW as LPVOID);
    ret
}

unsafe fn char_decode(c: WCHAR) -> core::ffi::c_char {
    if c >= b'A' as WCHAR && c <= b'Z' as WCHAR {
        return (c - b'A' as WCHAR) as core::ffi::c_char;
    }
    if c >= b'a' as WCHAR && c <= b'z' as WCHAR {
        return (c - b'a' as WCHAR + 26) as core::ffi::c_char;
    }
    if c >= b'0' as WCHAR && c <= b'9' as WCHAR {
        return (c - b'0' as WCHAR + 52) as core::ffi::c_char;
    }
    if c == b'#' as WCHAR {
        return 62;
    }
    if c == b'-' as WCHAR {
        return 63;
    }
    64
}

unsafe fn cred_decode(cred: *const WCHAR, mut len: core::ffi::c_uint, buf: *mut core::ffi::c_char) -> BOOL {
    let mut i: core::ffi::c_uint = 0;
    let mut p = cred;

    while len >= 4 {
        let c0 = char_decode(*p.offset(0));
        if (c0 as core::ffi::c_uchar) > 63 { return 0; }
        let c1 = char_decode(*p.offset(1));
        if (c1 as core::ffi::c_uchar) > 63 { return 0; }
        let c2 = char_decode(*p.offset(2));
        if (c2 as core::ffi::c_uchar) > 63 { return 0; }
        let c3 = char_decode(*p.offset(3));
        if (c3 as core::ffi::c_uchar) > 63 { return 0; }

        *buf.offset((i + 0) as isize) = ((c1 << 6) | c0) as core::ffi::c_char;
        *buf.offset((i + 1) as isize) = ((c2 << 4) | (c1 >> 2)) as core::ffi::c_char;
        *buf.offset((i + 2) as isize) = ((c3 << 2) | (c2 >> 4)) as core::ffi::c_char;
        len -= 4;
        i += 3;
        p = p.offset(4);
    }
    if len == 3 {
        let c0 = char_decode(*p.offset(0));
        if (c0 as core::ffi::c_uchar) > 63 { return 0; }
        let c1 = char_decode(*p.offset(1));
        if (c1 as core::ffi::c_uchar) > 63 { return 0; }
        let c2 = char_decode(*p.offset(2));
        if (c2 as core::ffi::c_uchar) > 63 { return 0; }

        *buf.offset((i + 0) as isize) = ((c1 << 6) | c0) as core::ffi::c_char;
        *buf.offset((i + 1) as isize) = ((c2 << 4) | (c1 >> 2)) as core::ffi::c_char;
    } else if len == 2 {
        let c0 = char_decode(*p.offset(0));
        if (c0 as core::ffi::c_uchar) > 63 { return 0; }
        let c1 = char_decode(*p.offset(1));
        if (c1 as core::ffi::c_uchar) > 63 { return 0; }

        *buf.offset((i + 0) as isize) = ((c1 << 6) | c0) as core::ffi::c_char;
    } else if len == 1 {
        return 0;
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn CredUnmarshalCredentialW(
    cred: LPCWSTR,
    type_: PCRED_MARSHAL_TYPE,
    out: *mut PVOID,
) -> BOOL {
    let len: core::ffi::c_uint;
    let buflen: core::ffi::c_uint;

    if cred.is_null()
        || *cred.offset(0) != '@' as WCHAR
        || *cred.offset(1) != '@' as WCHAR
        || char_decode(*cred.offset(2)) > 63
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    len = strlenW(cred.offset(3)) as core::ffi::c_uint;
    *type_ = char_decode(*cred.offset(2)) as _;
    match *type_ {
        CertCredential => {
            let mut hash: [core::ffi::c_char; CERT_HASH_LENGTH as usize] =
                [0; CERT_HASH_LENGTH as usize];
            let cert: *mut CERT_CREDENTIAL_INFO;

            if len != 27 || cred_decode(cred.offset(3), len, hash.as_mut_ptr()) == 0 {
                SetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
            cert = heap_alloc(core::mem::size_of::<CERT_CREDENTIAL_INFO>()) as *mut CERT_CREDENTIAL_INFO;
            if cert.is_null() {
                return 0;
            }
            memcpy(
                core::ptr::addr_of_mut!((*cert).rgbHashOfCert) as *mut core::ffi::c_void,
                hash.as_ptr() as *const core::ffi::c_void,
                core::mem::size_of_val(&(*cert).rgbHashOfCert),
            );
            (*cert).cbSize = core::mem::size_of::<CERT_CREDENTIAL_INFO>() as DWORD;
            *out = cert as PVOID;
        }
        UsernameTargetCredential => {
            let target: *mut USERNAME_TARGET_CREDENTIAL_INFO;
            let mut size: DWORD = 0;

            if len < 9
                || cred_decode(cred.offset(3), 6, core::ptr::addr_of_mut!(size) as *mut core::ffi::c_char) == 0
                || size as usize % core::mem::size_of::<WCHAR>() != 0
                || len - 6 != (size * 4 + 2) / 3
            {
                SetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
            buflen = core::mem::size_of::<USERNAME_TARGET_CREDENTIAL_INFO>() as core::ffi::c_uint
                + size
                + core::mem::size_of::<WCHAR>() as core::ffi::c_uint;
            target = heap_alloc(buflen as usize) as *mut USERNAME_TARGET_CREDENTIAL_INFO;
            if target.is_null() {
                return 0;
            }
            if cred_decode(cred.offset(9), len - 6, target.offset(1) as *mut core::ffi::c_char) == 0 {
                heap_free(target as *mut core::ffi::c_void);
                return 0;
            }
            (*target).UserName = target.offset(1) as *mut WCHAR;
            *(*target).UserName.offset((size as usize / core::mem::size_of::<WCHAR>()) as isize) = 0;
            *out = target as PVOID;
        }
        BinaryBlobCredential => {
            return 0;
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }
    1
}

#[no_mangle]
pub unsafe extern "system" fn CredIsMarshaledCredentialW(name: LPCWSTR) -> BOOL {
    const CERT_HASH_LENGTH: usize = 20;
    const CertCredential: i32 = 1;
    const UsernameTargetCredential: i32 = 2;
    const BinaryBlobCredential: i32 = 3;

    if !name.is_null()
        && *name.offset(0) == '@' as WCHAR
        && *name.offset(1) == '@' as WCHAR
        && *name.offset(2) > 'A' as WCHAR
        && *name.offset(3) != 0
    {
        let mut hash: [core::ffi::c_char; CERT_HASH_LENGTH] = [0; CERT_HASH_LENGTH];
        let len = strlenW(name.offset(3)) as i32;
        let mut size: DWORD = 0;

        let type_val = (*name.offset(2) as i32) - ('A' as i32);

        if type_val == CertCredential
            && len == 27
            && cred_decode(name.offset(3), len as core::ffi::c_uint, hash.as_mut_ptr()) != 0
        {
            return 1;
        }

        if type_val == UsernameTargetCredential
            && len >= 9
            && cred_decode(name.offset(3), 6, core::ptr::addr_of_mut!(size) as *mut core::ffi::c_char) != 0
            && size != 0
        {
            return 1;
        }

        if type_val == BinaryBlobCredential {
            FIXME!("BinaryBlobCredential not checked\n");
        }

        if type_val > BinaryBlobCredential {
            TRACE!(b"unknown type: %d\n\0".as_ptr() as *const core::ffi::c_char, type_val);
        }
    }

    SetLastError(ERROR_INVALID_PARAMETER);
    0
}

#[no_mangle]
pub unsafe extern "system" fn CredIsMarshaledCredentialA(name: LPCSTR) -> BOOL {
    let mut nameW: LPWSTR = core::ptr::null_mut();
    let res: BOOL;
    let len: i32;

    if !name.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, name, -1, core::ptr::null_mut(), 0);
        nameW = heap_alloc((len as usize * core::mem::size_of::<u16>()) as _) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, name, -1, nameW, len);
    }

    res = CredIsMarshaledCredentialW(nameW);
    heap_free(nameW as _);
    res
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Credential Management APIs
//  *
//  * Copyright 2007 Robert Shearman for CodeWeavers
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #ifdef __REACTOS__
// #include <advapi32.h>
//
// #include <wincred.h>
// #else
// #include <stdarg.h>
// #include <time.h>
// #include <limits.h>
//
// #ifdef __APPLE__
// # include <Security/SecKeychain.h>
// # include <Security/SecKeychainItem.h>
// # include <Security/SecKeychainSearch.h>
// #endif
//
// #include "windef.h"
// #include "winbase.h"
// #include "winreg.h"
// #include "wincred.h"
// #include "winternl.h"
//
// #include "crypt.h"
//
// #include "wine/unicode.h"
// #include "wine/debug.h"
//
// #include "advapi32_misc.h"
// #endif /* __REACTOS__ */
//
// WINE_DEFAULT_DEBUG_CHANNEL(cred);
//
// /* the size of the ARC4 key used to encrypt the password data */
// #define KEY_SIZE 8
//
// static const WCHAR wszCredentialManagerKey[] = {'S','o','f','t','w','a','r','e','\\','W','i','n','e','\\',
//     'C','r','e','d','e','n','t','i','a','l',' ','M','a','n','a','g','e','r',0};
// static const WCHAR wszEncryptionKeyValue[] = {'E','n','c','r','y','p','t','i','o','n','K','e','y',0};
//
// static const WCHAR wszFlagsValue[] = {'F','l','a','g','s',0};
// static const WCHAR wszTypeValue[] = {'T','y','p','e',0};
// static const WCHAR wszCommentValue[] = {'C','o','m','m','e','n','t',0};
// static const WCHAR wszLastWrittenValue[] = {'L','a','s','t','W','r','i','t','t','e','n',0};
// static const WCHAR wszPersistValue[] = {'P','e','r','s','i','s','t',0};
// static const WCHAR wszTargetAliasValue[] = {'T','a','r','g','e','t','A','l','i','a','s',0};
// static const WCHAR wszUserNameValue[] = {'U','s','e','r','N','a','m','e',0};
// static const WCHAR wszPasswordValue[] = {'P','a','s','s','w','o','r','d',0};
//
// static DWORD read_credential_blob(HKEY hkey, const BYTE key_data[KEY_SIZE],
//                                   LPBYTE credential_blob,
//                                   DWORD *credential_blob_size)
// {
//     DWORD ret;
//     DWORD type;
//
//     *credential_blob_size = 0;
//     ret = RegQueryValueExW(hkey, wszPasswordValue, 0, &type, NULL, credential_blob_size);
//     if (ret != ERROR_SUCCESS)
//         return ret;
//     else if (type != REG_BINARY)
//         return ERROR_REGISTRY_CORRUPT;
//     if (credential_blob)
//     {
//         struct ustring data;
//         struct ustring key;
//
//         ret = RegQueryValueExW(hkey, wszPasswordValue, 0, &type, credential_blob,
//                                credential_blob_size);
//         if (ret != ERROR_SUCCESS)
//             return ret;
//         else if (type != REG_BINARY)
//             return ERROR_REGISTRY_CORRUPT;
//
//         key.Length = key.MaximumLength = KEY_SIZE;
//         key.Buffer = (unsigned char *)key_data;
//
//         data.Length = data.MaximumLength = *credential_blob_size;
//         data.Buffer = credential_blob;
//         SystemFunction032(&data, &key);
//     }
//     return ERROR_SUCCESS;
// }
//
// static DWORD registry_read_credential(HKEY hkey, PCREDENTIALW credential,
//                                       const BYTE key_data[KEY_SIZE],
//                                       char *buffer, DWORD *len)
// {
//     DWORD type;
//     DWORD ret;
//     DWORD count;
//
//     ret = RegQueryValueExW(hkey, NULL, 0, &type, NULL, &count);
//     if (ret != ERROR_SUCCESS)
//         return ret;
//     else if (type != REG_SZ)
//         return ERROR_REGISTRY_CORRUPT;
//     *len += count;
//     if (credential)
//     {
//         credential->TargetName = (LPWSTR)buffer;
//         ret = RegQueryValueExW(hkey, NULL, 0, &type, (LPVOID)credential->TargetName,
//                                &count);
// #ifdef __REACTOS__
//         if (ret != ERROR_SUCCESS)
//             return ret;
//         else if (type != REG_SZ)
//             return ERROR_REGISTRY_CORRUPT;
// #else
//         if (ret != ERROR_SUCCESS || type != REG_SZ) return ret;
// #endif
//         buffer += count;
//     }
//
//     ret = RegQueryValueExW(hkey, wszCommentValue, 0, &type, NULL, &count);
// #ifdef __REACTOS__
//     if (ret != ERROR_FILE_NOT_FOUND)
//     {
//         if (ret != ERROR_SUCCESS)
//             return ret;
//         else if (type != REG_SZ)
//             return ERROR_REGISTRY_CORRUPT;
//         *len += count;
//     }
// #else
//     if (ret != ERROR_FILE_NOT_FOUND && ret != ERROR_SUCCESS)
//         return ret;
//     else if (type != REG_SZ)
//         return ERROR_REGISTRY_CORRUPT;
//     *len += count;
// #endif
//     if (credential)
//     {
//         credential->Comment = (LPWSTR)buffer;
//         ret = RegQueryValueExW(hkey, wszCommentValue, 0, &type, (LPVOID)credential->Comment,
//                                &count);
//         if (ret == ERROR_FILE_NOT_FOUND)
//             credential->Comment = NULL;
//         else if (ret != ERROR_SUCCESS)
//             return ret;
//         else if (type != REG_SZ)
//             return ERROR_REGISTRY_CORRUPT;
//         else
//             buffer += count;
//     }
//
//     ret = RegQueryValueExW(hkey, wszTargetAliasValue, 0, &type, NULL, &count);
// #ifdef __REACTOS__
//     if (ret != ERROR_FILE_NOT_FOUND)
//     {
//         if (ret != ERROR_SUCCESS)
//             return ret;
//         else if (type != REG_SZ)
//             return ERROR_REGISTRY_CORRUPT;
//         *len += count;
//     }
// #else
//     if (ret != ERROR_FILE_NOT_FOUND && ret != ERROR_SUCCESS)
//         return ret;
//     else if (type != REG_SZ)
//         return ERROR_REGISTRY_CORRUPT;
//     *len += count;
// #endif
//     if (credential)
//     {
//         credential->TargetAlias = (LPWSTR)buffer;
//         ret = RegQueryValueExW(hkey, wszTargetAliasValue, 0, &type, (LPVOID)credential->TargetAlias,
//                                &count);
//         if (ret == ERROR_FILE_NOT_FOUND)
//             credential->TargetAlias = NULL;
//         else if (ret != ERROR_SUCCESS)
//             return ret;
//         else if (type != REG_SZ)
//             return ERROR_REGISTRY_CORRUPT;
//         else
//             buffer += count;
//     }
//
//     ret = RegQueryValueExW(hkey, wszUserNameValue, 0, &type, NULL, &count);
// #ifdef __REACTOS__
//     if (ret != ERROR_FILE_NOT_FOUND)
//     {
//         if (ret != ERROR_SUCCESS)
//             return ret;
//         else if (type != REG_SZ)
//             return ERROR_REGISTRY_CORRUPT;
//         *len += count;
//     }
// #else
//     if (ret != ERROR_FILE_NOT_FOUND && ret != ERROR_SUCCESS)
//         return ret;
//     else if (type != REG_SZ)
//         return ERROR_REGISTRY_CORRUPT;
//     *len += count;
// #endif
//     if (credential)
//     {
//         credential->UserName = (LPWSTR)buffer;
//         ret = RegQueryValueExW(hkey, wszUserNameValue, 0, &type, (LPVOID)credential->UserName,
//                                &count);
//         if (ret == ERROR_FILE_NOT_FOUND)
//             credential->UserName = NULL;
//         else if (ret != ERROR_SUCCESS)
//             return ret;
//         else if (type != REG_SZ)
//             return ERROR_REGISTRY_CORRUPT;
//         else
//             buffer += count;
//     }
//
//     ret = read_credential_blob(hkey, key_data, NULL, &count);
// #ifdef __REACTOS__
//     if (ret != ERROR_FILE_NOT_FOUND)
//     {
//         if (ret != ERROR_SUCCESS)
//             return ret;
//         *len += count;
//     }
// #else
//     if (ret != ERROR_FILE_NOT_FOUND && ret != ERROR_SUCCESS)
//         return ret;
//     *len += count;
// #endif
//     if (credential)
//     {
//         credential->CredentialBlob = (LPBYTE)buffer;
//         ret = read_credential_blob(hkey, key_data, credential->CredentialBlob, &count);
//         if (ret == ERROR_FILE_NOT_FOUND)
//             credential->CredentialBlob = NULL;
//         else if (ret != ERROR_SUCCESS)
//             return ret;
//         credential->CredentialBlobSize = count;
//     }
//
//     /* FIXME: Attributes */
//     if (credential)
//     {
//         credential->AttributeCount = 0;
//         credential->Attributes = NULL;
//     }
//
//     if (!credential) return ERROR_SUCCESS;
//
//     count = sizeof(credential->Flags);
//     ret = RegQueryValueExW(hkey, wszFlagsValue, NULL, &type, (LPVOID)&credential->Flags,
//                            &count);
//     if (ret != ERROR_SUCCESS)
//         return ret;
//     else if (type != REG_DWORD)
//         return ERROR_REGISTRY_CORRUPT;
//     count = sizeof(credential->Type);
//     ret = RegQueryValueExW(hkey, wszTypeValue, NULL, &type, (LPVOID)&credential->Type,
//                            &count);
//     if (ret != ERROR_SUCCESS)
//         return ret;
//     else if (type != REG_DWORD)
//         return ERROR_REGISTRY_CORRUPT;
//
//     count = sizeof(credential->LastWritten);
//     ret = RegQueryValueExW(hkey, wszLastWrittenValue, NULL, &type, (LPVOID)&credential->LastWritten,
//                            &count);
//     if (ret != ERROR_SUCCESS)
//         return ret;
//     else if (type != REG_BINARY)
//         return ERROR_REGISTRY_CORRUPT;
//     count = sizeof(credential->Persist);
//     ret = RegQueryValueExW(hkey, wszPersistValue, NULL, &type, (LPVOID)&credential->Persist,
//                            &count);
//     if (ret == ERROR_SUCCESS && type != REG_DWORD)
//         return ERROR_REGISTRY_CORRUPT;
//     return ret;
// }
//
// #ifdef __APPLE__
// static DWORD mac_read_credential_from_item(SecKeychainItemRef item, BOOL require_password,
//                                            PCREDENTIALW credential, char *buffer,
//                                            DWORD *len)
// {
//     int status;
//     UInt32 i, cred_blob_len;
//     void *cred_blob;
//     WCHAR *user = NULL;
//     BOOL user_name_present = FALSE;
//     SecKeychainAttributeInfo info;
//     SecKeychainAttributeList *attr_list;
//     UInt32 info_tags[] = { kSecServiceItemAttr, kSecAccountItemAttr,
//                            kSecCommentItemAttr, kSecCreationDateItemAttr };
//     info.count = sizeof(info_tags)/sizeof(info_tags[0]);
//     info.tag = info_tags;
//     info.format = NULL;
//     status = SecKeychainItemCopyAttributesAndData(item, &info, NULL, &attr_list, &cred_blob_len, &cred_blob);
//     if (status == errSecAuthFailed && !require_password)
//     {
//         cred_blob_len = 0;
//         cred_blob = NULL;
//         status = SecKeychainItemCopyAttributesAndData(item, &info, NULL, &attr_list, &cred_blob_len, NULL);
//     }
//     if (status != noErr)
//     {
//         WARN("SecKeychainItemCopyAttributesAndData returned status %d\n", status);
//         return ERROR_NOT_FOUND;
//     }
//
//     for (i = 0; i < attr_list->count; i++)
//         if (attr_list->attr[i].tag == kSecAccountItemAttr && attr_list->attr[i].data)
//         {
//             user_name_present = TRUE;
//             break;
//         }
//     if (!user_name_present)
//     {
//         WARN("no kSecAccountItemAttr for item\n");
//         SecKeychainItemFreeAttributesAndData(attr_list, cred_blob);
//         return ERROR_NOT_FOUND;
//     }
//
//     if (buffer)
//     {
//         credential->Flags = 0;
//         credential->Type = CRED_TYPE_DOMAIN_PASSWORD;
//         credential->TargetName = NULL;
//         credential->Comment = NULL;
//         memset(&credential->LastWritten, 0, sizeof(credential->LastWritten));
//         credential->CredentialBlobSize = 0;
//         credential->CredentialBlob = NULL;
//         credential->Persist = CRED_PERSIST_LOCAL_MACHINE;
//         credential->AttributeCount = 0;
//         credential->Attributes = NULL;
//         credential->TargetAlias = NULL;
//         credential->UserName = NULL;
//     }
//     for (i = 0; i < attr_list->count; i++)
//     {
//         switch (attr_list->attr[i].tag)
//         {
//             case kSecServiceItemAttr:
//                 TRACE("kSecServiceItemAttr: %.*s\n", (int)attr_list->attr[i].length,
//                       (char *)attr_list->attr[i].data);
//                 if (!attr_list->attr[i].data) continue;
//                 if (buffer)
//                 {
//                     INT str_len;
//                     credential->TargetName = (LPWSTR)buffer;
//                     str_len = MultiByteToWideChar(CP_UTF8, 0, attr_list->attr[i].data,
//                                                   attr_list->attr[i].length, (LPWSTR)buffer, 0xffff);
//                     credential->TargetName[str_len] = '\0';
//                     buffer += (str_len + 1) * sizeof(WCHAR);
//                     *len += (str_len + 1) * sizeof(WCHAR);
//                 }
//                 else
//                 {
//                     INT str_len;
//                     str_len = MultiByteToWideChar(CP_UTF8, 0, attr_list->attr[i].data,
//                                                   attr_list->attr[i].length, NULL, 0);
//                     *len += (str_len + 1) * sizeof(WCHAR);
//                 }
//                 break;
//             case kSecAccountItemAttr:
//             {
//                 INT str_len;
//                 TRACE("kSecAccountItemAttr: %.*s\n", (int)attr_list->attr[i].length,
//                       (char *)attr_list->attr[i].data);
//                 if (!attr_list->attr[i].data) continue;
//                 str_len = MultiByteToWideChar(CP_UTF8, 0, attr_list->attr[i].data,
//                                               attr_list->attr[i].length, NULL, 0);
//                 user = heap_alloc((str_len + 1) * sizeof(WCHAR));
//                 MultiByteToWideChar(CP_UTF8, 0, attr_list->attr[i].data,
//                                     attr_list->attr[i].length, user, str_len);
//                 user[str_len] = '\0';
//                 break;
//             }
//             case kSecCommentItemAttr:
//                 TRACE("kSecCommentItemAttr: %.*s\n", (int)attr_list->attr[i].length,
//                       (char *)attr_list->attr[i].data);
//                 if (!attr_list->attr[i].data) continue;
//                 if (buffer)
//                 {
//                     INT str_len;
//                     credential->Comment = (LPWSTR)buffer;
//                     str_len = MultiByteToWideChar(CP_UTF8, 0, attr_list->attr[i].data,
//                                                   attr_list->attr[i].length, (LPWSTR)buffer, 0xffff);
//                     credential->Comment[str_len] = '\0';
//                     buffer += (str_len + 1) * sizeof(WCHAR);
//                     *len += (str_len + 1) * sizeof(WCHAR);
//                 }
//                 else
//                 {
//                     INT str_len;
//                     str_len = MultiByteToWideChar(CP_UTF8, 0, attr_list->attr[i].data,
//                                                   attr_list->attr[i].length, NULL, 0);
//                     *len += (str_len + 1) * sizeof(WCHAR);
//                 }
//                 break;
//             case kSecCreationDateItemAttr:
//                 TRACE("kSecCreationDateItemAttr: %.*s\n", (int)attr_list->attr[i].length,
//                       (char *)attr_list->attr[i].data);
//                 if (!attr_list->attr[i].data) continue;
//                 if (buffer)
//                 {
//                     LARGE_INTEGER win_time;
//                     struct tm tm;
//                     time_t time;
//                     memset(&tm, 0, sizeof(tm));
//                     strptime(attr_list->attr[i].data, "%Y%m%d%H%M%SZ", &tm);
//                     time = mktime(&tm);
//                     RtlSecondsSince1970ToTime(time, &win_time);
//                     credential->LastWritten.dwLowDateTime = win_time.u.LowPart;
//                     credential->LastWritten.dwHighDateTime = win_time.u.HighPart;
//                 }
//                 break;
//             default:
//                 FIXME("unhandled attribute %u\n", (unsigned)attr_list->attr[i].tag);
//                 break;
//         }
//     }
//
//     if (user)
//     {
//         INT str_len;
//         if (buffer)
//             credential->UserName = (LPWSTR)buffer;
//         str_len = strlenW(user);
//         *len += (str_len + 1) * sizeof(WCHAR);
//         if (buffer)
//         {
//             memcpy(buffer, user, (str_len + 1) * sizeof(WCHAR));
//             buffer += (str_len + 1) * sizeof(WCHAR);
//             TRACE("UserName = %s\n", debugstr_w(credential->UserName));
//         }
//     }
//     heap_free(user);
//
//     if (cred_blob)
//     {
//         if (buffer)
//         {
//             INT str_len;
//             credential->CredentialBlob = (BYTE *)buffer;
//             str_len = MultiByteToWideChar(CP_UTF8, 0, cred_blob, cred_blob_len,
//                                           (LPWSTR)buffer, 0xffff);
//             credential->CredentialBlobSize = str_len * sizeof(WCHAR);
//             *len += str_len * sizeof(WCHAR);
//         }
//         else
//         {
//             INT str_len;
//             str_len = MultiByteToWideChar(CP_UTF8, 0, cred_blob, cred_blob_len,
//                                           NULL, 0);
//             *len += str_len * sizeof(WCHAR);
//         }
//     }
//     SecKeychainItemFreeAttributesAndData(attr_list, cred_blob);
//     return ERROR_SUCCESS;
// }
// #endif
//
// static DWORD write_credential_blob(HKEY hkey, LPCWSTR target_name, DWORD type,
//                                    const BYTE key_data[KEY_SIZE],
//                                    const BYTE *credential_blob, DWORD credential_blob_size)
// {
//     LPBYTE encrypted_credential_blob;
//     struct ustring data;
//     struct ustring key;
//     DWORD ret;
//
//     key.Length = key.MaximumLength = KEY_SIZE;
//     key.Buffer = (unsigned char *)key_data;
//
//     encrypted_credential_blob = heap_alloc(credential_blob_size);
//     if (!encrypted_credential_blob) return ERROR_OUTOFMEMORY;
//
//     memcpy(encrypted_credential_blob, credential_blob, credential_blob_size);
//     data.Length = data.MaximumLength = credential_blob_size;
//     data.Buffer = encrypted_credential_blob;
//     SystemFunction032(&data, &key);
//
//     ret = RegSetValueExW(hkey, wszPasswordValue, 0, REG_BINARY, encrypted_credential_blob, credential_blob_size);
//     heap_free(encrypted_credential_blob);
//
//     return ret;
// }
//
// static DWORD registry_write_credential(HKEY hkey, const CREDENTIALW *credential,
//                                        const BYTE key_data[KEY_SIZE], BOOL preserve_blob)
// {
//     DWORD ret;
//     FILETIME LastWritten;
//
//     GetSystemTimeAsFileTime(&LastWritten);
//
//     ret = RegSetValueExW(hkey, wszFlagsValue, 0, REG_DWORD, (const BYTE*)&credential->Flags,
//                          sizeof(credential->Flags));
//     if (ret != ERROR_SUCCESS) return ret;
//     ret = RegSetValueExW(hkey, wszTypeValue, 0, REG_DWORD, (const BYTE*)&credential->Type,
//                          sizeof(credential->Type));
//     if (ret != ERROR_SUCCESS) return ret;
//     ret = RegSetValueExW(hkey, NULL, 0, REG_SZ, (LPVOID)credential->TargetName,
//                          sizeof(WCHAR)*(strlenW(credential->TargetName)+1));
//     if (ret != ERROR_SUCCESS) return ret;
//     if (credential->Comment)
//     {
//         ret = RegSetValueExW(hkey, wszCommentValue, 0, REG_SZ, (LPVOID)credential->Comment,
//                              sizeof(WCHAR)*(strlenW(credential->Comment)+1));
//         if (ret != ERROR_SUCCESS) return ret;
//     }
//     ret = RegSetValueExW(hkey, wszLastWrittenValue, 0, REG_BINARY, (LPVOID)&LastWritten,
//                          sizeof(LastWritten));
//     if (ret != ERROR_SUCCESS) return ret;
//     ret = RegSetValueExW(hkey, wszPersistValue, 0, REG_DWORD, (const BYTE*)&credential->Persist,
//                          sizeof(credential->Persist));
//     if (ret != ERROR_SUCCESS) return ret;
//     /* FIXME: Attributes */
//     if (credential->TargetAlias)
//     {
//         ret = RegSetValueExW(hkey, wszTargetAliasValue, 0, REG_SZ, (LPVOID)credential->TargetAlias,
//                              sizeof(WCHAR)*(strlenW(credential->TargetAlias)+1));
//         if (ret != ERROR_SUCCESS) return ret;
//     }
//     if (credential->UserName)
//     {
//         ret = RegSetValueExW(hkey, wszUserNameValue, 0, REG_SZ, (LPVOID)credential->UserName,
//                              sizeof(WCHAR)*(strlenW(credential->UserName)+1));
//         if (ret != ERROR_SUCCESS) return ret;
//     }
//     if (!preserve_blob)
//     {
//         ret = write_credential_blob(hkey, credential->TargetName, credential->Type,
//                                     key_data, credential->CredentialBlob,
//                                     credential->CredentialBlobSize);
//     }
//     return ret;
// }
//
// #ifdef __APPLE__
// static DWORD mac_write_credential(const CREDENTIALW *credential, BOOL preserve_blob)
// {
//     int status;
//     SecKeychainItemRef keychain_item;
//     char *username, *password, *servername;
//     UInt32 userlen, pwlen, serverlen;
//     SecKeychainAttribute attrs[1];
//     SecKeychainAttributeList attr_list;
//
//     if (credential->Flags)
//         FIXME("Flags 0x%x not written\n", credential->Flags);
//     if (credential->Type != CRED_TYPE_DOMAIN_PASSWORD)
//         FIXME("credential type of %d not supported\n", credential->Type);
//     if (credential->Persist != CRED_PERSIST_LOCAL_MACHINE)
//         FIXME("persist value of %d not supported\n", credential->Persist);
//     if (credential->AttributeCount)
//         FIXME("custom attributes not supported\n");
//
//     userlen = WideCharToMultiByte(CP_UTF8, 0, credential->UserName, -1, NULL, 0, NULL, NULL);
//     username = heap_alloc(userlen * sizeof(*username));
//     WideCharToMultiByte(CP_UTF8, 0, credential->UserName, -1, username, userlen, NULL, NULL);
//
//     serverlen = WideCharToMultiByte(CP_UTF8, 0, credential->TargetName, -1, NULL, 0, NULL, NULL);
//     servername = heap_alloc(serverlen * sizeof(*servername));
//     WideCharToMultiByte(CP_UTF8, 0, credential->TargetName, -1, servername, serverlen, NULL, NULL);
//     pwlen = WideCharToMultiByte(CP_UTF8, 0, (LPCWSTR)credential->CredentialBlob,
//                                 credential->CredentialBlobSize / sizeof(WCHAR), NULL, 0, NULL, NULL);
//     password = heap_alloc(pwlen * sizeof(*password));
//     WideCharToMultiByte(CP_UTF8, 0, (LPCWSTR)credential->CredentialBlob,
//                         credential->CredentialBlobSize / sizeof(WCHAR), password, pwlen, NULL, NULL);
//
//     TRACE("adding server %s, username %s using Keychain\n", servername, username);
//     status = SecKeychainAddGenericPassword(NULL, strlen(servername), servername, strlen(username),
//                                            username, strlen(password), password, &keychain_item);
//     if (status != noErr)
//         ERR("SecKeychainAddGenericPassword returned %d\n", status);
//     if (status == errSecDuplicateItem)
//     {
//         status = SecKeychainFindGenericPassword(NULL, strlen(servername), servername, strlen(username),
//                                                 username, NULL, NULL, &keychain_item);
//         if (status != noErr)
//             ERR("SecKeychainFindGenericPassword returned %d\n", status);
//     }
//     heap_free(username);
//     heap_free(servername);
//     if (status != noErr)
//     {
//         heap_free(password);
//         return ERROR_GEN_FAILURE;
//     }
//     if (credential->Comment)
//     {
//         attr_list.count = 1;
//         attr_list.attr = attrs;
//         attrs[0].tag = kSecCommentItemAttr;
//         attrs[0].length = WideCharToMultiByte(CP_UTF8, 0, credential->Comment, -1, NULL, 0, NULL, NULL);
//         if (attrs[0].length) attrs[0].length--;
//         attrs[0].data = heap_alloc(attrs[0].length);
//         WideCharToMultiByte(CP_UTF8, 0, credential->Comment, -1, attrs[0].data, attrs[0].length, NULL, NULL);
//     }
//     else
//     {
//         attr_list.count = 0;
//         attr_list.attr = NULL;
//     }
//     status = SecKeychainItemModifyAttributesAndData(keychain_item, &attr_list,
//                                                     preserve_blob ? 0 : strlen(password),
//                                                     preserve_blob ? NULL : password);
//     if (credential->Comment)
//         heap_free(attrs[0].data);
//     heap_free(password);
//     /* FIXME: set TargetAlias attribute */
//     CFRelease(keychain_item);
//     if (status != noErr)
//         return ERROR_GEN_FAILURE;
//     return ERROR_SUCCESS;
// }
// #endif
//
// static DWORD open_cred_mgr_key(HKEY *hkey, BOOL open_for_write)
// {
//     return RegCreateKeyExW(HKEY_CURRENT_USER, wszCredentialManagerKey, 0,
//                            NULL, REG_OPTION_NON_VOLATILE,
//                            KEY_READ | (open_for_write ? KEY_WRITE : 0), NULL, hkey, NULL);
// }
//
// static DWORD get_cred_mgr_encryption_key(HKEY hkeyMgr, BYTE key_data[KEY_SIZE])
// {
//     static const BYTE my_key_data[KEY_SIZE] = { 0 };
//     DWORD type;
//     DWORD count;
//     FILETIME ft;
//     ULONG seed;
//     ULONG value;
//     DWORD ret;
//
//     memcpy(key_data, my_key_data, KEY_SIZE);
//
//     count = KEY_SIZE;
//     ret = RegQueryValueExW(hkeyMgr, wszEncryptionKeyValue, NULL, &type, key_data,
//                            &count);
//     if (ret == ERROR_SUCCESS)
//     {
//         if (type != REG_BINARY)
//             return ERROR_REGISTRY_CORRUPT;
//         else
//             return ERROR_SUCCESS;
//     }
//     if (ret != ERROR_FILE_NOT_FOUND)
//         return ret;
//
//     GetSystemTimeAsFileTime(&ft);
//     seed = ft.dwLowDateTime;
//     value = RtlUniform(&seed);
//     *(DWORD *)key_data = value;
//     seed = ft.dwHighDateTime;
//     value = RtlUniform(&seed);
//     *(DWORD *)(key_data + 4) = value;
//
//     ret = RegSetValueExW(hkeyMgr, wszEncryptionKeyValue, 0, REG_BINARY,
//                          key_data, KEY_SIZE);
//     if (ret == ERROR_ACCESS_DENIED)
//     {
//         ret = open_cred_mgr_key(&hkeyMgr, TRUE);
//         if (ret == ERROR_SUCCESS)
//         {
//             ret = RegSetValueExW(hkeyMgr, wszEncryptionKeyValue, 0, REG_BINARY,
//                                  key_data, KEY_SIZE);
//             RegCloseKey(hkeyMgr);
//         }
//     }
//     return ret;
// }
//
// static LPWSTR get_key_name_for_target(LPCWSTR target_name, DWORD type)
// {
//     static const WCHAR wszGenericPrefix[] = {'G','e','n','e','r','i','c',':',' ',0};
//     static const WCHAR wszDomPasswdPrefix[] = {'D','o','m','P','a','s','s','w','d',':',' ',0};
//     INT len;
//     LPCWSTR prefix = NULL;
//     LPWSTR key_name, p;
//
//     len = strlenW(target_name);
//     if (type == CRED_TYPE_GENERIC)
//     {
//         prefix = wszGenericPrefix;
//         len += sizeof(wszGenericPrefix)/sizeof(wszGenericPrefix[0]);
//     }
//     else
//     {
//         prefix = wszDomPasswdPrefix;
//         len += sizeof(wszDomPasswdPrefix)/sizeof(wszDomPasswdPrefix[0]);
//     }
//
//     key_name = heap_alloc(len * sizeof(WCHAR));
//     if (!key_name) return NULL;
//
//     strcpyW(key_name, prefix);
//     strcatW(key_name, target_name);
//
//     for (p = key_name; *p; p++)
//         if (*p == '\\') *p = '_';
//
//     return key_name;
// }
//
// static BOOL registry_credential_matches_filter(HKEY hkeyCred, LPCWSTR filter)
// {
//     LPWSTR target_name;
//     DWORD ret;
//     DWORD type;
//     DWORD count;
//     LPCWSTR p;
//
//     if (!filter) return TRUE;
//
//     ret = RegQueryValueExW(hkeyCred, NULL, 0, &type, NULL, &count);
//     if (ret != ERROR_SUCCESS)
//         return FALSE;
//     else if (type != REG_SZ)
//         return FALSE;
//
//     target_name = heap_alloc(count);
//     if (!target_name)
//         return FALSE;
//     ret = RegQueryValueExW(hkeyCred, NULL, 0, &type, (LPVOID)target_name, &count);
//     if (ret != ERROR_SUCCESS || type != REG_SZ)
//     {
//         heap_free(target_name);
//         return FALSE;
//     }
//
//     TRACE("comparing filter %s to target name %s\n", debugstr_w(filter),
//           debugstr_w(target_name));
//
//     p = strchrW(filter, '*');
//     ret = CompareStringW(GetThreadLocale(), NORM_IGNORECASE, filter,
//                          (p && !p[1] ? p - filter : -1), target_name,
//                          (p && !p[1] ? p - filter : -1)) == CSTR_EQUAL;
//
//     heap_free(target_name);
//     return ret;
// }
//
// static DWORD registry_enumerate_credentials(HKEY hkeyMgr, LPCWSTR filter,
//                                             LPWSTR target_name,
//                                             DWORD target_name_len, const BYTE key_data[KEY_SIZE],
//                                             PCREDENTIALW *credentials, char **buffer,
//                                             DWORD *len, DWORD *count)
// {
//     DWORD i;
//     DWORD ret;
//     for (i = 0;; i++)
//     {
//         HKEY hkeyCred;
//         ret = RegEnumKeyW(hkeyMgr, i, target_name, target_name_len+1);
//         if (ret == ERROR_NO_MORE_ITEMS)
//         {
//             ret = ERROR_SUCCESS;
//             break;
//         }
//         else if (ret != ERROR_SUCCESS)
//             continue;
//         TRACE("target_name = %s\n", debugstr_w(target_name));
//         ret = RegOpenKeyExW(hkeyMgr, target_name, 0, KEY_QUERY_VALUE, &hkeyCred);
//         if (ret != ERROR_SUCCESS)
//             continue;
//         if (!registry_credential_matches_filter(hkeyCred, filter))
//         {
//             RegCloseKey(hkeyCred);
//             continue;
//         }
//         if (buffer)
//         {
//             *len = sizeof(CREDENTIALW);
//             credentials[*count] = (PCREDENTIALW)*buffer;
//         }
//         else
//             *len += sizeof(CREDENTIALW);
//         ret = registry_read_credential(hkeyCred, buffer ? credentials[*count] : NULL,
//                                        key_data, buffer ? *buffer + sizeof(CREDENTIALW) : NULL,
//                                        len);
//         RegCloseKey(hkeyCred);
//         if (ret != ERROR_SUCCESS) break;
//         if (buffer) *buffer += *len;
//         (*count)++;
//     }
//     return ret;
// }
//
// #ifdef __APPLE__
// static BOOL mac_credential_matches_filter(void *data, UInt32 data_len, const WCHAR *filter)
// {
//     int len;
//     WCHAR *target_name;
//     const WCHAR *p;
//     BOOL ret;
//
//     if (!filter) return TRUE;
//
//     len = MultiByteToWideChar(CP_UTF8, 0, data, data_len, NULL, 0);
//     if (!(target_name = heap_alloc((len + 1) * sizeof(WCHAR)))) return FALSE;
//     MultiByteToWideChar(CP_UTF8, 0, data, data_len, target_name, len);
//     target_name[len] = 0;
//
//     TRACE("comparing filter %s to target name %s\n", debugstr_w(filter), debugstr_w(target_name));
//
//     p = strchrW(filter, '*');
//     ret = CompareStringW(GetThreadLocale(), NORM_IGNORECASE, filter,
//                          (p && !p[1] ? p - filter : -1), target_name,
//                          (p && !p[1] ? p - filter : -1)) == CSTR_EQUAL;
//     heap_free(target_name);
//     return ret;
// }
//
// static DWORD mac_enumerate_credentials(LPCWSTR filter, PCREDENTIALW *credentials,
//                                        char *buffer, DWORD *len, DWORD *count)
// {
//     SecKeychainSearchRef search;
//     SecKeychainItemRef item;
//     int status;
//     Boolean saved_user_interaction_allowed;
//     DWORD ret;
//
//     SecKeychainGetUserInteractionAllowed(&saved_user_interaction_allowed);
//     SecKeychainSetUserInteractionAllowed(false);
//
//     status = SecKeychainSearchCreateFromAttributes(NULL, kSecGenericPasswordItemClass, NULL, &search);
//     if (status == noErr)
//     {
//         while (SecKeychainSearchCopyNext(search, &item) == noErr)
//         {
//             SecKeychainAttributeInfo info;
//             SecKeychainAttributeList *attr_list;
//             UInt32 info_tags[] = { kSecServiceItemAttr };
//             BOOL match;
//
//             info.count = sizeof(info_tags)/sizeof(info_tags[0]);
//             info.tag = info_tags;
//             info.format = NULL;
//             status = SecKeychainItemCopyAttributesAndData(item, &info, NULL, &attr_list, NULL, NULL);
//             if (status != noErr)
//             {
//                 WARN("SecKeychainItemCopyAttributesAndData returned status %d\n", status);
//                 continue;
//             }
//             if (buffer)
//             {
//                 *len = sizeof(CREDENTIALW);
//                 credentials[*count] = (PCREDENTIALW)buffer;
//             }
//             else
//                 *len += sizeof(CREDENTIALW);
//             if (attr_list->count != 1 || attr_list->attr[0].tag != kSecServiceItemAttr)
//             {
//                 SecKeychainItemFreeAttributesAndData(attr_list, NULL);
//                 continue;
//             }
//             TRACE("service item: %.*s\n", (int)attr_list->attr[0].length, (char *)attr_list->attr[0].data);
//             match = mac_credential_matches_filter(attr_list->attr[0].data, attr_list->attr[0].length, filter);
//             SecKeychainItemFreeAttributesAndData(attr_list, NULL);
//             if (!match) continue;
//             ret = mac_read_credential_from_item(item, FALSE,
//                                                 buffer ? credentials[*count] : NULL,
//                                                 buffer ? buffer + sizeof(CREDENTIALW) : NULL,
//                                                 len);
//             CFRelease(item);
//             if (ret == ERROR_SUCCESS)
//             {
//                 (*count)++;
//                 if (buffer) buffer += *len;
//             }
//         }
//         CFRelease(search);
//     }
//     else
//         ERR("SecKeychainSearchCreateFromAttributes returned status %d\n", status);
//     SecKeychainSetUserInteractionAllowed(saved_user_interaction_allowed);
//     return ERROR_SUCCESS;
// }
//
// static DWORD mac_delete_credential(LPCWSTR TargetName)
// {
//     int status;
//     SecKeychainSearchRef search;
//     status = SecKeychainSearchCreateFromAttributes(NULL, kSecGenericPasswordItemClass, NULL, &search);
//     if (status == noErr)
//     {
//         SecKeychainItemRef item;
//         while (SecKeychainSearchCopyNext(search, &item) == noErr)
//         {
//             SecKeychainAttributeInfo info;
//             SecKeychainAttributeList *attr_list;
//             UInt32 info_tags[] = { kSecServiceItemAttr };
//             LPWSTR target_name;
//             INT str_len;
//             info.count = sizeof(info_tags)/sizeof(info_tags[0]);
//             info.tag = info_tags;
//             info.format = NULL;
//             status = SecKeychainItemCopyAttributesAndData(item, &info, NULL, &attr_list, NULL, NULL);
//             if (status != noErr)
//             {
//                 WARN("SecKeychainItemCopyAttributesAndData returned status %d\n", status);
//                 continue;
//             }
//             if (attr_list->count != 1 || attr_list->attr[0].tag != kSecServiceItemAttr)
//             {
//                 CFRelease(item);
//                 continue;
//             }
//             str_len = MultiByteToWideChar(CP_UTF8, 0, attr_list->attr[0].data, attr_list->attr[0].length, NULL, 0);
//             target_name = heap_alloc((str_len + 1) * sizeof(WCHAR));
//             MultiByteToWideChar(CP_UTF8, 0, attr_list->attr[0].data, attr_list->attr[0].length, target_name, str_len);
//             /* nul terminate */
//             target_name[str_len] = '\0';
//             if (strcmpiW(TargetName, target_name))
//             {
//                 CFRelease(item);
//                 heap_free(target_name);
//                 continue;
//             }
//             heap_free(target_name);
//             SecKeychainItemFreeAttributesAndData(attr_list, NULL);
//             SecKeychainItemDelete(item);
//             CFRelease(item);
//             CFRelease(search);
//
//             return ERROR_SUCCESS;
//         }
//         CFRelease(search);
//     }
//     return ERROR_NOT_FOUND;
// }
// #endif
//
// /******************************************************************************
//  * convert_PCREDENTIALW_to_PCREDENTIALA [internal]
//  *
//  * convert a Credential struct from UNICODE to ANSI and return the needed size in Bytes
//  *
//  */
//
// static INT convert_PCREDENTIALW_to_PCREDENTIALA(const CREDENTIALW *CredentialW, PCREDENTIALA CredentialA, DWORD len)
// {
//     char *buffer;
//     INT string_len;
//     INT needed = sizeof(CREDENTIALA);
//
//     if (!CredentialA)
//     {
//         if (CredentialW->TargetName)
//             needed += WideCharToMultiByte(CP_ACP, 0, CredentialW->TargetName, -1, NULL, 0, NULL, NULL);
//         if (CredentialW->Comment)
//             needed += WideCharToMultiByte(CP_ACP, 0, CredentialW->Comment, -1, NULL, 0, NULL, NULL);
//         needed += CredentialW->CredentialBlobSize;
//         if (CredentialW->TargetAlias)
//             needed += WideCharToMultiByte(CP_ACP, 0, CredentialW->TargetAlias, -1, NULL, 0, NULL, NULL);
//         if (CredentialW->UserName)
//             needed += WideCharToMultiByte(CP_ACP, 0, CredentialW->UserName, -1, NULL, 0, NULL, NULL);
//
//         return needed;
//     }
//
//
//     buffer = (char *)CredentialA + sizeof(CREDENTIALA);
//     len -= sizeof(CREDENTIALA);
//     CredentialA->Flags = CredentialW->Flags;
//     CredentialA->Type = CredentialW->Type;
//
//     if (CredentialW->TargetName)
//     {
//         CredentialA->TargetName = buffer;
//         string_len = WideCharToMultiByte(CP_ACP, 0, CredentialW->TargetName, -1, buffer, len, NULL, NULL);
//         buffer += string_len;
//         needed += string_len;
//         len -= string_len;
//     }
//     else
//         CredentialA->TargetName = NULL;
//     if (CredentialW->Comment)
//     {
//         CredentialA->Comment = buffer;
//         string_len = WideCharToMultiByte(CP_ACP, 0, CredentialW->Comment, -1, buffer, len, NULL, NULL);
//         buffer += string_len;
//         needed += string_len;
//         len -= string_len;
//     }
//     else
//         CredentialA->Comment = NULL;
//     CredentialA->LastWritten = CredentialW->LastWritten;
//     CredentialA->CredentialBlobSize = CredentialW->CredentialBlobSize;
//     if (CredentialW->CredentialBlobSize && (CredentialW->CredentialBlobSize <= len))
//     {
//         CredentialA->CredentialBlob =(LPBYTE)buffer;
//         memcpy(CredentialA->CredentialBlob, CredentialW->CredentialBlob,
//                CredentialW->CredentialBlobSize);
//         buffer += CredentialW->CredentialBlobSize;
//         needed += CredentialW->CredentialBlobSize;
//         len -= CredentialW->CredentialBlobSize;
//     }
//     else
//         CredentialA->CredentialBlob = NULL;
//     CredentialA->Persist = CredentialW->Persist;
//     CredentialA->AttributeCount = 0;
//     CredentialA->Attributes = NULL; /* FIXME */
//     if (CredentialW->TargetAlias)
//     {
//         CredentialA->TargetAlias = buffer;
//         string_len = WideCharToMultiByte(CP_ACP, 0, CredentialW->TargetAlias, -1, buffer, len, NULL, NULL);
//         buffer += string_len;
//         needed += string_len;
//         len -= string_len;
//     }
//     else
//         CredentialA->TargetAlias = NULL;
//     if (CredentialW->UserName)
//     {
//         CredentialA->UserName = buffer;
//         string_len = WideCharToMultiByte(CP_ACP, 0, CredentialW->UserName, -1, buffer, len, NULL, NULL);
//         needed += string_len;
//     }
//     else
//         CredentialA->UserName = NULL;
//
//     return needed;
// }
//
// /******************************************************************************
//  * convert_PCREDENTIALA_to_PCREDENTIALW [internal]
//  *
//  * convert a Credential struct from ANSI to UNICODE and return the needed size in Bytes
//  *
//  */
// static INT convert_PCREDENTIALA_to_PCREDENTIALW(const CREDENTIALA *CredentialA, PCREDENTIALW CredentialW, INT len)
// {
//     char *buffer;
//     INT string_len;
//     INT needed = sizeof(CREDENTIALW);
//
//     if (!CredentialW)
//     {
//         if (CredentialA->TargetName)
//             needed += sizeof(WCHAR) * MultiByteToWideChar(CP_ACP, 0, CredentialA->TargetName, -1, NULL, 0);
//         if (CredentialA->Comment)
//             needed += sizeof(WCHAR) * MultiByteToWideChar(CP_ACP, 0, CredentialA->Comment, -1, NULL, 0);
//         needed += CredentialA->CredentialBlobSize;
//         if (CredentialA->TargetAlias)
//             needed += sizeof(WCHAR) * MultiByteToWideChar(CP_ACP, 0, CredentialA->TargetAlias, -1, NULL, 0);
//         if (CredentialA->UserName)
//             needed += sizeof(WCHAR) * MultiByteToWideChar(CP_ACP, 0, CredentialA->UserName, -1, NULL, 0);
//
//         return needed;
//     }
//
//     buffer = (char *)CredentialW + sizeof(CREDENTIALW);
//     len -= sizeof(CREDENTIALW);
//     CredentialW->Flags = CredentialA->Flags;
//     CredentialW->Type = CredentialA->Type;
//     if (CredentialA->TargetName)
//     {
//         CredentialW->TargetName = (LPWSTR)buffer;
//         string_len = MultiByteToWideChar(CP_ACP, 0, CredentialA->TargetName, -1, CredentialW->TargetName, len / sizeof(WCHAR));
//         buffer += sizeof(WCHAR) * string_len;
//         needed += sizeof(WCHAR) * string_len;
//         len -= sizeof(WCHAR) * string_len;
//     }
//     else
//         CredentialW->TargetName = NULL;
//     if (CredentialA->Comment)
//     {
//         CredentialW->Comment = (LPWSTR)buffer;
//         string_len = MultiByteToWideChar(CP_ACP, 0, CredentialA->Comment, -1, CredentialW->Comment, len / sizeof(WCHAR));
//         buffer += sizeof(WCHAR) * string_len;
//         needed += sizeof(WCHAR) * string_len;
//         len -= sizeof(WCHAR) * string_len;
//     }
//     else
//         CredentialW->Comment = NULL;
//     CredentialW->LastWritten = CredentialA->LastWritten;
//     CredentialW->CredentialBlobSize = CredentialA->CredentialBlobSize;
//     if (CredentialA->CredentialBlobSize)
//     {
//         CredentialW->CredentialBlob =(LPBYTE)buffer;
//         memcpy(CredentialW->CredentialBlob, CredentialA->CredentialBlob,
//                CredentialA->CredentialBlobSize);
//         buffer += CredentialA->CredentialBlobSize;
//         needed += CredentialA->CredentialBlobSize;
//         len -= CredentialA->CredentialBlobSize;
//     }
//     else
//         CredentialW->CredentialBlob = NULL;
//     CredentialW->Persist = CredentialA->Persist;
//     CredentialW->AttributeCount = 0;
//     CredentialW->Attributes = NULL; /* FIXME */
//     if (CredentialA->TargetAlias)
//     {
//         CredentialW->TargetAlias = (LPWSTR)buffer;
//         string_len = MultiByteToWideChar(CP_ACP, 0, CredentialA->TargetAlias, -1, CredentialW->TargetAlias, len / sizeof(WCHAR));
//         buffer += sizeof(WCHAR) * string_len;
//         needed += sizeof(WCHAR) * string_len;
//         len -= sizeof(WCHAR) * string_len;
//     }
//     else
//         CredentialW->TargetAlias = NULL;
//     if (CredentialA->UserName)
//     {
//         CredentialW->UserName = (LPWSTR)buffer;
//         string_len = MultiByteToWideChar(CP_ACP, 0, CredentialA->UserName, -1, CredentialW->UserName, len / sizeof(WCHAR));
//         needed += sizeof(WCHAR) * string_len;
//     }
//     else
//         CredentialW->UserName = NULL;
//
//     return needed;
// }
//
// /******************************************************************************
//  * CredDeleteA [ADVAPI32.@]
//  */
// BOOL WINAPI CredDeleteA(LPCSTR TargetName, DWORD Type, DWORD Flags)
// {
//     LPWSTR TargetNameW;
//     DWORD len;
//     BOOL ret;
//
//     TRACE("(%s, %d, 0x%x)\n", debugstr_a(TargetName), Type, Flags);
//
//     if (!TargetName)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     len = MultiByteToWideChar(CP_ACP, 0, TargetName, -1, NULL, 0);
//     TargetNameW = heap_alloc(len * sizeof(WCHAR));
//     if (!TargetNameW)
//     {
//         SetLastError(ERROR_OUTOFMEMORY);
//         return FALSE;
//     }
//     MultiByteToWideChar(CP_ACP, 0, TargetName, -1, TargetNameW, len);
//
//     ret = CredDeleteW(TargetNameW, Type, Flags);
//
//     heap_free(TargetNameW);
//
//     return ret;
// }
//
// /******************************************************************************
//  * CredDeleteW [ADVAPI32.@]
//  */
// BOOL WINAPI CredDeleteW(LPCWSTR TargetName, DWORD Type, DWORD Flags)
// {
//     HKEY hkeyMgr;
//     DWORD ret;
//     LPWSTR key_name;
//
//     TRACE("(%s, %d, 0x%x)\n", debugstr_w(TargetName), Type, Flags);
//
//     if (!TargetName)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (Type != CRED_TYPE_GENERIC && Type != CRED_TYPE_DOMAIN_PASSWORD)
//     {
//         FIXME("unhandled type %d\n", Type);
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (Flags)
//     {
//         FIXME("unhandled flags 0x%x\n", Flags);
//         SetLastError(ERROR_INVALID_FLAGS);
//         return FALSE;
//     }
//
// #ifdef __APPLE__
//     if (Type == CRED_TYPE_DOMAIN_PASSWORD)
//     {
//         ret = mac_delete_credential(TargetName);
//         if (ret == ERROR_SUCCESS)
//             return TRUE;
//     }
// #endif
//
//     ret = open_cred_mgr_key(&hkeyMgr, TRUE);
//     if (ret != ERROR_SUCCESS)
//     {
//         WARN("couldn't open/create manager key, error %d\n", ret);
//         SetLastError(ERROR_NO_SUCH_LOGON_SESSION);
//         return FALSE;
//     }
//
//     key_name = get_key_name_for_target(TargetName, Type);
//     ret = RegDeleteKeyW(hkeyMgr, key_name);
//     heap_free(key_name);
//     RegCloseKey(hkeyMgr);
//     if (ret != ERROR_SUCCESS)
//     {
//         SetLastError(ERROR_NOT_FOUND);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /******************************************************************************
//  * CredEnumerateA [ADVAPI32.@]
//  */
// BOOL WINAPI CredEnumerateA(LPCSTR Filter, DWORD Flags, DWORD *Count,
//                            PCREDENTIALA **Credentials)
// {
//     LPWSTR FilterW;
//     PCREDENTIALW *CredentialsW;
//     DWORD i;
//     INT len;
//     INT needed;
//     char *buffer;
//
//     TRACE("(%s, 0x%x, %p, %p)\n", debugstr_a(Filter), Flags, Count, Credentials);
//
//     if (Filter)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, Filter, -1, NULL, 0);
//         FilterW = heap_alloc(len * sizeof(WCHAR));
//         if (!FilterW)
//         {
//             SetLastError(ERROR_OUTOFMEMORY);
//             return FALSE;
//         }
//         MultiByteToWideChar(CP_ACP, 0, Filter, -1, FilterW, len);
//     }
//     else
//         FilterW = NULL;
//
//     if (!CredEnumerateW(FilterW, Flags, Count, &CredentialsW))
//     {
//         heap_free(FilterW);
//         return FALSE;
//     }
//     heap_free(FilterW);
//
//     len = *Count * sizeof(PCREDENTIALA);
//     for (i = 0; i < *Count; i++)
//         len += convert_PCREDENTIALW_to_PCREDENTIALA(CredentialsW[i], NULL, 0);
//
//     *Credentials = heap_alloc(len);
//     if (!*Credentials)
//     {
//         CredFree(CredentialsW);
//         SetLastError(ERROR_OUTOFMEMORY);
//         return FALSE;
//     }
//
//     buffer = (char *)&(*Credentials)[*Count];
//     len -= *Count * sizeof(PCREDENTIALA);
//     for (i = 0; i < *Count; i++)
//     {
//         (*Credentials)[i] = (PCREDENTIALA)buffer;
//         needed = convert_PCREDENTIALW_to_PCREDENTIALA(CredentialsW[i], (*Credentials)[i], len);
//         buffer += needed;
//         len -= needed;
//     }
//
//     CredFree(CredentialsW);
//
//     return TRUE;
// }
//
// /******************************************************************************
//  * CredEnumerateW [ADVAPI32.@]
//  */
// BOOL WINAPI CredEnumerateW(LPCWSTR Filter, DWORD Flags, DWORD *Count,
//                            PCREDENTIALW **Credentials)
// {
//     HKEY hkeyMgr;
//     DWORD ret;
//     LPWSTR target_name;
//     DWORD target_name_len;
//     DWORD len;
//     char *buffer;
//     BYTE key_data[KEY_SIZE];
//
//     TRACE("(%s, 0x%x, %p, %p)\n", debugstr_w(Filter), Flags, Count, Credentials);
//
//     if (Flags)
//     {
//         SetLastError(ERROR_INVALID_FLAGS);
//         return FALSE;
//     }
//
//     ret = open_cred_mgr_key(&hkeyMgr, FALSE);
//     if (ret != ERROR_SUCCESS)
//     {
//         WARN("couldn't open/create manager key, error %d\n", ret);
//         SetLastError(ERROR_NO_SUCH_LOGON_SESSION);
//         return FALSE;
//     }
//
//     ret = get_cred_mgr_encryption_key(hkeyMgr, key_data);
//     if (ret != ERROR_SUCCESS)
//     {
//         RegCloseKey(hkeyMgr);
//         SetLastError(ret);
//         return FALSE;
//     }
//
//     ret = RegQueryInfoKeyW(hkeyMgr, NULL, NULL, NULL, NULL, &target_name_len, NULL, NULL, NULL, NULL, NULL, NULL);
//     if (ret != ERROR_SUCCESS)
//     {
//         RegCloseKey(hkeyMgr);
//         SetLastError(ret);
//         return FALSE;
//     }
//
//     target_name = heap_alloc((target_name_len+1)*sizeof(WCHAR));
//     if (!target_name)
//     {
//         RegCloseKey(hkeyMgr);
//         SetLastError(ERROR_OUTOFMEMORY);
//         return FALSE;
//     }
//
//     *Count = 0;
//     len = 0;
//     ret = registry_enumerate_credentials(hkeyMgr, Filter, target_name, target_name_len,
//                                          key_data, NULL, NULL, &len, Count);
// #ifdef __APPLE__
//     if (ret == ERROR_SUCCESS)
//         ret = mac_enumerate_credentials(Filter, NULL, NULL, &len, Count);
// #endif
//     if (ret == ERROR_SUCCESS && *Count == 0)
//         ret = ERROR_NOT_FOUND;
//     if (ret != ERROR_SUCCESS)
//     {
//         heap_free(target_name);
//         RegCloseKey(hkeyMgr);
//         SetLastError(ret);
//         return FALSE;
//     }
//     len += *Count * sizeof(PCREDENTIALW);
//
//     if (ret == ERROR_SUCCESS)
//     {
//         buffer = heap_alloc(len);
//         *Credentials = (PCREDENTIALW *)buffer;
//         if (buffer)
//         {
//             buffer += *Count * sizeof(PCREDENTIALW);
//             *Count = 0;
//             ret = registry_enumerate_credentials(hkeyMgr, Filter, target_name,
//                                                  target_name_len, key_data,
//                                                  *Credentials, &buffer, &len,
//                                                  Count);
// #ifdef __APPLE__
//             if (ret == ERROR_SUCCESS)
//                 ret = mac_enumerate_credentials(Filter, *Credentials,
//                                                 buffer, &len, Count);
// #endif
//         }
//         else
//             ret = ERROR_OUTOFMEMORY;
//     }
//
//     heap_free(target_name);
//     RegCloseKey(hkeyMgr);
//
//     if (ret != ERROR_SUCCESS)
//     {
//         SetLastError(ret);
//         return FALSE;
//     }
//     return TRUE;
// }
//
// /******************************************************************************
//  * CredFree [ADVAPI32.@]
//  */
// VOID WINAPI CredFree(PVOID Buffer)
// {
//     heap_free(Buffer);
// }
//
// /******************************************************************************
//  * CredReadA [ADVAPI32.@]
//  */
// BOOL WINAPI CredReadA(LPCSTR TargetName, DWORD Type, DWORD Flags, PCREDENTIALA *Credential)
// {
//     LPWSTR TargetNameW;
//     PCREDENTIALW CredentialW;
//     INT len;
//
//     TRACE("(%s, %d, 0x%x, %p)\n", debugstr_a(TargetName), Type, Flags, Credential);
//
//     if (!TargetName)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     len = MultiByteToWideChar(CP_ACP, 0, TargetName, -1, NULL, 0);
//     TargetNameW = heap_alloc(len * sizeof(WCHAR));
//     if (!TargetNameW)
//     {
//         SetLastError(ERROR_OUTOFMEMORY);
//         return FALSE;
//     }
//     MultiByteToWideChar(CP_ACP, 0, TargetName, -1, TargetNameW, len);
//
//     if (!CredReadW(TargetNameW, Type, Flags, &CredentialW))
//     {
//         heap_free(TargetNameW);
//         return FALSE;
//     }
//     heap_free(TargetNameW);
//
//     len = convert_PCREDENTIALW_to_PCREDENTIALA(CredentialW, NULL, 0);
//     *Credential = heap_alloc(len);
//     if (!*Credential)
//     {
//         SetLastError(ERROR_OUTOFMEMORY);
//         return FALSE;
//     }
//     convert_PCREDENTIALW_to_PCREDENTIALA(CredentialW, *Credential, len);
//
//     CredFree(CredentialW);
//
//     return TRUE;
// }
//
// /******************************************************************************
//  * CredReadW [ADVAPI32.@]
//  */
// BOOL WINAPI CredReadW(LPCWSTR TargetName, DWORD Type, DWORD Flags, PCREDENTIALW *Credential)
// {
//     HKEY hkeyMgr;
//     HKEY hkeyCred;
//     DWORD ret;
//     LPWSTR key_name;
//     DWORD len;
//     BYTE key_data[KEY_SIZE];
//
//     TRACE("(%s, %d, 0x%x, %p)\n", debugstr_w(TargetName), Type, Flags, Credential);
//
//     if (!TargetName)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (Type != CRED_TYPE_GENERIC && Type != CRED_TYPE_DOMAIN_PASSWORD)
//     {
//         FIXME("unhandled type %d\n", Type);
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (Flags)
//     {
//         FIXME("unhandled flags 0x%x\n", Flags);
//         SetLastError(ERROR_INVALID_FLAGS);
//         return FALSE;
//     }
//
// #ifdef __APPLE__
//     if (Type == CRED_TYPE_DOMAIN_PASSWORD)
//     {
//         int status;
//         SecKeychainSearchRef search;
//         status = SecKeychainSearchCreateFromAttributes(NULL, kSecGenericPasswordItemClass, NULL, &search);
//         if (status == noErr)
//         {
//             SecKeychainItemRef item;
//             while (SecKeychainSearchCopyNext(search, &item) == noErr)
//             {
//                 SecKeychainAttributeInfo info;
//                 SecKeychainAttributeList *attr_list;
//                 UInt32 info_tags[] = { kSecServiceItemAttr };
//                 LPWSTR target_name;
//                 INT str_len;
//                 info.count = sizeof(info_tags)/sizeof(info_tags[0]);
//                 info.tag = info_tags;
//                 info.format = NULL;
//                 status = SecKeychainItemCopyAttributesAndData(item, &info, NULL, &attr_list, NULL, NULL);
//                 len = sizeof(**Credential);
//                 if (status != noErr)
//                 {
//                     WARN("SecKeychainItemCopyAttributesAndData returned status %d\n", status);
//                     continue;
//                 }
//                 if (attr_list->count != 1 || attr_list->attr[0].tag != kSecServiceItemAttr)
//                 {
//                     CFRelease(item);
//                     continue;
//                 }
//                 str_len = MultiByteToWideChar(CP_UTF8, 0, attr_list->attr[0].data, attr_list->attr[0].length, NULL, 0);
//                 target_name = heap_alloc((str_len + 1) * sizeof(WCHAR));
//                 MultiByteToWideChar(CP_UTF8, 0, attr_list->attr[0].data, attr_list->attr[0].length, target_name, str_len);
//                 /* nul terminate */
//                 target_name[str_len] = '\0';
//                 if (strcmpiW(TargetName, target_name))
//                 {
//                     CFRelease(item);
//                     heap_free(target_name);
//                     continue;
//                 }
//                 heap_free(target_name);
//                 SecKeychainItemFreeAttributesAndData(attr_list, NULL);
//                 ret = mac_read_credential_from_item(item, TRUE, NULL, NULL, &len);
//                 if (ret == ERROR_SUCCESS)
//                 {
//                     *Credential = heap_alloc(len);
//                     if (*Credential)
//                     {
//                         len = sizeof(**Credential);
//                         ret = mac_read_credential_from_item(item, TRUE, *Credential,
//                                                             (char *)(*Credential + 1), &len);
//                     }
//                     else
//                         ret = ERROR_OUTOFMEMORY;
//                     CFRelease(item);
//                     CFRelease(search);
//                     if (ret != ERROR_SUCCESS)
//                     {
//                         SetLastError(ret);
//                         return FALSE;
//                     }
//                     return TRUE;
//                 }
//                 CFRelease(item);
//             }
//             CFRelease(search);
//         }
//     }
// #endif
//
//     ret = open_cred_mgr_key(&hkeyMgr, FALSE);
//     if (ret != ERROR_SUCCESS)
//     {
//         WARN("couldn't open/create manager key, error %d\n", ret);
//         SetLastError(ERROR_NO_SUCH_LOGON_SESSION);
//         return FALSE;
//     }
//
//     ret = get_cred_mgr_encryption_key(hkeyMgr, key_data);
//     if (ret != ERROR_SUCCESS)
//     {
//         RegCloseKey(hkeyMgr);
//         SetLastError(ret);
//         return FALSE;
//     }
//
//     key_name = get_key_name_for_target(TargetName, Type);
//     ret = RegOpenKeyExW(hkeyMgr, key_name, 0, KEY_QUERY_VALUE, &hkeyCred);
//     heap_free(key_name);
//     if (ret != ERROR_SUCCESS)
//     {
//         TRACE("credentials for target name %s not found\n", debugstr_w(TargetName));
//         SetLastError(ERROR_NOT_FOUND);
//         return FALSE;
//     }
//
//     len = sizeof(**Credential);
//     ret = registry_read_credential(hkeyCred, NULL, key_data, NULL, &len);
//     if (ret == ERROR_SUCCESS)
//     {
//         *Credential = heap_alloc(len);
//         if (*Credential)
//         {
//             len = sizeof(**Credential);
//             ret = registry_read_credential(hkeyCred, *Credential, key_data,
//                                            (char *)(*Credential + 1), &len);
//         }
//         else
//             ret = ERROR_OUTOFMEMORY;
//     }
//
//     RegCloseKey(hkeyCred);
//     RegCloseKey(hkeyMgr);
//
//     if (ret != ERROR_SUCCESS)
//     {
//         SetLastError(ret);
//         return FALSE;
//     }
//     return TRUE;
// }
//
// /******************************************************************************
//  * CredReadDomainCredentialsA [ADVAPI32.@]
//  */
// BOOL WINAPI CredReadDomainCredentialsA(PCREDENTIAL_TARGET_INFORMATIONA TargetInformation,
//                                        DWORD Flags, DWORD *Size, PCREDENTIALA **Credentials)
// {
//     PCREDENTIAL_TARGET_INFORMATIONW TargetInformationW;
//     INT len;
//     DWORD i;
//     WCHAR *buffer, *end;
//     BOOL ret;
//     PCREDENTIALW* CredentialsW;
//
//     TRACE("(%p, 0x%x, %p, %p)\n", TargetInformation, Flags, Size, Credentials);
//
//     /* follow Windows behavior - do not test for NULL, initialize early */
//     *Size = 0;
//     *Credentials = NULL;
//
//     if (!TargetInformation)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     len = sizeof(*TargetInformationW);
//     if (TargetInformation->TargetName)
//         len += MultiByteToWideChar(CP_ACP, 0, TargetInformation->TargetName, -1, NULL, 0) * sizeof(WCHAR);
//     if (TargetInformation->NetbiosServerName)
//         len += MultiByteToWideChar(CP_ACP, 0, TargetInformation->NetbiosServerName, -1, NULL, 0) * sizeof(WCHAR);
//     if (TargetInformation->DnsServerName)
//         len += MultiByteToWideChar(CP_ACP, 0, TargetInformation->DnsServerName, -1, NULL, 0) * sizeof(WCHAR);
//     if (TargetInformation->NetbiosDomainName)
//         len += MultiByteToWideChar(CP_ACP, 0, TargetInformation->NetbiosDomainName, -1, NULL, 0) * sizeof(WCHAR);
//     if (TargetInformation->DnsDomainName)
//         len += MultiByteToWideChar(CP_ACP, 0, TargetInformation->DnsDomainName, -1, NULL, 0) * sizeof(WCHAR);
//     if (TargetInformation->DnsTreeName)
//         len += MultiByteToWideChar(CP_ACP, 0, TargetInformation->DnsTreeName, -1, NULL, 0) * sizeof(WCHAR);
//     if (TargetInformation->PackageName)
//         len += MultiByteToWideChar(CP_ACP, 0, TargetInformation->PackageName, -1, NULL, 0) * sizeof(WCHAR);
//
//     TargetInformationW = heap_alloc(len);
//     if (!TargetInformationW)
//     {
//         SetLastError(ERROR_OUTOFMEMORY);
//         return FALSE;
//     }
//     buffer = (WCHAR*)(TargetInformationW + 1);
//     end = (WCHAR *)((char *)TargetInformationW + len);
//
//     if (TargetInformation->TargetName)
//     {
//         TargetInformationW->TargetName = buffer;
//         buffer += MultiByteToWideChar(CP_ACP, 0, TargetInformation->TargetName, -1,
//                                       TargetInformationW->TargetName, end - buffer);
//     } else
//         TargetInformationW->TargetName = NULL;
//
//     if (TargetInformation->NetbiosServerName)
//     {
//         TargetInformationW->NetbiosServerName = buffer;
//         buffer += MultiByteToWideChar(CP_ACP, 0, TargetInformation->NetbiosServerName, -1,
//                                       TargetInformationW->NetbiosServerName, end - buffer);
//     } else
//         TargetInformationW->NetbiosServerName = NULL;
//
//     if (TargetInformation->DnsServerName)
//     {
//         TargetInformationW->DnsServerName = buffer;
//         buffer += MultiByteToWideChar(CP_ACP, 0, TargetInformation->DnsServerName, -1,
//                                       TargetInformationW->DnsServerName, end - buffer);
//     } else
//         TargetInformationW->DnsServerName = NULL;
//
//     if (TargetInformation->NetbiosDomainName)
//     {
//         TargetInformationW->NetbiosDomainName = buffer;
//         buffer += MultiByteToWideChar(CP_ACP, 0, TargetInformation->NetbiosDomainName, -1,
//                                       TargetInformationW->NetbiosDomainName, end - buffer);
//     } else
//         TargetInformationW->NetbiosDomainName = NULL;
//
//     if (TargetInformation->DnsDomainName)
//     {
//         TargetInformationW->DnsDomainName = buffer;
//         buffer += MultiByteToWideChar(CP_ACP, 0, TargetInformation->DnsDomainName, -1,
//                                       TargetInformationW->DnsDomainName, end - buffer);
//     } else
//         TargetInformationW->DnsDomainName = NULL;
//
//     if (TargetInformation->DnsTreeName)
//     {
//         TargetInformationW->DnsTreeName = buffer;
//         buffer += MultiByteToWideChar(CP_ACP, 0, TargetInformation->DnsTreeName, -1,
//                                       TargetInformationW->DnsTreeName, end - buffer);
//     } else
//         TargetInformationW->DnsTreeName = NULL;
//
//     if (TargetInformation->PackageName)
//     {
//         TargetInformationW->PackageName = buffer;
//         MultiByteToWideChar(CP_ACP, 0, TargetInformation->PackageName, -1,
//                             TargetInformationW->PackageName, end - buffer);
//     } else
//         TargetInformationW->PackageName = NULL;
//
//     TargetInformationW->Flags = TargetInformation->Flags;
//     TargetInformationW->CredTypeCount = TargetInformation->CredTypeCount;
//     TargetInformationW->CredTypes = TargetInformation->CredTypes;
//
//     ret = CredReadDomainCredentialsW(TargetInformationW, Flags, Size, &CredentialsW);
//
//     heap_free(TargetInformationW);
//
//     if (ret)
//     {
//         char *buf;
//         INT needed;
//
//         len = *Size * sizeof(PCREDENTIALA);
//         for (i = 0; i < *Size; i++)
//             len += convert_PCREDENTIALW_to_PCREDENTIALA(CredentialsW[i], NULL, 0);
//
//         *Credentials = heap_alloc(len);
//         if (!*Credentials)
//         {
//             CredFree(CredentialsW);
//             SetLastError(ERROR_OUTOFMEMORY);
//             return FALSE;
//         }
//
//         buf = (char *)&(*Credentials)[*Size];
//         len -= *Size * sizeof(PCREDENTIALA);
//         for (i = 0; i < *Size; i++)
//         {
//             (*Credentials)[i] = (PCREDENTIALA)buf;
//             needed = convert_PCREDENTIALW_to_PCREDENTIALA(CredentialsW[i], (*Credentials)[i], len);
//             buf += needed;
//             len -= needed;
//         }
//
//         CredFree(CredentialsW);
//     }
//     return ret;
// }
//
// /******************************************************************************
//  * CredReadDomainCredentialsW [ADVAPI32.@]
//  */
// BOOL WINAPI CredReadDomainCredentialsW(PCREDENTIAL_TARGET_INFORMATIONW TargetInformation, DWORD Flags,
//                                        DWORD *Size, PCREDENTIALW **Credentials)
// {
//     FIXME("(%p, 0x%x, %p, %p) stub\n", TargetInformation, Flags, Size, Credentials);
//
//     /* follow Windows behavior - do not test for NULL, initialize early */
//     *Size = 0;
//     *Credentials = NULL;
//     if (!TargetInformation)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     SetLastError(ERROR_NOT_FOUND);
//     return FALSE;
// }
//
// /******************************************************************************
//  * CredWriteA [ADVAPI32.@]
//  */
// BOOL WINAPI CredWriteA(PCREDENTIALA Credential, DWORD Flags)
// {
//     BOOL ret;
//     INT len;
//     PCREDENTIALW CredentialW;
//
//     TRACE("(%p, 0x%x)\n", Credential, Flags);
//
//     if (!Credential || !Credential->TargetName)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     len = convert_PCREDENTIALA_to_PCREDENTIALW(Credential, NULL, 0);
//     CredentialW = heap_alloc(len);
//     if (!CredentialW)
//     {
//         SetLastError(ERROR_OUTOFMEMORY);
//         return FALSE;
//     }
//
//     convert_PCREDENTIALA_to_PCREDENTIALW(Credential, CredentialW, len);
//
//     ret = CredWriteW(CredentialW, Flags);
//
//     heap_free(CredentialW);
//
//     return ret;
// }
//
// /******************************************************************************
//  * CredWriteW [ADVAPI32.@]
//  */
// BOOL WINAPI CredWriteW(PCREDENTIALW Credential, DWORD Flags)
// {
//     HKEY hkeyMgr;
//     HKEY hkeyCred;
//     DWORD ret;
//     LPWSTR key_name;
//     BYTE key_data[KEY_SIZE];
//
//     TRACE("(%p, 0x%x)\n", Credential, Flags);
//
//     if (!Credential || !Credential->TargetName)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (Flags & ~CRED_PRESERVE_CREDENTIAL_BLOB)
//     {
//         FIXME("unhandled flags 0x%x\n", Flags);
//         SetLastError(ERROR_INVALID_FLAGS);
//         return FALSE;
//     }
//
//     if (Credential->Type != CRED_TYPE_GENERIC && Credential->Type != CRED_TYPE_DOMAIN_PASSWORD)
//     {
//         FIXME("unhandled type %d\n", Credential->Type);
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     TRACE("Credential->Flags = 0x%08x\n", Credential->Flags);
//     TRACE("Credential->Type = %u\n", Credential->Type);
//     TRACE("Credential->TargetName = %s\n", debugstr_w(Credential->TargetName));
//     TRACE("Credential->Comment = %s\n", debugstr_w(Credential->Comment));
//     TRACE("Credential->Persist = %u\n", Credential->Persist);
//     TRACE("Credential->TargetAlias = %s\n", debugstr_w(Credential->TargetAlias));
//     TRACE("Credential->UserName = %s\n", debugstr_w(Credential->UserName));
//
//     if (Credential->Type == CRED_TYPE_DOMAIN_PASSWORD)
//     {
//         if (!Credential->UserName ||
//             (Credential->Persist == CRED_PERSIST_ENTERPRISE &&
//             (!strchrW(Credential->UserName, '\\') && !strchrW(Credential->UserName, '@'))))
//         {
//             ERR("bad username %s\n", debugstr_w(Credential->UserName));
//             SetLastError(ERROR_BAD_USERNAME);
//             return FALSE;
//         }
//     }
//
// #ifdef __APPLE__
//     if (!Credential->AttributeCount &&
//         Credential->Type == CRED_TYPE_DOMAIN_PASSWORD &&
//         (Credential->Persist == CRED_PERSIST_LOCAL_MACHINE || Credential->Persist == CRED_PERSIST_ENTERPRISE))
//     {
//         ret = mac_write_credential(Credential, Flags & CRED_PRESERVE_CREDENTIAL_BLOB);
//         if (ret != ERROR_SUCCESS)
//         {
//             SetLastError(ret);
//             return FALSE;
//         }
//         return TRUE;
//     }
// #endif
//
//     ret = open_cred_mgr_key(&hkeyMgr, FALSE);
//     if (ret != ERROR_SUCCESS)
//     {
//         WARN("couldn't open/create manager key, error %d\n", ret);
//         SetLastError(ERROR_NO_SUCH_LOGON_SESSION);
//         return FALSE;
//     }
//
//     ret = get_cred_mgr_encryption_key(hkeyMgr, key_data);
//     if (ret != ERROR_SUCCESS)
//     {
//         RegCloseKey(hkeyMgr);
//         SetLastError(ret);
//         return FALSE;
//     }
//
//     key_name = get_key_name_for_target(Credential->TargetName, Credential->Type);
//     ret = RegCreateKeyExW(hkeyMgr, key_name, 0, NULL,
//                           Credential->Persist == CRED_PERSIST_SESSION ? REG_OPTION_VOLATILE : REG_OPTION_NON_VOLATILE,
//                           KEY_READ|KEY_WRITE, NULL, &hkeyCred, NULL);
//     heap_free(key_name);
//     if (ret != ERROR_SUCCESS)
//     {
//         TRACE("credentials for target name %s not found\n",
//               debugstr_w(Credential->TargetName));
//         SetLastError(ERROR_NOT_FOUND);
//         return FALSE;
//     }
//
//     ret = registry_write_credential(hkeyCred, Credential, key_data,
//                                     Flags & CRED_PRESERVE_CREDENTIAL_BLOB);
//
//     RegCloseKey(hkeyCred);
//     RegCloseKey(hkeyMgr);
//
//     if (ret != ERROR_SUCCESS)
//     {
//         SetLastError(ret);
//         return FALSE;
//     }
//     return TRUE;
// }
//
// /******************************************************************************
//  * CredGetSessionTypes [ADVAPI32.@]
//  */
// WINADVAPI BOOL WINAPI CredGetSessionTypes(DWORD persistCount, LPDWORD persists)
// {
//     TRACE("(%u, %p)\n", persistCount, persists);
//
//     memset(persists, CRED_PERSIST_NONE, persistCount*sizeof(*persists));
//     if (CRED_TYPE_GENERIC < persistCount)
//     {
//         persists[CRED_TYPE_GENERIC] = CRED_PERSIST_ENTERPRISE;
//
//         if (CRED_TYPE_DOMAIN_PASSWORD < persistCount)
//         {
//             persists[CRED_TYPE_DOMAIN_PASSWORD] = CRED_PERSIST_ENTERPRISE;
//         }
//     }
//     return TRUE;
// }
//
// /******************************************************************************
//  * CredMarshalCredentialA [ADVAPI32.@]
//  */
// BOOL WINAPI CredMarshalCredentialA( CRED_MARSHAL_TYPE type, PVOID cred, LPSTR *out )
// {
//     BOOL ret;
//     WCHAR *outW;
//
//     TRACE("%u, %p, %p\n", type, cred, out);
//
//     if ((ret = CredMarshalCredentialW( type, cred, &outW )))
//     {
//         int len = WideCharToMultiByte( CP_ACP, 0, outW, -1, NULL, 0, NULL, NULL );
//         if (!(*out = heap_alloc( len )))
//         {
//             heap_free( outW );
//             return FALSE;
//         }
//         WideCharToMultiByte( CP_ACP, 0, outW, -1, *out, len, NULL, NULL );
//         heap_free( outW );
//     }
//     return ret;
// }
//
// static UINT cred_encode( const char *bin, unsigned int len, WCHAR *cred )
// {
//     static const char enc[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789#-";
//     UINT n = 0, x;
//
//     while (len > 0)
//     {
//         cred[n++] = enc[bin[0] & 0x3f];
//         x = (bin[0] & 0xc0) >> 6;
//         if (len == 1)
//         {
//             cred[n++] = enc[x];
//             break;
//         }
//         cred[n++] = enc[((bin[1] & 0xf) << 2) | x];
//         x = (bin[1] & 0xf0) >> 4;
//         if (len == 2)
//         {
//             cred[n++] = enc[x];
//             break;
//         }
//         cred[n++] = enc[((bin[2] & 0x3) << 4) | x];
//         cred[n++] = enc[(bin[2] & 0xfc) >> 2];
//         bin += 3;
//         len -= 3;
//     }
//     return n;
// }
//
// /******************************************************************************
//  * CredMarshalCredentialW [ADVAPI32.@]
//  */
// BOOL WINAPI CredMarshalCredentialW( CRED_MARSHAL_TYPE type, PVOID cred, LPWSTR *out )
// {
//     CERT_CREDENTIAL_INFO *cert = cred;
//     USERNAME_TARGET_CREDENTIAL_INFO *target = cred;
//     DWORD len, size;
//     WCHAR *p;
//
//     TRACE("%u, %p, %p\n", type, cred, out);
//
//     if (!cred || (type == CertCredential && cert->cbSize < sizeof(*cert)) ||
//         (type != CertCredential && type != UsernameTargetCredential && type != BinaryBlobCredential) ||
//         (type == UsernameTargetCredential && (!target->UserName || !target->UserName[0])))
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//     switch (type)
//     {
//     case CertCredential:
//     {
//         size = (sizeof(cert->rgbHashOfCert) + 2) * 4 / 3;
//         if (!(p = heap_alloc( (size + 4) * sizeof(WCHAR) ))) return FALSE;
//         p[0] = '@';
//         p[1] = '@';
//         p[2] = 'A' + type;
//         len = cred_encode( (const char *)cert->rgbHashOfCert, sizeof(cert->rgbHashOfCert), p + 3 );
//         p[len + 3] = 0;
//         break;
//     }
//     case UsernameTargetCredential:
//     {
//         len = strlenW( target->UserName );
//         size = (sizeof(DWORD) + len * sizeof(WCHAR) + 2) * 4 / 3;
//         if (!(p = heap_alloc( (size + 4) * sizeof(WCHAR) ))) return FALSE;
//         p[0] = '@';
//         p[1] = '@';
//         p[2] = 'A' + type;
//         size = len * sizeof(WCHAR);
//         len = cred_encode( (const char *)&size, sizeof(DWORD), p + 3 );
//         len += cred_encode( (const char *)target->UserName, size, p + 3 + len );
//         p[len + 3] = 0;
//         break;
//     }
//     case BinaryBlobCredential:
//         FIXME("BinaryBlobCredential not implemented\n");
//         return FALSE;
//     default:
//         return FALSE;
//     }
//     *out = p;
//     return TRUE;
// }
//
// /******************************************************************************
//  * CredUnmarshalCredentialA [ADVAPI32.@]
//  */
// BOOL WINAPI CredUnmarshalCredentialA( LPCSTR cred, PCRED_MARSHAL_TYPE type, PVOID *out )
// {
//     BOOL ret;
//     WCHAR *credW = NULL;
//
//     TRACE("%s, %p, %p\n", debugstr_a(cred), type, out);
//
//     if (cred)
//     {
//         int len = MultiByteToWideChar( CP_ACP, 0, cred, -1, NULL, 0 );
//         if (!(credW = heap_alloc( len * sizeof(WCHAR) ))) return FALSE;
//         MultiByteToWideChar( CP_ACP, 0, cred, -1, credW, len );
//     }
//     ret = CredUnmarshalCredentialW( credW, type, out );
//     heap_free( credW );
//     return ret;
// }
//
// static inline char char_decode( WCHAR c )
// {
//     if (c >= 'A' && c <= 'Z') return c - 'A';
//     if (c >= 'a' && c <= 'z') return c - 'a' + 26;
//     if (c >= '0' && c <= '9') return c - '0' + 52;
//     if (c == '#') return 62;
//     if (c == '-') return 63;
//     return 64;
// }
//
// static BOOL cred_decode( const WCHAR *cred, unsigned int len, char *buf )
// {
//     unsigned int i = 0;
//     char c0, c1, c2, c3;
//     const WCHAR *p = cred;
//
//     while (len >= 4)
//     {
//         if ((c0 = char_decode( p[0] )) > 63) return FALSE;
//         if ((c1 = char_decode( p[1] )) > 63) return FALSE;
//         if ((c2 = char_decode( p[2] )) > 63) return FALSE;
//         if ((c3 = char_decode( p[3] )) > 63) return FALSE;
//
//         buf[i + 0] = (c1 << 6) | c0;
//         buf[i + 1] = (c2 << 4) | (c1 >> 2);
//         buf[i + 2] = (c3 << 2) | (c2 >> 4);
//         len -= 4;
//         i += 3;
//         p += 4;
//     }
//     if (len == 3)
//     {
//         if ((c0 = char_decode( p[0] )) > 63) return FALSE;
//         if ((c1 = char_decode( p[1] )) > 63) return FALSE;
//         if ((c2 = char_decode( p[2] )) > 63) return FALSE;
//
//         buf[i + 0] = (c1 << 6) | c0;
//         buf[i + 1] = (c2 << 4) | (c1 >> 2);
//     }
//     else if (len == 2)
//     {
//         if ((c0 = char_decode( p[0] )) > 63) return FALSE;
//         if ((c1 = char_decode( p[1] )) > 63) return FALSE;
//
//         buf[i + 0] = (c1 << 6) | c0;
//     }
//     else if (len == 1)
//     {
//         return FALSE;
//     }
//     return TRUE;
// }
//
// /******************************************************************************
//  * CredUnmarshalCredentialW [ADVAPI32.@]
//  */
// BOOL WINAPI CredUnmarshalCredentialW( LPCWSTR cred, PCRED_MARSHAL_TYPE type, PVOID *out )
// {
//     unsigned int len, buflen;
//
//     TRACE("%s, %p, %p\n", debugstr_w(cred), type, out);
//
//     if (!cred || cred[0] != '@' || cred[1] != '@' ||
//         char_decode( cred[2] ) > 63)
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//     len = strlenW( cred + 3 );
//     *type = char_decode( cred[2] );
//     switch (*type)
//     {
//     case CertCredential:
//     {
//         char hash[CERT_HASH_LENGTH];
//         CERT_CREDENTIAL_INFO *cert;
//
//         if (len != 27 || !cred_decode( cred + 3, len, hash ))
//         {
//             SetLastError( ERROR_INVALID_PARAMETER );
//             return FALSE;
//         }
//         if (!(cert = heap_alloc( sizeof(*cert) ))) return FALSE;
//         memcpy( cert->rgbHashOfCert, hash, sizeof(cert->rgbHashOfCert) );
//         cert->cbSize = sizeof(*cert);
//         *out = cert;
//         break;
//     }
//     case UsernameTargetCredential:
//     {
//         USERNAME_TARGET_CREDENTIAL_INFO *target;
//         DWORD size;
//
//         if (len < 9 || !cred_decode( cred + 3, 6, (char *)&size ) ||
//             size % sizeof(WCHAR) || len - 6 != (size * 4 + 2) / 3)
//         {
//             SetLastError( ERROR_INVALID_PARAMETER );
//             return FALSE;
//         }
//         buflen = sizeof(*target) + size + sizeof(WCHAR);
//         if (!(target = heap_alloc( buflen ))) return FALSE;
//         if (!cred_decode( cred + 9, len - 6, (char *)(target + 1) ))
//         {
//             heap_free( target );
//             return FALSE;
//         }
//         target->UserName = (WCHAR *)(target + 1);
//         target->UserName[size / sizeof(WCHAR)] = 0;
//         *out = target;
//         break;
//     }
//     case BinaryBlobCredential:
//         FIXME("BinaryBlobCredential not implemented\n");
//         return FALSE;
//     default:
//         WARN("unhandled type %u\n", *type);
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//     return TRUE;
// }
//
// /******************************************************************************
//  * CredIsMarshaledCredentialW [ADVAPI32.@]
//  *
//  * Check, if the name parameter is a marshaled credential, hash or binary blob
//  *
//  * PARAMS
//  *  name    the name to check
//  *
//  * RETURNS
//  *  TRUE:  the name parameter is a marshaled credential, hash or binary blob
//  *  FALSE: the name is a plain username
//  */
// BOOL WINAPI CredIsMarshaledCredentialW(LPCWSTR name)
// {
//     TRACE("(%s)\n", debugstr_w(name));
//
//     if (name && name[0] == '@' && name[1] == '@' && name[2] > 'A' && name[3])
//     {
//         char hash[CERT_HASH_LENGTH];
//         int len = strlenW(name + 3 );
//         DWORD size;
//
//         if ((name[2] - 'A') == CertCredential && (len == 27) && cred_decode(name + 3, len, hash))
//             return TRUE;
//
//         if (((name[2] - 'A') == UsernameTargetCredential) &&
//             (len >= 9) && cred_decode(name + 3, 6, (char *)&size) && size)
//             return TRUE;
//
//         if ((name[2] - 'A') == BinaryBlobCredential)
//             FIXME("BinaryBlobCredential not checked\n");
//
//         if ((name[2] - 'A') > BinaryBlobCredential)
//             TRACE("unknown type: %d\n", (name[2] - 'A'));
//     }
//
//     SetLastError(ERROR_INVALID_PARAMETER);
//     return FALSE;
// }
//
// /******************************************************************************
//  * CredIsMarshaledCredentialA [ADVAPI32.@]
//  *
//  * See CredIsMarshaledCredentialW
//  *
//  */
// BOOL WINAPI CredIsMarshaledCredentialA(LPCSTR name)
// {
//     LPWSTR nameW = NULL;
//     BOOL res;
//     int len;
//
//     TRACE("(%s)\n", debugstr_a(name));
//
//     if (name)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, name, -1, NULL, 0);
//         nameW = heap_alloc(len * sizeof(WCHAR));
//         MultiByteToWideChar(CP_ACP, 0, name, -1, nameW, len);
//     }
//
//     res = CredIsMarshaledCredentialW(nameW);
//     heap_free(nameW);
//     return res;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
