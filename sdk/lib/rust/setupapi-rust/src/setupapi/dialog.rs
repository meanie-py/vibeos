use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct promptdisk_params {
    pub DialogTitle: PCWSTR,
    pub DiskName: PCWSTR,
    pub PathToSource: PCWSTR,
    pub FileSought: PCWSTR,
    pub TagFile: PCWSTR,
    pub DiskPromptStyle: DWORD,
    pub PathBuffer: PWSTR,
    pub PathBufferSize: DWORD,
    pub PathRequiredSize: PDWORD,
}

unsafe extern "C" fn promptdisk_init(hwnd: HWND, params: *mut promptdisk_params) {
    SetWindowLongPtrW(hwnd, DWLP_USER, params as LONG_PTR);

    if !(*params).DialogTitle.is_null() {
        SetWindowTextW(hwnd, (*params).DialogTitle);
    }
    if !(*params).PathToSource.is_null() {
        SetDlgItemTextW(hwnd, (IDC_PATH as i32), (*params).PathToSource);
    }

    if ((*params).DiskPromptStyle & IDF_OEMDISK) == 0 {
        let mut message: [WCHAR; (256 + 2 * MAX_PATH) as _] =
            [0; (256 + 2 * MAX_PATH) as _];
        let mut format: [WCHAR; 256] = [0; 256];
        let mut unknown: [WCHAR; 256] = [0; 256];
        let mut args: [DWORD_PTR; 2] = [0; 2];
        LoadStringW(
            hInstance,
            IDS_PROMPTDISK,
            format.as_mut_ptr(),
            (core::mem::size_of_val(&format) / core::mem::size_of_val(&format[0]))
                as ::core::ffi::c_int,
        );

        args[0] = (*params).FileSought as DWORD_PTR;
        if !(*params).DiskName.is_null() {
            args[1] = (*params).DiskName as DWORD_PTR;
        } else {
            LoadStringW(
                hInstance,
                IDS_UNKNOWN,
                unknown.as_mut_ptr(),
                (core::mem::size_of_val(&unknown) / core::mem::size_of_val(&unknown[0]))
                    as ::core::ffi::c_int,
            );
            args[1] = unknown.as_mut_ptr() as DWORD_PTR;
        }
        FormatMessageW(
            (FORMAT_MESSAGE_FROM_STRING | FORMAT_MESSAGE_ARGUMENT_ARRAY) as DWORD,
            format.as_mut_ptr() as LPCVOID,
            0,
            0,
            message.as_mut_ptr(),
            (core::mem::size_of_val(&message) / core::mem::size_of_val(&message[0])) as DWORD,
            // C: `(__ms_va_list*)args`, an argument ARRAY (FORMAT_MESSAGE_ARGUMENT_ARRAY), not
            // varargs: the prelude's `*mut va_list`, never core's unstable VaListImpl.
            args.as_mut_ptr() as *mut va_list,
        );
        SetDlgItemTextW(hwnd, (IDC_FILENEEDED as i32), message.as_ptr());

        LoadStringW(
            hInstance,
            IDS_INFO,
            message.as_mut_ptr(),
            (core::mem::size_of_val(&message) / core::mem::size_of_val(&message[0]))
                as ::core::ffi::c_int,
        );
        SetDlgItemTextW(hwnd, (IDC_INFO as i32), message.as_ptr());
        LoadStringW(
            hInstance,
            IDS_COPYFROM,
            message.as_mut_ptr(),
            (core::mem::size_of_val(&message) / core::mem::size_of_val(&message[0]))
                as ::core::ffi::c_int,
        );
        SetDlgItemTextW(hwnd, (IDC_COPYFROM as i32), message.as_ptr());
    }
    if ((*params).DiskPromptStyle & IDF_NOBROWSE) != 0 {
        ShowWindow(GetDlgItem(hwnd, (IDC_RUNDLG_BROWSE as i32)), (SW_HIDE as i32));
    }
}

unsafe extern "C" fn promptdisk_ok(hwnd: HWND, params: *mut promptdisk_params) {
    let mut requiredSize: core::ffi::c_int;
    let mut aux: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    GetWindowTextW(
        GetDlgItem(hwnd, (IDC_PATH as i32)),
        core::ptr::addr_of_mut!(aux) as PWSTR,
        (MAX_PATH as i32),
    );
    requiredSize = strlenW(core::ptr::addr_of_mut!(aux) as PCWSTR) as core::ffi::c_int + 1;

    if !(*params).PathRequiredSize.is_null() {
        *(*params).PathRequiredSize = requiredSize as DWORD;
    }
    if (*params).PathBuffer.is_null() {
        EndDialog(hwnd, (NO_ERROR) as INT_PTR);
        return;
    }
    if (requiredSize as DWORD) > (*params).PathBufferSize {
        EndDialog(hwnd, (DPROMPT_BUFFERTOOSMALL) as INT_PTR);
        return;
    }
    strcpyW((*params).PathBuffer, core::ptr::addr_of_mut!(aux) as PCWSTR);

    EndDialog(hwnd, (DPROMPT_SUCCESS) as INT_PTR);
}

unsafe extern "C" fn promptdisk_browse(hwnd: HWND, params: *mut promptdisk_params) {
    let mut ofn: OPENFILENAMEW = core::mem::zeroed();
    ZeroMemory!(
        core::ptr::addr_of_mut!(ofn) as *mut core::ffi::c_void,
        core::mem::size_of::<OPENFILENAMEW>(),
    );

    ofn.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
    ofn.Flags = (OFN_EXPLORER | OFN_HIDEREADONLY | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST) as DWORD;
    ofn.hwndOwner = hwnd;
    ofn.nMaxFile = MAX_PATH as DWORD;
    ofn.lpstrFile = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (MAX_PATH as SIZE_T) * (core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as PWSTR;
    strcpyW(ofn.lpstrFile, (*params).FileSought);

    if GetOpenFileNameW(core::ptr::addr_of_mut!(ofn)) != 0 {
        let mut last_slash: PWSTR = strrchrW(ofn.lpstrFile, '\\' as WCHAR);
        if !last_slash.is_null() {
            *last_slash = 0 as WCHAR;
        }
        SetDlgItemTextW(hwnd, (IDC_PATH as i32), ofn.lpstrFile);
    }
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        ofn.lpstrFile as PVOID,
    );
}

unsafe extern "system" fn promptdisk_proc(hwnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match msg {
        WM_INITDIALOG => {
            promptdisk_init(hwnd, lParam as *mut promptdisk_params);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => match wParam {
            __c if __c == IDOK as _ => {
                let mut params: *mut promptdisk_params =
                    GetWindowLongPtrW(hwnd, DWLP_USER as ::core::ffi::c_int)
                        as *mut promptdisk_params;
                promptdisk_ok(hwnd, params);
                return TRUE as INT_PTR;
            }
            __c if __c == IDCANCEL as _ => {
                EndDialog(hwnd, DPROMPT_CANCEL as INT_PTR);
                return TRUE as INT_PTR;
            }
            __c if __c == IDC_RUNDLG_BROWSE as _ => {
                let mut params_0: *mut promptdisk_params =
                    GetWindowLongPtrW(hwnd, DWLP_USER as ::core::ffi::c_int)
                        as *mut promptdisk_params;
                promptdisk_browse(hwnd, params_0);
                return TRUE as INT_PTR;
            }
            _ => {}
        },
        _ => {}
    }
    return FALSE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn SetupPromptForDiskA(
    hwndParent: HWND,
    DialogTitle: PCSTR,
    DiskName: PCSTR,
    PathToSource: PCSTR,
    FileSought: PCSTR,
    TagFile: PCSTR,
    DiskPromptStyle: DWORD,
    PathBuffer: PSTR,
    PathBufferSize: DWORD,
    PathRequiredSize: PDWORD,
) -> UINT {
    let mut DialogTitleW: *mut WCHAR = core::ptr::null_mut();
    let mut DiskNameW: *mut WCHAR = core::ptr::null_mut();
    let mut PathToSourceW: *mut WCHAR = core::ptr::null_mut();
    let mut FileSoughtW: *mut WCHAR = core::ptr::null_mut();
    let mut TagFileW: *mut WCHAR = core::ptr::null_mut();
    let mut PathBufferW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ret: UINT = 0;
    let mut length: UINT = 0;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/dialog.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupPromptForDiskA\0" as *const u8 as *const ::core::ffi::c_char,
            185 as ::core::ffi::c_int,
            b"%p, %s, %s, %s, %s, %s, 0x%08x, %p, %d, %p\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwndParent,
            debugstr_a(DialogTitle as *const ::core::ffi::c_char),
            debugstr_a(DiskName as *const ::core::ffi::c_char),
            debugstr_a(PathToSource as *const ::core::ffi::c_char),
            debugstr_a(FileSought as *const ::core::ffi::c_char),
            debugstr_a(TagFile as *const ::core::ffi::c_char),
            DiskPromptStyle,
            PathBuffer,
            PathBufferSize,
            PathRequiredSize,
        );
    }

    DialogTitleW = strdupAtoW(DialogTitle as *const ::core::ffi::c_char) as *mut WCHAR;
    DiskNameW = strdupAtoW(DiskName as *const ::core::ffi::c_char) as *mut WCHAR;
    PathToSourceW = strdupAtoW(PathToSource as *const ::core::ffi::c_char) as *mut WCHAR;
    FileSoughtW = strdupAtoW(FileSought as *const ::core::ffi::c_char) as *mut WCHAR;
    TagFileW = strdupAtoW(TagFile as *const ::core::ffi::c_char) as *mut WCHAR;

    ret = SetupPromptForDiskW(
        hwndParent,
        DialogTitleW as PCWSTR,
        DiskNameW as PCWSTR,
        PathToSourceW as PCWSTR,
        FileSoughtW as PCWSTR,
        TagFileW as PCWSTR,
        DiskPromptStyle,
        core::ptr::addr_of_mut!(PathBufferW) as PWSTR,
        MAX_PATH as DWORD,
        PathRequiredSize,
    );

    HeapFree(GetProcessHeap(), 0 as DWORD, DialogTitleW as PVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, DiskNameW as PVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, PathToSourceW as PVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, FileSoughtW as PVOID);
    HeapFree(GetProcessHeap(), 0 as DWORD, TagFileW as PVOID);

    if ret == DPROMPT_SUCCESS as UINT {
        length = WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            core::ptr::addr_of_mut!(PathBufferW) as *mut WCHAR as LPCWSTR,
            -(1 as ::core::ffi::c_int),
            core::ptr::null_mut::<CHAR>(),
            0 as ::core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) as UINT;
        if !PathRequiredSize.is_null() {
            *PathRequiredSize = length as DWORD;
        }
        if !PathBuffer.is_null() {
            if length as DWORD > PathBufferSize {
                return DPROMPT_BUFFERTOOSMALL as UINT;
            }
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!(PathBufferW) as *mut WCHAR as LPCWSTR,
                -(1 as ::core::ffi::c_int),
                PathBuffer as LPSTR,
                length as ::core::ffi::c_int,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            );
        }
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupPromptForDiskW(
    hwndParent: HWND,
    DialogTitle: PCWSTR,
    DiskName: PCWSTR,
    PathToSource: PCWSTR,
    FileSought: PCWSTR,
    TagFile: PCWSTR,
    DiskPromptStyle: DWORD,
    PathBuffer: PWSTR,
    PathBufferSize: DWORD,
    PathRequiredSize: PDWORD,
) -> UINT {
    let mut params: promptdisk_params = core::mem::zeroed();
    let mut ret: UINT;

    if FileSought.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return DPROMPT_CANCEL;
    }

    if !PathToSource.is_null() && (DiskPromptStyle & IDF_CHECKFIRST) != 0 {
        static format: [u16; 6] = [0x0025u16, 0x0073, 0x005C, 0x0025, 0x0073, 0x0000];
        let mut filepath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

        if (strlenW(PathToSource) as usize)
            + (1 as usize)
            + (strlenW(FileSought) as usize)
            // Divergence: the C compares this CHARACTER count with sizeof(filepath), 520
            // bytes, so a 260..519-char path passes, snprintfW(.., MAX_PATH, ..) truncates
            // without a NUL, and GetFileAttributesW reads past the buffer.
            < (MAX_PATH) as usize
        {
            snprintfW(
                core::ptr::addr_of_mut!(filepath) as *mut WCHAR,
                (MAX_PATH) as _,
                core::ptr::addr_of!(format) as PCWSTR,
                PathToSource,
                FileSought,
            );

            if GetFileAttributesW(core::ptr::addr_of_mut!(filepath) as *mut WCHAR as PCWSTR)
                != INVALID_FILE_ATTRIBUTES
            {
                if !PathRequiredSize.is_null() {
                    *PathRequiredSize =
                        ((strlenW(PathToSource) as usize) + (1 as usize))
                            as DWORD;
                }

                if PathBuffer.is_null() {
                    return DPROMPT_SUCCESS;
                }

                if (PathBufferSize as usize)
                    >= ((strlenW(PathToSource) as usize) + (1 as usize))
                {
                    strcpyW(PathBuffer, PathToSource);
                    return DPROMPT_SUCCESS;
                } else {
                    return DPROMPT_BUFFERTOOSMALL;
                }
            }
        }
    }

    params.DialogTitle = DialogTitle;
    params.DiskName = DiskName;
    params.PathToSource = PathToSource;
    params.FileSought = FileSought;
    params.TagFile = TagFile;
    params.DiskPromptStyle = DiskPromptStyle;
    params.PathBuffer = PathBuffer;
    params.PathBufferSize = PathBufferSize;
    params.PathRequiredSize = PathRequiredSize;

    ret = DialogBoxParamW(
        hInstance,
        MAKEINTRESOURCEW!(IDPROMPTFORDISK),
        hwndParent,
        Some(promptdisk_proc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR),
        core::ptr::addr_of_mut!(params) as LPARAM,
    ) as UINT;

    if ret == DPROMPT_CANCEL {
        SetLastError(ERROR_CANCELLED);
    }
    return ret;
}
