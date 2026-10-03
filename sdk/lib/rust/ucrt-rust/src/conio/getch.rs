//! `getch.cpp`: `_getch`/`_getche` (read one key, without/with echo),
//! `_kbhit`, `_ungetch`, and `_getextendedkeycode` with its two key tables.
//!
//! A key is read with the console in raw mode (mode 0, restored afterwards).
//! A key with a Unicode character is converted to the console input code
//! page; a multibyte result returns its first byte and queues the rest in the
//! push-back buffer. A key without one (arrows, F-keys, ...) is looked up in
//! the tables and returns its lead byte (0 or 224), queueing the second.

use core::ffi::{c_int, c_void};
use core::ptr::{addr_of, addr_of_mut, null, null_mut};

use super::initconin::{
    __dcrt_get_input_console_mode, __dcrt_get_number_of_console_input_events,
    __dcrt_lowio_ensure_console_input_initialized, __dcrt_peek_console_input_a, __dcrt_read_console_input,
    __dcrt_set_input_console_mode,
};
use super::putch::_putch_nolock;
use super::{
    set_errno, with_conio_lock, GetLastError, InputRecord, KeyEventRecord, __acrt_errno_map_os_error, EOF, ERANGE,
    KEY_EVENT,
};

const ENHANCED_KEY: u32 = 0x0100;
const RIGHT_ALT_PRESSED: u32 = 0x0001;
const LEFT_ALT_PRESSED: u32 = 0x0002;
const RIGHT_CTRL_PRESSED: u32 = 0x0004;
const LEFT_CTRL_PRESSED: u32 = 0x0008;
const SHIFT_PRESSED: u32 = 0x0010;

extern "system" {
    fn GetConsoleCP() -> u32;
}

extern "C" {
    fn __acrt_WideCharToMultiByte(
        cp: u32,
        flags: u32,
        src: *const u16,
        src_len: c_int,
        dst: *mut core::ffi::c_char,
        dst_len: c_int,
        default_char: *const core::ffi::c_char,
        used_default: *mut c_int,
    ) -> c_int;
    fn malloc(size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
}

// ── the key tables ───────────────────────────────────────────────────────────

/// `CharPair`: the lead byte (0 or 224 for an extended code) and the code.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CharPair {
    pub lead_char: u8,
    pub second_char: u8,
}

#[repr(C)]
struct EnhKeyVals {
    scan_code: u16,
    reg_chars: CharPair,
    shift_chars: CharPair,
    ctrl_chars: CharPair,
    alt_chars: CharPair,
}

#[repr(C)]
struct NormKeyVals {
    reg_chars: CharPair,
    shift_chars: CharPair,
    ctrl_chars: CharPair,
    alt_chars: CharPair,
}

const _: () = assert!(core::mem::size_of::<EnhKeyVals>() == 10);
const _: () = assert!(core::mem::size_of::<NormKeyVals>() == 8);

const fn p(lead_char: u8, second_char: u8) -> CharPair {
    CharPair { lead_char, second_char }
}

const fn e(scan_code: u16, r: CharPair, s: CharPair, c: CharPair, a: CharPair) -> EnhKeyVals {
    EnhKeyVals { scan_code, reg_chars: r, shift_chars: s, ctrl_chars: c, alt_chars: a }
}

const fn n(r: CharPair, s: CharPair, c: CharPair, a: CharPair) -> NormKeyVals {
    NormKeyVals { reg_chars: r, shift_chars: s, ctrl_chars: c, alt_chars: a }
}

/// `EnhancedKeys[]`.
static ENHANCED_KEYS: [EnhKeyVals; 12] = [
    e(28, p(13, 0), p(13, 0), p(10, 0), p(0, 166)),
    e(53, p(47, 0), p(63, 0), p(0, 149), p(0, 164)),
    e(71, p(224, 71), p(224, 71), p(224, 119), p(0, 151)),
    e(72, p(224, 72), p(224, 72), p(224, 141), p(0, 152)),
    e(73, p(224, 73), p(224, 73), p(224, 134), p(0, 153)),
    e(75, p(224, 75), p(224, 75), p(224, 115), p(0, 155)),
    e(77, p(224, 77), p(224, 77), p(224, 116), p(0, 157)),
    e(79, p(224, 79), p(224, 79), p(224, 117), p(0, 159)),
    e(80, p(224, 80), p(224, 80), p(224, 145), p(0, 160)),
    e(81, p(224, 81), p(224, 81), p(224, 118), p(0, 161)),
    e(82, p(224, 82), p(224, 82), p(224, 146), p(0, 162)),
    e(83, p(224, 83), p(224, 83), p(224, 147), p(0, 163)),
];

const Z: NormKeyVals = n(p(0, 0), p(0, 0), p(0, 0), p(0, 0));

/// `NormalKeys[]`, padded so the scan code is the index (0..=88).
static NORMAL_KEYS: [NormKeyVals; 89] = [
    /*  0 */ Z,
    /*  1 */ n(p(27, 0), p(27, 0), p(27, 0), p(0, 1)),
    /*  2 */ n(p(49, 0), p(33, 0), p(0, 0), p(0, 120)),
    /*  3 */ n(p(50, 0), p(64, 0), p(0, 3), p(0, 121)),
    /*  4 */ n(p(51, 0), p(35, 0), p(0, 0), p(0, 122)),
    /*  5 */ n(p(52, 0), p(36, 0), p(0, 0), p(0, 123)),
    /*  6 */ n(p(53, 0), p(37, 0), p(0, 0), p(0, 124)),
    /*  7 */ n(p(54, 0), p(94, 0), p(30, 0), p(0, 125)),
    /*  8 */ n(p(55, 0), p(38, 0), p(0, 0), p(0, 126)),
    /*  9 */ n(p(56, 0), p(42, 0), p(0, 0), p(0, 127)),
    /* 10 */ n(p(57, 0), p(40, 0), p(0, 0), p(0, 128)),
    /* 11 */ n(p(48, 0), p(41, 0), p(0, 0), p(0, 129)),
    /* 12 */ n(p(45, 0), p(95, 0), p(31, 0), p(0, 130)),
    /* 13 */ n(p(61, 0), p(43, 0), p(0, 0), p(0, 131)),
    /* 14 */ n(p(8, 0), p(8, 0), p(127, 0), p(0, 14)),
    /* 15 */ n(p(9, 0), p(0, 15), p(0, 148), p(0, 15)),
    /* 16 */ n(p(113, 0), p(81, 0), p(17, 0), p(0, 16)),
    /* 17 */ n(p(119, 0), p(87, 0), p(23, 0), p(0, 17)),
    /* 18 */ n(p(101, 0), p(69, 0), p(5, 0), p(0, 18)),
    /* 19 */ n(p(114, 0), p(82, 0), p(18, 0), p(0, 19)),
    /* 20 */ n(p(116, 0), p(84, 0), p(20, 0), p(0, 20)),
    /* 21 */ n(p(121, 0), p(89, 0), p(25, 0), p(0, 21)),
    /* 22 */ n(p(117, 0), p(85, 0), p(21, 0), p(0, 22)),
    /* 23 */ n(p(105, 0), p(73, 0), p(9, 0), p(0, 23)),
    /* 24 */ n(p(111, 0), p(79, 0), p(15, 0), p(0, 24)),
    /* 25 */ n(p(112, 0), p(80, 0), p(16, 0), p(0, 25)),
    /* 26 */ n(p(91, 0), p(123, 0), p(27, 0), p(0, 26)),
    /* 27 */ n(p(93, 0), p(125, 0), p(29, 0), p(0, 27)),
    /* 28 */ n(p(13, 0), p(13, 0), p(10, 0), p(0, 28)),
    /* 29 */ Z,
    /* 30 */ n(p(97, 0), p(65, 0), p(1, 0), p(0, 30)),
    /* 31 */ n(p(115, 0), p(83, 0), p(19, 0), p(0, 31)),
    /* 32 */ n(p(100, 0), p(68, 0), p(4, 0), p(0, 32)),
    /* 33 */ n(p(102, 0), p(70, 0), p(6, 0), p(0, 33)),
    /* 34 */ n(p(103, 0), p(71, 0), p(7, 0), p(0, 34)),
    /* 35 */ n(p(104, 0), p(72, 0), p(8, 0), p(0, 35)),
    /* 36 */ n(p(106, 0), p(74, 0), p(10, 0), p(0, 36)),
    /* 37 */ n(p(107, 0), p(75, 0), p(11, 0), p(0, 37)),
    /* 38 */ n(p(108, 0), p(76, 0), p(12, 0), p(0, 38)),
    /* 39 */ n(p(59, 0), p(58, 0), p(0, 0), p(0, 39)),
    /* 40 */ n(p(39, 0), p(34, 0), p(0, 0), p(0, 40)),
    /* 41 */ n(p(96, 0), p(126, 0), p(0, 0), p(0, 41)),
    /* 42 */ Z,
    /* 43 */ n(p(92, 0), p(124, 0), p(28, 0), p(0, 0)),
    /* 44 */ n(p(122, 0), p(90, 0), p(26, 0), p(0, 44)),
    /* 45 */ n(p(120, 0), p(88, 0), p(24, 0), p(0, 45)),
    /* 46 */ n(p(99, 0), p(67, 0), p(3, 0), p(0, 46)),
    /* 47 */ n(p(118, 0), p(86, 0), p(22, 0), p(0, 47)),
    /* 48 */ n(p(98, 0), p(66, 0), p(2, 0), p(0, 48)),
    /* 49 */ n(p(110, 0), p(78, 0), p(14, 0), p(0, 49)),
    /* 50 */ n(p(109, 0), p(77, 0), p(13, 0), p(0, 50)),
    /* 51 */ n(p(44, 0), p(60, 0), p(0, 0), p(0, 51)),
    /* 52 */ n(p(46, 0), p(62, 0), p(0, 0), p(0, 52)),
    /* 53 */ n(p(47, 0), p(63, 0), p(0, 0), p(0, 53)),
    /* 54 */ Z,
    /* 55 */ n(p(42, 0), p(0, 0), p(114, 0), p(0, 0)),
    /* 56 */ Z,
    /* 57 */ n(p(32, 0), p(32, 0), p(32, 0), p(32, 0)),
    /* 58 */ Z,
    /* 59 */ n(p(0, 59), p(0, 84), p(0, 94), p(0, 104)),
    /* 60 */ n(p(0, 60), p(0, 85), p(0, 95), p(0, 105)),
    /* 61 */ n(p(0, 61), p(0, 86), p(0, 96), p(0, 106)),
    /* 62 */ n(p(0, 62), p(0, 87), p(0, 97), p(0, 107)),
    /* 63 */ n(p(0, 63), p(0, 88), p(0, 98), p(0, 108)),
    /* 64 */ n(p(0, 64), p(0, 89), p(0, 99), p(0, 109)),
    /* 65 */ n(p(0, 65), p(0, 90), p(0, 100), p(0, 110)),
    /* 66 */ n(p(0, 66), p(0, 91), p(0, 101), p(0, 111)),
    /* 67 */ n(p(0, 67), p(0, 92), p(0, 102), p(0, 112)),
    /* 68 */ n(p(0, 68), p(0, 93), p(0, 103), p(0, 113)),
    /* 69 */ Z,
    /* 70 */ Z,
    /* 71 */ n(p(0, 71), p(55, 0), p(0, 119), p(0, 0)),
    /* 72 */ n(p(0, 72), p(56, 0), p(0, 141), p(0, 0)),
    /* 73 */ n(p(0, 73), p(57, 0), p(0, 132), p(0, 0)),
    /* 74 */ n(p(0, 0), p(45, 0), p(0, 0), p(0, 0)),
    /* 75 */ n(p(0, 75), p(52, 0), p(0, 115), p(0, 0)),
    /* 76 */ n(p(0, 0), p(53, 0), p(0, 0), p(0, 0)),
    /* 77 */ n(p(0, 77), p(54, 0), p(0, 116), p(0, 0)),
    /* 78 */ n(p(0, 0), p(43, 0), p(0, 0), p(0, 0)),
    /* 79 */ n(p(0, 79), p(49, 0), p(0, 117), p(0, 0)),
    /* 80 */ n(p(0, 80), p(50, 0), p(0, 145), p(0, 0)),
    /* 81 */ n(p(0, 81), p(51, 0), p(0, 118), p(0, 0)),
    /* 82 */ n(p(0, 82), p(48, 0), p(0, 146), p(0, 0)),
    /* 83 */ n(p(0, 83), p(46, 0), p(0, 147), p(0, 0)),
    /* 84 */ Z,
    /* 85 */ Z,
    /* 86 */ Z,
    /* 87 */ n(p(224, 133), p(224, 135), p(224, 137), p(224, 139)),
    /* 88 */ n(p(224, 134), p(224, 136), p(224, 138), p(224, 140)),
];

// ── the push-back buffer ─────────────────────────────────────────────────────

/// `getch_pushback_buffer_capacity`: room for the three trailing bytes of a
/// 4-byte UTF-8 character.
const GETCH_PUSHBACK_BUFFER_CAPACITY: c_int = 3;

static mut GETCH_PUSHBACK_BUFFER: [c_int; GETCH_PUSHBACK_BUFFER_CAPACITY as usize] = [0; 3];
static mut GETCH_PUSHBACK_BUFFER_INDEX: c_int = 0;
static mut GETCH_PUSHBACK_BUFFER_CURRENT_SIZE: c_int = 0;

unsafe fn is_getch_pushback_buffer_full() -> bool {
    *addr_of!(GETCH_PUSHBACK_BUFFER_CURRENT_SIZE) >= GETCH_PUSHBACK_BUFFER_CAPACITY
}

/// `add_to_getch_pushback_buffer`. The C only `_ASSERTE`s there is room; every
/// caller has checked (or, on the read path, knows the buffer is empty).
unsafe fn add_to_getch_pushback_buffer(c: c_int) {
    let size = addr_of_mut!(GETCH_PUSHBACK_BUFFER_CURRENT_SIZE);
    (addr_of_mut!(GETCH_PUSHBACK_BUFFER) as *mut c_int).add(*size as usize).write(c);
    *size += 1;
}

unsafe fn peek_next_getch_pushback_buffer() -> c_int {
    if *addr_of!(GETCH_PUSHBACK_BUFFER_CURRENT_SIZE) == 0 {
        return EOF;
    }
    (addr_of!(GETCH_PUSHBACK_BUFFER) as *const c_int).add(*addr_of!(GETCH_PUSHBACK_BUFFER_INDEX) as usize).read()
}

/// `get_next_getch_pushback_buffer`: FIFO; the buffer resets only once every
/// queued character has been read.
unsafe fn get_next_getch_pushback_buffer() -> c_int {
    if *addr_of!(GETCH_PUSHBACK_BUFFER_CURRENT_SIZE) == 0 {
        return EOF;
    }

    let index = addr_of_mut!(GETCH_PUSHBACK_BUFFER_INDEX);
    let ret_val = (addr_of!(GETCH_PUSHBACK_BUFFER) as *const c_int).add(*index as usize).read();
    *index += 1;

    if *index == *addr_of!(GETCH_PUSHBACK_BUFFER_CURRENT_SIZE) {
        *index = 0;
        *addr_of_mut!(GETCH_PUSHBACK_BUFFER_CURRENT_SIZE) = 0;
    }

    ret_val
}

/// `__acrt_wcs_to_mbs_cp_array(c_buffer, mb_chars /* [4] */, code_page)`:
/// the byte count (terminator excluded), or 0 on failure -- with errno set
/// (mapped OS error, or ERANGE when four bytes are not enough).
unsafe fn wcs_to_mbs_cp_array4(input: *const u16, buffer: *mut u8, code_page: u32) -> usize {
    // (`input` is never empty here: the caller checked c != 0.)
    let required = __acrt_WideCharToMultiByte(code_page, 0, input, -1, null_mut(), 0, null(), null_mut()) as usize;
    if required == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return 0;
    }
    if required > 4 {
        // __crt_win32_buffer_no_resizing::allocate.
        set_errno(ERANGE);
        return 0;
    }
    let converted = __acrt_WideCharToMultiByte(code_page, 0, input, -1, buffer.cast(), 4, null(), null_mut()) as usize;
    if converted == 0 {
        __acrt_errno_map_os_error(GetLastError());
        return 0;
    }
    converted - 1
}

// ── _getch / _getche ─────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _getch() -> c_int {
    with_conio_lock(|| unsafe { _getch_nolock() })
}

#[no_mangle]
pub unsafe extern "C" fn _getche() -> c_int {
    with_conio_lock(|| unsafe { _getche_nolock() })
}

/// The read loop of `_getch_nolock`, between the mode switch and its restore.
unsafe fn getch_read_loop() -> c_int {
    loop {
        let mut input_record: InputRecord = core::mem::zeroed();
        let mut num_read: u32 = 0;

        if __dcrt_read_console_input(&mut input_record, 1, &mut num_read) == 0 || num_read == 0 {
            return EOF;
        }

        if input_record.event_type == KEY_EVENT && input_record.event.key_event.key_down != 0 {
            // Simple case: a Unicode character, converted to the console CP.
            let c = input_record.event.key_event.u_char;
            if c != 0 {
                let c_buffer: [u16; 2] = [c, 0];
                let mut mb_chars = [0u8; 4];

                let amount_written = wcs_to_mbs_cp_array4(c_buffer.as_ptr(), mb_chars.as_mut_ptr(), GetConsoleCP());

                // A failed conversion returns 0 (not EOF).
                let mut result = 0;
                if amount_written >= 1 {
                    result = (mb_chars[0] & 0xFF) as c_int;
                }
                if amount_written >= 2 {
                    let mut i = 1;
                    while i < amount_written {
                        add_to_getch_pushback_buffer((mb_chars[i] & 0xFF) as c_int);
                        i += 1;
                    }
                }
                return result;
            }

            // Hard case: an extended code, or an event to ignore.
            let cp = _getextendedkeycode(&mut input_record.event.key_event);
            if !cp.is_null() {
                add_to_getch_pushback_buffer(((*cp).second_char & 0xFF) as c_int);
                return ((*cp).lead_char & 0xFF) as c_int;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _getch_nolock() -> c_int {
    let pushback = get_next_getch_pushback_buffer();
    if pushback != EOF {
        return pushback;
    }

    if __dcrt_lowio_ensure_console_input_initialized() == 0 {
        return EOF;
    }

    // Raw mode. (If the mode query fails the C restores an indeterminate
    // value; this restores 0.)
    let mut old_console_mode: u32 = 0;
    __dcrt_get_input_console_mode(&mut old_console_mode);
    __dcrt_set_input_console_mode(0);

    let result = getch_read_loop();

    __dcrt_set_input_console_mode(old_console_mode);
    result
}

#[no_mangle]
pub unsafe extern "C" fn _getche_nolock() -> c_int {
    // A pushed-back character is returned WITHOUT echo.
    let pushback = get_next_getch_pushback_buffer();
    if pushback != EOF {
        return pushback;
    }

    let c = _getch_nolock();
    if c == EOF {
        return EOF;
    }

    if _putch_nolock(c) == EOF {
        return EOF;
    }

    c
}

// ── _kbhit ───────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _kbhit() -> c_int {
    with_conio_lock(|| unsafe { _kbhit_nolock() })
}

/// `__crt_scoped_stack_ptr<INPUT_RECORD>` over `_malloca`: always the heap
/// here (Rust has no `alloca`); only where the memory lives differs.
struct PeekBuffer(*mut InputRecord);

impl Drop for PeekBuffer {
    fn drop(&mut self) {
        unsafe { free(self.0.cast()) }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _kbhit_nolock() -> c_int {
    if peek_next_getch_pushback_buffer() != EOF {
        return 1;
    }

    if __dcrt_lowio_ensure_console_input_initialized() == 0 {
        return 0;
    }

    let mut num_pending: u32 = 0;
    if __dcrt_get_number_of_console_input_events(&mut num_pending) == 0 {
        return 0;
    }

    if num_pending == 0 {
        return 0;
    }

    // `_malloca_crt_t(INPUT_RECORD, n)`: `n * sizeof` in size_t.
    let input_buffer =
        PeekBuffer(malloc((num_pending as usize).wrapping_mul(core::mem::size_of::<InputRecord>())).cast());
    if input_buffer.0.is_null() {
        return 0;
    }

    let mut num_peeked: u32 = 0;
    // AsciiChar is not read, so the narrow API is fine.
    if __dcrt_peek_console_input_a(input_buffer.0, num_pending, &mut num_peeked) == 0 {
        return 0;
    }

    if num_peeked == 0 || num_peeked > num_pending {
        return 0;
    }

    let mut rec = input_buffer.0;
    while num_peeked > 0 {
        let this = rec;
        num_peeked -= 1;
        rec = rec.add(1);

        if (*this).event_type != KEY_EVENT {
            continue;
        }
        let key = addr_of_mut!((*this).event.key_event);
        if (*key).key_down == 0 {
            continue;
        }
        // `uChar.AsciiChar`: the low byte.
        if ((*key).u_char & 0xFF) == 0 && _getextendedkeycode(key).is_null() {
            continue;
        }
        return 1;
    }

    0
}

// ── _ungetch ─────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _ungetch(c: c_int) -> c_int {
    with_conio_lock(|| unsafe { _ungetch_nolock(c) })
}

#[no_mangle]
pub unsafe extern "C" fn _ungetch_nolock(c: c_int) -> c_int {
    if c == EOF || is_getch_pushback_buffer_full() {
        return EOF;
    }

    add_to_getch_pushback_buffer(c);
    c
}

// ── _getextendedkeycode ──────────────────────────────────────────────────────

/// The extended code for a key event that carries no character, or NULL for
/// an event that should not be recognized (a bare Shift, say).
///
/// A non-enhanced key indexes `NormalKeys` by scan code WITHOUT a bounds
/// check, as the C does (scan codes above 88 read past the table); the
/// address is formed with `wrapping_add` so the compiler is not told it stays
/// in bounds.
#[no_mangle]
pub unsafe extern "C" fn _getextendedkeycode(p_ke: *mut KeyEventRecord) -> *const CharPair {
    let cks = (*p_ke).control_key_state;

    if cks & ENHANCED_KEY != 0 {
        let mut i = 0;
        while i < ENHANCED_KEYS.len() {
            let entry = &ENHANCED_KEYS[i];
            i += 1;
            if entry.scan_code != (*p_ke).virtual_scan_code {
                continue;
            }

            return if cks & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED) != 0 {
                &entry.alt_chars
            } else if cks & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) != 0 {
                &entry.ctrl_chars
            } else if cks & SHIFT_PRESSED != 0 {
                &entry.shift_chars
            } else {
                &entry.reg_chars
            };
        }

        null()
    } else {
        let row = NORMAL_KEYS.as_ptr().wrapping_add((*p_ke).virtual_scan_code as usize);
        let p_cp: *const CharPair = if cks & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED) != 0 {
            addr_of!((*row).alt_chars)
        } else if cks & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) != 0 {
            addr_of!((*row).ctrl_chars)
        } else if cks & SHIFT_PRESSED != 0 {
            addr_of!((*row).shift_chars)
        } else {
            addr_of!((*row).reg_chars)
        };

        // Not a recognized key event (e.g. only Shift was pressed):
        if ((*p_cp).lead_char != 0 && (*p_cp).lead_char != 224) || (*p_cp).second_char == 0 {
            return null();
        }

        p_cp
    }
}
