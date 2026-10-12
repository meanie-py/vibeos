use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type PROCESSINFOCLASS = i32;
pub const ProcessBasicInformation: PROCESSINFOCLASS = 0;
pub const ProcessDebugPort: PROCESSINFOCLASS = 7;
// ProcessWow64Information = 26: already provided by the import ntdll_sys::nt::ProcessWow64Information
pub const ProcessImageFileName: PROCESSINFOCLASS = 27;
pub const ProcessBreakOnTermination: PROCESSINFOCLASS = 29;

pub type NTSTATUS = i32;

pub type FN_NtQueryInformationProcess = unsafe extern "system" fn(HANDLE, PROCESSINFOCLASS, PVOID, ULONG, PULONG) -> NTSTATUS;

#[no_mangle]
pub unsafe extern "C" fn cxx_operator_new(size: usize, _: &CicNoThrow) -> *mut core::ffi::c_void {
    return cicMemAllocClear(size as SIZE_T);
}
#[no_mangle]
pub unsafe extern "C" fn cxx_operator_new_array(size: usize, _nothrow: &CicNoThrow) -> *mut core::ffi::c_void {
    return cicMemAllocClear(size as SIZE_T);
}
#[no_mangle]
pub unsafe extern "C" fn cxx_operator_delete__0(ptr: *mut core::ffi::c_void) {
    cicMemFree(ptr);
}
#[no_mangle]
pub unsafe extern "C" fn cxx_operator_delete_array__0(ptr: *mut core::ffi::c_void) {
    cicMemFree(ptr);
}
#[no_mangle]
pub unsafe extern "C" fn cxx_operator_delete__1(ptr: *mut core::ffi::c_void, size: usize) {
    cicMemFree(ptr);
}
#[no_mangle]
pub unsafe extern "C" fn cxx_operator_delete_array__1(ptr: *mut core::ffi::c_void, size: usize) {
    let _ = size;
    cicMemFree(ptr);
}
#[no_mangle]
pub unsafe extern "C" fn cicMemReCalloc(mem: LPVOID, num: SIZE_T, size: SIZE_T) -> LPVOID {
    let old_size: SIZE_T;
    let new_size: SIZE_T = num.wrapping_mul(size);
    let ret: LPVOID;

    if mem.is_null() {
        return cicMemAllocClear(new_size);
    }

    old_size = LocalSize(mem);
    ret = cicMemReAlloc(mem, (new_size) as _);
    if ret.is_null() {
        return core::ptr::null_mut();
    }

    if new_size > old_size {
        ZeroMemory!((ret as PBYTE).add((old_size) as size_t), new_size - old_size);
    }

    ret
}
#[no_mangle]
pub unsafe extern "C" fn cicIsWow64() -> i32 {
    static mut s_fnNtQueryInformationProcessPtr: *const core::ffi::c_void = core::ptr::null();
    let mut Value: ULONG_PTR;

    if s_fnNtQueryInformationProcessPtr.is_null() {
        let ntdll_name: [u16; 10] = ['n' as u16, 't' as u16, 'd' as u16, 'l' as u16, 'l' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];
        let hNTDLL: HMODULE = cicGetSystemModuleHandle((ntdll_name.as_ptr()), (FALSE) as BOOL);
        if hNTDLL.is_null() {
            return (FALSE) as i32;
        }

        s_fnNtQueryInformationProcessPtr = core::mem::transmute::<_, *const core::ffi::c_void>(
            GetProcAddress(hNTDLL, b"NtQueryInformationProcess\0".as_ptr() as *const i8),
        );
        if s_fnNtQueryInformationProcessPtr.is_null() {
            return (FALSE) as i32;
        }
    }

    let s_fnNtQueryInformationProcess: FN_NtQueryInformationProcess =
        core::mem::transmute::<*const core::ffi::c_void, FN_NtQueryInformationProcess>(
            s_fnNtQueryInformationProcessPtr,
        );

    Value = 0;
    s_fnNtQueryInformationProcess(
        GetCurrentProcess(),
        ProcessWow64Information,
        core::ptr::addr_of_mut!(Value) as LPVOID,
        core::mem::size_of::<ULONG_PTR>() as ULONG,
        core::ptr::null_mut(),
    );
    return (Value != 0) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn cicGetOSInfo(puACP: LPUINT, pdwOSInfo: LPDWORD) {
    *pdwOSInfo = 0;

    /* Check OS version info */
    let mut VerInfo: OSVERSIONINFO = core::mem::zeroed();
    VerInfo.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFO>() as u32;
    GetVersionEx(core::ptr::addr_of_mut!(VerInfo));
    if VerInfo.dwPlatformId == DLLVER_PLATFORM_NT {
        *pdwOSInfo |= CIC_OSINFO_NT;
        if VerInfo.dwMajorVersion >= 5 {
            *pdwOSInfo |= CIC_OSINFO_2KPLUS;
            if VerInfo.dwMinorVersion > 0 {
                *pdwOSInfo |= CIC_OSINFO_XPPLUS;
            }
        }
    } else {
        if VerInfo.dwMinorVersion >= 10 {
            *pdwOSInfo |= CIC_OSINFO_98PLUS;
        } else {
            *pdwOSInfo |= CIC_OSINFO_95;
        }
    }

    /* Check codepage */
    *puACP = GetACP();
    match *puACP {
        v if v == 932 /* Japanese (Japan) */
            || v == 936 /* Chinese (PRC, Singapore) */
            || v == 949 /* Korean (Korea) */
            || v == 950 /* Chinese (Taiwan, Hong Kong) */ =>
        {
            *pdwOSInfo |= CIC_OSINFO_CJK;
        }
        _ => {}
    }

    if GetSystemMetrics((SM_IMMENABLED as i32)) != 0 {
        *pdwOSInfo |= CIC_OSINFO_IMM;
    }

    if GetSystemMetrics((SM_DBCSENABLED as i32)) != 0 {
        *pdwOSInfo |= CIC_OSINFO_DBCS;
    }
}
#[no_mangle]
pub unsafe extern "C" fn cicGetSystemModuleHandle(pszFileName: LPCTSTR, bSysWinDir: BOOL) -> HINSTANCE {
    let mut ModPath = CicSystemModulePath__make_0();
    if ModPath.Init(pszFileName, bSysWinDir) == 0 {
        return core::ptr::null_mut();
    }
    return GetModuleHandle(ModPath.m_szPath.as_ptr() as LPCTSTR);
}
#[no_mangle]
pub unsafe extern "C" fn cicLoadSystemLibrary(pszFileName: LPCTSTR, bSysWinDir: BOOL) -> HINSTANCE {
    let mut ModPath: CicSystemModulePath = CicSystemModulePath__make_0();
    if !(ModPath.Init(pszFileName, bSysWinDir) != 0) {
        return core::ptr::null_mut();
    }
    return LoadLibrary(core::ptr::addr_of!(ModPath.m_szPath) as *const TCHAR);
}

// FN_CoCreateInstance is already the nullable fn pointer (`Option<fn>`): no second Option.
unsafe extern "C" fn _cicGetSetUserCoCreateInstance(fnUserCoCreateInstance: FN_CoCreateInstance) -> FN_CoCreateInstance {
    static mut s_fn: FN_CoCreateInstance = None;
    if fnUserCoCreateInstance.is_some() {
        s_fn = fnUserCoCreateInstance;
    }
    return s_fn;
}
#[no_mangle]
pub unsafe extern "C" fn cicRealCoCreateInstance(rclsid: REFCLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: REFIID, ppv: *mut LPVOID) -> HRESULT {
    static mut s_hOle32: HINSTANCE = core::ptr::null_mut();
    static mut s_fnCoCreateInstance: FN_CoCreateInstance = None;
    if s_fnCoCreateInstance.is_none() {
        if s_hOle32.is_null() {
            let ole32_name: [u16; 10] = ['o' as u16, 'l' as u16, 'e' as u16, '3' as u16, '2' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];
            s_hOle32 = cicLoadSystemLibrary((ole32_name.as_ptr()), (FALSE) as BOOL);
        }
        s_fnCoCreateInstance = core::mem::transmute(GetProcAddress(s_hOle32, b"CoCreateInstance\0".as_ptr() as *const i8));
        if s_fnCoCreateInstance.is_none() {
            return E_NOTIMPL;
        }
    }

    return (s_fnCoCreateInstance.unwrap_unchecked())(rclsid, pUnkOuter, dwClsContext, iid, ppv);
}
#[no_mangle]
pub unsafe extern "C" fn cicCoCreateInstance(
    rclsid: REFCLSID,
    pUnkOuter: LPUNKNOWN,
    dwClsContext: DWORD,
    iid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    // NOTE: It looks like Cicero wants to hook CoCreateInstance
    let fnUserCoCreateInstance: FN_CoCreateInstance = _cicGetSetUserCoCreateInstance(None);
    if fnUserCoCreateInstance.is_some() {
        return (fnUserCoCreateInstance.unwrap_unchecked())(rclsid, pUnkOuter, dwClsContext, iid, ppv);
    }

    return cicRealCoCreateInstance(rclsid, pUnkOuter, dwClsContext, iid, ppv);
}
#[no_mangle]
pub unsafe extern "C" fn TFInitLib(fnCoCreateInstance: FN_CoCreateInstance) -> BOOL {
    if fnCoCreateInstance.is_some() {
        _cicGetSetUserCoCreateInstance(fnCoCreateInstance);
    }
    return (TRUE) as BOOL;
}

impl CicSystemModulePath {
    pub unsafe fn Init(&mut self, pszFileName: LPCTSTR, bSysWinDir: BOOL) -> BOOL {
        let this_: *mut Self = self;
        let mut cchPath: SIZE_T;
        if bSysWinDir != 0 {
            // Usually C:\Windows or C:\ReactOS
            cchPath = GetSystemWindowsDirectory((*this_).m_szPath.as_mut_ptr(), _countof!((*this_).m_szPath)) as SIZE_T;
        } else {
            // Usually C:\Windows\system32 or C:\ReactOS\system32
            cchPath = GetSystemDirectory((*this_).m_szPath.as_mut_ptr(), _countof!((*this_).m_szPath)) as SIZE_T;
        }
    
        (*this_).m_szPath[(_countof!((*this_).m_szPath) - 1) as usize] = ('\0' as u16); // Avoid buffer overrun
    
        'body: {
            if (cchPath == 0) || (cchPath > (_countof!((*this_).m_szPath) - 2) as _) {
                break 'body;
            }
    
            // Add backslash if necessary
            if (cchPath > 0) && ((*this_).m_szPath[(cchPath - 1) as usize] != ('\\' as u16)) {
                (*this_).m_szPath[(cchPath + 0) as usize] = ('\\' as u16);
                (*this_).m_szPath[(cchPath + 1) as usize] = ('\0' as u16);
            }
    
            // Append pszFileName
            if (StringCchCat((*this_).m_szPath.as_mut_ptr(), (_countof!((*this_).m_szPath)) as _, pszFileName)) < 0 {
                break 'body;
            }
    
            (*this_).m_cchPath = _tcslen((*this_).m_szPath.as_mut_ptr()) as SIZE_T;
            return (TRUE) as BOOL;
        }
    
        // Failure:
        (*this_).m_szPath[0] = UNICODE_NULL;
        (*this_).m_cchPath = 0;
        return (FALSE) as BOOL;
    }
}
