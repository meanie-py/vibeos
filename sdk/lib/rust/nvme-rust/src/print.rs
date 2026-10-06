//! `[NVME] ...` lines on the debug port: one formatted line per DbgPrint.

use core::fmt::{self, Write};

extern "C" {
    fn DbgPrint(format: *const u8, ...) -> u32;
}

struct Line {
    buf: [u8; 256],
    len: usize,
}

impl Write for Line {
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
    let mut line = Line { buf: [0; 256], len: 0 };
    let _ = line.write_str("[NVME] ");
    let _ = line.write_fmt(args);
    line.buf[line.len] = 0;
    unsafe {
        DbgPrint(b"%s\0".as_ptr(), line.buf.as_ptr());
    }
}

macro_rules! nvprint {
    ($($arg:tt)*) => { $crate::print::emit(core::format_args!($($arg)*)) };
}
pub(crate) use nvprint;
