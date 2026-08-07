//! Grades `__unDName` / `__unDNameEx` — undname.c, reference TU #8.
//!
//! 1,788 lines of paid Rust behind two exports, and until now not one differential row. It
//! is also the cheapest file in msvcrt to grade, which is why it went first among what was
//! owed: the demangler takes its ALLOCATOR as parameters (`memget`/`memfree`), so there is
//! no heap model to build, and its entire undefined set — `_assert`, `_vsnprintf`, `atoi`,
//! `lstrcpynA`, `memmove`, `ros_dbg_log`, `strcpy`, `strlen`, `wine_dbgstr_an` — was already
//! modelled or already a reference TU before this file existed.
//!
//! THE CORPUS IS THE TREE'S OWN. Every `?`-prefixed export in every `.spec` under the repo
//! is a genuine MSVC-mangled name that some real DLL publishes: constructors, destructors,
//! operators, vtables, templates, and all four calling conventions across three arches.
//! build.rs scrapes them into `$OUT_DIR/undname_corpus.rs`, so the corpus grows with the
//! tree instead of being a transcription that rots. A hand-typed list would also be the
//! §9.73 trap in its usual shape — the cases we thought of, grading the code we wrote.
//!
//! WHAT EACH ROW COMPARES. Not just the demangled text:
//!
//!   * the RETURNED POINTER's identity. With a buffer, both sides must return that buffer;
//!     without one, both must return freshly allocated memory. A side that quietly returns
//!     its internal scratch instead would produce identical bytes and a dangling pointer.
//!   * the ALLOCATION LEDGER. `memget`/`memfree` are ours, one counting pair per side, so
//!     "did `und_free_all` release everything the parse allocated" is directly observable —
//!     and it is the half of this file a text comparison cannot reach at all.
//!   * the TRUNCATION path, swept over buffer sizes. `lstrcpynA` copies at most n-1 bytes
//!     and always terminates, so the interesting sizes are the ones just below the answer.
//!   * a FAILED parse. `symbol_demangle` returning 0 makes the result the mangled input
//!     itself, not NULL — a fallback easy to translate into a null return that no
//!     well-formed name would ever expose.

#![allow(non_snake_case)]

extern crate msvcrt_crt_rust;

use core::ffi::{c_char, c_int, c_void};

include!(concat!(env!("OUT_DIR"), "/undname_corpus.rs"));

/// `dbghelp.h:1251-1267`. Swept rather than passed as 0, because `UNDNAME_NAME_ONLY` is
/// expanded by the entry point itself into five more bits before any parsing happens, and
/// `UNDNAME_32_BIT_DECODE` changes how calling conventions print.
const FLAGS: &[u16] = &[
    0x0000, // UNDNAME_COMPLETE
    0x0001, // NO_LEADING_UNDERSCORES
    0x0002, // NO_MS_KEYWORDS
    0x0004, // NO_FUNCTION_RETURNS
    0x0060, // NO_THISTYPE (both CV and MS bits)
    0x0080, // NO_ACCESS_SPECIFIERS
    0x0200, // NO_MEMBER_TYPE
    0x0800, // 32_BIT_DECODE
    0x1000, // NAME_ONLY — expands into five more inside __unDNameEx
    0x2000, // NO_ARGUMENTS
    0x4000, // NO_SPECIAL_SYMS
    0x1800, // NAME_ONLY | 32_BIT_DECODE
];

type MallocFn = unsafe extern "C" fn(usize) -> *mut c_void;
type FreeFn = unsafe extern "C" fn(*mut c_void);

extern "C" {
    fn __unDName(
        buffer: *mut c_char,
        mangled: *const c_char,
        buflen: c_int,
        memget: MallocFn,
        memfree: FreeFn,
        flags: u16,
    ) -> *mut c_char;
    fn ref___unDName(
        buffer: *mut c_char,
        mangled: *const c_char,
        buflen: c_int,
        memget: MallocFn,
        memfree: FreeFn,
        flags: u16,
    ) -> *mut c_char;
    fn __unDNameEx(
        buffer: *mut c_char,
        mangled: *const c_char,
        buflen: c_int,
        memget: MallocFn,
        memfree: FreeFn,
        unknown: *mut c_void,
        flags: u16,
    ) -> *mut c_char;
    fn ref___unDNameEx(
        buffer: *mut c_char,
        mangled: *const c_char,
        buflen: c_int,
        memget: MallocFn,
        memfree: FreeFn,
        unknown: *mut c_void,
        flags: u16,
    ) -> *mut c_char;
}

/// One counting allocator per side. Not shared: the ledger is the only way to see
/// `und_free_all`, and a shared counter would net one side's leak against the other's
/// double-free and read as balanced.
mod ledger {
    use core::ffi::c_void;
    use std::alloc::{alloc, dealloc, Layout};

    pub struct Side {
        pub gets: usize,
        pub frees: usize,
        pub live: Vec<(*mut c_void, usize)>,
    }
    pub static mut SUBJ: Side = Side { gets: 0, frees: 0, live: Vec::new() };
    pub static mut REF: Side = Side { gets: 0, frees: 0, live: Vec::new() };

    unsafe fn get(side: *mut Side, n: usize) -> *mut c_void {
        // `n` can be 0 for a degenerate parse; the C allocator would return a unique
        // pointer, so round up rather than hand back a null the caller would read as OOM.
        let n = n.max(1);
        let p = alloc(Layout::from_size_align(n, 16).unwrap()) as *mut c_void;
        (*side).gets += 1;
        (*side).live.push((p, n));
        p
    }
    unsafe fn put(side: *mut Side, p: *mut c_void) {
        if p.is_null() {
            return;
        }
        let at = (*side).live.iter().position(|&(q, _)| q == p);
        let Some(at) = at else {
            panic!("freed a pointer this side never allocated: {p:?}");
        };
        let (_, n) = (*side).live.remove(at);
        (*side).frees += 1;
        dealloc(p as *mut u8, Layout::from_size_align(n, 16).unwrap());
    }

    pub unsafe extern "C" fn subj_get(n: usize) -> *mut c_void {
        get(core::ptr::addr_of_mut!(SUBJ), n)
    }
    pub unsafe extern "C" fn subj_put(p: *mut c_void) {
        put(core::ptr::addr_of_mut!(SUBJ), p)
    }
    pub unsafe extern "C" fn ref_get(n: usize) -> *mut c_void {
        get(core::ptr::addr_of_mut!(REF), n)
    }
    pub unsafe extern "C" fn ref_put(p: *mut c_void) {
        put(core::ptr::addr_of_mut!(REF), p)
    }

    /// Blocks still outstanding on one side. Through a fn so the caller never forms a `&`
    /// to the `static mut` (the `static_mut_refs` lint), which is otherwise easy to do by
    /// accident inside an `assert_eq!`.
    pub fn live(side: usize) -> usize {
        unsafe {
            let s = if side == 0 { core::ptr::addr_of!(SUBJ) } else { core::ptr::addr_of!(REF) };
            (*s).live.len()
        }
    }
}

/// The C returns `buffer` when one was given and a `memget` block otherwise, and in BOTH
/// cases the string is NUL-terminated. Read it as bytes rather than as a `CStr` so a
/// missing terminator is a hang the test can localise, not UB.
unsafe fn text(p: *const c_char, cap: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for i in 0..cap {
        let b = *p.add(i) as u8;
        if b == 0 {
            return out;
        }
        out.push(b);
    }
    panic!("no NUL within {cap} bytes of {p:?}");
}

fn main() {
    eprintln!("\nundname differential — {} names x {} flag sets\n", CORPUS.len(), FLAGS.len());
    let mut rows = 0usize;

    for &mangled in CORPUS {
        let c_mangled = std::ffi::CString::new(mangled).unwrap();
        for &flags in FLAGS {
            unsafe {
                // ── no buffer: the allocate-and-return path ─────────────────────────
                let s = __unDName(
                    core::ptr::null_mut(),
                    c_mangled.as_ptr(),
                    0,
                    ledger::subj_get,
                    ledger::subj_put,
                    flags,
                );
                let r = ref___unDName(
                    core::ptr::null_mut(),
                    c_mangled.as_ptr(),
                    0,
                    ledger::ref_get,
                    ledger::ref_put,
                    flags,
                );
                assert_eq!(
                    s.is_null(),
                    r.is_null(),
                    "{mangled} @ {flags:#06x}: one side returned NULL and the other did not"
                );
                if !s.is_null() {
                    let (st, rt) = (text(s, 1 << 16), text(r, 1 << 16));
                    assert_eq!(
                        String::from_utf8_lossy(&st),
                        String::from_utf8_lossy(&rt),
                        "{mangled} @ {flags:#06x}: demangled differently"
                    );
                    // The caller owns this block; the ledger below would otherwise read
                    // every no-buffer row as a leak.
                    ledger::subj_put(s as *mut c_void);
                    ledger::ref_put(r as *mut c_void);
                }
                // Everything the parse itself allocated must be gone: `und_free_all` is
                // the only thing that releases it, and no text comparison can see it.
                let (subj_live, ref_live) = (ledger::live(0), ledger::live(1));
                assert_eq!(
                    (subj_live, ref_live),
                    (0, 0),
                    "{mangled} @ {flags:#06x}: undname leaked \
                     (subject {subj_live} block(s), reference {ref_live})"
                );
                rows += 1;

                // ── with a buffer: the truncation path, through __unDNameEx ─────────
                //
                // `lstrcpynA` copies at most n-1 bytes and always terminates, so the
                // interesting sizes are the ones just below the answer — 1 (terminator
                // only) and the two either side of the exact fit. A row that only ever
                // passes a generous buffer never enters this path at all, which is the
                // vacuous-arm trap the field manual records for the `_s` printf shells.
                let want = text(r, 1 << 16);
                for &n in &[1usize, 2, want.len(), want.len() + 1, want.len() + 8] {
                    let mut sb = vec![0xAAu8; n + 64];
                    let mut rb = vec![0xAAu8; n + 64];
                    let sp = __unDNameEx(
                        sb.as_mut_ptr() as *mut c_char,
                        c_mangled.as_ptr(),
                        n as c_int,
                        ledger::subj_get,
                        ledger::subj_put,
                        core::ptr::null_mut(),
                        flags,
                    );
                    let rp = ref___unDNameEx(
                        rb.as_mut_ptr() as *mut c_char,
                        c_mangled.as_ptr(),
                        n as c_int,
                        ledger::ref_get,
                        ledger::ref_put,
                        core::ptr::null_mut(),
                        flags,
                    );
                    // WHICH pointer comes back is itself a branch: `if (buffer && buflen)`
                    // copies into the caller's, and otherwise the C ALLOCATES and returns
                    // that instead — reachable here whenever the demangled text is empty and
                    // the sweep hands over a zero length. Both sides must make the same
                    // choice, and identical bytes returned through a DIFFERENT pointer is a
                    // dangling return that only this check can see.
                    let (sown, rown) = (
                        sp as usize == sb.as_ptr() as usize,
                        rp as usize == rb.as_ptr() as usize,
                    );
                    assert_eq!(
                        sown, rown,
                        "{mangled} @ {flags:#06x} buflen {n}: one side wrote the caller's \
                         buffer and the other allocated"
                    );
                    if sown {
                        assert_eq!(
                            sb, rb,
                            "{mangled} @ {flags:#06x} buflen {n}: buffers differ (the 0xAA \
                             tail past the terminator is part of the comparison — writing \
                             past n is the failure this size sweep exists for)"
                        );
                    } else {
                        assert_eq!(
                            String::from_utf8_lossy(&text(sp, 1 << 16)),
                            String::from_utf8_lossy(&text(rp, 1 << 16)),
                            "{mangled} @ {flags:#06x} buflen {n}: allocated results differ"
                        );
                        assert!(
                            sb.iter().all(|&b| b == 0xAA) && rb.iter().all(|&b| b == 0xAA),
                            "{mangled} @ {flags:#06x} buflen {n}: a side took the allocate \
                             path and STILL wrote the caller's buffer"
                        );
                        ledger::subj_put(sp as *mut c_void);
                        ledger::ref_put(rp as *mut c_void);
                    }
                    assert_eq!(
                        (ledger::live(0), ledger::live(1)),
                        (0, 0),
                        "{mangled} @ {flags:#06x} buflen {n}: leaked"
                    );
                    rows += 1;
                }
            }
        }
    }

    eprintln!("\nundname: {rows} rows, allocation ledger balanced on both sides\n");
}
