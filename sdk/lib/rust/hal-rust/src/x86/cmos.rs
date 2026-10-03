//! CMOS access — 8-bit RTC + ESCD register file at I/O ports 0x70/0x71.
//!
//! Translated from `hal/halx86/generic/cmos.c`.  Two responsibilities:
//!
//! 1. Low-level read/write helpers (`HalpReadCmos`, `HalpWriteCmos`,
//!    `HalpGetCmosData`, `HalpSetCmosData`) used by spinlock.c, timer.c,
//!    rtctimer.c, and the upcoming HAL bus drivers.
//! 2. Wall-clock + LastKnownGood RTC services exported to the kernel
//!    (`HalQueryRealTimeClock`, `HalSetRealTimeClock`,
//!    `HalGetEnvironmentVariable`, `HalSetEnvironmentVariable`).
//!
//! All read/write helpers expect the caller to already hold the CMOS
//! spinlock (`HalpAcquireCmosSpinLock` from spinlock.c); the higher-level
//! Hal* exports take it themselves.

use ntkrnl_sys::hal_ddk::{
    HalpAcquireCmosSpinLock, HalpReleaseCmosSpinLock, HalpBusType, BOOLEAN, MACHINE_TYPE_ISA,
    PCH, PTIME_FIELDS, PVOID, UCHAR, ULONG, USHORT,
};

// ARC return codes (sdk/include/reactos/arc/arc.h enum _ARC_CODES).
type ArcStatus = ULONG;
const ESUCCESS: ArcStatus = 0;
const ENOENT: ArcStatus = 14; // nolint: const-contradiction (ARC_CODES enumerator, arc/arc.h:46; arc.h #undefs the CRT errno ENOENT)
const ENOMEM: ArcStatus = 16; // nolint: const-contradiction (ARC_CODES enumerator, arc/arc.h:48; arc.h #undefs the CRT errno ENOMEM)

// CMOS / RTC port + register layout (hal/halx86/include/halhw.h).
const CMOS_CONTROL_PORT: u16 = 0x70;
const CMOS_DATA_PORT: u16 = 0x71;
const RTC_REGISTER_A: u8 = 0x0A;
const RTC_REGISTER_B: u8 = 0x0B;
const RTC_REG_A_UIP: u8 = 0x80; // Update-in-progress flag

// Mirrors UCHAR HalpCmosCenturyOffset; in the C source.  Initialised in
// HalpInitializeCmos to 50 (FIXME notes in the C: never used because we
// always heuristically split at year > 80, but kept as the canonical
// global other code may consult later).
#[unsafe(no_mangle)]
pub static mut HalpCmosCenturyOffset: UCHAR = 0;

// ---------------------------------------------------------------------------
// Local port-I/O helpers (mirrors the inline pattern in beep.rs / reboot.rs).
// On i386, exported PORT_* primitives in portio.rs cover the same job; on
// amd64, READ_PORT_UCHAR / WRITE_PORT_UCHAR are macros that expand to
// __inbyte / __outbyte intrinsics — no exported function exists.  Inline asm
// works on both arches.
// ---------------------------------------------------------------------------

#[inline(always)]
unsafe fn outbyte(port: u16, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
unsafe fn inbyte(port: u16) -> u8 {
    let value: u8;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            in("dx") port,
            out("al") value,
            options(nostack, preserves_flags),
        );
    }
    value
}

// ---------------------------------------------------------------------------
// BCD <-> binary helpers (FORCEINLINE in halp.h).
// ---------------------------------------------------------------------------

#[inline(always)]
fn bcd_int(bcd: u8) -> u8 {
    ((bcd & 0xF0) >> 4) * 10 + (bcd & 0x0F)
}

#[inline(always)]
fn int_bcd(int: i16) -> u8 {
    (((int / 10) << 4) + (int % 10)) as u8
}

// ---------------------------------------------------------------------------
// Tiny ASCII string helpers.  Only "LastKnownGood" / "TRUE" / "FALSE" —
// keeps us from pulling libcntpr's _stricmp / strncpy across the FFI.
// ---------------------------------------------------------------------------

/// Case-insensitive ASCII compare, NUL-terminated haystack vs static needle.
/// Returns true on match.  Mirrors `_stricmp(a, b) == 0`.
unsafe fn stricmp_ascii(a: *const i8, b: &[u8]) -> bool {
    if a.is_null() {
        return false;
    }
    unsafe {
        let mut p = a as *const u8;
        for &nb in b {
            let mut hb = *p;
            if hb == 0 {
                return false;
            }
            // Lowercase ASCII A-Z.
            if (b'A'..=b'Z').contains(&hb) {
                hb += 32;
            }
            let mut nb_low = nb;
            if (b'A'..=b'Z').contains(&nb_low) {
                nb_low += 32;
            }
            if hb != nb_low {
                return false;
            }
            p = p.add(1);
        }
        // Both terminators (or implicit terminator at end of needle).
        *p == 0
    }
}

/// `strncpy(dst, src, n)` for a static ASCII source, no NUL-padding past the
/// last byte written.  C strncpy zero-fills the rest of the buffer; the
/// callers in cmos.c don't rely on that, so we skip it.
unsafe fn strncpy_ascii(dst: *mut i8, src: &[u8], n: usize) {
    if dst.is_null() {
        return;
    }
    unsafe {
        let mut p = dst as *mut u8;
        let mut written = 0usize;
        for &b in src {
            if written >= n {
                return;
            }
            *p = b;
            p = p.add(1);
            written += 1;
        }
        if written < n {
            *p = 0;
        }
    }
}

// ---------------------------------------------------------------------------
// Low-level CMOS access (NTAPI — caller holds HalpSystemHardwareLock).
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpReadCmos(Reg: UCHAR) -> UCHAR {
    unsafe {
        outbyte(CMOS_CONTROL_PORT, 0x80 | Reg);
        inbyte(CMOS_DATA_PORT)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpWriteCmos(Reg: UCHAR, Value: UCHAR) {
    unsafe {
        outbyte(CMOS_CONTROL_PORT, 0x80 | Reg);
        outbyte(CMOS_DATA_PORT, Value);
    }
}

// ---------------------------------------------------------------------------
// Bulk CMOS access — NTAPI, takes the spinlock itself.
//
// BusNumber 0: 8-bit register space (0x00..0xFF), BusNumber 1: 16-bit
// register space (0x0000..0xFFFF).  Both still go through the same 8-bit
// register-select port 0x70 — the Address upper bits are technically
// chipset-dependent, but the C version just truncates to UCHAR for the
// register-select and lets the I/O find what it finds.  Mirror the C.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpGetCmosData(
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    if Length == 0 {
        return 0;
    }
    unsafe {
        let mut ptr = Buffer as *mut u8;
        let mut address = SlotNumber;
        let mut len = Length;

        HalpAcquireCmosSpinLock();

        if BusNumber == 0 {
            while len > 0 && address < 0x100 {
                *ptr = HalpReadCmos(address as UCHAR);
                ptr = ptr.add(1);
                address += 1;
                len -= 1;
            }
        } else if BusNumber == 1 {
            while len > 0 && address < 0x10000 {
                *ptr = HalpReadCmos(address as UCHAR);
                ptr = ptr.add(1);
                address += 1;
                len -= 1;
            }
        }

        HalpReleaseCmosSpinLock();
        Length - len
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpSetCmosData(
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    if Length == 0 {
        return 0;
    }
    unsafe {
        let mut ptr = Buffer as *const u8;
        let mut address = SlotNumber;
        let mut len = Length;

        HalpAcquireCmosSpinLock();

        if BusNumber == 0 {
            while len > 0 && address < 0x100 {
                HalpWriteCmos(address as UCHAR, *ptr);
                ptr = ptr.add(1);
                address += 1;
                len -= 1;
            }
        } else if BusNumber == 1 {
            while len > 0 && address < 0x10000 {
                HalpWriteCmos(address as UCHAR, *ptr);
                ptr = ptr.add(1);
                address += 1;
                len -= 1;
            }
        }

        HalpReleaseCmosSpinLock();
        Length - len
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpInitializeCmos() {
    unsafe {
        core::ptr::write_volatile(&raw mut HalpCmosCenturyOffset, 50);
        // C: ASSERT(HalpBusType == MACHINE_TYPE_ISA).  We only ship for ISA
        // PC-AT; debug_assert keeps it free in release.
        ntkrnl_sys::ASSERT!(core::ptr::read_volatile(&raw const HalpBusType) == MACHINE_TYPE_ISA as ULONG);
    }
}

// ---------------------------------------------------------------------------
// Public ARC environment variable interface — only "LastKnownGood" honored.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalGetEnvironmentVariable(
    Name: PCH,
    ValueLength: USHORT,
    Value: PCH,
) -> ArcStatus {
    unsafe {
        if !stricmp_ascii(Name, b"LastKnownGood") {
            return ENOENT;
        }
        HalpAcquireCmosSpinLock();
        let val = HalpReadCmos(RTC_REGISTER_B) & 0x01;
        HalpReleaseCmosSpinLock();
        if val != 0 {
            strncpy_ascii(Value, b"FALSE", ValueLength as usize);
        } else {
            strncpy_ascii(Value, b"TRUE", ValueLength as usize);
        }
        ESUCCESS
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetEnvironmentVariable(Name: PCH, Value: PCH) -> ArcStatus {
    unsafe {
        if !stricmp_ascii(Name, b"LastKnownGood") {
            return ENOMEM;
        }
        let val: u8;
        if stricmp_ascii(Value, b"TRUE") {
            HalpAcquireCmosSpinLock();
            val = HalpReadCmos(RTC_REGISTER_B) | 1;
        } else if stricmp_ascii(Value, b"FALSE") {
            HalpAcquireCmosSpinLock();
            val = HalpReadCmos(RTC_REGISTER_B) & !1;
        } else {
            return ENOMEM;
        }
        HalpWriteCmos(RTC_REGISTER_B, val);
        HalpReleaseCmosSpinLock();
        ESUCCESS
    }
}

// ---------------------------------------------------------------------------
// Real-time clock — BCD reads from the standard MC146818 register file.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalQueryRealTimeClock(Time: PTIME_FIELDS) -> BOOLEAN {
    unsafe {
        HalpAcquireCmosSpinLock();
        // Spin until the chip finishes its update tick.  CMOS reads during UIP
        // can return torn fields.
        while (HalpReadCmos(RTC_REGISTER_A) & RTC_REG_A_UIP) != 0 {}
        let t = &mut *Time;
        t.Second = bcd_int(HalpReadCmos(0)) as i16;
        t.Minute = bcd_int(HalpReadCmos(2)) as i16;
        t.Hour = bcd_int(HalpReadCmos(4)) as i16;
        t.Weekday = bcd_int(HalpReadCmos(6)) as i16;
        t.Day = bcd_int(HalpReadCmos(7)) as i16;
        t.Month = bcd_int(HalpReadCmos(8)) as i16;
        let year = bcd_int(HalpReadCmos(9)) as i16;
        t.Milliseconds = 0;
        // Crude century split — preserves the C heuristic; the FIXME about
        // the actual century byte applies here just like in the C source.
        t.Year = year + if year > 80 { 1900 } else { 2000 };
        HalpReleaseCmosSpinLock();
        1 /* TRUE */
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetRealTimeClock(Time: PTIME_FIELDS) -> BOOLEAN {
    unsafe {
        HalpAcquireCmosSpinLock();
        while (HalpReadCmos(RTC_REGISTER_A) & RTC_REG_A_UIP) != 0 {}
        let t = &*Time;
        HalpWriteCmos(0, int_bcd(t.Second));
        HalpWriteCmos(2, int_bcd(t.Minute));
        HalpWriteCmos(4, int_bcd(t.Hour));
        HalpWriteCmos(6, int_bcd(t.Weekday));
        HalpWriteCmos(7, int_bcd(t.Day));
        HalpWriteCmos(8, int_bcd(t.Month));
        HalpWriteCmos(9, int_bcd(t.Year % 100));
        // FIXME: century byte (mirrors the C).
        HalpReleaseCmosSpinLock();
        1 /* TRUE */
    }
}
