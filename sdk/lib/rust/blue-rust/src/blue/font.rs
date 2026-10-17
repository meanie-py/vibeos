use crate::prelude::*;
use ntkrnl_sys::ddk::{MmMapIoSpace, MmUnmapIoSpace, PHYSICAL_ADDRESS, UINT32};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// File-scope preamble of font.c: only function prototypes (OpenBitPlane, CloseBitPlane, LoadFont), no constants, types, or tables. Their Rust forms are emitted with the function definitions.
#[no_mangle]
pub unsafe extern "C" fn ScrSetFont(FontBitfield: PUCHAR) {
    let mut BaseAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut Bitplane: PUCHAR = core::ptr::null_mut();

    /* open bit plane for font table access */
    OpenBitPlane();

    /* get pointer to video memory */
    BaseAddress.QuadPart = (BITPLANE_BASE as _);
    Bitplane = MmMapIoSpace(BaseAddress, 0xFFFF, MmNonCached) as PUCHAR;

    LoadFont(Bitplane, FontBitfield);

    MmUnmapIoSpace(Bitplane as PVOID, 0xFFFF);

    /* close bit plane */
    CloseBitPlane();
}
#[no_mangle]
pub unsafe extern "C" fn OpenBitPlane() {
    /* disable interrupts */
    _disable();

    /* sequence reg */
    WRITE_PORT_UCHAR(SEQ_COMMAND, (SEQ_RESET) as UCHAR); WRITE_PORT_UCHAR(SEQ_DATA, 0x01);
    WRITE_PORT_UCHAR(SEQ_COMMAND, (SEQ_ENABLE_WRT_PLANE) as UCHAR); WRITE_PORT_UCHAR(SEQ_DATA, 0x04);
    WRITE_PORT_UCHAR(SEQ_COMMAND, (SEQ_MEM_MODE) as UCHAR); WRITE_PORT_UCHAR(SEQ_DATA, 0x07);
    WRITE_PORT_UCHAR(SEQ_COMMAND, (SEQ_RESET) as UCHAR); WRITE_PORT_UCHAR(SEQ_DATA, 0x03);

    /* graphic reg */
    WRITE_PORT_UCHAR(GCT_COMMAND, (GCT_READ_PLANE) as UCHAR); WRITE_PORT_UCHAR(GCT_DATA, 0x02);
    WRITE_PORT_UCHAR(GCT_COMMAND, (GCT_RW_MODES) as UCHAR); WRITE_PORT_UCHAR(GCT_DATA, 0x00);
    WRITE_PORT_UCHAR(GCT_COMMAND, (GCT_GRAPH_MODE) as UCHAR); WRITE_PORT_UCHAR(GCT_DATA, 0x00);

    /* enable interrupts */
    _enable();
}
#[no_mangle]
pub unsafe extern "C" fn CloseBitPlane() {
    /* disable interrupts */
    _disable();

    /* sequence reg */
    WRITE_PORT_UCHAR(SEQ_COMMAND, (SEQ_RESET) as UCHAR); WRITE_PORT_UCHAR(SEQ_DATA, 0x01);
    WRITE_PORT_UCHAR(SEQ_COMMAND, (SEQ_ENABLE_WRT_PLANE) as UCHAR); WRITE_PORT_UCHAR(SEQ_DATA, 0x03);
    WRITE_PORT_UCHAR(SEQ_COMMAND, (SEQ_MEM_MODE) as UCHAR); WRITE_PORT_UCHAR(SEQ_DATA, 0x03);
    WRITE_PORT_UCHAR(SEQ_COMMAND, (SEQ_RESET) as UCHAR); WRITE_PORT_UCHAR(SEQ_DATA, 0x03);

    /* graphic reg */
    WRITE_PORT_UCHAR(GCT_COMMAND, (GCT_READ_PLANE) as UCHAR); WRITE_PORT_UCHAR(GCT_DATA, 0x00);
    WRITE_PORT_UCHAR(GCT_COMMAND, (GCT_RW_MODES) as UCHAR); WRITE_PORT_UCHAR(GCT_DATA, 0x10);
    WRITE_PORT_UCHAR(GCT_COMMAND, (GCT_GRAPH_MODE) as UCHAR); WRITE_PORT_UCHAR(GCT_DATA, 0x0e);

    /* enable interrupts */
    _enable();
}
#[no_mangle]
pub unsafe extern "C" fn LoadFont(mut Bitplane: PUCHAR, mut FontBitfield: PUCHAR) {
    let mut i: UINT32 = 0;
    let mut j: UINT32 = 0;
    while i < 256 {
        j = 0;
        while j < 8 {
            *Bitplane = *FontBitfield.offset((i * 8 + j) as isize);
            Bitplane = Bitplane.offset(1);
            j = j + 1;
        }
        // padding
        j = 8;
        while j < 32 {
            *Bitplane = 0 as ::core::ffi::c_uchar;
            Bitplane = Bitplane.offset(1);
            j = j + 1;
        }
        i = i + 1;
    }
}
