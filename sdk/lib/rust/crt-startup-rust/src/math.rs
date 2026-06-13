//! Math + ctype leaves: sincos (math/sincos.c), round[f] (math/round[f].c),
//! exp2[f] (math/exp2[f].c), sqrtf (math/i386/sqrtf.c, i386 only),
//! isblank (misc/isblank.c), iswblank (misc/iswblank.c).
//!
//! The transcendental kernels delegate to msvcrt's libm; only the thin
//! wrappers live here (matching the C, which exists to satisfy Clang's
//! optimizer / GCC 11). i386 builds no-SSE (`.cargo/config.toml`), so the
//! f64/f32 arithmetic lowers to x87 — same as the C msvcrtex compile.

extern "C" {
    fn sin(x: f64) -> f64;
    fn cos(x: f64) -> f64;
    fn pow(b: f64, e: f64) -> f64;
    fn powf(b: f32, e: f32) -> f32;
    fn ceil(x: f64) -> f64;
    fn floor(x: f64) -> f64;
    fn ceilf(x: f32) -> f32;
    fn floorf(x: f32) -> f32;
    #[cfg(target_arch = "x86")]
    fn sqrt(x: f64) -> f64;
    // ctype classification (msvcrt).
    fn _isctype(c: i32, mask: i32) -> i32;
    fn iswctype(c: u16, mask: u16) -> i32;
}

const _BLANK: i32 = 0x40;

// math/sincos.c
#[no_mangle]
pub unsafe extern "C" fn sincos(x: f64, s: *mut f64, c: *mut f64) {
    *s = sin(x);
    *c = cos(x);
}

// math/round.c
#[no_mangle]
pub unsafe extern "C" fn round(arg: f64) -> f64 {
    if arg < 0.0 {
        ceil(arg - 0.5)
    } else {
        floor(arg + 0.5)
    }
}

// math/roundf.c
#[no_mangle]
pub unsafe extern "C" fn roundf(arg: f32) -> f32 {
    if arg < 0.0 {
        ceilf(arg - 0.5)
    } else {
        floorf(arg + 0.5)
    }
}

// math/exp2[f].c — the base lives in a `static mut` (the Rust analogue of
// the C `static const TWO`): LLVM won't const-propagate a mutable static, so
// it can't fold pow(2,x) back into exp2 (→ self recursion). `black_box`
// can't be used here — its by-value f64/f32 ABI needs SSE, which i386 lacks.
static mut TWO_F64: f64 = 2.0;
static mut TWO_F32: f32 = 2.0;

#[no_mangle]
pub unsafe extern "C" fn exp2(x: f64) -> f64 {
    pow(TWO_F64, x)
}

#[no_mangle]
pub unsafe extern "C" fn exp2f(x: f32) -> f32 {
    powf(TWO_F32, x)
}

// math/i386/sqrtf.c
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn sqrtf(x: f32) -> f32 {
    sqrt(x as f64) as f32
}

// misc/isblank.c
#[no_mangle]
pub unsafe extern "C" fn isblank(c: i32) -> i32 {
    (_isctype(c, _BLANK) != 0 || c == b'\t' as i32) as i32
}

// misc/iswblank.c — wint_t is 16-bit on Windows.
#[no_mangle]
pub unsafe extern "C" fn iswblank(c: u16) -> i32 {
    (iswctype(c, _BLANK as u16) != 0 || c == b'\t' as u16) as i32
}
