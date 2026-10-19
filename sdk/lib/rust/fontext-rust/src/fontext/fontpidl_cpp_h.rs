use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn _ILCreate(lpName: LPCWSTR, lpFileName: LPCWSTR) -> PITEMID_CHILD {
    ATLASSERT!(!lpName.is_null());
    ATLASSERT!(!lpFileName.is_null());

    if *lpName.add(0 as usize) == 0 || *lpFileName.add(0 as usize) == 0
    {
        return core::ptr::null_mut();
    }

    let mut hr: HRESULT;
    let mut cbName: usize;
    let mut cbFileName: usize;
    cbName = 0;
    cbFileName = 0;
    hr = StringCbLengthW(lpName, min!((MAXWORD as usize) - (1 as usize), STRSAFE_MAX_CCH as usize) * core::mem::size_of::<WCHAR>(), &mut cbName as *mut usize);
    if FAILED_UNEXPECTEDLY!(hr) {
        return core::ptr::null_mut();
    }
    hr = StringCbLengthW(lpFileName, min!((MAXWORD as usize) - (1 as usize), STRSAFE_MAX_CCH as usize) * core::mem::size_of::<WCHAR>(), &mut cbFileName as *mut usize);
    if FAILED_UNEXPECTEDLY!(hr) {
        return core::ptr::null_mut();
    }
    cbName += core::mem::size_of_val(&UNICODE_NULL);
    cbFileName += core::mem::size_of_val(&UNICODE_NULL);

    let ibName: usize = ALIGN_DWORD!(core::mem::size_of::<FontPidlEntry>());
    let ibFileName: usize = ALIGN_DWORD!(ibName + cbName);
    let cbData: usize = ibFileName + cbFileName;
    if cbData > (MAXWORD as usize) - core::mem::size_of::<WORD>() {
        ATLASSERT!(FALSE);
        return core::ptr::null_mut();
    }

    let pidl: *mut FontPidlEntry = CoTaskMemAlloc((cbData + core::mem::size_of::<WORD>()) as SIZE_T) as *mut FontPidlEntry;
    if pidl.is_null() {
        return core::ptr::null_mut();
    }

    ZeroMemory!(pidl as *mut core::ffi::c_void, cbData + core::mem::size_of::<WORD>());
    (*pidl).cb = cbData as WORD;
    (*pidl).Magic = FONTPIDL_MAGIC as WORD;
    (*pidl).ibName = ibName as WORD;
    (*pidl).ibFileName = ibFileName as WORD;

    hr = StringCbCopyW((*pidl).Name__0(), cbName, lpName);
    if FAILED_UNEXPECTEDLY!(hr) {
        CoTaskMemFree(pidl as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }
    hr = StringCbCopyW((*pidl).FileName__0(), cbFileName, lpFileName);
    if FAILED_UNEXPECTEDLY!(hr) {
        CoTaskMemFree(pidl as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    *((pidl as PBYTE).add(cbData) as PWORD) = UNICODE_NULL as WORD;

    ATLASSERT!(_FontFromIL(pidl as PITEMID_CHILD));

    return pidl as PITEMID_CHILD;
}
#[no_mangle]
pub unsafe extern "C" fn _FontFromIL(pidl: PCITEMID_CHILD) -> *const FontPidlEntry {
    if pidl.is_null() || ((*pidl).mkid.cb as usize) < core::mem::size_of::<FontPidlEntry>() {
        return core::ptr::null();
    }

    let fontEntry: *const FontPidlEntry = pidl as *const FontPidlEntry;
    if (*fontEntry).Magic != (FONTPIDL_MAGIC as u16) {
        return core::ptr::null();
    }

    // The function gets an arbitrary PIDL here. Security is important.
    if ((*fontEntry).ibName as usize) < core::mem::size_of::<FontPidlEntry>()
        || ((*fontEntry).ibFileName as usize) < core::mem::size_of::<FontPidlEntry>()
        || (*fontEntry).ibName >= (*fontEntry).cb
        || (*fontEntry).ibFileName >= (*fontEntry).cb
        || (*fontEntry).ibName as usize % core::mem::size_of::<WCHAR>() != 0
        || (*fontEntry).ibFileName as usize % core::mem::size_of::<WCHAR>() != 0
    {
        return core::ptr::null();
    }

    let mut cbName: usize = 0;
    let cbNameMax: usize = ((*fontEntry).cb as usize) - ((*fontEntry).ibName as usize);
    if FAILED_UNEXPECTEDLY!(StringCbLengthW(
        (*fontEntry.cast_mut()).Name__1(),
        cbNameMax,
        core::ptr::addr_of_mut!(cbName),
    )) != false
    {
        return core::ptr::null();
    }
    let mut cbFileName: usize = 0;
    let cbFileNameMax: usize = ((*fontEntry).cb as usize) - ((*fontEntry).ibFileName as usize);
    if FAILED_UNEXPECTEDLY!(StringCbLengthW(
        (*fontEntry.cast_mut()).FileName__1(),
        cbFileNameMax,
        core::ptr::addr_of_mut!(cbFileName),
    )) != false
    {
        return core::ptr::null();
    }

    fontEntry
}
