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
    /// Windows-10 tables from sortdefault.nls (`w10_*` below) instead of XP's
    /// sortkey.nls + sorttbls.nls. Same key array either way.
    w10: bool,
    w10_base: *const u8,
    w10_len: usize,
    w10_guids: usize,
    w10_n_guids: usize,
    w10_names: usize,
    w10_n_names: usize,
    keys: *const u8,  // the 64K key array (sortkey.nls past its DWORD header)
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

/// `sortdefault.nls`, generated by `tools/codegen sortdefault` from Microsoft's Windows 10
/// sorting-weight table; the layout is documented in
/// tools/vibeos-codegen/src/sortdefault.rs. Every offset and count is checked against the
/// file size here, so a short or foreign file is refused, not trusted.
const W10_MAGIC: u32 = u32::from_le_bytes(*b"SORT");
const W10_LAYOUT: u32 = 1;
const W10_GUID_RECORD: usize = 16 + 4 * 5 + 8 * 7;
const W10_NAME_RECORD: usize = 24;
const W10_FLAG_REVERSE_DIACRITICS: u32 = 0x1;

unsafe fn parse_sortdefault(t: &mut Tables, p: *const u8, len: usize) -> bool {
    if len < 17 * 4 || rd32(p, 0) != W10_MAGIC || rd32(p, 4) != W10_LAYOUT || rd32(p, 64) as usize != len {
        return false;
    }
    let field = |i: usize| rd32(p, i * 4) as usize;
    let (n_guids, guids, n_names, names) = (field(3), field(4), field(5), field(6));
    let (n_exp, exp, keys) = (field(7), field(8), field(15));
    let fits = |off: usize, n: usize, size: usize| n.checked_mul(size).and_then(|b| off.checked_add(b)).is_some_and(|e| e <= len);
    if !fits(keys, 0x10000, 4) || !fits(guids, n_guids, W10_GUID_RECORD) || !fits(names, n_names, W10_NAME_RECORD) || !fits(exp, n_exp, 8) {
        return false;
    }
    for g in 0..n_guids {
        let r = guids + g * W10_GUID_RECORD;
        if !fits(rd32(p, r + 20) as usize, rd32(p, r + 24) as usize, 8) || !fits(rd32(p, r + 28) as usize, rd32(p, r + 32) as usize, 8) {
            return false;
        }
        for k in 2..=8usize {
            let c = r + 36 + (k - 2) * 8;
            if !fits(rd32(p, c) as usize, rd32(p, c + 4) as usize, w10_comp_size(k)) {
                return false;
            }
        }
    }
    t.w10 = true;
    t.w10_base = p;
    t.w10_len = len;
    t.w10_guids = guids;
    t.w10_n_guids = n_guids;
    t.w10_names = names;
    t.w10_n_names = n_names;
    t.keys = p.add(keys);
    t.expansion = exp;
    t.n_expansion = n_exp as u32;
    true
}

/// Bytes per compression entry of `k` code points: k WORDs padded to a DWORD, then a key.
fn w10_comp_size(k: usize) -> usize {
    (2 * k + 3) / 4 * 4 + 4
}

unsafe fn load() -> Option<*const Tables> {
    let sortdefault: [u16; 16] = [
        b's' as u16, b'o' as u16, b'r' as u16, b't' as u16, b'd' as u16, b'e' as u16, b'f' as u16, b'a' as u16,
        b'u' as u16, b'l' as u16, b't' as u16, b'.' as u16, b'n' as u16, b'l' as u16, b's' as u16, 0,
    ];
    let t = (*core::ptr::addr_of_mut!(TABLES_STORE)).as_mut_ptr();
    core::ptr::write_bytes(t as *mut u8, 0, core::mem::size_of::<Tables>());
    if let Some((base, size)) = map_system_file(&sortdefault) {
        if parse_sortdefault(&mut *t, base, size as usize) {
            return Some(t);
        }
        ERR!("sortdefault.nls is malformed: using the XP sortkey.nls/sorttbls.nls tables\n");
        core::ptr::write_bytes(t as *mut u8, 0, core::mem::size_of::<Tables>());
    }
    load_xp(t)
}

unsafe fn load_xp(t: *mut Tables) -> Option<*const Tables> {
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
    /// Windows-10 tables: the sort GUID record of this locale, if it has one.
    w10_guid: Option<usize>,
}

extern "system" {
    fn LCIDToLocaleName(Locale: LCID, lpName: *mut u16, cchName: i32, dwFlags: DWORD) -> i32;
}

/// The sortdefault.nls GUID record for `lcid`: its locale name, else its parent's
/// ("fr-CA" -> "fr"), compared ASCII-case-insensitively against the name table.
unsafe fn w10_guid_for(t: &Tables, lcid: LCID) -> Option<usize> {
    let mut name = [0u16; 86];
    let n = LCIDToLocaleName(lcid, name.as_mut_ptr(), name.len() as i32, 0x0800_0000 /* LOCALE_ALLOW_NEUTRAL_NAMES */);
    if n <= 1 {
        return None;
    }
    let mut len = n as usize - 1;
    loop {
        for i in 0..t.w10_n_names {
            let rec = t.w10_names + i * W10_NAME_RECORD;
            let mut j = 0usize;
            while j < len && j < W10_NAME_RECORD - 2 {
                let c = *t.w10_base.add(rec + j) as u16;
                if c == 0 || (c | 0x20) != (name[j] | 0x20) {
                    break;
                }
                j += 1;
            }
            if j == len && (len == W10_NAME_RECORD - 2 || *t.w10_base.add(rec + len) == 0) {
                let g = rd16(t.w10_base, rec + W10_NAME_RECORD - 2) as usize;
                return (g < t.w10_n_guids).then_some(g);
            }
        }
        // Parent locale: drop the last '-' or '_' subtag.
        match name[..len].iter().rposition(|&c| c == b'-' as u16 || c == b'_' as u16) {
            Some(cut) if cut > 0 => len = cut,
            _ => return None,
        }
    }
}

impl<'a> Locale<'a> {
    unsafe fn new(t: &'a Tables, lcid: LCID) -> Locale<'a> {
        if t.w10 {
            let g = w10_guid_for(t, lcid);
            let reverse = g.is_some_and(|g| rd32(t.w10_base, t.w10_guids + g * W10_GUID_RECORD + 16) & W10_FLAG_REVERSE_DIACRITICS != 0);
            return Locale { t, reverse, comp: None, exc: None, w10_guid: g };
        }
        let id = lcid & 0x000f_ffff; // LANGID + sort id
        let find = |v: &[LocaleSlice]| v.iter().copied().find(|s| s.lcid == id);
        Locale {
            t,
            reverse: t.reverse[..t.n_reverse].iter().any(|&l| l == id),
            comp: find(&t.comp[..t.n_comp]),
            exc: find(&t.exc[..t.n_exc]),
            w10_guid: None,
        }
    }

    /// Windows-10 exception key for `c` (entries sorted by code point).
    unsafe fn w10_exception(&self, c: u16) -> Option<Key> {
        let g = self.t.w10_guids + self.w10_guid? * W10_GUID_RECORD;
        let (off, n) = (rd32(self.t.w10_base, g + 20) as usize, rd32(self.t.w10_base, g + 24) as usize);
        let (mut lo, mut hi) = (0usize, n);
        while lo < hi {
            let mid = (lo + hi) / 2;
            let cp = rd32(self.t.w10_base, off + mid * 8);
            if cp == c as u32 {
                return Some(Key::from(rd32(self.t.w10_base, off + mid * 8 + 4)));
            }
            if cp < c as u32 { lo = mid + 1 } else { hi = mid }
        }
        None
    }

    /// Windows-10 compression starting at `s[0]`, longest first (8 down to 2).
    unsafe fn w10_compression(&self, s: &[u16]) -> Option<(Key, usize)> {
        let g = self.t.w10_guids + self.w10_guid? * W10_GUID_RECORD;
        let p = self.t.w10_base;
        for k in (2..=s.len().min(8)).rev() {
            let c = g + 36 + (k - 2) * 8;
            let (off, n) = (rd32(p, c) as usize, rd32(p, c + 4) as usize);
            let size = w10_comp_size(k);
            let (mut lo, mut hi) = (0usize, n);
            while lo < hi {
                let mid = (lo + hi) / 2;
                let e = off + mid * size;
                let mut ord = core::cmp::Ordering::Equal;
                for i in 0..k {
                    ord = rd16(p, e + i * 2).cmp(&s[i]);
                    if ord != core::cmp::Ordering::Equal {
                        break;
                    }
                }
                match ord {
                    core::cmp::Ordering::Equal => return Some((Key::from(rd32(p, e + size - 4)), k)),
                    core::cmp::Ordering::Less => lo = mid + 1,
                    core::cmp::Ordering::Greater => hi = mid,
                }
            }
        }
        None
    }

    /// The two code points an expansion key stands for.
    unsafe fn expansion(&self, k: Key) -> Option<(u16, u16)> {
        let t = self.t;
        if t.w10 {
            // Windows-10 layout: the index is in the high 16 bits (730 entries > AW).
            let i = k.dw as usize | (k.cw as usize) << 8;
            if i >= t.n_expansion as usize {
                return None;
            }
            let e = t.expansion + i * 8;
            return Some((rd16(t.w10_base, e + 2), rd16(t.w10_base, e + 4)));
        }
        if (k.aw as u32) >= t.n_expansion {
            return None;
        }
        let e = t.expansion + k.aw as usize * 4;
        Some((rd16(t.tbls, e), rd16(t.tbls, e + 2)))
    }

    unsafe fn key(&self, c: u16) -> Key {
        if self.t.w10 {
            if let Some(k) = self.w10_exception(c) {
                return k;
            }
            return Key::from(rd32(self.t.keys, c as usize * 4));
        }
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
        if self.t.w10 {
            return self.w10_compression(s);
        }
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
                    let Some((first, second)) = self.loc.expansion(k) else {
                        continue;
                    };
                    self.pending = Some(second);
                    k = self.loc.key(first);
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

/// Writes a sort key, counting every byte but storing only the first `limit`; records
/// where each section separator ends so a short buffer can be cut at one.
struct KeyWriter {
    dst: *mut u8,
    limit: usize,
    n: usize,
    seps: [usize; 4],
    n_seps: usize,
}

impl KeyWriter {
    #[inline]
    unsafe fn put(&mut self, b: u8) {
        if self.n < self.limit {
            *self.dst.add(self.n) = b;
        }
        self.n += 1;
    }
    unsafe fn sep(&mut self) {
        self.put(SORTKEY_SEPARATOR);
        if self.n_seps < self.seps.len() {
            self.seps[self.n_seps] = self.n;
            self.n_seps += 1;
        }
    }
}

const SORTKEY_SEPARATOR: u8 = 0x01;
const SORTKEY_TERMINATOR: u8 = 0x00;
/// The default diacritic and case weight; trailing runs of it are left out of a key, so
/// "a" and "a" + nothing compare as equal prefixes.
const W_BASE: u8 = 2;

unsafe fn write_sort_key(loc: &Locale, flags: DWORD, s: &[u16], w: &mut KeyWriter) {
    // Primary: (SM, AW) per element, high byte first so memcmp is the numeric order.
    let mut count = 0usize;
    let mut last_dw = 0usize; // elements up to the last non-base diacritic
    let mut last_cw = 0usize;
    let mut it = Walker::new(loc, s, flags, true);
    while let Some((p, d, c)) = it.next_weight() {
        w.put((p >> 8) as u8);
        w.put(p as u8);
        count += 1;
        if d != W_BASE {
            last_dw = count;
        }
        if c != W_BASE {
            last_cw = count;
        }
    }
    w.sep();

    // Diacritic: one byte per element, trailing base weights dropped; back to front for
    // the reverse-diacritic (French) locales, so the LAST difference decides, as in
    // nt_compare_string. LINGUISTIC_IGNOREDIACRITIC/NORM_IGNORENONSPACE leave it empty.
    if flags & (NORM_IGNORENONSPACE | LINGUISTIC_IGNOREDIACRITIC) as DWORD == 0 {
        if loc.reverse {
            // Reversed, the trailing base run is the run at the START of the string.
            let mut first_dw = count;
            let mut it = Walker::new(loc, s, flags, true);
            let mut k = 0usize;
            while let Some((_, d, _)) = it.next_weight() {
                if d != W_BASE && k < first_dw {
                    first_dw = k;
                }
                k += 1;
            }
            let mut idx = count;
            while idx > first_dw {
                idx -= 1;
                let mut it = Walker::new(loc, s, flags, true);
                let mut k = 0usize;
                while let Some((_, d, _)) = it.next_weight() {
                    if k == idx {
                        w.put(d);
                        break;
                    }
                    k += 1;
                }
            }
        } else {
            let mut it = Walker::new(loc, s, flags, true);
            let mut k = 0usize;
            while let Some((_, d, _)) = it.next_weight() {
                if k >= last_dw {
                    break;
                }
                w.put(d);
                k += 1;
            }
        }
    }
    w.sep();

    // Case (already masked by the IGNORE* flags), trailing base weights dropped.
    let mut it = Walker::new(loc, s, flags, true);
    let mut k = 0usize;
    while let Some((_, _, c)) = it.next_weight() {
        if k >= last_cw {
            break;
        }
        w.put(c);
        k += 1;
    }
    w.sep();
    // The East Asian extra-weight level: not modelled (see the module notes), empty.
    w.sep();

    // Word-sort punctuation: position (big-endian, so earlier sorts first) then weight.
    let mut it = Walker::new(loc, s, flags, false);
    while let Some((pos, wt)) = it.next_special() {
        w.put((pos >> 8) as u8);
        w.put(pos as u8);
        w.put((wt >> 8) as u8);
        w.put(wt as u8);
    }
    w.put(SORTKEY_TERMINATOR);
}

/// LCMapString(LCMAP_SORTKEY) from the same weights CompareString uses, so comparing two
/// keys byte-wise orders the strings exactly as nt_compare_string does. `dst`/`dstlen` are
/// bytes; dstlen == 0 asks for the size. A buffer too small for the whole key gets the
/// sections that fit, cut just after a separator, and Err (ERROR_INSUFFICIENT_BUFFER), as
/// Windows fills it. None when the tables are not available.
pub unsafe fn nt_sort_key(lcid: LCID, flags: DWORD, s: &[u16], dst: *mut u8, dstlen: usize) -> Option<Result<usize, ()>> {
    let t = tables()?;
    let loc = Locale::new(t, lcid);
    let mut measure = KeyWriter { dst, limit: 0, n: 0, seps: [0; 4], n_seps: 0 };
    write_sort_key(&loc, flags, s, &mut measure);
    let size = measure.n;
    if dstlen == 0 {
        return Some(Ok(size));
    }
    let limit = if size <= dstlen {
        size
    } else {
        measure.seps[..measure.n_seps].iter().copied().filter(|&e| e <= dstlen).max().unwrap_or(0)
    };
    let mut out = KeyWriter { dst, limit, n: 0, seps: [0; 4], n_seps: 0 };
    write_sort_key(&loc, flags, s, &mut out);
    Some(if size <= dstlen { Ok(size) } else { Err(()) })
}

/// What LCMapString(NORM_IGNORENONSPACE) does with one UTF-16 unit.
pub enum Nonspace {
    Keep,
    Drop,
    Base(u16),
}

/// First script member that is a letter (12 is digits, below are symbols).
const SM_FIRST_LETTER: u8 = 13;
/// The diacritic weight of an unaccented character.
const DW_BASE: u8 = 2;

/// NORM_IGNORENONSPACE for LCMapString: a nonspacing mark is dropped and an
/// accented letter becomes its base letter.
///
/// Windows does this by canonical decomposition. We have no decomposition
/// tables (the C's `wine_decompose` call is commented out), but sortkey.nls
/// already encodes the same relation the way CompareString sees it: an
/// accented letter has its base letter's script member, alphabetic weight and
/// case weight, and a diacritic weight above 2 (`é` = (33,14,14,2), `e` =
/// (33,14,2,2)). So the base is the character with the same (SM, AW, CW) and
/// DW 2. This also strips letters NFD leaves alone because their mark is not
/// a combining sequence (`ø`, `ı`), which is what CompareString's
/// NORM_IGNORENONSPACE does with them too.
pub unsafe fn nonspace(c: u16) -> Nonspace {
    let Some(t) = tables() else { return Nonspace::Keep };
    let k = Key::from(rd32(t.keys, c as usize * 4));
    if k.sm == SM_NONSPACE_MARK {
        return Nonspace::Drop;
    }
    if k.sm < SM_FIRST_LETTER || k.dw <= DW_BASE {
        return Nonspace::Keep;
    }
    // Bases are almost all below their accented forms (Latin ones in ASCII),
    // so a forward scan stops early; the full scan only runs for a letter with
    // no unaccented twin, and then the letter is kept as it is.
    for b in 0..=0xffffu32 {
        let bk = Key::from(rd32(t.keys, b as usize * 4));
        if bk.dw == DW_BASE && bk.sm == k.sm && bk.aw == k.aw && bk.cw == k.cw {
            return Nonspace::Base(b as u16);
        }
    }
    Nonspace::Keep
}
