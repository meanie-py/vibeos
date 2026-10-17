use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::prelude::EnumWindows;
use core::ffi::c_void;

extern "system" {
}

// ---- file-scope preamble (translated) ----
pub type FN_TF_CreateLangProfileUtil = Option<unsafe extern "system" fn(*mut *mut ITfFnLangProfileUtil) -> HRESULT>;
pub type FN_RegNotifyChangeKeyValue = Option<unsafe extern "system" fn(HKEY, BOOL, DWORD, HANDLE, BOOL) -> LONG>;

pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001usize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002usize as HKEY;

pub static mut CRegWatcher__s_ahWatchEvents: [HANDLE; 12] = [core::ptr::null_mut(); 12];

// (&[0x002e, 0x002e, 0x002e, 0x0000u16]).as_ptr() /* TEXT "..." */ literals as NUL-terminated UTF-16 slices.
pub static WE_TXT_0: &[u16] = &['K' as u16, 'e' as u16, 'y' as u16, 'b' as u16, 'o' as u16, 'a' as u16, 'r' as u16, 'd' as u16, ' ' as u16, 'L' as u16, 'a' as u16, 'y' as u16, 'o' as u16, 'u' as u16, 't' as u16, '\\' as u16, 'T' as u16, 'o' as u16, 'g' as u16, 'g' as u16, 'l' as u16, 'e' as u16, 0];
pub static WE_TXT_1: &[u16] = &['S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'C' as u16, 'T' as u16, 'F' as u16, '\\' as u16, 'T' as u16, 'I' as u16, 'P' as u16, 0];
pub static WE_TXT_2: &[u16] = &['K' as u16, 'e' as u16, 'y' as u16, 'b' as u16, 'o' as u16, 'a' as u16, 'r' as u16, 'd' as u16, ' ' as u16, 'L' as u16, 'a' as u16, 'y' as u16, 'o' as u16, 'u' as u16, 't' as u16, '\\' as u16, 'P' as u16, 'r' as u16, 'e' as u16, 'l' as u16, 'o' as u16, 'a' as u16, 'd' as u16, 0];
pub static WE_TXT_3: &[u16] = &['S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16, '\\' as u16, 'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16, 'V' as u16, 'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16, 'n' as u16, '\\' as u16, 'R' as u16, 'u' as u16, 'n' as u16, 0];
pub static WE_TXT_4: &[u16] = &['S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'C' as u16, 'T' as u16, 'F' as u16, '\\' as u16, 'T' as u16, 'I' as u16, 'P' as u16, 0];
pub static WE_TXT_5: &[u16] = &['S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'S' as u16, 'p' as u16, 'e' as u16, 'e' as u16, 'c' as u16, 'h' as u16, 0];
pub static WE_TXT_6: &[u16] = &['C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, 'P' as u16, 'a' as u16, 'n' as u16, 'e' as u16, 'l' as u16, '\\' as u16, 'A' as u16, 'p' as u16, 'p' as u16, 'e' as u16, 'a' as u16, 'r' as u16, 'a' as u16, 'n' as u16, 'c' as u16, 'e' as u16, 0];
pub static WE_TXT_7: &[u16] = &['C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, 'P' as u16, 'a' as u16, 'n' as u16, 'e' as u16, 'l' as u16, '\\' as u16, 'C' as u16, 'o' as u16, 'l' as u16, 'o' as u16, 'r' as u16, 's' as u16, 0];
pub static WE_TXT_8: &[u16] = &['C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16, ' ' as u16, 'P' as u16, 'a' as u16, 'n' as u16, 'e' as u16, 'l' as u16, '\\' as u16, 'D' as u16, 'e' as u16, 's' as u16, 'k' as u16, 't' as u16, 'o' as u16, 'p' as u16, '\\' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 'M' as u16, 'e' as u16, 't' as u16, 'r' as u16, 'i' as u16, 'c' as u16, 's' as u16, 0];
pub static WE_TXT_9: &[u16] = &['S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'S' as u16, 'p' as u16, 'e' as u16, 'e' as u16, 'c' as u16, 'h' as u16, 0];
pub static WE_TXT_10: &[u16] = &['K' as u16, 'e' as u16, 'y' as u16, 'b' as u16, 'o' as u16, 'a' as u16, 'r' as u16, 'd' as u16, ' ' as u16, 'L' as u16, 'a' as u16, 'y' as u16, 'o' as u16, 'u' as u16, 't' as u16, 0];
pub static WE_TXT_11: &[u16] = &['S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16, '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16, '\\' as u16, 'C' as u16, 'T' as u16, 'F' as u16, '\\' as u16, 'A' as u16, 's' as u16, 's' as u16, 'e' as u16, 'm' as u16, 'b' as u16, 'l' as u16, 'i' as u16, 'e' as u16, 's' as u16, 0];

// Positional in the C: { hRootKey, pszSubKey }, hKey (the opened key) starting NULL.
pub static mut CRegWatcher__s_WatchEntries: [WATCHENTRY; 12] = [
    WATCHENTRY { hRootKey: HKEY_CURRENT_USER, pszSubKey: WE_TXT_0.as_ptr() as *const TCHAR, hKey: core::ptr::null_mut() },  // WI_TOGGLE
    WATCHENTRY { hRootKey: HKEY_LOCAL_MACHINE, pszSubKey: WE_TXT_1.as_ptr() as *const TCHAR, hKey: core::ptr::null_mut() }, // WI_MACHINE_TIF
    WATCHENTRY { hRootKey: HKEY_CURRENT_USER, pszSubKey: WE_TXT_2.as_ptr() as *const TCHAR, hKey: core::ptr::null_mut() },  // WI_PRELOAD
    WATCHENTRY { hRootKey: HKEY_CURRENT_USER, pszSubKey: WE_TXT_3.as_ptr() as *const TCHAR, hKey: core::ptr::null_mut() },  // WI_RUN
    WATCHENTRY { hRootKey: HKEY_CURRENT_USER, pszSubKey: WE_TXT_4.as_ptr() as *const TCHAR, hKey: core::ptr::null_mut() },  // WI_USER_TIF
    WATCHENTRY { hRootKey: HKEY_CURRENT_USER, pszSubKey: WE_TXT_5.as_ptr() as *const TCHAR, hKey: core::ptr::null_mut() },  // WI_USER_SPEECH
    WATCHENTRY { hRootKey: HKEY_CURRENT_USER, pszSubKey: WE_TXT_6.as_ptr() as *const TCHAR, hKey: core::ptr::null_mut() },  // WI_APPEARANCE
    WATCHENTRY { hRootKey: HKEY_CURRENT_USER, pszSubKey: WE_TXT_7.as_ptr() as *const TCHAR, hKey: core::ptr::null_mut() },  // WI_COLORS
    WATCHENTRY { hRootKey: HKEY_CURRENT_USER, pszSubKey: WE_TXT_8.as_ptr() as *const TCHAR, hKey: core::ptr::null_mut() },  // WI_WINDOW_METRICS
    WATCHENTRY { hRootKey: HKEY_LOCAL_MACHINE, pszSubKey: WE_TXT_9.as_ptr() as *const TCHAR, hKey: core::ptr::null_mut() }, // WI_MACHINE_SPEECH
    WATCHENTRY { hRootKey: HKEY_CURRENT_USER, pszSubKey: WE_TXT_10.as_ptr() as *const TCHAR, hKey: core::ptr::null_mut() }, // WI_KEYBOARD_LAYOUT
    WATCHENTRY { hRootKey: HKEY_CURRENT_USER, pszSubKey: WE_TXT_11.as_ptr() as *const TCHAR, hKey: core::ptr::null_mut() }, // WI_ASSEMBLIES
];

pub static mut CRegWatcher__s_nSysColorTimerId: UINT = 0;
pub static mut CRegWatcher__s_nKbdToggleTimerId: UINT = 0;
pub static mut CRegWatcher__s_nRegImxTimerId: UINT = 0;











#[no_mangle]
pub unsafe extern "system" fn DelayedRegNotifyChangeKeyValue(hKey: HKEY, bWatchSubtree: BOOL, dwNotifyFilter: DWORD, hEvent: HANDLE, fAsynchronous: BOOL) -> LONG {
    static mut s_fnRegNotifyChangeKeyValue: FN_RegNotifyChangeKeyValue = None;

    if s_fnRegNotifyChangeKeyValue.is_none() {
        let hAdvApi32: HINSTANCE = cicGetSystemModuleHandle((&[0x0061, 0x0064, 0x0076, 0x0061, 0x0070, 0x0069, 0x0033, 0x0032, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000u16]).as_ptr() /* TEXT "advapi32.dll" */, (FALSE) as BOOL);
        s_fnRegNotifyChangeKeyValue = core::mem::transmute(GetProcAddress(hAdvApi32, b"RegNotifyChangeKeyValue\0".as_ptr() as *const i8));
        if s_fnRegNotifyChangeKeyValue.is_none() {
            return (ERROR_CALL_NOT_IMPLEMENTED) as LONG;
        }
    }

    return (s_fnRegNotifyChangeKeyValue.unwrap_unchecked())(hKey, bWatchSubtree, dwNotifyFilter, hEvent, fAsynchronous);
}

impl CRegWatcher {
    pub unsafe fn Init() -> i32 {
        // NOTE: We don't support Win95/98/Me
    
        // Create some nameless events and initialize them
        let mut iEvent: usize = 0;
        while iEvent < _countof!(CRegWatcher__s_ahWatchEvents) as usize {
            CRegWatcher__s_ahWatchEvents[iEvent] = CreateEvent(core::ptr::null_mut(), (TRUE) as i32, (FALSE) as i32, core::ptr::null_mut());
            CRegWatcher::InitEvent(iEvent, (FALSE) as i32);
            iEvent += 1;
        }
    
        // Internat.exe is an enemy of ctfmon.exe
        CRegWatcher::KillInternat();
    
        CRegWatcher::UpdateSpTip();
    
        return (TRUE) as i32;
    }
    pub unsafe fn Uninit() {
        let mut iEvent: usize = 0;
        while iEvent < (_countof!(CRegWatcher__s_ahWatchEvents) as usize) {
            // Close the key
            let entry: &mut WATCHENTRY = &mut (*core::ptr::addr_of_mut!(CRegWatcher__s_WatchEntries))[iEvent];
            if !entry.hKey.is_null() {
                RegCloseKey(entry.hKey);
                entry.hKey = core::ptr::null_mut();
            }
    
            // Close the event handle
            let hEvent: &mut HANDLE = &mut (*core::ptr::addr_of_mut!(CRegWatcher__s_ahWatchEvents))[iEvent];
            if !hEvent.is_null() {
                CloseHandle(*hEvent);
                *hEvent = core::ptr::null_mut();
            }
            iEvent += 1;
        }
    }
    pub unsafe fn InitEvent(iEvent: usize, bResetEvent: i32) -> i32 {
        if iEvent >= _countof!(CRegWatcher__s_ahWatchEvents) as usize {
            return (FALSE) as i32;
        }
    
        // Reset the signal status
        if bResetEvent != 0 {
            ResetEvent(CRegWatcher__s_ahWatchEvents[iEvent]);
        }
    
        // Close once to re-open
        let entry: &mut WATCHENTRY = &mut *core::ptr::addr_of_mut!(CRegWatcher__s_WatchEntries[iEvent]);
        if !entry.hKey.is_null() {
            RegCloseKey(entry.hKey);
            entry.hKey = core::ptr::null_mut();
        }
    
        // Open or create a registry key to watch registry key
        let mut error: LSTATUS;
        error = RegOpenKeyEx(
            entry.hRootKey,
            entry.pszSubKey,
            0,
            KEY_READ,
            core::ptr::addr_of_mut!(entry.hKey),
        );
        if (error) as u32 != ERROR_SUCCESS {
            error = RegCreateKeyEx(
                entry.hRootKey,
                entry.pszSubKey,
                0,
                core::ptr::null_mut(),
                0,
                KEY_ALL_ACCESS,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(entry.hKey),
                core::ptr::null_mut(),
            );
            if (error) as u32 != ERROR_SUCCESS {
                return (FALSE) as i32;
            }
        }
    
        // Start registry watching
        error = DelayedRegNotifyChangeKeyValue(
            entry.hKey,
            (TRUE) as BOOL,
            REG_NOTIFY_CHANGE_LAST_SET | REG_NOTIFY_CHANGE_NAME,
            CRegWatcher__s_ahWatchEvents[iEvent],
            (TRUE) as BOOL,
        );
        if (error) as u32 != ERROR_SUCCESS {
            OutputDebugStringA(b"RegNotifyChangeKeyValue failed\n\0".as_ptr() as *const i8);
        }
        return ((error) as u32 == ERROR_SUCCESS) as i32;
    }
    pub unsafe fn UpdateSpTip() {
        // Post message 0x8002 to "SapiTipWorkerClass" windows
        EnumWindows(Some(CRegWatcher::EnumWndProc), 0);
    
        // Clear "ProfileInitialized" value
        let mut hKey: HKEY = core::ptr::null_mut();
        let error: LSTATUS = RegOpenKeyEx(
            HKEY_CURRENT_USER,
            (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0043, 0x0054, 0x0046, 0x005c, 0x0053, 0x0061, 0x0070, 0x0069, 0x006c, 0x0061, 0x0079, 0x0072, 0x0000u16]).as_ptr() /* TEXT "SOFTWARE\\Microsoft\\CTF\\Sapilayr" */,
            0,
            KEY_WRITE,
            core::ptr::addr_of_mut!(hKey),
        );
        if (error) as u32 == ERROR_SUCCESS {
            let mut dwValue: DWORD = 0;
            let mut cbValue: DWORD = core::mem::size_of::<DWORD>() as DWORD;
            RegSetValueEx(
                hKey,
                (&[0x0050, 0x0072, 0x006f, 0x0066, 0x0069, 0x006c, 0x0065, 0x0049, 0x006e, 0x0069, 0x0074, 0x0069, 0x0061, 0x006c, 0x0069, 0x007a, 0x0065, 0x0064, 0x0000u16]).as_ptr() /* TEXT "ProfileInitialized" */,
                0, // Reserved: the C passes NULL to a DWORD
                REG_DWORD,
                core::ptr::addr_of_mut!(dwValue) as LPBYTE,
                cbValue,
            );
            RegCloseKey(hKey);
        }
    
        // Get %WINDIR%/IME/sptip.dll!TF_CreateLangProfileUtil function
        let hSPTIP: HINSTANCE = cicLoadSystemLibrary((&[0x0049, 0x004d, 0x0045, 0x005c, 0x0073, 0x0070, 0x0074, 0x0069, 0x0070, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000u16]).as_ptr() /* TEXT "IME\\sptip.dll" */, (TRUE) as BOOL);
        let FpRaw: *const core::ffi::c_void = (GetProcAddress(
            hSPTIP,
            b"TF_CreateLangProfileUtil\0".as_ptr() as *const i8,
        )).map_or(core::ptr::null(), |f| f as *const core::ffi::c_void);
        let fnTF_CreateLangProfileUtil: FN_TF_CreateLangProfileUtil =
            core::mem::transmute::<*const core::ffi::c_void, FN_TF_CreateLangProfileUtil>(FpRaw);
        if !FpRaw.is_null() {
            // Call it
            let mut pProfileUtil: *mut ITfFnLangProfileUtil = core::ptr::null_mut();
            let mut hr: HRESULT = (fnTF_CreateLangProfileUtil.unwrap_unchecked())(core::ptr::addr_of_mut!(pProfileUtil));
            if (hr == S_OK) && !pProfileUtil.is_null() {
                // Success!
                // Register profile
                hr = (*pProfileUtil).RegisterActiveProfiles();
                if hr == S_OK {
                    TF_InvalidAssemblyListCacheIfExist(); // Invalidate the assembly list cache
                }
    
                (*pProfileUtil).Release();
            }
        }
    
        if !hSPTIP.is_null() {
            FreeLibrary(hSPTIP);
        }
    }
    pub unsafe fn KillInternat() {
        let mut hKey: HKEY = core::ptr::null_mut();
        let entry: &mut WATCHENTRY = &mut CRegWatcher__s_WatchEntries[WI_RUN as usize];
    
        // Delete internat.exe from registry "Run" key
        let error: LSTATUS = RegOpenKeyEx(entry.hRootKey, entry.pszSubKey, 0, KEY_ALL_ACCESS, core::ptr::addr_of_mut!(hKey));
        if (error) as u32 == ERROR_SUCCESS {
            RegDeleteValue(hKey, (&[0x0069, 0x006e, 0x0074, 0x0065, 0x0072, 0x006e, 0x0061, 0x0074, 0x002e, 0x0065, 0x0078, 0x0065, 0x0000u16]).as_ptr() /* TEXT "internat.exe" */);
            RegCloseKey(hKey);
        }
    
        // Kill the "Indicator" window (that internat.exe creates)
        let hwndInternat: HWND = FindWindow(INDICATOR_CLASS.as_ptr(), core::ptr::null_mut());
        if !hwndInternat.is_null() {
            PostMessage(hwndInternat, WM_CLOSE, 0, 0);
        }
    }
    pub unsafe fn StartSysColorChangeTimer() {
        // Call SysColorTimerProc 0.5 seconds later (Delayed)
        if CRegWatcher__s_nSysColorTimerId != 0 {
            KillTimer(core::ptr::null_mut(), (CRegWatcher__s_nSysColorTimerId) as UINT_PTR);
            CRegWatcher__s_nSysColorTimerId = 0;
        }
        CRegWatcher__s_nSysColorTimerId = (SetTimer(core::ptr::null_mut(), 0, 500, Some(CRegWatcher::SysColorTimerProc))) as u32;
    }
    pub unsafe fn OnEvent(iEvent: SIZE_T) {
        if iEvent >= _countof!(CRegWatcher__s_ahWatchEvents) as SIZE_T {
            return;
        }
    
        CRegWatcher::InitEvent((iEvent) as _, (TRUE) as i32);
    
        match iEvent {
            x if x == WI_TOGGLE as SIZE_T => {
                // Call KbdToggleTimerProc 0.5 seconds later (Delayed)
                if CRegWatcher__s_nKbdToggleTimerId != 0 {
                    KillTimer(core::ptr::null_mut(), (CRegWatcher__s_nKbdToggleTimerId) as UINT_PTR);
                    CRegWatcher__s_nKbdToggleTimerId = 0;
                }
                CRegWatcher__s_nKbdToggleTimerId = (SetTimer(
                    core::ptr::null_mut(),
                    0,
                    500,
                    Some(CRegWatcher::KbdToggleTimerProc),
                )) as u32;
            }
            x if x == WI_MACHINE_TIF as SIZE_T
                || x == WI_PRELOAD as SIZE_T
                || x == WI_USER_TIF as SIZE_T
                || x == WI_MACHINE_SPEECH as SIZE_T
                || x == WI_KEYBOARD_LAYOUT as SIZE_T
                || x == WI_ASSEMBLIES as SIZE_T =>
            {
                if iEvent == (WI_MACHINE_SPEECH) as _ {
                    CRegWatcher::UpdateSpTip();
                }
    
                // Call RegImxTimerProc 0.2 seconds later (Delayed)
                if CRegWatcher__s_nRegImxTimerId != 0 {
                    KillTimer(core::ptr::null_mut(), (CRegWatcher__s_nRegImxTimerId) as UINT_PTR);
                    CRegWatcher__s_nRegImxTimerId = 0;
                }
                CRegWatcher__s_nRegImxTimerId = (SetTimer(
                    core::ptr::null_mut(),
                    0,
                    200,
                    Some(CRegWatcher::RegImxTimerProc),
                )) as u32;
            }
            x if x == WI_RUN as SIZE_T => {
                // The "Run" key is changed
                CRegWatcher::KillInternat(); // Deny internat.exe the right to live
            }
            x if x == WI_USER_SPEECH as SIZE_T
                || x == WI_APPEARANCE as SIZE_T
                || x == WI_COLORS as SIZE_T
                || x == WI_WINDOW_METRICS as SIZE_T =>
            {
                CRegWatcher::StartSysColorChangeTimer();
            }
            _ => {}
        }
    }
    pub unsafe extern "system" fn EnumWndProc(hWnd: HWND, lParam: LPARAM) -> BOOL {
        let mut ClassName: [TCHAR; (MAX_PATH) as _] = [0 as TCHAR; (MAX_PATH) as _];
    
        UNREFERENCED_PARAMETER!(lParam);
    
        if GetClassName(hWnd, ClassName.as_mut_ptr(), _countof!(ClassName) as i32) != 0
            && _tcsicmp(ClassName.as_ptr(), (&[0x0053, 0x0061, 0x0070, 0x0069, 0x0054, 0x0069, 0x0070, 0x0057, 0x006f, 0x0072, 0x006b, 0x0065, 0x0072, 0x0043, 0x006c, 0x0061, 0x0073, 0x0073, 0x0000u16]).as_ptr() /* TEXT "SapiTipWorkerClass" */) == 0
        {
            PostMessage(hWnd, 0x8002, 0, 0); // FIXME: Magic number
        }
    
        return (TRUE) as BOOL;
    }
    pub unsafe extern "system" fn SysColorTimerProc(hwnd: HWND, uMsg: u32, idEvent: UINT_PTR, dwTime: u32) {
        UNREFERENCED_PARAMETER!(hwnd);
        UNREFERENCED_PARAMETER!(uMsg);
        UNREFERENCED_PARAMETER!(idEvent);
        UNREFERENCED_PARAMETER!(dwTime);
    
        // Cancel the timer
        if CRegWatcher__s_nSysColorTimerId != 0 {
            KillTimer(core::ptr::null_mut(), CRegWatcher__s_nSysColorTimerId as UINT_PTR);
            CRegWatcher__s_nSysColorTimerId = 0;
        }
    
        TF_PostAllThreadMsg(15, 16);
    }
    pub unsafe extern "system" fn KbdToggleTimerProc(hwnd: HWND, uMsg: UINT, idEvent: UINT_PTR, dwTime: DWORD) {
        UNREFERENCED_PARAMETER!(hwnd);
        UNREFERENCED_PARAMETER!(uMsg);
        UNREFERENCED_PARAMETER!(idEvent);
        UNREFERENCED_PARAMETER!(dwTime);
    
        // Cancel the timer
        if CRegWatcher__s_nKbdToggleTimerId != 0 {
            KillTimer(core::ptr::null_mut(), (CRegWatcher__s_nKbdToggleTimerId) as UINT_PTR);
            CRegWatcher__s_nKbdToggleTimerId = 0;
        }
    
        TF_PostAllThreadMsg(11, 16);
    }
    pub unsafe extern "system" fn RegImxTimerProc(hwnd: HWND, uMsg: UINT, idEvent: UINT_PTR, dwTime: DWORD) {
        UNREFERENCED_PARAMETER!(hwnd);
        UNREFERENCED_PARAMETER!(uMsg);
        UNREFERENCED_PARAMETER!(idEvent);
        UNREFERENCED_PARAMETER!(dwTime);
    
        // Cancel the timer
        if CRegWatcher__s_nRegImxTimerId != 0 {
            KillTimer(core::ptr::null_mut(), (CRegWatcher__s_nRegImxTimerId) as UINT_PTR);
            CRegWatcher__s_nRegImxTimerId = 0;
        }
    
        TF_InvalidAssemblyListCache();
        TF_PostAllThreadMsg(12, 16);
    }
}
