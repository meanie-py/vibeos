use crate::eventvwr::evtdetctl::CreateEventDetailsCtrl;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
// Hand port: eventvwr.h's StatusBar_*/ProgressBar_* macros expand to SNDMSG(...), which
// windowsx.h defines as SendMessage -> SendMessageW (UNICODE). The prelude's generated SNDMSG
// names an extern `SendMessage` that user32 does not export (unresolved at link), and its
// tree-consensus SB_SETTEXT is an i32 where SendMessage's Msg is a UINT. Both resolve at the
// macro's call site, so these module-level items shadow the glob for every expansion here.
#[inline(always)]
unsafe fn SNDMSG(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    SendMessageW(hWnd, Msg, wParam, lParam)
}
pub const SB_SETTEXT: UINT = SB_SETTEXTW;
pub static RUSTCONV_WSTR_075DE2B906DB: [u16; 2] = [0x000Au16, 0x0000];
pub static RUSTCONV_WSTR_08C4A5A5D53A: [u16; 26] = [
    0x0048u16, 0x0065, 0x006C, 0x0070, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020, 0x0069,
    0x006D, 0x0070, 0x006C, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0065, 0x0064,
    0x0020, 0x0079, 0x0065, 0x0074, 0x0021, 0x0000,
];
pub static RUSTCONV_WSTR_0AF4B1084E0C: [u16; 8] = [0x004Du16, 0x0061, 0x0078, 0x0053, 0x0069, 0x007A, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_2E5F286A7F7B: [u16; 15] = [
    0x0025u16, 0x0064, 0x0020, 0x0025, 0x0064, 0x0020, 0x0025, 0x0064, 0x0020, 0x0025,
    0x0064, 0x0020, 0x0025, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_2F23369E2DF4: [u16; 10] = [0x0052u16, 0x0065, 0x0074, 0x0065, 0x006E, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_4B164B324B2E: [u16; 7] = [0x0057u16, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0000];
pub static RUSTCONV_WSTR_4F2D892D6AF4: [u16; 2] = [0x0029u16, 0x0000];
pub static RUSTCONV_WSTR_69A283E639E4: [u16; 3] = [0x0025u16, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_6EA98D9C3198: [u16; 10] = [0x0053u16, 0x006F, 0x0072, 0x0074, 0x004F, 0x0072, 0x0064, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_7B514F84AE9A: [u16; 9] = [0x0053u16, 0x0068, 0x006F, 0x0077, 0x0047, 0x0072, 0x0069, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_82294FD79A0D: [u16; 3] = [0x002Cu16, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_8D3D94053677: [u16; 10] = [0x0056u16, 0x0053, 0x0070, 0x006C, 0x0069, 0x0074, 0x0050, 0x006F, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_9711BD294B9C: [u16; 34] = [
    0x0045u16, 0x0076, 0x0065, 0x006E, 0x0074, 0x0020, 0x0049, 0x0044, 0x0020, 0x0028,
    0x0020, 0x0025, 0x006C, 0x0075, 0x0020, 0x0029, 0x002C, 0x0020, 0x0053, 0x006F,
    0x0075, 0x0072, 0x0063, 0x0065, 0x0020, 0x0028, 0x0020, 0x0025, 0x0073, 0x0020,
    0x0029, 0x000A, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_A5D19B228349: [u16; 13] = [
    0x0072u16, 0x0069, 0x0063, 0x0068, 0x0065, 0x0064, 0x0032, 0x0030, 0x002E, 0x0064,
    0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_ACBE5EA9B8F8: [u16; 3] = [0x0020u16, 0x0028, 0x0000];
pub static RUSTCONV_WSTR_B6D17ED8374A: [u16; 3] = [0x0025u16, 0x0025, 0x0000];
pub static RUSTCONV_WSTR_C44D38660B9F: [u16; 16] = [
    0x0044u16, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061, 0x0079, 0x004E, 0x0061, 0x006D,
    0x0065, 0x0046, 0x0069, 0x006C, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_C4BA81AEA105: [u16; 5] = [0x0046u16, 0x0069, 0x006C, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_C543E6446BAB: [u16; 6] = [0x0028u16, 0x0025, 0x006C, 0x0075, 0x0029, 0x0000];
pub static RUSTCONV_WSTR_D01499AAA203: [u16; 19] = [
    0x004Eu16, 0x006F, 0x0020, 0x0073, 0x0065, 0x006C, 0x0065, 0x0063, 0x0074, 0x0065,
    0x0064, 0x0020, 0x0069, 0x0074, 0x0065, 0x006D, 0x0073, 0x0021, 0x0000,
];
pub static RUSTCONV_WSTR_D1046FF7D861: [u16; 13] = [
    0x0053u16, 0x0061, 0x0076, 0x0065, 0x0053, 0x0065, 0x0074, 0x0074, 0x0069, 0x006E,
    0x0067, 0x0073, 0x0000,
];
pub static RUSTCONV_WSTR_DD19C0E56A07: [u16; 45] = [
    0x004Du16, 0x0061, 0x006E, 0x0079, 0x002D, 0x006C, 0x006F, 0x0067, 0x0073, 0x0020,
    0x0066, 0x0069, 0x006C, 0x0074, 0x0065, 0x0072, 0x0069, 0x006E, 0x0067, 0x0020,
    0x0069, 0x0073, 0x0020, 0x006E, 0x006F, 0x0074, 0x0020, 0x0069, 0x006D, 0x0070,
    0x006C, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0065, 0x0064, 0x0020, 0x0079,
    0x0065, 0x0074, 0x0021, 0x0021, 0x0000,
];
pub static RUSTCONV_WSTR_DFF1564CD4E4: [u16; 14] = [
    0x0044u16, 0x0069, 0x0073, 0x0070, 0x006C, 0x0061, 0x0079, 0x004E, 0x0061, 0x006D,
    0x0065, 0x0049, 0x0044, 0x0000,
];
pub static RUSTCONV_WSTR_EAFE2DCE91A2: [u16; 6] = [0x0025u16, 0x0049, 0x0036, 0x0034, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_EBFA2006F04A: [u16; 12] = [
    0x0044u16, 0x0065, 0x0074, 0x0061, 0x0069, 0x006C, 0x0073, 0x0050, 0x0061, 0x006E,
    0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_F159A9A27825: [u16; 10] = [0x0048u16, 0x0053, 0x0070, 0x006C, 0x0069, 0x0074, 0x0050, 0x006F, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_FAC6BE05036B: [u16; 4] = [0x0065u16, 0x0076, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_0C7319B93B0B: [u16; 12] = [
    0x0041u16, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_0DC053ABF082: [u16; 58] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0076, 0x0065, 0x006E, 0x0074,
    0x0020, 0x0056, 0x0069, 0x0065, 0x0077, 0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_376FD66F1A87: [u16; 44] = [
    0x0053u16, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F,
    0x006C, 0x0053, 0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069,
    0x0063, 0x0065, 0x0073, 0x005C, 0x0045, 0x0076, 0x0065, 0x006E, 0x0074, 0x004C,
    0x006F, 0x0067, 0x005C, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_6555D1ED30C5: [u16; 9] = [0x0045u16, 0x0056, 0x0045, 0x004E, 0x0054, 0x0056, 0x0057, 0x0052, 0x0000];
pub static RUSTCONV_C2R_WSTR_B4271F29F828: [u16; 9] = [0x0053u16, 0x0065, 0x0063, 0x0075, 0x0072, 0x0069, 0x0074, 0x0079, 0x0000];
pub static RUSTCONV_C2R_WSTR_E9A72DDF42EB: [u16; 7] = [0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0000];

pub const STATUSCLASSNAMEW: [::core::ffi::c_ushort; 19] = unsafe {
    ::core::mem::transmute::<[u8; 38], [::core::ffi::c_ushort; 19]>(
        *b"m\0s\0c\0t\0l\0s\0_\0s\0t\0a\0t\0u\0s\0b\0a\0r\x003\x002\0\0\0",
    )
};

pub const PROGRESS_CLASSW: [::core::ffi::c_ushort; 18] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_ushort; 18]>(
        *b"m\0s\0c\0t\0l\0s\0_\0p\0r\0o\0g\0r\0e\0s\0s\x003\x002\0\0\0",
    )
};

pub const WC_LISTVIEWW: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"S\0y\0s\0L\0i\0s\0t\0V\0i\0e\0w\x003\x002\0\0\0",
    )
};

pub const WC_TREEVIEWW: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"S\0y\0s\0T\0r\0e\0e\0V\0i\0e\0w\x003\x002\0\0\0",
    )
};

static mut EVENTVWR_WNDCLASS: LPCWSTR = unsafe {
    RUSTCONV_C2R_WSTR_6555D1ED30C5.as_ptr() as LPCWSTR
};

static mut EVENTLOG_BASE_KEY: LPCWSTR = unsafe {
    RUSTCONV_C2R_WSTR_376FD66F1A87.as_ptr() as LPCWSTR
};

static mut EVNTVWR_PARAM_KEY: LPCWSTR = unsafe {
    RUSTCONV_C2R_WSTR_0DC053ABF082.as_ptr() as LPCWSTR
};

static mut SystemLogs: [LPCWSTR; 3] = unsafe {
    [
        RUSTCONV_C2R_WSTR_0C7319B93B0B.as_ptr(),
        RUSTCONV_C2R_WSTR_B4271F29F828.as_ptr(),
        RUSTCONV_C2R_WSTR_E9A72DDF42EB.as_ptr(),
    ]
};

pub const EVENT_DLL_SEPARATOR: [::core::ffi::c_ushort; 2] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_ushort; 2]>(*b";\0\0\0") };

pub const EVENT_CATEGORY_MESSAGE_FILE: [::core::ffi::c_ushort; 20] = unsafe {
    ::core::mem::transmute::<[u8; 40], [::core::ffi::c_ushort; 20]>(
        *b"C\0a\0t\0e\0g\0o\0r\0y\0M\0e\0s\0s\0a\0g\0e\0F\0i\0l\0e\0\0\0",
    )
};

pub const EVENT_MESSAGE_FILE: [::core::ffi::c_ushort; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_ushort; 17]>(
        *b"E\0v\0e\0n\0t\0M\0e\0s\0s\0a\0g\0e\0F\0i\0l\0e\0\0\0",
    )
};

pub const EVENT_PARAMETER_MESSAGE_FILE: [::core::ffi::c_ushort; 21] = unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_ushort; 21]>(
        *b"P\0a\0r\0a\0m\0e\0t\0e\0r\0M\0e\0s\0s\0a\0g\0e\0F\0i\0l\0e\0\0\0",
    )
};

#[no_mangle]
pub static mut hInst: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut szTitle: [WCHAR; 255] = [0; 255];

#[no_mangle]
pub static mut szTitleTemplate: [WCHAR; 255] = [0; 255];

#[no_mangle]
pub static mut szStatusBarTemplate: [WCHAR; 255] = [0; 255];

#[no_mangle]
pub static mut szLoadingWait: [WCHAR; 255] = [0; 255];

#[no_mangle]
pub static mut szEmptyList: [WCHAR; 255] = [0; 255];

#[no_mangle]
pub static mut szSaveFilter: [WCHAR; 255] = [0; 255];

#[no_mangle]
pub static mut nVSplitPos: INT = 0;

#[no_mangle]
pub static mut nHSplitPos: INT = 0;

#[no_mangle]
pub static mut bSplit: BYTE = 0 as BYTE;

#[no_mangle]
pub static mut hwndMainWindow: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hwndTreeView: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hwndListView: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hwndEventDetails: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hwndStatus: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hwndStatusProgress: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hMainMenu: HMENU = ::core::ptr::null::<HMENU__>() as *mut HMENU__;

#[no_mangle]
pub static mut htiAppLogs: HTREEITEM = ::core::ptr::null::<_TREEITEM>() as *mut _TREEITEM;

#[no_mangle]
pub static mut htiUserLogs: HTREEITEM = ::core::ptr::null::<_TREEITEM>() as *mut _TREEITEM;

#[no_mangle]
pub static mut htiSystemLogs: HTREEITEM = ::core::ptr::null::<_TREEITEM>() as *mut _TREEITEM;

#[no_mangle]
pub static mut lpComputerName: LPWSTR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;

#[no_mangle]
pub static mut lpszzUserLogsToLoad: LPWSTR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;

#[no_mangle]
pub static mut cbUserLogsSize: SIZE_T = 0 as SIZE_T;

#[no_mangle]
pub static mut hkMachine: HKEY = ::core::ptr::null::<HKEY__>() as *mut HKEY__;

#[no_mangle]
pub static mut g_TotalRecords: DWORD = 0 as DWORD;

#[no_mangle]
pub static mut g_RecordPtrs: *mut PEVENTLOGRECORD =
    ::core::ptr::null::<PEVENTLOGRECORD>() as *mut PEVENTLOGRECORD;

#[no_mangle]
pub static mut EventLogList: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

#[no_mangle]
pub static mut EventLogFilterList: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

#[no_mangle]
pub static mut ActiveFilter: PEVENTLOGFILTER =
    ::core::ptr::null::<_EVENTLOGFILTER>() as *mut _EVENTLOGFILTER;

#[no_mangle]
pub static mut hEnumEventsThread: HANDLE = NULL;

#[no_mangle]
pub static mut hStopEnumEvent: HANDLE = NULL;

#[no_mangle]
pub static mut EnumFilter: PEVENTLOGFILTER =
    ::core::ptr::null::<_EVENTLOGFILTER>() as *mut _EVENTLOGFILTER;

#[no_mangle]
pub static mut hStartStopEnumEvent: HANDLE = NULL;

#[no_mangle]
pub static mut hStartEnumEvent: HANDLE = NULL;

#[no_mangle]
pub static mut sfn: OPENFILENAMEW = tagOFNW {
    lStructSize: 0,
    hwndOwner: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hInstance: ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__,
    lpstrFilter: ::core::ptr::null::<WCHAR>(),
    lpstrCustomFilter: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
    nMaxCustFilter: 0,
    nFilterIndex: 0,
    lpstrFile: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
    nMaxFile: 0,
    lpstrFileTitle: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
    nMaxFileTitle: 0,
    lpstrInitialDir: ::core::ptr::null::<WCHAR>(),
    lpstrTitle: ::core::ptr::null::<WCHAR>(),
    Flags: 0,
    nFileOffset: 0,
    nFileExtension: 0,
    lpstrDefExt: ::core::ptr::null::<WCHAR>(),
    lCustData: 0,
    lpfnHook: None,
    lpTemplateName: ::core::ptr::null::<WCHAR>(),
    pvReserved: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    dwReserved: 0,
    FlagsEx: 0,
};

#[no_mangle]
pub static mut Settings: SETTINGS = _SETTINGS {
    bShowDetailsPane: 0,
    bShowGrid: 0,
    bSaveSettings: 0,
    bNewestEventsFirst: 0,
    nVSplitPos: 0,
    nHSplitPos: 0,
    wpPos: _WINDOWPLACEMENT {
        length: 0,
        flags: 0,
        showCmd: 0,
        ptMinPosition: tagPOINT { x: 0, y: 0 },
        ptMaxPosition: tagPOINT { x: 0, y: 0 },
        rcNormalPosition: tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
    },
};

static mut hWndDetailsCtrl: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut hWndGrip: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

static mut cxMin: INT = 0;

static mut cyMin: INT = 0;

static mut cxOld: INT = 0;

static mut cyOld: INT = 0;

pub const LVM_PROGRESS: u32 = 0x8001; // WM_APP + 1

pub const EVENT_MESSAGE_EVENTTEXT_BUFFER: usize = 1024 * 10; // NOTE: Used by evtdetctl.c

pub const EVENT_MESSAGE_FILE_BUFFER: usize = 1024 * 10;

pub const MAX_LOADSTRING: usize = 255; // nolint: const-contradiction (eventvwr.c defines 255; regedit/magnify 100)

pub const SPLIT_WIDTH: i32 = 4;

pub type PSETTINGS = *mut SETTINGS;

#[repr(C)]
pub struct param_strings_format_data {
    pub pStartingAddress: LPWSTR, // Pointer to the beginning of a parameter string in pMessage
    pub pEndingAddress: LPWSTR,   // Pointer to the end of a parameter string in pMessage
    pub pParameterID: DWORD,      // Parameter identifier found in pMessage
    pub pParameter: LPWSTR,       // Actual parameter string
}
#[no_mangle]
pub unsafe extern "C" fn ShowWin32Error(mut dwError: DWORD) {
    let mut lpMessageBuffer: LPWSTR = core::ptr::null_mut();

    if dwError == ERROR_SUCCESS as DWORD {
        return;
    }

    if FormatMessageW(
        (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS) as DWORD,
        core::ptr::null(),
        dwError,
        LANG_USER_DEFAULT as DWORD,
        core::ptr::addr_of_mut!(lpMessageBuffer) as LPWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
    ) == 0
    {
        return;
    }

    MessageBoxW(
        hwndMainWindow,
        lpMessageBuffer as LPCWSTR,
        core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR,
        (MB_OK | MB_ICONERROR) as UINT,
    );
    LocalFree(lpMessageBuffer as HLOCAL);
}
#[no_mangle]
pub unsafe extern "C" fn DisplayUsage() {
    let mut lpBuffer: LPWSTR;
    let mut lpUsage: LPCWSTR = core::ptr::null();
    let iUsageLen: INT = LoadStringW(
        hInst,
        IDS_USAGE,
        core::ptr::addr_of_mut!(lpUsage) as LPWSTR,
        0,
    );

    if iUsageLen == 0 {
        return;
    }

    lpBuffer = HeapAlloc(
        GetProcessHeap(),
        0,
        ((iUsageLen + 1) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if lpBuffer.is_null() {
        return;
    }

    StringCchCopyNW(
        lpBuffer,
        (iUsageLen + 1) as usize,
        lpUsage,
        iUsageLen as usize,
    );
    MessageBoxW(
        core::ptr::null_mut(),
        lpBuffer as LPCWSTR,
        core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR,
        (MB_OK | MB_ICONINFORMATION) as UINT,
    );

    HeapFree(
        GetProcessHeap(),
        0,
        lpBuffer as PVOID,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ProcessCmdLine(mut lpCmdLine: LPWSTR) -> BOOL {
    let mut Success: BOOL = (FALSE as i32);
    let mut argc: INT = 0;
    let mut argv: *mut LPWSTR = core::ptr::null_mut();

    /* Skip any leading whitespace */
    if !lpCmdLine.is_null() {
        while iswspace(*lpCmdLine as wint_t) != 0 {
            lpCmdLine = lpCmdLine.offset(1);
        }
    }

    /* No command line means no processing needed */
    if lpCmdLine.is_null() || *lpCmdLine == 0 {
        return (TRUE) as BOOL;
    }

    /* Build the arguments vector */
    argv = CommandLineToArgvW(lpCmdLine, core::ptr::addr_of_mut!(argc));
    if argv.is_null() {
        return (FALSE) as BOOL;
    }

    'quit: {
        /* Parse the command line for options (skip the program name) */
        for i in 1..argc {
            /* Check for new options */
            if *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Du16
                || *(*argv.offset(i as isize)).offset(0 as isize) == 0x002Fu16
            {
                if *(*argv.offset(i as isize)).offset(1 as isize) == 0x003Fu16
                    && *(*argv.offset(i as isize)).offset(2 as isize) == 0
                {
                    /* Display help */
                    DisplayUsage();
                    break 'quit;
                } else if *(*argv.offset(i as isize)).offset(2 as isize) == 0x003Au16 {
                    match towupper(
                        (*(*argv.offset(i as isize)).offset(1 as isize) as wint_t) as _,
                    ) as u16
                    {
                        0x004Cu16 => {
                            let mut lpNewBuffer: LPWSTR = core::ptr::null_mut();
                            let mut lpFileName: LPWSTR =
                                (*argv.offset(i as isize)).offset(3 as isize);
                            let mut cbFileName: SIZE_T = 0;

                            /* Check for a quoted file name */
                            if *lpFileName == 0x0022u16 {
                                /* Skip this quote, and the last one too if any */
                                lpFileName = lpFileName.offset(1);
                                cbFileName = wcslen(lpFileName as *const wchar_t) as SIZE_T;
                                if cbFileName > 0 as SIZE_T
                                    && *lpFileName.offset(
                                        cbFileName.wrapping_sub(1 as SIZE_T) as isize,
                                    ) == 0x0022u16
                                {
                                    *lpFileName.offset(
                                        cbFileName.wrapping_sub(1 as SIZE_T) as isize,
                                    ) = UNICODE_NULL as WCHAR;
                                }
                            }

                            /* Skip this one if we do not actually have a file name */
                            if *lpFileName == 0 {
                                continue;
                            }

                            cbFileName = (wcslen(lpFileName as *const wchar_t) as usize)
                                .wrapping_add(1 as usize)
                                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)
                                as SIZE_T;

                            /* Reallocate the list of user logs to load */
                            if !lpszzUserLogsToLoad.is_null() {
                                lpNewBuffer = HeapReAlloc(
                                    GetProcessHeap(),
                                    HEAP_ZERO_MEMORY as DWORD,
                                    lpszzUserLogsToLoad as PVOID,
                                    /* Count the multi-string NULL-terminator */
                                    cbUserLogsSize
                                        .wrapping_add(cbFileName)
                                        .wrapping_add(core::mem::size_of::<WCHAR>() as SIZE_T),
                                ) as LPWSTR;
                            } else {
                                cbUserLogsSize = 0 as SIZE_T;
                                lpNewBuffer = HeapAlloc(
                                    GetProcessHeap(),
                                    HEAP_ZERO_MEMORY as DWORD,
                                    /* Count the multi-string NULL-terminator */
                                    cbUserLogsSize
                                        .wrapping_add(cbFileName)
                                        .wrapping_add(core::mem::size_of::<WCHAR>() as SIZE_T),
                                ) as LPWSTR;
                            }

                            if lpNewBuffer.is_null() {
                                ShowWin32Error(ERROR_NOT_ENOUGH_MEMORY as DWORD);
                                break 'quit;
                            }

                            lpszzUserLogsToLoad = lpNewBuffer;
                            lpNewBuffer = (lpNewBuffer as SIZE_T).wrapping_add(cbUserLogsSize) as LPWSTR;
                            cbUserLogsSize = cbUserLogsSize.wrapping_add(cbFileName);

                            /* Save the file name */
                            StringCbCopyW(lpNewBuffer, cbFileName as usize, lpFileName);

                            continue;
                        }

                        _ => {}
                    }
                }

                /* Unknown argument: display help and bail out */
                DisplayUsage();
                break 'quit;
            } else {
                /*
                 * An argument that does not start with the switch character.
                 * If this is the first argument then this corresponds to the
                 * optional computer name. Otherwise this is a wrong argument.
                 */
                if i == 1 as INT {
                    /* Store the computer name */
                    let mut lpTemp: LPWSTR = *argv.offset(i as isize);
                    let mut cbLength: SIZE_T = 0;

                    /* Strip any leading backslashes */
                    while *lpTemp == 0x005Cu16 {
                        lpTemp = lpTemp.offset(1);
                    }

                    cbLength = (wcslen(lpTemp as *const wchar_t) as usize)
                        .wrapping_add(1 as usize)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as usize)
                        as SIZE_T;
                    lpComputerName = HeapAlloc(
                        GetProcessHeap(),
                        HEAP_ZERO_MEMORY as DWORD,
                        cbLength,
                    ) as LPWSTR;
                    if !lpComputerName.is_null() {
                        StringCbCopyW(lpComputerName, cbLength as usize, lpTemp);
                    }
                    /* else, fall back to local computer */
                } else {
                    /* Invalid syntax: display help and bail out */
                    DisplayUsage();
                    break 'quit;
                }
            }
        }

        Success = (TRUE as i32);
    }

    /* In case of failure, free anything we have allocated */
    if Success == 0 {
        if !lpszzUserLogsToLoad.is_null() {
            cbUserLogsSize = 0 as SIZE_T;
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                lpszzUserLogsToLoad as PVOID,
            );
            lpszzUserLogsToLoad = core::ptr::null_mut();
        }
        if !lpComputerName.is_null() {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                lpComputerName as PVOID,
            );
            lpComputerName = core::ptr::null_mut();
        }
    }

    /* Free the arguments vector and exit */
    LocalFree(argv as HLOCAL);
    return Success;
}
#[no_mangle]
pub unsafe extern "C" fn LoadSettings(nDefCmdShow: ::core::ffi::c_int) -> BOOL {
    let mut Result: LONG;
    let mut hKeyEventVwr: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = core::mem::zeroed();
    let mut cbData: DWORD;
    let mut buffer: [WCHAR; 100] = [0; 100];

    /* Load the default values */
    Settings.bSaveSettings = (TRUE as i32);
    Settings.bShowDetailsPane = (TRUE as i32);
    Settings.bShowGrid = (FALSE as i32);
    Settings.bNewestEventsFirst = (TRUE as i32);
    Settings.nVSplitPos = 250; /* Splitter default positions */
    Settings.nHSplitPos = 250;
    ZeroMemory!(
        core::ptr::addr_of_mut!(Settings.wpPos) as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<WINDOWPLACEMENT>(),
    );
    Settings.wpPos.length = ::core::mem::size_of::<WINDOWPLACEMENT>() as UINT;
    SetRect(
        core::ptr::addr_of_mut!(Settings.wpPos.rcNormalPosition),
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
    );
    Settings.wpPos.showCmd = nDefCmdShow as UINT; // SW_SHOWNORMAL;

    /* Try to open the Event Viewer user key */
    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        EVNTVWR_PARAM_KEY,
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(hKeyEventVwr),
    ) != (ERROR_SUCCESS as i32)
    {
        return (FALSE) as BOOL;
    }

    // Result = RegQueryValueExW(hKeyEventVwr, L"Filter", NULL, &dwType, (LPBYTE)&szFilter, &cbData); // REG_SZ
    // Result = RegQueryValueExW(hKeyEventVwr, L"Find", NULL, &dwType, (LPBYTE)&szFind, &cbData); // REG_SZ
    // Result = RegQueryValueExW(hKeyEventVwr, L"Module", NULL, &dwType, (LPBYTE)&szModule, &cbData); // REG_SZ

    cbData = ::core::mem::size_of::<[WCHAR; 100]>() as DWORD;
    Result = RegQueryValueExW(
        hKeyEventVwr,
        RUSTCONV_WSTR_D1046FF7D861.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(cbData),
    );
    if Result == (ERROR_SUCCESS as i32) {
        if dwType == REG_SZ {
            buffer[((cbData / ::core::mem::size_of::<WCHAR>() as DWORD) - 1)
                as usize] = UNICODE_NULL;
            Settings.bSaveSettings =
                ((_wtoi(core::ptr::addr_of_mut!(buffer) as *mut WCHAR) as DWORD) != 0)
                    as ::core::ffi::c_int as BOOL;
        } else if dwType == REG_DWORD
            && cbData == ::core::mem::size_of::<DWORD>() as DWORD
        {
            Settings.bSaveSettings =
                (*(core::ptr::addr_of_mut!(buffer) as *mut WCHAR as PDWORD) != 0) as ::core::ffi::c_int
                    as BOOL;
        }
    }

    cbData = ::core::mem::size_of::<[WCHAR; 100]>() as DWORD;
    Result = RegQueryValueExW(
        hKeyEventVwr,
        RUSTCONV_WSTR_EBFA2006F04A.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(cbData),
    );
    if Result == (ERROR_SUCCESS as i32) {
        if dwType == REG_SZ {
            buffer[((cbData / ::core::mem::size_of::<WCHAR>() as DWORD) - 1)
                as usize] = UNICODE_NULL;
            Settings.bShowDetailsPane =
                ((_wtoi(core::ptr::addr_of_mut!(buffer) as *mut WCHAR) as DWORD) != 0)
                    as ::core::ffi::c_int as BOOL;
        } else if dwType == REG_DWORD
            && cbData == ::core::mem::size_of::<DWORD>() as DWORD
        {
            Settings.bShowDetailsPane =
                (*(core::ptr::addr_of_mut!(buffer) as *mut WCHAR as PDWORD) != 0) as ::core::ffi::c_int
                    as BOOL;
        }
    }

    cbData = ::core::mem::size_of::<[WCHAR; 100]>() as DWORD;
    Result = RegQueryValueExW(
        hKeyEventVwr,
        RUSTCONV_WSTR_7B514F84AE9A.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(cbData),
    );
    if Result == (ERROR_SUCCESS as i32) {
        if dwType == REG_SZ {
            buffer[((cbData / ::core::mem::size_of::<WCHAR>() as DWORD) - 1)
                as usize] = UNICODE_NULL;
            Settings.bShowGrid =
                ((_wtoi(core::ptr::addr_of_mut!(buffer) as *mut WCHAR) as DWORD) != 0)
                    as ::core::ffi::c_int as BOOL;
        } else if dwType == REG_DWORD
            && cbData == ::core::mem::size_of::<DWORD>() as DWORD
        {
            Settings.bShowGrid =
                (*(core::ptr::addr_of_mut!(buffer) as *mut WCHAR as PDWORD) != 0) as ::core::ffi::c_int
                    as BOOL;
        }
    }

    cbData = ::core::mem::size_of::<[WCHAR; 100]>() as DWORD;
    Result = RegQueryValueExW(
        hKeyEventVwr,
        RUSTCONV_WSTR_6EA98D9C3198.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(cbData),
    );
    if Result == (ERROR_SUCCESS as i32) {
        if dwType == REG_SZ {
            buffer[((cbData / ::core::mem::size_of::<WCHAR>() as DWORD) - 1)
                as usize] = UNICODE_NULL;
            Settings.bNewestEventsFirst =
                ((_wtoi(core::ptr::addr_of_mut!(buffer) as *mut WCHAR) as DWORD) != 0)
                    as ::core::ffi::c_int as BOOL;
        } else if dwType == REG_DWORD
            && cbData == ::core::mem::size_of::<DWORD>() as DWORD
        {
            Settings.bNewestEventsFirst =
                (*(core::ptr::addr_of_mut!(buffer) as *mut WCHAR as PDWORD) != 0) as ::core::ffi::c_int
                    as BOOL;
        }
    }

    /* Retrieve the splitter positions */
    cbData = ::core::mem::size_of::<[WCHAR; 100]>() as DWORD;
    Result = RegQueryValueExW(
        hKeyEventVwr,
        RUSTCONV_WSTR_8D3D94053677.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(cbData),
    );
    if Result == (ERROR_SUCCESS as i32) {
        if dwType == REG_SZ {
            buffer[((cbData / ::core::mem::size_of::<WCHAR>() as DWORD) - 1)
                as usize] = UNICODE_NULL;
            Settings.nVSplitPos =
                _wtoi(core::ptr::addr_of_mut!(buffer) as *mut WCHAR) as DWORD as INT;
        } else if dwType == REG_DWORD
            && cbData == ::core::mem::size_of::<DWORD>() as DWORD
        {
            Settings.nVSplitPos = *(core::ptr::addr_of_mut!(buffer) as *mut WCHAR as PDWORD) as INT;
        }
    }

    cbData = ::core::mem::size_of::<[WCHAR; 100]>() as DWORD;
    Result = RegQueryValueExW(
        hKeyEventVwr,
        RUSTCONV_WSTR_F159A9A27825.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(cbData),
    );
    if Result == (ERROR_SUCCESS as i32) {
        if dwType == REG_SZ {
            buffer[((cbData / ::core::mem::size_of::<WCHAR>() as DWORD) - 1)
                as usize] = UNICODE_NULL;
            Settings.nHSplitPos =
                _wtoi(core::ptr::addr_of_mut!(buffer) as *mut WCHAR) as DWORD as INT;
        } else if dwType == REG_DWORD
            && cbData == ::core::mem::size_of::<DWORD>() as DWORD
        {
            Settings.nHSplitPos = *(core::ptr::addr_of_mut!(buffer) as *mut WCHAR as PDWORD) as INT;
        }
    }

    /* Retrieve the geometry of the main window */
    cbData = ::core::mem::size_of::<[WCHAR; 100]>() as DWORD;
    Result = RegQueryValueExW(
        hKeyEventVwr,
        RUSTCONV_WSTR_4B164B324B2E.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(cbData),
    );
    if (Result == (ERROR_SUCCESS as i32)) && (dwType == REG_SZ) {
        buffer[((cbData / ::core::mem::size_of::<WCHAR>() as DWORD) - 1)
            as usize] = UNICODE_NULL;
    } else {
        buffer[0 as usize] = UNICODE_NULL;
    }

    if swscanf(
        core::ptr::addr_of_mut!(buffer) as *mut WCHAR,
        RUSTCONV_WSTR_2E5F286A7F7B.as_ptr(),
        core::ptr::addr_of_mut!(Settings.wpPos.rcNormalPosition.left),
        core::ptr::addr_of_mut!(Settings.wpPos.rcNormalPosition.top),
        core::ptr::addr_of_mut!(Settings.wpPos.rcNormalPosition.right),
        core::ptr::addr_of_mut!(Settings.wpPos.rcNormalPosition.bottom),
        core::ptr::addr_of_mut!(Settings.wpPos.showCmd),
    ) != 5
    {
        /* Parsing failed, use defaults */
        SetRect(
            core::ptr::addr_of_mut!(Settings.wpPos.rcNormalPosition),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
        );
        Settings.wpPos.showCmd = nDefCmdShow as UINT; // SW_SHOWNORMAL;
    }

    RegCloseKey(hKeyEventVwr);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SaveSettings() -> BOOL {
    let mut hKeyEventVwr: HKEY = core::ptr::null_mut();
    let mut dwSize: DWORD = 0;
    let mut buffer: [WCHAR; 100] = [0; 100];

    /* Try to create/open the Event Viewer user key */
    if RegCreateKeyExW(
        HKEY_CURRENT_USER,
        EVNTVWR_PARAM_KEY,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_SET_VALUE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hKeyEventVwr),
        core::ptr::null_mut(),
    ) != (ERROR_SUCCESS as i32)
    {
        return (FALSE) as BOOL;
    }

    'quit: {
        dwSize = core::mem::size_of::<BOOL>() as DWORD;
        RegSetValueExW(
            hKeyEventVwr,
            RUSTCONV_WSTR_D1046FF7D861.as_ptr(),
            0,
            REG_DWORD,
            core::ptr::addr_of_mut!(Settings.bSaveSettings) as LPBYTE,
            dwSize,
        );

        /* Do not save more settings if we are not asked to do so */
        if Settings.bSaveSettings == 0 {
            break 'quit;
        }

        dwSize = core::mem::size_of::<BOOL>() as DWORD;
        RegSetValueExW(
            hKeyEventVwr,
            RUSTCONV_WSTR_EBFA2006F04A.as_ptr(),
            0,
            REG_DWORD,
            core::ptr::addr_of_mut!(Settings.bShowDetailsPane) as LPBYTE,
            dwSize,
        );

        dwSize = core::mem::size_of::<BOOL>() as DWORD;
        RegSetValueExW(
            hKeyEventVwr,
            RUSTCONV_WSTR_7B514F84AE9A.as_ptr(),
            0,
            REG_DWORD,
            core::ptr::addr_of_mut!(Settings.bShowGrid) as LPBYTE,
            dwSize,
        );

        dwSize = core::mem::size_of::<BOOL>() as DWORD;
        RegSetValueExW(
            hKeyEventVwr,
            RUSTCONV_WSTR_6EA98D9C3198.as_ptr(),
            0,
            REG_DWORD,
            core::ptr::addr_of_mut!(Settings.bNewestEventsFirst) as LPBYTE,
            dwSize,
        );

        Settings.nVSplitPos = nVSplitPos;
        dwSize = core::mem::size_of::<INT>() as DWORD;
        RegSetValueExW(
            hKeyEventVwr,
            RUSTCONV_WSTR_8D3D94053677.as_ptr(),
            0,
            REG_DWORD,
            core::ptr::addr_of_mut!(Settings.nVSplitPos) as LPBYTE,
            dwSize,
        );

        Settings.nHSplitPos = nHSplitPos;
        dwSize = core::mem::size_of::<INT>() as DWORD;
        RegSetValueExW(
            hKeyEventVwr,
            RUSTCONV_WSTR_F159A9A27825.as_ptr(),
            0,
            REG_DWORD,
            core::ptr::addr_of_mut!(Settings.nHSplitPos) as LPBYTE,
            dwSize,
        );

        StringCbPrintfW!(
            buffer.as_mut_ptr(),
            core::mem::size_of::<[WCHAR; 100]>(),
            RUSTCONV_WSTR_2E5F286A7F7B.as_ptr(),
            Settings.wpPos.rcNormalPosition.left,
            Settings.wpPos.rcNormalPosition.top,
            Settings.wpPos.rcNormalPosition.right,
            Settings.wpPos.rcNormalPosition.bottom,
            Settings.wpPos.showCmd,
        );

        dwSize = ((wcslen(buffer.as_mut_ptr()) + 1)
            * core::mem::size_of::<WCHAR>()) as DWORD;
        RegSetValueExW(
            hKeyEventVwr,
            RUSTCONV_WSTR_4B164B324B2E.as_ptr(),
            0,
            REG_SZ,
            buffer.as_mut_ptr() as LPBYTE,
            dwSize,
        );
    }

    /* Quit: */
    RegCloseKey(hKeyEventVwr);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPWSTR,
    nCmdShow: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut hThread: HANDLE = core::ptr::null_mut();
    let mut iccx: INITCOMMONCONTROLSEX = core::mem::zeroed();
    let mut hRichEdit: HMODULE = core::ptr::null_mut();
    let mut hAccelTable: HACCEL = core::ptr::null_mut();
    let mut msg: MSG = core::mem::zeroed();

    UNREFERENCED_PARAMETER!(hPrevInstance);
    UNREFERENCED_PARAMETER!(lpCmdLine);

    /* Whenever any of the common controls are used in your app,
     * you must call InitCommonControlsEx() to register the classes
     * for those controls. */
    iccx.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
    iccx.dwICC = ICC_LISTVIEW_CLASSES;
    InitCommonControlsEx(core::ptr::addr_of_mut!(iccx));

    /* Load the RichEdit DLL to add support for RichEdit controls */
    hRichEdit = LoadLibraryW(RUSTCONV_WSTR_A5D19B228349.as_ptr());
    if hRichEdit.is_null() {
        return -1;
    }

    msg.wParam = (-1 as ::core::ffi::c_int) as WPARAM;

    /* Store the instance handle in the global variable */
    hInst = hInstance;

    /* Initialize global strings */
    LoadStringW(hInstance, IDS_APP_TITLE, core::ptr::addr_of_mut!(szTitle) as LPWSTR, (ARRAYSIZE!(szTitle)) as i32);
    LoadStringW(hInstance, IDS_APP_TITLE_EX, core::ptr::addr_of_mut!(szTitleTemplate) as LPWSTR, (ARRAYSIZE!(szTitleTemplate)) as i32);
    LoadStringW(hInstance, IDS_STATUS_MSG, core::ptr::addr_of_mut!(szStatusBarTemplate) as LPWSTR, (ARRAYSIZE!(szStatusBarTemplate)) as i32);
    LoadStringW(hInstance, IDS_LOADING_WAIT, core::ptr::addr_of_mut!(szLoadingWait) as LPWSTR, (ARRAYSIZE!(szLoadingWait)) as i32);
    LoadStringW(hInstance, IDS_NO_ITEMS, core::ptr::addr_of_mut!(szEmptyList) as LPWSTR, (ARRAYSIZE!(szEmptyList)) as i32);

    /*
     * Process the command-line arguments. Note that we need the full
     * command-line, with the program file included, and not just what
     * WinMain() provides in its lpCmdLine parameter.
     */
    'quit: {
        'cleanup: {
            if ProcessCmdLine(GetCommandLineW()) == 0 {
                break 'quit;
            }

            if MyRegisterClass(hInstance) == 0 {
                break 'quit;
            }

            /* Load the settings */
            LoadSettings(nCmdShow);

            /* Perform application initialization */
            if InitInstance(hInstance) == 0 {
                break 'quit;
            }

            hAccelTable = LoadAcceleratorsW(hInstance, MAKEINTRESOURCEW!(IDA_EVENTVWR));

            /* Create the Start/Stop enumerator thread */
            // Manual-reset event
            hStartStopEnumEvent = CreateEventW(core::ptr::null_mut(), (TRUE) as BOOL, (FALSE) as BOOL, core::ptr::null());
            if hStartStopEnumEvent.is_null() {
                break 'cleanup;
            }

            // Auto-reset event
            hStartEnumEvent = CreateEventW(core::ptr::null_mut(), (FALSE) as BOOL, (FALSE) as BOOL, core::ptr::null());
            if hStartEnumEvent.is_null() {
                break 'cleanup;
            }

            hThread = CreateThread(
                core::ptr::null_mut(),
                0,
                Some(StartStopEnumEventsThread as unsafe extern "system" fn(LPVOID) -> DWORD),
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );
            if hThread.is_null() {
                break 'cleanup;
            }

            /* Retrieve the available event logs on this computer and create filters for them */
            InitializeListHead(core::ptr::addr_of_mut!(EventLogList));
            InitializeListHead(core::ptr::addr_of_mut!(EventLogFilterList));
            BuildLogListAndFilterList(lpComputerName as LPCWSTR);

            /* Open the user-specified logs if any are present on the command-line */
            if !lpszzUserLogsToLoad.is_null() {
                let mut lpUserLog: LPWSTR = core::ptr::null_mut();
                lpUserLog = lpszzUserLogsToLoad;
                while *lpUserLog != 0 {
                    OpenUserEventLogFile(lpUserLog as LPCWSTR);
                    lpUserLog = lpUserLog.offset(
                        (wcslen(lpUserLog as *const u16) + 1 as usize) as isize,
                    );
                }

                /* Now cleanup the list of user logs */
                cbUserLogsSize = 0;
                HeapFree(GetProcessHeap(), 0, lpszzUserLogsToLoad as PVOID);
                lpszzUserLogsToLoad = core::ptr::null_mut();
            }

            /* Main message loop */
            while GetMessageW(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0) != 0 {
                if TranslateAcceleratorW(hwndMainWindow, hAccelTable, core::ptr::addr_of_mut!(msg)) == 0 {
                    TranslateMessage(core::ptr::addr_of_mut!(msg));
                    DispatchMessageW(core::ptr::addr_of_mut!(msg));
                }
            }

            /* Save the settings */
            SaveSettings();

            /* Disconnect from computer */
            if !hkMachine.is_null() && hkMachine != HKEY_LOCAL_MACHINE {
                /* We are connected to some other computer, close the old connection */
                RegCloseKey(hkMachine);
                hkMachine = core::ptr::null_mut();
            }

            /* Stop the enumerator thread */
            SetEvent(hStartStopEnumEvent);
            WaitForSingleObject(hThread, INFINITE);
            CloseHandle(hThread);

            /* Free the filters list and the event logs list */
            FreeLogFilterList();
            FreeLogList();
        }

        /* Handle cleanup */
        // Cleanup:
        if !hStartEnumEvent.is_null() {
            CloseHandle(hStartEnumEvent);
        }
        if !hStartStopEnumEvent.is_null() {
            CloseHandle(hStartStopEnumEvent);
        }
    }

    /* Final cleanup */
    // Quit:
    if !lpszzUserLogsToLoad.is_null() {
        cbUserLogsSize = 0;
        HeapFree(GetProcessHeap(), 0, lpszzUserLogsToLoad as PVOID);
        lpszzUserLogsToLoad = core::ptr::null_mut();
    }
    if !lpComputerName.is_null() {
        HeapFree(GetProcessHeap(), 0, lpComputerName as PVOID);
        lpComputerName = core::ptr::null_mut();
    }
    FreeLibrary(hRichEdit);

    return msg.wParam as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn EventTimeToSystemTime(mut EventTime: DWORD, mut pSystemTime: PSYSTEMTIME) {
    let mut st1970: SYSTEMTIME = SYSTEMTIME {
        wYear: 1970 as WORD,
        wMonth: 1 as WORD,
        wDayOfWeek: 0 as WORD,
        wDay: 1 as WORD,
        wHour: 0 as WORD,
        wMinute: 0 as WORD,
        wSecond: 0 as WORD,
        wMilliseconds: 0 as WORD,
    };
    let mut ftLocal: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    // The C declares an anonymous local `union { FILETIME ft; ULONGLONG ll; } u1970, uUCT;`
    // (c2rust named it C2RustUnnamed_12); no island carries a name for a function-local type.
    #[repr(C)]
    #[derive(Copy, Clone)]
    union FileTimeU64 {
        ft: FILETIME,
        ll: ULONGLONG,
    }
    let mut u1970: FileTimeU64 = FileTimeU64 {
        ft: FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        },
    };
    let mut uUCT: FileTimeU64 = FileTimeU64 {
        ft: FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        },
    };
    uUCT.ft.dwHighDateTime = 0 as DWORD;
    uUCT.ft.dwLowDateTime = EventTime;
    SystemTimeToFileTime(
        core::ptr::addr_of_mut!(st1970),
        core::ptr::addr_of_mut!(u1970.ft),
    );
    uUCT.ll = uUCT
        .ll
        .wrapping_mul(10000000 as ULONGLONG)
        .wrapping_add(u1970.ll);
    FileTimeToLocalFileTime(
        core::ptr::addr_of_mut!(uUCT.ft),
        core::ptr::addr_of_mut!(ftLocal),
    );
    FileTimeToSystemTime(
        core::ptr::addr_of_mut!(ftLocal),
        pSystemTime,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GetMessageStringFromDll(
    lpMessageDll: LPCWSTR,
    mut dwFlags: DWORD,
    dwMessageId: DWORD,
    nSize: DWORD,
    Arguments: *mut va_list,
) -> LPWSTR {
    let mut hLibrary: HMODULE = core::ptr::null_mut();
    let mut dwLength: DWORD = 0;
    let mut lpMsgBuf: LPWSTR = core::ptr::null_mut();
    let mut _SEH2_Volatile0_local: i32 = 0;

    hLibrary = LoadLibraryExW(
        lpMessageDll,
        core::ptr::null_mut(),
        (LOAD_LIBRARY_AS_DATAFILE) as DWORD,
    ) as HMODULE;
    if hLibrary.is_null() {
        return core::ptr::null_mut();
    }

    /* Sanitize dwFlags */
    dwFlags &= (!(FORMAT_MESSAGE_FROM_STRING)) as DWORD;
    dwFlags |= (FORMAT_MESSAGE_FROM_HMODULE) as DWORD;

    // _SEH2_TRY { .. } _SEH2_FINALLY { FreeLibrary } _SEH2_END: the try block has no early
    // exit (no return/break/_SEH2_LEAVE), so running it to completion and then the finally
    // block is the whole of the C's control flow on every path.
    /* _SEH2_TRY */
    {
        /*
         * Retrieve the message string without appending extra newlines.
         * Wrap in SEH to protect from invalid string parameters.
         */
        /* _SEH2_TRY */
        {
            dwLength = FormatMessageW(
                dwFlags,
                hLibrary as LPCVOID,
                dwMessageId,
                (LANG_USER_DEFAULT) as DWORD,
                core::ptr::addr_of_mut!(lpMsgBuf) as LPWSTR,
                nSize,
                (Arguments).cast(),
            );
        }
        // _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER): lowered as the tree's dummy PSEH does
        // (`if (_SEH2_Volatile0)`): Rust cannot catch a fault raised inside FormatMessageW,
        // so the handler is guarded by a flag that is never set and a fault propagates.
        if core::ptr::read_volatile(core::ptr::addr_of!(_SEH2_Volatile0_local)) != 0 {
            /*
             * An exception occurred while calling FormatMessage, this is usually
             * the sign that a parameter was invalid, either 'lpMsgBuf' was NULL
             * but we did not pass the flag FORMAT_MESSAGE_ALLOCATE_BUFFER, or the
             * array pointer 'Arguments' was NULL or did not contain enough elements,
             * and we did not pass the flag FORMAT_MESSAGE_IGNORE_INSERTS, and the
             * message string expected too many inserts.
             * In this last case only, we can call again FormatMessage but ignore
             * explicitly the inserts. The string that we will return to the user
             * will not be pre-formatted.
             */
            dwLength = (0) as DWORD;

            if (((dwFlags & ((FORMAT_MESSAGE_ALLOCATE_BUFFER) as DWORD)) != 0) || !lpMsgBuf.is_null())
                && ((dwFlags & ((FORMAT_MESSAGE_IGNORE_INSERTS) as DWORD)) == 0)
            {
                /* Remove any possible harmful flags and always ignore inserts */
                dwFlags &= (!(FORMAT_MESSAGE_ARGUMENT_ARRAY)) as DWORD;
                dwFlags |= (FORMAT_MESSAGE_IGNORE_INSERTS) as DWORD;

                /* If this call also throws an exception, we are really dead */
                dwLength = FormatMessageW(
                    dwFlags,
                    hLibrary as LPCVOID,
                    dwMessageId,
                    (LANG_USER_DEFAULT) as DWORD,
                    core::ptr::addr_of_mut!(lpMsgBuf) as LPWSTR,
                    nSize,
                    core::ptr::null_mut(),
                );
            }
        }
        /* _SEH2_END */
    }
    /* _SEH2_FINALLY */
    {
        FreeLibrary(hLibrary);
    }
    /* _SEH2_END */

    if dwLength == (0) as DWORD {
        ASSERT!(lpMsgBuf.is_null());
        lpMsgBuf = core::ptr::null_mut();
    } else {
        let mut ptr: LPWSTR = core::ptr::null_mut();

        ASSERT!(!lpMsgBuf.is_null());

        /* Trim any trailing whitespace */
        // The C has no lower bound: an all-whitespace message walks off the front of the
        // buffer. Stop at its first character.
        ptr = lpMsgBuf
            .offset(dwLength as isize - ((1) as isize));
        while iswspace((*ptr as u32) as u16) != 0 {
            *ptr = (UNICODE_NULL) as WCHAR;
            if ptr == lpMsgBuf {
                break;
            }
            ptr = ptr.offset(-1);
        }
    }

    return lpMsgBuf;
}
#[no_mangle]
pub unsafe extern "C" fn GetMessageStringFromDllList(
    lpMessageDllList: LPCWSTR,
    dwFlags: DWORD,
    dwMessageId: DWORD,
    nSize: DWORD,
    Arguments: *mut va_list,
) -> LPWSTR {
    let mut Success: BOOL = (FALSE as i32);
    let mut cbLength: SIZE_T = 0;
    let mut szMessageDllList: LPWSTR = core::ptr::null_mut();
    let mut szDll: LPWSTR = core::ptr::null_mut();
    let mut lpMsgBuf: LPWSTR = core::ptr::null_mut();

    /* Allocate a local buffer for the DLL list that can be tokenized */
    cbLength = ((wcslen(lpMessageDllList) as SIZE_T)
        .wrapping_add(1 as SIZE_T))
        .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T);
    szMessageDllList = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        cbLength,
    ) as LPWSTR;
    if szMessageDllList.is_null() {
        return core::ptr::null_mut();
    }
    CopyMemory!(
        szMessageDllList as *mut core::ffi::c_void,
        lpMessageDllList as *const core::ffi::c_void,
        cbLength as usize,
    );

    /* Loop through the list of message DLLs */
    szDll = wcstok(szMessageDllList, (EVENT_DLL_SEPARATOR).as_ptr()) as LPWSTR;
    while !szDll.is_null() && Success == 0 {
        lpMsgBuf = GetMessageStringFromDll(
            szDll as LPCWSTR,
            dwFlags,
            dwMessageId,
            nSize,
            Arguments,
        );
        if !lpMsgBuf.is_null() {
            Success = (TRUE as i32);
            break;
        }

        /*
         * The DLL could not be loaded, or the message could not be found,
         * try the next DLL, if any.
         */
        szDll = wcstok(core::ptr::null_mut(), (EVENT_DLL_SEPARATOR).as_ptr()) as LPWSTR;
    }

    HeapFree(
        GetProcessHeap(),
        0,
        szMessageDllList as PVOID,
    );

    return lpMsgBuf;
}
#[no_mangle]
pub unsafe extern "C" fn ApplyParameterStringsToMessage(
    lpMessageDllList: LPCWSTR,
    bMessagePreFormatted: BOOL,
    pMessage: LPCWSTR,
    pFinalMessage: *mut LPWSTR,
) -> DWORD {
    let mut Status: DWORD = ERROR_SUCCESS as DWORD;
    let mut dwParamCount: DWORD = 0 as DWORD;
    let mut cchBuffer: size_t = 0 as size_t;
    let mut cchParams: size_t = 0 as size_t;
    let mut cch: size_t = 0 as size_t;
    let mut i: DWORD = 0 as DWORD;
    let mut pParamData: *mut param_strings_format_data = core::ptr::null_mut();
    let mut pTempMessage: LPWSTR = pMessage as LPWSTR;
    let mut pTempFinalMessage: LPWSTR = core::ptr::null_mut();

    *pFinalMessage = core::ptr::null_mut();

    /* Determine the number of parameter insertion strings in pMessage */
    if bMessagePreFormatted != 0 {
        loop {
            pTempMessage = wcschr(
                pTempMessage as *const u16,
                0x0025u16,
            ) as LPWSTR;
            if pTempMessage.is_null() {
                break;
            }
            pTempMessage = pTempMessage.offset(1);
            if iswdigit(*pTempMessage) != 0 {
                dwParamCount = dwParamCount.wrapping_add(1);
                loop {
                    pTempMessage = pTempMessage.offset(1);
                    if !(iswdigit(*pTempMessage) != 0) {
                        break;
                    }
                }
            }
        }
    } else {
        loop {
            pTempMessage = wcsstr(
                pTempMessage as *const u16,
                RUSTCONV_WSTR_B6D17ED8374A.as_ptr(),
            ) as LPWSTR;
            if pTempMessage.is_null() {
                break;
            }
            pTempMessage = pTempMessage.offset(2 as isize);
            if iswdigit(*pTempMessage) != 0 {
                dwParamCount = dwParamCount.wrapping_add(1);
                loop {
                    pTempMessage = pTempMessage.offset(1);
                    if !(iswdigit(*pTempMessage) != 0) {
                        break;
                    }
                }
            }
        }
    }

    'quit: {
        /* If there are no parameter insertion strings in pMessage, just return */
        if dwParamCount == 0 as DWORD {
            // *pFinalMessage = NULL;
            break 'quit;
        }

        /* Allocate the array of parameter string format data */
        pParamData = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (dwParamCount as SIZE_T).wrapping_mul(
                core::mem::size_of::<param_strings_format_data>() as SIZE_T,
            ),
        ) as *mut param_strings_format_data;
        if pParamData.is_null() {
            Status = ERROR_OUTOFMEMORY as DWORD;
            break 'quit;
        }

        /*
         * Retrieve each parameter in pMessage and the beginning and end of the
         * insertion string, as well as the message identifier of the parameter.
         */
        pTempMessage = pMessage as LPWSTR;
        if bMessagePreFormatted != 0 {
            while {
                pTempMessage = wcschr(
                    pTempMessage as *const u16,
                    0x0025u16,
                ) as LPWSTR;
                !pTempMessage.is_null() && i < dwParamCount
            } {
                pTempMessage = pTempMessage.offset(1);
                if iswdigit(*pTempMessage) != 0 {
                    (*pParamData.offset(i as isize)).pStartingAddress =
                        pTempMessage.offset(-(1 as isize));
                    (*pParamData.offset(i as isize)).pParameterID =
                        _wtol(pTempMessage as *const u16) as DWORD;

                    loop {
                        pTempMessage = pTempMessage.offset(1);
                        if !(iswdigit(*pTempMessage) != 0) {
                            break;
                        }
                    }

                    (*pParamData.offset(i as isize)).pEndingAddress = pTempMessage;
                    i = i.wrapping_add(1);
                }
            }
        } else {
            while {
                pTempMessage = wcsstr(
                    pTempMessage as *const u16,
                    RUSTCONV_WSTR_B6D17ED8374A.as_ptr(),
                ) as LPWSTR;
                !pTempMessage.is_null() && i < dwParamCount
            } {
                pTempMessage = pTempMessage.offset(2 as isize);
                if iswdigit(*pTempMessage) != 0 {
                    (*pParamData.offset(i as isize)).pStartingAddress =
                        pTempMessage.offset(-(2 as isize));
                    (*pParamData.offset(i as isize)).pParameterID =
                        _wtol(pTempMessage as *const u16) as DWORD;

                    loop {
                        pTempMessage = pTempMessage.offset(1);
                        if !(iswdigit(*pTempMessage) != 0) {
                            break;
                        }
                    }

                    (*pParamData.offset(i as isize)).pEndingAddress = pTempMessage;
                    i = i.wrapping_add(1);
                }
            }
        }

        /* Retrieve each parameter string */
        i = 0 as DWORD;
        while i < dwParamCount {
            'body: {
                // pParamData[i].pParameter = GetMessageString(pParamData[i].pParameterID, 0, NULL);
                (*pParamData.offset(i as isize)).pParameter = GetMessageStringFromDllList(
                    lpMessageDllList,
                    (FORMAT_MESSAGE_ALLOCATE_BUFFER
                        | FORMAT_MESSAGE_FROM_HMODULE
                        | FORMAT_MESSAGE_IGNORE_INSERTS
                        | FORMAT_MESSAGE_MAX_WIDTH_MASK) as DWORD,
                    (*pParamData.offset(i as isize)).pParameterID,
                    0 as DWORD,
                    core::ptr::null_mut(),
                );
                if (*pParamData.offset(i as isize)).pParameter.is_null() {
                    /* Skip the insertion string */
                    break 'body;
                }

                cchParams = cchParams.wrapping_add(
                    wcslen((*pParamData.offset(i as isize)).pParameter as *const u16)
                        as size_t,
                );
            }
            i = i.wrapping_add(1);
        }

        /*
         * Allocate the final message buffer, the size of which is based on the
         * length of the original message and the length of each parameter string.
         */
        cchBuffer = (wcslen(pMessage as *const u16) as size_t)
            .wrapping_add(cchParams)
            .wrapping_add(1 as size_t);
        *pFinalMessage = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (cchBuffer as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if (*pFinalMessage).is_null() {
            Status = ERROR_OUTOFMEMORY as DWORD;
            break 'quit;
        }

        pTempFinalMessage = *pFinalMessage;

        /* Build the final message string */
        pTempMessage = pMessage as LPWSTR;
        i = 0 as DWORD;
        while i < dwParamCount {
            /* Append the segment from pMessage */
            cch = (*pParamData.offset(i as isize))
                .pStartingAddress
                .offset_from(pTempMessage) as size_t;
            StringCchCopyNW(pTempFinalMessage, cchBuffer, pTempMessage, cch);
            pTempMessage = (*pParamData.offset(i as isize)).pEndingAddress;
            cchBuffer = cchBuffer.wrapping_sub(cch);
            pTempFinalMessage = pTempFinalMessage.offset(cch as isize);

            /* Append the parameter string */
            if !(*pParamData.offset(i as isize)).pParameter.is_null() {
                StringCchCopyW(
                    pTempFinalMessage,
                    cchBuffer,
                    (*pParamData.offset(i as isize)).pParameter,
                );
                cch = wcslen(
                    (*pParamData.offset(i as isize)).pParameter as *const u16,
                ) as size_t; // pTempFinalMessage
            } else {
                /*
                 * We failed to retrieve the parameter string before, so just
                 * place back the original string placeholder.
                 */
                cch = (*pParamData.offset(i as isize))
                    .pEndingAddress /* == pTempMessage */
                    .offset_from((*pParamData.offset(i as isize)).pStartingAddress)
                    as size_t;
                StringCchCopyNW(
                    pTempFinalMessage,
                    cchBuffer,
                    (*pParamData.offset(i as isize)).pStartingAddress,
                    cch,
                );
                // cch = wcslen(pTempFinalMessage);
            }
            cchBuffer = cchBuffer.wrapping_sub(cch);
            pTempFinalMessage = pTempFinalMessage.offset(cch as isize);
            i = i.wrapping_add(1);
        }

        /* Append the last segment from pMessage */
        StringCchCopyW(pTempFinalMessage, cchBuffer, pTempMessage);
    }

    // Cleanup:

    // if (Status != ERROR_SUCCESS)
    //     *pFinalMessage = NULL;

    if !pParamData.is_null() {
        i = 0 as DWORD;
        while i < dwParamCount {
            if !(*pParamData.offset(i as isize)).pParameter.is_null() {
                LocalFree(((*pParamData.offset(i as isize)).pParameter).cast());
            }
            i = i.wrapping_add(1);
        }

        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            pParamData as PVOID,
        );
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "C" fn FormatInteger(Num: LONGLONG, pwszResult: LPWSTR, cchResultMax: UINT) -> UINT {
    let mut wszNumber: [WCHAR; 24] = [0; 24];
    let mut wszDecimalSep: [WCHAR; 8] = [0; 8];
    let mut wszThousandSep: [WCHAR; 8] = [0; 8];
    let mut nf: NUMBERFMTW = core::mem::zeroed();
    let mut wszGrouping: [WCHAR; 12] = [0; 12];
    let mut cchGrouping: INT;
    let mut cchResult: INT;
    let mut i: INT;

    // Print the number in uniform mode
    _swprintf(
        core::ptr::addr_of_mut!(wszNumber) as *mut WCHAR,
        RUSTCONV_WSTR_EAFE2DCE91A2.as_ptr(),
        Num,
    );

    // Get system strings for decimal and thousand separators.
    GetLocaleInfoW(
        LOCALE_USER_DEFAULT,
        LOCALE_SDECIMAL,
        core::ptr::addr_of_mut!(wszDecimalSep) as LPWSTR,
        _countof!(wszDecimalSep) as INT,
    );
    GetLocaleInfoW(
        LOCALE_USER_DEFAULT,
        LOCALE_STHOUSAND,
        core::ptr::addr_of_mut!(wszThousandSep) as LPWSTR,
        _countof!(wszThousandSep) as INT,
    );

    // Initialize format for printing the number in bytes
    ZeroMemory!(
        core::ptr::addr_of_mut!(nf) as *mut core::ffi::c_void,
        core::mem::size_of::<NUMBERFMTW>(),
    );
    nf.lpDecimalSep = core::ptr::addr_of_mut!(wszDecimalSep) as *mut WCHAR as LPWSTR;
    nf.lpThousandSep = core::ptr::addr_of_mut!(wszThousandSep) as *mut WCHAR as LPWSTR;

    // Get system string for groups separator
    cchGrouping = GetLocaleInfoW(
        LOCALE_USER_DEFAULT,
        LOCALE_SGROUPING,
        core::ptr::addr_of_mut!(wszGrouping) as LPWSTR,
        _countof!(wszGrouping) as INT,
    );

    // Convert grouping specs from string to integer
    i = 0 as INT;
    while i < cchGrouping {
        let wch: WCHAR = wszGrouping[i as usize];

        if wch >= 0x0030u16 && wch <= 0x0039u16 {
            nf.Grouping = nf
                .Grouping
                .wrapping_mul(10 as UINT)
                .wrapping_add((wch as UINT).wrapping_sub(0x0030u16 as UINT));
        } else if wch != 0x003Bu16 {
            break;
        }
        i += 1;
    }

    if nf.Grouping.wrapping_rem(10 as UINT) == 0 as UINT {
        nf.Grouping = nf.Grouping.wrapping_div(10 as UINT);
    } else {
        nf.Grouping = nf.Grouping.wrapping_mul(10 as UINT);
    }

    // Format the number
    cchResult = GetNumberFormatW(
        LOCALE_USER_DEFAULT,
        0,
        core::ptr::addr_of_mut!(wszNumber) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(nf),
        pwszResult,
        cchResultMax as INT,
    );

    if cchResult == 0 {
        return 0 as UINT;
    }

    // GetNumberFormatW returns number of characters including UNICODE_NULL
    return (cchResult - 1 as INT) as UINT;
}
#[no_mangle]
pub unsafe extern "C" fn FormatByteSize(cbSize: LONGLONG, pwszResult: LPWSTR, cchResultMax: UINT) -> UINT {
    let mut cchWritten: UINT = 0;
    let mut cchRemaining: UINT = 0;
    let mut pwszEnd: LPWSTR = core::ptr::null_mut();
    let mut cchStringRemaining: size_t = 0;

    /* Write formated bytes count */
    cchWritten = FormatInteger(cbSize, pwszResult, cchResultMax);
    if cchWritten == 0 {
        return 0;
    }

    /* Copy " bytes" to buffer */
    pwszEnd = pwszResult.add(cchWritten as usize);
    cchRemaining = cchResultMax.wrapping_sub(cchWritten);
    StringCchCopyExW(
        pwszEnd,
        cchRemaining as size_t,
        RUSTCONV_WSTR_8D71B3FAAB82.as_ptr(),
        core::ptr::addr_of_mut!(pwszEnd),
        core::ptr::addr_of_mut!(cchStringRemaining),
        0,
    );
    cchRemaining = cchStringRemaining as UINT;
    cchWritten = LoadStringW(
        hInst,
        IDS_BYTES_FORMAT,
        pwszEnd,
        cchRemaining as core::ffi::c_int,
    ) as UINT;
    cchRemaining = cchRemaining.wrapping_sub(cchWritten);

    return cchResultMax.wrapping_sub(cchRemaining);
}
#[no_mangle]
pub unsafe extern "C" fn FormatFileSizeWithBytes(lpQwSize: PULARGE_INTEGER, pwszResult: LPWSTR, cchResultMax: UINT) -> LPWSTR {
    let mut cchWritten: UINT;
    let mut cchRemaining: UINT;
    let mut pwszEnd: LPWSTR;
    let mut cchCopyRemaining: size_t = core::mem::zeroed();

    /* Format bytes in KBs, MBs etc */
    if StrFormatByteSizeW(((*lpQwSize).QuadPart) as LONGLONG, pwszResult, cchResultMax).is_null() {
        return core::ptr::null_mut();
    }

    /* If there is less bytes than 1KB, we have nothing to do */
    if (*lpQwSize).QuadPart < 1024 {
        return pwszResult;
    }

    /* Concatenate " (" */
    cchWritten = wcslen(pwszResult as *const u16) as UINT;
    pwszEnd = pwszResult.offset(cchWritten as isize);
    cchRemaining = cchResultMax.wrapping_sub(cchWritten);
    StringCchCopyExW(
        pwszEnd as STRSAFE_LPWSTR,
        cchRemaining as size_t,
        RUSTCONV_WSTR_ACBE5EA9B8F8.as_ptr(),
        core::ptr::addr_of_mut!(pwszEnd),
        core::ptr::addr_of_mut!(cchCopyRemaining),
        0,
    );
    cchRemaining = cchCopyRemaining as UINT;

    /* Write formated bytes count */
    cchWritten = FormatByteSize((*lpQwSize).QuadPart as LONGLONG, pwszEnd, cchRemaining);
    pwszEnd = pwszEnd.offset(cchWritten as isize);
    cchRemaining = cchRemaining.wrapping_sub(cchWritten);

    /* Copy ")" to the buffer */
    StringCchCopyW(pwszEnd as STRSAFE_LPWSTR, cchRemaining as size_t, RUSTCONV_WSTR_4F2D892D6AF4.as_ptr());

    return pwszResult;
}
#[no_mangle]
pub unsafe extern "C" fn GetFileTimeString(mut lpFileTime: LPFILETIME, mut pwszResult: LPWSTR, mut cchResult: UINT) -> BOOL {
    let mut ft: FILETIME = core::mem::zeroed();
    let mut st: SYSTEMTIME = core::mem::zeroed();
    let mut cchWritten: core::ffi::c_int = 0;
    let mut cchRemaining: UINT = cchResult;
    let mut cchCopyRemaining: size_t = 0;
    let mut pwszEnd: LPWSTR = pwszResult;

    if FileTimeToLocalFileTime(lpFileTime as *const FILETIME, core::ptr::addr_of_mut!(ft)) == 0
        || FileTimeToSystemTime(core::ptr::addr_of_mut!(ft) as *const FILETIME, core::ptr::addr_of_mut!(st)) == 0
    {
        return (FALSE) as BOOL;
    }

    cchWritten = GetDateFormatW(
        LOCALE_USER_DEFAULT,
        DATE_LONGDATE as DWORD,
        core::ptr::addr_of_mut!(st),
        core::ptr::null_mut(),
        pwszEnd,
        cchRemaining as core::ffi::c_int,
    );
    if cchWritten != 0 {
        cchWritten -= 1; // GetDateFormatW returns count with terminating zero
    }
    // else
        // ERR("GetDateFormatW failed\n");

    cchRemaining = cchRemaining.wrapping_sub(cchWritten as UINT);
    pwszEnd = pwszEnd.offset(cchWritten as isize);

    StringCchCopyExW(
        pwszEnd,
        cchRemaining as size_t,
        RUSTCONV_WSTR_82294FD79A0D.as_ptr(),
        core::ptr::addr_of_mut!(pwszEnd),
        core::ptr::addr_of_mut!(cchCopyRemaining),
        0,
    );
    cchRemaining = cchCopyRemaining as UINT;

    cchWritten = GetTimeFormatW(
        LOCALE_USER_DEFAULT,
        0 as DWORD,
        core::ptr::addr_of_mut!(st),
        core::ptr::null_mut(),
        pwszEnd,
        cchRemaining as core::ffi::c_int,
    );
    if cchWritten != 0 {
        cchWritten -= 1; // GetTimeFormatW returns count with terminating zero
    }
    // else
        // ERR("GetTimeFormatW failed\n");

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn TreeViewAddItem(
    hTreeView: HWND,
    hParent: HTREEITEM,
    lpText: LPWSTR,
    Image: INT,
    SelectedImage: INT,
    lParam: LPARAM,
) -> HTREEITEM {
    let mut Insert: TV_INSERTSTRUCTW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(Insert) as *mut core::ffi::c_void,
        core::mem::size_of::<TV_INSERTSTRUCTW>(),
    );

    Insert.__bindgen_anon_1.item.mask = TVIF_TEXT | TVIF_PARAM | TVIF_IMAGE | TVIF_SELECTEDIMAGE;
    Insert.hInsertAfter = TVI_LAST;
    Insert.hParent = hParent;
    Insert.__bindgen_anon_1.item.pszText = lpText;
    Insert.__bindgen_anon_1.item.iImage = Image;
    Insert.__bindgen_anon_1.item.iSelectedImage = SelectedImage;
    Insert.__bindgen_anon_1.item.lParam = lParam;

    Insert.__bindgen_anon_1.item.mask |= TVIF_STATE;
    Insert.__bindgen_anon_1.item.stateMask = TVIS_OVERLAYMASK;
    Insert.__bindgen_anon_1.item.state = INDEXTOOVERLAYMASK!(1);

    return TreeView_InsertItem!(hTreeView, core::ptr::addr_of_mut!(Insert));
}
#[no_mangle]
pub unsafe extern "C" fn AllocEventLog(
    ComputerName: PCWSTR,
    LogName: PCWSTR,
    Permanent: BOOL,
) -> PEVENTLOG {
    let mut EventLog: PEVENTLOG;
    let mut cchName: SIZE_T;

    /* Allocate a new event log entry */
    EventLog = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as DWORD, core::mem::size_of::<EVENTLOG>() as SIZE_T) as PEVENTLOG;
    if EventLog.is_null() {
        return core::ptr::null_mut();
    }

    /* Allocate the computer name string (optional) and copy it */
    if !ComputerName.is_null() {
        cchName = wcslen(ComputerName).wrapping_add((1 as SIZE_T) as _) as SIZE_T;
        (*EventLog).ComputerName = HeapAlloc(GetProcessHeap(), 0 as DWORD, cchName.wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as PWSTR;
        if !(*EventLog).ComputerName.is_null() {
            StringCchCopyW((*EventLog).ComputerName, cchName as size_t, ComputerName);
        }
    }

    /* Allocate the event log name string and copy it */
    cchName = wcslen(LogName).wrapping_add((1 as SIZE_T) as _) as SIZE_T;
    (*EventLog).LogName = HeapAlloc(GetProcessHeap(), 0 as DWORD, cchName.wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as PWSTR;
    if (*EventLog).LogName.is_null() {
        if !(*EventLog).ComputerName.is_null() {
            HeapFree(GetProcessHeap(), 0 as DWORD, (*EventLog).ComputerName as PVOID);
        }
        HeapFree(GetProcessHeap(), 0 as DWORD, EventLog as PVOID);
        return core::ptr::null_mut();
    }
    StringCchCopyW((*EventLog).LogName, cchName as size_t, LogName);

    (*EventLog).Permanent = Permanent;

    return EventLog;
}
#[no_mangle]
pub unsafe extern "C" fn EventLog_Free(EventLog: PEVENTLOG) {
    if !(*EventLog).LogName.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, (*EventLog).LogName as PVOID);
    }

    if !(*EventLog).ComputerName.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, (*EventLog).ComputerName as PVOID);
    }

    if !(*EventLog).FileName.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, (*EventLog).FileName as PVOID);
    }

    HeapFree(GetProcessHeap(), 0 as DWORD, EventLog as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn AllocAndCopyMultiStr(MultiStr: PCWSTR) -> PWSTR {
    let mut pStr: PWSTR;
    let mut Length: ULONG;

    if MultiStr.is_null() {
        return core::ptr::null_mut();
    }

    pStr = MultiStr as PWSTR;
    while *pStr != 0 {
        let step = wcslen(pStr as *const WCHAR) + 1;
        pStr = pStr.offset(step as isize);
    }
    Length = (MultiStr.offset_from(pStr) as isize + 2) as ULONG;

    pStr = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (Length as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWSTR;
    // NOTE: If we failed allocating the string, then fall back into no filter!
    if !pStr.is_null() {
        CopyMemory!(
            pStr as *mut core::ffi::c_void,
            MultiStr as *const core::ffi::c_void,
            (Length as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
    }

    pStr
}
#[no_mangle]
pub unsafe extern "C" fn AllocEventLogFilter(
    Information: BOOL,
    Warning: BOOL,
    Error: BOOL,
    AuditSuccess: BOOL,
    AuditFailure: BOOL,
    Sources: PCWSTR,
    Users: PCWSTR,
    ComputerNames: PCWSTR,
    NumOfEventLogs: ULONG,
    EventLogs: *mut PEVENTLOG,
) -> PEVENTLOGFILTER {
    let mut EventLogFilter: PEVENTLOGFILTER = core::ptr::null_mut();

    /* Allocate a new event log filter entry, big enough to accommodate the list of logs */
    EventLogFilter = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::offset_of!(EVENTLOGFILTER, EventLogs)
            + (NumOfEventLogs as usize) * core::mem::size_of::<PEVENTLOG>()) as SIZE_T,
    ) as PEVENTLOGFILTER;
    if EventLogFilter.is_null() {
        return core::ptr::null_mut();
    }

    (*EventLogFilter).Information = Information;
    (*EventLogFilter).Warning = Warning;
    (*EventLogFilter).Error = Error;
    (*EventLogFilter).AuditSuccess = AuditSuccess;
    (*EventLogFilter).AuditFailure = AuditFailure;

    /* Allocate and copy the sources, users, and computers multi-strings */
    (*EventLogFilter).Sources = AllocAndCopyMultiStr(Sources);
    (*EventLogFilter).Users = AllocAndCopyMultiStr(Users);
    (*EventLogFilter).ComputerNames = AllocAndCopyMultiStr(ComputerNames);

    /* Copy the list of event logs */
    (*EventLogFilter).NumOfEventLogs = NumOfEventLogs;
    core::ptr::copy_nonoverlapping(
        EventLogs,
        core::ptr::addr_of_mut!((*EventLogFilter).EventLogs) as *mut PEVENTLOG,
        NumOfEventLogs as usize,
    );

    /* Initialize the filter reference count */
    (*EventLogFilter).ReferenceCount = 1;

    return EventLogFilter;
}
#[no_mangle]
pub unsafe extern "C" fn EventLogFilter_Free(mut EventLogFilter: PEVENTLOGFILTER) {
    if !(*EventLogFilter).Sources.is_null() {
        HeapFree(GetProcessHeap(), 0, (*EventLogFilter).Sources as PVOID);
    }

    if !(*EventLogFilter).Users.is_null() {
        HeapFree(GetProcessHeap(), 0, (*EventLogFilter).Users as PVOID);
    }

    if !(*EventLogFilter).ComputerNames.is_null() {
        HeapFree(GetProcessHeap(), 0, (*EventLogFilter).ComputerNames as PVOID);
    }

    HeapFree(GetProcessHeap(), 0, EventLogFilter as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn EventLogFilter_AddRef(EventLogFilter: PEVENTLOGFILTER) -> LONG {
    ASSERT!(!EventLogFilter.is_null());
    return InterlockedIncrement(core::ptr::addr_of_mut!((*EventLogFilter).ReferenceCount));
}
#[no_mangle]
pub unsafe extern "C" fn EventLogFilter_Release(EventLogFilter: PEVENTLOGFILTER) -> LONG {
    let mut RefCount: LONG;

    ASSERT!(!EventLogFilter.is_null());

    /* When the reference count reaches zero, delete the filter */
    RefCount = InterlockedDecrement(core::ptr::addr_of_mut!((*EventLogFilter).ReferenceCount));
    if RefCount <= 0 {
        /* Remove the filter from the list */
        /* RemoveEntryList(&EventLogFilter->ListEntry); */
        EventLogFilter_Free(EventLogFilter);
    }

    return RefCount;
}
#[no_mangle]
pub unsafe extern "C" fn TrimNulls(s: LPWSTR) {
    let mut c: PWCHAR = core::ptr::null_mut();

    if !s.is_null() {
        // The C steps `c` to s-1 (and starts there for an empty string) before testing
        // `c >= s`; count down by index so the cursor never leaves the string.
        let mut n = wcslen(s);
        while n > 0 && iswspace(*s.add(n - 1)) != 0 {
            n -= 1;
        }
        c = s.add(n);
        *c = 0x0000u16;
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetExpandedFilePathName(
    mut ComputerName: PCWSTR,
    mut FileName: PCWSTR,
    mut pFullFileName: PWSTR,
    mut nSize: SIZE_T,
) -> SIZE_T {
    let mut dwLength: SIZE_T;

    if nSize == 0 {
        return 0;
    }

    /* Determine the needed size after expansion of any environment strings */
    dwLength = (ExpandEnvironmentStringsW(FileName, core::ptr::null_mut(), 0)) as _;
    if dwLength == 0 {
        /* We failed, bail out */
        return 0;
    }

    /* If the file path is on a remote computer, estimate its length */
    // FIXME: Use WNetGetUniversalName instead?
    if !ComputerName.is_null() && *ComputerName != 0 {
        /* Skip any leading backslashes */
        while *ComputerName == 0x005Cu16 {
            ComputerName = ComputerName.add(1);
        }

        if *ComputerName != 0 {
            /* Count 2 backslashes plus the computer name and one backslash separator */
            dwLength += (2 + wcslen(ComputerName) + 1) as SIZE_T;
        }
    }

    /* Check whether we have enough space */
    if dwLength > nSize {
        /* No, return the needed size in characters (includes NULL-terminator) */
        *pFullFileName = UNICODE_NULL;
        return dwLength;
    }

    /* Now expand the file path */

    /* Expand any existing environment strings */
    if ExpandEnvironmentStringsW(FileName, pFullFileName, (dwLength) as DWORD) == 0 {
        /* We failed, bail out */
        return 0;
    }

    /* If the file path is on a remote computer, retrieve the network share form of the file name */
    // FIXME: Use WNetGetUniversalName instead?
    if !ComputerName.is_null() && *ComputerName != 0 {
        /* Note that we previously skipped any potential leading backslashes */

        /* Replace ':' by '$' in the drive letter */
        if *pFullFileName != 0 && *pFullFileName.add(1) == 0x003Au16 {
            *pFullFileName.add(1) = 0x0024u16;
        }

        /* Prepend the computer name */
        MoveMemory!(
            pFullFileName.add(
                (2 + wcslen(ComputerName) + 1) as usize,
            ),
            pFullFileName,
            dwLength * (core::mem::size_of::<WCHAR>()) as SIZE_T
                - ((2 + wcslen(ComputerName) + 1)
                    * core::mem::size_of::<WCHAR>()) as SIZE_T,
        );
        *pFullFileName.add(0) = 0x005Cu16;
        *pFullFileName.add(1) = 0x005Cu16;
        wcsncpy(
            pFullFileName.add(2),
            ComputerName,
            wcslen(ComputerName),
        );
        *pFullFileName.add((2 + wcslen(ComputerName)) as usize) = 0x005Cu16;
    }

    /* Return the number of stored characters (includes NULL-terminator) */
    return dwLength;
}
#[no_mangle]
pub unsafe extern "C" fn GetEventMessageFileDLL(
    LogName: PCWSTR,
    SourceName: PCWSTR,
    EntryName: PCWSTR,
    pModuleName: PWSTR,
    cchName: SIZE_T,
) -> BOOL {
    let mut Success: BOOL = FALSE as BOOL;
    let mut Result: LONG;
    let mut dwType: DWORD = core::mem::zeroed();
    let mut dwSize: DWORD;
    let mut szModuleName: [WCHAR; MAX_PATH as usize] = [0 as WCHAR; MAX_PATH as usize];
    let mut KeyPath: PWSTR;
    let mut cbKeyPath: SIZE_T;
    let mut hLogKey: HKEY = core::ptr::null_mut();
    let mut hSourceKey: HKEY = core::ptr::null_mut();

    if cchName == 0 as SIZE_T {
        return FALSE as BOOL;
    }

    cbKeyPath = (wcslen(EVENTLOG_BASE_KEY as PCWSTR) as usize)
        .wrapping_add(wcslen(LogName as PCWSTR) as usize)
        .wrapping_add(1 as usize)
        .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as SIZE_T;
    KeyPath = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        cbKeyPath,
    ) as PWSTR;
    if KeyPath.is_null() {
        ShowWin32Error(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return FALSE as BOOL;
    }

    StringCbCopyW(KeyPath, (cbKeyPath) as _, EVENTLOG_BASE_KEY as PCWSTR);
    StringCbCatW(KeyPath, (cbKeyPath) as _, LogName as PCWSTR);

    Result = RegOpenKeyExW(
        hkMachine,
        KeyPath as PCWSTR,
        0 as DWORD,
        KEY_READ as REGSAM,
        &raw mut hLogKey,
    ) as LONG;
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        KeyPath as PVOID,
    );
    if Result != ERROR_SUCCESS as LONG {
        return FALSE as BOOL;
    }

    Result = RegOpenKeyExW(
        hLogKey,
        SourceName as PCWSTR,
        0 as DWORD,
        KEY_QUERY_VALUE as REGSAM,
        &raw mut hSourceKey,
    ) as LONG;
    RegCloseKey(hLogKey);
    if Result != ERROR_SUCCESS as LONG {
        return FALSE as BOOL;
    }

    dwSize = core::mem::size_of::<[WCHAR; MAX_PATH as usize]>() as DWORD;
    Result = RegQueryValueExW(
        hSourceKey,
        EntryName as PCWSTR,
        core::ptr::null_mut(),
        &raw mut dwType,
        &raw mut szModuleName as *mut WCHAR as LPBYTE,
        &raw mut dwSize,
    ) as LONG;
    if (Result != ERROR_SUCCESS as LONG)
        || (dwType != REG_EXPAND_SZ as DWORD && dwType != REG_SZ as DWORD)
    {
        *(&raw mut szModuleName as *mut WCHAR) = UNICODE_NULL as WCHAR;
    } else {
        /* NULL-terminate the string and expand it */
        szModuleName[(dwSize as usize)
            .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
            .wrapping_sub(1 as usize) as usize] = UNICODE_NULL as WCHAR;
        Success = (GetExpandedFilePathName(
            lpComputerName as PCWSTR,
            &raw mut szModuleName as *mut WCHAR as PCWSTR,
            pModuleName,
            cchName,
        ) != 0 as SIZE_T
            && *pModuleName as core::ffi::c_int != 0) as core::ffi::c_int as BOOL;
    }

    RegCloseKey(hSourceKey);

    return Success;
}
#[no_mangle]
pub unsafe extern "C" fn GetEventCategory(
    KeyName: PCWSTR,
    SourceName: PCWSTR,
    pevlr: PEVENTLOGRECORD,
    CategoryName: PWSTR,
    cchName: SIZE_T,
) -> BOOL {
    let mut Success: BOOL = (FALSE as i32);
    let mut szMessageDLL: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpMsgBuf: LPWSTR = core::ptr::null_mut();

    if cchName == 0 {
        return (FALSE) as BOOL;
    }

    'body: {
        if GetEventMessageFileDLL(
            KeyName,
            SourceName,
            (EVENT_CATEGORY_MESSAGE_FILE).as_ptr(),
            core::ptr::addr_of_mut!(szMessageDLL) as PWSTR,
            _countof!(szMessageDLL) as SIZE_T,
        ) == 0
        {
            break 'body;
        }

        /* Retrieve the message string without appending extra newlines */
        lpMsgBuf = GetMessageStringFromDllList(
            core::ptr::addr_of_mut!(szMessageDLL) as *mut WCHAR as LPCWSTR,
            (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_HMODULE
                | FORMAT_MESSAGE_IGNORE_INSERTS | FORMAT_MESSAGE_MAX_WIDTH_MASK) as DWORD,
            (*pevlr).EventCategory as DWORD,
            EVENT_MESSAGE_FILE_BUFFER as DWORD,
            core::ptr::null_mut(),
        );
        if !lpMsgBuf.is_null() {
            /* Trim the string */
            TrimNulls(lpMsgBuf);

            /* Copy the category name */
            StringCchCopyW(CategoryName, (cchName) as _, lpMsgBuf);

            /* Free the buffer allocated by FormatMessage */
            LocalFree((lpMsgBuf).cast());

            /* The ID was found and the message was formatted */
            Success = (TRUE as i32);
        }
    }

    // Quit:
    if Success == 0 {
        if (*pevlr).EventCategory != 0 {
            StringCchPrintfW!(
                CategoryName,
                cchName,
                RUSTCONV_WSTR_C543E6446BAB.as_ptr(),
                (*pevlr).EventCategory as core::ffi::c_uint,
            );
            Success = (TRUE as i32);
        }
    }

    return Success;
}
#[no_mangle]
pub unsafe extern "C" fn GetEventMessage(
    KeyName: PCWSTR,
    SourceName: PCWSTR,
    pevlr: PEVENTLOGRECORD,
    EventText: PWSTR,
    cchText: SIZE_T,
) -> BOOL {
    let mut Success: BOOL = (FALSE as i32);
    let mut i: DWORD = 0;
    let mut cch: size_t = 0;
    let mut SourceModuleName: [WCHAR; 1024] = [0; 1024];
    let mut ParameterModuleName: [WCHAR; 1024] = [0; 1024];
    let mut IsParamModNameCached: BOOL = (FALSE as i32);
    let mut lpMsgBuf: LPWSTR = core::ptr::null_mut();
    let mut szStringArray: LPWSTR = core::ptr::null_mut();
    let mut szMessage: LPWSTR = core::ptr::null_mut();
    let mut szArguments: *mut LPWSTR = core::ptr::null_mut();

    if cchText == 0 {
        return (FALSE) as BOOL;
    }

    /* Get the event string array */
    szStringArray = (pevlr as LPBYTE).offset((*pevlr).StringOffset as isize) as LPWSTR;

    'quit: {
        /* NOTE: GetEventMessageFileDLL can return a comma-separated list of DLLs */
        if GetEventMessageFileDLL(
            KeyName,
            SourceName,
            (EVENT_MESSAGE_FILE).as_ptr(),
            &raw mut SourceModuleName as PWSTR,
            (_countof!(SourceModuleName)) as SIZE_T,
        ) == 0
        {
            break 'quit;
        }

        /* Allocate space for insertion strings */
        szArguments = HeapAlloc(
            GetProcessHeap(),
            0,
            ((*pevlr).NumStrings as SIZE_T).wrapping_mul(core::mem::size_of::<LPVOID>() as SIZE_T),
        ) as *mut LPWSTR;
        if szArguments.is_null() {
            break 'quit;
        }

        // TODO: Revisit this whole IsParamModNameCached later,
        // see commits c1ecc98f60 (r71368) and d5ba2a3784 (r71958).
        ParameterModuleName[0] = UNICODE_NULL; // TEMP fix in case GetEventMessageFileDLL fails.
        if IsParamModNameCached == 0 {
            /* Now that the parameter file list is loaded, no need to reload it at the next run! */
            IsParamModNameCached = GetEventMessageFileDLL(
                KeyName,
                SourceName,
                (EVENT_PARAMETER_MESSAGE_FILE).as_ptr(),
                &raw mut ParameterModuleName as PWSTR,
                (_countof!(ParameterModuleName)) as SIZE_T,
            );
            // FIXME: If the string loading failed the first time, no need to retry it just after???
        }

        if IsParamModNameCached != 0 {
            /* Not yet support for reading messages from parameter message DLL */
        }

        szMessage = szStringArray;
        /*
         * HACK:
         * We do some hackish preformatting of the cached event strings...
         * That's because below, we pass the string to FormatMessage
         * (via GetMessageStringFromDllList) with the FORMAT_MESSAGE_ARGUMENT_ARRAY
         * flag, instead of ignoring the insertion parameters and do the formatting
         * by ourselves. Therefore, the resulting string should have the parameter
         * string placeholders starting with a single '%' instead of a mix of one
         * and two '%'.
         */
        /* HACK part 1: Compute the full length of the string array */
        cch = 0;
        i = 0;
        while i < (*pevlr).NumStrings as DWORD {
            szMessage = szMessage.offset(
                (wcslen(szMessage as PCWSTR) + 1) as isize,
            );
            i = i.wrapping_add(1);
        }
        cch = szMessage.offset_from(szStringArray) as size_t;

        /* HACK part 2: Now do the HACK proper! */
        szMessage = szStringArray;
        i = 0;
        while i < (*pevlr).NumStrings as DWORD {
            lpMsgBuf = szMessage;
            loop {
                lpMsgBuf = wcsstr(lpMsgBuf as PCWSTR, RUSTCONV_WSTR_B6D17ED8374A.as_ptr()) as LPWSTR;
                if lpMsgBuf.is_null() {
                    break;
                }
                if iswdigit((*lpMsgBuf.offset(2 as isize) as u32) as u16) != 0 {
                    MoveMemory!(
                        lpMsgBuf as *mut core::ffi::c_void,
                        lpMsgBuf.offset(1 as isize) as *const core::ffi::c_void,
                        ((szStringArray.offset(cch as isize).offset_from(lpMsgBuf) as isize
                            - 1 as isize) as SIZE_T)
                            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                    );
                } else {
                    // Divergence from the C, which never advances here: a "%%" NOT followed by
                    // a digit is found again by the next wcsstr at the same address, so the C
                    // spins forever on any insertion string containing e.g. "%%s" or a
                    // trailing "%%". Step past the first '%' (one char, so "%%%1" still
                    // reaches its "%%1") and keep scanning.
                    lpMsgBuf = lpMsgBuf.offset(1);
                }
            }

            *szArguments.offset(i as isize) = szMessage;
            szMessage = szMessage.offset(
                (wcslen(szMessage as PCWSTR) + 1) as isize,
            );
            i = i.wrapping_add(1);
        }

        /* Retrieve the message string without appending extra newlines */
        lpMsgBuf = GetMessageStringFromDllList(
            &raw mut SourceModuleName as *mut WCHAR as LPCWSTR,
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_HMODULE | FORMAT_MESSAGE_ARGUMENT_ARRAY | FORMAT_MESSAGE_MAX_WIDTH_MASK,
            (*pevlr).EventID,
            0,
            szArguments as *mut va_list,
        );
        if !lpMsgBuf.is_null() {
            /* Trim the string */
            TrimNulls(lpMsgBuf);

            szMessage = core::ptr::null_mut();
            Success = (ApplyParameterStringsToMessage(
                &raw mut ParameterModuleName as *mut WCHAR as LPCWSTR,
                (TRUE) as BOOL,
                lpMsgBuf as LPCWSTR,
                &raw mut szMessage,
            ) == ERROR_SUCCESS) as BOOL;
            if Success != 0 && !szMessage.is_null() {
                /* Free the buffer allocated by FormatMessage */
                LocalFree(lpMsgBuf as HLOCAL);
                lpMsgBuf = szMessage;
            }

            /* Copy the event text */
            StringCchCopyW(EventText as STRSAFE_LPWSTR, cchText as size_t, lpMsgBuf as STRSAFE_LPCWSTR);

            /* Free the buffer allocated by FormatMessage */
            LocalFree(lpMsgBuf as HLOCAL);
        }

        HeapFree(GetProcessHeap(), 0, szArguments as PVOID);
    }

    /* Quit: */
    if Success == 0 {
        /* Get a read-only pointer to the "Event-Not-Found" string */
        let mut cchResLen: SIZE_T = LoadStringW(
            hInst,
            IDS_EVENTSTRINGIDNOTFOUND,
            &raw mut szMessage as PWSTR,
            0,
        ) as SIZE_T;

        lpMsgBuf = HeapAlloc(
            GetProcessHeap(),
            0,
            (cchResLen + 1) * (core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if !lpMsgBuf.is_null() {
            StringCchCopyNW(
                lpMsgBuf as STRSAFE_LPWSTR,
                (cchResLen + 1) as _,
                szMessage as STRSAFE_LPCWSTR,
                (cchResLen) as _,
            );
            szMessage = lpMsgBuf;
        } else {
            /* Use a hardcoded format string */
            szMessage = RUSTCONV_WSTR_9711BD294B9C.as_ptr() as LPWSTR;
        }
        StringCchPrintfW!(
            EventText as STRSAFE_LPWSTR,
            cchText as size_t,
            szMessage as STRSAFE_LPCWSTR,
            (*pevlr).EventID & 0xFFFF,
            SourceName,
        );
        if !lpMsgBuf.is_null() {
            HeapFree(GetProcessHeap(), 0, lpMsgBuf as PVOID);
        }

        /* Append the strings */
        szMessage = szStringArray;
        i = 0;
        while i < (*pevlr).NumStrings as DWORD {
            StringCchCatW(EventText as STRSAFE_LPWSTR, cchText as size_t, szMessage as STRSAFE_LPCWSTR);
            StringCchCatW(EventText as STRSAFE_LPWSTR, cchText as size_t, RUSTCONV_WSTR_075DE2B906DB.as_ptr());
            szMessage = szMessage.offset(
                (wcslen(szMessage as PCWSTR) + 1) as isize,
            );
            i = i.wrapping_add(1);
        }
    }

    return Success;
}
#[no_mangle]
pub unsafe extern "C" fn GetEventType(
    mut dwEventType: WORD,
    mut pszEventType: PWSTR,
    mut cchText: SIZE_T,
) {
    match (dwEventType as ::core::ffi::c_int) as u32 {
        EVENTLOG_ERROR_TYPE => {
            LoadStringW(
                hInst,
                IDS_EVENTLOG_ERROR_TYPE as UINT,
                pszEventType as LPWSTR,
                cchText as ::core::ffi::c_int,
            );
        }
        EVENTLOG_WARNING_TYPE => {
            LoadStringW(
                hInst,
                IDS_EVENTLOG_WARNING_TYPE as UINT,
                pszEventType as LPWSTR,
                cchText as ::core::ffi::c_int,
            );
        }
        EVENTLOG_INFORMATION_TYPE => {
            LoadStringW(
                hInst,
                IDS_EVENTLOG_INFORMATION_TYPE as UINT,
                pszEventType as LPWSTR,
                cchText as ::core::ffi::c_int,
            );
        }
        EVENTLOG_SUCCESS => {
            LoadStringW(
                hInst,
                IDS_EVENTLOG_SUCCESS as UINT,
                pszEventType as LPWSTR,
                cchText as ::core::ffi::c_int,
            );
        }
        EVENTLOG_AUDIT_SUCCESS => {
            LoadStringW(
                hInst,
                IDS_EVENTLOG_AUDIT_SUCCESS as UINT,
                pszEventType as LPWSTR,
                cchText as ::core::ffi::c_int,
            );
        }
        EVENTLOG_AUDIT_FAILURE => {
            LoadStringW(
                hInst,
                IDS_EVENTLOG_AUDIT_FAILURE as UINT,
                pszEventType as LPWSTR,
                cchText as ::core::ffi::c_int,
            );
        }
        _ => {
            LoadStringW(
                hInst,
                IDS_EVENTLOG_UNKNOWN_TYPE as UINT,
                pszEventType as LPWSTR,
                cchText as ::core::ffi::c_int,
            );
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn GetEventUserName(
    mut pelr: PEVENTLOGRECORD,
    mut pLastSid: *mut PSID,
    mut pszUser: PWSTR,
    mut cchUser: SIZE_T,
) -> BOOL {
    let mut pCurrentSid: PSID = core::ptr::null_mut();
    let mut StringSid: PWSTR = core::ptr::null_mut();
    let mut szName: [WCHAR; 1024] = [0; 1024];
    let mut szDomain: [WCHAR; 1024] = [0; 1024];
    let mut peUse: SID_NAME_USE = 0 as SID_NAME_USE;
    let mut cchName: DWORD = _countof!(szName) as DWORD;
    let mut cchDomain: DWORD = _countof!(szDomain) as DWORD;
    let mut Success: BOOL = (FALSE as i32);

    if cchUser == 0 as SIZE_T {
        return (FALSE) as BOOL;
    }

    /* Point to the SID */
    pCurrentSid = (pelr as LPBYTE).offset((*pelr).UserSidOffset as isize) as PSID;

    if IsValidSid(pCurrentSid) == 0 {
        *pLastSid = core::ptr::null_mut();
        return (FALSE) as BOOL;
    } else if !(*pLastSid).is_null() && EqualSid(*pLastSid, pCurrentSid) != 0 {
        return (TRUE) as BOOL;
    }

    /* User SID */
    if (*pelr).UserSidLength > 0 as DWORD {
        /*
         * Try to retrieve the user account name and domain name corresponding
         * to the SID. If it cannot be retrieved, try to convert the SID to a
         * string-form. It should not be bigger than the user-provided buffer
         * 'pszUser', otherwise we return an error.
         */
        if LookupAccountSidW(
            lpComputerName,
            pCurrentSid,
            core::ptr::addr_of_mut!(szName) as LPWSTR,
            core::ptr::addr_of_mut!(cchName),
            core::ptr::addr_of_mut!(szDomain) as LPWSTR,
            core::ptr::addr_of_mut!(cchDomain),
            core::ptr::addr_of_mut!(peUse),
        ) != 0
        {
            StringCchCopyW(
                pszUser as STRSAFE_LPWSTR,
                cchUser as size_t,
                core::ptr::addr_of_mut!(szName) as *mut WCHAR as STRSAFE_LPCWSTR,
            );
            Success = (TRUE as i32);
        } else if ConvertSidToStringSidW(pCurrentSid, core::ptr::addr_of_mut!(StringSid)) != 0 {
            /* Copy the string only if the user-provided buffer is big enough */
            if (wcslen(StringSid as *const wchar_t) as SIZE_T + 1 as SIZE_T) <= cchUser {
                StringCchCopyW(
                    pszUser as STRSAFE_LPWSTR,
                    cchUser as size_t,
                    StringSid as STRSAFE_LPCWSTR,
                );
                Success = (TRUE as i32);
            } else {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
                Success = (FALSE as i32);
            }

            /* Free the allocated buffer */
            LocalFree(StringSid as HLOCAL);
        }
    }

    *pLastSid = if Success != 0 { pCurrentSid } else { core::ptr::null_mut() };

    return Success;
}

unsafe extern "C" fn FreeRecords() {
    let mut iIndex: DWORD;
    if g_RecordPtrs.is_null() {
        return;
    }
    iIndex = 0;
    while iIndex < g_TotalRecords {
        if !(*g_RecordPtrs.offset(iIndex as isize)).is_null() {
            HeapFree(GetProcessHeap(), 0, *g_RecordPtrs.offset(iIndex as isize) as PVOID);
        }
        iIndex = iIndex.wrapping_add(1);
    }
    HeapFree(GetProcessHeap(), 0, g_RecordPtrs as PVOID);
    g_RecordPtrs = ::core::ptr::null_mut::<PEVENTLOGRECORD>();
    g_TotalRecords = 0;
}
#[no_mangle]
pub unsafe extern "C" fn FilterByType(EventLogFilter: PEVENTLOGFILTER, pevlr: PEVENTLOGRECORD) -> BOOL {
    if ((*pevlr).EventType as i32 == (EVENTLOG_SUCCESS as i32) && (*EventLogFilter).Information == 0)
        || ((*pevlr).EventType as i32 == (EVENTLOG_INFORMATION_TYPE as i32) && (*EventLogFilter).Information == 0)
        || ((*pevlr).EventType as i32 == (EVENTLOG_WARNING_TYPE as i32) && (*EventLogFilter).Warning == 0)
        || ((*pevlr).EventType as i32 == (EVENTLOG_ERROR_TYPE as i32) && (*EventLogFilter).Error == 0)
        || ((*pevlr).EventType as i32 == (EVENTLOG_AUDIT_SUCCESS as i32) && (*EventLogFilter).AuditSuccess == 0)
        || ((*pevlr).EventType as i32 == (EVENTLOG_AUDIT_FAILURE as i32) && (*EventLogFilter).AuditFailure == 0)
    {
        return (FALSE) as BOOL;
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn FilterByString(FilterString: PCWSTR, String: PWSTR) -> BOOL {
    let mut pStr: PCWSTR = core::ptr::null::<WCHAR>();

    /* The filter string is NULL so it does not filter anything */
    if FilterString.is_null() {
        return (TRUE) as BOOL;
    }

    /*
     * If the filter string filters for an empty string AND the source string
     * is an empty string, we have a match (particular case of the last one).
     */
    if *FilterString == 0 && *String == 0 {
        return (TRUE) as BOOL;
    }

    // if (*FilterString || *String)

    /*
     * If the filter string is empty BUT the source string is not empty,
     * OR vice-versa, we cannot have a match.
     */
    if (*FilterString == 0 && *String != 0) || (*FilterString != 0 && *String == 0) {
        return (FALSE) as BOOL;
    }

    /*
     * If the filter string filters for at least a non-empty string,
     * browse it and search for a string that matches the source string.
     */
    // else if (*FilterString && *String)
    {
        pStr = FilterString;
        while *pStr != 0 {
            if _wcsicmp(pStr as *const WCHAR, String as *const WCHAR) == 0 {
                /* We have a match, break the loop */
                break;
            }

            pStr = pStr.offset(
                (wcslen(pStr as *const WCHAR) + 1) as isize,
            );
        }
        if *pStr == 0 {
            /* We do not have a match */
            return (FALSE) as BOOL;
        }
    }

    /* We have a match */
    return (TRUE) as BOOL;
}

unsafe extern "system" fn EnumEventsThread(lpParameter: LPVOID) -> DWORD {
    let mut EventLogFilter: PEVENTLOGFILTER = lpParameter as PEVENTLOGFILTER;
    let mut EventLog: PEVENTLOG = core::ptr::null_mut();

    let mut LogIndex: ULONG = 0;
    let mut hEventLog: HANDLE = core::ptr::null_mut();
    let mut pEvlr: PEVENTLOGRECORD = core::ptr::null_mut();
    let mut pEvlrEnd: PBYTE = core::ptr::null_mut();
    let mut pEvlrBuffer: PBYTE = core::ptr::null_mut();
    let mut dwWanted: DWORD = 0;
    let mut dwRead: DWORD = 0;
    let mut dwNeeded: DWORD = 0;
    let mut dwStatus: DWORD = ERROR_SUCCESS;
    let mut dwTotalRecords: DWORD = 0;
    let mut dwCurrentRecord: DWORD = 0;
    let mut dwFlags: DWORD = 0;
    let mut dwMaxLength: DWORD = 0;
    let mut cchRemaining: size_t = 0;
    let mut lpszSourceName: LPWSTR = core::ptr::null_mut();
    let mut lpszComputerName: LPWSTR = core::ptr::null_mut();
    let mut bResult: BOOL = (TRUE as i32); /* Read succeeded */
    let mut hProcessHeap: HANDLE = GetProcessHeap();
    let mut pLastSid: PSID = core::ptr::null_mut();

    let mut uStep: UINT = 0;
    let mut uStepAt: UINT = 0;
    let mut uPos: UINT = 0;

    let mut szWindowTitle: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szStatusText: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szLocalDate: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szLocalTime: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szEventID: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szEventTypeText: [WCHAR; MAX_LOADSTRING] = [0; MAX_LOADSTRING];
    let mut szCategoryID: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szUsername: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szNoUsername: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szCategory: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szNoCategory: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpTitleTemplateEnd: PWCHAR = core::ptr::null_mut();

    let mut time: SYSTEMTIME = core::mem::zeroed();
    let mut lviEventItem: LVITEMW = core::mem::zeroed();

    /* Disable the Previous/Next buttons */
    SendMessageW(
        hwndEventDetails,
        EVT_DISPLAY,
        (FALSE) as WPARAM,
        -(1 as LPARAM),
    );

    /* Save the current event log filter globally */
    EventLogFilter_AddRef(EventLogFilter);
    ActiveFilter = EventLogFilter;

    /* HACK!! */
    EventLog = (*EventLogFilter).EventLogs[0];

    // FIXME: Use something else instead of EventLog->LogName !!

    /*
     * Use a different formatting, whether the event log filter holds
     * only one log, or many logs (the latter case is WIP TODO!)
     */
    if (*EventLogFilter).NumOfEventLogs <= 1 {
        StringCchPrintfExW!(
            core::ptr::addr_of_mut!(szWindowTitle) as LPWSTR,
            ARRAYSIZE!(szWindowTitle),
            core::ptr::addr_of_mut!(lpTitleTemplateEnd),
            core::ptr::addr_of_mut!(cchRemaining),
            0,
            core::ptr::addr_of_mut!(szTitleTemplate) as LPCWSTR,
            core::ptr::addr_of_mut!(szTitle) as LPCWSTR,
            (*EventLog).LogName,
        ); /* i = number of characters written */
        dwMaxLength = cchRemaining as DWORD;
        if (*EventLog).ComputerName.is_null() {
            GetComputerNameW(lpTitleTemplateEnd, core::ptr::addr_of_mut!(dwMaxLength));
        } else {
            StringCchCopyW(
                lpTitleTemplateEnd,
                dwMaxLength as size_t,
                (*EventLog).ComputerName,
            );
        }

        StringCbPrintfW!(
            core::ptr::addr_of_mut!(szStatusText) as LPWSTR,
            core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>(),
            core::ptr::addr_of_mut!(szStatusBarTemplate) as LPCWSTR,
            (*EventLog).LogName,
            0,
            0,
        );
    } else {
        // TODO: Use a different title & implement filtering for multi-log filters !!
        // (EventLogFilter->NumOfEventLogs > 1)
        MessageBoxW(
            hwndMainWindow,
            RUSTCONV_WSTR_DD19C0E56A07.as_ptr(),
            core::ptr::addr_of_mut!(szTitle) as LPWSTR,
            MB_OK | MB_ICONINFORMATION,
        );
    }

    /* Set the window title */
    SetWindowTextW(hwndMainWindow, core::ptr::addr_of_mut!(szWindowTitle) as LPWSTR);

    /* Update the status bar */
    StatusBar_SetText!(
        hwndStatus,
        0,
        core::ptr::addr_of_mut!(szStatusText) as LPWSTR,
    );

    /* Disable list view redraw */
    SendMessageW(
        hwndListView,
        WM_SETREDRAW,
        (FALSE) as WPARAM,
        0,
    );

    /* Clear the list view and free the cached records */
    ListView_DeleteAllItems!(hwndListView);
    FreeRecords();

    SendMessageW(
        hwndListView,
        LVM_PROGRESS,
        0,
        (TRUE) as LPARAM,
    );
    ProgressBar_SetRange!(hwndStatusProgress, 0);
    StatusBar_SetText!(hwndStatus, 0, core::ptr::null_mut::<WCHAR>());
    ShowWindow(hwndStatusProgress, (SW_SHOW as i32));

    /* Do a loop over the logs enumerated in the filter */
    // FIXME: For now we only support 1 event log per filter!
    LogIndex = 0;
    // for (LogIndex = 0; LogIndex < EventLogFilter->NumOfEventLogs; ++LogIndex)
    'cleanup: {
        'quit: {
            EventLog = (*core::ptr::addr_of!((*EventLogFilter).EventLogs[0]).add(LogIndex as usize));

            /* Open the event log */
            if (*EventLog).Permanent != 0 {
                hEventLog = OpenEventLogW((*EventLog).ComputerName, (*EventLog).LogName);
            } else {
                hEventLog = OpenBackupEventLogW((*EventLog).ComputerName, (*EventLog).LogName); // FileName
            }

            if hEventLog.is_null() {
                ShowWin32Error(GetLastError());
                break 'cleanup;
            }

            // GetOldestEventLogRecord(hEventLog, &dwThisRecord);

            /* Get the total number of event log records */
            GetNumberOfEventLogRecords(hEventLog, core::ptr::addr_of_mut!(dwTotalRecords));

            if dwTotalRecords > 0 {
                EnableMenuItem(
                    hMainMenu,
                    IDM_CLEAR_EVENTS,
                    MF_BYCOMMAND | MF_ENABLED,
                );
                EnableMenuItem(
                    hMainMenu,
                    IDM_SAVE_EVENTLOG,
                    MF_BYCOMMAND | MF_ENABLED,
                );
            } else {
                EnableMenuItem(
                    hMainMenu,
                    IDM_CLEAR_EVENTS,
                    MF_BYCOMMAND | MF_GRAYED,
                );
                EnableMenuItem(
                    hMainMenu,
                    IDM_SAVE_EVENTLOG,
                    MF_BYCOMMAND | MF_GRAYED,
                );
            }

            /* Set up the event records cache */
            g_RecordPtrs = HeapAlloc(
                hProcessHeap,
                HEAP_ZERO_MEMORY,
                (dwTotalRecords as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<_EVENTLOGRECORD>() as SIZE_T),
            ) as *mut PEVENTLOGRECORD;
            if g_RecordPtrs.is_null() {
                // ShowWin32Error(GetLastError());
                break 'quit;
            }
            g_TotalRecords = dwTotalRecords;

            if WaitForSingleObject(hStopEnumEvent, 0) == WAIT_OBJECT_0 {
                break 'quit;
            }

            LoadStringW(
                hInst,
                IDS_NOT_AVAILABLE,
                core::ptr::addr_of_mut!(szNoUsername) as LPWSTR,
                (ARRAYSIZE!(szNoUsername)) as i32,
            );
            LoadStringW(
                hInst,
                IDS_NONE,
                core::ptr::addr_of_mut!(szNoCategory) as LPWSTR,
                (ARRAYSIZE!(szNoCategory)) as i32,
            );

            ProgressBar_SetRange!(
                hwndStatusProgress,
                MAKELPARAM!(0, 100),
            );
            uStepAt = (dwTotalRecords / 100) + 1;

            dwFlags = EVENTLOG_SEQUENTIAL_READ
                | (if Settings.bNewestEventsFirst != 0 {
                    EVENTLOG_FORWARDS_READ
                } else {
                    EVENTLOG_BACKWARDS_READ
                });

            /* 0x7ffff is the maximum buffer size ReadEventLog will accept */
            dwWanted = 0x7ffff;
            pEvlr = HeapAlloc(
                hProcessHeap,
                0,
                dwWanted as SIZE_T,
            ) as PEVENTLOGRECORD;

            if pEvlr.is_null() {
                break 'quit;
            }

            while dwStatus == ERROR_SUCCESS {
                bResult = ReadEventLogW(
                    hEventLog,
                    dwFlags,
                    0,
                    pEvlr as LPVOID,
                    dwWanted,
                    core::ptr::addr_of_mut!(dwRead),
                    core::ptr::addr_of_mut!(dwNeeded),
                );
                dwStatus = GetLastError();

                if bResult == 0 && dwStatus == ERROR_INSUFFICIENT_BUFFER {
                    pEvlr = HeapReAlloc(
                        hProcessHeap,
                        0,
                        pEvlr as PVOID,
                        dwNeeded as SIZE_T,
                    ) as PEVENTLOGRECORD;
                    dwWanted = dwNeeded;

                    if pEvlr.is_null() {
                        break;
                    }

                    bResult = ReadEventLogW(
                        hEventLog,
                        dwFlags,
                        0,
                        pEvlr as LPVOID,
                        dwNeeded,
                        core::ptr::addr_of_mut!(dwRead),
                        core::ptr::addr_of_mut!(dwNeeded),
                    );

                    if bResult == 0 {
                        break;
                    }
                } else if bResult == 0 {
                    /* Exit on other errors (ERROR_HANDLE_EOF) */
                    break;
                }

                pEvlrBuffer = pEvlr as LPBYTE as PBYTE;
                pEvlrEnd = pEvlrBuffer.add(dwRead as usize);

                while pEvlrBuffer < pEvlrEnd {
                    let mut pEvlrTmp: PEVENTLOGRECORD = pEvlrBuffer as PEVENTLOGRECORD;
                    let mut lpszUsername: PWSTR = core::ptr::null_mut();
                    let mut lpszCategoryName: PWSTR = core::ptr::null_mut();
                    *g_RecordPtrs.add(dwCurrentRecord as usize) = core::ptr::null_mut();

                    // ProgressBar_StepIt(hwndStatusProgress);
                    uStep = uStep.wrapping_add(1);
                    if uStep % uStepAt == 0 {
                        uPos = uPos.wrapping_add(1);
                        ProgressBar_SetPos!(hwndStatusProgress, uPos);
                    }

                    if WaitForSingleObject(hStopEnumEvent, 0) == WAIT_OBJECT_0 {
                        break 'quit;
                    }

                    'skip: {
                        /* Filter by event type */
                        if FilterByType(EventLogFilter, pEvlrTmp) == 0 {
                            break 'skip;
                        }

                        /* Get the event source name and filter it */
                        lpszSourceName = pEvlrBuffer
                            .add(core::mem::size_of::<EVENTLOGRECORD>() as usize) as LPWSTR;
                        if FilterByString((*EventLogFilter).Sources as PCWSTR, lpszSourceName as PWSTR) == 0 {
                            break 'skip;
                        }

                        /* Get the computer name and filter it */
                        lpszComputerName = pEvlrBuffer.add(
                            core::mem::size_of::<EVENTLOGRECORD>()
                                + (wcslen(lpszSourceName) as usize
                                    + 1 as usize)
                                    * core::mem::size_of::<WCHAR>(),
                        ) as LPWSTR;
                        if FilterByString((*EventLogFilter).ComputerNames as PCWSTR, lpszComputerName as PWSTR) == 0 {
                            break 'skip;
                        }

                        /* Compute the event time */
                        EventTimeToSystemTime((*pEvlrTmp).TimeWritten, core::ptr::addr_of_mut!(time));
                        GetDateFormatW(
                            LOCALE_USER_DEFAULT,
                            DATE_SHORTDATE,
                            core::ptr::addr_of_mut!(time),
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!(szLocalDate) as LPWSTR,
                            (ARRAYSIZE!(szLocalDate)) as i32,
                        );
                        GetTimeFormatW(
                            LOCALE_USER_DEFAULT,
                            0,
                            core::ptr::addr_of_mut!(time),
                            core::ptr::null_mut(),
                            core::ptr::addr_of_mut!(szLocalTime) as LPWSTR,
                            (ARRAYSIZE!(szLocalTime)) as i32,
                        );

                        /* Get the username that generated the event, and filter it */
                        lpszUsername = (if GetEventUserName(
                            pEvlrTmp,
                            core::ptr::addr_of_mut!(pLastSid),
                            core::ptr::addr_of_mut!(szUsername) as PWSTR,
                            (_countof!(szUsername)) as SIZE_T,
                        ) != 0
                        {
                            core::ptr::addr_of_mut!(szUsername) as PWSTR
                        } else {
                            core::ptr::addr_of_mut!(szNoUsername) as PWSTR
                        }) as PWSTR;

                        if FilterByString((*EventLogFilter).Users as PCWSTR, lpszUsername) == 0 {
                            break 'skip;
                        }

                        // TODO: Filter by event ID and category
                        GetEventType(
                            (*pEvlrTmp).EventType,
                            core::ptr::addr_of_mut!(szEventTypeText) as PWSTR,
                            (_countof!(szEventTypeText)) as SIZE_T,
                        );

                        lpszCategoryName = (if GetEventCategory(
                            (*EventLog).LogName as PCWSTR,
                            lpszSourceName as PCWSTR,
                            pEvlrTmp,
                            core::ptr::addr_of_mut!(szCategory) as PWSTR,
                            (_countof!(szCategory)) as SIZE_T,
                        ) != 0
                        {
                            core::ptr::addr_of_mut!(szCategory) as PWSTR
                        } else {
                            core::ptr::addr_of_mut!(szNoCategory) as PWSTR
                        }) as PWSTR;

                        StringCbPrintfW!(
                            core::ptr::addr_of_mut!(szEventID) as LPWSTR,
                            core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>(),
                            RUSTCONV_WSTR_69A283E639E4.as_ptr(),
                            ((*pEvlrTmp).EventID & 0xFFFF),
                        );
                        StringCbPrintfW!(
                            core::ptr::addr_of_mut!(szCategoryID) as LPWSTR,
                            core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>(),
                            RUSTCONV_WSTR_69A283E639E4.as_ptr(),
                            (*pEvlrTmp).EventCategory as core::ffi::c_uint,
                        );

                        *g_RecordPtrs.add(dwCurrentRecord as usize) = HeapAlloc(
                            hProcessHeap,
                            0,
                            (*pEvlrTmp).Length as SIZE_T,
                        ) as PEVENTLOGRECORD;
                        CopyMemory!(
                            *g_RecordPtrs.add(dwCurrentRecord as usize) as LPVOID,
                            pEvlrTmp as LPVOID,
                            (*pEvlrTmp).Length as usize,
                        );

                        lviEventItem.mask = LVIF_IMAGE | LVIF_TEXT | LVIF_PARAM;
                        lviEventItem.iItem = 0;
                        lviEventItem.iSubItem = 0;
                        lviEventItem.lParam = *g_RecordPtrs.add(dwCurrentRecord as usize) as LPARAM;
                        lviEventItem.pszText = core::ptr::addr_of_mut!(szEventTypeText) as LPWSTR;

                        match ((*pEvlrTmp).EventType as i32) as u32 {
                            EVENTLOG_SUCCESS | EVENTLOG_INFORMATION_TYPE => {
                                lviEventItem.iImage = 0;
                            }
                            EVENTLOG_WARNING_TYPE => {
                                lviEventItem.iImage = 1;
                            }
                            EVENTLOG_ERROR_TYPE => {
                                lviEventItem.iImage = 2;
                            }
                            EVENTLOG_AUDIT_SUCCESS => {
                                lviEventItem.iImage = 3;
                            }
                            EVENTLOG_AUDIT_FAILURE => {
                                lviEventItem.iImage = 4;
                            }
                            _ => {}
                        }

                        lviEventItem.iItem = ListView_InsertItem!(
                            hwndListView,
                            core::ptr::addr_of_mut!(lviEventItem),
                        );

                        ListView_SetItemText!(
                            hwndListView,
                            lviEventItem.iItem,
                            1,
                            core::ptr::addr_of_mut!(szLocalDate) as LPWSTR,
                        );
                        ListView_SetItemText!(
                            hwndListView,
                            lviEventItem.iItem,
                            2,
                            core::ptr::addr_of_mut!(szLocalTime) as LPWSTR,
                        );
                        ListView_SetItemText!(
                            hwndListView,
                            lviEventItem.iItem,
                            3,
                            lpszSourceName,
                        );
                        ListView_SetItemText!(
                            hwndListView,
                            lviEventItem.iItem,
                            4,
                            lpszCategoryName,
                        );
                        ListView_SetItemText!(
                            hwndListView,
                            lviEventItem.iItem,
                            5,
                            core::ptr::addr_of_mut!(szEventID) as LPWSTR,
                        );
                        ListView_SetItemText!(
                            hwndListView,
                            lviEventItem.iItem,
                            6,
                            lpszUsername,
                        );
                        ListView_SetItemText!(
                            hwndListView,
                            lviEventItem.iItem,
                            7,
                            lpszComputerName,
                        );
                    }
                    // SkipEvent:
                    pEvlrBuffer = pEvlrBuffer.add((*pEvlrTmp).Length as usize);
                    dwCurrentRecord = dwCurrentRecord.wrapping_add(1);
                }
            }
        }
        // Quit:

        if !pEvlr.is_null() {
            HeapFree(hProcessHeap, 0, pEvlr as PVOID);
        }

        /* Close the event log */
        CloseEventLog(hEventLog);
    } // end-for (LogIndex)

    /* All events loaded */

    // Cleanup:
    ShowWindow(hwndStatusProgress, (SW_HIDE as i32));
    SendMessageW(hwndListView, LVM_PROGRESS, 0, (FALSE) as LPARAM);

    // FIXME: Use something else instead of EventLog->LogName !!

    /*
     * Use a different formatting, whether the event log filter holds
     * only one log, or many logs (the latter case is WIP TODO!)
     */
    if (*EventLogFilter).NumOfEventLogs <= 1 {
        StringCbPrintfW!(
            core::ptr::addr_of_mut!(szStatusText) as LPWSTR,
            core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>(),
            core::ptr::addr_of_mut!(szStatusBarTemplate) as LPCWSTR,
            (*EventLog).LogName,
            dwTotalRecords,
            ListView_GetItemCount!(hwndListView),
        );
    } else {
        // TODO: Use a different title & implement filtering for multi-log filters !!
        // (EventLogFilter->NumOfEventLogs > 1)
    }

    /* Update the status bar */
    StatusBar_SetText!(
        hwndStatus,
        0,
        core::ptr::addr_of_mut!(szStatusText) as LPWSTR,
    );

    /* Resume list view redraw */
    SendMessageW(
        hwndListView,
        WM_SETREDRAW,
        (TRUE) as WPARAM,
        0,
    );

    /* Re-enable the Previous/Next buttons, keeping the current event details
     * displayed, if any. Don't auto-select the first list item but wait for
     * the user to do it. */
    SendMessageW(
        hwndEventDetails,
        EVT_DISPLAY,
        (FALSE) as WPARAM,
        0,
    );

    EventLogFilter_Release(EventLogFilter);

    CloseHandle(hStopEnumEvent);
    InterlockedExchangePointer(core::ptr::addr_of_mut!(hStopEnumEvent) as *mut PVOID, core::ptr::null_mut());

    return 0;
}

unsafe extern "system" fn StartStopEnumEventsThread(lpParameter: LPVOID) -> DWORD {
    let mut WaitHandles: [HANDLE; 2] = [core::ptr::null_mut(); 2];
    let mut WaitResult: DWORD;

    WaitHandles[0] = hStartStopEnumEvent; // End-of-application event
    WaitHandles[1] = hStartEnumEvent;     // Command event

    loop {
        WaitResult = WaitForMultipleObjects(ARRAYSIZE!(WaitHandles) as DWORD,
                                            addr_of_mut!(WaitHandles) as *mut HANDLE,
                                            (FALSE) as BOOL, // WaitAny
                                            INFINITE);
        'sw: {
            match WaitResult {
                v if v == WAIT_OBJECT_0 + 0 => {
                    /* End-of-application event signaled, quit this thread */

                    /* Stop the previous enumeration */
                    if !hEnumEventsThread.is_null() {
                        if !hStopEnumEvent.is_null() {
                            SetEvent(hStopEnumEvent);
                            WaitForSingleObject(hEnumEventsThread, INFINITE);
                            // NOTE: The following is done by the enumeration thread just before terminating.
                            // hStopEnumEvent = NULL;
                        }

                        CloseHandle(hEnumEventsThread);
                        hEnumEventsThread = core::ptr::null_mut();
                    }

                    /* Clear the list view and free the cached records */
                    ListView_DeleteAllItems!(hwndListView);
                    FreeRecords();

                    /* Reset the active filter */
                    ActiveFilter = core::ptr::null_mut();

                    return 0 as DWORD;
                }

                v if v == WAIT_OBJECT_0 + 1 => {
                    /* Restart a new enumeration if needed */
                    let mut EventLogFilter: PEVENTLOGFILTER = core::ptr::null_mut();

                    /* Stop the previous enumeration */
                    if !hEnumEventsThread.is_null() {
                        if !hStopEnumEvent.is_null() {
                            SetEvent(hStopEnumEvent);
                            WaitForSingleObject(hEnumEventsThread, INFINITE);
                            // NOTE: The following is done by the enumeration thread just before terminating.
                            // hStopEnumEvent = NULL;
                        }

                        CloseHandle(hEnumEventsThread);
                        hEnumEventsThread = core::ptr::null_mut();
                    }

                    /* Clear the list view and free the cached records */
                    ListView_DeleteAllItems!(hwndListView);
                    FreeRecords();

                    /* Reset the active filter */
                    ActiveFilter = core::ptr::null_mut();

                    EventLogFilter = InterlockedExchangePointer(
                        addr_of_mut!(EnumFilter) as *mut PVOID,
                        core::ptr::null_mut(),
                    ) as PEVENTLOGFILTER;
                    if EventLogFilter.is_null() {
                        break 'sw;
                    }

                    // Manual-reset event
                    hStopEnumEvent = CreateEventW(
                        core::ptr::null_mut(),
                        (TRUE) as BOOL,
                        (FALSE) as BOOL,
                        core::ptr::null(),
                    );
                    if hStopEnumEvent.is_null() {
                        break 'sw;
                    }

                    hEnumEventsThread = CreateThread(
                        core::ptr::null_mut(),
                        0,
                        Some(EnumEventsThread as unsafe extern "system" fn(LPVOID) -> DWORD),
                        EventLogFilter as LPVOID,
                        CREATE_SUSPENDED,
                        core::ptr::null_mut(),
                    );
                    if hEnumEventsThread.is_null() {
                        CloseHandle(hStopEnumEvent);
                        hStopEnumEvent = core::ptr::null_mut();
                        break 'sw;
                    }
                    // CloseHandle(hEnumEventsThread);
                    ResumeThread(hEnumEventsThread);

                    break 'sw;
                }

                _ => {
                    /* Unknown command, must never go there! */
                    return GetLastError();
                }
            }
        }
    }

    return 0 as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn EnumEvents(EventLogFilter: PEVENTLOGFILTER) {
    InterlockedExchangePointer(
        core::ptr::addr_of_mut!(EnumFilter) as *mut PVOID as *mut *mut core::ffi::c_void,
        EventLogFilter as *mut core::ffi::c_void,
    );
    SetEvent(hStartEnumEvent);
}
#[no_mangle]
pub unsafe extern "C" fn GetSelectedFilter(phti: *mut HTREEITEM) -> PEVENTLOGFILTER {
    let mut tvItemEx: TVITEMEXW = core::mem::zeroed();
    let mut hti: HTREEITEM;

    if !phti.is_null() {
        *phti = core::ptr::null_mut();
    }

    /* Get index of selected item */
    hti = TreeView_GetSelection!(hwndTreeView);
    if hti.is_null() {
        return core::ptr::null_mut(); // No filter
    }

    tvItemEx.mask = TVIF_PARAM;
    tvItemEx.hItem = hti;

    TreeView_GetItem!(hwndTreeView, core::ptr::addr_of_mut!(tvItemEx));

    if !phti.is_null() {
        *phti = tvItemEx.hItem;
    }

    return tvItemEx.lParam as PEVENTLOGFILTER;
}
#[no_mangle]
pub unsafe extern "C" fn OpenUserEventLogFile(lpszFileName: LPCWSTR) {
    let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hFind: HANDLE = core::ptr::null_mut();
    let mut EventLog: PEVENTLOG = core::ptr::null_mut();
    let mut EventLogFilter: PEVENTLOGFILTER = core::ptr::null_mut();
    let mut cchFileName: SIZE_T = 0;
    let mut hItem: HTREEITEM = core::ptr::null_mut();

    /* Check whether the file actually exists */
    hFind = FindFirstFileW(lpszFileName, core::ptr::addr_of_mut!(FindData));
    if hFind == INVALID_HANDLE_VALUE {
        ShowWin32Error(GetLastError());
        return;
    }
    FindClose(hFind);

    /* Allocate a new event log entry */
    EventLog = AllocEventLog(core::ptr::null::<WCHAR>(), lpszFileName as PCWSTR, (FALSE) as BOOL);
    if EventLog.is_null() {
        ShowWin32Error(ERROR_NOT_ENOUGH_MEMORY);
        return;
    }

    /* Allocate a new event log filter entry for this event log */
    EventLogFilter = AllocEventLogFilter(
        // LogName,
        (TRUE) as BOOL, (TRUE) as BOOL, (TRUE) as BOOL, (TRUE) as BOOL, (TRUE) as BOOL,
        core::ptr::null::<WCHAR>(), core::ptr::null::<WCHAR>(), core::ptr::null::<WCHAR>(),
        1, core::ptr::addr_of_mut!(EventLog));
    if EventLogFilter.is_null() {
        ShowWin32Error(ERROR_NOT_ENOUGH_MEMORY);
        EventLog_Free(EventLog);
        return;
    }

    /* Add the event log and the filter into their lists */
    InsertTailList(core::ptr::addr_of_mut!(EventLogList), core::ptr::addr_of_mut!((*EventLog).ListEntry));
    InsertTailList(core::ptr::addr_of_mut!(EventLogFilterList), core::ptr::addr_of_mut!((*EventLogFilter).ListEntry));

    /* Retrieve and cache the event log file */
    cchFileName = (wcslen(lpszFileName) + 1) as _;
    (*EventLog).FileName = HeapAlloc(GetProcessHeap(), 0, cchFileName * core::mem::size_of::<WCHAR>() as SIZE_T) as PWSTR;
    if !(*EventLog).FileName.is_null() {
        StringCchCopyW((*EventLog).FileName, (cchFileName) as _, lpszFileName);
    }

    hItem = TreeViewAddItem(hwndTreeView, htiUserLogs,
                            lpszFileName as LPWSTR,
                            2, 3, EventLogFilter as LPARAM);

    /* Select the event log */
    if !hItem.is_null() {
        // TreeView_Expand(hwndTreeView, htiUserLogs, TVE_EXPAND);
        TreeView_SelectItem!(hwndTreeView, hItem);
        TreeView_EnsureVisible!(hwndTreeView, hItem);
    }
    InvalidateRect(hwndTreeView, core::ptr::null::<RECT>(), (FALSE) as BOOL);
    SetFocus(hwndTreeView);
}
#[no_mangle]
pub unsafe extern "C" fn OpenUserEventLog() {
    let mut szFileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    ZeroMemory!(
        &raw mut szFileName as *mut WCHAR as *mut core::ffi::c_void,
        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>(),
    );

    sfn.lpstrFile = &raw mut szFileName as *mut WCHAR as LPWSTR;
    sfn.nMaxFile = ARRAYSIZE!(szFileName) as DWORD;

    if GetOpenFileNameW(&raw mut sfn) == 0 {
        return;
    }
    *sfn.lpstrFile.offset(
        sfn.nMaxFile.wrapping_sub(1 as DWORD) as isize
    ) = UNICODE_NULL as WCHAR;

    OpenUserEventLogFile(sfn.lpstrFile as LPCWSTR);
}
#[no_mangle]
pub unsafe extern "C" fn SaveEventLog(mut EventLogFilter: PEVENTLOGFILTER) {
    let mut EventLog: PEVENTLOG = core::ptr::null_mut();
    let mut hEventLog: HANDLE = core::ptr::null_mut();
    let mut szFileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    /* Bail out if there is no available filter */
    if EventLogFilter.is_null() {
        return;
    }

    ZeroMemory!(
        szFileName.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>(),
    );

    sfn.lpstrFile = szFileName.as_mut_ptr() as LPWSTR;
    sfn.nMaxFile = ARRAYSIZE!(szFileName) as DWORD;

    if GetSaveFileNameW(core::ptr::addr_of_mut!(sfn)) == 0 {
        return;
    }

    EventLogFilter_AddRef(EventLogFilter);

    EventLog = (*(*EventLogFilter).EventLogs.as_ptr().add(0 as usize));
    hEventLog = OpenEventLogW(
        (*EventLog).ComputerName as LPCWSTR,
        (*EventLog).LogName as LPCWSTR,
    );

    EventLogFilter_Release(EventLogFilter);

    if hEventLog.is_null() {
        ShowWin32Error(GetLastError());
        return;
    }

    if BackupEventLogW(hEventLog, szFileName.as_mut_ptr() as LPCWSTR) == 0 {
        ShowWin32Error(GetLastError());
    }

    CloseEventLog(hEventLog);
}
#[no_mangle]
pub unsafe extern "C" fn CloseUserEventLog(
    EventLogFilter: PEVENTLOGFILTER,
    hti: HTREEITEM,
) {
    /* Bail out if there is no available filter */
    if EventLogFilter.is_null() {
        return;
    }

    if InterlockedCompareExchangePointer(
        &raw mut ActiveFilter as *mut PVOID as *mut *mut ::core::ffi::c_void,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == EventLogFilter as *mut ::core::ffi::c_void
    {
        /* Signal the enumerator thread we want to stop enumerating events */
        // EnumEvents(NULL);
        InterlockedExchangePointer(
            &raw mut EnumFilter as *mut PVOID as *mut *mut ::core::ffi::c_void,
            core::ptr::null_mut(),
        );
        SetEvent(hStartEnumEvent);
    }

    /*
     * The deletion of the item automatically triggers a TVN_SELCHANGED
     * notification, that will reset the ActiveFilter (in case the item
     * selected is a filter). Otherwise we reset it there.
     */
    TreeView_DeleteItem!(hwndTreeView, hti);

    /* Remove the filter from the list */
    RemoveEntryList(&raw mut (*EventLogFilter).ListEntry);
    EventLogFilter_Release(EventLogFilter);

    // /* Select the default event log */
    // // TreeView_Expand(hwndTreeView, htiUserLogs, TVE_EXPAND);
    // TreeView_SelectItem(hwndTreeView, hItem);
    // TreeView_EnsureVisible(hwndTreeView, hItem);
    InvalidateRect(hwndTreeView, core::ptr::null_mut(), (FALSE) as BOOL);
    SetFocus(hwndTreeView);
}
#[no_mangle]
pub unsafe extern "C" fn ClearEvents(EventLogFilter: PEVENTLOGFILTER) -> BOOL {
    let mut Success: BOOL;
    let mut EventLog: PEVENTLOG;
    let mut hEventLog: HANDLE;
    let mut szFileName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szMessage: [WCHAR; MAX_LOADSTRING] = [0; MAX_LOADSTRING];

    /* Bail out if there is no available filter */
    if EventLogFilter.is_null() {
        return (FALSE) as BOOL;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(szFileName) as *mut WCHAR as *mut core::ffi::c_void,
        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>(),
    );
    ZeroMemory!(
        core::ptr::addr_of_mut!(szMessage) as *mut WCHAR as *mut core::ffi::c_void,
        core::mem::size_of::<[WCHAR; MAX_LOADSTRING]>(),
    );

    LoadStringW(
        hInst,
        IDS_CLEAREVENTS_MSG as UINT,
        core::ptr::addr_of_mut!(szMessage) as *mut WCHAR as LPWSTR,
        (core::mem::size_of::<[WCHAR; MAX_LOADSTRING]>()
            / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
    );

    sfn.lpstrFile = core::ptr::addr_of_mut!(szFileName) as *mut WCHAR as LPWSTR;
    sfn.nMaxFile = (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>()
        / core::mem::size_of::<WCHAR>()) as DWORD;

    match (MessageBoxW(
        hwndMainWindow,
        core::ptr::addr_of_mut!(szMessage) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR,
        (MB_YESNOCANCEL | MB_ICONINFORMATION) as UINT,
    )) as u32 {
        IDCANCEL => {
            return (FALSE) as BOOL;
        }
        IDNO => {
            sfn.lpstrFile = core::ptr::null_mut::<WCHAR>();
        }
        IDYES => {
            if GetSaveFileNameW(core::ptr::addr_of_mut!(sfn)) == 0 {
                return (FALSE) as BOOL;
            }
        }
        _ => {}
    }

    EventLogFilter_AddRef(EventLogFilter);

    EventLog = (*(*EventLogFilter).EventLogs.as_ptr().add(0 as usize));
    hEventLog = OpenEventLogW((*EventLog).ComputerName, (*EventLog).LogName);

    EventLogFilter_Release(EventLogFilter);

    if hEventLog.is_null() {
        ShowWin32Error(GetLastError());
        return (FALSE) as BOOL;
    }

    Success = ClearEventLogW(hEventLog, sfn.lpstrFile as LPCWSTR);
    if Success == 0 {
        ShowWin32Error(GetLastError());
    }

    CloseEventLog(hEventLog);
    return Success;
}
#[no_mangle]
pub unsafe extern "C" fn Refresh(EventLogFilter: PEVENTLOGFILTER) {
    if EventLogFilter.is_null() {
        return;
    }
    EnumEvents(EventLogFilter);
}
#[no_mangle]
pub unsafe extern "C" fn MyRegisterClass(hInstance: HINSTANCE) -> ATOM {
    let mut wcex: WNDCLASSEXW = core::mem::zeroed();

    wcex.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    wcex.style = 0;
    wcex.lpfnWndProc = Some(WndProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT);
    wcex.cbClsExtra = 0;
    wcex.cbWndExtra = 0;
    wcex.hInstance = hInstance;
    wcex.hIcon = LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_EVENTVWR));
    wcex.hCursor = LoadCursorW(core::ptr::null_mut(), IDC_ARROW);
    wcex.hbrBackground = (COLOR_3DFACE + 1) as HBRUSH; // COLOR_WINDOW + 1
    wcex.lpszMenuName = MAKEINTRESOURCEW!(IDM_EVENTVWR);
    wcex.lpszClassName = EVENTVWR_WNDCLASS;
    wcex.hIconSm = LoadImageW(
        hInstance,
        MAKEINTRESOURCEW!(IDI_EVENTVWR),
        IMAGE_ICON,
        16,
        16,
        LR_SHARED,
    ) as HICON;

    return RegisterClassExW(core::ptr::addr_of_mut!(wcex));
}
#[no_mangle]
pub unsafe extern "C" fn GetDisplayNameFileAndID(
    LogName: PCWSTR,
    pModuleName: PWSTR,
    cchName: SIZE_T,
    pdwMessageID: PDWORD,
) -> BOOL {
    let mut Success: BOOL = (FALSE as i32);
    let mut Result: LONG;
    let mut hLogKey: HKEY = core::ptr::null_mut();
    let mut KeyPath: PWSTR;
    let mut cbKeyPath: SIZE_T;
    let mut dwType: DWORD = core::mem::zeroed();
    let mut cbData: DWORD;
    let mut dwMessageID: DWORD = 0;
    let mut szModuleName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    /* Use a default value for the message ID */
    *pdwMessageID = 0;

    if cchName == 0 {
        return (FALSE) as BOOL;
    }

    cbKeyPath = ((wcslen(EVENTLOG_BASE_KEY) as SIZE_T)
        .wrapping_add(wcslen(LogName) as SIZE_T)
        .wrapping_add(1 as SIZE_T))
        .wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T);
    KeyPath = HeapAlloc(GetProcessHeap(), 0, cbKeyPath) as PWSTR;
    if KeyPath.is_null() {
        ShowWin32Error(ERROR_NOT_ENOUGH_MEMORY);
        return (FALSE) as BOOL;
    }

    StringCbCopyW(KeyPath, (cbKeyPath) as _, EVENTLOG_BASE_KEY);
    StringCbCatW(KeyPath, (cbKeyPath) as _, LogName);

    Result = RegOpenKeyExW(
        hkMachine,
        KeyPath,
        0,
        KEY_QUERY_VALUE,
        core::ptr::addr_of_mut!(hLogKey),
    );
    HeapFree(GetProcessHeap(), 0, (KeyPath).cast());
    if Result != (ERROR_SUCCESS as i32) {
        ShowWin32Error(Result as DWORD);
        return (FALSE) as BOOL;
    }

    cbData = ::core::mem::size_of::<[WCHAR; MAX_PATH as usize]>() as DWORD;
    Result = RegQueryValueExW(
        hLogKey,
        RUSTCONV_WSTR_C44D38660B9F.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(szModuleName) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(cbData),
    );
    if (Result != (ERROR_SUCCESS as i32))
        || (dwType != REG_EXPAND_SZ && dwType != REG_SZ)
    {
        szModuleName[0] = UNICODE_NULL;
    } else {
        /* NULL-terminate the string and expand it */
        szModuleName[((cbData as usize) / ::core::mem::size_of::<WCHAR>())
            - (1 as usize)] = UNICODE_NULL;
        Success = ((GetExpandedFilePathName(
            lpComputerName,
            core::ptr::addr_of_mut!(szModuleName) as *mut WCHAR,
            pModuleName,
            cchName,
        ) != 0)
            && (*pModuleName != 0)) as BOOL;
    }

    /*
     * If we have a 'DisplayNameFile', query for 'DisplayNameID';
     * otherwise it's not really useful. 'DisplayNameID' is optional.
     */
    if Success != 0 {
        cbData = ::core::mem::size_of::<DWORD>() as DWORD;
        Result = RegQueryValueExW(
            hLogKey,
            RUSTCONV_WSTR_DFF1564CD4E4.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            core::ptr::addr_of_mut!(dwMessageID) as LPBYTE,
            core::ptr::addr_of_mut!(cbData),
        );
        if (Result != (ERROR_SUCCESS as i32)) || (dwType != REG_DWORD) {
            dwMessageID = 0;
        }

        *pdwMessageID = dwMessageID;
    }

    RegCloseKey(hLogKey);

    return Success;
}
#[no_mangle]
pub unsafe extern "C" fn BuildLogListAndFilterList(lpComputerNameArg: LPCWSTR) {
    // The C parameter is `lpComputerName`, shadowing the global of the same name; Rust forbids a
    // parameter shadowing a static, so it is renamed. Every use below is the PARAMETER, as in C.
    let mut Result: LONG = 0;
    let mut hEventLogKey: HKEY = core::ptr::null_mut();
    let mut hLogKey: HKEY = core::ptr::null_mut();
    let mut dwNumLogs: DWORD = 0;
    let mut dwIndex: DWORD = 0;
    let mut dwMaxKeyLength: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut EventLog: PEVENTLOG = core::ptr::null_mut();
    let mut EventLogFilter: PEVENTLOGFILTER = core::ptr::null_mut();
    let mut LogName: LPWSTR = core::ptr::null_mut();
    let mut szModuleName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpcName: DWORD = 0;
    let mut dwMessageID: DWORD = 0;
    let mut lpDisplayName: LPWSTR = core::ptr::null_mut();
    let mut hRootNode: HTREEITEM = core::ptr::null_mut();
    let mut hItem: HTREEITEM = core::ptr::null_mut();
    let mut hItemDefault: HTREEITEM = core::ptr::null_mut();

    if !hkMachine.is_null() && hkMachine != HKEY_LOCAL_MACHINE {
        /* We are connected to some other computer, close the old connection */
        RegCloseKey(hkMachine);
        hkMachine = core::ptr::null_mut();
    }
    if lpComputerNameArg.is_null() || *lpComputerNameArg == 0 {
        /* Use the local computer registry */
        hkMachine = HKEY_LOCAL_MACHINE;
    } else {
        /* Connect to the remote computer registry */
        Result = RegConnectRegistry(
            lpComputerNameArg,
            HKEY_LOCAL_MACHINE,
            core::ptr::addr_of_mut!(hkMachine),
        ) as LONG;
        if Result != (ERROR_SUCCESS as i32) {
            /* Connection failed, display a message and bail out */
            hkMachine = core::ptr::null_mut();
            ShowWin32Error(GetLastError());
            return;
        }
    }

    /* Open the EventLog key */
    Result = RegOpenKeyExW(
        hkMachine,
        EVENTLOG_BASE_KEY,
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hEventLogKey),
    ) as LONG;
    if Result != (ERROR_SUCCESS as i32) {
        return;
    }

    'quit: {
        /* Retrieve the number of event logs enumerated as registry keys */
        Result = RegQueryInfoKeyW(
            hEventLogKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwNumLogs),
            core::ptr::addr_of_mut!(dwMaxKeyLength),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as LONG;
        if Result != (ERROR_SUCCESS as i32) {
            break 'quit;
        }
        if dwNumLogs == 0 {
            break 'quit;
        }

        /* Take the NULL terminator into account */
        dwMaxKeyLength = dwMaxKeyLength.wrapping_add(1);

        /* Allocate the temporary buffer */
        LogName = HeapAlloc(
            GetProcessHeap(),
            0,
            (dwMaxKeyLength as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if LogName.is_null() {
            break 'quit;
        }

        /* Enumerate and retrieve each event log name */
        dwIndex = 0;
        while dwIndex < dwNumLogs {
            'body: {
                lpcName = dwMaxKeyLength;
                Result = RegEnumKeyExW(
                    hEventLogKey,
                    dwIndex,
                    LogName,
                    core::ptr::addr_of_mut!(lpcName),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) as LONG;
                if Result != (ERROR_SUCCESS as i32) {
                    break 'body;
                }

                /* Take the NULL terminator into account */
                lpcName = lpcName.wrapping_add(1);

                /* Allocate a new event log entry */
                EventLog = AllocEventLog(
                    lpComputerNameArg as PCWSTR,
                    LogName as PCWSTR,
                    (TRUE) as BOOL,
                ) as PEVENTLOG;
                if EventLog.is_null() {
                    break 'body;
                }

                /* Allocate a new event log filter entry for this event log */
                EventLogFilter = AllocEventLogFilter(
                    (TRUE) as BOOL,
                    (TRUE) as BOOL,
                    (TRUE) as BOOL,
                    (TRUE) as BOOL,
                    (TRUE) as BOOL,
                    core::ptr::null(),
                    core::ptr::null(),
                    core::ptr::null(),
                    1,
                    core::ptr::addr_of_mut!(EventLog),
                ) as PEVENTLOGFILTER;
                if EventLogFilter.is_null() {
                    EventLog_Free(EventLog);
                    break 'body;
                }

                /* Add the event log and the filter into their lists */
                InsertTailList(
                    core::ptr::addr_of_mut!(EventLogList),
                    core::ptr::addr_of_mut!((*EventLog).ListEntry),
                );
                InsertTailList(
                    core::ptr::addr_of_mut!(EventLogFilterList),
                    core::ptr::addr_of_mut!((*EventLogFilter).ListEntry),
                );

                (*EventLog).FileName = core::ptr::null_mut();

                /* Retrieve and cache the event log file */
                Result = RegOpenKeyExW(
                    hEventLogKey,
                    LogName as LPCWSTR,
                    0,
                    KEY_QUERY_VALUE,
                    core::ptr::addr_of_mut!(hLogKey),
                ) as LONG;
                if Result == (ERROR_SUCCESS as i32) {
                    lpcName = 0;
                    Result = RegQueryValueExW(
                        hLogKey,
                        RUSTCONV_WSTR_C4BA81AEA105.as_ptr(),
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(dwType),
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(lpcName),
                    ) as LONG;
                    if (Result != (ERROR_SUCCESS as i32))
                        || (dwType != REG_EXPAND_SZ && dwType != REG_SZ)
                    {
                        // Windows' EventLog uses some kind of default value, we do not.
                        (*EventLog).FileName = core::ptr::null_mut();
                    } else {
                        lpcName = ROUND_DOWN!(lpcName, core::mem::size_of::<WCHAR>() as DWORD);
                        (*EventLog).FileName = HeapAlloc(
                            GetProcessHeap(),
                            0,
                            lpcName as SIZE_T,
                        ) as PWSTR;
                        if !(*EventLog).FileName.is_null() {
                            Result = RegQueryValueExW(
                                hLogKey,
                                RUSTCONV_WSTR_C4BA81AEA105.as_ptr(),
                                core::ptr::null_mut(),
                                core::ptr::addr_of_mut!(dwType),
                                (*EventLog).FileName as LPBYTE,
                                core::ptr::addr_of_mut!(lpcName),
                            ) as LONG;
                            if Result != (ERROR_SUCCESS as i32) {
                                HeapFree(
                                    GetProcessHeap(),
                                    0,
                                    (*EventLog).FileName as PVOID,
                                );
                                (*EventLog).FileName = core::ptr::null_mut();
                            } else {
                                *(*EventLog).FileName.offset(
                                    ((lpcName as usize / core::mem::size_of::<WCHAR>())
                                        - 1 as usize) as isize,
                                ) = UNICODE_NULL;
                            }
                        }
                    }

                    RegCloseKey(hLogKey);
                }

                /* Get the display name for the event log */
                lpDisplayName = core::ptr::null_mut();

                szModuleName[0] = UNICODE_NULL;
                if GetDisplayNameFileAndID(
                    LogName,
                    szModuleName.as_mut_ptr(),
                    (_countof!(szModuleName)) as SIZE_T,
                    core::ptr::addr_of_mut!(dwMessageID),
                ) != 0
                {
                    /* Retrieve the message string without appending extra newlines */
                    lpDisplayName = GetMessageStringFromDll(
                        szModuleName.as_mut_ptr() as LPCWSTR,
                        (FORMAT_MESSAGE_ALLOCATE_BUFFER
                            | FORMAT_MESSAGE_FROM_HMODULE
                            | FORMAT_MESSAGE_IGNORE_INSERTS
                            | FORMAT_MESSAGE_MAX_WIDTH_MASK) as DWORD,
                        dwMessageID,
                        0,
                        core::ptr::null_mut(),
                    ) as LPWSTR;
                }

                /*
                 * Select the correct tree root node, whether the log is a System
                 * or an Application log. Default to Application log otherwise.
                 */
                hRootNode = htiAppLogs;
                lpcName = 0;
                while (lpcName as usize) < ARRAYSIZE!(SystemLogs) as usize {
                    /* Check whether the log name is part of the system logs */
                    if _wcsicmp(LogName as LPCWSTR, SystemLogs[lpcName as usize])
                        == 0
                    {
                        hRootNode = htiSystemLogs;
                        break;
                    }
                    lpcName = lpcName.wrapping_add(1);
                }

                hItem = TreeViewAddItem(
                    hwndTreeView,
                    hRootNode,
                    if !lpDisplayName.is_null() {
                        lpDisplayName
                    } else {
                        LogName
                    },
                    2,
                    3,
                    EventLogFilter as LPARAM,
                );

                /* Try to get the default event log: "Application" */
                if hItemDefault.is_null()
                    && _wcsicmp(
                        LogName as LPCWSTR,
                        SystemLogs[(0) as usize],
                    ) == 0
                {
                    hItemDefault = hItem;
                }

                /* Free the buffer allocated by FormatMessage */
                if !lpDisplayName.is_null() {
                    LocalFree(lpDisplayName as HLOCAL);
                }
            }
            dwIndex = dwIndex.wrapping_add(1);
        }

        HeapFree(GetProcessHeap(), 0, LogName as PVOID);
    }

    /* Quit: */
    RegCloseKey(hEventLogKey);

    /* Select the default event log */
    if !hItemDefault.is_null() {
        // TreeView_Expand(hwndTreeView, hRootNode, TVE_EXPAND);
        TreeView_SelectItem!(hwndTreeView, hItemDefault);
        TreeView_EnsureVisible!(hwndTreeView, hItemDefault);
    }
    InvalidateRect(hwndTreeView, core::ptr::null(), (FALSE) as BOOL);
    SetFocus(hwndTreeView);
}
#[no_mangle]
pub unsafe extern "C" fn FreeLogList() {
    let mut Entry: PLIST_ENTRY;
    let mut EventLog: PEVENTLOG;
    while IsListEmpty(core::ptr::addr_of_mut!(EventLogList)) == false {
        Entry = RemoveHeadList(core::ptr::addr_of_mut!(EventLogList));
        EventLog = CONTAINING_RECORD!(Entry, EVENTLOG, ListEntry) as PEVENTLOG;
        EventLog_Free(EventLog);
    }

    return;
}
#[no_mangle]
pub unsafe extern "C" fn FreeLogFilterList() {
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut EventLogFilter: PEVENTLOGFILTER = core::ptr::null_mut();
    while IsListEmpty(core::ptr::addr_of_mut!(EventLogFilterList)) == false {
        Entry = RemoveHeadList(core::ptr::addr_of_mut!(EventLogFilterList));
        EventLogFilter = CONTAINING_RECORD!(Entry, EVENTLOGFILTER, ListEntry) as PEVENTLOGFILTER;
        EventLogFilter_Free(EventLogFilter);
    }

    ActiveFilter = core::ptr::null_mut();

    return;
}
#[no_mangle]
pub unsafe extern "C" fn InitInstance(hInstance: HINSTANCE) -> BOOL {
    let mut rcClient: RECT = core::mem::zeroed();
    let mut rs: RECT = core::mem::zeroed();
    let mut StatusHeight: LONG = 0;
    let mut hSmall: HIMAGELIST = core::ptr::null_mut();
    let mut lvc: LVCOLUMNW = core::mem::zeroed();
    let mut szTemp: [WCHAR; 256] = [0; 256];
    let mut iColumn: INT;
    static columnItems: [(WORD, WORD); 8] = [
        (90, (IDS_COLUMNTYPE as u16)),
        (70, (IDS_COLUMNDATE as u16)),
        (70, (IDS_COLUMNTIME as u16)),
        (150, (IDS_COLUMNSOURCE as u16)),
        (100, (IDS_COLUMNCATEGORY as u16)),
        (60, (IDS_COLUMNEVENT as u16)),
        (120, (IDS_COLUMNUSER as u16)),
        (100, (IDS_COLUMNCOMPUTER as u16)),
    ];

    /* Create the main window */
    rs = Settings.wpPos.rcNormalPosition;
    hwndMainWindow = CreateWindowW!(
        EVENTVWR_WNDCLASS,
        core::ptr::addr_of_mut!(szTitle) as LPCWSTR,
        (WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN) as DWORD,
        rs.left,
        rs.top,
        (if rs.right != CW_USEDEFAULT && rs.left != CW_USEDEFAULT {
            rs.right - rs.left
        } else {
            CW_USEDEFAULT
        }) as INT,
        (if rs.bottom != CW_USEDEFAULT && rs.top != CW_USEDEFAULT {
            rs.bottom - rs.top
        } else {
            CW_USEDEFAULT
        }) as INT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        hInstance,
        core::ptr::null_mut(),
    );
    if hwndMainWindow.is_null() {
        return (FALSE) as BOOL;
    }

    /* Create the status bar */
    hwndStatus = CreateWindowExW(
        0,
        (STATUSCLASSNAMEW).as_ptr(),
        RUSTCONV_WSTR_96A296D224F2.as_ptr(),
        (WS_CHILD | WS_VISIBLE | CCS_BOTTOM | SBARS_SIZEGRIP) as DWORD,
        0,
        0,
        0,
        0,
        hwndMainWindow,
        100 as usize as HMENU,
        hInstance,
        core::ptr::null_mut(),
    );

    GetClientRect(hwndMainWindow, core::ptr::addr_of_mut!(rcClient));
    GetWindowRect(hwndStatus, core::ptr::addr_of_mut!(rs));
    StatusHeight = rs.bottom - rs.top;

    /* Create a progress bar in the status bar (hidden by default) */
    StatusBar_GetItemRect!(hwndStatus, 0, core::ptr::addr_of_mut!(rs));
    hwndStatusProgress = CreateWindowExW(
        0,
        (PROGRESS_CLASSW).as_ptr(),
        core::ptr::null_mut(),
        (WS_CHILD | PBS_SMOOTH) as DWORD,
        rs.left,
        rs.top,
        rs.right - rs.left,
        rs.bottom - rs.top,
        hwndStatus,
        core::ptr::null_mut(),
        hInstance,
        core::ptr::null_mut(),
    );
    /* Remove its static edge */
    SetWindowLongPtrW(
        hwndStatusProgress,
        GWL_EXSTYLE,
        (GetWindowLongPtrW(hwndStatusProgress, GWL_EXSTYLE) & !(WS_EX_STATICEDGE as LONG_PTR)) as _,
    );
    ProgressBar_SetStep!(hwndStatusProgress, 1);

    /* Initialize the splitter default positions */
    nVSplitPos = Settings.nVSplitPos;
    nHSplitPos = Settings.nHSplitPos;

    /* Create the TreeView */
    hwndTreeView = CreateWindowExW(
        WS_EX_CLIENTEDGE,
        (WC_TREEVIEWW).as_ptr(),
        core::ptr::null_mut(),
        (WS_CHILD | WS_VISIBLE | TVS_HASLINES | TVS_HASBUTTONS | TVS_LINESATROOT | TVS_EDITLABELS | TVS_SHOWSELALWAYS) as DWORD,
        0,
        0,
        nVSplitPos - SPLIT_WIDTH / 2,
        (rcClient.bottom - rcClient.top) - StatusHeight,
        hwndMainWindow,
        core::ptr::null_mut(),
        hInstance,
        core::ptr::null_mut(),
    );

    /* Create the ImageList */
    hSmall = ImageList_Create(
        GetSystemMetrics((SM_CXSMICON as i32)),
        GetSystemMetrics((SM_CYSMICON as i32)),
        ILC_COLOR32 | ILC_MASK, // ILC_COLOR24
        1,
        1,
    );

    /* Add event type icons to the ImageList: closed/opened folder, event log (normal/viewed) */
    ImageList_AddIcon!(hSmall, LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_CLOSED_CATEGORY)));
    ImageList_AddIcon!(hSmall, LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_OPENED_CATEGORY)));
    ImageList_AddIcon!(hSmall, LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_EVENTLOG)));
    ImageList_AddIcon!(hSmall, LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_EVENTVWR)));

    /* Assign the ImageList to the Tree View */
    TreeView_SetImageList!(hwndTreeView, hSmall, TVSIL_NORMAL);

    /* Add the event logs nodes */
    // "System Logs"
    LoadStringW(hInstance, IDS_EVENTLOG_SYSTEM, core::ptr::addr_of_mut!(szTemp) as LPWSTR, ARRAYSIZE!(szTemp) as INT);
    htiSystemLogs = TreeViewAddItem(
        hwndTreeView,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(szTemp) as LPWSTR,
        0,
        1,
        core::ptr::null_mut::<core::ffi::c_void>() as LPARAM,
    );
    // "Application Logs"
    LoadStringW(hInstance, IDS_EVENTLOG_APP, core::ptr::addr_of_mut!(szTemp) as LPWSTR, ARRAYSIZE!(szTemp) as INT);
    htiAppLogs = TreeViewAddItem(
        hwndTreeView,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(szTemp) as LPWSTR,
        0,
        1,
        core::ptr::null_mut::<core::ffi::c_void>() as LPARAM,
    );
    // "User Logs"
    LoadStringW(hInstance, IDS_EVENTLOG_USER, core::ptr::addr_of_mut!(szTemp) as LPWSTR, ARRAYSIZE!(szTemp) as INT);
    htiUserLogs = TreeViewAddItem(
        hwndTreeView,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(szTemp) as LPWSTR,
        0,
        1,
        core::ptr::null_mut::<core::ffi::c_void>() as LPARAM,
    );

    /* Create the Event details pane (optional) */
    hwndEventDetails = CreateEventDetailsCtrl(hInst, hwndMainWindow, core::ptr::null_mut::<core::ffi::c_void>() as LPARAM);
    if !hwndEventDetails.is_null() {
        SetWindowLongPtrW(
            hwndEventDetails,
            GWL_EXSTYLE,
            (GetWindowLongPtrW(hwndEventDetails, GWL_EXSTYLE) | (WS_EX_CLIENTEDGE as LONG_PTR)) as _,
        );
        SetWindowPos(
            hwndEventDetails,
            core::ptr::null_mut(),
            nVSplitPos + SPLIT_WIDTH / 2,
            nHSplitPos + SPLIT_WIDTH / 2,
            (rcClient.right - rcClient.left) - nVSplitPos - SPLIT_WIDTH / 2,
            (rcClient.bottom - rcClient.top) - nHSplitPos - SPLIT_WIDTH / 2 - StatusHeight,
            SWP_NOZORDER
                | SWP_NOACTIVATE
                | (if Settings.bShowDetailsPane != 0 {
                    SWP_SHOWWINDOW
                } else {
                    SWP_HIDEWINDOW
                }),
        );
    }

    /* Create the ListView */
    hwndListView = CreateWindowExW(
        WS_EX_CLIENTEDGE,
        (WC_LISTVIEWW).as_ptr(),
        core::ptr::null_mut(),
        WS_CHILD | WS_VISIBLE | LVS_SHOWSELALWAYS | LVS_REPORT,
        nVSplitPos + SPLIT_WIDTH / 2,
        0,
        (rcClient.right - rcClient.left) - nVSplitPos - SPLIT_WIDTH / 2,
        if !hwndEventDetails.is_null() && Settings.bShowDetailsPane != 0 {
            nHSplitPos - SPLIT_WIDTH / 2
        } else {
            (rcClient.bottom - rcClient.top) - StatusHeight
        },
        hwndMainWindow,
        core::ptr::null_mut(),
        hInstance,
        core::ptr::null_mut(),
    );

    /* Add the extended ListView styles */
    ListView_SetExtendedListViewStyle!(
        hwndListView,
        LVS_EX_HEADERDRAGDROP
            | LVS_EX_FULLROWSELECT
            | LVS_EX_LABELTIP
            | (if Settings.bShowGrid != 0 {
                LVS_EX_GRIDLINES
            } else {
                0
            }),
    );

    /* Create the ImageList */
    hSmall = ImageList_Create(
        GetSystemMetrics((SM_CXSMICON as i32)),
        GetSystemMetrics((SM_CYSMICON as i32)),
        ILC_COLOR32 | ILC_MASK, // ILC_COLOR24
        1,
        1,
    );

    /* Add event type icons to the ImageList */
    ImageList_AddIcon!(hSmall, LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_INFORMATIONICON)));
    ImageList_AddIcon!(hSmall, LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_WARNINGICON)));
    ImageList_AddIcon!(hSmall, LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_ERRORICON)));
    ImageList_AddIcon!(hSmall, LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_AUDITSUCCESSICON)));
    ImageList_AddIcon!(hSmall, LoadIconW(hInstance, MAKEINTRESOURCEW!(IDI_AUDITFAILUREICON)));

    /* Assign the ImageList to the List View */
    ListView_SetImageList!(hwndListView, hSmall, LVSIL_SMALL);

    /* Now set up the listview with its columns */
    lvc.mask = (LVCF_TEXT | LVCF_WIDTH) as UINT;
    lvc.pszText = core::ptr::addr_of_mut!(szTemp) as LPWSTR;
    iColumn = 0;
    while iColumn < ARRAYSIZE!(columnItems) as INT {
        lvc.cx = columnItems[iColumn as usize].0 as INT;
        LoadStringW(
            hInstance,
            (columnItems[iColumn as usize].1) as UINT,
            core::ptr::addr_of_mut!(szTemp) as LPWSTR,
            ARRAYSIZE!(szTemp) as INT,
        );
        ListView_InsertColumn!(hwndListView, iColumn, core::ptr::addr_of_mut!(lvc));
        iColumn += 1;
    }

    /* Initialize the save Dialog */
    ZeroMemory!(
        core::ptr::addr_of_mut!(sfn) as *mut core::ffi::c_void,
        core::mem::size_of::<OPENFILENAMEW>(),
    );
    ZeroMemory!(
        core::ptr::addr_of_mut!(szSaveFilter) as *mut core::ffi::c_void,
        core::mem::size_of::<[WCHAR; 255]>(),
    );

    LoadStringW(
        hInst,
        IDS_SAVE_FILTER,
        core::ptr::addr_of_mut!(szSaveFilter) as LPWSTR,
        ARRAYSIZE!(szSaveFilter) as INT,
    );

    sfn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
    sfn.hwndOwner = hwndMainWindow;
    sfn.hInstance = hInstance;
    sfn.lpstrFilter = core::ptr::addr_of_mut!(szSaveFilter) as LPCWSTR;
    sfn.lpstrInitialDir = core::ptr::null();
    sfn.Flags = (OFN_EXPLORER | OFN_HIDEREADONLY | OFN_SHAREAWARE) as DWORD;
    sfn.lpstrDefExt = RUSTCONV_WSTR_FAC6BE05036B.as_ptr();

    ShowWindow(hwndMainWindow, Settings.wpPos.showCmd as INT);
    UpdateWindow(hwndMainWindow);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ResizeWnd(cx: INT, cy: INT) {
    let mut rs: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut StatusHeight: LONG;
    let mut dwExStyle: LONG_PTR;
    let mut hdwp: HDWP;

    /* Resize the status bar -- now done in WM_SIZE */
    // SendMessageW(hwndStatus, WM_SIZE, 0, 0);
    GetWindowRect(hwndStatus, core::ptr::addr_of_mut!(rs));
    StatusHeight = rs.bottom - rs.top;

    /*
     * Move the progress bar -- Take into account for extra size due to the static edge
     * (AdjustWindowRectEx() does not seem to work for the progress bar).
     */
    StatusBar_GetItemRect!(hwndStatus, 0, core::ptr::addr_of_mut!(rs));
    dwExStyle = GetWindowLongPtrW(hwndStatusProgress, GWL_EXSTYLE);
    SetWindowLongPtrW(hwndStatusProgress, GWL_EXSTYLE, (dwExStyle | (WS_EX_STATICEDGE as LONG_PTR)) as _);
    MoveWindow(
        hwndStatusProgress,
        rs.left,
        rs.top,
        rs.right - rs.left,
        rs.bottom - rs.top,
        if IsWindowVisible(hwndStatusProgress) != 0 { (TRUE as i32) } else { (FALSE as i32) },
    );
    SetWindowLongPtrW(hwndStatusProgress, GWL_EXSTYLE, (dwExStyle) as _);

    /*
     * TODO: Adjust the splitter positions:
     * - Vertical splitter (1)  : fixed position from the left window side.
     * - Horizontal splitter (2): fixed position from the bottom window side.
     */
    nVSplitPos = min!(max!(nVSplitPos, SPLIT_WIDTH / 2), cx - SPLIT_WIDTH / 2); // OK
    nHSplitPos = min!(max!(nHSplitPos, SPLIT_WIDTH / 2), cy - SPLIT_WIDTH / 2 - StatusHeight); // FIXME!

    hdwp = BeginDeferWindowPos(3);

    if !hdwp.is_null() {
        hdwp = DeferWindowPos(
            hdwp,
            hwndTreeView,
            HWND_TOP,
            0,
            0,
            nVSplitPos - SPLIT_WIDTH / 2,
            cy - StatusHeight,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }

    if !hdwp.is_null() {
        hdwp = DeferWindowPos(
            hdwp,
            hwndListView,
            HWND_TOP,
            nVSplitPos + SPLIT_WIDTH / 2,
            0,
            cx - nVSplitPos - SPLIT_WIDTH / 2,
            if !hwndEventDetails.is_null() && Settings.bShowDetailsPane != 0 {
                nHSplitPos - SPLIT_WIDTH / 2
            } else {
                cy - StatusHeight
            },
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }

    if !hwndEventDetails.is_null() && Settings.bShowDetailsPane != 0 && !hdwp.is_null() {
        hdwp = DeferWindowPos(
            hdwp,
            hwndEventDetails,
            HWND_TOP,
            nVSplitPos + SPLIT_WIDTH / 2,
            nHSplitPos + SPLIT_WIDTH / 2,
            cx - nVSplitPos - SPLIT_WIDTH / 2,
            cy - nHSplitPos - SPLIT_WIDTH / 2 - StatusHeight,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }

    if !hdwp.is_null() {
        EndDeferWindowPos(hdwp);
    }
}

#[no_mangle]
pub unsafe extern "system" fn WndProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut rect: RECT = core::mem::zeroed();
    let mut at_default: bool = false;
    'sw: {
        match uMsg {
            WM_CREATE => {
                hMainMenu = GetMenu(hWnd);
            }

            WM_DESTROY => {
                GetWindowPlacement(hwndMainWindow, core::ptr::addr_of_mut!(Settings.wpPos));
                PostQuitMessage(0);
            }

            WM_NOTIFY => {
                let mut hdr: LPNMHDR = lParam as LPNMHDR;

                if (*hdr).hwndFrom == hwndListView {
                    'hs1: {
                        match ((*hdr).code) as i32 {
                            LVN_ITEMCHANGED => {
                                let mut pnmv: LPNMLISTVIEW = lParam as LPNMLISTVIEW;

                                if ((*pnmv).uChanged & LVIF_STATE) != 0 && /* The state has changed */
                                    ((*pnmv).uNewState & LVIS_SELECTED) != 0 /* The item has been (de)selected */
                                {
                                    if hwndEventDetails.is_null() {
                                        break 'hs1;
                                    }

                                    /* Verify the index of selected item */
                                    if (*pnmv).iItem == -(1) {
                                        MessageBoxW(
                                            hWnd,
                                            RUSTCONV_WSTR_D01499AAA203.as_ptr(),
                                            core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR,
                                            MB_OK | MB_ICONERROR,
                                        );
                                        break 'hs1;
                                    }
                                    SendMessageW(
                                        hwndEventDetails,
                                        EVT_DISPLAY,
                                        (TRUE) as WPARAM,
                                        (*pnmv).iItem as LPARAM,
                                    );
                                }
                            }

                            LVN_ITEMACTIVATE => {
                                /* Get the index of the single focused selected item */
                                let mut lpnmitem: LPNMITEMACTIVATE = lParam as LPNMITEMACTIVATE;
                                let mut iItem: INT = (*lpnmitem).iItem as INT;
                                if iItem != -(1) {
                                    SendMessageW(hWnd, WM_COMMAND, (IDM_EVENT_DETAILS) as WPARAM, iItem as LPARAM);
                                }
                            }

                            _ => {}
                        }
                    }
                } else if (*hdr).hwndFrom == hwndTreeView {
                    'hs2: {
                        match (*hdr).code {
                            TVN_BEGINLABELEDIT => {
                                let mut hItem: HTREEITEM = (*(lParam as LPNMTVDISPINFO)).item.hItem;

                                /* Disable label editing for root nodes */
                                return ((hItem == htiSystemLogs) ||
                                        (hItem == htiAppLogs) ||
                                        (hItem == htiUserLogs)) as LRESULT;
                            }

                            TVN_ENDLABELEDIT => {
                                let mut item: TVITEMW = (*(lParam as LPNMTVDISPINFO)).item;
                                let mut hItem: HTREEITEM = item.hItem;

                                /* Disable label editing for root nodes */
                                if (hItem == htiSystemLogs) ||
                                    (hItem == htiAppLogs) ||
                                    (hItem == htiUserLogs)
                                {
                                    return (FALSE) as LRESULT;
                                }

                                if !item.pszText.is_null() {
                                    let mut pszText: LPWSTR = item.pszText;

                                    /* Trim leading whitespace */
                                    while *pszText != 0 && iswspace(*pszText) != 0 {
                                        pszText = pszText.add(1);
                                    }

                                    if *pszText == 0 {
                                        return (FALSE) as LRESULT;
                                    }

                                    return (TRUE) as LRESULT;
                                } else {
                                    return (FALSE) as LRESULT;
                                }
                            }

                            TVN_SELCHANGED => {
                                let mut EventLogFilter: PEVENTLOGFILTER =
                                    (*(lParam as LPNMTREEVIEW)).itemNew.lParam as usize as PEVENTLOGFILTER;

                                // FIXME: It might be nice to reference here the filter,
                                // so that we don't have to reference/dereference it many times
                                // in the other functions???

                                // FIXME: This is a hack!!
                                if !hwndEventDetails.is_null() && !EventLogFilter.is_null() {
                                    SendMessageW(hwndEventDetails, EVT_SETFILTER, 0, EventLogFilter as LPARAM);
                                }

                                if !EventLogFilter.is_null() {
                                    /*
                                     * If we have selected a filter, enable the menu commands;
                                     * they will possibly be updated after events enumeration.
                                     */
                                    EnableMenuItem(hMainMenu, IDM_SAVE_EVENTLOG, MF_BYCOMMAND | MF_ENABLED);
                                    EnableMenuItem(hMainMenu, IDM_CLOSE_EVENTLOG, MF_BYCOMMAND | MF_ENABLED);
                                    EnableMenuItem(hMainMenu, IDM_CLEAR_EVENTS, MF_BYCOMMAND | MF_ENABLED);
                                    EnableMenuItem(hMainMenu, IDM_RENAME_EVENTLOG, MF_BYCOMMAND | MF_ENABLED);
                                    EnableMenuItem(hMainMenu, IDM_EVENTLOG_SETTINGS, MF_BYCOMMAND | MF_ENABLED);
                                } else {
                                    EnableMenuItem(hMainMenu, IDM_SAVE_EVENTLOG, MF_BYCOMMAND | MF_GRAYED);
                                    EnableMenuItem(hMainMenu, IDM_CLOSE_EVENTLOG, MF_BYCOMMAND | MF_GRAYED);
                                    EnableMenuItem(hMainMenu, IDM_CLEAR_EVENTS, MF_BYCOMMAND | MF_GRAYED);
                                    EnableMenuItem(hMainMenu, IDM_RENAME_EVENTLOG, MF_BYCOMMAND | MF_GRAYED);
                                    EnableMenuItem(hMainMenu, IDM_EVENTLOG_SETTINGS, MF_BYCOMMAND | MF_GRAYED);
                                }

                                /*
                                 * The enumeration thread that is triggered by EnumEvents
                                 * will set a new value for the 'ActiveFilter'.
                                 */
                                if !EventLogFilter.is_null() {
                                    EnumEvents(EventLogFilter);
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }

            WM_COMMAND => {
                /* Parse the menu selections */
                'cs: {
                    match (LOWORD!(wParam)) as u32 {
                        IDM_OPEN_EVENTLOG => {
                            OpenUserEventLog();
                        }

                        IDM_SAVE_EVENTLOG => {
                            SaveEventLog(GetSelectedFilter(core::ptr::null_mut()));
                        }

                        IDM_CLOSE_EVENTLOG => {
                            let mut hti: HTREEITEM = core::ptr::null_mut();
                            let mut EventLogFilter: PEVENTLOGFILTER = GetSelectedFilter(core::ptr::addr_of_mut!(hti));
                            CloseUserEventLog(EventLogFilter, hti);
                        }

                        IDM_CLEAR_EVENTS => {
                            let mut EventLogFilter: PEVENTLOGFILTER = GetSelectedFilter(core::ptr::null_mut());
                            if !EventLogFilter.is_null() && ClearEvents(EventLogFilter) != 0 {
                                Refresh(EventLogFilter);
                            }
                        }

                        IDM_RENAME_EVENTLOG => {
                            if GetFocus() == hwndTreeView {
                                TreeView_EditLabel!(hwndTreeView, TreeView_GetSelection!(hwndTreeView));
                            }
                        }

                        IDM_EVENTLOG_SETTINGS => {
                            let mut EventLogFilter: PEVENTLOGFILTER = GetSelectedFilter(core::ptr::null_mut());
                            // TODO: Check the returned value?
                            if !EventLogFilter.is_null() {
                                EventLogProperties(hInst, hWnd, EventLogFilter);
                            }
                        }

                        IDM_LIST_NEWEST | IDM_LIST_OLDEST => {
                            let mut bNewest: BOOL = (LOWORD!(wParam) == (IDM_LIST_NEWEST as u16)) as BOOL;
                            CheckMenuRadioItem(hMainMenu, IDM_LIST_NEWEST, IDM_LIST_OLDEST, LOWORD!(wParam) as UINT, MF_BYCOMMAND);

                            if bNewest != Settings.bNewestEventsFirst {
                                Settings.bNewestEventsFirst = bNewest;
                                Refresh(GetSelectedFilter(core::ptr::null_mut()));
                            }
                        }

                        IDM_EVENT_DETAILS => {
                            let mut iItem: INT;
                            let mut EventLogFilter: PEVENTLOGFILTER;

                            /* Get the index of the single focused selected item */
                            iItem = ListView_GetNextItem!(hwndListView, -(1), LVNI_FOCUSED | LVNI_SELECTED);
                            if iItem == -(1) {
                                /*
                                // FIXME: Reenable this check once menu items are
                                // correctly disabled when no event is selected, etc.
                                MessageBoxW(hWnd,
                                            L"No selected items!",
                                            szTitle,
                                            MB_OK | MB_ICONERROR);
                                **/
                                break 'cs;
                            }

                            EventLogFilter = GetSelectedFilter(core::ptr::null_mut());
                            if !EventLogFilter.is_null() {
                                let mut DetailInfo: EVENTDETAIL_INFO = EVENTDETAIL_INFO {
                                    EventLogFilter: EventLogFilter,
                                    iEventItem: iItem,
                                };

                                EventLogFilter_AddRef(EventLogFilter);
                                DialogBoxParamW(
                                    hInst,
                                    MAKEINTRESOURCEW!(IDD_EVENTDETAILS_DLG) as LPCWSTR,
                                    hWnd,
                                    Some(EventDetails),
                                    core::ptr::addr_of_mut!(DetailInfo) as LPARAM,
                                );
                                EventLogFilter_Release(EventLogFilter);
                            }
                        }

                        IDM_REFRESH => {
                            Refresh(GetSelectedFilter(core::ptr::null_mut()));
                        }

                        IDM_EVENT_DETAILS_VIEW => {
                            Settings.bShowDetailsPane = (Settings.bShowDetailsPane == 0) as BOOL;
                            CheckMenuItem(
                                hMainMenu,
                                IDM_EVENT_DETAILS_VIEW,
                                MF_BYCOMMAND | (if Settings.bShowDetailsPane != 0 { MF_CHECKED } else { MF_UNCHECKED }),
                            );

                            GetClientRect(hWnd, core::ptr::addr_of_mut!(rect));
                            if Settings.bShowDetailsPane != 0 {
                                ResizeWnd(rect.right - rect.left, rect.bottom - rect.top);
                                ShowWindow(hwndEventDetails, (SW_SHOW as i32));
                            } else {
                                ShowWindow(hwndEventDetails, (SW_HIDE as i32));
                                ResizeWnd(rect.right - rect.left, rect.bottom - rect.top);
                            }
                        }

                        IDM_LIST_GRID_LINES => {
                            Settings.bShowGrid = (Settings.bShowGrid == 0) as BOOL;
                            CheckMenuItem(
                                hMainMenu,
                                IDM_LIST_GRID_LINES,
                                MF_BYCOMMAND | (if Settings.bShowGrid != 0 { MF_CHECKED } else { MF_UNCHECKED }),
                            );

                            ListView_SetExtendedListViewStyleEx!(
                                hwndListView,
                                LVS_EX_GRIDLINES,
                                if Settings.bShowGrid != 0 { LVS_EX_GRIDLINES } else { 0 },
                            );
                        }

                        IDM_SAVE_SETTINGS => {
                            Settings.bSaveSettings = (Settings.bSaveSettings == 0) as BOOL;
                            CheckMenuItem(
                                hMainMenu,
                                IDM_SAVE_SETTINGS,
                                MF_BYCOMMAND | (if Settings.bSaveSettings != 0 { MF_CHECKED } else { MF_UNCHECKED }),
                            );
                        }

                        IDM_ABOUT => {
                            let mut szCopyright: [WCHAR; MAX_LOADSTRING] = [0; MAX_LOADSTRING];

                            LoadStringW(
                                hInst,
                                IDS_COPYRIGHT,
                                core::ptr::addr_of_mut!(szCopyright) as *mut WCHAR as LPWSTR,
                                (ARRAYSIZE!(szCopyright)) as i32,
                            );
                            ShellAboutW(
                                hWnd,
                                core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR,
                                core::ptr::addr_of_mut!(szCopyright) as *mut WCHAR as LPCWSTR,
                                LoadIconW(hInst, MAKEINTRESOURCEW!(IDI_EVENTVWR) as LPCWSTR),
                            );
                        }

                        IDM_HELP => {
                            MessageBoxW(
                                hWnd,
                                RUSTCONV_WSTR_08C4A5A5D53A.as_ptr(),
                                core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR,
                                MB_OK | MB_ICONINFORMATION,
                            );
                        }

                        IDM_EXIT => {
                            DestroyWindow(hWnd);
                        }

                        _ => {
                            return DefWindowProcW(hWnd, uMsg, wParam, lParam);
                        }
                    }
                }
            }

            WM_INITMENU => {
                if (wParam as HMENU) != hMainMenu {
                    break 'sw;
                }

                CheckMenuRadioItem(
                    hMainMenu,
                    IDM_LIST_NEWEST,
                    IDM_LIST_OLDEST,
                    if Settings.bNewestEventsFirst != 0 { IDM_LIST_NEWEST } else { IDM_LIST_OLDEST },
                    MF_BYCOMMAND,
                );

                if hwndEventDetails.is_null() {
                    EnableMenuItem(hMainMenu, IDM_EVENT_DETAILS_VIEW, MF_BYCOMMAND | MF_GRAYED);
                }
                CheckMenuItem(
                    hMainMenu,
                    IDM_EVENT_DETAILS_VIEW,
                    MF_BYCOMMAND | (if Settings.bShowDetailsPane != 0 { MF_CHECKED } else { MF_UNCHECKED }),
                );

                CheckMenuItem(
                    hMainMenu,
                    IDM_LIST_GRID_LINES,
                    MF_BYCOMMAND | (if Settings.bShowGrid != 0 { MF_CHECKED } else { MF_UNCHECKED }),
                );

                CheckMenuItem(
                    hMainMenu,
                    IDM_SAVE_SETTINGS,
                    MF_BYCOMMAND | (if Settings.bSaveSettings != 0 { MF_CHECKED } else { MF_UNCHECKED }),
                );
            }

            WM_CONTEXTMENU => {
                let mut rc: RECT = core::mem::zeroed();
                let mut hItem: HTREEITEM = core::ptr::null_mut();
                let mut hInfo: TVHITTESTINFO = core::mem::zeroed();

                let mut xPos: INT = GET_X_LPARAM!(lParam);
                let mut yPos: INT = GET_Y_LPARAM!(lParam);

                GetWindowRect(hwndTreeView, core::ptr::addr_of_mut!(rc));
                hInfo.pt.x = xPos - rc.left;
                hInfo.pt.y = yPos - rc.top;

                hItem = TreeView_HitTest!(hwndTreeView, core::ptr::addr_of_mut!(hInfo));
                if !hItem.is_null() {
                    TreeView_SelectItem!(hwndTreeView, hItem);

                    if !TreeView_GetParent!(hwndTreeView, hItem).is_null() {
                        let mut hCtxMenu: HMENU = GetSubMenu(
                            LoadMenuW(hInst, MAKEINTRESOURCEW!(IDM_EVENTWR_CTX) as LPCWSTR),
                            0,
                        );

                        let mut dwCmdID: DWORD = (TrackPopupMenuEx(
                            hCtxMenu,
                            TPM_LEFTALIGN | TPM_TOPALIGN | TPM_NONOTIFY | TPM_RETURNCMD,
                            xPos,
                            yPos,
                            hWnd,
                            core::ptr::null_mut(),
                        )) as u32;
                        SendMessageW(hWnd, WM_COMMAND, dwCmdID as WPARAM, hwndTreeView as LPARAM);
                    }
                }
            }

            WM_SETCURSOR => {
                let mut pt: POINT = core::mem::zeroed();

                if LOWORD!(lParam) != (HTCLIENT as u16) {
                    at_default = true;
                    break 'sw;
                }

                GetCursorPos(core::ptr::addr_of_mut!(pt));
                ScreenToClient(hWnd, core::ptr::addr_of_mut!(pt));

                /* Set the cursor for the vertical splitter */
                if pt.x >= nVSplitPos - (SPLIT_WIDTH / 2)
                    && pt.x < nVSplitPos + (SPLIT_WIDTH / 2) + 1
                {
                    let mut rs: RECT = core::mem::zeroed();
                    GetClientRect(hWnd, core::ptr::addr_of_mut!(rect));
                    GetWindowRect(hwndStatus, core::ptr::addr_of_mut!(rs));
                    if pt.y >= rect.top && pt.y < rect.bottom - (rs.bottom - rs.top) {
                        SetCursor(LoadCursorW(core::ptr::null_mut(), IDC_SIZEWE as LPCWSTR));
                        return (TRUE) as LRESULT;
                    }
                }
                /* Set the cursor for the horizontal splitter, if the Event details pane is displayed */
                else if !hwndEventDetails.is_null() && Settings.bShowDetailsPane != 0 &&
                    (pt.y >= nHSplitPos - (SPLIT_WIDTH / 2)
                        && pt.y < nHSplitPos + (SPLIT_WIDTH / 2) + 1)
                {
                    // RECT rs;
                    GetClientRect(hWnd, core::ptr::addr_of_mut!(rect));
                    // GetWindowRect(hwndStatus, &rs);
                    if pt.x >= nVSplitPos + (SPLIT_WIDTH / 2) + 1 /* rect.left + (rs.bottom - rs.top) */ &&
                        pt.x < rect.right
                    {
                        SetCursor(LoadCursorW(core::ptr::null_mut(), IDC_SIZENS as LPCWSTR));
                        return (TRUE) as LRESULT;
                    }
                }
                at_default = true;
                break 'sw;
            }

            WM_LBUTTONDOWN => {
                let mut x: INT = GET_X_LPARAM!(lParam);
                let mut y: INT = GET_Y_LPARAM!(lParam);

                /* Reset the splitter state */
                bSplit = 0;

                /* Capture the cursor for the vertical splitter */
                if x >= nVSplitPos - (SPLIT_WIDTH / 2)
                    && x < nVSplitPos + (SPLIT_WIDTH / 2) + 1
                {
                    bSplit = 1;
                    SetCapture(hWnd);
                }
                /* Capture the cursor for the horizontal splitter, if the Event details pane is displayed */
                else if !hwndEventDetails.is_null() && Settings.bShowDetailsPane != 0 &&
                    (y >= nHSplitPos - (SPLIT_WIDTH / 2)
                        && y < nHSplitPos + (SPLIT_WIDTH / 2) + 1)
                {
                    bSplit = 2;
                    SetCapture(hWnd);
                }
            }

            WM_LBUTTONUP | WM_RBUTTONDOWN => {
                if GetCapture() != hWnd {
                    break 'sw;
                }

                /* Adjust the correct splitter position */
                if bSplit == 1 {
                    nVSplitPos = GET_X_LPARAM!(lParam);
                } else if bSplit == 2 {
                    nHSplitPos = GET_Y_LPARAM!(lParam);
                }

                /* If we are splitting, resize the windows */
                if bSplit != 0 {
                    GetClientRect(hWnd, core::ptr::addr_of_mut!(rect));
                    ResizeWnd(rect.right - rect.left, rect.bottom - rect.top);
                }

                /* Reset the splitter state */
                bSplit = 0;

                ReleaseCapture();
            }

            WM_MOUSEMOVE => {
                if GetCapture() != hWnd {
                    break 'sw;
                }

                /* Move the correct splitter */
                if bSplit == 1 {
                    let mut x: INT = GET_X_LPARAM!(lParam);

                    GetClientRect(hWnd, core::ptr::addr_of_mut!(rect));

                    x = min!(max!(x, SPLIT_WIDTH / 2), rect.right - rect.left - (SPLIT_WIDTH / 2));
                    if nVSplitPos != x {
                        nVSplitPos = x;
                        ResizeWnd(rect.right - rect.left, rect.bottom - rect.top);
                    }
                } else if bSplit == 2 {
                    let mut rs: RECT = core::mem::zeroed();
                    let mut y: INT = GET_Y_LPARAM!(lParam);

                    GetClientRect(hWnd, core::ptr::addr_of_mut!(rect));
                    GetWindowRect(hwndStatus, core::ptr::addr_of_mut!(rs));

                    y = min!(
                        max!(y, SPLIT_WIDTH / 2),
                        rect.bottom - rect.top - (SPLIT_WIDTH / 2) - (rs.bottom - rs.top),
                    );
                    if nHSplitPos != y {
                        nHSplitPos = y;
                        ResizeWnd(rect.right - rect.left, rect.bottom - rect.top);
                    }
                }
            }

            WM_SIZE => {
                if wParam != (SIZE_MINIMIZED as _) {
                    SendMessageW(hwndStatus, WM_SIZE, 0, 0);
                    ResizeWnd((LOWORD!(lParam)) as INT, (HIWORD!(lParam)) as INT);
                    break 'sw;
                }
                /* Fall through the default case */
                at_default = true;
            }

            _ => {
                at_default = true;
            }
        }
    }

    if at_default {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }

    return 0;
}

unsafe extern "C" fn InitPropertiesDlg(hDlg: HWND, EventLog: PEVENTLOG) {
    let mut lpLogName: LPWSTR = (*EventLog).LogName;

    let mut Result: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwMaxSize: DWORD = 0;
    let mut dwRetention: DWORD = 0;
    let mut Success: BOOL = 0;
    let mut FileInfo: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut FileSize: ULARGE_INTEGER = core::mem::zeroed();
    let mut wszBuf: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szTemp: [WCHAR; MAX_LOADSTRING] = [0; MAX_LOADSTRING];
    let mut FileName: LPWSTR = core::ptr::null_mut();

    let mut hLogKey: HKEY = core::ptr::null_mut();
    let mut KeyPath: PWSTR = core::ptr::null_mut();
    let mut cbKeyPath: SIZE_T = 0;
    let mut cbData: DWORD = 0;

    'quit: {
        if (*EventLog).Permanent != 0 {
            cbKeyPath = (wcslen(EVENTLOG_BASE_KEY) as SIZE_T)
                .wrapping_add(wcslen(lpLogName) as SIZE_T)
                .wrapping_add(1 as SIZE_T)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T);
            KeyPath = HeapAlloc(
                GetProcessHeap(),
                0,
                cbKeyPath,
            ) as PWSTR;
            if KeyPath.is_null() {
                ShowWin32Error(ERROR_NOT_ENOUGH_MEMORY);
                break 'quit;
            }

            StringCbCopyW(KeyPath, (cbKeyPath) as _, EVENTLOG_BASE_KEY);
            StringCbCatW(KeyPath, (cbKeyPath) as _, lpLogName);

            Result = RegOpenKeyExW(
                hkMachine,
                KeyPath as LPCWSTR,
                0,
                KEY_QUERY_VALUE,
                core::ptr::addr_of_mut!(hLogKey),
            ) as DWORD;
            HeapFree(GetProcessHeap(), 0, KeyPath as PVOID);
            if Result != ERROR_SUCCESS {
                ShowWin32Error(Result);
                break 'quit;
            }

            cbData = core::mem::size_of::<DWORD>() as DWORD;
            Result = RegQueryValueExW(
                hLogKey,
                RUSTCONV_WSTR_0AF4B1084E0C.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwType),
                core::ptr::addr_of_mut!(dwMaxSize) as LPBYTE,
                core::ptr::addr_of_mut!(cbData),
            ) as DWORD;
            if (Result != ERROR_SUCCESS) || (dwType != REG_DWORD) {
                // dwMaxSize = 512 * 1024; /* 512 kBytes */
                /* Workstation: 512 KB; Server: 16384 KB */
                dwMaxSize = (16384 as DWORD) * (1024 as DWORD);
            }
            /* Convert in KB */
            dwMaxSize /= 1024;

            cbData = core::mem::size_of::<DWORD>() as DWORD;
            Result = RegQueryValueExW(
                hLogKey,
                RUSTCONV_WSTR_2F23369E2DF4.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwType),
                core::ptr::addr_of_mut!(dwRetention) as LPBYTE,
                core::ptr::addr_of_mut!(cbData),
            ) as DWORD;
            if (Result != ERROR_SUCCESS) || (dwType != REG_DWORD) {
                /* By default, it is 604800 (secs) == 7 days. On Server, the retention is zeroed out. */
                dwRetention = 0;
            }
            /* Convert in days, rounded up */
            if dwRetention != INFINITE {
                dwRetention = dwRetention
                    .wrapping_add((24 as DWORD).wrapping_mul(3600 as DWORD))
                    .wrapping_sub(1 as DWORD)
                    / ((24 as DWORD).wrapping_mul(3600 as DWORD));
            }

            RegCloseKey(hLogKey);
        }
    }

    // Quit:
    SetDlgItemTextW(hDlg, (IDC_DISPLAYNAME as i32), lpLogName); // FIXME!
    SetDlgItemTextW(hDlg, (IDC_LOGNAME as i32), lpLogName);

    FileName = (*EventLog).FileName;
    if !FileName.is_null() && *FileName != 0 {
        /* Expand the file name. If the log file is on a remote computer, retrieve the network share form of the file name. */
        GetExpandedFilePathName(
            (*EventLog).ComputerName,
            FileName as PCWSTR,
            core::ptr::addr_of_mut!(wszBuf) as PWSTR,
            _countof!(wszBuf) as SIZE_T,
        );
        FileName = core::ptr::addr_of_mut!(wszBuf) as LPWSTR;
    } else {
        FileName = RUSTCONV_WSTR_96A296D224F2.as_ptr() as LPWSTR;
    }
    SetDlgItemTextW(hDlg, (IDC_LOGFILE as i32), FileName);

    if !FileName.is_null() && *FileName != 0 {
        /*
         * The general problem here (and in the shell as well) is that
         * GetFileAttributesEx fails for files that are opened without
         * shared access. To retrieve file information for those we need
         * to use something else: FindFirstFile, on the full file name.
         */
        Success = GetFileAttributesExW(
            FileName as LPCWSTR,
            GetFileExInfoStandard,
            core::ptr::addr_of_mut!(FileInfo) as LPWIN32_FILE_ATTRIBUTE_DATA as PVOID,
        );
        if Success == 0 {
            let mut hFind: HANDLE = FindFirstFileW(FileName as LPCWSTR, core::ptr::addr_of_mut!(FileInfo));
            Success = (hFind != INVALID_HANDLE_VALUE) as BOOL;
            if Success != 0 {
                FindClose(hFind);
            }
        }
    } else {
        Success = FALSE as BOOL;
    }

    /* Starting there, FileName becomes invalid because we are reusing wszBuf */

    if Success != 0 {
        FileSize.u.LowPart = FileInfo.nFileSizeLow;
        FileSize.u.HighPart = FileInfo.nFileSizeHigh;
        if !FormatFileSizeWithBytes(
            core::ptr::addr_of_mut!(FileSize),
            addr_of_mut!(wszBuf) as LPWSTR,
            ARRAYSIZE!(wszBuf) as UINT,
        )
        .is_null()
        {
            SetDlgItemTextW(hDlg, (IDC_SIZE_LABEL as i32), addr_of_mut!(wszBuf) as LPCWSTR);
        }

        LoadStringW(
            hInst,
            IDS_NOT_AVAILABLE,
            addr_of_mut!(szTemp) as LPWSTR,
            ARRAYSIZE!(szTemp) as i32,
        );

        if GetFileTimeString(
            core::ptr::addr_of_mut!(FileInfo.ftCreationTime),
            addr_of_mut!(wszBuf) as LPWSTR,
            ARRAYSIZE!(wszBuf) as UINT,
        ) != 0
        {
            SetDlgItemTextW(hDlg, (IDC_CREATED_LABEL as i32), addr_of_mut!(wszBuf) as LPCWSTR);
        } else {
            SetDlgItemTextW(hDlg, (IDC_CREATED_LABEL as i32), addr_of_mut!(szTemp) as LPCWSTR);
        }

        if GetFileTimeString(
            core::ptr::addr_of_mut!(FileInfo.ftLastWriteTime),
            addr_of_mut!(wszBuf) as LPWSTR,
            ARRAYSIZE!(wszBuf) as UINT,
        ) != 0
        {
            SetDlgItemTextW(hDlg, (IDC_MODIFIED_LABEL as i32), addr_of_mut!(wszBuf) as LPCWSTR);
        } else {
            SetDlgItemTextW(hDlg, (IDC_MODIFIED_LABEL as i32), addr_of_mut!(szTemp) as LPCWSTR);
        }

        if GetFileTimeString(
            core::ptr::addr_of_mut!(FileInfo.ftLastAccessTime),
            addr_of_mut!(wszBuf) as LPWSTR,
            ARRAYSIZE!(wszBuf) as UINT,
        ) != 0
        {
            SetDlgItemTextW(hDlg, (IDC_ACCESSED_LABEL as i32), addr_of_mut!(wszBuf) as LPCWSTR);
        } else {
            SetDlgItemTextW(hDlg, (IDC_MODIFIED_LABEL as i32), addr_of_mut!(szTemp) as LPCWSTR);
        }
    } else {
        LoadStringW(
            hInst,
            IDS_NOT_AVAILABLE,
            addr_of_mut!(szTemp) as LPWSTR,
            ARRAYSIZE!(szTemp) as i32,
        );

        SetDlgItemTextW(hDlg, (IDC_SIZE_LABEL as i32), addr_of_mut!(szTemp) as LPCWSTR);
        SetDlgItemTextW(hDlg, (IDC_CREATED_LABEL as i32), addr_of_mut!(szTemp) as LPCWSTR);
        SetDlgItemTextW(hDlg, (IDC_MODIFIED_LABEL as i32), addr_of_mut!(szTemp) as LPCWSTR);
        SetDlgItemTextW(hDlg, (IDC_ACCESSED_LABEL as i32), addr_of_mut!(szTemp) as LPCWSTR);
    }

    if (*EventLog).Permanent != 0 {
        SendDlgItemMessageW(
            hDlg,
            (IDC_UPDOWN_MAXLOGSIZE as i32),
            UDM_SETRANGE32 as UINT,
            (1) as WPARAM,
            (0x3FFFC0) as LPARAM,
        );
        SendDlgItemMessageW(
            hDlg,
            (IDC_UPDOWN_EVENTS_AGE as i32),
            UDM_SETRANGE,
            0,
            MAKELONG!(365, 1) as LPARAM,
        );

        SetDlgItemInt(hDlg, (IDC_EDIT_MAXLOGSIZE as i32), dwMaxSize as UINT, (FALSE) as BOOL);
        SetDlgItemInt(
            hDlg,
            (IDC_EDIT_EVENTS_AGE as i32),
            (if dwRetention == 0 {
                7
            } else {
                dwRetention
            }) as UINT,
            (FALSE) as BOOL,
        );

        if dwRetention == 0 {
            CheckRadioButton(hDlg, (IDC_OVERWRITE_AS_NEEDED as i32), (IDC_NO_OVERWRITE as i32), (IDC_OVERWRITE_AS_NEEDED as i32));
            EnableDlgItem!(hDlg, (IDC_EDIT_EVENTS_AGE as i32), (FALSE as i32));
            EnableDlgItem!(hDlg, (IDC_UPDOWN_EVENTS_AGE as i32), (FALSE as i32));
        } else if dwRetention == INFINITE {
            CheckRadioButton(hDlg, (IDC_OVERWRITE_AS_NEEDED as i32), (IDC_NO_OVERWRITE as i32), (IDC_NO_OVERWRITE as i32));
            EnableDlgItem!(hDlg, (IDC_EDIT_EVENTS_AGE as i32), (FALSE as i32));
            EnableDlgItem!(hDlg, (IDC_UPDOWN_EVENTS_AGE as i32), (FALSE as i32));
        } else {
            CheckRadioButton(hDlg, (IDC_OVERWRITE_AS_NEEDED as i32), (IDC_NO_OVERWRITE as i32), (IDC_OVERWRITE_OLDER_THAN as i32));
            EnableDlgItem!(hDlg, (IDC_EDIT_EVENTS_AGE as i32), (TRUE as i32));
            EnableDlgItem!(hDlg, (IDC_UPDOWN_EVENTS_AGE as i32), (TRUE as i32));
        }
    } else {
        // TODO: Hide the unused controls! Or, just use another type of property sheet!
    }
}

unsafe extern "C" fn SavePropertiesDlg(hDlg: HWND, EventLog: PEVENTLOG) {
    let lpLogName: LPWSTR = (*EventLog).LogName;

    let mut Result: LONG;
    let mut dwMaxSize: DWORD = 0 as DWORD;
    let mut dwRetention: DWORD = 0 as DWORD;
    let mut hLogKey: HKEY = core::ptr::null_mut();
    let mut KeyPath: PWSTR;
    let mut cbKeyPath: SIZE_T;

    if (*EventLog).Permanent == 0 {
        return;
    }

    cbKeyPath = ((wcslen(EVENTLOG_BASE_KEY) as SIZE_T)
        + (wcslen(lpLogName) as SIZE_T)
        + (1 as SIZE_T))
        * (core::mem::size_of::<WCHAR>() as SIZE_T);
    KeyPath = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        cbKeyPath,
    ) as PWSTR;
    if KeyPath.is_null() {
        ShowWin32Error(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return;
    }

    StringCbCopyW(KeyPath, cbKeyPath as size_t, EVENTLOG_BASE_KEY as LPCWSTR);
    StringCbCatW(KeyPath, cbKeyPath as size_t, lpLogName);

    Result = RegOpenKeyExW(
        hkMachine,
        KeyPath,
        0 as DWORD,
        KEY_SET_VALUE as REGSAM,
        core::ptr::addr_of_mut!(hLogKey),
    ) as LONG;
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        KeyPath as PVOID,
    );
    if Result != ERROR_SUCCESS as LONG {
        ShowWin32Error(Result as DWORD);
        return;
    }

    dwMaxSize = (GetDlgItemInt(
        hDlg,
        IDC_EDIT_MAXLOGSIZE as ::core::ffi::c_int,
        core::ptr::null_mut(),
        FALSE as BOOL,
    ) as DWORD)
        .wrapping_mul(1024 as DWORD);
    RegSetValueExW(
        hLogKey,
        RUSTCONV_WSTR_0AF4B1084E0C.as_ptr() as *const u16,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(dwMaxSize) as LPBYTE,
        core::mem::size_of::<DWORD>() as DWORD,
    );

    if IsDlgButtonChecked(hDlg, IDC_OVERWRITE_AS_NEEDED as ::core::ffi::c_int)
        == BST_CHECKED as UINT
    {
        dwRetention = 0 as DWORD;
    } else if IsDlgButtonChecked(hDlg, IDC_NO_OVERWRITE as ::core::ffi::c_int)
        == BST_CHECKED as UINT
    {
        dwRetention = INFINITE as DWORD;
    } else {
        // if (IsDlgButtonChecked(hDlg, IDC_OVERWRITE_OLDER_THAN) == BST_CHECKED)
        dwRetention = (GetDlgItemInt(
            hDlg,
            IDC_EDIT_EVENTS_AGE as ::core::ffi::c_int,
            core::ptr::null_mut(),
            FALSE as BOOL,
        ) as DWORD)
            .wrapping_mul(
                (24 as DWORD).wrapping_mul(3600 as DWORD),
            );
    }

    RegSetValueExW(
        hLogKey,
        RUSTCONV_WSTR_2F23369E2DF4.as_ptr() as *const u16,
        0 as DWORD,
        REG_DWORD as DWORD,
        core::ptr::addr_of_mut!(dwRetention) as LPBYTE,
        core::mem::size_of::<DWORD>() as DWORD,
    );

    RegCloseKey(hLogKey);
}
#[no_mangle]
pub unsafe extern "system" fn EventLogPropProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut EventLog: PEVENTLOG = core::ptr::null_mut();
    let mut szText: [WCHAR; MAX_LOADSTRING] = [0; MAX_LOADSTRING];

    EventLog = GetWindowLongPtrW(hDlg, DWLP_USER) as PEVENTLOG;

    match uMsg {
        WM_INITDIALOG => {
            EventLog = (*(lParam as LPPROPSHEETPAGE)).lParam as PEVENTLOG;
            SetWindowLongPtrW(hDlg, DWLP_USER, (EventLog as LONG_PTR) as _);

            InitPropertiesDlg(hDlg, EventLog);

            PropSheet_UnChanged!(GetParent(hDlg), hDlg);
            return TRUE as INT_PTR;
        }

        WM_DESTROY => {
            return TRUE as INT_PTR;
        }

        WM_NOTIFY => {
            match ((*(lParam as LPNMHDR)).code) as i32 {
                PSN_APPLY => {
                    PropSheet_UnChanged!(GetParent(hDlg), hDlg);
                    SavePropertiesDlg(hDlg, EventLog);
                    return TRUE as INT_PTR;
                }
                _ => {}
            }
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDOK | IDCANCEL => {
                    EndDialog(hDlg, LOWORD!(wParam) as INT_PTR);
                    return TRUE as INT_PTR;
                }

                ID_CLEARLOG => {
                    let EventLogFilter: PEVENTLOGFILTER =
                        GetSelectedFilter(core::ptr::null_mut::<HTREEITEM>());
                    if !EventLogFilter.is_null() && ClearEvents(EventLogFilter) != 0 {
                        Refresh(EventLogFilter);
                        InitPropertiesDlg(hDlg, EventLog);
                    }
                    return TRUE as INT_PTR;
                }

                IDC_EDIT_EVENTS_AGE | IDC_EDIT_MAXLOGSIZE => {
                    if HIWORD!(wParam) == (EN_CHANGE as u16) {
                        PropSheet_Changed!(GetParent(hDlg), hDlg);
                    }
                    return TRUE as INT_PTR;
                }

                IDC_OVERWRITE_AS_NEEDED => {
                    CheckRadioButton(hDlg, (IDC_OVERWRITE_AS_NEEDED as i32), (IDC_NO_OVERWRITE as i32), (IDC_OVERWRITE_AS_NEEDED as i32));
                    EnableDlgItem!(hDlg, (IDC_EDIT_EVENTS_AGE as i32), (FALSE as i32));
                    EnableDlgItem!(hDlg, (IDC_UPDOWN_EVENTS_AGE as i32), (FALSE as i32));
                    PropSheet_Changed!(GetParent(hDlg), hDlg);
                    return TRUE as INT_PTR;
                }

                IDC_OVERWRITE_OLDER_THAN => {
                    CheckRadioButton(hDlg, (IDC_OVERWRITE_AS_NEEDED as i32), (IDC_NO_OVERWRITE as i32), (IDC_OVERWRITE_OLDER_THAN as i32));
                    EnableDlgItem!(hDlg, (IDC_EDIT_EVENTS_AGE as i32), (TRUE as i32));
                    EnableDlgItem!(hDlg, (IDC_UPDOWN_EVENTS_AGE as i32), (TRUE as i32));
                    PropSheet_Changed!(GetParent(hDlg), hDlg);
                    return TRUE as INT_PTR;
                }

                IDC_NO_OVERWRITE => {
                    CheckRadioButton(hDlg, (IDC_OVERWRITE_AS_NEEDED as i32), (IDC_NO_OVERWRITE as i32), (IDC_NO_OVERWRITE as i32));
                    EnableDlgItem!(hDlg, (IDC_EDIT_EVENTS_AGE as i32), (FALSE as i32));
                    EnableDlgItem!(hDlg, (IDC_UPDOWN_EVENTS_AGE as i32), (FALSE as i32));
                    PropSheet_Changed!(GetParent(hDlg), hDlg);
                    return TRUE as INT_PTR;
                }

                IDC_RESTOREDEFAULTS => {
                    LoadStringW(hInst, IDS_RESTOREDEFAULTS, core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPWSTR, (_countof!(szText)) as i32);

                    if MessageBoxW(hDlg, core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR, MB_YESNO | MB_ICONQUESTION) == (IDYES as i32) {
                        CheckRadioButton(hDlg, (IDC_OVERWRITE_AS_NEEDED as i32), (IDC_NO_OVERWRITE as i32), (IDC_OVERWRITE_AS_NEEDED as i32));
                        /* Workstation: 512 KB; Server: 16384 KB */
                        SetDlgItemInt(hDlg, (IDC_EDIT_MAXLOGSIZE as i32), 5120, (FALSE) as BOOL);
                        SetDlgItemInt(hDlg, (IDC_EDIT_EVENTS_AGE as i32), 7, (FALSE) as BOOL);
                        EnableDlgItem!(hDlg, (IDC_EDIT_EVENTS_AGE as i32), (FALSE as i32));
                        EnableDlgItem!(hDlg, (IDC_UPDOWN_EVENTS_AGE as i32), (FALSE as i32));
                        PropSheet_Changed!(GetParent(hDlg), hDlg);
                    }
                    return TRUE as INT_PTR;
                }

                IDHELP => {
                    MessageBoxW(hDlg,
                                RUSTCONV_WSTR_08C4A5A5D53A.as_ptr(),
                                core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR,
                                MB_OK | MB_ICONINFORMATION);
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn EventLogProperties(
    hInstance: HINSTANCE,
    hWndParent: HWND,
    EventLogFilter: PEVENTLOGFILTER,
) -> INT_PTR {
    let mut ret: INT_PTR = 0 as INT_PTR;
    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
    let mut psp: [PROPSHEETPAGEW; 1] = core::mem::zeroed(); // 2

    /*
     * Bail out if there is no available filter, or if the filter
     * contains more than one log.
     */
    if EventLogFilter.is_null() {
        return 0 as INT_PTR;
    }

    EventLogFilter_AddRef(EventLogFilter);

    'quit: {
        if (*EventLogFilter).NumOfEventLogs > 1 as ULONG
            || (*(*EventLogFilter).EventLogs.as_ptr().add(0 as usize)).is_null()
        {
            break 'quit;
        }

        /* Header */
        psh.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as DWORD;
        psh.dwFlags = (PSH_PROPSHEETPAGE /*| PSH_USEICONID */
            | PSH_PROPTITLE
            | PSH_HASHELP) as DWORD /*| PSH_NOCONTEXTHELP */ /*| PSH_USECALLBACK */;
        psh.hInstance = hInstance;
        psh.hwndParent = hWndParent;
        // psh.pszIcon     = MAKEINTRESOURCEW(IDI_APPICON); // Disabled because it only sets the small icon; the big icon is a stretched version of the small one.
        psh.pszCaption = (*(*(*EventLogFilter).EventLogs.as_ptr().add(0 as usize))).LogName
            as LPCWSTR;
        psh.__bindgen_anon_2.nStartPage = 0 as UINT;
        psh.__bindgen_anon_3.ppsp = core::ptr::addr_of_mut!(psp) as *mut PROPSHEETPAGEW as LPCPROPSHEETPAGEW;
        psh.nPages = ARRAYSIZE!(psp) as UINT;
        // psh.pfnCallback = PropSheetCallback;

        /* Log properties page */
        psp[0 as usize].dwSize =
            core::mem::size_of_val(&psp[0 as usize]) as DWORD;
        psp[0 as usize].dwFlags = PSP_HASHELP as DWORD;
        psp[0 as usize].hInstance = hInstance;
        psp[0 as usize].__bindgen_anon_1.pszTemplate =
            MAKEINTRESOURCEW!(IDD_LOGPROPERTIES_GENERAL) as LPCWSTR;
        psp[0 as usize].pfnDlgProc = Some(EventLogPropProc);
        psp[0 as usize].lParam =
            (*(*EventLogFilter).EventLogs.as_ptr().add(0 as usize)) as LPARAM;

        /* Create the property sheet */
        ret = PropertySheetW(core::ptr::addr_of_mut!(psh) as LPCPROPSHEETHEADERW);
    }

    // Quit:
    EventLogFilter_Release(EventLogFilter);
    return ret;
}
#[no_mangle]
pub unsafe extern "system" fn EventDetails(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            let mut dwStyle: LONG_PTR = 0;
            let mut rcWnd: RECT = core::mem::zeroed();
            let mut rect: RECT = core::mem::zeroed();
            let mut iEventItem: INT = 0;

            hWndDetailsCtrl = CreateEventDetailsCtrl(hInst, hDlg, lParam);
            if hWndDetailsCtrl.is_null() {
                EndDialog(hDlg, 0);
                return TRUE as INT_PTR;
            }

            /* Create a size grip if the dialog has a sizing border */
            GetClientRect(hDlg, core::ptr::addr_of_mut!(rcWnd));
            dwStyle = GetWindowLongPtrW(hDlg, GWL_STYLE);
            if (dwStyle & (WS_THICKFRAME as LONG_PTR)) != 0 /* == WS_SIZEBOX */ {
                let mut sbVXSize: INT = GetSystemMetrics((SM_CXVSCROLL as i32));
                let mut sbHYSize: INT = GetSystemMetrics((SM_CYHSCROLL as i32));

                hWndGrip = CreateWindowW!(
                    WC_SCROLLBARW as LPCWSTR,
                    core::ptr::null_mut(),
                    (WS_CHILD | WS_VISIBLE | /**/ WS_CLIPSIBLINGS | /**/ SBS_SIZEGRIP | SBS_SIZEBOXBOTTOMRIGHTALIGN) as DWORD,
                    rcWnd.right - sbVXSize,
                    rcWnd.bottom - sbHYSize,
                    sbVXSize,
                    sbHYSize,
                    hDlg,
                    core::ptr::null_mut(),
                    hInst,
                    core::ptr::null_mut(),
                );
            }

            // SetWindowLongPtrW(hDlg, DWLP_USER, (LONG_PTR)hWndDetailsCtrl);

            /*
             * Compute the minimum window size (in window coordinates) by
             * adding the widths/heights of the "Help" and "Close" buttons,
             * together with the margins, and add some minimal spacing
             * between the buttons.
             */
            GetWindowRect(hDlg, core::ptr::addr_of_mut!(rcWnd));
            cyMin = 0 as INT;
            cxMin = cyMin;

            GetWindowRect(GetDlgItem(hDlg, (IDHELP as i32)), core::ptr::addr_of_mut!(rect));
            cxMin += (rect.right - rect.left) + (rect.left - rcWnd.left); // == (rect.right - rcWnd.left);
            cyMin += (rect.bottom - rect.top) + (rcWnd.bottom - rect.bottom); // == (rcWnd.bottom - rect.top);

            GetWindowRect(GetDlgItem(hDlg, (IDOK as i32)), core::ptr::addr_of_mut!(rect));
            cxMin += (rect.right - rect.left) + (rcWnd.right - rect.right); // == (rcWnd.right - rect.left);
            cyMin += (rect.bottom - rect.top) + (rcWnd.bottom - rect.bottom); // == (rcWnd.bottom - rect.top);

            /*
             * Convert the window rect from window to client coordinates
             * in order to retrieve the sizes of the left and top margins,
             * and add some extra space.
             */
            MapWindowPoints(
                HWND_DESKTOP as HWND /*NULL*/,
                hDlg,
                core::ptr::addr_of_mut!(rcWnd) as LPPOINT,
                (core::mem::size_of::<RECT>() / core::mem::size_of::<POINT>() as usize) as UINT,
            );

            cxMin += -(2) * rcWnd.left;   // Minimal spacing between the buttons == 2 * left margin
            cyMin += -rcWnd.top + 12; // Add some space on top

            GetClientRect(hDlg, core::ptr::addr_of_mut!(rcWnd));
            cxOld = rcWnd.right - rcWnd.left;
            cyOld = rcWnd.bottom - rcWnd.top;

            /* Display the event info in the dialog */
            iEventItem = if lParam != 0 as LPARAM {
                (*(lParam as PEVENTDETAIL_INFO)).iEventItem
            } else {
                -(1)
            };
            SendMessageW(hWndDetailsCtrl, EVT_DISPLAY, (TRUE) as WPARAM, iEventItem as LPARAM);

            // SetWindowPos(hWndDetailsCtrl, NULL,
            // 0, 0,
            // (rcWnd.right - rcWnd.left),
            // (rcWnd.bottom - rcWnd.top),
            // SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW);

            /*
             * Hide the placeholder static control and show the event details
             * control instead. Note that the placeholder is here so far just
             * to get the dimensions right in the dialog resource editor.
             * I plan to remove it and use a custom control with a suitable
             * window class for it, that would create the event details control
             * instead.
             */
            ShowWindow(GetDlgItem(hDlg, IDC_STATIC), (SW_HIDE as i32));
            ShowWindow(hWndDetailsCtrl, (SW_SHOW as i32));
            return TRUE as INT_PTR;
        }

        WM_DESTROY => {
            if IsWindow(hWndDetailsCtrl) != 0 {
                DestroyWindow(hWndDetailsCtrl);
            }
            hWndDetailsCtrl = core::ptr::null_mut();
            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDOK | IDCANCEL => {
                    EndDialog(hDlg, LOWORD!(wParam) as INT_PTR);
                    return TRUE as INT_PTR;
                }

                IDHELP => {
                    MessageBoxW(
                        hDlg,
                        RUSTCONV_WSTR_08C4A5A5D53A.as_ptr(),
                        core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR,
                        (MB_OK | MB_ICONINFORMATION) as UINT,
                    );
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
            // break;
        }

        WM_SETCURSOR => {
            if (wParam as HWND == hWndGrip) && (LOWORD!(lParam) == (HTCLIENT as u16)) {
                SetCursor(LoadCursorW(core::ptr::null_mut(), IDC_SIZENWSE as LPCWSTR));
                SetWindowLongPtrW(hDlg, DWLP_MSGRESULT, (TRUE as LONG_PTR) as _);
                return TRUE as INT_PTR;
            }
            // break;
        }

        WM_SIZING => {
            /* Forbid resizing the dialog smaller than its minimal size */
            let mut dragRect: PRECT = lParam as PRECT;

            if (wParam == WMSZ_LEFT as WPARAM) || (wParam == WMSZ_TOPLEFT as WPARAM) || (wParam == WMSZ_BOTTOMLEFT as WPARAM) {
                if (*dragRect).right - (*dragRect).left < cxMin {
                    (*dragRect).left = (*dragRect).right - cxMin;
                }
            }

            if (wParam == WMSZ_RIGHT as WPARAM) || (wParam == WMSZ_TOPRIGHT as WPARAM) || (wParam == WMSZ_BOTTOMRIGHT as WPARAM) {
                if (*dragRect).right - (*dragRect).left < cxMin {
                    (*dragRect).right = (*dragRect).left + cxMin;
                }
            }

            if (wParam == WMSZ_TOP as WPARAM) || (wParam == WMSZ_TOPLEFT as WPARAM) || (wParam == WMSZ_TOPRIGHT as WPARAM) {
                if (*dragRect).bottom - (*dragRect).top < cyMin {
                    (*dragRect).top = (*dragRect).bottom - cyMin;
                }
            }

            if (wParam == WMSZ_BOTTOM as WPARAM) || (wParam == WMSZ_BOTTOMLEFT as WPARAM) || (wParam == WMSZ_BOTTOMRIGHT as WPARAM) {
                if (*dragRect).bottom - (*dragRect).top < cyMin {
                    (*dragRect).bottom = (*dragRect).top + cyMin;
                }
            }

            SetWindowLongPtrW(hDlg, DWLP_MSGRESULT, (TRUE as LONG_PTR) as _);
            return TRUE as INT_PTR;
        }

        WM_SIZE => {
            let mut cx: INT = (LOWORD!(lParam)) as i32;
            let mut cy: INT = (HIWORD!(lParam)) as i32;

            let mut hdwp: HDWP = core::ptr::null_mut();
            let mut hItemWnd: HWND;
            let mut rect: RECT = core::mem::zeroed();

            hdwp = BeginDeferWindowPos(4);

            /* Resize the event details control window */

            hItemWnd = hWndDetailsCtrl;
            GetWindowRect(hItemWnd, core::ptr::addr_of_mut!(rect));
            MapWindowPoints(
                HWND_DESKTOP as HWND /*NULL*/,
                hDlg,
                core::ptr::addr_of_mut!(rect) as LPPOINT,
                (core::mem::size_of::<RECT>() / core::mem::size_of::<POINT>() as usize) as UINT,
            );

            if !hdwp.is_null() {
                hdwp = DeferWindowPos(
                    hdwp,
                    hItemWnd,
                    HWND_TOP as HWND,
                    0,
                    0,
                    (rect.right - rect.left) + (cx - cxOld),
                    (rect.bottom - rect.top) + (cy - cyOld),
                    (SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE) as UINT,
                );
            }

            /* Move the buttons */

            hItemWnd = GetDlgItem(hDlg, (IDHELP as i32));
            GetWindowRect(hItemWnd, core::ptr::addr_of_mut!(rect));
            MapWindowPoints(
                HWND_DESKTOP as HWND /*NULL*/,
                hDlg,
                core::ptr::addr_of_mut!(rect) as LPPOINT,
                (core::mem::size_of::<RECT>() / core::mem::size_of::<POINT>() as usize) as UINT,
            );

            if !hdwp.is_null() {
                hdwp = DeferWindowPos(
                    hdwp,
                    hItemWnd,
                    HWND_TOP as HWND,
                    rect.left,
                    rect.top + (cy - cyOld),
                    0,
                    0,
                    (SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE) as UINT,
                );
            }

            hItemWnd = GetDlgItem(hDlg, (IDOK as i32));
            GetWindowRect(hItemWnd, core::ptr::addr_of_mut!(rect));
            MapWindowPoints(
                HWND_DESKTOP as HWND /*NULL*/,
                hDlg,
                core::ptr::addr_of_mut!(rect) as LPPOINT,
                (core::mem::size_of::<RECT>() / core::mem::size_of::<POINT>() as usize) as UINT,
            );

            if !hdwp.is_null() {
                hdwp = DeferWindowPos(
                    hdwp,
                    hItemWnd,
                    HWND_TOP as HWND,
                    rect.left + (cx - cxOld),
                    rect.top + (cy - cyOld),
                    0,
                    0,
                    (SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE) as UINT,
                );
            }

            /* Move the size grip */
            if !hWndGrip.is_null() && !hdwp.is_null() {
                GetWindowRect(hWndGrip, core::ptr::addr_of_mut!(rect));
                MapWindowPoints(
                    HWND_DESKTOP as HWND /*NULL*/,
                    hDlg,
                    core::ptr::addr_of_mut!(rect) as LPPOINT,
                    (core::mem::size_of::<RECT>() / core::mem::size_of::<POINT>() as usize) as UINT,
                );

                hdwp = DeferWindowPos(
                    hdwp,
                    hWndGrip,
                    HWND_TOP as HWND,
                    rect.left + (cx - cxOld),
                    rect.top + (cy - cyOld),
                    0,
                    0,
                    (SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE) as UINT,
                );
            }

            if !hdwp.is_null() {
                EndDeferWindowPos(hdwp);
            }

            /* Hide the size grip if we are in maximized mode */
            if !hWndGrip.is_null() {
                ShowWindow(
                    hWndGrip,
                    if wParam == SIZE_MAXIMIZED as WPARAM {
                        (SW_HIDE as i32)
                    } else {
                        (SW_SHOW as i32)
                    },
                );
            }

            cxOld = cx;
            cyOld = cy;

            SetWindowLongPtrW(hDlg, DWLP_MSGRESULT, (0 as LONG_PTR) as _);
            return TRUE as INT_PTR;
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
