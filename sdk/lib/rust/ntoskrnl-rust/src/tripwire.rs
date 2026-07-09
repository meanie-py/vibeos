//! vibeos tripwire channel — per-boot debug knobs delivered via QEMU fw_cfg.
//!
//! The harness passes `-fw_cfg name=opt/org.vibeos.tripwire,string=<cfg>` and the
//! kernel reads it during MmArmInitSystem Phase 0, *before* system-PTE sizing and
//! MiInitializeSpecialPool. No boot media rebuild, works on every backend
//! (native / windows-host TCG / WHPX). On real hardware or a QEMU run without the
//! knob this is a no-op (fw_cfg signature probe fails / file absent).
//!
//! Config grammar (space/semicolon-separated KEY=VAL tokens, no commas — QEMU's
//! -fw_cfg parser owns the comma):
//!
//!   SPECIALPOOL=<tag>   arm special pool for a 4-char pool tag, written as it
//!                       DISPLAYS in pool dumps (e.g. Gla5, Usqu) — i.e. the
//!                       little-endian byte order, same convention as verifier's
//!                       registry PoolTag value. `*` = every eligible allocation.
//!   SPECIALPOOL_UNDERRUNS=1   catch underruns instead of overruns.
//!   SPECIALPOOL_SELFTEST=pattern|guard   deliberately corrupt one allocation
//!                       after pool init and expect BUGCHECK 0xC1 (pattern) or
//!                       0x50 (guard page) — end-to-end detection proof.
//!   CSWITCH_SELFTEST=1  prove cswitchdep barks (needs --cswitchdep build).
//!   RUSTPANIC_SELFTEST=1  deliberate Rust panic!() at arm time — proves the
//!                       shared panic handler (panic-common): expect the
//!                       "*** RUST PANIC in ntoskrnl …" report + backtrace,
//!                       then BUGCHECK 0xE0525354 (RUST_PANIC).
//!
//! Transports: x86 (i386/amd64) = port I/O 0x510/0x511; arm64 = MMIO at
//! 0x09020000 on the QEMU virt machine, reached through the early-boot identity
//! map (same mechanism the pre-Phase1 UART writes to phys 0x09000000 use — the
//! whole 0x09000000 2MB device block is identity-mapped during MmArmInitSystem
//! Phase 0). The signature probe guards non-QEMU/real hardware.

#![allow(non_snake_case)]

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;

// Same-crate globals — no FFI decls needed.
use crate::ex::pool::globals::MmSpecialPoolTag;
use crate::ARM3::special::MmSpecialPoolCatchOverruns;

extern "C" {
    fn DbgPrint(Format: *const u8, ...) -> u32;
}

/// Selftest mode parsed from the tripwire blob: 0 none, 1 pattern, 2 guard.
static mut VIBE_SELFTEST: u8 = 0;
/// CSWITCH_SELFTEST=1 armed via fw_cfg (consumed by `KiStartSyncCensus`).
pub static mut CSWITCH_SELFTEST_ARMED: bool = false;

const TRIPWIRE_FILE: &[u8] = b"opt/org.vibeos.tripwire";
const CFG_MAX: usize = 256;

// ---------------------------------------------------------------------------
// fw_cfg transport (x86: ports 0x510 selector / 0x511 data;
//                   arm64: MMIO 0x09020000 — data @+0, selector @+8 big-endian)
// ---------------------------------------------------------------------------

mod fwcfg {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    use core::arch::asm;

    const FW_CFG_SIGNATURE: u16 = 0x0000;
    const FW_CFG_FILE_DIR: u16 = 0x0019;

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe fn select(key: u16) {
        const PORT_SEL: u16 = 0x510;
        asm!("out dx, ax", in("dx") PORT_SEL, in("ax") key, options(nomem, nostack));
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe fn read_u8() -> u8 {
        const PORT_DATA: u16 = 0x511;
        let v: u8;
        asm!("in al, dx", in("dx") PORT_DATA, out("al") v, options(nomem, nostack));
        v
    }

    // QEMU virt fw_cfg MMIO block, reached via the early-boot identity map
    // (proven live for this 2MB device block by the pre-Phase1 UART writes to
    // phys 0x09000000). Selector is BIG-ENDIAN on the MMIO transport; the data
    // register streams successive bytes on repeated 1-byte reads.
    #[cfg(target_arch = "aarch64")]
    const FW_CFG_MMIO_BASE: usize = 0x0902_0000;

    #[cfg(target_arch = "aarch64")]
    unsafe fn select(key: u16) {
        core::ptr::write_volatile((FW_CFG_MMIO_BASE + 8) as *mut u16, key.to_be());
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn read_u8() -> u8 {
        core::ptr::read_volatile(FW_CFG_MMIO_BASE as *const u8)
    }

    unsafe fn read_into(buf: &mut [u8]) {
        for b in buf.iter_mut() {
            *b = read_u8();
        }
    }

    unsafe fn skip(n: usize) {
        for _ in 0..n {
            read_u8();
        }
    }

    unsafe fn read_be32() -> u32 {
        let mut b = [0u8; 4];
        read_into(&mut b);
        u32::from_be_bytes(b)
    }

    /// Read the named fw_cfg file into `out`; returns the byte count read, or
    /// None if fw_cfg is absent (not QEMU / signature mismatch) or no such file.
    pub unsafe fn read_file(name: &[u8], out: &mut [u8]) -> Option<usize> {
        // Probe the signature: "QEMU". Real hardware floats the port (0xFF).
        select(FW_CFG_SIGNATURE);
        let mut sig = [0u8; 4];
        read_into(&mut sig);
        if &sig != b"QEMU" {
            return None;
        }

        // Walk the file directory: be32 count, then 64-byte entries of
        // { be32 size, be16 select, u16 reserved, char name[56] }.
        select(FW_CFG_FILE_DIR);
        let count = read_be32();
        if count > 0x1000 {
            return None; // implausible — treat as absent
        }
        for _ in 0..count {
            let size = read_be32() as usize;
            let mut sel_b = [0u8; 2];
            read_into(&mut sel_b);
            let sel = u16::from_be_bytes(sel_b);
            skip(2); // reserved
            let mut fname = [0u8; 56];
            read_into(&mut fname);

            let nul = fname.iter().position(|&c| c == 0).unwrap_or(56);
            if &fname[..nul] == name {
                // Found it — select and stream out the contents.
                select(sel);
                let n = core::cmp::min(size, out.len());
                read_into(&mut out[..n]);
                return Some(n);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Config parsing
// ---------------------------------------------------------------------------

fn token_val<'a>(tok: &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
    if tok.len() > key.len() && &tok[..key.len()] == key && tok[key.len()] == b'=' {
        Some(&tok[key.len() + 1..])
    } else {
        None
    }
}

unsafe fn apply_token(tok: &[u8]) {
    if let Some(val) = token_val(tok, b"SPECIALPOOL") {
        let tag: ULONG = if val == b"*" {
            b'*' as ULONG
        } else if val.len() == 4 {
            // Tag written as displayed in pool dumps => little-endian bytes.
            u32::from_le_bytes([val[0], val[1], val[2], val[3]])
        } else {
            DbgPrint(
                b"[TRIPWIRE] ERROR: SPECIALPOOL tag must be 4 chars or '*' (got %lu bytes)\n\0"
                    .as_ptr(),
                val.len() as u32,
            );
            return;
        };
        MmSpecialPoolTag = tag;
        DbgPrint(
            b"[TRIPWIRE] special pool armed: tag=0x%08lX (%c%c%c%c)\n\0".as_ptr(),
            tag,
            (tag & 0xFF) as u32,
            ((tag >> 8) & 0xFF) as u32,
            ((tag >> 16) & 0xFF) as u32,
            ((tag >> 24) & 0xFF) as u32,
        );
    } else if let Some(val) = token_val(tok, b"SPECIALPOOL_UNDERRUNS") {
        if val == b"1" {
            MmSpecialPoolCatchOverruns = 0;
            DbgPrint(b"[TRIPWIRE] special pool mode: catch UNDERRUNS\n\0".as_ptr());
        }
    } else if let Some(val) = token_val(tok, b"SPECIALPOOL_SELFTEST") {
        VIBE_SELFTEST = match val {
            b"pattern" => 1,
            b"guard" => 2,
            _ => {
                DbgPrint(b"[TRIPWIRE] ERROR: unknown SPECIALPOOL_SELFTEST value\n\0".as_ptr());
                0
            }
        };
    } else if let Some(val) = token_val(tok, b"CSWITCH_SELFTEST") {
        // cswitchdep-design.md: prove the detector barks — deliberately hold a
        // spinlock across a (simulated) voluntary switch at census start and
        // expect SLEEP_WITH_SPINLOCK. Non-destructive; runs once at PASSIVE.
        if val == b"1" {
            CSWITCH_SELFTEST_ARMED = true;
            #[cfg(feature = "cswitchdep")]
            DbgPrint(b"[TRIPWIRE] cswitchdep selftest armed\n\0".as_ptr());
            // Fail loud, never silently degrade: armed without the detector
            // in the build is a config error, not a no-op.
            #[cfg(not(feature = "cswitchdep"))]
            DbgPrint(
                b"[TRIPWIRE] ERROR: CSWITCH_SELFTEST armed but cswitchdep NOT built in (need --cswitchdep)\n\0"
                    .as_ptr(),
            );
        }
    } else if let Some(val) = token_val(tok, b"RUSTPANIC_SELFTEST") {
        // crash-value lane: prove the shared panic handler barks — deliberate
        // panic!() with a formatted message. Expect on serial: the full
        // "*** RUST PANIC in ntoskrnl at …tripwire.rs:…" report (message +
        // location + raw-PC backtrace) followed by BUGCHECK 0xE0525354
        // (panic_common::RUST_PANIC). Destructive by design — the boot dies.
        if val == b"1" {
            DbgPrint(
                b"[TRIPWIRE] rust-panic selftest: panicking NOW (expect *** RUST PANIC + BUGCHECK 0xE0525354)\n\0"
                    .as_ptr(),
            );
            panic!("rust-panic selftest: deliberate panic via tripwire (answer={})", 42);
        }
    } else if !tok.is_empty() {
        DbgPrint(b"[TRIPWIRE] ERROR: unknown tripwire token (ignored)\n\0".as_ptr());
    }
}

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

/// Called at the top of MmArmInitSystem Phase 0 — before system-PTE sizing and
/// MiInitializeSpecialPool, so an armed MmSpecialPoolTag influences both.
pub unsafe fn vibe_tripwire_init() {
    let mut cfg = [0u8; CFG_MAX];
    let Some(n) = fwcfg::read_file(TRIPWIRE_FILE, &mut cfg) else {
        return; // no fw_cfg / no tripwire file — normal boot
    };

    DbgPrint(b"[TRIPWIRE] config received (%lu bytes)\n\0".as_ptr(), n as u32);

    for tok in cfg[..n].split(|&c| c == b' ' || c == b';' || c == b'\n' || c == b'\0') {
        if !tok.is_empty() {
            apply_token(tok);
        }
    }
}

/// FFI lookup for OTHER kernel crates (crash-value: kd64-rust's `KD=COM2`
/// opt-in runs at KdInitSystem time, long BEFORE `vibe_tripwire_init`).
/// Re-reads the fw_cfg tripwire blob (cheap port/MMIO I/O, side-effect-free)
/// and reports whether an exact space/semicolon-separated token is present.
/// Returns 0 on non-QEMU / no tripwire file.
#[no_mangle]
pub unsafe extern "system" fn VibeTripwireHasToken(needle: *const u8, len: usize) -> u8 {
    if needle.is_null() || len == 0 {
        return 0;
    }
    let mut cfg = [0u8; CFG_MAX];
    let Some(n) = fwcfg::read_file(TRIPWIRE_FILE, &mut cfg) else {
        return 0;
    };
    let needle = core::slice::from_raw_parts(needle, len);
    for tok in cfg[..n].split(|&c| c == b' ' || c == b';' || c == b'\n' || c == b'\0') {
        if tok == needle {
            return 1;
        }
    }
    0
}

/// Called right after MiInitializeSpecialPool. Reports arming outcome loudly and
/// runs the destructive selftest when requested (expected result: BUGCHECK).
pub unsafe fn vibe_tripwire_post_pool_init() {
    use crate::ARM3::special::{MmAllocateSpecialPool, MmFreeSpecialPool, MmSpecialPoolStart};

    if MmSpecialPoolTag == 0 || MmSpecialPoolTag == u32::MAX {
        return;
    }

    // Fail loud: the tag was armed but init didn't produce a pool.
    if MmSpecialPoolStart.is_null() {
        DbgPrint(b"[TRIPWIRE] ERROR: special pool armed but NOT initialized\n\0".as_ptr());
        return;
    }
    DbgPrint(b"[TRIPWIRE] special pool ACTIVE\n\0".as_ptr());

    match VIBE_SELFTEST {
        1 => {
            // Pattern selftest: corrupt the fill pattern just BEFORE the user
            // buffer (overrun mode places data at page end; the slack between
            // POOL_HEADER and the buffer is pattern-checked at free time) =>
            // BUGCHECK 0xC1 case 0x23. NB `p.add(size)` would hit the GUARD
            // page instead when size is align-multiple — that's the other test.
            DbgPrint(b"[TRIPWIRE] SELFTEST pattern: expecting BUGCHECK 0xC1\n\0".as_ptr());
            let p = MmAllocateSpecialPool(64, u32::from_le_bytes(*b"Vibe"), 0 /* NonPagedPool */, 2);
            if p.is_null() {
                DbgPrint(b"[TRIPWIRE] ERROR: selftest allocation failed\n\0".as_ptr());
                return;
            }
            core::ptr::write_volatile((p as *mut u8).sub(1), 0xAA);
            MmFreeSpecialPool(p);
            DbgPrint(b"[TRIPWIRE] ERROR: pattern selftest NOT caught\n\0".as_ptr());
        }
        2 => {
            // Guard selftest: touch the guard page just past the allocation's
            // page => immediate fault (BUGCHECK 0x50) at the corrupting store.
            DbgPrint(b"[TRIPWIRE] SELFTEST guard: expecting BUGCHECK 0x50\n\0".as_ptr());
            let p = MmAllocateSpecialPool(64, u32::from_le_bytes(*b"Vibe"), 0 /* NonPagedPool */, 2);
            if p.is_null() {
                DbgPrint(b"[TRIPWIRE] ERROR: selftest allocation failed\n\0".as_ptr());
                return;
            }
            let guard = ((p as usize & !(PAGE_SIZE - 1)) + PAGE_SIZE) as *mut u8;
            core::ptr::write_volatile(guard, 0xAA);
            DbgPrint(b"[TRIPWIRE] ERROR: guard selftest NOT caught\n\0".as_ptr());
        }
        _ => {}
    }
}
