use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::reg_reg::RegCloseKey;
use crate::reg_reg::RegDeleteValueW;
use crate::reg_reg::RegEnumKeyExW;
use crate::reg_reg::RegEnumKeyW;
use crate::reg_reg::RegOpenKeyExW;
use crate::reg_reg::RegOpenKeyW;
use crate::reg_reg::RegQueryInfoKeyW;
use crate::reg_reg::RegQueryValueExW;
use crate::reg_reg::RegSetValueExW;
use core::ffi::c_void;

extern "C" {
    fn memcmp(s1: LPCVOID, s2: LPCVOID, n: usize) -> i32;
    fn open(path: *const i8, flags: i32, ...) -> i32;
    fn close(fd: i32) -> i32;
    fn read(fd: i32, buf: PVOID, count: usize) -> isize;
}
extern "system" {
    fn SetLastError(dwErrCode: u32);
    fn IsBadWritePtr(lp: PVOID, ucb: usize) -> i32;
}

// wine/unicode.h strcpyW/strlenW are inline (wcscpy/wcslen) — no exported symbol;
// match the inline (same as wine/cred.rs).
#[inline]
unsafe fn strcpyW(dst: PWSTR, src: PCWSTR) -> PWSTR { wcscpy(dst, src) }
#[inline]
unsafe fn strlenW(s: PCWSTR) -> usize { wcslen(s) }

// Reg* error-code returns used as DWORD here; DWORD-returning local decls shadow
// the prelude's LONG ones (same advapi32 reg exports — ABI-identical).
extern "system" {
}

// wine/crypt.c: #define CRYPT_Alloc(size) (LocalAlloc(LMEM_ZEROINIT, size)).
// Private helper (matches the macro) — not an exported symbol.
#[inline]
unsafe fn CRYPT_Alloc(size: usize) -> *mut core::ffi::c_void {
    LocalAlloc(LMEM_ZEROINIT, size as SIZE_T) as *mut core::ffi::c_void
}

// wine/crypt.c: #define CRYPT_Free(buffer) (LocalFree(buffer)). Private static
// helper (matches the macro) — callers pass `ptr as _` which infers PVOID.
#[inline]
unsafe fn CRYPT_Free(buffer: PVOID) -> HLOCAL {
    LocalFree(buffer as HLOCAL)
}

// ---- file-scope preamble (translated) ----
static mut crypt_hWindow: HWND = core::ptr::null_mut();

unsafe fn CRYPT_GetProvKeyName(pProvName: PCWSTR) -> PWSTR {
    static KEYSTR: [u16; 51] = [
        b'S' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'w' as u16, b'a' as u16, b'r' as u16, b'e' as u16, b'\\' as u16,
        b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16,
        b'C' as u16, b'r' as u16, b'y' as u16, b'p' as u16, b't' as u16, b'o' as u16, b'g' as u16, b'r' as u16, b'a' as u16, b'p' as u16, b'h' as u16, b'y' as u16, b'\\' as u16,
        b'D' as u16, b'e' as u16, b'f' as u16, b'a' as u16, b'u' as u16, b'l' as u16, b't' as u16, b's' as u16, b'\\' as u16,
        b'P' as u16, b'r' as u16, b'o' as u16, b'v' as u16, b'i' as u16, b'd' as u16, b'e' as u16, b'r' as u16, b'\\' as u16, 0,
    ];

    let keyname: PWSTR = CRYPT_Alloc(
        (strlenW(KEYSTR.as_ptr()) + strlenW(pProvName) + 1) * core::mem::size_of::<u16>(),
    ) as PWSTR;

    if !keyname.is_null() {
        strcpyW(keyname, KEYSTR.as_ptr());
        strcpyW(keyname.add(strlenW(KEYSTR.as_ptr())), pProvName);
    } else {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
    }
    keyname
}

unsafe fn CRYPT_GetTypeKeyName(dwType: u32, user: i32) -> PWSTR {
    static MACHINESTR: [u16; 65] = [
        b'S' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'w' as u16, b'a' as u16, b'r' as u16, b'e' as u16, b'\\' as u16,
        b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16,
        b'C' as u16, b'r' as u16, b'y' as u16, b'p' as u16, b't' as u16, b'o' as u16, b'g' as u16, b'r' as u16, b'a' as u16, b'p' as u16, b'h' as u16, b'y' as u16, b'\\' as u16,
        b'D' as u16, b'e' as u16, b'f' as u16, b'a' as u16, b'u' as u16, b'l' as u16, b't' as u16, b's' as u16, b'\\' as u16,
        b'P' as u16, b'r' as u16, b'o' as u16, b'v' as u16, b'i' as u16, b'd' as u16, b'e' as u16, b'r' as u16, b' ' as u16, b'T' as u16, b'y' as u16, b'p' as u16, b'e' as u16, b's' as u16, b'\\' as u16,
        b'T' as u16, b'y' as u16, b'p' as u16, b'e' as u16, b' ' as u16, b'X' as u16, b'X' as u16, b'X' as u16, 0,
    ];
    static USERSTR: [u16; 50] = [
        b'S' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'w' as u16, b'a' as u16, b'r' as u16, b'e' as u16, b'\\' as u16,
        b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16,
        b'C' as u16, b'r' as u16, b'y' as u16, b'p' as u16, b't' as u16, b'o' as u16, b'g' as u16, b'r' as u16, b'a' as u16, b'p' as u16, b'h' as u16, b'y' as u16, b'\\' as u16,
        b'P' as u16, b'r' as u16, b'o' as u16, b'v' as u16, b'i' as u16, b'd' as u16, b'e' as u16, b'r' as u16, b' ' as u16, b'T' as u16, b'y' as u16, b'p' as u16, b'e' as u16, b' ' as u16, b'X' as u16, b'X' as u16, b'X' as u16, 0,
    ];

    let keyname: PWSTR = CRYPT_Alloc(
        ((if user != 0 { strlenW(USERSTR.as_ptr()) } else { strlenW(MACHINESTR.as_ptr()) }) + 1)
            * core::mem::size_of::<u16>() as usize,
    ) as PWSTR;
    if !keyname.is_null() {
        if user != 0 {
            strcpyW(keyname, USERSTR.as_ptr());
        } else {
            strcpyW(keyname, MACHINESTR.as_ptr());
        }
        let mut ptr = keyname.add(strlenW(keyname) as usize);
        ptr = ptr.offset(-1);
        *ptr = ((dwType % 10) as u16) + b'0' as u16;
        ptr = ptr.offset(-1);
        *ptr = (((dwType / 10) % 10) as u16) + b'0' as u16;
        ptr = ptr.offset(-1);
        *ptr = ((dwType / 100) as u16) + b'0' as u16;
    } else {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
    }
    keyname
}

unsafe fn CRYPT_UnicodeToANSI(wstr: LPCWSTR, str: *mut LPSTR, mut strsize: i32) -> BOOL {
    if wstr.is_null() {
        *str = core::ptr::null_mut();
        return 1;
    }

    if (*str).is_null() {
        strsize = WideCharToMultiByte(CP_ACP, 0, wstr, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
        *str = CRYPT_Alloc((strsize as usize * core::mem::size_of::<CHAR>()) as _) as LPSTR;
    } else if strsize < 0 {
        strsize = i32::MAX;
    }

    if !(*str).is_null() {
        WideCharToMultiByte(CP_ACP, 0, wstr, -1, *str, strsize, core::ptr::null_mut(), core::ptr::null_mut());
        return 1;
    }

    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
    0
}

unsafe fn CRYPT_ANSIToUnicode(str: LPCSTR, wstr: *mut LPWSTR, wstrsize: i32) -> BOOL {
    let mut wcount: u32;

    if str.is_null() {
        *wstr = core::ptr::null_mut();
        return TRUE;
    }
    wcount = MultiByteToWideChar(CP_ACP, 0, str, -1, core::ptr::null_mut(), 0) as u32;
    if wstrsize == -1 {
        *wstr = CRYPT_Alloc((wcount as usize * core::mem::size_of::<WCHAR>()) as _) as LPWSTR;
    } else {
        let limit = (wstrsize as u32) / core::mem::size_of::<WCHAR>() as u32;
        wcount = core::cmp::min(wcount, limit);
    }
    if !(*wstr).is_null() {
        MultiByteToWideChar(CP_ACP, 0, str, -1, *wstr, wcount as i32);
        return TRUE;
    }
    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
    FALSE
}
unsafe extern "system" fn CRYPT_VerifyImage(lpszImage: LPCSTR, pData: *mut u8) -> i32 {
    if lpszImage.is_null() || pData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    FIXME!("(%s, %p): not verifying image\n", lpszImage, pData);

    1
}
unsafe extern "system" fn CRYPT_ReturnhWnd(phWnd: *mut HWND) {
    if !phWnd.is_null() {
        *phWnd = crypt_hWindow;
    }
}

unsafe fn CRYPT_LoadProvider(pImage: PCWSTR) -> PCRYPTPROV {
    let mut provider: PCRYPTPROV = core::ptr::null_mut();
    let mut errorcode: DWORD = ERROR_NOT_ENOUGH_MEMORY;

    'error: {
        provider = CRYPT_Alloc(core::mem::size_of::<CRYPTPROV>() as _) as PCRYPTPROV;
        if provider.is_null() {
            break 'error;
        }
        (*provider).pFuncs = CRYPT_Alloc(core::mem::size_of::<PROVFUNCS>() as _) as *mut PROVFUNCS;
        if (*provider).pFuncs.is_null() {
            break 'error;
        }
        (*provider).pVTable =
            CRYPT_Alloc(core::mem::size_of::<VTableProvStruc>() as _) as *mut VTableProvStruc;
        if (*provider).pVTable.is_null() {
            break 'error;
        }
        (*provider).hModule = LoadLibraryW(pImage);
        if (*provider).hModule.is_null() {
            errorcode = if GetLastError() == ERROR_FILE_NOT_FOUND {
                NTE_PROV_DLL_NOT_FOUND
            } else {
                NTE_PROVIDER_DLL_FAIL
            };
            FIXME!("Failed to load dll %S\n", pImage);
            break 'error;
        }
        (*provider).dwMagic = MAGIC_CRYPTPROV;
        (*provider).refcount = 1;

        errorcode = NTE_PROVIDER_DLL_FAIL;

        macro_rules! get_func {
            ($name:ident, $sym:expr) => {{
                let f = GetProcAddress((*provider).hModule, $sym.as_ptr() as *const i8);
                if f.is_null() {
                    break 'error;
                }
                (*(*provider).pFuncs).$name = core::mem::transmute(f);
            }};
        }
        macro_rules! get_func_opt {
            ($name:ident, $sym:expr) => {{
                let f = GetProcAddress((*provider).hModule, $sym.as_ptr() as *const i8);
                (*(*provider).pFuncs).$name = core::mem::transmute(f);
            }};
        }

        get_func!(pCPAcquireContext, b"CPAcquireContext\0");
        get_func!(pCPCreateHash, b"CPCreateHash\0");
        get_func!(pCPDecrypt, b"CPDecrypt\0");
        get_func!(pCPDeriveKey, b"CPDeriveKey\0");
        get_func!(pCPDestroyHash, b"CPDestroyHash\0");
        get_func!(pCPDestroyKey, b"CPDestroyKey\0");
        get_func_opt!(pCPDuplicateHash, b"CPDuplicateHash\0");
        get_func_opt!(pCPDuplicateKey, b"CPDuplicateKey\0");
        get_func!(pCPEncrypt, b"CPEncrypt\0");
        get_func!(pCPExportKey, b"CPExportKey\0");
        get_func!(pCPGenKey, b"CPGenKey\0");
        get_func!(pCPGenRandom, b"CPGenRandom\0");
        get_func!(pCPGetHashParam, b"CPGetHashParam\0");
        get_func!(pCPGetKeyParam, b"CPGetKeyParam\0");
        get_func!(pCPGetProvParam, b"CPGetProvParam\0");
        get_func!(pCPGetUserKey, b"CPGetUserKey\0");
        get_func!(pCPHashData, b"CPHashData\0");
        get_func!(pCPHashSessionKey, b"CPHashSessionKey\0");
        get_func!(pCPImportKey, b"CPImportKey\0");
        get_func!(pCPReleaseContext, b"CPReleaseContext\0");
        get_func!(pCPSetHashParam, b"CPSetHashParam\0");
        get_func!(pCPSetKeyParam, b"CPSetKeyParam\0");
        get_func!(pCPSetProvParam, b"CPSetProvParam\0");
        get_func!(pCPSignHash, b"CPSignHash\0");
        get_func!(pCPVerifySignature, b"CPVerifySignature\0");

        (*(*provider).pVTable).Version = 3;
        (*(*provider).pVTable).FuncVerifyImage =
            Some(CRYPT_VerifyImage);
        (*(*provider).pVTable).FuncReturnhWnd =
            Some(CRYPT_ReturnhWnd);
        (*(*provider).pVTable).dwProvType = 0;
        (*(*provider).pVTable).pbContextInfo = core::ptr::null_mut();
        (*(*provider).pVTable).cbContextInfo = 0;
        (*(*provider).pVTable).pszProvName = core::ptr::null_mut();
        return provider;
    }

    SetLastError(errorcode);
    if !provider.is_null() {
        (*provider).dwMagic = 0;
        if !(*provider).hModule.is_null() {
            FreeLibrary((*provider).hModule);
        }
        CRYPT_Free((*provider).pVTable as _);
        CRYPT_Free((*provider).pFuncs as _);
        CRYPT_Free(provider as _);
    }
    core::ptr::null_mut()
}

unsafe fn CRYPT_CreateMachineGuid() {
    static cryptographyW: [u16; 32] = [
        b'S' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'w' as u16, b'a' as u16, b'r' as u16, b'e' as u16, b'\\' as u16,
        b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16,
        b'C' as u16, b'r' as u16, b'y' as u16, b'p' as u16, b't' as u16, b'o' as u16, b'g' as u16, b'r' as u16, b'a' as u16, b'p' as u16, b'h' as u16, b'y' as u16, 0,
    ];
    static machineGuidW: [u16; 12] = [
        b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b'G' as u16, b'u' as u16, b'i' as u16, b'd' as u16, 0,
    ];

    let mut key: HKEY = core::ptr::null_mut();

    let r = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        cryptographyW.as_ptr(),
        0,
        KEY_ALL_ACCESS,
        &mut key,
    );
    if r == 0 {
        let mut size: u32 = 0;

        let r = RegQueryValueExW(
            key,
            machineGuidW.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut size,
        );
        if (r) as u32 == ERROR_FILE_NOT_FOUND {
            let mut uuid: UUID = core::mem::zeroed();
            let mut buf: [u16; 37] = [0; 37];
            // L"%08x-%04x-%04x-%02x%02x-%02x%02x%02x%02x%02x%02x" (48 wide chars + NUL).
            // crypt.c:300 writes 49 elements; this had 39. It was not merely missing the
            // terminator: it ended mid-conversion at a dangling `%02`, so the format supplied
            // NINE complete conversions for the ELEVEN arguments sprintfW is called with below,
            // and then ran off the end of the array looking for a NUL. A UUID needs 8-4-4-4-12
            // hex digits, which is exactly the eleven `%0Nx` here.
            static uuidFmt: [u16; 49] = [
                b'%' as u16, b'0' as u16, b'8' as u16, b'x' as u16, b'-' as u16, b'%' as u16, b'0' as u16, b'4' as u16, b'x' as u16, b'-' as u16,
                b'%' as u16, b'0' as u16, b'4' as u16, b'x' as u16, b'-' as u16, b'%' as u16, b'0' as u16, b'2' as u16, b'x' as u16,
                b'%' as u16, b'0' as u16, b'2' as u16, b'x' as u16, b'-' as u16, b'%' as u16, b'0' as u16, b'2' as u16, b'x' as u16,
                b'%' as u16, b'0' as u16, b'2' as u16, b'x' as u16,
                b'%' as u16, b'0' as u16, b'2' as u16, b'x' as u16, b'%' as u16, b'0' as u16, b'2' as u16, b'x' as u16,
                b'%' as u16, b'0' as u16, b'2' as u16, b'x' as u16, b'%' as u16, b'0' as u16, b'2' as u16, b'x' as u16,
                0,
            ];

            let rs = UuidCreate(&mut uuid);
            if rs == STATUS_SUCCESS {
                sprintfW(
                    buf.as_mut_ptr(),
                    uuidFmt.as_ptr(),
                    uuid.Data1,
                    uuid.Data2 as u32,
                    uuid.Data3 as u32,
                    uuid.Data4[0] as u32,
                    uuid.Data4[1] as u32,
                    uuid.Data4[2] as u32,
                    uuid.Data4[3] as u32,
                    uuid.Data4[4] as u32,
                    uuid.Data4[5] as u32,
                    uuid.Data4[6] as u32,
                    uuid.Data4[7] as u32,
                );
                RegSetValueExW(
                    key,
                    machineGuidW.as_ptr(),
                    0,
                    REG_SZ,
                    buf.as_ptr() as LPBYTE,
                    ((lstrlenW(buf.as_ptr()) + 1) as u32) * core::mem::size_of::<u16>() as u32,
                );
            }
        }
        RegCloseKey(key);
    }
}

/// The C's `pointer_from_handle`: a handle is good when it points at the right magic, and
/// reading it is wrapped in `__TRY/__EXCEPT_PAGE_FAULT`, because callers hand in garbage.
/// crypt32's root import passes the uninitialized `HCRYPTPROV` that a failed
/// `CryptAcquireContextW` left behind (no CSP registered yet), and on amd64 the bare read
/// killed the process instead of failing ERROR_INVALID_PARAMETER.
unsafe fn handle_has_magic(handle: usize, magic: DWORD) -> bool {
    if handle == 0 {
        return false;
    }
    let mut found: DWORD = 0;
    ntdll_sys::seh::try_except(
        &mut found,
        |f| *f = core::ptr::read_volatile(handle as *const DWORD),
        |_, ptrs| {
            if ntdll_sys::seh::exception_code(ptrs) as u32 == 0xC000_0005 {
                ntdll_sys::seh::EXCEPTION_EXECUTE_HANDLER
            } else {
                ntdll_sys::seh::EXCEPTION_CONTINUE_SEARCH
            }
        },
    )
    .is_ok()
        && found == magic
}

#[no_mangle]
pub unsafe extern "system" fn CloseEncryptedFileRaw(context: *mut core::ffi::c_void) {
    FIXME!("(%p): stub\n", context);
}

#[no_mangle]
pub unsafe extern "system" fn CryptAcquireContextW(
    phProv: *mut HCRYPTPROV,
    pszContainer: LPCWSTR,
    pszProvider: LPCWSTR,
    dwProvType: DWORD,
    dwFlags: DWORD,
) -> BOOL {
    let mut pProv: PCRYPTPROV = core::ptr::null_mut();
    let mut key: HKEY = core::ptr::null_mut();
    let mut imagepath: PWSTR = core::ptr::null_mut();
    let mut keyname: PWSTR;
    let mut provname: PWSTR = core::ptr::null_mut();
    let mut temp: PWSTR = core::ptr::null_mut();
    let mut provnameA: PSTR = core::ptr::null_mut();
    let mut pszContainerA: PSTR = core::ptr::null_mut();
    let mut keytype: DWORD = 0;
    let mut type_: DWORD = 0;
    let mut len: DWORD = 0;
    let mut r: ULONG;

    static nameW: [WCHAR; 5] = [b'N' as WCHAR, b'a' as WCHAR, b'm' as WCHAR, b'e' as WCHAR, 0];
    static typeW: [WCHAR; 5] = [b'T' as WCHAR, b'y' as WCHAR, b'p' as WCHAR, b'e' as WCHAR, 0];
    static imagepathW: [WCHAR; 11] = [
        b'I' as WCHAR, b'm' as WCHAR, b'a' as WCHAR, b'g' as WCHAR, b'e' as WCHAR,
        b' ' as WCHAR, b'P' as WCHAR, b'a' as WCHAR, b't' as WCHAR, b'h' as WCHAR, 0,
    ];

    if dwProvType < 1 || dwProvType > MAXPROVTYPES {
        SetLastError(NTE_BAD_PROV_TYPE as DWORD);
        return FALSE;
    }

    if phProv.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    CRYPT_CreateMachineGuid();

    'error: {
        if pszProvider.is_null() || *pszProvider == 0 {
            keyname = CRYPT_GetTypeKeyName(dwProvType, TRUE as i32);
            if keyname.is_null() {
                SetLastError(NTE_PROV_TYPE_NOT_DEF as DWORD);
                return FALSE;
            }
            if RegOpenKeyW(HKEY_CURRENT_USER, keyname, &mut key) != 0 {
                CRYPT_Free(keyname as _);
                keyname = CRYPT_GetTypeKeyName(dwProvType, FALSE as i32);
                if keyname.is_null() {
                    RegCloseKey(key);
                    SetLastError(NTE_PROV_TYPE_NOT_DEF as DWORD);
                    break 'error;
                }
                if RegOpenKeyW(HKEY_LOCAL_MACHINE, keyname, &mut key) != 0 {
                    CRYPT_Free(keyname as _);
                    RegCloseKey(key);
                    SetLastError(NTE_PROV_TYPE_NOT_DEF as DWORD);
                    break 'error;
                }
            }
            CRYPT_Free(keyname as _);
            r = (RegQueryValueExW(key, nameW.as_ptr(), core::ptr::null_mut(), &mut keytype, core::ptr::null_mut(), &mut len)) as u32;
            if r != ERROR_SUCCESS || len == 0 || keytype != REG_SZ {
                RegCloseKey(key);
                SetLastError(NTE_PROV_TYPE_ENTRY_BAD as DWORD);
                break 'error;
            }
            provname = CRYPT_Alloc(len as usize) as PWSTR;
            if provname.is_null() {
                RegCloseKey(key);
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'error;
            }
            r = (RegQueryValueExW(key, nameW.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), provname as LPBYTE, &mut len)) as u32;
            if r != ERROR_SUCCESS {
                RegCloseKey(key);
                SetLastError(NTE_PROV_TYPE_ENTRY_BAD as DWORD);
                break 'error;
            }
            RegCloseKey(key);
        } else {
            provname = CRYPT_Alloc(((strlenW(pszProvider) + 1) as usize) * core::mem::size_of::<WCHAR>()) as PWSTR;
            if provname.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'error;
            }
            strcpyW(provname, pszProvider);
        }

        keyname = CRYPT_GetProvKeyName(provname);
        r = (RegOpenKeyW(HKEY_LOCAL_MACHINE, keyname, &mut key)) as u32;
        CRYPT_Free(keyname as _);
        if r != ERROR_SUCCESS {
            SetLastError(NTE_KEYSET_NOT_DEF as DWORD);
            break 'error;
        }
        len = core::mem::size_of::<DWORD>() as DWORD;
        r = (RegQueryValueExW(key, typeW.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), &mut type_ as *mut DWORD as LPBYTE, &mut len)) as u32;
        if r != ERROR_SUCCESS {
            SetLastError(NTE_PROV_TYPE_ENTRY_BAD as DWORD);
            break 'error;
        }
        if type_ != dwProvType {
            SetLastError(NTE_PROV_TYPE_NO_MATCH as DWORD);
            break 'error;
        }

        r = (RegQueryValueExW(key, imagepathW.as_ptr(), core::ptr::null_mut(), &mut keytype, core::ptr::null_mut(), &mut len)) as u32;
        if r != ERROR_SUCCESS || keytype != REG_SZ {
            RegCloseKey(key);
            SetLastError(NTE_PROV_TYPE_ENTRY_BAD as DWORD);
            break 'error;
        }
        temp = CRYPT_Alloc(len as usize) as PWSTR;
        if temp.is_null() {
            RegCloseKey(key);
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'error;
        }
        r = (RegQueryValueExW(key, imagepathW.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), temp as LPBYTE, &mut len)) as u32;
        if r != ERROR_SUCCESS {
            RegCloseKey(key);
            SetLastError(NTE_PROV_TYPE_ENTRY_BAD as DWORD);
            break 'error;
        }
        RegCloseKey(key);
        len = ExpandEnvironmentStringsW(temp, core::ptr::null_mut(), 0);
        imagepath = CRYPT_Alloc((len as usize) * core::mem::size_of::<WCHAR>()) as PWSTR;
        if imagepath.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'error;
        }
        if ExpandEnvironmentStringsW(temp, imagepath, len) == 0 {
            break 'error;
        }
        pProv = CRYPT_LoadProvider(imagepath);
        if pProv.is_null() {
            break 'error;
        }
        (*(*pProv).pVTable).dwProvType = dwProvType;
        if CRYPT_UnicodeToANSI(provname, &mut provnameA, 0) == 0 {
            break 'error;
        }
        (*(*pProv).pVTable).pszProvName = provnameA;
        if CRYPT_UnicodeToANSI(pszContainer, &mut pszContainerA, 0) == 0 {
            break 'error;
        }
        if ((*(*pProv).pFuncs).pCPAcquireContext.unwrap())(
            &mut (*pProv).hPrivate,
            pszContainerA,
            dwFlags,
            (*pProv).pVTable,
        ) != 0
        {
            if dwFlags & CRYPT_DELETEKEYSET != 0 {
                (*pProv).dwMagic = 0;
                FreeLibrary((*pProv).hModule);
                CRYPT_Free(provnameA as _);
                CRYPT_Free((*pProv).pVTable as _);
                CRYPT_Free((*pProv).pFuncs as _);
                CRYPT_Free(pProv as _);
            } else {
                *phProv = pProv as HCRYPTPROV;
            }
            CRYPT_Free(pszContainerA as _);
            CRYPT_Free(provname as _);
            CRYPT_Free(temp as _);
            CRYPT_Free(imagepath as _);
            return TRUE;
        }
    }

    if !pProv.is_null() {
        (*pProv).dwMagic = 0;
        if !(*pProv).hModule.is_null() {
            FreeLibrary((*pProv).hModule);
        }
        CRYPT_Free((*pProv).pVTable as _);
        CRYPT_Free((*pProv).pFuncs as _);
        CRYPT_Free(pProv as _);
    }
    CRYPT_Free(pszContainerA as _);
    CRYPT_Free(provnameA as _);
    CRYPT_Free(provname as _);
    CRYPT_Free(temp as _);
    CRYPT_Free(imagepath as _);
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn CryptAcquireContextA(
    phProv: *mut HCRYPTPROV,
    pszContainer: LPCSTR,
    pszProvider: LPCSTR,
    dwProvType: DWORD,
    dwFlags: DWORD,
) -> BOOL {
    let mut pProvider: PWSTR = core::ptr::null_mut();
    let mut pContainer: PWSTR = core::ptr::null_mut();
    let ret: BOOL;

    TRACE!(
        b"(%p, %s, %s, %d, %08x)\n\0".as_ptr() as LPCSTR,
        phProv,
        debugstr_a(pszContainer),
        debugstr_a(pszProvider),
        dwProvType,
        dwFlags,
    );

    if CRYPT_ANSIToUnicode(pszContainer, &mut pContainer, -1) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    if CRYPT_ANSIToUnicode(pszProvider, &mut pProvider, -1) == 0 {
        CRYPT_Free(pContainer as LPVOID);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    ret = CryptAcquireContextW(phProv, pContainer, pProvider, dwProvType, dwFlags);

    CRYPT_Free(pContainer as LPVOID);
    CRYPT_Free(pProvider as LPVOID);

    ret
}

#[no_mangle]
pub unsafe extern "system" fn CryptContextAddRef(
    hProv: HCRYPTPROV,
    pdwReserved: *mut DWORD,
    dwFlags: DWORD,
) -> BOOL {
    let pProv: PCRYPTPROV = hProv as PCRYPTPROV;

    if pProv.is_null() {
        SetLastError(NTE_BAD_UID);
        return 0;
    }

    if !handle_has_magic(pProv as usize, MAGIC_CRYPTPROV) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    InterlockedIncrement(core::ptr::addr_of_mut!((*pProv).refcount));
    1
}

#[no_mangle]
pub unsafe extern "system" fn CryptReleaseContext(hProv: HCRYPTPROV, dwFlags: DWORD) -> BOOL {
    let pProv: PCRYPTPROV = hProv as PCRYPTPROV;
    let mut ret: BOOL = 1;

    if pProv.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if !handle_has_magic(pProv as usize, MAGIC_CRYPTPROV) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if InterlockedDecrement(core::ptr::addr_of_mut!((*pProv).refcount)) == 0 {
        ret = ((*(*pProv).pFuncs).pCPReleaseContext.unwrap())((*pProv).hPrivate, dwFlags);
        (*pProv).dwMagic = 0;
        FreeLibrary((*pProv).hModule);
        CRYPT_Free((*(*pProv).pVTable).pszProvName as *mut core::ffi::c_void);
        CRYPT_Free((*pProv).pVTable as *mut core::ffi::c_void);
        CRYPT_Free((*pProv).pFuncs as *mut core::ffi::c_void);
        CRYPT_Free(pProv as *mut core::ffi::c_void);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn CryptGenRandom(hProv: HCRYPTPROV, dwLen: DWORD, pbBuffer: *mut BYTE) -> BOOL {
    let prov = hProv as PCRYPTPROV;

    if hProv == 0 {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if !handle_has_magic(prov as usize, MAGIC_CRYPTPROV) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    ((*(*prov).pFuncs).pCPGenRandom.unwrap())((*prov).hPrivate, dwLen, pbBuffer)
}

#[no_mangle]
pub unsafe extern "system" fn CryptCreateHash(
    hProv: HCRYPTPROV,
    Algid: ALG_ID,
    hKey: HCRYPTKEY,
    dwFlags: DWORD,
    phHash: *mut HCRYPTHASH,
) -> BOOL {
    let prov: PCRYPTPROV = hProv as PCRYPTPROV;
    let key: PCRYPTKEY = hKey as PCRYPTKEY;
    let hash: PCRYPTHASH;

    TRACE!(
        b"(0x%lx, 0x%x, 0x%lx, %08x, %p)\n\0".as_ptr() as *const i8,
        hProv,
        Algid,
        hKey,
        dwFlags,
        phHash,
    );

    if prov.is_null()
        || phHash.is_null()
        || !handle_has_magic(prov as usize, MAGIC_CRYPTPROV)
        || (!key.is_null() && !handle_has_magic(key as usize, MAGIC_CRYPTKEY))
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if dwFlags != 0 {
        SetLastError(NTE_BAD_FLAGS);
        return 0;
    }
    hash = CRYPT_Alloc(core::mem::size_of::<CRYPTHASH>()) as PCRYPTHASH;
    if hash.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    (*hash).pProvider = prov;
    (*hash).dwMagic = MAGIC_CRYPTHASH;
    if (*(*prov).pFuncs).pCPCreateHash.unwrap()(
        (*prov).hPrivate,
        Algid,
        if !key.is_null() { (*key).hPrivate } else { 0 },
        0,
        core::ptr::addr_of_mut!((*hash).hPrivate),
    ) != 0
    {
        *phHash = hash as HCRYPTHASH;
        return 1;
    }

    /* CSP error! */
    (*hash).dwMagic = 0;
    CRYPT_Free(hash as _);
    *phHash = 0;
    0
}

#[no_mangle]
pub unsafe extern "system" fn CryptDecrypt(
    hKey: HCRYPTKEY,
    hHash: HCRYPTHASH,
    Final: BOOL,
    dwFlags: DWORD,
    pbData: *mut BYTE,
    pdwDataLen: *mut DWORD,
) -> BOOL {
    let prov: PCRYPTPROV;
    let key: PCRYPTKEY = hKey as PCRYPTKEY;
    let hash: PCRYPTHASH = hHash as PCRYPTHASH;

    TRACE!(
        b"(0x%lx, 0x%lx, %d, %08x, %p, %p)\n\0".as_ptr() as *const i8,
        hKey,
        hHash,
        Final,
        dwFlags,
        pbData,
        pdwDataLen,
    );

    if key.is_null()
        || pbData.is_null()
        || pdwDataLen.is_null()
        || (*key).pProvider.is_null()
        || !handle_has_magic(key as usize, MAGIC_CRYPTKEY)
        || !handle_has_magic((*key).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    prov = (*key).pProvider;
    ((*(*prov).pFuncs).pCPDecrypt.unwrap())(
        (*prov).hPrivate,
        (*key).hPrivate,
        if !hash.is_null() { (*hash).hPrivate } else { 0 },
        Final,
        dwFlags,
        pbData,
        pdwDataLen,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CryptDeriveKey(
    hProv: HCRYPTPROV,
    Algid: ALG_ID,
    hBaseData: HCRYPTHASH,
    dwFlags: DWORD,
    phKey: *mut HCRYPTKEY,
) -> BOOL {
    let prov = hProv as PCRYPTPROV;
    let hash = hBaseData as PCRYPTHASH;
    let key: PCRYPTKEY;

    if prov.is_null() || hash.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    if phKey.is_null()
        || !handle_has_magic(prov as usize, MAGIC_CRYPTPROV)
        || !handle_has_magic(hash as usize, MAGIC_CRYPTHASH)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    key = CRYPT_Alloc(core::mem::size_of::<CRYPTKEY>()) as PCRYPTKEY;
    if key.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    (*key).pProvider = prov;
    (*key).dwMagic = MAGIC_CRYPTKEY;
    if (*(*prov).pFuncs).pCPDeriveKey.unwrap()(
        (*prov).hPrivate,
        Algid,
        (*hash).hPrivate,
        dwFlags,
        core::ptr::addr_of_mut!((*key).hPrivate),
    ) != 0
    {
        *phKey = key as HCRYPTKEY;
        return 1;
    }

    (*key).dwMagic = 0;
    CRYPT_Free(key as *mut _);
    *phKey = 0;
    0
}

#[no_mangle]
pub unsafe extern "system" fn CryptDestroyHash(hHash: HCRYPTHASH) -> BOOL {
    let hash: PCRYPTHASH = hHash as PCRYPTHASH;
    let prov: PCRYPTPROV;
    let ret: BOOL;

    if hash.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    if (*hash).pProvider.is_null()
        || !handle_has_magic(hash as usize, MAGIC_CRYPTHASH)
        || !handle_has_magic((*hash).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    prov = (*hash).pProvider;
    ret = ((*(*prov).pFuncs).pCPDestroyHash.unwrap())((*prov).hPrivate, (*hash).hPrivate);
    (*hash).dwMagic = 0;
    CRYPT_Free(hash as *mut core::ffi::c_void);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn CryptDestroyKey(hKey: HCRYPTKEY) -> BOOL {
    let key: PCRYPTKEY = hKey as PCRYPTKEY;
    let prov: PCRYPTPROV;
    let ret: BOOL;

    if key.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    if (*key).pProvider.is_null()
        || !handle_has_magic(key as usize, MAGIC_CRYPTKEY)
        || !handle_has_magic((*key).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    prov = (*key).pProvider;
    ret = ((*(*prov).pFuncs).pCPDestroyKey.unwrap())((*prov).hPrivate, (*key).hPrivate);
    (*key).dwMagic = 0;
    CRYPT_Free(key as *mut _);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn CryptDuplicateHash(
    hHash: HCRYPTHASH,
    pdwReserved: *mut DWORD,
    dwFlags: DWORD,
    phHash: *mut HCRYPTHASH,
) -> BOOL {
    let prov: PCRYPTPROV;
    let orghash: PCRYPTHASH;
    let newhash: PCRYPTHASH;

    orghash = hHash as PCRYPTHASH;
    if orghash.is_null()
        || !pdwReserved.is_null()
        || phHash.is_null()
        || (*orghash).pProvider.is_null()
        || !handle_has_magic(orghash as usize, MAGIC_CRYPTHASH)
        || !handle_has_magic((*orghash).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    prov = (*orghash).pProvider;
    if (*(*prov).pFuncs).pCPDuplicateHash.is_none() {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        return FALSE;
    }

    newhash = CRYPT_Alloc(core::mem::size_of::<CRYPTHASH>() as _) as PCRYPTHASH;
    if newhash.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE;
    }

    (*newhash).pProvider = prov;
    (*newhash).dwMagic = MAGIC_CRYPTHASH;
    if (*(*prov).pFuncs).pCPDuplicateHash.unwrap()(
        (*prov).hPrivate,
        (*orghash).hPrivate,
        pdwReserved,
        dwFlags,
        core::ptr::addr_of_mut!((*newhash).hPrivate),
    ) != 0
    {
        *phHash = newhash as HCRYPTHASH;
        return TRUE;
    }
    (*newhash).dwMagic = 0;
    CRYPT_Free(newhash as _);
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn CryptDuplicateKey(
    hKey: HCRYPTKEY,
    pdwReserved: *mut DWORD,
    dwFlags: DWORD,
    phKey: *mut HCRYPTKEY,
) -> BOOL {
    let prov: PCRYPTPROV;
    let orgkey: PCRYPTKEY;
    let newkey: PCRYPTKEY;

    TRACE!(b"(0x%lx, %p, %08x, %p)\n\0".as_ptr() as *const i8, hKey, pdwReserved, dwFlags, phKey);

    orgkey = hKey as PCRYPTKEY;
    if orgkey.is_null()
        || !pdwReserved.is_null()
        || phKey.is_null()
        || (*orgkey).pProvider.is_null()
        || !handle_has_magic(orgkey as usize, MAGIC_CRYPTKEY)
        || !handle_has_magic((*orgkey).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    prov = (*orgkey).pProvider;
    if (*(*prov).pFuncs).pCPDuplicateKey.is_none() {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        return 0;
    }

    newkey = CRYPT_Alloc(core::mem::size_of::<CRYPTKEY>()) as PCRYPTKEY;
    if newkey.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    (*newkey).pProvider = prov;
    (*newkey).dwMagic = MAGIC_CRYPTKEY;
    if (*(*prov).pFuncs).pCPDuplicateKey.unwrap()(
        (*prov).hPrivate,
        (*orgkey).hPrivate,
        pdwReserved,
        dwFlags,
        core::ptr::addr_of_mut!((*newkey).hPrivate),
    ) != 0
    {
        *phKey = newkey as HCRYPTKEY;
        return 1;
    }
    (*newkey).dwMagic = 0;
    CRYPT_Free(newkey as *mut core::ffi::c_void);
    0
}

#[no_mangle]
pub unsafe extern "system" fn CryptEncrypt(
    hKey: HCRYPTKEY,
    hHash: HCRYPTHASH,
    Final: BOOL,
    dwFlags: DWORD,
    pbData: *mut BYTE,
    pdwDataLen: *mut DWORD,
    dwBufLen: DWORD,
) -> BOOL {
    let key: PCRYPTKEY = hKey as PCRYPTKEY;
    let hash: PCRYPTHASH = hHash as PCRYPTHASH;

    TRACE!(
        b"(0x%lx, 0x%lx, %d, %08x, %p, %p, %d)\n\0".as_ptr() as *const i8,
        hKey,
        hHash,
        Final,
        dwFlags,
        pbData,
        pdwDataLen,
        dwBufLen,
    );

    if key.is_null()
        || pdwDataLen.is_null()
        || (*key).pProvider.is_null()
        || !handle_has_magic(key as usize, MAGIC_CRYPTKEY)
        || !handle_has_magic((*key).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let prov: PCRYPTPROV = (*key).pProvider;
    ((*(*prov).pFuncs).pCPEncrypt.unwrap())(
        (*prov).hPrivate,
        (*key).hPrivate,
        if !hash.is_null() { (*hash).hPrivate } else { 0 },
        Final,
        dwFlags,
        pbData,
        pdwDataLen,
        dwBufLen,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CryptEnumProvidersW(
    dwIndex: DWORD,
    pdwReserved: *mut DWORD,
    dwFlags: DWORD,
    pdwProvType: *mut DWORD,
    pszProvName: LPWSTR,
    pcbProvName: *mut DWORD,
) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();

    static providerW: [WCHAR; 49] = [
        b'S' as WCHAR, b'o' as WCHAR, b'f' as WCHAR, b't' as WCHAR, b'w' as WCHAR,
        b'a' as WCHAR, b'r' as WCHAR, b'e' as WCHAR, b'\\' as WCHAR,
        b'M' as WCHAR, b'i' as WCHAR, b'c' as WCHAR, b'r' as WCHAR, b'o' as WCHAR,
        b's' as WCHAR, b'o' as WCHAR, b'f' as WCHAR, b't' as WCHAR, b'\\' as WCHAR,
        b'C' as WCHAR, b'r' as WCHAR, b'y' as WCHAR, b'p' as WCHAR, b't' as WCHAR,
        b'o' as WCHAR, b'g' as WCHAR, b'r' as WCHAR, b'a' as WCHAR, b'p' as WCHAR,
        b'h' as WCHAR, b'y' as WCHAR, b'\\' as WCHAR,
        b'D' as WCHAR, b'e' as WCHAR, b'f' as WCHAR, b'a' as WCHAR, b'u' as WCHAR,
        b'l' as WCHAR, b't' as WCHAR, b's' as WCHAR, b'\\' as WCHAR,
        b'P' as WCHAR, b'r' as WCHAR, b'o' as WCHAR, b'v' as WCHAR, b'i' as WCHAR,
        b'd' as WCHAR, b'e' as WCHAR, b'r' as WCHAR,
    ];
    static typeW: [WCHAR; 5] = [
        b'T' as WCHAR, b'y' as WCHAR, b'p' as WCHAR, b'e' as WCHAR, 0,
    ];
    let mut providerW_z: [WCHAR; 50] = [0; 50];
    core::ptr::copy_nonoverlapping(providerW.as_ptr(), providerW_z.as_mut_ptr(), 49);

    let ret: BOOL;

    TRACE!(
        b"(%d, %p, %d, %p, %p, %p)\n\0".as_ptr() as LPCSTR,
        dwIndex,
        pdwReserved,
        dwFlags,
        pdwProvType,
        pszProvName,
        pcbProvName,
    );

    if !pdwReserved.is_null() || pcbProvName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if dwFlags != 0 {
        SetLastError(NTE_BAD_FLAGS);
        return 0;
    }

    if RegOpenKeyW(HKEY_LOCAL_MACHINE, providerW_z.as_ptr(), &mut hKey) != 0 {
        SetLastError(NTE_FAIL);
        return 0;
    }

    let mut ret = 1;
    if pszProvName.is_null() {
        let mut numkeys: DWORD = 0;

        RegQueryInfoKeyW(
            hKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut numkeys,
            pcbProvName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        let provNameW = CRYPT_Alloc(*pcbProvName as usize * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
        if provNameW.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            RegCloseKey(hKey);
            return 0;
        }

        RegEnumKeyExW(
            hKey,
            dwIndex,
            provNameW,
            pcbProvName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        CRYPT_Free(provNameW as *mut _);
        *pcbProvName += 1;
        *pcbProvName *= core::mem::size_of::<WCHAR>() as DWORD;

        if dwIndex >= numkeys {
            SetLastError(ERROR_NO_MORE_ITEMS);
            ret = 0;
        }
    } else {
        let mut size: DWORD = core::mem::size_of::<DWORD>() as DWORD;
        let result: DWORD;
        let mut subkey: HKEY = core::ptr::null_mut();

        result = (RegEnumKeyW(
            hKey,
            dwIndex,
            pszProvName,
            *pcbProvName / core::mem::size_of::<WCHAR>() as DWORD,
        )) as u32;
        if result != 0 {
            SetLastError(result);
            RegCloseKey(hKey);
            return 0;
        }
        if RegOpenKeyW(hKey, pszProvName, &mut subkey) != 0 {
            RegCloseKey(hKey);
            return 0;
        }

        if RegQueryValueExW(
            subkey,
            typeW.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pdwProvType as LPBYTE,
            &mut size,
        ) != 0
        {
            ret = 0;
        }

        RegCloseKey(subkey);
    }
    RegCloseKey(hKey);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn CryptEnumProvidersA(
    dwIndex: DWORD,
    pdwReserved: *mut DWORD,
    dwFlags: DWORD,
    pdwProvType: *mut DWORD,
    mut pszProvName: LPSTR,
    pcbProvName: *mut DWORD,
) -> BOOL {
    let mut str: PWSTR = core::ptr::null_mut();
    let mut bufsize: DWORD = 0;
    let ret: BOOL;

    if CryptEnumProvidersW(
        dwIndex,
        pdwReserved,
        dwFlags,
        pdwProvType,
        core::ptr::null_mut(),
        &mut bufsize,
    ) == 0
    {
        return 0;
    }

    if !pszProvName.is_null() {
        str = CRYPT_Alloc(bufsize as usize) as PWSTR;
        if str.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
    }

    ret = CryptEnumProvidersW(
        dwIndex,
        pdwReserved,
        dwFlags,
        pdwProvType,
        str,
        &mut bufsize,
    );

    if !str.is_null() {
        CRYPT_UnicodeToANSI(str, &mut pszProvName, *pcbProvName as i32);
    }

    *pcbProvName = bufsize / core::mem::size_of::<u16>() as DWORD;

    if !str.is_null() {
        CRYPT_Free(str as *mut _);
        if GetLastError() == ERROR_INSUFFICIENT_BUFFER {
            SetLastError(ERROR_MORE_DATA);
            return 0;
        }
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn CryptEnumProviderTypesW(
    dwIndex: DWORD,
    pdwReserved: *mut DWORD,
    dwFlags: DWORD,
    pdwProvType: *mut DWORD,
    pszTypeName: LPWSTR,
    pcbTypeName: *mut DWORD,
) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubkey: HKEY = core::ptr::null_mut();
    let mut keylen: DWORD = 0;
    let mut numkeys: DWORD = 0;
    let mut dwType: DWORD = 0;
    let keyname: PWSTR;
    let mut ch: PWSTR;
    let result: DWORD;

    static KEYSTR: [WCHAR; 56] = [
        b'S' as WCHAR, b'o' as WCHAR, b'f' as WCHAR, b't' as WCHAR, b'w' as WCHAR,
        b'a' as WCHAR, b'r' as WCHAR, b'e' as WCHAR, b'\\' as WCHAR,
        b'M' as WCHAR, b'i' as WCHAR, b'c' as WCHAR, b'r' as WCHAR, b'o' as WCHAR,
        b's' as WCHAR, b'o' as WCHAR, b'f' as WCHAR, b't' as WCHAR, b'\\' as WCHAR,
        b'C' as WCHAR, b'r' as WCHAR, b'y' as WCHAR, b'p' as WCHAR, b't' as WCHAR,
        b'o' as WCHAR, b'g' as WCHAR, b'r' as WCHAR, b'a' as WCHAR, b'p' as WCHAR,
        b'h' as WCHAR, b'y' as WCHAR, b'\\' as WCHAR,
        b'D' as WCHAR, b'e' as WCHAR, b'f' as WCHAR, b'a' as WCHAR, b'u' as WCHAR,
        b'l' as WCHAR, b't' as WCHAR, b's' as WCHAR, b'\\' as WCHAR,
        b'P' as WCHAR, b'r' as WCHAR, b'o' as WCHAR, b'v' as WCHAR, b'i' as WCHAR,
        b'd' as WCHAR, b'e' as WCHAR, b'r' as WCHAR, b' ' as WCHAR,
        b'T' as WCHAR, b'y' as WCHAR, b'p' as WCHAR, b'e' as WCHAR, b's' as WCHAR, 0,
    ];
    static typenameW: [WCHAR; 9] = [
        b'T' as WCHAR, b'y' as WCHAR, b'p' as WCHAR, b'e' as WCHAR,
        b'N' as WCHAR, b'a' as WCHAR, b'm' as WCHAR, b'e' as WCHAR, 0,
    ];

    if !pdwReserved.is_null() || pdwProvType.is_null() || pcbTypeName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if dwFlags != 0 {
        SetLastError(NTE_BAD_FLAGS);
        return 0;
    }

    if RegOpenKeyW(HKEY_LOCAL_MACHINE, KEYSTR.as_ptr(), &mut hKey) != 0 {
        return 0;
    }

    RegQueryInfoKeyW(
        hKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut numkeys,
        &mut keylen,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if dwIndex >= numkeys {
        SetLastError(ERROR_NO_MORE_ITEMS);
        RegCloseKey(hKey);
        return 0;
    }
    keylen += 1;
    keyname = CRYPT_Alloc((keylen as usize) * core::mem::size_of::<WCHAR>()) as PWSTR;
    if keyname.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        RegCloseKey(hKey);
        return 0;
    }
    if RegEnumKeyW(hKey, dwIndex, keyname, keylen) != 0 {
        CRYPT_Free(keyname as *mut _);
        RegCloseKey(hKey);
        return 0;
    }
    RegOpenKeyW(hKey, keyname, &mut hSubkey);
    RegCloseKey(hKey);

    ch = keyname.add(strlenW(keyname) as usize);
    /* Convert "Type 000" to 0, etc/ */
    ch = ch.sub(1);
    *pdwProvType = (*ch as DWORD).wrapping_sub(b'0' as DWORD);
    ch = ch.sub(1);
    *pdwProvType = (*pdwProvType).wrapping_add((*ch as DWORD).wrapping_sub(b'0' as DWORD) * 10);
    ch = ch.sub(1);
    *pdwProvType = (*pdwProvType).wrapping_add((*ch as DWORD).wrapping_sub(b'0' as DWORD) * 100);
    CRYPT_Free(keyname as *mut _);

    result = (RegQueryValueExW(
        hSubkey,
        typenameW.as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        pszTypeName as LPBYTE,
        pcbTypeName,
    )) as u32;
    if result != 0 {
        SetLastError(result);
        RegCloseKey(hSubkey);
        return 0;
    }

    RegCloseKey(hSubkey);
    1
}

#[no_mangle]
pub unsafe extern "system" fn CryptEnumProviderTypesA(
    dwIndex: DWORD,
    pdwReserved: *mut DWORD,
    dwFlags: DWORD,
    pdwProvType: *mut DWORD,
    pszTypeName: LPSTR,
    pcbTypeName: *mut DWORD,
) -> BOOL {
    let mut str: PWSTR = core::ptr::null_mut();
    let mut bufsize: DWORD = 0;
    let ret: BOOL;

    TRACE!(
        b"(%d, %p, %08x, %p, %p, %p)\n\0".as_ptr() as LPCSTR,
        dwIndex,
        pdwReserved,
        dwFlags,
        pdwProvType,
        pszTypeName,
        pcbTypeName,
    );

    if CryptEnumProviderTypesW(
        dwIndex,
        pdwReserved,
        dwFlags,
        pdwProvType,
        core::ptr::null_mut(),
        &mut bufsize,
    ) == 0
    {
        return 0;
    }

    if !pszTypeName.is_null() {
        str = CRYPT_Alloc(bufsize as usize) as PWSTR;
        if str.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
    }

    ret = CryptEnumProviderTypesW(
        dwIndex,
        pdwReserved,
        dwFlags,
        pdwProvType,
        str,
        &mut bufsize,
    );

    if !str.is_null() {
        let mut local = pszTypeName;
        CRYPT_UnicodeToANSI(str, &mut local, *pcbTypeName as i32);
    }

    *pcbTypeName = bufsize / core::mem::size_of::<u16>() as DWORD;

    if !str.is_null() {
        CRYPT_Free(str as *mut _);
        if GetLastError() == ERROR_INSUFFICIENT_BUFFER {
            SetLastError(ERROR_MORE_DATA);
            return 0;
        }
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn CryptExportKey(
    hKey: HCRYPTKEY,
    hExpKey: HCRYPTKEY,
    dwBlobType: DWORD,
    dwFlags: DWORD,
    pbData: *mut BYTE,
    pdwDataLen: *mut DWORD,
) -> BOOL {
    let prov: PCRYPTPROV;
    let key: PCRYPTKEY = hKey as PCRYPTKEY;
    let expkey: PCRYPTKEY = hExpKey as PCRYPTKEY;

    TRACE!(
        b"(0x%lx, 0x%lx, %d, %08x, %p, %p)\n\0".as_ptr() as *const i8,
        hKey,
        hExpKey,
        dwBlobType,
        dwFlags,
        pbData,
        pdwDataLen,
    );

    if key.is_null()
        || pdwDataLen.is_null()
        || (*key).pProvider.is_null()
        || !handle_has_magic(key as usize, MAGIC_CRYPTKEY)
        || !handle_has_magic((*key).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    prov = (*key).pProvider;
    ((*(*prov).pFuncs).pCPExportKey.unwrap())(
        (*prov).hPrivate,
        (*key).hPrivate,
        if !expkey.is_null() { (*expkey).hPrivate } else { 0 },
        dwBlobType,
        dwFlags,
        pbData,
        pdwDataLen,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CryptGenKey(
    hProv: HCRYPTPROV,
    Algid: ALG_ID,
    dwFlags: DWORD,
    phKey: *mut HCRYPTKEY,
) -> BOOL {
    let prov: PCRYPTPROV = hProv as PCRYPTPROV;
    let key: PCRYPTKEY;

    TRACE!(
        b"(0x%lx, %d, %08x, %p)\n\0".as_ptr() as LPCSTR,
        hProv,
        Algid,
        dwFlags,
        phKey,
    );

    if phKey.is_null() || prov.is_null() || !handle_has_magic(prov as usize, MAGIC_CRYPTPROV) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    key = CRYPT_Alloc(core::mem::size_of::<CRYPTKEY>()) as PCRYPTKEY;
    if key.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    (*key).pProvider = prov;
    (*key).dwMagic = MAGIC_CRYPTKEY;
    if (*(*prov).pFuncs).pCPGenKey.unwrap()(
        (*prov).hPrivate,
        Algid,
        dwFlags,
        core::ptr::addr_of_mut!((*key).hPrivate),
    ) != 0
    {
        *phKey = key as HCRYPTKEY;
        return 1;
    }

    (*key).dwMagic = 0;
    CRYPT_Free(key as *mut _);
    0
}

#[no_mangle]
pub unsafe extern "system" fn CryptGetDefaultProviderW(
    dwProvType: u32,
    pdwReserved: *mut u32,
    dwFlags: u32,
    pszProvName: LPWSTR,
    pcbProvName: *mut u32,
) -> i32 {
    let mut hKey: HKEY = core::ptr::null_mut();
    let keyname: PWSTR;
    let result: u32;
    let nameW: [u16; 5] = [b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, 0];

    if !pdwReserved.is_null() || pcbProvName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if dwFlags & !(CRYPT_USER_DEFAULT | CRYPT_MACHINE_DEFAULT) != 0 {
        SetLastError(NTE_BAD_FLAGS);
        return 0;
    }
    if dwProvType > 999 {
        SetLastError(NTE_BAD_PROV_TYPE);
        return 0;
    }
    keyname = CRYPT_GetTypeKeyName(dwProvType, (dwFlags & CRYPT_USER_DEFAULT) as i32);
    if keyname.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    if RegOpenKeyW(
        if dwFlags & CRYPT_USER_DEFAULT != 0 {
            HKEY_CURRENT_USER
        } else {
            HKEY_LOCAL_MACHINE
        },
        keyname,
        &mut hKey,
    ) != 0
    {
        CRYPT_Free(keyname as LPVOID);
        SetLastError(NTE_PROV_TYPE_NOT_DEF);
        return 0;
    }
    CRYPT_Free(keyname as LPVOID);

    result = (RegQueryValueExW(
        hKey,
        nameW.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pszProvName as LPBYTE,
        pcbProvName,
    )) as u32;
    RegCloseKey(hKey);

    if result != 0 {
        if result != ERROR_MORE_DATA {
            SetLastError(NTE_PROV_TYPE_ENTRY_BAD);
        } else {
            SetLastError(result);
        }
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn CryptGetDefaultProviderA(
    dwProvType: u32,
    pdwReserved: *mut u32,
    dwFlags: u32,
    pszProvName: LPSTR,
    pcbProvName: *mut u32,
) -> BOOL {
    let mut str: PWSTR = core::ptr::null_mut();
    let mut bufsize: u32 = 0;
    let ret: BOOL;

    TRACE!(
        b"(%d, %p, %08x, %p, %p)\n\0".as_ptr() as LPCSTR,
        dwProvType,
        pdwReserved,
        dwFlags,
        pszProvName,
        pcbProvName,
    );

    CryptGetDefaultProviderW(
        dwProvType,
        pdwReserved,
        dwFlags,
        core::ptr::null_mut(),
        &mut bufsize,
    );
    if !pszProvName.is_null() {
        str = CRYPT_Alloc(bufsize as usize) as PWSTR;
        if str.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
    }
    ret = CryptGetDefaultProviderW(dwProvType, pdwReserved, dwFlags, str, &mut bufsize);
    if !str.is_null() {
        let mut local_pszProvName = pszProvName;
        CRYPT_UnicodeToANSI(str, &mut local_pszProvName, *pcbProvName as i32);
    }
    *pcbProvName = bufsize / core::mem::size_of::<u16>() as u32;
    if !str.is_null() {
        CRYPT_Free(str as LPVOID);
        if GetLastError() == ERROR_INSUFFICIENT_BUFFER {
            SetLastError(ERROR_MORE_DATA);
            return 0;
        }
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn CryptGetHashParam(
    hHash: HCRYPTHASH,
    dwParam: DWORD,
    pbData: *mut BYTE,
    pdwDataLen: *mut DWORD,
    dwFlags: DWORD,
) -> BOOL {
    let hash: PCRYPTHASH = hHash as PCRYPTHASH;

    TRACE!(
        b"(0x%lx, %d, %p, %p, %08x)\n\0".as_ptr() as *const i8,
        hHash,
        dwParam,
        pbData,
        pdwDataLen,
        dwFlags,
    );

    if hash.is_null()
        || pdwDataLen.is_null()
        || (*hash).pProvider.is_null()
        || !handle_has_magic(hash as usize, MAGIC_CRYPTHASH)
        || !handle_has_magic((*hash).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let prov: PCRYPTPROV = (*hash).pProvider;
    ((*(*prov).pFuncs).pCPGetHashParam.unwrap())(
        (*prov).hPrivate,
        (*hash).hPrivate,
        dwParam,
        pbData,
        pdwDataLen,
        dwFlags,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CryptGetKeyParam(
    hKey: HCRYPTKEY,
    dwParam: DWORD,
    pbData: *mut BYTE,
    pdwDataLen: *mut DWORD,
    dwFlags: DWORD,
) -> BOOL {
    let key: PCRYPTKEY = hKey as PCRYPTKEY;

    if key.is_null()
        || pdwDataLen.is_null()
        || (*key).pProvider.is_null()
        || !handle_has_magic(key as usize, MAGIC_CRYPTKEY)
        || !handle_has_magic((*key).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let prov: PCRYPTPROV = (*key).pProvider;
    ((*(*prov).pFuncs).pCPGetKeyParam.unwrap())(
        (*prov).hPrivate,
        (*key).hPrivate,
        dwParam,
        pbData,
        pdwDataLen,
        dwFlags,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CryptGetProvParam(
    hProv: HCRYPTPROV,
    dwParam: DWORD,
    pbData: *mut BYTE,
    pdwDataLen: *mut DWORD,
    dwFlags: DWORD,
) -> BOOL {
    let prov = hProv as PCRYPTPROV;

    if prov.is_null() || !handle_has_magic(prov as usize, MAGIC_CRYPTPROV) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    ((*(*prov).pFuncs).pCPGetProvParam.unwrap())(
        (*prov).hPrivate,
        dwParam,
        pbData,
        pdwDataLen,
        dwFlags,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CryptGetUserKey(
    hProv: HCRYPTPROV,
    dwKeySpec: DWORD,
    phUserKey: *mut HCRYPTKEY,
) -> BOOL {
    let prov = hProv as PCRYPTPROV;
    let key: PCRYPTKEY;

    if prov.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    if phUserKey.is_null() || !handle_has_magic(prov as usize, MAGIC_CRYPTPROV) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    key = CRYPT_Alloc(core::mem::size_of::<CRYPTKEY>() as _) as PCRYPTKEY;
    if key.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    (*key).pProvider = prov;
    (*key).dwMagic = MAGIC_CRYPTKEY;
    if (*(*prov).pFuncs).pCPGetUserKey.unwrap()(
        (*prov).hPrivate,
        dwKeySpec,
        core::ptr::addr_of_mut!((*key).hPrivate),
    ) != 0
    {
        *phUserKey = key as HCRYPTKEY;
        return 1;
    }

    (*key).dwMagic = 0;
    CRYPT_Free(key as _);
    *phUserKey = 0;
    0
}

#[no_mangle]
pub unsafe extern "system" fn CryptHashData(
    hHash: HCRYPTHASH,
    pbData: *const BYTE,
    dwDataLen: DWORD,
    dwFlags: DWORD,
) -> BOOL {
    let hash = hHash as PCRYPTHASH;

    if hash.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    if (*hash).pProvider.is_null()
        || !handle_has_magic(hash as usize, MAGIC_CRYPTHASH)
        || !handle_has_magic((*hash).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let prov = (*hash).pProvider;
    ((*(*prov).pFuncs).pCPHashData.unwrap())(
        (*prov).hPrivate,
        (*hash).hPrivate,
        pbData,
        dwDataLen,
        dwFlags,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CryptHashSessionKey(
    hHash: HCRYPTHASH,
    hKey: HCRYPTKEY,
    dwFlags: DWORD,
) -> BOOL {
    let hash = hHash as PCRYPTHASH;
    let key = hKey as PCRYPTKEY;
    let prov: PCRYPTPROV;

    TRACE!(b"(0x%lx, 0x%lx, %08x)\n\0".as_ptr() as *const i8, hHash, hKey, dwFlags);

    if hash.is_null() || key.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if (*hash).pProvider.is_null()
        || !handle_has_magic(hash as usize, MAGIC_CRYPTHASH)
        || !handle_has_magic((*hash).pProvider as usize, MAGIC_CRYPTPROV)
        || !handle_has_magic(key as usize, MAGIC_CRYPTKEY)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    prov = (*hash).pProvider;
    ((*(*prov).pFuncs).pCPHashSessionKey.unwrap())(
        (*prov).hPrivate,
        (*hash).hPrivate,
        (*key).hPrivate,
        dwFlags,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CryptImportKey(
    hProv: HCRYPTPROV,
    pbData: *const BYTE,
    dwDataLen: DWORD,
    hPubKey: HCRYPTKEY,
    dwFlags: DWORD,
    phKey: *mut HCRYPTKEY,
) -> BOOL {
    let prov = hProv as PCRYPTPROV;
    let pubkey = hPubKey as PCRYPTKEY;

    TRACE!(
        b"(0x%lx, %p, %d, 0x%lx, %08x, %p)\n\0".as_ptr() as *const i8,
        hProv,
        pbData,
        dwDataLen,
        hPubKey,
        dwFlags,
        phKey,
    );

    if prov.is_null()
        || pbData.is_null()
        || dwDataLen == 0
        || phKey.is_null()
        || !handle_has_magic(prov as usize, MAGIC_CRYPTPROV)
        || (!pubkey.is_null() && !handle_has_magic(pubkey as usize, MAGIC_CRYPTKEY))
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let importkey = CRYPT_Alloc(core::mem::size_of::<CRYPTKEY>()) as PCRYPTKEY;
    if importkey.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    (*importkey).pProvider = prov;
    (*importkey).dwMagic = MAGIC_CRYPTKEY;
    if (*(*prov).pFuncs).pCPImportKey.unwrap()(
        (*prov).hPrivate,
        pbData,
        dwDataLen,
        if !pubkey.is_null() { (*pubkey).hPrivate } else { 0 },
        dwFlags,
        core::ptr::addr_of_mut!((*importkey).hPrivate),
    ) != 0
    {
        *phKey = importkey as HCRYPTKEY;
        return 1;
    }

    (*importkey).dwMagic = 0;
    CRYPT_Free(importkey as *mut _);
    0
}

#[no_mangle]
pub unsafe extern "system" fn CryptSignHashW(
    hHash: HCRYPTHASH,
    dwKeySpec: DWORD,
    sDescription: LPCWSTR,
    dwFlags: DWORD,
    pbSignature: *mut BYTE,
    pdwSigLen: *mut DWORD,
) -> BOOL {
    let hash: PCRYPTHASH = hHash as PCRYPTHASH;
    let prov: PCRYPTPROV;

    TRACE!(
        b"(0x%lx, %d, %s, %08x, %p, %p)\n\0".as_ptr() as *const i8,
        hHash,
        dwKeySpec,
        debugstr_w(sDescription),
        dwFlags,
        pbSignature,
        pdwSigLen,
    );

    if hash.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    if pdwSigLen.is_null()
        || (*hash).pProvider.is_null()
        || !handle_has_magic(hash as usize, MAGIC_CRYPTHASH)
        || !handle_has_magic((*hash).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    prov = (*hash).pProvider;
    ((*(*prov).pFuncs).pCPSignHash.unwrap())(
        (*prov).hPrivate,
        (*hash).hPrivate,
        dwKeySpec,
        sDescription,
        dwFlags,
        pbSignature,
        pdwSigLen,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CryptSignHashA(
    hHash: HCRYPTHASH,
    dwKeySpec: DWORD,
    sDescription: LPCSTR,
    dwFlags: DWORD,
    pbSignature: *mut BYTE,
    pdwSigLen: *mut DWORD,
) -> BOOL {
    let mut wsDescription: LPWSTR = core::ptr::null_mut();
    let result: BOOL;

    CRYPT_ANSIToUnicode(sDescription, &mut wsDescription, -1);
    result = CryptSignHashW(hHash, dwKeySpec, wsDescription, dwFlags, pbSignature, pdwSigLen);
    CRYPT_Free(wsDescription as _);

    result
}

#[no_mangle]
pub unsafe extern "system" fn CryptSetHashParam(
    hHash: HCRYPTHASH,
    dwParam: DWORD,
    pbData: *const BYTE,
    dwFlags: DWORD,
) -> BOOL {
    let hash: PCRYPTHASH = hHash as PCRYPTHASH;

    if hash.is_null()
        || pbData.is_null()
        || (*hash).pProvider.is_null()
        || !handle_has_magic(hash as usize, MAGIC_CRYPTHASH)
        || !handle_has_magic((*hash).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let prov: PCRYPTPROV = (*hash).pProvider;
    ((*(*prov).pFuncs).pCPSetHashParam.unwrap())(
        (*prov).hPrivate,
        (*hash).hPrivate,
        dwParam,
        pbData,
        dwFlags,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CryptSetKeyParam(
    hKey: HCRYPTKEY,
    dwParam: DWORD,
    pbData: *const BYTE,
    dwFlags: DWORD,
) -> BOOL {
    let key: PCRYPTKEY = hKey as PCRYPTKEY;

    TRACE!(
        b"(0x%lx, %d, %p, %08x)\n\0".as_ptr() as *const i8,
        hKey,
        dwParam,
        pbData,
        dwFlags,
    );

    if key.is_null()
        || pbData.is_null()
        || (*key).pProvider.is_null()
        || !handle_has_magic(key as usize, MAGIC_CRYPTKEY)
        || !handle_has_magic((*key).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let prov: PCRYPTPROV = (*key).pProvider;
    ((*(*prov).pFuncs).pCPSetKeyParam.unwrap())(
        (*prov).hPrivate,
        (*key).hPrivate,
        dwParam,
        pbData,
        dwFlags,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CryptSetProviderA(
    pszProvName: LPCSTR,
    dwProvType: u32,
) -> BOOL {
    CryptSetProviderExA(pszProvName, dwProvType, core::ptr::null_mut(), CRYPT_USER_DEFAULT)
}

#[no_mangle]
pub unsafe extern "system" fn CryptSetProviderW(
    pszProvName: LPCWSTR,
    dwProvType: u32,
) -> i32 {
    CryptSetProviderExW(pszProvName, dwProvType, core::ptr::null_mut(), CRYPT_USER_DEFAULT)
}

#[no_mangle]
pub unsafe extern "system" fn CryptSetProviderExW(
    pszProvName: LPCWSTR,
    dwProvType: u32,
    pdwReserved: *mut u32,
    dwFlags: u32,
) -> i32 {
    let mut hProvKey: HKEY = core::ptr::null_mut();
    let mut hTypeKey: HKEY = core::ptr::null_mut();
    let mut keyname: PWSTR;
    static nameW: [u16; 5] = [b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, 0];

    TRACE!(
        b"(%s, %d, %p, %08x)\n\0".as_ptr() as LPCSTR,
        debugstr_w(pszProvName),
        dwProvType,
        pdwReserved,
        dwFlags,
    );

    if pszProvName.is_null() || !pdwReserved.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if dwProvType > MAXPROVTYPES {
        SetLastError(NTE_BAD_PROV_TYPE);
        return 0;
    }
    if (dwFlags & !(CRYPT_MACHINE_DEFAULT | CRYPT_USER_DEFAULT | CRYPT_DELETE_DEFAULT)) != 0
        || dwFlags == CRYPT_DELETE_DEFAULT
    {
        SetLastError(NTE_BAD_FLAGS);
        return 0;
    }

    keyname = CRYPT_GetTypeKeyName(dwProvType, (dwFlags & CRYPT_USER_DEFAULT) as i32);
    if keyname.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    if RegOpenKeyW(
        if (dwFlags & CRYPT_USER_DEFAULT) != 0 {
            HKEY_CURRENT_USER
        } else {
            HKEY_LOCAL_MACHINE
        },
        keyname,
        &mut hTypeKey,
    ) != 0
    {
        CRYPT_Free(keyname as _);
        SetLastError(NTE_BAD_PROVIDER);
        return 0;
    }
    CRYPT_Free(keyname as _);

    if (dwFlags & CRYPT_DELETE_DEFAULT) != 0 {
        RegDeleteValueW(hTypeKey, nameW.as_ptr());
    } else {
        keyname = CRYPT_GetProvKeyName(pszProvName);
        if keyname.is_null() {
            RegCloseKey(hTypeKey);
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        if RegOpenKeyW(
            if (dwFlags & CRYPT_USER_DEFAULT) != 0 {
                HKEY_CURRENT_USER
            } else {
                HKEY_LOCAL_MACHINE
            },
            keyname,
            &mut hProvKey,
        ) != 0
        {
            CRYPT_Free(keyname as _);
            RegCloseKey(hTypeKey);
            SetLastError(NTE_BAD_PROVIDER);
            return 0;
        }
        CRYPT_Free(keyname as _);

        if RegSetValueExW(
            hTypeKey,
            nameW.as_ptr(),
            0,
            REG_SZ,
            pszProvName as LPBYTE,
            ((strlenW(pszProvName) + 1) * core::mem::size_of::<u16>()) as u32,
        ) != 0
        {
            RegCloseKey(hTypeKey);
            RegCloseKey(hProvKey);
            return 0;
        }

        RegCloseKey(hProvKey);
    }
    RegCloseKey(hTypeKey);

    1
}

#[no_mangle]
pub unsafe extern "system" fn CryptSetProviderExA(
    pszProvName: LPCSTR,
    dwProvType: u32,
    pdwReserved: *mut u32,
    dwFlags: u32,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut str: PWSTR = core::ptr::null_mut();

    if CRYPT_ANSIToUnicode(pszProvName, &mut str, -1) != 0 {
        ret = CryptSetProviderExW(str, dwProvType, pdwReserved, dwFlags);
        CRYPT_Free(str as LPVOID);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn CryptSetProvParam(
    hProv: HCRYPTPROV,
    dwParam: DWORD,
    pbData: *const BYTE,
    dwFlags: DWORD,
) -> BOOL {
    let prov: PCRYPTPROV = hProv as PCRYPTPROV;

    if prov.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    if !handle_has_magic(prov as usize, MAGIC_CRYPTPROV) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if dwParam == PP_USE_HARDWARE_RNG {
    }
    if (dwFlags & PP_CLIENT_HWND) != 0 {
        if !pbData.is_null() {
            crypt_hWindow = pbData as HWND;
            return 1;
        } else {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }
    ((*(*prov).pFuncs).pCPSetProvParam.unwrap())(
        (*prov).hPrivate,
        dwParam,
        pbData,
        dwFlags,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CryptVerifySignatureW(
    hHash: HCRYPTHASH,
    pbSignature: *const BYTE,
    dwSigLen: DWORD,
    hPubKey: HCRYPTKEY,
    sDescription: LPCWSTR,
    dwFlags: DWORD,
) -> BOOL {
    let hash = hHash as PCRYPTHASH;
    let key = hPubKey as PCRYPTKEY;

    TRACE!(
        b"(0x%lx, %p, %d, 0x%lx, %s, %08x)\n\0".as_ptr() as LPCSTR,
        hHash,
        pbSignature,
        dwSigLen,
        hPubKey,
        debugstr_w(sDescription),
        dwFlags,
    );

    if hash.is_null()
        || key.is_null()
        || !handle_has_magic(key as usize, MAGIC_CRYPTKEY)
        || !handle_has_magic(hash as usize, MAGIC_CRYPTHASH)
        || (*hash).pProvider.is_null()
        || !handle_has_magic((*hash).pProvider as usize, MAGIC_CRYPTPROV)
        || (*key).pProvider.is_null()
        || !handle_has_magic((*key).pProvider as usize, MAGIC_CRYPTPROV)
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let prov = (*hash).pProvider;
    ((*(*prov).pFuncs).pCPVerifySignature.unwrap())(
        (*prov).hPrivate,
        (*hash).hPrivate,
        pbSignature,
        dwSigLen,
        (*key).hPrivate,
        sDescription,
        dwFlags,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CryptVerifySignatureA(
    hHash: HCRYPTHASH,
    pbSignature: *const BYTE,
    dwSigLen: DWORD,
    hPubKey: HCRYPTKEY,
    sDescription: LPCSTR,
    dwFlags: DWORD,
) -> BOOL {
    let mut wsDescription: LPWSTR = core::ptr::null_mut();
    let result: BOOL;

    CRYPT_ANSIToUnicode(sDescription, &mut wsDescription, -1);
    result = CryptVerifySignatureW(
        hHash,
        pbSignature,
        dwSigLen,
        hPubKey,
        wsDescription,
        dwFlags,
    );
    CRYPT_Free(wsDescription as _);

    result
}

#[no_mangle]
pub unsafe extern "system" fn OpenEncryptedFileRawA(
    filename: LPCSTR,
    flags: u32,
    context: *mut *mut core::ffi::c_void,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn OpenEncryptedFileRawW(
    filename: LPCWSTR,
    flags: ULONG,
    context: *mut PVOID,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn ReadEncryptedFileRaw(
    export: PFE_EXPORT_FUNC,
    callback: *mut core::ffi::c_void,
    context: *mut core::ffi::c_void,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}

// NOTE: SystemFunction030/035/036/040/041 are also exported by sysfunc.rs (and
// the still-C wine/crypt.c). Keep these faithful bodies but drop #[no_mangle] so
// the single Rust export comes from sysfunc.rs (avoids in-rlib symbol clash).
pub unsafe extern "system" fn SystemFunction030(b1: LPCVOID, b2: LPCVOID) -> i32 {
    (memcmp(b1, b2, 0x10) == 0) as i32
}

pub unsafe extern "system" fn SystemFunction035(_lpszDllFilePath: LPCSTR) -> BOOL {
    1
}

pub unsafe extern "system" fn SystemFunction036(pbBuffer: PVOID, dwLen: ULONG) -> BOOLEAN {

    let dev_random = open(b"/dev/urandom\0".as_ptr() as *const i8, O_RDONLY);
    if dev_random != -1 {
        if IsBadWritePtr(pbBuffer, dwLen as usize) == 0
            && read(dev_random, pbBuffer, dwLen as usize) == dwLen as isize
        {
            close(dev_random);
            return 1u8;
        }
        close(dev_random);
    } else {
        // FIXME("couldn't open /dev/urandom\n");
    }
    SetLastError(NTE_FAIL);
    0u8
}

pub unsafe extern "system" fn SystemFunction040(
    memory: PVOID,
    length: ULONG,
    flags: ULONG,
) -> NTSTATUS {
    STATUS_SUCCESS
}

pub unsafe extern "system" fn SystemFunction041(
    memory: *mut core::ffi::c_void,
    length: u32,
    flags: u32,
) -> i32 {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WriteEncryptedFileRaw(
    import: PFE_IMPORT_FUNC,
    callback: *mut core::ffi::c_void,
    context: *mut core::ffi::c_void,
) -> u32 {
    ERROR_CALL_NOT_IMPLEMENTED
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright 1999 Ian Schmidt
//  * Copyright 2001 Travis Michielsen
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
// /***********************************************************************
//  *
//  *  TODO:
//  *  - Reference counting
//  *  - Thread-safing
//  */
//
// #ifdef __REACTOS__
// #include <advapi32.h>
// #else
// #include "config.h"
// #include "wine/port.h"
//
// #include <limits.h>
// #include <time.h>
// #include <stdlib.h>
// #include <stdio.h>
// #include <sys/types.h>
// #ifdef HAVE_SYS_STAT_H
// # include <sys/stat.h>
// #endif
// #include <fcntl.h>
// #ifdef HAVE_UNISTD_H
// # include <unistd.h>
// #endif
//
// #include "ntstatus.h"
// #define WIN32_NO_STATUS
// #include "crypt.h"
// #include "winnls.h"
// #include "winreg.h"
// #include "rpc.h"
// #include "wine/debug.h"
// #include "wine/unicode.h"
// #include "winternl.h"
// #endif /* __REACTOS__ */
//
// WINE_DEFAULT_DEBUG_CHANNEL(crypt);
//
// static HWND crypt_hWindow;
//
// #define CRYPT_Alloc(size) (LocalAlloc(LMEM_ZEROINIT, size))
// #define CRYPT_Free(buffer) (LocalFree(buffer))
//
// static inline PWSTR CRYPT_GetProvKeyName(PCWSTR pProvName)
// {
// 	static const WCHAR KEYSTR[] = {
//                 'S','o','f','t','w','a','r','e','\\',
//                 'M','i','c','r','o','s','o','f','t','\\',
//                 'C','r','y','p','t','o','g','r','a','p','h','y','\\',
//                 'D','e','f','a','u','l','t','s','\\',
//                 'P','r','o','v','i','d','e','r','\\',0
// 	};
// 	PWSTR keyname;
//
// 	keyname = CRYPT_Alloc((strlenW(KEYSTR) + strlenW(pProvName) +1)*sizeof(WCHAR));
// 	if (keyname)
// 	{
// 		strcpyW(keyname, KEYSTR);
// 		strcpyW(keyname + strlenW(KEYSTR), pProvName);
// 	} else
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 	return keyname;
// }
//
// static inline PWSTR CRYPT_GetTypeKeyName(DWORD dwType, BOOL user)
// {
// 	static const WCHAR MACHINESTR[] = {
//                 'S','o','f','t','w','a','r','e','\\',
//                 'M','i','c','r','o','s','o','f','t','\\',
//                 'C','r','y','p','t','o','g','r','a','p','h','y','\\',
//                 'D','e','f','a','u','l','t','s','\\',
//                 'P','r','o','v','i','d','e','r',' ','T','y','p','e','s','\\',
//                 'T','y','p','e',' ','X','X','X',0
// 	};
// 	static const WCHAR USERSTR[] = {
//                 'S','o','f','t','w','a','r','e','\\',
//                 'M','i','c','r','o','s','o','f','t','\\',
//                 'C','r','y','p','t','o','g','r','a','p','h','y','\\',
//                 'P','r','o','v','i','d','e','r',' ','T','y','p','e',' ','X','X','X',0
// 	};
// 	PWSTR keyname;
// 	PWSTR ptr;
//
// 	keyname = CRYPT_Alloc( ((user ? strlenW(USERSTR) : strlenW(MACHINESTR)) +1)*sizeof(WCHAR));
// 	if (keyname)
// 	{
// 		user ? strcpyW(keyname, USERSTR) : strcpyW(keyname, MACHINESTR);
// 		ptr = keyname + strlenW(keyname);
// 		*(--ptr) = (dwType % 10) + '0';
// 		*(--ptr) = ((dwType / 10) % 10) + '0';
// 		*(--ptr) = (dwType / 100) + '0';
// 	} else
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 	return keyname;
// }
//
// /* CRYPT_UnicodeToANSI
//  * wstr - unicode string
//  * str - pointer to ANSI string or pointer to null pointer if we have to do the allocation
//  * strsize - size of buffer pointed to by str
//  *
//  * returns TRUE if unsuccessful, FALSE otherwise.
//  * if wstr is NULL, returns TRUE and sets str to NULL! Value of str should be checked after call
//  */
// static inline BOOL CRYPT_UnicodeToANSI(LPCWSTR wstr, LPSTR* str, int strsize)
// {
// 	if (!wstr)
// 	{
// 		*str = NULL;
// 		return TRUE;
// 	}
//
// 	if (!*str)
// 	{
// 		strsize = WideCharToMultiByte(CP_ACP, 0, wstr, -1, NULL, 0, NULL, NULL);
// 		*str = CRYPT_Alloc(strsize * sizeof(CHAR));
// 	}
// 	else if (strsize < 0)
// 	{
// 		strsize = INT_MAX; /* windows will pretend that the buffer is infinitely long */
// 	}
//
// 	if (*str)
// 	{
// 		WideCharToMultiByte(CP_ACP, 0, wstr, -1, *str, strsize, NULL, NULL);
// 		return TRUE;
// 	}
//
// 	SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 	return FALSE;
// }
//
// /* CRYPT_ANSITOUnicode
//  * str - ANSI string
//  * wstr - pointer to unicode string
//  * wstrsize - size of buffer pointed to by wstr or -1 if we have to do the allocation
//  */
// static inline BOOL CRYPT_ANSIToUnicode(LPCSTR str, LPWSTR* wstr, int wstrsize)
// {
// 	unsigned int wcount;
//
// 	if (!str)
// 	{
// 		*wstr = NULL;
// 		return TRUE;
// 	}
// 	wcount = MultiByteToWideChar(CP_ACP, 0, str, -1, NULL, 0);
// 	if (wstrsize == -1)
// 		*wstr = CRYPT_Alloc(wcount * sizeof(WCHAR));
// 	else
// 		wcount = min( wcount, wstrsize/sizeof(WCHAR) );
// 	if (*wstr)
// 	{
// 		MultiByteToWideChar(CP_ACP, 0, str, -1, *wstr, wcount);
// 		return TRUE;
// 	}
// 	SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 	return FALSE;
// }
//
// /* These next 2 functions are used by the VTableProvStruc structure */
// static BOOL __stdcall CRYPT_VerifyImage(LPCSTR lpszImage, BYTE* pData)
// {
// 	if (!lpszImage || !pData)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	FIXME("(%s, %p): not verifying image\n", lpszImage, pData);
//
// 	return TRUE;
// }
//
// static void __stdcall CRYPT_ReturnhWnd(HWND *phWnd)
// {
// 	if (phWnd) *phWnd = crypt_hWindow;
// }
//
// #define CRYPT_GetProvFunc(name) \
// 	if ( !(provider->pFuncs->p##name = (void*)GetProcAddress(provider->hModule, #name)) ) goto error
// #define CRYPT_GetProvFuncOpt(name) \
// 	provider->pFuncs->p##name = (void*)GetProcAddress(provider->hModule, #name)
// static PCRYPTPROV CRYPT_LoadProvider(PCWSTR pImage)
// {
// 	PCRYPTPROV provider;
// 	DWORD errorcode = ERROR_NOT_ENOUGH_MEMORY;
//
// 	if ( !(provider = CRYPT_Alloc(sizeof(CRYPTPROV))) ) goto error;
// 	if ( !(provider->pFuncs = CRYPT_Alloc(sizeof(PROVFUNCS))) ) goto error;
// 	if ( !(provider->pVTable = CRYPT_Alloc(sizeof(VTableProvStruc))) ) goto error;
// 	if ( !(provider->hModule = LoadLibraryW(pImage)) )
// 	{
// 		errorcode = (GetLastError() == ERROR_FILE_NOT_FOUND) ? NTE_PROV_DLL_NOT_FOUND : NTE_PROVIDER_DLL_FAIL;
// 		FIXME("Failed to load dll %s\n", debugstr_w(pImage));
// 		goto error;
// 	}
// 	provider->dwMagic = MAGIC_CRYPTPROV;
// 	provider->refcount = 1;
//
// 	errorcode = NTE_PROVIDER_DLL_FAIL;
// 	CRYPT_GetProvFunc(CPAcquireContext);
// 	CRYPT_GetProvFunc(CPCreateHash);
// 	CRYPT_GetProvFunc(CPDecrypt);
// 	CRYPT_GetProvFunc(CPDeriveKey);
// 	CRYPT_GetProvFunc(CPDestroyHash);
// 	CRYPT_GetProvFunc(CPDestroyKey);
// 	CRYPT_GetProvFuncOpt(CPDuplicateHash);
// 	CRYPT_GetProvFuncOpt(CPDuplicateKey);
// 	CRYPT_GetProvFunc(CPEncrypt);
// 	CRYPT_GetProvFunc(CPExportKey);
// 	CRYPT_GetProvFunc(CPGenKey);
// 	CRYPT_GetProvFunc(CPGenRandom);
// 	CRYPT_GetProvFunc(CPGetHashParam);
// 	CRYPT_GetProvFunc(CPGetKeyParam);
// 	CRYPT_GetProvFunc(CPGetProvParam);
// 	CRYPT_GetProvFunc(CPGetUserKey);
// 	CRYPT_GetProvFunc(CPHashData);
// 	CRYPT_GetProvFunc(CPHashSessionKey);
// 	CRYPT_GetProvFunc(CPImportKey);
// 	CRYPT_GetProvFunc(CPReleaseContext);
// 	CRYPT_GetProvFunc(CPSetHashParam);
// 	CRYPT_GetProvFunc(CPSetKeyParam);
// 	CRYPT_GetProvFunc(CPSetProvParam);
// 	CRYPT_GetProvFunc(CPSignHash);
// 	CRYPT_GetProvFunc(CPVerifySignature);
//
// 	/* FIXME: Not sure what the pbContextInfo field is for.
// 	 *        Does it need memory allocation?
//          */
// 	provider->pVTable->Version = 3;
// 	provider->pVTable->FuncVerifyImage = CRYPT_VerifyImage;
// 	provider->pVTable->FuncReturnhWnd = CRYPT_ReturnhWnd;
// 	provider->pVTable->dwProvType = 0;
// 	provider->pVTable->pbContextInfo = NULL;
// 	provider->pVTable->cbContextInfo = 0;
// 	provider->pVTable->pszProvName = NULL;
// 	return provider;
//
// error:
// 	SetLastError(errorcode);
// 	if (provider)
// 	{
// 		provider->dwMagic = 0;
// 		if (provider->hModule)
// 			FreeLibrary(provider->hModule);
// 		CRYPT_Free(provider->pVTable);
// 		CRYPT_Free(provider->pFuncs);
// 		CRYPT_Free(provider);
// 	}
// 	return NULL;
// }
// #undef CRYPT_GetProvFunc
// #undef CRYPT_GetProvFuncOpt
//
//
// static void CRYPT_CreateMachineGuid(void)
// {
// 	static const WCHAR cryptographyW[] = {
//                 'S','o','f','t','w','a','r','e','\\',
//                 'M','i','c','r','o','s','o','f','t','\\',
//                 'C','r','y','p','t','o','g','r','a','p','h','y',0 };
// 	static const WCHAR machineGuidW[] = {
// 		'M','a','c','h','i','n','e','G','u','i','d',0 };
// 	LONG r;
// 	HKEY key;
//
// 	r = RegOpenKeyExW(HKEY_LOCAL_MACHINE, cryptographyW, 0, KEY_ALL_ACCESS,
// 			  &key);
// 	if (!r)
// 	{
// 		DWORD size;
//
// 		r = RegQueryValueExW(key, machineGuidW, NULL, NULL, NULL, &size);
// 		if (r == ERROR_FILE_NOT_FOUND)
// 		{
//                     UUID uuid;
//                     WCHAR buf[37];
//                     RPC_STATUS rs;
//                     static const WCHAR uuidFmt[] = {
//                         '%','0','8','x','-','%','0','4','x','-',
//                         '%','0','4','x','-','%','0','2','x',
//                         '%','0','2','x','-','%','0','2','x',
//                         '%','0','2','x','%','0','2','x',
//                         '%','0','2','x','%','0','2','x',
//                         '%','0','2','x',0 };
//
//                     rs = UuidCreate(&uuid);
//                     if (rs == S_OK)
//                     {
//                         sprintfW(buf, uuidFmt,
//                                  uuid.Data1, uuid.Data2, uuid.Data3,
//                                  uuid.Data4[0], uuid.Data4[1],
//                                  uuid.Data4[2], uuid.Data4[3],
//                                  uuid.Data4[4], uuid.Data4[5],
//                                  uuid.Data4[6], uuid.Data4[7] );
//                         RegSetValueExW(key, machineGuidW, 0, REG_SZ,
//                                        (const BYTE *)buf,
//                                        (lstrlenW(buf)+1)*sizeof(WCHAR));
//                     }
// 		}
// 		RegCloseKey(key);
// 	}
// }
//
//
// /******************************************************************************
//  * CloseEncryptedFileRaw   (ADVAPI32.@)
//  *
//  * Close encrypted files
//  *
//  * PARAMS
//  *  context    [I] pointer to the context
//  * RETURNS
//  *  Success: ERROR_SUCCESS
//  *  Failure: NTSTATUS error code
//  */
// void WINAPI CloseEncryptedFileRaw(PVOID context)
// {
//     FIXME("(%p): stub\n", context);
// }
//
// /******************************************************************************
//  * CryptAcquireContextW (ADVAPI32.@)
//  *
//  * Acquire a crypto provider context handle.
//  *
//  * PARAMS
//  *  phProv       [O] Pointer to HCRYPTPROV for the output.
//  *  pszContainer [I] Key Container Name
//  *  pszProvider  [I] Cryptographic Service Provider Name
//  *  dwProvType   [I] Crypto provider type to get a handle.
//  *  dwFlags      [I] flags for the operation
//  *
//  * RETURNS 
//  *  TRUE on success, FALSE on failure.
//  */
// BOOL WINAPI CryptAcquireContextW (HCRYPTPROV *phProv, LPCWSTR pszContainer,
// 		LPCWSTR pszProvider, DWORD dwProvType, DWORD dwFlags)
// {
// 	PCRYPTPROV pProv = NULL;
// 	HKEY key;
// 	PWSTR imagepath = NULL, keyname = NULL, provname = NULL, temp = NULL;
// 	PSTR provnameA = NULL, pszContainerA = NULL;
// 	DWORD keytype, type, len;
// 	ULONG r;
// 	static const WCHAR nameW[] = {'N','a','m','e',0};
// 	static const WCHAR typeW[] = {'T','y','p','e',0};
// 	static const WCHAR imagepathW[] = {'I','m','a','g','e',' ','P','a','t','h',0};
//
// 	TRACE("(%p, %s, %s, %d, %08x)\n", phProv, debugstr_w(pszContainer),
// 		debugstr_w(pszProvider), dwProvType, dwFlags);
//
// 	if (dwProvType < 1 || dwProvType > MAXPROVTYPES)
// 	{
// 		SetLastError(NTE_BAD_PROV_TYPE);
// 		return FALSE;
// 	}
// 	
// 	if (!phProv)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	/* Make sure the MachineGuid value exists */
// 	CRYPT_CreateMachineGuid();
//
// 	if (!pszProvider || !*pszProvider)
// 	{
// 		/* No CSP name specified so try the user default CSP first
// 		 * then try the machine default CSP
// 		 */
// 		if ( !(keyname = CRYPT_GetTypeKeyName(dwProvType, TRUE)) ) {
// 			TRACE("No provider registered for crypto provider type %d.\n", dwProvType);
// 			SetLastError(NTE_PROV_TYPE_NOT_DEF);
// 			return FALSE;
// 		}
// 		if (RegOpenKeyW(HKEY_CURRENT_USER, keyname, &key))
// 		{
// 			CRYPT_Free(keyname as _);
// 			if ( !(keyname = CRYPT_GetTypeKeyName(dwProvType, FALSE)) ) {
// 				TRACE("No type registered for crypto provider type %d.\n", dwProvType);
// 				RegCloseKey(key);
// 				SetLastError(NTE_PROV_TYPE_NOT_DEF);
// 				goto error;
// 			}
// 			if (RegOpenKeyW(HKEY_LOCAL_MACHINE, keyname, &key)) {
// 				TRACE("Did not find registry entry of crypto provider for %s.\n", debugstr_w(keyname));
// 				CRYPT_Free(keyname as _);
// 				RegCloseKey(key);
// 				SetLastError(NTE_PROV_TYPE_NOT_DEF);
// 				goto error;
// 			}
// 		}
// 		CRYPT_Free(keyname as _);
// 		r = RegQueryValueExW(key, nameW, NULL, &keytype, NULL, &len);
// 		if( r != ERROR_SUCCESS || !len || keytype != REG_SZ)
// 		{
// 			TRACE("error %d reading size of 'Name' from registry\n", r );
// 			RegCloseKey(key);
// 			SetLastError(NTE_PROV_TYPE_ENTRY_BAD);
// 			goto error;
// 		}
// 		if(!(provname = CRYPT_Alloc(len)))
// 		{
// 			RegCloseKey(key);
// 			SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 			goto error;
// 		}
// 		r = RegQueryValueExW(key, nameW, NULL, NULL, (LPBYTE)provname, &len);
// 		if( r != ERROR_SUCCESS )
// 		{
// 			TRACE("error %d reading 'Name' from registry\n", r );
// 			RegCloseKey(key);
// 			SetLastError(NTE_PROV_TYPE_ENTRY_BAD);
// 			goto error;
// 		}
// 		RegCloseKey(key);
// 	} else {
// 		if ( !(provname = CRYPT_Alloc((strlenW(pszProvider) +1)*sizeof(WCHAR))) )
// 		{
// 			SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 			goto error;
// 		}
// 		strcpyW(provname, pszProvider);
// 	}
//
// 	keyname = CRYPT_GetProvKeyName(provname);
// 	r = RegOpenKeyW(HKEY_LOCAL_MACHINE, keyname, &key);
// 	CRYPT_Free(keyname as _);
// 	if (r != ERROR_SUCCESS)
// 	{
// 		SetLastError(NTE_KEYSET_NOT_DEF);
// 		goto error;
// 	}
// 	len = sizeof(DWORD);
// 	r = RegQueryValueExW(key, typeW, NULL, NULL, (BYTE*)&type, &len);
// 	if (r != ERROR_SUCCESS)
// 	{
// 		SetLastError(NTE_PROV_TYPE_ENTRY_BAD);
// 		goto error;
// 	}
// 	if (type != dwProvType)
// 	{
// 		TRACE("Crypto provider has wrong type (%d vs expected %d).\n", type, dwProvType);
// 		SetLastError(NTE_PROV_TYPE_NO_MATCH);
// 		goto error;
// 	}
//
// 	r = RegQueryValueExW(key, imagepathW, NULL, &keytype, NULL, &len);
// 	if ( r != ERROR_SUCCESS || keytype != REG_SZ)
// 	{
// 		TRACE("error %d reading size of 'Image Path' from registry\n", r );
// 		RegCloseKey(key);
// 		SetLastError(NTE_PROV_TYPE_ENTRY_BAD);
// 		goto error;
// 	}
// 	if (!(temp = CRYPT_Alloc(len)))
// 	{
// 		RegCloseKey(key);
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		goto error;
// 	}
// 	r = RegQueryValueExW(key, imagepathW, NULL, NULL, (LPBYTE)temp, &len);
// 	if( r != ERROR_SUCCESS )
// 	{
// 		TRACE("error %d reading 'Image Path' from registry\n", r );
// 		RegCloseKey(key);
// 		SetLastError(NTE_PROV_TYPE_ENTRY_BAD);
// 		goto error;
// 	}
// 	RegCloseKey(key);
// 	len = ExpandEnvironmentStringsW(temp, NULL, 0);
// 	if ( !(imagepath = CRYPT_Alloc(len*sizeof(WCHAR))) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		goto error;
// 	}
// 	if (!ExpandEnvironmentStringsW(temp, imagepath, len))
// 	{
// 		/* ExpandEnvironmentStrings will call SetLastError */
// 		goto error;
// 	}
// 	pProv = CRYPT_LoadProvider(imagepath);
// 	if (!pProv) {
// 		/* CRYPT_LoadProvider calls SetLastError */
// 		goto error;
// 	}
// 	pProv->pVTable->dwProvType = dwProvType;
// 	if(!CRYPT_UnicodeToANSI(provname, &provnameA, 0))
// 	{
// 		/* CRYPT_UnicodeToANSI calls SetLastError */
// 		goto error;
// 	}
// 	pProv->pVTable->pszProvName = provnameA;
// 	if(!CRYPT_UnicodeToANSI(pszContainer, &pszContainerA, 0))
// 	{
// 		/* CRYPT_UnicodeToANSI calls SetLastError */
// 		goto error;
// 	}
// 	if (pProv->pFuncs->pCPAcquireContext(&pProv->hPrivate, pszContainerA, dwFlags, pProv->pVTable))
// 	{
// 		/* MSDN: When this flag is set, the value returned in phProv is undefined,
// 		 *       and thus, the CryptReleaseContext function need not be called afterwards.
// 		 *       Therefore, we must clean up everything now.
// 		 */
// 		if (dwFlags & CRYPT_DELETEKEYSET)
// 		{
// 			pProv->dwMagic = 0;
// 			FreeLibrary(pProv->hModule);
// 			CRYPT_Free(provnameA);
// 			CRYPT_Free(pProv->pVTable);
// 			CRYPT_Free(pProv->pFuncs);
// 			CRYPT_Free(pProv);
// 		} else {
// 			*phProv = (HCRYPTPROV)pProv;
// 		}
// 		CRYPT_Free(pszContainerA);
// 		CRYPT_Free(provname);
// 		CRYPT_Free(temp);
// 		CRYPT_Free(imagepath);
// 		return TRUE;
// 	}
// 	/* FALLTHROUGH TO ERROR IF FALSE - CSP internal error! */
// error:
// 	if (pProv)
// 	{
// 		pProv->dwMagic = 0;
// 		if (pProv->hModule)
// 			FreeLibrary(pProv->hModule);
// 		CRYPT_Free(pProv->pVTable);
// 		CRYPT_Free(pProv->pFuncs);
// 		CRYPT_Free(pProv);
// 	}
// 	CRYPT_Free(pszContainerA);
// 	CRYPT_Free(provnameA);
// 	CRYPT_Free(provname);
// 	CRYPT_Free(temp);
// 	CRYPT_Free(imagepath);
// 	return FALSE;
// }
//
// /******************************************************************************
//  * CryptAcquireContextA (ADVAPI32.@)
//  *
//  * See CryptAcquireContextW.
//  */
// BOOL WINAPI CryptAcquireContextA (HCRYPTPROV *phProv, LPCSTR pszContainer,
// 		LPCSTR pszProvider, DWORD dwProvType, DWORD dwFlags)
// {
// 	PWSTR pProvider = NULL, pContainer = NULL;
// 	BOOL ret = FALSE;
//
// 	TRACE("(%p, %s, %s, %d, %08x)\n", phProv, debugstr_a(pszContainer),
//               debugstr_a(pszProvider), dwProvType, dwFlags);
//
// 	if ( !CRYPT_ANSIToUnicode(pszContainer, &pContainer, -1) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
// 	if ( !CRYPT_ANSIToUnicode(pszProvider, &pProvider, -1) )
// 	{
// 		CRYPT_Free(pContainer);
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
//
// 	ret = CryptAcquireContextW(phProv, pContainer, pProvider, dwProvType, dwFlags);
//
// 	CRYPT_Free(pContainer);
// 	CRYPT_Free(pProvider);
//
// 	return ret;
// }
//
// /******************************************************************************
//  * CryptContextAddRef (ADVAPI32.@)
//  *
//  * Increases reference count of a cryptographic service provider handle
//  * by one.
//  *
//  * PARAMS
//  *  hProv       [I] Handle to the CSP whose reference is being incremented.
//  *  pdwReserved [IN] Reserved for future use and must be NULL.
//  *  dwFlags     [I] Reserved for future use and must be 0.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptContextAddRef (HCRYPTPROV hProv, DWORD *pdwReserved, DWORD dwFlags)
// {
// 	PCRYPTPROV pProv = (PCRYPTPROV)hProv;	
//
// 	TRACE("(0x%lx, %p, %08x)\n", hProv, pdwReserved, dwFlags);
//
// 	if (!pProv)
// 	{
// 		SetLastError(NTE_BAD_UID);
// 		return FALSE;
// 	}
//
// 	if (pProv->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	InterlockedIncrement(&pProv->refcount);
// 	return TRUE;
// }
//
// /******************************************************************************
//  * CryptReleaseContext (ADVAPI32.@)
//  *
//  * Releases the handle of a CSP.  Reference count is decreased.
//  *
//  * PARAMS
//  *  hProv   [I] Handle of a CSP.
//  *  dwFlags [I] Reserved for future use and must be 0.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptReleaseContext (HCRYPTPROV hProv, DWORD dwFlags)
// {
// 	PCRYPTPROV pProv = (PCRYPTPROV)hProv;
// 	BOOL ret = TRUE;
//
// 	TRACE("(0x%lx, %08x)\n", hProv, dwFlags);
//
// 	if (!pProv)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	if (pProv->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	if (InterlockedDecrement(&pProv->refcount) == 0)
// 	{
// 		ret = pProv->pFuncs->pCPReleaseContext(pProv->hPrivate, dwFlags);
// 		pProv->dwMagic = 0;
// 		FreeLibrary(pProv->hModule);
// 		CRYPT_Free(pProv->pVTable->pszProvName);
// 		CRYPT_Free(pProv->pVTable);
// 		CRYPT_Free(pProv->pFuncs);
// 		CRYPT_Free(pProv);
// 	}
// 	return ret;
// }
//
// /******************************************************************************
//  * CryptGenRandom (ADVAPI32.@)
//  *
//  * Fills a buffer with cryptographically random bytes.
//  *
//  * PARAMS
//  *  hProv    [I] Handle of a CSP.
//  *  dwLen    [I] Number of bytes to generate.
//  *  pbBuffer [I/O] Buffer to contain random bytes.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  *
//  * NOTES
//  *  pdBuffer must be at least dwLen bytes long.
//  */
// BOOL WINAPI CryptGenRandom (HCRYPTPROV hProv, DWORD dwLen, BYTE *pbBuffer)
// {
// 	PCRYPTPROV prov = (PCRYPTPROV)hProv;
//
// 	TRACE("(0x%lx, %d, %p)\n", hProv, dwLen, pbBuffer);
//
// 	if (!hProv)
// 	{
// 		SetLastError(ERROR_INVALID_HANDLE);
// 		return FALSE;
// 	}
//
// 	if (prov->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	return prov->pFuncs->pCPGenRandom(prov->hPrivate, dwLen, pbBuffer);
// }
//
// /******************************************************************************
//  * CryptCreateHash (ADVAPI32.@)
//  *
//  * Initiates the hashing of a stream of data.
//  *
//  * PARAMS
//  *  hProv   [I] Handle of a CSP.
//  *  Algid   [I] Identifies the hash algorithm to use.
//  *  hKey    [I] Key for the hash (if required).
//  *  dwFlags [I] Reserved for future use and must be 0.
//  *  phHash  [O] Address of the future handle to the new hash object.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  *
//  * NOTES
//  *  If the algorithm is a keyed hash, hKey is the key.
//  */
// BOOL WINAPI CryptCreateHash (HCRYPTPROV hProv, ALG_ID Algid, HCRYPTKEY hKey,
// 		DWORD dwFlags, HCRYPTHASH *phHash)
// {
// 	PCRYPTPROV prov = (PCRYPTPROV)hProv;
// 	PCRYPTKEY key = (PCRYPTKEY)hKey;
// 	PCRYPTHASH hash;
//
// 	TRACE("(0x%lx, 0x%x, 0x%lx, %08x, %p)\n", hProv, Algid, hKey, dwFlags, phHash);
//
// 	if (!prov || !phHash || prov->dwMagic != MAGIC_CRYPTPROV ||
// 		(key && key->dwMagic != MAGIC_CRYPTKEY))
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
// 	if (dwFlags)
// 	{
// 		SetLastError(NTE_BAD_FLAGS);
// 		return FALSE;
// 	}
// 	if ( !(hash = CRYPT_Alloc(sizeof(CRYPTHASH))) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
//
// 	hash->pProvider = prov;
// 	hash->dwMagic = MAGIC_CRYPTHASH;
// 	if (prov->pFuncs->pCPCreateHash(prov->hPrivate, Algid,
// 			key ? key->hPrivate : 0, 0, &hash->hPrivate))
//         {
//             *phHash = (HCRYPTHASH)hash;
//             return TRUE;
//         }
//
// 	/* CSP error! */
// 	hash->dwMagic = 0;
// 	CRYPT_Free(hash);
// 	*phHash = 0;
// 	return FALSE;
// }
//
// /******************************************************************************
//  * CryptDecrypt (ADVAPI32.@)
//  *
//  * Decrypts data encrypted by CryptEncrypt.
//  *
//  * PARAMS
//  *  hKey       [I] Handle to the decryption key.
//  *  hHash      [I] Handle to a hash object.
//  *  Final      [I] TRUE if this is the last section to be decrypted.
//  *  dwFlags    [I] Reserved for future use. Can be CRYPT_OAEP.
//  *  pbData     [I/O] Buffer that holds the encrypted data. Holds decrypted
//  *                   data on return
//  *  pdwDataLen [I/O] Length of pbData before and after the call.
//  *
//  *  RETURNS
//  *   Success: TRUE
//  *   Failure: FALSE
//  */
// BOOL WINAPI CryptDecrypt (HCRYPTKEY hKey, HCRYPTHASH hHash, BOOL Final,
// 		DWORD dwFlags, BYTE *pbData, DWORD *pdwDataLen)
// {
// 	PCRYPTPROV prov;
// 	PCRYPTKEY key = (PCRYPTKEY)hKey;
// 	PCRYPTHASH hash = (PCRYPTHASH)hHash;
//
// 	TRACE("(0x%lx, 0x%lx, %d, %08x, %p, %p)\n", hKey, hHash, Final, dwFlags, pbData, pdwDataLen);
//
// 	if (!key || !pbData || !pdwDataLen ||
// 		!key->pProvider || key->dwMagic != MAGIC_CRYPTKEY ||
// 		key->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = key->pProvider;
// 	return prov->pFuncs->pCPDecrypt(prov->hPrivate, key->hPrivate, hash ? hash->hPrivate : 0,
// 			Final, dwFlags, pbData, pdwDataLen);
// }
//
// /******************************************************************************
//  * CryptDeriveKey (ADVAPI32.@)
//  *
//  * Generates session keys derived from a base data value.
//  *
//  * PARAMS
//  *  hProv     [I] Handle to a CSP.
//  *  Algid     [I] Identifies the symmetric encryption algorithm to use.
//  *  hBaseData [I] Handle to a hash object.
//  *  dwFlags   [I] Type of key to generate.
//  *  phKey     [I/O] Address of the newly generated key.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptDeriveKey (HCRYPTPROV hProv, ALG_ID Algid, HCRYPTHASH hBaseData,
// 		DWORD dwFlags, HCRYPTKEY *phKey)
// {
// 	PCRYPTPROV prov = (PCRYPTPROV)hProv;
// 	PCRYPTHASH hash = (PCRYPTHASH)hBaseData;
// 	PCRYPTKEY key;
//
// 	TRACE("(0x%lx, 0x%08x, 0x%lx, 0x%08x, %p)\n", hProv, Algid, hBaseData, dwFlags, phKey);
//
// 	if (!prov || !hash)
// 	{
// 		SetLastError(ERROR_INVALID_HANDLE);
// 		return FALSE;
// 	}
// 	if (!phKey || prov->dwMagic != MAGIC_CRYPTPROV || hash->dwMagic != MAGIC_CRYPTHASH)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
// 	if ( !(key = CRYPT_Alloc(sizeof(CRYPTKEY))) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
//
// 	key->pProvider = prov;
// 	key->dwMagic = MAGIC_CRYPTKEY;
// 	if (prov->pFuncs->pCPDeriveKey(prov->hPrivate, Algid, hash->hPrivate, dwFlags, &key->hPrivate))
//         {
//             *phKey = (HCRYPTKEY)key;
//             return TRUE;
//         }
//
// 	/* CSP error! */
// 	key->dwMagic = 0;
// 	CRYPT_Free(key);
// 	*phKey = 0;
// 	return FALSE;
// }
//
// /******************************************************************************
//  * CryptDestroyHash (ADVAPI32.@)
//  *
//  * Destroys the hash object referenced by hHash.
//  *
//  * PARAMS
//  *  hHash [I] Handle of the hash object to be destroyed.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptDestroyHash (HCRYPTHASH hHash)
// {
// 	PCRYPTHASH hash = (PCRYPTHASH)hHash;
// 	PCRYPTPROV prov;
// 	BOOL ret;
//
// 	TRACE("(0x%lx)\n", hHash);
//
// 	if (!hash)
// 	{
// 		SetLastError(ERROR_INVALID_HANDLE);
// 		return FALSE;
// 	}
//
// 	if (!hash->pProvider || hash->dwMagic != MAGIC_CRYPTHASH ||
// 		hash->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = hash->pProvider;
// 	ret = prov->pFuncs->pCPDestroyHash(prov->hPrivate, hash->hPrivate);
// 	hash->dwMagic = 0;
// 	CRYPT_Free(hash);
// 	return ret;
// }
//
// /******************************************************************************
//  * CryptDestroyKey (ADVAPI32.@)
//  *
//  * Releases the handle referenced by hKey.
//  *
//  * PARAMS
//  *  hKey [I] Handle of the key to be destroyed.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptDestroyKey (HCRYPTKEY hKey)
// {
// 	PCRYPTKEY key = (PCRYPTKEY)hKey;
// 	PCRYPTPROV prov;
// 	BOOL ret;
//
// 	TRACE("(0x%lx)\n", hKey);
//
// 	if (!key)
// 	{
// 		SetLastError(ERROR_INVALID_HANDLE);
// 		return FALSE;
// 	}
//
// 	if (!key->pProvider || key->dwMagic != MAGIC_CRYPTKEY ||
// 		key->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = key->pProvider;
// 	ret = prov->pFuncs->pCPDestroyKey(prov->hPrivate, key->hPrivate);
// 	key->dwMagic = 0;
// 	CRYPT_Free(key);
// 	return ret;
// }
//
// /******************************************************************************
//  * CryptDuplicateHash (ADVAPI32.@)
//  *
//  * Duplicates a hash.
//  *
//  * PARAMS
//  *  hHash       [I] Handle to the hash to be copied.
//  *  pdwReserved [I] Reserved for future use and must be NULL.
//  *  dwFlags     [I] Reserved for future use and must be zero.
//  *  phHash      [O] Address of the handle to receive the copy.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptDuplicateHash (HCRYPTHASH hHash, DWORD *pdwReserved,
// 		DWORD dwFlags, HCRYPTHASH *phHash)
// {
// 	PCRYPTPROV prov;
// 	PCRYPTHASH orghash, newhash;
//
// 	TRACE("(0x%lx, %p, %08x, %p)\n", hHash, pdwReserved, dwFlags, phHash);
//
// 	orghash = (PCRYPTHASH)hHash;
// 	if (!orghash || pdwReserved || !phHash || !orghash->pProvider ||
// 		orghash->dwMagic != MAGIC_CRYPTHASH || orghash->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = orghash->pProvider;
// 	if (!prov->pFuncs->pCPDuplicateHash)
// 	{
// 		SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
// 		return FALSE;
// 	}
//
// 	if ( !(newhash = CRYPT_Alloc(sizeof(CRYPTHASH))) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
//
// 	newhash->pProvider = prov;
// 	newhash->dwMagic = MAGIC_CRYPTHASH;
// 	if (prov->pFuncs->pCPDuplicateHash(prov->hPrivate, orghash->hPrivate, pdwReserved, dwFlags, &newhash->hPrivate))
// 	{
// 		*phHash = (HCRYPTHASH)newhash;
// 		return TRUE;
// 	}
// 	newhash->dwMagic = 0;
// 	CRYPT_Free(newhash);
// 	return FALSE;
// }
//
// /******************************************************************************
//  * CryptDuplicateKey (ADVAPI32.@)
//  *
//  * Duplicate a key and the key's state.
//  *
//  * PARAMS
//  *  hKey        [I] Handle of the key to copy.
//  *  pdwReserved [I] Reserved for future use and must be NULL.
//  *  dwFlags     [I] Reserved for future use and must be zero.
//  *  phKey       [I] Address of the handle to the duplicated key.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptDuplicateKey (HCRYPTKEY hKey, DWORD *pdwReserved, DWORD dwFlags, HCRYPTKEY *phKey)
// {
// 	PCRYPTPROV prov;
// 	PCRYPTKEY orgkey, newkey;
//
// 	TRACE("(0x%lx, %p, %08x, %p)\n", hKey, pdwReserved, dwFlags, phKey);
//
// 	orgkey = (PCRYPTKEY)hKey;
// 	if (!orgkey || pdwReserved || !phKey || !orgkey->pProvider ||
// 		orgkey->dwMagic != MAGIC_CRYPTKEY ||
// 		orgkey->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = orgkey->pProvider;
// 	if (!prov->pFuncs->pCPDuplicateKey)
// 	{
// 		SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
// 		return FALSE;
// 	}
//
// 	if ( !(newkey = CRYPT_Alloc(sizeof(CRYPTKEY))) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
//
// 	newkey->pProvider = prov;
// 	newkey->dwMagic = MAGIC_CRYPTKEY;
// 	if (prov->pFuncs->pCPDuplicateKey(prov->hPrivate, orgkey->hPrivate, pdwReserved, dwFlags, &newkey->hPrivate))
// 	{
// 		*phKey = (HCRYPTKEY)newkey;
// 		return TRUE;
// 	}
// 	newkey->dwMagic = 0;
// 	CRYPT_Free(newkey);
// 	return FALSE;
// }
//
// /******************************************************************************
//  * CryptEncrypt (ADVAPI32.@)
//  *
//  * Encrypts data.
//  *
//  * PARAMS
//  *  hKey       [I] Handle to the encryption key.
//  *  hHash      [I] Handle to a hash object.
//  *  Final      [I] TRUE if this is the last section to encrypt.
//  *  dwFlags    [I] Can be CRYPT_OAEP.
//  *  pbData     [I/O] Data to be encrypted. Contains encrypted data after call.
//  *  pdwDataLen [I/O] Length of the data to encrypt. Contains the length of the
//  *                   encrypted data after call.
//  *  dwBufLen   [I] Length of the input pbData buffer.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  *
//  *  NOTES
//  *   If pbData is NULL, CryptEncrypt determines stores the number of bytes
//  *   required for the returned data in pdwDataLen.
//  */
// BOOL WINAPI CryptEncrypt (HCRYPTKEY hKey, HCRYPTHASH hHash, BOOL Final,
// 		DWORD dwFlags, BYTE *pbData, DWORD *pdwDataLen, DWORD dwBufLen)
// {
// 	PCRYPTPROV prov;
// 	PCRYPTKEY key = (PCRYPTKEY)hKey;
// 	PCRYPTHASH hash = (PCRYPTHASH)hHash;
//
// 	TRACE("(0x%lx, 0x%lx, %d, %08x, %p, %p, %d)\n", hKey, hHash, Final, dwFlags, pbData, pdwDataLen, dwBufLen);
//
// 	if (!key || !pdwDataLen || !key->pProvider ||
// 		key->dwMagic != MAGIC_CRYPTKEY || key->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = key->pProvider;
// 	return prov->pFuncs->pCPEncrypt(prov->hPrivate, key->hPrivate, hash ? hash->hPrivate : 0,
// 			Final, dwFlags, pbData, pdwDataLen, dwBufLen);
// }
//
// /******************************************************************************
//  * CryptEnumProvidersW (ADVAPI32.@)
//  *
//  * Returns the next available CSP.
//  *
//  * PARAMS
//  *  dwIndex     [I] Index of the next provider to be enumerated.
//  *  pdwReserved [I] Reserved for future use and must be NULL.
//  *  dwFlags     [I] Reserved for future use and must be zero.
//  *  pdwProvType [O] DWORD designating the type of the provider.
//  *  pszProvName [O] Buffer that receives data from the provider.
//  *  pcbProvName [I/O] Specifies the size of pszProvName. Contains the number
//  *                    of bytes stored in the buffer on return.
//  *
//  *  RETURNS
//  *   Success: TRUE
//  *   Failure: FALSE
//  *
//  *  NOTES
//  *   If pszProvName is NULL, CryptEnumProvidersW sets the size of the name
//  *   for memory allocation purposes.
//  */
// BOOL WINAPI CryptEnumProvidersW (DWORD dwIndex, DWORD *pdwReserved,
// 		DWORD dwFlags, DWORD *pdwProvType, LPWSTR pszProvName, DWORD *pcbProvName)
// {
// 	HKEY hKey;
// 	static const WCHAR providerW[] = {
//                 'S','o','f','t','w','a','r','e','\\',
//                 'M','i','c','r','o','s','o','f','t','\\',
//                 'C','r','y','p','t','o','g','r','a','p','h','y','\\',
//                 'D','e','f','a','u','l','t','s','\\',
//                 'P','r','o','v','i','d','e','r',0
//         };
// 	static const WCHAR typeW[] = {'T','y','p','e',0};
// 	BOOL ret;
//
// 	TRACE("(%d, %p, %d, %p, %p, %p)\n", dwIndex, pdwReserved, dwFlags,
// 			pdwProvType, pszProvName, pcbProvName);
//
// 	if (pdwReserved || !pcbProvName)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
// 	if (dwFlags)
// 	{
// 		SetLastError(NTE_BAD_FLAGS);
// 		return FALSE;
// 	}
//
// 	if (RegOpenKeyW(HKEY_LOCAL_MACHINE, providerW, &hKey))
// 	{
// 		SetLastError(NTE_FAIL);
// 		return FALSE;
// 	}
//
// 	ret = TRUE;
// 	if (!pszProvName)
// 	{
// 		DWORD numkeys;
// 		WCHAR *provNameW;
// 		
// 		RegQueryInfoKeyW(hKey, NULL, NULL, NULL, &numkeys, pcbProvName,
// 				 NULL, NULL, NULL, NULL, NULL, NULL);
// 		
// 		if (!(provNameW = CRYPT_Alloc(*pcbProvName * sizeof(WCHAR))))
// 		{
// 			SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 			RegCloseKey(hKey);
// 			return FALSE;
// 		}
//
// 		RegEnumKeyExW(hKey, dwIndex, provNameW, pcbProvName, NULL, NULL, NULL, NULL);
// 		CRYPT_Free(provNameW);
// 		(*pcbProvName)++;
// 		*pcbProvName *= sizeof(WCHAR);
//
// 		if (dwIndex >= numkeys)
// 		{
// 			SetLastError(ERROR_NO_MORE_ITEMS);
// 			ret = FALSE;
// 		}
// 	} else {
// 		DWORD size = sizeof(DWORD);
// 		DWORD result;
// 		HKEY subkey;
// 		
// 		result = RegEnumKeyW(hKey, dwIndex, pszProvName, *pcbProvName / sizeof(WCHAR));
// 		if (result)
// 		{
// 			SetLastError(result);
// 			RegCloseKey(hKey);
// 			return FALSE;
// 		}
// 		if (RegOpenKeyW(hKey, pszProvName, &subkey))
// 		{
// 			RegCloseKey(hKey);
// 			return FALSE;
// 		}
//
// 		if (RegQueryValueExW(subkey, typeW, NULL, NULL, (BYTE*)pdwProvType, &size))
// 			ret = FALSE;
//
// 		RegCloseKey(subkey);
// 	}
// 	RegCloseKey(hKey);
// 	return ret;
// }
//
// /******************************************************************************
//  * CryptEnumProvidersA (ADVAPI32.@)
//  *
//  * See CryptEnumProvidersW.
//  */
// BOOL WINAPI CryptEnumProvidersA (DWORD dwIndex, DWORD *pdwReserved,
// 		DWORD dwFlags, DWORD *pdwProvType, LPSTR pszProvName, DWORD *pcbProvName)
// {
// 	PWSTR str = NULL;
// 	DWORD bufsize;
// 	BOOL ret;
//
// 	TRACE("(%d, %p, %08x, %p, %p, %p)\n", dwIndex, pdwReserved, dwFlags,
// 			pdwProvType, pszProvName, pcbProvName);
//
// 	if(!CryptEnumProvidersW(dwIndex, pdwReserved, dwFlags, pdwProvType, NULL, &bufsize))
// 		return FALSE;
// 	if ( pszProvName && !(str = CRYPT_Alloc(bufsize)) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
// 	ret = CryptEnumProvidersW(dwIndex, pdwReserved, dwFlags, pdwProvType, str, &bufsize);
// 	if (str)
// 		CRYPT_UnicodeToANSI(str, &pszProvName, *pcbProvName);
// 	*pcbProvName = bufsize / sizeof(WCHAR);  /* FIXME: not correct */
// 	if (str)
// 	{
// 		CRYPT_Free(str);
// 		if (GetLastError() == ERROR_INSUFFICIENT_BUFFER)
// 		{
// 			SetLastError(ERROR_MORE_DATA);
// 			return FALSE;
// 		}
// 	}
// 	return ret;
// }
//
// /******************************************************************************
//  * CryptEnumProviderTypesW (ADVAPI32.@)
//  *
//  * Retrieves the next type of CSP supported.
//  *
//  * PARAMS
//  *  dwIndex     [I] Index of the next provider to be enumerated.
//  *  pdwReserved [I] Reserved for future use and must be NULL.
//  *  dwFlags     [I] Reserved for future use and must be zero.
//  *  pdwProvType [O] DWORD designating the type of the provider.
//  *  pszTypeName [O] Buffer that receives data from the provider type.
//  *  pcbTypeName [I/O] Specifies the size of pszTypeName. Contains the number
//  *                    of bytes stored in the buffer on return.
//  *
//  *  RETURNS
//  *   Success: TRUE
//  *   Failure: FALSE
//  *
//  *  NOTES
//  *   If pszTypeName is NULL, CryptEnumProviderTypesW sets the size of the name
//  *   for memory allocation purposes.
//  */
// BOOL WINAPI CryptEnumProviderTypesW (DWORD dwIndex, DWORD *pdwReserved,
// 		DWORD dwFlags, DWORD *pdwProvType, LPWSTR pszTypeName, DWORD *pcbTypeName)
// {
// 	HKEY hKey, hSubkey;
// 	DWORD keylen, numkeys, dwType;
// 	PWSTR keyname, ch;
// 	DWORD result;
// 	static const WCHAR KEYSTR[] = {
//                 'S','o','f','t','w','a','r','e','\\',
//                 'M','i','c','r','o','s','o','f','t','\\',
//                 'C','r','y','p','t','o','g','r','a','p','h','y','\\',
//                 'D','e','f','a','u','l','t','s','\\',
//                 'P','r','o','v','i','d','e','r',' ','T','y','p','e','s',0
// 	};
// 	static const WCHAR typenameW[] = {'T','y','p','e','N','a','m','e',0};
//
// 	TRACE("(%d, %p, %08x, %p, %p, %p)\n", dwIndex, pdwReserved,
// 		dwFlags, pdwProvType, pszTypeName, pcbTypeName);
//
// 	if (pdwReserved || !pdwProvType || !pcbTypeName)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
// 	if (dwFlags)
// 	{
// 		SetLastError(NTE_BAD_FLAGS);
// 		return FALSE;
// 	}
//
// 	if (RegOpenKeyW(HKEY_LOCAL_MACHINE, KEYSTR, &hKey))
// 		return FALSE;
//
// 	RegQueryInfoKeyW(hKey, NULL, NULL, NULL, &numkeys, &keylen, NULL, NULL, NULL, NULL, NULL, NULL);
// 	if (dwIndex >= numkeys)
// 	{
// 		SetLastError(ERROR_NO_MORE_ITEMS);
// 		RegCloseKey(hKey);
// 		return FALSE;
// 	}
// 	keylen++;
// 	if ( !(keyname = CRYPT_Alloc(keylen*sizeof(WCHAR))) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		RegCloseKey(hKey);
// 		return FALSE;
// 	}
// 	if ( RegEnumKeyW(hKey, dwIndex, keyname, keylen) ) {
//                 CRYPT_Free(keyname as _);
// 		RegCloseKey(hKey);
// 		return FALSE;
//         }
// 	RegOpenKeyW(hKey, keyname, &hSubkey);
// 	RegCloseKey(hKey);
//
// 	ch = keyname + strlenW(keyname);
// 	/* Convert "Type 000" to 0, etc/ */
// 	*pdwProvType = *(--ch) - '0';
// 	*pdwProvType += (*(--ch) - '0') * 10;
// 	*pdwProvType += (*(--ch) - '0') * 100;
// 	CRYPT_Free(keyname as _);
// 	
// 	result = RegQueryValueExW(hSubkey, typenameW, NULL, &dwType, (LPBYTE)pszTypeName, pcbTypeName);
// 	if (result)
// 	{
// 		SetLastError(result);
// 		RegCloseKey(hSubkey);
// 		return FALSE;
// 	}
//
// 	RegCloseKey(hSubkey);
// 	return TRUE;
// }
//
// /******************************************************************************
//  * CryptEnumProviderTypesA (ADVAPI32.@)
//  *
//  * See CryptEnumProviderTypesW.
//  */
// BOOL WINAPI CryptEnumProviderTypesA (DWORD dwIndex, DWORD *pdwReserved,
// 		DWORD dwFlags, DWORD *pdwProvType, LPSTR pszTypeName, DWORD *pcbTypeName)
// {
// 	PWSTR str = NULL;
// 	DWORD bufsize;
// 	BOOL ret;
//
// 	TRACE("(%d, %p, %08x, %p, %p, %p)\n", dwIndex, pdwReserved, dwFlags,
// 			pdwProvType, pszTypeName, pcbTypeName);
//
// 	if(!CryptEnumProviderTypesW(dwIndex, pdwReserved, dwFlags, pdwProvType, NULL, &bufsize))
// 		return FALSE;
// 	if ( pszTypeName && !(str = CRYPT_Alloc(bufsize)) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
// 	ret = CryptEnumProviderTypesW(dwIndex, pdwReserved, dwFlags, pdwProvType, str, &bufsize);
// 	if (str)
// 		CRYPT_UnicodeToANSI(str, &pszTypeName, *pcbTypeName);
// 	*pcbTypeName = bufsize / sizeof(WCHAR);
// 	if (str)
// 	{
// 		CRYPT_Free(str);
// 		if (GetLastError() == ERROR_INSUFFICIENT_BUFFER)
// 		{
// 			SetLastError(ERROR_MORE_DATA);
// 			return FALSE;
// 		}
// 	}
// 	return ret;
// }
//
// /******************************************************************************
//  * CryptExportKey (ADVAPI32.@)
//  * 
//  * Exports a cryptographic key from a CSP.
//  *
//  * PARAMS
//  *  hKey       [I] Handle to the key to export.
//  *  hExpKey    [I] Handle to a cryptographic key of the end user.
//  *  dwBlobType [I] Type of BLOB to be exported.
//  *  dwFlags    [I] CRYPT_DESTROYKEY/SSL2_FALLBACK/OAEP.
//  *  pbData     [O] Buffer to receive BLOB data.
//  *  pdwDataLen [I/O] Specifies the size of pbData.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  *
//  * NOTES
//  *  if pbData is NULL, CryptExportKey sets pdwDataLen as the size of the
//  *  buffer needed to hold the BLOB.
//  */
// BOOL WINAPI CryptExportKey (HCRYPTKEY hKey, HCRYPTKEY hExpKey, DWORD dwBlobType,
// 		DWORD dwFlags, BYTE *pbData, DWORD *pdwDataLen)
// {
// 	PCRYPTPROV prov;
// 	PCRYPTKEY key = (PCRYPTKEY)hKey, expkey = (PCRYPTKEY)hExpKey;
//
// 	TRACE("(0x%lx, 0x%lx, %d, %08x, %p, %p)\n", hKey, hExpKey, dwBlobType, dwFlags, pbData, pdwDataLen);
//
// 	if (!key || !pdwDataLen || !key->pProvider ||
// 		key->dwMagic != MAGIC_CRYPTKEY || key->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = key->pProvider;
// 	return prov->pFuncs->pCPExportKey(prov->hPrivate, key->hPrivate, expkey ? expkey->hPrivate : 0,
// 			dwBlobType, dwFlags, pbData, pdwDataLen);
// }
//
// /******************************************************************************
//  * CryptGenKey (ADVAPI32.@)
//  *
//  * Generates a random cryptographic session key or a pub/priv key pair.
//  *
//  * PARAMS
//  *  hProv   [I] Handle to a CSP.
//  *  Algid   [I] Algorithm to use to make key.
//  *  dwFlags [I] Specifies type of key to make.
//  *  phKey   [I] Address of the handle to which the new key is copied.
//  *
//  *  RETURNS
//  *   Success: TRUE
//  *   Failure: FALSE
//  */
// BOOL WINAPI CryptGenKey (HCRYPTPROV hProv, ALG_ID Algid, DWORD dwFlags, HCRYPTKEY *phKey)
// {
// 	PCRYPTPROV prov = (PCRYPTPROV)hProv;
// 	PCRYPTKEY key;
//
// 	TRACE("(0x%lx, %d, %08x, %p)\n", hProv, Algid, dwFlags, phKey);
//
// 	if (!phKey || !prov || prov->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
// 	if ( !(key = CRYPT_Alloc(sizeof(CRYPTKEY))) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
//
// 	key->pProvider = prov;
// 	key->dwMagic = MAGIC_CRYPTKEY;
// 	if (prov->pFuncs->pCPGenKey(prov->hPrivate, Algid, dwFlags, &key->hPrivate))
//         {
//             *phKey = (HCRYPTKEY)key;
//             return TRUE;
//         }
//
// 	/* CSP error! */
// 	key->dwMagic = 0;
// 	CRYPT_Free(key);
// 	return FALSE;
// }
//
// /******************************************************************************
//  * CryptGetDefaultProviderW (ADVAPI32.@)
//  *
//  * Finds the default CSP of a certain provider type.
//  *
//  * PARAMS
//  *  dwProvType  [I] Provider type to look for.
//  *  pdwReserved [I] Reserved for future use and must be NULL.
//  *  dwFlags     [I] CRYPT_MACHINE_DEFAULT/USER_DEFAULT
//  *  pszProvName [O] Name of the default CSP.
//  *  pcbProvName [I/O] Size of pszProvName
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  *
//  * NOTES
//  *  If pszProvName is NULL, pcbProvName will hold the size of the buffer for
//  *  memory allocation purposes on return.
//  */
// BOOL WINAPI CryptGetDefaultProviderW (DWORD dwProvType, DWORD *pdwReserved,
// 		DWORD dwFlags, LPWSTR pszProvName, DWORD *pcbProvName)
// {
// 	HKEY hKey;
// 	PWSTR keyname;
// 	DWORD result;
// 	static const WCHAR nameW[] = {'N','a','m','e',0};
//
// 	if (pdwReserved || !pcbProvName)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
// 	if (dwFlags & ~(CRYPT_USER_DEFAULT | CRYPT_MACHINE_DEFAULT))
// 	{
// 		SetLastError(NTE_BAD_FLAGS);
// 		return FALSE;
// 	}
// 	if (dwProvType > 999)
// 	{
// 		SetLastError(NTE_BAD_PROV_TYPE);
// 		return FALSE;
// 	}
// 	if ( !(keyname = CRYPT_GetTypeKeyName(dwProvType, dwFlags & CRYPT_USER_DEFAULT)) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
// 	if (RegOpenKeyW((dwFlags & CRYPT_USER_DEFAULT) ?  HKEY_CURRENT_USER : HKEY_LOCAL_MACHINE ,keyname, &hKey))
// 	{
// 		CRYPT_Free(keyname as _);
// 		SetLastError(NTE_PROV_TYPE_NOT_DEF);
// 		return FALSE;
// 	}
// 	CRYPT_Free(keyname as _);
// 	
// 	result = RegQueryValueExW(hKey, nameW, NULL, NULL, (LPBYTE)pszProvName, pcbProvName); 
// 	RegCloseKey(hKey);
//
// 	if (result)
// 	{
// 		if (result != ERROR_MORE_DATA)
// 			SetLastError(NTE_PROV_TYPE_ENTRY_BAD);
// 		else
// 			SetLastError(result);
// 		
// 		return FALSE;
// 	}
// 	
// 	return TRUE;
// }
//
// /******************************************************************************
//  * CryptGetDefaultProviderA (ADVAPI32.@)
//  *
//  * See CryptGetDefaultProviderW.
//  */
// BOOL WINAPI CryptGetDefaultProviderA (DWORD dwProvType, DWORD *pdwReserved,
// 		DWORD dwFlags, LPSTR pszProvName, DWORD *pcbProvName)
// {
// 	PWSTR str = NULL;
// 	DWORD bufsize;
// 	BOOL ret;
//
// 	TRACE("(%d, %p, %08x, %p, %p)\n", dwProvType, pdwReserved, dwFlags, pszProvName, pcbProvName);
//
// 	CryptGetDefaultProviderW(dwProvType, pdwReserved, dwFlags, NULL, &bufsize);
// 	if ( pszProvName && !(str = CRYPT_Alloc(bufsize)) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
// 	ret = CryptGetDefaultProviderW(dwProvType, pdwReserved, dwFlags, str, &bufsize);
// 	if (str)
// 		CRYPT_UnicodeToANSI(str, &pszProvName, *pcbProvName);
// 	*pcbProvName = bufsize / sizeof(WCHAR);
// 	if (str)
// 	{
// 		CRYPT_Free(str);
// 		if (GetLastError() == ERROR_INSUFFICIENT_BUFFER)
// 		{
// 			SetLastError(ERROR_MORE_DATA);
// 			return FALSE;
// 		}
// 	}
// 	return ret;
// }
//
// /******************************************************************************
//  * CryptGetHashParam (ADVAPI32.@)
//  *
//  * Retrieves data that controls the operations of a hash object.
//  *
//  * PARAMS
//  *  hHash      [I] Handle of the hash object to question.
//  *  dwParam    [I] Query type.
//  *  pbData     [O] Buffer that receives the value data.
//  *  pdwDataLen [I/O] Size of the pbData buffer.
//  *  dwFlags    [I] Reserved for future use and must be zero.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  *
//  * NOTES
//  *  If pbData is NULL, pdwDataLen will contain the length required.
//  */
// BOOL WINAPI CryptGetHashParam (HCRYPTHASH hHash, DWORD dwParam, BYTE *pbData,
// 		DWORD *pdwDataLen, DWORD dwFlags)
// {
// 	PCRYPTPROV prov;
// 	PCRYPTHASH hash = (PCRYPTHASH)hHash;
//
// 	TRACE("(0x%lx, %d, %p, %p, %08x)\n", hHash, dwParam, pbData, pdwDataLen, dwFlags);
//
// 	if (!hash || !pdwDataLen || !hash->pProvider ||
// 		hash->dwMagic != MAGIC_CRYPTHASH || hash->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = hash->pProvider;
// 	return prov->pFuncs->pCPGetHashParam(prov->hPrivate, hash->hPrivate, dwParam,
// 			pbData, pdwDataLen, dwFlags);
// }
//
// /******************************************************************************
//  * CryptGetKeyParam (ADVAPI32.@)
//  *
//  * Retrieves data that controls the operations of a key.
//  *
//  * PARAMS
//  *  hKey       [I] Handle to they key in question.
//  *  dwParam    [I] Specifies query type.
//  *  pbData     [O] Sequence of bytes to receive data.
//  *  pdwDataLen [I/O] Size of pbData.
//  *  dwFlags    [I] Reserved for future use and must be zero.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  *
//  * NOTES
//  *  If pbData is NULL, pdwDataLen is set to the needed length of the buffer.
//  */
// BOOL WINAPI CryptGetKeyParam (HCRYPTKEY hKey, DWORD dwParam, BYTE *pbData,
// 		DWORD *pdwDataLen, DWORD dwFlags)
// {
// 	PCRYPTPROV prov;
// 	PCRYPTKEY key = (PCRYPTKEY)hKey;
//
// 	TRACE("(0x%lx, %d, %p, %p, %08x)\n", hKey, dwParam, pbData, pdwDataLen, dwFlags);
//
// 	if (!key || !pdwDataLen || !key->pProvider ||
// 		key->dwMagic != MAGIC_CRYPTKEY || key->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = key->pProvider;
// 	return prov->pFuncs->pCPGetKeyParam(prov->hPrivate, key->hPrivate, dwParam,
// 			pbData, pdwDataLen, dwFlags);
// }
//
// /******************************************************************************
//  * CryptGetProvParam (ADVAPI32.@)
//  *
//  * Retrieves parameters that control the operations of a CSP.
//  *
//  * PARAMS
//  *  hProv      [I] Handle of the CSP in question.
//  *  dwParam    [I] Specifies query type.
//  *  pbData     [O] Buffer to receive the data.
//  *  pdwDataLen [I/O] Size of pbData.
//  *  dwFlags    [I] see MSDN Docs.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  *
//  * NOTES
//  *  If pbData is NULL, pdwDataLen is set to the needed buffer length.
//  */
// BOOL WINAPI CryptGetProvParam (HCRYPTPROV hProv, DWORD dwParam, BYTE *pbData,
// 		DWORD *pdwDataLen, DWORD dwFlags)
// {
// 	PCRYPTPROV prov = (PCRYPTPROV)hProv;
//
// 	TRACE("(0x%lx, %d, %p, %p, %08x)\n", hProv, dwParam, pbData, pdwDataLen, dwFlags);
//
// 	if (!prov || prov->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	return prov->pFuncs->pCPGetProvParam(prov->hPrivate, dwParam, pbData, pdwDataLen, dwFlags);
// }
//
// /******************************************************************************
//  * CryptGetUserKey (ADVAPI32.@)
//  *
//  * Gets a handle of one of a user's two public/private key pairs.
//  *
//  * PARAMS
//  *  hProv     [I] Handle of a CSP.
//  *  dwKeySpec [I] Private key to use.
//  *  phUserKey [O] Pointer to the handle of the retrieved keys.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptGetUserKey (HCRYPTPROV hProv, DWORD dwKeySpec, HCRYPTKEY *phUserKey)
// {
// 	PCRYPTPROV prov = (PCRYPTPROV)hProv;
// 	PCRYPTKEY key;
//
// 	TRACE("(0x%lx, %d, %p)\n", hProv, dwKeySpec, phUserKey);
//
// 	if (!prov)
// 	{
// 		SetLastError(ERROR_INVALID_HANDLE);
// 		return FALSE;
// 	}
// 	if (!phUserKey || prov->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
// 	if ( !(key = CRYPT_Alloc(sizeof(CRYPTKEY))) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
//
// 	key->pProvider = prov;
// 	key->dwMagic = MAGIC_CRYPTKEY;
// 	if (prov->pFuncs->pCPGetUserKey(prov->hPrivate, dwKeySpec, &key->hPrivate))
//         {
//             *phUserKey = (HCRYPTKEY)key;
//             return TRUE;
//         }
//
// 	/* CSP Error */
// 	key->dwMagic = 0;
// 	CRYPT_Free(key);
// 	*phUserKey = 0;
// 	return FALSE;
// }
//
// /******************************************************************************
//  * CryptHashData (ADVAPI32.@)
//  *
//  * Adds data to a hash object.
//  *
//  * PARAMS
//  *  hHash     [I] Handle of the hash object.
//  *  pbData    [I] Buffer of data to be hashed.
//  *  dwDataLen [I] Number of bytes to add.
//  *  dwFlags   [I] Can be CRYPT_USERDATA
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptHashData (HCRYPTHASH hHash, const BYTE *pbData, DWORD dwDataLen, DWORD dwFlags)
// {
// 	PCRYPTHASH hash = (PCRYPTHASH)hHash;
// 	PCRYPTPROV prov;
//
// 	TRACE("(0x%lx, %p, %d, %08x)\n", hHash, pbData, dwDataLen, dwFlags);
//
// 	if (!hash)
// 	{
// 		SetLastError(ERROR_INVALID_HANDLE);
// 		return FALSE;
// 	}
// 	if (!hash->pProvider || hash->dwMagic != MAGIC_CRYPTHASH ||
// 		hash->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = hash->pProvider;
// 	return prov->pFuncs->pCPHashData(prov->hPrivate, hash->hPrivate, pbData, dwDataLen, dwFlags);
// }
//
// /******************************************************************************
//  * CryptHashSessionKey (ADVAPI32.@)
//  *
//  * Compute the cryptographic hash of a session key object.
//  *
//  * PARAMS 
//  *  hHash   [I] Handle to the hash object.
//  *  hKey    [I] Handle to the key to be hashed.
//  *  dwFlags [I] Can be CRYPT_LITTLE_ENDIAN.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptHashSessionKey (HCRYPTHASH hHash, HCRYPTKEY hKey, DWORD dwFlags)
// {
// 	PCRYPTHASH hash = (PCRYPTHASH)hHash;
// 	PCRYPTKEY key = (PCRYPTKEY)hKey;
// 	PCRYPTPROV prov;
//
// 	TRACE("(0x%lx, 0x%lx, %08x)\n", hHash, hKey, dwFlags);
//
// 	if (!hash || !key)
// 	{
// 		SetLastError(ERROR_INVALID_HANDLE);
// 		return FALSE;
// 	}
//
// 	if (!hash->pProvider || hash->dwMagic != MAGIC_CRYPTHASH ||
// 		hash->pProvider->dwMagic != MAGIC_CRYPTPROV || key->dwMagic != MAGIC_CRYPTKEY)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = hash->pProvider;
// 	return prov->pFuncs->pCPHashSessionKey(prov->hPrivate, hash->hPrivate, key->hPrivate, dwFlags);
// }
//
// /******************************************************************************
//  * CryptImportKey (ADVAPI32.@)
//  *
//  * Transfer a cryptographic key from a key BLOB into a cryptographic service provider (CSP).
//  *
//  * PARAMS
//  *  hProv     [I] Handle of a CSP.
//  *  pbData    [I] Contains the key to be imported.
//  *  dwDataLen [I] Length of the key.
//  *  hPubKey   [I] Cryptographic key that decrypts pdData
//  *  dwFlags   [I] Used only with a public/private key pair.
//  *  phKey     [O] Imported key.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptImportKey (HCRYPTPROV hProv, const BYTE *pbData, DWORD dwDataLen,
// 		HCRYPTKEY hPubKey, DWORD dwFlags, HCRYPTKEY *phKey)
// {
// 	PCRYPTPROV prov = (PCRYPTPROV)hProv;
// 	PCRYPTKEY pubkey = (PCRYPTKEY)hPubKey, importkey;
//
// 	TRACE("(0x%lx, %p, %d, 0x%lx, %08x, %p)\n", hProv, pbData, dwDataLen, hPubKey, dwFlags, phKey);
//
// 	if (!prov || !pbData || !dwDataLen || !phKey ||
// 		prov->dwMagic != MAGIC_CRYPTPROV ||
// 		(pubkey && pubkey->dwMagic != MAGIC_CRYPTKEY))
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	if ( !(importkey = CRYPT_Alloc(sizeof(CRYPTKEY))) )
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
//
// 	importkey->pProvider = prov;
// 	importkey->dwMagic = MAGIC_CRYPTKEY;
// 	if (prov->pFuncs->pCPImportKey(prov->hPrivate, pbData, dwDataLen,
// 			pubkey ? pubkey->hPrivate : 0, dwFlags, &importkey->hPrivate))
// 	{
// 		*phKey = (HCRYPTKEY)importkey;
// 		return TRUE;
// 	}
//
// 	importkey->dwMagic = 0;
// 	CRYPT_Free(importkey);
// 	return FALSE;
// }
//
// /******************************************************************************
//  * CryptSignHashW (ADVAPI32.@)
//  *
//  * Signs data.
//  *
//  * PARAMS
//  *  hHash        [I] Handle of the hash object to be signed.
//  *  dwKeySpec    [I] Private key to use.
//  *  sDescription [I] Should be NULL.
//  *  dwFlags      [I] CRYPT_NOHASHOID/X931_FORMAT.
//  *  pbSignature  [O] Buffer of the signature data.
//  *  pdwSigLen    [I/O] Size of the pbSignature buffer.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  *
//  * NOTES
//  *  Because of security flaws sDescription should not be used and should thus be
//  *  NULL. It is supported only for compatibility with Microsoft's Cryptographic
//  *  Providers.
//  */
// BOOL WINAPI CryptSignHashW (HCRYPTHASH hHash, DWORD dwKeySpec, LPCWSTR sDescription,
// 		DWORD dwFlags, BYTE *pbSignature, DWORD *pdwSigLen)
// {
// 	PCRYPTHASH hash = (PCRYPTHASH)hHash;
// 	PCRYPTPROV prov;
//
// 	TRACE("(0x%lx, %d, %s, %08x, %p, %p)\n", 
// 		hHash, dwKeySpec, debugstr_w(sDescription), dwFlags, pbSignature, pdwSigLen);
//
// 	if (!hash)
// 	{
// 		SetLastError(ERROR_INVALID_HANDLE);
// 		return FALSE;
// 	}
// 	if (!pdwSigLen || !hash->pProvider || hash->dwMagic != MAGIC_CRYPTHASH ||
// 		 hash->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = hash->pProvider;
// 	return prov->pFuncs->pCPSignHash(prov->hPrivate, hash->hPrivate, dwKeySpec, sDescription,
// 		dwFlags, pbSignature, pdwSigLen);
// }
//
// /******************************************************************************
//  * CryptSignHashA (ADVAPI32.@)
//  *
//  * See CryptSignHashW.
//  */
// BOOL WINAPI CryptSignHashA (HCRYPTHASH hHash, DWORD dwKeySpec, LPCSTR sDescription,
// 		DWORD dwFlags, BYTE *pbSignature, DWORD *pdwSigLen)
// {
// 	LPWSTR wsDescription;
// 	BOOL result;
//
// 	TRACE("(0x%lx, %d, %s, %08x, %p, %p)\n", 
// 		hHash, dwKeySpec, debugstr_a(sDescription), dwFlags, pbSignature, pdwSigLen);
//
// 	CRYPT_ANSIToUnicode(sDescription, &wsDescription, -1);
// 	result = CryptSignHashW(hHash, dwKeySpec, wsDescription, dwFlags, pbSignature, pdwSigLen);
// 	CRYPT_Free(wsDescription);
//
// 	return result;
// }
//
// /******************************************************************************
//  * CryptSetHashParam (ADVAPI32.@)
//  *
//  * Customizes the operations of a hash object.
//  *
//  * PARAMS
//  *  hHash   [I] Handle of the hash object to set parameters.
//  *  dwParam [I] HP_HMAC_INFO/HASHVAL.
//  *  pbData  [I] Value data buffer.
//  *  dwFlags [I] Reserved for future use and must be zero.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptSetHashParam (HCRYPTHASH hHash, DWORD dwParam, const BYTE *pbData, DWORD dwFlags)
// {
// 	PCRYPTPROV prov;
// 	PCRYPTHASH hash = (PCRYPTHASH)hHash;
//
// 	TRACE("(0x%lx, %d, %p, %08x)\n", hHash, dwParam, pbData, dwFlags);
//
// 	if (!hash || !pbData || !hash->pProvider ||
// 		hash->dwMagic != MAGIC_CRYPTHASH || hash->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = hash->pProvider;
// 	return prov->pFuncs->pCPSetHashParam(prov->hPrivate, hash->hPrivate,
// 			dwParam, pbData, dwFlags);
// }
//
// /******************************************************************************
//  * CryptSetKeyParam (ADVAPI32.@)
//  *
//  * Customizes a session key's operations.
//  *
//  * PARAMS
//  *  hKey    [I] Handle to the key to set values.
//  *  dwParam [I] See MSDN Doc.
//  *  pbData  [I] Buffer of values to set.
//  *  dwFlags [I] Only used when dwParam == KP_ALGID.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptSetKeyParam (HCRYPTKEY hKey, DWORD dwParam, const BYTE *pbData, DWORD dwFlags)
// {
// 	PCRYPTPROV prov;
// 	PCRYPTKEY key = (PCRYPTKEY)hKey;
//
// 	TRACE("(0x%lx, %d, %p, %08x)\n", hKey, dwParam, pbData, dwFlags);
//
// 	if (!key || !pbData || !key->pProvider ||
// 		key->dwMagic != MAGIC_CRYPTKEY || key->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
//
// 	prov = key->pProvider;
// 	return prov->pFuncs->pCPSetKeyParam(prov->hPrivate, key->hPrivate,
// 			dwParam, pbData, dwFlags);
// }
//
// /******************************************************************************
//  * CryptSetProviderA (ADVAPI32.@)
//  *
//  * Specifies the current user's default CSP.
//  *
//  * PARAMS
//  *  pszProvName [I] Name of the new default CSP.
//  *  dwProvType  [I] Provider type of the CSP.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptSetProviderA (LPCSTR pszProvName, DWORD dwProvType)
// {
// 	TRACE("(%s, %d)\n", pszProvName, dwProvType);
// 	return CryptSetProviderExA(pszProvName, dwProvType, NULL, CRYPT_USER_DEFAULT);
// }
//
// /******************************************************************************
//  * CryptSetProviderW (ADVAPI32.@)
//  *
//  * See CryptSetProviderA.
//  */
// BOOL WINAPI CryptSetProviderW (LPCWSTR pszProvName, DWORD dwProvType)
// {
// 	TRACE("(%s, %d)\n", debugstr_w(pszProvName), dwProvType);
// 	return CryptSetProviderExW(pszProvName, dwProvType, NULL, CRYPT_USER_DEFAULT);
// }
//
// /******************************************************************************
//  * CryptSetProviderExW (ADVAPI32.@)
//  *
//  * Specifies the default CSP.
//  *
//  * PARAMS
//  *  pszProvName [I] Name of the new default CSP.
//  *  dwProvType  [I] Provider type of the CSP.
//  *  pdwReserved [I] Reserved for future use and must be NULL.
//  *  dwFlags     [I] See MSDN Doc.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptSetProviderExW (LPCWSTR pszProvName, DWORD dwProvType, DWORD *pdwReserved, DWORD dwFlags)
// {
// 	HKEY hProvKey, hTypeKey;
// 	PWSTR keyname;
// 	static const WCHAR nameW[] = {'N','a','m','e',0};
//
// 	TRACE("(%s, %d, %p, %08x)\n", debugstr_w(pszProvName), dwProvType, pdwReserved, dwFlags);
//
// 	if (!pszProvName || pdwReserved)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
// 	if (dwProvType > MAXPROVTYPES)
// 	{
// 		SetLastError(NTE_BAD_PROV_TYPE);
// 		return FALSE;
// 	}
// 	if (dwFlags & ~(CRYPT_MACHINE_DEFAULT | CRYPT_USER_DEFAULT | CRYPT_DELETE_DEFAULT)
// 			|| dwFlags == CRYPT_DELETE_DEFAULT)
// 	{
// 		SetLastError(NTE_BAD_FLAGS);
// 		return FALSE;
// 	}
//
// 	if (!(keyname = CRYPT_GetTypeKeyName(dwProvType, dwFlags & CRYPT_USER_DEFAULT)))
// 	{
// 		SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 		return FALSE;
// 	}
// 	if (RegOpenKeyW((dwFlags & CRYPT_USER_DEFAULT) ? HKEY_CURRENT_USER : HKEY_LOCAL_MACHINE,
// 		keyname, &hTypeKey))
// 	{
// 		CRYPT_Free(keyname as _);
// 		SetLastError(NTE_BAD_PROVIDER);
// 		return FALSE;
// 	}
// 	CRYPT_Free(keyname as _);
// 	
// 	if (dwFlags & CRYPT_DELETE_DEFAULT)
// 	{
// 		RegDeleteValueW(hTypeKey, nameW);
// 	}
// 	else
// 	{
// 		if (!(keyname = CRYPT_GetProvKeyName(pszProvName)))
// 		{
// 			RegCloseKey(hTypeKey);
// 			SetLastError(ERROR_NOT_ENOUGH_MEMORY);
// 			return FALSE;
// 		}
// 		if (RegOpenKeyW((dwFlags & CRYPT_USER_DEFAULT) ? HKEY_CURRENT_USER : HKEY_LOCAL_MACHINE,
// 			keyname, &hProvKey))
// 		{
// 			CRYPT_Free(keyname as _);
// 			RegCloseKey(hTypeKey);
// 			SetLastError(NTE_BAD_PROVIDER);
// 			return FALSE;
// 		}
// 		CRYPT_Free(keyname as _);
// 		
// 		if (RegSetValueExW(hTypeKey, nameW, 0, REG_SZ, (const BYTE *)pszProvName,
// 			(strlenW(pszProvName) + 1)*sizeof(WCHAR)))
// 		{
// 			RegCloseKey(hTypeKey);
// 			RegCloseKey(hProvKey);
// 			return FALSE;
// 		}
// 		
// 		RegCloseKey(hProvKey);
// 	}
// 	RegCloseKey(hTypeKey);
//
// 	return TRUE;
// }
//
// /******************************************************************************
//  * CryptSetProviderExA (ADVAPI32.@)
//  *
//  * See CryptSetProviderExW.
//  */
// BOOL WINAPI CryptSetProviderExA (LPCSTR pszProvName, DWORD dwProvType, DWORD *pdwReserved, DWORD dwFlags)
// {
// 	BOOL ret = FALSE;
// 	PWSTR str = NULL;
//
// 	TRACE("(%s, %d, %p, %08x)\n", pszProvName, dwProvType, pdwReserved, dwFlags);
//
// 	if (CRYPT_ANSIToUnicode(pszProvName, &str, -1))
// 	{
// 		ret = CryptSetProviderExW(str, dwProvType, pdwReserved, dwFlags);
// 		CRYPT_Free(str);
// 	}
// 	return ret;
// }
//
// /******************************************************************************
//  * CryptSetProvParam (ADVAPI32.@)
//  *
//  * Customizes the operations of a CSP.
//  *
//  * PARAMS
//  *  hProv   [I] Handle of a CSP.
//  *  dwParam [I] See MSDN Doc.
//  *  pbData  [I] Buffer that contains a value to set as a parameter.
//  *  dwFlags [I] if dwParam is PP_USE_HARDWARE_RNG, dwFlags must be zero.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
// BOOL WINAPI CryptSetProvParam (HCRYPTPROV hProv, DWORD dwParam, const BYTE *pbData, DWORD dwFlags)
// {
// 	PCRYPTPROV prov = (PCRYPTPROV)hProv;
//
// 	TRACE("(0x%lx, %d, %p, %08x)\n", hProv, dwParam, pbData, dwFlags);
//
// 	if (!prov)
// 	{
// 		SetLastError(ERROR_INVALID_HANDLE);
// 		return FALSE;
// 	}
// 	if (prov->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
// 	if (dwParam == PP_USE_HARDWARE_RNG)
// 	{
// 		FIXME("PP_USE_HARDWARE_RNG: What do I do with this?\n");
// 		FIXME("\tLetting the CSP decide.\n");
// 	}
// 	if (dwFlags & PP_CLIENT_HWND)
// 	{
// 		/* FIXME: Should verify the parameter */
// 		if (pbData /* && IsWindow((HWND)pbData) */)
// 		{
// 			crypt_hWindow = (HWND)(pbData);
// 			return TRUE;
// 		} else {
// 			SetLastError(ERROR_INVALID_PARAMETER);
// 			return FALSE;
// 		}
// 	}
// 	/* All other flags go to the CSP */
// 	return prov->pFuncs->pCPSetProvParam(prov->hPrivate, dwParam, pbData, dwFlags);
// }
//
// /******************************************************************************
//  * CryptVerifySignatureW (ADVAPI32.@)
//  *
//  * Verifies the signature of a hash object.
//  *
//  * PARAMS
//  *  hHash        [I] Handle of the hash object to verify.
//  *  pbSignature  [I] Signature data to verify.
//  *  dwSigLen     [I] Size of pbSignature.
//  *  hPubKey      [I] Handle to the public key to authenticate signature.
//  *  sDescription [I] Should be NULL.
//  *  dwFlags      [I] See MSDN doc.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  * 
//  * NOTES
//  *  Because of security flaws sDescription should not be used and should thus be
//  *  NULL. It is supported only for compatibility with Microsoft's Cryptographic
//  *  Providers.
//  */
// BOOL WINAPI CryptVerifySignatureW (HCRYPTHASH hHash, const BYTE *pbSignature, DWORD dwSigLen,
// 		HCRYPTKEY hPubKey, LPCWSTR sDescription, DWORD dwFlags)
// {
// 	PCRYPTHASH hash = (PCRYPTHASH)hHash;
// 	PCRYPTKEY key = (PCRYPTKEY)hPubKey;
// 	PCRYPTPROV prov;
//
// 	TRACE("(0x%lx, %p, %d, 0x%lx, %s, %08x)\n", hHash, pbSignature,
// 			dwSigLen, hPubKey, debugstr_w(sDescription), dwFlags);
//
// 	if (!hash || !key || key->dwMagic != MAGIC_CRYPTKEY || hash->dwMagic != MAGIC_CRYPTHASH ||
// 	    !hash->pProvider || hash->pProvider->dwMagic != MAGIC_CRYPTPROV ||
// 	    !key->pProvider || key->pProvider->dwMagic != MAGIC_CRYPTPROV)
// 	{
// 		SetLastError(ERROR_INVALID_PARAMETER);
// 		return FALSE;
// 	}
// 		
// 	prov = hash->pProvider;
// 	return prov->pFuncs->pCPVerifySignature(prov->hPrivate, hash->hPrivate, pbSignature, dwSigLen,
// 		key->hPrivate, sDescription, dwFlags);
// }
//
// /******************************************************************************
//  * CryptVerifySignatureA (ADVAPI32.@)
//  *
//  * See CryptVerifySignatureW.
//  */
// BOOL WINAPI CryptVerifySignatureA (HCRYPTHASH hHash, const BYTE *pbSignature, DWORD dwSigLen,
// 		HCRYPTKEY hPubKey, LPCSTR sDescription, DWORD dwFlags)
// {
// 	LPWSTR wsDescription;
// 	BOOL result;
//
// 	TRACE("(0x%lx, %p, %d, 0x%lx, %s, %08x)\n", hHash, pbSignature,
// 			dwSigLen, hPubKey, debugstr_a(sDescription), dwFlags);
//
// 	CRYPT_ANSIToUnicode(sDescription, &wsDescription, -1);
// 	result = CryptVerifySignatureW(hHash, pbSignature, dwSigLen, hPubKey, wsDescription, dwFlags);
// 	CRYPT_Free(wsDescription);
//
// 	return result;
// }
//
// /******************************************************************************
//  * OpenEncryptedFileRawA   (ADVAPI32.@)
//  *
//  * See OpenEncryptedFileRawW
//  */
// DWORD WINAPI OpenEncryptedFileRawA(LPCSTR filename, ULONG flags, PVOID *context)
// {
//     FIXME("(%s, %x, %p): stub\n", debugstr_a(filename), flags, context);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// /******************************************************************************
//  * OpenEncryptedFileRawW   (ADVAPI32.@)
//  *
//  * Opens an EFS encrypted file for backup/restore
//  *
//  * PARAMS
//  *  filename   [I] Filename to operate on
//  *  flags     [I] Operation to perform
//  *  context    [I] Handle to the context (out)
//  * RETURNS
//  *  Success: ERROR_SUCCESS
//  *  Failure: NTSTATUS error code
//  */
// DWORD WINAPI OpenEncryptedFileRawW(LPCWSTR filename, ULONG flags, PVOID *context)
// {
//     FIXME("(%s, %x, %p): stub\n", debugstr_w(filename), flags, context);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// /******************************************************************************
//  * ReadEncryptedFileRaw   (ADVAPI32.@)
//  *
//  * Export encrypted files
//  *
//  * PARAMS
//  *  export   [I] pointer to the export callback function
//  *  callback     [I] pointer to the application defined context
//  *  context    [I] pointer to the system context
//  * RETURNS
//  *  Success: ERROR_SUCCESS
//  *  Failure: NTSTATUS error code
//  */
// DWORD WINAPI ReadEncryptedFileRaw(PFE_EXPORT_FUNC export, PVOID callback, PVOID context)
// {
//     FIXME("(%p, %p, %p): stub\n", export, callback, context);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// #ifndef __REACTOS__
// /******************************************************************************
//  * SystemFunction030   (ADVAPI32.@)
//  *
//  * Tests if two blocks of 16 bytes are equal
//  *
//  * PARAMS
//  *  b1,b2   [I] block of 16 bytes
//  *
//  * RETURNS
//  *  TRUE  if blocks are the same
//  *  FALSE if blocks are different
//  */
// BOOL WINAPI SystemFunction030(LPCVOID b1, LPCVOID b2)
// {
//     return !memcmp(b1, b2, 0x10);
// }
//
// /******************************************************************************
//  * SystemFunction035   (ADVAPI32.@)
//  *
//  * Described here:
// http://disc.server.com/discussion.cgi?disc=148775;article=942;title=Coding%2FASM%2FSystem
//  *
//  * NOTES
//  *  Stub, always return TRUE.
//  */
// BOOL WINAPI SystemFunction035(LPCSTR lpszDllFilePath)
// {
//     FIXME("%s: stub\n", debugstr_a(lpszDllFilePath));
//     return TRUE;
// }
//
// /******************************************************************************
//  * SystemFunction036   (ADVAPI32.@)
//  *
//  * MSDN documents this function as RtlGenRandom and declares it in ntsecapi.h
//  *
//  * PARAMS
//  *  pbBuffer [O] Pointer to memory to receive random bytes.
//  *  dwLen   [I] Number of random bytes to fetch.
//  *
//  * RETURNS
//  *  Success: TRUE
//  *  Failure: FALSE
//  */
//
// BOOLEAN WINAPI SystemFunction036(PVOID pbBuffer, ULONG dwLen)
// {
//     int dev_random;
//
//     dev_random = open("/dev/urandom", O_RDONLY);
//     if (dev_random != -1)
//     {
//         if (!IsBadWritePtr( pbBuffer, dwLen ) &&
//             read(dev_random, pbBuffer, dwLen) == (ssize_t)dwLen)
//         {
//             close(dev_random);
//             return TRUE;
//         }
//         close(dev_random);
//     }
//     else
//         FIXME("couldn't open /dev/urandom\n");
//     SetLastError(NTE_FAIL);
//     return FALSE;
// }    
//     
// /*
//    These functions have nearly identical prototypes to CryptProtectMemory and CryptUnprotectMemory,
//    in crypt32.dll.
//  */
//
// /******************************************************************************
//  * SystemFunction040   (ADVAPI32.@)
//  *
//  * MSDN documents this function as RtlEncryptMemory and declares it in ntsecapi.h.
//  *
//  * PARAMS
//  *  memory [I/O] Pointer to memory to encrypt.
//  *  length [I] Length of region to encrypt in bytes.
//  *  flags  [I] Control whether other processes are able to decrypt the memory.
//  *    RTL_ENCRYPT_OPTION_SAME_PROCESS 
//  *    RTL_ENCRYPT_OPTION_CROSS_PROCESS 
//  *    RTL_ENCRYPT_OPTION_SAME_LOGON
//  *    
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: NTSTATUS error code
//  *
//  * NOTES
//  *  length must be a multiple of RTL_ENCRYPT_MEMORY_SIZE.
//  *  If flags are specified when encrypting, the same flag value must be given
//  *  when decrypting the memory.
//  */
// NTSTATUS WINAPI SystemFunction040(PVOID memory, ULONG length, ULONG flags)
// {
// 	FIXME("(%p, %x, %x): stub [RtlEncryptMemory]\n", memory, length, flags);
// 	return STATUS_SUCCESS;
// }
//
// /******************************************************************************
//  * SystemFunction041  (ADVAPI32.@)
//  *
//  * MSDN documents this function as RtlDecryptMemory and declares it in ntsecapi.h.
//  *
//  * PARAMS
//  *  memory [I/O] Pointer to memory to decrypt.
//  *  length [I] Length of region to decrypt in bytes.
//  *  flags  [I] Control whether other processes are able to decrypt the memory.
//  *    RTL_ENCRYPT_OPTION_SAME_PROCESS
//  *    RTL_ENCRYPT_OPTION_CROSS_PROCESS
//  *    RTL_ENCRYPT_OPTION_SAME_LOGON
//  *
//  * RETURNS
//  *  Success: STATUS_SUCCESS
//  *  Failure: NTSTATUS error code
//  *
//  * NOTES
//  *  length must be a multiple of RTL_ENCRYPT_MEMORY_SIZE.
//  *  If flags are specified when encrypting, the same flag value must be given
//  *  when decrypting the memory.
//  */
// NTSTATUS WINAPI SystemFunction041(PVOID memory, ULONG length, ULONG flags)
// {
// 	FIXME("(%p, %x, %x): stub [RtlDecryptMemory]\n", memory, length, flags);
// 	return STATUS_SUCCESS;
// }
// #endif /* !__REACTOS __ */
//
// /******************************************************************************
//  * WriteEncryptedFileRaw   (ADVAPI32.@)
//  *
//  * Import encrypted files
//  *
//  * PARAMS
//  *  import   [I] pointer to the import callback function
//  *  callback     [I] pointer to the application defined context
//  *  context    [I] pointer to the system context
//  * RETURNS
//  *  Success: ERROR_SUCCESS
//  *  Failure: NTSTATUS error code
//  */
// DWORD WINAPI WriteEncryptedFileRaw(PFE_IMPORT_FUNC import, PVOID callback, PVOID context)
// {
//     FIXME("(%p, %p, %p): stub\n", import, callback, context);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
