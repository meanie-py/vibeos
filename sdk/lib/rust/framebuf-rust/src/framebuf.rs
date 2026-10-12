// framebuf.rs — shared "header" module for the framebuffer display driver.
//
// This is the Rust analog of framebuf.h: the driver-private PDEV plus the
// constants and externs that are NOT in the win32k-sys FFI surface.
//
// IMPORTANT: the VIDEO_* DDK structs (VIDEO_MODE_INFORMATION, VIDEO_MEMORY,
// VIDEO_CLUT, VIDEO_COLOR_CAPABILITIES, …) and all winddi/wingdi types and
// constants (DRVFN, DEVMODEW, GDIINFO, Eng*, BMF_*, PAL_*, DDI_*, …) come
// from `win32k_sys::win32k::*` — do NOT redefine them here or callers hit
// E0659 ambiguity. Only the genuinely-missing pieces live here.

use win32k_sys::win32k::*;

/// `'FUBF'` — framebuf.h `ALLOC_TAG`. C multi-char constant `'FUBF'` is
/// big-endian; the engine pool tag is the four bytes F,U,B,F. EngAllocMem
/// takes the tag as a u32.
pub const ALLOC_TAG: u32 = u32::from_be_bytes(*b"FUBF");

/// framebuf.h `DEVICE_NAME L"framebuf"` — used by DrvGetModes to stamp
/// dmDeviceName. NUL-terminated UTF-16, copied by-bytes (sizeof includes
/// the terminator, matching the C `sizeof(DEVICE_NAME)`).
pub static DEVICE_NAME: [u16; 9] = [
    b'f' as u16, b'r' as u16, b'a' as u16, b'm' as u16, b'e' as u16,
    b'b' as u16, b'u' as u16, b'f' as u16, 0,
];

// ---- IOCTL_VIDEO_* — function numbers verbatim from ntddvdeo.h ----
// (win32k-sys bindgen does not constify these IOCTL macros.) METHOD_BUFFERED
// (0) + FILE_ANY_ACCESS (0) for all. Mirrors sysfb-rust's ctl_code; the
// miniport on the other end (sysfb.sys) decodes the identical values.
const fn ctl_code(devtype: u32, func: u32, method: u32, access: u32) -> u32 {
    (devtype << 16) | (access << 14) | (func << 2) | method
}
const FILE_DEVICE_VIDEO: u32 = 0x00000023;

pub const IOCTL_VIDEO_QUERY_AVAIL_MODES: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x100, 0, 0);
pub const IOCTL_VIDEO_QUERY_NUM_AVAIL_MODES: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x101, 0, 0);
pub const IOCTL_VIDEO_SET_CURRENT_MODE: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x103, 0, 0);
pub const IOCTL_VIDEO_RESET_DEVICE: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x104, 0, 0);
pub const IOCTL_VIDEO_SET_COLOR_REGISTERS: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x107, 0, 0);
pub const IOCTL_VIDEO_MAP_VIDEO_MEMORY: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x116, 0, 0);
pub const IOCTL_VIDEO_UNMAP_VIDEO_MEMORY: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x117, 0, 0);
pub const IOCTL_VIDEO_QUERY_COLOR_CAPABILITIES: u32 = ctl_code(FILE_DEVICE_VIDEO, 0x11a, 0, 0);

/// VIDEO mode attribute flag (ntddvdeo.h) — bindgen has VIDEO_MODE_GRAPHICS
/// and VIDEO_DEVICE_COLOR, so those come from the win32k glob; nothing extra
/// needed here.

// ---- driver-private PDEV (framebuf.h `struct _PDEV`) ----
// EXPERIMENTAL_MOUSE_CURSOR_SUPPORT is off (matches the C default), so the
// pointer-cursor fields are omitted. DirectX support fields are retained for
// layout fidelity with the C struct even though dd.c is not compiled.
#[repr(C)]
pub struct PDEV {
    pub hDriver: HANDLE,
    pub hDevEng: HDEV,
    pub hSurfEng: HSURF,
    pub ModeIndex: ULONG,
    pub ScreenWidth: ULONG,
    pub ScreenHeight: ULONG,
    pub ScreenDelta: ULONG,
    pub BitsPerPixel: BYTE,
    pub RedMask: ULONG,
    pub GreenMask: ULONG,
    pub BlueMask: ULONG,
    pub PaletteShift: BYTE,
    pub ScreenPtr: PVOID,
    pub DefaultPalette: HPALETTE,
    pub PaletteEntries: *mut PALETTEENTRY,

    // DirectX support (framebuf.h tail) — kept for struct-size fidelity.
    pub iDitherFormat: DWORD,
    pub MemHeight: ULONG,
    pub MemWidth: ULONG,
    pub dwHeap: DWORD,
    pub pvmList: *mut VIDEOMEMORY,
    pub bDDInitialized: BOOL,
    pub ddpfDisplay: DDPIXELFORMAT,
}
pub type PPDEV = *mut PDEV;

// ---- externs absent from win32k-sys bindgen ----
extern "system" {
    /// win32k engine export (PALOBJ method). Returns the number of colors
    /// copied into pulColors.
    pub fn PALOBJ_cGetColors(
        ppalo: *mut PALOBJ,
        iStart: ULONG,
        cColors: ULONG,
        pulColors: *mut ULONG,
    ) -> ULONG;
}

/// `RtlZeroMemory(dst, len)` — bindgen exposes it only as a macro. framebuf
/// links libcntpr (memset), but a plain volatile-free byte fill is exactly
/// the C semantics and avoids a cross-module memset dependency here.
#[inline(always)]
pub unsafe fn RtlZeroMemory(dst: *mut core::ffi::c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}
