//! Validation harness: diff `ntprintf_w` numeric output against host libc
//! `vsnprintf`. The wide format string is widened from an ASCII byte literal,
//! and the wide output is downcast back to ASCII for comparison (legitimate
//! because every value-producing specifier this harness exercises only emits
//! ASCII codepoints — digits, hex chars, sign, padding, prefix).
//!
//! String arguments (`%s`, `%hs`, `%S`) aren't byte-diffable here because the
//! W-fmt pointer types differ from libc's narrow-fmt expectations; the
//! per-spec unit tests in `formatter.rs` cover those.
//!
//! Format-string spec rewrites are the same as the ANSI sibling
//! (`ntprintf/tests/libc_diff.rs`): `%I64x` → `%llx`, `%ld` → `%d`,
//! `%p` → fixed-width zero-padded hex.

use std::ffi::CString;

use ntprintf_w::{format_into, WriterW};

struct VecBuf(Vec<u16>);
impl WriterW for VecBuf {
    fn write_chars(&mut self, chars: &[u16]) { self.0.extend_from_slice(chars); }
    fn written(&self) -> usize { self.0.len() }
}

fn widen(s: &[u8]) -> Vec<u16> {
    s.iter().map(|&b| b as u16).collect()
}

fn narrow(buf: &[u16]) -> Vec<u8> {
    buf.iter().map(|&c| (c & 0xFF) as u8).collect()
}

/// Same Win→libc rewrite as the ANSI harness.
fn rewrite_fmt_for_libc(fmt: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(fmt.len() + 16);
    let mut i = 0;
    while i < fmt.len() {
        if fmt[i] != b'%' {
            out.push(fmt[i]);
            i += 1;
            continue;
        }
        out.push(b'%');
        i += 1;
        while i < fmt.len() && matches!(fmt[i],
            b'-' | b'+' | b' ' | b'#' | b'0' | b'.' | b'*' | b'1'..=b'9') {
            out.push(fmt[i]);
            i += 1;
        }
        // Length-modifier rewrite.
        if i + 2 < fmt.len() && fmt[i] == b'I' && fmt[i+1] == b'6' && fmt[i+2] == b'4' {
            out.extend_from_slice(b"ll");
            i += 3;
        } else if i + 2 < fmt.len() && fmt[i] == b'I' && fmt[i+1] == b'3' && fmt[i+2] == b'2' {
            i += 3;
        } else if i < fmt.len() && fmt[i] == b'l'
            && i + 1 < fmt.len() && fmt[i+1] != b'l'
        {
            // Drop a single `l` so libc doesn't read 8 bytes off stack
            i += 1;
        } else if i < fmt.len() && fmt[i] == b'h' {
            i += 1;
            if i < fmt.len() && fmt[i] == b'h' { i += 1; }
        }
        // %p → %0Nx
        if i < fmt.len() && fmt[i] == b'p' {
            #[cfg(target_pointer_width = "64")]
            out.extend_from_slice(b"016lx");
            #[cfg(target_pointer_width = "32")]
            out.extend_from_slice(b"08lx");
            i += 1;
            continue;
        }
        if i < fmt.len() {
            out.push(fmt[i]);
            i += 1;
        }
    }
    out
}

fn rust_render(fmt: &[u8], slots: &[u64]) -> String {
    let wfmt = widen(fmt);
    let mut buf = VecBuf(Vec::new());
    format_into(&mut buf, &wfmt, slots);
    String::from_utf8(narrow(&buf.0)).expect("ASCII output")
}

fn libc_render_0(fmt: &[u8]) -> String { libc_render(fmt, &[]) }
fn libc_render_1(fmt: &[u8], a: u64) -> String { libc_render(fmt, &[a]) }
fn libc_render_2(fmt: &[u8], a: u64, b: u64) -> String { libc_render(fmt, &[a, b]) }
fn libc_render_3(fmt: &[u8], a: u64, b: u64, c: u64) -> String { libc_render(fmt, &[a, b, c]) }

/// Render via libc `snprintf`, dispatched on slot count.
fn libc_render(fmt: &[u8], slots: &[u64]) -> String {
    let cfmt = CString::new(rewrite_fmt_for_libc(fmt)).unwrap();
    let mut buf = vec![0u8; 256];
    let n = unsafe {
        match slots.len() {
            0 => libc::snprintf(buf.as_mut_ptr() as *mut i8, buf.len(),
                                cfmt.as_ptr()),
            1 => libc::snprintf(buf.as_mut_ptr() as *mut i8, buf.len(),
                                cfmt.as_ptr(), slots[0]),
            2 => libc::snprintf(buf.as_mut_ptr() as *mut i8, buf.len(),
                                cfmt.as_ptr(), slots[0], slots[1]),
            3 => libc::snprintf(buf.as_mut_ptr() as *mut i8, buf.len(),
                                cfmt.as_ptr(), slots[0], slots[1], slots[2]),
            n => panic!("slot count {} not supported by harness", n),
        }
    };
    assert!(n >= 0);
    buf.truncate(n as usize);
    String::from_utf8(buf).expect("ASCII output")
}

#[track_caller]
fn diff(fmt: &[u8], slots: &[u64]) {
    let r = rust_render(fmt, slots);
    let l = match slots.len() {
        0 => libc_render_0(fmt),
        1 => libc_render_1(fmt, slots[0]),
        2 => libc_render_2(fmt, slots[0], slots[1]),
        3 => libc_render_3(fmt, slots[0], slots[1], slots[2]),
        _ => unreachable!(),
    };
    assert_eq!(r, l, "fmt={:?} slots={:?}", String::from_utf8_lossy(fmt), slots);
}

#[test] fn literals() {
    diff(b"hello", &[]);
    diff(b"100%%", &[]);
    diff(b"a%db", &[42]);
}

#[test] fn decimals() {
    diff(b"%d", &[42]);
    diff(b"%d", &[(-42i32) as u32 as u64]);
    diff(b"%u", &[0xFFFFFFFF]);
    diff(b"%i", &[(-1i32) as u32 as u64]);
    diff(b"%ld", &[1234]);
    diff(b"%lu", &[1234]);
}

#[test] fn hex() {
    diff(b"%x", &[0xdeadbeef]);
    diff(b"%X", &[0xdeadbeef]);
    diff(b"%08x", &[0x42]);
    diff(b"%#x", &[0x42]);
    diff(b"%lx", &[0xabcd]);
}

#[test] fn long_long_and_i64() {
    diff(b"%I64d", &[0x1_0000_0001]);
    diff(b"%I64u", &[0xdeadbeef_cafef00d]);
    diff(b"%I64x", &[0x12345678_abcdef00]);
    diff(b"%016I64x", &[0x42]);
}

#[test] fn width_and_padding() {
    diff(b"%5d", &[42]);
    diff(b"%-5d|", &[42]);
    diff(b"%05d", &[42]);
    diff(b"%08x", &[0xab]);
    diff(b"%03ld", &[7]);
}

#[test] fn precision() {
    diff(b"%.5d", &[42]);
    diff(b"%5.3d", &[42]);
    diff(b"%.0d", &[0]);
}

#[test] fn signed_flags() {
    diff(b"%+d", &[42]);
    diff(b"% d", &[42]);
    diff(b"%+d", &[(-42i32) as u32 as u64]);
}

#[test] fn pointer() {
    diff(b"%p", &[0xdeadbeef]);
}

#[test] fn real_world_wide() {
    // Translated from inventory:
    diff(b"\\Device\\Harddisk%d\\Partition%d", &[0, 1]);
    diff(b"\\Registry\\Machine\\System\\ControlSet%03ld", &[7]);
    diff(b"%lx&%lx", &[0x1234_5678, 0xcafe_d00d]);
    diff(b"\\DosDevices\\%c:", &[b'C' as u64]);
    diff(b"%04ld", &[42]);
    diff(b"ControlSet%03lu", &[1]);
}
