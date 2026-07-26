//! Wave 14c — `wcs.c`'s printf ENTRY POINTS, over wave 14b's Rust engine.
//!
//! Wave 14b ported the engine (`printf.rs`) and flipped its six symbols, expecting
//! whole-archive first-match to swing every shell. It swung the shells in OTHER
//! translation units — file.c's stream family calls the Rust `pf_printf_a` — and left
//! `wcs.c`'s own alone, because clang had already IPA-specialized the same-TU engine
//! into them (`pf_printf_a.specialized.N`) and intra-TU inlining never consults the
//! linker. **Symbol replacement flips a TU's callers, not its housemates.** So the only
//! way these reach the Rust engine is to define the shells themselves, which is this
//! file.
//!
//! Everything here is a transcription of `dll/win32/msvcrt/wcs.c` at `_MSVCR_VER=0`,
//! which is what `msvcrt_static` is compiled with. The `#if _MSVCR_VER>=140`
//! (`__stdio_common_*`) and `>=80` (`_vswprintf_p`, `_vscwprintf_p`) blocks are not
//! built into msvcrt.dll and are not here either — grep this file for a name and its
//! absence is the answer.
//!
//! **`va_list` is `*mut u8` BY VALUE.** On all three of our Windows targets a `va_list`
//! is a plain `char*` over the flat argument-slot area, which is why the engine takes
//! `*mut *mut u8` and why stable Rust needs no `c_variadic` to be here at all: a fixed-
//! arity function that *receives* an already-started cursor is ordinary code. Each shell
//! keeps its own `let mut ap = valist` before handing `&mut ap` down, mirroring the C
//! exactly: the C's parameter IS a private copy, and `&valist` is the address of that
//! copy, so a callee that advances the cursor never disturbs its caller's.
//!
//! The `...` shells (`sprintf`, `_snprintf`, `_swprintf`, …) cannot live here: stable
//! Rust cannot `va_start`. They are three-line C trampolines in
//! `dll/win32/msvcrt/rust_varargs_shim.c` that delegate to the twins below — the same
//! sanctioned language-gap shim the 12 driver crates use (`docs/engineering/
//! stable-toolchain-plan.md`). They must live in their OWN TU for the housemate reason
//! above; left in wcs.c they would keep inlining wcs.c's C engine.
//!
//! Graded row-for-row against the genuine wcs.c by `msvcrt-difftest/tests/printf_shells.rs`.

use core::ffi::c_void;

use crate::printf::{
    arg_clbk_positional, arg_clbk_valist, create_positional_ctx_a, create_positional_ctx_w,
    pf_printf_a, pf_printf_w, puts_clbk_str_a, puts_clbk_str_w, PrintfArg, StrCtxA, StrCtxW,
    ARGMAX, MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER, MSVCRT_PRINTF_POSITIONAL_PARAMS,
};

/// `INT_MAX` as the C spells it in these shells — an `int` constant that then converts
/// to the `size_t` parameter.
const INT_MAX_SZ: usize = i32::MAX as usize;

/// `_TRUNCATE` / a `size_t` compared against the literal `-1`. Both are all-ones.
const SIZE_MAX: usize = usize::MAX;

/// The positional shells spell the failure out rather than using `MSVCRT_INVALID_PMT`,
/// and in the OPPOSITE order — handler FIRST, then errno:
///
/// ```c
/// _invalid_parameter(NULL, NULL, NULL, 0, 0);
/// *_errno() = EINVAL;
/// ```
///
/// `MSVCRT_INVALID_PMT` (msvcrt.h:376) is `(*_errno() = err, _invalid_parameter(...))`,
/// the other way round. The order is observable: an application-installed handler that
/// longjmps out never reaches the assignment, so it sees whatever errno held before.
/// printf.rs's unknown-conversion arm keeps the same distinction.
#[inline]
unsafe fn invalid_param_then_einval() {
    crate::_invalid_parameter(
        core::ptr::null(),
        core::ptr::null(),
        core::ptr::null(),
        0,
        0,
    );
    *crate::_errno() = crate::EINVAL;
}

/// The C writes `static const char nullbyte = '\0'` and passes `&nullbyte`; the wide
/// side passes `L""`. Same thing: a pointer to one zero unit.
#[inline]
unsafe fn append_nul_a(ctx: *mut StrCtxA) {
    let nullbyte = 0u8;
    puts_clbk_str_a(ctx as *mut c_void, 1, &nullbyte);
}

#[inline]
unsafe fn append_nul_w(ctx: *mut StrCtxW) {
    let nullbyte = 0u16;
    puts_clbk_str_w(ctx as *mut c_void, 1, &nullbyte);
}

// ── narrow: the plain sinks (wcs.c:878-960) ──────────────────────────────────

/// wcs.c:881-891 — `_vsnprintf`. The `_MSVCR_VER < 140` body; the `>=140` build
/// replaces it with `__stdio_common_vsprintf`, which msvcrt.dll does not compile.
///
/// Note there is no NULL/`len` validation and no truncation report: the return is the
/// engine's would-have-written count and the trailing NUL is appended THROUGH the
/// callback, so it is dropped rather than forced when the buffer is full. Both are
/// contract, not oversight.
#[no_mangle]
pub unsafe extern "C" fn _vsnprintf(
    str_: *mut u8,
    len: usize,
    format: *const u8,
    valist: *mut u8,
) -> i32 {
    let mut ctx = StrCtxA { len, buf: str_ };
    let mut ap = valist;
    let ret = pf_printf_a(
        puts_clbk_str_a,
        &mut ctx as *mut StrCtxA as *mut c_void,
        format,
        core::ptr::null_mut(),
        0,
        arg_clbk_valist,
        core::ptr::null_mut(),
        &mut ap,
    );
    append_nul_a(&mut ctx);
    ret
}

/// wcs.c:949-960 — `_vsnprintf_l`.
#[no_mangle]
pub unsafe extern "C" fn _vsnprintf_l(
    str_: *mut u8,
    len: usize,
    format: *const u8,
    locale: *mut c_void,
    valist: *mut u8,
) -> i32 {
    let mut ctx = StrCtxA { len, buf: str_ };
    let mut ap = valist;
    let ret = pf_printf_a(
        puts_clbk_str_a,
        &mut ctx as *mut StrCtxA as *mut c_void,
        format,
        locale,
        0,
        arg_clbk_valist,
        core::ptr::null_mut(),
        &mut ap,
    );
    append_nul_a(&mut ctx);
    ret
}

/// wcs.c:965-969 — `_vsprintf_l`.
#[no_mangle]
pub unsafe extern "C" fn _vsprintf_l(
    str_: *mut u8,
    format: *const u8,
    locale: *mut c_void,
    valist: *mut u8,
) -> i32 {
    _vsnprintf_l(str_, INT_MAX_SZ, format, locale, valist)
}

/// wcs.c:1160-1163 — `vsprintf`. The C calls plain `vsnprintf`, which
/// `sdk/include/crt/stdio.h:719-720` macro-defines to `_vsnprintf`.
#[no_mangle]
pub unsafe extern "C" fn vsprintf(str_: *mut u8, format: *const u8, valist: *mut u8) -> i32 {
    _vsnprintf(str_, INT_MAX_SZ, format, valist)
}

/// wcs.c:1168-1171 — `vsprintf_s`. Despite the `_s`, it is `vsnprintf` with a bound:
/// no invalid-parameter handler, no zeroing, no `-1`. Quirk, kept.
#[no_mangle]
pub unsafe extern "C" fn vsprintf_s(
    str_: *mut u8,
    num: usize,
    format: *const u8,
    valist: *mut u8,
) -> i32 {
    _vsnprintf(str_, num, format, valist)
}

// ── narrow: the counting sinks (wcs.c:1174-1226) ─────────────────────────────

/// wcs.c:1176-1179 — `_vscprintf`: a NULL buffer makes the callback count without
/// storing.
#[no_mangle]
pub unsafe extern "C" fn _vscprintf(format: *const u8, valist: *mut u8) -> i32 {
    _vsnprintf_l(core::ptr::null_mut(), INT_MAX_SZ, format, core::ptr::null_mut(), valist)
}

/// wcs.c:1184-1188 — `_vscprintf_l`.
#[no_mangle]
pub unsafe extern "C" fn _vscprintf_l(
    format: *const u8,
    locale: *mut c_void,
    valist: *mut u8,
) -> i32 {
    _vsnprintf_l(core::ptr::null_mut(), INT_MAX_SZ, format, locale, valist)
}

/// wcs.c:1193-1218 — `_vscprintf_p_l`: the positional (`%1$s`) counting form.
///
/// Three-way on `create_positional_ctx_a`: negative is a malformed/mixed format (handler
/// + EINVAL, and the NEGATIVE value is returned, not -1), zero means the format turned
/// out to be purely sequential so the va cursor is used directly, positive means the
/// argument table was filled and is read out of order.
#[no_mangle]
pub unsafe extern "C" fn _vscprintf_p_l(
    format: *const u8,
    locale: *mut c_void,
    args: *mut u8,
) -> i32 {
    let mut args_ctx = [PrintfArg { get_longlong: 0 }; ARGMAX as usize + 1];
    let mut puts_ctx = StrCtxA { len: INT_MAX_SZ, buf: core::ptr::null_mut() };
    let mut ap = args;

    let ret = create_positional_ctx_a(args_ctx.as_mut_ptr() as *mut c_void, format, args);
    if ret < 0 {
        invalid_param_then_einval();
        return ret;
    } else if ret == 0 {
        pf_printf_a(
            puts_clbk_str_a,
            &mut puts_ctx as *mut StrCtxA as *mut c_void,
            format,
            locale,
            MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER,
            arg_clbk_valist,
            core::ptr::null_mut(),
            &mut ap,
        )
    } else {
        pf_printf_a(
            puts_clbk_str_a,
            &mut puts_ctx as *mut StrCtxA as *mut c_void,
            format,
            locale,
            MSVCRT_PRINTF_POSITIONAL_PARAMS | MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER,
            arg_clbk_positional,
            args_ctx.as_mut_ptr() as *mut c_void,
            core::ptr::null_mut(),
        )
    }
}

/// wcs.c:1223-1226 — `_vscprintf_p`.
#[no_mangle]
pub unsafe extern "C" fn _vscprintf_p(format: *const u8, argptr: *mut u8) -> i32 {
    _vscprintf_p_l(format, core::ptr::null_mut(), argptr)
}

// ── narrow: the `_s` bounded family (wcs.c:985-1105) ─────────────────────────

/// wcs.c:985-1015 — `vsnprintf_s_l_opt`, the shared `_s` body.
///
/// The `int len` is the C's, and it is load-bearing in two places: a `size_t`
/// `sizeOfBuffer` above `INT_MAX` TRUNCATES into it (and then sign-extends back on the
/// way into `ctx.len`), and `ret == len` is the "filled it exactly" test that decides
/// truncation. Transcribed with the casts spelled out rather than widened to `size_t`,
/// because widening would change which inputs report -1.
unsafe fn vsnprintf_s_l_opt(
    str_: *mut u8,
    size_of_buffer: usize,
    count: usize,
    format: *const u8,
    options: u32,
    locale: *mut c_void,
    valist: *mut u8,
) -> i32 {
    // `if(sizeOfBuffer < count+1 || count == -1)` — `count+1` is size_t arithmetic and
    // WRAPS to 0 when count is SIZE_MAX, which is why the explicit `count == -1` arm
    // exists beside it.
    let len: i32 = if size_of_buffer < count.wrapping_add(1) || count == SIZE_MAX {
        size_of_buffer as i32
    } else {
        count.wrapping_add(1) as i32
    };

    let mut ctx = StrCtxA { len: len as usize, buf: str_ };
    let mut ap = valist;
    let ret = pf_printf_a(
        puts_clbk_str_a,
        &mut ctx as *mut StrCtxA as *mut c_void,
        format,
        locale,
        MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER | options,
        arg_clbk_valist,
        core::ptr::null_mut(),
        &mut ap,
    );
    append_nul_a(&mut ctx);

    if ret < 0 || ret == len {
        if count != SIZE_MAX && count > size_of_buffer {
            crate::invalid_pmt(crate::ERANGE);
            crate::memset(str_ as *mut c_void, 0, size_of_buffer);
        } else {
            *str_.offset(len as isize - 1) = 0;
        }
        return -1;
    }

    ret
}

/// wcs.c:1050-1055 — `_vsnprintf_s_l`.
#[no_mangle]
pub unsafe extern "C" fn _vsnprintf_s_l(
    str_: *mut u8,
    size_of_buffer: usize,
    count: usize,
    format: *const u8,
    locale: *mut c_void,
    valist: *mut u8,
) -> i32 {
    vsnprintf_s_l_opt(str_, size_of_buffer, count, format, 0, locale, valist)
}

/// wcs.c:1060-1064 — `_vsprintf_s_l`.
#[no_mangle]
pub unsafe extern "C" fn _vsprintf_s_l(
    str_: *mut u8,
    count: usize,
    format: *const u8,
    locale: *mut c_void,
    valist: *mut u8,
) -> i32 {
    _vsnprintf_s_l(str_, INT_MAX_SZ, count, format, locale, valist)
}

/// wcs.c:1083-1087 — `_vsnprintf_s`.
#[no_mangle]
pub unsafe extern "C" fn _vsnprintf_s(
    str_: *mut u8,
    size_of_buffer: usize,
    count: usize,
    format: *const u8,
    valist: *mut u8,
) -> i32 {
    _vsnprintf_s_l(str_, size_of_buffer, count, format, core::ptr::null_mut(), valist)
}

/// wcs.c:1092-1096 — `_vsnprintf_c_l`. C99 truncation semantics reached by passing the
/// buffer size as BOTH bound and count, so the `count > sizeOfBuffer` ERANGE arm above
/// can never fire and the tail is simply NUL-terminated.
#[no_mangle]
pub unsafe extern "C" fn _vsnprintf_c_l(
    str_: *mut u8,
    len: usize,
    format: *const u8,
    locale: *mut c_void,
    valist: *mut u8,
) -> i32 {
    vsnprintf_s_l_opt(str_, len, len, format, 0, locale, valist)
}

/// wcs.c:1101-1105 — `_vsnprintf_c`.
#[no_mangle]
pub unsafe extern "C" fn _vsnprintf_c(
    str_: *mut u8,
    len: usize,
    format: *const u8,
    valist: *mut u8,
) -> i32 {
    _vsnprintf_c_l(str_, len, format, core::ptr::null_mut(), valist)
}

// ── narrow: the positional (`_p`) family (wcs.c:1832-1875) ───────────────────

/// wcs.c:1832-1857 — `vsprintf_p_l_opt`. Same three-way as `_vscprintf_p_l`, but into a
/// real buffer and with the trailing NUL appended after either branch.
unsafe fn vsprintf_p_l_opt(
    buffer: *mut u8,
    length: usize,
    format: *const u8,
    options: u32,
    locale: *mut c_void,
    args: *mut u8,
) -> i32 {
    let mut args_ctx = [PrintfArg { get_longlong: 0 }; ARGMAX as usize + 1];
    let mut puts_ctx = StrCtxA { len: length, buf: buffer };
    let mut ap = args;

    let ret = create_positional_ctx_a(args_ctx.as_mut_ptr() as *mut c_void, format, args);
    let ret = if ret < 0 {
        invalid_param_then_einval();
        return ret; // NOTE: returns BEFORE appending the NUL. Verbatim.
    } else if ret == 0 {
        pf_printf_a(
            puts_clbk_str_a,
            &mut puts_ctx as *mut StrCtxA as *mut c_void,
            format,
            locale,
            MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER | options,
            arg_clbk_valist,
            core::ptr::null_mut(),
            &mut ap,
        )
    } else {
        pf_printf_a(
            puts_clbk_str_a,
            &mut puts_ctx as *mut StrCtxA as *mut c_void,
            format,
            locale,
            MSVCRT_PRINTF_POSITIONAL_PARAMS | MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER | options,
            arg_clbk_positional,
            args_ctx.as_mut_ptr() as *mut c_void,
            core::ptr::null_mut(),
        )
    };

    append_nul_a(&mut puts_ctx);
    ret
}

/// wcs.c:1862-1866 — `_vsprintf_p_l`.
#[no_mangle]
pub unsafe extern "C" fn _vsprintf_p_l(
    buffer: *mut u8,
    length: usize,
    format: *const u8,
    locale: *mut c_void,
    args: *mut u8,
) -> i32 {
    vsprintf_p_l_opt(buffer, length, format, 0, locale, args)
}

/// wcs.c:1871-1875 — `_vsprintf_p`.
#[no_mangle]
pub unsafe extern "C" fn _vsprintf_p(
    buffer: *mut u8,
    length: usize,
    format: *const u8,
    args: *mut u8,
) -> i32 {
    _vsprintf_p_l(buffer, length, format, core::ptr::null_mut(), args)
}

// ── wide: the plain sinks (wcs.c:1362-1408) ──────────────────────────────────

/// wcs.c:1365-1375 — `_vsnwprintf`.
#[no_mangle]
pub unsafe extern "C" fn _vsnwprintf(
    str_: *mut u16,
    len: usize,
    format: *const u16,
    valist: *mut u8,
) -> i32 {
    let mut ctx = StrCtxW { len, buf: str_ };
    let mut ap = valist;
    let ret = pf_printf_w(
        puts_clbk_str_w,
        &mut ctx as *mut StrCtxW as *mut c_void,
        format,
        core::ptr::null_mut(),
        0,
        arg_clbk_valist,
        core::ptr::null_mut(),
        &mut ap,
    );
    append_nul_w(&mut ctx);
    ret
}

/// wcs.c:1380-1390 — `_vsnwprintf_l`.
#[no_mangle]
pub unsafe extern "C" fn _vsnwprintf_l(
    str_: *mut u16,
    len: usize,
    format: *const u16,
    locale: *mut c_void,
    valist: *mut u8,
) -> i32 {
    let mut ctx = StrCtxW { len, buf: str_ };
    let mut ap = valist;
    let ret = pf_printf_w(
        puts_clbk_str_w,
        &mut ctx as *mut StrCtxW as *mut c_void,
        format,
        locale,
        0,
        arg_clbk_valist,
        core::ptr::null_mut(),
        &mut ap,
    );
    append_nul_w(&mut ctx);
    ret
}

/// wcs.c:1765-1768 — `_vswprintf`.
#[no_mangle]
pub unsafe extern "C" fn _vswprintf(str_: *mut u16, format: *const u16, args: *mut u8) -> i32 {
    _vsnwprintf(str_, INT_MAX_SZ, format, args)
}

/// wcs.c:1773-1777 — `_vswprintf_l`.
#[no_mangle]
pub unsafe extern "C" fn _vswprintf_l(
    str_: *mut u16,
    format: *const u16,
    locale: *mut c_void,
    args: *mut u8,
) -> i32 {
    _vsnwprintf_l(str_, INT_MAX_SZ, format, locale, args)
}

/// wcs.c:1782-1785 — `_vscwprintf`.
#[no_mangle]
pub unsafe extern "C" fn _vscwprintf(format: *const u16, args: *mut u8) -> i32 {
    _vsnwprintf(core::ptr::null_mut(), INT_MAX_SZ, format, args)
}

/// wcs.c:1790-1793 — `_vscwprintf_l`.
#[no_mangle]
pub unsafe extern "C" fn _vscwprintf_l(
    format: *const u16,
    locale: *mut c_void,
    args: *mut u8,
) -> i32 {
    _vsnwprintf_l(core::ptr::null_mut(), INT_MAX_SZ, format, locale, args)
}

// ── wide: the `_s` bounded family (wcs.c:1017-1045, 1471-1488) ───────────────

/// wcs.c:1017-1045 — `vsnwprintf_s_l_opt`.
///
/// NOT the narrow body with a width swap, and the difference is real: the narrow one
/// clamps with `sizeOfBuffer < count+1`, this one starts at `sizeOfBuffer` and lowers
/// only `if(count != -1 && len > count+1)`. The comparison `len > count+1` is
/// `int > size_t`, so `len` converts to UNSIGNED — a `sizeOfBuffer` that truncated
/// negative compares as huge and the clamp fires. Transcribed as the C computes it.
unsafe fn vsnwprintf_s_l_opt(
    str_: *mut u16,
    size_of_buffer: usize,
    count: usize,
    format: *const u16,
    options: u32,
    locale: *mut c_void,
    valist: *mut u8,
) -> i32 {
    let mut len: i32 = size_of_buffer as i32;
    if count != SIZE_MAX && (len as usize) > count.wrapping_add(1) {
        len = count.wrapping_add(1) as i32;
    }

    let mut ctx = StrCtxW { len: len as usize, buf: str_ };
    let mut ap = valist;
    let ret = pf_printf_w(
        puts_clbk_str_w,
        &mut ctx as *mut StrCtxW as *mut c_void,
        format,
        locale,
        MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER | options,
        arg_clbk_valist,
        core::ptr::null_mut(),
        &mut ap,
    );
    append_nul_w(&mut ctx);

    if ret < 0 || ret == len {
        if count != SIZE_MAX && count > size_of_buffer {
            crate::invalid_pmt(crate::ERANGE);
            // `memset(str, 0, sizeOfBuffer*sizeof(wchar_t))` — BYTES, not units.
            crate::memset(str_ as *mut c_void, 0, size_of_buffer * 2);
        } else {
            *str_.offset(len as isize - 1) = 0;
        }
        return -1;
    }

    ret
}

/// wcs.c:1473-1478 — `_vsnwprintf_s_l`.
#[no_mangle]
pub unsafe extern "C" fn _vsnwprintf_s_l(
    str_: *mut u16,
    size_of_buffer: usize,
    count: usize,
    format: *const u16,
    locale: *mut c_void,
    valist: *mut u8,
) -> i32 {
    vsnwprintf_s_l_opt(str_, size_of_buffer, count, format, 0, locale, valist)
}

/// wcs.c:1483-1488 — `_vsnwprintf_s`.
#[no_mangle]
pub unsafe extern "C" fn _vsnwprintf_s(
    str_: *mut u16,
    size_of_buffer: usize,
    count: usize,
    format: *const u16,
    valist: *mut u8,
) -> i32 {
    _vsnwprintf_s_l(str_, size_of_buffer, count, format, core::ptr::null_mut(), valist)
}

/// wcs.c:1395-1399 — `_vswprintf_c_l`.
#[no_mangle]
pub unsafe extern "C" fn _vswprintf_c_l(
    str_: *mut u16,
    len: usize,
    format: *const u16,
    locale: *mut c_void,
    valist: *mut u8,
) -> i32 {
    vsnwprintf_s_l_opt(str_, len, len, format, 0, locale, valist)
}

/// wcs.c:1404-1408 — `_vswprintf_c`.
#[no_mangle]
pub unsafe extern "C" fn _vswprintf_c(
    str_: *mut u16,
    len: usize,
    format: *const u16,
    valist: *mut u8,
) -> i32 {
    _vswprintf_c_l(str_, len, format, core::ptr::null_mut(), valist)
}

/// wcs.c:1816-1820 — `vswprintf_s`.
#[no_mangle]
pub unsafe extern "C" fn vswprintf_s(
    str_: *mut u16,
    number_of_elements: usize,
    format: *const u16,
    args: *mut u8,
) -> i32 {
    _vsnwprintf_s(str_, number_of_elements, INT_MAX_SZ, format, args)
}

/// wcs.c:1825-1830 — `_vswprintf_s_l`.
#[no_mangle]
pub unsafe extern "C" fn _vswprintf_s_l(
    str_: *mut u16,
    number_of_elements: usize,
    format: *const u16,
    locale: *mut c_void,
    args: *mut u8,
) -> i32 {
    _vsnwprintf_s_l(str_, number_of_elements, INT_MAX_SZ, format, locale, args)
}

// ── wide: the positional (`_p`) family (wcs.c:1410-1443, 1798-1801) ──────────

/// wcs.c:1410-1434 — `vswprintf_p_l_opt`.
///
/// Note the asymmetry with the narrow twin: on the `ret < 0` arm the narrow one returns
/// without appending a NUL and so does this, but this one seeds `puts_ctx` with the
/// caller's `length` where the counting shells pass `INT_MAX`. Verbatim.
unsafe fn vswprintf_p_l_opt(
    buffer: *mut u16,
    length: usize,
    format: *const u16,
    options: u32,
    locale: *mut c_void,
    args: *mut u8,
) -> i32 {
    let mut args_ctx = [PrintfArg { get_longlong: 0 }; ARGMAX as usize + 1];
    let mut puts_ctx = StrCtxW { len: length, buf: buffer };
    let mut ap = args;

    let ret = create_positional_ctx_w(args_ctx.as_mut_ptr() as *mut c_void, format, args);
    let ret = if ret < 0 {
        invalid_param_then_einval();
        return ret;
    } else if ret == 0 {
        pf_printf_w(
            puts_clbk_str_w,
            &mut puts_ctx as *mut StrCtxW as *mut c_void,
            format,
            locale,
            MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER | options,
            arg_clbk_valist,
            core::ptr::null_mut(),
            &mut ap,
        )
    } else {
        pf_printf_w(
            puts_clbk_str_w,
            &mut puts_ctx as *mut StrCtxW as *mut c_void,
            format,
            locale,
            MSVCRT_PRINTF_POSITIONAL_PARAMS | MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER | options,
            arg_clbk_positional,
            args_ctx.as_mut_ptr() as *mut c_void,
            core::ptr::null_mut(),
        )
    };

    append_nul_w(&mut puts_ctx);
    ret
}

/// wcs.c:1439-1443 — `_vswprintf_p_l`.
#[no_mangle]
pub unsafe extern "C" fn _vswprintf_p_l(
    buffer: *mut u16,
    length: usize,
    format: *const u16,
    locale: *mut c_void,
    args: *mut u8,
) -> i32 {
    vswprintf_p_l_opt(buffer, length, format, 0, locale, args)
}

/// wcs.c:1798-1801 — `_vscwprintf_p_l`.
#[no_mangle]
pub unsafe extern "C" fn _vscwprintf_p_l(
    format: *const u16,
    locale: *mut c_void,
    args: *mut u8,
) -> i32 {
    vswprintf_p_l_opt(core::ptr::null_mut(), INT_MAX_SZ, format, 0, locale, args)
}
