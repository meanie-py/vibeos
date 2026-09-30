//! The CRT's narrow wrappers over the wide NLS API: `__acrt_CompareStringA/W`,
//! `__acrt_LCMapStringA/W`, `__acrt_GetLocaleInfoA`, `__acrt_GetStringTypeA`.
//! Replaces `sdk/lib/ucrt/locale/CompareStringA.cpp`, `CompareStringW.cpp`,
//! `LCMapStringA.cpp`, `LCMapStringW.cpp`, `GetLocaleInfoA.cpp` and
//! `GetStringTypeA.cpp` in full.
//!
//! The A forms convert through a code page (`0` = the locale's
//! `_locale_lc_codepage`, resolved through `_LocaleUpdate`), call the `Ex`
//! thunk from winapi_thunks.cpp, and convert back. They are what setlocale's
//! initialisers and every `_stricoll` / `_strlwr_l` / `strxfrm` path sit on.
//!
//! ## Details that are silent if wrong
//!
//! * **Counts are clipped at an embedded NUL** (`__strncnt` / `wcsnlen`) because
//!   the OS compares and maps PAST a NUL. LCMapString then counts the NUL back
//!   in if one was found inside the count; CompareString does not.
//! * **CompareStringA's empty-string special case** answers without the OS:
//!   both empty = 2; the other side longer than ONE char = 1 or 3; exactly one
//!   char vs empty = "equal" (2) if that char is a naked lead byte of the code
//!   page, else unequal. A count of -1 against an empty string skips every arm
//!   and goes to the OS. A count below -1 is failure (0) -- in the A form only.
//! * **The size query uses MB_ERR_INVALID_CHARS, the conversion does not**
//!   (CompareStringA always; LCMapStringA and GetStringTypeA only when `bError`).
//!   So an invalid byte fails the call only when it is strict about it.
//! * **LCMapStringA passes the WIDE count, including any NUL, to the OS**, and
//!   with LCMAP_SORTKEY the destination is the caller's BYTE buffer, passed
//!   straight through as if it were wide (`cchDest` in bytes); a too-small
//!   `cchDest` is a plain 0 BEFORE the OS is asked. `cchDest == 0` is a size
//!   query in both arms.
//! * **`__acrt_GetLocaleInfoA` stores a POINTER-sized NULL through
//!   `void_result` first, for every `lc_type`** -- including LC_INT_TYPE, whose
//!   target is a one-byte (or int) field. Callers' neighbouring fields get
//!   zeroed; see `locale_init`'s module docs for where that lands. The integer
//!   result is truncated to `unsigned char`.
//! * **LC_STR_TYPE tries a 128-byte stack buffer first** and retries at the
//!   exact size only when that failed with ERROR_INSUFFICIENT_BUFFER; any other
//!   failure is -1 at once. The result is calloc'd at exactly `length` bytes.
//! * **GetStringTypeA returns what GetStringTypeW returns** (a BOOL), and
//!   classifies `actual_extent` WIDE chars -- one per multibyte character,
//!   which is only one per byte for an SBCS page.
//!
//! `_malloca` is reproduced (`Malloca` below) rather than replaced with the
//! heap: the C takes the stack when `bytes + _ALLOCA_S_MARKER_SIZE <= 1024`, and
//! a heap allocation there could fail (and set errno) where the C cannot.

use crate::locale::{CrtLocalePointers, LC_INT_TYPE, LC_STR_TYPE, LC_WSTR_TYPE};
use crate::locale_abi::LocaleT;
use crate::locale_init::{CpInfo, GetCPInfo};
use crate::locale_update::LocaleUpdate;
use core::ffi::{c_char, c_int, c_void};
use core::mem::MaybeUninit;
use core::ptr::{null, null_mut};

// ── constants (values from sdk/include/psdk/winnls.h, winerror.h, malloc.h) ──

const MB_PRECOMPOSED: u32 = 1;
const MB_ERR_INVALID_CHARS: u32 = 8;
const LCMAP_SORTKEY: u32 = 0x0000_0400;
const LOCALE_RETURN_NUMBER: u32 = 0x2000_0000;
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;

/// `_ALLOCA_S_THRESHOLD`
const ALLOCA_S_THRESHOLD: usize = 1024;
/// `_ALLOCA_S_MARKER_SIZE`: 16 on _WIN64, 8 otherwise.
#[cfg(target_pointer_width = "64")]
const ALLOCA_S_MARKER_SIZE: usize = 16;
#[cfg(target_pointer_width = "32")]
const ALLOCA_S_MARKER_SIZE: usize = 8;

extern "system" {
    fn GetLastError() -> u32;
    fn GetStringTypeW(info_type: u32, src: *const u16, cch_src: c_int, char_type: *mut u16) -> c_int;

    // winapi_thunks.cpp (corecrt_internal.h: `int WINAPI ...`).
    fn __acrt_CompareStringEx(
        locale_name: *const u16,
        flags: u32,
        string1: *const u16,
        string1_count: c_int,
        string2: *const u16,
        string2_count: c_int,
        version: *mut c_void,
        reserved: *mut c_void,
        param: isize,
    ) -> c_int;
    fn __acrt_LCMapStringEx(
        locale_name: *const u16,
        flags: u32,
        source: *const u16,
        source_count: c_int,
        destination: *mut u16,
        destination_count: c_int,
        version: *mut c_void,
        reserved: *mut c_void,
        sort_handle: isize,
    ) -> c_int;
    fn __acrt_GetLocaleInfoEx(locale_name: *const u16, lc_type: u32, data: *mut u16, data_count: c_int) -> c_int;
}

extern "C" {
    // corecrt_internal.h:601 / :611 (cdecl).
    fn __acrt_MultiByteToWideChar(
        code_page: u32,
        flags: u32,
        mb: *const c_char,
        cb_mb: c_int,
        wide: *mut u16,
        cch_wide: c_int,
    ) -> c_int;
    fn __acrt_WideCharToMultiByte(
        code_page: u32,
        flags: u32,
        wide: *const u16,
        cch_wide: c_int,
        mb: *mut c_char,
        cb_mb: c_int,
        default_char: *const c_char,
        used_default_char: *mut c_int,
    ) -> c_int;
    /// string/strncnt.cpp
    fn __strncnt(string: *const c_char, count: usize) -> usize;
    fn wcsnlen(string: *const u16, count: usize) -> usize;
    fn strncpy_s(dst: *mut c_char, dst_size: usize, src: *const c_char, count: usize) -> c_int;
    fn _invoke_watson(e: *const u16, f: *const u16, file: *const u16, line: u32, r: usize) -> !;
    // `_malloc_crt` / `_calloc_crt` / `_free_crt` (internal_shared.h:176-178).
    fn malloc(size: usize) -> *mut c_void;
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn free(block: *mut c_void);
}

/// `_malloca_crt_t(wchar_t, n)` + `__crt_scoped_stack_ptr` (release `_malloca`):
/// the stack when the marked size is within `_ALLOCA_S_THRESHOLD`, otherwise
/// the heap at the marked size, NULL if the marked size overflows. The pointer
/// is only valid while this value stays where it was declared.
struct Malloca {
    stack: MaybeUninit<[u16; ALLOCA_S_THRESHOLD / 2]>,
    /// The start of the heap block (the C's marker slot), or NULL.
    heap: *mut u8,
    use_stack: bool,
    ok: bool,
}

impl Malloca {
    unsafe fn new(count: c_int) -> Self {
        let mut m = Self { stack: MaybeUninit::uninit(), heap: null_mut(), use_stack: false, ok: false };
        let bytes = (count as usize).wrapping_mul(2);
        // _MallocaComputeSize: 0 (=> NULL) when size + marker wraps.
        let marked = match bytes.checked_add(ALLOCA_S_MARKER_SIZE) {
            Some(n) => n,
            None => return m,
        };
        if marked <= ALLOCA_S_THRESHOLD {
            m.use_stack = true;
            m.ok = true;
        } else {
            // malloc at the marked size, as the C does; the marker itself is
            // only ever read back by _freea, which this replaces.
            m.heap = malloc(marked) as *mut u8;
            m.ok = !m.heap.is_null();
        }
        m
    }

    fn ptr(&mut self) -> *mut u16 {
        if !self.ok {
            null_mut()
        } else if self.use_stack {
            // Stack: the C's _MarkAllocaS skips a marker at the front; its
            // position in the frame is not observable, so no offset here.
            self.stack.as_mut_ptr() as *mut u16
        } else {
            unsafe { self.heap.add(ALLOCA_S_MARKER_SIZE) as *mut u16 }
        }
    }
}

impl Drop for Malloca {
    fn drop(&mut self) {
        if !self.heap.is_null() {
            unsafe { free(self.heap as *mut c_void) };
        }
    }
}

// ── CompareStringA.cpp ───────────────────────────────────────────────────────

/// Is `byte` inside one of the code page's lead-byte ranges?
fn is_lead_byte(info: &CpInfo, byte: u8) -> bool {
    info.lead_byte_ranges(|lo, hi| byte >= lo && byte <= hi)
}

unsafe fn internal_compare_string_a(
    plocinfo: LocaleT,
    locale_name: *const u16,
    cmp_flags: u32,
    string1: *const c_char,
    mut count1: c_int,
    string2: *const c_char,
    mut count2: c_int,
    mut code_page: c_int,
) -> c_int {
    // CompareString compares past a NUL: clip each count at one.
    if count1 > 0 {
        count1 = __strncnt(string1, count1 as usize) as c_int;
    } else if count1 < -1 {
        return 0;
    }

    if count2 > 0 {
        count2 = __strncnt(string2, count2 as usize) as c_int;
    } else if count2 < -1 {
        return 0;
    }

    if code_page == 0 {
        code_page = (*(*plocinfo).locinfo).public.locale_lc_codepage as c_int;
    }

    // At least one count is zero.
    if count1 == 0 || count2 == 0 {
        if count1 == count2 {
            return 2;
        }
        if count2 > 1 {
            return 1;
        }
        if count1 > 1 {
            return 3;
        }

        // One is empty and the other has ONE char: a naked lead byte compares
        // equal to nothing, anything else is greater. (A count of -1 against 0
        // falls through every arm below to the OS.)
        let mut cp_info = CpInfo::zeroed();
        if GetCPInfo(code_page as u32, &mut cp_info) == 0 {
            return 0;
        }

        if count1 > 0 {
            if cp_info.max_char_size < 2 {
                return 3;
            }
            if is_lead_byte(&cp_info, *string1 as u8) {
                return 2;
            }
            return 3;
        }

        if count2 > 0 {
            if cp_info.max_char_size < 2 {
                return 1;
            }
            if is_lead_byte(&cp_info, *string2 as u8) {
                return 2;
            }
            return 1;
        }
    }

    // Size (strict), then convert (lenient) -- each string in turn.
    let buff_size1 = __acrt_MultiByteToWideChar(
        code_page as u32,
        MB_PRECOMPOSED | MB_ERR_INVALID_CHARS,
        string1,
        count1,
        null_mut(),
        0,
    );
    if buff_size1 == 0 {
        return 0;
    }
    let mut wbuffer1 = Malloca::new(buff_size1);
    let w1 = wbuffer1.ptr();
    if w1.is_null() {
        return 0;
    }
    if __acrt_MultiByteToWideChar(code_page as u32, MB_PRECOMPOSED, string1, count1, w1, buff_size1) == 0 {
        return 0;
    }

    let buff_size2 = __acrt_MultiByteToWideChar(
        code_page as u32,
        MB_PRECOMPOSED | MB_ERR_INVALID_CHARS,
        string2,
        count2,
        null_mut(),
        0,
    );
    if buff_size2 == 0 {
        return 0;
    }
    let mut wbuffer2 = Malloca::new(buff_size2);
    let w2 = wbuffer2.ptr();
    if w2.is_null() {
        return 0;
    }
    let actual_size = __acrt_MultiByteToWideChar(code_page as u32, MB_PRECOMPOSED, string2, count2, w2, buff_size2);
    if actual_size == 0 {
        return 0;
    }

    // The buffer SIZES, not the clipped counts: a -1 count's NUL is included.
    __acrt_CompareStringEx(locale_name, cmp_flags, w1, buff_size1, w2, buff_size2, null_mut(), null_mut(), 0)
}

/// `int __acrt_CompareStringA(_locale_t, LPCWSTR, DWORD, PCCH, int, PCCH, int, int)`:
/// 1 / 2 / 3 for less / equal / greater, 0 on failure.
#[no_mangle]
pub unsafe extern "C" fn __acrt_CompareStringA(
    locale: LocaleT,
    locale_name: *const u16,
    compare_flags: u32,
    string1: *const c_char,
    string1_count: c_int,
    string2: *const c_char,
    string2_count: c_int,
    code_page: c_int,
) -> c_int {
    let mut locale_update = LocaleUpdate::new(locale);
    internal_compare_string_a(
        locale_update.locale_t(),
        locale_name,
        compare_flags,
        string1,
        string1_count,
        string2,
        string2_count,
        code_page,
    )
}

// ── CompareStringW.cpp ───────────────────────────────────────────────────────

/// `int __acrt_CompareStringW(LPCWSTR, DWORD, PCWCH, int, PCWCH, int)`.
#[no_mangle]
pub unsafe extern "C" fn __acrt_CompareStringW(
    locale_name: *const u16,
    cmp_flags: u32,
    string1: *const u16,
    mut count1: c_int,
    string2: *const u16,
    mut count2: c_int,
) -> c_int {
    if count1 > 0 {
        count1 = wcsnlen(string1, count1 as usize) as c_int;
    }
    if count2 > 0 {
        count2 = wcsnlen(string2, count2 as usize) as c_int;
    }

    // Unlike the A form: no lead-byte arm, no rejection of counts below -1,
    // and the answer is by SUBTRACTION, so a -1 count against 0 is "less".
    if count1 == 0 || count2 == 0 {
        let d = count1.wrapping_sub(count2);
        return if d == 0 {
            2
        } else if d < 0 {
            1
        } else {
            3
        };
    }

    __acrt_CompareStringEx(locale_name, cmp_flags, string1, count1, string2, count2, null_mut(), null_mut(), 0)
}

// ── LCMapStringA.cpp ─────────────────────────────────────────────────────────

unsafe fn lcmap_string_a_stat(
    plocinfo: LocaleT,
    locale_name: *const u16,
    map_flags: u32,
    src: *const c_char,
    mut cch_src: c_int,
    dest: *mut c_char,
    cch_dest: c_int,
    mut code_page: c_int,
    b_error: c_int,
) -> c_int {
    // LCMapString maps past a NUL: clip at one, then count it back in if it
    // was inside the given count.
    if cch_src > 0 {
        let cnt = __strncnt(src, cch_src as usize) as c_int;
        cch_src = if cnt < cch_src { cnt + 1 } else { cnt };
    }

    if code_page == 0 {
        code_page = (*(*plocinfo).locinfo).public.locale_lc_codepage as c_int;
    }

    let inbuff_size = __acrt_MultiByteToWideChar(
        code_page as u32,
        if b_error != 0 { MB_PRECOMPOSED | MB_ERR_INVALID_CHARS } else { MB_PRECOMPOSED },
        src,
        cch_src,
        null_mut(),
        0,
    );
    if inbuff_size == 0 {
        return 0;
    }

    let mut inwbuffer = Malloca::new(inbuff_size);
    let inw = inwbuffer.ptr();
    if inw.is_null() {
        return 0;
    }

    if __acrt_MultiByteToWideChar(code_page as u32, MB_PRECOMPOSED, src, cch_src, inw, inbuff_size) == 0 {
        return 0;
    }

    // Size of the mapping: BYTES for a sort key, wide chars otherwise.
    let mut retval =
        __acrt_LCMapStringEx(locale_name, map_flags, inw, inbuff_size, null_mut(), 0, null_mut(), null_mut(), 0);
    if retval == 0 {
        return retval;
    }

    if map_flags & LCMAP_SORTKEY != 0 {
        if cch_dest != 0 {
            if retval > cch_dest {
                return 0;
            }
            // The caller's byte buffer goes straight to the OS as the
            // "wide" destination, with its BYTE count.
            retval = __acrt_LCMapStringEx(
                locale_name,
                map_flags,
                inw,
                inbuff_size,
                dest as *mut u16,
                cch_dest,
                null_mut(),
                null_mut(),
                0,
            );
            if retval == 0 {
                return retval;
            }
        }
    } else {
        let outbuff_size = retval;

        let mut outwbuffer = Malloca::new(outbuff_size);
        let outw = outwbuffer.ptr();
        if outw.is_null() {
            return 0;
        }

        retval = __acrt_LCMapStringEx(
            locale_name,
            map_flags,
            inw,
            inbuff_size,
            outw,
            outbuff_size,
            null_mut(),
            null_mut(),
            0,
        );
        if retval == 0 {
            return retval;
        }

        if cch_dest == 0 {
            // Size required.
            retval =
                __acrt_WideCharToMultiByte(code_page as u32, 0, outw, outbuff_size, null_mut(), 0, null(), null_mut());
            if retval == 0 {
                return retval;
            }
        } else {
            retval = __acrt_WideCharToMultiByte(
                code_page as u32,
                0,
                outw,
                outbuff_size,
                dest,
                cch_dest,
                null(),
                null_mut(),
            );
            if retval == 0 {
                return retval;
            }
        }
    }

    retval
}

/// `int __acrt_LCMapStringA(_locale_t, PCWSTR, DWORD, PCCH, int, PCH, int, int, BOOL)`:
/// chars (bytes for LCMAP_SORTKEY) written or required, 0 on failure.
#[no_mangle]
pub unsafe extern "C" fn __acrt_LCMapStringA(
    plocinfo: LocaleT,
    locale_name: *const u16,
    map_flags: u32,
    src: *const c_char,
    cch_src: c_int,
    dest: *mut c_char,
    cch_dest: c_int,
    code_page: c_int,
    error: c_int,
) -> c_int {
    let mut loc_update = LocaleUpdate::new(plocinfo);
    lcmap_string_a_stat(
        loc_update.locale_t(),
        locale_name,
        map_flags,
        src,
        cch_src,
        dest,
        cch_dest,
        code_page,
        error,
    )
}

// ── LCMapStringW.cpp ─────────────────────────────────────────────────────────

/// `int __acrt_LCMapStringW(LPCWSTR, DWORD, LPCWSTR, int, LPWSTR, int)`.
#[no_mangle]
pub unsafe extern "C" fn __acrt_LCMapStringW(
    locale_name: *const u16,
    map_flags: u32,
    source: *const u16,
    mut source_count: c_int,
    destination: *mut u16,
    destination_count: c_int,
) -> c_int {
    if source_count > 0 {
        let source_length = wcsnlen(source, source_count as usize) as c_int;
        source_count = if source_length < source_count { source_length + 1 } else { source_length };
    }

    __acrt_LCMapStringEx(
        locale_name,
        map_flags,
        source,
        source_count,
        destination,
        destination_count,
        null_mut(),
        null_mut(),
        0,
    )
}

// ── GetLocaleInfoA.cpp ───────────────────────────────────────────────────────

/// `InternalGetLocaleInfoA`: `GetLocaleInfoEx` converted through the locale's
/// code page. Returns the narrow length (incl. NUL), or 0. `result_size == 0`
/// is a size query.
unsafe fn internal_get_locale_info_a(
    locale: LocaleT,
    locale_name: *const u16,
    locale_type: u32,
    result: *mut c_char,
    result_size: c_int,
) -> c_int {
    let mut locale_update = LocaleUpdate::new(locale);
    let code_page = (*(*locale_update.locale_t()).locinfo).public.locale_lc_codepage as c_int;

    let buffer_size = __acrt_GetLocaleInfoEx(locale_name, locale_type, null_mut(), 0);
    if buffer_size == 0 {
        return 0;
    }

    let mut buffer = Malloca::new(buffer_size);
    let b = buffer.ptr();
    if b.is_null() {
        return 0;
    }

    if __acrt_GetLocaleInfoEx(locale_name, locale_type, b, buffer_size) == 0 {
        return 0;
    }

    __acrt_WideCharToMultiByte(
        code_page as u32,
        0,
        b,
        -1,
        if result_size != 0 { result } else { null_mut() },
        result_size,
        null(),
        null_mut(),
    )
}

/// `int __acrt_GetLocaleInfoA(_locale_t, int lc_type, const wchar_t*, LCTYPE, void*)`:
/// 0 on success, -1 on failure. `void_result` is a `char**` (LC_STR_TYPE), a
/// `wchar_t**` (LC_WSTR_TYPE, both malloc'd for the caller to free) or an
/// `unsigned char*` (LC_INT_TYPE).
#[no_mangle]
pub unsafe extern "C" fn __acrt_GetLocaleInfoA(
    locale: LocaleT,
    lc_type: c_int,
    locale_name: *const u16,
    locale_type: u32,
    void_result: *mut c_void,
) -> c_int {
    // `*static_cast<void**>(void_result) = nullptr;` -- a POINTER-sized store
    // for EVERY lc_type, even when the target is a one-byte field (so it may
    // be misaligned: hence write_unaligned, which is the same store).
    core::ptr::write_unaligned(void_result as *mut *mut c_void, null_mut());

    if lc_type == LC_STR_TYPE {
        let char_result = void_result as *mut *mut c_char;

        const LOCAL_BUFFER_SIZE: c_int = 128;
        let mut local_buffer = MaybeUninit::<[c_char; LOCAL_BUFFER_SIZE as usize]>::uninit();
        let local = local_buffer.as_mut_ptr() as *mut c_char;

        let local_length = internal_get_locale_info_a(locale, locale_name, locale_type, local, LOCAL_BUFFER_SIZE);

        if local_length != 0 {
            let p = calloc(local_length as usize, 1) as *mut c_char;
            *char_result = p;
            if p.is_null() {
                return -1;
            }
            // _ERRCHECK: watson on any nonzero errno_t.
            if strncpy_s(p, local_length as usize, local, (local_length - 1) as usize) != 0 {
                _invoke_watson(null(), null(), null(), 0, 0);
            }
            return 0;
        }

        if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            return -1;
        }

        // Too small for the stack buffer: size it, then fetch into the heap.
        let required_length = internal_get_locale_info_a(locale, locale_name, locale_type, null_mut(), 0);
        if required_length == 0 {
            return -1;
        }

        let dynamic_buffer = calloc(required_length as usize, 1) as *mut c_char;
        if dynamic_buffer.is_null() {
            return -1;
        }

        let actual_length =
            internal_get_locale_info_a(locale, locale_name, locale_type, dynamic_buffer, required_length);
        if actual_length == 0 {
            free(dynamic_buffer as *mut c_void); // __crt_unique_heap_ptr
            return -1;
        }

        *char_result = dynamic_buffer;
        0
    } else if lc_type == LC_WSTR_TYPE {
        let wchar_result = void_result as *mut *mut u16;

        let required_length = __acrt_GetLocaleInfoEx(locale_name, locale_type, null_mut(), 0);
        if required_length == 0 {
            return -1;
        }

        let dynamic_buffer = calloc(required_length as usize, 2) as *mut u16;
        if dynamic_buffer.is_null() {
            return -1;
        }

        let actual_length = __acrt_GetLocaleInfoEx(locale_name, locale_type, dynamic_buffer, required_length);
        if actual_length == 0 {
            free(dynamic_buffer as *mut c_void);
            return -1;
        }

        *wchar_result = dynamic_buffer;
        0
    } else if lc_type == LC_INT_TYPE {
        let uchar_result = void_result as *mut u8;

        // A DWORD received through the wide buffer: 2 wchar_t's.
        let mut value: u32 = 0;
        let actual_length = __acrt_GetLocaleInfoEx(
            locale_name,
            locale_type | LOCALE_RETURN_NUMBER,
            &mut value as *mut u32 as *mut u16,
            (core::mem::size_of::<u32>() / core::mem::size_of::<u16>()) as c_int,
        );
        if actual_length == 0 {
            return -1;
        }

        *uchar_result = value as u8;
        0
    } else {
        -1
    }
}

// ── GetStringTypeA.cpp ───────────────────────────────────────────────────────

/// `BOOL __acrt_GetStringTypeA(_locale_t, DWORD, LPCSTR, int, unsigned short*, int, BOOL)`.
#[no_mangle]
pub unsafe extern "C" fn __acrt_GetStringTypeA(
    locale: LocaleT,
    info_type: u32,
    string: *const c_char,
    string_size_in_bytes: c_int,
    char_type: *mut u16,
    code_page: c_int,
    error: c_int,
) -> c_int {
    let mut locale_update = LocaleUpdate::new(locale);

    let actual_code_page = if code_page != 0 {
        code_page
    } else {
        let t: *mut CrtLocalePointers = locale_update.locale_t();
        (*(*t).locinfo).public.locale_lc_codepage as c_int
    };

    let required_extent = __acrt_MultiByteToWideChar(
        actual_code_page as u32,
        if error != 0 { MB_PRECOMPOSED | MB_ERR_INVALID_CHARS } else { MB_PRECOMPOSED },
        string,
        string_size_in_bytes,
        null_mut(),
        0,
    );
    if required_extent == 0 {
        return 0;
    }

    let mut buffer = Malloca::new(required_extent);
    let b = buffer.ptr();
    if b.is_null() {
        return 0;
    }

    core::ptr::write_bytes(b, 0, required_extent as usize);

    let actual_extent = __acrt_MultiByteToWideChar(
        actual_code_page as u32,
        MB_PRECOMPOSED,
        string,
        string_size_in_bytes,
        b,
        required_extent,
    );
    if actual_extent == 0 {
        return 0;
    }

    GetStringTypeW(info_type, b, actual_extent, char_type)
}
