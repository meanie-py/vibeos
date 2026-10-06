//! Hand-written cabi shims that have no differential target (pure
//! side-effect / handler-dispatch — nothing for cnt-difftest to pin).
//! NOT generated, NOT in cnt-manifest.toml / symbols.json.

use core::sync::atomic::{AtomicUsize, Ordering};

/// `void (__cdecl *)(const wchar_t*, const wchar_t*, const wchar_t*,
/// unsigned int, uintptr_t)` — the MSVCRT invalid-parameter handler.
pub type InvalidParameterHandler =
    unsafe extern "C" fn(*const u16, *const u16, *const u16, u32, usize);

/// Faithful to `_invalid_parameter.c`'s `static ... = NULL;` (a relaxed
/// atomic is behaviorally equivalent to the C non-atomic global for
/// this contract and keeps the shim sound).
static HANDLER: AtomicUsize = AtomicUsize::new(0);

/// `void __cdecl _invalid_parameter(expr, func, file, line, arg)`.
/// libcntpr (`_LIBCNT_`, `_MSVCR_VER` unset): if a handler is
/// installed, invoke it; otherwise the C only emits a debug `ERR(...)`
/// and returns — it does NOT `RaiseException` (that path is
/// `#if _MSVCR_VER > 0`). So the faithful behavior with no handler is
/// a no-op return (the kernel's debug print has no observable contract
/// and isn't part of the translation surface).
#[no_mangle]
pub unsafe extern "C" fn _invalid_parameter(
    expr: *const u16,
    func: *const u16,
    file: *const u16,
    line: u32,
    arg: usize,
) {
    let h = HANDLER.load(Ordering::Relaxed);
    if h != 0 {
        let f: InvalidParameterHandler = core::mem::transmute(h);
        f(expr, func, file, line, arg);
    }
}

/// `_invalid_parameter_handler __cdecl _get_invalid_parameter_handler(void)`
#[no_mangle]
pub unsafe extern "C" fn _get_invalid_parameter_handler() -> usize {
    HANDLER.load(Ordering::Relaxed)
}

/// `_invalid_parameter_handler __cdecl _set_invalid_parameter_handler(h)`
/// — returns the previous handler (faithful to the C).
#[no_mangle]
pub unsafe extern "C" fn _set_invalid_parameter_handler(handler: usize) -> usize {
    HANDLER.swap(handler, Ordering::Relaxed)
}

// ===================================================================
// scanf family — sscanf/swscanf/_snscanf/_snwscanf.
//
// The public symbols are irreducibly C-variadic and Rust cannot DEFINE
// a variadic fn on stable (the `c_variadic` feature is nightly; this
// project's discipline is "declare, never define" variadics). So a
// tiny C trampoline (src/cabi/scanf_trampoline.c) owns `va_start`/
// `va_end` and calls these non-variadic Rust entry points with a
// `va_list*` (opaque `*mut c_void` here — never dereferenced in Rust).
// Each assignment target is fetched via the C helper `cnt_va_next_ptr`
// (one `va_arg(*ap, void*)` — every scanf target is a pointer, so the
// ABI slot is uniform) at the EXACT per-directive point the genuine
// scanf.h does its `va_arg`. This mirrors genuine scanf.c, which is
// itself a 4-line `va_start; vsscanf_l(); va_end` shell over the
// engine. All 4 (cnt_vsscanf/cnt_vswscanf/cnt_vsnscanf/cnt_vsnwscanf)
// are present; the trampoline provides the matching sscanf/swscanf/
// _snscanf/_snwscanf.
// ===================================================================

// No longer `float`-gated: the engine + this cabi half are no-SSE-safe
// (the float synth in store_float is pure f64 arithmetic + a memory
// store — no by-value f64 register return), so scanf is the Rust
// provider on ALL 3 arches (i386 included). libcntpr.cmake compiles
// scanf_trampoline.c and REMOVE_ITEMs the C scanf.c — Rust owns
// sscanf/swscanf/_snscanf/_snwscanf in the kernel.
use crate::core_rs::scanf::{scanf_engine, IntWidth, Sink, TargetKind};

extern "C" {
    /// `void *cnt_va_next_ptr(va_list *ap)` — one `va_arg(*ap, void*)`.
    fn cnt_va_next_ptr(ap: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
}

struct VaSink {
    ap: *mut core::ffi::c_void,
    tgt: *mut u8,
    idx: usize,
    wide: bool,
}

impl Sink for VaSink {
    fn store_int(&mut self, w: IntWidth, value: u64) {
        let p = unsafe { cnt_va_next_ptr(self.ap) };
        unsafe {
            match w {
                IntWidth::I64 => *(p as *mut i64) = value as i64,
                IntWidth::Long | IntWidth::Int => *(p as *mut i32) = value as i32,
                IntWidth::Short => *(p as *mut i16) = value as i16,
            }
        }
    }
    fn store_float(&mut self, negative: bool, exp: i32, d: u64, is_double: bool) {
        // Faithful to winesup.c::_internal_handle_float. Genuine uses
        // x87 `long double`; this synth is f64 (no f80 in Rust) — pinned
        // by cnt-difftest `diff_scanf_float` against the genuine logic
        // built `-mlong-double-64` (long double == double == f64), so
        // bit-exact on amd64/arm64 + the documented i386 f80 deviation
        // (same "all-3 f64" call as strtod). No-SSE-safe: pure f64
        // arithmetic + a memory store, no by-value f64 register return.
        let mut curv: f64 = 1.0;
        let mut expcnt: f64 = 10.0;
        let mut e = exp;
        let negexp = e < 0;
        if negexp {
            // C `exp = -exp;` is signed UB at INT_MIN (gcc wraps to
            // INT_MIN); wrapping_neg matches it (the loop below still
            // terminates: `e /= 2` walks INT_MIN → 0).
            e = e.wrapping_neg();
        }
        while e != 0 {
            if e & 1 != 0 {
                curv *= expcnt;
            }
            e /= 2;
            expcnt *= expcnt;
        }
        let mut val = if negexp { d as f64 / curv } else { d as f64 * curv };
        if negative {
            val = -val;
        }
        let p = unsafe { cnt_va_next_ptr(self.ap) };
        unsafe {
            if is_double {
                *(p as *mut f64) = val;
            } else {
                *(p as *mut f32) = val as f32;
            }
        }
    }
    fn store_n(&mut self, value: i32) {
        let p = unsafe { cnt_va_next_ptr(self.ap) };
        unsafe { *(p as *mut i32) = value };
    }
    fn begin_target(&mut self, kind: TargetKind) {
        self.tgt = unsafe { cnt_va_next_ptr(self.ap) } as *mut u8;
        self.idx = 0;
        self.wide = kind == TargetKind::Wide;
    }
    fn push(&mut self, e: u32) {
        unsafe {
            if self.wide {
                *(self.tgt as *mut u16).add(self.idx) = e as u16;
            } else {
                *self.tgt.add(self.idx) = e as u8;
            }
        }
        self.idx += 1;
    }
    fn terminate(&mut self) {
        unsafe {
            if self.wide {
                *(self.tgt as *mut u16).add(self.idx) = 0;
            } else {
                *self.tgt.add(self.idx) = 0;
            }
        }
    }
}

/// `int sscanf(const char*, const char*, ...)` engine half — called by
/// the C trampoline with the live `va_list*`. NUL-bounded input.
#[no_mangle]
pub unsafe extern "C" fn cnt_vsscanf(
    file: *const u8,
    fmt: *const u8,
    ap: *mut core::ffi::c_void,
) -> i32 {
    let input = super::cstr_slice(file);
    let format = super::cstr_slice(fmt);
    let mut sink = VaSink { ap, tgt: core::ptr::null_mut(), idx: 0, wide: false };
    scanf_engine::<u8, _>(input, false, format, &mut sink)
}

/// `int swscanf(const wchar_t*, const wchar_t*, ...)` engine half.
#[no_mangle]
pub unsafe extern "C" fn cnt_vswscanf(
    file: *const u16,
    fmt: *const u16,
    ap: *mut core::ffi::c_void,
) -> i32 {
    let input = super::wcstr_slice(file);
    let format = super::wcstr_slice(fmt);
    let mut sink = VaSink { ap, tgt: core::ptr::null_mut(), idx: 0, wide: false };
    scanf_engine::<u16, _>(input, false, format, &mut sink)
}

/// `int _snscanf(const char*, size_t, const char*, ...)` engine half.
/// Input is exactly `length` bytes (NOT NUL-bounded — `_snscanf` can
/// read embedded NULs up to `length`); the engine's `has_len_limit`
/// yields the genuine `consumed==length ? '\0'` (no advance) quirk.
#[no_mangle]
pub unsafe extern "C" fn cnt_vsnscanf(
    file: *const u8,
    length: usize,
    fmt: *const u8,
    ap: *mut core::ffi::c_void,
) -> i32 {
    let input: &[u8] = if file.is_null() {
        &[]
    } else {
        core::slice::from_raw_parts(file, length)
    };
    let format = super::cstr_slice(fmt);
    let mut sink = VaSink { ap, tgt: core::ptr::null_mut(), idx: 0, wide: false };
    scanf_engine::<u8, _>(input, true, format, &mut sink)
}

/// `int _snwscanf(const wchar_t*, size_t, const wchar_t*, ...)` half.
#[no_mangle]
pub unsafe extern "C" fn cnt_vsnwscanf(
    file: *const u16,
    length: usize,
    fmt: *const u16,
    ap: *mut core::ffi::c_void,
) -> i32 {
    let input: &[u16] = if file.is_null() {
        &[]
    } else {
        core::slice::from_raw_parts(file, length)
    };
    let format = super::wcstr_slice(fmt);
    let mut sink = VaSink { ap, tgt: core::ptr::null_mut(), idx: 0, wide: false };
    scanf_engine::<u16, _>(input, true, format, &mut sink)
}

// ===================================================================
// env family — getenv/_wgetenv/_putenv/_wputenv + the 4 environment
// pointer accessors (__p__environ/__p__wenviron/__p___initenv/
// __p___winitenv).
//
// Faithful to ReactOS `stdlib/getenv.c` + `misc/environ.c::SetEnv`.
// Like scanf/strtok this is irreducibly STATEFUL (a process-global
// environment block, plus getenv returns an interior pointer into it)
// so it's hand-written here, NOT generated (no manifest rows /
// symbols.json — the _invalid_parameter precedent). core_rs::env owns
// the pure decisions; this layer owns the owned block + the
// realloc / duplicate-on-first-write / dual narrow+wide sync.
//
// State model (single-threaded-equivalent, mirroring environ.c's
// non-atomic globals — the strtok AtomicPtr precedent):
//  * Wide is the source of truth (SetEnv operates on wide); the narrow
//    block is the CP1252 image of it (the Wave E2 mbcp model — what
//    WideCharToMultiByte(CP_ACP) yields for the default ANSI CP).
//  * A FROZEN seed copy backs __initenv/__winitenv; the working copy
//    backs _environ/_wenviron. Before the first SetEnv they are the
//    SAME pointer array (`_environ == __initenv`); the first SetEnv
//    flips _environ/_wenviron to the independent working arrays
//    (genuine DuplicateEnvironment) — __initenv stays frozen.
//  * Fixed-capacity arena (MAXE entries × E* units). The genuine
//    malloc/realloc-failure → -1 paths are a documented unfuzzed
//    domain limit (the null_ret / strpbrk≤2 precedent); within the
//    arena the full add/replace/remove/asymmetry contract is exact.
// ===================================================================

use crate::core_rs::env as cenv;
use crate::core_rs::mbcp;

#[cfg(not(feature = "small-env"))]
const MAXE: usize = 48; // max env entries (arena bound; realloc-fail unfuzzed)
// The boot loaders never set an environment; see cnt-rust's `small-env` feature.
#[cfg(feature = "small-env")]
const MAXE: usize = 2;
const EW: usize = 384; // u16 units per entry incl '=' and NUL
const EN: usize = 384; // bytes per entry incl '=' and NUL

struct EnvState {
    // working (mutable) — backs _environ/_wenviron after dup.
    wbuf: [[u16; EW]; MAXE],
    nbuf: [[u8; EN]; MAXE],
    wlen: [usize; MAXE], // units, NUL excluded
    nlen: [usize; MAXE], // bytes, NUL excluded
    count: usize,
    // frozen seed — backs __initenv/__winitenv forever.
    iwbuf: [[u16; EW]; MAXE],
    inbuf: [[u8; EN]; MAXE],
    iwlen: [usize; MAXE],
    inlen: [usize; MAXE],
    icount: usize,
    duped: bool, // false ⇒ _environ == __initenv (pre first SetEnv)
    // NULL-terminated pointer arrays handed back via __p__*.
    wptr_env: [*mut u16; MAXE + 1],
    nptr_env: [*mut u8; MAXE + 1],
    wptr_init: [*mut u16; MAXE + 1],
    nptr_init: [*mut u8; MAXE + 1],
    // the `_environ`/`_wenviron`/`__initenv`/`__winitenv` cells; the
    // __p__* accessors return &of these (genuine `char ***`).
    environ: *mut *mut u8,
    wenviron: *mut *mut u16,
    initenv: *mut *mut u8,
    winitenv: *mut *mut u16,
}

impl EnvState {
    const fn new() -> Self {
        EnvState {
            wbuf: [[0; EW]; MAXE],
            nbuf: [[0; EN]; MAXE],
            wlen: [0; MAXE],
            nlen: [0; MAXE],
            count: 0,
            iwbuf: [[0; EW]; MAXE],
            inbuf: [[0; EN]; MAXE],
            iwlen: [0; MAXE],
            inlen: [0; MAXE],
            icount: 0,
            duped: false,
            wptr_env: [core::ptr::null_mut(); MAXE + 1],
            nptr_env: [core::ptr::null_mut(); MAXE + 1],
            wptr_init: [core::ptr::null_mut(); MAXE + 1],
            nptr_init: [core::ptr::null_mut(); MAXE + 1],
            environ: core::ptr::null_mut(),
            wenviron: core::ptr::null_mut(),
            initenv: core::ptr::null_mut(),
            winitenv: core::ptr::null_mut(),
        }
    }
}

struct EnvCell(core::cell::UnsafeCell<EnvState>);
// Single-threaded-equivalent: the genuine globals are non-atomic and
// the diff drives one deterministic call sequence (strtok precedent).
unsafe impl Sync for EnvCell {}
static ENV: EnvCell = EnvCell(core::cell::UnsafeCell::new(EnvState::new()));

#[inline]
unsafe fn env<'a>() -> &'a mut EnvState {
    &mut *ENV.0.get()
}

/// Rebuild the working pointer arrays from the working entries and the
/// frozen init arrays from the seed; (re)link _environ/_wenviron to
/// init (pre-dup) or working (post-dup). __initenv/__winitenv always
/// point at the frozen seed arrays.
unsafe fn env_relink(e: &mut EnvState) {
    for i in 0..e.count {
        e.nptr_env[i] = e.nbuf[i].as_mut_ptr();
        e.wptr_env[i] = e.wbuf[i].as_mut_ptr();
    }
    e.nptr_env[e.count] = core::ptr::null_mut();
    e.wptr_env[e.count] = core::ptr::null_mut();
    for i in 0..e.icount {
        e.nptr_init[i] = e.inbuf[i].as_mut_ptr();
        e.wptr_init[i] = e.iwbuf[i].as_mut_ptr();
    }
    e.nptr_init[e.icount] = core::ptr::null_mut();
    e.wptr_init[e.icount] = core::ptr::null_mut();
    e.initenv = e.nptr_init.as_mut_ptr();
    e.winitenv = e.wptr_init.as_mut_ptr();
    if e.duped {
        e.environ = e.nptr_env.as_mut_ptr();
        e.wenviron = e.wptr_env.as_mut_ptr();
    } else {
        // genuine: _environ == __initenv until the first SetEnv.
        e.environ = e.nptr_init.as_mut_ptr();
        e.wenviron = e.wptr_init.as_mut_ptr();
    }
}

/// Test/init seed. `block` is a Win32 `GetEnvironmentStringsW`-format
/// double-NUL-terminated wide block (`KEY=VAL\0KEY=VAL\0\0`). Mirrors
/// `BlockEnvToEnvironW`+`BlockEnvToEnvironA`: entries whose first unit
/// is `=` (drive-letter settings) are skipped; the narrow image is the
/// CP1252 encoding (== what GetEnvironmentStringsA would return for a
/// CP1252-clean block). Resets to the un-duped state.
#[no_mangle]
pub unsafe extern "C" fn cnt_env_seed_w(block: *const u16) {
    let e = env();
    *e = EnvState::new();
    let mut p = block;
    let mut n = 0usize;
    if !p.is_null() {
        while *p != 0 && n < MAXE {
            // one entry up to its NUL
            let mut len = 0usize;
            while *p.add(len) != 0 {
                len += 1;
            }
            let entry = core::slice::from_raw_parts(p, len);
            if entry.first().copied() != Some(b'=' as u16) {
                let cap = len.min(EW - 1);
                e.iwbuf[n][..cap].copy_from_slice(&entry[..cap]);
                e.iwbuf[n][cap] = 0;
                e.iwlen[n] = cap;
                e.wbuf[n][..cap].copy_from_slice(&entry[..cap]);
                e.wbuf[n][cap] = 0;
                e.wlen[n] = cap;
                let mut bn = 0usize;
                for &w in &entry[..cap] {
                    if bn >= EN - 1 {
                        break;
                    }
                    e.inbuf[n][bn] = mbcp::enc(w).unwrap_or(b'?');
                    e.nbuf[n][bn] = e.inbuf[n][bn];
                    bn += 1;
                }
                e.inbuf[n][bn] = 0;
                e.inlen[n] = bn;
                e.nbuf[n][bn] = 0;
                e.nlen[n] = bn;
                n += 1;
            }
            p = p.add(len + 1);
        }
    }
    e.count = n;
    e.icount = n;
    e.duped = false;
    env_relink(e);
}

/// Internal `SetEnv(const wchar_t *option)` (environ.c). `opt` is the
/// option, NUL excluded. Returns the genuine int (0 / -1).
unsafe fn set_env(opt: &[u16]) -> i32 {
    let e = env();
    let so = match cenv::classify_set(opt, b'=' as u16) {
        None => return -1, // NULL / no '=' ⇒ genuine -1
        Some(s) => s,
    };
    let name_len = so.eq_index;

    // Duplicate-on-first-write (both narrow+wide flip together since
    // the seed sets _environ==__initenv AND _wenviron==__winitenv).
    if !e.duped {
        e.duped = true;
    }

    // Find an existing entry by the SetEnv prefix rule (asymmetry vs
    // getenv: prefix compare of name_len units over the WHOLE entry).
    let mut found: Option<usize> = None;
    for i in 0..e.count {
        if cenv::set_entry_matches(&e.wbuf[i][..e.wlen[i]], opt, name_len) {
            found = Some(i);
            break;
        }
    }

    if so.remove {
        if let Some(idx) = found {
            for j in idx..e.count - 1 {
                e.wbuf[j] = e.wbuf[j + 1];
                e.nbuf[j] = e.nbuf[j + 1];
                e.wlen[j] = e.wlen[j + 1];
                e.nlen[j] = e.nlen[j + 1];
            }
            e.count -= 1;
        }
        // remove of a missing entry ⇒ genuine returns 0;
        // SetEnvironmentVariableW(name,NULL) stub ⇒ success.
        env_relink(e);
        return 0;
    }

    // set/replace: store the FULL option ("NAME=value") wide, plus its
    // CP1252 narrow image.
    let wcap = opt.len().min(EW - 1);
    let mut wtmp = [0u16; EW];
    wtmp[..wcap].copy_from_slice(&opt[..wcap]);
    let mut ntmp = [0u8; EN];
    let mut nl = 0usize;
    for &w in &opt[..wcap] {
        if nl >= EN - 1 {
            break;
        }
        ntmp[nl] = mbcp::enc(w).unwrap_or(b'?');
        nl += 1;
    }

    let idx = match found {
        Some(i) => i,
        None => {
            if e.count >= MAXE {
                return -1; // arena bound (genuine realloc-fail; unfuzzed)
            }
            let i = e.count;
            e.count += 1;
            i
        }
    };
    e.wbuf[idx] = wtmp;
    e.wlen[idx] = wcap;
    e.nbuf[idx] = ntmp;
    e.nlen[idx] = nl;
    env_relink(e);
    0 // SetEnvironmentVariableW(name, epos+1) stub ⇒ success
}

/// `char *getenv(const char *name)` — walk `*__p__environ()`, split at
/// the first `=`, exact-name-length + NT-ASCII-fold match, return the
/// interior pointer just past `=` (or NULL).
#[no_mangle]
pub unsafe extern "C" fn getenv(name: *const u8) -> *mut u8 {
    let e = env();
    let nm = super::cstr_slice(name);
    let mut arr = e.environ;
    if arr.is_null() {
        return core::ptr::null_mut();
    }
    loop {
        let s = *arr;
        if s.is_null() {
            break;
        }
        let entry = super::cstr_slice(s);
        if let Some(v) = cenv::getenv_value_start(entry, nm, b'=') {
            return s.add(v);
        }
        arr = arr.add(1);
    }
    core::ptr::null_mut()
}

/// `wchar_t *_wgetenv(const wchar_t *name)` — wide twin.
#[no_mangle]
pub unsafe extern "C" fn _wgetenv(name: *const u16) -> *mut u16 {
    let e = env();
    let nm = super::wcstr_slice(name);
    let mut arr = e.wenviron;
    if arr.is_null() {
        return core::ptr::null_mut();
    }
    loop {
        let s = *arr;
        if s.is_null() {
            break;
        }
        let entry = super::wcstr_slice(s);
        if let Some(v) = cenv::getenv_value_start(entry, nm, b'=' as u16) {
            return s.add(v);
        }
        arr = arr.add(1);
    }
    core::ptr::null_mut()
}

/// `int _putenv(const char *val)` — widen via CP1252
/// (MultiByteToWideChar(CP_ACP) model) then SetEnv.
#[no_mangle]
pub unsafe extern "C" fn _putenv(val: *const u8) -> i32 {
    let v = super::cstr_slice(val);
    let mut w = [0u16; EW];
    let n = v.len().min(EW - 1);
    for (i, &b) in v[..n].iter().enumerate() {
        w[i] = mbcp::dec(b);
    }
    set_env(&w[..n])
}

/// `int _wputenv(const wchar_t *val)` — SetEnv directly.
#[no_mangle]
pub unsafe extern "C" fn _wputenv(val: *const u16) -> i32 {
    let v = super::wcstr_slice(val);
    set_env(v)
}

/// `char ***__p__environ(void)` — &_environ.
#[no_mangle]
pub unsafe extern "C" fn __p__environ() -> *mut *mut *mut u8 {
    core::ptr::addr_of_mut!(env().environ)
}

/// `wchar_t ***__p__wenviron(void)` — &_wenviron.
#[no_mangle]
pub unsafe extern "C" fn __p__wenviron() -> *mut *mut *mut u16 {
    core::ptr::addr_of_mut!(env().wenviron)
}

/// `char ***__p___initenv(void)` — &__initenv.
#[no_mangle]
pub unsafe extern "C" fn __p___initenv() -> *mut *mut *mut u8 {
    core::ptr::addr_of_mut!(env().initenv)
}

/// `wchar_t ***__p___winitenv(void)` — &__winitenv.
#[no_mangle]
pub unsafe extern "C" fn __p___winitenv() -> *mut *mut *mut u16 {
    core::ptr::addr_of_mut!(env().winitenv)
}

// ===================================================================
// float-cvt — _fcvt (Phase 1; _ecvt/_gcvt route through the genuine
// streamout sprintf and land in later phases). Faithful to
// `stdlib/fcvt.c` + `fcvtbuf.c`. The genuine `_fcvt` returns a
// non-reentrant `static char fcvt_buf[2*DBL_MAX_10_EXP+10]`; that
// static lives here (a no-unsafe core can't own a mutable static),
// so this is hand-written (no manifest rows — the ecvt/static-buffer
// precedent). The decimal arithmetic is core_rs::floatcvt.
// ===================================================================

// Float-cvt cabi (_fcvt/_ecvt/_gcvt/_gcvt_s). These take a by-value
// `f64` ARG (fine no-SSE — it arrives on the stack) and RETURN a
// `char*`, so they compile on the i386 no-SSE kernel now that
// core_rs::floatcvt is unconditional (no `float` gate). `pub use`
// re-exports for cabi/mod.rs `pub use manual::*`. _ecvt/_fcvt/_gcvt are
// CRT-only (no libcntpr .c), so the kernel link doesn't pull them —
// they ride along harmlessly + stay difftest-pinned on every config.
mod floatcvt_cabi {
use crate::core_rs::floatcvt as fcvt;

struct FcvtCell(core::cell::UnsafeCell<[u8; fcvt::FCVT_BUF]>);
unsafe impl Sync for FcvtCell {}
static FCVT_BUF: FcvtCell = FcvtCell(core::cell::UnsafeCell::new([0; fcvt::FCVT_BUF]));

/// `char *_fcvt(double value, int ndigits, int *decpt, int *sign)`.
/// Returns the non-reentrant static buffer; writes `*decpt`/`*sign`.
#[no_mangle]
pub unsafe extern "C" fn _fcvt(
    value: f64,
    ndigits: i32,
    decpt: *mut i32,
    sign: *mut i32,
) -> *mut u8 {
    let buf = &mut *FCVT_BUF.0.get();
    match fcvt::fcvtbuf(value, ndigits) {
        Ok(r) => {
            buf[..r.buf_len].copy_from_slice(&r.buf[..r.buf_len]);
            core::ptr::write_volatile(decpt, r.decpt);
            core::ptr::write_volatile(sign, r.sign);
        }
        Err((is_inf, dp, sg)) => {
            // genuine: `snprintf(buf, ndigits, "1.#QNAN"/"1.#INF");
            // *decpt=0; *sign=0;` — ndigits is the snprintf SIZE.
            // This NaN/Inf path is a documented UNFUZZED domain limit
            // (host-snprintf-with-odd-size dependent; the scanf-
            // extreme-exponent / strpbrk≤2 precedent). Modeled: the
            // common ndigits>0 truncation; ndigits<=0 ⇒ the C
            // `size_t` wrap writes the whole string.
            let s: &[u8] = if is_inf { b"1.#INF" } else { b"1.#QNAN" };
            let cap = if ndigits > 0 {
                ((ndigits - 1) as usize).min(s.len())
            } else {
                s.len()
            };
            buf[..cap].copy_from_slice(&s[..cap]);
            buf[cap] = 0;
            core::ptr::write_volatile(decpt, dp);
            core::ptr::write_volatile(sign, sg);
        }
    }
    buf.as_mut_ptr()
}

struct EcvtCell(core::cell::UnsafeCell<[u8; fcvt::ECVT_BUF]>);
unsafe impl Sync for EcvtCell {}
static ECVT_BUF: EcvtCell = EcvtCell(core::cell::UnsafeCell::new([0; fcvt::ECVT_BUF]));

/// `char *_ecvt(double value, int ndigits, int *decpt, int *sign)`.
/// `sprintf("%-+.*E")` + text surgery; non-reentrant static buffer.
#[no_mangle]
pub unsafe extern "C" fn _ecvt(
    value: f64,
    ndigits: i32,
    decpt: *mut i32,
    sign: *mut i32,
) -> *mut u8 {
    let buf = &mut *ECVT_BUF.0.get();
    core::ptr::write_volatile(decpt, 0);
    core::ptr::write_volatile(sign, 0);
    match fcvt::ecvt(value, ndigits) {
        Some(r) => {
            buf[..r.buf_len].copy_from_slice(&r.buf[..r.buf_len]);
            core::ptr::write_volatile(decpt, r.decpt);
            core::ptr::write_volatile(sign, r.sign);
        }
        None => {
            // NaN/Inf: documented UNFUZZED edge (the genuine `%-+.*E`
            // of NaN under format_float is "+1.#QNAN…" and ecvt's
            // `strncmp(s,"NaN",3)` is dead → finite-path garbage;
            // pinning those exact bytes is owed, not faked here).
            // *decpt/*sign already 0 (genuine inits them so).
            let s: &[u8] = if fcvt::is_nan(value) {
                b"1#QNAN"
            } else {
                b"1#INF"
            };
            buf[..s.len()].copy_from_slice(s);
            buf[s.len()] = 0;
        }
    }
    buf.as_mut_ptr()
}

/// `%.*g` rendered text (sign prefix + body) into `out`, NUL added.
/// Returns the char count written (NUL excluded) — the value
/// `_scprintf("%.*g",...)` yields. NaN/Inf is the documented unfuzzed
/// edge (mirrors _ecvt: a best-effort glyph, not pinned).
unsafe fn gcvt_render(out: *mut u8, number: f64, ndigit: i32) -> usize {
    let g = fcvt::format_g(number, ndigit);
    let mut n = 0usize;
    if g.special != 0 {
        let s: &[u8] = if g.special == 1 { b"1#QNAN" } else { b"1#INF" };
        if g.negative {
            *out = b'-';
            n = 1;
        }
        for &b in s {
            *out.add(n) = b;
            n += 1;
        }
        *out.add(n) = 0;
        return n;
    }
    if g.negative {
        *out = b'-';
        n = 1;
    }
    for i in 0..g.len {
        *out.add(n) = g.body[i];
        n += 1;
    }
    *out.add(n) = 0;
    n
}

/// `char *_gcvt(double number, int ndigit, char *buff)` — gcvt.c.
/// NULL buff → NULL; ndigit<0 → NULL; else `sprintf(buff,"%.*g")`.
/// (The genuine `*_errno()=EINVAL/ERANGE` is not part of the value
/// contract the diff pins — kernel `_LIBCNT_` ignores it.)
#[no_mangle]
pub unsafe extern "C" fn _gcvt(number: f64, ndigit: i32, buff: *mut u8) -> *mut u8 {
    if buff.is_null() {
        return core::ptr::null_mut();
    }
    if ndigit < 0 {
        return core::ptr::null_mut();
    }
    gcvt_render(buff, number, ndigit);
    buff
}

/// `int _gcvt_s(char *buff, size_t size, double number, int digits)`
/// — gcvt.c. Faithful incl. the genuine `digits>=size` /`len>size`
/// boundary (sprintf then writes `len+1` bytes, so `len==size` is the
/// preserved off-by-one). Returns 0 / EINVAL(22) / ERANGE(34).
#[no_mangle]
pub unsafe extern "C" fn _gcvt_s(
    buff: *mut u8,
    size: usize,
    number: f64,
    digits: i32,
) -> i32 {
    const EINVAL: i32 = 22;
    const ERANGE: i32 = 34;
    if buff.is_null() {
        return EINVAL;
    }
    if digits < 0 || digits as usize >= size {
        if size != 0 {
            *buff = 0;
        }
        return ERANGE;
    }
    // `len = _scprintf("%.*g", digits, number);`
    let g = fcvt::format_g(number, digits);
    let len = (g.negative as usize)
        + if g.special == 1 {
            6
        } else if g.special == 2 {
            5
        } else {
            g.len
        };
    if len > size {
        *buff = 0;
        return ERANGE;
    }
    gcvt_render(buff, number, digits);
    0
}
} // mod floatcvt_cabi

pub use floatcvt_cabi::*;
