//! Hand-translated `static` helper functions from BusLogic958.h. These live in the
//! header (so every TU sees them) — here, the single cohab TU. Each is a faithful
//! transliteration: the register helpers do one ScsiPort port-I/O at IO_Address+offset;
//! the statistics helpers bump software counters. The translated BusLogic958.rs calls
//! them by name (resolved via `use crate::inlines::*`).
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use crate::types::*;
use ntkrnl_sys::base::*; // UCHAR/USHORT/PUCHAR (blocklisted out of crate::types)
use core::ffi::c_uint;

// SCSIPORT UCHAR port-I/O — NTAPI = stdcall on i386 (`extern "system"`); declared
// explicitly because the amd64 substrate bindgen renders them inline (no decl emitted),
// but scsiport-rust still exports the symbol on all 3 arches. (Mirrors atapi-rust.)
extern "system" {
    pub fn ScsiPortReadPortUchar(Port: PUCHAR) -> UCHAR;
    pub fn ScsiPortWritePortUchar(Port: PUCHAR, Value: UCHAR);
}

// BusLogic SCSI Host Adapter I/O register offsets (BusLogic958.h §947–952).
const BusLogic_ControlRegisterOffset: usize = 0; // WO
const BusLogic_StatusRegisterOffset: usize = 0; // RO
const BusLogic_CommandParameterRegisterOffset: usize = 1; // WO
const BusLogic_DataInRegisterOffset: usize = 1; // RO
const BusLogic_InterruptRegisterOffset: usize = 2; // RO
const BusLogic_GeometryRegisterOffset: usize = 3; // RO

// Execute Mailbox Command opcode (enum BusLogic_ExecuteMailboxCommand = 0x02).
const BusLogic_ExecuteMailboxCommand: UCHAR = 0x02;

// Control Register single-bit commands. The C zeroes ControlRegister.All then sets one
// Bits.* flag; with the little-endian bitfield layout (bit4 SCSIBusReset .. bit7 HardReset,
// all 3 target arches LE) each reduces to one byte:
//   SCSIBusReset = bit4 = 0x10, InterruptReset = bit5 = 0x20,
//   SoftReset    = bit6 = 0x40, HardReset      = bit7 = 0x80.
const CONTROL_INTERRUPT_RESET: UCHAR = 0x20;
const CONTROL_SOFT_RESET: UCHAR = 0x40;
const CONTROL_HARD_RESET: UCHAR = 0x80;

// ReadBusLogicPort / WriteBusLogicPort (BusLogic958.h §929–940) — the ScsiPort UCHAR
// port-I/O wrappers every register helper goes through.
#[inline]
unsafe fn ReadBusLogicPort(adr: PUCHAR) -> UCHAR {
    ScsiPortReadPortUchar(adr)
}
#[inline]
unsafe fn WriteBusLogicPort(data: UCHAR, adr: PUCHAR) {
    ScsiPortWritePortUchar(adr, data)
}
#[inline]
unsafe fn reg(HostAdapter: *mut BusLogic_HostAdapter_T, off: usize) -> PUCHAR {
    (*HostAdapter).IO_Address.add(off)
}

// ---- Control Register writes (write-only) ------------------------------------------
pub unsafe fn BusLogic_InterruptReset(HostAdapter: *mut BusLogic_HostAdapter_T) {
    WriteBusLogicPort(CONTROL_INTERRUPT_RESET, reg(HostAdapter, BusLogic_ControlRegisterOffset));
}
pub unsafe fn BusLogic_SoftReset(HostAdapter: *mut BusLogic_HostAdapter_T) {
    WriteBusLogicPort(CONTROL_SOFT_RESET, reg(HostAdapter, BusLogic_ControlRegisterOffset));
}
pub unsafe fn BusLogic_HardReset(HostAdapter: *mut BusLogic_HostAdapter_T) {
    WriteBusLogicPort(CONTROL_HARD_RESET, reg(HostAdapter, BusLogic_ControlRegisterOffset));
}

// ---- Read-only registers -----------------------------------------------------------
pub unsafe fn BusLogic_ReadStatusRegister(HostAdapter: *mut BusLogic_HostAdapter_T) -> UCHAR {
    ReadBusLogicPort(reg(HostAdapter, BusLogic_StatusRegisterOffset))
}
pub unsafe fn BusLogic_ReadInterruptRegister(HostAdapter: *mut BusLogic_HostAdapter_T) -> UCHAR {
    ReadBusLogicPort(reg(HostAdapter, BusLogic_InterruptRegisterOffset))
}
pub unsafe fn BusLogic_ReadGeometryRegister(HostAdapter: *mut BusLogic_HostAdapter_T) -> UCHAR {
    ReadBusLogicPort(reg(HostAdapter, BusLogic_GeometryRegisterOffset))
}
pub unsafe fn BusLogic_ReadDataInRegister(HostAdapter: *mut BusLogic_HostAdapter_T) -> UCHAR {
    ReadBusLogicPort(reg(HostAdapter, BusLogic_DataInRegisterOffset))
}

// ---- Command Parameter Register (write-only) + Start Mailbox Command ----------------
pub unsafe fn BusLogic_WriteCommandParameterRegister(
    HostAdapter: *mut BusLogic_HostAdapter_T,
    Value: UCHAR,
) {
    WriteBusLogicPort(Value, reg(HostAdapter, BusLogic_CommandParameterRegisterOffset));
}
pub unsafe fn BusLogic_StartMailboxCommand(HostAdapter: *mut BusLogic_HostAdapter_T) {
    BusLogic_WriteCommandParameterRegister(HostAdapter, BusLogic_ExecuteMailboxCommand);
}

// ---- Statistics counters (BusLogic958.h §1137–1182) --------------------------------
// Signatures match the translated extern decls (`*mut c_uint` / `*mut c_void`); the
// pointer is reinterpreted as the real struct/array, which is layout-compatible.
pub unsafe fn BusLogic_IncrementByteCounter(ByteCounter: *mut BusLogic_ByteCounter_T, Amount: c_uint) {
    (*ByteCounter).Units += Amount;
    if (*ByteCounter).Units > 999999999 {
        (*ByteCounter).Units -= 1000000000;
        (*ByteCounter).Billions += 1;
    }
}
pub unsafe fn BusLogic_IncrementSizeBucket(SizeBuckets: *mut c_uint, Amount: c_uint) {
    let Index: usize = if Amount < 8 * 1024 {
        if Amount < 2 * 1024 {
            if Amount < 1 * 1024 { 0 } else { 1 }
        } else if Amount < 4 * 1024 { 2 } else { 3 }
    } else if Amount < 128 * 1024 {
        if Amount < 32 * 1024 {
            if Amount < 16 * 1024 { 4 } else { 5 }
        } else if Amount < 64 * 1024 { 6 } else { 7 }
    } else if Amount < 256 * 1024 { 8 } else { 9 };
    *SizeBuckets.add(Index) += 1;
}
pub unsafe fn BusLogic_IncrementErrorCounter(Counter: *mut USHORT) {
    if *Counter < 65535 {
        *Counter += 1;
    }
}
