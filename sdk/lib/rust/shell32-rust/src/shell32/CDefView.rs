// rustconv-assembly: class:CDefView
//! `CDefView.cpp` -- the shell's default folder view (IShellView3 + 7 more interfaces), the
//! largest single class in shell32 and the first module to sit on BOTH sides of the substrate
//! boundary. Every worker is translated, present, and compiles clean on i686/amd64/arm64.
//!
//! COHAB-FLIPPED 2026-09-19: `CDefView.cpp` is commented out of
//! `dll/win32/shell32/CMakeLists.txt` and this module is the shipping implementation.
//!
//! Wiring and the flip had to be ONE change, and the earlier banner here was a GUARD saying so.
//! Wiring alone, with the `.cpp` still compiled, is a PARTIAL cohab flip: this module's
//! `#[no_mangle]` symbols collide with the live C++ ones, the linker resolves archive members
//! against object files without raising a duplicate-symbol error, and live entry points reroute
//! into Rust that has never executed. Measured -- wired-only, the guest booted, passed smoke on
//! ALL THREE arches, and then froze in explorer after
//! `[EXPLORER] startup: shell window ready`, so `ProcessStartupItems` never ran and the kmtest
//! sweep launched 0 of 91 tests. Smoke cannot see that: the freeze is after smoke's marker, so
//! `vibeos-cli conformance` (launch count) is the oracle for this class of change, not smoke.
//!
//! What made the flip safe is that the TU is a LEAF. `llvm-nm --undefined-only` over every
//! other object linked into shell32.dll -- shell32.dir plus dialogs/, folders/, shelldesktop,
//! shellmenu and recyclebin -- references NOTHING defined in `CDefView.cpp.obj`: the
//! intersection is 0 symbols. The TU's only outward surface is the spec export
//! `SHCreateShellFolderView` (ordinal 256), which this module publishes;
//! `CDefView_CreateInstance` was C++-mangled with no caller outside the TU. So removing the
//! `.cpp` leaves no unresolved reference and no name for Rust to shadow. Recipe: field manual,
//! "Cohab link plumbing".
//!
//!
//! This module is now WIRED, so `cargo check` and the lint gate cover it on every arch and it
//! can no longer rot the way it once did when it was orphan (it reached 799 errors that way).
//! That is half the point of flipping rather than leaving a finished translation parked.
//!
//! Three things in here are decisions rather than repairs, and each one is the reason a later
//! ATL/COM class will or will not need the same work:
//!
//! 1. **ATL's `CWindowImpl` is `crate::atlwin`, on GWLP_USERDATA** rather than ATL's
//!    per-instance machine-code thunk. The recorded constraint was single ownership of thunk +
//!    registered class, NOT the mechanism, and CDefView registers `SHELLDLL_DefView` in exactly
//!    one place. The mechanism decision, the ATL line-by-line correspondence and the one
//!    behavioural difference it carries are in that module's header; read it before porting
//!    CMenuFocusManager, CMenuToolbars, CChangeNotifyServer or CDirectoryWatcher, each of which
//!    inherits the mechanism and still needs its own ownership check.
//!
//!    `CDefView_WindowProc`'s original `let pThis = hWnd as *mut CDefView` was FAITHFUL, not a
//!    bug: ATL's thunk rewrites the first argument from the HWND to `this`. Under atlwin the
//!    first argument really is the HWND, so the same contract is spelled as the USERDATA lookup.
//!
//! 2. **The 25-entry BEGIN_MSG_MAP is `CDefView_ProcessWindowMessage_body`**, a sequence of
//!    `MESSAGE_HANDLER!` expansions rather than a `match`, because ATL's macro is three-state:
//!    it sets `bHandled = TRUE`, calls the handler, and falls through to the NEXT entry if the
//!    handler cleared it. That is why every handler takes `&mut BOOL`, and why two entries
//!    (WM_DRAWITEM and WM_MEASUREITEM) share one handler that inspects the message and declines.
//!
//! 3. **The substrate boundary**, which prelude.rs states as "the island owns DATA LAYOUTS;
//!    `shellcom` owns VTABLE ABI". Getting this module to compile was almost entirely a matter
//!    of making that true rather than of fixing bodies: VARIANT, SAFEARRAY, POINT, SIZE, RECT,
//!    MSG and POINTL were hand-written copies in types.rs/shellcom.rs, and a second nominal copy
//!    of a layout is a type no cast can bridge -- `as *mut _` across two structs splits one
//!    error into two rather than joining them. Where the two sides meet legitimately (a
//!    `shellcom` interface pointer handed to an island function that takes the com-sys one) the
//!    bridge is a raw-pointer `.cast()`, which is a pointer-to-pointer cast between opaque COM
//!    objects and nothing more.
//!
//! Real bugs fixed in here, recorded because each is a CLASS the next TU will hit:
//! `Draw`'s C++ function-pointer parameter sat in three places as raw C text; the static
//! members `fill_list`/`ListViewCompareItems` were given a receiver they do not take, and
//! fill_list then passed BOTH it and the unpacked one to single-receiver helpers; IID_PPV_ARG,
//! C_ASSERT, MAKEINTRESOURCEA and ListView_GetColumn survived as unexpanded macro calls; an
//! `HMENU` import shadowed the island's type with a weaker HANDLE alias (43 errors on its own);
//! `GetWindowProc()` returned `(*this).WindowProc` although `WindowProc` is a STATIC member
//! function with no `this` to read it out of; three CComPtr null TESTS came back as calls to
//! the ASSIGNMENT helper, one of which would have stored the address of the field INTO the
//! field; a C `break` out of a `switch` case became a `continue` with no loop to leave (it is a
//! labelled block now); and the OnNotify switch matched on EIGHT undefined `NM_*` names, the
//! first of which swallowed every notification and made all seventeen arms behind it dead.

// `m_ListView` is a `CListView`, which Derefs to `ATL_CWindow` for `m_hWnd` and for every
// CWindow method -- so each of the ~50 `(*this).m_ListView.<x>` sites in this file autorefs
// through that Deref and trips `dangerous_implicit_autorefs`. Writing `(&(*this).m_ListView)`
// at every one creates exactly the same reference, just spelled out, so it buys nothing;
// scope the allow to THIS module rather than the crate, where 40 other COM classes have no
// Deref in the way and still get the lint. (usbport/usbuhci/usbohci/usbehci-rust carry the
// same allow crate-wide.)
#![allow(dangerous_implicit_autorefs)]

use crate::prelude::*;
use crate::def_view_support::*;
use crate::rosctrls::*;
use crate::def_view_support::UPDATEFOLDERVIEWFLAGS;
use crate::atl::CSimpleMap;
use crate::atlwin::{AtlCreateWindow, AtlRegisterWndClass, AtlStartWindowProc, AtlWindow, AtlWindowProc, AtlWindowState};

// ReactOS ASSERT, like ATLASSERT, compiles out in a release build; the crate's other
// modules each carry this same no-op copy.
macro_rules! ASSERT { ($($arg:tt)*) => {{}}; }
// The GUID family by EXPLICIT import, so the whole TU agrees on one of them. Two -sys crates
// render `_GUID`: the island's comes from ntdll-sys, and com-sys has its own. They are
// layout-identical and nominally distinct, so whichever a name happens to resolve to through a
// glob decides whether a call typechecks. An explicit import beats a glob and cannot go
// ambiguous, which is the whole reason prelude.rs re-exports one name at a time.
use crate::shellcom::{CLSID, GUID, IID, REFCLSID, REFGUID, REFIID};
// The COM interfaces come from `shellcom`, which owns VTABLE ABI (see prelude.rs's
// BOUNDARY note): the island renders these as bindgen C++ classes with `Option<fn>`
// slots, and a call through an `Option` slot is E0618. Slot order and arity were
// compared against the local copies these replace before deleting them.
use crate::shellcom::{IFolderView, IFolderViewVtbl, IDropTarget, IDropTargetVtbl,
                      IDataObject, IDataObjectVtbl, IStream, IStreamVtbl,
                      IUnknown, IUnknownVtbl, IBindCtx, IBindCtxVtbl,
                      IContextMenu, IEnumIDList, IPersistFolder2,
                      IShellFolder, IShellFolder2, IShellDetails, SHELLDETAILS};
use crate::drv_def_ext::RGB;
use crate::drives_folder::HResultFromWin32;
use crate::drives_folder_tu::UlongToPtr;
use crate::win32::{DeleteObject, SHELL_ErrorBox};
use crate::wine::shellord::{DoDragDrop, RegisterDragDrop, RevokeDragDrop};
use crate::def_view_bckgrnd_menu::CDefViewBckgrndMenu_CreateInstance;
use crate::wine::shlmenu::COLORREF;
use crate::def_view_util::FolderViewModeToShellViewId;
use crate::win32::GetWindowLongPtr;
use crate::default_context_menu_support::HRESULT_CODE;
use crate::shellcom::IErrorLog;
use crate::wine::pidl::ILGetSize;
use crate::wine::pidl::ILIsParent;
use crate::shell_link::LCID;
use crate::cfs_folder::LPCRECT;
use crate::user_notification::MSG;
use crate::default_context_menu_tu::MapFcidmCmdToVerb;
use crate::cfs_folder_tu::SHELL_FS_HideExtension;
use crate::dialogs_general::SHELL_GetIconUnderlineFlags;
use crate::wine::shellord::SHLimitInputEdit;
use crate::wine::shlmenu::SIZE;
use crate::drv_def_ext::ShellObjectCreator;
use crate::def_view_util::ShellViewIdToFolderViewMode;
use crate::wine::shlmenu::Shell_MergeMenus;
use crate::wine::shellstring::StrRetToStrNA;
// TPMPARAMS from the ISLAND, not wine::shlmenu: the call below is the island's
// TrackPopupMenuEx, and a second nominal copy of the layout cannot be passed to it.
use crate::island::TPMPARAMS;
use crate::shellcom::USHORT;
use crate::shellcom::VARIANT;
use crate::wine::pidl::_ILGetFileSize;
// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellView {
    pub lpVtbl: *const IShellViewVtbl,
}
#[repr(C)]
pub struct IShellViewVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellView, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellView) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellView) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IShellView, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IShellView, fEnterMode: BOOL) -> HRESULT,
    pub TranslateAccelerator: unsafe extern "system" fn(This: *mut IShellView, pmsg: *mut MSG) -> HRESULT,
    pub EnableModeless: unsafe extern "system" fn(This: *mut IShellView, fEnable: BOOL) -> HRESULT,
    pub UIActivate: unsafe extern "system" fn(This: *mut IShellView, uState: UINT) -> HRESULT,
    pub Refresh: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub CreateViewWindow: unsafe extern "system" fn(This: *mut IShellView, psvPrevious: *mut IShellView, lpfs: LPCFOLDERSETTINGS, psb: *mut IShellBrowser, prcView: *mut RECT, phWnd: *mut HWND) -> HRESULT,
    pub DestroyViewWindow: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub GetCurrentInfo: unsafe extern "system" fn(This: *mut IShellView, lpfs: LPFOLDERSETTINGS) -> HRESULT,
    pub AddPropertySheetPages: unsafe extern "system" fn(This: *mut IShellView, dwReserved: DWORD, pfn: LPFNSVADDPROPSHEETPAGE, lparam: LPARAM) -> HRESULT,
    pub SaveViewState: unsafe extern "system" fn(This: *mut IShellView) -> HRESULT,
    pub SelectItem: unsafe extern "system" fn(This: *mut IShellView, pidlItem: PCUITEMID_CHILD, uFlags: SVSIF) -> HRESULT,
    pub GetItemObject: unsafe extern "system" fn(This: *mut IShellView, uItem: UINT, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
}
extern "C" {
    pub static IID_IShellView: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellView2 {
    pub lpVtbl: *const IShellView2Vtbl,
}
#[repr(C)]
pub struct IShellView2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellView2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellView2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellView2) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IShellView2, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IShellView2, fEnterMode: BOOL) -> HRESULT,
    pub TranslateAccelerator: unsafe extern "system" fn(This: *mut IShellView2, pmsg: *mut MSG) -> HRESULT,
    pub EnableModeless: unsafe extern "system" fn(This: *mut IShellView2, fEnable: BOOL) -> HRESULT,
    pub UIActivate: unsafe extern "system" fn(This: *mut IShellView2, uState: UINT) -> HRESULT,
    pub Refresh: unsafe extern "system" fn(This: *mut IShellView2) -> HRESULT,
    pub CreateViewWindow: unsafe extern "system" fn(This: *mut IShellView2, psvPrevious: *mut IShellView, lpfs: LPCFOLDERSETTINGS, psb: *mut IShellBrowser, prcView: *mut RECT, phWnd: *mut HWND) -> HRESULT,
    pub DestroyViewWindow: unsafe extern "system" fn(This: *mut IShellView2) -> HRESULT,
    pub GetCurrentInfo: unsafe extern "system" fn(This: *mut IShellView2, lpfs: LPFOLDERSETTINGS) -> HRESULT,
    pub AddPropertySheetPages: unsafe extern "system" fn(This: *mut IShellView2, dwReserved: DWORD, pfn: LPFNSVADDPROPSHEETPAGE, lparam: LPARAM) -> HRESULT,
    pub SaveViewState: unsafe extern "system" fn(This: *mut IShellView2) -> HRESULT,
    pub SelectItem: unsafe extern "system" fn(This: *mut IShellView2, pidlItem: PCUITEMID_CHILD, uFlags: SVSIF) -> HRESULT,
    pub GetItemObject: unsafe extern "system" fn(This: *mut IShellView2, uItem: UINT, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetView: unsafe extern "system" fn(This: *mut IShellView2, view_guid: *mut SHELLVIEWID, view_type: ULONG) -> HRESULT,
    pub CreateViewWindow2: unsafe extern "system" fn(This: *mut IShellView2, view_params: LPSV2CVW2_PARAMS) -> HRESULT,
    pub HandleRename: unsafe extern "system" fn(This: *mut IShellView2, new_pidl: PCUITEMID_CHILD) -> HRESULT,
    pub SelectAndPositionItem: unsafe extern "system" fn(This: *mut IShellView2, item: PCUITEMID_CHILD, flags: UINT, point: *mut POINT) -> HRESULT,
}
extern "C" {
    pub static IID_IShellView2: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellView3 {
    pub lpVtbl: *const IShellView3Vtbl,
}
#[repr(C)]
pub struct IShellView3Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellView3, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellView3) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellView3) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IShellView3, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IShellView3, fEnterMode: BOOL) -> HRESULT,
    pub TranslateAccelerator: unsafe extern "system" fn(This: *mut IShellView3, pmsg: *mut MSG) -> HRESULT,
    pub EnableModeless: unsafe extern "system" fn(This: *mut IShellView3, fEnable: BOOL) -> HRESULT,
    pub UIActivate: unsafe extern "system" fn(This: *mut IShellView3, uState: UINT) -> HRESULT,
    pub Refresh: unsafe extern "system" fn(This: *mut IShellView3) -> HRESULT,
    pub CreateViewWindow: unsafe extern "system" fn(This: *mut IShellView3, psvPrevious: *mut IShellView, lpfs: LPCFOLDERSETTINGS, psb: *mut IShellBrowser, prcView: *mut RECT, phWnd: *mut HWND) -> HRESULT,
    pub DestroyViewWindow: unsafe extern "system" fn(This: *mut IShellView3) -> HRESULT,
    pub GetCurrentInfo: unsafe extern "system" fn(This: *mut IShellView3, lpfs: LPFOLDERSETTINGS) -> HRESULT,
    pub AddPropertySheetPages: unsafe extern "system" fn(This: *mut IShellView3, dwReserved: DWORD, pfn: LPFNSVADDPROPSHEETPAGE, lparam: LPARAM) -> HRESULT,
    pub SaveViewState: unsafe extern "system" fn(This: *mut IShellView3) -> HRESULT,
    pub SelectItem: unsafe extern "system" fn(This: *mut IShellView3, pidlItem: PCUITEMID_CHILD, uFlags: SVSIF) -> HRESULT,
    pub GetItemObject: unsafe extern "system" fn(This: *mut IShellView3, uItem: UINT, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub GetView: unsafe extern "system" fn(This: *mut IShellView3, view_guid: *mut SHELLVIEWID, view_type: ULONG) -> HRESULT,
    pub CreateViewWindow2: unsafe extern "system" fn(This: *mut IShellView3, view_params: LPSV2CVW2_PARAMS) -> HRESULT,
    pub HandleRename: unsafe extern "system" fn(This: *mut IShellView3, new_pidl: PCUITEMID_CHILD) -> HRESULT,
    pub SelectAndPositionItem: unsafe extern "system" fn(This: *mut IShellView3, item: PCUITEMID_CHILD, flags: UINT, point: *mut POINT) -> HRESULT,
    pub CreateViewWindow3: unsafe extern "system" fn(This: *mut IShellView3, owner: *mut IShellBrowser, prev_view: *mut IShellView, view_flags: SV3CVW3_FLAGS, mask: FOLDERFLAGS, flags: FOLDERFLAGS, mode: FOLDERVIEWMODE, view_id: *const SHELLVIEWID, rect: *const RECT, hwnd: *mut HWND) -> HRESULT,
}
extern "C" {
    pub static IID_IShellView3: IID;
}

extern "C" {
    pub static IID_IFolderView: IID;
}

// Deterministic from shlobj.h
#[repr(C)]
pub struct IShellFolderView {
    pub lpVtbl: *const IShellFolderViewVtbl,
}
#[repr(C)]
pub struct IShellFolderViewVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellFolderView, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolderView) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolderView) -> ULONG,
    pub Rearrange: unsafe extern "system" fn(This: *mut IShellFolderView, lParamSort: LPARAM) -> HRESULT,
    pub GetArrangeParam: unsafe extern "system" fn(This: *mut IShellFolderView, plParamSort: *mut LPARAM) -> HRESULT,
    pub ArrangeGrid: unsafe extern "system" fn(This: *mut IShellFolderView) -> HRESULT,
    pub AutoArrange: unsafe extern "system" fn(This: *mut IShellFolderView) -> HRESULT,
    pub GetAutoArrange: unsafe extern "system" fn(This: *mut IShellFolderView) -> HRESULT,
    pub AddObject: unsafe extern "system" fn(This: *mut IShellFolderView, pidl: PITEMID_CHILD, puItem: *mut UINT) -> HRESULT,
    pub GetObject: unsafe extern "system" fn(This: *mut IShellFolderView, ppidl: *mut PITEMID_CHILD, uItem: UINT) -> HRESULT,
    pub RemoveObject: unsafe extern "system" fn(This: *mut IShellFolderView, pidl: PITEMID_CHILD, puItem: *mut UINT) -> HRESULT,
    pub GetObjectCount: unsafe extern "system" fn(This: *mut IShellFolderView, puCount: *mut UINT) -> HRESULT,
    pub SetObjectCount: unsafe extern "system" fn(This: *mut IShellFolderView, uCount: UINT, dwFlags: UINT) -> HRESULT,
    pub UpdateObject: unsafe extern "system" fn(This: *mut IShellFolderView, pidlOld: PITEMID_CHILD, pidlNew: PITEMID_CHILD, puItem: *mut UINT) -> HRESULT,
    pub RefreshObject: unsafe extern "system" fn(This: *mut IShellFolderView, pidl: PITEMID_CHILD, puItem: *mut UINT) -> HRESULT,
    pub SetRedraw: unsafe extern "system" fn(This: *mut IShellFolderView, bRedraw: BOOL) -> HRESULT,
    pub GetSelectedCount: unsafe extern "system" fn(This: *mut IShellFolderView, puSelected: *mut UINT) -> HRESULT,
    pub GetSelectedObjects: unsafe extern "system" fn(This: *mut IShellFolderView, pppidl: *mut *mut PCUITEMID_CHILD, puItems: *mut UINT) -> HRESULT,
    pub IsDropOnSource: unsafe extern "system" fn(This: *mut IShellFolderView, pDropTarget: *mut IDropTarget) -> HRESULT,
    pub GetDragPoint: unsafe extern "system" fn(This: *mut IShellFolderView, ppt: *mut POINT) -> HRESULT,
    pub GetDropPoint: unsafe extern "system" fn(This: *mut IShellFolderView, ppt: *mut POINT) -> HRESULT,
    pub MoveIcons: unsafe extern "system" fn(This: *mut IShellFolderView, pDataObject: *mut IDataObject) -> HRESULT,
    pub SetItemPos: unsafe extern "system" fn(This: *mut IShellFolderView, pidl: PCUITEMID_CHILD, ppt: *mut POINT) -> HRESULT,
    pub IsBkDropTarget: unsafe extern "system" fn(This: *mut IShellFolderView, pDropTarget: *mut IDropTarget) -> HRESULT,
    pub SetClipboard: unsafe extern "system" fn(This: *mut IShellFolderView, bMove: BOOL) -> HRESULT,
    pub SetPoints: unsafe extern "system" fn(This: *mut IShellFolderView, pDataObject: *mut IDataObject) -> HRESULT,
    pub GetItemSpacing: unsafe extern "system" fn(This: *mut IShellFolderView, pSpacing: *mut ITEMSPACING) -> HRESULT,
    pub SetCallback: unsafe extern "system" fn(This: *mut IShellFolderView, pNewCB: *mut IShellFolderViewCB, ppOldCB: *mut *mut IShellFolderViewCB) -> HRESULT,
    pub Select: unsafe extern "system" fn(This: *mut IShellFolderView, dwFlags: UINT) -> HRESULT,
    pub QuerySupport: unsafe extern "system" fn(This: *mut IShellFolderView, pdwSupport: *mut UINT) -> HRESULT,
    pub SetAutomationObject: unsafe extern "system" fn(This: *mut IShellFolderView, pdisp: *mut IDispatch) -> HRESULT,
}
extern "C" {
    pub static IID_IShellFolderView: IID;
}

// Deterministic from docobj.h
#[repr(C)]
pub struct IOleCommandTarget {
    pub lpVtbl: *const IOleCommandTargetVtbl,
}
#[repr(C)]
pub struct IOleCommandTargetVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IOleCommandTarget, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IOleCommandTarget) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IOleCommandTarget) -> ULONG,
    pub QueryStatus: unsafe extern "system" fn(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, cCmds: ULONG, prgCmds: *mut OLECMD, pCmdText: *mut OLECMDTEXT) -> HRESULT,
    pub Exec: unsafe extern "system" fn(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, nCmdID: DWORD, nCmdexecopt: DWORD, pvaIn: *mut VARIANT, pvaOut: *mut VARIANT) -> HRESULT,
}
extern "C" {
    pub static IID_IOleCommandTarget: IID;
}

extern "C" {
    pub static IID_IDropTarget: IID;
}

// Deterministic from oleidl.h
#[repr(C)]
pub struct IDropSource {
    pub lpVtbl: *const IDropSourceVtbl,
}
#[repr(C)]
pub struct IDropSourceVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDropSource, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDropSource) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDropSource) -> ULONG,
    pub QueryContinueDrag: unsafe extern "system" fn(This: *mut IDropSource, fEscapePressed: BOOL, grfKeyState: DWORD) -> HRESULT,
    pub GiveFeedback: unsafe extern "system" fn(This: *mut IDropSource, dwEffect: DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IDropSource: IID;
}

// Deterministic from oleidl.h
/// `BOOL (STDMETHODCALLTYPE *pfnContinue)(ULONG_PTR)` — IViewObject::Draw's continuation
/// callback. A C++ function-POINTER parameter, which the TU translation left as raw C text in
/// all three places it appears (vtable slot, thunk, body); this is its Rust spelling. `Option`
/// because the COM contract lets a caller pass NULL to mean "never abort".
pub type LPFNDRAWCONTINUE = Option<unsafe extern "system" fn(dwContinue: ULONG_PTR) -> BOOL>;

#[repr(C)]
pub struct IViewObject {
    pub lpVtbl: *const IViewObjectVtbl,
}
#[repr(C)]
pub struct IViewObjectVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IViewObject, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IViewObject) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IViewObject) -> ULONG,
    pub Draw: unsafe extern "system" fn(This: *mut IViewObject, dwDrawAspect: DWORD, lindex: LONG, pvAspect: *mut core::ffi::c_void, ptd: *mut DVTARGETDEVICE, hdcTargetDev: HDC, hdcDraw: HDC, lprcBounds: LPCRECTL, lprcWBounds: LPCRECTL, pfnContinue: LPFNDRAWCONTINUE, dwContinue: ULONG_PTR) -> HRESULT,
    pub GetColorSet: unsafe extern "system" fn(This: *mut IViewObject, dwDrawAspect: DWORD, lindex: LONG, pvAspect: *mut core::ffi::c_void, ptd: *mut DVTARGETDEVICE, hicTargetDev: HDC, ppColorSet: *mut *mut LOGPALETTE) -> HRESULT,
    pub Freeze: unsafe extern "system" fn(This: *mut IViewObject, dwDrawAspect: DWORD, lindex: LONG, pvAspect: *mut core::ffi::c_void, pdwFreeze: *mut DWORD) -> HRESULT,
    pub Unfreeze: unsafe extern "system" fn(This: *mut IViewObject, dwFreeze: DWORD) -> HRESULT,
    pub SetAdvise: unsafe extern "system" fn(This: *mut IViewObject, aspects: DWORD, advf: DWORD, pAdvSink: *mut IAdviseSink) -> HRESULT,
    pub GetAdvise: unsafe extern "system" fn(This: *mut IViewObject, pAspects: *mut DWORD, pAdvf: *mut DWORD, ppAdvSink: *mut *mut IAdviseSink) -> HRESULT,
}
extern "C" {
    pub static IID_IViewObject: IID;
}

// Deterministic from servprov.h
#[repr(C)]
pub struct IServiceProvider {
    pub lpVtbl: *const IServiceProviderVtbl,
}
#[repr(C)]
pub struct IServiceProviderVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IServiceProvider, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IServiceProvider) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IServiceProvider) -> ULONG,
    pub QueryService: unsafe extern "system" fn(This: *mut IServiceProvider, guidService: REFGUID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
}
extern "C" {
    pub static IID_IServiceProvider: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellBrowser {
    pub lpVtbl: *const IShellBrowserVtbl,
}
#[repr(C)]
pub struct IShellBrowserVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellBrowser, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellBrowser) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellBrowser) -> ULONG,
    pub GetWindow: unsafe extern "system" fn(This: *mut IShellBrowser, phwnd: *mut HWND) -> HRESULT,
    pub ContextSensitiveHelp: unsafe extern "system" fn(This: *mut IShellBrowser, fEnterMode: BOOL) -> HRESULT,
    pub InsertMenusSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU, lpMenuWidths: LPOLEMENUGROUPWIDTHS) -> HRESULT,
    pub SetMenuSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU, holemenuReserved: HOLEMENU, hwndActiveObject: HWND) -> HRESULT,
    pub RemoveMenusSB: unsafe extern "system" fn(This: *mut IShellBrowser, hmenuShared: HMENU) -> HRESULT,
    pub SetStatusTextSB: unsafe extern "system" fn(This: *mut IShellBrowser, pszStatusText: LPCOLESTR) -> HRESULT,
    pub EnableModelessSB: unsafe extern "system" fn(This: *mut IShellBrowser, fEnable: BOOL) -> HRESULT,
    pub TranslateAcceleratorSB: unsafe extern "system" fn(This: *mut IShellBrowser, pmsg: *mut MSG, wID: WORD) -> HRESULT,
    pub BrowseObject: unsafe extern "system" fn(This: *mut IShellBrowser, pidl: PCUIDLIST_RELATIVE, wFlags: UINT) -> HRESULT,
    pub GetViewStateStream: unsafe extern "system" fn(This: *mut IShellBrowser, grfMode: DWORD, ppStrm: *mut *mut IStream) -> HRESULT,
    pub GetControlWindow: unsafe extern "system" fn(This: *mut IShellBrowser, id: UINT, phwnd: *mut HWND) -> HRESULT,
    pub SendControlMsg: unsafe extern "system" fn(This: *mut IShellBrowser, id: UINT, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, pret: *mut LRESULT) -> HRESULT,
    pub QueryActiveShellView: unsafe extern "system" fn(This: *mut IShellBrowser, ppshv: *mut *mut IShellView) -> HRESULT,
    pub OnViewWindowActive: unsafe extern "system" fn(This: *mut IShellBrowser, pshv: *mut IShellView) -> HRESULT,
    pub SetToolbarItems: unsafe extern "system" fn(This: *mut IShellBrowser, lpButtons: LPTBBUTTONSB, nButtons: UINT, uFlags: UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IShellBrowser: IID;
}

extern "C" {
    pub static IID_IDataObject: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct IDispatch {
    pub lpVtbl: *const IDispatchVtbl,
}
#[repr(C)]
pub struct IDispatchVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IDispatch, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IDispatch) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IDispatch) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut IDispatch, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut IDispatch, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut IDispatch, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut IDispatch, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
}
extern "C" {
    pub static IID_IDispatch: IID;
}

// Deterministic from shlobj.h
#[repr(C)]
pub struct IShellFolderViewCB {
    pub lpVtbl: *const IShellFolderViewCBVtbl,
}
#[repr(C)]
pub struct IShellFolderViewCBVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellFolderViewCB, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolderViewCB) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolderViewCB) -> ULONG,
    pub MessageSFVCB: unsafe extern "system" fn(This: *mut IShellFolderViewCB, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT,
}
extern "C" {
    pub static IID_IShellFolderViewCB: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IAdviseSink {
    pub lpVtbl: *const IAdviseSinkVtbl,
}
#[repr(C)]
pub struct IAdviseSinkVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IAdviseSink, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IAdviseSink) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IAdviseSink) -> ULONG,
    pub OnDataChange: unsafe extern "system" fn(This: *mut IAdviseSink, pFormatetc: *mut FORMATETC, pStgmed: *mut STGMEDIUM) -> core::ffi::c_void,
    pub OnViewChange: unsafe extern "system" fn(This: *mut IAdviseSink, dwAspect: DWORD, lindex: LONG) -> core::ffi::c_void,
    pub OnRename: unsafe extern "system" fn(This: *mut IAdviseSink, pmk: *mut IMoniker) -> core::ffi::c_void,
    pub OnSave: unsafe extern "system" fn(This: *mut IAdviseSink) -> core::ffi::c_void,
    pub OnClose: unsafe extern "system" fn(This: *mut IAdviseSink) -> core::ffi::c_void,
}
extern "C" {
    pub static IID_IAdviseSink: IID;
}

extern "C" {
    pub static IID_IStream: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumFORMATETC {
    pub lpVtbl: *const IEnumFORMATETCVtbl,
}
#[repr(C)]
pub struct IEnumFORMATETCVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumFORMATETC, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumFORMATETC) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumFORMATETC) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumFORMATETC, celt: ULONG, rgelt: *mut FORMATETC, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumFORMATETC, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumFORMATETC) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumFORMATETC, ppenum: *mut *mut IEnumFORMATETC) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumFORMATETC: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumSTATDATA {
    pub lpVtbl: *const IEnumSTATDATAVtbl,
}
#[repr(C)]
pub struct IEnumSTATDATAVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumSTATDATA, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumSTATDATA) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumSTATDATA) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumSTATDATA, celt: ULONG, rgelt: *mut STATDATA, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumSTATDATA, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumSTATDATA) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumSTATDATA, ppenum: *mut *mut IEnumSTATDATA) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumSTATDATA: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct ITypeInfo {
    pub lpVtbl: *const ITypeInfoVtbl,
}
#[repr(C)]
pub struct ITypeInfoVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ITypeInfo, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ITypeInfo) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ITypeInfo) -> ULONG,
    pub GetTypeAttr: unsafe extern "system" fn(This: *mut ITypeInfo, ppTypeAttr: *mut *mut TYPEATTR) -> HRESULT,
    pub GetTypeComp: unsafe extern "system" fn(This: *mut ITypeInfo, ppTComp: *mut *mut ITypeComp) -> HRESULT,
    pub GetFuncDesc: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, ppFuncDesc: *mut *mut FUNCDESC) -> HRESULT,
    pub GetVarDesc: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, ppVarDesc: *mut *mut VARDESC) -> HRESULT,
    pub GetNames: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, rgBstrNames: *mut BSTR, cMaxNames: UINT, pcNames: *mut UINT) -> HRESULT,
    pub GetRefTypeOfImplType: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, pRefType: *mut HREFTYPE) -> HRESULT,
    pub GetImplTypeFlags: unsafe extern "system" fn(This: *mut ITypeInfo, index: UINT, pImplTypeFlags: *mut INT) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut ITypeInfo, rgszNames: *mut LPOLESTR, cNames: UINT, pMemId: *mut MEMBERID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut ITypeInfo, pvInstance: PVOID, memid: MEMBERID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub GetDocumentation: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, pBstrName: *mut BSTR, pBstrDocString: *mut BSTR, pdwHelpContext: *mut DWORD, pBstrHelpFile: *mut BSTR) -> HRESULT,
    pub GetDllEntry: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, invKind: INVOKEKIND, pBstrDllName: *mut BSTR, pBstrName: *mut BSTR, pwOrdinal: *mut WORD) -> HRESULT,
    pub GetRefTypeInfo: unsafe extern "system" fn(This: *mut ITypeInfo, hRefType: HREFTYPE, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub AddressOfMember: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, invKind: INVOKEKIND, ppv: *mut PVOID) -> HRESULT,
    pub CreateInstance: unsafe extern "system" fn(This: *mut ITypeInfo, pUnkOuter: *mut IUnknown, riid: REFIID, ppvObj: *mut PVOID) -> HRESULT,
    pub GetMops: unsafe extern "system" fn(This: *mut ITypeInfo, memid: MEMBERID, pBstrMops: *mut BSTR) -> HRESULT,
    pub GetContainingTypeLib: unsafe extern "system" fn(This: *mut ITypeInfo, ppTLib: *mut *mut ITypeLib, pIndex: *mut UINT) -> HRESULT,
    pub ReleaseTypeAttr: unsafe extern "system" fn(This: *mut ITypeInfo, pTypeAttr: *mut TYPEATTR) -> core::ffi::c_void,
    pub ReleaseFuncDesc: unsafe extern "system" fn(This: *mut ITypeInfo, pFuncDesc: *mut FUNCDESC) -> core::ffi::c_void,
    pub ReleaseVarDesc: unsafe extern "system" fn(This: *mut ITypeInfo, pVarDesc: *mut VARDESC) -> core::ffi::c_void,
}
extern "C" {
    pub static IID_ITypeInfo: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IMoniker {
    pub lpVtbl: *const IMonikerVtbl,
}
#[repr(C)]
pub struct IMonikerVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IMoniker, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IMoniker) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IMoniker) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IMoniker, pClassID: *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(This: *mut IMoniker) -> HRESULT,
    pub Load: unsafe extern "system" fn(This: *mut IMoniker, pStm: *mut IStream) -> HRESULT,
    pub Save: unsafe extern "system" fn(This: *mut IMoniker, pStm: *mut IStream, fClearDirty: BOOL) -> HRESULT,
    pub GetSizeMax: unsafe extern "system" fn(This: *mut IMoniker, pcbSize: *mut ULARGE_INTEGER) -> HRESULT,
    pub BindToObject: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, riidResult: REFIID, ppvResult: *mut *mut core::ffi::c_void) -> HRESULT,
    pub BindToStorage: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, riid: REFIID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT,
    pub Reduce: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, dwReduceHowFar: DWORD, ppmkToLeft: *mut *mut IMoniker, ppmkReduced: *mut *mut IMoniker) -> HRESULT,
    pub ComposeWith: unsafe extern "system" fn(This: *mut IMoniker, pmkRight: *mut IMoniker, fOnlyIfNotGeneric: BOOL, ppmkComposite: *mut *mut IMoniker) -> HRESULT,
    pub Enum: unsafe extern "system" fn(This: *mut IMoniker, fForward: BOOL, ppenumMoniker: *mut *mut IEnumMoniker) -> HRESULT,
    pub IsEqual: unsafe extern "system" fn(This: *mut IMoniker, pmkOtherMoniker: *mut IMoniker) -> HRESULT,
    pub Hash: unsafe extern "system" fn(This: *mut IMoniker, pdwHash: *mut DWORD) -> HRESULT,
    pub IsRunning: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pmkNewlyRunning: *mut IMoniker) -> HRESULT,
    pub GetTimeOfLastChange: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pFileTime: *mut FILETIME) -> HRESULT,
    pub Inverse: unsafe extern "system" fn(This: *mut IMoniker, ppmk: *mut *mut IMoniker) -> HRESULT,
    pub CommonPrefixWith: unsafe extern "system" fn(This: *mut IMoniker, pmkOther: *mut IMoniker, ppmkPrefix: *mut *mut IMoniker) -> HRESULT,
    pub RelativePathTo: unsafe extern "system" fn(This: *mut IMoniker, pmkOther: *mut IMoniker, ppmkRelPath: *mut *mut IMoniker) -> HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, ppszDisplayName: *mut LPOLESTR) -> HRESULT,
    pub ParseDisplayName: unsafe extern "system" fn(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, pszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppmkOut: *mut *mut IMoniker) -> HRESULT,
    pub IsSystemMoniker: unsafe extern "system" fn(This: *mut IMoniker, pdwMksys: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IMoniker: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct ITypeComp {
    pub lpVtbl: *const ITypeCompVtbl,
}
#[repr(C)]
pub struct ITypeCompVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ITypeComp, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ITypeComp) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ITypeComp) -> ULONG,
    pub Bind: unsafe extern "system" fn(This: *mut ITypeComp, szName: LPOLESTR, lHashVal: ULONG, wFlags: WORD, ppTInfo: *mut *mut ITypeInfo, pDescKind: *mut DESCKIND, pBindPtr: *mut BINDPTR) -> HRESULT,
    pub BindType: unsafe extern "system" fn(This: *mut ITypeComp, szName: LPOLESTR, lHashVal: ULONG, ppTInfo: *mut *mut ITypeInfo, ppTComp: *mut *mut ITypeComp) -> HRESULT,
}
extern "C" {
    pub static IID_ITypeComp: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct ITypeLib {
    pub lpVtbl: *const ITypeLibVtbl,
}
#[repr(C)]
pub struct ITypeLibVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ITypeLib, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ITypeLib) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ITypeLib) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut ITypeLib) -> UINT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut ITypeLib, index: UINT, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetTypeInfoType: unsafe extern "system" fn(This: *mut ITypeLib, index: UINT, pTKind: *mut TYPEKIND) -> HRESULT,
    pub GetTypeInfoOfGuid: unsafe extern "system" fn(This: *mut ITypeLib, guid: REFGUID, ppTinfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetLibAttr: unsafe extern "system" fn(This: *mut ITypeLib, ppTLibAttr: *mut *mut TLIBATTR) -> HRESULT,
    pub GetTypeComp: unsafe extern "system" fn(This: *mut ITypeLib, ppTComp: *mut *mut ITypeComp) -> HRESULT,
    pub GetDocumentation: unsafe extern "system" fn(This: *mut ITypeLib, index: INT, pBstrName: *mut BSTR, pBstrDocString: *mut BSTR, pdwHelpContext: *mut DWORD, pBstrHelpFile: *mut BSTR) -> HRESULT,
    pub IsName: unsafe extern "system" fn(This: *mut ITypeLib, szNameBuf: LPOLESTR, lHashVal: ULONG, pfName: *mut BOOL) -> HRESULT,
    pub FindName: unsafe extern "system" fn(This: *mut ITypeLib, szNameBuf: LPOLESTR, lHashVal: ULONG, ppTInfo: *mut *mut ITypeInfo, rgMemId: *mut MEMBERID, pcFound: *mut USHORT) -> HRESULT,
    pub ReleaseTLibAttr: unsafe extern "system" fn(This: *mut ITypeLib, pTLibAttr: *mut TLIBATTR) -> core::ffi::c_void,
}
extern "C" {
    pub static IID_ITypeLib: IID;
}

extern "C" {
    pub static IID_IUnknown: IID;
}

extern "C" {
    pub static IID_IBindCtx: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumMoniker {
    pub lpVtbl: *const IEnumMonikerVtbl,
}
#[repr(C)]
pub struct IEnumMonikerVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumMoniker, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumMoniker) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumMoniker) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumMoniker, celt: ULONG, rgelt: *mut *mut IMoniker, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumMoniker, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumMoniker) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumMoniker, ppenum: *mut *mut IEnumMoniker) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumMoniker: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IEnumString {
    pub lpVtbl: *const IEnumStringVtbl,
}
#[repr(C)]
pub struct IEnumStringVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IEnumString, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IEnumString) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IEnumString) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IEnumString, celt: ULONG, rgelt: *mut LPOLESTR, pceltFetched: *mut ULONG) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IEnumString, celt: ULONG) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IEnumString) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IEnumString, ppenum: *mut *mut IEnumString) -> HRESULT,
}
extern "C" {
    pub static IID_IEnumString: IID;
}

// Deterministic from objidl.h
#[repr(C)]
pub struct IRunningObjectTable {
    pub lpVtbl: *const IRunningObjectTableVtbl,
}
#[repr(C)]
pub struct IRunningObjectTableVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IRunningObjectTable, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IRunningObjectTable) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IRunningObjectTable) -> ULONG,
    pub Register: unsafe extern "system" fn(This: *mut IRunningObjectTable, grfFlags: DWORD, punkObject: *mut IUnknown, pmkObjectName: *mut IMoniker, pdwRegister: *mut DWORD) -> HRESULT,
    pub Revoke: unsafe extern "system" fn(This: *mut IRunningObjectTable, dwRegister: DWORD) -> HRESULT,
    pub IsRunning: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker) -> HRESULT,
    pub GetObject: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker, ppunkObject: *mut *mut IUnknown) -> HRESULT,
    pub NoteChangeTime: unsafe extern "system" fn(This: *mut IRunningObjectTable, dwRegister: DWORD, pfiletime: *mut FILETIME) -> HRESULT,
    pub GetTimeOfLastChange: unsafe extern "system" fn(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker, pfiletime: *mut FILETIME) -> HRESULT,
    pub EnumRunning: unsafe extern "system" fn(This: *mut IRunningObjectTable, ppenumMoniker: *mut *mut IEnumMoniker) -> HRESULT,
}
extern "C" {
    pub static IID_IRunningObjectTable: IID;
}

#[repr(C)]
pub struct LISTVIEW_SORT_INFO {
    pub Direction: INT8,
    pub bLoadedFromViewState: bool,
    pub bColumnIsFolderColumn: bool,
    pub Reserved: UINT8,
    pub ListColumn: INT,
    pub FolderColumn: INT,
}

/// CDefView.cpp:70 `enum { UNSPECIFIEDCOLUMN = -1 };` plus the `Reset()` the struct declares
/// inline. A C++ member enum is an associated const, not a field: `cols.UNSPECIFIEDCOLUMN`
/// and `LISTVIEW_SORT_INFO::UNSPECIFIEDCOLUMN` name the same thing there, only the second here.
impl LISTVIEW_SORT_INFO {
    pub const UNSPECIFIEDCOLUMN: INT = -1;

    /// C:71 `*(UINT*)this = 0; ListColumn = UNSPECIFIEDCOLUMN;` -- the cast-write clears
    /// exactly the first UINT, which is the four bytes Direction/bLoadedFromViewState/
    /// bColumnIsFolderColumn/Reserved occupy. It deliberately does NOT touch FolderColumn.
    pub unsafe fn Reset(&mut self) {
        core::ptr::write(self as *mut Self as *mut UINT, 0);
        self.ListColumn = Self::UNSPECIFIEDCOLUMN;
    }
}

// Deterministic COM scaffold for CDefView; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
// SIZE_T is u64 on the LLP64 targets and u32 on i386, so both widths are needed.
impl CppComSize for u64 { fn cpp_com_size(self) -> usize { self as usize } }
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())
}
#[inline]
unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    cpp_com_HeapFree(heap, flags, memory.cast())
}

#[repr(C)]
pub struct CDefView {
    pub IShellView3_iface: IShellView3,
    pub IFolderView_iface: IFolderView,
    pub IShellFolderView_iface: IShellFolderView,
    pub IOleCommandTarget_iface: IOleCommandTarget,
    pub IDropTarget_iface: IDropTarget,
    pub IDropSource_iface: IDropSource,
    pub IViewObject_iface: IViewObject,
    pub IServiceProvider_iface: IServiceProvider,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_hWnd: HWND,
    /// What C++ inherits from `CWindowImplRoot`/`CWindowImplBaseT`. See `crate::atlwin`.
    pub m_atl: AtlWindowState,
    pub m_pSFParent: *mut IShellFolder,
    pub m_pSF2Parent: *mut IShellFolder2,
    pub m_pSDParent: *mut IShellDetails,
    pub m_pShellFolderViewCB: *mut IShellFolderViewCB,
    pub m_pShellBrowser: *mut IShellBrowser,
    pub m_pCommDlgBrowser: *mut ICommDlgBrowser,
    pub m_pFolderFilter: *mut IFolderFilter,
    pub m_pShellFolderViewDual: *mut IShellFolderViewDual,
    pub m_ClipboardChain: ClipboardViewerChain,
    pub m_ListView: CListView,
    pub m_hWndParent: HWND,
    pub m_FolderSettings: FOLDERSETTINGS,
    pub m_hMenu: HMENU,
    pub m_hMenuArrangeModes: HMENU,
    pub m_hMenuViewModes: HMENU,
    pub m_hContextMenu: HMENU,
    pub m_bmenuBarInitialized: BOOL,
    pub m_uState: UINT,
    pub m_cidl: UINT,
    pub m_apidl: *mut PCUITEMID_CHILD,
    pub m_pidlParent: PIDLIST_ABSOLUTE,
    pub m_LoadColumnsList: HDPA,
    pub m_ListToFolderColMap: HDPA,
    pub m_sortInfo: LISTVIEW_SORT_INFO,
    pub m_hNotify: ULONG,
    pub m_hAccel: HACCEL,
    pub m_dwAspects: DWORD,
    pub m_dwAdvf: DWORD,
    pub m_pAdvSink: *mut IAdviseSink,
    pub m_pSourceDataObject: *mut IDataObject,
    pub m_pCurDropTarget: *mut IDropTarget,
    pub m_pCurDataObject: *mut IDataObject,
    pub m_iDragOverItem: LONG,
    pub m_cScrollDelay: UINT,
    pub m_ptLastMousePos: POINT,
    pub m_ptFirstMousePos: POINT,
    pub m_grfKeyState: DWORD,
    pub m_pCM: *mut IContextMenu,
    pub m_pFileMenu: *mut IContextMenu,
    pub m_isEditing: BOOL,
    pub m_SpecialFolder: i8,
    pub m_isFullStatusBar: bool,
    pub m_ScheduledStatusbarUpdate: bool,
    pub m_HasCutItems: bool,
    pub m_Category: CLSID,
    pub m_Destroyed: BOOL,
    pub m_viewinfo_data: SFVM_CUSTOMVIEWINFO_DATA,
    pub m_hMyComputerIcon: HICON,
}

unsafe fn CDefView_cpp_com_assign_m_pSFParent(this: *mut CDefView, value: *mut IShellFolder) {
    let old = (*this).m_pSFParent;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pSFParent = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefView_cpp_com_assign_m_pSF2Parent(this: *mut CDefView, value: *mut IShellFolder2) {
    let old = (*this).m_pSF2Parent;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pSF2Parent = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefView_cpp_com_assign_m_pSDParent(this: *mut CDefView, value: *mut IShellDetails) {
    let old = (*this).m_pSDParent;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pSDParent = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefView_cpp_com_assign_m_pShellFolderViewCB(this: *mut CDefView, value: *mut IShellFolderViewCB) {
    let old = (*this).m_pShellFolderViewCB;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pShellFolderViewCB = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefView_cpp_com_assign_m_pShellBrowser(this: *mut CDefView, value: *mut IShellBrowser) {
    let old = (*this).m_pShellBrowser;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pShellBrowser = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefView_cpp_com_assign_m_pCommDlgBrowser(this: *mut CDefView, value: *mut ICommDlgBrowser) {
    let old = (*this).m_pCommDlgBrowser;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pCommDlgBrowser = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefView_cpp_com_assign_m_pFolderFilter(this: *mut CDefView, value: *mut IFolderFilter) {
    let old = (*this).m_pFolderFilter;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pFolderFilter = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefView_cpp_com_assign_m_pShellFolderViewDual(this: *mut CDefView, value: *mut IShellFolderViewDual) {
    let old = (*this).m_pShellFolderViewDual;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pShellFolderViewDual = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefView_cpp_com_assign_m_pAdvSink(this: *mut CDefView, value: *mut IAdviseSink) {
    let old = (*this).m_pAdvSink;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pAdvSink = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefView_cpp_com_assign_m_pSourceDataObject(this: *mut CDefView, value: *mut IDataObject) {
    let old = (*this).m_pSourceDataObject;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pSourceDataObject = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefView_cpp_com_assign_m_pCurDropTarget(this: *mut CDefView, value: *mut IDropTarget) {
    let old = (*this).m_pCurDropTarget;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pCurDropTarget = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefView_cpp_com_assign_m_pCurDataObject(this: *mut CDefView, value: *mut IDataObject) {
    let old = (*this).m_pCurDataObject;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pCurDataObject = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefView_cpp_com_assign_m_pCM(this: *mut CDefView, value: *mut IContextMenu) {
    let old = (*this).m_pCM;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pCM = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe fn CDefView_cpp_com_assign_m_pFileMenu(this: *mut CDefView, value: *mut IContextMenu) {
    let old = (*this).m_pFileMenu;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_pFileMenu = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CDefView_IShellView3_QueryInterface(This: *mut IShellView3, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDefView_IShellView3_AddRef(This: *mut IShellView3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_AddRef(this)
}
unsafe extern "system" fn CDefView_IShellView3_Release(This: *mut IShellView3) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_Release(this)
}
unsafe extern "system" fn CDefView_IShellView3_GetWindow(This: *mut IShellView3, phwnd: *mut HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_GetWindow_body(this, phwnd)
}
unsafe extern "system" fn CDefView_IShellView3_ContextSensitiveHelp(This: *mut IShellView3, fEnterMode: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_ContextSensitiveHelp_body(this, fEnterMode)
}
unsafe extern "system" fn CDefView_IShellView3_TranslateAccelerator(This: *mut IShellView3, pmsg: *mut MSG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_TranslateAccelerator_body(this, pmsg)
}
unsafe extern "system" fn CDefView_IShellView3_EnableModeless(This: *mut IShellView3, fEnable: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_EnableModeless_body(this, fEnable)
}
unsafe extern "system" fn CDefView_IShellView3_UIActivate(This: *mut IShellView3, uState: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_UIActivate_body(this, uState)
}
unsafe extern "system" fn CDefView_IShellView3_Refresh(This: *mut IShellView3) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_Refresh_body(this)
}
unsafe extern "system" fn CDefView_IShellView3_CreateViewWindow(This: *mut IShellView3, psvPrevious: *mut IShellView, lpfs: LPCFOLDERSETTINGS, psb: *mut IShellBrowser, prcView: *mut RECT, phWnd: *mut HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_CreateViewWindow_body(this, psvPrevious, lpfs, psb, prcView, phWnd)
}
unsafe extern "system" fn CDefView_IShellView3_DestroyViewWindow(This: *mut IShellView3) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_DestroyViewWindow_body(this)
}
unsafe extern "system" fn CDefView_IShellView3_GetCurrentInfo(This: *mut IShellView3, lpfs: LPFOLDERSETTINGS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_GetCurrentInfo_body(this, lpfs)
}
unsafe extern "system" fn CDefView_IShellView3_AddPropertySheetPages(This: *mut IShellView3, dwReserved: DWORD, pfn: LPFNSVADDPROPSHEETPAGE, lparam: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_AddPropertySheetPages_body(this, dwReserved, pfn, lparam)
}
unsafe extern "system" fn CDefView_IShellView3_SaveViewState(This: *mut IShellView3) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_SaveViewState_body(this)
}
unsafe extern "system" fn CDefView_IShellView3_SelectItem(This: *mut IShellView3, pidlItem: PCUITEMID_CHILD, uFlags: SVSIF) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_SelectItem_v1_body(this, pidlItem, uFlags)
}
unsafe extern "system" fn CDefView_IShellView3_GetItemObject(This: *mut IShellView3, uItem: UINT, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_GetItemObject_body(this, uItem, riid, ppv)
}
unsafe extern "system" fn CDefView_IShellView3_GetView(This: *mut IShellView3, view_guid: *mut SHELLVIEWID, view_type: ULONG) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_GetView_body(this, view_guid, view_type)
}
unsafe extern "system" fn CDefView_IShellView3_CreateViewWindow2(This: *mut IShellView3, view_params: LPSV2CVW2_PARAMS) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_CreateViewWindow2_body(this, view_params)
}
unsafe extern "system" fn CDefView_IShellView3_HandleRename(This: *mut IShellView3, new_pidl: PCUITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_HandleRename_body(this, new_pidl)
}
unsafe extern "system" fn CDefView_IShellView3_SelectAndPositionItem(This: *mut IShellView3, item: PCUITEMID_CHILD, flags: UINT, point: *mut POINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_SelectAndPositionItem_body(this, item, flags, point)
}
unsafe extern "system" fn CDefView_IShellView3_CreateViewWindow3(This: *mut IShellView3, owner: *mut IShellBrowser, prev_view: *mut IShellView, view_flags: SV3CVW3_FLAGS, mask: FOLDERFLAGS, flags: FOLDERFLAGS, mode: FOLDERVIEWMODE, view_id: *const SHELLVIEWID, rect: *const RECT, hwnd: *mut HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellView3_iface)) as *mut CDefView;
    CDefView_CreateViewWindow3_body(this, owner, prev_view, view_flags, mask, flags, mode, view_id, rect, hwnd)
}

pub static CDEFVIEW_ISHELLVIEW3_VTBL: IShellView3Vtbl = IShellView3Vtbl {
    QueryInterface: CDefView_IShellView3_QueryInterface,
    AddRef: CDefView_IShellView3_AddRef,
    Release: CDefView_IShellView3_Release,
    GetWindow: CDefView_IShellView3_GetWindow,
    ContextSensitiveHelp: CDefView_IShellView3_ContextSensitiveHelp,
    TranslateAccelerator: CDefView_IShellView3_TranslateAccelerator,
    EnableModeless: CDefView_IShellView3_EnableModeless,
    UIActivate: CDefView_IShellView3_UIActivate,
    Refresh: CDefView_IShellView3_Refresh,
    CreateViewWindow: CDefView_IShellView3_CreateViewWindow,
    DestroyViewWindow: CDefView_IShellView3_DestroyViewWindow,
    GetCurrentInfo: CDefView_IShellView3_GetCurrentInfo,
    AddPropertySheetPages: CDefView_IShellView3_AddPropertySheetPages,
    SaveViewState: CDefView_IShellView3_SaveViewState,
    SelectItem: CDefView_IShellView3_SelectItem,
    GetItemObject: CDefView_IShellView3_GetItemObject,
    GetView: CDefView_IShellView3_GetView,
    CreateViewWindow2: CDefView_IShellView3_CreateViewWindow2,
    HandleRename: CDefView_IShellView3_HandleRename,
    SelectAndPositionItem: CDefView_IShellView3_SelectAndPositionItem,
    CreateViewWindow3: CDefView_IShellView3_CreateViewWindow3,
};

unsafe extern "system" fn CDefView_IFolderView_QueryInterface(This: *mut IFolderView, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDefView_IFolderView_AddRef(This: *mut IFolderView) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_AddRef(this)
}
unsafe extern "system" fn CDefView_IFolderView_Release(This: *mut IFolderView) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_Release(this)
}
unsafe extern "system" fn CDefView_IFolderView_GetCurrentViewMode(This: *mut IFolderView, mode: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_GetCurrentViewMode_body(this, mode)
}
unsafe extern "system" fn CDefView_IFolderView_SetCurrentViewMode(This: *mut IFolderView, mode: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_SetCurrentViewMode_body(this, mode)
}
unsafe extern "system" fn CDefView_IFolderView_GetFolder(This: *mut IFolderView, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_GetFolder_body(this, riid, ppv)
}
unsafe extern "system" fn CDefView_IFolderView_Item(This: *mut IFolderView, index: i32, ppidl: *mut PITEMID_CHILD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_Item_body(this, index, ppidl)
}
unsafe extern "system" fn CDefView_IFolderView_ItemCount(This: *mut IFolderView, flags: UINT, items: *mut i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_ItemCount_body(this, flags, items)
}
unsafe extern "system" fn CDefView_IFolderView_Items(This: *mut IFolderView, flags: UINT, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_Items_body(this, flags, riid, ppv)
}
unsafe extern "system" fn CDefView_IFolderView_GetSelectionMarkedItem(This: *mut IFolderView, item: *mut i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_GetSelectionMarkedItem_body(this, item)
}
unsafe extern "system" fn CDefView_IFolderView_GetFocusedItem(This: *mut IFolderView, item: *mut i32) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_GetFocusedItem_body(this, item)
}
unsafe extern "system" fn CDefView_IFolderView_GetItemPosition(This: *mut IFolderView, pidl: PCUITEMID_CHILD, ppt: *mut POINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_GetItemPosition_body(this, pidl, ppt)
}
unsafe extern "system" fn CDefView_IFolderView_GetSpacing(This: *mut IFolderView, pt: *mut POINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_GetSpacing_body(this, pt)
}
unsafe extern "system" fn CDefView_IFolderView_GetDefaultSpacing(This: *mut IFolderView, pt: *mut POINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_GetDefaultSpacing_body(this, pt)
}
unsafe extern "system" fn CDefView_IFolderView_GetAutoArrange(This: *mut IFolderView) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_GetAutoArrange_body(this)
}
unsafe extern "system" fn CDefView_IFolderView_SelectItem(This: *mut IFolderView, item: i32, flags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_SelectItem_v2_body(this, item, flags)
}
unsafe extern "system" fn CDefView_IFolderView_SelectAndPositionItems(This: *mut IFolderView, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, apt: *mut POINT, flags: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IFolderView_iface)) as *mut CDefView;
    CDefView_SelectAndPositionItems_body(this, cidl, apidl, apt, flags)
}

pub static CDEFVIEW_IFOLDERVIEW_VTBL: IFolderViewVtbl = IFolderViewVtbl {
    QueryInterface: CDefView_IFolderView_QueryInterface,
    AddRef: CDefView_IFolderView_AddRef,
    Release: CDefView_IFolderView_Release,
    GetCurrentViewMode: CDefView_IFolderView_GetCurrentViewMode,
    SetCurrentViewMode: CDefView_IFolderView_SetCurrentViewMode,
    GetFolder: CDefView_IFolderView_GetFolder,
    Item: CDefView_IFolderView_Item,
    ItemCount: CDefView_IFolderView_ItemCount,
    Items: CDefView_IFolderView_Items,
    GetSelectionMarkedItem: CDefView_IFolderView_GetSelectionMarkedItem,
    GetFocusedItem: CDefView_IFolderView_GetFocusedItem,
    GetItemPosition: CDefView_IFolderView_GetItemPosition,
    GetSpacing: CDefView_IFolderView_GetSpacing,
    GetDefaultSpacing: CDefView_IFolderView_GetDefaultSpacing,
    GetAutoArrange: CDefView_IFolderView_GetAutoArrange,
    SelectItem: CDefView_IFolderView_SelectItem,
    SelectAndPositionItems: CDefView_IFolderView_SelectAndPositionItems,
};

unsafe extern "system" fn CDefView_IShellFolderView_QueryInterface(This: *mut IShellFolderView, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_QueryInterface(this, riid, ppv)
}
unsafe extern "system" fn CDefView_IShellFolderView_AddRef(This: *mut IShellFolderView) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_AddRef(this)
}
unsafe extern "system" fn CDefView_IShellFolderView_Release(This: *mut IShellFolderView) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_Release(this)
}
unsafe extern "system" fn CDefView_IShellFolderView_Rearrange(This: *mut IShellFolderView, lParamSort: LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_Rearrange_body(this, lParamSort)
}
unsafe extern "system" fn CDefView_IShellFolderView_GetArrangeParam(This: *mut IShellFolderView, plParamSort: *mut LPARAM) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_GetArrangeParam_body(this, plParamSort)
}
unsafe extern "system" fn CDefView_IShellFolderView_ArrangeGrid(This: *mut IShellFolderView) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_ArrangeGrid_body(this)
}
unsafe extern "system" fn CDefView_IShellFolderView_AutoArrange(This: *mut IShellFolderView) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_AutoArrange_body(this)
}
unsafe extern "system" fn CDefView_IShellFolderView_GetAutoArrange(This: *mut IShellFolderView) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_GetAutoArrange_body(this)
}
unsafe extern "system" fn CDefView_IShellFolderView_AddObject(This: *mut IShellFolderView, pidl: PITEMID_CHILD, puItem: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_AddObject_body(this, pidl, puItem)
}
unsafe extern "system" fn CDefView_IShellFolderView_GetObject(This: *mut IShellFolderView, ppidl: *mut PITEMID_CHILD, uItem: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_GetObject_v1_body(this, ppidl, uItem)
}
unsafe extern "system" fn CDefView_IShellFolderView_RemoveObject(This: *mut IShellFolderView, pidl: PITEMID_CHILD, puItem: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_RemoveObject_body(this, pidl, puItem)
}
unsafe extern "system" fn CDefView_IShellFolderView_GetObjectCount(This: *mut IShellFolderView, puCount: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_GetObjectCount_body(this, puCount)
}
unsafe extern "system" fn CDefView_IShellFolderView_SetObjectCount(This: *mut IShellFolderView, uCount: UINT, dwFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_SetObjectCount_body(this, uCount, dwFlags)
}
unsafe extern "system" fn CDefView_IShellFolderView_UpdateObject(This: *mut IShellFolderView, pidlOld: PITEMID_CHILD, pidlNew: PITEMID_CHILD, puItem: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_UpdateObject_body(this, pidlOld, pidlNew, puItem)
}
unsafe extern "system" fn CDefView_IShellFolderView_RefreshObject(This: *mut IShellFolderView, pidl: PITEMID_CHILD, puItem: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_RefreshObject_body(this, pidl, puItem)
}
unsafe extern "system" fn CDefView_IShellFolderView_SetRedraw(This: *mut IShellFolderView, bRedraw: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_SetRedraw_body(this, bRedraw)
}
unsafe extern "system" fn CDefView_IShellFolderView_GetSelectedCount(This: *mut IShellFolderView, puSelected: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_GetSelectedCount_body(this, puSelected)
}
unsafe extern "system" fn CDefView_IShellFolderView_GetSelectedObjects(This: *mut IShellFolderView, pppidl: *mut *mut PCUITEMID_CHILD, puItems: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_GetSelectedObjects_body(this, pppidl, puItems)
}
unsafe extern "system" fn CDefView_IShellFolderView_IsDropOnSource(This: *mut IShellFolderView, pDropTarget: *mut IDropTarget) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_IsDropOnSource_body(this, pDropTarget)
}
unsafe extern "system" fn CDefView_IShellFolderView_GetDragPoint(This: *mut IShellFolderView, ppt: *mut POINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_GetDragPoint_body(this, ppt)
}
unsafe extern "system" fn CDefView_IShellFolderView_GetDropPoint(This: *mut IShellFolderView, ppt: *mut POINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_GetDropPoint_body(this, ppt)
}
unsafe extern "system" fn CDefView_IShellFolderView_MoveIcons(This: *mut IShellFolderView, pDataObject: *mut IDataObject) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_MoveIcons_body(this, pDataObject)
}
unsafe extern "system" fn CDefView_IShellFolderView_SetItemPos(This: *mut IShellFolderView, pidl: PCUITEMID_CHILD, ppt: *mut POINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_SetItemPos_body(this, pidl, ppt)
}
unsafe extern "system" fn CDefView_IShellFolderView_IsBkDropTarget(This: *mut IShellFolderView, pDropTarget: *mut IDropTarget) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_IsBkDropTarget_body(this, pDropTarget)
}
unsafe extern "system" fn CDefView_IShellFolderView_SetClipboard(This: *mut IShellFolderView, bMove: BOOL) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_SetClipboard_body(this, bMove)
}
unsafe extern "system" fn CDefView_IShellFolderView_SetPoints(This: *mut IShellFolderView, pDataObject: *mut IDataObject) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_SetPoints_body(this, pDataObject)
}
unsafe extern "system" fn CDefView_IShellFolderView_GetItemSpacing(This: *mut IShellFolderView, pSpacing: *mut ITEMSPACING) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_GetItemSpacing_body(this, pSpacing)
}
unsafe extern "system" fn CDefView_IShellFolderView_SetCallback(This: *mut IShellFolderView, pNewCB: *mut IShellFolderViewCB, ppOldCB: *mut *mut IShellFolderViewCB) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_SetCallback_body(this, pNewCB, ppOldCB)
}
unsafe extern "system" fn CDefView_IShellFolderView_Select(This: *mut IShellFolderView, dwFlags: UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_Select_body(this, dwFlags)
}
unsafe extern "system" fn CDefView_IShellFolderView_QuerySupport(This: *mut IShellFolderView, pdwSupport: *mut UINT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_QuerySupport_body(this, pdwSupport)
}
unsafe extern "system" fn CDefView_IShellFolderView_SetAutomationObject(This: *mut IShellFolderView, pdisp: *mut IDispatch) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IShellFolderView_iface)) as *mut CDefView;
    CDefView_SetAutomationObject_body(this, pdisp)
}

pub static CDEFVIEW_ISHELLFOLDERVIEW_VTBL: IShellFolderViewVtbl = IShellFolderViewVtbl {
    QueryInterface: CDefView_IShellFolderView_QueryInterface,
    AddRef: CDefView_IShellFolderView_AddRef,
    Release: CDefView_IShellFolderView_Release,
    Rearrange: CDefView_IShellFolderView_Rearrange,
    GetArrangeParam: CDefView_IShellFolderView_GetArrangeParam,
    ArrangeGrid: CDefView_IShellFolderView_ArrangeGrid,
    AutoArrange: CDefView_IShellFolderView_AutoArrange,
    GetAutoArrange: CDefView_IShellFolderView_GetAutoArrange,
    AddObject: CDefView_IShellFolderView_AddObject,
    GetObject: CDefView_IShellFolderView_GetObject,
    RemoveObject: CDefView_IShellFolderView_RemoveObject,
    GetObjectCount: CDefView_IShellFolderView_GetObjectCount,
    SetObjectCount: CDefView_IShellFolderView_SetObjectCount,
    UpdateObject: CDefView_IShellFolderView_UpdateObject,
    RefreshObject: CDefView_IShellFolderView_RefreshObject,
    SetRedraw: CDefView_IShellFolderView_SetRedraw,
    GetSelectedCount: CDefView_IShellFolderView_GetSelectedCount,
    GetSelectedObjects: CDefView_IShellFolderView_GetSelectedObjects,
    IsDropOnSource: CDefView_IShellFolderView_IsDropOnSource,
    GetDragPoint: CDefView_IShellFolderView_GetDragPoint,
    GetDropPoint: CDefView_IShellFolderView_GetDropPoint,
    MoveIcons: CDefView_IShellFolderView_MoveIcons,
    SetItemPos: CDefView_IShellFolderView_SetItemPos,
    IsBkDropTarget: CDefView_IShellFolderView_IsBkDropTarget,
    SetClipboard: CDefView_IShellFolderView_SetClipboard,
    SetPoints: CDefView_IShellFolderView_SetPoints,
    GetItemSpacing: CDefView_IShellFolderView_GetItemSpacing,
    SetCallback: CDefView_IShellFolderView_SetCallback,
    Select: CDefView_IShellFolderView_Select,
    QuerySupport: CDefView_IShellFolderView_QuerySupport,
    SetAutomationObject: CDefView_IShellFolderView_SetAutomationObject,
};

unsafe extern "system" fn CDefView_IOleCommandTarget_QueryInterface(This: *mut IOleCommandTarget, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IOleCommandTarget_iface)) as *mut CDefView;
    CDefView_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDefView_IOleCommandTarget_AddRef(This: *mut IOleCommandTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IOleCommandTarget_iface)) as *mut CDefView;
    CDefView_AddRef(this)
}
unsafe extern "system" fn CDefView_IOleCommandTarget_Release(This: *mut IOleCommandTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IOleCommandTarget_iface)) as *mut CDefView;
    CDefView_Release(this)
}
unsafe extern "system" fn CDefView_IOleCommandTarget_QueryStatus(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, cCmds: ULONG, prgCmds: *mut OLECMD, pCmdText: *mut OLECMDTEXT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IOleCommandTarget_iface)) as *mut CDefView;
    CDefView_QueryStatus_body(this, pguidCmdGroup, cCmds, prgCmds, pCmdText)
}
unsafe extern "system" fn CDefView_IOleCommandTarget_Exec(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, nCmdID: DWORD, nCmdexecopt: DWORD, pvaIn: *mut VARIANT, pvaOut: *mut VARIANT) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IOleCommandTarget_iface)) as *mut CDefView;
    CDefView_Exec_body(this, pguidCmdGroup, nCmdID, nCmdexecopt, pvaIn, pvaOut)
}

pub static CDEFVIEW_IOLECOMMANDTARGET_VTBL: IOleCommandTargetVtbl = IOleCommandTargetVtbl {
    QueryInterface: CDefView_IOleCommandTarget_QueryInterface,
    AddRef: CDefView_IOleCommandTarget_AddRef,
    Release: CDefView_IOleCommandTarget_Release,
    QueryStatus: CDefView_IOleCommandTarget_QueryStatus,
    Exec: CDefView_IOleCommandTarget_Exec,
};

unsafe extern "system" fn CDefView_IDropTarget_QueryInterface(This: *mut IDropTarget, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IDropTarget_iface)) as *mut CDefView;
    CDefView_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDefView_IDropTarget_AddRef(This: *mut IDropTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IDropTarget_iface)) as *mut CDefView;
    CDefView_AddRef(this)
}
unsafe extern "system" fn CDefView_IDropTarget_Release(This: *mut IDropTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IDropTarget_iface)) as *mut CDefView;
    CDefView_Release(this)
}
unsafe extern "system" fn CDefView_IDropTarget_DragEnter(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IDropTarget_iface)) as *mut CDefView;
    CDefView_DragEnter_body(this, pDataObj, grfKeyState, pt, pdwEffect)
}
unsafe extern "system" fn CDefView_IDropTarget_DragOver(This: *mut IDropTarget, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IDropTarget_iface)) as *mut CDefView;
    CDefView_DragOver_body(this, grfKeyState, pt, pdwEffect)
}
unsafe extern "system" fn CDefView_IDropTarget_DragLeave(This: *mut IDropTarget) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IDropTarget_iface)) as *mut CDefView;
    CDefView_DragLeave_body(this)
}
unsafe extern "system" fn CDefView_IDropTarget_Drop(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IDropTarget_iface)) as *mut CDefView;
    CDefView_Drop_body(this, pDataObj, grfKeyState, pt, pdwEffect)
}

pub static CDEFVIEW_IDROPTARGET_VTBL: IDropTargetVtbl = IDropTargetVtbl {
    QueryInterface: CDefView_IDropTarget_QueryInterface,
    AddRef: CDefView_IDropTarget_AddRef,
    Release: CDefView_IDropTarget_Release,
    DragEnter: CDefView_IDropTarget_DragEnter,
    DragOver: CDefView_IDropTarget_DragOver,
    DragLeave: CDefView_IDropTarget_DragLeave,
    Drop: CDefView_IDropTarget_Drop,
};

unsafe extern "system" fn CDefView_IDropSource_QueryInterface(This: *mut IDropSource, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IDropSource_iface)) as *mut CDefView;
    CDefView_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDefView_IDropSource_AddRef(This: *mut IDropSource) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IDropSource_iface)) as *mut CDefView;
    CDefView_AddRef(this)
}
unsafe extern "system" fn CDefView_IDropSource_Release(This: *mut IDropSource) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IDropSource_iface)) as *mut CDefView;
    CDefView_Release(this)
}
unsafe extern "system" fn CDefView_IDropSource_QueryContinueDrag(This: *mut IDropSource, fEscapePressed: BOOL, grfKeyState: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IDropSource_iface)) as *mut CDefView;
    CDefView_QueryContinueDrag_body(this, fEscapePressed, grfKeyState)
}
unsafe extern "system" fn CDefView_IDropSource_GiveFeedback(This: *mut IDropSource, dwEffect: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IDropSource_iface)) as *mut CDefView;
    CDefView_GiveFeedback_body(this, dwEffect)
}

pub static CDEFVIEW_IDROPSOURCE_VTBL: IDropSourceVtbl = IDropSourceVtbl {
    QueryInterface: CDefView_IDropSource_QueryInterface,
    AddRef: CDefView_IDropSource_AddRef,
    Release: CDefView_IDropSource_Release,
    QueryContinueDrag: CDefView_IDropSource_QueryContinueDrag,
    GiveFeedback: CDefView_IDropSource_GiveFeedback,
};

unsafe extern "system" fn CDefView_IViewObject_QueryInterface(This: *mut IViewObject, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IViewObject_iface)) as *mut CDefView;
    CDefView_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDefView_IViewObject_AddRef(This: *mut IViewObject) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IViewObject_iface)) as *mut CDefView;
    CDefView_AddRef(this)
}
unsafe extern "system" fn CDefView_IViewObject_Release(This: *mut IViewObject) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IViewObject_iface)) as *mut CDefView;
    CDefView_Release(this)
}
unsafe extern "system" fn CDefView_IViewObject_Draw(This: *mut IViewObject, dwDrawAspect: DWORD, lindex: LONG, pvAspect: *mut core::ffi::c_void, ptd: *mut DVTARGETDEVICE, hdcTargetDev: HDC, hdcDraw: HDC, lprcBounds: LPCRECTL, lprcWBounds: LPCRECTL, pfnContinue: LPFNDRAWCONTINUE, dwContinue: ULONG_PTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IViewObject_iface)) as *mut CDefView;
    CDefView_Draw_body(this, dwDrawAspect, lindex, pvAspect, ptd, hdcTargetDev, hdcDraw, lprcBounds, lprcWBounds, pfnContinue, dwContinue)
}
unsafe extern "system" fn CDefView_IViewObject_GetColorSet(This: *mut IViewObject, dwDrawAspect: DWORD, lindex: LONG, pvAspect: *mut core::ffi::c_void, ptd: *mut DVTARGETDEVICE, hicTargetDev: HDC, ppColorSet: *mut *mut LOGPALETTE) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IViewObject_iface)) as *mut CDefView;
    CDefView_GetColorSet_body(this, dwDrawAspect, lindex, pvAspect, ptd, hicTargetDev, ppColorSet)
}
unsafe extern "system" fn CDefView_IViewObject_Freeze(This: *mut IViewObject, dwDrawAspect: DWORD, lindex: LONG, pvAspect: *mut core::ffi::c_void, pdwFreeze: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IViewObject_iface)) as *mut CDefView;
    CDefView_Freeze_body(this, dwDrawAspect, lindex, pvAspect, pdwFreeze)
}
unsafe extern "system" fn CDefView_IViewObject_Unfreeze(This: *mut IViewObject, dwFreeze: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IViewObject_iface)) as *mut CDefView;
    CDefView_Unfreeze_body(this, dwFreeze)
}
unsafe extern "system" fn CDefView_IViewObject_SetAdvise(This: *mut IViewObject, aspects: DWORD, advf: DWORD, pAdvSink: *mut IAdviseSink) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IViewObject_iface)) as *mut CDefView;
    CDefView_SetAdvise_body(this, aspects, advf, pAdvSink)
}
unsafe extern "system" fn CDefView_IViewObject_GetAdvise(This: *mut IViewObject, pAspects: *mut DWORD, pAdvf: *mut DWORD, ppAdvSink: *mut *mut IAdviseSink) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IViewObject_iface)) as *mut CDefView;
    CDefView_GetAdvise_body(this, pAspects, pAdvf, ppAdvSink)
}

pub static CDEFVIEW_IVIEWOBJECT_VTBL: IViewObjectVtbl = IViewObjectVtbl {
    QueryInterface: CDefView_IViewObject_QueryInterface,
    AddRef: CDefView_IViewObject_AddRef,
    Release: CDefView_IViewObject_Release,
    Draw: CDefView_IViewObject_Draw,
    GetColorSet: CDefView_IViewObject_GetColorSet,
    Freeze: CDefView_IViewObject_Freeze,
    Unfreeze: CDefView_IViewObject_Unfreeze,
    SetAdvise: CDefView_IViewObject_SetAdvise,
    GetAdvise: CDefView_IViewObject_GetAdvise,
};

unsafe extern "system" fn CDefView_IServiceProvider_QueryInterface(This: *mut IServiceProvider, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IServiceProvider_iface)) as *mut CDefView;
    CDefView_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CDefView_IServiceProvider_AddRef(This: *mut IServiceProvider) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IServiceProvider_iface)) as *mut CDefView;
    CDefView_AddRef(this)
}
unsafe extern "system" fn CDefView_IServiceProvider_Release(This: *mut IServiceProvider) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IServiceProvider_iface)) as *mut CDefView;
    CDefView_Release(this)
}
unsafe extern "system" fn CDefView_IServiceProvider_QueryService(This: *mut IServiceProvider, guidService: REFGUID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CDefView, IServiceProvider_iface)) as *mut CDefView;
    CDefView_QueryService_body(this, guidService, riid, ppvObject)
}

pub static CDEFVIEW_ISERVICEPROVIDER_VTBL: IServiceProviderVtbl = IServiceProviderVtbl {
    QueryInterface: CDefView_IServiceProvider_QueryInterface,
    AddRef: CDefView_IServiceProvider_AddRef,
    Release: CDefView_IServiceProvider_Release,
    QueryService: CDefView_IServiceProvider_QueryService,
};

unsafe fn CDefView_QueryInterface(
    this: *mut CDefView, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellView)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellView3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_CDefView)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellView3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellView2)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellView3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellView3)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellView3_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IFolderView)) {
        *ppv = core::ptr::addr_of_mut!((*this).IFolderView_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IShellFolderView)) {
        *ppv = core::ptr::addr_of_mut!((*this).IShellFolderView_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IOleCommandTarget)) {
        *ppv = core::ptr::addr_of_mut!((*this).IOleCommandTarget_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDropTarget)) {
        *ppv = core::ptr::addr_of_mut!((*this).IDropTarget_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDropSource)) {
        *ppv = core::ptr::addr_of_mut!((*this).IDropSource_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IViewObject)) {
        *ppv = core::ptr::addr_of_mut!((*this).IViewObject_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IServiceProvider)) {
        *ppv = core::ptr::addr_of_mut!((*this).IServiceProvider_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CDefView_AddRef(this);
    S_OK
}

unsafe fn CDefView_AddRef(this: *mut CDefView) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CDefView_Release(this: *mut CDefView) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CDefView_dtor_body(this);
        CDefView_cpp_com_assign_m_pSFParent(this, core::ptr::null_mut());
        CDefView_cpp_com_assign_m_pSF2Parent(this, core::ptr::null_mut());
        CDefView_cpp_com_assign_m_pSDParent(this, core::ptr::null_mut());
        CDefView_cpp_com_assign_m_pShellFolderViewCB(this, core::ptr::null_mut());
        CDefView_cpp_com_assign_m_pShellBrowser(this, core::ptr::null_mut());
        CDefView_cpp_com_assign_m_pCommDlgBrowser(this, core::ptr::null_mut());
        CDefView_cpp_com_assign_m_pFolderFilter(this, core::ptr::null_mut());
        CDefView_cpp_com_assign_m_pShellFolderViewDual(this, core::ptr::null_mut());
        CDefView_cpp_com_assign_m_pAdvSink(this, core::ptr::null_mut());
        CDefView_cpp_com_assign_m_pSourceDataObject(this, core::ptr::null_mut());
        CDefView_cpp_com_assign_m_pCurDropTarget(this, core::ptr::null_mut());
        CDefView_cpp_com_assign_m_pCurDataObject(this, core::ptr::null_mut());
        CDefView_cpp_com_assign_m_pCM(this, core::ptr::null_mut());
        CDefView_cpp_com_assign_m_pFileMenu(this, core::ptr::null_mut());
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null() && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Internal activation: shellutils.h ShellObjectCreator<T> uses this non-aggregating factory.
#[no_mangle]
pub unsafe extern "C" fn CDefView_CreateInstance(
    shellFolder: *mut IShellFolder,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CDefView>()) as *mut CDefView;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CDefView {
        IShellView3_iface: IShellView3 { lpVtbl: core::ptr::addr_of!(CDEFVIEW_ISHELLVIEW3_VTBL) },
        IFolderView_iface: IFolderView { lpVtbl: core::ptr::addr_of!(CDEFVIEW_IFOLDERVIEW_VTBL) as *mut _ },
        IShellFolderView_iface: IShellFolderView { lpVtbl: core::ptr::addr_of!(CDEFVIEW_ISHELLFOLDERVIEW_VTBL) },
        IOleCommandTarget_iface: IOleCommandTarget { lpVtbl: core::ptr::addr_of!(CDEFVIEW_IOLECOMMANDTARGET_VTBL) },
        IDropTarget_iface: IDropTarget { lpVtbl: core::ptr::addr_of!(CDEFVIEW_IDROPTARGET_VTBL) as *mut _ },
        IDropSource_iface: IDropSource { lpVtbl: core::ptr::addr_of!(CDEFVIEW_IDROPSOURCE_VTBL) },
        IViewObject_iface: IViewObject { lpVtbl: core::ptr::addr_of!(CDEFVIEW_IVIEWOBJECT_VTBL) },
        IServiceProvider_iface: IServiceProvider { lpVtbl: core::ptr::addr_of!(CDEFVIEW_ISERVICEPROVIDER_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_hWnd: core::mem::zeroed(),
        m_atl: AtlWindowState::new(),
        m_pSFParent: core::mem::zeroed(),
        m_pSF2Parent: core::mem::zeroed(),
        m_pSDParent: core::mem::zeroed(),
        m_pShellFolderViewCB: core::mem::zeroed(),
        m_pShellBrowser: core::mem::zeroed(),
        m_pCommDlgBrowser: core::mem::zeroed(),
        m_pFolderFilter: core::mem::zeroed(),
        m_pShellFolderViewDual: core::mem::zeroed(),
        // NOT `zeroed()`: utils.h gives m_hWndNext the default member initializer
        // `HWND_BOTTOM`, and `Hook` only hooks when it still holds that sentinel.
        m_ClipboardChain: ClipboardViewerChain::new(),
        m_ListView: core::mem::zeroed(),
        m_hWndParent: core::mem::zeroed(),
        m_FolderSettings: core::mem::zeroed(),
        m_hMenu: core::mem::zeroed(),
        m_hMenuArrangeModes: core::mem::zeroed(),
        m_hMenuViewModes: core::mem::zeroed(),
        m_hContextMenu: core::mem::zeroed(),
        m_bmenuBarInitialized: core::mem::zeroed(),
        m_uState: core::mem::zeroed(),
        m_cidl: core::mem::zeroed(),
        m_apidl: core::mem::zeroed(),
        m_pidlParent: core::mem::zeroed(),
        m_LoadColumnsList: core::mem::zeroed(),
        m_ListToFolderColMap: core::mem::zeroed(),
        m_sortInfo: core::mem::zeroed(),
        m_hNotify: core::mem::zeroed(),
        m_hAccel: core::mem::zeroed(),
        m_dwAspects: core::mem::zeroed(),
        m_dwAdvf: core::mem::zeroed(),
        m_pAdvSink: core::mem::zeroed(),
        m_pSourceDataObject: core::mem::zeroed(),
        m_pCurDropTarget: core::mem::zeroed(),
        m_pCurDataObject: core::mem::zeroed(),
        m_iDragOverItem: core::mem::zeroed(),
        m_cScrollDelay: core::mem::zeroed(),
        m_ptLastMousePos: core::mem::zeroed(),
        m_ptFirstMousePos: core::mem::zeroed(),
        m_grfKeyState: core::mem::zeroed(),
        m_pCM: core::mem::zeroed(),
        m_pFileMenu: core::mem::zeroed(),
        m_isEditing: core::mem::zeroed(),
        m_SpecialFolder: core::mem::zeroed(),
        m_isFullStatusBar: core::mem::zeroed(),
        m_ScheduledStatusbarUpdate: core::mem::zeroed(),
        m_HasCutItems: core::mem::zeroed(),
        m_Category: core::mem::zeroed(),
        m_Destroyed: core::mem::zeroed(),
        m_viewinfo_data: core::mem::zeroed(),
        m_hMyComputerIcon: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CDefView_ctor_body(this);
    let mut hr = CDefView_Initialize_body(this, shellFolder);
    if hr >= 0 {
        hr = CDefView_QueryInterface(this, riid, ppv);
    }
    CDefView_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const CONTEXT_MENU_BASE_ID: UINT = 1;

pub const FCIDM_SHVIEW_ALIGNTOGRID: UINT = 0x7033;

pub const FCIDM_SHVIEW_ARRANGE: UINT = 0x7001;

pub const FCIDM_SHVIEW_AUTOARRANGE: UINT = 0x7031;

pub const FCIDM_SHVIEW_BIGICON: UINT = 0x7029;

pub const FCIDM_SHVIEW_COPY: UINT = 0x7019;

pub const FCIDM_SHVIEW_COPYTO: UINT = 0x701E;

pub const FCIDM_SHVIEW_CREATELINK: UINT = 0x7010;

pub const FCIDM_SHVIEW_CUT: UINT = 0x7018;

pub const FCIDM_SHVIEW_DELETE: UINT = 0x7011;

pub const FCIDM_SHVIEW_INSERT: UINT = 0x701A;

pub const FCIDM_SHVIEW_INSERTLINK: UINT = 0x701C;

pub const FCIDM_SHVIEW_INVERTSELECTION: UINT = 0x7022;

pub const FCIDM_SHVIEW_LISTVIEW: UINT = 0x702B;

pub const FCIDM_SHVIEW_MOVETO: UINT = 0x701F;

pub const FCIDM_SHVIEW_NEWFOLDER: UINT = 0x7053;

pub const FCIDM_SHVIEW_PROPERTIES: UINT = 0x7013;

pub const FCIDM_SHVIEW_REFRESH: UINT = 0x7103;

pub const FCIDM_SHVIEW_RENAME: UINT = 0x7012;

pub const FCIDM_SHVIEW_REPORTVIEW: UINT = 0x702C;

pub const FCIDM_SHVIEW_SELECTALL: UINT = 0x7021;

pub const FCIDM_SHVIEW_SMALLICON: UINT = 0x702A;

pub const FCIDM_SHVIEW_SNAPTOGRID: UINT = 0x7032;

pub const FCIDM_SHVIEW_TILEVIEW: UINT = 0x702E;

pub const FCIDM_SHVIEW_UNDO: UINT = 0x701B;

pub const FCIDM_SHVIEW_VIEW: UINT = 0x7002;

pub const FCIDM_TB_REPORTVIEW: UINT = 0xA004;

pub const FCIDM_TB_SMALLICON: UINT = 0xA003;

pub const IDA_SHELLVIEW: UINT = 1;

pub const IDI_SHELL_COMPUTER_DESKTOP: UINT = 181;

pub const IDM_DVSELECT: UINT = 0x104;

pub const IDS_MYCOMPUTER: UINT = 9216;

pub const IDS_OBJECTS: UINT = 6466;

pub const IDS_OBJECTS_SELECTED: UINT = 6477;

pub const ID_LISTVIEW: UINT = 1;

pub const SCROLLAREAWIDTH: UINT = 20;

pub const CSIDL_BITBUCKET: INT = 0x000a;

pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

pub const SFGAO_CANCOPY: SFGAOF = 0x0000_0001;

pub const SFGAO_CANDELETE: SFGAOF = 0x0000_0020;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const SFGAO_CANRENAME: SFGAOF = 0x0000_0010;

pub const SFGAO_HASPROPSHEET: SFGAOF = 0x0000_0040;

pub const CDB2GVF_NOINCLUDEITEM: DWORD = 0x00000010;

pub const CDB2GVF_NOSELECTVERB: DWORD = 0x00000008;

pub const CDB2GVF_SHOWALLFILES: DWORD = 0x00000001;

pub const CDB2N_CONTEXTMENU_DONE: DWORD = 1;

pub const CDB2N_CONTEXTMENU_START: DWORD = 2;

pub const CDBOSC_KILLFOCUS: DWORD = 0x00000001;

pub const CDBOSC_RENAME: DWORD = 0x00000003;

pub const CDBOSC_SELCHANGE: DWORD = 0x00000002;

pub const CDBOSC_SETFOCUS: DWORD = 0x00000000;

pub const CDRF_DODEFAULT: DWORD = 0x0;

pub const CLR_INVALID: DWORD = 0xFFFFFFFF;

pub const CLR_NONE: DWORD = 0xffffffff;

pub const CMF_CANRENAME: DWORD = 0x00000010;

pub const CMF_DEFAULTONLY: DWORD = 0x00000001;

pub const CMF_EXPLORE: DWORD = 0x00000004;

pub const CMF_NORMAL: DWORD = 0x00000000;

pub const CMIC_MASK_CONTROL_DOWN: DWORD = 0x40000000;

pub const CMIC_MASK_PTINVOKE: DWORD = 0x20000000;

pub const CMIC_MASK_SHIFT_DOWN: DWORD = 0x10000000;

pub const COLOR_WINDOW: DWORD = 5;

pub const COLOR_WINDOWTEXT: DWORD = 8;

pub const CSIDL_DESKTOP: DWORD = 0x0000;

pub const CSIDL_NETWORK: DWORD = 0x0012;

pub const CS_PARENTDC: DWORD = 128;

pub const DVCMDID_RESET_DEFAULTFOLDER_SETTINGS: DWORD = 1;

pub const DVCMDID_SET_DEFAULTFOLDER_SETTINGS: DWORD = 0;

pub const EM_SCROLLCARET: DWORD = 183;

pub const EM_SETLIMITTEXT: DWORD = 197;

pub const EM_SETSEL: DWORD = 177;

pub const ERROR_INVALID_DATA: DWORD = 13;

pub const ERROR_MORE_DATA: DWORD = 234;

pub const FCIDM_BROWSERFIRST: DWORD = 0xA000;

pub const FCIDM_BROWSERLAST: DWORD = 0xbf00;

pub const FCIDM_SHVIEWFIRST: DWORD = 0x0000;

pub const FCIDM_SHVIEWLAST: DWORD = 0x7fff;

pub const FCIDM_SHVIEW_DESELECTALL: DWORD = 0x7023;

pub const FCW_STATUS: DWORD = 0x0001;

pub const FCW_TOOLBAR: DWORD = 0x0002;

pub const FCW_TREE: DWORD = 0x0003;

pub const FVM_DETAILS: DWORD = 4;

pub const FVM_ICON: DWORD = 1;

pub const FVM_LIST: DWORD = 3;

pub const FVM_SMALLICON: DWORD = 2;

pub const FWF_ALIGNLEFT: DWORD = 0x800;

pub const FWF_AUTOARRANGE: DWORD = 0x1;

pub const FWF_DESKTOP: DWORD = 0x20;

pub const FWF_FULLROWSELECT: DWORD = 0x200000;

pub const FWF_NOBROWSERVIEWSTATE: DWORD = 0x10000000;

pub const FWF_NOCLIENTEDGE: DWORD = 0x200;

pub const FWF_NOCOLUMNHEADER: DWORD = 0x800000;

pub const FWF_NOGROUPING: DWORD = 0x100000;

pub const FWF_NOHEADERINALLVIEWS: DWORD = 0x1000000;

pub const FWF_NOSCROLL: DWORD = 0x400;

pub const FWF_NOSUBFOLDERS: DWORD = 0x80;

pub const FWF_OWNERDATA: DWORD = 0x8;

pub const FWF_SINGLECLICKACTIVATE: DWORD = 0x8000;

pub const FWF_SINGLESEL: DWORD = 0x40;

pub const FWF_SNAPTOGRID: DWORD = 0x4;

pub const FWF_TRANSPARENT: DWORD = 0x100;

pub const HDF_SORTDOWN: DWORD = 0x200;

pub const HDF_SORTUP: DWORD = 0x400;

pub const HDI_FORMAT: DWORD = 0x4;

pub const HTCLIENT: DWORD = 1;

pub const KEY_READ: DWORD = 0x20019;

pub const LVA_DEFAULT: DWORD = 0x0;

pub const LVA_SNAPTOGRID: DWORD = 0x5;

pub const LVCF_FMT: DWORD = 0x1;

pub const LVCF_SUBITEM: DWORD = 0x8;

pub const LVCF_TEXT: DWORD = 0x4;

pub const LVCF_WIDTH: DWORD = 0x2;

pub const LVIF_DI_SETITEM: DWORD = 0x1000;

pub const LVIF_IMAGE: DWORD = 0x2;

pub const LVIF_PARAM: DWORD = 0x4;

pub const LVIF_STATE: DWORD = 0x8;

pub const LVIF_TEXT: DWORD = 0x1;

pub const LVIR_ICON: DWORD = 1;

pub const LVIR_SELECTBOUNDS: DWORD = 3;

pub const LVIS_CUT: DWORD = 0x4;

pub const LVIS_FOCUSED: DWORD = 0x1;

pub const LVIS_SELECTED: DWORD = 0x2;

pub const LVKF_ALT: DWORD = 0x1;

pub const LVNI_ALL: DWORD = 0x0;

pub const LVNI_FOCUSED: DWORD = 0x1;

pub const LVNI_SELECTED: DWORD = 0x2;

pub const LVSIL_NORMAL: DWORD = 0;

pub const LVSIL_SMALL: DWORD = 1;

pub const LVS_ALIGNLEFT: DWORD = 0x800;

pub const LVS_ALIGNTOP: DWORD = 0x0;

pub const LVS_AUTOARRANGE: DWORD = 0x100;

pub const LVS_EDITLABELS: DWORD = 0x200;

pub const LVS_EX_DOUBLEBUFFER: DWORD = 0x10000;

pub const LVS_EX_FULLROWSELECT: DWORD = 0x20;

pub const LVS_EX_HEADERINALLVIEWS: DWORD = 0x2000000;

pub const LVS_EX_INFOTIP: DWORD = 0x400;

pub const LVS_EX_LABELTIP: DWORD = 0x4000;

pub const LVS_EX_ONECLICKACTIVATE: DWORD = 0x40;

pub const LVS_EX_SNAPTOGRID: DWORD = 0x80000;

pub const LVS_EX_TRACKSELECT: DWORD = 0x8;

pub const LVS_EX_TRANSPARENTBKGND: DWORD = 0x400000;

pub const LVS_EX_TRANSPARENTSHADOWTEXT: DWORD = 0x800000;

pub const LVS_EX_UNDERLINECOLD: DWORD = 0x1000;

pub const LVS_EX_UNDERLINEHOT: DWORD = 0x800;

pub const LVS_ICON: DWORD = 0x0;

pub const LVS_LIST: DWORD = 0x3;

pub const LVS_NOCOLUMNHEADER: DWORD = 0x4000;

pub const LVS_NOSORTHEADER: DWORD = 0x8000;

pub const LVS_REPORT: DWORD = 0x1;

pub const LVS_SHAREIMAGELISTS: DWORD = 0x40;

pub const LVS_SHOWSELALWAYS: DWORD = 0x8;

pub const LVS_SINGLESEL: DWORD = 0x4;

pub const LVS_SMALLICON: DWORD = 0x2;

pub const LVS_TYPEMASK: DWORD = 0x3;

pub const MFS_CHECKED: DWORD = 8;

pub const MFS_DISABLED: DWORD = 3;

pub const MF_BYCOMMAND: DWORD = 0;

pub const MF_BYPOSITION: DWORD = 1024;

pub const MF_CHECKED: DWORD = 8;

pub const MF_DISABLED: DWORD = 2;

pub const MF_ENABLED: DWORD = 0;

pub const MF_GRAYED: DWORD = 1;

pub const MF_SEPARATOR: DWORD = 0x800;

pub const MF_STRING: DWORD = 0;

pub const MF_UNCHECKED: DWORD = 0;

pub const MIIM_DATA: DWORD = 32;

pub const MIIM_STATE: DWORD = 1;

pub const MIIM_SUBMENU: DWORD = 4;

pub const MK_LBUTTON: DWORD = 1;

pub const MK_RBUTTON: DWORD = 2;

pub const MM_ADDSEPARATOR: DWORD = 0x00000001;

pub const MM_DONTREMOVESEPS: DWORD = 0x00000004;

pub const MM_SUBMENUSHAVEIDS: DWORD = 0x00000002;

pub const REST_NOVIEWCONTEXTMENU: DWORD = 0x04000000;

pub const SB_HORZ: DWORD = 0;

pub const SB_LINEDOWN: DWORD = 1;

pub const SB_LINEUP: DWORD = 0;

pub const SB_VERT: DWORD = 1;

pub const SFGAO_BROWSABLE: DWORD = 0x08000000;

pub const SFGAO_GHOSTED: DWORD = 0x00008000;

pub const SFGAO_HIDDEN: DWORD = 0x00080000;

pub const SFVM_ADDINGOBJECT: DWORD = 29;

pub const SFVM_ADDPROPERTYPAGES: DWORD = 47;

pub const SFVM_CANSELECTALL: DWORD = 35;

pub const SFVM_COLUMNCLICK: DWORD = 24;

pub const SFVM_DEFVIEWMODE: DWORD = 27;

pub const SFVM_FSNOTIFY: DWORD = 14;

pub const SFVM_GETCOMMANDDIR: DWORD = 33;

pub const SFVM_GETNAMELENGTH: DWORD = 13;

pub const SFVM_GETNOTIFY: DWORD = 49;

pub const SFVM_GET_CUSTOMVIEWINFO: DWORD = 77;

pub const SFVM_HWNDMAIN: DWORD = 46;

pub const SFVM_LISTREFRESHED: DWORD = 17;

pub const SFVM_QUERYFSNOTIFY: DWORD = 25;

pub const SFVM_REMOVINGOBJECT: DWORD = 30;

pub const SFVM_SELECTIONCHANGED: DWORD = 8;

pub const SFVM_SIZE: DWORD = 57;

pub const SFVM_UPDATINGOBJECT: DWORD = 43;

pub const SFVM_VIEWRELEASE: DWORD = 12;

pub const SFVM_WINDOWCLOSING: DWORD = 16;

pub const SFVM_WINDOWCREATED: DWORD = 15;

pub const SHCIDS_CANONICALONLY: DWORD = 0x10000000;

pub const SHCIDS_COLUMNMASK: DWORD = 0x0000FFFF;

pub const SHCNE_ALLEVENTS: DWORD = 0x7FFFFFFF;

pub const SHCNE_ASSOCCHANGED: DWORD = 0x08000000;

pub const SHCNE_ATTRIBUTES: DWORD = 0x00000800;

pub const SHCNE_CREATE: DWORD = 0x00000002;

pub const SHCNE_DELETE: DWORD = 0x00000004;

pub const SHCNE_DRIVEADD: DWORD = 0x00000100;

pub const SHCNE_DRIVEREMOVED: DWORD = 0x00000080;

pub const SHCNE_EXTENDED_EVENT: DWORD = 0x04000000;

pub const SHCNE_FREESPACE: DWORD = 0x00040000;

pub const SHCNE_INTERRUPT: DWORD = 0x80000000;

pub const SHCNE_MEDIAINSERTED: DWORD = 0x00000020;

pub const SHCNE_MEDIAREMOVED: DWORD = 0x00000040;

pub const SHCNE_MKDIR: DWORD = 0x00000008;

pub const SHCNE_RENAMEFOLDER: DWORD = 0x00020000;

pub const SHCNE_RENAMEITEM: DWORD = 0x00000001;

pub const SHCNE_RMDIR: DWORD = 0x00000010;

pub const SHCNE_UPDATEDIR: DWORD = 0x00001000;

pub const SHCNE_UPDATEIMAGE: DWORD = 0x00008000;

pub const SHCNE_UPDATEITEM: DWORD = 0x00002000;

pub const SHCNRF_InterruptLevel: DWORD = 0x0001;

pub const SHCNRF_NewDelivery: DWORD = 0x8000;

pub const SHCNRF_ShellLevel: DWORD = 0x0002;

pub const SHCOLSTATE_HIDDEN: DWORD = 0x100;

pub const SHCOLSTATE_ONBYDEFAULT: DWORD = 0x10;

pub const SHCOLSTATE_SECONDARYUI: DWORD = 0x80;

pub const SHCOLSTATE_TYPE_STR: DWORD = 1;

pub const SHCONTF_FOLDERS: DWORD = 32;

pub const SHCONTF_INCLUDEHIDDEN: DWORD = 128;

pub const SHCONTF_NONFOLDERS: DWORD = 64;

pub const SHGDN_FOREDITING: DWORD = 0x1000;

pub const SHGDN_INFOLDER: DWORD = 1;

pub const SPI_GETICONTITLELOGFONT: DWORD = 0x001F;

pub const SPI_SETDESKWALLPAPER: DWORD = 0x0014;

pub const SRCCOPY: DWORD = 0xCC0020;

pub const SSF_DOUBLECLICKINWEBVIEW: DWORD = 0x00000080;

pub const SSF_SHOWALLOBJECTS: DWORD = 0x00000001;

pub const SSF_SHOWSUPERHIDDEN: DWORD = 0x00040000;

pub const SSF_WIN95CLASSIC: DWORD = 0x00000400;

pub const STGM_READ: DWORD = 0x00000000;

pub const STGM_WRITE: DWORD = 0x00000001;

pub const SV3CVW3_DEFAULT: DWORD = 0x0;

pub const SV3CVW3_FORCEVIEWMODE: DWORD = 0x2;

pub const SVGIO_ALLVIEW: DWORD = 0x2;

pub const SVGIO_BACKGROUND: DWORD = 0x0;

pub const SVGIO_SELECTION: DWORD = 0x1;

pub const SVGIO_TYPE_MASK: DWORD = 0xf;

pub const SVSI_DESELECTOTHERS: DWORD = 0x00000004;

pub const SVSI_EDIT: DWORD = 0x00000003;

pub const SVSI_ENSUREVISIBLE: DWORD = 0x00000008;

pub const SVSI_FOCUSED: DWORD = 0x00000010;

pub const SVSI_SELECT: DWORD = 0x00000001;

pub const SVUIA_ACTIVATE_FOCUS: DWORD = 2;

pub const SVUIA_ACTIVATE_NOFOCUS: DWORD = 1;

pub const SVUIA_DEACTIVATE: DWORD = 0;

pub const SWP_NOMOVE: DWORD = 2;

pub const SWP_NOSIZE: DWORD = 1;

pub const SWP_SHOWWINDOW: DWORD = 64;

pub const SW_SHOW: DWORD = 5;

pub const TPM_LEFTALIGN: DWORD = 0;

pub const TPM_LEFTBUTTON: DWORD = 0;

pub const TPM_RETURNCMD: DWORD = 256;

pub const TPM_RIGHTBUTTON: DWORD = 2;

pub const TPM_TOPALIGN: DWORD = 0;

pub const UINT_MAX: DWORD = 0xffffffff;

pub const VK_CONTROL: DWORD = 17;

pub const VK_SHIFT: DWORD = 16;

pub const VT_I4: DWORD = 3;

pub const VT_INT_PTR: DWORD = 37;

pub const WM_ACTIVATE: DWORD = 6;

pub const WM_CHANGECBCHAIN: DWORD = 781;

pub const WM_COMMAND: DWORD = 273;

pub const WM_CREATE: DWORD = 1;

pub const WM_DESTROY: DWORD = 2;

pub const WM_DRAWCLIPBOARD: DWORD = 776;

pub const WM_DRAWITEM: DWORD = 43;

pub const WM_ERASEBKGND: DWORD = 20;

pub const WM_GETDLGCODE: DWORD = 135;

pub const WM_GETFONT: DWORD = 49;

pub const WM_HSCROLL: DWORD = 276;

pub const WM_INITMENUPOPUP: DWORD = 279;

pub const WM_KEYFIRST: DWORD = 256;

pub const WM_KEYLAST: DWORD = 264;

pub const WM_KILLFOCUS: DWORD = 8;

pub const WM_MEASUREITEM: DWORD = 44;

pub const WM_NCCREATE: DWORD = 129;

pub const WM_SETCURSOR: DWORD = 32;

pub const WM_SETFOCUS: DWORD = 7;

pub const WM_SETTINGCHANGE: DWORD = 26;

pub const WM_SHOWWINDOW: DWORD = 24;

pub const WM_SIZE: DWORD = 5;

pub const WM_SYSCOLORCHANGE: DWORD = 21;

pub const WM_VSCROLL: DWORD = 277;

pub const WS_CHILD: DWORD = 0x40000000;

pub const WS_CHILDWINDOW: DWORD = 0x40000000;

pub const WS_CLIPCHILDREN: DWORD = 0x2000000;

pub const WS_CLIPSIBLINGS: DWORD = 0x4000000;

pub const WS_EX_CLIENTEDGE: DWORD = 512;

pub const WS_TABSTOP: DWORD = 0x10000;

pub const WS_VISIBLE: DWORD = 0x10000000;

pub const _: DWORD = 1;

pub const _M_ARM64: DWORD = 1;

#[repr(C)]
pub struct BIND_OPTS {
    pub cbStruct: DWORD,
    pub grfFlags: DWORD,
    pub grfMode: DWORD,
    pub dwTickCountDeadline: DWORD,
}



#[repr(C)]
pub struct DISPPARAMS {
    pub rgvarg: *mut VARIANTARG,
    pub rgdispidNamedArgs: *mut DISPID,
    pub cArgs: UINT,
    pub cNamedArgs: UINT,
}

#[repr(C)]
pub struct DVTARGETDEVICE {
    pub tdSize: DWORD,
    pub tdDriverNameOffset: WORD,
    pub tdDeviceNameOffset: WORD,
    pub tdPortNameOffset: WORD,
    pub tdExtDevmodeOffset: WORD,
    pub tdData: [BYTE; 1],
}

#[repr(C)]
pub struct IDLDESC {
    pub dwReserved: ULONG_PTR,
    pub wIDLFlags: USHORT,
}







#[repr(C)]
pub struct OLEMENUGROUPWIDTHS {
    pub width: [LONG; 6],
}

#[repr(C)]
pub struct PARAMDESC {
    pub pparamdescex: LPPARAMDESCEX,
    pub wParamFlags: USHORT,
}



#[repr(C)]
pub struct STATDATA {
    pub formatetc: FORMATETC,
    pub advf: DWORD,
    pub pAdvSink: *mut IAdviseSink,
    pub dwConnection: DWORD,
}

#[repr(C)]
pub struct STATSTG {
    pub pwcsName: LPOLESTR,
    pub r#type: DWORD,
    pub cbSize: ULARGE_INTEGER,
    pub mtime: FILETIME,
    pub ctime: FILETIME,
    pub atime: FILETIME,
    pub grfMode: DWORD,
    pub grfLocksSupported: DWORD,
    pub clsid: CLSID,
    pub grfStateBits: DWORD,
    pub reserved: DWORD,
}

#[repr(C)]
pub struct ULARGE_INTEGER {
    pub QuadPart: ULONGLONG,
}

// Deterministic from shldisp.h
#[repr(C)]
pub struct IAsyncOperation {
    pub lpVtbl: *const IAsyncOperationVtbl,
}
#[repr(C)]
pub struct IAsyncOperationVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IAsyncOperation, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IAsyncOperation) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IAsyncOperation) -> ULONG,
    pub SetAsyncMode: unsafe extern "system" fn(This: *mut IAsyncOperation, fDoOpAsync: BOOL) -> HRESULT,
    pub GetAsyncMode: unsafe extern "system" fn(This: *mut IAsyncOperation, pfIsOpAsync: *mut BOOL) -> HRESULT,
    pub StartOperation: unsafe extern "system" fn(This: *mut IAsyncOperation, pbcReserved: *mut IBindCtx) -> HRESULT,
    pub InOperation: unsafe extern "system" fn(This: *mut IAsyncOperation, pfInAsyncOp: *mut BOOL) -> HRESULT,
    pub EndOperation: unsafe extern "system" fn(This: *mut IAsyncOperation, hResult: HRESULT, pbcReserved: *mut IBindCtx, dwEffects: DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IAsyncOperation: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct ICommDlgBrowser {
    pub lpVtbl: *const ICommDlgBrowserVtbl,
}
#[repr(C)]
pub struct ICommDlgBrowserVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ICommDlgBrowser, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ICommDlgBrowser) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ICommDlgBrowser) -> ULONG,
    pub OnDefaultCommand: unsafe extern "system" fn(This: *mut ICommDlgBrowser, shv: *mut IShellView) -> HRESULT,
    pub OnStateChange: unsafe extern "system" fn(This: *mut ICommDlgBrowser, shv: *mut IShellView, uChange: ULONG) -> HRESULT,
    pub IncludeObject: unsafe extern "system" fn(This: *mut ICommDlgBrowser, shv: *mut IShellView, pidl: PCUITEMID_CHILD) -> HRESULT,
}
extern "C" {
    pub static IID_ICommDlgBrowser: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct ICommDlgBrowser2 {
    pub lpVtbl: *const ICommDlgBrowser2Vtbl,
}
#[repr(C)]
pub struct ICommDlgBrowser2Vtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut ICommDlgBrowser2, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut ICommDlgBrowser2) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut ICommDlgBrowser2) -> ULONG,
    pub OnDefaultCommand: unsafe extern "system" fn(This: *mut ICommDlgBrowser2, shv: *mut IShellView) -> HRESULT,
    pub OnStateChange: unsafe extern "system" fn(This: *mut ICommDlgBrowser2, shv: *mut IShellView, uChange: ULONG) -> HRESULT,
    pub IncludeObject: unsafe extern "system" fn(This: *mut ICommDlgBrowser2, shv: *mut IShellView, pidl: PCUITEMID_CHILD) -> HRESULT,
    pub Notify: unsafe extern "system" fn(This: *mut ICommDlgBrowser2, ppshv: *mut IShellView, dwNotifyType: DWORD) -> HRESULT,
    pub GetDefaultMenuText: unsafe extern "system" fn(This: *mut ICommDlgBrowser2, ppshv: *mut IShellView, pszText: LPWSTR, cchMax: i32) -> HRESULT,
    pub GetViewFlags: unsafe extern "system" fn(This: *mut ICommDlgBrowser2, pdwFlags: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_ICommDlgBrowser2: IID;
}

// Deterministic from shlobj_undoc.h
#[repr(C)]
pub struct IExplorerToolbar {
    pub lpVtbl: *const IExplorerToolbarVtbl,
}
#[repr(C)]
pub struct IExplorerToolbarVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IExplorerToolbar, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IExplorerToolbar) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IExplorerToolbar) -> ULONG,
    pub SetCommandTarget: unsafe extern "system" fn(This: *mut IExplorerToolbar, theTarget: *mut IUnknown, category: *mut GUID, param14: i32) -> HRESULT,
    pub Unknown1: unsafe extern "system" fn(This: *mut IExplorerToolbar) -> HRESULT,
    pub AddButtons: unsafe extern "system" fn(This: *mut IExplorerToolbar, pguidCmdGroup: *const GUID, buttonCount: i32, buttons: *mut TBBUTTON) -> HRESULT,
    pub AddString: unsafe extern "system" fn(This: *mut IExplorerToolbar, pguidCmdGroup: *const GUID, param10: HINSTANCE, param14: LPCTSTR, param18: *mut i32) -> HRESULT,
    pub GetButton: unsafe extern "system" fn(This: *mut IExplorerToolbar, paramC: *const GUID, param10: i32, param14: i32) -> HRESULT,
    pub GetState: unsafe extern "system" fn(This: *mut IExplorerToolbar, pguidCmdGroup: *const GUID, commandID: i32, theState: *mut i32) -> HRESULT,
    pub SetState: unsafe extern "system" fn(This: *mut IExplorerToolbar, pguidCmdGroup: *const GUID, commandID: i32, theState: i32) -> HRESULT,
    pub AddBitmap: unsafe extern "system" fn(This: *mut IExplorerToolbar, pguidCmdGroup: *const GUID, param10: i32, buttonCount: i32, lParam: *mut TBADDBITMAP, newIndex: *mut i32, param20: COLORREF) -> HRESULT,
    pub GetBitmapSize: unsafe extern "system" fn(This: *mut IExplorerToolbar, paramC: *mut i32) -> HRESULT,
    pub SendToolbarMsg: unsafe extern "system" fn(This: *mut IExplorerToolbar, pguidCmdGroup: *const GUID, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, result: *mut LRESULT) -> HRESULT,
    pub SetImageList: unsafe extern "system" fn(This: *mut IExplorerToolbar, pguidCmdGroup: *const GUID, param10: HIMAGELIST, param14: HIMAGELIST, param18: HIMAGELIST) -> HRESULT,
    pub ModifyButton: unsafe extern "system" fn(This: *mut IExplorerToolbar, paramC: *const GUID, param10: i32, param14: i32) -> HRESULT,
}
extern "C" {
    pub static IID_IExplorerToolbar: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IFolderFilter {
    pub lpVtbl: *const IFolderFilterVtbl,
}
#[repr(C)]
pub struct IFolderFilterVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IFolderFilter, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IFolderFilter) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IFolderFilter) -> ULONG,
    pub ShouldShow: unsafe extern "system" fn(This: *mut IFolderFilter, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, pidlItem: PCUITEMID_CHILD) -> HRESULT,
    pub GetEnumFlags: unsafe extern "system" fn(This: *mut IFolderFilter, psf: *mut IShellFolder, pidlFolder: PCIDLIST_ABSOLUTE, phwnd: *mut HWND, pgrfFlags: *mut DWORD) -> HRESULT,
}
extern "C" {
    pub static IID_IFolderFilter: IID;
}

// Deterministic from oaidl.h
#[repr(C)]
pub struct IPropertyBag {
    pub lpVtbl: *const IPropertyBagVtbl,
}
#[repr(C)]
pub struct IPropertyBagVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPropertyBag, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPropertyBag) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPropertyBag) -> ULONG,
    pub Read: unsafe extern "system" fn(This: *mut IPropertyBag, pszPropName: LPCOLESTR, pVar: *mut VARIANT, pErrorLog: *mut IErrorLog) -> HRESULT,
    pub Write: unsafe extern "system" fn(This: *mut IPropertyBag, pszPropName: LPCOLESTR, pVar: *mut VARIANT) -> HRESULT,
}
extern "C" {
    pub static IID_IPropertyBag: IID;
}

// Deterministic from shldisp.h
#[repr(C)]
pub struct IShellFolderViewDual {
    pub lpVtbl: *const IShellFolderViewDualVtbl,
}
#[repr(C)]
pub struct IShellFolderViewDualVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellFolderViewDual, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolderViewDual) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolderViewDual) -> ULONG,
    pub GetTypeInfoCount: unsafe extern "system" fn(This: *mut IShellFolderViewDual, pctinfo: *mut UINT) -> HRESULT,
    pub GetTypeInfo: unsafe extern "system" fn(This: *mut IShellFolderViewDual, iTInfo: UINT, lcid: LCID, ppTInfo: *mut *mut ITypeInfo) -> HRESULT,
    pub GetIDsOfNames: unsafe extern "system" fn(This: *mut IShellFolderViewDual, riid: REFIID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT,
    pub Invoke: unsafe extern "system" fn(This: *mut IShellFolderViewDual, dispIdMember: DISPID, riid: REFIID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT,
    pub get_Application: unsafe extern "system" fn(This: *mut IShellFolderViewDual, app: *mut *mut IDispatch) -> HRESULT,
    pub get_Parent: unsafe extern "system" fn(This: *mut IShellFolderViewDual, parent: *mut *mut IDispatch) -> HRESULT,
    pub get_Folder: unsafe extern "system" fn(This: *mut IShellFolderViewDual, folder: *mut *mut Folder) -> HRESULT,
    pub SelectedItems: unsafe extern "system" fn(This: *mut IShellFolderViewDual, items: *mut *mut FolderItems) -> HRESULT,
    pub get_FocusedItem: unsafe extern "system" fn(This: *mut IShellFolderViewDual, item: *mut *mut FolderItem) -> HRESULT,
    pub SelectItem: unsafe extern "system" fn(This: *mut IShellFolderViewDual, item: *mut VARIANT, flags: i32) -> HRESULT,
    pub PopupItemMenu: unsafe extern "system" fn(This: *mut IShellFolderViewDual, item: *mut FolderItem, vx: VARIANT, vy: VARIANT, command: *mut BSTR) -> HRESULT,
    pub get_Script: unsafe extern "system" fn(This: *mut IShellFolderViewDual, script: *mut *mut IDispatch) -> HRESULT,
    pub get_ViewOptions: unsafe extern "system" fn(This: *mut IShellFolderViewDual, options: *mut LONG) -> HRESULT,
}
extern "C" {
    pub static IID_IShellFolderViewDual: IID;
}



macro_rules! ASSERT { ($($arg:tt)*) => {{}}; }



macro_rules! LOWORD { ($value:expr) => { (((($value as usize) & 0xffff) as UINT) as u16) }; }

pub type LPFNSVADDPROPSHEETPAGE = Option<unsafe extern "system" fn(HPROPSHEETPAGE, LPARAM) -> BOOL>;


pub const DROPEFFECT_NONE: DWORD = 0;
pub const DROPEFFECT_MOVE: DWORD = 2;

extern "system" {
    fn CreateMenu() -> HMENU;
    fn DestroyMenu(menu: HMENU) -> BOOL;
    fn InsertMenuW(menu: HMENU, position: UINT, flags: UINT, id: UINT_PTR, text: LPCWSTR) -> BOOL;
    fn DestroyWindow(window: HWND) -> BOOL;
    fn LoadIconW(instance: HINSTANCE, name: LPCWSTR) -> HICON;
    fn GetMenuItemInfoW(menu: HMENU, item: UINT, by_position: BOOL, info: *mut MENUITEMINFOW) -> BOOL;
    fn IUnknown_QueryService(object: *mut IUnknown, service: REFGUID, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT;
    fn RegCloseKey(key: HKEY) -> LONG;
    fn RegOpenKeyExW(key: HKEY, subkey: LPCWSTR, options: DWORD, access: REGSAM, result: PHKEY) -> LONG;
    fn RegQueryValueExW(key: HKEY, value: LPCWSTR, reserved: LPDWORD, kind: LPDWORD, data: PBYTE, bytes: LPDWORD) -> LONG;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn InsertMenuItemW(menu: HMENU, item: UINT, by_position: BOOL, info: *const MENUITEMINFOW) -> BOOL;
}

extern "C" {
    fn _ILIsSpecialFolder(pidl: LPCITEMIDLIST) -> BOOL;
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

extern "system" {
    #[link_name = "SHFree"] fn cpp_com_SHFree(memory: LPVOID);
}
#[inline]
unsafe fn SHFree<T>(memory: *mut T) { cpp_com_SHFree(memory.cast()); }

#[repr(transparent)]
pub struct CComHeapPtr<T> { ptr: *mut T }
impl<T> CComHeapPtr<T> {
    pub const fn empty() -> Self { Self { ptr: core::ptr::null_mut() } }
    pub const fn new(ptr: *mut T) -> Self { Self { ptr } }
    pub const fn as_raw(&self) -> *mut T { self.ptr }
    pub fn as_mut_slot(&mut self) -> *mut *mut T { core::ptr::addr_of_mut!(self.ptr) }
    pub unsafe fn Attach(&mut self, ptr: *mut T) {
        if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); }
        self.ptr = ptr;
    }
    pub fn Detach(&mut self) -> *mut T {
        let ptr = self.ptr; self.ptr = core::ptr::null_mut(); ptr
    }
}
impl<T> Drop for CComHeapPtr<T> {
    fn drop(&mut self) {
        unsafe { if !self.ptr.is_null() { CoTaskMemFree(self.ptr.cast()); } }
    }
}

extern "C" {
    static CLSID_RecycleBin: GUID;
    static IID_CDefView: GUID;
    static IID_IContextMenu: GUID;
    static IID_IPersistFolder2: GUID;
    static IID_IShellDetails: GUID;
    static IID_IShellFolder2: GUID;
}

#[inline]
unsafe fn CopyMemory<D, S, N: CppComSize>(
    dst: *mut D, src: *const S, bytes: N) {
    let _ = memcpy(dst, src, bytes);
}
#[inline]
unsafe fn ZeroMemory<T, N: CppComSize>(dst: *mut T, bytes: N) {
    let _ = memset(dst.cast(), 0, bytes.cpp_com_size());
}

pub struct CppComPtrGuard<T> { slot: *mut *mut T }
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self { Self { slot } }
}
impl<T> Drop for CppComPtrGuard<T> {
    fn drop(&mut self) {
        unsafe {
            let value = *self.slot;
            if !value.is_null() {
                let unknown = value as *mut IUnknown;
                ((*(*unknown).lpVtbl).Release)(unknown);
                *self.slot = core::ptr::null_mut();
            }
        }
    }
}

extern "C" {
    static mut shell32_hInstance: HINSTANCE;
}

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_22C79B88393E: [WCHAR; 9] = [0x004D, 0x0045, 0x004E, 0x0055, 0x005F, 0x0030, 0x0030, 0x0031, 0x0000];
pub static CPP_COM_WSTR_4FA92E643652: [WCHAR; 9] = [0x004D, 0x0045, 0x004E, 0x0055, 0x005F, 0x0030, 0x0030, 0x0033, 0x0000];
pub static CPP_COM_WSTR_7E65B4F82DCB: [WCHAR; 8] = [0x0053, 0x006F, 0x0072, 0x0074, 0x0044, 0x0069, 0x0072, 0x0000];
pub static CPP_COM_WSTR_8888EF3C3D8B: [WCHAR; 60] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0041, 0x0064, 0x0076, 0x0061, 0x006E, 0x0063, 0x0065, 0x0064, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_98BD7125CFE5: [WCHAR; 8] = [0x0043, 0x006F, 0x006C, 0x0049, 0x006E, 0x0066, 0x006F, 0x0000];
pub static CPP_COM_WSTR_9F585864DA1D: [WCHAR; 15] = [0x005C, 0x0049, 0x0063, 0x006F, 0x006E, 0x0055, 0x006E, 0x0064, 0x0065, 0x0072, 0x006C, 0x0069, 0x006E, 0x0065, 0x0000];
pub static CPP_COM_WSTR_A7D0FE10F8A8: [WCHAR; 16] = [0x0053, 0x0074, 0x0072, 0x0065, 0x0061, 0x006D, 0x0073, 0x005C, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0000];
pub static CPP_COM_WSTR_A95A961DFF0A: [WCHAR; 44] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x004E, 0x006F, 0x0052, 0x006F, 0x0061, 0x006D, 0x005C, 0x0042, 0x0061, 0x0067, 0x0073, 0x0000];
pub static CPP_COM_WSTR_B1320DB9F2E8: [WCHAR; 17] = [0x0053, 0x0048, 0x0045, 0x004C, 0x004C, 0x0044, 0x004C, 0x004C, 0x005F, 0x0044, 0x0065, 0x0066, 0x0056, 0x0069, 0x0065, 0x0077, 0x0000];
pub static CPP_COM_WSTR_BA2342D760A2: [WCHAR; 6] = [0x0025, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_C47C37CF2996: [WCHAR; 5] = [0x004D, 0x006F, 0x0064, 0x0065, 0x0000];
pub static CPP_COM_WSTR_C6AD24AE112E: [WCHAR; 5] = [0x0053, 0x006F, 0x0072, 0x0074, 0x0000];
pub static CPP_COM_WSTR_C7C306713E36: [WCHAR; 15] = [0x004C, 0x0069, 0x0073, 0x0074, 0x0076, 0x0069, 0x0065, 0x0077, 0x0053, 0x0068, 0x0061, 0x0064, 0x006F, 0x0077, 0x0000];
pub static CPP_COM_WSTR_CA7C6DD01581: [WCHAR; 11] = [0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0056, 0x0069, 0x0065, 0x0077, 0x0000];
pub static CPP_COM_WSTR_EC73128D2969: [WCHAR; 7] = [0x0046, 0x0046, 0x006C, 0x0061, 0x0067, 0x0073, 0x0000];
pub static CPP_COM_WSTR_EE1C560F63B6: [WCHAR; 18] = [0x0053, 0x0065, 0x006C, 0x0065, 0x0063, 0x0074, 0x0045, 0x0078, 0x0074, 0x004F, 0x006E, 0x0052, 0x0065, 0x006E, 0x0061, 0x006D, 0x0065, 0x0000];

// --local-raii MenuCleanup: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
// --local-raii MenuCleanup: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
//
// Hand-verified Rust lowering of the file-local C++ RAII guard `MenuCleanup`
// (dll/win32/shell32/CDefView.cpp:131). Consumed by
//   rustconv translate --cpp-com --class CDefView \
//     --local-raii MenuCleanup=tools/rustconv/data/local_raii/MenuCleanup.rs
//
// C++ semantics preserved exactly:
//   ctor  — binds REFERENCES to the caller's two variables; copies nothing
//   dtor  — if (m_pCM)   { IUnknown_SetSite(m_pCM, NULL); m_pCM.Release(); }
//           if (m_hMenu) { DestroyMenu(m_hMenu); m_hMenu = NULL; }
//
// Both members are C++ REFERENCES (`CComPtr<IContextMenu>&`, `HMENU&`) and that is the entire
// point of the guard: it clears the CALLER's variables, not copies of them. All four call sites
// depend on it, and two bind CDefView's own fields — `MenuCleanup _(m_pCM, m_hContextMenu)` at
// :2308 and `MenuCleanup _(m_pFileMenu, Dummy)` at :2646.
//
// So this holds RAW POINTERS rather than `&mut`. A Rust `&mut` into a field of `*mut CDefView`
// would claim exclusive access to storage the surrounding worker keeps reaching through `this`
// for the whole guarded scope, which is exactly the aliasing the C++ does freely. The null
// checks on the pointers themselves cannot fire from correct code — a C++ reference is never
// null — and exist only because lowering a reference to a raw pointer removes the guarantee.
//
// Both slots end NULL and that is load-bearing, not tidiness: `CComPtr::Release()` nulls `p`
// the way ATL's does (atl.rs:314) and the C++ assigns `m_hMenu = NULL` explicitly, so a caller
// that re-tests either variable after the guarded scope sees the cleared value.
// The guarded slot is a FLATTENED CComPtr: every `CComPtr<T>` member of this class is a bare
// `*mut T` (see shell32/mod.rs on why the scaffold flattens them), so the guard holds a
// pointer-to-pointer, not a pointer-to-CComPtr. `CComPtr::Release()` is therefore spelled out:
// release through the vtable, then null the slot, which is what atl.rs:314 does.
struct MenuCleanup {
    m_pCM: *mut *mut IContextMenu,
    m_hMenu: *mut HMENU,
}

impl MenuCleanup {
    unsafe fn new(pCM: *mut *mut IContextMenu, menu: *mut HMENU) -> MenuCleanup {
        MenuCleanup {
            m_pCM: pCM,
            m_hMenu: menu,
        }
    }
}

impl Drop for MenuCleanup {
    fn drop(&mut self) {
        unsafe {
            if !self.m_pCM.is_null() && !(*self.m_pCM).is_null() {
                IUnknown_SetSite((*self.m_pCM).cast(), core::ptr::null_mut());
                ((*(*(*self.m_pCM)).lpVtbl).Release)(*self.m_pCM);
                *self.m_pCM = core::ptr::null_mut();
            }
            if !self.m_hMenu.is_null() && !(*self.m_hMenu).is_null() {
                DestroyMenu(*self.m_hMenu);
                *self.m_hMenu = core::ptr::null_mut();
            }
        }
    }
}

use core::ffi::c_void;

pub unsafe fn GetContextMenuFlags_body(pSB: *mut IShellBrowser, sfgao: SFGAOF) -> UINT {
    let mut cmf: UINT = CMF_NORMAL;
    if GetKeyState(((VK_SHIFT) as i32)) < 0 {
        cmf |= CMF_EXTENDEDVERBS;
    }
    if sfgao & SFGAO_CANRENAME != 0 {
        cmf |= CMF_CANRENAME;
    }
    let mut hwnd = core::ptr::null_mut();
    if !pSB.is_null()
        && (((*(*pSB).lpVtbl).GetControlWindow)(
            pSB,
            FCW_TREE,
            &mut hwnd,
        )) >= 0
        && !hwnd.is_null()
    {
        cmf |= CMF_EXPLORE;
    }
    return cmf;
}

pub unsafe fn ClientToListView_body(hwndLV: HWND, ppt: *mut POINT) -> () {
    let mut Origin = POINT { x: 0, y: 0 };

    // FIXME: LVM_GETORIGIN is broken. See CORE-17266
    if ListView_GetOrigin(hwndLV, &mut Origin) == 0 {
        return;
    }

    (*ppt).x += Origin.x;
    (*ppt).y += Origin.y;
}

pub unsafe fn AppendMenuItem_body(hMenu: HMENU, MF: UINT, Id: UINT, String: LPCWSTR, Data: SIZE_T) -> BOOL {
    let _ = MF;
    let mut mii: MENUITEMINFOW = unsafe { core::mem::zeroed() };
    mii.cbSize = core::mem::size_of_val(&mii) as u32;
    mii.fMask = MIIM_TYPE | MIIM_ID | MIIM_DATA | MIIM_STATE;
    mii.fState = MF & (MFS_CHECKED | MFS_DISABLED);
    mii.fType = MF & !mii.fState;
    mii.wID = Id;
    mii.dwTypeData = String as *mut u16;
    mii.dwItemData = Data as ULONG_PTR;
    unsafe { InsertMenuItemW(hMenu, (-(1 as i32)) as u32, TRUE, &mii) }
}

pub unsafe fn GetMenuItemDataById_body(hMenu: HMENU, Id: UINT) -> SIZE_T {
    let mut mii: MENUITEMINFOW = unsafe { ::core::mem::zeroed() };
    mii.cbSize = FIELD_OFFSET!(MENUITEMINFOW, hbmpItem) as _;
    mii.fMask = MIIM_DATA;
    if unsafe { GetMenuItemInfoW(hMenu, Id, FALSE, &mut mii) } != 0 {
        return mii.dwItemData as SIZE_T;
    } else {
        return 0;
    }
}

pub unsafe fn GetSubmenuByID_body(hmenu: HMENU, id: UINT) -> HMENU {
    let mut mii: MENUITEMINFOW = unsafe { core::mem::zeroed() };
    mii.cbSize = core::mem::size_of_val(&mii) as UINT;
    mii.fMask = MIIM_SUBMENU;
    if unsafe {
        GetMenuItemInfoW(hmenu, id, FALSE, core::ptr::addr_of_mut!(mii))
    } != 0
    {
        return ((mii.hSubMenu) as _);
    }
    core::ptr::null_mut()
}

pub unsafe fn ReallyGetMenuItemID_body(hmenu: HMENU, i: i32) -> UINT {
    let mut mii: MENUITEMINFOW = unsafe { core::mem::zeroed() };
    mii.cbSize = core::mem::size_of_val(&mii) as _;
    mii.fMask = MIIM_ID;
    if unsafe { GetMenuItemInfoW(hmenu, ((i) as u32), TRUE, &mut mii as *mut MENUITEMINFOW) } != 0 {
        return mii.wID;
    }
    UINT_MAX
}

pub unsafe fn CalculateCharWidth_body(hwnd: HWND) -> UINT {
    unsafe {
        let mut ret: UINT = 0;
        let hDC = GetDC(hwnd);
        if !hDC.is_null() {
            let hOrg = SelectObject(
                hDC,
                SendMessage(
                    hwnd,
                    WM_GETFONT,
                    0,
                    0,
                ) as HGDIOBJ,
            );
            ret = ((GdiGetCharDimensions(hDC, core::ptr::null_mut(), core::ptr::null_mut())) as u32);
            SelectObject(hDC, hOrg);
            ReleaseDC(hwnd, hDC);
        }
        ret
    }
}

pub unsafe fn GetViewColor_body(Clr: COLORREF, SysFallback: UINT) -> COLORREF {
    if Clr != CLR_INVALID {
        Clr
    } else {
        GetSysColor(((SysFallback) as i32))
    }
}

pub unsafe fn CDefView_IsDesktop_body(this: *mut CDefView) -> BOOL {
    ((*this).m_FolderSettings.fFlags & FWF_DESKTOP) as BOOL
}

pub unsafe fn CDefView_IsSpecialFolder_body(this: *mut CDefView, csidl: &mut i32) -> BOOL {
    *csidl = (((*this).m_SpecialFolder) as i32);
    ((*this).m_SpecialFolder >= 0) as BOOL
}

pub unsafe fn CDefView_GetCommDlgViewFlags_body(this: *mut CDefView) -> DWORD {
    let mut pcdb2: *mut ICommDlgBrowser2 = core::ptr::null_mut();
    let _pcdb2_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pcdb2));
    if !((*this).m_pCommDlgBrowser).is_null()
        && ((*(*(*this).m_pCommDlgBrowser).lpVtbl).QueryInterface)(
            (*this).m_pCommDlgBrowser,
            core::ptr::addr_of!(IID_ICommDlgBrowser2),
            core::ptr::addr_of_mut!(pcdb2).cast(),
        ) >= 0
    {
        let mut flags: DWORD = 0;
        if ((*(*pcdb2).lpVtbl).GetViewFlags)(pcdb2, &mut flags) >= 0 {
            ((*(*pcdb2).lpVtbl).Release)(pcdb2);
            pcdb2 = core::ptr::null_mut();
            return flags;
        }
        ((*(*pcdb2).lpVtbl).Release)(pcdb2);
        pcdb2 = core::ptr::null_mut();
    }
    return 0;
}

pub unsafe fn CDefView_IsSupportedFolderViewMode_body(this: *mut CDefView, Mode: i32) -> bool {
    Mode >= FVM_FIRST && Mode <= ((FVM_DETAILS) as i32)
}

pub unsafe fn CDefView_GetWndClassInfo_body(this: *mut CDefView) -> &'static mut ATL_CWndClassInfo {
    static mut wc: core::mem::MaybeUninit<ATL_CWndClassInfo> = core::mem::MaybeUninit::uninit();
    static mut initialized: bool = false;

    unsafe {
        if !initialized {
            let value: ATL_CWndClassInfo = ATL_CWndClassInfo {
                m_wc: WNDCLASSEX {
                    cbSize: core::mem::size_of::<WNDCLASSEX>() as u32,
                    style: CS_PARENTDC,
                    lpfnWndProc: Some(AtlStartWindowProc::<CDefView>),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: core::ptr::null_mut(),
                    hIcon: core::ptr::null_mut(),
                    hCursor: LoadCursorW(core::ptr::null_mut(), IDC_ARROW),
                    hbrBackground: core::ptr::null_mut(),
                    lpszMenuName: core::ptr::null(),
                    lpszClassName: ((CPP_COM_WSTR_B1320DB9F2E8.as_ptr()) as _),
                    hIconSm: core::ptr::null_mut(),
                },
                m_lpszOrigName: core::ptr::null(),
                pWndProc: None,
                m_lpszCursorID: ((IDC_ARROW) as _),
                m_bSystemCursor: TRUE,
                m_atom: 0,
                // ATL's own scratch buffer; the island sizes it per target, so zero it
                // rather than spelling a length that is right on only one arch.
                m_szAutoName: core::mem::zeroed(),
            };
            core::ptr::write(wc.as_mut_ptr(), value);
            initialized = true;
        }

        let result: &mut ATL_CWndClassInfo = &mut *wc.as_mut_ptr();
        result
    }
}

pub unsafe fn CDefView_GetWindowProc_body(this: *mut CDefView) -> WNDPROC {
    let _ = this;
    // `return WindowProc;` names the STATIC member below, not a data member -- there is no
    // `this` to read it out of.
    Some(CDefView_WindowProc)
}

/// CDefView.cpp:1013 `CDefView::WindowProc` — holds a reference across the dispatch, because a
/// handler can Release the last one and ATL's base would then touch a freed object.
///
/// `let pThis = hWnd as *mut CDefView` is what the C++ says, and it was FAITHFUL: ATL's thunk
/// rewrites the first argument from the HWND to `this`. Under `crate::atlwin` the first
/// argument really is the HWND, so the same contract is spelled as the USERDATA lookup.
pub unsafe extern "system" fn CDefView_WindowProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let pThis = GetWindowLongPtr(hWnd.cast(), GWLP_USERDATA as INT) as *mut CDefView;
    if pThis.is_null() {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }
    let _ = CDefView_AddRef(pThis);
    let result = AtlWindowProc::<CDefView>(hWnd, uMsg, wParam, lParam);
    let _ = CDefView_Release(pThis);
    result
}

unsafe impl AtlWindow for CDefView {
    #[inline]
    unsafe fn atl_hwnd(this: *mut Self) -> *mut HWND {
        core::ptr::addr_of_mut!((*this).m_hWnd)
    }
    #[inline]
    unsafe fn atl_state(this: *mut Self) -> *mut AtlWindowState {
        core::ptr::addr_of_mut!((*this).m_atl)
    }
    #[inline]
    unsafe fn atl_process_window_message(
        this: *mut Self,
        hWnd: HWND,
        uMsg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
        lResult: *mut LRESULT,
        dwMsgMapID: DWORD,
    ) -> BOOL {
        CDefView_ProcessWindowMessage_body(this, hWnd, uMsg, wParam, lParam, lResult, dwMsgMapID)
    }
    /// CDefView.cpp:535 `virtual WNDPROC GetWindowProc() override { return WindowProc; }`.
    /// This is an OVERRIDE and dropping it is a use-after-free, not a style difference — see
    /// the trait method's own comment and `CDefView_WindowProc` below.
    #[inline]
    unsafe fn atl_window_proc() -> WNDPROC {
        Some(CDefView_WindowProc)
    }
}

/// CDefView.cpp:433 `BEGIN_MSG_MAP(CDefView)` — all 25 entries, in source order.
///
/// ATL's `MESSAGE_HANDLER(msg, f)` expands to: match the message, set `bHandled = TRUE`, call
/// the handler, and return TRUE only if the handler LEFT it true — a handler that clears
/// `bHandled` falls through to the next entry and ultimately to DefWindowProc. That
/// three-state behaviour is the whole reason the `&mut BOOL` is in every handler's signature,
/// and it is why this cannot be a plain `match`: two entries (WM_DRAWITEM and WM_MEASUREITEM)
/// share one handler, and any of them may decline after inspecting the message.
pub unsafe fn CDefView_ProcessWindowMessage_body(
    this: *mut CDefView,
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    lResult: *mut LRESULT,
    dwMsgMapID: DWORD,
) -> BOOL {
    let _ = hWnd;
    if dwMsgMapID != 0 {
        return FALSE;
    }
    macro_rules! MESSAGE_HANDLER {
        ($msg:expr, $handler:ident) => {
            if uMsg == $msg as UINT {
                let mut bHandled: BOOL = TRUE;
                *lResult = $handler(this, uMsg, wParam, lParam, &mut bHandled);
                if bHandled != FALSE {
                    return TRUE;
                }
            }
        };
    }
    MESSAGE_HANDLER!(WM_SIZE, CDefView_OnSize_body);
    MESSAGE_HANDLER!(WM_SETFOCUS, CDefView_OnSetFocus_body);
    MESSAGE_HANDLER!(WM_KILLFOCUS, CDefView_OnKillFocus_body);
    MESSAGE_HANDLER!(WM_NCCREATE, CDefView_OnNCCreate_body);
    MESSAGE_HANDLER!(WM_CREATE, CDefView_OnCreate_body);
    MESSAGE_HANDLER!(WM_ACTIVATE, CDefView_OnActivate_body);
    MESSAGE_HANDLER!(WM_NOTIFY, CDefView_OnNotify_body);
    MESSAGE_HANDLER!(WM_COMMAND, CDefView_OnCommand_body);
    MESSAGE_HANDLER!(SHV_CHANGE_NOTIFY, CDefView_OnChangeNotify_body);
    MESSAGE_HANDLER!(SHV_UPDATESTATUSBAR, CDefView_OnUpdateStatusbar_body);
    MESSAGE_HANDLER!(WM_CONTEXTMENU, CDefView_OnContextMenu_body);
    MESSAGE_HANDLER!(WM_DRAWITEM, CDefView_OnMenuMessage_body);
    MESSAGE_HANDLER!(WM_MEASUREITEM, CDefView_OnMenuMessage_body);
    MESSAGE_HANDLER!(WM_SHOWWINDOW, CDefView_OnShowWindow_body);
    MESSAGE_HANDLER!(WM_GETDLGCODE, CDefView_OnGetDlgCode_body);
    MESSAGE_HANDLER!(WM_DESTROY, CDefView_OnDestroy_body);
    MESSAGE_HANDLER!(WM_ERASEBKGND, CDefView_OnEraseBackground_body);
    MESSAGE_HANDLER!(WM_PRINTCLIENT, CDefView_OnPrintClient_body);
    MESSAGE_HANDLER!(WM_SYSCOLORCHANGE, CDefView_OnSysColorChange_body);
    MESSAGE_HANDLER!(CWM_GETISHELLBROWSER, CDefView_OnGetShellBrowser_body);
    MESSAGE_HANDLER!(WM_SETTINGCHANGE, CDefView_OnSettingChange_body);
    MESSAGE_HANDLER!(WM_INITMENUPOPUP, CDefView_OnInitMenuPopup_body);
    MESSAGE_HANDLER!(WM_CHANGECBCHAIN, CDefView_OnChangeCBChain_body);
    MESSAGE_HANDLER!(WM_DRAWCLIPBOARD, CDefView_OnDrawClipboard_body);
    FALSE
}

pub unsafe fn CDefView_ctor_body(this: *mut CDefView) -> () {
    (*this).m_ListView = core::mem::zeroed();
    (*this).m_hWndParent = ((NULL) as _);
    (*this).m_hMenu = ((NULL) as _);
    (*this).m_hMenuArrangeModes = ((NULL) as _);
    (*this).m_hMenuViewModes = ((NULL) as _);
    (*this).m_hContextMenu = ((NULL) as _);
    (*this).m_bmenuBarInitialized = FALSE;
    (*this).m_uState = 0;
    (*this).m_cidl = 0;
    (*this).m_apidl = ((NULL) as _);
    (*this).m_pidlParent = ((NULL) as _);
    (*this).m_LoadColumnsList = ((NULL) as _);
    (*this).m_hNotify = 0;
    (*this).m_hAccel = ((NULL) as _);
    (*this).m_dwAspects = 0;
    (*this).m_dwAdvf = 0;
    (*this).m_iDragOverItem = 0;
    (*this).m_cScrollDelay = 0;
    (*this).m_isEditing = FALSE;
    (*this).m_SpecialFolder = ((UINT::MAX) as i8);
    (*this).m_isFullStatusBar = true;
    (*this).m_ScheduledStatusbarUpdate = false;
    (*this).m_HasCutItems = false;
    (*this).m_Destroyed = FALSE;

    ZeroMemory(
        core::ptr::addr_of_mut!((*this).m_FolderSettings) as *mut _,
        core::mem::size_of_val(&(*this).m_FolderSettings)
    );
    ZeroMemory(
        core::ptr::addr_of_mut!((*this).m_ptLastMousePos) as *mut _,
        core::mem::size_of_val(&(*this).m_ptLastMousePos)
    );
    ZeroMemory(
        core::ptr::addr_of_mut!((*this).m_Category) as *mut _,
        core::mem::size_of_val(&(*this).m_Category)
    );

    (*this).m_viewinfo_data.clrText = CLR_INVALID;
    (*this).m_viewinfo_data.clrTextBack = CLR_INVALID;
    (*this).m_viewinfo_data.hbmBack = ((NULL) as _);

    ((*this).m_sortInfo).Reset();

    (*this).m_ListToFolderColMap = DPA_Create(0);
    (*this).m_hMyComputerIcon = LoadIconW(
        shell32_hInstance,
        MAKEINTRESOURCEW(IDI_SHELL_COMPUTER_DESKTOP)
    );
}

pub unsafe fn CDefView_dtor_body(this: *mut CDefView) -> () {
    CDefView__DoFolderViewCB_body(this, SFVM_VIEWRELEASE, 0, 0);

    if (*this).m_viewinfo_data.hbmBack != 0 as _ {
        DeleteObject((((*this).m_viewinfo_data.hbmBack) as _));
        (*this).m_viewinfo_data.hbmBack = 0 as _;
    }

    if !(*this).m_hWnd.is_null() as _ {
        CDefView_DestroyViewWindow_body(this);
    }

    SHFree(((*this).m_apidl) as LPVOID);
    DPA_Destroy((*this).m_LoadColumnsList);
    DPA_Destroy((*this).m_ListToFolderColMap);
}

pub unsafe fn CDefView_Initialize_body(this: *mut CDefView, shellFolder: *mut IShellFolder) -> HRESULT {
    unsafe {
        CDefView_cpp_com_assign_m_pSFParent(this, shellFolder);
        ((*(*shellFolder).lpVtbl).QueryInterface)(shellFolder, ((core::ptr::addr_of!(IID_IShellFolder2)) as _), core::ptr::addr_of_mut!((*this).m_pSF2Parent).cast());
        ((*(*shellFolder).lpVtbl).QueryInterface)(shellFolder, ((core::ptr::addr_of!(IID_IShellDetails)) as _), core::ptr::addr_of_mut!((*this).m_pSDParent).cast());

        return S_OK;
    }
}

pub unsafe fn CDefView_IncludeObject_body(this: *mut CDefView, pidl: PCUITEMID_CHILD) -> HRESULT {
    let mut ret = S_OK;
    if !((*this).m_pCommDlgBrowser.is_null()) && (CDefView_GetCommDlgViewFlags_body(this) & CDB2GVF_NOINCLUDEITEM) == 0 {
        ret = ((*(*((*this).m_pCommDlgBrowser)).lpVtbl).IncludeObject)((*this).m_pCommDlgBrowser, this as *mut _, pidl);
    } else if !((*this).m_pFolderFilter.is_null()) {
        ret = ((*(*((*this).m_pFolderFilter)).lpVtbl).ShouldShow)((*this).m_pFolderFilter, (*this).m_pSFParent, (*this).m_pidlParent, pidl);
    }
    ret
}

pub unsafe fn CDefView_OnDefaultCommand_body(this: *mut CDefView) -> HRESULT {
    let mut ret: HRESULT = S_FALSE;

    let browser = (*this).m_pCommDlgBrowser;
    if !browser.is_null() {
        ret = ((*(*browser).lpVtbl).OnDefaultCommand)(browser, this.cast());
    }

    ret
}

pub unsafe fn CDefView_OnStateChange_body(this: *mut CDefView, uFlags: UINT) -> HRESULT {
    let mut ret: HRESULT = S_FALSE;

    let pCommDlgBrowser = (*this).m_pCommDlgBrowser;
    if !pCommDlgBrowser.is_null() {
        ret = ((*(*pCommDlgBrowser).lpVtbl).OnStateChange)(pCommDlgBrowser, this as _, uFlags);
    }

    ret
}

pub unsafe fn CDefView_CheckToolbar_body(this: *mut CDefView) -> () {
    unsafe {
        let mut result: LRESULT = 0;

        if !((*this).m_pCommDlgBrowser).is_null() {
            ((*(*(*this).m_pShellBrowser).lpVtbl).SendControlMsg)(
                (*this).m_pShellBrowser,
                FCW_TOOLBAR,
                TB_CHECKBUTTON,
                FCIDM_TB_SMALLICON as WPARAM,
                (if (*this).m_FolderSettings.ViewMode == FVM_LIST {
                    TRUE
                } else {
                    FALSE
                }) as LPARAM,
                &mut result,
            );
            ((*(*(*this).m_pShellBrowser).lpVtbl).SendControlMsg)(
                (*this).m_pShellBrowser,
                FCW_TOOLBAR,
                TB_CHECKBUTTON,
                FCIDM_TB_REPORTVIEW as WPARAM,
                (if (*this).m_FolderSettings.ViewMode == FVM_DETAILS {
                    TRUE
                } else {
                    FALSE
                }) as LPARAM,
                &mut result,
            );
            ((*(*(*this).m_pShellBrowser).lpVtbl).SendControlMsg)(
                (*this).m_pShellBrowser,
                FCW_TOOLBAR,
                TB_ENABLEBUTTON,
                FCIDM_TB_SMALLICON as WPARAM,
                TRUE as LPARAM,
                &mut result,
            );
            ((*(*(*this).m_pShellBrowser).lpVtbl).SendControlMsg)(
                (*this).m_pShellBrowser,
                FCW_TOOLBAR,
                TB_ENABLEBUTTON,
                FCIDM_TB_REPORTVIEW as WPARAM,
                TRUE as LPARAM,
                &mut result,
            );
        }
    }
}

pub unsafe fn CDefView_UpdateStatusbar_body(this: *mut CDefView) -> () {
    let mut szFormat = [0u16; ((MAX_PATH) as usize)];
    let mut szPartText = [0u16; ((MAX_PATH) as usize)];

    if (*this).m_ListView.is_null() {
        return;
    }

    let cSelectedItems = (*this).m_ListView.GetSelectedCount();
    if cSelectedItems != 0 {
        LoadStringW(shell32_hInstance, IDS_OBJECTS_SELECTED, szFormat.as_mut_ptr(), szFormat.len() as INT);
        StringCchPrintfW(szPartText.as_mut_ptr(), szPartText.len(), szFormat.as_ptr(), cSelectedItems);
    } else {
        LoadStringW(shell32_hInstance, IDS_OBJECTS, szFormat.as_mut_ptr(), szFormat.len() as INT);
        StringCchPrintfW(szPartText.as_mut_ptr(), szPartText.len(), szFormat.as_ptr(), (*this).m_ListView.GetItemCount());
    }

    let mut lResult: LRESULT = 0;
    ((*(*(*this).m_pShellBrowser).lpVtbl).SendControlMsg)(
        (*this).m_pShellBrowser,
        FCW_STATUS,
        SB_SETTEXT,
        0,
        szPartText.as_mut_ptr() as LPARAM,
        &mut lResult as *mut _,
    );

    if (*this).m_isFullStatusBar {
        let mut uTotalFileSize: u64 = 0;
        let mut uFileFlags: u16 = ((LVNI_ALL) as u16);
        let mut nItem: i32 = -1;
        let mut bIsOnlyFoldersSelected = true;

        if cSelectedItems != 0 {
            uFileFlags = ((LVNI_SELECTED) as u16);
        }

        while {
            nItem = (*this).m_ListView.GetNextItem(nItem, uFileFlags);
            nItem >= 0
        } {
            let pidl: PCUITEMID_CHILD = CDefView__PidlByItem_body(this, nItem);

            uTotalFileSize += ((_ILGetFileSize(((pidl) as _), core::ptr::null_mut(), 0)) as u64);

            if _ILIsFolder(pidl) == 0 {
                bIsOnlyFoldersSelected = false;
            }
        }

        if ((cSelectedItems != 0 && !bIsOnlyFoldersSelected) || uTotalFileSize != 0) {
            StrFormatByteSizeW(((uTotalFileSize) as i64), szPartText.as_mut_ptr(), ((szPartText.len()) as u32));
        } else {
            szPartText[0] = 0;
        }

        ((*(*(*this).m_pShellBrowser).lpVtbl).SendControlMsg)(
            (*this).m_pShellBrowser,
            FCW_STATUS,
            SB_SETTEXT,
            1,
            szPartText.as_mut_ptr() as LPARAM,
            &mut lResult as *mut _,
        );
    }

    let mut att: SFGAOF = 0;
    if cSelectedItems > 0 {
        let maxquery = 42;
        att = SFGAO_CANCOPY | SFGAO_CANMOVE;
        if cSelectedItems <= maxquery && (CDefView_GetSelections_body(this) == 0 || ((*(*(*this).m_pSFParent).lpVtbl).GetAttributesOf)(
                (*this).m_pSFParent,
                (*this).m_cidl,
                (((*this).m_apidl) as _),
                &mut att,
            ) < 0) {
            att = 0;
        }
    }

    ((*(*(*this).m_pShellBrowser).lpVtbl).SendControlMsg)(
        (*this).m_pShellBrowser,
        FCW_TOOLBAR,
        TB_ENABLEBUTTON,
        FCIDM_SHVIEW_COPYTO as WPARAM,
        if (att & SFGAO_CANCOPY) != 0 { 1 as LPARAM } else { 0 as LPARAM },
        &mut lResult,
    );

    ((*(*(*this).m_pShellBrowser).lpVtbl).SendControlMsg)(
        (*this).m_pShellBrowser,
        FCW_TOOLBAR,
        TB_ENABLEBUTTON,
        FCIDM_SHVIEW_MOVETO as WPARAM,
        if (att & SFGAO_CANMOVE) != 0 { 1 as LPARAM } else { 0 as LPARAM },
        &mut lResult,
    );
}

pub unsafe fn CDefView_UpdateStatusbarLocation_body(this: *mut CDefView) -> () {
    let mut lResult = 0;
    let mut pIcon: LPARAM = 0;
    let mut szPartText = [0u16; ((MAX_PATH) as usize)];
    szPartText[0] = 0;
    if (*this).m_isFullStatusBar {
        let mut csidl: i32 = 0;
        if CDefView_IsSpecialFolder_body(this, &mut csidl) == 0
            || (csidl != ((CSIDL_NETWORK) as i32) && csidl != CSIDL_BITBUCKET)
        {
            LoadStringW(
                shell32_hInstance,
                IDS_MYCOMPUTER,
                szPartText.as_mut_ptr(),
                szPartText.len() as INT,
            );
            pIcon = (*this).m_hMyComputerIcon as LPARAM;
        }
    }
    ((*(*(*this).m_pShellBrowser).lpVtbl).SendControlMsg)(
        (*this).m_pShellBrowser,
        FCW_STATUS,
        SB_SETICON,
        2,
        pIcon,
        &mut lResult,
    );
    ((*(*(*this).m_pShellBrowser).lpVtbl).SendControlMsg)(
        (*this).m_pShellBrowser,
        FCW_STATUS,
        SB_SETTEXT,
        2,
        szPartText.as_mut_ptr() as LPARAM,
        &mut lResult,
    );
}

pub unsafe fn CDefView_OnUpdateStatusbar_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    (*this).m_ScheduledStatusbarUpdate = false;
    CDefView_UpdateStatusbar_body(this);
    return 0;
}

pub unsafe fn CDefView_CreateList_body(this: *mut CDefView) -> BOOL {
    let mut dwStyle = WS_TABSTOP | WS_VISIBLE | WS_CHILDWINDOW | WS_CLIPSIBLINGS | WS_CLIPCHILDREN |
                      LVS_SHAREIMAGELISTS | LVS_EDITLABELS | LVS_AUTOARRANGE;
    let mut dwExStyle = if ((*this).m_FolderSettings.fFlags & FWF_NOCLIENTEDGE) != 0 {
        0
    } else {
        WS_EX_CLIENTEDGE
    };
    let mut ListExStyle = LVS_EX_INFOTIP | LVS_EX_LABELTIP;

    if ((*this).m_FolderSettings.fFlags & FWF_DESKTOP) != 0 {
        (*this).m_FolderSettings.fFlags |= FWF_NOCLIENTEDGE | FWF_NOSCROLL;
        dwStyle |= LVS_ALIGNLEFT;
    } else {
        dwStyle |= LVS_SHOWSELALWAYS;
        dwStyle |= if ((*this).m_FolderSettings.fFlags & FWF_ALIGNLEFT) != 0 {
            LVS_ALIGNLEFT
        } else {
            LVS_ALIGNTOP
        };
    }

    match (*this).m_FolderSettings.ViewMode {
        FVM_ICON => {
            dwStyle |= LVS_ICON;
        }
        FVM_DETAILS => {
            dwStyle |= LVS_REPORT;
        }
        FVM_SMALLICON => {
            dwStyle |= LVS_SMALLICON;
        }
        FVM_LIST => {
            dwStyle |= LVS_LIST;
        }
        _ => {
            dwStyle |= LVS_LIST;
        }
    }

    if ((*this).m_FolderSettings.fFlags & FWF_AUTOARRANGE) != 0 {
        dwStyle |= LVS_AUTOARRANGE;
    }
    if ((*this).m_FolderSettings.fFlags & FWF_SNAPTOGRID) != 0 {
        ListExStyle |= LVS_EX_SNAPTOGRID;
    }
    if ((*this).m_FolderSettings.fFlags & FWF_SINGLESEL) != 0 {
        dwStyle |= LVS_SINGLESEL;
    }
    if ((*this).m_FolderSettings.fFlags & FWF_FULLROWSELECT) != 0 {
        ListExStyle |= LVS_EX_FULLROWSELECT;
    }

    ListExStyle |= CDefView_GetItemActivateFlags_body(this);
    if (ListExStyle & LVS_EX_ONECLICKACTIVATE) != 0 {
        ListExStyle |= SHELL_GetIconUnderlineFlags();
    }

    if ((*this).m_FolderSettings.fFlags & FWF_NOCOLUMNHEADER) != 0 {
        dwStyle |= LVS_NOCOLUMNHEADER;
    }
    if ((*this).m_FolderSettings.fFlags & FWF_NOCLIENTEDGE) != 0 {
        dwExStyle &= !WS_EX_CLIENTEDGE;
    }

    let rcListView = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    (*this).m_ListView.Create(
        (*this).m_hWnd,
        core::ptr::addr_of!(rcListView),
        CPP_COM_WSTR_CA7C6DD01581.as_ptr(),
        dwStyle,
        dwExStyle,
        ID_LISTVIEW as usize as HMENU,
        core::ptr::null_mut(), // rosctrls.h:8 `LPVOID lpCreateParam = NULL`
    );

    if (*this).m_ListView.is_null() {
        return FALSE;
    }

    (*this).m_ListView.SetExtendedListViewStyle(ListExStyle, 0); // rosctrls.h:47 `dwMask = 0`

    return TRUE;
}

pub unsafe fn CDefView_UpdateListColors_body(this: *mut CDefView) -> () {
    if ((*this).m_FolderSettings.fFlags & FWF_DESKTOP) != 0 {
        let mut bDropShadow: i32 = FALSE;
        let mut cbDropShadow: u32 = core::mem::size_of_val(&bDropShadow) as u32;
        let list_view = core::ptr::addr_of_mut!((*this).m_ListView);

        CListView_SetBkColor_body(list_view, CLR_NONE);

        SHGetValueW(
            HKEY_CURRENT_USER,
            CPP_COM_WSTR_8888EF3C3D8B.as_ptr(),
            CPP_COM_WSTR_C7C306713E36.as_ptr(),
            core::ptr::null_mut(),
            &mut bDropShadow as *mut i32 as *mut core::ffi::c_void,
            &mut cbDropShadow,
        );

        if bDropShadow != 0 {
            CListView_SetTextBkColor_body(list_view, CLR_NONE);
            CListView_SetTextColor_body(
                list_view,
                RGB(255, 255, 255),
            );
            CListView_SetExtendedListViewStyle_body(
                list_view,
                LVS_EX_TRANSPARENTSHADOWTEXT,
                LVS_EX_TRANSPARENTSHADOWTEXT,
            );
        } else {
            let crDesktop: u32 = GetSysColor(((COLOR_DESKTOP) as i32));
            CListView_SetTextBkColor_body(list_view, crDesktop);
            if (GetRValue(crDesktop) as u32)
                + (GetGValue(crDesktop) as u32)
                + (GetBValue(crDesktop) as u32)
                > 128 * 3
            {
                CListView_SetTextColor_body(
                    list_view,
                    RGB(0, 0, 0),
                );
            } else {
                CListView_SetTextColor_body(
                    list_view,
                    RGB(255, 255, 255),
                );
            }
            CListView_SetExtendedListViewStyle_body(
                list_view,
                0,
                LVS_EX_TRANSPARENTSHADOWTEXT,
            );
        }
    } else {
        let list_view = core::ptr::addr_of_mut!((*this).m_ListView);
        CListView_SetTextBkColor_body(
            list_view,
            GetViewColor_body((*this).m_viewinfo_data.clrTextBack, COLOR_WINDOW),
        );
        CListView_SetTextColor_body(
            list_view,
            GetViewColor_body((*this).m_viewinfo_data.clrText, COLOR_WINDOWTEXT),
        );

        CListView_SetExtendedListViewStyle_body(
            list_view,
            LVS_EX_TRANSPARENTBKGND,
            LVS_EX_TRANSPARENTBKGND,
        );
    }
}

pub unsafe fn CDefView_InitList_body(this: *mut CDefView) -> BOOL {
    unsafe {
        let mut big_icons = ::core::mem::MaybeUninit::<HIMAGELIST>::uninit();
        let mut small_icons = ::core::mem::MaybeUninit::<HIMAGELIST>::uninit();

        ((*this).m_ListView).DeleteAllItems();

        Shell_GetImageLists(big_icons.as_mut_ptr(), small_icons.as_mut_ptr());

        let big_icons = big_icons.assume_init();
        let small_icons = small_icons.assume_init();

        ((*this).m_ListView).SetImageList(big_icons, ((LVSIL_NORMAL) as i32));
        ((*this).m_ListView).SetImageList(small_icons, ((LVSIL_SMALL) as i32));

        (*this).m_hMenuArrangeModes = CreateMenu();

        let pColumns: *mut SIZE_T = if !(*this).m_LoadColumnsList.is_null() {
            DPA_GetPtrPtr((*this).m_LoadColumnsList) as *mut SIZE_T
        } else {
            ::core::ptr::null_mut()
        };
        let ColumnCount: UINT = if !pColumns.is_null() {
            DPA_GetPtrCount((*this).m_LoadColumnsList) as UINT
        } else {
            0
        };
        CDefView_LoadColumns_body(this, pColumns, ColumnCount);
        if (*this).m_sortInfo.bColumnIsFolderColumn {
            (*this).m_sortInfo.bColumnIsFolderColumn = ((FALSE) != 0);
            let hr: HRESULT = CDefView_MapFolderColumnToListColumn_body(this, (*this).m_sortInfo.ListColumn as UINT);
            (*this).m_sortInfo.ListColumn = if hr >= 0 { hr as _ } else { 0 };
        }
        TRUE
    }
}

pub unsafe fn SHGetLVColumnSubItem_body(List: HWND, Col: UINT) -> HRESULT {
    let mut lvc = core::mem::MaybeUninit::<LVCOLUMN>::uninit();
    (*lvc.as_mut_ptr()).mask = LVCF_SUBITEM;
    if ListView_GetColumn(List, ((Col) as i32), lvc.as_mut_ptr()) == 0 {
        return E_FAIL;
    } else {
        return (*lvc.as_mut_ptr()).iSubItem as HRESULT;
    }
}

pub unsafe fn CDefView_MapFolderColumnToListColumn_body(this: *mut CDefView, FoldCol: UINT) -> HRESULT {
    let mut i: UINT = 0;
    loop {
        let r: HRESULT = SHGetLVColumnSubItem_body((*this).m_ListView.m_hWnd, i);
        if (r as UINT) == FoldCol {
            return i as HRESULT;
        } else if r < 0 {
            return r;
        }
        i = i.wrapping_add(1);
    }
}

pub unsafe fn CDefView_MapListColumnToFolderColumn_body(this: *mut CDefView, ListCol: UINT) -> HRESULT {
    if !((*this).m_ListToFolderColMap).is_null() {
        let count: UINT = DPA_GetPtrCount((*this).m_ListToFolderColMap) as UINT;
        if ListCol < count {
            let col: HRESULT = DPA_FastGetPtr((*this).m_ListToFolderColMap, ListCol as i32) as isize as HRESULT;
            ASSERT!(col >= 0 && col == SHGetLVColumnSubItem_body((*this).m_ListView.m_hWnd, ListCol));
            return col;
        } else if count != 0 {
        }
    }
    return SHGetLVColumnSubItem_body((*this).m_ListView.m_hWnd, ListCol);
}

pub unsafe fn CDefView_GetDetailsByFolderColumn_body(this: *mut CDefView, pidl: PCUITEMID_CHILD, FoldCol: UINT, sd: &mut SHELLDETAILS) -> HRESULT {
    let mut hr: HRESULT = E_FAIL;
    if !(*this).m_pSF2Parent.is_null() {
        hr = ((*(*(*this).m_pSF2Parent).lpVtbl).GetDetailsOf)((*this).m_pSF2Parent, pidl, FoldCol, sd as *mut SHELLDETAILS);
    }
    if (hr < 0) && !(*this).m_pSDParent.is_null() {
        hr = ((*(*(*this).m_pSDParent).lpVtbl).GetDetailsOf)((*this).m_pSDParent, pidl, FoldCol, sd as *mut SHELLDETAILS);
    }
    hr
}

pub unsafe fn CDefView_GetDetailsByListColumn_body(this: *mut CDefView, pidl: PCUITEMID_CHILD, ListCol: UINT, sd: &mut SHELLDETAILS) -> HRESULT {
    let hr: HRESULT = CDefView_MapListColumnToFolderColumn_body(this, ListCol);
    if hr >= 0 {
        return CDefView_GetDetailsByFolderColumn_body(this, pidl, ((hr) as u32), sd);
    }
    return hr;
}

pub unsafe fn CDefView_LoadColumn_body(this: *mut CDefView, FoldCol: UINT, ListCol: UINT, Insert: BOOL, ForceWidth: UINT) -> HRESULT {
    let mut ListCol = ListCol;
    let mut buf = [0u16; ((MAX_PATH) as usize)];
    let mut sd: SHELLDETAILS = core::mem::zeroed();
    sd.str_.uType = if STRRET_WSTR != 0 { 0 } else { 1 };

    let mut hr: HRESULT;
    hr = CDefView_GetDetailsByFolderColumn_body(this, core::ptr::null(), FoldCol, &mut sd);
    if hr < 0 {
        return hr;
    }

    hr = StrRetToStrNW(
        buf.as_mut_ptr(),
        buf.len() as UINT,
        core::ptr::addr_of_mut!(sd.str_).cast(),
        core::ptr::null(),
    );
    if hr < 0 {
        return hr;
    }

    let hwnd = (*this).m_ListView.m_hWnd;
    let mut chavewidth = CalculateCharWidth_body(hwnd);
    if chavewidth == 0 {
        chavewidth = 6;
    }

    let mut lvc: LVCOLUMN = core::mem::zeroed();
    lvc.mask = LVCF_TEXT | LVCF_FMT | LVCF_WIDTH | LVCF_SUBITEM;
    lvc.pszText = buf.as_mut_ptr();
    lvc.fmt = sd.fmt;
    lvc.cx = if ForceWidth != 0 {
        ForceWidth as i32
    } else {
        (sd.cxChar as u32 * chavewidth) as i32
    };
    lvc.iSubItem = FoldCol as i32;

    if (ListCol as i32) == -(1 as i32) {
        ASSERT!(Insert);
        if Insert != 0 {
            ListCol = 0x7fffffff;
        }
    }

    if Insert != 0 {
        ListView_InsertColumn(hwnd, ListCol as i32, &lvc);
    } else {
        ListView_SetColumn(hwnd, ListCol as i32, &lvc);
    }

    S_OK
}

pub unsafe fn CDefView_LoadColumns_body(this: *mut CDefView, pColList: *mut SIZE_T, ColListCount: UINT) -> HRESULT {
    unsafe {
        let hWndHdr = ListView_GetHeader((*this).m_ListView.m_hWnd);
        let mut newColCount: UINT = 0;
        let mut oldColCount: UINT = ((Header_GetItemCount(hWndHdr)) as u32);
        let mut width: UINT = 0;
        let mut foldCol: UINT = 0;
        let mut i: UINT = 0;
        let mut hr: HRESULT = S_FALSE;

        (*this).m_ListView.SetRedraw(FALSE);
        loop {
            if newColCount >= 0xffff {
                break;
            }

            if !pColList.is_null() {
                if i >= ColListCount {
                    break;
                }
                let current = *pColList.offset(i as isize);
                width = HIWORD(current as DWORD) as UINT;
                foldCol = LOWORD!(current) as UINT;
                i += 1;
            }

            let mut state: SHCOLSTATEF = 0;
            let p_sf2_parent = (*this).m_pSF2Parent;
            if p_sf2_parent.is_null()
                || ((*(*p_sf2_parent).lpVtbl).GetDefaultColumnState)(p_sf2_parent, foldCol, &mut state) < 0
            {
                state = SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT;
            }

            if foldCol == 0 {
                // Force the first column
            } else if (state & SHCOLSTATE_HIDDEN) != 0 {
                foldCol += 1;
                continue;
            } else if pColList.is_null() && (state & SHCOLSTATE_ONBYDEFAULT) == 0 {
                foldCol += 1;
                continue;
            }

            let insert: bool = newColCount >= oldColCount;
            let listCol: UINT = if insert {
                (-1i32) as UINT
            } else {
                newColCount
            };
            hr = CDefView_LoadColumn_body(this, foldCol, listCol, insert as BOOL, width);
            if hr < 0 {
                if pColList.is_null() {
                    hr = S_OK;
                }
                break;
            }
            newColCount += 1;
            foldCol += 1;
        }

        i = newColCount;
        while i < oldColCount {
            ListView_DeleteColumn((*this).m_ListView.m_hWnd, ((i) as i32));
            i += 1;
        }

        (*this).m_ListView.SetRedraw(TRUE);
        CDefView_ColumnListChanged_body(this);
        assert!((CDefView_MapFolderColumnToListColumn_body(this, 0)) >= 0);
        if !(*this).m_LoadColumnsList.is_null() {
            DPA_Destroy((*this).m_LoadColumnsList);
            (*this).m_LoadColumnsList = 0 as _;
        }

        return hr;
    }
}

pub unsafe fn CDefView_ColumnListChanged_body(this: *mut CDefView) -> () {
    let cache: HDPA = (*this).m_ListToFolderColMap;
    (*this).m_ListToFolderColMap = 0 as _;
    DPA_DeleteAllPtrs(cache);

    let mut i: UINT = 0;
    loop {
        let hr: HRESULT = CDefView_MapListColumnToFolderColumn_body(this, i);
        if hr < 0 {
            break;
        }
        if DPA_SetPtr(cache, ((i) as i32), hr as isize as *mut _) == 0 {
            break;
        }
        i = i.wrapping_add(1);
    }
    (*this).m_ListToFolderColMap = cache;

    loop {
        if RemoveMenu((*this).m_hMenuArrangeModes, 0, MF_BYPOSITION) == 0 {
            break;
        }
    }

    let mut hMenu: HMENU = GetSubmenuByID_body((*this).m_hMenu, FCIDM_MENU_VIEW);
    if !hMenu.is_null() {
        hMenu = GetSubmenuByID_body(hMenu, FCIDM_SHVIEW_ARRANGE);
        let mut i: UINT = DVIDM_ARRANGESORT_FIRST;
        while i <= DVIDM_ARRANGESORT_LAST && !hMenu.is_null() {
            RemoveMenu(hMenu, i, MF_BYCOMMAND);
            i = i.wrapping_add(1);
        }
        if (GetMenuItemID(hMenu, 0) as i32) <= 0 {
            RemoveMenu(hMenu, 0, MF_BYPOSITION);
        }
    }

    let mut buf: [WCHAR; ((MAX_PATH) as usize)] = [0; ((MAX_PATH) as usize)];
    let mut lvc: LVCOLUMN = core::mem::zeroed();
    lvc.mask = LVCF_TEXT;
    lvc.pszText = buf.as_mut_ptr();
    lvc.cchTextMax = buf.len() as _;

    let mut listCol: UINT = 0;
    while listCol < DEFVIEW_ARRANGESORT_MAXENUM {
        if ListView_GetColumn((*this).m_ListView.m_hWnd, ((listCol) as i32), &mut lvc) == 0 {
            break;
        }
        let foldCol: HRESULT = CDefView_MapListColumnToFolderColumn_body(this, listCol);
        assert!(foldCol >= 0);
        let _ = foldCol;
        AppendMenuItem_body((*this).m_hMenuArrangeModes, MF_STRING,
                           DVIDM_ARRANGESORT_FIRST + listCol, lvc.pszText, listCol as SIZE_T);
        listCol = listCol.wrapping_add(1);
    }

    ListView_RedrawItems((*this).m_ListView.m_hWnd, 0, 0x7fffffff);
    InvalidateRect((*this).m_ListView.m_hWnd, 0 as *const _, TRUE);
}

pub unsafe extern "system" fn CDefView_ListViewCompareItems(lParam1: LPARAM, lParam2: LPARAM, lpData: LPARAM) -> c_int {
    let pidl1: PCUIDLIST_RELATIVE = lParam1 as PCUIDLIST_RELATIVE;
    let pidl2: PCUIDLIST_RELATIVE = lParam2 as PCUIDLIST_RELATIVE;
    let pThis: *mut CDefView = lpData as *mut CDefView;

    let hres: i32 = ((*(*(*pThis).m_pSFParent).lpVtbl).CompareIDs)(
        (*pThis).m_pSFParent,
        (*pThis).m_sortInfo.FolderColumn as crate::shellcom::LPARAM,
        pidl1,
        pidl2,
    );

    if ((hres) < 0) {
        return 0;
    }

    let nDiff: i16 = HRESULT_CODE(hres) as i16;
    return (nDiff as i32) * (((*pThis).m_sortInfo.Direction) as i32);
}

pub unsafe fn CDefView__Sort_body(this: *mut CDefView, Col: i32) -> BOOL {
    let hHeader;
    let mut hColumn: HDITEM = unsafe { core::mem::zeroed() };
    let prevCol = (*this).m_sortInfo.ListColumn;
    (*this).m_sortInfo.bLoadedFromViewState = ((FALSE) != 0);

    if ((*this).m_ListView.GetWindowLongPtr(GWL_STYLE) & (LVS_NOSORTHEADER as LONG_PTR)) != 0 {
        return TRUE;
    }

    hHeader = ListView_GetHeader((*this).m_ListView.m_hWnd);
    if Col != -1 {
        if Col >= Header_GetItemCount(hHeader) {
            return FALSE;
        }

        if prevCol == Col {
            (*this).m_sortInfo.Direction *= -1;
        } else {
            (*this).m_sortInfo.Direction = 0;
        }
        (*this).m_sortInfo.ListColumn = Col;
    }
    if (*this).m_sortInfo.Direction == 0 {
        (*this).m_sortInfo.Direction += 1;
    }

    if prevCol != -1 && prevCol != (*this).m_sortInfo.ListColumn {
        hColumn.mask = HDI_FORMAT;
        Header_GetItem(hHeader, prevCol, &mut hColumn);
        hColumn.fmt &= !((HDF_SORTUP | HDF_SORTDOWN) as INT);
        Header_SetItem(hHeader, prevCol, &mut hColumn);
    }

    hColumn.mask = HDI_FORMAT;
    Header_GetItem(hHeader, (*this).m_sortInfo.ListColumn, &mut hColumn);
    hColumn.fmt &= !((HDF_SORTDOWN | HDF_SORTUP) as INT);
    hColumn.fmt |= (((if (*this).m_sortInfo.Direction > 0 {
        HDF_SORTUP
    } else {
        HDF_SORTDOWN
    })) as i32);
    Header_SetItem(hHeader, (*this).m_sortInfo.ListColumn, &mut hColumn);

    (*this).m_sortInfo.FolderColumn =
        CDefView_MapListColumnToFolderColumn_body(this, (*this).m_sortInfo.ListColumn as UINT);

    return (*this).m_ListView.SortItems(Some(CDefView_ListViewCompareItems), this as PVOID);
}

pub unsafe fn CDefView_GetItemAttributes_body(this: *mut CDefView, pidl: PCUITEMID_CHILD, Query: UINT) -> SFGAOF {
    let mut Attr: SFGAOF = Query as SFGAOF;
    return if ((*(*(*this).m_pSFParent).lpVtbl).GetAttributesOf)((*this).m_pSFParent, 1, &pidl as *const _, &mut Attr) >= 0 {
        Attr & Query
    } else {
        0
    };
}

pub unsafe fn CDefView_GetItemAttributes_2_body(this: *mut CDefView, i: i32, Query: UINT) -> SFGAOF {
    let pItem = CDefView__PidlByItem_body(this, i);
    if pItem.is_null() {
        0
    } else {
        CDefView_GetItemAttributes_body(this, pItem, Query)
    }
}

pub unsafe fn CDefView__PidlByItem_body(this: *mut CDefView, i: i32) -> PCUITEMID_CHILD {
    if ((*this).m_ListView.m_hWnd).is_null() {
        return core::ptr::null();
    }
    ((*this).m_ListView).GetItemData(i) as PCUITEMID_CHILD
}

pub unsafe fn CDefView__PidlByItem_2_body(this: *mut CDefView, lvItem: &mut LVITEM) -> PCUITEMID_CHILD {
    if (*this).m_ListView.is_null() {
        return core::ptr::null_mut();
    }
    lvItem.lParam as PCUITEMID_CHILD
}

pub unsafe fn CDefView_LV_FindItemByPidl_body(this: *mut CDefView, pidl: PCUITEMID_CHILD) -> i32 {
    let cItems: i32 = (*this).m_ListView.GetItemCount();
    let lParam: LPARAM = if !(*this).m_pSF2Parent.is_null() {
        SHCIDS_CANONICALONLY as LPARAM
    } else {
        0 as LPARAM
    };

    let mut i: i32 = 0 as i32;
    while i < cItems {
        let mut currentpidl: PCUITEMID_CHILD = CDefView__PidlByItem_body(this, i);
        let hr = ((*(*(*this).m_pSFParent).lpVtbl).CompareIDs)(
            (*this).m_pSFParent, lParam as crate::shellcom::LPARAM,
            pidl as PCUIDLIST_RELATIVE,
            currentpidl as PCUIDLIST_RELATIVE,
        );

        if hr >= 0 as i32 {
            if hr == S_EQUAL as i32 {
                return i;
            }
        } else {
            i = 0 as i32;
            while !pidl.is_null() && i < cItems {
                currentpidl = CDefView__PidlByItem_body(this, i);
                if !currentpidl.is_null()
                    && (*currentpidl).mkid.cb == (*pidl).mkid.cb
                    && memcmp(
                        currentpidl as *const core::ffi::c_void,
                        pidl as *const core::ffi::c_void,
                        (*pidl).mkid.cb as usize,
                    ) == 0
                {
                    DbgPrint(
                        b"Matched item #%d, broken CompareIDs?\n\0".as_ptr() as *const _,
                        i,
                    );
                }
                i += 1;
            }
            break;
        }
        i += 1;
    }

    return - (1 as i32);
}

pub unsafe fn CDefView_LV_AddItem_body(this: *mut CDefView, pidl: PCUITEMID_CHILD) -> i32 {
    let mut lvItem: LVITEMW = unsafe { core::mem::zeroed() };

    if CDefView__DoFolderViewCB_body(this, SFVM_ADDINGOBJECT, 0, pidl as LPARAM) == S_FALSE {
        return -1;
    }

    lvItem.mask = LVIF_TEXT | LVIF_IMAGE | LVIF_PARAM;
    lvItem.iItem = (*this).m_ListView.GetItemCount();
    lvItem.iSubItem = 0;
    lvItem.lParam = ILClone(pidl) as LPARAM;
    lvItem.pszText = LPSTR_TEXTCALLBACKW;
    lvItem.iImage = I_IMAGECALLBACK;
    lvItem.stateMask = LVIS_CUT;
    if (*this).m_HasCutItems {
        lvItem.mask |= LVIF_STATE;
        lvItem.state = if CDefView_GetItemAttributes_body(this, pidl, SFGAO_HIDDEN | SFGAO_GHOSTED) != 0 {
            LVIS_CUT
        } else {
            0
        };
    }

    return (*this).m_ListView.InsertItem(&mut lvItem);
}

pub unsafe fn CDefView_LV_DeleteItem_body(this: *mut CDefView, pidl: PCUITEMID_CHILD) -> BOOLEAN {
    let nIndex: i32;
    nIndex = CDefView_LV_FindItemByPidl_body(this, pidl);
    if nIndex < 0 {
        return ((FALSE) as u8);
    }
    CDefView__DoFolderViewCB_body(this, SFVM_REMOVINGOBJECT, 0, pidl as LPARAM);

    (((*this).m_ListView.DeleteItem(nIndex)) as u8)
}

pub unsafe fn CDefView_LV_RenameItem_body(this: *mut CDefView, pidlOld: PCUITEMID_CHILD, pidlNew: PCUITEMID_CHILD) -> BOOLEAN {
    let nItem: i32 = CDefView_LV_FindItemByPidl_body(this, pidlOld);

    if -1 != nItem
    {
        let mut lvItem: LVITEMW = unsafe { core::mem::zeroed() };
        lvItem.mask = LVIF_PARAM;
        lvItem.iItem = nItem;
        lvItem.iSubItem = 0;
        (*this).m_ListView.GetItem(&mut lvItem);

        let oldPidl: *mut core::ffi::c_void = lvItem.lParam as *mut core::ffi::c_void;

        lvItem.mask = LVIF_PARAM | LVIF_IMAGE | LVIF_TEXT;
        lvItem.iItem = nItem;
        lvItem.iSubItem = 0;
        lvItem.lParam = ILClone(pidlNew) as LPARAM;
        lvItem.pszText = LPSTR_TEXTCALLBACKW;
        lvItem.iImage = SHMapPIDLToSystemImageListIndex((*this).m_pSFParent, pidlNew, core::ptr::null_mut());
        (*this).m_ListView.SetItem(&mut lvItem);
        (*this).m_ListView.Update(nItem);

        SHFree(oldPidl);

        return ((TRUE) as u8);
    }

    return ((FALSE) as u8);
}

pub unsafe fn CDefView_LV_UpdateItem_body(this: *mut CDefView, nItem: INT, pidl: PCUITEMID_CHILD) -> BOOL {
    unsafe {
        let mut bResult: BOOL = FALSE;
        let mut lvItem: LVITEMW = core::mem::zeroed::<LVITEMW>();
        if nItem >= 0 {
            CDefView__DoFolderViewCB_body(this, SFVM_UPDATINGOBJECT, nItem as WPARAM, pidl as LPARAM);

            lvItem.mask = LVIF_IMAGE;
            lvItem.iItem = nItem;
            lvItem.iSubItem = 0;
            lvItem.iImage = SHMapPIDLToSystemImageListIndex((*this).m_pSFParent, pidl, core::ptr::null_mut());
            lvItem.stateMask = LVIS_CUT;
            if (*this).m_HasCutItems {
                lvItem.mask |= LVIF_STATE;
                lvItem.state = if CDefView_GetItemAttributes_body(this, pidl, SFGAO_HIDDEN | SFGAO_GHOSTED) != 0 {
                    LVIS_CUT
                } else {
                    0
                };
            }

            let pidlOld: PCUITEMID_CHILD = CDefView__PidlByItem_body(this, nItem);
            if !pidlOld.is_null() && {
                let clone = ILClone(pidl);
                lvItem.lParam = clone as LPARAM;
                !clone.is_null()
            } {
                lvItem.mask |= LVIF_PARAM;
            }

            bResult = (*this).m_ListView.SetItem(&lvItem);
            if bResult != 0 && lvItem.lParam != 0 && !pidlOld.is_null() {
                ILFree(pidlOld as PITEMID_CHILD);
            }

            let mut i: u32 = 0;
            while (*this).m_ListView.SetItemText(nItem, ((i) as i32), LPSTR_TEXTCALLBACK) != 0 {
                i += 1;
            }
        }
        bResult
    }
}

pub unsafe fn CDefView_LV_UpdateItem_2_body(this: *mut CDefView, pidl: PCUITEMID_CHILD) -> BOOL {
    let nItem = CDefView_LV_FindItemByPidl_body(this, pidl);
    if nItem >= 0 {
        CDefView_LV_UpdateItem_body(this, nItem, pidl)
    } else {
        FALSE
    }
}

pub unsafe fn CDefView_LV_RefreshIcon_body(this: *mut CDefView, iItem: INT) -> () {
    let mut lvItem: LVITEMW = unsafe { core::mem::zeroed() };
    lvItem.mask = LVIF_IMAGE;
    lvItem.iItem = iItem;
    lvItem.iImage = I_IMAGECALLBACK;
    CListView_SetItem_body(&mut (*this).m_ListView, &mut lvItem);
    // Re-query the text too, or an ejected CD keeps its old volume label
    // (upstream 6730ec5508f; it renames this LV_RefreshItem).
    let mut iCol: u32 = 0;
    while (*this).m_ListView.SetItemText(iItem, iCol as i32, LPSTR_TEXTCALLBACK) != 0 {
        iCol += 1;
    }
    CListView_Update_body(&mut (*this).m_ListView, iItem);
}

pub unsafe fn CDefView_LV_RefreshIcons_body(this: *mut CDefView) -> () {
    let mut iItem: i32 = -1;
    loop {
        iItem = ListView_GetNextItem((*this).m_ListView.m_hWnd, iItem, LVNI_ALL);
        if iItem == -1 {
            break;
        }

        CDefView_LV_RefreshIcon_body(this, iItem);
    }
}

pub unsafe extern "system" fn CDefView_fill_list(ptr: *mut core::ffi::c_void, arg: *mut core::ffi::c_void) -> c_int {
    let pidl = ptr as PITEMID_CHILD;
    let p_this = arg as *mut CDefView;

    if CDefView_IncludeObject_body(p_this, pidl) == S_OK && !(*p_this).m_ListView.is_null() {
        CDefView_LV_AddItem_body(p_this, pidl);
    }

    SHFree(pidl);
    return TRUE;
}

pub unsafe fn CDefView_FillList_body(this: *mut CDefView, IsRefreshCommand: BOOL) -> HRESULT {
    let mut pEnumIDList: *mut IEnumIDList = core::ptr::null_mut();
    let _pEnumIDList_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pEnumIDList));
    let mut pidl: PITEMID_CHILD = core::ptr::null_mut();
    let mut dwFetched: DWORD = 0;
    let mut hRes: HRESULT;
    let mut hdpa: HDPA;
    let mut dFlags: DWORD = SHCONTF_NONFOLDERS
        | (if ((*this).m_FolderSettings.fFlags & FWF_NOSUBFOLDERS) != 0 {
            0
        } else {
            SHCONTF_FOLDERS
        });

    let mut shellstate: SHELLSTATE = core::mem::zeroed();
    SHGetSetSettings(
        &mut shellstate,
        SSF_SHOWALLOBJECTS | SSF_SHOWSUPERHIDDEN,
        FALSE,
    );
    if (CDefView_GetCommDlgViewFlags_body(this) & CDB2GVF_SHOWALLFILES) != 0 {
        shellstate.set_fShowAllObjects(TRUE);
        shellstate.set_fShowSuperHidden(TRUE);
    }

    if shellstate.fShowAllObjects() != 0 {
        dFlags |= SHCONTF_INCLUDEHIDDEN;
        let _ = (*this).m_ListView.SendMessageW(
            LVM_SETCALLBACKMASK,
            LVIS_CUT as WPARAM,
            0,
        );
    }
    if shellstate.fShowSuperHidden() != 0 {
        dFlags |= SHCONTF_INCLUDESUPERHIDDEN;
        let _ = (*this).m_ListView.SendMessageW(
            LVM_SETCALLBACKMASK,
            LVIS_CUT as WPARAM,
            0,
        );
    }

    hRes = ((*(*(*this).m_pSFParent).lpVtbl).EnumObjects)(
        (*this).m_pSFParent,
        (((*this).m_hWnd) as _),
        dFlags,
        &mut pEnumIDList,
    );
    if hRes != S_OK {
        // NO explicit Release here. CDefView.cpp:1545 declares `CComPtr<IEnumIDList>
        // pEnumIDList` and releases NOWHERE -- the destructor does it once, on every return
        // path. `_pEnumIDList_cpp_com_guard` IS that destructor. Releasing here as well drops
        // the refcount twice; the guard then reads `lpVtbl` out of a freed block and calls
        // through it. See the header of this file and the field manual.
        if hRes == S_FALSE {
            return NOERROR;
        }
        return hRes;
    }

    hdpa = DPA_Create(16);
    if hdpa.is_null() {
        // Guard-owned; see above.
        return E_OUTOFMEMORY;
    }

    while (S_OK
        == ((*(*pEnumIDList).lpVtbl).Next)(
            pEnumIDList,
            1,
            &mut pidl as *mut _,
            &mut dwFetched,
        ))
        && dwFetched != 0
    {
        if DPA_AppendPtr(hdpa, ((pidl) as _)) == -(1) {
            SHFree(pidl);
        }
    }

    let _ = (*this).m_ListView.SetRedraw(FALSE);

    DPA_DestroyCallback(hdpa, Some(CDefView_fill_list), this as *mut _);

    let mut sortCol: i32 = -(1 as i32);
    if IsRefreshCommand == 0 && !(*this).m_sortInfo.bLoadedFromViewState {
        (*this).m_sortInfo.Direction = 0;
        sortCol = 0 as i32;
        if !(*this).m_pSF2Parent.is_null() {
            let mut folderSortCol: ULONG = sortCol as ULONG;
            let mut dummy: ULONG = 0;
            let mut hr: HRESULT = ((*(*(*this).m_pSF2Parent).lpVtbl).GetDefaultColumn)(
                (*this).m_pSF2Parent,
                0,
                &mut folderSortCol,
                &mut dummy,
            );
            if hr >= 0 {
                hr = CDefView_MapFolderColumnToListColumn_body(this, folderSortCol);
            }
            if hr >= 0 {
                sortCol = hr as i32;
            }
        }
    }
    CDefView__Sort_body(this, sortCol);

    if !(*this).m_viewinfo_data.hbmBack.is_null() {
        DeleteObject((((*this).m_viewinfo_data.hbmBack) as _));
        (*this).m_viewinfo_data.hbmBack = core::ptr::null_mut();
    }

    (*this).m_viewinfo_data.cbSize = core::mem::size_of_val(&(*this).m_viewinfo_data) as _;
    CDefView__DoFolderViewCB_body(this, SFVM_GET_CUSTOMVIEWINFO,
        0,
        &mut (*this).m_viewinfo_data as *mut _ as LPARAM,
    );

    let _ = (*this).m_ListView.SetRedraw(TRUE);

    CDefView_UpdateListColors_body(this);

    if ((*this).m_FolderSettings.fFlags & FWF_DESKTOP) == 0 {
        let _ = (*this).m_ListView.InvalidateRect(core::ptr::null(), TRUE);
    }

    if IsRefreshCommand != 0 {
        CDefView_UpdateStatusbarLocation_body(this);
    }

    CDefView__DoFolderViewCB_body(
        this,
        SFVM_LISTREFRESHED,
        0,
        0,
    );

    // Guard-owned; see above.
    return S_OK;
}

pub unsafe fn CDefView_OnShowWindow_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    if (((*this).m_ListView.IsWindow()) != 0) {
        (*this).m_ListView.UpdateWindow();
    }
    *bHandled = FALSE;
    0
}

pub unsafe fn CDefView_OnGetDlgCode_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    (*this).m_ListView.SendMessageW(uMsg, 0, 0)
}

pub unsafe fn CDefView_OnDestroy_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    if (*this).m_Destroyed == 0 {
        (*this).m_Destroyed = TRUE;
        RevokeDragDrop((((*this).m_hWnd) as _));
        SHChangeNotifyDeregister((*this).m_hNotify);
        (*this).m_hNotify = 0;
        ((*this).m_ClipboardChain).Unhook((*this).m_hWnd);
        if !(*this).m_pFileMenu.is_null() {
            IUnknown_SetSite((((*this).m_pFileMenu) as _), core::ptr::null_mut());
            CDefView_cpp_com_assign_m_pFileMenu(this, core::ptr::null_mut());
        }
        if !(*this).m_hMenu.is_null() {
            DestroyMenu((*this).m_hMenu);
            (*this).m_hMenu = core::ptr::null_mut();
        }
        SHFree(((*this).m_pidlParent) as LPVOID);
        (*this).m_pidlParent = core::ptr::null_mut();
    }
    *bHandled = FALSE;
    0
}

pub unsafe fn CDefView_OnEraseBackground_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    if ((*this).m_FolderSettings.fFlags & (FWF_DESKTOP | FWF_TRANSPARENT)) != 0 {
        return SendMessageW(GetParent((*this).m_hWnd), WM_ERASEBKGND, wParam, lParam);
    }
    *bHandled = FALSE;
    0
}

pub unsafe fn DrawTileBitmap_body(hDC: HDC, prc: LPCRECT, hbm: HBITMAP, nWidth: INT, nHeight: INT, dx: INT, dy: INT) -> () {
    let x0 = (*prc).left;
    let y0 = (*prc).top;
    let x1 = (*prc).right;
    let y1 = (*prc).bottom;
    let x0 = x0 + dx;
    let y0 = y0 + dy;

    let hMemDC: HDC = CreateCompatibleDC(hDC);
    let hbmOld: HGDIOBJ = SelectObject(hMemDC, ((hbm) as _));

    let mut y = y0;
    while y < y1 {
        let mut x = x0;
        while x < x1 {
            BitBlt(
                hDC,
                x,
                y,
                nWidth,
                nHeight,
                hMemDC,
                0,
                0,
                SRCCOPY,
            );
            x += nWidth;
        }
        y += nHeight;
    }

    SelectObject(hMemDC, hbmOld);
    DeleteDC(hMemDC);
}

pub unsafe fn CDefView_OnPrintClient_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let hDC: HDC = wParam as HDC;
    let mut rc: RECT = core::mem::zeroed();

    GetClientRect((*this).m_ListView.m_hWnd, &mut rc as *mut RECT);

    if (*this).m_viewinfo_data.hbmBack as usize != 0 {
        let mut bm: BITMAP = core::mem::zeroed();
        if GetObjectW(
            (((*this).m_viewinfo_data.hbmBack) as _),
            core::mem::size_of::<BITMAP>() as INT,
            (&mut bm as *mut BITMAP) as *mut core::ffi::c_void,
        ) != 0 {
            let dx: INT = -(GetScrollPos((*this).m_ListView.m_hWnd, ((SB_HORZ) as i32)) % bm.bmWidth);
            let dy: INT = -(GetScrollPos((*this).m_ListView.m_hWnd, ((SB_VERT) as i32)) % bm.bmHeight);
            DrawTileBitmap_body(
                hDC,
                &rc as *const RECT,
                (*this).m_viewinfo_data.hbmBack,
                bm.bmWidth,
                bm.bmHeight,
                dx,
                dy,
            );
        }
    } else {
        SHFillRectClr(
            hDC,
            &rc as *const RECT,
            GetViewColor_body((*this).m_viewinfo_data.clrTextBack, COLOR_WINDOW),
        );
    }

    *bHandled = TRUE;

    return TRUE as LRESULT;
}

pub unsafe fn CDefView_OnSysColorChange_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    CDefView_UpdateListColors_body(this);
    return (*this).m_ListView.SendMessageW(uMsg, 0, 0);
}

pub unsafe fn CDefView_OnGetShellBrowser_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    ((*this).m_pShellBrowser) as LRESULT
}

pub unsafe fn CDefView_OnNCCreate_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    CDefView_AddRef(this);
    *bHandled = FALSE;
    0
}

pub unsafe fn CDefView_OnFinalMessage_body(this: *mut CDefView, arg0: HWND) -> () {
    ((*(*(this as *mut IUnknown)).lpVtbl).Release)(this as *mut IUnknown);
}

pub unsafe fn CDefView_OnCreate_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let pSFParent = (*this).m_pSFParent;
    let mut pdt: *mut IDropTarget = core::ptr::null_mut();
    let _pdt_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pdt));
    let mut ppf2: *mut IPersistFolder2 = core::ptr::null_mut();
    let _ppf2_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(ppf2));

    // CDefView.cpp:1776 `if (SUCCEEDED(QueryInterface(IID_PPV_ARG(IDropTarget, &pdt))))` --
    // a BARE member call, so the receiver is `this`, the VIEW. It asks the view for its own
    // IDropTarget and registers THAT as the window's drop target. Routing it to
    // `m_pSFParent` instead asks the FOLDER, which is a different object with a different
    // drop target, and registers the wrong one (or none). An unqualified call in a C++
    // member function is `this->`; the nearest COM pointer in scope is not a substitute.
    if CDefView_QueryInterface(
        this,
        ((core::ptr::addr_of!(IID_IDropTarget)) as _), core::ptr::addr_of_mut!(pdt).cast(),
    ) >= 0
    {
        if RegisterDragDrop((*this).m_hWnd.cast(), pdt.cast()) < 0 {
            ERR!("Error Registering DragDrop\n");
        }
    }

    let _ = ((*((*pSFParent).lpVtbl)).QueryInterface)(
        pSFParent,
        ((core::ptr::addr_of!(IID_IPersistFolder2)) as _), core::ptr::addr_of_mut!(ppf2).cast(),
    );
    if !ppf2.is_null() {
        let _ = ((*((*ppf2).lpVtbl)).GetCurFolder)(
            ppf2,
            &mut (*this).m_pidlParent as *mut _,
        );
    }

    if CDefView_CreateList_body(this) != 0 {
        if CDefView_InitList_body(this) != 0 {
            let _ = CDefView_FillList_body(this, FALSE);
        }
    }

    if (((*this).m_FolderSettings.fFlags) & FWF_DESKTOP) != 0 {
        let mut hwndSB: HWND = core::mem::zeroed();
        let pShellBrowser = (*this).m_pShellBrowser;
        let _ = ((*((*pShellBrowser).lpVtbl)).GetWindow)(
            pShellBrowser,
            &mut hwndSB as *mut _,
        );
        let _ = SetShellWindowEx(hwndSB, (*this).m_ListView.m_hWnd);
    }

    let mut pidlTarget: LPITEMIDLIST = core::ptr::null_mut();
    let mut fEvents: LONG = 0;
    let mut hr: HRESULT = CDefView__DoFolderViewCB_body(
        this,
        SFVM_GETNOTIFY,
        &mut pidlTarget as *mut _ as WPARAM,
        &mut fEvents as *mut _ as LPARAM,
    );
    if hr < 0 || (pidlTarget.is_null() && fEvents == 0) {
        pidlTarget = (*this).m_pidlParent;
        fEvents = ((SHCNE_ALLEVENTS) as i32);
    }

    let mut ntreg: SHChangeNotifyEntry = core::mem::zeroed();
    hr = CDefView__DoFolderViewCB_body(
        this,
        SFVM_QUERYFSNOTIFY,
        0,
        &mut ntreg as *mut _ as LPARAM,
    );
    if hr < 0 {
        ntreg.fRecursive = FALSE;
        ntreg.pidl = pidlTarget;
    }

    // CDefView.cpp:1820 `#ifdef _M_ARM64` — carried, not resolved away. SHChangeNotifyRegister
    // sends a cross-thread SendMessageW to the shell window (Shell_TrayWnd, on the main
    // explorer thread); on arm64 that deadlocks the desktop thread before the taskbar paints.
    // The C++ skips registration there so the desktop comes up, and change notifications are
    // silently dropped until win32k's cross-thread message dispatch is fixed. Dropping the
    // `#ifdef` and calling unconditionally reproduced exactly that hang: the arm64 guest
    // stopped after `[EXPLORER] startup: runCmd FAILED (2)` with a frozen serial log and the
    // kmtest sweep never started, while amd64 and i386 were green.
    #[cfg(target_arch = "aarch64")]
    {
        (*this).m_hNotify = 0;
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        (*this).m_hNotify = SHChangeNotifyRegister(
            (*this).m_hWnd,
            (SHCNRF_InterruptLevel | SHCNRF_ShellLevel | SHCNRF_NewDelivery) as c_int,
            fEvents,
            SHV_CHANGE_NOTIFY,
            1,
            core::ptr::addr_of!(ntreg).cast_mut(),
        );
    }

    (*this).m_hAccel = LoadAcceleratorsW(
        shell32_hInstance,
        MAKEINTRESOURCEW(IDA_SHELLVIEW),
    );

    let mut bForceFullStatusBar: i32 = 0;
    let bIsFileSystem: BOOL = (SHGetAttributes(
        core::ptr::null_mut(),
        (*this).m_pidlParent,
        SFGAO_FILESYSTEM,
    ) & SFGAO_FILESYSTEM) as BOOL;
    (*this).m_SpecialFolder = if bIsFileSystem != 0 {
        -1
    } else {
        0x7f
    };

    if _ILIsDesktop((*this).m_pidlParent) != 0 {
        (*this).m_SpecialFolder = ((CSIDL_DESKTOP) as i8);
    } else if IsEqualPersistClassID(ppf2.cast(), core::ptr::addr_of!(CLSID_RecycleBin).cast()) != 0 {
        bForceFullStatusBar = CSIDL_BITBUCKET;
        (*this).m_SpecialFolder = ((bForceFullStatusBar) as i8);
    } else if bIsFileSystem != 0 {
        let mut pidlNet: *mut ITEMIDLIST = SHCloneSpecialIDList(
            core::ptr::null_mut(),
            ((CSIDL_NETWORK) as i32),
            FALSE,
        );
        if ILIsParent(
            pidlNet,
            (*this).m_pidlParent,
            FALSE,
        ) != 0
            && ILGetSize(((pidlNet) as _))
                < ILGetSize((((*this).m_pidlParent) as _))
        {
            (*this).m_SpecialFolder = ((CSIDL_NETWORK) as i8);
        }
    }

    (*this).m_isFullStatusBar = bIsFileSystem != 0 || bForceFullStatusBar != 0;

    CDefView__ForceStatusBarResize_body(this);
    CDefView_UpdateStatusbar_body(this);
    CDefView_UpdateStatusbarLocation_body(this);

    if !pdt.is_null() {
        ((*((*pdt).lpVtbl)).Release)(pdt);
    }
    if !ppf2.is_null() {
        ((*((*ppf2).lpVtbl)).Release)(ppf2);
    }

    S_OK as LRESULT
}

pub unsafe fn CDefView_FillFileMenu_body(this: *mut CDefView) -> HRESULT {
    let hFileMenu: HMENU = GetSubmenuByID_body((*this).m_hMenu, FCIDM_MENU_FILE);
    if hFileMenu.is_null() {
        return E_FAIL;
    }

    let mut i: i32 = GetMenuItemCount(hFileMenu) as i32 - 1 as i32;
    while i >= 0 as i32 {
        let id: u32 = GetMenuItemID(hFileMenu, ((i as u32) as i32)) as u32;
        if id < FCIDM_BROWSERFIRST as u32 || id > FCIDM_BROWSERLAST as u32 {
            DeleteMenu(hFileMenu, i as u32, MF_BYPOSITION as u32);
        }
        i -= 1;
    }

    if !(*this).m_pFileMenu.is_null() {
        IUnknown_SetSite((*this).m_pFileMenu.cast(), core::ptr::null_mut());
        if !(*this).m_pFileMenu.is_null() {
        let value = (*this).m_pFileMenu as *mut IUnknown;
        ((*(*value).lpVtbl).Release)(value);
        (*this).m_pFileMenu = core::ptr::null_mut();
    }
    }

    let selcount: u32 = (*this).m_ListView.GetSelectedCount();

    let mut hr: HRESULT = CDefView_GetItemObject_body(this, if selcount != 0 { SVGIO_SELECTION } else { SVGIO_BACKGROUND },
        core::ptr::addr_of!(IID_IContextMenu),
        core::ptr::addr_of_mut!((*this).m_pFileMenu).cast()
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let hmenu: HMENU = CreatePopupMenu();

    let cmf: u32 = GetContextMenuFlags_body((*this).m_pShellBrowser, CDefView_GetSelectionAttributes_body(this, SFGAO_CANRENAME));

    let pFileMenu: *mut IContextMenu = (*this).m_pFileMenu;
    hr = ((*(*pFileMenu).lpVtbl).QueryContextMenu)(
        pFileMenu,
        hmenu.cast(),
        0,
        DVIDM_CONTEXTMENU_FIRST,
        DVIDM_CONTEXTMENU_LAST,
        cmf
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    if selcount == 0 {
        DeleteMenu(hmenu, FCIDM_SHVIEW_VIEW, MF_BYCOMMAND);
        DeleteMenu(hmenu, FCIDM_SHVIEW_ARRANGE, MF_BYCOMMAND);
        DeleteMenu(hmenu, FCIDM_SHVIEW_REFRESH, MF_BYCOMMAND);
    }

    _ = Shell_MergeMenus(
        hFileMenu.cast(),
        hmenu.cast(),
        0,
        0,
        0xFFFF,
        MM_ADDSEPARATOR | MM_SUBMENUSHAVEIDS
    );
    _ = DestroyMenu(hmenu);

    S_OK
}

pub unsafe fn CDefView_FillEditMenu_body(this: *mut CDefView) -> HRESULT {
    let hEditMenu = GetSubmenuByID_body((*this).m_hMenu, FCIDM_MENU_EDIT);
    if hEditMenu.is_null() {
        return E_FAIL;
    }

    let hmenuContents = LoadMenuW(shell32_hInstance, CPP_COM_WSTR_4FA92E643652.as_ptr());
    if hmenuContents.is_null() {
        return E_FAIL;
    }

    Shell_MergeMenus(
        hEditMenu.cast(),
        hmenuContents.cast(),
        0,
        0,
        0xFFFF,
        0
    );

    DestroyMenu(hmenuContents);

    return S_OK;
}

pub unsafe fn CDefView_FillViewMenu_body(this: *mut CDefView) -> HRESULT {
    let hViewMenu = GetSubmenuByID_body((*this).m_hMenu, FCIDM_MENU_VIEW);
    if hViewMenu.is_null() {
        return E_FAIL;
    }

    (*this).m_hMenuViewModes = LoadMenuW(shell32_hInstance, CPP_COM_WSTR_22C79B88393E.as_ptr());
    if (*this).m_hMenuViewModes.is_null() {
        return E_FAIL;
    }

    let i = SHMenuIndexFromID(hViewMenu, FCIDM_MENU_VIEW_SEP_OPTIONS);
    Shell_MergeMenus(
        hViewMenu.cast(),
        (*this).m_hMenuViewModes.cast(),
        i,
        0,
        0xFFFF,
        MM_ADDSEPARATOR | MM_DONTREMOVESEPS | MM_SUBMENUSHAVEIDS,
    );

    return S_OK;
}

pub unsafe fn CDefView_FillArrangeAsMenu_body(this: *mut CDefView, hmenuArrange: HMENU) -> HRESULT {
    unsafe {
        let mut forceMerge: bool = false;
        let mut currentSortId: UINT = DVIDM_ARRANGESORT_FIRST + (((*this).m_sortInfo.ListColumn) as u32);

        RemoveMenu((*this).m_hMenuArrangeModes, DVIDM_ARRANGESORT_LAST, MF_BYCOMMAND);
        if (*this).m_sortInfo.ListColumn >= ((DEFVIEW_ARRANGESORT_MAXENUM) as i32) {
            const _: () = assert!(DEFVIEW_ARRANGESORT_MAXENUM < DEFVIEW_ARRANGESORT_MAX);
            const _: () = assert!(DVIDM_ARRANGESORT_FIRST + DEFVIEW_ARRANGESORT_MAXENUM == DVIDM_ARRANGESORT_LAST);
            let mut buf: [WCHAR; ((MAX_PATH) as usize)] = [0; ((MAX_PATH) as usize)];
            let mut lvc: LVCOLUMN = core::mem::zeroed();
            lvc.mask = LVCF_TEXT;
            lvc.pszText = buf.as_mut_ptr();
            lvc.cchTextMax = (buf).len() as i32;
            currentSortId = DVIDM_ARRANGESORT_LAST;
            forceMerge = true;
            ListView_GetColumn((*this).m_ListView.m_hWnd, (*this).m_sortInfo.ListColumn, &mut lvc);
            AppendMenuItem_body((*this).m_hMenuArrangeModes, MF_STRING, currentSortId, lvc.pszText, (*this).m_sortInfo.ListColumn as SIZE_T);
        }

        if GetMenuItemID(hmenuArrange, 0) == FCIDM_SHVIEW_AUTOARRANGE || forceMerge {
            Shell_MergeMenus(hmenuArrange.cast(), (*this).m_hMenuArrangeModes.cast(), 0, 0, 0xFFFF, MM_ADDSEPARATOR);
        }

        CheckMenuRadioItem(hmenuArrange,
            DVIDM_ARRANGESORT_FIRST, DVIDM_ARRANGESORT_LAST,
            currentSortId, MF_BYCOMMAND);

        if (*this).m_FolderSettings.ViewMode == FVM_DETAILS || (*this).m_FolderSettings.ViewMode == FVM_LIST {
            EnableMenuItem(hmenuArrange, FCIDM_SHVIEW_AUTOARRANGE, MF_BYCOMMAND | MF_GRAYED);
            EnableMenuItem(hmenuArrange, FCIDM_SHVIEW_ALIGNTOGRID, MF_BYCOMMAND | MF_GRAYED);
        } else {
            EnableMenuItem(hmenuArrange, FCIDM_SHVIEW_AUTOARRANGE, MF_BYCOMMAND);
            EnableMenuItem(hmenuArrange, FCIDM_SHVIEW_ALIGNTOGRID, MF_BYCOMMAND);

            if CDefView_GetAutoArrange_body(this) == S_OK {
                CheckMenuItem(hmenuArrange, FCIDM_SHVIEW_AUTOARRANGE, MF_CHECKED);
            } else {
                CheckMenuItem(hmenuArrange, FCIDM_SHVIEW_AUTOARRANGE, MF_UNCHECKED);
            }

            if CDefView__GetSnapToGrid_body(this) == S_OK {
                CheckMenuItem(hmenuArrange, FCIDM_SHVIEW_ALIGNTOGRID, MF_CHECKED);
            } else {
                CheckMenuItem(hmenuArrange, FCIDM_SHVIEW_ALIGNTOGRID, MF_UNCHECKED);
            }
        }

        S_OK
    }
}

pub unsafe fn CDefView_CheckViewMode_body(this: *mut CDefView, hmenuView: HMENU) -> HRESULT {
    if (*this).m_FolderSettings.ViewMode >= ((FVM_FIRST) as u32) && (*this).m_FolderSettings.ViewMode <= ((FVM_LAST) as u32) {
        let iItemFirst: u32 = FCIDM_SHVIEW_BIGICON;
        let iItemLast: u32 = iItemFirst + ((FVM_LAST) as u32) - ((FVM_FIRST) as u32);
        let iItem: u32 = iItemFirst + (*this).m_FolderSettings.ViewMode - ((FVM_FIRST) as u32);
        CheckMenuRadioItem(hmenuView, iItemFirst, iItemLast, iItem, MF_BYCOMMAND);
    }

    S_OK
}

pub unsafe fn CDefView_DoColumnContextMenu_body(this: *mut CDefView, lParam: LPARAM) -> LRESULT {
    let maxItems: UINT = 15 as UINT;
    let idMore: UINT = 0x1337 as UINT;
    let mut idFirst: UINT = idMore + (1 as UINT);
    let mut idLast: UINT = idFirst;
    let mut lastValidListCol: UINT = 0 as UINT;
    let mut showMore: UINT = ((GetKeyState(((VK_SHIFT) as i32)) as i32) < (0 as i32)) as UINT;
    let mut pt: POINT = POINT { x: GET_X_LPARAM(lParam), y: GET_Y_LPARAM(lParam) };
    let hWndHdr: HWND = ListView_GetHeader((*this).m_ListView.m_hWnd);

    if lParam == !(0 as LPARAM) {
        let mut r: RECT = core::mem::zeroed();
        GetWindowRect(hWndHdr, &mut r);
        pt.x = r.left + ((r.right - r.left) / (2 as i32));
        pt.y = r.top + ((r.bottom - r.top) / (2 as i32));
    }

    let hMenu: HMENU = CreatePopupMenu();
    if hMenu.is_null() {
        return 0 as LRESULT;
    }

    let mut foldCol: UINT = 0 as UINT;
    loop {
        let mut buf: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut sd: SHELLDETAILS = core::mem::zeroed();
        sd.str_.uType = (STRRET_WSTR as UINT == 0) as UINT;
        if CDefView_GetDetailsByFolderColumn_body(this, core::ptr::null(), foldCol, &mut sd) < (0 as HRESULT) {
            break;
        }
        if StrRetToStrNW(buf.as_mut_ptr(), buf.len() as UINT, core::ptr::addr_of_mut!(sd.str_).cast(), core::ptr::null()) < (0 as HRESULT) {
            break;
        }

        let mut state: SHCOLSTATEF = 0 as SHCOLSTATEF;
        if (*this).m_pSF2Parent.is_null() || ((*(*(*this).m_pSF2Parent).lpVtbl).GetDefaultColumnState)((*this).m_pSF2Parent, foldCol, &mut state) < (0 as HRESULT) {
            state = 0 as SHCOLSTATEF;
        }
        showMore |= (state & (SHCOLSTATE_SECONDARYUI as SHCOLSTATEF)) as UINT;

        let mut mf: UINT = MF_STRING as UINT;
        let listCol: HRESULT = CDefView_MapFolderColumnToListColumn_body(this, foldCol);

        if foldCol == (0 as UINT) {
            mf |= (MF_CHECKED | MF_GRAYED | MF_DISABLED) as UINT;
        } else if (state & ((SHCOLSTATE_SECONDARYUI | SHCOLSTATE_HIDDEN) as SHCOLSTATEF)) != 0 {
            foldCol += 1;
            continue;
        } else if listCol >= (0 as HRESULT) {
            mf |= MF_CHECKED as UINT;
        }

        if AppendMenuItem_body(hMenu, mf, idLast, buf.as_ptr(), (lastValidListCol + (1 as UINT)) as SIZE_T) != 0 {
            idLast += 1;
            if listCol >= (0 as HRESULT) {
                lastValidListCol = listCol as UINT;
            }
        }

        if idLast - idFirst == maxItems {
            showMore += 1;
            break;
        }

        foldCol += 1;
    }

    if showMore != 0 {
    }

    PostMessage((*this).m_ListView.m_hWnd, WM_SETCURSOR as UINT, (*this).m_ListView.m_hWnd as WPARAM, HTCLIENT as LPARAM);

    let idCmd: UINT = TrackPopupMenu(
        hMenu,
        (TPM_RETURNCMD | TPM_LEFTBUTTON | TPM_RIGHTBUTTON) as UINT,
        pt.x,
        pt.y,
        ((0 as UINT) as i32),
        hWndHdr,
        core::ptr::null(),
    ) as UINT;

    if idCmd == idMore {
    } else if idCmd != 0 {
        let foldCol: UINT = idCmd - idFirst;
        let mut listCol: HRESULT = CDefView_MapFolderColumnToListColumn_body(this, foldCol);
        if listCol >= (0 as HRESULT) {
            ListView_DeleteColumn((*this).m_ListView.m_hWnd, listCol);
        } else {
            listCol = GetMenuItemDataById_body(hMenu, idCmd) as HRESULT;
            CDefView_LoadColumn_body(this, foldCol, listCol as UINT, TRUE as BOOL, 0 as UINT);
        }
        CDefView_ColumnListChanged_body(this);
    }

    DestroyMenu(hMenu);

    return 0 as LRESULT;
}

pub unsafe fn CDefView_GetSelections_body(this: *mut CDefView) -> UINT {
    let count: UINT = ((*this).m_ListView).GetSelectedCount() as UINT;
    if count > (*this).m_cidl || count == 0 || (*this).m_apidl.is_null() {
        SHFree((*this).m_apidl as *mut core::ffi::c_void);
        (*this).m_apidl = SHAlloc((count as usize).wrapping_mul(core::mem::size_of::<PCUITEMID_CHILD>()) as SIZE_T) as *mut PCUITEMID_CHILD;
        if (*this).m_apidl.is_null() {
            (*this).m_cidl = 0;
            return 0;
        }
    }
    (*this).m_cidl = count;

    let mut i: UINT = 0;
    let mut lv_index: i32 = -1;
    while {
        lv_index = ((*this).m_ListView).GetNextItem(lv_index, ((LVNI_SELECTED) as u16));
        lv_index > -1
    } {
        (*this).m_apidl.add(i as usize).write(CDefView__PidlByItem_body(this, lv_index));
        i = i.wrapping_add(1);
        if i == (*this).m_cidl {
            break;
        }
    }

    (*this).m_cidl
}

pub unsafe fn CDefView_GetSelectionAttributes_body(this: *mut CDefView, Query: SFGAOF) -> SFGAOF {
    if CDefView_GetSelections_body(this) == 0 {
        return 0;
    }
    let mut Attr: SFGAOF = Query;
    if ((*(*(*this).m_pSFParent).lpVtbl).GetAttributesOf)(
        (*this).m_pSFParent,
        (*this).m_cidl,
        (((*this).m_apidl) as _),
        &mut Attr,
    ) >= 0
    {
        Attr & Query
    } else {
        0
    }
}

pub unsafe fn CDefView_InvokeContextMenuCommand_body(this: *mut CDefView, pCM: &mut *mut IContextMenu, lpVerb: LPCSTR, pt: *mut POINT, TryMapVerb: bool) -> HRESULT {
    let mut cmi = core::mem::MaybeUninit::<CMINVOKECOMMANDINFOEX>::uninit();
    ZeroMemory(
        cmi.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of::<CMINVOKECOMMANDINFOEX>(),
    );
    let mut cmi = cmi.assume_init();

    cmi.cbSize = core::mem::size_of::<CMINVOKECOMMANDINFOEX>() as u32;
    cmi.hwnd = (*this).m_hWnd;
    cmi.lpVerb = lpVerb;
    cmi.nShow = ((SW_SHOW) as i32);

    let mut szverbW: [u16; core::mem::size_of_val(b"properties\0")] =
        [0; core::mem::size_of_val(b"properties\0")];
    let verbmap: [u16; 7] = [
        ((FCIDM_SHVIEW_DELETE) as u16),
        ((FCIDM_SHVIEW_RENAME) as u16),
        ((FCIDM_SHVIEW_PROPERTIES) as u16),
        ((FCIDM_SHVIEW_CREATELINK) as u16),
        ((FCIDM_SHVIEW_CUT) as u16),
        ((FCIDM_SHVIEW_COPY) as u16),
        ((FCIDM_SHVIEW_INSERT) as u16),
    ];

    let mut i: usize = 0;
    while TryMapVerb && i < (verbmap).len() {
        if cmi.lpVerb == MAKEINTRESOURCEA(verbmap[i] as UINT) {
            let pszverbA = MapFcidmCmdToVerb(cmi.lpVerb as UINT_PTR);
            if !pszverbA.is_null() {
                SHAnsiToUnicode(pszverbA, szverbW.as_mut_ptr(), (((szverbW).len()) as i32));
                cmi.lpVerb = pszverbA;
                cmi.lpVerbW = szverbW.as_ptr();
                break;
            }
        }
        i += 1;
    }

    if GetKeyState(((VK_SHIFT) as i32)) < 0 {
        cmi.fMask |= CMIC_MASK_SHIFT_DOWN;
    }

    if GetKeyState(((VK_CONTROL) as i32)) < 0 {
        cmi.fMask |= CMIC_MASK_CONTROL_DOWN;
    }

    if !pt.is_null() {
        cmi.fMask |= CMIC_MASK_PTINVOKE;
        cmi.ptInvoke = *pt;
    }

    let mut szDirW: [u16; ((MAX_PATH) as usize)] = [0; ((MAX_PATH) as usize)];
    let _ = CPP_COM_WSTR_96A296D224F2.as_ptr();
    let mut szDirA: [i8; ((MAX_PATH) as usize)] = [0; ((MAX_PATH) as usize)];

    if CDefView__DoFolderViewCB_body(
        this,
        SFVM_GETCOMMANDDIR,
        (szDirW).len() as WPARAM,
        szDirW.as_mut_ptr() as LPARAM,
    ) >= 0
        && szDirW[0] != UNICODE_NULL
    {
        SHUnicodeToAnsi(szDirW.as_ptr(), szDirA.as_mut_ptr(), (((szDirA).len()) as i32));
        cmi.fMask |= CMIC_MASK_UNICODE;
        cmi.lpDirectory = szDirA.as_ptr();
        cmi.lpDirectoryW = szDirW.as_ptr();
    }

    let pContext: *mut IContextMenu = *pCM;
    let hr = ((*(*pContext).lpVtbl).InvokeCommand)(
        pContext,
        core::ptr::addr_of_mut!(cmi).cast(),
    );

    IUnknown_SetSite(pContext.cast(), core::ptr::null_mut());
    ((*(**pCM).lpVtbl).Release)(*pCM);

    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    S_OK
}

pub unsafe fn CDefView_OpenSelectedItems_body(this: *mut CDefView, pszVerb: PCSTR) -> HRESULT {
    let mut pszVerb = pszVerb;

    if ((*this).m_ListView).GetSelectedCount() == 0 {
        return S_OK;
    }

    let mut hResult = CDefView_OnDefaultCommand_body(this);
    if hResult == S_OK {
        return hResult;
    }

    let mut hMenu = CreatePopupMenu();
    if hMenu.is_null() {
        return E_FAIL;
    }

    let mut pCM: *mut IContextMenu = core::ptr::null_mut();
    let _pCM_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pCM));
    let (riid, ppv): (REFIID, *mut *mut core::ffi::c_void) =
        (core::ptr::addr_of!(IID_IContextMenu), &mut pCM as *mut _ as *mut *mut core::ffi::c_void);
    hResult = CDefView_GetItemObject_body(this, SVGIO_SELECTION, riid, ppv);
    let _menu_cleanup = MenuCleanup::new(&mut pCM as *mut _, &mut hMenu);
    if ((hResult) < 0) {
        return hResult;
    }

    let cmf = CMF_DEFAULTONLY | GetContextMenuFlags_body((*this).m_pShellBrowser, 0);
    let pCMRaw = pCM;
    hResult = ((*(*pCMRaw).lpVtbl).QueryContextMenu)(
        pCMRaw,
        hMenu.cast(),
        0,
        DVIDM_CONTEXTMENU_FIRST,
        DVIDM_CONTEXTMENU_LAST,
        cmf
    );
    if ((hResult) < 0) {
        return hResult;
    }

    let uCommand = GetMenuDefaultItem(hMenu, ((FALSE) as u32), 0);
    if uCommand == (1 as UINT).wrapping_neg() && pszVerb.is_null() {
        return E_FAIL;
    }
    if pszVerb.is_null() {
        pszVerb = MAKEINTRESOURCEA(uCommand);
    }

    let _ = CDefView_InvokeContextMenuCommand_body(this, &mut pCM, pszVerb, core::ptr::null_mut(), true);

    return hResult;
}

pub unsafe fn CDefView_OnContextMenu_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let mut pt = POINT {
        x: GET_X_LPARAM(lParam),
        y: GET_Y_LPARAM(lParam),
    };
    let mut hResult: HRESULT;

    if !(*this).m_hContextMenu.is_null() {
        return 0;
    }

    let hWndHdr = ListView_GetHeader((*this).m_ListView.m_hWnd);
    let mut r: RECT = core::mem::zeroed();
    if GetWindowRect(hWndHdr, &mut r) != 0
        && PtInRect(&r, pt) != 0
        && IsWindowVisible(hWndHdr) != 0
    {
        return CDefView_DoColumnContextMenu_body(this, lParam);
    }

    (*this).m_hContextMenu = CreatePopupMenu();
    if (*this).m_hContextMenu.is_null() {
        return E_FAIL as LRESULT;
    }

    if lParam != (!0 as LPARAM) {
        let mut hittest: LV_HITTESTINFO = core::mem::zeroed();
        hittest.pt = pt;
        ScreenToClient((*this).m_ListView.m_hWnd, &mut hittest.pt);
        (*this).m_ListView.HitTest(&mut hittest);

        if (hittest.flags & LVHT_ONITEM) != 0
            && (*this).m_ListView.GetItemState(hittest.iItem, LVIS_SELECTED)
                != LVIS_SELECTED
        {
            CDefView_SelectItem_v2_body(
                this,
                hittest.iItem,
                SVSI_SELECT | SVSI_DESELECTOTHERS | SVSI_ENSUREVISIBLE,
            );
        }
    }

    let count = (*this).m_ListView.GetSelectedCount();
    let uItem: UINT = if count != 0 {
        SVGIO_SELECTION
    } else {
        SVGIO_BACKGROUND
    };

    hResult = CDefView_GetItemObject_body(
        this,
        uItem,
        core::ptr::addr_of!(IID_IContextMenu),
        core::ptr::addr_of_mut!((*this).m_pCM).cast(),
    );
    let mut _menu_cleanup = MenuCleanup::new(&mut (*this).m_pCM as *mut _, &mut (*this).m_hContextMenu);
    if ((hResult) < 0) as i32 != 0 {
        return 0;
    }

    let cmf = GetContextMenuFlags_body(
        (*this).m_pShellBrowser,
        CDefView_GetSelectionAttributes_body(this, SFGAO_CANRENAME),
    );

    let pContextMenu = (*this).m_pCM;
    hResult = ((*(*pContextMenu).lpVtbl).QueryContextMenu)(
        pContextMenu,
        (*this).m_hContextMenu.cast(),
        0,
        CONTEXT_MENU_BASE_ID,
        DVIDM_CONTEXTMENU_LAST,
        cmf,
    );
    if ((hResult) < 0) as i32 != 0 {
        return 0;
    }

    if !(*this).m_pCommDlgBrowser.is_null()
        && (CDefView_GetCommDlgViewFlags_body(this) & CDB2GVF_NOSELECTVERB) == 0
    {
        let hMenuSource = LoadMenuW(
            ((crate::win32::shell32_hInstance) as _),
            MAKEINTRESOURCEW(IDM_DVSELECT),
        );
        Shell_MergeMenus(
            (*this).m_hContextMenu.cast(),
            GetSubMenu(hMenuSource, 0).cast(),
            0,
            DVIDM_COMMDLG_SELECT,
            0xffff,
            MM_ADDSEPARATOR | MM_DONTREMOVESEPS,
        );
        DestroyMenu(hMenuSource);
        SetMenuDefaultItem(
            (*this).m_hContextMenu,
            DVIDM_COMMDLG_SELECT,
            MF_BYCOMMAND,
        );
    }

    if lParam == (!0 as LPARAM) {
        let hFocus = GetFocus();
        let mut lvIndex: i32 = -(1 as i32);

        if hFocus == (*this).m_ListView.m_hWnd || (*this).m_ListView.IsChild(hFocus) != 0 {
            lvIndex = (*this).m_ListView.GetNextItem(
                -(1 as i32),
                ((LVNI_SELECTED | LVNI_FOCUSED) as u16),
            );
            if lvIndex < 0 as i32 {
                lvIndex = (*this).m_ListView.GetNextItem(
                    -(1 as i32),
                    ((LVNI_SELECTED) as u16),
                );
            }
        }

        if lvIndex > -(1 as i32) {
            let mut rc: RECT = core::mem::zeroed();
            rc.left = ((LVIR_ICON) as i32);
            SendMessage(
                (*this).m_ListView.m_hWnd,
                LVM_GETITEMRECT,
                lvIndex as WPARAM,
                &mut rc as *mut RECT as LPARAM,
            );
            pt.x = (rc.right + rc.left) / 2;
            pt.y = (rc.bottom + rc.top) / 2;
        } else {
            pt.x = 0;
            pt.y = 0;
        }

        (*this).m_ListView.ClientToScreen(&mut pt);
    }

    let mut pcdb2: *mut ICommDlgBrowser2 = core::ptr::null_mut();
    let _pcdb2_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pcdb2));
    if !(*this).m_pCommDlgBrowser.is_null() {
        let pCommDlgBrowser = (*this).m_pCommDlgBrowser;
        let hr_qi = ((*(*pCommDlgBrowser).lpVtbl).QueryInterface)(
            pCommDlgBrowser,
            core::ptr::addr_of!(IID_ICommDlgBrowser2),
            core::ptr::addr_of_mut!(pcdb2).cast(),
        );
        if hr_qi >= 0 {
            let pcdb2_ptr = pcdb2;
            ((*(*pcdb2_ptr).lpVtbl).Notify)(
                pcdb2_ptr,
                this as *mut IShellView,
                CDB2N_CONTEXTMENU_START,
            );
        }
    }

    let uCommand: UINT = TrackPopupMenu(
        (*this).m_hContextMenu,
        TPM_LEFTALIGN | TPM_RETURNCMD | TPM_LEFTBUTTON | TPM_RIGHTBUTTON,
        pt.x,
        pt.y,
        0,
        (*this).m_hWnd,
        core::ptr::null::<RECT>(),
    ) as UINT;

    if uCommand >= DVIDM_ARRANGESORT_FIRST && uCommand <= DVIDM_ARRANGESORT_LAST {
        SendMessage(
            (*this).m_hWnd,
            WM_COMMAND,
            uCommand as WPARAM,
            0,
        );
    } else if uCommand != 0
        && !(uCommand == DVIDM_COMMDLG_SELECT
            && CDefView_OnDefaultCommand_body(this) == S_OK)
    {
        CDefView_InvokeContextMenuCommand_body(
            this,
            &mut (*this).m_pCM,
            MAKEINTRESOURCEA(uCommand - (CONTEXT_MENU_BASE_ID as UINT)),
            &mut pt,
            true,
        );
    }

    if !pcdb2.is_null() {
        let pcdb2_ptr = pcdb2;
        ((*(*pcdb2_ptr).lpVtbl).Notify)(
            pcdb2_ptr,
            this as *mut IShellView,
            CDB2N_CONTEXTMENU_DONE,
        );
    }

    0
}

pub unsafe fn CDefView_OnExplorerCommand_body(this: *mut CDefView, uCommand: UINT, bUseSelection: BOOL) -> LRESULT {
    let mut hResult: HRESULT;
    let mut hMenu: HMENU = core::ptr::null_mut();
    let mut pCM: *mut IContextMenu = core::ptr::null_mut();
    let _pCM_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pCM));

    hResult = CDefView_GetItemObject_body(
        this,
        if bUseSelection != 0 { SVGIO_SELECTION } else { SVGIO_BACKGROUND },
        core::ptr::addr_of!(IID_IContextMenu), core::ptr::addr_of_mut!(pCM).cast(),
    );
    if ((hResult) < 0) {
        return 0;
    }

    let mut _cleanup = MenuCleanup::new(&mut pCM as *mut _, &mut hMenu);

    if uCommand != FCIDM_SHVIEW_DELETE && uCommand != FCIDM_SHVIEW_RENAME {
        hMenu = CreatePopupMenu();
        if hMenu.is_null() {
            return 0;
        }

        let pCM_raw = pCM;
        hResult = ((*(*pCM_raw).lpVtbl).QueryContextMenu)(
            pCM_raw,
            hMenu.cast(),
            0,
            DVIDM_CONTEXTMENU_FIRST,
            DVIDM_CONTEXTMENU_LAST,
            CMF_NORMAL,
        );
        if ((hResult) < 0) {
            return 0;
        }
    }

    if bUseSelection != 0 {
        let mut rfg: SFGAOF = SFGAO_BROWSABLE
            | SFGAO_CANCOPY
            | SFGAO_CANLINK
            | SFGAO_CANMOVE
            | SFGAO_CANDELETE
            | SFGAO_CANRENAME
            | SFGAO_HASPROPSHEET
            | SFGAO_FILESYSTEM
            | SFGAO_FOLDER;

        let pParent = (*this).m_pSFParent;
        hResult = ((*(*pParent).lpVtbl).GetAttributesOf)(
            pParent,
            (*this).m_cidl,
            (((*this).m_apidl) as _),
            &mut rfg,
        );
        if ((hResult) < 0) {
            return 0;
        }

        if (rfg & SFGAO_CANMOVE) == 0 && uCommand == FCIDM_SHVIEW_CUT {
            return 0;
        }
        if (rfg & SFGAO_CANCOPY) == 0 && uCommand == FCIDM_SHVIEW_COPY {
            return 0;
        }
        if (rfg & SFGAO_CANDELETE) == 0 && uCommand == FCIDM_SHVIEW_DELETE {
            return 0;
        }
        if (rfg & SFGAO_CANRENAME) == 0 && uCommand == FCIDM_SHVIEW_RENAME {
            return 0;
        }
        if (rfg & SFGAO_HASPROPSHEET) == 0 && uCommand == FCIDM_SHVIEW_PROPERTIES {
            return 0;
        }
    }

    CDefView_InvokeContextMenuCommand_body(
        this,
        &mut pCM,
        MAKEINTRESOURCEA(uCommand),
        core::ptr::null_mut(),
        true,
    );
    return 0;
}

pub unsafe fn CDefView_OnSize_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let mut wWidth: u16;
    let mut wHeight: u16;

    wWidth = LOWORD!(lParam);
    wHeight = HIWORD(lParam as DWORD);

    // C++ `if (!m_ListView)` goes through CWindow's `operator HWND`.
    if (*this).m_ListView.m_hWnd.is_null() {
        return 0;
    }

    MoveWindow((*this).m_ListView.m_hWnd, 0, 0, wWidth as i32, wHeight as i32, TRUE);

    CDefView__DoFolderViewCB_body(this, SFVM_SIZE, 0, 0);

    CDefView__ForceStatusBarResize_body(this);
    CDefView_UpdateStatusbar_body(this);

    return 0;
}

pub unsafe fn CDefView_OnDeactivate_body(this: *mut CDefView) -> () {
    if (*this).m_uState != SVUIA_DEACTIVATE {
        // TODO: cleanup menu after deactivation
        (*this).m_uState = SVUIA_DEACTIVATE;
    }
}

pub unsafe fn CDefView_DoActivate_body(this: *mut CDefView, uState: UINT) -> () {
    if (*this).m_uState == uState {
        return;
    }

    if uState == SVUIA_DEACTIVATE {
        CDefView_OnDeactivate_body(this);
    } else {
        if !(*this).m_hMenu.is_null() && (*this).m_bmenuBarInitialized == 0 {
            CDefView_FillEditMenu_body(this);
            CDefView_FillViewMenu_body(this);
            ((*(*(*this).m_pShellBrowser).lpVtbl).SetMenuSB)(
                (*this).m_pShellBrowser,
                (*this).m_hMenu,
                core::ptr::null_mut(),
                (*this).m_hWnd,
            );
            (*this).m_bmenuBarInitialized = TRUE;
        }

        if SVUIA_ACTIVATE_FOCUS == uState {
            ((*this).m_ListView).SetFocus();
        }
    }

    (*this).m_uState = uState;
}

pub unsafe fn CDefView__DoCopyToMoveToFolder_body(this: *mut CDefView, bCopy: BOOL) -> () {
    if CDefView_GetSelections_body(this) == 0 {
        return;
    }

    let mut rfg: SFGAOF = SFGAO_CANCOPY | SFGAO_CANMOVE | SFGAO_FILESYSTEM;
    let parent = (*this).m_pSFParent;
    let mut hr: HRESULT = ((*(*parent).lpVtbl).GetAttributesOf)(
        parent,
        (*this).m_cidl,
        (((*this).m_apidl) as _),
        &mut rfg,
    );
    if ((hr) < 0) {
        return;
    }

    if bCopy == 0 && (rfg & SFGAO_CANMOVE) == 0 {
        return;
    }
    if bCopy != 0 && (rfg & SFGAO_CANCOPY) == 0 {
        return;
    }

    let mut pCM: *mut IContextMenu = core::ptr::null_mut();
    let _pCM_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pCM));
    hr = ((*(*parent).lpVtbl).GetUIObjectOf)(
        parent,
        (*this).m_hWnd.cast(),
        (*this).m_cidl,
        (*this).m_apidl,
        &IID_IContextMenu,
        core::ptr::null_mut(),
        &mut pCM as *mut *mut IContextMenu as *mut *mut core::ffi::c_void,
    );
    if ((hr) < 0) {
        return;
    }

    CDefView_InvokeContextMenuCommand_body(
        this,
        &mut pCM,
        if bCopy != 0 {
            b"copyto\0".as_ptr() as LPCSTR
        } else {
            b"moveto\0".as_ptr() as LPCSTR
        },
        core::ptr::null_mut(),
        false,
    );
}

pub unsafe fn CDefView_OnActivate_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    CDefView_DoActivate_body(this, SVUIA_ACTIVATE_FOCUS);
    0
}

pub unsafe fn CDefView_OnSetFocus_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    ((*(*(*this).m_pShellBrowser).lpVtbl).OnViewWindowActive)((*this).m_pShellBrowser, this as *mut _);
    CDefView_DoActivate_body(this, SVUIA_ACTIVATE_FOCUS);
    ((*this).m_ListView).SetFocus();
    CDefView_OnStateChange_body(this, CDBOSC_SETFOCUS);
    0
}

pub unsafe fn CDefView_OnKillFocus_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    CDefView_DoActivate_body(this, SVUIA_ACTIVATE_NOFOCUS);
    /* Notify the ICommDlgBrowser */
    CDefView_OnStateChange_body(this, CDBOSC_KILLFOCUS);

    return 0;
}

pub unsafe fn CDefView_OnCommand_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let dwCmdID = GET_WM_COMMAND_ID(wParam, lParam);
    let dwCmd = GET_WM_COMMAND_CMD(wParam, lParam);
    let hwndCmd = GET_WM_COMMAND_HWND(wParam, lParam);

    if dwCmdID >= ((DVIDM_ARRANGESORT_FIRST) as u16) && dwCmdID <= ((DVIDM_ARRANGESORT_LAST) as u16) {
        let listCol: UINT = GetMenuItemDataById_body((*this).m_hMenuArrangeModes, ((dwCmdID) as u32)) as UINT;
        CDefView__Sort_body(this, ((listCol) as i32));
        return 0;
    }

    match dwCmdID as u32 {
        FCIDM_SHVIEW_BIGICON => {
            const _: () = assert!(FCIDM_SHVIEW_BIGICON == 0x7029);
            CDefView_SetCurrentViewMode_body(this, ((dwCmdID.wrapping_sub(((FCIDM_SHVIEW_BIGICON) as u16)).wrapping_add(1)) as u32));
        }
        FCIDM_SHVIEW_SMALLICON => {
            const _: () = assert!(FCIDM_SHVIEW_SMALLICON == 0x702A);
            CDefView_SetCurrentViewMode_body(this, ((dwCmdID.wrapping_sub(((FCIDM_SHVIEW_BIGICON) as u16)).wrapping_add(1)) as u32));
        }
        FCIDM_SHVIEW_LISTVIEW => {
            const _: () = assert!(FCIDM_SHVIEW_LISTVIEW == 0x702B);
            CDefView_SetCurrentViewMode_body(this, ((dwCmdID.wrapping_sub(((FCIDM_SHVIEW_BIGICON) as u16)).wrapping_add(1)) as u32));
        }
        FCIDM_SHVIEW_REPORTVIEW => {
            const _: () = assert!(FCIDM_SHVIEW_REPORTVIEW == 0x702C);
            CDefView_SetCurrentViewMode_body(this, ((dwCmdID.wrapping_sub(((FCIDM_SHVIEW_BIGICON) as u16)).wrapping_add(1)) as u32));
        }
        0x702D => {
            CDefView_SetCurrentViewMode_body(this, ((dwCmdID.wrapping_sub(((FCIDM_SHVIEW_BIGICON) as u16)).wrapping_add(1)) as u32));
        }
        FCIDM_SHVIEW_TILEVIEW => {
            const _: () = assert!(FCIDM_SHVIEW_TILEVIEW == 0x702E);
            CDefView_SetCurrentViewMode_body(this, ((dwCmdID.wrapping_sub(((FCIDM_SHVIEW_BIGICON) as u16)).wrapping_add(1)) as u32));
        }
        0x702F => {
            CDefView_SetCurrentViewMode_body(this, ((dwCmdID.wrapping_sub(((FCIDM_SHVIEW_BIGICON) as u16)).wrapping_add(1)) as u32));
        }
        FCIDM_SHVIEW_SNAPTOGRID => {
            (*this).m_ListView.Arrange(LVA_SNAPTOGRID);
        }
        FCIDM_SHVIEW_ALIGNTOGRID => {
            if CDefView__GetSnapToGrid_body(this) == S_OK {
                (*this).m_ListView.SetExtendedListViewStyle(0, LVS_EX_SNAPTOGRID);
            } else {
                CDefView_ArrangeGrid_body(this);
            }
        }
        FCIDM_SHVIEW_AUTOARRANGE => {
            if CDefView_GetAutoArrange_body(this) == S_OK {
                (*this).m_ListView.ModifyStyle(LVS_AUTOARRANGE, 0, 0); // atlwin.h:1041 `nFlags = 0`
            } else {
                CDefView_AutoArrange_body(this);
            }
        }
        FCIDM_SHVIEW_SELECTALL => {
            if CDefView__DoFolderViewCB_body(this, SFVM_CANSELECTALL, 0, 0) != S_FALSE {
                (*this).m_ListView.SetItemState(-1, LVIS_SELECTED, LVIS_SELECTED);
            }
        }
        FCIDM_SHVIEW_INVERTSELECTION => {
            let nCount = (*this).m_ListView.GetItemCount();
            let mut i = 0;
            while i < nCount {
                (*this).m_ListView.SetItemState(i, (*this).m_ListView.GetItemState(i, LVIS_SELECTED) ^ LVIS_SELECTED, LVIS_SELECTED);
                i += 1;
            }
        }
        FCIDM_SHVIEW_DESELECTALL => {
            (*this).m_ListView.SetItemState(-1, 0, LVIS_SELECTED);
        }
        FCIDM_SHVIEW_REFRESH => {
            CDefView_Refresh_body(this);
        }
        FCIDM_SHVIEW_DELETE | FCIDM_SHVIEW_CUT | FCIDM_SHVIEW_COPY | FCIDM_SHVIEW_RENAME | FCIDM_SHVIEW_PROPERTIES => {
            if ((SHRestricted(REST_NOVIEWCONTEXTMENU)) != 0) {
                return 0;
            }
            return CDefView_OnExplorerCommand_body(this, ((dwCmdID) as u32), TRUE);
        }
        FCIDM_SHVIEW_COPYTO => {
            const _: () = assert!(FCIDM_SHVIEW_COPYTO == 0x701e);
            CDefView__DoCopyToMoveToFolder_body(this, ((dwCmdID == ((FCIDM_SHVIEW_COPYTO) as u16)) as i32));
            return 0;
        }
        FCIDM_SHVIEW_MOVETO => {
            const _: () = assert!(FCIDM_SHVIEW_MOVETO == 0x701f);
            CDefView__DoCopyToMoveToFolder_body(this, ((dwCmdID == ((FCIDM_SHVIEW_COPYTO) as u16)) as i32));
            return 0;
        }
        FCIDM_SHVIEW_INSERT | FCIDM_SHVIEW_UNDO | FCIDM_SHVIEW_INSERTLINK | FCIDM_SHVIEW_NEWFOLDER => {
            return CDefView_OnExplorerCommand_body(this, ((dwCmdID) as u32), FALSE);
        }
        _ => {
            if !((*this).m_pFileMenu).is_null() && dwCmd == 0 {
                let mut Dummy: HMENU = core::ptr::null_mut();
                let mut _guard = MenuCleanup::new(((core::ptr::addr_of_mut!((*this).m_pFileMenu)) as _), core::ptr::addr_of_mut!(Dummy));
                CDefView_InvokeContextMenuCommand_body(this, &mut (*this).m_pFileMenu, MAKEINTRESOURCEA(((dwCmdID) as u32)), core::ptr::null_mut(), false); // CDefView.cpp:361 `TryMapVerb = false`
            }
        }
    }

    return 0;
}

pub unsafe fn SelectExtOnRename_body() -> BOOL {
    unsafe {
        let mut hKey: HKEY = core::mem::zeroed();
        let mut dwValue: DWORD = ((FALSE) as u32);
        let mut cbValue: DWORD;

        let error: LONG = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            REGSTR_PATH_EXPLORER.as_ptr(),
            0,
            KEY_READ,
            &mut hKey,
        );
        if error != 0 {
            return dwValue as BOOL;
        }

        cbValue = core::mem::size_of::<DWORD>() as DWORD;
        RegQueryValueExW(
            hKey,
            CPP_COM_WSTR_EE1C560F63B6.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwValue).cast::<u8>(),
            &mut cbValue,
        );

        RegCloseKey(hKey);
        return (dwValue != 0) as BOOL;
    }
}

// commctrl.h's NM_* codes, as the INT that this switch reads `NMHDR::code` as -- the same
// signedness the crate already gives the LVN_*/HDN_* codes it is matched against.
//
// FOUR of them (NM_SETFOCUS, NM_KILLFOCUS, NM_RCLICK, NM_RELEASEDCAPTURE) existed NOWHERE in
// the crate's dependency closure, which does not fail to compile: an undefined name in a
// `match` pattern is a catch-all BINDING. NM_SETFOCUS is the first arm, so it swallowed every
// notification this view receives and made all seventeen arms after it dead code. That is the
// §9.112 class and `verify_match_consts` flags it HIGH. The other four exist as UINT for
// dialogs/view.rs and COpenWithDialog.rs; these locals shadow the glob for THIS file only, so
// those call sites keep their own spelling.
//
// `#define NM_FIRST (0U-0U)` means each of these is an unsigned wraparound -- NM_SETFOCUS is
// 0xFFFFFFF9 -- so the composing expression is kept and cast, not hand-evaluated to -7.
const NM_CLICK: INT = crate::island::NM_FIRST.wrapping_sub(2) as INT;
const NM_DBLCLK: INT = crate::island::NM_FIRST.wrapping_sub(3) as INT;
const NM_RETURN: INT = crate::island::NM_FIRST.wrapping_sub(4) as INT;
const NM_RCLICK: INT = crate::island::NM_FIRST.wrapping_sub(5) as INT;
const NM_SETFOCUS: INT = crate::island::NM_FIRST.wrapping_sub(7) as INT;
const NM_KILLFOCUS: INT = crate::island::NM_FIRST.wrapping_sub(8) as INT;
const NM_CUSTOMDRAW: INT = crate::island::NM_FIRST.wrapping_sub(12) as INT;
const NM_RELEASEDCAPTURE: INT = crate::island::NM_FIRST.wrapping_sub(16) as INT;

pub unsafe fn CDefView_OnNotify_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let _ctl_id: UINT = wParam as UINT;
    let lpnmh: *mut NMHDR = lParam as *mut NMHDR;
    let lpnmlv: *mut NMLISTVIEW = lpnmh as *mut NMLISTVIEW;
    let lpdi: *mut NMLVDISPINFOW = lpnmh as *mut NMLVDISPINFOW;
    let mut pidl: PCUITEMID_CHILD;
    let mut unused: BOOL = 0;

    match (*lpnmh).code as i32 {
        NM_SETFOCUS => {
            CDefView_OnSetFocus_body(this, 0, 0, 0, &mut unused);
        }
        NM_KILLFOCUS => {
            CDefView_OnDeactivate_body(this);
            CDefView_OnStateChange_body(this, CDBOSC_KILLFOCUS);
        }
        NM_CUSTOMDRAW => {
            return CDRF_DODEFAULT as LRESULT;
        }
        NM_RELEASEDCAPTURE => {}
        NM_CLICK => {}
        NM_RCLICK => {}
        NM_DBLCLK => {}
        NM_RETURN => {}
        HDN_ENDTRACKW => {}
        LVN_DELETEITEM => {
            SHFree((*lpnmlv).lParam as *mut core::ffi::c_void);
        }
        LVN_DELETEALLITEMS => {
            return FALSE as LRESULT;
        }
        LVN_INSERTITEM => {}
        LVN_ITEMACTIVATE => {
            let pszVerb = if (*(lpnmh as *mut NMITEMACTIVATE)).uKeyFlags & LVKF_ALT != 0 {
                b"properties\0".as_ptr() as PCSTR
            } else {
                core::ptr::null()
            };
            CDefView_OpenSelectedItems_body(this, pszVerb);
        }
        LVN_COLUMNCLICK => {
            let foldercol: UINT = CDefView_MapListColumnToFolderColumn_body(this, (((*lpnmlv).iSubItem) as u32)) as UINT;
            let mut hr: HRESULT = S_FALSE;
            if !(*this).m_pSDParent.is_null() {
                hr = ((*(*(*this).m_pSDParent).lpVtbl).ColumnClick)((*this).m_pSDParent, foldercol);
            }
            if hr != S_OK {
                hr = CDefView__DoFolderViewCB_body(this, SFVM_COLUMNCLICK, foldercol as WPARAM, 0);
            }
            if hr != S_OK {
                CDefView__Sort_body(this, (*lpnmlv).iSubItem);
            }
        }
        // The C++ `break`s out of the SWITCH partway through this case (CDefView.cpp:2759),
        // which a `match` arm has no keyword for -- hence the labelled block.
        LVN_GETDISPINFOA | LVN_GETDISPINFOW => 'getdispinfo: {
            pidl = CDefView__PidlByItem_2_body(this, &mut (*lpdi).item);

            if (*lpdi).item.mask & LVIF_TEXT != 0 {
                let mut sd: SHELLDETAILS = core::mem::zeroed();
                if FAILED_UNEXPECTEDLY!(CDefView_GetDetailsByListColumn_body(this, pidl, (((*lpdi).item.iSubItem) as u32), &mut sd)) {
                    break 'getdispinfo;
                }

                if (*lpnmh).code == ((LVN_GETDISPINFOA) as u32) {
                    let lpdi_a = lpnmh as *mut NMLVDISPINFOA;
                    StrRetToStrNA(
                        (*lpdi_a).item.pszText,
                        (((*lpdi_a).item.cchTextMax) as u32),
                        core::ptr::addr_of_mut!(sd.str_).cast(),
                        core::ptr::null_mut(),
                    );
                } else {
                    StrRetToStrNW(
                        (*lpdi).item.pszText,
                        (((*lpdi).item.cchTextMax) as u32),
                        core::ptr::addr_of_mut!(sd.str_).cast(),
                        core::ptr::null_mut(),
                    );
                }
            }
            if (*lpdi).item.mask & LVIF_IMAGE != 0 {
                (*lpdi).item.iImage = SHMapPIDLToSystemImageListIndex((*this).m_pSFParent, pidl, core::ptr::null_mut()) as i32;
            }
            if (*lpdi).item.mask & LVIF_STATE != 0 {
                if ((*lpdi).item.stateMask & LVIS_CUT != 0)
                    && CDefView_GetItemAttributes_body(this, pidl, SFGAO_HIDDEN | SFGAO_GHOSTED) != 0
                {
                    (*lpdi).item.state |= LVIS_CUT;
                }
            }
            (*lpdi).item.mask |= LVIF_DI_SETITEM;
        }
        LVN_ITEMCHANGED => {
            if ((*lpnmlv).uOldState ^ (*lpnmlv).uNewState) & (LVIS_SELECTED | LVIS_FOCUSED) != 0 {
                CDefView_OnStateChange_body(this, CDBOSC_SELCHANGE);
                if !(*this).m_ScheduledStatusbarUpdate
                    && ((*this).m_iDragOverItem == -(1 as i32)
                        // CDefView.cpp:2791 `m_pCurDropTarget == NULL` -- a null TEST, which
                        // came back as a call to the CComPtr ASSIGNMENT helper.
                        || (*this).m_pCurDropTarget.is_null())
                {
                    (*this).m_ScheduledStatusbarUpdate = true;
                    PostMessageW((*this).m_hWnd, SHV_UPDATESTATUSBAR, 0, 0);
                }
                CDefView__DoFolderViewCB_body(this, SFVM_SELECTIONCHANGED, 0, 0);
            }
        }
        LVN_BEGINDRAG | LVN_BEGINRDRAG => {
            if CDefView_GetSelections_body(this) != 0 {
                let mut pda: *mut IDataObject = core::ptr::null_mut();
                let _pda_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pda));
                let mut dwAttributes: DWORD = SFGAO_CANCOPY | SFGAO_CANLINK;
                let mut dwEffect: DWORD = DROPEFFECT_MOVE;

                let hr_ui = ((*(*(*this).m_pSFParent).lpVtbl).GetUIObjectOf)(
                    (*this).m_pSFParent,
                    (*this).m_hWnd.cast(),
                    (*this).m_cidl,
                    (*this).m_apidl,
                    core::ptr::addr_of!(IID_IDataObject), core::ptr::null_mut(), core::ptr::addr_of_mut!(pda).cast(),
                );
                if hr_ui >= 0 {
                    let params = lParam as *mut NMLISTVIEW;

                    let hr_attr = ((*(*(*this).m_pSFParent).lpVtbl).GetAttributesOf)(
                        (*this).m_pSFParent,
                        (*this).m_cidl,
                        (((*this).m_apidl) as _),
                        &mut dwAttributes,
                    );
                    if hr_attr >= 0 {
                        dwEffect |= dwAttributes & (SFGAO_CANCOPY | SFGAO_CANLINK);
                    }

                    let mut piaso: *mut IAsyncOperation = core::ptr::null_mut();
                    let _piaso_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(piaso));
                    let hr_qi = ((*(*pda).lpVtbl).QueryInterface)(pda, core::ptr::addr_of!(IID_IAsyncOperation), core::ptr::addr_of_mut!(piaso).cast());
                    if hr_qi >= 0 {
                        ((*(*piaso).lpVtbl).SetAsyncMode)(piaso, TRUE);
                    }

                    let mut dwEffect2: DWORD = 0;
                    CDefView_cpp_com_assign_m_pSourceDataObject(this, pda);
                    (*this).m_ptFirstMousePos = (*params).ptAction;
                    ClientToScreen((*this).m_hWnd, &mut (*this).m_ptFirstMousePos);
                    ClientToListView_body((*this).m_ListView.m_hWnd, &mut (*this).m_ptFirstMousePos);

                    let mut big_icons: HIMAGELIST = core::ptr::null_mut();
                    let mut small_icons: HIMAGELIST = core::ptr::null_mut();
                    Shell_GetImageLists(&mut big_icons, &mut small_icons);
                    let pidl_drag = CDefView__PidlByItem_body(this, (*params).iItem);
                    let iIcon = SHMapPIDLToSystemImageListIndex((*this).m_pSFParent, pidl_drag, core::ptr::null_mut()) as i32;
                    let mut ptItem: POINT = core::mem::zeroed();
                    (*this).m_ListView.GetItemPosition((*params).iItem, &mut ptItem);

                    ImageList_BeginDrag(
                        big_icons,
                        iIcon,
                        (*params).ptAction.x - ptItem.x,
                        (*params).ptAction.y - ptItem.y,
                    );
                    DoDragDrop(pda.cast(), this.cast(), dwEffect, &mut dwEffect2);

                    if !(*this).m_pSourceDataObject.is_null() {
                        ((*(*(*this).m_pSourceDataObject).lpVtbl).Release)((*this).m_pSourceDataObject);
                        (*this).m_pSourceDataObject = core::ptr::null_mut();
                    }

                    if !piaso.is_null() {
                        ((*(*piaso).lpVtbl).Release)(piaso);
                        piaso = core::ptr::null_mut();
                    }
                }

                if !pda.is_null() {
                    ((*(*pda).lpVtbl).Release)(pda);
                    pda = core::ptr::null_mut();
                }
            }
        }
        LVN_BEGINLABELEDITW => {
            let hEdit: HWND = ListView_GetEditControl((*this).m_ListView.m_hWnd);
            pidl = CDefView__PidlByItem_2_body(this, &mut (*lpdi).item);
            let fAttr: DWORD = if !pidl.is_null() {
                CDefView_GetItemAttributes_body(this, pidl, SFGAO_CANRENAME | SFGAO_FOLDER | SFGAO_FILESYSTEM)
            } else {
                0
            };

            if hEdit.is_null() || (fAttr & SFGAO_CANRENAME) == 0 {
                MessageBeep(0xffffffff);
                return TRUE as LRESULT;
            }

            let mut szName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
            let mut pszText: PWSTR = (*lpdi).item.pszText;

            if DisplayNameOfW(
                (*this).m_pSFParent,
                pidl,
                SHGDN_FOREDITING | SHGDN_INFOLDER,
                szName.as_mut_ptr(),
                ((szName.len()) as u32),
            ) >= 0
            {
                pszText = szName.as_mut_ptr();
                SetWindowTextW(hEdit, pszText);
            }

            if (fAttr & SFGAO_FOLDER) == 0
                && (fAttr & SFGAO_FILESYSTEM) != 0
                && (*lpdi).item.mask & LVIF_TEXT != 0
                && SelectExtOnRename_body() == 0
            {
                let mut pidl_full: *mut ITEMIDLIST_RELATIVE = ILCombine((*this).m_pidlParent, pidl);
                let mut szFullPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
                if SHGetPathFromIDListW(pidl_full, szFullPath.as_mut_ptr()) != 0
                    && SHELL_FS_HideExtension(szFullPath.as_mut_ptr()) == 0
                {
                    let pchDotExt = PathFindExtensionW(pszText);
                    PostMessageW(
                        hEdit,
                        EM_SETSEL,
                        0,
                        pchDotExt.offset_from(pszText) as LPARAM,
                    );
                    PostMessageW(
                        hEdit,
                        EM_SCROLLCARET,
                        0,
                        0,
                    );
                }
                if !pidl_full.is_null() {
                    ILFree(pidl_full);
                }
            }

            let mut cchLimit: INT = 0;
            CDefView__DoFolderViewCB_body(this, SFVM_GETNAMELENGTH, pidl as WPARAM, &mut cchLimit as *mut INT as LPARAM);
            if cchLimit != 0 {
                SendMessageW(hEdit, EM_SETLIMITTEXT, cchLimit as WPARAM, 0);
            }
            SHLimitInputEdit(((hEdit) as _), (*this).m_pSFParent);
            (*this).m_isEditing = TRUE;
            return FALSE as LRESULT;
        }
        LVN_ENDLABELEDITW => {
            (*this).m_isEditing = FALSE;
            if (*lpdi).item.pszText.is_null() {
                return TRUE as LRESULT;
            }

            pidl = CDefView__PidlByItem_2_body(this, &mut (*lpdi).item);
            let mut pidl_old: *mut ITEMIDLIST_ABSOLUTE = ILClone(pidl);
            if pidl_old.is_null() {
                SHELL_ErrorBox((*lpdi).hdr.hwndFrom.cast(), E_OUTOFMEMORY as UINT);
                return FALSE as LRESULT;
            }

            let mut pidl_new: PITEMID_CHILD = core::ptr::null_mut();
            let hr = ((*(*(*this).m_pSFParent).lpVtbl).SetNameOf)(
                (*this).m_pSFParent,
                core::ptr::null_mut(),
                pidl_old,
                (*lpdi).item.pszText,
                SHGDN_INFOLDER,
                &mut pidl_new,
            );

            if hr >= 0 && !pidl_new.is_null() {
                let iNew = CDefView_LV_FindItemByPidl_body(this, pidl_new);
                if iNew != (*lpdi).item.iItem && iNew >= 0 {
                    ILFree(pidl_new);
                } else if CDefView_LV_UpdateItem_body(this, (*lpdi).item.iItem, pidl_new) == FALSE {
                    ILFree(pidl_new);
                }
                CDefView_OnStateChange_body(this, CDBOSC_RENAME);
            } else {
                PostMessageW(
                    (*this).m_ListView.m_hWnd,
                    LVM_EDITLABEL,
                    (*lpdi).item.iItem as WPARAM,
                    0,
                );
            }

            if !pidl_old.is_null() {
                ILFree(pidl_old);
            }
            return FALSE as LRESULT;
        }
        _ => {}
    }

    return 0;
}

pub unsafe fn ILIsParentOrSpecialParent_body(pidl1: PCIDLIST_ABSOLUTE, pidl2: PCIDLIST_ABSOLUTE) -> BOOL {
    if pidl1.is_null() || pidl2.is_null() {
        return FALSE;
    }
    if ILIsParent(pidl1, pidl2, TRUE) != 0 {
        return TRUE;
    }

    let pidl2Clone: *mut ITEMIDLIST_ABSOLUTE = ILClone(pidl2);
    ILRemoveLastID(pidl2Clone);
    ILIsEqual(pidl1, pidl2Clone)
}

pub unsafe fn CDefView_OnChangeNotify_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let _ = &uMsg;
    let _ = &bHandled;

    if (*this).m_ListView.is_null() {
        return FALSE as LRESULT;
    }

    let h_change: HANDLE = wParam as HANDLE;
    let dw_proc_id: DWORD = lParam as DWORD;
    let mut pidls: *mut PIDLIST_ABSOLUTE = core::ptr::null_mut();
    let mut l_event: LONG = 0;
    let h_lock: HANDLE = SHChangeNotification_Lock(h_change, dw_proc_id, &mut pidls, &mut l_event);

    if h_lock.is_null() {
        return FALSE as LRESULT;
    }

    l_event &= !(SHCNE_INTERRUPT as LONG);

    if CDefView__DoFolderViewCB_body(this, SFVM_FSNOTIFY, pidls as usize as WPARAM, l_event as LPARAM) == S_FALSE {
        SHChangeNotification_Unlock(h_lock);
        return FALSE as LRESULT;
    }

    let mut hr: HRESULT = 0;
    let mut child0: PITEMID_CHILD = core::ptr::null_mut();
    let mut child1: PITEMID_CHILD = core::ptr::null_mut();
    let mut pidl0_temp: *mut ITEMIDLIST_RELATIVE = core::ptr::null_mut();
    let mut pidl1_temp: *mut ITEMIDLIST_RELATIVE = core::ptr::null_mut();

    if l_event != ((SHCNE_UPDATEIMAGE) as i32) && l_event < ((SHCNE_EXTENDED_EVENT) as i32) {
        if _ILIsSpecialFolder(*pidls.add(0) as PCIDLIST_ABSOLUTE) != 0
            || ILIsParentOrSpecialParent_body((*this).m_pidlParent as PCIDLIST_ABSOLUTE, *pidls.add(0) as PCIDLIST_ABSOLUTE) != 0
        {
            child0 = ILFindLastID(*pidls.add(0) as PCIDLIST_ABSOLUTE);
            hr = SHGetRealIDL((*this).m_pSFParent, child0, &mut pidl0_temp);
            if hr >= 0 {
                child0 = pidl0_temp as PITEMID_CHILD;
            }
        }

        if _ILIsSpecialFolder(*pidls.add(1) as PCIDLIST_ABSOLUTE) != 0
            || ILIsParentOrSpecialParent_body((*this).m_pidlParent as PCIDLIST_ABSOLUTE, *pidls.add(1) as PCIDLIST_ABSOLUTE) != 0
        {
            child1 = ILFindLastID(*pidls.add(1) as PCIDLIST_ABSOLUTE);
            hr = SHGetRealIDL((*this).m_pSFParent, child1, &mut pidl1_temp);
            if hr >= 0 {
                child1 = pidl1_temp as PITEMID_CHILD;
            }
        }
    }

    match l_event as u32 {
        SHCNE_MKDIR | SHCNE_CREATE | SHCNE_DRIVEADD => {
            if !child0.is_null() {
                if CDefView_LV_FindItemByPidl_body(this, child0) < 0 {
                    CDefView_LV_AddItem_body(this, child0);
                } else {
                    CDefView_LV_UpdateItem_2_body(this, child0);
                }
            }
        }
        SHCNE_RMDIR | SHCNE_DELETE | SHCNE_DRIVEREMOVED => {
            if !child0.is_null() {
                CDefView_LV_DeleteItem_body(this, child0);
            }
        }
        SHCNE_RENAMEFOLDER | SHCNE_RENAMEITEM => {
            if !child0.is_null() && !child1.is_null() {
                CDefView_LV_RenameItem_body(this, child0, child1);
            } else if !child0.is_null() {
                CDefView_LV_DeleteItem_body(this, child0);
            } else if !child1.is_null() {
                CDefView_LV_AddItem_body(this, child1);
            }
        }
        SHCNE_UPDATEITEM => {
            if !child0.is_null() {
                CDefView_LV_UpdateItem_2_body(this, child0);
            }
        }
        SHCNE_UPDATEIMAGE | SHCNE_MEDIAINSERTED | SHCNE_MEDIAREMOVED | SHCNE_ASSOCCHANGED => {
            CDefView_LV_RefreshIcons_body(this);
        }
        SHCNE_UPDATEDIR | SHCNE_ATTRIBUTES => {
            if !child0.is_null() {
                CDefView_LV_UpdateItem_2_body(this, child0);
            } else {
                CDefView_Refresh_body(this);
            }
            CDefView_UpdateStatusbar_body(this);
        }
        SHCNE_FREESPACE => {
            CDefView_UpdateStatusbar_body(this);
        }
        _ => {}
    }

    SHChangeNotification_Unlock(h_lock);

    CoTaskMemFree(pidl1_temp as *mut core::ffi::c_void);
    CoTaskMemFree(pidl0_temp as *mut core::ffi::c_void);

    return TRUE as LRESULT;
}

pub unsafe fn CDefView_OnMenuMessage_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    if (*this).m_pCM.is_null() {
        return FALSE as LRESULT;
    }
    let mut result: LRESULT = 0;
    let hres = SHForwardContextMenuMsg((((*this).m_pCM) as _), uMsg, wParam, lParam, &mut result, TRUE);
    (hres >= 0) as LRESULT
}

pub unsafe fn CDefView_OnSettingChange_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let _ = bHandled;

    if wParam == SPI_SETDESKWALLPAPER as WPARAM || wParam == 0 {
        CDefView_UpdateListColors_body(this);
    }

    let mut ListExMask: UINT = LVS_EX_TRACKSELECT | LVS_EX_ONECLICKACTIVATE;
    let mut ListExBits: UINT = CDefView_GetItemActivateFlags_body(this);

    if wParam == SPI_GETICONTITLELOGFONT as WPARAM
        || (lParam != 0
            && lstrcmpiW(
                lParam as PWSTR,
                REGSTR_PATH_EXPLORER_ICONUNDERLINE.as_ptr(),
            ) == 0)
    {
        ListExMask |= LVS_EX_UNDERLINEHOT | LVS_EX_UNDERLINECOLD;
        ListExBits |= SHELL_GetIconUnderlineFlags();
    }

    (*this).m_ListView.SetExtendedListViewStyle(ListExBits, ListExMask);
    (*this).m_ListView.SendMessage(uMsg, wParam, lParam);

    S_OK as LRESULT
}

pub unsafe fn CDefView_OnInitMenuPopup_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let hmenu = wParam as HMENU;
    let nPos = LOWORD!(lParam) as i32;

    if (*this).m_isEditing != 0 {
        ListView_CancelEditLabel((*this).m_ListView.m_hWnd);
    }
    if ((*this).m_pCM as usize) != 0 {
        CDefView_OnMenuMessage_body(this, uMsg, wParam, lParam, bHandled);
    }

    let hViewMenu = GetSubmenuByID_body((*this).m_hMenu, FCIDM_MENU_VIEW);

    if CDefView_GetSelections_body(this) == 0 {
        EnableMenuItem(hmenu, FCIDM_SHVIEW_CUT, MF_GRAYED);
        EnableMenuItem(hmenu, FCIDM_SHVIEW_COPY, MF_GRAYED);
        EnableMenuItem(hmenu, FCIDM_SHVIEW_RENAME, MF_GRAYED);
        EnableMenuItem(hmenu, FCIDM_SHVIEW_COPYTO, MF_GRAYED);
        EnableMenuItem(hmenu, FCIDM_SHVIEW_MOVETO, MF_GRAYED);
        EnableMenuItem(hmenu, FCIDM_SHVIEW_DELETE, MF_GRAYED);
    } else {
        EnableMenuItem(hmenu, FCIDM_SHVIEW_CUT, MF_ENABLED);
        EnableMenuItem(hmenu, FCIDM_SHVIEW_COPY, MF_ENABLED);
        EnableMenuItem(hmenu, FCIDM_SHVIEW_RENAME, MF_ENABLED);
        EnableMenuItem(hmenu, FCIDM_SHVIEW_COPYTO, MF_ENABLED);
        EnableMenuItem(hmenu, FCIDM_SHVIEW_MOVETO, MF_ENABLED);
        EnableMenuItem(hmenu, FCIDM_SHVIEW_DELETE, MF_ENABLED);
    }

    let menuItemId: UINT;
    if hmenu == GetSubMenu((*this).m_hMenu, nPos) {
        menuItemId = ReallyGetMenuItemID_body((*this).m_hMenu, nPos);
    } else if (hViewMenu as usize) != 0 && hmenu == GetSubMenu(hViewMenu, nPos) {
        menuItemId = ReallyGetMenuItemID_body(hViewMenu, nPos);
    } else if ((*this).m_hContextMenu as usize) != 0 && hmenu == GetSubMenu((*this).m_hContextMenu, nPos) {
        menuItemId = ReallyGetMenuItemID_body((*this).m_hContextMenu, nPos);
    } else {
        return FALSE as LRESULT;
    }

    match menuItemId {
        FCIDM_MENU_FILE => {
            CDefView_FillFileMenu_body(this);
        }
        FCIDM_MENU_VIEW | FCIDM_SHVIEW_VIEW => {
            CDefView_CheckViewMode_body(this, hmenu);
        }
        FCIDM_SHVIEW_ARRANGE => {
            CDefView_FillArrangeAsMenu_body(this, hmenu);
        }
        _ => {}
    }

    return FALSE as LRESULT;
}

pub unsafe fn CDefView_OnChangeCBChain_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    *bHandled = TRUE;
    unsafe { (*this).m_ClipboardChain.HandleChangeCBChain(wParam, lParam) }
}

pub unsafe fn CDefView_OnDrawClipboard_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
    let _ = uMsg;
    *bHandled = TRUE;
    let res: LRESULT = (*this).m_ClipboardChain.HandleDrawClipboard(wParam, lParam);
    if (*this).m_HasCutItems {
        (*this).m_ClipboardChain.Unhook((*this).m_hWnd);
        (*this).m_HasCutItems = false as _;
        (*this).m_ListView.SetItemState(-1, 0, LVIS_CUT);
        (*this).m_ListView.SendMessageW(LVM_SETCALLBACKMASK, ((*this).m_ListView.SendMessageW(LVM_GETCALLBACKMASK, 0, 0) | (LVIS_CUT as LRESULT)) as WPARAM, 0);
    }
    res
}

pub unsafe fn CDefView_RefreshGhostedState_body(this: *mut CDefView) -> () {
    let mut i = 0;
    let c = (*this).m_ListView.GetItemCount() as u32;
    while i < c {
        let attr = CDefView_GetItemAttributes_2_body(this, i as i32, SFGAO_HIDDEN | SFGAO_GHOSTED);
        let state = if attr != 0 { LVIS_CUT } else { 0 };
        (*this).m_ListView.SetItemState(i as i32, state, LVIS_CUT);
        i += 1;
    }
}

pub unsafe fn CDefView_GetWindow_body(this: *mut CDefView, phWnd: *mut HWND) -> HRESULT {
    *phWnd = (*this).m_hWnd;
    S_OK
}

pub unsafe fn CDefView_ContextSensitiveHelp_body(this: *mut CDefView, fEnterMode: BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_TranslateAccelerator_body(this: *mut CDefView, lpmsg: *mut MSG) -> HRESULT {
    if (*this).m_isEditing != 0 {
        return S_FALSE;
    }

    if (*lpmsg).message >= WM_KEYFIRST && (*lpmsg).message <= WM_KEYLAST {
        if TranslateAcceleratorW((*this).m_hWnd, (*this).m_hAccel, ((lpmsg) as _)) != 0 {
            return S_OK;
        }
    }

    return if !(*this).m_pShellBrowser.is_null() {
        ((*(*(*this).m_pShellBrowser).lpVtbl).TranslateAcceleratorSB)((*this).m_pShellBrowser, lpmsg, 0)
    } else {
        S_FALSE
    };
}

pub unsafe fn CDefView_EnableModeless_body(this: *mut CDefView, fEnable: BOOL) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_UIActivate_body(this: *mut CDefView, uState: UINT) -> HRESULT {
    if (*this).m_uState == uState {
        return S_OK;
    }

    CDefView_DoActivate_body(this, uState);

    if uState != SVUIA_DEACTIVATE {
        CDefView__ForceStatusBarResize_body(this);

        CDefView_UpdateStatusbar_body(this);
    }

    return S_OK;
}

pub unsafe fn CDefView_Refresh_body(this: *mut CDefView) -> HRESULT {
    CDefView__DoFolderViewCB_body(this, SFVM_LISTREFRESHED, TRUE as WPARAM, 0);

    (*this).m_ListView.DeleteAllItems();
    CDefView_FillList_body(this, TRUE); // CDefView.cpp:347 `IsRefreshCommand = TRUE`

    return S_OK;
}

pub unsafe fn CDefView_CreateViewWindow_body(this: *mut CDefView, lpPrevView: *mut IShellView, lpfs: LPCFOLDERSETTINGS, psb: *mut IShellBrowser, prcView: *mut RECT, phWnd: *mut HWND) -> HRESULT {
    CDefView_CreateViewWindow3_body(this, psb, lpPrevView, SV3CVW3_DEFAULT, (*lpfs).fFlags as FOLDERFLAGS, (*lpfs).fFlags as FOLDERFLAGS, (*lpfs).ViewMode as FOLDERVIEWMODE, core::ptr::null(), prcView, phWnd)
}

pub unsafe fn CDefView_DestroyViewWindow_body(this: *mut CDefView) -> HRESULT {
    CDefView_UIActivate_body(this, SVUIA_DEACTIVATE);

    if !(*this).m_hAccel.is_null() {
        (*this).m_hAccel = ::core::ptr::null_mut();
    }

    if !(*this).m_hMenuArrangeModes.is_null() {
        DestroyMenu((*this).m_hMenuArrangeModes);
        (*this).m_hMenuArrangeModes = ::core::ptr::null_mut();
    }

    if !(*this).m_hMenuViewModes.is_null() {
        DestroyMenu((*this).m_hMenuViewModes);
        (*this).m_hMenuViewModes = ::core::ptr::null_mut();
    }

    if !(*this).m_hMenu.is_null() {
        DestroyMenu((*this).m_hMenu);
        (*this).m_hMenu = ::core::ptr::null_mut();
    }

    // SFVM_WINDOWCLOSING goes out BEFORE the list view dies: a callback that
    // subclassed DefView (browseui's CFindFolder) must unsubclass while its
    // children still exist, or teardown crashes (upstream 68a03b2e1ae, CORE-20504).
    if !(*this).m_hWnd.is_null() {
        CDefView__DoFolderViewCB_body(
            this,
            SFVM_WINDOWCLOSING,
            (*this).m_hWnd as WPARAM,
            0,
        );
    }

    if !(*this).m_ListView.is_null() {
        // ATL CWindow::DestroyWindow clears m_hWnd on success.
        if DestroyWindow((*this).m_ListView.m_hWnd) != 0 {
            (*this).m_ListView.m_hWnd = ::core::ptr::null_mut();
        }
    }

    if !(*this).m_hWnd.is_null() {
        DestroyWindow((*this).m_hWnd);
    }

    if !(*this).m_pShellBrowser.is_null() {
        ((*(*(*this).m_pShellBrowser).lpVtbl).Release)((*this).m_pShellBrowser);
    }
    (*this).m_pShellBrowser = ::core::ptr::null_mut();

    if !(*this).m_pCommDlgBrowser.is_null() {
        ((*(*(*this).m_pCommDlgBrowser).lpVtbl).Release)((*this).m_pCommDlgBrowser);
    }
    (*this).m_pCommDlgBrowser = ::core::ptr::null_mut();

    return S_OK;
}

pub unsafe fn CDefView_GetCurrentInfo_body(this: *mut CDefView, lpfs: LPFOLDERSETTINGS) -> HRESULT {
    if lpfs.is_null() {
        return E_INVALIDARG;
    }

    unsafe {
        core::ptr::copy_nonoverlapping(core::ptr::addr_of!((*this).m_FolderSettings), lpfs, 1);
    }
    S_OK
}

pub unsafe fn CDefView_AddPropertySheetPages_body(this: *mut CDefView, dwReserved: DWORD, lpfn: LPFNSVADDPROPSHEETPAGE, lparam: LPARAM) -> HRESULT {
    let Data = SFVM_PROPPAGE_DATA { dwReserved, pfn: lpfn, lParam: lparam };
    CDefView__DoFolderViewCB_body(this, SFVM_ADDPROPERTYPAGES, 0, &Data as *const _ as LPARAM);
    return S_OK;
}

pub unsafe fn Read_body(pS: *mut IStream, buffer: LPVOID, cb: ULONG) -> HRESULT {
    let mut read: ULONG = 0;
    let hr: HRESULT = ((*(*pS).lpVtbl).Read)(pS, buffer, cb, &mut read);
    if hr < 0 {
        hr
    } else if cb == read {
        S_OK
    } else {
        HResultFromWin32(ERROR_MORE_DATA)
    }
}

pub unsafe fn ReadDWORD_body(pPB: *mut IPropertyBag, name: LPCWSTR, def: DWORD) -> DWORD {
    let mut value: DWORD = 0;
    let hr = SHPropertyBag_ReadDWORD(pPB.cast(), name, &mut value);
    if hr >= 0 {
        value
    } else {
        def
    }
}

pub unsafe fn CDefView_GetDefaultViewStream_body(this: *mut CDefView, Stgm: DWORD, ppStream: *mut *mut IStream) -> HRESULT {
    let mut clsid: CLSID = core::mem::zeroed();
    let clsid_ptr = core::ptr::addr_of_mut!(clsid);
    let mut hr: HRESULT = IUnknown_GetClassID((*this).m_pSFParent.cast(), clsid_ptr);
    if hr >= 0 {
        let mut path: [u16; MAX_PATH as usize] = core::mem::zeroed();
        let path_ptr = core::ptr::addr_of_mut!(path).cast::<u16>();
        let mut name: [u16; 39 as usize] = core::mem::zeroed();
        let name_ptr = core::ptr::addr_of_mut!(name).cast::<u16>();
        wsprintfW(path_ptr, CPP_COM_WSTR_BA2342D760A2.as_ptr(), REGSTR_PATH_EXPLORER.as_ptr(), CPP_COM_WSTR_A7D0FE10F8A8.as_ptr());
        StringFromGUID2(clsid_ptr as *const CLSID, name_ptr, 39);
        *ppStream = SHOpenRegStream2W(HKEY_CURRENT_USER, path_ptr as *const u16, name_ptr as *const u16, Stgm).cast();
        hr = if !(*ppStream).is_null() { S_OK } else { E_FAIL };
    }
    hr
}

pub unsafe fn LoadColumnsStream_body(cols: &mut PERSISTCOLUMNS, pS: *mut IStream) -> HRESULT {
    let hr = Read_body(
        pS,
        (&mut *cols) as *mut PERSISTCOLUMNS as _,
        FIELD_OFFSET!(PERSISTCOLUMNS, Columns) as _
    );
    if hr < 0 {
        return hr;
    }
    if cols.Signature != PERSISTCOLUMNS::SIG
        || cols.Count > PERSISTCOLUMNS::MAXCOUNT
    {
        return HResultFromWin32(ERROR_INVALID_DATA);
    }
    let cb = (core::mem::size_of_val(&cols.Columns[0]) * cols.Count as usize) as _;
    return Read_body(
        pS,
        (&mut cols.Columns) as *mut _ as _,
        cb
    );
}

pub unsafe fn CDefView_LoadViewState_body(this: *mut CDefView) -> HRESULT {
    let mut cvs: PERSISTCLASSICVIEWSTATE = core::mem::zeroed();
    let mut cols: PERSISTCOLUMNS = core::mem::zeroed();
    let mut pS: *mut IStream = core::ptr::null_mut();
    let _pS_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pS));
    let mut pPB: *mut IPropertyBag = core::ptr::null_mut();
    let _pPB_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pPB));
    let mut fallback = false;
    let mut hrColumns: HRESULT = E_FAIL;
    let mut hr: HRESULT = IUnknown_QueryServicePropertyBag(
        (*this).m_pShellBrowser.cast(),
        SHGVSPB_FOLDER as c_long,
        core::ptr::addr_of!(IID_IPropertyBag),
        core::ptr::addr_of_mut!(pPB).cast()
    );

    if hr >= 0 {
        let mut data: DWORD = 0;
        hr = SHPropertyBag_ReadDWORD(pPB.cast(), CPP_COM_WSTR_C47C37CF2996.as_ptr(), &mut data);
        if hr < 0 {
            hr = CDefView_GetDefaultViewStream_body(this, STGM_READ, &mut pS);
            fallback = true;

            if hr < 0 || {
                hr = Read_body(
                    pS,
                    &mut cvs as *mut PERSISTCLASSICVIEWSTATE as *mut core::ffi::c_void,
                    core::mem::size_of_val(&cvs) as ULONG
                );
                hr < 0
            } {
                return hr;
            }

            if cvs.Signature != PERSISTCLASSICVIEWSTATE::SIG {
                return HResultFromWin32(ERROR_INVALID_DATA);
            }

            hrColumns = LoadColumnsStream_body(&mut cols, pS);
        } else {
            cvs.FolderSettings.ViewMode = data;
            cvs.FolderSettings.fFlags = ReadDWORD_body(
                pPB,
                CPP_COM_WSTR_EC73128D2969.as_ptr(),
                FWF_NOGROUPING
            );
            data = ReadDWORD_body(
                pPB,
                CPP_COM_WSTR_C6AD24AE112E.as_ptr(),
                !0u32
            );
            cvs.SortColId = if data != !0u32 {
                data as WORD
            } else {
                LISTVIEW_SORT_INFO::UNSPECIFIEDCOLUMN as WORD
            };
            cvs.SortDir = ReadDWORD_body(
                pPB,
                CPP_COM_WSTR_7E65B4F82DCB.as_ptr(),
                1
            ) as INT8;

            hrColumns = SHPropertyBag_ReadStream(
                pPB.cast(),
                CPP_COM_WSTR_98BD7125CFE5.as_ptr(),
                core::ptr::addr_of_mut!(pS).cast(),
            );
            if hrColumns >= 0 {
                hrColumns = LoadColumnsStream_body(&mut cols, pS);
            }
        }
    } else {
        let pBrowser = (*this).m_pShellBrowser;
        hr = if !pBrowser.is_null() {
            ((*(*pBrowser).lpVtbl).GetViewStateStream)(
                pBrowser,
                STGM_READ,
                &mut pS
            )
        } else {
            E_UNEXPECTED
        };

        if hr < 0 {
            hr = CDefView_GetDefaultViewStream_body(this, STGM_READ, &mut pS);
            fallback = true;
        }

        if hr < 0 || {
            hr = Read_body(
                pS,
                &mut cvs as *mut PERSISTCLASSICVIEWSTATE as *mut core::ffi::c_void,
                core::mem::size_of_val(&cvs) as ULONG
            );
            hr < 0
        } {
            return hr;
        }

        if cvs.Signature != PERSISTCLASSICVIEWSTATE::SIG {
            return HResultFromWin32(ERROR_INVALID_DATA);
        }

        hrColumns = LoadColumnsStream_body(&mut cols, pS);
    }

    (*this).m_FolderSettings.ViewMode = cvs.FolderSettings.ViewMode;
    (*this).m_FolderSettings.fFlags &= !PERSISTCLASSICVIEWSTATE::VALIDFWF;
    (*this).m_FolderSettings.fFlags |= cvs.FolderSettings.fFlags & PERSISTCLASSICVIEWSTATE::VALIDFWF;

    if hrColumns >= 0 {
        let mut failed: BOOL = FALSE;
        (*this).m_LoadColumnsList = DPA_Create(cols.Count as i32);

        if !(*this).m_LoadColumnsList.is_null() {
            let mut i = 0;
            while i < cols.Count {
                failed |= if DPA_SetPtr(
                    (*this).m_LoadColumnsList,
                    ((i) as i32),
                    UlongToPtr(((cols.Columns[i as usize] as usize) as u32))
                ) == 0 {
                    1
                } else {
                    0
                };
                i += 1;
            }
        }

        if failed != 0 || cols.Count == 0 {
            DPA_Destroy((*this).m_LoadColumnsList);
            (*this).m_LoadColumnsList = core::ptr::null_mut();
        }
    }

    (*this).m_sortInfo.bLoadedFromViewState =
        !fallback &&
        !(*this).m_LoadColumnsList.is_null() &&
        (cvs.SortColId as i32) != (LISTVIEW_SORT_INFO::UNSPECIFIEDCOLUMN as i32);
    (*this).m_sortInfo.bColumnIsFolderColumn = ((TRUE) != 0);
    (*this).m_sortInfo.Direction = if (cvs.SortDir as i32) > 0 {
        1
    } else {
        -1
    };
    (*this).m_sortInfo.ListColumn = cvs.SortColId as i32;

    hr
}

pub unsafe fn CDefView_SaveViewState_helper_body(this: *mut CDefView, pStream: *mut IStream) -> HRESULT {
    if (*this).m_ListView.m_hWnd.is_null() {
        return E_UNEXPECTED;
    }

    let sortcol = CDefView_MapListColumnToFolderColumn_body(this, (((*this).m_sortInfo.ListColumn) as u32));

    let mut cvs = core::mem::zeroed::<PERSISTCLASSICVIEWSTATE>();
    cvs.SortColId = if sortcol >= 0 { sortcol as u16 } else { 0 };
    cvs.SortDir = (*this).m_sortInfo.Direction;

    let mut cols = core::mem::zeroed::<PERSISTCOLUMNS>();
    cols.Signature = PERSISTCOLUMNS::SIG;
    cols.Count = 0;

    let mut lvc = core::mem::zeroed::<LVCOLUMN>();
    lvc.mask = LVCF_WIDTH;

    let mut i = 0;
    let mut j = 0;
    while i < PERSISTCOLUMNS::MAXCOUNT && ListView_GetColumn((*this).m_ListView.m_hWnd, j, &mut lvc) != 0 {
        let map_hr = CDefView_MapListColumnToFolderColumn_body(this, ((j) as u32));
        if map_hr >= 0 {
            cols.Columns[i as usize] = MAKELONG!(map_hr, lvc.cx) as u32;
            i += 1;
            cols.Count = i;
        }
        j += 1;
    }

    let cbColumns: u32 = FIELD_OFFSET!(PERSISTCOLUMNS, Columns) as u32 + (core::mem::size_of_val(&cols.Columns[0]) as u32) * cols.Count;
    CDefView_UpdateFolderViewFlags_body(this);

    let mut hr: HRESULT = S_OK;
    let mut pStream = pStream;
    let mut do_stream = false;
    let mut pPB: *mut IPropertyBag = core::ptr::null_mut();

    if !pStream.is_null() {
        ((*(*pStream).lpVtbl).AddRef)(pStream);
        do_stream = true;
    } else {
        hr = IUnknown_QueryServicePropertyBag((*this).m_pShellBrowser.cast(), SHGVSPB_FOLDER as LONG, core::ptr::addr_of!(IID_IPropertyBag), core::ptr::addr_of_mut!(pPB).cast());
        if hr >= 0 {
            let mut uViewMode: u32 = 0;
            CDefView_GetCurrentViewMode_body(this, &mut uViewMode);
            hr = SHPropertyBag_WriteDWORD(pPB.cast(), CPP_COM_WSTR_C47C37CF2996.as_ptr(), uViewMode);
            SHPropertyBag_WriteDWORD(pPB.cast(), CPP_COM_WSTR_EC73128D2969.as_ptr(), (*this).m_FolderSettings.fFlags);
            SHPropertyBag_WriteDWORD(pPB.cast(), CPP_COM_WSTR_C6AD24AE112E.as_ptr(), cvs.SortColId as DWORD);
            SHPropertyBag_WriteDWORD(pPB.cast(), CPP_COM_WSTR_7E65B4F82DCB.as_ptr(), cvs.SortDir as DWORD);
            pStream = if cols.Count != 0 {
                SHCreateMemStream(&mut cols as *mut PERSISTCOLUMNS as *mut u8, cbColumns).cast()
            } else {
                core::ptr::null_mut()
            };
            if pStream.is_null() || SHPropertyBag_WriteStream(pPB.cast(), CPP_COM_WSTR_98BD7125CFE5.as_ptr(), pStream.cast()) < 0 {
                SHPropertyBag_Delete(pPB.cast(), CPP_COM_WSTR_98BD7125CFE5.as_ptr());
            }
            ((*(*pPB).lpVtbl).Release)(pPB);
        } else {
            let view_state_hr = if !(*this).m_pShellBrowser.is_null() {
                ((*(*(*this).m_pShellBrowser).lpVtbl).GetViewStateStream)((*this).m_pShellBrowser, STGM_WRITE, &mut pStream)
            } else {
                E_UNEXPECTED
            };
            hr = view_state_hr;
            if hr >= 0 {
                do_stream = true;
            }
        }
    }

    if do_stream {
        let mut written: u32 = 0;
        cvs.Signature = PERSISTCLASSICVIEWSTATE::SIG;
        // FOLDERSETTINGS is a bindgen struct without Copy, so this member-wise C++
        // assignment has to be a byte copy.
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*this).m_FolderSettings),
            core::ptr::addr_of_mut!(cvs.FolderSettings),
            1,
        );
        hr = ((*(*pStream).lpVtbl).Write)(pStream, &cvs as *const PERSISTCLASSICVIEWSTATE as *const core::ffi::c_void, core::mem::size_of::<PERSISTCLASSICVIEWSTATE>() as u32, &mut written);
        if hr >= 0 {
            hr = ((*(*pStream).lpVtbl).Write)(pStream, &cols as *const PERSISTCOLUMNS as *const core::ffi::c_void, cbColumns, &mut written);
        }
    }

    if !pStream.is_null() {
        ((*(*pStream).lpVtbl).Release)(pStream);
    }

    hr
}

pub unsafe fn CDefView_SaveViewState_body(this: *mut CDefView) -> HRESULT {
    if ((*this).m_FolderSettings.fFlags & FWF_NOBROWSERVIEWSTATE) == 0 {
        return CDefView_SaveViewState_helper_body(this, core::ptr::null_mut());
    }
    S_FALSE
}

pub unsafe fn CDefView_UpdateFolderViewFlags_body(this: *mut CDefView) -> () {
    UPDATEFOLDERVIEWFLAGS!((*this).m_FolderSettings.fFlags, FWF_AUTOARRANGE, CDefView_GetAutoArrange_body(this) == S_OK);
    UPDATEFOLDERVIEWFLAGS!((*this).m_FolderSettings.fFlags, FWF_SNAPTOGRID, CDefView__GetSnapToGrid_body(this) == S_OK);
    UPDATEFOLDERVIEWFLAGS!((*this).m_FolderSettings.fFlags, FWF_NOGROUPING, ListView_IsGroupViewEnabled((*this).m_ListView.m_hWnd) == 0);
}

pub unsafe fn CDefView_GetItemActivateFlags_body(this: *mut CDefView) -> UINT {
    let mut ss: SHELLSTATE = core::mem::zeroed();
    SHGetSetSettings(
        &mut ss,
        SSF_DOUBLECLICKINWEBVIEW | SSF_WIN95CLASSIC,
        FALSE
    );

    if ((*this).m_FolderSettings.fFlags & FWF_SINGLECLICKACTIVATE) != 0
        || (ss.fDoubleClickInWebView() == 0 && ss.fWin95Classic() == 0)
    {
        LVS_EX_TRACKSELECT | LVS_EX_ONECLICKACTIVATE
    } else {
        0
    }
}

pub unsafe fn CDefView_SelectItem_v1_body(this: *mut CDefView, pidl: PCUITEMID_CHILD, uFlags: SVSIF) -> HRESULT {
    let i: i32;

    if (*this).m_ListView.is_null() {
        return E_FAIL;
    }

    i = CDefView_LV_FindItemByPidl_body(this, pidl);
    if i == -1 {
        return S_OK;
    }

    let mut lvItem: LVITEMW = unsafe { core::mem::zeroed() };
    lvItem.mask = LVIF_STATE;
    lvItem.stateMask = LVIS_SELECTED | LVIS_FOCUSED;

    while (*this).m_ListView.GetItem(&mut lvItem) != 0 {
        if lvItem.iItem == i {
            if (uFlags & SVSI_SELECT) != 0 {
                lvItem.state |= LVIS_SELECTED;
            } else {
                lvItem.state &= !LVIS_SELECTED;
            }

            if (uFlags & SVSI_FOCUSED) != 0 {
                lvItem.state |= LVIS_FOCUSED;
            } else {
                lvItem.state &= !LVIS_FOCUSED;
            }
        } else {
            if (uFlags & SVSI_DESELECTOTHERS) != 0 {
                lvItem.state &= !LVIS_SELECTED;
            }
            lvItem.state &= !LVIS_FOCUSED;
        }

        (*this).m_ListView.SetItem(&mut lvItem);
        lvItem.iItem += 1;
    }

    if (uFlags & SVSI_ENSUREVISIBLE) != 0 {
        (*this).m_ListView.EnsureVisible(i, FALSE);
    }

    if (uFlags & SVSI_EDIT) == SVSI_EDIT {
        (*this).m_ListView.EditLabel(i);
    }

    return S_OK;
}

pub unsafe fn CDefView_GetItemObject_body(this: *mut CDefView, uItem: UINT, riid: REFIID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut hr: HRESULT = E_NOINTERFACE;

    if ppvOut.is_null() {
        return E_INVALIDARG;
    }

    *ppvOut = core::ptr::null_mut();

    match uItem {
        SVGIO_BACKGROUND => {
            if cpp_com_iid_equal(riid, &IID_IContextMenu as *const _) {
                hr = CDefViewBckgrndMenu_CreateInstance((*this).m_pSF2Parent.cast(), riid, ppvOut);
                if ((hr) < 0) {
                    return hr;
                }

                IUnknown_SetSite((*ppvOut).cast(), this as *mut _);
            } else if cpp_com_iid_equal(riid, &IID_IDispatch as *const _) {
                // CDefView.cpp:3586 `m_pShellFolderViewDual == NULL` -- same null-test-as-
                // assignment confusion; here it would have stored the ADDRESS OF THE FIELD
                // into the field.
                if (*this).m_pShellFolderViewDual.is_null() {
                    hr = CDefViewDual_Constructor(riid, &mut (*this).m_pShellFolderViewDual as *mut _ as *mut *mut core::ffi::c_void);
                    if ((hr) < 0) {
                        return hr;
                    }
                }
                hr = ((*(*(*this).m_pShellFolderViewDual).lpVtbl).QueryInterface)((*this).m_pShellFolderViewDual, riid, ppvOut);
            }
        }
        SVGIO_SELECTION => {
            CDefView_GetSelections_body(this);
            hr = ((*(*(*this).m_pSFParent).lpVtbl).GetUIObjectOf)((*this).m_pSFParent, (*this).m_hWnd.cast(), (*this).m_cidl, (*this).m_apidl, riid, core::ptr::null_mut(), ppvOut);
            if ((hr) < 0) {
                return hr;
            }

            if cpp_com_iid_equal(riid, &IID_IContextMenu as *const _) {
                IUnknown_SetSite((*ppvOut).cast(), this as *mut _);
            }
        }
        _ => {}
    }

    hr
}

pub unsafe fn CDefView_GetDefaultViewMode_body(this: *mut CDefView) -> FOLDERVIEWMODE {
    let mut mode: FOLDERVIEWMODE = if ((*this).m_FolderSettings.fFlags & FWF_DESKTOP) != 0 || IsOS(OS_SERVERADMINUI) == 0 {
        ((FVM_ICON) as i32)
    } else {
        ((FVM_DETAILS) as i32)
    };
    let mut temp: FOLDERVIEWMODE = mode;
    if (CDefView__DoFolderViewCB_body(this, SFVM_DEFVIEWMODE, 0, &mut temp as *mut FOLDERVIEWMODE as LPARAM) >= 0) && CDefView_IsSupportedFolderViewMode_body(this, temp) {
        mode = temp;
    }
    mode
}

pub unsafe fn CDefView_GetCurrentViewMode_body(this: *mut CDefView, pViewMode: *mut UINT) -> HRESULT {
    if pViewMode.is_null() {
        return E_INVALIDARG;
    }

    *pViewMode = (*this).m_FolderSettings.ViewMode;
    S_OK
}

pub unsafe fn CDefView_SetCurrentViewMode_body(this: *mut CDefView, ViewMode: UINT) -> HRESULT {
    let view_mode_int = ViewMode as i32;

    if ((view_mode_int < FVM_FIRST as i32) || (view_mode_int > FVM_LAST as i32))
        && (view_mode_int != FVM_AUTO as i32)
    {
        return E_INVALIDARG;
    }

    let dwStyle = match ViewMode {
        FVM_ICON => LVS_ICON,
        FVM_SMALLICON => LVS_SMALLICON,
        FVM_LIST => LVS_LIST,
        FVM_DETAILS => LVS_REPORT,
        _ => {
            unsafe {
                DbgPrint(
                    b"ViewMode %d not implemented\n\0".as_ptr().cast(),
                    ViewMode,
                );
            }
            LVS_LIST
        }
    };

    ((*this).m_ListView).ModifyStyle(LVS_TYPEMASK, dwStyle, 0);

    (*this).m_FolderSettings.ViewMode = ViewMode;
    CDefView_CheckToolbar_body(this);

    return S_OK;
}

pub unsafe fn CDefView_GetFolder_body(this: *mut CDefView, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    // CDefView.cpp:3668 `if (m_pSFParent == NULL) return E_FAIL;` -- a NULL TEST. The
    // translation turned it into a call to the CComPtr ASSIGNMENT helper, which would have
    // released the parent folder and then queried through the null it just stored.
    if (*this).m_pSFParent.is_null() {
        return E_FAIL;
    }
    ((*(*(*this).m_pSFParent).lpVtbl).QueryInterface)((*this).m_pSFParent, ((riid) as _), ppv)
}

pub unsafe fn CDefView_Item_body(this: *mut CDefView, iItemIndex: i32, ppidl: *mut PITEMID_CHILD) -> HRESULT {
    let pidl = CDefView__PidlByItem_body(this, iItemIndex);
    if !pidl.is_null() {
        *ppidl = ILClone(pidl);
        return S_OK;
    }

    *ppidl = core::ptr::null_mut();
    return E_INVALIDARG;
}

pub unsafe fn CDefView_ItemCount_body(this: *mut CDefView, uFlags: UINT, pcItems: *mut i32) -> HRESULT {
    if uFlags != SVGIO_ALLVIEW && uFlags != SVGIO_SELECTION {
        unsafe {
            DbgPrint(b"some flags unsupported, %x\n\0".as_ptr() as *const _, uFlags & !(SVGIO_ALLVIEW | SVGIO_SELECTION));
        }
    }
    if (uFlags & SVGIO_TYPE_MASK) == SVGIO_SELECTION {
        *pcItems = ((CListView_GetSelectedCount_body(&mut (*this).m_ListView as *mut _)) as i32);
    } else {
        *pcItems = CListView_GetItemCount_body(&mut (*this).m_ListView as *mut _);
    }
    return S_OK;
}

pub unsafe fn CDefView_Items_body(this: *mut CDefView, uFlags: UINT, riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_GetSelectionMarkedItem_body(this: *mut CDefView, piItem: *mut i32) -> HRESULT {
    *piItem = (*this).m_ListView.GetSelectionMark();
    return S_OK;
}

pub unsafe fn CDefView_GetFocusedItem_body(this: *mut CDefView, piItem: *mut i32) -> HRESULT {
    *piItem = ((*this).m_ListView).GetNextItem(-1, ((LVNI_FOCUSED) as u16));
    return S_OK;
}

pub unsafe fn CDefView_GetItemPosition_body(this: *mut CDefView, pidl: PCUITEMID_CHILD, ppt: *mut POINT) -> HRESULT {
    if (*this).m_ListView.is_null() {
        return E_FAIL;
    }

    let lvIndex = CDefView_LV_FindItemByPidl_body(this, pidl);
    if lvIndex == -1 || ppt.is_null() {
        return E_INVALIDARG;
    }

    ((*this).m_ListView).GetItemPosition(lvIndex, ppt);
    return S_OK;
}

pub unsafe fn CDefView_GetSpacing_body(this: *mut CDefView, ppt: *mut POINT) -> HRESULT {
    if (*this).m_ListView.is_null() {
        return S_FALSE;
    }

    if !ppt.is_null() {
        let mut spacing: crate::island::SIZE = core::mem::zeroed();
        (*this).m_ListView.GetItemSpacing(&mut spacing, FALSE); // rosctrls.h:166 `bSmallIconView = FALSE`

        (*ppt).x = spacing.cx;
        (*ppt).y = spacing.cy;
    }

    S_OK
}

pub unsafe fn CDefView_GetDefaultSpacing_body(this: *mut CDefView, ppt: *mut POINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_GetAutoArrange_body(this: *mut CDefView) -> HRESULT {
    return if ((*this).m_ListView.GetStyle() & LVS_AUTOARRANGE) != 0 {
        S_OK
    } else {
        S_FALSE
    };
}

pub unsafe fn CDefView__GetSnapToGrid_body(this: *mut CDefView) -> HRESULT {
    let dwExStyle: u32 = (*this).m_ListView.SendMessage(
        LVM_GETEXTENDEDLISTVIEWSTYLE,
        0,
        0,
    ) as u32;

    if (dwExStyle & LVS_EX_SNAPTOGRID) != 0 {
        S_OK
    } else {
        S_FALSE
    }
}

pub unsafe fn CDefView_SelectItem_v2_body(this: *mut CDefView, iItem: i32, dwFlags: DWORD) -> HRESULT {
    unsafe {
        let mut lv_item_state: u32 = 0 as u32;
        let mut lv_item_state_mask: u32 = LVIS_SELECTED as u32;

        if (dwFlags & (SVSI_ENSUREVISIBLE as u32)) != 0 {
            CListView_EnsureVisible_body(&mut (*this).m_ListView as *mut _, iItem, 0 as i32);
        }

        if (dwFlags & (SVSI_DESELECTOTHERS as u32)) != 0 {
            CListView_SetItemState_body(
                &mut (*this).m_ListView as *mut _,
                -(1 as i32),
                0 as u32,
                LVIS_SELECTED as u32,
            );
        }

        if (dwFlags & (SVSI_SELECT as u32)) != 0 {
            lv_item_state |= LVIS_SELECTED as u32;
        }

        if (dwFlags & (SVSI_FOCUSED as u32)) != 0 {
            lv_item_state_mask |= LVIS_FOCUSED as u32;
        }

        CListView_SetItemState_body(
            &mut (*this).m_ListView as *mut _,
            iItem,
            lv_item_state,
            lv_item_state_mask,
        );

        if (dwFlags & (SVSI_EDIT as u32)) == (SVSI_EDIT as u32) {
            CListView_EditLabel_body(&mut (*this).m_ListView as *mut _, iItem);
        }

        S_OK as HRESULT
    }
}

pub unsafe fn CDefView_SelectAndPositionItems_body(this: *mut CDefView, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, apt: *mut POINT, dwFlags: DWORD) -> HRESULT {
    _ = ((*this).m_ListView).SetItemState(-1, 0, LVIS_SELECTED);
    let mut i: UINT = 0;
    while i < cidl {
        let lvIndex: i32 = CDefView_LV_FindItemByPidl_body(this, *apidl.add(i as usize));
        if lvIndex != -1 {
            _ = CDefView_SelectAndPositionItem_helper_body(
                this,
                lvIndex as INT,
                dwFlags,
                if !apt.is_null() { apt.add(i as usize) } else { core::ptr::null_mut() }
            );
        }
        i += 1;
    }
    return S_OK;
}

pub unsafe fn CDefView_GetView_body(this: *mut CDefView, pVid: *mut SHELLVIEWID, view_type: ULONG) -> HRESULT {
    let mut view_type = view_type;

    if view_type == SV2GV_DEFAULTVIEW {
        *pVid = IID_CDefView;
        return S_OK;
    }
    if view_type == SV2GV_CURRENTVIEW {
        view_type = (*this).m_FolderSettings.ViewMode;
    }
    if (view_type as i32) < 0 {
        return E_UNEXPECTED;
    }

    view_type += ((FVM_FIRST) as u32);
    if CDefView_IsSupportedFolderViewMode_body(this, view_type as i32) == ((FALSE) != 0) {
        return E_INVALIDARG;
    }
    *pVid = *FolderViewModeToShellViewId(view_type);
    return S_OK;
}

pub unsafe fn CDefView_CreateViewWindow2_body(this: *mut CDefView, view_params: LPSV2CVW2_PARAMS) -> HRESULT {
    CDefView_CreateViewWindow3_body(
        this,
        (*view_params).psbOwner.cast(),
        (*view_params).psvPrev.cast(),
        SV3CVW3_DEFAULT,
        (*(*view_params).pfs).fFlags as FOLDERFLAGS,
        (*(*view_params).pfs).fFlags as FOLDERFLAGS,
        (*(*view_params).pfs).ViewMode as FOLDERVIEWMODE,
        (*view_params).pvid,
        (*view_params).prcView,
        &mut (*view_params).hwndView,
    )
}

pub unsafe fn CDefView_CreateViewWindow3_body(this: *mut CDefView, psb: *mut IShellBrowser, psvPrevious: *mut IShellView, view_flags: SV3CVW3_FLAGS, mask: FOLDERFLAGS, flags: FOLDERFLAGS, mode: FOLDERVIEWMODE, view_id: *const SHELLVIEWID, prcView: *const RECT, hwnd: *mut HWND) -> HRESULT {
    let mut omw = OLEMENUGROUPWIDTHS { width: [0, 0, 0, 0, 0, 0] };
    let SUPPORTED_SV3CVW3: UINT = SV3CVW3_FORCEVIEWMODE | SV3CVW3_FORCEFOLDERFLAGS;
    let IGNORE_FWF: UINT = FWF_OWNERDATA;
    let _ = SUPPORTED_SV3CVW3;
    let _ = IGNORE_FWF;
    let mut mode = mode;
    let _ = psvPrevious;
    *hwnd = core::ptr::null_mut();

    if !prcView.is_null() {
    }

    // CDefView.cpp:3862 `if (psb == NULL || m_hWnd) return E_UNEXPECTED;` -- the second
    // disjunct is C truthiness on a HWND, so it means "this view ALREADY has a window",
    // i.e. the already-created guard. Negating it instead (`m_hWnd.is_null()`) inverts the
    // guard into "no window yet", which is exactly the normal FIRST call, so every caller
    // got E_UNEXPECTED. That shipped as explorer's desktop failing at
    // CDesktopBrowser.cpp:253 with 0x8000FFFF and the kmtest sweep launching 0 of 91.
    if psb.is_null() || !(*this).m_hWnd.is_null() {
        return E_UNEXPECTED;
    }

    if (view_flags & !SUPPORTED_SV3CVW3) != 0 {
        let _ = view_flags & !SUPPORTED_SV3CVW3;
    }

    CDefView_cpp_com_assign_m_pShellBrowser(this, psb);
    (*this).m_FolderSettings.ViewMode = ((mode) as u32);
    (*this).m_FolderSettings.fFlags = (mask & flags) & !IGNORE_FWF;

    if !view_id.is_null() {
        let mut temp: FOLDERVIEWMODE = ShellViewIdToFolderViewMode(view_id) as FOLDERVIEWMODE;
        if CDefView_IsSupportedFolderViewMode_body(this, temp) {
            mode = temp;
        } else if IsEqualGUID(view_id, core::ptr::addr_of!(IID_CDefView)) {
            mode = FVM_AUTO;
        } else {
        }
    }

    if mode == FVM_AUTO {
        (*this).m_FolderSettings.ViewMode = ((CDefView_GetDefaultViewMode_body(this)) as u32);
    }

    if !CDefView_IsSupportedFolderViewMode_body(this, (((*this).m_FolderSettings.ViewMode) as i32)) {
        (*this).m_FolderSettings.ViewMode = FVM_ICON;
    }

    let requestedViewMode: UINT = (*this).m_FolderSettings.ViewMode as UINT;

    ((*(*(*this).m_pShellBrowser).lpVtbl).GetWindow)((*this).m_pShellBrowser, &mut (*this).m_hWndParent as *mut HWND);
    CDefView__DoFolderViewCB_body(this, SFVM_HWNDMAIN, 0, (*this).m_hWndParent as LPARAM);

    CDefView_cpp_com_assign_m_pCommDlgBrowser(this, core::ptr::null_mut());
    if (((*(*(*this).m_pShellBrowser).lpVtbl).QueryInterface)(
            (*this).m_pShellBrowser,
            core::ptr::addr_of!(IID_ICommDlgBrowser), core::ptr::addr_of_mut!((*this).m_pCommDlgBrowser).cast()
        )) >= 0 {
    }

    CDefView_LoadViewState_body(this);

    if (view_flags & SV3CVW3_FORCEVIEWMODE) != 0 {
        (*this).m_FolderSettings.ViewMode = requestedViewMode;
    }
    if (view_flags & SV3CVW3_FORCEFOLDERFLAGS) != 0 {
        (*this).m_FolderSettings.fFlags = (mask & flags) & !IGNORE_FWF;
    }

    let rcView: RECT = *prcView;
    // CDefView.cpp:3912 `Create(m_hWndParent, rcView, NULL, WS_CHILD|..., 0, 0U)`. C++ resolves
    // this through CWindowImpl::Create, which registers the class and then calls the base.
    {
        let wc = CDefView_GetWndClassInfo_body(this);
        if wc.m_lpszOrigName.is_null() {
            wc.m_lpszOrigName = wc.m_wc.lpszClassName;
        }
        let atom = AtlRegisterWndClass(wc, GetModuleHandleW(core::ptr::null()));
        AtlCreateWindow::<CDefView>(
            this,
            (*this).m_hWndParent,
            &rcView,
            core::ptr::null(),
            WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | WS_TABSTOP,
            0,
            core::ptr::null_mut(),
            atom,
            GetModuleHandleW(core::ptr::null()),
        );
    }

    if (*this).m_hWnd.is_null() {
        return E_FAIL;
    }

    *hwnd = (*this).m_hWnd;

    CDefView_CheckToolbar_body(this);

    if (*hwnd).is_null() {
        return E_FAIL;
    }

    CDefView__DoFolderViewCB_body(this, SFVM_WINDOWCREATED, (*this).m_hWnd as WPARAM, 0);

    SetWindowPos((*this).m_hWnd, HWND_TOP, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW);
    UpdateWindow((*this).m_hWnd);

    if (*this).m_hMenu.is_null() {
        (*this).m_hMenu = CreateMenu();
        ((*(*(*this).m_pShellBrowser).lpVtbl).InsertMenusSB)((*this).m_pShellBrowser, (*this).m_hMenu, core::ptr::addr_of_mut!(omw).cast());
    }

    CDefView__MergeToolbar_body(this);

    return S_OK;
}

pub unsafe fn CDefView_HandleRename_body(this: *mut CDefView, pidl: PCUITEMID_CHILD) -> HRESULT {
    let mut pidl = pidl;
    if pidl.is_null() {
        let mut idx = (*this).m_ListView.GetNextItem(-1, ((LVNI_SELECTED | LVNI_FOCUSED) as u16));
        if idx < 0 {
            idx = (*this).m_ListView.GetNextItem(-1, ((LVNI_SELECTED) as u16));
        }
        pidl = CDefView__PidlByItem_body(this, idx);
    }
    if ILFindLastID(pidl) != pidl as *mut _ {
        return E_INVALIDARG;
    }
    return CDefView_SelectItem_v1_body(this, pidl, SVSI_EDIT | SVSI_ENSUREVISIBLE | SVSI_FOCUSED | SVSI_SELECT);
}

pub unsafe fn CDefView_SelectAndPositionItem_helper_body(this: *mut CDefView, Idx: i32, fSVSI: UINT, ppt: *mut POINT) -> HRESULT {
    if Idx == -1 {
        if fSVSI == SVSI_DESELECTOTHERS {
            return CDefView_SelectItem_v2_body(this, -2, fSVSI);
        } else {
            return E_INVALIDARG;
        }
    }
    if !ppt.is_null() {
        ((*this).m_ListView).SetItemPosition(Idx, ppt);
    }
    return CDefView_SelectItem_v2_body(this, Idx, fSVSI);
}

pub unsafe fn CDefView_SelectAndPositionItem_body(this: *mut CDefView, item: PCUITEMID_CHILD, flags: UINT, point: *mut POINT) -> HRESULT {
    if item.is_null() {
        return CDefView_SelectAndPositionItem_helper_body(this, -1, flags, point);
    }
    let idx = CDefView_LV_FindItemByPidl_body(this, item);
    if idx != -1 {
        CDefView_SelectAndPositionItem_helper_body(this, idx, flags, point)
    } else {
        S_FALSE
    }
}

pub unsafe fn CDefView_Rearrange_body(this: *mut CDefView, sort: LPARAM) -> HRESULT {
    CDefView__Sort_body(this, ((LOWORD!(sort & (SHCIDS_COLUMNMASK as LPARAM))) as i32));
    return S_OK;
}

pub unsafe fn CDefView_GetArrangeParam_body(this: *mut CDefView, sort: *mut LPARAM) -> HRESULT {
    let col = (*this).m_sortInfo.ListColumn;
    *sort = if col != LISTVIEW_SORT_INFO::UNSPECIFIEDCOLUMN {
        col as LPARAM
    } else {
        0 as LPARAM
    };
    return if col != LISTVIEW_SORT_INFO::UNSPECIFIEDCOLUMN {
        S_OK
    } else {
        S_FALSE
    };
}

pub unsafe fn CDefView_ArrangeGrid_body(this: *mut CDefView) -> HRESULT {
    (*this).m_ListView.SetExtendedListViewStyle(LVS_EX_SNAPTOGRID, LVS_EX_SNAPTOGRID);
    S_OK
}

pub unsafe fn CDefView_AutoArrange_body(this: *mut CDefView) -> HRESULT {
    let _ = (*this).m_ListView.ModifyStyle(0, LVS_AUTOARRANGE, 0);
    let _ = (*this).m_ListView.Arrange(LVA_DEFAULT);
    S_OK
}

pub unsafe fn CDefView_AddObject_body(this: *mut CDefView, pidl: PITEMID_CHILD, item: *mut UINT) -> HRESULT {
    if (*this).m_ListView.is_null() {
        return E_FAIL;
    }
    *item = CDefView_LV_AddItem_body(this, pidl as PCUITEMID_CHILD) as UINT;
    return if (*item as i32) >= 0 {
        S_OK
    } else {
        E_OUTOFMEMORY
    };
}

pub unsafe fn CDefView_GetObject_v1_body(this: *mut CDefView, pidl: *mut PITEMID_CHILD, item: UINT) -> HRESULT {
    CDefView_Item_body(this, ((item) as i32), pidl)
}

pub unsafe fn CDefView_RemoveObject_body(this: *mut CDefView, pidl: PITEMID_CHILD, item: *mut UINT) -> HRESULT {
    if (*this).m_ListView.m_hWnd.is_null() {
        return E_FAIL;
    }

    if !pidl.is_null() {
        *item = ((CDefView_LV_FindItemByPidl_body(this, ILFindLastID(pidl))) as u32);
        (*this).m_ListView.DeleteItem(((*item) as i32));
    } else {
        *item = 0;
        (*this).m_ListView.DeleteAllItems();
    }

    return S_OK;
}

pub unsafe fn CDefView_GetObjectCount_body(this: *mut CDefView, count: *mut UINT) -> HRESULT {
    unsafe {
        *count = (((*this).m_ListView.GetItemCount()) as u32);
    }
    S_OK
}

pub unsafe fn CDefView_SetObjectCount_body(this: *mut CDefView, count: UINT, flags: UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_UpdateObject_body(this: *mut CDefView, pidl_old: PITEMID_CHILD, pidl_new: PITEMID_CHILD, item: *mut UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_RefreshObject_body(this: *mut CDefView, pidl: PITEMID_CHILD, item: *mut UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_SetRedraw_body(this: *mut CDefView, redraw: BOOL) -> HRESULT {
    if CListView_operator_BOOL_body(&(*this).m_ListView.m_hWnd) != 0 {
        CListView_SetRedraw_body(&mut (*this).m_ListView, redraw);
    }
    S_OK
}

pub unsafe fn CDefView_GetSelectedCount_body(this: *mut CDefView, count: *mut UINT) -> HRESULT {
    *count = (*this).m_ListView.GetSelectedCount();
    return S_OK;
}

pub unsafe fn CDefView_GetSelectedObjects_body(this: *mut CDefView, pidl: *mut *mut PCUITEMID_CHILD, items: *mut UINT) -> HRESULT {
    *items = CDefView_GetSelections_body(this);

    if *items != 0 {
        *pidl = LocalAlloc(
            0,
            ((*items as usize) * ::core::mem::size_of::<PCUITEMID_CHILD>()) as SIZE_T,
        ) as *mut PCUITEMID_CHILD;

        if (*pidl).is_null() {
            return E_OUTOFMEMORY;
        }

        memcpy(
            *pidl as *mut _,
            (*this).m_apidl as *const _,
            ((*items as usize) * ::core::mem::size_of::<PCUITEMID_CHILD>()) as SIZE_T,
        );
    }

    return S_OK;
}

pub unsafe fn CDefView_IsDropOnSource_body(this: *mut CDefView, drop_target: *mut IDropTarget) -> HRESULT {
        // CDefView.cpp `m_pCurDropTarget && m_pSourceDataObject` -- two null TESTS, and the
    // first came back as a call to the CComPtr ASSIGNMENT helper, which would have
    // OVERWRITTEN the field with the argument being compared against it.
    let _ = drop_target;
    if (*this).m_iDragOverItem == -1 || (!(*this).m_pCurDropTarget.is_null() && !(*this).m_pSourceDataObject.is_null()) {
        return S_OK;
    }

    return S_FALSE;
}

pub unsafe fn CDefView_GetDragPoint_body(this: *mut CDefView, pt: *mut POINT) -> HRESULT {
    if pt.is_null() {
        return E_INVALIDARG;
    }

    (*pt) = (*this).m_ptFirstMousePos;
    return S_OK;
}

pub unsafe fn CDefView_GetDropPoint_body(this: *mut CDefView, pt: *mut POINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_MoveIcons_body(this: *mut CDefView, obj: *mut IDataObject) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_SetItemPos_body(this: *mut CDefView, pidl: PCUITEMID_CHILD, pt: *mut POINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_IsBkDropTarget_body(this: *mut CDefView, drop_target: *mut IDropTarget) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_SetClipboard_body(this: *mut CDefView, r#move: BOOL) -> HRESULT {
    unsafe {
        if r#move == 0 {
            return S_OK;
        }

        let mut CutCount: u32 = 0;
        let mut i: i32 = -1;

        loop {
            i = (*this).m_ListView.GetNextItem(i, ((LVNI_SELECTED) as u16));
            if i == -1 {
                break;
            }

            let was_zero = CutCount == 0;
            CutCount = CutCount.wrapping_add(1);

            if was_zero {
                (*this).m_ListView.SendMessageW(
                    LVM_SETCALLBACKMASK,
                    ((*this).m_ListView.SendMessageW(
                        LVM_GETCALLBACKMASK,
                        0,
                        0,
                    ) & !(LVIS_CUT as LRESULT)) as WPARAM,
                    0,
                );
                CDefView_RefreshGhostedState_body(this);
            }

            (*this).m_ListView.SetItemState(i, LVIS_CUT, LVIS_CUT);
        }

        if CutCount != 0 {
            (*this).m_ClipboardChain.Hook((*this).m_hWnd);
            (*this).m_HasCutItems = true;
        }

        S_OK
    }
}

pub unsafe fn CDefView_SetPoints_body(this: *mut CDefView, obj: *mut IDataObject) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_GetItemSpacing_body(this: *mut CDefView, spacing: *mut ITEMSPACING) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_SetCallback_body(this: *mut CDefView, new_cb: *mut IShellFolderViewCB, old_cb: *mut *mut IShellFolderViewCB) -> HRESULT {
    unsafe {
        if !old_cb.is_null() {
            *old_cb = (*this).m_pShellFolderViewCB;
            (*this).m_pShellFolderViewCB = core::ptr::null_mut();
        }

        CDefView_cpp_com_assign_m_pShellFolderViewCB(this, new_cb);
        CDefView_cpp_com_assign_m_pFolderFilter(this, core::ptr::null_mut());
        if !new_cb.is_null() {
            ((*(*new_cb).lpVtbl).QueryInterface)(new_cb, core::ptr::addr_of!(IID_IFolderFilter), core::ptr::addr_of_mut!((*this).m_pFolderFilter).cast());
        }
        return S_OK;
    }
}

pub unsafe fn CDefView_Select_body(this: *mut CDefView, flags: UINT) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_QuerySupport_body(this: *mut CDefView, support: *mut UINT) -> HRESULT { S_OK }

pub unsafe fn CDefView_SetAutomationObject_body(this: *mut CDefView, disp: *mut IDispatch) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_QueryStatus_body(this: *mut CDefView, pguidCmdGroup: *const GUID, cCmds: ULONG, prgCmds: *mut OLECMD, pCmdText: *mut OLECMDTEXT) -> HRESULT {
    if prgCmds.is_null() {
        return E_INVALIDARG;
    }

    let mut i = 0;
    while i < cCmds {
        unsafe {
            (*prgCmds.add(i as usize)).cmdf = 0;
        }
        i += 1;
    }

    OLECMDERR_E_UNKNOWNGROUP
}

pub unsafe fn CDefView_Exec_body(this: *mut CDefView, pguidCmdGroup: *const GUID, nCmdID: DWORD, nCmdexecopt: DWORD, pvaIn: *mut VARIANT, pvaOut: *mut VARIANT) -> HRESULT {
    if pguidCmdGroup.is_null() {
        return OLECMDERR_E_UNKNOWNGROUP;
    }

    if IsEqualCLSID(pguidCmdGroup.cast(), core::ptr::addr_of!((*this).m_Category).cast()) != 0 {
        if nCmdID == FCIDM_SHVIEW_AUTOARRANGE {
            if V_VT!(pvaIn) != VT_INT_PTR as VARTYPE {
                return OLECMDERR_E_NOTSUPPORTED;
            }

            let mut params: TPMPARAMS = core::mem::zeroed();
            params.cbSize = core::mem::size_of::<TPMPARAMS>() as u32;
            params.rcExclude = *((*pvaIn).n1.n2.n3.pintVal as *const RECT);

            if (*this).m_hMenuViewModes as usize != 0 {
                let hmenuViewPopup: HMENU = CreatePopupMenu();
                Shell_MergeMenus(
                    hmenuViewPopup.cast(),
                    (*this).m_hMenuViewModes.cast(),
                    0,
                    0,
                    0xFFFF,
                    0,
                );
                DeleteMenu(
                    hmenuViewPopup,
                    ((GetMenuItemCount(hmenuViewPopup) - 1) as u32),
                    MF_BYPOSITION,
                );
                DeleteMenu(
                    hmenuViewPopup,
                    ((GetMenuItemCount(hmenuViewPopup) - 1) as u32),
                    MF_BYPOSITION,
                );
                CDefView_CheckViewMode_body(this, hmenuViewPopup);
                TrackPopupMenuEx(
                    hmenuViewPopup,
                    TPM_LEFTALIGN | TPM_TOPALIGN,
                    params.rcExclude.left,
                    params.rcExclude.bottom,
                    (*this).m_hWndParent,
                    &mut params as *mut _,
                );
                DestroyMenu(hmenuViewPopup);
            }

            V_VT!(pvaOut) = VT_I4 as VARTYPE;
            (*pvaOut).n1.n2.n3.lVal = 0x403;
        }
    }

    if cpp_com_iid_equal(pguidCmdGroup, &CGID_Explorer as *const _)
        && nCmdID == 0x29
        && nCmdexecopt == 4
        && !pvaOut.is_null()
    {
        return S_OK;
    }

    if cpp_com_iid_equal(pguidCmdGroup, &CGID_ShellDocView as *const _)
        && nCmdID == 9
        && nCmdexecopt == 0
    {
        return 1;
    }

    if cpp_com_iid_equal(pguidCmdGroup, &CGID_DefView as *const _) {
        let mut pStream: *mut IStream = core::ptr::null_mut();
        let _pStream_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pStream));
        let mut SubKey = [0u16; ((MAX_PATH) as usize)];

        match nCmdID {
            DVCMDID_SET_DEFAULTFOLDER_SETTINGS => {
                SHDeleteKeyW(HKEY_CURRENT_USER, CPP_COM_WSTR_A95A961DFF0A.as_ptr());
                if CDefView_GetDefaultViewStream_body(
                    this,
                    STGM_WRITE,
                    &mut pStream,
                ) >= 0
                {
                    CDefView_SaveViewState_helper_body(this, pStream);
                }
                if !pStream.is_null() {
                    ((*(*pStream).lpVtbl).Release)(pStream);
                    pStream = core::ptr::null_mut();
                }
                return S_OK;
            }
            DVCMDID_RESET_DEFAULTFOLDER_SETTINGS => {
                PathCombineW(
                    SubKey.as_mut_ptr(),
                    REGSTR_PATH_EXPLORER.as_ptr(),
                    CPP_COM_WSTR_A7D0FE10F8A8.as_ptr(),
                );
                SHDeleteKeyW(HKEY_CURRENT_USER, SubKey.as_ptr());
                SHDeleteKeyW(HKEY_CURRENT_USER, CPP_COM_WSTR_A95A961DFF0A.as_ptr());
                (*this).m_FolderSettings.fFlags |= FWF_NOBROWSERVIEWSTATE;
                return S_OK;
            }
            _ => {}
        }
    }

    return OLECMDERR_E_UNKNOWNGROUP;
}

pub unsafe fn CDefView_drag_notify_subitem_body(this: *mut CDefView, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let mut lResult: LONG;
    let mut hr: HRESULT;
    let mut clientRect: RECT = unsafe { core::mem::zeroed() };

    (*this).m_grfKeyState = grfKeyState;

    let mut htinfo: LVHITTESTINFO = LVHITTESTINFO {
        pt: POINT { x: pt.x, y: pt.y },
        flags: LVHT_ONITEM,
        iItem: 0,
        iSubItem: 0,
    };

    ScreenToClient((*this).m_ListView.m_hWnd, &mut htinfo.pt);
    lResult = ListView_HitTest((*this).m_ListView.m_hWnd, &mut htinfo);

    GetClientRect((*this).m_ListView.m_hWnd, &mut clientRect);

    if htinfo.pt.x == (*this).m_ptLastMousePos.x
        && htinfo.pt.y == (*this).m_ptLastMousePos.y
        && (htinfo.pt.x < ((SCROLLAREAWIDTH) as i32)
            || htinfo.pt.x > clientRect.right - ((SCROLLAREAWIDTH) as i32)
            || htinfo.pt.y < ((SCROLLAREAWIDTH) as i32)
            || htinfo.pt.y > clientRect.bottom - ((SCROLLAREAWIDTH) as i32))
    {
        (*this).m_cScrollDelay = ((*this).m_cScrollDelay + 1) % 5;
        if (*this).m_cScrollDelay == 0 {
            if htinfo.pt.x < ((SCROLLAREAWIDTH) as i32) {
                SendMessageW((*this).m_ListView.m_hWnd, WM_HSCROLL, SB_LINEUP as WPARAM, 0);
            }
            if htinfo.pt.x > clientRect.right - ((SCROLLAREAWIDTH) as i32) {
                SendMessageW((*this).m_ListView.m_hWnd, WM_HSCROLL, SB_LINEDOWN as WPARAM, 0);
            }
            if htinfo.pt.y < ((SCROLLAREAWIDTH) as i32) {
                SendMessageW((*this).m_ListView.m_hWnd, WM_VSCROLL, SB_LINEUP as WPARAM, 0);
            }
            if htinfo.pt.y > clientRect.bottom - ((SCROLLAREAWIDTH) as i32) {
                SendMessageW((*this).m_ListView.m_hWnd, WM_VSCROLL, SB_LINEDOWN as WPARAM, 0);
            }
        }
    } else {
        (*this).m_cScrollDelay = 0;
    }

    (*this).m_ptLastMousePos = htinfo.pt;
    ClientToListView_body((*this).m_ListView.m_hWnd, &mut (*this).m_ptLastMousePos);

    if lResult != -1 && !((*this).m_pSourceDataObject).is_null() {
        let pidl: PCUITEMID_CHILD = CDefView__PidlByItem_body(this, lResult);

        for i in 0..(*this).m_cidl {
            if pidl == *((*this).m_apidl.offset(i as isize)) {
                lResult = -1;
                break;
            }
        }
    }

    if !((*this).m_pCurDropTarget).is_null() && lResult == (*this).m_iDragOverItem {
        return ((*(*(*this).m_pCurDropTarget).lpVtbl).DragOver)((*this).m_pCurDropTarget, grfKeyState, pt, pdwEffect);
    }

    if !((*this).m_pCurDropTarget).is_null() {
        let pidl: PCUITEMID_CHILD = CDefView__PidlByItem_body(this, (*this).m_iDragOverItem);
        if !pidl.is_null() {
            CDefView_SelectItem_v1_body(this, pidl, 0);
        }

        ((*(*(*this).m_pCurDropTarget).lpVtbl).DragLeave)((*this).m_pCurDropTarget);
        ((*(*(*this).m_pCurDropTarget).lpVtbl).Release)((*this).m_pCurDropTarget);
        (*this).m_pCurDropTarget = core::ptr::null_mut();
    }

    (*this).m_iDragOverItem = lResult;

    if lResult == -1 {
        hr = ((*(*(*this).m_pSFParent).lpVtbl).CreateViewObject)((*this).m_pSFParent, core::ptr::null_mut(), core::ptr::addr_of!(IID_IDropTarget), core::ptr::addr_of_mut!((*this).m_pCurDropTarget).cast());
    } else {
        let mut pidl: PCUITEMID_CHILD = CDefView__PidlByItem_body(this, lResult);

        hr = ((*(*(*this).m_pSFParent).lpVtbl).GetUIObjectOf)((*this).m_pSFParent, (*this).m_ListView.m_hWnd.cast(), 1, &mut pidl, core::ptr::addr_of!(IID_IDropTarget), core::ptr::null_mut(), core::ptr::addr_of_mut!((*this).m_pCurDropTarget).cast());
    }

    IUnknown_SetSite((*this).m_pCurDropTarget.cast(), (this as *mut IUnknown).cast());

    if hr < 0 {
        *pdwEffect = DROPEFFECT_NONE;
        return hr;
    }

    if (*this).m_iDragOverItem != -1 {
        CDefView_SelectItem_v2_body(this, (*this).m_iDragOverItem, SVSI_SELECT);
    }

    return ((*(*(*this).m_pCurDropTarget).lpVtbl).DragEnter)((*this).m_pCurDropTarget, (*this).m_pCurDataObject, grfKeyState, pt, pdwEffect);
}

pub unsafe fn CDefView_DragEnter_body(this: *mut CDefView, pDataObject: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    if (*pdwEffect) == DROPEFFECT_NONE {
        return S_OK;
    }

    CDefView_cpp_com_assign_m_pCurDataObject(this, pDataObject);

    let hr: HRESULT = CDefView_drag_notify_subitem_body(this, grfKeyState, pt, pdwEffect);
    if hr >= 0 {
        let mut ptClient = POINT { x: pt.x, y: pt.y };
        ScreenToClient((*this).m_hWnd, &mut ptClient);
        ImageList_DragEnter((*this).m_hWnd, ptClient.x, ptClient.y);
    }

    hr
}

pub unsafe fn CDefView_DragOver_body(this: *mut CDefView, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let mut ptClient = POINT { x: pt.x, y: pt.y };
    ScreenToClient((*this).m_hWnd, &mut ptClient);
    ImageList_DragMove(ptClient.x, ptClient.y);
    return CDefView_drag_notify_subitem_body(this, grfKeyState, pt, pdwEffect);
}

pub unsafe fn CDefView_DragLeave_body(this: *mut CDefView) -> HRESULT {
    ImageList_DragLeave((*this).m_hWnd);

    if !(*this).m_pCurDropTarget.is_null() {
        let _ = ((*(*(*this).m_pCurDropTarget).lpVtbl).DragLeave)((*this).m_pCurDropTarget);
        let pDropTarget = (*this).m_pCurDropTarget;
        (*this).m_pCurDropTarget = core::ptr::null_mut();
        let _ = ((*(*pDropTarget).lpVtbl).Release)(pDropTarget);
    }

    if !(*this).m_pCurDataObject.is_null() {
        let pDataObject = (*this).m_pCurDataObject;
        (*this).m_pCurDataObject = core::ptr::null_mut();
        let _ = ((*(*pDataObject).lpVtbl).Release)(pDataObject);
    }

    (*this).m_iDragOverItem = 0;

    return S_OK;
}

pub unsafe fn CDefView__FindInsertableIndexFromPoint_body(this: *mut CDefView, mut pt: POINT) -> INT {
    let mut rcBound: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut i: INT;
    let nCount: INT = (*this).m_ListView.GetItemCount();
    let mut dwSpacing: DWORD;
    let mut dx: INT;
    let mut dy: INT;
    let bSmall: BOOL = (((*this).m_ListView.GetStyle() & LVS_TYPEMASK) != LVS_ICON ) as BOOL;

    // FIXME: LVM_GETORIGIN is broken. See CORE-17266
    pt.x += (*this).m_ListView.GetScrollPos(((SB_HORZ) as i32));
    pt.y += (*this).m_ListView.GetScrollPos(((SB_VERT) as i32));

    if ((*this).m_ListView.GetStyle() & LVS_ALIGNLEFT) != 0 {
        i = 0;
        while i < nCount {
            dwSpacing = ListView_GetItemSpacing((*this).m_ListView.m_hWnd, bSmall);
            dx = LOWORD!(dwSpacing) as INT;
            dy = HIWORD(dwSpacing) as INT;
            ListView_GetItemRect((*this).m_ListView.m_hWnd, i, &mut rcBound, ((LVIR_SELECTBOUNDS) as i32));
            rcBound.right = rcBound.left + dx;
            rcBound.bottom = rcBound.top + dy;
            if pt.x < rcBound.right && pt.y < (rcBound.top + rcBound.bottom) / 2 {
                return i;
            }
            i += 1;
        }
        i = nCount - 1;
        while i >= 0 {
            ListView_GetItemRect((*this).m_ListView.m_hWnd, i, &mut rcBound, ((LVIR_SELECTBOUNDS) as i32));
            if rcBound.left < pt.x && rcBound.top < pt.y {
                return i + 1;
            }
            i -= 1;
        }
    } else {
        i = 0;
        while i < nCount {
            dwSpacing = ListView_GetItemSpacing((*this).m_ListView.m_hWnd, bSmall);
            dx = LOWORD!(dwSpacing) as INT;
            dy = HIWORD(dwSpacing) as INT;
            ListView_GetItemRect((*this).m_ListView.m_hWnd, i, &mut rcBound, ((LVIR_SELECTBOUNDS) as i32));
            rcBound.right = rcBound.left + dx;
            rcBound.bottom = rcBound.top + dy;
            if pt.y < rcBound.bottom && pt.x < rcBound.left {
                return i;
            }
            if pt.y < rcBound.bottom && pt.x < rcBound.right {
                return i + 1;
            }
            i += 1;
        }
        i = nCount - 1;
        while i >= 0 {
            ListView_GetItemRect((*this).m_ListView.m_hWnd, i, &mut rcBound, ((LVIR_SELECTBOUNDS) as i32));
            if rcBound.left < pt.x && rcBound.top < pt.y {
                return i + 1;
            }
            i -= 1;
        }
    }

    return nCount;
}

pub unsafe fn CDefView__HandleStatusBarResize_body(this: *mut CDefView, nWidth: i32) -> () {
    let mut lResult = core::mem::MaybeUninit::<LRESULT>::uninit();

    if !(*this).m_isFullStatusBar {
        let mut nPartArray: [i32; 1] = [-1];
        let shell_browser = (*this).m_pShellBrowser;
        ((*(*shell_browser).lpVtbl).SendControlMsg)(
            shell_browser,
            FCW_STATUS,
            SB_SETPARTS,
            nPartArray.len() as WPARAM,
            nPartArray.as_mut_ptr() as LPARAM,
            lResult.as_mut_ptr()
        );
        return;
    }

    let mut nFileSizePartLength: i32 = 125;
    let nLocationPartLength: i32 = 150;
    let nRightPartsLength: i32 = nFileSizePartLength + nLocationPartLength;
    let mut nObjectsPartLength: i32 = nWidth - nRightPartsLength;

    if nObjectsPartLength <= nLocationPartLength {
        nFileSizePartLength = nWidth / 3;
        nObjectsPartLength = nFileSizePartLength;
    }

    let mut nPartArray: [i32; 3] = [
        nObjectsPartLength,
        nObjectsPartLength + nFileSizePartLength,
        -1,
    ];

    let shell_browser = (*this).m_pShellBrowser;
    ((*(*shell_browser).lpVtbl).SendControlMsg)(
        shell_browser,
        FCW_STATUS,
        SB_SETPARTS,
        nPartArray.len() as WPARAM,
        nPartArray.as_mut_ptr() as LPARAM,
        lResult.as_mut_ptr()
    );
}

pub unsafe fn CDefView__ForceStatusBarResize_body(this: *mut CDefView) -> () {
    let mut fStatusBar: HWND = core::mem::zeroed();
    ((*(*(*this).m_pShellBrowser).lpVtbl).GetControlWindow)(
        (*this).m_pShellBrowser,
        FCW_STATUS,
        &mut fStatusBar as *mut HWND,
    );

    let mut statusBarSize: RECT = core::mem::zeroed();
    GetWindowRect(fStatusBar, &mut statusBarSize as *mut RECT);

    CDefView__HandleStatusBarResize_body(this, statusBarSize.right - statusBarSize.left);
}

pub unsafe fn CDefView__MoveSelectionOnAutoArrange_body(this: *mut CDefView, pt: POINT) -> () {
    let iPosition: INT = CDefView__FindInsertableIndexFromPoint_body(this, pt);

    let mut array = CSimpleArray::<INT>::new();
    let nCount: INT = ((*this).m_ListView).GetItemCount();
    let mut i: INT = 0;
    while i < nCount {
        array.Add(i);
        i += 1;
    }

    let mut iItem: INT = -1;
    loop {
        iItem = ((*this).m_ListView).GetNextItem(iItem, ((LVNI_SELECTED) as u16));
        if iItem < 0 {
            break;
        }
        let mut iFrom: INT = iItem;
        let mut iTo: INT = iPosition;
        if iFrom < iTo {
            iTo -= 1;
        }
        if iFrom >= nCount {
            iFrom = nCount - 1;
        }
        if iTo >= nCount {
            iTo = nCount - 1;
        }

        if iFrom < iTo {
            let mut i: INT = iFrom;
            while i < iTo {
                let tmp: INT = *array.GetAt(i);
                *array.GetAt(i) = *array.GetAt(i + 1);
                *array.GetAt(i + 1) = tmp;
                i += 1;
            }
        } else {
            let mut i: INT = iFrom;
            while i > iTo {
                let tmp: INT = *array.GetAt(i);
                *array.GetAt(i) = *array.GetAt(i - 1);
                *array.GetAt(i - 1) = tmp;
                i -= 1;
            }
        }
    }

    let mut map = CLParamIndexMap::new();
    let mut i: INT = 0;
    while i < nCount {
        let lParam: LPARAM = ((*this).m_ListView).GetItemData(*array.GetAt(i)) as LPARAM;
        map.Add(lParam, i);
        i += 1;
    }

    ((*this).m_ListView).SortItems(Some(SelectionMoveCompareFunc), core::ptr::addr_of!(map) as *mut core::ffi::c_void);
}

pub unsafe fn CDefView_Drop_body(this: *mut CDefView, pDataObject: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    ImageList_DragLeave((*this).m_hWnd);
    ImageList_EndDrag();

    if CDefView_IsDropOnSource_body(this, core::ptr::null_mut()) == S_OK &&
        (*pdwEffect & DROPEFFECT_MOVE) != 0 &&
        ((*this).m_grfKeyState & MK_LBUTTON) != 0
    {
        let drop_target = (*this).m_pCurDropTarget;
        if !drop_target.is_null() {
            ((*(*drop_target).lpVtbl).DragLeave)(drop_target);
            (*this).m_pCurDropTarget = core::ptr::null_mut();
            ((*(*drop_target).lpVtbl).Release)(drop_target);
        }

        let mut ptDrop = POINT { x: pt.x, y: pt.y };
        ScreenToClient((*this).m_ListView.m_hWnd, &mut ptDrop);
        ClientToListView_body((*this).m_ListView.m_hWnd, &mut ptDrop);
        (*this).m_ptLastMousePos = ptDrop;

        ((*this).m_ListView).SetRedraw(FALSE);
        if (((*this).m_ListView).GetStyle() & LVS_AUTOARRANGE) != 0 {
            CDefView__MoveSelectionOnAutoArrange_body(this, (*this).m_ptLastMousePos);
        } else {
            let mut ptItem = POINT { x: 0, y: 0 };
            let mut iItem: i32 = -1;
            while {
                iItem = ((*this).m_ListView).GetNextItem(iItem, ((LVNI_SELECTED) as u16));
                iItem >= 0
            } {
                if ((*this).m_ListView).GetItemPosition(iItem, &mut ptItem) != 0 {
                    ptItem.x += (*this).m_ptLastMousePos.x - (*this).m_ptFirstMousePos.x;
                    ptItem.y += (*this).m_ptLastMousePos.y - (*this).m_ptFirstMousePos.y;
                    ((*this).m_ListView).SetItemPosition(iItem, &mut ptItem);
                }
            }
        }
        ((*this).m_ListView).SetRedraw(TRUE);
    } else {
        let drop_target = (*this).m_pCurDropTarget;
        if !drop_target.is_null() {
            ((*(*drop_target).lpVtbl).Drop)(drop_target, pDataObject, grfKeyState, pt, pdwEffect);
            (*this).m_pCurDropTarget = core::ptr::null_mut();
            ((*(*drop_target).lpVtbl).Release)(drop_target);
        }
    }

    let data_object = (*this).m_pCurDataObject;
    if !data_object.is_null() {
        (*this).m_pCurDataObject = core::ptr::null_mut();
        ((*(*data_object).lpVtbl).Release)(data_object);
    }

    (*this).m_iDragOverItem = 0;
    S_OK
}

pub unsafe fn CDefView_QueryContinueDrag_body(this: *mut CDefView, fEscapePressed: BOOL, grfKeyState: DWORD) -> HRESULT {
    if fEscapePressed != 0 {
        DRAGDROP_S_CANCEL
    } else if (grfKeyState & MK_LBUTTON) == 0 && (grfKeyState & MK_RBUTTON) == 0 {
        DRAGDROP_S_DROP
    } else {
        S_OK
    }
}

pub unsafe fn CDefView_GiveFeedback_body(this: *mut CDefView, dwEffect: DWORD) -> HRESULT { DRAGDROP_S_USEDEFAULTCURSORS }

pub unsafe fn CDefView_Draw_body(this: *mut CDefView, dwDrawAspect: DWORD, lindex: LONG, pvAspect: *mut core::ffi::c_void, ptd: *mut DVTARGETDEVICE, hdcTargetDev: HDC, hdcDraw: HDC, lprcBounds: LPCRECTL, lprcWBounds: LPCRECTL, pfnContinue: LPFNDRAWCONTINUE, dwContinue: ULONG_PTR) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_GetColorSet_body(this: *mut CDefView, dwDrawAspect: DWORD, lindex: LONG, pvAspect: *mut core::ffi::c_void, ptd: *mut DVTARGETDEVICE, hicTargetDevice: HDC, ppColorSet: *mut *mut LOGPALETTE) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_Freeze_body(this: *mut CDefView, dwDrawAspect: DWORD, lindex: LONG, pvAspect: *mut core::ffi::c_void, pdwFreeze: *mut DWORD) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_Unfreeze_body(this: *mut CDefView, dwFreeze: DWORD) -> HRESULT { E_NOTIMPL }

pub unsafe fn CDefView_SetAdvise_body(this: *mut CDefView, aspects: DWORD, advf: DWORD, pAdvSink: *mut IAdviseSink) -> HRESULT {
    CDefView_cpp_com_assign_m_pAdvSink(this, pAdvSink);
    (*this).m_dwAspects = aspects;
    (*this).m_dwAdvf = advf;

    return S_OK;
}

pub unsafe fn CDefView_GetAdvise_body(this: *mut CDefView, pAspects: *mut DWORD, pAdvf: *mut DWORD, ppAdvSink: *mut *mut IAdviseSink) -> HRESULT {
    if !ppAdvSink.is_null() {
        let pSink = (*this).m_pAdvSink;
        *ppAdvSink = pSink;
        ((*(*pSink).lpVtbl).AddRef)(pSink);
    }

    if !pAspects.is_null() {
        *pAspects = (*this).m_dwAspects;
    }

    if !pAdvf.is_null() {
        *pAdvf = (*this).m_dwAdvf;
    }

    return S_OK;
}

pub unsafe fn CDefView_QueryService_body(this: *mut CDefView, guidService: REFGUID, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    if cpp_com_iid_equal(guidService, core::ptr::addr_of!(IID_IFolderView)) {
        return CDefView_QueryInterface(this, riid, ppvObject);
    }
    if !(*this).m_pShellBrowser.is_null() {
        IUnknown_QueryService((((*this).m_pShellBrowser) as _), guidService, riid, ppvObject)
    } else {
        E_NOINTERFACE
    }
}

pub unsafe fn CDefView__MergeToolbar_body(this: *mut CDefView) -> HRESULT {
    let mut ptb: *mut IExplorerToolbar = core::ptr::null_mut();
    let _ptb_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(ptb));
    let mut hr: HRESULT = S_OK;

    hr = IUnknown_QueryService(
        (*this).m_pShellBrowser as *mut IUnknown,
        &IID_IExplorerToolbar as *const _,
        core::ptr::addr_of!(IID_IExplorerToolbar), core::ptr::addr_of_mut!(ptb).cast(),
    );
    if hr < 0 {
        if !ptb.is_null() {
            ((*(*ptb).lpVtbl).Release)(ptb);
            ptb = core::ptr::null_mut();
        }
        return hr;
    }

    (*this).m_Category = CGID_DefViewFrame;

    hr = ((*(*ptb).lpVtbl).SetCommandTarget)(
        ptb,
        this.cast(),
        core::ptr::addr_of_mut!((*this).m_Category),
        0,
    );
    if hr < 0 {
        if !ptb.is_null() {
            ((*(*ptb).lpVtbl).Release)(ptb);
            ptb = core::ptr::null_mut();
        }
        return hr;
    }

    if hr == S_FALSE {
        if !ptb.is_null() {
            ((*(*ptb).lpVtbl).Release)(ptb);
            ptb = core::ptr::null_mut();
        }
        return S_OK;
    }

    if !ptb.is_null() {
        ((*(*ptb).lpVtbl).Release)(ptb);
        ptb = core::ptr::null_mut();
    }
    S_OK
}

pub unsafe fn CDefView__DoFolderViewCB_body(this: *mut CDefView, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    let mut hr: HRESULT = E_NOTIMPL;
    if !(*this).m_pShellFolderViewCB.is_null() {
        let p_callback = (*this).m_pShellFolderViewCB;
        hr = ((*(*p_callback).lpVtbl).MessageSFVCB)(p_callback, uMsg, wParam, lParam);
    }
    hr
}


/// `typedef CSimpleMap<LPARAM, INT> CLParamIndexMap` — the lParam→index association the
/// auto-arrange move builds so the sort can recover each item's ORIGINAL position.
type CLParamIndexMap = CSimpleMap<LPARAM, INT>;

unsafe extern "system" fn SelectionMoveCompareFunc(lParam1: LPARAM, lParam2: LPARAM, lParamSort: LPARAM) -> i32 {
    let pmap = lParamSort as *mut CLParamIndexMap;
    let i1 = (*pmap).Lookup(lParam1);
    let i2 = (*pmap).Lookup(lParam2);
    if i1 < i2 {
        return -1;
    }
    if i1 > i2 {
        return 1;
    }
    0
}

// The C++ holds both interface pointers in CComPtr, so `sfv` is released on scope exit while
// `psv` is handed to the caller with Detach() — i.e. transferred WITHOUT a release. Writing
// psv straight into *ppsv and never releasing it is that Detach.
#[no_mangle]
pub unsafe extern "system" fn SHCreateShellFolderView(
    pcsfv: *const SFV_CREATE,
    ppsv: *mut *mut IShellView,
) -> HRESULT {
    if ppsv.is_null() {
        return E_INVALIDARG;
    }

    *ppsv = core::ptr::null_mut();

    if pcsfv.is_null() || (*pcsfv).cbSize as usize != core::mem::size_of::<SFV_CREATE>() {
        return E_INVALIDARG;
    }

    let mut psv: *mut IShellView = core::ptr::null_mut();
    let hRes = CDefView_CreateInstance(
        (*pcsfv).pshf,
        core::ptr::addr_of!(IID_IShellView),
        core::ptr::addr_of_mut!(psv).cast(),
    );
    if hRes < 0 {
        return hRes;
    }

    if !(*pcsfv).psfvcb.is_null() {
        let mut sfv: *mut IShellFolderView = core::ptr::null_mut();
        if ((*(*psv).lpVtbl).QueryInterface)(
            psv,
            core::ptr::addr_of!(IID_IShellFolderView),
            core::ptr::addr_of_mut!(sfv).cast(),
        ) >= 0
        {
            ((*(*sfv).lpVtbl).SetCallback)(sfv, (*pcsfv).psfvcb.cast(), core::ptr::null_mut());
            ((*(*sfv).lpVtbl).Release)(sfv);
        }
    }

    *ppsv = psv;
    hRes
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *    ShellView
//  *
//  *    Copyright 1998,1999    <juergen.schmied@debitel.net>
//  *    Copyright 2022         Russell Johnson <russell.johnson@superdark.net>
//  *
//  * This is the view visualizing the data provided by the shellfolder.
//  * No direct access to data from pidls should be done from here.
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
//  *
//  * FIXME: CheckToolbar: handle the "new folder" and "folder up" button
//  */
//
// /*
// TODO:
// - When editing starts on item, set edit text to for editing value.
// - Fix shell view to handle view mode popup exec.
// - The background context menu should have a pidl just like foreground menus. This
//    causes crashes when dynamic handlers try to use the NULL pidl.
// - Reorder of columns doesn't work - might be bug in comctl32
// */
//
// #include "precomp.h"
//
// #include <atlwin.h>
// #include <ui/rosctrls.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// // It would be easier to allocate these from 1 and up but the original code used the entire
// // FCIDM_SHVIEWFIRST..FCIDM_SHVIEWLAST range when dealing with IContextMenu and to avoid
// // breaking anything relying on this we will allocate our ranges from the end instead.
// enum  {
//     DEFVIEW_ARRANGESORT_MAXENUM = 9, // Limit the number of the visible columns we will display in the submenu
//     DEFVIEW_ARRANGESORT_MAX = DEFVIEW_ARRANGESORT_MAXENUM + 1, // Reserve one extra for the current sort-by column
//     DVIDM_ARRANGESORT_LAST = FCIDM_SHVIEWLAST,
//     DVIDM_ARRANGESORT_FIRST = DVIDM_ARRANGESORT_LAST - (DEFVIEW_ARRANGESORT_MAX - 1),
//     DVIDM_COMMDLG_SELECT = DVIDM_ARRANGESORT_FIRST - 1,
//
//     DVIDM_CONTEXTMENU_LAST = DVIDM_COMMDLG_SELECT - 1,
//     // FIXME: FCIDM_SHVIEWFIRST is 0 and using that with QueryContextMenu is a
//     // bad idea because it hides bugs related to the ids in ici.lpVerb.
//     // CONTEXT_MENU_BASE_ID acknowledges this but failed to apply the fix everywhere.
//     DVIDM_CONTEXTMENU_FIRST = FCIDM_SHVIEWFIRST,
// };
// #undef FCIDM_SHVIEWLAST // Don't use this constant, change DVIDM_CONTEXTMENU_LAST if you need a new id.
//
// struct LISTVIEW_SORT_INFO
// {
//     INT8    Direction;
//     bool    bLoadedFromViewState;
//     bool    bColumnIsFolderColumn;
//     UINT8   Reserved; // Unused
//     INT     ListColumn;
//     INT     FolderColumn; // Only valid during a sorting operation
//
//     enum { UNSPECIFIEDCOLUMN = -1 };
// static UINT
// GetContextMenuFlags(IShellBrowser *pSB, SFGAOF sfgao)
// {
//     UINT cmf = CMF_NORMAL;
//     if (GetKeyState(VK_SHIFT) < 0)
//         cmf |= CMF_EXTENDEDVERBS;
//     if (sfgao & SFGAO_CANRENAME)
//         cmf |= CMF_CANRENAME;
//     HWND hwnd = NULL;
//     if (pSB && SUCCEEDED(pSB->GetControlWindow(FCW_TREE, &hwnd)) && hwnd)
//         cmf |= CMF_EXPLORE;
//     return cmf;
// }
// static void
// ClientToListView(HWND hwndLV, POINT *ppt)
// {
//     POINT Origin = {};
//
//     // FIXME: LVM_GETORIGIN is broken. See CORE-17266
//     if (!ListView_GetOrigin(hwndLV, &Origin))
//         return;
//
//     ppt->x += Origin.x;
//     ppt->y += Origin.y;
// }
// static BOOL AppendMenuItem(HMENU hMenu, UINT MF, UINT Id, LPCWSTR String, SIZE_T Data = 0)
// {
//     MENUITEMINFOW mii;
//     mii.cbSize = sizeof(mii);
//     mii.fMask = MIIM_TYPE | MIIM_ID | MIIM_DATA | MIIM_STATE;
//     mii.fState = MF & (MFS_CHECKED | MFS_DISABLED);
//     mii.fType = MF & ~mii.fState;
//     mii.wID = Id;
//     mii.dwTypeData = const_cast<LPWSTR>(String);
//     mii.dwItemData = Data as ULONG_PTR;
//     return InsertMenuItemW(hMenu, -1, TRUE, &mii);
// }
// static SIZE_T GetMenuItemDataById(HMENU hMenu, UINT Id)
// {
//     MENUITEMINFOW mii;
//     mii.cbSize = FIELD_OFFSET(MENUITEMINFOW, hbmpItem);
//     mii.fMask = MIIM_DATA;
//     if (GetMenuItemInfoW(hMenu, Id, FALSE, &mii))
//         return mii.dwItemData as SIZE_T;
//     else
//         return 0;
// }
// static HMENU GetSubmenuByID(HMENU hmenu, UINT id)
// {
//     MENUITEMINFOW mii = {sizeof(mii), MIIM_SUBMENU};
//     if (::GetMenuItemInfoW(hmenu, id, FALSE, &mii))
//         return mii.hSubMenu;
//
//     return NULL;
// }
// static UINT ReallyGetMenuItemID(HMENU hmenu, int i)
// {
//     MENUITEMINFOW mii = {sizeof(mii), MIIM_ID};
//     if (::GetMenuItemInfoW(hmenu, i, TRUE, &mii))
//         return mii.wID;
//
//     return UINT_MAX;
// }
// static UINT CalculateCharWidth(HWND hwnd)
// {
//     UINT ret = 0;
//     HDC hDC = GetDC(hwnd);
//     if (hDC)
//     {
//         HGDIOBJ hOrg = SelectObject(hDC, (HGDIOBJ)SendMessage(hwnd, WM_GETFONT, 0, 0));
//         ret = GdiGetCharDimensions(hDC, NULL, NULL);
//         SelectObject(hDC, hOrg);
//         ReleaseDC(hwnd, hDC);
//     }
//     return ret;
// }
// static inline COLORREF GetViewColor(COLORREF Clr, UINT SysFallback)
// {
//     return Clr != CLR_INVALID ? Clr : GetSysColor(SysFallback);
// }
//     BOOL IsDesktop() const { return m_FolderSettings.fFlags & FWF_DESKTOP; }
//     inline BOOL IsSpecialFolder(int &csidl) const
//     {
//         csidl = m_SpecialFolder;
//         return m_SpecialFolder >= 0;
//     }
//     DWORD GetCommDlgViewFlags()
//     {
//         CComPtr<ICommDlgBrowser2> pcdb2;
//         if (m_pCommDlgBrowser && SUCCEEDED(m_pCommDlgBrowser->QueryInterface(IID_PPV_ARG(ICommDlgBrowser2, &pcdb2))))
//         {
//             DWORD flags;
//             if (SUCCEEDED(pcdb2->GetViewFlags(&flags)))
//                 return flags;
//         }
//         return 0;
//     }
//     static inline bool IsSupportedFolderViewMode(int Mode)
//     {
//         return Mode >= FVM_FIRST && Mode <= FVM_DETAILS; // We don't support Tile nor Thumbstrip
//     }
//     static ATL::CWndClassInfo& GetWndClassInfo()
//     {
//         static ATL::CWndClassInfo wc =
//         {
//             {   sizeof(WNDCLASSEX), CS_PARENTDC, StartWindowProc,
//                 0, 0, NULL, NULL,
//                 LoadCursor(NULL, IDC_ARROW), NULL, NULL, L"SHELLDLL_DefView", NULL
//             },
//             NULL, NULL, IDC_ARROW, TRUE, 0, _T("")
//         };
//         return wc;
//     }
//     virtual WNDPROC GetWindowProc() override
//     {
//         return WindowProc;
//     }
//     static LRESULT __stdcall WindowProc(HWND hWnd, UINT uMsg, WPARAM wParam, LPARAM lParam)
//     {
//         CDefView *pThis;
//         LRESULT  result;
//
//         // Must hold a reference during message handling
//         pThis = reinterpret_cast<CDefView *>(hWnd);
//         pThis->AddRef();
//         result = CWindowImpl<CDefView, CWindow, CControlWinTraits>::WindowProc(hWnd, uMsg, wParam, lParam);
//         pThis->Release();
//         return result;
//     }
// HRESULT WINAPI CDefView::Initialize(IShellFolder *shellFolder)
// {
//     m_pSFParent = shellFolder;
//     shellFolder->QueryInterface(IID_PPV_ARG(IShellFolder2, &m_pSF2Parent));
//     shellFolder->QueryInterface(IID_PPV_ARG(IShellDetails, &m_pSDParent));
//
//     return S_OK;
// }
// HRESULT CDefView::IncludeObject(PCUITEMID_CHILD pidl)
// {
//     HRESULT ret = S_OK;
//     if (m_pCommDlgBrowser && !(GetCommDlgViewFlags() & CDB2GVF_NOINCLUDEITEM))
//     {
//         TRACE("ICommDlgBrowser::IncludeObject pidl=%p\n", pidl);
//         ret = m_pCommDlgBrowser->IncludeObject(this, pidl);
//         TRACE("-- returns 0x%08x\n", ret);
//     }
//     else if (m_pFolderFilter)
//     {
//         ret = m_pFolderFilter->ShouldShow(m_pSFParent, m_pidlParent, pidl);
//     }
//     return ret;
// }
// HRESULT CDefView::OnDefaultCommand()
// {
//     HRESULT ret = S_FALSE;
//
//     if (m_pCommDlgBrowser.p != NULL)
//     {
//         TRACE("ICommDlgBrowser::OnDefaultCommand\n");
//         ret = m_pCommDlgBrowser->OnDefaultCommand(this);
//         TRACE("-- returns 0x%08x\n", ret);
//     }
//
//     return ret;
// }
// HRESULT CDefView::OnStateChange(UINT uFlags)
// {
//     HRESULT ret = S_FALSE;
//
//     if (m_pCommDlgBrowser.p != NULL)
//     {
//         TRACE("ICommDlgBrowser::OnStateChange flags=%x\n", uFlags);
//         ret = m_pCommDlgBrowser->OnStateChange(this, uFlags);
//         TRACE("--\n");
//     }
//
//     return ret;
// }
// void CDefView::CheckToolbar()
// {
//     LRESULT result;
//
//     TRACE("\n");
//
//     if (m_pCommDlgBrowser != NULL)
//     {
//         m_pShellBrowser->SendControlMsg(FCW_TOOLBAR, TB_CHECKBUTTON,
//                                       FCIDM_TB_SMALLICON, (m_FolderSettings.ViewMode == FVM_LIST) ? TRUE : FALSE, &result);
//         m_pShellBrowser->SendControlMsg(FCW_TOOLBAR, TB_CHECKBUTTON,
//                                       FCIDM_TB_REPORTVIEW, (m_FolderSettings.ViewMode == FVM_DETAILS) ? TRUE : FALSE, &result);
//         m_pShellBrowser->SendControlMsg(FCW_TOOLBAR, TB_ENABLEBUTTON,
//                                       FCIDM_TB_SMALLICON, TRUE, &result);
//         m_pShellBrowser->SendControlMsg(FCW_TOOLBAR, TB_ENABLEBUTTON,
//                                       FCIDM_TB_REPORTVIEW, TRUE, &result);
//     }
// }
// void CDefView::UpdateStatusbar()
// {
//     WCHAR szFormat[MAX_PATH];
//     WCHAR szPartText[MAX_PATH];
//     UINT cSelectedItems;
//
//     if (!m_ListView)
//         return;
//
//     cSelectedItems = m_ListView.GetSelectedCount();
//     if (cSelectedItems)
//     {
//         LoadStringW(shell32_hInstance, IDS_OBJECTS_SELECTED, szFormat, _countof(szFormat));
//         StringCchPrintfW(szPartText, _countof(szPartText), szFormat, cSelectedItems);
//     }
//     else
//     {
//         LoadStringW(shell32_hInstance, IDS_OBJECTS, szFormat, _countof(szFormat));
//         StringCchPrintfW(szPartText, _countof(szPartText), szFormat, m_ListView.GetItemCount());
//     }
//
//     LRESULT lResult;
//     m_pShellBrowser->SendControlMsg(FCW_STATUS, SB_SETTEXT, 0, (LPARAM)szPartText, &lResult);
//
//     // Don't bother with the extra processing if we only have one StatusBar part
//     if (m_isFullStatusBar)
//     {
//         UINT64 uTotalFileSize = 0;
//         WORD uFileFlags = LVNI_ALL;
//         INT nItem = -1;
//         bool bIsOnlyFoldersSelected = true;
//
//         // If we have something selected then only count selected file sizes
//         if (cSelectedItems)
//         {
//             uFileFlags = LVNI_SELECTED;
//         }
//
//         while ((nItem = m_ListView.GetNextItem(nItem, uFileFlags)) >= 0)
//         {
//             PCUITEMID_CHILD pidl = _PidlByItem(nItem);
//
//             uTotalFileSize += _ILGetFileSize(pidl, NULL, 0);
//
//             if (!_ILIsFolder(pidl))
//             {
//                 bIsOnlyFoldersSelected = false;
//             }
//         }
//
//         // Don't show the file size text if there is 0 bytes in the folder
//         // OR we only have folders selected
//         if ((cSelectedItems && !bIsOnlyFoldersSelected) || uTotalFileSize)
//             StrFormatByteSizeW(uTotalFileSize, szPartText, _countof(szPartText));
//         else
//             *szPartText = 0;
//
//         m_pShellBrowser->SendControlMsg(FCW_STATUS, SB_SETTEXT, 1, (LPARAM)szPartText, &lResult);
//     }
//
//     SFGAOF att = 0;
//     if (cSelectedItems > 0)
//     {
//         UINT maxquery = 42; // Checking the attributes can be slow, only check small selections (_DoCopyToMoveToFolder will verify the full array)
//         att = SFGAO_CANCOPY | SFGAO_CANMOVE;
//         if (cSelectedItems <= maxquery && (!GetSelections() || FAILED(m_pSFParent->GetAttributesOf(m_cidl, m_apidl, &att))))
//             att = 0;
//     }
//     m_pShellBrowser->SendControlMsg(FCW_TOOLBAR, TB_ENABLEBUTTON, FCIDM_SHVIEW_COPYTO, (att & SFGAO_CANCOPY) != 0, &lResult);
//     m_pShellBrowser->SendControlMsg(FCW_TOOLBAR, TB_ENABLEBUTTON, FCIDM_SHVIEW_MOVETO, (att & SFGAO_CANMOVE) != 0, &lResult);
// }
// void CDefView::UpdateStatusbarLocation()
// {
//     LRESULT lResult;
//     LPARAM pIcon = NULL;
//     WCHAR szPartText[MAX_PATH];
//     *szPartText = 0;
//     if (m_isFullStatusBar)
//     {
//         // If we are in a Recycle Bin then show no text for the location part
//         int csidl;
//         if (!IsSpecialFolder(csidl) || (csidl != CSIDL_NETWORK && csidl != CSIDL_BITBUCKET))
//         {
//             LoadStringW(shell32_hInstance, IDS_MYCOMPUTER, szPartText, _countof(szPartText));
//             pIcon = (LPARAM)m_hMyComputerIcon;
//         }
//         /*else if (csidl == CSIDL_NETWORK) // TODO: Figure out the type of share (My Computer/Local Intranet/Internet?)
//         {
//             ImageList_GetIconSize(ListView_GetImageList(m_ListView, LVSIL_SMALL), &x, &y);
//             pIcon = (LPARAM)LoadImage(shell32_hInstance, MAKEINTRESOURCEW(IDI_SHELL_MY_NETWORK_PLACES),
//                                       IMAGE_ICON, x, y, LR_SHARED);
//         }*/
//     }
//     m_pShellBrowser->SendControlMsg(FCW_STATUS, SB_SETICON, 2, pIcon, &lResult);
//     m_pShellBrowser->SendControlMsg(FCW_STATUS, SB_SETTEXT, 2, (LPARAM)szPartText, &lResult);
// }
// LRESULT CDefView::OnUpdateStatusbar(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     m_ScheduledStatusbarUpdate = false;
//     UpdateStatusbar();
//     return 0;
// }
// BOOL CDefView::CreateList()
// {
//     DWORD dwStyle, dwExStyle, ListExStyle;
//     TRACE("%p\n", this);
//
//     dwStyle = WS_TABSTOP | WS_VISIBLE | WS_CHILDWINDOW | WS_CLIPSIBLINGS | WS_CLIPCHILDREN |
//               LVS_SHAREIMAGELISTS | LVS_EDITLABELS | LVS_AUTOARRANGE; // FIXME: Remove LVS_AUTOARRANGE when the view is able to save ItemPos
//     dwExStyle = (m_FolderSettings.fFlags & FWF_NOCLIENTEDGE) ? 0 : WS_EX_CLIENTEDGE;
//     ListExStyle = LVS_EX_INFOTIP | LVS_EX_LABELTIP;
//
//     if (m_FolderSettings.fFlags & FWF_DESKTOP)
//     {
//         m_FolderSettings.fFlags |= FWF_NOCLIENTEDGE | FWF_NOSCROLL;
//         dwStyle |= LVS_ALIGNLEFT;
//         // LVS_EX_REGIONAL?
//     }
//     else
//     {
//         dwStyle |= LVS_SHOWSELALWAYS; // MSDN says FWF_SHOWSELALWAYS is deprecated, always turn on for folders
//         dwStyle |= (m_FolderSettings.fFlags & FWF_ALIGNLEFT) ? LVS_ALIGNLEFT : LVS_ALIGNTOP;
//     }
//
//     switch (m_FolderSettings.ViewMode)
//     {
//         case FVM_ICON:
//             dwStyle |= LVS_ICON;
//             break;
//         case FVM_DETAILS:
//             dwStyle |= LVS_REPORT;
//             break;
//         case FVM_SMALLICON:
//             dwStyle |= LVS_SMALLICON;
//             break;
//         case FVM_LIST:
//             dwStyle |= LVS_LIST;
//             break;
//         default:
//             dwStyle |= LVS_LIST;
//             break;
//     }
//
//     if (m_FolderSettings.fFlags & FWF_AUTOARRANGE)
//         dwStyle |= LVS_AUTOARRANGE;
//
//     if (m_FolderSettings.fFlags & FWF_SNAPTOGRID)
//         ListExStyle |= LVS_EX_SNAPTOGRID;
//
//     if (m_FolderSettings.fFlags & FWF_SINGLESEL)
//         dwStyle |= LVS_SINGLESEL;
//
//     if (m_FolderSettings.fFlags & FWF_FULLROWSELECT)
//         ListExStyle |= LVS_EX_FULLROWSELECT;
//
//     ListExStyle |= GetItemActivateFlags();
//     if (ListExStyle & LVS_EX_ONECLICKACTIVATE)
//         ListExStyle |= SHELL_GetIconUnderlineFlags();
//
//     if (m_FolderSettings.fFlags & FWF_NOCOLUMNHEADER)
//         dwStyle |= LVS_NOCOLUMNHEADER;
//
//
//     if (m_FolderSettings.fFlags & FWF_NOCLIENTEDGE)
//         dwExStyle &= ~WS_EX_CLIENTEDGE;
//
//     RECT rcListView = {0,0,0,0};
//     m_ListView.Create(m_hWnd, rcListView, L"FolderView", dwStyle, dwExStyle, ID_LISTVIEW);
//
//     if (!m_ListView)
//         return FALSE;
//
//     m_ListView.SetExtendedListViewStyle(ListExStyle);
//
//     /*  UpdateShellSettings(); */
//     return TRUE;
// }
// void CDefView::UpdateListColors()
// {
//     if (m_FolderSettings.fFlags & FWF_DESKTOP)
//     {
//         /* Check if drop shadows option is enabled */
//         BOOL bDropShadow = FALSE;
//         DWORD cbDropShadow = sizeof(bDropShadow);
//
//         /*
//          * The desktop ListView always take the default desktop colours, by
//          * remaining transparent and letting user32/win32k paint itself the
//          * desktop background color, if any.
//          */
//         m_ListView.SetBkColor(CLR_NONE);
//
//         SHGetValueW(HKEY_CURRENT_USER, L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
//                      L"ListviewShadow", NULL, &bDropShadow, &cbDropShadow);
//         if (bDropShadow)
//         {
//             /* Set the icon background transparent */
//             m_ListView.SetTextBkColor(CLR_NONE);
//             m_ListView.SetTextColor(RGB(255, 255, 255));
//             m_ListView.SetExtendedListViewStyle(LVS_EX_TRANSPARENTSHADOWTEXT, LVS_EX_TRANSPARENTSHADOWTEXT);
//         }
//         else
//         {
//             /* Set the icon background as the same colour as the desktop */
//             COLORREF crDesktop = GetSysColor(COLOR_DESKTOP);
//             m_ListView.SetTextBkColor(crDesktop);
//             if (GetRValue(crDesktop) + GetGValue(crDesktop) + GetBValue(crDesktop) > 128 * 3)
//                 m_ListView.SetTextColor(RGB(0, 0, 0));
//             else
//                 m_ListView.SetTextColor(RGB(255, 255, 255));
//             m_ListView.SetExtendedListViewStyle(0, LVS_EX_TRANSPARENTSHADOWTEXT);
//         }
//     }
//     else
//     {
//         m_ListView.SetTextBkColor(GetViewColor(m_viewinfo_data.clrTextBack, COLOR_WINDOW));
//         m_ListView.SetTextColor(GetViewColor(m_viewinfo_data.clrText, COLOR_WINDOWTEXT));
//
//         // Background is painted by the parent via WM_PRINTCLIENT
//         m_ListView.SetExtendedListViewStyle(LVS_EX_TRANSPARENTBKGND, LVS_EX_TRANSPARENTBKGND);
//     }
// }
// BOOL CDefView::InitList()
// {
//     HIMAGELIST big_icons, small_icons;
//
//     TRACE("%p\n", this);
//
//     m_ListView.DeleteAllItems();
//
//     Shell_GetImageLists(&big_icons, &small_icons);
//     m_ListView.SetImageList(big_icons, LVSIL_NORMAL);
//     m_ListView.SetImageList(small_icons, LVSIL_SMALL);
//
//     m_hMenuArrangeModes = CreateMenu();
//
//     SIZE_T *pColumns = m_LoadColumnsList ? (SIZE_T*)DPA_GetPtrPtr(m_LoadColumnsList) : NULL;
//     UINT ColumnCount = pColumns ? DPA_GetPtrCount(m_LoadColumnsList) : 0;
//     LoadColumns(pColumns, ColumnCount);
//     if (m_sortInfo.bColumnIsFolderColumn)
//     {
//         m_sortInfo.bColumnIsFolderColumn = FALSE;
//         HRESULT hr = MapFolderColumnToListColumn(m_sortInfo.ListColumn);
//         m_sortInfo.ListColumn = SUCCEEDED(hr) ? hr : 0;
//     }
//     return TRUE;
// }
// static HRESULT SHGetLVColumnSubItem(HWND List, UINT Col)
// {
//     LVCOLUMN lvc;
//     lvc.mask = LVCF_SUBITEM;
//     if (!ListView_GetColumn(List, Col, &lvc))
//         return E_FAIL;
//     else
//         return lvc.iSubItem;
// }
// HRESULT CDefView::MapFolderColumnToListColumn(UINT FoldCol)
// {
//     // This function is only called during column management, performance is not critical.
//     for (UINT i = 0;; ++i)
//     {
//         HRESULT r = SHGetLVColumnSubItem(m_ListView.m_hWnd, i);
//         if ((UINT)r == FoldCol)
//             return i;
//         else if (FAILED(r))
//             return r;
//     }
// }
// HRESULT CDefView::MapListColumnToFolderColumn(UINT ListCol)
// {
//     // This function is called every time a LVITEM::iSubItem is mapped to
//     // a folder column (calls to GetDetailsOf etc.) and should be fast.
//     if (m_ListToFolderColMap)
//     {
//         UINT count = DPA_GetPtrCount(m_ListToFolderColMap);
//         if (ListCol < count)
//         {
//             HRESULT col = (INT)(INT_PTR)DPA_FastGetPtr(m_ListToFolderColMap, ListCol);
//             assert(col >= 0 && col == SHGetLVColumnSubItem(m_ListView.m_hWnd, ListCol));
//             return col;
//         }
//         else if (count)
//         {
//             TRACE("m_ListToFolderColMap cache miss while mapping %d\n", ListCol);
//         }
//     }
//     return SHGetLVColumnSubItem(m_ListView.m_hWnd, ListCol);
// }
// HRESULT CDefView::GetDetailsByFolderColumn(PCUITEMID_CHILD pidl, UINT FoldCol, SHELLDETAILS &sd)
// {
//     // According to learn.microsoft.com/en-us/windows/win32/shell/sfvm-getdetailsof
//     // the query order is IShellFolder2, IShellDetails, SFVM_GETDETAILSOF.
//     HRESULT hr = E_FAIL;
//     if (m_pSF2Parent)
//     {
//         hr = m_pSF2Parent->GetDetailsOf(pidl, FoldCol, &sd);
//     }
//     if (FAILED(hr) && m_pSDParent)
//     {
//         hr = m_pSDParent->GetDetailsOf(pidl, FoldCol, &sd);
//     }
//     return hr;
// }
// HRESULT CDefView::GetDetailsByListColumn(PCUITEMID_CHILD pidl, UINT ListCol, SHELLDETAILS &sd)
// {
//     HRESULT hr = MapListColumnToFolderColumn(ListCol);
//     if (SUCCEEDED(hr))
//         return GetDetailsByFolderColumn(pidl, hr, sd);
//     ERR("Unable to determine folder column from list column %d\n", (int) ListCol);
//     return hr;
// }
// HRESULT CDefView::LoadColumn(UINT FoldCol, UINT ListCol, BOOL Insert, UINT ForceWidth)
// {
//     WCHAR buf[MAX_PATH];
//     SHELLDETAILS sd;
//     HRESULT hr;
//
//     sd.str_.uType = !STRRET_WSTR; // Make sure "uninitialized" uType is not WSTR
//     hr = GetDetailsByFolderColumn(NULL, FoldCol, sd);
//     if (FAILED(hr))
//         return hr;
//     hr = StrRetToStrNW(buf, _countof(buf), &sd.str, NULL);
//     if (FAILED(hr))
//         return hr;
//
//     UINT chavewidth = CalculateCharWidth(m_ListView.m_hWnd);
//     if (!chavewidth)
//         chavewidth = 6; // 6 is a reasonable default fallback
//
//     LVCOLUMN lvc;
//     lvc.mask = LVCF_TEXT | LVCF_FMT | LVCF_WIDTH | LVCF_SUBITEM;
//     lvc.pszText = buf;
//     lvc.fmt = sd.fmt;
//     lvc.cx = ForceWidth ? ForceWidth : (sd.cxChar * chavewidth); // FIXME: DPI?
//     lvc.iSubItem = FoldCol; // Used by MapFolderColumnToListColumn & MapListColumnToFolderColumn
//     if ((int)ListCol == -1)
//     {
//         assert(Insert); // You can insert at the end but you can't change something that is not there
//         if (Insert)
//             ListCol = 0x7fffffff;
//     }
//     if (Insert)
//         ListView_InsertColumn(m_ListView.m_hWnd, ListCol, &lvc);
//     else
//         ListView_SetColumn(m_ListView.m_hWnd, ListCol, &lvc);
//     return S_OK;
// }
// HRESULT CDefView::LoadColumns(SIZE_T *pColList, UINT ColListCount)
// {
//     HWND hWndHdr = ListView_GetHeader(m_ListView.m_hWnd);
//     UINT newColCount = 0, oldColCount = Header_GetItemCount(hWndHdr);
//     UINT width = 0, foldCol, i;
//     HRESULT hr = S_FALSE;
//
//     m_ListView.SetRedraw(FALSE);
//     for (i = 0, foldCol = 0;; ++foldCol)
//     {
//         if (newColCount >= 0xffff)
//             break; // CompareIDs limit reached
//
//         if (pColList)
//         {
//             if (i >= ColListCount)
//                 break;
//             width = HIWORD(pColList[i]);
//             foldCol = LOWORD(pColList[i++]);
//         }
//
//         SHCOLSTATEF state = 0;
//         if (!m_pSF2Parent || FAILED(m_pSF2Parent->GetDefaultColumnState(foldCol, &state)))
//             state = SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT;
//
//         if (foldCol == 0)
//         {
//             // Force the first column
//         }
//         else if (state & SHCOLSTATE_HIDDEN)
//         {
//             continue;
//         }
//         else if (!pColList && !(state & SHCOLSTATE_ONBYDEFAULT))
//         {
//             continue;
//         }
//
//         bool insert = newColCount >= oldColCount;
//         UINT listCol = insert ? -1 : newColCount;
//         hr = LoadColumn(foldCol, listCol, insert, width);
//         if (FAILED(hr))
//         {
//             if (!pColList)
//                 hr = S_OK; // No more items, we are done
//             break;
//         }
//         ++newColCount;
//     }
//     for (i = newColCount; i < oldColCount; ++i)
//     {
//         ListView_DeleteColumn(m_ListView.m_hWnd, i);
//     }
//
//     m_ListView.SetRedraw(TRUE);
//     ColumnListChanged();
//     assert(SUCCEEDED(MapFolderColumnToListColumn(0))); // We don't allow turning off the Name column
//     if (m_LoadColumnsList)
//     {
//         DPA_Destroy(m_LoadColumnsList);
//         m_LoadColumnsList = NULL;
//     }
//     return hr;
// }
// void CDefView::ColumnListChanged()
// {
//     HDPA cache = m_ListToFolderColMap;
//     m_ListToFolderColMap = NULL; // No cache while we are building the cache
//     DPA_DeleteAllPtrs(cache);
//     for (UINT i = 0;; ++i)
//     {
//         HRESULT hr = MapListColumnToFolderColumn(i);
//         if (FAILED(hr))
//             break; // No more columns
//         if (!DPA_SetPtr(cache, i, (void*)(INT_PTR) hr))
//             break; // Cannot allow holes in the cache, must stop now.
//     }
//     m_ListToFolderColMap = cache;
//
//     for (;;)
//     {
//         if (!RemoveMenu(m_hMenuArrangeModes, 0, MF_BYPOSITION))
//             break;
//     }
//     HMENU hMenu = GetSubmenuByID(m_hMenu, FCIDM_MENU_VIEW);
//     if (hMenu)
//     {
//         hMenu = GetSubmenuByID(hMenu, FCIDM_SHVIEW_ARRANGE);
//         for (UINT i = DVIDM_ARRANGESORT_FIRST; i <= DVIDM_ARRANGESORT_LAST && hMenu; ++i)
//         {
//             RemoveMenu(hMenu, i, MF_BYCOMMAND);
//         }
//         if ((int) GetMenuItemID(hMenu, 0) <= 0)
//             RemoveMenu(hMenu, 0, MF_BYPOSITION); // Separator
//     }
//     WCHAR buf[MAX_PATH];
//     LVCOLUMN lvc;
//     lvc.mask = LVCF_TEXT;
//     lvc.pszText = buf;
//     lvc.cchTextMax = _countof(buf);
//     for (UINT listCol = 0; listCol < DEFVIEW_ARRANGESORT_MAXENUM; ++listCol)
//     {
//         if (!ListView_GetColumn(m_ListView.m_hWnd, listCol, &lvc))
//             break;
//         HRESULT foldCol = MapListColumnToFolderColumn(listCol);
//         assert(SUCCEEDED(foldCol));
//         DBG_UNREFERENCED_LOCAL_VARIABLE(foldCol);
//         AppendMenuItem(m_hMenuArrangeModes, MF_STRING,
//                        DVIDM_ARRANGESORT_FIRST + listCol, lvc.pszText, listCol);
//     }
//
//     ListView_RedrawItems(m_ListView.m_hWnd, 0, 0x7fffffff);
//     m_ListView.InvalidateRect(NULL, TRUE);
// }
// INT __stdcall CDefView::ListViewCompareItems(LPARAM lParam1, LPARAM lParam2, LPARAM lpData)
// {
//     PCUIDLIST_RELATIVE pidl1 = reinterpret_cast<PCUIDLIST_RELATIVE>(lParam1);
//     PCUIDLIST_RELATIVE pidl2 = reinterpret_cast<PCUIDLIST_RELATIVE>(lParam2);
//     CDefView *pThis = reinterpret_cast<CDefView*>(lpData);
//
//     HRESULT hres = pThis->m_pSFParent->CompareIDs(pThis->m_sortInfo.FolderColumn, pidl1, pidl2);
//     if (FAILED_UNEXPECTEDLY(hres))
//         return 0;
//
//     SHORT nDiff = HRESULT_CODE(hres);
//     return nDiff * pThis->m_sortInfo.Direction;
// }
// BOOL CDefView::_Sort(int Col)
// {
//     HWND hHeader;
//     HDITEM hColumn;
//     int prevCol = m_sortInfo.ListColumn;
//     m_sortInfo.bLoadedFromViewState = FALSE;
//
//     // FIXME: Is this correct? Who sets this style?
//     // And if it is set, should it also block sorting using the menu?
//     // Any why should it block sorting when the view is loaded initially?
//     if (m_ListView.GetWindowLongPtr(GWL_STYLE) & LVS_NOSORTHEADER)
//         return TRUE;
//
//     hHeader = ListView_GetHeader(m_ListView.m_hWnd);
//     if (Col != -1)
//     {
//         if (Col >= Header_GetItemCount(hHeader))
//         {
//             ERR("Sort column out of range\n");
//             return FALSE;
//         }
//
//         if (prevCol == Col)
//             m_sortInfo.Direction *= -1;
//         else
//             m_sortInfo.Direction = 0;
//         m_sortInfo.ListColumn = Col;
//     }
//     if (!m_sortInfo.Direction)
//         m_sortInfo.Direction += 1;
//
//     /* If the sorting column changed, remove the sorting style from the old column */
//     if (prevCol != -1 && prevCol != m_sortInfo.ListColumn)
//     {
//         hColumn.mask = HDI_FORMAT;
//         Header_GetItem(hHeader, prevCol, &hColumn);
//         hColumn.fmt &= ~(HDF_SORTUP | HDF_SORTDOWN);
//         Header_SetItem(hHeader, prevCol, &hColumn);
//     }
//
//     /* Set the sorting style on the new column */
//     hColumn.mask = HDI_FORMAT;
//     Header_GetItem(hHeader, m_sortInfo.ListColumn, &hColumn);
//     hColumn.fmt &= ~(HDF_SORTDOWN | HDF_SORTUP);
//     hColumn.fmt |= (m_sortInfo.Direction > 0 ?  HDF_SORTUP : HDF_SORTDOWN);
//     Header_SetItem(hHeader, m_sortInfo.ListColumn, &hColumn);
//
//     m_sortInfo.FolderColumn = MapListColumnToFolderColumn(m_sortInfo.ListColumn);
//     ASSERT(m_sortInfo.Direction == 1 || m_sortInfo.Direction == -1);
//     return m_ListView.SortItems(ListViewCompareItems, this);
// }
// SFGAOF CDefView::GetItemAttributes(PCUITEMID_CHILD pidl, UINT Query)
// {
//     SFGAOF Attr = (SFGAOF)Query;
//     return SUCCEEDED(m_pSFParent->GetAttributesOf(1, &pidl, &Attr)) ? (Attr & Query) : 0;
// }
// SFGAOF CDefView::GetItemAttributes(int i, UINT Query)
// {
//     PCUITEMID_CHILD pItem = _PidlByItem(i);
//     return pItem ? GetItemAttributes(pItem, Query) : 0;
// }
// PCUITEMID_CHILD CDefView::_PidlByItem(int i)
// {
//     if (!m_ListView)
//         return nullptr;
//     return reinterpret_cast<PCUITEMID_CHILD>(m_ListView.GetItemData(i));
// }
// PCUITEMID_CHILD CDefView::_PidlByItem(LVITEM& lvItem)
// {
//     if (!m_ListView)
//         return nullptr;
//     return reinterpret_cast<PCUITEMID_CHILD>(lvItem.lParam);
// }
// int CDefView::LV_FindItemByPidl(PCUITEMID_CHILD pidl)
// {
//     ASSERT(m_ListView && m_pSFParent);
//
//     int cItems = m_ListView.GetItemCount();
//     LPARAM lParam = m_pSF2Parent ? SHCIDS_CANONICALONLY : 0;
//     for (int i = 0; i < cItems; i++)
//     {
//         PCUITEMID_CHILD currentpidl = _PidlByItem(i);
//         HRESULT hr = m_pSFParent->CompareIDs(lParam, pidl, currentpidl);
//         if (SUCCEEDED(hr))
//         {
//             if (hr == S_EQUAL)
//                 return i;
//         }
//         else
//         {
//             for (i = 0; pidl && i < cItems; i++)
//             {
//                 currentpidl = _PidlByItem(i);
//                 if (currentpidl && currentpidl->mkid.cb == pidl->mkid.cb && !memcmp(currentpidl, pidl, pidl->mkid.cb))
//                     DbgPrint("Matched item #%d, broken CompareIDs?\n", i);
//             }
//             break;
//         }
//     }
//     return -1;
// }
// int CDefView::LV_AddItem(PCUITEMID_CHILD pidl)
// {
//     LVITEMW lvItem;
//
//     TRACE("(%p)(pidl=%p)\n", this, pidl);
//
//     ASSERT(m_ListView);
//
//     if (_DoFolderViewCB(SFVM_ADDINGOBJECT, 0, (LPARAM)pidl) == S_FALSE)
//         return -1;
//
//     lvItem.mask = LVIF_TEXT | LVIF_IMAGE | LVIF_PARAM;
//     lvItem.iItem = m_ListView.GetItemCount();             // add item to lists end
//     lvItem.iSubItem = 0;
//     lvItem.lParam = reinterpret_cast<LPARAM>(ILClone(pidl)); // set item's data
//     lvItem.pszText = LPSTR_TEXTCALLBACKW;                 // get text on a callback basis
//     lvItem.iImage = I_IMAGECALLBACK;                      // get image on a callback basis
//     lvItem.stateMask = LVIS_CUT;
//     if (m_HasCutItems)
//     {
//         lvItem.mask |= LVIF_STATE;
//         lvItem.state = GetItemAttributes(pidl, SFGAO_HIDDEN | SFGAO_GHOSTED) ? LVIS_CUT : 0;
//     }
//
//     return m_ListView.InsertItem(&lvItem);
// }
// BOOLEAN CDefView::LV_DeleteItem(PCUITEMID_CHILD pidl)
// {
//     int nIndex;
//
//     TRACE("(%p)(pidl=%p)\n", this, pidl);
//
//     ASSERT(m_ListView);
//
//     nIndex = LV_FindItemByPidl(pidl);
//     if (nIndex < 0)
//         return FALSE;
//
//     _DoFolderViewCB(SFVM_REMOVINGOBJECT, 0, (LPARAM)pidl);
//
//     return m_ListView.DeleteItem(nIndex);
// }
// BOOLEAN CDefView::LV_RenameItem(PCUITEMID_CHILD pidlOld, PCUITEMID_CHILD pidlNew)
// {
//     int nItem;
//     LVITEMW lvItem;
//
//     TRACE("(%p)(pidlold=%p pidlnew=%p)\n", this, pidlOld, pidlNew);
//
//     ASSERT(m_ListView);
//
//     nItem = LV_FindItemByPidl(pidlOld);
//
//     if (-1 != nItem)
//     {
//         lvItem.mask = LVIF_PARAM; // only the pidl
//         lvItem.iItem = nItem;
//         lvItem.iSubItem = 0;
//         m_ListView.GetItem(&lvItem);
//
//         // Store old pidl until new item is replaced
//         LPVOID oldPidl = reinterpret_cast<LPVOID>(lvItem.lParam);
//
//         lvItem.mask = LVIF_PARAM | LVIF_IMAGE | LVIF_TEXT;
//         lvItem.iItem = nItem;
//         lvItem.iSubItem = 0;
//         lvItem.lParam = reinterpret_cast<LPARAM>(ILClone(pidlNew)); // set item's data
//         lvItem.pszText = LPSTR_TEXTCALLBACKW;
//         lvItem.iImage = SHMapPIDLToSystemImageListIndex(m_pSFParent, pidlNew, 0);
//         m_ListView.SetItem(&lvItem);
//         m_ListView.Update(nItem);
//
//         // Now that the new item is in place, we can safely release the old pidl
//         SHFree(oldPidl);
//
//         return TRUE; // FIXME: better handling
//     }
//
//     return FALSE;
// }
// BOOL CDefView::LV_UpdateItem(INT nItem, PCUITEMID_CHILD pidl)
// {
//     BOOL bResult = FALSE;
//     LVITEMW lvItem;
//     if (nItem >= 0)
//     {
//         _DoFolderViewCB(SFVM_UPDATINGOBJECT, nItem, (LPARAM)pidl);
//
//         lvItem.mask = LVIF_IMAGE;
//         lvItem.iItem = nItem;
//         lvItem.iSubItem = 0;
//         lvItem.iImage = SHMapPIDLToSystemImageListIndex(m_pSFParent, pidl, 0);
//         lvItem.stateMask = LVIS_CUT;
//         if (m_HasCutItems)
//         {
//             lvItem.mask |= LVIF_STATE;
//             lvItem.state = GetItemAttributes(pidl, SFGAO_HIDDEN | SFGAO_GHOSTED) ? LVIS_CUT : 0;
//         }
//         PCUITEMID_CHILD pidlOld = _PidlByItem(nItem);
//         if (pidlOld && (lvItem.lParam = reinterpret_cast<LPARAM>(ILClone(pidl))) != NULL)
//             lvItem.mask |= LVIF_PARAM;
//         bResult = m_ListView.SetItem(&lvItem);
//         if (bResult && lvItem.lParam && pidlOld)
//             ILFree(const_cast<PUITEMID_CHILD>(pidlOld));
//         for (UINT i = 0; m_ListView.SetItemText(nItem, i, LPSTR_TEXTCALLBACK); ++i) {} // Update all columns
//     }
//     return bResult;
// }
// BOOL CDefView::LV_UpdateItem(PCUITEMID_CHILD pidl)
// {
//     TRACE("(%p)(pidl=%p)\n", this, pidl);
//     ASSERT(m_ListView);
//     INT nItem = LV_FindItemByPidl(pidl);
//     return nItem >= 0 ? LV_UpdateItem(nItem, pidl) : FALSE;
// }
// void CDefView::LV_RefreshIcon(INT iItem)
// {
//     ASSERT(m_ListView);
//
//     LVITEMW lvItem = { LVIF_IMAGE };
//     lvItem.iItem = iItem;
//     lvItem.iImage = I_IMAGECALLBACK;
//     m_ListView.SetItem(&lvItem);
//     m_ListView.Update(iItem);
// }
// void CDefView::LV_RefreshIcons()
// {
//     ASSERT(m_ListView);
//
//     for (INT iItem = -1;;)
//     {
//         iItem = ListView_GetNextItem(m_ListView, iItem, LVNI_ALL);
//         if (iItem == -1)
//             break;
//
//         LV_RefreshIcon(iItem);
//     }
// }
// INT __stdcall CDefView::fill_list(LPVOID ptr, LPVOID arg)
// {
//     PITEMID_CHILD pidl = static_cast<PITEMID_CHILD>(ptr);
//     CDefView *pThis = static_cast<CDefView *>(arg);
//
//     // in a commdlg this works as a filemask
//     if (pThis->IncludeObject(pidl) == S_OK && pThis->m_ListView)
//         pThis->LV_AddItem(pidl);
//
//     SHFree(pidl);
//     return TRUE;
// }
// HRESULT CDefView::FillList(BOOL IsRefreshCommand)
// {
//     CComPtr<IEnumIDList> pEnumIDList;
//     PITEMID_CHILD pidl;
//     DWORD         dwFetched;
//     HRESULT       hRes;
//     HDPA          hdpa;
//     DWORD         dFlags = SHCONTF_NONFOLDERS | ((m_FolderSettings.fFlags & FWF_NOSUBFOLDERS) ? 0 : SHCONTF_FOLDERS);
//
//     TRACE("%p\n", this);
//
//     SHELLSTATE shellstate;
//     SHGetSetSettings(&shellstate, SSF_SHOWALLOBJECTS | SSF_SHOWSUPERHIDDEN, FALSE);
//     if (GetCommDlgViewFlags() & CDB2GVF_SHOWALLFILES)
//         shellstate.fShowAllObjects = shellstate.fShowSuperHidden = TRUE;
//
//     if (shellstate.fShowAllObjects)
//     {
//         dFlags |= SHCONTF_INCLUDEHIDDEN;
//         m_ListView.SendMessageW(LVM_SETCALLBACKMASK, LVIS_CUT, 0);
//     }
//     if (shellstate.fShowSuperHidden)
//     {
//         dFlags |= SHCONTF_INCLUDESUPERHIDDEN;
//         m_ListView.SendMessageW(LVM_SETCALLBACKMASK, LVIS_CUT, 0);
//     }
//
//     // get the itemlist from the shfolder
//     hRes = m_pSFParent->EnumObjects(m_hWnd, dFlags, &pEnumIDList);
//     if (hRes != S_OK)
//     {
//         if (hRes == S_FALSE)
//             return(NOERROR);
//         return(hRes);
//     }
//
//     // create a pointer array
//     hdpa = DPA_Create(16);
//     if (!hdpa)
//         return(E_OUTOFMEMORY);
//
//     // copy the items into the array
//     while((S_OK == pEnumIDList->Next(1, &pidl, &dwFetched)) && dwFetched)
//     {
//         if (DPA_AppendPtr(hdpa, pidl) == -1)
//         {
//             SHFree(pidl);
//         }
//     }
//
//     // turn listview's redrawing off
//     m_ListView.SetRedraw(FALSE);
//
//     DPA_DestroyCallback( hdpa, fill_list, this);
//
//     /* sort the array */
//     int sortCol = -1;
//     if (!IsRefreshCommand && !m_sortInfo.bLoadedFromViewState) // Are we loading for the first time?
//     {
//         m_sortInfo.Direction = 0;
//         sortCol = 0; // In case the folder does not know/care
//         if (m_pSF2Parent)
//         {
//             ULONG folderSortCol = sortCol, dummy;
//             HRESULT hr = m_pSF2Parent->GetDefaultColumn(NULL, &folderSortCol, &dummy);
//             if (SUCCEEDED(hr))
//                 hr = MapFolderColumnToListColumn(folderSortCol);
//             if (SUCCEEDED(hr))
//                 sortCol = (int) hr;
//         }
//     }
//     _Sort(sortCol);
//
//     if (m_viewinfo_data.hbmBack)
//     {
//         ::DeleteObject(m_viewinfo_data.hbmBack);
//         m_viewinfo_data.hbmBack = NULL;
//     }
//
//     // load custom background image and custom text color
//     m_viewinfo_data.cbSize = sizeof(m_viewinfo_data);
//     _DoFolderViewCB(SFVM_GET_CUSTOMVIEWINFO, 0, (LPARAM)&m_viewinfo_data);
//
//     // turn listview's redrawing back on and force it to draw
//     m_ListView.SetRedraw(TRUE);
//
//     UpdateListColors();
//
//     if (!(m_FolderSettings.fFlags & FWF_DESKTOP))
//     {
//         // redraw now
//         m_ListView.InvalidateRect(NULL, TRUE);
//     }
//
//     if (IsRefreshCommand)
//         UpdateStatusbarLocation();
//
//     _DoFolderViewCB(SFVM_LISTREFRESHED, 0, 0);
//
//     return S_OK;
// }
// LRESULT CDefView::OnShowWindow(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     if (m_ListView.IsWindow())
//         m_ListView.UpdateWindow();
//     bHandled = FALSE;
//     return 0;
// }
// LRESULT CDefView::OnGetDlgCode(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     return m_ListView.SendMessageW(uMsg, 0, 0);
// }
// LRESULT CDefView::OnDestroy(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     if (!m_Destroyed)
//     {
//         m_Destroyed = TRUE;
//         RevokeDragDrop(m_hWnd);
//         SHChangeNotifyDeregister(m_hNotify);
//         m_hNotify = NULL;
//         m_ClipboardChain.Unhook(m_hWnd);
//         if (m_pFileMenu)
//         {
//             IUnknown_SetSite(m_pFileMenu, NULL);
//             m_pFileMenu = NULL;
//         }
//         if (m_hMenu)
//         {
//             DestroyMenu(m_hMenu);
//             m_hMenu = NULL;
//         }
//         SHFree(m_pidlParent);
//         m_pidlParent = NULL;
//     }
//     bHandled = FALSE;
//     return 0;
// }
// LRESULT CDefView::OnEraseBackground(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     /* redirect to parent */
//     if (m_FolderSettings.fFlags & (FWF_DESKTOP | FWF_TRANSPARENT))
//         return SendMessageW(GetParent((*this).m_hWnd), WM_ERASEBKGND, wParam, lParam);
//
//     bHandled = FALSE;
//     return 0;
// }
// static VOID
// DrawTileBitmap(HDC hDC, LPCRECT prc, HBITMAP hbm, INT nWidth, INT nHeight, INT dx, INT dy)
// {
//     INT x0 = prc->left, y0 = prc->top, x1 = prc->right, y1 = prc->bottom;
//     x0 += dx;
//     y0 += dy;
//
//     HDC hMemDC = CreateCompatibleDC(hDC);
//     HGDIOBJ hbmOld = SelectObject(hMemDC, hbm);
//
//     for (INT y = y0; y < y1; y += nHeight)
//     {
//         for (INT x = x0; x < x1; x += nWidth)
//         {
//             BitBlt(hDC, x, y, nWidth, nHeight, hMemDC, 0, 0, SRCCOPY);
//         }
//     }
//
//     SelectObject(hMemDC, hbmOld);
//     DeleteDC(hMemDC);
// }
// LRESULT CDefView::OnPrintClient(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     HDC hDC = (HDC)wParam;
//
//     RECT rc;
//     ::GetClientRect(m_ListView, &rc);
//
//     if (m_viewinfo_data.hbmBack)
//     {
//         BITMAP bm;
//         if (::GetObject(m_viewinfo_data.hbmBack, sizeof(BITMAP), &bm))
//         {
//             INT dx = -(::GetScrollPos(m_ListView, SB_HORZ) % bm.bmWidth);
//             INT dy = -(::GetScrollPos(m_ListView, SB_VERT) % bm.bmHeight);
//             DrawTileBitmap(hDC, &rc, m_viewinfo_data.hbmBack, bm.bmWidth, bm.bmHeight, dx, dy);
//         }
//     }
//     else
//     {
//         SHFillRectClr(hDC, &rc, GetViewColor(m_viewinfo_data.clrTextBack, COLOR_WINDOW));
//     }
//
//     bHandled = TRUE;
//
//     return TRUE;
// }
// LRESULT CDefView::OnSysColorChange(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     /* Update desktop labels color */
//     UpdateListColors();
//
//     /* Forward WM_SYSCOLORCHANGE to common controls */
//     return m_ListView.SendMessageW(uMsg, 0, 0);
// }
// LRESULT CDefView::OnGetShellBrowser(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     return reinterpret_cast<LRESULT>(m_pShellBrowser.p);
// }
// LRESULT CDefView::OnNCCreate(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     this->AddRef();
//     bHandled = FALSE;
//     return 0;
// }
// VOID CDefView::OnFinalMessage(HWND)
// {
//     this->Release();
// }
// LRESULT CDefView::OnCreate(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     CComPtr<IDropTarget>     pdt;
//     CComPtr<IPersistFolder2> ppf2;
//
//     TRACE("%p\n", this);
//
//     if (SUCCEEDED(QueryInterface(IID_PPV_ARG(IDropTarget, &pdt))))
//     {
//         if (FAILED(RegisterDragDrop(m_hWnd, pdt)))
//             ERR("Error Registering DragDrop\n");
//     }
//
//     /* register for receiving notifications */
//     m_pSFParent->QueryInterface(IID_PPV_ARG(IPersistFolder2, &ppf2));
//     if (ppf2)
//     {
//         ppf2->GetCurFolder(&m_pidlParent);
//     }
//
//     if (CreateList())
//     {
//         if (InitList())
//         {
//             FillList(FALSE);
//         }
//     }
//
//     if (m_FolderSettings.fFlags & FWF_DESKTOP)
//     {
//         HWND hwndSB;
//         m_pShellBrowser->GetWindow(&hwndSB);
//         SetShellWindowEx(hwndSB, m_ListView);
//     }
//
//     // Set up change notification
//     LPITEMIDLIST pidlTarget = NULL;
//     LONG fEvents = 0;
//     HRESULT hr = _DoFolderViewCB(SFVM_GETNOTIFY, (WPARAM)&pidlTarget, (LPARAM)&fEvents);
//     if (FAILED(hr) || (!pidlTarget && !fEvents)) // FIXME: MSDN says both zero means no notifications
//     {
//         pidlTarget = m_pidlParent;
//         fEvents = SHCNE_ALLEVENTS;
//     }
//     SHChangeNotifyEntry ntreg = {};
//     hr = _DoFolderViewCB(SFVM_QUERYFSNOTIFY, 0, (LPARAM)&ntreg);
//     if (FAILED(hr))
//     {
//         ntreg.fRecursive = FALSE;
//         ntreg.pidl = pidlTarget;
//     }
//     m_hNotify = SHChangeNotifyRegister(m_hWnd,
//                                        SHCNRF_InterruptLevel | SHCNRF_ShellLevel |
//                                        SHCNRF_NewDelivery,
//                                        fEvents, SHV_CHANGE_NOTIFY,
//                                        1, &ntreg);
//
//     m_hAccel = LoadAcceleratorsW(shell32_hInstance, MAKEINTRESOURCEW(IDA_SHELLVIEW));
//
//     int bForceFullStatusBar = false;
//     BOOL bIsFileSystem = SHGetAttributes(NULL, m_pidlParent, SFGAO_FILESYSTEM) & SFGAO_FILESYSTEM;
//     m_SpecialFolder = bIsFileSystem ? -1 : 0x7f; // FS folder or "generic" CSIDL
//     if (_ILIsDesktop(m_pidlParent))
//     {
//         m_SpecialFolder = CSIDL_DESKTOP;
//     }
//     else if (IsEqualPersistClassID(ppf2, CLSID_RecycleBin))
//     {
//         m_SpecialFolder = bForceFullStatusBar = CSIDL_BITBUCKET;
//     }
//     else if (bIsFileSystem)
//     {
//         CComHeapPtr<ITEMIDLIST> pidlNet(SHCloneSpecialIDList(NULL, CSIDL_NETWORK, FALSE));
//         if (ILIsParent(pidlNet, m_pidlParent, FALSE) && ILGetSize(pidlNet) < ILGetSize(m_pidlParent))
//             m_SpecialFolder = CSIDL_NETWORK;
//     }
//     m_isFullStatusBar = bIsFileSystem || bForceFullStatusBar;
//     _ForceStatusBarResize(); // This handles changing StatusBar parts
//     UpdateStatusbar();
//     UpdateStatusbarLocation();
//
//     return S_OK;
// }
// HRESULT CDefView::FillFileMenu()
// {
//     HMENU hFileMenu = GetSubmenuByID(m_hMenu, FCIDM_MENU_FILE);
//     if (!hFileMenu)
//         return E_FAIL;
//
//     /* Cleanup the items added previously */
//     for (int i = GetMenuItemCount(hFileMenu) - 1; i >= 0; i--)
//     {
//         UINT id = GetMenuItemID(hFileMenu, i);
//         if (id < FCIDM_BROWSERFIRST || id > FCIDM_BROWSERLAST)
//             DeleteMenu(hFileMenu, i, MF_BYPOSITION);
//     }
//
//     // In case we still have this left over, clean it up
//     if (m_pFileMenu)
//     {
//         IUnknown_SetSite(m_pFileMenu, NULL);
//         m_pFileMenu.Release();
//     }
//     UINT selcount = m_ListView.GetSelectedCount();
//     // Store context menu in m_pFileMenu and keep it to invoke the selected command later on
//     HRESULT hr = GetItemObject(selcount ? SVGIO_SELECTION : SVGIO_BACKGROUND, IID_PPV_ARG(IContextMenu, &m_pFileMenu));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     HMENU hmenu = CreatePopupMenu();
//
//     UINT cmf = GetContextMenuFlags(m_pShellBrowser, GetSelectionAttributes(SFGAO_CANRENAME));
//     hr = m_pFileMenu->QueryContextMenu(hmenu, 0, DVIDM_CONTEXTMENU_FIRST, DVIDM_CONTEXTMENU_LAST, cmf);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     // TODO: filter or something
//     if (!selcount)
//     {
//         DeleteMenu(hmenu, FCIDM_SHVIEW_VIEW, MF_BYCOMMAND);
//         DeleteMenu(hmenu, FCIDM_SHVIEW_ARRANGE, MF_BYCOMMAND);
//         DeleteMenu(hmenu, FCIDM_SHVIEW_REFRESH, MF_BYCOMMAND);
//     }
//
//     Shell_MergeMenus(hFileMenu, hmenu, 0, 0, 0xFFFF, MM_ADDSEPARATOR | MM_SUBMENUSHAVEIDS);
//     ::DestroyMenu(hmenu);
//     return S_OK;
// }
// HRESULT CDefView::FillEditMenu()
// {
//     HMENU hEditMenu = GetSubmenuByID(m_hMenu, FCIDM_MENU_EDIT);
//     if (!hEditMenu)
//         return E_FAIL;
//
//     HMENU hmenuContents = ::LoadMenuW(shell32_hInstance, L"MENU_003");
//     if (!hmenuContents)
//         return E_FAIL;
//
//     Shell_MergeMenus(hEditMenu, hmenuContents, 0, 0, 0xFFFF, 0);
//
//     ::DestroyMenu(hmenuContents);
//
//     return S_OK;
// }
// HRESULT CDefView::FillViewMenu()
// {
//     HMENU hViewMenu = GetSubmenuByID(m_hMenu, FCIDM_MENU_VIEW);
//     if (!hViewMenu)
//         return E_FAIL;
//
//     m_hMenuViewModes = ::LoadMenuW(shell32_hInstance, L"MENU_001");
//     if (!m_hMenuViewModes)
//         return E_FAIL;
//
//     UINT i = SHMenuIndexFromID(hViewMenu, FCIDM_MENU_VIEW_SEP_OPTIONS);
//     Shell_MergeMenus(hViewMenu, m_hMenuViewModes, i, 0, 0xFFFF, MM_ADDSEPARATOR | MM_DONTREMOVESEPS | MM_SUBMENUSHAVEIDS);
//
//     return S_OK;
// }
// HRESULT CDefView::FillArrangeAsMenu(HMENU hmenuArrange)
// {
//     bool forceMerge = false;
//     UINT currentSortId = DVIDM_ARRANGESORT_FIRST + m_sortInfo.ListColumn;
//
//     // Make sure the column we currently sort by is in the menu
//     RemoveMenu(m_hMenuArrangeModes, DVIDM_ARRANGESORT_LAST, MF_BYCOMMAND);
//     if (m_sortInfo.ListColumn >= DEFVIEW_ARRANGESORT_MAXENUM)
//     {
//         C_ASSERT(DEFVIEW_ARRANGESORT_MAXENUM < DEFVIEW_ARRANGESORT_MAX);
//         C_ASSERT(DVIDM_ARRANGESORT_FIRST + DEFVIEW_ARRANGESORT_MAXENUM == DVIDM_ARRANGESORT_LAST);
//         WCHAR buf[MAX_PATH];
//         LVCOLUMN lvc;
//         lvc.mask = LVCF_TEXT;
//         lvc.pszText = buf;
//         lvc.cchTextMax = _countof(buf);
//         currentSortId = DVIDM_ARRANGESORT_LAST;
//         forceMerge = true;
//         ListView_GetColumn(m_ListView.m_hWnd, m_sortInfo.ListColumn, &lvc);
//         AppendMenuItem(m_hMenuArrangeModes, MF_STRING, currentSortId, lvc.pszText, m_sortInfo.ListColumn);
//     }
//
//     // Prepend the sort-by items unless they are aleady there
//     if (GetMenuItemID(hmenuArrange, 0) == FCIDM_SHVIEW_AUTOARRANGE || forceMerge)
//     {
//         Shell_MergeMenus(hmenuArrange, m_hMenuArrangeModes, 0, 0, 0xFFFF, MM_ADDSEPARATOR);
//     }
//
//     CheckMenuRadioItem(hmenuArrange,
//                        DVIDM_ARRANGESORT_FIRST, DVIDM_ARRANGESORT_LAST,
//                        currentSortId, MF_BYCOMMAND);
//
//     if (m_FolderSettings.ViewMode == FVM_DETAILS || m_FolderSettings.ViewMode == FVM_LIST)
//     {
//         EnableMenuItem(hmenuArrange, FCIDM_SHVIEW_AUTOARRANGE, MF_BYCOMMAND | MF_GRAYED);
//         EnableMenuItem(hmenuArrange, FCIDM_SHVIEW_ALIGNTOGRID, MF_BYCOMMAND | MF_GRAYED);
//     }
//     else
//     {
//         EnableMenuItem(hmenuArrange, FCIDM_SHVIEW_AUTOARRANGE, MF_BYCOMMAND);
//         EnableMenuItem(hmenuArrange, FCIDM_SHVIEW_ALIGNTOGRID, MF_BYCOMMAND);
//
//         if (GetAutoArrange() == S_OK)
//             CheckMenuItem(hmenuArrange, FCIDM_SHVIEW_AUTOARRANGE, MF_CHECKED);
//         else
//             CheckMenuItem(hmenuArrange, FCIDM_SHVIEW_AUTOARRANGE, MF_UNCHECKED);
//
//         if (_GetSnapToGrid() == S_OK)
//             CheckMenuItem(hmenuArrange, FCIDM_SHVIEW_ALIGNTOGRID, MF_CHECKED);
//         else
//             CheckMenuItem(hmenuArrange, FCIDM_SHVIEW_ALIGNTOGRID, MF_UNCHECKED);
//     }
//
//     return S_OK;
// }
// HRESULT CDefView::CheckViewMode(HMENU hmenuView)
// {
//     if (m_FolderSettings.ViewMode >= FVM_FIRST && m_FolderSettings.ViewMode <= FVM_LAST)
//     {
//         UINT iItemFirst = FCIDM_SHVIEW_BIGICON;
//         UINT iItemLast = iItemFirst + FVM_LAST - FVM_FIRST;
//         UINT iItem = iItemFirst + m_FolderSettings.ViewMode - FVM_FIRST;
//         CheckMenuRadioItem(hmenuView, iItemFirst, iItemLast, iItem, MF_BYCOMMAND);
//     }
//
//     return S_OK;
// }
// LRESULT CDefView::DoColumnContextMenu(LPARAM lParam)
// {
//     const UINT maxItems = 15; // Feels about right
//     const UINT idMore = 0x1337;
//     UINT idFirst = idMore + 1, idLast = idFirst;
//     UINT lastValidListCol = 0; // Keep track of where the new column should be inserted
//     UINT showMore = GetKeyState(VK_SHIFT) < 0;
//     POINT pt = { GET_X_LPARAM(lParam), GET_Y_LPARAM(lParam) };
//     HWND hWndHdr = ListView_GetHeader(m_ListView.m_hWnd);
//
//     if (lParam == ~0)
//     {
//         RECT r;
//         ::GetWindowRect(hWndHdr, &r);
//         pt.x = r.left + ((r.right - r.left) / 2);
//         pt.y = r.top + ((r.bottom - r.top) / 2);
//     }
//
//     HMENU hMenu = CreatePopupMenu();
//     if (!hMenu)
//         return 0;
//
//     for (UINT foldCol = 0;; ++foldCol)
//     {
//         WCHAR buf[MAX_PATH];
//         SHELLDETAILS sd;
//         sd.str_.uType = !STRRET_WSTR;
//         if (FAILED(GetDetailsByFolderColumn(NULL, foldCol, sd)))
//             break;
//         if (FAILED(StrRetToStrNW(buf, _countof(buf), &sd.str, NULL)))
//             break;
//
//         SHCOLSTATEF state = 0;
//         if (!m_pSF2Parent || FAILED(m_pSF2Parent->GetDefaultColumnState(foldCol, &state)))
//             state = 0;
//         showMore |= (state & (SHCOLSTATE_SECONDARYUI));
//
//         UINT mf = MF_STRING;
//         HRESULT listCol = MapFolderColumnToListColumn(foldCol);
//
//         if (foldCol == 0)
//             mf |= MF_CHECKED | MF_GRAYED | MF_DISABLED; // Force column 0
//         else if (state & (SHCOLSTATE_SECONDARYUI | SHCOLSTATE_HIDDEN))
//             continue;
//         else if (SUCCEEDED(listCol))
//             mf |= MF_CHECKED;
//
//         if (AppendMenuItem(hMenu, mf, idLast, buf, lastValidListCol + 1))
//         {
//             idLast++;
//             if (SUCCEEDED(listCol))
//                 lastValidListCol = listCol;
//         }
//
//         if (idLast - idFirst == maxItems)
//         {
//             showMore++;
//             break;
//         }
//     }
//
//     if (showMore)
//     {
//     }
//
//     // A cludge to force the cursor to update so we are not stuck with "size left/right" if
//     // the right-click was on a column divider.
//     ::PostMessage(m_ListView.m_hWnd, WM_SETCURSOR, (WPARAM) m_ListView.m_hWnd, HTCLIENT);
//
//     // Note: Uses the header as the owner so CDefView::OnInitMenuPopup does not mess us up.
//     UINT idCmd = TrackPopupMenu(hMenu, TPM_RETURNCMD | TPM_LEFTBUTTON | TPM_RIGHTBUTTON,
//                                 pt.x, pt.y, 0, hWndHdr, NULL);
//     if (idCmd == idMore)
//     {
//         FIXME("Open More dialog\n");
//     }
//     else if (idCmd)
//     {
//         UINT foldCol = idCmd - idFirst;
//         HRESULT listCol = MapFolderColumnToListColumn(foldCol);
//         if (SUCCEEDED(listCol))
//         {
//             ListView_DeleteColumn(m_ListView.m_hWnd, listCol);
//         }
//         else
//         {
//             listCol = (UINT) GetMenuItemDataById(hMenu, idCmd);
//             LoadColumn(foldCol, listCol, TRUE);
//         }
//         ColumnListChanged();
//     }
//     DestroyMenu(hMenu);
//     return 0;
// }
// UINT CDefView::GetSelections()
// {
//     UINT count = m_ListView.GetSelectedCount();
//     if (count > m_cidl || !count || !m_apidl) // !count to free possibly large cache, !m_apidl to make sure m_apidl is a valid pointer
//     {
//         SHFree(m_apidl);
//         m_apidl = static_cast<PCUITEMID_CHILD*>(SHAlloc(count * sizeof(PCUITEMID_CHILD)));
//         if (!m_apidl)
//         {
//             m_cidl = 0;
//             return 0;
//         }
//     }
//     m_cidl = count;
//
//     TRACE("-- Items selected =%u\n", m_cidl);
//
//     ASSERT(m_ListView);
//
//     UINT i = 0;
//     int lvIndex = -1;
//     while ((lvIndex = m_ListView.GetNextItem(lvIndex, LVNI_SELECTED)) > -1)
//     {
//         m_apidl[i] = _PidlByItem(lvIndex);
//         i++;
//         if (i == m_cidl)
//              break;
//         TRACE("-- selected Item found\n");
//     }
//
//     return m_cidl;
// }
// SFGAOF CDefView::GetSelectionAttributes(SFGAOF Query)
// {
//     if (!GetSelections())
//         return 0;
//     SFGAOF Attr = Query;
//     return SUCCEEDED(m_pSFParent->GetAttributesOf(m_cidl, m_apidl, &Attr)) ? (Attr & Query) : 0;
// }
// HRESULT CDefView::InvokeContextMenuCommand(CComPtr<IContextMenu>& pCM, LPCSTR lpVerb, POINT* pt, bool TryMapVerb)
// {
//     CMINVOKECOMMANDINFOEX cmi;
//
//     ZeroMemory(&cmi, sizeof(cmi));
//     cmi.cbSize = sizeof(cmi);
//     cmi.hwnd = m_hWnd;
//     cmi.lpVerb = lpVerb;
//     cmi.nShow = SW_SHOW;
//
//     WCHAR szverbW[sizeof("properties")];
//     static const WORD verbmap[] = { FCIDM_SHVIEW_DELETE, FCIDM_SHVIEW_RENAME,
//                                     FCIDM_SHVIEW_PROPERTIES, FCIDM_SHVIEW_CREATELINK,
//                                     FCIDM_SHVIEW_CUT, FCIDM_SHVIEW_COPY, FCIDM_SHVIEW_INSERT };
//     for (SIZE_T i = 0; TryMapVerb && i < _countof(verbmap); ++i)
//     {
//         if (cmi.lpVerb != MAKEINTRESOURCEA(verbmap[i]))
//             continue;
//         if (PCSTR pszverbA = MapFcidmCmdToVerb((UINT_PTR)cmi.lpVerb))
//         {
//             // Map our internal commands to canonical verbs so non-shell32 menus can understand us
//             SHAnsiToUnicode(pszverbA, szverbW, _countof(szverbW));
//             cmi.lpVerb = pszverbA;
//             cmi.lpVerbW = szverbW;
//             break;
//         }
//     }
//
//     if (GetKeyState(VK_SHIFT) < 0)
//         cmi.fMask |= CMIC_MASK_SHIFT_DOWN;
//
//     if (GetKeyState(VK_CONTROL) < 0)
//         cmi.fMask |= CMIC_MASK_CONTROL_DOWN;
//
//     if (pt)
//     {
//         cmi.fMask |= CMIC_MASK_PTINVOKE;
//         cmi.ptInvoke = *pt;
//     }
//
//     WCHAR szDirW[MAX_PATH] = L"";
//     CHAR szDirA[MAX_PATH];
//     if (SUCCEEDED(_DoFolderViewCB(SFVM_GETCOMMANDDIR, _countof(szDirW), (LPARAM)szDirW)) &&
//         *szDirW != UNICODE_NULL)
//     {
//         SHUnicodeToAnsi(szDirW, szDirA, _countof(szDirA));
//         cmi.fMask |= CMIC_MASK_UNICODE;
//         cmi.lpDirectory = szDirA;
//         cmi.lpDirectoryW = szDirW;
//     }
//
//     HRESULT hr = pCM->InvokeCommand((LPCMINVOKECOMMANDINFO)&cmi);
//     // Most of our callers will do this, but if they would forget (File menu!)
//     IUnknown_SetSite(pCM, NULL);
//     pCM.Release();
//
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     return S_OK;
// }
// HRESULT CDefView::OpenSelectedItems(PCSTR pszVerb)
// {
//     HMENU hMenu;
//     UINT uCommand;
//     HRESULT hResult;
//
//     if (m_ListView.GetSelectedCount() == 0)
//         return S_OK;
//
//     hResult = OnDefaultCommand();
//     if (hResult == S_OK)
//         return hResult;
//
//     hMenu = CreatePopupMenu();
//     if (!hMenu)
//         return E_FAIL;
//
//     CComPtr<IContextMenu> pCM;
//     hResult = GetItemObject(SVGIO_SELECTION, IID_PPV_ARG(IContextMenu, &pCM));
//     MenuCleanup _(pCM, hMenu);
//     if (FAILED_UNEXPECTEDLY(hResult))
//         return hResult;
//
//     UINT cmf = CMF_DEFAULTONLY | GetContextMenuFlags(m_pShellBrowser, 0);
//     hResult = pCM->QueryContextMenu(hMenu, 0, DVIDM_CONTEXTMENU_FIRST, DVIDM_CONTEXTMENU_LAST, cmf);
//     if (FAILED_UNEXPECTEDLY(hResult))
//         return hResult;
//
//     uCommand = GetMenuDefaultItem(hMenu, FALSE, 0);
//     if (uCommand == (UINT)-1 && !pszVerb)
//     {
//         ERR("GetMenuDefaultItem returned -1\n");
//         return E_FAIL;
//     }
//     if (!pszVerb)
//         pszVerb = MAKEINTRESOURCEA(uCommand);
//
//     InvokeContextMenuCommand(pCM, pszVerb, NULL);
//
//     return hResult;
// }
// LRESULT CDefView::OnContextMenu(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     POINT pt = { pt.x = GET_X_LPARAM(lParam), pt.y = GET_Y_LPARAM(lParam) };
//     UINT    uCommand;
//     HRESULT hResult;
//
//     TRACE("(%p)\n", this);
//
//     if (m_hContextMenu != NULL)
//     {
//         ERR("HACK: Aborting context menu in nested call\n");
//         return 0;
//     }
//
//     HWND hWndHdr = ListView_GetHeader(m_ListView.m_hWnd);
//     RECT r;
//     if (::GetWindowRect(hWndHdr, &r) && PtInRect(&r, pt) && ::IsWindowVisible(hWndHdr))
//     {
//         return DoColumnContextMenu(lParam);
//     }
//
//     m_hContextMenu = CreatePopupMenu();
//     if (!m_hContextMenu)
//         return E_FAIL;
//
//     if (lParam != ~0) // unless app key (menu key) was pressed
//     {
//         LV_HITTESTINFO hittest = { pt };
//         ScreenToClient(&hittest.pt);
//         m_ListView.HitTest(&hittest);
//
//         // Right-Clicked item is selected? If selected, no selection change.
//         // If not selected, then reset the selection and select the item.
//         if ((hittest.flags & LVHT_ONITEM) &&
//             m_ListView.GetItemState(hittest.iItem, LVIS_SELECTED) != LVIS_SELECTED)
//         {
//             SelectItem(hittest.iItem, SVSI_SELECT | SVSI_DESELECTOTHERS | SVSI_ENSUREVISIBLE);
//         }
//     }
//
//     UINT count = m_ListView.GetSelectedCount();
//     // In case we still have this left over, clean it up
//     hResult = GetItemObject(count ? SVGIO_SELECTION : SVGIO_BACKGROUND, IID_PPV_ARG(IContextMenu, &m_pCM));
//     MenuCleanup _(m_pCM, m_hContextMenu);
//     if (FAILED_UNEXPECTEDLY(hResult))
//         return 0;
//
//     UINT cmf = GetContextMenuFlags(m_pShellBrowser, GetSelectionAttributes(SFGAO_CANRENAME));
//     // Use 1 as the first id we want. 0 means that user canceled the menu
//     hResult = m_pCM->QueryContextMenu(m_hContextMenu, 0, CONTEXT_MENU_BASE_ID, DVIDM_CONTEXTMENU_LAST, cmf);
//     if (FAILED_UNEXPECTEDLY(hResult))
//         return 0;
//
//     if (m_pCommDlgBrowser && !(GetCommDlgViewFlags() & CDB2GVF_NOSELECTVERB))
//     {
//         HMENU hMenuSource = LoadMenuW(crate::win32::shell32_hInstance, MAKEINTRESOURCEW(IDM_DVSELECT));
//         Shell_MergeMenus(m_hContextMenu, GetSubMenu(hMenuSource, 0), 0, DVIDM_COMMDLG_SELECT, 0xffff, MM_ADDSEPARATOR | MM_DONTREMOVESEPS);
//         DestroyMenu(hMenuSource);
//         SetMenuDefaultItem(m_hContextMenu, DVIDM_COMMDLG_SELECT, MF_BYCOMMAND);
//         // TODO: ICommDlgBrowser2::GetDefaultMenuText == S_OK
//     }
//
//     // There is no position requested, so try to find one
//     if (lParam == ~0)
//     {
//         HWND hFocus = ::GetFocus();
//         int lvIndex = -1;
//
//         if (hFocus == m_ListView.m_hWnd || m_ListView.IsChild(hFocus))
//         {
//             // Is there an item focused and selected?
//             lvIndex = m_ListView.GetNextItem(-1, LVNI_SELECTED | LVNI_FOCUSED);
//             // If not, find the first selected item
//             if (lvIndex < 0)
//                 lvIndex = m_ListView.GetNextItem(-1, LVNI_SELECTED);
//         }
//
//         // We got something
//         if (lvIndex > -1)
//         {
//             // Find the center of the icon
//             RECT rc = { LVIR_ICON };
//             m_ListView.SendMessage(LVM_GETITEMRECT, lvIndex, (LPARAM)&rc);
//             pt.x = (rc.right + rc.left) / 2;
//             pt.y = (rc.bottom + rc.top) / 2;
//         }
//         else
//         {
//             // We have to drop it somewhere
//             pt.x = pt.y = 0;
//         }
//
//         m_ListView.ClientToScreen(&pt);
//     }
//
//     CComPtr<ICommDlgBrowser2> pcdb2;
//     if (m_pCommDlgBrowser && SUCCEEDED(m_pCommDlgBrowser->QueryInterface(IID_PPV_ARG(ICommDlgBrowser2, &pcdb2))))
//         pcdb2->Notify(static_cast<IShellView*>(this), CDB2N_CONTEXTMENU_START);
//
//     // This runs the message loop, calling back to us with f.e. WM_INITPOPUP (hence why m_hContextMenu and m_pCM exist)
//     uCommand = TrackPopupMenu(m_hContextMenu,
//                               TPM_LEFTALIGN | TPM_RETURNCMD | TPM_LEFTBUTTON | TPM_RIGHTBUTTON,
//                               pt.x, pt.y, 0, m_hWnd, NULL);
//     if (uCommand >= DVIDM_ARRANGESORT_FIRST && uCommand <= DVIDM_ARRANGESORT_LAST)
//     {
//         SendMessage(WM_COMMAND, uCommand, 0);
//     }
//     else if (uCommand != 0 && !(uCommand == DVIDM_COMMDLG_SELECT && OnDefaultCommand() == S_OK))
//     {
//         InvokeContextMenuCommand(m_pCM, MAKEINTRESOURCEA(uCommand - CONTEXT_MENU_BASE_ID), &pt);
//     }
//
//     if (pcdb2)
//         pcdb2->Notify(static_cast<IShellView*>(this), CDB2N_CONTEXTMENU_DONE);
//     return 0;
// }
// LRESULT CDefView::OnExplorerCommand(UINT uCommand, BOOL bUseSelection)
// {
//     HRESULT hResult;
//     HMENU hMenu = NULL;
//
//     CComPtr<IContextMenu> pCM;
//     hResult = GetItemObject(bUseSelection ? SVGIO_SELECTION : SVGIO_BACKGROUND, IID_PPV_ARG(IContextMenu, &pCM));
//     if (FAILED_UNEXPECTEDLY(hResult))
//         return 0;
//
//     MenuCleanup _(pCM, hMenu);
//
//     if ((uCommand != FCIDM_SHVIEW_DELETE) && (uCommand != FCIDM_SHVIEW_RENAME))
//     {
//         hMenu = CreatePopupMenu();
//         if (!hMenu)
//             return 0;
//
//         hResult = pCM->QueryContextMenu(hMenu, 0, DVIDM_CONTEXTMENU_FIRST, DVIDM_CONTEXTMENU_LAST, CMF_NORMAL);
//         if (FAILED_UNEXPECTEDLY(hResult))
//             return 0;
//     }
//
//     if (bUseSelection)
//     {
//         // FIXME: we should cache this
//         SFGAOF rfg = SFGAO_BROWSABLE | SFGAO_CANCOPY | SFGAO_CANLINK | SFGAO_CANMOVE | SFGAO_CANDELETE | SFGAO_CANRENAME | SFGAO_HASPROPSHEET | SFGAO_FILESYSTEM | SFGAO_FOLDER;
//         hResult = m_pSFParent->GetAttributesOf(m_cidl, m_apidl, &rfg);
//         if (FAILED_UNEXPECTEDLY(hResult))
//             return 0;
//
//         if (!(rfg & SFGAO_CANMOVE) && uCommand == FCIDM_SHVIEW_CUT)
//             return 0;
//         if (!(rfg & SFGAO_CANCOPY) && uCommand == FCIDM_SHVIEW_COPY)
//             return 0;
//         if (!(rfg & SFGAO_CANDELETE) && uCommand == FCIDM_SHVIEW_DELETE)
//             return 0;
//         if (!(rfg & SFGAO_CANRENAME) && uCommand == FCIDM_SHVIEW_RENAME)
//             return 0;
//         if (!(rfg & SFGAO_HASPROPSHEET) && uCommand == FCIDM_SHVIEW_PROPERTIES)
//             return 0;
//     }
//
//     // FIXME: We should probably use the objects position?
//     InvokeContextMenuCommand(pCM, MAKEINTRESOURCEA(uCommand), NULL, true);
//     return 0;
// }
// LRESULT CDefView::OnSize(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     WORD wWidth, wHeight;
//
//     wWidth  = LOWORD(lParam);
//     wHeight = HIWORD(lParam as DWORD);
//
//     TRACE("%p width=%u height=%u\n", this, wWidth, wHeight);
//
//     // WM_SIZE can come before WM_CREATE
//     if (!m_ListView)
//         return 0;
//
//     /* Resize the ListView to fit our window */
//     ::MoveWindow(m_ListView, 0, 0, wWidth, wHeight, TRUE);
//
//     _DoFolderViewCB(SFVM_SIZE, 0, 0);
//
//     _ForceStatusBarResize();
//     UpdateStatusbar();
//
//     return 0;
// }
// void CDefView::OnDeactivate()
// {
//     TRACE("%p\n", this);
//
//     if (m_uState != SVUIA_DEACTIVATE)
//     {
//         // TODO: cleanup menu after deactivation
//         m_uState = SVUIA_DEACTIVATE;
//     }
// }
// void CDefView::DoActivate(UINT uState)
// {
//     TRACE("%p uState=%x\n", this, uState);
//
//     // don't do anything if the state isn't really changing
//     if (m_uState == uState)
//     {
//         return;
//     }
//
//     if (uState == SVUIA_DEACTIVATE)
//     {
//         OnDeactivate();
//     }
//     else
//     {
//         if(m_hMenu && !m_bmenuBarInitialized)
//         {
//             FillEditMenu();
//             FillViewMenu();
//             m_pShellBrowser->SetMenuSB(m_hMenu, 0, m_hWnd);
//             m_bmenuBarInitialized = TRUE;
//         }
//
//         if (SVUIA_ACTIVATE_FOCUS == uState)
//         {
//             m_ListView.SetFocus();
//         }
//     }
//
//     m_uState = uState;
//     TRACE("--\n");
// }
// void CDefView::_DoCopyToMoveToFolder(BOOL bCopy)
// {
//     if (!GetSelections())
//         return;
//
//     SFGAOF rfg = SFGAO_CANCOPY | SFGAO_CANMOVE | SFGAO_FILESYSTEM;
//     HRESULT hr = m_pSFParent->GetAttributesOf(m_cidl, m_apidl, &rfg);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return;
//
//     if (!bCopy && !(rfg & SFGAO_CANMOVE))
//         return;
//     if (bCopy && !(rfg & SFGAO_CANCOPY))
//         return;
//
//     CComPtr<IContextMenu> pCM;
//     hr = m_pSFParent->GetUIObjectOf(m_hWnd, m_cidl, m_apidl, IID_IContextMenu, 0, (void **)&pCM);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return;
//
//     InvokeContextMenuCommand(pCM, (bCopy ? "copyto" : "moveto"), NULL);
// }
// LRESULT CDefView::OnActivate(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     DoActivate(SVUIA_ACTIVATE_FOCUS);
//     return 0;
// }
// LRESULT CDefView::OnSetFocus(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     TRACE("%p\n", this);
//
//     /* Tell the browser one of our windows has received the focus. This
//     should always be done before merging menus (OnActivate merges the
//     menus) if one of our windows has the focus.*/
//
//     m_pShellBrowser->OnViewWindowActive(this);
//     DoActivate(SVUIA_ACTIVATE_FOCUS);
//
//     /* Set the focus to the listview */
//     m_ListView.SetFocus();
//
//     /* Notify the ICommDlgBrowser interface */
//     OnStateChange(CDBOSC_SETFOCUS);
//
//     return 0;
// }
// LRESULT CDefView::OnKillFocus(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     TRACE("(%p) stub\n", this);
//
//     DoActivate(SVUIA_ACTIVATE_NOFOCUS);
//     /* Notify the ICommDlgBrowser */
//     OnStateChange(CDBOSC_KILLFOCUS);
//
//     return 0;
// }
// LRESULT CDefView::OnCommand(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     DWORD dwCmdID;
//     DWORD dwCmd;
//     HWND  hwndCmd;
//     int   nCount;
//
//     dwCmdID = GET_WM_COMMAND_ID(wParam, lParam);
//     dwCmd = GET_WM_COMMAND_CMD(wParam, lParam);
//     hwndCmd = GET_WM_COMMAND_HWND(wParam, lParam);
//
//     TRACE("(%p)->(0x%08x 0x%08x %p) stub\n", this, dwCmdID, dwCmd, hwndCmd);
//
//     if (dwCmdID >= DVIDM_ARRANGESORT_FIRST && dwCmdID <= DVIDM_ARRANGESORT_LAST)
//     {
//         UINT listCol = (UINT)GetMenuItemDataById(m_hMenuArrangeModes, dwCmdID);
//         _Sort(listCol);
//         return 0;
//     }
//
//     switch (dwCmdID)
//     {
//         case FCIDM_SHVIEW_BIGICON:    C_ASSERT(FCIDM_SHVIEW_BIGICON == 0x7029);
//         case FCIDM_SHVIEW_SMALLICON:  C_ASSERT(FCIDM_SHVIEW_SMALLICON == 0x702A);
//         case FCIDM_SHVIEW_LISTVIEW:   C_ASSERT(FCIDM_SHVIEW_LISTVIEW == 0x702B);
//         case FCIDM_SHVIEW_REPORTVIEW: C_ASSERT(FCIDM_SHVIEW_REPORTVIEW == 0x702C);
//         case 0x702D:
//         case FCIDM_SHVIEW_TILEVIEW:   C_ASSERT(FCIDM_SHVIEW_TILEVIEW == 0x702E);
//         case 0x702F:
//             SetCurrentViewMode(dwCmdID - FCIDM_SHVIEW_BIGICON + 1);
//             break;
//         case FCIDM_SHVIEW_SNAPTOGRID:
//             m_ListView.Arrange(LVA_SNAPTOGRID);
//             break;
//         case FCIDM_SHVIEW_ALIGNTOGRID:
//             if (_GetSnapToGrid() == S_OK)
//                 m_ListView.SetExtendedListViewStyle(0, LVS_EX_SNAPTOGRID);
//             else
//                 ArrangeGrid();
//             break;
//         case FCIDM_SHVIEW_AUTOARRANGE:
//             if (GetAutoArrange() == S_OK)
//                 m_ListView.ModifyStyle(LVS_AUTOARRANGE, 0);
//             else
//                 AutoArrange();
//             break;
//         case FCIDM_SHVIEW_SELECTALL:
//             if (_DoFolderViewCB(SFVM_CANSELECTALL, 0, 0) != S_FALSE)
//                 m_ListView.SetItemState(-1, LVIS_SELECTED, LVIS_SELECTED);
//             break;
//         case FCIDM_SHVIEW_INVERTSELECTION:
//             nCount = m_ListView.GetItemCount();
//             for (int i=0; i < nCount; i++)
//                 m_ListView.SetItemState(i, m_ListView.GetItemState(i, LVIS_SELECTED) ^ LVIS_SELECTED, LVIS_SELECTED);
//             break;
//         case FCIDM_SHVIEW_DESELECTALL:
//             m_ListView.SetItemState(-1, 0, LVIS_SELECTED);
//             break;
//         case FCIDM_SHVIEW_REFRESH:
//             Refresh();
//             break;
//         case FCIDM_SHVIEW_DELETE:
//         case FCIDM_SHVIEW_CUT:
//         case FCIDM_SHVIEW_COPY:
//         case FCIDM_SHVIEW_RENAME:
//         case FCIDM_SHVIEW_PROPERTIES:
//             if (SHRestricted(REST_NOVIEWCONTEXTMENU))
//                 return 0;
//             return OnExplorerCommand(dwCmdID, TRUE);
//         case FCIDM_SHVIEW_COPYTO: C_ASSERT(FCIDM_SHVIEW_COPYTO == 0x701e);
//         case FCIDM_SHVIEW_MOVETO: C_ASSERT(FCIDM_SHVIEW_MOVETO == 0x701f);
//             _DoCopyToMoveToFolder(dwCmdID == FCIDM_SHVIEW_COPYTO);
//             return 0;
//         case FCIDM_SHVIEW_INSERT:
//         case FCIDM_SHVIEW_UNDO:
//         case FCIDM_SHVIEW_INSERTLINK:
//         case FCIDM_SHVIEW_NEWFOLDER:
//             return OnExplorerCommand(dwCmdID, FALSE);
//         default:
//             // WM_COMMAND messages from file menu are routed to CDefView to let m_pFileMenu handle them
//             if (m_pFileMenu && dwCmd == 0)
//             {
//                 HMENU Dummy = NULL;
//                 MenuCleanup _(m_pFileMenu, Dummy);
//                 InvokeContextMenuCommand(m_pFileMenu, MAKEINTRESOURCEA(dwCmdID), NULL);
//             }
//     }
//
//     return 0;
// }
// static BOOL
// SelectExtOnRename(void)
// {
//     HKEY hKey;
//     LONG error;
//     DWORD dwValue = FALSE, cbValue;
//
//     error = RegOpenKeyExW(HKEY_CURRENT_USER, REGSTR_PATH_EXPLORER, 0, KEY_READ, &hKey);
//     if (error)
//         return dwValue;
//
//     cbValue = sizeof(dwValue);
//     RegQueryValueExW(hKey, L"SelectExtOnRename", NULL, NULL, (LPBYTE)&dwValue, &cbValue);
//
//     RegCloseKey(hKey);
//     return !!dwValue;
// }
// LRESULT CDefView::OnNotify(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     UINT             CtlID;
//     LPNMHDR          lpnmh;
//     LPNMLISTVIEW     lpnmlv;
//     NMLVDISPINFOW    *lpdi;
//     PCUITEMID_CHILD  pidl;
//     BOOL             unused;
//
//     CtlID = wParam;
//     lpnmh = (LPNMHDR)lParam;
//     lpnmlv = (LPNMLISTVIEW)lpnmh;
//     lpdi = (NMLVDISPINFOW *)lpnmh;
//
//     TRACE("%p CtlID=%u lpnmh->code=%x\n", this, CtlID, lpnmh->code);
//
//     switch (lpnmh->code)
//     {
//         case NM_SETFOCUS:
//             TRACE("-- NM_SETFOCUS %p\n", this);
//             OnSetFocus(0, 0, 0, unused);
//             break;
//         case NM_KILLFOCUS:
//             TRACE("-- NM_KILLFOCUS %p\n", this);
//             OnDeactivate();
//             /* Notify the ICommDlgBrowser interface */
//             OnStateChange(CDBOSC_KILLFOCUS);
//             break;
//         case NM_CUSTOMDRAW:
//             TRACE("-- NM_CUSTOMDRAW %p\n", this);
//             return CDRF_DODEFAULT as LRESULT;
//         case NM_RELEASEDCAPTURE:
//             TRACE("-- NM_RELEASEDCAPTURE %p\n", this);
//             break;
//         case NM_CLICK:
//             TRACE("-- NM_CLICK %p\n", this);
//             break;
//         case NM_RCLICK:
//             TRACE("-- NM_RCLICK %p\n", this);
//             break;
//         case NM_DBLCLK:
//             TRACE("-- NM_DBLCLK %p\n", this);
//             break;
//         case NM_RETURN:
//             TRACE("-- NM_RETURN %p\n", this);
//             break;
//         case HDN_ENDTRACKW:
//             TRACE("-- HDN_ENDTRACKW %p\n", this);
//             //nColumn1 = m_ListView.GetColumnWidth(0);
//             //nColumn2 = m_ListView.GetColumnWidth(1);
//             break;
//         case LVN_DELETEITEM:
//             TRACE("-- LVN_DELETEITEM %p\n", this);
//             /*delete the pidl because we made a copy of it*/
//             SHFree(reinterpret_cast<LPVOID>(lpnmlv->lParam));
//             break;
//         case LVN_DELETEALLITEMS:
//             TRACE("-- LVN_DELETEALLITEMS %p\n", this);
//             return FALSE;
//         case LVN_INSERTITEM:
//             TRACE("-- LVN_INSERTITEM (STUB)%p\n", this);
//             break;
//         case LVN_ITEMACTIVATE:
//             TRACE("-- LVN_ITEMACTIVATE %p\n", this);
//             OpenSelectedItems(((NMITEMACTIVATE *)lpnmh)->uKeyFlags & LVKF_ALT ? "properties" : NULL);
//             break;
//         case LVN_COLUMNCLICK:
//         {
//             UINT foldercol = MapListColumnToFolderColumn(lpnmlv->iSubItem);
//             HRESULT hr = S_FALSE;
//             if (m_pSDParent)
//                 hr = m_pSDParent->ColumnClick(foldercol);
//             if (hr != S_OK)
//                 hr = _DoFolderViewCB(SFVM_COLUMNCLICK, foldercol, 0);
//             if (hr != S_OK)
//                 _Sort(lpnmlv->iSubItem);
//             break;
//         }
//         case LVN_GETDISPINFOA:
//         case LVN_GETDISPINFOW:
//             TRACE("-- LVN_GETDISPINFO %p\n", this);
//             pidl = _PidlByItem(lpdi->item);
//
//             if (lpdi->item.mask & LVIF_TEXT)    /* text requested */
//             {
//                 SHELLDETAILS sd;
//                 if (FAILED_UNEXPECTEDLY(GetDetailsByListColumn(pidl, lpdi->item.iSubItem, sd)))
//                     break;
//
//                 if (lpnmh->code == LVN_GETDISPINFOA)
//                 {
//                     /* shouldn't happen */
//                     NMLVDISPINFOA *lpdiA = (NMLVDISPINFOA *)lpnmh;
//                     StrRetToStrNA( lpdiA->item.pszText, lpdiA->item.cchTextMax, &sd.str, NULL);
//                     TRACE("-- text=%s\n", lpdiA->item.pszText);
//                 }
//                 else /* LVN_GETDISPINFOW */
//                 {
//                     StrRetToStrNW( lpdi->item.pszText, lpdi->item.cchTextMax, &sd.str, NULL);
//                     TRACE("-- text=%s\n", debugstr_w(lpdi->item.pszText));
//                 }
//             }
//             if (lpdi->item.mask & LVIF_IMAGE)
//             {
//                 lpdi->item.iImage = SHMapPIDLToSystemImageListIndex(m_pSFParent, pidl, 0);
//             }
//             if (lpdi->item.mask & LVIF_STATE)
//             {
//                 if ((lpdi->item.stateMask & LVIS_CUT) && GetItemAttributes(pidl, SFGAO_HIDDEN | SFGAO_GHOSTED))
//                     lpdi->item.state |= LVIS_CUT;
//             }
//             lpdi->item.mask |= LVIF_DI_SETITEM;
//             break;
//         case LVN_ITEMCHANGED:
//             TRACE("-- LVN_ITEMCHANGED %p\n", this);
//             if ((lpnmlv->uOldState ^ lpnmlv->uNewState) & (LVIS_SELECTED | LVIS_FOCUSED))
//             {
//                 OnStateChange(CDBOSC_SELCHANGE); // browser will get the IDataObject
//                 // FIXME: Use LVIS_DROPHILITED instead in drag_notify_subitem
//                 if (!m_ScheduledStatusbarUpdate && (m_iDragOverItem == -1 || m_pCurDropTarget == NULL))
//                 {
//                     m_ScheduledStatusbarUpdate = true;
//                     PostMessageW((*this).m_hWnd, SHV_UPDATESTATUSBAR, 0, 0);
//                 }
//                 _DoFolderViewCB(SFVM_SELECTIONCHANGED, NULL/* FIXME */, NULL/* FIXME */);
//             }
//             break;
//         case LVN_BEGINDRAG:
//         case LVN_BEGINRDRAG:
//             TRACE("-- LVN_BEGINDRAG\n");
//             if (GetSelections())
//             {
//                 CComPtr<IDataObject> pda;
//                 DWORD dwAttributes = SFGAO_CANCOPY | SFGAO_CANLINK;
//                 DWORD dwEffect = DROPEFFECT_MOVE;
//
//                 if (SUCCEEDED(m_pSFParent->GetUIObjectOf(m_hWnd, m_cidl, m_apidl, IID_NULL_PPV_ARG(IDataObject, &pda))))
//                 {
//                     LPNMLISTVIEW params = (LPNMLISTVIEW)lParam;
//
//                     if (SUCCEEDED(m_pSFParent->GetAttributesOf(m_cidl, m_apidl, &dwAttributes)))
//                         dwEffect |= dwAttributes & (SFGAO_CANCOPY | SFGAO_CANLINK);
//
//                     CComPtr<IAsyncOperation> piaso;
//                     if (SUCCEEDED(pda->QueryInterface(IID_PPV_ARG(IAsyncOperation, &piaso))))
//                         piaso->SetAsyncMode(TRUE);
//
//                     DWORD dwEffect2;
//
//                     m_pSourceDataObject = pda;
//                     m_ptFirstMousePos = params->ptAction;
//                     ClientToScreen(&m_ptFirstMousePos);
//                     ::ClientToListView(m_ListView, &m_ptFirstMousePos);
//
//                     HIMAGELIST big_icons, small_icons;
//                     Shell_GetImageLists(&big_icons, &small_icons);
//                     PCUITEMID_CHILD pidl = _PidlByItem(params->iItem);
//                     int iIcon = SHMapPIDLToSystemImageListIndex(m_pSFParent, pidl, 0);
//                     POINT ptItem;
//                     m_ListView.GetItemPosition(params->iItem, &ptItem);
//
//                     ImageList_BeginDrag(big_icons, iIcon, params->ptAction.x - ptItem.x, params->ptAction.y - ptItem.y);
//                     DoDragDrop(pda, this, dwEffect, &dwEffect2);
//                     m_pSourceDataObject.Release();
//                 }
//             }
//             break;
//         case LVN_BEGINLABELEDITW:
//         {
//             TRACE("-- LVN_BEGINLABELEDITW %p\n", this);
//             HWND hEdit = ListView_GetEditControl(m_ListView);
//             pidl = _PidlByItem(lpdi->item);
//             DWORD fAttr = pidl ? GetItemAttributes(pidl, SFGAO_CANRENAME | SFGAO_FOLDER | SFGAO_FILESYSTEM) : 0;
//             if (!hEdit || !(fAttr & SFGAO_CANRENAME))
//             {
//                 MessageBeep(0xffffffff);
//                 return TRUE;
//             }
//
//             WCHAR szName[MAX_PATH], *pszText = lpdi->item.pszText;
//             if (SUCCEEDED(DisplayNameOfW(m_pSFParent, pidl, SHGDN_FOREDITING | SHGDN_INFOLDER,
//                                          szName, _countof(szName))))
//             {
//                 pszText = szName;
//                 ::SetWindowText(hEdit, pszText);
//             }
//
//             // smartass-renaming: See CORE-15242
//             if (!(fAttr & SFGAO_FOLDER) && (fAttr & SFGAO_FILESYSTEM) &&
//                 (lpdi->item.mask & LVIF_TEXT) && !SelectExtOnRename())
//             {
//                 CComHeapPtr<ITEMIDLIST_RELATIVE> pidlFull(ILCombine(m_pidlParent, pidl));
//                 WCHAR szFullPath[MAX_PATH];
//                 if (SHGetPathFromIDListW(pidlFull, szFullPath) && !SHELL_FS_HideExtension(szFullPath))
//                 {
//                     LPWSTR pchDotExt = PathFindExtensionW(pszText);
//                     ::PostMessageW(hEdit, EM_SETSEL, 0, pchDotExt - pszText);
//                     ::PostMessageW(hEdit, EM_SCROLLCARET, 0, 0);
//                 }
//             }
//
//             INT cchLimit = 0;
//             _DoFolderViewCB(SFVM_GETNAMELENGTH, (WPARAM)pidl, (LPARAM)&cchLimit);
//             if (cchLimit)
//                 ::SendMessageW(hEdit, EM_SETLIMITTEXT, cchLimit as WPARAM, 0);
//             SHLimitInputEdit(hEdit, m_pSFParent);
//             m_isEditing = TRUE;
//             return FALSE;
//         }
//         case LVN_ENDLABELEDITW:
//         {
//             TRACE("-- LVN_ENDLABELEDITW %p\n", this);
//             m_isEditing = FALSE;
//             if (!lpdi->item.pszText)
//                 return TRUE;
//
//             pidl = _PidlByItem(lpdi->item);
//             // We have to copy the old PIDL because SetNameOf might generate a SHCNE_UPDATEITEM
//             // and that notification can cause us to call LV_UpdateItem and free the old PIDL too soon.
//             CComHeapPtr<ITEMIDLIST_ABSOLUTE> pidlOld(ILClone(pidl));
//             if (!pidlOld)
//             {
//                 SHELL_ErrorBox(lpdi->hdr.hwndFrom, E_OUTOFMEMORY);
//                 return FALSE;
//             }
//             PITEMID_CHILD pidlNew = NULL;
//             HRESULT hr = m_pSFParent->SetNameOf(0, pidlOld, lpdi->item.pszText, SHGDN_INFOLDER, &pidlNew);
//             if (SUCCEEDED(hr) && pidlNew)
//             {
//                 int iNew = LV_FindItemByPidl(pidlNew);
//                 if (iNew != lpdi->item.iItem && iNew >= 0)
//                     ILFree(pidlNew);// A SHCNE has updated the item already
//                 else if (!LV_UpdateItem(lpdi->item.iItem, pidlNew))
//                     ILFree(pidlNew);
//                 OnStateChange(CDBOSC_RENAME);
//             }
//             else
//             {
//                 ::PostMessageW(m_ListView, LVM_EDITLABEL, lpdi->item.iItem, 0); // Renaming failed, let the user try again
//             }
//             return FALSE;
//         }
//         default:
//             TRACE("-- %p WM_COMMAND %x unhandled\n", this, lpnmh->code);
//             break;
//     }
//
//     return 0;
// }
// static BOOL ILIsParentOrSpecialParent(PCIDLIST_ABSOLUTE pidl1, PCIDLIST_ABSOLUTE pidl2)
// {
//     if (!pidl1 || !pidl2)
//         return FALSE;
//     if (ILIsParent(pidl1, pidl2, TRUE))
//         return TRUE;
//
//     CComHeapPtr<ITEMIDLIST_ABSOLUTE> pidl2Clone(ILClone(pidl2));
//     ILRemoveLastID(pidl2Clone);
//     return ILIsEqual(pidl1, pidl2Clone);
// }
// LRESULT CDefView::OnChangeNotify(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     // The change notify can come before WM_CREATE
//     if (!m_ListView)
//         return FALSE;
//
//     HANDLE hChange = (HANDLE)wParam;
//     DWORD dwProcID = (DWORD)lParam;
//     PIDLIST_ABSOLUTE *Pidls;
//     LONG lEvent;
//     HANDLE hLock = SHChangeNotification_Lock(hChange, dwProcID, &Pidls, &lEvent);
//     if (hLock == NULL)
//     {
//         ERR("hLock == NULL\n");
//         return FALSE;
//     }
//     lEvent &= ~SHCNE_INTERRUPT;
//     TRACE("(%p)(%p,%p,%p) %#x\n", this, Pidls[0], Pidls[1], lParam, lEvent);
//
//     if (_DoFolderViewCB(SFVM_FSNOTIFY, (WPARAM)Pidls, lEvent) == S_FALSE)
//     {
//         SHChangeNotification_Unlock(hLock);
//         return FALSE;
//     }
//
//     // Translate child IDLs.
//     // SHSimpleIDListFromPathW creates fake PIDLs (lacking some attributes)
//     HRESULT hr;
//     PITEMID_CHILD child0 = NULL, child1 = NULL;
//     CComHeapPtr<ITEMIDLIST_RELATIVE> pidl0Temp, pidl1Temp;
//     if (lEvent != SHCNE_UPDATEIMAGE && lEvent < SHCNE_EXTENDED_EVENT)
//     {
//         if (_ILIsSpecialFolder(Pidls[0]) || ILIsParentOrSpecialParent(m_pidlParent, Pidls[0]))
//         {
//             child0 = ILFindLastID(Pidls[0]);
//             hr = SHGetRealIDL(m_pSFParent, child0, &pidl0Temp);
//             if (SUCCEEDED(hr))
//                 child0 = pidl0Temp;
//         }
//         if (_ILIsSpecialFolder(Pidls[1]) || ILIsParentOrSpecialParent(m_pidlParent, Pidls[1]))
//         {
//             child1 = ILFindLastID(Pidls[1]);
//             hr = SHGetRealIDL(m_pSFParent, child1, &pidl1Temp);
//             if (SUCCEEDED(hr))
//                 child1 = pidl1Temp;
//         }
//     }
//
//     switch (lEvent)
//     {
//         case SHCNE_MKDIR:
//         case SHCNE_CREATE:
//         case SHCNE_DRIVEADD:
//             if (!child0)
//                 break;
//             if (LV_FindItemByPidl(child0) < 0)
//                 LV_AddItem(child0);
//             else
//                 LV_UpdateItem(child0);
//             break;
//         case SHCNE_RMDIR:
//         case SHCNE_DELETE:
//         case SHCNE_DRIVEREMOVED:
//             if (child0)
//                 LV_DeleteItem(child0);
//             break;
//         case SHCNE_RENAMEFOLDER:
//         case SHCNE_RENAMEITEM:
//             if (child0 && child1)
//                 LV_RenameItem(child0, child1);
//             else if (child0)
//                 LV_DeleteItem(child0);
//             else if (child1)
//                 LV_AddItem(child1);
//             break;
//         case SHCNE_UPDATEITEM:
//             if (child0)
//                 LV_UpdateItem(child0);
//             break;
//         case SHCNE_UPDATEIMAGE:
//         case SHCNE_MEDIAINSERTED:
//         case SHCNE_MEDIAREMOVED:
//         case SHCNE_ASSOCCHANGED:
//             LV_RefreshIcons();
//             break;
//         case SHCNE_UPDATEDIR:
//         case SHCNE_ATTRIBUTES:
//             if (child0)
//                 LV_UpdateItem(child0);
//             else
//                 Refresh();
//             UpdateStatusbar();
//             break;
//         case SHCNE_FREESPACE:
//             UpdateStatusbar();
//             break;
//     }
//
//     SHChangeNotification_Unlock(hLock);
//     return TRUE;
// }
// LRESULT CDefView::OnMenuMessage(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     if (!m_pCM)
//     {
//         /* no menu */
//         ERR("no context menu\n");
//         return FALSE;
//     }
//     LRESULT result = 0;
//     HRESULT hres = SHForwardContextMenuMsg(m_pCM, uMsg, wParam, lParam, &result, TRUE);
//     return SUCCEEDED(hres);
// }
// LRESULT CDefView::OnSettingChange(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     /* Wallpaper setting affects drop shadows effect */
//     if (wParam == SPI_SETDESKWALLPAPER || wParam == 0)
//         UpdateListColors();
//
//     UINT ListExMask = LVS_EX_TRACKSELECT | LVS_EX_ONECLICKACTIVATE;
//     UINT ListExBits = GetItemActivateFlags();
//     if (wParam == SPI_GETICONTITLELOGFONT ||
//         (lParam && !lstrcmpiW((PWSTR)lParam, REGSTR_PATH_EXPLORER L"\\IconUnderline")))
//     {
//         ListExMask |= LVS_EX_UNDERLINEHOT | LVS_EX_UNDERLINECOLD;
//         ListExBits |= SHELL_GetIconUnderlineFlags();
//     }
//     m_ListView.SetExtendedListViewStyle(ListExBits, ListExMask);
//     m_ListView.SendMessage(uMsg, wParam, lParam);
//     return S_OK;
// }
// LRESULT CDefView::OnInitMenuPopup(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     HMENU hmenu = (HMENU) wParam;
//     int nPos = LOWORD(lParam);
//     UINT menuItemId;
//
//     if (m_isEditing)
//         ListView_CancelEditLabel(m_ListView);
//     if (m_pCM)
//         OnMenuMessage(uMsg, wParam, lParam, bHandled);
//
//     HMENU hViewMenu = GetSubmenuByID(m_hMenu, FCIDM_MENU_VIEW);
//
//     if (GetSelections() == 0)
//     {
//         ::EnableMenuItem(hmenu, FCIDM_SHVIEW_CUT, MF_GRAYED);
//         ::EnableMenuItem(hmenu, FCIDM_SHVIEW_COPY, MF_GRAYED);
//         ::EnableMenuItem(hmenu, FCIDM_SHVIEW_RENAME, MF_GRAYED);
//         ::EnableMenuItem(hmenu, FCIDM_SHVIEW_COPYTO, MF_GRAYED);
//         ::EnableMenuItem(hmenu, FCIDM_SHVIEW_MOVETO, MF_GRAYED);
//         ::EnableMenuItem(hmenu, FCIDM_SHVIEW_DELETE, MF_GRAYED);
//     }
//     else
//     {
//         // FIXME: Check copyable
//         ::EnableMenuItem(hmenu, FCIDM_SHVIEW_CUT, MF_ENABLED);
//         ::EnableMenuItem(hmenu, FCIDM_SHVIEW_COPY, MF_ENABLED);
//         ::EnableMenuItem(hmenu, FCIDM_SHVIEW_RENAME, MF_ENABLED);
//         ::EnableMenuItem(hmenu, FCIDM_SHVIEW_COPYTO, MF_ENABLED);
//         ::EnableMenuItem(hmenu, FCIDM_SHVIEW_MOVETO, MF_ENABLED);
//         ::EnableMenuItem(hmenu, FCIDM_SHVIEW_DELETE, MF_ENABLED);
//     }
//
//     /* Lets try to find out what the hell wParam is */
//     if (hmenu == GetSubMenu(m_hMenu, nPos))
//         menuItemId = ReallyGetMenuItemID(m_hMenu, nPos);
//     else if (hViewMenu && hmenu == GetSubMenu(hViewMenu, nPos))
//         menuItemId = ReallyGetMenuItemID(hViewMenu, nPos);
//     else if (m_hContextMenu && hmenu == GetSubMenu(m_hContextMenu, nPos))
//         menuItemId = ReallyGetMenuItemID(m_hContextMenu, nPos);
//     else
//         return FALSE;
//
//     switch (menuItemId)
//     {
//     case FCIDM_MENU_FILE:
//         FillFileMenu();
//         break;
//     case FCIDM_MENU_VIEW:
//     case FCIDM_SHVIEW_VIEW:
//         CheckViewMode(hmenu);
//         break;
//     case FCIDM_SHVIEW_ARRANGE:
//         FillArrangeAsMenu(hmenu);
//         break;
//     }
//
//     return FALSE;
// }
// LRESULT CDefView::OnChangeCBChain(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     bHandled = TRUE;
//     return m_ClipboardChain.HandleChangeCBChain(wParam, lParam);
// }
// LRESULT CDefView::OnDrawClipboard(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL &bHandled)
// {
//     bHandled = TRUE;
//     const LRESULT res = m_ClipboardChain.HandleDrawClipboard(wParam, lParam);
//     if (m_HasCutItems)
//     {
//         m_ClipboardChain.Unhook(m_hWnd);
//         m_HasCutItems = false;
//         m_ListView.SetItemState(-1, 0, LVIS_CUT); // Somebody copied or pasted, nothing can be "cut" anymore
//         m_ListView.SendMessageW(LVM_SETCALLBACKMASK, m_ListView.SendMessageW(LVM_GETCALLBACKMASK, 0, 0) | LVIS_CUT, 0);
//     }
//     return res;
// }
// void CDefView::RefreshGhostedState()
// {
//     for (UINT i = 0, c = m_ListView.GetItemCount(); i < c; ++i)
//         m_ListView.SetItemState(i, GetItemAttributes(i, SFGAO_HIDDEN | SFGAO_GHOSTED) ? LVIS_CUT : 0, LVIS_CUT);
// }
// HRESULT WINAPI CDefView::GetWindow(HWND *phWnd)
// {
//     TRACE("(%p)\n", this);
//
//     *phWnd = m_hWnd;
//
//     return S_OK;
// }
// HRESULT WINAPI CDefView::ContextSensitiveHelp(BOOL fEnterMode)
// {
//     FIXME("(%p) stub\n", this);
//
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CDefView::TranslateAccelerator(LPMSG lpmsg)
// {
//     if (m_isEditing)
//         return S_FALSE;
//
//     if (lpmsg->message >= WM_KEYFIRST && lpmsg->message <= WM_KEYLAST)
//     {
//         if (::TranslateAcceleratorW(m_hWnd, m_hAccel, lpmsg) != 0)
//             return S_OK;
//
//         TRACE("-- key=0x%04lx\n", lpmsg->wParam);
//     }
//
//     return m_pShellBrowser ? m_pShellBrowser->TranslateAcceleratorSB(lpmsg, 0) : S_FALSE;
// }
// HRESULT WINAPI CDefView::EnableModeless(BOOL fEnable)
// {
//     FIXME("(%p)\n", this);
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CDefView::UIActivate(UINT uState)
// {
//     TRACE("(%p)->(state=%x)\n", this, uState);
//
//     // don't do anything if the state isn't changing
//     if (m_uState == uState)
//         return S_OK;
//
//     // OnActivate handles the menu merging and internal state
//     DoActivate(uState);
//
//     // only do this if we are active
//     if (uState != SVUIA_DEACTIVATE)
//     {
//         _ForceStatusBarResize();
//
//         // Set the text for the status bar
//         UpdateStatusbar();
//     }
//
//     return S_OK;
// }
// HRESULT WINAPI CDefView::Refresh()
// {
//     TRACE("(%p)\n", this);
//
//     _DoFolderViewCB(SFVM_LISTREFRESHED, TRUE, 0);
//
//     m_ListView.DeleteAllItems();
//     FillList();
//
//     return S_OK;
// }
// HRESULT WINAPI CDefView::CreateViewWindow(IShellView *lpPrevView, LPCFOLDERSETTINGS lpfs, IShellBrowser *psb, RECT *prcView, HWND *phWnd)
// {
//     return CreateViewWindow3(psb, lpPrevView, SV3CVW3_DEFAULT,
//         (FOLDERFLAGS)lpfs->fFlags, (FOLDERFLAGS)lpfs->fFlags, (FOLDERVIEWMODE)lpfs->ViewMode, NULL, prcView, phWnd);
// }
// HRESULT WINAPI CDefView::DestroyViewWindow()
// {
//     TRACE("(%p)\n", this);
//
//     /* Make absolutely sure all our UI is cleaned up */
//     UIActivate(SVUIA_DEACTIVATE);
//
//     if (m_hAccel)
//     {
//         // MSDN: Accelerator tables loaded from resources are freed automatically when application terminates
//         m_hAccel = NULL;
//     }
//
//     if (m_hMenuArrangeModes)
//     {
//         DestroyMenu(m_hMenuArrangeModes);
//         m_hMenuArrangeModes = NULL;
//     }
//
//     if (m_hMenuViewModes)
//     {
//         DestroyMenu(m_hMenuViewModes);
//         m_hMenuViewModes = NULL;
//     }
//
//     if (m_hMenu)
//     {
//         DestroyMenu(m_hMenu);
//         m_hMenu = NULL;
//     }
//
//     if (m_ListView)
//     {
//         m_ListView.DestroyWindow();
//     }
//
//     if (m_hWnd)
//     {
//         _DoFolderViewCB(SFVM_WINDOWCLOSING, (WPARAM)m_hWnd, 0);
//         DestroyWindow();
//     }
//
//     m_pShellBrowser.Release();
//     m_pCommDlgBrowser.Release();
//
//     return S_OK;
// }
// HRESULT WINAPI CDefView::GetCurrentInfo(LPFOLDERSETTINGS lpfs)
// {
//     TRACE("(%p)->(%p) vmode=%x flags=%x\n", this, lpfs,
//           m_FolderSettings.ViewMode, m_FolderSettings.fFlags);
//
//     if (!lpfs)
//         return E_INVALIDARG;
//
//     *lpfs = m_FolderSettings;
//     return S_OK;
// }
// HRESULT WINAPI CDefView::AddPropertySheetPages(DWORD dwReserved, LPFNADDPROPSHEETPAGE lpfn, LPARAM lparam)
// {
//     TRACE("(%p)->(0x%lX, %p, %p)\n", this, dwReserved, lpfn, lparam);
//
//     SFVM_PROPPAGE_DATA Data = { dwReserved, lpfn, lparam };
//     _DoFolderViewCB(SFVM_ADDPROPERTYPAGES, 0, (LPARAM)&Data);
//     return S_OK;
// }
// static HRESULT Read(IStream *pS, LPVOID buffer, ULONG cb)
// {
//     ULONG read;
//     HRESULT hr = pS->Read(buffer, cb, &read);
//     return FAILED(hr) ? hr : (cb == read ? S_OK : HResultFromWin32(ERROR_MORE_DATA));
// }
// static DWORD ReadDWORD(IPropertyBag *pPB, LPCWSTR name, DWORD def)
// {
//     DWORD value;
//     HRESULT hr = SHPropertyBag_ReadDWORD(pPB, name, &value);
//     return SUCCEEDED(hr) ? value : def;
// }
// HRESULT CDefView::GetDefaultViewStream(DWORD Stgm, IStream **ppStream)
// {
//     CLSID clsid;
//     HRESULT hr = IUnknown_GetClassID(m_pSFParent, &clsid);
//     if (SUCCEEDED(hr))
//     {
//         WCHAR path[MAX_PATH], name[39];
//         wsprintfW(path, L"%s\\%s", REGSTR_PATH_EXPLORER, L"Streams\\Default");
//         StringFromGUID2(clsid, name, 39);
//         *ppStream = SHOpenRegStream2W(HKEY_CURRENT_USER, path, name, Stgm);
//         hr = *ppStream ? S_OK : E_FAIL;
//     }
//     return hr;
// }
// static HRESULT LoadColumnsStream(PERSISTCOLUMNS &cols, IStream *pS)
// {
//     HRESULT hr = Read(pS, &cols, FIELD_OFFSET(PERSISTCOLUMNS, Columns));
//     if (FAILED(hr))
//         return hr;
//     if (cols.Signature != PERSISTCOLUMNS::SIG || cols.Count > PERSISTCOLUMNS::MAXCOUNT)
//         return HResultFromWin32(ERROR_INVALID_DATA);
//     return Read(pS, &cols.Columns, sizeof(*cols.Columns) * cols.Count);
// }
// HRESULT CDefView::LoadViewState()
// {
//     PERSISTCLASSICVIEWSTATE cvs;
//     PERSISTCOLUMNS cols;
//     CComPtr<IStream> pS;
//     CComPtr<IPropertyBag> pPB;
//     bool fallback = false;
//     HRESULT hrColumns = E_FAIL;
//     HRESULT hr = IUnknown_QueryServicePropertyBag(m_pShellBrowser, SHGVSPB_FOLDER, IID_PPV_ARG(IPropertyBag, &pPB));
//     if (SUCCEEDED(hr))
//     {
//         DWORD data;
//         if (FAILED(hr = SHPropertyBag_ReadDWORD(pPB, L"Mode", &data)))
//             goto loadfallback;
//         cvs.FolderSettings.ViewMode = data;
//         cvs.FolderSettings.fFlags = ReadDWORD(pPB, L"FFlags", FWF_NOGROUPING);
//         data = ReadDWORD(pPB, L"Sort", ~0ul);
//         cvs.SortColId = data != ~0ul ? (WORD)data : LISTVIEW_SORT_INFO::UNSPECIFIEDCOLUMN;
//         cvs.SortDir = (INT8)ReadDWORD(pPB, L"SortDir", 1);
//         if (SUCCEEDED(hrColumns = SHPropertyBag_ReadStream(pPB, L"ColInfo", &pS)))
//             hrColumns = LoadColumnsStream(cols, pS);
//     }
//     else
//     {
//         if (FAILED(hr = (m_pShellBrowser ? m_pShellBrowser->GetViewStateStream(STGM_READ, &pS) : E_UNEXPECTED)))
//         {
//         loadfallback:
//             hr = GetDefaultViewStream(STGM_READ, &pS);
//             fallback = true;
//         }
//         if (FAILED(hr) || FAILED(hr = Read(pS, &cvs, sizeof(cvs))))
//             return hr;
//         if (cvs.Signature != cvs.SIG)
//             return HResultFromWin32(ERROR_INVALID_DATA);
//         hrColumns = LoadColumnsStream(cols, pS);
//     }
//     m_FolderSettings.ViewMode = cvs.FolderSettings.ViewMode;
//     m_FolderSettings.fFlags &= ~cvs.VALIDFWF;
//     m_FolderSettings.fFlags |= cvs.FolderSettings.fFlags & cvs.VALIDFWF;
//     if (SUCCEEDED(hrColumns))
//     {
//         BOOL failed = FALSE;
//         if ((m_LoadColumnsList = DPA_Create(cols.Count)) != NULL)
//         {
//             for (UINT i = 0; i < cols.Count; ++i)
//             {
//                 failed |= !DPA_SetPtr(m_LoadColumnsList, i, UlongToPtr(cols.Columns[i]));
//             }
//         }
//         if (failed || !cols.Count)
//         {
//             DPA_Destroy(m_LoadColumnsList);
//             m_LoadColumnsList = NULL;
//         }
//     }
//     m_sortInfo.bLoadedFromViewState = !fallback && m_LoadColumnsList && (int)cvs.SortColId != LISTVIEW_SORT_INFO::UNSPECIFIEDCOLUMN;
//     m_sortInfo.bColumnIsFolderColumn = TRUE;
//     m_sortInfo.Direction = cvs.SortDir > 0 ? 1 : -1;
//     m_sortInfo.ListColumn = cvs.SortColId;
//     return hr;
// }
// HRESULT CDefView::SaveViewState(IStream *pStream)
// {
//     if (!m_ListView.m_hWnd)
//         return E_UNEXPECTED;
//     int sortcol = MapListColumnToFolderColumn(m_sortInfo.ListColumn);
//     PERSISTCLASSICVIEWSTATE cvs;
//     cvs.SortColId = sortcol >= 0 ? (WORD)sortcol : 0;
//     cvs.SortDir = m_sortInfo.Direction;
//     PERSISTCOLUMNS cols;
//     cols.Signature = PERSISTCOLUMNS::SIG;
//     cols.Count = 0;
//     LVCOLUMN lvc;
//     lvc.mask = LVCF_WIDTH;
//     for (UINT i = 0, j = 0; i < PERSISTCOLUMNS::MAXCOUNT && ListView_GetColumn(m_ListView, j, &lvc); ++j)
//     {
//         HRESULT hr = MapListColumnToFolderColumn(j);
//         if (SUCCEEDED(hr))
//         {
//             cols.Columns[i] = MAKELONG(hr, lvc.cx);
//             cols.Count = ++i;
//         }
//     }
//     UINT cbColumns = FIELD_OFFSET(PERSISTCOLUMNS, Columns) + (sizeof(*cols.Columns) * cols.Count);
//     UpdateFolderViewFlags();
//
//     IPropertyBag *pPB;
//     HRESULT hr = S_OK;
//     if (pStream)
//     {
//         pStream->AddRef();
//         goto stream;
//     }
//     hr = IUnknown_QueryServicePropertyBag(m_pShellBrowser, SHGVSPB_FOLDER, IID_PPV_ARG(IPropertyBag, &pPB));
//     if (SUCCEEDED(hr))
//     {
//         UINT uViewMode;
//         GetCurrentViewMode(&uViewMode);
//         hr = SHPropertyBag_WriteDWORD(pPB, L"Mode", uViewMode);
//         SHPropertyBag_WriteDWORD(pPB, L"FFlags", m_FolderSettings.fFlags);
//         SHPropertyBag_WriteDWORD(pPB, L"Sort", cvs.SortColId);
//         SHPropertyBag_WriteDWORD(pPB, L"SortDir", cvs.SortDir);
//         pStream = cols.Count ? SHCreateMemStream((LPBYTE)&cols, cbColumns) : NULL;
//         if (!pStream || FAILED(SHPropertyBag_WriteStream(pPB, L"ColInfo", pStream)))
//             SHPropertyBag_Delete(pPB, L"ColInfo");
//         pPB->Release();
//     }
//     else if (SUCCEEDED(hr = (m_pShellBrowser ? m_pShellBrowser->GetViewStateStream(STGM_WRITE, &pStream) : E_UNEXPECTED)))
//     {
//     stream:
//         ULONG written;
//         cvs.Signature = cvs.SIG;
//         cvs.FolderSettings = m_FolderSettings;
//         hr = pStream->Write(&cvs, sizeof(cvs), &written);
//         if (SUCCEEDED(hr))
//             hr = pStream->Write(&cols, cbColumns, &written);
//     }
//     // TODO: else if (SUCCEEDED(_DoFolderViewCB(SFVM_GETCOLUMNSTREAM)))
//     if (pStream)
//         pStream->Release();
//     return hr;
// }
// HRESULT WINAPI CDefView::SaveViewState()
// {
//     if (!(m_FolderSettings.fFlags & FWF_NOBROWSERVIEWSTATE))
//         return SaveViewState(NULL);
//     return S_FALSE;
// }
// void CDefView::UpdateFolderViewFlags()
// {
//     UPDATEFOLDERVIEWFLAGS(m_FolderSettings.fFlags, FWF_AUTOARRANGE, GetAutoArrange() == S_OK);
//     UPDATEFOLDERVIEWFLAGS(m_FolderSettings.fFlags, FWF_SNAPTOGRID, _GetSnapToGrid() == S_OK);
//     UPDATEFOLDERVIEWFLAGS(m_FolderSettings.fFlags, FWF_NOGROUPING, !ListView_IsGroupViewEnabled(m_ListView.m_hWnd));
// }
// UINT CDefView::GetItemActivateFlags()
// {
//     SHELLSTATE ss;
//     SHGetSetSettings(&ss, SSF_DOUBLECLICKINWEBVIEW | SSF_WIN95CLASSIC, FALSE);
//     return ((m_FolderSettings.fFlags & FWF_SINGLECLICKACTIVATE) || (ss.fDoubleClickInWebView() == 0 && ss.fWin95Classic() == 0))
//             ? (LVS_EX_TRACKSELECT | LVS_EX_ONECLICKACTIVATE) : 0;
// }
// HRESULT WINAPI CDefView::GetItemObject(UINT uItem, REFIID riid, LPVOID *ppvOut)
// {
//     HRESULT hr = E_NOINTERFACE;
//
//     TRACE("(%p)->(uItem=0x%08x,\n\tIID=%s, ppv=%p)\n", this, uItem, debugstr_guid(&riid), ppvOut);
//
//     if (!ppvOut)
//         return E_INVALIDARG;
//
//     *ppvOut = NULL;
//
//     switch (uItem)
//     {
//         case SVGIO_BACKGROUND:
//             if (IsEqualIID(riid, IID_IContextMenu))
//             {
//                 hr = CDefViewBckgrndMenu_CreateInstance(m_pSF2Parent, riid, ppvOut);
//                 if (FAILED_UNEXPECTEDLY(hr))
//                     return hr;
//
//                 IUnknown_SetSite(*((IUnknown**)ppvOut), (IShellView *)this);
//             }
//             else if (IsEqualIID(riid, IID_IDispatch))
//             {
//                 if (m_pShellFolderViewDual == NULL)
//                 {
//                     hr = CDefViewDual_Constructor(riid, (LPVOID*)&m_pShellFolderViewDual);
//                     if (FAILED_UNEXPECTEDLY(hr))
//                         return hr;
//                 }
//                 hr = m_pShellFolderViewDual->QueryInterface(riid, ppvOut);
//             }
//             break;
//         case SVGIO_SELECTION:
//             GetSelections();
//             hr = m_pSFParent->GetUIObjectOf(m_hWnd, m_cidl, m_apidl, riid, 0, ppvOut);
//             if (FAILED_UNEXPECTEDLY(hr))
//                 return hr;
//
//             if (IsEqualIID(riid, IID_IContextMenu))
//                 IUnknown_SetSite(*((IUnknown**)ppvOut), (IShellView *)this);
//
//             break;
//     }
//
//     TRACE("-- (%p)->(interface=%p)\n", this, *ppvOut);
//
//     return hr;
// }
// FOLDERVIEWMODE CDefView::GetDefaultViewMode()
// {
//     FOLDERVIEWMODE mode = ((m_FolderSettings.fFlags & FWF_DESKTOP) || !IsOS(OS_SERVERADMINUI)) ? FVM_ICON : FVM_DETAILS;
//     FOLDERVIEWMODE temp = mode;
//     if (SUCCEEDED(_DoFolderViewCB(SFVM_DEFVIEWMODE, 0, (LPARAM)&temp)) && IsSupportedFolderViewMode(temp))
//         mode = temp;
//     return mode;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetCurrentViewMode(UINT *pViewMode)
// {
//     TRACE("(%p)->(%p), stub\n", this, pViewMode);
//
//     if (!pViewMode)
//         return E_INVALIDARG;
//
//     *pViewMode = m_FolderSettings.ViewMode;
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::SetCurrentViewMode(UINT ViewMode)
// {
//     DWORD dwStyle;
//     TRACE("(%p)->(%u), stub\n", this, ViewMode);
//
//     /* It's not redundant to check FVM_AUTO because it's a (UINT)-1 */
//     if (((INT)ViewMode < FVM_FIRST || (INT)ViewMode > FVM_LAST) && ((INT)ViewMode != FVM_AUTO))
//         return E_INVALIDARG;
//
//     /* Windows before Vista uses LVM_SETVIEW and possibly
//        LVM_SETEXTENDEDLISTVIEWSTYLE to set the style of the listview,
//        while later versions seem to accomplish this through other
//        means. */
//     switch (ViewMode)
//     {
//         case FVM_ICON:      dwStyle = LVS_ICON;      break;
//         case FVM_SMALLICON: dwStyle = LVS_SMALLICON; break;
//         case FVM_LIST:      dwStyle = LVS_LIST;      break;
//         case FVM_DETAILS:   dwStyle = LVS_REPORT;    break;
//         default:
//             FIXME("ViewMode %d not implemented\n", ViewMode);
//             dwStyle = LVS_LIST;
//             break;
//     }
//
//     m_ListView.ModifyStyle(LVS_TYPEMASK, dwStyle);
//
//     /* This will not necessarily be the actual mode set above.
//        This mimics the behavior of Windows XP. */
//     m_FolderSettings.ViewMode = ViewMode;
//     CheckToolbar();
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetFolder(REFIID riid, void **ppv)
// {
//     if (m_pSFParent == NULL)
//         return E_FAIL;
//
//     return m_pSFParent->QueryInterface(riid, ppv);
// }
// HRESULT STDMETHODCALLTYPE CDefView::Item(int iItemIndex, PITEMID_CHILD *ppidl)
// {
//     PCUITEMID_CHILD pidl = _PidlByItem(iItemIndex);
//     if (pidl)
//     {
//         *ppidl = ILClone(pidl);
//         return S_OK;
//     }
//
//     *ppidl = 0;
//     return E_INVALIDARG;
// }
// HRESULT STDMETHODCALLTYPE CDefView::ItemCount(UINT uFlags, int *pcItems)
// {
//     TRACE("(%p)->(%u %p)\n", this, uFlags, pcItems);
//     if (uFlags != SVGIO_ALLVIEW && uFlags != SVGIO_SELECTION)
//         FIXME("some flags unsupported, %x\n", uFlags & ~(SVGIO_ALLVIEW | SVGIO_SELECTION));
//     if ((uFlags & SVGIO_TYPE_MASK) == SVGIO_SELECTION)
//         *pcItems = m_ListView.GetSelectedCount();
//     else
//         *pcItems = m_ListView.GetItemCount();
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::Items(UINT uFlags, REFIID riid, void **ppv)
// {
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetSelectionMarkedItem(int *piItem)
// {
//     TRACE("(%p)->(%p)\n", this, piItem);
//
//     *piItem = m_ListView.GetSelectionMark();
//
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetFocusedItem(int *piItem)
// {
//     TRACE("(%p)->(%p)\n", this, piItem);
//
//     *piItem = m_ListView.GetNextItem(-1, LVNI_FOCUSED);
//
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetItemPosition(PCUITEMID_CHILD pidl, POINT *ppt)
// {
//     if (!m_ListView)
//     {
//         ERR("!m_ListView\n");
//         return E_FAIL;
//     }
//
//     int lvIndex = LV_FindItemByPidl(pidl);
//     if (lvIndex == -1 || ppt == NULL)
//         return E_INVALIDARG;
//
//     m_ListView.GetItemPosition(lvIndex, ppt);
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetSpacing(POINT *ppt)
// {
//     TRACE("(%p)->(%p)\n", this, ppt);
//
//     if (!m_ListView)
//     {
//         ERR("!m_ListView\n");
//         return S_FALSE;
//     }
//
//     if (ppt)
//     {
//         SIZE spacing;
//         m_ListView.GetItemSpacing(spacing);
//
//         ppt->x = spacing.cx;
//         ppt->y = spacing.cy;
//     }
//
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetDefaultSpacing(POINT *ppt)
// {
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetAutoArrange()
// {
//     return ((m_ListView.GetStyle() & LVS_AUTOARRANGE) ? S_OK : S_FALSE);
// }
// HRESULT CDefView::_GetSnapToGrid()
// {
//     DWORD dwExStyle = (DWORD)m_ListView.SendMessage(LVM_GETEXTENDEDLISTVIEWSTYLE, 0, 0);
//     return ((dwExStyle & LVS_EX_SNAPTOGRID) ? S_OK : S_FALSE);
// }
// HRESULT STDMETHODCALLTYPE CDefView::SelectAndPositionItems(UINT cidl, PCUITEMID_CHILD_ARRAY apidl, POINT *apt, DWORD dwFlags)
// {
//     ASSERT(m_ListView);
//     m_ListView.SetItemState(-1, 0, LVIS_SELECTED); // Reset the selection
//     for (UINT i = 0 ; i < cidl; i++)
//     {
//         int lvIndex = LV_FindItemByPidl(apidl[i]);
//         if (lvIndex != -1)
//             SelectAndPositionItem(lvIndex, dwFlags, apt ? &apt[i] : NULL);
//     }
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetView(SHELLVIEWID *pVid, ULONG view_type)
// {
//     if (view_type == SV2GV_DEFAULTVIEW)
//     {
//         *pVid = VID_Default;
//         return S_OK;
//     }
//     if (view_type == SV2GV_CURRENTVIEW)
//         view_type = m_FolderSettings.ViewMode;
//     if ((int)view_type < 0)
//         return E_UNEXPECTED;
//
//     view_type += FVM_FIRST;
//     if (!IsSupportedFolderViewMode(view_type))
//         return E_INVALIDARG;
//     *pVid = *FolderViewModeToShellViewId(view_type);
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::CreateViewWindow2(LPSV2CVW2_PARAMS view_params)
// {
//     return CreateViewWindow3(view_params->psbOwner, view_params->psvPrev,
//         SV3CVW3_DEFAULT, (FOLDERFLAGS)view_params->pfs->fFlags, (FOLDERFLAGS)view_params->pfs->fFlags,
//         (FOLDERVIEWMODE)view_params->pfs->ViewMode, view_params->pvid, view_params->prcView, &view_params->hwndView);
// }
// HRESULT STDMETHODCALLTYPE CDefView::CreateViewWindow3(IShellBrowser *psb, IShellView *psvPrevious, SV3CVW3_FLAGS view_flags, FOLDERFLAGS mask, FOLDERFLAGS flags, FOLDERVIEWMODE mode, const SHELLVIEWID *view_id, const RECT *prcView, HWND *hwnd)
// {
//     OLEMENUGROUPWIDTHS omw = { { 0, 0, 0, 0, 0, 0 } };
//     const UINT SUPPORTED_SV3CVW3 = SV3CVW3_FORCEVIEWMODE | SV3CVW3_FORCEFOLDERFLAGS;
//     const UINT IGNORE_FWF = FWF_OWNERDATA; // FIXME: Support this
//
//     *hwnd = NULL;
//
//     TRACE("(%p)->(shlview=%p shlbrs=%p rec=%p hwnd=%p vmode=%x flags=%x)\n", this, psvPrevious, psb, prcView, hwnd, mode, flags);
//     if (prcView != NULL)
//         TRACE("-- left=%i top=%i right=%i bottom=%i\n", prcView->left, prcView->top, prcView->right, prcView->bottom);
//
//     /* Validate the Shell Browser */
//     if (psb == NULL || m_hWnd)
//         return E_UNEXPECTED;
//
//     if (view_flags & ~SUPPORTED_SV3CVW3)
//         FIXME("unsupported view flags 0x%08x\n", view_flags & ~SUPPORTED_SV3CVW3);
//
//     /* Set up the member variables */
//     m_pShellBrowser = psb;
//     m_FolderSettings.ViewMode = mode;
//     m_FolderSettings.fFlags = (mask & flags) & ~IGNORE_FWF;
//
//     if (view_id)
//     {
//         FOLDERVIEWMODE temp = (FOLDERVIEWMODE)ShellViewIdToFolderViewMode(view_id);
//         if (IsSupportedFolderViewMode(temp))
//             mode = temp;
//         else if (*view_id == VID_Default)
//             mode = FVM_AUTO;
//         else
//             FIXME("Ignoring unrecognized VID %s\n", debugstr_guid(view_id));
//     }
//     if (mode == FVM_AUTO)
//         m_FolderSettings.ViewMode = GetDefaultViewMode();
//
//     if (!IsSupportedFolderViewMode(m_FolderSettings.ViewMode))
//     {
//         ERR("Ignoring %s FVM %u\n", FolderViewModeToShellViewId(m_FolderSettings.ViewMode)
//             ? "unsupported" : "invalid", m_FolderSettings.ViewMode);
//         m_FolderSettings.ViewMode = FVM_ICON;
//     }
//     const UINT requestedViewMode = m_FolderSettings.ViewMode;
//
//     /* Get our parent window */
//     m_pShellBrowser->GetWindow(&m_hWndParent);
//     _DoFolderViewCB(SFVM_HWNDMAIN, 0, (LPARAM)m_hWndParent);
//
//     /* Try to get the ICommDlgBrowserInterface, adds a reference !!! */
//     m_pCommDlgBrowser = NULL;
//     if (SUCCEEDED(m_pShellBrowser->QueryInterface(IID_PPV_ARG(ICommDlgBrowser, &m_pCommDlgBrowser))))
//     {
//         TRACE("-- CommDlgBrowser\n");
//     }
//
//     LoadViewState();
//     if (view_flags & SV3CVW3_FORCEVIEWMODE)
//         m_FolderSettings.ViewMode = requestedViewMode;
//     if (view_flags & SV3CVW3_FORCEFOLDERFLAGS)
//         m_FolderSettings.fFlags = (mask & flags) & ~IGNORE_FWF;
//
//     RECT rcView = *prcView;
//     Create(m_hWndParent, rcView, NULL, WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | WS_TABSTOP, 0, 0U);
//     if (m_hWnd == NULL)
//         return E_FAIL;
//
//     *hwnd = m_hWnd;
//
//     CheckToolbar();
//
//     if (!*hwnd)
//         return E_FAIL;
//
//     _DoFolderViewCB(SFVM_WINDOWCREATED, (WPARAM)m_hWnd, 0);
//
//     SetWindowPos((*this).m_hWnd, HWND_TOP, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW);
//     UpdateWindow((*this).m_hWnd);
//
//     if (!m_hMenu)
//     {
//         m_hMenu = CreateMenu();
//         m_pShellBrowser->InsertMenusSB(m_hMenu, &omw);
//         TRACE("-- after fnInsertMenusSB\n");
//     }
//
//     _MergeToolbar();
//
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::HandleRename(LPCITEMIDLIST pidl)
// {
//     if (!pidl)
//     {
//         int idx = m_ListView.GetNextItem(-1, LVNI_SELECTED | LVNI_FOCUSED);
//         if (idx < 0)
//             idx = m_ListView.GetNextItem(-1, LVNI_SELECTED);
//         pidl = _PidlByItem(idx);
//     }
//     if (ILFindLastID(pidl) != pidl)
//         return E_INVALIDARG;
//     return SelectItem(pidl, SVSI_EDIT | SVSI_ENSUREVISIBLE | SVSI_FOCUSED | SVSI_SELECT);
// }
// HRESULT CDefView::SelectAndPositionItem(int Idx, UINT fSVSI, POINT *ppt)
// {
//     if (Idx == -1)
//         return fSVSI == SVSI_DESELECTOTHERS ? SelectItem(-2, fSVSI) : E_INVALIDARG;
//     if (ppt)
//         m_ListView.SetItemPosition(Idx, ppt);
//     return SelectItem(Idx, fSVSI); // After SetItemPosition for SVSI_ENSUREVISIBLE
// }
// HRESULT STDMETHODCALLTYPE CDefView::SelectAndPositionItem(LPCITEMIDLIST item, UINT flags, POINT *point)
// {
//     if (!item)
//         return SelectAndPositionItem(-1, flags, point);
//     int idx = LV_FindItemByPidl(item);
//     return idx != -1 ? SelectAndPositionItem(idx, flags, point) : S_FALSE;
// }
// HRESULT STDMETHODCALLTYPE CDefView::Rearrange(LPARAM sort)
// {
//     _Sort(LOWORD(sort & SHCIDS_COLUMNMASK));
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetArrangeParam(LPARAM *sort)
// {
//     int col = m_sortInfo.ListColumn;
//     *sort = col != LISTVIEW_SORT_INFO::UNSPECIFIEDCOLUMN ? col : 0;
//     return col != LISTVIEW_SORT_INFO::UNSPECIFIEDCOLUMN ? S_OK : S_FALSE;
// }
// HRESULT STDMETHODCALLTYPE CDefView::ArrangeGrid()
// {
//     m_ListView.SetExtendedListViewStyle(LVS_EX_SNAPTOGRID, LVS_EX_SNAPTOGRID);
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::AutoArrange()
// {
//     m_ListView.ModifyStyle(0, LVS_AUTOARRANGE);
//     m_ListView.Arrange(LVA_DEFAULT);
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::AddObject(PITEMID_CHILD pidl, UINT *item)
// {
//     TRACE("(%p)->(%p %p)\n", this, pidl, item);
//     if (!m_ListView)
//     {
//         ERR("!m_ListView\n");
//         return E_FAIL;
//     }
//     *item = LV_AddItem(pidl);
//     return (int)*item >= 0 ? S_OK : E_OUTOFMEMORY;
// }
// HRESULT STDMETHODCALLTYPE CDefView::RemoveObject(PITEMID_CHILD pidl, UINT *item)
// {
//     TRACE("(%p)->(%p %p)\n", this, pidl, item);
//
//     if (!m_ListView)
//     {
//         ERR("!m_ListView\n");
//         return E_FAIL;
//     }
//
//     if (pidl)
//     {
//         *item = LV_FindItemByPidl(ILFindLastID(pidl));
//         m_ListView.DeleteItem(*item);
//     }
//     else
//     {
//         *item = 0;
//         m_ListView.DeleteAllItems();
//     }
//
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetObjectCount(UINT *count)
// {
//     TRACE("(%p)->(%p)\n", this, count);
//     *count = m_ListView.GetItemCount();
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::SetObjectCount(UINT count, UINT flags)
// {
//     FIXME("(%p)->(%d %x) stub\n", this, count, flags);
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDefView::UpdateObject(PITEMID_CHILD pidl_old, PITEMID_CHILD pidl_new, UINT *item)
// {
//     FIXME("(%p)->(%p %p %p) stub\n", this, pidl_old, pidl_new, item);
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDefView::RefreshObject(PITEMID_CHILD pidl, UINT *item)
// {
//     FIXME("(%p)->(%p %p) stub\n", this, pidl, item);
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDefView::SetRedraw(BOOL redraw)
// {
//     TRACE("(%p)->(%d)\n", this, redraw);
//     if (m_ListView)
//         m_ListView.SetRedraw(redraw);
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetSelectedCount(UINT *count)
// {
//     *count = m_ListView.GetSelectedCount();
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetSelectedObjects(PCUITEMID_CHILD **pidl, UINT *items)
// {
//     TRACE("(%p)->(%p %p)\n", this, pidl, items);
//
//     *items = GetSelections();
//
//     if (*items)
//     {
//         *pidl = static_cast<PCUITEMID_CHILD *>(LocalAlloc(0, *items * sizeof(PCUITEMID_CHILD)));
//         if (!*pidl)
//         {
//             return E_OUTOFMEMORY;
//         }
//
//         // It's documented that caller shouldn't free the PIDLs, only the array itself
//         memcpy(*pidl, m_apidl, *items * sizeof(PCUITEMID_CHILD));
//     }
//
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::IsDropOnSource(IDropTarget *drop_target)
// {
//     if ((m_iDragOverItem == -1 || m_pCurDropTarget == NULL) &&
//         (m_pSourceDataObject.p))
//     {
//         return S_OK;
//     }
//
//     return S_FALSE;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetDragPoint(POINT *pt)
// {
//     if (!pt)
//         return E_INVALIDARG;
//
//     *pt = m_ptFirstMousePos;
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetDropPoint(POINT *pt)
// {
//     FIXME("(%p)->(%p) stub\n", this, pt);
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDefView::MoveIcons(IDataObject *obj)
// {
//     TRACE("(%p)->(%p)\n", this, obj);
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDefView::SetItemPos(PCUITEMID_CHILD pidl, POINT *pt)
// {
//     FIXME("(%p)->(%p %p) stub\n", this, pidl, pt);
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDefView::IsBkDropTarget(IDropTarget *drop_target)
// {
//     FIXME("(%p)->(%p) stub\n", this, drop_target);
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDefView::SetClipboard(BOOL move)
// {
//     if (!move)
//         return S_OK;
//
//     UINT CutCount = 0;
//     for (int i = -1; (i = m_ListView.GetNextItem(i, LVNI_SELECTED)) != -1;)
//     {
//         if (!CutCount++)
//         {
//             // Turn off the LVIS_CUT LVM_SETCALLBACKMASK optimization
//             m_ListView.SendMessageW(LVM_SETCALLBACKMASK, m_ListView.SendMessageW(LVM_GETCALLBACKMASK, 0, 0) & ~LVIS_CUT, 0);
//             RefreshGhostedState();
//         }
//         m_ListView.SetItemState(i, LVIS_CUT, LVIS_CUT);
//     }
//     if (CutCount)
//     {
//         m_ClipboardChain.Hook(m_hWnd);
//         m_HasCutItems = true;
//     }
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::SetPoints(IDataObject *obj)
// {
//     FIXME("(%p)->(%p) stub\n", this, obj);
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDefView::GetItemSpacing(ITEMSPACING *spacing)
// {
//     FIXME("(%p)->(%p) stub\n", this, spacing);
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDefView::SetCallback(IShellFolderViewCB  *new_cb, IShellFolderViewCB **old_cb)
// {
//     if (old_cb)
//         *old_cb = m_pShellFolderViewCB.Detach();
//
//     m_pShellFolderViewCB = new_cb;
//     m_pFolderFilter = NULL;
//     if (new_cb)
//         new_cb->QueryInterface(IID_PPV_ARG(IFolderFilter, &m_pFolderFilter));
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::Select(UINT flags)
// {
//     FIXME("(%p)->(%d) stub\n", this, flags);
//     return E_NOTIMPL;
// }
// HRESULT STDMETHODCALLTYPE CDefView::QuerySupport(UINT *support)
// {
//     TRACE("(%p)->(%p)\n", this, support);
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::SetAutomationObject(IDispatch *disp)
// {
//     FIXME("(%p)->(%p) stub\n", this, disp);
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CDefView::QueryStatus(const GUID *pguidCmdGroup, ULONG cCmds, OLECMD *prgCmds, OLECMDTEXT *pCmdText)
// {
//     FIXME("(%p)->(%p(%s) 0x%08x %p %p\n",
//           this, pguidCmdGroup, debugstr_guid(pguidCmdGroup), cCmds, prgCmds, pCmdText);
//
//     if (!prgCmds)
//         return E_INVALIDARG;
//
//     for (UINT i = 0; i < cCmds; i++)
//     {
//         FIXME("\tprgCmds[%d].cmdID = %d\n", i, prgCmds[i].cmdID);
//         prgCmds[i].cmdf = 0;
//     }
//
//     return OLECMDERR_E_UNKNOWNGROUP;
// }
// HRESULT WINAPI CDefView::Exec(const GUID *pguidCmdGroup, DWORD nCmdID, DWORD nCmdexecopt, VARIANT *pvaIn, VARIANT *pvaOut)
// {
//     FIXME("(%p)->(\n\tTarget GUID:%s Command:0x%08x Opt:0x%08x %p %p)\n",
//           this, debugstr_guid(pguidCmdGroup), nCmdID, nCmdexecopt, pvaIn, pvaOut);
//
//     if (!pguidCmdGroup)
//         return OLECMDERR_E_UNKNOWNGROUP;
//
//     if (IsEqualCLSID(*pguidCmdGroup, m_Category))
//     {
//         if (nCmdID == FCIDM_SHVIEW_AUTOARRANGE)
//         {
//             if (V_VT(pvaIn) != VT_INT_PTR)
//                 return OLECMDERR_E_NOTSUPPORTED;
//
//             TPMPARAMS params;
//             params.cbSize = sizeof(params);
//             params.rcExclude = *(RECT*) V_INTREF(pvaIn);
//
//             if (m_hMenuViewModes)
//             {
//                 // Duplicate all but the last two items of the view modes menu
//                 HMENU hmenuViewPopup = CreatePopupMenu();
//                 Shell_MergeMenus(hmenuViewPopup, m_hMenuViewModes, 0, 0, 0xFFFF, 0);
//                 DeleteMenu(hmenuViewPopup, GetMenuItemCount(hmenuViewPopup) - 1, MF_BYPOSITION);
//                 DeleteMenu(hmenuViewPopup, GetMenuItemCount(hmenuViewPopup) - 1, MF_BYPOSITION);
//                 CheckViewMode(hmenuViewPopup);
//                 TrackPopupMenuEx(hmenuViewPopup, TPM_LEFTALIGN | TPM_TOPALIGN, params.rcExclude.left, params.rcExclude.bottom, m_hWndParent, &params);
//                 ::DestroyMenu(hmenuViewPopup);
//             }
//
//             // pvaOut is VT_I4 with value 0x403 (cmd id of the new mode maybe?)
//             V_VT(pvaOut) = VT_I4;
//             V_I4(pvaOut) = 0x403;
//         }
//     }
//
//     if (IsEqualIID(*pguidCmdGroup, CGID_Explorer) &&
//             (nCmdID == 0x29) &&
//             (nCmdexecopt == 4) && pvaOut)
//         return S_OK;
//
//     if (IsEqualIID(*pguidCmdGroup, CGID_ShellDocView) &&
//             (nCmdID == 9) &&
//             (nCmdexecopt == 0))
//         return 1;
//
//     if (IsEqualIID(*pguidCmdGroup, CGID_DefView))
//     {
//         CComPtr<IStream> pStream;
//         WCHAR SubKey[MAX_PATH];
//         switch (nCmdID)
//         {
//             case DVCMDID_SET_DEFAULTFOLDER_SETTINGS:
//                 SHDeleteKey(HKEY_CURRENT_USER, L"Software\\Microsoft\\Windows\\ShellNoRoam\\Bags");
//                 if (SUCCEEDED(GetDefaultViewStream(STGM_WRITE, &pStream)))
//                     SaveViewState(pStream);
//                 return S_OK;
//             case DVCMDID_RESET_DEFAULTFOLDER_SETTINGS:
//                 PathCombineW(SubKey, REGSTR_PATH_EXPLORER, L"Streams\\Default");
//                 SHDeleteKey(HKEY_CURRENT_USER, SubKey);
//                 SHDeleteKey(HKEY_CURRENT_USER, L"Software\\Microsoft\\Windows\\ShellNoRoam\\Bags");
//                 m_FolderSettings.fFlags |= FWF_NOBROWSERVIEWSTATE; // Don't let this folder save itself
//                 return S_OK;
//         }
//     }
//
//     return OLECMDERR_E_UNKNOWNGROUP;
// }
// HRESULT CDefView::drag_notify_subitem(DWORD grfKeyState, POINTL pt, DWORD *pdwEffect)
// {
//     LONG lResult;
//     HRESULT hr;
//     RECT clientRect;
//
//     /* The key state on drop doesn't have MK_LBUTTON or MK_RBUTTON because it
//        reflects the key state after the user released the button, so we need
//        to remember the last key state when the button was pressed */
//     m_grfKeyState = grfKeyState;
//
//     // Map from global to client coordinates and query the index of the
//     // listview-item, which is currently under the mouse cursor.
//     LVHITTESTINFO htinfo = {{pt.x, pt.y}, LVHT_ONITEM};
//     ScreenToClient(&htinfo.pt);
//     lResult = m_ListView.HitTest(&htinfo);
//
//     /* Send WM_*SCROLL messages every 250 ms during drag-scrolling */
//     ::GetClientRect(m_ListView, &clientRect);
//     if (htinfo.pt.x == m_ptLastMousePos.x && htinfo.pt.y == m_ptLastMousePos.y &&
//             (htinfo.pt.x < SCROLLAREAWIDTH || htinfo.pt.x > clientRect.right - SCROLLAREAWIDTH ||
//              htinfo.pt.y < SCROLLAREAWIDTH || htinfo.pt.y > clientRect.bottom - SCROLLAREAWIDTH))
//     {
//         m_cScrollDelay = (m_cScrollDelay + 1) % 5; // DragOver is called every 50 ms
//         if (m_cScrollDelay == 0)
//         {
//             /* Mouse did hover another 250 ms over the scroll-area */
//             if (htinfo.pt.x < SCROLLAREAWIDTH)
//                 m_ListView.SendMessageW(WM_HSCROLL, SB_LINEUP, 0);
//
//             if (htinfo.pt.x > clientRect.right - SCROLLAREAWIDTH)
//                 m_ListView.SendMessageW(WM_HSCROLL, SB_LINEDOWN, 0);
//
//             if (htinfo.pt.y < SCROLLAREAWIDTH)
//                 m_ListView.SendMessageW(WM_VSCROLL, SB_LINEUP, 0);
//
//             if (htinfo.pt.y > clientRect.bottom - SCROLLAREAWIDTH)
//                 m_ListView.SendMessageW(WM_VSCROLL, SB_LINEDOWN, 0);
//         }
//     }
//     else
//     {
//         m_cScrollDelay = 0; // Reset, if cursor is not over the listview's scroll-area
//     }
//
//     m_ptLastMousePos = htinfo.pt;
//     ::ClientToListView(m_ListView, &m_ptLastMousePos);
//
//     /* We need to check if we drag the selection over itself */
//     if (lResult != -1 && m_pSourceDataObject.p != NULL)
//     {
//         PCUITEMID_CHILD pidl = _PidlByItem(lResult);
//
//         for (UINT i = 0; i < m_cidl; i++)
//         {
//             if (pidl == m_apidl[i])
//             {
//                 /* The item that is being draged is hovering itself. */
//                 lResult = -1;
//                 break;
//             }
//         }
//     }
//
//     // If we are still over the previous sub-item, notify it via DragOver and return
//     if (m_pCurDropTarget && lResult == m_iDragOverItem)
//         return m_pCurDropTarget->DragOver(grfKeyState, pt, pdwEffect);
//
//     // We've left the previous sub-item, notify it via DragLeave and release it
//     if (m_pCurDropTarget)
//     {
//         PCUITEMID_CHILD pidl = _PidlByItem(m_iDragOverItem);
//         if (pidl)
//             SelectItem(pidl, 0);
//
//         m_pCurDropTarget->DragLeave();
//         m_pCurDropTarget.Release();
//     }
//
//     m_iDragOverItem = lResult;
//
//     if (lResult == -1)
//     {
//         // We are not above one of the listview's subitems. Bind to the
//         // parent folder's DropTarget interface.
//         hr = m_pSFParent->CreateViewObject(NULL, IID_PPV_ARG(IDropTarget,&m_pCurDropTarget));
//     }
//     else
//     {
//         // Query the relative PIDL of the shellfolder object represented
//         // by the currently dragged over listview-item ...
//         PCUITEMID_CHILD pidl = _PidlByItem(lResult);
//
//         // ... and bind m_pCurDropTarget to the IDropTarget interface of an UIObject of this object
//         hr = m_pSFParent->GetUIObjectOf(m_ListView, 1, &pidl, IID_NULL_PPV_ARG(IDropTarget, &m_pCurDropTarget));
//     }
//
//     IUnknown_SetSite(m_pCurDropTarget, (IShellView *)this);
//
//     // If anything failed, m_pCurDropTarget should be NULL now, which ought to be a save state
//     if (FAILED(hr))
//     {
//         *pdwEffect = DROPEFFECT_NONE;
//         return hr;
//     }
//
//     if (m_iDragOverItem != -1)
//     {
//         SelectItem(m_iDragOverItem, SVSI_SELECT);
//     }
//
//     // Notify the item just entered via DragEnter
//     return m_pCurDropTarget->DragEnter(m_pCurDataObject, grfKeyState, pt, pdwEffect);
// }
// HRESULT WINAPI CDefView::DragEnter(IDataObject *pDataObject, DWORD grfKeyState, POINTL pt, DWORD *pdwEffect)
// {
//     if (*pdwEffect == DROPEFFECT_NONE)
//         return S_OK;
//
//     /* Get a hold on the data object for later calls to DragEnter on the sub-folders */
//     m_pCurDataObject = pDataObject;
//
//     HRESULT hr = drag_notify_subitem(grfKeyState, pt, pdwEffect);
//     if (SUCCEEDED(hr))
//     {
//         POINT ptClient = {pt.x, pt.y};
//         ScreenToClient(&ptClient);
//         ImageList_DragEnter(m_hWnd, ptClient.x, ptClient.y);
//     }
//
//     return hr;
// }
// HRESULT WINAPI CDefView::DragOver(DWORD grfKeyState, POINTL pt, DWORD *pdwEffect)
// {
//     POINT ptClient = {pt.x, pt.y};
//     ScreenToClient(&ptClient);
//     ImageList_DragMove(ptClient.x, ptClient.y);
//     return drag_notify_subitem(grfKeyState, pt, pdwEffect);
// }
// HRESULT WINAPI CDefView::DragLeave()
// {
//     ImageList_DragLeave(m_hWnd);
//
//     if (m_pCurDropTarget)
//     {
//         m_pCurDropTarget->DragLeave();
//         m_pCurDropTarget.Release();
//     }
//
//     if (m_pCurDataObject != NULL)
//     {
//         m_pCurDataObject.Release();
//     }
//
//     m_iDragOverItem = 0;
//
//     return S_OK;
// }
// INT CDefView::_FindInsertableIndexFromPoint(POINT pt)
// {
//     RECT rcBound;
//     INT i, nCount = m_ListView.GetItemCount();
//     DWORD dwSpacing;
//     INT dx, dy;
//     BOOL bSmall = ((m_ListView.GetStyle() & LVS_TYPEMASK) != LVS_ICON);
//
//     // FIXME: LVM_GETORIGIN is broken. See CORE-17266
//     pt.x += m_ListView.GetScrollPos(SB_HORZ);
//     pt.y += m_ListView.GetScrollPos(SB_VERT);
//
//     if (m_ListView.GetStyle() & LVS_ALIGNLEFT)
//     {
//         // vertically
//         for (i = 0; i < nCount; ++i)
//         {
//             dwSpacing = ListView_GetItemSpacing(m_ListView, bSmall);
//             dx = LOWORD(dwSpacing);
//             dy = HIWORD(dwSpacing);
//             ListView_GetItemRect(m_ListView, i, &rcBound, LVIR_SELECTBOUNDS);
//             rcBound.right = rcBound.left + dx;
//             rcBound.bottom = rcBound.top + dy;
//             if (pt.x < rcBound.right && pt.y < (rcBound.top + rcBound.bottom) / 2)
//             {
//                 return i;
//             }
//         }
//         for (i = nCount - 1; i >= 0; --i)
//         {
//             ListView_GetItemRect(m_ListView, i, &rcBound, LVIR_SELECTBOUNDS);
//             if (rcBound.left < pt.x && rcBound.top < pt.y)
//             {
//                 return i + 1;
//             }
//         }
//     }
//     else
//     {
//         // horizontally
//         for (i = 0; i < nCount; ++i)
//         {
//             dwSpacing = ListView_GetItemSpacing(m_ListView, bSmall);
//             dx = LOWORD(dwSpacing);
//             dy = HIWORD(dwSpacing);
//             ListView_GetItemRect(m_ListView, i, &rcBound, LVIR_SELECTBOUNDS);
//             rcBound.right = rcBound.left + dx;
//             rcBound.bottom = rcBound.top + dy;
//             if (pt.y < rcBound.bottom && pt.x < rcBound.left)
//             {
//                 return i;
//             }
//             if (pt.y < rcBound.bottom && pt.x < rcBound.right)
//             {
//                 return i + 1;
//             }
//         }
//         for (i = nCount - 1; i >= 0; --i)
//         {
//             ListView_GetItemRect(m_ListView, i, &rcBound, LVIR_SELECTBOUNDS);
//             if (rcBound.left < pt.x && rcBound.top < pt.y)
//             {
//                 return i + 1;
//             }
//         }
//     }
//
//     return nCount;
// }
// void CDefView::_HandleStatusBarResize(int nWidth)
// {
//     LRESULT lResult;
//
//     if (!m_isFullStatusBar)
//     {
//         int nPartArray[] = {-1};
//         m_pShellBrowser->SendControlMsg(FCW_STATUS, SB_SETPARTS, _countof(nPartArray), (LPARAM)nPartArray, &lResult);
//         return;
//     }
//
//     int nFileSizePartLength = 125;
//     const int nLocationPartLength = 150;
//     const int nRightPartsLength = nFileSizePartLength + nLocationPartLength;
//     int nObjectsPartLength = nWidth - nRightPartsLength;
//
//     // If the window is small enough just divide each part into thirds
//     // to match the behavior of Windows Server 2003
//     if (nObjectsPartLength <= nLocationPartLength)
//         nObjectsPartLength = nFileSizePartLength = nWidth / 3;
//
//     int nPartArray[] = {nObjectsPartLength, nObjectsPartLength + nFileSizePartLength, -1};
//
//     m_pShellBrowser->SendControlMsg(FCW_STATUS, SB_SETPARTS, _countof(nPartArray), (LPARAM)nPartArray, &lResult);
// }
// void CDefView::_ForceStatusBarResize()
// {
//     // Get the handle for the status bar
//     HWND fStatusBar;
//     m_pShellBrowser->GetControlWindow(FCW_STATUS, &fStatusBar);
//
//     // Get the size of our status bar
//     RECT statusBarSize;
//     ::GetWindowRect(fStatusBar, &statusBarSize);
//
//     // Resize the status bar
//     _HandleStatusBarResize(statusBarSize.right - statusBarSize.left);
// }
// void CDefView::_MoveSelectionOnAutoArrange(POINT pt)
// {
//     // get insertable index from position
//     INT iPosition = _FindInsertableIndexFromPoint(pt);
//
//     // create identity mapping of indexes
//     CSimpleArray<INT> array;
//     INT nCount = m_ListView.GetItemCount();
//     for (INT i = 0; i < nCount; ++i)
//     {
//         array.Add(i);
//     }
//
//     // re-ordering mapping
//     INT iItem = -1;
//     while ((iItem = m_ListView.GetNextItem(iItem, LVNI_SELECTED)) >= 0)
//     {
//         INT iFrom = iItem, iTo = iPosition;
//         if (iFrom < iTo)
//             --iTo;
//         if (iFrom >= nCount)
//             iFrom = nCount - 1;
//         if (iTo >= nCount)
//             iTo = nCount - 1;
//
//         // shift indexes by swapping (like a bucket relay)
//         if (iFrom < iTo)
//         {
//             for (INT i = iFrom; i < iTo; ++i)
//             {
//                 // swap array[i] and array[i + 1]
//                 INT tmp = array[i];
//                 array[i] = array[i + 1];
//                 array[i + 1] = tmp;
//             }
//         }
//         else
//         {
//             for (INT i = iFrom; i > iTo; --i)
//             {
//                 // swap array[i] and array[i - 1]
//                 INT tmp = array[i];
//                 array[i] = array[i - 1];
//                 array[i - 1] = tmp;
//             }
//         }
//     }
//
//     // create mapping (ListView's lParam to index) from array
//     CLParamIndexMap map;
//     for (INT i = 0; i < nCount; ++i)
//     {
//         LPARAM lParam = m_ListView.GetItemData(*array.GetAt(i));
//         map.Add(lParam, i);
//     }
//
//     // finally sort
//     m_ListView.SortItems(SelectionMoveCompareFunc, &map);
// }
// HRESULT WINAPI CDefView::Drop(IDataObject* pDataObject, DWORD grfKeyState, POINTL pt, DWORD *pdwEffect)
// {
//     ImageList_DragLeave(m_hWnd);
//     ImageList_EndDrag();
//
//     if ((IsDropOnSource(NULL) == S_OK) &&
//         (*pdwEffect & DROPEFFECT_MOVE) &&
//         (m_grfKeyState & MK_LBUTTON))
//     {
//         if (m_pCurDropTarget)
//         {
//             m_pCurDropTarget->DragLeave();
//             m_pCurDropTarget.Release();
//         }
//
//         POINT ptDrop = { pt.x, pt.y };
//         ::ScreenToClient(m_ListView, &ptDrop);
//         ::ClientToListView(m_ListView, &ptDrop);
//         m_ptLastMousePos = ptDrop;
//
//         m_ListView.SetRedraw(FALSE);
//         if (m_ListView.GetStyle() & LVS_AUTOARRANGE)
//         {
//             _MoveSelectionOnAutoArrange(m_ptLastMousePos);
//         }
//         else
//         {
//             POINT ptItem;
//             INT iItem = -1;
//             while ((iItem = m_ListView.GetNextItem(iItem, LVNI_SELECTED)) >= 0)
//             {
//                 if (m_ListView.GetItemPosition(iItem, &ptItem))
//                 {
//                     ptItem.x += m_ptLastMousePos.x - m_ptFirstMousePos.x;
//                     ptItem.y += m_ptLastMousePos.y - m_ptFirstMousePos.y;
//                     m_ListView.SetItemPosition(iItem, &ptItem);
//                 }
//             }
//         }
//         m_ListView.SetRedraw(TRUE);
//     }
//     else if (m_pCurDropTarget)
//     {
//         m_pCurDropTarget->Drop(pDataObject, grfKeyState, pt, pdwEffect);
//         m_pCurDropTarget.Release();
//     }
//
//     m_pCurDataObject.Release();
//     m_iDragOverItem = 0;
//     return S_OK;
// }
// HRESULT WINAPI CDefView::QueryContinueDrag(BOOL fEscapePressed, DWORD grfKeyState)
// {
//     TRACE("(%p)\n", this);
//
//     if (fEscapePressed)
//         return DRAGDROP_S_CANCEL;
//     else if (!(grfKeyState & MK_LBUTTON) && !(grfKeyState & MK_RBUTTON))
//         return DRAGDROP_S_DROP;
//     else
//         return S_OK;
// }
// HRESULT WINAPI CDefView::GiveFeedback(DWORD dwEffect)
// {
//     TRACE("(%p)\n", this);
//
//     return DRAGDROP_S_USEDEFAULTCURSORS;
// }
// HRESULT WINAPI CDefView::Draw(DWORD dwDrawAspect, LONG lindex, void *pvAspect, DVTARGETDEVICE *ptd, HDC hdcTargetDev, HDC hdcDraw, LPCRECTL lprcBounds, LPCRECTL lprcWBounds, BOOL (__stdcall *pfnContinue)(ULONG_PTR dwContinue), ULONG_PTR dwContinue)
// {
//     FIXME("Stub: this=%p\n", this);
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CDefView::GetColorSet(DWORD dwDrawAspect, LONG lindex, void *pvAspect, DVTARGETDEVICE *ptd, HDC hicTargetDevice, LOGPALETTE **ppColorSet)
// {
//     FIXME("Stub: this=%p\n", this);
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CDefView::Freeze(DWORD dwDrawAspect, LONG lindex, void *pvAspect, DWORD *pdwFreeze)
// {
//     FIXME("Stub: this=%p\n", this);
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CDefView::Unfreeze(DWORD dwFreeze)
// {
//     FIXME("Stub: this=%p\n", this);
//     return E_NOTIMPL;
// }
// HRESULT WINAPI CDefView::SetAdvise(DWORD aspects, DWORD advf, IAdviseSink *pAdvSink)
// {
//     FIXME("partial stub: %p 0x%08x 0x%08x %p\n", this, aspects, advf, pAdvSink);
//
//     // FIXME: we set the AdviseSink, but never use it to send any advice
//     m_pAdvSink = pAdvSink;
//     m_dwAspects = aspects;
//     m_dwAdvf = advf;
//
//     return S_OK;
// }
// HRESULT WINAPI CDefView::GetAdvise(DWORD *pAspects, DWORD *pAdvf, IAdviseSink **ppAdvSink)
// {
//     TRACE("this=%p pAspects=%p pAdvf=%p ppAdvSink=%p\n", this, pAspects, pAdvf, ppAdvSink);
//
//     if (ppAdvSink)
//     {
//         *ppAdvSink = m_pAdvSink;
//         m_pAdvSink.p->AddRef();
//     }
//
//     if (pAspects)
//         *pAspects = m_dwAspects;
//
//     if (pAdvf)
//         *pAdvf = m_dwAdvf;
//
//     return S_OK;
// }
// HRESULT STDMETHODCALLTYPE CDefView::QueryService(REFGUID guidService, REFIID riid, void **ppvObject)
// {
//     if (IsEqualIID(guidService, SID_SFolderView))
//         return QueryInterface(riid, ppvObject);
//     return m_pShellBrowser ? IUnknown_QueryService(m_pShellBrowser, guidService, riid, ppvObject) : E_NOINTERFACE;
// }
// HRESULT CDefView::_MergeToolbar()
// {
//     CComPtr<IExplorerToolbar> ptb;
//     HRESULT hr = S_OK;
//
//     hr = IUnknown_QueryService(m_pShellBrowser, IID_IExplorerToolbar, IID_PPV_ARG(IExplorerToolbar, &ptb));
//     if (FAILED(hr))
//         return hr;
//
//     m_Category = CGID_DefViewFrame;
//
//     hr = ptb->SetCommandTarget(static_cast<IOleCommandTarget*>(this), &m_Category, 0);
//     if (FAILED(hr))
//         return hr;
//
//     if (hr == S_FALSE)
//         return S_OK;
//
//
//     return S_OK;
// }
// HRESULT CDefView::_DoFolderViewCB(UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     HRESULT hr = E_NOTIMPL;
//
//     if (m_pShellFolderViewCB)
//     {
//         hr = m_pShellFolderViewCB->MessageSFVCB(uMsg, wParam, lParam);
//     }
//
//     return hr;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
