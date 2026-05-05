//! Format-string scanner for wide (UTF-16) format strings. Walks
//! `%`-specifiers and emits a typed tag per arg.
//!
//! Tag bytes match the FFI contract in `lib.rs::ntprintf_w_scan_arg_types` —
//! same byte values as the ANSI scanner, so the C trampoline reuses one
//! `va_arg` pull loop for both kinds of format strings.
//!
//! Format-spec characters are themselves ASCII even when the surrounding
//! string is UTF-16, so this scanner casts each `u16` to `u8` for byte
//! comparisons. Codepoints above 0x7F are treated as ordinary literal text.

/// Argument-type discriminator. One byte per slot — kept narrow so the C side
/// can switch on it cheaply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    /// Signed 32-bit (`%d %i %ld %li %hd %hi`).
    I32,
    /// Unsigned 32-bit (`%u %x %X %o %lu %lx %lX %lo %hu %hx %c %C`).
    U32,
    /// Signed 64-bit (`%lld %lli %I64d`).
    I64,
    /// Unsigned 64-bit (`%llu %llx %I64u %I64x`).
    U64,
    /// Native pointer width (`%p %s %S %wZ`). Slot type is identical
    /// regardless of W vs A semantics — the formatter decides whether to
    /// dereference as PCWSTR or PCSTR based on the conversion + length.
    Ptr,
    /// Width-or-precision-from-arg (`*` or `.*` modifier). Always 32-bit.
    Star,
}

impl ArgType {
    /// One-byte tag matching the FFI contract.
    pub fn tag(self) -> u8 {
        match self {
            ArgType::I32 => b'd',
            ArgType::U32 => b'u',
            ArgType::I64 => b'D',
            ArgType::U64 => b'U',
            ArgType::Ptr => b'p',
            ArgType::Star => b'*',
        }
    }
}

/// Read one ASCII byte from the wide format string, or `None` if the codepoint
/// at `p` isn't ASCII (which means it can't be part of a `%`-specifier
/// machinery character — flags, digits, length modifier, conversion).
#[inline]
fn ascii_byte(fmt: &[u16], p: usize) -> Option<u8> {
    let c = fmt[p];
    if c <= 0x7F { Some(c as u8) } else { None }
}

/// Walk `fmt` (UTF-16), calling `emit(tag)` once per implied argument slot.
/// Returns `Err` if `emit` ever returns false (caller's buffer overflowed) or
/// if a truly malformed specifier is found.
pub fn scan_format<F: FnMut(ArgType) -> bool>(fmt: &[u16], mut emit: F) -> Result<(), ()> {
    let mut p = 0usize;
    while p < fmt.len() {
        if fmt[p] != b'%' as u16 {
            p += 1;
            continue;
        }
        p += 1;
        if p >= fmt.len() {
            return Ok(());
        }
        if let Some(next_p) = scan_one(fmt, p, &mut emit)? {
            p = next_p;
        } else {
            return Err(());
        }
    }
    Ok(())
}

/// Scan a single `%…` specifier starting at `start` (just past the `%`).
fn scan_one<F: FnMut(ArgType) -> bool>(
    fmt: &[u16],
    start: usize,
    emit: &mut F,
) -> Result<Option<usize>, ()> {
    let mut p = start;

    // Skip flags: -, +, space, #, 0
    while p < fmt.len() {
        match ascii_byte(fmt, p) {
            Some(b'-') | Some(b'+') | Some(b' ') | Some(b'#') | Some(b'0') => p += 1,
            _ => break,
        }
    }
    // Width (digits or *)
    if p < fmt.len() && ascii_byte(fmt, p) == Some(b'*') {
        if !emit(ArgType::Star) {
            return Ok(None);
        }
        p += 1;
    } else {
        while p < fmt.len() && ascii_byte(fmt, p).map_or(false, |b| b.is_ascii_digit()) {
            p += 1;
        }
    }
    // .precision (digits or .*)
    if p < fmt.len() && ascii_byte(fmt, p) == Some(b'.') {
        p += 1;
        if p < fmt.len() && ascii_byte(fmt, p) == Some(b'*') {
            if !emit(ArgType::Star) {
                return Ok(None);
            }
            p += 1;
        } else {
            while p < fmt.len() && ascii_byte(fmt, p).map_or(false, |b| b.is_ascii_digit()) {
                p += 1;
            }
        }
    }
    // Length modifier: hh, h, ll, l, L, q, j, z, t, I64, I32, I, w
    let mut is64 = false;
    let mut is_wide_str = false;     // true after `w` modifier (e.g. %wZ)
    let mut is_long = false;         // true after `l` modifier (affects %ls/%lc)
    let mut is_short = false;        // true after `h` modifier (affects %hs/%hc)
    while p < fmt.len() {
        match ascii_byte(fmt, p) {
            Some(b'h') => {
                p += 1;
                is_short = true;
                if p < fmt.len() && ascii_byte(fmt, p) == Some(b'h') { p += 1; }
            }
            Some(b'l') => {
                p += 1;
                is_long = true;
                if p < fmt.len() && ascii_byte(fmt, p) == Some(b'l') {
                    p += 1;
                    is64 = true;
                }
            }
            Some(b'L') | Some(b'q') | Some(b'j') | Some(b'z') | Some(b't') => {
                p += 1;
            }
            Some(b'I') => {
                p += 1;
                if p + 1 < fmt.len()
                    && ascii_byte(fmt, p) == Some(b'6')
                    && ascii_byte(fmt, p + 1) == Some(b'4')
                {
                    p += 2;
                    is64 = true;
                } else if p + 1 < fmt.len()
                    && ascii_byte(fmt, p) == Some(b'3')
                    && ascii_byte(fmt, p + 1) == Some(b'2')
                {
                    p += 2;
                }
            }
            Some(b'w') => {
                p += 1;
                is_wide_str = true;
            }
            _ => break,
        }
    }
    let _ = is_long;
    let _ = is_short;

    // Conversion character.
    if p >= fmt.len() {
        return Err(());
    }
    let conv = match ascii_byte(fmt, p) {
        Some(b) => b,
        None => return Err(()),  // non-ASCII conversion char — malformed
    };
    p += 1;

    let tag = match conv {
        b'%' => return Ok(Some(p)),  // %% — no arg
        b'd' | b'i' => if is64 { ArgType::I64 } else { ArgType::I32 },
        b'u' | b'o' | b'x' | b'X' => if is64 { ArgType::U64 } else { ArgType::U32 },
        b'c' | b'C' => ArgType::U32,        // promoted via va_arg(int)
        b's' | b'S' | b'p' => ArgType::Ptr,
        b'Z' if is_wide_str => ArgType::Ptr,  // %wZ — UNICODE_STRING*
        b'Z' => ArgType::Ptr,                 // %Z — STRING*
        b'n' => return Err(()),  // %n is a security footgun; reject.
        _ => return Err(()),
    };

    if !emit(tag) {
        return Ok(None);
    }
    Ok(Some(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convenience: widen an ASCII byte string into a UTF-16 vec for tests.
    fn w(s: &[u8]) -> Vec<u16> {
        s.iter().map(|&b| b as u16).collect()
    }

    fn collect(s: &[u8]) -> Vec<ArgType> {
        let fmt = w(s);
        let mut out = Vec::new();
        scan_format(&fmt, |t| { out.push(t); true }).unwrap();
        out
    }

    #[test]
    fn simple_int() {
        assert_eq!(collect(b"%d"), vec![ArgType::I32]);
        assert_eq!(collect(b"%u"), vec![ArgType::U32]);
        assert_eq!(collect(b"%x"), vec![ArgType::U32]);
        assert_eq!(collect(b"%lu"), vec![ArgType::U32]);
        assert_eq!(collect(b"%I64x"), vec![ArgType::U64]);
    }

    #[test]
    fn pointers_and_strings() {
        assert_eq!(collect(b"%p"), vec![ArgType::Ptr]);
        assert_eq!(collect(b"%s"), vec![ArgType::Ptr]);
        assert_eq!(collect(b"%S"), vec![ArgType::Ptr]);
        assert_eq!(collect(b"%hs"), vec![ArgType::Ptr]);
        assert_eq!(collect(b"%ls"), vec![ArgType::Ptr]);
        assert_eq!(collect(b"%wZ"), vec![ArgType::Ptr]);
    }

    #[test]
    fn real_world_wide() {
        // From config/cmsysini.rs: w!("\\Registry\\Machine\\System\\ControlSet%03ld")
        assert_eq!(collect(b"\\Registry\\Machine\\System\\ControlSet%03ld"),
                   vec![ArgType::I32]);
        // From fstub/disksup.rs: W_HARDDISK_PARTITION_FMT = "\\Device\\Harddisk%d\\Partition%d"
        assert_eq!(collect(b"\\Device\\Harddisk%d\\Partition%d"),
                   vec![ArgType::I32, ArgType::I32]);
    }

    #[test]
    fn star_width() {
        assert_eq!(collect(b"%*s"), vec![ArgType::Star, ArgType::Ptr]);
        assert_eq!(collect(b"%.*s"), vec![ArgType::Star, ArgType::Ptr]);
    }

    #[test]
    fn percent_literal() {
        assert_eq!(collect(b"100%%"), Vec::<ArgType>::new());
    }

    #[test]
    fn rejects_percent_n() {
        let fmt = w(b"%n");
        assert!(scan_format(&fmt, |_| true).is_err());
    }
}
