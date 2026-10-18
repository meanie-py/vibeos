use crate::prelude::*;
use core::ffi::c_void;
pub unsafe extern "C" fn IsPathSep__T0(c: wchar_t) -> bool {
    return c == '\\' as wchar_t || c == '/' as wchar_t;
}

pub unsafe extern "C" fn IsPathSep__T1(c: i8) -> bool {
    c == b'\\' as i8 || c == b'/' as i8
}
#[no_mangle]
pub unsafe extern "C" fn IsEqual(scid: *const SHCOLUMNID, guid: REFGUID, pid: UINT) -> bool {
    return (*scid).pid == pid && IsEqualGUID!(core::ptr::addr_of!((*scid).fmtid), guid) != false;
}
#[no_mangle]
pub unsafe extern "C" fn InitVariantFromBuffer(buffer: *const core::ffi::c_void, cb: UINT, pv: *mut VARIANT) -> HRESULT {
    let pa: *mut SAFEARRAY = SafeArrayCreateVector(VT_UI1, 0, cb);
    if !pa.is_null() {
        CopyMemory!((*pa).pvData, buffer, cb as usize);
        V_VT!(pv) = (VT_UI1 | VT_ARRAY) as _;
        V_UNION!(pv, parray) = pa;
        return S_OK;
    }
    V_VT!(pv) = VT_EMPTY as _;
    return E_OUTOFMEMORY;
}
#[no_mangle]
pub unsafe extern "C" fn FreeStrRet(str: *mut STRRET) {
    if (*str).uType == STRRET_WSTR {
        SHFree(((*str).__bindgen_anon_1.pOleStr).cast());
        (*str).uType = STRRET_CSTR;
    }
}
#[no_mangle]
pub unsafe extern "C" fn StrTo__0(str: LPCWSTR, mut len: UINT, sr: *mut STRRET) -> HRESULT {
    len = len.wrapping_add(1);
    let data: LPWSTR = SHAlloc((len.wrapping_mul(core::mem::size_of::<u16>() as UINT)) as SIZE_T) as LPWSTR;
    if data.is_null() {
        return E_OUTOFMEMORY;
    }
    lstrcpynW(data, str, len as i32);
    (*sr).uType = STRRET_WSTR;
    (*sr).__bindgen_anon_1.pOleStr = data;
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn StrTo__1(str: LPCWSTR, sr: *mut STRRET) -> HRESULT {
    return StrTo__0(str, lstrlenW(str) as UINT, sr);
}
#[no_mangle]
pub unsafe extern "C" fn StrTo__2(str: LPCWSTR, len: UINT, v: *mut VARIANT) -> HRESULT {
    let data = SysAllocStringLen(str, len);
    if data.is_null() {
        return E_OUTOFMEMORY;
    }
    V_VT!(v) = (VT_BSTR as u16);
    V_BSTR!(v) = data;
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn StrTo__3(str: LPCWSTR, v: *mut VARIANT) -> HRESULT {
    return StrTo__2(str, lstrlenW(str) as UINT, v);
}
#[no_mangle]
pub unsafe extern "C" fn StrRetToVariantBSTR(psr: *mut STRRET, v: *mut VARIANT) -> HRESULT {
    let mut hr: HRESULT = StrRetToBSTR(psr, core::ptr::null_mut(), core::ptr::addr_of_mut!(V_BSTR!(v)));
    if hr >= 0 {
        V_VT!(v) = (VT_BSTR as u16);
    }
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn GetDetailsOf(Folder: *mut IShellFolder2, pidl: PCUITEMID_CHILD, Column: UINT, String: *mut PWSTR) -> HRESULT {
    let mut details: SHELLDETAILS = core::mem::zeroed();
    let mut hr: HRESULT = (*Folder).GetDetailsOf(pidl, Column, core::ptr::addr_of_mut!(details));
    if hr >= 0 {
        hr = StrRetToStrW(core::ptr::addr_of_mut!(details.str_), pidl, String);
    }
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn InsertMenuItem(qcmi: &mut QCMINFO, Pos_: &mut UINT, TrackId: &mut UINT, mut Id: UINT, ResId: UINT, State: i32) -> HRESULT {
    let mut flags: UINT = 0;
    let mut string: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    string[0 as usize] = UNICODE_NULL as WCHAR;
    Id += qcmi.idCmdFirst;
    if Id > qcmi.idCmdLast {
        return E_FAIL;
    } else if ResId == (1 as UINT).wrapping_neg() {
        flags |= MF_SEPARATOR;
    } else if LoadStringW(_AtlBaseModule.GetResourceInstance(), ResId, string.as_mut_ptr(), (_countof!(string)) as i32) == 0 {
        return E_FAIL;
    }

    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    mii.cbSize = FIELD_OFFSET!(MENUITEMINFOW, hbmpItem) as UINT; // USER32 version agnostic
    mii.fMask = MIIM_FTYPE | MIIM_ID | MIIM_STRING | MIIM_STATE;
    mii.fType = flags;
    mii.wID = Id;
    mii.dwTypeData = string.as_mut_ptr();
    mii.cch = 0;
    mii.fState = State as UINT;
    if InsertMenuItemW(qcmi.hmenu, *Pos_, TRUE as BOOL, core::ptr::addr_of_mut!(mii)) == 0 {
        return E_FAIL;
    }
    *Pos_ += 1; // `Pos` is a constant in the island: as a parameter it was a pattern
    *TrackId = max!(*TrackId, Id);
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn MapFSToSFAttributes(att: UINT) -> SFGAOF {
    return ((if (att & FILE_ATTRIBUTE_READONLY) != 0 { SFGAO_READONLY } else { 0 }) as SFGAOF) |
           ((if (att & FILE_ATTRIBUTE_HIDDEN) != 0 { SFGAO_HIDDEN } else { 0 }) as SFGAOF) |
           ((if (att & FILE_ATTRIBUTE_SYSTEM) != 0 { SFGAO_SYSTEM } else { 0 }) as SFGAOF);
}
#[no_mangle]
pub unsafe extern "C" fn IncludeInEnumIDList(contf: u32, att: u32) -> bool {
    let both: u32 = SHCONTF_FOLDERS | SHCONTF_NONFOLDERS;
    let superbits: u32 = SFGAO_HIDDEN | SFGAO_READONLY | SFGAO_SYSTEM;
    let isfile: bool = (att & (SFGAO_STREAM | SFGAO_FOLDER)) != SFGAO_FOLDER;
    if (contf & both) != both && !((contf & SHCONTF_STORAGE) != 0) {
        if isfile && (contf & SHCONTF_FOLDERS) != 0 {
            return false;
        }
        if (att & SFGAO_FOLDER) != 0 && (contf & SHCONTF_NONFOLDERS) != 0 {
            return false;
        }
    }
    if (att & SFGAO_HIDDEN) != 0 && !((contf & (SHCONTF_INCLUDEHIDDEN | SHCONTF_STORAGE)) != 0) {
        return false;
    }
    if (att & superbits) > SFGAO_HIDDEN && !((contf & (SHCONTF_INCLUDESUPERHIDDEN | SHCONTF_STORAGE)) != 0) {
        return false;
    }
    true
}
#[no_mangle]
pub unsafe extern "C" fn MapPIDLToSystemImageListIndex(pSF: *mut IShellFolder, pidl: PCUITEMID_CHILD, GilFlags: UINT) -> i32 {
    let mut normal: i32;
    let mut open: i32 = 0;
    let qopen: BOOL = (GilFlags & GIL_OPENICON) as BOOL;
    normal = SHMapPIDLToSystemImageListIndex(pSF, pidl, if qopen != 0 { core::ptr::addr_of_mut!(open) } else { core::ptr::null_mut() });
    return if qopen != 0 && open != -1 { open } else { normal };
}

pub unsafe extern "C" fn ErrorBox__T0(hWnd: *mut HWND__, Error: i32) -> i32 {
    SHELL_ErrorBox(hWnd, (Error) as u32);
    return Error;
}
