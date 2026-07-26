//! Wave 16 — msvcrt's **scanf engine**, in Rust: `dll/win32/msvcrt/scanf.h` +
//! `scanf.c`, at `_MSVCR_VER == 0`.
//!
//! # One engine instead of sixteen
//!
//! `scanf.h` is a TEMPLATE. `scanf.c` `#include`s it **16 times** under every
//! combination of `WIDE_SCANF` / `SECURE` / `STRING` / `STRING_LEN` / `CONSOLE`,
//! producing sixteen near-identical ~600-line engines (~9,600 lines of C). Here it is
//! one generic function over three axes, exactly the three the preprocessor varies:
//!
//! | C macro axis | Rust |
//! |---|---|
//! | `WIDE_SCANF` | `W: Width` — `Narrow` (`char`) / `Wide` (`wchar_t`) |
//! | `STRING` / `STRING_LEN` / `CONSOLE` / (none) | `S: Source` — eight impls (4 kinds × 2 widths) |
//! | `SECURE` | `const SECURE: bool` |
//!
//! The `SECURE` axis is a `const` and not a second body because the C's five `#ifdef
//! SECURE` sites are all the *same line*:
//!
//! ```c
//! unsigned size = suppress ? UINT_MAX : va_arg(ap, unsigned);   /* SECURE */
//! unsigned size = UINT_MAX;                                     /* not */
//! ```
//!
//! — so the non-secure build is byte-for-byte the secure one with `size == UINT_MAX`.
//! There is no behavioural fork anywhere else in the file.
//!
//! # `_MSVCR_VER == 0` — what that decides
//!
//! msvcrt.dll is built `-D_MSVCR_VER=0` (`dll/win32/msvcrt/CMakeLists.txt`), and three
//! `#if`s inside the engine turn on that. Each is transcribed with its guard visible:
//!
//! * `case 'z'` (scanf.h:290-292, `_MSVCR_VER == 0 || >= 140`) — **IS** compiled.
//! * `h_prefix == 2 -> _SET_NUMBER_(char)` (scanf.h:382-384, `>= 140`) — is **NOT**,
//!   so `%hhd` stores a full `int`.
//! * the EOF_SENTINEL-and-nothing-read early return inside `widecharstring:` (scanf.h:521-527,
//!   `>= 80`) — is **NOT**. So `charstring:` returns `_EOF_RET` when it reads nothing at
//!   EOF_SENTINEL and `widecharstring:` returns 0 fields instead. **That asymmetry is
//!   load-bearing and deliberate here** — see `Dest`/`EOF_AT_EMPTY`.
//!
//! # Zero gotos, because the C's eight are all forward dispatch
//!
//! `scanf.h` has 8 `goto`s (`number` ×4, `charstring`/`widecharstring` ×6,
//! `character`/`widecharacter` ×6) and every one of them jumps FORWARD into a sibling
//! labelled block of the same `switch`. There is no backward goto and no shared
//! cleanup tail, so each is just "pick a base / pick a destination width, then run one
//! block". No labelled-loop gymnastics were needed; the four blocks are two helper
//! functions parameterised by destination element type.
//!
//! # va_arg is done inline, at the C's own `va_arg` points
//!
//! cnt-rust's engine (`cnt_rust::core_rs::scanf`) routes stores through a `Sink` trait
//! because that engine is pure — no `unsafe`, no raw pointers. This one is not: it must
//! reproduce `sptr_beg`, the `unsigned size` countdown, and the truncation semantics
//! ("blow away the WHOLE result") through real pointers anyway. So each C `va_arg` is
//! ONE `va_next` call in the same statement position, which is the closest thing to a
//! transcription and keeps the cursor's lockstep checkable by eye. The two rules that
//! carry the risk, both preserved verbatim:
//!
//! 1. order is **pointer, then size**;
//! 2. `*`-suppression skips **both** — the C's `suppress ? NULL : va_arg(...)`.
//!
//! # What this shares, and what it deliberately does not
//!
//! `%f` goes through **msvcrt's own bignum**: `fpnum_parse`/`fpnum_double` in
//! `crate::fpnum` (wave 13), reached by implementing that module's `CharSrc` over
//! `scanf.h`'s `strtod_scanf_ctx`. That is the same correctly-rounded parser `strtod`
//! uses, which is what the C does too — it is literally the same two calls.
//!
//! `char2digit`/`wchar2digit` are transcribed from `scanf.c:42-58` rather than imported
//! from cnt-rust's byte-identical copy: cnt-rust's is a transcription of a DIFFERENT
//! header (`sdk/lib/crt/string/scanf.h`, libcntpr) graded by a different difftest, and
//! coupling the two would let a libcntpr-side change silently move msvcrt.
//!
//! `%[` uses a stack bitmap, not `RtlInitializeBitMap` + `HeapAlloc`. See `set_bits`
//! for the one place that is NOT a literal transcription and why it cannot be.

use core::ffi::c_void;

use msvcrt_sys as sys;

use crate::fpnum::CharSrc;
use crate::printf::va_next;

// ── the C declarations this engine calls ──────────────────────────────────────────────
//
// Transcribed from the include closure of `dll/win32/msvcrt/scanf.c`, never inferred
// from a call site (prelude.rs explains why that distinction is the whole game).
//
// NOT here on purpose: `isspace` and `iswspace`. This crate DEFINES both (lib.rs's
// `isc!`/`isw!` macros), so an `extern` declaration would bind to our own definition
// with whatever width the declaration happened to claim — exactly the silent mismatch
// prelude.rs records for the `_isw*_l` family. They are called through `crate::`.

/// `FILE` is opaque here: the engine only ever hands the pointer straight back to
/// `fgetc`/`ungetc`/`_lock_file`. Never dereferenced, so its layout is not this
/// module's business.
#[repr(C)]
pub struct FILE {
    _opaque: [u8; 0],
}

extern "C" {
    // include/stdio.h:116/:147/:93/:94
    fn fgetc(file: *mut FILE) -> i32;
    fn ungetc(c: i32, file: *mut FILE) -> i32;
    fn _lock_file(file: *mut FILE);
    fn _unlock_file(file: *mut FILE);
    // include/corecrt_wstdio.h:74/:85 — `wint_t` is 16-bit in msvcrt, so `fgetwc`
    // ZERO-extends into the engine's `int nch` and `WEOF` arrives as 65535, never -1.
    fn fgetwc(file: *mut FILE) -> u16;
    fn ungetwc(c: u16, file: *mut FILE) -> u16;
    // THIS tree's `sdk/include/crt/stdio.h:92,98` — `#define _iob __iob_func()` and
    // `#define stdin (&_iob[0])`, so `stdin` is just the array base. `__acrt_iob_func`
    // is the UCRT spelling and msvcrt does not export it at `_MSVCR_VER=0`; using it
    // linked the Rust fine and failed only at the DLL link. msvcrt.spec:221.
    fn __iob_func() -> *mut FILE;
    // include/conio.h:21/:25
    fn _getch() -> i32;
    fn _ungetch(c: i32) -> i32;
}

/// `UINT_MAX` — the `unsigned size` sentinel the non-SECURE build hardcodes.
const UINT_MAX: u32 = u32::MAX;

/// `sizeof(void *) == sizeof(LONGLONG)` (scanf.h:293, :305). A compile-time fact on
/// each target, kept as the C spells it rather than as `cfg(target_pointer_width)`.
const PTR_IS_64: bool = core::mem::size_of::<*const c_void>() == core::mem::size_of::<i64>();

// ── scanf.c:39-58 — the two digit helpers ─────────────────────────────────────────────

/// `char2digit(char c, int base)` (scanf.c:42-48).
///
/// The parameter is a **`char`**, so the `int nch` the engine holds truncates on the
/// call; and the C's odd bounds are kept exactly — `c <= '0'+base-1` (which makes
/// `base <= 10` accept only real digits) and `c <= 'A'+base-11` for the letter ranges.
#[inline]
fn char2digit(c: i8, base: i32) -> i32 {
    let c = c as i32; // the C integer-promotes the `char` parameter
    if c >= b'0' as i32 && c <= b'9' as i32 && c <= b'0' as i32 + base - 1 {
        return c - b'0' as i32;
    }
    if base <= 10 {
        return -1;
    }
    if c >= b'A' as i32 && c <= b'Z' as i32 && c <= b'A' as i32 + base - 11 {
        return c - b'A' as i32 + 10;
    }
    if c >= b'a' as i32 && c <= b'z' as i32 && c <= b'a' as i32 + base - 11 {
        return c - b'a' as i32 + 10;
    }
    -1
}

/// `wchar2digit(wchar_t c, int base)` (scanf.c:53-59) — the same body over an
/// **unsigned** 16-bit parameter, so `nch` truncates to 16 bits instead of 8.
#[inline]
fn wchar2digit(c: u16, base: i32) -> i32 {
    let c = c as i32;
    if c >= b'0' as i32 && c <= b'9' as i32 && c <= b'0' as i32 + base - 1 {
        return c - b'0' as i32;
    }
    if base <= 10 {
        return -1;
    }
    if c >= b'A' as i32 && c <= b'Z' as i32 && c <= b'A' as i32 + base - 11 {
        return c - b'A' as i32 + 10;
    }
    if c >= b'a' as i32 && c <= b'z' as i32 && c <= b'a' as i32 + base - 11 {
        return c - b'a' as i32 + 10;
    }
    -1
}

// ── axis 1: WIDE_SCANF (scanf.h:26-44) ────────────────────────────────────────────────

/// Everything `#ifdef WIDE_SCANF` selects.
///
/// `_WIDE2SUPPORTED_` and `_CHAR2SUPPORTED_` are **identity** macros in the C — both
/// carry a `/* FIXME: convert */` comment and neither converts anything. The conversion
/// that DOES happen is the assignment's truncation to the destination element type, and
/// that lives in `Dest::from_nch`. Documented no-op, not a dropped step.
pub trait Width {
    /// `_CHAR_` — `char` (signed on all three targets, asserted in
    /// `dll/win32/msvcrt/rust_layout_assert.c`) or `wchar_t`. It is also `%[`'s
    /// destination element (unlike `%s`/`%c`, the scanset has no cross-width table),
    /// hence the `Dest` bound.
    type Ch: Copy + Dest;
    /// `_BITMAPSIZE_/64` words. An associated type because `[u64; W::BITS/64]` needs
    /// generic const expressions, which are not stable.
    type Bitmap: AsRef<[u64]> + AsMut<[u64]>;

    /// The engine is the native width — selects the `%s`/`%S`/`%c`/`%C` cross table.
    const WIDE: bool;
    /// `_EOF_RET`. `EOF_SENTINEL` narrow, `(short)WEOF` wide — **both are `-1`**, but they are
    /// spelled separately because they are separate macros in the C.
    const EOF_RET: i32;
    /// `_BITMAPSIZE_` — 256 narrow, `256*256` wide.
    const BITMAP_BITS: u32;

    fn zero_bitmap() -> Self::Bitmap;

    /// `_ISSPACE_(c)`.
    ///
    /// # Safety
    /// Reads the current locale's ctype table.
    unsafe fn is_space(c: i32) -> bool;

    /// `_CHAR2DIGIT_(c, base)`.
    fn char2digit(c: i32, base: i32) -> i32;

    /// The value of a format element wherever the C uses `*format` in an `int` context.
    /// Narrow `char` SIGN-extends (a format byte >= 0x80 arrives negative); wide
    /// `wchar_t` zero-extends.
    fn fmt(c: Self::Ch) -> i32;

    /// `(_CHAR_)nch` — the cast in `if ((_CHAR_)nch == *format)`, as an `int`.
    fn narrow(nch: i32) -> i32;
}

/// `#else /* WIDE_SCANF */` — the `char` engine.
pub struct Narrow;
/// `#ifdef WIDE_SCANF` — the `wchar_t` engine.
pub struct Wide;

impl Width for Narrow {
    type Ch = i8;
    type Bitmap = [u64; 4]; // 256 bits
    const WIDE: bool = false;
    const EOF_RET: i32 = -1; // EOF_SENTINEL
    const BITMAP_BITS: u32 = 256;

    fn zero_bitmap() -> [u64; 4] {
        [0; 4]
    }
    unsafe fn is_space(c: i32) -> bool {
        crate::isspace(c) != 0
    }
    fn char2digit(c: i32, base: i32) -> i32 {
        char2digit(c as i8, base)
    }
    fn fmt(c: i8) -> i32 {
        c as i32
    }
    fn narrow(nch: i32) -> i32 {
        nch as i8 as i32
    }
}

impl Width for Wide {
    type Ch = u16;
    type Bitmap = [u64; 1024]; // 65536 bits
    const WIDE: bool = true;
    const EOF_RET: i32 = -1; // (short)WEOF
    const BITMAP_BITS: u32 = 256 * 256;

    fn zero_bitmap() -> [u64; 1024] {
        [0; 1024]
    }
    unsafe fn is_space(c: i32) -> bool {
        // `iswspace`'s C parameter is `wint_t` (16-bit — corecrt_wctype.h:57), so the
        // `int nch` truncates on the call. That matters: the wide CONSOLE engine can
        // reach here with nch == -1, which the C narrows to 0xFFFF.
        crate::iswspace(c as u16 as u32) != 0
    }
    fn char2digit(c: i32, base: i32) -> i32 {
        wchar2digit(c as u16, base)
    }
    fn fmt(c: u16) -> i32 {
        c as i32
    }
    fn narrow(nch: i32) -> i32 {
        nch as u16 as i32
    }
}

// ── the `%f` character source (scanf.h:151-196) ───────────────────────────────────────

/// `_GETC_FUNC_` — the RAW getter the float context uses, which is **not** `_GETC_`: it
/// does not touch `consumed` and, for `STRING_LEN`, does not honour `length` either
/// (`_GETC_FUNC_` is defined once under `#ifdef STRING`, before the `STRING_LEN` split).
///
/// There are exactly FIVE of these in the C, because scanf.h:150 guards the ctx block
/// with `!SECURE && !STRING_LEN && (!CONSOLE || !WIDE_SCANF)`: `file_`, `filew_`,
/// `str_`, `wstr_`, `console_`. SECURE reuses the non-secure one, `STRING_LEN` reuses
/// `str_`/`wstr_`, and — this one bites — **the wide CONSOLE engine reuses the NARROW
/// `console_` one**, so its ctx `_EOF_` is `EOF_SENTINEL` (-1) while the engine's is `WEOF`.
trait RawGet: Copy {
    /// `_EOF_` as it was defined when this ctx's `strtod_scanf_get` was compiled.
    /// scanf.h's `_EOF_`, which is deliberately NOT one value: `EOF` for a narrow FILE
    /// source, `WEOF` for a wide one, and **0** for the STRING sources, which redefine it
    /// so a NUL terminates the input. Differing per impl is the entire point of putting it
    /// on the trait.
    ///
    /// Every impl carries `// nolint: const-contradiction`. That scanner flags one NAME
    /// holding two VALUES — exactly right for free constants, exactly wrong for a trait
    /// associated constant, and it has no notion of `impl` blocks so it cannot tell them
    /// apart. Suppressed per site rather than baselined, so the reason sits next to the
    /// code instead of in a flat list that cannot hold one.
    /// See `Source::EOF_SENTINEL` above for why this varies and is nolint'd.
    const EOF_SENTINEL: i32;
    /// # Safety
    /// The cursor must be valid; a string cursor must be NUL-terminated.
    unsafe fn get(&mut self) -> i32;
}

/// `file_` (`fgetc`, `_EOF_ == EOF_SENTINEL`).
#[derive(Clone, Copy)]
struct RawFileA(*mut FILE);
impl RawGet for RawFileA {
    const EOF_SENTINEL: i32 = -1;  // nolint: const-contradiction
    unsafe fn get(&mut self) -> i32 {
        fgetc(self.0)
    }
}

/// `filew_` (`fgetwc`, `_EOF_ == WEOF == 0xFFFF`).
#[derive(Clone, Copy)]
struct RawFileW(*mut FILE);
impl RawGet for RawFileW {
    const EOF_SENTINEL: i32 = 0xFFFF;  // nolint: const-contradiction
    unsafe fn get(&mut self) -> i32 {
        fgetwc(self.0) as i32
    }
}

/// `str_` (`*file++`, `_EOF_ == 0`).
///
/// **No `(unsigned char)` cast** — `_GETC_FUNC_` is `(*file++)` while `_GETC_` is
/// `(unsigned char)*file++`. So the float path SIGN-extends where the main path does
/// not, and a 0xFF byte reaches `fpnum_parse` as `WEOF`. Same trap `fpnum.rs`'s
/// `StrSrc` documents for `strtod`; do not "fix" it.
#[derive(Clone, Copy)]
struct RawStrA(*const i8);
impl RawGet for RawStrA {
    const EOF_SENTINEL: i32 = 0;  // nolint: const-contradiction
    unsafe fn get(&mut self) -> i32 {
        let c = *self.0 as i32;
        self.0 = self.0.add(1);
        c
    }
}

/// `wstr_` (`*file++`, `_EOF_ == 0`). `wchar_t` is unsigned, so no sign trap.
#[derive(Clone, Copy)]
struct RawStrW(*const u16);
impl RawGet for RawStrW {
    const EOF_SENTINEL: i32 = 0;  // nolint: const-contradiction
    unsafe fn get(&mut self) -> i32 {
        let c = *self.0 as i32;
        self.0 = self.0.add(1);
        c
    }
}

/// `console_` (`_getch()`, `_EOF_ == EOF_SENTINEL`) — used by BOTH console engines.
#[derive(Clone, Copy)]
struct RawCon;
impl RawGet for RawCon {
    const EOF_SENTINEL: i32 = -1;  // nolint: const-contradiction
    unsafe fn get(&mut self) -> i32 {
        _getch()
    }
}

/// `struct _STRTOD_NAME_(strtod_scanf_ctx)` (scanf.h:151-163).
///
/// The C's `pthreadlocinfo locinfo` member is **dead**: neither `strtod_scanf_get` nor
/// `strtod_scanf_unget` reads it, and `fpnum_parse` takes the locinfo as its own
/// argument. Omitted rather than carried.
struct FpCtx<R: RawGet> {
    raw: R,
    length: i32,
    read: i32,
    cur: i32,
    unget: i32,
    err: bool,
}

/// `strtod_scanf_get` / `strtod_scanf_unget` (scanf.h:165-195), verbatim.
impl<R: RawGet> CharSrc for FpCtx<R> {
    unsafe fn get(&mut self) -> u16 {
        self.cur = R::EOF_SENTINEL;
        if self.length == 0 {
            return 0xFFFF; // WEOF — note this is WEOF, not the ctx's _EOF_
        }
        if self.unget != R::EOF_SENTINEL {
            self.cur = self.unget;
            self.unget = R::EOF_SENTINEL;
        } else {
            self.cur = self.raw.get();
            if self.cur == R::EOF_SENTINEL {
                return 0xFFFF;
            }
        }

        if self.length > 0 {
            self.length -= 1;
        }
        self.read += 1;
        // `return context->cur;` through a `wchar_t` return type — so the narrow string
        // source's sign-extended 0xFF byte comes back as WEOF. See `RawStrA`.
        self.cur as u16
    }

    unsafe fn unget(&mut self) {
        if self.length >= 0 {
            self.length += 1;
        }
        self.read -= 1;
        if self.unget != R::EOF_SENTINEL || self.cur == R::EOF_SENTINEL {
            self.err = true;
            return;
        }
        self.unget = self.cur;
    }
}

// ── axis 2: the source kind (scanf.h:46-148) ──────────────────────────────────────────

/// Everything `#ifdef CONSOLE` / `#ifdef STRING` / `#ifdef STRING_LEN` selects:
/// `_GETC_`, `_UNGETC_`, `_LOCK_FILE_`/`_UNLOCK_FILE_`, `_EOF_`, and the `%f` context's
/// raw getter.
trait Source {
    type W: Width;
    /// `_GETC_FUNC_`'s carrier for this source.
    type Raw: RawGet;
    /// The RAII form of `_UNLOCK_FILE_`. Every early return in the engine unlocks (there
    /// are eight: scanf.h:218, 420, 477, 488, 512, 564, 590, 679), so the unlock is a
    /// `Drop` instead of eight hand-written calls one of which would eventually be
    /// forgotten. A ZST for the string sources, whose `_UNLOCK_FILE_` is `do{}while(0)`.
    type Guard;

    /// `_EOF_`. **`STRING`/`STRING_LEN` redefine it to 0** for both widths (scanf.h:68);
    /// `CONSOLE` does not, so the wide console engine keeps `WEOF` even though
    /// `_getch()` reports end-of-input as -1. That mismatch is msvcrt's, and it is
    /// preserved: `nch == _EOF_` is simply never true there.
    const EOF_SENTINEL: i32;

    /// # Safety
    /// FFI.
    unsafe fn lock(&mut self) -> Self::Guard;
    /// `_GETC_(file)` — includes the `consumed++`.
    ///
    /// # Safety
    /// FFI / raw cursor.
    unsafe fn getc(&mut self) -> i32;
    /// `_UNGETC_(nch, file)` — includes the `consumed--`.
    ///
    /// # Safety
    /// FFI / raw cursor.
    unsafe fn ungetc(&mut self, nch: i32);

    fn consumed(&self) -> i32;
    fn add_consumed(&mut self, n: i32);

    /// The `{locinfo, file, width}` initialiser's `file` member, plus the redundant
    /// `#ifdef STRING ctx.file = file;` that follows it.
    fn fp_raw(&self) -> Self::Raw;
    /// `#ifdef STRING file = ctx.file;` — STRING **and** STRING_LEN (the latter is
    /// nested inside `#ifdef STRING`, so the guard holds for both).
    fn fp_commit(&mut self, raw: Self::Raw);
    /// `#ifdef STRING_LEN if(ctx.length > length-consumed+1) ctx.length = length-consumed+1;`
    /// — a `size_t` comparison against an `int`, so a `width` of -1 compares as
    /// `SIZE_MAX` and always loses. Identity for every other source.
    fn fp_clamp(&self, length: i32) -> i32;
}

/// `_UNLOCK_FILE_(file)` for the FILE sources.
struct FileGuard(*mut FILE);
impl Drop for FileGuard {
    fn drop(&mut self) {
        unsafe { _unlock_file(self.0) }
    }
}

/// `_UNLOCK_FILE_(file)` for CONSOLE — `_unlock_file(stdin)`, with `stdin` re-evaluated
/// at the unlock exactly as the macro does.
struct ConsoleGuard;
impl Drop for ConsoleGuard {
    fn drop(&mut self) {
        unsafe { _unlock_file(__iob_func()) }
    }
}

/// `_UNLOCK_FILE_(file)` for the string sources — `do {} while(0)`.
struct NoGuard;

macro_rules! file_source {
    ($name:ident, $w:ty, $raw:ident, $eof:expr, $get:expr, $unget:expr) => {
        struct $name {
            file: *mut FILE,
            consumed: i32,
        }
        impl Source for $name {
            type W = $w;
            type Raw = $raw;
            type Guard = FileGuard;
            const EOF_SENTINEL: i32 = $eof;  // nolint: const-contradiction
            unsafe fn lock(&mut self) -> FileGuard {
                _lock_file(self.file);
                FileGuard(self.file)
            }
            unsafe fn getc(&mut self) -> i32 {
                self.consumed += 1;
                #[allow(clippy::redundant_closure_call)]
                $get(self.file)
            }
            unsafe fn ungetc(&mut self, nch: i32) {
                #[allow(clippy::redundant_closure_call)]
                $unget(nch, self.file);
                self.consumed -= 1;
            }
            fn consumed(&self) -> i32 {
                self.consumed
            }
            fn add_consumed(&mut self, n: i32) {
                self.consumed += n;
            }
            fn fp_raw(&self) -> $raw {
                $raw(self.file)
            }
            fn fp_commit(&mut self, _raw: $raw) {}
            fn fp_clamp(&self, length: i32) -> i32 {
                length
            }
        }
    };
}

file_source!(
    SrcFileA,
    Narrow,
    RawFileA,
    -1,
    |f| fgetc(f),
    |c: i32, f| ungetc(c, f)
);
file_source!(
    SrcFileW,
    Wide,
    RawFileW,
    0xFFFF,
    |f| fgetwc(f) as i32,
    |c: i32, f| ungetwc(c as u16, f)
);

macro_rules! console_source {
    ($name:ident, $w:ty, $eof:expr) => {
        struct $name {
            consumed: i32,
        }
        impl Source for $name {
            type W = $w;
            type Raw = RawCon;
            type Guard = ConsoleGuard;
            const EOF_SENTINEL: i32 = $eof;  // nolint: const-contradiction
            unsafe fn lock(&mut self) -> ConsoleGuard {
                _lock_file(__iob_func());
                ConsoleGuard
            }
            unsafe fn getc(&mut self) -> i32 {
                self.consumed += 1;
                _getch()
            }
            unsafe fn ungetc(&mut self, nch: i32) {
                _ungetch(nch);
                self.consumed -= 1;
            }
            fn consumed(&self) -> i32 {
                self.consumed
            }
            fn add_consumed(&mut self, n: i32) {
                self.consumed += n;
            }
            fn fp_raw(&self) -> RawCon {
                // `struct ctx = {locinfo, 0, width}` — CONSOLE has no `file`.
                RawCon
            }
            fn fp_commit(&mut self, _raw: RawCon) {}
            fn fp_clamp(&self, length: i32) -> i32 {
                length
            }
        }
    };
}

console_source!(SrcConA, Narrow, -1);
// NOT a typo, and NOT `-1`: CONSOLE does not `#undef _EOF_`, so the wide console engine
// tests `nch == WEOF` against a `_getch()` that reports EOF_SENTINEL as -1. msvcrt's bug, kept.
console_source!(SrcConW, Wide, 0xFFFF);

macro_rules! string_source {
    ($name:ident, $w:ty, $raw:ident, $elem:ty, $load:expr) => {
        struct $name {
            file: *const $elem,
            consumed: i32,
        }
        impl Source for $name {
            type W = $w;
            type Raw = $raw;
            type Guard = NoGuard;
            const EOF_SENTINEL: i32 = 0; // `#undef _EOF_` / `#define _EOF_ 0`  // nolint: const-contradiction
            unsafe fn lock(&mut self) -> NoGuard {
                NoGuard
            }
            unsafe fn getc(&mut self) -> i32 {
                self.consumed += 1;
                #[allow(clippy::redundant_closure_call)]
                let c = $load(*self.file);
                self.file = self.file.add(1);
                c
            }
            unsafe fn ungetc(&mut self, _nch: i32) {
                self.file = self.file.sub(1);
                self.consumed -= 1;
            }
            fn consumed(&self) -> i32 {
                self.consumed
            }
            fn add_consumed(&mut self, n: i32) {
                self.consumed += n;
            }
            fn fp_raw(&self) -> $raw {
                $raw(self.file)
            }
            fn fp_commit(&mut self, raw: $raw) {
                self.file = raw.0;
            }
            fn fp_clamp(&self, length: i32) -> i32 {
                length
            }
        }
    };
}

// `_GETC_` for STRING casts narrow bytes to `(unsigned char)`; `_GETC_FUNC_` (RawStrA)
// does not. The two really do disagree — see RawStrA.
string_source!(SrcStrA, Narrow, RawStrA, i8, |c: i8| c as u8 as i32);
string_source!(SrcStrW, Wide, RawStrW, u16, |c: u16| c as i32);

macro_rules! string_len_source {
    ($name:ident, $w:ty, $raw:ident, $elem:ty, $load:expr) => {
        struct $name {
            file: *const $elem,
            length: usize,
            consumed: i32,
        }
        impl Source for $name {
            type W = $w;
            type Raw = $raw;
            type Guard = NoGuard;
            const EOF_SENTINEL: i32 = 0;  // nolint: const-contradiction
            unsafe fn lock(&mut self) -> NoGuard {
                NoGuard
            }
            /// `(consumed++, consumed>length ? '\0' : *file++)`.
            ///
            /// The increment happens **before** the limit test and is NOT undone, so at
            /// the limit this yields 0 and still counts one consumed character. (The
            /// libcntpr copy of scanf.h does it the other way round — that engine is
            /// `cnt_rust::core_rs::scanf` and it is not this one.) `consumed` is `int`
            /// and `length` is `size_t`, so the C converts the int to unsigned for the
            /// comparison; `as usize` is exactly that conversion.
            unsafe fn getc(&mut self) -> i32 {
                self.consumed += 1;
                if self.consumed as usize > self.length {
                    return 0;
                }
                #[allow(clippy::redundant_closure_call)]
                let c = $load(*self.file);
                self.file = self.file.add(1);
                c
            }
            unsafe fn ungetc(&mut self, _nch: i32) {
                self.file = self.file.sub(1);
                self.consumed -= 1;
            }
            fn consumed(&self) -> i32 {
                self.consumed
            }
            fn add_consumed(&mut self, n: i32) {
                self.consumed += n;
            }
            fn fp_raw(&self) -> $raw {
                $raw(self.file)
            }
            fn fp_commit(&mut self, raw: $raw) {
                self.file = raw.0;
            }
            fn fp_clamp(&self, length: i32) -> i32 {
                // `if(ctx.length > length-consumed+1) ctx.length = length-consumed+1;`
                // — size_t arithmetic throughout, then a truncating assignment back to
                // `int`. `length` here is `ctx.length`, which is `width`: -1 when no
                // width was given, and (size_t)(-1) is SIZE_MAX, so the clamp always
                // fires in that case.
                let lim = self.length.wrapping_sub(self.consumed as usize).wrapping_add(1);
                if length as usize > lim {
                    lim as i32
                } else {
                    length
                }
            }
        }
    };
}

string_len_source!(SrcStrNA, Narrow, RawStrA, i8, |c: i8| c as u8 as i32);
string_len_source!(SrcStrNW, Wide, RawStrW, u16, |c: u16| c as i32);

// ── the `%[` bitmap ───────────────────────────────────────────────────────────────────

/// `RtlSetBits(&bitMask, start, count)`.
///
/// **The one place this file is not a literal transcription, and it cannot be.** The C
/// passes `*format` — a *signed* `char` for the narrow engine — as ntdll's `ULONG`
/// starting index. A format byte >= 0x80 therefore arrives as ~0xFFFFFF80, and
/// `RtlSetBits` range-checks only under `ASSERT` (a no-op in a release ntdll), so
/// `%[\x80]` is a wild write roughly 128 MB below a 32-byte heap block. That is
/// undefined behaviour, not behaviour, and there is nothing to reproduce.
///
/// So out-of-range indices are clamped away, which reproduces everything that IS
/// defined: the bit is not in the map either way, `RtlAreBitsSet` would refuse the same
/// index on the scanning side, and the character simply never matches. What is lost is
/// only the heap corruption.
fn set_bits(buf: &mut [u64], bits: u32, start: u32, count: u32) {
    if count == 0 {
        return;
    }
    let end = start.saturating_add(count).min(bits);
    let mut k = start.min(bits);
    while k < end {
        buf[(k >> 6) as usize] |= 1u64 << (k & 63);
        k += 1;
    }
}

/// `RtlAreBitsSet(&bitMask, nch, 1)` — FALSE for an index past the map, exactly as the
/// real one is (`sdk/lib/rust/bitmap/src/ops.rs::are_bits_set` returns false on
/// `start + length > size`).
fn bit_is_set(buf: &[u64], bits: u32, idx: u32) -> bool {
    idx < bits && (buf[(idx >> 6) as usize] >> (idx & 63)) & 1 == 1
}

/// `RtlAreBitsClear(&bitMask, nch, 1)` — also FALSE past the map, which is NOT the
/// negation of `bit_is_set`. Both range checks fail closed, so an out-of-range character
/// matches neither an ordinary nor an inverted scanset.
fn bit_is_clear(buf: &[u64], bits: u32, idx: u32) -> bool {
    idx < bits && (buf[(idx >> 6) as usize] >> (idx & 63)) & 1 == 0
}

// ── the string/char destination element ───────────────────────────────────────────────

/// The element `charstring:`/`character:` (`char`) and `widecharstring:`/
/// `widecharacter:` (`wchar_t`) write. `from_nch` IS `_CHAR2SUPPORTED_`/
/// `_WIDE2SUPPORTED_`: both macros are the identity, and the only conversion the C
/// performs is the assignment's truncation to this type.
pub trait Dest: Copy {
    const ZERO: Self;
    fn from_nch(nch: i32) -> Self;
}
impl Dest for i8 {
    const ZERO: i8 = 0;
    fn from_nch(nch: i32) -> i8 {
        nch as i8
    }
}
impl Dest for u16 {
    const ZERO: u16 = 0;
    fn from_nch(nch: i32) -> u16 {
        nch as u16
    }
}

/// How a directive block finished. The C spells these as `break` (out of the `switch`),
/// `return rd` and `return _EOF_RET`; the last two must unlock, which the caller's
/// `Guard` does on the way out.
enum Arm {
    Done,
    RetRd,
    RetEof,
}

/// `charstring:` (scanf.h:460-494, `T = char`, `EOF_AT_EMPTY = true`) and
/// `widecharstring:` (scanf.h:495-531, `T = wchar_t`, `EOF_AT_EMPTY = false`).
///
/// The two blocks are the same text apart from the element type and the
/// `#if _MSVCR_VER >= 80` early return, which this CRT does **not** compile — hence the
/// const parameter rather than one shared body. `%s` into a `wchar_t*` at EOF_SENTINEL with
/// nothing read therefore returns 0 where the `char*` form returns `_EOF_RET`.
#[allow(clippy::too_many_arguments)]
unsafe fn string_arm<W, S, T, const SECURE: bool, const EOF_AT_EMPTY: bool>(
    src: &mut S,
    ap: &mut *mut u8,
    nch: &mut i32,
    width: &mut i32,
    st: &mut i32,
    suppress: bool,
    rd: i32,
) -> Arm
where
    W: Width,
    S: Source<W = W>,
    T: Dest,
{
    let mut sptr: *mut T = if suppress {
        core::ptr::null_mut()
    } else {
        va_next(ap)
    };
    let sptr_beg = sptr;
    let mut size: u32 = if !SECURE || suppress {
        UINT_MAX
    } else {
        va_next::<u32>(ap)
    };

    // skip initial whitespace
    while *nch != S::EOF_SENTINEL && W::is_space(*nch) {
        *nch = src.getc();
    }
    // read until whitespace
    while *width != 0 && *nch != S::EOF_SENTINEL && !W::is_space(*nch) {
        if !suppress {
            *sptr = T::from_nch(*nch);
            sptr = sptr.add(1);
            if size > 1 {
                size -= 1;
            } else {
                // Truncation blows away the WHOLE result, not just the tail: the C
                // writes the terminator at `sptr_beg`. No `_invalid_parameter`, no
                // errno, no -1 — it returns the fields assigned BEFORE this directive.
                *sptr_beg = T::ZERO;
                return Arm::RetRd;
            }
        }
        *st += 1;
        *nch = src.getc();
        if *width > 0 {
            *width -= 1;
        }
    }
    if EOF_AT_EMPTY && *nch == S::EOF_SENTINEL && rd == 0 && *st == 0 {
        return Arm::RetEof;
    }
    if *st != 0 && !suppress {
        *sptr = T::ZERO;
    }
    Arm::Done
}

/// `character:` (scanf.h:550-575, `T = char`) and `widecharacter:` (scanf.h:576-601,
/// `T = wchar_t`) — identical text, no `_MSVCR_VER` asymmetry here.
///
/// Two differences from `string_arm` that look like typos and are not: the size test is
/// `if(size)`, not `if(size>1)`, and it happens **before** the store; and there is no
/// terminator on the normal path, because `%c` does not NUL-terminate. The overflow path
/// still writes one, at the buffer start.
unsafe fn char_arm<W, S, T, const SECURE: bool>(
    src: &mut S,
    ap: &mut *mut u8,
    nch: &mut i32,
    width: &mut i32,
    st: &mut i32,
    suppress: bool,
) -> Arm
where
    W: Width,
    S: Source<W = W>,
    T: Dest,
{
    let mut str_: *mut T = if suppress {
        core::ptr::null_mut()
    } else {
        va_next(ap)
    };
    let pstr = str_;
    let mut size: u32 = if !SECURE || suppress {
        UINT_MAX
    } else {
        va_next::<u32>(ap)
    };

    if *width == -1 {
        *width = 1;
    }
    while *width != 0 && *nch != S::EOF_SENTINEL {
        if !suppress {
            if size != 0 {
                size -= 1;
            } else {
                *pstr = T::ZERO;
                return Arm::RetRd;
            }
            *str_ = T::from_nch(*nch);
            str_ = str_.add(1);
        }
        *st += 1;
        *width -= 1;
        *nch = src.getc();
    }
    Arm::Done
}

// ── the engine (scanf.h:198-727) ──────────────────────────────────────────────────────

/// `_FUNCTION_` — the body all sixteen instantiations share.
///
/// `format` is a raw cursor walked exactly as the C walks it, including where the C
/// walks off the end: an unterminated `%[` (`"%[abc"`) leaves `*format == 0`, the outer
/// `format++` steps past the NUL, and `while (*format)` then reads out of bounds. That
/// is msvcrt's bug and it is transcribed rather than papered over with a bound the C
/// does not have.
unsafe fn scanf_engine<W, S, const SECURE: bool>(
    src: &mut S,
    mut format: *const W::Ch,
    locale: sys::_locale_t,
    valist: *mut u8,
) -> i32
where
    W: Width,
    S: Source<W = W>,
{
    let mut ap = valist;
    let mut rd: i32 = 0;
    let mut nch: i32;

    // BEFORE the lock, and before anything else (scanf.h:202).
    if W::fmt(*format) == 0 {
        return 0;
    }
    // TRACE(...) — Wine debug channel, dropped (see msvcrt/mbcs.rs).

    let _guard = src.lock();

    nch = src.getc();
    if nch == S::EOF_SENTINEL {
        return W::EOF_RET;
    }

    // `if(!locale) locinfo = get_locinfo(); else locinfo = locale->locinfo;`
    let locinfo = crate::locinfo_of(locale as *mut c_void);

    while W::fmt(*format) != 0 {
        if W::is_space(W::fmt(*format)) {
            // A whitespace character in the format matches any run, including none.
            while nch != S::EOF_SENTINEL && W::is_space(nch) {
                nch = src.getc();
            }
        } else if W::fmt(*format) == b'%' as i32 {
            let mut st: i32 = 0;
            let mut suppress = false;
            let mut width: i32 = 0;
            let mut h_prefix: i32 = 0;
            let mut l_prefix = false;
            let mut big_l_prefix = false;
            let mut w_prefix = false;
            let mut i64_prefix = false;
            let mut prefix_finished = false;
            format = format.add(1);

            if W::fmt(*format) == b'*' as i32 {
                format = format.add(1);
                suppress = true;
            }

            // Width digits and prefix characters INTERLEAVE: the digit scan re-runs at
            // the top of every prefix pass, so `%1h2d` accumulates a width of 12.
            while !prefix_finished {
                while W::fmt(*format) >= b'0' as i32 && W::fmt(*format) <= b'9' as i32 {
                    width = width.wrapping_mul(10);
                    width = width.wrapping_add(W::fmt(*format) - b'0' as i32);
                    format = format.add(1);
                }

                const H: i32 = b'h' as i32;
                const L_LOW: i32 = b'l' as i32;
                const W_LOW: i32 = b'w' as i32;
                const L_UP: i32 = b'L' as i32;
                const I_UP: i32 = b'I' as i32;
                const Z_LOW: i32 = b'z' as i32;
                match W::fmt(*format) {
                    H => h_prefix += 1,
                    L_LOW => {
                        if W::fmt(*format.add(1)) == L_LOW {
                            i64_prefix = true;
                            format = format.add(1);
                        }
                        l_prefix = true;
                    }
                    W_LOW => w_prefix = true,
                    L_UP => big_l_prefix = true,
                    I_UP => {
                        if W::fmt(*format.add(1)) == b'6' as i32
                            && W::fmt(*format.add(2)) == b'4' as i32
                        {
                            i64_prefix = true;
                            format = format.add(2);
                        } else if W::fmt(*format.add(1)) == b'3' as i32
                            && W::fmt(*format.add(2)) == b'2' as i32
                        {
                            format = format.add(2);
                        } else if PTR_IS_64 {
                            // C `/* fall through */` into `case 'z'`.
                            i64_prefix = true;
                        }
                    }
                    // `#if _MSVCR_VER == 0 || _MSVCR_VER >= 140` — compiled here.
                    Z_LOW => {
                        if PTR_IS_64 {
                            i64_prefix = true;
                        }
                    }
                    _ => prefix_finished = true,
                }
                if !prefix_finished {
                    format = format.add(1);
                }
            }
            if width == 0 {
                width = -1; // no width spec seen
            }

            let ty = W::fmt(*format);

            const P_LOW: i32 = b'p' as i32;
            const P_UP: i32 = b'P' as i32;
            const X_LOW: i32 = b'x' as i32;
            const X_UP: i32 = b'X' as i32;
            const O_LOW: i32 = b'o' as i32;
            const U_LOW: i32 = b'u' as i32;
            const D_LOW: i32 = b'd' as i32;
            const I_LOW: i32 = b'i' as i32;
            const E_LOW: i32 = b'e' as i32;
            const E_UP: i32 = b'E' as i32;
            const F_LOW: i32 = b'f' as i32;
            const G_LOW: i32 = b'g' as i32;
            const G_UP: i32 = b'G' as i32;
            const S_LOW: i32 = b's' as i32;
            const S_UP: i32 = b'S' as i32;
            const C_LOW: i32 = b'c' as i32;
            const C_UP: i32 = b'C' as i32;
            const N_LOW: i32 = b'n' as i32;
            const BRACKET: i32 = b'[' as i32;

            match ty {
                // ── number: p P x X o u d i (scanf.h:303-388) ─────────────────────
                P_LOW | P_UP | X_LOW | X_UP | O_LOW | U_LOW | D_LOW | I_LOW => 'arm: {
                    let mut base: i32 = match ty {
                        P_LOW | P_UP => {
                            if PTR_IS_64 {
                                i64_prefix = true;
                            }
                            16 // `/* fall through */` into case 'x'
                        }
                        X_LOW | X_UP => 16,
                        O_LOW => 8,
                        U_LOW | D_LOW => 10,
                        _ => 0, // 'i': generic integer
                    };
                    let is_p = ty == P_LOW || ty == P_UP;

                    let mut cur: u64 = 0;
                    let mut negative = false;
                    let mut seendigit = false;

                    while nch != S::EOF_SENTINEL && W::is_space(nch) {
                        nch = src.getc();
                    }
                    if nch == b'-' as i32 || nch == b'+' as i32 {
                        negative = nch == b'-' as i32;
                        nch = src.getc();
                        if width > 0 {
                            width -= 1;
                        }
                    }
                    // leading indication of base
                    if width != 0 && nch == b'0' as i32 && !is_p {
                        nch = src.getc();
                        if width > 0 {
                            width -= 1;
                        }
                        seendigit = true;
                        if width != 0 && (nch == b'x' as i32 || nch == b'X' as i32) {
                            if base == 0 {
                                base = 16;
                            }
                            if base == 16 {
                                nch = src.getc();
                                if width > 0 {
                                    width -= 1;
                                }
                                seendigit = false;
                            }
                        } else if base == 0 {
                            base = 8;
                        }
                    }
                    if base == 0 {
                        base = 10;
                    }
                    while width != 0 && nch == b'0' as i32 {
                        nch = src.getc();
                        if width > 0 {
                            width -= 1;
                        }
                        seendigit = true;
                    }
                    if width != 0 && W::char2digit(nch, base) != -1 {
                        cur = W::char2digit(nch, base) as u64;
                        nch = src.getc();
                        if width > 0 {
                            width -= 1;
                        }
                        seendigit = true;
                    }
                    while width != 0 && nch != S::EOF_SENTINEL && W::char2digit(nch, base) != -1 {
                        cur = cur
                            .wrapping_mul(base as u64)
                            .wrapping_add(W::char2digit(nch, base) as u64);
                        nch = src.getc();
                        if width > 0 {
                            width -= 1;
                        }
                        seendigit = true;
                    }
                    if !seendigit {
                        break 'arm; // not a valid number
                    }
                    st = 1;
                    if !suppress {
                        // `*va_arg(ap, type*) = negative ? -cur : cur` — `cur` is
                        // ULONGLONG, so `-cur` is the unsigned negation and the store
                        // truncates to the target width.
                        let v = if negative { cur.wrapping_neg() } else { cur };
                        if i64_prefix {
                            *va_next::<*mut i64>(&mut ap) = v as i64;
                        } else if l_prefix {
                            // Windows LONG is 32-bit on all three targets.
                            *va_next::<*mut i32>(&mut ap) = v as i32;
                        } else if h_prefix == 1 {
                            *va_next::<*mut i16>(&mut ap) = v as i16;
                        } else {
                            // `#if _MSVCR_VER >= 140  else if (h_prefix == 2)
                            // _SET_NUMBER_(char)` is NOT compiled at _MSVCR_VER == 0,
                            // so `%hhd` lands here and stores a full `int`.
                            *va_next::<*mut i32>(&mut ap) = v as i32;
                        }
                    }
                }
                // ── float: e E f g G (scanf.h:389-437) ────────────────────────────
                E_LOW | E_UP | F_LOW | G_LOW | G_UP => 'arm: {
                    let mut ctx = FpCtx {
                        raw: src.fp_raw(),
                        length: width,
                        read: 0,
                        cur: 0,
                        unget: 0,
                        err: false,
                    };
                    // `int negative = 0;` — declared, never assigned in this arm, so
                    // `negative ? -cur : cur` is always `cur`. Dead in the C; dropped.

                    while nch != S::EOF_SENTINEL && W::is_space(nch) {
                        nch = src.getc();
                    }
                    if nch == S::EOF_SENTINEL {
                        break 'arm;
                    }
                    ctx.unget = nch;
                    ctx.length = src.fp_clamp(ctx.length);

                    let mut cur = 0.0f64;
                    crate::fpnum::scanf_fpnum_double(&mut ctx, locinfo, &mut cur);

                    if rd == 0 && ctx.err {
                        return W::EOF_RET;
                    }
                    if ctx.err || ctx.read == 0 {
                        break 'arm;
                    }
                    src.add_consumed(ctx.read);
                    src.fp_commit(ctx.raw);
                    nch = ctx.cur;

                    st = 1;
                    if !suppress {
                        if big_l_prefix || l_prefix {
                            *va_next::<*mut f64>(&mut ap) = cur;
                        } else {
                            *va_next::<*mut f32>(&mut ap) = cur as f32;
                        }
                    }
                }
                // ── string: s S (scanf.h:444-531) ─────────────────────────────────
                //
                // `%s` reads the engine's native width and `%S` the other one; `h`
                // forces narrow, `w`/`l` force wide. Both `case`s are pure dispatch —
                // every path is a `goto`, so there is no fallthrough to model.
                S_LOW | S_UP => {
                    let wide_target = if w_prefix || l_prefix {
                        true
                    } else if h_prefix != 0 {
                        false
                    } else if ty == S_LOW {
                        W::WIDE
                    } else {
                        !W::WIDE
                    };
                    let arm = if wide_target {
                        string_arm::<W, S, u16, SECURE, false>(
                            src, &mut ap, &mut nch, &mut width, &mut st, suppress, rd,
                        )
                    } else {
                        string_arm::<W, S, i8, SECURE, true>(
                            src, &mut ap, &mut nch, &mut width, &mut st, suppress, rd,
                        )
                    };
                    match arm {
                        Arm::Done => {}
                        Arm::RetRd => return rd,
                        Arm::RetEof => return W::EOF_RET,
                    }
                }
                // ── char: c C (scanf.h:534-601) ───────────────────────────────────
                C_LOW | C_UP => {
                    let wide_target = if w_prefix || l_prefix {
                        true
                    } else if h_prefix != 0 {
                        false
                    } else if ty == C_LOW {
                        W::WIDE
                    } else {
                        !W::WIDE
                    };
                    let arm = if wide_target {
                        char_arm::<W, S, u16, SECURE>(
                            src, &mut ap, &mut nch, &mut width, &mut st, suppress,
                        )
                    } else {
                        char_arm::<W, S, i8, SECURE>(
                            src, &mut ap, &mut nch, &mut width, &mut st, suppress,
                        )
                    };
                    match arm {
                        Arm::Done => {}
                        Arm::RetRd => return rd,
                        Arm::RetEof => return W::EOF_RET,
                    }
                }
                // ── %n (scanf.h:602-621) ──────────────────────────────────────────
                N_LOW => {
                    if !suppress {
                        *va_next::<*mut i32>(&mut ap) = src.consumed() - 1;
                    }
                    // "Execution of a %n directive does not increment the assignment
                    // count" — the C forces `suppress` to get that, and the comment
                    // there records that the Corrigendum disagrees and Windows does
                    // not. Kept.
                    suppress = true;
                    st = 1;
                }
                // ── scanset %[ (scanf.h:622-689) ──────────────────────────────────
                BRACKET => {
                    // Names as the C has them: `str` is the FIXED buffer start (what the
                    // overflow path terminates), `sptr` is the walker.
                    let str_: *mut W::Ch = if suppress {
                        core::ptr::null_mut()
                    } else {
                        va_next(&mut ap)
                    };
                    let mut sptr = str_;
                    let mut invert = false;
                    let mut size: u32 = if !SECURE || suppress {
                        UINT_MAX
                    } else {
                        va_next::<u32>(&mut ap)
                    };

                    // The C heap-allocates `_BITMAPSIZE_/8` bytes and drives
                    // RtlInitializeBitMap/RtlSetBits/RtlAreBitsSet. A stack bitmap is
                    // the same data structure without the allocation, the ntdll import
                    // or the two HeapFree exits.
                    let mut bitmap = W::zero_bitmap();
                    let mask = bitmap.as_mut();
                    let bits = W::BITMAP_BITS;

                    format = format.add(1);
                    if W::fmt(*format) == b'^' as i32 {
                        invert = true;
                        format = format.add(1);
                    }
                    if W::fmt(*format) == b']' as i32 {
                        set_bits(mask, bits, b']' as u32, 1);
                        format = format.add(1);
                    }
                    while W::fmt(*format) != 0 && W::fmt(*format) != b']' as i32 {
                        // msvcrt's range syntax is LOOKAHEAD: `a-z` is recognised at
                        // `a`, by testing `format[1]=='-' && format[2] && format[2]!=']'`.
                        // (libcntpr's copy — and therefore cnt-rust's port of it — uses
                        // a lookbehind form that differs on `%[-a]` and `%[a-]`.)
                        // `%[a-z]` and `%[z-a]` are documented as equivalent, hence the
                        // swap.
                        if W::fmt(*format.add(1)) == b'-' as i32
                            && W::fmt(*format.add(2)) != 0
                            && W::fmt(*format.add(2)) != b']' as i32
                        {
                            let f0 = W::fmt(*format);
                            let f2 = W::fmt(*format.add(2));
                            if f0 < f2 {
                                set_bits(mask, bits, f0 as u32, (f2 - f0 + 1) as u32);
                            } else {
                                set_bits(mask, bits, f2 as u32, (f0 - f2 + 1) as u32);
                            }
                            format = format.add(2);
                        } else {
                            set_bits(mask, bits, W::fmt(*format) as u32, 1);
                        }
                        format = format.add(1);
                    }

                    // read until char is not suitable
                    while width != 0 && nch != S::EOF_SENTINEL {
                        let hit = if !invert {
                            bit_is_set(mask, bits, nch as u32)
                        } else {
                            bit_is_clear(mask, bits, nch as u32)
                        };
                        if hit {
                            if !suppress {
                                *sptr = <W::Ch as Dest>::from_nch(nch);
                                sptr = sptr.add(1);
                            }
                        } else {
                            break;
                        }
                        st += 1;
                        nch = src.getc();
                        if width > 0 {
                            width -= 1;
                        }
                        if size > 1 {
                            size -= 1;
                        } else {
                            // `*str = 0` — the buffer START again, as in `charstring:`.
                            *str_ = <W::Ch as Dest>::ZERO;
                            return rd;
                        }
                    }
                    // terminate — unconditionally, even when nothing matched.
                    if !suppress {
                        *sptr = <W::Ch as Dest>::ZERO;
                    }
                }
                // ── default, which is also `%%` (scanf.h:690-705) ─────────────────
                _ => {
                    // "if a percent sign is followed by a character that has no meaning
                    // as a format-control character, that character and the following
                    // characters are treated as an ordinary sequence of characters".
                    while nch != S::EOF_SENTINEL && W::is_space(nch) {
                        nch = src.getc();
                    }
                    if W::narrow(nch) == W::fmt(*format) {
                        suppress = true; // whoops, no field to be read
                        st = 1; // but we got what we expected
                        nch = src.getc();
                    }
                }
            }

            if st != 0 && !suppress {
                rd += 1;
            } else if st == 0 {
                break;
            }
        } else {
            // A non-whitespace character must match, but is not stored.
            if W::narrow(nch) == W::fmt(*format) {
                nch = src.getc();
            } else {
                break;
            }
        }
        format = format.add(1);
    }

    if nch != S::EOF_SENTINEL {
        src.ungetc(nch);
    }
    // TRACE("returning %d\n", rd) — dropped.
    rd
}

// ══════════════════════════════════════════════════════════════════════════════════════
// The sixteen `_FUNCTION_` instantiations (scanf.h:53-146)
//
// These are `static` in scanf.c, so they are new global symbols rather than overrides.
// They exist for `dll/win32/msvcrt/rust_scanf_shim.c`, whose 40 `va_start` trampolines
// are the irreducible C half: stable Rust cannot DEFINE a C-variadic. Receiving an
// already-started cursor is ordinary code (a `va_list` is a flat `char *` on all three
// Windows targets), which is why the four `__stdio_common_*` doors below need no C at
// all.
// ══════════════════════════════════════════════════════════════════════════════════════

macro_rules! shim {
    // FILE / CONSOLE forms: no separate buffer argument.
    ($name:ident, $w:ty, $src:ident, $secure:literal, $ch:ty, ($($arg:ident : $aty:ty),*), $mk:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(
            $($arg: $aty,)*
            format: *const $ch,
            locale: sys::_locale_t,
            ap: *mut u8,
        ) -> i32 {
            #[allow(clippy::redundant_closure_call)]
            let mut src = $mk($($arg),*);
            scanf_engine::<$w, $src, $secure>(&mut src, format, locale, ap)
        }
    };
}

// ── FILE (scanf.h:129-145) ────────────────────────────────────────────────────────────
shim!(vfscanf_l,    Narrow, SrcFileA, false, i8,  (file: *mut FILE), |f| SrcFileA { file: f, consumed: 0 });
shim!(vfscanf_s_l,  Narrow, SrcFileA, true,  i8,  (file: *mut FILE), |f| SrcFileA { file: f, consumed: 0 });
shim!(vfwscanf_l,   Wide,   SrcFileW, false, u16, (file: *mut FILE), |f| SrcFileW { file: f, consumed: 0 });
shim!(vfwscanf_s_l, Wide,   SrcFileW, true,  u16, (file: *mut FILE), |f| SrcFileW { file: f, consumed: 0 });

// ── STRING (scanf.h:107-119) ──────────────────────────────────────────────────────────
shim!(vsscanf_l,    Narrow, SrcStrA, false, i8,  (file: *const i8),  |f| SrcStrA { file: f, consumed: 0 });
shim!(vsscanf_s_l,  Narrow, SrcStrA, true,  i8,  (file: *const i8),  |f| SrcStrA { file: f, consumed: 0 });
shim!(vswscanf_l,   Wide,   SrcStrW, false, u16, (file: *const u16), |f| SrcStrW { file: f, consumed: 0 });
shim!(vswscanf_s_l, Wide,   SrcStrW, true,  u16, (file: *const u16), |f| SrcStrW { file: f, consumed: 0 });

// ── STRING_LEN (scanf.h:85-97) ────────────────────────────────────────────────────────
shim!(vsnscanf_l,    Narrow, SrcStrNA, false, i8,  (file: *const i8,  length: usize), |f, l| SrcStrNA { file: f, length: l, consumed: 0 });
shim!(vsnscanf_s_l,  Narrow, SrcStrNA, true,  i8,  (file: *const i8,  length: usize), |f, l| SrcStrNA { file: f, length: l, consumed: 0 });
shim!(vsnwscanf_l,   Wide,   SrcStrNW, false, u16, (file: *const u16, length: usize), |f, l| SrcStrNW { file: f, length: l, consumed: 0 });
shim!(vsnwscanf_s_l, Wide,   SrcStrNW, true,  u16, (file: *const u16, length: usize), |f, l| SrcStrNW { file: f, length: l, consumed: 0 });

// ── CONSOLE (scanf.h:53-65) ───────────────────────────────────────────────────────────
shim!(vcscanf_l,     Narrow, SrcConA, false, i8,  (), || SrcConA { consumed: 0 });
shim!(vcscanf_s_l,   Narrow, SrcConA, true,  i8,  (), || SrcConA { consumed: 0 });
shim!(vcwscanf_l,    Wide,   SrcConW, false, u16, (), || SrcConW { consumed: 0 });
shim!(vcwscanf_s_l,  Wide,   SrcConW, true,  u16, (), || SrcConW { consumed: 0 });

// ══════════════════════════════════════════════════════════════════════════════════════
// scanf.c:664-736 — the four `__stdio_common_*` doors.
//
// Not variadic: they take an already-started `va_list`, so they are pure Rust. The
// `FIXME("options %#I64x not handled")` in each is a Wine debug channel and is dropped
// (never lowered to a no-op `fn` — AGENTS.md), which leaves the `options & ~MASK` test
// with nothing to do, so it is dropped with it rather than left as dead arithmetic.
// The option bits that DO matter are still read below.
//
// These are NOT in msvcrt.spec — compiled but unexported, reached only through the
// inline `sscanf_s`/`fscanf_s`/… in `dll/win32/msvcrt/include/stdio.h`.
// ══════════════════════════════════════════════════════════════════════════════════════

/// `corecrt_stdio_config.h:19`.
const CRT_INTERNAL_SCANF_SECURECRT: u64 = 0x0001;

#[no_mangle]
pub unsafe extern "C" fn __stdio_common_vsscanf(
    options: u64,
    input: *const i8,
    length: usize,
    format: *const i8,
    locale: sys::_locale_t,
    valist: *mut u8,
) -> i32 {
    // LEGACY_WIDE_SPECIFIERS only affects the wide scanf; LEGACY_MSVCRT_COMPATIBILITY
    // affects nan/inf parsing, which is not implemented at all at this vintage.
    if options & CRT_INTERNAL_SCANF_SECURECRT != 0 {
        vsnscanf_s_l(input, length, format, locale, valist)
    } else {
        vsnscanf_l(input, length, format, locale, valist)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __stdio_common_vswscanf(
    options: u64,
    input: *const u16,
    length: usize,
    format: *const u16,
    locale: sys::_locale_t,
    valist: *mut u8,
) -> i32 {
    if options & CRT_INTERNAL_SCANF_SECURECRT != 0 {
        vsnwscanf_s_l(input, length, format, locale, valist)
    } else {
        vsnwscanf_l(input, length, format, locale, valist)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __stdio_common_vfscanf(
    options: u64,
    file: *mut FILE,
    format: *const i8,
    locale: sys::_locale_t,
    valist: *mut u8,
) -> i32 {
    if options & CRT_INTERNAL_SCANF_SECURECRT != 0 {
        vfscanf_s_l(file, format, locale, valist)
    } else {
        vfscanf_l(file, format, locale, valist)
    }
}

#[no_mangle]
pub unsafe extern "C" fn __stdio_common_vfwscanf(
    options: u64,
    file: *mut FILE,
    format: *const u16,
    locale: sys::_locale_t,
    valist: *mut u8,
) -> i32 {
    if options & CRT_INTERNAL_SCANF_SECURECRT != 0 {
        vfwscanf_s_l(file, format, locale, valist)
    } else {
        vfwscanf_l(file, format, locale, valist)
    }
}

// scanf.c:798-820 — `vsscanf`/`vswscanf` are `#if _MSVCR_VER==120`. This DLL builds at
// _MSVCR_VER=0, so they are not here either; grep for the name and its absence is the
// answer (the same convention printf_shells.rs uses).
