//! kd/kdps2kbd — PS/2 Keyboard driver for KDBG terminal.
//!
//! Translated from ntoskrnl/kd/kdps2kbd.c. Used by kdio.rs (mouse enable/disable
//! around prompt) and kdterminal.rs (KdbpTryGetCharKeyboard).
//!
//! Port I/O is x86-only (PS/2 controller at 0x60/0x64). On ARM64 the I/O
//! primitives are stubbed since this driver is only used by kdbg builds, and
//! kdbg is disabled on ARM64 (see ntos.cmake). The Rust crate still compiles
//! on all 3 arches since `pub mod kdps2kbd;` is unconditional.

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::KeStallExecutionProcessor;

// PS/2 controller ports
const KBD_STATUS_REG: u16 = 0x64;
const KBD_CNTL_REG: u16 = 0x64;
const KBD_DATA_REG: u16 = 0x60;

// Status register bits
const KBD_STAT_OBF: u8 = 0x01;
const KBD_STAT_IBF: u8 = 0x02;

// Mouse / controller commands
const CTRL_WRITE_MOUSE: u8 = 0xD4;
const MOU_ENAB: u8 = 0xF4;
const MOU_DISAB: u8 = 0xF5;
const MOUSE_ACK: u8 = 0xFA;

// ---------------------------------------------------------------------------
// Port I/O — x86/x86_64 use real `in`/`out`. ARM64 stubs (kdbg not built).
// ---------------------------------------------------------------------------

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
unsafe fn read_port_uchar(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!(
        "in al, dx",
        in("dx") port,
        out("al") value,
        options(nomem, nostack, preserves_flags)
    );
    value
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
unsafe fn write_port_uchar(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn read_port_uchar(_port: u16) -> u8 {
    // PS/2 controller does not exist on ARM64 / QEMU virt. KDBG is disabled
    // on ARM64 so these helpers should never actually be called.
    0
}

#[cfg(target_arch = "aarch64")]
#[inline]
unsafe fn write_port_uchar(_port: u16, _value: u8) {
    // See note above; no-op on ARM64.
}

#[inline]
unsafe fn kbd_read_input() -> u8 {
    read_port_uchar(KBD_DATA_REG)
}

#[inline]
unsafe fn kbd_read_status() -> u8 {
    read_port_uchar(KBD_STATUS_REG)
}

#[inline]
unsafe fn kbd_write_command(cmd: u8) {
    write_port_uchar(KBD_CNTL_REG, cmd);
}

#[inline]
unsafe fn kbd_write_data(cmd: u8) {
    write_port_uchar(KBD_DATA_REG, cmd);
}

// ---------------------------------------------------------------------------
// Keyboard layout table.
//
// Mirrors the C `keyb_layout[2][128]` from kdps2kbd.c. Each row is the
// scan-code → ASCII map for unshifted (row 0) and shifted (row 1) state.
// Bytes >= 0x80 are special keys (function keys, arrows, etc.) and are
// represented here as their unsigned octal values; the lookup result is
// reinterpreted as `i8` and rejected when negative (matches `if (c > 0)` in C).
// ---------------------------------------------------------------------------

#[no_mangle]
pub static mut keyb_layout: [[u8; 128]; 2] = [
    // Unshifted
    [
        // 0x00-0x0f: "\000\0331234567890-=\177\t"
        0x00, 0x1B, b'1', b'2', b'3', b'4', b'5', b'6',
        b'7', b'8', b'9', b'0', b'-', b'=', 0x7F, b'\t',
        // 0x10-0x1f: "qwertyuiop[]\r\000as"
        b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i',
        b'o', b'p', b'[', b']', b'\r', 0x00, b'a', b's',
        // 0x20-0x2f: "dfghjkl;'`\000\\zxcv"
        b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';',
        b'\'', b'`', 0x00, b'\\', b'z', b'x', b'c', b'v',
        // 0x30-0x3f: "bnm,./\000*\000 \000\201\202\203\204\205"
        b'b', b'n', b'm', b',', b'.', b'/', 0x00, b'*',
        0x00, b' ', 0x00, 0o201, 0o202, 0o203, 0o204, 0o205,
        // 0x40-0x4f: "\206\207\210\211\212\000\000789-456+1"
        0o206, 0o207, 0o210, 0o211, 0o212, 0x00, 0x00, b'7',
        b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1',
        // 0x50-0x5f: "230\177\000\000\213\214\000\000\000\000\000\000\000\000\000\000"
        b'2', b'3', b'0', 0x7F, 0x00, 0x00, 0o213, 0o214,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // 0x60-0x6f: "\r\000/"
        b'\r', 0x00, b'/', 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // 0x70-0x7f: zero-padded
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],
    // Shifted
    [
        // 0x00-0x0f: "\000\033!@#$%^&*()_+\177\t"
        0x00, 0x1B, b'!', b'@', b'#', b'$', b'%', b'^',
        b'&', b'*', b'(', b')', b'_', b'+', 0x7F, b'\t',
        // 0x10-0x1f: "QWERTYUIOP{}\r\000AS"
        b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I',
        b'O', b'P', b'{', b'}', b'\r', 0x00, b'A', b'S',
        // 0x20-0x2f: "DFGHJKL:\"`\000\\ZXCV"
        b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':',
        b'"', b'`', 0x00, b'\\', b'Z', b'X', b'C', b'V',
        // 0x30-0x3f: "BNM<>?\000*\000 \000\201\202\203\204\205"
        b'B', b'N', b'M', b'<', b'>', b'?', 0x00, b'*',
        0x00, b' ', 0x00, 0o201, 0o202, 0o203, 0o204, 0o205,
        // 0x40-0x4f: "\206\207\210\211\212\000\000789-456+1"
        0o206, 0o207, 0o210, 0o211, 0o212, 0x00, 0x00, b'7',
        b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1',
        // 0x50-0x5f
        b'2', b'3', b'0', 0x7F, 0x00, 0x00, 0o213, 0o214,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // 0x60-0x6f: "\r\000/"
        b'\r', 0x00, b'/', 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // 0x70-0x7f
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],
];

// `last_key` and `shift` are persistent across calls. C marks them `static`
// inside the function; we hoist them to module-level statics with no_mangle
// so the symbols are owned by this Rust file.
#[no_mangle]
pub static mut KdbpKbd_last_key: u8 = 0;

#[no_mangle]
pub static mut KdbpKbd_shift: u8 = 0;

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

unsafe fn KbdSendCommandToMouse(Command: u8) {
    let mut Retry: u32 = 20000;

    while (kbd_read_status() & KBD_STAT_OBF) != 0 && Retry > 0 {
        Retry -= 1;
        let _ = kbd_read_input();
        KeStallExecutionProcessor(50);
    }

    Retry = 20000;
    while (kbd_read_status() & KBD_STAT_IBF) != 0 && Retry > 0 {
        Retry -= 1;
        KeStallExecutionProcessor(50);
    }

    kbd_write_command(CTRL_WRITE_MOUSE);

    Retry = 20000;
    while (kbd_read_status() & KBD_STAT_IBF) != 0 && Retry > 0 {
        Retry -= 1;
        KeStallExecutionProcessor(50);
    }

    kbd_write_data(Command);

    Retry = 20000;
    while (kbd_read_status() & KBD_STAT_OBF) == 0 && Retry > 0 {
        Retry -= 1;
        KeStallExecutionProcessor(50);
    }

    if kbd_read_input() != MOUSE_ACK {
        // C source has `{ ; }` — intentional empty body
    }
}

#[no_mangle]
pub unsafe extern "C" fn KbdEnableMouse() {
    KbdSendCommandToMouse(MOU_ENAB);
}

#[no_mangle]
pub unsafe extern "C" fn KbdDisableMouse() {
    KbdSendCommandToMouse(MOU_DISAB);
}

#[no_mangle]
pub unsafe extern "system" fn KdbpTryGetCharKeyboard(ScanCode: PULONG, Retry: ULONG) -> LONG {
    let KeepRetrying: bool = Retry == 0;
    let mut retry_count = Retry;

    while KeepRetrying || retry_count > 0 {
        if !KeepRetrying {
            retry_count = retry_count.wrapping_sub(1);
        }

        while (kbd_read_status() & KBD_STAT_OBF) != 0 {
            let scancode: u8 = kbd_read_input();

            // SHIFT-key make/break (left-shift = 42, right-shift = 54)
            if (scancode & 0x7F) == 42 || (scancode & 0x7F) == 54 {
                KdbpKbd_shift = if (scancode & 0x80) != 0 { 0 } else { 1 };
                continue;
            }

            // Ignore other RELEASED scan codes (top bit set)
            if (scancode & 0x80) != 0 {
                KdbpKbd_last_key = 0;
            } else if KdbpKbd_last_key != scancode {
                KdbpKbd_last_key = scancode;
                let raw: u8 = keyb_layout[KdbpKbd_shift as usize][scancode as usize];
                let c: i8 = raw as i8;
                *ScanCode = scancode as ULONG;

                if c > 0 {
                    return c as LONG;
                }
            }
        }
    }

    -1
}
