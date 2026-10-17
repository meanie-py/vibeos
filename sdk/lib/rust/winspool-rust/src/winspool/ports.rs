use crate::winspool::utils::DllAllocSplMem;
use ntdll_sys::seh::rpc_try;
use crate::winspool::utils::DllFreeSplMem;
use crate::winspool::marshalling::MarshallUpStructuresArray;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_32434DC5B0F7: [u16; 2] = [0x002Cu16, 0x0000];
pub static RUSTCONV_WSTR_657B9E92C1BA: [u16; 10] = [0x004Du16, 0x006F, 0x006E, 0x0069, 0x0074, 0x006F, 0x0072, 0x0055, 0x0049, 0x0000];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];
pub static RUSTCONV_WSTR_CDE948BA27D8: [u16; 8] = [0x0058u16, 0x0063, 0x0076, 0x0050, 0x006F, 0x0072, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_D483EA29E8A8: [u16; 11] = [
    0x0058u16, 0x0063, 0x0076, 0x004D, 0x006F, 0x006E, 0x0069, 0x0074, 0x006F, 0x0072,
    0x0000,
];

use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct MONITORUIDATA {
    pub hLibrary: HMODULE,
    pub hActCtx: HANDLE,
    pub ulpCookie: ULONG_PTR,
    pub pModuleName: PWSTR,
    pub Activeated: BOOL,
}
pub type PMONITORUIDATA = *mut MONITORUIDATA;

// C: `typedef DWORD (*PPfpFunction)(LPWSTR, ULONG_PTR, LPWSTR);` -- default (cdecl) convention,
// the MIDL-generated _Rpc* stubs it points at are __cdecl. Plain fn pointer (never NULL here).
// A C function-POINTER typedef is nullable: as a bare `fn`, `zeroed()` of the PORTTHREADINFO
// holding it panicked in every StartPortThread (rustc's invalid_value).
pub type PPfpFunction = Option<unsafe extern "C" fn(LPWSTR, ULONG_PTR, LPWSTR) -> DWORD>;

#[repr(C)]
pub struct PORTTHREADINFO {
    pub pName: LPWSTR,
    pub hWnd: ULONG_PTR,
    pub pPortName: LPWSTR,
    pub fpFunction: PPfpFunction,
    pub dwErrorCode: DWORD,
    pub hEvent: HANDLE,
}
pub type PPORTTHREADINFO = *mut PORTTHREADINFO;

// The remaining lines of the preamble are function-body statements (if/goto/
// assignments/cleanup), not file-scope declarations, so they are not emitted here.

#[no_mangle]
pub unsafe extern "system" fn IntPortThread(pPortThreadInfo: PPORTTHREADINFO) {
    // RpcTryExcept
    {
        (*pPortThreadInfo).dwErrorCode = rpc_try(|| {
            ((*pPortThreadInfo).fpFunction.unwrap_unchecked())(
                (*pPortThreadInfo).pName,
                (*pPortThreadInfo).hWnd,
                (*pPortThreadInfo).pPortName,
            )
        })
        .unwrap_or_else(|code| code as _);
    }
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER): handler runs only when the try body raises
    // (*pPortThreadInfo).dwErrorCode = RpcExceptionCode();
    // RpcEndExcept;

    SetEvent((*pPortThreadInfo).hEvent);
}

#[no_mangle]
pub unsafe extern "system" fn StartPortThread(pName: LPWSTR, hWnd: HWND, pPortName: LPWSTR, fpFunction: PPfpFunction) -> BOOL {
    let mut PortThreadInfo: PORTTHREADINFO = core::mem::zeroed();
    let htHandle: HANDLE;
    let mut Msg: MSG = core::mem::zeroed();
    let mut tid: DWORD = 0;

    if !hWnd.is_null() { EnableWindow( hWnd, FALSE ); }

    PortThreadInfo.pName = pName;
    PortThreadInfo.hWnd = hWnd as ULONG_PTR;
    PortThreadInfo.pPortName = pPortName;
    PortThreadInfo.fpFunction = fpFunction;
    PortThreadInfo.dwErrorCode = ERROR_SUCCESS;
    PortThreadInfo.hEvent = CreateEventW( core::ptr::null_mut(), TRUE, FALSE, core::ptr::null_mut() );

    htHandle = CreateThread( core::ptr::null_mut(),
                             32 * 1024,
                            Some(core::mem::transmute::<unsafe extern "system" fn(PPORTTHREADINFO), unsafe extern "system" fn(LPVOID) -> DWORD>(IntPortThread)),
                            core::ptr::addr_of_mut!(PortThreadInfo) as *mut core::ffi::c_void,
                             0,
                            core::ptr::addr_of_mut!(tid) );

    CloseHandle( htHandle );

    while MsgWaitForMultipleObjects( 1, core::ptr::addr_of_mut!(PortThreadInfo.hEvent), FALSE, INFINITE, QS_ALLINPUT ) == 1 {
        while PeekMessageW( core::ptr::addr_of_mut!(Msg), core::ptr::null_mut(), 0, 0, PM_REMOVE ) != 0 {
            TranslateMessage( core::ptr::addr_of_mut!(Msg) );
            DispatchMessageW( core::ptr::addr_of_mut!(Msg) );
        }
    }

    CloseHandle( PortThreadInfo.hEvent );

    if !hWnd.is_null() {
        EnableWindow(hWnd, TRUE);
        SetForegroundWindow(hWnd);
        SetFocus(hWnd);
    }

    SetLastError(PortThreadInfo.dwErrorCode);
    return (PortThreadInfo.dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetMonitorUIFullName(pDeviceName: PWSTR, pModuleName: *mut PWSTR) -> BOOL {
    let SysDir: STRSAFE_LPWSTR;
    let length: UINT;
    let hr: HRESULT;

    *pModuleName = core::ptr::null_mut();

    SysDir = HeapAlloc(hProcessHeap, 0, (MAX_PATH as usize * core::mem::size_of::<u16>()) as SIZE_T) as STRSAFE_LPWSTR;

    if !SysDir.is_null() {
        memset(SysDir as *mut core::ffi::c_void, 0, MAX_PATH as usize * core::mem::size_of::<u16>());

        length = GetSystemDirectoryW(SysDir, (MAX_PATH as usize * core::mem::size_of::<u16>()) as UINT);

        if length > 0 {
            StringCbCatW(SysDir, MAX_PATH as usize * core::mem::size_of::<u16>(), RUSTCONV_WSTR_B7FC02907B1F.as_ptr());

            hr = StringCchCatW(SysDir, MAX_PATH as usize * core::mem::size_of::<u16>(), pDeviceName);
            if !((hr) < 0) {
                *pModuleName = SysDir;
                return TRUE as BOOL;
            }
            SetLastError(HRESULT_CODE!(hr) as DWORD);
        }

        HeapFree(hProcessHeap, 0, SysDir as *mut core::ffi::c_void);
    }
    return FALSE;
}
#[no_mangle]
pub unsafe extern "system" fn GetMonitorUIActivationContext(pDeviceName: PWSTR, pmuid: PMONITORUIDATA) -> BOOL {
    let mut Ret: BOOL = FALSE as BOOL;

    if GetMonitorUIFullName(pDeviceName, core::ptr::addr_of_mut!((*pmuid).pModuleName)) == 0 {
        return Ret;
    }
    /*    OMG! SxS again?
    memset(&actctx, 0, sizeof(ACTCTXW));
    actctx.cbSize = sizeof(ACTCTXW);
    actctx.dwFlags = ACTCTX_FLAG_RESOURCE_NAME_VALID;
    actctx.lpResourceName = MAKEINTRESOURCEW(123); This might be the reason....
    actctx.lpSource = pmuid->pModuleName;

    handle = CreateActCtxW(&actctx);

    if ( handle != INVALID_HANDLE_VALUE )
    {
        pmuid->hActCtx = handle;
        if ( ActivateActCtx( handle, &pmuid->ulpCookie ) )
        {
            pmuid->Activeated = TRUE;
            Ret = TRUE;
        }
        else
        {
            pmuid->Activeated = FALSE;
        }
    }
    else
    {
        ERR("GetMonitorUIActivationContext Failed %S\n",pmuid->pModuleName);
    }*/
    (*pmuid).hActCtx = INVALID_HANDLE_VALUE;
    Ret = TRUE as BOOL;
    return Ret;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn FreeMonitorUI(pmuid: PMONITORUIDATA) {
    if !pmuid.is_null() {
        if !(*pmuid).hLibrary.is_null() {
            FreeLibrary((*pmuid).hLibrary);
        }
        if (*pmuid).Activeated != 0 {
            DeactivateActCtx(0, (*pmuid).ulpCookie);
        }
        if (*pmuid).hActCtx != INVALID_HANDLE_VALUE {
            ReleaseActCtx((*pmuid).hActCtx);
        }
        if !(*pmuid).pModuleName.is_null() {
            DllFreeSplMem((*pmuid).pModuleName as PVOID);
        }
        DllFreeSplMem(pmuid as PVOID);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn FreeMonitorUI(pmuid: PMONITORUIDATA) {
    if !pmuid.is_null() {
        if !(*pmuid).hLibrary.is_null() {
            FreeLibrary((*pmuid).hLibrary);
        }
        if (*pmuid).Activeated != 0 {
            DeactivateActCtx(0, (*pmuid).ulpCookie);
        }
        if (*pmuid).hActCtx != INVALID_HANDLE_VALUE {
            ReleaseActCtx((*pmuid).hActCtx);
        }
        if !(*pmuid).pModuleName.is_null() {
            DllFreeSplMem((*pmuid).pModuleName as PVOID);
        }
        DllFreeSplMem(pmuid as PVOID);
    }
}
// C: `BOOL FASTCALL StrNCatBuff(PWSTR ptr, size_t Size, PWSTR args, ...)` -- a NULL-terminated
// vararg list. Stable Rust cannot define a C-variadic function, and the function is internal
// (only ConstructXcvName calls it), so the list is passed as a slice of the strings with the
// NULL terminator implied by the slice end (a NULL element also stops, as in C).
pub unsafe fn StrNCatBuff(ptr: PWSTR, Size: usize, args: &[PWSTR]) -> BOOL {
    let mut Ret: BOOL = TRUE as BOOL;

    for &pwstr in args {
        if pwstr.is_null() {
            break;
        }
        let hr: HRESULT = StringCchCatNW(ptr, Size, pwstr, wcslen(pwstr));
        if hr < 0 {
            SetLastError(HRESULT_CODE!(hr) as DWORD);
            Ret = FALSE as BOOL;
            break;
        }
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn ConstructXcvName(pName: PWSTR, pMonitorPortName: PWSTR, pXcvName: PWSTR) -> PWSTR {
    let mut Ret: i32 = FALSE as i32;
    let mut pwstr: PWSTR = core::ptr::null_mut();
    let mut sXcv: usize;
    let mut smpn: usize = 0;
    let mut Size: usize = 0;

    if !pName.is_null() {
        Size = wcslen(pName) + 1;
    }

    sXcv = wcslen(pXcvName) + Size;

    if !pMonitorPortName.is_null() {
        smpn = wcslen(pMonitorPortName);
    }

    Size = sXcv + smpn + 3;

    pwstr = DllAllocSplMem((Size * core::mem::size_of::<u16>()) as DWORD) as PWSTR;

    // Divergence from the C: it calls memset(pwstr, 0, Size) BEFORE the NULL check (crashes
    // when the allocation fails) and with a WCHAR count where memset wants bytes (clears only
    // half the buffer StrNCatBuff then appends into). Both are demonstrably wrong in the C.
    if !pwstr.is_null() {
        memset(pwstr as *mut core::ffi::c_void, 0, Size * core::mem::size_of::<u16>());
        // The caller wants an Xcv handle and provided a string like:
        //    ", XcvMonitor Local Port"
        //    "\\COMPUTERNAME\, XcvMonitor Local Port"
        //    ", XcvPort LPT1:"
        //    "\\COMPUTERNAME\, XcvPort LPT1:"
        //
        //    This produces; !pName ",XcvMonitor " or pName "\\COMPUTERNAME\XcvMonitor "
        //
        Ret = StrNCatBuff(
            pwstr,
            Size,
            &[
                if !pName.is_null() { pName } else { RUSTCONV_WSTR_96A296D224F2.as_ptr() as PWSTR },
                if !pName.is_null() { RUSTCONV_WSTR_B7FC02907B1F.as_ptr() as PWSTR } else { RUSTCONV_WSTR_32434DC5B0F7.as_ptr() as PWSTR },
                pXcvName,
                RUSTCONV_WSTR_8D71B3FAAB82.as_ptr() as PWSTR,
                if !pMonitorPortName.is_null() { pMonitorPortName } else { RUSTCONV_WSTR_96A296D224F2.as_ptr() as PWSTR },
            ],
        );
    }

    if Ret == 0 {
        DllFreeSplMem(pwstr as PVOID);
        pwstr = core::ptr::null_mut();
    }

    pwstr
}

#[no_mangle]
pub unsafe extern "system" fn GetMonitorUI(pName: PWSTR, pMonitorPortName: PWSTR, pXcvName: PWSTR, pmui: *mut PMONITORUI, ppmuid: *mut PMONITORUIDATA) -> DWORD {
    let mut dwErrorCode: DWORD = ERROR_SUCCESS;
    let mut cbOutputNeeded: DWORD = 0;
    let mut dwStatus: DWORD = 0;
    let mut hPrinter: HANDLE = core::ptr::null_mut();
    let mut hModule: HMODULE = core::ptr::null_mut();
    let mut pHandle: PSPOOLER_HANDLE = hPrinter as PSPOOLER_HANDLE;
    let mut pDevice: PWSTR = core::ptr::null_mut();
    let mut pOutputString: PWSTR = core::ptr::null_mut();
    let mut pmuid: PMONITORUIDATA = core::ptr::null_mut();
    let mut wDefault: PRINTER_DEFAULTSW = PRINTER_DEFAULTSW {
        pDatatype: core::ptr::null_mut(),
        pDevMode: core::ptr::null_mut(),
        DesiredAccess: PRINTER_ATTRIBUTE_QUEUED,
    };
    let mut OutputData: [u8; 1024] = [0u8; 1024];
    let mut InputData: [u8; 4] = [0u8; 4];

    *pmui = core::ptr::null_mut();
    *ppmuid = core::ptr::null_mut();

    pDevice = ConstructXcvName(pName, pMonitorPortName, pXcvName);

    if pDevice.is_null() {
        return GetLastError();
    }

    if OpenPrinterW(pDevice as LPWSTR, core::ptr::addr_of_mut!(hPrinter), core::ptr::addr_of_mut!(wDefault)) != 0 {
        pHandle = hPrinter as PSPOOLER_HANDLE;

        // RpcTryExcept
        {
            dwErrorCode = rpc_try(|| _RpcXcvData(
                (*pHandle).hPrinter,
                RUSTCONV_WSTR_657B9E92C1BA.as_ptr(),
                core::ptr::addr_of_mut!(InputData) as PBYTE,
                0,
                core::ptr::addr_of_mut!(OutputData) as PBYTE,
                1024,
                core::ptr::addr_of_mut!(cbOutputNeeded),
                core::ptr::addr_of_mut!(dwStatus),
            )).unwrap_or_else(|code| code as _);
        }
        // RpcExcept(EXCEPTION_EXECUTE_HANDLER): handler is
        //     dwErrorCode = RpcExceptionCode();
        // Structured exception handling is not modeled in Rust; the
        // protected body above is the only executed path.

        if dwErrorCode == ERROR_INSUFFICIENT_BUFFER {
            pOutputString = DllAllocSplMem(cbOutputNeeded) as PWSTR;

            // RpcTryExcept
            {
                dwErrorCode = rpc_try(|| _RpcXcvData(
                    (*pHandle).hPrinter,
                    RUSTCONV_WSTR_657B9E92C1BA.as_ptr(),
                    core::ptr::addr_of_mut!(InputData) as PBYTE,
                    0,
                    pOutputString as PBYTE,
                    cbOutputNeeded,
                    core::ptr::addr_of_mut!(cbOutputNeeded),
                    core::ptr::addr_of_mut!(dwStatus),
                )).unwrap_or_else(|code| code as _);
            }
            // RpcExcept(EXCEPTION_EXECUTE_HANDLER): handler is
            //     dwErrorCode = RpcExceptionCode();
            // Not modeled (see above).
        }
    }

    dwErrorCode
}

#[no_mangle]
pub unsafe extern "system" fn AddPortA(pName: PSTR, hWnd: HWND, pMonitorName: PSTR) -> BOOL {
    let mut nameW: LPWSTR = core::ptr::null_mut();
    let mut monitorW: LPWSTR = core::ptr::null_mut();
    let mut len: DWORD;
    let mut res: BOOL;

    if !pName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pName as LPCSTR, -1, core::ptr::null_mut(), 0) as DWORD;
        nameW = HeapAlloc(GetProcessHeap(), 0, (len * core::mem::size_of::<u16>() as DWORD) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pName as LPCSTR, -1, nameW, len as i32);
    }

    if !pMonitorName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pMonitorName as LPCSTR, -1, core::ptr::null_mut(), 0) as DWORD;
        monitorW = HeapAlloc(GetProcessHeap(), 0, (len * core::mem::size_of::<u16>() as DWORD) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pMonitorName as LPCSTR, -1, monitorW, len as i32);
    }

    res = AddPortW(nameW, hWnd, monitorW);

    if !nameW.is_null() {
        HeapFree(GetProcessHeap(), 0, nameW as *mut core::ffi::c_void);
    }
    if !monitorW.is_null() {
        HeapFree(GetProcessHeap(), 0, monitorW as *mut core::ffi::c_void);
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn AddPortExW(pName: PWSTR, Level: DWORD, lpBuffer: PBYTE, lpMonitorName: PWSTR) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut PortInfoContainer: WINSPOOL_PORT_CONTAINER = core::mem::zeroed();
    let mut PortVarContainer: WINSPOOL_PORT_VAR_CONTAINER = core::mem::zeroed();
    let mut pPortInfoFF: *mut WINSPOOL_PORT_INFO_FF;

    match Level {
        1 => {
            // FIXME!!!! Only Level 1 is supported? See note in wine winspool test info.c : line 575. It's just not supported here.
            PortInfoContainer.PortInfo.pPortInfo1 = lpBuffer as *mut WINSPOOL_PORT_INFO_1;
            PortInfoContainer.Level = Level;
            PortVarContainer.cbMonitorData = 0;
            PortVarContainer.pMonitorData = core::ptr::null_mut();
        }
        0xFFFFFFFF => {
            pPortInfoFF = lpBuffer as *mut WINSPOOL_PORT_INFO_FF;
            PortInfoContainer.PortInfo.pPortInfoFF = pPortInfoFF;
            PortInfoContainer.Level = Level;
            PortVarContainer.cbMonitorData = (*pPortInfoFF).cbMonitorData;
            PortVarContainer.pMonitorData = (*pPortInfoFF).pMonitorData;
        }
        _ => {
            // ERR("Level = %d, unsupported!\n", Level);
            SetLastError(ERROR_INVALID_LEVEL);
            return FALSE as BOOL;
        }
    }

    // Do the RPC call
    // RpcTryExcept
    dwErrorCode = rpc_try(|| _RpcAddPortEx(
        pName,
        core::ptr::addr_of_mut!(PortInfoContainer),
        core::ptr::addr_of_mut!(PortVarContainer),
        lpMonitorName,
    )).unwrap_or_else(|code| code as _);
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER) { dwErrorCode = RpcExceptionCode(); }
    // (SEH exception path: RpcExceptionCode() supplies the code on an RPC fault; not reachable without SEH support)
    // RpcEndExcept;

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn AddPortExA(pName: PSTR, Level: DWORD, lpBuffer: PBYTE, lpMonitorName: PSTR) -> BOOL {
    let mut pi1W: PORT_INFO_1W = core::mem::zeroed();
    let mut pi1A: *mut PORT_INFO_1A = lpBuffer as *mut PORT_INFO_1A;
    let mut nameW: LPWSTR = core::ptr::null_mut();
    let mut monitorW: LPWSTR = core::ptr::null_mut();
    let mut len: DWORD = 0;
    let mut res: BOOL = FALSE as BOOL;
    let mut pPortInfoFF: *mut WINSPOOL_PORT_INFO_FF = lpBuffer as *mut WINSPOOL_PORT_INFO_FF;
    let mut PortInfoFF: WINSPOOL_PORT_INFO_FF = core::mem::zeroed();

    'cleanup: {
        if lpBuffer.is_null() || lpMonitorName.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
            return FALSE as BOOL;
        }

        if !pName.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, pName as LPCSTR, -1, core::ptr::null_mut(), 0) as DWORD;
            nameW = HeapAlloc(GetProcessHeap(), 0, len as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T) as LPWSTR;
            MultiByteToWideChar(CP_ACP, 0, pName as LPCSTR, -1, nameW, len as core::ffi::c_int);
        }

        if !lpMonitorName.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, lpMonitorName as LPCSTR, -1, core::ptr::null_mut(), 0) as DWORD;
            monitorW = HeapAlloc(GetProcessHeap(), 0, len as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T) as LPWSTR;
            MultiByteToWideChar(CP_ACP, 0, lpMonitorName as LPCSTR, -1, monitorW, len as core::ffi::c_int);
        }

        pi1W.pName = core::ptr::null_mut();
        ZeroMemory!(core::ptr::addr_of_mut!(PortInfoFF) as *mut core::ffi::c_void, core::mem::size_of::<WINSPOOL_PORT_INFO_FF>());

        if Level == 1 {
            if !(*pi1A).pName.is_null() {
                len = MultiByteToWideChar(CP_ACP, 0, (*pi1A).pName as LPCSTR, -1, core::ptr::null_mut(), 0) as DWORD;
                pi1W.pName = HeapAlloc(GetProcessHeap(), 0, len as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T) as LPWSTR;
                MultiByteToWideChar(CP_ACP, 0, (*pi1A).pName as LPCSTR, -1, pi1W.pName, len as core::ffi::c_int);
            }
        } else if Level == 0xFFFFFFFF {
            // Remember the calling parameter is Ansi.
            if (*pPortInfoFF).pPortName.is_null() || ((*pPortInfoFF).pPortName as PCHAR).is_null() {
                SetLastError(ERROR_INVALID_PARAMETER);
                break 'cleanup;
            }

            len = MultiByteToWideChar(CP_ACP, 0, (*pPortInfoFF).pPortName as LPCSTR, -1, core::ptr::null_mut(), 0) as DWORD;
            PortInfoFF.pPortName = HeapAlloc(GetProcessHeap(), 0, len as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T) as LPWSTR;
            MultiByteToWideChar(CP_ACP, 0, (*pPortInfoFF).pPortName as LPCSTR, -1, PortInfoFF.pPortName as LPWSTR, len as core::ffi::c_int);

            PortInfoFF.cbMonitorData = (*pPortInfoFF).cbMonitorData;
            PortInfoFF.pMonitorData = (*pPortInfoFF).pMonitorData;
        } else {
            // ERR("Level = %d, unsupported!\n", Level);
            SetLastError(ERROR_INVALID_LEVEL);
            break 'cleanup;
        }

        res = AddPortExW(
            nameW,
            Level,
            if Level == 1 {
                core::ptr::addr_of_mut!(pi1W) as PBYTE
            } else {
                core::ptr::addr_of_mut!(PortInfoFF) as PBYTE
            },
            monitorW,
        );
    }

    // Cleanup:
    if !nameW.is_null() {
        HeapFree(GetProcessHeap(), 0, nameW as *mut core::ffi::c_void);
    }
    if !monitorW.is_null() {
        HeapFree(GetProcessHeap(), 0, monitorW as *mut core::ffi::c_void);
    }
    if !pi1W.pName.is_null() {
        HeapFree(GetProcessHeap(), 0, pi1W.pName as *mut core::ffi::c_void);
    }
    if !PortInfoFF.pPortName.is_null() {
        HeapFree(GetProcessHeap(), 0, PortInfoFF.pPortName as *mut core::ffi::c_void);
    }

    res
}

#[no_mangle]
pub unsafe extern "system" fn AddPortW(pName: PWSTR, hWnd: HWND, pMonitorName: PWSTR) -> BOOL {
    let mut dwSessionId: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut pmuid: PMONITORUIDATA = core::ptr::null_mut();
    let mut pmui: PMONITORUI = core::ptr::null_mut();
    let mut Ret: BOOL = FALSE;

    dwErrorCode = GetMonitorUI(
        pName,
        pMonitorName,
        RUSTCONV_WSTR_D483EA29E8A8.as_ptr() as PWSTR,
        core::ptr::addr_of_mut!(pmui),
        core::ptr::addr_of_mut!(pmuid),
    );

    if dwErrorCode != ERROR_SUCCESS {
        if dwErrorCode == ERROR_NOT_SUPPORTED
            || dwErrorCode == ERROR_MOD_NOT_FOUND
            || dwErrorCode == ERROR_INVALID_PRINT_MONITOR
            || dwErrorCode == ERROR_UNKNOWN_PORT
            || dwErrorCode == ERROR_INVALID_PRINTER_NAME
        {
            if (ProcessIdToSessionId(
                GetCurrentProcessId(),
                core::ptr::addr_of_mut!(dwSessionId),
            ) != 0)
                && dwSessionId != 0
            {
                dwErrorCode = ERROR_NOT_SUPPORTED;
            } else {
                Ret = StartPortThread(
                    pName,
                    hWnd,
                    pMonitorName,
                    Some(_RpcAddPort),
                );
                dwErrorCode = GetLastError();
            }
        }
    } else {
        // C calls the pointer unchecked; a monitor UI lacking it would crash there.
        Ret = match (*pmui).pfnAddPortUI {
            Some(pfnAddPortUI) => pfnAddPortUI(pName, hWnd, pMonitorName, core::ptr::null_mut()),
            None => FALSE,
        };
    }

    SetLastError(dwErrorCode);
    FreeMonitorUI(pmuid);

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn ConfigurePortA(pName: PSTR, hWnd: HWND, pPortName: PSTR) -> i32 {
    let mut nameW: LPWSTR = core::ptr::null_mut();
    let mut portW: LPWSTR = core::ptr::null_mut();
    let mut len: i32;
    let mut res: u32;

    /* convert servername to unicode */
    if !pName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pName, -1, core::ptr::null_mut(), 0);
        nameW = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pName, -1, nameW, len);
    }

    /* convert portname to unicode */
    if !pPortName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pPortName, -1, core::ptr::null_mut(), 0);
        portW = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pPortName, -1, portW, len);
    }

    res = ConfigurePortW(nameW, hWnd, portW) as u32;

    if !nameW.is_null() {
        HeapFree(GetProcessHeap(), 0, nameW as *mut core::ffi::c_void);
    }
    if !portW.is_null() {
        HeapFree(GetProcessHeap(), 0, portW as *mut core::ffi::c_void);
    }

    return res as i32;
}

#[no_mangle]
pub unsafe extern "system" fn ConfigurePortW(pName: PWSTR, hWnd: HWND, pPortName: PWSTR) -> BOOL {
    let mut dwSessionId: DWORD = 0;
    let mut dwErrorCode: DWORD = 0;
    let mut pmuid: *mut MONITORUIDATA = core::ptr::null_mut();
    let mut pmui: PMONITORUI = core::ptr::null_mut();
    let mut Ret: BOOL = FALSE;

    dwErrorCode = GetMonitorUI(pName, pPortName, RUSTCONV_WSTR_CDE948BA27D8.as_ptr() as PWSTR, core::ptr::addr_of_mut!(pmui), core::ptr::addr_of_mut!(pmuid));

    if dwErrorCode != ERROR_SUCCESS {
        if dwErrorCode == ERROR_NOT_SUPPORTED
            || dwErrorCode == ERROR_MOD_NOT_FOUND
            || dwErrorCode == ERROR_INVALID_PRINT_MONITOR
            || dwErrorCode == ERROR_UNKNOWN_PORT
            || dwErrorCode == ERROR_INVALID_PRINTER_NAME
        {
            if ProcessIdToSessionId(GetCurrentProcessId(), core::ptr::addr_of_mut!(dwSessionId)) != 0 && dwSessionId != 0 {
                // Looking if this is remote.
                dwErrorCode = ERROR_NOT_SUPPORTED;
            } else {
                Ret = StartPortThread(pName, hWnd, pPortName, Some(_RpcConfigurePort));
                dwErrorCode = GetLastError();
            }
        }
    } else {
        Ret = match (*pmui).pfnConfigurePortUI {
            Some(pfnConfigurePortUI) => pfnConfigurePortUI(pName, hWnd, pPortName),
            None => FALSE,
        };
    }

    SetLastError(dwErrorCode);
    FreeMonitorUI(pmuid);

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePortA(pName: PSTR, hWnd: HWND, pPortName: PSTR) -> BOOL {
    let mut nameW: LPWSTR = core::ptr::null_mut();
    let mut portW: LPWSTR = core::ptr::null_mut();
    let mut len: INT;
    let mut res: DWORD;

    /* convert servername to unicode */
    if !pName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pName, -(1), core::ptr::null_mut(), 0);
        nameW = HeapAlloc(GetProcessHeap(), 0, (len as SIZE_T) * (core::mem::size_of::<u16>() as SIZE_T)) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pName, -(1), nameW, len);
    }

    /* convert portname to unicode */
    if !pPortName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pPortName, -(1), core::ptr::null_mut(), 0);
        portW = HeapAlloc(GetProcessHeap(), 0, (len as SIZE_T) * (core::mem::size_of::<u16>() as SIZE_T)) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pPortName, -(1), portW, len);
    }

    res = DeletePortW(nameW, hWnd, portW) as DWORD;

    if !nameW.is_null() {
        HeapFree(GetProcessHeap(), 0, nameW as LPVOID);
    }
    if !portW.is_null() {
        HeapFree(GetProcessHeap(), 0, portW as LPVOID);
    }

    return res as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn DeletePortW(pName: PWSTR, hWnd: HWND, pPortName: PWSTR) -> BOOL {
    let mut dwErrorCode: DWORD = 0;
    let mut pmuid: PMONITORUIDATA = core::ptr::null_mut();
    let mut pmui: PMONITORUI = core::ptr::null_mut();
    let mut Ret: BOOL = FALSE as BOOL;

    dwErrorCode = GetMonitorUI(pName, pPortName, RUSTCONV_WSTR_CDE948BA27D8.as_ptr() as PWSTR, core::ptr::addr_of_mut!(pmui), core::ptr::addr_of_mut!(pmuid));

    if dwErrorCode != ERROR_SUCCESS {
        if dwErrorCode == ERROR_NOT_SUPPORTED
            || dwErrorCode == ERROR_MOD_NOT_FOUND
            || dwErrorCode == ERROR_INVALID_PRINT_MONITOR
            || dwErrorCode == ERROR_UNKNOWN_PORT
            || dwErrorCode == ERROR_INVALID_PRINTER_NAME
        {
            Ret = StartPortThread(pName, hWnd, pPortName, Some(_RpcDeletePort));
            dwErrorCode = GetLastError();
        }
    } else {
        Ret = match (*pmui).pfnDeletePortUI {
            Some(pfnDeletePortUI) => pfnDeletePortUI(pName, hWnd, pPortName),
            None => FALSE,
        };
    }

    SetLastError(dwErrorCode);
    FreeMonitorUI(pmuid);

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn EnumPortsA(pName: PSTR, Level: DWORD, pPorts: PBYTE, mut cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut res: BOOL = 0;
    let mut bufferW: LPBYTE = core::ptr::null_mut();
    let mut nameW: LPWSTR = core::ptr::null_mut();
    let mut needed: DWORD = 0;
    let mut numentries: DWORD = 0;
    let mut len: INT = 0;

    if (Level < 1) || (Level > 2)
    {
        SetLastError(ERROR_INVALID_LEVEL);
        return FALSE as BOOL;
    }

    'cleanup: {
        /* convert servername to unicode */
        if !pName.is_null()
        {
            len = MultiByteToWideChar(CP_ACP, 0, pName, -1, core::ptr::null_mut(), 0);
            nameW = HeapAlloc(GetProcessHeap(), 0, ((len as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
            MultiByteToWideChar(CP_ACP, 0, pName, -1, nameW, len);
        }
        /* alloc (userbuffersize*sizeof(WCHAR) and try to enum the Ports */
        needed = cbBuf * (core::mem::size_of::<WCHAR>() as DWORD);
        if needed != 0 {
            bufferW = HeapAlloc(GetProcessHeap(), 0, needed as SIZE_T) as LPBYTE;
        }
        res = EnumPortsW(nameW, Level, bufferW, needed, pcbNeeded, pcReturned);

        if res == 0 && (GetLastError() == ERROR_INSUFFICIENT_BUFFER)
        {
            if !pcbNeeded.is_null() {
                needed = *pcbNeeded;
            }
            /* HeapReAlloc return NULL, when bufferW was NULL */
            bufferW = if !bufferW.is_null() {
                HeapReAlloc(GetProcessHeap(), 0, bufferW as LPVOID, needed as SIZE_T) as LPBYTE
            } else {
                HeapAlloc(GetProcessHeap(), 0, needed as SIZE_T) as LPBYTE
            };

            /* Try again with the large Buffer */
            res = EnumPortsW(nameW, Level, bufferW, needed, pcbNeeded, pcReturned);
        }
        needed = if !pcbNeeded.is_null() { *pcbNeeded } else { 0 };
        numentries = if !pcReturned.is_null() { *pcReturned } else { 0 };

        /*
           W2k require the buffersize from EnumPortsW also for EnumPortsA.
           We use the smaller Ansi-Size to avoid conflicts with fixed Buffers of old Apps.
         */
        if res != 0
        {
            /* EnumPortsW collected all Data. Parse them to calculate ANSI-Size */
            let mut entrysize: DWORD = 0;
            let mut index: DWORD = 0;
            let mut ptr: LPSTR = core::ptr::null_mut();
            let mut pi2w: LPPORT_INFO_2W = core::ptr::null_mut();
            let mut pi2a: LPPORT_INFO_2A = core::ptr::null_mut();

            needed = 0;
            entrysize = if Level == 1 {
                core::mem::size_of::<PORT_INFO_1A>() as DWORD
            } else {
                core::mem::size_of::<PORT_INFO_2A>() as DWORD
            };

            /* First pass: calculate the size for all Entries */
            pi2w = bufferW as LPPORT_INFO_2W;
            pi2a = pPorts as LPPORT_INFO_2A;
            index = 0;
            while index < numentries
            {
                index += 1;
                needed += entrysize;    /* PORT_INFO_?A */

                needed += WideCharToMultiByte(CP_ACP, 0, (*pi2w).pPortName, -1,
                                                core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as DWORD;
                if Level > 1
                {
                    needed += WideCharToMultiByte(CP_ACP, 0, (*pi2w).pMonitorName, -1,
                                                    core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as DWORD;
                    needed += WideCharToMultiByte(CP_ACP, 0, (*pi2w).pDescription, -1,
                                                    core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as DWORD;
                }
                /* use LPBYTE with entrysize to avoid double code (PORT_INFO_1 + PORT_INFO_2) */
                pi2w = (pi2w as LPBYTE).add(entrysize as usize) as LPPORT_INFO_2W;
                pi2a = (pi2a as LPBYTE).add(entrysize as usize) as LPPORT_INFO_2A;
            }

            /* check for errors and quit on failure */
            if cbBuf < needed
            {
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                res = FALSE as BOOL;
                break 'cleanup;
            }
            len = (entrysize * numentries) as INT;       /* room for all PORT_INFO_?A */
            ptr = pPorts.add(len as usize) as LPSTR;     /* room for strings */
            cbBuf -= len as DWORD ;                      /* free Bytes in the user-Buffer */
            pi2w = bufferW as LPPORT_INFO_2W;
            pi2a = pPorts as LPPORT_INFO_2A;
            index = 0;
            /* Second Pass: Fill the User Buffer (if we have one) */
            while (index < numentries) && !pPorts.is_null()
            {
                index += 1;

                (*pi2a).pPortName = ptr;
                len = WideCharToMultiByte(CP_ACP, 0, (*pi2w).pPortName, -1,
                                                ptr, cbBuf as INT, core::ptr::null_mut(), core::ptr::null_mut());
                ptr = ptr.add(len as usize);
                cbBuf -= len as DWORD;
                if Level > 1
                {
                    (*pi2a).pMonitorName = ptr;
                    len = WideCharToMultiByte(CP_ACP, 0, (*pi2w).pMonitorName, -1,
                                                ptr, cbBuf as INT, core::ptr::null_mut(), core::ptr::null_mut());
                    ptr = ptr.add(len as usize);
                    cbBuf -= len as DWORD;

                    (*pi2a).pDescription = ptr;
                    len = WideCharToMultiByte(CP_ACP, 0, (*pi2w).pDescription, -1,
                                                ptr, cbBuf as INT, core::ptr::null_mut(), core::ptr::null_mut());
                    ptr = ptr.add(len as usize);
                    cbBuf -= len as DWORD;

                    (*pi2a).fPortType = (*pi2w).fPortType;
                    (*pi2a).Reserved = 0; /* documented: "must be zero" */
                }
                /* use LPBYTE with entrysize to avoid double code (PORT_INFO_1 + PORT_INFO_2) */
                pi2w = (pi2w as LPBYTE).add(entrysize as usize) as LPPORT_INFO_2W;
                pi2a = (pi2a as LPBYTE).add(entrysize as usize) as LPPORT_INFO_2A;
            }
        }
    }

    // cleanup:
    if !pcbNeeded.is_null() {
        *pcbNeeded = needed;
    }
    if !pcReturned.is_null() {
        *pcReturned = if res != 0 { numentries } else { 0 };
    }

    if !nameW.is_null() {
        HeapFree(GetProcessHeap(), 0, nameW as LPVOID);
    }
    if !bufferW.is_null() {
        HeapFree(GetProcessHeap(), 0, bufferW as LPVOID);
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn EnumPortsW(pName: PWSTR, Level: DWORD, pPorts: PBYTE, cbBuf: DWORD, pcbNeeded: PDWORD, pcReturned: PDWORD) -> BOOL {
    let mut dwErrorCode: DWORD = 0;

    if (Level < 1) || (Level > 2) {
        SetLastError(ERROR_INVALID_LEVEL);
        return FALSE as BOOL;
    }

    // Do the RPC call
    // RpcTryExcept
    dwErrorCode = rpc_try(|| _RpcEnumPorts(pName, Level, pPorts, cbBuf, pcbNeeded, pcReturned)).unwrap_or_else(|code| code as _);
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER) -- handler reached only on an SEH exception
    // RpcEndExcept;

    if dwErrorCode == ERROR_SUCCESS {
        // Replace relative offset addresses in the output by absolute pointers.
        MarshallUpStructuresArray(cbBuf, pPorts as PVOID, *pcReturned, core::ptr::addr_of!((*pPortInfoMarshalling[Level as usize]).pInfo) as *const _MARSHALLING_INFO, (*pPortInfoMarshalling[Level as usize]).cbStructureSize, TRUE as BOOL);
    }

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetPortA(pName: PSTR, pPortName: PSTR, dwLevel: DWORD, pPortInfo: PBYTE) -> BOOL {
    let mut NameW: LPWSTR = core::ptr::null_mut();
    let mut PortNameW: LPWSTR = core::ptr::null_mut();
    let mut pi3W: PORT_INFO_3W = core::mem::zeroed();
    let mut pi3A: *mut PORT_INFO_3A;
    let mut len: core::ffi::c_int;
    let mut res: BOOL;

    pi3A = pPortInfo as *mut PORT_INFO_3A;

    if dwLevel != 3 {
        SetLastError(ERROR_INVALID_LEVEL);
        return FALSE as BOOL;
    }

    if !pName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pName, -1, core::ptr::null_mut(), 0);
        NameW = HeapAlloc(GetProcessHeap(), 0, (len as usize * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pName, -1, NameW, len);
    }

    if !pPortName.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, pPortName, -1, core::ptr::null_mut(), 0);
        PortNameW = HeapAlloc(GetProcessHeap(), 0, (len as usize * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, pPortName, -1, PortNameW, len);
    }

    if !(*pi3A).pszStatus.is_null() {
        len = MultiByteToWideChar(CP_ACP, 0, (*pi3A).pszStatus, -1, core::ptr::null_mut(), 0);
        pi3W.pszStatus = HeapAlloc(GetProcessHeap(), 0, (len as usize * core::mem::size_of::<u16>()) as SIZE_T) as PWSTR;
        MultiByteToWideChar(CP_ACP, 0, (*pi3A).pszStatus, -1, pi3W.pszStatus, len);
    }

    pi3W.dwStatus = (*pi3A).dwStatus;
    pi3W.dwSeverity = (*pi3A).dwSeverity;

    res = SetPortW(NameW, PortNameW, dwLevel, core::ptr::addr_of_mut!(pi3W) as PBYTE);

    if !NameW.is_null() {
        HeapFree(GetProcessHeap(), 0, NameW as *mut core::ffi::c_void);
    }
    if !PortNameW.is_null() {
        HeapFree(GetProcessHeap(), 0, PortNameW as *mut core::ffi::c_void);
    }
    if !pi3W.pszStatus.is_null() {
        HeapFree(GetProcessHeap(), 0, pi3W.pszStatus as *mut core::ffi::c_void);
    }

    return res;
}

#[no_mangle]
pub unsafe extern "system" fn SetPortW(pName: PWSTR, pPortName: PWSTR, dwLevel: DWORD, pPortInfo: PBYTE) -> BOOL {
    let mut dwErrorCode: DWORD;
    let mut PortInfoContainer: WINSPOOL_PORT_CONTAINER = core::mem::zeroed();

    if dwLevel != 3 {
        SetLastError(ERROR_INVALID_LEVEL);
        return FALSE as BOOL;
    }

    PortInfoContainer.PortInfo.pPortInfo3 = pPortInfo as *mut WINSPOOL_PORT_INFO_3;
    PortInfoContainer.Level = dwLevel;

    // RpcTryExcept
    let dwErrorCode: DWORD = rpc_try(|| _RpcSetPort(pName, pPortName, core::ptr::addr_of_mut!(PortInfoContainer))).unwrap_or_else(|code| code as _);
    // RpcExcept handler: dead under this tree's USE_DUMMY_PSEH build, and stable Rust has no SEH.
    // Original: dwErrorCode = RpcExceptionCode();
    // RpcEndExcept;

    SetLastError(dwErrorCode);
    return (dwErrorCode == ERROR_SUCCESS) as BOOL;
}
