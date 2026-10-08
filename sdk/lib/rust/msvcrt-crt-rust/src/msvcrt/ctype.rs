//! `ctype.c`'s three DATA exports — the narrow and wide locale tables, and the pointer
//! into the wide one. This is the last of that file still in C.
//!
//! WHY THIS MODULE IS ONLY THE TABLES. ctype.c's 44 FUNCTIONS were translated by hand in
//! wave 9/10 and live in `lib.rs` beside the wide `isw*` family they share `_isctype_l`
//! with; they have been graded MATCH against reference TU #3 ever since. A rustconv pass
//! over this file on 2026-08-04 re-derived 23 of them independently and agreed with every
//! one — same masks, same `_isctype_l` delegation — which is a free confirmation, not a
//! replacement. That output is archived at
//! `tools/rustconv/.responses/20260804-164740-ctype.c/`; it is not assembled here, because
//! two `#[no_mangle]` definitions of `isalpha` in one crate is a link error and the
//! already-graded copy is the one to keep. Nothing is hidden from the build by that choice:
//! every one of those 44 symbols IS Rust today.
//!
//! What was NOT already Rust is here: the two tables, `MSVCRT__pwctype`, and `_ispunct_l`
//! (in lib.rs with its family — see the note there; a comment claimed msvcrt exports no
//! such symbol, and `llvm-nm` says otherwise).
//!
//! THE TABLES ARE GENERATED, NOT COPIED. 514 numbers with no compiler check is the worst
//! shape a translation bug can take: one wrong entry misclassifies exactly one character,
//! and no test that reads the same copy on both sides can see it. So the block below is
//! emitted by `tools/gen_ctype_tables.py`, which parses ctype.c's initializers, resolves
//! the `_MSVCR_VER`-gated arms for this DLL, evaluates every entry against the msvcrt-sys
//! ISLAND's mask values, and REFUSES to emit unless all 514 match the bytes the C compiler
//! put in `ctype.c.obj`'s `.data`. Two independent derivations agreeing is the proof.
//! Re-run it rather than editing between the markers.

use crate::prelude::*;

// <gen_ctype_tables BEGIN>
// ctype.c's own abbreviations, so the two tables below stay diffable against
// the C line by line. WORD-typed, because the island types the masks u32.
#[allow(non_upper_case_globals)]
const _C_: WORD = msvcrt_sys::_CONTROL as WORD;
const _S_: WORD = msvcrt_sys::_SPACE as WORD;
const _P_: WORD = msvcrt_sys::_PUNCT as WORD;
const _D_: WORD = msvcrt_sys::_DIGIT as WORD;
const _H_: WORD = msvcrt_sys::_HEX as WORD;
const _U_: WORD = msvcrt_sys::_UPPER as WORD;
const _L_: WORD = msvcrt_sys::_LOWER as WORD;
const _BLANK: WORD = msvcrt_sys::_BLANK as WORD;

// The `_MSVCR_VER`-gated arms, resolved for THIS DLL (`-D_MSVCR_VER=0`).
#[allow(non_upper_case_globals)]
const B110: WORD = msvcrt_sys::_BLANK as WORD;  // _MSVCR_VER <= 110
const D120: WORD = 4;  // _MSVCR_VER != 120
const S140: WORD = 0;  // _MSVCR_VER < 140
const L140: WORD = 0;  // _MSVCR_VER < 140
const C140: WORD = 0;  // _MSVCR_VER < 140

#[no_mangle]
pub static mut MSVCRT__ctype: [WORD; 257] = [
    /* EOF + 0x00 */ 0, _C_, _C_, _C_, _C_, _C_, _C_, _C_,
    /* 0x07 */ _C_, _C_, _S_|_C_, _S_|_C_, _S_|_C_, _S_|_C_, _S_|_C_, _C_,
    /* 0x0f */ _C_, _C_, _C_, _C_, _C_, _C_, _C_, _C_,
    /* 0x17 */ _C_, _C_, _C_, _C_, _C_, _C_, _C_, _C_,
    /* 0x1f */ _C_, _S_|_BLANK, _P_, _P_, _P_, _P_, _P_, _P_,
    /* 0x27 */ _P_, _P_, _P_, _P_, _P_, _P_, _P_, _P_,
    /* 0x2f */ _P_, _D_|_H_, _D_|_H_, _D_|_H_, _D_|_H_, _D_|_H_, _D_|_H_, _D_|_H_,
    /* 0x37 */ _D_|_H_, _D_|_H_, _D_|_H_, _P_, _P_, _P_, _P_, _P_,
    /* 0x3f */ _P_, _P_, _U_|_H_, _U_|_H_, _U_|_H_, _U_|_H_, _U_|_H_, _U_|_H_,
    /* 0x47 */ _U_, _U_, _U_, _U_, _U_, _U_, _U_, _U_,
    /* 0x4f */ _U_, _U_, _U_, _U_, _U_, _U_, _U_, _U_,
    /* 0x57 */ _U_, _U_, _U_, _U_, _P_, _P_, _P_, _P_,
    /* 0x5f */ _P_, _P_, _L_|_H_, _L_|_H_, _L_|_H_, _L_|_H_, _L_|_H_, _L_|_H_,
    /* 0x67 */ _L_, _L_, _L_, _L_, _L_, _L_, _L_, _L_,
    /* 0x6f */ _L_, _L_, _L_, _L_, _L_, _L_, _L_, _L_,
    /* 0x77 */ _L_, _L_, _L_, _L_, _P_, _P_, _P_, _P_,
    /* 0x7f */ _C_, 0, 0, 0, 0, 0, 0, 0,
    /* 0x87 */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0x8f */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0x97 */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0x9f */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0xa7 */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0xaf */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0xb7 */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0xbf */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0xc7 */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0xcf */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0xd7 */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0xdf */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0xe7 */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0xef */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0xf7 */ 0, 0, 0, 0, 0, 0, 0, 0,
    /* 0xff */ 0
];

#[no_mangle]
pub static mut MSVCRT__wctype: [WORD; 257] = [
    /* EOF + 0x00 */ 0, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    /* 0x07 */ 0x0020, 0x0020, 0x0028 | B110, 0x0028, 0x0028, 0x0028, 0x0028, 0x0020,
    /* 0x0f */ 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    /* 0x17 */ 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    /* 0x1f */ 0x0020, 0x0048, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010,
    /* 0x27 */ 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010,
    /* 0x2f */ 0x0010, 0x0084, 0x0084, 0x0084, 0x0084, 0x0084, 0x0084, 0x0084,
    /* 0x37 */ 0x0084, 0x0084, 0x0084, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010,
    /* 0x3f */ 0x0010, 0x0010, 0x0181, 0x0181, 0x0181, 0x0181, 0x0181, 0x0181,
    /* 0x47 */ 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101,
    /* 0x4f */ 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101,
    /* 0x57 */ 0x0101, 0x0101, 0x0101, 0x0101, 0x0010, 0x0010, 0x0010, 0x0010,
    /* 0x5f */ 0x0010, 0x0010, 0x0182, 0x0182, 0x0182, 0x0182, 0x0182, 0x0182,
    /* 0x67 */ 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102,
    /* 0x6f */ 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102,
    /* 0x77 */ 0x0102, 0x0102, 0x0102, 0x0102, 0x0010, 0x0010, 0x0010, 0x0010,
    /* 0x7f */ 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020 | S140, 0x0020,
    /* 0x87 */ 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    /* 0x8f */ 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    /* 0x97 */ 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0020,
    /* 0x9f */ 0x0020, 0x0008 | B110, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010, 0x0010,
    /* 0xa7 */ 0x0010, 0x0010, 0x0010, 0x0010 | L140, 0x0010, 0x0010, 0x0010 | C140, 0x0010,
    /* 0xaf */ 0x0010, 0x0010, 0x0010, 0x0010 | D120, 0x0010 | D120, 0x0010, 0x0010 | L140, 0x0010,
    /* 0xb7 */ 0x0010, 0x0010, 0x0010 | D120, 0x0010 | L140, 0x0010, 0x0010, 0x0010, 0x0010,
    /* 0xbf */ 0x0010, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101,
    /* 0xc7 */ 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101,
    /* 0xcf */ 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101,
    /* 0xd7 */ 0x0010, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101, 0x0101,
    /* 0xdf */ 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102,
    /* 0xe7 */ 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102,
    /* 0xef */ 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102,
    /* 0xf7 */ 0x0010, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102, 0x0102,
    /* 0xff */ 0x0102
];
// <gen_ctype_tables END>

// `WORD *MSVCRT__pwctype = MSVCRT__wctype + 1;` — the +1 is the same EOF-slot offset the
// narrow table has, and wcs.c indexes this pointer directly (`MSVCRT__pwctype[wc] & type`
// for `wc < 256`), so the element it points AT is load-bearing.
//
// `#[no_mangle]` on all three of these, and it is not decoration: the object's symbol table
// lists `MSVCRT__ctype`, `MSVCRT__wctype` and `MSVCRT__pwctype` as `D` (global data), and
// rustconv emitted all three as PRIVATE Rust statics. That is the same defect this crate
// hit on locale.c, data.c and dir.c — a private static links clean, so the C copies would
// simply have stayed live with nothing anywhere reporting it. Tracked as task #43.
#[no_mangle]
pub static mut MSVCRT__pwctype: *mut WORD = unsafe { (&raw mut MSVCRT__wctype).cast::<WORD>().add(1) };
