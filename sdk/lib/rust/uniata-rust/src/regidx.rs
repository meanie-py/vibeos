//! regidx — UniATA register-index map (FOUNDATION module).
//!
//! The `IDX_*` constants are the *first argument* to every `AtapiReadPort*` /
//! `AtapiWritePort*` call: they index `HW_CHANNEL::RegTranslation[]`, mapping a logical
//! IDE/ATAPI/BusMaster/SATA/AHCI register to its physical address. In C they are
//! `FIELD_OFFSET(...)`-expression `#define`s (atapi.h / bsmaster.h) — which bindgen
//! drops — so they must be reconstructed here, shared across every fan-out module.
//!
//! Reconstructed correct-by-construction with `core::mem::offset_of!` on the bindgen
//! structs (NOT hand-typed numbers): a wrong struct/field name fails to *compile*,
//! whereas a fat-fingered literal would silently mis-address a hardware register and
//! break boot. The `+ IDX_*` chain arithmetic mirrors the C `#define` chain verbatim.

use core::mem::{offset_of, size_of};
use crate::types::{
    _AHCI_ATA_H2D_FIS, _ATAPI_REGISTERS_1__i, _ATAPI_REGISTERS_1__o, _IDE_AHCI_PORT_REGISTERS,
    _IDE_AHCI_REGISTERS, _IDE_BUSMASTER_REGISTERS, _IDE_REGISTERS_1__i, _IDE_REGISTERS_1__o,
    IDE_REGISTERS_1, IDE_REGISTERS_2,
};
use ntkrnl_sys::ddk::ULONG_PTR;

// Helper: offset_of! yields usize; the IO API takes ULONG_PTR (== usize-width on each arch).
macro_rules! off {
    ($ty:ty, $field:ident) => {
        (offset_of!($ty, $field) as ULONG_PTR)
    };
}

// ===== IDE_REGISTERS_1 — primary command block (8 ports, read 'i' / write 'o' overlay) =====
pub const IDX_IO1: ULONG_PTR = 0;
pub const IDX_IO1_SZ: ULONG_PTR = size_of::<IDE_REGISTERS_1>() as ULONG_PTR; // 8

pub const IDX_IO1_i_Data: ULONG_PTR = off!(_IDE_REGISTERS_1__i, Data) + IDX_IO1;
pub const IDX_IO1_i_Error: ULONG_PTR = off!(_IDE_REGISTERS_1__i, Error) + IDX_IO1;
pub const IDX_IO1_i_BlockCount: ULONG_PTR = off!(_IDE_REGISTERS_1__i, BlockCount) + IDX_IO1;
pub const IDX_IO1_i_BlockNumber: ULONG_PTR = off!(_IDE_REGISTERS_1__i, BlockNumber) + IDX_IO1;
pub const IDX_IO1_i_CylinderLow: ULONG_PTR = off!(_IDE_REGISTERS_1__i, CylinderLow) + IDX_IO1;
pub const IDX_IO1_i_CylinderHigh: ULONG_PTR = off!(_IDE_REGISTERS_1__i, CylinderHigh) + IDX_IO1;
pub const IDX_IO1_i_DriveSelect: ULONG_PTR = off!(_IDE_REGISTERS_1__i, DriveSelect) + IDX_IO1;
pub const IDX_IO1_i_Status: ULONG_PTR = off!(_IDE_REGISTERS_1__i, Status) + IDX_IO1;

pub const IDX_IO1_o: ULONG_PTR = IDX_IO1_SZ; // 8
pub const IDX_IO1_o_SZ: ULONG_PTR = size_of::<IDE_REGISTERS_1>() as ULONG_PTR; // 8

pub const IDX_IO1_o_Data: ULONG_PTR = off!(_IDE_REGISTERS_1__o, Data) + IDX_IO1_o;
pub const IDX_IO1_o_Feature: ULONG_PTR = off!(_IDE_REGISTERS_1__o, Feature) + IDX_IO1_o;
pub const IDX_IO1_o_BlockCount: ULONG_PTR = off!(_IDE_REGISTERS_1__o, BlockCount) + IDX_IO1_o;
pub const IDX_IO1_o_BlockNumber: ULONG_PTR = off!(_IDE_REGISTERS_1__o, BlockNumber) + IDX_IO1_o;
pub const IDX_IO1_o_CylinderLow: ULONG_PTR = off!(_IDE_REGISTERS_1__o, CylinderLow) + IDX_IO1_o;
pub const IDX_IO1_o_CylinderHigh: ULONG_PTR = off!(_IDE_REGISTERS_1__o, CylinderHigh) + IDX_IO1_o;
pub const IDX_IO1_o_DriveSelect: ULONG_PTR = off!(_IDE_REGISTERS_1__o, DriveSelect) + IDX_IO1_o;
pub const IDX_IO1_o_Command: ULONG_PTR = off!(_IDE_REGISTERS_1__o, Command) + IDX_IO1_o;

// ===== IDE_REGISTERS_2 — control block (AltStatus read / Control write, same byte) =====
pub const IDX_IO2: ULONG_PTR = IDX_IO1_o + IDX_IO1_o_SZ; // 16
pub const IDX_IO2_SZ: ULONG_PTR = size_of::<IDE_REGISTERS_2>() as ULONG_PTR; // 1
// AltStatus & Control are a 1-byte union (offset 0 within the register).
pub const IDX_IO2_AltStatus: ULONG_PTR = 0 + IDX_IO2; // 16
pub const IDX_IO2_o: ULONG_PTR = IDX_IO2 + IDX_IO2_SZ; // 17
pub const IDX_IO2_o_SZ: ULONG_PTR = size_of::<IDE_REGISTERS_2>() as ULONG_PTR; // 1
pub const IDX_IO2_o_Control: ULONG_PTR = 0 + IDX_IO2_o; // 17

// ===== ATAPI_REGISTERS_1 — ATAPI overlay of the primary command block (IDX_ATAPI_IO1==IDX_IO1) =====
pub const IDX_ATAPI_IO1: ULONG_PTR = IDX_IO1; // 0
pub const IDX_ATAPI_IO1_i_Data: ULONG_PTR = off!(_ATAPI_REGISTERS_1__i, Data) + IDX_ATAPI_IO1;
pub const IDX_ATAPI_IO1_i_Error: ULONG_PTR = off!(_ATAPI_REGISTERS_1__i, Error) + IDX_ATAPI_IO1;
pub const IDX_ATAPI_IO1_i_InterruptReason: ULONG_PTR =
    off!(_ATAPI_REGISTERS_1__i, InterruptReason) + IDX_ATAPI_IO1;
pub const IDX_ATAPI_IO1_i_ByteCountLow: ULONG_PTR =
    off!(_ATAPI_REGISTERS_1__i, ByteCountLow) + IDX_ATAPI_IO1;
pub const IDX_ATAPI_IO1_i_ByteCountHigh: ULONG_PTR =
    off!(_ATAPI_REGISTERS_1__i, ByteCountHigh) + IDX_ATAPI_IO1;
pub const IDX_ATAPI_IO1_i_DriveSelect: ULONG_PTR =
    off!(_ATAPI_REGISTERS_1__i, DriveSelect) + IDX_ATAPI_IO1;
pub const IDX_ATAPI_IO1_i_Status: ULONG_PTR = off!(_ATAPI_REGISTERS_1__i, Status) + IDX_ATAPI_IO1;

pub const IDX_ATAPI_IO1_o_Feature: ULONG_PTR = off!(_ATAPI_REGISTERS_1__o, Feature) + IDX_ATAPI_IO1;
pub const IDX_ATAPI_IO1_o_Unused0: ULONG_PTR = off!(_ATAPI_REGISTERS_1__o, Unused0) + IDX_ATAPI_IO1;
pub const IDX_ATAPI_IO1_o_Unused1: ULONG_PTR = off!(_ATAPI_REGISTERS_1__o, Unused1) + IDX_ATAPI_IO1;
pub const IDX_ATAPI_IO1_o_ByteCountLow: ULONG_PTR =
    off!(_ATAPI_REGISTERS_1__o, ByteCountLow) + IDX_ATAPI_IO1;
pub const IDX_ATAPI_IO1_o_ByteCountHigh: ULONG_PTR =
    off!(_ATAPI_REGISTERS_1__o, ByteCountHigh) + IDX_ATAPI_IO1;
pub const IDX_ATAPI_IO1_o_Command: ULONG_PTR = off!(_ATAPI_REGISTERS_1__o, Command) + IDX_ATAPI_IO1;

// ===== IDE_BUSMASTER_REGISTERS — DMA engine (5-byte window after the control block) =====
pub const IDX_BM_IO: ULONG_PTR = IDX_IO2_o + IDX_IO2_o_SZ; // 18
pub const IDX_BM_IO_SZ: ULONG_PTR = 5;
pub const IDX_BM_Command: ULONG_PTR = off!(_IDE_BUSMASTER_REGISTERS, Command) + IDX_BM_IO;
pub const IDX_BM_DeviceSpecific0: ULONG_PTR =
    off!(_IDE_BUSMASTER_REGISTERS, DeviceSpecific0) + IDX_BM_IO;
pub const IDX_BM_Status: ULONG_PTR = off!(_IDE_BUSMASTER_REGISTERS, Status) + IDX_BM_IO;
pub const IDX_BM_DeviceSpecific1: ULONG_PTR =
    off!(_IDE_BUSMASTER_REGISTERS, DeviceSpecific1) + IDX_BM_IO;
pub const IDX_BM_PRD_Table: ULONG_PTR = off!(_IDE_BUSMASTER_REGISTERS, PRD_Table) + IDX_BM_IO;

// ===== SATA SCR window (5-byte window after the BusMaster block; literal sub-offsets in C) =====
pub const IDX_SATA_IO: ULONG_PTR = IDX_BM_IO + IDX_BM_IO_SZ; // 23
pub const IDX_SATA_IO_SZ: ULONG_PTR = 5;
pub const IDX_SATA_SStatus: ULONG_PTR = 0 + IDX_SATA_IO;
pub const IDX_SATA_SError: ULONG_PTR = 1 + IDX_SATA_IO;
pub const IDX_SATA_SControl: ULONG_PTR = 2 + IDX_SATA_IO;
pub const IDX_SATA_SActive: ULONG_PTR = 3 + IDX_SATA_IO;
pub const IDX_SATA_SNTF_PMN: ULONG_PTR = 4 + IDX_SATA_IO;

// ===== Indexed-IO window + the table sentinel =====
pub const IDX_INDEXED_IO: ULONG_PTR = IDX_SATA_IO + IDX_SATA_IO_SZ; // 28
pub const IDX_INDEXED_IO_SZ: ULONG_PTR = 2;
pub const IDX_INDEXED_ADDR: ULONG_PTR = 0 + IDX_INDEXED_IO; // 28
pub const IDX_INDEXED_DATA: ULONG_PTR = 1 + IDX_INDEXED_IO; // 29
/// Number of slots in `HW_CHANNEL::RegTranslation[]` (== 30; the array is `[IORES; 30]`).
pub const IDX_MAX_REG: ULONG_PTR = IDX_INDEXED_IO + IDX_INDEXED_IO_SZ; // 30

// ===== AHCI HBA global registers (absolute byte offsets, FIELD_OFFSET into the HBA block) =====
pub const IDX_AHCI_CAP: ULONG_PTR = off!(_IDE_AHCI_REGISTERS, CAP);
pub const IDX_AHCI_GHC: ULONG_PTR = off!(_IDE_AHCI_REGISTERS, GHC);
pub const IDX_AHCI_IS: ULONG_PTR = off!(_IDE_AHCI_REGISTERS, IS);
pub const IDX_AHCI_VS: ULONG_PTR = off!(_IDE_AHCI_REGISTERS, VS);
pub const IDX_AHCI_PI: ULONG_PTR = off!(_IDE_AHCI_REGISTERS, PI);
pub const IDX_AHCI_CAP2: ULONG_PTR = off!(_IDE_AHCI_REGISTERS, CAP2);
pub const IDX_AHCI_BOHC: ULONG_PTR = off!(_IDE_AHCI_REGISTERS, BOHC);

// ===== AHCI per-port registers (FIELD_OFFSET into IDE_AHCI_PORT_REGISTERS).
// bindgen wraps the union-typed registers (CLB/FB/IS/SStatus/SControl/SError/SACT) in
// anonymous fields; the named register is the head of each anon group so its offset is
// the anon field's offset. The plain registers (IE/CMD/TFD/SIG/CI/SNTF) are direct. =====
pub const IDX_AHCI_P_CLB: ULONG_PTR = off!(_IDE_AHCI_PORT_REGISTERS, __bindgen_anon_1);
pub const IDX_AHCI_P_FB: ULONG_PTR = off!(_IDE_AHCI_PORT_REGISTERS, __bindgen_anon_2);
pub const IDX_AHCI_P_IS: ULONG_PTR = off!(_IDE_AHCI_PORT_REGISTERS, __bindgen_anon_3);
pub const IDX_AHCI_P_IE: ULONG_PTR = off!(_IDE_AHCI_PORT_REGISTERS, IE);
pub const IDX_AHCI_P_CMD: ULONG_PTR = off!(_IDE_AHCI_PORT_REGISTERS, CMD);
pub const IDX_AHCI_P_TFD: ULONG_PTR = off!(_IDE_AHCI_PORT_REGISTERS, TFD);
pub const IDX_AHCI_P_SIG: ULONG_PTR = off!(_IDE_AHCI_PORT_REGISTERS, SIG);
pub const IDX_AHCI_P_SStatus: ULONG_PTR = off!(_IDE_AHCI_PORT_REGISTERS, __bindgen_anon_4);
pub const IDX_AHCI_P_SControl: ULONG_PTR = off!(_IDE_AHCI_PORT_REGISTERS, __bindgen_anon_5);
pub const IDX_AHCI_P_SError: ULONG_PTR = off!(_IDE_AHCI_PORT_REGISTERS, __bindgen_anon_6);
pub const IDX_AHCI_P_ACT: ULONG_PTR = off!(_IDE_AHCI_PORT_REGISTERS, __bindgen_anon_7);
pub const IDX_AHCI_P_CI: ULONG_PTR = off!(_IDE_AHCI_PORT_REGISTERS, CI);
pub const IDX_AHCI_P_SNTF: ULONG_PTR = off!(_IDE_AHCI_PORT_REGISTERS, SNTF);

// ===== AHCI H2D FIS field offsets (the 'o'-side register map for AHCI command issue) =====
pub const IDX_AHCI_o_Command: ULONG_PTR = off!(_AHCI_ATA_H2D_FIS, Command);
pub const IDX_AHCI_o_Feature: ULONG_PTR = off!(_AHCI_ATA_H2D_FIS, Feature);
pub const IDX_AHCI_o_BlockNumber: ULONG_PTR = off!(_AHCI_ATA_H2D_FIS, BlockNumber);
pub const IDX_AHCI_o_CylinderLow: ULONG_PTR = off!(_AHCI_ATA_H2D_FIS, CylinderLow);
pub const IDX_AHCI_o_CylinderHigh: ULONG_PTR = off!(_AHCI_ATA_H2D_FIS, CylinderHigh);
pub const IDX_AHCI_o_DriveSelect: ULONG_PTR = off!(_AHCI_ATA_H2D_FIS, DriveSelect);
pub const IDX_AHCI_o_BlockCount: ULONG_PTR = off!(_AHCI_ATA_H2D_FIS, BlockCount);
pub const IDX_AHCI_o_Control: ULONG_PTR = off!(_AHCI_ATA_H2D_FIS, Control);
pub const IDX_AHCI_o_FeatureExp: ULONG_PTR = off!(_AHCI_ATA_H2D_FIS, FeatureExp);
pub const IDX_AHCI_o_BlockNumberExp: ULONG_PTR = off!(_AHCI_ATA_H2D_FIS, BlockNumberExp);
pub const IDX_AHCI_o_CylinderLowExp: ULONG_PTR = off!(_AHCI_ATA_H2D_FIS, CylinderLowExp);
pub const IDX_AHCI_o_CylinderHighExp: ULONG_PTR = off!(_AHCI_ATA_H2D_FIS, CylinderHighExp);
pub const IDX_AHCI_o_BlockCountExp: ULONG_PTR = off!(_AHCI_ATA_H2D_FIS, BlockCountExp);

// ===== Compile-time locks on the legacy-register chain (the boot-critical PIIX-IDE path).
// These are the well-known ATA register layout positions; any drift in the IDX_* chain
// arithmetic or a changed struct layout trips a build error rather than mis-addressing
// a register at runtime. =====
const _: () = {
    assert!(IDX_IO1_i_Data == 0);
    assert!(IDX_IO1_i_Status == 7); // status is the 8th primary register
    assert!(IDX_IO1_o_Command == 15); // command = write-side of the primary block
    assert!(IDX_IO2_AltStatus == 16);
    assert!(IDX_BM_IO == 18);
    assert!(IDX_SATA_IO == 23);
    assert!(IDX_INDEXED_DATA == 29);
    assert!(IDX_MAX_REG == 30); // == HW_CHANNEL::RegTranslation[].len()
};
