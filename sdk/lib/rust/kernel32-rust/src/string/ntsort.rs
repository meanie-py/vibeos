//! NT collation for CompareString over the shipped `sortkey.nls` + `sorttbls.nls`.
//!
//! The kernel32 C (and its first Rust port) compared through old Wine's
//! `collation_table`, which gives `í` exactly the weights of `i`: the table was
//! built to be used after a canonical decomposition that ReactOS commented out
//! (`decomposed_len = 1;/*wine_decompose(...)*/`). So every accent was invisible
//! to CompareString and lstrcmpi, and the locale argument was dropped on the
//! floor, so no language tailoring existed at all (Swedish Ä/Ö/Ø after Z, Czech
//! "ch", Danish "aa", French reverse diacritics).
//!
//! `media/nls` has shipped NT's own default table and tailoring file all along,
//! and nothing read them. This module does, the way NT's sort key is built:
//!
//! * `sortkey.nls`: a DWORD header then one 4-byte key per UTF-16 unit:
//!   `[AW alphanumeric weight, SM script member, DW diacritic, CW case]`.
//!   The primary weight is `(SM, AW)`. Special script members:
//!   0 unsortable (ignored), 1 nonspace mark (its DW is ADDED to the previous
//!   character's, so `e` + U+0301 == `é`), 2 expansion (AW indexes the
//!   expansion table: `ß` -> "ss", `Æ` -> "AE"), 6 punctuation (word sort keeps
//!   it out of the primary level and compares it last), 7..11 symbols.
//!   CW bits: 0x01 width, 0x08|0x10 case, 0x20 hiragana.
//! * `sorttbls.nls`: reverse-diacritic locales, double-compression locales,
//!   CJK ideograph files, the expansion pairs, per-locale compressions (2- and
//!   3-unit sequences that sort as one letter) and per-locale exceptions
//!   (single units re-weighted). Layout notes are on [`parse_sorttbls`].
//!
//! Comparison is level by level, exactly as comparing two NT sort keys:
//! primary, then diacritic (back to front for reverse-diacritic locales), then
//! case, then the word-sort punctuation weights.
//!
//! Not modelled (owed in the upstream-sync plan): Hungarian double
//! compression ("ccs" = "cs"+"cs"), the CJK alternate-ideograph files, and the
//! East Asian / Jamo / Extension-A special script members, which are compared
//! by their plain (SM, AW) weight.
//!
//! If the tables cannot be loaded the caller falls back to the old comparator
//! and an ERR is logged once: CompareString has no failure mode that callers
//! survive (it sorts every list in the shell), so failing the call would be
//! strictly worse than the old, accent-blind order.

use core::sync::atomic::{AtomicPtr, AtomicU32, Ordering};
use ntdll_sys::nt::*;
use winedebug_rust::ERR;

use crate::string::nls_subst::{
    LINGUISTIC_IGNORECASE, LINGUISTIC_IGNOREDIACRITIC, NORM_IGNORECASE, NORM_IGNOREKANATYPE, NORM_IGNORENONSPACE, NORM_IGNORESYMBOLS, NORM_IGNOREWIDTH,
    SORT_STRINGSORT,
};

use crate::client::handle::CloseHandle;
use crate::client::path::GetSystemDirectoryW;
use crate::file::create::CreateFileW;
use crate::file::fileinfo::GetFileSize;
use crate::file::filemap::CreateFileMappingW;

extern "system" {
    fn GetLastError() -> DWORD;
    fn SetLastError(dwErrCode: DWORD);
}

const SM_UNSORTABLE: u8 = 0;
const SM_NONSPACE_MARK: u8 = 1;
const SM_EXPANSION: u8 = 2;
const SM_PUNCTUATION: u8 = 6;
const SM_LAST_SYMBOL: u8 = 11;

const CW_WIDTH: u8 = 0x01;
const CW_CASE: u8 = 0x18;
const CW_HIRAGANA: u8 = 0x20;

const INVALID_HANDLE_VALUE: HANDLE = !0usize as HANDLE; // (HANDLE)-1
const SORTKEY_SIZE: u32 = 4 + 0x10000 * 4;

/// One locale's slice of a tailoring section: `off` is in WORDs from the
/// section's data base.
#[derive(Clone, Copy)]
struct LocaleSlice {
    lcid: u32,
    off: u32,
    n2: u32,
    n3: u32,
}

const MAX_LOCALES: usize = 64;
const MAX_REVERSE: usize = 16;

struct Tables {
    keys: *const u8,  // sortkey.nls past its DWORD header
    tbls: *const u8,  // sorttbls.nls
    tbls_len: usize,
    expansion: usize, // byte offset of the (u16, u16) pairs
    n_expansion: u32,
    reverse: [u32; MAX_REVERSE],
    n_reverse: usize,
    comp_base: usize,
    comp: [LocaleSlice; MAX_LOCALES],
    n_comp: usize,
    exc_base: usize,
    exc: [LocaleSlice; MAX_LOCALES],
    n_exc: usize,
}

static TABLES: AtomicPtr<Tables> = AtomicPtr::new(core::ptr::null_mut());
static mut TABLES_STORE: core::mem::MaybeUninit<Tables> = core::mem::MaybeUninit::uninit();
/// 0 never tried, 1 a thread is loading, 2 ready, 3 failed for good.
static STATE: AtomicU32 = AtomicU32::new(0);

#[inline]
unsafe fn rd16(p: *const u8, off: usize) -> u16 {
    core::ptr::read_unaligned(p.add(off) as *const u16)
}
#[inline]
unsafe fn rd32(p: *const u8, off: usize) -> u32 {
    core::ptr::read_unaligned(p.add(off) as *const u32)
}

/// Map `%SystemRoot%\system32\<name>` read-only; returns (base, size).
unsafe fn map_system_file(name: &[u16]) -> Option<(*const u8, u32)> {
    let mut path = [0u16; MAX_PATH as usize + 32];
    let n = GetSystemDirectoryW(path.as_mut_ptr(), MAX_PATH as UINT) as usize;
    if n == 0 || n + 1 + name.len() >= path.len() {
        return None;
    }
    path[n] = b'\\' as u16;
    path[n + 1..n + 1 + name.len()].copy_from_slice(name); // name carries its NUL
    let file = CreateFileW(
        path.as_ptr(),
        GENERIC_READ as DWORD,
        FILE_SHARE_READ as DWORD,
        core::ptr::null_mut(),
        OPEN_EXISTING as DWORD,
        0,
        core::ptr::null_mut(),
    );
    if file == INVALID_HANDLE_VALUE {
        return None;
    }
    let size = GetFileSize(file, core::ptr::null_mut());
    let mapping = CreateFileMappingW(file, core::ptr::null_mut(), PAGE_READONLY as DWORD, 0, 0, core::ptr::null());
    CloseHandle(file);
    if mapping.is_null() {
        return None;
    }
    // TOP-DOWN, like NT's own NLS views: these live for the life of the
    // process, and a bottom-up view lands in the low hole an app may want
    // (kmtest's ZwAllocateVirtualMemory ZeroBitsTest reserves at 0x500000 and
    // lost two checks on amd64 when this mapped bottom-up).
    let mut view: PVOID = core::ptr::null_mut();
    let mut view_size: SIZE_T = 0;
    let status = NtMapViewOfSection(
        mapping,
        NtCurrentProcess(),
        &mut view,
        0,
        0,
        core::ptr::null_mut(),
        &mut view_size,
        ViewShare,
        MEM_TOP_DOWN as ULONG,
        PAGE_READONLY as ULONG,
    );
    CloseHandle(mapping);
    if status < 0 || view.is_null() || size == u32::MAX {
        return None;
    }
    Some((view as *const u8, size))
}

/// Read a `count, lcid[count]` list at `*o`.
unsafe fn read_lcid_list(p: *const u8, len: usize, o: &mut usize, out: &mut [u32]) -> Option<usize> {
    if *o + 4 > len {
        return None;
    }
    let n = rd32(p, *o) as usize;
    *o += 4;
    if n > out.len() || *o + n * 4 > len {
        return None;
    }
    for (i, slot) in out.iter_mut().enumerate().take(n) {
        *slot = rd32(p, *o + i * 4);
    }
    *o += n * 4;
    Some(n)
}

/// `sorttbls.nls`, in file order (every offset is checked against the size):
///
/// ```text
/// 6 DWORDs                         1,1,0,1,0xffffffff,1 (validated literally)
/// count, LCID[count]               reverse-diacritic locales (the French ones)
/// count, LCID[count]               double-compression locales (Hungarian)
/// count, {LCID, WCHAR file[14]}    CJK alternate-ideograph tables (32 bytes)
/// count, {WCHAR a, b}[count]       expansions, indexed by an SM 2 key's AW
/// count, {LCID, off, n2 | n3<<16}  compressions; data follows the index:
///                                    n2 x {c1, c2, KEY}          (8 bytes)
///                                    n3 x {c1, c2, c3, pad, KEY} (12 bytes)
/// count, {LCID, off, n}            exceptions; data follows: n x {c, KEY} (6)
/// ```
unsafe fn parse_sorttbls(t: &mut Tables) -> bool {
    let p = t.tbls;
    let len = t.tbls_len;
    if len < 24 {
        return false;
    }
    let preamble = [1u32, 1, 0, 1, 0xffff_ffff, 1];
    for (i, want) in preamble.iter().enumerate() {
        if rd32(p, i * 4) != *want {
            return false;
        }
    }
    let mut o = 24usize;
    match read_lcid_list(p, len, &mut o, &mut t.reverse) {
        Some(n) => t.n_reverse = n,
        None => return false,
    }
    let mut dbl = [0u32; MAX_REVERSE];
    if read_lcid_list(p, len, &mut o, &mut dbl).is_none() {
        return false;
    }
    if o + 4 > len {
        return false;
    }
    let n_ideo = rd32(p, o) as usize;
    o += 4 + n_ideo * 32;
    if o + 4 > len {
        return false;
    }
    t.n_expansion = rd32(p, o);
    t.expansion = o + 4;
    o = t.expansion + t.n_expansion as usize * 4;

    // compressions
    if o + 4 > len {
        return false;
    }
    let n = rd32(p, o) as usize;
    o += 4;
    if n > MAX_LOCALES || o + n * 12 > len {
        return false;
    }
    for i in 0..n {
        let e = o + i * 12;
        let counts = rd32(p, e + 8);
        t.comp[i] = LocaleSlice { lcid: rd32(p, e), off: rd32(p, e + 4), n2: counts & 0xffff, n3: counts >> 16 };
    }
    t.n_comp = n;
    t.comp_base = o + n * 12;
    let mut comp_end = t.comp_base;
    for s in &t.comp[..n] {
        let end = t.comp_base + (s.off as usize) * 2 + s.n2 as usize * 8 + s.n3 as usize * 12;
        comp_end = comp_end.max(end);
    }
    if comp_end > len {
        return false;
    }

    // exceptions
    o = comp_end;
    if o + 4 > len {
        return false;
    }
    let n = rd32(p, o) as usize;
    o += 4;
    if n > MAX_LOCALES || o + n * 12 > len {
        return false;
    }
    for i in 0..n {
        let e = o + i * 12;
        t.exc[i] = LocaleSlice { lcid: rd32(p, e), off: rd32(p, e + 4), n2: rd32(p, e + 8), n3: 0 };
    }
    t.n_exc = n;
    t.exc_base = o + n * 12;
    for s in &t.exc[..n] {
        if t.exc_base + (s.off as usize) * 2 + s.n2 as usize * 6 > len {
            return false;
        }
    }
    true
}

unsafe fn load() -> Option<*const Tables> {
    let sortkey: [u16; 12] = [
        b's' as u16, b'o' as u16, b'r' as u16, b't' as u16, b'k' as u16, b'e' as u16, b'y' as u16, b'.' as u16,
        b'n' as u16, b'l' as u16, b's' as u16, 0,
    ];
    let sorttbls: [u16; 13] = [
        b's' as u16, b'o' as u16, b'r' as u16, b't' as u16, b't' as u16, b'b' as u16, b'l' as u16, b's' as u16,
        b'.' as u16, b'n' as u16, b'l' as u16, b's' as u16, 0,
    ];
    let (keys, ksize) = map_system_file(&sortkey)?;
    if ksize != SORTKEY_SIZE {
        return None;
    }
    let (tbls, tsize) = map_system_file(&sorttbls)?;
    let t = (*core::ptr::addr_of_mut!(TABLES_STORE)).as_mut_ptr();
    core::ptr::write_bytes(t as *mut u8, 0, core::mem::size_of::<Tables>());
    (*t).keys = keys.add(4);
    (*t).tbls = tbls;
    (*t).tbls_len = tsize as usize;
    if !parse_sorttbls(&mut *t) {
        return None;
    }
    Some(t)
}

/// The loaded tables, or None while another thread (or a re-entrant call from
/// inside the load itself) is loading them, or for good once loading failed.
unsafe fn tables() -> Option<&'static Tables> {
    let p = TABLES.load(Ordering::Acquire);
    if !p.is_null() {
        return Some(&*p);
    }
    if STATE.compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire).is_err() {
        return None;
    }
    let saved = GetLastError();
    let loaded = load();
    SetLastError(saved);
    match loaded {
        Some(t) => {
            TABLES.store(t as *mut Tables, Ordering::Release);
            STATE.store(2, Ordering::Release);
            Some(&*t)
        }
        None => {
            STATE.store(3, Ordering::Release);
            ERR!("sortkey.nls/sorttbls.nls unavailable or malformed: CompareString falls back to the accent-blind Wine table\n");
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Key {
    aw: u8,
    sm: u8,
    dw: u8,
    cw: u8,
}

impl Key {
    #[inline]
    fn from(k: u32) -> Key {
        Key { aw: k as u8, sm: (k >> 8) as u8, dw: (k >> 16) as u8, cw: (k >> 24) as u8 }
    }
}

/// The tailoring that applies to one compare.
struct Locale<'a> {
    t: &'a Tables,
    reverse: bool,
    comp: Option<LocaleSlice>,
    exc: Option<LocaleSlice>,
}

impl<'a> Locale<'a> {
    fn new(t: &'a Tables, lcid: LCID) -> Locale<'a> {
        let id = lcid & 0x000f_ffff; // LANGID + sort id
        let find = |v: &[LocaleSlice]| v.iter().copied().find(|s| s.lcid == id);
        Locale {
            t,
            reverse: t.reverse[..t.n_reverse].iter().any(|&l| l == id),
            comp: find(&t.comp[..t.n_comp]),
            exc: find(&t.exc[..t.n_exc]),
        }
    }

    unsafe fn key(&self, c: u16) -> Key {
        if let Some(s) = self.exc {
            let base = self.t.exc_base + s.off as usize * 2;
            for i in 0..s.n2 as usize {
                let e = base + i * 6;
                if rd16(self.t.tbls, e) == c {
                    return Key::from(rd32(self.t.tbls, e + 2));
                }
            }
        }
        Key::from(rd32(self.t.keys, c as usize * 4))
    }

    /// A compression starting at `s[0]`: (key, units consumed).
    unsafe fn compression(&self, s: &[u16]) -> Option<(Key, usize)> {
        let sl = self.comp?;
        let p = self.t.tbls;
        let base = self.t.comp_base + sl.off as usize * 2;
        if s.len() >= 3 {
            let b3 = base + sl.n2 as usize * 8;
            for i in 0..sl.n3 as usize {
                let e = b3 + i * 12;
                if rd16(p, e) == s[0] && rd16(p, e + 2) == s[1] && rd16(p, e + 4) == s[2] {
                    return Some((Key::from(rd32(p, e + 8)), 3));
                }
            }
        }
        if s.len() >= 2 {
            for i in 0..sl.n2 as usize {
                let e = base + i * 8;
                if rd16(p, e) == s[0] && rd16(p, e + 2) == s[1] {
                    return Some((Key::from(rd32(p, e + 4)), 2));
                }
            }
        }
        None
    }
}

/// One element of the sort-key stream.
#[derive(Clone, Copy)]
enum Elem {
    /// A primary element, with its diacritic and (masked) case weight.
    Weight(u16, u8, u8),
    /// A word-sort punctuation unit: (primary position, weight).
    Special(u32, u16),
}

struct Walker<'a, 'b> {
    loc: &'b Locale<'a>,
    s: &'b [u16],
    i: usize,
    pending: Option<u16>,
    pos: u32,
    word_sort: bool,
    ignore_symbols: bool,
    cw_mask: u8,
    /// Fold following nonspace marks into each unit's DW. Only the diacritic
    /// level needs it; the others must not read ahead, because Windows stops
    /// at the first differing unit (Wine's locale test puts a guard page right
    /// after the difference, with a length that runs past it).
    fold_marks: bool,
}

impl<'a, 'b> Walker<'a, 'b> {
    fn new(loc: &'b Locale<'a>, s: &'b [u16], flags: DWORD, fold_marks: bool) -> Self {
        let mut cw_mask = 0xffu8;
        if flags & (NORM_IGNORECASE | LINGUISTIC_IGNORECASE) as DWORD != 0 {
            cw_mask &= !CW_CASE;
        }
        if flags & NORM_IGNOREWIDTH as DWORD != 0 {
            cw_mask &= !CW_WIDTH;
        }
        if flags & NORM_IGNOREKANATYPE as DWORD != 0 {
            cw_mask &= !CW_HIRAGANA;
        }
        Walker {
            loc,
            s,
            i: 0,
            pending: None,
            pos: 0,
            word_sort: flags & SORT_STRINGSORT as DWORD == 0,
            ignore_symbols: flags & NORM_IGNORESYMBOLS as DWORD != 0,
            cw_mask,
            fold_marks,
        }
    }

    unsafe fn next(&mut self) -> Option<Elem> {
        loop {
            let mut k = if let Some(c) = self.pending.take() {
                self.loc.key(c)
            } else if self.i < self.s.len() {
                if let Some((k, n)) = self.loc.compression(&self.s[self.i..]) {
                    self.i += n;
                    k
                } else {
                    let c = self.s[self.i];
                    self.i += 1;
                    self.loc.key(c)
                }
            } else {
                return None;
            };

            match k.sm {
                // A mark with nothing before it, or an unsortable unit.
                SM_UNSORTABLE | SM_NONSPACE_MARK => continue,
                SM_EXPANSION => {
                    let t = self.loc.t;
                    if (k.aw as u32) >= t.n_expansion {
                        continue;
                    }
                    let e = t.expansion + k.aw as usize * 4;
                    self.pending = Some(rd16(t.tbls, e + 2));
                    k = self.loc.key(rd16(t.tbls, e));
                    if k.sm == SM_EXPANSION || k.sm == SM_UNSORTABLE {
                        continue; // malformed table: never recurse
                    }
                }
                _ => {}
            }

            if (SM_PUNCTUATION..=SM_LAST_SYMBOL).contains(&k.sm) && self.ignore_symbols {
                continue;
            }
            if k.sm == SM_PUNCTUATION && self.word_sort {
                return Some(Elem::Special(self.pos, ((k.sm as u16) << 8) | k.aw as u16));
            }

            // Fold the nonspace marks that follow into this unit's diacritic.
            let mut dw = k.dw;
            if self.fold_marks && self.pending.is_none() {
                while self.i < self.s.len() {
                    let m = self.loc.key(self.s[self.i]);
                    if m.sm != SM_NONSPACE_MARK {
                        break;
                    }
                    dw = dw.wrapping_add(m.dw);
                    self.i += 1;
                }
            }
            self.pos += 1;
            return Some(Elem::Weight(((k.sm as u16) << 8) | k.aw as u16, dw, k.cw & self.cw_mask));
        }
    }

    unsafe fn next_weight(&mut self) -> Option<(u16, u8, u8)> {
        loop {
            match self.next()? {
                Elem::Weight(p, d, c) => return Some((p, d, c)),
                Elem::Special(..) => continue,
            }
        }
    }

    unsafe fn next_special(&mut self) -> Option<(u32, u16)> {
        loop {
            match self.next()? {
                Elem::Special(pos, w) => return Some((pos, w)),
                Elem::Weight(..) => continue,
            }
        }
    }
}

#[inline]
fn ord<T: Ord>(a: T, b: T) -> i32 {
    match a.cmp(&b) {
        core::cmp::Ordering::Less => -1,
        core::cmp::Ordering::Equal => 0,
        core::cmp::Ordering::Greater => 1,
    }
}

/// Compare one level; `pick` projects the (primary, dw, cw) triple.
unsafe fn level(
    loc: &Locale,
    s1: &[u16],
    s2: &[u16],
    flags: DWORD,
    pick: fn((u16, u8, u8)) -> u16,
    reverse: bool,
    fold_marks: bool,
) -> i32 {
    let mut a = Walker::new(loc, s1, flags, fold_marks);
    let mut b = Walker::new(loc, s2, flags, fold_marks);
    let mut last = 0;
    loop {
        match (a.next_weight(), b.next_weight()) {
            (None, None) => return last,
            (None, Some(_)) => return -1,
            (Some(_), None) => return 1,
            (Some(x), Some(y)) => {
                let r = ord(pick(x), pick(y));
                if r != 0 {
                    if !reverse {
                        return r;
                    }
                    last = r; // back-to-front: the LAST difference decides
                }
            }
        }
    }
}

/// NT-order compare of two counted strings. None when the tables are not
/// available (the caller then uses the legacy comparator).
pub unsafe fn nt_compare_string(lcid: LCID, flags: DWORD, s1: &[u16], s2: &[u16]) -> Option<i32> {
    let t = tables()?;
    let loc = Locale::new(t, lcid);

    let r = level(&loc, s1, s2, flags, |w| w.0, false, false);
    if r != 0 {
        return Some(r);
    }
    // LINGUISTIC_IGNOREDIACRITIC (Vista) ignores the same diacritic level.
    if flags & (NORM_IGNORENONSPACE | LINGUISTIC_IGNOREDIACRITIC) as DWORD == 0 {
        let r = level(&loc, s1, s2, flags, |w| w.1 as u16, loc.reverse, true);
        if r != 0 {
            return Some(r);
        }
    }
    let r = level(&loc, s1, s2, flags, |w| w.2 as u16, false, false);
    if r != 0 {
        return Some(r);
    }

    let mut a = Walker::new(&loc, s1, flags, false);
    let mut b = Walker::new(&loc, s2, flags, false);
    loop {
        match (a.next_special(), b.next_special()) {
            (None, None) => return Some(0),
            (None, Some(_)) => return Some(-1),
            (Some(_), None) => return Some(1),
            (Some(x), Some(y)) => {
                let r = ord(x, y);
                if r != 0 {
                    return Some(r);
                }
            }
        }
    }
}
