use winedebug_rust::{TRACE};
use ntdll_sys::nt::*;
use core::ffi::c_void;
use crate::string::nls_subst::*;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut gDebugChannel: ULONG = 203 as ULONG;

// BOOL (int) TRUE/FALSE — shadow the u32 winnt FALSE/TRUE from nt::* so they
// match the i32 BOOL fields/locals here (same pattern as lcformat.rs).
const TRUE: BOOL = 1;
const FALSE: BOOL = 0;

// ---- kernel32 / NT externs format_msg.c calls (real stdcall @N exports) ----
// These are declared as *private* extern "system" blocks in the sibling string
// modules (nls.rs/locale.rs/lcformat.rs), so they cannot be imported; declare
// the ones this file needs locally, copying the canonical signatures.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn SetLastError(dwErrCode: DWORD);
    fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
    fn LocalAlloc(uFlags: UINT, uBytes: SIZE_T) -> HLOCAL;
    fn MultiByteToWideChar(
        CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: INT,
        lpWideCharStr: LPWSTR, cchWideChar: INT,
    ) -> INT;
    fn WideCharToMultiByte(
        CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: INT,
        lpMultiByteStr: LPSTR, cbMultiByte: INT, lpDefaultChar: LPCSTR, lpUsedDefaultChar: LPBOOL,
    ) -> INT;
}
extern "C" {
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    fn memmove(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    // snprintfW == _snwprintf (msvcrt). Variadic; resolves to the real ntdll export.
    // count is size_t (usize) — matches nls_subst/lcformat. Declaring it c_int here
    // fed garbage upper bits to the variadic callee on 64-bit (amd64/arm64).
    #[link_name = "_snwprintf"]
    fn snprintfW(buffer: *mut WCHAR, count: usize, format: *const WCHAR, ...) -> i32;
}

// `__ms_va_list` on the gnullvm/gnu targets is the compiler's MS-variadic
// va_list, which is a plain char pointer walking the stack-passed argument
// slots (matches bindgen's `__builtin_va_list = *mut c_char`).
#[allow(non_camel_case_types)]
type __ms_va_list = *mut ::core::ffi::c_char;

// va_arg(*list, ULONG_PTR): read one pointer-sized slot at the current cursor
// and advance the cursor by sizeof(ULONG_PTR). This is exactly what the MS
// variadic ABI builtin does (all variadic args occupy stack slots).
#[inline]
unsafe fn va_arg_ulong_ptr(list: *mut __ms_va_list) -> ULONG_PTR {
    let cur = *list;
    let val = core::ptr::read_unaligned(cur as *const ULONG_PTR);
    *list = cur.add(core::mem::size_of::<ULONG_PTR>());
    val
}

// HRESULT_SEVERITY / HRESULT_FACILITY helpers (winerror.h macros).
#[inline]
fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}

// isdigitW (wine/unicode.h): ASCII '0'..'9'.
#[inline]
fn isdigitW(ch: WCHAR) -> i32 {
    (ch >= '0' as WCHAR && ch <= '9' as WCHAR) as i32
}

#[repr(C)]
pub struct format_args {
    pub args: *mut ULONG_PTR,
    pub list: *mut __ms_va_list,
    pub last: i32,
}

#[repr(C)]
pub struct _format_message_data {
    pub formatted: LPWSTR,
    pub size: DWORD,
    pub t: LPWSTR,
    pub space: LPWSTR,
    pub inspace: BOOL,
    pub width: DWORD,
    pub w: DWORD,
}
unsafe fn load_message(mut module: HMODULE, id: UINT, lang: WORD) -> LPWSTR {
    let mut mre: *mut MESSAGE_RESOURCE_ENTRY = core::ptr::null_mut();
    let mut buffer: *mut WCHAR = core::ptr::null_mut();
    let status: NTSTATUS;

    TRACE!(b"module = %p, id = %08x\n\0" as *const u8 as *const i8, module, id);

    if module.is_null() {
        module = GetModuleHandleW(core::ptr::null());
    }

    status = RtlFindMessage(
        module as *mut c_void,
        (11 as ULONG_PTR) as ULONG,
        lang as ULONG,
        id as ULONG,
        &mut mre as *mut *mut MESSAGE_RESOURCE_ENTRY,
    );

    if !nt_success(status) {
        SetLastError(RtlNtStatusToDosError(status));
        return core::ptr::null_mut();
    }

    if (*mre).Flags as i32 & MESSAGE_RESOURCE_UNICODE as i32 != 0 {
        let len: i32 = ((strlenW((*mre).Text.as_ptr() as *const WCHAR) as usize + 1)
            * core::mem::size_of::<WCHAR>()) as i32;
        buffer = HeapAlloc(GetProcessHeap(), 0, len as usize) as *mut WCHAR;
        if buffer.is_null() {
            return core::ptr::null_mut();
        }
        memcpy(
            buffer as *mut core::ffi::c_void,
            (*mre).Text.as_ptr() as *const core::ffi::c_void,
            len as usize,
        );
    } else {
        let len: i32 = MultiByteToWideChar(
            CP_ACP as UINT,
            0,
            (*mre).Text.as_ptr() as *const i8,
            -1,
            core::ptr::null_mut(),
            0,
        );
        buffer = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<WCHAR>()) as *mut WCHAR;
        if buffer.is_null() {
            return core::ptr::null_mut();
        }
        MultiByteToWideChar(
            CP_ACP as UINT,
            0,
            (*mre).Text.as_ptr() as *const i8,
            -1,
            buffer,
            len,
        );
    }

    TRACE!(b"returning %s\n\0" as *const u8 as *const i8, wine_dbgstr_w(buffer));
    buffer
}
unsafe fn search_message(
    mut flags: DWORD,
    mut module: HMODULE,
    mut id: UINT,
    mut lang: WORD,
) -> LPWSTR {
    let mut from: LPWSTR = core::ptr::null_mut();
    if flags & FORMAT_MESSAGE_FROM_HMODULE != 0 {
        from = load_message(module, id, lang);
    }
    if from.is_null() && flags & FORMAT_MESSAGE_FROM_SYSTEM != 0 {
        if (id >> 31) & 0x1 == SEVERITY_ERROR as UINT
            && (id >> 16) & 0x1fff == FACILITY_WIN32 as UINT
        {
            id = (id & 0xffff) as UINT;
        }
        from = load_message(kernel32_handle, id, lang);
    }
    return from;
}
unsafe fn get_arg(
    mut nr: ::core::ffi::c_int,
    _flags: DWORD,
    mut args: *mut format_args,
) -> ULONG_PTR {
    if nr == -1 {
        nr = (*args).last + 1;
    }
    if !(*args).list.is_null() {
        if (*args).args.is_null() {
            (*args).args = HeapAlloc(
                GetProcessHeap(),
                0,
                (99 as usize).wrapping_mul(::core::mem::size_of::<ULONG_PTR>()),
            ) as *mut ULONG_PTR;
        }
        while nr > (*args).last {
            let fresh21 = (*args).last;
            (*args).last = (*args).last + 1;
            *(*args).args.offset(fresh21 as isize) = va_arg_ulong_ptr((*args).list);
        }
    }
    if nr > (*args).last {
        (*args).last = nr;
    }
    return *(*args).args.offset((nr - 1) as isize);
}
unsafe fn format_insert(
    mut unicode_caller: BOOL,
    mut insert: ::core::ffi::c_int,
    mut format: LPCWSTR,
    mut flags: DWORD,
    mut args: *mut format_args,
    mut result: *mut LPWSTR,
) -> LPCWSTR {
    static mut fmt_u: [WCHAR; 3] = ['%' as WCHAR, 'u' as WCHAR, 0 as WCHAR];
    let mut wstring: *mut WCHAR = core::ptr::null_mut();
    let mut p: *mut WCHAR = core::ptr::null_mut();
    let mut fmt: [WCHAR; 256] = [0; 256];
    let mut arg: ULONG_PTR = 0;
    let mut size: ::core::ffi::c_int = 0;

    if *format as ::core::ffi::c_int != '!' as i32 {
        arg = get_arg(insert, flags, args);
        if unicode_caller != 0 || arg == 0 {
            static mut nullW: [WCHAR; 7] = [
                '(' as WCHAR, 'n' as WCHAR, 'u' as WCHAR, 'l' as WCHAR,
                'l' as WCHAR, ')' as WCHAR, 0 as WCHAR,
            ];
            let mut str: *const WCHAR = arg as *const WCHAR;
            if str.is_null() {
                str = &raw const nullW as *const WCHAR;
            }
            *result = HeapAlloc(
                GetProcessHeap(),
                0,
                (strlenW(str) as usize + 1) * core::mem::size_of::<WCHAR>(),
            ) as LPWSTR;
            strcpyW(*result, str);
        } else {
            let mut str_0: *const ::core::ffi::c_char = arg as *const ::core::ffi::c_char;
            let mut length: DWORD = MultiByteToWideChar(
                CP_ACP,
                0,
                str_0,
                -1,
                core::ptr::null_mut(),
                0,
            ) as DWORD;
            *result = HeapAlloc(
                GetProcessHeap(),
                0,
                (length as usize) * core::mem::size_of::<WCHAR>(),
            ) as LPWSTR;
            MultiByteToWideChar(
                CP_ACP,
                0,
                str_0,
                -1,
                *result,
                length as ::core::ffi::c_int,
            );
        }
        return format;
    }

    format = format.offset(1);
    p = &raw mut fmt as *mut WCHAR;
    let fresh6 = p;
    p = p.offset(1);
    *fresh6 = '%' as WCHAR;

    while *format as ::core::ffi::c_int == '0' as i32
        || *format as ::core::ffi::c_int == '+' as i32
        || *format as ::core::ffi::c_int == '-' as i32
        || *format as ::core::ffi::c_int == ' ' as i32
        || *format as ::core::ffi::c_int == '*' as i32
        || *format as ::core::ffi::c_int == '#' as i32
    {
        if *format as ::core::ffi::c_int == '*' as i32 {
            p = p.offset(
                sprintfW(
                    p,
                    &raw const fmt_u as *const WCHAR,
                    get_arg(insert, flags, args) as u32,
                ) as isize,
            );
            insert = -1;
            format = format.offset(1);
        } else {
            let fresh7 = format;
            format = format.offset(1);
            let fresh8 = p;
            p = p.offset(1);
            *fresh8 = *fresh7;
        }
    }

    while isdigitW(*format) != 0 {
        let fresh9 = format;
        format = format.offset(1);
        let fresh10 = p;
        p = p.offset(1);
        *fresh10 = *fresh9;
    }

    if *format as ::core::ffi::c_int == '.' as i32 {
        let fresh11 = format;
        format = format.offset(1);
        let fresh12 = p;
        p = p.offset(1);
        *fresh12 = *fresh11;
        if *format as ::core::ffi::c_int == '*' as i32 {
            p = p.offset(
                sprintfW(
                    p,
                    &raw const fmt_u as *const WCHAR,
                    get_arg(insert, flags, args) as u32,
                ) as isize,
            );
            insert = -1;
            format = format.offset(1);
        } else {
            while isdigitW(*format) != 0 {
                let fresh13 = format;
                format = format.offset(1);
                let fresh14 = p;
                p = p.offset(1);
                *fresh14 = *fresh13;
            }
        }
    }

    if insert == -1 && !(*args).list.is_null() {
        (*args).last -= 1;
    }
    arg = get_arg(insert, flags, args);

    if (*format.offset(0) as ::core::ffi::c_int == 'h' as i32
        && *format.offset(1) as ::core::ffi::c_int == 's' as i32)
        || (*format.offset(0) as ::core::ffi::c_int == 'h' as i32
            && *format.offset(1) as ::core::ffi::c_int == 'S' as i32)
        || (unicode_caller != 0
            && *format.offset(0) as ::core::ffi::c_int == 'S' as i32)
        || (unicode_caller == 0
            && *format.offset(0) as ::core::ffi::c_int == 's' as i32)
    {
        let mut len: DWORD = MultiByteToWideChar(
            CP_ACP,
            0,
            arg as *mut ::core::ffi::c_char,
            -1,
            core::ptr::null_mut(),
            0,
        ) as DWORD;
        wstring = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as usize) * core::mem::size_of::<WCHAR>(),
        ) as *mut WCHAR;
        MultiByteToWideChar(
            CP_ACP,
            0,
            arg as *mut ::core::ffi::c_char,
            -1,
            wstring,
            len as ::core::ffi::c_int,
        );
        arg = wstring as ULONG_PTR;
        let fresh15 = p;
        p = p.offset(1);
        *fresh15 = 's' as WCHAR;
    } else if (*format.offset(0) as ::core::ffi::c_int == 'h' as i32
        && *format.offset(1) as ::core::ffi::c_int == 'c' as i32)
        || (*format.offset(0) as ::core::ffi::c_int == 'h' as i32
            && *format.offset(1) as ::core::ffi::c_int == 'C' as i32)
        || (unicode_caller != 0
            && *format.offset(0) as ::core::ffi::c_int == 'C' as i32)
        || (unicode_caller == 0
            && *format.offset(0) as ::core::ffi::c_int == 'c' as i32)
    {
        let mut ch: ::core::ffi::c_char = arg as ::core::ffi::c_char;
        wstring = HeapAlloc(
            GetProcessHeap(),
            0,
            2 * core::mem::size_of::<WCHAR>(),
        ) as *mut WCHAR;
        MultiByteToWideChar(
            CP_ACP,
            0,
            &raw mut ch,
            1,
            wstring,
            1,
        );
        *wstring.offset(1) = 0 as WCHAR;
        arg = wstring as ULONG_PTR;
        let fresh16 = p;
        p = p.offset(1);
        *fresh16 = 's' as WCHAR;
    } else if (*format.offset(0) as ::core::ffi::c_int == 'l' as i32
        && *format.offset(1) as ::core::ffi::c_int == 's' as i32)
        || (*format.offset(0) as ::core::ffi::c_int == 'l' as i32
            && *format.offset(1) as ::core::ffi::c_int == 'S' as i32)
        || (*format.offset(0) as ::core::ffi::c_int == 'w' as i32
            && *format.offset(1) as ::core::ffi::c_int == 's' as i32)
        || (unicode_caller == 0
            && *format.offset(0) as ::core::ffi::c_int == 'S' as i32)
    {
        let fresh17 = p;
        p = p.offset(1);
        *fresh17 = 's' as WCHAR;
    } else if (*format.offset(0) as ::core::ffi::c_int == 'l' as i32
        && *format.offset(1) as ::core::ffi::c_int == 'c' as i32)
        || (*format.offset(0) as ::core::ffi::c_int == 'l' as i32
            && *format.offset(1) as ::core::ffi::c_int == 'C' as i32)
        || (*format.offset(0) as ::core::ffi::c_int == 'w' as i32
            && *format.offset(1) as ::core::ffi::c_int == 'c' as i32)
        || (unicode_caller == 0
            && *format.offset(0) as ::core::ffi::c_int == 'C' as i32)
    {
        let fresh18 = p;
        p = p.offset(1);
        *fresh18 = 'c' as WCHAR;
    } else {
        while *format as ::core::ffi::c_int != 0
            && *format as ::core::ffi::c_int != '!' as i32
        {
            let fresh19 = format;
            format = format.offset(1);
            let fresh20 = p;
            p = p.offset(1);
            *fresh20 = *fresh19;
        }
    }

    *p = 0 as WCHAR;
    size = 256;
    loop {
        let mut ret: *mut WCHAR = HeapAlloc(
            GetProcessHeap(),
            0,
            (size as usize) * core::mem::size_of::<WCHAR>(),
        ) as *mut WCHAR;
        let mut needed: ::core::ffi::c_int = snprintfW(
            ret,
            size as usize,
            &raw mut fmt as *mut WCHAR,
            arg,
        );
        if needed == -1 || needed >= size {
            HeapFree(GetProcessHeap(), 0, ret as *mut ::core::ffi::c_void);
            size = if needed + 1 > size * 2 {
                needed + 1
            } else {
                size * 2
            };
        } else {
            *result = ret;
            break;
        }
    }

    while *format as ::core::ffi::c_int != 0
        && *format as ::core::ffi::c_int != '!' as i32
    {
        format = format.offset(1);
    }
    if *format as ::core::ffi::c_int == '!' as i32 {
        format = format.offset(1);
    }

    HeapFree(GetProcessHeap(), 0, wstring as *mut ::core::ffi::c_void);
    return format;
}
unsafe fn format_add_char(mut fmd: *mut _format_message_data, mut c: WCHAR) {
    let fresh3 = (*fmd).t;
    (*fmd).t = (*fmd).t.offset(1);
    *fresh3 = c;
    if (*fmd).width != 0 && (*fmd).width != FORMAT_MESSAGE_MAX_WIDTH_MASK {
        match c as i32 {
            13 | 10 => {
                (*fmd).space = core::ptr::null_mut::<WCHAR>();
                (*fmd).inspace = 0;
                (*fmd).w = 0;
            }
            32 => {
                if (*fmd).inspace == 0 {
                    (*fmd).space = (*fmd).t.offset(-1);
                }
                (*fmd).inspace = 1;
                (*fmd).w = (*fmd).w.wrapping_add(1);
            }
            _ => {
                (*fmd).inspace = 0;
                (*fmd).w = (*fmd).w.wrapping_add(1);
            }
        }
        if (*fmd).w == (*fmd).width {
            let mut notspace: LPWSTR = core::ptr::null_mut::<WCHAR>();
            if !(*fmd).space.is_null() {
                notspace = (*fmd).space;
                while notspace != (*fmd).t && *notspace as i32 == ' ' as i32 {
                    notspace = notspace.offset(1);
                }
            } else {
                (*fmd).space = (*fmd).t;
                notspace = (*fmd).space;
            }
            (*fmd).w = ((*fmd).t.offset_from(notspace)) as DWORD;
            memmove(
                (*fmd).space.offset(2) as *mut core::ffi::c_void,
                notspace as *const core::ffi::c_void,
                ((*fmd).w as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
            );
            let fresh4 = (*fmd).space;
            (*fmd).space = (*fmd).space.offset(1);
            *fresh4 = '\r' as i32 as WCHAR;
            let fresh5 = (*fmd).space;
            (*fmd).space = (*fmd).space.offset(1);
            *fresh5 = '\n' as i32 as WCHAR;
            (*fmd).t = (*fmd).space.offset((*fmd).w as isize);
            (*fmd).space = core::ptr::null_mut::<WCHAR>();
            (*fmd).inspace = 0;
        }
    }
    if ((*fmd).t.offset_from((*fmd).formatted)) as DWORD == (*fmd).size {
        let ispace: DWORD_PTR = ((*fmd).space.offset_from((*fmd).formatted)) as DWORD_PTR;
        (*fmd).formatted = HeapReAlloc(
            GetProcessHeap(),
            0,
            (*fmd).formatted as LPVOID,
            (((*fmd).size as usize).wrapping_mul(2).wrapping_add(2)).wrapping_mul(core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as LPWSTR;
        (*fmd).t = (*fmd).formatted.offset((*fmd).size as isize);
        if !(*fmd).space.is_null() {
            (*fmd).space = (*fmd).formatted.offset(ispace as isize);
        }
        (*fmd).size = ((*fmd).size as usize).wrapping_mul(2) as DWORD;
    }
}
unsafe fn format_message(
    mut unicode_caller: BOOL,
    mut dwFlags: DWORD,
    mut fmtstr: LPCWSTR,
    mut format_args: *mut format_args,
) -> LPWSTR {
    let mut fmd: _format_message_data = _format_message_data {
        formatted: core::ptr::null_mut::<WCHAR>(),
        size: 0,
        t: core::ptr::null_mut::<WCHAR>(),
        space: core::ptr::null_mut::<WCHAR>(),
        inspace: 0,
        width: 0,
        w: 0,
    };
    let mut f: LPCWSTR = core::ptr::null::<WCHAR>();
    let mut eos: BOOL = FALSE;

    fmd.size = 100;
    fmd.t = HeapAlloc(GetProcessHeap(), 0, (fmd.size as usize + 2) * core::mem::size_of::<WCHAR>()) as LPWSTR;
    fmd.formatted = fmd.t;

    fmd.width = dwFlags & FORMAT_MESSAGE_MAX_WIDTH_MASK;
    fmd.w = 0;
    fmd.inspace = FALSE;
    fmd.space = core::ptr::null_mut::<WCHAR>();
    f = fmtstr;

    while *f != 0 && eos == 0 {
        if *f as i32 == '%' as i32 {
            let mut insertnr: i32 = 0;
            let mut str: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
            let mut x: *mut WCHAR = core::ptr::null_mut::<WCHAR>();

            f = f.offset(1);
            let mut current_block: u64;

            match *f as i32 {
                49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
                    if dwFlags & FORMAT_MESSAGE_IGNORE_INSERTS != 0 {
                        current_block = 9094421243039455209;
                    } else {
                        if ((dwFlags & FORMAT_MESSAGE_ARGUMENT_ARRAY) != 0 && (*format_args).args.is_null())
                            || ((dwFlags & FORMAT_MESSAGE_ARGUMENT_ARRAY) == 0 && (*format_args).list.is_null())
                        {
                            SetLastError(ERROR_INVALID_PARAMETER);
                            HeapFree(GetProcessHeap(), 0, fmd.formatted as LPVOID);
                            return core::ptr::null_mut::<WCHAR>();
                        }
                        insertnr = *f as i32 - '0' as i32;
                        match *f.offset(1) as i32 {
                            48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
                                f = f.offset(1);
                                insertnr = insertnr * 10 + *f as i32 - '0' as i32;
                                f = f.offset(1);
                            }
                            _ => {
                                f = f.offset(1);
                            }
                        }
                        f = format_insert(
                            unicode_caller,
                            insertnr,
                            f,
                            dwFlags,
                            format_args,
                            &raw mut str,
                        );
                        x = str;
                        while *x != 0 {
                            format_add_char(&raw mut fmd, *x);
                            x = x.offset(1);
                        }
                        HeapFree(GetProcessHeap(), 0, str as LPVOID);
                        current_block = 9441801433784995173;
                    }
                }
                110 => {
                    format_add_char(&raw mut fmd, '\r' as i32 as WCHAR);
                    format_add_char(&raw mut fmd, '\n' as i32 as WCHAR);
                    f = f.offset(1);
                    current_block = 9441801433784995173;
                }
                114 => {
                    format_add_char(&raw mut fmd, '\r' as i32 as WCHAR);
                    f = f.offset(1);
                    current_block = 9441801433784995173;
                }
                116 => {
                    format_add_char(&raw mut fmd, '\t' as i32 as WCHAR);
                    f = f.offset(1);
                    current_block = 9441801433784995173;
                }
                48 => {
                    eos = TRUE;
                    f = f.offset(1);
                    current_block = 9441801433784995173;
                }
                0 => {
                    SetLastError(ERROR_INVALID_PARAMETER);
                    HeapFree(GetProcessHeap(), 0, fmd.formatted as LPVOID);
                    return core::ptr::null_mut::<WCHAR>();
                }
                _ => {
                    current_block = 9094421243039455209;
                }
            }

            match current_block {
                9094421243039455209 => {
                    if dwFlags & FORMAT_MESSAGE_IGNORE_INSERTS != 0 {
                        format_add_char(&raw mut fmd, '%' as i32 as WCHAR);
                    }
                    let fresh2 = f;
                    f = f.offset(1);
                    format_add_char(&raw mut fmd, *fresh2);
                }
                _ => {}
            }
        } else {
            let mut ch: WCHAR = *f;
            f = f.offset(1);
            if ch as i32 == '\r' as i32 {
                if *f as i32 == '\n' as i32 {
                    f = f.offset(1);
                }
                if fmd.width != 0 {
                    format_add_char(&raw mut fmd, ' ' as i32 as WCHAR);
                } else {
                    format_add_char(&raw mut fmd, '\r' as i32 as WCHAR);
                    format_add_char(&raw mut fmd, '\n' as i32 as WCHAR);
                }
            } else if ch as i32 == '\n' as i32 {
                if fmd.width != 0 {
                    format_add_char(&raw mut fmd, ' ' as i32 as WCHAR);
                } else {
                    format_add_char(&raw mut fmd, '\r' as i32 as WCHAR);
                    format_add_char(&raw mut fmd, '\n' as i32 as WCHAR);
                }
            } else {
                format_add_char(&raw mut fmd, ch);
            }
        }
    }

    *fmd.t = '\0' as i32 as WCHAR;
    return fmd.formatted;
}

#[no_mangle]
pub unsafe extern "system" fn FormatMessageA(
    dwFlags: DWORD,
    lpSource: LPCVOID,
    dwMessageId: DWORD,
    dwLanguageId: DWORD,
    lpBuffer: LPSTR,
    nSize: DWORD,
    args: *mut __ms_va_list,
) -> DWORD {
    let mut format_args: format_args = core::mem::zeroed();
    let mut ret: DWORD = 0;
    let mut target: LPWSTR = core::ptr::null_mut();
    let mut destlength: DWORD = 0;
    let mut from: LPWSTR = core::ptr::null_mut();

    TRACE!("(0x%x,%p,%d,0x%x,%p,%d,%p)\n", dwFlags, lpSource, dwMessageId, dwLanguageId, lpBuffer, nSize, args);

    if dwFlags & FORMAT_MESSAGE_ALLOCATE_BUFFER != 0 {
        if lpBuffer.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        } else {
            *(lpBuffer as *mut LPSTR) = core::ptr::null_mut();
        }
    }

    if dwFlags & FORMAT_MESSAGE_ARGUMENT_ARRAY != 0 {
        format_args.args = args as *mut ULONG_PTR;
        format_args.list = core::ptr::null_mut();
        format_args.last = 0;
    } else {
        format_args.args = core::ptr::null_mut();
        format_args.list = args;
        format_args.last = 0;
    }

    from = core::ptr::null_mut();
    if dwFlags & FORMAT_MESSAGE_FROM_STRING != 0 {
        let length: DWORD = MultiByteToWideChar(
            CP_ACP,
            0,
            lpSource as LPCSTR,
            -1,
            core::ptr::null_mut(),
            0,
        ) as DWORD;
        from = HeapAlloc(
            GetProcessHeap(),
            0,
            (length as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        ) as LPWSTR;
        MultiByteToWideChar(CP_ACP, 0, lpSource as LPCSTR, -1, from, length as i32);
    } else if dwFlags & (FORMAT_MESSAGE_FROM_HMODULE | FORMAT_MESSAGE_FROM_SYSTEM) != 0 {
        from = search_message(
            dwFlags,
            lpSource as HMODULE,
            dwMessageId as UINT,
            dwLanguageId as WORD,
        );
        if from.is_null() {
            return 0;
        }
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    target = format_message(0, dwFlags, from as LPCWSTR, &raw mut format_args);
    if !target.is_null() {
        TRACE!("-- %s\n", debugstr_w(target));

        if *target != 0 {
            destlength = WideCharToMultiByte(
                CP_ACP,
                0,
                target as LPCWSTR,
                -1,
                core::ptr::null_mut(),
                0,
                core::ptr::null(),
                core::ptr::null_mut(),
            ) as DWORD;

            if dwFlags & FORMAT_MESSAGE_ALLOCATE_BUFFER != 0 {
                let buf: LPSTR = LocalAlloc(
                    LMEM_ZEROINIT,
                    (if nSize > destlength { nSize } else { destlength }) as SIZE_T,
                ) as LPSTR;
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    target as LPCWSTR,
                    -1,
                    buf,
                    destlength as i32,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
                *(lpBuffer as *mut LPSTR) = buf;
            } else {
                if nSize < destlength {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER);
                } else {
                    WideCharToMultiByte(
                        CP_ACP,
                        0,
                        target as LPCWSTR,
                        -1,
                        lpBuffer,
                        destlength as i32,
                        core::ptr::null(),
                        core::ptr::null_mut(),
                    );
                    ret = destlength.wrapping_sub(1);
                }
            }
        }
    }

    HeapFree(GetProcessHeap(), 0, target as LPVOID);
    HeapFree(GetProcessHeap(), 0, from as LPVOID);
    if dwFlags & FORMAT_MESSAGE_ARGUMENT_ARRAY == 0 {
        HeapFree(GetProcessHeap(), 0, format_args.args as LPVOID);
    }

    TRACE!("-- returning %u\n", ret);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn FormatMessageW(
    dwFlags: DWORD,
    lpSource: LPCVOID,
    dwMessageId: DWORD,
    dwLanguageId: DWORD,
    lpBuffer: LPWSTR,
    nSize: DWORD,
    args: *mut __ms_va_list,
) -> DWORD {
    let mut format_args: format_args = core::mem::zeroed();
    let mut ret: DWORD = 0;
    let mut target: LPWSTR = core::ptr::null_mut();
    let mut talloced: DWORD = 0;
    let mut from: LPWSTR = core::ptr::null_mut();

    TRACE!("(0x%x,%p,%d,0x%x,%p,%d,%p)\n", dwFlags, lpSource, dwMessageId, dwLanguageId, lpBuffer, nSize, args);

    if lpBuffer.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if dwFlags & FORMAT_MESSAGE_ALLOCATE_BUFFER != 0 {
        *(lpBuffer as *mut LPWSTR) = core::ptr::null_mut();
    }

    if dwFlags & FORMAT_MESSAGE_ARGUMENT_ARRAY != 0 {
        format_args.args = args as *mut ULONG_PTR;
        format_args.list = core::ptr::null_mut();
        format_args.last = 0;
    } else {
        format_args.args = core::ptr::null_mut();
        format_args.list = args;
        format_args.last = 0;
    }

    from = core::ptr::null_mut();
    if dwFlags & FORMAT_MESSAGE_FROM_STRING != 0 {
        from = HeapAlloc(
            GetProcessHeap(),
            0,
            (strlenW(lpSource as LPCWSTR) as usize + 1) * core::mem::size_of::<WCHAR>(),
        ) as LPWSTR;
        strcpyW(from, lpSource as LPCWSTR);
    } else if dwFlags & (FORMAT_MESSAGE_FROM_HMODULE | FORMAT_MESSAGE_FROM_SYSTEM) != 0 {
        from = search_message(dwFlags, lpSource as HMODULE, dwMessageId as UINT, dwLanguageId as WORD);
        if from.is_null() {
            return 0;
        }
    } else {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    target = format_message(1, dwFlags, from as LPCWSTR, &raw mut format_args);
    if !target.is_null() {
        talloced = (strlenW(target) + 1) as DWORD;
        TRACE!("-- %s\n", debugstr_w(target));

        if dwFlags & FORMAT_MESSAGE_ALLOCATE_BUFFER != 0 {
            if *target != 0 {
                *((lpBuffer as *mut LPVOID)) = LocalAlloc(
                    LMEM_ZEROINIT,
                    (if nSize > talloced { nSize } else { talloced }) as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T,
                ) as LPVOID;
                strcpyW(*(lpBuffer as *mut LPWSTR), target);
            }
            // C falls through to `ret = talloced - 1` after this branch (only the
            // insufficient-buffer `goto failure` skips it). DeepSeek dropped this
            // assignment on the ALLOCATE_BUFFER path → FormatMessageW returned 0.
            ret = talloced.wrapping_sub(1);
        } else {
            if nSize < talloced {
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
            } else {
                strcpyW(lpBuffer, target);
                ret = talloced.wrapping_sub(1);
            }
        }
    }

    HeapFree(GetProcessHeap(), 0, target as LPVOID);
    HeapFree(GetProcessHeap(), 0, from as LPVOID);
    if dwFlags & FORMAT_MESSAGE_ARGUMENT_ARRAY == 0 {
        HeapFree(GetProcessHeap(), 0, format_args.args as LPVOID);
    }
    TRACE!("-- returning %u\n", ret);
    ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * FormatMessage implementation
//  *
//  * Copyright 1996 Marcus Meissner
//  * Copyright 2009 Alexandre Julliard
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
// #ifdef __REACTOS__
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
// DEBUG_CHANNEL(resource);
//
// extern HMODULE kernel32_handle;
//
// #else /* __REACTOS__ */
//
// #include "config.h"
//
// #include <stdarg.h>
// #include <stdio.h>
// #include <string.h>
//
// #include "ntstatus.h"
// #define WIN32_NO_STATUS
// #include "windef.h"
// #include "winbase.h"
// #include "winerror.h"
// #include "winternl.h"
// #include "winuser.h"
// #include "winnls.h"
// #include "wine/unicode.h"
// #include "kernel_private.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(resource);
//
// #endif /* __REACTOS__ */
//
// struct format_args
// {
//     ULONG_PTR    *args;
//     __ms_va_list *list;
//     int           last;
// };
//
// /* Messages used by FormatMessage
//  *
//  * They can be specified either directly or using a message ID and
//  * loading them from the resource.
//  *
//  * The resourcedata has following format:
//  * start:
//  * 0: DWORD nrofentries
//  * nrofentries * subentry:
//  *	0: DWORD firstentry
//  *	4: DWORD lastentry
//  *      8: DWORD offset from start to the stringentries
//  *
//  * (lastentry-firstentry) * stringentry:
//  * 0: WORD len (0 marks end)	[ includes the 4 byte header length ]
//  * 2: WORD flags
//  * 4: CHAR[len-4]
//  * 	(stringentry i of a subentry refers to the ID 'firstentry+i')
//  *
//  * Yes, ANSI strings in win32 resources. Go figure.
//  */
//
// /**********************************************************************
//  *	load_message    (internal)
//  */
// static LPWSTR load_message( HMODULE module, UINT id, WORD lang )
// {
// #ifdef __REACTOS__
//     MESSAGE_RESOURCE_ENTRY *mre;
// #else
//     const MESSAGE_RESOURCE_ENTRY *mre;
// #endif
//     WCHAR *buffer;
//     NTSTATUS status;
//
//     TRACE("module = %p, id = %08x\n", module, id );
//
//     if (!module) module = GetModuleHandleW( NULL );
// #ifdef __REACTOS__
//     status = RtlFindMessage(module, (ULONG_PTR)RT_MESSAGETABLE, lang, id, &mre);
//     if (!NT_SUCCESS(status))
// #else
//     if ((status = RtlFindMessage( module, RT_MESSAGETABLE, lang, id, &mre )) != STATUS_SUCCESS)
// #endif
//     {
//         SetLastError( RtlNtStatusToDosError(status) );
//         return NULL;
//     }
//
//     if (mre->Flags & MESSAGE_RESOURCE_UNICODE)
//     {
//         int len = (strlenW( (const WCHAR *)mre->Text ) + 1) * sizeof(WCHAR);
//         if (!(buffer = HeapAlloc( GetProcessHeap(), 0, len ))) return NULL;
//         memcpy( buffer, mre->Text, len );
//     }
//     else
//     {
//         int len = MultiByteToWideChar( CP_ACP, 0, (const char *)mre->Text, -1, NULL, 0 );
//         if (!(buffer = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) ))) return NULL;
//         MultiByteToWideChar( CP_ACP, 0, (const char*)mre->Text, -1, buffer, len );
//     }
//     TRACE("returning %s\n", wine_dbgstr_w(buffer));
//     return buffer;
// }
//
// static LPWSTR search_message( DWORD flags, HMODULE module, UINT id, WORD lang )
// {
//     LPWSTR from = NULL;
//     if (flags & FORMAT_MESSAGE_FROM_HMODULE)
//         from = load_message( module, id, lang );
//     if (!from && (flags & FORMAT_MESSAGE_FROM_SYSTEM))
//     {
//         /* Fold win32 hresult to its embedded error code. */
//         if (HRESULT_SEVERITY(id) == SEVERITY_ERROR &&
//             HRESULT_FACILITY(id) == FACILITY_WIN32)
//         {
//             id = HRESULT_CODE(id);
//         }
//         from = load_message( kernel32_handle, id, lang );
//     }
//     return from;
// }
//
// /**********************************************************************
//  *	get_arg    (internal)
//  */
// static ULONG_PTR get_arg( int nr, DWORD flags, struct format_args *args )
// {
//     if (nr == -1) nr = args->last + 1;
//     if (args->list)
//     {
//         if (!args->args) args->args = HeapAlloc( GetProcessHeap(), 0, 99 * sizeof(ULONG_PTR) );
//         while (nr > args->last)
//             args->args[args->last++] = va_arg( *args->list, ULONG_PTR );
//     }
//     if (nr > args->last) args->last = nr;
//     return args->args[nr - 1];
// }
//
// /**********************************************************************
//  *	format_insert    (internal)
//  */
// static LPCWSTR format_insert( BOOL unicode_caller, int insert, LPCWSTR format,
//                               DWORD flags, struct format_args *args,
//                               LPWSTR *result )
// {
//     static const WCHAR fmt_u[] = {'%','u',0};
//     WCHAR *wstring = NULL, *p, fmt[256];
//     ULONG_PTR arg;
//     int size;
//
//     if (*format != '!')  /* simple string */
//     {
//         arg = get_arg( insert, flags, args );
//         if (unicode_caller || !arg)
//         {
//             static const WCHAR nullW[] = {'(','n','u','l','l',')',0};
//             const WCHAR *str = (const WCHAR *)arg;
//
//             if (!str) str = nullW;
//             *result = HeapAlloc( GetProcessHeap(), 0, (strlenW(str) + 1) * sizeof(WCHAR) );
//             strcpyW( *result, str );
//         }
//         else
//         {
//             const char *str = (const char *)arg;
//             DWORD length = MultiByteToWideChar( CP_ACP, 0, str, -1, NULL, 0 );
//             *result = HeapAlloc( GetProcessHeap(), 0, length * sizeof(WCHAR) );
//             MultiByteToWideChar( CP_ACP, 0, str, -1, *result, length );
//         }
//         return format;
//     }
//
//     format++;
//     p = fmt;
//     *p++ = '%';
//
//     while (*format == '0' ||
//            *format == '+' ||
//            *format == '-' ||
//            *format == ' ' ||
//            *format == '*' ||
//            *format == '#')
//     {
//         if (*format == '*')
//         {
//             p += sprintfW( p, fmt_u, get_arg( insert, flags, args ));
//             insert = -1;
//             format++;
//         }
//         else *p++ = *format++;
//     }
//     while (isdigitW(*format)) *p++ = *format++;
//
//     if (*format == '.')
//     {
//         *p++ = *format++;
//         if (*format == '*')
//         {
//             p += sprintfW( p, fmt_u, get_arg( insert, flags, args ));
//             insert = -1;
//             format++;
//         }
//         else
//             while (isdigitW(*format)) *p++ = *format++;
//     }
//
//     /* replicate MS bug: drop an argument when using va_list with width/precision */
//     if (insert == -1 && args->list) args->last--;
//     arg = get_arg( insert, flags, args );
//
//     /* check for ascii string format */
//     if ((format[0] == 'h' && format[1] == 's') ||
//         (format[0] == 'h' && format[1] == 'S') ||
//         (unicode_caller && format[0] == 'S') ||
//         (!unicode_caller && format[0] == 's'))
//     {
//         DWORD len = MultiByteToWideChar( CP_ACP, 0, (char *)arg, -1, NULL, 0 );
//         wstring = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) );
//         MultiByteToWideChar( CP_ACP, 0, (char *)arg, -1, wstring, len );
//         arg = (ULONG_PTR)wstring;
//         *p++ = 's';
//     }
//     /* check for ascii character format */
//     else if ((format[0] == 'h' && format[1] == 'c') ||
//              (format[0] == 'h' && format[1] == 'C') ||
//              (unicode_caller && format[0] == 'C') ||
//              (!unicode_caller && format[0] == 'c'))
//     {
//         char ch = arg;
//         wstring = HeapAlloc( GetProcessHeap(), 0, 2 * sizeof(WCHAR) );
//         MultiByteToWideChar( CP_ACP, 0, &ch, 1, wstring, 1 );
//         wstring[1] = 0;
//         arg = (ULONG_PTR)wstring;
//         *p++ = 's';
//     }
//     /* check for wide string format */
//     else if ((format[0] == 'l' && format[1] == 's') ||
//              (format[0] == 'l' && format[1] == 'S') ||
//              (format[0] == 'w' && format[1] == 's') ||
//              (!unicode_caller && format[0] == 'S'))
//     {
//         *p++ = 's';
//     }
//     /* check for wide character format */
//     else if ((format[0] == 'l' && format[1] == 'c') ||
//              (format[0] == 'l' && format[1] == 'C') ||
//              (format[0] == 'w' && format[1] == 'c') ||
//              (!unicode_caller && format[0] == 'C'))
//     {
//         *p++ = 'c';
//     }
//     /* FIXME: handle I64 etc. */
//     else while (*format && *format != '!') *p++ = *format++;
//
//     *p = 0;
//     size = 256;
//     for (;;)
//     {
//         WCHAR *ret = HeapAlloc( GetProcessHeap(), 0, size * sizeof(WCHAR) );
//         int needed = snprintfW( ret, size, fmt, arg );
//         if (needed == -1 || needed >= size)
//         {
//             HeapFree( GetProcessHeap(), 0, ret );
//             size = max( needed + 1, size * 2 );
//         }
//         else
//         {
//             *result = ret;
//             break;
//         }
//     }
//
//     while (*format && *format != '!') format++;
//     if (*format == '!') format++;
//
//     HeapFree( GetProcessHeap(), 0, wstring );
//     return format;
// }
//
// struct _format_message_data
// {
//     LPWSTR formatted;
//     DWORD size;
//     LPWSTR t;
//     LPWSTR space;
//     BOOL inspace;
//     DWORD width, w;
// };
//
// static void format_add_char(struct _format_message_data *fmd, WCHAR c)
// {
//     *fmd->t++ = c;
//     if (fmd->width && fmd->width != FORMAT_MESSAGE_MAX_WIDTH_MASK)
//     {
//         switch (c) {
//         case '\r':
//         case '\n':
//             fmd->space = NULL;
//             fmd->inspace = FALSE;
//             fmd->w = 0;
//             break;
//         case ' ':
//             if (!fmd->inspace)
//                 fmd->space = fmd->t - 1;
//             fmd->inspace = TRUE;
//             fmd->w++;
//             break;
//         default:
//             fmd->inspace = FALSE;
//             fmd->w++;
//         }
//         if (fmd->w == fmd->width) {
//             LPWSTR notspace;
//             if (fmd->space) {
//                 notspace = fmd->space;
//                 while (notspace != fmd->t && *notspace == ' ')
//                     notspace++;
//             } else
//                 notspace = fmd->space = fmd->t;
//             fmd->w = fmd->t - notspace;
//             memmove(fmd->space+2, notspace, fmd->w * sizeof(*fmd->t));
//             *fmd->space++ = '\r';
//             *fmd->space++ = '\n';
//             fmd->t = fmd->space + fmd->w;
//             fmd->space = NULL;
//             fmd->inspace = FALSE;
//         }
//     }
//     if ((DWORD)(fmd->t - fmd->formatted) == fmd->size) {
//         DWORD_PTR ispace = fmd->space - fmd->formatted;
//         /* Allocate two extra characters so we can insert a '\r\n' in
//          * the middle of a word.
//          */
//         fmd->formatted = HeapReAlloc(GetProcessHeap(), 0, fmd->formatted, (fmd->size * 2 + 2) * sizeof(WCHAR));
//         fmd->t = fmd->formatted + fmd->size;
//         if (fmd->space)
//             fmd->space = fmd->formatted + ispace;
//         fmd->size *= 2;
//     }
// }
//
// /**********************************************************************
//  *	format_message    (internal)
//  */
// static LPWSTR format_message( BOOL unicode_caller, DWORD dwFlags, LPCWSTR fmtstr,
//                               struct format_args *format_args )
// {
//     struct _format_message_data fmd;
//     LPCWSTR f;
//     BOOL eos = FALSE;
//
//     fmd.size = 100;
//     fmd.formatted = fmd.t = HeapAlloc( GetProcessHeap(), 0, (fmd.size + 2) * sizeof(WCHAR) );
//
//     fmd.width = dwFlags & FORMAT_MESSAGE_MAX_WIDTH_MASK;
//     fmd.w = 0;
//     fmd.inspace = FALSE;
//     fmd.space = NULL;
//     f = fmtstr;
//     while (*f && !eos) {
//         if (*f=='%') {
//             int insertnr;
//             WCHAR *str,*x;
//
//             f++;
//             switch (*f) {
//             case '1':case '2':case '3':case '4':case '5':
//             case '6':case '7':case '8':case '9':
//                 if (dwFlags & FORMAT_MESSAGE_IGNORE_INSERTS)
//                     goto ignore_inserts;
//                 else if (((dwFlags & FORMAT_MESSAGE_ARGUMENT_ARRAY) && !format_args->args) ||
//                         (!(dwFlags & FORMAT_MESSAGE_ARGUMENT_ARRAY) && !format_args->list))
//                 {
//                     SetLastError(ERROR_INVALID_PARAMETER);
//                     HeapFree(GetProcessHeap(), 0, fmd.formatted);
//                     return NULL;
//                 }
//                 insertnr = *f-'0';
//                 switch (f[1]) {
//                 case '0':case '1':case '2':case '3':
//                 case '4':case '5':case '6':case '7':
//                 case '8':case '9':
//                     f++;
//                     insertnr = insertnr*10 + *f-'0';
//                     f++;
//                     break;
//                 default:
//                     f++;
//                     break;
//                 }
//                 f = format_insert( unicode_caller, insertnr, f, dwFlags, format_args, &str );
//                 for (x = str; *x; x++) format_add_char(&fmd, *x);
//                 HeapFree( GetProcessHeap(), 0, str );
//                 break;
//             case 'n':
//                 format_add_char(&fmd, '\r');
//                 format_add_char(&fmd, '\n');
//                 f++;
//                 break;
//             case 'r':
//                 format_add_char(&fmd, '\r');
//                 f++;
//                 break;
//             case 't':
//                 format_add_char(&fmd, '\t');
//                 f++;
//                 break;
//             case '0':
//                 eos = TRUE;
//                 f++;
//                 break;
//             case '\0':
//                 SetLastError(ERROR_INVALID_PARAMETER);
//                 HeapFree(GetProcessHeap(), 0, fmd.formatted);
//                 return NULL;
//             ignore_inserts:
//             default:
//                 if (dwFlags & FORMAT_MESSAGE_IGNORE_INSERTS)
//                     format_add_char(&fmd, '%');
//                 format_add_char(&fmd, *f++);
//                 break;
//             }
//         } else {
//             WCHAR ch = *f;
//             f++;
//             if (ch == '\r') {
//                 if (*f == '\n')
//                     f++;
//                 if(fmd.width)
//                     format_add_char(&fmd, ' ');
//                 else
//                 {
//                     format_add_char(&fmd, '\r');
//                     format_add_char(&fmd, '\n');
//                 }
//             } else {
//                 if (ch == '\n')
//                 {
//                     if(fmd.width)
//                         format_add_char(&fmd, ' ');
//                     else
//                     {
//                         format_add_char(&fmd, '\r');
//                         format_add_char(&fmd, '\n');
//                     }
//                 }
//                 else
//                     format_add_char(&fmd, ch);
//             }
//         }
//     }
//     *fmd.t = '\0';
//
//     return fmd.formatted;
// }
//
// /***********************************************************************
//  *           FormatMessageA   (KERNEL32.@)
//  */
// DWORD WINAPI FormatMessageA(
// 	DWORD	dwFlags,
// 	LPCVOID	lpSource,
// 	DWORD	dwMessageId,
// 	DWORD	dwLanguageId,
// 	LPSTR	lpBuffer,
// 	DWORD	nSize,
// 	__ms_va_list* args )
// {
//     struct format_args format_args;
//     DWORD ret = 0;
//     LPWSTR	target;
//     DWORD	destlength;
//     LPWSTR	from;
//
//     TRACE("(0x%x,%p,%d,0x%x,%p,%d,%p)\n",
//           dwFlags,lpSource,dwMessageId,dwLanguageId,lpBuffer,nSize,args);
//
//     if (dwFlags & FORMAT_MESSAGE_ALLOCATE_BUFFER)
//     {
//         if (!lpBuffer)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             return 0;
//         }
//         else
//             *(LPSTR *)lpBuffer = NULL;
//     }
//
//     if (dwFlags & FORMAT_MESSAGE_ARGUMENT_ARRAY)
//     {
//         format_args.args = (ULONG_PTR *)args;
//         format_args.list = NULL;
//         format_args.last = 0;
//     }
//     else
//     {
//         format_args.args = NULL;
//         format_args.list = args;
//         format_args.last = 0;
//     }
//
//     from = NULL;
//     if (dwFlags & FORMAT_MESSAGE_FROM_STRING)
//     {
//         DWORD length = MultiByteToWideChar(CP_ACP, 0, lpSource, -1, NULL, 0);
//         from = HeapAlloc( GetProcessHeap(), 0, length * sizeof(WCHAR) );
//         MultiByteToWideChar(CP_ACP, 0, lpSource, -1, from, length);
//     }
//     else if (dwFlags & (FORMAT_MESSAGE_FROM_HMODULE | FORMAT_MESSAGE_FROM_SYSTEM))
//     {
//         from = search_message( dwFlags, (HMODULE)lpSource, dwMessageId, dwLanguageId );
//         if (!from) return 0;
//     }
//     else
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     target = format_message( FALSE, dwFlags, from, &format_args );
//     if (!target)
//         goto failure;
//
//     TRACE("-- %s\n", debugstr_w(target));
//
//     /* Only try writing to an output buffer if there are processed characters
//      * in the temporary output buffer. */
//     if (*target)
//     {
//         destlength = WideCharToMultiByte(CP_ACP, 0, target, -1, NULL, 0, NULL, NULL);
//         if (dwFlags & FORMAT_MESSAGE_ALLOCATE_BUFFER)
//         {
//             LPSTR buf = LocalAlloc(LMEM_ZEROINIT, max(nSize, destlength));
//             WideCharToMultiByte(CP_ACP, 0, target, -1, buf, destlength, NULL, NULL);
//             *((LPSTR*)lpBuffer) = buf;
//         }
//         else
//         {
//             if (nSize < destlength)
//             {
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 goto failure;
//             }
//             WideCharToMultiByte(CP_ACP, 0, target, -1, lpBuffer, destlength, NULL, NULL);
//         }
//         ret = destlength - 1; /* null terminator */
//     }
//
// failure:
//     HeapFree(GetProcessHeap(),0,target);
//     HeapFree(GetProcessHeap(),0,from);
//     if (!(dwFlags & FORMAT_MESSAGE_ARGUMENT_ARRAY)) HeapFree( GetProcessHeap(), 0, format_args.args );
//     TRACE("-- returning %u\n", ret);
//     return ret;
// }
//
// /***********************************************************************
//  *           FormatMessageW   (KERNEL32.@)
//  */
// DWORD WINAPI FormatMessageW(
// 	DWORD	dwFlags,
// 	LPCVOID	lpSource,
// 	DWORD	dwMessageId,
// 	DWORD	dwLanguageId,
// 	LPWSTR	lpBuffer,
// 	DWORD	nSize,
// 	__ms_va_list* args )
// {
//     struct format_args format_args;
//     DWORD ret = 0;
//     LPWSTR target;
//     DWORD talloced;
//     LPWSTR from;
//
//     TRACE("(0x%x,%p,%d,0x%x,%p,%d,%p)\n",
//           dwFlags,lpSource,dwMessageId,dwLanguageId,lpBuffer,nSize,args);
//
//     if (!lpBuffer)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     if (dwFlags & FORMAT_MESSAGE_ALLOCATE_BUFFER)
//         *(LPWSTR *)lpBuffer = NULL;
//
//     if (dwFlags & FORMAT_MESSAGE_ARGUMENT_ARRAY)
//     {
//         format_args.args = (ULONG_PTR *)args;
//         format_args.list = NULL;
//         format_args.last = 0;
//     }
//     else
//     {
//         format_args.args = NULL;
//         format_args.list = args;
//         format_args.last = 0;
//     }
//
//     from = NULL;
//     if (dwFlags & FORMAT_MESSAGE_FROM_STRING) {
//         from = HeapAlloc( GetProcessHeap(), 0, (strlenW(lpSource) + 1) *
//             sizeof(WCHAR) );
//         strcpyW( from, lpSource );
//     }
//     else if (dwFlags & (FORMAT_MESSAGE_FROM_HMODULE | FORMAT_MESSAGE_FROM_SYSTEM))
//     {
//         from = search_message( dwFlags, (HMODULE)lpSource, dwMessageId, dwLanguageId );
//         if (!from) return 0;
//     }
//     else
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     target = format_message( TRUE, dwFlags, from, &format_args );
//     if (!target)
//         goto failure;
//
//     talloced = strlenW(target)+1;
//     TRACE("-- %s\n",debugstr_w(target));
//
//     /* Only allocate a buffer if there are processed characters in the
//      * temporary output buffer. If a caller supplies the buffer, then
//      * a null terminator will be written to it. */
//     if (dwFlags & FORMAT_MESSAGE_ALLOCATE_BUFFER)
//     {
//         if (*target)
//         {
//             /* nSize is the MINIMUM size */
//             *((LPVOID*)lpBuffer) = LocalAlloc(LMEM_ZEROINIT, max(nSize, talloced)*sizeof(WCHAR));
//             strcpyW(*(LPWSTR*)lpBuffer, target);
//         }
//     }
//     else
//     {
//         if (nSize < talloced)
//         {
//             SetLastError(ERROR_INSUFFICIENT_BUFFER);
//             goto failure;
//         }
//         strcpyW(lpBuffer, target);
//     }
//
//     ret = talloced - 1; /* null terminator */
// failure:
//     HeapFree(GetProcessHeap(),0,target);
//     HeapFree(GetProcessHeap(),0,from);
//     if (!(dwFlags & FORMAT_MESSAGE_ARGUMENT_ARRAY)) HeapFree( GetProcessHeap(), 0, format_args.args );
//     TRACE("-- returning %u\n", ret);
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
