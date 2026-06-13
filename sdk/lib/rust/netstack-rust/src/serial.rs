//! Tiny serial/debug output for the netstack bring-up.
//!
//! Formats into a fixed stack buffer and emits one line via `DbgPrint("%s")`.
//! The `[NETSTACK] ...` lines are the contract tests/qemu/test_net.py asserts
//! on — keep the marker strings in sync with that file.

use core::fmt::{self, Write};

extern "C" {
    // ntoskrnl cdecl-variadic export (same on all arches).
    fn DbgPrint(format: *const u8, ...) -> u32;
}

struct BufWriter {
    buf: [u8; 256],
    len: usize,
}

impl Write for BufWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &b in s.as_bytes() {
            if self.len < self.buf.len() - 1 {
                self.buf[self.len] = b;
                self.len += 1;
            }
        }
        Ok(())
    }
}

pub fn emit(args: fmt::Arguments) {
    let mut w = BufWriter { buf: [0; 256], len: 0 };
    let _ = w.write_fmt(args);
    w.buf[w.len] = 0; // NUL-terminate for %s
    unsafe {
        DbgPrint(b"%s\0".as_ptr(), w.buf.as_ptr());
    }
}

macro_rules! nsprint {
    ($($arg:tt)*) => { $crate::serial::emit(core::format_args!($($arg)*)) };
}
pub(crate) use nsprint;
