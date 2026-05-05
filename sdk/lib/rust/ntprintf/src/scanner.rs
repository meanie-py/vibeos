//! Format-string scanner. Walks `%`-specifiers and emits a typed tag per arg.
//!
//! Tag bytes match the FFI contract in `lib.rs::ntprintf_scan_arg_types`.
//! Width-from-arg (`*`) and precision-from-arg (`.*`) emit a `*` tag *before*
//! the value tag — the C trampoline pulls them in this order.

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
    /// Native pointer width (`%p %s %S %wZ`).
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

/// Walk `fmt`, calling `emit(tag)` once per implied argument slot. Returns
/// `Err` if `emit` ever returns false (caller's buffer overflowed) or if a
/// truly malformed specifier is found.
pub fn scan_format<F: FnMut(ArgType) -> bool>(fmt: &[u8], mut emit: F) -> Result<(), ()> {
    let mut p = 0usize;
    while p < fmt.len() {
        if fmt[p] != b'%' {
            p += 1;
            continue;
        }
        p += 1;
        if p >= fmt.len() {
            return Ok(());
        }
        // Parse one specifier.
        if let Some(next_p) = scan_one(fmt, p, &mut emit)? {
            p = next_p;
        } else {
            return Err(());
        }
    }
    Ok(())
}

/// Scan a single `%…` specifier starting at `start` (just past the `%`).
/// Returns `Ok(Some(end))` on success, with `end` pointing past the conversion
/// character. Returns `Ok(None)` if the caller's emit closure refused.
fn scan_one<F: FnMut(ArgType) -> bool>(
    fmt: &[u8],
    start: usize,
    emit: &mut F,
) -> Result<Option<usize>, ()> {
    let mut p = start;

    // Skip flags: -, +, space, #, 0
    while p < fmt.len() && matches!(fmt[p], b'-' | b'+' | b' ' | b'#' | b'0') {
        p += 1;
    }
    // Width (digits or *)
    if p < fmt.len() && fmt[p] == b'*' {
        if !emit(ArgType::Star) {
            return Ok(None);
        }
        p += 1;
    } else {
        while p < fmt.len() && fmt[p].is_ascii_digit() {
            p += 1;
        }
    }
    // .precision (digits or .*)
    if p < fmt.len() && fmt[p] == b'.' {
        p += 1;
        if p < fmt.len() && fmt[p] == b'*' {
            if !emit(ArgType::Star) {
                return Ok(None);
            }
            p += 1;
        } else {
            while p < fmt.len() && fmt[p].is_ascii_digit() {
                p += 1;
            }
        }
    }
    // Length modifier: hh, h, ll, l, L, q, j, z, t, I64, I32, I, w
    let mut is64 = false;
    let mut is_wide_str = false;
    while p < fmt.len() {
        match fmt[p] {
            b'h' => {
                p += 1;
                if p < fmt.len() && fmt[p] == b'h' { p += 1; }
            }
            b'l' => {
                p += 1;
                if p < fmt.len() && fmt[p] == b'l' {
                    p += 1;
                    is64 = true;
                }
            }
            b'L' | b'q' | b'j' | b'z' | b't' => {
                p += 1;
            }
            b'I' => {
                // I64, I32, or bare I (size_t-ish — treat as native pointer width)
                p += 1;
                if p + 1 < fmt.len() && fmt[p] == b'6' && fmt[p+1] == b'4' {
                    p += 2;
                    is64 = true;
                } else if p + 1 < fmt.len() && fmt[p] == b'3' && fmt[p+1] == b'2' {
                    p += 2;
                }
                // bare I — no extra advance (size_t)
            }
            b'w' => {
                p += 1;
                is_wide_str = true;
            }
            _ => break,
        }
    }
    // Conversion character.
    if p >= fmt.len() {
        return Err(());
    }
    let conv = fmt[p];
    p += 1;

    let tag = match conv {
        b'%' => return Ok(Some(p)),  // %% — no arg
        b'd' | b'i' => if is64 { ArgType::I64 } else { ArgType::I32 },
        b'u' | b'o' | b'x' | b'X' => if is64 { ArgType::U64 } else { ArgType::U32 },
        b'c' | b'C' => ArgType::U32,  // promoted via va_arg(int)
        b's' | b'S' | b'p' => ArgType::Ptr,
        b'Z' if is_wide_str => ArgType::Ptr,  // %wZ — UNICODE_STRING*
        b'Z' => ArgType::Ptr,                 // %Z — STRING*  (rare but exists)
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

    fn collect(fmt: &[u8]) -> Vec<ArgType> {
        let mut out = Vec::new();
        scan_format(fmt, |t| { out.push(t); true }).unwrap();
        out
    }

    #[test]
    fn simple_int() {
        assert_eq!(collect(b"%d"), vec![ArgType::I32]);
        assert_eq!(collect(b"%u"), vec![ArgType::U32]);
        assert_eq!(collect(b"%x"), vec![ArgType::U32]);
        assert_eq!(collect(b"%i"), vec![ArgType::I32]);
        assert_eq!(collect(b"%o"), vec![ArgType::U32]);
    }

    #[test]
    fn long_modifiers_are_still_32bit() {
        // Windows LLP64: long is 32-bit
        assert_eq!(collect(b"%ld"), vec![ArgType::I32]);
        assert_eq!(collect(b"%lu"), vec![ArgType::U32]);
        assert_eq!(collect(b"%lx"), vec![ArgType::U32]);
        assert_eq!(collect(b"%lX"), vec![ArgType::U32]);
        assert_eq!(collect(b"%hd"), vec![ArgType::I32]);
    }

    #[test]
    fn long_long_and_i64() {
        assert_eq!(collect(b"%lld"), vec![ArgType::I64]);
        assert_eq!(collect(b"%llu"), vec![ArgType::U64]);
        assert_eq!(collect(b"%I64d"), vec![ArgType::I64]);
        assert_eq!(collect(b"%I64u"), vec![ArgType::U64]);
        assert_eq!(collect(b"%I64x"), vec![ArgType::U64]);
        assert_eq!(collect(b"%I64X"), vec![ArgType::U64]);
    }

    #[test]
    fn pointers_and_strings() {
        assert_eq!(collect(b"%p"), vec![ArgType::Ptr]);
        assert_eq!(collect(b"%s"), vec![ArgType::Ptr]);
        assert_eq!(collect(b"%S"), vec![ArgType::Ptr]);
        assert_eq!(collect(b"%wZ"), vec![ArgType::Ptr]);
    }

    #[test]
    fn char() {
        assert_eq!(collect(b"%c"), vec![ArgType::U32]);
    }

    #[test]
    fn flags_and_width() {
        assert_eq!(collect(b"%08x"), vec![ArgType::U32]);
        assert_eq!(collect(b"%-10s"), vec![ArgType::Ptr]);
        assert_eq!(collect(b"%04ld"), vec![ArgType::I32]);
        assert_eq!(collect(b"%+d"), vec![ArgType::I32]);
        assert_eq!(collect(b"%#016I64x"), vec![ArgType::U64]);
    }

    #[test]
    fn star_width_emits_extra_arg() {
        // %*s — first arg is width (i32), second is the string pointer
        assert_eq!(collect(b"%*s"), vec![ArgType::Star, ArgType::Ptr]);
        // %.*s — first arg is precision (i32), second is the string pointer
        assert_eq!(collect(b"%.*s"), vec![ArgType::Star, ArgType::Ptr]);
        // %-.*s
        assert_eq!(collect(b"%-.*s"), vec![ArgType::Star, ArgType::Ptr]);
    }

    #[test]
    fn precision() {
        assert_eq!(collect(b"%.4s"), vec![ArgType::Ptr]);
        assert_eq!(collect(b"%.10d"), vec![ArgType::I32]);
    }

    #[test]
    fn percent_literal_consumes_no_arg() {
        assert_eq!(collect(b"100%%"), Vec::<ArgType>::new());
        assert_eq!(collect(b"%d%%"), vec![ArgType::I32]);
    }

    #[test]
    fn complex_real_world() {
        // Picked from the inventory:
        //   "%p Thread %p current stack (%x, %x) belongs to %wZ\n"
        assert_eq!(
            collect(b"%p Thread %p current stack (%x, %x) belongs to %wZ\n"),
            vec![ArgType::Ptr, ArgType::Ptr, ArgType::U32, ArgType::U32, ArgType::Ptr]
        );
        //   "0x%016I64x  %20I64u  %20I64d\n"
        assert_eq!(
            collect(b"0x%016I64x  %20I64u  %20I64d\n"),
            vec![ArgType::U64, ArgType::U64, ArgType::I64]
        );
        //   "Handle table at %p with %d entries in use\n"
        assert_eq!(
            collect(b"Handle table at %p with %d entries in use\n"),
            vec![ArgType::Ptr, ArgType::I32]
        );
    }

    #[test]
    fn rejects_percent_n() {
        let result = scan_format(b"%n", |_| true);
        assert!(result.is_err());
    }
}
