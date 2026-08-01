#![no_builtins] // the #[no_mangle] libc-named shims must be CALLED, not folded to builtins.
//! Wave 17 — grade the **scanf engine** (`msvcrt-crt-rust/src/scanf.rs`) against the
//! GENUINE `dll/win32/msvcrt/scanf.c` + `scanf.h` it replaces.
//!
//! `scanf.h` is a template `scanf.c` `#include`s SIXTEEN times under every combination of
//! `WIDE_SCANF` / `SECURE` / `STRING` / `STRING_LEN` / `CONSOLE`. The Rust replaces all
//! sixteen with one body generic over `W: Width`, `S: Source` and `const SECURE: bool`.
//! Sixteen instantiations collapsing into one is exactly the shape where a single
//! mis-transcribed line moves *every* entry point at once, so all sixteen are graded, not
//! a representative sample.
//!
//! # How the two sides are made comparable
//!
//! ## The argument cursor
//!
//! Everything risky in this engine is the `va_arg` cursor — pointer-then-size under
//! SECURE, **both** skipped when `*`-suppressed. The shipping Rust walks a `*mut u8` over
//! 8-byte slots (`printf::va_next`), which IS the Windows-x64 `va_list`; the host's SysV
//! `va_list` is a register-save-area struct and the two cannot consume the same buffer.
//! So build.rs compiles the reference TU against `refstub_printf/<stdarg.h>` — the flat
//! `char *` model the pfflat printf probe already proved — and both sides are handed the
//! SAME hand-packed slot buffer. See build.rs's "reference TU #5" note.
//!
//! Every case's argument list is DERIVED from the directives the generator emitted, never
//! parsed back out of the rendered format: a format and an arg list that disagree is a
//! harness bug that looks exactly like an engine bug. Past the last modelled argument sits
//! a SPILL object, checked pristine after every call on both sides — if the model is one
//! slot short, that fires instead of a silent wrong answer.
//!
//! ## The character source
//!
//! The FILE and CONSOLE engines read through `fgetc`/`ungetc`/`fgetwc`/`ungetwc`/`_getch`/
//! `_ungetch`. Those are modelled ONCE, here, over a string-backed stream, and exported
//! twice: under the plain names (which is what the shipping Rust imports) and under
//! `msvcrt_ref_*` (which is what build.rs's scanf prelude renames the reference's calls
//! to). One model, two names — so neither side can bind to the other, and a mistake in the
//! model makes both sides wrong the same way rather than manufacturing a divergence.
//!
//! The stream's final `(position, pushback)` is part of the compared image. That is the
//! ONLY place the engine's single `_UNGETC_` is observable: on a string source `_UNGETC_`
//! is `file--; consumed--` on values the caller never sees, and it runs after the last
//! `%n` could have read `consumed`. Dropping it is invisible on strings and loud on FILEs.
//!
//! # What is compared, per case
//!
//! return value · errno · invalid-parameter count · every output object BYTE FOR BYTE with
//! an 8-byte guard on each side · the input buffer (a scanf must never write to it) · the
//! format buffer · the stream position and pushback. Scalar destinations get 16 bytes of
//! payload, not 4, so `%hhd` storing a full `int` is *measured* rather than assumed.
//!
//! # Two exclusions, honoured and printed
//!
//! 1. **`%[` with bytes >= 0x80, NARROW only.** The C passes `*format` — a signed `char` —
//!    as ntdll's `ULONG` bit index, so `%[\x80]` is `RtlSetBits(map, 0xFFFFFF80, 1)`: a
//!    wild write ~128 MB below a 32-byte heap block, range-checked only under an `ASSERT`
//!    a release ntdll compiles out. That is UB, not behaviour. Narrow scanset corpora stop
//!    at 0x7F. The WIDE engine has no such problem — `wchar_t` zero-extends and
//!    `_BITMAPSIZE_` is 65536 — so wide scansets run the full 16-bit range. refimpl.c's
//!    bitmap COUNTS every out-of-range index and `main` asserts the count is zero, so the
//!    exclusion cannot rot into a silently-graded wild write.
//! 2. **The wide CONSOLE engine.** `CONSOLE` does not `#undef _EOF_`, so `vcwscanf_l`
//!    tests `nch == WEOF` against a `_getch()` that reports end-of-input as `-1`: it never
//!    sees EOF and `_cwscanf(L"%s")` can spin forever. No row drives it. Everything about
//!    that engine except `_GETC_`/`_UNGETC_`/`_EOF_` is shared with `vcscanf_l`, which IS
//!    graded.
//!
//! There is NOT a third. A narrow format byte >= 0x80 looks like it should be one —
//! `_ISSPACE_(*format)` sign-extends the loop's `char` into `isspace` — but
//! `_isctype_l`'s guard is `if (c >= -1 && c <= 255)` and a negative `c` falls straight
//! through to `return 0` (ctype.c:174-192). Well defined, so high format bytes are in the
//! corpus (`high-format-byte`, required) rather than quietly dropped. The exclusion is
//! narrower than it first looks: it is the `%[` scanset INDEX specifically, not the
//! format byte in general.

extern crate msvcrt_crt_rust;

use core::ffi::{c_int, c_void};

/// `_locale_t`. Every row passes NULL — the case that resolves the AMBIENT locale.
type LocT = *mut c_void;

// ── the GENUINE compiled msvcrt C ─────────────────────────────────────────────────────
//
// `refs_*` and not `ref_*`: these are build.rs-generated forwarders to scanf.c's SIXTEEN
// `static` engines, which no rename prelude can reach (internal linkage). The 40 variadic
// shells around them are `refs_*` too, and are NOT graded here —
// tools/scan_varargs_shim_parity.py owns them.
extern "C" {
    fn refs_vsscanf_l(f: *const i8, fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn refs_vsscanf_s_l(f: *const i8, fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn refs_vswscanf_l(f: *const u16, fmt: *const u16, l: LocT, ap: *mut u8) -> c_int;
    fn refs_vswscanf_s_l(f: *const u16, fmt: *const u16, l: LocT, ap: *mut u8) -> c_int;
    fn refs_vsnscanf_l(f: *const i8, n: usize, fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn refs_vsnscanf_s_l(f: *const i8, n: usize, fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn refs_vsnwscanf_l(f: *const u16, n: usize, fmt: *const u16, l: LocT, ap: *mut u8) -> c_int;
    fn refs_vsnwscanf_s_l(f: *const u16, n: usize, fmt: *const u16, l: LocT, ap: *mut u8) -> c_int;
    fn refs_vfscanf_l(f: *mut c_void, fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn refs_vfscanf_s_l(f: *mut c_void, fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn refs_vfwscanf_l(f: *mut c_void, fmt: *const u16, l: LocT, ap: *mut u8) -> c_int;
    fn refs_vfwscanf_s_l(f: *mut c_void, fmt: *const u16, l: LocT, ap: *mut u8) -> c_int;
    fn refs_vcscanf_l(fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn refs_vcscanf_s_l(fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    // (refs_vcwscanf_l / refs_vcwscanf_s_l exist and are deliberately never called —
    //  exclusion 2 in the module docs.)
    fn refs___stdio_common_vsscanf(
        o: u64, f: *const i8, n: usize, fmt: *const i8, l: LocT, ap: *mut u8,
    ) -> c_int;
    fn refs___stdio_common_vswscanf(
        o: u64, f: *const u16, n: usize, fmt: *const u16, l: LocT, ap: *mut u8,
    ) -> c_int;
    fn refs___stdio_common_vfscanf(
        o: u64, f: *mut c_void, fmt: *const i8, l: LocT, ap: *mut u8,
    ) -> c_int;
    fn refs___stdio_common_vfwscanf(
        o: u64, f: *mut c_void, fmt: *const u16, l: LocT, ap: *mut u8,
    ) -> c_int;
}

// ── the SHIPPING Rust: the code under test ────────────────────────────────────────────
extern "C" {
    fn vsscanf_l(f: *const i8, fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn vsscanf_s_l(f: *const i8, fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn vswscanf_l(f: *const u16, fmt: *const u16, l: LocT, ap: *mut u8) -> c_int;
    fn vswscanf_s_l(f: *const u16, fmt: *const u16, l: LocT, ap: *mut u8) -> c_int;
    fn vsnscanf_l(f: *const i8, n: usize, fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn vsnscanf_s_l(f: *const i8, n: usize, fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn vsnwscanf_l(f: *const u16, n: usize, fmt: *const u16, l: LocT, ap: *mut u8) -> c_int;
    fn vsnwscanf_s_l(f: *const u16, n: usize, fmt: *const u16, l: LocT, ap: *mut u8) -> c_int;
    fn vfscanf_l(f: *mut c_void, fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn vfscanf_s_l(f: *mut c_void, fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn vfwscanf_l(f: *mut c_void, fmt: *const u16, l: LocT, ap: *mut u8) -> c_int;
    fn vfwscanf_s_l(f: *mut c_void, fmt: *const u16, l: LocT, ap: *mut u8) -> c_int;
    fn vcscanf_l(fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn vcscanf_s_l(fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn __stdio_common_vsscanf(
        o: u64, f: *const i8, n: usize, fmt: *const i8, l: LocT, ap: *mut u8,
    ) -> c_int;
    fn __stdio_common_vswscanf(
        o: u64, f: *const u16, n: usize, fmt: *const u16, l: LocT, ap: *mut u8,
    ) -> c_int;
    fn __stdio_common_vfscanf(o: u64, f: *mut c_void, fmt: *const i8, l: LocT, ap: *mut u8) -> c_int;
    fn __stdio_common_vfwscanf(
        o: u64, f: *mut c_void, fmt: *const u16, l: LocT, ap: *mut u8,
    ) -> c_int;
}

// ── side channels (SEPARATE storage per side) ─────────────────────────────────────────
extern "C" {
    fn ref_errno_get() -> c_int;
    fn ref_errno_set(v: c_int);
    fn ref_invalid_pmt_count() -> c_int;
    fn ref_invalid_pmt_reset();
    fn subj_errno_get() -> c_int;
    fn subj_errno_set(v: c_int);
    fn subj_invalid_pmt_count() -> c_int;
    fn subj_invalid_pmt_reset();
    /// refimpl.c's ntdll bitmap model: how many `RtlSetBits` calls asked for an index
    /// outside the map. Exclusion 1 says this must stay at zero.
    fn msvcrt_ref_bitmap_oob_count() -> u64;
    fn msvcrt_ref_bitmap_oob_reset();
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The model character source
//
// One implementation, two symbol names. `msvcrt_ref_*` is what build.rs's scanf prelude
// renames the reference's calls to; the plain names are what msvcrt-crt-rust's `extern`
// block imports. Defining the plain names here overrides glibc's for THIS binary only,
// which is why the Cargo.toml stanza gives this test its own `[[test]]`.
// ══════════════════════════════════════════════════════════════════════════════════════

struct Stream {
    /// Code units: bytes for the narrow engines, `wchar_t` for the wide ones.
    data: Vec<u16>,
    pos: usize,
    push: i32,
    has_push: bool,
}

static mut STREAM: Stream = Stream { data: Vec::new(), pos: 0, push: 0, has_push: false };

fn stream() -> &'static mut Stream {
    unsafe { &mut *core::ptr::addr_of_mut!(STREAM) }
}

fn stream_reset(data: &[u16]) {
    let s = stream();
    s.data.clear();
    s.data.extend_from_slice(data);
    s.pos = 0;
    s.push = 0;
    s.has_push = false;
}

/// The stream's final state, as bytes in the compared image. THIS is the ungetc oracle.
fn stream_image() -> Vec<u8> {
    let s = stream();
    let mut v = Vec::with_capacity(9);
    v.extend_from_slice(&(s.pos as u32).to_le_bytes());
    v.extend_from_slice(&s.push.to_le_bytes());
    v.push(s.has_push as u8);
    v
}

fn model_get(wide: bool) -> i32 {
    let s = stream();
    if s.has_push {
        s.has_push = false;
        return s.push;
    }
    if s.pos >= s.data.len() {
        return if wide { 0xFFFF } else { -1 };
    }
    let c = s.data[s.pos];
    s.pos += 1;
    if wide {
        c as i32
    } else {
        (c as u8) as i32
    }
}

/// `ungetc` / `ungetwc` / `_ungetch`: one slot, and a second push fails — the C stdio
/// contract. The engine only ever ungets once, so the failure arm is documentation, not a
/// graded path.
fn model_unget(c: i32, wide: bool) -> i32 {
    let s = stream();
    let eof = if wide { 0xFFFF } else { -1 };
    if c == eof || s.has_push {
        return eof;
    }
    s.push = c;
    s.has_push = true;
    c
}

macro_rules! stream_shims {
    ($plain:ident, $namespaced:ident, $body:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $plain() -> c_int {
            #[allow(clippy::redundant_closure_call)]
            $body()
        }
        #[no_mangle]
        pub unsafe extern "C" fn $namespaced() -> c_int {
            $plain()
        }
    };
}

stream_shims!(_getch, msvcrt_ref__getch, || model_get(false));

#[no_mangle]
pub unsafe extern "C" fn _ungetch(c: c_int) -> c_int {
    model_unget(c, false)
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_ref__ungetch(c: c_int) -> c_int {
    _ungetch(c)
}

#[no_mangle]
pub unsafe extern "C" fn fgetc(_f: *mut c_void) -> c_int {
    model_get(false)
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_ref_fgetc(f: *mut c_void) -> c_int {
    fgetc(f)
}

#[no_mangle]
pub unsafe extern "C" fn ungetc(c: c_int, _f: *mut c_void) -> c_int {
    model_unget(c, false)
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_ref_ungetc(c: c_int, f: *mut c_void) -> c_int {
    ungetc(c, f)
}

/// msvcrt's `wint_t` is 16-bit, so `WEOF` is 0xFFFF and `fgetwc` ZERO-extends into the
/// engine's `int nch` — it never arrives as -1. The shipping Rust declares this exact
/// `-> u16` for the same reason.
#[no_mangle]
pub unsafe extern "C" fn fgetwc(_f: *mut c_void) -> u16 {
    model_get(true) as u16
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_ref_fgetwc(f: *mut c_void) -> u16 {
    fgetwc(f)
}

#[no_mangle]
pub unsafe extern "C" fn ungetwc(c: u16, _f: *mut c_void) -> u16 {
    model_unget(c as i32, true) as u16
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_ref_ungetwc(c: u16, f: *mut c_void) -> u16 {
    ungetwc(c, f)
}

// `_lock_file`/`_unlock_file` are genuinely nothing here: this harness is single-threaded
// and the engine's only use of the FILE* is to hand it straight back. Modelling a lock
// would be modelling the harness, not msvcrt.
#[no_mangle]
pub unsafe extern "C" fn _lock_file(_f: *mut c_void) {}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_ref__lock_file(_f: *mut c_void) {}
#[no_mangle]
pub unsafe extern "C" fn _unlock_file(_f: *mut c_void) {}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_ref__unlock_file(_f: *mut c_void) {}

/// `stdin` is `&_iob[0]` in this tree (`sdk/include/crt/stdio.h:92,98`), i.e. the array
/// base — which is why the zero index disappears. Only `_lock_file`/`_unlock_file` ever
/// see it and both ignore it, so a distinct non-NULL address is a complete model.
static mut FAKE_IOB: [u8; 8] = [0; 8];
#[no_mangle]
pub unsafe extern "C" fn __iob_func() -> *mut c_void {
    core::ptr::addr_of_mut!(FAKE_IOB) as *mut c_void
}

/// The `FILE *` the FILE rows pass. Never dereferenced by either side.
fn fake_file() -> *mut c_void {
    unsafe { __iob_func() }
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The oracle
// ══════════════════════════════════════════════════════════════════════════════════════

/// Seeded before every call so a function that CLEARS errno is observable (zeroing first
/// makes a zeroing write invisible — classify.rs learned that the hard way).
const ERRNO_SEED: c_int = 0x5A5A;

#[derive(PartialEq, Debug)]
struct Effects {
    ret: i64,
    errno: c_int,
    pmt: c_int,
}

impl core::fmt::Display for Effects {
    fn fmt(&self, w: &mut core::fmt::Formatter) -> core::fmt::Result {
        let e = match self.errno {
            0 => "0".to_string(),
            ERRNO_SEED => "untouched".to_string(),
            22 => "EINVAL".to_string(),
            34 => "ERANGE".to_string(),
            42 => "EILSEQ".to_string(),
            n => format!("errno{n}"),
        };
        write!(
            w,
            "ret={} errno={} handler={}",
            self.ret,
            e,
            if self.pmt > 0 { format!("FIRED x{}", self.pmt) } else { "no".into() }
        )
    }
}

struct Obs {
    fx: Effects,
    img: Vec<u8>,
}

#[derive(PartialEq)]
enum V {
    Match,
    Diverge,
    Unexercised,
}

struct Res {
    name: String,
    v: V,
    note: String,
}

static mut RESULTS: Vec<Res> = Vec::new();
static mut CASES: u64 = 0;
static mut SPILL_HITS: u64 = 0;

fn record(name: String, v: V, note: String) {
    unsafe { (*core::ptr::addr_of_mut!(RESULTS)).push(Res { name, v, note }) }
}

// ── guarded buffers ───────────────────────────────────────────────────────────────────

/// Bytes of untouchable poison on EACH side of every destination, input and format. A
/// pre-buffer write is invisible without it — wave 16's controls proved that, and this
/// engine has two arms (`%s` truncation, `%c` overflow) that deliberately write to the
/// buffer START after walking away from it.
const GUARD: usize = 8;
const POISON: u8 = 0xAA;

struct Obj(Vec<u8>);

impl Obj {
    fn new(payload: usize) -> Obj {
        Obj(vec![POISON; GUARD + payload + GUARD])
    }
    fn seeded_u8(src: &[u8]) -> Obj {
        let mut o = Obj::new(src.len());
        o.0[GUARD..GUARD + src.len()].copy_from_slice(src);
        o
    }
    fn seeded_u16(src: &[u16]) -> Obj {
        let mut o = Obj::new(src.len() * 2);
        for (i, w) in src.iter().enumerate() {
            o.0[GUARD + i * 2..GUARD + i * 2 + 2].copy_from_slice(&w.to_le_bytes());
        }
        o
    }
    fn ptr(&mut self) -> *mut u8 {
        unsafe { self.0.as_mut_ptr().add(GUARD) }
    }
    fn pristine(&self) -> bool {
        self.0.iter().all(|b| *b == POISON)
    }
}

fn hex(v: &[u8]) -> String {
    let mut s = String::new();
    for (i, b) in v.iter().enumerate() {
        if i > 0 && i % 32 == 0 {
            s.push('|');
        }
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn first_diff(a: &[u8], b: &[u8]) -> Option<usize> {
    for i in 0..a.len().min(b.len()) {
        if a[i] != b[i] {
            return Some(i);
        }
    }
    if a.len() != b.len() {
        Some(a.len().min(b.len()))
    } else {
        None
    }
}

// ── deterministic RNG (wave16's) ──────────────────────────────────────────────────────
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 17
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() as usize) % n.max(1)
    }
    fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T {
        &v[self.below(v.len())]
    }
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The case model
// ══════════════════════════════════════════════════════════════════════════════════════

/// One entry in the packed `va_list`.
///
/// Derived from the directive that will consume it, never re-parsed out of the rendered
/// format. `Scalar` always gets SIXTEEN bytes of payload however narrow the C's store is,
/// so "`%hhd` writes a full `int` at `_MSVCR_VER == 0`" is measured rather than trusted.
#[derive(Clone, Copy, Debug)]
enum Arg {
    Scalar,
    Buf { wide: bool, elems: usize },
    /// The SECURE `unsigned size`, immediately AFTER its pointer.
    Size(u32),
}

struct Case {
    fmt: Vec<u32>,
    args: Vec<Arg>,
    input: Vec<u32>,
    /// STRING_LEN's `length`.
    length: usize,
    tags: Vec<&'static str>,
}

/// The live objects behind one side's call: the destinations, the input, the format, and
/// the spill.
struct Bound {
    objs: Vec<Obj>,
    input: Obj,
    fmt: Obj,
    spill: Obj,
    packed: Vec<u8>,
}

/// How many extra slots to pack past the last modelled argument, all pointing at `spill`.
///
/// A port that consumes one slot too many (the whole `*`-suppression and pointer-then-size
/// risk surface) would otherwise read whatever follows `packed` and store through it. This
/// gives it somewhere defined to land AND makes the mistake visible: `spill` is compared
/// like any other object, and `main` also asserts it stayed pristine on both sides — a
/// shared harness bug would be invisible in the diff alone.
const SPILL_SLOTS: usize = 6;

fn bind(case: &Case, wide_engine: bool) -> Bound {
    let mut objs = Vec::new();
    let mut packed: Vec<u8> = Vec::new();
    // 64 bytes of payload: room for any over-wide store a wrong port might make.
    let mut spill = Obj::new(64);
    let spill_ptr = spill.ptr();

    for a in &case.args {
        match *a {
            Arg::Scalar => {
                let mut o = Obj::new(16);
                packed.extend_from_slice(&(o.ptr() as usize).to_le_bytes());
                objs.push(o);
            }
            Arg::Buf { wide, elems } => {
                let mut o = Obj::new(elems * if wide { 2 } else { 1 });
                packed.extend_from_slice(&(o.ptr() as usize).to_le_bytes());
                objs.push(o);
            }
            Arg::Size(v) => packed.extend_from_slice(&(v as u64).to_le_bytes()),
        }
    }
    for _ in 0..SPILL_SLOTS {
        packed.extend_from_slice(&(spill_ptr as usize).to_le_bytes());
    }

    // Inputs and formats get 16 trailing NULs INSIDE the guarded region. The C walks past
    // the terminator in two places on purpose — an unterminated `%[` steps `format` past
    // its NUL, and a trailing bare `%` can re-read the input's — so the padding is what
    // makes those cases DEFINED and gradeable instead of merely reproduced.
    const PAD: usize = 16;
    let (input, fmt) = if wide_engine {
        let mut i: Vec<u16> = case.input.iter().map(|c| *c as u16).collect();
        i.extend(core::iter::repeat(0).take(PAD));
        let mut f: Vec<u16> = case.fmt.iter().map(|c| *c as u16).collect();
        f.extend(core::iter::repeat(0).take(PAD));
        (Obj::seeded_u16(&i), Obj::seeded_u16(&f))
    } else {
        let mut i: Vec<u8> = case.input.iter().map(|c| *c as u8).collect();
        i.extend(core::iter::repeat(0).take(PAD));
        let mut f: Vec<u8> = case.fmt.iter().map(|c| *c as u8).collect();
        f.extend(core::iter::repeat(0).take(PAD));
        (Obj::seeded_u8(&i), Obj::seeded_u8(&f))
    };

    Bound { objs, input, fmt, spill, packed }
}

impl Bound {
    fn image(self, with_stream: bool) -> Vec<u8> {
        let mut v = Vec::new();
        for o in self.objs {
            v.extend_from_slice(&o.0);
        }
        v.extend_from_slice(&self.spill.0);
        v.extend_from_slice(&self.input.0);
        v.extend_from_slice(&self.fmt.0);
        if with_stream {
            v.extend_from_slice(&stream_image());
        }
        v
    }
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Entry points
// ══════════════════════════════════════════════════════════════════════════════════════

type FStrA = unsafe extern "C" fn(*const i8, *const i8, LocT, *mut u8) -> c_int;
type FStrW = unsafe extern "C" fn(*const u16, *const u16, LocT, *mut u8) -> c_int;
type FStrNA = unsafe extern "C" fn(*const i8, usize, *const i8, LocT, *mut u8) -> c_int;
type FStrNW = unsafe extern "C" fn(*const u16, usize, *const u16, LocT, *mut u8) -> c_int;
type FFileA = unsafe extern "C" fn(*mut c_void, *const i8, LocT, *mut u8) -> c_int;
type FFileW = unsafe extern "C" fn(*mut c_void, *const u16, LocT, *mut u8) -> c_int;
type FConA = unsafe extern "C" fn(*const i8, LocT, *mut u8) -> c_int;

/// The `__stdio_common_*` doors, adapted to the shapes above. Routing them through the
/// same machinery grades the option-bit dispatch — the ONE thing those four bodies do —
/// for free: `_CRT_INTERNAL_SCANF_SECURECRT` picks the `_s` engine and every other bit is
/// a `FIXME` the Rust dropped.
const SCANF_SECURECRT: u64 = 0x0001;
const SCANF_LEGACY_WIDE: u64 = 0x0002;

macro_rules! common_adapter {
    ($name:ident, $target:ident, ($($a:ident : $t:ty),*), $opts:expr) => {
        unsafe extern "C" fn $name($($a: $t,)* l: LocT, ap: *mut u8) -> c_int {
            $target($opts, $($a,)* l, ap)
        }
    };
}
common_adapter!(sc_s_a, __stdio_common_vsscanf, (f: *const i8, n: usize, fmt: *const i8), SCANF_LEGACY_WIDE | SCANF_SECURECRT);
common_adapter!(sc_s_a_r, refs___stdio_common_vsscanf, (f: *const i8, n: usize, fmt: *const i8), SCANF_LEGACY_WIDE | SCANF_SECURECRT);
common_adapter!(sc_p_a, __stdio_common_vsscanf, (f: *const i8, n: usize, fmt: *const i8), SCANF_LEGACY_WIDE);
common_adapter!(sc_p_a_r, refs___stdio_common_vsscanf, (f: *const i8, n: usize, fmt: *const i8), SCANF_LEGACY_WIDE);
common_adapter!(sc_s_w, __stdio_common_vswscanf, (f: *const u16, n: usize, fmt: *const u16), SCANF_LEGACY_WIDE | SCANF_SECURECRT);
common_adapter!(sc_s_w_r, refs___stdio_common_vswscanf, (f: *const u16, n: usize, fmt: *const u16), SCANF_LEGACY_WIDE | SCANF_SECURECRT);
common_adapter!(sc_p_w, __stdio_common_vswscanf, (f: *const u16, n: usize, fmt: *const u16), SCANF_LEGACY_WIDE);
common_adapter!(sc_p_w_r, refs___stdio_common_vswscanf, (f: *const u16, n: usize, fmt: *const u16), SCANF_LEGACY_WIDE);
common_adapter!(sc_s_fa, __stdio_common_vfscanf, (f: *mut c_void, fmt: *const i8), SCANF_LEGACY_WIDE | SCANF_SECURECRT);
common_adapter!(sc_s_fa_r, refs___stdio_common_vfscanf, (f: *mut c_void, fmt: *const i8), SCANF_LEGACY_WIDE | SCANF_SECURECRT);
common_adapter!(sc_p_fw, __stdio_common_vfwscanf, (f: *mut c_void, fmt: *const u16), 0);
common_adapter!(sc_p_fw_r, refs___stdio_common_vfwscanf, (f: *mut c_void, fmt: *const u16), 0);

#[derive(Clone, Copy)]
enum Ep {
    StrA(FStrA, FStrA),
    StrW(FStrW, FStrW),
    StrNA(FStrNA, FStrNA),
    StrNW(FStrNW, FStrNW),
    FileA(FFileA, FFileA),
    FileW(FFileW, FFileW),
    ConA(FConA, FConA),
}

impl Ep {
    fn wide(&self) -> bool {
        matches!(self, Ep::StrW(..) | Ep::StrNW(..) | Ep::FileW(..))
    }
    /// Does this entry point read a stream (so the `_UNGETC_` is observable)?
    fn streamed(&self) -> bool {
        matches!(self, Ep::FileA(..) | Ep::FileW(..) | Ep::ConA(..))
    }
}

/// Run ONE side of ONE case.
unsafe fn run_side(ep: Ep, case: &Case, subject: bool) -> Obs {
    let mut b = bind(case, ep.wide());
    if ep.streamed() {
        // A FILE/CONSOLE stream has no terminator: end-of-data IS EOF. That is a genuinely
        // different source from a string, which stops on an embedded NUL, and both are
        // driven from the same corpus on purpose.
        let data: Vec<u16> = case.input.iter().map(|c| *c as u16).collect();
        stream_reset(&data);
    }
    let ip = b.input.ptr();
    let fp = b.fmt.ptr();
    let ap = b.packed.as_mut_ptr();
    let n = case.length;

    if subject {
        subj_errno_set(ERRNO_SEED);
        subj_invalid_pmt_reset();
    } else {
        ref_errno_set(ERRNO_SEED);
        ref_invalid_pmt_reset();
    }

    let ret = match ep {
        Ep::StrA(s, r) => {
            let f = if subject { s } else { r };
            f(ip as *const i8, fp as *const i8, core::ptr::null_mut(), ap)
        }
        Ep::StrW(s, r) => {
            let f = if subject { s } else { r };
            f(ip as *const u16, fp as *const u16, core::ptr::null_mut(), ap)
        }
        Ep::StrNA(s, r) => {
            let f = if subject { s } else { r };
            f(ip as *const i8, n, fp as *const i8, core::ptr::null_mut(), ap)
        }
        Ep::StrNW(s, r) => {
            let f = if subject { s } else { r };
            f(ip as *const u16, n, fp as *const u16, core::ptr::null_mut(), ap)
        }
        Ep::FileA(s, r) => {
            let f = if subject { s } else { r };
            f(fake_file(), fp as *const i8, core::ptr::null_mut(), ap)
        }
        Ep::FileW(s, r) => {
            let f = if subject { s } else { r };
            f(fake_file(), fp as *const u16, core::ptr::null_mut(), ap)
        }
        Ep::ConA(s, r) => {
            let f = if subject { s } else { r };
            f(fp as *const i8, core::ptr::null_mut(), ap)
        }
    };

    let fx = if subject {
        Effects { ret: ret as i64, errno: subj_errno_get(), pmt: subj_invalid_pmt_count() }
    } else {
        Effects { ret: ret as i64, errno: ref_errno_get(), pmt: ref_invalid_pmt_count() }
    };
    if !b.spill.pristine() {
        SPILL_HITS += 1;
    }
    Obs { fx, img: b.image(ep.streamed()) }
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Grading
// ══════════════════════════════════════════════════════════════════════════════════════

/// Outcome classes read off the REFERENCE's answer, so a `require` is a claim about
/// msvcrt's contract rather than about our implementation.
fn outcome_tags(fx: &Effects) -> Vec<&'static str> {
    let mut t = vec![];
    match fx.ret {
        -1 => t.push("eof-ret"),
        0 => t.push("zero"),
        _ => t.push("assigned"),
    }
    if fx.errno != ERRNO_SEED {
        t.push("errno-touched");
    }
    if fx.pmt > 0 {
        t.push("handler");
    }
    t
}

fn grade(
    name: &str,
    require: &[&'static str],
    iters: usize,
    mut f: impl FnMut(&mut Rng) -> (Obs, Obs, String, Vec<&'static str>),
) {
    if let Ok(only) = std::env::var("VIBEOS_SCANF_ROW") {
        if !name.contains(&only) {
            return;
        }
    }
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    for b in name.bytes() {
        seed = seed.wrapping_mul(0x100_0000_01B3).wrapping_add(b as u64);
    }
    let mut r = Rng(seed | 1);
    let mut seen: Vec<&'static str> = Vec::new();
    for _ in 0..iters {
        let (got, want, ctx, tags) = f(&mut r);
        unsafe { CASES += 1 };
        for t in tags {
            if !seen.contains(&t) {
                seen.push(t);
            }
        }
        if got.fx != want.fx || got.img != want.img {
            let mut note = format!("[{ctx}]\n        rust  = {}", got.fx);
            note.push_str(&format!("\n        msvcrt= {}", want.fx));
            if got.img != want.img {
                note.push_str(&format!("\n        rust  img={}", hex(&got.img)));
                note.push_str(&format!("\n        msvcrt img={}", hex(&want.img)));
                if let Some(i) = first_diff(&got.img, &want.img) {
                    note.push_str(&format!(
                        "\n        first differing byte: index {i} (objects, then spill, \
                         then input, then format, then stream; each object starts with \
                         {GUARD} guard bytes)"
                    ));
                }
            }
            record(name.to_string(), V::Diverge, note);
            return;
        }
    }
    let missing: Vec<&&str> = require.iter().filter(|w| !seen.contains(w)).collect();
    if !missing.is_empty() {
        record(
            name.to_string(),
            V::Unexercised,
            format!(
                "the fuzz never reached {missing:?} — this MATCH is worthless, the branch \
                 never ran.\n        reached: {seen:?}"
            ),
        );
        return;
    }
    record(name.to_string(), V::Match, format!("{iters} cases; reached {seen:?}"));
}

/// The one place a case is turned into a graded comparison.
unsafe fn run_case(ep: Ep, case: &Case) -> (Obs, Obs, String, Vec<&'static str>) {
    let got = run_side(ep, case, true);
    let want = run_side(ep, case, false);
    let mut tags = outcome_tags(&want.fx);
    for t in &case.tags {
        tags.push(t);
    }
    (got, want, show(case, ep.wide()), tags)
}

fn show(case: &Case, wide: bool) -> String {
    let esc = |v: &[u32]| {
        let mut s = String::new();
        for c in v {
            match *c {
                0x20..=0x7E => s.push(*c as u8 as char),
                9 => s.push_str("\\t"),
                10 => s.push_str("\\n"),
                13 => s.push_str("\\r"),
                c => s.push_str(&format!("\\x{c:02x}")),
            }
        }
        s
    };
    format!(
        "{}fmt=\"{}\" input=\"{}\" length={} args={:?}",
        if wide { "L" } else { "" },
        esc(&case.fmt),
        esc(&case.input),
        case.length,
        case.args
    )
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Generators
// ══════════════════════════════════════════════════════════════════════════════════════

/// The narrow input alphabet. Full 1..=0xFF: `_GETC_` casts to `(unsigned char)`, so a
/// high byte is in range for `isspace`/`RtlAreBitsSet` — unlike a high FORMAT byte, which
/// sign-extends and is excluded (module docs).
fn gen_input(r: &mut Rng, wide: bool, maxlen: usize) -> Vec<u32> {
    let n = 1 + r.below(maxlen);
    let mut v: Vec<u32> = Vec::with_capacity(n);
    while v.len() < n {
        match r.below(14) {
            0 => v.extend(b"0123456789"[..1 + r.below(9)].iter().map(|b| *b as u32)),
            1 => v.push(*r.pick(b" \t\n\r\x0b\x0c") as u32),
            2 => v.push(*r.pick(b"+-.eE") as u32),
            3 => v.extend(b"0x".iter().map(|b| *b as u32)),
            4 => v.extend(b"0X".iter().map(|b| *b as u32)),
            5 => v.push((b'a' + r.below(26) as u8) as u32),
            6 => v.push((b'A' + r.below(26) as u8) as u32),
            7 => v.push(*r.pick(b"0123456789abcdefABCDEF") as u32),
            8 => {
                // A float literal, the shape a general fuzzer essentially never emits.
                let lit: &str = *r.pick(&[
                    "1.5", "-0.25e3", "3.14159", "1e40", "0.0", "-1e-40", "12345.6789", "+7.",
                    ".5", "1e",
                ]);
                v.extend(lit.bytes().map(|b| b as u32));
            }
            9 => {
                // The 0xFF trap: narrow-STRING `_GETC_FUNC_` sign-extends where `_GETC_`
                // casts to unsigned char, so this byte reaches the float parser as WEOF
                // and comes back out as `nch == -1` on a source whose `_EOF_` is 0.
                v.push(0xFF);
            }
            10 => v.push(1 + r.below(0xFE) as u32),
            11 if wide => v.push(0x100 + r.below(0xFF00) as u32),
            12 => v.push(*r.pick(b"[]^-%*") as u32),
            _ => v.push((b'0' + r.below(10) as u8) as u32),
        }
    }
    v.truncate(n);
    v
}

/// The rendered form of one conversion, plus the arguments it will consume.
struct Conv {
    text: Vec<u32>,
    args: Vec<Arg>,
    tags: Vec<&'static str>,
    /// True when the C's format walk cannot get past this directive deterministically
    /// (an unterminated `%[` eats everything after it), so nothing may follow.
    terminal: bool,
}

const TYPES: &[u8] = b"diuoxXpPeEfgGsScCn[%q";
/// The same set without `[`. Used for the cases that carry a high format byte — see
/// `allow_high` in `gen_case`.
const TYPES_NO_SCANSET: &[u8] = b"diuoxXpPeEfgGsScCn%q";

fn gen_conv(r: &mut Rng, wide_engine: bool, secure: bool, inlen: usize, allow_high: bool) -> Conv {
    let mut text: Vec<u32> = vec![b'%' as u32];
    let mut args = Vec::new();
    let mut tags = Vec::new();

    let suppress = r.below(6) == 0;
    if suppress {
        text.push(b'*' as u32);
        tags.push("suppressed");
        if secure {
            // The C reads NEITHER the pointer NOR the size when suppressed. This is the
            // arm a one-slot-off cursor dies on.
            tags.push("secure-suppressed");
        }
    }

    // ── the width/prefix scan ────────────────────────────────────────────────────────
    //
    // The C re-runs the digit scan at the top of EVERY prefix pass, so digits and prefix
    // characters interleave and `%1h2d` accumulates a width of 12. Modelled by emitting
    // digit runs between prefix items and computing `width` from their concatenation, so
    // the harness's idea of the width comes from the same rule the engine uses.
    let mut digits = String::new();
    let mut h_count = 0i32;
    let mut l_pref = false;
    let mut big_l = false;
    let mut w_pref = false;
    let mut i64_pref = false;
    let mut seg_count = 0usize;

    let emit_digits = |text: &mut Vec<u32>, digits: &mut String, r: &mut Rng| {
        if r.below(3) == 0 {
            let n = 1 + r.below(2);
            for _ in 0..n {
                let d = if digits.is_empty() && r.below(2) == 0 {
                    b'1' + r.below(3) as u8 // avoid a leading 0 width most of the time
                } else {
                    b'0' + r.below(10) as u8
                };
                text.push(d as u32);
                digits.push(d as char);
            }
        }
    };

    let nseg = r.below(3);
    for _ in 0..nseg {
        emit_digits(&mut text, &mut digits, r);
        let item: &str = *r.pick(&["h", "l", "ll", "w", "L", "I64", "I32", "I", "z"]);
        for b in item.bytes() {
            text.push(b as u32);
        }
        match item {
            "h" => h_count += 1,
            "l" => l_pref = true,
            "ll" => {
                l_pref = true;
                i64_pref = true;
            }
            "w" => w_pref = true,
            "L" => big_l = true,
            "I64" => i64_pref = true,
            "I32" => {}
            // Bare `I` falls through to `case 'z'`, and `z` IS compiled at _MSVCR_VER == 0.
            // Both set I64 on a 64-bit host. NOTE: no digit run may follow a bare `I`, or
            // "I" + "64" would silently become the I64 prefix.
            "I" | "z" => i64_pref = true,
            _ => unreachable!(),
        }
        seg_count += 1;
        if item == "I" {
            break; // see above
        }
    }
    if seg_count == 0 || !matches!(text.last(), Some(&c) if c == b'I' as u32) {
        emit_digits(&mut text, &mut digits, r);
    }
    if digits.len() > 1 {
        tags.push("interleaved-or-multi-digit-width");
    }
    let width: i64 = digits.parse().unwrap_or(0);
    let width_cap = if width == 0 { inlen + 4 } else { (width as usize).min(inlen + 4) };

    // ── the type ─────────────────────────────────────────────────────────────────────
    //
    // Occasionally a byte with the top bit set: it lands in the `default:` arm and must
    // match the input literally, which is the same path `%%` takes. Safe for the same
    // reason the high literal filler is (see there) — nothing indexes a bitmap with it.
    if allow_high && r.below(8) == 0 {
        let c = if wide_engine { 0x80 + r.below(0xFF80) as u32 } else { 0x80 + r.below(0x80) as u32 };
        text.push(c);
        tags.push("literal-conv");
        tags.push("high-format-byte");
        return Conv { text, args, tags, terminal: false };
    }
    let ty = *r.pick(if allow_high { TYPES_NO_SCANSET } else { TYPES });
    let mut terminal = false;

    match ty {
        b'd' | b'i' | b'u' | b'o' | b'x' | b'X' | b'p' | b'P' => {
            text.push(ty as u32);
            if ty == b'p' || ty == b'P' {
                tags.push("pointer-conv");
            }
            if h_count == 2 {
                tags.push("hh");
            }
            if i64_pref {
                // ll / I64 / z / bare-I, plus %p and %P on a 64-bit host — all one store.
                tags.push("i64-store");
            }
            if !suppress {
                args.push(Arg::Scalar);
            }
        }
        b'e' | b'E' | b'f' | b'g' | b'G' => {
            text.push(ty as u32);
            tags.push("float");
            if big_l || l_pref {
                tags.push("float-double");
            }
            if !suppress {
                args.push(Arg::Scalar);
            }
        }
        b's' | b'S' | b'c' | b'C' => {
            text.push(ty as u32);
            let wide_target = if w_pref || l_pref {
                true
            } else if h_count != 0 {
                false
            } else if ty == b's' || ty == b'c' {
                wide_engine
            } else {
                !wide_engine
            };
            let is_char = ty == b'c' || ty == b'C';
            tags.push(if is_char { "char-conv" } else { "string-conv" });
            tags.push(if wide_target { "dest-wide" } else { "dest-narrow" });
            let elems = width_cap + inlen + 8;
            if !suppress {
                args.push(Arg::Buf { wide: wide_target, elems });
                if secure {
                    // Small sizes on purpose: overflow is a first-class outcome here, not
                    // an accident. The buffer is always big enough for `size` elements, so
                    // the engine's writes stay inside the guarded payload either way.
                    let size = match r.below(4) {
                        0 => 0,
                        1 => 1 + r.below(4) as u32,
                        2 => 1 + r.below(elems) as u32,
                        _ => elems as u32 + 4,
                    };
                    args.push(Arg::Size(size));
                    tags.push("secure-sized");
                }
            }
        }
        b'n' => {
            text.push(ty as u32);
            tags.push("n-conv");
            if !suppress {
                args.push(Arg::Scalar);
            }
        }
        b'[' => {
            text.push(b'[' as u32);
            let invert = r.below(3) == 0;
            if invert {
                text.push(b'^' as u32);
                tags.push("scanset-invert");
            }
            if r.below(4) == 0 {
                text.push(b']' as u32); // a leading ] is a member, not the terminator
                tags.push("scanset-leading-bracket");
            }
            let items = 1 + r.below(4);
            for _ in 0..items {
                if r.below(3) == 0 {
                    // A range. NARROW stops at 0x7F — exclusion 1.
                    let (a, b) = if wide_engine && r.below(2) == 0 {
                        let lo = 1 + r.below(0xFF00) as u32;
                        (lo, lo + r.below(256) as u32)
                    } else {
                        let lo = 0x21 + r.below(0x5C) as u32;
                        (lo, 0x21 + r.below(0x5C) as u32)
                    };
                    let (a, b) = if r.below(2) == 0 { (a, b) } else { (b, a) };
                    text.push(a);
                    text.push(b'-' as u32);
                    text.push(b);
                    tags.push("scanset-range");
                } else if wide_engine && r.below(4) == 0 {
                    text.push(0x100 + r.below(0xFF00) as u32);
                    tags.push("scanset-wide-char");
                } else {
                    let c = *r.pick(b"abcxyzXYZ0123456789 \t+-.^%");
                    text.push(c as u32);
                }
            }
            if r.below(12) == 0 {
                // Unterminated: the C's scanset loop stops at the format's NUL, then the
                // outer `format++` steps PAST it. The format buffer's trailing NULs are
                // what make that defined here rather than a read into whatever follows.
                tags.push("unterm-scanset");
                terminal = true;
            } else {
                text.push(b']' as u32);
            }
            let elems = width_cap + inlen + 8;
            if !suppress {
                // `%[` writes `_CHAR_` — the ENGINE's width, always. Unlike %s/%c there is
                // no cross-width dispatch.
                args.push(Arg::Buf { wide: wide_engine, elems });
                if secure {
                    let size = match r.below(4) {
                        0 => 0,
                        1 => 1 + r.below(4) as u32,
                        2 => 1 + r.below(elems) as u32,
                        _ => elems as u32 + 4,
                    };
                    args.push(Arg::Size(size));
                    tags.push("secure-sized");
                }
            }
            tags.push("scanset");
        }
        _ => {
            // `%%` and the "no meaning as a format-control character" arm, which are the
            // SAME code in the C: match the literal, assign nothing.
            text.push(ty as u32);
            tags.push("literal-conv");
        }
    }
    Conv { text, args, tags, terminal }
}

fn gen_case(r: &mut Rng, wide_engine: bool, secure: bool, strn: bool) -> Case {
    let input = gen_input(r, wide_engine, 24);
    let mut fmt: Vec<u32> = Vec::new();
    let mut args: Vec<Arg> = Vec::new();
    let mut tags: Vec<&'static str> = Vec::new();

    // Does THIS case get format bytes with the top bit set?
    //
    // They are perfectly legal (see the module docs: `_isctype_l` rejects a negative `c`
    // and returns 0, it does not read out of bounds) — but they must never end up inside
    // a `%[`, where the C would sign-extend one into ntdll's ULONG bit index. And "inside
    // a `%[`" is not something the generator can promise locally: an UNTERMINATED scanset
    // swallows the whole rest of the format, and a scanset can become unterminated by
    // accident, because an item `^` emitted in first position is read by the C as the
    // INVERT flag, which shifts the terminator `]` into being a member.
    //
    // (That is not hypothetical. It is what refimpl.c's out-of-range counter caught on
    // the first run with high bytes enabled — a case whose format was `%2h3h[^]\xf1%*hG`,
    // where the generator believed it had written a closed one-item scanset.)
    //
    // So the two features are made mutually exclusive per case rather than reasoned
    // about: a high-byte case has no `[` in its type set at all.
    let allow_high = r.below(3) == 0;

    let nconv = 1 + r.below(3);
    for i in 0..nconv {
        if i > 0 || r.below(3) == 0 {
            // literal filler: whitespace runs match any run (including none), ordinary
            // characters must match and are not stored.
            match r.below(5) {
                0 => fmt.push(b' ' as u32),
                1 => {
                    fmt.push(*r.pick(b" \t\n") as u32);
                    fmt.push(*r.pick(b" \t") as u32);
                }
                2 => fmt.push(*r.pick(b"abc019+-.") as u32),
                3 if allow_high => {
                    // A format character with the top bit set. On the narrow engine
                    // `_ISSPACE_(*format)` sign-extends it into `_isctype_l`, whose
                    // `c >= -1 && c <= 255` guard REJECTS it and falls through to
                    // `return 0` (ctype.c:174-192) — well defined, so this is GRADED and
                    // not excluded. `%[`'s scanset index is the one place a high narrow
                    // byte is genuinely UB, and that alone stays ASCII.
                    fmt.push(if wide_engine {
                        0x80 + r.below(0xFF80) as u32
                    } else {
                        0x80 + r.below(0x80) as u32
                    });
                }
                _ => {}
            }
        }
        let c = gen_conv(r, wide_engine, secure, input.len(), allow_high);
        fmt.extend_from_slice(&c.text);
        args.extend_from_slice(&c.args);
        for t in c.tags {
            if !tags.contains(&t) {
                tags.push(t);
            }
        }
        if c.terminal {
            break;
        }
    }

    // The consumed-count probe. `%n` writes `consumed - 1` and forces `suppress`, so it
    // never increments the return — appending it is transparent except that it reports how
    // far the source cursor got. On a STRING source that is the ONLY window onto the
    // cursor at all (the `file` parameter is by value), which is why it is not optional
    // decoration: it is the oracle for `_GETC_`/`_UNGETC_` arithmetic.
    let probe_n = !matches!(tags.last(), Some(&"unterm-scanset")) && r.below(2) == 0;
    if probe_n {
        fmt.push(b'%' as u32);
        fmt.push(b'n' as u32);
        args.push(Arg::Scalar);
        tags.push("n-probe");
    }

    let length = if strn {
        // Deliberately spanning below, at, and above the data — the limit interacting with
        // an embedded NUL and with the padding is where STRING_LEN's `_GETC_` differs from
        // STRING's.
        match r.below(4) {
            0 => 0,
            1 => r.below(input.len() + 1),
            2 => input.len(),
            _ => input.len() + r.below(6),
        }
    } else {
        0
    };
    if strn && length <= input.len() {
        tags.push("at-or-below-limit");
    }
    Case { fmt, args, input, length, tags }
}

// ══════════════════════════════════════════════════════════════════════════════════════
// Rows
// ══════════════════════════════════════════════════════════════════════════════════════

const FUZZ: usize = 20_000;

/// Every entry point, as (row name, Ep, is-STRING_LEN).
fn entry_points() -> Vec<(&'static str, Ep, bool)> {
    vec![
        ("vsscanf_l", Ep::StrA(vsscanf_l, refs_vsscanf_l), false),
        ("vsscanf_s_l", Ep::StrA(vsscanf_s_l, refs_vsscanf_s_l), false),
        ("vswscanf_l", Ep::StrW(vswscanf_l, refs_vswscanf_l), false),
        ("vswscanf_s_l", Ep::StrW(vswscanf_s_l, refs_vswscanf_s_l), false),
        ("vsnscanf_l", Ep::StrNA(vsnscanf_l, refs_vsnscanf_l), true),
        ("vsnscanf_s_l", Ep::StrNA(vsnscanf_s_l, refs_vsnscanf_s_l), true),
        ("vsnwscanf_l", Ep::StrNW(vsnwscanf_l, refs_vsnwscanf_l), true),
        ("vsnwscanf_s_l", Ep::StrNW(vsnwscanf_s_l, refs_vsnwscanf_s_l), true),
        ("vfscanf_l", Ep::FileA(vfscanf_l, refs_vfscanf_l), false),
        ("vfscanf_s_l", Ep::FileA(vfscanf_s_l, refs_vfscanf_s_l), false),
        ("vfwscanf_l", Ep::FileW(vfwscanf_l, refs_vfwscanf_l), false),
        ("vfwscanf_s_l", Ep::FileW(vfwscanf_s_l, refs_vfwscanf_s_l), false),
        ("vcscanf_l", Ep::ConA(vcscanf_l, refs_vcscanf_l), false),
        ("vcscanf_s_l", Ep::ConA(vcscanf_s_l, refs_vcscanf_s_l), false),
        ("__stdio_common_vsscanf", Ep::StrNA(sc_p_a, sc_p_a_r), true),
        ("__stdio_common_vsscanf [SECURE]", Ep::StrNA(sc_s_a, sc_s_a_r), true),
        ("__stdio_common_vswscanf", Ep::StrNW(sc_p_w, sc_p_w_r), true),
        ("__stdio_common_vswscanf [SECURE]", Ep::StrNW(sc_s_w, sc_s_w_r), true),
        ("__stdio_common_vfscanf [SECURE]", Ep::FileA(sc_s_fa, sc_s_fa_r), false),
        ("__stdio_common_vfwscanf", Ep::FileW(sc_p_fw, sc_p_fw_r), false),
    ]
}

fn fuzz_rows() {
    for (name, ep, strn) in entry_points() {
        let secure = name.contains("_s_l") || name.contains("SECURE");
        let wide = ep.wide();
        // The classes a scanf corpus is worthless without. `n-probe` is the cursor oracle;
        // `float` reaches msvcrt's bignum; `scanset` the ntdll bitmap; `secure-sized` the
        // size va_arg (which is what makes the SECURE rows different code at all).
        let mut req: Vec<&'static str> = vec![
            "eof-ret", "zero", "assigned", "n-probe", "float", "scanset", "string-conv",
            "char-conv", "suppressed", "literal-conv", "n-conv", "unterm-scanset",
            "scanset-range", "dest-wide", "dest-narrow", "pointer-conv", "hh",
            "high-format-byte",
        ];
        if secure {
            req.push("secure-sized");
            req.push("secure-suppressed");
        }
        if strn {
            req.push("at-or-below-limit");
        }
        let req: &'static [&'static str] = Box::leak(req.into_boxed_slice());
        grade(name, req, FUZZ, move |r| {
            let case = gen_case(r, wide, secure, strn);
            unsafe { run_case(ep, &case) }
        });
    }
}

// ── the deliberate behaviours, one hand-shaped row each ───────────────────────────────
//
// The fuzz above reaches these, but a fuzz row cannot ASSERT that it reached them: the
// outcome classes it can read off the reference (ret < 0 / == 0 / > 0) do not name a
// branch. Each row below constructs the branch directly, so its `require` is a real claim
// and a negative control has something specific to break.

fn ascii(s: &str) -> Vec<u32> {
    s.bytes().map(|b| b as u32).collect()
}

/// The SECURE truncation contract, in one shape: **no handler, no errno, not -1** — the
/// return is the count of fields assigned BEFORE the current directive.
fn row_secure_overflow() {
    for (name, ep, wide) in [
        ("SECURE %s overflow", Ep::StrA(vsscanf_s_l, refs_vsscanf_s_l), false),
        ("SECURE %s overflow [wide]", Ep::StrW(vswscanf_s_l, refs_vswscanf_s_l), true),
    ] {
        grade(name, &["overflowed", "fitted", "rd-nonzero-at-overflow"], 4_000, move |r| {
            let n = 1 + r.below(8);
            let word: String = (0..n).map(|i| (b'a' + (i % 26) as u8) as char).collect();
            // A leading `%d` that SUCCEEDS, so `rd` is non-zero when the overflow fires:
            // "returns 0" and "returns the prior count" are otherwise indistinguishable.
            let lead = r.below(2) == 0;
            let fmt = ascii(if lead { "%d %s" } else { " %s" });
            let input = ascii(&format!("{} {}", if lead { "7" } else { "" }, word));
            let size = r.below(n + 3) as u32;
            let elems = n + 16;
            let mut args = vec![];
            if lead {
                args.push(Arg::Scalar);
            }
            args.push(Arg::Buf { wide, elems });
            args.push(Arg::Size(size));
            let case = Case { fmt, args, input, length: 0, tags: vec![] };
            let (got, want, ctx, mut tags) = unsafe { run_case(ep, &case) };
            let overflowed = (size as usize) <= n;
            tags.push(if overflowed { "overflowed" } else { "fitted" });
            if overflowed && lead {
                tags.push("rd-nonzero-at-overflow");
                // The contract, asserted rather than merely compared: a subject that
                // matched a reference we had mis-modelled would still be graded green.
                assert_eq!(want.fx.ret, 1, "msvcrt's %s truncation must return rd, not -1/0");
                assert_eq!(want.fx.errno, ERRNO_SEED, "%s truncation must not set errno");
                assert_eq!(want.fx.pmt, 0, "%s truncation must not fire the handler");
            }
            (got, want, ctx, tags)
        });
    }
}

/// `%c`'s size test is `if(size)` — not `if(size>1)` — and happens BEFORE the store, so a
/// size of 0 writes NOTHING but the NUL at the buffer start. `%[`'s is at the END of the
/// loop body, like `%s`'s but one directive later.
fn row_secure_overflow_char_and_scanset() {
    grade(
        "SECURE %c / %[ overflow",
        &["char-zero-size", "char-overflow", "scanset-overflow", "fitted"],
        6_000,
        |r| {
            let n = 1 + r.below(6);
            let word: String = (0..n).map(|i| (b'a' + (i % 26) as u8) as char).collect();
            let scanset = r.below(2) == 0;
            let w = 1 + r.below(8);
            let fmt = ascii(&if scanset {
                format!("%d %{}[a-z]", w)
            } else {
                format!("%d %{}c", w)
            });
            let input = ascii(&format!("7 {word}"));
            let size = r.below(n + 3) as u32;
            let case = Case {
                fmt,
                args: vec![Arg::Scalar, Arg::Buf { wide: false, elems: n + 16 }, Arg::Size(size)],
                input,
                length: 0,
                tags: vec![],
            };
            let (got, want, ctx, mut tags) = unsafe { run_case(Ep::StrA(vsscanf_s_l, refs_vsscanf_s_l), &case) };
            let consumed = n.min(w);
            if scanset {
                if (size as usize) <= consumed {
                    tags.push("scanset-overflow");
                    assert_eq!(want.fx.ret, 1, "%[ truncation must return rd");
                } else {
                    tags.push("fitted");
                }
            } else if size == 0 {
                tags.push("char-zero-size");
                assert_eq!(want.fx.ret, 1, "%c with size 0 must return rd");
            } else if (size as usize) < consumed {
                tags.push("char-overflow");
                assert_eq!(want.fx.ret, 1, "%c truncation must return rd");
            } else {
                tags.push("fitted");
            }
            (got, want, ctx, tags)
        },
    );
}

/// `*`-suppression skips the pointer AND the size. The arg list here contains ONLY the
/// trailing `%d`'s pointer, so an engine that consumed a size for `%*s` would read the
/// spill and store there — caught by the spill check and by the `%d` object staying poison.
fn row_secure_suppressed_skips_both() {
    grade("SECURE %*s skips both slots", &["ran"], 3_000, |r| {
        // Paired so the suppressed directive always consumes its whole word and the
        // trailing `%d` is really reached: a `%*c` (width 1) against "abc" would stop on
        // 'b' and the row would be asserting the wrong thing.
        let (conv, word) = *r.pick(&[
            ("%*s", "abc"),
            ("%*c", "a"),
            ("%*[a-z]", "abc"),
            ("%*3c", "abc"),
            ("%*2s", "ab"),
            ("%*d", "7"),
        ]);
        let fmt = ascii(&format!("{conv} %d"));
        let input = ascii(&format!("{word} 42"));
        let case = Case {
            fmt,
            args: vec![Arg::Scalar],
            input,
            length: 0,
            tags: vec!["secure-suppressed"],
        };
        let (got, want, ctx, mut tags) = unsafe { run_case(Ep::StrA(vsscanf_s_l, refs_vsscanf_s_l), &case) };
        tags.push("ran");
        assert_eq!(want.fx.ret, 1, "the trailing %d must assign — arg model is off by a slot");
        (got, want, ctx, tags)
    });
}

/// `charstring:` returns `_EOF_RET` on empty-at-EOF; `widecharstring:` does NOT, because
/// its guard is `#if _MSVCR_VER >= 80` and this DLL builds at 0. Asymmetric ON PURPOSE.
fn row_eof_empty_asymmetry() {
    grade(
        "charstring/widecharstring EOF asymmetry",
        &["narrow-dest-eof", "wide-dest-zero"],
        400,
        |r| {
            // Which engine, and which destination width, chosen independently: on the
            // narrow engine `%s` is charstring and `%ls` widecharstring; on the wide
            // engine it is the other way round. Both orderings are exercised.
            let wide_engine = r.below(2) == 0;
            let narrow_dest = r.below(2) == 0;
            let spec = match (wide_engine, narrow_dest) {
                (false, true) => "%s",
                (false, false) => "%ls",
                (true, true) => "%hs",
                (true, false) => "%s",
            };
            let input = ascii(*r.pick(&[" ", "\t", "  \n", " \t "]));
            let case = Case {
                fmt: ascii(spec),
                args: vec![Arg::Buf { wide: !narrow_dest, elems: 16 }],
                input,
                length: 0,
                tags: vec![],
            };
            let ep = if wide_engine {
                Ep::StrW(vswscanf_l, refs_vswscanf_l)
            } else {
                Ep::StrA(vsscanf_l, refs_vsscanf_l)
            };
            let (got, want, ctx, mut tags) = unsafe { run_case(ep, &case) };
            if narrow_dest {
                tags.push("narrow-dest-eof");
                assert_eq!(want.fx.ret, -1, "charstring: must report EOF when nothing was read");
            } else {
                tags.push("wide-dest-zero");
                assert_eq!(
                    want.fx.ret, 0,
                    "widecharstring: must NOT report EOF at _MSVCR_VER == 0"
                );
            }
            (got, want, ctx, tags)
        },
    );
}

/// STRING_LEN's `_GETC_` is `(consumed++, consumed>length ? '\\0' : *file++)`: the
/// increment happens BEFORE the limit test and is never undone, so at the limit it yields
/// 0 and STILL COUNTS. `%n` is what sees it.
fn row_strn_counts_refused_char() {
    for (name, ep, wide) in [
        ("STRING_LEN counts the refused char", Ep::StrNA(vsnscanf_l, refs_vsnscanf_l), false),
        ("STRING_LEN counts the refused char [wide]", Ep::StrNW(vsnwscanf_l, refs_vsnwscanf_l), true),
    ] {
        let _ = wide;
        grade(name, &["past-limit", "within-limit"], 2_000, move |r| {
            let data = "abcdefgh";
            let length = r.below(data.len() + 2);
            let nc = 1 + r.below(4);
            let mut f = String::new();
            for _ in 0..nc {
                f.push_str("%c");
            }
            f.push_str("%n");
            let mut args: Vec<Arg> = (0..nc).map(|_| Arg::Buf { wide, elems: 8 }).collect();
            args.push(Arg::Scalar);
            let case = Case {
                fmt: ascii(&f),
                args,
                input: ascii(data),
                length,
                tags: vec![],
            };
            let (got, want, ctx, mut tags) = unsafe { run_case(ep, &case) };
            tags.push(if nc >= length { "past-limit" } else { "within-limit" });
            (got, want, ctx, tags)
        });
    }
}

/// `%hhd` — the `h_prefix == 2 -> _SET_NUMBER_(char)` arm is `#if _MSVCR_VER >= 140`, so
/// at 0 it falls through and a full `int` is stored. The scalar object's 16-byte payload
/// is what turns "stores 4 bytes, not 1" from an assumption into a measurement.
fn row_hh_stores_int() {
    grade("%hhd stores a full int", &["ran"], 1_500, |r| {
        let v = r.below(100000);
        let spec = *r.pick(&["%hhd", "%hhu", "%hhx", "%hhi"]);
        let case = Case {
            fmt: ascii(spec),
            args: vec![Arg::Scalar],
            input: ascii(&format!("{v}")),
            length: 0,
            tags: vec!["hh"],
        };
        let (got, want, ctx, mut tags) = unsafe { run_case(Ep::StrA(vsscanf_l, refs_vsscanf_l), &case) };
        tags.push("ran");
        (got, want, ctx, tags)
    });
}

/// `%1h2d` accumulates a width of 12: the digit scan re-runs at the top of every prefix
/// pass. A port that read the width once, before the prefixes, gets 1.
fn row_interleaved_width() {
    grade("interleaved width digits", &["ran"], 2_000, |r| {
        let spec = *r.pick(&["%1h2d", "%1l2d", "%2I643d", "%1w2s", "%1L2f", "%0h3d", "%1h2h3d"]);
        let input = ascii("123456789012345678");
        let args = if spec.ends_with('s') {
            vec![Arg::Buf { wide: true, elems: 40 }]
        } else {
            vec![Arg::Scalar]
        };
        let case = Case { fmt: ascii(spec), args, input, length: 0, tags: vec![] };
        let (got, want, ctx, mut tags) = unsafe { run_case(Ep::StrA(vsscanf_l, refs_vsscanf_l), &case) };
        tags.push("ran");
        (got, want, ctx, tags)
    });
}

/// An unterminated `%[` walks `format` past the NUL. The format buffer's trailing NULs are
/// what make that DEFINED for both sides; the C's own buffer has no such guarantee, which
/// is exactly why it is a bug worth pinning rather than papering over.
fn row_unterminated_scanset() {
    grade("unterminated %[", &["ran"], 1_200, |r| {
        let spec = *r.pick(&["%[abc", "%[^abc", "%[", "%[]abc", "%[a-", "%5[xyz"]);
        let input = gen_input(r, false, 12);
        let case = Case {
            fmt: ascii(spec),
            args: vec![Arg::Buf { wide: false, elems: 32 }],
            input,
            length: 0,
            tags: vec!["unterm-scanset"],
        };
        let (got, want, ctx, mut tags) = unsafe { run_case(Ep::StrA(vsscanf_l, refs_vsscanf_l), &case) };
        tags.push("ran");
        (got, want, ctx, tags)
    });
}

/// Narrow-STRING `_GETC_FUNC_` is `(*file++)` while `_GETC_` is `(unsigned char)*file++`.
/// The float context uses the former, so a 0xFF byte sign-extends to -1, comes back
/// through a `wchar_t` return as WEOF, and leaves `nch == -1` on a source whose `_EOF_` is
/// 0 — the engine then keeps going with a character it can never match.
fn row_float_sign_extension() {
    grade("%f sign-extension trap (0xFF)", &["has-ff", "clean"], 3_000, |r| {
        let has_ff = r.below(2) == 0;
        let mut input: Vec<u32> = ascii("1.5");
        if has_ff {
            input.push(0xFF);
        }
        input.extend(ascii("9z"));
        let spec = *r.pick(&["%f%d", "%f%c", "%lf%s", "%f%[0-9]", "%f%n"]);
        let mut args: Vec<Arg> = vec![Arg::Scalar];
        args.push(match spec {
            "%f%c" => Arg::Buf { wide: false, elems: 16 },
            "%lf%s" => Arg::Buf { wide: false, elems: 24 },
            "%f%[0-9]" => Arg::Buf { wide: false, elems: 24 },
            _ => Arg::Scalar,
        });
        let case = Case { fmt: ascii(spec), args, input, length: 0, tags: vec![] };
        let (got, want, ctx, mut tags) = unsafe { run_case(Ep::StrA(vsscanf_l, refs_vsscanf_l), &case) };
        tags.push(if has_ff { "has-ff" } else { "clean" });
        (got, want, ctx, tags)
    });
}

/// The final `if (nch != _EOF_) _UNGETC_(nch, file)`.
///
/// On a string source this is `file--; consumed--` on values nobody can see, and it runs
/// after the last `%n` could have read `consumed` — dropping it is UNOBSERVABLE there. On a
/// FILE it pushes a character back into the stream, and the stream's final
/// `(pos, pushback)` is part of the compared image.
fn row_file_ungetc() {
    for (name, ep, wide) in [
        ("final _UNGETC_ (FILE)", Ep::FileA(vfscanf_l, refs_vfscanf_l), false),
        ("final _UNGETC_ (FILE, wide)", Ep::FileW(vfwscanf_l, refs_vfwscanf_l), true),
    ] {
        grade(name, &["ungot", "at-eof"], 3_000, move |r| {
            // A format that COMPLETES (so the engine reaches the tail) with a character
            // still in hand, versus one that runs the stream dry.
            let trailing = r.below(2) == 0;
            let input = ascii(if trailing { "42abc" } else { "42" });
            let case = Case {
                fmt: ascii("%d"),
                args: vec![Arg::Scalar],
                input,
                length: 0,
                tags: vec![],
            };
            let (got, want, ctx, mut tags) = unsafe { run_case(ep, &case) };
            let _ = wide;
            tags.push(if trailing { "ungot" } else { "at-eof" });
            (got, want, ctx, tags)
        });
    }
}

/// `z` IS compiled at `_MSVCR_VER == 0` (`#if _MSVCR_VER == 0 || >= 140`), a bare `I` falls
/// THROUGH into it, and `%p`/`%P` set the same flag and skip the `0`/`0x` base sniff. All
/// four land on the 64-bit store on this host.
fn row_prefix_widths() {
    grade("prefix widths (z / I / p / ll / I64)", &["ran"], 4_000, |r| {
        let spec = *r.pick(&[
            "%zd", "%Id", "%I64d", "%lld", "%ld", "%hd", "%I32d", "%p", "%P", "%zx", "%Iu",
            "%I64x", "%llo", "%p%d",
        ]);
        let input = ascii(*r.pick(&[
            "0x1234abcd", "1234567890123", "0777", "-42", "0", "00x10", "18446744073709551615",
            "0x7fffffffffffffff 5",
        ]));
        let n = if spec == "%p%d" { 2 } else { 1 };
        let case = Case {
            fmt: ascii(spec),
            args: (0..n).map(|_| Arg::Scalar).collect(),
            input,
            length: 0,
            tags: vec![],
        };
        let (got, want, ctx, mut tags) = unsafe { run_case(Ep::StrA(vsscanf_l, refs_vsscanf_l), &case) };
        tags.push("ran");
        (got, want, ctx, tags)
    });
}

// ══════════════════════════════════════════════════════════════════════════════════════

fn assert_shims_live() {
    // 1. The wide engine really is 16-bit. glibc's wchar_t is 32-bit, so a 16-bit L"12"
    //    is `31 00 32 00 00 00` — one character U+00320031 to glibc, and it cannot answer
    //    12. (There is no glibc `vswscanf_l` to bind to, but a future refactor could
    //    reintroduce one; this costs nothing and names the failure.)
    let input: Vec<u16> = "12 ".encode_utf16().chain(core::iter::once(0)).collect();
    let fmt: Vec<u16> = "%d".encode_utf16().chain(core::iter::once(0)).collect();
    let mut v: i32 = 0;
    let mut ap = (&mut v as *mut i32 as usize).to_le_bytes().to_vec();
    ap.extend_from_slice(&[0u8; 64]);
    unsafe {
        assert_eq!(
            vswscanf_l(input.as_ptr(), fmt.as_ptr(), core::ptr::null_mut(), ap.as_mut_ptr()),
            1
        );
    }
    assert_eq!(v, 12, "vswscanf_l(L\"12\", L\"%d\") != 12 — the wide shim is not the Rust's");

    // 2. The REFERENCE is msvcrt, not something the host supplied. `%[a-]` separates
    //    msvcrt's LOOKAHEAD range syntax (`format[1]=='-' && format[2] && format[2]!=']'`
    //    — so a trailing `-` is a plain member) from the lookbehind form libcntpr's copy of
    //    scanf.h uses. Both sides must agree, and both must answer "a-" for "a-b".
    let mut buf = [0xAAu8; 16];
    let mut ap = (buf.as_mut_ptr() as usize).to_le_bytes().to_vec();
    ap.extend_from_slice(&[0u8; 64]);
    let r = unsafe {
        refs_vsscanf_l(
            b"a-b\0".as_ptr() as *const i8,
            b"%[a-]\0".as_ptr() as *const i8,
            core::ptr::null_mut(),
            ap.as_mut_ptr(),
        )
    };
    assert_eq!(r, 1);
    assert_eq!(&buf[..3], b"a-\0", "the reference's %[ is not msvcrt's lookahead form");

    // 3. The invalid-parameter counters are wired to SEPARATE storage. scanf never fires
    //    the handler, so this proves the channel rather than a value.
    unsafe {
        subj_invalid_pmt_reset();
        ref_invalid_pmt_reset();
        assert_eq!(subj_invalid_pmt_count(), 0);
        assert_eq!(ref_invalid_pmt_count(), 0);
    }
}

fn main() {
    assert_shims_live();
    unsafe { msvcrt_ref_bitmap_oob_reset() };

    fuzz_rows();
    row_secure_overflow();
    row_secure_overflow_char_and_scanset();
    row_secure_suppressed_skips_both();
    row_eof_empty_asymmetry();
    row_strn_counts_refused_char();
    row_hh_stores_int();
    row_interleaved_width();
    row_unterminated_scanset();
    row_float_sign_extension();
    row_file_ungetc();
    row_prefix_widths();

    // ── the two exclusions, MEASURED ────────────────────────────────────────────────
    let oob = unsafe { msvcrt_ref_bitmap_oob_count() };
    if oob == 0 {
        record(
            "%[ high-byte exclusion".to_string(),
            V::Match,
            "0 out-of-range RtlSetBits indices across every scanset case — the narrow \
             corpora really did stop at 0x7F,\n        and the WIDE corpora (which run the \
             full 16-bit range) really are in range. See refimpl.c's bitmap model."
                .to_string(),
        );
    } else {
        record(
            "%[ high-byte exclusion".to_string(),
            V::Diverge,
            format!(
                "{oob} RtlSetBits calls asked for an index outside the map. A NARROW \
                 scanset byte >= 0x80 reached the\n        corpus: real ntdll would write \
                 ~128 MB below the heap block there, so those cases are grading a clamp \
                 against\n        a Rust clamp and proving nothing. Fix the generator, not \
                 this row."
            ),
        );
    }
    let spill = unsafe { SPILL_HITS };
    if spill == 0 {
        record(
            "va_list arg model".to_string(),
            V::Match,
            "the SPILL object past the last modelled argument was pristine after every \
             call on both sides —\n        no engine consumed a slot the generator did not \
             account for."
                .to_string(),
        );
    } else {
        record(
            "va_list arg model".to_string(),
            V::Diverge,
            format!(
                "{spill} calls wrote through the SPILL slots, i.e. consumed MORE va_list \
                 slots than the directives\n        account for. Either a port is off by a \
                 slot (a real bug — pointer-then-size, or a `*`-suppressed\n        \
                 directive still reading), or the generator's arg model is wrong. Both \
                 sides are counted, so compare\n        which one moved."
            ),
        );
    }

    let res = unsafe { &*core::ptr::addr_of!(RESULTS) };
    let (mut m, mut d, mut u) = (0, 0, 0);
    println!("\n=== wave 17: msvcrt-crt-rust scanf (SHIPPING) vs GENUINE msvcrt scanf.c ===\n");
    for r in res {
        let tag = match r.v {
            V::Match => {
                m += 1;
                "MATCH       ✓"
            }
            V::Diverge => {
                d += 1;
                "DIVERGE     ✗"
            }
            V::Unexercised => {
                u += 1;
                "UNEXERCISED ✗"
            }
        };
        println!("  {:<40} {}", r.name, tag);
        if !r.note.is_empty() {
            println!("        {}", r.note);
        }
    }
    let cases = unsafe { CASES };
    println!("\n  {m} MATCH   {d} DIVERGE   {u} UNEXERCISED   (of {} rows)", res.len());
    println!(
        "\n  EXCLUDED, by construction and not by omission:\n    \
         * `%[` scanset bytes >= 0x80 on the NARROW engine — the C sign-extends a `char`\n      \
           into ntdll's ULONG bit index, so `%[\\x80]` is RtlSetBits(map, 0xFFFFFF80, 1): a\n      \
           wild write ~128 MB below the heap block, under an ASSERT release ntdll drops.\n      \
           UB, not behaviour. The WIDE engine is unrestricted (wchar_t zero-extends into a\n      \
           65536-bit map) and its scansets DO run the full 16-bit range.\n    \
         * the wide CONSOLE engine (`vcwscanf_l` / `vcwscanf_s_l`) — CONSOLE does not\n      \
           `#undef _EOF_`, so it tests `nch == WEOF` against a `_getch()` that reports EOF\n      \
           as -1 and can spin forever. `vcscanf_l` / `vcscanf_s_l` ARE graded, and the two\n      \
           differ only in `_GETC_`/`_UNGETC_`/`_EOF_`.\n\n  \
         NOT excluded, though it looks like it should be: a narrow FORMAT byte >= 0x80.\n    \
         `_ISSPACE_(*format)` does sign-extend it, but `_isctype_l`'s guard is\n    \
         `c >= -1 && c <= 255` and a negative c falls through to `return 0`\n    \
         (ctype.c:174-192) — defined behaviour, so those bytes are IN the corpus\n    \
         (`high-format-byte`, a required tag). Only the `%[` scanset INDEX is UB."
    );

    if d + u > 0 {
        println!("\n  ✗ FAIL — scanf.c is FLIPPED, so a divergence means msvcrt.dll is shipping a");
        println!("    wrong answer, and an UNEXERCISED row means the green above proves nothing");
        println!("    about the branch that row exists to test.\n");
        println!("{cases} scanf cases passed");
        std::process::exit(1);
    }

    println!("\n{cases} scanf cases passed");
}
