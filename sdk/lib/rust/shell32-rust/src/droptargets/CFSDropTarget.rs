//! CFSDropTarget — the file-system folder's IDropTarget. Whole-TU flip of
//! `dll/win32/shell32/droptargets/CFSDropTarget.cpp` (commented out of the CMake source list);
//! 19 of the 21 bodies were bought, `_RepositionItems` came back on the sanctioned retry, and
//! `_DoDrop` is hand-written because the model refused it twice on the fact/call envelope.
//!
//! WHICH MODULE OWNS A TYPE is the whole story of this file, and it has three answers, all
//! load-bearing:
//!   * `shellcom` owns the COM vtable ABI — IDropTarget / IDataObject / IUnknown come from the
//!     explicit `use crate::shellcom::{...}` below, which outranks the prelude's island glob.
//!     The GUID family has to move WITH them: the slots are spelled with shellcom's `GUID`, the
//!     `IID_*` statics with `IID`, the signatures with `REFIID`, so taking some and not the rest
//!     merely moves the mismatch from the slots to the statics (measured: importing GUID/IID
//!     without REFIID left the error count exactly where importing none of them did).
//!   * The ISLAND owns data layouts, so every PIDL, POINT, STRRET and GUID that crosses INTO a
//!     shellcom vtable slot gets a `.cast()` at the call site. shellcom.rs:91 says why it keeps
//!     its own layout-identical copies (it is `#[path]`-included into menuband-rust, which has
//!     no island) and that the cast is the price.
//!   * This file owns the fourteen interfaces neither of the other two declares — including
//!     `IShellFolderView`, whose island form is bindgen's OPAQUE shape with no callable slots,
//!     and `_DoDropData`, whose island form names the island's ATL-shaped `CFSDropTarget` and
//!     so would read `data->This` at the wrong offsets on the receiving thread.
//!
//! Two trailing-slot extensions made this compile, and both are additive for the same reason:
//! a `usize` slot is the same SIZE as a fn pointer and cannot be called, so typing one changes
//! the vtable's type and not its layout — and nothing in the tree constructs either vtable as a
//! literal. shellcom's `IDataObject` got QueryInterface/AddRef/Release/QueryGetData typed, and
//! its `IFolderViewVtbl` — which stopped dead at `GetFolder` — got the remaining eleven slots
//! from shobjidl.idl:866 so `GetItemPosition` and `SelectAndPositionItems` sit at the offsets
//! the C++ vtable puts them at.
//!
//! One measurement worth keeping: the error count ROSE from 2 to 89 when the last two
//! name-resolution errors were fixed, because name resolution aborts before type checking ever
//! runs. A low count that comes with unresolved names is masking, not progress.

// rustconv-assembly: class:CFSDropTarget
// Deterministic from oleidl.h
// ONE glob only. The scaffold header emits `use crate::prelude::*` and the body assembly emits
// `use crate::{shellcom,types,win32}::*`; having both makes all 311 shared names E0659-
// ambiguous (846 errors), which is precisely what prelude.rs's own header warns about.
// Glob choice controls NAME resolution and declarations control TYPE identity, and the two are
// orthogonal: prelude-alone left 80 errors and types/win32/shellcom-alone left 78, because they
// fail differently. The prelude glob resolves far more names, so it is the one kept, and the
// per-type ownership is then stated explicitly below.
use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::desktop_folder_tu::SHGetDesktopFolder;
// One declaration per external symbol: island.rs.
use crate::island::DestroyMenu;
use crate::island::DestroyWindow;
use crate::island::IUnknown_QueryService;
use crate::island::LoadStringW;
use crate::island::PathRemoveExtensionW;
use crate::island::SHCreateThread;
// Explicit, not a glob: prelude globs the ISLAND, which declares its own IDataObject /
// IDropTarget / IUnknown. An explicit import outranks a glob, so naming these three here is
// what makes shellcom's the one nominal identity in this file — the same three whose local
// copies were deleted above.
use crate::shellcom::{
    IDataObject, IDataObjectVtbl, IDropTarget, IDropTargetVtbl, IUnknown, IUnknownVtbl,
    REFGUID, REFIID, SIGDN_DESKTOPABSOLUTEPARSING,
    // GUID/IID/CLSID come from here too, and it has to be all-or-nothing: the vtable slots
    // are spelled with shellcom's GUID and the IID_* statics below are declared with `IID`,
    // so taking one from shellcom and the other from the island's glob just moves the
    // mismatch from the slots to the statics. Importing GUID/IID without REFIID does the
    // same thing, which is why an earlier attempt at this stayed at 89 errors.
    CLSID, GUID, IID,
};
// The vtable slots above are spelled with `crate::types`' POINTL, and
// the island's POINTL is a DIFFERENT nominal type (ntdll_sys::nt::_POINTL vs
// com_sys::_POINTL), so the fn items would not coerce into the slots. GUID is the same
// story but pulls the OTHER way — the IID_* statics are island-typed — so it is left to
// the glob and aligned per site instead.
use crate::types::POINTL;

// ---- integration substrate (hand-added; the assembly references these but declares none) ----
// Explicit imports, not a second glob: prelude.rs owns the one glob in this module and says so.
use crate::shell32::shlfolder::{AddPidlClassKeysToArray, SH_GetApidlFromDataObject};
use crate::shell32::shldataobject::SHGetAttributesFromDataObject;
use crate::shell32::shlfileop::SHFileOperationW;
use crate::shell_link::CShellLink_CreateInstance;
use crate::shellutils::CFSTR_SHELLIDLISTW;
// SHELL_ErrorBox is a C++ TEMPLATE in shellutils.h, so it has no linkable symbol of its own —
// utils.cpp publishes SHELL_ErrorBox_RustBridge instead, and win32.rs owns the one `#[link_name]`
// declaration the whole crate shares. Declaring it locally as a plain `SHELL_ErrorBox` links
// against a symbol that does not exist (i386: `_SHELL_ErrorBox@8` undefined).
use crate::win32::SHELL_ErrorBox;
use crate::wine::pidl::{SHELL32_CreateSimpleIDListFromPath, _ILCopyCidaToaPidl, _ILFreeaPidl};
use crate::wine::shellole::DragQueryFileW;

// CFSDropTarget.cpp:33-35 — file-local #defines, so they exist in no header and no island.
const D_COPY: DWORD = DROPEFFECT_COPY;
const D_MOVE: DWORD = DROPEFFECT_MOVE;
const D_LINK: DWORD = DROPEFFECT_LINK;

/// `L"Static"` — the window class of the drag-image placeholder (WC_STATIC, winuser.h).
static WC_STATIC_W: [WCHAR; 7] = [0x0053, 0x0074, 0x0061, 0x0074, 0x0069, 0x0063, 0x0000];

use crate::shfldr::CRegKeyHandleArray;

// shell32_main.h:153 — a #define, so it is in no header the island reads.
#[inline]
unsafe fn KeyStateToDropEffect(kst: DWORD) -> DWORD {
    if (kst & (MK_CONTROL | MK_SHIFT)) == (MK_CONTROL | MK_SHIFT) {
        DROPEFFECT_LINK
    } else if (kst & MK_CONTROL) != 0 {
        DROPEFFECT_COPY
    } else if (kst & MK_SHIFT) != 0 {
        DROPEFFECT_MOVE
    } else {
        DROPEFFECT_NONE
    }
}

// shell32_main.h:144 — a #define, so it is in no header the island reads. An out-of-line fn
// is safe here where it would not be for a debug macro: this one has a real body, so it is
// not the no-op-optimization-barrier shape that uncloaks ABI mismatches.
#[inline]
unsafe fn InitFormatEtc(fe: *mut FORMATETC, cf: UINT, med: DWORD) {
    (*fe).cfFormat = cf as CLIPFORMAT;
    (*fe).dwAspect = DVASPECT_CONTENT;
    (*fe).ptd = core::ptr::null_mut();
    (*fe).tymed = med;
    (*fe).lindex = -1;
}

// utils.h:124 / precomp.h:198 — both are `#define X_Create() (HDSA)DSA_Create(sizeof(T), 4)`.
type HDCIA = HDSA;
type HDCMA = HDSA;
#[inline]
unsafe fn DCIA_Create() -> HDCIA {
    DSA_Create(core::mem::size_of::<CLSID>() as INT, 4)
}
#[inline]
unsafe fn DCMA_Create() -> HDCMA {
    DSA_Create(core::mem::size_of::<DCMENTRY>() as INT, 4)
}

// utils.h:125 — `#define DCIA_Destroy(h) DSA_Destroy((HDSA)(h))`, so there is no symbol.
#[inline]
unsafe fn DCIA_Destroy(hDCIA: HDCIA) {
    DSA_Destroy(hDCIA);
}

// DSA_Create/DSA_Destroy are NOT declared here: they are comctl32 exports spelled `WINAPI`, so
// the island's `extern "stdcall"` declarations (reached through the prelude glob) are the
// correct ones. A local `extern "C"` redeclaration compiles and then fails the i386 link on
// `_DSA_Create`, because cdecl and stdcall decorate the name differently.
//
// The four below are the opposite problem: they are ordinary C++ functions in utils.cpp and
// CDefaultContextMenu.cpp — no EXTERN_C, no WINAPI — so their real symbols are MANGLED and no
// spelling of an extern block reaches them. Each gets a `_RustBridge` shim on the C++ side,
// the same treatment SHELL_ErrorBox and the other cross-language helpers already have.
extern "system" {
    #[link_name = "DCIA_AddShellExSubkey_RustBridge"]
    fn DCIA_AddShellExSubkey(hDCIA: HDCIA, hProgId: HKEY, pszSubkey: PCWSTR);
    #[link_name = "DCMA_InsertMenuItems_RustBridge"]
    fn DCMA_InsertMenuItems(
        hDCMA: HDCMA,
        hDCIA: HDCIA,
        pidlFolder: LPCITEMIDLIST,
        pDO: *mut IDataObject,
        pKeys: *mut HKEY,
        nKeys: UINT,
        pQCMI: *mut QCMINFO,
        fCmf: UINT,
        pUnkSite: *mut IUnknown,
    ) -> UINT;
    #[link_name = "DCMA_InvokeCommand_RustBridge"]
    fn DCMA_InvokeCommand(hDCMA: HDCMA, pICI: *mut CMINVOKECOMMANDINFO) -> HRESULT;
    #[link_name = "DCMA_Destroy_RustBridge"]
    fn DCMA_Destroy(hDCMA: HDCMA);
}

extern "C" {
    pub static IID_IDropTarget: IID;
}

// Deterministic from ocidl.h
#[repr(C)]
pub struct IObjectWithSite {
    pub lpVtbl: *const IObjectWithSiteVtbl,
}
#[repr(C)]
pub struct IObjectWithSiteVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IObjectWithSite) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IObjectWithSite) -> ULONG,
    pub SetSite: unsafe extern "system" fn(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT,
    pub GetSite: unsafe extern "system" fn(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT,
}
extern "C" {
    pub static IID_IObjectWithSite: IID;
}

// Deterministic from shlobj.h:893 (DECLARE_INTERFACE_(IShellFolderView, IUnknown)).
// The island's IShellFolderView is bindgen's OPAQUE form — a bare `vtable_: *const c_void`
// with no callable slots — so it cannot express `psfv->GetDragPoint(...)`. A local
// declaration outranks the glob, same as the thirteen other interfaces above. Slot ORDER is
// the whole contract here: everything up to `SetAutomationObject` is listed so the four slots
// this file calls sit at the offsets the C++ vtable puts them at. Untyped slots are `usize`
// placeholders — same size as a fn pointer, so the layout is identical either way, and a
// `usize` cannot be called by accident.
#[repr(C)]
pub struct IShellFolderView {
    pub lpVtbl: *const IShellFolderViewVtbl,
}
#[repr(C)]
pub struct IShellFolderViewVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellFolderView, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellFolderView) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellFolderView) -> ULONG,
    pub Rearrange: usize,
    pub GetArrangeParam: usize,
    pub ArrangeGrid: usize,
    pub AutoArrange: usize,
    pub GetAutoArrange: usize,
    pub AddObject: usize,
    pub GetObject: usize,
    pub RemoveObject: usize,
    pub GetObjectCount: usize,
    pub SetObjectCount: usize,
    pub UpdateObject: usize,
    pub RefreshObject: usize,
    pub SetRedraw: usize,
    pub GetSelectedCount: usize,
    pub GetSelectedObjects: usize,
    pub IsDropOnSource: unsafe extern "system" fn(This: *mut IShellFolderView, pDropTarget: *mut IDropTarget) -> HRESULT,
    pub GetDragPoint: unsafe extern "system" fn(This: *mut IShellFolderView, ppt: *mut POINT) -> HRESULT,
    pub GetDropPoint: usize,
    pub MoveIcons: usize,
    pub SetItemPos: usize,
    pub IsBkDropTarget: usize,
    pub SetClipboard: usize,
    pub SetPoints: usize,
    pub GetItemSpacing: usize,
    pub SetCallback: usize,
    pub Select: usize,
    pub QuerySupport: usize,
    pub SetAutomationObject: usize,
}

// Deterministic from objidl.h
extern "C" {
    pub static IID_IDataObject: IID;
}

// Deterministic from unknwn.h
extern "C" {
    pub static IID_IUnknown: IID;
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

// Deterministic from objidl.h
#[repr(C)]
pub struct IBindCtx {
    pub lpVtbl: *const IBindCtxVtbl,
}
#[repr(C)]
pub struct IBindCtxVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IBindCtx, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IBindCtx) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IBindCtx) -> ULONG,
    pub RegisterObjectBound: unsafe extern "system" fn(This: *mut IBindCtx, punk: *mut IUnknown) -> HRESULT,
    pub RevokeObjectBound: unsafe extern "system" fn(This: *mut IBindCtx, punk: *mut IUnknown) -> HRESULT,
    pub ReleaseBoundObjects: unsafe extern "system" fn(This: *mut IBindCtx) -> HRESULT,
    pub SetBindOptions: unsafe extern "system" fn(This: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT,
    pub GetBindOptions: unsafe extern "system" fn(This: *mut IBindCtx, pbindopts: *mut BIND_OPTS) -> HRESULT,
    pub GetRunningObjectTable: unsafe extern "system" fn(This: *mut IBindCtx, pprot: *mut *mut IRunningObjectTable) -> HRESULT,
    pub RegisterObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR, punk: *mut IUnknown) -> HRESULT,
    pub GetObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR, ppunk: *mut *mut IUnknown) -> HRESULT,
    pub EnumObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, ppenum: *mut *mut IEnumString) -> HRESULT,
    pub RevokeObjectParam: unsafe extern "system" fn(This: *mut IBindCtx, pszKey: LPOLESTR) -> HRESULT,
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
pub struct IStream {
    pub lpVtbl: *const IStreamVtbl,
}
#[repr(C)]
pub struct IStreamVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IStream, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IStream) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IStream) -> ULONG,
    pub Read: unsafe extern "system" fn(This: *mut IStream, pv: *mut core::ffi::c_void, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT,
    pub Write: unsafe extern "system" fn(This: *mut IStream, pv: *const core::ffi::c_void, cb: ULONG, pcbWritten: *mut ULONG) -> HRESULT,
    pub Seek: unsafe extern "system" fn(This: *mut IStream, dlibMove: LARGE_INTEGER, dwOrigin: DWORD, plibNewPosition: *mut ULARGE_INTEGER) -> HRESULT,
    pub SetSize: unsafe extern "system" fn(This: *mut IStream, libNewSize: ULARGE_INTEGER) -> HRESULT,
    pub CopyTo: unsafe extern "system" fn(This: *mut IStream, pstm: *mut IStream, cb: ULARGE_INTEGER, pcbRead: *mut ULARGE_INTEGER, pcbWritten: *mut ULARGE_INTEGER) -> HRESULT,
    pub Commit: unsafe extern "system" fn(This: *mut IStream, grfCommitFlags: DWORD) -> HRESULT,
    pub Revert: unsafe extern "system" fn(This: *mut IStream) -> HRESULT,
    pub LockRegion: unsafe extern "system" fn(This: *mut IStream, libOffset: ULARGE_INTEGER, cb: ULARGE_INTEGER, dwLockType: DWORD) -> HRESULT,
    pub UnlockRegion: unsafe extern "system" fn(This: *mut IStream, libOffset: ULARGE_INTEGER, cb: ULARGE_INTEGER, dwLockType: DWORD) -> HRESULT,
    pub Stat: unsafe extern "system" fn(This: *mut IStream, pstatstg: *mut STATSTG, grfStatFlag: DWORD) -> HRESULT,
    pub Clone: unsafe extern "system" fn(This: *mut IStream, ppstm: *mut *mut IStream) -> HRESULT,
}
extern "C" {
    pub static IID_IStream: IID;
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

// Deterministic COM scaffold for CFSDropTarget; method bodies are generated separately.
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
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())
}
#[inline]
unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    cpp_com_HeapFree(heap, flags, memory.cast())
}

#[repr(C)]
pub struct CFSDropTarget {
    pub IDropTarget_iface: IDropTarget,
    pub IObjectWithSite_iface: IObjectWithSite,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub m_cfShellIDList: UINT,
    pub m_fAcceptFmt: BOOL,
    pub m_sPathTarget: LPWSTR,
    pub m_hwndSite: HWND,
    pub m_grfKeyState: DWORD,
    pub m_dwDefaultEffect: DWORD,
    pub m_AllowedEffects: DWORD,
    pub m_site: *mut IUnknown,
}

// CFSDropTarget.h:78 — the async-drop thread's argument block. It MUST be declared here and
// not taken from the island glob: the island's copy names the island's own `CFSDropTarget`
// (bindgen's ATL-shaped `vtable_ + _base: ATL_CComObjectRootEx + ...`), which is a different
// nominal type from the one above and, worse, a different LAYOUT — so `data->This` would be
// read at the wrong offsets on the receiving thread.
#[repr(C)]
pub struct _DoDropData {
    pub This: *mut CFSDropTarget,
    pub pStream: *mut IStream,
    pub dwKeyState: DWORD,
    pub pt: POINTL,
    pub pdwEffect: DWORD,
}

unsafe fn CFSDropTarget_cpp_com_assign_m_site(this: *mut CFSDropTarget, value: *mut IUnknown) {
    let old = (*this).m_site;
    if old == value { return; }
    if !value.is_null() {
        let unknown = value as *mut IUnknown;
        let add_ref: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).AddRef);
        add_ref(unknown);
    }
    (*this).m_site = value;
    if !old.is_null() {
        let unknown = old as *mut IUnknown;
        let release: unsafe extern "system" fn(*mut IUnknown) -> ULONG =
            core::mem::transmute((*(*unknown).lpVtbl).Release);
        release(unknown);
    }
}

unsafe extern "system" fn CFSDropTarget_IDropTarget_QueryInterface(This: *mut IDropTarget, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSDropTarget, IDropTarget_iface)) as *mut CFSDropTarget;
    CFSDropTarget_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFSDropTarget_IDropTarget_AddRef(This: *mut IDropTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSDropTarget, IDropTarget_iface)) as *mut CFSDropTarget;
    CFSDropTarget_AddRef(this)
}
unsafe extern "system" fn CFSDropTarget_IDropTarget_Release(This: *mut IDropTarget) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSDropTarget, IDropTarget_iface)) as *mut CFSDropTarget;
    CFSDropTarget_Release(this)
}
unsafe extern "system" fn CFSDropTarget_IDropTarget_DragEnter(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSDropTarget, IDropTarget_iface)) as *mut CFSDropTarget;
    CFSDropTarget_DragEnter_body(this, pDataObj, grfKeyState, pt, pdwEffect)
}
unsafe extern "system" fn CFSDropTarget_IDropTarget_DragOver(This: *mut IDropTarget, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSDropTarget, IDropTarget_iface)) as *mut CFSDropTarget;
    CFSDropTarget_DragOver_body(this, grfKeyState, pt, pdwEffect)
}
unsafe extern "system" fn CFSDropTarget_IDropTarget_DragLeave(This: *mut IDropTarget) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSDropTarget, IDropTarget_iface)) as *mut CFSDropTarget;
    CFSDropTarget_DragLeave_body(this)
}
unsafe extern "system" fn CFSDropTarget_IDropTarget_Drop(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSDropTarget, IDropTarget_iface)) as *mut CFSDropTarget;
    CFSDropTarget_Drop_body(this, pDataObj, grfKeyState, pt, pdwEffect)
}

pub static CFSDROPTARGET_IDROPTARGET_VTBL: IDropTargetVtbl = IDropTargetVtbl {
    QueryInterface: CFSDropTarget_IDropTarget_QueryInterface,
    AddRef: CFSDropTarget_IDropTarget_AddRef,
    Release: CFSDropTarget_IDropTarget_Release,
    DragEnter: CFSDropTarget_IDropTarget_DragEnter,
    DragOver: CFSDropTarget_IDropTarget_DragOver,
    DragLeave: CFSDropTarget_IDropTarget_DragLeave,
    Drop: CFSDropTarget_IDropTarget_Drop,
};

unsafe extern "system" fn CFSDropTarget_IObjectWithSite_QueryInterface(This: *mut IObjectWithSite, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSDropTarget, IObjectWithSite_iface)) as *mut CFSDropTarget;
    CFSDropTarget_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFSDropTarget_IObjectWithSite_AddRef(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSDropTarget, IObjectWithSite_iface)) as *mut CFSDropTarget;
    CFSDropTarget_AddRef(this)
}
unsafe extern "system" fn CFSDropTarget_IObjectWithSite_Release(This: *mut IObjectWithSite) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSDropTarget, IObjectWithSite_iface)) as *mut CFSDropTarget;
    CFSDropTarget_Release(this)
}
unsafe extern "system" fn CFSDropTarget_IObjectWithSite_SetSite(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSDropTarget, IObjectWithSite_iface)) as *mut CFSDropTarget;
    CFSDropTarget_SetSite_body(this, pUnkSite)
}
unsafe extern "system" fn CFSDropTarget_IObjectWithSite_GetSite(This: *mut IObjectWithSite, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFSDropTarget, IObjectWithSite_iface)) as *mut CFSDropTarget;
    CFSDropTarget_GetSite_body(this, riid, ppvSite)
}

pub static CFSDROPTARGET_IOBJECTWITHSITE_VTBL: IObjectWithSiteVtbl = IObjectWithSiteVtbl {
    QueryInterface: CFSDropTarget_IObjectWithSite_QueryInterface,
    AddRef: CFSDropTarget_IObjectWithSite_AddRef,
    Release: CFSDropTarget_IObjectWithSite_Release,
    SetSite: CFSDropTarget_IObjectWithSite_SetSite,
    GetSite: CFSDropTarget_IObjectWithSite_GetSite,
};

unsafe fn CFSDropTarget_QueryInterface(
    this: *mut CFSDropTarget, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IDropTarget)) {
        *ppv = core::ptr::addr_of_mut!((*this).IDropTarget_iface) as *mut core::ffi::c_void;
    }
    else if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IObjectWithSite)) {
        *ppv = core::ptr::addr_of_mut!((*this).IObjectWithSite_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CFSDropTarget_AddRef(this);
    S_OK
}

unsafe fn CFSDropTarget_AddRef(this: *mut CFSDropTarget) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CFSDropTarget_Release(this: *mut CFSDropTarget) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CFSDropTarget_dtor_body(this);
        CFSDropTarget_cpp_com_assign_m_site(this, core::ptr::null_mut());
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
pub unsafe extern "C" fn CFSDropTarget_CreateInstance(
    PathTarget: LPWSTR,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CFSDropTarget>()) as *mut CFSDropTarget;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CFSDropTarget {
        // `.cast_mut()` because shellcom spells IDropTarget's slot `*mut`, not `*const`.
        // The vtable itself stays immutable; only the pointer's constness is adjusted.
        IDropTarget_iface: IDropTarget { lpVtbl: core::ptr::addr_of!(CFSDROPTARGET_IDROPTARGET_VTBL).cast_mut() },
        IObjectWithSite_iface: IObjectWithSite { lpVtbl: core::ptr::addr_of!(CFSDROPTARGET_IOBJECTWITHSITE_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        m_cfShellIDList: core::mem::zeroed(),
        m_fAcceptFmt: core::mem::zeroed(),
        m_sPathTarget: core::mem::zeroed(),
        m_hwndSite: core::mem::zeroed(),
        m_grfKeyState: core::mem::zeroed(),
        m_dwDefaultEffect: core::mem::zeroed(),
        m_AllowedEffects: core::mem::zeroed(),
        m_site: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CFSDropTarget_ctor_body(this);
    let mut hr = CFSDropTarget_Initialize_body(this, PathTarget);
    if hr >= 0 {
        hr = CFSDropTarget_QueryInterface(this, riid, ppv);
    }
    CFSDropTarget_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDM_COPYHERE: UINT = 0x7;

pub const IDM_DRAGFILE: UINT = 0xce;

pub const IDM_LINKHERE: UINT = 0xB;

pub const IDM_MOVEHERE: UINT = 0x8;

pub const IDS_LNK_FILE: UINT = 215;

pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

pub const SFGAO_CANCOPY: SFGAOF = 0x0000_0001;

pub const SFGAO_CANLINK: SFGAOF = 0x0000_0004;

pub const CF_HDROP: DWORD = 15;

pub const CMIC_MASK_CONTROL_DOWN: DWORD = 0x40000000;

pub const CMIC_MASK_SHIFT_DOWN: DWORD = 0x10000000;

pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;

pub const FOF_ALLOWUNDO: DWORD = 0x0040;

pub const FOF_NOCONFIRMMKDIR: DWORD = 0x0200;

pub const FOF_RENAMEONCOLLISION: DWORD = 0x0008;

pub const FO_COPY: DWORD = 2;

pub const FO_MOVE: DWORD = 1;

pub const MF_BYCOMMAND: DWORD = 0;

pub const MK_CONTROL: DWORD = 8;

pub const MK_RBUTTON: DWORD = 2;

pub const MK_SHIFT: DWORD = 4;

pub const SFGAO_STREAM: DWORD = 0x00400000;

pub const SHCNE_CREATE: DWORD = 0x00000002;

pub const SHCNF_PATHW: DWORD = 0x0005;

pub const SHGDN_FOREDITING: DWORD = 0x1000;

pub const SHGDN_INFOLDER: DWORD = 1;

pub const SIGDN_FILESYSPATH: DWORD = 0x80058000;

pub const SS_LEFT: DWORD = 0;

pub const SVSI_SELECT: DWORD = 0x00000001;

pub const SW_SHOW: DWORD = 5;

pub const TPM_LEFTALIGN: DWORD = 0;

pub const TPM_LEFTBUTTON: DWORD = 0;

pub const TPM_NONOTIFY: DWORD = 128;

pub const TPM_RETURNCMD: DWORD = 256;

pub const TPM_RIGHTBUTTON: DWORD = 2;

pub const TYMED_HGLOBAL: DWORD = 1;

pub const WS_BORDER: DWORD = 0x800000;

pub const WS_CLIPSIBLINGS: DWORD = 0x4000000;

pub const WS_DISABLED: DWORD = 0x8000000;

pub const WS_OVERLAPPED: DWORD = 0;

#[repr(C)]
pub struct BIND_OPTS {
    pub cbStruct: DWORD,
    pub grfFlags: DWORD,
    pub grfMode: DWORD,
    pub dwTickCountDeadline: DWORD,
}

#[repr(C)]
pub struct CIDA {
    pub cidl: UINT,
    pub aoffset: [UINT; 1],
}
pub type LPIDA = *mut CIDA;

pub use crate::island::DROPFILES;
pub type LPDROPFILES = *mut DROPFILES;

#[repr(C)]
pub struct DVTARGETDEVICE {
    pub tdSize: DWORD,
    pub tdDriverNameOffset: WORD,
    pub tdDeviceNameOffset: WORD,
    pub tdPortNameOffset: WORD,
    pub tdExtDevmodeOffset: WORD,
    pub tdData: [BYTE; 1],
}

pub use crate::island::QCMINFO;

pub use crate::island::QCMINFO_IDMAP;

pub use crate::island::QCMINFO_IDMAP_PLACEMENT;

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

// Deterministic from objidl.h
#[repr(C)]
pub struct IPersistFile {
    pub lpVtbl: *const IPersistFileVtbl,
}
#[repr(C)]
pub struct IPersistFileVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IPersistFile, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IPersistFile) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IPersistFile) -> ULONG,
    pub GetClassID: unsafe extern "system" fn(This: *mut IPersistFile, pClassID: *mut CLSID) -> HRESULT,
    pub IsDirty: unsafe extern "system" fn(This: *mut IPersistFile) -> HRESULT,
    pub Load: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR, dwMode: DWORD) -> HRESULT,
    pub Save: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR, fRemember: BOOL) -> HRESULT,
    pub SaveCompleted: unsafe extern "system" fn(This: *mut IPersistFile, pszFileName: LPCOLESTR) -> HRESULT,
    pub GetCurFile: unsafe extern "system" fn(This: *mut IPersistFile, ppszFileName: *mut LPOLESTR) -> HRESULT,
}
extern "C" {
    pub static IID_IPersistFile: IID;
}

// Deterministic from shobjidl.h
#[repr(C)]
pub struct IShellLinkW {
    pub lpVtbl: *const IShellLinkWVtbl,
}
#[repr(C)]
pub struct IShellLinkWVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IShellLinkW, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IShellLinkW) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IShellLinkW) -> ULONG,
    pub GetPath: unsafe extern "system" fn(This: *mut IShellLinkW, pszFile: LPWSTR, cch: i32, pfd: *mut WIN32_FIND_DATAW, fFlags: DWORD) -> HRESULT,
    pub GetIDList: unsafe extern "system" fn(This: *mut IShellLinkW, ppidl: *mut PIDLIST_ABSOLUTE) -> HRESULT,
    pub SetIDList: unsafe extern "system" fn(This: *mut IShellLinkW, pidl: PCIDLIST_ABSOLUTE) -> HRESULT,
    pub GetDescription: unsafe extern "system" fn(This: *mut IShellLinkW, pszName: LPWSTR, cch: i32) -> HRESULT,
    pub SetDescription: unsafe extern "system" fn(This: *mut IShellLinkW, pszName: LPCWSTR) -> HRESULT,
    pub GetWorkingDirectory: unsafe extern "system" fn(This: *mut IShellLinkW, pszDir: LPWSTR, cch: i32) -> HRESULT,
    pub SetWorkingDirectory: unsafe extern "system" fn(This: *mut IShellLinkW, pszDir: LPCWSTR) -> HRESULT,
    pub GetArguments: unsafe extern "system" fn(This: *mut IShellLinkW, pszArgs: LPWSTR, cch: i32) -> HRESULT,
    pub SetArguments: unsafe extern "system" fn(This: *mut IShellLinkW, pszArgs: LPCWSTR) -> HRESULT,
    pub GetHotkey: unsafe extern "system" fn(This: *mut IShellLinkW, pwHotkey: *mut WORD) -> HRESULT,
    pub SetHotkey: unsafe extern "system" fn(This: *mut IShellLinkW, wHotkey: WORD) -> HRESULT,
    pub GetShowCmd: unsafe extern "system" fn(This: *mut IShellLinkW, piShowCmd: *mut i32) -> HRESULT,
    pub SetShowCmd: unsafe extern "system" fn(This: *mut IShellLinkW, iShowCmd: i32) -> HRESULT,
    pub GetIconLocation: unsafe extern "system" fn(This: *mut IShellLinkW, pszIconPath: LPWSTR, cch: i32, piIcon: *mut i32) -> HRESULT,
    pub SetIconLocation: unsafe extern "system" fn(This: *mut IShellLinkW, pszIconPath: LPCWSTR, iIcon: i32) -> HRESULT,
    pub SetRelativePath: unsafe extern "system" fn(This: *mut IShellLinkW, pszPathRel: LPCWSTR, dwReserved: DWORD) -> HRESULT,
    pub Resolve: unsafe extern "system" fn(This: *mut IShellLinkW, hwnd: HWND, fFlags: DWORD) -> HRESULT,
    pub SetPath: unsafe extern "system" fn(This: *mut IShellLinkW, pszFile: LPCWSTR) -> HRESULT,
}
extern "C" {
    pub static IID_IShellLinkW: IID;
}

pub use crate::island::WIN32_FIND_DATAW;

pub static INVALID_FILETITLE_CHARACTERSW: [WCHAR; 10] = [0x005c, 0x002f, 0x003a, 0x002a, 0x003f, 0x0022, 0x003c, 0x003e, 0x007c, 0];

pub type ITEMIDLIST_ABSOLUTE = ITEMIDLIST;

pub const DROPEFFECT_NONE: DWORD = 0;
pub const DROPEFFECT_COPY: DWORD = 1;
pub const DROPEFFECT_MOVE: DWORD = 2;
pub const DROPEFFECT_LINK: DWORD = 4;

extern "system" {
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn GlobalLock(memory: HGLOBAL) -> LPVOID;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn SHStrDupW(source: LPCWSTR, out: *mut LPWSTR) -> HRESULT;
    fn SHGetNameFromIDList(pidl: PCIDLIST_ABSOLUTE, sigdn: SIGDN, name: *mut PWSTR) -> HRESULT;
    // Redeclared rather than taken from the island, which spells the thread proc as bindgen's
    // per-target concrete ABI (`extern "stdcall"` on i386). `extern "system"` IS that same
    // calling convention on every target we build, but rustc keeps the two SPELLINGS as
    // distinct fn types, so an `extern "system"` fn item will not coerce into the island's
    // slot. Same shape as dialogs/filedefext.rs:1025. Both `pfnCallback` args are NULL at
    // every call site here, so LPVOID is enough.
}

extern "C" {
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
    static IID_IFolderView: GUID;
    static IID_IShellFolder: GUID;
    static IID_IShellFolderView: GUID;
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

#[inline]
unsafe fn StringCchCopyW(dst: LPWSTR, capacity: usize, src: LPCWSTR) -> HRESULT {
    if dst.is_null() || src.is_null() || capacity == 0 { return E_INVALIDARG; }
    let len = lstrlenW(src) as usize;
    if len >= capacity { *dst = 0; return E_FAIL; }
    let bytes = (len + 1) * core::mem::size_of::<WCHAR>();
    let _ = memcpy(dst, src, bytes);
    S_OK
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
pub static CPP_COM_WSTR_03360AA7B5C4: [WCHAR; 12] = [0x0025, 0x0073, 0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0075, 0x0029, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_0E3E6B83CD96: [WCHAR; 17] = [0x0044, 0x0072, 0x0061, 0x0067, 0x0044, 0x0072, 0x006F, 0x0070, 0x0048, 0x0061, 0x006E, 0x0064, 0x006C, 0x0065, 0x0072, 0x0073, 0x0000];
pub static CPP_COM_WSTR_46251A0ACA27: [WCHAR; 10] = [0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0075, 0x0029, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_69D435449EDE: [WCHAR; 7] = [0x0025, 0x0073, 0x0025, 0x0073, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_E2CD87D16D16: [WCHAR; 5] = [0x0025, 0x0073, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_FE706C396A0C: [WCHAR; 5] = [0x002E, 0x006C, 0x006E, 0x006B, 0x0000];

use core::ffi::c_void;

pub unsafe fn SHELL_StripIllegalFsNameCharacters_body(Buf: LPWSTR) -> () {
    let mut src = Buf;
    let mut dst = src;
    loop {
        *dst = *src;
        if *dst == 0 {
            break;
        } else if wcschr(INVALID_FILETITLE_CHARACTERSW.as_ptr(), *dst) != core::ptr::null_mut() {
            src = CharNextW(src);
        } else {
            src = src.offset(1);
            dst = dst.offset(1);
        }
    }
}

pub unsafe fn PathIsSameDrive_body(Path1: LPCWSTR, Path2: LPCWSTR) -> bool {
    let d1 = PathGetDriveNumberW(Path1);
    let d2 = PathGetDriveNumberW(Path2);
    d1 == d2 && d2 >= 0
}

pub unsafe fn PathIsDriveRoot_body(Path: LPCWSTR) -> bool {
    return PathIsRootW(Path) != 0 && PathGetDriveNumberW(Path) >= 0;
}

pub unsafe fn SHELL_LimitDropEffectToItemAttributes_body(pDataObject: *mut IDataObject, pdwEffect: PDWORD) -> HRESULT {
    let att = (*pdwEffect) & (SFGAO_CANCOPY | SFGAO_CANMOVE | SFGAO_CANLINK);
    let mut att_out = att;
    let hr = SHGetAttributesFromDataObject(pDataObject, att, &mut att_out, core::ptr::null_mut());
    if hr < 0 {
        return S_FALSE;
    }
    *pdwEffect &= !(SFGAO_CANCOPY | SFGAO_CANMOVE | SFGAO_CANLINK) | att_out;
    hr
}

pub unsafe fn GetDefaultCopyMoveEffect_body() -> () {}

pub unsafe fn CFSDropTarget__CopyItems_body(this: *mut CFSDropTarget, pSFFrom: *mut IShellFolder, cidl: UINT, apidl: *mut LPCITEMIDLIST, bCopy: BOOL) -> HRESULT {
    let mut ret: HRESULT;
    let mut wszDstPath: [WCHAR; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
    let pwszSrcPathsList: PWSTR = HeapAlloc(GetProcessHeap(), 0, MAX_PATH as usize * core::mem::size_of::<WCHAR>() * cidl as usize + 1) as PWSTR;
    if pwszSrcPathsList.is_null() {
        return E_OUTOFMEMORY;
    }

    let mut pwszListPos: PWSTR = pwszSrcPathsList;
    let mut strretFrom: STRRET = core::mem::zeroed();
    let mut fop: SHFILEOPSTRUCTW = core::mem::zeroed();
    let mut bRenameOnCollision: BOOL = FALSE;

    for i in 0..cidl {
        // shellcom owns the vtable ABI and declares its own PIDL/STRRET layouts (see the note
        // at shellcom.rs:91), so island-typed locals cross the boundary with a `.cast()`.
        ret = ((*(*pSFFrom).lpVtbl).GetDisplayNameOf)(pSFFrom, (*apidl.add(i as usize)).cast(), SHGDN_FORPARSING, core::ptr::addr_of_mut!(strretFrom).cast());
        if ret < 0 {
            HeapFree(GetProcessHeap(), 0, pwszSrcPathsList as *mut _);
            return ret;
        }

        ret = StrRetToBufW(&mut strretFrom, core::ptr::null_mut(), pwszListPos, MAX_PATH);
        if ret < 0 {
            HeapFree(GetProcessHeap(), 0, pwszSrcPathsList as *mut _);
            return ret;
        }

        pwszListPos = pwszListPos.add(lstrlenW(pwszListPos) as usize + 1);
    }

    *pwszListPos = 0x0000u16;

    ret = StringCchCopyW(wszDstPath.as_mut_ptr(), MAX_PATH as usize, (*this).m_sPathTarget);
    if ret < 0 {
        HeapFree(GetProcessHeap(), 0, pwszSrcPathsList as *mut _);
        return ret;
    }

    wszDstPath[lstrlenW(wszDstPath.as_ptr()) as usize + 1] = UNICODE_NULL;

    if bCopy != 0 {
        let mut szPath1: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut szPath2: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        GetFullPathNameW(pwszSrcPathsList, szPath1.len() as DWORD, szPath1.as_mut_ptr(), core::ptr::null_mut());
        GetFullPathNameW(wszDstPath.as_ptr(), szPath2.len() as DWORD, szPath2.as_mut_ptr(), core::ptr::null_mut());
        PathRemoveFileSpecW(szPath1.as_mut_ptr());
        if _wcsicmp(szPath1.as_ptr(), szPath2.as_ptr()) == 0 {
            bRenameOnCollision = TRUE;
        }
    }

    ZeroMemory(&mut fop as *mut _ as *mut _, core::mem::size_of_val(&fop));
    fop.hwnd = (*this).m_hwndSite;
    fop.wFunc = if bCopy != 0 { FO_COPY } else { FO_MOVE };
    fop.pFrom = pwszSrcPathsList;
    fop.pTo = wszDstPath.as_ptr();
    fop.fFlags = (FOF_ALLOWUNDO | FOF_NOCONFIRMMKDIR) as FILEOP_FLAGS;
    if bRenameOnCollision != 0 {
        fop.fFlags |= FOF_RENAMEONCOLLISION as FILEOP_FLAGS;
    }

    ret = S_OK;

    if SHFileOperationW(&mut fop) != 0 {
        ret = E_FAIL;
    }

    HeapFree(GetProcessHeap(), 0, pwszSrcPathsList as *mut _);
    ret
}

pub unsafe fn CFSDropTarget_ctor_body(this: *mut CFSDropTarget) -> () {}

pub unsafe fn CFSDropTarget_Initialize_body(this: *mut CFSDropTarget, PathTarget: LPWSTR) -> HRESULT {
    if PathTarget.is_null() {
        return E_UNEXPECTED;
    }

    (*this).m_cfShellIDList = RegisterClipboardFormatW(CFSTR_SHELLIDLISTW.as_ptr());
    if (*this).m_cfShellIDList == 0 {
        return E_FAIL;
    }

    SHStrDupW(PathTarget, &mut (*this).m_sPathTarget)
}

pub unsafe fn CFSDropTarget_dtor_body(this: *mut CFSDropTarget) -> () {
    SHFree(((*this).m_sPathTarget) as LPVOID);
}

pub unsafe fn CFSDropTarget__GetUniqueFileName_body(this: *mut CFSDropTarget, pwszBasePath: LPCWSTR, pwszExt: LPCWSTR, pwszTarget: LPWSTR, bShortcut: BOOL) -> BOOL {
    let mut wszLink: [u16; 40] = [0; 40];

    if bShortcut == 0 {
        if LoadStringW(
            shell32_hInstance,
            IDS_LNK_FILE,
            wszLink.as_mut_ptr(),
            wszLink.len() as INT,
        ) == 0
        {
            wszLink[0] = 0x0000u16;
        }
    }

    if bShortcut == 0 {
        swprintf(
            pwszTarget,
            CPP_COM_WSTR_69D435449EDE.as_ptr(),
            wszLink.as_ptr(),
            pwszBasePath,
            pwszExt,
        );
    } else {
        swprintf(
            pwszTarget,
            CPP_COM_WSTR_E2CD87D16D16.as_ptr(),
            pwszBasePath,
            pwszExt,
        );
    }

    let mut i: u32 = 2;
    while PathFileExistsW(pwszTarget) != 0 {
        if bShortcut == 0 {
            swprintf(
                pwszTarget,
                CPP_COM_WSTR_03360AA7B5C4.as_ptr(),
                wszLink.as_ptr(),
                pwszBasePath,
                i,
                pwszExt,
            );
        } else {
            swprintf(
                pwszTarget,
                CPP_COM_WSTR_46251A0ACA27.as_ptr(),
                pwszBasePath,
                i,
                pwszExt,
            );
        }
        i = i.wrapping_add(1);
    }

    TRUE
}

pub unsafe fn CFSDropTarget__QueryDrop_body(this: *mut CFSDropTarget, dwKeyState: DWORD, pdwEffect: LPDWORD) -> BOOL {
    GetDefaultCopyMoveEffect_body();

    let dwEffect = (*this).m_dwDefaultEffect;

    *pdwEffect = DROPEFFECT_NONE;

    if (*this).m_fAcceptFmt != 0 {
        *pdwEffect = KeyStateToDropEffect(dwKeyState);

        if (*pdwEffect & D_MOVE != 0)
            && ((*this).m_AllowedEffects & (D_MOVE | D_COPY)
                == D_COPY)
        {
            *pdwEffect = D_COPY;
        }

        *pdwEffect &= (*this).m_AllowedEffects;

        if *pdwEffect == DROPEFFECT_NONE {
            *pdwEffect = dwEffect;
        }

        if dwEffect & *pdwEffect != 0 {
            return TRUE;
        }
    }
    FALSE
}

pub unsafe fn CFSDropTarget__GetEffectFromMenu_body(this: *mut CFSDropTarget, pDataObject: *mut IDataObject, pt: POINTL, pdwEffect: *mut DWORD, dwAvailableEffects: DWORD) -> HRESULT {
    let mut dwAvailableEffects = dwAvailableEffects;
    let hmenu = LoadMenuW(shell32_hInstance, MAKEINTRESOURCEW(IDM_DRAGFILE));
    if hmenu.is_null() {
        return E_OUTOFMEMORY;
    }
    let hpopupmenu = GetSubMenu(hmenu, 0);

    SHELL_LimitDropEffectToItemAttributes_body(pDataObject, &mut dwAvailableEffects);
    if (dwAvailableEffects & DROPEFFECT_COPY) == 0 {
        DeleteMenu(hpopupmenu, IDM_COPYHERE, MF_BYCOMMAND);
    }
    if (dwAvailableEffects & DROPEFFECT_MOVE) == 0 {
        DeleteMenu(hpopupmenu, IDM_MOVEHERE, MF_BYCOMMAND);
    }
    if (dwAvailableEffects & DROPEFFECT_LINK) == 0 && (dwAvailableEffects & (DROPEFFECT_COPY | DROPEFFECT_MOVE)) != 0 {
        DeleteMenu(hpopupmenu, IDM_LINKHERE, MF_BYCOMMAND);
    }

    GetDefaultCopyMoveEffect_body();
    if (*pdwEffect & dwAvailableEffects & DROPEFFECT_COPY) != 0 {
        SetMenuDefaultItem(hpopupmenu, IDM_COPYHERE as UINT, FALSE as UINT);
    } else if (*pdwEffect & dwAvailableEffects & DROPEFFECT_MOVE) != 0 {
        SetMenuDefaultItem(hpopupmenu, IDM_MOVEHERE as UINT, FALSE as UINT);
    } else if (dwAvailableEffects & DROPEFFECT_LINK) != 0 {
        SetMenuDefaultItem(hpopupmenu, IDM_LINKHERE as UINT, FALSE as UINT);
    }

    let mut keys = CRegKeyHandleArray::default();
    let hDCIA = DCIA_Create();
    let hDCMA = DCMA_Create();
    let pidlFolder = CComHeapPtr::new(SHELL32_CreateSimpleIDListFromPath((*this).m_sPathTarget, FILE_ATTRIBUTE_DIRECTORY));
    if !hDCMA.is_null() && !hDCIA.is_null() && !pidlFolder.as_raw().is_null() {
        // Two casts per call, both nominal-only: shfldr's HKEY is `*mut c_void` where the
        // island's is `*mut HKEY__`, and `SHELL32_CreateSimpleIDListFromPath` hands back
        // shellcom's ITEMIDLIST where shlfolder/the C externs want the island's.
        AddPidlClassKeysToArray(pidlFolder.as_raw().cast(), keys.hKeys.as_mut_ptr().cast(), &mut keys.cKeys);
        let mut i = 0;
        while i < keys.len() {
            DCIA_AddShellExSubkey(hDCIA, keys.hKeys[i].cast(), CPP_COM_WSTR_0E3E6B83CD96.as_ptr());
            i += 1;
        }

        // The C aggregate-initializes only four members; the trailing `pIdMap` is zero-filled
        // by the language, so spell that zero out rather than leaving it to look deliberate.
        let mut qcmi = QCMINFO {
            hmenu: hpopupmenu,
            indexMenu: 0,
            idCmdFirst: DROPIDM_EXTFIRST,
            idCmdLast: DROPIDM_EXTLAST,
            pIdMap: core::ptr::null(),
        };
        DCMA_InsertMenuItems(hDCMA, hDCIA, pidlFolder.as_raw().cast(), pDataObject, keys.hKeys.as_mut_ptr().cast(), keys.cKeys, &mut qcmi, 0, (*this).m_site);
    }

    let hwndDummy = CreateWindowExW(
        0,
        WC_STATIC_W.as_ptr(),
        core::ptr::null_mut(),
        WS_OVERLAPPED | WS_DISABLED | WS_CLIPSIBLINGS | WS_BORDER | SS_LEFT,
        pt.x,
        pt.y,
        1,
        1,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    SetForegroundWindow(hwndDummy);
    // `UINT uCommand` in the C (CFSDropTarget.cpp:439); the island types TrackPopupMenu's
    // return as BOOL, so pin the width HERE rather than at each of the six comparisons.
    let uCommand: UINT = TrackPopupMenu(
        hpopupmenu,
        TPM_LEFTALIGN | TPM_RETURNCMD | TPM_LEFTBUTTON | TPM_RIGHTBUTTON | TPM_NONOTIFY,
        pt.x,
        pt.y,
        0,
        hwndDummy,
        core::ptr::null_mut(),
    ) as UINT;

    DestroyWindow(hwndDummy);

    let mut hr = S_FALSE;
    // C_ASSERT is a COMPILE-TIME assertion, so it lowers to a const assert, not a call.
    const _: () = assert!(
        IDM_COPYHERE < DROPIDM_EXTFIRST
            && IDM_MOVEHERE < DROPIDM_EXTFIRST
            && IDM_LINKHERE < DROPIDM_EXTFIRST
            && DROPIDM_EXTFIRST > 0
    );
    if uCommand >= DROPIDM_EXTFIRST as UINT && uCommand <= DROPIDM_EXTLAST as UINT {
        // C aggregate init names four members and zero-fills the other five.
        let mut ici = CMINVOKECOMMANDINFO {
            cbSize: core::mem::size_of::<CMINVOKECOMMANDINFO>() as DWORD,
            fMask: 0,
            hwnd: (*this).m_hwndSite,
            lpVerb: MAKEINTRESOURCEA(uCommand - DROPIDM_EXTFIRST as UINT),
            ..core::mem::zeroed()
        };
        ici.nShow = SW_SHOW as INT;
        if ((*this).m_grfKeyState & MK_SHIFT) != 0 {
            ici.fMask |= CMIC_MASK_SHIFT_DOWN;
        }
        if ((*this).m_grfKeyState & MK_CONTROL) != 0 {
            ici.fMask |= CMIC_MASK_CONTROL_DOWN;
        }
        DCMA_InvokeCommand(hDCMA, &mut ici);
        hr = S_OK;
        *pdwEffect = DROPEFFECT_NONE;
    } else if uCommand == 0 {
        hr = S_OK;
        *pdwEffect = DROPEFFECT_NONE;
    } else if uCommand == IDM_COPYHERE as UINT {
        *pdwEffect = DROPEFFECT_COPY;
    } else if uCommand == IDM_MOVEHERE as UINT {
        *pdwEffect = DROPEFFECT_MOVE;
    } else if uCommand == IDM_LINKHERE as UINT {
        *pdwEffect = DROPEFFECT_LINK;
    } else {
        hr = E_UNEXPECTED;
    }

    DCMA_Destroy(hDCMA);
    DCIA_Destroy(hDCIA);
    DestroyMenu(hmenu);
    hr
}

pub unsafe fn CFSDropTarget__RepositionItems_body(this: *mut CFSDropTarget, psfv: *mut IShellFolderView, pdtobj: *mut IDataObject, pt: POINTL) -> HRESULT {
    let mut pfv: *mut IFolderView = core::ptr::null_mut();
    let _pfv_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pfv));
    let mut ptDrag: POINT = core::mem::zeroed();
    let hr = ((*(*psfv).lpVtbl).QueryInterface)(
        psfv,
        core::ptr::addr_of!(IID_IFolderView),
        core::ptr::addr_of_mut!(pfv).cast(),
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        if !pfv.is_null() {
            ((*(*pfv).lpVtbl).Release)(pfv);
            pfv = core::ptr::null_mut();
        }
        return hr;
    }

    let hr = ((*(*psfv).lpVtbl).GetDragPoint)(psfv, &mut ptDrag);
    if FAILED_UNEXPECTEDLY!(hr) {
        ((*(*pfv).lpVtbl).Release)(pfv);
        pfv = core::ptr::null_mut();
        return hr;
    }

    let mut pidlFolder: PIDLIST_ABSOLUTE = core::ptr::null_mut();
    let mut apidl: *mut LPITEMIDLIST = core::ptr::null_mut();
    let mut cidl: u32 = 0;
    let hr = SH_GetApidlFromDataObject(pdtobj, &mut pidlFolder, &mut apidl, &mut cidl);
    if FAILED_UNEXPECTEDLY!(hr) {
        ((*(*pfv).lpVtbl).Release)(pfv);
        pfv = core::ptr::null_mut();
        return hr;
    }

    let apt = CoTaskMemAlloc(
        ((cidl as usize) * core::mem::size_of::<POINT>()) as SIZE_T
    ) as *mut POINT;
    if apt.is_null() {
        SHFree(pidlFolder);
        _ILFreeaPidl(apidl.cast(), cidl);
        ((*(*pfv).lpVtbl).Release)(pfv);
        pfv = core::ptr::null_mut();
        return E_OUTOFMEMORY;
    }

    // `.cast()` on every PIDL and POINT crossing into shellcom's vtable: the locals are
    // island-typed (prelude), the slots are spelled in shellcom's own layout-identical
    // copies. Layout matches; only the nominal type differs.
    for i in 0..cidl {
        ((*(*pfv).lpVtbl).GetItemPosition)(pfv, (*apidl.offset(i as isize)).cast(), apt.offset(i as isize).cast());
        (*apt.offset(i as isize)).x += pt.x - ptDrag.x;
        (*apt.offset(i as isize)).y += pt.y - ptDrag.y;
    }

    ((*(*pfv).lpVtbl).SelectAndPositionItems)(pfv, cidl, apidl.cast(), apt.cast(), SVSI_SELECT);

    CoTaskMemFree(apt as *mut core::ffi::c_void);
    SHFree(pidlFolder);
    _ILFreeaPidl(apidl.cast(), cidl);
    ((*(*pfv).lpVtbl).Release)(pfv);
    pfv = core::ptr::null_mut();
    S_OK
}

pub unsafe fn CFSDropTarget_DragEnter_body(this: *mut CFSDropTarget, pDataObject: *mut IDataObject, dwKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    let bAnyKeyMod = dwKeyState & (MK_SHIFT | MK_CONTROL);
    (*this).m_AllowedEffects = *pdwEffect;

    if *pdwEffect == DROPEFFECT_NONE {
        return S_OK;
    }

    let mut fmt: FORMATETC = core::mem::zeroed();
    let mut fmt2: FORMATETC = core::mem::zeroed();
    (*this).m_fAcceptFmt = FALSE;

    InitFormatEtc(&mut fmt, (*this).m_cfShellIDList, TYMED_HGLOBAL);
    InitFormatEtc(&mut fmt2, CF_HDROP, TYMED_HGLOBAL);

    if ((*(*pDataObject).lpVtbl).QueryGetData)(pDataObject, core::ptr::addr_of_mut!(fmt).cast()) >= 0 {
        (*this).m_fAcceptFmt = TRUE;
    } else if ((*(*pDataObject).lpVtbl).QueryGetData)(pDataObject, core::ptr::addr_of_mut!(fmt2).cast()) >= 0 {
        (*this).m_fAcceptFmt = TRUE;
    }

    (*this).m_grfKeyState = dwKeyState;

    SHELL_LimitDropEffectToItemAttributes_body(pDataObject, pdwEffect);
    (*this).m_AllowedEffects = *pdwEffect;
    (*this).m_dwDefaultEffect = (*this).m_AllowedEffects;
    match *pdwEffect & (D_COPY | D_MOVE | D_LINK) {
        x if x == (D_COPY | D_MOVE) => {
            if dwKeyState & MK_CONTROL != 0 {
                (*this).m_dwDefaultEffect = D_COPY;
            } else {
                (*this).m_dwDefaultEffect = D_MOVE;
            }
        }
        x if x == (D_COPY | D_MOVE | D_LINK) => {
            if (dwKeyState & (MK_SHIFT | MK_CONTROL)) == (MK_SHIFT | MK_CONTROL) {
                (*this).m_dwDefaultEffect = D_LINK;
            } else if (dwKeyState & (MK_SHIFT | MK_CONTROL)) == MK_CONTROL {
                (*this).m_dwDefaultEffect = D_COPY;
            } else {
                (*this).m_dwDefaultEffect = D_MOVE;
            }
        }
        x if x == (D_COPY | D_LINK) => {
            if (dwKeyState & (MK_SHIFT | MK_CONTROL)) == (MK_SHIFT | MK_CONTROL) {
                (*this).m_dwDefaultEffect = D_LINK;
            } else {
                (*this).m_dwDefaultEffect = D_COPY;
            }
        }
        x if x == (D_MOVE | D_LINK) => {
            if (dwKeyState & (MK_SHIFT | MK_CONTROL)) == (MK_SHIFT | MK_CONTROL) {
                (*this).m_dwDefaultEffect = D_LINK;
            } else {
                (*this).m_dwDefaultEffect = D_MOVE;
            }
        }
        _ => {}
    }

    let mut medium: STGMEDIUM = core::mem::zeroed();
    if ((*(*pDataObject).lpVtbl).GetData)(pDataObject, core::ptr::addr_of_mut!(fmt2).cast(), core::ptr::addr_of_mut!(medium).cast()) >= 0 {
        let mut wstrFirstFile: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
        if DragQueryFileW(medium.__bindgen_anon_1.hGlobal.cast(), 0, wstrFirstFile.as_mut_ptr(), wstrFirstFile.len() as UINT) != 0 {
            if !PathIsSameDrive_body(wstrFirstFile.as_ptr(), (*this).m_sPathTarget)
                && (*this).m_dwDefaultEffect != D_LINK
            {
                (*this).m_dwDefaultEffect = DROPEFFECT_COPY;
            }

            if bAnyKeyMod == 0 && PathIsDriveRoot_body(wstrFirstFile.as_ptr()) && ((*this).m_AllowedEffects & DROPEFFECT_LINK) != 0 {
                (*this).m_dwDefaultEffect = DROPEFFECT_LINK;
            }
        }
        ReleaseStgMedium(&mut medium);
    }

    if (*this).m_fAcceptFmt == 0 {
        (*this).m_AllowedEffects = DROPEFFECT_NONE;
    } else {
        *pdwEffect = (*this).m_dwDefaultEffect;
    }
    *pdwEffect &= (*this).m_AllowedEffects;

    S_OK
}

pub unsafe fn CFSDropTarget_DragOver_body(this: *mut CFSDropTarget, dwKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    if pdwEffect.is_null() {
        return E_INVALIDARG;
    }

    (*this).m_grfKeyState = dwKeyState;

    CFSDropTarget__QueryDrop_body(this, dwKeyState, pdwEffect);

    S_OK
}

pub unsafe fn CFSDropTarget_DragLeave_body(this: *mut CFSDropTarget) -> HRESULT {
    (*this).m_fAcceptFmt = FALSE;

    return S_OK;
}

pub unsafe fn CFSDropTarget_Drop_body(this: *mut CFSDropTarget, pDataObject: *mut IDataObject, dwKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT {
    if pdwEffect.is_null() {
        return E_INVALIDARG;
    }

    IUnknown_GetWindow((*this).m_site.cast(), &mut (*this).m_hwndSite);

    let dwAvailableEffects = *pdwEffect;

    CFSDropTarget__QueryDrop_body(this, dwKeyState, pdwEffect);

    if (*this).m_grfKeyState & MK_RBUTTON != 0 {
        let hr = CFSDropTarget__GetEffectFromMenu_body(this, pDataObject, pt, pdwEffect, dwAvailableEffects);
        if ((hr) < 0) || hr == S_OK {
            return hr;
        }
    }

    if *pdwEffect == DROPEFFECT_MOVE && !(*this).m_site.is_null() {
        let mut psfv: *mut IShellFolderView = core::ptr::null_mut();
        let _psfv_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfv));
        // `SID_SFolderView` is `#define`d to IID_IFolderView (shobjidl.idl:904) — a REFGUID,
        // so it goes in by address, not by value.
        let hr = IUnknown_QueryService(((*this).m_site).cast(), core::ptr::addr_of!(IID_IFolderView), core::ptr::addr_of!(IID_IShellFolderView), core::ptr::addr_of_mut!(psfv).cast());
        if hr >= 0 {
            // C++ passes `this` and lets the compiler upcast CFSDropTarget* -> IDropTarget*.
            // Rust has no implicit upcast, so name the embedded interface explicitly; the
            // offset is what the C++ adjustor thunk would have added.
            let is_drop_result = ((*(*psfv).lpVtbl).IsDropOnSource)(psfv, core::ptr::addr_of_mut!((*this).IDropTarget_iface));
            if is_drop_result == S_OK {
                CFSDropTarget__RepositionItems_body(this, psfv, pDataObject, pt);
                if !psfv.is_null() {
                    ((*(*psfv).lpVtbl).Release)(psfv);
                    psfv = core::ptr::null_mut();
                }
                return S_OK;
            }
        }
        if !psfv.is_null() {
            ((*(*psfv).lpVtbl).Release)(psfv);
            psfv = core::ptr::null_mut();
        }
    }

    let mut fIsOpAsync = FALSE;
    let mut pAsyncOperation: *mut IAsyncOperation = core::ptr::null_mut();
    let _pAsyncOperation_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pAsyncOperation));

    let qi_result = ((*(*pDataObject).lpVtbl).QueryInterface)(pDataObject, core::ptr::addr_of!(IID_IAsyncOperation), core::ptr::addr_of_mut!(pAsyncOperation).cast());
    if qi_result >= 0 {
        let async_mode_hr = ((*(*pAsyncOperation).lpVtbl).GetAsyncMode)(pAsyncOperation, &mut fIsOpAsync);
        if async_mode_hr >= 0 && fIsOpAsync != 0 {
            let data = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<_DoDropData>()) as *mut _DoDropData;
            (*data).This = this;
            ((*(*pDataObject).lpVtbl).AddRef)(pDataObject);
            ((*(*pAsyncOperation).lpVtbl).StartOperation)(pAsyncOperation, core::ptr::null_mut());
            CoMarshalInterThreadInterfaceInStream(core::ptr::addr_of!(IID_IDataObject).cast(), pDataObject.cast(), core::ptr::addr_of_mut!((*data).pStream).cast());
            // `this->AddRef()` in the C++ is the CLASS's AddRef, not a vtable dispatch —
            // CFSDropTarget has no `lpVtbl` of its own, its two embedded interfaces do.
            CFSDropTarget_AddRef(this);
            (*data).dwKeyState = dwKeyState;
            (*data).pt = pt;
            (*data).pdwEffect = *pdwEffect;
            SHCreateThread(Some(CFSDropTarget_DoDropThreadProc_entry), data as *mut core::ffi::c_void, CTF_COINIT | CTF_PROCESS_REF, None);
            ((*(*pAsyncOperation).lpVtbl).Release)(pAsyncOperation);
            pAsyncOperation = core::ptr::null_mut();
            return S_OK;
        }
        ((*(*pAsyncOperation).lpVtbl).Release)(pAsyncOperation);
        pAsyncOperation = core::ptr::null_mut();
    }

    CFSDropTarget__DoDrop_body(this, pDataObject, dwKeyState, pt, pdwEffect)
}

pub unsafe fn CFSDropTarget_SetSite_body(this: *mut CFSDropTarget, pUnkSite: *mut IUnknown) -> HRESULT {
    CFSDropTarget_cpp_com_assign_m_site(this, pUnkSite);
    return S_OK;
}

pub unsafe fn CFSDropTarget_GetSite_body(this: *mut CFSDropTarget, riid: REFIID, ppvSite: *mut PVOID) -> HRESULT {
    if (*this).m_site.is_null() {
        return E_FAIL;
    }

    ((*(*(*this).m_site).lpVtbl).QueryInterface)((*this).m_site, riid, ppvSite)
}

pub unsafe fn CFSDropTarget__DoDropThreadProc_body(this: *mut CFSDropTarget, lpParameter: LPVOID) -> DWORD {
    let data = lpParameter as *mut _DoDropData;
    let mut pDataObject: *mut IDataObject = core::ptr::null_mut();
    let _pDataObject_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pDataObject));
    let hr = CoGetInterfaceAndReleaseStream((*data).pStream.cast(), core::ptr::addr_of!(IID_IDataObject).cast(), core::ptr::addr_of_mut!(pDataObject).cast());

    if hr >= 0 {
        let mut pAsyncOperation: *mut IAsyncOperation = core::ptr::null_mut();
        let _pAsyncOperation_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pAsyncOperation));
        // `_DoDropThreadProc` is a STATIC member — the C has no `this`, and the receiver is
        // `data->This` (`data->This->_DoDrop(...)`). The scaffold gives every method body a
        // `this` parameter anyway, and the assembled call passed BOTH it and `data->This`,
        // which is one argument too many and the wrong receiver.
        let hr = CFSDropTarget__DoDrop_body((*data).This, pDataObject, (*data).dwKeyState, (*data).pt, &mut (*data).pdwEffect);
        if ((*(*pDataObject).lpVtbl).QueryInterface)(pDataObject, core::ptr::addr_of!(IID_IAsyncOperation), core::ptr::addr_of_mut!(pAsyncOperation).cast()) >= 0 {
            ((*(*pAsyncOperation).lpVtbl).EndOperation)(pAsyncOperation, hr, core::ptr::null_mut(), (*data).pdwEffect);
            ((*(*pAsyncOperation).lpVtbl).Release)(pAsyncOperation);
            pAsyncOperation = core::ptr::null_mut();
        }
        ((*(*pDataObject).lpVtbl).Release)(pDataObject);
        pDataObject = core::ptr::null_mut();
    }

    // `data->This->Release()` — again the class's own Release, not a vtable slot.
    CFSDropTarget_Release((*data).This);
    HeapFree(GetProcessHeap(), 0, data as LPVOID);
    0
}

// ============================================================================
// HAND-WRITTEN: CFSDropTarget::_DoDrop  (CFSDropTarget.cpp:561-787)
//
// Not a translation left in C and not a gap: `translate` was paid for this body and
// REFUSED it twice, both times on the fact/call envelope, so no paid artifact was
// accepted. The stored replies are in .responses/ (20260805-005459 and -005919) and the
// run was reassembled with `--skip-fn CFSDropTarget__DoDrop`, which records the omission
// as an audit finding rather than letting it go quiet. This body is transliterated from
// the C by hand instead.
//
// The reason the model could not land it is the same one behind `--local-raii`: the C
// nests four C++ RAII locals (CComPtr psfDesktop/psfFrom/ppf/pLink and
// CComHeapPtr<ITEMIDLIST_ABSOLUTE> pidlFull) across early returns and a `break`, and one
// of them — pidlFull — exists ONLY as a template argument, so its sealed fact token has
// no place to land in Rust and the envelope reports it missing. The lowering below uses
// the file's own CppComPtrGuard/CComHeapPtr, which is what the accepted _RepositionItems
// body uses; the rejected replies reached for `scopeguard::` instead.
//
// Three faithfulness points that are easy to get wrong here, all preserved deliberately:
//   * `psfFrom = psfDesktop` is a CComPtr ASSIGNMENT, which AddRefs. Both smart pointers
//     then release at scope end. Dropping the AddRef would over-release the desktop
//     folder — a refcount bug that only shows up under a drop onto the desktop.
//   * The CF_HDROP arm `return hr`s from inside the `if`, so it never reaches this
//     function's ReleaseStgMedium, and the `!lpdf` path returns without one either. That
//     leaks the medium. It is what the C does; a translation is not the place to fix it.
//   * The C never GlobalUnlocks what it GlobalLocks. Same rule.
// ============================================================================

extern "system" {
    fn ReleaseStgMedium(pmedium: *mut STGMEDIUM);
    fn DisplayNameOfW(
        psf: *mut IShellFolder,
        pidl: LPCITEMIDLIST,
        dwFlags: DWORD,
        pszBuf: LPWSTR,
        cchBuf: UINT,
    ) -> HRESULT;
    fn PathCombineW(lpszDest: LPWSTR, lpszDir: LPCWSTR, lpszFile: LPCWSTR) -> LPWSTR;
    fn PathFindExtensionW(pszPath: LPCWSTR) -> LPWSTR;
    fn PathFindFileNameW(pszPath: LPCWSTR) -> LPWSTR;
    fn IShellLink_ConstructFromPath(path: LPCWSTR, riid: REFIID, ppv: *mut LPVOID) -> HRESULT;
}

extern "C" {
    fn wcscpy(dest: LPWSTR, src: LPCWSTR) -> LPWSTR;
    fn wcslen(s: LPCWSTR) -> usize;
}

pub unsafe fn CFSDropTarget__DoDrop_body(
    this: *mut CFSDropTarget,
    pDataObject: *mut IDataObject,
    dwKeyState: DWORD,
    pt: POINTL,
    pdwEffect: *mut DWORD,
) -> HRESULT {
    let _ = (dwKeyState, pt);
    let mut fmt: FORMATETC = core::mem::zeroed();
    let mut fmt2: FORMATETC = core::mem::zeroed();
    let mut medium: STGMEDIUM = core::mem::zeroed();

    InitFormatEtc(&mut fmt, (*this).m_cfShellIDList, TYMED_HGLOBAL);
    InitFormatEtc(&mut fmt2, CF_HDROP, TYMED_HGLOBAL);

    let mut hr: HRESULT;
    let mut bCopy: BOOL = TRUE;
    let mut bLinking: BOOL = FALSE;

    /* Figure out what drop operation we're doing */
    if !pdwEffect.is_null() {
        if (*pdwEffect & DROPEFFECT_MOVE) == DROPEFFECT_MOVE {
            bCopy = FALSE;
        }
        if (*pdwEffect & DROPEFFECT_LINK) == DROPEFFECT_LINK {
            bLinking = TRUE;
        }
    }

    hr = ((*(*pDataObject).lpVtbl).QueryGetData)(pDataObject, core::ptr::addr_of_mut!(fmt).cast());
    if hr >= 0 {
        // The C only ERR()s on failure here and keeps going; medium stays zeroed and the
        // GlobalLock below is what actually turns it into E_FAIL.
        hr = ((*(*pDataObject).lpVtbl).GetData)(pDataObject, core::ptr::addr_of_mut!(fmt).cast(), core::ptr::addr_of_mut!(medium).cast());

        /* lock the handle */
        let lpcida = GlobalLock(medium.__bindgen_anon_1.hGlobal) as *mut CIDA;
        if lpcida.is_null() {
            ReleaseStgMedium(&mut medium);
            return E_FAIL;
        }

        /* convert the data into pidl */
        let mut pidl: LPITEMIDLIST = core::ptr::null_mut();
        let apidl = _ILCopyCidaToaPidl(core::ptr::addr_of_mut!(pidl).cast(), lpcida.cast());
        if apidl.is_null() {
            ReleaseStgMedium(&mut medium);
            return E_FAIL;
        }

        let mut psfDesktop: *mut IShellFolder = core::ptr::null_mut();
        let _psfDesktop_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfDesktop));
        let mut psfFrom: *mut IShellFolder = core::ptr::null_mut();
        let _psfFrom_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(psfFrom));

        /* Grab the desktop shell folder */
        hr = SHGetDesktopFolder(&mut psfDesktop);
        if hr < 0 {
            SHFree(pidl);
            _ILFreeaPidl(apidl, (*lpcida).cidl);
            ReleaseStgMedium(&mut medium);
            return E_FAIL;
        }

        /* Find source folder, this is where the clipboard data was copied from */
        if _ILIsDesktop(pidl) != 0 {
            /* use desktop shell folder */
            psfFrom = psfDesktop;
            // CComPtr assignment AddRefs; both guards Release.
            ((*(*psfFrom).lpVtbl).AddRef)(psfFrom);
        } else {
            hr = ((*(*psfDesktop).lpVtbl).BindToObject)(
                psfDesktop,
                pidl.cast(),
                core::ptr::null_mut(),
                core::ptr::addr_of!(IID_IShellFolder),
                core::ptr::addr_of_mut!(psfFrom).cast(),
            );
            if hr < 0 {
                SHFree(pidl);
                _ILFreeaPidl(apidl, (*lpcida).cidl);
                ReleaseStgMedium(&mut medium);
                return E_FAIL;
            }
        }

        if bLinking != 0 {
            let mut wszNewLnk: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

            /* We need to create a link for each pidl in the copied items, so step
             * through the pidls from the clipboard */
            for i in 0..(*lpcida).cidl {
                let att: SFGAOF = SHGetAttributes(
                    psfFrom,
                    (*apidl.add(i as usize)).cast(),
                    SFGAO_FOLDER | SFGAO_STREAM | SFGAO_FILESYSTEM,
                );
                let mut pidlFull: CComHeapPtr<ITEMIDLIST> = CComHeapPtr::empty();
                hr = SHILCombine(pidl, (*apidl.add(i as usize)).cast(), pidlFull.as_mut_slot());

                let mut targetName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
                if hr >= 0 {
                    // If the target is a file, we use SHGDN_FORPARSING because "NeverShowExt"
                    // will hide the ".lnk" extension. If the target is a virtual item, we ask
                    // for the friendly name because SHGDN_FORPARSING will return a GUID.
                    let UseParsing: BOOL =
                        if (att & (SFGAO_FILESYSTEM | SFGAO_FOLDER)) == SFGAO_FILESYSTEM {
                            TRUE
                        } else {
                            FALSE
                        };
                    let ShgdnFor: DWORD = if UseParsing != 0 {
                        SHGDN_FORPARSING
                    } else {
                        SHGDN_FOREDITING
                    };
                    hr = DisplayNameOfW(
                        psfFrom,
                        (*apidl.add(i as usize)).cast(),
                        ShgdnFor | SHGDN_INFOLDER,
                        targetName.as_mut_ptr(),
                        targetName.len() as UINT,
                    );
                }
                if hr < 0 {
                    SHELL_ErrorBox((*this).m_hwndSite.cast(), hr as UINT);
                    break;
                }
                SHELL_StripIllegalFsNameCharacters_body(targetName.as_mut_ptr());

                let mut wszCombined: [WCHAR; MAX_PATH as usize + MAX_PATH as usize] = [0; MAX_PATH as usize + MAX_PATH as usize];
                PathCombineW(
                    wszCombined.as_mut_ptr(),
                    (*this).m_sPathTarget,
                    targetName.as_ptr(),
                );

                // Check to see if the source is a link
                let mut fSourceIsLink: BOOL = FALSE;
                if _wcsicmp(PathFindExtensionW(targetName.as_ptr()), CPP_COM_WSTR_FE706C396A0C.as_ptr()) == 0
                    && (att & (SFGAO_FOLDER | SFGAO_STREAM)) != SFGAO_FOLDER
                {
                    fSourceIsLink = TRUE;
                    PathRemoveExtensionW(wszCombined.as_mut_ptr());
                }

                // Create a pathname to save the new link.
                CFSDropTarget__GetUniqueFileName_body(
                    this,
                    wszCombined.as_ptr(),
                    CPP_COM_WSTR_FE706C396A0C.as_ptr(),
                    wszNewLnk.as_mut_ptr(),
                    TRUE,
                );

                let mut ppf: *mut IPersistFile = core::ptr::null_mut();
                let _ppf_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(ppf));
                if fSourceIsLink != 0 {
                    let mut pwszTargetFull: PWSTR = core::ptr::null_mut();
                    hr = SHGetNameFromIDList(
                        pidlFull.as_raw(),
                        SIGDN_DESKTOPABSOLUTEPARSING,
                        &mut pwszTargetFull,
                    );
                    if hr >= 0 {
                        hr = IShellLink_ConstructFromPath(
                            pwszTargetFull,
                            core::ptr::addr_of!(IID_IPersistFile),
                            core::ptr::addr_of_mut!(ppf).cast(),
                        );
                        SHFree(pwszTargetFull);
                    }
                } else {
                    let mut pLink: *mut IShellLinkW = core::ptr::null_mut();
                    let _pLink_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(pLink));
                    hr = CShellLink_CreateInstance(
                        core::ptr::null_mut(),
                        core::ptr::addr_of!(IID_IShellLinkW),
                        core::ptr::addr_of_mut!(pLink).cast(),
                    );
                    if hr >= 0 {
                        hr = ((*(*pLink).lpVtbl).SetIDList)(pLink, pidlFull.as_raw());
                        if hr >= 0 {
                            hr = ((*(*pLink).lpVtbl).QueryInterface)(
                                pLink,
                                core::ptr::addr_of!(IID_IPersistFile),
                                core::ptr::addr_of_mut!(ppf).cast(),
                            );
                        }

                        let mut pwszPath: PWSTR = core::ptr::null_mut();
                        if (att & SFGAO_FILESYSTEM) != 0
                            && SHGetNameFromIDList(
                                pidlFull.as_raw(),
                                SIGDN_FILESYSPATH,
                                &mut pwszPath,
                            ) >= 0
                        {
                            let pSep = PathFindFileNameW(pwszPath);
                            if pSep > pwszPath {
                                *pSep.offset(-1) = UNICODE_NULL;
                                ((*(*pLink).lpVtbl).SetWorkingDirectory)(pLink, pwszPath);
                            }
                            SHFree(pwszPath);
                        }
                    }
                }
                if hr >= 0 {
                    // `!fSourceIsLink` — C logical NOT on a BOOL, not a bitwise one.
                    let fRemember: BOOL = if fSourceIsLink != 0 { FALSE } else { TRUE };
                    hr = ((*(*ppf).lpVtbl).Save)(ppf, wszNewLnk.as_ptr(), fRemember);
                }
                if hr < 0 {
                    SHELL_ErrorBox((*this).m_hwndSite.cast(), hr as UINT);
                    break;
                }
                SHChangeNotify(
                    SHCNE_CREATE as LONG,
                    SHCNF_PATHW,
                    wszNewLnk.as_ptr() as LPCVOID,
                    core::ptr::null(),
                );
            }
        } else {
            hr = CFSDropTarget__CopyItems_body(
                this,
                psfFrom,
                (*lpcida).cidl,
                apidl as *mut LPCITEMIDLIST,
                bCopy,
            );
        }

        SHFree(pidl);
        _ILFreeaPidl(apidl, (*lpcida).cidl);
        ReleaseStgMedium(&mut medium);
    } else {
        hr = ((*(*pDataObject).lpVtbl).QueryGetData)(pDataObject, core::ptr::addr_of_mut!(fmt2).cast());
        if hr >= 0 {
            // The C re-declares and re-inits a local `fmt2` here, shadowing the outer one.
            let mut fmt2_inner: FORMATETC = core::mem::zeroed();
            InitFormatEtc(&mut fmt2_inner, CF_HDROP, TYMED_HGLOBAL);
            hr = ((*(*pDataObject).lpVtbl).GetData)(pDataObject, core::ptr::addr_of_mut!(fmt2_inner).cast(), core::ptr::addr_of_mut!(medium).cast());
            if hr >= 0 {
                let mut wszTargetPath: [WCHAR; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];

                wcscpy(wszTargetPath.as_mut_ptr(), (*this).m_sPathTarget);
                //Double NULL terminate.
                wszTargetPath[wcslen(wszTargetPath.as_ptr()) + 1] = 0;

                let lpdf = GlobalLock(medium.__bindgen_anon_1.hGlobal) as LPDROPFILES;
                if lpdf.is_null() {
                    return E_FAIL;
                }
                let pszSrcList =
                    ((lpdf as *mut u8).add((*lpdf).pFiles as usize)) as LPWSTR;

                let mut op: SHFILEOPSTRUCTW = core::mem::zeroed();
                ZeroMemory(&mut op as *mut _ as *mut u8, core::mem::size_of_val(&op));
                op.pFrom = pszSrcList;
                op.pTo = wszTargetPath.as_ptr();
                op.hwnd = (*this).m_hwndSite;
                op.wFunc = if bCopy != 0 { FO_COPY } else { FO_MOVE };
                op.fFlags = (FOF_ALLOWUNDO | FOF_NOCONFIRMMKDIR) as FILEOP_FLAGS;
                let res = SHFileOperationW(&mut op);
                if res != 0 {
                    hr = E_FAIL;
                }

                // The C returns from inside here, so the medium is never released.
                return hr;
            }
            hr = E_FAIL;
        } else {
            hr = E_FAIL;
        }
    }
    hr
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
//
// /*
//  * file system folder drop target
//  *
//  * Copyright 1997             Marcus Meissner
//  * Copyright 1998, 1999, 2002 Juergen Schmied
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
// #include <precomp.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL (shell);
//
// /* CShellLink is Rust-owned (shell32-rust shell_link); every construction must go
//  * through the Rust factory so no C++ CShellLink can coexist with the Rust one. */
// extern "C" HRESULT WINAPI CShellLink_CreateInstance(
//     IUnknown *pUnkOuter, REFIID riid, LPVOID *ppv);
//
// #define D_NONE DROPEFFECT_NONE
// #define D_COPY DROPEFFECT_COPY
// #define D_MOVE DROPEFFECT_MOVE
// #define D_LINK DROPEFFECT_LINK
//
// static void SHELL_StripIllegalFsNameCharacters( LPWSTR Buf)
// {
//     for (LPWSTR src = Buf, dst = src;;)
//     {
//         *dst = *src;
//         if (!*dst)
//             break;
//         else if (wcschr(INVALID_FILETITLE_CHARACTERSW, *dst))
//             src = CharNextW(src);
//         else
//             ++src, ++dst;
//     }
// }
// static bool PathIsSameDrive(LPCWSTR Path1, LPCWSTR Path2)
// {
//     int d1 = PathGetDriveNumberW(Path1), d2 = PathGetDriveNumberW(Path2);
//     return d1 == d2 && d2 >= 0;
// }
// static bool PathIsDriveRoot(LPCWSTR Path)
// {
//     return PathIsRootW(Path) != 0 && PathGetDriveNumberW(Path) >= 0;
// }
// static HRESULT
// SHELL_LimitDropEffectToItemAttributes( IDataObject *pDataObject,  PDWORD pdwEffect)
// {
//     DWORD att = *pdwEffect & (SFGAO_CANCOPY | SFGAO_CANMOVE | SFGAO_CANLINK); // DROPEFFECT maps perfectly to these SFGAO bits
//     HRESULT hr = SHGetAttributesFromDataObject(pDataObject, att, &att, NULL);
//     if (FAILED(hr))
//         return S_FALSE;
//     *pdwEffect &= ~(SFGAO_CANCOPY | SFGAO_CANMOVE | SFGAO_CANLINK) | att;
//     return hr;
// }
// static void GetDefaultCopyMoveEffect()
// {
//     // FIXME: When the source is on a different volume than the target, change default from move to copy
// }
// HRESULT CFSDropTarget::_CopyItems(IShellFolder * pSFFrom, UINT cidl,
//                                   LPCITEMIDLIST * apidl, BOOL bCopy)
// {
//     HRESULT ret;
//     WCHAR wszDstPath[MAX_PATH + 1] = {0};
//     PWCHAR pwszSrcPathsList = (PWCHAR) HeapAlloc(GetProcessHeap(), 0, MAX_PATH * sizeof(WCHAR) * cidl + 1);
//     if (!pwszSrcPathsList)
//         return E_OUTOFMEMORY;
//
//     PWCHAR pwszListPos = pwszSrcPathsList;
//     STRRET strretFrom;
//     SHFILEOPSTRUCTW fop;
//     BOOL bRenameOnCollision = FALSE;
//
//     /* Build a double null terminated list of C strings from source paths */
//     for (UINT i = 0; i < cidl; i++)
//     {
//         ret = pSFFrom->GetDisplayNameOf(apidl[i], SHGDN_FORPARSING, &strretFrom);
//         if (FAILED(ret))
//             goto cleanup;
//
//         ret = StrRetToBufW(&strretFrom, NULL, pwszListPos, MAX_PATH);
//         if (FAILED(ret))
//             goto cleanup;
//
//         pwszListPos += lstrlenW(pwszListPos) + 1;
//     }
//     /* Append the final null. */
//     *pwszListPos = L'\0';
//
//     /* Build a double null terminated target (this path) */
//     ret = StringCchCopyW(wszDstPath, MAX_PATH, m_sPathTarget);
//     if (FAILED(ret))
//         goto cleanup;
//
//     wszDstPath[lstrlenW(wszDstPath) + 1] = UNICODE_NULL;
//
//     /* Set bRenameOnCollision to TRUE if necesssary */
//     if (bCopy)
//     {
//         WCHAR szPath1[MAX_PATH], szPath2[MAX_PATH];
//         GetFullPathNameW(pwszSrcPathsList, _countof(szPath1), szPath1, NULL);
//         GetFullPathNameW(wszDstPath, _countof(szPath2), szPath2, NULL);
//         PathRemoveFileSpecW(szPath1);
//         if (_wcsicmp(szPath1, szPath2) == 0)
//             bRenameOnCollision = TRUE;
//     }
//
//     ZeroMemory(&fop, sizeof(fop));
//     fop.hwnd = m_hwndSite;
//     fop.wFunc = bCopy ? FO_COPY : FO_MOVE;
//     fop.pFrom = pwszSrcPathsList;
//     fop.pTo = wszDstPath;
//     fop.fFlags = (FOF_ALLOWUNDO | FOF_NOCONFIRMMKDIR) as FILEOP_FLAGS;
//     if (bRenameOnCollision)
//         fop.fFlags |= FOF_RENAMEONCOLLISION as FILEOP_FLAGS;
//
//     ret = S_OK;
//
//     if (SHFileOperationW(&fop))
//     {
//         ERR("SHFileOperationW failed\n");
//         ret = E_FAIL;
//     }
//
// cleanup:
//     HeapFree(GetProcessHeap(), 0, pwszSrcPathsList);
//     return ret;
// }
// HRESULT CFSDropTarget::Initialize(LPWSTR PathTarget)
// {
//     if (!PathTarget)
//         return E_UNEXPECTED;
//
//     m_cfShellIDList = RegisterClipboardFormatW(CFSTR_SHELLIDLISTW.as_ptr());
//     if (!m_cfShellIDList)
//         return E_FAIL;
//
//     return SHStrDupW(PathTarget, &m_sPathTarget);
// }
// BOOL
// CFSDropTarget::_GetUniqueFileName(LPCWSTR pwszBasePath, LPCWSTR pwszExt, LPWSTR pwszTarget, BOOL bShortcut)
// {
//     WCHAR wszLink[40];
//
//     if (!bShortcut)
//     {
//         if (!LoadStringW(shell32_hInstance, IDS_LNK_FILE, wszLink, _countof(wszLink)))
//             wszLink[0] = L'\0';
//     }
//
//     if (!bShortcut)
//         swprintf(pwszTarget, L"%s%s%s", wszLink, pwszBasePath, pwszExt);
//     else
//         swprintf(pwszTarget, L"%s%s", pwszBasePath, pwszExt);
//
//     for (UINT i = 2; PathFileExistsW(pwszTarget); ++i)
//     {
//         if (!bShortcut)
//             swprintf(pwszTarget, L"%s%s (%u)%s", wszLink, pwszBasePath, i, pwszExt);
//         else
//             swprintf(pwszTarget, L"%s (%u)%s", pwszBasePath, i, pwszExt);
//     }
//
//     return TRUE;
// }
// BOOL CFSDropTarget::_QueryDrop(DWORD dwKeyState, LPDWORD pdwEffect)
// {
//     /* TODO Windows does different drop effects if dragging across drives.
//     i.e., it will copy instead of move if the directories are on different disks. */
//     GetDefaultCopyMoveEffect();
//
//     DWORD dwEffect = m_dwDefaultEffect;
//
//     *pdwEffect = DROPEFFECT_NONE;
//
//     if (m_fAcceptFmt) { /* Does our interpretation of the keystate ... */
//         *pdwEffect = KeyStateToDropEffect(dwKeyState);
//
//         // Transform disallowed move to a copy
//         if ((*pdwEffect & D_MOVE) && (m_AllowedEffects & (D_MOVE | D_COPY)) == D_COPY)
//             *pdwEffect = D_COPY;
//
//         *pdwEffect &= m_AllowedEffects;
//
//         if (*pdwEffect == DROPEFFECT_NONE)
//             *pdwEffect = dwEffect;
//
//         /* ... matches the desired effect ? */
//         if (dwEffect & *pdwEffect) {
//             return TRUE;
//         }
//     }
//     return FALSE;
// }
// HRESULT CFSDropTarget::_GetEffectFromMenu(IDataObject *pDataObject, POINTL pt, DWORD *pdwEffect, DWORD dwAvailableEffects)
// {
//     HMENU hmenu = LoadMenuW(shell32_hInstance, MAKEINTRESOURCEW(IDM_DRAGFILE));
//     if (!hmenu)
//         return E_OUTOFMEMORY;
//     HMENU hpopupmenu = GetSubMenu(hmenu, 0);
//
//     SHELL_LimitDropEffectToItemAttributes(pDataObject, &dwAvailableEffects);
//     if ((dwAvailableEffects & DROPEFFECT_COPY) == 0)
//         DeleteMenu(hpopupmenu, IDM_COPYHERE, MF_BYCOMMAND);
//     if ((dwAvailableEffects & DROPEFFECT_MOVE) == 0)
//         DeleteMenu(hpopupmenu, IDM_MOVEHERE, MF_BYCOMMAND);
//     if ((dwAvailableEffects & DROPEFFECT_LINK) == 0 && (dwAvailableEffects & (DROPEFFECT_COPY | DROPEFFECT_MOVE)))
//         DeleteMenu(hpopupmenu, IDM_LINKHERE, MF_BYCOMMAND);
//
//     GetDefaultCopyMoveEffect();
//     if (*pdwEffect & dwAvailableEffects & DROPEFFECT_COPY)
//         SetMenuDefaultItem(hpopupmenu, IDM_COPYHERE as UINT, FALSE);
//     else if (*pdwEffect & dwAvailableEffects & DROPEFFECT_MOVE)
//         SetMenuDefaultItem(hpopupmenu, IDM_MOVEHERE as UINT, FALSE);
//     else if (dwAvailableEffects & DROPEFFECT_LINK)
//         SetMenuDefaultItem(hpopupmenu, IDM_LINKHERE as UINT, FALSE);
//
//     CRegKeyHandleArray keys;
//     HDCIA hDCIA = DCIA_Create();
//     HDCMA hDCMA = DCMA_Create();
//     CComHeapPtr<ITEMIDLIST_ABSOLUTE> pidlFolder(SHELL32_CreateSimpleIDListFromPath(m_sPathTarget, FILE_ATTRIBUTE_DIRECTORY));
//     if (hDCMA && hDCIA && pidlFolder)
//     {
//         AddPidlClassKeysToArray(pidlFolder, keys, keys);
//         for (UINT i = 0; i < keys; ++i)
//             DCIA_AddShellExSubkey(hDCIA, keys.hKeys[i], L"DragDropHandlers");
//
//         QCMINFO qcmi = { hpopupmenu, 0, DROPIDM_EXTFIRST, DROPIDM_EXTLAST };
//         DCMA_InsertMenuItems(hDCMA, hDCIA, pidlFolder, pDataObject, keys, keys, &qcmi, 0, m_site);
//     }
//
//     /* We shouldn't use the site window here because the menu should work even when we don't have a site */
//     HWND hwndDummy = CreateWindowEx(0,
//                               WC_STATIC_W.as_ptr(),
//                               NULL,
//                               WS_OVERLAPPED | WS_DISABLED | WS_CLIPSIBLINGS | WS_BORDER | SS_LEFT,
//                               pt.x,
//                               pt.y,
//                               1,
//                               1,
//                               NULL,
//                               NULL,
//                               NULL,
//                               NULL);
//
//     SetForegroundWindow(hwndDummy); // Required for aborting by pressing Esc when dragging from Explorer to desktop
//     UINT uCommand = TrackPopupMenu(hpopupmenu,
//                                    TPM_LEFTALIGN | TPM_RETURNCMD | TPM_LEFTBUTTON | TPM_RIGHTBUTTON | TPM_NONOTIFY,
//                                    pt.x, pt.y, 0, hwndDummy, NULL);
//
//     DestroyWindow(hwndDummy);
//
//     HRESULT hr = S_FALSE; // S_FALSE means we did not handle the command
//     C_ASSERT(IDM_COPYHERE < DROPIDM_EXTFIRST && IDM_MOVEHERE < DROPIDM_EXTFIRST &&
//              IDM_LINKHERE < DROPIDM_EXTFIRST && DROPIDM_EXTFIRST > 0);
//     if (uCommand >= DROPIDM_EXTFIRST && uCommand <= DROPIDM_EXTLAST)
//     {
//         CMINVOKECOMMANDINFO ici = { sizeof(ici), 0, m_hwndSite, MAKEINTRESOURCEA(uCommand - DROPIDM_EXTFIRST) };
//         ici.nShow = SW_SHOW as INT;
//         if (m_grfKeyState & MK_SHIFT)
//             ici.fMask |= CMIC_MASK_SHIFT_DOWN;
//         if (m_grfKeyState & MK_CONTROL)
//             ici.fMask |= CMIC_MASK_CONTROL_DOWN;
//         DCMA_InvokeCommand(hDCMA, &ici);
//         hr = S_OK;
//         *pdwEffect = DROPEFFECT_NONE;
//     }
//     else if (uCommand == 0)
//     {
//         hr = S_OK;
//         *pdwEffect = DROPEFFECT_NONE;
//     }
//     else if (uCommand == IDM_COPYHERE)
//         *pdwEffect = DROPEFFECT_COPY;
//     else if (uCommand == IDM_MOVEHERE)
//         *pdwEffect = DROPEFFECT_MOVE;
//     else if (uCommand == IDM_LINKHERE)
//         *pdwEffect = DROPEFFECT_LINK;
//     else
//         hr = E_UNEXPECTED;
//
//     DCMA_Destroy(hDCMA);
//     DCIA_Destroy(hDCIA);
//     DestroyMenu(hmenu);
//     return hr;
// }
// HRESULT CFSDropTarget::_RepositionItems(IShellFolderView *psfv, IDataObject *pdtobj, POINTL pt)
// {
//     CComPtr<IFolderView> pfv;
//     POINT ptDrag;
//     HRESULT hr = psfv->QueryInterface(IID_PPV_ARG(IFolderView, &pfv));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     hr = psfv->GetDragPoint(&ptDrag);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     PIDLIST_ABSOLUTE pidlFolder;
//     PUITEMID_CHILD *apidl;
//     UINT cidl;
//     hr = SH_GetApidlFromDataObject(pdtobj, &pidlFolder, &apidl, &cidl);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     CComHeapPtr<POINT> apt;
//     if (!apt.Allocate(cidl))
//     {
//         SHFree(pidlFolder);
//         _ILFreeaPidl(apidl, cidl);
//         return E_OUTOFMEMORY;
//     }
//
//     for (UINT i = 0; i<cidl; i++)
//     {
//         pfv->GetItemPosition(apidl[i], &apt[i]);
//         apt[i].x += pt.x - ptDrag.x;
//         apt[i].y += pt.y - ptDrag.y;
//     }
//
//     pfv->SelectAndPositionItems(cidl, apidl, apt, SVSI_SELECT);
//
//     SHFree(pidlFolder);
//     _ILFreeaPidl(apidl, cidl);
//     return S_OK;
// }
// HRESULT WINAPI CFSDropTarget::DragEnter(IDataObject *pDataObject,
//                                         DWORD dwKeyState, POINTL pt, DWORD *pdwEffect)
// {
//     TRACE("(%p)->(DataObject=%p)\n", this, pDataObject);
//
//     const BOOL bAnyKeyMod = dwKeyState & (MK_SHIFT | MK_CONTROL);
//     m_AllowedEffects = *pdwEffect;
//
//     if (*pdwEffect == DROPEFFECT_NONE)
//         return S_OK;
//
//     FORMATETC fmt;
//     FORMATETC fmt2;
//     m_fAcceptFmt = FALSE;
//
//     InitFormatEtc (fmt, m_cfShellIDList, TYMED_HGLOBAL);
//     InitFormatEtc (fmt2, CF_HDROP, TYMED_HGLOBAL);
//
//     if (SUCCEEDED(pDataObject->QueryGetData(&fmt)))
//         m_fAcceptFmt = TRUE;
//     else if (SUCCEEDED(pDataObject->QueryGetData(&fmt2)))
//         m_fAcceptFmt = TRUE;
//
//     m_grfKeyState = dwKeyState;
//
//     SHELL_LimitDropEffectToItemAttributes(pDataObject, pdwEffect);
//     m_AllowedEffects = *pdwEffect;
//     m_dwDefaultEffect = m_AllowedEffects;
//     switch (*pdwEffect & (D_COPY | D_MOVE | D_LINK))
//     {
//         case D_COPY | D_MOVE:
//             if (dwKeyState & MK_CONTROL)
//                 m_dwDefaultEffect = D_COPY;
//             else
//                 m_dwDefaultEffect = D_MOVE;
//             break;
//         case D_COPY | D_MOVE | D_LINK:
//             if ((dwKeyState & (MK_SHIFT | MK_CONTROL)) == (MK_SHIFT | MK_CONTROL))
//                 m_dwDefaultEffect = D_LINK;
//             else if ((dwKeyState & (MK_SHIFT | MK_CONTROL)) == MK_CONTROL)
//                 m_dwDefaultEffect = D_COPY;
//             else
//                 m_dwDefaultEffect = D_MOVE;
//             break;
//         case D_COPY | D_LINK:
//             if ((dwKeyState & (MK_SHIFT | MK_CONTROL)) == (MK_SHIFT | MK_CONTROL))
//                 m_dwDefaultEffect = D_LINK;
//             else
//                 m_dwDefaultEffect = D_COPY;
//             break;
//         case D_MOVE | D_LINK:
//             if ((dwKeyState & (MK_SHIFT | MK_CONTROL)) == (MK_SHIFT | MK_CONTROL))
//                 m_dwDefaultEffect = D_LINK;
//             else
//                 m_dwDefaultEffect = D_MOVE;
//             break;
//     }
//
//     STGMEDIUM medium;
//     if (SUCCEEDED(pDataObject->GetData(&fmt2, &medium)))
//     {
//         WCHAR wstrFirstFile[MAX_PATH];
//         if (DragQueryFileW((HDROP)medium.__bindgen_anon_1.hGlobal, 0, wstrFirstFile, _countof(wstrFirstFile)))
//         {
//             if (!PathIsSameDrive(wstrFirstFile, m_sPathTarget) && m_dwDefaultEffect != D_LINK)
//             {
//                 m_dwDefaultEffect = DROPEFFECT_COPY;
//             }
//
//             if (!bAnyKeyMod && PathIsDriveRoot(wstrFirstFile) && (m_AllowedEffects & DROPEFFECT_LINK))
//             {
//                 m_dwDefaultEffect = DROPEFFECT_LINK; // Don't copy a drive by default
//             }
//         }
//         ReleaseStgMedium(&medium);
//     }
//
//     if (!m_fAcceptFmt)
//         m_AllowedEffects = DROPEFFECT_NONE;
//     else
//         *pdwEffect = m_dwDefaultEffect;
//     *pdwEffect &= m_AllowedEffects;
//
//     return S_OK;
// }
// HRESULT WINAPI CFSDropTarget::DragOver(DWORD dwKeyState, POINTL pt,
//                                        DWORD *pdwEffect)
// {
//     TRACE("(%p)\n", this);
//
//     if (!pdwEffect)
//         return E_INVALIDARG;
//
//     m_grfKeyState = dwKeyState;
//
//     _QueryDrop(dwKeyState, pdwEffect);
//
//     return S_OK;
// }
// HRESULT WINAPI CFSDropTarget::DragLeave()
// {
//     TRACE("(%p)\n", this);
//
//     m_fAcceptFmt = FALSE;
//
//     return S_OK;
// }
// HRESULT WINAPI CFSDropTarget::Drop(IDataObject *pDataObject,
//                                    DWORD dwKeyState, POINTL pt, DWORD *pdwEffect)
// {
//     TRACE("(%p) object dropped, effect %u\n", this, *pdwEffect);
//
//     if (!pdwEffect)
//         return E_INVALIDARG;
//
//     IUnknown_GetWindow(m_site, &m_hwndSite);
//
//     DWORD dwAvailableEffects = *pdwEffect;
//
//     _QueryDrop(dwKeyState, pdwEffect);
//
//     TRACE("pdwEffect: 0x%x, m_dwDefaultEffect: 0x%x, dwAvailableEffects: 0x%x\n", *pdwEffect, m_dwDefaultEffect, dwAvailableEffects);
//
//     if (m_grfKeyState & MK_RBUTTON)
//     {
//         HRESULT hr = _GetEffectFromMenu(pDataObject, pt, pdwEffect, dwAvailableEffects);
//         if (FAILED_UNEXPECTEDLY(hr) || hr == S_OK)
//             return hr;
//     }
//
//     if (*pdwEffect == DROPEFFECT_MOVE && m_site)
//     {
//         CComPtr<IShellFolderView> psfv;
//         HRESULT hr = IUnknown_QueryService(m_site, SID_SFolderView, IID_PPV_ARG(IShellFolderView, &psfv));
//         if (SUCCEEDED(hr) && psfv->IsDropOnSource(this) == S_OK)
//         {
//             _RepositionItems(psfv, pDataObject, pt);
//             return S_OK;
//         }
//     }
//
//     BOOL fIsOpAsync = FALSE;
//     CComPtr<IAsyncOperation> pAsyncOperation;
//
//     if (SUCCEEDED(pDataObject->QueryInterface(IID_PPV_ARG(IAsyncOperation, &pAsyncOperation))))
//     {
//         if (SUCCEEDED(pAsyncOperation->GetAsyncMode(&fIsOpAsync)) && fIsOpAsync)
//         {
//             _DoDropData *data = static_cast<_DoDropData*>(HeapAlloc(GetProcessHeap(), 0, sizeof(_DoDropData)));
//             data->This = this;
//             // Need to maintain this class in case the window is closed or the class exists temporarily (when dropping onto a folder).
//             pDataObject->AddRef();
//             pAsyncOperation->StartOperation(NULL);
//             CoMarshalInterThreadInterfaceInStream(IID_IDataObject, pDataObject, &data->pStream);
//             this->AddRef();
//             data->dwKeyState = dwKeyState;
//             data->pt = pt;
//             // Need to dereference as pdweffect gets freed.
//             data->pdwEffect = *pdwEffect;
//             SHCreateThread(CFSDropTarget::_DoDropThreadProc, data, CTF_COINIT | CTF_PROCESS_REF, NULL);
//             return S_OK;
//         }
//     }
//     return this->_DoDrop(pDataObject, dwKeyState, pt, pdwEffect);
// }
// HRESULT
// WINAPI
// CFSDropTarget::SetSite(IUnknown *pUnkSite)
// {
//     m_site = pUnkSite;
//     return S_OK;
// }
// HRESULT
// WINAPI
// CFSDropTarget::GetSite(REFIID riid, void **ppvSite)
// {
//     if (!m_site)
//         return E_FAIL;
//
//     return m_site->QueryInterface(riid, ppvSite);
// }
// DWORD WINAPI CFSDropTarget::_DoDropThreadProc(LPVOID lpParameter)
// {
//     _DoDropData *data = static_cast<_DoDropData*>(lpParameter);
//     CComPtr<IDataObject> pDataObject;
//     HRESULT hr = CoGetInterfaceAndReleaseStream (data->pStream, IID_PPV_ARG(IDataObject, &pDataObject));
//
//     if (SUCCEEDED(hr))
//     {
//         CComPtr<IAsyncOperation> pAsyncOperation;
//         hr = data->This->_DoDrop(pDataObject, data->dwKeyState, data->pt, &data->pdwEffect);
//         if (SUCCEEDED(pDataObject->QueryInterface(IID_PPV_ARG(IAsyncOperation, &pAsyncOperation))))
//         {
//             pAsyncOperation->EndOperation(hr, NULL, data->pdwEffect);
//         }
//     }
//     //Release the CFSFolder and data object holds in the copying thread.
//     data->This->Release();
//     //Release the parameter from the heap.
//     HeapFree(GetProcessHeap(), 0, data);
//     return 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// The C++ TU also exported a WINAPI trampoline so the (already Rust) CFSFolder could reach the
// factory across the C ABI. cfs_folder.rs still declares it inside an `extern "system"` block,
// so it must stay STDCALL: the factory above is `extern "C"`, and repointing the #[link_name]
// straight at it would silently mismatch the calling convention on i386 -- caller-cleans vs
// callee-cleans, which corrupts the stack rather than failing to link.
#[no_mangle]
pub unsafe extern "system" fn CFSDropTarget_CreateInstance_RustBridge(
    sPathTarget: LPWSTR,
    riid: REFIID,
    ppvOut: *mut LPVOID,
) -> HRESULT {
    CFSDropTarget_CreateInstance(sPathTarget, riid, ppvOut.cast())
}

// `_DoDropThreadProc` is a STATIC member used as a raw LPTHREAD_START_ROUTINE. The scaffold gave
// its body a `this` receiver that the C has no equivalent for, so this is the actual thread entry
// point SHCreateThread receives: `extern "system"` (WINAPI in the C), and it recovers the object
// from the parameter block exactly as the C does via `data->This`.
unsafe extern "system" fn CFSDropTarget_DoDropThreadProc_entry(lpParameter: LPVOID) -> DWORD {
    CFSDropTarget__DoDropThreadProc_body((*(lpParameter as *mut _DoDropData)).This, lpParameter)
}
