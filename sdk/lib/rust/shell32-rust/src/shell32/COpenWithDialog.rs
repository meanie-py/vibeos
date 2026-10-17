use crate::prelude::*;
// The crate's own functions: the definition is the declaration.
use crate::shell32::iconcache::ExtractIconExW;
use crate::wine::pidl::ILCombine;
use crate::wine::shell32_main::SHGetFileInfoW;
use crate::wine::pidl::SHGetPathFromIDListW;
use crate::shell32::shlexec::ShellExecuteExW;
// One declaration per external symbol: island.rs.
use crate::island::DestroyIcon;
use crate::island::DrawIconEx;
use crate::island::GetMenuItemInfoW;
use crate::island::GetSystemMetrics;
use crate::island::InsertMenuItemW;
use crate::island::LoadStringW;
use crate::island::RegCloseKey;
use crate::island::RegGetValueW;
use crate::island::RegOpenKeyExW;
use crate::island::RegQueryValueExW;
use crate::island::RegSetValueExW;
use crate::shell32::sdk_consts::*;
// `SApp` is a POD nested inside class COpenWithList; the generator only emits nested
// types reached by VALUE, and this one is reached through `SApp *m_pApp`.
use crate::open_with_menu_support::{
    NM_DBLCLK, NM_RETURN, TVN_SELCHANGED,
    CmicFlagsToSeeFlags, HResultFromWin32, PathFindFileNameW, SApp, SHELL_ErrorBox,
    StringCbCopyW, ERROR_SUCCESS, HRESULT_FROM_WIN32, REST_NOFILEASSOCIATE, DialogBoxParamW,
};
// COpenWithList is a SEPARATE module that owns the type. Each generated class file
// re-emits its member types, and two identical `#[repr(C)]` structs in two modules are
// still two unrelated Rust types — `m_pAppList` would not accept `COpenWithList_new()`.
use crate::open_with_list::*;
use crate::open_with_list::COpenWithList;

// Deterministic COM scaffold for COpenWithDialog; method bodies are generated separately.
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
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct COpenWithDialog {
    pub m_pInfo: *const OPENASINFO,
    pub m_pAppList: *mut COpenWithList,
    pub m_InFlags: UINT,
    pub m_bListAllocated: BOOL,
    pub m_hDialog: HWND,
    pub m_hTreeView: HWND,
    pub m_hRecommend: HTREEITEM,
    pub m_hOther: HTREEITEM,
    pub m_hImgList: HIMAGELIST,
    pub m_bNoOpen: BOOL,
}

/// `new COpenWithDialog` — operator new plus the constructor body.
pub unsafe fn COpenWithDialog_new(
    pInfo: *const OPENASINFO,
    pAppList: *mut COpenWithList,
) -> *mut COpenWithDialog {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<COpenWithDialog>(),
    ) as *mut COpenWithDialog;
    if this.is_null() { return core::ptr::null_mut(); }
    COpenWithDialog_ctor_body(this, pInfo, pAppList);
    this
}

/// `delete` — the destructor body, then the matching free. Pairs with
/// `COpenWithDialog_new`; freeing through any other allocator is a heap bug.
pub unsafe fn COpenWithDialog_delete(this: *mut COpenWithDialog) {
    if this.is_null() { return; }
    COpenWithDialog_dtor_body(this);
    HeapFree(GetProcessHeap(), 0, this);
}

// Deterministic C++ support closure selected from source identifiers.

pub const IDD_NOOPEN: UINT = 27;

pub const IDD_OPEN_WITH: UINT = 24;

pub const IDI_SHELL_PROGRAMS_FOLDER: UINT = 20;

pub const IDS_OPEN: UINT = 23;

pub const IDS_OPEN_WITH: UINT = 142;

pub const IDS_OPEN_WITH_CHOOSE: UINT = 143;

pub const IDS_OPEN_WITH_FILTER: UINT = 155;

pub const IDS_OPEN_WITH_OTHER: UINT = 179;

pub const IDS_OPEN_WITH_RECOMMENDED: UINT = 178;

pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

pub const SW_SHOWNORMAL: INT = 1;

pub const ASSOCSTR_COMMAND: DWORD = 1;

pub const BM_SETCHECK: DWORD = 241;

pub const BST_CHECKED: DWORD = 1;

pub const COLOR_MENU: DWORD = 4;

pub const DI_NORMAL: DWORD = 3;

pub const ERROR_FILE_NOT_FOUND: LSTATUS = 2;

pub const ERROR_INVALID_PARAMETER: LSTATUS = 87;

pub const ERROR_NO_ASSOCIATION: LSTATUS = 1155;

pub const IDCANCEL: DWORD = 2;

pub const IDNO: DWORD = 7;

pub const IDOK: DWORD = 1;

pub const IDYES: DWORD = 6;

pub const ILC_COLOR32: DWORD = 0x20;

pub const ILC_MASK: DWORD = 0x1;

pub const IMAGE_ICON: DWORD = 1;

pub const KEY_READ: DWORD = 0x20019;

pub const KEY_WRITE: DWORD = 0x20006;

pub const MFS_DEFAULT: DWORD = 4096;

pub const MFS_ENABLED: DWORD = 0;

pub const MFT_SEPARATOR: DWORD = 0x800;

pub const MIIM_CHECKMARKS: DWORD = 8;

pub const MIIM_DATA: DWORD = 32;

pub const MIIM_FTYPE: DWORD = 256;

pub const MIIM_STATE: DWORD = 1;

pub const MIIM_SUBMENU: DWORD = 4;

pub const MRU_STRING: DWORD = 0x0;

pub const OAIF_ALLOW_REGISTRATION: DWORD = 1;

pub const OAIF_EXEC: DWORD = 4;

pub const OAIF_FORCE_REGISTRATION: DWORD = 8;

pub const OAIF_HIDE_REGISTRATION: DWORD = 32;

pub const OAIF_REGISTER_EXT: DWORD = 2;

pub const OFN_EXPLORER: DWORD = 0x80000;

pub const OFN_FILEMUSTEXIST: DWORD = 0x1000;

pub const OFN_PATHMUSTEXIST: DWORD = 0x800;

pub const PRF_DONTFINDLNK: DWORD = 0x08;

pub const PRF_FIRSTDIRDEF: DWORD = 0x04;

pub const PRF_VERIFYEXISTS: DWORD = 0x01;

pub const REG_EXPAND_SZ: DWORD = 2;

pub const REG_SZ: DWORD = 1;

pub const SEE_MASK_CLASSKEY: DWORD = 0x00000003;

pub const SEE_MASK_NOCLOSEPROCESS: DWORD = 0x00000040;

pub const SEE_MASK_NOZONECHECKS: DWORD = 0x00800000;

pub const SHCNE_ASSOCCHANGED: DWORD = 0x08000000;

pub const SHCNF_FLUSHNOWAIT: DWORD = 0x3000;

pub const SHGFI_ICON: DWORD = 256;

pub const SHGFI_SHELLICONSIZE: DWORD = 4;

pub const SHGFI_SMALLICON: DWORD = 1;

pub const SM_CXMENUCHECK: DWORD = 71;

pub const SM_CYMENUCHECK: DWORD = 72;

pub const SW_HIDE: DWORD = 0;

pub const TVIF_IMAGE: DWORD = 0x2;

pub const TVIF_PARAM: DWORD = 0x4;

pub const TVIF_SELECTEDIMAGE: DWORD = 0x20;

pub const TVIF_STATE: DWORD = 0x8;

pub const TVIF_TEXT: DWORD = 0x1;

pub const TVIS_EXPANDED: DWORD = 0x20;

pub const TVSIL_NORMAL: DWORD = 0;

pub const WM_CLOSE: DWORD = 16;

pub const WM_COMMAND: DWORD = 273;

pub const WM_INITDIALOG: DWORD = 272;

macro_rules! ASSERT { ($($arg:tt)*) => {{}}; }

macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

#[inline]
const fn MAKEINTRESOURCE(value: UINT) -> LPCWSTR {
    value as usize as LPCWSTR
}

#[repr(C)]
pub struct SHELLEXECUTEINFOW {
    pub cbSize: DWORD, pub fMask: ULONG, pub hwnd: HWND,
    pub lpVerb: LPCWSTR, pub lpFile: LPCWSTR, pub lpParameters: LPCWSTR,
    pub lpDirectory: LPCWSTR, pub nShow: INT, pub hInstApp: HINSTANCE,
    pub lpIDList: LPVOID, pub lpClass: LPCWSTR, pub hkeyClass: HKEY,
    pub dwHotKey: DWORD, pub icon_or_monitor: HANDLE, pub hProcess: HANDLE,
}

#[repr(C)]
pub struct CMVERBMAP { pub Verb: LPCSTR, pub CmdId: WORD }
pub static mut g_VerbMap: [CMVERBMAP; 2] = [
    CMVERBMAP { Verb: b"openas\0".as_ptr() as LPCSTR, CmdId: 0 as WORD },
    CMVERBMAP { Verb: core::ptr::null(), CmdId: 0 as WORD },
];

extern "system" {
    #[link_name = "LoadImageW"] fn LoadImage(instance: HINSTANCE, name: LPCWSTR, kind: UINT, width: INT, height: INT, flags: UINT) -> HANDLE;
    #[link_name = "RegCreateKeyExW"] fn RegCreateKeyExW(key: HKEY, subkey: LPCWSTR, reserved: DWORD, class: LPWSTR, options: DWORD, access: REGSAM, security: LPVOID, result: PHKEY, disposition: LPDWORD) -> LONG;
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    #[link_name = "Shell32RustMapContextMenuVerbToCmdId"] fn SHELL_MapContextMenuVerbToCmdId(info: LPCMINVOKECOMMANDINFO, map: *const CMVERBMAP) -> INT;
    #[link_name = "Shell32RustGetCommandStringImpl"] fn SHELL_GetCommandStringImpl(command: SIZE_T, flags: UINT, buffer: LPSTR, capacity: UINT, map: *const CMVERBMAP) -> HRESULT;
}

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
    fn _wcsicmp(left: LPCWSTR, right: LPCWSTR) -> INT;
}

unsafe fn cpp_com_string_cb_printf_w(
    dest: LPWSTR, bytes: usize, format: LPCWSTR, args: &[LPCWSTR],
) -> HRESULT {
    // strsafe.h StringCbPrintfW, for the conversions translated call sites use: `%s` and `%%`.
    // Anything else is REFUSED (STRSAFE_E_INVALID_PARAMETER, empty result). This used to copy
    // an unknown conversion through literally and leave its argument unconsumed — and it did
    // not know `%%` either, so COpenWithList's `"%s" "%%1"` registered `"app.exe" "%%1"` as an
    // Open-With command line instead of `"app.exe" "%1"`, with nothing red anywhere.
    let capacity = bytes / core::mem::size_of::<WCHAR>();
    if dest.is_null() || format.is_null() || capacity == 0 { return E_FAIL; }
    let (mut out, mut input, mut arg_index) = (0usize, 0usize, 0usize);
    while *format.add(input) != 0 {
        let unit = *format.add(input);
        if unit == b'%' as WCHAR {
            let spec = *format.add(input + 1);
            if spec == b'%' as WCHAR {
                if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return 0x8007007Au32 as HRESULT; }
                *dest.add(out) = b'%' as WCHAR; out += 1; input += 2;
                continue;
            }
            if spec != b's' as WCHAR { *dest = 0; return 0x80070057u32 as HRESULT; }
            if arg_index >= args.len() { return E_FAIL; }
            let arg = args[arg_index]; arg_index += 1; input += 2;
            if arg.is_null() { continue; }
            let mut index = 0usize;
            while *arg.add(index) != 0 {
                if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return 0x8007007Au32 as HRESULT; }
                *dest.add(out) = *arg.add(index); out += 1; index += 1;
            }
            continue;
        }
        if out + 1 >= capacity { *dest.add(capacity - 1) = 0; return 0x8007007Au32 as HRESULT; }
        *dest.add(out) = unit; out += 1; input += 1;
    }
    *dest.add(out) = 0; S_OK
}
macro_rules! StringCbPrintfW {
    ($dest:expr, $bytes:expr, $format:expr, $arg1:expr $(,)?) => {
        cpp_com_string_cb_printf_w($dest, $bytes, $format, &[$arg1])
    };
    ($dest:expr, $bytes:expr, $format:expr, $arg1:expr, $arg2:expr $(,)?) => {
        cpp_com_string_cb_printf_w($dest, $bytes, $format, &[$arg1, $arg2])
    };
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

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
    fn memset(dst: *mut core::ffi::c_void, value: INT, bytes: usize) -> *mut core::ffi::c_void;
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

extern "C" {
    static mut shell32_hInstance: HINSTANCE;
}

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_CB6189A20402: [WCHAR; 15] = [0x0025, 0x0070, 0x0072, 0x006F, 0x0067, 0x0072, 0x0061, 0x006D, 0x0066, 0x0069, 0x006C, 0x0065, 0x0073, 0x0025, 0x0000];

// --local-raii CDataObjectHIDA: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
// --local-raii CDataObjectHIDA: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
//
// THE canonical copy. Pass it as
//   rustconv translate <tu> --free-functions --local-raii CDataObjectHIDA=tools/rustconv/data/local_raii/CDataObjectHIDA.rs
// Extracted from shell32/shldataobject.rs, which is one of the three already-flipped TUs that
// each had to carry an identical copy (the others: shell32/shlfolder.rs, dialogs/item_prop.rs).
// A fourth, COpenWithMenu.cpp, stalled unpaid for want of it — the refusal is `translate`
// declining to guess drop semantics, and the fix was always a file, never a retry.
// `hida_clipformat()` comes from the crate prelude (shellutils.rs:68); this file assumes it.
// Hand-verified Rust lowering of the C++ RAII struct `CDataObjectHIDA`
// (sdk/include/reactos/shellutils.h:738). Consumed by
// `rustconv translate --free-functions --local-raii CDataObjectHIDA=<this file>`:
// spliced verbatim into the assembled module, and its API sealed into each
// affected worker prompt so the model maps C++ usage onto this fixed surface
// instead of inventing the drop semantics.
//
// It borrows the HIDA out of an IDataObject WITHOUT copying it, so the medium
// must stay locked for exactly the lifetime of the guard — which is the whole
// reason it is RAII and the whole reason a hand lowering is required.
//
// C++ semantics preserved exactly:
//   ctor        — CreateCIDA(pDataObject, &m_cida, m_medium); m_hr = result
//   dtor        — DestroyCIDA(m_cida, m_medium)                → impl Drop
//   hr()        — the stored ctor HRESULT
//   operator bool()        → is_valid()      (m_cida != nullptr)
//   operator const CIDA*() → as_ptr()        (the implicit conversion at a call
//                                             site such as `f(x, cida)`)
//   operator->()           → (*cida.as_ptr()).field
//
// Two details that look like bugs and are not, so do not "fix" them:
//   * DestroyCIDA calls ReleaseStgMedium unconditionally. On the failure paths
//     CreateCIDA has already set tymed = TYMED_NULL, which makes that call a
//     documented no-op — including after CreateCIDA itself released the medium.
//   * C++ leaves m_medium uninitialized and only assigns pUnkForRelease. Every
//     path assigns tymed before anything reads it (GetData on success,
//     TYMED_NULL otherwise). Zeroing here is strictly safer and changes no path.
struct CDataObjectHIDA {
    m_medium: STGMEDIUM,
    m_cida: *const CIDA,
    m_hr: HRESULT,
}

impl CDataObjectHIDA {
    unsafe fn new(pDataObject: *mut IDataObject) -> CDataObjectHIDA {
        let mut medium: STGMEDIUM = core::mem::zeroed();
        let mut cida: *const CIDA = core::ptr::null();
        let hr = CDataObjectHIDA::CreateCIDA(pDataObject, &mut cida, &mut medium);
        CDataObjectHIDA { m_medium: medium, m_cida: cida, m_hr: hr }
    }

    unsafe fn CreateCIDA(
        pDataObject: *mut IDataObject,
        ppcida: *mut *const CIDA,
        medium: *mut STGMEDIUM,
    ) -> HRESULT {
        *ppcida = core::ptr::null();
        (*medium).pUnkForRelease = core::ptr::null_mut();
        // g_cfHIDA in C++ is a DECLSPEC_SELECTANY cache of the registered format.
        let mut format = FORMATETC {
            cfFormat: hida_clipformat(),
            ptd: core::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL,
        };
        let hr = ((*(*pDataObject).lpVtbl).GetData)(
            pDataObject,
            (&mut format as *mut FORMATETC).cast(),
            medium.cast(),
        );
        if hr >= 0 {
            *ppcida = GlobalLock((*medium).__bindgen_anon_1.hGlobal) as *const CIDA;
            if !(*ppcida).is_null() {
                return S_OK;
            }
            ReleaseStgMedium(medium);
            (*medium).tymed = TYMED_NULL;
            return E_UNEXPECTED;
        }
        (*medium).tymed = TYMED_NULL;
        hr
    }

    unsafe fn hr(&self) -> HRESULT {
        self.m_hr
    }

    unsafe fn is_valid(&self) -> bool {
        !self.m_cida.is_null()
    }

    unsafe fn as_ptr(&self) -> *const CIDA {
        self.m_cida
    }
}

impl Drop for CDataObjectHIDA {
    fn drop(&mut self) {
        unsafe {
            if !self.m_cida.is_null() {
                GlobalUnlock(self.m_medium.__bindgen_anon_1.hGlobal);
            }
            ReleaseStgMedium(&mut self.m_medium);
        }
    }
}

use core::ffi::c_void;

pub unsafe fn COpenWithDialog_ctor_body(this: *mut COpenWithDialog, pInfo: *const OPENASINFO, pAppList: *mut COpenWithList) -> () {
    (*this).m_pInfo = pInfo;
    (*this).m_pAppList = pAppList;
    (*this).m_hImgList = core::ptr::null_mut();
    (*this).m_bNoOpen = 0;
    if (*this).m_pAppList.is_null() {
        (*this).m_pAppList = COpenWithList_new();
        (*this).m_bListAllocated = TRUE;
    } else {
        (*this).m_bListAllocated = FALSE;
    }
}

pub unsafe fn COpenWithDialog_dtor_body(this: *mut COpenWithDialog) -> () {
    if (*this).m_bListAllocated != 0 && !(*this).m_pAppList.is_null() {
        COpenWithList_delete((*this).m_pAppList);
    }
    if !(*this).m_hImgList.is_null() {
        ImageList_Destroy((*this).m_hImgList);
    }
}

pub unsafe fn COpenWithDialog_NoOpenDlgProc_body(this: *mut COpenWithDialog, hwnd: HWND, Message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match Message {
        WM_INITDIALOG => return TRUE as INT_PTR,
        WM_CLOSE => {
            EndDialog(hwnd, IDNO as INT_PTR);
        }
        WM_COMMAND => {
            match LOWORD!(wParam) {
                IDYES => {
                    EndDialog(hwnd, IDYES as INT_PTR);
                }
                IDNO => {
                    EndDialog(hwnd, IDNO as INT_PTR);
                }
                _ => {}
            }
        }
        _ => return FALSE as INT_PTR,
    }
    TRUE as INT_PTR
}

/// DLGPROC entry for `COpenWithDialog::NoOpenDlgProc`, a C++ STATIC member.
///
/// The scaffold gives every method a `this`, static or not, so the body carries one it never
/// reads — this dialog keeps no state and stashes nothing in DWLP_USER. Passing null makes
/// that explicit; the day the body starts dereferencing `this` it faults here rather than
/// reading whatever the parameter slot happened to hold.
unsafe extern "system" fn COpenWithDialog_NoOpenDlgProc_thunk(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    COpenWithDialog_NoOpenDlgProc_body(core::ptr::null_mut(), hwndDlg, uMsg, wParam, lParam)
}

pub unsafe fn COpenWithDialog_IsNoOpen_body(this: *mut COpenWithDialog, hwnd: HWND) -> BOOL {
    if (*this).m_bNoOpen != FALSE {
        let dReturnValue = DialogBoxParamW(
            shell32_hInstance,
            MAKEINTRESOURCE(IDD_NOOPEN),
            hwnd,
            Some(COpenWithDialog_NoOpenDlgProc_thunk),
            0,
        );
        if dReturnValue == IDNO as INT_PTR {
            return TRUE;
        } else if dReturnValue == -1 as INT_PTR {
            return TRUE;
        }
    }
    FALSE
}

pub unsafe fn COpenWithDialog_AddApp_body(this: *mut COpenWithDialog, pApp: *mut SApp, bSelected: BOOL) -> () {
    let pwszName: PCWSTR = COpenWithList_GetName_body((*this).m_pAppList, pApp);
    if pwszName.is_null() {
        return;
    }
    let hIcon = COpenWithList_GetIcon_body((*this).m_pAppList, pApp);

    let mut tvins: TVINSERTSTRUCTW = core::mem::zeroed();
    if (*pApp).bRecommended != 0 {
        tvins.hParent = (*this).m_hRecommend;
        tvins.hInsertAfter = (*this).m_hRecommend;
    } else {
        tvins.hParent = (*this).m_hOther;
        tvins.hInsertAfter = (*this).m_hOther;
    }
    // The C writes `tvins.item.X`; bindgen renders that tail as an anonymous UNION of
    // itemex/item, so the member is reached through `__bindgen_anon_1` and a union accessor.
    // Binding it once keeps the run of writes reading like the C. The mutable borrow ends at
    // its last use, so `&tvins` below is free to take a shared one.
    let item = tvins.__bindgen_anon_1.item.as_mut();
    item.mask = TVIF_TEXT | TVIF_PARAM;
    item.pszText = pwszName as PWSTR;
    item.lParam = pApp as LPARAM;

    if !hIcon.is_null() {
        item.mask |= TVIF_IMAGE | TVIF_SELECTEDIMAGE;
        let iIcon = ImageList_AddIcon!((*this).m_hImgList, hIcon);
        item.iImage = iIcon;
        item.iSelectedImage = iIcon;
    }

    let hItem = TreeView_InsertItem((*this).m_hTreeView, &tvins);

    if bSelected != 0 {
        TreeView_SelectItem((*this).m_hTreeView, hItem);
    }
}

pub unsafe fn COpenWithDialog_Browse_body(this: *mut COpenWithDialog) -> () {
    let mut wszTitle: [u16; 64] = [0u16; 64];
    let mut wszFilter: [u16; 256] = [0u16; 256];
    let mut wszPath: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    let mut ofn: OPENFILENAMEW = core::mem::zeroed();

    // ZeroMemory the structure (original: ZeroMemory(&ofn, sizeof(COMMENT_TOKEN_ECHO____)))
    ZeroMemory(
        core::ptr::addr_of_mut!(ofn).cast::<core::ffi::c_void>(),
        core::mem::size_of::<OPENFILENAMEW>(),
    );

    ofn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hInstance = shell32_hInstance;
    ofn.hwndOwner = (*this).m_hDialog;
    ofn.Flags = (OFN_EXPLORER | OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST) as u32;
    ofn.nMaxFile = (core::mem::size_of_val(&wszPath) / core::mem::size_of_val(&wszPath[0])) as u32;
    ofn.lpstrFile = wszPath.as_mut_ptr();
    ofn.lpstrInitialDir = CPP_COM_WSTR_CB6189A20402.as_ptr();

    // Load title string
    let title_len = (core::mem::size_of_val(&wszTitle) / core::mem::size_of_val(&wszTitle[0])) as i32;
    if LoadStringW(shell32_hInstance, IDS_OPEN_WITH, wszTitle.as_mut_ptr(), title_len) != 0 {
        ofn.lpstrTitle = wszTitle.as_mut_ptr();
        ofn.nMaxFileTitle = wcslen(wszTitle.as_ptr()) as u32;
    }

    // Load filter string
    let filter_len = (core::mem::size_of_val(&wszFilter) / core::mem::size_of_val(&wszFilter[0])) as i32;
    if LoadStringW(shell32_hInstance, IDS_OPEN_WITH_FILTER, wszFilter.as_mut_ptr(), filter_len) != 0 {
        ofn.lpstrFilter = wszFilter.as_mut_ptr();
    }

    // ZeroMemory the path buffer
    ZeroMemory(wszPath.as_mut_ptr().cast::<core::ffi::c_void>(), core::mem::size_of_val(&wszPath));

    // Open the file dialog
    if GetOpenFileNameW(core::ptr::addr_of_mut!(ofn) as *mut OPENFILENAMEW) == 0 {
        return;
    }

    // Add the chosen path to the app list and insert it into the UI
    let pApp = COpenWithList_Add_body((*this).m_pAppList, wszPath.as_mut_ptr()) as *mut SApp;
    COpenWithDialog_AddApp_body(this, pApp, TRUE);
}

pub unsafe fn COpenWithDialog_GetCurrentApp_body(this: *mut COpenWithDialog) -> *mut SApp {
    let mut tvi: TVITEMW = core::mem::zeroed();
    tvi.hItem = TreeView_GetSelection((*this).m_hTreeView);
    if tvi.hItem.is_null() {
        return core::ptr::null_mut();
    }
    tvi.mask = TVIF_PARAM;
    if TreeView_GetItem((*this).m_hTreeView, &mut tvi) == 0 {
        return core::ptr::null_mut();
    }
    tvi.lParam as *mut SApp
}

pub unsafe fn COpenWithDialog_Init_body(this: *mut COpenWithDialog, hwnd: HWND) -> () {
    (*this).m_hDialog = hwnd;
    SetWindowLongPtrW(hwnd, DWLP_USER, this as LONG_PTR);

    let mut fDisallow: UINT = 0;
    let pszExt: PCWSTR = PathFindExtensionW((*(*this).m_pInfo).pcszFile);
    if pszExt.is_null() || *pszExt.add(0 as usize) == 0 || *pszExt.add(1 as usize) == 0 || PathIsExeW((*(*this).m_pInfo).pcszFile) != FALSE {
        fDisallow |= OAIF_ALLOW_REGISTRATION | OAIF_FORCE_REGISTRATION;
    }
    if SHRestricted(REST_NOFILEASSOCIATE) != 0 {
        fDisallow |= OAIF_ALLOW_REGISTRATION | OAIF_FORCE_REGISTRATION | OAIF_REGISTER_EXT;
    }

    (*this).m_InFlags = ((*(*this).m_pInfo).oaifInFlags as UINT) & !fDisallow;
    let hRegisterCheckbox: HWND = GetDlgItem(hwnd, 14003);
    if ((*this).m_InFlags & OAIF_ALLOW_REGISTRATION) == 0 {
        EnableWindow(hRegisterCheckbox, FALSE);
    }
    if ((*this).m_InFlags & OAIF_FORCE_REGISTRATION) != 0 {
        SendMessage(hRegisterCheckbox, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
    }
    if ((*this).m_InFlags & OAIF_HIDE_REGISTRATION) != 0 {
        ShowWindow(hRegisterCheckbox, SW_HIDE as INT);
    }

    let m_pInfo = (*this).m_pInfo;
    let m_pAppList = (*this).m_pAppList;
    if !(*m_pInfo).pcszFile.is_null() {
        let mut wszBuf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let cchBuf: UINT = GetDlgItemTextW(hwnd, 14001, wszBuf.as_mut_ptr(), wszBuf.len() as INT) as UINT;
        StringCchCopyW(wszBuf.as_mut_ptr().add(cchBuf as usize), wszBuf.len() - cchBuf as usize, PathFindFileNameW((*m_pInfo).pcszFile));
        SetDlgItemTextW(hwnd, 14001, wszBuf.as_mut_ptr());

        COpenWithList_Load_body(m_pAppList);
        COpenWithList_LoadRecommended_body(m_pAppList, (*m_pInfo).pcszFile);

        if COpenWithList_IsNoOpen_body(m_pAppList) != FALSE {
            (*this).m_bNoOpen = TRUE;
        }

        (*this).m_hTreeView = GetDlgItem(hwnd, 14002);
        let cApps = COpenWithList_GetCount_body(m_pAppList);
        (*this).m_hImgList = ImageList_Create(16, 16, ILC_COLOR32 | ILC_MASK, (cApps + 1) as INT, (cApps + 1) as INT);
        TreeView_SetImageList((*this).m_hTreeView, (*this).m_hImgList, TVSIL_NORMAL as INT);

        let cRecommended: UINT = COpenWithList_GetRecommendedCount_body(m_pAppList);
        if cRecommended > 0 {
            let mut tvins: TVINSERTSTRUCTW = core::mem::zeroed();
            tvins.hParent = TVI_ROOT;
            tvins.hInsertAfter = TVI_ROOT;
            let hFolderIcon: HICON = LoadImage(shell32_hInstance, MAKEINTRESOURCE(IDI_SHELL_PROGRAMS_FOLDER), IMAGE_ICON, 0, 0, 0) as HICON;
            let iImage = ImageList_AddIcon!((*this).m_hImgList, hFolderIcon);
            // anonymous itemex/item union — see the note in COpenWithDialog_AddApp_body
            let item = tvins.__bindgen_anon_1.item.as_mut();
            item.mask = TVIF_TEXT | TVIF_STATE | TVIF_IMAGE | TVIF_SELECTEDIMAGE;
            item.pszText = wszBuf.as_mut_ptr() as LPWSTR;
            item.state = TVIS_EXPANDED;
            item.stateMask = TVIS_EXPANDED;
            item.iImage = iImage;
            item.iSelectedImage = iImage;

            LoadStringW(shell32_hInstance, IDS_OPEN_WITH_RECOMMENDED, wszBuf.as_mut_ptr(), wszBuf.len() as INT);
            (*this).m_hRecommend = TreeView_InsertItem((*this).m_hTreeView, &tvins);

            LoadStringW(shell32_hInstance, IDS_OPEN_WITH_OTHER, wszBuf.as_mut_ptr(), wszBuf.len() as INT);
            (*this).m_hOther = TreeView_InsertItem((*this).m_hTreeView, &tvins);
        } else {
            (*this).m_hRecommend = TVI_ROOT;
            (*this).m_hOther = TVI_ROOT;
        }

        let mut bNoAppSelected: BOOL = TRUE;
        let pAppList: *mut SApp = COpenWithList_GetList_body(m_pAppList);
        let totalApps = COpenWithList_GetCount_body(m_pAppList);
        let mut i: UINT = 0;
        while i < totalApps {
            if COpenWithList_IsHidden_body(m_pAppList, pAppList.add(i as usize)) == 0 {
                if bNoAppSelected != 0 && ((*pAppList.add(i as usize)).bRecommended != 0 || cRecommended == 0) {
                    COpenWithDialog_AddApp_body(this, pAppList.add(i as usize), TRUE);
                    bNoAppSelected = FALSE;
                } else {
                    COpenWithDialog_AddApp_body(this, pAppList.add(i as usize), FALSE);
                }
            }
            i += 1;
        }
    }
}

pub unsafe fn COpenWithDialog_Accept_body(this: *mut COpenWithDialog) -> () {
    let pApp = COpenWithDialog_GetCurrentApp_body(this);
    if !pApp.is_null() {
        if IsDlgButtonChecked((*this).m_hDialog, 14003) == BST_CHECKED && ((*this).m_InFlags & OAIF_REGISTER_EXT) != 0 {
            COpenWithList_SetDefaultHandler_body((*this).m_pAppList, pApp, (*(*this).m_pInfo).pcszFile);
        }
        if ((*this).m_InFlags & OAIF_EXEC) != 0 {
            COpenWithList_Execute_body((*this).m_pAppList, pApp, (*(*this).m_pInfo).pcszFile);
        }
        EndDialog((*this).m_hDialog, 1);
    }
}

pub unsafe fn COpenWithDialog_DialogProc_body(this: *mut COpenWithDialog, hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let pThis = GetWindowLongPtrW(hwndDlg, DWLP_USER) as *mut COpenWithDialog;

    match uMsg {
        WM_INITDIALOG => {
            let pThis = lParam as *mut COpenWithDialog;
            COpenWithDialog_Init_body(pThis, hwndDlg);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match LOWORD!(wParam) {
                14004 => {
                    COpenWithDialog_Browse_body(pThis);
                    return TRUE as INT_PTR;
                }
                IDOK => {
                    COpenWithDialog_Accept_body(pThis);
                    return TRUE as INT_PTR;
                }
                IDCANCEL => {
                    EndDialog(hwndDlg, 0);
                    return TRUE as INT_PTR;
                }
                _ => {}
            }
        }
        WM_NOTIFY => {
            match (*(lParam as *const NMHDR)).code {
                TVN_SELCHANGED => {
                    let current_app = COpenWithDialog_GetCurrentApp_body(pThis);
                    let enable = if !current_app.is_null() {
                        TRUE
                    } else {
                        FALSE
                    };
                    EnableWindow(
                        GetDlgItem(hwndDlg, IDOK as INT),
                        enable,
                    );
                }
                NM_DBLCLK | NM_RETURN => {
                    COpenWithDialog_Accept_body(pThis);
                }
                _ => {}
            }
        }
        WM_CLOSE => {
            EndDialog(hwndDlg, 0);
            return TRUE as INT_PTR;
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}
