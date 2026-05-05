//! Validation harness: for each corpus entry, run libc `vsnprintf` and our
//! `ntprintf::format_into` with the same args, byte-diff.
//!
//! Corpus is seeded from the actual format strings observed across `ntoskrnl`
//! Rust + C call sites (collected during the variadic-shim inventory pass).
//! Add new entries here whenever a new specifier shape appears.
//!
//! The harness manually dispatches per arg-count because `vsnprintf` needs
//! real C args, not a slot array. For up to 6 args (covers every fmt in the
//! kernel), we hand-roll the dispatch.

use std::ffi::CString;

use ntprintf::{format_into, Writer};

struct VecBuf(Vec<u8>);
impl Writer for VecBuf {
    fn write_bytes(&mut self, bytes: &[u8]) { self.0.extend_from_slice(bytes); }
    fn written(&self) -> usize { self.0.len() }
}

/// Rewrite a Windows-flavored fmt string to its libc-equivalent shape so the
/// SysV-on-Linux libc snprintf produces what Windows printf would produce:
///   - `%ld %lu %lx %lX %lo %li` → drop the `l` (Win long == 32-bit, Linux
///     long == 64-bit; without rewrite libc would read 8 bytes off the stack)
///   - `%I64x` etc → `%llx` etc (libc doesn't understand `I64`)
///   - `%p` → fixed-width zero-padded hex matching native pointer width
///   - `%wZ` → libc has no UNICODE_STRING; tests using `%wZ` skip libc and
///     compare against a hand-written expectation instead.
fn rewrite_fmt_for_libc(fmt: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(fmt.len() + 16);
    let mut i = 0;
    while i < fmt.len() {
        if fmt[i] != b'%' {
            out.push(fmt[i]);
            i += 1;
            continue;
        }
        // Copy '%' + flags/width/precision verbatim.
        out.push(b'%');
        i += 1;
        while i < fmt.len() && matches!(fmt[i],
            b'-' | b'+' | b' ' | b'#' | b'0' | b'.' | b'*' | b'1'..=b'9') {
            out.push(fmt[i]);
            i += 1;
        }
        // Length modifier rewrite.
        if i + 2 < fmt.len() && fmt[i] == b'I' && fmt[i+1] == b'6' && fmt[i+2] == b'4' {
            out.extend_from_slice(b"ll");
            i += 3;
        } else if i + 1 < fmt.len() && fmt[i] == b'I' && fmt[i+1] == b'3' && (fmt[i+1] == b'2') {
            // %I32 — drop entirely (32-bit on both)
            i += 2;
        } else if i < fmt.len() && fmt[i] == b'l' {
            // Drop single `l` (Win long is 32-bit). Keep `ll` as-is.
            if i + 1 < fmt.len() && fmt[i+1] == b'l' {
                out.extend_from_slice(b"ll");
                i += 2;
            } else {
                i += 1;  // skip the lone `l`
            }
        } else if i < fmt.len() && fmt[i] == b'h' {
            // Keep `h`/`hh` — both are integer-promoted to int on libc.
            out.push(fmt[i]);
            i += 1;
            if i < fmt.len() && fmt[i] == b'h' {
                out.push(fmt[i]);
                i += 1;
            }
        }
        // Conversion character.
        if i < fmt.len() {
            // Special: %p — replace with %0Nx
            if fmt[i] == b'p' {
                #[cfg(target_pointer_width = "64")]
                out.extend_from_slice(b"016lx");
                #[cfg(target_pointer_width = "32")]
                out.extend_from_slice(b"08x");
                i += 1;
            } else {
                out.push(fmt[i]);
                i += 1;
            }
        }
    }
    out
}

/// Render via libc vsnprintf. Caller must build the C va list correctly.
/// We use macros + fixed dispatch because Rust can't variadic-call C on
/// stable.
fn libc_render(fmt: &[u8], args: &[Arg]) -> String {
    let rewritten = rewrite_fmt_for_libc(fmt);
    let cfmt = CString::new(rewritten).unwrap();
    let mut buf = vec![0u8; 4096];

    // Hand-dispatch up to 6 args by type. This is tedious but covers every
    // format string in our kernel corpus (max args observed = 5).
    let n = unsafe {
        match args {
            [] => libc::snprintf(buf.as_mut_ptr() as *mut _, buf.len(),
                                  cfmt.as_ptr()),
            [a] => render1(&mut buf, &cfmt, a),
            [a, b] => render2(&mut buf, &cfmt, a, b),
            [a, b, c] => render3(&mut buf, &cfmt, a, b, c),
            [a, b, c, d] => render4(&mut buf, &cfmt, a, b, c, d),
            [a, b, c, d, e] => render5(&mut buf, &cfmt, a, b, c, d, e),
            [a, b, c, d, e, f] => render6(&mut buf, &cfmt, a, b, c, d, e, f),
            [a, b, c, d, e, f, g] => render7(&mut buf, &cfmt, a, b, c, d, e, f, g),
            [a, b, c, d, e, f, g, h] => render8(&mut buf, &cfmt, a, b, c, d, e, f, g, h),
            _ => panic!("more than 8 args not supported in test dispatcher"),
        }
    };
    if n < 0 { return "<vsnprintf-error>".into(); }
    let n = n as usize;
    buf.truncate(n.min(buf.len()));
    String::from_utf8_lossy(&buf).into_owned()
}

/// Test-arg union. Each variant maps to a libc va_arg type + an ntprintf slot.
#[derive(Debug, Clone, Copy)]
enum Arg {
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    Ptr(usize),         // %p / %s — caller responsible for keeping the pointee alive
}

impl Arg {
    fn slot(self) -> u64 {
        match self {
            Arg::I32(v) => v as u32 as u64,
            Arg::U32(v) => v as u64,
            Arg::I64(v) => v as u64,
            Arg::U64(v) => v,
            Arg::Ptr(v) => v as u64,
        }
    }
}

// snprintf takes args by C calling convention. We're on a 64-bit Linux host,
// so all int/long/ptr args go in registers as 8-byte slots. libc::snprintf
// signature is variadic, so we just spell out each combo.

unsafe fn render1(buf: &mut [u8], fmt: &CString, a: &Arg) -> i32 {
    use libc::{c_int, c_uint, c_longlong, c_ulonglong};
    let p = buf.as_mut_ptr() as *mut _;
    let l = buf.len();
    let f = fmt.as_ptr();
    match a {
        Arg::I32(v) => libc::snprintf(p, l, f, *v as c_int),
        Arg::U32(v) => libc::snprintf(p, l, f, *v as c_uint),
        Arg::I64(v) => libc::snprintf(p, l, f, *v as c_longlong),
        Arg::U64(v) => libc::snprintf(p, l, f, *v as c_ulonglong),
        Arg::Ptr(v) => libc::snprintf(p, l, f, *v as *const u8),
    }
}

macro_rules! disp_arg {
    ($a:expr) => {
        match $a {
            Arg::I32(v) => *v as libc::c_int as libc::c_longlong,
            Arg::U32(v) => *v as libc::c_uint as libc::c_longlong,
            Arg::I64(v) => *v as libc::c_longlong,
            Arg::U64(v) => *v as libc::c_ulonglong as libc::c_longlong,
            Arg::Ptr(v) => *v as libc::c_longlong,
        }
    }
}

// On 64-bit Linux SysV ABI, integer/pointer varargs all pass through %rdi/rsi/
// rdx/rcx/r8/r9 as 8-byte slots — same width regardless of the source type
// once promoted. So passing them as c_longlong roundtrips cleanly *for fmt
// strings that use %d/%u/%x/%lld/%I64x/%p* without size mismatches in the
// register file. The format string itself drives the va_arg width.

unsafe fn render2(buf: &mut [u8], fmt: &CString, a: &Arg, b: &Arg) -> i32 {
    libc::snprintf(buf.as_mut_ptr() as *mut _, buf.len(), fmt.as_ptr(),
        disp_arg!(a), disp_arg!(b))
}
unsafe fn render3(buf: &mut [u8], fmt: &CString, a: &Arg, b: &Arg, c: &Arg) -> i32 {
    libc::snprintf(buf.as_mut_ptr() as *mut _, buf.len(), fmt.as_ptr(),
        disp_arg!(a), disp_arg!(b), disp_arg!(c))
}
unsafe fn render4(buf: &mut [u8], fmt: &CString, a: &Arg, b: &Arg, c: &Arg, d: &Arg) -> i32 {
    libc::snprintf(buf.as_mut_ptr() as *mut _, buf.len(), fmt.as_ptr(),
        disp_arg!(a), disp_arg!(b), disp_arg!(c), disp_arg!(d))
}
unsafe fn render5(buf: &mut [u8], fmt: &CString, a: &Arg, b: &Arg, c: &Arg, d: &Arg, e: &Arg) -> i32 {
    libc::snprintf(buf.as_mut_ptr() as *mut _, buf.len(), fmt.as_ptr(),
        disp_arg!(a), disp_arg!(b), disp_arg!(c), disp_arg!(d), disp_arg!(e))
}
unsafe fn render6(buf: &mut [u8], fmt: &CString, a: &Arg, b: &Arg, c: &Arg, d: &Arg, e: &Arg, f: &Arg) -> i32 {
    libc::snprintf(buf.as_mut_ptr() as *mut _, buf.len(), fmt.as_ptr(),
        disp_arg!(a), disp_arg!(b), disp_arg!(c), disp_arg!(d), disp_arg!(e), disp_arg!(f))
}
unsafe fn render7(buf: &mut [u8], fmt: &CString, a: &Arg, b: &Arg, c: &Arg, d: &Arg, e: &Arg, f: &Arg, g: &Arg) -> i32 {
    libc::snprintf(buf.as_mut_ptr() as *mut _, buf.len(), fmt.as_ptr(),
        disp_arg!(a), disp_arg!(b), disp_arg!(c), disp_arg!(d), disp_arg!(e), disp_arg!(f), disp_arg!(g))
}
unsafe fn render8(buf: &mut [u8], fmt: &CString, a: &Arg, b: &Arg, c: &Arg, d: &Arg, e: &Arg, f: &Arg, g: &Arg, h: &Arg) -> i32 {
    libc::snprintf(buf.as_mut_ptr() as *mut _, buf.len(), fmt.as_ptr(),
        disp_arg!(a), disp_arg!(b), disp_arg!(c), disp_arg!(d), disp_arg!(e), disp_arg!(f), disp_arg!(g), disp_arg!(h))
}

/// Render via our Rust impl.
fn rust_render(fmt: &[u8], args: &[Arg]) -> String {
    let slots: Vec<u64> = args.iter().map(|a| a.slot()).collect();
    let mut sink = VecBuf(Vec::new());
    format_into(&mut sink, fmt, &slots);
    String::from_utf8_lossy(&sink.0).into_owned()
}

/// Diff helper. Panics on mismatch with both renders + the input.
fn diff_one(fmt_with_nul: &[u8], args: &[Arg], label: &str) {
    // Strip trailing NUL for the libc call (CString adds its own).
    let fmt = if let Some(&0) = fmt_with_nul.last() {
        &fmt_with_nul[..fmt_with_nul.len() - 1]
    } else {
        fmt_with_nul
    };
    let libc = libc_render(fmt, args);
    let rust = rust_render(fmt, args);
    if libc != rust {
        panic!(
            "[{}] fmt={:?} args={:?}\n  libc:  {:?}\n  rust:  {:?}",
            label, std::str::from_utf8(fmt).unwrap_or("<non-utf8>"),
            args, libc, rust
        );
    }
}

// =============================================================================
// CORPUS — every entry must produce byte-identical output across libc + rust.
//
// Categories (mirror the inventory's specifier histogram):
//   1. Plain integers (d/i/u/x/X/o, with l/h modifiers — all 32-bit on Win)
//   2. 64-bit integers (lld/llu/llx/I64d/I64u/I64x — also 32-bit-host-promoted)
//   3. Width + flag combinations (-, 0, +, ' ', width, .precision)
//   4. Strings (%s, %.Ns, %-Ns, %*s, %.*s)
//   5. Pointers (%p)
//   6. Mixed real-world fmt strings from the call-site inventory
// =============================================================================

#[test]
fn cat1_plain_integers() {
    let cases: &[(&[u8], &[Arg])] = &[
        (b"%d", &[Arg::I32(0)]),
        (b"%d", &[Arg::I32(42)]),
        (b"%d", &[Arg::I32(-1)]),
        (b"%d", &[Arg::I32(i32::MIN)]),
        (b"%d", &[Arg::I32(i32::MAX)]),
        (b"%i", &[Arg::I32(-12345)]),
        (b"%u", &[Arg::U32(0)]),
        (b"%u", &[Arg::U32(u32::MAX)]),
        (b"%x", &[Arg::U32(0xdeadbeef)]),
        (b"%X", &[Arg::U32(0xdeadbeef)]),
        (b"%o", &[Arg::U32(0o755)]),
        (b"%lu", &[Arg::U32(0xfeedface)]),
        (b"%lx", &[Arg::U32(0xfeedface)]),
        (b"%ld", &[Arg::I32(-9999)]),
    ];
    for (fmt, args) in cases {
        diff_one(fmt, args, "plain_integers");
    }
}

#[test]
fn cat2_64bit_integers() {
    let cases: &[(&[u8], &[Arg])] = &[
        (b"%lld", &[Arg::I64(0)]),
        (b"%lld", &[Arg::I64(0x1_0000_0001)]),
        (b"%lld", &[Arg::I64(-1)]),
        (b"%lld", &[Arg::I64(i64::MIN)]),
        (b"%llu", &[Arg::U64(0)]),
        (b"%llu", &[Arg::U64(u64::MAX)]),
        (b"%llx", &[Arg::U64(0x12345678abcdef00)]),
        (b"%llX", &[Arg::U64(0x12345678abcdef00)]),
    ];
    for (fmt, args) in cases {
        diff_one(fmt, args, "64bit");
    }
}

// %I64* is Microsoft-specific; libc on Linux doesn't grok it directly, but
// rewrite_fmt_for_libc translates `I64` → `ll` so the same diff_one path works.
#[test]
fn cat2b_ms_i64_specifiers() {
    let cases: &[(&[u8], &[Arg])] = &[
        (b"%I64d", &[Arg::I64(0x1_0000_0001)]),
        (b"%I64u", &[Arg::U64(0x1_0000_0001)]),
        (b"%I64x", &[Arg::U64(0x12345678abcdef00)]),
        (b"%I64X", &[Arg::U64(0xfeedfacedeadbeef)]),
        (b"%016I64x", &[Arg::U64(0x42)]),
        (b"%20I64u", &[Arg::U64(99999)]),
        (b"%20I64d", &[Arg::I64(-12345)]),
        (b"%#I64x", &[Arg::U64(0xab)]),
        (b"%#016I64x", &[Arg::U64(0xab)]),
    ];
    for (fmt, args) in cases {
        diff_one(fmt, args, "ms_i64");
    }
}

#[test]
fn cat3_width_flag_precision() {
    let cases: &[(&[u8], &[Arg])] = &[
        (b"%5d", &[Arg::I32(42)]),
        (b"%-5d", &[Arg::I32(42)]),
        (b"%05d", &[Arg::I32(42)]),
        (b"%+5d", &[Arg::I32(42)]),
        (b"% 5d", &[Arg::I32(42)]),
        (b"%5.3d", &[Arg::I32(42)]),
        (b"%.5d", &[Arg::I32(42)]),
        (b"%.0d", &[Arg::I32(0)]),
        (b"%08x", &[Arg::U32(0xab)]),
        (b"%08lx", &[Arg::U32(0xab)]),
        (b"%04x", &[Arg::U32(0)]),
        (b"%02d", &[Arg::I32(7)]),
        (b"%03d", &[Arg::I32(7)]),
        (b"%4d", &[Arg::I32(7)]),
        (b"%10lu", &[Arg::U32(0xff)]),
        (b"%#x", &[Arg::U32(0)]),         // alt + zero — no prefix
        (b"%#x", &[Arg::U32(0x10)]),
        (b"%#08x", &[Arg::U32(0x10)]),
    ];
    for (fmt, args) in cases {
        diff_one(fmt, args, "width_flags");
    }
}

#[test]
fn cat4_strings() {
    let s1 = CString::new("Hello").unwrap();
    let s2 = CString::new("WindowsKernelDriverObject").unwrap();
    let p1 = Arg::Ptr(s1.as_ptr() as usize);
    let p2 = Arg::Ptr(s2.as_ptr() as usize);

    let cases: &[(&[u8], &[Arg])] = &[
        (b"%s", &[p1]),
        (b"%-10s|", &[p1]),
        (b"%10s|", &[p1]),
        (b"%-20s|", &[p1]),
        (b"%.3s", &[p1]),
        (b"%.4s", &[p2]),
        (b"%-5s|", &[p1]),
        (b"%-8s|", &[p1]),
        (b"%-11s|", &[p1]),
        (b"%-.*s", &[Arg::I32(8), p2]),
        (b"%.*s", &[Arg::I32(4), p2]),
        (b"%*s|", &[Arg::I32(10), p1]),
    ];
    for (fmt, args) in cases {
        diff_one(fmt, args, "strings");
    }
}

#[test]
fn cat5_pointer() {
    // %p — rewrite_fmt_for_libc translates to %016lx (or %08x on 32-bit hosts),
    // which gives Windows-style fixed-width zero-padded output.
    let cases: &[(&[u8], &[Arg])] = &[
        (b"%p", &[Arg::Ptr(0xdeadbeef)]),
        (b"%p", &[Arg::Ptr(0)]),
        (b"at %p\n", &[Arg::Ptr(0x12345)]),
    ];
    for (fmt, args) in cases {
        diff_one(fmt, args, "pointer");
    }
}

#[test]
fn cat6_mixed_real_world() {
    let s = CString::new("ACPI").unwrap();
    let sp = Arg::Ptr(s.as_ptr() as usize);

    let cases: &[(&[u8], &[Arg])] = &[
        (b"Adding boot driver: '%s'\n", &[sp]),
        (b"%s%sSystem32\\Drivers\\%s.sys%s\r\n",
            &[sp, sp, sp, sp]),
        (b"Handle table at %p with %d entries in use\n",
            &[Arg::Ptr(0xdeadbeef), Arg::I32(42)]),
        (b"%p Thread %p current stack (%x, %x) belongs to %s\n",
            &[Arg::Ptr(0x1000), Arg::Ptr(0x2000), Arg::U32(0xab), Arg::U32(0xcd), sp]),
        (b"[BLT] IopInitializeBuiltinDriver: %s\r\n", &[sp]),
        (b"%4d-%02d-%02d %02d:%02d:%02d.%03d\n",
            &[Arg::I32(2026), Arg::I32(5), Arg::I32(4),
              Arg::I32(12), Arg::I32(34), Arg::I32(56), Arg::I32(789)]),
    ];
    for (fmt, args) in cases {
        diff_one(fmt, args, "real_world");
    }
}

#[test]
fn cat7_char() {
    let cases: &[(&[u8], &[Arg])] = &[
        (b"%c", &[Arg::U32(b'A' as u32)]),
        (b"<%c>", &[Arg::U32(b'\n' as u32)]),
        (b"%c%c%c", &[Arg::U32(b'h' as u32), Arg::U32(b'i' as u32), Arg::U32(b'!' as u32)]),
    ];
    for (fmt, args) in cases {
        diff_one(fmt, args, "char");
    }
}

#[test]
fn cat8_literal_only() {
    let cases: &[(&[u8], &[Arg])] = &[
        (b"hello\n", &[]),
        (b"100%%", &[]),
        (b"", &[]),
        (b"\n", &[]),
        (b"\t\tHandleCount: \n", &[]),
    ];
    for (fmt, args) in cases {
        diff_one(fmt, args, "literal");
    }
}
