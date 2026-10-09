use crate::prelude::*;



// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate.
// The C preamble contains only a debug channel macro and a function signature fragment.

#[no_mangle]
pub unsafe extern "system" fn OleCreateLinkToFile(
    lpszFileName: LPCOLESTR,
    riid: REFIID,
    renderopt: DWORD,
    lpFormatEtc: LPFORMATETC,
    pClientSite: LPOLECLIENTSITE,
    pStg: LPSTORAGE,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    FIXME!(
        b"(%p,%p,%i,%p,%p,%p,%p), stub!\n\0".as_ptr() as *const i8,
        lpszFileName,
        riid,
        renderopt,
        lpFormatEtc,
        pClientSite,
        pStg,
        ppvObj,
    );
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn OleCreateLink(
    _pmkLinkSrc: LPMONIKER,
    _riid: REFIID,
    _renderopt: DWORD,
    _lpFormatEtc: LPFORMATETC,
    _pClientSite: LPOLECLIENTSITE,
    _pStg: LPSTORAGE,
    _ppvObj: *mut LPVOID,
) -> HRESULT {
    FIXME!(b"(not shown), stub!\n\0".as_ptr() as *const i8);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn OleGetIconOfClass(
    rclsid: *const CLSID,
    lpszLabel: LPOLESTR,
    fUseTypeAsLabel: BOOL,
) -> HGLOBAL {
    FIXME!(
        b"(%p,%p,%x), stub!\n\0".as_ptr(),
        rclsid,
        lpszLabel,
        fUseTypeAsLabel,
    );
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn OleGetIconOfFile(
    path: LPOLESTR,
    use_file_as_label: BOOL,
) -> HGLOBAL {
    FIXME!(
        b"(%s, %d), stub!\n\0".as_ptr() as *const i8,
        debugstr_w(path.cast()),
        use_file_as_label,
    );
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn OleRegEnumFormatEtc(
    clsid: *const GUID,
    dwDirection: DWORD,
    ppenumFormatetc: *mut LPENUMFORMATETC,
) -> HRESULT {
    FIXME!(
        b"(%p, %d, %p), stub!\n\0".as_ptr() as *const i8,
        clsid,
        dwDirection,
        ppenumFormatetc,
    );

    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn CoGetCallerTID(_lpdwTID: LPDWORD) -> HRESULT {
    FIXME!(b"stub!\n\0".as_ptr());
    E_NOTIMPL
}
