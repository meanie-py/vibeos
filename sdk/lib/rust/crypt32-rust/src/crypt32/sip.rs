use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_02FDEA4222E8: [u16; 63] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0043,
    0x0072, 0x0079, 0x0070, 0x0074, 0x006F, 0x0067, 0x0072, 0x0061, 0x0070, 0x0068,
    0x0079, 0x005C, 0x004F, 0x0049, 0x0044, 0x005C, 0x0045, 0x006E, 0x0063, 0x006F,
    0x0064, 0x0069, 0x006E, 0x0067, 0x0054, 0x0079, 0x0070, 0x0065, 0x0020, 0x0030,
    0x005C, 0x0043, 0x0072, 0x0079, 0x0070, 0x0074, 0x0053, 0x0049, 0x0050, 0x0044,
    0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_1FF4217A2E88: [u16; 18] = [
    0x0050u16, 0x0075, 0x0074, 0x0053, 0x0069, 0x0067, 0x006E, 0x0065, 0x0064, 0x0044,
    0x0061, 0x0074, 0x0061, 0x004D, 0x0073, 0x0067, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_330AE2DDC0DF: [u16; 76] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0043,
    0x0072, 0x0079, 0x0070, 0x0074, 0x006F, 0x0067, 0x0072, 0x0061, 0x0070, 0x0068,
    0x0079, 0x005C, 0x004F, 0x0049, 0x0044, 0x005C, 0x0045, 0x006E, 0x0063, 0x006F,
    0x0064, 0x0069, 0x006E, 0x0067, 0x0054, 0x0079, 0x0070, 0x0065, 0x0020, 0x0030,
    0x005C, 0x0043, 0x0072, 0x0079, 0x0070, 0x0074, 0x0053, 0x0049, 0x0050, 0x0044,
    0x006C, 0x006C, 0x0049, 0x0073, 0x004D, 0x0079, 0x0046, 0x0069, 0x006C, 0x0065,
    0x0054, 0x0079, 0x0070, 0x0065, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_5FCBA2C2CE88: [u16; 20] = [
    0x0056u16, 0x0065, 0x0072, 0x0069, 0x0066, 0x0079, 0x0049, 0x006E, 0x0064, 0x0069,
    0x0072, 0x0065, 0x0063, 0x0074, 0x0044, 0x0061, 0x0074, 0x0061, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_6184A7B89A8A: [u16; 4] = [0x0044u16, 0x006C, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_6FB3BB38F1CD: [u16; 18] = [
    0x0047u16, 0x0065, 0x0074, 0x0053, 0x0069, 0x0067, 0x006E, 0x0065, 0x0064, 0x0044,
    0x0061, 0x0074, 0x0061, 0x004D, 0x0073, 0x0067, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_71A628F66082: [u16; 20] = [
    0x0043u16, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0049, 0x006E, 0x0064, 0x0069,
    0x0072, 0x0065, 0x0063, 0x0074, 0x0044, 0x0061, 0x0074, 0x0061, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_73E2FEDC0237: [u16; 9] = [0x0046u16, 0x0075, 0x006E, 0x0063, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_8261CC28DF28: [u16; 15] = [
    0x0049u16, 0x0073, 0x004D, 0x0079, 0x0046, 0x0069, 0x006C, 0x0065, 0x0054, 0x0079,
    0x0070, 0x0065, 0x0032, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_8759990C0E16: [u16; 21] = [
    0x0052u16, 0x0065, 0x006D, 0x006F, 0x0076, 0x0065, 0x0053, 0x0069, 0x0067, 0x006E,
    0x0065, 0x0064, 0x0044, 0x0061, 0x0074, 0x0061, 0x004D, 0x0073, 0x0067, 0x005C,
    0x0000,
];
pub static RUSTCONV_WSTR_A018BC58C69A: [u16; 14] = [
    0x0049u16, 0x0073, 0x004D, 0x0079, 0x0046, 0x0069, 0x006C, 0x0065, 0x0054, 0x0079,
    0x0070, 0x0065, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_C7DC8D73B849: [u16; 77] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0043,
    0x0072, 0x0079, 0x0070, 0x0074, 0x006F, 0x0067, 0x0072, 0x0061, 0x0070, 0x0068,
    0x0079, 0x005C, 0x004F, 0x0049, 0x0044, 0x005C, 0x0045, 0x006E, 0x0063, 0x006F,
    0x0064, 0x0069, 0x006E, 0x0067, 0x0054, 0x0079, 0x0070, 0x0065, 0x0020, 0x0030,
    0x005C, 0x0043, 0x0072, 0x0079, 0x0070, 0x0074, 0x0053, 0x0049, 0x0050, 0x0044,
    0x006C, 0x006C, 0x0049, 0x0073, 0x004D, 0x0079, 0x0046, 0x0069, 0x006C, 0x0065,
    0x0054, 0x0079, 0x0070, 0x0065, 0x0032, 0x005C, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_2 {
    pub Flags: DWORD,
    pub CreatorBackTraceIndexHigh: WORD,
    pub SpareWORD: WORD,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub WineDebugString: DWORD_PTR,
    pub Spare: [DWORD_PTR; 1],
    pub c2rust_unnamed: C2RustUnnamed_2,
}

static mut providers: list = unsafe {
    list {
        next: &raw const providers as *mut list,
        prev: &raw const providers as *mut list,
    }
};

static mut providers_cs_debug: CRITICAL_SECTION_DEBUG = unsafe { _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut providers_cs,
        ProcessLocksList: _LIST_ENTRY {
            Flink: &raw mut providers_cs_debug.ProcessLocksList,
            Blink: &raw mut providers_cs_debug.ProcessLocksList,
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    }};

static mut providers_cs: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const providers_cs_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as ::core::ffi::c_int) as LONG,
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(crypt);  (debug channel registration macro; no Rust declaration emitted)

#[repr(C)]
pub struct WINE_SIP_PROVIDER {
    pub subject: GUID,
    pub info: SIP_DISPATCH_INFO,
    pub entry: list,
}
pub type PWINE_SIP_PROVIDER = *mut WINE_SIP_PROVIDER;

unsafe extern "C" fn CRYPT_guid2wstr(guid: *const GUID, wstr: LPWSTR) {
    let mut str: [::core::ffi::c_char; 40] = [0; 40];
    sprintf(
        core::ptr::addr_of_mut!(str) as *mut ::core::ffi::c_char,
        b"{%08lX-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}\0".as_ptr() as *const ::core::ffi::c_char,
        (*guid).Data1,
        (*guid).Data2 as ::core::ffi::c_int,
        (*guid).Data3 as ::core::ffi::c_int,
        (*guid).Data4[0] as ::core::ffi::c_int,
        (*guid).Data4[1] as ::core::ffi::c_int,
        (*guid).Data4[2] as ::core::ffi::c_int,
        (*guid).Data4[3] as ::core::ffi::c_int,
        (*guid).Data4[4] as ::core::ffi::c_int,
        (*guid).Data4[5] as ::core::ffi::c_int,
        (*guid).Data4[6] as ::core::ffi::c_int,
        (*guid).Data4[7] as ::core::ffi::c_int,
    );
    MultiByteToWideChar(
        CP_ACP,
        0,
        core::ptr::addr_of_mut!(str) as *mut ::core::ffi::c_char as LPCSTR,
        -1,
        wstr,
        40,
    );
}

unsafe extern "C" fn CRYPT_SIPDeleteFunction(guid: *const GUID, szKey: LPCWSTR) -> LONG {
    let mut szFullKey: [WCHAR; 0x100] = [0; 0x100];
    let mut r: LONG = (ERROR_SUCCESS as i32);

    /* max length of szFullKey depends on our code only, so we won't overrun */
    lstrcpyW(szFullKey.as_mut_ptr(), RUSTCONV_WSTR_02FDEA4222E8.as_ptr());
    lstrcatW(szFullKey.as_mut_ptr(), szKey);
    CRYPT_guid2wstr(
        guid,
        szFullKey.as_mut_ptr().offset(lstrlenW(szFullKey.as_ptr()) as isize),
    );

    r = RegDeleteKeyW(HKEY_LOCAL_MACHINE, szFullKey.as_ptr()) as LONG;

    return r;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSIPRemoveProvider(pgProv: *mut GUID) -> BOOL {
    let mut r: LONG = (ERROR_SUCCESS as i32);
    let mut remove_error: LONG = (ERROR_SUCCESS as i32);

    if pgProv.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    r = CRYPT_SIPDeleteFunction(pgProv, RUSTCONV_WSTR_1FF4217A2E88.as_ptr());
    if r != (ERROR_SUCCESS as LONG) {
        remove_error = r;
    }

    r = CRYPT_SIPDeleteFunction(pgProv, RUSTCONV_WSTR_6FB3BB38F1CD.as_ptr());
    if r != (ERROR_SUCCESS as LONG) {
        remove_error = r;
    }

    r = CRYPT_SIPDeleteFunction(pgProv, RUSTCONV_WSTR_8759990C0E16.as_ptr());
    if r != (ERROR_SUCCESS as LONG) {
        remove_error = r;
    }

    r = CRYPT_SIPDeleteFunction(pgProv, RUSTCONV_WSTR_71A628F66082.as_ptr());
    if r != (ERROR_SUCCESS as LONG) {
        remove_error = r;
    }

    r = CRYPT_SIPDeleteFunction(pgProv, RUSTCONV_WSTR_5FCBA2C2CE88.as_ptr());
    if r != (ERROR_SUCCESS as LONG) {
        remove_error = r;
    }

    r = CRYPT_SIPDeleteFunction(pgProv, RUSTCONV_WSTR_A018BC58C69A.as_ptr());
    if r != (ERROR_SUCCESS as LONG) {
        remove_error = r;
    }

    r = CRYPT_SIPDeleteFunction(pgProv, RUSTCONV_WSTR_8261CC28DF28.as_ptr());
    if r != (ERROR_SUCCESS as LONG) {
        remove_error = r;
    }

    if remove_error != (ERROR_SUCCESS as LONG) {
        SetLastError((remove_error) as DWORD);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn CRYPT_SIPWriteFunction(
    guid: *const GUID,
    szKey: LPCWSTR,
    szDll: LPCWSTR,
    szFunction: LPCWSTR,
) -> LONG {
    let mut szFullKey: [WCHAR; 0x100] = [0; 0x100];
    let mut r: LONG = ERROR_SUCCESS as LONG;
    let mut hKey: HKEY = core::ptr::null_mut();

    if szFunction.is_null() {
        return ERROR_SUCCESS as LONG;
    }

    /* max length of szFullKey depends on our code only, so we won't overrun */
    lstrcpyW(addr_of_mut!(szFullKey) as *mut WCHAR as LPWSTR, RUSTCONV_WSTR_02FDEA4222E8.as_ptr());
    lstrcatW(addr_of_mut!(szFullKey) as *mut WCHAR as LPWSTR, szKey);
    CRYPT_guid2wstr(
        guid,
        (addr_of_mut!(szFullKey) as *mut WCHAR)
            .offset(lstrlenW(addr_of_mut!(szFullKey) as *mut WCHAR as LPCWSTR) as isize)
            as LPWSTR,
    );

    r = RegCreateKeyW(
        HKEY_LOCAL_MACHINE,
        addr_of_mut!(szFullKey) as *mut WCHAR as LPCWSTR,
        addr_of_mut!(hKey),
    ) as LONG;

    'error_close_key: {
        if r != ERROR_SUCCESS as LONG {
            break 'error_close_key;
        }

        /* write the values */
        r = RegSetValueExW(
            hKey,
            RUSTCONV_WSTR_73E2FEDC0237.as_ptr(),
            0 as DWORD,
            REG_SZ as DWORD,
            szFunction as *const BYTE,
            ((lstrlenW(szFunction) + 1) as usize
                * core::mem::size_of::<WCHAR>()) as DWORD,
        ) as LONG;
        if r != ERROR_SUCCESS as LONG {
            break 'error_close_key;
        }
        r = RegSetValueExW(
            hKey,
            RUSTCONV_WSTR_6184A7B89A8A.as_ptr(),
            0 as DWORD,
            REG_SZ as DWORD,
            szDll as *const BYTE,
            ((lstrlenW(szDll) + 1) as usize
                * core::mem::size_of::<WCHAR>()) as DWORD,
        ) as LONG;
    }

    RegCloseKey(hKey);

    return r;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSIPAddProvider(psNewProv: *mut SIP_ADD_NEWPROVIDER) -> BOOL {
    let mut r: LONG = ERROR_SUCCESS as LONG;

    if psNewProv.is_null()
        || (*psNewProv).cbStruct < core::mem::offset_of!(SIP_ADD_NEWPROVIDER, pwszGetCapFuncName) as DWORD
        || (*psNewProv).pwszGetFuncName.is_null()
        || (*psNewProv).pwszPutFuncName.is_null()
        || (*psNewProv).pwszCreateFuncName.is_null()
        || (*psNewProv).pwszVerifyFuncName.is_null()
        || (*psNewProv).pwszRemoveFuncName.is_null()
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    'end_function: {
        r = CRYPT_SIPWriteFunction((*psNewProv).pgSubject, RUSTCONV_WSTR_1FF4217A2E88.as_ptr(), (*psNewProv).pwszDLLFileName, (*psNewProv).pwszPutFuncName);
        if r != ERROR_SUCCESS as LONG {
            break 'end_function;
        }
        r = CRYPT_SIPWriteFunction((*psNewProv).pgSubject, RUSTCONV_WSTR_6FB3BB38F1CD.as_ptr(), (*psNewProv).pwszDLLFileName, (*psNewProv).pwszGetFuncName);
        if r != ERROR_SUCCESS as LONG {
            break 'end_function;
        }
        r = CRYPT_SIPWriteFunction((*psNewProv).pgSubject, RUSTCONV_WSTR_8759990C0E16.as_ptr(), (*psNewProv).pwszDLLFileName, (*psNewProv).pwszRemoveFuncName);
        if r != ERROR_SUCCESS as LONG {
            break 'end_function;
        }
        r = CRYPT_SIPWriteFunction((*psNewProv).pgSubject, RUSTCONV_WSTR_71A628F66082.as_ptr(), (*psNewProv).pwszDLLFileName, (*psNewProv).pwszCreateFuncName);
        if r != ERROR_SUCCESS as LONG {
            break 'end_function;
        }
        r = CRYPT_SIPWriteFunction((*psNewProv).pgSubject, RUSTCONV_WSTR_5FCBA2C2CE88.as_ptr(), (*psNewProv).pwszDLLFileName, (*psNewProv).pwszVerifyFuncName);
        if r != ERROR_SUCCESS as LONG {
            break 'end_function;
        }
        r = CRYPT_SIPWriteFunction((*psNewProv).pgSubject, RUSTCONV_WSTR_A018BC58C69A.as_ptr(), (*psNewProv).pwszDLLFileName, (*psNewProv).pwszIsFunctionName);
        if r != ERROR_SUCCESS as LONG {
            break 'end_function;
        }
        r = CRYPT_SIPWriteFunction((*psNewProv).pgSubject, RUSTCONV_WSTR_8261CC28DF28.as_ptr(), (*psNewProv).pwszDLLFileName, (*psNewProv).pwszIsFunctionNameFmt2);
        if r != ERROR_SUCCESS as LONG {
            break 'end_function;
        }
    }

    if r != ERROR_SUCCESS as LONG {
        SetLastError(r as DWORD);
        return FALSE as BOOL;
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn CRYPT_LoadSIPFuncFromKey(key: HKEY, pLib: *mut HMODULE) -> *mut ::core::ffi::c_void {
    let mut r: LONG = 0;
    let mut size: DWORD = 0;
    let mut dllName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut functionName: [::core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lib: HMODULE = ::core::ptr::null_mut();
    let mut func: *mut ::core::ffi::c_void = ::core::ptr::null_mut();

    'end: {
        /* Read the DLL entry */
        size = ::core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
        r = RegQueryValueExW(
            key,
            RUSTCONV_WSTR_6184A7B89A8A.as_ptr(),
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            dllName.as_mut_ptr() as LPBYTE,
            &raw mut size,
        ) as LONG;
        if r != 0 {
            break 'end;
        }

        /* Read the Function entry */
        size = ::core::mem::size_of::<[::core::ffi::c_char; (MAX_PATH) as _]>() as DWORD;
        r = RegQueryValueExA(
            key,
            b"FuncName\0".as_ptr() as LPCSTR,
            ::core::ptr::null_mut::<DWORD>(),
            ::core::ptr::null_mut::<DWORD>(),
            functionName.as_mut_ptr() as LPBYTE,
            &raw mut size,
        ) as LONG;
        if r != 0 {
            break 'end;
        }

        lib = LoadLibraryW(dllName.as_ptr()) as HMODULE;
        if lib.is_null() {
            break 'end;
        }
        func = ::core::mem::transmute::<FARPROC, *mut ::core::ffi::c_void>(GetProcAddress(
            lib as HINSTANCE,
            functionName.as_ptr() as LPCSTR,
        ));
        if !func.is_null() {
            *pLib = lib;
        } else {
            FreeLibrary(lib);
        }
    }

    return func;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSIPRetrieveSubjectGuid(
    FileName: LPCWSTR,
    hFileIn: HANDLE,
    pgSubject: *mut GUID,
) -> BOOL {
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut bRet: BOOL = (FALSE as i32);
    let mut count: DWORD = 0;
    let mut zero: LARGE_INTEGER = core::mem::zeroed();
    let mut oldPos: LARGE_INTEGER = core::mem::zeroed();
    /* FIXME, find out if there is a name for this GUID */
    static unknown: GUID = GUID {
        Data1: 0xC689AAB8,
        Data2: 0x8E78,
        Data3: 0x11D0,
        Data4: [
            0x8C,
            0x47,
            0x00,
            0xC0,
            0x4F,
            0xC2,
            0x95,
            0xEE,
        ],
    };
    static cabGUID: GUID = GUID {
        Data1: 0xc689aaba,
        Data2: 0x8e78,
        Data3: 0x11d0,
        Data4: [
            0x8c,
            0x47,
            0x00,
            0xc0,
            0x4f,
            0xc2,
            0x95,
            0xee,
        ],
    };
    static catGUID: GUID = GUID {
        Data1: 0xDE351A43,
        Data2: 0x8E59,
        Data3: 0x11D0,
        Data4: [
            0x8C,
            0x47,
            0x00,
            0xC0,
            0x4F,
            0xC2,
            0x95,
            0xEE,
        ],
    };
    static dosHdr: WORD = (IMAGE_DOS_SIGNATURE as u16);
    static cabHdr: [BYTE; 4] = ['M' as BYTE, 'S' as BYTE, 'C' as BYTE, 'F' as BYTE];
    let mut hdr: [BYTE; (SIP_MAX_MAGIC_NUMBER as _)] = [0; (SIP_MAX_MAGIC_NUMBER as _)];
    let mut r: LONG = (ERROR_SUCCESS as i32);
    let mut key: HKEY = core::ptr::null_mut();

    if pgSubject.is_null() || (FileName.is_null() && hFileIn.is_null()) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    /* Set pgSubject to zero's */
    memset(
        pgSubject as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<GUID>() as usize,
    );

    if !hFileIn.is_null() {
        /* Use the given handle, make sure not to close this one ourselves */
        hFile = hFileIn;
    } else {
        hFile = CreateFileW(
            FileName,
            GENERIC_READ,
            FILE_SHARE_READ,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            core::ptr::null_mut(),
        );
        /* Last error is set by CreateFile */
        if hFile == INVALID_HANDLE_VALUE {
            return (FALSE) as BOOL;
        }
    }

    'cleanup: {
        zero.QuadPart = 0;
        SetFilePointerEx(
            hFile,
            zero,
            core::ptr::addr_of_mut!(oldPos),
            FILE_CURRENT,
        );
        SetFilePointer(
            hFile,
            0,
            core::ptr::null_mut(),
            FILE_BEGIN,
        );
        if ReadFile(
            hFile,
            (hdr.as_mut_ptr()).cast(),
            core::mem::size_of::<[BYTE; (SIP_MAX_MAGIC_NUMBER as _)]>() as DWORD,
            core::ptr::addr_of_mut!(count),
            core::ptr::null_mut(),
        ) == 0
        {
            break 'cleanup;
        }

        if count < SIP_MAX_MAGIC_NUMBER {
            SetLastError(ERROR_INVALID_PARAMETER);
            break 'cleanup;
        }

        /* As everything is in place now we start looking at the file header */
        if memcmp(
            hdr.as_mut_ptr() as *const core::ffi::c_void,
            core::ptr::addr_of!(dosHdr) as *const core::ffi::c_void,
            core::mem::size_of::<WORD>() as usize,
        ) == 0
        {
            *pgSubject = unknown;
            SetLastError((S_OK) as DWORD);
            bRet = (TRUE as i32);
            break 'cleanup;
        }
        /* Quick-n-dirty check for a cab file. */
        if memcmp(
            hdr.as_mut_ptr() as *const core::ffi::c_void,
            cabHdr.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of::<[BYTE; 4]>() as usize,
        ) == 0
        {
            *pgSubject = cabGUID;
            SetLastError((S_OK) as DWORD);
            bRet = (TRUE as i32);
            break 'cleanup;
        }
        /* If it's asn.1-encoded, it's probably a .cat file. */
        if hdr[0 as usize] == 0x30 {
            let mut fileLen: DWORD = GetFileSize(hFile, core::ptr::null_mut());

            /* Sanity-check length */
            if hdr[1 as usize] < 0x80
                && fileLen
                    == (2
                        + hdr[1 as usize] as DWORD)
            {
                *pgSubject = catGUID;
                SetLastError((S_OK) as DWORD);
                bRet = (TRUE as i32);
                break 'cleanup;
            } else if hdr[1 as usize] == 0x80 {
                /* Indefinite length, can't verify with just the header, assume it
                 * is.
                 */
                *pgSubject = catGUID;
                SetLastError((S_OK) as DWORD);
                bRet = (TRUE as i32);
                break 'cleanup;
            } else {
                let mut lenBytes: BYTE =
                    hdr[1 as usize] & 0x7f;

                if lenBytes == 1
                    && fileLen
                        == (2
                            + lenBytes as DWORD
                            + hdr[2 as usize] as DWORD)
                {
                    *pgSubject = catGUID;
                    SetLastError((S_OK) as DWORD);
                    bRet = (TRUE as i32);
                    break 'cleanup;
                } else if lenBytes == 2
                    && fileLen
                        == (2
                            + lenBytes as DWORD
                            + (((hdr[2 as usize] as DWORD)
                                << 8)
                                | hdr[3 as usize] as DWORD))
                {
                    *pgSubject = catGUID;
                    SetLastError((S_OK) as DWORD);
                    bRet = (TRUE as i32);
                    break 'cleanup;
                } else if fileLen > 0xffff {
                    /* The file size must be greater than 2 bytes in length, so
                     * assume it is a .cat file
                     */
                    *pgSubject = catGUID;
                    SetLastError((S_OK) as DWORD);
                    bRet = (TRUE as i32);
                    break 'cleanup;
                }
            }
        }

        /* Check for supported functions using CryptSIPDllIsMyFileType */
        r = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            RUSTCONV_WSTR_330AE2DDC0DF.as_ptr(),
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(key),
        );
        if r == (ERROR_SUCCESS as LONG) {
            let mut index: DWORD = 0;
            let mut size: DWORD = 0;
            let mut subKeyName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

            loop {
                size = ARRAY_SIZE!(subKeyName) as DWORD;
                let fresh15 = index;
                index = index.wrapping_add(1);
                r = RegEnumKeyExW(
                    key,
                    fresh15,
                    subKeyName.as_mut_ptr(),
                    core::ptr::addr_of_mut!(size),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if r == (ERROR_SUCCESS as LONG) {
                    let mut subKey: HKEY = core::ptr::null_mut();

                    r = RegOpenKeyExW(
                        key,
                        subKeyName.as_mut_ptr(),
                        0,
                        KEY_READ,
                        core::ptr::addr_of_mut!(subKey),
                    );
                    if r == (ERROR_SUCCESS as LONG) {
                        let mut lib: HMODULE = core::ptr::null_mut();
                        let mut isMy: pfnIsFileSupported = core::mem::transmute::<
                            *mut core::ffi::c_void,
                            pfnIsFileSupported,
                        >(CRYPT_LoadSIPFuncFromKey(
                            subKey,
                            core::ptr::addr_of_mut!(lib),
                        ));

                        if isMy.is_some() {
                            bRet = (isMy.unwrap_unchecked())(hFile, pgSubject);
                            FreeLibrary(lib);
                        }
                        RegCloseKey(subKey);
                    }
                }
                if !(bRet == 0 && r == (ERROR_SUCCESS as LONG)) {
                    break;
                }
            }
            RegCloseKey(key);
        }

        /* Check for supported functions using CryptSIPDllIsMyFileType2 */
        if bRet == 0 {
            r = RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                RUSTCONV_WSTR_C7DC8D73B849.as_ptr(),
                0,
                KEY_READ,
                core::ptr::addr_of_mut!(key),
            );
            if r == (ERROR_SUCCESS as LONG) {
                let mut index: DWORD = 0;
                let mut size: DWORD = 0;
                let mut subKeyName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

                loop {
                    size = ARRAY_SIZE!(subKeyName) as DWORD;
                    let fresh16 = index;
                    index = index.wrapping_add(1);
                    r = RegEnumKeyExW(
                        key,
                        fresh16,
                        subKeyName.as_mut_ptr(),
                        core::ptr::addr_of_mut!(size),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                    if r == (ERROR_SUCCESS as LONG) {
                        let mut subKey: HKEY = core::ptr::null_mut();

                        r = RegOpenKeyExW(
                            key,
                            subKeyName.as_mut_ptr(),
                            0,
                            KEY_READ,
                            core::ptr::addr_of_mut!(subKey),
                        );
                        if r == (ERROR_SUCCESS as LONG) {
                            let mut lib: HMODULE = core::ptr::null_mut();
                            let mut isMy2: pfnIsFileSupportedName = core::mem::transmute::<
                                *mut core::ffi::c_void,
                                pfnIsFileSupportedName,
                            >(CRYPT_LoadSIPFuncFromKey(
                                subKey,
                                core::ptr::addr_of_mut!(lib),
                            ));

                            if isMy2.is_some() {
                                bRet = (isMy2.unwrap_unchecked())(
                                    FileName as LPWSTR,
                                    pgSubject,
                                );
                                FreeLibrary(lib);
                            }
                            RegCloseKey(subKey);
                        }
                    }
                    if !(bRet == 0 && r == (ERROR_SUCCESS as LONG)) {
                        break;
                    }
                }
                RegCloseKey(key);
            }
        }

        if bRet == 0 {
            SetLastError((TRUST_E_SUBJECT_FORM_UNKNOWN) as DWORD);
        }
    }

    /* If we didn't open this one we shouldn't close it (hFile is a copy),
     * but we should reset the file pointer to its original position.
     */
    if hFileIn.is_null() {
        CloseHandle(hFile);
    } else {
        SetFilePointerEx(hFile, oldPos, core::ptr::null_mut(), FILE_BEGIN);
    }

    return bRet;
}

unsafe extern "C" fn CRYPT_OpenSIPFunctionKey(guid: *const GUID, function: LPCWSTR, key: *mut HKEY) -> LONG {
    let mut szFullKey: [WCHAR; 0x100] = [0; 0x100];

    lstrcpyW(core::ptr::addr_of_mut!(szFullKey) as LPWSTR, RUSTCONV_WSTR_02FDEA4222E8.as_ptr());
    lstrcatW(core::ptr::addr_of_mut!(szFullKey) as LPWSTR, function);
    CRYPT_guid2wstr(
        guid,
        (core::ptr::addr_of_mut!(szFullKey) as *mut WCHAR)
            .offset(lstrlenW(core::ptr::addr_of_mut!(szFullKey) as LPCWSTR) as isize) as LPWSTR,
    );
    return RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        core::ptr::addr_of_mut!(szFullKey) as *mut WCHAR as LPCWSTR,
        0,
        KEY_READ,
        key as PHKEY,
    ) as LONG;
}

unsafe extern "C" fn CRYPT_LoadSIPFunc(
    pgSubject: *const GUID,
    function: LPCWSTR,
    pLib: *mut HMODULE,
) -> *mut ::core::ffi::c_void {
    let mut r: LONG = 0;
    let mut key: HKEY = core::ptr::null_mut();
    let mut func: *mut ::core::ffi::c_void = core::ptr::null_mut();
    WINE_TRACE!("(%s, %s)\n", debugstr_guid(pgSubject as *const _GUID), debugstr_w(function as *const WCHAR));
    r = CRYPT_OpenSIPFunctionKey(pgSubject, function, &raw mut key);
    if r == 0 {
        func = CRYPT_LoadSIPFuncFromKey(key, pLib);
        RegCloseKey(key);
    }
    WINE_TRACE!("returning %p\n", func);
    return func;
}

unsafe extern "C" fn CRYPT_CacheSIP(pgSubject: *const GUID, info: *mut SIP_DISPATCH_INFO) {
    let prov: *mut WINE_SIP_PROVIDER = CryptMemAlloc(core::mem::size_of::<WINE_SIP_PROVIDER>() as ULONG) as *mut WINE_SIP_PROVIDER;

    if !prov.is_null() {
        (*prov).subject = *pgSubject;
        (*prov).info = *info;
        EnterCriticalSection(core::ptr::addr_of_mut!(providers_cs));
        list_add_tail(core::ptr::addr_of_mut!(providers), core::ptr::addr_of_mut!((*prov).entry));
        LeaveCriticalSection(core::ptr::addr_of_mut!(providers_cs));
    }
}

unsafe extern "C" fn CRYPT_GetCachedSIP(pgSubject: *const GUID) -> *mut WINE_SIP_PROVIDER {
    let mut provider: *mut WINE_SIP_PROVIDER = core::ptr::null_mut();
    let mut ret: *mut WINE_SIP_PROVIDER = core::ptr::null_mut();
    EnterCriticalSection(&raw mut providers_cs);
    provider = ((*(&raw mut providers)).next as *mut u8).offset(
        -(core::mem::offset_of!(WINE_SIP_PROVIDER, entry) as isize),
    ) as *mut WINE_SIP_PROVIDER;
    while &raw mut (*provider).entry != &raw mut providers {
        if IsEqualGUID!(pgSubject, &raw mut (*provider).subject) != false {
            break;
        }
        provider = ((*provider).entry.next as *mut u8).offset(
            -(core::mem::offset_of!(WINE_SIP_PROVIDER, entry) as isize),
        ) as *mut WINE_SIP_PROVIDER;
    }
    if !provider.is_null() && IsEqualGUID!(pgSubject, &raw mut (*provider).subject) != false {
        ret = provider;
    }
    LeaveCriticalSection(&raw mut providers_cs);
    return ret;
}

unsafe extern "C" fn CRYPT_IsSIPCached(pgSubject: *const GUID) -> BOOL {
    return (CRYPT_GetCachedSIP(pgSubject) != core::ptr::null_mut()) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn crypt_sip_free() {
    let mut prov: *mut WINE_SIP_PROVIDER = core::ptr::null_mut::<WINE_SIP_PROVIDER>();
    let mut next: *mut WINE_SIP_PROVIDER = core::ptr::null_mut::<WINE_SIP_PROVIDER>();
    prov = (providers.next as *mut u8).sub(core::mem::offset_of!(WINE_SIP_PROVIDER, entry))
        as *mut WINE_SIP_PROVIDER;
    next = ((*prov).entry.next as *mut u8).sub(core::mem::offset_of!(WINE_SIP_PROVIDER, entry))
        as *mut WINE_SIP_PROVIDER;
    while core::ptr::addr_of_mut!((*prov).entry) != core::ptr::addr_of_mut!(providers) {
        list_remove(core::ptr::addr_of_mut!((*prov).entry));
        FreeLibrary((*prov).info.hSIP as HMODULE);
        CryptMemFree(prov as LPVOID);
        prov = next;
        next = ((*prov).entry.next as *mut u8).sub(core::mem::offset_of!(WINE_SIP_PROVIDER, entry))
            as *mut WINE_SIP_PROVIDER;
    }
    DeleteCriticalSection(core::ptr::addr_of_mut!(providers_cs));
}

unsafe extern "C" fn CRYPT_LoadSIP(pgSubject: *const GUID) -> BOOL {
    let mut sip: SIP_DISPATCH_INFO = core::mem::zeroed();
    let mut lib: HMODULE = core::ptr::null_mut();
    let mut temp: HMODULE = core::ptr::null_mut();
    'error: {
        sip.pfGet = core::mem::transmute::<*mut core::ffi::c_void, Option<pCryptSIPGetSignedDataMsg>>(
            CRYPT_LoadSIPFunc(pgSubject, RUSTCONV_WSTR_6FB3BB38F1CD.as_ptr(), core::ptr::addr_of_mut!(lib)),
        );
        if sip.pfGet.is_none() {
            break 'error;
        }
        sip.pfPut = core::mem::transmute::<*mut core::ffi::c_void, Option<pCryptSIPPutSignedDataMsg>>(
            CRYPT_LoadSIPFunc(pgSubject, RUSTCONV_WSTR_1FF4217A2E88.as_ptr(), core::ptr::addr_of_mut!(temp)),
        );
        if sip.pfPut.is_none() || temp != lib {
            break 'error;
        }
        FreeLibrary(temp);
        temp = core::ptr::null_mut();
        sip.pfCreate = core::mem::transmute::<*mut core::ffi::c_void, Option<pCryptSIPCreateIndirectData>>(
            CRYPT_LoadSIPFunc(pgSubject, RUSTCONV_WSTR_71A628F66082.as_ptr(), core::ptr::addr_of_mut!(temp)),
        );
        if sip.pfCreate.is_none() || temp != lib {
            break 'error;
        }
        FreeLibrary(temp);
        temp = core::ptr::null_mut();
        sip.pfVerify = core::mem::transmute::<*mut core::ffi::c_void, Option<pCryptSIPVerifyIndirectData>>(
            CRYPT_LoadSIPFunc(pgSubject, RUSTCONV_WSTR_5FCBA2C2CE88.as_ptr(), core::ptr::addr_of_mut!(temp)),
        );
        if sip.pfVerify.is_none() || temp != lib {
            break 'error;
        }
        FreeLibrary(temp);
        temp = core::ptr::null_mut();
        sip.pfRemove = core::mem::transmute::<*mut core::ffi::c_void, Option<pCryptSIPRemoveSignedDataMsg>>(
            CRYPT_LoadSIPFunc(pgSubject, RUSTCONV_WSTR_8759990C0E16.as_ptr(), core::ptr::addr_of_mut!(temp)),
        );
        if sip.pfRemove.is_none() || temp != lib {
            break 'error;
        }
        FreeLibrary(temp);
        sip.hSIP = lib as HANDLE;
        CRYPT_CacheSIP(pgSubject, core::ptr::addr_of_mut!(sip));
        return (TRUE) as BOOL;
    }
    FreeLibrary(lib);
    FreeLibrary(temp);
    SetLastError((TRUST_E_SUBJECT_FORM_UNKNOWN) as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSIPLoad(
    pgSubject: *const GUID,
    dwFlags: DWORD,
    pSipDispatch: *mut SIP_DISPATCH_INFO,
) -> BOOL {
    if pgSubject.is_null() || dwFlags != 0 || pSipDispatch.is_null()
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    if CRYPT_IsSIPCached(pgSubject) == 0 && CRYPT_LoadSIP(pgSubject) == 0 {
        return (FALSE) as BOOL;
    }

    (*pSipDispatch).hSIP = core::ptr::null_mut();
    (*pSipDispatch).pfGet = Some(CryptSIPGetSignedDataMsg);
    (*pSipDispatch).pfPut = Some(CryptSIPPutSignedDataMsg);
    (*pSipDispatch).pfCreate = Some(CryptSIPCreateIndirectData);
    (*pSipDispatch).pfVerify = Some(CryptSIPVerifyIndirectData);
    (*pSipDispatch).pfRemove = Some(CryptSIPRemoveSignedDataMsg);

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSIPCreateIndirectData(
    pSubjectInfo: *mut SIP_SUBJECTINFO,
    pcbIndirectData: *mut DWORD,
    pIndirectData: *mut SIP_INDIRECT_DATA,
) -> BOOL {
    let mut sip: *mut WINE_SIP_PROVIDER = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    if pSubjectInfo.is_null() || (*pSubjectInfo).pgSubjectType.is_null() || pcbIndirectData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }
    sip = CRYPT_GetCachedSIP((*pSubjectInfo).pgSubjectType);
    if !sip.is_null() {
        ret = (*sip).info.pfCreate.unwrap_unchecked()(
            pSubjectInfo,
            pcbIndirectData,
            pIndirectData,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSIPGetSignedDataMsg(
    pSubjectInfo: *mut SIP_SUBJECTINFO,
    pdwEncodingType: *mut DWORD,
    dwIndex: DWORD,
    pcbSignedDataMsg: *mut DWORD,
    pbSignedDataMsg: *mut BYTE,
) -> BOOL {
    let mut sip: *mut WINE_SIP_PROVIDER = core::ptr::null_mut::<WINE_SIP_PROVIDER>();
    let mut ret: BOOL = (FALSE as i32);
    WINE_TRACE!("(%p %p %ld %p %p)\n", pSubjectInfo, pdwEncodingType, dwIndex, pcbSignedDataMsg, pbSignedDataMsg);
    sip = CRYPT_GetCachedSIP((*pSubjectInfo).pgSubjectType);
    if !sip.is_null() {
        ret = (*sip).info.pfGet.unwrap_unchecked()(
            pSubjectInfo,
            pdwEncodingType,
            dwIndex,
            pcbSignedDataMsg,
            pbSignedDataMsg,
        );
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSIPPutSignedDataMsg(
    pSubjectInfo: *mut SIP_SUBJECTINFO,
    pdwEncodingType: DWORD,
    pdwIndex: *mut DWORD,
    cbSignedDataMsg: DWORD,
    pbSignedDataMsg: *mut BYTE,
) -> BOOL {
    let mut sip: *mut WINE_SIP_PROVIDER = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);
    WINE_TRACE!("(%p %ld %p %ld %p)\n", pSubjectInfo, pdwEncodingType, pdwIndex, cbSignedDataMsg, pbSignedDataMsg);
    sip = CRYPT_GetCachedSIP((*pSubjectInfo).pgSubjectType);
    if !sip.is_null() {
        ret = (*sip).info.pfPut.unwrap_unchecked()(
            pSubjectInfo,
            pdwEncodingType,
            pdwIndex,
            cbSignedDataMsg,
            pbSignedDataMsg,
        );
    }
    WINE_TRACE!("returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSIPRemoveSignedDataMsg(
    pSubjectInfo: *mut SIP_SUBJECTINFO,
    dwIndex: DWORD,
) -> BOOL {
    let mut sip: *mut WINE_SIP_PROVIDER = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    sip = CRYPT_GetCachedSIP((*pSubjectInfo).pgSubjectType);
    if !sip.is_null() {
        ret = (*sip).info.pfRemove.unwrap_unchecked()(pSubjectInfo, dwIndex);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSIPVerifyIndirectData(
    pSubjectInfo: *mut SIP_SUBJECTINFO,
    pIndirectData: *mut SIP_INDIRECT_DATA,
) -> BOOL {
    let mut sip: *mut WINE_SIP_PROVIDER = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    sip = CRYPT_GetCachedSIP((*pSubjectInfo).pgSubjectType);
    if !sip.is_null() {
        ret = (*sip).info.pfVerify.unwrap_unchecked()(pSubjectInfo, pIndirectData);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CryptSIPRetrieveSubjectGuidForCatalogFile(filename: LPCWSTR, handle: HANDLE, subject: *mut GUID) -> BOOL {
    SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    return (FALSE) as BOOL;
}
