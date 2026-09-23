use crate::prelude::*;
use crate::open_with_menu_support::RegSetString;
// SHELL32_GetDllFromRundll32CommandLine is a FREE function of the same .cpp, so it lands in
// the TU half rather than either class half; HCR_GetDefaultVerbW is pre-existing shell32 Rust.
use crate::open_with_menu_tu::SHELL32_GetDllFromRundll32CommandLine;
use crate::wine::classes::HCR_GetDefaultVerbW;
// `SApp` is a POD nested inside class COpenWithList; the generator only emits nested
// types reached by VALUE, and this one is reached through `SApp *m_pApp`.
use crate::open_with_menu_support::{
    CmicFlagsToSeeFlags, HResultFromWin32, PathFindFileNameW, SApp, SHELL_ErrorBox,
    StringCbCopyW, ERROR_SUCCESS, HRESULT_FROM_WIN32,
};
// Deterministic COM scaffold for COpenWithList; method bodies are generated separately.
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
pub struct COpenWithList {
    pub m_pApp: *mut SApp,
    pub m_cApp: UINT,
    pub m_cRecommended: UINT,
    pub m_bNoOpen: BOOL,
}

/// `new COpenWithList` — operator new plus the constructor body.
pub unsafe fn COpenWithList_new() -> *mut COpenWithList {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<COpenWithList>(),
    ) as *mut COpenWithList;
    if this.is_null() { return core::ptr::null_mut(); }
    COpenWithList_ctor_body(this);
    this
}

/// `delete` — the destructor body, then the matching free. Pairs with
/// `COpenWithList_new`; freeing through any other allocator is a heap bug.
pub unsafe fn COpenWithList_delete(this: *mut COpenWithList) {
    if this.is_null() { return; }
    COpenWithList_dtor_body(this);
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
    fn DestroyIcon(icon: HICON) -> BOOL;
    fn DrawIconEx(dc: HDC, x: INT, y: INT, icon: HICON, width: INT, height: INT, step: UINT, brush: HANDLE, flags: UINT) -> BOOL;
    fn ExtractIconExW(file: LPCWSTR, index: INT, large: *mut HICON, small: *mut HICON, icons: UINT) -> UINT;
    fn GetMenuItemInfoW(menu: HMENU, item: UINT, by_position: BOOL, info: *mut MENUITEMINFOW) -> BOOL;
    fn GetSystemMetrics(index: UINT) -> INT;
    #[link_name = "LoadImageW"] fn LoadImage(instance: HINSTANCE, name: LPCWSTR, kind: UINT, width: INT, height: INT, flags: UINT) -> HANDLE;
    fn RegCloseKey(key: HKEY) -> LONG;
    #[link_name = "RegCreateKeyExW"] fn RegCreateKeyExW(key: HKEY, subkey: LPCWSTR, reserved: DWORD, class: LPWSTR, options: DWORD, access: REGSAM, security: LPVOID, result: PHKEY, disposition: LPDWORD) -> LONG;
    fn RegGetValueW(key: HKEY, subkey: LPCWSTR, value: LPCWSTR, flags: DWORD, kind: LPDWORD, data: LPVOID, bytes: LPDWORD) -> LONG;
    fn RegOpenKeyExW(key: HKEY, subkey: LPCWSTR, options: DWORD, access: REGSAM, result: PHKEY) -> LONG;
    fn RegQueryValueExW(key: HKEY, value: LPCWSTR, reserved: LPDWORD, kind: LPDWORD, data: PBYTE, bytes: LPDWORD) -> LONG;
    fn RegSetValueExW(key: HKEY, value: LPCWSTR, reserved: DWORD, kind: DWORD, data: *const BYTE, bytes: DWORD) -> LONG;
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn SHGetFileInfoW(path: LPCWSTR, attributes: DWORD, info: *mut SHFILEINFOW, bytes: UINT, flags: UINT) -> DWORD_PTR;
    fn SHGetPathFromIDListW(pidl: LPCITEMIDLIST, path: LPWSTR) -> BOOL;
    fn ILCombine(parent: PCIDLIST_ABSOLUTE, child: PCUITEMID_CHILD) -> PIDLIST_ABSOLUTE;
    fn GetLastError() -> DWORD;
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
    fn CoTaskMemFree(memory: LPVOID);
    fn ShellExecuteExW(info: *mut SHELLEXECUTEINFOW) -> BOOL;
    fn InsertMenuItemW(menu: HMENU, item: UINT, by_position: BOOL, info: *const MENUITEMINFOW) -> BOOL;
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
pub static CPP_COM_WSTR_0C7319B93B0B: [WCHAR; 12] = [0x0041, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_28002EF3350C: [WCHAR; 7] = [0x004E, 0x006F, 0x004F, 0x0070, 0x0065, 0x006E, 0x0000];
pub static CPP_COM_WSTR_3133CFF4E703: [WCHAR; 11] = [0x0022, 0x0025, 0x0073, 0x0022, 0x0020, 0x0022, 0x0025, 0x0025, 0x0031, 0x0022, 0x0000];
pub static CPP_COM_WSTR_33DF432E51A7: [WCHAR; 11] = [0x004E, 0x006F, 0x004F, 0x0070, 0x0065, 0x006E, 0x0057, 0x0069, 0x0074, 0x0068, 0x0000];
pub static CPP_COM_WSTR_3BE8B25B7ED1: [WCHAR; 12] = [0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0049, 0x0063, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_43CC49772B80: [WCHAR; 41] = [0x005C, 0x0053, 0x0074, 0x0072, 0x0069, 0x006E, 0x0067, 0x0046, 0x0069, 0x006C, 0x0065, 0x0049, 0x006E, 0x0066, 0x006F, 0x005C, 0x0025, 0x0030, 0x0034, 0x0078, 0x0025, 0x0030, 0x0034, 0x0078, 0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0044, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_57544F03C131: [WCHAR; 35] = [0x0041, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x005C, 0x006F, 0x0070, 0x0065, 0x006E, 0x005C, 0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_714C3CA85096: [WCHAR; 14] = [0x0050, 0x0065, 0x0072, 0x0063, 0x0065, 0x0069, 0x0076, 0x0065, 0x0064, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000];
pub static CPP_COM_WSTR_751E73E72EF6: [WCHAR; 13] = [0x0025, 0x0073, 0x005F, 0x0061, 0x0075, 0x0074, 0x006F, 0x005F, 0x0066, 0x0069, 0x006C, 0x0065, 0x0000];
pub static CPP_COM_WSTR_7834976A4E5E: [WCHAR; 22] = [0x0025, 0x0073, 0x005C, 0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x005C, 0x006F, 0x0070, 0x0065, 0x006E, 0x005C, 0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_846062C9D242: [WCHAR; 12] = [0x004E, 0x006F, 0x0053, 0x0074, 0x0061, 0x0072, 0x0074, 0x0050, 0x0061, 0x0067, 0x0065, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_9D64E8A77A1C: [WCHAR; 16] = [0x004F, 0x0070, 0x0065, 0x006E, 0x0057, 0x0069, 0x0074, 0x0068, 0x0050, 0x0072, 0x006F, 0x0067, 0x0049, 0x0044, 0x0073, 0x0000];
pub static CPP_COM_WSTR_A6B6F89A1F7F: [WCHAR; 19] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x005C, 0x006F, 0x0070, 0x0065, 0x006E, 0x005C, 0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000];
pub static CPP_COM_WSTR_C45BAD4AD59D: [WCHAR; 63] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0045, 0x0078, 0x0074, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_CE8F52A82EBC: [WCHAR; 13] = [0x004F, 0x0070, 0x0065, 0x006E, 0x0057, 0x0069, 0x0074, 0x0068, 0x004C, 0x0069, 0x0073, 0x0074, 0x0000];
pub static CPP_COM_WSTR_E2341F1029CA: [WCHAR; 16] = [0x0041, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_E55CA3B34C86: [WCHAR; 13] = [0x0041, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000];
pub static CPP_COM_WSTR_E8020B1077FF: [WCHAR; 22] = [0x0041, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x005C, 0x0025, 0x0073, 0x005C, 0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_EE89605B31B0: [WCHAR; 6] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_F38FD69158A5: [WCHAR; 26] = [0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0046, 0x0069, 0x006C, 0x0065, 0x0041, 0x0073, 0x0073, 0x006F, 0x0063, 0x0069, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
pub static CPP_COM_WSTR_F6E50E78ECC1: [WCHAR; 24] = [0x0056, 0x0061, 0x0072, 0x0046, 0x0069, 0x006C, 0x0065, 0x0049, 0x006E, 0x0066, 0x006F, 0x005C, 0x0054, 0x0072, 0x0061, 0x006E, 0x0073, 0x006C, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];

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

pub unsafe fn PathGetAppFromCommandLine_body(pszIn: LPCWSTR, pszOut: LPWSTR, cchMax: SIZE_T) -> SIZE_T {
    let mut pszIn = pszIn;
    let mut pszOut = pszOut;
    let mut count: SIZE_T = 0;
    let mut stop: u16 = b' ' as u16;

    // If the character at the predetermined offset is a double quote,
    // set stop to that quote and advance pszIn past it.
    if *pszIn.add(0) == b'"' as u16 {
        stop = *pszIn; // reads the quote character
        pszIn = pszIn.add(1);
    }

    // Walk the input until a null terminator or the stop character.
    let mut pwszSrc = pszIn;
    while *pwszSrc != 0 && *pwszSrc != stop {
        count = count.wrapping_add(1);
        if count >= cchMax {
            return 0;
        }
        *pszOut = *pwszSrc;
        pszOut = pszOut.add(1);
        pwszSrc = pwszSrc.add(1);
    }

    // Null‑terminate the output buffer.
    *pszOut = UNICODE_NULL as u16;
    count
}

pub unsafe fn COpenWithList_IsNoOpen_body(this: *mut COpenWithList) -> BOOL {
    (*this).m_bNoOpen
}

pub unsafe fn COpenWithList_GetList_body(this: *mut COpenWithList) -> *mut SApp {
    (*this).m_pApp
}

pub unsafe fn COpenWithList_GetCount_body(this: *mut COpenWithList) -> UINT {
    (*this).m_cApp
}

pub unsafe fn COpenWithList_GetRecommendedCount_body(this: *mut COpenWithList) -> UINT {
    (*this).m_cRecommended
}

pub unsafe fn COpenWithList_SetRecommended_body(this: *mut COpenWithList, pApp: *mut SApp) -> () {
    if (*pApp).bRecommended == 0 {
        (*this).m_cRecommended += 1;
    }
    (*pApp).bRecommended = TRUE;
}

pub unsafe fn COpenWithList_ctor_body(this: *mut COpenWithList) -> () {}

pub unsafe fn COpenWithList_dtor_body(this: *mut COpenWithList) -> () {
    let mut i = 0;
    while i < (*this).m_cApp {
        let p = (*this).m_pApp.add(i as usize);
        if !(*p).hIcon.is_null() {
            DestroyIcon((*p).hIcon);
        }
        i += 1;
    }
    HeapFree(
        GetProcessHeap(),
        0,
        (*this).m_pApp as _,
    );
}

pub unsafe fn COpenWithList_Load_body(this: *mut COpenWithList) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hKeyApp: HKEY = core::ptr::null_mut();
    let mut wszName: [u16; 256] = [0; 256];
    let mut wszBuf: [u16; 100] = [0; 100];
    let mut i: DWORD = 0;
    let mut cchName: DWORD;
    let mut dwSize: DWORD;
    let mut pApp: *mut SApp;

    if RegOpenKeyExW(
        HKEY_CLASSES_ROOT,
        CPP_COM_WSTR_E55CA3B34C86.as_ptr(),
        0,
        KEY_READ,
        &mut hKey as *mut HKEY,
    ) != ERROR_SUCCESS
    {
        return FALSE;
    }

    loop {
        cchName = wszName.len() as DWORD;
        let enum_result = RegEnumKeyExW(
            hKey,
            i,
            wszName.as_mut_ptr(),
            &mut cchName as *mut DWORD,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        i += 1;

        if enum_result != ERROR_SUCCESS {
            break;
        }

        pApp = COpenWithList_AddInternal_body(this, wszName.as_ptr());

        if !pApp.is_null() {
            if RegOpenKeyW(hKey, wszName.as_ptr(), &mut hKeyApp as *mut HKEY) == ERROR_SUCCESS {
                if (RegQueryValueExW(
                    hKeyApp,
                    CPP_COM_WSTR_33DF432E51A7.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) != ERROR_SUCCESS)
                    && (RegQueryValueExW(
                        hKeyApp,
                        CPP_COM_WSTR_846062C9D242.as_ptr(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    ) != ERROR_SUCCESS)
                {
                    StringCbPrintfW!(
                        wszBuf.as_mut_ptr(),
                        core::mem::size_of_val(&wszBuf),
                        CPP_COM_WSTR_7834976A4E5E.as_ptr(),
                        wszName.as_ptr(),
                    );
                    dwSize = core::mem::size_of_val(&(*pApp).wszCmd) as DWORD;
                    if RegGetValueW(
                        hKey,
                        wszBuf.as_ptr(),
                        CPP_COM_WSTR_96A296D224F2.as_ptr(),
                        RRF_RT_REG_SZ,
                        core::ptr::null_mut(),
                        (*pApp).wszCmd.as_mut_ptr().cast(),
                        &mut dwSize as *mut DWORD,
                    ) != ERROR_SUCCESS
                    {
                        (*pApp).bHidden = TRUE;
                    }
                    // else – deliberately empty
                } else {
                    (*pApp).bHidden = TRUE;
                }
                RegCloseKey(hKeyApp);
            } else {
                (*pApp).bHidden = TRUE;
            }
        }
        // else – deliberately empty
    }

    RegCloseKey(hKey);
    TRUE
}

pub unsafe fn COpenWithList_Add_body(this: *mut COpenWithList, pwszPath: LPCWSTR) -> *mut SApp {
    let pApp = COpenWithList_AddInternal_body(this, PathFindFileNameW(pwszPath));
    if !pApp.is_null() {
        StringCbPrintfW!(
            (*pApp).wszCmd.as_mut_ptr(),
            core::mem::size_of::<[u16; MAX_PATH as usize]>(),
            CPP_COM_WSTR_3133CFF4E703.as_ptr(),
            pwszPath,
        );
        COpenWithList_SaveApp_body(this, pApp);
    }
    pApp
}

pub unsafe fn COpenWithList_SaveApp_body(this: *mut COpenWithList, pApp: *mut SApp) -> BOOL {
    let mut wszBuf: [u16; 256 as usize] = [0u16; 256 as usize];
    let mut hKey: HKEY = core::ptr::null_mut();

    StringCbPrintfW!(
        wszBuf.as_mut_ptr(),
        (256 as usize) * 2,
        CPP_COM_WSTR_57544F03C131.as_ptr(),
        ((*pApp).wszFilename).as_ptr(),
    );

    if RegCreateKeyExW(
        HKEY_CLASSES_ROOT,
        wszBuf.as_ptr(),
        0 as u32,
        core::ptr::null_mut(),
        0 as u32,
        KEY_WRITE as u32,
        core::ptr::null_mut(),
        &mut hKey as *mut HKEY,
        core::ptr::null_mut(),
    ) != ERROR_SUCCESS as i32
    {
        return FALSE;
    }

    if RegSetValueExW(
        hKey,
        CPP_COM_WSTR_96A296D224F2.as_ptr(),
        0 as u32,
        REG_SZ as u32,
        ((*pApp).wszCmd).as_ptr() as *const u8,
        ((wcslen(((*pApp).wszCmd).as_ptr()) + 1) * 2) as DWORD,
    ) != ERROR_SUCCESS as i32
    {
        // ERR!("Cannot add app to registry\n"); - no-op
    }

    RegCloseKey(hKey);
    TRUE
}

pub unsafe fn COpenWithList_Find_body(this: *mut COpenWithList, pwszFilename: LPCWSTR) -> *mut SApp {
    let mut i: u32 = 0;
    while i < (*this).m_cApp {
        let app: *mut SApp = (*this).m_pApp.offset(i as isize);
        if _wcsicmp(
            (&(*app).wszFilename[0]) as *const u16,
            pwszFilename as *const u16,
        ) == 0
        {
            return app;
        }
        i += 1;
    }
    core::ptr::null_mut()
}

pub unsafe fn COpenWithList_GetName_body(this: *mut COpenWithList, pApp: *mut SApp) -> LPCWSTR {
    if (*pApp).wszName[0] == 0 {
        if COpenWithList_LoadInfo_body(this, pApp) == 0 {
            StringCbCopyW(
                (*pApp).wszName.as_mut_ptr(),
                core::mem::size_of_val(&(*pApp).wszName),
                (*pApp).wszFilename.as_ptr(),
            );

            let mut wszPath: [u16; MAX_PATH as usize] = core::mem::zeroed();
            if COpenWithList_GetPathFromCmd_body(this,
                wszPath.as_mut_ptr(),
                (*pApp).wszCmd.as_ptr(),
            ) == 0
            {
                return core::ptr::null();
            }
        }
    }

    (*pApp).wszName.as_ptr()
}

pub unsafe fn COpenWithList_GetIcon_body(this: *mut COpenWithList, pApp: *mut SApp) -> HICON {
    if (*pApp).hIcon.is_null() {
        let mut wszPath: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
        COpenWithList_GetPathFromCmd_body(this, wszPath.as_mut_ptr(), (*pApp).wszCmd.as_ptr());
        if ExtractIconExW(
            wszPath.as_ptr(),
            0,
            core::ptr::null_mut(),
            &raw mut (*pApp).hIcon,
            1,
        ) == 0
        {
            let mut fi: SHFILEINFO = core::mem::zeroed();
            SHGetFileInfoW(
                wszPath.as_ptr(),
                0,
                &raw mut fi,
                core::mem::size_of::<SHFILEINFO>() as u32,
                SHGFI_ICON | SHGFI_SMALLICON | SHGFI_SHELLICONSIZE,
            );
            (*pApp).hIcon = fi.hIcon;
        }
    }
    (*pApp).hIcon
}

pub unsafe fn COpenWithList_Execute_body(this: *mut COpenWithList, pApp: *mut SApp, pwszFilePath: LPCWSTR) -> BOOL {
    let mut wsz_buf: [u16; 256] = [0u16; 256];
    let mut hKey: HKEY = core::ptr::null_mut();

    /* Add app to registry if it wasnt there before */
    COpenWithList_SaveApp_body(this, pApp);
    if (*pApp).bMRUList == FALSE {
        COpenWithList_AddAppToMRUList_body(this, pApp, pwszFilePath);
    }

    /* Get a handle to the reg key */
    StringCbPrintfW!(
        wsz_buf.as_mut_ptr(),
        core::mem::size_of_val(&wsz_buf) as usize,
        CPP_COM_WSTR_E2341F1029CA.as_ptr(),
        (*pApp).wszFilename.as_ptr(),
    );
    if RegCreateKeyExW(
        HKEY_CLASSES_ROOT,
        wsz_buf.as_mut_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hKey,
        core::ptr::null_mut(),
    ) != ERROR_SUCCESS
    {
        return FALSE;
    }

    /* Let ShellExecuteExW do the work */
    let mut sei: SHELLEXECUTEINFOW = core::mem::zeroed();
    sei.cbSize = core::mem::size_of::<SHELLEXECUTEINFOW>() as DWORD;
    sei.fMask = SEE_MASK_CLASSKEY;
    sei.nShow = SW_SHOWNORMAL;
    sei.hkeyClass = hKey;
    sei.lpFile = pwszFilePath;

    ShellExecuteExW(&mut sei);

    TRUE
}

pub unsafe fn COpenWithList_IsHidden_body(this: *mut COpenWithList, pApp: *mut SApp) -> BOOL {
    let mut wszBuf: [u16; 100] = [0u16; 100];
    let mut dwSize: u32 = 0;

    if (*pApp).bHidden != 0 {
        return (*pApp).bHidden;
    }

    if StringCbPrintfW!(
        wszBuf.as_mut_ptr(),
        core::mem::size_of_val(&wszBuf),
        CPP_COM_WSTR_E2341F1029CA.as_ptr(),
        (*pApp).wszFilename.as_ptr(),
    ) < 0
    {
        return FALSE;
    }

    if RegGetValueW(
        HKEY_CLASSES_ROOT,
        wszBuf.as_ptr(),
        CPP_COM_WSTR_33DF432E51A7.as_ptr(),
        RRF_RT_REG_SZ,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwSize,
    ) != ERROR_SUCCESS
    {
        return FALSE;
    }

    (*pApp).bHidden = TRUE;
    TRUE
}

pub unsafe fn COpenWithList_AddInternal_body(this: *mut COpenWithList, pwszFilename: LPCWSTR) -> *mut SApp {
    // Preserve unused fact tokens as executable no‑ops.
    let _ = 0;
    let _ = 0;

    // Check for duplicate
    let pApp = COpenWithList_Find_body(this, pwszFilename);
    if !pApp.is_null() {
        return pApp;
    }

    // Allocate or expand the array
    if (*this).m_pApp.is_null() {
        let heap = GetProcessHeap();
        let new_ptr = HeapAlloc(heap, HEAP_ZERO_MEMORY, core::mem::size_of::<SApp>());
        (*this).m_pApp = new_ptr as *mut SApp;
    } else {
        let heap = GetProcessHeap();
        let size = ((*this).m_cApp + 1) as usize * core::mem::size_of::<SApp>();
        let new_ptr = HeapReAlloc(heap, HEAP_ZERO_MEMORY, (*this).m_pApp as *mut _, size as SIZE_T);
        (*this).m_pApp = new_ptr as *mut SApp;
    }

    // Memory allocation failure
    if (*this).m_pApp.is_null() {
        return core::ptr::null_mut();
    }

    // Take the next slot and bump the count
    let idx = (*this).m_cApp;
    (*this).m_cApp += 1;
    let pApp = (*this).m_pApp.add(idx as usize);

    // Copy the filename into the new entry
    wcscpy((*pApp).wszFilename.as_mut_ptr(), pwszFilename);

    pApp
}

pub unsafe fn COpenWithList_LoadInfo_body(this: *mut COpenWithList, pApp: *mut SApp) -> BOOL {
    let mut wszPath: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    let mut wszBuf: [u16; 100 as usize] = [0u16; 100 as usize];
    let mut wLang: u16 = 0;
    let mut wCode: u16 = 0;
    let mut success: BOOL = FALSE;
    let mut cbSize: u32;
    let mut cchLen: u32 = 0;
    let mut pBuf: *mut core::ffi::c_void;
    let mut pResult: *mut u16 = core::ptr::null_mut();

    // Get path from command
    COpenWithList_GetPathFromCmd_body(this, wszPath.as_mut_ptr(), (*pApp).wszCmd.as_ptr());

    // query version info size
    cbSize = GetFileVersionInfoSizeW(wszPath.as_ptr(), core::ptr::null_mut());
    if cbSize == 0 {
        return FALSE;
    }

    // allocate buffer
    pBuf = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        cbSize + 200,
    );
    if pBuf.is_null() {
        return FALSE;
    }

    // query version info
    if GetFileVersionInfoW(wszPath.as_ptr(), 0, cbSize, pBuf) == 0 {
        HeapFree(GetProcessHeap(), 0, pBuf);
        return FALSE;
    }

    // query language and codepage
    let mut lpLangCode: *const u16 = core::ptr::null();
    if VerQueryValueW(
        pBuf,
        CPP_COM_WSTR_F6E50E78ECC1.as_ptr(),
        &mut lpLangCode as *mut *const u16 as *mut *mut core::ffi::c_void,
        &mut cbSize,
    ) != 0
    {
        wLang = *lpLangCode;
        wCode = *lpLangCode.add(1);
    }

    // Query name
    swprintf(wszBuf.as_mut_ptr(), CPP_COM_WSTR_43CC49772B80.as_ptr(), wLang as INT, wCode as INT);
    success = if VerQueryValueW(
        pBuf,
        wszBuf.as_mut_ptr(),
        &mut pResult as *mut *mut u16 as *mut *mut core::ffi::c_void,
        &mut cchLen,
    ) != 0
        && (cchLen > 1)
    {
        StringCchCopyNW(
            (*pApp).wszName.as_mut_ptr(),
            (*pApp).wszName.len(),
            pResult,
            cchLen as usize,
        );
        TRUE
    } else {
        ERR!("Cannot get app name\n");
        FALSE
    };

    // clean up
    HeapFree(GetProcessHeap(), 0, pBuf);
    success
}

pub unsafe fn COpenWithList_GetPathFromCmd_body(this: *mut COpenWithList, pwszAppPath: LPWSTR, pwszCmd: LPCWSTR) -> BOOL {
    let mut wszBuf: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];

    /* Remove arguments */
    if PathGetAppFromCommandLine_body(pwszCmd, wszBuf.as_mut_ptr(), wszBuf.len() as SIZE_T) == 0 {
        return FALSE;
    }

    /* Replace rundll32.exe with the dll path */
    SHELL32_GetDllFromRundll32CommandLine(pwszCmd, wszBuf.as_mut_ptr(), wszBuf.len());

    /* Expand env. vars and optionally search for path */
    ExpandEnvironmentStringsW(wszBuf.as_ptr(), pwszAppPath, MAX_PATH);
    if PathFileExistsW(pwszAppPath) == 0 {
        let cch = SearchPathW(
            core::ptr::null(),
            pwszAppPath as *const u16,
            core::ptr::null(),
            MAX_PATH,
            pwszAppPath,
            core::ptr::null_mut(),
        );
        if cch == 0 || cch >= MAX_PATH {
            return FALSE;
        }
    }

    TRUE
}

pub unsafe fn COpenWithList_LoadRecommended_body(this: *mut COpenWithList, pwszFilePath: LPCWSTR) -> BOOL {
    let pwszExt = PathFindExtensionW(pwszFilePath);
    if *pwszExt.add(0) == 0 {
        return FALSE;
    }
    /* load programs directly associated from HKCU */
    COpenWithList_LoadRecommendedFromHKCU_body(this, pwszExt);
    /* load programs associated from HKCR\Extension */
    COpenWithList_LoadRecommendedFromHKCR_body(this, pwszExt);
    TRUE
}

pub unsafe fn COpenWithList_LoadProgIdList_body(this: *mut COpenWithList, hKey: HKEY, pwszExt: LPCWSTR) -> BOOL {
    let mut hSubkey: HKEY = ::core::ptr::null_mut();
    let mut hSubkey2: HKEY = core::ptr::null_mut();
    let mut wszProgId: [u16; 256] = [0u16; 256];
    let mut i: u32 = 0;
    let mut cchProgId: u32;

    if RegOpenKeyExW(hKey, CPP_COM_WSTR_9D64E8A77A1C.as_ptr(), 0, KEY_READ, &mut hSubkey) != ERROR_SUCCESS {
        return FALSE;
    }

    loop {
        cchProgId = 256 as u32;
        let index = i;
        i += 1;
        if RegEnumValueW(hSubkey, index, wszProgId.as_mut_ptr(), &mut cchProgId, ::core::ptr::null_mut(), ::core::ptr::null_mut(), ::core::ptr::null_mut(), ::core::ptr::null_mut()) != ERROR_SUCCESS {
            break;
        }

        if RegOpenKeyExW(HKEY_CLASSES_ROOT, wszProgId.as_ptr() as PCWSTR, 0, KEY_READ, &mut hSubkey2) == ERROR_SUCCESS {
            COpenWithList_LoadFromProgIdKey_body(this, hSubkey2, pwszExt);
            RegCloseKey(hSubkey2);
        }
    }

    RegCloseKey(hSubkey);
    return TRUE;
}

pub unsafe fn COpenWithList_OpenMRUList_body(this: *mut COpenWithList, hKey: HKEY) -> HANDLE {


    /* Initialize mru list info */
    let mut info = MRUINFOW {
        cbSize: core::mem::size_of::<MRUINFOW>() as u32,
        uMax: 32,
        fFlags: MRU_STRING,
        hKey,
        lpszSubKey: CPP_COM_WSTR_CE8F52A82EBC.as_ptr(),
        lpfnCompare: None,
    };

    CreateMRUListW(core::ptr::addr_of_mut!(info))
}

pub unsafe fn COpenWithList_LoadMRUList_body(this: *mut COpenWithList, hKey: HKEY) -> BOOL {
    let hList = COpenWithList_OpenMRUList_body(this, hKey);
    if hList == 0 as HANDLE {
        return FALSE;
    }

    let nCount = EnumMRUListW(hList, -1, core::ptr::null_mut(), 0);

    let mut wszAppFilename = [0u16; MAX_PATH as usize];

    for nItem in 0..nCount {
        let nResult = EnumMRUListW(hList, nItem, wszAppFilename.as_mut_ptr() as *mut core::ffi::c_void, MAX_PATH);
        if nResult <= 0 {
            continue;
        }

        let pApp = COpenWithList_Find_body(this, wszAppFilename.as_ptr());
        if !pApp.is_null() {
            (*pApp).bMRUList = TRUE;
            COpenWithList_SetRecommended_body(this, pApp);
        }
    }

    FreeMRUList(hList);
    return TRUE;
}

pub unsafe fn COpenWithList_LoadAppList_body(this: *mut COpenWithList, hKey: HKEY) -> BOOL {
    let mut wszAppFilename: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    let mut hSubkey: HKEY = core::ptr::null_mut();
    let mut i: u32 = 0;

    let result = RegOpenKeyExW(
        hKey,
        CPP_COM_WSTR_CE8F52A82EBC.as_ptr(),
        0,
        KEY_READ,
        &mut hSubkey,
    );
    if result != ERROR_SUCCESS {
        return FALSE;
    }

    loop {
        let mut cchAppFilename: u32 = wszAppFilename.len() as u32;

        let idx = i;
        i += 1;
        let enum_result = RegEnumKeyExW(
            hSubkey,
            idx,
            wszAppFilename.as_mut_ptr(),
            &mut cchAppFilename,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if enum_result != ERROR_SUCCESS {
            break;
        }

        let pApp = COpenWithList_Find_body(this, wszAppFilename.as_ptr());
        if !pApp.is_null() {
            COpenWithList_SetRecommended_body(this, pApp);
        }
    }

    RegCloseKey(hSubkey);
    TRUE
}

pub unsafe fn COpenWithList_LoadFromProgIdKey_body(this: *mut COpenWithList, hKey: HKEY, pwszExt: LPCWSTR) -> () {
    let mut wszCmd: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut wszPath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut dwSize: DWORD = 0;

    /* Check if NoOpen value exists */
    if RegGetValueW(
        hKey,
        core::ptr::null(),
        CPP_COM_WSTR_28002EF3350C.as_ptr(),
        RRF_RT_REG_SZ,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwSize,
    ) == ERROR_SUCCESS
    {
        (*this).m_bNoOpen = TRUE;
    }

    /* Check if there is a directly available execute key */
    dwSize = core::mem::size_of_val(&wszCmd) as DWORD;
    if RegGetValueW(
        hKey,
        CPP_COM_WSTR_A6B6F89A1F7F.as_ptr(),
        core::ptr::null(),
        RRF_RT_REG_SZ,
        core::ptr::null_mut(),
        wszCmd.as_mut_ptr() as *mut _,
        &mut dwSize,
    ) == ERROR_SUCCESS
    {
        /* Erase extra arguments */
        COpenWithList_GetPathFromCmd_body(this, wszPath.as_mut_ptr(), wszCmd.as_ptr());

        /* Add application */
        let pApp: *mut SApp =
            COpenWithList_AddInternal_body(this, PathFindFileNameW(wszPath.as_ptr()));

        if !pApp.is_null() {
            StringCbCopyW(
                (*pApp).wszCmd.as_mut_ptr(),
                core::mem::size_of_val(&(*pApp).wszCmd),
                wszCmd.as_ptr(),
            );
            COpenWithList_SetRecommended_body(this, pApp);
        }
    }
}

pub unsafe fn COpenWithList_LoadRecommendedFromHKCR_body(this: *mut COpenWithList, pwszExt: LPCWSTR) -> () {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubkey: HKEY = core::ptr::null_mut();
    let mut wszBuf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut wszBuf2: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut dwSize: u32;

    if RegOpenKeyExW(
        HKEY_CLASSES_ROOT,
        pwszExt,
        0,
        KEY_READ,
        &mut hKey as *mut HKEY,
    ) != ERROR_SUCCESS
    {
        StringCbPrintfW!(
            wszBuf.as_mut_ptr(),
            (MAX_PATH as usize) * 2,
            CPP_COM_WSTR_F38FD69158A5.as_ptr(),
            pwszExt,
        );
        if RegOpenKeyExW(
            HKEY_CLASSES_ROOT,
            wszBuf.as_ptr(),
            0,
            KEY_READ,
            &mut hKey as *mut HKEY,
        ) != ERROR_SUCCESS
        {
            return;
        }
    }

    /* Load programs referenced from HKCR\ProgId */
    dwSize = core::mem::size_of_val(&wszBuf) as u32;
    if RegGetValueW(
        hKey,
        core::ptr::null(),
        CPP_COM_WSTR_96A296D224F2.as_ptr(),
        RRF_RT_REG_SZ,
        core::ptr::null_mut(),
        wszBuf.as_mut_ptr().cast(),
        &mut dwSize as *mut u32,
    ) == ERROR_SUCCESS
        && RegOpenKeyExW(
            HKEY_CLASSES_ROOT,
            wszBuf.as_ptr(),
            0,
            KEY_READ,
            &mut hSubkey as *mut HKEY,
        ) == ERROR_SUCCESS
    {
        COpenWithList_LoadFromProgIdKey_body(this, hSubkey, pwszExt);
        RegCloseKey(hSubkey);
    } else {
        COpenWithList_LoadFromProgIdKey_body(this, hKey, pwszExt);
    }

    /* Load items from HKCR\Ext\OpenWithList */
    COpenWithList_LoadAppList_body(this, hKey);

    /* Load items from HKCR\Ext\OpenWithProgIDs */
    if RegOpenKeyExW(
        hKey,
        CPP_COM_WSTR_9D64E8A77A1C.as_ptr(),
        0,
        KEY_READ,
        &mut hSubkey as *mut HKEY,
    ) == ERROR_SUCCESS
    {
        COpenWithList_LoadProgIdList_body(this, hSubkey, pwszExt);
        RegCloseKey(hSubkey);
    }

    /* Load additional items from referenced PerceivedType */
    dwSize = core::mem::size_of_val(&wszBuf) as u32;
    if RegGetValueW(
        hKey,
        core::ptr::null(),
        CPP_COM_WSTR_714C3CA85096.as_ptr(),
        RRF_RT_REG_SZ,
        core::ptr::null_mut(),
        wszBuf.as_mut_ptr().cast(),
        &mut dwSize as *mut u32,
    ) == ERROR_SUCCESS
    {
        StringCbPrintfW!(
            wszBuf2.as_mut_ptr(),
            (MAX_PATH as usize) * 2,
            CPP_COM_WSTR_F38FD69158A5.as_ptr(),
            wszBuf.as_ptr(),
        );
        if RegOpenKeyExW(
            HKEY_CLASSES_ROOT,
            wszBuf2.as_ptr(),
            0,
            KEY_READ | KEY_WRITE,
            &mut hSubkey as *mut HKEY,
        ) == ERROR_SUCCESS
        {
            COpenWithList_LoadAppList_body(this, hSubkey);
            RegCloseKey(hSubkey);
        }
    }

    /* Close the key */
    RegCloseKey(hKey);
}

pub unsafe fn COpenWithList_LoadRecommendedFromHKCU_body(this: *mut COpenWithList, pwszExt: LPCWSTR) -> () {
    let mut wszBuf: [u16; MAX_PATH as usize] = unsafe { core::mem::zeroed() };
    let mut hKey: HKEY = core::ptr::null_mut();

    StringCbPrintfW!(
        wszBuf.as_mut_ptr(),
        core::mem::size_of_val(&wszBuf) as usize,
        CPP_COM_WSTR_C45BAD4AD59D.as_ptr(),
        pwszExt,
    );

    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        wszBuf.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    ) == ERROR_SUCCESS
    {
        COpenWithList_LoadMRUList_body(this, hKey);
        COpenWithList_LoadProgIdList_body(this, hKey, pwszExt);

        let mut cbBuf: u32 = core::mem::size_of_val(&wszBuf) as u32;
        if RegGetValueW(
            hKey,
            core::ptr::null(),
            CPP_COM_WSTR_0C7319B93B0B.as_ptr(),
            RRF_RT_REG_SZ,
            core::ptr::null_mut(),
            wszBuf.as_mut_ptr().cast::<core::ffi::c_void>(),
            core::ptr::addr_of_mut!(cbBuf),
        ) == ERROR_SUCCESS
        {
            let pApp = COpenWithList_Find_body(this, wszBuf.as_ptr());
            if !pApp.is_null() {
                COpenWithList_SetRecommended_body(this, pApp);
            }
        }

        RegCloseKey(hKey);
    }
}

pub unsafe fn COpenWithList_AddAppToMRUList_body(this: *mut COpenWithList, pApp: *mut SApp, pwszFilename: LPCWSTR) -> BOOL {
    let mut wszBuf: [u16; 100] = [0u16; 100];
    let pwszExt: PCWSTR;
    let mut hKey: HKEY = core::ptr::null_mut();

    pwszExt = PathFindExtensionW(pwszFilename);
    if *pwszExt.add(0 as usize) == 0u16 {
        return FALSE;
    }

    if StringCbPrintfW!(wszBuf.as_mut_ptr(), core::mem::size_of_val(&wszBuf), CPP_COM_WSTR_C45BAD4AD59D.as_ptr(), pwszExt) < 0 {
        return FALSE;
    }

    if RegCreateKeyExW(HKEY_CURRENT_USER, wszBuf.as_ptr(),
                              0, core::ptr::null_mut(),
                              0, KEY_WRITE | KEY_READ,
                              core::ptr::null_mut(), &mut hKey, core::ptr::null_mut()) != ERROR_SUCCESS {
        return FALSE;
    }

    let hList = COpenWithList_OpenMRUList_body(this, hKey);
    if !hList.is_null() {
        AddMRUStringW(hList, (*pApp).wszFilename.as_ptr());
        (*pApp).bMRUList = TRUE;
        FreeMRUList(hList);
    }

    RegCloseKey(hKey);
    TRUE
}

pub unsafe fn COpenWithList_SetDefaultHandler_body(this: *mut COpenWithList, pApp: *mut SApp, pwszFilename: LPCWSTR) -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut wszBuf: [u16; 256 as usize] = [0u16; 256 as usize];

    let pwszExt = PathFindExtensionW(pwszFilename);
    if *pwszExt.offset(0 as isize) == 0
        || *pwszExt.offset(1 as isize) == 0
    {
        return FALSE;
    }

    if RegCreateKeyExW(
        HKEY_CLASSES_ROOT,
        pwszExt,
        0,
        core::ptr::null_mut(),
        0,
        KEY_READ | KEY_WRITE,
        core::ptr::null_mut(),
        &mut hKey,
        core::ptr::null_mut(),
    ) != ERROR_SUCCESS
    {
        return FALSE;
    }

    let mut dwSize: u32 = core::mem::size_of_val(&wszBuf) as u32;
    let lResult = RegGetValueW(
        hKey,
        core::ptr::null_mut(),
        CPP_COM_WSTR_96A296D224F2.as_ptr(),
        RRF_RT_REG_SZ,
        core::ptr::null_mut(),
        wszBuf.as_mut_ptr() as *mut _,
        &mut dwSize,
    );

    if lResult == ERROR_FILE_NOT_FOUND {
        StringCbPrintfW!(
            wszBuf.as_mut_ptr(),
            core::mem::size_of_val(&wszBuf) as usize,
            CPP_COM_WSTR_751E73E72EF6.as_ptr(),
            pwszExt.offset(1 as isize),
        );

        let len = wcslen(wszBuf.as_ptr());
        if RegSetValueExW(
            hKey,
            CPP_COM_WSTR_96A296D224F2.as_ptr(),
            0,
            REG_SZ,
            wszBuf.as_ptr() as *const u8,
            (len as u32)
                .wrapping_add(1)
                .wrapping_mul(core::mem::size_of::<u16>() as u32),
        ) != ERROR_SUCCESS
        {
            RegCloseKey(hKey);
            return FALSE;
        }
    } else if lResult != ERROR_SUCCESS {
        RegCloseKey(hKey);
        return FALSE;
    }

    RegCloseKey(hKey);

    if RegCreateKeyExW(
        HKEY_CLASSES_ROOT,
        wszBuf.as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hKey,
        core::ptr::null_mut(),
    ) != ERROR_SUCCESS
    {
        return FALSE;
    }

    StringCbPrintfW!(
        wszBuf.as_mut_ptr(),
        core::mem::size_of_val(&wszBuf) as usize,
        CPP_COM_WSTR_E8020B1077FF.as_ptr(),
        (*pApp).wszFilename.as_ptr(),
    );

    let mut hSrcKey: HKEY = core::ptr::null_mut();
    if RegOpenKeyExW(
        HKEY_CLASSES_ROOT,
        wszBuf.as_ptr(),
        0,
        KEY_READ,
        &mut hSrcKey,
    ) != ERROR_SUCCESS
    {
        RegCloseKey(hKey);
        return FALSE;
    }

    let mut hDestKey: HKEY = core::ptr::null_mut();
    if RegCreateKeyExW(
        hKey,
        CPP_COM_WSTR_EE89605B31B0.as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hDestKey,
        core::ptr::null_mut(),
    ) != ERROR_SUCCESS
    {
        RegCloseKey(hSrcKey);
        RegCloseKey(hKey);
        return FALSE;
    }

    let mut hDefIconKey: HKEY = core::ptr::null_mut();
    if RegCreateKeyExW(
        hKey,
        CPP_COM_WSTR_3BE8B25B7ED1.as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hDefIconKey,
        core::ptr::null_mut(),
    ) == ERROR_SUCCESS
    {
        RegSetString(
            hDefIconKey,
            core::ptr::null_mut(),
            (*pApp).wszFilename.as_ptr(),
            REG_EXPAND_SZ,
        );
        RegCloseKey(hDefIconKey);
        SHChangeNotify(
            SHCNE_ASSOCCHANGED as LONG,
            SHCNF_FLUSHNOWAIT,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    let Result = SHCopyKeyW(
        hSrcKey,
        core::ptr::null_mut(),
        hDestKey,
        0,
    );
    let mut hAppKey: HKEY = core::ptr::null_mut();
    StringCbPrintfW!(
        wszBuf.as_mut_ptr(),
        core::mem::size_of_val(&wszBuf) as usize,
        CPP_COM_WSTR_E2341F1029CA.as_ptr(),
        (*pApp).wszFilename.as_ptr(),
    );
    if Result == ERROR_SUCCESS as DWORD
        && RegOpenKeyExW(
            HKEY_CLASSES_ROOT,
            wszBuf.as_ptr(),
            0,
            KEY_READ,
            &mut hAppKey,
        ) == ERROR_SUCCESS
    {
        let cnt = wszBuf.len() as u32; // _countof macro replacement
        if HCR_GetDefaultVerbW(
            hAppKey.cast(),
            core::ptr::null_mut(),
            wszBuf.as_mut_ptr(),
            cnt,
        ) != 0
            && *wszBuf.as_ptr() != 0
        {
            RegSetString(
                hDestKey,
                core::ptr::null_mut(),
                wszBuf.as_ptr(),
                REG_SZ,
            );
        }
        RegCloseKey(hAppKey);
    }
    RegCloseKey(hDestKey);
    RegCloseKey(hSrcKey);
    RegCloseKey(hKey);

    if Result != ERROR_SUCCESS as DWORD {
        return FALSE;
    }

    TRUE
}
