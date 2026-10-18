// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_69A283E639E4: [u16; 3] = [0x0025u16, 0x0075, 0x0000];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DsBindA(
    controller: LPCSTR,
    domain: LPCSTR,
    handle: *mut HANDLE,
) -> DWORD {
    // FIXME("(%s,%s, %p): stub!\n", debugstr_a(controller), debugstr_a(domain), handle);
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn DsBindW(
    controller: LPCWSTR,
    domain: LPCWSTR,
    handle: *mut HANDLE,
) -> DWORD {
    // FIXME("(%s,%s, %p): stub!\n", debugstr_w(controller), debugstr_w(domain), handle);
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn DsMakeSpnW(svc_class: LPCWSTR, svc_name: LPCWSTR, inst_name: LPCWSTR, inst_port: USHORT, ref_: LPCWSTR, spn_length: *mut DWORD, spn: LPWSTR) -> DWORD {
    let mut new_spn_length: DWORD = 0;
    let mut len: INT = 0;
    let mut p: LPWSTR = core::ptr::null_mut();

    if svc_class.is_null() || svc_name.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    new_spn_length = (lstrlenW(svc_class) as DWORD)
        .wrapping_add(1 as DWORD)
        .wrapping_add(1 as DWORD);
    if !inst_name.is_null() {
        new_spn_length = new_spn_length.wrapping_add(lstrlenW(inst_name) as DWORD);
    } else {
        new_spn_length = new_spn_length.wrapping_add(lstrlenW(svc_name) as DWORD);
    }
    if inst_port != 0 {
        let mut n: USHORT = inst_port;
        new_spn_length = new_spn_length.wrapping_add(1 as DWORD);
        loop {
            n = n / (10 as USHORT);
            new_spn_length = new_spn_length.wrapping_add(1);
            if n == (0 as USHORT) {
                break;
            }
        }
    }
    if !inst_name.is_null() {
        new_spn_length = new_spn_length.wrapping_add(
            (1 as DWORD).wrapping_add(lstrlenW(svc_name) as DWORD),
        );
    }

    if *spn_length < new_spn_length {
        *spn_length = new_spn_length;
        return ERROR_BUFFER_OVERFLOW as DWORD;
    }
    *spn_length = new_spn_length;

    p = spn;
    len = lstrlenW(svc_class) as INT;
    memcpy(
        p as *mut core::ffi::c_void,
        svc_class as *const core::ffi::c_void,
        (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    );
    p = p.offset(len as isize);
    *p = '/' as WCHAR;
    p = p.offset(1);
    if !inst_name.is_null() {
        len = lstrlenW(inst_name) as INT;
        memcpy(
            p as *mut core::ffi::c_void,
            inst_name as *const core::ffi::c_void,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        p = p.offset(len as isize);
        *p = 0 as WCHAR;
    } else {
        len = lstrlenW(svc_name) as INT;
        memcpy(
            p as *mut core::ffi::c_void,
            svc_name as *const core::ffi::c_void,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        p = p.offset(len as isize);
        *p = 0 as WCHAR;
    }

    if inst_port != 0 {
        *p = ':' as WCHAR;
        p = p.offset(1);
        wsprintfW(p, RUSTCONV_WSTR_69A283E639E4.as_ptr(), inst_port as core::ffi::c_int);
        p = p.offset(lstrlenW(p) as isize);
    }

    if !inst_name.is_null() {
        *p = '/' as WCHAR;
        p = p.offset(1);
        len = lstrlenW(svc_name) as INT;
        memcpy(
            p as *mut core::ffi::c_void,
            svc_name as *const core::ffi::c_void,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        p = p.offset(len as isize);
        *p = 0 as WCHAR;
    }

    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn DsMakeSpnA(
    svc_class: LPCSTR,
    svc_name: LPCSTR,
    inst_name: LPCSTR,
    inst_port: USHORT,
    r#ref: LPCSTR,
    spn_length: *mut DWORD,
    spn: LPSTR,
) -> DWORD {
    // FIXME("(%s,%s,%s,%d,%s,%p,%p): stub!\n", debugstr_a(svc_class), debugstr_a(svc_name), debugstr_a(inst_name), inst_port, debugstr_a(ref), spn_length, spn);
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn DsGetSpnA(
    ServType: DS_SPN_NAME_TYPE,
    Servlass: LPCSTR,
    ServName: LPCSTR,
    InstPort: USHORT,
    nInstanceNames: USHORT,
    pInstanceNames: *mut LPCSTR,
    pInstancePorts: *const USHORT,
    pSpn: *mut DWORD,
    pszSpn: *mut *mut LPSTR,
) -> DWORD {
    // FIXME("(%d,%s,%s,%d,%d,%p,%p,%p,%p): stub!\n", ServType, debugstr_a(Servlass), debugstr_a(ServName), InstPort, nInstanceNames, pInstanceNames, pInstancePorts, pSpn, pszSpn);
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn DsServerRegisterSpnA(
    operation: DS_SPN_WRITE_OP,
    ServiceClass: LPCSTR,
    UserObjectDN: LPCSTR,
) -> DWORD {
    // FIXME("(%d,%s,%s): stub!\n", operation, debugstr_a(ServiceClass), debugstr_a(UserObjectDN));
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn DsServerRegisterSpnW(
    operation: DS_SPN_WRITE_OP,
    ServiceClass: LPCWSTR,
    UserObjectDN: LPCWSTR,
) -> DWORD {
    // FIXME("(%d,%s,%s): stub!\n", operation, debugstr_w(ServiceClass), debugstr_w(UserObjectDN));
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn DsClientMakeSpnForTargetServerW(
    class: LPCWSTR,
    name: LPCWSTR,
    buflen: *mut DWORD,
    buf: LPWSTR,
) -> DWORD {
    let mut len: DWORD;
    let mut p: *mut WCHAR;

    if class.is_null() || name.is_null() || buflen.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    len = (lstrlenW(class) + 1 + lstrlenW(name) + 1) as DWORD;
    if *buflen < len {
        *buflen = len;
        return ERROR_BUFFER_OVERFLOW;
    }
    *buflen = len;

    memcpy(
        buf as *mut core::ffi::c_void,
        class as *const core::ffi::c_void,
        (lstrlenW(class) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    );
    p = buf.add(lstrlenW(class) as usize);
    let fresh15 = p;
    p = p.add(1);
    *fresh15 = '/' as WCHAR;
    memcpy(
        p as *mut core::ffi::c_void,
        name as *const core::ffi::c_void,
        (lstrlenW(name) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    );
    *buf.add((len - 1) as usize) = 0;

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DsCrackNamesA(
    handle: HANDLE,
    flags: DS_NAME_FLAGS,
    offered: DS_NAME_FORMAT,
    desired: DS_NAME_FORMAT,
    num: DWORD,
    names: *mut *const CHAR,
    result: *mut PDS_NAME_RESULTA,
) -> DWORD {
    // FIXME("(%p %u %u %u %lu %p %p stub\n", handle, flags, offered, desired, num, names, result);
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn DsCrackNamesW(
    handle: HANDLE,
    flags: DS_NAME_FLAGS,
    offered: DS_NAME_FORMAT,
    desired: DS_NAME_FORMAT,
    num: DWORD,
    names: *mut *const WCHAR,
    result: *mut PDS_NAME_RESULTW,
) -> DWORD {
    // FIXME("(%p %u %u %u %lu %p %p stub\n", handle, flags, offered, desired, num, names, result);
    ERROR_CALL_NOT_IMPLEMENTED
}
