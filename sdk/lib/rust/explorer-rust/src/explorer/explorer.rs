//! explorer.cpp — the explorer.exe process entry point (`wWinMain`) plus the
//! file-scope process globals and the two startup drivers (`StartWithDesktop`,
//! `StartWithCommandLine`). This is THE boot-critical translation: if the
//! entry point or the globals-flip is wrong, nothing boots.
//!
//! ENTRY POINT: the C++ `_tWinMain` is a tchar.h macro that expands to
//! `wWinMain` in a UNICODE build, and the mingw CRT startup
//! (`sdk/lib/crt/startup/crt0_w.c`) calls `wWinMain` (WINAPI = stdcall on i386,
//! cdecl on amd64/arm64). So the Rust entry point is named `wWinMain`, NOT
//! `_tWinMain` — same proven pattern as lsass-rust (`wWinMain`) and
//! winlogon-rust (`WinMain`). `extern "system"` resolves to the correct
//! per-arch decoration (`_wWinMain@16` on i386). A pure-Rust entry point works
//! here; no C++ shell is needed (see SUBSTRATE MANIFEST in the handoff).
//!
//! GLOBALS FLIP: hExplorerInstance / hProcessHeap / hkExplorer /
//! bExplorerIsShell were DEFINED in explorer.cpp and EXTERN-declared by
//! explorer_sys::globals (for the sibling Rust modules) and by precomp.h (for
//! the still-C++ trayntfy.cpp / traydeskbtn.cpp). This module now OWNS the
//! real storage via `#[no_mangle] pub static mut`; the substrate extern block
//! and the precomp.h externs resolve to these definitions at link. C++
//! namespace-scope globals are unmangled, so plain `hExplorerInstance` etc.
//! bind on all three arches. explorer_sys::globals needs NO change.

use core::ffi::c_void;
use explorer_sys::*;
use explorer_sys::com::*;
use explorer_sys::externs::*;
use winsrv_sys::consts::*;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

// Sibling explorer-rust entry points, referenced by crate path so the ABI is
// type-checked (they are all `#[no_mangle] pub extern` — the linker sees one
// symbol; the crate path just gives us the correct Rust signature).
use crate::explorer::rshell::{InitRSHELL, _ShellDDEInit, _WinList_Init};
use crate::explorer::traywnd::{CreateTrayWindow, TrayMessageLoop};
use crate::explorer::desktop::{DesktopCreateWindow, DesktopDestroyShellWindow};
use crate::explorer::startup::{
    DoStartStartupItems, ProcessRunOnceItems, ProcessStartupItems, ReleaseStartupMutex,
};
use crate::explorer::settings::{g_TaskbarSettings, TaskbarSettings_Load};

// C-library imports beyond the winsrv / explorer_sys surface. stdcall Win32 →
// extern "system"; the DLL each resolves against (via explorer's importlibs) is
// noted. OleInitialize/OleUninitialize are also declared in desktop.rs — a
// different module, so no conflict (each is just an undefined reference the
// same ole32 export satisfies).
extern "system" {
    fn SHGetShellKey(flags: u32, path: LPCWSTR, create: BOOL) -> HKEY;      // shell32 (undoc)
    fn IsOS(dwOS: u32) -> BOOL;                                             // shlwapi
    fn PathFindFileNameW(pszPath: LPCWSTR) -> LPWSTR;                       // shlwapi
    fn StrStrIW(pszFirst: LPCWSTR, pszSrch: LPCWSTR) -> LPWSTR;            // shlwapi
    fn SHFree(pv: *mut c_void);                                             // shell32
    fn FileIconInit(bFullInit: BOOL) -> BOOL;                              // shell32 (undoc)
    fn InitCommonControls();                                               // comctl32
    fn OleInitialize(pvReserved: *mut c_void) -> HRESULT;                  // ole32
    fn OleUninitialize();                                                  // ole32
    fn SHExplorerParseCmdLine(pInfo: *mut EXPLORER_CMDLINE_PARSE_RESULTS) -> usize; // browseui
    fn SHCreateFromDesktop(parseResults: *mut EXPLORER_CMDLINE_PARSE_RESULTS) -> BOOL; // browseui
}

// ---------------------------------------------------------------------------
// File-scope process globals (were `HINSTANCE hExplorerInstance;` etc. in
// explorer.cpp). #[no_mangle] so explorer_sys::globals' extern block and the
// still-C++ precomp.h externs bind to THIS storage. See module doc.
// ---------------------------------------------------------------------------
#[no_mangle]
pub static mut hExplorerInstance: HINSTANCE = core::ptr::null_mut();
#[no_mangle]
pub static mut hProcessHeap: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut hkExplorer: HKEY = core::ptr::null_mut();
#[no_mangle]
pub static mut bExplorerIsShell: BOOL = 0;

/// `EXPLORER_CMDLINE_PARSE_RESULTS` (browseui_undoc.h) — only strPath/pidlPath/
/// pidlRoot are touched by StartWithCommandLine; the full layout is spelled out
/// so `core::mem::zeroed()` and the field offsets match the C++ struct exactly.
#[repr(C)]
pub struct EXPLORER_CMDLINE_PARSE_RESULTS {
    pub strPath: LPWSTR,
    pub pidlPath: *mut c_void, // LPITEMIDLIST
    pub dwFlags: DWORD,
    pub nCmdShow: i32,
    pub offset10_18: DWORD,
    pub offset14_1C: DWORD,
    pub offset18_20: DWORD,
    pub offset1C_24: DWORD,
    pub pidlRoot: *mut c_void, // LPITEMIDLIST
    pub clsid: GUID,
    pub guidInproc: GUID,
}

// SHGetShellKey flag bits + IsOS predicate used by InitializeServerAdminUI.
const SHKEY_ROOT_HKCU: u32 = 0x8000_0001;
const SHKEY_KEY_EXPLORER: u32 = 0x0000_0000;
const OS_ANYSERVER: u32 = 29;
const IDS_STARTUP_ERROR: u32 = 105; // base/shell/explorer/resource.h

/// `CExplorerModule gModule;` / `CAtlWinModule gWinModule;` +
/// `InitializeAtlModule`. For an EMPTY object map (BEGIN_OBJECT_MAP/
/// END_OBJECT_MAP with no OBJECT_ENTRY) and no OBJECT_ENTRY_AUTO classes,
/// `CComModule::Init` walks a terminator-only map and the empty `_AtlComModule`
/// auto-map — registering nothing — and `Term` mirrors it: both are no-ops
/// (verified against sdk/lib/atl/atlbase.h CComModule::Init/Term). The ATL
/// runtime globals `_AtlBaseModule`/`_AtlComModule`/`_AtlWinModule` are
/// `__declspec(selectany)` self-constructing (atlcore.h/atlbase.h), so they
/// exist independent of explorer.cpp, and the tier-C window classes use the
/// explorer_sys::server WndProc/registration substrate rather than ATL's
/// CAtlWinModule thunk heap. Nothing outside explorer.cpp referenced
/// `gModule`/`gWinModule` (grep-confirmed), so they need no Rust storage.
/// This preserves observable behavior exactly.
#[inline]
unsafe fn InitializeAtlModule(_hInstance: HINSTANCE, _bInitialize: BOOL) {
    // no-op — see doc comment.
}

unsafe fn InitializeServerAdminUI() {
    let hKey = SHGetShellKey(
        SHKEY_ROOT_HKCU | SHKEY_KEY_EXPLORER,
        w!("Advanced").as_ptr(),
        1,
    );
    if hKey.is_null() {
        return;
    }

    let mut value: DWORD = 0;
    let mut size: DWORD = core::mem::size_of::<DWORD>() as DWORD;
    let mut r#type: DWORD = 0;
    let error = SHGetValueW(
        hKey,
        core::ptr::null(),
        w!("ServerAdminUI").as_ptr(),
        &mut r#type,
        &mut value as *mut DWORD as *mut c_void,
        &mut size,
    );
    if error != ERROR_SUCCESS || r#type != REG_DWORD || size != core::mem::size_of::<DWORD>() as DWORD
    {
        // The value doesn't exist or is invalid — compute and apply a default.
        value = (IsOS(OS_ANYSERVER) != 0 && IsUserAnAdmin() != 0) as DWORD;
        SHSetValueW(
            hKey,
            core::ptr::null(),
            w!("ServerAdminUI").as_ptr(),
            REG_DWORD,
            &value as *const DWORD as *const c_void,
            core::mem::size_of::<DWORD>() as DWORD,
        );
        if value != 0 {
            // TODO: RegInstallW RegServerAdmin tweaks from shell32's REGINST.
            SystemParametersInfoW(
                SPI_SETKEYBOARDCUES,
                0,
                1usize as *mut c_void, // IntToPtr(TRUE)
                SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
            );
        }
    }
    RegCloseKey(hKey);
}

unsafe fn SetShellReadyEvent(lpEventName: LPCWSTR) -> BOOL {
    let hEvent = OpenEventW(EVENT_MODIFY_STATE, 0, lpEventName);
    if !hEvent.is_null() {
        SetEvent(hEvent);
        CloseHandle(hEvent);
        return 1;
    }
    0
}

unsafe fn HideMinimizedWindows(bHide: BOOL) {
    let mut mm: MINIMIZEDMETRICS = core::mem::zeroed();
    mm.cbSize = core::mem::size_of::<MINIMIZEDMETRICS>() as u32;
    if SystemParametersInfoW(
        SPI_GETMINIMIZEDMETRICS,
        core::mem::size_of::<MINIMIZEDMETRICS>() as u32,
        &mut mm as *mut _ as *mut c_void,
        0,
    ) == 0
    {
        return; // ERR("SystemParametersInfoW failed ...")
    }
    if bHide != 0 {
        mm.iArrange |= ARW_HIDE as i32;
    } else {
        mm.iArrange &= !(ARW_HIDE as i32);
    }
    SystemParametersInfoW(
        SPI_SETMINIMIZEDMETRICS,
        core::mem::size_of::<MINIMIZEDMETRICS>() as u32,
        &mut mm as *mut _ as *mut c_void,
        0,
    );
}

unsafe fn IsExplorerSystemShell() -> BOOL {
    let mut bIsSystemShell: BOOL = 1; // Assume system shell by default.
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if GetModuleFileNameW(core::ptr::null_mut(), szPath.as_mut_ptr(), MAX_PATH) == 0 {
        return 0;
    }

    let szExplorer = PathFindFileNameW(szPath.as_ptr());

    let mut hKeyWinlogon: HKEY = core::ptr::null_mut();
    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon").as_ptr(),
        0,
        KEY_READ,
        &mut hKeyWinlogon,
    ) == ERROR_SUCCESS as i32
    {
        let mut dwType: DWORD = 0;
        let mut szShell: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut cbShell: DWORD = core::mem::size_of_val(&szShell) as DWORD;

        // TODO: Add support for paths longer than MAX_PATH
        let status = RegQueryValueExW(
            hKeyWinlogon,
            w!("Shell").as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            szShell.as_mut_ptr() as LPBYTE,
            &mut cbShell,
        );
        if status == ERROR_SUCCESS as i32
            && ((dwType != REG_SZ && dwType != REG_EXPAND_SZ)
                || StrStrIW(szShell.as_ptr(), szExplorer).is_null())
        {
            bIsSystemShell = 0;
        }

        RegCloseKey(hKeyWinlogon);
    }

    bIsSystemShell
}

unsafe fn StartWithCommandLine(_hInstance: HINSTANCE) -> i32 {
    let mut b: BOOL = 0;
    let mut parseResults: EXPLORER_CMDLINE_PARSE_RESULTS = core::mem::zeroed();

    if SHExplorerParseCmdLine(&mut parseResults) != 0 {
        b = SHCreateFromDesktop(&mut parseResults);
    }

    if !parseResults.strPath.is_null() {
        SHFree(parseResults.strPath as *mut c_void);
    }
    if !parseResults.pidlPath.is_null() {
        ILFree(parseResults.pidlPath);
    }
    if !parseResults.pidlRoot.is_null() {
        ILFree(parseResults.pidlRoot);
    }

    b
}

unsafe fn StartWithDesktop(hInstance: HINSTANCE) -> i32 {
    InitializeAtlModule(hInstance, 1);

    if RegOpenKeyW(
        HKEY_CURRENT_USER,
        w!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer").as_ptr(),
        core::ptr::addr_of_mut!(hkExplorer),
    ) != ERROR_SUCCESS as i32
    {
        let mut message: [u16; 256] = [0; 256];
        LoadStringW(hInstance, IDS_STARTUP_ERROR, message.as_mut_ptr(), 256);
        MessageBoxW(
            core::ptr::null_mut(),
            message.as_ptr(),
            core::ptr::null(),
            MB_ICONERROR,
        );
        return 1;
    }

    hExplorerInstance = hInstance;
    hProcessHeap = GetProcessHeap();

    // g_TaskbarSettings.Load() — the ::Load method is settings.rs' free fn.
    TaskbarSettings_Load(core::ptr::addr_of_mut!(g_TaskbarSettings));

    InitCommonControls();
    OleInitialize(core::ptr::null_mut());

    /* Initialize shell dde support */
    _ShellDDEInit(1);

    /* Initialize shell icons */
    FileIconInit(1);

    /* Initialize CLSID_ShellWindows class */
    _WinList_Init();

    InitializeServerAdminUI();

    /* Boot/test marker — taskbar (tray window) creation milestone. */
    OutputDebugStringA(b"[EXPLORER] taskbar: creating tray window\n\0".as_ptr() as *const i8);
    // CComPtr<ITrayWindow> Tray; — raw pointer + explicit Release at scope end.
    let mut Tray: *mut ITrayWindow = core::ptr::null_mut();
    CreateTrayWindow(&mut Tray);

    /* Hide minimized-window captions AND enable HSHELL_* notifications. */
    HideMinimizedWindows(1);

    ProcessRunOnceItems(); // Must run before the desktop is created

    let mut hShellDesktop: HANDLE = core::ptr::null_mut();
    if !Tray.is_null() {
        hShellDesktop = DesktopCreateWindow(Tray);
    }

    /* WinXP: notify msgina to hide the welcome screen */
    if SetShellReadyEvent(w!("msgina: ShellReadyEvent").as_ptr()) == 0 {
        SetShellReadyEvent(w!("Global\\msgina: ShellReadyEvent").as_ptr());
    }

    if DoStartStartupItems(Tray as *mut c_void) != 0 {
        ProcessStartupItems(0); // no-arg overload == ProcessStartupItems(FALSE)
        ReleaseStartupMutex(); // DoFinishStartupItems() is a C++ inline over this
    }
    ReleaseStartupMutex(); // For ProcessRunOnceItems

    if !Tray.is_null() {
        /* Boot/test marker — taskbar up, entering its message loop. */
        OutputDebugStringA(b"[EXPLORER] taskbar: entering message loop\n\0".as_ptr() as *const i8);
        TrayMessageLoop(Tray);
        HideMinimizedWindows(0);
    }

    if !hShellDesktop.is_null() {
        DesktopDestroyShellWindow(hShellDesktop);
    }

    OleUninitialize();

    RegCloseKey(hkExplorer);
    hkExplorer = core::ptr::null_mut();

    InitializeAtlModule(hInstance, 0);

    // CComPtr<ITrayWindow> Tray destructor — release the CreateTrayWindow ref.
    if !Tray.is_null() {
        (*Tray).Release();
    }

    0
}

/// `_tWinMain` (== `wWinMain` in a UNICODE build) — the process entry point the
/// CRT startup calls. Named `wWinMain` so mingw's `crt0_w.c` binds it.
#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    _hPrevInstance: HINSTANCE,
    _lpCmdLine: LPWSTR,
    _nCmdShow: i32,
) -> i32 {
    /* Boot marker — proves EXPLORER user-mode code is running. The smoke
     * harness (tests/qemu/vm.py) keys on this exact string. DO NOT change it. */
    OutputDebugStringA(b"[EXPLORER] _tWinMain entered\n\0".as_ptr() as *const i8);

    /* Set our shutdown parameters: shut down very last, but before any TaskMgr
     * instance (which has a shutdown level of 1). */
    SetProcessShutdownParameters(2, 0);

    InitRSHELL();

    // TRACE("Explorer starting... Command line: %S\n", lpCmdLine); — elided.

    bExplorerIsShell =
        (GetShellWindow().is_null() && IsExplorerSystemShell() != 0) as BOOL;

    if bExplorerIsShell == 0 {
        return StartWithCommandLine(hInstance);
    }

    StartWithDesktop(hInstance)
}
