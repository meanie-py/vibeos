//! Support surface for the i386/pc VGA bootvid translation — the
//! C headers that rustconv does not translate (functions only):
//!   - `drivers/bootvid/display.h`   screen + BV_COLOR_* constants
//!   - `drivers/base/bootvid/precomp.h`  BOOTCHAR_*, RGBQUAD, RGB/Get*Value, types
//!   - `i386/pc/vga.h`               VGA port / index register constants
//!   - `i386/pc/cmdcnst.h`           VGA command-stream opcodes
//!   - `i386/pc/pc.h`                __inpb/__outpb/… , SetPixel, globals
//!
//! Plus the cross-DLL FFI (i386 stdcall = `extern "system"`; see
//! hal.spec / ntoskrnl.spec) and the data globals defined in C as
//! plain `const`/mutable globals (rustconv skips non-function TUs).

#![allow(non_upper_case_globals)]

use ntkrnl_sys::base::*;
use core::ffi::c_void;

/// `typedef PCSTR` — pointer to const CHAR.
pub type PCSTR = *const CHAR;
/// `MEMORY_CACHING_TYPE::MmNonCached`.
pub const MmNonCached: ULONG = 0;

/// `LARGE_INTEGER` / `PHYSICAL_ADDRESS` (i386 split form; the C uses
/// `.LowPart`/`.HighPart` and the `{{lo,hi}}` aggregate init).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LARGE_INTEGER {
    pub LowPart: ULONG,
    pub HighPart: LONG,
}
pub type PHYSICAL_ADDRESS = LARGE_INTEGER;
pub type PPHYSICAL_ADDRESS = *mut LARGE_INTEGER;
pub type PULONG_PTR = *mut usize;

// --- HAL_PRIVATE_DISPATCH (ndk/haltypes.h). bootvid reaches it via the
// `HalFindBusAddressTranslation` / `HalResetDisplay` macros, which are
// `HALPRIVATEDISPATCH->field`. ntoskrnl exports the table itself
// (`@ extern HalPrivateDispatchTable`); the _NTDRIVER_ macro variant
// is `(PHAL_PRIVATE_DISPATCH)&HalPrivateDispatchTable` — address of the
// exported table either way. Layout truncated after HalResetDisplay
// (the only fields bootvid touches); accessed only via pointer.
pub type pHalFindBusAddressTranslation = Option<
    unsafe extern "system" fn(
        BusAddress: PHYSICAL_ADDRESS,
        AddressSpace: *mut ULONG,
        TranslatedAddress: PPHYSICAL_ADDRESS,
        Context: PULONG_PTR,
        NextBus: BOOLEAN,
    ) -> BOOLEAN,
>;
pub type pHalResetDisplay = Option<unsafe extern "system" fn() -> BOOLEAN>;

#[repr(C)]
pub struct HAL_PRIVATE_DISPATCH {
    pub Version: ULONG,
    pub HalHandlerForBus: *mut c_void,
    pub HalHandlerForConfigSpace: *mut c_void,
    pub HalLocateHiberRanges: *mut c_void,
    pub HalRegisterBusHandler: *mut c_void,
    pub HalSetWakeEnable: *mut c_void,
    pub HalSetWakeAlarm: *mut c_void,
    pub HalPciTranslateBusAddress: *mut c_void,
    pub HalPciAssignSlotResources: *mut c_void,
    pub HalHaltSystem: *mut c_void,
    pub HalFindBusAddressTranslation: pHalFindBusAddressTranslation,
    pub HalResetDisplay: pHalResetDisplay,
}

// Cross-DLL *data* import: a self-generated import lib only provides
// the `__imp_` thunk, not the bare symbol Rust's refptr wants — so the
// data symbol must come via raw-dylib (same fix as hal-rust kernel-data
// externs; see memory feedback_raw_dylib_cross_dll_data).
// `+verbatim` so the import descriptor is exactly `ntoskrnl.exe` (the
// kernel module name); without it raw-dylib emits `ntoskrnl.exe.dll`
// which the kernel PE loader can't resolve. Two `ntoskrnl.exe`
// descriptors (this data one + the function import-lib one) is fine
// for a kernel-loaded driver — the standard loader processes both.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    /// ntoskrnl `@ extern HalPrivateDispatchTable` — the table data.
    pub static HalPrivateDispatchTable: HAL_PRIVATE_DISPATCH;
}

/// `#define HALPRIVATEDISPATCH (&HalPrivateDispatchTable)` (driver
/// variant casts `&` of the export — same address).
#[inline]
pub unsafe fn HALPRIVATEDISPATCH() -> *const HAL_PRIVATE_DISPATCH {
    core::ptr::addr_of!(HalPrivateDispatchTable)
}

// ---------------------------------------------------------------------
// display.h — screen geometry + boot palette indices
// ---------------------------------------------------------------------
pub const SCREEN_WIDTH: ULONG = 640;
pub const SCREEN_HEIGHT: ULONG = 480;

pub const BV_COLOR_BLACK: ULONG = 0;
pub const BV_COLOR_RED: ULONG = 1;
pub const BV_COLOR_GREEN: ULONG = 2;
pub const BV_COLOR_BROWN: ULONG = 3;
pub const BV_COLOR_BLUE: ULONG = 4;
pub const BV_COLOR_MAGENTA: ULONG = 5;
pub const BV_COLOR_CYAN: ULONG = 6;
pub const BV_COLOR_DARK_GRAY: ULONG = 7;
pub const BV_COLOR_LIGHT_GRAY: ULONG = 8;
pub const BV_COLOR_LIGHT_RED: ULONG = 9;
pub const BV_COLOR_LIGHT_GREEN: ULONG = 10;
pub const BV_COLOR_YELLOW: ULONG = 11;
pub const BV_COLOR_LIGHT_BLUE: ULONG = 12;
pub const BV_COLOR_LIGHT_MAGENTA: ULONG = 13;
pub const BV_COLOR_LIGHT_CYAN: ULONG = 14;
pub const BV_COLOR_WHITE: ULONG = 15;
pub const BV_COLOR_NONE: ULONG = 16;
pub const BV_MAX_COLORS: usize = 16;

// ---------------------------------------------------------------------
// precomp.h
// ---------------------------------------------------------------------
pub const BOOTCHAR_HEIGHT: ULONG = 13;
pub const BOOTCHAR_WIDTH: ULONG = 8;
pub const BI_RGB: ULONG = 0;
pub const BI_RLE4: ULONG = 2;

/// `typedef ULONG RGBQUAD;`
pub type RGBQUAD = ULONG;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct URECT {
    pub Left: ULONG,
    pub Top: ULONG,
    pub Right: ULONG,
    pub Bottom: ULONG,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BITMAPINFOHEADER {
    pub biSize: ULONG,
    pub biWidth: LONG,
    pub biHeight: LONG,
    pub biPlanes: USHORT,
    pub biBitCount: USHORT,
    pub biCompression: ULONG,
    pub biSizeImage: ULONG,
    pub biXPelsPerMeter: LONG,
    pub biYPelsPerMeter: LONG,
    pub biClrUsed: ULONG,
    pub biClrImportant: ULONG,
}
pub type PBITMAPINFOHEADER = *mut BITMAPINFOHEADER;

#[inline]
pub const fn RGB(r: u8, g: u8, b: u8) -> RGBQUAD {
    (b as RGBQUAD) | ((g as RGBQUAD) << 8) | ((r as RGBQUAD) << 16)
}
#[inline]
pub const fn GetRValue(q: RGBQUAD) -> UCHAR {
    ((q >> 16) & 0xFF) as UCHAR
}
#[inline]
pub const fn GetGValue(q: RGBQUAD) -> UCHAR {
    ((q >> 8) & 0xFF) as UCHAR
}
#[inline]
pub const fn GetBValue(q: RGBQUAD) -> UCHAR {
    (q & 0xFF) as UCHAR
}

/// `GetFontPtr(_Char)` — CHAR_GEN_UPSIDE_DOWN is undef, so plain start.
#[inline]
pub unsafe fn GetFontPtr(ch: CHAR) -> *const UCHAR {
    crate::fontdata::VidpFontData
        .as_ptr()
        .add((ch as u8 as usize) * BOOTCHAR_HEIGHT as usize)
}
pub const FONT_PTR_DELTA: isize = 1;

// ---------------------------------------------------------------------
// vga.h — VGA register port / index constants
// ---------------------------------------------------------------------
pub const MEM_VGA: ULONG = 0xA0000;
pub const MEM_VGA_SIZE: ULONG = 0x20000;
pub const MEMORY_MAPPED_IO_OFFSET: ULONG = 0xB8000 - 0xA0000;
pub const VGA_BASE_IO_PORT: USHORT = 0x03B0;
pub const VGA_START_BREAK_PORT: USHORT = 0x03BB;
pub const VGA_END_BREAK_PORT: USHORT = 0x03C0;
pub const VGA_MAX_IO_PORT: USHORT = 0x03DF;
pub const CRTC_ADDRESS_PORT_MONO: USHORT = 0x0004;
pub const CRTC_DATA_PORT_MONO: USHORT = 0x0005;
pub const FEAT_CTRL_WRITE_PORT_MONO: USHORT = 0x000A;
pub const INPUT_STATUS_1_MONO: USHORT = 0x000A;
pub const ATT_INITIALIZE_PORT_MONO: USHORT = INPUT_STATUS_1_MONO;
pub const ATT_ADDRESS_PORT: USHORT = 0x0010;
pub const ATT_DATA_WRITE_PORT: USHORT = 0x0010;
pub const ATT_DATA_READ_PORT: USHORT = 0x0011;
pub const MISC_OUTPUT_REG_WRITE_PORT: USHORT = 0x0012;
pub const INPUT_STATUS_0_PORT: USHORT = 0x0012;
pub const VIDEO_SUBSYSTEM_ENABLE_PORT: USHORT = 0x0013;
pub const SEQ_ADDRESS_PORT: USHORT = 0x0014;
pub const SEQ_DATA_PORT: USHORT = 0x0015;
pub const DAC_PIXEL_MASK_PORT: USHORT = 0x0016;
pub const DAC_ADDRESS_READ_PORT: USHORT = 0x0017;
pub const DAC_STATE_PORT: USHORT = 0x0017;
pub const DAC_ADDRESS_WRITE_PORT: USHORT = 0x0018;
pub const DAC_DATA_REG_PORT: USHORT = 0x0019;
pub const FEAT_CTRL_READ_PORT: USHORT = 0x001A;
pub const MISC_OUTPUT_REG_READ_PORT: USHORT = 0x001C;
pub const GRAPH_ADDRESS_PORT: USHORT = 0x001E;
pub const GRAPH_DATA_PORT: USHORT = 0x001F;
pub const CRTC_ADDRESS_PORT_COLOR: USHORT = 0x0024;
pub const CRTC_DATA_PORT_COLOR: USHORT = 0x0025;
pub const FEAT_CTRL_WRITE_PORT_COLOR: USHORT = 0x002A;
pub const INPUT_STATUS_1_COLOR: USHORT = 0x002A;
pub const ATT_INITIALIZE_PORT_COLOR: USHORT = INPUT_STATUS_1_COLOR;
pub const IND_CURSOR_START: UCHAR = 0x0A;
pub const IND_CURSOR_END: UCHAR = 0x0B;
pub const IND_CURSOR_HIGH_LOC: UCHAR = 0x0E;
pub const IND_CURSOR_LOW_LOC: UCHAR = 0x0F;
pub const IND_VSYNC_END: UCHAR = 0x11;
pub const IND_CR2C: UCHAR = 0x2C;
pub const IND_CR2D: UCHAR = 0x2D;
pub const IND_SET_RESET_ENABLE: UCHAR = 0x01;
pub const IND_DATA_ROTATE: UCHAR = 0x03;
pub const IND_READ_MAP: UCHAR = 0x04;
pub const IND_GRAPH_MODE: UCHAR = 0x05;
pub const IND_GRAPH_MISC: UCHAR = 0x06;
pub const IND_BIT_MASK: UCHAR = 0x08;
pub const IND_SYNC_RESET: UCHAR = 0x00;
pub const IND_MAP_MASK: UCHAR = 0x02;
pub const IND_MEMORY_MODE: UCHAR = 0x04;
pub const IND_CRTC_PROTECT: UCHAR = 0x11;
pub const IND_CRTC_COMPAT: UCHAR = 0x34;
pub const IND_PERF_TUNING: UCHAR = 0x16;
pub const START_SYNC_RESET_VALUE: UCHAR = 0x01;
pub const END_SYNC_RESET_VALUE: UCHAR = 0x03;
pub const VIDEO_DISABLE: UCHAR = 0;
pub const VIDEO_ENABLE: USHORT = 0x20;
pub const INDEX_ENABLE_AUTO_START: UCHAR = 0x31;
pub const GRAPH_ADDR_MASK: UCHAR = 0x0F;
pub const SEQ_ADDR_MASK: UCHAR = 0x07;
pub const CHAIN4_MASK: UCHAR = 0x08;
pub const READ_MAP_TEST_SETTING: UCHAR = 0x03;
pub const MEMORY_MODE_TEXT_DEFAULT: UCHAR = 0x02;
pub const BIT_MASK_DEFAULT: UCHAR = 0xFF;
pub const READ_MAP_DEFAULT: UCHAR = 0x00;
pub const VIDEO_MAX_COLOR_REGISTER: UCHAR = 0xFF;
pub const VIDEO_MAX_PALETTE_REGISTER: UCHAR = 0x0F;
pub const DEFAULT_MODE: ULONG = 0;
pub const VGA_PLANE_SIZE: ULONG = 0x10000;
pub const VGA_NUM_SEQUENCER_PORTS: USHORT = 5;
pub const VGA_NUM_CRTC_PORTS: USHORT = 25;
pub const VGA_NUM_GRAPH_CONT_PORTS: USHORT = 9;
pub const VGA_NUM_ATTRIB_CONT_PORTS: USHORT = 21;
pub const VGA_NUM_DAC_ENTRIES: USHORT = 256;

// ---------------------------------------------------------------------
// cmdcnst.h — VGA command-stream opcodes
// ---------------------------------------------------------------------
pub const EOD: USHORT = 0x000;
pub const INOUT: USHORT = 0x010;
pub const METAOUT: USHORT = 0x020;
pub const NCMD: USHORT = 0x0f0;
pub const MULTI: USHORT = 0x02;
pub const BW: USHORT = 0x04;
pub const IO: USHORT = 0x08;
pub const INDXOUT: USHORT = 0x00;
pub const ATCOUT: USHORT = 0x01;
pub const MASKOUT: USHORT = 0x02;
pub const OB: USHORT = INOUT;
pub const OBM: USHORT = INOUT + MULTI;
pub const OW: USHORT = INOUT + BW;
pub const OWM: USHORT = INOUT + BW + MULTI;
pub const IB: USHORT = INOUT + IO;
pub const IBM: USHORT = INOUT + IO + MULTI;
pub const IW: USHORT = INOUT + IO + BW;
pub const IWM: USHORT = INOUT + IO + BW + MULTI;

// ---------------------------------------------------------------------
// Cross-DLL FFI. i386 NTAPI = stdcall → `extern "system"`
// (hal.spec / ntoskrnl.spec `@ stdcall`). DbgPrint/memset are cdecl.
// ---------------------------------------------------------------------
extern "system" {
    pub fn READ_PORT_UCHAR(port: *mut UCHAR) -> UCHAR;
    pub fn READ_PORT_USHORT(port: *mut USHORT) -> USHORT;
    pub fn WRITE_PORT_UCHAR(port: *mut UCHAR, value: UCHAR);
    pub fn WRITE_PORT_USHORT(port: *mut USHORT, value: USHORT);
    pub fn WRITE_PORT_BUFFER_USHORT(port: *mut USHORT, buffer: *const USHORT, count: ULONG);
    pub fn READ_REGISTER_UCHAR(reg: *mut UCHAR) -> UCHAR;
    pub fn WRITE_REGISTER_UCHAR(reg: *mut UCHAR, value: UCHAR);
    pub fn MmMapIoSpace(
        phys: PHYSICAL_ADDRESS,
        size: ULONG,
        cache: ULONG,
    ) -> *mut c_void;
    pub fn MmUnmapIoSpace(base: *mut c_void, size: ULONG);
}
extern "C" {
    pub fn DbgPrint(format: *const CHAR, ...) -> ULONG;
    pub fn memset(dst: *mut c_void, val: i32, n: usize) -> *mut c_void;
}

// ---------------------------------------------------------------------
// Globals (C: common.c / vga.c — plain globals; single #[no_mangle]
// definitions live here).
// ---------------------------------------------------------------------
/// `ULONG_PTR VgaRegisterBase` — port-I/O base (offset added by __in/out).
#[no_mangle]
pub static mut VgaRegisterBase: usize = 0;
/// `ULONG_PTR VgaBase` — mapped VGA framebuffer (0xA0000 window).
#[no_mangle]
pub static mut VgaBase: usize = 0;
/// `UCHAR VidpTextColor`
#[no_mangle]
pub static mut VidpTextColor: UCHAR = BV_COLOR_WHITE as UCHAR;
/// `ULONG VidpCurrentX`
#[no_mangle]
pub static mut VidpCurrentX: ULONG = 0;
/// `ULONG VidpCurrentY`
#[no_mangle]
pub static mut VidpCurrentY: ULONG = 0;
/// `URECT VidpScrollRegion`
#[no_mangle]
pub static mut VidpScrollRegion: URECT = URECT {
    Left: 0,
    Top: 0,
    Right: 0,
    Bottom: 0,
};
/// `const UCHAR PixelMask[8]` (vga.c data).
#[no_mangle]
pub static PixelMask: [UCHAR; 8] = [
    1 << 7,
    1 << 6,
    1 << 5,
    1 << 4,
    1 << 3,
    1 << 2,
    1 << 1,
    1 << 0,
];

// ---------------------------------------------------------------------
// pc.h inline port helpers — wrap hal port I/O with VgaRegisterBase.
// ---------------------------------------------------------------------
#[inline]
pub unsafe fn __inpb(port: USHORT) -> UCHAR {
    READ_PORT_UCHAR((VgaRegisterBase + port as usize) as *mut UCHAR)
}
#[inline]
pub unsafe fn __inpw(port: USHORT) -> USHORT {
    READ_PORT_USHORT((VgaRegisterBase + port as usize) as *mut USHORT)
}
#[inline]
pub unsafe fn __outpb(port: USHORT, value: UCHAR) {
    WRITE_PORT_UCHAR((VgaRegisterBase + port as usize) as *mut UCHAR, value);
}
#[inline]
pub unsafe fn __outpw(port: USHORT, value: USHORT) {
    WRITE_PORT_USHORT((VgaRegisterBase + port as usize) as *mut USHORT, value);
}

/// `__movsb` MSVC intrinsic — `rep movsb`. memmove-safe forward copy.
#[inline]
pub unsafe fn __movsb(dst: *mut UCHAR, src: *const UCHAR, count: usize) {
    core::ptr::copy(src, dst, count);
}

/// pc.h `FORCEINLINE VOID SetPixel(Left, Top, Color)`.
#[inline]
pub unsafe fn SetPixel(Left: ULONG, Top: ULONG, Color: UCHAR) {
    let pixel_position =
        (VgaBase + (Left >> 3) as usize + (Top * (SCREEN_WIDTH / 8)) as usize) as *mut UCHAR;
    __outpw(
        VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT,
        ((PixelMask[(Left & 7) as usize] as USHORT) << 8) | IND_BIT_MASK as USHORT,
    );
    let _ = READ_REGISTER_UCHAR(pixel_position);
    WRITE_REGISTER_UCHAR(pixel_position, Color);
}

/// vga.c `SET_PIXELS(_PixelPtr, _PixelMask, _TextColor)` macro.
#[inline]
pub unsafe fn SET_PIXELS(pixel_ptr: *mut UCHAR, pixel_mask: USHORT, text_color: ULONG) {
    __outpw(
        VGA_BASE_IO_PORT + GRAPH_ADDRESS_PORT,
        (pixel_mask << 8) | IND_BIT_MASK as USHORT,
    );
    let _ = READ_REGISTER_UCHAR(pixel_ptr);
    WRITE_REGISTER_UCHAR(pixel_ptr, text_color as UCHAR);
}

/// `const RGBQUAD VidpDefaultPalette[BV_MAX_COLORS]` (common.c data) —
/// CGA-like 16-color palette with R/B and dark/light-gray swapped.
#[no_mangle]
pub static VidpDefaultPalette: [RGBQUAD; BV_MAX_COLORS] = [
    RGB(0, 0, 0),       // Black
    RGB(128, 0, 0),     // Red
    RGB(0, 128, 0),     // Green
    RGB(128, 128, 0),   // Brown
    RGB(0, 0, 128),     // Blue
    RGB(128, 0, 128),   // Magenta
    RGB(0, 128, 128),   // Cyan
    RGB(128, 128, 128), // Dark Gray
    RGB(192, 192, 192), // Light Gray
    RGB(255, 0, 0),     // Light Red
    RGB(0, 255, 0),     // Light Green
    RGB(255, 255, 0),   // Yellow
    RGB(0, 0, 255),     // Light Blue
    RGB(255, 0, 255),   // Light Magenta
    RGB(0, 255, 255),   // Light Cyan
    RGB(255, 255, 255), // White
];
