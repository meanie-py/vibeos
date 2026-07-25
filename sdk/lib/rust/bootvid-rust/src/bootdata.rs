//! ORPHAN-BY-DESIGN: VGA command streams for the parked vga.rs — the legacy
//! i386/pc INT10h path removed with the BIOS prune. Referenced by nothing else.
//!
//! Translated from drivers/base/bootvid/i386/pc/bootdata.c —
//! VGA command streams (EGA ATC init + 640x480 mode set).
//! Mechanically transliterated; opcode/port symbols resolve via
//! `crate::pc::*`. Pure data (rustconv skips non-function TUs).

#![allow(non_upper_case_globals)]
use crate::pc::*;
use ntkrnl_sys::base::USHORT;

/// `const USHORT AT_Initialization[]`
pub static AT_Initialization: [USHORT; 28] = [
    IB, VGA_BASE_IO_PORT + ATT_INITIALIZE_PORT_COLOR, METAOUT+ATCOUT, VGA_BASE_IO_PORT + ATT_ADDRESS_PORT, 16, 0, 0x00, 0x01,
    0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, IB, VGA_BASE_IO_PORT + ATT_INITIALIZE_PORT_COLOR,
    OB, VGA_BASE_IO_PORT + ATT_ADDRESS_PORT, VIDEO_ENABLE, EOD,
];

/// `const USHORT VGA_640x480[]`
pub static VGA_640x480: [USHORT; 98] = [
    OWM, VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT, VGA_NUM_SEQUENCER_PORTS, 0x0100, 0x0101, 0x0F02, 0x0003, 0x0604,
    OB, VGA_BASE_IO_PORT + MISC_OUTPUT_REG_WRITE_PORT, 0xE3, OW, VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, 0x506, OW, VGA_BASE_IO_PORT + SEQ_ADDRESS_PORT,
    0x0300, OW, VGA_BASE_IO_PORT + CRTC_ADDRESS_PORT_COLOR, 0x511, METAOUT+INDXOUT, VGA_BASE_IO_PORT + CRTC_ADDRESS_PORT_COLOR, VGA_NUM_CRTC_PORTS, 0,
    0x5F, 0x4F, 0x50, 0x82, 0x54, 0x80, 0x0B, 0x3E,
    0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xEA, 0x8C, 0xDF, 0x28, 0x00, 0xE7, 0x04, 0xE3,
    0xFF, IB, VGA_BASE_IO_PORT + ATT_INITIALIZE_PORT_COLOR, METAOUT+ATCOUT, VGA_BASE_IO_PORT + ATT_ADDRESS_PORT, VGA_NUM_ATTRIB_CONT_PORTS, 0, 0x00,
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x01,
    0x00, 0x0F, 0x00, 0x00, METAOUT+INDXOUT, VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT, VGA_NUM_GRAPH_CONT_PORTS, 0,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x0F,
    0xFF, OB, VGA_BASE_IO_PORT + DAC_PIXEL_MASK_PORT, 0xFF, IB, VGA_BASE_IO_PORT + ATT_INITIALIZE_PORT_COLOR, OB, VGA_BASE_IO_PORT + ATT_ADDRESS_PORT,
    VIDEO_ENABLE, EOD,
];
