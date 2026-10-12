use crate::wordpad::print::close_preview;
use crate::wordpad::print::dialog_print;
use crate::wordpad::print::dialog_printsetup;
use crate::wordpad::print::get_default_printer_opts;
use crate::wordpad::print::init_preview;
use crate::wordpad::print::preview_command;
use crate::wordpad::print::preview_isactive;
use crate::wordpad::print::preview_proc;
use crate::wordpad::print::print_quick;
use crate::wordpad::print::redraw_ruler;
use crate::wordpad::registry::reg_formatindex;
use crate::wordpad::registry::registry_read_filelist;
use crate::wordpad::registry::registry_read_formatopts_all;
use crate::wordpad::registry::registry_read_maximized;
use crate::wordpad::registry::registry_read_options;
use crate::wordpad::registry::registry_read_winrect;
use crate::wordpad::registry::registry_set_filelist;
use crate::wordpad::registry::registry_set_formatopts_all;
use crate::wordpad::registry::registry_set_options;
use crate::wordpad::print::ruler_proc;
use crate::wordpad::olecallback::setup_richedit_olecallback;
use crate::wordpad::print::target_device;
use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn _errno() -> *mut ::core::ffi::c_int;
}
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const RICHEDIT_CLASS20W: [WCHAR; 12] = [
    'R' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'h' as i32 as WCHAR,
    'E' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    't' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    '0' as i32 as WCHAR,
    'W' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

pub const REBARCLASSNAMEW: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"R\0e\0B\0a\0r\0W\0i\0n\0d\0o\0w\x003\x002\0\0\0",
    )
};

pub const WC_COMBOBOXEXW: [::core::ffi::c_ushort; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_ushort; 13]>(
        *b"C\0o\0m\0b\0o\0B\0o\0x\0E\0x\x003\x002\0\0\0",
    )
};

pub const WC_STATICW: [::core::ffi::c_ushort; 7] = unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_ushort; 7]>(*b"S\0t\0a\0t\0i\0c\0\0\0")
};

pub const FINDMSGSTRINGW: [::core::ffi::c_ushort; 20] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_ushort; 20]>(
        *b"c\0o\0m\0m\0d\0l\0g\0_\0F\0i\0n\0d\0R\0e\0p\0l\0a\0c\0e\0\0\0",
    )
};

static mut wszAppTitle: [WCHAR; 8] = [
    'W' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'p' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut wszMainWndClass: [WCHAR; 11] = [
    'W' as i32 as WCHAR,
    'O' as i32 as WCHAR,
    'R' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'A' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'T' as i32 as WCHAR,
    'O' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut stringFormat: [WCHAR; 4] = [
    '%' as i32 as WCHAR,
    '2' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    '\0' as i32 as WCHAR,
];

#[no_mangle]
pub static mut wszPreviewWndClass: [WCHAR; 11] = [
    'P' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'P' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut hMainWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hEditorWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hFindWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hColorPopupMenu: HMENU = ::core::ptr::null::<HMENU__>() as *mut HMENU__;

static mut ID_FINDMSGSTRING: UINT = 0;

static mut wordWrap: [DWORD; 2] = [0; 2];

static mut barState: [DWORD; 2] = [0; 2];

static mut fileFormat: WPARAM = SF_RTF as WPARAM;

static mut wszFileName: [WCHAR; 260] = [0; 260];

static mut wszFilter: [WCHAR; 1043] = [0; 1043];

static mut wszDefaultFileName: [WCHAR; 255] = [0; 255];

static mut wszSaveChanges: [WCHAR; 255] = [0; 255];

static mut units_cmW: [WCHAR; 255] = [0; 255];

static mut units_inW: [WCHAR; 255] = [0; 255];

static mut units_inchW: [WCHAR; 255] = [0; 255];

static mut units_ptW: [WCHAR; 255] = [0; 255];

static mut textcolors: [COLORREF; 16] = [
    0 as ::core::ffi::c_long as COLORREF,
    0x80 as ::core::ffi::c_long as COLORREF,
    0x8000 as ::core::ffi::c_long as COLORREF,
    0x8080 as ::core::ffi::c_long as COLORREF,
    0x800000 as ::core::ffi::c_long as COLORREF,
    0x800080 as ::core::ffi::c_long as COLORREF,
    0x808000 as ::core::ffi::c_long as COLORREF,
    0x808080 as ::core::ffi::c_long as COLORREF,
    0xc0c0c0 as ::core::ffi::c_long as COLORREF,
    0xff as ::core::ffi::c_long as COLORREF,
    0xff00 as ::core::ffi::c_long as COLORREF,
    0xffff as ::core::ffi::c_long as COLORREF,
    0xff0000 as ::core::ffi::c_long as COLORREF,
    0xff00ff as ::core::ffi::c_long as COLORREF,
    0xffff00 as ::core::ffi::c_long as COLORREF,
    0xffffff as ::core::ffi::c_long as COLORREF,
];


pub type UNIT = i32;
pub const UNIT_CM: UNIT = 0;
pub const UNIT_INCH: UNIT = 1;
pub const UNIT_PT: UNIT = 2;

#[repr(C)]
pub struct FINDREPLACE_custom {
    pub endPos: i32,
    pub wrapped: BOOL,
    pub findBuffer: [u16; 128],
}

unsafe extern "C" fn DoLoadStrings() {
    let mut p: LPWSTR = &raw mut wszFilter as LPWSTR;
    static mut files_rtf: [WCHAR; 6] = [0x002Au16, 0x002E, 0x0072, 0x0074, 0x0066, 0x0000];
    static mut files_txt: [WCHAR; 6] = [0x002Au16, 0x002E, 0x0074, 0x0078, 0x0074, 0x0000];
    static mut files_all: [WCHAR; 4] = [0x002Au16, 0x002E, 0x002A, 0x0000];

    let mut hInstance: HINSTANCE = GetModuleHandleW(core::ptr::null()) as HINSTANCE;

    LoadStringW(hInstance, STRING_RICHTEXT_FILES_RTF, p, (MAX_STRING_LEN as i32));
    p = p.offset((lstrlenW(p as LPCWSTR) + 1) as isize);
    lstrcpyW(p, &raw const files_rtf as LPCWSTR);
    p = p.offset((lstrlenW(p as LPCWSTR) + 1) as isize);
    LoadStringW(hInstance, STRING_TEXT_FILES_TXT, p, (MAX_STRING_LEN as i32));
    p = p.offset((lstrlenW(p as LPCWSTR) + 1) as isize);
    lstrcpyW(p, &raw const files_txt as LPCWSTR);
    p = p.offset((lstrlenW(p as LPCWSTR) + 1) as isize);
    LoadStringW(hInstance, STRING_TEXT_FILES_UNICODE_TXT, p, (MAX_STRING_LEN as i32));
    p = p.offset((lstrlenW(p as LPCWSTR) + 1) as isize);
    lstrcpyW(p, &raw const files_txt as LPCWSTR);
    p = p.offset((lstrlenW(p as LPCWSTR) + 1) as isize);
    LoadStringW(hInstance, STRING_ALL_FILES, p, (MAX_STRING_LEN as i32));
    p = p.offset((lstrlenW(p as LPCWSTR) + 1) as isize);
    lstrcpyW(p, &raw const files_all as LPCWSTR);
    p = p.offset((lstrlenW(p as LPCWSTR) + 1) as isize);
    *p = '\0' as i32 as WCHAR;

    p = &raw mut wszDefaultFileName as *mut WCHAR as LPWSTR;
    LoadStringW(hInstance, STRING_DEFAULT_FILENAME, p, (MAX_STRING_LEN as i32));

    p = &raw mut wszSaveChanges as *mut WCHAR as LPWSTR;
    LoadStringW(hInstance, STRING_PROMPT_SAVE_CHANGES, p, (MAX_STRING_LEN as i32));

    LoadStringW(hInstance, STRING_UNITS_CM, &raw mut units_cmW as LPWSTR, (MAX_STRING_LEN as i32));
    LoadStringW(hInstance, STRING_UNITS_IN, &raw mut units_inW as LPWSTR, (MAX_STRING_LEN as i32));
    LoadStringW(hInstance, STRING_UNITS_INCH, &raw mut units_inchW as LPWSTR, (MAX_STRING_LEN as i32));
    LoadStringW(hInstance, STRING_UNITS_PT, &raw mut units_ptW as LPWSTR, (MAX_STRING_LEN as i32));
}

unsafe extern "C" fn MessageBoxWithResStringW(
    mut hWnd: HWND,
    mut lpText: LPCWSTR,
    mut lpCaption: LPCWSTR,
    mut uType: UINT,
) -> ::core::ffi::c_int {
    let mut params: MSGBOXPARAMSW = MSGBOXPARAMSW {
        cbSize: 0,
        hwndOwner: core::ptr::null_mut(),
        hInstance: core::ptr::null_mut(),
        lpszText: core::ptr::null(),
        lpszCaption: core::ptr::null(),
        dwStyle: 0,
        lpszIcon: core::ptr::null(),
        dwContextHelpId: 0,
        lpfnMsgBoxCallback: None,
        dwLanguageId: 0,
    };
    params.cbSize = core::mem::size_of::<MSGBOXPARAMSW>() as UINT;
    params.hwndOwner = hWnd;
    params.hInstance = GetModuleHandleW(core::ptr::null()) as HINSTANCE;
    params.lpszText = lpText;
    params.lpszCaption = lpCaption;
    params.dwStyle = uType as DWORD;
    params.lpszIcon = core::ptr::null();
    params.dwContextHelpId = 0;
    params.lpfnMsgBoxCallback = None;
    params.dwLanguageId = 0;
    return MessageBoxIndirectW(core::ptr::addr_of_mut!(params));
}

unsafe extern "C" fn AddButton(hwndToolBar: HWND, nImage: ::core::ffi::c_int, nCommand: ::core::ffi::c_int) {
    let mut button: TBBUTTON = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(button) as *mut ::core::ffi::c_void,
        core::mem::size_of::<TBBUTTON>() as SIZE_T,
    );
    button.iBitmap = nImage;
    button.idCommand = nCommand;
    button.fsState = TBSTATE_ENABLED as BYTE;
    button.fsStyle = BTNS_BUTTON as BYTE;
    button.dwData = 0 as DWORD_PTR;
    button.iString = -(1) as INT_PTR;
    SendMessageW(
        hwndToolBar,
        TB_ADDBUTTONSW as UINT,
        1 as WPARAM,
        core::ptr::addr_of_mut!(button) as LPARAM,
    );
}

unsafe extern "C" fn AddSeparator(hwndToolBar: HWND) {
    let mut button: TBBUTTON = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(button) as *mut core::ffi::c_void,
        core::mem::size_of::<TBBUTTON>(),
    );
    button.iBitmap = -1;
    button.idCommand = 0;
    button.fsState = 0 as BYTE;
    button.fsStyle = BTNS_SEP as BYTE;
    button.dwData = 0 as DWORD_PTR;
    button.iString = -1 as INT_PTR;
    SendMessageW(
        hwndToolBar,
        TB_ADDBUTTONSW as UINT,
        1 as WPARAM,
        core::ptr::addr_of_mut!(button) as LPARAM,
    );
}

unsafe extern "system" fn stream_in(cookie: DWORD_PTR, buffer: LPBYTE, cb: LONG, pcb: *mut LONG) -> DWORD {
    let hFile: HANDLE = cookie as HANDLE;
    let mut read: DWORD = 0;

    if ReadFile(hFile, buffer as LPVOID, cb as DWORD, core::ptr::addr_of_mut!(read), core::ptr::null_mut()) == 0 {
        return 1;
    }

    *pcb = read as LONG;

    return 0;
}

unsafe extern "system" fn stream_out(cookie: DWORD_PTR, buffer: LPBYTE, cb: LONG, pcb: *mut LONG) -> DWORD {
    let mut written: DWORD = 0;
    let mut ret: i32;
    let mut hFile: HANDLE = cookie as HANDLE;

    ret = WriteFile(hFile, buffer as LPCVOID, cb as DWORD, core::ptr::addr_of_mut!(written), core::ptr::null_mut());

    if ret == 0 || (cb as DWORD) != written {
        return 1 as DWORD;
    }

    *pcb = cb;

    return 0 as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn file_basename(path: LPWSTR) -> LPWSTR {
    let mut pos: LPWSTR = path.add(lstrlenW(path) as usize);

    while pos > path {
        if *pos == '\\' as u16 || *pos == '/' as u16 {
            pos = pos.add(1);
            break;
        }
        pos = pos.sub(1);
    }
    return pos;
}

unsafe extern "C" fn set_caption(mut wszNewFileName: LPCWSTR) {
    static wszSeparator: [WCHAR; 3] = [0x0020u16, 0x002D, 0x0020];
    let mut wszCaption: *mut WCHAR = core::ptr::null_mut();
    let mut length: SIZE_T = 0;

    if wszNewFileName.is_null() {
        wszNewFileName = core::ptr::addr_of_mut!(wszDefaultFileName) as *mut WCHAR as LPCWSTR;
    } else {
        wszNewFileName = file_basename(wszNewFileName as LPWSTR) as LPCWSTR;
    }

    wszCaption = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (lstrlenW(wszNewFileName) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)
            .wrapping_add(core::mem::size_of::<[WCHAR; 3]>() as SIZE_T)
            .wrapping_add(core::mem::size_of::<[WCHAR; 8]>() as SIZE_T),
    ) as *mut WCHAR;

    if wszCaption.is_null() {
        return;
    }

    memcpy(
        wszCaption as *mut core::ffi::c_void,
        wszNewFileName as *const core::ffi::c_void,
        (lstrlenW(wszNewFileName) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
    );
    length = length.wrapping_add(lstrlenW(wszNewFileName) as SIZE_T);
    memcpy(
        wszCaption.add(length as usize) as *mut core::ffi::c_void,
        core::ptr::addr_of!(wszSeparator) as *const WCHAR as *const core::ffi::c_void,
        core::mem::size_of::<[WCHAR; 3]>(),
    );
    length = length.wrapping_add(
        (core::mem::size_of::<[WCHAR; 3]>() / core::mem::size_of::<WCHAR>()) as SIZE_T,
    );
    memcpy(
        wszCaption.add(length as usize) as *mut core::ffi::c_void,
        core::ptr::addr_of!(wszAppTitle) as *const WCHAR as *const core::ffi::c_void,
        core::mem::size_of::<[WCHAR; 8]>(),
    );

    SetWindowTextW(hMainWnd, wszCaption);

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        wszCaption as PVOID,
    );
}

unsafe extern "C" fn validate_endptr(mut endptr: LPCWSTR, mut punit: *mut UNIT) -> BOOL {
    if !punit.is_null() {
        *punit = UNIT_CM;
    }
    if endptr.is_null() {
        return (FALSE) as BOOL;
    }
    if *endptr == 0 {
        return (TRUE) as BOOL;
    }
    while *endptr as ::core::ffi::c_int == ' ' as i32 {
        endptr = endptr.offset(1);
    }
    if punit.is_null() {
        return (*endptr as ::core::ffi::c_int == '\0' as i32) as ::core::ffi::c_int;
    }
    if lstrcmpW(endptr, &raw mut units_cmW as *mut WCHAR as LPCWSTR) == 0 {
        *punit = UNIT_CM;
        endptr = endptr.offset(lstrlenW(&raw mut units_cmW as *mut WCHAR as LPCWSTR) as isize);
    } else if lstrcmpW(endptr, &raw mut units_inW as *mut WCHAR as LPCWSTR) == 0 {
        *punit = UNIT_INCH;
        endptr = endptr.offset(lstrlenW(&raw mut units_inW as *mut WCHAR as LPCWSTR) as isize);
    } else if lstrcmpW(endptr, &raw mut units_inchW as *mut WCHAR as LPCWSTR) == 0 {
        *punit = UNIT_INCH;
        endptr = endptr.offset(lstrlenW(&raw mut units_inchW as *mut WCHAR as LPCWSTR) as isize);
    } else if lstrcmpW(endptr, &raw mut units_ptW as *mut WCHAR as LPCWSTR) == 0 {
        *punit = UNIT_PT;
        endptr = endptr.offset(lstrlenW(&raw mut units_ptW as *mut WCHAR as LPCWSTR) as isize);
    }
    return (*endptr as ::core::ffi::c_int == '\0' as i32) as ::core::ffi::c_int;
}

unsafe extern "C" fn number_from_string(string: LPCWSTR, num: *mut f32, punit: *mut UNIT) -> BOOL {
    let mut ret: f64 = 0.0;
    let mut endptr: *mut WCHAR = core::ptr::null_mut();

    *num = 0 as f32;
    *_errno() = 0;
    ret = wcstod(string, core::ptr::addr_of_mut!(endptr));

    if !punit.is_null() {
        *punit = UNIT_CM;
    }
    if (ret == 0 as f64 && *_errno() != 0)
        || endptr == string as *mut WCHAR
        || validate_endptr(endptr as LPCWSTR, punit) == 0
    {
        return (FALSE) as BOOL;
    } else {
        *num = ret as f32;
        return (TRUE) as BOOL;
    }
}

unsafe extern "C" fn set_size(size: f32) {
    let mut fmt: CHARFORMAT2W = core::mem::zeroed();

    ZeroMemory!(core::ptr::addr_of_mut!(fmt) as *mut core::ffi::c_void, core::mem::size_of::<CHARFORMAT2W>());
    fmt.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;
    fmt.dwMask = CFM_SIZE;
    fmt.yHeight = (size as f64 * 20.0) as i32; // C: `size * 20.0`, a double multiply
    SendMessageW(hEditorWnd, EM_SETCHARFORMAT, (SCF_SELECTION) as WPARAM, core::ptr::addr_of_mut!(fmt) as LPARAM);
}

unsafe extern "C" fn on_sizelist_modified(hwndSizeList: HWND, wszNewFontSize: LPWSTR) {
    let mut sizeBuffer: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
    let mut format: CHARFORMAT2W = core::mem::zeroed();

    core::ptr::write_bytes(&raw mut format as *mut u8, 0, core::mem::size_of::<CHARFORMAT2W>());
    format.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;
    SendMessageW(
        hEditorWnd,
        EM_GETCHARFORMAT as UINT,
        SCF_SELECTION as WPARAM,
        &raw mut format as LPARAM,
    );

    wsprintfW(
        &raw mut sizeBuffer as LPWSTR,
        &raw const stringFormat as LPCWSTR,
        format.yHeight / 20,
    );
    if lstrcmpW(
        &raw mut sizeBuffer as *mut WCHAR as LPCWSTR,
        wszNewFontSize as LPCWSTR,
    ) != 0
    {
        let mut size: f32 = 0 as f32;
        if number_from_string(
            wszNewFontSize as LPCWSTR,
            &raw mut size,
            core::ptr::null_mut(),
        ) != 0
            && size > 0 as f32
        {
            set_size(size);
        } else {
            SetWindowTextW(hwndSizeList, &raw mut sizeBuffer as *mut WCHAR as LPCWSTR);
            MessageBoxWithResStringW(
                hMainWnd,
                MAKEINTRESOURCEW!(STRING_INVALID_NUMBER),
                &raw const wszAppTitle as LPCWSTR,
                (MB_OK | MB_ICONINFORMATION) as UINT,
            );
        }
    }
}

unsafe extern "C" fn add_size(mut hSizeListWnd: HWND, mut size: ::core::ffi::c_uint) {
    let mut buffer: [WCHAR; 3] = [0; 3];
    let mut cbItem: COMBOBOXEXITEMW = COMBOBOXEXITEMW {
        mask: 0,
        iItem: 0,
        pszText: ::core::ptr::null_mut::<WCHAR>(),
        cchTextMax: 0,
        iImage: 0,
        iSelectedImage: 0,
        iOverlay: 0,
        iIndent: 0,
        lParam: 0,
    };
    cbItem.mask = CBEIF_TEXT as UINT;
    cbItem.iItem = -(1 as INT_PTR);

    wsprintfW(
        &raw mut buffer as LPWSTR,
        &raw const stringFormat as LPCWSTR,
        size,
    );
    cbItem.pszText = &raw mut buffer as *mut WCHAR as LPWSTR;
    SendMessageW(
        hSizeListWnd,
        CBEM_INSERTITEMW as UINT,
        0 as WPARAM,
        &raw mut cbItem as LPARAM,
    );
}

unsafe extern "C" fn populate_size_list(hSizeListWnd: HWND) {
    let hReBarWnd: HWND = GetDlgItem(hMainWnd, (IDC_REBAR as i32));
    let hFontListWnd: HWND = GetDlgItem(hReBarWnd, (IDC_FONTLIST as i32));
    let mut cbFontItem: COMBOBOXEXITEMW = core::mem::zeroed();
    let mut fmt: CHARFORMAT2W = core::mem::zeroed();
    let hListEditWnd: HWND = SendMessageW(
        hSizeListWnd,
        CBEM_GETEDITCONTROL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as HWND;
    let hdc: HDC = GetDC(hMainWnd);
    static choices: [::core::ffi::c_uint; 16] = [
        8 as ::core::ffi::c_uint,
        9 as ::core::ffi::c_uint,
        10 as ::core::ffi::c_uint,
        11 as ::core::ffi::c_uint,
        12 as ::core::ffi::c_uint,
        14 as ::core::ffi::c_uint,
        16 as ::core::ffi::c_uint,
        18 as ::core::ffi::c_uint,
        20 as ::core::ffi::c_uint,
        22 as ::core::ffi::c_uint,
        24 as ::core::ffi::c_uint,
        26 as ::core::ffi::c_uint,
        28 as ::core::ffi::c_uint,
        36 as ::core::ffi::c_uint,
        48 as ::core::ffi::c_uint,
        72 as ::core::ffi::c_uint,
    ];
    let mut buffer: [WCHAR; 3 as usize] = [0 as WCHAR; 3 as usize];
    let mut i: usize;
    let mut fontStyle: DWORD;

    ZeroMemory!(
        core::ptr::addr_of_mut!(fmt) as *mut ::core::ffi::c_void,
        core::mem::size_of::<CHARFORMAT2W>(),
    );
    fmt.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;
    SendMessageW(
        hEditorWnd,
        EM_GETCHARFORMAT as UINT,
        SCF_SELECTION as WPARAM,
        core::ptr::addr_of_mut!(fmt) as LPARAM,
    );

    cbFontItem.mask = CBEIF_LPARAM as UINT;
    cbFontItem.iItem = SendMessageW(
        hFontListWnd,
        CB_FINDSTRINGEXACT as UINT,
        -(1 as ::core::ffi::c_int) as WPARAM,
        core::ptr::addr_of_mut!(fmt.szFaceName) as *mut WCHAR as LPARAM,
    ) as INT_PTR;
    SendMessageW(
        hFontListWnd,
        CBEM_GETITEMW as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(cbFontItem) as LPARAM,
    );

    fontStyle = LOWORD!(cbFontItem.lParam as DWORD) as DWORD;

    SendMessageW(
        hSizeListWnd,
        CB_RESETCONTENT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    if (fontStyle & (RASTER_FONTTYPE as DWORD)) != 0 && cbFontItem.iItem != 0 {
        add_size(
            hSizeListWnd,
            MulDiv(
                HIWORD!(cbFontItem.lParam as DWORD) as ::core::ffi::c_int,
                72 as ::core::ffi::c_int,
                GetDeviceCaps(hdc, LOGPIXELSY as ::core::ffi::c_int),
            ) as BYTE as ::core::ffi::c_uint,
        );
    } else {
        i = 0 as usize;
        while i < core::mem::size_of::<[::core::ffi::c_uint; 16]>()
            / core::mem::size_of_val(&choices[0 as usize])
        {
            add_size(hSizeListWnd, choices[i]);
            i += 1;
        }
    }

    wsprintfW(
        core::ptr::addr_of_mut!(buffer) as LPWSTR,
        core::ptr::addr_of!(stringFormat) as LPCWSTR,
        fmt.yHeight / (20 as LONG),
    );
    SendMessageW(
        hListEditWnd,
        WM_SETTEXT as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPARAM,
    );
}

unsafe extern "C" fn update_size_list() {
    let mut hReBar: HWND = GetDlgItem(hMainWnd, (IDC_REBAR as i32));
    let mut hwndSizeList: HWND = GetDlgItem(hReBar, (IDC_SIZELIST as i32));
    let mut hwndSizeListEdit: HWND = SendMessageW(hwndSizeList, CBEM_GETEDITCONTROL, 0, 0) as HWND;
    let mut fontSize: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
    let mut sizeBuffer: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
    let mut fmt: CHARFORMAT2W = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(fmt) as *mut ::core::ffi::c_void,
        core::mem::size_of::<CHARFORMAT2W>(),
    );
    fmt.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;

    SendMessageW(hEditorWnd, EM_GETCHARFORMAT, (SCF_SELECTION) as WPARAM, core::ptr::addr_of_mut!(fmt) as LPARAM);

    // The C passes MAX_PATH (260) for this MAX_STRING_LEN (255) buffer; pass its real length.
    SendMessageW(hwndSizeListEdit, WM_GETTEXT, (MAX_STRING_LEN) as WPARAM, core::ptr::addr_of_mut!(fontSize) as *mut WCHAR as LPARAM);
    wsprintfW(core::ptr::addr_of_mut!(sizeBuffer) as *mut WCHAR, core::ptr::addr_of!(stringFormat) as *const WCHAR, fmt.yHeight / 20);

    if lstrcmpW(core::ptr::addr_of_mut!(fontSize) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(sizeBuffer) as *mut WCHAR as LPCWSTR) != 0 {
        SendMessageW(hwndSizeListEdit, WM_SETTEXT, 0, core::ptr::addr_of_mut!(sizeBuffer) as *mut WCHAR as LPARAM);
    }
}

unsafe extern "C" fn update_font_list() {
    let hReBar: HWND = GetDlgItem(hMainWnd, (IDC_REBAR as i32)) as HWND;
    let hFontList: HWND = GetDlgItem(hReBar, (IDC_FONTLIST as i32)) as HWND;
    let hFontListEdit: HWND = SendMessageW(hFontList, CBEM_GETEDITCONTROL as UINT, 0 as WPARAM, 0 as LPARAM) as HWND;
    let mut fontName: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
    let mut fmt: CHARFORMAT2W = core::mem::zeroed();

    ZeroMemory!(core::ptr::addr_of_mut!(fmt) as *mut ::core::ffi::c_void, core::mem::size_of::<CHARFORMAT2W>() as usize);
    fmt.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;

    SendMessageW(hEditorWnd, EM_GETCHARFORMAT as UINT, SCF_SELECTION as WPARAM, core::ptr::addr_of_mut!(fmt) as LPARAM);
    // The C passes MAX_PATH (260) for this MAX_STRING_LEN (255) buffer; pass its real length.
    if SendMessageW(hFontListEdit, WM_GETTEXT as UINT, MAX_STRING_LEN as WPARAM, core::ptr::addr_of_mut!(fontName) as *mut WCHAR as LPARAM) == 0 {
        return;
    }

    if lstrcmpW(core::ptr::addr_of_mut!(fontName) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(fmt.szFaceName) as *mut WCHAR as LPCWSTR) != 0 {
        SendMessageW(hFontListEdit, WM_SETTEXT as UINT, 0 as WPARAM, core::ptr::addr_of_mut!(fmt.szFaceName) as *mut WCHAR as LPARAM);
        populate_size_list(GetDlgItem(hReBar, (IDC_SIZELIST as i32)) as HWND);
    } else {
        update_size_list();
    }
}

unsafe extern "C" fn clear_formatting() {
    let mut pf: PARAFORMAT2 = core::mem::zeroed();
    pf.cbSize = ::core::mem::size_of::<PARAFORMAT2>() as UINT;
    pf.dwMask = PFM_ALIGNMENT as DWORD;
    pf.wAlignment = PFA_LEFT as WORD;
    SendMessageW(
        hEditorWnd,
        EM_SETPARAFORMAT as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(pf) as LPARAM,
    );
}

unsafe extern "C" fn fileformat_number(format: WPARAM) -> ::core::ffi::c_int {
    let mut number: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if format == SF_TEXT as WPARAM {
        number = 1 as ::core::ffi::c_int;
    } else if format == (SF_TEXT | SF_UNICODE) as WPARAM {
        number = 2 as ::core::ffi::c_int;
    }
    return number;
}

unsafe extern "C" fn fileformat_flags(format: ::core::ffi::c_int) -> WPARAM {
    let flags: [WPARAM; 3] = [
        SF_RTF as WPARAM,
        SF_TEXT as WPARAM,
        (SF_TEXT | SF_UNICODE) as WPARAM,
    ];
    return flags[format as usize];
}

unsafe extern "C" fn set_font(wszFaceName: LPCWSTR) {
    let hReBarWnd: HWND = GetDlgItem(hMainWnd, (IDC_REBAR as i32));
    let hSizeListWnd: HWND = GetDlgItem(hReBarWnd, (IDC_SIZELIST as i32));
    let hFontListWnd: HWND = GetDlgItem(hReBarWnd, (IDC_FONTLIST as i32));
    let hFontListEditWnd: HWND = SendMessageW(
        hFontListWnd,
        CBEM_GETEDITCONTROL,
        0,
        0,
    ) as HWND;
    let mut fmt: CHARFORMAT2W = core::mem::zeroed();

    ZeroMemory!(
        &raw mut fmt as *mut core::ffi::c_void,
        core::mem::size_of::<CHARFORMAT2W>(),
    );

    fmt.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;
    fmt.dwMask = CFM_FACE;

    lstrcpyW(&raw mut fmt.szFaceName as LPWSTR, wszFaceName);

    SendMessageW(
        hEditorWnd,
        EM_SETCHARFORMAT,
        (SCF_SELECTION) as WPARAM,
        &raw mut fmt as LPARAM,
    );

    populate_size_list(hSizeListWnd);

    SendMessageW(
        hFontListEditWnd,
        WM_SETTEXT,
        0,
        wszFaceName as LPARAM,
    );
}

unsafe extern "C" fn set_default_font() {
    static richTextFont: [WCHAR; 16] = [
    0x0054u16, 0x0069, 0x006D, 0x0065, 0x0073, 0x0020, 0x004E, 0x0065, 0x0077, 0x0020,
    0x0052, 0x006F, 0x006D, 0x0061, 0x006E, 0x0000,
];
    static plainTextFont: [WCHAR; 12] = [
    0x0043u16, 0x006F, 0x0075, 0x0072, 0x0069, 0x0065, 0x0072, 0x0020, 0x004E, 0x0065,
    0x0077, 0x0000,
];
    let mut fmt: CHARFORMAT2W = core::mem::zeroed();
    let mut font: LPCWSTR;

    ZeroMemory!(
        core::ptr::addr_of_mut!(fmt) as *mut core::ffi::c_void,
        core::mem::size_of::<CHARFORMAT2W>(),
    );

    fmt.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;
    fmt.dwMask = CFM_FACE | CFM_BOLD | CFM_ITALIC | CFM_UNDERLINE;
    fmt.dwEffects = 0;

    if (fileFormat & SF_RTF as WPARAM) != 0 {
        font = core::ptr::addr_of!(richTextFont) as *const WCHAR as LPCWSTR;
    } else {
        font = core::ptr::addr_of!(plainTextFont) as *const WCHAR as LPCWSTR;
    }

    lstrcpyW(
        core::ptr::addr_of_mut!(fmt.szFaceName) as *mut WCHAR as LPWSTR,
        font,
    );

    SendMessageW(
        hEditorWnd,
        EM_SETCHARFORMAT as UINT,
        SCF_DEFAULT as WPARAM,
        core::ptr::addr_of_mut!(fmt) as LPARAM,
    );
}

unsafe extern "C" fn on_fontlist_modified(wszNewFaceName: LPWSTR) {
    let mut format: CHARFORMAT2W = core::mem::zeroed();
    ZeroMemory!(
        core::ptr::addr_of_mut!(format) as *mut core::ffi::c_void,
        core::mem::size_of::<CHARFORMAT2W>(),
    );
    format.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;
    SendMessageW(
        hEditorWnd,
        EM_GETCHARFORMAT as UINT,
        SCF_SELECTION as WPARAM,
        core::ptr::addr_of_mut!(format) as LPARAM,
    );

    if lstrcmpW(
        core::ptr::addr_of_mut!(format.szFaceName) as *mut WCHAR as LPCWSTR,
        wszNewFaceName as LPCWSTR,
    ) != 0
    {
        set_font(wszNewFaceName as LPCWSTR);
    }
}

unsafe extern "C" fn add_font(
    fontName: LPCWSTR,
    fontType: DWORD,
    hListWnd: HWND,
    ntmc: *const NEWTEXTMETRICEXW,
) {
    let mut cbItem: COMBOBOXEXITEMW = core::mem::zeroed();
    let mut buffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut fontHeight: i32 = 0;

    cbItem.mask = CBEIF_TEXT as UINT;
    cbItem.pszText = core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPWSTR;
    cbItem.cchTextMax = (MAX_STRING_LEN as i32);
    cbItem.iItem = 0 as INT_PTR;

    while SendMessageW(
        hListWnd,
        CBEM_GETITEMW as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(cbItem) as LPARAM,
    ) != 0
    {
        if lstrcmpiW(cbItem.pszText as LPCWSTR, fontName) <= 0 {
            cbItem.iItem += 1;
        } else {
            break;
        }
    }
    cbItem.pszText = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ((lstrlenW(fontName) + 1) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    lstrcpyW(cbItem.pszText, fontName);

    cbItem.mask |= CBEIF_LPARAM as UINT;
    if (fontType & RASTER_FONTTYPE) != 0 {
        fontHeight = (*ntmc).ntmTm.tmHeight - (*ntmc).ntmTm.tmInternalLeading;
    }

    cbItem.lParam = MAKELONG!(fontType as WORD, fontHeight as WORD) as LPARAM;
    SendMessageW(
        hListWnd,
        CBEM_INSERTITEMW as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(cbItem) as LPARAM,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        cbItem.pszText as PVOID,
    );
}

unsafe extern "C" fn dialog_choose_font() {
    let mut cf: CHOOSEFONTW = core::mem::zeroed();
    let mut lf: LOGFONTW = core::mem::zeroed();
    let mut fmt: CHARFORMAT2W = core::mem::zeroed();
    let mut hDC: HDC = GetDC(hMainWnd);

    ZeroMemory!(
        core::ptr::addr_of_mut!(cf) as *mut core::ffi::c_void,
        core::mem::size_of::<CHOOSEFONTW>(),
    );
    cf.lStructSize = core::mem::size_of::<CHOOSEFONTW>() as DWORD;
    cf.hwndOwner = hMainWnd;
    cf.lpLogFont = core::ptr::addr_of_mut!(lf);
    cf.Flags = CF_SCREENFONTS | CF_NOSCRIPTSEL | CF_INITTOLOGFONTSTRUCT | CF_EFFECTS | CF_NOVERTFONTS;

    ZeroMemory!(
        core::ptr::addr_of_mut!(fmt) as *mut core::ffi::c_void,
        core::mem::size_of::<CHARFORMAT2W>(),
    );
    fmt.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;

    SendMessageW(
        hEditorWnd,
        EM_GETCHARFORMAT,
        (SCF_SELECTION) as WPARAM,
        core::ptr::addr_of_mut!(fmt) as LPARAM,
    );
    lstrcpyW(
        core::ptr::addr_of_mut!((*cf.lpLogFont).lfFaceName) as LPWSTR,
        core::ptr::addr_of_mut!(fmt.szFaceName) as *mut WCHAR as LPCWSTR,
    );
    (*cf.lpLogFont).lfItalic = ((fmt.dwEffects & CFE_ITALIC) != 0) as BYTE;
    (*cf.lpLogFont).lfWeight = (if (fmt.dwEffects & CFE_BOLD) != 0 {
        FW_BOLD
    } else {
        FW_NORMAL
    }) as LONG;
    (*cf.lpLogFont).lfUnderline = ((fmt.dwEffects & CFE_UNDERLINE) != 0) as BYTE;
    (*cf.lpLogFont).lfStrikeOut = ((fmt.dwEffects & CFE_STRIKEOUT) != 0) as BYTE;
    (*cf.lpLogFont).lfHeight = -(MulDiv(
        fmt.yHeight / 20,
        GetDeviceCaps(hDC, (LOGPIXELSY as i32)),
        72,
    )) as LONG;
    cf.rgbColors = fmt.crTextColor;

    if ChooseFontW(core::ptr::addr_of_mut!(cf)) != 0 {
        ZeroMemory!(
            core::ptr::addr_of_mut!(fmt) as *mut core::ffi::c_void,
            core::mem::size_of::<CHARFORMAT2W>(),
        );
        fmt.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;
        fmt.dwMask = CFM_BOLD | CFM_ITALIC | CFM_SIZE | CFM_UNDERLINE | CFM_STRIKEOUT | CFM_COLOR;
        fmt.yHeight = cf.iPointSize * 2;

        if (cf.nFontType & (BOLD_FONTTYPE as u16)) != 0 {
            fmt.dwEffects |= CFE_BOLD;
        }
        if (cf.nFontType & (ITALIC_FONTTYPE as u16)) != 0 {
            fmt.dwEffects |= CFE_ITALIC;
        }
        if (*cf.lpLogFont).lfUnderline != 0 {
            fmt.dwEffects |= CFE_UNDERLINE;
        }
        if (*cf.lpLogFont).lfStrikeOut != 0 {
            fmt.dwEffects |= CFE_STRIKEOUT;
        }

        fmt.crTextColor = cf.rgbColors;

        SendMessageW(
            hEditorWnd,
            EM_SETCHARFORMAT,
            (SCF_SELECTION) as WPARAM,
            core::ptr::addr_of_mut!(fmt) as LPARAM,
        );
        set_font(
            core::ptr::addr_of_mut!((*cf.lpLogFont).lfFaceName) as *mut WCHAR as LPCWSTR,
        );
    }
}

unsafe extern "system" fn enum_font_proc(lpelfe: *const LOGFONTW, lpntme: *const TEXTMETRICW, FontType: DWORD, lParam: LPARAM) -> i32 {
    let mut hListWnd: HWND = lParam as HWND;
    if (*lpelfe).lfFaceName[0 as usize] == '@' as WCHAR {
        return 1 as i32;
    }
    if SendMessageW(
        hListWnd,
        CB_FINDSTRINGEXACT as UINT,
        -(1 as isize) as WPARAM,
        core::ptr::addr_of!((*lpelfe).lfFaceName) as *const WCHAR as LPARAM,
    ) == CB_ERR as LRESULT
    {
        add_font(
            core::ptr::addr_of!((*lpelfe).lfFaceName) as LPCWSTR,
            FontType,
            hListWnd,
            lpntme as *const NEWTEXTMETRICEXW,
        );
    }
    return 1 as i32;
}

unsafe extern "C" fn populate_font_list(mut hListWnd: HWND) {
    let mut hdc: HDC = GetDC(hMainWnd);
    let mut fontinfo: LOGFONTW = core::mem::zeroed();
    let mut hListEditWnd: HWND = SendMessageW(
        hListWnd,
        CBEM_GETEDITCONTROL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as HWND;
    let mut fmt: CHARFORMAT2W = core::mem::zeroed();

    fontinfo.lfCharSet = DEFAULT_CHARSET as BYTE;
    *(core::ptr::addr_of_mut!(fontinfo.lfFaceName) as *mut WCHAR) = '\0' as WCHAR;
    fontinfo.lfPitchAndFamily = 0 as BYTE;

    EnumFontFamiliesExW(
        hdc,
        core::ptr::addr_of_mut!(fontinfo),
        Some(
            enum_font_proc,
        ),
        hListWnd as LPARAM,
        0 as DWORD,
    );

    ZeroMemory!(
        core::ptr::addr_of_mut!(fmt) as *mut ::core::ffi::c_void,
        core::mem::size_of::<CHARFORMAT2W>() as size_t,
    );
    fmt.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;
    SendMessageW(
        hEditorWnd,
        EM_GETCHARFORMAT as UINT,
        SCF_DEFAULT as WPARAM,
        core::ptr::addr_of_mut!(fmt) as LPARAM,
    );
    SendMessageW(
        hListEditWnd,
        WM_SETTEXT as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(fmt.szFaceName) as *mut WCHAR as LPARAM,
    );
}

unsafe extern "C" fn update_window() {
    let mut rect: RECT = core::mem::zeroed();

    GetClientRect(hMainWnd, core::ptr::addr_of_mut!(rect));

    OnSize(hMainWnd, SIZE_RESTORED as WPARAM, MAKELPARAM!(rect.right as WORD, rect.bottom as WORD));
}

unsafe extern "C" fn is_bar_visible(bandId: ::core::ffi::c_int) -> BOOL {
    return (barState[reg_formatindex(fileFormat) as usize]
        & (((1 as ::core::ffi::c_int) << bandId) as DWORD)) as BOOL;
}

unsafe extern "C" fn store_bar_state(bandId: i32, show: BOOL) {
    let mut formatIndex: i32 = reg_formatindex(fileFormat);

    if show != 0 {
        barState[formatIndex as usize] |= (1 << bandId) as DWORD;
    } else {
        barState[formatIndex as usize] &= !(1 << bandId) as DWORD;
    }
}

unsafe extern "C" fn set_toolbar_state(mut bandId: ::core::ffi::c_int, mut show: BOOL) {
    let mut hwndReBar: HWND = GetDlgItem(hMainWnd, IDC_REBAR as _);
    SendMessageW(
        hwndReBar,
        RB_SHOWBAND as UINT,
        SendMessageW(
            hwndReBar,
            RB_IDTOINDEX as UINT,
            bandId as WPARAM,
            0 as LPARAM,
        ) as WPARAM,
        show as LPARAM,
    );
    if bandId == (BANDID_TOOLBAR as i32) {
        let mut rbbinfo: REBARBANDINFOW = core::mem::zeroed();
        let mut index: ::core::ffi::c_int = SendMessageW(
            hwndReBar,
            RB_IDTOINDEX as UINT,
            BANDID_FONTLIST as WPARAM,
            0 as LPARAM,
        ) as ::core::ffi::c_int;
        rbbinfo.cbSize = REBARBANDINFOW_V6_SIZE as UINT;
        rbbinfo.fMask = RBBIM_STYLE as UINT;
        SendMessageW(
            hwndReBar,
            RB_GETBANDINFOW as UINT,
            index as WPARAM,
            core::ptr::addr_of_mut!(rbbinfo) as LPARAM,
        );
        if show == 0 {
            rbbinfo.fStyle &= !(RBBS_BREAK as UINT);
        } else {
            rbbinfo.fStyle |= RBBS_BREAK as UINT;
        }
        SendMessageW(
            hwndReBar,
            RB_SETBANDINFOW as UINT,
            index as WPARAM,
            core::ptr::addr_of_mut!(rbbinfo) as LPARAM,
        );
    }
    if bandId == (BANDID_TOOLBAR as i32)
        || bandId == (BANDID_FORMATBAR as i32)
        || bandId == (BANDID_RULER as i32)
    {
        store_bar_state(bandId, show);
    }
}

unsafe extern "C" fn set_statusbar_state(show: BOOL) {
    let mut hStatusWnd: HWND = GetDlgItem(hMainWnd, (IDC_STATUSBAR as i32));

    ShowWindow(hStatusWnd, if show != 0 { (SW_SHOW as i32) } else { (SW_HIDE as i32) });
    store_bar_state((BANDID_STATUSBAR as i32), show);
}

unsafe extern "C" fn set_bar_states() {
    set_toolbar_state((BANDID_TOOLBAR as i32), is_bar_visible((BANDID_TOOLBAR as i32)));
    set_toolbar_state((BANDID_FONTLIST as i32), is_bar_visible((BANDID_FORMATBAR as i32)));
    set_toolbar_state((BANDID_SIZELIST as i32), is_bar_visible((BANDID_FORMATBAR as i32)));
    set_toolbar_state((BANDID_FORMATBAR as i32), is_bar_visible((BANDID_FORMATBAR as i32)));
    set_toolbar_state((BANDID_RULER as i32), is_bar_visible((BANDID_RULER as i32)));
    set_statusbar_state(is_bar_visible((BANDID_STATUSBAR as i32)));

    update_window();
}

unsafe extern "C" fn preview_exit(hMainWnd_: HWND) {
    // The C parameter shadows the global hMainWnd; Rust forbids a parameter shadowing a static.
    let hMenu: HMENU = LoadMenuW(GetModuleHandleW(core::ptr::null()), MAKEINTRESOURCEW!(IDM_MAINMENU));
    let hEditorWnd_local: HWND = GetDlgItem(hMainWnd_, (IDC_EDITOR as i32));

    set_bar_states();
    ShowWindow(hEditorWnd_local, (TRUE as i32));

    close_preview(hMainWnd_);

    SetMenu(hMainWnd_, hMenu);
    registry_read_filelist(hMainWnd_);

    update_window();
}

unsafe extern "C" fn set_fileformat(format: WPARAM) {
    fileFormat = format;

    set_bar_states();
    set_default_font();
    target_device(hMainWnd, wordWrap[reg_formatindex(fileFormat) as usize]);
}

unsafe extern "C" fn ShowOpenError(Code: DWORD) {
    let mut Message: LPWSTR;
    match Code {
        ERROR_ACCESS_DENIED => {
            Message = MAKEINTRESOURCEW!(STRING_OPEN_ACCESS_DENIED);
        }
        _ => {
            Message = MAKEINTRESOURCEW!(STRING_OPEN_FAILED);
        }
    }
    MessageBoxW(
        hMainWnd,
        Message,
        core::ptr::addr_of!(wszAppTitle) as LPCWSTR,
        MB_ICONEXCLAMATION | MB_OK,
    );
}

unsafe extern "C" fn DoOpenFile(szOpenFileName: LPCWSTR) {
    let mut hFile: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut es: EDITSTREAM = _editstream {
        dwCookie: 0,
        dwError: 0,
        pfnCallback: None,
    };
    let mut fileStart: [core::ffi::c_char; 5] = [0; 5];
    let mut readOut: DWORD = 0;
    let mut format: WPARAM = SF_TEXT as WPARAM;

    hFile = CreateFileW(
        szOpenFileName,
        GENERIC_READ,
        FILE_SHARE_READ as DWORD,
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        OPEN_EXISTING as DWORD,
        FILE_ATTRIBUTE_NORMAL as DWORD,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        ShowOpenError(GetLastError());
        return;
    }

    ReadFile(
        hFile,
        &raw mut fileStart as *mut core::ffi::c_char as PVOID,
        5 as DWORD,
        &raw mut readOut,
        core::ptr::null_mut::<_OVERLAPPED>(),
    );
    SetFilePointer(
        hFile,
        0 as LONG,
        core::ptr::null_mut::<LONG>(),
        FILE_BEGIN as DWORD,
    );

    if readOut >= 2 as DWORD
        && (fileStart[0 as usize] as BYTE) == 0xff as BYTE
        && (fileStart[1 as usize] as BYTE) == 0xfe as BYTE
    {
        format = (SF_TEXT | SF_UNICODE) as WPARAM;
        SetFilePointer(
            hFile,
            2 as LONG,
            core::ptr::null_mut::<LONG>(),
            FILE_BEGIN as DWORD,
        );
    } else if readOut >= 5 as DWORD {
        static header: [u8; 6] = [b'{', b'\\', b'r', b't', b'f', 0];
        static STG_magic: [BYTE; 4] = [
            0xd0 as BYTE,
            0xcf as BYTE,
            0x11 as BYTE,
            0xe0 as BYTE,
        ];

        if memcmp(
            &raw const header as *const core::ffi::c_void,
            &raw mut fileStart as *mut core::ffi::c_char as *const core::ffi::c_void,
            5 as size_t,
        ) == 0
        {
            format = SF_RTF as WPARAM;
        } else if memcmp(
            &raw const STG_magic as *const BYTE as *const core::ffi::c_void,
            &raw mut fileStart as *mut core::ffi::c_char as *const core::ffi::c_void,
            core::mem::size_of::<[BYTE; 4]>() as size_t,
        ) == 0
        {
            CloseHandle(hFile);
            MessageBoxWithResStringW(
                hMainWnd,
                MAKEINTRESOURCEW!(STRING_OLE_STORAGE_NOT_SUPPORTED),
                &raw const wszAppTitle as LPCWSTR,
                (MB_OK | MB_ICONEXCLAMATION) as UINT,
            );
            return;
        }
    }

    es.dwCookie = hFile as DWORD_PTR;
    es.pfnCallback =
        Some(stream_in);

    clear_formatting();
    set_fileformat(format);
    SendMessageW(
        hEditorWnd,
        EM_STREAMIN as UINT,
        format,
        &raw mut es as LPARAM,
    );

    CloseHandle(hFile);

    SetFocus(hEditorWnd);

    set_caption(szOpenFileName);
    if *szOpenFileName.offset(0 as isize) != 0 {
        SHAddToRecentDocs(SHARD_PATHW as UINT, szOpenFileName as LPCVOID);
    }

    lstrcpyW(&raw mut wszFileName as LPWSTR, szOpenFileName);
    SendMessageW(
        hEditorWnd,
        EM_SETMODIFY as UINT,
        FALSE as WPARAM,
        0 as LPARAM,
    );
    registry_set_filelist(szOpenFileName, hMainWnd);
    update_font_list();
}

unsafe extern "C" fn ShowWriteError(Code: DWORD) {
    let mut Message: LPWSTR = core::ptr::null_mut::<WCHAR>();
    match Code {
        ERROR_ACCESS_DENIED => {
            Message = MAKEINTRESOURCEW!(STRING_WRITE_ACCESS_DENIED);
        }
        _ => {
            Message = MAKEINTRESOURCEW!(STRING_WRITE_FAILED);
        }
    }
    MessageBoxW(
        hMainWnd,
        Message as LPCWSTR,
        core::ptr::addr_of!(wszAppTitle) as LPCWSTR,
        MB_ICONEXCLAMATION | MB_OK,
    );
}

unsafe extern "C" fn DoSaveFile(wszSaveFileName: LPCWSTR, format: WPARAM) -> BOOL {
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut stream: EDITSTREAM = core::mem::zeroed();
    let mut ret: LRESULT = 0;

    hFile = if true {
        /* Use OPEN_ALWAYS instead of CREATE_ALWAYS in order to succeed
         * even if the file has HIDDEN or SYSTEM attributes */
        CreateFileW(wszSaveFileName, GENERIC_WRITE as DWORD, FILE_SHARE_READ as DWORD, core::ptr::null_mut(), OPEN_ALWAYS as DWORD,
            FILE_ATTRIBUTE_NORMAL as DWORD, core::ptr::null_mut())
    } else {
        CreateFileW(wszSaveFileName, GENERIC_WRITE as DWORD, 0 as DWORD, core::ptr::null_mut(), CREATE_ALWAYS as DWORD,
            FILE_ATTRIBUTE_NORMAL as DWORD, core::ptr::null_mut())
    };

    if hFile == INVALID_HANDLE_VALUE as HANDLE {
        ShowWriteError(GetLastError());
        return FALSE as BOOL;
    }

    if format == (SF_TEXT | SF_UNICODE) as WPARAM {
        static mut unicode: [BYTE; 2] = [
            0xff as BYTE,
            0xfe as BYTE,
        ];
        let mut writeOut: DWORD = 0;
        WriteFile(
            hFile,
            core::ptr::addr_of!(unicode) as LPCVOID,
            core::mem::size_of::<[BYTE; 2]>() as DWORD,
            core::ptr::addr_of_mut!(writeOut),
            0 as *mut _OVERLAPPED,
        );

        if writeOut as usize != core::mem::size_of::<[BYTE; 2]>() {
            CloseHandle(hFile);
            return FALSE as BOOL;
        }
    }

    stream.dwCookie = hFile as DWORD_PTR;
    stream.pfnCallback =
        Some(stream_out);

    ret = SendMessageW(
        hEditorWnd,
        EM_STREAMOUT as UINT,
        format,
        core::ptr::addr_of_mut!(stream) as LPARAM,
    );

    /* Truncate the file and close it */
    SetEndOfFile(hFile);
    CloseHandle(hFile);

    SetFocus(hEditorWnd);

    if ret == 0 {
        let mut gt: GETTEXTLENGTHEX = core::mem::zeroed();
        gt.flags = GTL_DEFAULT as DWORD;
        gt.codepage = 1200 as UINT;

        if SendMessageW(
            hEditorWnd,
            EM_GETTEXTLENGTHEX as UINT,
            core::ptr::addr_of_mut!(gt) as WPARAM,
            0 as LPARAM,
        ) != 0
        {
            return FALSE as BOOL;
        }
    }

    lstrcpyW(core::ptr::addr_of_mut!(wszFileName) as LPWSTR, wszSaveFileName);
    set_caption(core::ptr::addr_of_mut!(wszFileName) as *mut WCHAR as LPCWSTR);
    if wszFileName[0 as usize] != 0 {
        SHAddToRecentDocs(
            SHARD_PATHW as UINT,
            core::ptr::addr_of_mut!(wszFileName) as *mut WCHAR as LPCVOID,
        );
    }

    SendMessageW(
        hEditorWnd,
        EM_SETMODIFY as UINT,
        FALSE as WPARAM,
        0 as LPARAM,
    );
    set_fileformat(format);

    return TRUE as BOOL;
}

unsafe extern "C" fn DialogSaveFile() -> BOOL {
    let mut sfn: OPENFILENAMEW = core::mem::zeroed();

    let mut wszFile: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
    static wszDefExt: [WCHAR; 4] = [0x0072u16, 0x0074, 0x0066, 0x0000];

    ZeroMemory!(
        &raw mut sfn as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<OPENFILENAMEW>() as usize,
    );

    sfn.lStructSize = ::core::mem::size_of::<OPENFILENAMEW>() as DWORD;
    sfn.Flags = (OFN_EXPLORER | OFN_HIDEREADONLY | OFN_PATHMUSTEXIST | OFN_OVERWRITEPROMPT | OFN_ENABLESIZING) as DWORD;
    sfn.hwndOwner = hMainWnd;
    sfn.lpstrFilter = &raw mut wszFilter as *mut WCHAR as LPCWSTR;
    sfn.lpstrFile = &raw mut wszFile as *mut WCHAR as LPWSTR;
    sfn.nMaxFile = MAX_PATH as DWORD;
    sfn.lpstrDefExt = &raw const wszDefExt as *const WCHAR as LPCWSTR;
    sfn.nFilterIndex = (fileformat_number(fileFormat) + 1) as DWORD;

    while GetSaveFileNameW(&raw mut sfn) != 0 {
        if fileformat_flags(sfn.nFilterIndex.wrapping_sub(1 as DWORD) as ::core::ffi::c_int) != SF_RTF as WPARAM {
            if MessageBoxWithResStringW(
                hMainWnd,
                MAKEINTRESOURCEW!(STRING_SAVE_LOSEFORMATTING) as LPCWSTR,
                &raw const wszAppTitle as LPCWSTR,
                (MB_YESNO | MB_ICONEXCLAMATION) as UINT,
            ) != IDYES as ::core::ffi::c_int
            {
                continue;
            }
        }
        return DoSaveFile(
            sfn.lpstrFile as LPCWSTR,
            fileformat_flags(sfn.nFilterIndex.wrapping_sub(1 as DWORD) as ::core::ffi::c_int),
        );
    }
    return FALSE as BOOL;
}

unsafe extern "C" fn prompt_save_changes() -> BOOL {
    if wszFileName[0] == 0 {
        let mut gt: GETTEXTLENGTHEX = core::mem::zeroed();
        gt.flags = GTL_NUMCHARS as DWORD;
        gt.codepage = 1200 as UINT;
        if SendMessageW(hEditorWnd, EM_GETTEXTLENGTHEX, &raw mut gt as WPARAM, 0 as LPARAM) == 0 {
            return (TRUE) as BOOL;
        }
    }

    if SendMessageW(hEditorWnd, EM_GETMODIFY as UINT, 0 as WPARAM, 0 as LPARAM) == 0 {
        return (TRUE) as BOOL;
    } else {
        let mut displayFileName: LPWSTR = core::ptr::null_mut::<WCHAR>();
        let mut text: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
        let mut ret: i32 = 0;

        if wszFileName[0] == 0 {
            displayFileName = &raw mut wszDefaultFileName as *mut WCHAR as LPWSTR;
        } else {
            displayFileName = file_basename(&raw mut wszFileName as LPWSTR);
        }

        text = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            ((lstrlenW(displayFileName as LPCWSTR)
                + lstrlenW(&raw mut wszSaveChanges as *mut WCHAR as LPCWSTR))
                as SIZE_T)
                * (core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;

        if text.is_null() {
            return (FALSE) as BOOL;
        }

        wsprintfW(
            text as LPWSTR,
            &raw mut wszSaveChanges as *mut WCHAR as LPCWSTR,
            displayFileName,
        );

        ret = MessageBoxW(
            hMainWnd,
            text as LPCWSTR,
            &raw const wszAppTitle as LPCWSTR,
            (MB_YESNOCANCEL | MB_ICONEXCLAMATION) as UINT,
        );

        HeapFree(GetProcessHeap(), 0 as DWORD, text as PVOID);

        match (ret) as u32 {
            IDNO => {
                return (TRUE) as BOOL;
            }
            IDYES => {
                if wszFileName[0] != 0 {
                    return DoSaveFile(&raw mut wszFileName as *mut WCHAR as LPCWSTR, fileFormat);
                }
                return DialogSaveFile();
            }
            _ => {
                return (FALSE) as BOOL;
            }
        }
    }
}

unsafe extern "C" fn DialogOpenFile() {
    let mut ofn: OPENFILENAMEW = core::mem::zeroed();
    let mut wszFile: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    static wszDefExt: [WCHAR; 4] = [0x0072u16, 0x0074, 0x0066, 0x0000];

    ZeroMemory!(
        core::ptr::addr_of_mut!(ofn) as *mut core::ffi::c_void,
        core::mem::size_of::<OPENFILENAMEW>(),
    );

    ofn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
    ofn.Flags = (OFN_EXPLORER | OFN_HIDEREADONLY | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_ENABLESIZING) as DWORD;
    ofn.hwndOwner = hMainWnd;
    ofn.lpstrFilter = core::ptr::addr_of_mut!(wszFilter) as *mut WCHAR as LPCWSTR;
    ofn.lpstrFile = core::ptr::addr_of_mut!(wszFile) as *mut WCHAR as LPWSTR;
    ofn.nMaxFile = MAX_PATH as DWORD;
    ofn.lpstrDefExt = core::ptr::addr_of!(wszDefExt) as *const WCHAR as LPCWSTR;
    ofn.nFilterIndex = (fileformat_number(fileFormat) + 1) as DWORD;

    if GetOpenFileNameW(core::ptr::addr_of_mut!(ofn)) != 0 {
        if prompt_save_changes() != 0 {
            DoOpenFile(ofn.lpstrFile as LPCWSTR);
        }
    }
}

unsafe extern "C" fn dialog_about() {
    let icon: HICON = LoadImageW(
        GetModuleHandleW(core::ptr::null_mut()) as HINSTANCE,
        MAKEINTRESOURCEW!(IDI_WORDPAD) as LPCWSTR,
        IMAGE_ICON as UINT,
        48 as ::core::ffi::c_int,
        48 as ::core::ffi::c_int,
        LR_SHARED as UINT,
    ) as HICON;
    ShellAboutW(
        hMainWnd,
        core::ptr::addr_of!(wszAppTitle) as LPCWSTR,
        core::ptr::null_mut(),
        icon,
    );
}

unsafe extern "system" fn formatopts_proc(
    mut hWnd: HWND,
    mut message: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    match message {
        WM_INITDIALOG => {
            let mut ps: LPPROPSHEETPAGEW = lParam as LPPROPSHEETPAGEW;
            let mut wrap: ::core::ffi::c_int = -1;
            let mut id: [::core::ffi::c_char; 4] = [0; 4];
            let mut hIdWnd: HWND = GetDlgItem(hWnd, (IDC_PAGEFMT_ID as i32));
            sprintf(
                core::ptr::addr_of_mut!(id) as *mut ::core::ffi::c_char,
                b"%d\n\0".as_ptr() as *const ::core::ffi::c_char,
                (*ps).lParam as ::core::ffi::c_int,
            );
            SetWindowTextA(
                hIdWnd,
                core::ptr::addr_of_mut!(id) as *mut ::core::ffi::c_char as LPCSTR,
            );
            if wordWrap[(*ps).lParam as usize] == ID_WORDWRAP_NONE {
                wrap = (IDC_PAGEFMT_WN as i32);
            } else if wordWrap[(*ps).lParam as usize] == ID_WORDWRAP_WINDOW {
                wrap = (IDC_PAGEFMT_WW as i32);
            } else if wordWrap[(*ps).lParam as usize] == ID_WORDWRAP_MARGIN {
                wrap = (IDC_PAGEFMT_WM as i32);
            }

            if wrap != -1 {
                CheckRadioButton(hWnd, (IDC_PAGEFMT_WN as i32), (IDC_PAGEFMT_WM as i32), wrap);
            }

            if barState[(*ps).lParam as usize]
                & ((1 << BANDID_TOOLBAR) as DWORD)
                != 0
            {
                CheckDlgButton(hWnd, (IDC_PAGEFMT_TB as i32), TRUE);
            }
            if barState[(*ps).lParam as usize]
                & ((1 << BANDID_FORMATBAR) as DWORD)
                != 0
            {
                CheckDlgButton(hWnd, (IDC_PAGEFMT_FB as i32), TRUE);
            }
            if barState[(*ps).lParam as usize]
                & ((1 << BANDID_RULER) as DWORD)
                != 0
            {
                CheckDlgButton(hWnd, (IDC_PAGEFMT_RU as i32), TRUE);
            }
            if barState[(*ps).lParam as usize]
                & ((1 << BANDID_STATUSBAR) as DWORD)
                != 0
            {
                CheckDlgButton(hWnd, (IDC_PAGEFMT_SB as i32), TRUE);
            }
        }

        WM_COMMAND => match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
            IDC_PAGEFMT_WN | IDC_PAGEFMT_WW | IDC_PAGEFMT_WM => {
                CheckRadioButton(
                    hWnd,
                    (IDC_PAGEFMT_WN as i32),
                    (IDC_PAGEFMT_WM as i32),
                    LOWORD!(wParam) as ::core::ffi::c_int,
                );
            }
            IDC_PAGEFMT_TB
            | IDC_PAGEFMT_FB
            | IDC_PAGEFMT_RU
            | IDC_PAGEFMT_SB => {
                CheckDlgButton(
                    hWnd,
                    LOWORD!(wParam) as ::core::ffi::c_int,
                    (IsDlgButtonChecked(hWnd, LOWORD!(wParam) as ::core::ffi::c_int) == 0)
                        as ::core::ffi::c_int as UINT,
                );
            }
            _ => {}
        },

        WM_NOTIFY => {
            let mut header: LPNMHDR = lParam as LPNMHDR;
            if (*header).code == (PSN_APPLY) as u32 {
                let mut hIdWnd_0: HWND = GetDlgItem(hWnd, (IDC_PAGEFMT_ID as i32));
                let mut sid: [::core::ffi::c_char; 4] =
                    [0; 4];
                let mut id_0: ::core::ffi::c_int = 0;

                GetWindowTextA(
                    hIdWnd_0,
                    core::ptr::addr_of_mut!(sid) as *mut ::core::ffi::c_char as LPSTR,
                    4 as ::core::ffi::c_int,
                );
                id_0 = atoi(core::ptr::addr_of!(sid) as *const u8);
                if IsDlgButtonChecked(hWnd, (IDC_PAGEFMT_WN as i32)) != 0 {
                    wordWrap[id_0 as usize] = ID_WORDWRAP_NONE;
                } else if IsDlgButtonChecked(hWnd, (IDC_PAGEFMT_WW as i32)) != 0 {
                    wordWrap[id_0 as usize] = ID_WORDWRAP_WINDOW;
                } else if IsDlgButtonChecked(hWnd, (IDC_PAGEFMT_WM as i32)) != 0 {
                    wordWrap[id_0 as usize] = ID_WORDWRAP_MARGIN;
                }

                if IsDlgButtonChecked(hWnd, (IDC_PAGEFMT_TB as i32)) != 0 {
                    barState[id_0 as usize] |=
                        ((1 << BANDID_TOOLBAR) as DWORD);
                } else {
                    barState[id_0 as usize] &=
                        !((1 << BANDID_TOOLBAR) as DWORD);
                }

                if IsDlgButtonChecked(hWnd, (IDC_PAGEFMT_FB as i32)) != 0 {
                    barState[id_0 as usize] |=
                        ((1 << BANDID_FORMATBAR) as DWORD);
                } else {
                    barState[id_0 as usize] &=
                        !((1 << BANDID_FORMATBAR) as DWORD);
                }

                if IsDlgButtonChecked(hWnd, (IDC_PAGEFMT_RU as i32)) != 0 {
                    barState[id_0 as usize] |=
                        ((1 << BANDID_RULER) as DWORD);
                } else {
                    barState[id_0 as usize] &=
                        !((1 << BANDID_RULER) as DWORD);
                }

                if IsDlgButtonChecked(hWnd, (IDC_PAGEFMT_SB as i32)) != 0 {
                    barState[id_0 as usize] |=
                        ((1 << BANDID_STATUSBAR) as DWORD);
                } else {
                    barState[id_0 as usize] &=
                        !((1 << BANDID_STATUSBAR) as DWORD);
                }
            }
        }

        _ => {}
    }
    return FALSE as INT_PTR;
}

unsafe extern "C" fn dialog_viewproperties() {
    let mut psp: [PROPSHEETPAGEW; 2] = core::mem::zeroed();
    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
    let mut i: size_t = 0;
    let mut hInstance: HINSTANCE = GetModuleHandleW(0 as LPCWSTR) as HINSTANCE;
    let mut ppsp: LPCPROPSHEETPAGEW = core::ptr::addr_of_mut!(psp) as LPCPROPSHEETPAGEW;

    psp[0].dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
    psp[0].dwFlags = PSP_USETITLE as DWORD;
    psp[0].__bindgen_anon_1.pszTemplate = MAKEINTRESOURCEW!(IDD_FORMATOPTS) as LPCWSTR;
    psp[0].pfnDlgProc =
        Some(formatopts_proc);
    psp[0].hInstance = hInstance;
    psp[0].lParam = reg_formatindex((SF_TEXT) as WPARAM) as LPARAM;
    psp[0].pfnCallback = None;
    psp[0].pszTitle = MAKEINTRESOURCEW!(STRING_VIEWPROPS_TEXT) as LPCWSTR;
    i = 1 as size_t;
    while (i as usize)
        < (core::mem::size_of_val(&psp) as usize)
            .wrapping_div(core::mem::size_of_val(&psp[0]) as usize)
    {
        psp[i].dwSize = psp[0].dwSize;
        psp[i].dwFlags = psp[0].dwFlags;
        psp[i].__bindgen_anon_1.pszTemplate = psp[0].__bindgen_anon_1.pszTemplate;
        psp[i].pfnDlgProc = psp[0].pfnDlgProc;
        psp[i].hInstance = psp[0].hInstance;
        psp[i].lParam = reg_formatindex((SF_RTF) as WPARAM) as LPARAM;
        psp[i].pfnCallback = psp[0].pfnCallback;
        psp[i].pszTitle = MAKEINTRESOURCEW!(STRING_VIEWPROPS_RICHTEXT) as LPCWSTR;
        i = i.wrapping_add(1);
    }

    psh.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as DWORD;
    psh.dwFlags = (PSH_USEICONID | PSH_PROPSHEETPAGE | PSH_NOAPPLYNOW) as DWORD;
    psh.hwndParent = hMainWnd;
    psh.hInstance = hInstance;
    psh.pszCaption = MAKEINTRESOURCEW!(STRING_VIEWPROPS_TITLE) as LPCWSTR;
    psh.nPages = ((core::mem::size_of_val(&psp) as usize)
        .wrapping_div(core::mem::size_of_val(&psp[0]) as usize)) as UINT;
    psh.__bindgen_anon_3.ppsp = ppsp;
    psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDI_WORDPAD) as LPCWSTR;

    if (fileFormat & (SF_RTF as WPARAM)) != 0 {
        psh.__bindgen_anon_2.nStartPage = 1 as UINT;
    } else {
        psh.__bindgen_anon_2.nStartPage = 0 as UINT;
    }
    PropertySheetW(core::ptr::addr_of_mut!(psh) as LPCPROPSHEETHEADERW);
    set_bar_states();
    target_device(hMainWnd, wordWrap[reg_formatindex(fileFormat) as usize]);
}

unsafe extern "C" fn HandleCommandLine(mut cmdline: LPWSTR) {
    let mut delimiter: WCHAR = 0;
    let mut opt_print: BOOL = (FALSE as i32);
    /* skip white space */
    while *cmdline == ' ' as WCHAR {
        cmdline = cmdline.offset(1);
    }
    /* skip executable name */
    delimiter = (if *cmdline == '"' as WCHAR { '"' as WCHAR } else { ' ' as WCHAR }) as WCHAR;
    if *cmdline == delimiter {
        cmdline = cmdline.offset(1);
    }
    while *cmdline != 0 && *cmdline != delimiter {
        cmdline = cmdline.offset(1);
    }
    if *cmdline == delimiter {
        cmdline = cmdline.offset(1);
    }
    while *cmdline != 0 {
        while isspace(*cmdline as i32) != 0 {
            cmdline = cmdline.offset(1);
        }
        if *cmdline == '-' as WCHAR || *cmdline == '/' as WCHAR {
            if *cmdline.offset(2 as isize) == 0
                || isspace(*cmdline.offset(2 as isize) as i32) != 0
            {
                match *cmdline.offset(1 as isize) as i32 {
                    x if x == 'P' as i32 || x == 'p' as i32 => {
                        opt_print = (TRUE as i32);
                        cmdline = cmdline.offset(2 as isize);
                        continue;
                    }
                    _ => {}
                }
            }
            /* a filename starting by / */
        }
        break;
    }
    if *cmdline != 0 {
        /* file name is passed on the command line */
        if *cmdline.offset(0 as isize) == '"' as WCHAR {
            cmdline = cmdline.offset(1);
            *cmdline.offset(
                (lstrlenW(cmdline as LPCWSTR) - 1) as isize,
            ) = 0 as WCHAR;
        }
        DoOpenFile(cmdline as LPCWSTR);
        InvalidateRect(hMainWnd, core::ptr::null_mut(), (FALSE) as BOOL);
    }
    if opt_print != 0 {
        MessageBoxWithResStringW(
            hMainWnd,
            MAKEINTRESOURCEW!(STRING_PRINTING_NOT_IMPLEMENTED),
            core::ptr::addr_of!(wszAppTitle) as LPCWSTR,
            MB_OK as UINT,
        );
    }
}

unsafe extern "C" fn handle_findmsg(pFr: LPFINDREPLACEW) -> LRESULT {
    if (*pFr).Flags & FR_DIALOGTERM != 0 {
        hFindWnd = core::ptr::null_mut();
        (*pFr).Flags = FR_FINDNEXT;
        return 0;
    }

    if (*pFr).Flags & FR_FINDNEXT != 0
        || (*pFr).Flags & FR_REPLACE != 0
        || (*pFr).Flags & FR_REPLACEALL != 0
    {
        let mut custom_data: *mut FINDREPLACE_custom = (*pFr).lCustData as *mut FINDREPLACE_custom;
        let mut flags: DWORD = 0;
        let mut ft: FINDTEXTEXW = core::mem::zeroed();
        let mut sel: CHARRANGE = core::mem::zeroed();
        let mut ret: LRESULT = -1;
        let mut hMenu: HMENU = GetMenu(hMainWnd);
        let mut mi: MENUITEMINFOW = core::mem::zeroed();

        mi.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
        mi.fMask = MIIM_DATA;
        mi.dwItemData = 1;
        SetMenuItemInfoW(hMenu, ID_FIND_NEXT, (FALSE) as BOOL, core::ptr::addr_of_mut!(mi));

        /* Make sure find field is saved. */
        if (*pFr).lpstrFindWhat != core::ptr::addr_of_mut!((*custom_data).findBuffer) as *mut WCHAR {
            lstrcpynW(
                core::ptr::addr_of_mut!((*custom_data).findBuffer) as LPWSTR,
                (*pFr).lpstrFindWhat as LPCWSTR,
                (core::mem::size_of::<[WCHAR; 128]>() / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
            );
            (*pFr).lpstrFindWhat = core::ptr::addr_of_mut!((*custom_data).findBuffer) as *mut WCHAR as LPWSTR;
        }

        SendMessageW(
            hEditorWnd,
            EM_GETSEL,
            core::ptr::addr_of_mut!(sel.cpMin) as WPARAM,
            core::ptr::addr_of_mut!(sel.cpMax) as LPARAM,
        );
        if (*custom_data).endPos == -1 {
            (*custom_data).endPos = sel.cpMin as ::core::ffi::c_int;
            (*custom_data).wrapped = (FALSE as i32);
        }

        flags = FR_DOWN | (*pFr).Flags & (FR_MATCHCASE | FR_WHOLEWORD);
        ft.lpstrText = (*pFr).lpstrFindWhat as LPCWSTR;

        /* Only replace the existing selection if it is an exact match. */
        if sel.cpMin != sel.cpMax
            && ((*pFr).Flags & FR_REPLACE != 0
                || (*pFr).Flags & FR_REPLACEALL != 0)
        {
            ft.chrg = sel;
            SendMessageW(
                hEditorWnd,
                EM_FINDTEXTEXW,
                flags as WPARAM,
                core::ptr::addr_of_mut!(ft) as LPARAM,
            );
            if ft.chrgText.cpMin == sel.cpMin && ft.chrgText.cpMax == sel.cpMax {
                SendMessageW(
                    hEditorWnd,
                    EM_REPLACESEL,
                    (TRUE) as WPARAM,
                    (*pFr).lpstrReplaceWith as LPARAM,
                );
                SendMessageW(
                    hEditorWnd,
                    EM_GETSEL,
                    core::ptr::addr_of_mut!(sel.cpMin) as WPARAM,
                    core::ptr::addr_of_mut!(sel.cpMax) as LPARAM,
                );
            }
        }

        /* Search from the start of the selection, but exclude the first character
         * from search if there is a selection. */
        ft.chrg.cpMin = sel.cpMin;
        if sel.cpMin != sel.cpMax {
            ft.chrg.cpMin += 1;
        }

        /* Search to the end, then wrap around and search from the start. */
        if (*custom_data).wrapped == 0 {
            ft.chrg.cpMax = -1;
            ret = SendMessageW(
                hEditorWnd,
                EM_FINDTEXTEXW,
                flags as WPARAM,
                core::ptr::addr_of_mut!(ft) as LPARAM,
            );
            if ret == -1 {
                (*custom_data).wrapped = (TRUE as i32);
                ft.chrg.cpMin = 0;
            }
        }

        if ret == -1 {
            ft.chrg.cpMax = ((*custom_data).endPos
                + lstrlenW((*pFr).lpstrFindWhat as LPCWSTR)
                - 1) as LONG;
            if ft.chrg.cpMax > ft.chrg.cpMin {
                ret = SendMessageW(
                    hEditorWnd,
                    EM_FINDTEXTEXW,
                    flags as WPARAM,
                    core::ptr::addr_of_mut!(ft) as LPARAM,
                );
            }
        }

        if ret == -1 {
            (*custom_data).endPos = -1;
            EnableWindow(hMainWnd, (FALSE) as BOOL);
            MessageBoxWithResStringW(
                hFindWnd,
                MAKEINTRESOURCEW!(STRING_SEARCH_FINISHED),
                core::ptr::addr_of!(wszAppTitle) as LPCWSTR,
                MB_OK | MB_ICONASTERISK | MB_TASKMODAL,
            );
            EnableWindow(hMainWnd, (TRUE) as BOOL);
        } else {
            SendMessageW(
                hEditorWnd,
                EM_SETSEL,
                ft.chrgText.cpMin as WPARAM,
                ft.chrgText.cpMax as LPARAM,
            );
            SendMessageW(
                hEditorWnd,
                EM_SCROLLCARET,
                0 as WPARAM,
                0 as LPARAM,
            );

            if (*pFr).Flags & FR_REPLACEALL != 0 {
                return handle_findmsg(pFr);
            }
        }
    }

    return 0;
}

unsafe extern "C" fn dialog_find(fr: LPFINDREPLACEW, replace: BOOL) {
    static mut selBuffer: [WCHAR; 128] = [0; 128];
    static mut replaceBuffer: [WCHAR; 128] = [0; 128];
    static mut custom_data: FINDREPLACE_custom = FINDREPLACE_custom {
        endPos: 0,
        wrapped: 0,
        findBuffer: [0; 128],
    };
    static mut endl: WCHAR = '\r' as u16;
    let mut ft: FINDTEXTW = core::mem::zeroed();

    /* Allow only one search/replace dialog to open */
    if !hFindWnd.is_null() {
        SetActiveWindow(hFindWnd);
        return;
    }

    ZeroMemory!(fr as *mut core::ffi::c_void, core::mem::size_of::<FINDREPLACEW>() as usize);
    (*fr).lStructSize = core::mem::size_of::<FINDREPLACEW>() as DWORD;
    (*fr).hwndOwner = hMainWnd;
    (*fr).Flags = FR_HIDEUPDOWN as DWORD;
    /* Find field is filled with the selected text if it is non-empty
     * and stays within the same paragraph, otherwise the previous
     * find field is used. */
    SendMessageW(
        hEditorWnd,
        EM_GETSEL as UINT,
        core::ptr::addr_of_mut!(ft.chrg.cpMin) as WPARAM,
        core::ptr::addr_of_mut!(ft.chrg.cpMax) as LPARAM,
    );
    ft.lpstrText = core::ptr::addr_of!(endl) as LPCWSTR;
    if ft.chrg.cpMin != ft.chrg.cpMax
        && SendMessageW(
            hEditorWnd,
            EM_FINDTEXTW as UINT,
            FR_DOWN as WPARAM,
            core::ptr::addr_of_mut!(ft) as LPARAM,
        ) == -(1 as LRESULT)
    {
        /* Use a temporary buffer for the selected text so that the saved
         * find field is only overwritten when a find/replace is clicked. */
        let mut gt: GETTEXTEX = GETTEXTEX {
            cb: core::mem::size_of::<[WCHAR; 128]>() as DWORD,
            flags: GT_SELECTION as DWORD,
            codepage: 1200 as UINT,
            lpDefaultChar: core::ptr::null(),
            lpUsedDefChar: core::ptr::null_mut(),
        };
        SendMessageW(
            hEditorWnd,
            EM_GETTEXTEX as UINT,
            core::ptr::addr_of_mut!(gt) as WPARAM,
            core::ptr::addr_of_mut!(selBuffer) as *mut WCHAR as LPARAM,
        );
        (*fr).lpstrFindWhat = core::ptr::addr_of_mut!(selBuffer) as *mut WCHAR as LPWSTR;
    } else {
        (*fr).lpstrFindWhat = core::ptr::addr_of_mut!(custom_data.findBuffer) as *mut WCHAR as LPWSTR;
    }
    (*fr).lpstrReplaceWith = core::ptr::addr_of_mut!(replaceBuffer) as *mut WCHAR as LPWSTR;
    custom_data.endPos = -(1 as i32);
    custom_data.wrapped = FALSE as BOOL;
    (*fr).lCustData = core::ptr::addr_of_mut!(custom_data) as LPARAM;
    (*fr).wFindWhatLen = core::mem::size_of::<[WCHAR; 128]>() as WORD;
    (*fr).wReplaceWithLen = core::mem::size_of::<[WCHAR; 128]>() as WORD;

    if replace != 0 {
        hFindWnd = ReplaceTextW(fr);
    } else {
        hFindWnd = FindTextW(fr);
    }
}

unsafe extern "C" fn units_to_twips(mut unit: UNIT, mut number: ::core::ffi::c_float) -> ::core::ffi::c_int {
    let mut twips: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if unit as ::core::ffi::c_uint == UNIT_CM as ::core::ffi::c_uint {
        twips = (number as ::core::ffi::c_double * 1000.0f64
            / CENTMM_PER_INCH as ::core::ffi::c_float as ::core::ffi::c_double
            * TWIPS_PER_INCH as ::core::ffi::c_float as ::core::ffi::c_double)
            as ::core::ffi::c_int;
    } else if unit as ::core::ffi::c_uint == UNIT_INCH as ::core::ffi::c_uint {
        twips = (number * TWIPS_PER_INCH as ::core::ffi::c_float) as ::core::ffi::c_int;
    } else if unit as ::core::ffi::c_uint == UNIT_PT as ::core::ffi::c_uint {
        twips = (number as ::core::ffi::c_double
            * (0.0138f64 * TWIPS_PER_INCH as ::core::ffi::c_float as ::core::ffi::c_double))
            as ::core::ffi::c_int;
    }
    return twips;
}

unsafe extern "C" fn append_current_units(buffer: LPWSTR) {
    static space: [u16; 2] = [0x0020u16, 0x0000];
    lstrcatW(buffer, space.as_ptr());
    lstrcatW(buffer, core::ptr::addr_of_mut!(units_cmW) as *mut WCHAR as LPCWSTR);
}

unsafe extern "C" fn number_with_units(buffer: LPWSTR, number: ::core::ffi::c_int) {
    static mut fmt: [u16; 8] = [0x0025u16, 0x002E, 0x0032, 0x0066, 0x0020, 0x0025, 0x0073, 0x0000];
    // C: `(float)number / (float)TWIPS_PER_INCH * (float)CENTMM_PER_INCH / 1000.0`, the last
    // step in double. The translation had appended a `/ 0`: every value printed as inf/NaN.
    let converted: f32 =
        (((number as f32) / (TWIPS_PER_INCH as f32) * (CENTMM_PER_INCH as f32)) as f64 / 1000.0) as f32;
    sprintfW(buffer, core::ptr::addr_of!(fmt) as *const WCHAR, converted as f64, core::ptr::addr_of_mut!(units_cmW) as *mut WCHAR);
}

unsafe extern "C" fn get_comboexlist_selection(
    mut hComboEx: HWND,
    mut wszBuffer: LPWSTR,
    mut bufferLength: UINT,
) -> BOOL {
    let mut cbItem: COMBOBOXEXITEMW = core::mem::zeroed();
    let mut cbInfo: COMBOBOXINFO = core::mem::zeroed();
    let mut hCombo: HWND = core::ptr::null_mut::<HWND__>();
    let mut hList: HWND = core::ptr::null_mut::<HWND__>();
    let mut idx: ::core::ffi::c_int = 0;
    let mut result: ::core::ffi::c_int = 0;
    hCombo = SendMessageW(
        hComboEx,
        CBEM_GETCOMBOCONTROL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as HWND;
    if hCombo.is_null() {
        return (FALSE) as BOOL;
    }
    cbInfo.cbSize = ::core::mem::size_of::<COMBOBOXINFO>() as DWORD;
    result = SendMessageW(
        hCombo,
        CB_GETCOMBOBOXINFO as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(cbInfo) as LPARAM,
    ) as ::core::ffi::c_int;
    if result == 0 {
        return (FALSE) as BOOL;
    }
    hList = cbInfo.hwndList;
    idx = SendMessageW(
        hList,
        LB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as ::core::ffi::c_int;
    if idx < 0 {
        return (FALSE) as BOOL;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(cbItem) as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<COMBOBOXEXITEMW>() as size_t,
    );
    cbItem.mask = CBEIF_TEXT as UINT;
    cbItem.iItem = idx as INT_PTR;
    cbItem.pszText = wszBuffer;
    cbItem.cchTextMax = bufferLength.wrapping_sub(1 as UINT) as ::core::ffi::c_int;
    result = SendMessageW(
        hComboEx,
        CBEM_GETITEMW as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(cbItem) as LPARAM,
    ) as ::core::ffi::c_int;

    return (result != 0) as ::core::ffi::c_int;
}
unsafe extern "system" fn datetime_proc(mut hWnd: HWND, mut message: UINT, mut wParam: WPARAM, mut lParam: LPARAM) -> INT_PTR {
    match message {
        WM_INITDIALOG => {
            let mut buffer: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
            let mut st: SYSTEMTIME = core::mem::zeroed();
            let mut hListWnd: HWND = GetDlgItem(hWnd, (IDC_DATETIME as i32));
            GetLocalTime(core::ptr::addr_of_mut!(st));

            GetDateFormatW(LOCALE_USER_DEFAULT, DATE_SHORTDATE, core::ptr::addr_of_mut!(st), core::ptr::null(), core::ptr::addr_of_mut!(buffer) as LPWSTR,
                (MAX_STRING_LEN as i32));
            SendMessageW(hListWnd, LB_ADDSTRING as UINT, 0 as WPARAM, core::ptr::addr_of_mut!(buffer) as LPARAM);
            GetDateFormatW(LOCALE_USER_DEFAULT, DATE_LONGDATE, core::ptr::addr_of_mut!(st), core::ptr::null(), core::ptr::addr_of_mut!(buffer) as LPWSTR,
                (MAX_STRING_LEN as i32));
            SendMessageW(hListWnd, LB_ADDSTRING as UINT, 0 as WPARAM, core::ptr::addr_of_mut!(buffer) as LPARAM);
            GetTimeFormatW(LOCALE_USER_DEFAULT, 0, core::ptr::addr_of_mut!(st), core::ptr::null(), core::ptr::addr_of_mut!(buffer) as LPWSTR, (MAX_STRING_LEN as i32));
            SendMessageW(hListWnd, LB_ADDSTRING as UINT, 0 as WPARAM, core::ptr::addr_of_mut!(buffer) as LPARAM);

            SendMessageW(hListWnd, LB_SETSEL as UINT, TRUE as WPARAM, 0 as LPARAM);
        }

        WM_COMMAND => {
            let mut at: u8 = match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_DATETIME => 0,
                IDOK => 1,
                IDCANCEL => 2,
                _ => 3,
            };
            loop {
                match at {
                    0 => {
                        if HIWORD!(wParam) != (LBN_DBLCLK as u16) {
                            break;
                        }
                        at = 1;
                        continue;
                    }
                    1 => {
                        {
                            let mut index: LRESULT = 0;
                            let mut hListWnd_0: HWND = GetDlgItem(hWnd, (IDC_DATETIME as i32));

                            index = SendMessageW(hListWnd_0, LB_GETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM);

                            if index != LB_ERR as LRESULT {
                                let mut buffer_0: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
                                SendMessageW(hListWnd_0, LB_GETTEXT as UINT, index as WPARAM, core::ptr::addr_of_mut!(buffer_0) as LPARAM);
                                SendMessageW(hEditorWnd, EM_REPLACESEL as UINT, TRUE as WPARAM, core::ptr::addr_of_mut!(buffer_0) as LPARAM);
                            }
                        }
                        at = 2;
                        continue;
                    }
                    2 => {
                        EndDialog(hWnd, (wParam) as INT_PTR);
                        return TRUE as INT_PTR;
                    }
                    _ => break,
                }
            }
        }

        _ => {}
    }
    return FALSE as INT_PTR;
}

unsafe extern "system" fn newfile_proc(hWnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match message {
        WM_INITDIALOG => {
            let mut hInstance: HINSTANCE =
                GetModuleHandleW(0 as *const WCHAR) as HINSTANCE;
            let mut buffer: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
            let mut hListWnd: HWND =
                GetDlgItem(hWnd, IDC_NEWFILE as ::core::ffi::c_int);

            LoadStringW(
                hInstance,
                STRING_NEWFILE_RICHTEXT as UINT,
                buffer.as_mut_ptr() as LPWSTR,
                MAX_STRING_LEN as ::core::ffi::c_int,
            );
            SendMessageW(
                hListWnd,
                LB_ADDSTRING as UINT,
                0 as WPARAM,
                core::ptr::addr_of_mut!(buffer) as LPARAM,
            );
            LoadStringW(
                hInstance,
                STRING_NEWFILE_TXT as UINT,
                buffer.as_mut_ptr() as LPWSTR,
                MAX_STRING_LEN as ::core::ffi::c_int,
            );
            SendMessageW(
                hListWnd,
                LB_ADDSTRING as UINT,
                0 as WPARAM,
                core::ptr::addr_of_mut!(buffer) as LPARAM,
            );
            LoadStringW(
                hInstance,
                STRING_NEWFILE_TXT_UNICODE as UINT,
                buffer.as_mut_ptr() as LPWSTR,
                MAX_STRING_LEN as ::core::ffi::c_int,
            );
            SendMessageW(
                hListWnd,
                LB_ADDSTRING as UINT,
                0 as WPARAM,
                core::ptr::addr_of_mut!(buffer) as LPARAM,
            );

            SendMessageW(
                hListWnd,
                LB_SETSEL as UINT,
                TRUE as WPARAM,
                0 as LPARAM,
            );
        }

        WM_COMMAND => match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
            IDOK => {
                let mut index: LRESULT = 0;
                let mut hListWnd: HWND =
                    GetDlgItem(hWnd, IDC_NEWFILE as ::core::ffi::c_int);
                index = SendMessageW(
                    hListWnd,
                    LB_GETCURSEL as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                );

                if index != LB_ERR as LRESULT {
                    EndDialog(
                        hWnd,
                        MAKELONG!(
                            fileformat_flags(index as ::core::ffi::c_int) as WORD,
                            0 as WORD,
                        ) as INT_PTR,
                    );
                }
                return TRUE as INT_PTR;
            }

            IDCANCEL => {
                EndDialog(
                    hWnd,
                    MAKELONG!(
                        ID_NEWFILE_ABORT as WORD,
                        0 as WORD,
                    ) as INT_PTR,
                );
                return TRUE as INT_PTR;
            }

            _ => {}
        },

        _ => {}
    }
    return FALSE as INT_PTR;
}

unsafe extern "system" fn paraformat_proc(hWnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    static ALIGNMENT_VALUES: [WORD; 3] = [PFA_LEFT as WORD, PFA_RIGHT as WORD, PFA_CENTER as WORD];

    match message {
        WM_INITDIALOG => {
            let mut hInstance: HINSTANCE = GetModuleHandleW(core::ptr::null()) as HINSTANCE;
            let mut buffer: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
            let mut hListWnd: HWND = GetDlgItem(hWnd, (IDC_PARA_ALIGN as i32));
            let mut hLeftWnd: HWND = GetDlgItem(hWnd, (IDC_PARA_LEFT as i32));
            let mut hRightWnd: HWND = GetDlgItem(hWnd, (IDC_PARA_RIGHT as i32));
            let mut hFirstWnd: HWND = GetDlgItem(hWnd, (IDC_PARA_FIRST as i32));
            let mut pf: PARAFORMAT2 = core::mem::zeroed();
            let mut index: ::core::ffi::c_int = 0;

            LoadStringW(hInstance, STRING_ALIGN_LEFT, addr_of_mut!(buffer) as LPWSTR,
                        (MAX_STRING_LEN as i32));
            SendMessageW(hListWnd, CB_ADDSTRING, 0, addr_of_mut!(buffer) as LPARAM);
            LoadStringW(hInstance, STRING_ALIGN_RIGHT, addr_of_mut!(buffer) as LPWSTR,
                        (MAX_STRING_LEN as i32));
            SendMessageW(hListWnd, CB_ADDSTRING, 0, addr_of_mut!(buffer) as LPARAM);
            LoadStringW(hInstance, STRING_ALIGN_CENTER, addr_of_mut!(buffer) as LPWSTR,
                        (MAX_STRING_LEN as i32));
            SendMessageW(hListWnd, CB_ADDSTRING, 0, addr_of_mut!(buffer) as LPARAM);

            pf.cbSize = ::core::mem::size_of::<PARAFORMAT2>() as UINT;
            pf.dwMask = (PFM_ALIGNMENT | PFM_OFFSET | PFM_RIGHTINDENT |
                        PFM_STARTINDENT) as DWORD;
            SendMessageW(hEditorWnd, EM_GETPARAFORMAT, 0, addr_of_mut!(pf) as LPARAM);

            if (pf.wAlignment as ::core::ffi::c_int) == (PFA_RIGHT as i32) {
                index += 1;
            } else if (pf.wAlignment as ::core::ffi::c_int) == (PFA_CENTER as i32) {
                index += 2;
            }

            SendMessageW(hListWnd, CB_SETCURSEL, index as WPARAM, 0);

            number_with_units(addr_of_mut!(buffer) as LPWSTR, (pf.dxStartIndent + pf.dxOffset) as ::core::ffi::c_int);
            SetWindowTextW(hLeftWnd, addr_of_mut!(buffer) as LPCWSTR);
            number_with_units(addr_of_mut!(buffer) as LPWSTR, pf.dxRightIndent as ::core::ffi::c_int);
            SetWindowTextW(hRightWnd, addr_of_mut!(buffer) as LPCWSTR);
            number_with_units(addr_of_mut!(buffer) as LPWSTR, -pf.dxOffset as ::core::ffi::c_int);
            SetWindowTextW(hFirstWnd, addr_of_mut!(buffer) as LPCWSTR);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDOK => {
                    let mut hListWnd: HWND = GetDlgItem(hWnd, (IDC_PARA_ALIGN as i32));
                    let mut hLeftWnd: HWND = GetDlgItem(hWnd, (IDC_PARA_LEFT as i32));
                    let mut hRightWnd: HWND = GetDlgItem(hWnd, (IDC_PARA_RIGHT as i32));
                    let mut hFirstWnd: HWND = GetDlgItem(hWnd, (IDC_PARA_FIRST as i32));
                    let mut buffer: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
                    let mut index: ::core::ffi::c_int = 0;
                    let mut num: f32 = 0.0;
                    let mut ret: ::core::ffi::c_int = 0;
                    let mut pf: PARAFORMAT = core::mem::zeroed();
                    let mut unit: UNIT = UNIT_CM;

                    index = SendMessageW(hListWnd, CB_GETCURSEL, 0, 0) as ::core::ffi::c_int;
                    // The C indexes with CB_GETCURSEL's result unchecked (CB_ERR reads before
                    // the table); a Rust index would panic, so CB_ERR keeps the left alignment.
                    pf.wAlignment = *ALIGNMENT_VALUES.get(index as usize).unwrap_or(&ALIGNMENT_VALUES[0]);

                    GetWindowTextW(hLeftWnd, addr_of_mut!(buffer) as LPWSTR, (MAX_STRING_LEN as i32));
                    if number_from_string(addr_of_mut!(buffer) as LPCWSTR, addr_of_mut!(num), addr_of_mut!(unit)) != 0 {
                        ret += 1;
                    }
                    pf.dxOffset = units_to_twips(unit, num) as LONG;
                    GetWindowTextW(hRightWnd, addr_of_mut!(buffer) as LPWSTR, (MAX_STRING_LEN as i32));
                    if number_from_string(addr_of_mut!(buffer) as LPCWSTR, addr_of_mut!(num), addr_of_mut!(unit)) != 0 {
                        ret += 1;
                    }
                    pf.dxRightIndent = units_to_twips(unit, num) as LONG;
                    GetWindowTextW(hFirstWnd, addr_of_mut!(buffer) as LPWSTR, (MAX_STRING_LEN as i32));
                    if number_from_string(addr_of_mut!(buffer) as LPCWSTR, addr_of_mut!(num), addr_of_mut!(unit)) != 0 {
                        ret += 1;
                    }
                    pf.dxStartIndent = units_to_twips(unit, num) as LONG;

                    if ret != 3 {
                        MessageBoxWithResStringW(hMainWnd, MAKEINTRESOURCEW!(STRING_INVALID_NUMBER),
                                    core::ptr::addr_of!(wszAppTitle) as LPCWSTR, MB_OK | MB_ICONASTERISK);
                        return FALSE as INT_PTR;
                    } else {
                        if pf.dxOffset + pf.dxStartIndent < 0
                            && pf.dxStartIndent < 0
                        {
                            /* The first line is before the left edge, so
                             * make sure it is at the left edge. */
                            pf.dxOffset = -pf.dxStartIndent;
                        } else if pf.dxOffset < 0 {
                            /* The second and following lines are before
                             * the left edge, so set it to be at the left
                             * edge, and adjust the first line since it
                             * is relative to it. */
                            pf.dxStartIndent = max!(pf.dxStartIndent + pf.dxOffset, 0);
                            pf.dxOffset = 0;
                        }
                        /* Internally the dxStartIndent is the absolute
                         * offset for the first line and dxOffset is
                         * to it value as opposed how it is displayed with
                         * the first line being the relative value.
                         * These two lines make the adjustments. */
                        pf.dxStartIndent = pf.dxStartIndent + pf.dxOffset;
                        pf.dxOffset = pf.dxOffset - pf.dxStartIndent;

                        pf.cbSize = ::core::mem::size_of::<PARAFORMAT>() as UINT;
                        pf.dwMask = (PFM_ALIGNMENT | PFM_OFFSET | PFM_RIGHTINDENT |
                                    PFM_STARTINDENT) as DWORD;
                        SendMessageW(hEditorWnd, EM_SETPARAFORMAT, 0, addr_of_mut!(pf) as LPARAM);
                    }
                    EndDialog(hWnd, (wParam) as INT_PTR);
                    return TRUE as INT_PTR;
                }

                IDCANCEL => {
                    EndDialog(hWnd, (wParam) as INT_PTR);
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }
    return FALSE as INT_PTR;
}

unsafe extern "system" fn tabstops_proc(hWnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match message {
        WM_INITDIALOG => {
            let mut hTabWnd: HWND = GetDlgItem(hWnd, IDC_TABSTOPS as ::core::ffi::c_int);
            let mut pf: PARAFORMAT = core::mem::zeroed();
            let mut buffer: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
            let mut i: ::core::ffi::c_int;

            pf.cbSize = ::core::mem::size_of::<PARAFORMAT>() as UINT;
            pf.dwMask = PFM_TABSTOPS as DWORD;
            SendMessageW(
                hEditorWnd,
                EM_GETPARAFORMAT as UINT,
                0 as WPARAM,
                core::ptr::addr_of_mut!(pf) as LPARAM,
            );
            SendMessageW(
                hTabWnd,
                CB_LIMITTEXT as UINT,
                (MAX_STRING_LEN - 1) as WPARAM,
                0 as LPARAM,
            );

            i = 0 as ::core::ffi::c_int;
            while i < pf.cTabCount as ::core::ffi::c_int {
                number_with_units(
                    core::ptr::addr_of_mut!(buffer) as LPWSTR,
                    pf.rgxTabs[i as usize] as ::core::ffi::c_int,
                );
                SendMessageW(
                    hTabWnd,
                    CB_ADDSTRING as UINT,
                    0 as WPARAM,
                    core::ptr::addr_of_mut!(buffer) as LPARAM,
                );
                i += 1;
            }
            SetFocus(hTabWnd);
        }

        WM_COMMAND => {
            'sw: {
                match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                    IDC_TABSTOPS => {
                        let mut hTabWnd: HWND = lParam as HWND;
                        let mut hAddWnd: HWND = GetDlgItem(hWnd, ID_TAB_ADD as ::core::ffi::c_int);
                        let mut hDelWnd: HWND = GetDlgItem(hWnd, ID_TAB_DEL as ::core::ffi::c_int);
                        let mut hEmptyWnd: HWND = GetDlgItem(hWnd, ID_TAB_EMPTY as ::core::ffi::c_int);

                        if GetWindowTextLengthW(hTabWnd) != 0 {
                            EnableWindow(hAddWnd, TRUE as BOOL);
                        } else {
                            EnableWindow(hAddWnd, FALSE as BOOL);
                        }

                        if SendMessageW(
                            hTabWnd,
                            CB_GETCOUNT as UINT,
                            0 as WPARAM,
                            0 as LPARAM,
                        ) != 0
                        {
                            EnableWindow(hEmptyWnd, TRUE as BOOL);

                            if SendMessageW(
                                hTabWnd,
                                CB_GETCURSEL as UINT,
                                0 as WPARAM,
                                0 as LPARAM,
                            ) == CB_ERR as LRESULT
                            {
                                EnableWindow(hDelWnd, FALSE as BOOL);
                            } else {
                                EnableWindow(hDelWnd, TRUE as BOOL);
                            }
                        } else {
                            EnableWindow(hEmptyWnd, FALSE as BOOL);
                        }
                        break 'sw;
                    }

                    ID_TAB_ADD => {
                        let mut hTabWnd: HWND = GetDlgItem(hWnd, IDC_TABSTOPS as ::core::ffi::c_int);
                        let mut buffer: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
                        let mut unit: UNIT = core::mem::zeroed();

                        GetWindowTextW(
                            hTabWnd,
                            core::ptr::addr_of_mut!(buffer) as LPWSTR,
                            MAX_STRING_LEN as ::core::ffi::c_int,
                        );
                        append_current_units(core::ptr::addr_of_mut!(buffer) as LPWSTR);

                        if SendMessageW(
                            hTabWnd,
                            CB_FINDSTRINGEXACT as UINT,
                            (-1 as ::core::ffi::c_int) as WPARAM,
                            core::ptr::addr_of_mut!(buffer) as LPARAM,
                        ) == CB_ERR as LRESULT
                        {
                            let mut number: ::core::ffi::c_float = 0 as ::core::ffi::c_float;
                            let mut item_count: ::core::ffi::c_int = SendMessageW(
                                hTabWnd,
                                CB_GETCOUNT as UINT,
                                0 as WPARAM,
                                0 as LPARAM,
                            ) as ::core::ffi::c_int;

                            if number_from_string(
                                core::ptr::addr_of_mut!(buffer) as LPCWSTR,
                                core::ptr::addr_of_mut!(number),
                                core::ptr::addr_of_mut!(unit),
                            ) == 0
                            {
                                MessageBoxWithResStringW(
                                    hWnd,
                                    MAKEINTRESOURCEW!(STRING_INVALID_NUMBER) as LPCWSTR,
                                    core::ptr::addr_of!(wszAppTitle) as LPCWSTR,
                                    (MB_OK | MB_ICONINFORMATION) as UINT,
                                );
                            } else if item_count >= MAX_TAB_STOPS as ::core::ffi::c_int {
                                MessageBoxWithResStringW(
                                    hWnd,
                                    MAKEINTRESOURCEW!(STRING_MAX_TAB_STOPS) as LPCWSTR,
                                    core::ptr::addr_of!(wszAppTitle) as LPCWSTR,
                                    (MB_OK | MB_ICONINFORMATION) as UINT,
                                );
                            } else {
                                let mut i: ::core::ffi::c_int;
                                let mut next_number: ::core::ffi::c_float = -(1) as ::core::ffi::c_float;
                                let mut next_number_in_twips: ::core::ffi::c_int = -(1) as ::core::ffi::c_int;
                                let mut insert_number: ::core::ffi::c_int = units_to_twips(unit, number);

                                /* linear search for position to insert the string */
                                i = 0 as ::core::ffi::c_int;
                                while i < item_count {
                                    SendMessageW(
                                        hTabWnd,
                                        CB_GETLBTEXT as UINT,
                                        i as WPARAM,
                                        core::ptr::addr_of_mut!(buffer) as LPARAM,
                                    );
                                    number_from_string(
                                        core::ptr::addr_of_mut!(buffer) as LPCWSTR,
                                        core::ptr::addr_of_mut!(next_number),
                                        core::ptr::addr_of_mut!(unit),
                                    );
                                    next_number_in_twips = units_to_twips(unit, next_number);
                                    if insert_number <= next_number_in_twips {
                                        break;
                                    }
                                    i += 1;
                                }
                                if insert_number != next_number_in_twips {
                                    number_with_units(core::ptr::addr_of_mut!(buffer) as LPWSTR, insert_number);
                                    SendMessageW(
                                        hTabWnd,
                                        CB_INSERTSTRING as UINT,
                                        i as WPARAM,
                                        core::ptr::addr_of_mut!(buffer) as LPARAM,
                                    );
                                    SetWindowTextW(hTabWnd, 0 as LPWSTR);
                                }
                            }
                        }
                        SetFocus(hTabWnd);
                        break 'sw;
                    }

                    ID_TAB_DEL => {
                        let mut hTabWnd: HWND = GetDlgItem(hWnd, IDC_TABSTOPS as ::core::ffi::c_int);
                        let mut ret: LRESULT;
                        ret = SendMessageW(
                            hTabWnd,
                            CB_GETCURSEL as UINT,
                            0 as WPARAM,
                            0 as LPARAM,
                        );
                        if ret != CB_ERR as LRESULT {
                            SendMessageW(
                                hTabWnd,
                                CB_DELETESTRING as UINT,
                                ret as WPARAM,
                                0 as LPARAM,
                            );
                        }
                        break 'sw;
                    }

                    ID_TAB_EMPTY => {
                        let mut hTabWnd: HWND = GetDlgItem(hWnd, IDC_TABSTOPS as ::core::ffi::c_int);
                        SendMessageW(
                            hTabWnd,
                            CB_RESETCONTENT as UINT,
                            0 as WPARAM,
                            0 as LPARAM,
                        );
                        SetFocus(hTabWnd);
                        break 'sw;
                    }

                    IDOK => {
                        let mut hTabWnd: HWND = GetDlgItem(hWnd, IDC_TABSTOPS as ::core::ffi::c_int);
                        let mut i: ::core::ffi::c_int;
                        let mut buffer: [WCHAR; (MAX_STRING_LEN as _)] = [0; (MAX_STRING_LEN as _)];
                        let mut pf: PARAFORMAT = core::mem::zeroed();
                        let mut number: ::core::ffi::c_float = 0.0;
                        let mut unit: UNIT = core::mem::zeroed();

                        pf.cbSize = ::core::mem::size_of::<PARAFORMAT>() as UINT;
                        pf.dwMask = PFM_TABSTOPS as DWORD;

                        i = 0 as ::core::ffi::c_int;
                        while SendMessageW(
                            hTabWnd,
                            CB_GETLBTEXT as UINT,
                            i as WPARAM,
                            core::ptr::addr_of_mut!(buffer) as LPARAM,
                        ) != CB_ERR as LRESULT
                            && i < MAX_TAB_STOPS as ::core::ffi::c_int
                        {
                            number_from_string(
                                core::ptr::addr_of_mut!(buffer) as LPCWSTR,
                                core::ptr::addr_of_mut!(number),
                                core::ptr::addr_of_mut!(unit),
                            );
                            pf.rgxTabs[i as usize] = units_to_twips(unit, number) as LONG;
                            i += 1;
                        }
                        pf.cTabCount = i as SHORT;
                        SendMessageW(
                            hEditorWnd,
                            EM_SETPARAFORMAT as UINT,
                            0 as WPARAM,
                            core::ptr::addr_of_mut!(pf) as LPARAM,
                        );
                        /* Fall through */
                        EndDialog(hWnd, (wParam) as INT_PTR);
                        return TRUE as INT_PTR;
                    }

                    IDCANCEL => {
                        EndDialog(hWnd, (wParam) as INT_PTR);
                        return TRUE as INT_PTR;
                    }

                    _ => {}
                }
            }
        }

        _ => {}
    }
    return FALSE as INT_PTR;
}

unsafe extern "C" fn OnCreate(hWnd: HWND) -> LRESULT {
    let mut hToolBarWnd: HWND;
    let mut hFormatBarWnd: HWND;
    let mut hReBarWnd: HWND;
    let mut hFontListWnd: HWND;
    let mut hSizeListWnd: HWND;
    let mut hRulerWnd: HWND;
    let mut hInstance: HINSTANCE = GetModuleHandleW(core::ptr::null());
    let mut hDLL: HANDLE;
    let mut ab: TBADDBITMAP = core::mem::zeroed();
    let mut nStdBitmaps: i32 = 0;
    let mut rbi: REBARINFO = core::mem::zeroed();
    let mut rbb: REBARBANDINFOW = core::mem::zeroed();
    static wszRichEditDll: [u16; 13] = [
    0x0052u16, 0x0049, 0x0043, 0x0048, 0x0045, 0x0044, 0x0032, 0x0030, 0x002E, 0x0044,
    0x004C, 0x004C, 0x0000,
];
    static wszRichEditText: [u16; 14] = [
    0x0052u16, 0x0069, 0x0063, 0x0068, 0x0045, 0x0064, 0x0069, 0x0074, 0x0020, 0x0074,
    0x0065, 0x0078, 0x0074, 0x0000,
];

    CreateStatusWindowW(
        (CCS_NODIVIDER | WS_CHILD | WS_VISIBLE) as LONG,
        core::ptr::addr_of!(wszRichEditText) as LPCWSTR,
        hWnd,
        IDC_STATUSBAR as UINT,
    );

    hReBarWnd = CreateWindowExW(
        WS_EX_TOOLWINDOW,
        (REBARCLASSNAMEW).as_ptr(),
        core::ptr::null::<WCHAR>(),
        (CCS_NODIVIDER | WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WS_CLIPCHILDREN | RBS_VARHEIGHT | CCS_TOP) as DWORD,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        0,
        0,
        hWnd,
        (IDC_REBAR as HMENU),
        hInstance,
        core::ptr::null_mut(),
    );

    rbi.cbSize = core::mem::size_of::<REBARINFO>() as UINT;
    rbi.fMask = 0 as UINT;
    rbi.himl = core::ptr::null_mut();
    if SendMessageW(
        hReBarWnd,
        RB_SETBARINFO as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(rbi) as LPARAM,
    ) == 0
    {
        return (-(1 as i32)) as LRESULT;
    }

    hToolBarWnd = CreateToolbarEx(
        hReBarWnd,
        (CCS_NOPARENTALIGN | CCS_NOMOVEY | WS_VISIBLE | WS_CHILD | TBSTYLE_TOOLTIPS | BTNS_BUTTON) as DWORD,
        IDC_TOOLBAR as UINT,
        1,
        hInstance,
        IDB_TOOLBAR as UINT_PTR,
        core::ptr::null::<TBBUTTON>(),
        0,
        24,
        24,
        16,
        16,
        core::mem::size_of::<TBBUTTON>() as UINT,
    );

    ab.hInst = HINST_COMMCTRL;
    ab.nID = IDB_STD_SMALL_COLOR as UINT_PTR;
    nStdBitmaps = SendMessageW(
        hToolBarWnd,
        TB_ADDBITMAP as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(ab) as LPARAM,
    ) as i32;

    AddButton(hToolBarWnd, nStdBitmaps + (STD_FILENEW as i32), (ID_FILE_NEW as i32));
    AddButton(hToolBarWnd, nStdBitmaps + (STD_FILEOPEN as i32), (ID_FILE_OPEN as i32));
    AddButton(hToolBarWnd, nStdBitmaps + (STD_FILESAVE as i32), (ID_FILE_SAVE as i32));
    AddSeparator(hToolBarWnd);
    AddButton(hToolBarWnd, nStdBitmaps + (STD_PRINT as i32), (ID_PRINT_QUICK as i32));
    AddButton(hToolBarWnd, nStdBitmaps + (STD_PRINTPRE as i32), (ID_PREVIEW as i32));
    AddSeparator(hToolBarWnd);
    AddButton(hToolBarWnd, nStdBitmaps + (STD_FIND as i32), (ID_FIND as i32));
    AddSeparator(hToolBarWnd);
    AddButton(hToolBarWnd, nStdBitmaps + (STD_CUT as i32), (ID_EDIT_CUT as i32));
    AddButton(hToolBarWnd, nStdBitmaps + (STD_COPY as i32), (ID_EDIT_COPY as i32));
    AddButton(hToolBarWnd, nStdBitmaps + (STD_PASTE as i32), (ID_EDIT_PASTE as i32));
    AddButton(hToolBarWnd, nStdBitmaps + (STD_UNDO as i32), (ID_EDIT_UNDO as i32));
    AddButton(hToolBarWnd, nStdBitmaps + (STD_REDOW as i32), (ID_EDIT_REDO as i32));
    AddSeparator(hToolBarWnd);
    AddButton(hToolBarWnd, 0, (ID_DATETIME as i32));

    SendMessageW(hToolBarWnd, TB_AUTOSIZE as UINT, 0 as WPARAM, 0 as LPARAM);

    rbb.cbSize = REBARBANDINFOW_V6_SIZE as UINT;
    rbb.fMask = (RBBIM_SIZE | RBBIM_CHILDSIZE | RBBIM_CHILD | RBBIM_STYLE | RBBIM_ID) as UINT;
    rbb.fStyle = (RBBS_CHILDEDGE | RBBS_BREAK | RBBS_NOGRIPPER) as UINT;
    rbb.cx = 0 as UINT;
    rbb.hwndChild = hToolBarWnd;
    rbb.cxMinChild = 0 as UINT;
    rbb.cyMinChild = HIWORD!(SendMessageW(
        hToolBarWnd,
        TB_GETBUTTONSIZE as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as DWORD_PTR) as UINT;
    rbb.cyChild = rbb.cyMinChild;
    rbb.wID = BANDID_TOOLBAR as UINT;

    SendMessageW(
        hReBarWnd,
        RB_INSERTBANDW as UINT,
        (-(1 as i32)) as WPARAM,
        core::ptr::addr_of_mut!(rbb) as LPARAM,
    );

    hFontListWnd = CreateWindowExW(
        0,
        (WC_COMBOBOXEXW).as_ptr(),
        core::ptr::null::<WCHAR>(),
        (WS_BORDER | WS_VISIBLE | WS_CHILD | CBS_DROPDOWN | CBS_SORT) as DWORD,
        0,
        0,
        200,
        150,
        hReBarWnd,
        (IDC_FONTLIST as HMENU),
        hInstance,
        core::ptr::null_mut(),
    );

    rbb.hwndChild = hFontListWnd;
    rbb.cx = 200 as UINT;
    rbb.wID = BANDID_FONTLIST as UINT;

    SendMessageW(
        hReBarWnd,
        RB_INSERTBANDW as UINT,
        (-(1 as i32)) as WPARAM,
        core::ptr::addr_of_mut!(rbb) as LPARAM,
    );

    hSizeListWnd = CreateWindowExW(
        0,
        (WC_COMBOBOXEXW).as_ptr(),
        core::ptr::null::<WCHAR>(),
        (WS_BORDER | WS_VISIBLE | WS_CHILD | CBS_DROPDOWN) as DWORD,
        0,
        0,
        50,
        150,
        hReBarWnd,
        (IDC_SIZELIST as HMENU),
        hInstance,
        core::ptr::null_mut(),
    );

    rbb.hwndChild = hSizeListWnd;
    rbb.cx = 50 as UINT;
    rbb.fStyle ^= RBBS_BREAK as UINT;
    rbb.wID = BANDID_SIZELIST as UINT;

    SendMessageW(
        hReBarWnd,
        RB_INSERTBANDW as UINT,
        (-(1 as i32)) as WPARAM,
        core::ptr::addr_of_mut!(rbb) as LPARAM,
    );

    hFormatBarWnd = CreateToolbarEx(
        hReBarWnd,
        (CCS_NOPARENTALIGN | CCS_NOMOVEY | WS_VISIBLE | TBSTYLE_TOOLTIPS | BTNS_BUTTON) as DWORD,
        IDC_FORMATBAR as UINT,
        8,
        hInstance,
        IDB_FORMATBAR as UINT_PTR,
        core::ptr::null::<TBBUTTON>(),
        0,
        16,
        16,
        16,
        16,
        core::mem::size_of::<TBBUTTON>() as UINT,
    );

    AddButton(hFormatBarWnd, 0, (ID_FORMAT_BOLD as i32));
    AddButton(hFormatBarWnd, 1, (ID_FORMAT_ITALIC as i32));
    AddButton(hFormatBarWnd, 2, (ID_FORMAT_UNDERLINE as i32));
    AddButton(hFormatBarWnd, 3, (ID_FORMAT_COLOR as i32));
    AddSeparator(hFormatBarWnd);
    AddButton(hFormatBarWnd, 4, (ID_ALIGN_LEFT as i32));
    AddButton(hFormatBarWnd, 5, (ID_ALIGN_CENTER as i32));
    AddButton(hFormatBarWnd, 6, (ID_ALIGN_RIGHT as i32));
    AddSeparator(hFormatBarWnd);
    AddButton(hFormatBarWnd, 7, (ID_BULLET as i32));

    SendMessageW(
        hFormatBarWnd,
        TB_AUTOSIZE as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    rbb.hwndChild = hFormatBarWnd;
    rbb.wID = BANDID_FORMATBAR as UINT;

    SendMessageW(
        hReBarWnd,
        RB_INSERTBANDW as UINT,
        (-(1 as i32)) as WPARAM,
        core::ptr::addr_of_mut!(rbb) as LPARAM,
    );

    hRulerWnd = CreateWindowExW(
        0,
        (WC_STATICW).as_ptr(),
        core::ptr::null::<WCHAR>(),
        (WS_VISIBLE | WS_CHILD) as DWORD,
        0,
        0,
        200,
        10,
        hReBarWnd,
        (IDC_RULER as HMENU),
        hInstance,
        core::ptr::null_mut(),
    );

    rbb.hwndChild = hRulerWnd;
    rbb.wID = BANDID_RULER as UINT;
    rbb.fStyle |= RBBS_BREAK as UINT;

    SendMessageW(
        hReBarWnd,
        RB_INSERTBANDW as UINT,
        (-(1 as i32)) as WPARAM,
        core::ptr::addr_of_mut!(rbb) as LPARAM,
    );

    hDLL = (LoadLibraryW(core::ptr::addr_of!(wszRichEditDll) as LPCWSTR)).cast();
    if hDLL.is_null() {
        MessageBoxWithResStringW(
            hWnd,
            MAKEINTRESOURCEW!(STRING_LOAD_RICHED_FAILED),
            core::ptr::addr_of!(wszAppTitle) as LPCWSTR,
            (MB_OK | MB_ICONEXCLAMATION) as UINT,
        );
        PostQuitMessage(1);
    }

    hEditorWnd = CreateWindowExW(
        WS_EX_CLIENTEDGE,
        (RICHEDIT_CLASS20W).as_ptr(),
        core::ptr::null::<WCHAR>(),
        (WS_CHILD | WS_VISIBLE | ES_SELECTIONBAR | ES_MULTILINE | ES_AUTOVSCROLL
            | ES_WANTRETURN | WS_VSCROLL | ES_NOHIDESEL | WS_HSCROLL) as DWORD,
        0,
        0,
        1000,
        100,
        hWnd,
        (IDC_EDITOR as HMENU),
        hInstance,
        core::ptr::null_mut(),
    );

    if hEditorWnd.is_null() {
        fprintf(
            __iob_func().add(2),
            b"Error code %u\n\0".as_ptr() as *const i8,
            GetLastError(),
        );
        return (-(1 as i32)) as LRESULT;
    }
    assert!(!hEditorWnd.is_null());

    setup_richedit_olecallback(hEditorWnd);
    SetFocus(hEditorWnd);
    SendMessageW(
        hEditorWnd,
        EM_SETEVENTMASK as UINT,
        0 as WPARAM,
        ENM_SELCHANGE as LPARAM,
    );

    set_default_font();

    populate_font_list(hFontListWnd);
    populate_size_list(hSizeListWnd);
    DoLoadStrings();
    SendMessageW(
        hEditorWnd,
        EM_SETMODIFY as UINT,
        FALSE as WPARAM,
        0 as LPARAM,
    );

    ID_FINDMSGSTRING = RegisterWindowMessageW((FINDMSGSTRINGW).as_ptr());

    registry_read_filelist(hWnd);
    registry_read_formatopts_all(
        core::ptr::addr_of!(barState) as *mut DWORD,
        core::ptr::addr_of!(wordWrap) as *mut DWORD,
    );
    registry_read_options();
    DragAcceptFiles(hWnd, TRUE as BOOL);

    return 0 as LRESULT;
}

unsafe extern "C" fn OnUser(hWnd: HWND) -> LRESULT {
    let hwndEditor: HWND = GetDlgItem(hWnd, (IDC_EDITOR as i32));
    let hwndReBar: HWND = GetDlgItem(hWnd, (IDC_REBAR as i32));
    let hwndToolBar: HWND = GetDlgItem(hwndReBar, (IDC_TOOLBAR as i32));
    let hwndFormatBar: HWND = GetDlgItem(hwndReBar, (IDC_FORMATBAR as i32));
    let mut from: i32 = 0;
    let mut to: i32 = 0;
    let mut fmt: CHARFORMAT2W = core::mem::zeroed();
    let mut pf: PARAFORMAT2 = core::mem::zeroed();
    let mut gt: GETTEXTLENGTHEX = core::mem::zeroed();

    ZeroMemory!(core::ptr::addr_of_mut!(fmt) as *mut core::ffi::c_void, core::mem::size_of::<CHARFORMAT2W>());
    fmt.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;

    ZeroMemory!(core::ptr::addr_of_mut!(pf) as *mut core::ffi::c_void, core::mem::size_of::<PARAFORMAT2>());
    pf.cbSize = core::mem::size_of::<PARAFORMAT2>() as UINT;

    gt.flags = GTL_NUMCHARS;
    gt.codepage = 1200;

    SendMessageW(hwndToolBar, TB_ENABLEBUTTON, (ID_FIND) as WPARAM,
                 if SendMessageW(hwndEditor, EM_GETTEXTLENGTHEX, core::ptr::addr_of_mut!(gt) as WPARAM, 0) != 0 { 1 } else { 0 } as LPARAM);

    SendMessageW(hwndEditor, EM_GETCHARFORMAT, (TRUE) as WPARAM, core::ptr::addr_of_mut!(fmt) as LPARAM);

    SendMessageW(hwndEditor, EM_GETSEL, core::ptr::addr_of_mut!(from) as WPARAM, core::ptr::addr_of_mut!(to) as LPARAM);
    SendMessageW(hwndToolBar, TB_ENABLEBUTTON, (ID_EDIT_UNDO) as WPARAM,
      SendMessageW(hwndEditor, EM_CANUNDO, 0, 0) as LPARAM);
    SendMessageW(hwndToolBar, TB_ENABLEBUTTON, (ID_EDIT_REDO) as WPARAM,
      SendMessageW(hwndEditor, EM_CANREDO, 0, 0) as LPARAM);
    SendMessageW(hwndToolBar, TB_ENABLEBUTTON, (ID_EDIT_CUT) as WPARAM, if from == to { 0 } else { 1 });
    SendMessageW(hwndToolBar, TB_ENABLEBUTTON, (ID_EDIT_COPY) as WPARAM, if from == to { 0 } else { 1 });

    SendMessageW(hwndFormatBar, TB_CHECKBUTTON, (ID_FORMAT_BOLD) as WPARAM, ((fmt.dwMask & CFM_BOLD) != 0 &&
            (fmt.dwEffects & CFE_BOLD) != 0) as LPARAM);
    SendMessageW(hwndFormatBar, TB_INDETERMINATE, (ID_FORMAT_BOLD) as WPARAM, ((fmt.dwMask & CFM_BOLD) == 0) as LPARAM);
    SendMessageW(hwndFormatBar, TB_CHECKBUTTON, (ID_FORMAT_ITALIC) as WPARAM, ((fmt.dwMask & CFM_ITALIC) != 0 &&
            (fmt.dwEffects & CFE_ITALIC) != 0) as LPARAM);
    SendMessageW(hwndFormatBar, TB_INDETERMINATE, (ID_FORMAT_ITALIC) as WPARAM, ((fmt.dwMask & CFM_ITALIC) == 0) as LPARAM);
    SendMessageW(hwndFormatBar, TB_CHECKBUTTON, (ID_FORMAT_UNDERLINE) as WPARAM, ((fmt.dwMask & CFM_UNDERLINE) != 0 &&
            (fmt.dwEffects & CFE_UNDERLINE) != 0) as LPARAM);
    SendMessageW(hwndFormatBar, TB_INDETERMINATE, (ID_FORMAT_UNDERLINE) as WPARAM, ((fmt.dwMask & CFM_UNDERLINE) == 0) as LPARAM);

    SendMessageW(hwndEditor, EM_GETPARAFORMAT, 0, core::ptr::addr_of_mut!(pf) as LPARAM);
    SendMessageW(hwndFormatBar, TB_CHECKBUTTON, (ID_ALIGN_LEFT) as WPARAM, (pf.wAlignment == (PFA_LEFT as u16)) as LPARAM);
    SendMessageW(hwndFormatBar, TB_CHECKBUTTON, (ID_ALIGN_CENTER) as WPARAM, (pf.wAlignment == (PFA_CENTER as u16)) as LPARAM);
    SendMessageW(hwndFormatBar, TB_CHECKBUTTON, (ID_ALIGN_RIGHT) as WPARAM, (pf.wAlignment == (PFA_RIGHT as u16)) as LPARAM);

    SendMessageW(hwndFormatBar, TB_CHECKBUTTON, (ID_BULLET) as WPARAM, ((pf.wNumbering & (PFN_BULLET as u16)) != 0) as LPARAM);
    return 0;
}

unsafe extern "C" fn OnNotify(hWnd: HWND, lParam: LPARAM) -> LRESULT {
    let hwndEditor: HWND = GetDlgItem(hWnd, (IDC_EDITOR as i32));
    let hwndReBar: HWND = GetDlgItem(hWnd, (IDC_REBAR as i32));
    let pHdr: *mut NMHDR = lParam as *mut NMHDR;
    let hwndFontList: HWND = GetDlgItem(hwndReBar, (IDC_FONTLIST as i32));
    let hwndSizeList: HWND = GetDlgItem(hwndReBar, (IDC_SIZELIST as i32));

    if (*pHdr).hwndFrom == hwndFontList || (*pHdr).hwndFrom == hwndSizeList {
        if (*pHdr).code == CBEN_ENDEDITW as UINT {
            let endEdit: *mut NMCBEENDEDITW = lParam as *mut NMCBEENDEDITW;
            if (*pHdr).hwndFrom == hwndFontList {
                on_fontlist_modified(&raw mut (*endEdit).szText as LPWSTR);
            } else if (*pHdr).hwndFrom == hwndSizeList {
                on_sizelist_modified(hwndSizeList, &raw mut (*endEdit).szText as LPWSTR);
            }
        }
        return 0 as LRESULT;
    }

    if (*pHdr).hwndFrom != hwndEditor {
        return 0 as LRESULT;
    }

    if (*pHdr).code == EN_SELCHANGE as UINT {
        let pSC: *mut SELCHANGE = lParam as *mut SELCHANGE;
        let mut buf: [::core::ffi::c_char; 128] = [0; 128];

        update_font_list();

        sprintf(
            &raw mut buf as *mut ::core::ffi::c_char,
            b"Selection: %d..%d | Lines: %u\0" as *const u8 as *const ::core::ffi::c_char,
            (*pSC).chrg.cpMin,
            (*pSC).chrg.cpMax,
            SendMessageW(hwndEditor, EM_GETLINECOUNT as UINT, 0 as WPARAM, 0 as LPARAM) as UINT,
        );
        SetWindowTextA(
            GetDlgItem(hWnd, (IDC_STATUSBAR as i32)),
            &raw mut buf as *mut ::core::ffi::c_char as LPCSTR,
        );
        SendMessageW(hWnd, WM_USER as UINT, 0 as WPARAM, 0 as LPARAM);
        return 1 as LRESULT;
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn OnCommand(mut hWnd: HWND, mut wParam: WPARAM, mut lParam: LPARAM) -> LRESULT {
    let mut hwndEditor: HWND = GetDlgItem(hWnd, (IDC_EDITOR as i32));
    static mut findreplace: FINDREPLACEW = unsafe { core::mem::zeroed() };

    if (lParam as HWND) == hwndEditor {
        return 0;
    }

    match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
        ID_FILE_EXIT => {
            PostMessageW(hWnd, WM_CLOSE, 0, 0);
        }

        ID_FILE_NEW => {
            let mut hInstance: HINSTANCE = GetModuleHandleW(core::ptr::null()) as HINSTANCE;
            let mut ret: ::core::ffi::c_int = DialogBoxW!(
                hInstance,
                MAKEINTRESOURCEW!(IDD_NEWFILE),
                hWnd,
                Some(newfile_proc),
            ) as ::core::ffi::c_int;

            if ret != (ID_NEWFILE_ABORT as i32) {
                if prompt_save_changes() != 0 {
                    let mut st: SETTEXTEX = core::mem::zeroed();

                    set_caption(core::ptr::null());
                    wszFileName[0 as usize] = 0;

                    clear_formatting();

                    st.flags = ST_DEFAULT;
                    st.codepage = 1200;
                    SendMessageW(hEditorWnd, EM_SETTEXTEX, &raw mut st as WPARAM, 0);

                    SendMessageW(hEditorWnd, EM_SETMODIFY, (FALSE) as WPARAM, 0);
                    set_fileformat(ret as WPARAM);
                    update_font_list();
                }
            }
        }

        ID_FILE_OPEN => {
            DialogOpenFile();
        }

        ID_FILE_SAVE => {
            if wszFileName[0 as usize] != 0 {
                DoSaveFile(&raw mut wszFileName as *mut WCHAR as LPCWSTR, fileFormat);
            } else {
                DialogSaveFile();
            }
        }

        ID_FILE_SAVEAS => {
            DialogSaveFile();
        }

        ID_FILE_RECENT1 | ID_FILE_RECENT2 | ID_FILE_RECENT3 | ID_FILE_RECENT4 => {
            let mut hMenu: HMENU = GetMenu(hWnd);
            let mut mi: MENUITEMINFOW = core::mem::zeroed();

            mi.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
            mi.fMask = MIIM_DATA;
            if GetMenuItemInfoW(hMenu, LOWORD!(wParam) as UINT, FALSE as BOOL, &raw mut mi) != 0 {
                DoOpenFile(mi.dwItemData as LPWSTR);
            }
        }

        ID_FIND => {
            dialog_find(&raw mut findreplace, (FALSE) as BOOL);
        }

        ID_FIND_NEXT => {
            handle_findmsg(&raw mut findreplace);
        }

        ID_REPLACE => {
            dialog_find(&raw mut findreplace, (TRUE) as BOOL);
        }

        ID_FONTSETTINGS => {
            dialog_choose_font();
        }

        ID_PRINT => {
            dialog_print(hWnd, &raw mut wszFileName as LPWSTR);
            target_device(hMainWnd, wordWrap[reg_formatindex(fileFormat) as usize]);
        }

        ID_PRINT_QUICK => {
            print_quick(hMainWnd, &raw mut wszFileName as LPWSTR);
            target_device(hMainWnd, wordWrap[reg_formatindex(fileFormat) as usize]);
        }

        ID_PREVIEW => {
            let mut index: ::core::ffi::c_int = reg_formatindex(fileFormat) as ::core::ffi::c_int;
            let mut tmp: DWORD = barState[index as usize];
            barState[index as usize] = (1 << BANDID_STATUSBAR) as DWORD;
            set_bar_states();
            barState[index as usize] = tmp;
            ShowWindow(hEditorWnd, (FALSE as i32));

            init_preview(hWnd, &raw mut wszFileName as LPWSTR);

            SetMenu(hWnd, core::ptr::null_mut());
            InvalidateRect(core::ptr::null_mut(), core::ptr::null(), (TRUE) as BOOL);
        }

        ID_PRINTSETUP => {
            dialog_printsetup(hWnd);
            target_device(hMainWnd, wordWrap[reg_formatindex(fileFormat) as usize]);
        }

        ID_FORMAT_BOLD | ID_FORMAT_ITALIC | ID_FORMAT_UNDERLINE => {
            let mut fmt: CHARFORMAT2W = core::mem::zeroed();
            let mut effects: ::core::ffi::c_int = (CFE_BOLD as i32);

            ZeroMemory!(&raw mut fmt as *mut ::core::ffi::c_void, core::mem::size_of::<CHARFORMAT2W>());
            fmt.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;
            SendMessageW(hwndEditor, EM_GETCHARFORMAT, (SCF_SELECTION) as WPARAM, &raw mut fmt as LPARAM);

            fmt.dwMask = CFM_BOLD;

            if LOWORD!(wParam) == (ID_FORMAT_ITALIC as u16) {
                effects = (CFE_ITALIC as i32);
                fmt.dwMask = CFM_ITALIC;
            } else if LOWORD!(wParam) == (ID_FORMAT_UNDERLINE as u16) {
                effects = (CFE_UNDERLINE as i32);
                fmt.dwMask = CFM_UNDERLINE;
            }

            fmt.dwEffects ^= effects as DWORD;

            SendMessageW(hwndEditor, EM_SETCHARFORMAT, (SCF_SELECTION) as WPARAM, &raw mut fmt as LPARAM);
        }

        ID_FORMAT_COLOR => {
            let mut hReBarWnd: HWND = GetDlgItem(hWnd, (IDC_REBAR as i32));
            let mut hFormatBarWnd: HWND = GetDlgItem(hReBarWnd, (IDC_FORMATBAR as i32));
            let mut hPop: HMENU = core::ptr::null_mut();
            let mut itemrc: RECT = core::mem::zeroed();
            let mut pt: POINT = core::mem::zeroed();
            let mut mid: ::core::ffi::c_int = 0;
            let mut itemidx: ::core::ffi::c_int = SendMessageW(
                hFormatBarWnd,
                TB_COMMANDTOINDEX,
                (ID_FORMAT_COLOR) as WPARAM,
                0,
            ) as ::core::ffi::c_int;

            SendMessageW(hFormatBarWnd, TB_GETITEMRECT, itemidx as WPARAM, &raw mut itemrc as LPARAM);
            pt.x = itemrc.left;
            pt.y = itemrc.bottom;
            ClientToScreen(hFormatBarWnd, &raw mut pt);
            hPop = GetSubMenu(hColorPopupMenu, 0);
            mid = TrackPopupMenu(
                hPop,
                (TPM_LEFTALIGN | TPM_TOPALIGN | TPM_LEFTBUTTON
                    | TPM_RETURNCMD | TPM_NONOTIFY) as UINT,
                pt.x as ::core::ffi::c_int,
                pt.y as ::core::ffi::c_int,
                0,
                hWnd,
                core::ptr::null(),
            ) as ::core::ffi::c_int;
            if mid >= (ID_COLOR_FIRST as i32) && mid <= (ID_COLOR_AUTOMATIC as i32) {
                let mut fmt: CHARFORMAT2W = core::mem::zeroed();

                ZeroMemory!(&raw mut fmt as *mut ::core::ffi::c_void, core::mem::size_of::<CHARFORMAT2W>());
                fmt.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;
                SendMessageW(hwndEditor, EM_GETCHARFORMAT, (SCF_SELECTION) as WPARAM, &raw mut fmt as LPARAM);

                fmt.dwMask = CFM_COLOR;

                if mid < (ID_COLOR_AUTOMATIC as i32) {
                    fmt.crTextColor = textcolors[(mid - (ID_COLOR_FIRST as i32)) as usize];
                    fmt.dwEffects &= !(CFE_AUTOCOLOR as DWORD);
                } else {
                    fmt.dwEffects |= CFE_AUTOCOLOR as DWORD;
                }

                SendMessageW(hwndEditor, EM_SETCHARFORMAT, (SCF_SELECTION) as WPARAM, &raw mut fmt as LPARAM);
            }
        }

        ID_EDIT_CUT => {
            PostMessageW(hwndEditor, WM_CUT, 0, 0);
        }

        ID_EDIT_COPY => {
            PostMessageW(hwndEditor, WM_COPY, 0, 0);
        }

        ID_EDIT_PASTE => {
            PostMessageW(hwndEditor, WM_PASTE, 0, 0);
        }

        ID_EDIT_CLEAR => {
            PostMessageW(hwndEditor, WM_CLEAR, 0, 0);
        }

        ID_EDIT_SELECTALL => {
            let mut range: CHARRANGE = CHARRANGE {
                cpMin: 0,
                cpMax: -1,
            };
            SendMessageW(hwndEditor, EM_EXSETSEL, 0, &raw mut range as LPARAM);
            return 0;
        }

        ID_EDIT_GETTEXT => {
            let mut nLen: ::core::ffi::c_int = GetWindowTextLengthW(hwndEditor) as ::core::ffi::c_int;
            let mut data: LPWSTR = HeapAlloc(
                GetProcessHeap(),
                0,
                (nLen + 1) as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T,
            ) as LPWSTR;
            let mut tr: TEXTRANGEW = core::mem::zeroed();

            GetWindowTextW(hwndEditor, data, ((nLen + 1) as WPARAM) as i32);
            MessageBoxW(core::ptr::null_mut(), data, &raw const wszAppTitle as LPCWSTR, MB_OK);

            HeapFree(GetProcessHeap(), 0, data as PVOID);
            data = HeapAlloc(
                GetProcessHeap(),
                0,
                (nLen + 1) as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T,
            ) as LPWSTR;
            tr.chrg.cpMin = 0;
            tr.chrg.cpMax = nLen as LONG;
            tr.lpstrText = data;
            SendMessageW(hwndEditor, EM_GETTEXTRANGE, 0, &raw mut tr as LPARAM);
            *data.offset((tr.chrg.cpMax - tr.chrg.cpMin) as isize) = UNICODE_NULL;
            MessageBoxW(core::ptr::null_mut(), data, &raw const wszAppTitle as LPCWSTR, MB_OK);
            HeapFree(GetProcessHeap(), 0, data as PVOID);

            return 0;
        }

        ID_EDIT_CHARFORMAT | ID_EDIT_DEFCHARFORMAT => {
            let mut cf: CHARFORMAT2W = core::mem::zeroed();

            ZeroMemory!(&raw mut cf as *mut ::core::ffi::c_void, core::mem::size_of::<CHARFORMAT2W>());
            cf.cbSize = core::mem::size_of::<CHARFORMAT2W>() as UINT;
            cf.dwMask = 0;
            SendMessageW(
                hwndEditor,
                EM_GETCHARFORMAT,
                (LOWORD!(wParam) == (ID_EDIT_CHARFORMAT as u16)) as WPARAM,
                &raw mut cf as LPARAM,
            );
            return 0;
        }

        ID_EDIT_PARAFORMAT => {
            let mut pf: PARAFORMAT2 = core::mem::zeroed();
            ZeroMemory!(&raw mut pf as *mut ::core::ffi::c_void, core::mem::size_of::<PARAFORMAT2>());
            pf.cbSize = core::mem::size_of::<PARAFORMAT2>() as UINT;
            SendMessageW(hwndEditor, EM_GETPARAFORMAT, 0, &raw mut pf as LPARAM);
            return 0;
        }

        ID_EDIT_SELECTIONINFO => {
            let mut range: CHARRANGE = CHARRANGE {
                cpMin: 0,
                cpMax: -1,
            };
            let mut buf: [::core::ffi::c_char; 128] = [0; 128];
            let mut data: *mut WCHAR = core::ptr::null_mut();

            SendMessageW(hwndEditor, EM_EXGETSEL, 0, &raw mut range as LPARAM);
            data = HeapAlloc(
                GetProcessHeap(),
                0,
                core::mem::size_of::<WCHAR>() as SIZE_T * (range.cpMax - range.cpMin + 1) as SIZE_T,
            ) as *mut WCHAR;
            SendMessageW(hwndEditor, EM_GETSELTEXT, 0, data as LPARAM);
            sprintf(
                &raw mut buf as *mut ::core::ffi::c_char,
                b"Start = %d, End = %d\0".as_ptr() as *const ::core::ffi::c_char,
                range.cpMin,
                range.cpMax,
            );
            MessageBoxA(
                hWnd,
                &raw mut buf as *mut ::core::ffi::c_char as LPCSTR,
                b"Editor\0".as_ptr() as LPCSTR,
                MB_OK,
            );
            MessageBoxW(hWnd, data as LPCWSTR, &raw const wszAppTitle as LPCWSTR, MB_OK);
            HeapFree(GetProcessHeap(), 0, data as PVOID);
            return 0;
        }

        ID_EDIT_READONLY => {
            let mut nStyle: LONG = GetWindowLongW(hwndEditor, GWL_STYLE) as LONG;
            if nStyle & (ES_READONLY as i32) != 0 {
                SendMessageW(hwndEditor, EM_SETREADONLY, 0, 0);
            } else {
                SendMessageW(hwndEditor, EM_SETREADONLY, 1, 0);
            }
            return 0;
        }

        ID_EDIT_MODIFIED => {
            if SendMessageW(hwndEditor, EM_GETMODIFY, 0, 0) != 0 {
                SendMessageW(hwndEditor, EM_SETMODIFY, 0, 0);
            } else {
                SendMessageW(hwndEditor, EM_SETMODIFY, 1, 0);
            }
            return 0;
        }

        ID_EDIT_UNDO => {
            SendMessageW(hwndEditor, EM_UNDO, 0, 0);
            return 0;
        }

        ID_EDIT_REDO => {
            SendMessageW(hwndEditor, EM_REDO, 0, 0);
            return 0;
        }

        ID_BULLET => {
            let mut pf: PARAFORMAT = core::mem::zeroed();

            pf.cbSize = core::mem::size_of::<PARAFORMAT>() as UINT;
            pf.dwMask = PFM_NUMBERING;
            SendMessageW(hwndEditor, EM_GETPARAFORMAT, 0, &raw mut pf as LPARAM);

            pf.dwMask |= PFM_OFFSET;

            if pf.wNumbering == (PFN_BULLET as u16) {
                pf.wNumbering = 0;
                pf.dxOffset = 0;
            } else {
                pf.wNumbering = (PFN_BULLET as u16);
                pf.dxOffset = 720;
            }

            SendMessageW(hwndEditor, EM_SETPARAFORMAT, 0, &raw mut pf as LPARAM);
        }

        ID_ALIGN_LEFT | ID_ALIGN_CENTER | ID_ALIGN_RIGHT => {
            let mut pf: PARAFORMAT2 = core::mem::zeroed();

            pf.cbSize = core::mem::size_of::<PARAFORMAT2>() as UINT;
            pf.dwMask = PFM_ALIGNMENT;
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                ID_ALIGN_LEFT => {
                    pf.wAlignment = PFA_LEFT as WORD;
                }
                ID_ALIGN_CENTER => {
                    pf.wAlignment = PFA_CENTER as WORD;
                }
                ID_ALIGN_RIGHT => {
                    pf.wAlignment = PFA_RIGHT as WORD;
                }
                _ => {}
            }
            SendMessageW(hwndEditor, EM_SETPARAFORMAT, 0, &raw mut pf as LPARAM);
        }

        ID_BACK_1 => {
            SendMessageW(hwndEditor, EM_SETBKGNDCOLOR, 1, 0);
        }

        ID_BACK_2 => {
            SendMessageW(
                hwndEditor,
                EM_SETBKGNDCOLOR,
                0,
                (RGB!(255, 255, 192)) as LPARAM,
            );
        }

        ID_TOGGLE_TOOLBAR => {
            set_toolbar_state(
                (BANDID_TOOLBAR as i32),
                (is_bar_visible((BANDID_TOOLBAR as i32)) == 0) as ::core::ffi::c_int,
            );
            update_window();
        }

        ID_TOGGLE_FORMATBAR => {
            set_toolbar_state(
                (BANDID_FONTLIST as i32),
                (is_bar_visible((BANDID_FORMATBAR as i32)) == 0) as ::core::ffi::c_int,
            );
            set_toolbar_state(
                (BANDID_SIZELIST as i32),
                (is_bar_visible((BANDID_FORMATBAR as i32)) == 0) as ::core::ffi::c_int,
            );
            set_toolbar_state(
                (BANDID_FORMATBAR as i32),
                (is_bar_visible((BANDID_FORMATBAR as i32)) == 0) as ::core::ffi::c_int,
            );
            update_window();
        }

        ID_TOGGLE_STATUSBAR => {
            set_statusbar_state((is_bar_visible((BANDID_STATUSBAR as i32)) == 0) as ::core::ffi::c_int);
            update_window();
        }

        ID_TOGGLE_RULER => {
            set_toolbar_state(
                (BANDID_RULER as i32),
                (is_bar_visible((BANDID_RULER as i32)) == 0) as ::core::ffi::c_int,
            );
            update_window();
        }

        ID_DATETIME => {
            DialogBoxW!(
                GetModuleHandleW(core::ptr::null()),
                MAKEINTRESOURCEW!(IDD_DATETIME),
                hWnd,
                Some(datetime_proc),
            );
        }

        ID_PARAFORMAT => {
            DialogBoxW!(
                GetModuleHandleW(core::ptr::null()),
                MAKEINTRESOURCEW!(IDD_PARAFORMAT),
                hWnd,
                Some(paraformat_proc),
            );
        }

        ID_TABSTOPS => {
            DialogBoxW!(
                GetModuleHandleW(core::ptr::null()),
                MAKEINTRESOURCEW!(IDD_TABSTOPS),
                hWnd,
                Some(tabstops_proc),
            );
        }

        ID_ABOUT => {
            dialog_about();
        }

        ID_VIEWPROPERTIES => {
            dialog_viewproperties();
        }

        IDC_FONTLIST => {
            if HIWORD!(wParam) == (CBN_SELENDOK as u16) {
                let mut buffer: [WCHAR; (LF_FACESIZE as _)] = [0; (LF_FACESIZE as _)];
                let mut hwndFontList: HWND = lParam as HWND;
                get_comboexlist_selection(hwndFontList, &raw mut buffer as LPWSTR, LF_FACESIZE);
                on_fontlist_modified(&raw mut buffer as LPWSTR);
            }
        }

        IDC_SIZELIST => {
            if HIWORD!(wParam) == (CBN_SELENDOK as u16) {
                let mut buffer: [WCHAR; (MAX_STRING_LEN + 1) as _] =
                    [0; (MAX_STRING_LEN + 1) as _];
                let mut hwndSizeList: HWND = lParam as HWND;
                get_comboexlist_selection(
                    hwndSizeList,
                    &raw mut buffer as LPWSTR,
                    (MAX_STRING_LEN + 1) as UINT,
                );
                on_sizelist_modified(hwndSizeList, &raw mut buffer as LPWSTR);
            }
        }

        _ => {
            SendMessageW(hwndEditor, WM_COMMAND, wParam, lParam);
        }
    }
    return 0;
}

unsafe extern "C" fn OnInitPopupMenu(hWnd: HWND, wParam: WPARAM) -> LRESULT {
    let mut hMenu: HMENU = wParam as HMENU;
    let mut hwndEditor: HWND = GetDlgItem(hWnd, (IDC_EDITOR as i32));
    let mut hwndStatus: HWND = GetDlgItem(hWnd, (IDC_STATUSBAR as i32));
    let mut pf: PARAFORMAT = core::mem::zeroed();
    let mut nAlignment: ::core::ffi::c_int = -1;
    let mut selFrom: ::core::ffi::c_int = 0;
    let mut selTo: ::core::ffi::c_int = 0;
    let mut gt: GETTEXTLENGTHEX = core::mem::zeroed();
    let mut textLength: LRESULT = 0;
    let mut mi: MENUITEMINFOW = core::mem::zeroed();

    SendMessageW(
        hEditorWnd,
        EM_GETSEL as UINT,
        core::ptr::addr_of_mut!(selFrom) as WPARAM,
        core::ptr::addr_of_mut!(selTo) as LPARAM,
    );
    EnableMenuItem(
        hMenu,
        ID_EDIT_COPY as UINT,
        (if selFrom == selTo { MF_GRAYED } else { MF_ENABLED }) as UINT,
    );
    EnableMenuItem(
        hMenu,
        ID_EDIT_CUT as UINT,
        (if selFrom == selTo { MF_GRAYED } else { MF_ENABLED }) as UINT,
    );

    pf.cbSize = core::mem::size_of::<PARAFORMAT>() as UINT;
    SendMessageW(
        hwndEditor,
        EM_GETPARAFORMAT as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(pf) as LPARAM,
    );
    CheckMenuItem(
        hMenu,
        ID_EDIT_READONLY as UINT,
        (if (GetWindowLongW(hwndEditor, GWL_STYLE) & ES_READONLY as LONG) != 0 {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        }) as UINT,
    );
    CheckMenuItem(
        hMenu,
        ID_EDIT_MODIFIED as UINT,
        (if SendMessageW(
            hwndEditor,
            EM_GETMODIFY as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) != 0
        {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        }) as UINT,
    );
    if (pf.dwMask & PFM_ALIGNMENT) != 0 {
        nAlignment = pf.wAlignment as ::core::ffi::c_int;
    }
    CheckMenuItem(
        hMenu,
        ID_ALIGN_LEFT as UINT,
        (if nAlignment == PFA_LEFT as ::core::ffi::c_int {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        }) as UINT,
    );
    CheckMenuItem(
        hMenu,
        ID_ALIGN_CENTER as UINT,
        (if nAlignment == PFA_CENTER as ::core::ffi::c_int {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        }) as UINT,
    );
    CheckMenuItem(
        hMenu,
        ID_ALIGN_RIGHT as UINT,
        (if nAlignment == PFA_RIGHT as ::core::ffi::c_int {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        }) as UINT,
    );
    CheckMenuItem(
        hMenu,
        ID_BULLET as UINT,
        (if pf.wNumbering as ::core::ffi::c_int == PFN_BULLET as ::core::ffi::c_int {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        }) as UINT,
    );
    EnableMenuItem(
        hMenu,
        ID_EDIT_UNDO as UINT,
        (if SendMessageW(
            hwndEditor,
            EM_CANUNDO as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) != 0
        {
            MF_ENABLED
        } else {
            MF_GRAYED
        }) as UINT,
    );
    EnableMenuItem(
        hMenu,
        ID_EDIT_REDO as UINT,
        (if SendMessageW(
            hwndEditor,
            EM_CANREDO as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) != 0
        {
            MF_ENABLED
        } else {
            MF_GRAYED
        }) as UINT,
    );

    CheckMenuItem(
        hMenu,
        ID_TOGGLE_TOOLBAR as UINT,
        (if is_bar_visible((BANDID_TOOLBAR as i32)) != 0 {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        }) as UINT,
    );

    CheckMenuItem(
        hMenu,
        ID_TOGGLE_FORMATBAR as UINT,
        (if is_bar_visible((BANDID_FORMATBAR as i32)) != 0 {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        }) as UINT,
    );

    CheckMenuItem(
        hMenu,
        ID_TOGGLE_STATUSBAR as UINT,
        (if IsWindowVisible(hwndStatus) != 0 {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        }) as UINT,
    );

    CheckMenuItem(
        hMenu,
        ID_TOGGLE_RULER as UINT,
        (if is_bar_visible((BANDID_RULER as i32)) != 0 {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        }) as UINT,
    );

    gt.flags = GTL_NUMCHARS;
    gt.codepage = 1200;
    textLength = SendMessageW(
        hEditorWnd,
        EM_GETTEXTLENGTHEX as UINT,
        core::ptr::addr_of_mut!(gt) as WPARAM,
        0 as LPARAM,
    );
    EnableMenuItem(
        hMenu,
        ID_FIND as UINT,
        (if textLength != 0 { MF_ENABLED } else { MF_GRAYED }) as UINT,
    );

    mi.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
    mi.fMask = MIIM_DATA as UINT;

    GetMenuItemInfoW(
        hMenu,
        ID_FIND_NEXT as UINT,
        FALSE as BOOL,
        core::ptr::addr_of_mut!(mi),
    );

    EnableMenuItem(
        hMenu,
        ID_FIND_NEXT as UINT,
        (if textLength != 0 && mi.dwItemData != 0 {
            MF_ENABLED
        } else {
            MF_GRAYED
        }) as UINT,
    );

    EnableMenuItem(
        hMenu,
        ID_REPLACE as UINT,
        (if textLength != 0 { MF_ENABLED } else { MF_GRAYED }) as UINT,
    );

    return 0 as LRESULT;
}

unsafe extern "C" fn OnSize(mut hWnd: HWND, mut wParam: WPARAM, mut lParam: LPARAM) -> LRESULT {
    let mut nStatusSize: i32 = 0 as i32;
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hwndEditor: HWND = if preview_isactive() != 0 {
        GetDlgItem(hWnd, IDC_PREVIEW as i32)
    } else {
        GetDlgItem(hWnd, IDC_EDITOR as i32)
    };
    let mut hwndStatusBar: HWND = GetDlgItem(hWnd, IDC_STATUSBAR as i32);
    let mut hwndReBar: HWND = GetDlgItem(hWnd, IDC_REBAR as i32);
    let mut hRulerWnd: HWND = GetDlgItem(hwndReBar, IDC_RULER as i32);
    let mut rebarHeight: i32 = 0 as i32;

    if !hwndStatusBar.is_null() {
        SendMessageW(
            hwndStatusBar,
            WM_SIZE as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
        if IsWindowVisible(hwndStatusBar) != 0 {
            GetClientRect(hwndStatusBar, &raw mut rc);
            nStatusSize = (rc.bottom - rc.top) as i32;
        } else {
            nStatusSize = 0 as i32;
        }
    }
    if !hwndReBar.is_null() {
        rebarHeight = SendMessageW(
            hwndReBar,
            RB_GETBARHEIGHT as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as i32;

        MoveWindow(
            hwndReBar,
            0 as i32,
            0 as i32,
            LOWORD!(lParam) as i32,
            rebarHeight,
            TRUE as i32,
        );
    }
    if !hwndEditor.is_null() {
        GetClientRect(hWnd, &raw mut rc);
        MoveWindow(
            hwndEditor,
            0 as i32,
            rebarHeight,
            rc.right as i32,
            rc.bottom - nStatusSize - rebarHeight,
            TRUE as i32,
        );
    }

    redraw_ruler(hRulerWnd);

    return DefWindowProcW(hWnd, WM_SIZE as UINT, wParam, lParam);
}

unsafe extern "system" fn WndProc(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if msg == ID_FINDMSGSTRING {
        return handle_findmsg(lParam as LPFINDREPLACEW);
    }

    let mut dflt: bool = false;
    match msg {
        WM_CREATE => {
            return OnCreate(hWnd);
        }
        WM_USER => {
            return OnUser(hWnd);
        }
        WM_NOTIFY => {
            return OnNotify(hWnd, lParam);
        }
        WM_COMMAND => {
            if preview_isactive() != 0 {
                return preview_command(hWnd, wParam);
            }
            return OnCommand(hWnd, wParam, lParam);
        }
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        WM_CLOSE => {
            if preview_isactive() != 0 {
                preview_exit(hWnd);
            } else if prompt_save_changes() != 0 {
                registry_set_options(hMainWnd);
                registry_set_formatopts_all(
                    addr_of_mut!(barState) as *mut DWORD,
                    addr_of_mut!(wordWrap) as *mut DWORD,
                );
                PostQuitMessage(0);
            }
        }
        WM_ACTIVATE => {
            if LOWORD!(wParam) != 0 {
                SetFocus(GetDlgItem(hWnd, (IDC_EDITOR as i32)));
            }
            return 0;
        }
        WM_INITMENUPOPUP => {
            return OnInitPopupMenu(hWnd, wParam);
        }
        WM_SIZE => {
            return OnSize(hWnd, wParam, lParam);
        }
        WM_CONTEXTMENU => {
            return DefWindowProcW(hWnd, msg, wParam, lParam);
        }
        WM_DROPFILES => {
            let mut file: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            DragQueryFileW(
                wParam as HDROP,
                0,
                addr_of_mut!(file) as LPWSTR,
                MAX_PATH,
            );
            DragFinish(wParam as HDROP);

            if prompt_save_changes() != 0 {
                DoOpenFile(addr_of_mut!(file) as *mut WCHAR as LPCWSTR);
            }
        }
        WM_PAINT => {
            if preview_isactive() == 0 {
                return DefWindowProcW(hWnd, msg, wParam, lParam);
            }
            dflt = true;
        }
        _ => {
            dflt = true;
        }
    }

    if dflt {
        return DefWindowProcW(hWnd, msg, wParam, lParam);
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn WinMain(
    hInstance: HINSTANCE,
    hOldInstance: HINSTANCE,
    szCmdParagraph: LPSTR,
    mut nCmdShow: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut classes: INITCOMMONCONTROLSEX = INITCOMMONCONTROLSEX {
        dwSize: 8 as DWORD,
        dwICC: (ICC_BAR_CLASSES | ICC_COOL_CLASSES | ICC_USEREX_CLASSES) as DWORD,
    };
    let mut hAccel: HACCEL = core::ptr::null_mut();
    let mut wc: WNDCLASSEXW = core::mem::zeroed();
    let mut msg: MSG = core::mem::zeroed();
    let mut rc: RECT = core::mem::zeroed();
    let mut hPrevRulerProc: UINT_PTR = 0;
    let mut hRulerWnd: HWND = core::ptr::null_mut();
    let mut EditPoint: POINTL = core::mem::zeroed();
    let mut bMaximized: DWORD = 0;
    static wszAccelTable: [u16; 15] = [
    0x004Du16, 0x0041, 0x0049, 0x004E, 0x0041, 0x0043, 0x0043, 0x0045, 0x004C, 0x0054,
    0x0041, 0x0042, 0x004C, 0x0045, 0x0000,
];

    InitCommonControlsEx(core::ptr::addr_of_mut!(classes));

    match GetUserDefaultUILanguage() as ::core::ffi::c_int {
        x if x == MAKELANGID!(LANG_HEBREW as WORD, SUBLANG_DEFAULT as WORD) as ::core::ffi::c_int => {
            SetProcessDefaultLayout(LAYOUT_RTL as DWORD);
        }
        _ => {}
    }

    hAccel = LoadAcceleratorsW(hInstance, core::ptr::addr_of!(wszAccelTable) as *const WCHAR as LPCWSTR);

    wc.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    wc.style = 0 as UINT;
    wc.lpfnWndProc = Some(WndProc);
    wc.cbClsExtra = 0 as ::core::ffi::c_int;
    wc.cbWndExtra = 4 as ::core::ffi::c_int;
    wc.hInstance = hInstance;
    wc.hIcon = LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_WORDPAD) as LPCWSTR);
    wc.hIconSm = LoadImageW(
        hInstance,
        MAKEINTRESOURCEW!(IDI_WORDPAD) as LPCWSTR,
        IMAGE_ICON as UINT,
        GetSystemMetrics(SM_CXSMICON as ::core::ffi::c_int),
        GetSystemMetrics(SM_CYSMICON as ::core::ffi::c_int),
        LR_SHARED as UINT,
    ) as HICON;
    wc.hCursor = LoadCursorW(core::ptr::null_mut(), (IDC_IBEAM as usize) as LPWSTR as LPCWSTR);
    wc.hbrBackground = GetSysColorBrush(COLOR_WINDOW as ::core::ffi::c_int);
    wc.lpszMenuName = MAKEINTRESOURCEW!(IDM_MAINMENU) as LPCWSTR;
    wc.lpszClassName = core::ptr::addr_of!(wszMainWndClass) as *const WCHAR as LPCWSTR;
    RegisterClassExW(core::ptr::addr_of_mut!(wc));

    wc.style = 0 as UINT;
    wc.lpfnWndProc = Some(preview_proc);
    wc.cbClsExtra = 0 as ::core::ffi::c_int;
    wc.cbWndExtra = 0 as ::core::ffi::c_int;
    wc.hInstance = hInstance;
    wc.hIcon = core::ptr::null_mut();
    wc.hIconSm = core::ptr::null_mut();
    wc.hCursor = LoadCursorW(core::ptr::null_mut(), (IDC_IBEAM as usize) as LPWSTR as LPCWSTR);
    wc.hbrBackground = core::ptr::null_mut();
    wc.lpszMenuName = core::ptr::null();
    wc.lpszClassName = core::ptr::addr_of!(wszPreviewWndClass) as *const WCHAR as LPCWSTR;
    RegisterClassExW(core::ptr::addr_of_mut!(wc));

    registry_read_winrect(core::ptr::addr_of_mut!(rc));
    hMainWnd = CreateWindowExW(
        0 as DWORD,
        core::ptr::addr_of!(wszMainWndClass) as *const WCHAR as LPCWSTR,
        core::ptr::addr_of!(wszAppTitle) as *const WCHAR as LPCWSTR,
        (WS_CLIPCHILDREN | WS_OVERLAPPEDWINDOW) as DWORD,
        rc.left as ::core::ffi::c_int,
        rc.top as ::core::ffi::c_int,
        (rc.right - rc.left) as ::core::ffi::c_int,
        (rc.bottom - rc.top) as ::core::ffi::c_int,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        hInstance,
        core::ptr::null_mut(),
    );
    registry_read_maximized(core::ptr::addr_of_mut!(bMaximized));
    if (nCmdShow == SW_SHOWNORMAL as ::core::ffi::c_int || nCmdShow == SW_SHOWDEFAULT as ::core::ffi::c_int)
        && bMaximized != 0
    {
        nCmdShow = SW_SHOWMAXIMIZED as ::core::ffi::c_int;
    }
    ShowWindow(hMainWnd, nCmdShow);

    set_caption(core::ptr::null());
    set_bar_states();
    set_fileformat(SF_RTF as WPARAM);
    hColorPopupMenu = LoadMenuW(hInstance, MAKEINTRESOURCEW!(IDM_COLOR_POPUP) as LPCWSTR);
    get_default_printer_opts();
    target_device(hMainWnd, wordWrap[reg_formatindex(fileFormat) as usize]);

    hRulerWnd = GetDlgItem(
        GetDlgItem(hMainWnd, IDC_REBAR as ::core::ffi::c_int),
        IDC_RULER as ::core::ffi::c_int,
    );
    SendMessageW(
        GetDlgItem(hMainWnd, IDC_EDITOR as ::core::ffi::c_int),
        EM_POSFROMCHAR as UINT,
        core::ptr::addr_of_mut!(EditPoint) as WPARAM,
        0 as LPARAM,
    );
    hPrevRulerProc = SetWindowLongPtrW(
        hRulerWnd,
        GWLP_WNDPROC as ::core::ffi::c_int,
        ((ruler_proc as UINT_PTR) as LONG_PTR) as _,
    ) as UINT_PTR;
    SendMessageW(
        hRulerWnd,
        WM_USER as UINT,
        core::ptr::addr_of_mut!(EditPoint) as WPARAM,
        hPrevRulerProc as LPARAM,
    );

    HandleCommandLine(GetCommandLineW() as LPWSTR);

    while GetMessageW(
        core::ptr::addr_of_mut!(msg),
        0 as HWND,
        0 as UINT,
        0 as UINT,
    ) != 0
    {
        if IsDialogMessageW(hFindWnd, core::ptr::addr_of_mut!(msg)) != 0 {
            continue;
        }

        if TranslateAcceleratorW(hMainWnd, hAccel, core::ptr::addr_of_mut!(msg)) != 0 {
            continue;
        }
        TranslateMessage(core::ptr::addr_of_mut!(msg));
        DispatchMessageW(core::ptr::addr_of_mut!(msg));
        if PeekMessageW(
            core::ptr::addr_of_mut!(msg),
            0 as HWND,
            0 as UINT,
            0 as UINT,
            PM_NOREMOVE as UINT,
        ) == 0
        {
            SendMessageW(
                hMainWnd,
                WM_USER as UINT,
                0 as WPARAM,
                0 as LPARAM,
            );
        }
    }

    return 0 as ::core::ffi::c_int;
}
