//! `RtlIsNameInExpression` (Windows 7+): file-name wildcard matching with `*`, `?` and the
//! DOS wildcards `<` (DOS_STAR), `>` (DOS_QM) and `"` (DOS_DOT), per MS-FSA 2.1.4.4.
//!
//! Upstream added sdk/lib/rtl/name.c for this (d2b688a5ca8, compiled into rtl_vista). Our
//! rtl_vista is all Rust and the merge dropped name.c (no CMake list names it). It is NOT
//! translated here, because it is wrong against Windows. A differential run of a line-for-line
//! port of name.c against the real ntdll export (the ntdll-gated-oracle harness, ~300k cases)
//! disagreed on about 1% of them, in five ways, each confirmed on Windows 10.0.26300:
//!   1. IgnoreCase upcases the NAME only. The caller passes an expression that is already
//!      upcased, as with FsRtlIsNameInExpression: ("he*o", "hello", TRUE) is FALSE.
//!      name.c upcases both sides.
//!   2. `*.*` is matched literally: ("*.*", "a") is FALSE. Treating `*.*` as match-all is
//!      FindFirstFile's DOS convention, which the caller applies before it gets here.
//!      name.c short-circuits it to TRUE.
//!   3. `<` may consume the name's FINAL dot, and that ends it. A later `<` then matches
//!      the extension: ("FI<<<<<<<<", "FILE.TXT") is TRUE, ("F<", "FILE.TXT") is FALSE.
//!      name.c keeps only one backtrack point, so a run of `<` cannot do this.
//!   4. Past the end of the name nothing matches. name.c reads a past-the-end NUL and
//!      matches it against a NUL inside the name: ("?", "a\0") is TRUE there, FALSE on Windows.
//!   5. A zero-width wildcard that exhausts the expression lets the character under
//!      examination go unmatched: (">", ".") and ("A>", "a.") are TRUE (see `step`).
//!
//! So this is the MS-FSA semantics as a direct NFA over expression positions: the state set
//! is the set of expression offsets reachable after consuming name[..k]. Wildcards that can
//! match zero characters (`*`, `<`, `>` at a dot or at the end, `"` at the end) only ever
//! advance the offset by one, so the zero-width closure is a single forward pass. The state
//! set fits on the stack up to 1023 expression characters, and is heap-allocated beyond
//! that (UNICODE_STRING allows up to 32767).

use ntkrnl_sys::ddk::{RtlUpcaseUnicodeChar, PUNICODE_STRING};

const DOS_STAR: u16 = b'<' as u16;
const DOS_QM: u16 = b'>' as u16;
const DOS_DOT: u16 = b'"' as u16;
const STAR: u16 = b'*' as u16;
const QM: u16 = b'?' as u16;
const DOT: u16 = b'.' as u16;

const STATUS_NO_MEMORY: i32 = 0xC000_0017_u32 as i32;
const TAG_NAME_EXPRESSION: u32 = u32::from_le_bytes(*b"RtlN");
const STACK_WORDS: usize = 16; // 1024 states: expressions up to 1023 characters

extern "system" {
    fn RtlpAllocateMemory(Bytes: u32, Tag: u32) -> *mut core::ffi::c_void;
    fn RtlpFreeMemory(Mem: *mut core::ffi::c_void, Tag: u32);
    fn RtlRaiseStatus(Status: i32) -> !;
}

#[inline]
unsafe fn chars<'a>(s: PUNICODE_STRING) -> &'a [u16] {
    let len = ((*s).Length / 2) as usize;
    if len == 0 {
        &[]
    } else {
        core::slice::from_raw_parts((*s).Buffer as *const u16, len)
    }
}

#[inline]
fn test(set: &[u64], i: usize) -> bool {
    set[i / 64] & (1u64 << (i % 64)) != 0
}

#[inline]
fn set(set: &mut [u64], i: usize) {
    set[i / 64] |= 1u64 << (i % 64);
}

/// Add every offset reachable by zero-width matches, given the name character the next
/// step will see (`None` = end of name). Returns whether a zero-width match ran off the end
/// of the expression (see `step`).
fn closure(states: &mut [u64], expr: &[u16], c: Option<u16>) -> bool {
    let mut ran_off_end = false;
    for e in 0..expr.len() {
        if !test(states, e) {
            continue;
        }
        let zero_width = match expr[e] {
            STAR | DOS_STAR => true,
            DOS_QM => matches!(c, None | Some(DOT)),
            DOS_DOT => c.is_none(),
            _ => false,
        };
        if zero_width {
            set(states, e + 1);
            ran_off_end |= e + 1 == expr.len();
        }
    }
    ran_off_end
}

/// Consume name character `c` (at index `k`; `upcased` is what literals compare against).
///
/// `free_end`: a zero-width wildcard reached the END of the expression while this character
/// was under examination. Windows then lets the character go unmatched, so the end state
/// carries to k+1 (which only matters when k is the last character): (">", ".") and
/// ("A>", "a.") are TRUE. This is the FsRtl backtracking quirk of recording "expression
/// exhausted" for the NEXT name position; a consumed path to the end gets no such pass
/// (("x", "xy") is FALSE).
fn step(from: &[u64], to: &mut [u64], expr: &[u16], k: usize, c: u16, upcased: u16, last_dot: Option<usize>, free_end: bool) {
    to.fill(0);
    if free_end {
        set(to, expr.len());
    }
    for e in 0..expr.len() {
        if !test(from, e) {
            continue;
        }
        match expr[e] {
            STAR => set(to, e),
            // `<` runs up to the final dot; it may swallow that dot, which ends it.
            DOS_STAR => set(to, if Some(k) == last_dot { e + 1 } else { e }),
            DOS_QM => {
                if c != DOT {
                    set(to, e + 1);
                }
            }
            DOS_DOT => {
                if c == DOT {
                    set(to, e + 1);
                }
            }
            QM => set(to, e + 1),
            literal => {
                if literal == upcased {
                    set(to, e + 1);
                }
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlIsNameInExpression(
    Expression: PUNICODE_STRING,
    Name: PUNICODE_STRING,
    IgnoreCase: u8,
    UpcaseTable: *mut u16,
) -> u8 {
    let expr = chars(Expression);
    let name = chars(Name);

    if expr.is_empty() || name.is_empty() {
        return (expr.is_empty() && name.is_empty()) as u8;
    }
    if expr.len() == 1 && expr[0] == STAR {
        return 1;
    }

    let words = (expr.len() + 1).div_ceil(64);
    let mut stack = [0u64; 2 * STACK_WORDS];
    let mut heap: *mut u64 = core::ptr::null_mut();
    let buf: &mut [u64] = if words <= STACK_WORDS {
        &mut stack[..2 * words]
    } else {
        let bytes = (2 * words * core::mem::size_of::<u64>()) as u32;
        heap = RtlpAllocateMemory(bytes, TAG_NAME_EXPRESSION) as *mut u64;
        if heap.is_null() {
            RtlRaiseStatus(STATUS_NO_MEMORY);
        }
        core::ptr::write_bytes(heap, 0, 2 * words);
        core::slice::from_raw_parts_mut(heap, 2 * words)
    };
    let (mut cur, mut next) = buf.split_at_mut(words);

    let last_dot = name.iter().rposition(|&c| c == DOT);
    set(cur, 0);
    let mut alive = true;
    for (k, &c) in name.iter().enumerate() {
        let upcased = if IgnoreCase == 0 {
            c
        } else if !UpcaseTable.is_null() {
            *UpcaseTable.add(c as usize)
        } else {
            RtlUpcaseUnicodeChar(c)
        };
        let free_end = closure(cur, expr, Some(c));
        step(cur, next, expr, k, c, upcased, last_dot, free_end);
        core::mem::swap(&mut cur, &mut next);
        if cur.iter().all(|&w| w == 0) {
            alive = false;
            break;
        }
    }
    let matched = if alive {
        closure(cur, expr, None);
        test(cur, expr.len())
    } else {
        false
    };

    if !heap.is_null() {
        RtlpFreeMemory(heap as *mut core::ffi::c_void, TAG_NAME_EXPRESSION);
    }
    matched as u8
}
