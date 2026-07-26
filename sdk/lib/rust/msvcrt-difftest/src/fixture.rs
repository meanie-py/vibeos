//! What the generated mbcs cases are given to work with.
//!
//! `rustconv difftest-gen` decides the SHAPE of every case — which arguments to sweep,
//! what "equal" means for the result — but it cannot know what a good MBCS input looks
//! like. That is this file. It is the one part of the mbcs harness that is judgement
//! rather than derivation, so the judgement is written down.
//!
//! The governing rule is that a case which cannot FAIL is worse than no case, because a
//! green suite is what stops anyone looking again. Everything below exists to keep the
//! inputs capable of provoking a difference.

use core::ffi::c_void;
use core::fmt;

// ── the pinned state ──────────────────────────────────────────────────────────

extern "C" {
    fn mbcs_fixture_install(cp: u32) -> i32;
    fn mbcs_fixture_check() -> i32;
    fn mbcs_ref_slot() -> *mut c_void;
    fn mbcs_subj_slot() -> *mut c_void;
    /// The reference's locinfo (namespaced) and the subject's (plain).
    fn msvcrt_ref_get_locinfo() -> *mut c_void;
    fn get_locinfo() -> *mut c_void;
}

/// Every codepage each case sweeps.
///
/// The five DBCS ones are the point; 1252 and 20127 are here because a bug that only
/// shows up in the SINGLE-byte path is still a bug, and because 20127 is the one that
/// reaches create_mbcinfo's built-in ASCII fallback (GetCPInfo deliberately fails for
/// it — see nlsimpl.c).
pub static CODEPAGES: &[u32] = &[932, 936, 949, 950, 1361, 1252, 20127];

/// Initial `mbstate_t` values every restartable-conversion case sweeps.
///
/// msvcrt types `mbstate_t` as a plain `int` holding the pending LEAD BYTE, so 0 means
/// "at a character boundary" and any other value means "resume this partial character".
/// Sweeping only 0 would leave the resume path — the half of `mbrtowc`/`mbrlen` that has
/// any state to get wrong — completely unreached, while the suite still reported a green
/// per-function row. 0x81 and 0xE0 are lead bytes in every DBCS codepage here; 0xFF is
/// not a lead byte in any of them, so it also grades the "stale state that cannot be
/// completed" path both sides must reject the same way.
pub static MBSTATES: [i32; 4] = [0, 0x81, 0xE0, 0xFF];

/// Installs a codepage's `threadmbcinfo` on BOTH sides and, on the way out, checks
/// neither side scribbled on it.
///
/// The check is in `Drop` so it runs per case without every case having to remember it.
/// A function that mutates its mbcinfo is a translation bug whose symptom would
/// otherwise appear in whichever unrelated case ran next.
pub struct MbcsFixture {
    cp: u32,
}

impl MbcsFixture {
    pub fn install(cp: u32) -> Self {
        assert_eq!(
            unsafe { mbcs_fixture_install(cp) },
            0,
            "could not build the mbcinfo fixture for cp{cp} — every case pinned against \
             it would grade against zeros"
        );
        Self { cp }
    }
}

impl Drop for MbcsFixture {
    fn drop(&mut self) {
        // Do not panic while already unwinding: a case that failed for a real reason
        // must report THAT, not be replaced by a double-panic abort.
        if std::thread::panicking() {
            return;
        }
        let which = unsafe { mbcs_fixture_check() };
        assert_eq!(
            which,
            0,
            "cp{}: the {} mutated its threadmbcinfo — the fixture is read-only state, \
             and every later case would have been graded against the damage",
            self.cp,
            if which == 1 { "REFERENCE" } else { "SUBJECT" }
        );
    }
}

// ── locales ───────────────────────────────────────────────────────────────────

/// The `_locale_t` values swept by every `_l` entry point.
pub struct Locales;

/// A `_locale_t`: `{ locinfo, mbcinfo }`. Built here rather than in C so each side gets
/// one aimed at ITS OWN state — see `Locales::pairs`.
#[repr(C)]
struct LocaleT {
    locinfo: *mut c_void,
    mbcinfo: *mut c_void,
}

static mut REF_LOCALE: LocaleT = LocaleT {
    locinfo: core::ptr::null_mut(),
    mbcinfo: core::ptr::null_mut(),
};
static mut SUBJ_LOCALE: LocaleT = LocaleT {
    locinfo: core::ptr::null_mut(),
    mbcinfo: core::ptr::null_mut(),
};

impl Locales {
    /// `(subject, reference)` pairs — never one pointer handed to both.
    ///
    /// A `_locale_t` carries `mbcinfo`. Sharing one would aim both sides at the same
    /// fixture, so a function that wrongly mutated it would have the reference read the
    /// mutated state and agree — hiding the very bug the two separate slots exist to
    /// catch, and defeating `MbcsFixture`'s drop check along with it.
    ///
    /// NULL is swept too, and is not redundant: it takes the `get_mbcinfo()` path
    /// instead of the explicit-locale one, and those are different code in every `_l`
    /// function.
    pub fn pairs(&self) -> impl Iterator<Item = (*mut c_void, *mut c_void)> {
        unsafe {
            let r = &raw mut REF_LOCALE;
            let s = &raw mut SUBJ_LOCALE;
            (*r).locinfo = msvcrt_ref_get_locinfo();
            (*r).mbcinfo = mbcs_ref_slot();
            (*s).locinfo = get_locinfo();
            (*s).mbcinfo = mbcs_subj_slot();
            [
                (core::ptr::null_mut(), core::ptr::null_mut()),
                (s.cast::<c_void>(), r.cast::<c_void>()),
            ]
            .into_iter()
        }
    }
}

pub static LOCALES: Locales = Locales;

// ── input strings ─────────────────────────────────────────────────────────────

/// A NUL-terminated byte string handed to the code under test.
#[derive(Clone, PartialEq, Eq)]
pub struct Bytes(Vec<u8>);

impl Bytes {
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    /// Byte offsets to use as a CURSOR into this string, for the `(start, cur)` family.
    ///
    /// Every position up to and including the terminator: `_mbsdec` walking back from
    /// the middle of a double-byte character is precisely the case that separates a
    /// correct implementation from one that just subtracts one.
    pub fn offsets(&self) -> impl Iterator<Item = usize> {
        0..self.0.len().min(Guarded::CAPACITY)
    }
}

/// Hex, because these are deliberately not valid UTF-8 and a failure message has to
/// show exactly which byte sequence provoked it.
impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\"")?;
        for &b in self.0.iter().take_while(|b| **b != 0) {
            if b.is_ascii_graphic() || b == b' ' {
                write!(f, "{}", b as char)?;
            } else {
                write!(f, "\\x{b:02x}")?;
            }
        }
        f.write_str("\"")
    }
}

pub struct Corpus {
    items: Vec<Bytes>,
}

impl Default for Corpus {
    fn default() -> Self {
        Self::new()
    }
}

impl Corpus {
    /// Characters swept by a non-exhaustive `unsigned int c` parameter.
    ///
    /// Both halves matter. The single bytes cover the SBCS paths and the boundaries the
    /// lead-byte test is written against (0x80, 0xA1, 0xDF, 0xFF). The two-byte values
    /// are real CP932 characters, including a fullwidth A/a pair so the DBCS case
    /// mapping has something to actually change — `_mbctolower` on a character with no
    /// case is a no-op both sides agree on.
    pub const CHARS: &'static [u32] = &[
        0x00, 0x20, 0x41, 0x5A, 0x61, 0x7A, 0x7F, 0x80, 0x81, 0xA1, 0xDF, 0xE0, 0xFC, 0xFF,
        0x8140, // ideographic space
        0x8260, // fullwidth A  — has a lowercase form
        0x8281, // fullwidth a  — has an uppercase form
        0x82A0, // hiragana A
        0x8340, // katakana A
        0x889F, // a kanji
        0x81FF, // lead byte + an invalid trail
        0xFFFF,
    ];

    pub fn new() -> Self {
        let mut items: Vec<&[u8]> = vec![
            b"",
            b"A",
            b"a",
            b"Z",
            b"abc",
            b"ABC",
            b"AbCdEf",
            b"Hello, World!",
            b"0123456789",
            // Single-byte high range: halfwidth katakana under CP932, plain latin-1
            // under 1252 — the same bytes classify differently per codepage, which is
            // what makes sweeping CODEPAGES worth doing.
            b"\xa1\xb1\xc1\xdf",
            // Well-formed DBCS.
            b"\x82\xa0\x82\xa2",
            b"\x88\x9f\x88\xa0",
            // DBCS mixed with ASCII — the case every "walk the string" loop gets wrong
            // first, because the stride changes mid-string.
            b"a\x82\xa0b",
            b"\x82\xa0A\x82\xa2",
            // A lead byte with NOTHING after it. The classic MBCS overrun: a naive
            // walker steps past the NUL looking for the trail byte.
            b"ab\x82",
            b"\x82",
            // Lead byte followed by an invalid trail.
            b"\x82\x20",
            // Fullwidth A / fullwidth a, so case functions have real work.
            b"\x82\x60\x82\x81",
            // All-trail-byte junk, and 0xFF which is a lead byte in no codepage here.
            b"\x40\x7e\xfc",
            b"\xff\xff",
        ];
        // A string long enough that the `n`-bounded functions have room to stop early
        // rather than always running to the terminator.
        items.push(b"\x82\xa0\x82\xa2\x82\xa4\x82\xa6\x82\xa8abcdefgh");

        Self {
            items: items.into_iter().map(|s| Bytes(with_nul(s))).collect(),
        }
    }

    pub fn items(&self) -> impl Iterator<Item = Bytes> + '_ {
        self.items.iter().cloned()
    }

    /// Two independent copies of each input, one per side.
    ///
    /// Separate storage even though both are read-only inputs: a function that writes
    /// through a `const` parameter is a real translation bug (a lost `const` in the
    /// signature), and with one shared buffer both sides would read the damage.
    pub fn pairs(&self) -> impl Iterator<Item = (Bytes, Bytes)> + '_ {
        self.items.iter().map(|s| (s.clone(), s.clone()))
    }

    /// Lengths for a `size_t n`.
    ///
    /// Capped at HALF the buffer, not all of it. In the `_mbsn*` family `n` counts
    /// CHARACTERS, and a DBCS character is two bytes, so `_mbsncpy_l(dst, src, n)` may
    /// legitimately write `2n`. Allowing `n == CAPACITY` therefore asks for a 64-byte
    /// write into a 32-byte buffer and both sides scribble past the end — which the
    /// canary correctly reports as a difference, in a message that reads exactly like a
    /// translation bug. (It did; that is why this is written down.)
    pub fn sizes(&self) -> impl Iterator<Item = usize> {
        [0usize, 1, 2, 3, 4, 5, 8, Guarded::CAPACITY / 2].into_iter()
    }
}

// ── output buffers ────────────────────────────────────────────────────────────

/// A writable buffer with a canary tail.
///
/// The canary is not a substitute for `oob_guard`'s PROT_NONE probe — that one catches
/// an out-of-bounds READ, which no value comparison can see. This catches a write that
/// lands just past the end, which would otherwise show up as a mysterious difference in
/// whatever the allocator put there next.
pub struct Guarded {
    buf: Vec<u8>,
}

impl Guarded {
    pub const CAPACITY: usize = 32;
    const CANARY: u8 = 0xCD;
    const TAIL: usize = 16;

    fn blank() -> Self {
        let mut buf = vec![0u8; Self::CAPACITY];
        buf.extend(core::iter::repeat_n(Self::CANARY, Self::TAIL));
        Self { buf }
    }

    /// Two blank buffers, for a function whose out-parameter is a DESTINATION.
    ///
    /// Blank, and allocated fresh per iteration: a reused buffer keeps the previous
    /// iteration's bytes, and a short write then leaves stale content that looks exactly
    /// like a correct one.
    pub fn pair() -> (Self, Self) {
        (Self::blank(), Self::blank())
    }

    /// Two buffers preloaded with `seed`, for a function that READS AND REWRITES its
    /// argument (`_mbslwr`, `_mbsnbset`, …). Handed a blank buffer those see an empty
    /// string, do nothing, and agree — see `mutates_in_place` in difftest_gen.py.
    pub fn seeded(seed: &Bytes) -> (Self, Self) {
        let mut a = Self::blank();
        let n = seed.0.len().min(Self::CAPACITY);
        a.buf[..n].copy_from_slice(&seed.0[..n]);
        a.buf[Self::CAPACITY - 1] = 0; // never let a seed eat the terminator
        let b = Self {
            buf: a.buf.clone(),
        };
        (a, b)
    }

    pub fn ptr(&mut self) -> *mut u8 {
        self.buf.as_mut_ptr()
    }

    /// The same buffer as `*mut u16`, for a wide out-parameter (`mbtowc`, `mbstowcs`).
    ///
    /// A separate accessor rather than a cast at the call site: the callee writes
    /// `wchar_t`s, so the usable element count is half, and a cast would let a
    /// byte-sized buffer accept a wchar-sized write and overrun by 2x. `CAPACITY` is
    /// even and the tail canary catches it if this is ever got wrong.
    pub fn wptr(&mut self) -> *mut u16 {
        self.buf.as_mut_ptr().cast::<u16>()
    }

    /// Do the two buffers hold the same bytes — and did either overrun?
    ///
    /// The whole buffer is compared, not just up to the first NUL: a function that
    /// writes the right string but scribbles past it is still wrong, and stopping at the
    /// terminator would call that a match.
    pub fn same(a: &Self, b: &Self) -> bool {
        a.canary_intact() && b.canary_intact() && a.buf == b.buf
    }

    fn canary_intact(&self) -> bool {
        self.buf[Self::CAPACITY..].iter().all(|&b| b == Self::CANARY)
    }
}

/// NUL-terminate, then pad with a fixed pattern.
///
/// The padding is not slack — it is what keeps an upstream over-read GRADEABLE.
///
/// msvcrt walks off the end of a string that ends in a truncated DBCS lead byte:
/// `_mbscmp_l("ab\x82", …)` reaches the 0x82, `_mbsnextc_l` returns `0x82<<8 | NUL`,
/// that is > 255, so both cursors advance by two and step straight over the
/// terminator. The Rust reproduces this faithfully — it is msvcrt's behaviour, not a
/// translation defect — but the reference and the subject hold separate allocations,
/// so each then reads different heap garbage and the comparison becomes a coin flip
/// reported as a divergence.
///
/// Padding both copies identically turns that back into a real test: the two sides
/// read the same bytes past the terminator, so they still have to AGREE about how far
/// they walk and what they conclude. A Rust that over-reads differently from the C is
/// caught; one that over-reads the same way is not punished for the C's own bug.
fn with_nul(s: &[u8]) -> Vec<u8> {
    let mut v = s.to_vec();
    v.push(0);
    // Deliberately non-zero and not a lead byte in any codepage here, so a walker that
    // runs on does something observable rather than immediately re-terminating.
    v.extend(core::iter::repeat_n(0x5A, 16));
    v
}
