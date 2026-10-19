use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut source_disks_names: [WCHAR; 17] = [
    'S' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'k' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'N' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'm' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut source_disks_files: [WCHAR; 17] = [
    'S' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'k' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'F' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static infW: [u16; 6] = [0x005Cu16, 0x0069, 0x006E, 0x0066, 0x005C, 0x0000];
static system32W: [u16; 11] = [
    0x005Cu16, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x005C,
    0x0000,
];

unsafe extern "C" fn fill_inf_info(
    inf: HINF,
    buffer: PSP_INF_INFORMATION,
    size: DWORD,
    required: *mut DWORD,
) -> BOOL {
    let filename: LPCWSTR = PARSER_get_inf_filename(inf) as LPCWSTR;
    let total_size: DWORD = (core::mem::offset_of!(SP_INF_INFORMATION, VersionData) as DWORD)
        + ((lstrlenW(filename) + 1) as DWORD)
            * (core::mem::size_of::<WCHAR>() as DWORD);

    if !required.is_null() {
        *required = total_size;
    }

    /* FIXME: we need to parse the INF file to find the correct version info */
    if !buffer.is_null() {
        if size < total_size {
            SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
            return (FALSE) as BOOL;
        }
        (*buffer).InfStyle = INF_STYLE_WIN4 as DWORD;
        (*buffer).InfCount = 1 as DWORD;
        /* put the filename in buffer->VersionData */
        lstrcpyW(
            (core::ptr::addr_of_mut!((*buffer).VersionData) as *mut BYTE)
                .offset(0 as isize) as *mut BYTE as LPWSTR,
            filename,
        );
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn search_for_inf(InfSpec: LPCVOID, SearchControl: DWORD) -> HINF {
    let mut hInf: HINF = INVALID_HANDLE_VALUE;
    let mut inf_path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if SearchControl == INFINFO_REVERSE_DEFAULT_SEARCH as DWORD {
        GetWindowsDirectoryW(core::ptr::addr_of_mut!(inf_path) as LPWSTR, MAX_PATH as UINT);
        lstrcatW(core::ptr::addr_of_mut!(inf_path) as LPWSTR, core::ptr::addr_of!(system32W) as LPCWSTR);
        lstrcatW(core::ptr::addr_of_mut!(inf_path) as LPWSTR, InfSpec as LPCWSTR);

        hInf = SetupOpenInfFileW(
            core::ptr::addr_of_mut!(inf_path) as *mut WCHAR as PCWSTR,
            core::ptr::null::<WCHAR>(),
            (INF_STYLE_OLDNT | INF_STYLE_WIN4) as DWORD,
            core::ptr::null_mut(),
        );
        if hInf != INVALID_HANDLE_VALUE {
            return hInf;
        }

        GetWindowsDirectoryW(core::ptr::addr_of_mut!(inf_path) as LPWSTR, MAX_PATH as UINT);
        // Divergence: the C's lstrcpyW throws the Windows directory away and opens
        // `\inf\<spec>`; the system32 search above shows lstrcatW was meant (Wine has it).
        lstrcatW(core::ptr::addr_of_mut!(inf_path) as LPWSTR, core::ptr::addr_of!(infW) as LPCWSTR);
        lstrcatW(core::ptr::addr_of_mut!(inf_path) as LPWSTR, InfSpec as LPCWSTR);

        return SetupOpenInfFileW(
            core::ptr::addr_of_mut!(inf_path) as *mut WCHAR as PCWSTR,
            core::ptr::null::<WCHAR>(),
            (INF_STYLE_OLDNT | INF_STYLE_WIN4) as DWORD,
            core::ptr::null_mut(),
        );
    }

    return INVALID_HANDLE_VALUE;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetInfInformationA(InfSpec: LPCVOID, SearchControl: DWORD, ReturnBuffer: PSP_INF_INFORMATION, ReturnBufferSize: DWORD, RequiredSize: PDWORD) -> BOOL {
    let mut inf: LPWSTR = InfSpec as LPWSTR;
    let mut len: DWORD = 0;
    let mut ret: BOOL = 0;
    if !InfSpec.is_null() && SearchControl >= INFINFO_INF_NAME_IS_ABSOLUTE {
        len = MultiByteToWideChar(CP_ACP, 0, InfSpec as LPCSTR, -1, ::core::ptr::null_mut::<WCHAR>(), 0) as DWORD;
        inf = HeapAlloc(GetProcessHeap(), 0, (len as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;
        if inf.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            return (FALSE) as BOOL;
        }
        MultiByteToWideChar(CP_ACP, 0, InfSpec as LPCSTR, -1, inf, len as ::core::ffi::c_int);
    }
    ret = SetupGetInfInformationW(inf as LPCVOID, SearchControl, ReturnBuffer, ReturnBufferSize, RequiredSize);
    if SearchControl >= INFINFO_INF_NAME_IS_ABSOLUTE {
        HeapFree(GetProcessHeap(), 0, inf as PVOID);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetInfInformationW(
    InfSpec: LPCVOID,
    SearchControl: DWORD,
    ReturnBuffer: PSP_INF_INFORMATION,
    ReturnBufferSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    let mut inf: HINF = core::ptr::null_mut::<core::ffi::c_void>();
    let mut ret: BOOL;
    let mut infSize: DWORD = 0;

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/query.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"SetupGetInfInformationW\0" as *const u8 as *const core::ffi::c_char,
            133 as core::ffi::c_int,
            b"(%p, %d, %p, %d, %p)\n\0" as *const u8 as *const core::ffi::c_char,
            InfSpec,
            SearchControl,
            ReturnBuffer,
            ReturnBufferSize,
            RequiredSize,
        );
    }

    if InfSpec.is_null() {
        if SearchControl == INFINFO_INF_SPEC_IS_HINF {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
        } else {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        }
        return (FALSE) as BOOL;
    }

    match SearchControl {
        INFINFO_INF_SPEC_IS_HINF => {
            inf = InfSpec as HINF;
        }
        INFINFO_INF_NAME_IS_ABSOLUTE | INFINFO_DEFAULT_SEARCH => {
            inf = SetupOpenInfFileW(
                InfSpec as PCWSTR,
                core::ptr::null::<WCHAR>(),
                (INF_STYLE_OLDNT | INF_STYLE_WIN4) as DWORD,
                core::ptr::null_mut::<core::ffi::c_uint>(),
            );
        }
        INFINFO_REVERSE_DEFAULT_SEARCH => {
            inf = search_for_inf(InfSpec, SearchControl);
        }
        INFINFO_INF_PATH_LIST_SEARCH => {
            if (*__wine_dbch___default).flags as core::ffi::c_int
                & (1 as core::ffi::c_int) << __WINE_DBCL_FIXME as core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/setupapi/query.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"SetupGetInfInformationW\0" as *const u8 as *const core::ffi::c_char,
                    159 as core::ffi::c_int,
                    b"Unhandled search control: %d\n\0" as *const u8 as *const core::ffi::c_char,
                    SearchControl,
                );
            }

            if !RequiredSize.is_null() {
                *RequiredSize = 0 as DWORD;
            }

            return (FALSE) as BOOL;
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            return (FALSE) as BOOL;
        }
    }

    if inf == INVALID_HANDLE_VALUE {
        SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
        return (FALSE) as BOOL;
    }

    ret = fill_inf_info(
        inf,
        ReturnBuffer,
        ReturnBufferSize,
        core::ptr::addr_of_mut!(infSize),
    );
    if ReturnBuffer.is_null() && ReturnBufferSize >= infSize {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        ret = (FALSE as i32);
    }
    if !RequiredSize.is_null() {
        *RequiredSize = infSize;
    }

    if SearchControl >= INFINFO_INF_NAME_IS_ABSOLUTE {
        SetupCloseInfFile(inf);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueryInfFileInformationA(
    InfInformation: PSP_INF_INFORMATION,
    InfIndex: UINT,
    ReturnBuffer: PSTR,
    ReturnBufferSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    let mut filenameW: LPWSTR = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let mut ret: BOOL;

    ret = SetupQueryInfFileInformationW(InfInformation, InfIndex, core::ptr::null_mut(), 0, core::ptr::addr_of_mut!(size));
    if ret == 0 {
        return (FALSE) as BOOL;
    }

    filenameW = HeapAlloc(GetProcessHeap(), 0, (size as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;

    ret = SetupQueryInfFileInformationW(InfInformation, InfIndex,
                                        filenameW as PWSTR, size, core::ptr::addr_of_mut!(size));
    if ret == 0 {
        HeapFree(GetProcessHeap(), 0, filenameW as PVOID);
        return (FALSE) as BOOL;
    }

    if !RequiredSize.is_null() {
        *RequiredSize = size;
    }

    if ReturnBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, filenameW as PVOID);
        if ReturnBufferSize != 0 {
            SetLastError(ERROR_INVALID_PARAMETER);
            return (FALSE) as BOOL;
        }

        return (TRUE) as BOOL;
    }

    if size > ReturnBufferSize {
        HeapFree(GetProcessHeap(), 0, filenameW as PVOID);
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return (FALSE) as BOOL;
    }

    WideCharToMultiByte(CP_ACP as UINT, 0 as DWORD, filenameW as LPCWSTR, -1, ReturnBuffer as LPSTR, size as ::core::ffi::c_int, core::ptr::null::<CHAR>(), core::ptr::null_mut::<BOOL>());
    HeapFree(GetProcessHeap(), 0, filenameW as PVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueryInfFileInformationW(
    InfInformation: PSP_INF_INFORMATION,
    InfIndex: UINT,
    ReturnBuffer: PWSTR,
    ReturnBufferSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    let mut len: DWORD;
    let mut ptr: LPWSTR;

    if InfInformation.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    if InfIndex != 0 {
        FIXME!("Appended INF files are not handled\n");
    }

    ptr = core::ptr::addr_of_mut!((*InfInformation).VersionData) as *mut WCHAR as LPWSTR;
    len = lstrlenW(ptr as LPCWSTR) as DWORD;

    if !RequiredSize.is_null() {
        *RequiredSize = len.wrapping_add(1);
    }

    if ReturnBuffer.is_null() {
        return (TRUE) as BOOL;
    }

    // Divergence: the C tests `< len`, letting a buffer of exactly len WCHARs through to an
    // lstrcpyW that writes len + 1 -- the same function reports RequiredSize = len + 1.
    if ReturnBufferSize < len + 1 {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return (FALSE) as BOOL;
    }

    lstrcpyW(ReturnBuffer as LPWSTR, ptr as LPCWSTR);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetSourceFileLocationA(
    hinf: HINF,
    context: PINFCONTEXT,
    filename: PCSTR,
    source_id: PUINT,
    buffer: PSTR,
    buffer_size: DWORD,
    required_size: PDWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut filenameW: *mut WCHAR = core::ptr::null_mut();
    let mut bufferW: *mut WCHAR = core::ptr::null_mut();
    let mut required: DWORD = 0;
    let mut size: INT = 0;

    if !filename.is_null() && *filename as ::core::ffi::c_int != 0 {
        filenameW = strdupAtoW(filename);
        if filenameW.is_null() {
            return (FALSE) as BOOL;
        }
    }

    'done: {
        if SetupGetSourceFileLocationW(
            hinf,
            context,
            filenameW as PCWSTR,
            source_id,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(required),
        ) == 0
        {
            break 'done;
        }

        bufferW = HeapAlloc(
            GetProcessHeap(),
            0,
            (required as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        if bufferW.is_null() {
            break 'done;
        }

        if SetupGetSourceFileLocationW(
            hinf,
            context,
            filenameW as PCWSTR,
            source_id,
            bufferW as PWSTR,
            required,
            core::ptr::null_mut(),
        ) == 0
        {
            break 'done;
        }

        size = WideCharToMultiByte(
            CP_ACP,
            0,
            bufferW as LPCWSTR,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as INT;
        if !required_size.is_null() {
            *required_size = size as DWORD;
        }

        if !buffer.is_null() {
            if buffer_size >= size as DWORD {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    bufferW as LPCWSTR,
                    -1,
                    buffer as LPSTR,
                    buffer_size as ::core::ffi::c_int,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            } else {
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                break 'done;
            }
        }
        ret = (TRUE as i32);
    }

    HeapFree(
        GetProcessHeap(),
        0,
        filenameW as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        bufferW as PVOID,
    );
    return ret;
}

unsafe extern "C" fn get_source_id(mut hinf: HINF, mut context: PINFCONTEXT, mut filename: PCWSTR) -> LPWSTR {
    let mut Section: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut size: DWORD = 0;
    let mut source_id: LPWSTR = core::ptr::null_mut::<WCHAR>();
    if SetupDiGetActualSectionToInstallW(
        hinf,
        core::ptr::addr_of!(source_disks_files) as PCWSTR,
        core::ptr::addr_of_mut!(Section) as PWSTR,
        MAX_PATH as DWORD,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<PWSTR>(),
    ) == 0
    {
        return core::ptr::null_mut::<WCHAR>();
    }
    if SetupFindFirstLineW(
        hinf,
        core::ptr::addr_of_mut!(Section) as *mut WCHAR as PCWSTR,
        filename,
        context,
    ) == 0
        && SetupFindFirstLineW(
            hinf,
            core::ptr::addr_of!(source_disks_files) as PCWSTR,
            filename,
            context,
        ) == 0
    {
        return core::ptr::null_mut::<WCHAR>();
    }
    if SetupGetStringFieldW(
        context,
        1 as DWORD,
        core::ptr::null_mut::<WCHAR>(),
        0 as DWORD,
        core::ptr::addr_of_mut!(size),
    ) == 0
    {
        return core::ptr::null_mut::<WCHAR>();
    }
    source_id = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (size as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if source_id.is_null() {
        return core::ptr::null_mut::<WCHAR>();
    }
    if SetupGetStringFieldW(
        context,
        1 as DWORD,
        source_id as PWSTR,
        size,
        core::ptr::null_mut::<DWORD>(),
    ) == 0
    {
        HeapFree(GetProcessHeap(), 0 as DWORD, source_id as PVOID);
        return core::ptr::null_mut::<WCHAR>();
    }
    if SetupDiGetActualSectionToInstallW(
        hinf,
        core::ptr::addr_of!(source_disks_names) as PCWSTR,
        core::ptr::addr_of_mut!(Section) as PWSTR,
        MAX_PATH as DWORD,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<PWSTR>(),
    ) == 0
    {
        HeapFree(GetProcessHeap(), 0 as DWORD, source_id as PVOID);
        return core::ptr::null_mut::<WCHAR>();
    }
    if SetupFindFirstLineW(
        hinf,
        core::ptr::addr_of_mut!(Section) as *mut WCHAR as PCWSTR,
        source_id as PCWSTR,
        context,
    ) == 0
        && SetupFindFirstLineW(
            hinf,
            core::ptr::addr_of!(source_disks_names) as PCWSTR,
            source_id as PCWSTR,
            context,
        ) == 0
    {
        HeapFree(GetProcessHeap(), 0 as DWORD, source_id as PVOID);
        return core::ptr::null_mut::<WCHAR>();
    }
    return source_id;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetSourceFileLocationW(
    hinf: HINF,
    mut context: PINFCONTEXT,
    filename: PCWSTR,
    source_id: PUINT,
    buffer: PWSTR,
    buffer_size: DWORD,
    required_size: PDWORD,
) -> BOOL {
    let mut ctx: INFCONTEXT = _INFCONTEXT {
        Inf: core::ptr::null_mut::<core::ffi::c_void>(),
        CurrentInf: core::ptr::null_mut::<core::ffi::c_void>(),
        Section: 0,
        Line: 0,
    };
    let mut end: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut source_id_str: *mut WCHAR = core::ptr::null_mut::<WCHAR>();

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/query.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"SetupGetSourceFileLocationW\0" as *const u8 as *const core::ffi::c_char,
            385 as core::ffi::c_int,
            b"%p, %p, %s, %p, %p, 0x%08x, %p\n\0" as *const u8 as *const core::ffi::c_char,
            hinf,
            context,
            debugstr_w(filename as *const WCHAR),
            source_id,
            buffer,
            buffer_size,
            required_size,
        );
    }

    if context.is_null() {
        context = core::ptr::addr_of_mut!(ctx) as PINFCONTEXT;
    }

    source_id_str = get_source_id(hinf, context, filename) as *mut WCHAR;
    if source_id_str.is_null() {
        return (FALSE) as BOOL;
    }

    *source_id = strtolW(source_id_str, core::ptr::addr_of_mut!(end), 10) as UINT;
    if end == source_id_str || *end != 0 {
        HeapFree(GetProcessHeap(), 0, source_id_str as PVOID);
        return (FALSE) as BOOL;
    }
    HeapFree(GetProcessHeap(), 0, source_id_str as PVOID);

    if SetupGetStringFieldW(context, 4, buffer, buffer_size, required_size) != 0 {
        return (TRUE) as BOOL;
    }

    if !required_size.is_null() {
        *required_size = 1;
    }
    if !buffer.is_null() {
        if buffer_size >= 1 {
            *buffer.offset(0 as isize) = 0;
        } else {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            return (FALSE) as BOOL;
        }
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetSourceInfoA(
    hinf: HINF,
    source_id: UINT,
    info: UINT,
    buffer: PSTR,
    buffer_size: DWORD,
    required_size: LPDWORD,
) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;
    let mut bufferW: *mut WCHAR = core::ptr::null_mut();
    let mut required: DWORD = 0;
    let mut size: INT = 0;

    if SetupGetSourceInfoW(
        hinf,
        source_id,
        info,
        core::ptr::null_mut(),
        0 as DWORD,
        core::ptr::addr_of_mut!(required),
    ) == 0
    {
        return FALSE as BOOL;
    }

    bufferW = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (required as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;
    if bufferW.is_null() {
        return FALSE as BOOL;
    }

    'done: {
        if SetupGetSourceInfoW(
            hinf,
            source_id,
            info,
            bufferW as PWSTR,
            required,
            core::ptr::null_mut(),
        ) == 0
        {
            break 'done;
        }

        size = WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            bufferW as LPCWSTR,
            -(1 as core::ffi::c_int),
            core::ptr::null_mut(),
            0 as core::ffi::c_int,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as INT;
        if !required_size.is_null() {
            *required_size = size as DWORD;
        }

        if !buffer.is_null() {
            if buffer_size >= size as DWORD {
                WideCharToMultiByte(
                    CP_ACP as UINT,
                    0 as DWORD,
                    bufferW as LPCWSTR,
                    -(1 as core::ffi::c_int),
                    buffer as LPSTR,
                    buffer_size as core::ffi::c_int,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            } else {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                break 'done;
            }
        }
        ret = TRUE as BOOL;
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        bufferW as PVOID,
    );
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetSourceInfoW(
    hinf: HINF,
    source_id: UINT,
    info: UINT,
    buffer: PWSTR,
    buffer_size: DWORD,
    required_size: LPDWORD,
) -> BOOL {
    let mut Section: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ctx: INFCONTEXT = core::mem::zeroed();
    let mut source_id_str: [WCHAR; 11] = [0; 11];
    static fmt: [u16; 3] = [0x0025u16, 0x0064, 0x0000];
    let mut index: DWORD;

    sprintfW(
        core::ptr::addr_of_mut!(source_id_str) as *mut WCHAR,
        core::ptr::addr_of!(fmt) as PCWSTR,
        source_id,
    );

    if SetupDiGetActualSectionToInstallW(
        hinf,
        core::ptr::addr_of!(source_disks_names) as PCWSTR,
        core::ptr::addr_of_mut!(Section) as PWSTR,
        MAX_PATH,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<PWSTR>(),
    ) == 0
    {
        return FALSE as BOOL;
    }

    if SetupFindFirstLineW(
        hinf,
        core::ptr::addr_of_mut!(Section) as *mut WCHAR as PCWSTR,
        core::ptr::addr_of_mut!(source_id_str) as *mut WCHAR as PCWSTR,
        core::ptr::addr_of_mut!(ctx),
    ) == 0
        && SetupFindFirstLineW(
            hinf,
            core::ptr::addr_of!(source_disks_names) as PCWSTR,
            core::ptr::addr_of_mut!(source_id_str) as *mut WCHAR as PCWSTR,
            core::ptr::addr_of_mut!(ctx),
        ) == 0
    {
        return FALSE as BOOL;
    }

    match info {
        SRCINFO_PATH => {
            index = 4 as DWORD;
        }
        SRCINFO_TAGFILE => {
            index = 2 as DWORD;
        }
        SRCINFO_DESCRIPTION => {
            index = 1 as DWORD;
        }
        _ => {
            WARN!("unknown info level: %d\n", info);
            return FALSE as BOOL;
        }
    }

    if SetupGetStringFieldW(
        core::ptr::addr_of_mut!(ctx),
        index,
        buffer,
        buffer_size,
        required_size as PDWORD,
    ) != 0
    {
        return TRUE as BOOL;
    }

    if !required_size.is_null() {
        *required_size = 1 as DWORD;
    }
    if !buffer.is_null() {
        if buffer_size >= 1 as DWORD {
            *buffer.offset(0 as isize) = 0 as WCHAR;
        } else {
            SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
            return FALSE as BOOL;
        }
    }
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetTargetPathA(
    hinf: HINF,
    context: PINFCONTEXT,
    section: PCSTR,
    buffer: PSTR,
    buffer_size: DWORD,
    required_size: PDWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut sectionW: *mut WCHAR = core::ptr::null_mut();
    let mut bufferW: *mut WCHAR = core::ptr::null_mut();
    let mut required: DWORD = 0;
    let mut size: INT = 0;

    if !section.is_null() {
        sectionW = strdupAtoW(section as *const i8) as *mut WCHAR;
        if sectionW.is_null() {
            return (FALSE) as BOOL;
        }
    }

    'done: {
        if SetupGetTargetPathW(
            hinf,
            context,
            sectionW as PCWSTR,
            core::ptr::null_mut::<WCHAR>(),
            0,
            core::ptr::addr_of_mut!(required),
        ) == 0
        {
            break 'done;
        }

        bufferW = HeapAlloc(
            GetProcessHeap(),
            0,
            (required as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        if bufferW.is_null() {
            break 'done;
        }

        if SetupGetTargetPathW(
            hinf,
            context,
            sectionW as PCWSTR,
            bufferW as PWSTR,
            required,
            core::ptr::null_mut::<DWORD>(),
        ) == 0
        {
            break 'done;
        }

        size = WideCharToMultiByte(
            CP_ACP as UINT,
            0,
            bufferW as LPCWSTR,
            -1,
            core::ptr::null_mut::<CHAR>(),
            0,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) as INT;
        if !required_size.is_null() {
            *required_size = size as DWORD;
        }

        if !buffer.is_null() {
            if buffer_size >= size as DWORD {
                WideCharToMultiByte(
                    CP_ACP as UINT,
                    0,
                    bufferW as LPCWSTR,
                    -1,
                    buffer as LPSTR,
                    buffer_size as ::core::ffi::c_int,
                    core::ptr::null::<CHAR>(),
                    core::ptr::null_mut::<BOOL>(),
                );
            } else {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                break 'done;
            }
        }
        ret = (TRUE as i32);
    }

    HeapFree(GetProcessHeap(), 0, sectionW as PVOID);
    HeapFree(GetProcessHeap(), 0, bufferW as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetTargetPathW(
    hinf: HINF,
    context: PINFCONTEXT,
    section: PCWSTR,
    buffer: PWSTR,
    buffer_size: DWORD,
    required_size: PDWORD,
) -> BOOL {
    static destination_dirs: [u16; 16] = [
    0x0044u16, 0x0065, 0x0073, 0x0074, 0x0069, 0x006E, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0044, 0x0069, 0x0072, 0x0073, 0x0000,
];
    static default_dest_dir: [u16; 15] = [
    0x0044u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0044, 0x0065, 0x0073,
    0x0074, 0x0044, 0x0069, 0x0072, 0x0000,
];

    let mut ctx: INFCONTEXT = core::mem::zeroed();
    let mut dir: *mut WCHAR = core::ptr::null_mut();
    let mut systemdir: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut size: u32;
    let mut ret: BOOL = (FALSE as i32);

    if !context.is_null() {
        ret = SetupFindFirstLineW(
            hinf,
            core::ptr::addr_of!(destination_dirs) as PCWSTR,
            core::ptr::null::<WCHAR>(),
            context,
        );
    } else if !section.is_null() {
        ret = SetupFindFirstLineW(
            hinf,
            core::ptr::addr_of!(destination_dirs) as PCWSTR,
            section,
            core::ptr::addr_of_mut!(ctx),
        );
        if ret == 0 {
            ret = SetupFindFirstLineW(
                hinf,
                core::ptr::addr_of!(destination_dirs) as PCWSTR,
                core::ptr::addr_of!(default_dest_dir) as PCWSTR,
                core::ptr::addr_of_mut!(ctx),
            );
        }
    }
    if ret == 0 || {
        dir = PARSER_get_dest_dir(if !context.is_null() {
            context
        } else {
            core::ptr::addr_of_mut!(ctx)
        });
        dir.is_null()
    } {
        GetSystemDirectoryW(
            core::ptr::addr_of_mut!(systemdir) as LPWSTR,
            MAX_PATH,
        );
        dir = core::ptr::addr_of_mut!(systemdir) as *mut WCHAR;
    }
    size = strlenW(dir) as u32 + 1;
    if !required_size.is_null() {
        *required_size = size;
    }

    if !buffer.is_null() {
        if buffer_size >= size {
            lstrcpyW(buffer, dir);
        } else {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            if dir != core::ptr::addr_of_mut!(systemdir) as *mut WCHAR {
                HeapFree(
                    GetProcessHeap(),
                    0,
                    dir as PVOID,
                );
            }
            return (FALSE) as BOOL;
        }
    }
    if dir != core::ptr::addr_of_mut!(systemdir) as *mut WCHAR {
        HeapFree(
            GetProcessHeap(),
            0,
            dir as PVOID,
        );
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueryInfOriginalFileInformationA(
    InfInformation: PSP_INF_INFORMATION,
    InfIndex: UINT,
    AlternativePlatformInfo: PSP_ALTPLATFORM_INFO,
    OriginalFileInfo: PSP_ORIGINAL_FILE_INFO_A,
) -> BOOL {
    let mut ret: BOOL;
    let mut OriginalFileInfoW: SP_ORIGINAL_FILE_INFO_W = core::mem::zeroed();

    if (*OriginalFileInfo).cbSize as usize != core::mem::size_of::<_SP_ORIGINAL_FILE_INFO_A>() {
        WARN!("incorrect OriginalFileInfo->cbSize of %d\n", (*OriginalFileInfo).cbSize);
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return (FALSE) as BOOL;
    }

    OriginalFileInfoW.cbSize = core::mem::size_of::<SP_ORIGINAL_FILE_INFO_W>() as DWORD;
    ret = SetupQueryInfOriginalFileInformationW(
        InfInformation,
        InfIndex,
        AlternativePlatformInfo,
        core::ptr::addr_of_mut!(OriginalFileInfoW),
    );
    if ret != 0 {
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            core::ptr::addr_of_mut!(OriginalFileInfoW.OriginalInfName) as *mut WCHAR as LPCWSTR,
            -1,
            core::ptr::addr_of_mut!((*OriginalFileInfo).OriginalInfName) as LPSTR,
            (MAX_PATH as i32),
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            core::ptr::addr_of_mut!(OriginalFileInfoW.OriginalCatalogName) as *mut WCHAR as LPCWSTR,
            -1,
            core::ptr::addr_of_mut!((*OriginalFileInfo).OriginalCatalogName) as LPSTR,
            (MAX_PATH as i32),
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueryInfOriginalFileInformationW(
    InfInformation: PSP_INF_INFORMATION,
    InfIndex: UINT,
    AlternativePlatformInfo: PSP_ALTPLATFORM_INFO,
    OriginalFileInfo: PSP_ORIGINAL_FILE_INFO_W,
) -> BOOL {
    let mut inf_name: LPCWSTR = core::ptr::null();
    let mut inf_path: LPCWSTR = core::ptr::null();
    let mut hinf: HINF = core::ptr::null_mut();
    static wszVersion: [u16; 8] = [0x0056u16, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000];
    static wszCatalogFile: [u16; 12] = [
    0x0043u16, 0x0061, 0x0074, 0x0061, 0x006C, 0x006F, 0x0067, 0x0046, 0x0069, 0x006C,
    0x0065, 0x0000,
];

    if (*OriginalFileInfo).cbSize != core::mem::size_of::<_SP_ORIGINAL_FILE_INFO_W>() as DWORD {
        WARN!("incorrect OriginalFileInfo->cbSize of %d\n", (*OriginalFileInfo).cbSize);
        SetLastError(ERROR_INVALID_USER_BUFFER);
        return (FALSE) as BOOL;
    }

    inf_path = &raw mut (*InfInformation).VersionData as *mut BYTE as LPWSTR as LPCWSTR;

    /* FIXME: we should get OriginalCatalogName from CatalogFile line in
     * the original inf file and cache it, but that would require building a
     * .pnf file. */
    hinf = SetupOpenInfFileW(
        inf_path as PCWSTR,
        core::ptr::null(),
        INF_STYLE_WIN4 as DWORD,
        core::ptr::null_mut(),
    );
    if hinf == INVALID_HANDLE_VALUE {
        return (FALSE) as BOOL;
    }

    if SetupGetLineTextW(
        core::ptr::null_mut(),
        hinf,
        &raw const wszVersion as PCWSTR,
        &raw const wszCatalogFile as PCWSTR,
        &raw mut (*OriginalFileInfo).OriginalCatalogName as PWSTR,
        // `sizeof(x)/sizeof(x[0])` of a field of a packed struct: a reference to it is
        // unaligned, so read the element count off the array TYPE through a raw pointer.
        {
            const fn elems<T, const N: usize>(_: *const [T; N]) -> usize {
                N
            }
            elems(&raw const (*OriginalFileInfo).OriginalCatalogName)
        } as DWORD,
        core::ptr::null_mut(),
    ) == 0
    {
        (*OriginalFileInfo).OriginalCatalogName[0] = '\0' as u16;
    }
    SetupCloseInfFile(hinf);

    /* FIXME: not quite correct as we just return the same file name as
     * destination (copied) inf file, not the source (original) inf file.
     * to fix it properly would require building a .pnf file */
    /* file name is stored in VersionData field of InfInformation */
    inf_name = strrchrW(inf_path as PCWSTR, '\\' as u16) as LPCWSTR;
    if !inf_name.is_null() {
        inf_name = inf_name.offset(1);
    } else {
        inf_name = inf_path;
    }

    strcpyW(
        &raw mut (*OriginalFileInfo).OriginalInfName as PWSTR,
        inf_name as PCWSTR,
    );

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueryInfVersionInformationA(info: *mut SP_INF_INFORMATION, index: UINT, key: *const core::ffi::c_char, buff: *mut core::ffi::c_char, size: DWORD, req_size: *mut DWORD) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueryInfVersionInformationW(
    info: *mut SP_INF_INFORMATION,
    index: UINT,
    key: *const WCHAR,
    buff: *mut WCHAR,
    size: DWORD,
    req_size: *mut DWORD,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}
