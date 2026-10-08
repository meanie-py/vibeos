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
//!                       0x50 (guard page) — proves the DETECTION machinery. It
//!                       does not prove routing (it calls MmAllocateSpecialPool
//!                       directly), which is why arming now always runs a routing
//!                       probe through ExAllocatePoolWithTag first and says
//!                       "routing verified" only when the allocation really landed
//!                       in special pool.
//!   CSWITCH_SELFTEST=1  prove cswitchdep barks (needs --cswitchdep build).
//!   QUIET               not read by the kernel: winlogon skips its boot-time
//!                       test ladder (see VibeTripwireRead, and winlogon's
//!                       QuietBootRequested). Conformance boots set it.
//!   NVME_AFFINITY=<hex> not read by the kernel: nvme.sys writes it as its device's
//!                       interrupt Affinity Policy (IrqPolicySpecifiedProcessors),
//!                       the way an INF would, so a boot proves pci.sys, the PnP
//!                       manager and the HAL honour a requested processor set.
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
use ntkrnl_sys::mm::POOL_TYPE;

// Same-crate globals — no FFI decls needed.
// MmSpecialPoolTag: ex-rust global read across the crate boundary as a link symbol
// (#[no_mangle] in ex-rust). crate-split Phase 1b.
extern "C" {
    static mut MmSpecialPoolTag: ntkrnl_sys::base::ULONG;
}
use crate::ARM3::special::MmSpecialPoolCatchOverruns;

extern "C" {
    fn DbgPrint(Format: *const u8, ...) -> u32;
}

// ex-rust exports. `extern "system"` is load-bearing on i386 (stdcall, not cdecl).
extern "system" {
    fn ExAllocatePoolWithTag(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID;
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
}

/// Selftest mode parsed from the tripwire blob: 0 none, 1 pattern, 2 guard.
static mut VIBE_SELFTEST: u8 = 0;
/// CSWITCH_SELFTEST=1 armed via fw_cfg (consumed by `KiStartSyncCensus`).
pub static mut CSWITCH_SELFTEST_ARMED: bool = false;
/// FREEZE_SELFTEST=1: see `ke::census::freeze_selftest`. Destructive.
pub static mut FREEZE_SELFTEST_ARMED: bool = false;
/// `UEXCVERBOSE`: no cap on the arm64 first-chance user-exception records.
pub static mut UEXC_VERBOSE: bool = false;

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
    /// Zero unless the loader identified this as QEMU virt AND identity-mapped
    /// the page (ke::arm64::stubs). Never a bare constant: `select` WRITES, and
    /// on a physical board this address is DRAM or unmapped, so an ungated probe
    /// corrupts memory or faults. `present()` is the only way in.
    #[cfg(target_arch = "aarch64")]
    unsafe fn fw_cfg_base() -> usize {
        extern "C" {
            static mut Arm64FwCfgPhysical: u64;
        }
        Arm64FwCfgPhysical as usize
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn present() -> bool {
        fw_cfg_base() != 0
    }

    #[cfg(not(target_arch = "aarch64"))]
    unsafe fn present() -> bool {
        true
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn select(key: u16) {
        core::ptr::write_volatile((fw_cfg_base() + 8) as *mut u16, key.to_be());
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn read_u8() -> u8 {
        core::ptr::read_volatile(fw_cfg_base() as *const u8)
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
        // No fw_cfg here — do not touch the MMIO window at all (see fw_cfg_base).
        if !present() {
            return None;
        }
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
    } else if let Some(val) = token_val(tok, b"FREEZE_SELFTEST") {
        // Prove a bugcheck stops the OTHER CPUs: once the system is up, a thread on
        // the last CPU prints ticks while this one bugchecks. Pass = ticks stop at the
        // *** BUGCHECK banner (which prints before the freeze, on purpose: a hung freeze
        // still leaves the crash code). Destructive by design -- the boot dies (0xE2).
        if val == b"1" {
            FREEZE_SELFTEST_ARMED = true;
            DbgPrint(b"[TRIPWIRE] freeze selftest armed\n\0".as_ptr());
        }
    } else if let Some(val) = token_val(tok, b"RUSTPANIC_SELFTEST") {
        // crash-value lane: prove the shared panic handler barks — deliberate
        // panic!() with a formatted message. Expect on serial: the full
        // "*** RUST PANIC in ntoskrnl at …tripwire.rs:…" report (message +
        // location + raw-PC backtrace) followed by BUGCHECK 0xE0525354
        // (panic_common::RUST_PANIC). Destructive by design — the boot dies.
        if val == b"1" {
            // Seed a few recognizable flight-recorder events so the Layer-3
            // crash sidecar (KeBugCheckWithTf -> flightrec_dump_recent) has
            // content to format. The selftest fires at Mm Phase 0, before any
            // real gate/pushlock traffic, so the ring is otherwise empty here.
            // Sentinel obj/thr values (0xABCD.../0x1111...) make the seeded
            // rows unmistakable in the [FLIGHTREC] tail. Destructive path only.
            use crate::ke::flightrec;
            flightrec::flightrec_log(flightrec::OP_GATE_PARK, 0xABCD_0001, 0x1111_0001, 0);
            flightrec::flightrec_log(flightrec::OP_GATE_SIGNAL, 0xABCD_0001, 0x2222_0002, 0);
            flightrec::flightrec_log(flightrec::OP_PL_WAKE, 0xABCD_0002, 0x3333_0003, 2);
            DbgPrint(
                b"[TRIPWIRE] rust-panic selftest: panicking NOW (expect *** RUST PANIC + BUGCHECK 0xE0525354)\n\0"
                    .as_ptr(),
            );
            panic!("rust-panic selftest: deliberate panic via tripwire (answer={})", 42);
        }
    } else if token_val(tok, b"KD").is_some() {
        // KD=COM2 is consumed directly by kd64 KdInitSystem (via
        // VibeTripwireHasToken, long before this Mm-phase parse). Recognized
        // here only so it doesn't trip the unknown-token error below.
    } else if tok == b"UEXCVERBOSE" {
        // Print EVERY first-chance user exception in full, lifting the per-code
        // cap in ke/arm64/stubs.rs `uexc_should_print` (which exists because
        // FEX takes access violations and alignment faults by design).
        UEXC_VERBOSE = true;
    } else if token_val(tok, b"NVME_AFFINITY").is_some() {
        // For nvme.sys, through the registry copy, like QUIET.
    } else if tok == b"QUIET" {
        // For user mode, not the kernel: winlogon reads it back through
        // VibeTripwireRead's registry copy. Recognized here only so it doesn't
        // trip the unknown-token error below.
    } else if !tok.is_empty() {
        DbgPrint(b"[TRIPWIRE] ERROR: unknown tripwire token (ignored)\n\0".as_ptr());
    }
}

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

/// This boot's raw config, kept from `vibe_tripwire_init` for `VibeTripwireRead`.
/// A copy rather than a re-read: arm64 reaches fw_cfg through the early-boot
/// identity map, which a Phase 1 caller cannot count on.
struct TripwireCopy {
    len: usize,
    bytes: [u8; CFG_MAX],
}
static mut TRIPWIRE_COPY: TripwireCopy = TripwireCopy { len: 0, bytes: [0; CFG_MAX] };

/// Called at the top of MmArmInitSystem Phase 0 — before system-PTE sizing and
/// MiInitializeSpecialPool, so an armed MmSpecialPoolTag influences both.
pub unsafe fn vibe_tripwire_init() {
    let mut cfg = [0u8; CFG_MAX];
    let Some(n) = fwcfg::read_file(TRIPWIRE_FILE, &mut cfg) else {
        return; // no fw_cfg / no tripwire file — normal boot
    };

    DbgPrint(b"[TRIPWIRE] config received (%lu bytes)\n\0".as_ptr(), n as u32);
    let copy = core::ptr::addr_of_mut!(TRIPWIRE_COPY);
    core::ptr::copy_nonoverlapping(cfg.as_ptr(), (*copy).bytes.as_mut_ptr(), n);
    (*copy).len = n;

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

/// FFI for OTHER kernel crates: copy this boot's raw tripwire config (no NUL)
/// into `out`, returning the bytes copied, 0 when the boot had none. Lets the
/// config manager publish it to user mode (`VibeTripwire` under
/// CurrentControlSet\Control), so a user-mode harness hook can take per-boot
/// knobs too: winlogon's QUIET, which conformance boots set. Valid from
/// MmArmInitSystem Phase 0 on.
#[no_mangle]
pub unsafe extern "system" fn VibeTripwireRead(out: *mut u8, cap: usize) -> usize {
    let copy = core::ptr::addr_of!(TRIPWIRE_COPY);
    let n = core::cmp::min((*copy).len, cap);
    if out.is_null() || n == 0 {
        return 0;
    }
    core::ptr::copy_nonoverlapping((*copy).bytes.as_ptr(), out, n);
    n
}

/// Called right after MiInitializeSpecialPool. Reports arming outcome loudly and
/// runs the destructive selftest when requested (expected result: BUGCHECK).
pub unsafe fn vibe_tripwire_post_pool_init() {
    use crate::ARM3::special::{
        MmAllocateSpecialPool, MmFreeSpecialPool, MmIsSpecialPoolAddress, MmSpecialPoolStart,
    };

    if MmSpecialPoolTag == 0 || MmSpecialPoolTag == u32::MAX {
        return;
    }

    // Fail loud: the tag was armed but init didn't produce a pool.
    if MmSpecialPoolStart.is_null() {
        DbgPrint(b"[TRIPWIRE] ERROR: special pool armed but NOT initialized\n\0".as_ptr());
        return;
    }

    // ROUTING CHECK — the one that matters, and the one whose absence let a dead
    // tripwire ship. An initialized special pool proves nothing on its own: the
    // production path is ExAllocatePoolWithTag consulting ExpPoolFlags, and when
    // MiInitializeSpecialPool set the wrong flag bit, every allocation went to
    // ordinary pool while this function still printed "ACTIVE". Anything that only
    // calls MmAllocateSpecialPool directly is a component test wearing an
    // end-to-end label. So: allocate the way a driver does, and ask where it landed.
    let probe = ExAllocatePoolWithTag(0 /* NonPagedPool */, 64, u32::from_le_bytes(*b"Vibe"));
    if probe.is_null() {
        DbgPrint(b"[TRIPWIRE] ERROR: routing probe allocation failed\n\0".as_ptr());
        return;
    }
    if MmIsSpecialPoolAddress(probe) == 0 {
        // Loud and specific: armed, initialized, and NOT routing. Every corruption
        // hunt run in this state silently produces a normal-pool boot.
        DbgPrint(
            b"[TRIPWIRE] ERROR: special pool NOT ROUTING - ExAllocatePoolWithTag returned\
              \n[TRIPWIRE]        ordinary pool %p. Corruption hunting is DISABLED this boot.\n\0"
                .as_ptr(),
            probe,
        );
        ExFreePoolWithTag(probe, u32::from_le_bytes(*b"Vibe"));
        return;
    }
    ExFreePoolWithTag(probe, u32::from_le_bytes(*b"Vibe"));
    DbgPrint(b"[TRIPWIRE] special pool ACTIVE (routing verified)\n\0".as_ptr());

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
