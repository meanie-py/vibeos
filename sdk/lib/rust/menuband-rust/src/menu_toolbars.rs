//! CMenuToolbars.cpp / CMenuToolbars.h — CMenuToolbarBase (the bespoke, non-COM base class
//! that wraps a comctl32 toolbar, plus an optional pager) and its two single-inheritance
//! derived classes CMenuStaticToolbar (menu-driven) and CMenuSFToolbar (shell-folder-driven).
//! Hand-translated: the constructors on both base and derived, and the bespoke virtual table
//! (InternalPopupItem/InternalGetTooltip/InternalHasSubMenu/InternalContextMenu/
//! InternalExecuteItem/OnDeletingButton/FillToolbar), put it outside the deterministic
//! scaffold. Publishes the 29 `*_RustBridge` entry points `menu_band.rs` declares and calls
//! back into the Rust-owned `CMenuBand` through the `CMenuBand_*_RustExport` externs
//! (CMenuBandRustExports.h).
//!
//! VIRTUAL DISPATCH. C++ gives CMenuToolbarBase five PURE virtuals plus one more
//! (OnDeletingButton) that only the derived classes implement. There is no vtable here: a
//! `kind: ToolbarKind` tag on the base struct plus a `match` at each of the six call sites
//! plays the same role, cheaper than a fn-pointer table because every caller already has a
//! concrete `*mut CMenuToolbarBase` and the two derived bodies are all in this one file (see
//! the `dispatch_*` functions below `CMenuToolbarBase`'s method impls).
//!
//! CTOR/DTOR CHAINING. `CMenuStaticToolbar_new`/`CMenuSFToolbar_new` HeapAlloc, zero, run the
//! base ctor body into the embedded `base: CMenuToolbarBase` field, then the derived ctor body
//! — mirroring C++'s base-then-derived member-init order. `_delete` runs the derived dtor body
//! then the base dtor body then frees — the reverse, mirroring C++'s derived-then-base
//! destruction. `Create_RustBridge`/`Delete_RustBridge` are the allocation points (mirrors
//! `new CMenuStaticToolbar(this)` / `delete m_staticToolbar` in the old CMenuBand.cpp).
//!
//! WINDOW SUBCLASSING. Neither the toolbar ("ToolbarWindow32") nor the pager ("SysPager") is
//! OUR window class — `CreateToolbar` creates both with `CreateWindowExW` against the existing
//! comctl32 classes and then SUBCLASSES them, so `shell32-rust/src/atlwin.rs`'s
//! `WM_NCCREATE`/`lpCreateParams` mechanism (built for a window OUR OWN class registers) does
//! not apply — a subclassed window never gets `WM_NCCREATE` again. Instead this file reproduces
//! ATL's `SubclassWindow` directly: `GWLP_USERDATA` holds `*mut CMenuToolbarBase` (verified
//! safe — `dll/win32/comctl32/toolbar.c` and `pager.c` never touch `GWLP_USERDATA`, matching
//! the precedent `atlwin.rs`'s own `AtlSubclassWindow` already relies on), and the window proc
//! we install chains anything it doesn't claim to the SAVED ORIGINAL proc via
//! `CallWindowProcW`, exactly what `CWindowImplBaseT::DefWindowProc` does for a subclassed
//! window. There are TWO independent subclass slots on one base object — `tb_orig_proc` for the
//! toolbar and `pager_orig_proc` for the pager — because `m_pager` is a second, separate
//! `CContainedWindow` in the C++, not a second CMenuToolbarBase.
//!
//! THE PAGER'S ROUTING. `CContainedWindow(WC_PAGESCROLLER, this)` forwards every message the
//! pager window receives into the OWNER's (`this` = the toolbar object's) message map — the
//! same `BEGIN_MSG_MAP` as the toolbar itself, map id 0 (no second map is declared). Concretely
//! that means `PGN_CALCSIZE` (WM_NOTIFY from the pager) reaches the very same
//! `OnWinEventWrap`/`OnWinEvent` this file installs for the toolbar. Crucially, NONE of the
//! `BEGIN_MSG_MAP` handler signatures take an `hWnd` parameter (ATL's `MESSAGE_HANDLER` binds
//! `this` implicitly) — `OnWinEvent` explicitly reads `m_hWnd` (the TOOLBAR's own handle), not
//! whichever window physically received the message. So one shared dispatcher
//! (`process_window_message`) serves both windows unmodified; only the "which original proc do
//! I chain to, and which HWND field do I clear on WM_NCDESTROY" bookkeeping differs, and that
//! lives in the two thin per-window wndproc trampolines (`tb_wndproc`, `pager_wndproc`).

#![allow(non_snake_case)]

use crate::prelude::*;
// One declaration per external symbol: island.rs.
pub use crate::island::StrRetToStrW;
// One declaration per external symbol: the variant matching the C prototype.
use crate::menu_desk_bar::GetWindowLongPtrW;
use crate::menu_desk_bar::SetWindowLongPtrW;
use crate::menu_band::{
    CppComPtrGuard, IContextMenu, IContextMenuVtbl, IEnumIDList, IEnumIDListVtbl, IShellFolder,
    IShellFolderVtbl, IShellMenu, IShellMenuVtbl, IShellMenuCallback, IShellMenuCallbackVtbl,
    IID_IContextMenu, IID_IShellMenu,
};
use crate::shellcom::{STRRET, STRRET_CSTR};

// ── CMenuBand: opaque back-pointer, never dereferenced here ─────────────────────────────────
// Mirrors CMenuBandRustExports.h's own `class CMenuBand;` forward declaration: every operation
// on it goes through a CMenuBand_*_RustExport call, never a field/member access. A pointer is a
// pointer at the C ABI, so this local opaque type need not (and must not) match the real
// 12-interface `crate::menu_band::CMenuBand` layout byte for byte.
#[repr(C)]
pub struct CMenuBand {
    _opaque: [u8; 0],
}

// ── Constants missing from the bindgen island ────────────────────────────────────────────────
// The island's derive run has never seen this TU (it was still C++), so anything only this file
// references was never extracted. Each value below is checked against its header.

/// shobjidl.idl `typedef enum tagMPOS { MPOS_EXECUTE = 0, ... } MPOS;` — CMenuBand.cpp's
/// `MenuItemSelect(MPOS_EXECUTE)` call needs this constant, which no already-ported TU used.
const MPOS_EXECUTE: DWORD = 0;
/// shtypes.h `SHCONTF_FOLDERS = 0x20`, `SHCONTF_NONFOLDERS = 0x40`.
const SHCONTF_FOLDERS: SHCONTF = 0x0020;
const SHCONTF_NONFOLDERS: SHCONTF = 0x0040;
/// shobjidl.idl `SIGDN_NORMALDISPLAY = 0`. The C++ passes this SIGDN value as the `uFlags`
/// (SHGDNF) argument of `IShellFolder::GetDisplayNameOf` — almost certainly meant
/// `SHGDN_NORMAL` (also 0), but the two share a numeric value so the C++ behaves correctly by
/// coincidence. Preserved as written, not "fixed".
const SIGDN_NORMALDISPLAY: DWORD = 0;
/// commctrl.h generic notification codes (`NM_FIRST = 0U`, counting down). Only the ones this
/// TU's `switch` actually names — the rest fall through its `default:` arm either way.
const NM_CUSTOMDRAW: i32 = -12;
const NM_RELEASEDCAPTURE: i32 = -16;
const NM_TOOLTIPSCREATED: i32 = -19;
const NM_LDOWN: i32 = -20;
const NM_RDOWN: i32 = -21;
const NM_CLICK: i32 = -2;
/// CMenuToolbars.cpp:31 `#define IDS_MENU_EMPTY 34561`.
const IDS_MENU_EMPTY: UINT = 34561;
/// CMenuToolbars.cpp:39 `#define TIMERID_HOTTRACK 1` — user timer id for hot-track delay.
const TIMERID_HOTTRACK: UINT_PTR = 1;

// shellutils.h `SHELL_GetAttributesOf(psf, pidlChild, query)`: the queried bits that are set,
// or 0 when the folder fails the call.
unsafe fn SHELL_GetAttributesOf(psf: *mut IShellFolder, pidlChild: LPCITEMIDLIST, query: SFGAOF) -> SFGAOF {
    if psf.is_null() {
        return 0;
    }
    let mut attr: SFGAOF = query;
    let hr = ((*(*psf).lpVtbl).GetAttributesOf)(psf, 1, &pidlChild, &mut attr);
    if hr >= 0 { attr & query } else { 0 }
}

// A stream (.zip/.cab) can also report SFGAO_FOLDER; it is still a file, not a menu to expand.
fn GetFolderItemBaseType(sfgao: SFGAOF) -> SFGAOF {
    if sfgao & SFGAO_STREAM as SFGAOF != 0 { SFGAO_STREAM as SFGAOF } else { sfgao & SFGAO_FOLDER as SFGAOF }
}

unsafe fn GetFolderItemBaseTypeOf(psf: *mut IShellFolder, pidl: LPCITEMIDLIST) -> SFGAOF {
    GetFolderItemBaseType(SHELL_GetAttributesOf(psf, pidl, (SFGAO_FOLDER | SFGAO_STREAM) as SFGAOF))
}
/// shellutils.h `MAX_PATH` (260) — declared locally rather than pulled from a hand-written
/// module whose own `MAX_PATH` might be a different width; only the value matters here.
const MAX_PATH_: usize = 260;

/// The island's `TRUE`/`FALSE` are untyped `u32` constants; `BOOL` is `c_int` (`i32`). Shadow
/// them locally so every `BOOL`-typed comparison/assignment in this file needs no per-site
/// cast — a local `const` item outranks the glob-imported `u32` ones of the same name.
const TRUE: BOOL = 1;
const FALSE: BOOL = 0;

/// commoncontrols.h — not in the island (nothing else ported here has needed it yet).
// commoncontrols.h — not in the island (nothing else ported here has needed it yet).
// shlobj.h `SHGetPathFromIDListW` — a real shell32 export; not in the island under this exact
// spelling (the island only carries the internal `SHGetPathFromIDListWrapW`).
// shtypes.h `StrRetToStrW` — declared here against `shellcom::STRRET`, matching
// `IShellFolder::GetDisplayNameOf`'s own output type. The island's OWN `StrRetToStrW` takes its
// own, differently-nominal `STRRET` (byte-identical layout, not the same Rust type), which is
// why rustc warns about a signature mismatch on this redeclaration — harmless, same real ABI.
// winuser.h `#define`s GetWindowLongPtrW/SetWindowLongPtrW to GetWindowLongW/SetWindowLongW on
// 32-bit targets: user32 exports no *Ptr* entry points there, so the island has nothing to
// offer on i386 and the build fails to resolve them. A local declaration shadows the island's
// glob on 64-bit too, which keeps one spelling for all three arches. menu_desk_bar.rs and
// menu_site.rs carry the same two declarations privately.
extern "system" {
}

extern "system" {
    fn SHGetImageList(iImageList: INT, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT;
    fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, pszPath: LPWSTR) -> BOOL;
}
extern "C" {
    static IID_IImageList: IID;
}

/// commctrl.h macros the island renders as opaque (`_DPA { _unused: [u8; 0] }`) because they
/// are not real exports: `#define DPA_AppendPtr(hdpa,p) DPA_InsertPtr(hdpa,0x7FFFFFFF,p)` and
/// `#define DPA_GetPtrCount(hdpa) (*(int*)(hdpa))` — the count is the struct's first field,
/// read straight through the pointer since bindgen gives the type no fields to name.
#[inline]
unsafe fn DPA_AppendPtr(hdpa: HDPA, p: PVOID) -> INT {
    DPA_InsertPtr(hdpa, 0x7FFF_FFFF, p)
}
#[inline]
unsafe fn DPA_GetPtrCount(hdpa: HDPA) -> INT {
    *(hdpa as *const INT)
}

/// wine/debug.h `UNIMPLEMENTED` — a stub body, on by default like ERR/FIXME (see winedebug.rs).
/// No module in this crate has needed it yet, so it is defined here rather than centrally.
macro_rules! UNIMPLEMENTED {
    () => {
        unsafe {
            winedebug_rust::ffi::DbgPrint(
                concat!("fixme:(", file!(), ":", line!(), ") UNIMPLEMENTED\n\0").as_ptr() as *const _,
            )
        }
    };
}

// windowsx.h GET_X_LPARAM/GET_Y_LPARAM — no island equivalent (macros, not symbols).
#[inline]
fn GET_X_LPARAM(lp: LPARAM) -> i32 {
    (lp as i32) as i16 as i32
}
#[inline]
fn GET_Y_LPARAM(lp: LPARAM) -> i32 {
    ((lp as i32) >> 16) as i16 as i32
}

/// winerror.h `HRESULT_FROM_WIN32` macro.
#[inline]
fn HRESULT_FROM_WIN32(x: DWORD) -> HRESULT {
    if (x as i32) <= 0 {
        x as HRESULT
    } else {
        ((x & 0x0000_FFFF) | (7 << 16) | 0x8000_0000) as HRESULT // FACILITY_WIN32 == 7
    }
}

// Wide-literal window class names / strings the crate's convention writes out explicitly
// (the island renders `TOOLBARCLASSNAMEW`/`WC_PAGESCROLLERW` as ANSI byte strings, which would
// make CreateWindowExW read garbage).
static TOOLBARCLASSNAME_W: [u16; 16] = [
    'T' as u16, 'o' as u16, 'o' as u16, 'l' as u16, 'b' as u16, 'a' as u16, 'r' as u16,
    'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, '3' as u16,
    '2' as u16, 0,
];
static WC_PAGESCROLLER_W: [u16; 9] = [
    'S' as u16, 'y' as u16, 's' as u16, 'P' as u16, 'a' as u16, 'g' as u16, 'e' as u16,
    'r' as u16, 0,
];
static MARLETT_W: [u16; 8] = [
    'M' as u16, 'a' as u16, 'r' as u16, 'l' as u16, 'e' as u16, 't' as u16, 't' as u16, 0,
];
static EMPTY_W: [u16; 1] = [0];

// ── CMenuBand_*_RustExport (CMenuBandRustExports.h) ─────────────────────────────────────────
// Every one of these is DEFINED in menu_band.rs; only declared here. Opaque `CMenuBand`
// pointer convention per that header — never dereferenced, only threaded through.
extern "system" {
    fn CMenuBand_GetMenuInfo_RustExport(
        band: *mut CMenuBand,
        ppsmc: *mut *mut IShellMenuCallback,
        puId: *mut UINT,
        puIdAncestor: *mut UINT,
        pdwFlags: *mut DWORD,
    ) -> HRESULT;
    fn CMenuBand_UseBigIcons_RustExport(band: *mut CMenuBand) -> BOOL;
    fn CMenuBand_AdjustForTheme_RustExport(band: *mut CMenuBand, bFlatStyle: BOOL) -> HRESULT;
    fn CMenuBand_CallCBWithItemId_RustExport(
        band: *mut CMenuBand,
        Id: UINT,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> HRESULT;
    fn CMenuBand_CallCBWithItemPidl_RustExport(
        band: *mut CMenuBand,
        pidl: LPITEMIDLIST,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> HRESULT;
    fn CMenuBand_TrackSubMenu_RustExport(
        band: *mut CMenuBand,
        popup: HMENU,
        x: INT,
        y: INT,
        rcExclude: *mut RECT,
    ) -> HRESULT;
    fn CMenuBand_TrackContextMenu_RustExport(
        band: *mut CMenuBand,
        popup: *mut IContextMenu,
        x: INT,
        y: INT,
    ) -> HRESULT;
    fn CMenuBand_GetTopLevelWindow_RustExport(band: *mut CMenuBand, topLevel: *mut HWND) -> HRESULT;
    fn CMenuBand_ChangeHotItem_RustExport(
        band: *mut CMenuBand,
        tb: *mut CMenuToolbarBase,
        id: INT,
        dwFlags: DWORD,
    ) -> HRESULT;
    fn CMenuBand_ChangePopupItem_RustExport(
        band: *mut CMenuBand,
        tb: *mut CMenuToolbarBase,
        id: INT,
    ) -> HRESULT;
    fn CMenuBand_MenuItemSelect_RustExport(band: *mut CMenuBand, changeType: DWORD) -> HRESULT;
    fn CMenuBand_CancelCurrentPopup_RustExport(band: *mut CMenuBand) -> HRESULT;
    fn CMenuBand_OnPopupSubMenu_RustExport(
        band: *mut CMenuBand,
        childShellMenu: *mut IShellMenu,
        pAt: *mut POINTL,
        pExclude: *mut RECTL,
        keyInitiated: BOOL,
    ) -> HRESULT;
    #[allow(dead_code)]
    fn CMenuBand_BeforeCancelPopup_RustExport(band: *mut CMenuBand) -> HRESULT;
    #[allow(dead_code)]
    fn CMenuBand_DisableMouseTrack_RustExport(band: *mut CMenuBand, bDisable: BOOL) -> HRESULT;
    fn CMenuBand_KillPopupTimers_RustExport(band: *mut CMenuBand) -> HRESULT;
    #[allow(dead_code)]
    fn CMenuBand_MenuBarMouseDown_RustExport(
        band: *mut CMenuBand,
        hwnd: HWND,
        item: INT,
        isLButton: BOOL,
    ) -> HRESULT;
    #[allow(dead_code)]
    fn CMenuBand_MenuBarMouseUp_RustExport(
        band: *mut CMenuBand,
        hwnd: HWND,
        item: INT,
        isLButton: BOOL,
    ) -> HRESULT;
    #[allow(dead_code)]
    fn CMenuBand_KeyboardItemChange_RustExport(band: *mut CMenuBand, change: DWORD) -> HRESULT;

    // Companion creator (shellmenu.h:80 `#define CMenuBand_CreateInstance
    // RSHELL_CMenuBand_CreateInstance`) — InternalPopupItem uses it to spin up the child
    // IShellMenu for a folder submenu, exactly as the C++ macro-expanded call did.
    fn RSHELL_CMenuBand_CreateInstance(riid: REFIID, ppv: *mut LPVOID) -> HRESULT;

    // ── Win32/shell imports this TU needs that the island never picked up ──
    fn SHMapPIDLToSystemImageListIndex(
        psf: *mut IShellFolder,
        pidl: LPCITEMIDLIST,
        pIndex: *mut INT,
    ) -> INT;
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
}

// ── CToolbar<DWORD_PTR> — thin SendMessage wrappers (rosctrls.h) ────────────────────────────
// Ported as free functions taking the toolbar HWND directly (this class has no Rust wrapper
// elsewhere in the tree yet; shell32-rust/src/rosctrls.rs only carries CListView). Message ids
// and WPARAM/LPARAM packing copied exactly from rosctrls.h's CToolbar<TItemData>.

#[inline]
unsafe fn tbsend(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT {
    SendMessageW(hwnd, msg, w, l)
}

unsafe fn tb_set_button_struct_size(hwnd: HWND) -> LRESULT {
    tbsend(hwnd, TB_BUTTONSTRUCTSIZE, core::mem::size_of::<TBBUTTON>() as WPARAM, 0)
}
unsafe fn tb_get_button_count(hwnd: HWND) -> INT {
    tbsend(hwnd, TB_BUTTONCOUNT, 0, 0) as INT
}
unsafe fn tb_get_button(hwnd: HWND, index: INT, btn: *mut TBBUTTON) -> LRESULT {
    tbsend(hwnd, TB_GETBUTTON, index as WPARAM, btn as LPARAM)
}
unsafe fn tb_add_buttons(hwnd: HWND, count: INT, buttons: *mut TBBUTTON) -> LRESULT {
    tbsend(hwnd, TB_ADDBUTTONSW, count as WPARAM, buttons as LPARAM)
}
unsafe fn tb_delete_button(hwnd: HWND, index: INT) -> LRESULT {
    tbsend(hwnd, TB_DELETEBUTTON, index as WPARAM, 0)
}
unsafe fn tb_get_button_info(hwnd: HWND, cmdId: INT, info: *mut TBBUTTONINFOW) -> LRESULT {
    tbsend(hwnd, TB_GETBUTTONINFOW, cmdId as WPARAM, info as LPARAM)
}
unsafe fn tb_check_button(hwnd: HWND, cmdId: INT, check: BOOL) -> LRESULT {
    tbsend(hwnd, TB_CHECKBUTTON, cmdId as WPARAM, MAKELPARAM!(check, 0))
}
unsafe fn tb_get_button_size(hwnd: HWND) -> DWORD {
    tbsend(hwnd, TB_GETBUTTONSIZE, 0, 0) as DWORD
}
unsafe fn tb_set_button_size(hwnd: HWND, w: INT, h: INT) -> LRESULT {
    tbsend(hwnd, TB_SETBUTTONSIZE, 0, MAKELPARAM!(w, h))
}
unsafe fn tb_get_max_size(hwnd: HWND, size: *mut SIZE) -> LRESULT {
    tbsend(hwnd, TB_GETMAXSIZE, 0, size as LPARAM)
}
unsafe fn tb_get_ideal_size(hwnd: HWND, useHeight: BOOL, size: *mut SIZE) -> LRESULT {
    tbsend(hwnd, TB_GETIDEALSIZE, useHeight as WPARAM, size as LPARAM)
}
unsafe fn tb_get_item_rect(hwnd: HWND, index: INT, rc: *mut RECT) -> LRESULT {
    tbsend(hwnd, TB_GETITEMRECT, index as WPARAM, rc as LPARAM)
}
unsafe fn tb_set_image_list(hwnd: HWND, himl: HIMAGELIST) -> HIMAGELIST {
    tbsend(hwnd, TB_SETIMAGELIST, 0, himl as LPARAM) as HIMAGELIST
}
unsafe fn tb_set_padding(hwnd: HWND, x: INT, y: INT) -> LRESULT {
    tbsend(hwnd, TB_SETPADDING, 0, MAKELPARAM!(x, y))
}
unsafe fn tb_set_hot_item(hwnd: HWND, item: INT) -> LRESULT {
    tbsend(hwnd, TB_SETHOTITEM, item as WPARAM, 0)
}
unsafe fn pg_set_child(hwnd: HWND, child: HWND) -> LRESULT {
    tbsend(hwnd, PGM_SETCHILD, 0, child as LPARAM)
}

// ── The bespoke non-COM "vtable" tag ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
enum ToolbarKind {
    Static,
    SF,
}

// ── CMenuToolbarBase (CMenuToolbars.h:32-143) ────────────────────────────────────────────────
// Field order does not need to match the C++ layout (nothing outside this file reaches into it
// by offset — every cross-language access is a *_RustBridge call), EXCEPT that a derived
// struct's `base` field must be FIRST so the `as *mut CMenuToolbarBase` upcast menu_band.rs
// performs stays address-preserving.
#[repr(C)]
pub struct CMenuToolbarBase {
    kind: ToolbarKind,

    // -- toolbar window / subclass state (ATL CWindowImplBaseT's own m_hWnd) --
    m_hWnd: HWND,
    tb_orig_proc: WNDPROC,

    // -- m_pager (CContainedWindow) --
    m_pager_hwnd: HWND,
    pager_orig_proc: WNDPROC,
    m_usePager: BOOL,

    // -- private --
    m_marlett: HFONT,
    m_useFlatMenus: BOOL,
    m_disableMouseTrack: BOOL,
    m_timerEnabled: BOOL,

    // -- protected --
    m_menuBand: *mut CMenuBand,
    m_dwMenuFlags: DWORD,
    m_hasSizes: BOOL,
    m_idealSize: SIZE,
    m_itemSize: SIZE,
    m_hotBar: *mut CMenuToolbarBase,
    m_hotItem: INT,
    m_popupBar: *mut CMenuToolbarBase,
    m_popupItem: INT,

    m_initFlags: DWORD,
    m_isTrackingPopup: BOOL,

    m_executeIndex: INT,
    m_executeItem: INT,
    m_executeData: DWORD_PTR,

    m_cancelingPopup: BOOL,
}

#[repr(C)]
pub struct CMenuStaticToolbar {
    base: CMenuToolbarBase,
    m_hmenu: HMENU,
    m_hwndMenu: HWND,
}

#[repr(C)]
pub struct CMenuSFToolbar {
    base: CMenuToolbarBase,
    /// `CComPtr<IShellFolder>` — an owning reference, AddRef'd in `SetShellFolder`, Release'd
    /// in the destructor (mirrors the CComPtr member's dtor).
    m_shellFolder: *mut IShellFolder,
    m_idList: LPITEMIDLIST,
    m_hKey: HKEY,
}

// ── construction / destruction ───────────────────────────────────────────────────────────────

unsafe fn heap_alloc_zeroed<T>() -> *mut T {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<T>() as SIZE_T) as *mut T
}

/// `CMenuToolbarBase::CMenuToolbarBase(menuBand, usePager)` member-init list + body.
unsafe fn CMenuToolbarBase_ctor_body(this: *mut CMenuToolbarBase, kind: ToolbarKind, menuBand: *mut CMenuBand, usePager: BOOL) {
    (*this).kind = kind;
    (*this).m_hWnd = core::ptr::null_mut();
    (*this).tb_orig_proc = None;
    (*this).m_pager_hwnd = core::ptr::null_mut();
    (*this).pager_orig_proc = None;
    (*this).m_usePager = usePager;
    (*this).m_useFlatMenus = FALSE;
    (*this).m_disableMouseTrack = FALSE;
    (*this).m_timerEnabled = FALSE;
    (*this).m_menuBand = menuBand;
    (*this).m_dwMenuFlags = 0;
    (*this).m_hasSizes = FALSE;
    (*this).m_idealSize = SIZE { cx: 0, cy: 0 };
    (*this).m_itemSize = SIZE { cx: 0, cy: 0 };
    (*this).m_hotBar = core::ptr::null_mut();
    (*this).m_hotItem = -1;
    (*this).m_popupBar = core::ptr::null_mut();
    (*this).m_popupItem = -1;
    (*this).m_initFlags = 0;
    (*this).m_isTrackingPopup = FALSE;
    (*this).m_executeIndex = 0;
    (*this).m_executeItem = 0;
    (*this).m_executeData = 0;
    (*this).m_cancelingPopup = FALSE;
    (*this).m_marlett = CreateFontW(
        0, 0, 0, 0, 0, 0, 0, 0,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, DEFAULT_QUALITY, FF_DONTCARE,
        MARLETT_W.as_ptr(),
    );
}

/// `CMenuToolbarBase::~CMenuToolbarBase()`.
unsafe fn CMenuToolbarBase_dtor_body(this: *mut CMenuToolbarBase) {
    if !(*this).m_hWnd.is_null() {
        CMenuToolbarBase_ClearToolbar(this);
        DestroyWindow((*this).m_hWnd);
    }
    if !(*this).m_pager_hwnd.is_null() {
        DestroyWindow((*this).m_pager_hwnd);
    }
    if !(*this).m_marlett.is_null() {
        DeleteObject((*this).m_marlett as HGDIOBJ);
    }
}

/// `new CMenuStaticToolbar(menuBand)` — mirrors old CMenuBand.cpp:140.
unsafe fn CMenuStaticToolbar_new(menuBand: *mut CMenuBand) -> *mut CMenuStaticToolbar {
    let this: *mut CMenuStaticToolbar = heap_alloc_zeroed();
    if this.is_null() {
        return core::ptr::null_mut();
    }
    CMenuToolbarBase_ctor_body(core::ptr::addr_of_mut!((*this).base), ToolbarKind::Static, menuBand, FALSE);
    (*this).m_hmenu = core::ptr::null_mut();
    (*this).m_hwndMenu = core::ptr::null_mut();
    this
}

unsafe fn CMenuStaticToolbar_delete(this: *mut CMenuStaticToolbar) {
    if this.is_null() {
        return;
    }
    // CMenuStaticToolbar::~CMenuStaticToolbar() has an empty body.
    CMenuToolbarBase_dtor_body(core::ptr::addr_of_mut!((*this).base));
    HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
}

/// `new CMenuSFToolbar(menuBand)` — mirrors old CMenuBand.cpp:692.
unsafe fn CMenuSFToolbar_new(menuBand: *mut CMenuBand) -> *mut CMenuSFToolbar {
    let this: *mut CMenuSFToolbar = heap_alloc_zeroed();
    if this.is_null() {
        return core::ptr::null_mut();
    }
    CMenuToolbarBase_ctor_body(core::ptr::addr_of_mut!((*this).base), ToolbarKind::SF, menuBand, TRUE);
    (*this).m_shellFolder = core::ptr::null_mut();
    (*this).m_idList = core::ptr::null_mut();
    (*this).m_hKey = core::ptr::null_mut();
    this
}

unsafe fn CMenuSFToolbar_delete(this: *mut CMenuSFToolbar) {
    if this.is_null() {
        return;
    }
    // CMenuSFToolbar::~CMenuSFToolbar() has an empty body; the CComPtr<IShellFolder> member's
    // own destructor is what actually releases m_shellFolder, so that release belongs here.
    if !(*this).m_shellFolder.is_null() {
        let unk = (*this).m_shellFolder as *mut IUnknown;
        ((*(*unk).lpVtbl).Release)(unk);
    }
    CMenuToolbarBase_dtor_body(core::ptr::addr_of_mut!((*this).base));
    HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
}

// ── window procedure / subclassing ───────────────────────────────────────────────────────────

/// `CWindowImplBaseT::DefWindowProc` for a SUBCLASSED window: chain to the saved original proc
/// rather than `DefWindowProcW` (there is always a saved proc once subclassed successfully).
#[inline]
unsafe fn call_super(orig: WNDPROC, hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match orig {
        Some(_) => CallWindowProcW(orig, hwnd, uMsg, wParam, lParam),
        None => DefWindowProcW(hwnd, uMsg, wParam, lParam),
    }
}

/// The `BEGIN_MSG_MAP(CMenuToolbarBase)` table (CMenuToolbars.h:136-142), shared by the toolbar
/// AND the pager subclass (see the module header — the pager forwards into this same map).
/// Returns `TRUE` if a handler claimed the message.
unsafe fn process_window_message(
    this: *mut CMenuToolbarBase,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    lResult: *mut LRESULT,
) -> BOOL {
    let mut bHandled: BOOL = TRUE;
    *lResult = match uMsg {
        WM_USER_ISTRACKEDITEM => CMenuToolbarBase_IsTrackedItem(this, wParam, lParam, &mut bHandled),
        WM_USER_CHANGETRACKEDITEM => CMenuToolbarBase_ChangeTrackedItem(this, wParam, lParam, &mut bHandled),
        WM_COMMAND | WM_NOTIFY => CMenuToolbarBase_OnWinEventWrap(this, uMsg, wParam, lParam, &mut bHandled),
        WM_TIMER => CMenuToolbarBase_OnPopupTimer(this, wParam, &mut bHandled),
        _ => {
            bHandled = FALSE;
            0
        }
    };
    bHandled
}

/// Common tail for both window procs: dispatch through the message map, chain unhandled
/// messages to the saved original proc, and on `WM_NCDESTROY` restore that proc and clear the
/// slot — the same cleanup `shell32-rust/src/atlwin.rs`'s `AtlWindowProc` does for
/// `WINSTATE_DESTROYED`, simplified here because neither handler in this map re-enters the
/// window proc, so the dispatch-depth bookkeeping ATL needs for reentrancy is not needed.
unsafe fn dispatch(
    this: *mut CMenuToolbarBase,
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    orig_slot: *mut WNDPROC,
    hwnd_slot: *mut HWND,
) -> LRESULT {
    let mut lResult: LRESULT = 0;
    let handled = process_window_message(this, uMsg, wParam, lParam, &mut lResult);

    if handled == FALSE {
        lResult = call_super(*orig_slot, hwnd, uMsg, wParam, lParam);
    }

    if uMsg == WM_NCDESTROY {
        if !(*orig_slot).is_none() {
            SetWindowLongPtrW((hwnd).cast(), GWLP_WNDPROC, (core::mem::transmute::<WNDPROC, LONG_PTR>(*orig_slot)) as _);
        }
        SetWindowLongPtrW((hwnd).cast(), GWLP_USERDATA, 0);
        *orig_slot = None;
        *hwnd_slot = core::ptr::null_mut();
    }

    lResult
}

unsafe extern "system" fn tb_wndproc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let this = GetWindowLongPtrW((hwnd).cast(), GWLP_USERDATA) as *mut CMenuToolbarBase;
    if this.is_null() {
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }
    dispatch(this, hwnd, uMsg, wParam, lParam, core::ptr::addr_of_mut!((*this).tb_orig_proc), core::ptr::addr_of_mut!((*this).m_hWnd))
}

unsafe extern "system" fn pager_wndproc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let this = GetWindowLongPtrW((hwnd).cast(), GWLP_USERDATA) as *mut CMenuToolbarBase;
    if this.is_null() {
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }
    dispatch(this, hwnd, uMsg, wParam, lParam, core::ptr::addr_of_mut!((*this).pager_orig_proc), core::ptr::addr_of_mut!((*this).m_pager_hwnd))
}

/// `CWindowImplBaseT::SubclassWindow` (atlwin.h:1585), reproduced directly rather than reused
/// from `shell32-rust/src/atlwin.rs` — see the module header for why that file's
/// `AtlSubclassWindow` (built around its own `AtlWindow` trait and single hwnd/state pair) does
/// not fit a class with TWO independently-subclassed windows.
unsafe fn subclass_window(hwnd: HWND, this: *mut CMenuToolbarBase, wndproc: WNDPROC, orig_slot: *mut WNDPROC, hwnd_slot: *mut HWND) -> BOOL {
    if hwnd.is_null() || IsWindow(hwnd) == 0 {
        return FALSE;
    }
    SetWindowLongPtrW((hwnd).cast(), GWLP_USERDATA, this as _);
    let previous = SetWindowLongPtrW((hwnd).cast(), GWLP_WNDPROC, (core::mem::transmute::<WNDPROC, LONG_PTR>(wndproc)) as _);
    if previous == 0 {
        SetWindowLongPtrW((hwnd).cast(), GWLP_USERDATA, 0);
        return FALSE;
    }
    *orig_slot = core::mem::transmute::<LONG_PTR, WNDPROC>((previous) as _);
    *hwnd_slot = hwnd;
    TRUE
}

/// The calling module's own HINSTANCE, resolved by address rather than a per-DLL global —
/// menuband-rust links into both shell32.dll and rshell.dll (see Cargo.toml), the same trick
/// `menu_desk_bar.rs`'s `CMenuDeskBar_CreateWindow` already uses for the same reason.
unsafe fn current_module_hinstance() -> HINSTANCE {
    let mut instance: HINSTANCE = core::ptr::null_mut();
    GetModuleHandleExW(
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        tb_wndproc as usize as LPCWSTR,
        &mut instance,
    );
    instance
}

// ── CMenuToolbarBase methods (CMenuToolbars.h:63-134) ────────────────────────────────────────

unsafe fn CMenuToolbarBase_IsWindowOwner(this: *mut CMenuToolbarBase, hwnd: HWND) -> HRESULT {
    if !(*this).m_hWnd.is_null() && (*this).m_hWnd == hwnd {
        return S_OK;
    }
    if !(*this).m_pager_hwnd.is_null() && (*this).m_pager_hwnd == hwnd {
        return S_OK;
    }
    S_FALSE
}

unsafe fn CMenuToolbarBase_CreateToolbar(this: *mut CMenuToolbarBase, hwndParent: HWND, dwFlags: DWORD) -> HRESULT {
    let mut tbStyles: LONG = (WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN
        | TBSTYLE_TOOLTIPS | TBSTYLE_TRANSPARENT | TBSTYLE_REGISTERDROP | TBSTYLE_LIST | TBSTYLE_FLAT | TBSTYLE_CUSTOMERASE
        | CCS_NODIVIDER | CCS_NOPARENTALIGN | CCS_NORESIZE | CCS_TOP) as LONG;
    let tbExStyles: LONG = (TBSTYLE_EX_DOUBLEBUFFER | WS_EX_TOOLWINDOW) as LONG;

    if dwFlags & SMINIT_VERTICAL != 0 {
        tbStyles |= CCS_VERT as LONG;
        // USE_TBSTYLE_EX_VERTICAL is 0 in the C++ (disabled pending wine comctl support), so
        // TBSTYLE_EX_VERTICAL is deliberately never added here either.
    }

    (*this).m_initFlags = dwFlags;

    let mut rc: RECT = core::mem::zeroed();
    if GetClientRect(hwndParent, &mut rc) == 0 || rc.left == rc.right || rc.top == rc.bottom {
        rc.left = 0;
        rc.top = 0;
        rc.right = 1;
        rc.bottom = 1;
    }

    let instance = current_module_hinstance();

    // HACK & FIXME: CORE-17505 (preserved from the C++ comment).
    let toolbar = CreateWindowExW(
        tbExStyles as DWORD,
        TOOLBARCLASSNAME_W.as_ptr(),
        core::ptr::null(),
        tbStyles as DWORD,
        0, 0, 0, 0,
        hwndParent,
        core::ptr::null_mut(),
        instance,
        core::ptr::null_mut(),
    );
    if toolbar.is_null() {
        return HRESULT_FROM_WIN32(GetLastError());
    }
    tb_set_button_struct_size(toolbar);

    (*this).m_hWnd = core::ptr::null_mut();
    subclass_window(toolbar, this, Some(tb_wndproc), core::ptr::addr_of_mut!((*this).tb_orig_proc), core::ptr::addr_of_mut!((*this).m_hWnd));

    SetWindowTheme((*this).m_hWnd, EMPTY_W.as_ptr(), EMPTY_W.as_ptr());

    SystemParametersInfoW(SPI_GETFLATMENU, 0, core::ptr::addr_of_mut!((*this).m_useFlatMenus) as PVOID, 0);

    CMenuBand_AdjustForTheme_RustExport((*this).m_menuBand, (*this).m_useFlatMenus);

    if (*this).m_usePager != FALSE {
        let pgStyles: DWORD = PGS_VERT | WS_CHILD | WS_VISIBLE;
        let hwndPager = CreateWindowExW(
            0,
            WC_PAGESCROLLER_W.as_ptr(),
            core::ptr::null(),
            pgStyles,
            rc.left, rc.top, rc.right - rc.left, rc.bottom - rc.top,
            hwndParent,
            core::ptr::null_mut(),
            instance,
            core::ptr::null_mut(),
        );

        subclass_window(hwndPager, this, Some(pager_wndproc), core::ptr::addr_of_mut!((*this).pager_orig_proc), core::ptr::addr_of_mut!((*this).m_pager_hwnd));

        SetParent((*this).m_hWnd, (*this).m_pager_hwnd);

        pg_set_child((*this).m_pager_hwnd, (*this).m_hWnd);
    }

    CMenuToolbarBase_UpdateImageLists(this);

    S_OK
}

unsafe fn CMenuToolbarBase_UpdateImageLists(this: *mut CMenuToolbarBase) -> HRESULT {
    if ((*this).m_initFlags & (SMINIT_TOPLEVEL | SMINIT_VERTICAL)) == SMINIT_TOPLEVEL {
        tb_set_image_list((*this).m_hWnd, core::ptr::null_mut());
        return S_OK;
    }

    let shiml: INT;
    if CMenuBand_UseBigIcons_RustExport((*this).m_menuBand) != FALSE {
        shiml = SHIL_LARGE as INT;
        tb_set_padding((*this).m_hWnd, 4, 0);
    } else {
        shiml = SHIL_SMALL as INT;
        tb_set_padding((*this).m_hWnd, 4, 4);
    }

    let mut piml: *mut core::ffi::c_void = core::ptr::null_mut();
    let hr = SHGetImageList(shiml, core::ptr::addr_of!(IID_IImageList), core::ptr::addr_of_mut!(piml));
    if FAILED_UNEXPECTEDLY!(hr) {
        tb_set_image_list((*this).m_hWnd, core::ptr::null_mut());
    } else {
        tb_set_image_list((*this).m_hWnd, piml as HIMAGELIST);
    }
    S_OK
}

unsafe fn CMenuToolbarBase_GetWindow(this: *mut CMenuToolbarBase, phwnd: *mut HWND) -> HRESULT {
    if phwnd.is_null() {
        return E_FAIL;
    }
    if !(*this).m_pager_hwnd.is_null() {
        *phwnd = (*this).m_pager_hwnd;
    } else {
        *phwnd = (*this).m_hWnd;
    }
    S_OK
}

unsafe fn CMenuToolbarBase_ShowDW(this: *mut CMenuToolbarBase, fShow: BOOL) -> HRESULT {
    if (*this).m_hWnd.is_null() {
        return S_FALSE;
    }
    ShowWindow((*this).m_hWnd, if fShow != FALSE { SW_SHOW as INT } else { SW_HIDE as INT });
    CMenuToolbarBase_UpdateImageLists(this);
    SystemParametersInfoW(SPI_GETFLATMENU, 0, core::ptr::addr_of_mut!((*this).m_useFlatMenus) as PVOID, 0);
    S_OK
}

unsafe fn CMenuToolbarBase_Close(this: *mut CMenuToolbarBase) -> HRESULT {
    if !(*this).m_hWnd.is_null() {
        DestroyWindow((*this).m_hWnd);
    }
    if !(*this).m_pager_hwnd.is_null() {
        DestroyWindow((*this).m_pager_hwnd);
    }
    S_OK
}

unsafe fn CMenuToolbarBase_OnWinEventWrap(this: *mut CMenuToolbarBase, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: *mut BOOL) -> LRESULT {
    let mut lr: LRESULT = 0;
    let hr = CMenuToolbarBase_OnWinEvent(this, (*this).m_hWnd, uMsg, wParam, lParam, &mut lr);
    *bHandled = if hr != S_FALSE { TRUE } else { FALSE };
    lr
}

unsafe fn CMenuToolbarBase_OnWinEvent(this: *mut CMenuToolbarBase, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, theResult: *mut LRESULT) -> HRESULT {
    if !theResult.is_null() {
        *theResult = 0;
    }

    match uMsg {
        WM_COMMAND => return S_OK,
        WM_NOTIFY => {
            let hdr = lParam as *const NMHDR;
            let code = (*hdr).code as i32;
            match code {
                _ if code == TBN_DELETINGBUTTON => {
                    return dispatch_on_deleting_button(this, lParam as *const NMTOOLBARW);
                }
                _ if code == PGN_CALCSIZE as i32 => {
                    return CMenuToolbarBase_OnPagerCalcSize(this, lParam as *mut NMPGCALCSIZE);
                }
                _ if code == TBN_DROPDOWN => {
                    return CMenuToolbarBase_ProcessClick(this, (*(lParam as *const NMTOOLBARW)).iItem);
                }
                _ if code == TBN_HOTITEMCHANGE => return S_OK,
                _ if code == NM_CUSTOMDRAW => {
                    let mut result: LRESULT = 0;
                    let hr = CMenuToolbarBase_OnCustomDraw(this, lParam as *mut NMTBCUSTOMDRAW, &mut result);
                    if !theResult.is_null() {
                        *theResult = result;
                    }
                    return hr;
                }
                _ if code == TBN_GETINFOTIPW => {
                    return CMenuToolbarBase_OnGetInfoTip(this, lParam as *mut NMTBGETINFOTIPW);
                }
                _ if code == RBN_CHILDSIZE => return S_OK,
                _ if code == TTN_GETDISPINFOW => return S_OK,
                _ if code == NM_RELEASEDCAPTURE => {}
                _ if code == NM_CLICK || code == NM_RDOWN || code == NM_LDOWN => {}
                _ if code == TBN_GETDISPINFOW => {}
                _ if code == TBN_BEGINDRAG || code == TBN_ENDDRAG => {}
                _ if code == NM_TOOLTIPSCREATED => {}
                _ if code == TBN_DRAGOUT => return S_FALSE,
                _ => {
                    TRACE!("WM_NOTIFY unknown code %d, %d\n", code, (*hdr).idFrom);
                    return S_OK;
                }
            }
            return S_FALSE;
        }
        WM_WININICHANGE => {
            if wParam == SPI_SETFLATMENU as WPARAM {
                SystemParametersInfoW(SPI_GETFLATMENU, 0, core::ptr::addr_of_mut!((*this).m_useFlatMenus) as PVOID, 0);
            }
        }
        _ => {}
    }

    S_FALSE
}

unsafe fn CMenuToolbarBase_DisableMouseTrack(this: *mut CMenuToolbarBase, bDisable: BOOL) -> HRESULT {
    if (*this).m_disableMouseTrack != bDisable {
        (*this).m_disableMouseTrack = bDisable;
        TRACE!("DisableMouseTrack %d\n", bDisable);
    }
    S_OK
}

unsafe fn CMenuToolbarBase_OnPagerCalcSize(this: *mut CMenuToolbarBase, csize: *mut NMPGCALCSIZE) -> HRESULT {
    let mut tbs: SIZE = core::mem::zeroed();
    CMenuToolbarBase_GetSizes(this, core::ptr::null_mut(), &mut tbs, core::ptr::null_mut());
    if (*csize).dwFlag == PGF_CALCHEIGHT {
        (*csize).iHeight = tbs.cy;
    } else if (*csize).dwFlag == PGF_CALCWIDTH {
        (*csize).iWidth = tbs.cx;
    }
    S_OK
}

unsafe fn CMenuToolbarBase_OnCustomDraw(this: *mut CMenuToolbarBase, cdraw: *mut NMTBCUSTOMDRAW, theResult: *mut LRESULT) -> HRESULT {
    match (*cdraw).nmcd.dwDrawStage {
        CDDS_PREPAINT => {
            *theResult = CDRF_NOTIFYITEMDRAW as LRESULT;
            return S_OK;
        }
        CDDS_ITEMPREPAINT => {
            let mut tlw: HWND = core::ptr::null_mut();
            CMenuBand_GetTopLevelWindow_RustExport((*this).m_menuBand, &mut tlw);

            let mut isHot = (*this).m_hotBar == this && (*cdraw).nmcd.dwItemSpec as INT == (*this).m_hotItem;
            let isPopup = (*this).m_popupBar == this && (*cdraw).nmcd.dwItemSpec as INT == (*this).m_popupItem;
            let isActive = GetForegroundWindow() == tlw || (*this).m_popupBar == this;

            if (*this).m_hotItem < 0 && isPopup {
                isHot = true;
            }

            if ((*this).m_useFlatMenus != FALSE && isHot) || ((*this).m_initFlags & SMINIT_VERTICAL) != 0 {
                let rc = (*cdraw).nmcd.rc;
                let hdc = (*cdraw).nmcd.hdc;

                (*cdraw).nmcd.uItemState &= !(CDIS_HOT | CDIS_CHECKED);

                let clrText: COLORREF;
                let bgBrush: HBRUSH;
                if isHot {
                    (*cdraw).nmcd.uItemState |= CDIS_HOT;
                    clrText = GetSysColor(COLOR_HIGHLIGHTTEXT as INT);
                    bgBrush = GetSysColorBrush(if (*this).m_useFlatMenus != FALSE { COLOR_MENUHILIGHT as INT } else { COLOR_HIGHLIGHT as INT });
                } else {
                    clrText = GetSysColor(COLOR_MENUTEXT as INT);
                    bgBrush = GetSysColorBrush(COLOR_MENU as INT);
                }

                FillRect(hdc, &rc, bgBrush);
                SetTextColor(hdc, clrText);

                (*cdraw).clrText = clrText;
                (*cdraw).iListGap += 4;

                *theResult = (CDRF_NOTIFYPOSTPAINT | TBCDRF_NOBACKGROUND | TBCDRF_NOEDGES | TBCDRF_NOOFFSET | TBCDRF_NOMARK | 0x0080_0000) as LRESULT;
            } else {
                (*cdraw).clrText = GetSysColor(if isActive { COLOR_MENUTEXT as INT } else { COLOR_GRAYTEXT as INT });
                (*cdraw).nmcd.uItemState &= !CDIS_HOT;
                if isHot {
                    (*cdraw).nmcd.uItemState |= CDIS_HOT;
                }
                *theResult = 0;
            }
            return S_OK;
        }
        CDDS_ITEMPOSTPAINT => {
            let mut btni: TBBUTTONINFOW = core::mem::zeroed();
            btni.cbSize = core::mem::size_of::<TBBUTTONINFOW>() as UINT;
            btni.dwMask = TBIF_STYLE;
            tb_get_button_info((*this).m_hWnd, (*cdraw).nmcd.dwItemSpec as INT, &mut btni);

            if (btni.fsStyle as u32) & BTNS_DROPDOWN != 0 {
                let mut text: [u16; 2] = ['8' as u16, 0];
                let hFontOld = SelectObject((*cdraw).nmcd.hdc, (*this).m_marlett as HGDIOBJ);
                SetBkMode((*cdraw).nmcd.hdc, TRANSPARENT as INT);

                let mut rc = (*cdraw).nmcd.rc;
                rc.right += 1;

                DrawTextExW((*cdraw).nmcd.hdc, text.as_mut_ptr(), 1, &mut rc, DT_NOCLIP | DT_VCENTER | DT_RIGHT | DT_SINGLELINE, core::ptr::null_mut());

                SelectObject((*cdraw).nmcd.hdc, hFontOld);
            }
            *theResult = TRUE as LRESULT;
            return S_OK;
        }
        _ => {
            *theResult = 0;
        }
    }
    S_OK
}

unsafe fn CMenuToolbarBase_GetSizes(this: *mut CMenuToolbarBase, pMinSize: *mut SIZE, pMaxSize: *mut SIZE, pIntegralSize: *mut SIZE) -> HRESULT {
    if !pMinSize.is_null() {
        *pMinSize = (*this).m_idealSize;
    }
    if !pMaxSize.is_null() {
        *pMaxSize = (*this).m_idealSize;
    }
    if !pIntegralSize.is_null() {
        *pIntegralSize = (*this).m_itemSize;
    }

    if (*this).m_hasSizes != FALSE {
        return S_OK;
    }

    TRACE!("Sizes out of date, recalculating.\n");

    if (*this).m_hWnd.is_null() {
        return S_OK;
    }

    tb_get_max_size((*this).m_hWnd, &mut (*this).m_idealSize);
    tb_get_ideal_size((*this).m_hWnd, ((*this).m_initFlags & SMINIT_VERTICAL != 0) as BOOL, &mut (*this).m_idealSize);

    let size = tb_get_button_size((*this).m_hWnd);
    (*this).m_itemSize.cx = GET_X_LPARAM(size as LPARAM);
    (*this).m_itemSize.cy = GET_Y_LPARAM(size as LPARAM);
    (*this).m_hasSizes = TRUE;

    if !pMinSize.is_null() {
        *pMinSize = (*this).m_idealSize;
    }
    if !pMaxSize.is_null() {
        *pMaxSize = (*this).m_idealSize;
    }
    if !pIntegralSize.is_null() {
        *pIntegralSize = (*this).m_itemSize;
    }

    S_OK
}

unsafe fn CMenuToolbarBase_SetPosSize(this: *mut CMenuToolbarBase, x: INT, y: INT, cx: INT, cy: INT) -> HRESULT {
    if !(*this).m_pager_hwnd.is_null() {
        SetWindowPos((*this).m_hWnd, core::ptr::null_mut(), x, y, cx, (*this).m_idealSize.cy, 0);
        SetWindowPos((*this).m_pager_hwnd, core::ptr::null_mut(), x, y, cx, cy, 0);
    } else {
        SetWindowPos((*this).m_hWnd, core::ptr::null_mut(), x, y, cx, cy, 0);
    }

    if (*this).m_initFlags & SMINIT_VERTICAL != 0 {
        let btnSize = tb_get_button_size((*this).m_hWnd);
        tb_set_button_size((*this).m_hWnd, cx, GET_Y_LPARAM(btnSize as LPARAM));
    }

    S_OK
}

unsafe fn CMenuToolbarBase_OnGetInfoTip(this: *mut CMenuToolbarBase, tip: *mut NMTBGETINFOTIPW) -> HRESULT {
    let mut index: INT = 0;
    let mut dwData: DWORD_PTR = 0;
    let iItem = (*tip).iItem;
    CMenuToolbarBase_GetDataFromId(this, iItem, &mut index, &mut dwData);
    dispatch_internal_get_tooltip(this, iItem, index, dwData, (*tip).pszText, (*tip).cchTextMax)
}

unsafe fn CMenuToolbarBase_OnPopupTimer(this: *mut CMenuToolbarBase, wParam: WPARAM, bHandled: *mut BOOL) -> LRESULT {
    if wParam != TIMERID_HOTTRACK {
        *bHandled = FALSE;
        return 0;
    }

    KillTimer((*this).m_hWnd, TIMERID_HOTTRACK);

    if (*this).m_timerEnabled == FALSE {
        return 0;
    }
    (*this).m_timerEnabled = FALSE;

    if (*this).m_hotItem < 0 {
        return 0;
    }

    let hr = CMenuToolbarBase_PopupItem(this, (*this).m_hotItem, FALSE);
    if hr != S_FALSE {
        return 0;
    }

    if !(*this).m_popupBar.is_null() {
        let hr2 = CMenuToolbarBase_CancelCurrentPopup(this);
        if FAILED_UNEXPECTEDLY!(hr2) {
            return 0;
        }
    }

    0
}

unsafe fn CMenuToolbarBase_KillPopupTimer(this: *mut CMenuToolbarBase) -> HRESULT {
    if (*this).m_timerEnabled != FALSE {
        (*this).m_timerEnabled = FALSE;
        KillTimer((*this).m_hWnd, TIMERID_HOTTRACK);
        return S_OK;
    }
    S_FALSE
}

unsafe fn CMenuToolbarBase_ChangeHotItem(this: *mut CMenuToolbarBase, toolbar: *mut CMenuToolbarBase, item: INT, dwFlags: DWORD) -> HRESULT {
    if (*this).m_hotBar == toolbar && (*this).m_hotItem == item {
        return S_FALSE;
    }

    if (*this).m_disableMouseTrack != FALSE && dwFlags & HICF_MOUSE != 0 {
        TRACE!("Hot item change prevented by DisableMouseTrack\n");
        return S_OK;
    }

    if (*this).m_hotBar == this && toolbar != this {
        tb_set_hot_item((*this).m_hWnd, -1);
    }

    TRACE!("Hot item changed from %p %p, to %p %p\n", (*this).m_hotBar, (*this).m_hotItem, toolbar, item);
    (*this).m_hotBar = toolbar;
    (*this).m_hotItem = item;

    if (*this).m_hotBar == this {
        if (*this).m_isTrackingPopup != FALSE && (*this).m_initFlags & SMINIT_VERTICAL == 0 {
            CMenuToolbarBase_PopupItem(this, (*this).m_hotItem, FALSE);
        } else if dwFlags & HICF_MOUSE != 0 {
            if (*this).m_initFlags & SMINIT_VERTICAL != 0 {
                let mut elapsed: DWORD = 0;
                SystemParametersInfoW(SPI_GETMENUSHOWDELAY, 0, core::ptr::addr_of_mut!(elapsed) as PVOID, 0);
                SetTimer((*this).m_hWnd, TIMERID_HOTTRACK, elapsed, None);
                (*this).m_timerEnabled = TRUE;
                TRACE!("SetTimer called with m_hotItem=%d\n", (*this).m_hotItem);
            }
        } else {
            let mut info: TBBUTTONINFOW = core::mem::zeroed();
            info.cbSize = core::mem::size_of::<TBBUTTONINFOW>() as UINT;
            info.dwMask = 0;
            let index = tb_get_button_info((*this).m_hWnd, item, &mut info);
            tb_set_hot_item((*this).m_hWnd, index as INT);
        }
    }

    CMenuToolbarBase_Invalidate_Draw(this);
    S_OK
}

unsafe fn CMenuToolbarBase_ChangePopupItem(this: *mut CMenuToolbarBase, toolbar: *mut CMenuToolbarBase, item: INT) -> HRESULT {
    if (*this).m_popupBar == toolbar && (*this).m_popupItem == item {
        return S_FALSE;
    }

    if (*this).m_popupBar == this && toolbar != this {
        tb_check_button((*this).m_hWnd, (*this).m_popupItem, FALSE);
        (*this).m_isTrackingPopup = FALSE;
    }

    (*this).m_popupBar = toolbar;
    (*this).m_popupItem = item;

    if (*this).m_popupBar == this {
        tb_check_button((*this).m_hWnd, (*this).m_popupItem, TRUE);
    }

    CMenuToolbarBase_Invalidate_Draw(this);
    S_OK
}

unsafe fn CMenuToolbarBase_IsTrackedItem(this: *mut CMenuToolbarBase, wParam: WPARAM, _lParam: LPARAM, bHandled: *mut BOOL) -> LRESULT {
    let idx = wParam as INT;

    if (*this).m_hotBar != this {
        return S_FALSE as LRESULT;
    }
    if idx < 0 {
        return S_FALSE as LRESULT;
    }

    let mut btn: TBBUTTON = core::mem::zeroed();
    if tb_get_button((*this).m_hWnd, idx, &mut btn) == 0 {
        return E_FAIL as LRESULT;
    }

    let _ = bHandled; // ATL's map entry never sets bHandled=FALSE here; TRUE (the default) stands.
    if (*this).m_hotItem == btn.idCommand {
        return S_OK as LRESULT;
    }
    if (*this).m_popupItem == btn.idCommand {
        return S_OK as LRESULT;
    }
    S_FALSE as LRESULT
}

unsafe fn CMenuToolbarBase_ChangeTrackedItem(this: *mut CMenuToolbarBase, wParam: WPARAM, lParam: LPARAM, _bHandled: *mut BOOL) -> LRESULT {
    let wasTracking = (lParam & 0xffff) as BOOL;
    let mouse = ((lParam >> 16) & 0xffff) as BOOL;
    let idx = wParam as INT;

    if idx < 0 {
        (*this).m_isTrackingPopup = FALSE;
        return CMenuBand_ChangeHotItem_RustExport((*this).m_menuBand, core::ptr::null_mut(), -1, HICF_MOUSE) as LRESULT;
    }

    let mut btn: TBBUTTON = core::mem::zeroed();
    if tb_get_button((*this).m_hWnd, idx, &mut btn) == 0 {
        return E_FAIL as LRESULT;
    }

    TRACE!("ChangeTrackedItem %d, %d\n", idx, wasTracking);
    (*this).m_isTrackingPopup = wasTracking;
    CMenuBand_ChangeHotItem_RustExport((*this).m_menuBand, this, btn.idCommand, if mouse != FALSE { HICF_MOUSE } else { 0 }) as LRESULT
}

/// `CMenuToolbarBase::PopupSubMenu(itemId, index, childShellMenu, keyInitiated)` overload.
unsafe fn CMenuToolbarBase_PopupSubMenu_ShellMenu(this: *mut CMenuToolbarBase, iItem: UINT, index: UINT, childShellMenu: *mut IShellMenu, keyInitiated: BOOL) -> HRESULT {
    let mut rc: RECT = core::mem::zeroed();
    if tb_get_item_rect((*this).m_hWnd, index as INT, &mut rc) == 0 {
        return E_FAIL;
    }

    let mut a = POINT { x: rc.left, y: rc.top };
    let mut b = POINT { x: rc.right, y: rc.bottom };
    ClientToScreen((*this).m_hWnd, &mut a);
    ClientToScreen((*this).m_hWnd, &mut b);

    let mut pt = POINTL { x: a.x, y: b.y };
    let rcl = RECTL { left: a.x, top: a.y, right: b.x, bottom: b.y };

    if (*this).m_initFlags & SMINIT_VERTICAL != 0 {
        // FIXME (preserved from the C++): hardcoding this here feels hacky.
        if IsAppThemed() != FALSE {
            pt.x = b.x - 1;
            pt.y = a.y - 1;
        } else {
            pt.x = b.x - 3;
            pt.y = a.y - 3;
        }
    }

    (*this).m_isTrackingPopup = TRUE;

    CMenuBand_ChangePopupItem_RustExport((*this).m_menuBand, this, iItem as INT);
    let mut rcl_mut = rcl;
    CMenuBand_OnPopupSubMenu_RustExport((*this).m_menuBand, childShellMenu, &mut pt, &mut rcl_mut, keyInitiated);

    S_OK
}

/// `CMenuToolbarBase::PopupSubMenu(itemId, index, menu)` overload — an HMENU-backed submenu.
unsafe fn CMenuToolbarBase_PopupSubMenu_Menu(this: *mut CMenuToolbarBase, iItem: UINT, index: UINT, menu: HMENU) -> HRESULT {
    let mut rc: RECT = core::mem::zeroed();
    if tb_get_item_rect((*this).m_hWnd, index as INT, &mut rc) == 0 {
        return E_FAIL;
    }

    let mut a = POINT { x: rc.left, y: rc.top };
    let mut b = POINT { x: rc.right, y: rc.bottom };
    ClientToScreen((*this).m_hWnd, &mut a);
    ClientToScreen((*this).m_hWnd, &mut b);

    let mut pt = POINT { x: a.x, y: b.y };
    let mut rcl = RECT { left: a.x, top: a.y, right: b.x, bottom: b.y };

    if (*this).m_initFlags & SMINIT_VERTICAL != 0 {
        pt.x = b.x;
        pt.y = a.y;
    }

    let popup = GetSubMenu(menu, index as INT);

    (*this).m_isTrackingPopup = TRUE;
    CMenuBand_ChangePopupItem_RustExport((*this).m_menuBand, this, iItem as INT);
    CMenuBand_TrackSubMenu_RustExport((*this).m_menuBand, popup, pt.x, pt.y, &mut rcl);
    CMenuBand_ChangePopupItem_RustExport((*this).m_menuBand, core::ptr::null_mut(), -1);
    (*this).m_isTrackingPopup = FALSE;

    S_OK
}

unsafe fn CMenuToolbarBase_TrackContextMenu(this: *mut CMenuToolbarBase, contextMenu: *mut IContextMenu, pt: POINT) -> HRESULT {
    CMenuBand_KillPopupTimers_RustExport((*this).m_menuBand);
    if !(*this).m_popupBar.is_null() {
        CMenuBand_CancelCurrentPopup_RustExport((*this).m_menuBand);
    }
    CMenuBand_TrackContextMenu_RustExport((*this).m_menuBand, contextMenu, pt.x, pt.y)
}

unsafe fn CMenuToolbarBase_BeforeCancelPopup(this: *mut CMenuToolbarBase) -> HRESULT {
    (*this).m_cancelingPopup = TRUE;
    TRACE!("BeforeCancelPopup\n");
    S_OK
}

unsafe fn CMenuToolbarBase_ProcessClick(this: *mut CMenuToolbarBase, iItem: INT) -> HRESULT {
    if (*this).m_disableMouseTrack != FALSE {
        TRACE!("Item click prevented by DisableMouseTrack\n");
        return S_OK;
    }

    if (*this).m_initFlags & SMINIT_VERTICAL == 0 && (*this).m_isTrackingPopup != FALSE {
        TRACE!("OnCommand cancelled because it was tracking submenu.\n");
        return S_FALSE;
    }

    if CMenuToolbarBase_PopupItem(this, iItem, FALSE) == S_OK {
        TRACE!("PopupItem returned S_OK\n");
        return S_FALSE;
    }

    TRACE!("Executing...\n");

    CMenuBand_MenuItemSelect_RustExport((*this).m_menuBand, MPOS_EXECUTE)
}

unsafe fn CMenuToolbarBase_ProcessContextMenu(this: *mut CMenuToolbarBase, iItem: INT) -> HRESULT {
    let mut index: INT = 0;
    let mut data: DWORD_PTR = 0;
    CMenuToolbarBase_GetDataFromId(this, iItem, &mut index, &mut data);

    let pos = GetMessagePos();
    let pt = POINT { x: GET_X_LPARAM(pos as LPARAM), y: GET_Y_LPARAM(pos as LPARAM) };

    dispatch_internal_context_menu(this, iItem, index, data, pt)
}

unsafe fn CMenuToolbarBase_MenuBarMouseDown(this: *mut CMenuToolbarBase, iIndex: INT, _isLButton: BOOL) -> HRESULT {
    let mut btn: TBBUTTON = core::mem::zeroed();
    tb_get_button((*this).m_hWnd, iIndex, &mut btn);

    if (btn.fsState as u32) & TBSTATE_ENABLED == 0 {
        return S_OK;
    }

    if (*this).m_initFlags & SMINIT_VERTICAL != 0 || !(*this).m_popupBar.is_null() || (*this).m_cancelingPopup != FALSE {
        (*this).m_cancelingPopup = FALSE;
        return S_OK;
    }

    CMenuToolbarBase_ProcessClick(this, btn.idCommand)
}

unsafe fn CMenuToolbarBase_MenuBarMouseUp(this: *mut CMenuToolbarBase, iIndex: INT, isLButton: BOOL) -> HRESULT {
    (*this).m_cancelingPopup = FALSE;

    if (*this).m_initFlags & SMINIT_VERTICAL == 0 {
        return S_OK;
    }

    let mut btn: TBBUTTON = core::mem::zeroed();
    tb_get_button((*this).m_hWnd, iIndex, &mut btn);

    if (btn.fsState as u32) & TBSTATE_ENABLED == 0 {
        return S_OK;
    }

    if isLButton != FALSE {
        CMenuToolbarBase_ProcessClick(this, btn.idCommand)
    } else {
        CMenuToolbarBase_ProcessContextMenu(this, btn.idCommand)
    }
}

unsafe fn CMenuToolbarBase_PrepareExecuteItem(this: *mut CMenuToolbarBase, iItem: INT) -> HRESULT {
    CMenuBand_KillPopupTimers_RustExport((*this).m_menuBand);
    (*this).m_executeItem = iItem;
    CMenuToolbarBase_GetDataFromId(this, iItem, core::ptr::addr_of_mut!((*this).m_executeIndex), core::ptr::addr_of_mut!((*this).m_executeData))
}

unsafe fn CMenuToolbarBase_ExecuteItem(this: *mut CMenuToolbarBase) -> HRESULT {
    // Faithful to the C++: InternalExecuteItem's `index` parameter is passed `m_executeItem`
    // here too, NOT `m_executeIndex` — a pre-existing bug in CMenuToolbarBase::ExecuteItem(),
    // preserved rather than fixed.
    dispatch_internal_execute_item(this, (*this).m_executeItem, (*this).m_executeItem, (*this).m_executeData)
}

unsafe fn CMenuToolbarBase_KeyboardItemChange(this: *mut CMenuToolbarBase, dwSelectType_in: DWORD) -> HRESULT {
    let prev = (*this).m_hotItem;
    let mut index: INT = -1;
    let mut dwSelectType = dwSelectType_in;

    if dwSelectType != 0xFFFF_FFFF {
        let count = tb_get_button_count((*this).m_hWnd);

        if dwSelectType == VK_HOME {
            index = 0;
            dwSelectType = VK_DOWN;
        } else if dwSelectType == VK_END {
            index = count - 1;
            dwSelectType = VK_UP;
        } else {
            if (*this).m_hotItem >= 0 {
                let mut info: TBBUTTONINFOW = core::mem::zeroed();
                info.cbSize = core::mem::size_of::<TBBUTTONINFOW>() as UINT;
                info.dwMask = 0;
                index = tb_get_button_info((*this).m_hWnd, (*this).m_hotItem, &mut info) as INT;
            }

            if index < 0 {
                if dwSelectType == VK_UP {
                    index = count - 1;
                } else if dwSelectType == VK_DOWN {
                    index = 0;
                }
            } else if dwSelectType == VK_UP {
                index -= 1;
            } else if dwSelectType == VK_DOWN {
                index += 1;
            }
        }

        let mut btn: TBBUTTON = core::mem::zeroed();
        while index >= 0 && index < count {
            let res = tb_get_button((*this).m_hWnd, index, &mut btn);
            if res == 0 {
                return E_FAIL;
            }

            if btn.dwData != 0 {
                if prev != btn.idCommand {
                    TRACE!("Setting Hot item to %d\n", index);
                    if (*this).m_initFlags & SMINIT_VERTICAL == 0 && (*this).m_isTrackingPopup != FALSE {
                        let mut tlw: HWND = core::ptr::null_mut();
                        CMenuBand_GetTopLevelWindow_RustExport((*this).m_menuBand, &mut tlw);
                        SendMessageW(tlw, WM_CANCELMODE, 0, 0);
                        PostMessageW((*this).m_hWnd, WM_USER_CHANGETRACKEDITEM, index as WPARAM, MAKELPARAM!((*this).m_isTrackingPopup, FALSE));
                    } else {
                        CMenuBand_ChangeHotItem_RustExport((*this).m_menuBand, this, btn.idCommand, 0);
                    }
                }
                return S_OK;
            }

            if dwSelectType == VK_UP {
                index -= 1;
            } else if dwSelectType == VK_DOWN {
                index += 1;
            }
        }

        return S_FALSE;
    }

    if prev != -1 {
        TRACE!("Setting Hot item to null\n");
        CMenuBand_ChangeHotItem_RustExport((*this).m_menuBand, core::ptr::null_mut(), -1, 0);
    }

    S_FALSE
}

unsafe fn CMenuToolbarBase_AddButton(this: *mut CMenuToolbarBase, commandId: DWORD, caption: LPCWSTR, hasSubMenu: BOOL, iconId: INT, buttonData: DWORD_PTR, last: BOOL) -> HRESULT {
    let mut tbb: TBBUTTON = core::mem::zeroed();

    tbb.fsState = TBSTATE_ENABLED as BYTE;
    // USE_TBSTYLE_EX_VERTICAL is 0 in the C++, so this branch is always live, matching source.
    if last == FALSE && (*this).m_initFlags & SMINIT_VERTICAL != 0 {
        tbb.fsState |= TBSTATE_WRAP as BYTE;
    }
    tbb.fsStyle = BTNS_CHECKGROUP as BYTE;

    if hasSubMenu != FALSE && (*this).m_initFlags & SMINIT_VERTICAL != 0 {
        tbb.fsStyle |= BTNS_DROPDOWN as BYTE;
    }
    if (*this).m_initFlags & SMINIT_VERTICAL == 0 {
        tbb.fsStyle |= BTNS_AUTOSIZE as BYTE;
    }

    tbb.iString = caption as INT_PTR;
    tbb.idCommand = commandId as INT;
    tbb.iBitmap = iconId;
    tbb.dwData = buttonData;

    (*this).m_hasSizes = FALSE;

    if tb_add_buttons((*this).m_hWnd, 1, &mut tbb) == 0 {
        return HRESULT_FROM_WIN32(GetLastError());
    }
    S_OK
}

unsafe fn CMenuToolbarBase_AddSeparator(this: *mut CMenuToolbarBase, last: BOOL) -> HRESULT {
    let mut tbb: TBBUTTON = core::mem::zeroed();
    tbb.fsState = TBSTATE_ENABLED as BYTE;
    if last == FALSE && (*this).m_initFlags & SMINIT_VERTICAL != 0 {
        tbb.fsState |= TBSTATE_WRAP as BYTE;
    }
    tbb.fsStyle = BTNS_SEP as BYTE;
    tbb.iBitmap = 0;

    (*this).m_hasSizes = FALSE;

    if tb_add_buttons((*this).m_hWnd, 1, &mut tbb) == 0 {
        return HRESULT_FROM_WIN32(GetLastError());
    }
    S_OK
}

unsafe fn CMenuToolbarBase_AddPlaceholder(this: *mut CMenuToolbarBase) -> HRESULT {
    let mut tbb: TBBUTTON = core::mem::zeroed();
    let mut MenuString: [u16; 128] = [0; 128];

    LoadStringW(GetModuleHandleW(SHELL32_DLL_W.as_ptr()), IDS_MENU_EMPTY, MenuString.as_mut_ptr(), MenuString.len() as INT);

    tbb.fsState = 0;
    tbb.fsStyle = 0;
    tbb.iString = MenuString.as_ptr() as INT_PTR;
    tbb.iBitmap = -1;

    (*this).m_hasSizes = FALSE;

    if tb_add_buttons((*this).m_hWnd, 1, &mut tbb) == 0 {
        return HRESULT_FROM_WIN32(GetLastError());
    }
    S_OK
}

unsafe fn CMenuToolbarBase_ClearToolbar(this: *mut CMenuToolbarBase) -> HRESULT {
    while tb_delete_button((*this).m_hWnd, 0) != 0 {}
    (*this).m_hasSizes = FALSE;
    S_OK
}

unsafe fn CMenuToolbarBase_GetDataFromId(this: *mut CMenuToolbarBase, iItem: INT, pIndex: *mut INT, pData: *mut DWORD_PTR) -> HRESULT {
    if !pData.is_null() {
        *pData = 0;
    }
    if !pIndex.is_null() {
        *pIndex = -1;
    }
    if iItem < 0 {
        return S_OK;
    }

    let mut info: TBBUTTONINFOW = core::mem::zeroed();
    info.cbSize = core::mem::size_of::<TBBUTTONINFOW>() as UINT;
    info.dwMask = TBIF_COMMAND | TBIF_LPARAM;

    let index = tb_get_button_info((*this).m_hWnd, iItem, &mut info) as INT;
    if index < 0 {
        return E_FAIL;
    }

    if !pIndex.is_null() {
        *pIndex = index;
    }
    if !pData.is_null() {
        *pData = info.lParam;
    }

    S_OK
}

unsafe fn CMenuToolbarBase_CancelCurrentPopup(this: *mut CMenuToolbarBase) -> HRESULT {
    CMenuBand_CancelCurrentPopup_RustExport((*this).m_menuBand)
}

unsafe fn CMenuToolbarBase_PopupItem(this: *mut CMenuToolbarBase, iItem: INT, keyInitiated: BOOL) -> HRESULT {
    let mut index: INT = 0;
    let mut dwData: DWORD_PTR = 0;

    if iItem < 0 {
        return S_OK;
    }
    if (*this).m_popupBar == this && (*this).m_popupItem == iItem {
        return S_OK;
    }

    CMenuToolbarBase_GetDataFromId(this, iItem, &mut index, &mut dwData);

    let hr = dispatch_internal_has_sub_menu(this, iItem, index, dwData);
    if hr != S_OK {
        return hr;
    }

    if !(*this).m_popupBar.is_null() {
        let hr2 = CMenuToolbarBase_CancelCurrentPopup(this);
        if FAILED_UNEXPECTEDLY!(hr2) {
            return hr2;
        }
    }

    if (*this).m_initFlags & SMINIT_VERTICAL == 0 {
        TRACE!("PopupItem non-vertical %d %d\n", index, iItem);
        CMenuBand_ChangeHotItem_RustExport((*this).m_menuBand, this, iItem, 0);
    }

    dispatch_internal_popup_item(this, iItem, index, dwData, keyInitiated)
}

/// `CWindow::Invalidate(TRUE)` — the *_RustBridge for this one resolves to this (not
/// `InvalidateDraw`); see that bridge's comment for why they differ.
unsafe fn CMenuToolbarBase_Invalidate_Window(this: *mut CMenuToolbarBase) {
    InvalidateRect((*this).m_hWnd, core::ptr::null(), TRUE);
}

/// `CMenuToolbarBase::InvalidateDraw()` == `InvalidateRect(NULL, FALSE)`.
unsafe fn CMenuToolbarBase_Invalidate_Draw(this: *mut CMenuToolbarBase) {
    InvalidateRect((*this).m_hWnd, core::ptr::null(), FALSE);
}

static SHELL32_DLL_W: [u16; 12] = [
    's' as u16, 'h' as u16, 'e' as u16, 'l' as u16, 'l' as u16, '3' as u16, '2' as u16,
    '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0,
];

// ── virtual dispatch (see module header) ─────────────────────────────────────────────────────

unsafe fn dispatch_on_deleting_button(this: *mut CMenuToolbarBase, tb: *const NMTOOLBARW) -> HRESULT {
    match (*this).kind {
        ToolbarKind::Static => CMenuStaticToolbar_OnDeletingButton(this as *mut CMenuStaticToolbar, tb),
        ToolbarKind::SF => CMenuSFToolbar_OnDeletingButton(this as *mut CMenuSFToolbar, tb),
    }
}
unsafe fn dispatch_internal_get_tooltip(this: *mut CMenuToolbarBase, iItem: INT, index: INT, dwData: DWORD_PTR, pszText: LPWSTR, cchTextMax: INT) -> HRESULT {
    match (*this).kind {
        ToolbarKind::Static => CMenuStaticToolbar_InternalGetTooltip(this as *mut CMenuStaticToolbar, iItem, index, dwData, pszText, cchTextMax),
        ToolbarKind::SF => CMenuSFToolbar_InternalGetTooltip(this as *mut CMenuSFToolbar, iItem, index, dwData, pszText, cchTextMax),
    }
}
unsafe fn dispatch_internal_execute_item(this: *mut CMenuToolbarBase, iItem: INT, index: INT, dwData: DWORD_PTR) -> HRESULT {
    match (*this).kind {
        ToolbarKind::Static => CMenuStaticToolbar_InternalExecuteItem(this as *mut CMenuStaticToolbar, iItem, index, dwData),
        ToolbarKind::SF => CMenuSFToolbar_InternalExecuteItem(this as *mut CMenuSFToolbar, iItem, index, dwData),
    }
}
unsafe fn dispatch_internal_popup_item(this: *mut CMenuToolbarBase, iItem: INT, index: INT, dwData: DWORD_PTR, keyInitiated: BOOL) -> HRESULT {
    match (*this).kind {
        ToolbarKind::Static => CMenuStaticToolbar_InternalPopupItem(this as *mut CMenuStaticToolbar, iItem, index, dwData, keyInitiated),
        ToolbarKind::SF => CMenuSFToolbar_InternalPopupItem(this as *mut CMenuSFToolbar, iItem, index, dwData, keyInitiated),
    }
}
unsafe fn dispatch_internal_has_sub_menu(this: *mut CMenuToolbarBase, iItem: INT, index: INT, dwData: DWORD_PTR) -> HRESULT {
    match (*this).kind {
        ToolbarKind::Static => CMenuStaticToolbar_InternalHasSubMenu(this as *mut CMenuStaticToolbar, iItem, index, dwData),
        ToolbarKind::SF => CMenuSFToolbar_InternalHasSubMenu(this as *mut CMenuSFToolbar, iItem, index, dwData),
    }
}
unsafe fn dispatch_internal_context_menu(this: *mut CMenuToolbarBase, iItem: INT, index: INT, dwData: DWORD_PTR, pt: POINT) -> HRESULT {
    match (*this).kind {
        ToolbarKind::Static => CMenuStaticToolbar_InternalContextMenu(this as *mut CMenuStaticToolbar, iItem, index, dwData, pt),
        ToolbarKind::SF => CMenuSFToolbar_InternalContextMenu(this as *mut CMenuSFToolbar, iItem, index, dwData, pt),
    }
}
/// `FillToolbar` is virtual in C++ too — dispatched the same way, called both internally
/// (nowhere in this TU) and via its own `*_RustBridge`.
unsafe fn dispatch_fill_toolbar(this: *mut CMenuToolbarBase, clearFirst: BOOL) -> HRESULT {
    match (*this).kind {
        ToolbarKind::Static => CMenuStaticToolbar_FillToolbar(this as *mut CMenuStaticToolbar, clearFirst),
        ToolbarKind::SF => CMenuSFToolbar_FillToolbar(this as *mut CMenuSFToolbar, clearFirst),
    }
}

// ── CMenuStaticToolbar (CMenuToolbars.h:145-169) ─────────────────────────────────────────────

unsafe fn CMenuStaticToolbar_SetMenu(this: *mut CMenuStaticToolbar, hmenu: HMENU, hwnd: HWND, dwFlags: DWORD) -> HRESULT {
    (*this).m_hmenu = hmenu;
    (*this).m_hwndMenu = hwnd;
    (*this).base.m_dwMenuFlags = dwFlags;

    if IsWindow((*this).base.m_hWnd) != 0 {
        CMenuToolbarBase_ClearToolbar(core::ptr::addr_of_mut!((*this).base));
    }

    S_OK
}

unsafe fn CMenuStaticToolbar_GetMenu(this: *mut CMenuStaticToolbar, phmenu: *mut HMENU, phwnd: *mut HWND, pdwFlags: *mut DWORD) -> HRESULT {
    if !phmenu.is_null() {
        *phmenu = (*this).m_hmenu;
    }
    if !phwnd.is_null() {
        *phwnd = (*this).m_hwndMenu;
    }
    if !pdwFlags.is_null() {
        *pdwFlags = (*this).base.m_dwMenuFlags;
    }
    S_OK
}

unsafe fn CMenuStaticToolbar_FillToolbar(this: *mut CMenuStaticToolbar, clearFirst: BOOL) -> HRESULT {
    let base = core::ptr::addr_of_mut!((*this).base);
    let ic = GetMenuItemCount((*this).m_hmenu);

    if clearFirst != FALSE {
        CMenuToolbarBase_ClearToolbar(base);
    }

    for i in 0..ic {
        let last = (i + 1 == ic) as BOOL;

        let mut info: MENUITEMINFOW = core::mem::zeroed();
        info.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
        info.dwTypeData = core::ptr::null_mut();
        info.fMask = MIIM_FTYPE | MIIM_STRING | MIIM_ID;

        if GetMenuItemInfoW((*this).m_hmenu, i as UINT, TRUE, &mut info) == 0 {
            TRACE!("Error obtaining info for menu item at pos=%d\n", i);
            continue;
        }

        if info.fType & MFT_SEPARATOR != 0 {
            CMenuToolbarBase_AddSeparator(base, last);
        } else if info.fType & MFT_BITMAP == 0 {
            info.cch += 1;
            info.dwTypeData = HeapAlloc(GetProcessHeap(), 0, ((info.cch as usize + 1) * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;

            info.fMask = MIIM_STRING | MIIM_SUBMENU | MIIM_ID;
            GetMenuItemInfoW((*this).m_hmenu, i as UINT, TRUE, &mut info);

            let sminfo: *mut SMINFO = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<SMINFO>() as SIZE_T) as *mut SMINFO;
            if sminfo.is_null() {
                HeapFree(GetProcessHeap(), 0, info.dwTypeData as *mut core::ffi::c_void);
                return E_OUTOFMEMORY;
            }
            (*sminfo).dwMask = SMIM_ICON | SMIM_FLAGS;

            let hr = CMenuBand_CallCBWithItemId_RustExport((*this).base.m_menuBand, info.wID, SMC_GETINFO, 0, sminfo as LPARAM);
            if FAILED_UNEXPECTEDLY!(hr) {
                HeapFree(GetProcessHeap(), 0, sminfo as *mut core::ffi::c_void);
                HeapFree(GetProcessHeap(), 0, info.dwTypeData as *mut core::ffi::c_void);
                return hr;
            }

            CMenuToolbarBase_AddButton(base, info.wID, info.dwTypeData, (!info.hSubMenu.is_null()) as BOOL, (*sminfo).iIcon, sminfo as DWORD_PTR, last);

            HeapFree(GetProcessHeap(), 0, info.dwTypeData as *mut core::ffi::c_void);
        }
    }

    S_OK
}

unsafe fn CMenuStaticToolbar_InternalGetTooltip(_this: *mut CMenuStaticToolbar, _iItem: INT, _index: INT, _dwData: DWORD_PTR, _pszText: LPWSTR, _cchTextMax: INT) -> HRESULT {
    UNIMPLEMENTED!();
    E_NOTIMPL
}

unsafe fn CMenuStaticToolbar_OnDeletingButton(_this: *mut CMenuStaticToolbar, tb: *const NMTOOLBARW) -> HRESULT {
    let sminfo = (*tb).tbButton.dwData as *mut SMINFO;
    if !sminfo.is_null() {
        HeapFree(GetProcessHeap(), 0, sminfo as *mut core::ffi::c_void);
    }
    S_OK
}

unsafe fn CMenuStaticToolbar_InternalContextMenu(this: *mut CMenuStaticToolbar, iItem: INT, _index: INT, _dwData: DWORD_PTR, pt: POINT) -> HRESULT {
    let mut contextMenu: *mut IContextMenu = core::ptr::null_mut();
    let _guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(contextMenu));

    let hr = CMenuBand_CallCBWithItemId_RustExport(
        (*this).base.m_menuBand,
        iItem as UINT,
        SMC_GETOBJECT,
        core::ptr::addr_of!(IID_IContextMenu) as WPARAM,
        core::ptr::addr_of_mut!(contextMenu) as LPARAM,
    );
    if hr != S_OK {
        return hr;
    }

    CMenuToolbarBase_TrackContextMenu(core::ptr::addr_of_mut!((*this).base), contextMenu, pt)
}

unsafe fn CMenuStaticToolbar_InternalExecuteItem(this: *mut CMenuStaticToolbar, iItem: INT, _index: INT, _dwData: DWORD_PTR) -> HRESULT {
    CMenuBand_CallCBWithItemId_RustExport((*this).base.m_menuBand, iItem as UINT, SMC_EXEC, 0, 0)
}

unsafe fn CMenuStaticToolbar_InternalPopupItem(this: *mut CMenuStaticToolbar, iItem: INT, index: INT, dwData: DWORD_PTR, keyInitiated: BOOL) -> HRESULT {
    let nfo = dwData as *mut SMINFO;
    if nfo.is_null() {
        return E_FAIL;
    }

    if (*nfo).dwFlags & SMIF_TRACKPOPUP != 0 {
        CMenuToolbarBase_PopupSubMenu_Menu(core::ptr::addr_of_mut!((*this).base), iItem as UINT, index as UINT, (*this).m_hmenu)
    } else {
        let mut shellMenu: *mut IShellMenu = core::ptr::null_mut();
        let _guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(shellMenu));
        let hr = CMenuBand_CallCBWithItemId_RustExport(
            (*this).base.m_menuBand,
            iItem as UINT,
            SMC_GETOBJECT,
            core::ptr::addr_of!(IID_IShellMenu) as WPARAM,
            core::ptr::addr_of_mut!(shellMenu) as LPARAM,
        );
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }
        CMenuToolbarBase_PopupSubMenu_ShellMenu(core::ptr::addr_of_mut!((*this).base), iItem as UINT, index as UINT, shellMenu, keyInitiated)
    }
}

unsafe fn CMenuStaticToolbar_InternalHasSubMenu(this: *mut CMenuStaticToolbar, _iItem: INT, index: INT, _dwData: DWORD_PTR) -> HRESULT {
    if !GetSubMenu((*this).m_hmenu, index).is_null() {
        S_OK
    } else {
        S_FALSE
    }
}

// ── CMenuSFToolbar (CMenuToolbars.h:171-196) ─────────────────────────────────────────────────

/// `int CALLBACK PidlListSort(item1, item2, lParam)` — DPA_Sort comparator.
unsafe extern "system" fn PidlListSort(item1: PVOID, item2: PVOID, lParam: LPARAM) -> INT {
    let psf = lParam as *mut IShellFolder;
    let pidl1 = item1 as PCUIDLIST_RELATIVE;
    let pidl2 = item2 as PCUIDLIST_RELATIVE;
    let hr = ((*(*psf).lpVtbl).CompareIDs)(psf, 0, pidl1, pidl2);
    if hr < 0 {
        // No way to cancel, so sort to equal.
        return 0;
    }
    (hr as u16) as i16 as INT
}

/// `static BOOL IsPidlPrograms(LPCITEMIDLIST pidlTarget)`.
unsafe fn IsPidlPrograms(pidlTarget: LPCITEMIDLIST) -> BOOL {
    let mut szTarget: [u16; 260] = [0; 260];
    let mut szPath: [u16; 260] = [0; 260];
    if SHGetPathFromIDListW(pidlTarget, szTarget.as_mut_ptr()) == 0 {
        return FALSE;
    }
    SHGetSpecialFolderPathW(core::ptr::null_mut(), szPath.as_mut_ptr(), CSIDL_COMMON_PROGRAMS as INT, FALSE);
    if lstrcmpiW(szTarget.as_ptr(), szPath.as_ptr()) == 0 {
        return TRUE;
    }
    SHGetSpecialFolderPathW(core::ptr::null_mut(), szPath.as_mut_ptr(), CSIDL_PROGRAMS as INT, FALSE);
    (lstrcmpiW(szTarget.as_ptr(), szPath.as_ptr()) == 0) as BOOL
}

static ADVANCED_KEY_W: [u16; 62] = {
    // "Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced\0"
    let s = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced\0";
    let mut out = [0u16; 62];
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        out[i] = bytes[i] as u16;
        i += 1;
    }
    out
};
static STARTMENUADMINTOOLS_W: [u16; 21] = {
    let s = "StartMenuAdminTools\0";
    let mut out = [0u16; 21];
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        out[i] = bytes[i] as u16;
        i += 1;
    }
    out
};

unsafe fn CMenuSFToolbar_FillToolbar(this: *mut CMenuSFToolbar, _clearFirst: BOOL) -> HRESULT {
    let base = core::ptr::addr_of_mut!((*this).base);

    let mut eidl: *mut IEnumIDList = core::ptr::null_mut();
    let _eidl_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(eidl));
    let sf = (*this).m_shellFolder;
    let mut hr = ((*(*sf).lpVtbl).EnumObjects)((*this).m_shellFolder, (*base).m_hWnd.cast(), SHCONTF_FOLDERS | SHCONTF_NONFOLDERS, &mut eidl);
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let dpaSort = DPA_Create(10);

    let mut item: LPITEMIDLIST = core::ptr::null_mut();
    hr = ((*(*eidl).lpVtbl).Next)(eidl, 1, &mut item, core::ptr::null_mut());
    while hr == S_OK {
        if CMenuBand_CallCBWithItemPidl_RustExport((*base).m_menuBand, item, 0x1000_0000, 0, 0) == S_FALSE {
            DPA_AppendPtr(dpaSort, item as PVOID);
        } else {
            CoTaskMemFree(item as PVOID);
        }
        hr = ((*(*eidl).lpVtbl).Next)(eidl, 1, &mut item, core::ptr::null_mut());
    }

    if DPA_GetPtrCount(dpaSort) == 0 {
        DPA_Destroy(dpaSort);
        return CMenuToolbarBase_AddPlaceholder(base);
    }

    TRACE!("FillToolbar added %d items to the DPA\n", DPA_GetPtrCount(dpaSort));

    DPA_Sort(dpaSort, Some(PidlListSort), (*this).m_shellFolder as LPARAM);

    let StartMenuAdminTools = SHRegGetBoolUSValueW(ADVANCED_KEY_W.as_ptr(), STARTMENUADMINTOOLS_W.as_ptr(), FALSE, TRUE);

    let bMustHideAdminTools = IsPidlPrograms((*this).m_idList) != FALSE && StartMenuAdminTools == FALSE;
    TRACE!("StartMenuAdminTools: %d\n", StartMenuAdminTools);
    TRACE!("bMustHideAdminTools: %d\n", bMustHideAdminTools);

    let mut szAdminTools: [u16; 260] = [0; 260];
    if bMustHideAdminTools {
        LoadStringW(GetModuleHandleW(SHELL32_DLL_W.as_ptr()), IDS_ADMINISTRATIVETOOLS, szAdminTools.as_mut_ptr(), szAdminTools.len() as INT);
    }

    let mut i: INT = 0;
    while i < DPA_GetPtrCount(dpaSort) {
        item = DPA_GetPtr(dpaSort, i as INT_PTR) as LPITEMIDLIST;

        let mut sr: STRRET = core::mem::zeroed();
        sr.uType = STRRET_CSTR;
        hr = ((*(*sf).lpVtbl).GetDisplayNameOf)((*this).m_shellFolder, item, SIGDN_NORMALDISPLAY, &mut sr);
        if FAILED_UNEXPECTEDLY!(hr) {
            DPA_Destroy(dpaSort);
            return hr;
        }

        let mut MenuString: LPWSTR = core::ptr::null_mut();
        StrRetToStrW((&raw mut sr).cast(), item, &mut MenuString);

        if bMustHideAdminTools && lstrcmpiW(MenuString, szAdminTools.as_ptr()) == 0 {
            i += 1;
            CoTaskMemFree(MenuString as PVOID);
            continue;
        }

        let index = SHMapPIDLToSystemImageListIndex((*this).m_shellFolder, item, core::ptr::null_mut());
        let isPureFolder = GetFolderItemBaseTypeOf((*this).m_shellFolder, item) == SFGAO_FOLDER as SFGAOF; // Don't expand .cab/.zip

        // Fetch next item already, so we know if the current one is the last (matches the C++'s
        // pre-increment-then-compare, including reusing the incremented `i` as the button id).
        i += 1;

        CMenuToolbarBase_AddButton(base, i as DWORD, MenuString, isPureFolder as BOOL, index, item as DWORD_PTR, (i >= DPA_GetPtrCount(dpaSort)) as BOOL);

        CoTaskMemFree(MenuString as PVOID);
    }

    DPA_Destroy(dpaSort);
    hr
}

unsafe fn CMenuSFToolbar_InternalGetTooltip(_this: *mut CMenuSFToolbar, _iItem: INT, _index: INT, _dwData: DWORD_PTR, _pszText: LPWSTR, _cchTextMax: INT) -> HRESULT {
    UNIMPLEMENTED!();
    E_NOTIMPL
}

unsafe fn CMenuSFToolbar_OnDeletingButton(_this: *mut CMenuSFToolbar, tb: *const NMTOOLBARW) -> HRESULT {
    ILFree((*tb).tbButton.dwData as LPITEMIDLIST);
    S_OK
}

unsafe fn CMenuSFToolbar_SetShellFolder(this: *mut CMenuSFToolbar, psf: *mut IShellFolder, pidlFolder: LPCITEMIDLIST, hKey: HKEY, dwFlags: DWORD) -> HRESULT {
    // CComPtr<IShellFolder> assignment: AddRef the new one, Release whatever the old one was.
    if !psf.is_null() {
        let unk = psf as *mut IUnknown;
        ((*(*unk).lpVtbl).AddRef)(unk);
    }
    if !(*this).m_shellFolder.is_null() {
        let unk = (*this).m_shellFolder as *mut IUnknown;
        ((*(*unk).lpVtbl).Release)(unk);
    }
    (*this).m_shellFolder = psf;

    (*this).m_idList = ILClone(pidlFolder);
    (*this).m_hKey = hKey;
    (*this).base.m_dwMenuFlags = dwFlags;

    if IsWindow((*this).base.m_hWnd) != 0 {
        CMenuToolbarBase_ClearToolbar(core::ptr::addr_of_mut!((*this).base));
    }

    S_OK
}

unsafe fn CMenuSFToolbar_GetShellFolder(this: *mut CMenuSFToolbar, pdwFlags: *mut DWORD, ppidl: *mut LPITEMIDLIST, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let sf = (*this).m_shellFolder;
    let hr = ((*(*sf).lpVtbl).QueryInterface)(sf, riid, ppv);
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    if !pdwFlags.is_null() {
        *pdwFlags = (*this).base.m_dwMenuFlags;
    }

    if !ppidl.is_null() {
        let mut pidl: LPITEMIDLIST = core::ptr::null_mut();

        if !(*this).m_idList.is_null() {
            pidl = ILClone((*this).m_idList);
            if pidl.is_null() {
                ERR!("ILClone failed!\n");
                let unk = *ppv as *mut IUnknown;
                ((*(*unk).lpVtbl).Release)(unk);
                return E_FAIL;
            }
        }

        *ppidl = pidl;
    }

    hr
}

unsafe fn CMenuSFToolbar_InternalContextMenu(this: *mut CMenuSFToolbar, _iItem: INT, _index: INT, dwData: DWORD_PTR, pt: POINT) -> HRESULT {
    let mut contextMenu: *mut IContextMenu = core::ptr::null_mut();
    let _guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(contextMenu));
    let pidl = dwData as LPCITEMIDLIST;

    let sf = (*this).m_shellFolder;
    let mut hr = ((*(*sf).lpVtbl).GetUIObjectOf)(
        (*this).m_shellFolder,
        (*this).base.m_hWnd.cast(),
        1,
        &pidl,
        core::ptr::addr_of!(IID_IContextMenu),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(contextMenu) as *mut *mut core::ffi::c_void,
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    hr = CMenuToolbarBase_TrackContextMenu(core::ptr::addr_of_mut!((*this).base), contextMenu, pt);
    hr
}

unsafe fn CMenuSFToolbar_InternalExecuteItem(this: *mut CMenuSFToolbar, _iItem: INT, _index: INT, data: DWORD_PTR) -> HRESULT {
    CMenuBand_CallCBWithItemPidl_RustExport((*this).base.m_menuBand, data as LPITEMIDLIST, SMC_SFEXEC, 0, 0)
}

unsafe fn CMenuSFToolbar_InternalPopupItem(this: *mut CMenuSFToolbar, iItem: INT, index: INT, dwData: DWORD_PTR, keyInitiated: BOOL) -> HRESULT {
    let pidl = dwData as LPITEMIDLIST;
    if pidl.is_null() {
        return E_FAIL;
    }

    let mut shellMenu: *mut IShellMenu = core::ptr::null_mut();
    let _sm_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(shellMenu));
    let mut hr = RSHELL_CMenuBand_CreateInstance(core::ptr::addr_of!(IID_IShellMenu), core::ptr::addr_of_mut!(shellMenu).cast());
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let mut psmc: *mut IShellMenuCallback = core::ptr::null_mut();
    let mut uId: UINT = 0;
    let mut uIdAncestor: UINT = 0;
    let mut flags: DWORD = 0;
    CMenuBand_GetMenuInfo_RustExport((*this).base.m_menuBand, &mut psmc, &mut uId, &mut uIdAncestor, &mut flags);

    // FIXME (preserved from the C++): not sure what to use as uId/uIdAncestor here.
    hr = ((*(*shellMenu).lpVtbl).Initialize)(shellMenu, psmc, 0, uId, SMINIT_VERTICAL);
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let mut childFolder: *mut IShellFolder = core::ptr::null_mut();
    let _cf_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(childFolder));
    let sf = (*this).m_shellFolder;
    hr = ((*(*sf).lpVtbl).BindToObject)((*this).m_shellFolder, pidl, core::ptr::null_mut(), core::ptr::addr_of!(IID_IShellFolder_local), core::ptr::addr_of_mut!(childFolder) as *mut *mut core::ffi::c_void);
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    hr = ((*(*shellMenu).lpVtbl).SetShellFolder)(shellMenu, childFolder, core::ptr::null_mut(), core::ptr::null_mut(), SMSET_TOP);
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    CMenuToolbarBase_PopupSubMenu_ShellMenu(core::ptr::addr_of_mut!((*this).base), iItem as UINT, index as UINT, shellMenu, keyInitiated)
}

unsafe fn CMenuSFToolbar_InternalHasSubMenu(this: *mut CMenuSFToolbar, _iItem: INT, _index: INT, dwData: DWORD_PTR) -> HRESULT {
    let pidl = dwData as LPCITEMIDLIST;
    let isPureFolder = GetFolderItemBaseTypeOf((*this).m_shellFolder, pidl) == SFGAO_FOLDER as SFGAOF;
    if isPureFolder { S_OK } else { S_FALSE }
}

extern "C" {
    /// `IID_IShellFolder` — needed for the `BindToObject` call in `InternalPopupItem`. Named
    /// `_local` because `crate::menu_band` does not re-export it (only `IID_IShellMenu` and
    /// `IID_IContextMenu` are used across that boundary elsewhere in this file).
    #[link_name = "IID_IShellFolder"]
    static IID_IShellFolder_local: IID;
}

// ── Forward bridges: menu_band.rs -> here (see CMenuBand-bridges.md, shell32-com lane) ──────
// menu_band.rs declares these exact 29 symbols against ITS OWN opaque CMenuToolbarBase /
// CMenuStaticToolbar / CMenuSFToolbar marker types; only the pointer identity crosses the FFI
// boundary; my concrete struct types are unrelated to menu_band.rs's markers at the Rust type
// level, which is fine — a pointer is a pointer at the C ABI (task brief, and this crate's own
// `_opaque: [u8; 0]` convention for the other direction).

#[no_mangle]
pub unsafe extern "system" fn CMenuStaticToolbar_Create_RustBridge(band: *mut CMenuBand) -> *mut CMenuStaticToolbar {
    CMenuStaticToolbar_new(band)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuStaticToolbar_Delete_RustBridge(tb: *mut CMenuStaticToolbar) {
    CMenuStaticToolbar_delete(tb);
}

#[no_mangle]
pub unsafe extern "system" fn CMenuStaticToolbar_SetMenu_RustBridge(tb: *mut CMenuStaticToolbar, hmenu: HMENU, hwnd: HWND, dwFlags: DWORD) -> HRESULT {
    CMenuStaticToolbar_SetMenu(tb, hmenu, hwnd, dwFlags)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuStaticToolbar_GetMenu_RustBridge(tb: *mut CMenuStaticToolbar, phmenu: *mut HMENU, phwnd: *mut HWND, pdwFlags: *mut DWORD) -> HRESULT {
    CMenuStaticToolbar_GetMenu(tb, phmenu, phwnd, pdwFlags)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuSFToolbar_Create_RustBridge(band: *mut CMenuBand) -> *mut CMenuSFToolbar {
    CMenuSFToolbar_new(band)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuSFToolbar_Delete_RustBridge(tb: *mut CMenuSFToolbar) {
    CMenuSFToolbar_delete(tb);
}

#[no_mangle]
pub unsafe extern "system" fn CMenuSFToolbar_SetShellFolder_RustBridge(tb: *mut CMenuSFToolbar, psf: *mut IShellFolder, pidlFolder: LPCITEMIDLIST, hKey: HKEY, dwFlags: DWORD) -> HRESULT {
    CMenuSFToolbar_SetShellFolder(tb, psf, pidlFolder, hKey, dwFlags)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuSFToolbar_GetShellFolder_RustBridge(tb: *mut CMenuSFToolbar, pdwFlags: *mut DWORD, ppidl: *mut LPITEMIDLIST, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    CMenuSFToolbar_GetShellFolder(tb, pdwFlags, ppidl, riid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_IsWindowOwner_RustBridge(tb: *mut CMenuToolbarBase, hwnd: HWND) -> HRESULT {
    CMenuToolbarBase_IsWindowOwner(tb, hwnd)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_CreateToolbar_RustBridge(tb: *mut CMenuToolbarBase, hwndParent: HWND, dwFlags: DWORD) -> HRESULT {
    CMenuToolbarBase_CreateToolbar(tb, hwndParent, dwFlags)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_GetWindow_RustBridge(tb: *mut CMenuToolbarBase, phwnd: *mut HWND) -> HRESULT {
    CMenuToolbarBase_GetWindow(tb, phwnd)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_ShowDW_RustBridge(tb: *mut CMenuToolbarBase, fShow: BOOL) -> HRESULT {
    CMenuToolbarBase_ShowDW(tb, fShow)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_Close_RustBridge(tb: *mut CMenuToolbarBase) -> HRESULT {
    CMenuToolbarBase_Close(tb)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_OnWinEvent_RustBridge(tb: *mut CMenuToolbarBase, hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, theResult: *mut LRESULT) -> HRESULT {
    CMenuToolbarBase_OnWinEvent(tb, hWnd, uMsg, wParam, lParam, theResult)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_ChangeHotItem_RustBridge(tb: *mut CMenuToolbarBase, toolbar: *mut CMenuToolbarBase, item: INT, dwFlags: DWORD) -> HRESULT {
    CMenuToolbarBase_ChangeHotItem(tb, toolbar, item, dwFlags)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_ChangePopupItem_RustBridge(tb: *mut CMenuToolbarBase, toolbar: *mut CMenuToolbarBase, item: INT) -> HRESULT {
    CMenuToolbarBase_ChangePopupItem(tb, toolbar, item)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_KeyboardItemChange_RustBridge(tb: *mut CMenuToolbarBase, changeType: DWORD) -> HRESULT {
    CMenuToolbarBase_KeyboardItemChange(tb, changeType)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_PrepareExecuteItem_RustBridge(tb: *mut CMenuToolbarBase, iItem: INT) -> HRESULT {
    CMenuToolbarBase_PrepareExecuteItem(tb, iItem)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_ExecuteItem_RustBridge(tb: *mut CMenuToolbarBase) -> HRESULT {
    CMenuToolbarBase_ExecuteItem(tb)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_GetSizes_RustBridge(tb: *mut CMenuToolbarBase, pMinSize: *mut SIZE, pMaxSize: *mut SIZE, pIntegralSize: *mut SIZE) -> HRESULT {
    CMenuToolbarBase_GetSizes(tb, pMinSize, pMaxSize, pIntegralSize)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_SetPosSize_RustBridge(tb: *mut CMenuToolbarBase, x: INT, y: INT, cx: INT, cy: INT) -> HRESULT {
    CMenuToolbarBase_SetPosSize(tb, x, y, cx, cy)
}

/// Old CMenuBand.cpp:677/679 `Invalidate()` resolved to `CWindow::Invalidate(TRUE)` (inherited
/// via CWindowImplBaseT), NOT `InvalidateDraw()` — keep that resolution (see
/// CMenuToolbarBase_Invalidate_Window's own comment).
#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_Invalidate_RustBridge(tb: *mut CMenuToolbarBase) {
    CMenuToolbarBase_Invalidate_Window(tb);
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_DisableMouseTrack_RustBridge(tb: *mut CMenuToolbarBase, bDisable: BOOL) -> HRESULT {
    CMenuToolbarBase_DisableMouseTrack(tb, bDisable)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_PopupItem_RustBridge(tb: *mut CMenuToolbarBase, iItem: INT, keyInitiated: BOOL) -> HRESULT {
    CMenuToolbarBase_PopupItem(tb, iItem, keyInitiated)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_KillPopupTimer_RustBridge(tb: *mut CMenuToolbarBase) -> HRESULT {
    CMenuToolbarBase_KillPopupTimer(tb)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_MenuBarMouseDown_RustBridge(tb: *mut CMenuToolbarBase, iIndex: INT, isLButton: BOOL) -> HRESULT {
    CMenuToolbarBase_MenuBarMouseDown(tb, iIndex, isLButton)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_MenuBarMouseUp_RustBridge(tb: *mut CMenuToolbarBase, iIndex: INT, isLButton: BOOL) -> HRESULT {
    CMenuToolbarBase_MenuBarMouseUp(tb, iIndex, isLButton)
}

#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_BeforeCancelPopup_RustBridge(tb: *mut CMenuToolbarBase) -> HRESULT {
    CMenuToolbarBase_BeforeCancelPopup(tb)
}

/// Virtual — dispatched by `kind`, same as C++'s vtable pick between the two overrides.
#[no_mangle]
pub unsafe extern "system" fn CMenuToolbarBase_FillToolbar_RustBridge(tb: *mut CMenuToolbarBase, clearFirst: BOOL) -> HRESULT {
    dispatch_fill_toolbar(tb, clearFirst)
}
