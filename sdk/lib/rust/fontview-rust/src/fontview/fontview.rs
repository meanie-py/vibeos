use crate::fontview::display::Display_InitClass;
use crate::fontview::display::Display_OnPrint;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_06491E66FB66: [u16; 51] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0046, 0x006F, 0x006E, 0x0074, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_09DBAEEA59BF: [u16; 7] = [0x0062u16, 0x0075, 0x0074, 0x0074, 0x006F, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_99F5D5B7A782: [u16; 8] = [0x005Cu16, 0x0046, 0x006F, 0x006E, 0x0074, 0x0073, 0x005C, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

#[no_mangle]
pub static mut g_hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut g_FontIndex: INT = 0 as INT;

#[no_mangle]
pub static mut g_NumFonts: INT = 0 as INT;

#[no_mangle]
pub static mut g_LogFonts: [LOGFONTW; 64] = [tagLOGFONTW {
    lfHeight: 0,
    lfWidth: 0,
    lfEscapement: 0,
    lfOrientation: 0,
    lfWeight: 0,
    lfItalic: 0,
    lfUnderline: 0,
    lfStrikeOut: 0,
    lfCharSet: 0,
    lfOutPrecision: 0,
    lfClipPrecision: 0,
    lfQuality: 0,
    lfPitchAndFamily: 0,
    lfFaceName: [0; 32],
}; 64];

#[no_mangle]
pub static mut g_fileName: LPCWSTR = unsafe {
    RUSTCONV_C2R_WSTR_96A296D224F2.as_ptr() as LPCWSTR
};

#[no_mangle]
pub static mut g_FontTitle: [WCHAR; 1024] = unsafe {
    ::core::mem::transmute::<
        [u8; 2048],
        [WCHAR; 1024],
    >(
        *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    )
};

#[no_mangle]
pub static mut g_FontPrint: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut g_DisableInstall: BOOL = (FALSE) as i32;

static mut g_szFontViewClassName: [WCHAR; 15] = unsafe {
    ::core::mem::transmute::<[u8; 30], [WCHAR; 15]>(
        *b"F\0o\0n\0t\0V\0i\0e\0w\0W\0C\0l\0a\0s\0s\0\0\0",
    )
};

// GetFontResourceInfoW: prototype only; provided via the extern imports, not emitted here.
#[no_mangle]
pub unsafe extern "C" fn FormatString(
    mut dwFlags: DWORD,
    mut hInstance: HINSTANCE,
    mut dwStringId: DWORD,
    mut dwLanguageId: DWORD,
    mut lpBuffer: LPWSTR,
    mut nSize: DWORD,
    mut Arguments: *mut va_list,
) -> DWORD {
    let mut dwRet: DWORD;
    let mut len: ::core::ffi::c_int;
    let mut Buffer: [WCHAR; 1000] = [0; 1000];

    len = LoadStringW(
        hInstance,
        dwStringId as UINT,
        core::ptr::addr_of_mut!(Buffer) as *mut WCHAR,
        1000 as ::core::ffi::c_int,
    );

    if len != 0 {
        dwFlags |= FORMAT_MESSAGE_FROM_STRING as DWORD;
        dwFlags &= !(FORMAT_MESSAGE_FROM_HMODULE | FORMAT_MESSAGE_FROM_SYSTEM) as DWORD;
        dwRet = FormatMessageW(
            dwFlags,
            core::ptr::addr_of_mut!(Buffer) as *mut WCHAR as LPCVOID,
            0 as DWORD,
            dwLanguageId,
            lpBuffer,
            nSize,
            Arguments,
        );
        return dwRet;
    }
    return 0 as DWORD;
}

unsafe extern "C" fn FormatMsgBox(
    mut hParent: HWND,
    mut dwMessageId: DWORD,
    mut dwCaptionId: DWORD,
    mut uType: UINT,
    mut args: va_list,
) {
    let mut hMemCaption: HLOCAL = core::ptr::null_mut();
    let mut hMemText: HLOCAL = core::ptr::null_mut();
    FormatString(
        FORMAT_MESSAGE_ALLOCATE_BUFFER as DWORD,
        core::ptr::null_mut(),
        dwMessageId,
        0 as DWORD,
        core::ptr::addr_of_mut!(hMemText) as LPWSTR,
        0 as DWORD,
        core::ptr::addr_of_mut!(args),
    );
    FormatString(
        FORMAT_MESSAGE_ALLOCATE_BUFFER as DWORD,
        core::ptr::null_mut(),
        dwCaptionId,
        0 as DWORD,
        core::ptr::addr_of_mut!(hMemCaption) as LPWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
    );

    MessageBoxW(hParent, hMemText as LPCWSTR, hMemCaption as LPCWSTR, uType);
    LocalFree(hMemCaption);
    LocalFree(hMemText);
}

// ErrorMsgBox / SuccessMsgBox are C-variadic; stable Rust cannot define one, so the
// generated rust_varargs_shim.c trampoline `va_start`s and hands over `&ap`. Both bodies only
// FORWARD the list (FormatMsgBox -> FormatMessageW's `va_list *Arguments`), so the twin
// passes the trampoline's va_list on by value, as the C does.
#[no_mangle]
pub unsafe extern "C" fn ErrorMsgBox_v(hParent: HWND, dwMessageId: DWORD, ap_list: *mut ::core::ffi::c_void) {
    FormatMsgBox(
        hParent,
        dwMessageId,
        IDS_ERROR as DWORD,
        MB_ICONERROR as UINT,
        *(ap_list as *mut va_list),
    );
}

#[no_mangle]
pub unsafe extern "C" fn SuccessMsgBox_v(hParent: HWND, dwMessageId: DWORD, ap_list: *mut ::core::ffi::c_void) {
    FormatMsgBox(
        hParent,
        dwMessageId,
        IDS_SUCCESS as DWORD,
        MB_ICONINFORMATION as UINT,
        *(ap_list as *mut va_list),
    );
}



#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hThisInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpCmdLine: LPWSTR,
    nCmdShow: core::ffi::c_int,
) -> core::ffi::c_int {
    let mut argc: core::ffi::c_int = 0;
    let mut i: INT;
    let mut argv: *mut *mut WCHAR = core::ptr::null_mut();
    let mut dwSize: DWORD;
    let mut hMainWnd: HWND;
    let mut msg: MSG = core::mem::zeroed();
    let mut wincl: WNDCLASSEXW = core::mem::zeroed();
    let mut fileName: LPCWSTR = core::ptr::null();

    let sw = GetUserDefaultUILanguage() as core::ffi::c_int;
    if sw == MAKELANGID!(LANG_HEBREW, SUBLANG_DEFAULT) as core::ffi::c_int {
        SetProcessDefaultLayout(LAYOUT_RTL);
    }

    g_hInstance = hThisInstance;

    /* Get unicode command line */
    argv = CommandLineToArgvW(GetCommandLineW(), core::ptr::addr_of_mut!(argc)) as *mut *mut WCHAR;
    if argc < 2 {
    } else {
        /* Try to add the font resource from command line */
        i = 1 as INT;
        while i < argc {
            // Treat the last argument as filename
            if i + 1 == argc {
                fileName = *argv.offset(i as isize) as LPCWSTR;
            } else if *(*argv.offset(i as isize)).offset(0 as isize) == '/' as WCHAR
                || *(*argv.offset(i as isize)).offset(0 as isize) == '-' as WCHAR
            {
                let sw2 = *(*argv.offset(i as isize)).offset(1 as isize);
                if sw2 == 'p' as WCHAR || sw2 == 'P' as WCHAR {
                    g_FontPrint = (TRUE) as i32;
                } else if sw2 == 'd' as WCHAR || sw2 == 'D' as WCHAR {
                    g_DisableInstall = (TRUE) as i32;
                } else {
                    fileName = *argv.offset(i as isize) as LPCWSTR;
                }
            } else {
                fileName = *argv.offset(i as isize) as LPCWSTR;
            }
            i += 1;
        }
        g_fileName = fileName;
    }

    if AddFontResourceW(g_fileName) == 0 {
        ErrorMsgBox(core::ptr::null_mut(), IDS_ERROR_NOFONT, g_fileName);
        return -1;
    }

    /* Get the font name */
    dwSize = core::mem::size_of::<[LOGFONTW; 64]>() as DWORD;
    ZeroMemory!(
        core::ptr::addr_of_mut!(g_LogFonts) as *mut core::ffi::c_void,
        core::mem::size_of::<[LOGFONTW; 64]>(),
    );
    if GetFontResourceInfoW(
        fileName,
        core::ptr::addr_of_mut!(dwSize),
        core::ptr::addr_of_mut!(g_LogFonts) as *mut core::ffi::c_void,
        2,
    ) == 0
    {
        ErrorMsgBox(core::ptr::null_mut(), IDS_ERROR_NOFONT, fileName);
        return -1;
    }
    g_NumFonts = 0;
    i = 0 as INT;
    while i < ARRAYSIZE!(&*core::ptr::addr_of!(g_LogFonts)) as INT {
        if g_LogFonts[i as usize].lfFaceName[0 as usize] == 0 as WCHAR {
            break;
        }

        g_NumFonts += 1;
        i += 1;
    }
    if g_NumFonts == 0 {
        ErrorMsgBox(core::ptr::null_mut(), IDS_ERROR_NOFONT, fileName);
        return -1;
    }

    /* get font title */
    dwSize = core::mem::size_of::<[WCHAR; 1024]>() as DWORD;
    ZeroMemory!(
        core::ptr::addr_of_mut!(g_FontTitle) as *mut core::ffi::c_void,
        core::mem::size_of::<[WCHAR; 1024]>(),
    );
    GetFontResourceInfoW(
        fileName,
        core::ptr::addr_of_mut!(dwSize),
        core::ptr::addr_of_mut!(g_FontTitle) as *mut core::ffi::c_void,
        1,
    );

    if Display_InitClass(hThisInstance) == 0 {
        ErrorMsgBox(core::ptr::null_mut(), IDS_ERROR_NOCLASS);
        return -1;
    }

    /* The main window class */
    wincl.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    wincl.style = CS_DBLCLKS as UINT;
    wincl.lpfnWndProc = Some(MainWndProc);
    wincl.cbClsExtra = 0;
    wincl.cbWndExtra = 0;
    wincl.hInstance = hThisInstance;
    wincl.hIcon = LoadIcon(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        MAKEINTRESOURCE!(IDI_TT) as LPCWSTR,
    );
    wincl.hCursor = LoadCursor(
        core::ptr::null_mut(),
        IDC_ARROW as LPCWSTR,
    );
    wincl.hbrBackground = COLOR_BACKGROUND as HBRUSH;
    wincl.lpszMenuName = core::ptr::null();
    wincl.lpszClassName = core::ptr::addr_of!(g_szFontViewClassName) as *const WCHAR as LPCWSTR;
    wincl.hIconSm = LoadIcon(
        GetModuleHandle(core::ptr::null_mut()) as HINSTANCE,
        MAKEINTRESOURCE!(IDI_TT) as LPCWSTR,
    );

    /* Register the window class, and if it fails quit the program */
    if RegisterClassExW(core::ptr::addr_of_mut!(wincl)) == 0 {
        ErrorMsgBox(core::ptr::null_mut(), IDS_ERROR_NOCLASS);
        return 0;
    }

    /* The class is registered, let's create the main window */
    hMainWnd = CreateWindowExW(
        0,                      /* Extended possibilities for variation */
        core::ptr::addr_of!(g_szFontViewClassName) as LPCWSTR,  /* Classname */
        core::ptr::addr_of_mut!(g_FontTitle) as *mut WCHAR as LPCWSTR,            /* Title Text */
        WS_OVERLAPPEDWINDOW,    /* default window */
        CW_USEDEFAULT,          /* Windows decides the position */
        CW_USEDEFAULT,          /* where the window ends up on the screen */
        544,                    /* The programs width */
        375,                    /* and height in pixels */
        HWND_DESKTOP,           /* The window is a child-window to desktop */
        core::ptr::null_mut(),                   /* No menu */
        hThisInstance,          /* Program Instance handler */
        core::ptr::null_mut(),                   /* No Window Creation data */
    );
    ShowWindow(hMainWnd, nCmdShow);

    /* Main message loop */
    while GetMessage(
        core::ptr::addr_of_mut!(msg),
        core::ptr::null_mut(),
        0,
        0,
    ) != 0
    {
        if IsDialogMessage(hMainWnd, core::ptr::addr_of_mut!(msg)) != 0 {
            continue;
        }
        TranslateMessage(core::ptr::addr_of_mut!(msg));
        DispatchMessage(core::ptr::addr_of_mut!(msg));
    }

    RemoveFontResourceW(*argv.offset(1 as isize));

    return msg.wParam as core::ffi::c_int;
}

unsafe extern "C" fn MainWnd_OnCreate(hwnd: HWND) -> LRESULT {
    let mut szQuit: [WCHAR; (MAX_BUTTONNAME as usize)] = [0; (MAX_BUTTONNAME as usize)];
    let mut szPrint: [WCHAR; (MAX_BUTTONNAME as usize)] = [0; (MAX_BUTTONNAME as usize)];
    let mut szString: [WCHAR; (MAX_STRING as usize)] = [0; (MAX_STRING as usize)];
    let mut szPrevious: [WCHAR; (MAX_STRING as usize)] = [0; (MAX_STRING as usize)];
    let mut szNext: [WCHAR; (MAX_STRING as usize)] = [0; (MAX_STRING as usize)];
    let mut hDisplay: HWND = core::ptr::null_mut();
    let mut hButtonInstall: HWND = core::ptr::null_mut();
    let mut hButtonPrint: HWND = core::ptr::null_mut();
    let mut hButtonPrev: HWND = core::ptr::null_mut();
    let mut hButtonNext: HWND = core::ptr::null_mut();

    /* create the display window */
    hDisplay = CreateWindowExW(
        0,
        core::ptr::addr_of!(g_szFontDisplayClassName) as LPCWSTR,
        RUSTCONV_WSTR_96A296D224F2.as_ptr(),
        WS_CHILD | WS_VSCROLL,
        0,
        (HEADER_SIZE as i32),
        550,
        (370 - HEADER_SIZE) as i32,
        hwnd,
        IDC_DISPLAY as HMENU,
        g_hInstance,
        core::ptr::null_mut(),
    );

    LoadStringW(g_hInstance, IDS_STRING, szString.as_mut_ptr(), (MAX_STRING) as INT);
    SendMessage(hDisplay, FVM_SETSTRING, 0, szString.as_mut_ptr() as LPARAM);

    /* Create the install button */
    LoadStringW(g_hInstance, IDS_INSTALL, szQuit.as_mut_ptr(), (MAX_BUTTONNAME) as INT);
    hButtonInstall = CreateWindowExW(
        0,
        RUSTCONV_WSTR_09DBAEEA59BF.as_ptr(),
        szQuit.as_ptr(),
        WS_CHILD | WS_VISIBLE,
        (BUTTON_POS_X as i32),
        (BUTTON_POS_Y as i32),
        (BUTTON_WIDTH as i32),
        (BUTTON_HEIGHT as i32),
        hwnd,
        IDC_INSTALL as HMENU,
        g_hInstance,
        core::ptr::null_mut(),
    );
    SendMessage(
        hButtonInstall,
        WM_SETFONT,
        GetStockObject((DEFAULT_GUI_FONT as i32)) as WPARAM,
        TRUE as LPARAM,
    );
    EnableWindow(hButtonInstall, (g_DisableInstall == 0) as ::core::ffi::c_int);

    /* Create the print button */
    LoadStringW(g_hInstance, IDS_PRINT, szPrint.as_mut_ptr(), (MAX_BUTTONNAME) as INT);
    hButtonPrint = CreateWindowExW(
        0,
        RUSTCONV_WSTR_09DBAEEA59BF.as_ptr(),
        szPrint.as_ptr(),
        WS_CHILD | WS_VISIBLE,
        450,
        (BUTTON_POS_Y as i32),
        (BUTTON_WIDTH as i32),
        (BUTTON_HEIGHT as i32),
        hwnd,
        IDC_PRINT as HMENU,
        g_hInstance,
        core::ptr::null_mut(),
    );
    SendMessage(
        hButtonPrint,
        WM_SETFONT,
        GetStockObject((DEFAULT_GUI_FONT as i32)) as WPARAM,
        TRUE as LPARAM,
    );

    /* Create the previous button */
    LoadStringW(g_hInstance, IDS_PREVIOUS, szPrevious.as_mut_ptr(), (MAX_BUTTONNAME) as INT);
    hButtonPrev = CreateWindowExW(
        0,
        RUSTCONV_WSTR_09DBAEEA59BF.as_ptr(),
        szPrevious.as_ptr(),
        WS_CHILD | WS_VISIBLE,
        450,
        (BUTTON_POS_Y as i32),
        (BUTTON_WIDTH as i32),
        (BUTTON_HEIGHT as i32),
        hwnd,
        IDC_PREV as HMENU,
        g_hInstance,
        core::ptr::null_mut(),
    );
    SendMessage(
        hButtonPrev,
        WM_SETFONT,
        GetStockObject((DEFAULT_GUI_FONT as i32)) as WPARAM,
        TRUE as LPARAM,
    );

    /* Create the next button */
    LoadStringW(g_hInstance, IDS_NEXT, szNext.as_mut_ptr(), (MAX_BUTTONNAME) as INT);
    hButtonNext = CreateWindowExW(
        0,
        RUSTCONV_WSTR_09DBAEEA59BF.as_ptr(),
        szNext.as_ptr(),
        WS_CHILD | WS_VISIBLE,
        450,
        (BUTTON_POS_Y as i32),
        (BUTTON_WIDTH as i32),
        (BUTTON_HEIGHT as i32),
        hwnd,
        IDC_NEXT as HMENU,
        g_hInstance,
        core::ptr::null_mut(),
    );
    SendMessage(
        hButtonNext,
        WM_SETFONT,
        GetStockObject((DEFAULT_GUI_FONT as i32)) as WPARAM,
        TRUE as LPARAM,
    );

    EnableWindow(hButtonPrev, (FALSE) as BOOL);
    if g_NumFonts <= 1 {
        EnableWindow(hButtonNext, (FALSE) as BOOL);
    }

    /* Init the display window with the font name */
    g_FontIndex = 0;
    SendMessage(
        hDisplay,
        FVM_SETTYPEFACE,
        0,
        (core::ptr::addr_of_mut!(g_LogFonts) as *mut LOGFONTW).offset(g_FontIndex as isize) as LPARAM,
    );
    ShowWindow(hDisplay, (SW_SHOWNORMAL) as i32);

    if g_FontPrint != 0 {
        PostMessage(hwnd, WM_COMMAND, (IDC_PRINT as WPARAM), 0);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn MainWnd_OnSize(hwnd: HWND) -> LRESULT {
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hInstall: HWND = core::ptr::null_mut();
    let mut hPrint: HWND = core::ptr::null_mut();
    let mut hPrev: HWND = core::ptr::null_mut();
    let mut hNext: HWND = core::ptr::null_mut();
    let mut hDisplay: HWND = core::ptr::null_mut();
    let mut hDWP: HDWP = core::ptr::null_mut();

    GetClientRect(hwnd, core::ptr::addr_of_mut!(rc));

    hDWP = BeginDeferWindowPos(5);

    hInstall = GetDlgItem(hwnd, (IDC_INSTALL as i32));
    if !hDWP.is_null() {
        hDWP = DeferWindowPos(
            hDWP,
            hInstall,
            core::ptr::null_mut(),
            (BUTTON_POS_X as i32),
            (BUTTON_POS_Y as i32),
            (BUTTON_WIDTH as i32),
            (BUTTON_HEIGHT as i32),
            SWP_NOZORDER as UINT,
        );
    }

    hPrint = GetDlgItem(hwnd, (IDC_PRINT as i32));
    if !hDWP.is_null() {
        hDWP = DeferWindowPos(
            hDWP,
            hPrint,
            core::ptr::null_mut(),
            (BUTTON_POS_X + BUTTON_WIDTH + BUTTON_PADDING) as i32,
            (BUTTON_POS_Y as i32),
            (BUTTON_WIDTH as i32),
            (BUTTON_HEIGHT as i32),
            SWP_NOZORDER as UINT,
        );
    }

    hPrev = GetDlgItem(hwnd, (IDC_PREV as i32));
    if !hDWP.is_null() {
        hDWP = DeferWindowPos(
            hDWP,
            hPrev,
            core::ptr::null_mut(),
            (rc.right
                - (BUTTON_WIDTH * 2 + BUTTON_PADDING + BUTTON_POS_X) as LONG)
                as core::ffi::c_int,
            (BUTTON_POS_Y as i32),
            (BUTTON_WIDTH as i32),
            (BUTTON_HEIGHT as i32),
            SWP_NOZORDER as UINT,
        );
    }

    hNext = GetDlgItem(hwnd, (IDC_NEXT as i32));
    if !hDWP.is_null() {
        hDWP = DeferWindowPos(
            hDWP,
            hNext,
            core::ptr::null_mut(),
            (rc.right - (BUTTON_WIDTH + BUTTON_POS_X) as LONG)
                as core::ffi::c_int,
            (BUTTON_POS_Y as i32),
            (BUTTON_WIDTH as i32),
            (BUTTON_HEIGHT as i32),
            SWP_NOZORDER as UINT,
        );
    }

    hDisplay = GetDlgItem(hwnd, (IDC_DISPLAY as i32));
    if !hDWP.is_null() {
        hDWP = DeferWindowPos(
            hDWP,
            hDisplay,
            core::ptr::null_mut(),
            0,
            (HEADER_SIZE as i32),
            rc.right as core::ffi::c_int,
            (rc.bottom - HEADER_SIZE as LONG) as core::ffi::c_int,
            SWP_NOZORDER as UINT,
        );
    }

    EndDeferWindowPos(hDWP);

    InvalidateRect(hwnd, core::ptr::null::<RECT>(), (TRUE) as BOOL);

    return 0 as LRESULT;
}

unsafe extern "C" fn MainWnd_OnPaint(hwnd: HWND) -> LRESULT {
    let mut hDC: HDC;
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut rc: RECT = core::mem::zeroed();

    hDC = BeginPaint(hwnd, core::ptr::addr_of_mut!(ps));
    GetClientRect(hwnd, core::ptr::addr_of_mut!(rc));
    rc.top = (HEADER_SIZE - 2) as i32;
    rc.bottom = (HEADER_SIZE as i32);
    FillRect(
        hDC,
        core::ptr::addr_of_mut!(rc) as LPCRECT,
        GetStockObject((GRAY_BRUSH as i32)) as HBRUSH,
    );
    EndPaint(hwnd, core::ptr::addr_of_mut!(ps));
    return 0;
}

unsafe extern "C" fn MainWnd_OnInstall(hwnd: HWND) -> LRESULT {
    let mut szFullName: [WCHAR; 64] = [0; 64];

    let mut szSrcPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szDestPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pszFileName: PWSTR = core::ptr::null_mut();
    let mut res: LONG;
    let mut hKey: HKEY = core::ptr::null_mut();

    SendDlgItemMessage(
        hwnd,
        (IDC_DISPLAY as i32),
        FVM_GETFULLNAME,
        64,
        szFullName.as_mut_ptr() as LPARAM,
    );

    /* First, we have to find out if the font still exists */
    if GetFileAttributes(g_fileName) == (INVALID_FILE_ATTRIBUTES) as DWORD {
        /* Fail, if the source file does not exist */
        ErrorMsgBox(
            core::ptr::null_mut(),
            IDS_ERROR_NOFONT,
            g_fileName,
        );
        return -((1) as LRESULT);
    }

    /* Build the full destination file name */
    GetFullPathNameW(
        g_fileName,
        MAX_PATH,
        szSrcPath.as_mut_ptr(),
        &raw mut pszFileName,
    );

    GetWindowsDirectoryW(szDestPath.as_mut_ptr(), MAX_PATH);
    wcscat(szDestPath.as_mut_ptr(), RUSTCONV_WSTR_99F5D5B7A782.as_ptr());
    wcscat(szDestPath.as_mut_ptr(), pszFileName);

    /* Check if the file already exists */
    if GetFileAttributesW(szDestPath.as_mut_ptr()) != (INVALID_FILE_ATTRIBUTES) as DWORD {
        ErrorMsgBox(hwnd, IDS_ERROR_ISINSTALLED);
        return (0) as LRESULT;
    }

    /* Copy the font file */
    if CopyFileW(
        g_fileName,
        szDestPath.as_mut_ptr(),
        (TRUE) as BOOL,
    ) == 0
    {
        ErrorMsgBox(hwnd, IDS_ERROR_FONTCPY);
        return -((1) as LRESULT);
    }

    /* Open the fonts key */
    res = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        RUSTCONV_WSTR_06491E66FB66.as_ptr(),
        0,
        KEY_ALL_ACCESS,
        &raw mut hKey,
    ) as LONG;
    if res != (ERROR_SUCCESS) as LONG {
        ErrorMsgBox(hwnd, IDS_ERROR_OPENKEY);
        return -((1) as LRESULT);
    }

    /* Register the font */
    res = RegSetValueExW(
        hKey,
        szFullName.as_mut_ptr(),
        0,
        REG_SZ,
        pszFileName as LPBYTE,
        ((wcslen(pszFileName) as DWORD + (1) as DWORD)
            * core::mem::size_of::<WCHAR>() as DWORD),
    ) as LONG;
    if res != (ERROR_SUCCESS) as LONG {
        ErrorMsgBox(hwnd, IDS_ERROR_REGISTER);
        RegCloseKey(hKey);
        return -((1) as LRESULT);
    }

    /* Close the fonts key */
    RegCloseKey(hKey);

    /* Broadcast WM_FONTCHANGE message */
    SendMessageW(
        HWND_BROADCAST,
        WM_FONTCHANGE,
        0,
        0,
    );

    /* if all of this goes correctly, message the user about success */
    SuccessMsgBox(hwnd, IDS_COMPLETED);

    return (0) as LRESULT;
}

unsafe extern "C" fn MainWnd_OnPrev(hwnd: HWND) -> LRESULT {
    let mut hDisplay: HWND = core::ptr::null_mut();
    if g_FontIndex > 0 {
        g_FontIndex -= 1;
        EnableWindow(GetDlgItem(hwnd, (IDC_NEXT as i32)), (TRUE) as BOOL);
        if g_FontIndex == 0 {
            EnableWindow(GetDlgItem(hwnd, (IDC_PREV as i32)), (FALSE) as BOOL);
        }

        hDisplay = GetDlgItem(hwnd, (IDC_DISPLAY as i32));
        SendMessage(
            hDisplay,
            FVM_SETTYPEFACE,
            0,
            (core::ptr::addr_of_mut!(g_LogFonts) as *mut LOGFONTW).offset(g_FontIndex as isize) as LPARAM,
        );
        InvalidateRect(hDisplay, core::ptr::null_mut(), (TRUE) as BOOL);
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn MainWnd_OnNext(hwnd: HWND) -> LRESULT {
    let mut hDisplay: HWND = core::ptr::null_mut();
    if g_FontIndex + 1 < g_NumFonts {
        g_FontIndex += 1;
        EnableWindow(GetDlgItem(hwnd, (IDC_PREV as i32)), (TRUE) as BOOL);
        if g_FontIndex == g_NumFonts - 1 {
            EnableWindow(GetDlgItem(hwnd, (IDC_NEXT as i32)), (FALSE) as BOOL);
        }

        hDisplay = GetDlgItem(hwnd, (IDC_DISPLAY as i32));
        SendMessage(hDisplay, FVM_SETTYPEFACE, 0, core::ptr::addr_of_mut!(g_LogFonts[g_FontIndex as usize]) as LPARAM);
        InvalidateRect(hDisplay, core::ptr::null_mut(), (TRUE) as BOOL);
    }
    return 0 as LRESULT;
}
#[no_mangle]
pub unsafe extern "system" fn MainWndProc(
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    match message {
        WM_CREATE => {
            return MainWnd_OnCreate(hwnd);
        }
        WM_PAINT => {
            return MainWnd_OnPaint(hwnd);
        }
        WM_SIZE => {
            return MainWnd_OnSize(hwnd);
        }
        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_INSTALL => {
                    return MainWnd_OnInstall(hwnd);
                }
                IDC_PRINT => {
                    return Display_OnPrint(hwnd);
                }
                IDC_PREV => {
                    return MainWnd_OnPrev(hwnd);
                }
                IDC_NEXT => {
                    return MainWnd_OnNext(hwnd);
                }
                _ => {}
            }
        }
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => {
            return DefWindowProcW(hwnd, message, wParam, lParam);
        }
    }

    return 0;
}
