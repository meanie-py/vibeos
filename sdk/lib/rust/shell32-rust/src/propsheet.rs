// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_511C62595BCD: [WCHAR; 30] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0065, 0x0078, 0x005C, 0x0050, 0x0072, 0x006F, 0x0070, 0x0065, 0x0072, 0x0074, 0x0079, 0x0053, 0x0068, 0x0065, 0x0065, 0x0074, 0x0048, 0x0061, 0x006E, 0x0064, 0x006C, 0x0065, 0x0072, 0x0073, 0x0000];

use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;
use crate::win32::*;

// ---- file-scope preamble (translated) ----
pub static __wine_dbch_shell: &'static str = "shell";
#[no_mangle]
pub unsafe extern "C" fn SHELL_GetShellExtensionRegCLSID(hKey: HKEY, KeyName: PCWSTR, pClsId: *mut CLSID) -> HRESULT {
    if SHCLSIDFromStringW(KeyName, pClsId.cast()) >= 0 {
        return S_OK;
    }
    let mut buf: [u16; 42 as usize] = [0u16; 42 as usize];
    let mut cb: u32 = core::mem::size_of_val(&buf) as u32;
    let err: LONG = RegGetValueW(
        hKey,
        KeyName,
        core::ptr::null_mut(),
        RRF_RT_REG_SZ,
        core::ptr::null_mut(),
        buf.as_mut_ptr().cast(),
        core::ptr::addr_of_mut!(cb),
    );
    if err == 0 {
        SHCLSIDFromStringW(buf.as_ptr(), pClsId.cast()) as HRESULT
    } else {
        HRESULT_FROM_WIN32(err)
    }
}
unsafe extern "C" fn SHELL_InitializeExtension(
    clsid: REFCLSID,
    pidlFolder: PCIDLIST_ABSOLUTE,
    pDO: *mut IDataObject,
    hkeyProgID: HKEY,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    *ppv = core::ptr::null_mut();
    let mut pUnk: *mut IUnknown = core::ptr::null_mut();
    // SHExtCoCreateInstance is in wine/shellole.rs, which declares its own GUID
    // record — byte-identical to shellcom's, distinct to rustc.
    let mut hr = SHExtCoCreateInstance(
        core::ptr::null_mut(),
        clsid.cast(),
        core::ptr::null_mut(),
        riid.cast(),
        (&mut pUnk as *mut *mut IUnknown).cast(),
    );
    if hr >= 0 {
        let mut init: *mut IShellExtInit = core::ptr::null_mut();

        hr = (*pUnk).QueryInterface(core::ptr::addr_of!(IID_IShellExtInit), core::ptr::addr_of_mut!(init).cast());
        if hr >= 0 {
            hr = (*(init)).Initialize(pidlFolder, pDO, hkeyProgID);
        }
        if hr >= 0 {
            *ppv = pUnk as *mut core::ffi::c_void;
        } else {
            (*pUnk).Release();
        }
        if !init.is_null() {
            (*(init as *mut IUnknown)).Release();
        }
    }
    hr
}
unsafe extern "C" fn AddPropSheetHandlerPages(
    clsid: REFCLSID,
    pDO: *mut IDataObject,
    hkeyProgID: HKEY,
    psh: *mut PROPSHEETHEADERW,
) -> HRESULT {
    let mut sheet_ext: *mut IShellPropSheetExt = core::ptr::null_mut();
    let hr = SHELL_InitializeExtension(
        clsid,
        core::ptr::null_mut(),
        pDO,
        hkeyProgID,
        core::ptr::addr_of!(IID_IShellPropSheetExt),
        core::ptr::addr_of_mut!(sheet_ext).cast(),
    );
    if hr >= 0 {
        let old_count = (*psh).nPages;
        let add_hr = (*sheet_ext).AddPages(AddPropSheetPageCallback as *mut c_void, psh as LPARAM);
        if add_hr > 0 {
            let adjusted_hr = add_hr + old_count as HRESULT;
            (*psh).set_nStartPage((adjusted_hr - 1) as UINT);
        }
    }
    if !sheet_ext.is_null() {
        (*(sheet_ext as *mut IUnknown)).Release();
    }
    hr
}
// C: `SHELL_CreatePropSheetStubWindow(CStubWindow32 &stub, ...)`. CStubWindow32 is an
// ATL CWindowImpl<> still owned by utils.cpp, so the object is created and destroyed
// across `CStubWindow32_*_RustBridge`; everything the C++ then does with the stub
// (SendMessage, hwndParent, DestroyWindow) is plain Win32 on its HWND.
unsafe extern "C" fn SHELL_CreatePropSheetStubWindow(
    phWnd: *mut HWND,
    ppStub: *mut *mut c_void,
    pidl: PCIDLIST_ABSOLUTE,
    pPt: *const POINT,
) -> HRESULT {
    let mut path: PWSTR = core::ptr::null_mut();
    if pidl.is_null() || SHGetNameFromIDList(pidl, SIGDN_DESKTOPABSOLUTEPARSING, &raw mut path) < 0 {
        path = core::ptr::null_mut();
    }

    let hr = CStubWindow32_CreateStub_RustBridge(
        CSTUBWINDOW32_TYPE_PROPERTYSHEET,
        path,
        pPt,
        // crate::win32's HWND is a bare HANDLE; the island's is `*mut HWND__`. Same pointer.
        phWnd as _,
        ppStub,
    );

    SHFree(path.cast());

    let flags: u32 = SHGFI_ICON | SHGFI_ADDOVERLAYS;
    let mut sfi = core::mem::MaybeUninit::<SHFILEINFO>::uninit();
    if hr >= 0 && SHGetFileInfoW(
        pidl as LPWSTR,
        0,
        sfi.as_mut_ptr().cast(),
        core::mem::size_of::<SHFILEINFO>() as u32,
        SHGFI_PIDL | flags
    ) > 0 {
        let sfi = sfi.assume_init();
        SendMessageW(*phWnd, WM_SETICON, ICON_BIG as WPARAM, sfi.hIcon as LPARAM);
    }
    hr
}
unsafe extern "C" fn SHELL32_PropertySheet(
    ppsh: *mut PROPSHEETHEADERW,
    pDO: *mut IDataObject,
) -> isize {
    let mut stub_hwnd: HWND = core::ptr::null_mut();
    let mut stub_obj: *mut c_void = core::ptr::null_mut();
    let mut pt: POINT = core::mem::zeroed();
    let mut ppt: *const POINT = core::ptr::null();
    if !pDO.is_null()
        && DataObject_GetOffset(pDO, core::ptr::addr_of_mut!(pt)) >= 0
    {
        ppt = core::ptr::addr_of!(pt);
    }
    let pidl = SHELL_DataObject_ILCloneFullItem(pDO, 0);
    let hr = SHELL_CreatePropSheetStubWindow(
        core::ptr::addr_of_mut!(stub_hwnd),
        core::ptr::addr_of_mut!(stub_obj),
        pidl,
        ppt,
    );
    ILFree(pidl);
    if hr < 0 {
        return if hr == HRESULT_FROM_WIN32(ERROR_ALREADY_EXISTS as LONG) {
            0
        } else {
            -1
        };
    }
    let mut Result = -1;
    (*ppsh).hwndParent = stub_hwnd;
    if (*ppsh).nPages != 0 {
        Result = PropertySheetW(ppsh);
    } else {
        let mut szFormat: [u16; 128] = [0u16; 128];
        let mut szMessage: [u16; 128 + 42] =
            [0u16; 128 + 42];
        LoadStringW(
            shell32_hInstance,
            IDS_CANTSHOWPROPERTIES,
            szFormat.as_mut_ptr(),
            128,
        );
        wsprintfW(
            szMessage.as_mut_ptr(),
            szFormat.as_ptr(),
            ERROR_CAN_NOT_COMPLETE,
        );
        MessageBoxW(
            core::ptr::null_mut(),
            szMessage.as_ptr(),
            core::ptr::null_mut(),
            MB_ICONERROR,
        );
    }
    CStubWindow32_DestroyStub_RustBridge(stub_obj);
    Result
}
unsafe extern "C" fn SHELL32_OpenPropSheet(
    pszCaption: PCWSTR,
    ahKeys: *mut HKEY,
    cKeys: UINT,
    pclsidDefault: *const CLSID,
    pDO: *mut IDataObject,
    pStartPage: PCWSTR,
) -> i32 {
    let hKeyProgID = if cKeys > 0 {
        *ahKeys.add((cKeys - 1) as usize)
    } else {
        core::ptr::null_mut()
    };

    let mut hppages: [HPROPSHEETPAGE; MAX_PROPERTY_SHEET_PAGE as usize] =
        [core::mem::zeroed(); MAX_PROPERTY_SHEET_PAGE as usize];

    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as u32;
    psh.dwFlags = PSH_PROPTITLE as u32;
    psh.phpage = hppages.as_mut_ptr() as *mut HPROPSHEETPAGE;
    psh.hInstance = shell32_hInstance;
    psh.pszCaption = pszCaption;
    psh.pStartPage = pStartPage;

    if !pclsidDefault.is_null() {
        AddPropSheetHandlerPages(pclsidDefault, pDO, hKeyProgID, &mut psh);
    }

    let mut i = 0;
    while i < cKeys {
        let mut hKey: HKEY = core::ptr::null_mut();
        if RegOpenKeyExW(
            *ahKeys.add(i as usize),
            CPP_COM_WSTR_511C62595BCD.as_ptr(),
            0,
            KEY_ENUMERATE_SUB_KEYS,
            &mut hKey,
        ) != 0
        {
            i += 1;
            continue;
        }

        let mut index: UINT = 0;
        loop {
            let mut KeyName: [u16; MAX_PATH as usize] =
                [0u16; MAX_PATH as usize];
            let err = RegEnumKeyW(
                hKey,
                index,
                KeyName.as_mut_ptr(),
                _countof!(KeyName) as DWORD,
            );
            KeyName[(_countof!(KeyName) - 1) as usize] =
                UNICODE_NULL;

            if err as DWORD == ERROR_MORE_DATA {
                index += 1;
                continue;
            }
            if err != 0 {
                break;
            }

            let mut clsid: CLSID = core::mem::zeroed();
            if SHELL_GetShellExtensionRegCLSID(hKey, KeyName.as_mut_ptr(), &mut clsid)
                >= 0
            {
                AddPropSheetHandlerPages(core::ptr::addr_of!(clsid), pDO, hKeyProgID, &mut psh);
            }

            index += 1;
        }

        RegCloseKey(hKey);
        i += 1;
    }

    if pStartPage == psh.pStartPage && !pStartPage.is_null() {
        psh.dwFlags |= PSH_USEPSTARTPAGE as u32;
    }

    let result = SHELL32_PropertySheet(&mut psh, pDO);
    if result != -(1 as isize) { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn SHOpenPropSheetA(
    pszCaption: *const i8,
    ahKeys: *mut HKEY,
    cKeys: u32,
    pclsidDefault: *const CLSID,
    pDataObject: *mut IDataObject,
    pShellBrowser: *mut IShellBrowser,
    pszStartPage: *const i8,
) -> i32 {
    let mut strStartPageW: CStringW;
    // C: `CStringW strStartPageW, strCaptionW;` then two A->W assignments.
    //
    // UPSTREAM BUG, preserved deliberately: propsheet.cpp:182-186 converts pszCaption
    // into strStartPageW (not strCaptionW) and then reads strCaptionW, which is still
    // empty — so the caption is silently dropped and immediately overwritten by the
    // start-page branch below. Translating it faithfully keeps Rust and C++ behaviour
    // identical; fixing it here would be an undeclared behaviour change.
    //
    // CStringW is dynamic; these are fixed MAX_PATH buffers, so an over-long caption or
    // start page truncates where the C++ would grow.
    let mut caption_w: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut start_page_w: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let mut pszCaptionW: *const u16 = core::ptr::null();
    let mut pszStartPageW: *const u16 = core::ptr::null();

    if !pszCaption.is_null() {
        MultiByteToWideChar(CP_ACP, 0, pszCaption, -1, start_page_w.as_mut_ptr(), MAX_PATH as INT);
        // Empty, exactly as the C++ leaves it.
        pszCaptionW = caption_w.as_ptr();
    }

    if !pszStartPage.is_null() {
        MultiByteToWideChar(CP_ACP, 0, pszStartPage, -1, start_page_w.as_mut_ptr(), MAX_PATH as INT);
        pszStartPageW = start_page_w.as_ptr();
    }

    SHOpenPropSheetW(
        pszCaptionW,
        ahKeys,
        cKeys,
        pclsidDefault,
        pDataObject,
        pShellBrowser,
        pszStartPageW,
    )
}

#[no_mangle]
pub unsafe extern "system" fn SHOpenPropSheetW(
    pszCaption: *const u16,
    ahKeys: *mut HKEY,
    cKeys: u32,
    pclsidDefault: *const CLSID,
    pDataObject: *mut IDataObject,
    pShellBrowser: *mut IShellBrowser,
    pszStartPage: *const u16,
) -> i32 {
    UNREFERENCED_PARAMETER!(pShellBrowser);
    SHELL32_OpenPropSheet(pszCaption, ahKeys, cKeys, pclsidDefault, pDataObject, pszStartPage)
}
#[no_mangle]
pub unsafe extern "C" fn SH_CreatePropertySheetPageEx(
    wDialogId: WORD,
    pfnDlgProc: DLGPROC,
    lParam: LPARAM,
    pwszTitle: LPCWSTR,
    Callback: LPFNPSPCALLBACK,
) -> HPROPSHEETPAGE {
    let mut Page: PROPSHEETPAGEW = core::mem::zeroed();
    Page.dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as u32;
    Page.dwFlags = PSP_DEFAULT;
    Page.hInstance = shell32_hInstance;
    Page.pszTemplate = MAKEINTRESOURCEW(wDialogId as UINT) as *const _;
    Page.pfnDlgProc = Some(pfnDlgProc);
    Page.lParam = lParam;
    Page.pszTitle = pwszTitle;
    Page.pfnCallback = core::mem::transmute::<*mut c_void, Option<LPFNPSPCALLBACKW>>(Callback);

    if !pwszTitle.is_null() {
        Page.dwFlags |= PSP_USETITLE;
    }
    if !Callback.is_null() {
        Page.dwFlags |= PSP_USECALLBACK;
    }

    CreatePropertySheetPageW(core::ptr::addr_of!(Page))
}
#[no_mangle]
pub unsafe extern "C" fn SH_CreatePropertySheetPage(
    wDialogId: WORD,
    pfnDlgProc: DLGPROC,
    lParam: LPARAM,
    pwszTitle: LPCWSTR,
) -> HPROPSHEETPAGE {
    SH_CreatePropertySheetPageEx(wDialogId, pfnDlgProc, lParam, pwszTitle, core::ptr::null_mut())
}
