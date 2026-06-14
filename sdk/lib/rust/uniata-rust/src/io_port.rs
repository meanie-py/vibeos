//! io_port — UniATA hardware register access layer (FOUNDATION module).
//!
//! Hand-translated from the `AtapiReadPortN_template` / `AtapiWritePortN_template`
//! macro families in id_ata.cpp (and the AtapiReadPort1/2/4 etc. decls in bsmaster.h).
//! These route every IDE register touch through a per-register `IORES` translation
//! (port-mapped `ScsiPortReadPort*` vs memory-mapped `ScsiPortReadRegister*`), so they
//! are THE boot-critical correctness surface — every other uniata module reaches the
//! drive through here.
//!
//! Translation decisions (encoded once, here, so the fan-out subagents share them):
//!   * The C functions are `DDKFASTAPI` (__fastcall) with C++-mangled names. For an
//!     all-Rust cohab driver every call is Rust→Rust *inside* this crate, so these are
//!     plain `pub unsafe fn` (Rust ABI) — NO `extern "fastcall"`, NO `#[no_mangle]`.
//!     (bindgen also emitted `extern "fastcall"` decls for these in `crate::types`;
//!     those are dead — modules must import the IO primitives from `crate::io_port`,
//!     and a named `use` shadows the `crate::types::*` glob.)
//!   * `KdPrint`/`ASSERT` debug noise is dropped (the `res->Proc` "should never happen"
//!     branch becomes a no-op, matching DBG-off release behaviour).
//!   * `UniataNanoSleep` + its calibration globals `g_Perf`/`g_PerfDt` live here too —
//!     they are the only cross-cutting timing primitive the buffer loops need. The perf
//!     calibration site in id_ata.cpp writes them via `crate::io_port::{g_Perf,g_PerfDt}`.

use crate::types::{
    PHW_CHANNEL, PIORES,
    ScsiPortReadPortUchar, ScsiPortReadPortUlong, ScsiPortReadPortUshort,
    ScsiPortReadPortBufferUlong, ScsiPortReadPortBufferUshort,
    ScsiPortReadRegisterUchar, ScsiPortReadRegisterUlong, ScsiPortReadRegisterUshort,
    ScsiPortWritePortUchar, ScsiPortWritePortUlong, ScsiPortWritePortUshort,
    ScsiPortWritePortBufferUlong, ScsiPortWritePortBufferUshort,
    ScsiPortWriteRegisterUchar, ScsiPortWriteRegisterUlong, ScsiPortWriteRegisterUshort,
};
use ntkrnl_sys::ddk::{
    MmIsAddressValid,
    LARGE_INTEGER, LONGLONG, PUCHAR, PULONG, PUSHORT, PVOID, UCHAR, ULONG, ULONG_PTR, USHORT,
};
// KeQuerySystemTime: real exported fn on i386, SharedUserData inline on amd64/arm64.
#[cfg(target_arch = "x86")]
use ntkrnl_sys::ddk::KeQuerySystemTime;
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::inline::ke::KeQuerySystemTime;

/// `IDX_MAX_REG` — number of register slots in `HW_CHANNEL::RegTranslation[]`.
/// (IDX_INDEXED_IO + IDX_INDEXED_IO_SZ resolves to 30; the array is `[IORES; 30]`.)
pub const IDX_MAX_REG: ULONG_PTR = 30;

// --- UniataNanoSleep calibration globals (set by the perf-calibration site) ----------
pub static mut g_Perf: LONGLONG = 0;
pub static mut g_PerfDt: ULONG = 0;

/// Resolve a `_port` token to its `IORES`: either an absolute `PIORES` (token at/above
/// IDX_MAX_REG is a real pointer) or a slot in the channel's translation table.
/// Returns null when neither applies (the C path that `KdPrint`s and bails).
#[inline]
unsafe fn resolve_res(chan: PHW_CHANNEL, _port: ULONG_PTR) -> PIORES {
    if _port >= IDX_MAX_REG {
        _port as PIORES
    } else if !chan.is_null() {
        &mut (*chan).RegTranslation[_port as usize] as PIORES
    } else {
        core::ptr::null_mut()
    }
}

#[inline]
unsafe fn res_addr(res: PIORES) -> ULONG_PTR {
    *(*res).__bindgen_anon_1.Addr.as_ref()
}

/// Busy-wait `nano` nanoseconds via repeated `KeQuerySystemTime` (uniata's own spin).
pub unsafe fn UniataNanoSleep(nano: ULONG) {
    if nano == 0 || g_Perf == 0 || g_PerfDt == 0 {
        return;
    }
    let mut t: LONGLONG = (g_Perf * nano as LONGLONG) / g_PerfDt as LONGLONG / 1000;
    if t == 0 {
        t = 1;
    }
    let mut t0: LARGE_INTEGER = core::mem::zeroed();
    // do { KeQuerySystemTime(&t0); t--; } while(t);
    loop {
        KeQuerySystemTime(&mut t0);
        t -= 1;
        if t == 0 {
            break;
        }
    }
}

// ======================================================================================
// Single-register writes  (AtapiWritePortN_template: 4=ULONG, 2=USHORT, 1=UCHAR)
// ======================================================================================
pub unsafe fn AtapiWritePort4(chan: PHW_CHANNEL, _port: ULONG_PTR, data: ULONG) {
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return;
    }
    let addr = res_addr(res);
    if (*res).MemIo() == 0 {
        ScsiPortWritePortUlong(addr as PULONG, data);
    } else {
        ScsiPortWriteRegisterUlong(addr as PULONG, data);
    }
}

pub unsafe fn AtapiWritePort2(chan: PHW_CHANNEL, _port: ULONG_PTR, data: USHORT) {
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return;
    }
    let addr = res_addr(res);
    if (*res).MemIo() == 0 {
        ScsiPortWritePortUshort(addr as PUSHORT, data);
    } else {
        ScsiPortWriteRegisterUshort(addr as PUSHORT, data);
    }
}

pub unsafe fn AtapiWritePort1(chan: PHW_CHANNEL, _port: ULONG_PTR, data: UCHAR) {
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return;
    }
    let addr = res_addr(res);
    if (*res).MemIo() == 0 {
        ScsiPortWritePortUchar(addr as PUCHAR, data);
    } else {
        ScsiPortWriteRegisterUchar(addr as PUCHAR, data);
    }
}

// ======================================================================================
// Offset writes  (AtapiWritePortExN_template: 4=ULONG, 1=UCHAR; Ex2 is commented out in C)
// ======================================================================================
pub unsafe fn AtapiWritePortEx4(chan: PHW_CHANNEL, _port: ULONG_PTR, offs: ULONG, data: ULONG) {
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return;
    }
    let addr = res_addr(res) + offs as ULONG_PTR;
    if (*res).MemIo() == 0 {
        ScsiPortWritePortUlong(addr as PULONG, data);
    } else {
        ScsiPortWriteRegisterUlong(addr as PULONG, data);
    }
}

pub unsafe fn AtapiWritePortEx1(chan: PHW_CHANNEL, _port: ULONG_PTR, offs: ULONG, data: UCHAR) {
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return;
    }
    let addr = res_addr(res) + offs as ULONG_PTR;
    if (*res).MemIo() == 0 {
        ScsiPortWritePortUchar(addr as PUCHAR, data);
    } else {
        ScsiPortWriteRegisterUchar(addr as PUCHAR, data);
    }
}

// ======================================================================================
// Single-register reads  (AtapiReadPortN_template: 4=ULONG, 2=USHORT, 1=UCHAR)
// ======================================================================================
pub unsafe fn AtapiReadPort4(chan: PHW_CHANNEL, _port: ULONG_PTR) -> ULONG {
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return !0; // (ULONG)(-1)
    }
    let addr = res_addr(res);
    if (*res).MemIo() == 0 {
        ScsiPortReadPortUlong(addr as PULONG)
    } else {
        ScsiPortReadRegisterUlong(addr as PULONG)
    }
}

pub unsafe fn AtapiReadPort2(chan: PHW_CHANNEL, _port: ULONG_PTR) -> USHORT {
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return !0; // (USHORT)(-1)
    }
    let addr = res_addr(res);
    if (*res).MemIo() == 0 {
        ScsiPortReadPortUshort(addr as PUSHORT)
    } else {
        ScsiPortReadRegisterUshort(addr as PUSHORT)
    }
}

pub unsafe fn AtapiReadPort1(chan: PHW_CHANNEL, _port: ULONG_PTR) -> UCHAR {
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return !0; // (UCHAR)(-1)
    }
    let addr = res_addr(res);
    if (*res).MemIo() == 0 {
        ScsiPortReadPortUchar(addr as PUCHAR)
    } else {
        ScsiPortReadRegisterUchar(addr as PUCHAR)
    }
}

// ======================================================================================
// Offset reads  (AtapiReadPortExN_template: 4=ULONG, 1=UCHAR; Ex2 is commented out in C)
// ======================================================================================
pub unsafe fn AtapiReadPortEx4(chan: PHW_CHANNEL, _port: ULONG_PTR, offs: ULONG) -> ULONG {
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return !0;
    }
    let addr = res_addr(res) + offs as ULONG_PTR;
    if (*res).MemIo() == 0 {
        ScsiPortReadPortUlong(addr as PULONG)
    } else {
        ScsiPortReadRegisterUlong(addr as PULONG)
    }
}

pub unsafe fn AtapiReadPortEx1(chan: PHW_CHANNEL, _port: ULONG_PTR, offs: ULONG) -> UCHAR {
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return !0;
    }
    let addr = res_addr(res) + offs as ULONG_PTR;
    if (*res).MemIo() == 0 {
        ScsiPortReadPortUchar(addr as PUCHAR)
    } else {
        ScsiPortReadRegisterUchar(addr as PUCHAR)
    }
}

// ======================================================================================
// Buffer reads  (AtapiReadPortBufferN_template: AtapiReadBuffer4=ULONG, AtapiReadBuffer2=USHORT)
// ======================================================================================
pub unsafe fn AtapiReadBuffer4(
    chan: PHW_CHANNEL,
    _port: ULONG_PTR,
    Buffer: PVOID,
    mut Count: ULONG,
    Timing: ULONG,
) {
    if Buffer.is_null() || Count == 0 || MmIsAddressValid(Buffer) == 0 {
        return;
    }
    let mut p = Buffer as *mut ULONG;
    if Timing != 0 {
        while Count != 0 {
            *p = AtapiReadPort4(chan, _port);
            Count -= 1;
            p = p.add(1);
            UniataNanoSleep(Timing);
        }
        return;
    }
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return;
    }
    let addr = res_addr(res);
    if (*res).MemIo() == 0 {
        ScsiPortReadPortBufferUlong(addr as PULONG, p, Count);
        return;
    }
    while Count != 0 {
        *p = ScsiPortReadRegisterUlong(addr as PULONG);
        Count -= 1;
        p = p.add(1);
    }
}

pub unsafe fn AtapiReadBuffer2(
    chan: PHW_CHANNEL,
    _port: ULONG_PTR,
    Buffer: PVOID,
    mut Count: ULONG,
    Timing: ULONG,
) {
    if Buffer.is_null() || Count == 0 || MmIsAddressValid(Buffer) == 0 {
        return;
    }
    let mut p = Buffer as *mut USHORT;
    if Timing != 0 {
        while Count != 0 {
            *p = AtapiReadPort2(chan, _port);
            Count -= 1;
            p = p.add(1);
            UniataNanoSleep(Timing);
        }
        return;
    }
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return;
    }
    let addr = res_addr(res);
    if (*res).MemIo() == 0 {
        ScsiPortReadPortBufferUshort(addr as PUSHORT, p, Count);
        return;
    }
    while Count != 0 {
        *p = ScsiPortReadRegisterUshort(addr as PUSHORT);
        Count -= 1;
        p = p.add(1);
    }
}

// ======================================================================================
// Buffer writes (AtapiWritePortBufferN_template: AtapiWriteBuffer4=ULONG, AtapiWriteBuffer2=USHORT)
// ======================================================================================
pub unsafe fn AtapiWriteBuffer4(
    chan: PHW_CHANNEL,
    _port: ULONG_PTR,
    Buffer: PVOID,
    mut Count: ULONG,
    Timing: ULONG,
) {
    if Buffer.is_null() || Count == 0 || MmIsAddressValid(Buffer) == 0 {
        return;
    }
    let mut p = Buffer as *mut ULONG;
    if Timing != 0 {
        while Count != 0 {
            AtapiWritePort4(chan, _port, *p);
            p = p.add(1);
            Count -= 1;
            UniataNanoSleep(Timing);
        }
        return;
    }
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return;
    }
    let addr = res_addr(res);
    if (*res).MemIo() == 0 {
        ScsiPortWritePortBufferUlong(addr as PULONG, p, Count);
        return;
    }
    while Count != 0 {
        ScsiPortWriteRegisterUlong(addr as PULONG, *p);
        Count -= 1;
        p = p.add(1);
    }
}

pub unsafe fn AtapiWriteBuffer2(
    chan: PHW_CHANNEL,
    _port: ULONG_PTR,
    Buffer: PVOID,
    mut Count: ULONG,
    Timing: ULONG,
) {
    if Buffer.is_null() || Count == 0 || MmIsAddressValid(Buffer) == 0 {
        return;
    }
    let mut p = Buffer as *mut USHORT;
    if Timing != 0 {
        while Count != 0 {
            AtapiWritePort2(chan, _port, *p);
            p = p.add(1);
            Count -= 1;
            UniataNanoSleep(Timing);
        }
        return;
    }
    let res = resolve_res(chan, _port);
    if res.is_null() {
        return;
    }
    let addr = res_addr(res);
    if (*res).MemIo() == 0 {
        ScsiPortWritePortBufferUshort(addr as PUSHORT, p, Count);
        return;
    }
    while Count != 0 {
        ScsiPortWriteRegisterUshort(addr as PUSHORT, *p);
        Count -= 1;
        p = p.add(1);
    }
}
