//! Shared foundation for the atapi translation: the SCSIPORT services atapi.sys
//! imports, plus the IDE port-I/O / wait "beautification" macros from atapi.h
//! rendered as `#[inline]` helpers.
//!
//! The C header macros set a caller `Status` variable in place; the Rust helpers
//! instead RETURN the status byte — so `WaitOnBusy(base, status);` in C becomes
//! `status = WaitOnBusy(base);` in Rust. Every translated function calls these
//! (never re-derives them), so the whole driver agrees on the exact poll loops.

use ntkrnl_sys::ddk::{
    BOOLEAN, INTERFACE_TYPE, PUCHAR, PULONG, PUSHORT, PVOID, UCHAR, ULONG, ULONG_PTR, USHORT,
};

use crate::types::{
    PIDE_REGISTERS_1, PIDE_REGISTERS_2, PSCSI_REQUEST_BLOCK, SCSI_PHYSICAL_ADDRESS,
    IDE_COMMAND_ATAPI_RESET, IDE_DC_REENABLE_CONTROLLER, IDE_DC_RESET_CONTROLLER, IDE_STATUS_BUSY,
    IDE_STATUS_DRQ, IDE_STATUS_IDLE, SCSIOP_ERASE, SCSIOP_LOAD_UNLOAD, SCSIOP_LOCATE,
    SCSIOP_REWIND, SCSIOP_SEEK, SCSIOP_SPACE, SCSIOP_WRITE_FILEMARKS,
};

// ---------------------------------------------------------------------------
// SCSIPORT services (exported by scsiport.sys; NTAPI = stdcall on i386). atapi
// is a SCSIPORT miniport, so these resolve against scsiport-rust at link time.
// ---------------------------------------------------------------------------
extern "system" {
    pub fn ScsiPortReadPortUchar(Port: PUCHAR) -> UCHAR;
    pub fn ScsiPortReadPortUshort(Port: PUSHORT) -> USHORT;
    pub fn ScsiPortWritePortUchar(Port: PUCHAR, Value: UCHAR);
    pub fn ScsiPortReadPortBufferUshort(Port: PUSHORT, Buffer: PUSHORT, Count: ULONG);
    pub fn ScsiPortWritePortBufferUshort(Port: PUSHORT, Buffer: PUSHORT, Count: ULONG);
    pub fn ScsiPortReadPortBufferUlong(Port: PULONG, Buffer: PULONG, Count: ULONG);
    pub fn ScsiPortWritePortBufferUlong(Port: PULONG, Buffer: PULONG, Count: ULONG);
    pub fn ScsiPortStallExecution(Delay: ULONG);
    pub fn ScsiPortGetDeviceBase(
        HwDeviceExtension: PVOID,
        BusType: INTERFACE_TYPE,
        SystemIoBusNumber: ULONG,
        IoAddress: SCSI_PHYSICAL_ADDRESS,
        NumberOfBytes: ULONG,
        InIoSpace: BOOLEAN,
    ) -> PVOID;
    pub fn ScsiPortFreeDeviceBase(HwDeviceExtension: PVOID, MappedAddress: PVOID);
    pub fn ScsiPortGetBusData(
        DeviceExtension: PVOID,
        BusDataType: ULONG,
        SystemIoBusNumber: ULONG,
        SlotNumber: ULONG,
        Buffer: PVOID,
        Length: ULONG,
    ) -> ULONG;
    pub fn ScsiPortConvertUlongToPhysicalAddress(UlongAddress: ULONG_PTR) -> SCSI_PHYSICAL_ADDRESS;
    pub fn ScsiPortCompleteRequest(
        HwDeviceExtension: PVOID,
        PathId: UCHAR,
        TargetId: UCHAR,
        Lun: UCHAR,
        SrbStatus: UCHAR,
    );
    pub fn ScsiPortMoveMemory(WriteBuffer: PVOID, ReadBuffer: PVOID, Length: ULONG);
    pub fn ScsiPortLogError(
        HwDeviceExtension: PVOID,
        Srb: PSCSI_REQUEST_BLOCK,
        PathId: UCHAR,
        TargetId: UCHAR,
        Lun: UCHAR,
        ErrorCode: ULONG,
        UniqueId: ULONG,
    );
    pub fn ScsiPortInitialize(
        Argument1: PVOID,
        Argument2: PVOID,
        HwInitializationData: PVOID,
        HwContext: PVOID,
    ) -> ULONG;
}

extern "C" {
    /// cdecl varargs — `ScsiPortNotification(NotificationType, HwDeviceExtension, ...)`.
    /// The variadic tail depends on NotificationType (see scsiport's c_variadic impl).
    pub fn ScsiPortNotification(NotificationType: ULONG, HwDeviceExtension: PVOID, ...);
}

/// `#define ScsiPortConvertPhysicalAddressToUlong(Address) ((Address).LowPart)`
#[inline]
pub unsafe fn ScsiPortConvertPhysicalAddressToUlong(Address: SCSI_PHYSICAL_ADDRESS) -> ULONG {
    // SCSI_PHYSICAL_ADDRESS == LARGE_INTEGER; .LowPart is the low 32 bits.
    Address.u.LowPart
}

// ---------------------------------------------------------------------------
// IDE register helpers (atapi.h "Beautification macros"). The Status-setting
// macros return the byte instead of mutating a caller variable.
// ---------------------------------------------------------------------------

/// `GetStatus` — read the alternate-status register (IDE_REGISTERS_2).
#[inline]
pub unsafe fn GetStatus(base: PIDE_REGISTERS_2) -> UCHAR {
    ScsiPortReadPortUchar(&raw mut (*base).AlternateStatus)
}

/// `GetBaseStatus` — read the command/status register (IDE_REGISTERS_1).
#[inline]
pub unsafe fn GetBaseStatus(base: PIDE_REGISTERS_1) -> UCHAR {
    ScsiPortReadPortUchar(&raw mut (*base).Command)
}

/// `WriteCommand` — write the command register (IDE_REGISTERS_1).
#[inline]
pub unsafe fn WriteCommand(base: PIDE_REGISTERS_1, command: UCHAR) {
    ScsiPortWritePortUchar(&raw mut (*base).Command, command);
}

/// `ReadBuffer` — PIO read `count` ushorts from the data register.
#[inline]
pub unsafe fn ReadBuffer(base: PIDE_REGISTERS_1, buffer: PUSHORT, count: ULONG) {
    ScsiPortReadPortBufferUshort(&raw mut (*base).Data, buffer, count);
}

/// `WriteBuffer` — PIO write `count` ushorts to the data register.
#[inline]
pub unsafe fn WriteBuffer(base: PIDE_REGISTERS_1, buffer: PUSHORT, count: ULONG) {
    ScsiPortWritePortBufferUshort(&raw mut (*base).Data, buffer, count);
}

/// `ReadBuffer2` — 32-bit PIO read `count` ulongs from the data register.
#[inline]
pub unsafe fn ReadBuffer2(base: PIDE_REGISTERS_1, buffer: PULONG, count: ULONG) {
    ScsiPortReadPortBufferUlong(&raw mut (*base).Data as PULONG, buffer, count);
}

/// `WriteBuffer2` — 32-bit PIO write `count` ulongs to the data register.
#[inline]
pub unsafe fn WriteBuffer2(base: PIDE_REGISTERS_1, buffer: PULONG, count: ULONG) {
    ScsiPortWritePortBufferUlong(&raw mut (*base).Data as PULONG, buffer, count);
}

/// `WaitOnBusy` — poll alt-status up to 20000× clearing BUSY; returns last status.
#[inline]
pub unsafe fn WaitOnBusy(base: PIDE_REGISTERS_2) -> UCHAR {
    let mut status: UCHAR = 0;
    for _ in 0..20000u32 {
        status = GetStatus(base);
        if (status & IDE_STATUS_BUSY as UCHAR) != 0 {
            ScsiPortStallExecution(150);
            continue;
        } else {
            break;
        }
    }
    status
}

/// `WaitOnBaseBusy` — same on the base (command) register (IDE_REGISTERS_1).
#[inline]
pub unsafe fn WaitOnBaseBusy(base: PIDE_REGISTERS_1) -> UCHAR {
    let mut status: UCHAR = 0;
    for _ in 0..20000u32 {
        status = GetBaseStatus(base);
        if (status & IDE_STATUS_BUSY as UCHAR) != 0 {
            ScsiPortStallExecution(150);
            continue;
        } else {
            break;
        }
    }
    status
}

/// `WaitForDrq` — poll up to 1000× for DRQ (clear of BUSY); returns last status.
#[inline]
pub unsafe fn WaitForDrq(base: PIDE_REGISTERS_2) -> UCHAR {
    let mut status: UCHAR = 0;
    for _ in 0..1000u32 {
        status = GetStatus(base);
        if (status & IDE_STATUS_BUSY as UCHAR) != 0 {
            ScsiPortStallExecution(100);
        } else if (status & IDE_STATUS_DRQ as UCHAR) != 0 {
            break;
        } else {
            ScsiPortStallExecution(200);
        }
    }
    status
}

/// `WaitShortForDrq` — short (2×) DRQ poll; returns last status.
#[inline]
pub unsafe fn WaitShortForDrq(base: PIDE_REGISTERS_2) -> UCHAR {
    let mut status: UCHAR = 0;
    for _ in 0..2u32 {
        status = GetStatus(base);
        if (status & IDE_STATUS_BUSY as UCHAR) != 0 {
            ScsiPortStallExecution(100);
        } else if (status & IDE_STATUS_DRQ as UCHAR) != 0 {
            break;
        } else {
            ScsiPortStallExecution(100);
        }
    }
    status
}

/// `AtapiSoftReset` — ATAPI soft reset of `DeviceNumber` on the channel at `base`
/// (IDE_REGISTERS_1). The follow-up busy-wait uses the alt-status register at
/// `base + 0x206`, exactly as the C macro.
#[inline]
pub unsafe fn AtapiSoftReset(base: PIDE_REGISTERS_1, DeviceNumber: ULONG) {
    ScsiPortWritePortUchar(
        &raw mut (*base).DriveSelect,
        (((DeviceNumber & 0x1) << 4) | 0xA0) as UCHAR,
    );
    ScsiPortStallExecution(500);
    ScsiPortWritePortUchar(&raw mut (*base).Command, IDE_COMMAND_ATAPI_RESET as UCHAR);
    let mut i: ULONG = 1000 * 1000;
    while (ScsiPortReadPortUchar(&raw mut (*base).Command) & IDE_STATUS_BUSY as UCHAR) != 0 && i != 0 {
        i -= 1;
        ScsiPortStallExecution(30);
    }
    ScsiPortWritePortUchar(
        &raw mut (*base).DriveSelect,
        ((DeviceNumber << 4) | 0xA0) as UCHAR,
    );
    let alt: PIDE_REGISTERS_2 = (base as PUCHAR).add(0x206) as PIDE_REGISTERS_2;
    let _statusByte = WaitOnBusy(alt);
    ScsiPortStallExecution(500);
}

/// `IdeHardReset` — pulse the controller-reset bit, poll alt-status (IDE_REGISTERS_2).
/// NB: faithful to the C macro, which assigns `result = TRUE` unconditionally at the
/// end, so this always returns TRUE.
#[inline]
pub unsafe fn IdeHardReset(base: PIDE_REGISTERS_2) -> BOOLEAN {
    ScsiPortWritePortUchar(
        &raw mut (*base).AlternateStatus,
        IDE_DC_RESET_CONTROLLER as UCHAR,
    );
    ScsiPortStallExecution(50 * 1000);
    ScsiPortWritePortUchar(
        &raw mut (*base).AlternateStatus,
        IDE_DC_REENABLE_CONTROLLER as UCHAR,
    );
    let mut _i: ULONG = 0;
    while _i < 1000 * 1000 {
        let statusByte = ScsiPortReadPortUchar(&raw mut (*base).AlternateStatus);
        if statusByte != IDE_STATUS_IDLE as UCHAR && statusByte != 0x0 {
            ScsiPortStallExecution(5);
        } else {
            break;
        }
        _i += 1;
    }
    // C: `if (i == 1000*1000) result = FALSE; result = TRUE;` → always TRUE.
    1 as BOOLEAN
}

/// `IS_RDP` — tape "DSC restrictive" opcodes.
#[inline]
pub fn IS_RDP(OperationCode: UCHAR) -> bool {
    OperationCode == SCSIOP_ERASE as UCHAR
        || OperationCode == SCSIOP_LOAD_UNLOAD as UCHAR
        || OperationCode == SCSIOP_LOCATE as UCHAR
        || OperationCode == SCSIOP_REWIND as UCHAR
        || OperationCode == SCSIOP_SPACE as UCHAR
        || OperationCode == SCSIOP_SEEK as UCHAR
        || OperationCode == SCSIOP_WRITE_FILEMARKS as UCHAR
}
