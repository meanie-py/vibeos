// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// undname.c's two TU-local `#define`s. `und_alloc` bump-allocates out of 1 KiB blocks
// chained through the first `sizeof(void*)` bytes of each, which is why the usable size is
// the block minus a pointer -- 1020 on i386 and 1016 on amd64/arm64. Spelled with
// `size_of` for that reason: a transcribed 1020 over-reads by four bytes on every 64-bit
// block, at the far end, only for the longest symbols.
const BLOCK_SIZE: core::ffi::c_uint = 1024;
const AVAIL_SIZE: core::ffi::c_uint =
    1024 - core::mem::size_of::<*mut core::ffi::c_void>() as core::ffi::c_uint;

// wine/debug.h's channels. No-ops here for the same reason every other translated msvcrt
// module makes them no-ops -- and `macro_rules!`, never an out-of-line `fn`: an
// out-of-line no-op is an optimization barrier that uncloaks latent ABI mismatches.

// `atoi` and `isdigit` are ordinary CRT siblings undname.c calls, and after the split out of
// msvcrt-crt-rust they are no longer defined by THIS crate, so they are `extern` again.
//
// That inversion is the point of the split, not an accident of it. This crate is linked into
// two different CRTs: in msvcrt.dll the names bind to msvcrt-crt-rust's own Rust bodies
// (string.c's and ctype.c's families), and in ucrtbase.dll they bind to the genuine Microsoft
// UCRT's. undname must call ITS HOST's atoi, and a `use` would have hard-wired msvcrt's into
// both. The signatures below are transcribed from stdlib.h and ctype.h, per the prelude's
// rule -- a declaration that disagrees with the host's is not a build error, it is a silent
// ABI mismatch.
unsafe extern "C" {
    /// `stdlib.h`: `int __cdecl atoi(const char *_Str);`
    fn atoi(_Str: *const c_char) -> c_int;
    /// `ctype.h`: `int __cdecl isdigit(int _C);`
    fn isdigit(_C: c_int) -> c_int;
}

// ── str_printf: the one function in this file that is hand-written, and why ──────────
//
// The C is `static char* WINAPIV str_printf(struct parsed_symbol*, const char* format, ...)`
// — a C VARIADIC, and DEFINING one needs `#![feature(c_variadic)]`, which this repo is
// pinned off. file.c and process.c solved their variadics with `va_start` trampolines in a
// C shim, but that answer does not reach here: `str_printf` is `static`, so a shim could
// not name it, and it calls the equally-static `und_alloc` to allocate out of the symbol's
// own block list. Keeping a C shim would also mean keeping undname.c.obj alive for one
// helper, which is the opposite of the point.
//
// So the variadic list becomes an explicit slice. That is sound HERE and would not be in
// general: `str_printf` is internal to this TU, has no ABI contract with anything, and
// implements exactly two conversions (`%s` and `%c`) whose argument types are fixed by the
// format literal at every one of its 50-odd call sites. The `str_printf!` macro keeps those
// call sites reading as they do in the C — `str_printf!(sym, fmt, a, b)` — so this file
// still diffs against undname.c line for line.
//
// The BODY below is a faithful port, including the two things about it that look like bugs
// and are not:
//   * the `default: i--; /* fall through */ case '%':` chain. An unknown conversion emits a
//     literal '%' and then re-reads the conversion character as an ordinary character, so
//     "%z" comes out as "%z". The `i -= 1` is what makes the re-read happen.
//   * a NULL `%s` argument contributes nothing — no "(null)", no crash. get_args and
//     get_qualified_type both rely on that to splice optional fragments.
// It also walks the format string TWICE, once to size and once to fill, exactly as the C
// does — the allocation comes from `und_alloc`, which never frees, so a second pass is
// cheaper than growing.

/// One entry of `str_printf`'s argument list. The C pushed these through `...`; the enum is
/// what makes the `%s`/`%c` split explicit instead of trusting the format string at runtime.
#[derive(Clone, Copy)]
pub enum PArg {
    S(*const core::ffi::c_char),
    C(core::ffi::c_int),
}

impl From<*const core::ffi::c_char> for PArg {
    fn from(p: *const core::ffi::c_char) -> Self {
        PArg::S(p)
    }
}
impl From<*mut core::ffi::c_char> for PArg {
    fn from(p: *mut core::ffi::c_char) -> Self {
        PArg::S(p)
    }
}
impl From<*const u8> for PArg {
    fn from(p: *const u8) -> Self {
        PArg::S(p as *const core::ffi::c_char)
    }
}
impl From<core::ffi::c_char> for PArg {
    // `va_arg(args, int)` — C's default argument promotion, spelled out.
    fn from(c: core::ffi::c_char) -> Self {
        PArg::C(c as core::ffi::c_int)
    }
}
impl From<core::ffi::c_int> for PArg {
    fn from(c: core::ffi::c_int) -> Self {
        PArg::C(c)
    }
}

impl PArg {
    /// `va_arg(args, char*)`. A `%c` argument read through `%s` is a translation bug, not a
    /// runtime condition — C would have reinterpreted the int as a pointer and read wild
    /// memory. A null yields the empty string instead, which is the same visible outcome as
    /// the NULL `%s` the callers legitimately pass.
    #[inline]
    fn as_ptr(self) -> *const core::ffi::c_char {
        match self {
            PArg::S(p) => p,
            PArg::C(_) => core::ptr::null(),
        }
    }
    /// `va_arg(args, int)`.
    #[inline]
    fn as_int(self) -> core::ffi::c_int {
        match self {
            PArg::C(c) => c,
            PArg::S(_) => 0,
        }
    }
}

macro_rules! str_printf {
    ($sym:expr, $fmt:expr $(, $a:expr)* $(,)?) => {
        str_printf_impl(
            $sym,
            $fmt as *const core::ffi::c_char,
            &[$(PArg::from($a)),*],
        )
    };
}

unsafe fn str_printf_impl(
    sym: *mut parsed_symbol,
    format: *const core::ffi::c_char,
    args: &[PArg],
) -> *mut core::ffi::c_char {
    let mut len: core::ffi::c_uint = 1;
    let mut i: usize;
    let mut ai: usize;

    i = 0;
    ai = 0;
    while *format.add(i) != 0 {
        if *format.add(i) == b'%' as core::ffi::c_char {
            i += 1;
            match *format.add(i) as u8 {
                b's' => {
                    let t = args[ai].as_ptr();
                    ai += 1;
                    if !t.is_null() {
                        len += strlen(t) as core::ffi::c_uint;
                    }
                }
                b'c' => {
                    let _ = args[ai].as_int();
                    ai += 1;
                    len += 1;
                }
                other => {
                    // `default: i--;` then FALL THROUGH into `case '%'`.
                    if other != b'%' {
                        i -= 1;
                    }
                    len += 1;
                }
            }
        } else {
            len += 1;
        }
        i += 1;
    }

    let tmp = und_alloc(sym, len) as *mut core::ffi::c_char;
    if tmp.is_null() {
        return core::ptr::null_mut();
    }

    let mut p = tmp;
    i = 0;
    ai = 0;
    while *format.add(i) != 0 {
        if *format.add(i) == b'%' as core::ffi::c_char {
            i += 1;
            match *format.add(i) as u8 {
                b's' => {
                    let t = args[ai].as_ptr();
                    ai += 1;
                    if !t.is_null() {
                        let sz = strlen(t);
                        memcpy(p as *mut c_void, t as *const c_void, sz);
                        p = p.add(sz);
                    }
                }
                b'c' => {
                    *p = args[ai].as_int() as core::ffi::c_char;
                    ai += 1;
                    p = p.add(1);
                }
                other => {
                    if other != b'%' {
                        i -= 1;
                    }
                    *p = b'%' as core::ffi::c_char;
                    p = p.add(1);
                }
            }
        } else {
            *p = *format.add(i);
            p = p.add(1);
        }
        i += 1;
    }
    *p = 0;
    tmp
}

// ---- file-scope preamble (translated) ----
pub const UNDNAME_NO_COMPLEX_TYPE: u32 = 0x8000;

// `struct array stack = sym->stack;` in the `'?'` case is a C STRUCT COPY — save the
// whole descriptor, run a nested demangle on a fresh stack, restore. `Copy` is what
// makes that assignment mean the same thing in Rust; without it the save/restore turns
// into a move out of `*sym`, which is not what the C does and will not compile through a
// raw pointer anyway. Plain POD: four counters and one owning-nothing pointer.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct array {
    pub start: u32,
    pub num: u32,
    pub max: u32,
    pub alloc: u32,
    pub elts: *mut *mut i8,
}

pub type malloc_func_t = Option<unsafe extern "C" fn(usize) -> *mut core::ffi::c_void>;
pub type free_func_t = Option<unsafe extern "C" fn(*mut core::ffi::c_void)>;

#[repr(C)]
pub struct parsed_symbol {
    pub flags: u32,
    pub mem_alloc_ptr: malloc_func_t,
    pub mem_free_ptr: free_func_t,
    pub current: *const i8,
    pub result: *mut i8,
    pub names: array,
    pub args: array,
    pub stack: array,
    pub alloc_list: *mut core::ffi::c_void,
    pub avail_in_first: u32,
}

pub const DT_NO_LEADING_WS: u32 = 0x01;
pub const DT_NO_LRSEP_WS: u32 = 0x02;

pub type datatype_e = u32;

#[repr(C)]
pub struct datatype_t {
    pub left: *const i8,
    pub right: *const i8,
    pub flags: datatype_e,
}

pub const IN_ARGS: u32 = 0x01;
pub const WS_AFTER_QUAL_IF: u32 = 0x02;

pub type datatype_flags = u32;

#[repr(C)]
pub struct function_signature {
    pub call_conv: *const i8,
    pub exported: *const i8,
    pub return_ct: datatype_t,
    pub arguments: *const i8,
}
unsafe extern "C" fn und_alloc(sym: *mut parsed_symbol, len: u32) -> *mut core::ffi::c_void {
    let mut ptr: *mut core::ffi::c_void;

    if len > AVAIL_SIZE {
        ptr = ((*sym).mem_alloc_ptr.unwrap_unchecked())(((core::mem::size_of::<*mut core::ffi::c_void>() as u32).wrapping_add(len)) as usize);
        if ptr.is_null() {
            return core::ptr::null_mut();
        }
        *(ptr as *mut *mut core::ffi::c_void) = (*sym).alloc_list;
        (*sym).alloc_list = ptr;
        (*sym).avail_in_first = 0;
        ptr = ((*sym).alloc_list as *mut u8).offset(core::mem::size_of::<*mut core::ffi::c_void>() as isize) as *mut core::ffi::c_void;
    } else {
        if len > (*sym).avail_in_first {
            ptr = ((*sym).mem_alloc_ptr.unwrap_unchecked())((BLOCK_SIZE) as usize);
            if ptr.is_null() {
                return core::ptr::null_mut();
            }
            *(ptr as *mut *mut core::ffi::c_void) = (*sym).alloc_list;
            (*sym).alloc_list = ptr;
            (*sym).avail_in_first = AVAIL_SIZE;
        }
        ptr = ((*sym).alloc_list as *mut u8).offset((BLOCK_SIZE).wrapping_sub((*sym).avail_in_first) as isize) as *mut core::ffi::c_void;
        (*sym).avail_in_first = (*sym).avail_in_first.wrapping_sub(len);
    }
    ptr
}
unsafe extern "C" fn und_free_all(sym: *mut parsed_symbol) {
    let mut next: *mut core::ffi::c_void;

    while !(*sym).alloc_list.is_null() {
        next = *((*sym).alloc_list as *mut *mut core::ffi::c_void);
        if (*sym).mem_free_ptr.is_some() {
            (*sym).mem_free_ptr.unwrap_unchecked()((*sym).alloc_list);
        }
        (*sym).alloc_list = next;
    }
    (*sym).avail_in_first = 0;
}
unsafe extern "C" fn str_array_init(a: *mut array) {
    (*a).start = 0;
    (*a).num = 0;
    (*a).max = 0;
    (*a).alloc = 0;
    (*a).elts = core::ptr::null_mut();
}
unsafe extern "C" fn str_array_push(
    sym: *mut parsed_symbol,
    ptr: *const core::ffi::c_char,
    len: core::ffi::c_int,
    a: *mut array,
) -> u8 {
    let mut new: *mut *mut core::ffi::c_char;

    assert!(!ptr.is_null());
    assert!(!a.is_null());

    if (*a).alloc == 0 {
        (*a).alloc = 32;
        new = und_alloc(sym, (*a).alloc * core::mem::size_of_val(&(*(*a).elts.add(0))) as u32) as *mut *mut core::ffi::c_char;
        if new.is_null() {
            return FALSE;
        }
        (*a).elts = new;
    } else if (*a).max >= (*a).alloc {
        new = und_alloc(sym, ((*a).alloc * 2) * core::mem::size_of_val(&(*(*a).elts.add(0))) as u32) as *mut *mut core::ffi::c_char;
        if new.is_null() {
            return FALSE;
        }
        memcpy(new as *mut core::ffi::c_void, (*a).elts as *const core::ffi::c_void, ((*a).alloc * (core::mem::size_of_val(&(*(*a).elts.add(0))) as usize) as u32) as usize);
        (*a).alloc *= 2;
        (*a).elts = new;
    }
    let mut len = len;
    if len == -1 {
        len = strlen(ptr) as core::ffi::c_int;
    }
    *(*a).elts.add((*a).num as usize) = und_alloc(sym, (len + 1) as u32) as *mut core::ffi::c_char;
    assert!(!(*(*a).elts.add((*a).num as usize)).is_null());
    memcpy(*(*a).elts.add((*a).num as usize) as *mut core::ffi::c_void, ptr as *const core::ffi::c_void, len as usize);
    *(*(*a).elts.add((*a).num as usize)).add(len as usize) = b'\0' as core::ffi::c_char;
    (*a).num += 1;
    if (*a).num >= (*a).max {
        (*a).max = (*a).num;
    }
    // The C's `if (TRACE_ON(msvcrt))` dump loop fed only a TRACE, and the prints are dropped.

    (TRUE) as u8
}
unsafe extern "C" fn str_array_get_ref(cref: *mut array, idx: u32) -> *mut i8 {
    assert!(!cref.is_null());
    if (*cref).start + idx >= (*cref).max {
        return core::ptr::null_mut();
    }
    *(*cref).elts.offset(((*cref).start + idx) as isize)
}
unsafe extern "C" fn get_number(sym: *mut parsed_symbol) -> *const i8 {
    let mut ptr: *mut i8;
    let mut sgn: u8 = FALSE;

    if *(*sym).current == b'?' as i8 {
        sgn = (TRUE) as u8;
        (*sym).current = (*sym).current.offset(1);
    }
    if *(*sym).current >= b'0' as i8 && *(*sym).current <= b'8' as i8 {
        ptr = und_alloc(sym, 3) as *mut i8;
        if sgn != 0 {
            *ptr.offset(0 as isize) = b'-' as i8;
        }
        *ptr.offset(if sgn != 0 { 1 } else { 0 } as isize) =
            *(*sym).current + 1;
        *ptr.offset(if sgn != 0 { 2 } else { 1 } as isize) = b'\0' as i8;
        (*sym).current = (*sym).current.offset(1);
    } else if *(*sym).current == b'9' as i8 {
        ptr = und_alloc(sym, 4) as *mut i8;
        if sgn != 0 {
            *ptr.offset(0 as isize) = b'-' as i8;
        }
        *ptr.offset(if sgn != 0 { 1 } else { 0 } as isize) = b'1' as i8;
        *ptr.offset(if sgn != 0 { 2 } else { 1 } as isize) = b'0' as i8;
        *ptr.offset(if sgn != 0 { 3 } else { 2 } as isize) = b'\0' as i8;
        (*sym).current = (*sym).current.offset(1);
    } else if *(*sym).current >= b'A' as i8 && *(*sym).current <= b'P' as i8 {
        let mut ret: i32 = 0;

        while *(*sym).current >= b'A' as i8 && *(*sym).current <= b'P' as i8 {
            ret *= 16;
            ret += (*(*sym).current - b'A' as i8) as i32;
            (*sym).current = (*sym).current.offset(1);
        }
        if *(*sym).current != b'@' as i8 {
            return core::ptr::null();
        }

        ptr = und_alloc(sym, 17) as *mut i8;
        sprintf(ptr, b"%s%u\0".as_ptr() as *const i8, if sgn != 0 { b"-\0".as_ptr() } else { b"\0".as_ptr() } as *const i8, ret as u32);
        (*sym).current = (*sym).current.offset(1);
    } else {
        return core::ptr::null();
    }
    ptr as *const i8
}
unsafe extern "C" fn get_args(
    sym: *mut parsed_symbol,
    z_term: u8,
    open_char: core::ffi::c_char,
    close_char: core::ffi::c_char,
) -> *mut core::ffi::c_char {
    let mut ct: datatype_t = core::mem::zeroed();
    let mut arg_collect: array = core::mem::zeroed();
    let mut args_str: *mut core::ffi::c_char = core::ptr::null_mut();
    let mut last: *mut core::ffi::c_char;
    let mut i: core::ffi::c_uint;
    let mut p: *const core::ffi::c_char;

    if z_term != 0
        && *(*sym).current.offset(0) == b'X' as core::ffi::c_char
        && *(*sym).current.offset(1) == b'Z' as core::ffi::c_char
    {
        (*sym).current = (*sym).current.offset(2);
        return str_printf!(sym, b"%cvoid%c\0".as_ptr() as *const core::ffi::c_char, open_char, close_char);
    }

    str_array_init(&mut arg_collect);

    while *(*sym).current != 0 {
        p = (*sym).current;

        if *(*sym).current == b'@' as core::ffi::c_char {
            (*sym).current = (*sym).current.offset(1);
            break;
        }
        if z_term != 0 && *(*sym).current.offset(0) == b'Z' as core::ffi::c_char {
            (*sym).current = (*sym).current.offset(1);
            if str_array_push(sym, b"...\0".as_ptr() as *const core::ffi::c_char, -1, &mut arg_collect) == 0 {
                return core::ptr::null_mut();
            }
            break;
        }
        if z_term == 0
            && *(*sym).current.offset(0) == b'$' as core::ffi::c_char
            && *(*sym).current.offset(1) == b'$' as core::ffi::c_char
            && *(*sym).current.offset(2) == b'V' as core::ffi::c_char
        {
            (*sym).current = (*sym).current.offset(3);
            continue;
        }
        if demangle_datatype(sym, &mut ct, IN_ARGS) == 0 {
            return core::ptr::null_mut();
        }
        if str_array_push(
            sym,
            str_printf!(sym, b"%s%s\0".as_ptr() as *const core::ffi::c_char, ct.left, ct.right),
            -1,
            &mut arg_collect,
        ) == 0
        {
            return core::ptr::null_mut();
        }
        if z_term != 0
            && (*sym).current.offset_from(p) as isize > 1
            && (*sym).args.num < 20
        {
            if str_array_push(
                sym,
                if !ct.left.is_null() { ct.left } else { b"\0".as_ptr() as *const core::ffi::c_char },
                -1,
                &mut (*sym).args,
            ) == 0
                || str_array_push(
                    sym,
                    if !ct.right.is_null() { ct.right } else { b"\0".as_ptr() as *const core::ffi::c_char },
                    -1,
                    &mut (*sym).args,
                ) == 0
            {
                return core::ptr::null_mut();
            }
        }
    }

    if z_term != 0 {
        let current_char = *(*sym).current;
        (*sym).current = (*sym).current.offset(1);
        if current_char != b'Z' as core::ffi::c_char {
            return core::ptr::null_mut();
        }
    }

    if arg_collect.num == 0 {
        return str_printf!(sym, b"%c%c\0".as_ptr() as *const core::ffi::c_char, open_char, close_char);
    }

    i = 1;
    while i < arg_collect.num {
        args_str = str_printf!(sym, b"%s,%s\0".as_ptr() as *const core::ffi::c_char, args_str, *arg_collect.elts.offset(i as isize));
        i = i.wrapping_add(1);
    }

    last = if !args_str.is_null() { args_str } else { *arg_collect.elts.offset(0) };
    if close_char == b'>' as core::ffi::c_char
        && *last.offset(strlen(last) as isize - 1) == b'>' as core::ffi::c_char
    {
        args_str = str_printf!(
            sym,
            b"%c%s%s %c\0".as_ptr() as *const core::ffi::c_char,
            open_char,
            *arg_collect.elts.offset(0),
            args_str,
            close_char,
        );
    } else {
        args_str = str_printf!(
            sym,
            b"%c%s%s%c\0".as_ptr() as *const core::ffi::c_char,
            open_char,
            *arg_collect.elts.offset(0),
            args_str,
            close_char,
        );
    }

    args_str
}
unsafe extern "C" fn append_extended_qualifier(
    sym: *mut parsed_symbol,
    where_: *mut *const i8,
    str: *const i8,
    is_ms_keyword: BOOL,
) {
    if is_ms_keyword == 0 || ((*sym).flags & UNDNAME_NO_MS_KEYWORDS) == 0 {
        let str_to_use = if is_ms_keyword != 0 && ((*sym).flags & UNDNAME_NO_LEADING_UNDERSCORES) != 0 {
            str.offset(2 as isize)
        } else {
            str
        };

        if !(*where_).is_null() {
            *where_ = str_printf!(
                sym,
                b"%s%s%s%s\0".as_ptr() as *const i8,
                *where_,
                if is_ms_keyword != 0 { b" \0".as_ptr() as *const i8 } else { b"\0".as_ptr() as *const i8 },
                str_to_use,
                if is_ms_keyword != 0 { b"\0".as_ptr() as *const i8 } else { b" \0".as_ptr() as *const i8 }
            );
        } else {
            *where_ = str_printf!(
                sym,
                b"%s%s\0".as_ptr() as *const i8,
                str_to_use,
                if is_ms_keyword != 0 { b"\0".as_ptr() as *const i8 } else { b" \0".as_ptr() as *const i8 }
            );
        }
    }
}
unsafe extern "C" fn get_extended_qualifier(sym: *mut parsed_symbol, xdt: *mut datatype_t) {
    let mut fl = 0;
    (*xdt).left = core::ptr::null();
    (*xdt).right = core::ptr::null();
    (*xdt).flags = 0;
    loop {
        match *(*sym).current as u8 {
            b'E' => {
                append_extended_qualifier(sym, core::ptr::addr_of_mut!((*xdt).right), b"__ptr64\0".as_ptr() as *const i8, (TRUE) as i32);
                fl |= 2;
            }
            b'F' => {
                append_extended_qualifier(sym, core::ptr::addr_of_mut!((*xdt).left), b"__unaligned\0".as_ptr() as *const i8, (TRUE) as i32);
                fl |= 2;
            }
            // `#ifdef _UCRT`: the ref-qualifiers of a member function (`void f() &`). The
            // translation pinned these OFF with `cfg(any())`, which was right for msvcrt.dll
            // and wrong for ucrtbase.dll, which links this same crate: Wine's ucrtbase cpp
            // test expects `Q::meth(void)const __ptr64& ` and got the mangled name back.
            // `ucrtsupport` is the crate's existing "this is the UCRT build" switch.
            #[cfg(feature = "ucrtsupport")]
            b'G' => {
                append_extended_qualifier(sym, core::ptr::addr_of_mut!((*xdt).right), b"&\0".as_ptr() as *const i8, (FALSE) as i32);
                fl |= 1;
            }
            #[cfg(feature = "ucrtsupport")] // `#ifdef _UCRT`, as above.
            b'H' => {
                append_extended_qualifier(sym, core::ptr::addr_of_mut!((*xdt).right), b"&&\0".as_ptr() as *const i8, (FALSE) as i32);
                fl |= 1;
            }
            b'I' => {
                append_extended_qualifier(sym, core::ptr::addr_of_mut!((*xdt).right), b"__restrict\0".as_ptr() as *const i8, (TRUE) as i32);
                fl |= 2;
            }
            _ => {
                if fl == 1 || (fl == 3 && ((*sym).flags & UNDNAME_NO_MS_KEYWORDS) != 0) {
                    (*xdt).flags = DT_NO_LRSEP_WS;
                }
                return;
            }
        }
        (*sym).current = (*sym).current.offset(1);
    }
}
unsafe extern "C" fn get_qualifier(sym: *mut parsed_symbol, xdt: *mut datatype_t, pclass: *mut *const i8) -> u8 {
    let ch: i8;
    let mut qualif: *const i8;

    get_extended_qualifier(sym, xdt);
    ch = *(*sym).current;
    (*sym).current = (*sym).current.offset(1);
    match ch as u8 {
        b'A' => qualif = core::ptr::null(),
        b'B' => qualif = b"const\0".as_ptr() as *const i8,
        b'C' => qualif = b"volatile\0".as_ptr() as *const i8,
        b'D' => qualif = b"const volatile\0".as_ptr() as *const i8,
        b'Q' => qualif = core::ptr::null(),
        b'R' => qualif = b"const\0".as_ptr() as *const i8,
        b'S' => qualif = b"volatile\0".as_ptr() as *const i8,
        b'T' => qualif = b"const volatile\0".as_ptr() as *const i8,
        _ => return FALSE,
    }
    if !qualif.is_null() {
        (*xdt).flags &= !DT_NO_LRSEP_WS;
        (*xdt).left = if !(*xdt).left.is_null() {
            str_printf!(sym, b"%s %s\0".as_ptr() as *const i8, qualif, (*xdt).left)
        } else {
            qualif
        };
    }
    if ch as u8 >= b'Q' && ch as u8 <= b'T' {
        let class = get_class_name(sym);
        if class.is_null() {
            return FALSE;
        }
        if pclass.is_null() {
            FIXME!("Got pointer to class %s member without storage\n", class);
            return FALSE;
        }
        *pclass = class;
    } else if !pclass.is_null() {
        *pclass = core::ptr::null();
    }
    (TRUE) as u8
}
unsafe extern "C" fn get_function_qualifier(sym: *mut parsed_symbol, qualif: *mut *const i8) -> u8 {
    let mut xdt: datatype_t = core::mem::zeroed();

    if get_qualifier(sym, &mut xdt, core::ptr::null_mut()) == FALSE {
        return FALSE;
    }
    *qualif = if !xdt.left.is_null() || !xdt.right.is_null() {
        str_printf!(sym, b"%s%s%s\0".as_ptr() as *const i8, xdt.left, if (xdt.flags as u32 & DT_NO_LRSEP_WS) != 0 { b"\0".as_ptr() } else { b" \0".as_ptr() } as *const i8, xdt.right)
    } else {
        core::ptr::null()
    };
    (TRUE) as u8
}
unsafe extern "C" fn get_qualified_type(ct: *mut datatype_t, sym: *mut parsed_symbol,
                             qualif: i8, flags: datatype_flags) -> u8 {
    let mut xdt1: datatype_t = core::mem::zeroed();
    let mut xdt2: datatype_t = core::mem::zeroed();
    let mut ref_: *const i8;
    let str_qualif: *const i8;
    let mut class: *const i8 = core::ptr::null();

    get_extended_qualifier(sym, &mut xdt1);

    match qualif as u8 {
        b'A' => { ref_ = b" &\0".as_ptr() as *const i8; str_qualif = core::ptr::null(); }
        b'B' => { ref_ = b" &\0".as_ptr() as *const i8; str_qualif = b" volatile\0".as_ptr() as *const i8; }
        b'P' => { ref_ = b" *\0".as_ptr() as *const i8; str_qualif = core::ptr::null(); }
        b'Q' => { ref_ = b" *\0".as_ptr() as *const i8; str_qualif = b" const\0".as_ptr() as *const i8; }
        b'R' => { ref_ = b" *\0".as_ptr() as *const i8; str_qualif = b" volatile\0".as_ptr() as *const i8; }
        b'S' => { ref_ = b" *\0".as_ptr() as *const i8; str_qualif = b" const volatile\0".as_ptr() as *const i8; }
        b'?' => { ref_ = core::ptr::null(); str_qualif = core::ptr::null(); }
        b'$' => { ref_ = b" &&\0".as_ptr() as *const i8; str_qualif = core::ptr::null(); }
        _ => return FALSE,
    }
    (*ct).right = core::ptr::null();
    (*ct).flags = 0;

    if *(*sym).current.offset(0) == b'$' as i8 && *(*sym).current.offset(1) == b'A' as i8 {
        (*sym).current = (*sym).current.offset(2);

        match qualif as u8 {
            b'A' | b'B' => {
                ref_ = b" %\0".as_ptr() as *const i8;
            }
            b'P' | b'Q' | b'R' | b'S' => {
                ref_ = b" ^\0".as_ptr() as *const i8;
            }
            _ => {
                return FALSE;
            }
        }
    }

    if get_qualifier(sym, &mut xdt2, &mut class) != 0 {
        let mark = (*sym).stack.num;
        let mut sub_ct: datatype_t = core::mem::zeroed();

        if !ref_.is_null() || !str_qualif.is_null() || !xdt1.left.is_null() || !xdt1.right.is_null() {
            if !class.is_null() {
                (*ct).left = str_printf!(sym, b"%s%s%s%s::%s%s%s\0".as_ptr() as *const i8,
                                      if !xdt1.left.is_null() { b" \0".as_ptr() as *const i8 } else { core::ptr::null() }, xdt1.left,
                                      if !class.is_null() { b" \0".as_ptr() as *const i8 } else { core::ptr::null() }, class, if !ref_.is_null() { ref_.offset(1) } else { core::ptr::null() },
                                      if !xdt1.right.is_null() { b" \0".as_ptr() as *const i8 } else { core::ptr::null() }, xdt1.right, str_qualif);
            } else {
                (*ct).left = str_printf!(sym, b"%s%s%s%s%s%s\0".as_ptr() as *const i8,
                                      if !xdt1.left.is_null() { b" \0".as_ptr() as *const i8 } else { core::ptr::null() }, xdt1.left, ref_,
                                      if !xdt1.right.is_null() { b" \0".as_ptr() as *const i8 } else { core::ptr::null() }, xdt1.right, str_qualif);
            }
        } else {
            (*ct).left = core::ptr::null();
        }

        if *(*sym).current == b'Y' as i8 {
            let n1: *const i8;
            let mut num: i32;

            (*sym).current = (*sym).current.offset(1);
            n1 = get_number(sym);
            if n1.is_null() {
                return FALSE;
            }
            num = atoi((n1).cast());

            (*ct).left = str_printf!(sym, b" (%s%s\0".as_ptr() as *const i8, xdt2.left, if !(*ct).left.is_null() && xdt2.left.is_null() { (*ct).left.offset(1) } else { (*ct).left });
            (*ct).right = b")\0".as_ptr() as *const i8;
            xdt2.left = core::ptr::null();

            while num > 0 {
                num -= 1;
                (*ct).right = str_printf!(sym, b"%s[%s]\0".as_ptr() as *const i8, (*ct).right, get_number(sym));
            }
        }

        if demangle_datatype(sym, &mut sub_ct, 0) == 0 {
            return FALSE;
        }
        if (sub_ct.flags & DT_NO_LEADING_WS) != 0 {
            (*ct).left = (*ct).left.offset(1);
        }
        (*ct).left = str_printf!(sym, b"%s%s%s%s%s\0".as_ptr() as *const i8, sub_ct.left, if !xdt2.left.is_null() { b" \0".as_ptr() as *const i8 } else { core::ptr::null() },
                              xdt2.left, (*ct).left,
                              if (!xdt2.left.is_null() || !str_qualif.is_null()) && (flags & WS_AFTER_QUAL_IF) != 0 { b" \0".as_ptr() as *const i8 } else { core::ptr::null() });
        if !sub_ct.right.is_null() {
            (*ct).right = str_printf!(sym, b"%s%s\0".as_ptr() as *const i8, (*ct).right, sub_ct.right);
        }
        (*sym).stack.num = mark;
    } else if !ref_.is_null() || !str_qualif.is_null() || !xdt1.left.is_null() || !xdt1.right.is_null() {
        (*ct).left = str_printf!(sym, b"%s%s%s%s%s%s\0".as_ptr() as *const i8,
                              if !xdt1.left.is_null() { b" \0".as_ptr() as *const i8 } else { core::ptr::null() }, xdt1.left, ref_,
                              if !xdt1.right.is_null() { b" \0".as_ptr() as *const i8 } else { core::ptr::null() }, xdt1.right, str_qualif);
    } else {
        (*ct).left = core::ptr::null();
    }
    return (TRUE) as u8;
}
unsafe extern "C" fn get_literal_string(sym: *mut parsed_symbol) -> *mut i8 {
    let ptr: *const i8 = (*sym).current;

    loop {
        if !((*(*sym).current >= b'A' as i8 && *(*sym).current <= b'Z' as i8) ||
             (*(*sym).current >= b'a' as i8 && *(*sym).current <= b'z' as i8) ||
             (*(*sym).current >= b'0' as i8 && *(*sym).current <= b'9' as i8) ||
             *(*sym).current == b'_' as i8 || *(*sym).current == b'$' as i8 ||
             *(*sym).current == b'<' as i8 || *(*sym).current == b'>' as i8) {
            return core::ptr::null_mut();
        }
        (*sym).current = (*sym).current.offset(1);
        if *(*sym).current == b'@' as i8 {
            break;
        }
    }
    (*sym).current = (*sym).current.offset(1);
    // `sym->current - 1 - ptr` in the C, which parses as `(current - 1) - ptr`: `current` is
    // one PAST the '@', so `current - 1` is the '@' itself and the difference is the
    // identifier's length. The translation folded `current - ptr` into `offset_from` and then
    // kept the `- 1` as well, subtracting it twice — so every identifier came out one byte
    // short. `CBaseUnknown` demangled as `CBaseUnknow`, in the shipping DLL, on every C++
    // symbol. Nothing could see it until undname.c became a reference TU.
    if str_array_push(sym, ptr, (*sym).current.offset_from(ptr) as i32 - 1, core::ptr::addr_of_mut!((*sym).names)) == 0 {
        return core::ptr::null_mut();
    }

    str_array_get_ref(core::ptr::addr_of_mut!((*sym).names), (*sym).names.num - (*sym).names.start - 1)
}
unsafe extern "C" fn get_template_name(sym: *mut parsed_symbol) -> *mut i8 {
    let num_mark = (*sym).names.num;
    let start_mark = (*sym).names.start;
    let stack_mark = (*sym).stack.num;
    let args_mark = (*sym).args.num;

    (*sym).names.start = (*sym).names.num;
    let name = get_literal_string(sym);
    if name.is_null() {
        (*sym).names.start = start_mark;
        // The C says `return FALSE;` from a `char*` function — Wine spelling NULL as the
        // BOOL false. Same value, and this is the pointer that means it.
        return core::ptr::null_mut();
    }
    let args = get_args(sym, FALSE, '<' as i8, '>' as i8);
    let name = if !args.is_null() {
        str_printf!(sym, b"%s%s\0".as_ptr() as *const i8, name, args)
    } else {
        name
    };
    (*sym).names.num = num_mark;
    (*sym).names.start = start_mark;
    (*sym).stack.num = stack_mark;
    (*sym).args.num = args_mark;
    name
}
unsafe extern "C" fn get_class(sym: *mut parsed_symbol) -> u8 {
    let mut name: *mut i8 = core::ptr::null_mut();

    while *(*sym).current != b'@' as i8 {
        match *(*sym).current as u8 {
            0 => return FALSE,

            b'0'..=b'9' => {
                let idx = (*(*sym).current - b'0' as i8) as u32;
                (*sym).current = (*sym).current.offset(1);
                name = str_array_get_ref(core::ptr::addr_of_mut!((*sym).names), idx);
            }
            b'?' => {
                (*sym).current = (*sym).current.offset(1);
                match *(*sym).current as u8 {
                    b'$' => {
                        (*sym).current = (*sym).current.offset(1);
                        name = get_template_name(sym);
                        if !name.is_null() &&
                           str_array_push(sym, name, -1, core::ptr::addr_of_mut!((*sym).names)) == 0 {
                            return FALSE;
                        }
                    }
                    b'?' => {
                        let stack = (*sym).stack;
                        let start = (*sym).names.start;
                        let num = (*sym).names.num;

                        str_array_init(core::ptr::addr_of_mut!((*sym).stack));
                        if symbol_demangle(sym) != 0 {
                            name = str_printf!(sym, b"`%s'\0".as_ptr() as *const i8, (*sym).result);
                        }
                        (*sym).names.start = start;
                        (*sym).names.num = num;
                        (*sym).stack = stack;
                    }
                    _ => {
                        name = get_number(sym) as *mut i8;
                        if name.is_null() {
                            return FALSE;
                        }
                        name = str_printf!(sym, b"`%s'\0".as_ptr() as *const i8, name);
                    }
                }
            }
            _ => {
                name = get_literal_string(sym);
            }
        }
        if name.is_null() || str_array_push(sym, name, -1, core::ptr::addr_of_mut!((*sym).stack)) == 0 {
            return FALSE;
        }
    }
    (*sym).current = (*sym).current.offset(1);
    (TRUE) as u8
}
unsafe extern "C" fn get_class_string(sym: *mut parsed_symbol, start: i32) -> *mut i8 {
    let mut i: i32;
    let mut len: u32;
    let mut sz: u32;
    let ret: *mut i8;
    let a: *mut array = core::ptr::addr_of_mut!((*sym).stack);

    len = 0;
    i = start;
    while i < (*a).num as i32 {
        assert!(!(*a).elts.offset(i as isize).read().is_null());
        len = len + 2 + (strlen((*a).elts.offset(i as isize).read())) as u32;
        i += 1;
    }

    ret = und_alloc(sym, len - 1) as *mut i8;
    if ret.is_null() {
        return core::ptr::null_mut();
    }

    len = 0;
    i = (*a).num as i32 - 1;
    while i >= start {
        sz = (strlen((*a).elts.offset(i as isize).read())) as u32;
        memcpy(ret.offset(len as isize) as *mut core::ffi::c_void, (*a).elts.offset(i as isize).read() as *const core::ffi::c_void, sz as usize);
        len += sz;
        if i > start {
            *ret.offset(len as isize) = b':' as i8;
            len += 1;
            *ret.offset(len as isize) = b':' as i8;
            len += 1;
        }
        i -= 1;
    }
    *ret.offset(len as isize) = b'\0' as i8;
    ret
}
unsafe extern "C" fn get_class_name(sym: *mut parsed_symbol) -> *mut i8 {
    let mark: u32 = (*sym).stack.num;
    let mut s: *mut i8 = core::ptr::null_mut();

    if get_class(sym) != 0 {
        s = get_class_string(sym, mark as i32);
    }
    (*sym).stack.num = mark;
    s
}
unsafe extern "C" fn get_calling_convention(
    ch: i8,
    call_conv: *mut *const i8,
    exported: *mut *const i8,
    flags: u32,
) -> i32 {
    *call_conv = core::ptr::null();
    *exported = core::ptr::null();

    if (flags & (UNDNAME_NO_MS_KEYWORDS | UNDNAME_NO_ALLOCATION_LANGUAGE)) == 0 {
        if (flags & UNDNAME_NO_LEADING_UNDERSCORES) != 0 {
            if ((ch - b'A' as i8) % 2) == 1 {
                *exported = b"dll_export \0".as_ptr() as *const i8;
            }
            match ch as u8 {
                b'A' | b'B' => *call_conv = b"cdecl\0".as_ptr() as *const i8,
                b'C' | b'D' => *call_conv = b"pascal\0".as_ptr() as *const i8,
                b'E' | b'F' => *call_conv = b"thiscall\0".as_ptr() as *const i8,
                b'G' | b'H' => *call_conv = b"stdcall\0".as_ptr() as *const i8,
                b'I' | b'J' => *call_conv = b"fastcall\0".as_ptr() as *const i8,
                b'K' | b'L' => {}
                b'M' => *call_conv = b"clrcall\0".as_ptr() as *const i8,
                _ => {
                    ERR!("Unknown calling convention %c\n", ch as core::ffi::c_int);
                    return (FALSE) as i32;
                }
            }
        } else {
            if ((ch - b'A' as i8) % 2) == 1 {
                *exported = b"__dll_export \0".as_ptr() as *const i8;
            }
            match ch as u8 {
                b'A' | b'B' => *call_conv = b"__cdecl\0".as_ptr() as *const i8,
                b'C' | b'D' => *call_conv = b"__pascal\0".as_ptr() as *const i8,
                b'E' | b'F' => *call_conv = b"__thiscall\0".as_ptr() as *const i8,
                b'G' | b'H' => *call_conv = b"__stdcall\0".as_ptr() as *const i8,
                b'I' | b'J' => *call_conv = b"__fastcall\0".as_ptr() as *const i8,
                b'K' | b'L' => {}
                b'M' => *call_conv = b"__clrcall\0".as_ptr() as *const i8,
                _ => {
                    ERR!("Unknown calling convention %c\n", ch as core::ffi::c_int);
                    return (FALSE) as i32;
                }
            }
        }
    }
    (TRUE) as i32
}
unsafe extern "C" fn get_simple_type(c: i8) -> *const i8 {
    let type_string: *const i8;

    match c as u8 {
        b'C' => type_string = b"signed char\0".as_ptr() as *const i8,
        b'D' => type_string = b"char\0".as_ptr() as *const i8,
        b'E' => type_string = b"unsigned char\0".as_ptr() as *const i8,
        b'F' => type_string = b"short\0".as_ptr() as *const i8,
        b'G' => type_string = b"unsigned short\0".as_ptr() as *const i8,
        b'H' => type_string = b"int\0".as_ptr() as *const i8,
        b'I' => type_string = b"unsigned int\0".as_ptr() as *const i8,
        b'J' => type_string = b"long\0".as_ptr() as *const i8,
        b'K' => type_string = b"unsigned long\0".as_ptr() as *const i8,
        b'M' => type_string = b"float\0".as_ptr() as *const i8,
        b'N' => type_string = b"double\0".as_ptr() as *const i8,
        b'O' => type_string = b"long double\0".as_ptr() as *const i8,
        b'X' => type_string = b"void\0".as_ptr() as *const i8,
        _ => type_string = core::ptr::null(),
    }
    type_string
}
unsafe extern "C" fn get_extended_type(c: i8) -> *const i8 {
    let type_string: *const i8;

    match c as u8 {
        b'D' => type_string = b"__int8\0".as_ptr() as *const i8,
        b'E' => type_string = b"unsigned __int8\0".as_ptr() as *const i8,
        b'F' => type_string = b"__int16\0".as_ptr() as *const i8,
        b'G' => type_string = b"unsigned __int16\0".as_ptr() as *const i8,
        b'H' => type_string = b"__int32\0".as_ptr() as *const i8,
        b'I' => type_string = b"unsigned __int32\0".as_ptr() as *const i8,
        b'J' => type_string = b"__int64\0".as_ptr() as *const i8,
        b'K' => type_string = b"unsigned __int64\0".as_ptr() as *const i8,
        b'L' => type_string = b"__int128\0".as_ptr() as *const i8,
        b'M' => type_string = b"unsigned __int128\0".as_ptr() as *const i8,
        b'N' => type_string = b"bool\0".as_ptr() as *const i8,
        b'Q' => type_string = b"char8_t\0".as_ptr() as *const i8,
        b'S' => type_string = b"char16_t\0".as_ptr() as *const i8,
        b'U' => type_string = b"char32_t\0".as_ptr() as *const i8,
        b'W' => type_string = b"wchar_t\0".as_ptr() as *const i8,
        _ => type_string = core::ptr::null(),
    }
    type_string
}
unsafe extern "C" fn get_function_signature(sym: *mut parsed_symbol, fs: *mut function_signature) -> i32 {
    let mark = (*sym).stack.num;

    // `get_calling_convention(*sym->current++, …)` in the C. The `++` is a side effect
    // INSIDE the first argument, so the cursor moves past the calling-convention letter
    // BEFORE `demangle_datatype` runs on the same `sym`. The translation hoisted it out of
    // the condition and to AFTER the whole `if`, which left the return type being parsed
    // from the calling-convention letter itself — `A` reads as a reference, so every
    // function signature in the DLL came out with a return type of ` &` and its real return
    // type shifted into the argument list. `void * (__cdecl*)(long)` demangled as
    // ` & (__cdecl*)(void,long)`.
    //
    // Sequencing, not arithmetic: both forms advance the cursor exactly once, so nothing
    // leaks and nothing crashes. Only the ORDER differs, which is why it survived a build,
    // a boot, and every scanner.
    let call_conv_char = *(*sym).current;
    (*sym).current = (*sym).current.offset(1);
    if get_calling_convention(
        call_conv_char,
        core::ptr::addr_of_mut!((*fs).call_conv),
        core::ptr::addr_of_mut!((*fs).exported),
        (*sym).flags & !UNDNAME_NO_ALLOCATION_LANGUAGE,
    ) == 0
        || demangle_datatype(sym, core::ptr::addr_of_mut!((*fs).return_ct), 0) == 0
    {
        return (FALSE) as i32;
    }

    (*fs).arguments = get_args(sym, (TRUE) as u8, '(' as i8, ')' as i8);
    if (*fs).arguments.is_null() {
        return (FALSE) as i32;
    }
    (*sym).stack.num = mark;

    (TRUE) as i32
}
unsafe extern "C" fn demangle_datatype(
    sym: *mut parsed_symbol,
    ct: *mut datatype_t,
    flags: datatype_flags,
) -> u8 {
    let mut dt: i8;

    assert!(!ct.is_null());
    (*ct).left = core::ptr::null();
    (*ct).right = core::ptr::null();
    (*ct).flags = 0;

    dt = *(*sym).current;
    (*sym).current = (*sym).current.offset(1);

    'done: loop {
        match dt as u8 {
            b'_' => {
                let c = *(*sym).current;
                (*sym).current = (*sym).current.offset(1);
                (*ct).left = get_extended_type(c);
            }
            b'C' | b'D' | b'E' | b'F' | b'G' | b'H'
            | b'I' | b'J' | b'K' | b'M' | b'N' | b'O'
            | b'X' | b'Z' => {
                (*ct).left = get_simple_type(dt);
            }
            b'T' | b'U' | b'V' | b'Y' => {
                let struct_name: *const i8;
                let mut type_name: *const i8 = core::ptr::null();

                struct_name = get_class_name(sym);
                if struct_name.is_null() {
                    break 'done;
                }
                if ((*sym).flags & UNDNAME_NO_COMPLEX_TYPE) == 0 {
                    type_name = match dt as u8 {
                        b'T' => b"union \0".as_ptr() as *const i8,
                        b'U' => b"struct \0".as_ptr() as *const i8,
                        b'V' => b"class \0".as_ptr() as *const i8,
                        b'Y' => b"cointerface \0".as_ptr() as *const i8,
                        _ => core::ptr::null(),
                    };
                }
                (*ct).left = str_printf!(sym, b"%s%s\0".as_ptr() as *const i8, type_name, struct_name);
            }
            b'?' => {
                if (flags & IN_ARGS) != 0 {
                    let ptr: *const i8;
                    ptr = get_number(sym);
                    if ptr.is_null() {
                        break 'done;
                    }
                    (*ct).left = str_printf!(sym, b"`template-parameter-%s'\0".as_ptr() as *const i8, ptr);
                } else {
                    if get_qualified_type(ct, sym, b'?' as i8, flags) == 0 {
                        break 'done;
                    }
                }
            }
            b'A' | b'B' => {
                if get_qualified_type(ct, sym, dt, flags) == 0 {
                    break 'done;
                }
            }
            b'P' | b'Q' | b'R' | b'S' => {
                let mut dt_local = dt;
                if (flags & IN_ARGS) == 0 {
                    dt_local = b'P' as i8;
                }
                if isdigit((*(*sym).current) as i32) != 0 {
                    let ptr_qualif: *const i8;
                    ptr_qualif = match dt_local as u8 {
                        b'P' => core::ptr::null(),
                        b'Q' => b"const\0".as_ptr() as *const i8,
                        b'R' => b"volatile\0".as_ptr() as *const i8,
                        b'S' => b"const volatile\0".as_ptr() as *const i8,
                        _ => core::ptr::null(),
                    };

                    if *(*sym).current == b'8' as i8 {
                        let mut fs: function_signature = core::mem::zeroed();
                        let class: *const i8;
                        // OUT parameter of get_function_qualifier, which writes it
                        // on every TRUE path; null is what its own `*qualif = ...`
                        // leaves when there is no qualifier. Rust needs the binding
                        // live because the address is taken.
                        let mut function_qualifier: *const i8 = core::ptr::null();

                        (*sym).current = (*sym).current.offset(1);

                        class = get_class_name(sym);
                        if class.is_null() {
                            break 'done;
                        }
                        if get_function_qualifier(sym, core::ptr::addr_of_mut!(function_qualifier)) == 0 {
                            break 'done;
                        }
                        if get_function_signature(sym, core::ptr::addr_of_mut!(fs)) == 0 {
                            break 'done;
                        }

                        (*ct).left = str_printf!(
                            sym,
                            b"%s%s (%s %s::*%s\0".as_ptr() as *const i8,
                            fs.return_ct.left,
                            fs.return_ct.right,
                            fs.call_conv,
                            class,
                            ptr_qualif,
                        );
                        (*ct).right = str_printf!(
                            sym,
                            b")%s%s\0".as_ptr() as *const i8,
                            fs.arguments,
                            function_qualifier,
                        );
                    } else if *(*sym).current == b'6' as i8 {
                        let mut fs: function_signature = core::mem::zeroed();

                        (*sym).current = (*sym).current.offset(1);

                        if get_function_signature(sym, core::ptr::addr_of_mut!(fs)) == 0 {
                            break 'done;
                        }

                        (*ct).left = str_printf!(
                            sym,
                            b"%s%s (%s*%s\0".as_ptr() as *const i8,
                            fs.return_ct.left,
                            fs.return_ct.right,
                            fs.call_conv,
                            ptr_qualif,
                        );
                        (*ct).flags = DT_NO_LEADING_WS;
                        (*ct).right = str_printf!(sym, b")%s\0".as_ptr() as *const i8, fs.arguments);
                    } else {
                        break 'done;
                    }
                } else if get_qualified_type(ct, sym, dt_local, flags) == 0 {
                    break 'done;
                }
            }
            b'W' => {
                if *(*sym).current == b'4' as i8 {
                    let enum_name: *mut i8;
                    (*sym).current = (*sym).current.offset(1);
                    enum_name = get_class_name(sym);
                    if enum_name.is_null() {
                        break 'done;
                    }
                    if ((*sym).flags & UNDNAME_NO_COMPLEX_TYPE) != 0 {
                        (*ct).left = enum_name;
                    } else {
                        (*ct).left = str_printf!(sym, b"enum %s\0".as_ptr() as *const i8, enum_name);
                    }
                } else {
                    break 'done;
                }
            }
            b'0' | b'1' | b'2' | b'3' | b'4' | b'5'
            | b'6' | b'7' | b'8' | b'9' => {
                (*ct).left = str_array_get_ref(
                    core::ptr::addr_of_mut!((*sym).args),
                    ((dt - b'0' as i8) as u32) * 2,
                );
                (*ct).right = str_array_get_ref(
                    core::ptr::addr_of_mut!((*sym).args),
                    ((dt - b'0' as i8) as u32) * 2 + 1,
                );
                if (*ct).left.is_null() {
                    break 'done;
                }
            }
            b'$' => {
                let sub_dt = *(*sym).current;
                (*sym).current = (*sym).current.offset(1);
                match sub_dt as u8 {
                    b'0' => {
                        (*ct).left = get_number(sym);
                        if (*ct).left.is_null() {
                            break 'done;
                        }
                    }
                    b'D' => {
                        let ptr: *const i8;
                        ptr = get_number(sym);
                        if ptr.is_null() {
                            break 'done;
                        }
                        (*ct).left = str_printf!(sym, b"`template-parameter%s'\0".as_ptr() as *const i8, ptr);
                    }
                    b'F' => {
                        let p1: *const i8;
                        let p2: *const i8;
                        p1 = get_number(sym);
                        if p1.is_null() {
                            break 'done;
                        }
                        p2 = get_number(sym);
                        if p2.is_null() {
                            break 'done;
                        }
                        (*ct).left = str_printf!(sym, b"{%s,%s}\0".as_ptr() as *const i8, p1, p2);
                    }
                    b'G' => {
                        let p1: *const i8;
                        let p2: *const i8;
                        let p3: *const i8;
                        p1 = get_number(sym);
                        if p1.is_null() {
                            break 'done;
                        }
                        p2 = get_number(sym);
                        if p2.is_null() {
                            break 'done;
                        }
                        p3 = get_number(sym);
                        if p3.is_null() {
                            break 'done;
                        }
                        (*ct).left = str_printf!(sym, b"{%s,%s,%s}\0".as_ptr() as *const i8, p1, p2, p3);
                    }
                    b'Q' => {
                        let ptr: *const i8;
                        ptr = get_number(sym);
                        if ptr.is_null() {
                            break 'done;
                        }
                        (*ct).left = str_printf!(sym, b"`non-type-template-parameter%s'\0".as_ptr() as *const i8, ptr);
                    }
                    b'$' => {
                        if *(*sym).current == b'A' as i8 {
                            (*sym).current = (*sym).current.offset(1);
                            if *(*sym).current == b'6' as i8 {
                                let mut fs: function_signature = core::mem::zeroed();

                                (*sym).current = (*sym).current.offset(1);

                                if get_function_signature(sym, core::ptr::addr_of_mut!(fs)) == 0 {
                                    break 'done;
                                }
                                (*ct).left = str_printf!(
                                    sym,
                                    b"%s%s %s%s\0".as_ptr() as *const i8,
                                    fs.return_ct.left,
                                    fs.return_ct.right,
                                    fs.call_conv,
                                    fs.arguments,
                                );
                            }
                        } else if *(*sym).current == b'B' as i8 {
                            let mark: u32 = (*sym).stack.num;
                            let mut sub_ct: datatype_t = core::mem::zeroed();
                            let mut arr: *const i8 = core::ptr::null();
                            (*sym).current = (*sym).current.offset(1);

                            if *(*sym).current == b'Y' as i8 {
                                let n1: *const i8;
                                let mut num: i32;

                                (*sym).current = (*sym).current.offset(1);
                                n1 = get_number(sym);
                                if n1.is_null() {
                                    break 'done;
                                }
                                num = atoi((n1).cast());

                                while num > 0 {
                                    arr = str_printf!(sym, b"%s[%s]\0".as_ptr() as *const i8, arr, get_number(sym));
                                    num -= 1;
                                }
                            }

                            if demangle_datatype(sym, core::ptr::addr_of_mut!(sub_ct), 0) == 0 {
                                break 'done;
                            }

                            if !arr.is_null() {
                                (*ct).left = str_printf!(sym, b"%s %s\0".as_ptr() as *const i8, sub_ct.left, arr);
                            } else {
                                (*ct).left = sub_ct.left;
                            }
                            (*ct).right = sub_ct.right;
                            (*sym).stack.num = mark;
                        } else if *(*sym).current == b'C' as i8 {
                            let mut xdt: datatype_t = core::mem::zeroed();

                            (*sym).current = (*sym).current.offset(1);
                            if get_qualifier(sym, core::ptr::addr_of_mut!(xdt), core::ptr::null_mut()) == 0 {
                                break 'done;
                            }
                            if demangle_datatype(sym, ct, flags) == 0 {
                                break 'done;
                            }
                            (*ct).left = str_printf!(sym, b"%s %s\0".as_ptr() as *const i8, (*ct).left, xdt.left);
                        } else if *(*sym).current == b'Q' as i8 {
                            (*sym).current = (*sym).current.offset(1);
                            if get_qualified_type(ct, sym, b'$' as i8, flags) == 0 {
                                break 'done;
                            }
                        } else if *(*sym).current == b'T' as i8 {
                            (*sym).current = (*sym).current.offset(1);
                            (*ct).left = str_printf!(sym, b"std::nullptr_t\0".as_ptr() as *const i8);
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                ERR!("Unknown type %c\n", dt as core::ffi::c_int);
            }
        }
        break;
    }

    if (*ct).left.is_null() {
        0u8
    } else {
        1u8
    }
}
unsafe extern "C" fn handle_data(sym: *mut parsed_symbol) -> u8 {
    let mut access: *const i8 = core::ptr::null();
    let mut member_type: *const i8 = core::ptr::null();
    let mut xdt: datatype_t = datatype_t {
        left: core::ptr::null(),
        right: core::ptr::null(),
        flags: 0,
    };
    let mut ct: datatype_t = core::mem::zeroed();
    let mut name: *mut i8;
    let mut ret: u8 = FALSE;

    'done: {
        if ((*sym).flags & UNDNAME_NO_ACCESS_SPECIFIERS) == 0 {
            match *(*sym).current as u8 {
                b'0' => access = b"private: \0".as_ptr() as *const i8,
                b'1' => access = b"protected: \0".as_ptr() as *const i8,
                b'2' => access = b"public: \0".as_ptr() as *const i8,
                _ => {}
            }
        }

        if ((*sym).flags & UNDNAME_NO_MEMBER_TYPE) == 0 {
            if *(*sym).current >= (b'0') as i8 && *(*sym).current <= (b'2') as i8 {
                member_type = b"static \0".as_ptr() as *const i8;
            }
        }

        name = get_class_string(sym, 0);

        match *(*sym).current as u8 {
            b'0' | b'1' | b'2' | b'3' | b'4' | b'5' => {
                (*sym).current = (*sym).current.offset(1);
                let mark = (*sym).stack.num;
                let mut class: *const i8 = core::ptr::null();

                if demangle_datatype(sym, &mut ct, 0) == 0 {
                    break 'done;
                }
                if get_qualifier(sym, &mut xdt, &mut class) == 0 {
                    break 'done;
                }
                if !xdt.left.is_null() && !xdt.right.is_null() {
                    xdt.left = str_printf!(sym, b"%s %s\0".as_ptr() as *const i8, xdt.left, xdt.right);
                } else if xdt.left.is_null() {
                    xdt.left = xdt.right;
                }
                (*sym).stack.num = mark;
            }
            b'6' | b'7' => {
                (*sym).current = (*sym).current.offset(1);
                ct.left = core::ptr::null();
                ct.right = core::ptr::null();
                if get_qualifier(sym, &mut xdt, core::ptr::null_mut()) == 0 {
                    break 'done;
                }
                if *(*sym).current != (b'@') as i8 {
                    let mut cls: *mut i8;

                    cls = get_class_name(sym);
                    if cls.is_null() {
                        break 'done;
                    }
                    ct.right = str_printf!(sym, b"{for `%s'}\0".as_ptr() as *const i8, cls);
                }
            }
            b'8' | b'9' => {
                (*sym).current = (*sym).current.offset(1);
                xdt.left = core::ptr::null();
                ct.left = core::ptr::null();
                ct.right = core::ptr::null();
            }
            _ => {
                break 'done;
            }
        }

        if ((*sym).flags & UNDNAME_NAME_ONLY) != 0 {
            ct.left = core::ptr::null();
            ct.right = core::ptr::null();
            xdt.left = core::ptr::null();
        }

        (*sym).result = str_printf!(
            sym,
            b"%s%s%s%s%s%s%s%s\0".as_ptr() as *const i8,
            access,
            member_type,
            ct.left,
            if !xdt.left.is_null() && !ct.left.is_null() { b" \0".as_ptr() as *const i8 } else { core::ptr::null() },
            xdt.left,
            if !xdt.left.is_null() || !ct.left.is_null() { b" \0".as_ptr() as *const i8 } else { core::ptr::null() },
            name,
            ct.right
        );
        ret = (TRUE) as u8;
    }

    ret
}
unsafe extern "C" fn handle_method(sym: *mut parsed_symbol, cast_op: u8) -> u8 {
    let mut accmem: i8;
    let mut access: *const i8 = core::ptr::null();
    let mut access_id: i32 = -1;
    let mut member_type: *const i8 = core::ptr::null();
    let mut ct_ret: datatype_t = core::mem::zeroed();
    let mut call_conv: *const i8 = core::ptr::null();
    let mut function_qualifier: *const i8 = core::ptr::null();
    let mut exported: *const i8 = core::ptr::null();
    let mut args_str: *const i8 = core::ptr::null();
    let mut name: *const i8;
    let mut ret: u8 = FALSE;
    let mut has_args: u8 = (TRUE) as u8;
    let mut has_ret: u8 = (TRUE) as u8;
    let mark: u32;

    accmem = *(*sym).current;
    (*sym).current = (*sym).current.offset(1);

    if accmem == '$' as i8 {
        if *(*sym).current >= '0' as i8 && *(*sym).current <= '5' as i8 {
            access_id = (*(*sym).current - '0' as i8) as i32 / 2;
        } else if *(*sym).current == 'R' as i8 {
            access_id = (*(*sym).current.offset(1) - '0' as i8) as i32 / 2;
        } else if *(*sym).current != 'B' as i8 {
            return ret;
        }
    } else if accmem >= 'A' as i8 && accmem <= 'Z' as i8 {
        access_id = (accmem - 'A' as i8) as i32 / 8;
    } else {
        return ret;
    }

    match access_id {
        0 => access = b"private: \0".as_ptr() as *const i8,
        1 => access = b"protected: \0".as_ptr() as *const i8,
        2 => access = b"public: \0".as_ptr() as *const i8,
        _ => {}
    }

    if accmem == '$' as i8 || (accmem - 'A' as i8) as i32 % 8 == 6 || (accmem - 'A' as i8) as i32 % 8 == 7 {
        access = str_printf!(sym, b"[thunk]:%s\0".as_ptr() as *const i8, if !access.is_null() { access } else { b" \0".as_ptr() as *const i8 });
    }

    if accmem == '$' as i8 && *(*sym).current != 'B' as i8 {
        member_type = b"virtual \0".as_ptr() as *const i8;
    } else if accmem <= 'X' as i8 {
        match (accmem - 'A' as i8) as i32 % 8 {
            2 | 3 => member_type = b"static \0".as_ptr() as *const i8,
            4 | 5 | 6 | 7 => member_type = b"virtual \0".as_ptr() as *const i8,
            _ => {}
        }
    }

    if (*sym).flags & UNDNAME_NO_ACCESS_SPECIFIERS != 0 {
        access = core::ptr::null();
    }
    if (*sym).flags & UNDNAME_NO_MEMBER_TYPE != 0 {
        member_type = core::ptr::null();
    }

    name = get_class_string(sym, 0);

    if accmem == '$' as i8 && *(*sym).current == 'B' as i8 {
        let n: *const i8;

        (*sym).current = (*sym).current.offset(1);
        n = get_number(sym);

        if n.is_null() || { let c = *(*sym).current; (*sym).current = (*sym).current.offset(1); c } != 'A' as i8 {
            return ret;
        }
        name = str_printf!(sym, b"%s{%s,{flat}}' }'\0".as_ptr() as *const i8, name, n);
        has_args = FALSE;
        has_ret = FALSE;
    } else if accmem == '$' as i8 && *(*sym).current == 'R' as i8 {
        let n1: *const i8;
        let n2: *const i8;
        let n3: *const i8;
        let n4: *const i8;

        (*sym).current = (*sym).current.offset(2);
        n1 = get_number(sym);
        n2 = get_number(sym);
        n3 = get_number(sym);
        n4 = get_number(sym);

        if n1.is_null() || n2.is_null() || n3.is_null() || n4.is_null() {
            return ret;
        }
        name = str_printf!(sym, b"%s`vtordispex{%s,%s,%s,%s}' \0".as_ptr() as *const i8, name, n1, n2, n3, n4);
    } else if accmem == '$' as i8 {
        let n1: *const i8;
        let n2: *const i8;

        (*sym).current = (*sym).current.offset(1);
        n1 = get_number(sym);
        n2 = get_number(sym);

        if n1.is_null() || n2.is_null() {
            return ret;
        }
        name = str_printf!(sym, b"%s`vtordisp{%s,%s}' \0".as_ptr() as *const i8, name, n1, n2);
    } else if (accmem - 'A' as i8) as i32 % 8 == 6 || (accmem - 'A' as i8) as i32 % 8 == 7 {
        name = str_printf!(sym, b"%s`adjustor{%s}' \0".as_ptr() as *const i8, name, get_number(sym));
    }

    if has_args != 0 && (accmem == '$' as i8 ||
                (accmem <= 'X' as i8 && (accmem - 'A' as i8) as i32 % 8 != 2 && (accmem - 'A' as i8) as i32 % 8 != 3)) {
        if get_function_qualifier(sym, core::ptr::addr_of_mut!(function_qualifier)) == 0 {
            return ret;
        }
    }

    if get_calling_convention({ let c = *(*sym).current; (*sym).current = (*sym).current.offset(1); c }, core::ptr::addr_of_mut!(call_conv), core::ptr::addr_of_mut!(exported), (*sym).flags) == 0 {
        return ret;
    }

    if has_ret != 0 && *(*sym).current == '@' as i8 {
        ct_ret.left = b"void\0".as_ptr() as *const i8;
        ct_ret.right = core::ptr::null();
        (*sym).current = (*sym).current.offset(1);
    } else if has_ret != 0 {
        if demangle_datatype(sym, core::ptr::addr_of_mut!(ct_ret), if cast_op != 0 { WS_AFTER_QUAL_IF } else { 0 }) == 0 {
            return ret;
        }
    }
    if has_ret == 0 || (*sym).flags & UNDNAME_NO_FUNCTION_RETURNS != 0 {
        ct_ret.left = core::ptr::null();
        ct_ret.right = core::ptr::null();
    }
    if cast_op != 0 {
        name = str_printf!(sym, b"%s %s%s\0".as_ptr() as *const i8, name, ct_ret.left, ct_ret.right);
        ct_ret.left = core::ptr::null();
        ct_ret.right = core::ptr::null();
    }

    mark = (*sym).stack.num;
    if has_ret != 0 {
        args_str = get_args(sym, (TRUE) as u8, '(' as i8, ')' as i8);
        if args_str.is_null() {
            return ret;
        }
    }
    if (*sym).flags & UNDNAME_NAME_ONLY != 0 {
        args_str = core::ptr::null();
        function_qualifier = core::ptr::null();
    }
    if (*sym).flags & UNDNAME_NO_THISTYPE != 0 {
        function_qualifier = core::ptr::null();
    }
    (*sym).stack.num = mark;

    (*sym).result = str_printf!(sym, b"%s%s%s%s%s%s%s%s%s%s%s\0".as_ptr() as *const i8,
                             access, member_type, ct_ret.left,
                             if !ct_ret.left.is_null() && ct_ret.right.is_null() { b" \0".as_ptr() as *const i8 } else { core::ptr::null() },
                             call_conv, if !call_conv.is_null() { b" \0".as_ptr() as *const i8 } else { core::ptr::null() }, exported,
                             name, args_str, function_qualifier, ct_ret.right);
    ret = (TRUE) as u8;

    ret
}
unsafe extern "C" fn symbol_demangle(sym: *mut parsed_symbol) -> u8 {
    let mut ret = FALSE;
    enum PostProcess {
        PP_NONE,
        PP_CONSTRUCTOR,
        PP_DESTRUCTOR,
        PP_CAST_OPERATOR,
    }
    let mut post_process = PostProcess::PP_NONE;

    'done: {
        if (*sym).flags & UNDNAME_NO_ARGUMENTS != 0 {
            let mut ct: datatype_t = core::mem::zeroed();

            if demangle_datatype(sym, &mut ct, 0) != 0 {
                (*sym).result = str_printf!(sym, b"%s%s\0".as_ptr() as *const i8, ct.left, ct.right);
                ret = (TRUE) as u8;
            }
            break 'done;
        }

        if *(*sym).current != b'?' as i8 {
            return FALSE;
        }
        (*sym).current = (*sym).current.offset(1);

        if *(*sym).current == b'?' as i8 {
            let mut function_name: *const i8 = core::ptr::null();
            let mut in_template = FALSE;

            if *(*sym).current.offset(1) == b'$' as i8 && *(*sym).current.offset(2) == b'?' as i8 {
                in_template = (TRUE) as u8;
                (*sym).current = (*sym).current.offset(2);
            }

            (*sym).current = (*sym).current.offset(1);
            match *(*sym).current as u8 {
                b'0' => { function_name = b"\0".as_ptr() as *const i8; post_process = PostProcess::PP_CONSTRUCTOR; }
                b'1' => { function_name = b"\0".as_ptr() as *const i8; post_process = PostProcess::PP_DESTRUCTOR; }
                b'2' => function_name = b"operator new\0".as_ptr() as *const i8,
                b'3' => function_name = b"operator delete\0".as_ptr() as *const i8,
                b'4' => function_name = b"operator=\0".as_ptr() as *const i8,
                b'5' => function_name = b"operator>>\0".as_ptr() as *const i8,
                b'6' => function_name = b"operator<<\0".as_ptr() as *const i8,
                b'7' => function_name = b"operator!\0".as_ptr() as *const i8,
                b'8' => function_name = b"operator==\0".as_ptr() as *const i8,
                b'9' => function_name = b"operator!=\0".as_ptr() as *const i8,
                b'A' => function_name = b"operator[]\0".as_ptr() as *const i8,
                b'B' => { function_name = b"operator\0".as_ptr() as *const i8; post_process = PostProcess::PP_CAST_OPERATOR; }
                b'C' => function_name = b"operator->\0".as_ptr() as *const i8,
                b'D' => function_name = b"operator*\0".as_ptr() as *const i8,
                b'E' => function_name = b"operator++\0".as_ptr() as *const i8,
                b'F' => function_name = b"operator--\0".as_ptr() as *const i8,
                b'G' => function_name = b"operator-\0".as_ptr() as *const i8,
                b'H' => function_name = b"operator+\0".as_ptr() as *const i8,
                b'I' => function_name = b"operator&\0".as_ptr() as *const i8,
                b'J' => function_name = b"operator->*\0".as_ptr() as *const i8,
                b'K' => function_name = b"operator/\0".as_ptr() as *const i8,
                b'L' => function_name = b"operator%\0".as_ptr() as *const i8,
                b'M' => function_name = b"operator<\0".as_ptr() as *const i8,
                b'N' => function_name = b"operator<=\0".as_ptr() as *const i8,
                b'O' => function_name = b"operator>\0".as_ptr() as *const i8,
                b'P' => function_name = b"operator>=\0".as_ptr() as *const i8,
                b'Q' => function_name = b"operator,\0".as_ptr() as *const i8,
                b'R' => function_name = b"operator()\0".as_ptr() as *const i8,
                b'S' => function_name = b"operator~\0".as_ptr() as *const i8,
                b'T' => function_name = b"operator^\0".as_ptr() as *const i8,
                b'U' => function_name = b"operator|\0".as_ptr() as *const i8,
                b'V' => function_name = b"operator&&\0".as_ptr() as *const i8,
                b'W' => function_name = b"operator||\0".as_ptr() as *const i8,
                b'X' => function_name = b"operator*=\0".as_ptr() as *const i8,
                b'Y' => function_name = b"operator+=\0".as_ptr() as *const i8,
                b'Z' => function_name = b"operator-=\0".as_ptr() as *const i8,
                b'_' => {
                    (*sym).current = (*sym).current.offset(1);
                    match *(*sym).current as u8 {
                        b'0' => function_name = b"operator/=\0".as_ptr() as *const i8,
                        b'1' => function_name = b"operator%=\0".as_ptr() as *const i8,
                        b'2' => function_name = b"operator>>=\0".as_ptr() as *const i8,
                        b'3' => function_name = b"operator<<=\0".as_ptr() as *const i8,
                        b'4' => function_name = b"operator&=\0".as_ptr() as *const i8,
                        b'5' => function_name = b"operator|=\0".as_ptr() as *const i8,
                        b'6' => function_name = b"operator^=\0".as_ptr() as *const i8,
                        b'7' => function_name = b"`vftable'\0".as_ptr() as *const i8,
                        b'8' => function_name = b"`vbtable'\0".as_ptr() as *const i8,
                        b'9' => function_name = b"`vcall'\0".as_ptr() as *const i8,
                        b'A' => function_name = b"`typeof'\0".as_ptr() as *const i8,
                        b'B' => function_name = b"`local static guard'\0".as_ptr() as *const i8,
                        b'C' => {
                            (*sym).result = b"`string'\0".as_ptr() as *mut i8;
                            if *(*sym).current != 0 && *(*sym).current.offset((strlen((*sym).current) - 1) as isize) == b'@' as i8 {
                                ret = (TRUE) as u8;
                            }
                            break 'done;
                        }
                        b'D' => function_name = b"`vbase destructor'\0".as_ptr() as *const i8,
                        b'E' => function_name = b"`vector deleting destructor'\0".as_ptr() as *const i8,
                        b'F' => function_name = b"`default constructor closure'\0".as_ptr() as *const i8,
                        b'G' => function_name = b"`scalar deleting destructor'\0".as_ptr() as *const i8,
                        b'H' => function_name = b"`vector constructor iterator'\0".as_ptr() as *const i8,
                        b'I' => function_name = b"`vector destructor iterator'\0".as_ptr() as *const i8,
                        b'J' => function_name = b"`vector vbase constructor iterator'\0".as_ptr() as *const i8,
                        b'K' => function_name = b"`virtual displacement map'\0".as_ptr() as *const i8,
                        b'L' => function_name = b"`eh vector constructor iterator'\0".as_ptr() as *const i8,
                        b'M' => function_name = b"`eh vector destructor iterator'\0".as_ptr() as *const i8,
                        b'N' => function_name = b"`eh vector vbase constructor iterator'\0".as_ptr() as *const i8,
                        b'O' => function_name = b"`copy constructor closure'\0".as_ptr() as *const i8,
                        b'R' => {
                            (*sym).flags |= UNDNAME_NO_FUNCTION_RETURNS;
                            (*sym).current = (*sym).current.offset(1);
                            match *(*sym).current as u8 {
                                b'0' => {
                                    let mut ct: datatype_t = core::mem::zeroed();
                                    (*sym).current = (*sym).current.offset(1);
                                    if demangle_datatype(sym, &mut ct, 0) == 0 {
                                        break 'done;
                                    }
                                    function_name = str_printf!(sym, b"%s%s `RTTI Type Descriptor'\0".as_ptr() as *const i8, ct.left, ct.right);
                                    (*sym).current = (*sym).current.offset(-1);
                                }
                                b'1' => {
                                    (*sym).current = (*sym).current.offset(1);
                                    let n1 = get_number(sym);
                                    let n2 = get_number(sym);
                                    let n3 = get_number(sym);
                                    let n4 = get_number(sym);
                                    (*sym).current = (*sym).current.offset(-1);
                                    function_name = str_printf!(sym, b"`RTTI Base Class Descriptor at (%s,%s,%s,%s)'\0".as_ptr() as *const i8, n1, n2, n3, n4);
                                }
                                b'2' => function_name = b"`RTTI Base Class Array'\0".as_ptr() as *const i8,
                                b'3' => function_name = b"`RTTI Class Hierarchy Descriptor'\0".as_ptr() as *const i8,
                                b'4' => function_name = b"`RTTI Complete Object Locator'\0".as_ptr() as *const i8,
                                _ => {
                                    ERR!("Unknown RTTI operator: _R%c\n", *(*sym).current as core::ffi::c_int);
                                }
                            }
                        }
                        b'S' => function_name = b"`local vftable'\0".as_ptr() as *const i8,
                        b'T' => function_name = b"`local vftable constructor closure'\0".as_ptr() as *const i8,
                        b'U' => function_name = b"operator new[]\0".as_ptr() as *const i8,
                        b'V' => function_name = b"operator delete[]\0".as_ptr() as *const i8,
                        b'X' => function_name = b"`placement delete closure'\0".as_ptr() as *const i8,
                        b'Y' => function_name = b"`placement delete[] closure'\0".as_ptr() as *const i8,
                        b'_' => {
                            (*sym).current = (*sym).current.offset(1);
                            match *(*sym).current as u8 {
                                b'K' => {
                                    (*sym).current = (*sym).current.offset(1);
                                    function_name = str_printf!(sym, b"operator \"\" %s\0".as_ptr() as *const i8, get_literal_string(sym));
                                    (*sym).current = (*sym).current.offset(-1);
                                }
                                _ => {
                                    FIXME!("Unknown operator: __%c\n", *(*sym).current as core::ffi::c_int);
                                    return FALSE;
                                }
                            }
                        }
                        _ => {
                            ERR!("Unknown operator: _%c\n", *(*sym).current as core::ffi::c_int);
                            return FALSE;
                        }
                    }
                }
                b'$' => {
                    (*sym).current = (*sym).current.offset(1);
                    function_name = get_template_name(sym);
                    if function_name.is_null() {
                        break 'done;
                    }
                    (*sym).current = (*sym).current.offset(-1);
                }
                _ => {
                    // FIXME: Other operators
                    ERR!("Unknown operator: %c\n", *(*sym).current as core::ffi::c_int);
                    return FALSE;
                }
            }
            (*sym).current = (*sym).current.offset(1);
            if in_template != 0 {
                let args_mark = (*sym).args.num;
                let args = get_args(sym, FALSE, b'<' as i8, b'>' as i8);
                if !args.is_null() {
                    function_name = if !function_name.is_null() {
                        str_printf!(sym, b"%s%s\0".as_ptr() as *const i8, function_name, args)
                    } else {
                        args
                    };
                }
                (*sym).args.num = args_mark;
                (*sym).names.num = 0;
            }
            if str_array_push(sym, function_name, -1, core::ptr::addr_of_mut!((*sym).stack)) == 0 {
                return FALSE;
            }
        } else if *(*sym).current == b'$' as i8 {
            (*sym).current = (*sym).current.offset(1);
            (*sym).result = get_template_name(sym);
            ret = if !(*sym).result.is_null() { 1 } else { 0 };
            break 'done;
        }

        match *(*sym).current as u8 {
            b'@' => { (*sym).current = (*sym).current.offset(1); }
            b'$' => {}
            _ => {
                if get_class(sym) == 0 {
                    break 'done;
                }
            }
        }

        match post_process {
            PostProcess::PP_NONE => {}
            PostProcess::PP_CONSTRUCTOR | PostProcess::PP_DESTRUCTOR => {
                if (*sym).stack.num <= 1 {
                    break 'done;
                }
                *(*sym).stack.elts.offset(0 as isize) = str_printf!(
                    sym,
                    b"%s%s%s\0".as_ptr() as *const i8,
                    if matches!(post_process, PostProcess::PP_DESTRUCTOR) { b"~\0".as_ptr() as *const i8 } else { core::ptr::null() },
                    *(*sym).stack.elts.offset(1 as isize),
                    *(*sym).stack.elts.offset(0 as isize)
                );
                (*sym).flags |= UNDNAME_NO_FUNCTION_RETURNS;
            }
            PostProcess::PP_CAST_OPERATOR => {
                (*sym).flags &= !UNDNAME_NO_FUNCTION_RETURNS;
            }
        }

        if *(*sym).current >= b'0' as i8 && *(*sym).current <= b'9' as i8 {
            ret = handle_data(sym);
        } else if (*(*sym).current >= b'A' as i8 && *(*sym).current <= b'Z' as i8) || *(*sym).current == b'$' as i8 {
            ret = handle_method(sym, if matches!(post_process, PostProcess::PP_CAST_OPERATOR) { 1 } else { 0 });
        } else {
            ret = FALSE;
        }
    }

    if ret != 0 {
        assert!(!(*sym).result.is_null());
    } else {
        WARN!(b"Failed at %s\n\0".as_ptr(), (*sym).current);
    }

    ret
}

#[no_mangle]
pub unsafe extern "C" fn __unDNameEx(
    buffer: *mut i8,
    mangled: *const i8,
    buflen: i32,
    memget: malloc_func_t,
    memfree: free_func_t,
    unknown: *mut core::ffi::c_void,
    flags: u16,
) -> *mut i8 {
    let mut sym: parsed_symbol = core::mem::zeroed();
    let result: *const i8;

    memset(
        &mut sym as *mut parsed_symbol as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<parsed_symbol>(),
    );
    let mut flags = flags as u32;
    if flags & UNDNAME_NAME_ONLY != 0 {
        flags |= UNDNAME_NO_FUNCTION_RETURNS
            | UNDNAME_NO_ACCESS_SPECIFIERS
            | UNDNAME_NO_MEMBER_TYPE
            | UNDNAME_NO_ALLOCATION_LANGUAGE
            | UNDNAME_NO_COMPLEX_TYPE;
    }

    sym.flags = flags;
    sym.mem_alloc_ptr = memget;
    sym.mem_free_ptr = memfree;
    sym.current = mangled;
    str_array_init(&mut sym.names);
    str_array_init(&mut sym.stack);

    result = if symbol_demangle(&mut sym) != 0 {
        sym.result
    } else {
        mangled
    };

    let buffer = if !buffer.is_null() && buflen != 0 {
        lstrcpynA(buffer, result, buflen);
        buffer
    } else {
        let new_buffer = memget.unwrap_unchecked()(strlen(result) + 1);
        if !new_buffer.is_null() {
            strcpy((new_buffer).cast(), result);
        }
        (new_buffer).cast()
    };

    und_free_all(&mut sym);

    buffer
}

#[no_mangle]
pub unsafe extern "C" fn __unDName(
    buffer: *mut i8,
    mangled: *const i8,
    buflen: i32,
    memget: malloc_func_t,
    memfree: free_func_t,
    flags: u16,
) -> *mut i8 {
    __unDNameEx(buffer, mangled, buflen, memget, memfree, core::ptr::null_mut(), flags)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  Demangle VC++ symbols into C function prototypes
//  *
//  *  Copyright 2000 Jon Griffiths
//  *            2004 Eric Pouech
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
// #include <assert.h>
// #include <stdio.h>
// #include <stdlib.h>
// #include "msvcrt.h"
// #include "winver.h"
// #include "imagehlp.h"
//
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// /* How data types qualifiers are stored:
//  * M (in the following definitions) is defined for
//  * 'A', 'B', 'C' and 'D' as follows
//  *      {<A>}:  ""
//  *      {<B>}:  "const "
//  *      {<C>}:  "volatile "
//  *      {<D>}:  "const volatile "
//  *
//  *      in arguments:
//  *              P<M>x   {<M>}x*
//  *              Q<M>x   {<M>}x* const
//  *              A<M>x   {<M>}x&
//  *      in data fields:
//  *              same as for arguments and also the following
//  *              ?<M>x   {<M>}x
//  *
//  */
//
// #define UNDNAME_NO_COMPLEX_TYPE (0x8000)
//
// struct array
// {
//     unsigned            start;          /* first valid reference in array */
//     unsigned            num;            /* total number of used elts */
//     unsigned            max;
//     unsigned            alloc;
//     char**              elts;
// };
//
// /* Structure holding a parsed symbol */
// struct parsed_symbol
// {
//     unsigned            flags;          /* the UNDNAME_ flags used for demangling */
//     malloc_func_t       mem_alloc_ptr;  /* internal allocator */
//     free_func_t         mem_free_ptr;   /* internal deallocator */
//
//     const char*         current;        /* pointer in input (mangled) string */
//     char*               result;         /* demangled string */
//
//     struct array        names;          /* array of names for back reference */
//     struct array        args;           /* array of arguments for back reference */
//     struct array        stack;          /* stack of parsed strings */
//
//     void*               alloc_list;     /* linked list of allocated blocks */
//     unsigned            avail_in_first; /* number of available bytes in head block */
// };
//
// enum datatype_e
// {
//     DT_NO_LEADING_WS = 0x01,
//     DT_NO_LRSEP_WS = 0x02,
// };
//
// /* Type for parsing mangled types */
// struct datatype_t
// {
//     const char*         left;
//     const char*         right;
//     enum datatype_e     flags;
// };
//
// static BOOL symbol_demangle(struct parsed_symbol* sym);
// static char* get_class_name(struct parsed_symbol* sym);
//
// /******************************************************************
//  *		und_alloc
//  *
//  * Internal allocator. Uses a simple linked list of large blocks
//  * where we use a poor-man allocator. It's fast, and since all
//  * allocation is pool, memory management is easy (esp. freeing).
//  */
// static void*    und_alloc(struct parsed_symbol* sym, unsigned int len)
// {
//     void*       ptr;
//
// #define BLOCK_SIZE      1024
// #define AVAIL_SIZE      (1024 - sizeof(void*))
//
//     if (len > AVAIL_SIZE)
//     {
//         /* allocate a specific block */
//         ptr = sym->mem_alloc_ptr(sizeof(void*) + len);
//         if (!ptr) return NULL;
//         *(void**)ptr = sym->alloc_list;
//         sym->alloc_list = ptr;
//         sym->avail_in_first = 0;
//         ptr = (char*)sym->alloc_list + sizeof(void*);
//     }
//     else
//     {
//         if (len > sym->avail_in_first)
//         {
//             /* add a new block */
//             ptr = sym->mem_alloc_ptr(BLOCK_SIZE);
//             if (!ptr) return NULL;
//             *(void**)ptr = sym->alloc_list;
//             sym->alloc_list = ptr;
//             sym->avail_in_first = AVAIL_SIZE;
//         }
//         /* grab memory from head block */
//         ptr = (char*)sym->alloc_list + BLOCK_SIZE - sym->avail_in_first;
//         sym->avail_in_first -= len;
//     }
//     return ptr;
// #undef BLOCK_SIZE
// #undef AVAIL_SIZE
// }
//
// /******************************************************************
//  *		und_free
//  * Frees all the blocks in the list of large blocks allocated by
//  * und_alloc.
//  */
// static void und_free_all(struct parsed_symbol* sym)
// {
//     void*       next;
//
//     while (sym->alloc_list)
//     {
//         next = *(void**)sym->alloc_list;
//         if(sym->mem_free_ptr) sym->mem_free_ptr(sym->alloc_list);
//         sym->alloc_list = next;
//     }
//     sym->avail_in_first = 0;
// }
//
// /******************************************************************
//  *		str_array_init
//  * Initialises an array of strings
//  */
// static void str_array_init(struct array* a)
// {
//     a->start = a->num = a->max = a->alloc = 0;
//     a->elts = NULL;
// }
//
// /******************************************************************
//  *		str_array_push
//  * Adding a new string to an array
//  */
// static BOOL str_array_push(struct parsed_symbol* sym, const char* ptr, int len,
//                            struct array* a)
// {
//     char**      new;
//
//     assert!(ptr);
//     assert!(a);
//
//     if (!a->alloc)
//     {
//         new = und_alloc(sym, (a->alloc = 32) * sizeof(a->elts[0]));
//         if (!new) return FALSE;
//         a->elts = new;
//     }
//     else if (a->max >= a->alloc)
//     {
//         new = und_alloc(sym, (a->alloc * 2) * sizeof(a->elts[0]));
//         if (!new) return FALSE;
//         memcpy(new, a->elts, a->alloc * sizeof(a->elts[0]));
//         a->alloc *= 2;
//         a->elts = new;
//     }
//     if (len == -1) len = strlen(ptr);
//     a->elts[a->num] = und_alloc(sym, len + 1);
//     assert!(a->elts[a->num]);
//     memcpy(a->elts[a->num], ptr, len);
//     a->elts[a->num][len] = '\0';
//     if (++a->num >= a->max) a->max = a->num;
//     {
//         int i;
//         char c;
//
//         for (i = a->max - 1; i >= 0; i--)
//         {
//             c = '>';
//             if (i < a->start) c = '-';
//             else if (i >= a->num) c = '}';
//             TRACE("%p\t%d%c %s\n", a, i, c, debugstr_a(a->elts[i]));
//         }
//     }
//
//     return TRUE;
// }
//
// /******************************************************************
//  *		str_array_get_ref
//  * Extracts a reference from an existing array (doing proper type
//  * checking)
//  */
// static char* str_array_get_ref(struct array* cref, unsigned idx)
// {
//     assert!(cref);
//     if (cref->start + idx >= cref->max)
//     {
//         WARN("Out of bounds: %p %d + %d >= %d\n",
//               cref, cref->start, idx, cref->max);
//         return NULL;
//     }
//     TRACE("Returning %p[%d] => %s\n",
//           cref, idx, debugstr_a(cref->elts[cref->start + idx]));
//     return cref->elts[cref->start + idx];
// }
//
// /******************************************************************
//  *		str_printf
//  * Helper for printf type of command (only %s and %c are implemented)
//  * while dynamically allocating the buffer
//  */
// static char*  str_printf(struct parsed_symbol* sym, const char* format, ...)
// {
//     va_list      args;
//     unsigned int len = 1, i, sz;
//     char*        tmp;
//     char*        p;
//     char*        t;
//
//     va_start(args, format);
//     for (i = 0; format[i]; i++)
//     {
//         if (format[i] == '%')
//         {
//             switch (format[++i])
//             {
//             case 's': t = va_arg(args, char*); if (t) len += strlen(t); break;
//             case 'c': (void)va_arg(args, int); len++; break;
//             default: i--; /* fall through */
//             case '%': len++; break;
//             }
//         }
//         else len++;
//     }
//     va_end(args);
//     if (!(tmp = und_alloc(sym, len))) return NULL;
//     va_start(args, format);
//     for (p = tmp, i = 0; format[i]; i++)
//     {
//         if (format[i] == '%')
//         {
//             switch (format[++i])
//             {
//             case 's':
//                 t = va_arg(args, char*);
//                 if (t)
//                 {
//                     sz = strlen(t);
//                     memcpy(p, t, sz);
//                     p += sz;
//                 }
//                 break;
//             case 'c':
//                 *p++ = (char)va_arg(args, int);
//                 break;
//             default: i--; /* fall through */
//             case '%': *p++ = '%'; break;
//             }
//         }
//         else *p++ = format[i];
//     }
//     va_end(args);
//     *p = '\0';
//     return tmp;
// }
//
// enum datatype_flags
// {
//     IN_ARGS = 0x01,
//     WS_AFTER_QUAL_IF = 0x02,
// };
//
// /* forward declaration */
// static BOOL demangle_datatype(struct parsed_symbol* sym, struct datatype_t* ct,
//                               enum datatype_flags flags);
//
// static const char* get_number(struct parsed_symbol* sym)
// {
//     char*       ptr;
//     BOOL        sgn = FALSE;
//
//     if (*sym->current == '?')
//     {
//         sgn = TRUE;
//         sym->current++;
//     }
//     if (*sym->current >= '0' && *sym->current <= '8')
//     {
//         ptr = und_alloc(sym, 3);
//         if (sgn) ptr[0] = '-';
//         ptr[sgn ? 1 : 0] = *sym->current + 1;
//         ptr[sgn ? 2 : 1] = '\0';
//         sym->current++;
//     }
//     else if (*sym->current == '9')
//     {
//         ptr = und_alloc(sym, 4);
//         if (sgn) ptr[0] = '-';
//         ptr[sgn ? 1 : 0] = '1';
//         ptr[sgn ? 2 : 1] = '0';
//         ptr[sgn ? 3 : 2] = '\0';
//         sym->current++;
//     }
//     else if (*sym->current >= 'A' && *sym->current <= 'P')
//     {
//         int ret = 0;
//
//         while (*sym->current >= 'A' && *sym->current <= 'P')
//         {
//             ret *= 16;
//             ret += *sym->current++ - 'A';
//         }
//         if (*sym->current != '@') return NULL;
//
//         ptr = und_alloc(sym, 17);
//         sprintf(ptr, "%s%u", sgn ? "-" : "", ret);
//         sym->current++;
//     }
//     else return NULL;
//     return ptr;
// }
//
// /******************************************************************
//  *		get_args
//  * Parses a list of function/method arguments, creates a string corresponding
//  * to the arguments' list.
//  */
// static char* get_args(struct parsed_symbol* sym, BOOL z_term,
//                       char open_char, char close_char)
//
// {
//     struct datatype_t   ct;
//     struct array        arg_collect;
//     char*               args_str = NULL;
//     char*               last;
//     unsigned int        i;
//     const char *p;
//
//     if (z_term && sym->current[0] == 'X' && sym->current[1] == 'Z')
//     {
//         sym->current += 2;
//         return str_printf(sym, "%cvoid%c", open_char, close_char);
//     }
//
//     str_array_init(&arg_collect);
//
//     /* Now come the function arguments */
//     while (*sym->current)
//     {
//         p = sym->current;
//
//         /* Decode each data type and append it to the argument list */
//         if (*sym->current == '@')
//         {
//             sym->current++;
//             break;
//         }
//         if (z_term && sym->current[0] == 'Z')
//         {
//             sym->current++;
//             if (!str_array_push(sym, "...", -1, &arg_collect))
//                 return NULL;
//             break;
//         }
//         /* Handle empty list in variadic template */
//         if (!z_term && sym->current[0] == '$' && sym->current[1] == '$' && sym->current[2] == 'V')
//         {
//             sym->current += 3;
//             continue;
//         }
//         if (!demangle_datatype(sym, &ct, IN_ARGS))
//             return NULL;
//         if (!str_array_push(sym, str_printf(sym, "%s%s", ct.left, ct.right), -1,
//                             &arg_collect))
//             return NULL;
//         if (z_term && sym->current - p > 1 && sym->args.num < 20)
//         {
//             if (!str_array_push(sym, ct.left ? ct.left : "", -1, &sym->args) ||
//                     !str_array_push(sym, ct.right ? ct.right : "", -1, &sym->args))
//                 return NULL;
//         }
//     }
//     /* Functions are always terminated by 'Z'. If we made it this far and
//      * don't find it, we have incorrectly identified a data type.
//      */
//     if (z_term && *sym->current++ != 'Z') return NULL;
//
//     if (!arg_collect.num) return str_printf(sym, "%c%c", open_char, close_char);
//     for (i = 1; i < arg_collect.num; i++)
//     {
//         args_str = str_printf(sym, "%s,%s", args_str, arg_collect.elts[i]);
//     }
//
//     last = args_str ? args_str : arg_collect.elts[0];
//     if (close_char == '>' && last[strlen(last) - 1] == '>')
//         args_str = str_printf(sym, "%c%s%s %c",
//                               open_char, arg_collect.elts[0], args_str, close_char);
//     else
//         args_str = str_printf(sym, "%c%s%s%c",
//                               open_char, arg_collect.elts[0], args_str, close_char);
//
//     return args_str;
// }
//
// static void append_extended_qualifier(struct parsed_symbol *sym, const char **where,
//                                       const char *str, BOOL is_ms_keyword)
// {
//     if (!is_ms_keyword || !(sym->flags & UNDNAME_NO_MS_KEYWORDS))
//     {
//         if (is_ms_keyword && (sym->flags & UNDNAME_NO_LEADING_UNDERSCORES))
//             str += 2;
//         *where = *where ? str_printf(sym, "%s%s%s%s", *where, is_ms_keyword ? " " : "", str, is_ms_keyword ? "" : " ") :
//             str_printf(sym, "%s%s", str, is_ms_keyword ? "" : " ");
//     }
// }
//
// static void get_extended_qualifier(struct parsed_symbol *sym, struct datatype_t *xdt)
// {
//     unsigned fl = 0;
//     xdt->left = xdt->right = NULL;
//     xdt->flags = 0;
//     for (;;)
//     {
//         switch (*sym->current)
//         {
//         case 'E': append_extended_qualifier(sym, &xdt->right, "__ptr64", TRUE);     fl |= 2; break;
//         case 'F': append_extended_qualifier(sym, &xdt->left,  "__unaligned", TRUE); fl |= 2; break;
// #ifdef _UCRT
//         case 'G': append_extended_qualifier(sym, &xdt->right, "&", FALSE);          fl |= 1; break;
//         case 'H': append_extended_qualifier(sym, &xdt->right, "&&", FALSE);         fl |= 1; break;
// #endif
//         case 'I': append_extended_qualifier(sym, &xdt->right, "__restrict", TRUE);  fl |= 2; break;
//         default: if (fl == 1 || (fl == 3 && (sym->flags & UNDNAME_NO_MS_KEYWORDS))) xdt->flags = DT_NO_LRSEP_WS; return;
//         }
//         sym->current++;
//     }
// }
//
// /******************************************************************
//  *		get_qualifier
//  * Parses the type qualifier. Always returns static strings.
//  */
// static BOOL get_qualifier(struct parsed_symbol *sym, struct datatype_t *xdt, const char** pclass)
// {
//     char ch;
//     const char* qualif;
//
//     get_extended_qualifier(sym, xdt);
//     switch (ch = *sym->current++)
//     {
//     case 'A': qualif = NULL; break;
//     case 'B': qualif = "const"; break;
//     case 'C': qualif = "volatile"; break;
//     case 'D': qualif = "const volatile"; break;
//     case 'Q': qualif = NULL; break;
//     case 'R': qualif = "const"; break;
//     case 'S': qualif = "volatile"; break;
//     case 'T': qualif = "const volatile"; break;
//     default: return FALSE;
//     }
//     if (qualif)
//     {
//         xdt->flags &= ~DT_NO_LRSEP_WS;
//         xdt->left = xdt->left ? str_printf(sym, "%s %s", qualif, xdt->left) : qualif;
//     }
//     if (ch >= 'Q' && ch <= 'T') /* pointer to member, fetch class */
//     {
//         const char* class = get_class_name(sym);
//         if (!class) return FALSE;
//         if (!pclass)
//         {
//             FIXME("Got pointer to class %s member without storage\n", class);
//             return FALSE;
//         }
//         *pclass = class;
//     }
//     else if (pclass) *pclass = NULL;
//     return TRUE;
// }
//
// static BOOL get_function_qualifier(struct parsed_symbol *sym, const char** qualif)
// {
//     struct datatype_t   xdt;
//
//     if (!get_qualifier(sym, &xdt, NULL)) return FALSE;
//     *qualif = (xdt.left || xdt.right) ?
//         str_printf(sym, "%s%s%s", xdt.left, (xdt.flags & DT_NO_LRSEP_WS) ? "" : " ", xdt.right) : NULL;
//     return TRUE;
// }
//
// static BOOL get_qualified_type(struct datatype_t *ct, struct parsed_symbol* sym,
//                                char qualif, enum datatype_flags flags)
// {
//     struct datatype_t xdt1;
//     struct datatype_t xdt2;
//     const char* ref;
//     const char* str_qualif;
//     const char* class;
//
//     get_extended_qualifier(sym, &xdt1);
//
//     switch (qualif)
//     {
//     case 'A': ref = " &";  str_qualif = NULL;              break;
//     case 'B': ref = " &";  str_qualif = " volatile";       break;
//     case 'P': ref = " *";  str_qualif = NULL;              break;
//     case 'Q': ref = " *";  str_qualif = " const";          break;
//     case 'R': ref = " *";  str_qualif = " volatile";       break;
//     case 'S': ref = " *";  str_qualif = " const volatile"; break;
//     case '?': ref = NULL;  str_qualif = NULL;              break;
//     case '$': ref = " &&"; str_qualif = NULL;              break;
//     default: return FALSE;
//     }
//     ct->right = NULL;
//     ct->flags = 0;
//
//     /* parse managed handle information */
//     if (sym->current[0] == '$' && sym->current[1] == 'A')
//     {
//         sym->current += 2;
//
//         switch (qualif)
//         {
//         case 'A':
//         case 'B':
//             ref = " %";
//             break;
//         case 'P':
//         case 'Q':
//         case 'R':
//         case 'S':
//             ref = " ^";
//             break;
//         default:
//             return FALSE;
//         }
//     }
//
//     if (get_qualifier(sym, &xdt2, &class))
//     {
//         unsigned            mark = sym->stack.num;
//         struct datatype_t   sub_ct;
//
//         if (ref || str_qualif || xdt1.left || xdt1.right)
//         {
//             if (class)
//                 ct->left = str_printf(sym, "%s%s%s%s::%s%s%s",
//                                       xdt1.left ? " " : NULL, xdt1.left,
//                                       class ? " " : NULL, class, ref ? ref + 1 : NULL,
//                                       xdt1.right ? " " : NULL, xdt1.right, str_qualif);
//             else
//                 ct->left = str_printf(sym, "%s%s%s%s%s%s",
//                                       xdt1.left ? " " : NULL, xdt1.left, ref,
//                                       xdt1.right ? " " : NULL, xdt1.right, str_qualif);
//         }
//         else
//             ct->left = NULL;
//         /* multidimensional arrays */
//         if (*sym->current == 'Y')
//         {
//             const char* n1;
//             int num;
//
//             sym->current++;
//             if (!(n1 = get_number(sym))) return FALSE;
//             num = atoi(n1);
//
//             ct->left = str_printf(sym, " (%s%s", xdt2.left, ct->left && !xdt2.left ? ct->left + 1 : ct->left);
//             ct->right = ")";
//             xdt2.left = NULL;
//
//             while (num--)
//                 ct->right = str_printf(sym, "%s[%s]", ct->right, get_number(sym));
//         }
//
//         /* Recurse to get the referred-to type */
//         if (!demangle_datatype(sym, &sub_ct, 0))
//             return FALSE;
//         if (sub_ct.flags & DT_NO_LEADING_WS)
//             ct->left++;
//         ct->left = str_printf(sym, "%s%s%s%s%s", sub_ct.left, xdt2.left ? " " : NULL,
//                               xdt2.left, ct->left,
//                               ((xdt2.left || str_qualif) && (flags & WS_AFTER_QUAL_IF)) ? " " : NULL);
//         if (sub_ct.right) ct->right = str_printf(sym, "%s%s", ct->right, sub_ct.right);
//         sym->stack.num = mark;
//     }
//     else if (ref || str_qualif || xdt1.left || xdt1.right)
//         ct->left = str_printf(sym, "%s%s%s%s%s%s",
//                               xdt1.left ? " " : NULL, xdt1.left, ref,
//                               xdt1.right ? " " : NULL, xdt1.right, str_qualif);
//     else
//         ct->left = NULL;
//     return TRUE;
// }
//
// /******************************************************************
//  *             get_literal_string
//  * Gets the literal name from the current position in the mangled
//  * symbol to the first '@' character. It pushes the parsed name to
//  * the symbol names stack and returns a pointer to it or NULL in
//  * case of an error.
//  */
// static char* get_literal_string(struct parsed_symbol* sym)
// {
//     const char *ptr = sym->current;
//
//     do {
//         if (!((*sym->current >= 'A' && *sym->current <= 'Z') ||
//               (*sym->current >= 'a' && *sym->current <= 'z') ||
//               (*sym->current >= '0' && *sym->current <= '9') ||
//               *sym->current == '_' || *sym->current == '$' ||
//               *sym->current == '<' || *sym->current == '>')) {
//             TRACE("Failed at '%c' in %s\n", *sym->current, debugstr_a(ptr));
//             return NULL;
//         }
//     } while (*++sym->current != '@');
//     sym->current++;
//     if (!str_array_push(sym, ptr, sym->current - 1 - ptr, &sym->names))
//         return NULL;
//
//     return str_array_get_ref(&sym->names, sym->names.num - sym->names.start - 1);
// }
//
// /******************************************************************
//  *		get_template_name
//  * Parses a name with a template argument list and returns it as
//  * a string.
//  * In a template argument list the back reference to the names
//  * table is separately created. '0' points to the class component
//  * name with the template arguments.  We use the same stack array
//  * to hold the names but save/restore the stack state before/after
//  * parsing the template argument list.
//  */
// static char* get_template_name(struct parsed_symbol* sym)
// {
//     char *name, *args;
//     unsigned num_mark = sym->names.num;
//     unsigned start_mark = sym->names.start;
//     unsigned stack_mark = sym->stack.num;
//     unsigned args_mark = sym->args.num;
//
//     sym->names.start = sym->names.num;
//     if (!(name = get_literal_string(sym))) {
//         sym->names.start = start_mark;
//         return FALSE;
//     }
//     args = get_args(sym, FALSE, '<', '>');
//     if (args != NULL)
//         name = str_printf(sym, "%s%s", name, args);
//     sym->names.num = num_mark;
//     sym->names.start = start_mark;
//     sym->stack.num = stack_mark;
//     sym->args.num = args_mark;
//     return name;
// }
//
// /******************************************************************
//  *		get_class
//  * Parses class as a list of parent-classes, terminated by '@' and stores the
//  * result in 'a' array. Each parent-classes, as well as the inner element
//  * (either field/method name or class name), are represented in the mangled
//  * name by a literal name ([a-zA-Z0-9_]+ terminated by '@') or a back reference
//  * ([0-9]) or a name with template arguments ('?$' literal name followed by the
//  * template argument list). The class name components appear in the reverse
//  * order in the mangled name, e.g aaa@bbb@ccc@@ will be demangled to
//  * ccc::bbb::aaa
//  * For each of these class name components a string will be allocated in the
//  * array.
//  */
// static BOOL get_class(struct parsed_symbol* sym)
// {
//     const char* name = NULL;
//
//     while (*sym->current != '@')
//     {
//         switch (*sym->current)
//         {
//         case '\0': return FALSE;
//
//         case '0': case '1': case '2': case '3':
//         case '4': case '5': case '6': case '7':
//         case '8': case '9':
//             name = str_array_get_ref(&sym->names, *sym->current++ - '0');
//             break;
//         case '?':
//             switch (*++sym->current)
//             {
//             case '$':
//                 sym->current++;
//                 if ((name = get_template_name(sym)) &&
//                     !str_array_push(sym, name, -1, &sym->names))
//                     return FALSE;
//                 break;
//             case '?':
//                 {
//                     struct array stack = sym->stack;
//                     unsigned int start = sym->names.start;
//                     unsigned int num = sym->names.num;
//
//                     str_array_init( &sym->stack );
//                     if (symbol_demangle( sym )) name = str_printf( sym, "`%s'", sym->result );
//                     sym->names.start = start;
//                     sym->names.num = num;
//                     sym->stack = stack;
//                 }
//                 break;
//             default:
//                 if (!(name = get_number( sym ))) return FALSE;
//                 name = str_printf( sym, "`%s'", name );
//                 break;
//             }
//             break;
//         default:
//             name = get_literal_string(sym);
//             break;
//         }
//         if (!name || !str_array_push(sym, name, -1, &sym->stack))
//             return FALSE;
//     }
//     sym->current++;
//     return TRUE;
// }
//
// /******************************************************************
//  *		get_class_string
//  * From an array collected by get_class in sym->stack, constructs the
//  * corresponding (allocated) string
//  */
// static char* get_class_string(struct parsed_symbol* sym, int start)
// {
//     int          i;
//     unsigned int len, sz;
//     char*        ret;
//     struct array *a = &sym->stack;
//
//     for (len = 0, i = start; i < a->num; i++)
//     {
//         assert!(a->elts[i]);
//         len += 2 + strlen(a->elts[i]);
//     }
//     if (!(ret = und_alloc(sym, len - 1))) return NULL;
//     for (len = 0, i = a->num - 1; i >= start; i--)
//     {
//         sz = strlen(a->elts[i]);
//         memcpy(ret + len, a->elts[i], sz);
//         len += sz;
//         if (i > start)
//         {
//             ret[len++] = ':';
//             ret[len++] = ':';
//         }
//     }
//     ret[len] = '\0';
//     return ret;
// }
//
// /******************************************************************
//  *            get_class_name
//  * Wrapper around get_class and get_class_string.
//  */
// static char* get_class_name(struct parsed_symbol* sym)
// {
//     unsigned    mark = sym->stack.num;
//     char*       s = NULL;
//
//     if (get_class(sym))
//         s = get_class_string(sym, mark);
//     sym->stack.num = mark;
//     return s;
// }
//
// /******************************************************************
//  *		get_calling_convention
//  * Returns a static string corresponding to the calling convention described
//  * by char 'ch'. Sets export to TRUE iff the calling convention is exported.
//  */
// static BOOL get_calling_convention(char ch, const char** call_conv,
//                                    const char** exported, unsigned flags)
// {
//     *call_conv = *exported = NULL;
//
//     if (!(flags & (UNDNAME_NO_MS_KEYWORDS | UNDNAME_NO_ALLOCATION_LANGUAGE)))
//     {
//         if (flags & UNDNAME_NO_LEADING_UNDERSCORES)
//         {
//             if (((ch - 'A') % 2) == 1) *exported = "dll_export ";
//             switch (ch)
//             {
//             case 'A': case 'B': *call_conv = "cdecl"; break;
//             case 'C': case 'D': *call_conv = "__stdcall"; break;
//             case 'E': case 'F': *call_conv = "thiscall"; break;
//             case 'G': case 'H': *call_conv = "stdcall"; break;
//             case 'I': case 'J': *call_conv = "fastcall"; break;
//             case 'K': case 'L': break;
//             case 'M': *call_conv = "clrcall"; break;
//             default: ERR("Unknown calling convention %c\n", ch); return FALSE;
//             }
//         }
//         else
//         {
//             if (((ch - 'A') % 2) == 1) *exported = "__dll_export ";
//             switch (ch)
//             {
//             case 'A': case 'B': *call_conv = "__cdecl"; break;
//             case 'C': case 'D': *call_conv = "__pascal"; break;
//             case 'E': case 'F': *call_conv = "__thiscall"; break;
//             case 'G': case 'H': *call_conv = "__stdcall"; break;
//             case 'I': case 'J': *call_conv = "__fastcall"; break;
//             case 'K': case 'L': break;
//             case 'M': *call_conv = "__clrcall"; break;
//             default: ERR("Unknown calling convention %c\n", ch); return FALSE;
//             }
//         }
//     }
//     return TRUE;
// }
//
// /*******************************************************************
//  *         get_simple_type
//  * Return a string containing an allocated string for a simple data type
//  */
// static const char* get_simple_type(char c)
// {
//     const char* type_string;
//
//     switch (c)
//     {
//     case 'C': type_string = "signed char"; break;
//     case 'D': type_string = "char"; break;
//     case 'E': type_string = "unsigned char"; break;
//     case 'F': type_string = "short"; break;
//     case 'G': type_string = "unsigned short"; break;
//     case 'H': type_string = "int"; break;
//     case 'I': type_string = "unsigned int"; break;
//     case 'J': type_string = "long"; break;
//     case 'K': type_string = "unsigned long"; break;
//     case 'M': type_string = "float"; break;
//     case 'N': type_string = "double"; break;
//     case 'O': type_string = "long double"; break;
//     case 'X': type_string = "void"; break;
// /*  case 'Z': (...) variadic function arguments. Handled in get_args() */
//     default:  type_string = NULL; break;
//     }
//     return type_string;
// }
//
// /*******************************************************************
//  *         get_extended_type
//  * Return a string containing an allocated string for a simple data type
//  */
// static const char* get_extended_type(char c)
// {
//     const char* type_string;
//
//     switch (c)
//     {
//     case 'D': type_string = "__int8"; break;
//     case 'E': type_string = "unsigned __int8"; break;
//     case 'F': type_string = "__int16"; break;
//     case 'G': type_string = "unsigned __int16"; break;
//     case 'H': type_string = "__int32"; break;
//     case 'I': type_string = "unsigned __int32"; break;
//     case 'J': type_string = "__int64"; break;
//     case 'K': type_string = "unsigned __int64"; break;
//     case 'L': type_string = "__int128"; break;
//     case 'M': type_string = "unsigned __int128"; break;
//     case 'N': type_string = "bool"; break;
//     case 'Q': type_string = "char8_t"; break;
//     case 'S': type_string = "char16_t"; break;
//     case 'U': type_string = "char32_t"; break;
//     case 'W': type_string = "wchar_t"; break;
//     default:  type_string = NULL; break;
//     }
//     return type_string;
// }
//
// struct function_signature
// {
//     const char*             call_conv;
//     const char*             exported;
//     struct datatype_t       return_ct;
//     const char*             arguments;
// };
//
// static BOOL get_function_signature(struct parsed_symbol* sym, struct function_signature* fs)
// {
//     unsigned mark = sym->stack.num;
//
//     if (!get_calling_convention(*sym->current++,
//                                 &fs->call_conv, &fs->exported,
//                                 sym->flags & ~UNDNAME_NO_ALLOCATION_LANGUAGE) ||
//         !demangle_datatype(sym, &fs->return_ct, 0))
//         return FALSE;
//
//     if (!(fs->arguments = get_args(sym, TRUE, '(', ')')))
//         return FALSE;
//     sym->stack.num = mark;
//
//     return TRUE;
// }
//
// /*******************************************************************
//  *         demangle_datatype
//  *
//  * Attempt to demangle a C++ data type, which may be datatype.
//  * a datatype type is made up of a number of simple types. e.g:
//  * char** = (pointer to (pointer to (char)))
//  */
// static BOOL demangle_datatype(struct parsed_symbol* sym, struct datatype_t* ct,
//                               enum datatype_flags flags)
// {
//     char                dt;
//
//     assert!(ct);
//     ct->left = ct->right = NULL;
//     ct->flags = 0;
//
//     switch (dt = *sym->current++)
//     {
//     case '_':
//         /* MS type: __int8,__int16 etc */
//         ct->left = get_extended_type(*sym->current++);
//         break;
//     case 'C': case 'D': case 'E': case 'F': case 'G':
//     case 'H': case 'I': case 'J': case 'K': case 'M':
//     case 'N': case 'O': case 'X': case 'Z':
//         /* Simple data types */
//         ct->left = get_simple_type(dt);
//         break;
//     case 'T': /* union */
//     case 'U': /* struct */
//     case 'V': /* class */
//     case 'Y': /* cointerface */
//         /* Class/struct/union/cointerface */
//         {
//             const char* struct_name = NULL;
//             const char* type_name = NULL;
//
//             if (!(struct_name = get_class_name(sym)))
//                 goto done;
//             if (!(sym->flags & UNDNAME_NO_COMPLEX_TYPE))
//             {
//                 switch (dt)
//                 {
//                 case 'T': type_name = "union ";  break;
//                 case 'U': type_name = "struct "; break;
//                 case 'V': type_name = "class ";  break;
//                 case 'Y': type_name = "cointerface "; break;
//                 }
//             }
//             ct->left = str_printf(sym, "%s%s", type_name, struct_name);
//         }
//         break;
//     case '?':
//         /* not all the time is seems */
//         if (flags & IN_ARGS)
//         {
//             const char*   ptr;
//             if (!(ptr = get_number(sym))) goto done;
//             ct->left = str_printf(sym, "`template-parameter-%s'", ptr);
//         }
//         else
//         {
//             if (!get_qualified_type(ct, sym, '?', flags)) goto done;
//         }
//         break;
//     case 'A': /* reference */
//     case 'B': /* volatile reference */
//         if (!get_qualified_type(ct, sym, dt, flags)) goto done;
//         break;
//     case 'P': /* Pointer */
//     case 'Q': /* const pointer */
//     case 'R': /* volatile pointer */
//     case 'S': /* const volatile pointer */
//         if (!(flags & IN_ARGS)) dt = 'P';
//         if (isdigit(*sym->current))
//         {
//             const char *ptr_qualif;
//             switch (dt)
//             {
//             default:
//             case 'P': ptr_qualif = NULL; break;
//             case 'Q': ptr_qualif = "const"; break;
//             case 'R': ptr_qualif = "volatile"; break;
//             case 'S': ptr_qualif = "const volatile"; break;
//             }
//             /* FIXME:
//              *   P6 = Function pointer
//              *   P8 = Member function pointer
//              *   others who knows.. */
//             if (*sym->current == '8')
//             {
//                 struct function_signature       fs;
//                 const char*                     class;
//                 const char*                     function_qualifier;
//
//                 sym->current++;
//
//                 if (!(class = get_class_name(sym)))
//                     goto done;
//                 if (!get_function_qualifier(sym, &function_qualifier))
//                     goto done;
//                 if (!get_function_signature(sym, &fs))
//                     goto done;
//
//                 ct->left  = str_printf(sym, "%s%s (%s %s::*%s",
//                                        fs.return_ct.left, fs.return_ct.right, fs.call_conv, class, ptr_qualif);
//                 ct->right = str_printf(sym, ")%s%s", fs.arguments, function_qualifier);
//             }
//             else if (*sym->current == '6')
//             {
//                 struct function_signature       fs;
//
//                 sym->current++;
//
//                 if (!get_function_signature(sym, &fs))
//                      goto done;
//
//                 ct->left  = str_printf(sym, "%s%s (%s*%s",
//                                        fs.return_ct.left, fs.return_ct.right, fs.call_conv, ptr_qualif);
//                 ct->flags = DT_NO_LEADING_WS;
//                 ct->right = str_printf(sym, ")%s", fs.arguments);
//             }
//             else goto done;
// 	}
// 	else if (!get_qualified_type(ct, sym, dt, flags)) goto done;
//         break;
//     case 'W':
//         if (*sym->current == '4')
//         {
//             char*               enum_name;
//             sym->current++;
//             if (!(enum_name = get_class_name(sym)))
//                 goto done;
//             if (sym->flags & UNDNAME_NO_COMPLEX_TYPE)
//                 ct->left = enum_name;
//             else
//                 ct->left = str_printf(sym, "enum %s", enum_name);
//         }
//         else goto done;
//         break;
//     case '0': case '1': case '2': case '3': case '4':
//     case '5': case '6': case '7': case '8': case '9':
//         /* Referring back to previously parsed type */
//         /* left and right are pushed as two separate strings */
//         ct->left = str_array_get_ref(&sym->args, (dt - '0') * 2);
//         ct->right = str_array_get_ref(&sym->args, (dt - '0') * 2 + 1);
//         if (!ct->left) goto done;
//         break;
//     case '$':
//         switch (*sym->current++)
//         {
//         case '0':
//             if (!(ct->left = get_number(sym))) goto done;
//             break;
//         case 'D':
//             {
//                 const char*   ptr;
//                 if (!(ptr = get_number(sym))) goto done;
//                 ct->left = str_printf(sym, "`template-parameter%s'", ptr);
//             }
//             break;
//         case 'F':
//             {
//                 const char*   p1;
//                 const char*   p2;
//                 if (!(p1 = get_number(sym))) goto done;
//                 if (!(p2 = get_number(sym))) goto done;
//                 ct->left = str_printf(sym, "{%s,%s}", p1, p2);
//             }
//             break;
//         case 'G':
//             {
//                 const char*   p1;
//                 const char*   p2;
//                 const char*   p3;
//                 if (!(p1 = get_number(sym))) goto done;
//                 if (!(p2 = get_number(sym))) goto done;
//                 if (!(p3 = get_number(sym))) goto done;
//                 ct->left = str_printf(sym, "{%s,%s,%s}", p1, p2, p3);
//             }
//             break;
//         case 'Q':
//             {
//                 const char*   ptr;
//                 if (!(ptr = get_number(sym))) goto done;
//                 ct->left = str_printf(sym, "`non-type-template-parameter%s'", ptr);
//             }
//             break;
//         case '$':
//             if (*sym->current == 'A')
//             {
//                 sym->current++;
//                 if (*sym->current == '6')
//                 {
//                     struct function_signature fs;
//
//                     sym->current++;
//
//                     if (!get_function_signature(sym, &fs))
//                         goto done;
//                     ct->left = str_printf(sym, "%s%s %s%s",
//                                           fs.return_ct.left, fs.return_ct.right, fs.call_conv, fs.arguments);
//                 }
//             }
//             else if (*sym->current == 'B')
//             {
//                 unsigned            mark = sym->stack.num;
//                 struct datatype_t   sub_ct;
//                 const char*         arr = NULL;
//                 sym->current++;
//
//                 /* multidimensional arrays */
//                 if (*sym->current == 'Y')
//                 {
//                     const char* n1;
//                     int num;
//
//                     sym->current++;
//                     if (!(n1 = get_number(sym))) goto done;
//                     num = atoi(n1);
//
//                     while (num--)
//                         arr = str_printf(sym, "%s[%s]", arr, get_number(sym));
//                 }
//
//                 if (!demangle_datatype(sym, &sub_ct, 0)) goto done;
//
//                 if (arr)
//                     ct->left = str_printf(sym, "%s %s", sub_ct.left, arr);
//                 else
//                     ct->left = sub_ct.left;
//                 ct->right = sub_ct.right;
//                 sym->stack.num = mark;
//             }
//             else if (*sym->current == 'C')
//             {
//                 struct datatype_t xdt;
//
//                 sym->current++;
//                 if (!get_qualifier(sym, &xdt, NULL)) goto done;
//                 if (!demangle_datatype(sym, ct, flags)) goto done;
//                 ct->left = str_printf(sym, "%s %s", ct->left, xdt.left);
//             }
//             else if (*sym->current == 'Q')
//             {
//                 sym->current++;
//                 if (!get_qualified_type(ct, sym, '$', flags)) goto done;
//             }
//             else if (*sym->current == 'T')
//             {
//                 sym->current++;
//                 ct->left = str_printf(sym, "std::nullptr_t");
//             }
//             break;
//         }
//         break;
//     default :
//         ERR("Unknown type %c\n", dt);
//         break;
//     }
// done:
//
//     return ct->left != NULL;
// }
//
// /******************************************************************
//  *		handle_data
//  * Does the final parsing and handling for a variable or a field in
//  * a class.
//  */
// static BOOL handle_data(struct parsed_symbol* sym)
// {
//     const char*         access = NULL;
//     const char*         member_type = NULL;
//     struct datatype_t   xdt = {NULL};
//     struct datatype_t   ct;
//     char*               name = NULL;
//     BOOL                ret = FALSE;
//
//     /* 0 private static
//      * 1 protected static
//      * 2 public static
//      * 3 private non-static
//      * 4 protected non-static
//      * 5 public non-static
//      * 6 ?? static
//      * 7 ?? static
//      */
//
//     if (!(sym->flags & UNDNAME_NO_ACCESS_SPECIFIERS))
//     {
//         /* we only print the access for static members */
//         switch (*sym->current)
//         {
//         case '0': access = "private: "; break;
//         case '1': access = "protected: "; break;
//         case '2': access = "public: "; break;
//         }
//     }
//
//     if (!(sym->flags & UNDNAME_NO_MEMBER_TYPE))
//     {
//         if (*sym->current >= '0' && *sym->current <= '2')
//             member_type = "static ";
//     }
//
//     name = get_class_string(sym, 0);
//
//     switch (*sym->current++)
//     {
//     case '0': case '1': case '2':
//     case '3': case '4': case '5':
//         {
//             unsigned mark = sym->stack.num;
//             const char* class;
//
//             if (!demangle_datatype(sym, &ct, 0)) goto done;
//             if (!get_qualifier(sym, &xdt, &class)) goto done; /* class doesn't seem to be displayed */
//             if (xdt.left && xdt.right) xdt.left = str_printf(sym, "%s %s", xdt.left, xdt.right);
//             else if (!xdt.left) xdt.left = xdt.right;
//             sym->stack.num = mark;
//         }
//         break;
//     case '6' : /* compiler generated static */
//     case '7' : /* compiler generated static */
//         ct.left = ct.right = NULL;
//         if (!get_qualifier(sym, &xdt, NULL)) goto done;
//         if (*sym->current != '@')
//         {
//             char*       cls = NULL;
//
//             if (!(cls = get_class_name(sym)))
//                 goto done;
//             ct.right = str_printf(sym, "{for `%s'}", cls);
//         }
//         break;
//     case '8':
//     case '9':
//         xdt.left = ct.left = ct.right = NULL;
//         break;
//     default: goto done;
//     }
//     if (sym->flags & UNDNAME_NAME_ONLY) ct.left = ct.right = xdt.left = NULL;
//
//     sym->result = str_printf(sym, "%s%s%s%s%s%s%s%s", access,
//                              member_type, ct.left,
//                              xdt.left && ct.left ? " " : NULL, xdt.left,
//                              xdt.left || ct.left ? " " : NULL, name, ct.right);
//     ret = TRUE;
// done:
//     return ret;
// }
//
// /******************************************************************
//  *		handle_method
//  * Does the final parsing and handling for a function or a method in
//  * a class.
//  */
// static BOOL handle_method(struct parsed_symbol* sym, BOOL cast_op)
// {
//     char                accmem;
//     const char*         access = NULL;
//     int                 access_id = -1;
//     const char*         member_type = NULL;
//     struct datatype_t   ct_ret;
//     const char*         call_conv;
//     const char*         function_qualifier = NULL;
//     const char*         exported;
//     const char*         args_str = NULL;
//     const char*         name = NULL;
//     BOOL                ret = FALSE, has_args = TRUE, has_ret = TRUE;
//     unsigned            mark;
//
//     /* FIXME: why 2 possible letters for each option?
//      * 'A' private:
//      * 'B' private:
//      * 'C' private: static
//      * 'D' private: static
//      * 'E' private: virtual
//      * 'F' private: virtual
//      * 'G' private: thunk
//      * 'H' private: thunk
//      * 'I' protected:
//      * 'J' protected:
//      * 'K' protected: static
//      * 'L' protected: static
//      * 'M' protected: virtual
//      * 'N' protected: virtual
//      * 'O' protected: thunk
//      * 'P' protected: thunk
//      * 'Q' public:
//      * 'R' public:
//      * 'S' public: static
//      * 'T' public: static
//      * 'U' public: virtual
//      * 'V' public: virtual
//      * 'W' public: thunk
//      * 'X' public: thunk
//      * 'Y'
//      * 'Z'
//      * "$0" private: thunk vtordisp
//      * "$1" private: thunk vtordisp
//      * "$2" protected: thunk vtordisp
//      * "$3" protected: thunk vtordisp
//      * "$4" public: thunk vtordisp
//      * "$5" public: thunk vtordisp
//      * "$B" vcall thunk
//      * "$R" thunk vtordispex
//      */
//     accmem = *sym->current++;
//     if (accmem == '$')
//     {
//         if (*sym->current >= '0' && *sym->current <= '5')
//             access_id = (*sym->current - '0') / 2;
//         else if (*sym->current == 'R')
//             access_id = (sym->current[1] - '0') / 2;
//         else if (*sym->current != 'B')
//             goto done;
//     }
//     else if (accmem >= 'A' && accmem <= 'Z')
//         access_id = (accmem - 'A') / 8;
//     else
//         goto done;
//
//     switch (access_id)
//     {
//     case 0: access = "private: "; break;
//     case 1: access = "protected: "; break;
//     case 2: access = "public: "; break;
//     }
//     if (accmem == '$' || (accmem - 'A') % 8 == 6 || (accmem - 'A') % 8 == 7)
//         access = str_printf(sym, "[thunk]:%s", access ? access : " ");
//
//     if (accmem == '$' && *sym->current != 'B')
//         member_type = "virtual ";
//     else if (accmem <= 'X')
//     {
//         switch ((accmem - 'A') % 8)
//         {
//         case 2: case 3: member_type = "static "; break;
//         case 4: case 5: case 6: case 7: member_type = "virtual "; break;
//         }
//     }
//
//     if (sym->flags & UNDNAME_NO_ACCESS_SPECIFIERS)
//         access = NULL;
//     if (sym->flags & UNDNAME_NO_MEMBER_TYPE)
//         member_type = NULL;
//
//     name = get_class_string(sym, 0);
//
//     if (accmem == '$' && *sym->current == 'B') /* vcall thunk */
//     {
//         const char *n;
//
//         sym->current++;
//         n = get_number(sym);
//
//         if(!n || *sym->current++ != 'A') goto done;
//         name = str_printf(sym, "%s{%s,{flat}}' }'", name, n);
//         has_args = FALSE;
//         has_ret = FALSE;
//     }
//     else if (accmem == '$' && *sym->current == 'R') /* vtordispex thunk */
//     {
//         const char *n1, *n2, *n3, *n4;
//
//         sym->current += 2;
//         n1 = get_number(sym);
//         n2 = get_number(sym);
//         n3 = get_number(sym);
//         n4 = get_number(sym);
//
//         if(!n1 || !n2 || !n3 || !n4) goto done;
//         name = str_printf(sym, "%s`vtordispex{%s,%s,%s,%s}' ", name, n1, n2, n3, n4);
//     }
//     else if (accmem == '$') /* vtordisp thunk */
//     {
//         const char *n1, *n2;
//
//         sym->current++;
//         n1 = get_number(sym);
//         n2 = get_number(sym);
//
//         if (!n1 || !n2) goto done;
//         name = str_printf(sym, "%s`vtordisp{%s,%s}' ", name, n1, n2);
//     }
//     else if ((accmem - 'A') % 8 == 6 || (accmem - 'A') % 8 == 7) /* a thunk */
//         name = str_printf(sym, "%s`adjustor{%s}' ", name, get_number(sym));
//
//     if (has_args && (accmem == '$' ||
//                 (accmem <= 'X' && (accmem - 'A') % 8 != 2 && (accmem - 'A') % 8 != 3)))
//     {
//         /* Implicit 'this' pointer */
//         if (!get_function_qualifier(sym, &function_qualifier)) goto done;
//     }
//
//     if (!get_calling_convention(*sym->current++, &call_conv, &exported,
//                                 sym->flags))
//         goto done;
//
//     /* Return type, or @ if 'void' */
//     if (has_ret && *sym->current == '@')
//     {
//         ct_ret.left = "void";
//         ct_ret.right = NULL;
//         sym->current++;
//     }
//     else if (has_ret)
//     {
//         if (!demangle_datatype(sym, &ct_ret, cast_op ? WS_AFTER_QUAL_IF : 0))
//             goto done;
//     }
//     if (!has_ret || sym->flags & UNDNAME_NO_FUNCTION_RETURNS)
//         ct_ret.left = ct_ret.right = NULL;
//     if (cast_op)
//     {
//         name = str_printf(sym, "%s %s%s", name, ct_ret.left, ct_ret.right);
//         ct_ret.left = ct_ret.right = NULL;
//     }
//
//     mark = sym->stack.num;
//     if (has_args && !(args_str = get_args(sym, TRUE, '(', ')'))) goto done;
//     if (sym->flags & UNDNAME_NAME_ONLY) args_str = function_qualifier = NULL;
//     if (sym->flags & UNDNAME_NO_THISTYPE) function_qualifier = NULL;
//     sym->stack.num = mark;
//
//     /* Note: '()' after 'Z' means 'throws', but we don't care here
//      * Yet!!! FIXME
//      */
//     sym->result = str_printf(sym, "%s%s%s%s%s%s%s%s%s%s%s",
//                              access, member_type, ct_ret.left,
//                              (ct_ret.left && !ct_ret.right) ? " " : NULL,
//                              call_conv, call_conv ? " " : NULL, exported,
//                              name, args_str, function_qualifier, ct_ret.right);
//     ret = TRUE;
// done:
//     return ret;
// }
//
// /*******************************************************************
//  *         symbol_demangle
//  * Demangle a C++ linker symbol
//  */
// static BOOL symbol_demangle(struct parsed_symbol* sym)
// {
//     BOOL                ret = FALSE;
//     enum {
//         PP_NONE,
//         PP_CONSTRUCTOR,
//         PP_DESTRUCTOR,
//         PP_CAST_OPERATOR,
//     } post_process = PP_NONE;
//
//     /* FIXME seems wrong as name, as it demangles a simple data type */
//     if (sym->flags & UNDNAME_NO_ARGUMENTS)
//     {
//         struct datatype_t   ct;
//
//         if (demangle_datatype(sym, &ct, 0))
//         {
//             sym->result = str_printf(sym, "%s%s", ct.left, ct.right);
//             ret = TRUE;
//         }
//         goto done;
//     }
//
//     /* MS mangled names always begin with '?' */
//     if (*sym->current != '?') return FALSE;
//     sym->current++;
//
//     /* Then function name or operator code */
//     if (*sym->current == '?')
//     {
//         const char* function_name = NULL;
//         BOOL in_template = FALSE;
//
//         if (sym->current[1] == '$' && sym->current[2] == '?')
//         {
//             in_template = TRUE;
//             sym->current += 2;
//         }
//
//         /* C++ operator code (one character, or two if the first is '_') */
//         switch (*++sym->current)
//         {
//         case '0': function_name = ""; post_process = PP_CONSTRUCTOR; break;
//         case '1': function_name = ""; post_process = PP_DESTRUCTOR; break;
//         case '2': function_name = "operator new"; break;
//         case '3': function_name = "operator delete"; break;
//         case '4': function_name = "operator="; break;
//         case '5': function_name = "operator>>"; break;
//         case '6': function_name = "operator<<"; break;
//         case '7': function_name = "operator!"; break;
//         case '8': function_name = "operator=="; break;
//         case '9': function_name = "operator!="; break;
//         case 'A': function_name = "operator[]"; break;
//         case 'B': function_name = "operator"; post_process = PP_CAST_OPERATOR; break;
//         case 'C': function_name = "operator->"; break;
//         case 'D': function_name = "operator*"; break;
//         case 'E': function_name = "operator++"; break;
//         case 'F': function_name = "operator--"; break;
//         case 'G': function_name = "operator-"; break;
//         case 'H': function_name = "operator+"; break;
//         case 'I': function_name = "operator&"; break;
//         case 'J': function_name = "operator->*"; break;
//         case 'K': function_name = "operator/"; break;
//         case 'L': function_name = "operator%"; break;
//         case 'M': function_name = "operator<"; break;
//         case 'N': function_name = "operator<="; break;
//         case 'O': function_name = "operator>"; break;
//         case 'P': function_name = "operator>="; break;
//         case 'Q': function_name = "operator,"; break;
//         case 'R': function_name = "operator()"; break;
//         case 'S': function_name = "operator~"; break;
//         case 'T': function_name = "operator^"; break;
//         case 'U': function_name = "operator|"; break;
//         case 'V': function_name = "operator&&"; break;
//         case 'W': function_name = "operator||"; break;
//         case 'X': function_name = "operator*="; break;
//         case 'Y': function_name = "operator+="; break;
//         case 'Z': function_name = "operator-="; break;
//         case '_':
//             switch (*++sym->current)
//             {
//             case '0': function_name = "operator/="; break;
//             case '1': function_name = "operator%="; break;
//             case '2': function_name = "operator>>="; break;
//             case '3': function_name = "operator<<="; break;
//             case '4': function_name = "operator&="; break;
//             case '5': function_name = "operator|="; break;
//             case '6': function_name = "operator^="; break;
//             case '7': function_name = "`vftable'"; break;
//             case '8': function_name = "`vbtable'"; break;
//             case '9': function_name = "`vcall'"; break;
//             case 'A': function_name = "`typeof'"; break;
//             case 'B': function_name = "`local static guard'"; break;
//             case 'C': sym->result = (char*)"`string'"; /* string literal: followed by string encoding (native never undecode it) */
//                 /* FIXME: should unmangle the whole string for error reporting */
//                 if (*sym->current && sym->current[strlen(sym->current) - 1] == '@') ret = TRUE;
//                 goto done;
//             case 'D': function_name = "`vbase destructor'"; break;
//             case 'E': function_name = "`vector deleting destructor'"; break;
//             case 'F': function_name = "`default constructor closure'"; break;
//             case 'G': function_name = "`scalar deleting destructor'"; break;
//             case 'H': function_name = "`vector constructor iterator'"; break;
//             case 'I': function_name = "`vector destructor iterator'"; break;
//             case 'J': function_name = "`vector vbase constructor iterator'"; break;
//             case 'K': function_name = "`virtual displacement map'"; break;
//             case 'L': function_name = "`eh vector constructor iterator'"; break;
//             case 'M': function_name = "`eh vector destructor iterator'"; break;
//             case 'N': function_name = "`eh vector vbase constructor iterator'"; break;
//             case 'O': function_name = "`copy constructor closure'"; break;
//             case 'R':
//                 sym->flags |= UNDNAME_NO_FUNCTION_RETURNS;
//                 switch (*++sym->current)
//                 {
//                 case '0':
//                     {
//                         struct datatype_t       ct;
//
//                         sym->current++;
//                         if (!demangle_datatype(sym, &ct, 0))
//                             goto done;
//                         function_name = str_printf(sym, "%s%s `RTTI Type Descriptor'",
//                                                    ct.left, ct.right);
//                         sym->current--;
//                     }
//                     break;
//                 case '1':
//                     {
//                         const char* n1, *n2, *n3, *n4;
//                         sym->current++;
//                         n1 = get_number(sym);
//                         n2 = get_number(sym);
//                         n3 = get_number(sym);
//                         n4 = get_number(sym);
//                         sym->current--;
//                         function_name = str_printf(sym, "`RTTI Base Class Descriptor at (%s,%s,%s,%s)'",
//                                                    n1, n2, n3, n4);
//                     }
//                     break;
//                 case '2': function_name = "`RTTI Base Class Array'"; break;
//                 case '3': function_name = "`RTTI Class Hierarchy Descriptor'"; break;
//                 case '4': function_name = "`RTTI Complete Object Locator'"; break;
//                 default:
//                     ERR("Unknown RTTI operator: _R%c\n", *sym->current);
//                     break;
//                 }
//                 break;
//             case 'S': function_name = "`local vftable'"; break;
//             case 'T': function_name = "`local vftable constructor closure'"; break;
//             case 'U': function_name = "operator new[]"; break;
//             case 'V': function_name = "operator delete[]"; break;
//             case 'X': function_name = "`placement delete closure'"; break;
//             case 'Y': function_name = "`placement delete[] closure'"; break;
//             case '_':
//                 switch (*++sym->current)
//                 {
//                 case 'K':
//                     sym->current++;
//                     function_name = str_printf(sym, "operator \"\" %s", get_literal_string(sym));
//                     --sym->current;
//                     break;
//                 default:
//                     FIXME("Unknown operator: __%c\n", *sym->current);
//                     return FALSE;
//                 }
//                 break;
//             default:
//                 ERR("Unknown operator: _%c\n", *sym->current);
//                 return FALSE;
//             }
//             break;
//         case '$':
//             sym->current++;
//             if (!(function_name = get_template_name(sym))) goto done;
//             --sym->current;
//             break;
//         default:
//             /* FIXME: Other operators */
//             ERR("Unknown operator: %c\n", *sym->current);
//             return FALSE;
//         }
//         sym->current++;
//         if (in_template)
//         {
//             unsigned args_mark = sym->args.num;
//             const char *args;
//
//             args = get_args(sym, FALSE, '<', '>');
//             if (args) function_name = function_name ? str_printf(sym, "%s%s", function_name, args) : args;
//             sym->args.num = args_mark;
//             sym->names.num = 0;
//         }
//         if (!str_array_push(sym, function_name, -1, &sym->stack))
//             return FALSE;
//     }
//     else if (*sym->current == '$')
//     {
//         /* Strange construct, it's a name with a template argument list
//            and that's all. */
//         sym->current++;
//         ret = (sym->result = get_template_name(sym)) != NULL;
//         goto done;
//     }
//
//     /* Either a class name, or '@' if the symbol is not a class member */
//     switch (*sym->current)
//     {
//     case '@': sym->current++; break;
//     case '$': break;
//     default:
//         /* Class the function is associated with, terminated by '@@' */
//         if (!get_class(sym)) goto done;
//         break;
//     }
//
//     switch (post_process)
//     {
//     case PP_NONE: default: break;
//     case PP_CONSTRUCTOR: case PP_DESTRUCTOR:
//         /* it's time to set the member name for ctor & dtor */
//         if (sym->stack.num <= 1) goto done;
//         sym->stack.elts[0] = str_printf(sym, "%s%s%s", post_process == PP_DESTRUCTOR ? "~" : NULL,
//                                         sym->stack.elts[1], sym->stack.elts[0]);
//         /* ctors and dtors don't have return type */
//         sym->flags |= UNDNAME_NO_FUNCTION_RETURNS;
//         break;
//     case PP_CAST_OPERATOR:
//         sym->flags &= ~UNDNAME_NO_FUNCTION_RETURNS;
//         break;
//     }
//
//     /* Function/Data type and access level */
//     if (*sym->current >= '0' && *sym->current <= '9')
//         ret = handle_data(sym);
//     else if ((*sym->current >= 'A' && *sym->current <= 'Z') || *sym->current == '$')
//         ret = handle_method(sym, post_process == PP_CAST_OPERATOR);
//     else ret = FALSE;
// done:
//     if (ret) assert!(sym->result);
//     else WARN("Failed at %s\n", debugstr_a(sym->current));
//
//     return ret;
// }
//
// /*********************************************************************
//  *		__unDNameEx (MSVCRT.@)
//  *
//  * Demangle a C++ identifier.
//  *
//  * PARAMS
//  *  buffer   [O] If not NULL, the place to put the demangled string
//  *  mangled  [I] Mangled name of the function
//  *  buflen   [I] Length of buffer
//  *  memget   [I] Function to allocate memory with
//  *  memfree  [I] Function to free memory with
//  *  unknown  [?] Unknown, possibly a call back
//  *  flags    [I] Flags determining demangled format
//  *
//  * RETURNS
//  *  Success: A string pointing to the unmangled name, allocated with memget.
//  *  Failure: NULL.
//  */
// char*  __unDNameEx(char* buffer, const char* mangled, int buflen,
//                         malloc_func_t memget, free_func_t memfree,
//                         void* unknown, unsigned short int flags)
// {
//     struct parsed_symbol        sym;
//     const char*                 result;
//
//     TRACE("(%p,%s,%d,%p,%p,%p,%x)\n",
//           buffer, debugstr_a(mangled), buflen, memget, memfree, unknown, flags);
//
//     /* The flags details is not documented by MS. However, it looks exactly
//      * like the UNDNAME_ manifest constants from imagehlp.h and dbghelp.h
//      * So, we copied those (on top of the file)
//      */
//     memset(&sym, 0, sizeof(struct parsed_symbol));
//     if (flags & UNDNAME_NAME_ONLY)
//         flags |= UNDNAME_NO_FUNCTION_RETURNS | UNDNAME_NO_ACCESS_SPECIFIERS |
//             UNDNAME_NO_MEMBER_TYPE | UNDNAME_NO_ALLOCATION_LANGUAGE |
//             UNDNAME_NO_COMPLEX_TYPE;
//
//     sym.flags         = flags;
//     sym.mem_alloc_ptr = memget;
//     sym.mem_free_ptr  = memfree;
//     sym.current       = mangled;
//     str_array_init( &sym.names );
//     str_array_init( &sym.stack );
//
//     result = symbol_demangle(&sym) ? sym.result : mangled;
//     if (buffer && buflen)
//     {
//         lstrcpynA( buffer, result, buflen);
//     }
//     else
//     {
//         buffer = memget.unwrap_unchecked()(strlen(result) + 1);
//         if (buffer) strcpy(buffer, result);
//     }
//
//     und_free_all(&sym);
//
//     return buffer;
// }
//
//
// /*********************************************************************
//  *		__unDName (MSVCRT.@)
//  */
// char*  __unDName(char* buffer, const char* mangled, int buflen,
//                       malloc_func_t memget, free_func_t memfree,
//                       unsigned short int flags)
// {
//     return __unDNameEx(buffer, mangled, buflen, memget, memfree, NULL, flags);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
