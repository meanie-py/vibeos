use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{PCCH, PCSTR, PSTR};
use core::ffi::c_void;

extern "C" {
    fn strcmp(s1: *const i8, s2: *const i8) -> i32;
    fn strlen(s: *const i8) -> usize;
    fn UiMessageBox(msg: *const i8);
    fn sprintf(s: *mut i8, format: *const i8, ...) -> i32;
    fn UiMessageBoxCritical(MessageText: *const i8);
}
extern "C" {
    fn Rs232PortPutByte(b: u8);
    fn MachConsPutChar(c: i32);
    fn WRITE_PORT_UCHAR(port: *mut u8, val: u8);
}
extern "system" {
    fn DbgBreakPoint();
}

// debug.c file globals (rustconv dropped).
const SCREEN: ULONG = 1;
const RS232: ULONG = 2;
const BOCHS: ULONG = 4;
const BOCHS_OUTPUT_PORT: ULONG = 0xE9;
static mut DbgChannels: [u8; crate::frldr::DBG_CHANNELS_COUNT] = [0; crate::frldr::DBG_CHANNELS_COUNT];
#[no_mangle] pub static mut DebugPort: ULONG = 2; // RS232
#[no_mangle] pub static mut DebugStartOfLine: BOOLEAN = 1;

unsafe fn DebugPrintChar(Character: u8) {
    if Character == b'\n' {
        DebugStartOfLine = 1u8;
    }

    if (DebugPort & RS232) != 0 {
        if Character == b'\n' {
            Rs232PortPutByte(b'\r');
        }

        Rs232PortPutByte(Character);
    }
    if (DebugPort & BOCHS) != 0 {
        WRITE_PORT_UCHAR(BOCHS_OUTPUT_PORT as *mut u8, Character);
    }
    if (DebugPort & SCREEN) != 0 {
        MachConsPutChar(Character as i32);
    }
}

#[no_mangle]
pub unsafe extern "C" fn DebugDisableScreenPort() {
    DebugPort &= !SCREEN;
}

unsafe fn DbgAddDebugChannel(channel: *mut i8, level: *mut i8, op: i8) -> u8 {

    let iLevel: ULONG;
    let iChannel: ULONG;

    if channel.is_null() || *channel == 0 || strlen(channel) == 0 {
        return 0u8;
    }

    if level.is_null() || *level == 0 || strlen(level) == 0 {
        iLevel = MAX_LEVEL;
    } else if strcmp(level, b"err\0".as_ptr() as *const i8) == 0 {
        iLevel = ERR_LEVEL;
    } else if strcmp(level, b"fixme\0".as_ptr() as *const i8) == 0 {
        iLevel = FIXME_LEVEL;
    } else if strcmp(level, b"warn\0".as_ptr() as *const i8) == 0 {
        iLevel = WARN_LEVEL;
    } else if strcmp(level, b"trace\0".as_ptr() as *const i8) == 0 {
        iLevel = TRACE_LEVEL;
    } else {
        return 0u8;
    }

    if strcmp(channel, b"memory\0".as_ptr() as *const i8) == 0 {
        iChannel = DPRINT_MEMORY;
    } else if strcmp(channel, b"filesystem\0".as_ptr() as *const i8) == 0 {
        iChannel = DPRINT_FILESYSTEM;
    } else if strcmp(channel, b"inifile\0".as_ptr() as *const i8) == 0 {
        iChannel = DPRINT_INIFILE;
    } else if strcmp(channel, b"ui\0".as_ptr() as *const i8) == 0 {
        iChannel = DPRINT_UI;
    } else if strcmp(channel, b"disk\0".as_ptr() as *const i8) == 0 {
        iChannel = DPRINT_DISK;
    } else if strcmp(channel, b"cache\0".as_ptr() as *const i8) == 0 {
        iChannel = DPRINT_CACHE;
    } else if strcmp(channel, b"registry\0".as_ptr() as *const i8) == 0 {
        iChannel = DPRINT_REGISTRY;
    } else if strcmp(channel, b"linux\0".as_ptr() as *const i8) == 0 {
        iChannel = DPRINT_LINUX;
    } else if strcmp(channel, b"hwdetect\0".as_ptr() as *const i8) == 0 {
        iChannel = DPRINT_HWDETECT;
    } else if strcmp(channel, b"windows\0".as_ptr() as *const i8) == 0 {
        iChannel = DPRINT_WINDOWS;
    } else if strcmp(channel, b"peloader\0".as_ptr() as *const i8) == 0 {
        iChannel = DPRINT_PELOADER;
    } else if strcmp(channel, b"scsiport\0".as_ptr() as *const i8) == 0 {
        iChannel = DPRINT_SCSIPORT;
    } else if strcmp(channel, b"heap\0".as_ptr() as *const i8) == 0 {
        iChannel = DPRINT_HEAP;
    } else if strcmp(channel, b"all\0".as_ptr() as *const i8) == 0 {
        for i in 0..DBG_CHANNELS_COUNT {
            if op == b'+' as i8 {
                DbgChannels[i as usize] |= iLevel as u8;
            } else {
                DbgChannels[i as usize] &= !(iLevel as u8);
            }
        }
        return 1u8;
    } else {
        return 0u8;
    }

    if op == b'+' as i8 {
        DbgChannels[iChannel as usize] |= iLevel as u8;
    } else {
        DbgChannels[iChannel as usize] &= !(iLevel as u8);
    }

    1u8
}

// Faithful port of C debug.c DebugInit/DbgPrint/DbgPrint2. The original
// translation shipped the C file's `#else` (!DBG) NO-OP stubs here, which
// silenced ALL freeldr debug output on serial — freeldr's whole load phase
// became one unattributable silent gap in every boot log, and rosload's
// diagnostics were lost entirely (found during the 2026-07 performance
// investigation; the [FLDR] boot-phase stamps ride on this channel).

static mut DebugInitialized: bool = false;
static mut DbgComPort: ULONG = 0; // 0 = probe COM4..COM1 (rs232.rs)
static mut DbgBaudRate: ULONG = 115200;

/// Case-sensitive substring find over a NUL-terminated C string (freeldr's
/// DebugString is upper-cased by settings.c / hand-written in the ini).
unsafe fn c_strstr(hay: *const i8, needle: &[u8]) -> *const i8 {
    if hay.is_null() { return core::ptr::null(); }
    let mut p = hay;
    while *p != 0 {
        let mut i = 0usize;
        while i < needle.len() && *p.add(i) as u8 == needle[i] { i += 1; }
        if i == needle.len() { return p; }
        p = p.add(1);
    }
    core::ptr::null()
}

unsafe fn c_hex(mut p: *const i8) -> ULONG {
    if *p == b'0' as i8 && (*p.add(1) == b'x' as i8 || *p.add(1) == b'X' as i8) {
        p = p.add(2);
    }
    let mut v: ULONG = 0;
    loop {
        let c = *p as u8;
        let d = match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => break,
        };
        v = v.wrapping_mul(16).wrapping_add(d as ULONG);
        p = p.add(1);
    }
    v
}

unsafe fn c_atol(mut p: *const i8) -> ULONG {
    let mut v: ULONG = 0;
    while *p == b' ' as i8 { p = p.add(1); }
    while (*p as u8).is_ascii_digit() {
        v = v.wrapping_mul(10).wrapping_add((*p as u8 - b'0') as ULONG);
        p = p.add(1);
    }
    v
}

#[no_mangle]
pub unsafe extern "C" fn DebugInit(DebugString: PCSTR) {
    // Empty/NULL string: pre-initialization — bring the port up with default
    // settings the first time; on later calls without a string, keep whatever
    // the main initialization configured (C semantics).
    if DebugString.is_null() || *DebugString == 0 {
        if DebugInitialized { return; }
    } else {
        // Parse DEBUGPORT= (SCREEN / BOCHS / COMn, OR-able) and BAUDRATE=.
        let mut port = c_strstr(DebugString as *const i8, b"DEBUGPORT");
        if !port.is_null() { DebugPort = 0; }
        while !port.is_null() {
            let mut q = port.add(b"DEBUGPORT".len() as usize);
            while *q == b' ' as i8 { q = q.add(1); }
            q = q.add(1); // '='
            if !c_strstr(q, b"SCREEN").is_null() && c_strstr(q, b"SCREEN") == q {
                DebugPort |= SCREEN;
            } else if !c_strstr(q, b"BOCHS").is_null() && c_strstr(q, b"BOCHS") == q {
                DebugPort |= BOCHS;
            } else {
                let com = c_strstr(q, b"COM");
                if com == q {
                    DebugPort |= RS232;
                    // COMn, or COM:0x<port> (the kernel's KD accepts the same).
                    let v = if *com.add(3) == b':' as i8 {
                        c_hex(com.add(4))
                    } else {
                        c_atol(com.add(3))
                    };
                    if v != 0 { DbgComPort = v; }
                }
            }
            port = c_strstr(q, b"DEBUGPORT");
        }
        let baud = c_strstr(DebugString as *const i8, b"BAUDRATE");
        if !baud.is_null() {
            let mut q = baud.add(b"BAUDRATE".len());
            while *q == b' ' as i8 { q = q.add(1); }
            let v = c_atol(q.add(1));
            if v != 0 { DbgBaudRate = v; }
        }
    }

    DebugInitialized = true;

    /* Try to initialize the port; if it fails, drop the RS232 flag */
    extern "C" {
        fn Rs232PortInitialize(ComPort: ULONG, BaudRate: ULONG) -> u8;
    }
    if (DebugPort & RS232) != 0 {
        if Rs232PortInitialize(DbgComPort, DbgBaudRate) == 0 && !spcr_fallback() {
            DebugPort &= !RS232;
        }
    }
}

/// The I/O port the debug channel moved to because the configured COM port is
/// absent and an ACPI SPCR described another UART; 0 when no move happened.
/// winldr hands it to the kernel (SpcrRewriteDebugPort) so KD follows.
static mut SpcrDebugPort: ULONG = 0;

#[cfg(all(feature = "uefi", any(target_arch = "x86", target_arch = "x86_64")))]
unsafe fn spcr_fallback() -> bool {
    extern "C" {
        fn UefiSpcrConsolePort() -> ULONG;
        fn Rs232PortInitialize(ComPort: ULONG, BaudRate: ULONG) -> u8;
    }
    let port = UefiSpcrConsolePort();
    if port == 0 || Rs232PortInitialize(port, DbgBaudRate) == 0 {
        return false;
    }
    let already = SpcrDebugPort == port;
    SpcrDebugPort = port;
    if !already {
        // DbgComPort 0 is the pre-init probe of COM4..COM1, not a named port.
        DbgPrint(
            b"[FLDR] debug port: no legacy COM port, using the ACPI SPCR UART at I/O 0x%lx\n\0"
                .as_ptr() as *const i8,
            port as usize,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        );
    }
    true
}

#[cfg(not(all(feature = "uefi", any(target_arch = "x86", target_arch = "x86_64"))))]
unsafe fn spcr_fallback() -> bool {
    false
}

/// When the debug channel moved to an SPCR UART, rewrite the kernel's
/// `DEBUGPORT=COMn` in `options` (a NUL-terminated buffer of `cap` bytes) to
/// `DEBUGPORT=COM:0x<port>`, so KD prints where the loader did. Otherwise, or
/// if the rewrite would not fit, the options are left alone.
#[no_mangle]
pub unsafe extern "C" fn SpcrRewriteDebugPort(options: *mut i8, cap: usize) {
    if SpcrDebugPort == 0 || options.is_null() {
        return;
    }
    let at = c_strstr(options, b"DEBUGPORT=COM");
    if at.is_null() {
        DbgPrint(
            b"[FLDR] kernel options carry no DEBUGPORT=COMn to move to the SPCR UART: %s\n\0".as_ptr()
                as *const i8,
            options as usize,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        );
        return;
    }
    let digit = at.add(b"DEBUGPORT=COM".len()) as *mut i8;
    if !(*digit as u8).is_ascii_digit() {
        return;
    }
    let mut repl = [0u8; 16];
    let n = format_port(&mut repl, SpcrDebugPort);
    let len = {
        let mut l = 0usize;
        while *options.add(l) != 0 { l += 1; }
        l
    };
    // One digit becomes `n` bytes: the tail (after the digit, with its NUL) shifts right.
    let tail = len - (digit.offset_from(options) as usize + 1) + 1;
    if len + n - 1 + 1 > cap {
        return;
    }
    core::ptr::copy(digit.add(1), digit.add(n), tail);
    core::ptr::copy_nonoverlapping(repl.as_ptr() as *const i8, digit, n);
    DbgPrint(
        b"[FLDR] kernel options: debug port moved to the SPCR UART: %s\n\0".as_ptr() as *const i8,
        options as usize,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    );
}

/// `:0x<hex>` for a port number; returns the byte count written.
fn format_port(out: &mut [u8; 16], port: ULONG) -> usize {
    let mut digits = [0u8; 8];
    let mut v = port;
    let mut d = 0;
    loop {
        digits[d] = b"0123456789ABCDEF"[(v & 0xf) as usize];
        d += 1;
        v >>= 4;
        if v == 0 { break; }
    }
    out[..3].copy_from_slice(b":0x");
    for i in 0..d {
        out[3 + i] = digits[d - 1 - i];
    }
    3 + d
}

unsafe fn DbgPrintString(s: *const i8) {
    let mut p = s;
    while *p != 0 {
        DebugPrintChar(*p as u8);
        p = p.add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn DbgPrintSz(Str: PCSTR) {
    // Unformatted string print for Rust callers (freeldr-rust builds on the
    // STABLE channel — no c_variadic, so no varargs DbgPrint definition).
    if !DebugInitialized {
        DebugInit(core::ptr::null());
    }
    DbgPrintString(Str as *const i8);
}

/// NOTE: the C prototype is variadic (`DbgPrint(fmt, ...)`); defining a
// C debug.c:33 -- ERR and FIXME print even when their channel is closed.
const DBG_DEFAULT_LEVELS: ULONG = crate::frldr::ERR_LEVEL | crate::frldr::FIXME_LEVEL;

// ---------------------------------------------------------------------------
// Formatting varargs without `c_variadic`: the slot-capture technique, the
// measured VA_SLOTS/VA_STAGE bounds and why they are safe all live in
// `va_slots`, which the tui/rosload printf family shares with these.
use va_slots_rust::format_slots_z;

/// `DbgPrint(Format, ...)` -- freeldr's own, not ntoskrnl's.
#[no_mangle]
pub unsafe extern "C" fn DbgPrint(
    Format: PCCH,
    a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize,
    a6: usize, a7: usize, a8: usize, a9: usize, a10: usize, a11: usize,
) -> u32 {
    if !DebugInitialized {
        DebugInit(core::ptr::null());
    }
    let mut buf: [i8; 512] = [0; 512];
    format_slots_z(&mut buf, Format as *const i8,
        &[a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11]);
    DbgPrintString(buf.as_ptr());
    0
}

#[no_mangle]
pub unsafe extern "C" fn DbgPrint2(
    Mask: ULONG,
    Level: ULONG,
    File: *const CHAR,
    Line: ULONG,
    Format: *mut CHAR,
    a0: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize,
    a6: usize, a7: usize, a8: usize, a9: usize, a10: usize, a11: usize,
) {
    // C: `if (!(DbgChannels[Mask] & Level) && !(Level & DBG_DEFAULT_LEVELS))`.
    // The second term was DROPPED in the original port, and it is the one that
    // matters: DbgChannels[] starts all-zero (no DEBUG_ALL/_WARN/_ERR define in
    // any freeldr CMakeLists) and only `/DEBUG=` opens a channel, so requiring
    // an open channel silenced every ERR and FIXME freeldr has -- ~2200 sites,
    // zero output in a boot log. ERR/FIXME are channel-INDEPENDENT here; only
    // WARN and TRACE are gated, and their silence at boot is faithful.
    //
    // The bounds check is ours: C indexes DbgChannels[Mask] unguarded. Written
    // so an out-of-range Mask still cannot suppress a default-level message.
    let channel_open = (Mask as usize) < crate::frldr::DBG_CHANNELS_COUNT
        && (DbgChannels[Mask as usize] as u32 & Level) != 0;
    if !channel_open && (Level & DBG_DEFAULT_LEVELS) == 0 {
        return;
    }
    if !DebugInitialized {
        DebugInit(core::ptr::null());
    }
    if DebugStartOfLine != 0 {
        let mut hdr: [i8; 128] = [0; 128];
        sprintf(hdr.as_mut_ptr(), b"(%s:%lu) \0".as_ptr() as *const i8, File, Line);
        DbgPrintString(hdr.as_ptr());
        // The level tag the C prints after the header, also dropped in the
        // original port -- without it ERR and TRACE are indistinguishable.
        DbgPrintString(match Level {
            crate::frldr::ERR_LEVEL => b"err: \0".as_ptr() as *const i8,
            crate::frldr::FIXME_LEVEL => b"fixme: \0".as_ptr() as *const i8,
            crate::frldr::WARN_LEVEL => b"warn: \0".as_ptr() as *const i8,
            crate::frldr::TRACE_LEVEL => b"trace: \0".as_ptr() as *const i8,
            _ => b"\0".as_ptr() as *const i8,
        });
        DebugStartOfLine = 0;
    }
    let mut buf: [i8; 2096] = [0; 2096];
    format_slots_z(&mut buf, Format as *const i8,
        &[a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11]);
    DbgPrintString(buf.as_ptr());
}

#[no_mangle]
pub unsafe extern "C" fn DebugDumpBuffer(Mask: ULONG, Buffer: PVOID, Length: ULONG) {
    // Faithful port of debug.c. Note this one gates on TRACE_LEVEL alone with no
    // DBG_DEFAULT_LEVELS bypass, so unlike DbgPrint2's ERR/FIXME it really is
    // silent until `/DEBUG=` opens the channel — that silence is the C's.
    let BufPtr = Buffer as *const u8;
    if Mask as usize >= crate::frldr::DBG_CHANNELS_COUNT
        || (DbgChannels[Mask as usize] as u32 & crate::frldr::TRACE_LEVEL) == 0
    {
        return;
    }
    if !DebugInitialized {
        DebugInit(core::ptr::null());
    }

    let mut line: [i8; 128] = [0; 128];
    // We don't want line headers.
    DebugStartOfLine = 0;
    sprintf(
        line.as_mut_ptr(),
        b"Dumping buffer at %p with length of %lu bytes:\n\0".as_ptr() as *const i8,
        Buffer,
        Length,
    );
    DbgPrintString(line.as_ptr());

    let mut Offset: ULONG = 0;
    while Offset < Length {
        DebugStartOfLine = 0;
        sprintf(line.as_mut_ptr(), b"%04x:\t\0".as_ptr() as *const i8, Offset);
        DbgPrintString(line.as_ptr());

        // Either 16 or the remaining number of bytes.
        let Count = core::cmp::min(Length - Offset, 16);
        for i in 0..Count {
            sprintf(
                line.as_mut_ptr(),
                b"%02x%c\0".as_ptr() as *const i8,
                *BufPtr.add(Offset as usize) as core::ffi::c_uint,
                if i == 7 { b'-' } else { b' ' } as core::ffi::c_int,
            );
            DbgPrintString(line.as_ptr());
            Offset += 1;
        }

        DbgPrintString(b"\n\0".as_ptr() as *const i8);
    }
}

/// Case-sensitive `strchr` over a NUL-terminated C string; NULL if absent.
/// Local rather than an extern so this file keeps its two-symbol libc surface.
unsafe fn c_strchr(mut p: *mut i8, ch: u8) -> *mut i8 {
    while *p != 0 {
        if *p as u8 == ch {
            return p;
        }
        p = p.add(1);
    }
    core::ptr::null_mut()
}

/// Faithful port of debug.c. This was a no-op stub, and since it is the ONLY
/// caller of `DbgAddDebugChannel`, the whole `/DEBUG=` channel mechanism was
/// dead: nothing could ever set a bit in `DbgChannels[]`, so freeldr's WARN and
/// TRACE were unreachable by any boot option rather than merely off by default.
/// (ERR/FIXME were separately silenced by the dropped `DBG_DEFAULT_LEVELS` term
/// in DbgPrint2, so between the two, freeldr printed nothing at all.)
///
/// Mutates `Value` in place exactly as the C does — it overwrites the ',' and
/// the '+'/'-' with NULs to split the string, which is why the parameter is
/// PCHAR and not PCSTR. Syntax is `level+channel` / `level-channel`, comma
/// separated, with an empty level meaning MAX_LEVEL.
#[no_mangle]
pub unsafe extern "C" fn DbgParseDebugChannels(Value: PCHAR) {
    if Value.is_null() {
        return;
    }
    let mut str_ = Value as *mut i8;
    loop {
        let separator = c_strchr(str_, b',');
        if !separator.is_null() {
            *separator = 0;
        }

        let mut c = c_strchr(str_, b'+');
        if c.is_null() {
            c = c_strchr(str_, b'-');
        }

        if !c.is_null() {
            let op = *c;
            *c = 0;
            c = c.add(1);
            DbgAddDebugChannel(c, str_, op);
        }

        // C writes `str = separator + 1` unconditionally and relies on the
        // while-condition to stop; that forms NULL+1 on the last pass, which is
        // UB in Rust as well as C. Same control flow, without the bad pointer.
        if separator.is_null() {
            break;
        }
        str_ = separator.add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn MsgBoxPrint(Format: *const i8) -> u32 {
    // DBG off in release; variadic MsgBox not on the boot-to-shell path.
    let _ = Format;
    0
}

#[no_mangle]
pub unsafe extern "system" fn KeBugCheckEx(
    BugCheckCode: ULONG,
    BugCheckParameter1: ULONG_PTR,
    BugCheckParameter2: ULONG_PTR,
    BugCheckParameter3: ULONG_PTR,
    BugCheckParameter4: ULONG_PTR,
) -> ! {

    let mut Buffer: [i8; 70] = [0; 70];

    sprintf(
        Buffer.as_mut_ptr(),
        b"*** STOP: 0x%08lX (0x%p,0x%p,0x%p,0x%p)\0".as_ptr() as *const i8,
        BugCheckCode,
        BugCheckParameter1 as PVOID,
        BugCheckParameter2 as PVOID,
        BugCheckParameter3 as PVOID,
        BugCheckParameter4 as PVOID,
    );

    UiMessageBoxCritical(Buffer.as_ptr());
    
    loop {}
}

extern "C" {
    // i386bug.rs / arm64_debug.rs (same decl as pcmem.rs). The Rust
    // definition prints its Format verbatim, so it is handed the message
    // already formatted, with no further arguments.
    fn FrLdrBugCheckWithMessage(BugCode: ULONG, File: PCHAR, Line: ULONG, Format: PCHAR, ...);
}

// Upstream b0bd092c0d4 "[FREELDR] Make ASSERT bugcheck on failure": after the
// debug print, an assertion is a FrLdrBugCheckWithMessage(ASSERT_FAILURE, ...)
// halt, not a DbgBreakPoint that boot then carries on from.
unsafe fn assert_bugcheck(FileName: PVOID, LineNumber: ULONG, msg: &[i8; 512]) {
    FrLdrBugCheckWithMessage(
        crate::frldr::ASSERT_FAILURE,
        FileName as PCHAR,
        LineNumber,
        msg.as_ptr() as PCHAR,
    );
}

#[no_mangle]
pub unsafe extern "system" fn RtlAssert(
    FailedAssertion: PVOID,
    FileName: PVOID,
    LineNumber: ULONG,
    Message: PCHAR,
) {
    // Format BEFORE printing. This predates format_slots_z and stays as it is:
    // sprintf is variadic here and the argument list is fixed and known, so
    // there is nothing for the slot-capture path to add. (Historically this
    // mattered because DbgPrint printed its format RAW, so every assertion
    // freeldr ever raised reported `Assertion '%s' failed at %s line %lu` with
    // no expression, file or line -- the one diagnostic that said nothing at
    // the moment it fired. DbgPrint formats properly now, but a direct sprintf
    // is still the clearer expression of a fixed argument list.)
    let mut buf: [i8; 512] = [0; 512];
    if !Message.is_null() {
        sprintf(
            buf.as_mut_ptr(),
            b"Assertion \'%s\' failed at %s line %lu: %s\n\0".as_ptr() as *const i8,
            FailedAssertion,
            FileName,
            LineNumber,
            Message,
        );
        DbgPrintSz(buf.as_ptr() as PCSTR);
        assert_bugcheck(FileName, LineNumber, &buf);
    } else {
        sprintf(
            buf.as_mut_ptr(),
            b"Assertion \'%s\' failed at %s line %lu\n\0".as_ptr() as *const i8,
            FailedAssertion,
            FileName,
            LineNumber,
        );
        DbgPrintSz(buf.as_ptr() as PCSTR);
        assert_bugcheck(FileName, LineNumber, &buf);
    }

    DbgBreakPoint();
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *  Copyright (C) 1998-2003  Brian Palmer  <brianp@sginet.com>
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #include <freeldr.h>
// #include <debug.h>
//
// #if DBG
//
// // #define DEBUG_ALL
// // #define DEBUG_WARN
// // #define DEBUG_ERR
// // #define DEBUG_INIFILE
// // #define DEBUG_REACTOS
// // #define DEBUG_CUSTOM
// #define DEBUG_NONE
//
// #define DBG_DEFAULT_LEVELS (ERR_LEVEL|FIXME_LEVEL)
//
// static UCHAR DbgChannels[DBG_CHANNELS_COUNT];
//
// #define SCREEN  1
// #define RS232   2
// #define BOCHS   4
//
// #define BOCHS_OUTPUT_PORT   0xE9
//
// ULONG DebugPort = RS232;
//
// /* Serial debug connection */
// ULONG BaudRate = 115200;
//
// ULONG ComPort  = 0; // The COM port initializer chooses the first available port starting from COM4 down to COM1.
// ULONG PortIrq  = 0; // Not used at the moment.
//
// BOOLEAN DebugStartOfLine = TRUE;
//
// #ifdef UEFIBOOT
// VOID
// ARMWriteToUART(UCHAR Data);
// #endif
//
// VOID
// DebugInit(
//     _In_ PCSTR DebugString)
// {
//     static BOOLEAN Initialized = FALSE;
//     PSTR CommandLine, PortString, BaudString, IrqString;
//     ULONG Value;
//     CHAR DbgStringBuffer[256];
//
//     /* Always reset the debugging channels */
//
// #if defined (DEBUG_ALL)
//     memset(DbgChannels, MAX_LEVEL, DBG_CHANNELS_COUNT);
// #elif defined (DEBUG_WARN)
//     memset(DbgChannels, WARN_LEVEL|FIXME_LEVEL|ERR_LEVEL, DBG_CHANNELS_COUNT);
// #elif defined (DEBUG_ERR)
//     memset(DbgChannels, ERR_LEVEL, DBG_CHANNELS_COUNT);
// #else
//     memset(DbgChannels, 0, DBG_CHANNELS_COUNT);
// #endif
//
// #if defined (DEBUG_INIFILE)
//     DbgChannels[DPRINT_INIFILE] = MAX_LEVEL;
// #elif defined (DEBUG_REACTOS)
//     DbgChannels[DPRINT_REACTOS] = MAX_LEVEL;
//     DbgChannels[DPRINT_REGISTRY] = MAX_LEVEL;
// #elif defined (DEBUG_CUSTOM)
//     DbgChannels[DPRINT_WARNING] = MAX_LEVEL;
//     DbgChannels[DPRINT_WINDOWS] = MAX_LEVEL;
// #endif
//
//     CommandLine = NULL;
//     if (!DebugString || !*DebugString)
//     {
//         /* No command-line is provided: during pre-initialization,
//          * initialize the debug port with default settings;
//          * otherwise just return during main initialization */
//         if (!Initialized)
//             goto Done;
//         return;
//     }
//
//     /* Get a copy of the command-line */
//     strcpy(DbgStringBuffer, DebugString);
//     CommandLine = DbgStringBuffer;
//
//     /* Upcase it */
//     _strupr(CommandLine);
//
//     /* Get the port and baud rate */
//     PortString = strstr(CommandLine, "DEBUGPORT");
//     BaudString = strstr(CommandLine, "BAUDRATE");
//     IrqString  = strstr(CommandLine, "IRQ");
//
//     /*
//      * Check if we got /DEBUGPORT parameters.
//      * NOTE: Inspired by reactos/ntoskrnl/kd/kdinit.c, KdInitSystem(...)
//      */
//     while (PortString)
//     {
//         /* Move past the actual string, to reach the port*/
//         PortString += strlen("DEBUGPORT");
//
//         /* Now get past any spaces and skip the equal sign */
//         while (*PortString == ' ') PortString++;
//         PortString++;
//
//         /* Check for possible ports and set the port to use */
//         if (strncmp(PortString, "SCREEN", 6) == 0)
//         {
//             PortString += 6;
//             DebugPort |= SCREEN;
//         }
//         else if (strncmp(PortString, "BOCHS", 5) == 0)
//         {
//             PortString += 5;
//             DebugPort |= BOCHS;
//         }
//         else if (strncmp(PortString, "COM", 3) == 0)
//         {
//             PortString += 3;
//             DebugPort |= RS232;
//
//             /* Set the port to use */
//             Value = atol(PortString);
//             if (Value) ComPort = Value;
//         }
//
//         PortString = strstr(PortString, "DEBUGPORT");
//    }
//
//     /* Check if we got a baud rate */
//     if (BaudString)
//     {
//         /* Move past the actual string, to reach the rate */
//         BaudString += strlen("BAUDRATE");
//
//         /* Now get past any spaces */
//         while (*BaudString == ' ') BaudString++;
//
//         /* And make sure we have a rate */
//         if (*BaudString)
//         {
//             /* Read and set it */
//             Value = atol(BaudString + 1);
//             if (Value) BaudRate = Value;
//         }
//     }
//
//     /* Check Serial Port Settings [IRQ] */
//     if (IrqString)
//     {
//         /* Move past the actual string, to reach the rate */
//         IrqString += strlen("IRQ");
//
//         /* Now get past any spaces */
//         while (*IrqString == ' ') IrqString++;
//
//         /* And make sure we have an IRQ */
//         if (*IrqString)
//         {
//             /* Read and set it */
//             Value = atol(IrqString + 1);
//             if (Value) PortIrq = Value;
//         }
//     }
//
// Done:
//     Initialized = TRUE;
//
//     /* Try to initialize the port; if it fails, remove the corresponding flag */
//     if (DebugPort & RS232)
//     {
//         if (!Rs232PortInitialize(ComPort, BaudRate))
//             DebugPort &= ~RS232;
//     }
// }
//
// VOID DebugPrintChar(UCHAR Character)
// {
//     if (Character == '\n')
//         DebugStartOfLine = TRUE;
//
//     if (DebugPort & RS232)
//     {
//         if (Character == '\n')
//             Rs232PortPutByte('\r');
//
//         Rs232PortPutByte(Character);
//     }
//     if (DebugPort & BOCHS)
//     {
//         WRITE_PORT_UCHAR((PUCHAR)BOCHS_OUTPUT_PORT, Character);
//     }
//     if (DebugPort & SCREEN)
//     {
//         MachConsPutChar(Character as i32);
//     }
// }
//
// ULONG
// DbgPrint(const char *Format, ...)
// {
//     va_list ap;
//     int Length;
//     char* ptr;
//     CHAR Buffer[512];
//
//     va_start(ap, Format);
//     Length = _vsnprintf(Buffer, sizeof(Buffer), Format, ap);
//     va_end(ap);
//
//     /* Check if we went past the buffer */
//     if (Length == -1)
//     {
//         /* Terminate it if we went over-board */
//         Buffer[sizeof(Buffer) - 1] = '\n';
//
//         /* Put maximum */
//         Length = sizeof(Buffer);
//     }
//
//     ptr = Buffer;
//     while (Length--)
//         DebugPrintChar(*ptr++);
//
//     return 0;
// }
//
// VOID
// DbgPrint2(ULONG Mask, ULONG Level, const char *File, ULONG Line, char *Format, ...)
// {
//     va_list ap;
//     char Buffer[2096];
//     char *ptr = Buffer;
//
//     /* Mask out unwanted debug messages */
//     if (!(DbgChannels[Mask] & Level) && !(Level & DBG_DEFAULT_LEVELS))
//     {
//         return;
//     }
//
//     /* Print the header if we have started a new line */
//     if (DebugStartOfLine)
//     {
//         DbgPrint("(%s:%lu) ", File, Line);
//
//         switch (Level)
//         {
//             case ERR_LEVEL:
//                 DbgPrint("err: ");
//                 break;
//             case FIXME_LEVEL:
//                 DbgPrint("fixme: ");
//                 break;
//             case WARN_LEVEL:
//                 DbgPrint("warn: ");
//                 break;
//             case TRACE_LEVEL:
//                 DbgPrint("trace: ");
//                 break;
//         }
//
//         DebugStartOfLine = FALSE;
//     }
//
//     va_start(ap, Format);
//     vsprintf(Buffer, Format, ap);
//     va_end(ap);
//
//     while (*ptr)
//     {
//         DebugPrintChar(*ptr++);
//     }
// }
//
// VOID
// DebugDumpBuffer(ULONG Mask, PVOID Buffer, ULONG Length)
// {
//     PUCHAR BufPtr = (PUCHAR)Buffer;
//     ULONG Offset, Count, i;
//
//     /* Mask out unwanted debug messages */
//     if (!(DbgChannels[Mask] & TRACE_LEVEL))
//         return;
//
//     DebugStartOfLine = FALSE; // We don't want line headers
//     DbgPrint("Dumping buffer at %p with length of %lu bytes:\n", Buffer, Length);
//
//     Offset = 0;
//     while (Offset < Length)
//     {
//         /* We don't want line headers */
//         DebugStartOfLine = FALSE;
//
//         /* Print the offset */
//         DbgPrint("%04x:\t", Offset);
//
//         /* Print either 16 or the remaining number of bytes */
//         Count = min(Length - Offset, 16);
//         for (i = 0; i < Count; i++, Offset++)
//         {
//             DbgPrint("%02x%c", BufPtr[Offset], (i == 7) ? '-' : ' ');
//         }
//
//         DbgPrint("\n");
//     }
// }
//
// VOID
// DebugDisableScreenPort(VOID)
// {
//     DebugPort &= ~SCREEN;
// }
//
// static BOOLEAN
// DbgAddDebugChannel(CHAR* channel, CHAR* level, CHAR op)
// {
//     int iLevel, iChannel;
//
//     if (channel == NULL || *channel == '\0' || strlen(channel) == 0)
//         return FALSE;
//
//     if (level == NULL || *level == '\0' || strlen(level) == 0)
//         iLevel = MAX_LEVEL;
//     else if (strcmp(level, "err") == 0)
//         iLevel = ERR_LEVEL;
//     else if (strcmp(level, "fixme") == 0)
//         iLevel = FIXME_LEVEL;
//     else if (strcmp(level, "warn") == 0)
//         iLevel = WARN_LEVEL;
//     else if (strcmp(level, "trace") == 0)
//         iLevel = TRACE_LEVEL;
//     else
//         return FALSE;
//
//          if (strcmp(channel, "memory"    ) == 0) iChannel = DPRINT_MEMORY;
//     else if (strcmp(channel, "filesystem") == 0) iChannel = DPRINT_FILESYSTEM;
//     else if (strcmp(channel, "inifile"   ) == 0) iChannel = DPRINT_INIFILE;
//     else if (strcmp(channel, "ui"        ) == 0) iChannel = DPRINT_UI;
//     else if (strcmp(channel, "disk"      ) == 0) iChannel = DPRINT_DISK;
//     else if (strcmp(channel, "cache"     ) == 0) iChannel = DPRINT_CACHE;
//     else if (strcmp(channel, "registry"  ) == 0) iChannel = DPRINT_REGISTRY;
//     else if (strcmp(channel, "linux"     ) == 0) iChannel = DPRINT_LINUX;
//     else if (strcmp(channel, "hwdetect"  ) == 0) iChannel = DPRINT_HWDETECT;
//     else if (strcmp(channel, "windows"   ) == 0) iChannel = DPRINT_WINDOWS;
//     else if (strcmp(channel, "peloader"  ) == 0) iChannel = DPRINT_PELOADER;
//     else if (strcmp(channel, "scsiport"  ) == 0) iChannel = DPRINT_SCSIPORT;
//     else if (strcmp(channel, "heap"      ) == 0) iChannel = DPRINT_HEAP;
//     else if (strcmp(channel, "all"       ) == 0)
//     {
//         int i;
//
//         for (i = 0; i < DBG_CHANNELS_COUNT; i++)
//         {
//             if (op == '+')
//                 DbgChannels[i] |= iLevel as u8;
//             else
//                 DbgChannels[i] &= ~iLevel;
//         }
//
//         return TRUE;
//     }
//     else return FALSE;
//
//     if (op == '+')
//         DbgChannels[iChannel] |= iLevel as u8;
//     else
//         DbgChannels[iChannel] &= ~iLevel;
//
//     return TRUE;
// }
//
// VOID
// DbgParseDebugChannels(PCHAR Value)
// {
//     CHAR *str, *separator, *c, op;
//
//     str = Value;
//
//     do
//     {
//         separator = strchr(str, ',');
//         if (separator != NULL)
//             *separator = '\0';
//
//         c = strchr(str, '+');
//         if (c == NULL)
//             c = strchr(str, '-');
//
//         if (c != NULL)
//         {
//             op = *c;
//             *c = '\0';
//             c++;
//
//             DbgAddDebugChannel(c, str, op);
//         }
//
//         str = separator + 1;
//     } while (separator != NULL);
// }
//
// #else
//
// #undef DebugInit
// VOID
// DebugInit(
//     _In_ PCSTR DebugString)
// {
//     UNREFERENCED_PARAMETER(DebugString);
// }
//
// ULONG
// DbgPrint(PCCH Format, ...)
// {
//     UNREFERENCED_PARAMETER(Format);
//     return 0;
// }
//
// VOID
// DbgPrint2(ULONG Mask, ULONG Level, const char *File, ULONG Line, char *Format, ...)
// {
//     UNREFERENCED_PARAMETER(Mask);
//     UNREFERENCED_PARAMETER(Level);
//     UNREFERENCED_PARAMETER(File);
//     UNREFERENCED_PARAMETER(Line);
//     UNREFERENCED_PARAMETER(Format);
// }
//
// VOID
// DebugDumpBuffer(ULONG Mask, PVOID Buffer, ULONG Length)
// {
//     UNREFERENCED_PARAMETER(Mask);
//     UNREFERENCED_PARAMETER(Buffer);
//     UNREFERENCED_PARAMETER(Length);
// }
//
// #undef DbgParseDebugChannels
// VOID
// DbgParseDebugChannels(PCHAR Value)
// {
//     UNREFERENCED_PARAMETER(Value);
// }
//
// #endif // DBG
//
// ULONG
// MsgBoxPrint(const char *Format, ...)
// {
//     va_list ap;
//     CHAR Buffer[512];
//     ULONG Length;
//
//     va_start(ap, Format);
//
//     /* Construct a string */
//     Length = _vsnprintf(Buffer, 512, Format, ap);
//
//     /* Check if we went past the buffer */
//     if (Length == MAXULONG)
//     {
//         /* Terminate it if we went over-board */
//         Buffer[sizeof(Buffer) - 1] = '\n';
//
//         /* Put maximum */
//         Length = sizeof(Buffer);
//     }
//
//     /* Show it as a message box */
//     UiMessageBox(Buffer);
//
//     /* Cleanup and exit */
//     va_end(ap);
//     return 0;
// }
//
// DECLSPEC_NORETURN
// VOID
// NTAPI
// KeBugCheckEx(
//     IN ULONG  BugCheckCode,
//     IN ULONG_PTR  BugCheckParameter1,
//     IN ULONG_PTR  BugCheckParameter2,
//     IN ULONG_PTR  BugCheckParameter3,
//     IN ULONG_PTR  BugCheckParameter4)
// {
//     char Buffer[70];
//
//     sprintf(Buffer,
//             "*** STOP: 0x%08lX (0x%p,0x%p,0x%p,0x%p)",
//             BugCheckCode,
//             (PVOID)BugCheckParameter1,
//             (PVOID)BugCheckParameter2,
//             (PVOID)BugCheckParameter3,
//             (PVOID)BugCheckParameter4);
//
//     UiMessageBoxCritical(Buffer);
//     ASSERT(FALSE);
//     for (;;);
// }
//
// VOID
// NTAPI
// RtlAssert(IN PVOID FailedAssertion,
//           IN PVOID FileName,
//           IN ULONG LineNumber,
//           IN PCHAR Message OPTIONAL)
// {
//     if (Message)
//     {
//         DbgPrint("Assertion \'%s\' failed at %s line %lu: %s\n",
//                  (PCHAR)FailedAssertion,
//                  (PCHAR)FileName,
//                  LineNumber,
//                  Message);
//     }
//     else
//     {
//         DbgPrint("Assertion \'%s\' failed at %s line %lu\n",
//                  (PCHAR)FailedAssertion,
//                  (PCHAR)FileName,
//                  LineNumber);
//     }
//
//     DbgBreakPoint();
// }
//
// char *BugCodeStrings[] =
// {
//     "TEST_BUGCHECK",
//     "MISSING_HARDWARE_REQUIREMENTS",
//     "FREELDR_IMAGE_CORRUPTION",
//     "MEMORY_INIT_FAILURE",
// #ifdef UEFIBOOT
//     "EXIT_BOOTSERVICES_FAILURE",
// #endif
// };
//
// ULONG_PTR BugCheckInfo[5];
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
