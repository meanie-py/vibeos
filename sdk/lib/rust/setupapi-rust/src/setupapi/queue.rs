// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut DotSecurity: [WCHAR; 10] = [
    '.' as i32 as WCHAR,
    'S' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'y' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct default_callback_context {
    pub owner: HWND,
    pub progress: HWND,
    pub message: UINT,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct file_op {
    pub next: *mut file_op,
    pub style: UINT,
    pub src_root: *mut WCHAR,
    pub src_path: *mut WCHAR,
    pub src_file: *mut WCHAR,
    pub src_descr: *mut WCHAR,
    pub src_tag: *mut WCHAR,
    pub dst_path: *mut WCHAR,
    pub dst_file: *mut WCHAR,
    pub dst_sd: PSECURITY_DESCRIPTOR,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct file_op_queue {
    pub head: *mut file_op,
    pub tail: *mut file_op,
    pub count: core::ffi::c_uint,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct file_queue {
    pub copy_queue: file_op_queue,
    pub delete_queue: file_op_queue,
    pub rename_queue: file_op_queue,
    pub flags: DWORD,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct extract_cab_ctx {
    pub src: *const WCHAR,
    pub dst: *const WCHAR,
}

unsafe extern "C" fn strdupW(str: *const WCHAR) -> *mut WCHAR {
    let mut ret: *mut WCHAR = core::ptr::null_mut();
    if !str.is_null() {
        let len: core::ffi::c_int = ((strlenW(str) + 1) * core::mem::size_of::<WCHAR>()) as core::ffi::c_int;
        ret = HeapAlloc(GetProcessHeap(), 0, (len) as SIZE_T) as *mut WCHAR;
        if !ret.is_null() {
            memcpy(ret as *mut core::ffi::c_void, str as *const core::ffi::c_void, len as usize);
        }
    }
    return ret;
}

unsafe extern "C" fn strdupWtoA(mut str: *const WCHAR) -> *mut ::core::ffi::c_char {
    let mut ret: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !str.is_null() {
        let mut len: DWORD = WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            str as LPCWSTR,
            -(1 as ::core::ffi::c_int),
            ::core::ptr::null_mut::<CHAR>(),
            0 as ::core::ffi::c_int,
            ::core::ptr::null::<CHAR>(),
            ::core::ptr::null_mut::<BOOL>(),
        ) as DWORD;
        ret = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            len as SIZE_T,
        ) as *mut ::core::ffi::c_char;
        if !ret.is_null() {
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                str as LPCWSTR,
                -(1 as ::core::ffi::c_int),
                ret as LPSTR,
                len as ::core::ffi::c_int,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            );
        }
    }
    return ret;
}

unsafe extern "C" fn queue_file_op(queue: *mut file_op_queue, op: *mut file_op) {
    (*op).next = core::ptr::null_mut();
    if !(*queue).tail.is_null() {
        (*(*queue).tail).next = op;
    } else {
        (*queue).head = op;
    }
    (*queue).tail = op;
    (*queue).count = (*queue).count.wrapping_add(1);
}

unsafe extern "C" fn free_file_op_queue(mut queue: *mut file_op_queue) {
    let mut t: *mut file_op = ::core::ptr::null_mut::<file_op>();
    let mut op: *mut file_op = (*queue).head;
    while !op.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, (*op).src_root as PVOID);
        HeapFree(GetProcessHeap(), 0 as DWORD, (*op).src_path as PVOID);
        HeapFree(GetProcessHeap(), 0 as DWORD, (*op).src_file as PVOID);
        HeapFree(GetProcessHeap(), 0 as DWORD, (*op).src_descr as PVOID);
        HeapFree(GetProcessHeap(), 0 as DWORD, (*op).src_tag as PVOID);
        HeapFree(GetProcessHeap(), 0 as DWORD, (*op).dst_path as PVOID);
        if !(*op).dst_sd.is_null() {
            LocalFree((*op).dst_sd as HLOCAL);
        }
        if (*op).dst_file != (*op).src_file {
            HeapFree(GetProcessHeap(), 0 as DWORD, (*op).dst_file as PVOID);
        }
        t = op;
        op = (*op).next;
        HeapFree(GetProcessHeap(), 0 as DWORD, t as PVOID);
    }
}

unsafe extern "C" fn concat_W(mut buffer: *mut WCHAR, mut src1: *const WCHAR, mut src2: *const WCHAR, mut src3: *const WCHAR) {
    *buffer = 0;
    if !src1.is_null() && *src1 != 0 {
        strcpyW(buffer, src1);
        buffer = buffer.offset(strlenW(buffer) as isize);
        if *buffer.offset(-(1 as isize)) != '\\' as WCHAR {
            *buffer = '\\' as WCHAR;
            buffer = buffer.offset(1);
        }
        if !src2.is_null() {
            while *src2 == '\\' as WCHAR {
                src2 = src2.offset(1);
            }
        }
    }

    if !src2.is_null() {
        strcpyW(buffer, src2);
        buffer = buffer.offset(strlenW(buffer) as isize);
        if *buffer.offset(-(1 as isize)) != '\\' as WCHAR {
            *buffer = '\\' as WCHAR;
            buffer = buffer.offset(1);
        }
        if !src3.is_null() {
            while *src3 == '\\' as WCHAR {
                src3 = src3.offset(1);
            }
        }
    }

    if !src3.is_null() {
        strcpyW(buffer, src3);
    }
}

unsafe extern "C" fn build_filepathsW(op: *const file_op, paths: *mut FILEPATHS_W) -> BOOL {
    let mut src_len: UINT = 1;
    let mut dst_len: UINT = 1;
    let mut source: PWSTR = (*paths).Source as PWSTR;
    let mut target: PWSTR = (*paths).Target as PWSTR;

    if !(*op).src_root.is_null() {
        src_len = src_len.wrapping_add((strlenW((*op).src_root) as UINT).wrapping_add(1));
    }
    if !(*op).src_path.is_null() {
        src_len = src_len.wrapping_add((strlenW((*op).src_path) as UINT).wrapping_add(1));
    }
    if !(*op).src_file.is_null() {
        src_len = src_len.wrapping_add((strlenW((*op).src_file) as UINT).wrapping_add(1));
    }
    if !(*op).dst_path.is_null() {
        dst_len = dst_len.wrapping_add((strlenW((*op).dst_path) as UINT).wrapping_add(1));
    }
    if !(*op).dst_file.is_null() {
        dst_len = dst_len.wrapping_add((strlenW((*op).dst_file) as UINT).wrapping_add(1));
    }
    src_len = src_len.wrapping_mul(core::mem::size_of::<WCHAR>() as UINT);
    dst_len = dst_len.wrapping_mul(core::mem::size_of::<WCHAR>() as UINT);

    if source.is_null()
        || HeapSize(GetProcessHeap(), 0, source as LPCVOID) < src_len as SIZE_T
    {
        HeapFree(GetProcessHeap(), 0, source as PVOID);
        source = HeapAlloc(GetProcessHeap(), 0, src_len as SIZE_T) as PWSTR;
        (*paths).Source = source as PCWSTR;
    }
    if target.is_null()
        || HeapSize(GetProcessHeap(), 0, target as LPCVOID) < dst_len as SIZE_T
    {
        HeapFree(GetProcessHeap(), 0, target as PVOID);
        target = HeapAlloc(GetProcessHeap(), 0, dst_len as SIZE_T) as PWSTR;
        (*paths).Target = target as PCWSTR;
    }
    if source.is_null() || target.is_null() {
        return (FALSE) as BOOL;
    }
    concat_W(source, (*op).src_root, (*op).src_path, (*op).src_file);
    concat_W(target, core::ptr::null_mut(), (*op).dst_path, (*op).dst_file);
    (*paths).Win32Error = 0;
    (*paths).Flags = 0;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn QUEUE_callback_WtoA(context: *mut ::core::ffi::c_void, notification: UINT, param1: UINT_PTR, mut param2: UINT_PTR) -> UINT {
    let callback_ctx: *mut callback_WtoA_context = context as *mut callback_WtoA_context;
    let mut buffer: [::core::ffi::c_char; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ret: UINT = 0;
    let old_param2: UINT_PTR = param2;
    let mut current_block_26: u8;
    match notification {
        SPFILENOTIFY_COPYERROR => {
            param2 = &raw mut buffer as UINT_PTR;
            current_block_26 = 0;
        }
        SPFILENOTIFY_STARTDELETE | SPFILENOTIFY_ENDDELETE | SPFILENOTIFY_DELETEERROR | SPFILENOTIFY_STARTRENAME | SPFILENOTIFY_ENDRENAME | SPFILENOTIFY_RENAMEERROR | SPFILENOTIFY_STARTCOPY | SPFILENOTIFY_ENDCOPY | SPFILENOTIFY_QUEUESCAN_EX => {
            current_block_26 = 0;
        }
        SPFILENOTIFY_STARTREGISTRATION | SPFILENOTIFY_ENDREGISTRATION => {
            let statusW: *mut SP_REGISTER_CONTROL_STATUSW = param1 as *mut SP_REGISTER_CONTROL_STATUSW;
            let mut statusA: SP_REGISTER_CONTROL_STATUSA = SP_REGISTER_CONTROL_STATUSA {
                cbSize: 0,
                FileName: ::core::ptr::null::<CHAR>(),
                Win32Error: 0,
                FailureCode: 0,
            };
            statusA.cbSize = ::core::mem::size_of::<SP_REGISTER_CONTROL_STATUSA>() as DWORD;
            statusA.FileName = strdupWtoA((*statusW).FileName as *const WCHAR) as PCSTR;
            statusA.Win32Error = (*statusW).Win32Error;
            statusA.FailureCode = (*statusW).FailureCode;
            ret = (*callback_ctx)
                .orig_handler
                .unwrap_unchecked()(
                (*callback_ctx).orig_context as PVOID,
                notification,
                &raw mut statusA as UINT_PTR,
                param2,
            );
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                statusA.FileName as LPSTR as PVOID,
            );
            current_block_26 = 2;
        }
        SPFILENOTIFY_QUEUESCAN => {
            let targetW: LPWSTR = param1 as LPWSTR;
            let target: LPSTR = strdupWtoA(targetW as *const WCHAR) as LPSTR;
            ret = (*callback_ctx)
                .orig_handler
                .unwrap_unchecked()(
                (*callback_ctx).orig_context as PVOID,
                notification,
                target as UINT_PTR,
                param2,
            );
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                target as PVOID,
            );
            current_block_26 = 2;
        }
        SPFILENOTIFY_NEEDMEDIA => {
            FIXME!("mapping for %d not implemented\n", notification);
            current_block_26 = 1;
        }
        SPFILENOTIFY_STARTQUEUE | SPFILENOTIFY_ENDQUEUE | SPFILENOTIFY_STARTSUBQUEUE | SPFILENOTIFY_ENDSUBQUEUE | _ => {
            current_block_26 = 1;
        }
    }
    match current_block_26 {
        0 => {
            let pathsW: *mut FILEPATHS_W = param1 as *mut FILEPATHS_W;
            let mut pathsA: FILEPATHS_A = FILEPATHS_A {
                Target: ::core::ptr::null::<CHAR>(),
                Source: ::core::ptr::null::<CHAR>(),
                Win32Error: 0,
                Flags: 0,
            };
            pathsA.Source = strdupWtoA((*pathsW).Source as *const WCHAR) as PCSTR;
            pathsA.Target = strdupWtoA((*pathsW).Target as *const WCHAR) as PCSTR;
            pathsA.Win32Error = (*pathsW).Win32Error;
            pathsA.Flags = (*pathsW).Flags;
            ret = (*callback_ctx)
                .orig_handler
                .unwrap_unchecked()(
                (*callback_ctx).orig_context as PVOID,
                notification,
                &raw mut pathsA as UINT_PTR,
                param2,
            );
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pathsA.Source as PVOID,
            );
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pathsA.Target as PVOID,
            );
            if notification == SPFILENOTIFY_COPYERROR as UINT {
                MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    &raw mut buffer as *mut ::core::ffi::c_char as LPCSTR,
                    -(1 as ::core::ffi::c_int),
                    old_param2 as LPWSTR,
                    (MAX_PATH as i32),
                );
            }
        }
        1 => {
            ret = (*callback_ctx)
                .orig_handler
                .unwrap_unchecked()(
                (*callback_ctx).orig_context as PVOID,
                notification,
                param1,
                param2,
            );
        }
        _ => {}
    }
    return ret;
}

unsafe extern "C" fn get_src_file_info(hinf: HINF, op: *mut file_op) {
    static SourceDisksNames: [u16; 17] = [
    0x0053u16, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0044, 0x0069, 0x0073, 0x006B,
    0x0073, 0x004E, 0x0061, 0x006D, 0x0065, 0x0073, 0x0000,
];
    static SourceDisksFiles: [u16; 17] = [
    0x0053u16, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0044, 0x0069, 0x0073, 0x006B,
    0x0073, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0000,
];

    let mut file_ctx: INFCONTEXT = _INFCONTEXT {
        Inf: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        CurrentInf: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        Section: 0,
        Line: 0,
    };
    let mut disk_ctx: INFCONTEXT = _INFCONTEXT {
        Inf: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        CurrentInf: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        Section: 0,
        Line: 0,
    };
    let mut id: INT = 0;
    let mut diskid: INT = 0;
    let mut len: DWORD = 0;
    let mut len2: DWORD = 0;
    let mut SectionName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    /* find the SourceDisksFiles entry */
    if SetupDiGetActualSectionToInstallW(
        hinf,
        &raw const SourceDisksFiles as PCWSTR,
        &raw mut SectionName as PWSTR,
        MAX_PATH,
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<PWSTR>(),
    ) == 0
    {
        return;
    }
    if SetupFindFirstLineW(
        hinf,
        &raw mut SectionName as *mut WCHAR as PCWSTR,
        (*op).src_file as PCWSTR,
        &raw mut file_ctx,
    ) == 0
    {
        if ((*op).style & (SP_COPY_SOURCE_ABSOLUTE | SP_COPY_SOURCEPATH_ABSOLUTE)) != 0 {
            return;
        }
        /* no specific info, use .inf file source directory */
        if (*op).src_root.is_null() {
            (*op).src_root = PARSER_get_src_root(hinf);
        }
        return;
    }
    if SetupGetIntField(&raw mut file_ctx, 1, &raw mut diskid) == 0 {
        return;
    }

    /* now find the diskid in the SourceDisksNames section */
    if SetupDiGetActualSectionToInstallW(
        hinf,
        &raw const SourceDisksNames as PCWSTR,
        &raw mut SectionName as PWSTR,
        MAX_PATH,
        ::core::ptr::null_mut::<DWORD>(),
        ::core::ptr::null_mut::<PWSTR>(),
    ) == 0
    {
        return;
    }
    if SetupFindFirstLineW(
        hinf,
        &raw mut SectionName as *mut WCHAR as PCWSTR,
        ::core::ptr::null::<WCHAR>(),
        &raw mut disk_ctx,
    ) == 0
    {
        return;
    }
    loop {
        if SetupGetIntField(&raw mut disk_ctx, 0, &raw mut id) != 0
            && id == diskid
        {
            break;
        }
        if SetupFindNextLine(&raw mut disk_ctx, &raw mut disk_ctx) == 0 {
            return;
        }
    }

    /* and fill in the missing info */

    if (*op).src_descr.is_null() {
        if SetupGetStringFieldW(
            &raw mut disk_ctx,
            1,
            ::core::ptr::null_mut::<WCHAR>(),
            0,
            &raw mut len,
        ) != 0
            && {
                (*op).src_descr = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    (len as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as *mut WCHAR;
                !(*op).src_descr.is_null()
            }
        {
            SetupGetStringFieldW(
                &raw mut disk_ctx,
                1,
                (*op).src_descr as PWSTR,
                len,
                ::core::ptr::null_mut::<DWORD>(),
            );
        }
    }
    if (*op).src_tag.is_null() {
        if SetupGetStringFieldW(
            &raw mut disk_ctx,
            2,
            ::core::ptr::null_mut::<WCHAR>(),
            0,
            &raw mut len,
        ) != 0
            && {
                (*op).src_tag = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    (len as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as *mut WCHAR;
                !(*op).src_tag.is_null()
            }
        {
            SetupGetStringFieldW(
                &raw mut disk_ctx,
                2,
                (*op).src_tag as PWSTR,
                len,
                ::core::ptr::null_mut::<DWORD>(),
            );
        }
    }
    if (*op).src_path.is_null() && ((*op).style & SP_COPY_SOURCE_ABSOLUTE) == 0 {
        len2 = 0;
        len = len2;
        if ((*op).style & SP_COPY_SOURCEPATH_ABSOLUTE) == 0 {
            /* retrieve relative path for this disk */
            if SetupGetStringFieldW(
                &raw mut disk_ctx,
                4,
                ::core::ptr::null_mut::<WCHAR>(),
                0,
                &raw mut len,
            ) == 0
            {
                len = 0;
            }
        }
        /* retrieve relative path for this file */
        if SetupGetStringFieldW(
            &raw mut file_ctx,
            2,
            ::core::ptr::null_mut::<WCHAR>(),
            0,
            &raw mut len2,
        ) == 0
        {
            len2 = 0;
        }

        if (len != 0 || len2 != 0)
            && {
                (*op).src_path = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    (len.wrapping_add(len2) as SIZE_T)
                        .wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as *mut WCHAR;
                !(*op).src_path.is_null()
            }
        {
            let mut ptr: *mut WCHAR = (*op).src_path;
            if len != 0 {
                SetupGetStringFieldW(
                    &raw mut disk_ctx,
                    4,
                    (*op).src_path as PWSTR,
                    len,
                    ::core::ptr::null_mut::<DWORD>(),
                );
                ptr = (*op).src_path.offset(strlenW((*op).src_path) as isize);
                if len2 != 0
                    && ptr > (*op).src_path
                    && *ptr.offset(-(1 as isize)) != '\\' as WCHAR
                {
                    let fresh18 = ptr;
                    ptr = ptr.offset(1);
                    *fresh18 = '\\' as WCHAR;
                }
            }
            if SetupGetStringFieldW(
                &raw mut file_ctx,
                2,
                ptr as PWSTR,
                len2,
                ::core::ptr::null_mut::<DWORD>(),
            ) == 0
            {
                *ptr = 0 as WCHAR;
            }
        }
    }
    if (*op).src_root.is_null() {
        (*op).src_root = PARSER_get_src_root(hinf);
    }
}

unsafe extern "C" fn get_destination_dir(hinf: HINF, section: *const WCHAR) -> *mut WCHAR {
    static mut Dest: [u16; 16] = [
    0x0044u16, 0x0065, 0x0073, 0x0074, 0x0069, 0x006E, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0044, 0x0069, 0x0072, 0x0073, 0x0000,
];
    static mut Def: [u16; 15] = [
    0x0044u16, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0044, 0x0065, 0x0073,
    0x0074, 0x0044, 0x0069, 0x0072, 0x0000,
];
    let mut context: INFCONTEXT = _INFCONTEXT {
        Inf: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        CurrentInf: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        Section: 0,
        Line: 0,
    };

    if SetupFindFirstLineW(
        hinf,
        &raw const Dest as PCWSTR,
        section as PCWSTR,
        &raw mut context,
    ) == 0
        && SetupFindFirstLineW(
            hinf,
            &raw const Dest as PCWSTR,
            &raw const Def as PCWSTR,
            &raw mut context,
        ) == 0
    {
        return core::ptr::null_mut();
    }
    return PARSER_get_dest_dir(&raw mut context);
}

unsafe extern "system" fn extract_cab_cb(
    arg: *mut core::ffi::c_void,
    message: UINT,
    param1: UINT_PTR,
    param2: UINT_PTR,
) -> UINT {
    let ctx: *mut extract_cab_ctx = arg as *mut extract_cab_ctx;

    match message {
        SPFILENOTIFY_FILEINCABINET => {
            let info: *mut FILE_IN_CABINET_INFO_W = param1 as *mut FILE_IN_CABINET_INFO_W;
            let mut filename: *const WCHAR;

            let t: *const WCHAR = strrchrW((*info).NameInCabinet, '\\' as u16) as *const WCHAR;
            if !t.is_null() {
                filename = t;
                filename = filename.add(1);
            } else {
                filename = (*info).NameInCabinet as *const WCHAR;
            }

            if lstrcmpiW(filename, (*ctx).src) != 0 {
                return FILEOP_SKIP;
            }

            strcpyW(
                core::ptr::addr_of_mut!((*info).FullTargetName) as *mut WCHAR,
                (*ctx).dst,
            );
            return FILEOP_DOIT;
        }
        SPFILENOTIFY_FILEEXTRACTED => {
            let paths = param1 as *const FILEPATHS_W;
            return (*paths).Win32Error;
        }
        SPFILENOTIFY_NEEDNEWCABINET => {
            let info = param1 as *const CABINET_INFO_W;
            strcpyW(
                param2 as *mut WCHAR,
                core::ptr::addr_of!((*info).CabinetPath) as *const WCHAR,
            );
            return ERROR_SUCCESS;
        }
        SPFILENOTIFY_CABINETINFO => {
            return 0;
        }
        _ => {
            FIXME!("Unexpected message %#x.\n", message);
            return 0;
        }
    }
}

unsafe extern "C" fn extract_cabinet_file(
    cabinet: *const WCHAR,
    root: *const WCHAR,
    src: *const WCHAR,
    dst: *const WCHAR,
) -> BOOL {
    static mut backslashW: [WCHAR; 2] = [0x005Cu16, 0x0000];
    let mut path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ctx: extract_cab_ctx = extract_cab_ctx { src: src, dst: dst };

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/queue.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"extract_cabinet_file\0" as *const u8 as *const ::core::ffi::c_char,
            428 as ::core::ffi::c_int,
            b"extract_cabinet_file(cab = '%s' ; root = '%s' ; src = '%s' ; dst = '%s')\n\0"
                as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(cabinet),
            debugstr_w(root),
            debugstr_w(src),
            debugstr_w(dst),
        );
    }

    strcpyW(&raw mut path as *mut WCHAR, root);
    strcatW(&raw mut path as *mut WCHAR, &raw const backslashW as *const WCHAR);
    strcatW(&raw mut path as *mut WCHAR, cabinet);

    return SetupIterateCabinetW(
        &raw mut path as *mut WCHAR as PCWSTR,
        0 as DWORD,
        Some(
            extract_cab_cb,
        ),
        &raw mut ctx as PVOID,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupOpenFileQueue() -> HSPFILEQ {
    let mut queue: *mut file_queue = core::ptr::null_mut::<file_queue>();
    queue = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<file_queue>()) as SIZE_T,
    ) as *mut file_queue;
    if queue.is_null() {
        return INVALID_HANDLE_VALUE;
    }
    return queue as HSPFILEQ;
}

#[no_mangle]
pub unsafe extern "system" fn SetupCloseFileQueue(handle: HSPFILEQ) -> BOOL {
    let queue: *mut file_queue = handle as *mut file_queue;
    free_file_op_queue(core::ptr::addr_of_mut!((*queue).copy_queue));
    free_file_op_queue(core::ptr::addr_of_mut!((*queue).rename_queue));
    free_file_op_queue(core::ptr::addr_of_mut!((*queue).delete_queue));
    HeapFree(GetProcessHeap(), 0, queue as PVOID);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueCopyIndirectA(params: PSP_FILE_COPY_PARAMS_A) -> BOOL {
    let queue: *mut file_queue = (*params).QueueHandle as *mut file_queue;
    let mut op: *mut file_op = core::ptr::null_mut();

    op = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<file_op>() as SIZE_T,
    ) as *mut file_op;
    if op.is_null() {
        return (FALSE) as BOOL;
    }

    (*op).style = (*params).CopyStyle;
    (*op).src_root = strdupAtoW((*params).SourceRootPath as *const ::core::ffi::c_char);
    (*op).src_path = strdupAtoW((*params).SourcePath as *const ::core::ffi::c_char);
    (*op).src_file = strdupAtoW((*params).SourceFilename as *const ::core::ffi::c_char);
    (*op).src_descr = strdupAtoW((*params).SourceDescription as *const ::core::ffi::c_char);
    (*op).src_tag = strdupAtoW((*params).SourceTagfile as *const ::core::ffi::c_char);
    (*op).dst_path = strdupAtoW((*params).TargetDirectory as *const ::core::ffi::c_char);
    (*op).dst_file = strdupAtoW((*params).TargetFilename as *const ::core::ffi::c_char);
    (*op).dst_sd = core::ptr::null_mut();

    /* some defaults */
    if (*op).src_file.is_null() {
        (*op).src_file = (*op).dst_file;
    }
    if !(*params).LayoutInf.is_null() {
        get_src_file_info((*params).LayoutInf, op);
        if (*op).dst_path.is_null() {
            (*op).dst_path = get_destination_dir((*params).LayoutInf, (*op).dst_file);
        }
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/queue.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupQueueCopyIndirectA\0" as *const u8 as *const ::core::ffi::c_char,
            500 as ::core::ffi::c_int,
            b"root=%s path=%s file=%s -> dir=%s file=%s  descr=%s tag=%s\n\0" as *const u8
                as *const ::core::ffi::c_char,
            debugstr_w((*op).src_root),
            debugstr_w((*op).src_path),
            debugstr_w((*op).src_file),
            debugstr_w((*op).dst_path),
            debugstr_w((*op).dst_file),
            debugstr_w((*op).src_descr),
            debugstr_w((*op).src_tag),
        );
    }

    queue_file_op(core::ptr::addr_of_mut!((*queue).copy_queue), op);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueCopyIndirectW(params: PSP_FILE_COPY_PARAMS_W) -> BOOL {
    let queue: *mut file_queue = (*params).QueueHandle as *mut file_queue;
    let mut op: *mut file_op = core::ptr::null_mut::<file_op>();

    op = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<file_op>() as SIZE_T,
    ) as *mut file_op;
    if op.is_null() {
        return (FALSE) as BOOL;
    }
    (*op).style = (*params).CopyStyle as UINT;
    (*op).src_root = strdupW((*params).SourceRootPath as *const WCHAR);
    (*op).src_path = strdupW((*params).SourcePath as *const WCHAR);
    (*op).src_file = strdupW((*params).SourceFilename as *const WCHAR);
    (*op).src_descr = strdupW((*params).SourceDescription as *const WCHAR);
    (*op).src_tag = strdupW((*params).SourceTagfile as *const WCHAR);
    (*op).dst_path = strdupW((*params).TargetDirectory as *const WCHAR);
    (*op).dst_file = strdupW((*params).TargetFilename as *const WCHAR);
    (*op).dst_sd = core::ptr::null_mut() as PSECURITY_DESCRIPTOR;
    if !(*params).SecurityDescriptor.is_null() {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            (*params).SecurityDescriptor as LPCWSTR,
            SDDL_REVISION_1 as DWORD,
            core::ptr::addr_of_mut!((*op).dst_sd),
            core::ptr::null_mut::<ULONG>(),
        );
    }

    /* some defaults */
    if (*op).src_file.is_null() {
        (*op).src_file = (*op).dst_file;
    }
    if !(*params).LayoutInf.is_null() {
        get_src_file_info((*params).LayoutInf, op);
        if (*op).dst_path.is_null() {
            (*op).dst_path = get_destination_dir((*params).LayoutInf, (*op).dst_file);
        }
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/queue.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupQueueCopyIndirectW\0" as *const u8 as *const ::core::ffi::c_char,
            539 as ::core::ffi::c_int,
            b"root=%s path=%s file=%s -> dir=%s file=%s  descr=%s tag=%s\n\0" as *const u8
                as *const ::core::ffi::c_char,
            debugstr_w((*op).src_root),
            debugstr_w((*op).src_path),
            debugstr_w((*op).src_file),
            debugstr_w((*op).dst_path),
            debugstr_w((*op).dst_file),
            debugstr_w((*op).src_descr),
            debugstr_w((*op).src_tag),
        );
    }

    queue_file_op(core::ptr::addr_of_mut!((*queue).copy_queue), op);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn SetupQueueCopyA(queue: HSPFILEQ, src_root: PCSTR, src_path: PCSTR, src_file: PCSTR,
                             src_descr: PCSTR, src_tag: PCSTR, dst_dir: PCSTR, dst_file: PCSTR,
                             style: DWORD) -> BOOL {
    let mut params: SP_FILE_COPY_PARAMS_A = core::mem::zeroed();

    params.cbSize             = core::mem::size_of::<SP_FILE_COPY_PARAMS_A>() as DWORD;
    params.QueueHandle        = queue;
    params.SourceRootPath     = src_root;
    params.SourcePath         = src_path;
    params.SourceFilename     = src_file;
    params.SourceDescription  = src_descr;
    params.SourceTagfile      = src_tag;
    params.TargetDirectory    = dst_dir;
    params.TargetFilename     = dst_file;
    params.CopyStyle          = style;
    params.LayoutInf          = core::ptr::null_mut();
    params.SecurityDescriptor = core::ptr::null_mut();
    return SetupQueueCopyIndirectA(core::ptr::addr_of_mut!(params));
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueCopyW(
    queue: HSPFILEQ,
    src_root: PCWSTR,
    src_path: PCWSTR,
    src_file: PCWSTR,
    src_descr: PCWSTR,
    src_tag: PCWSTR,
    dst_dir: PCWSTR,
    dst_file: PCWSTR,
    style: DWORD,
) -> BOOL {
    let mut params: SP_FILE_COPY_PARAMS_W = core::mem::zeroed();

    params.cbSize = core::mem::size_of::<SP_FILE_COPY_PARAMS_W>() as DWORD;
    params.QueueHandle = queue;
    params.SourceRootPath = src_root;
    params.SourcePath = src_path;
    params.SourceFilename = src_file;
    params.SourceDescription = src_descr;
    params.SourceTagfile = src_tag;
    params.TargetDirectory = dst_dir;
    params.TargetFilename = dst_file;
    params.CopyStyle = style;
    params.LayoutInf = core::ptr::null_mut();
    params.SecurityDescriptor = core::ptr::null_mut();
    return SetupQueueCopyIndirectW(core::ptr::addr_of_mut!(params));
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueDefaultCopyA(
    queue: HSPFILEQ,
    hinf: HINF,
    src_root: PCSTR,
    src_file: PCSTR,
    dst_file: PCSTR,
    style: DWORD,
) -> BOOL {
    let mut params: SP_FILE_COPY_PARAMS_A = core::mem::zeroed();
    params.cbSize = core::mem::size_of::<SP_FILE_COPY_PARAMS_A>() as DWORD;
    params.QueueHandle = queue;
    params.SourceRootPath = src_root;
    params.SourcePath = core::ptr::null();
    params.SourceFilename = src_file;
    params.SourceDescription = core::ptr::null();
    params.SourceTagfile = core::ptr::null();
    params.TargetDirectory = core::ptr::null();
    params.TargetFilename = dst_file;
    params.CopyStyle = style;
    params.LayoutInf = hinf;
    params.SecurityDescriptor = core::ptr::null();
    return SetupQueueCopyIndirectA(core::ptr::addr_of_mut!(params));
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueDefaultCopyW(
    queue: HSPFILEQ,
    hinf: HINF,
    src_root: PCWSTR,
    src_file: PCWSTR,
    dst_file: PCWSTR,
    style: DWORD,
) -> BOOL {
    let mut params: SP_FILE_COPY_PARAMS_W = core::mem::zeroed();

    params.cbSize = core::mem::size_of::<SP_FILE_COPY_PARAMS_W>() as DWORD;
    params.QueueHandle = queue;
    params.SourceRootPath = src_root;
    params.SourcePath = core::ptr::null_mut();
    params.SourceFilename = src_file;
    params.SourceDescription = core::ptr::null_mut();
    params.SourceTagfile = core::ptr::null_mut();
    params.TargetDirectory = core::ptr::null_mut();
    params.TargetFilename = dst_file;
    params.CopyStyle = style;
    params.LayoutInf = hinf;
    params.SecurityDescriptor = core::ptr::null_mut();
    return SetupQueueCopyIndirectW(core::ptr::addr_of_mut!(params));
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueDeleteA(handle: HSPFILEQ, part1: PCSTR, part2: PCSTR) -> BOOL {
    let queue: *mut file_queue = handle as *mut file_queue;
    let mut op: *mut file_op = core::ptr::null_mut();

    op = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<file_op>() as SIZE_T,
    ) as *mut file_op;
    if op.is_null() {
        return (FALSE) as BOOL;
    }
    (*op).style = 0;
    (*op).src_root = core::ptr::null_mut();
    (*op).src_path = core::ptr::null_mut();
    (*op).src_file = core::ptr::null_mut();
    (*op).src_descr = core::ptr::null_mut();
    (*op).src_tag = core::ptr::null_mut();
    (*op).dst_path = strdupAtoW(part1 as *const ::core::ffi::c_char);
    (*op).dst_file = strdupAtoW(part2 as *const ::core::ffi::c_char);
    (*op).dst_sd = core::ptr::null_mut();
    queue_file_op(core::ptr::addr_of_mut!((*queue).delete_queue), op);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueDeleteW(handle: HSPFILEQ, part1: PCWSTR, part2: PCWSTR) -> BOOL {
    let queue: *mut file_queue = handle as *mut file_queue;
    let mut op: *mut file_op = core::ptr::null_mut();

    op = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<file_op>() as SIZE_T) as *mut file_op;
    if op.is_null() { return (FALSE) as BOOL; }
    (*op).style = 0;
    (*op).src_root = core::ptr::null_mut();
    (*op).src_path = core::ptr::null_mut();
    (*op).src_file = core::ptr::null_mut();
    (*op).src_descr = core::ptr::null_mut();
    (*op).src_tag = core::ptr::null_mut();
    (*op).dst_path = strdupW(part1 as *const WCHAR);
    (*op).dst_file = strdupW(part2 as *const WCHAR);
    (*op).dst_sd = core::ptr::null_mut();
    queue_file_op(core::ptr::addr_of_mut!((*queue).delete_queue), op);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueRenameA(
    handle: HSPFILEQ,
    SourcePath: PCSTR,
    SourceFilename: PCSTR,
    TargetPath: PCSTR,
    TargetFilename: PCSTR,
) -> BOOL {
    let queue: *mut file_queue = handle as *mut file_queue;
    let mut op: *mut file_op = core::ptr::null_mut();

    op = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<file_op>()) as SIZE_T) as *mut file_op;
    if op.is_null() {
        return (FALSE) as BOOL;
    }
    (*op).style = 0;
    (*op).src_root = core::ptr::null_mut();
    (*op).src_path = strdupAtoW(SourcePath);
    (*op).src_file = strdupAtoW(SourceFilename);
    (*op).src_descr = core::ptr::null_mut();
    (*op).src_tag = core::ptr::null_mut();
    (*op).dst_path = strdupAtoW(TargetPath);
    (*op).dst_file = strdupAtoW(TargetFilename);
    (*op).dst_sd = core::ptr::null_mut();
    queue_file_op(core::ptr::addr_of_mut!((*queue).rename_queue), op);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueRenameW(
    handle: HSPFILEQ,
    SourcePath: PCWSTR,
    SourceFilename: PCWSTR,
    TargetPath: PCWSTR,
    TargetFilename: PCWSTR,
) -> BOOL {
    let queue: *mut file_queue = handle as *mut file_queue;
    let mut op: *mut file_op = core::ptr::null_mut::<file_op>();

    op = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<file_op>() as SIZE_T,
    ) as *mut file_op;
    if op.is_null() {
        return (FALSE) as BOOL;
    }
    (*op).style = 0 as UINT;
    (*op).src_root = core::ptr::null_mut::<WCHAR>();
    (*op).src_path = strdupW(SourcePath as *const WCHAR);
    (*op).src_file = strdupW(SourceFilename as *const WCHAR);
    (*op).src_descr = core::ptr::null_mut::<WCHAR>();
    (*op).src_tag = core::ptr::null_mut::<WCHAR>();
    (*op).dst_path = strdupW(TargetPath as *const WCHAR);
    (*op).dst_file = strdupW(TargetFilename as *const WCHAR);
    (*op).dst_sd = core::ptr::null_mut() as PSECURITY_DESCRIPTOR;
    queue_file_op(&raw mut (*queue).rename_queue, op);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueCopySectionA(
    queue: HSPFILEQ,
    src_root: PCSTR,
    hinf: HINF,
    hlist: HINF,
    section: PCSTR,
    style: DWORD,
) -> BOOL {
    let mut sectionW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ret: BOOL = (FALSE as i32);

    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(sectionW), section as PCSZ) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return (FALSE) as BOOL;
    }
    if src_root.is_null() {
        ret = SetupQueueCopySectionW(
            queue,
            core::ptr::null::<WCHAR>(),
            hinf,
            hlist,
            sectionW.Buffer as PCWSTR,
            style,
        );
    } else {
        let mut srcW: _UNICODE_STRING = _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: core::ptr::null_mut(),
        };
        if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(srcW), src_root as PCSZ) != 0 {
            ret = SetupQueueCopySectionW(
                queue,
                srcW.Buffer as PCWSTR,
                hinf,
                hlist,
                sectionW.Buffer as PCWSTR,
                style,
            );
            RtlFreeUnicodeString(core::ptr::addr_of_mut!(srcW));
        } else {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        }
    }
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(sectionW));
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueCopySectionW(
    mut queue: HSPFILEQ,
    mut src_root: PCWSTR,
    mut hinf: HINF,
    mut hlist: HINF,
    mut section: PCWSTR,
    mut style: DWORD,
) -> BOOL {
    let mut params: SP_FILE_COPY_PARAMS_W = core::mem::zeroed();
    let mut security_key: LPWSTR = core::ptr::null_mut();
    let mut security_descriptor: LPWSTR = core::ptr::null_mut();
    let mut context: INFCONTEXT = core::mem::zeroed();
    let mut security_context: INFCONTEXT = core::mem::zeroed();
    let mut dest: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut src: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut flags: INT = 0;
    let mut required: DWORD = 0;
    let mut ret: BOOL = 0;

    /* Check for .Security section */
    security_key = MyMalloc(
        ((strlenW(section) as usize + strlenW(core::ptr::addr_of!(DotSecurity) as PCWSTR) as usize)
            * core::mem::size_of::<WCHAR>()
            + core::mem::size_of_val(&UNICODE_NULL)) as DWORD,
    ) as LPWSTR;
    if security_key.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return (FALSE) as BOOL;
    }
    strcpyW(security_key, section);
    strcatW(security_key, core::ptr::addr_of!(DotSecurity) as PCWSTR);
    ret = SetupFindFirstLineW(
        hinf,
        security_key as PCWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(security_context),
    );
    MyFree(security_key as PVOID);
    if ret != 0 {
        if SetupGetLineTextW(
            core::ptr::addr_of_mut!(security_context),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(required),
        ) == 0
        {
            return (FALSE) as BOOL;
        }
        security_descriptor = MyMalloc(required * core::mem::size_of::<WCHAR>() as DWORD) as LPWSTR;
        if security_descriptor.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return (FALSE) as BOOL;
        }
        if SetupGetLineTextW(
            core::ptr::addr_of_mut!(security_context),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            security_descriptor as PWSTR,
            required,
            core::ptr::null_mut(),
        ) == 0
        {
            MyFree(security_descriptor as PVOID);
            return (FALSE) as BOOL;
        }
    }

    params.cbSize = core::mem::size_of::<SP_FILE_COPY_PARAMS_W>() as DWORD;
    params.QueueHandle = queue;
    params.SourceRootPath = src_root;
    params.SourcePath = core::ptr::null_mut();
    params.SourceDescription = core::ptr::null_mut();
    params.SourceTagfile = core::ptr::null_mut();
    params.TargetFilename = core::ptr::addr_of_mut!(dest) as *mut WCHAR as PCWSTR;
    params.CopyStyle = style;
    params.LayoutInf = hinf;
    params.SecurityDescriptor = security_descriptor as PCWSTR;

    ret = (FALSE as i32);
    if hlist.is_null() {
        hlist = hinf;
    }
    if hinf.is_null() {
        hinf = hlist;
    }
    'done: {
        if SetupFindFirstLineW(
            hlist,
            section,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(context),
        ) == 0
        {
            break 'done;
        }
        params.TargetDirectory = get_destination_dir(hinf, section as *const WCHAR) as PCWSTR;
        if params.TargetDirectory.is_null() {
            break 'done;
        }
        loop {
            if SetupGetStringFieldW(
                core::ptr::addr_of_mut!(context),
                1,
                core::ptr::addr_of_mut!(dest) as PWSTR,
                (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>()
                    / core::mem::size_of::<WCHAR>()) as DWORD,
                core::ptr::null_mut(),
            ) == 0
            {
                break 'done;
            }
            if SetupGetStringFieldW(
                core::ptr::addr_of_mut!(context),
                2,
                core::ptr::addr_of_mut!(src) as PWSTR,
                (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>()
                    / core::mem::size_of::<WCHAR>()) as DWORD,
                core::ptr::null_mut(),
            ) == 0
            {
                *(core::ptr::addr_of_mut!(src) as *mut WCHAR) = 0;
            }
            if SetupGetIntField(
                core::ptr::addr_of_mut!(context),
                4,
                core::ptr::addr_of_mut!(flags),
            ) == 0
            {
                flags = 0;
            }

            params.SourceFilename = (if *(core::ptr::addr_of_mut!(src) as *mut WCHAR) != 0 {
                core::ptr::addr_of_mut!(src) as *mut WCHAR
            } else {
                core::ptr::null_mut()
            }) as PCWSTR;
            if SetupQueueCopyIndirectW(core::ptr::addr_of_mut!(params)) == 0 {
                break 'done;
            }
            if !(SetupFindNextLine(
                core::ptr::addr_of_mut!(context),
                core::ptr::addr_of_mut!(context),
            ) != 0)
            {
                break;
            }
        }
        ret = (TRUE as i32);
    }
    if !security_descriptor.is_null() {
        MyFree(security_descriptor as PVOID);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueDeleteSectionA(queue: HSPFILEQ, hinf: HINF, hlist: HINF, section: PCSTR) -> BOOL {
    let mut sectionW: _UNICODE_STRING = core::mem::zeroed();
    let mut ret: BOOL = (FALSE as i32);

    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(sectionW), section) != 0 {
        ret = SetupQueueDeleteSectionW(queue, hinf, hlist, sectionW.Buffer);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(sectionW));
    } else {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueDeleteSectionW(
    mut queue: HSPFILEQ,
    mut hinf: HINF,
    mut hlist: HINF,
    mut section: PCWSTR,
) -> BOOL {
    let mut context: INFCONTEXT = core::mem::zeroed();
    let mut dest_dir: *mut WCHAR;
    let mut buffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ret: BOOL = (FALSE as i32);
    let mut flags: INT;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/queue.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupQueueDeleteSectionW\0" as *const u8 as *const ::core::ffi::c_char,
            878 as ::core::ffi::c_int,
            b"hinf=%p/%p section=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            hinf,
            hlist,
            debugstr_w(section as *const WCHAR),
        );
    }

    if hlist.is_null() {
        hlist = hinf;
    }
    if SetupFindFirstLineW(
        hlist,
        section,
        core::ptr::null(),
        core::ptr::addr_of_mut!(context),
    ) == 0
    {
        return (FALSE) as BOOL;
    }
    dest_dir = get_destination_dir(hinf, section as *const WCHAR);
    if dest_dir.is_null() {
        return (FALSE) as BOOL;
    }
    'done: {
        loop {
            if SetupGetStringFieldW(
                core::ptr::addr_of_mut!(context),
                1,
                core::ptr::addr_of_mut!(buffer) as PWSTR,
                (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>()
                    / core::mem::size_of::<WCHAR>()) as DWORD,
                core::ptr::null_mut(),
            ) == 0
            {
                break 'done;
            }
            flags = 0;
            if SetupGetIntField(
                core::ptr::addr_of_mut!(context),
                4,
                core::ptr::addr_of_mut!(flags),
            ) == 0
            {
                flags = 0;
            }
            if SetupQueueDeleteW(
                queue,
                dest_dir as PCWSTR,
                core::ptr::addr_of_mut!(buffer) as *mut WCHAR as PCWSTR,
            ) == 0
            {
                break 'done;
            }
            if !(SetupFindNextLine(
                core::ptr::addr_of_mut!(context),
                core::ptr::addr_of_mut!(context),
            ) != 0)
            {
                break;
            }
        }

        ret = (TRUE as i32);
    }

    HeapFree(
        GetProcessHeap(),
        0,
        dest_dir as PVOID,
    );
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueRenameSectionA(
    queue: HSPFILEQ,
    hinf: HINF,
    hlist: HINF,
    section: PCSTR,
) -> BOOL {
    let mut sectionW: _UNICODE_STRING = core::mem::zeroed::<_UNICODE_STRING>();
    let mut ret: BOOL = (FALSE as i32);

    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(sectionW), section as PCSZ) != 0 {
        ret = SetupQueueRenameSectionW(queue, hinf, hlist, sectionW.Buffer as PCWSTR);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(sectionW));
    } else {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueRenameSectionW(
    queue: HSPFILEQ,
    hinf: HINF,
    mut hlist: HINF,
    section: PCWSTR,
) -> BOOL {
    let mut context: INFCONTEXT = core::mem::zeroed();
    let mut dest_dir: *mut WCHAR;
    let mut src: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dst: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut ret: BOOL = (FALSE as i32);

    if hlist.is_null() {
        hlist = hinf;
    }
    if SetupFindFirstLineW(
        hlist,
        section,
        core::ptr::null::<WCHAR>(),
        core::ptr::addr_of_mut!(context),
    ) == 0
    {
        return (FALSE) as BOOL;
    }
    dest_dir = get_destination_dir(hinf, section);
    if dest_dir.is_null() {
        return (FALSE) as BOOL;
    }
    'done: {
        loop {
            if SetupGetStringFieldW(
                core::ptr::addr_of_mut!(context),
                1,
                core::ptr::addr_of_mut!(dst) as PWSTR,
                (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>()
                    / core::mem::size_of::<WCHAR>()) as DWORD,
                core::ptr::null_mut::<DWORD>(),
            ) == 0
            {
                break 'done;
            }
            if SetupGetStringFieldW(
                core::ptr::addr_of_mut!(context),
                2,
                core::ptr::addr_of_mut!(src) as PWSTR,
                (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>()
                    / core::mem::size_of::<WCHAR>()) as DWORD,
                core::ptr::null_mut::<DWORD>(),
            ) == 0
            {
                break 'done;
            }
            if SetupQueueRenameW(
                queue,
                dest_dir as PCWSTR,
                core::ptr::addr_of_mut!(src) as *mut WCHAR as PCWSTR,
                core::ptr::null::<WCHAR>(),
                core::ptr::addr_of_mut!(dst) as *mut WCHAR as PCWSTR,
            ) == 0
            {
                break 'done;
            }
            if !(SetupFindNextLine(
                core::ptr::addr_of_mut!(context),
                core::ptr::addr_of_mut!(context),
            ) != 0)
            {
                break;
            }
        }
        ret = (TRUE as i32);
    }
    HeapFree(
        GetProcessHeap(),
        0,
        dest_dir as PVOID,
    );
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupCommitFileQueueA(
    owner: HWND,
    queue: HSPFILEQ,
    handler: PSP_FILE_CALLBACK_A,
    context: PVOID,
) -> BOOL {
    let mut ctx: callback_WtoA_context = core::mem::zeroed();
    ctx.orig_context = context;
    ctx.orig_handler = handler;
    return SetupCommitFileQueueW(
        owner,
        queue,
        Some(QUEUE_callback_WtoA),
        core::ptr::addr_of_mut!(ctx) as PVOID,
    );
}

unsafe extern "C" fn create_full_pathW(path: *const WCHAR) -> BOOL {
    let mut ret: BOOL = TRUE as BOOL;
    let mut len: c_int = 0;
    let mut new_path: *mut WCHAR = core::ptr::null_mut();

    new_path = HeapAlloc(
        GetProcessHeap(),
        0,
        ((strlenW(path) as usize + 1 as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as *mut WCHAR;
    strcpyW(new_path, path);

    loop {
        len = strlenW(new_path) as c_int;
        if !(len != 0 && *new_path.offset((len - 1 as c_int) as isize) as c_int == '\\' as c_int) {
            break;
        }
        *new_path.offset((len - 1 as c_int) as isize) = 0 as WCHAR;
    }

    while CreateDirectoryW(new_path as LPCWSTR, core::ptr::null_mut()) == 0 {
        let mut slash: *mut WCHAR = core::ptr::null_mut();
        let last_error: DWORD = GetLastError();

        if last_error == ERROR_ALREADY_EXISTS as DWORD {
            break;
        }

        if last_error != ERROR_PATH_NOT_FOUND as DWORD {
            ret = FALSE as BOOL;
            break;
        }

        slash = strrchrW(new_path, '\\' as WCHAR) as *mut WCHAR;
        if slash.is_null() {
            ret = FALSE as BOOL;
            break;
        }

        len = slash.offset_from(new_path) as c_int;
        *new_path.offset(len as isize) = 0 as WCHAR;
        if create_full_pathW(new_path) == 0 {
            ret = FALSE as BOOL;
            break;
        }
        *new_path.offset(len as isize) = '\\' as WCHAR;
    }

    HeapFree(GetProcessHeap(), 0, new_path as LPVOID);
    return ret;
}

unsafe extern "C" fn do_file_copyW(source: LPCWSTR, target: LPCWSTR, style: DWORD, handler: PSP_FILE_CALLBACK_W, context: PVOID) -> BOOL {
    let mut rc: BOOL = (FALSE as i32);
    let mut docopy: BOOL = (TRUE as i32);
    let mut hSource: INT = 0;
    let mut hTemp: INT = 0;
    let mut OfStruct: OFSTRUCT = core::mem::zeroed();
    let mut TempPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut TempFile: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lRes: LONG = 0;
    let mut dwLastError: DWORD = 0;

    TRACE!(
        b"copy %s to %s style 0x%x\n\0".as_ptr() as *const i8,
        debugstr_w(source),
        debugstr_w(target),
        style
    );

    /* Get a temp file name */
    if GetTempPathW(ARRAYSIZE!(TempPath), core::ptr::addr_of_mut!(TempPath) as *mut WCHAR as LPWSTR) == 0 {
        ERR!(b"GetTempPathW error\n\0".as_ptr() as *const i8);
        return (FALSE) as BOOL;
    }

    /* Try to open the source file */
    hSource = LZOpenFileW(source as LPWSTR, core::ptr::addr_of_mut!(OfStruct), OF_READ as WORD);
    if hSource < 0 {
        return (FALSE) as BOOL;
    }

    if GetTempFileNameW(
        core::ptr::addr_of_mut!(TempPath) as *mut WCHAR as LPCWSTR,
        RUSTCONV_WSTR_96A296D224F2.as_ptr(),
        0 as UINT,
        core::ptr::addr_of_mut!(TempFile) as *mut WCHAR as LPWSTR,
    ) == 0
    {
        dwLastError = GetLastError();

        ERR!(
            b"GetTempFileNameW(%s) error\n\0".as_ptr() as *const i8,
            debugstr_w(core::ptr::addr_of_mut!(TempPath) as *mut WCHAR)
        );

        /* Close the source handle */
        LZClose(hSource);

        /* Restore error condition triggered by GetTempFileNameW */
        SetLastError(dwLastError);

        return (FALSE) as BOOL;
    }

    /* Extract the compressed file to a temp location */
    hTemp = LZOpenFileW(
        core::ptr::addr_of_mut!(TempFile) as *mut WCHAR as LPWSTR,
        core::ptr::addr_of_mut!(OfStruct),
        OF_CREATE as WORD,
    );
    if hTemp < 0 {
        dwLastError = GetLastError();

        ERR!(
            b"LZOpenFileW(2) error %d %s\n\0".as_ptr() as *const i8,
            hTemp as i32,
            debugstr_w(core::ptr::addr_of_mut!(TempFile) as *mut WCHAR)
        );

        /* Close the source handle */
        LZClose(hSource);

        /* Delete temp file if an error is signaled */
        DeleteFileW(core::ptr::addr_of_mut!(TempFile) as *mut WCHAR as LPCWSTR);

        /* Restore error condition triggered by LZOpenFileW */
        SetLastError(dwLastError);

        return (FALSE) as BOOL;
    }

    lRes = LZCopy(hSource, hTemp);

    dwLastError = GetLastError();

    LZClose(hSource);
    LZClose(hTemp);

    if lRes < 0 {
        ERR!(
            b"LZCopy error %d (%s, %s)\n\0".as_ptr() as *const i8,
            lRes as i32,
            debugstr_w(source),
            debugstr_w(core::ptr::addr_of_mut!(TempFile) as *mut WCHAR)
        );

        /* Delete temp file if copy was not successful */
        DeleteFileW(core::ptr::addr_of_mut!(TempFile) as *mut WCHAR as LPCWSTR);

        /* Restore error condition triggered by LZCopy */
        SetLastError(dwLastError);

        return (FALSE) as BOOL;
    }

    /* before copy processing */
    if (style & (SP_COPY_REPLACEONLY as DWORD)) != 0 {
        if GetFileAttributesW(target) == INVALID_FILE_ATTRIBUTES {
            docopy = (FALSE as i32);
        }
    }
    if (style & ((SP_COPY_NEWER_OR_SAME | SP_COPY_NEWER_ONLY | SP_COPY_FORCE_NEWER) as DWORD)) != 0 {
        let mut VersionSizeSource: DWORD = 0;
        let mut VersionSizeTarget: DWORD = 0;
        let mut zero: DWORD = 0;

        /*
         * This is sort of an interesting workaround. You see, calling
         * GetVersionInfoSize on a builtin dll loads that dll into memory
         * and we do not properly unload builtin dlls.. so we effectively
         * lock into memory all the targets we are replacing. This leads
         * to problems when we try to register the replaced dlls.
         *
         * So I will test for the existence of the files first so that
         * we just basically unconditionally replace the builtin versions.
         */
        if (GetFileAttributesW(target) != INVALID_FILE_ATTRIBUTES)
            && (GetFileAttributesW(core::ptr::addr_of_mut!(TempFile) as *mut WCHAR as LPCWSTR) != INVALID_FILE_ATTRIBUTES)
        {
            VersionSizeSource = GetFileVersionInfoSizeW(core::ptr::addr_of_mut!(TempFile) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(zero));
            VersionSizeTarget = GetFileVersionInfoSizeW(target as LPWSTR as LPCWSTR, core::ptr::addr_of_mut!(zero));
        }

        if VersionSizeSource != 0 && VersionSizeTarget != 0 {
            let mut VersionSource: LPVOID = core::ptr::null_mut::<core::ffi::c_void>();
            let mut VersionTarget: LPVOID = core::ptr::null_mut::<core::ffi::c_void>();
            let mut TargetInfo: *mut VS_FIXEDFILEINFO = core::ptr::null_mut::<VS_FIXEDFILEINFO>();
            let mut SourceInfo: *mut VS_FIXEDFILEINFO = core::ptr::null_mut::<VS_FIXEDFILEINFO>();
            let mut length: UINT = 0;
            let mut SubBlock: [WCHAR; 2] = [0x005Cu16, 0x0000];
            let mut ret: DWORD = 0;

            VersionSource = HeapAlloc(GetProcessHeap(), 0 as DWORD, VersionSizeSource as SIZE_T) as LPVOID;
            VersionTarget = HeapAlloc(GetProcessHeap(), 0 as DWORD, VersionSizeTarget as SIZE_T) as LPVOID;

            ret = GetFileVersionInfoW(
                core::ptr::addr_of_mut!(TempFile) as *mut WCHAR as LPCWSTR,
                0 as DWORD,
                VersionSizeSource,
                VersionSource as PVOID,
            ) as DWORD;
            if ret != 0 {
                ret = GetFileVersionInfoW(
                    target as LPWSTR as LPCWSTR,
                    0 as DWORD,
                    VersionSizeTarget,
                    VersionTarget as PVOID,
                ) as DWORD;
            }

            if ret != 0 {
                ret = VerQueryValueW(
                    VersionSource as LPCVOID,
                    core::ptr::addr_of_mut!(SubBlock) as *mut WCHAR as LPCWSTR,
                    core::ptr::addr_of_mut!(SourceInfo) as *mut LPVOID,
                    core::ptr::addr_of_mut!(length),
                ) as DWORD;
                if ret != 0 {
                    ret = VerQueryValueW(
                        VersionTarget as LPCVOID,
                        core::ptr::addr_of_mut!(SubBlock) as *mut WCHAR as LPCWSTR,
                        core::ptr::addr_of_mut!(TargetInfo) as *mut LPVOID,
                        core::ptr::addr_of_mut!(length),
                    ) as DWORD;
                }

                if ret != 0 {
                    let mut filepaths: FILEPATHS_W = FILEPATHS_W {
                        Target: core::ptr::null::<WCHAR>(),
                        Source: core::ptr::null::<WCHAR>(),
                        Win32Error: 0,
                        Flags: 0,
                    };

                    TRACE!(
                        b"Versions: Source %i.%i target %i.%i\n\0".as_ptr() as *const i8,
                        (*SourceInfo).dwFileVersionMS,
                        (*SourceInfo).dwFileVersionLS,
                        (*TargetInfo).dwFileVersionMS,
                        (*TargetInfo).dwFileVersionLS
                    );

                    /* used in case of notification */
                    filepaths.Target = target as PCWSTR;
                    filepaths.Source = source as PCWSTR;
                    filepaths.Win32Error = 0 as UINT;
                    filepaths.Flags = 0 as DWORD;

                    if (*TargetInfo).dwFileVersionMS > (*SourceInfo).dwFileVersionMS {
                        if handler.is_some() {
                            docopy = (handler.unwrap_unchecked())(context, SPFILENOTIFY_TARGETNEWER as UINT, core::ptr::addr_of_mut!(filepaths) as UINT_PTR, 0 as UINT_PTR) as BOOL;
                        } else {
                            docopy = (FALSE as i32);
                        }
                    } else if ((*TargetInfo).dwFileVersionMS == (*SourceInfo).dwFileVersionMS)
                        && ((*TargetInfo).dwFileVersionLS > (*SourceInfo).dwFileVersionLS)
                    {
                        if handler.is_some() {
                            docopy = (handler.unwrap_unchecked())(context, SPFILENOTIFY_TARGETNEWER as UINT, core::ptr::addr_of_mut!(filepaths) as UINT_PTR, 0 as UINT_PTR) as BOOL;
                        } else {
                            docopy = (FALSE as i32);
                        }
                    } else if ((style & (SP_COPY_NEWER_ONLY as DWORD)) != 0)
                        && ((*TargetInfo).dwFileVersionMS == (*SourceInfo).dwFileVersionMS)
                        && ((*TargetInfo).dwFileVersionLS == (*SourceInfo).dwFileVersionLS)
                    {
                        if handler.is_some() {
                            docopy = (handler.unwrap_unchecked())(context, SPFILENOTIFY_TARGETNEWER as UINT, core::ptr::addr_of_mut!(filepaths) as UINT_PTR, 0 as UINT_PTR) as BOOL;
                        } else {
                            docopy = (FALSE as i32);
                        }
                    }
                }
            }
            HeapFree(GetProcessHeap(), 0 as DWORD, VersionSource as PVOID);
            HeapFree(GetProcessHeap(), 0 as DWORD, VersionTarget as PVOID);
        }
    }
    if (style & ((SP_COPY_NOOVERWRITE | SP_COPY_FORCE_NOOVERWRITE) as DWORD)) != 0 {
        if GetFileAttributesW(target) != INVALID_FILE_ATTRIBUTES {
            /* Notify user target file exists */
            docopy = (FALSE as i32);
        }
    }
    if (style & ((SP_COPY_NODECOMP | SP_COPY_LANGUAGEAWARE | SP_COPY_FORCE_IN_USE |
                 SP_COPY_IN_USE_NEEDS_REBOOT | SP_COPY_NOSKIP | SP_COPY_WARNIFSKIP) as DWORD)) != 0
    {
        ERR!(b"Unsupported style(s) 0x%x\n\0".as_ptr() as *const i8, style);
    }

    if docopy != 0 {
        rc = MoveFileExW(
            core::ptr::addr_of_mut!(TempFile) as *mut WCHAR as LPCWSTR,
            target,
            MOVEFILE_REPLACE_EXISTING as DWORD,
        );
        TRACE!(b"Did copy... rc was %i\n\0".as_ptr() as *const i8, rc);
    }

    /* after copy processing */
    if (style & (SP_COPY_DELETESOURCE as DWORD)) != 0 {
        if rc != 0 {
            DeleteFileW(source);
        }
    }

    return rc;
}

#[no_mangle]
pub unsafe extern "system" fn SetupInstallFileA(
    hinf: HINF,
    inf_context: PINFCONTEXT,
    source: PCSTR,
    root: PCSTR,
    dest: PCSTR,
    style: DWORD,
    handler: PSP_FILE_CALLBACK_A,
    context: PVOID,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut ctx: callback_WtoA_context = callback_WtoA_context {
        orig_context: core::ptr::null_mut::<core::ffi::c_void>(),
        orig_handler: None,
    };
    let mut sourceW: _UNICODE_STRING = core::mem::zeroed();
    let mut rootW: _UNICODE_STRING = core::mem::zeroed();
    let mut destW: _UNICODE_STRING = core::mem::zeroed();

    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/queue.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"SetupInstallFileA\0" as *const u8 as *const core::ffi::c_char,
            1245 as i32,
            b"%p %p %s %s %s %x %p %p\n\0" as *const u8 as *const core::ffi::c_char,
            hinf,
            inf_context,
            debugstr_a(source as *const core::ffi::c_char),
            debugstr_a(root as *const core::ffi::c_char),
            debugstr_a(dest as *const core::ffi::c_char),
            style,
            handler,
            context,
        );
    }

    sourceW.Buffer = core::ptr::null_mut();
    rootW.Buffer = core::ptr::null_mut();
    destW.Buffer = core::ptr::null_mut();

    'exit: {
        if !source.is_null()
            && RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(sourceW), source) == 0
        {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return (FALSE) as BOOL;
        }
        if !root.is_null()
            && RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(rootW), root) == 0
        {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'exit;
        }
        if !dest.is_null()
            && RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(destW), dest) == 0
        {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'exit;
        }

        ctx.orig_context = context as *mut core::ffi::c_void;
        ctx.orig_handler = handler;

        ret = SetupInstallFileW(
            hinf,
            inf_context,
            sourceW.Buffer as PCWSTR,
            rootW.Buffer as PCWSTR,
            destW.Buffer as PCWSTR,
            style,
            Some(
                QUEUE_callback_WtoA,
            ),
            core::ptr::addr_of_mut!(ctx) as PVOID,
        );
    }

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(sourceW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(rootW));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(destW));
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupInstallFileW(
    mut hinf: HINF,
    mut inf_context: PINFCONTEXT,
    mut source: PCWSTR,
    mut root: PCWSTR,
    mut dest: PCWSTR,
    mut style: DWORD,
    mut handler: PSP_FILE_CALLBACK_W,
    mut context: PVOID,
) -> BOOL {
    static CopyFiles: [WCHAR; 10] = [0x0043u16, 0x006F, 0x0070, 0x0079, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0000];

    let mut ret: BOOL = 0;
    let mut absolute: BOOL = (!root.is_null()
        && *root as ::core::ffi::c_int != 0
        && (style & (SP_COPY_SOURCE_ABSOLUTE as DWORD)) == 0) as BOOL;
    let mut buffer: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut p: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut inf_source: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut len: ::core::ffi::c_uint = 0;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/queue.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupInstallFileW\0" as *const u8 as *const ::core::ffi::c_char,
            1289 as ::core::ffi::c_int,
            b"%p %p %s %s %s %x %p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            hinf,
            inf_context,
            debugstr_w(source as *const WCHAR),
            debugstr_w(root as *const WCHAR),
            debugstr_w(dest as *const WCHAR),
            style,
            handler,
            context,
        );
    }

    if !hinf.is_null() {
        let mut ctx: INFCONTEXT = core::mem::zeroed();

        if inf_context.is_null() {
            inf_context = core::ptr::addr_of_mut!(ctx) as PINFCONTEXT;
            if SetupFindFirstLineW(
                hinf,
                core::ptr::addr_of!(CopyFiles) as PCWSTR,
                core::ptr::null::<WCHAR>(),
                inf_context,
            ) == 0
            {
                return (FALSE) as BOOL;
            }
        }
        if SetupGetStringFieldW(
            inf_context,
            1 as DWORD,
            core::ptr::null_mut::<WCHAR>(),
            0 as DWORD,
            core::ptr::addr_of_mut!(len) as PDWORD,
        ) == 0
        {
            return (FALSE) as BOOL;
        }
        inf_source = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        if inf_source.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            return (FALSE) as BOOL;
        }
        if SetupGetStringFieldW(
            inf_context,
            1 as DWORD,
            inf_source as PWSTR,
            len as DWORD,
            core::ptr::null_mut::<DWORD>(),
        ) == 0
        {
            HeapFree(
                GetProcessHeap(),
                0,
                inf_source as PVOID,
            );
            return (FALSE) as BOOL;
        }
        source = inf_source as PCWSTR;
    } else if source.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }

    len = (strlenW(source) as ::core::ffi::c_uint)
        + (1 as ::core::ffi::c_uint);
    if absolute != 0 {
        len = len + (strlenW(root) as ::core::ffi::c_uint)
            + (1 as ::core::ffi::c_uint);
    }

    buffer = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;
    p = buffer;
    if p.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            inf_source as PVOID,
        );
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return (FALSE) as BOOL;
    }

    if absolute != 0 {
        strcpyW(buffer, root);
        p = p.offset(strlenW(buffer) as isize);
        if *p.offset(-(1 as isize)) as ::core::ffi::c_int != '\\' as i32 {
            let fresh17 = p;
            p = p.offset(1);
            *fresh17 = '\\' as WCHAR;
        }
    }
    while *source == '\\' as WCHAR {
        source = source.offset(1);
    }
    strcpyW(p, source);

    ret = do_file_copyW(buffer, dest, style, handler, context);

    HeapFree(
        GetProcessHeap(),
        0,
        inf_source as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        buffer as PVOID,
    );
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupCommitFileQueueW(
    owner: HWND,
    handle: HSPFILEQ,
    handler: PSP_FILE_CALLBACK_W,
    context: PVOID,
) -> BOOL {
    let queue: *mut file_queue = handle as *mut file_queue;
    let mut op: *mut file_op = core::ptr::null_mut();
    let mut result: BOOL = (FALSE as i32);
    let mut paths: FILEPATHS_W = core::mem::zeroed();
    let mut op_result: UINT = 0;

    paths.Source = core::ptr::null_mut();
    paths.Target = core::ptr::null_mut();

    if (*queue).copy_queue.count == 0 && (*queue).delete_queue.count == 0 && (*queue).rename_queue.count == 0 {
        return (TRUE) as BOOL; /* nothing to do */
    }

    if (handler.unwrap_unchecked())(context, SPFILENOTIFY_STARTQUEUE, owner as UINT_PTR, 0) == 0 {
        return (FALSE) as BOOL;
    }

    'done: {
        /* perform deletes */

        if (*queue).delete_queue.count != 0 {
            if (handler.unwrap_unchecked())(
                context,
                SPFILENOTIFY_STARTSUBQUEUE,
                (FILEOP_DELETE as _),
                ((*queue).delete_queue.count) as _,
            ) == 0
            {
                break 'done;
            }
            op = (*queue).delete_queue.head;
            while !op.is_null() {
                build_filepathsW(op, core::ptr::addr_of_mut!(paths));
                op_result = (handler.unwrap_unchecked())(
                    context,
                    SPFILENOTIFY_STARTDELETE,
                    core::ptr::addr_of_mut!(paths) as UINT_PTR,
                    (FILEOP_DELETE as _),
                );
                if op_result == FILEOP_ABORT {
                    break 'done;
                }
                while op_result == FILEOP_DOIT {
                    if DeleteFileW(paths.Target) != 0 {
                        break; /* success */
                    }
                    paths.Win32Error = GetLastError();
                    op_result = (handler.unwrap_unchecked())(
                        context,
                        SPFILENOTIFY_DELETEERROR,
                        core::ptr::addr_of_mut!(paths) as UINT_PTR,
                        0,
                    );
                    if op_result == FILEOP_ABORT {
                        break 'done;
                    }
                }
                (handler.unwrap_unchecked())(
                    context,
                    SPFILENOTIFY_ENDDELETE,
                    core::ptr::addr_of_mut!(paths) as UINT_PTR,
                    0,
                );
                op = (*op).next;
            }
            (handler.unwrap_unchecked())(context, SPFILENOTIFY_ENDSUBQUEUE, (FILEOP_DELETE as _), 0);
        }

        /* perform renames */

        if (*queue).rename_queue.count != 0 {
            if (handler.unwrap_unchecked())(
                context,
                SPFILENOTIFY_STARTSUBQUEUE,
                (FILEOP_RENAME as _),
                ((*queue).rename_queue.count) as _,
            ) == 0
            {
                break 'done;
            }
            op = (*queue).rename_queue.head;
            while !op.is_null() {
                build_filepathsW(op, core::ptr::addr_of_mut!(paths));
                op_result = (handler.unwrap_unchecked())(
                    context,
                    SPFILENOTIFY_STARTRENAME,
                    core::ptr::addr_of_mut!(paths) as UINT_PTR,
                    (FILEOP_RENAME as _),
                );
                if op_result == FILEOP_ABORT {
                    break 'done;
                }
                while op_result == FILEOP_DOIT {
                    if MoveFileW(paths.Source, paths.Target) != 0 {
                        break; /* success */
                    }
                    paths.Win32Error = GetLastError();
                    op_result = (handler.unwrap_unchecked())(
                        context,
                        SPFILENOTIFY_RENAMEERROR,
                        core::ptr::addr_of_mut!(paths) as UINT_PTR,
                        0,
                    );
                    if op_result == FILEOP_ABORT {
                        break 'done;
                    }
                }
                (handler.unwrap_unchecked())(
                    context,
                    SPFILENOTIFY_ENDRENAME,
                    core::ptr::addr_of_mut!(paths) as UINT_PTR,
                    0,
                );
                op = (*op).next;
            }
            (handler.unwrap_unchecked())(context, SPFILENOTIFY_ENDSUBQUEUE, (FILEOP_RENAME as _), 0);
        }

        /* perform copies */

        if (*queue).copy_queue.count != 0 {
            if (handler.unwrap_unchecked())(
                context,
                SPFILENOTIFY_STARTSUBQUEUE,
                (FILEOP_COPY as _),
                ((*queue).copy_queue.count) as _,
            ) == 0
            {
                break 'done;
            }
            op = (*queue).copy_queue.head;
            while !op.is_null() {
                let mut newpath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

                build_filepathsW(op, core::ptr::addr_of_mut!(paths));
                op_result = (handler.unwrap_unchecked())(
                    context,
                    SPFILENOTIFY_STARTCOPY,
                    core::ptr::addr_of_mut!(paths) as UINT_PTR,
                    (FILEOP_COPY as _),
                );
                if op_result == FILEOP_ABORT {
                    break 'done;
                }
                if op_result == FILEOP_NEWPATH {
                    op_result = FILEOP_DOIT;
                }
                while op_result == FILEOP_DOIT || op_result == FILEOP_NEWPATH {
                    if !(*op).dst_path.is_null() {
                        if create_full_pathW((*op).dst_path) == 0 {
                            paths.Win32Error = GetLastError();
                            op_result = (handler.unwrap_unchecked())(
                                context,
                                SPFILENOTIFY_COPYERROR,
                                core::ptr::addr_of_mut!(paths) as UINT_PTR,
                                newpath.as_mut_ptr() as UINT_PTR,
                            );
                            if op_result == FILEOP_ABORT {
                                break 'done;
                            }
                        }
                    }
                    if do_file_copyW(
                        if op_result == FILEOP_NEWPATH {
                            newpath.as_mut_ptr() as LPCWSTR
                        } else {
                            paths.Source as LPCWSTR
                        },
                        paths.Target as LPCWSTR,
                        (*op).style as DWORD,
                        handler,
                        context,
                    ) != 0
                    {
                        break; /* success */
                    }
                    /* try to extract it from the cabinet file */
                    if !(*op).src_tag.is_null() {
                        if extract_cabinet_file(
                            (*op).src_tag,
                            (*op).src_root,
                            (*op).src_file,
                            paths.Target,
                        ) != 0
                        {
                            break;
                        }
                    }
                    paths.Win32Error = GetLastError();
                    op_result = (handler.unwrap_unchecked())(
                        context,
                        SPFILENOTIFY_COPYERROR,
                        core::ptr::addr_of_mut!(paths) as UINT_PTR,
                        newpath.as_mut_ptr() as UINT_PTR,
                    );
                    if op_result == FILEOP_ABORT {
                        break 'done;
                    }
                }
                if !(*op).dst_sd.is_null() {
                    let mut psidOwner: PSID = core::ptr::null_mut();
                    let mut psidGroup: PSID = core::ptr::null_mut();
                    let mut pDacl: PACL = core::ptr::null_mut();
                    let mut pSacl: PACL = core::ptr::null_mut();
                    let mut security_info: SECURITY_INFORMATION = 0;
                    let mut present: BOOL = 0;
                    let mut dummy: BOOL = 0;

                    if GetSecurityDescriptorOwner(
                        (*op).dst_sd,
                        core::ptr::addr_of_mut!(psidOwner),
                        core::ptr::addr_of_mut!(dummy),
                    ) != 0
                        && !psidOwner.is_null()
                    {
                        security_info |= OWNER_SECURITY_INFORMATION;
                    }
                    if GetSecurityDescriptorGroup(
                        (*op).dst_sd,
                        core::ptr::addr_of_mut!(psidGroup),
                        core::ptr::addr_of_mut!(dummy),
                    ) != 0
                        && !psidGroup.is_null()
                    {
                        security_info |= GROUP_SECURITY_INFORMATION;
                    }
                    if GetSecurityDescriptorDacl(
                        (*op).dst_sd,
                        core::ptr::addr_of_mut!(present),
                        core::ptr::addr_of_mut!(pDacl),
                        core::ptr::addr_of_mut!(dummy),
                    ) != 0
                    {
                        security_info |= DACL_SECURITY_INFORMATION;
                    }
                    if GetSecurityDescriptorSacl(
                        (*op).dst_sd,
                        core::ptr::addr_of_mut!(present),
                        core::ptr::addr_of_mut!(pSacl),
                        core::ptr::addr_of_mut!(dummy),
                    ) != 0
                    {
                        security_info |= DACL_SECURITY_INFORMATION;
                    }
                    SetNamedSecurityInfoW(
                        paths.Target as LPWSTR,
                        SE_FILE_OBJECT,
                        security_info,
                        psidOwner,
                        psidGroup,
                        pDacl,
                        pSacl,
                    );
                    /* Yes, ignore the return code... */
                }
                (handler.unwrap_unchecked())(
                    context,
                    SPFILENOTIFY_ENDCOPY,
                    core::ptr::addr_of_mut!(paths) as UINT_PTR,
                    0,
                );
                op = (*op).next;
            }
            (handler.unwrap_unchecked())(context, SPFILENOTIFY_ENDSUBQUEUE, (FILEOP_COPY as _), 0);
        }

        result = (TRUE as i32);
    }

    /* done: */
    (handler.unwrap_unchecked())(context, SPFILENOTIFY_ENDQUEUE, result as UINT_PTR, 0);
    HeapFree(GetProcessHeap(), 0, paths.Source as PVOID);
    HeapFree(GetProcessHeap(), 0, paths.Target as PVOID);
    return result;
}

#[no_mangle]
pub unsafe extern "system" fn SetupScanFileQueueA(
    mut handle: HSPFILEQ,
    mut flags: DWORD,
    mut window: HWND,
    mut handler: PSP_FILE_CALLBACK_A,
    mut context: PVOID,
    mut result: PDWORD,
) -> BOOL {
    let mut ctx: callback_WtoA_context = callback_WtoA_context {
        orig_context: core::ptr::null_mut(),
        orig_handler: None,
    };
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/queue.c\0"
                as *const u8 as *const i8,
            b"SetupScanFileQueueA\0" as *const u8 as *const i8,
            1498 as i32,
            b"%p %x %p %p %p %p\n\0" as *const u8 as *const i8,
            handle,
            flags,
            window,
            handler,
            context,
            result,
        );
    }
    ctx.orig_context = context as *mut core::ffi::c_void;
    ctx.orig_handler = handler;
    return SetupScanFileQueueW(
        handle,
        flags,
        window,
        Some(
            QUEUE_callback_WtoA,
        ),
        core::ptr::addr_of_mut!(ctx) as PVOID,
        result,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetupScanFileQueueW(
    handle: HSPFILEQ,
    flags: DWORD,
    window: HWND,
    handler: PSP_FILE_CALLBACK_W,
    context: PVOID,
    result: PDWORD,
) -> BOOL {
    let queue: *mut file_queue = handle as *mut file_queue;
    let mut op: *mut file_op;
    let mut paths: FILEPATHS_W = core::mem::zeroed();
    let mut notification: UINT = 0;
    let mut ret: BOOL = (FALSE as i32);

    *result = FALSE;

    if (*queue).copy_queue.count == 0 {
        return (TRUE) as BOOL;
    }

    if (flags & SPQ_SCAN_USE_CALLBACK) != 0 {
        notification = SPFILENOTIFY_QUEUESCAN;
    } else if (flags & SPQ_SCAN_USE_CALLBACKEX) != 0 {
        notification = SPFILENOTIFY_QUEUESCAN_EX;
    }

    if (flags & !(SPQ_SCAN_USE_CALLBACK | SPQ_SCAN_USE_CALLBACKEX)) != 0 {
    }

    paths.Source = core::ptr::null_mut();
    paths.Target = core::ptr::null_mut();

    op = (*queue).copy_queue.head;
    'done: {
        while !op.is_null() {
            build_filepathsW(op, core::ptr::addr_of_mut!(paths));
            if notification == SPFILENOTIFY_QUEUESCAN {
                /* FIXME: handle delay flag */
                if (handler.unwrap_unchecked())(
                    context,
                    notification,
                    paths.Target as UINT_PTR,
                    0,
                ) != 0
                {
                    break 'done;
                }
            } else if notification == SPFILENOTIFY_QUEUESCAN_EX {
                if (handler.unwrap_unchecked())(
                    context,
                    notification,
                    core::ptr::addr_of_mut!(paths) as UINT_PTR,
                    0,
                ) != 0
                {
                    break 'done;
                }
            } else {
                ret = (TRUE as i32);
                break 'done;
            }
            op = (*op).next;
        }

        *result = TRUE;
    }

    HeapFree(
        GetProcessHeap(),
        0,
        paths.Source as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        paths.Target as PVOID,
    );
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetFileQueueCount(handle: HSPFILEQ, op: UINT, result: PUINT) -> BOOL {
    let queue: *mut file_queue = handle as *mut file_queue;

    match op {
        FILEOP_COPY => {
            *result = (*queue).copy_queue.count;
            return (TRUE) as BOOL;
        }
        FILEOP_RENAME => {
            *result = (*queue).rename_queue.count;
            return (TRUE) as BOOL;
        }
        FILEOP_DELETE => {
            *result = (*queue).delete_queue.count;
            return (TRUE) as BOOL;
        }
        _ => {}
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupGetFileQueueFlags(handle: HSPFILEQ, flags: PDWORD) -> BOOL {
    let queue: *mut file_queue = handle as *mut file_queue;
    *flags = (*queue).flags;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupSetFileQueueFlags(handle: HSPFILEQ, mask: DWORD, flags: DWORD) -> BOOL {
    let queue: *mut file_queue = handle as *mut file_queue;
    (*queue).flags = ((*queue).flags & !mask) | flags;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupSetFileQueueAlternatePlatformA(
    handle: HSPFILEQ,
    platform: PSP_ALTPLATFORM_INFO,
    catalogfile: PCSTR,
) -> BOOL {
    // FIXME("(%p, %p, %s) stub!\n", handle, platform, debugstr_a(catalogfile));
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SetupSetFileQueueAlternatePlatformW(
    handle: HSPFILEQ,
    platform: PSP_ALTPLATFORM_INFO,
    catalogfile: PCWSTR,
) -> BOOL {
    // FIXME("(%p, %p, %s) stub!\n", handle, platform, debugstr_w(catalogfile));
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SetupInitDefaultQueueCallback(owner: HWND) -> LPVOID {
    return SetupInitDefaultQueueCallbackEx(owner, core::ptr::null_mut(), 0, 0, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SetupInitDefaultQueueCallbackEx(
    owner: HWND,
    progress: HWND,
    msg: UINT,
    reserved1: DWORD,
    reserved2: PVOID,
) -> PVOID {
    let mut context: *mut default_callback_context;

    context = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<default_callback_context>() as SIZE_T) as *mut default_callback_context;
    if !context.is_null() {
        (*context).owner = owner;
        (*context).progress = progress;
        (*context).message = msg;
    }
    return context as PVOID;
}

#[no_mangle]
pub unsafe extern "system" fn SetupTermDefaultQueueCallback(context: LPVOID) {
    HeapFree(GetProcessHeap(), 0, context);
}

#[no_mangle]
pub unsafe extern "system" fn SetupDefaultQueueCallbackA(
    context: PVOID,
    notification: UINT,
    param1: UINT_PTR,
    param2: UINT_PTR,
) -> UINT {
    let paths: *mut FILEPATHS_A = param1 as *mut FILEPATHS_A;
    let ctx: *mut default_callback_context = context as *mut default_callback_context;

    match notification {
        SPFILENOTIFY_STARTQUEUE => {
            TRACE!("start queue\n");
            return TRUE;
        }
        SPFILENOTIFY_ENDQUEUE => {
            TRACE!("end queue\n");
            return 0;
        }
        SPFILENOTIFY_STARTSUBQUEUE => {
            TRACE!("start subqueue %ld count %ld\n", param1, param2);
            return TRUE;
        }
        SPFILENOTIFY_ENDSUBQUEUE => {
            TRACE!("end subqueue %ld\n", param1);
            return 0;
        }
        SPFILENOTIFY_STARTDELETE => {
            TRACE!("start delete %s\n", debugstr_a((*paths).Target));
            return FILEOP_DOIT;
        }
        SPFILENOTIFY_ENDDELETE => {
            TRACE!("end delete %s\n", debugstr_a((*paths).Target));
            return 0;
        }
        SPFILENOTIFY_DELETEERROR => {
            /*Windows Ignores attempts to delete files / folders which do not exist*/
            if ((*paths).Win32Error != ERROR_FILE_NOT_FOUND) && ((*paths).Win32Error != ERROR_PATH_NOT_FOUND) {
                SetupDeleteErrorA(
                    (*ctx).owner,
                    core::ptr::null(),
                    (*paths).Target,
                    (*paths).Win32Error,
                    0,
                );
            }
            return FILEOP_SKIP;
        }
        SPFILENOTIFY_STARTRENAME => {
            TRACE!(
                "start rename %s -> %s\n",
                debugstr_a((*paths).Source),
                debugstr_a((*paths).Target)
            );
            return FILEOP_DOIT;
        }
        SPFILENOTIFY_ENDRENAME => {
            TRACE!(
                "end rename %s -> %s\n",
                debugstr_a((*paths).Source),
                debugstr_a((*paths).Target)
            );
            return 0;
        }
        SPFILENOTIFY_RENAMEERROR => {
            SetupRenameErrorA(
                (*ctx).owner,
                core::ptr::null(),
                (*paths).Source,
                (*paths).Target,
                (*paths).Win32Error,
                0,
            );
            return FILEOP_SKIP;
        }
        SPFILENOTIFY_STARTCOPY => {
            TRACE!(
                "start copy %s -> %s\n",
                debugstr_a((*paths).Source),
                debugstr_a((*paths).Target)
            );
            return FILEOP_DOIT;
        }
        SPFILENOTIFY_ENDCOPY => {
            TRACE!(
                "end copy %s -> %s\n",
                debugstr_a((*paths).Source),
                debugstr_a((*paths).Target)
            );
            return 0;
        }
        SPFILENOTIFY_COPYERROR => {
            ERR!(
                "copy error %d %s -> %s\n",
                (*paths).Win32Error,
                debugstr_a((*paths).Source),
                debugstr_a((*paths).Target)
            );
            return FILEOP_SKIP;
        }
        SPFILENOTIFY_NEEDMEDIA => {
            TRACE!("need media\n");
            return FILEOP_SKIP;
        }
        _ => {
            FIXME!(
                "notification %d params %lx,%lx\n",
                notification,
                param1,
                param2
            );
        }
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDefaultQueueCallbackW(
    context: PVOID,
    notification: UINT,
    param1: UINT_PTR,
    param2: UINT_PTR,
) -> UINT {
    let paths: *mut FILEPATHS_W = param1 as *mut FILEPATHS_W;
    let ctx: *mut default_callback_context = context as *mut default_callback_context;

    match notification {
        SPFILENOTIFY_STARTQUEUE => {
            TRACE!("start queue\n");
            return TRUE;
        }
        SPFILENOTIFY_ENDQUEUE => {
            TRACE!("end queue\n");
            return 0;
        }
        SPFILENOTIFY_STARTSUBQUEUE => {
            TRACE!("start subqueue %ld count %ld\n", param1, param2);
            return TRUE;
        }
        SPFILENOTIFY_ENDSUBQUEUE => {
            TRACE!("end subqueue %ld\n", param1);
            return 0;
        }
        SPFILENOTIFY_STARTDELETE => {
            TRACE!(
                "start delete %s\n",
                debugstr_w((*paths).Target as *const WCHAR)
            );
            return FILEOP_DOIT;
        }
        SPFILENOTIFY_ENDDELETE => {
            TRACE!(
                "end delete %s\n",
                debugstr_w((*paths).Target as *const WCHAR)
            );
            return 0;
        }
        SPFILENOTIFY_DELETEERROR => {
            /*Windows Ignores attempts to delete files / folders which do not exist*/
            if (*paths).Win32Error != ERROR_FILE_NOT_FOUND && (*paths).Win32Error != ERROR_PATH_NOT_FOUND {
                SetupDeleteErrorW(
                    (*ctx).owner,
                    core::ptr::null::<WCHAR>(),
                    (*paths).Target,
                    (*paths).Win32Error,
                    0,
                );
            }
            return FILEOP_SKIP;
        }
        SPFILENOTIFY_STARTRENAME => {
            SetupRenameErrorW(
                (*ctx).owner,
                core::ptr::null::<WCHAR>(),
                (*paths).Source,
                (*paths).Target,
                (*paths).Win32Error,
                0,
            );
            return FILEOP_DOIT;
        }
        SPFILENOTIFY_ENDRENAME => {
            TRACE!(
                "end rename %s -> %s\n",
                debugstr_w((*paths).Source as *const WCHAR),
                debugstr_w((*paths).Target as *const WCHAR)
            );
            return 0;
        }
        SPFILENOTIFY_RENAMEERROR => {
            ERR!(
                "rename error %d %s -> %s\n",
                (*paths).Win32Error,
                debugstr_w((*paths).Source as *const WCHAR),
                debugstr_w((*paths).Target as *const WCHAR)
            );
            return FILEOP_SKIP;
        }
        SPFILENOTIFY_STARTCOPY => {
            TRACE!(
                "start copy %s -> %s\n",
                debugstr_w((*paths).Source as *const WCHAR),
                debugstr_w((*paths).Target as *const WCHAR)
            );
            return FILEOP_DOIT;
        }
        SPFILENOTIFY_ENDCOPY => {
            TRACE!(
                "end copy %s -> %s\n",
                debugstr_w((*paths).Source as *const WCHAR),
                debugstr_w((*paths).Target as *const WCHAR)
            );
            return 0;
        }
        SPFILENOTIFY_COPYERROR => {
            TRACE!(
                "copy error %d %s -> %s\n",
                (*paths).Win32Error,
                debugstr_w((*paths).Source as *const WCHAR),
                debugstr_w((*paths).Target as *const WCHAR)
            );
            return FILEOP_SKIP;
        }
        SPFILENOTIFY_NEEDMEDIA => {
            TRACE!("need media\n");
            return FILEOP_SKIP;
        }
        _ => {
            FIXME!(
                "notification %d params %lx,%lx\n",
                notification,
                param1,
                param2
            );
        }
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDeleteErrorA(
    parent: HWND,
    dialogTitle: PCSTR,
    file: PCSTR,
    w32error: UINT,
    style: DWORD,
) -> UINT {
    // FIXME( "stub: (Error Number %d when attempting to delete %s)\n", w32error, debugstr_a(file) );
    DPROMPT_SKIPFILE
}

#[no_mangle]
pub unsafe extern "system" fn SetupDeleteErrorW(
    parent: HWND,
    dialogTitle: PCWSTR,
    file: PCWSTR,
    w32error: UINT,
    style: DWORD,
) -> UINT {
    // FIXME( "stub: (Error Number %d when attempting to delete %s)\n", w32error, debugstr_w(file) );
    DPROMPT_SKIPFILE
}

#[no_mangle]
pub unsafe extern "system" fn SetupRenameErrorA(
    parent: HWND,
    dialogTitle: PCSTR,
    source: PCSTR,
    target: PCSTR,
    w32error: UINT,
    style: DWORD,
) -> UINT {
    // FIXME( "stub: (Error Number %d when attempting to rename %s to %s)\n", w32error, debugstr_a(source), debugstr_a(target));
    DPROMPT_SKIPFILE
}

#[no_mangle]
pub unsafe extern "system" fn SetupRenameErrorW(
    parent: HWND,
    dialogTitle: PCWSTR,
    source: PCWSTR,
    target: PCWSTR,
    w32error: UINT,
    style: DWORD,
) -> UINT {
    // FIXME( "stub: (Error Number %d when attempting to rename %s to %s)\n", w32error, debugstr_w(source), debugstr_w(target));
    DPROMPT_SKIPFILE
}

#[no_mangle]
pub unsafe extern "system" fn SetupCopyErrorA(
    parent: HWND,
    dialogTitle: PCSTR,
    diskname: PCSTR,
    sourcepath: PCSTR,
    sourcefile: PCSTR,
    targetpath: PCSTR,
    w32error: UINT,
    style: DWORD,
    pathbuffer: PSTR,
    buffersize: DWORD,
    requiredsize: PDWORD,
) -> UINT {
    // FIXME( "stub: (Error Number %d when attempting to copy file %s from %s to %s)\n", w32error, debugstr_a(sourcefile), debugstr_a(sourcepath) ,debugstr_a(targetpath));
    DPROMPT_SKIPFILE
}

#[no_mangle]
pub unsafe extern "system" fn SetupCopyErrorW(
    parent: HWND,
    dialogTitle: PCWSTR,
    diskname: PCWSTR,
    sourcepath: PCWSTR,
    sourcefile: PCWSTR,
    targetpath: PCWSTR,
    w32error: UINT,
    style: DWORD,
    pathbuffer: PWSTR,
    buffersize: DWORD,
    requiredsize: PDWORD,
) -> UINT {
    // FIXME( "stub: (Error Number %d when attempting to copy file %s from %s to %s)\n", w32error, debugstr_w(sourcefile), debugstr_w(sourcepath) ,debugstr_w(targetpath));
    DPROMPT_SKIPFILE
}

#[no_mangle]
pub unsafe extern "system" fn pSetupGetQueueFlags(handle: HSPFILEQ) -> DWORD {
    let queue: *mut file_queue = handle as *mut file_queue;
    return (*queue).flags;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupSetQueueFlags(handle: HSPFILEQ, flags: DWORD) -> BOOL {
    let queue: *mut file_queue = handle as *mut file_queue;
    (*queue).flags = flags;
    return (TRUE) as BOOL;
}
