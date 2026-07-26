use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// NB: wldap32.h only DECLARES `extern HINSTANCE hwldap32` — main.c DEFINES it, and
// crate::wldap32::main owns the Rust definition. A second `static mut` here would be a
// DIFFERENT object, so main's DllMain write would never be seen by a reader of this one.
pub unsafe extern "C" fn strdupU(src: *const i8) -> *mut i8 {
    let mut dst: *mut i8;
    if src.is_null() {
        return core::ptr::null_mut();
    }
    dst = heap_alloc((strlen(src) + 1) * core::mem::size_of::<i8>()) as *mut i8;
    if !dst.is_null() {
        strcpy(dst, src);
    }
    dst
}
pub unsafe extern "C" fn strdupW(src: LPCWSTR) -> LPWSTR {
    let mut dst: LPWSTR;
    if src.is_null() {
        return core::ptr::null_mut();
    }
    dst = heap_alloc((strlenW(src) + 1) * core::mem::size_of::<u16>()) as LPWSTR;
    if !dst.is_null() {
        strcpyW(dst, src);
    }
    dst
}
pub unsafe extern "C" fn strAtoW(str: LPCSTR) -> LPWSTR {
    let mut ret: LPWSTR = core::ptr::null_mut();
    if !str.is_null() {
        let len: DWORD = (MultiByteToWideChar(CP_ACP, 0, str, -1, core::ptr::null_mut(), 0)) as u32;
        ret = (heap_alloc((len * core::mem::size_of::<u16>() as u32) as usize)).cast();
        if !ret.is_null() {
            MultiByteToWideChar(CP_ACP, 0, str, -1, ret, (len) as i32);
        }
    }
    ret
}
pub unsafe extern "C" fn strWtoA(str: LPCWSTR) -> LPSTR {
    let mut ret: LPSTR = core::ptr::null_mut();
    if !str.is_null() {
        let len: DWORD = (WideCharToMultiByte(CP_ACP, 0, str, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut())) as u32;
        ret = (heap_alloc((len) as usize)).cast();
        if !ret.is_null() {
            WideCharToMultiByte(CP_ACP, 0, str, -1, ret, (len) as i32, core::ptr::null_mut(), core::ptr::null_mut());
        }
    }
    ret
}
pub unsafe extern "C" fn strWtoU(str: LPCWSTR) -> LPSTR {
    let mut ret: LPSTR = core::ptr::null_mut();
    if !str.is_null() {
        let len: DWORD = (WideCharToMultiByte(
            CP_UTF8,
            0,
            str,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )) as u32;
        ret = (heap_alloc((len) as usize)).cast();
        if !ret.is_null() {
            WideCharToMultiByte(
                CP_UTF8,
                0,
                str,
                -1,
                ret,
                (len) as i32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }
    }
    ret
}
pub unsafe extern "C" fn strUtoW(str_: *mut i8) -> LPWSTR {
    let mut ret: LPWSTR = core::ptr::null_mut();
    if !str_.is_null() {
        let len: u32 = (MultiByteToWideChar(CP_UTF8, 0, str_, -1, core::ptr::null_mut(), 0)) as u32;
        ret = heap_alloc((len as usize) * core::mem::size_of::<u16>()) as LPWSTR;
        if !ret.is_null() {
            MultiByteToWideChar(CP_UTF8, 0, str_, -1, ret, (len) as i32);
        }
    }
    ret
}

pub unsafe extern "C" fn strfreeA(str: LPSTR) {
    heap_free((str).cast());
}

pub unsafe extern "C" fn strfreeW(str: LPWSTR) {
    heap_free((str).cast());
}

pub unsafe extern "C" fn strfreeU(str: *mut i8) {
    heap_free((str).cast());
}
pub unsafe extern "C" fn strarraylenA(strarray: *mut LPSTR) -> DWORD {
    let mut p = strarray;
    while !(*p).is_null() {
        p = p.offset(1);
    }
    p.offset_from(strarray) as DWORD
}
pub unsafe extern "C" fn strarraylenW(strarray: *mut LPWSTR) -> DWORD {
    let mut p = strarray;
    while !(*p).is_null() {
        p = p.offset(1);
    }
    p.offset_from(strarray) as DWORD
}
pub unsafe extern "C" fn strarraylenU(strarray: *mut *mut i8) -> u32 {
    let mut p = strarray;
    while !(*p).is_null() {
        p = p.offset(1);
    }
    p.offset_from(strarray) as u32
}
pub unsafe extern "C" fn strarrayAtoW(strarray: *mut LPSTR) -> *mut LPWSTR {
    let mut strarrayW: *mut LPWSTR = core::ptr::null_mut();
    let size: DWORD;

    if !strarray.is_null() {
        size = (core::mem::size_of::<*mut u16>() as DWORD) * (strarraylenA(strarray) + 1);
        strarrayW = (heap_alloc((size) as usize)).cast();
        if !strarrayW.is_null() {
            let mut p: *mut LPSTR = strarray;
            let mut q: *mut LPWSTR = strarrayW;

            while !(*p).is_null() {
                *q = strAtoW(*p);
                q = q.add(1);
                p = p.add(1);
            }
            *q = core::ptr::null_mut();
        }
    }
    strarrayW
}
pub unsafe extern "C" fn strarrayWtoA(strarray: *mut LPWSTR) -> *mut LPSTR {
    let mut strarrayA: *mut LPSTR = core::ptr::null_mut();
    let size: DWORD;

    if !strarray.is_null() {
        size = (core::mem::size_of::<LPSTR>() as DWORD)
            .wrapping_mul(strarraylenW((strarray).cast()).wrapping_add(1));
        strarrayA = (heap_alloc((size) as usize)).cast();
        if !strarrayA.is_null() {
            let mut p: *mut LPWSTR = strarray;
            let mut q: *mut LPSTR = strarrayA;

            while !(*p).is_null() {
                *q = strWtoA(*p);
                q = q.add(1);
                p = p.add(1);
            }
            *q = core::ptr::null_mut();
        }
    }
    strarrayA
}
pub unsafe extern "C" fn strarrayWtoU(strarray: *mut LPWSTR) -> *mut LPSTR {
    let mut strarrayU: *mut LPSTR = core::ptr::null_mut();
    let mut size: DWORD;

    if !strarray.is_null() {
        size = (core::mem::size_of::<*mut CHAR>() as DWORD) * (strarraylenW((strarray).cast()) + 1);
        strarrayU = (heap_alloc((size) as usize)).cast();
        if !strarrayU.is_null() {
            let mut p: *mut LPWSTR = strarray;
            let mut q: *mut LPSTR = strarrayU;

            while !(*p).is_null() {
                *q = strWtoU(*p);
                q = q.offset(1);
                p = p.offset(1);
            }
            *q = core::ptr::null_mut();
        }
    }
    strarrayU
}
pub unsafe extern "C" fn strarrayUtoW(strarray: *mut *mut i8) -> *mut LPWSTR {
    let mut strarrayW: *mut LPWSTR = core::ptr::null_mut();
    let mut size: u32;

    if !strarray.is_null() {
        size = (core::mem::size_of::<*mut u16>() as u32) * (strarraylenU(strarray) + 1);
        strarrayW = heap_alloc(size as usize) as *mut LPWSTR;
        if !strarrayW.is_null() {
            let mut p: *mut *mut i8 = strarray;
            let mut q: *mut LPWSTR = strarrayW;

            while !(*p).is_null() {
                *q = strUtoW(*p);
                q = q.offset(1);
                p = p.offset(1);
            }
            *q = core::ptr::null_mut();
        }
    }
    strarrayW
}

pub unsafe extern "C" fn strarrayfreeA(strarray: *mut LPSTR) {
    if !strarray.is_null() {
        let mut p = strarray;
        while !(*p).is_null() {
            strfreeA(*p);
            p = p.offset(1);
        }
        heap_free(strarray as *mut _);
    }
}
pub unsafe extern "C" fn strarrayfreeW(strarray: *mut LPWSTR) {
    if !strarray.is_null() {
        let mut p = strarray;
        while !(*p).is_null() {
            let current = *p;
            p = p.offset(1);
            strfreeW(current);
        }
        heap_free((strarray).cast());
    }
}
pub unsafe extern "C" fn strarrayfreeU(strarray: *mut *mut i8) {
    if !strarray.is_null() {
        let mut p = strarray;
        while !(*p).is_null() {
            strfreeU(*p);
            p = p.offset(1);
        }
        heap_free(strarray as *mut _);
    }
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn bvdup(bv: *mut berval) -> *mut berval {
    let size: DWORD = (core::mem::size_of::<berval>() as DWORD)
        .wrapping_add((*bv).bv_len);

    let berval = heap_alloc(size);
    if !berval.is_null() {
        let val = (berval as *mut u8)
            .offset(core::mem::size_of::<berval>() as isize) as *mut i8;

        (*berval).bv_len = (*bv).bv_len;
        (*berval).bv_val = val;
        memcpy(val as *mut core::ffi::c_void,
                               (*bv).bv_val as *const core::ffi::c_void,
                               (*bv).bv_len as usize);
    }
    berval
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn bvarraylen(bv: *mut *mut berval) -> DWORD {
    let mut p = bv;
    while !(*p).is_null() {
        p = p.offset(1);
    }
    p.offset_from(bv) as DWORD
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn bvarraydup(bv: *mut *mut berval) -> *mut *mut berval {
    let mut berval: *mut *mut berval = core::ptr::null_mut();
    let mut size: DWORD;

    if !bv.is_null() {
        size = (core::mem::size_of::<*mut berval>() as DWORD) * (bvarraylen(bv) + 1);
        berval = heap_alloc(size);
        if !berval.is_null() {
            let mut p = bv;
            let mut q = berval;

            while !(*p).is_null() {
                *q = bvdup(*p);
                p = p.offset(1);
                q = q.offset(1);
            }
            *q = core::ptr::null_mut();
        }
    }
    berval
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn bvarrayfree(mut bv: *mut *mut berval) {
    let mut p: *mut *mut berval = bv;
    while !(*p).is_null() {
        let temp = *p;
        p = p.offset(1);
        heap_free(temp);
    }
    heap_free(bv);
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn modAtoW(mod_: *mut LDAPModA) -> *mut LDAPModW {
    let modW: *mut LDAPModW;

    modW = heap_alloc(core::mem::size_of::<LDAPModW>());
    if !modW.is_null() {
        (*modW).mod_op = (*mod_).mod_op;
        (*modW).mod_type = strAtoW((*mod_).mod_type);

        if (*mod_).mod_op & LDAP_MOD_BVALUES != 0 {
            (*modW).mod_vals.modv_bvals = bvarraydup((*mod_).mod_vals.modv_bvals);
        } else {
            (*modW).mod_vals.modv_strvals = strarrayAtoW((*mod_).mod_vals.modv_strvals);
        }
    }
    modW
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn modWtoU(mod_: *mut LDAPModW) -> *mut LDAPMod {
    let modU: *mut LDAPMod;

    modU = heap_alloc(core::mem::size_of::<LDAPMod>());
    if !modU.is_null() {
        (*modU).mod_op = (*mod_).mod_op;
        (*modU).mod_type = strWtoU((*mod_).mod_type);

        if ((*mod_).mod_op & LDAP_MOD_BVALUES) != 0 {
            (*modU).mod_vals.modv_bvals = bvarraydup((*mod_).mod_vals.modv_bvals);
        } else {
            (*modU).mod_vals.modv_strvals = strarrayWtoU((*mod_).mod_vals.modv_strvals);
        }
    }
    modU
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn modfreeW(mod_: *mut LDAPModW) {
    if (*mod_).mod_op & LDAP_MOD_BVALUES != 0 {
        bvarrayfree((*mod_).mod_vals.modv_bvals);
    } else {
        strarrayfreeW((*mod_).mod_vals.modv_strvals);
    }
    heap_free(mod_ as *mut core::ffi::c_void);
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn modfreeU(mut mod_: *mut LDAPMod) {
    if (*mod_).mod_op & LDAP_MOD_BVALUES != 0 {
        bvarrayfree((*mod_).mod_vals.modv_bvals);
    } else {
        strarrayfreeU((*mod_).mod_vals.modv_strvals);
    }
    heap_free(mod_ as *mut core::ffi::c_void);
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn modarraylenA(modarray: *mut *mut LDAPModA) -> DWORD {
    let mut p = modarray;
    while !(*p).is_null() {
        p = p.offset(1);
    }
    p.offset_from(modarray) as DWORD
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn modarraylenW(modarray: *mut *mut LDAPModW) -> DWORD {
    let mut p = modarray;
    while !(*p).is_null() {
        p = p.offset(1);
    }
    p.offset_from(modarray) as DWORD
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn modarrayAtoW(modarray: *mut *mut LDAPModA) -> *mut *mut LDAPModW {
    let mut modarrayW: *mut *mut LDAPModW = core::ptr::null_mut();
    let mut size: DWORD;

    if !modarray.is_null() {
        size = (core::mem::size_of::<*mut LDAPModW>() as DWORD) * (modarraylenA(modarray) + 1);
        modarrayW = heap_alloc(size);
        if !modarrayW.is_null() {
            let mut p: *mut *mut LDAPModA = modarray;
            let mut q: *mut *mut LDAPModW = modarrayW;

            while !(*p).is_null() {
                *q = modAtoW(*p);
                q = q.offset(1);
                p = p.offset(1);
            }
            *q = core::ptr::null_mut();
        }
    }
    modarrayW
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn modarrayWtoU(modarray: *mut *mut LDAPModW) -> *mut *mut LDAPMod {
    let mut modarrayU: *mut *mut LDAPMod = core::ptr::null_mut();
    let mut size: DWORD;

    if !modarray.is_null() {
        size = core::mem::size_of::<*mut LDAPMod>() as DWORD * (modarraylenW(modarray) + 1);
        modarrayU = heap_alloc(size);
        if !modarrayU.is_null() {
            let mut p = modarray;
            let mut q = modarrayU;

            while !(*p).is_null() {
                *q = modWtoU(*p);
                q = q.offset(1);
                p = p.offset(1);
            }
            *q = core::ptr::null_mut();
        }
    }
    modarrayU
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn modarrayfreeW(modarray: *mut *mut LDAPModW) {
    if !modarray.is_null() {
        let mut p = modarray;
        while !(*p).is_null() {
            let temp = *p;
            p = p.add(1);
            modfreeW(temp);
        }
        heap_free(modarray as *mut core::ffi::c_void);
    }
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn modarrayfreeU(mut modarray: *mut *mut LDAPMod) {
    if !modarray.is_null() {
        let mut p: *mut *mut LDAPMod = modarray;
        while !(*p).is_null() {
            let temp = *p;
            p = p.offset(1);
            modfreeU(temp);
        }
        heap_free(modarray);
    }
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlAtoW(control: *mut LDAPControlA) -> *mut LDAPControlW {
    let mut controlW: *mut LDAPControlW;
    let len: DWORD = (*control).ldctl_value.bv_len;
    let mut val: *mut i8 = core::ptr::null_mut();

    if !(*control).ldctl_value.bv_val.is_null() {
        val = heap_alloc(len);
        if val.is_null() {
            return core::ptr::null_mut();
        }
        memcpy(val as *mut core::ffi::c_void, (*control).ldctl_value.bv_val as *const core::ffi::c_void, len as usize);
    }

    controlW = heap_alloc(core::mem::size_of::<LDAPControlW>());
    if controlW.is_null() {
        heap_free(val as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*controlW).ldctl_oid = strAtoW((*control).ldctl_oid);
    (*controlW).ldctl_value.bv_len = len;
    (*controlW).ldctl_value.bv_val = val;
    (*controlW).ldctl_iscritical = (*control).ldctl_iscritical;

    controlW
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlWtoA(control: *mut LDAPControlW) -> *mut LDAPControlA {
    let controlA: *mut LDAPControlA;
    let len: DWORD = (*control).ldctl_value.bv_len;
    let mut val: *mut i8 = core::ptr::null_mut();

    if !(*control).ldctl_value.bv_val.is_null() {
        val = heap_alloc(len);
        if val.is_null() {
            return core::ptr::null_mut();
        }
        memcpy(val as *mut core::ffi::c_void, (*control).ldctl_value.bv_val as *const core::ffi::c_void, len as usize);
    }

    controlA = heap_alloc(core::mem::size_of::<LDAPControlA>());
    if controlA.is_null() {
        heap_free(val as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*controlA).ldctl_oid = strWtoA((*control).ldctl_oid);
    (*controlA).ldctl_value.bv_len = len;
    (*controlA).ldctl_value.bv_val = val;
    (*controlA).ldctl_iscritical = (*control).ldctl_iscritical;

    controlA
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlWtoU(control: *mut LDAPControlW) -> *mut LDAPControl {
    let mut controlU: *mut LDAPControl;
    let len: DWORD = (*control).ldctl_value.bv_len;
    let mut val: *mut i8 = core::ptr::null_mut();

    if !(*control).ldctl_value.bv_val.is_null() {
        val = heap_alloc(len);
        if val.is_null() {
            return core::ptr::null_mut();
        }
        memcpy(val as *mut core::ffi::c_void, (*control).ldctl_value.bv_val as *const core::ffi::c_void, len as usize);
    }

    controlU = heap_alloc(core::mem::size_of::<LDAPControl>());
    if controlU.is_null() {
        heap_free(val as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*controlU).ldctl_oid = strWtoU((*control).ldctl_oid);
    (*controlU).ldctl_value.bv_len = len;
    (*controlU).ldctl_value.bv_val = val;
    (*controlU).ldctl_iscritical = (*control).ldctl_iscritical;

    controlU
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlUtoW(control: *mut LDAPControl) -> *mut LDAPControlW {
    let mut controlW: *mut LDAPControlW;
    let len: DWORD = (*control).ldctl_value.bv_len;
    let mut val: *mut i8 = core::ptr::null_mut();

    if !(*control).ldctl_value.bv_val.is_null() {
        val = heap_alloc(len);
        if val.is_null() {
            return core::ptr::null_mut();
        }
        memcpy(val as *mut core::ffi::c_void, (*control).ldctl_value.bv_val as *const core::ffi::c_void, len as usize);
    }

    controlW = heap_alloc(core::mem::size_of::<LDAPControlW>());
    if controlW.is_null() {
        heap_free(val as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*controlW).ldctl_oid = strUtoW((*control).ldctl_oid);
    (*controlW).ldctl_value.bv_len = len;
    (*controlW).ldctl_value.bv_val = val;
    (*controlW).ldctl_iscritical = (*control).ldctl_iscritical;

    controlW
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlfreeA(control: *mut LDAPControlA) {
    if !control.is_null() {
        strfreeA((*control).ldctl_oid);
        heap_free((*control).ldctl_value.bv_val as *mut core::ffi::c_void);
        heap_free(control as *mut core::ffi::c_void);
    }
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlfreeW(control: *mut LDAPControlW) {
    if !control.is_null() {
        strfreeW((*control).ldctl_oid);
        heap_free((*control).ldctl_value.bv_val as *mut core::ffi::c_void);
        heap_free(control as *mut core::ffi::c_void);
    }
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlfreeU(control: *mut LDAPControl) {
    if !control.is_null() {
        strfreeU((*control).ldctl_oid);
        heap_free((*control).ldctl_value.bv_val);
        heap_free(control as *mut _);
    }
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlarraylenA(controlarray: *mut *mut LDAPControlA) -> DWORD {
    let mut p = controlarray;
    while !(*p).is_null() {
        p = p.offset(1);
    }
    p.offset_from(controlarray) as DWORD
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlarraylenW(controlarray: *mut *mut LDAPControlW) -> DWORD {
    let mut p = controlarray;
    while !(*p).is_null() {
        p = p.offset(1);
    }
    p.offset_from(controlarray) as DWORD
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlarraylenU(controlarray: *mut *mut LDAPControl) -> DWORD {
    let mut p = controlarray;
    while !(*p).is_null() {
        p = p.offset(1);
    }
    p.offset_from(controlarray) as DWORD
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlarrayAtoW(controlarray: *mut *mut LDAPControlA) -> *mut *mut LDAPControlW {
    let mut controlarrayW: *mut *mut LDAPControlW = core::ptr::null_mut();
    let mut size: DWORD;

    if !controlarray.is_null() {
        size = core::mem::size_of::<*mut LDAPControlW>() as DWORD * (controlarraylenA(controlarray) + 1);
        controlarrayW = heap_alloc(size) as *mut *mut LDAPControlW;
        if !controlarrayW.is_null() {
            let mut p = controlarray;
            let mut q = controlarrayW;

            while !(*p).is_null() {
                *q = controlAtoW(*p);
                q = q.offset(1);
                p = p.offset(1);
            }
            *q = core::ptr::null_mut();
        }
    }
    controlarrayW
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlarrayWtoA(controlarray: *mut *mut LDAPControlW) -> *mut *mut LDAPControlA {
    let mut controlarrayA: *mut *mut LDAPControlA = core::ptr::null_mut();
    let mut size: DWORD;

    if !controlarray.is_null() {
        size = core::mem::size_of::<*mut LDAPControl>() as DWORD * (controlarraylenW(controlarray) + 1);
        controlarrayA = heap_alloc(size);
        if !controlarrayA.is_null() {
            let mut p = controlarray;
            let mut q = controlarrayA;

            while !(*p).is_null() {
                *q = controlWtoA(*p);
                p = p.offset(1);
                q = q.offset(1);
            }
            *q = core::ptr::null_mut();
        }
    }
    controlarrayA
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlarrayWtoU(controlarray: *mut *mut LDAPControlW) -> *mut *mut LDAPControl {
    let mut controlarrayU: *mut *mut LDAPControl = core::ptr::null_mut();
    let mut size: DWORD;

    if !controlarray.is_null() {
        size = (core::mem::size_of::<*mut LDAPControl>() as DWORD)
            .wrapping_mul(controlarraylenW(controlarray).wrapping_add(1));
        controlarrayU = heap_alloc(size);
        if !controlarrayU.is_null() {
            let mut p = controlarray;
            let mut q = controlarrayU;

            while !(*p).is_null() {
                *q = controlWtoU(*p);
                p = p.offset(1);
                q = q.offset(1);
            }
            *q = core::ptr::null_mut();
        }
    }
    controlarrayU
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlarrayUtoW(controlarray: *mut *mut LDAPControl) -> *mut *mut LDAPControlW {
    let mut controlarrayW: *mut *mut LDAPControlW = core::ptr::null_mut();
    let mut size: DWORD;

    if !controlarray.is_null() {
        size = core::mem::size_of::<*mut LDAPControlW>() as DWORD * (controlarraylenU(controlarray) + 1);
        controlarrayW = heap_alloc(size) as *mut *mut LDAPControlW;
        if !controlarrayW.is_null() {
            let mut p = controlarray;
            let mut q = controlarrayW;

            while !(*p).is_null() {
                *q = controlUtoW(*p);
                q = q.offset(1);
                p = p.offset(1);
            }
            *q = core::ptr::null_mut();
        }
    }
    controlarrayW
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlarrayfreeA(controlarray: *mut *mut LDAPControlA) {
    if !controlarray.is_null() {
        let mut p = controlarray;
        while !(*p).is_null() {
            let current = *p;
            p = p.add(1);
            controlfreeA(current);
        }
        heap_free(controlarray as *mut core::ffi::c_void);
    }
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlarrayfreeW(controlarray: *mut *mut LDAPControlW) {
    if !controlarray.is_null() {
        let mut p = controlarray;
        while !(*p).is_null() {
            controlfreeW(*p);
            p = p.offset(1);
        }
        heap_free(controlarray);
    }
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn controlarrayfreeU(controlarray: *mut *mut LDAPControl) {
    if !controlarray.is_null() {
        let mut p = controlarray;
        while !(*p).is_null() {
            controlfreeU(*p);
            p = p.offset(1);
        }
        heap_free(controlarray as *mut core::ffi::c_void);
    }
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn sortkeyAtoW(sortkey: *mut LDAPSortKeyA) -> *mut LDAPSortKeyW {
    let sortkeyW: *mut LDAPSortKeyW;

    sortkeyW = heap_alloc(core::mem::size_of::<LDAPSortKeyW>());
    if !sortkeyW.is_null() {
        (*sortkeyW).sk_attrtype = strAtoW((*sortkey).sk_attrtype);
        (*sortkeyW).sk_matchruleoid = strAtoW((*sortkey).sk_matchruleoid);
        (*sortkeyW).sk_reverseorder = (*sortkey).sk_reverseorder;
    }
    sortkeyW
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn sortkeyWtoA(sortkey: *mut LDAPSortKeyW) -> *mut LDAPSortKeyA {
    let sortkeyA: *mut LDAPSortKeyA;

    sortkeyA = heap_alloc(core::mem::size_of::<LDAPSortKeyA>());
    if !sortkeyA.is_null() {
        (*sortkeyA).sk_attrtype = strWtoA((*sortkey).sk_attrtype);
        (*sortkeyA).sk_matchruleoid = strWtoA((*sortkey).sk_matchruleoid);
        (*sortkeyA).sk_reverseorder = (*sortkey).sk_reverseorder;
    }
    sortkeyA
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn sortkeyWtoU(sortkey: *mut LDAPSortKeyW) -> *mut LDAPSortKey {
    let sortkeyU: *mut LDAPSortKey;

    sortkeyU = heap_alloc(core::mem::size_of::<LDAPSortKey>());
    if !sortkeyU.is_null() {
        (*sortkeyU).attributeType = strWtoU((*sortkey).sk_attrtype);
        (*sortkeyU).orderingRule = strWtoU((*sortkey).sk_matchruleoid);
        (*sortkeyU).reverseOrder = (*sortkey).sk_reverseorder;
    }
    sortkeyU
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn sortkeyfreeA(sortkey: *mut LDAPSortKeyA) {
    if !sortkey.is_null() {
        strfreeA((*sortkey).sk_attrtype);
        strfreeA((*sortkey).sk_matchruleoid);
        heap_free(sortkey as *mut core::ffi::c_void);
    }
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn sortkeyfreeW(sortkey: *mut LDAPSortKeyW) {
    if !sortkey.is_null() {
        strfreeW((*sortkey).sk_attrtype);
        strfreeW((*sortkey).sk_matchruleoid);
        heap_free(sortkey as *mut core::ffi::c_void);
    }
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn sortkeyfreeU(sortkey: *mut LDAPSortKey) {
    if !sortkey.is_null() {
        strfreeU((*sortkey).attributeType);
        strfreeU((*sortkey).orderingRule);
        heap_free(sortkey);
    }
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn sortkeyarraylenA(sortkeyarray: *mut *mut LDAPSortKeyA) -> DWORD {
    let mut p = sortkeyarray;
    while !(*p).is_null() {
        p = p.offset(1);
    }
    p.offset_from(sortkeyarray) as DWORD
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn sortkeyarraylenW(sortkeyarray: *mut *mut LDAPSortKeyW) -> DWORD {
    let mut p = sortkeyarray;
    while !(*p).is_null() {
        p = p.offset(1);
    }
    p.offset_from(sortkeyarray) as DWORD
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn sortkeyarrayAtoW(sortkeyarray: *mut *mut LDAPSortKeyA) -> *mut *mut LDAPSortKeyW {
    let mut sortkeyarrayW: *mut *mut LDAPSortKeyW = core::ptr::null_mut();
    let mut size: DWORD;

    if !sortkeyarray.is_null() {
        size = (core::mem::size_of::<*mut LDAPSortKeyW>() as DWORD) * (sortkeyarraylenA(sortkeyarray) + 1);
        sortkeyarrayW = heap_alloc(size);
        if !sortkeyarrayW.is_null() {
            let mut p = sortkeyarray;
            let mut q = sortkeyarrayW;

            while !(*p).is_null() {
                *q = sortkeyAtoW(*p);
                p = p.offset(1);
                q = q.offset(1);
            }
            *q = core::ptr::null_mut();
        }
    }
    sortkeyarrayW
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn sortkeyarrayWtoU(sortkeyarray: *mut *mut LDAPSortKeyW) -> *mut *mut LDAPSortKey {
    let mut sortkeyarrayU: *mut *mut LDAPSortKey = core::ptr::null_mut();
    let mut size: DWORD;

    if !sortkeyarray.is_null() {
        size = (core::mem::size_of::<*mut LDAPSortKey>() as DWORD)
            .wrapping_mul(sortkeyarraylenW(sortkeyarray).wrapping_add(1));
        sortkeyarrayU = heap_alloc(size);
        if !sortkeyarrayU.is_null() {
            let mut p = sortkeyarray;
            let mut q = sortkeyarrayU;

            while !(*p).is_null() {
                *q = sortkeyWtoU(*p);
                p = p.offset(1);
                q = q.offset(1);
            }
            *q = core::ptr::null_mut();
        }
    }
    sortkeyarrayU
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn sortkeyarrayfreeW(sortkeyarray: *mut *mut LDAPSortKeyW) {
    if !sortkeyarray.is_null() {
        let mut p = sortkeyarray;
        while !(*p).is_null() {
            sortkeyfreeW(*p);
            p = p.offset(1);
        }
        heap_free(sortkeyarray);
    }
}
#[cfg(feature = "have_ldap")]
pub unsafe extern "C" fn sortkeyarrayfreeU(sortkeyarray: *mut *mut LDAPSortKey) {
    if !sortkeyarray.is_null() {
        let mut p = sortkeyarray;
        while !(*p).is_null() {
            sortkeyfreeU(*p);
            p = p.offset(1);
        }
        heap_free(sortkeyarray);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * WLDAP32 - LDAP support for Wine
//  *
//  * Copyright 2005 Hans Leidekker
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
// #pragma once
//
// #include "wine/heap.h"
// #include "wine/unicode.h"
//
// extern HINSTANCE hwldap32 DECLSPEC_HIDDEN;
//
// ULONG map_error( int ) DECLSPEC_HIDDEN;
//
// /* A set of helper functions to convert LDAP data structures
//  * to and from ansi (A), wide character (W) and utf8 (U) encodings.
//  */
//
// static inline char *strdupU( const char *src )
// {
//     char *dst;
//     if (!src) return NULL;
//     if ((dst = heap_alloc( (strlen( src ) + 1) * sizeof(char) ))) strcpy( dst, src );
//     return dst;
// }
//
// static inline WCHAR *strdupW( const WCHAR *src )
// {
//     WCHAR *dst;
//     if (!src) return NULL;
//     if ((dst = heap_alloc( (strlenW( src ) + 1) * sizeof(WCHAR) ))) strcpyW( dst, src );
//     return dst;
// }
//
// static inline LPWSTR strAtoW( LPCSTR str )
// {
//     LPWSTR ret = NULL;
//     if (str)
//     {
//         DWORD len = MultiByteToWideChar( CP_ACP, 0, str, -1, NULL, 0 );
//         if ((ret = heap_alloc( len * sizeof(WCHAR) )))
//             MultiByteToWideChar( CP_ACP, 0, str, -1, ret, len );
//     }
//     return ret;
// }
//
// static inline LPSTR strWtoA( LPCWSTR str )
// {
//     LPSTR ret = NULL;
//     if (str)
//     {
//         DWORD len = WideCharToMultiByte( CP_ACP, 0, str, -1, NULL, 0, NULL, NULL );
//         if ((ret = heap_alloc( len )))
//             WideCharToMultiByte( CP_ACP, 0, str, -1, ret, len, NULL, NULL );
//     }
//     return ret;
// }
//
// static inline char *strWtoU( LPCWSTR str )
// {
//     LPSTR ret = NULL;
//     if (str)
//     {
//         DWORD len = WideCharToMultiByte( CP_UTF8, 0, str, -1, NULL, 0, NULL, NULL );
//         if ((ret = heap_alloc( len )))
//             WideCharToMultiByte( CP_UTF8, 0, str, -1, ret, len, NULL, NULL );
//     }
//     return ret;
// }
//
// static inline LPWSTR strUtoW( char *str )
// {
//     LPWSTR ret = NULL;
//     if (str)
//     {
//         DWORD len = MultiByteToWideChar( CP_UTF8, 0, str, -1, NULL, 0 );
//         if ((ret = heap_alloc( len * sizeof(WCHAR) )))
//             MultiByteToWideChar( CP_UTF8, 0, str, -1, ret, len );
//     }
//     return ret;
// }
//
// static inline void strfreeA( LPSTR str )
// {
//     heap_free( str );
// }
//
// static inline void strfreeW( LPWSTR str )
// {
//     heap_free( str );
// }
//
// static inline void strfreeU( char *str )
// {
//     heap_free( str );
// }
//
// static inline DWORD strarraylenA( LPSTR *strarray )
// {
//     LPSTR *p = strarray;
//     while (*p) p++;
//     return p - strarray;
// }
//
// static inline DWORD strarraylenW( LPWSTR *strarray )
// {
//     LPWSTR *p = strarray;
//     while (*p) p++;
//     return p - strarray;
// }
//
// static inline DWORD strarraylenU( char **strarray )
// {
//     char **p = strarray;
//     while (*p) p++;
//     return p - strarray;
// }
//
// static inline LPWSTR *strarrayAtoW( LPSTR *strarray )
// {
//     LPWSTR *strarrayW = NULL;
//     DWORD size;
//
//     if (strarray)
//     {
//         size  = sizeof(WCHAR*) * (strarraylenA( strarray ) + 1);
//         if ((strarrayW = heap_alloc( size )))
//         {
//             LPSTR *p = strarray;
//             LPWSTR *q = strarrayW;
//
//             while (*p) *q++ = strAtoW( *p++ );
//             *q = NULL;
//         }
//     }
//     return strarrayW;
// }
//
// static inline LPSTR *strarrayWtoA( LPWSTR *strarray )
// {
//     LPSTR *strarrayA = NULL;
//     DWORD size;
//
//     if (strarray)
//     {
//         size = sizeof(LPSTR) * (strarraylenW( strarray ) + 1);
//         if ((strarrayA = heap_alloc( size )))
//         {
//             LPWSTR *p = strarray;
//             LPSTR *q = strarrayA;
//
//             while (*p) *q++ = strWtoA( *p++ );
//             *q = NULL;
//         }
//     }
//     return strarrayA;
// }
//
// static inline char **strarrayWtoU( LPWSTR *strarray )
// {
//     char **strarrayU = NULL;
//     DWORD size;
//
//     if (strarray)
//     {
//         size = sizeof(char*) * (strarraylenW( strarray ) + 1);
//         if ((strarrayU = heap_alloc( size )))
//         {
//             LPWSTR *p = strarray;
//             char **q = strarrayU;
//
//             while (*p) *q++ = strWtoU( *p++ );
//             *q = NULL;
//         }
//     }
//     return strarrayU;
// }
//
// static inline LPWSTR *strarrayUtoW( char **strarray )
// {
//     LPWSTR *strarrayW = NULL;
//     DWORD size;
//
//     if (strarray)
//     {
//         size = sizeof(WCHAR*) * (strarraylenU( strarray ) + 1);
//         if ((strarrayW = heap_alloc( size )))
//         {
//             char **p = strarray;
//             LPWSTR *q = strarrayW;
//
//             while (*p) *q++ = strUtoW( *p++ );
//             *q = NULL;
//         }
//     }
//     return strarrayW;
// }
//
// static inline void strarrayfreeA( LPSTR *strarray )
// {
//     if (strarray)
//     {
//         LPSTR *p = strarray;
//         while (*p) strfreeA( *p++ );
//         heap_free( strarray );
//     }
// }
//
// static inline void strarrayfreeW( LPWSTR *strarray )
// {
//     if (strarray)
//     {
//         LPWSTR *p = strarray;
//         while (*p) strfreeW( *p++ );
//         heap_free( strarray );
//     }
// }
//
// static inline void strarrayfreeU( char **strarray )
// {
//     if (strarray)
//     {
//         char **p = strarray;
//         while (*p) strfreeU( *p++ );
//         heap_free( strarray );
//     }
// }
//
// #ifdef HAVE_LDAP
//
// static inline struct berval *bvdup( struct berval *bv )
// {
//     struct berval *berval;
//     DWORD size = sizeof(struct berval) + bv->bv_len;
//
//     if ((berval = heap_alloc( size )))
//     {
//         char *val = (char *)berval + sizeof(struct berval);
//
//         berval->bv_len = bv->bv_len;
//         berval->bv_val = val;
//         memcpy( val, bv->bv_val, bv->bv_len );
//     }
//     return berval;
// }
//
// static inline DWORD bvarraylen( struct berval **bv )
// {
//     struct berval **p = bv;
//     while (*p) p++;
//     return p - bv;
// }
//
// static inline struct berval **bvarraydup( struct berval **bv )
// {
//     struct berval **berval = NULL;
//     DWORD size;
//
//     if (bv)
//     {
//         size = sizeof(struct berval *) * (bvarraylen( bv ) + 1);
//         if ((berval = heap_alloc( size )))
//         {
//             struct berval **p = bv;
//             struct berval **q = berval;
//
//             while (*p) *q++ = bvdup( *p++ );
//             *q = NULL;
//         }
//     }
//     return berval;
// }
//
// static inline void bvarrayfree( struct berval **bv )
// {
//     struct berval **p = bv;
//     while (*p) heap_free( *p++ );
//     heap_free( bv );
// }
//
// static inline LDAPModW *modAtoW( LDAPModA *mod )
// {
//     LDAPModW *modW;
//
//     if ((modW = heap_alloc( sizeof(LDAPModW) )))
//     {
//         modW->mod_op = mod->mod_op;
//         modW->mod_type = strAtoW( mod->mod_type );
//
//         if (mod->mod_op & LDAP_MOD_BVALUES)
//             modW->mod_vals.modv_bvals = bvarraydup( mod->mod_vals.modv_bvals );
//         else
//             modW->mod_vals.modv_strvals = strarrayAtoW( mod->mod_vals.modv_strvals );
//     }
//     return modW;
// }
//
// static inline LDAPMod *modWtoU( LDAPModW *mod )
// {
//     LDAPMod *modU;
//
//     if ((modU = heap_alloc( sizeof(LDAPMod) )))
//     {
//         modU->mod_op = mod->mod_op;
//         modU->mod_type = strWtoU( mod->mod_type );
//
//         if (mod->mod_op & LDAP_MOD_BVALUES)
//             modU->mod_vals.modv_bvals = bvarraydup( mod->mod_vals.modv_bvals );
//         else
//             modU->mod_vals.modv_strvals = strarrayWtoU( mod->mod_vals.modv_strvals );
//     }
//     return modU;
// }
//
// static inline void modfreeW( LDAPModW *mod )
// {
//     if (mod->mod_op & LDAP_MOD_BVALUES)
//         bvarrayfree( mod->mod_vals.modv_bvals );
//     else
//         strarrayfreeW( mod->mod_vals.modv_strvals );
//     heap_free( mod );
// }
//
// static inline void modfreeU( LDAPMod *mod )
// {
//     if (mod->mod_op & LDAP_MOD_BVALUES)
//         bvarrayfree( mod->mod_vals.modv_bvals );
//     else
//         strarrayfreeU( mod->mod_vals.modv_strvals );
//     heap_free( mod );
// }
//
// static inline DWORD modarraylenA( LDAPModA **modarray )
// {
//     LDAPModA **p = modarray;
//     while (*p) p++;
//     return p - modarray;
// }
//
// static inline DWORD modarraylenW( LDAPModW **modarray )
// {
//     LDAPModW **p = modarray;
//     while (*p) p++;
//     return p - modarray;
// }
//
// static inline LDAPModW **modarrayAtoW( LDAPModA **modarray )
// {
//     LDAPModW **modarrayW = NULL;
//     DWORD size;
//
//     if (modarray)
//     {
//         size = sizeof(LDAPModW*) * (modarraylenA( modarray ) + 1);
//         if ((modarrayW = heap_alloc( size )))
//         {
//             LDAPModA **p = modarray;
//             LDAPModW **q = modarrayW;
//
//             while (*p) *q++ = modAtoW( *p++ );
//             *q = NULL;
//         }
//     }
//     return modarrayW;
// }
//
// static inline LDAPMod **modarrayWtoU( LDAPModW **modarray )
// {
//     LDAPMod **modarrayU = NULL;
//     DWORD size;
//
//     if (modarray)
//     {
//         size = sizeof(LDAPMod*) * (modarraylenW( modarray ) + 1);
//         if ((modarrayU = heap_alloc( size )))
//         {
//             LDAPModW **p = modarray;
//             LDAPMod **q = modarrayU;
//
//             while (*p) *q++ = modWtoU( *p++ );
//             *q = NULL;
//         }
//     }
//     return modarrayU;
// }
//
// static inline void modarrayfreeW( LDAPModW **modarray )
// {
//     if (modarray)
//     {
//         LDAPModW **p = modarray;
//         while (*p) modfreeW( *p++ );
//         heap_free( modarray );
//     }
// }
//
// static inline void modarrayfreeU( LDAPMod **modarray )
// {
//     if (modarray)
//     {
//         LDAPMod **p = modarray;
//         while (*p) modfreeU( *p++ );
//         heap_free( modarray );
//     }
// }
//
// static inline LDAPControlW *controlAtoW( LDAPControlA *control )
// {
//     LDAPControlW *controlW;
//     DWORD len = control->ldctl_value.bv_len;
//     char *val = NULL;
//
//     if (control->ldctl_value.bv_val)
//     {
//         if (!(val = heap_alloc( len ))) return NULL;
//         memcpy( val, control->ldctl_value.bv_val, len );
//     }
//
//     if (!(controlW = heap_alloc( sizeof(LDAPControlW) )))
//     {
//         heap_free( val );
//         return NULL;
//     }
//
//     controlW->ldctl_oid = strAtoW( control->ldctl_oid );
//     controlW->ldctl_value.bv_len = len;
//     controlW->ldctl_value.bv_val = val;
//     controlW->ldctl_iscritical = control->ldctl_iscritical;
//
//     return controlW;
// }
//
// static inline LDAPControlA *controlWtoA( LDAPControlW *control )
// {
//     LDAPControlA *controlA;
//     DWORD len = control->ldctl_value.bv_len;
//     char *val = NULL;
//
//     if (control->ldctl_value.bv_val)
//     {
//         if (!(val = heap_alloc( len ))) return NULL;
//         memcpy( val, control->ldctl_value.bv_val, len );
//     }
//
//     if (!(controlA = heap_alloc( sizeof(LDAPControlA) )))
//     {
//         heap_free( val );
//         return NULL;
//     }
//
//     controlA->ldctl_oid = strWtoA( control->ldctl_oid );
//     controlA->ldctl_value.bv_len = len;
//     controlA->ldctl_value.bv_val = val;
//     controlA->ldctl_iscritical = control->ldctl_iscritical;
//
//     return controlA;
// }
//
// static inline LDAPControl *controlWtoU( LDAPControlW *control )
// {
//     LDAPControl *controlU;
//     DWORD len = control->ldctl_value.bv_len;
//     char *val = NULL;
//
//     if (control->ldctl_value.bv_val)
//     {
//         if (!(val = heap_alloc( len ))) return NULL;
//         memcpy( val, control->ldctl_value.bv_val, len );
//     }
//
//     if (!(controlU = heap_alloc( sizeof(LDAPControl) )))
//     {
//         heap_free( val );
//         return NULL;
//     }
//
//     controlU->ldctl_oid = strWtoU( control->ldctl_oid );
//     controlU->ldctl_value.bv_len = len;
//     controlU->ldctl_value.bv_val = val;
//     controlU->ldctl_iscritical = control->ldctl_iscritical;
//
//     return controlU;
// }
//
// static inline LDAPControlW *controlUtoW( LDAPControl *control )
// {
//     LDAPControlW *controlW;
//     DWORD len = control->ldctl_value.bv_len;
//     char *val = NULL;
//
//     if (control->ldctl_value.bv_val)
//     {
//         if (!(val = heap_alloc( len ))) return NULL;
//         memcpy( val, control->ldctl_value.bv_val, len );
//     }
//
//     if (!(controlW = heap_alloc( sizeof(LDAPControlW) )))
//     {
//         heap_free( val );
//         return NULL;
//     }
//
//     controlW->ldctl_oid = strUtoW( control->ldctl_oid );
//     controlW->ldctl_value.bv_len = len;
//     controlW->ldctl_value.bv_val = val;
//     controlW->ldctl_iscritical = control->ldctl_iscritical;
//
//     return controlW;
// }
//
// static inline void controlfreeA( LDAPControlA *control )
// {
//     if (control)
//     {
//         strfreeA( control->ldctl_oid );
//         heap_free( control->ldctl_value.bv_val );
//         heap_free( control );
//     }
// }
//
// static inline void controlfreeW( LDAPControlW *control )
// {
//     if (control)
//     {
//         strfreeW( control->ldctl_oid );
//         heap_free( control->ldctl_value.bv_val );
//         heap_free( control );
//     }
// }
//
// static inline void controlfreeU( LDAPControl *control )
// {
//     if (control)
//     {
//         strfreeU( control->ldctl_oid );
//         heap_free( control->ldctl_value.bv_val );
//         heap_free( control );
//     }
// }
//
// static inline DWORD controlarraylenA( LDAPControlA **controlarray )
// {
//     LDAPControlA **p = controlarray;
//     while (*p) p++;
//     return p - controlarray;
// }
//
// static inline DWORD controlarraylenW( LDAPControlW **controlarray )
// {
//     LDAPControlW **p = controlarray;
//     while (*p) p++;
//     return p - controlarray;
// }
//
// static inline DWORD controlarraylenU( LDAPControl **controlarray )
// {
//     LDAPControl **p = controlarray;
//     while (*p) p++;
//     return p - controlarray;
// }
//
// static inline LDAPControlW **controlarrayAtoW( LDAPControlA **controlarray )
// {
//     LDAPControlW **controlarrayW = NULL;
//     DWORD size;
//
//     if (controlarray)
//     {
//         size = sizeof(LDAPControlW*) * (controlarraylenA( controlarray ) + 1);
//         if ((controlarrayW = heap_alloc( size )))
//         {
//             LDAPControlA **p = controlarray;
//             LDAPControlW **q = controlarrayW;
//
//             while (*p) *q++ = controlAtoW( *p++ );
//             *q = NULL;
//         }
//     }
//     return controlarrayW;
// }
//
// static inline LDAPControlA **controlarrayWtoA( LDAPControlW **controlarray )
// {
//     LDAPControlA **controlarrayA = NULL;
//     DWORD size;
//
//     if (controlarray)
//     {
//         size = sizeof(LDAPControl*) * (controlarraylenW( controlarray ) + 1);
//         if ((controlarrayA = heap_alloc( size )))
//         {
//             LDAPControlW **p = controlarray;
//             LDAPControlA **q = controlarrayA;
//
//             while (*p) *q++ = controlWtoA( *p++ );
//             *q = NULL;
//         }
//     }
//     return controlarrayA;
// }
//
// static inline LDAPControl **controlarrayWtoU( LDAPControlW **controlarray )
// {
//     LDAPControl **controlarrayU = NULL;
//     DWORD size;
//
//     if (controlarray)
//     {
//         size = sizeof(LDAPControl*) * (controlarraylenW( controlarray ) + 1);
//         if ((controlarrayU = heap_alloc( size )))
//         {
//             LDAPControlW **p = controlarray;
//             LDAPControl **q = controlarrayU;
//
//             while (*p) *q++ = controlWtoU( *p++ );
//             *q = NULL;
//         }
//     }
//     return controlarrayU;
// }
//
// static inline LDAPControlW **controlarrayUtoW( LDAPControl **controlarray )
// {
//     LDAPControlW **controlarrayW = NULL;
//     DWORD size;
//
//     if (controlarray)
//     {
//         size = sizeof(LDAPControlW*) * (controlarraylenU( controlarray ) + 1);
//         if ((controlarrayW = heap_alloc( size )))
//         {
//             LDAPControl **p = controlarray;
//             LDAPControlW **q = controlarrayW;
//
//             while (*p) *q++ = controlUtoW( *p++ );
//             *q = NULL;
//         }
//     }
//     return controlarrayW;
// }
//
// static inline void controlarrayfreeA( LDAPControlA **controlarray )
// {
//     if (controlarray)
//     {
//         LDAPControlA **p = controlarray;
//         while (*p) controlfreeA( *p++ );
//         heap_free( controlarray );
//     }
// }
//
// static inline void controlarrayfreeW( LDAPControlW **controlarray )
// {
//     if (controlarray)
//     {
//         LDAPControlW **p = controlarray;
//         while (*p) controlfreeW( *p++ );
//         heap_free( controlarray );
//     }
// }
//
// static inline void controlarrayfreeU( LDAPControl **controlarray )
// {
//     if (controlarray)
//     {
//         LDAPControl **p = controlarray;
//         while (*p) controlfreeU( *p++ );
//         heap_free( controlarray );
//     }
// }
//
// static inline LDAPSortKeyW *sortkeyAtoW( LDAPSortKeyA *sortkey )
// {
//     LDAPSortKeyW *sortkeyW;
//
//     if ((sortkeyW = heap_alloc( sizeof(LDAPSortKeyW) )))
//     {
//         sortkeyW->sk_attrtype = strAtoW( sortkey->sk_attrtype );
//         sortkeyW->sk_matchruleoid = strAtoW( sortkey->sk_matchruleoid );
//         sortkeyW->sk_reverseorder = sortkey->sk_reverseorder;
//     }
//     return sortkeyW;
// }
//
// static inline LDAPSortKeyA *sortkeyWtoA( LDAPSortKeyW *sortkey )
// {
//     LDAPSortKeyA *sortkeyA;
//
//     if ((sortkeyA = heap_alloc( sizeof(LDAPSortKeyA) )))
//     {
//         sortkeyA->sk_attrtype = strWtoA( sortkey->sk_attrtype );
//         sortkeyA->sk_matchruleoid = strWtoA( sortkey->sk_matchruleoid );
//         sortkeyA->sk_reverseorder = sortkey->sk_reverseorder;
//     }
//     return sortkeyA;
// }
//
// static inline LDAPSortKey *sortkeyWtoU( LDAPSortKeyW *sortkey )
// {
//     LDAPSortKey *sortkeyU;
//
//     if ((sortkeyU = heap_alloc( sizeof(LDAPSortKey) )))
//     {
//         sortkeyU->attributeType = strWtoU( sortkey->sk_attrtype );
//         sortkeyU->orderingRule = strWtoU( sortkey->sk_matchruleoid );
//         sortkeyU->reverseOrder = sortkey->sk_reverseorder;
//     }
//     return sortkeyU;
// }
//
// static inline void sortkeyfreeA( LDAPSortKeyA *sortkey )
// {
//     if (sortkey)
//     {
//         strfreeA( sortkey->sk_attrtype );
//         strfreeA( sortkey->sk_matchruleoid );
//         heap_free( sortkey );
//     }
// }
//
// static inline void sortkeyfreeW( LDAPSortKeyW *sortkey )
// {
//     if (sortkey)
//     {
//         strfreeW( sortkey->sk_attrtype );
//         strfreeW( sortkey->sk_matchruleoid );
//         heap_free( sortkey );
//     }
// }
//
// static inline void sortkeyfreeU( LDAPSortKey *sortkey )
// {
//     if (sortkey)
//     {
//         strfreeU( sortkey->attributeType );
//         strfreeU( sortkey->orderingRule );
//         heap_free( sortkey );
//     }
// }
//
// static inline DWORD sortkeyarraylenA( LDAPSortKeyA **sortkeyarray )
// {
//     LDAPSortKeyA **p = sortkeyarray;
//     while (*p) p++;
//     return p - sortkeyarray;
// }
//
// static inline DWORD sortkeyarraylenW( LDAPSortKeyW **sortkeyarray )
// {
//     LDAPSortKeyW **p = sortkeyarray;
//     while (*p) p++;
//     return p - sortkeyarray;
// }
//
// static inline LDAPSortKeyW **sortkeyarrayAtoW( LDAPSortKeyA **sortkeyarray )
// {
//     LDAPSortKeyW **sortkeyarrayW = NULL;
//     DWORD size;
//
//     if (sortkeyarray)
//     {
//         size = sizeof(LDAPSortKeyW*) * (sortkeyarraylenA( sortkeyarray ) + 1);
//         if ((sortkeyarrayW = heap_alloc( size )))
//         {
//             LDAPSortKeyA **p = sortkeyarray;
//             LDAPSortKeyW **q = sortkeyarrayW;
//
//             while (*p) *q++ = sortkeyAtoW( *p++ );
//             *q = NULL;
//         }
//     }
//     return sortkeyarrayW;
// }
//
// static inline LDAPSortKey **sortkeyarrayWtoU( LDAPSortKeyW **sortkeyarray )
// {
//     LDAPSortKey **sortkeyarrayU = NULL;
//     DWORD size;
//
//     if (sortkeyarray)
//     {
//         size = sizeof(LDAPSortKey*) * (sortkeyarraylenW( sortkeyarray ) + 1);
//         if ((sortkeyarrayU = heap_alloc( size )))
//         {
//             LDAPSortKeyW **p = sortkeyarray;
//             LDAPSortKey **q = sortkeyarrayU;
//
//             while (*p) *q++ = sortkeyWtoU( *p++ );
//             *q = NULL;
//         }
//     }
//     return sortkeyarrayU;
// }
//
// static inline void sortkeyarrayfreeW( LDAPSortKeyW **sortkeyarray )
// {
//     if (sortkeyarray)
//     {
//         LDAPSortKeyW **p = sortkeyarray;
//         while (*p) sortkeyfreeW( *p++ );
//         heap_free( sortkeyarray );
//     }
// }
//
// static inline void sortkeyarrayfreeU( LDAPSortKey **sortkeyarray )
// {
//     if (sortkeyarray)
//     {
//         LDAPSortKey **p = sortkeyarray;
//         while (*p) sortkeyfreeU( *p++ );
//         heap_free( sortkeyarray );
//     }
// }
// #endif /* HAVE_LDAP */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
