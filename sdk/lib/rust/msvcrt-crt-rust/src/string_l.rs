//! `dll/win32/msvcrt/string.c` — the explicitly-locale'd (`_l`) exports, plus the three
//! stragglers that never took a locale at all (`_strdup`, `_atoflt`, `I10_OUTPUT`).
//!
//! Almost nothing here is new logic. `lib.rs` already ships every one of these algorithms
//! with the locale hardwired to `get_locinfo()`, because until now the only exports flipped
//! were the plain forms — `strtol` is `_strtol_l(…, NULL)`, `_stricmp` is
//! `_strnicmp_l(…, INT_MAX, NULL)`, and so on. So the work was to give those shared cores
//! the `_locale_t` parameter the C always had (`locinfo_of(NULL) == get_locinfo()`, which
//! is what makes that refactor a no-op for the plain forms) and to write the wrappers here.
//! Duplicating the engines would have forked five contracts that took eleven waves and a
//! bit-exact oracle to get right.
//!
//! The two spellings of the locale pointer are the same pointer: the crate-root helpers
//! take `*mut c_void` (ctype.c's `_l` exports were written that way), while `_locale_t` is
//! bindgen's `*mut localeinfo_struct`. `locinfo_of` casts back before touching a field.

#![allow(non_camel_case_types, non_snake_case)]
// `_I10_OUTPUT_DATA`, `MSVCRT__LDOUBLE` and `I10_OUTPUT` keep the C's spelling: they are
// ABI contracts with msvcrt's own headers and with `$I10_OUTPUT`'s spec forward.

use crate::prelude::*;

// `dll/win32/msvcrt/include/math.h:63/:67` — the TU's own include closure. Only
// `I10_OUTPUT` uses them, which is why they are here rather than in the prelude.
extern "C" {
    fn log10(x: f64) -> f64;
    fn floor(x: f64) -> f64;
}

// ── _strdup ──────────────────────────────────────────────────────────────────

/// `_strdup` — and `_mbsdup`, which msvcrt.spec forwards to this same symbol
/// (`msvcrt.spec:735`), so there is deliberately no second definition.
///
/// No `_errno` on allocation failure and no invalid-parameter handler: a NULL `str` is a
/// quiet NULL return, and a failed `malloc` returns NULL with whatever errno malloc left.
#[no_mangle]
pub unsafe extern "C" fn _strdup(str_: *const c_char) -> *mut c_char {
    if !str_.is_null() {
        let ret = malloc(strlen(str_) + 1) as *mut c_char;
        if !ret.is_null() {
            strcpy(ret, str_);
        }
        ret
    } else {
        core::ptr::null_mut()
    }
}

// ── the case folders ─────────────────────────────────────────────────────────

/// `_strlwr_s_l`.
#[no_mangle]
pub unsafe extern "C" fn _strlwr_s_l(str_: *mut c_char, len: size_t, locale: _locale_t) -> c_int {
    crate::strcase_s_core(str_ as *mut u8, len, false, locale as *mut c_void)
}

/// `_strupr_s_l`.
#[no_mangle]
pub unsafe extern "C" fn _strupr_s_l(str_: *mut c_char, len: size_t, locale: _locale_t) -> c_int {
    crate::strcase_s_core(str_ as *mut u8, len, true, locale as *mut c_void)
}

/// `_strlwr_l` — `_strlwr_s_l(str, -1, locale)`, return value discarded.
///
/// The `-1` is a `size_t`, i.e. SIZE_MAX, so the "is there a terminator inside `len`"
/// scan can never fail and the EINVAL arm is unreachable from here. It still walks the
/// whole string once before folding it; that second pass is the C's, not ours.
#[no_mangle]
pub unsafe extern "C" fn _strlwr_l(str_: *mut c_char, locale: _locale_t) -> *mut c_char {
    _strlwr_s_l(str_, size_t::MAX, locale);
    str_
}

/// `_strupr_l` — `_strupr_s_l(str, -1, locale)`, return value discarded.
#[no_mangle]
pub unsafe extern "C" fn _strupr_l(str_: *mut c_char, locale: _locale_t) -> *mut c_char {
    _strupr_s_l(str_, size_t::MAX, locale);
    str_
}

// ── the integer parsers ──────────────────────────────────────────────────────
//
// One engine each, signed and unsigned, and everything else is a clamp over the SIGNED
// one — including `_strtoul_l`, which is why its underflow arm is so strange.
//
// `__msvcrt_long` / `__msvcrt_ulong` are **32-bit on all three targets** (Windows LP32),
// so every `long`-returning function below is `i32`/`u32` and NOT `isize`. Getting that
// wrong on amd64/arm64 is silent: the clamp would simply never fire.

/// `_strtoi64_l` — msvcrt's signed integer engine.
#[no_mangle]
pub unsafe extern "C" fn _strtoi64_l(
    nptr: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
    locale: _locale_t,
) -> i64 {
    crate::strtoi64_contract(
        nptr as *const u8,
        endptr as *mut *mut u8,
        base,
        locale as *mut c_void,
    )
}

/// `_strtoui64_l` — the UNSIGNED engine. A separate function, not a cast of the signed
/// one: it saturates at `_UI64_MAX` and negates the unsigned accumulator at the end, and
/// it takes its decimal digit value from the RAW byte where the signed engine takes it
/// from the case-folded one.
#[no_mangle]
pub unsafe extern "C" fn _strtoui64_l(
    nptr: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
    locale: _locale_t,
) -> u64 {
    crate::strtoui64_contract(
        nptr as *const u8,
        endptr as *mut *mut u8,
        base,
        locale as *mut c_void,
    )
}

/// `_atoi_l` — `_strtoi64_l(str, NULL, 10, locale)` clamped to `int` with ERANGE.
///
/// Note this is NOT `atoi`: under `_MSVCR_VER == 0` (which is how msvcrt.dll is built)
/// `atoi` is a separate hand-rolled wrapping parser with no error channel at all, and it
/// stays where it is in `lib.rs`.
#[no_mangle]
pub unsafe extern "C" fn _atoi_l(str_: *const c_char, locale: _locale_t) -> c_int {
    let mut ret = _strtoi64_l(str_, core::ptr::null_mut(), 10, locale);

    if ret > c_int::MAX as i64 {
        ret = c_int::MAX as i64;
        *_errno() = crate::ERANGE;
    } else if ret < c_int::MIN as i64 {
        ret = c_int::MIN as i64;
        *_errno() = crate::ERANGE;
    }
    // The C returns `ret` through an `int`; the clamp above makes that narrowing exact.
    ret as c_int
}

/// `_atoi64_l`.
#[no_mangle]
pub unsafe extern "C" fn _atoi64_l(str_: *const c_char, locale: _locale_t) -> i64 {
    _strtoi64_l(str_, core::ptr::null_mut(), 10, locale)
}

/// `_atol_l` — the same clamp as `_atoi_l`, over the 32-bit Windows `long`.
#[no_mangle]
pub unsafe extern "C" fn _atol_l(str_: *const c_char, locale: _locale_t) -> i32 {
    let mut ret = _strtoi64_l(str_, core::ptr::null_mut(), 10, locale);

    if ret > i32::MAX as i64 {
        ret = i32::MAX as i64;
        *_errno() = crate::ERANGE;
    } else if ret < i32::MIN as i64 {
        ret = i32::MIN as i64;
        *_errno() = crate::ERANGE;
    }
    ret as i32
}

/// `_strtol_l`.
#[no_mangle]
pub unsafe extern "C" fn _strtol_l(
    nptr: *const c_char,
    end: *mut *mut c_char,
    base: c_int,
    locale: _locale_t,
) -> i32 {
    let mut ret = _strtoi64_l(nptr, end, base, locale);

    if ret > i32::MAX as i64 {
        ret = i32::MAX as i64;
        *_errno() = crate::ERANGE;
    } else if ret < i32::MIN as i64 {
        ret = i32::MIN as i64;
        *_errno() = crate::ERANGE;
    }
    ret as i32
}

/// `_strtoul_l`.
///
/// preserved misfeature: it parses with the **signed** `_strtoi64_l` and then clamps
/// ASYMMETRICALLY, and the underflow arm returns **1**:
///
/// ```c
/// if(ret > ULONG_MAX)                { ret = ULONG_MAX; errno = ERANGE; }
/// else if(ret < -(__int64)ULONG_MAX) { ret = 1;         errno = ERANGE; }
/// ```
///
/// Not 0, not ULONG_MAX — one. Anything in between falls through and is truncated to 32
/// bits, which is how `_strtoul_l("-1")` yields 0xFFFFFFFF while `_strtoul_l("-4294967296")`
/// yields 1. `ULONG_MAX` is `unsigned long`, so C's usual arithmetic conversions widen it
/// to `__int64` for both comparisons rather than pushing `ret` into unsigned territory.
#[no_mangle]
pub unsafe extern "C" fn _strtoul_l(
    nptr: *const c_char,
    end: *mut *mut c_char,
    base: c_int,
    locale: _locale_t,
) -> u32 {
    const ULONG_MAX: i64 = 4294967295;
    let mut ret = _strtoi64_l(nptr, end, base, locale);

    if ret > ULONG_MAX {
        ret = ULONG_MAX;
        *_errno() = crate::ERANGE;
    } else if ret < -ULONG_MAX {
        ret = 1;
        *_errno() = crate::ERANGE;
    }
    ret as u32
}

// ── the case-insensitive comparers ───────────────────────────────────────────

/// `_strnicmp_l`.
///
/// `count == 0` returns 0 BEFORE the front door, so `_strnicmp_l(NULL, NULL, 0, l)` is 0
/// and fires no handler. Under `_MSVCR_VER == 0` the front door is only `s1 && s2` — the
/// `count <= INT_MAX` half arrived in msvcr80 and is not compiled here.
#[no_mangle]
pub unsafe extern "C" fn _strnicmp_l(
    s1: *const c_char,
    s2: *const c_char,
    count: size_t,
    locale: _locale_t,
) -> c_int {
    crate::strnicmp_core(s1 as *const u8, s2 as *const u8, count, locale as *mut c_void)
}

/// `_stricmp_l` — `_strnicmp_l(s1, s2, INT_MAX, locale)`.
#[no_mangle]
pub unsafe extern "C" fn _stricmp_l(
    s1: *const c_char,
    s2: *const c_char,
    locale: _locale_t,
) -> c_int {
    _strnicmp_l(s1, s2, c_int::MAX as size_t, locale)
}

/// `_memicmp_l`.
///
/// Its NULL guard is hand-rolled rather than `MSVCRT_CHECK_PMT`, but it still fires the
/// handler — and only when `len != 0`, so `_memicmp_l(NULL, NULL, 0, l)` is a silent 0.
/// It also folds through `_tolower_l` on a SIGNED char and returns that difference RAW,
/// where `_strnicmp_l` casts to `unsigned char`; on bytes >= 0x80 the two disagree in
/// both the return value and the errno. They are not one comparison with two bounds.
#[no_mangle]
pub unsafe extern "C" fn _memicmp_l(
    v1: *const c_void,
    v2: *const c_void,
    len: size_t,
    locale: _locale_t,
) -> c_int {
    crate::memicmp_core(v1, v2, len, locale as *mut c_void)
}

// ── _atoflt ──────────────────────────────────────────────────────────────────

/// `_atoflt` — `_atoflt_l(value, str, NULL)`. The engine (and the `_UNDERFLOW` report off
/// the narrowed `float` rather than the `double`) is wave 13's bignum parser.
#[no_mangle]
pub unsafe extern "C" fn _atoflt(value: *mut crate::fpnum::CrtFloat, str_: *mut c_char) -> c_int {
    crate::fpnum::_atoflt_l(value, str_ as *mut u8, core::ptr::null_mut())
}

// ── $I10_OUTPUT ──────────────────────────────────────────────────────────────

/// `#define I10_OUTPUT_MAX_PREC 21` (string.c:2663).
const I10_OUTPUT_MAX_PREC: c_int = 21;

/// `struct _I10_OUTPUT_DATA` (string.c:2665).
#[repr(C)]
pub struct _I10_OUTPUT_DATA {
    pub pos: i16,
    pub sign: c_char,
    pub len: u8,
    pub str: [c_char; I10_OUTPUT_MAX_PREC as usize + 1],
}

/// `typedef struct {UCHAR ld[10];} MSVCRT__LDOUBLE;` (msvcrt.h:84, the `__REACTOS__` arm).
///
/// **Ten bytes, not twelve.** Wine's `{ULONG x80[3]}` is the wrong size and overran the
/// buffer in msvcrt's own winetest, which is why ReactOS carries both types and copies
/// between them below.
#[repr(C)]
pub struct MSVCRT__LDOUBLE {
    pub ld: [u8; 10],
}

/// `typedef struct {ULONG x80[3];} WINE_BROKEN_LDOUBLE;` (msvcrt.h:85).
#[repr(C)]
struct WINE_BROKEN_LDOUBLE {
    x80: [u32; 3],
}

/// `$I10_OUTPUT` — print an x87 80-bit extended value into `data` for MSVC's `%Lf`.
///
/// Returns 0 for NaN/INF (with the `1#INF` / `1#QNAN` / `1#SNAN` string), 1 otherwise.
/// The C's own FIXME stands: native sets the last byte of `data->str` to '0' or '9' for
/// reasons nobody has worked out, and this — like the C — always leaves it '0'.
///
/// msvcrt.spec exports it as `$I10_OUTPUT` on i386 and x86_64 only; the body is compiled
/// unconditionally on arm64, where nothing can reach it.
#[no_mangle]
pub unsafe extern "C" fn I10_OUTPUT(
    ld80_: MSVCRT__LDOUBLE,
    mut prec: c_int,
    flag: c_int,
    data: *mut _I10_OUTPUT_DATA,
) -> c_int {
    // The ReactOS preamble: widen the 10-byte parameter into the 12-byte word view the
    // rest of the function indexes, leaving the top two bytes of x80[2] zero. `memcpy` is
    // the imported CRT one — a hand-written byte loop here is the classic LLVM
    // loop-idiom self-recursion.
    let mut ld80 = WINE_BROKEN_LDOUBLE { x80: [0; 3] };
    memcpy(
        core::ptr::addr_of_mut!(ld80) as *mut c_void,
        core::ptr::addr_of!(ld80_) as *const c_void,
        core::mem::size_of::<MSVCRT__LDOUBLE>(),
    );

    // Parenthesized because Rust binds `&` TIGHTER than `==`, the opposite of C: the
    // unparenthesized spelling is correct here and misleading to anyone reading it as C.
    if (ld80.x80[2] & 0x7fff) == 0x7fff {
        if ld80.x80[0] == 0 && ld80.x80[1] == 0x8000_0000 {
            strcpy((*data).str.as_mut_ptr(), c_str(b"1#INF\0"));
        } else if (ld80.x80[1] & 0x4000_0000) != 0 {
            strcpy((*data).str.as_mut_ptr(), c_str(b"1#QNAN\0"));
        } else {
            strcpy((*data).str.as_mut_ptr(), c_str(b"1#SNAN\0"));
        }
        (*data).pos = 1;
        (*data).sign = if (ld80.x80[2] & 0x8000) != 0 { b'-' } else { b' ' } as c_char;
        (*data).len = strlen((*data).str.as_ptr()) as u8;
        return 0;
    }

    // `num.exp = (ld80.x80[2] & 0x7fff) - 0x3fff - 63;` — the operand is `ULONG`, so the
    // subtraction is UNSIGNED and only then narrowed to `int`. An exponent field of 0
    // therefore lands at -16254, which `fpnum_double` flushes to zero, rather than being
    // read as a huge positive.
    let mut d: f64 = 0.0;
    crate::fpnum::ld80_fpnum_double(
        if (ld80.x80[2] & 0x8000) != 0 { -1 } else { 1 },
        (ld80.x80[2] & 0x7fff).wrapping_sub(0x3fff).wrapping_sub(63) as c_int,
        ld80.x80[0] as u64 | ((ld80.x80[1] as u64) << 32),
        &mut d,
    );

    if d < 0.0 {
        (*data).sign = b'-' as c_char;
        d = -d;
    } else {
        (*data).sign = b' ' as c_char;
    }

    if (flag & 1) != 0 {
        // `int exp = 1 + floor(log10(d));`
        //
        // `d` is INFINITE whenever `fpnum_double` overflowed (any x87 exponent field
        // above ~0x40FE does it), and C's `(int)` of a non-finite double is UB that the
        // targets already answer differently: x86's `cvttsd2si` yields INT_MIN, arm64's
        // `fcvtzs` saturates to INT_MAX. Rust's `as` saturates, i.e. it agrees with the
        // arm64 C. Both extremes converge two lines down — whichever way `exp` goes,
        // `prec` wraps deeply negative and the `prec < 0` arm forces `d = 0.0, prec = 0`
        // — so the emitted digits are the same either way. `wrapping_*` because that is
        // signed overflow on exactly that path.
        let exp = (1.0 + floor(log10(d))) as c_int;

        prec = prec.wrapping_add(exp);
        if exp < 0 {
            prec = prec.wrapping_sub(1);
        }
    }
    prec = prec.wrapping_sub(1);

    if prec.wrapping_add(1) > I10_OUTPUT_MAX_PREC {
        prec = I10_OUTPUT_MAX_PREC - 1;
    } else if prec < 0 {
        d = 0.0;
        prec = 0;
    }
    // From here `prec` is in 0..=20, which is what keeps every index below inside `buf`.

    let mut format = [0 as c_char; 8];
    let mut buf = [0 as c_char; I10_OUTPUT_MAX_PREC as usize + 9]; // 9 = "0.e+0000" + NUL
    sprintf(format.as_mut_ptr(), c_str(b"%%.%dle\0"), prec);
    sprintf(buf.as_mut_ptr(), format.as_ptr(), d);

    // `%.*le` gives `D.DDDe+XXX`, so overwriting the '.' with the leading digit leaves the
    // mantissa contiguous at buf+1 and puts the exponent's sign at buf+prec+3.
    buf[1] = buf[0];
    (*data).pos = crate::atoi(buf.as_ptr().add(prec as usize + 3) as *const u8) as i16;
    if buf[1] != b'0' as c_char {
        (*data).pos = (*data).pos.wrapping_add(1);
    }

    let mut p = buf.as_ptr().add(prec as usize + 1);
    while p > buf.as_ptr().add(1) && *p == b'0' as c_char {
        p = p.sub(1);
    }
    (*data).len = p.offset_from(buf.as_ptr()) as u8;

    memcpy(
        (*data).str.as_mut_ptr() as *mut c_void,
        buf.as_ptr().add(1) as *const c_void,
        (*data).len as size_t,
    );
    // Pointer write, not `str[len]`: an index would add a bounds check the C does not
    // have. (`len <= prec + 1 <= 21` here, so it is in range — but the habit is the one
    // that stops a bindgen `[T; 1]` from panicking somewhere else.)
    *(*data).str.as_mut_ptr().add((*data).len as usize) = 0;

    // The trailing zeros stripped above are restored PAST the NUL, where native leaves
    // them. `data->len` is a BYTE, so it widens to `int` for this arithmetic.
    if buf[1] != b'0' as c_char && prec - (*data).len as c_int + 1 > 0 {
        memcpy(
            (*data).str.as_mut_ptr().add((*data).len as usize + 1) as *mut c_void,
            buf.as_ptr().add((*data).len as usize + 1) as *const c_void,
            (prec - (*data).len as c_int + 1) as size_t,
        );
    }

    1
}

/// A NUL-terminated literal as `*const c_char`. `c_char` is signed on all three targets,
/// so the byte-string spelling keeps the literal readable without a per-site cast.
#[inline]
fn c_str(s: &'static [u8]) -> *const c_char {
    s.as_ptr() as *const c_char
}
