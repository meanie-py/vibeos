//! Hand-written printf engine — a faithful port of ReactOS libcntpr
//! `printf/streamout.c` (Timo Kreuzer). NOT generated, NOT in
//! cnt-manifest.toml; the differential lives in cnt-difftest's
//! `ref_streamout` (the genuine streamout.c compiled on the host).
//!
//! `streamout`/`wstreamout` are the format-parser engine that every
//! printf-family entry point funnels through (sprintf, _snprintf,
//! _vsnprintf, _vscprintf, vsprintf, … stay C — they are irreducible
//! `va_start; streamout(...); va_end` shells, exactly like the scanf
//! trampoline). Those C wrappers call THIS `#[no_mangle] streamout`.
//!
//! ## va_list as a plain pointer
//! On every Windows target this crate builds for (i686 / x86_64 /
//! aarch64) `va_list` is a single `char*` walking the spilled-argument
//! area — NOT the SysV register-save-area struct. So the engine takes
//! the `va_list` by value as a `*mut u8` cursor and does `va_arg`
//! itself by pointer arithmetic: read `size_of::<T>()` bytes at the
//! cursor, then advance by the slot size (4 on i386, 8 on amd64/arm64;
//! a vararg slot is `max(size, ptr_width)`). Reading a `double` is just
//! an 8-byte memory load — it never crosses the by-value `f64` ABI, so
//! the engine compiles on the i386 no-SSE kernel build (no `float`
//! feature) too.
//!
//! ## The float path
//! `%f/%e/%g/%a` call out to `format_float`/`format_floatw` (the
//! genuine streamout's design — "one for kernel one for user mode").
//! That symbol is the only float-bearing piece:
//!   * i386 (cabi, no `float`): a C residual (`printf/format_float.c`)
//!     keeps the genuine x87 path (i386-kernel has no by-value f64 ABI).
//!   * amd64/arm64 (cabi + `float`): the Rust `format_float` below.
//! Either way the engine just hands it `(&mut string, &mut prefix,
//! &mut cursor)` and the implementation peels exactly one float arg.

//! ## Two configurations of one source
//! This file is also compiled by `user32-wsprintf-rust` (via `#[path]`), whose build.rs sets
//! `cfg(streamout_user32)`: the C's `-D_USER32_WSPRINTF` build of the same streamout.c, for
//! user32's wsprintf family. A cfg is per-crate, so unlike a cargo feature it cannot leak into
//! libcntpr's (the kernel's) build of this engine. The six differences are marked
//! `streamout_user32` below: no `_IOSTRG`/NULL-base counting stream, one size prefix
//! (USE_MULTISIZE 0), `*` width/precision printed literally, no float conversions, and
//! `fieldwidth += 2` for a `#` hex prefix.

#![allow(clippy::missing_safety_doc)]

use core::mem::size_of;

// ── FILE (genuine `struct _iobuf`, sdk/include/crt/stdio.h) ──────────
// streamout only touches _ptr / _cnt / _base / _flag; the rest are
// present for layout fidelity (repr(C) gives the per-arch ptr width).
#[repr(C)]
pub struct File {
    pub _ptr: *mut u8,
    pub _cnt: i32,
    pub _base: *mut u8,
    pub _flag: i32,
    pub _file: i32,
    pub _charbuf: i32,
    pub _bufsiz: i32,
    pub _tmpfname: *mut u8,
}

// `typedef struct _STRING { unsigned short Length, MaximumLength;
//  void *Buffer; } STRING;` — the `%Z` counted-string arg.
#[repr(C)]
struct NtString {
    length: u16,
    maximum_length: u16,
    buffer: *mut u8,
}

const _IOSTRG: i32 = 0x0040;
const MB_CUR_MAX: usize = 10;
const BUFFER_SIZE: usize = 32 + 17; // 49

// Formatting flags (genuine enum).
const FLAG_ALIGN_LEFT: u32 = 0x01;
const FLAG_FORCE_SIGN: u32 = 0x02;
const FLAG_FORCE_SIGNSP: u32 = 0x04;
const FLAG_PAD_ZERO: u32 = 0x08;
const FLAG_SPECIAL: u32 = 0x10;
const FLAG_SHORT: u32 = 0x100;
const FLAG_LONG: u32 = 0x200;
const FLAG_WIDECHAR: u32 = FLAG_LONG;
const FLAG_INT64: u32 = 0x400;
#[cfg(target_pointer_width = "64")]
const FLAG_INTPTR: u32 = FLAG_INT64;
#[cfg(target_pointer_width = "32")]
const FLAG_INTPTR: u32 = 0;

extern "C" {
    // `int mbtowc(wchar_t*, const char*, size_t)` / `int wctomb(char*,
    // wchar_t)` — libcntpr's mbstring; only reached by cross-width
    // conversions (%S in narrow printf, %s in wide). wchar_t = u16.
    fn mbtowc(wchar: *mut u16, mbchar: *const u8, count: usize) -> i32;
    fn wctomb(mbchar: *mut u8, wchar: u16) -> i32;
}

// The Rust `format_float`/`format_floatw` below now provide these names
// on ALL three arches — including the i386 no-SSE kernel, since the float
// engine (`mod ff` + core_rs::floatcvt) is no-SSE-safe via `extern "C"`
// x87 returns + the fb64/fb32 transmute shims. (Previously i386 used a
// C residual `printf/format_float.c`, now REMOVE_ITEM'd in libcntpr.cmake.)

// ── va_arg over a flat pointer cursor ────────────────────────────────
#[cfg(target_pointer_width = "32")]
const VA_SLOT: usize = 4;
#[cfg(target_pointer_width = "64")]
const VA_SLOT: usize = 8;

#[inline]
fn slot_of(sz: usize) -> usize {
    (sz + VA_SLOT - 1) / VA_SLOT * VA_SLOT
}

/// One `va_arg(*cursor, T)`: little-endian read at the slot start, then
/// advance the cursor by the rounded slot. `read_unaligned` covers the
/// i386 case where an 8-byte arg sits on a 4-byte boundary.
#[inline]
unsafe fn va_next<T: Copy>(cursor: &mut *mut u8) -> T {
    let v = core::ptr::read_unaligned(*cursor as *const T);
    *cursor = (*cursor).add(slot_of(size_of::<T>()));
    v
}

// ── stream char/string sinks (the `_LIBCNT_` branch) ─────────────────
/// `streamout_char` — write one stream unit (size_of::<T>()).
#[inline]
unsafe fn so_char<T: Unit>(s: *mut File, chr: u32) -> i32 {
    // `if ((_flag & _IOSTRG) && _base == NULL) return 1;` — the
    // _vscprintf counting nul-stream: succeed without storing. (`#if !_USER32_WSPRINTF`)
    #[cfg(not(streamout_user32))]
    if ((*s)._flag & _IOSTRG) != 0 && (*s)._base.is_null() {
        return 1;
    }
    let usz = size_of::<T>() as i32;
    // Genuine: `if (stream->_cnt < sizeof(_TCHAR)) return 0;` — `sizeof`
    // yields size_t, so C promotes `_cnt` (int) to UNSIGNED for the
    // compare. swprintf/_swprintf run UNBOUNDED: _sxprintf sets _cnt =
    // (int)(INT_MAX * sizeof(WCHAR)) which OVERFLOWS to a negative int;
    // under the unsigned compare that negative is a huge value, so the
    // "buffer full" guard is (correctly) never hit. A naive SIGNED
    // compare would treat it as full and refuse every write. Replicate
    // the unsigned promotion: sign-extend i32→isize, reinterpret usize.
    if ((*s)._cnt as isize as usize) < size_of::<T>() {
        return 0;
    }
    // Output write through stream->_ptr — volatile per the
    // captures(none)/ud2 discipline.
    core::ptr::write_volatile((*s)._ptr as *mut T, T::from_u32(chr));
    (*s)._ptr = (*s)._ptr.add(size_of::<T>());
    (*s)._cnt -= usz;
    1
}

/// `streamout_astring` — narrow (char) source into the T stream.
unsafe fn so_astring<T: Unit>(s: *mut File, mut string: *const u8, mut count: usize) -> i32 {
    #[cfg(not(streamout_user32))]
    if ((*s)._flag & _IOSTRG) != 0 && (*s)._base.is_null() {
        return count as i32;
    }
    let mut written = 0i32;
    while count > 0 {
        count -= 1;
        let chr: u32;
        if T::WIDE {
            let mut wc: u16 = 0;
            let len = mbtowc(&mut wc, string, MB_CUR_MAX);
            if len < 1 {
                break;
            }
            string = string.add(len as usize);
            chr = wc as u32;
        } else {
            chr = *string as u32;
            string = string.add(1);
        }
        if so_char::<T>(s, chr) == 0 {
            return -1;
        }
        written += 1;
    }
    written
}

/// `streamout_wstring` — wide (wchar) source into the T stream.
unsafe fn so_wstring<T: Unit>(s: *mut File, mut string: *const u16, mut count: usize) -> i32 {
    // genuine guard is `#if defined(_UNICODE) && !_USER32_WSPRINTF`.
    #[cfg(not(streamout_user32))]
    if T::WIDE && ((*s)._flag & _IOSTRG) != 0 && (*s)._base.is_null() {
        return count as i32;
    }
    let mut written = 0i32;
    while count > 0 {
        count -= 1;
        if !T::WIDE {
            let mut mbchar = [0u8; MB_CUR_MAX];
            let mblen = wctomb(mbchar.as_mut_ptr(), *string);
            string = string.add(1);
            if mblen <= 0 {
                return written;
            }
            for i in 0..mblen as usize {
                if so_char::<T>(s, mbchar[i] as u32) == 0 {
                    return -1;
                }
                written += 1;
            }
        } else {
            let chr = *string as u32;
            string = string.add(1);
            if so_char::<T>(s, chr) == 0 {
                return -1;
            }
            written += 1;
        }
    }
    written
}

/// `streamout_string` (the `#ifdef _UNICODE` alias): the stream-typed
/// prefix/string writer. `p` is `*const T`.
#[inline]
unsafe fn so_string<T: Unit>(s: *mut File, p: *const T, count: usize) -> i32 {
    if T::WIDE {
        so_wstring::<T>(s, p as *const u16, count)
    } else {
        so_astring::<T>(s, p as *const u8, count)
    }
}

// ── the stream char unit (u8 narrow / u16 wide) ──────────────────────
pub trait Unit: Copy + PartialEq {
    /// true when the STREAM is wide (`_UNICODE` build of streamout).
    const WIDE: bool;
    fn from_u32(v: u32) -> Self;
    fn to_u32(self) -> u32;
    /// `format_float` / `format_floatw` dispatch for this width.
    #[cfg(not(streamout_user32))]
    unsafe fn call_format_float(
        chr: u32,
        flags: u32,
        precision: i32,
        string: *mut *mut Self,
        prefix: *mut *const Self,
        cursor: *mut *mut u8,
    );
}

impl Unit for u8 {
    const WIDE: bool = false;
    #[inline]
    fn from_u32(v: u32) -> u8 {
        (v & 0xff) as u8
    }
    #[inline]
    fn to_u32(self) -> u32 {
        self as u32
    }
    #[cfg(not(streamout_user32))]
    #[inline]
    unsafe fn call_format_float(
        chr: u32,
        flags: u32,
        precision: i32,
        string: *mut *mut u8,
        prefix: *mut *const u8,
        cursor: *mut *mut u8,
    ) {
        format_float(chr as u8, flags, precision, string, prefix, cursor);
    }
}

impl Unit for u16 {
    const WIDE: bool = true;
    #[inline]
    fn from_u32(v: u32) -> u16 {
        v as u16
    }
    #[inline]
    fn to_u32(self) -> u32 {
        self as u32
    }
    #[cfg(not(streamout_user32))]
    #[inline]
    unsafe fn call_format_float(
        chr: u32,
        flags: u32,
        precision: i32,
        string: *mut *mut u16,
        prefix: *mut *const u16,
        cursor: *mut *mut u8,
    ) {
        format_floatw(chr as u16, flags, precision, string, prefix, cursor);
    }
}

#[inline]
unsafe fn tlen<T: Unit>(mut p: *const T) -> usize {
    let mut n = 0;
    while (*p).to_u32() != 0 {
        n += 1;
        p = p.add(1);
    }
    n
}

// ── the engine ───────────────────────────────────────────────────────
// One faithful body, monomorphized to streamout (T=u8) / wstreamout
// (T=u16). `_USER32_WSPRINTF` paths are NOT compiled (libcntpr is the
// `_LIBCNT_` / non-USER32 flavor), matching this crate's target.
// The literal-`*` branches end in `continue`, leaving the va_arg path after them dead.
#[cfg_attr(streamout_user32, allow(unreachable_code))]
unsafe fn streamout_impl<T: Unit>(stream: *mut File, format: *const T, mut cursor: *mut u8) -> i32 {
    // digits as bytes (ASCII), converted to T at use. [16]='0',[17]='x'.
    const DIGITS_L: &[u8; 18] = b"0123456789abcdef0x";
    const DIGITS_U: &[u8; 18] = b"0123456789ABCDEF0X";
    const NULLSTRING: &[u8; 7] = b"(null)\0";

    let mut buffer = [T::from_u32(0); BUFFER_SIZE + 1];
    buffer[BUFFER_SIZE] = T::from_u32(0);

    // small scratch for engine-built prefixes ("-","+"," ","0","0x").
    let mut pbuf = [T::from_u32(0); 3];

    let mut fmt = format;
    let mut written: i32 = 1;
    let mut written_all: i32 = 0;

    while written >= 0 {
        let mut chr = (*fmt).to_u32();
        fmt = fmt.add(1);

        // end of format string
        if chr == 0 {
            break;
        }

        // normal char or doubled '%'
        if chr != b'%' as u32 || {
            chr = (*fmt).to_u32();
            fmt = fmt.add(1);
            chr == b'%' as u32
        } {
            written = so_char::<T>(stream, chr);
            if written == 0 {
                return -1;
            }
            written_all += written;
            continue;
        }

        // flags
        let mut flags: u32 = 0;
        loop {
            if chr == b'-' as u32 {
                flags |= FLAG_ALIGN_LEFT;
            } else if chr == b'+' as u32 {
                flags |= FLAG_FORCE_SIGN;
            } else if chr == b' ' as u32 {
                flags |= FLAG_FORCE_SIGNSP;
            } else if chr == b'0' as u32 {
                flags |= FLAG_PAD_ZERO;
            } else if chr == b'#' as u32 {
                flags |= FLAG_SPECIAL;
            } else {
                break;
            }
            chr = (*fmt).to_u32();
            fmt = fmt.add(1);
        }

        // field width
        let mut fieldwidth: i32;
        if chr == b'*' as u32 {
            // _USER32_WSPRINTF prints the `*` and abandons the specification.
            #[cfg(streamout_user32)]
            {
                written = so_char::<T>(stream, chr);
                if written == 0 {
                    return -1;
                }
                written_all += written;
                continue;
            }
            fieldwidth = va_next::<i32>(&mut cursor);
            if fieldwidth < 0 {
                flags |= FLAG_ALIGN_LEFT;
                fieldwidth = -fieldwidth;
            }
            chr = (*fmt).to_u32();
            fmt = fmt.add(1);
        } else {
            fieldwidth = 0;
            while chr >= b'0' as u32 && chr <= b'9' as u32 {
                fieldwidth = fieldwidth * 10 + (chr - b'0' as u32) as i32;
                chr = (*fmt).to_u32();
                fmt = fmt.add(1);
            }
        }

        // precision
        let mut precision: i32;
        if chr == b'.' as u32 {
            chr = (*fmt).to_u32();
            fmt = fmt.add(1);
            if chr == b'*' as u32 {
                #[cfg(streamout_user32)]
                {
                    written = so_char::<T>(stream, chr);
                    if written == 0 {
                        return -1;
                    }
                    written_all += written;
                    continue;
                }
                precision = va_next::<i32>(&mut cursor);
                chr = (*fmt).to_u32();
                fmt = fmt.add(1);
            } else {
                precision = 0;
                while chr >= b'0' as u32 && chr <= b'9' as u32 {
                    precision = precision * 10 + (chr - b'0' as u32) as i32;
                    chr = (*fmt).to_u32();
                    fmt = fmt.add(1);
                }
            }
        } else {
            precision = -1;
        }

        // argument size prefix (USE_MULTISIZE loop; libcntpr = 1)
        loop {
            if chr == b'h' as u32 {
                flags |= FLAG_SHORT;
            } else if chr == b'w' as u32 {
                flags |= FLAG_WIDECHAR;
            } else if chr == b'L' as u32 {
                // FIXME: long double
            } else if chr == b'F' as u32 {
                // FIXME: what is that?
            } else if chr == b'z' as u32 && {
                // genuine: `*format && strchr("udxXion", *format)`.
                // Manual (no slice::contains → its try_fold emits an
                // always-unreachable NeverShortCircuit stub that
                // scan_ud2 flags).
                let n = (*fmt).to_u32();
                n != 0
                    && (n == b'u' as u32
                        || n == b'd' as u32
                        || n == b'x' as u32
                        || n == b'X' as u32
                        || n == b'i' as u32
                        || n == b'o' as u32
                        || n == b'n' as u32)
            } {
                flags |= FLAG_INTPTR;
            } else if chr == b'l' as u32 {
                // 2nd 'l' in a row → INT64 (genuine `format[-2] == 'l'`)
                if (*fmt.sub(2)).to_u32() == b'l' as u32 {
                    flags |= FLAG_INT64;
                } else {
                    flags |= FLAG_LONG;
                }
            } else if chr == b'I' as u32 {
                let f0 = (*fmt).to_u32();
                let f1 = (*fmt.add(1)).to_u32();
                if f0 == b'3' as u32 && f1 == b'2' as u32 {
                    fmt = fmt.add(2);
                } else if f0 == b'6' as u32 && f1 == b'4' as u32 {
                    fmt = fmt.add(2);
                    flags |= FLAG_INT64;
                } else if f0 == b'x' as u32
                    || f0 == b'X' as u32
                    || f0 == b'd' as u32
                    || f0 == b'i' as u32
                    || f0 == b'u' as u32
                    || f0 == b'o' as u32
                {
                    flags |= FLAG_INTPTR;
                } else {
                    break;
                }
            } else {
                break;
            }
            chr = (*fmt).to_u32();
            fmt = fmt.add(1);
            // USE_MULTISIZE: 1 for libcntpr (keep looping), 0 for _USER32_WSPRINTF.
            if cfg!(streamout_user32) {
                break;
            }
        }

        // format specifier
        let mut digits: &[u8; 18] = DIGITS_L;
        // `string = &buffer[BUFFER_SIZE]`; built backward via *--string.
        let mut string: *mut T = buffer.as_mut_ptr().add(BUFFER_SIZE);
        let mut base: u64 = 10;
        let mut prefix: *const T = core::ptr::null();
        let mut len: usize;
        let mut val64: u64;

        // Match the FULL char: a wide format unit > 0xFF must not alias
        // an ASCII specifier (genuine switches on the whole _TCHAR).
        let spec: u8 = if chr <= 0xff { chr as u8 } else { 0 };
        match spec {
            b'n' => {
                let p = va_next::<*mut u8>(&mut cursor);
                if flags & FLAG_INT64 != 0 {
                    core::ptr::write_volatile(p as *mut i64, written_all as i64);
                } else if flags & FLAG_SHORT != 0 {
                    core::ptr::write_volatile(p as *mut i16, written_all as i16);
                } else {
                    core::ptr::write_volatile(p as *mut i32, written_all);
                }
                continue;
            }

            b'C' | b'c' => {
                // 'C': in narrow build → wide unless SHORT; in wide
                // build the plain 'c' picks up WIDECHAR unless SHORT.
                if (spec == b'C' && !T::WIDE && (flags & FLAG_SHORT) == 0)
                    || (spec == b'c' && T::WIDE && (flags & FLAG_SHORT) == 0)
                {
                    flags |= FLAG_WIDECHAR;
                }
                string = buffer.as_mut_ptr();
                len = 1;
                let v = va_next::<i32>(&mut cursor) as u32;
                if flags & FLAG_WIDECHAR != 0 {
                    *(string as *mut u16) = v as u16;
                    *(string as *mut u16).add(1) = 0;
                } else {
                    *(string as *mut u8) = v as u8;
                    *(string as *mut u8).add(1) = 0;
                }
                emit::<T>(
                    stream, &mut written, &mut written_all, flags, fieldwidth, precision, prefix,
                    string, len, digits, &mut pbuf,
                );
                if written == -1 {
                    return -1;
                }
                continue;
            }

            b'Z' => {
                let p = va_next::<*mut u8>(&mut cursor) as *const NtString;
                if !p.is_null() && !(*p).buffer.is_null() {
                    string = (*p).buffer as *mut T;
                    len = (*p).length as usize;
                    if flags & FLAG_WIDECHAR != 0 {
                        len /= 2;
                    }
                    emit::<T>(
                        stream, &mut written, &mut written_all, flags, fieldwidth, precision,
                        prefix, string, len, digits, &mut pbuf,
                    );
                    if written == -1 {
                        return -1;
                    }
                    continue;
                }
                // fall through to case_string with NULL
                string = core::ptr::null_mut();
                len = case_string::<T>(&mut string, &mut flags, &mut precision, NULLSTRING);
                emit::<T>(
                    stream, &mut written, &mut written_all, flags, fieldwidth, precision, prefix,
                    string, len, digits, &mut pbuf,
                );
                if written == -1 {
                    return -1;
                }
                continue;
            }

            b'S' | b's' => {
                string = va_next::<*mut u8>(&mut cursor) as *mut T;
                // 'S': narrow build → wide unless SHORT. 's': wide build
                // → wide unless SHORT.
                if (spec == b'S' && !T::WIDE && (flags & FLAG_SHORT) == 0)
                    || (spec == b's' && T::WIDE && (flags & FLAG_SHORT) == 0)
                {
                    flags |= FLAG_WIDECHAR;
                }
                len = case_string::<T>(&mut string, &mut flags, &mut precision, NULLSTRING);
                emit::<T>(
                    stream, &mut written, &mut written_all, flags, fieldwidth, precision, prefix,
                    string, len, digits, &mut pbuf,
                );
                if written == -1 {
                    return -1;
                }
                continue;
            }

            #[cfg(not(streamout_user32))]
            b'G' | b'E' | b'A' | b'g' | b'e' | b'a' | b'f' => {
                if T::WIDE {
                    flags |= FLAG_WIDECHAR;
                } else {
                    flags &= !FLAG_WIDECHAR;
                }
                let mut prefix_mut: *const T = core::ptr::null();
                T::call_format_float(
                    chr,
                    flags,
                    precision,
                    &mut string,
                    &mut prefix_mut,
                    &mut cursor,
                );
                prefix = prefix_mut;
                len = tlen::<T>(string);
                precision = 0;
                emit::<T>(
                    stream, &mut written, &mut written_all, flags, fieldwidth, precision, prefix,
                    string, len, digits, &mut pbuf,
                );
                if written == -1 {
                    return -1;
                }
                continue;
            }

            b'd' | b'i' => {
                let v: i64 = if flags & FLAG_INT64 != 0 {
                    va_next::<i64>(&mut cursor)
                } else if flags & FLAG_SHORT != 0 {
                    va_next::<i32>(&mut cursor) as i16 as i64
                } else {
                    va_next::<i32>(&mut cursor) as i64
                };
                let mut sval = v;
                if sval < 0 {
                    sval = -sval;
                    prefix = lit::<T>(&mut pbuf, b"-");
                } else if flags & FLAG_FORCE_SIGN != 0 {
                    prefix = lit::<T>(&mut pbuf, b"+");
                } else if flags & FLAG_FORCE_SIGNSP != 0 {
                    prefix = lit::<T>(&mut pbuf, b" ");
                }
                val64 = sval as u64;
                len = case_number::<T>(
                    &mut string, &mut flags, &mut precision, &mut val64, base, digits,
                );
                emit::<T>(
                    stream, &mut written, &mut written_all, flags, fieldwidth, precision, prefix,
                    string, len, digits, &mut pbuf,
                );
                if written == -1 {
                    return -1;
                }
                continue;
            }

            b'o' => {
                base = 8;
                if flags & FLAG_SPECIAL != 0 {
                    prefix = lit::<T>(&mut pbuf, b"0");
                    if precision > 0 {
                        precision -= 1;
                    }
                }
                val64 = va_arg_fu::<T>(&mut cursor, flags);
                len = case_number::<T>(
                    &mut string, &mut flags, &mut precision, &mut val64, base, digits,
                );
                emit::<T>(
                    stream, &mut written, &mut written_all, flags, fieldwidth, precision, prefix,
                    string, len, digits, &mut pbuf,
                );
                if written == -1 {
                    return -1;
                }
                continue;
            }

            b'p' | b'X' | b'x' | b'u' => {
                let c = spec;
                if c == b'p' {
                    precision = 2 * size_of::<*mut u8>() as i32;
                    flags &= !FLAG_PAD_ZERO;
                    flags |= FLAG_INTPTR;
                }
                if c == b'p' || c == b'X' {
                    digits = DIGITS_U;
                }
                if c == b'p' || c == b'X' || c == b'x' {
                    base = 16;
                    if flags & FLAG_SPECIAL != 0 {
                        // prefix = &digits[16] → "0x"/"0X"
                        pbuf[0] = T::from_u32(digits[16] as u32);
                        pbuf[1] = T::from_u32(digits[17] as u32);
                        pbuf[2] = T::from_u32(0);
                        prefix = pbuf.as_ptr();
                        #[cfg(streamout_user32)]
                        {
                            fieldwidth += 2;
                        }
                    }
                }
                val64 = va_arg_fu::<T>(&mut cursor, flags);
                len = case_number::<T>(
                    &mut string, &mut flags, &mut precision, &mut val64, base, digits,
                );
                emit::<T>(
                    stream, &mut written, &mut written_all, flags, fieldwidth, precision, prefix,
                    string, len, digits, &mut pbuf,
                );
                if written == -1 {
                    return -1;
                }
                continue;
            }

            _ => {
                // anything else: re-emit as a normal character
                fmt = fmt.sub(1);
                continue;
            }
        }
    }

    if written == -1 {
        return -1;
    }
    written_all
}

/// `va_arg_fu` — unsigned integer arg honoring INT64/SHORT.
#[inline]
unsafe fn va_arg_fu<T: Unit>(cursor: &mut *mut u8, flags: u32) -> u64 {
    if flags & FLAG_INT64 != 0 {
        va_next::<u64>(cursor)
    } else if flags & FLAG_SHORT != 0 {
        va_next::<i32>(cursor) as u16 as u64
    } else {
        va_next::<i32>(cursor) as u32 as u64
    }
}

/// Build a NUL-terminated 1-char prefix in `pbuf`, return its pointer.
#[inline]
fn lit<T: Unit>(pbuf: &mut [T; 3], s: &[u8]) -> *const T {
    let mut i = 0;
    while i < s.len() {
        pbuf[i] = T::from_u32(s[i] as u32);
        i += 1;
    }
    pbuf[s.len()] = T::from_u32(0);
    pbuf.as_ptr()
}

/// `case_string` — compute the (bounded) length and NULL handling.
/// `string` may be retargeted to "(null)".
#[inline]
unsafe fn case_string<T: Unit>(
    string: &mut *mut T,
    flags: &mut u32,
    precision: &mut i32,
    nullstring: &[u8],
) -> usize {
    if string.is_null() {
        *string = nullstring.as_ptr() as *mut T; // narrow "(null)"
        *flags &= !FLAG_WIDECHAR;
    }
    let prec = *precision as u32 as usize; // (unsigned)precision
    let len = if *flags & FLAG_WIDECHAR != 0 {
        wcsnlen(*string as *const u16, prec)
    } else {
        strnlen(*string as *const u8, prec)
    };
    *precision = 0;
    len
}

/// `case_number` — gather digits in reverse, return length.
#[inline]
unsafe fn case_number<T: Unit>(
    string: &mut *mut T,
    flags: &mut u32,
    precision: &mut i32,
    val64: &mut u64,
    base: u64,
    digits: &[u8; 18],
) -> usize {
    if T::WIDE {
        *flags |= FLAG_WIDECHAR;
    } else {
        *flags &= !FLAG_WIDECHAR;
    }
    if *precision < 0 {
        *precision = 1;
    }
    while *val64 != 0 {
        *string = (*string).sub(1);
        **string = T::from_u32(digits[(*val64 % base) as usize] as u32);
        *val64 /= base;
        *precision -= 1;
    }
    tlen::<T>(*string)
}

/// The shared padding/prefix/output tail (genuine streamout lines
/// 642–700). Sets `*written`/`*written_all`; on a sink failure leaves
/// `*written == -1` for the caller's `return -1`.
#[inline]
#[allow(clippy::too_many_arguments)]
unsafe fn emit<T: Unit>(
    stream: *mut File,
    written: &mut i32,
    written_all: &mut i32,
    flags: u32,
    fieldwidth: i32,
    mut precision: i32,
    prefix: *const T,
    string: *mut T,
    len: usize,
    _digits: &[u8; 18],
    _pbuf: &mut [T; 3],
) {
    let prefixlen = if !prefix.is_null() { tlen::<T>(prefix) } else { 0 };
    if precision < 0 {
        precision = 0;
    }
    let mut padding = fieldwidth - len as i32 - prefixlen as i32 - precision;
    if padding < 0 {
        padding = 0;
    }

    // left space padding
    if flags & (FLAG_ALIGN_LEFT | FLAG_PAD_ZERO) == 0 {
        while padding > 0 {
            *written = so_char::<T>(stream, b' ' as u32);
            if *written == 0 {
                *written = -1;
                return;
            }
            *written_all += *written;
            padding -= 1;
        }
    }

    // prefix
    if !prefix.is_null() {
        *written = so_string::<T>(stream, prefix, prefixlen);
        if *written == -1 {
            return;
        }
        *written_all += *written;
    }

    // left '0' padding
    if flags & FLAG_ALIGN_LEFT == 0 {
        precision += padding;
    }
    while precision > 0 {
        *written = so_char::<T>(stream, b'0' as u32);
        if *written == 0 {
            *written = -1;
            return;
        }
        *written_all += *written;
        precision -= 1;
    }

    // the string
    if flags & FLAG_WIDECHAR != 0 {
        *written = so_wstring::<T>(stream, string as *const u16, len);
    } else {
        *written = so_astring::<T>(stream, string as *const u8, len);
    }
    if *written == -1 {
        return;
    }
    *written_all += *written;

    // right padding
    if flags & FLAG_ALIGN_LEFT != 0 {
        while padding > 0 {
            *written = so_char::<T>(stream, b' ' as u32);
            if *written == 0 {
                *written = -1;
                return;
            }
            *written_all += *written;
            padding -= 1;
        }
    }
}

// strnlen / wcsnlen — the engine's own bounded length (genuine
// streamout calls the libcntpr ones; these are equivalent and avoid an
// extra extern). `maxlen` is the (unsigned) precision.
#[inline]
unsafe fn strnlen(s: *const u8, maxlen: usize) -> usize {
    let mut n = 0;
    while n < maxlen && *s.add(n) != 0 {
        n += 1;
    }
    n
}
#[inline]
unsafe fn wcsnlen(s: *const u16, maxlen: usize) -> usize {
    let mut n = 0;
    while n < maxlen && *s.add(n) != 0 {
        n += 1;
    }
    n
}

// ── #[no_mangle] entry points the C printf wrappers link against ──────
/// `int __cdecl streamout(FILE*, const char*, va_list)`.
// Unexported in the user32 build: only its own wsprintf wrappers call it, and a second
// global `streamout` would clash wherever both builds meet (cnt-difftest).
#[cfg_attr(not(streamout_user32), no_mangle)]
pub unsafe extern "C" fn streamout(stream: *mut File, format: *const u8, argptr: *mut u8) -> i32 {
    streamout_impl::<u8>(stream, format, argptr)
}

/// `int __cdecl wstreamout(FILE*, const wchar_t*, va_list)`.
// Unexported in the user32 build: only its own wsprintf wrappers call it, and a second
// global `streamout` would clash wherever both builds meet (cnt-difftest).
#[cfg_attr(not(streamout_user32), no_mangle)]
pub unsafe extern "C" fn wstreamout(stream: *mut File, format: *const u16, argptr: *mut u8) -> i32 {
    streamout_impl::<u16>(stream, format, argptr)
}

// ── format_float — the Rust float path (amd64/arm64, `float` feature) ─
// On i386 (no `float`) the C residual `printf/format_float.c` provides
// these symbols instead (the engine's extern call resolves to it).
//
// Faithful port of streamout.c::format_float. The libm primitives
// (floor/log10/pow) and is_nan/is_finite are taken from
// `core_rs::floatcvt` so this agrees bit-for-bit with the differential
// reference, which models the SAME three primitives on both sides (the
// floatcvt ecvt/gcvt discipline). `long double == double == f64` on the
// Windows ABI, so `va_arg_ffp` is a single 8-byte read.
//
// No longer `float`-gated: the engine is no-SSE-safe (extern "C" x87
// returns + va_ffp's transmute read + core_rs::floatcvt's fb64 shims),
// so it owns format_float on i386 too. (`_USER32_WSPRINTF` has no float conversions.)
#[cfg(not(streamout_user32))]
mod ff {
    use super::{slot_of, Unit, FLAG_FORCE_SIGN, FLAG_FORCE_SIGNSP, FLAG_SPECIAL};
    use crate::core_rs::floatcvt::{fl, is_finite, is_nan};

    /// Per-width sign-prefix statics (the genuine `*prefix = _T("-")`).
    pub trait FloatUnit: Unit {
        /// 1 = "-", 2 = "+", 3 = " ".
        fn sign(which: u8) -> *const Self;
    }
    impl FloatUnit for u8 {
        fn sign(which: u8) -> *const u8 {
            static M: [u8; 2] = [b'-', 0];
            static P: [u8; 2] = [b'+', 0];
            static S: [u8; 2] = [b' ', 0];
            match which {
                1 => M.as_ptr(),
                2 => P.as_ptr(),
                _ => S.as_ptr(),
            }
        }
    }
    impl FloatUnit for u16 {
        fn sign(which: u8) -> *const u16 {
            static M: [u16; 2] = [b'-' as u16, 0];
            static P: [u16; 2] = [b'+' as u16, 0];
            static S: [u16; 2] = [b' ' as u16, 0];
            match which {
                1 => M.as_ptr(),
                2 => P.as_ptr(),
                _ => S.as_ptr(),
            }
        }
    }

    /// `va_arg_ffp` — read a `double`/`long double` (both f64 on
    /// Windows) at the cursor, advance by the rounded slot.
    ///
    /// `extern "C"` (x87 ST(0) return, not xmm) + read the bits via
    /// `*const u64` then `transmute` — both the Rust-ABI `-> f64` return
    /// AND `read_unaligned::<f64>` would otherwise need SSE and hard-error
    /// on the no-SSE i386 kernel. Pure bit read; value-identical.
    #[inline]
    unsafe extern "C" fn va_ffp(cursor: *mut *mut u8, _flags: u32) -> f64 {
        let v: f64 = core::mem::transmute(core::ptr::read_unaligned(*cursor as *const u64));
        *cursor = (*cursor).add(slot_of(core::mem::size_of::<f64>()));
        v
    }

    /// Write `s` (ASCII bytes) backward-anchored at `*string`: copies
    /// `s` then a NUL, WITHOUT moving `*string` past the body — i.e.
    /// `_tcscpy(*string -= n, _lit)` as the genuine does (caller has
    /// already retracted `*string`).
    #[inline]
    unsafe fn tcscpy<T: FloatUnit>(dst: *mut T, s: &[u8]) {
        let mut i = 0;
        while i < s.len() {
            *dst.add(i) = T::from_u32(s[i] as u32);
            i += 1;
        }
        *dst.add(s.len()) = T::from_u32(0);
    }

    pub unsafe fn format_float_impl<T: FloatUnit>(
        chr: u32,
        flags: u32,
        mut precision: i32,
        string: *mut *mut T,
        prefix: *mut *const T,
        cursor: *mut *mut u8,
    ) {
        const DIGITS_L: &[u8; 18] = b"0123456789abcdef0x";
        const DIGITS_U: &[u8; 18] = b"0123456789ABCDEF0X";
        const NAN: &[u8; 5] = b"#QNAN";
        const INF: &[u8; 4] = b"#INF";
        let mut digits: &[u8; 18] = DIGITS_L;
        let base: u64 = 10;

        macro_rules! push {
            ($c:expr) => {{
                *string = (*string).sub(1);
                **string = T::from_u32($c as u32);
            }};
        }

        // Normalize precision.
        let mut padding = 0i32;
        if precision < 0 {
            precision = 6;
        } else if precision > 17 {
            padding = precision - 17;
            precision = 17;
        }

        let fpval = va_ffp(cursor, flags);
        let mut exponent = if fpval == 0.0 { 0 } else { fl::exp10_floor(fpval) };
        let sign = if fpval < 0.0 { -1.0 } else { 1.0 };

        let spec = if chr <= 0xff { chr as u8 } else { 0 };
        // 'G'/'E'/'A' use uppercase digits; 'g'/'e'/'a' lowercase.
        if spec == b'G' || spec == b'E' || spec == b'A' {
            digits = DIGITS_U;
        }

        // Pick the form. `emit_exp` = the case_e exponent path; `handled`
        // = fpval2 already computed (g f-form or case_e).
        let mut fpval2: f64 = 0.0;
        let mut emit_exp = false;
        let mut handled = false;
        if spec == b'g' || spec == b'G' {
            if precision > 0 {
                precision -= 1;
            }
            if exponent < -4 || exponent >= precision {
                emit_exp = true;
            } else {
                // g f-form: round + strip trailing zeroes.
                fpval2 = fl::round(sign * fpval * fl::pow10(precision));
                while precision != 0 && (fpval2 as u64) % 10 == 0 {
                    precision -= 1;
                    fpval2 /= 10.0;
                }
                handled = true;
            }
        } else if spec == b'e' || spec == b'E' {
            emit_exp = true;
        }

        if emit_exp {
            // case_e — round, exponent compensation, then emit the
            // 3-digit exponent + sign + 'e'/'E' separator (backward).
            fpval2 = fl::round(sign * fpval * fl::pow10(precision - exponent));
            if fpval2 >= fl::pow10(precision + 1) {
                exponent += 1;
                fpval2 = fl::round(sign * fpval * fl::pow10(precision - exponent));
            }
            let mut val32 = if exponent >= 0 { exponent } else { -exponent };
            let mut num_digits = 3;
            while num_digits > 0 {
                push!(digits[(val32 % 10) as usize]);
                val32 /= 10;
                num_digits -= 1;
            }
            push!(if exponent >= 0 { b'+' } else { b'-' });
            push!(digits[0xe]);
            handled = true;
        }

        if !handled {
            // 'f' / 'a' / 'A' / default
            fpval2 = fl::round(sign * fpval * fl::pow10(precision));
        }

        // Handle sign prefix.
        if fpval < 0.0 {
            *prefix = T::sign(1);
        } else if flags & FLAG_FORCE_SIGN != 0 {
            *prefix = T::sign(2);
        } else if flags & FLAG_FORCE_SIGNSP != 0 {
            *prefix = T::sign(3);
        }

        // Special cases.
        if is_nan(fpval) {
            *string = (*string).sub(NAN.len());
            tcscpy::<T>(*string, NAN);
            fpval2 = 1.0;
        } else if !is_finite(fpval) {
            *string = (*string).sub(INF.len());
            tcscpy::<T>(*string, INF);
            fpval2 = 1.0;
        } else {
            while padding > 0 {
                push!(b'0');
                padding -= 1;
            }
            let mut num_digits = precision;
            while num_digits > 0 {
                push!(digits[((fpval2 as u64) % 10) as usize]);
                fpval2 /= base as f64;
                num_digits -= 1;
            }
        }

        if precision > 0 || flags & FLAG_SPECIAL != 0 {
            push!(b'.');
        }

        // Digits before the decimal point.
        loop {
            push!(digits[((fpval2 as u64) % base) as usize]);
            fpval2 /= base as f64;
            if (fpval2 as u64) == 0 {
                break;
            }
        }
    }
}

/// `void format_float(char, unsigned, int, char**, const char**,
/// va_list*)` — all 3 arches (Rust, no-SSE-safe on i386).
#[cfg(not(streamout_user32))]
#[no_mangle]
pub unsafe extern "C" fn format_float(
    chr: u8,
    flags: u32,
    precision: i32,
    string: *mut *mut u8,
    prefix: *mut *const u8,
    argptr: *mut *mut u8,
) {
    ff::format_float_impl::<u8>(chr as u32, flags, precision, string, prefix, argptr);
}

/// Wide twin `format_floatw`.
#[cfg(not(streamout_user32))]
#[no_mangle]
pub unsafe extern "C" fn format_floatw(
    chr: u16,
    flags: u32,
    precision: i32,
    string: *mut *mut u16,
    prefix: *mut *const u16,
    argptr: *mut *mut u8,
) {
    ff::format_float_impl::<u16>(chr as u32, flags, precision, string, prefix, argptr);
}
