use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type FN_SetProcessShutdownParameters = Option<unsafe extern "system" fn(DWORD, DWORD) -> BOOL>;
pub type FN_GetSystemWow64DirectoryA = Option<unsafe extern "system" fn(LPSTR, UINT) -> UINT>;
pub type FN_GetSystemWow64DirectoryW = Option<unsafe extern "system" fn(LPWSTR, UINT) -> UINT>;

pub const CP_ACP: UINT = 0;

#[allow(non_upper_case_globals)]
pub static mut g_fnSetProcessShutdownParameters: FN_SetProcessShutdownParameters = None;
#[allow(non_upper_case_globals)]
pub static mut g_fnGetSystemWow64DirectoryA: FN_GetSystemWow64DirectoryA = None;
#[allow(non_upper_case_globals)]
pub static mut g_fnGetSystemWow64DirectoryW: FN_GetSystemWow64DirectoryW = None;
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut g_hInst: HINSTANCE = core::ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut g_hKernel32: HINSTANCE = core::ptr::null_mut();
#[allow(non_upper_case_globals)]
pub static mut g_uACP: UINT = CP_ACP;
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut g_fWinLogon: BOOL = 0;
#[allow(non_upper_case_globals)]
pub static mut g_hCicMutex: HANDLE = core::ptr::null_mut();
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut g_bOnWow64: BOOL = 0;
#[allow(non_upper_case_globals)]
pub static mut g_fNoRunKey: BOOL = 0;
#[allow(non_upper_case_globals)]
pub static mut g_fJustRunKey: BOOL = 0;
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut g_dwOsInfo: DWORD = 0;
#[allow(non_upper_case_globals)]
pub static mut g_pLoaderWnd: *mut CLoaderWnd = core::ptr::null_mut();

unsafe extern "C" fn ParseCommandLine(pszCmdLine: LPCTSTR) {
    g_fJustRunKey = (FALSE) as i32;
    g_fNoRunKey = (FALSE) as i32;

    let mut pch: LPCTSTR = pszCmdLine;
    while *pch != 0 {
        // Skip space
        while *pch == (' ' as u16) {
            pch = pch.add(1);
        }

        if *pch == ('\0' as u16) {
            return;
        }

        if (*pch == ('-' as u16)) || (*pch == ('/' as u16)) {
            pch = pch.add(1);
            let sw: TCHAR = *pch;
            // switch (*pch): each C `break` ends the switch; no statement follows it in its case, so the arm simply ends.
            if sw == ('N' as u16) || sw == ('n' as u16) {
                // Found "/N" option
                g_fNoRunKey = (TRUE) as i32;
            } else if sw == ('R' as u16) || sw == ('r' as u16) {
                // Found "/R" option
                g_fJustRunKey = (TRUE) as i32;
            } else if sw == UNICODE_NULL {
                return;
            } else {
                // default: break;
            }
        }

        pch = pch.add(1);
    }
}

unsafe extern "C" fn WriteRegRun() {
    if g_fNoRunKey != 0 {
        // If "/N" option is specified
        return; // Don't write
    }

    // Open "Run" key
    let mut hKey: HKEY = core::mem::zeroed();
    let runKey: [u16; 46] = [
        'S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16, 'e' as u16,
        '\\' as u16,
        'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16,
        '\\' as u16,
        'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16,
        '\\' as u16,
        'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16,
        'V' as u16, 'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16,
        '\\' as u16,
        'R' as u16, 'u' as u16, 'n' as u16,
        0,
    ];
    let error: LSTATUS = RegCreateKey(
        HKEY_CURRENT_USER,
        runKey.as_ptr(),
        core::ptr::addr_of_mut!(hKey),
    );
    if (error) as u32 != ERROR_SUCCESS {
        return;
    }

    // Write the module path
    let mut ModPath: CicSystemModulePath = core::mem::zeroed();
    let ctfmonName: [u16; 11] = [
        'c' as u16, 't' as u16, 'f' as u16, 'm' as u16, 'o' as u16, 'n' as u16, '.' as u16,
        'e' as u16, 'x' as u16, 'e' as u16,
        0,
    ];
    if ModPath.Init(ctfmonName.as_ptr(), FALSE as BOOL) != 0 {
        let cbData: DWORD = ((ModPath.m_cchPath + 1) as DWORD)
            * (core::mem::size_of::<TCHAR>() as DWORD);
        RegSetValueEx(
            hKey,
            ctfmonName.as_ptr(),
            0,
            REG_SZ,
            core::ptr::addr_of_mut!(ModPath.m_szPath) as *mut BYTE,
            cbData,
        );
    }

    RegCloseKey(hKey);
}

unsafe extern "C" fn GetGlobalCompartment(guid: REFGUID, ppComp: *mut *mut ITfCompartment) -> HRESULT {
    *ppComp = core::ptr::null_mut();

    let mut pCompMgr: *mut ITfCompartmentMgr = core::ptr::null_mut();
    let mut hr: HRESULT = TF_GetGlobalCompartment(core::ptr::addr_of_mut!(pCompMgr));
    if hr < 0 {
        return hr;
    }

    if pCompMgr.is_null() {
        return E_FAIL;
    }

    hr = (*pCompMgr).GetCompartment(guid, ppComp);
    (*pCompMgr).Release();
    return hr;
}

unsafe extern "C" fn SetGlobalCompartmentDWORD(guid: REFGUID, dwValue: DWORD) -> HRESULT {
    let mut hr: HRESULT;
    let mut vari: VARIANT = core::mem::zeroed();
    let mut pComp: *mut ITfCompartment = core::ptr::null_mut();

    hr = GetGlobalCompartment(guid, core::ptr::addr_of_mut!(pComp));
    if (hr) < 0 {
        return hr;
    }

    V_VT!(core::ptr::addr_of_mut!(vari)) = VT_I4 as _;
    V_I4!(core::ptr::addr_of_mut!(vari)) = dwValue as _;
    hr = (*pComp).SetValue(0, core::ptr::addr_of_mut!(vari));

    (*pComp).Release();
    return hr;
}
unsafe extern "C" fn CheckX64System(lpCmdLine: LPTSTR) -> BOOL {
    // Is the system x64?
    let mut SystemInfo: SYSTEM_INFO = core::mem::zeroed();
    GetSystemInfo(core::ptr::addr_of_mut!(SystemInfo));
    // Kept as the C has it, and it is a tautology: an architecture is always unequal to ONE of
    // IA64 and AMD64, so this always returns FALSE and the WoW64 ctfmon below never starts.
    // `&&` is what was meant, but correcting it switches on a never-run path at every amd64
    // login (spawning a 32-bit ctfmon); that is a feature decision, not a translation fix.
    if (SystemInfo.__bindgen_anon_1.__bindgen_anon_1.wProcessorArchitecture) as u32 != PROCESSOR_ARCHITECTURE_IA64 ||
        (SystemInfo.__bindgen_anon_1.__bindgen_anon_1.wProcessorArchitecture) as u32 != PROCESSOR_ARCHITECTURE_AMD64
    {
        return (FALSE) as BOOL;
    }

    // Get GetSystemWow64DirectoryW function
    g_hKernel32 = cicGetSystemModuleHandle((&[0x006b, 0x0065, 0x0072, 0x006e, 0x0065, 0x006c, 0x0033, 0x0032, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000u16]).as_ptr() /* TEXT "kernel32.dll" */, (FALSE) as BOOL);
    // The C's UNICODE branch: TCHAR is WCHAR here, so the A variant would get a wide buffer.
    g_fnGetSystemWow64DirectoryW = core::mem::transmute::<_, FN_GetSystemWow64DirectoryW>(
        GetProcAddress(g_hKernel32, b"GetSystemWow64DirectoryW\0".as_ptr() as *const i8),
    );
    if g_fnGetSystemWow64DirectoryW.is_none() {
        return (FALSE) as BOOL;
    }

    // Build WoW64 ctfmon.exe pathname
    let mut szPath: [TCHAR; (MAX_PATH) as _] = core::mem::zeroed();
    let cchPath: UINT = (g_fnGetSystemWow64DirectoryW.unwrap_unchecked())(szPath.as_mut_ptr(), _countof!(szPath) as UINT);
    if cchPath == 0 && ((StringCchCat(szPath.as_mut_ptr(), (_countof!(szPath)) as _, (&[0x005c, 0x0063, 0x0074, 0x0066, 0x006d, 0x006f, 0x006e, 0x002e, 0x0065, 0x0078, 0x0065, 0x0000u16]).as_ptr() /* TEXT "\\ctfmon.exe" */)) < 0) {
        return (FALSE) as BOOL;
    }

    // Create a WoW64 ctfmon.exe process
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut si: STARTUPINFO = core::mem::zeroed();
    si.cb = core::mem::size_of::<STARTUPINFO>() as DWORD;
    si.wShowWindow = (SW_SHOWMINNOACTIVE) as u16;
    if CreateProcess(
        szPath.as_mut_ptr(),
        lpCmdLine,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        FALSE as BOOL,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(si),
        core::ptr::addr_of_mut!(pi),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn InitApp(hInstance: HINSTANCE, lpCmdLine: LPTSTR) -> BOOL {
    g_hInst = hInstance; // Save the instance handle

    g_bOnWow64 = cicIsWow64(); // Is the current process on WoW64?
    cicGetOSInfo(core::ptr::addr_of_mut!(g_uACP), core::ptr::addr_of_mut!(g_dwOsInfo)); // Get OS info

    // Create a mutex for Cicero
    g_hCicMutex = TF_CreateCicLoadMutex(core::ptr::addr_of_mut!(g_fWinLogon));
    if g_hCicMutex.is_null() {
        return (FALSE) as BOOL;
    }

    // Write to "Run" registry key for starting up
    WriteRegRun();

    // Call SetProcessShutdownParameters if possible
    if (g_dwOsInfo & CIC_OSINFO_NT) != 0
    {
        g_hKernel32 = cicGetSystemModuleHandle((&[0x006b, 0x0065, 0x0072, 0x006e, 0x0065, 0x006c, 0x0033, 0x0032, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000u16]).as_ptr() /* TEXT "kernel32.dll" */, (FALSE) as BOOL);
        g_fnSetProcessShutdownParameters =
            core::mem::transmute(
                GetProcAddress(g_hKernel32, b"SetProcessShutdownParameters\0".as_ptr() as *const i8));
        if g_fnSetProcessShutdownParameters.is_some()
        {
            (g_fnSetProcessShutdownParameters.unwrap_unchecked())(0xF0, SHUTDOWN_NORETRY);
        }
    }

    // Start text framework
    TF_InitSystem();

    // Start watching registry if x86/x64 native
    if g_bOnWow64 == 0
    {
        CRegWatcher::Init();
    }

    // Create Tipbar loader window
    g_pLoaderWnd = CLoaderWnd__new_in_0(cpp_alloc_CicNoThrow::<CLoaderWnd>());
    if g_pLoaderWnd.is_null() || (*g_pLoaderWnd).Init() == 0
    {
        return (FALSE) as BOOL;
    }

    if (*g_pLoaderWnd).CreateWnd() != core::ptr::null_mut()
    {
        // Go to the bottom of the (s)hell
        SetWindowPos((*g_pLoaderWnd).m_hWnd, HWND_BOTTOM, 0, 0, 0, 0,
                       SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE);
    }

    // Display Tipbar Popup if x86/x64 native and necessary
    if g_bOnWow64 == 0
    {
        GetPopupTipbar((*g_pLoaderWnd).m_hWnd, g_fWinLogon);
    }

    // Initialize x64-specific support
    CheckX64System(lpCmdLine);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn UninitApp() {
    // Close Tipbar Popup
    ClosePopupTipbar();

    // Close the mutex
    CloseHandle(g_hCicMutex);
    g_hCicMutex = core::ptr::null_mut();

    // Quit watching registry if x86/x64 native
    if g_bOnWow64 == 0 {
        CRegWatcher::Uninit();
    }
}

unsafe extern "C" fn DoMainLoop() -> INT {
    let mut msg: MSG = core::mem::zeroed();

    if g_bOnWow64 != 0 {
        // Simply run a message loop
        while GetMessage(&mut msg, core::ptr::null_mut(), 0, 0) != 0 {
            TranslateMessage(&mut msg);
            DispatchMessage(&mut msg);
        }
        return msg.wParam as INT;
    }

    // We wait on the CRegWatcher and the WinSta0 desktop-switch events.
    let mut ahEvents: [HANDLE; (WI_REGEVTS_MAX + 1) as _] = core::mem::zeroed();
    let mut dwEventCount: DWORD;

    // Get the CRegWatcher handles
    CopyMemory!(
        ahEvents.as_mut_ptr() as *mut _,
        CRegWatcher__s_ahWatchEvents.as_ptr() as *const _,
        (WI_REGEVTS_MAX as usize) * core::mem::size_of::<HANDLE>(),
    );
    dwEventCount = WI_REGEVTS_MAX;

    // Open the WinSta0 desktop-switch event and add it
    let hSwitchEvent: HANDLE = OpenEvent(SYNCHRONIZE, (FALSE) as i32, (['W' as u16, 'i' as u16, 'n' as u16, 'S' as u16, 't' as u16, 'a' as u16, '0' as u16, '_' as u16, 'D' as u16, 'e' as u16, 's' as u16, 'k' as u16, 't' as u16, 'o' as u16, 'p' as u16, 'S' as u16, 'w' as u16, 'i' as u16, 't' as u16, 'c' as u16, 'h' as u16, 0u16].as_ptr()));
    if !hSwitchEvent.is_null() {
        ahEvents[(WI_DESKTOP_SWITCH) as usize] = hSwitchEvent;
        dwEventCount += 1;
    }

    // Run the event loop
    'quit: {
        loop {
            let dwWait: DWORD = MsgWaitForMultipleObjects(
                dwEventCount,
                ahEvents.as_ptr(),
                (FALSE) as BOOL,
                INFINITE,
                QS_ALLINPUT,
            );
            if dwWait == (WAIT_OBJECT_0 + dwEventCount) {
                // Is input available?
                // Pump available messages
                while PeekMessage(&mut msg, core::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    if msg.message == WM_QUIT {
                        break 'quit;
                    }

                    TranslateMessage(&mut msg);
                    DispatchMessage(&mut msg);
                }
            } else if dwWait == (WAIT_OBJECT_0 + WI_DESKTOP_SWITCH) {
                // Desktop switch?
                SetGlobalCompartmentDWORD(&GUID_COMPARTMENT_SPEECH_OPENCLOSE, 0);
                ResetEvent(hSwitchEvent);
            } else if dwWait < (WAIT_OBJECT_0 + WI_REGEVTS_MAX) {
                // Do the other events
                CRegWatcher::OnEvent((dwWait - WAIT_OBJECT_0) as SIZE_T);
            } else if dwWait == WAIT_TIMEOUT {
                // Ignore
                OutputDebugStringA(b"DoMainLoop: WAIT_TIMEOUT\n\0".as_ptr() as *const i8);
            } else if dwWait == WAIT_FAILED {
                // Something failed, bail out
                OutputDebugStringA(b"DoMainLoop: WAIT_FAILED, exiting\n\0".as_ptr() as *const i8);
                break;
            }
        }
    }

    // Quit:
    if !hSwitchEvent.is_null() {
        CloseHandle(hSwitchEvent);
    }

    return msg.wParam as INT;
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(hInstance: HINSTANCE, hPrevInst: HINSTANCE, lpCmdLine: LPTSTR, nCmdShow: i32) -> i32 {
    UNREFERENCED_PARAMETER!(hPrevInst);
    UNREFERENCED_PARAMETER!(nCmdShow);

    // Parse command line
    ParseCommandLine(lpCmdLine);

    if g_fJustRunKey != 0 // If "/R" option is specified
    {
        // Just write registry and exit
        WriteRegRun();
        return 1;
    }

    // Initialize the application
    if !(InitApp(hInstance, lpCmdLine) != 0)
    {
        return 0;
    }

    // The main loop
    let ret: i32 = DoMainLoop();

    // Clean up the loader
    if !g_pLoaderWnd.is_null()
    {
        CLoaderWnd__delete(g_pLoaderWnd);
        g_pLoaderWnd = core::ptr::null_mut();
    }

    // Un-initialize app and text framework
    if CLoaderWnd__s_bUninitedSystem == 0
    {
        UninitApp();
        TF_UninitSystem();
    }

    ret
}
