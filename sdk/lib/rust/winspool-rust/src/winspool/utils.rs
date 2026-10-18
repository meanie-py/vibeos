// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_D30E8899E227: [u16; 12] = [
    0x0073u16, 0x0068, 0x0065, 0x006C, 0x006C, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C,
    0x006C, 0x0000,
];

use core::ffi::c_void;

extern "C" {
    fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, fmt: *const ::core::ffi::c_char, ... );
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut hShell32: HMODULE = unsafe { -(1 as ::core::ffi::c_int) as HMODULE };

// MAX_PATH expanded to 260
pub const MAX_GETPRINTER_SIZE: i32 = 4096 - 260;

pub type PPfpSHChangeNotify =
    Option<unsafe extern "system" fn(LONG, UINT, LPCVOID, LPCVOID)>;
#[no_mangle]
pub unsafe extern "C" fn UnicodeToAnsiInPlace(pwszField: PWSTR) -> DWORD {
    let mut pszTemp: PSTR;
    let mut cch: DWORD;

    let pszField: PSTR = pwszField as PSTR;

    if pwszField.is_null() {
        return ERROR_SUCCESS as DWORD;
    }

    cch = wcslen(pwszField as *const u16) as DWORD;
    if cch == 0 as DWORD {
        return ERROR_SUCCESS as DWORD;
    }

    pszTemp = HeapAlloc(
        hProcessHeap,
        0 as DWORD,
        ((cch + 1 as DWORD) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<CHAR>() as SIZE_T),
    ) as PSTR;
    if pszTemp.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY as DWORD;
    }

    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        pwszField as LPCWSTR,
        -(1 as i32),
        pszTemp as LPSTR,
        (cch + 1 as DWORD) as i32,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    StringCchCopyA(
        pszField as STRSAFE_LPSTR,
        (cch + 1 as DWORD) as usize,
        pszTemp as STRSAFE_LPCSTR,
    );

    HeapFree(
        hProcessHeap,
        0 as DWORD,
        pszTemp as LPVOID,
    );

    return ERROR_SUCCESS as DWORD;
}

unsafe extern "C" fn multi_sz_lenW(str: *const WCHAR) -> i32 {
    let mut ptr: *const WCHAR = str;
    if str.is_null() {
        return 0;
    }
    loop {
        ptr = ptr.offset(
            (lstrlenW(ptr) + 1) as isize
        );
        if !(*ptr != 0) {
            break;
        }
    }
    return (ptr.offset_from(str) as i64 + 1 as i64) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn UnicodeToAnsiZZInPlace(pwszzField: PWSTR) -> DWORD {
    let mut pszTemp: PSTR;
    let mut len: INT;
    let mut lenW: INT;
    let pszField: PSTR = pwszzField as PSTR;

    lenW = multi_sz_lenW(pwszzField as *const WCHAR) as INT;
    if lenW == 0 {
        return ERROR_SUCCESS as DWORD;
    }

    len = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        pwszzField as LPCWSTR,
        lenW as core::ffi::c_int,
        core::ptr::null_mut::<CHAR>(),
        0 as core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) as INT;

    pszTemp = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PSTR;

    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        pwszzField as LPCWSTR,
        lenW as core::ffi::c_int,
        pszTemp as LPSTR,
        len as core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    );

    StringCchCopyA(
        pszField as STRSAFE_LPSTR,
        len as size_t,
        pszTemp as STRSAFE_LPCSTR,
    );

    HeapFree(hProcessHeap, 0 as DWORD, pszTemp as PVOID);

    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn IntProtectHandle(hSpooler: HANDLE, Close: BOOL) -> LONG {
    let mut Bad: BOOL = TRUE;
    let mut Ret: LONG;
    let mut pHandle: PSPOOLER_HANDLE;

    EnterCriticalSection(core::ptr::addr_of_mut!(rtlCritSec));

    // _SEH2_TRY / _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {}: a garbage handle that faults
    // on the Sig read is just Bad. DUMMY_PSEH compiled the guard out of the C; rpc_try
    // (ntdll's RtlSehTryExcept, which catches everything as the C's filter does) is real.
    pHandle = hSpooler as PSPOOLER_HANDLE;
    if !pHandle.is_null()
        && ntdll_sys::seh::rpc_try(|| core::ptr::read_volatile(core::ptr::addr_of!((*pHandle).Sig)))
            == Ok(SPOOLER_HANDLE_SIG as DWORD_PTR)
    {
        Bad = FALSE; // Not bad.
    }

    Ret = Bad as LONG; // Set return Level to 1 if we are BAD.

    if Bad != 0 {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else {
        if Close != 0 {
            if (*pHandle).bShared() != 0 || (*pHandle).cCount != 0 {
                (*pHandle).set_bShared(TRUE);
                Ret = 2; // Return a high level and we are shared.
            } else {
                (*pHandle).set_bClosed(TRUE);
            }
        }
    }

    if Ret == 0 { // Need to be Level 0.
        (*pHandle).cCount = (*pHandle).cCount.wrapping_add(1);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(rtlCritSec));

    // Return Level:
    // 2 : Close and/or shared
    // 1 : Failed Handle
    // 0 : In use.
    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn IntUnprotectHandle(hSpooler: HANDLE) -> BOOL {
    let mut Ret: BOOL = FALSE;
    let mut pHandle: PSPOOLER_HANDLE = hSpooler as PSPOOLER_HANDLE;
    EnterCriticalSection(core::ptr::addr_of_mut!(rtlCritSec));
    if (*pHandle).bShared() != 0 && {
        (*pHandle).cCount = (*pHandle).cCount.wrapping_sub(1);
        (*pHandle).cCount == 0
    } {
        (*pHandle).set_bClosed(TRUE);
        (*pHandle).set_bShared(FALSE);
        Ret = TRUE;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(rtlCritSec));

    if Ret != 0 {
        // ClosePrinterWorker( pHandle );
    }
    return Ret;
}
#[no_mangle]
pub unsafe extern "system" fn AllocSplStr(pwszInput: PCWSTR) -> PWSTR {
    let mut cbInput: DWORD;
    let mut pwszOutput: PWSTR;

    // Sanity check
    if pwszInput.is_null() {
        return core::ptr::null_mut();
    }

    // Get the length of the input string.
    cbInput = ((wcslen(pwszInput) as DWORD) + (1 as DWORD))
        * (core::mem::size_of::<WCHAR>() as DWORD);

    // Allocate it. We don't use DllAllocSplMem here, because it unnecessarily zeroes the memory.
    pwszOutput = HeapAlloc(hProcessHeap, 0 as DWORD, cbInput as SIZE_T) as PWSTR;
    if pwszOutput.is_null() {
        return core::ptr::null_mut();
    }

    // Copy the string and return it.
    CopyMemory!(pwszOutput as *mut core::ffi::c_void, pwszInput as *const core::ffi::c_void, cbInput as usize);
    return pwszOutput;
}

#[no_mangle]
pub unsafe extern "system" fn DllAllocSplMem(mut dwBytes: DWORD) -> PVOID {
    return HeapAlloc(hProcessHeap, HEAP_ZERO_MEMORY as DWORD, dwBytes as SIZE_T);
}

#[no_mangle]
pub unsafe extern "system" fn DllFreeSplMem(pMem: PVOID) -> BOOL {
    if pMem.is_null() {
        return TRUE;
    }
    return HeapFree(hProcessHeap, 0, pMem);
}

#[no_mangle]
pub unsafe extern "system" fn DllFreeSplStr(pwszString: PWSTR) -> BOOL {
    if !pwszString.is_null() {
        return HeapFree(hProcessHeap, 0 as DWORD, pwszString as PVOID);
    }
    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn get_sd(sd: *mut SECURITY_DESCRIPTOR, size: *mut DWORD) -> *mut SECURITY_DESCRIPTOR {
    let mut sid_group: PSID = core::ptr::null_mut();
    let mut sid_owner: PSID = core::ptr::null_mut();
    let mut sacl: *mut ACL = core::ptr::null_mut();
    let mut dacl: *mut ACL = core::ptr::null_mut();
    let mut bSet: BOOL = FALSE;
    let mut bSetd: BOOL = FALSE;
    let mut bSets: BOOL = FALSE;
    let mut absolute_sd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut retsd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();

    if IsValidSecurityDescriptor(sd as PSECURITY_DESCRIPTOR) == 0 {
        return core::ptr::null_mut();
    }

    InitializeSecurityDescriptor(
        core::ptr::addr_of_mut!(absolute_sd) as PSECURITY_DESCRIPTOR,
        SECURITY_DESCRIPTOR_REVISION,
    );

    if GetSecurityDescriptorOwner(
        sd as PSECURITY_DESCRIPTOR,
        core::ptr::addr_of_mut!(sid_owner),
        core::ptr::addr_of_mut!(bSet),
    ) == 0
    {
        return core::ptr::null_mut();
    }

    SetSecurityDescriptorOwner(
        core::ptr::addr_of_mut!(absolute_sd) as PSECURITY_DESCRIPTOR,
        sid_owner,
        bSet,
    );

    if GetSecurityDescriptorGroup(
        sd as PSECURITY_DESCRIPTOR,
        core::ptr::addr_of_mut!(sid_group),
        core::ptr::addr_of_mut!(bSet),
    ) == 0
    {
        return core::ptr::null_mut();
    }

    SetSecurityDescriptorGroup(
        core::ptr::addr_of_mut!(absolute_sd) as PSECURITY_DESCRIPTOR,
        sid_group,
        bSet,
    );

    if GetSecurityDescriptorDacl(
        sd as PSECURITY_DESCRIPTOR,
        core::ptr::addr_of_mut!(bSetd),
        core::ptr::addr_of_mut!(dacl),
        core::ptr::addr_of_mut!(bSet),
    ) == 0
    {
        return core::ptr::null_mut();
    }

    SetSecurityDescriptorDacl(
        core::ptr::addr_of_mut!(absolute_sd) as PSECURITY_DESCRIPTOR,
        bSetd,
        dacl,
        bSet,
    );

    if GetSecurityDescriptorSacl(
        sd as PSECURITY_DESCRIPTOR,
        core::ptr::addr_of_mut!(bSets),
        core::ptr::addr_of_mut!(sacl),
        core::ptr::addr_of_mut!(bSet),
    ) == 0
    {
        return core::ptr::null_mut();
    }

    SetSecurityDescriptorSacl(
        core::ptr::addr_of_mut!(absolute_sd) as PSECURITY_DESCRIPTOR,
        bSets,
        sacl,
        bSet,
    );

    *size = GetSecurityDescriptorLength(core::ptr::addr_of_mut!(absolute_sd) as PSECURITY_DESCRIPTOR);

    retsd = HeapAlloc(
        GetProcessHeap(),
        0,
        *size as SIZE_T,
    ) as PSECURITY_DESCRIPTOR;

    if !retsd.is_null() {
        if MakeSelfRelativeSD(
            core::ptr::addr_of_mut!(absolute_sd) as PSECURITY_DESCRIPTOR,
            retsd,
            size as PDWORD,
        ) == 0
        {
            HeapFree(
                GetProcessHeap(),
                0,
                retsd as PVOID,
            );
            retsd = core::ptr::null_mut();
        }
    }

    return retsd as *mut _SECURITY_DESCRIPTOR;
}
#[no_mangle]
pub unsafe extern "C" fn UpdateTrayIcon(hPrinter: HANDLE, JobId: DWORD) {
    let mut pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;
    let mut spji: SHCNF_PRINTJOB_INFO = core::mem::zeroed();
    let mut pi1w: [PRINTER_INFO_1W; MAX_GETPRINTER_SIZE as usize] = core::mem::zeroed();
    let mut cbNeeded: DWORD = 0;
    let mut fpFunction: PPfpSHChangeNotify = None;

    (*pHandle).set_bTrayIcon(TRUE);

    spji.JobId = JobId;

    if GetPrinterW(
        hPrinter,
        1 as DWORD,
        &raw mut pi1w as PBYTE,
        MAX_GETPRINTER_SIZE as DWORD,
        &raw mut cbNeeded,
    ) == 0
    {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/printing/base/winspool/utils.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"UpdateTrayIcon\0" as *const u8 as *const ::core::ffi::c_char,
                358 as ::core::ffi::c_int,
                b"UpdateTrayIcon : GetPrinterW cbNeeded %d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return;
    }

    if hShell32 == (-(1 as isize)) as HMODULE {
        hShell32 = LoadLibraryW(RUSTCONV_WSTR_D30E8899E227.as_ptr()) as HMODULE;
    }

    if !hShell32.is_null() {
        fpFunction = ::core::mem::transmute::<FARPROC, PPfpSHChangeNotify>(
            GetProcAddress(
                hShell32 as HINSTANCE,
                b"SHChangeNotify\0" as *const u8 as LPCSTR,
            ),
        );

        if let Some(fpFunction) = fpFunction {
            fpFunction(
                SHCNE_CREATE as LONG,
                (SHCNF_FLUSHNOWAIT | SHCNF_FLUSH | SHCNF_PRINTJOBW) as UINT,
                (*(&raw mut pi1w as *mut PRINTER_INFO_1W)).pName as LPCVOID,
                &raw mut spji as LPCVOID,
            );
        }
    } else if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"win32ss/printing/base/winspool/utils.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"UpdateTrayIcon\0" as *const u8 as *const ::core::ffi::c_char,
            378 as ::core::ffi::c_int,
            b"UpdateTrayIcon : No Shell32!\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    }
}
