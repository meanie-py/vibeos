//! trayprop.cpp — the "Taskbar and Start Menu Properties" dialog. Three WTL
//! `CPropertyPageImpl<>` pages (CTaskBarSettingsPage / CStartMenuSettingsPage /
//! CNotifySettingsPage) hosted in a property sheet, plus the entry point
//! `DisplayTrayProperties()` (called from traywnd.cpp). DIALOG code — runs only
//! when the user opens taskbar properties, so it is NOT on the boot path
//! (boot-safe: must link + not break the build).
//!
//! WTL modeling: a `CPropertyPageImpl<T>` is NOT a COM object — it is a dialog
//! (a `PROPSHEETPAGEW` whose `pfnDlgProc` is a per-page CALLBACK). Each page's
//! BEGIN_MSG_MAP / MESSAGE_HANDLER / COMMAND_*_HANDLER / CHAIN_MSG_MAP collapses
//! into one flat `match uMsg { … }` inside an `extern "system"` DLGPROC. The
//! page object (stack-local in DisplayTrayProperties, alive for the modal
//! PropertySheet) rides in PROPSHEETPAGE.lParam; on WM_INITDIALOG we stash it in
//! the dialog's DWLP_USER (the ATL thunk equivalent) and set `m_hWnd`. The
//! CHAIN_MSG_MAP(CPropertyPageImpl) tail only needs PSN_APPLY → OnApply here
//! (the sole overridden property-page virtual).
//!
//! ABI: dialog/callback procs are CALLBACK = stdcall on i386, cdecl on LLP64,
//! i.e. `extern "system"` (same spelling notifyiconscust.rs uses). The `this`
//! stash uses the arch-split SetWindowLong{,Ptr}W exactly like
//! explorer_sys::server::set_window_this, but at DWLP_USER (dialogs) not
//! GWLP_USERDATA. See the SUBSTRATE MANIFEST accompanying this file.

use core::ffi::{c_int, c_void};
use core::mem::size_of;
use core::ptr::null_mut;

use crate::explorer::settings::{g_TaskbarSettings, GetHideClock};
use explorer_sys::externs::*; // SHRestricted, DLGPROC, REST_HIDECLOCK
use explorer_sys::globals::*; // hExplorerInstance
use explorer_sys::types::*; // TaskbarSettings, TW_STRUCKRECTS2
use explorer_sys::MAKEINTRESOURCEW;
use winsrv_sys::consts::*; // DWLP_USER + hand-written consts
use winsrv_sys::winsrv::*; // bindgen fns / types / consts (BST_*, WM_*, …)
// DLGPROC is in BOTH explorer_sys::externs and winsrv bindgen; the page procs
// are `extern "system"`, so use externs::DLGPROC (also extern "system", ABI-
// identical to the stdcall comctl32 expects). The PROPSHEETPAGEW.pfnDlgProc
// field is our own file-local struct, so any consistent type works.
use explorer_sys::externs::DLGPROC;

// ===========================================================================
// SUBSTRATE not (yet) in winsrv-sys / explorer-sys. Declared file-local so this
// staging module is self-contained; the MANIFEST recommends which parts to
// promote. explorer.exe already links comctl32/user32/gdi32/shell32, so these
// resolve at link time.
// ===========================================================================

// ---- prsht.h (comctl32) property-sheet substrate --------------------------
pub type HPROPSHEETPAGE = *mut c_void;

/// `LPFNPSPCALLBACKW` — prsht.h `UINT CALLBACK (*)(HWND,UINT,LPPROPSHEETPAGEW)`.
type LPFNPSPCALLBACK =
    Option<unsafe extern "system" fn(HWND, UINT, *mut PROPSHEETPAGEW) -> UINT>;
/// `PFNPROPSHEETCALLBACK` — prsht.h `int CALLBACK (*)(HWND,UINT,LPARAM)`.
type PFNPROPSHEETCALLBACK = Option<unsafe extern "system" fn(HWND, UINT, LPARAM) -> c_int>;

/// `PROPSHEETPAGEW` (prsht.h, `_WIN32_IE >= 0x0600` full layout). Every
/// `_ANONYMOUS_UNION` is a single pointer-sized field (all arms are
/// pointer-or-smaller, so the union takes pointer size+align — layout-exact).
#[repr(C)]
struct PROPSHEETPAGEW {
    dwSize: DWORD,
    dwFlags: DWORD,
    hInstance: HINSTANCE,
    u1_pszTemplate: LPCWSTR,       // union { pszTemplate; pResource }
    u2_pszIcon: LPCWSTR,           // union { hIcon; pszIcon }
    pszTitle: LPCWSTR,
    pfnDlgProc: DLGPROC,
    lParam: LPARAM,
    pfnCallback: LPFNPSPCALLBACK,
    pcRefParent: *mut UINT,
    pszHeaderTitle: LPCWSTR,       // _WIN32_IE >= 0x0400
    pszHeaderSubTitle: LPCWSTR,    // _WIN32_IE >= 0x0400
    hActCtx: HANDLE,               // _WIN32_IE >= 0x0501
    u3_hbmHeader: LPCWSTR,         // _WIN32_IE >= 0x0600 union { hbmHeader; pszbmHeader }
}

/// `PROPSHEETHEADERW` (prsht.h, `_WIN32_IE >= 0x0400` full layout).
#[repr(C)]
struct PROPSHEETHEADERW {
    dwSize: DWORD,
    dwFlags: DWORD,
    hwndParent: HWND,
    hInstance: HINSTANCE,
    u1_pszIcon: LPCWSTR,               // union { hIcon; pszIcon }
    pszCaption: LPCWSTR,
    nPages: UINT,
    u2_nStartPage: usize,              // union { nStartPage(UINT); pStartPage } — ptr-sized
    u3_phpage: *mut HPROPSHEETPAGE,    // union { ppsp; phpage }
    pfnCallback: PFNPROPSHEETCALLBACK,
    u4_hbmWatermark: LPCWSTR,          // union { hbmWatermark; pszbmWatermark }
    hplWatermark: HANDLE,              // HPALETTE
    u5_hbmHeader: LPCWSTR,             // union { hbmHeader; pszbmHeader }
}

extern "system" {
    fn CreatePropertySheetPageW(lppsp: *const PROPSHEETPAGEW) -> HPROPSHEETPAGE;
    fn PropertySheetW(lppsph: *const PROPSHEETHEADERW) -> INT_PTR;
    // user32 CheckDlgButton is NOT in the winsrv bindgen surface (IsDlgButtonChecked is).
    fn CheckDlgButton(hDlg: HWND, nIDButton: c_int, uCheck: UINT) -> BOOL;
    // shell32 (undocshell) — get/set the global shell-state bitfields.
    fn SHGetSetSettings(lpss: *mut SHELLSTATE, dwMask: DWORD, bSet: BOOL);
}

// ShowCustomizeClassic is still C++ (startmnucust.cpp; precomp.h already wraps
// it in `extern "C"`), so it is a plain-C import here.
extern "C" {
    fn ShowCustomizeClassic(hInst: HINSTANCE, hExplorer: HWND);
}
// ShowCustomizeNotifyIcons is already Rust (notifyiconscust.rs) — call it
// directly (no FFI ABI risk).
use crate::explorer::notifyiconscust::ShowCustomizeNotifyIcons;

// ---- prsht.h constants ----
const PSP_DEFAULT: DWORD = 0;
const PSH_PROPTITLE: DWORD = 0x0000_0001;
const PSH_USEICONID: DWORD = 0x0000_0004;
const PSH_USECALLBACK: DWORD = 0x0000_0100;
const PSCB_INITIALIZED: UINT = 1;
const PSNRET_NOERROR: c_int = 0;
/// `PSN_APPLY` — prsht.h `PSN_FIRST(0u-200) - 2` = 0xFFFFFF36. NMHDR.code is UINT.
const PSN_APPLY: UINT = (-202i32) as UINT;
/// `PSM_CHANGED` — prsht.h `WM_USER + 104` (SetModified → PropSheet_Changed).
const PSM_CHANGED: UINT = WM_USER + 104;

/// `DWLP_MSGRESULT` — winuser.h = 0 (DWLP_USER comes from winsrv consts).
const DWLP_MSGRESULT: c_int = 0;

// ---- precomp.h custom message ----
/// `TWM_SETTINGSCHANGED` — precomp.h `WM_USER + 300`.
const TWM_SETTINGSCHANGED: UINT = WM_USER + 300;

// ---- shlobj.h RESTRICTIONS / SSF_* ----
/// `REST_NOSTARTPANEL` — shlobj.h (0x4000005A): disallow the modern start menu.
const REST_NOSTARTPANEL: u32 = 0x4000_005A;
/// `SSF_STARTPANELON` — shlobj.h shell-state mask bit.
const SSF_STARTPANELON: DWORD = 0x0020_0000;

// ---- base/shell/explorer/resource.h dialog / control / bitmap / string IDs --
const IDI_STARTMENU: c_int = 107;

const IDD_TASKBARPROP_TASKBAR: c_int = 6;
const IDD_TASKBARPROP_STARTMENU: c_int = 205;
const IDD_TASKBARPROP_NOTIFY: c_int = 206;

const IDS_TASKBAR_STARTMENU_PROP_CAPTION: UINT = 810;

// Taskbar page controls (COMMAND range 1114..=1119).
const IDC_TASKBARPROP_FIRST_CMD: c_int = 1114;
const IDC_TASKBARPROP_ONTOP: c_int = 1114;
const IDC_TASKBARPROP_HIDE: c_int = 1115;
const IDC_TASKBARPROP_GROUP: c_int = 1116;
const IDC_TASKBARPROP_LOCK: c_int = 1117;
const IDC_TASKBARPROP_SHOWQL: c_int = 1118;
const IDC_TASKBARPROP_SMALLICONS: c_int = 1119;
const IDC_TASKBARPROP_LAST_CMD: c_int = 1119;
const IDC_TASKBARPROP_TASKBARBITMAP: c_int = 1120;

// Start-menu page controls.
const IDC_TASKBARPROP_STARTMENUCLASSICCUST: c_int = 1130;
const IDC_TASKBARPROP_STARTMENUCUST: c_int = 1131;
const IDC_TASKBARPROP_STARTMENU: c_int = 1132;
const IDC_TASKBARPROP_STARTMENUCLASSIC: c_int = 1133;
const IDC_TASKBARPROP_STARTMENU_BITMAP: c_int = 1134;
const IDC_TASKBARPROP_STARTMENUMODERNTEXT: c_int = 1141;

// Notification page controls (COMMAND range 1000..=1113).
const IDC_TASKBARPROP_NOTIFY_FIRST_CMD: c_int = 1000;
const IDC_TASKBARPROP_HIDEICONS: c_int = 1000;
const IDC_TASKBARPROP_ICONCUST: c_int = 1007;
const IDC_TASKBARPROP_CLOCK: c_int = 1103;
const IDC_TASKBARPROP_SECONDS: c_int = 1106;
const IDC_TASKBARPROP_NOTIFICATIONBITMAP: c_int = 1112;
const IDC_TASKBARPROP_DESKTOP: c_int = 1113;
const IDC_TASKBARPROP_NOTIFY_LAST_CMD: c_int = 1113;

// Bitmaps.
const IDB_TASKBARPROP_AUTOHIDE: UINT = 145;
const IDB_TASKBARPROP_LOCK_GROUP_QL: UINT = 146;
const IDB_TASKBARPROP_NOLOCK_GROUP_QL: UINT = 147;
const IDB_TASKBARPROP_LOCK_NOGROUP_QL: UINT = 148;
const IDB_TASKBARPROP_NOLOCK_NOGROUP_QL: UINT = 149;
const IDB_TASKBARPROP_LOCK_GROUP_NOQL: UINT = 150;
const IDB_TASKBARPROP_NOLOCK_GROUP_NOQL: UINT = 151;
const IDB_TASKBARPROP_LOCK_NOGROUP_NOQL: UINT = 152;
const IDB_TASKBARPROP_NOLOCK_NOGROUP_NOQL: UINT = 153;
const IDB_SYSTRAYPROP_SHOW_SECONDS_NODESK: UINT = 154;
const IDB_SYSTRAYPROP_HIDE_SECONDS_NODESK: UINT = 155;
const IDB_STARTPREVIEW: UINT = 170;
const IDB_STARTPREVIEW_CLASSIC: UINT = 171;
const IDB_SYSTRAYPROP_HIDE_CLOCK_NODESK: UINT = 180;
const IDB_SYSTRAYPROP_HIDE_NOCLOCK_NODESK: UINT = 181;
const IDB_SYSTRAYPROP_SHOW_CLOCK_NODESK: UINT = 182;
const IDB_SYSTRAYPROP_SHOW_NOCLOCK_NODESK: UINT = 183;
const IDB_SYSTRAYPROP_SHOW_SECONDS_DESK: UINT = 184;
const IDB_SYSTRAYPROP_HIDE_SECONDS_DESK: UINT = 185;
const IDB_SYSTRAYPROP_HIDE_CLOCK_DESK: UINT = 186;
const IDB_SYSTRAYPROP_HIDE_NOCLOCK_DESK: UINT = 187;
const IDB_SYSTRAYPROP_SHOW_CLOCK_DESK: UINT = 188;
const IDB_SYSTRAYPROP_SHOW_NOCLOCK_DESK: UINT = 189;

/// `SHELLSTATE` (shlobj.h) — the exact 32-byte gcc-bitfield layout (two 32-bit
/// flag words at offset 0 and 28 bracketing the scalar members). Mirrors
/// shell32-rust's model; only `fStartPanelOn` (w1 bit 1) is used here.
#[repr(C)]
#[derive(Copy, Clone)]
struct SHELLSTATE {
    w0: u32,
    dwWin95Unused: DWORD,
    uWin95Unused: UINT,
    lParamSort: LONG,
    iSortDirection: c_int,
    version: UINT,
    uNotUsed: UINT,
    w1: u32,
}
impl SHELLSTATE {
    const SS_STARTPANELON: u32 = 1; // w1 bit index
    #[inline]
    fn fStartPanelOn(&self) -> BOOL {
        ((self.w1 >> Self::SS_STARTPANELON) & 1) as BOOL
    }
    #[inline]
    fn set_fStartPanelOn(&mut self, v: BOOL) {
        let m = 1u32 << Self::SS_STARTPANELON;
        if v != 0 {
            self.w1 |= m;
        } else {
            self.w1 &= !m;
        }
    }
}

// ---- file-local page objects (stack-locals in DisplayTrayProperties; layout
//      is private, never shared) --------------------------------------------
struct CTaskBarSettingsPage {
    m_hWnd: HWND,
    m_hbmpTaskbar: HBITMAP,
    m_hwndTaskbar: HWND,
}
struct CStartMenuSettingsPage {
    m_hWnd: HWND,
    m_hbmpStartBitmap: HBITMAP,
}
struct CNotifySettingsPage {
    m_hWnd: HWND,
    m_hbmpTray: HBITMAP,
    m_hwndTaskbar: HWND,
}

/// `CNotifySettingsPage::wImageIdLookupTable[2][2][2][2]`
/// ([bShowClock][bShowSeconds][bHideInactive][bShowDesktopButton]).
static wImageIdLookupTable: [[[[UINT; 2]; 2]; 2]; 2] = [
    [
        [
            [IDB_SYSTRAYPROP_SHOW_NOCLOCK_NODESK, IDB_SYSTRAYPROP_SHOW_NOCLOCK_DESK],
            [IDB_SYSTRAYPROP_HIDE_NOCLOCK_NODESK, IDB_SYSTRAYPROP_HIDE_NOCLOCK_DESK],
        ],
        [
            [IDB_SYSTRAYPROP_SHOW_NOCLOCK_NODESK, IDB_SYSTRAYPROP_SHOW_NOCLOCK_DESK],
            [IDB_SYSTRAYPROP_HIDE_NOCLOCK_NODESK, IDB_SYSTRAYPROP_HIDE_NOCLOCK_DESK],
        ],
    ],
    [
        [
            [IDB_SYSTRAYPROP_SHOW_CLOCK_NODESK, IDB_SYSTRAYPROP_SHOW_CLOCK_DESK],
            [IDB_SYSTRAYPROP_HIDE_CLOCK_NODESK, IDB_SYSTRAYPROP_HIDE_CLOCK_DESK],
        ],
        [
            [IDB_SYSTRAYPROP_SHOW_SECONDS_NODESK, IDB_SYSTRAYPROP_SHOW_SECONDS_DESK],
            [IDB_SYSTRAYPROP_HIDE_SECONDS_NODESK, IDB_SYSTRAYPROP_HIDE_SECONDS_DESK],
        ],
    ],
];

// ---- helpers --------------------------------------------------------------

/// `SetBitmap(hwnd, &hbmp, uImageId)` (file-static in trayprop.cpp): load the
/// preview bitmap, resize the static control to fit, install it.
unsafe fn SetBitmap(hwnd: HWND, hbmp: *mut HBITMAP, uImageId: UINT) {
    if !(*hbmp).is_null() {
        DeleteObject(*hbmp as HGDIOBJ);
    }

    *hbmp = LoadImageW(
        hExplorerInstance,
        MAKEINTRESOURCEW(uImageId as u16),
        IMAGE_BITMAP,
        0,
        0,
        LR_DEFAULTCOLOR,
    ) as HBITMAP;

    if !(*hbmp).is_null() && !hwnd.is_null() {
        let mut bm: BITMAP = core::mem::zeroed();
        GetObjectW(
            *hbmp as HANDLE,
            size_of::<BITMAP>() as c_int,
            &mut bm as *mut BITMAP as LPVOID,
        );
        SetWindowPos(
            hwnd,
            null_mut(),
            0,
            0,
            bm.bmWidth + 2,
            bm.bmHeight + 2,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER,
        );
        SendMessageW(hwnd, STM_SETIMAGE, IMAGE_BITMAP as WPARAM, *hbmp as LPARAM);
    }
}

/// `CPropertyPageWindow::SetModified(TRUE)` → PropSheet_Changed(GetParent, page).
#[inline]
unsafe fn set_modified(hwnd_page: HWND) {
    SendMessageW(GetParent(hwnd_page), PSM_CHANGED, hwnd_page as WPARAM, 0);
}

/// TW_STUCKRECTS2 bitfield write (types.rs ships getters + set_HideClock/
/// set_SmSmallIcons but not set_AutoHide/set_AlwaysOnTop — set inline via F_*).
#[inline]
fn set_sr_flag(flags: &mut DWORD, mask: DWORD, on: UINT) {
    if on != 0 {
        *flags |= mask;
    } else {
        *flags &= !mask;
    }
}

// `this`-recovery over DWLP_USER (dialog user slot), ARCH-SPLIT exactly like
// explorer_sys::server::{set,get}_window_this: on i386 the *Ptr* names are C
// macros over SetWindowLongW; on LLP64 the real *Ptr* fns take LONG_PTR.
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn dlg_set_this(hwnd: HWND, this: *mut c_void) {
    SetWindowLongW(hwnd, DWLP_USER, this as LONG);
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn dlg_get_this(hwnd: HWND) -> *mut c_void {
    GetWindowLongW(hwnd, DWLP_USER) as u32 as usize as *mut c_void
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn dlg_set_msgresult(hwnd: HWND, v: LRESULT) {
    SetWindowLongW(hwnd, DWLP_MSGRESULT, v as LONG);
}

#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn dlg_set_this(hwnd: HWND, this: *mut c_void) {
    SetWindowLongPtrW(hwnd, DWLP_USER, this as LONG_PTR);
}
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn dlg_get_this(hwnd: HWND) -> *mut c_void {
    GetWindowLongPtrW(hwnd, DWLP_USER) as *mut c_void
}
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn dlg_set_msgresult(hwnd: HWND, v: LRESULT) {
    SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, v as LONG_PTR);
}

/// Recover the page `this` from PROPSHEETPAGE.lParam on WM_INITDIALOG and stash
/// it in DWLP_USER — the ATL StartDialogProc thunk equivalent. The caller sets
/// its own typed `m_hWnd` (structs aren't repr(C), so no offset-0 assumption).
#[inline]
unsafe fn init_page_this(hwnd: HWND, lParam: LPARAM) -> *mut c_void {
    let psp = lParam as *const PROPSHEETPAGEW;
    let this = (*psp).lParam as *mut c_void;
    if !this.is_null() {
        dlg_set_this(hwnd, this);
    }
    this
}

// ===========================================================================
// CTaskBarSettingsPage  (IDD = IDD_TASKBARPROP_TASKBAR)
// ===========================================================================

unsafe fn taskbar_update_dialog(this: *mut CTaskBarSettingsPage) {
    let hwnd = (*this).m_hWnd;
    let bLock = IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_LOCK);
    let bHide = IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_HIDE);
    let bGroup = IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_GROUP);
    let bShowQL = IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_SHOWQL);

    let hwndTaskbarBitmap = GetDlgItem(hwnd, IDC_TASKBARPROP_TASKBARBITMAP);

    let uImageId: UINT = if bHide != 0 {
        IDB_TASKBARPROP_AUTOHIDE
    } else if bLock != 0 && bGroup != 0 && bShowQL != 0 {
        IDB_TASKBARPROP_LOCK_GROUP_QL
    } else if bLock != 0 && bGroup == 0 && bShowQL == 0 {
        IDB_TASKBARPROP_LOCK_NOGROUP_NOQL
    } else if bLock != 0 && bGroup != 0 && bShowQL == 0 {
        IDB_TASKBARPROP_LOCK_GROUP_NOQL
    } else if bLock != 0 && bGroup == 0 && bShowQL != 0 {
        IDB_TASKBARPROP_LOCK_NOGROUP_QL
    } else if bLock == 0 && bGroup == 0 && bShowQL == 0 {
        IDB_TASKBARPROP_NOLOCK_NOGROUP_NOQL
    } else if bLock == 0 && bGroup != 0 && bShowQL == 0 {
        IDB_TASKBARPROP_NOLOCK_GROUP_NOQL
    } else if bLock == 0 && bGroup == 0 && bShowQL != 0 {
        IDB_TASKBARPROP_NOLOCK_NOGROUP_QL
    } else {
        // (!bLock && bGroup && bShowQL)
        IDB_TASKBARPROP_NOLOCK_GROUP_QL
    };

    SetBitmap(
        hwndTaskbarBitmap,
        core::ptr::addr_of_mut!((*this).m_hbmpTaskbar),
        uImageId,
    );
}

unsafe fn taskbar_on_init_dialog(this: *mut CTaskBarSettingsPage) {
    let hwnd = (*this).m_hWnd;
    CheckDlgButton(
        hwnd,
        IDC_TASKBARPROP_LOCK,
        if g_TaskbarSettings.bLock != 0 { BST_CHECKED } else { BST_UNCHECKED },
    );
    CheckDlgButton(
        hwnd,
        IDC_TASKBARPROP_HIDE,
        if (*&raw const g_TaskbarSettings).sr.AutoHide() != 0 { BST_CHECKED } else { BST_UNCHECKED },
    );
    CheckDlgButton(
        hwnd,
        IDC_TASKBARPROP_ONTOP,
        if (*&raw const g_TaskbarSettings).sr.AlwaysOnTop() != 0 { BST_CHECKED } else { BST_UNCHECKED },
    );
    CheckDlgButton(
        hwnd,
        IDC_TASKBARPROP_GROUP,
        if g_TaskbarSettings.bGroupButtons != 0 { BST_CHECKED } else { BST_UNCHECKED },
    );
    // (IDC_TASKBARPROP_SHOWQL / bShowQuickLaunch left commented out in the C++)
    CheckDlgButton(
        hwnd,
        IDC_TASKBARPROP_SMALLICONS,
        if g_TaskbarSettings.bSmallIcons != 0 { BST_CHECKED } else { BST_UNCHECKED },
    );

    taskbar_update_dialog(this);
}

unsafe fn taskbar_on_apply(this: *mut CTaskBarSettingsPage) -> c_int {
    let hwnd = (*this).m_hWnd;
    let mut newSettings: TaskbarSettings = g_TaskbarSettings;

    newSettings.bLock = IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_LOCK) as BOOL;
    set_sr_flag(
        &mut newSettings.sr.dwFlags,
        TW_STRUCKRECTS2::F_AUTOHIDE,
        IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_HIDE),
    );
    set_sr_flag(
        &mut newSettings.sr.dwFlags,
        TW_STRUCKRECTS2::F_ALWAYSONTOP,
        IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_ONTOP),
    );
    newSettings.bGroupButtons = IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_GROUP) as BOOL;
    newSettings.bSmallIcons = IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_SMALLICONS) as BOOL;

    SendMessageW(
        (*this).m_hwndTaskbar,
        TWM_SETTINGSCHANGED,
        0,
        core::ptr::addr_of_mut!(newSettings) as LPARAM,
    );

    PSNRET_NOERROR
}

#[no_mangle]
pub unsafe extern "system" fn TaskBarSettingsPageProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            let this = init_page_this(hwnd, lParam) as *mut CTaskBarSettingsPage;
            if this.is_null() {
                return 0;
            }
            (*this).m_hWnd = hwnd;
            taskbar_on_init_dialog(this);
            1
        }
        WM_COMMAND => {
            let this = dlg_get_this(hwnd) as *mut CTaskBarSettingsPage;
            if this.is_null() {
                return 0;
            }
            let id = (wParam & 0xFFFF) as c_int;
            if id >= IDC_TASKBARPROP_FIRST_CMD && id <= IDC_TASKBARPROP_LAST_CMD {
                taskbar_update_dialog(this);
                set_modified(hwnd);
                return 1;
            }
            0
        }
        WM_NOTIFY => {
            let this = dlg_get_this(hwnd) as *mut CTaskBarSettingsPage;
            if this.is_null() {
                return 0;
            }
            let pnmh = lParam as *const NMHDR;
            if (*pnmh).code == PSN_APPLY {
                dlg_set_msgresult(hwnd, taskbar_on_apply(this) as LRESULT);
                return 1;
            }
            0
        }
        _ => 0,
    }
}

// ===========================================================================
// CStartMenuSettingsPage  (IDD = IDD_TASKBARPROP_STARTMENU)
// ===========================================================================

unsafe fn startmenu_update_dialog(this: *mut CStartMenuSettingsPage) {
    let hwnd = (*this).m_hWnd;
    let hwndCustomizeClassic = GetDlgItem(hwnd, IDC_TASKBARPROP_STARTMENUCLASSICCUST);
    let hwndCustomizeModern = GetDlgItem(hwnd, IDC_TASKBARPROP_STARTMENUCUST);
    let hwndStartBitmap = GetDlgItem(hwnd, IDC_TASKBARPROP_STARTMENU_BITMAP);
    let hwndModernRadioBtn = GetDlgItem(hwnd, IDC_TASKBARPROP_STARTMENU);
    let hwndModernText = GetDlgItem(hwnd, IDC_TASKBARPROP_STARTMENUMODERNTEXT);
    let policyNoSimpleStartMenu = SHRestricted(REST_NOSTARTPANEL) != 0;
    let mut bModern: BOOL = 0;

    if policyNoSimpleStartMenu {
        // Switch to classic + disable/hide the modern controls.
        CheckDlgButton(hwnd, IDC_TASKBARPROP_STARTMENUCLASSIC, BST_CHECKED);
        EnableWindow(hwndModernRadioBtn, 0);
        ShowWindow(hwndModernRadioBtn, SW_HIDE as c_int);
        ShowWindow(hwndModernText, SW_HIDE as c_int);
        ShowWindow(hwndCustomizeModern, SW_HIDE as c_int);
    } else {
        bModern = IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_STARTMENU) as BOOL;
    }

    EnableWindow(hwndCustomizeModern, bModern);
    EnableWindow(hwndCustomizeClassic, if bModern != 0 { 0 } else { 1 });

    let uImageId = if bModern != 0 { IDB_STARTPREVIEW } else { IDB_STARTPREVIEW_CLASSIC };
    SetBitmap(
        hwndStartBitmap,
        core::ptr::addr_of_mut!((*this).m_hbmpStartBitmap),
        uImageId,
    );
}

unsafe fn startmenu_on_init_dialog(this: *mut CStartMenuSettingsPage) {
    // SHELL_GetSetting(SSF_STARTPANELON, fStartPanelOn) (shellutils.h).
    let mut ss: SHELLSTATE = core::mem::zeroed();
    SHGetSetSettings(&mut ss, SSF_STARTPANELON, 0);
    let modern = ss.fStartPanelOn();

    CheckDlgButton(
        (*this).m_hWnd,
        if modern != 0 { IDC_TASKBARPROP_STARTMENU } else { IDC_TASKBARPROP_STARTMENUCLASSIC },
        BST_CHECKED,
    );
    startmenu_update_dialog(this);
}

unsafe fn startmenu_on_apply(this: *mut CStartMenuSettingsPage) -> c_int {
    let mut ss: SHELLSTATE = core::mem::zeroed();
    ss.set_fStartPanelOn(
        (IsDlgButtonChecked((*this).m_hWnd, IDC_TASKBARPROP_STARTMENUCLASSIC) == 0) as BOOL,
    );
    SHGetSetSettings(&mut ss, SSF_STARTPANELON, 1);
    PSNRET_NOERROR
}

#[no_mangle]
pub unsafe extern "system" fn StartMenuSettingsPageProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            let this = init_page_this(hwnd, lParam) as *mut CStartMenuSettingsPage;
            if this.is_null() {
                return 0;
            }
            (*this).m_hWnd = hwnd;
            startmenu_on_init_dialog(this);
            1
        }
        WM_COMMAND => {
            let this = dlg_get_this(hwnd) as *mut CStartMenuSettingsPage;
            if this.is_null() {
                return 0;
            }
            let id = (wParam & 0xFFFF) as c_int;
            if id == IDC_TASKBARPROP_STARTMENUCLASSICCUST {
                ShowCustomizeClassic(hExplorerInstance, (*this).m_hWnd);
                return 1;
            }
            0
        }
        WM_NOTIFY => {
            let this = dlg_get_this(hwnd) as *mut CStartMenuSettingsPage;
            if this.is_null() {
                return 0;
            }
            let pnmh = lParam as *const NMHDR;
            if (*pnmh).code == PSN_APPLY {
                dlg_set_msgresult(hwnd, startmenu_on_apply(this) as LRESULT);
                return 1;
            }
            0
        }
        _ => 0,
    }
}

// ===========================================================================
// CNotifySettingsPage  (IDD = IDD_TASKBARPROP_NOTIFY)
// ===========================================================================

unsafe fn notify_update_dialog(this: *mut CNotifySettingsPage) {
    let hwnd = (*this).m_hWnd;
    let bShowClock = IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_CLOCK);
    let bShowSeconds = IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_SECONDS);
    let bHideInactive = IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_HIDEICONS);
    let bShowDesktopButton = IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_DESKTOP);

    let hwndCustomizeNotifyButton = GetDlgItem(hwnd, IDC_TASKBARPROP_ICONCUST);
    let hwndSeconds = GetDlgItem(hwnd, IDC_TASKBARPROP_SECONDS);
    let hwndTrayBitmap = GetDlgItem(hwnd, IDC_TASKBARPROP_NOTIFICATIONBITMAP);

    EnableWindow(hwndCustomizeNotifyButton, bHideInactive as BOOL);
    EnableWindow(hwndSeconds, bShowClock as BOOL);
    if bShowSeconds == 0 {
        CheckDlgButton(hwnd, IDC_TASKBARPROP_SECONDS, BST_UNCHECKED);
    }
    let uImageId = wImageIdLookupTable[bShowClock as usize][bShowSeconds as usize]
        [bHideInactive as usize][bShowDesktopButton as usize];

    SetBitmap(
        hwndTrayBitmap,
        core::ptr::addr_of_mut!((*this).m_hbmpTray),
        uImageId,
    );
}

unsafe fn notify_on_init_dialog(this: *mut CNotifySettingsPage) {
    let hwnd = (*this).m_hWnd;
    EnableWindow(
        GetDlgItem(hwnd, IDC_TASKBARPROP_CLOCK),
        if SHRestricted(REST_HIDECLOCK) != 0 { 0 } else { 1 },
    );
    CheckDlgButton(
        hwnd,
        IDC_TASKBARPROP_CLOCK,
        if GetHideClock() == 0 { BST_CHECKED } else { BST_UNCHECKED },
    );
    CheckDlgButton(
        hwnd,
        IDC_TASKBARPROP_SECONDS,
        if g_TaskbarSettings.bShowSeconds != 0 { BST_CHECKED } else { BST_UNCHECKED },
    );
    CheckDlgButton(
        hwnd,
        IDC_TASKBARPROP_HIDEICONS,
        if g_TaskbarSettings.bHideInactiveIcons != 0 { BST_CHECKED } else { BST_UNCHECKED },
    );
    CheckDlgButton(
        hwnd,
        IDC_TASKBARPROP_DESKTOP,
        if g_TaskbarSettings.bShowDesktopButton != 0 { BST_CHECKED } else { BST_UNCHECKED },
    );

    // TODO(upstream): bHideInactiveIcons not implemented yet — disable for now.
    CheckDlgButton(hwnd, IDC_TASKBARPROP_HIDEICONS, BST_UNCHECKED);
    EnableWindow(GetDlgItem(hwnd, IDC_TASKBARPROP_HIDEICONS), 0);

    notify_update_dialog(this);
}

unsafe fn notify_on_apply(this: *mut CNotifySettingsPage) -> c_int {
    let hwnd = (*this).m_hWnd;
    let mut newSettings: TaskbarSettings = g_TaskbarSettings;

    set_sr_flag(
        &mut newSettings.sr.dwFlags,
        TW_STRUCKRECTS2::F_HIDECLOCK,
        (IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_CLOCK) == 0) as UINT,
    );
    newSettings.bShowSeconds = (IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_SECONDS) != 0) as BOOL;
    newSettings.bHideInactiveIcons =
        (IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_HIDEICONS) != 0) as BOOL;
    newSettings.bShowDesktopButton =
        (IsDlgButtonChecked(hwnd, IDC_TASKBARPROP_DESKTOP) != 0) as BOOL;

    SendMessageW(
        (*this).m_hwndTaskbar,
        TWM_SETTINGSCHANGED,
        0,
        core::ptr::addr_of_mut!(newSettings) as LPARAM,
    );

    PSNRET_NOERROR
}

#[no_mangle]
pub unsafe extern "system" fn NotifySettingsPageProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            let this = init_page_this(hwnd, lParam) as *mut CNotifySettingsPage;
            if this.is_null() {
                return 0;
            }
            (*this).m_hWnd = hwnd;
            notify_on_init_dialog(this);
            1
        }
        WM_COMMAND => {
            let this = dlg_get_this(hwnd) as *mut CNotifySettingsPage;
            if this.is_null() {
                return 0;
            }
            let id = (wParam & 0xFFFF) as c_int;
            // COMMAND_ID_HANDLER(ICONCUST) is listed before the range handler,
            // so it wins for that specific id.
            if id == IDC_TASKBARPROP_ICONCUST {
                ShowCustomizeNotifyIcons(hExplorerInstance, (*this).m_hWnd);
                return 1;
            }
            if id >= IDC_TASKBARPROP_NOTIFY_FIRST_CMD && id <= IDC_TASKBARPROP_NOTIFY_LAST_CMD {
                notify_update_dialog(this);
                set_modified(hwnd);
                return 1;
            }
            0
        }
        WM_NOTIFY => {
            let this = dlg_get_this(hwnd) as *mut CNotifySettingsPage;
            if this.is_null() {
                return 0;
            }
            let pnmh = lParam as *const NMHDR;
            if (*pnmh).code == PSN_APPLY {
                dlg_set_msgresult(hwnd, notify_on_apply(this) as LRESULT);
                return 1;
            }
            0
        }
        _ => 0,
    }
}

// ===========================================================================
// property-sheet plumbing + entry point
// ===========================================================================

/// `PropSheetProc` — sets the sheet's large icon on PSCB_INITIALIZED.
unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: UINT, _lParam: LPARAM) -> c_int {
    if uMsg == PSCB_INITIALIZED {
        let hIcon = LoadIconW(hExplorerInstance, MAKEINTRESOURCEW(IDI_STARTMENU as u16));
        SendMessageW(hwndDlg, WM_SETICON, ICON_BIG as WPARAM, hIcon as LPARAM);
    }
    0
}

/// `CPropertyPageImpl<T>::Create()` — build a PROPSHEETPAGEW and register it.
/// `this` (the page object) rides in PROPSHEETPAGE.lParam.
unsafe fn create_page(idd: c_int, dlgproc: DLGPROC, this: LPARAM) -> HPROPSHEETPAGE {
    let mut psp: PROPSHEETPAGEW = core::mem::zeroed();
    psp.dwSize = size_of::<PROPSHEETPAGEW>() as DWORD;
    psp.dwFlags = PSP_DEFAULT;
    psp.hInstance = hExplorerInstance;
    psp.u1_pszTemplate = MAKEINTRESOURCEW(idd as u16);
    psp.pfnDlgProc = dlgproc;
    psp.lParam = this;
    CreatePropertySheetPageW(&psp)
}

#[no_mangle]
pub unsafe extern "C" fn DisplayTrayProperties(hwndOwner: HWND, hwndTaskbar: HWND) {
    let mut tbSettingsPage = CTaskBarSettingsPage {
        m_hWnd: null_mut(),
        m_hbmpTaskbar: null_mut(),
        m_hwndTaskbar: hwndTaskbar,
    };
    let mut smSettingsPage = CStartMenuSettingsPage {
        m_hWnd: null_mut(),
        m_hbmpStartBitmap: null_mut(),
    };
    let mut naSettingsPage = CNotifySettingsPage {
        m_hWnd: null_mut(),
        m_hbmpTray: null_mut(),
        m_hwndTaskbar: hwndTaskbar,
    };

    let mut caption: [u16; 256] = [0; 256];
    LoadStringW(
        hExplorerInstance,
        IDS_TASKBAR_STARTMENU_PROP_CAPTION,
        caption.as_mut_ptr(),
        caption.len() as c_int,
    );

    let mut hpsp: [HPROPSHEETPAGE; 3] = [
        create_page(
            IDD_TASKBARPROP_TASKBAR,
            Some(TaskBarSettingsPageProc),
            core::ptr::addr_of_mut!(tbSettingsPage) as LPARAM,
        ),
        create_page(
            IDD_TASKBARPROP_STARTMENU,
            Some(StartMenuSettingsPageProc),
            core::ptr::addr_of_mut!(smSettingsPage) as LPARAM,
        ),
        create_page(
            IDD_TASKBARPROP_NOTIFY,
            Some(NotifySettingsPageProc),
            core::ptr::addr_of_mut!(naSettingsPage) as LPARAM,
        ),
    ];

    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
    psh.dwSize = size_of::<PROPSHEETHEADERW>() as DWORD;
    psh.dwFlags = PSH_PROPTITLE | PSH_USEICONID | PSH_USECALLBACK;
    psh.hwndParent = hwndOwner;
    psh.hInstance = hExplorerInstance;
    psh.u1_pszIcon = MAKEINTRESOURCEW(IDI_STARTMENU as u16);
    psh.pszCaption = caption.as_ptr();
    psh.nPages = hpsp.len() as UINT;
    psh.u2_nStartPage = 0;
    psh.u3_phpage = hpsp.as_mut_ptr();
    psh.pfnCallback = Some(PropSheetProc);

    PropertySheetW(&psh);

    // Page destructors (~CTaskBarSettingsPage/…): free the preview bitmaps.
    if !tbSettingsPage.m_hbmpTaskbar.is_null() {
        DeleteObject(tbSettingsPage.m_hbmpTaskbar as HGDIOBJ);
    }
    if !smSettingsPage.m_hbmpStartBitmap.is_null() {
        DeleteObject(smSettingsPage.m_hbmpStartBitmap as HGDIOBJ);
    }
    if !naSettingsPage.m_hbmpTray.is_null() {
        DeleteObject(naSettingsPage.m_hbmpTray as HGDIOBJ);
    }
}

// ============================================================================
// ORIGINAL C++ SOURCE (base/shell/explorer/trayprop.cpp)
// ============================================================================
//
// static void SetBitmap(HWND hwnd, HBITMAP* hbmp, UINT uImageId)
// {
//     if (*hbmp)
//         DeleteObject(*hbmp);
//     *hbmp = (HBITMAP)LoadImageW(hExplorerInstance, MAKEINTRESOURCEW(uImageId),
//                                 IMAGE_BITMAP, 0, 0, LR_DEFAULTCOLOR);
//     if (*hbmp && hwnd)
//     {
//         BITMAP bm;
//         GetObject(*hbmp, sizeof(bm), &bm);
//         ::SetWindowPos(hwnd, NULL, 0, 0, bm.bmWidth + 2, bm.bmHeight + 2,
//                        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER);
//         SendMessage(hwnd, STM_SETIMAGE, IMAGE_BITMAP, (LPARAM)*hbmp);
//     }
// }
//
// class CTaskBarSettingsPage : public CPropertyPageImpl<CTaskBarSettingsPage>
// {
//     HBITMAP m_hbmpTaskbar; HWND m_hwndTaskbar;
//     void _UpdateDialog() { … lock/hide/group/showql → uImageId; SetBitmap(…); }
//   public:
//     enum { IDD = IDD_TASKBARPROP_TASKBAR };
//     BEGIN_MSG_MAP(CTaskBarSettingsPage)
//         MESSAGE_HANDLER(WM_INITDIALOG, OnInitDialog)
//         COMMAND_RANGE_HANDLER(IDC_TASKBARPROP_FIRST_CMD, IDC_TASKBARPROP_LAST_CMD, OnCtrlCommand)
//         CHAIN_MSG_MAP(CPropertyPageImpl<CTaskBarSettingsPage>)
//     END_MSG_MAP()
//     CTaskBarSettingsPage(HWND hwnd): m_hbmpTaskbar(NULL), m_hwndTaskbar(hwnd) {}
//     ~CTaskBarSettingsPage() { if (m_hbmpTaskbar) DeleteObject(m_hbmpTaskbar); }
//     LRESULT OnInitDialog(…) { CheckDlgButton(LOCK/HIDE(sr.AutoHide)/ONTOP(sr.AlwaysOnTop)/
//         GROUP/SMALLICONS); _UpdateDialog(); return TRUE; }
//     LRESULT OnCtrlCommand(…) { _UpdateDialog(); SetModified(TRUE); return 0; }
//     int OnApply() { TaskbarSettings n = g_TaskbarSettings; n.bLock=…; n.sr.AutoHide=…;
//         n.sr.AlwaysOnTop=…; n.bGroupButtons=…; n.bSmallIcons=…;
//         SendMessage(m_hwndTaskbar, TWM_SETTINGSCHANGED, 0, (LPARAM)&n); return PSNRET_NOERROR; }
// };
//
// class CStartMenuSettingsPage : public CPropertyPageImpl<CStartMenuSettingsPage>
// {
//     HBITMAP m_hbmpStartBitmap;
//     void _UpdateDialog() { policyNoSimpleStartMenu = SHRestricted(REST_NOSTARTPANEL);
//         if (policy) { CheckDlgButton(CLASSIC, CHECKED); EnableWindow(modernRadio, FALSE);
//             ShowWindow(modernRadio/modernText/customizeModern, SW_HIDE); }
//         else bModern = IsDlgButtonChecked(STARTMENU);
//         EnableWindow(customizeModern, bModern); EnableWindow(customizeClassic, !bModern);
//         SetBitmap(startBitmap, &m_hbmpStartBitmap, bModern?IDB_STARTPREVIEW:IDB_STARTPREVIEW_CLASSIC); }
//   public:
//     enum { IDD = IDD_TASKBARPROP_STARTMENU };
//     BEGIN_MSG_MAP(CTaskBarSettingsPage)
//         MESSAGE_HANDLER(WM_INITDIALOG, OnInitDialog)
//         COMMAND_ID_HANDLER(IDC_TASKBARPROP_STARTMENUCLASSICCUST, OnStartMenuCustomize)
//         CHAIN_MSG_MAP(CPropertyPageImpl<CStartMenuSettingsPage>)
//     END_MSG_MAP()
//     LRESULT OnInitDialog(…) { BOOL modern = SHELL_GetSetting(SSF_STARTPANELON, fStartPanelOn);
//         CheckDlgButton(modern?STARTMENU:CLASSIC, CHECKED); _UpdateDialog(); return TRUE; }
//     LRESULT OnStartMenuCustomize(…) { ShowCustomizeClassic(hExplorerInstance, m_hWnd); return 0; }
//     int OnApply() { SHELLSTATE ss; ss.fStartPanelOn = !IsDlgButtonChecked(CLASSIC);
//         SHGetSetSettings(&ss, SSF_STARTPANELON, TRUE); return PSNRET_NOERROR; }
// };
//
// class CNotifySettingsPage : public CPropertyPageImpl<CNotifySettingsPage>
// {
//     HBITMAP m_hbmpTray; HWND m_hwndTaskbar;
//     static const WORD wImageIdLookupTable[2][2][2][2];
//     void _UpdateDialog() { bShowClock/bShowSeconds/bHideInactive/bShowDesktopButton;
//         EnableWindow(customizeNotify, bHideInactive); EnableWindow(seconds, bShowClock);
//         if (!bShowSeconds) CheckDlgButton(SECONDS, UNCHECKED);
//         uImageId = wImageIdLookupTable[bShowClock][bShowSeconds][bHideInactive][bShowDesktopButton];
//         SetBitmap(trayBitmap, &m_hbmpTray, uImageId); }
//   public:
//     enum { IDD = IDD_TASKBARPROP_NOTIFY };
//     BEGIN_MSG_MAP(CNotifySettingsPage)
//         MESSAGE_HANDLER(WM_INITDIALOG, OnInitDialog)
//         COMMAND_ID_HANDLER(IDC_TASKBARPROP_ICONCUST, OnCustomizeTrayIcons)
//         COMMAND_RANGE_HANDLER(IDC_TASKBARPROP_NOTIFY_FIRST_CMD, IDC_TASKBARPROP_NOTIFY_LAST_CMD, OnCtrlCommand)
//         CHAIN_MSG_MAP(CPropertyPageImpl<CNotifySettingsPage>)
//     END_MSG_MAP()
//     LRESULT OnInitDialog(…) { EnableWindow(GetDlgItem(CLOCK), !SHRestricted(REST_HIDECLOCK));
//         CheckDlgButton(CLOCK, !GetHideClock()); CheckDlgButton(SECONDS, bShowSeconds);
//         CheckDlgButton(HIDEICONS, bHideInactiveIcons); CheckDlgButton(DESKTOP, bShowDesktopButton);
//         CheckDlgButton(HIDEICONS, UNCHECKED); EnableWindow(GetDlgItem(HIDEICONS), FALSE);
//         _UpdateDialog(); return TRUE; }
//     LRESULT OnCustomizeTrayIcons(…) { ShowCustomizeNotifyIcons(hExplorerInstance, m_hWnd); return 0; }
//     LRESULT OnCtrlCommand(…) { _UpdateDialog(); SetModified(TRUE); return 0; }
//     int OnApply() { TaskbarSettings n = g_TaskbarSettings; n.sr.HideClock = !IsDlgButtonChecked(CLOCK);
//         n.bShowSeconds=…; n.bHideInactiveIcons=…; n.bShowDesktopButton=…;
//         SendMessage(m_hwndTaskbar, TWM_SETTINGSCHANGED, 0, (LPARAM)&n); return PSNRET_NOERROR; }
// };
//
// static int CALLBACK PropSheetProc(HWND hwndDlg, UINT uMsg, LPARAM lParam) {
//     switch (uMsg) { case PSCB_INITIALIZED:
//         hIcon = LoadIconW(hExplorerInstance, MAKEINTRESOURCEW(IDI_STARTMENU));
//         SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon); break; }
//     return 0;
// }
//
// VOID DisplayTrayProperties(IN HWND hwndOwner, IN HWND hwndTaskbar) {
//     PROPSHEETHEADER psh; CSimpleArray<HPROPSHEETPAGE> hpsp;
//     CTaskBarSettingsPage tbSettingsPage(hwndTaskbar);
//     CStartMenuSettingsPage smSettingsPage;
//     CNotifySettingsPage naSettingsPage(hwndTaskbar);
//     CStringW caption; caption.LoadStringW(IDS_TASKBAR_STARTMENU_PROP_CAPTION);
//     hpsp.Add(tbSettingsPage.Create()); hpsp.Add(smSettingsPage.Create()); hpsp.Add(naSettingsPage.Create());
//     ZeroMemory(&psh, sizeof(psh));
//     psh.dwSize = sizeof(psh);
//     psh.dwFlags = PSH_PROPTITLE | PSH_USEICONID | PSH_USECALLBACK;
//     psh.hwndParent = hwndOwner; psh.hInstance = hExplorerInstance;
//     psh.pszIcon = MAKEINTRESOURCEW(IDI_STARTMENU); psh.pszCaption = caption.GetString();
//     psh.nPages = hpsp.GetSize(); psh.nStartPage = 0; psh.phpage = hpsp.GetData();
//     psh.pfnCallback = PropSheetProc;
//     PropertySheet(&psh);
// }
//
// ============================================================================
// END ORIGINAL C++ SOURCE
// ============================================================================
