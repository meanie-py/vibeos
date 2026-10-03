//! The multibyte codepage state -- `_setmbcp`, `_getmbcp`, the exported
//! `_mbctype`/`_mbcasemap` tables and the per-thread `__crt_multibyte_data`
//! refcounting under them. Replaces `sdk/lib/ucrt/mbstring/mbctype.cpp` in full.
//!
//! A PORT of Microsoft's, not msvcrt's: msvcrt-crt-rust's `_setmbcp` was graded
//! against UCRT's and differs (Wine's contract), so nothing is reused from it.
//!
//! State isolation is compiled out in this tree (`state_index_count == 1`,
//! `sdk/lib/ucrt_reactos/inc/corecrt_internal_state_isolation.h`), so every
//! `dual_state_global<T>` is exactly ONE `T`: `_mbctype`, `_mbcasemap` and
//! `__acrt_current_multibyte_data` are plain pointers, and `_mbctypes` /
//! `_mbcasemaps` are one-row arrays. All three pointers start NULL (.bss) and
//! are set by `__acrt_initialize_multibyte`.
//!
//! `_malloc_crt`/`_free_crt` are `malloc`/`free` in this tree
//! (`sdk/lib/vcruntime/inc/internal_shared.h:176`), which is what the compiled
//! object imports, so that is what is called here.
//!
//! ## Details that are silent if wrong
//!
//! * **`mbctype` is 257 bytes indexed at `1 + byte`**; entry 0 is the EOF (-1)
//!   slot. Every write below is `mbctype[ich + 1]`.
//! * **UTF-8 clears only `mbctype[0..6]`.** The C's loop runs to `NUM_ULINFO`
//!   (6), not `NUM_CHARS` (257) -- an obvious typo -- so `_setmbcp(932)` then
//!   `_setmbcp(65001)` KEEPS 932's lead/trail/single-byte flags from byte 5 up,
//!   and `setSBUpLow` then ORs the ASCII case bits on top. Transcribed: this is
//!   what `_ismbblead` reports under Microsoft's UCRT after that sequence.
//! * **`fSystemSet` is reset by the SECOND `getSystemCP`.** `setmbcp_internal`
//!   resolves -2/-3/-4 to a real codepage, then `_setmbcp_nolock` calls
//!   `getSystemCP` again on that real number, which clears the flag. So the
//!   "system default never fails, fall back to SBCS" rescue only fires for a
//!   direct `_setmbcp_nolock(-2/-3/-4, ...)` (wsetlocale never passes one).
//! * **`(WORD)codepage` for `IsValidCodePage`**: 65536+1252 passes validity,
//!   then `GetCPInfo(66788)` fails and the call returns -1.
//! * **CP_UTF7 (65000) is refused; 0 is SBCS, not an error** (0 is caught
//!   before the validity test, which also tests 0 -- dead there).
//! * **An unchanged codepage returns 0 without allocating** and without
//!   touching `_mbctype`/`__acrt_locale_changed_data`; the comparison is against
//!   the RESOLVED codepage, so `_setmbcp(-3)` when already on the ACP is a no-op.
//! * **The global tables are only updated when the thread syncs with the global
//!   locale** (`(_own_locale & __globallocalestatus) == 0`); a per-thread-locale
//!   thread changes only its own `_multibyte_info`.
//! * **Refcount dance**: the new block starts at 0, the thread's old block is
//!   decremented (freed at 0 unless it is `__acrt_initial_multibyte_data`), the
//!   new block becomes 1, then +1 again when it also becomes the global current
//!   block (whose old value is decremented/freed the same way, under
//!   `__acrt_multibyte_cp_lock`).
//! * **`_getmbcp` returns 0 for UTF-8 and every SBCS** -- it reports
//!   `mbcodepage` only when `ismbcodepage` is set.
//! * **`setSBUpLow` casing** comes from `__acrt_GetStringTypeA(CT_CTYPE1)` +
//!   `__acrt_LCMapStringA` over a 256-byte vector whose byte 0 and every lead
//!   byte are replaced by ' ', for any codepage `GetCPInfo` knows except UTF-8;
//!   UTF-8 and unknown codepages get plain ASCII casing.
//! * **Only the ORs**: `setSBUpLow` never clears `mbctype`; each caller cleared
//!   it first (except the UTF-8 path, above).

#![allow(non_upper_case_globals)]

use crate::locale::{CrtLocalePointers, CrtMultibyteData};
use crate::locale_abi::{
    __acrt_GetStringTypeA, __acrt_LCMapStringA, __acrt_initial_locale_pointers,
    __acrt_set_locale_changed, __globallocalestatus,
};
use crate::locale_update::{__acrt_getptd, LocaleUpdate};
use crate::ptd::AcrtPtd;
use core::ffi::{c_int, c_long, c_uint, c_void};
use core::ptr::{addr_of, addr_of_mut, null, null_mut};
use core::sync::atomic::{AtomicI32, Ordering};

// mbctype.cpp carried `_CRT_LINKER_FORCE_INCLUDE(__acrt_multibyte_initializer)`; see force_include.rs.
crate::force_include!(__acrt_multibyte_initializer);

// ── constants (ucrt/mbctype.h, winnls.h, corecrt_wctype.h) ─────

/// mbctype.h:42-45, 49-50 -- the `_mbctype[]` bits.
const _MS: u8 = 0x01;
const _MP: u8 = 0x02;
const _M1: u8 = 0x04;
const _M2: u8 = 0x08;
const _SBUP: u8 = 0x10;
const _SBLOW: u8 = 0x20;
/// corecrt_wctype.h:48-49 -- CT_CTYPE1 bits.
const _UPPER: u16 = 0x01;
const _LOWER: u16 = 0x02;

/// mbctype.h:59, 63-66.
const _KANJI_CP: c_int = 932;
const _MB_CP_SBCS: c_int = 0;
const _MB_CP_OEM: c_int = -2;
const _MB_CP_ANSI: c_int = -3;
const _MB_CP_LOCALE: c_int = -4;

/// mbctype.cpp:22-25.
const _CHINESE_SIMP_CP: c_int = 936;
const _KOREAN_WANGSUNG_CP: c_int = 949;
const _CHINESE_TRAD_CP: c_int = 950;
const _KOREAN_JOHAB_CP: c_int = 1361;

/// mbctype.cpp:27-30, corecrt_internal_mbstring.h:21.
const NUM_CHARS: usize = 257;
const NUM_CTYPES: usize = 4;
const MAX_RANGES: usize = 8;
const NUM_ULINFO: usize = 6;

/// winnls.h:255, 263-265, 201-202.
const CP_ACP: c_int = 0;
const CP_UTF7: c_int = 65000;
const CP_UTF8: c_int = 65001;
const CT_CTYPE1: u32 = 1;
const LCMAP_LOWERCASE: u32 = 0x0000_0100;
const LCMAP_UPPERCASE: u32 = 0x0000_0200;

/// errno.h
const EINVAL: c_int = 22;

/// `__acrt_multibyte_cp_lock` -- enum `__acrt_lock_id`, corecrt_internal.h:936.
const ACRT_MULTIBYTE_CP_LOCK: c_int = 5;

/// `CPINFO` (winnls.h:660). 18 bytes of fields, 20 with tail padding.
#[repr(C)]
struct CpInfo {
    max_char_size: c_uint,
    default_char: [u8; 2],
    lead_byte: [u8; 12],
}
const _: () = assert!(core::mem::size_of::<CpInfo>() == 20);

extern "system" {
    fn GetACP() -> c_uint;
    fn GetOEMCP() -> c_uint;
    fn GetCPInfo(code_page: c_uint, cp_info: *mut CpInfo) -> c_int;
    fn IsValidCodePage(code_page: c_uint) -> c_int;
}

extern "C" {
    fn __acrt_getptd_head() -> *mut AcrtPtd;
    fn __acrt_lock(lock: c_int);
    fn __acrt_unlock(lock: c_int);
    fn malloc(size: usize) -> *mut c_void;
    fn free(block: *mut c_void);
    fn abort() -> !;
    fn _errno() -> *mut c_int;
    fn memcpy_s(dst: *mut c_void, dst_size: usize, src: *const c_void, count: usize) -> c_int;
}

// ── the static tables ─────────────────────────────────────────────────────────

/// `_MBCTYPE_DEFAULT`: ASCII upper -> `_SBUP`, lower -> `_SBLOW`, at `1 + byte`.
const MBCTYPE_DEFAULT: [u8; NUM_CHARS] = {
    let mut t = [0u8; NUM_CHARS];
    let mut c = b'A' as usize;
    while c <= b'Z' as usize {
        t[c + 1] = _SBUP;
        t[c + 1 + 0x20] = _SBLOW;
        c += 1;
    }
    t
};

/// `_MBCASEMAP_DEFAULT`: each ASCII letter maps to the other case, all else 0.
const MBCASEMAP_DEFAULT: [u8; 256] = {
    let mut t = [0u8; 256];
    let mut c = b'A';
    while c <= b'Z' {
        t[c as usize] = c + 0x20;
        t[c as usize + 0x20] = c;
        c += 1;
    }
    t
};

/// `__acrt_initial_multibyte_data` -- the "C" locale's block. Refcount 0,
/// `mbcodepage` CP_ACP, not multibyte.
#[no_mangle]
pub static mut __acrt_initial_multibyte_data: CrtMultibyteData = CrtMultibyteData {
    refcount: 0,
    mbcodepage: CP_ACP,
    ismbcodepage: 0,
    mbulinfo: [0; NUM_ULINFO],
    mbctype: MBCTYPE_DEFAULT,
    mbcasemap: MBCASEMAP_DEFAULT,
    mblocalename: null(),
};

/// `_mbctypes[state_index_count][NUM_CHARS]` / `_mbcasemaps[...][256]`: the
/// storage `_mbctype`/`_mbcasemap` point at.
static mut MBCTYPES: [[u8; NUM_CHARS]; 1] = [MBCTYPE_DEFAULT];
static mut MBCASEMAPS: [[u8; 256]; 1] = [MBCASEMAP_DEFAULT];

/// Exported: the global multibyte ctype table (a `dual_state_global<unsigned
/// char*>`, i.e. one pointer). NULL until `__acrt_initialize_multibyte`.
#[no_mangle]
pub static mut _mbctype: *mut u8 = null_mut();

/// Exported: the global multibyte case map.
#[no_mangle]
pub static mut _mbcasemap: *mut u8 = null_mut();

/// The current global multibyte block.
#[no_mangle]
pub static mut __acrt_current_multibyte_data: *mut CrtMultibyteData = null_mut();

/// `fSystemSet`: set by `getSystemCP` when it resolved -2/-3/-4.
static mut F_SYSTEM_SET: c_int = 0;

/// `__rgctypeflag` (`char[4]`), indexed by the row of `rgrange`.
static RGCTYPEFLAG: [u8; NUM_CTYPES] = [_MS, _MP, _M1, _M2];

/// `code_page_info`: 48 bytes on every arch.
#[repr(C)]
struct CodePageInfo {
    code_page: c_int,
    mbulinfo: [u16; NUM_ULINFO],
    rgrange: [[u8; MAX_RANGES]; NUM_CTYPES],
}
const _: () = assert!(core::mem::size_of::<CodePageInfo>() == 48);

/// `__rgcode_page_info`. `mbulinfo` = upper range 1 lo/hi, case difference 1,
/// upper range 2 lo/hi, case difference 2. `rgrange` rows = single-byte,
/// punctuation, lead-byte, trail-byte ranges as (low, high) pairs, 0-terminated.
static RGCODE_PAGE_INFO: [CodePageInfo; 5] = [
    CodePageInfo {
        code_page: _KANJI_CP,
        mbulinfo: [0x8260, 0x8279, 0x8281 - 0x8260, 0x0000, 0x0000, 0x0000],
        rgrange: [
            [0xA6, 0xDF, 0, 0, 0, 0, 0, 0],
            [0xA1, 0xA5, 0, 0, 0, 0, 0, 0],
            [0x81, 0x9F, 0xE0, 0xFC, 0, 0, 0, 0],
            [0x40, 0x7E, 0x80, 0xFC, 0, 0, 0, 0],
        ],
    },
    CodePageInfo {
        code_page: _CHINESE_SIMP_CP,
        mbulinfo: [0xA3C1, 0xA3DA, 0xA3E1 - 0xA3C1, 0x0000, 0x0000, 0x0000],
        rgrange: [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0x81, 0xFE, 0, 0, 0, 0, 0, 0],
            [0x40, 0xFE, 0, 0, 0, 0, 0, 0],
        ],
    },
    CodePageInfo {
        code_page: _KOREAN_WANGSUNG_CP,
        mbulinfo: [0xA3C1, 0xA3DA, 0xA3E1 - 0xA3C1, 0x0000, 0x0000, 0x0000],
        rgrange: [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0x81, 0xFE, 0, 0, 0, 0, 0, 0],
            [0x41, 0xFE, 0, 0, 0, 0, 0, 0],
        ],
    },
    CodePageInfo {
        code_page: _CHINESE_TRAD_CP,
        mbulinfo: [0xA2CF, 0xA2E4, 0xA2E9 - 0xA2CF, 0xA2E5, 0xA2E8, 0xA340 - 0xA2E5],
        rgrange: [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0x81, 0xFE, 0, 0, 0, 0, 0, 0],
            [0x40, 0x7E, 0xA1, 0xFE, 0, 0, 0, 0],
        ],
    },
    CodePageInfo {
        code_page: _KOREAN_JOHAB_CP,
        mbulinfo: [0xDA51, 0xDA5E, 0xDA71 - 0xDA51, 0xDA5F, 0xDA6A, 0xDA91 - 0xDA5F],
        rgrange: [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0x81, 0xD3, 0xD8, 0xDE, 0xE0, 0xF9, 0, 0],
            [0x31, 0x7E, 0x81, 0xFE, 0, 0, 0, 0],
        ],
    },
];

/// `_mb_locale_names[]`, as the four strings `CPtoLocaleName` hands out.
static MB_LOCALE_JA_JP: [u16; 6] = wide6(b"ja-JP");
static MB_LOCALE_ZH_CN: [u16; 6] = wide6(b"zh-CN");
static MB_LOCALE_KO_KR: [u16; 6] = wide6(b"ko-KR");
static MB_LOCALE_ZH_TW: [u16; 6] = wide6(b"zh-TW");

const fn wide6(s: &[u8; 5]) -> [u16; 6] {
    [s[0] as u16, s[1] as u16, s[2] as u16, s[3] as u16, s[4] as u16, 0]
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// `InterlockedDecrement(&p->refcount)`, returning the new value.
unsafe fn interlocked_decrement(refcount: *mut c_long) -> c_long {
    AtomicI32::from_ptr(refcount as *mut i32).fetch_sub(1, Ordering::SeqCst) - 1
}

/// `InterlockedIncrement(&p->refcount)`.
unsafe fn interlocked_increment(refcount: *mut c_long) {
    AtomicI32::from_ptr(refcount as *mut i32).fetch_add(1, Ordering::SeqCst);
}

/// `__acrt_should_sync_with_global_locale` (corecrt_internal.h:1862), an
/// `__inline` with no symbol.
unsafe fn should_sync_with_global_locale(ptd: *mut AcrtPtd) -> bool {
    ((*ptd).own_locale & __globallocalestatus) == 0
}

fn initial_multibyte_data() -> *mut CrtMultibyteData {
    addr_of_mut!(__acrt_initial_multibyte_data)
}

/// `setSBCS`: the single-byte "codepage" 0, with the initial block's tables.
unsafe fn set_sbcs(ptmbci: *mut CrtMultibyteData) {
    let mbctype = addr_of_mut!((*ptmbci).mbctype) as *mut u8;
    core::ptr::write_bytes(mbctype, 0, NUM_CHARS);

    (*ptmbci).mbcodepage = 0;
    (*ptmbci).ismbcodepage = 0;
    (*ptmbci).mblocalename = null();
    core::ptr::write_bytes(addr_of_mut!((*ptmbci).mbulinfo) as *mut u16, 0, NUM_ULINFO);

    // `ptr::copy`, not `copy_nonoverlapping`: the C's element loops are defined
    // even if ptmbci IS the initial block, so this must be too.
    let initial = initial_multibyte_data();
    core::ptr::copy(addr_of!((*initial).mbctype) as *const u8, mbctype, 257);
    core::ptr::copy(
        addr_of!((*initial).mbcasemap) as *const u8,
        addr_of_mut!((*ptmbci).mbcasemap) as *mut u8,
        256,
    );
}

/// `update_thread_multibyte_data_internal`.
unsafe fn update_thread_multibyte_data_internal(
    ptd: *mut AcrtPtd,
    current_multibyte_data: *mut *mut CrtMultibyteData,
) -> *mut CrtMultibyteData {
    let mut ptmbci: *mut CrtMultibyteData;

    if should_sync_with_global_locale(ptd) || (*ptd).locale_info.is_null() {
        __acrt_lock(ACRT_MULTIBYTE_CP_LOCK);
        ptmbci = (*ptd).multibyte_info;
        if ptmbci != *current_multibyte_data {
            // Release the old block, freeing it at zero unless it is the
            // static initial block.
            if !ptmbci.is_null()
                && interlocked_decrement(addr_of_mut!((*ptmbci).refcount)) == 0
                && ptmbci != initial_multibyte_data()
            {
                free(ptmbci as *mut c_void);
            }

            ptmbci = *current_multibyte_data;
            (*ptd).multibyte_info = ptmbci;
            interlocked_increment(addr_of_mut!((*ptmbci).refcount));
        }
        __acrt_unlock(ACRT_MULTIBYTE_CP_LOCK);
    } else {
        ptmbci = (*ptd).multibyte_info;
    }

    if ptmbci.is_null() {
        abort();
    }

    ptmbci
}

/// `__acrt_update_thread_multibyte_data`: resync this thread's block with the
/// global one and return it.
#[no_mangle]
pub unsafe extern "C" fn __acrt_update_thread_multibyte_data() -> *mut CrtMultibyteData {
    update_thread_multibyte_data_internal(__acrt_getptd(), addr_of_mut!(__acrt_current_multibyte_data))
}

/// `setmbcp_internal`.
unsafe fn setmbcp_internal(
    requested_codepage: c_int,
    is_for_crt_initialization: bool,
    ptd: *mut AcrtPtd,
    current_multibyte_data: *mut *mut CrtMultibyteData,
) -> c_int {
    update_thread_multibyte_data_internal(ptd, current_multibyte_data);
    let system_codepage = get_system_cp(requested_codepage);

    // If it's not a new codepage, just return success.
    if system_codepage == (*(*ptd).multibyte_info).mbcodepage {
        return 0;
    }

    // Always allocate, so no lock is needed for the update itself.
    let mb_data = malloc(core::mem::size_of::<CrtMultibyteData>()) as *mut CrtMultibyteData;
    if mb_data.is_null() {
        return -1;
    }

    // Start from the thread's current block, unreferenced.
    core::ptr::copy_nonoverlapping((*ptd).multibyte_info as *const CrtMultibyteData, mb_data, 1);
    (*mb_data).refcount = 0;

    let setmbcp_status = _setmbcp_nolock(system_codepage, mb_data);
    if setmbcp_status == -1 {
        *_errno() = EINVAL;
        // The C's __crt_unique_heap_ptr frees the block on this return.
        free(mb_data as *mut c_void);
        return -1;
    }

    if !is_for_crt_initialization {
        __acrt_set_locale_changed();
    }

    let old = (*ptd).multibyte_info;
    if interlocked_decrement(addr_of_mut!((*old).refcount)) == 0 && old != initial_multibyte_data() {
        free(old as *mut c_void);
    }

    // The thread now owns the new block.
    (*mb_data).refcount = 1;
    (*ptd).multibyte_info = mb_data;

    // A thread with its own locale does not touch the global codepage.
    if !should_sync_with_global_locale(ptd) {
        return setmbcp_status;
    }

    // Otherwise publish it globally (the C's __acrt_lock_and_call lambda; no
    // early exit inside, so the pair cannot be skipped).
    __acrt_lock(ACRT_MULTIBYTE_CP_LOCK);
    {
        let mbi = (*ptd).multibyte_info;
        memcpy_s(
            _mbctype as *mut c_void,
            NUM_CHARS,
            addr_of!((*mbi).mbctype) as *const c_void,
            NUM_CHARS,
        );
        memcpy_s(
            _mbcasemap as *mut c_void,
            256,
            addr_of!((*mbi).mbcasemap) as *const c_void,
            256,
        );

        let cur = *current_multibyte_data;
        if interlocked_decrement(addr_of_mut!((*cur).refcount)) == 0 && cur != initial_multibyte_data() {
            free(cur as *mut c_void);
        }

        *current_multibyte_data = (*ptd).multibyte_info;
        interlocked_increment(addr_of_mut!((*(*ptd).multibyte_info).refcount));
    }
    __acrt_unlock(ACRT_MULTIBYTE_CP_LOCK);

    if is_for_crt_initialization {
        (*addr_of_mut!(__acrt_initial_locale_pointers)).mbcinfo = *current_multibyte_data;
    }

    setmbcp_status
}

/// `CPtoLocaleName`: the locale name the NLS calls use for a DBCS codepage.
fn cp_to_locale_name(codepage: c_int) -> *const u16 {
    match codepage {
        932 => MB_LOCALE_JA_JP.as_ptr(),
        936 => MB_LOCALE_ZH_CN.as_ptr(),
        949 => MB_LOCALE_KO_KR.as_ptr(),
        950 => MB_LOCALE_ZH_TW.as_ptr(),
        _ => null(),
    }
}

/// `getSystemCP`: resolve the three special codepages. Constructs a
/// `_LocaleUpdate` on EVERY call (the C does, for its side effects on the
/// thread's locale pointers), and clears `fSystemSet` for an ordinary number.
unsafe fn get_system_cp(codepage: c_int) -> c_int {
    let loc_update = LocaleUpdate::new(null_mut::<CrtLocalePointers>());
    F_SYSTEM_SET = 0;

    if codepage == _MB_CP_OEM {
        F_SYSTEM_SET = 1;
        return GetOEMCP() as c_int;
    } else if codepage == _MB_CP_ANSI {
        F_SYSTEM_SET = 1;
        return GetACP() as c_int;
    } else if codepage == _MB_CP_LOCALE {
        F_SYSTEM_SET = 1;
        return (*loc_update.locinfo()).public.locale_lc_codepage as c_int;
    }

    codepage
}

/// `setSBUpLow`: the single-byte case bits and case map.
unsafe fn set_sb_up_low(ptmbci: *mut CrtMultibyteData) {
    // Zeroed where the C leaves them uninitialised: they are only read after
    // the NLS calls fill them, unless one of those calls FAILS -- then the C
    // reads stack garbage and this reads zeros (no case bits, map 0).
    let mut cp_info = CpInfo { max_char_size: 0, default_char: [0; 2], lead_byte: [0; 12] };
    let mut sb_vector = [0u8; 256];
    let mut up_vector = [0u8; 256];
    let mut low_vector = [0u8; 256];
    let mut w_vector = [0u16; 512];

    if (*ptmbci).mbcodepage != CP_UTF8
        && GetCPInfo((*ptmbci).mbcodepage as c_uint, &mut cp_info) != 0
    {
        let mut ich = 0usize;
        while ich < 256 {
            sb_vector[ich] = ich as u8;
            ich += 1;
        }

        // Byte 0 and every lead byte become the non-alpha ' '. The walk checks
        // only the pair's LOW byte, as the C does; it cannot leave the struct
        // because the tail padding (offset 18) was zeroed above.
        sb_vector[0] = b' ';
        let base = &cp_info as *const CpInfo as *const u8;
        let mut pb_pair = base.add(core::mem::offset_of!(CpInfo, lead_byte));
        while *pb_pair != 0 {
            let mut ich = *pb_pair as c_uint;
            while ich <= *pb_pair.add(1) as c_uint && ich < 256 {
                sb_vector[ich as usize] = b' ';
                ich += 1;
            }
            pb_pair = pb_pair.add(2);
        }

        __acrt_GetStringTypeA(
            null_mut(),
            CT_CTYPE1,
            sb_vector.as_ptr() as *const _,
            256,
            w_vector.as_mut_ptr(),
            (*ptmbci).mbcodepage,
            0,
        );
        __acrt_LCMapStringA(
            null_mut(),
            (*ptmbci).mblocalename,
            LCMAP_LOWERCASE,
            sb_vector.as_ptr() as *const _,
            256,
            low_vector.as_mut_ptr() as *mut _,
            256,
            (*ptmbci).mbcodepage,
            0,
        );
        __acrt_LCMapStringA(
            null_mut(),
            (*ptmbci).mblocalename,
            LCMAP_UPPERCASE,
            sb_vector.as_ptr() as *const _,
            256,
            up_vector.as_mut_ptr() as *mut _,
            256,
            (*ptmbci).mbcodepage,
            0,
        );

        let mut ich = 0usize;
        while ich < 256 {
            if w_vector[ich] & _UPPER != 0 {
                (*ptmbci).mbctype[ich + 1] |= _SBUP;
                (*ptmbci).mbcasemap[ich] = low_vector[ich];
            } else if w_vector[ich] & _LOWER != 0 {
                (*ptmbci).mbctype[ich + 1] |= _SBLOW;
                (*ptmbci).mbcasemap[ich] = up_vector[ich];
            } else {
                (*ptmbci).mbcasemap[ich] = 0;
            }
            ich += 1;
        }
    } else {
        // UTF-8 or no codepage info: ASCII casing only.
        let mut ich = 0usize;
        while ich < 256 {
            if ich >= b'A' as usize && ich <= b'Z' as usize {
                (*ptmbci).mbctype[ich + 1] |= _SBUP;
                (*ptmbci).mbcasemap[ich] = (ich + (b'a' - b'A') as usize) as u8;
            } else if ich >= b'a' as usize && ich <= b'z' as usize {
                (*ptmbci).mbctype[ich + 1] |= _SBLOW;
                (*ptmbci).mbcasemap[ich] = (ich - (b'a' - b'A') as usize) as u8;
            } else {
                (*ptmbci).mbcasemap[ich] = 0;
            }
            ich += 1;
        }
    }
}

// ── exports ───────────────────────────────────────────────────────────────────

/// `__p__mbctype`
#[no_mangle]
pub unsafe extern "C" fn __p__mbctype() -> *mut u8 {
    _mbctype
}

/// `__p__mbcasemap`
#[no_mangle]
pub unsafe extern "C" fn __p__mbcasemap() -> *mut u8 {
    _mbcasemap
}

/// `_setmbcp`: 0 on success (including "already that codepage"), -1 with
/// errno EINVAL if the codepage is refused, -1 WITHOUT errno if the allocation
/// fails.
#[no_mangle]
pub unsafe extern "C" fn _setmbcp(codepage: c_int) -> c_int {
    setmbcp_internal(codepage, false, __acrt_getptd(), addr_of_mut!(__acrt_current_multibyte_data))
}

/// `_setmbcp_nolock`: fill `ptmbci` for `codepage`. 0 or -1; on -1 `ptmbci`
/// may already be partly rewritten (the C checks validity only after the CRT
/// table search, and the UTF-8/GetCPInfo paths write as they go).
#[no_mangle]
pub unsafe extern "C" fn _setmbcp_nolock(codepage: c_int, ptmbci: *mut CrtMultibyteData) -> c_int {
    let codepage = get_system_cp(codepage);

    if codepage == _MB_CP_SBCS {
        set_sbcs(ptmbci);
        return 0;
    }

    // The CRT's own tables first.
    let mut icp = 0usize;
    while icp < RGCODE_PAGE_INFO.len() {
        if RGCODE_PAGE_INFO[icp].code_page == codepage {
            core::ptr::write_bytes(addr_of_mut!((*ptmbci).mbctype) as *mut u8, 0, NUM_CHARS);

            // Walk each row's (low, high) pairs until a pair has a zero. The
            // pointer is derived from the whole table, as the C's is: no row
            // fills all four pairs, so the walk never actually crosses a row.
            let table = RGCODE_PAGE_INFO.as_ptr() as *const u8;
            let row0 = icp * core::mem::size_of::<CodePageInfo>()
                + core::mem::offset_of!(CodePageInfo, rgrange);
            let mut irg = 0usize;
            while irg < NUM_CTYPES {
                let mut rgptr = table.add(row0 + irg * MAX_RANGES);
                while *rgptr != 0 && *rgptr.add(1) != 0 {
                    let mut ich = *rgptr as c_uint;
                    while ich <= *rgptr.add(1) as c_uint && ich < 256 {
                        (*ptmbci).mbctype[ich as usize + 1] |= RGCTYPEFLAG[irg];
                        ich += 1;
                    }
                    rgptr = rgptr.add(2);
                }
                irg += 1;
            }

            (*ptmbci).mbcodepage = codepage;
            // Every tabled codepage is truly multibyte.
            (*ptmbci).ismbcodepage = 1;
            (*ptmbci).mblocalename = cp_to_locale_name((*ptmbci).mbcodepage);
            (*ptmbci).mbulinfo = RGCODE_PAGE_INFO[icp].mbulinfo;

            set_sb_up_low(ptmbci);
            return 0;
        }
        icp += 1;
    }

    // `codepage == 0` is dead here (SBCS returned above); kept as the C has it.
    // `(WORD)codepage`: IsValidCodePage sees only the low 16 bits.
    if codepage == 0 || codepage == CP_UTF7 || IsValidCodePage(codepage as u16 as c_uint) == 0 {
        return -1;
    }

    if codepage == CP_UTF8 {
        (*ptmbci).mbcodepage = CP_UTF8;
        (*ptmbci).mblocalename = null();

        // C BUG, transcribed: the loop bound is NUM_ULINFO (6), not NUM_CHARS,
        // so only mbctype[0..6] is cleared and the previous codepage's flags
        // survive from byte 5 upwards. Microsoft's UCRT behaves this way.
        core::ptr::write_bytes(addr_of_mut!((*ptmbci).mbctype) as *mut u8, 0, NUM_ULINFO);

        (*ptmbci).ismbcodepage = 0;
        core::ptr::write_bytes(addr_of_mut!((*ptmbci).mbulinfo) as *mut u16, 0, NUM_ULINFO);

        set_sb_up_low(ptmbci);
        return 0;
    }

    // Not one of ours: ask the OS.
    let mut cp_info = CpInfo { max_char_size: 0, default_char: [0; 2], lead_byte: [0; 12] };
    if GetCPInfo(codepage as c_uint, &mut cp_info) != 0 {
        core::ptr::write_bytes(addr_of_mut!((*ptmbci).mbctype) as *mut u8, 0, NUM_CHARS);

        (*ptmbci).mbcodepage = codepage;
        (*ptmbci).mblocalename = null();

        if cp_info.max_char_size == 2 {
            // Lead-byte pairs, terminated by a pair with a zero. No `< 256`
            // guard here (the C has none; a byte cannot exceed 255 anyway).
            let base = &cp_info as *const CpInfo as *const u8;
            let mut lbptr = base.add(core::mem::offset_of!(CpInfo, lead_byte));
            while *lbptr != 0 && *lbptr.add(1) != 0 {
                let mut ich = *lbptr as c_uint;
                while ich <= *lbptr.add(1) as c_uint {
                    (*ptmbci).mbctype[ich as usize + 1] |= _M1;
                    ich += 1;
                }
                lbptr = lbptr.add(2);
            }

            // Every byte 0x01..=0xFE is a valid trail byte (0xFF is not).
            let mut ich = 0x01usize;
            while ich < 0xFF {
                (*ptmbci).mbctype[ich + 1] |= _M2;
                ich += 1;
            }

            (*ptmbci).mblocalename = cp_to_locale_name((*ptmbci).mbcodepage);
            (*ptmbci).ismbcodepage = 1;
        } else {
            (*ptmbci).ismbcodepage = 0;
        }

        core::ptr::write_bytes(addr_of_mut!((*ptmbci).mbulinfo) as *mut u16, 0, NUM_ULINFO);

        set_sb_up_low(ptmbci);
        return 0;
    }

    // A system-default request never fails: fall back to SBCS. (Only reachable
    // by a direct call with -2/-3/-4 -- see the module docs.)
    if F_SYSTEM_SET != 0 {
        set_sbcs(ptmbci);
        return 0;
    }

    -1
}

/// `_getmbcp`: the current multibyte codepage, or 0 when it is not multibyte.
#[no_mangle]
pub unsafe extern "C" fn _getmbcp() -> c_int {
    let loc_update = LocaleUpdate::new(null_mut());
    if (*loc_update.mbcinfo()).ismbcodepage != 0 {
        (*loc_update.mbcinfo()).mbcodepage
    } else {
        0
    }
}

/// `initialized`, the function-local static of `__acrt_initialize_multibyte`.
static mut INITIALIZED: bool = false;

/// `__acrt_initialize_multibyte`: point the globals at their storage and set
/// the process codepage to the ACP. Runs once, at CRT startup; always true.
#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_multibyte() -> bool {
    if !INITIALIZED {
        __acrt_current_multibyte_data = initial_multibyte_data();

        _mbcasemap = addr_of_mut!(MBCASEMAPS[0]) as *mut u8;
        _mbctype = addr_of_mut!(MBCTYPES[0]) as *mut u8;

        let ptd_head = __acrt_getptd_head();
        // state_index_count == 1
        let mut i = 0usize;
        while i != 1 {
            setmbcp_internal(
                _MB_CP_ANSI,
                true,
                ptd_head.add(i),
                addr_of_mut!(__acrt_current_multibyte_data).add(i),
            );
            i += 1;
        }

        INITIALIZED = true;
    }

    true
}
