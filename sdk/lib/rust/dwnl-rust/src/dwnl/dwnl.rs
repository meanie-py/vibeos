// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_5B4354824105: [u16; 10] = [0x0061u16, 0x006E, 0x006F, 0x006E, 0x0079, 0x006D, 0x006F, 0x0075, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_80823D0FA1FA: [u16; 43] = [
    0x002Du16, 0x002D, 0x0025, 0x0064, 0x002D, 0x0025, 0x0030, 0x0032, 0x0064, 0x002D,
    0x0025, 0x0030, 0x0032, 0x0064, 0x0020, 0x0025, 0x0030, 0x0032, 0x0064, 0x003A,
    0x0025, 0x0030, 0x0032, 0x0064, 0x003A, 0x0025, 0x0030, 0x0032, 0x0064, 0x002D,
    0x002D, 0x0020, 0x0025, 0x0073, 0x000A, 0x0009, 0x003D, 0x003E, 0x0020, 0x0025,
    0x0073, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_DAEF6F579BB0: [u16; 7] = [0x000Du16, 0x0025, 0x002A, 0x002E, 0x002A, 0x0073, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// URLDownloadToFile -> URLDownloadToFileW (UNICODE branch assumed; the A variant is URLDownloadToFileA when UNICODE is undefined)

pub const DWNL_E_LASTERROR: i32 = 0;
pub const DWNL_E_NEEDTARGETFILENAME: i32 = -1;
pub const DWNL_E_UNSUPPORTEDSCHEME: i32 = -2;

#[repr(C)]
pub struct CBindStatusCallback {
    pub lpIBindStatusCallbackVtbl: *const IBindStatusCallbackVtbl,
    pub ref_: LONG,
    pub szHostName: [WCHAR; INTERNET_MAX_HOST_NAME_LENGTH as usize + 1],
    pub szMimeType: [WCHAR; 128],
    pub Size: UINT64,
    pub Progress: UINT64,
    // UINT bResolving:1 (bit 0), bConnecting:1 (bit 1), bSendingReq:1 (bit 2), bBeginTransfer:1 (bit 3)
    pub bitfields: UINT,
}

pub type PCBindStatusCallback = *mut CBindStatusCallback;

unsafe extern "C" fn CBindStatusCallback_Destroy(This: *mut CBindStatusCallback) {}

#[no_mangle]
unsafe extern "C" fn write_status_v(lpFmt: LPCWSTR, ap_list: *mut ::core::ffi::c_void) {
    let mut args: va_list = *(ap_list as *mut va_list);
    let mut szTxt: [WCHAR; 128] = [0; 128];
    let mut csbi: CONSOLE_SCREEN_BUFFER_INFO = core::mem::zeroed();
    let mut args_0: va_list = *(ap_list as *mut va_list);



    StringCbVPrintfW(
        core::ptr::addr_of_mut!(szTxt) as *mut WCHAR,
        core::mem::size_of::<[WCHAR; 128]>() as size_t,
        lpFmt,
        (args_0).cast(),
    );


    if GetConsoleScreenBufferInfo(
        GetStdHandle(STD_OUTPUT_HANDLE),
        core::ptr::addr_of_mut!(csbi),
    ) != 0
    {
        ConPrintf(
            StdOut,
            RUSTCONV_WSTR_DAEF6F579BB0.as_ptr(),
            -((csbi.dwSize.X as i32) - 1),
            (csbi.dwSize.X as i32) - 1,
            core::ptr::addr_of_mut!(szTxt) as *mut WCHAR,
        );
    } else {
        ConPuts(
            StdOut,
            core::ptr::addr_of_mut!(szTxt) as *mut WCHAR as PCWSTR,
        );
    }
}



unsafe extern "C" fn CBindStatusCallback_UpdateProgress(This: *mut CBindStatusCallback) {
    let mut szMessage: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    /* FIXME: better output */
    if (*This).Size != 0 {
        let mut Percentage: UINT;

        Percentage = (*This)
            .Progress
            .wrapping_mul(100)
            .wrapping_div((*This).Size) as UINT;
        // If percentage is greater than 99% but sizes don't match, do a failsafe.
        if (Percentage > 99) && ((*This).Progress != (*This).Size) {
            Percentage = 99;
        }

        LoadStringW(
            core::ptr::null_mut(),
            IDS_BYTES_DOWNLOADED_FULL,
            core::ptr::addr_of_mut!(szMessage) as LPWSTR,
            (_countof!(szMessage)) as i32,
        );

        write_status(
            core::ptr::addr_of_mut!(szMessage) as *mut WCHAR as LPCWSTR,
            Percentage,
            (*This).Progress,
        );
    } else {
        LoadStringW(
            core::ptr::null_mut(),
            IDS_BYTES_DOWNLOADED,
            core::ptr::addr_of_mut!(szMessage) as LPWSTR,
            (_countof!(szMessage)) as i32,
        );

        /* Unknown size */
        write_status(
            core::ptr::addr_of_mut!(szMessage) as *mut WCHAR as LPCWSTR,
            (*This).Progress,
        );
    }
}

unsafe extern "system" fn CBindStatusCallback_AddRef(iface: *mut IBindStatusCallback) -> ULONG {
    let This: *mut CBindStatusCallback = (iface as ULONG_PTR)
        .wrapping_sub(0 as ULONG_PTR) as *mut CBindStatusCallback;
    let mut ret: ULONG;
    ret = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_) as PLONG) as ULONG;
    return ret;
}

unsafe extern "system" fn CBindStatusCallback_Release(iface: *mut IBindStatusCallback) -> ULONG {
    let This: *mut CBindStatusCallback = interface_to_impl(iface, core::mem::size_of::<IBindStatusCallback>());
    let mut ret: ULONG;

    ret = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_) as PLONG) as ULONG;
    if ret == 0 {
        CBindStatusCallback_Destroy(This);

        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            This as PVOID,
        );
    }

    return ret;
}
unsafe extern "system" fn CBindStatusCallback_QueryInterface(
    iface: *mut IBindStatusCallback,
    iid: REFIID,
    pvObject: *mut PVOID,
) -> HRESULT {
    let This: *mut CBindStatusCallback = interface_to_impl(iface, core::mem::size_of::<IBindStatusCallback>());

    *pvObject = core::ptr::null_mut();

    if IsEqualIID!(iid, core::ptr::addr_of!(IID_IBindStatusCallback)) != 0
        || IsEqualIID!(iid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *pvObject = impl_to_interface(This, core::mem::size_of::<IBindStatusCallback>()) as PVOID;
    } else {
        return E_NOINTERFACE;
    }

    CBindStatusCallback_AddRef(iface);
    return S_OK;
}
unsafe extern "system" fn CBindStatusCallback_OnStartBinding(
    iface: *mut IBindStatusCallback,
    dwReserved: DWORD,
    pib: *mut IBinding,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn CBindStatusCallback_GetPriority(
    iface: *mut IBindStatusCallback,
    pnPriority: *mut LONG,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn CBindStatusCallback_OnLowResource(
    iface: *mut IBindStatusCallback,
    reserved: DWORD,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "system" fn CBindStatusCallback_OnProgress(
    iface: *mut IBindStatusCallback,
    ulProgress: ULONG,
    ulProgressMax: ULONG,
    ulStatusCode: ULONG,
    szStatusText: LPCWSTR,
) -> HRESULT {
    let mut This: *mut CBindStatusCallback = ((iface as ULONG_PTR)
        .wrapping_sub(core::mem::offset_of!(CBindStatusCallback, lpIBindStatusCallbackVtbl) as ULONG_PTR))
        as *mut CBindStatusCallback;
    match ulStatusCode {
        BINDSTATUS_FINDINGRESOURCE => {
            if (*This).bResolving() == 0 {
                wcscpy(
                    core::ptr::addr_of_mut!((*This).szHostName) as *mut WCHAR,
                    szStatusText,
                );
                (*This).set_bResolving(TRUE as UINT);

                ConResPrintf(
                    StdOut,
                    IDS_RESOLVING,
                    core::ptr::addr_of_mut!((*This).szHostName) as *mut WCHAR,
                );
            }
        }
        BINDSTATUS_CONNECTING => {
            (*This).set_bConnecting(TRUE as UINT);
            (*This).set_bSendingReq(FALSE as UINT);
            (*This).set_bBeginTransfer(FALSE as UINT);
            (*This).szMimeType[0 as usize] = 0x0000u16;
            if (*This).bResolving() != 0 {
                ConResPrintf(StdOut, IDS_DONE);
                ConResPrintf(
                    StdOut,
                    IDS_CONNECTING_TO_FULL,
                    core::ptr::addr_of_mut!((*This).szHostName) as *mut WCHAR,
                    szStatusText,
                );
            } else {
                ConResPrintf(StdOut, IDS_CONNECTING_TO, szStatusText);
            }
        }
        BINDSTATUS_REDIRECTING => {
            (*This).set_bResolving(FALSE as UINT);
            (*This).set_bConnecting(FALSE as UINT);
            (*This).set_bSendingReq(FALSE as UINT);
            (*This).set_bBeginTransfer(FALSE as UINT);
            (*This).szMimeType[0 as usize] = 0x0000u16;
            ConResPrintf(StdOut, IDS_REDIRECTING_TO, szStatusText);
        }
        BINDSTATUS_SENDINGREQUEST => {
            (*This).set_bBeginTransfer(FALSE as UINT);
            (*This).szMimeType[0 as usize] = 0x0000u16;
            if (*This).bResolving() != 0 || (*This).bConnecting() != 0 {
                ConResPrintf(StdOut, IDS_DONE);
            }

            if (*This).bSendingReq() == 0 {
                ConResPrintf(StdOut, IDS_SEND_REQUEST);
            }

            (*This).set_bSendingReq(TRUE as UINT);
        }
        BINDSTATUS_MIMETYPEAVAILABLE => {
            wcscpy(
                core::ptr::addr_of_mut!((*This).szMimeType) as *mut WCHAR,
                szStatusText,
            );
        }
        BINDSTATUS_BEGINDOWNLOADDATA => {
            (*This).Progress = ulProgress as UINT64;
            (*This).Size = ulProgressMax as UINT64;

            if (*This).bSendingReq() != 0 {
                ConResPrintf(StdOut, IDS_DONE);
            }

            if (*This).bBeginTransfer() == 0 && (*This).Size != 0 as UINT64 {
                if (*This).szMimeType[0 as usize] != 0x0000u16 {
                    ConResPrintf(
                        StdOut,
                        IDS_LENGTH_FULL,
                        (*This).Size,
                        core::ptr::addr_of_mut!((*This).szMimeType) as *mut WCHAR,
                    );
                } else {
                    ConResPrintf(StdOut, IDS_LENGTH, (*This).Size);
                }
            }

            ConPuts(StdOut, RUSTCONV_WSTR_075DE2B906DB.as_ptr());

            (*This).set_bBeginTransfer(TRUE as UINT);
        }
        BINDSTATUS_ENDDOWNLOADDATA => {
            /* Since download is completed, update progress one last time to be at 100% */
            (*This).Progress = (*This).Size; // Ensure progress == total size
            CBindStatusCallback_UpdateProgress(This); // Show 100% progress now

            ConResPrintf(StdOut, IDS_FILE_SAVED);
        }
        BINDSTATUS_DOWNLOADINGDATA => {
            (*This).Progress = ulProgress as UINT64;
            (*This).Size = ulProgressMax as UINT64;

            CBindStatusCallback_UpdateProgress(This);
        }
        _ => {}
    }

    return S_OK as HRESULT;
}
unsafe extern "system" fn CBindStatusCallback_OnStopBinding(
    iface: *mut IBindStatusCallback,
    hresult: HRESULT,
    szError: LPCWSTR,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn CBindStatusCallback_GetBindInfo(
    iface: *mut IBindStatusCallback,
    grfBINDF: *mut DWORD,
    pbindinfo: *mut BINDINFO,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn CBindStatusCallback_OnDataAvailable(
    iface: *mut IBindStatusCallback,
    grfBSCF: DWORD,
    dwSize: DWORD,
    pformatetc: *mut FORMATETC,
    pstgmed: *mut STGMEDIUM,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn CBindStatusCallback_OnObjectAvailable(
    iface: *mut IBindStatusCallback,
    riid: REFIID,
    punk: *mut IUnknown,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn CreateBindStatusCallback() -> *mut IBindStatusCallback {
    let mut This: *mut CBindStatusCallback = core::ptr::null_mut::<CBindStatusCallback>();

    This = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<CBindStatusCallback>() as SIZE_T,
    ) as *mut CBindStatusCallback;
    if This.is_null() {
        return core::ptr::null_mut::<IBindStatusCallback>();
    }

    (*This).lpIBindStatusCallbackVtbl =
        core::ptr::addr_of!(vtblIBindStatusCallback) as *const IBindStatusCallbackVtbl;
    (*This).ref_ = 1 as LONG;

    return core::ptr::addr_of_mut!((*This).lpIBindStatusCallbackVtbl) as *mut IBindStatusCallback;
}

unsafe extern "C" fn get_display_url(
    mut purl: LPURL_COMPONENTS,
    mut szBuffer: LPWSTR,
    mut pdwBufferSize: PDWORD,
) -> ::core::ffi::c_int {
    let mut urlc: URL_COMPONENTS = core::mem::zeroed();

    /* Hide the password */
    urlc = *purl;
    urlc.lpszPassword = core::ptr::null_mut();
    urlc.dwPasswordLength = 0;

    if InternetCreateUrl(
        core::ptr::addr_of_mut!(urlc),
        ICU_ESCAPE,
        szBuffer,
        pdwBufferSize,
    ) == 0
    {
        return DWNL_E_LASTERROR;
    }

    return 1;
}

unsafe extern "C" fn download_file(mut pszUrl: LPCWSTR, mut pszFile: LPCWSTR) -> ::core::ffi::c_int {
    let mut szScheme: [WCHAR; (INTERNET_MAX_SCHEME_LENGTH + 1) as _] = [0; (INTERNET_MAX_SCHEME_LENGTH + 1) as _];
    let mut szHostName: [WCHAR; (INTERNET_MAX_HOST_NAME_LENGTH + 1) as _] = [0; (INTERNET_MAX_HOST_NAME_LENGTH + 1) as _];
    let mut szUserName: [WCHAR; (INTERNET_MAX_USER_NAME_LENGTH + 1) as _] = [0; (INTERNET_MAX_USER_NAME_LENGTH + 1) as _];
    let mut szPassword: [WCHAR; (INTERNET_MAX_PASSWORD_LENGTH + 1) as _] = [0; (INTERNET_MAX_PASSWORD_LENGTH + 1) as _];
    let mut szUrlPath: [WCHAR; (INTERNET_MAX_PATH_LENGTH + 1) as _] = [0; (INTERNET_MAX_PATH_LENGTH + 1) as _];
    let mut szExtraInfo: [WCHAR; (INTERNET_MAX_PATH_LENGTH + 1) as _] = [0; (INTERNET_MAX_PATH_LENGTH + 1) as _];
    let mut szUrl: [WCHAR; (INTERNET_MAX_URL_LENGTH + 1) as _] = [0; (INTERNET_MAX_URL_LENGTH + 1) as _];
    let mut dwUrlLen: DWORD = 0;
    let mut pszFilePart: LPTSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut urlc: URL_COMPONENTS = core::mem::zeroed();
    let mut pbsc: *mut IBindStatusCallback = ::core::ptr::null_mut::<IBindStatusCallback>();
    let mut iRet: ::core::ffi::c_int = 0;
    let mut sysTime: SYSTEMTIME = core::mem::zeroed();

    if !pszFile.is_null() && *pszFile.offset(0 as isize) == 0x0000u16 {
        pszFile = ::core::ptr::null::<WCHAR>();
    }

    urlc.dwStructSize = ::core::mem::size_of::<URL_COMPONENTS>() as DWORD;
    urlc.lpszScheme = ::core::ptr::addr_of_mut!(szScheme) as *mut WCHAR as LPWSTR;
    urlc.dwSchemeLength = _countof!(szScheme) as DWORD;
    urlc.lpszHostName = ::core::ptr::addr_of_mut!(szHostName) as *mut WCHAR as LPWSTR;
    urlc.dwHostNameLength = _countof!(szHostName) as DWORD;
    urlc.lpszUserName = ::core::ptr::addr_of_mut!(szUserName) as *mut WCHAR as LPWSTR;
    urlc.dwUserNameLength = _countof!(szUserName) as DWORD;
    urlc.lpszPassword = ::core::ptr::addr_of_mut!(szPassword) as *mut WCHAR as LPWSTR;
    urlc.dwPasswordLength = _countof!(szPassword) as DWORD;
    urlc.lpszUrlPath = ::core::ptr::addr_of_mut!(szUrlPath) as *mut WCHAR as LPWSTR;
    urlc.dwUrlPathLength = _countof!(szUrlPath) as DWORD;
    urlc.lpszExtraInfo = ::core::ptr::addr_of_mut!(szExtraInfo) as *mut WCHAR as LPWSTR;
    urlc.dwExtraInfoLength = _countof!(szExtraInfo) as DWORD;
    if InternetCrackUrl(
        pszUrl,
        wcslen(pszUrl) as DWORD,
        ICU_ESCAPE,
        ::core::ptr::addr_of_mut!(urlc),
    ) == 0
    {
        return DWNL_E_LASTERROR;
    }

    if urlc.nScheme as ::core::ffi::c_int != INTERNET_SCHEME_FTP as ::core::ffi::c_int
        && urlc.nScheme as ::core::ffi::c_int != INTERNET_SCHEME_GOPHER as ::core::ffi::c_int
        && urlc.nScheme as ::core::ffi::c_int != INTERNET_SCHEME_HTTP as ::core::ffi::c_int
        && urlc.nScheme as ::core::ffi::c_int != INTERNET_SCHEME_HTTPS as ::core::ffi::c_int
    {
        return DWNL_E_UNSUPPORTEDSCHEME;
    }

    if urlc.nScheme as ::core::ffi::c_int == INTERNET_SCHEME_FTP as ::core::ffi::c_int
        && urlc.dwUserNameLength == 0
        && urlc.dwPasswordLength == 0
    {
        wcscpy(
            ::core::ptr::addr_of_mut!(szUserName) as *mut WCHAR,
            RUSTCONV_WSTR_5B4354824105.as_ptr(),
        );
        urlc.dwUserNameLength = wcslen(::core::ptr::addr_of_mut!(szUserName) as *mut WCHAR) as DWORD;
    }

    /* FIXME: Get file name from server */
    if urlc.dwUrlPathLength == 0 && pszFile.is_null() {
        return DWNL_E_NEEDTARGETFILENAME;
    }

    pszFilePart = wcsrchr(
        ::core::ptr::addr_of_mut!(szUrlPath) as *mut WCHAR,
        0x002Fu16,
    ) as LPTSTR;
    if !pszFilePart.is_null() {
        pszFilePart = pszFilePart.offset(1);
    }

    if pszFilePart.is_null() && pszFile.is_null() {
        return DWNL_E_NEEDTARGETFILENAME;
    }

    if pszFile.is_null() {
        pszFile = pszFilePart as LPCWSTR;
    }

    if urlc.dwUserNameLength == 0 {
        urlc.lpszUserName = ::core::ptr::null_mut::<WCHAR>();
    }

    if urlc.dwPasswordLength == 0 {
        urlc.lpszPassword = ::core::ptr::null_mut::<WCHAR>();
    }

    /* Generate the URL to be displayed (without a password) */
    dwUrlLen = _countof!(szUrl) as DWORD;
    iRet = get_display_url(
        ::core::ptr::addr_of_mut!(urlc),
        ::core::ptr::addr_of_mut!(szUrl) as *mut WCHAR,
        ::core::ptr::addr_of_mut!(dwUrlLen),
    );
    if iRet <= 0 {
        return iRet;
    }

    GetLocalTime(::core::ptr::addr_of_mut!(sysTime));

    ConPrintf(
        StdOut,
        RUSTCONV_WSTR_80823D0FA1FA.as_ptr(),
        sysTime.wYear as ::core::ffi::c_int,
        sysTime.wMonth as ::core::ffi::c_int,
        sysTime.wDay as ::core::ffi::c_int,
        sysTime.wHour as ::core::ffi::c_int,
        sysTime.wMinute as ::core::ffi::c_int,
        sysTime.wSecond as ::core::ffi::c_int,
        ::core::ptr::addr_of_mut!(szUrl) as *mut WCHAR,
        pszFile,
    );

    /* Generate the URL to download */
    dwUrlLen = _countof!(szUrl) as DWORD;
    if InternetCreateUrl(
        ::core::ptr::addr_of_mut!(urlc),
        ICU_ESCAPE,
        ::core::ptr::addr_of_mut!(szUrl) as LPWSTR,
        ::core::ptr::addr_of_mut!(dwUrlLen),
    ) == 0
    {
        return DWNL_E_LASTERROR;
    }

    pbsc = CreateBindStatusCallback();
    if pbsc.is_null() {
        return DWNL_E_LASTERROR;
    }

    if !(URLDownloadToFile(
        ::core::ptr::null_mut::<IUnknown>(),
        ::core::ptr::addr_of_mut!(szUrl) as *mut WCHAR as LPCWSTR,
        pszFile,
        0,
        pbsc as LPBINDSTATUSCALLBACK,
    ) >= 0)
    {
        IBindStatusCallback_Release(pbsc);
        return DWNL_E_LASTERROR; /* FIXME */
    }

    IBindStatusCallback_Release(pbsc);
    return 1;
}

unsafe extern "C" fn print_err(iErr: ::core::ffi::c_int) -> ::core::ffi::c_int {
    write_status(RUSTCONV_WSTR_96A296D224F2.as_ptr());

    if iErr == DWNL_E_LASTERROR {
        if GetLastError() == ERROR_SUCCESS as DWORD {
            /* File not found */
            ConResPrintf(&raw mut csStdErr, IDS_ERROR_DOWNLOAD as UINT);
        } else {
            /* Display last error code */
            ConResPrintf(
                &raw mut csStdErr,
                IDS_ERROR_CODE as UINT,
                GetLastError(),
            );
        }
    } else {
        match iErr {
            DWNL_E_NEEDTARGETFILENAME => {
                ConResPrintf(&raw mut csStdErr, IDS_ERROR_FILENAME as UINT);
            }
            DWNL_E_UNSUPPORTEDSCHEME => {
                ConResPrintf(&raw mut csStdErr, IDS_ERROR_PROTOCOL as UINT);
            }
            _ => {}
        }
    }

    return 1 as ::core::ffi::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: ::core::ffi::c_int, argv: *mut *mut WCHAR) -> ::core::ffi::c_int {
    let mut iErr: ::core::ffi::c_int;
    let mut iRet: ::core::ffi::c_int = 0;

    /* Initialize the Console Standard Streams */
    ConInitStdStreams();

    if argc != 2 && argc != 3 {
        ConResPrintf(core::ptr::addr_of_mut!(csStdOut), IDS_USAGE as UINT);
        return 2;
    }

    iErr = download_file(
        *argv.offset(1 as isize) as LPCWSTR,
        (if argc == 3 {
            *argv.offset(2 as isize)
        } else {
            core::ptr::null_mut::<WCHAR>()
        }) as LPCWSTR,
    );
    if iErr <= 0 {
        iRet = print_err(iErr);
    }

    return iRet;
}

// COM vtable tables — generated from dwnl.c by vtbl_gen.py
/// dwnl.c:302 `static const IBindStatusCallbackVtbl vtblIBindStatusCallback`  (slot order: com-sys/bindgen_i386.rs)
pub static vtblIBindStatusCallback: IBindStatusCallbackVtbl = IBindStatusCallbackVtbl {
    QueryInterface: Some(CBindStatusCallback_QueryInterface),
    AddRef: Some(CBindStatusCallback_AddRef),
    Release: Some(CBindStatusCallback_Release),
    OnStartBinding: Some(CBindStatusCallback_OnStartBinding),
    GetPriority: Some(CBindStatusCallback_GetPriority),
    OnLowResource: Some(CBindStatusCallback_OnLowResource),
    OnProgress: Some(CBindStatusCallback_OnProgress),
    OnStopBinding: Some(CBindStatusCallback_OnStopBinding),
    GetBindInfo: Some(CBindStatusCallback_GetBindInfo),
    OnDataAvailable: Some(CBindStatusCallback_OnDataAvailable),
    OnObjectAvailable: Some(CBindStatusCallback_OnObjectAvailable),
};
