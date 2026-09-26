// rustconv-assembly: class:_FILE_TYPE_ENTRY
use crate::prelude::*;
use crate::win32::max;
// Deterministic COM scaffold for _FILE_TYPE_ENTRY; method bodies are generated separately.
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
pub struct _FILE_TYPE_ENTRY {
    pub FileExtension: [WCHAR; 33],
    pub FileDescription: [WCHAR; 100],
    pub ClassKey: [WCHAR; 260],
    pub EditFlags: DWORD,
    pub AppName: [WCHAR; 64],
    pub hIconSmall: HICON,
    pub ProgramPath: [WCHAR; 260],
    pub IconPath: [WCHAR; 260],
    pub nIconIndex: INT,
}

pub type FILE_TYPE_ENTRY = _FILE_TYPE_ENTRY;
pub type PFILE_TYPE_ENTRY = *mut _FILE_TYPE_ENTRY;

/// `new _FILE_TYPE_ENTRY` — operator new, the vptr store, then the constructor body.
pub unsafe fn _FILE_TYPE_ENTRY_new() -> *mut _FILE_TYPE_ENTRY {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<_FILE_TYPE_ENTRY>(),
    ) as *mut _FILE_TYPE_ENTRY;
    if this.is_null() { return core::ptr::null_mut(); }
    _FILE_TYPE_ENTRY_ctor_body(this);
    this
}

/// `delete` — the destructor body, then the matching free. Pairs with
/// `_FILE_TYPE_ENTRY_new`; freeing through any other allocator is a heap bug.
pub unsafe fn _FILE_TYPE_ENTRY_delete(this: *mut _FILE_TYPE_ENTRY) {
    if this.is_null() { return; }
    _FILE_TYPE_ENTRY_dtor_body(this);
    HeapFree(GetProcessHeap(), 0, this);
}

// Deterministic C++ support closure selected from source identifiers.

#[repr(C)]
pub struct ACTION_DIALOG {
    pub hwndLB: HWND,
    pub pG: PFILE_TYPE_GLOBALS,
    pub pEntry: PFILE_TYPE_ENTRY,
    pub szAction: [WCHAR; 64],
    pub szApp: [WCHAR; 260],
    pub bUseDDE: BOOL,
}
pub type PACTION_DIALOG = *mut ACTION_DIALOG;

#[repr(C)]
pub struct _FILE_TYPE_GLOBALS {
    pub himlSmall: HIMAGELIST,
    pub IconSize: UINT,
    pub hDefExtIconSmall: HICON,
    pub hOpenWithImage: HBITMAP,
    pub hHeap: HANDLE,
    pub NoneString: [WCHAR; 42],
    pub SortCol: INT8,
    pub SortReverse: INT8,
    pub Restricted: UINT,
}
pub type FILE_TYPE_GLOBALS = _FILE_TYPE_GLOBALS;
pub type PFILE_TYPE_GLOBALS = *mut _FILE_TYPE_GLOBALS;

pub const IDC_ACTION_ACTION: UINT = 14001;

pub const IDC_ACTION_APP: UINT = 14002;

pub const IDC_ACTION_BROWSE: UINT = 14003;

pub const IDC_ACTION_USE_DDE: UINT = 14004;

pub const IDC_EDITTYPE_CHANGE_ICON: UINT = 14003;

pub const IDC_EDITTYPE_CONFIRM_OPEN: UINT = 14009;

pub const IDC_EDITTYPE_EDIT_BUTTON: UINT = 14006;

pub const IDC_EDITTYPE_ICON: UINT = 14001;

pub const IDC_EDITTYPE_LISTBOX: UINT = 14004;

pub const IDC_EDITTYPE_NEW: UINT = 14005;

pub const IDC_EDITTYPE_REMOVE: UINT = 14007;

pub const IDC_EDITTYPE_SAME_WINDOW: UINT = 14011;

pub const IDC_EDITTYPE_SET_DEFAULT: UINT = 14008;

pub const IDC_EDITTYPE_SHOW_EXT: UINT = 14010;

pub const IDC_EDITTYPE_TEXT: UINT = 14002;

pub const IDC_FILETYPES_ADVANCED: UINT = 14008;

pub const IDC_FILETYPES_APPNAME: UINT = 14005;

pub const IDC_FILETYPES_CHANGE: UINT = 14006;

pub const IDC_FILETYPES_DELETE: UINT = 14002;

pub const IDC_FILETYPES_DESCRIPTION: UINT = 14007;

pub const IDC_FILETYPES_DETAILS_GROUPBOX: UINT = 14003;

pub const IDC_FILETYPES_ICON: UINT = 14009;

pub const IDC_FILETYPES_LISTVIEW: UINT = 14000;

pub const IDC_FILETYPES_NEW: UINT = 14001;

pub const IDC_NEWEXT_ADVANCED: UINT = 14002;

pub const IDC_NEWEXT_ASSOC: UINT = 14004;

pub const IDC_NEWEXT_COMBOBOX: UINT = 14003;

pub const IDC_NEWEXT_EDIT: UINT = 14001;

pub const IDD_ACTION: UINT = 37;

pub const IDD_EDITTYPE: UINT = 36;

pub const IDD_NEWEXTENSION: UINT = 28;

pub const IDI_SHELL_DOCUMENT: UINT = 1;

pub const IDI_SHELL_EXE: UINT = 3;

pub const IDS_ACTION_EXISTS: UINT = 30526;

pub const IDS_COLUMN_EXTENSION: UINT = 151;

pub const IDS_EDITING_ACTION: UINT = 30528;

pub const IDS_EXE_FILTER: UINT = 30527;

pub const IDS_FILE_DETAILS: UINT = 175;

pub const IDS_FILE_DETAILSADV: UINT = 176;

pub const IDS_FILE_TYPES: UINT = 174;

pub const IDS_INVALID_PROGRAM: UINT = 30524;

pub const IDS_NEWEXT_ADVANCED_LEFT: UINT = 10109;

pub const IDS_NEWEXT_ADVANCED_RIGHT: UINT = 10110;

pub const IDS_NEWEXT_ALREADY_ASSOC: UINT = 10100;

pub const IDS_NEWEXT_EXT_IN_USE: UINT = 10101;

pub const IDS_NEWEXT_NEW: UINT = 10105;

pub const IDS_NEWEXT_SPECIFY_EXT: UINT = 10086;

pub const IDS_NONE: UINT = 161;

pub const IDS_OPEN_WITH: UINT = 142;

pub const IDS_REMOVE_ACTION: UINT = 30525;

pub const IDS_REMOVE_EXT: UINT = 30522;

pub const IDS_SPECIFY_ACTION: UINT = 30523;

pub const LISTBOX_MARGIN: UINT = 2;

pub const SUPPORT_EXTENSIONWITHOUTPROGID: UINT = 1;

pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFF_FFFF;

pub const BM_GETCHECK: DWORD = 240;

pub const BM_SETCHECK: DWORD = 241;

pub const CB_ADDSTRING: DWORD = 323;

pub const CB_SETCURSEL: DWORD = 334;

pub const COLOR_HIGHLIGHT: DWORD = 13;

pub const COLOR_HIGHLIGHTTEXT: DWORD = 14;

pub const COLOR_WINDOW: DWORD = 5;

pub const COLOR_WINDOWTEXT: DWORD = 8;

pub const DT_LEFT: DWORD = 0;

pub const DT_NOPREFIX: DWORD = 2048;

pub const DT_SINGLELINE: DWORD = 32;

pub const DT_VCENTER: DWORD = 4;

pub const EM_SETLIMITTEXT: DWORD = 197;

pub const EM_SETSEL: DWORD = 177;

pub const ERROR_BAD_FORMAT: DWORD = 11;

pub const ERROR_FILE_NOT_FOUND: DWORD = 2;

pub const FTA_Exclude: DWORD = 0x00000001;

pub const FTA_NoEdit: DWORD = 0x00000008;

pub const FTA_NoEditDesc: DWORD = 0x00000100;

pub const FTA_NoEditDflt: DWORD = 0x00000400;

pub const FTA_NoEditIcon: DWORD = 0x00000200;

pub const FTA_NoEditVerb: DWORD = 0x00000040;

pub const FTA_NoNewVerb: DWORD = 0x00000020;

pub const FTA_NoRemove: DWORD = 0x00000010;

pub const FTA_NoRemoveVerb: DWORD = 0x00000080;

pub const FTA_OpenIsSafe: DWORD = 0x00010000;

pub const FW_BOLD: DWORD = 700;

pub const IDCANCEL: DWORD = 2;

pub const IDNO: DWORD = 7;

pub const IDOK: DWORD = 1;

pub const IDYES: DWORD = 6;

pub const ILC_COLOR32: DWORD = 0x20;

pub const ILC_MASK: DWORD = 0x1;

pub const IMAGE_BITMAP: DWORD = 0;

pub const IMAGE_ICON: DWORD = 1;

pub const KEY_QUERY_VALUE: DWORD = 1;

pub const KEY_READ: DWORD = 0x20019;

pub const KEY_WRITE: DWORD = 0x20006;

pub const LBN_DBLCLK: DWORD = 2;

pub const LBN_SELCHANGE: DWORD = 1;

pub const LB_ADDSTRING: DWORD = 384;

pub const LB_DELETESTRING: DWORD = 386;

pub const LB_FINDSTRING: DWORD = 399;

pub const LB_GETCOUNT: DWORD = 395;

pub const LB_GETCURSEL: DWORD = 392;

pub const LB_GETTEXT: DWORD = 393;

pub const LB_INSERTSTRING: DWORD = 385;

pub const LB_SETCURSEL: DWORD = 390;

pub const LVCF_FMT: DWORD = 0x1;

pub const LVCF_SUBITEM: DWORD = 0x8;

pub const LVCF_TEXT: DWORD = 0x4;

pub const LVCF_WIDTH: DWORD = 0x2;

pub const LVFI_STRING: DWORD = 0x0002;

pub const LVIF_DI_SETITEM: DWORD = 0x1000;

pub const LVIF_IMAGE: DWORD = 0x2;

pub const LVIF_PARAM: DWORD = 0x4;

pub const LVIF_TEXT: DWORD = 0x1;

pub const LVIS_FOCUSED: DWORD = 0x1;

pub const LVIS_SELECTED: DWORD = 0x2;

pub const LVNI_SELECTED: DWORD = 0x2;

pub const LVSIL_SMALL: DWORD = 1;

pub const LVS_EX_FULLROWSELECT: DWORD = 0x20;

pub const LVS_EX_LABELTIP: DWORD = 0x4000;

pub const MB_ICONERROR: DWORD = 16;

pub const MB_ICONINFORMATION: DWORD = 64;

pub const MB_ICONQUESTION: DWORD = 32;

pub const MB_ICONWARNING: DWORD = 0x30;

pub const MB_YESNO: DWORD = 4;

pub const OAIF_FORCE_REGISTRATION: DWORD = 8;

pub const OAIF_REGISTER_EXT: DWORD = 2;

pub const ODS_FOCUS: DWORD = 16;

pub const ODS_SELECTED: DWORD = 1;

pub const OFN_ENABLESIZING: DWORD = 0x800000;

pub const OFN_EXPLORER: DWORD = 0x80000;

pub const OFN_FILEMUSTEXIST: DWORD = 0x1000;

pub const OFN_HIDEREADONLY: DWORD = 4;

pub const OFN_PATHMUSTEXIST: DWORD = 0x800;

pub const RDW_ALLCHILDREN: DWORD = 128;

pub const RDW_ERASE: DWORD = 4;

pub const RDW_FRAME: DWORD = 1024;

pub const RDW_INVALIDATE: DWORD = 1;

pub const RDW_UPDATENOW: DWORD = 256;

pub const REG_BINARY: DWORD = 3;

pub const REG_DWORD: DWORD = 4;

pub const REG_EXPAND_SZ: DWORD = 2;

pub const REG_SZ: DWORD = 1;

pub const SHCNE_ASSOCCHANGED: DWORD = 0x08000000;

pub const SHCNF_FLUSHNOWAIT: DWORD = 0x3000;

pub const SHGFI_DISPLAYNAME: DWORD = 512;

pub const SHGFI_TYPENAME: DWORD = 1024;

pub const SHGFI_USEFILEATTRIBUTES: DWORD = 16;

pub const SM_CXICON: DWORD = 11;

pub const SM_CXSMICON: DWORD = 49;

pub const SM_CYICON: DWORD = 12;

pub const SM_CYSMICON: DWORD = 50;

pub const SM_CYVSCROLL: DWORD = 20;

pub const STM_GETICON: DWORD = 369;

pub const STM_SETICON: DWORD = 368;

pub const STM_SETIMAGE: DWORD = 370;

pub const SWP_NOACTIVATE: DWORD = 16;

pub const SWP_NOZORDER: DWORD = 4;

pub const SW_HIDE: DWORD = 0;

pub const SW_SHOWNOACTIVATE: DWORD = 4;

pub const VK_DELETE: DWORD = 46;

pub const VK_F5: DWORD = 0x74;

pub const WM_COMMAND: DWORD = 273;

pub const WM_DESTROY: DWORD = 2;

pub const WM_DRAWITEM: DWORD = 43;

pub const WM_GETFONT: DWORD = 49;

pub const WM_INITDIALOG: DWORD = 272;

pub const WM_KEYDOWN: DWORD = 256;

pub const WM_MEASUREITEM: DWORD = 44;

pub const WM_SETREDRAW: DWORD = 11;

pub const VERBKEY_CCHMAX: INT = 64;

#[repr(C)]
pub struct LVITEMW {
    pub mask: UINT,
    pub iItem: i32,
    pub iSubItem: i32,
    pub state: UINT,
    pub stateMask: UINT,
    pub pszText: LPWSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub lParam: LPARAM,
    pub iIndent: i32,
    pub iGroupId: i32,
    pub cColumns: UINT,
    pub puColumns: PUINT,
}

#[repr(C)]
pub struct MEASUREITEMSTRUCT {
    pub CtlType: UINT,
    pub CtlID: UINT,
    pub itemID: UINT,
    pub itemWidth: UINT,
    pub itemHeight: UINT,
    pub itemData: ULONG_PTR,
}
pub type LPMEASUREITEMSTRUCT = *mut MEASUREITEMSTRUCT;

macro_rules! ASSERT { ($($arg:tt)*) => {{}}; }

macro_rules! LOWORD { ($value:expr) => { (($value as usize) & 0xffff) as UINT }; }

extern "system" {
    fn DestroyIcon(icon: HICON) -> BOOL;
    #[cfg_attr(target_pointer_width = "32", link_name = "GetWindowLongW")] fn GetWindowLongPtrW(window: HWND, index: INT) -> LONG_PTR;
    #[cfg_attr(target_pointer_width = "32", link_name = "SetWindowLongW")] fn SetWindowLongPtrW(window: HWND, index: INT, value: LONG_PTR) -> LONG_PTR;
    fn ExpandEnvironmentStringsW(source: LPCWSTR, dest: LPWSTR, size: DWORD) -> DWORD;
    fn ExtractIconExW(file: LPCWSTR, index: INT, large: *mut HICON, small: *mut HICON, icons: UINT) -> UINT;
    fn GetSystemMetrics(index: UINT) -> INT;
    fn MessageBoxW(owner: HWND, text: LPCWSTR, caption: LPCWSTR, kind: UINT) -> INT;
    fn RegCloseKey(key: HKEY) -> LONG;
    #[link_name = "RegCreateKeyExW"] fn RegCreateKeyEx(key: HKEY, subkey: LPCWSTR, reserved: DWORD, class: LPWSTR, options: DWORD, access: REGSAM, security: LPVOID, result: PHKEY, disposition: LPDWORD) -> LONG;
    fn RegEnumKeyW(key: HKEY, index: DWORD, name: LPWSTR, capacity: DWORD) -> LONG;
    fn RegOpenKeyExW(key: HKEY, subkey: LPCWSTR, options: DWORD, access: REGSAM, result: PHKEY) -> LONG;
    fn RegQueryValueExW(key: HKEY, value: LPCWSTR, reserved: LPDWORD, kind: LPDWORD, data: PBYTE, bytes: LPDWORD) -> LONG;
    fn RegSetValueExW(key: HKEY, value: LPCWSTR, reserved: DWORD, kind: DWORD, data: *const BYTE, bytes: DWORD) -> LONG;
    fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID);
    fn lstrlenW(text: LPCWSTR) -> INT;
    fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT;
    fn StrChrW(text: LPCWSTR, ch: WCHAR) -> LPWSTR;
}

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
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

extern "system" {
    #[link_name = "SHFree"] fn cpp_com_SHFree(memory: LPVOID);
}
#[inline]
unsafe fn SHFree<T>(memory: *mut T) { cpp_com_SHFree(memory.cast()); }

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

extern "system" {
    fn HeapReAlloc(heap: HANDLE, flags: DWORD, memory: LPVOID, bytes: usize) -> LPVOID;
    fn GetModuleHandleW(name: LPCWSTR) -> HINSTANCE;
}

pub struct CStringW { buffer: LPWSTR, len: usize, capacity: usize }
impl CStringW {
    pub const fn new() -> Self { Self { buffer: core::ptr::null_mut(), len: 0, capacity: 0 } }
    pub unsafe fn from_resource(id: UINT) -> Self {
        let mut value = Self::new();
        let mut stack = [0u16; 256];
        let count = LoadStringW(
            GetModuleHandleW(core::ptr::null()), id, stack.as_mut_ptr(), stack.len() as INT,
        );
        if count > 0 { value.append_ptr_len(stack.as_ptr(), count as usize); }
        value
    }
    pub unsafe fn new_from_lpcwstr(text: LPCWSTR) -> Self {
        if (text as usize) <= 0xffff { return Self::from_resource(text as usize as UINT); }
        let mut value = Self::new(); value.append_ptr(text); value
    }
    pub fn as_ptr(&self) -> LPCWSTR {
        if self.buffer.is_null() { CPP_COM_EMPTY_WSTR.as_ptr() } else { self.buffer }
    }
    pub fn as_mut_ptr(&mut self) -> LPWSTR { self.as_ptr() as LPWSTR }
    unsafe fn reserve(&mut self, needed: usize) -> bool {
        if needed <= self.capacity { return true; }
        let capacity = needed.next_power_of_two().max(16);
        let bytes = capacity * core::mem::size_of::<WCHAR>();
        let next = if self.buffer.is_null() {
            HeapAlloc(GetProcessHeap(), 0, bytes)
        } else {
            HeapReAlloc(GetProcessHeap(), 0, self.buffer.cast(), bytes)
        } as LPWSTR;
        if next.is_null() { return false; }
        self.buffer = next; self.capacity = capacity; true
    }
    unsafe fn append_ptr_len(&mut self, text: LPCWSTR, count: usize) {
        if text.is_null() || !self.reserve(self.len + count + 1) { return; }
        let _ = memcpy(
            self.buffer.add(self.len), text,
            count * core::mem::size_of::<WCHAR>(),
        );
        self.len += count; *self.buffer.add(self.len) = 0;
    }
    unsafe fn append_ptr(&mut self, text: LPCWSTR) {
        if !text.is_null() { self.append_ptr_len(text, lstrlenW(text) as usize); }
    }
    unsafe fn append_char(&mut self, ch: WCHAR) { self.append_ptr_len(core::ptr::addr_of!(ch), 1); }
    pub unsafe fn concat(&mut self, text: LPCWSTR) { self.append_ptr(text); }
    pub unsafe fn assign(&mut self, text: LPCWSTR) {
        self.len = 0;
        if !self.buffer.is_null() { *self.buffer = 0; }
        self.append_ptr(text);
    }
    pub unsafe fn format(&mut self, format: LPCWSTR, arg: LPCWSTR) {
        let mut resource = [0u16; 256];
        let format = if (format as usize) <= 0xffff {
            LoadStringW(GetModuleHandleW(core::ptr::null()), format as usize as UINT,
                resource.as_mut_ptr(), resource.len() as INT);
            resource.as_ptr()
        } else { format };
        self.len = 0;
        if self.reserve(512) {
            let _ = StringCbPrintfW!(self.buffer, self.capacity * 2, format, arg);
            self.len = lstrlenW(self.buffer) as usize;
        }
    }
    pub unsafe fn format_message(&mut self, arg: LPCWSTR) {
        let mut format = [0u16; 512];
        let count = lstrlenW(self.as_ptr()).max(0) as usize;
        let count = count.min(format.len() - 1);
        let _ = memcpy(format.as_mut_ptr(), self.as_ptr(), count * 2);
        self.format(format.as_ptr(), arg);
    }
}
impl core::ops::AddAssign<LPCWSTR> for CStringW {
    fn add_assign(&mut self, rhs: LPCWSTR) { unsafe { self.append_ptr(rhs) } }
}
impl core::ops::AddAssign<WCHAR> for CStringW {
    fn add_assign(&mut self, rhs: WCHAR) { unsafe { self.append_char(rhs) } }
}
impl Drop for CStringW {
    fn drop(&mut self) {
        unsafe {
            if !self.buffer.is_null() { HeapFree(GetProcessHeap(), 0, self.buffer); }
        }
    }
}
static CPP_COM_EMPTY_WSTR: [WCHAR; 1] = [0];

extern "C" {
    static mut shell32_hInstance: HINSTANCE;
}

use core::ffi::c_void;

pub unsafe fn _FILE_TYPE_ENTRY_ctor_body(this: *mut _FILE_TYPE_ENTRY) -> () {}

pub unsafe fn _FILE_TYPE_ENTRY_dtor_body(this: *mut _FILE_TYPE_ENTRY) -> () {}

pub unsafe fn _FILE_TYPE_ENTRY_IsExtension_body(this: *mut _FILE_TYPE_ENTRY) -> bool {
    (*this).FileExtension[0] == b'.' as u16
}

pub unsafe fn _FILE_TYPE_ENTRY_GetAssocForDisplay_body(this: *mut _FILE_TYPE_ENTRY) -> LPCWSTR {
    return (*this).FileExtension.as_ptr().add(((*this).FileExtension[0] == '.' as u16) as usize);
}

pub unsafe fn _FILE_TYPE_ENTRY_InvalidateTypeName_body(this: *mut _FILE_TYPE_ENTRY) -> () {
    (*this).FileDescription[1] = UNICODE_NULL;
    (*this).FileDescription[0] = UNICODE_NULL;
}

pub unsafe fn _FILE_TYPE_ENTRY_InvalidateDefaultApp_body(this: *mut _FILE_TYPE_ENTRY) -> () {
    unsafe {
        (*this).ProgramPath[0] = UNICODE_NULL;
        (*this).ProgramPath[1] = UNICODE_NULL;
        (*this).AppName[0] = UNICODE_NULL;
        (*this).AppName[1] = UNICODE_NULL;
    }
}

pub unsafe fn _FILE_TYPE_ENTRY_Initialize_body(this: *mut _FILE_TYPE_ENTRY) -> () {
    (*this).ClassKey[0] = UNICODE_NULL;
    (*this).IconPath[0] = UNICODE_NULL;
    (*this).nIconIndex = 0;
    _FILE_TYPE_ENTRY_InvalidateTypeName_body(this);
    _FILE_TYPE_ENTRY_InvalidateDefaultApp_body(this);
}

pub unsafe fn _FILE_TYPE_ENTRY_DestroyIcons_body(this: *mut _FILE_TYPE_ENTRY) -> () {
    if !(*this).hIconSmall.is_null() {
        DestroyIcon((*this).hIconSmall);
    }
    (*this).hIconSmall = core::ptr::null_mut();
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *     'File Types' tab property sheet of Folder Options
//  *
//  * Copyright 2007 Johannes Anderwald <johannes.anderwald@reactos.org>
//  * Copyright 2016-2018 Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
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
// #include "precomp.h"
// #include <atlpath.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL (fprop);
//
// // rundll32.exe shell32.dll,Options_RunDLL 0
//
// /////////////////////////////////////////////////////////////////////////////
//
// extern "C" BOOL PathIsExeW(LPCWSTR lpszPath);
//
// #define FTA_MODIFYMASK (FTA_OpenIsSafe) // Bits modified by EditTypeDlg
// #define NOASSOCRESID IDI_SHELL_DOCUMENT
// #define SUPPORT_EXTENSIONWITHOUTPROGID 1 // NT5 does not support these but NT6 does
//
// #define ASSOC_CCHMAX (32 + 1) // Extension or protocol (INTERNET_MAX_SCHEME_LENGTH)
// #define TYPENAME_CCHMAX max(100, RTL_FIELD_SIZE(SHFILEINFOA, szTypeName))
// #define ICONLOCATION_CCHMAX (MAX_PATH + 1 + 11)
//
// typedef struct _FILE_TYPE_ENTRY
// {
//     WCHAR FileExtension[ASSOC_CCHMAX];
//     WCHAR FileDescription[TYPENAME_CCHMAX];
//     WCHAR ClassKey[MAX_PATH];
//     DWORD EditFlags;
//     WCHAR AppName[64];
//     HICON hIconSmall;
//     WCHAR ProgramPath[MAX_PATH];
//     WCHAR IconPath[MAX_PATH];
//     INT nIconIndex;
//
//     bool IsExtension() const
//     {
//         return FileExtension[0] == '.';
//     }
//     LPCWSTR GetAssocForDisplay() const
//     {
//         return FileExtension + (FileExtension[0] == '.');
//     }
//     void InvalidateTypeName()
//     {
//         FileDescription[0] = FileDescription[1] = UNICODE_NULL;
//     }
//     void InvalidateDefaultApp()
//     {
//         ProgramPath[0] = ProgramPath[1] = AppName[0] = AppName[1] = UNICODE_NULL;
//     }
//     void Initialize()
//     {
//         ClassKey[0] = UNICODE_NULL;
//         IconPath[0] = UNICODE_NULL;
//         nIconIndex = 0;
//         InvalidateTypeName();
//         InvalidateDefaultApp();
//     }
//     void DestroyIcons()
//     {
//         if (hIconSmall)
//             DestroyIcon(hIconSmall);
//         hIconSmall = NULL;
//     }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
