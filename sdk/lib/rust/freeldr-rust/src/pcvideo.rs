use ntkrnl_sys::base::*;
use crate::frldr::*;
use crate::ui_keystone::{VIDEODISPLAYMODE, VideoTextMode, VideoGraphicsMode};
use ntkrnl_sys::ddk::{INT_PTR, PCSTR};
// freeldr_ddk_imports dropped (not in ddk): Depth, Init, Red
use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut};

// settings.h _BOOTMGRINFO (external freeldr C global, defined in settings.c).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BOOTMGRINFO {
    pub DebugString: PCSTR,
    pub VideoOptions: PCSTR,
    pub DefaultOs: PCSTR,
    pub TimeOut: LONG,
    pub FrLdrSection: ULONG_PTR,
}

extern "C" {
    fn Int386(vector: i32, in_regs: *mut REGS, out_regs: *mut REGS);
    fn WRITE_PORT_UCHAR(Port: *mut UCHAR, Value: UCHAR);
    fn WRITE_PORT_USHORT(Port: *mut USHORT, Value: USHORT);
    fn READ_PORT_UCHAR(Port: *mut UCHAR) -> UCHAR;
    fn TRACE(fmt: *const i8, ...);
    // freeldr debug printf family (external C: debug.c / ui). cdecl variadic.
    // TRACE_printf/printf are #[no_mangle]-defined in tui.rs (non-variadic
    // cdecl defs; caller-cleans → variadic call sites here are ABI-safe).
    fn TRACE_printf(fmt: PCSTR, ...);
    fn printf(fmt: PCSTR, ...) -> i32;
    // pccons.c console input (external C, not in pc-hw unit).
    fn PcConsGetCh() -> i32;
    // crt string/stdlib (external freeldr libcntpr).
    fn _strnicmp(s1: PCSTR, s2: PCSTR, n: usize) -> i32;
    fn strcspn(s: PCSTR, control: PCSTR) -> usize;
    fn strtoul(s: PCSTR, endptr: *mut *mut i8, radix: i32) -> ULONG;
    // settings.c global.
    static BootMgrInfo: BOOTMGRINFO;
}

// ---- pcvideo.c file-local #define constants (dropped by rustconv) ----
const VIDEOPORT_PALETTE_READ: u16 = 0x03C7;
const VIDEOPORT_PALETTE_WRITE: u16 = 0x03C8;
const VIDEOPORT_PALETTE_DATA: u16 = 0x03C9;
const VIDEOPORT_VERTICAL_RETRACE: u16 = 0x03DA;

const VIDEOVGA_MEM_ADDRESS: u32 = 0xA0000;
const VIDEOTEXT_MEM_ADDRESS: u32 = 0xB8000;
const VIDEOTEXT_MEM_SIZE: u32 = 0x8000;

const VIDEOCARD_CGA_OR_OTHER: ULONG = 0;
const VIDEOCARD_EGA: ULONG = 1;
const VIDEOCARD_VGA: ULONG = 2;

const VIDEOMODE_NORMAL_TEXT: USHORT = 0;
const VIDEOMODE_EXTENDED_TEXT: USHORT = 1;
const VIDEOMODE_80X28: USHORT = 0x501C;
const VIDEOMODE_80X30: USHORT = 0x501E;
const VIDEOMODE_80X34: USHORT = 0x5022;
const VIDEOMODE_80X43: USHORT = 0x502B;
const VIDEOMODE_80X60: USHORT = 0x503C;
const VIDEOMODE_132X25: USHORT = 0x8419;
const VIDEOMODE_132X43: USHORT = 0x842B;
const VIDEOMODE_132X50: USHORT = 0x8432;
const VIDEOMODE_132X60: USHORT = 0x843C;

const VERTRES_200_SCANLINES: UCHAR = 0x00;
const VERTRES_350_SCANLINES: UCHAR = 0x01;
const VERTRES_400_SCANLINES: UCHAR = 0x02;

// ---- pcvideo.c file-local font-type enum (dropped by rustconv) ----
const INT1FhFont: UCHAR = 0x00;
const INT43hFont: UCHAR = 0x01;
const ROM_8x14CharacterFont: UCHAR = 0x02;
const ROM_8x8DoubleDotFontLo: UCHAR = 0x03;
const ROM_8x8DoubleDotFontHi: UCHAR = 0x04;
const ROM_AlphaAlternate: UCHAR = 0x05;
const ROM_8x16Font: UCHAR = 0x06;
const ROM_Alternate9x16Font: UCHAR = 0x07;
const UltraVision_8x20Font: UCHAR = 0x11;
const UltraVision_8x10Font: UCHAR = 0x12;

// ---- pcvideo.c SVGA_MODE_INFORMATION (#include <pshpack2.h> ... poppack) ----
#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct SVGA_MODE_INFORMATION {
    pub ModeAttributes: USHORT,
    pub WindowAttributesA: UCHAR,
    pub WindowsAttributesB: UCHAR,
    pub WindowGranularity: USHORT,
    pub WindowSize: USHORT,
    pub WindowAStartSegment: USHORT,
    pub WindowBStartSegment: USHORT,
    pub WindowPositioningFunction: ULONG,
    pub BytesPerScanLine: USHORT,
    pub WidthInPixels: USHORT,
    pub HeightInPixels: USHORT,
    pub CharacterWidthInPixels: UCHAR,
    pub CharacterHeightInPixels: UCHAR,
    pub NumberOfMemoryPlanes: UCHAR,
    pub BitsPerPixel: UCHAR,
    pub NumberOfBanks: UCHAR,
    pub MemoryModel: UCHAR,
    pub BankSize: UCHAR,
    pub NumberOfImagePanes: UCHAR,
    pub Reserved1: UCHAR,
    pub RedMaskSize: UCHAR,
    pub RedMaskPosition: UCHAR,
    pub GreenMaskSize: UCHAR,
    pub GreenMaskPosition: UCHAR,
    pub BlueMaskSize: UCHAR,
    pub BlueMaskPosition: UCHAR,
    pub ReservedMaskSize: UCHAR,
    pub ReservedMaskPosition: UCHAR,
    pub DirectColorModeInfo: UCHAR,
    pub LinearVideoBufferAddress: ULONG,
    pub OffscreenMemoryPointer: ULONG,
    pub OffscreenMemorySize: USHORT,
    pub LinearBytesPerScanLine: USHORT,
    pub BankedNumberOfImages: UCHAR,
    pub LinearNumberOfImages: UCHAR,
    pub LinearRedMaskSize: UCHAR,
    pub LinearRedMaskPosition: UCHAR,
    pub LinearGreenMaskSize: UCHAR,
    pub LinearGreenMaskPosition: UCHAR,
    pub LinearBlueMaskSize: UCHAR,
    pub LinearBlueMaskPosition: UCHAR,
    pub LinearReservedMaskSize: UCHAR,
    pub LinearReservedMaskPosition: UCHAR,
    pub MaximumPixelClock: ULONG,
    pub Reserved2: [UCHAR; 190],
}
pub type PSVGA_MODE_INFORMATION = *mut SVGA_MODE_INFORMATION;
const _: () = assert!(core::mem::size_of::<SVGA_MODE_INFORMATION>() == 256);

// ---- pcvideo.c file-local statics (dropped by rustconv) ----
static mut VideoCard: ULONG = VIDEOCARD_CGA_OR_OTHER;
static mut BiosVideoMode: USHORT = VIDEOMODE_NORMAL_TEXT;
static mut ScreenWidth: ULONG = 80;
static mut ScreenHeight: ULONG = 25;
static mut BytesPerScanLine: ULONG = 160;
static mut DisplayMode: VIDEODISPLAYMODE = VideoTextMode;
static mut VesaVideoMode: BOOLEAN = FALSE as BOOLEAN;
static mut VesaVideoModeInformation: SVGA_MODE_INFORMATION =
    unsafe { core::mem::zeroed() };
static mut CurrentMemoryBank: ULONG = 0;

unsafe fn PcVideoDetectVideoCard() -> ULONG {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.b.ah = 0x12;
    Regs.b.bl = 0x10;
    Int386(0x10, &mut Regs, &mut Regs);

    if Regs.b.bl == 0x10 {
        return VIDEOCARD_CGA_OR_OTHER;
    }

    Regs.w.ax = 0x1A00;
    Int386(0x10, &mut Regs, &mut Regs);

    if Regs.b.al == 0x1a {
        return VIDEOCARD_VGA;
    } else {
        return VIDEOCARD_EGA;
    }
}

unsafe fn PcVideoVesaGetSVGAModeInformation(
    Mode: USHORT,
    ModeInformation: PSVGA_MODE_INFORMATION
) -> BOOLEAN {
    let mut Regs: REGS = core::mem::zeroed();

    core::ptr::write_bytes(BIOSCALLBUFFER as *mut u8, 0, core::mem::size_of::<SVGA_MODE_INFORMATION>());

    Regs.w.ax = 0x4F01;
    Regs.w.cx = Mode;
    Regs.w.es = (BIOSCALLBUFSEGMENT) as u16 ;
    Regs.w.di = (BIOSCALLBUFOFFSET) as u16 ;
    Int386(0x10, &mut Regs, &mut Regs);

    if Regs.w.ax != 0x004F {
        return 0u8;
    }

    core::ptr::copy_nonoverlapping(
        BIOSCALLBUFFER as *const u8,
        ModeInformation as *mut u8,
        core::mem::size_of::<SVGA_MODE_INFORMATION>()
    );

    TRACE(b"\n\0".as_ptr() as PCHAR);
    TRACE(b"BiosVesaGetSVGAModeInformation(Mode 0x%x)\n\0".as_ptr() as PCHAR, (Mode) as usize);
    TRACE(b"ModeAttributes = 0x%04x\n\0".as_ptr() as PCHAR, ((*ModeInformation).ModeAttributes) as usize);
    TRACE(b"WindowAttributesA = 0x%02x\n\0".as_ptr() as PCHAR, ((*ModeInformation).WindowAttributesA) as usize);
    TRACE(b"WindowAttributesB = 0x%02x\n\0".as_ptr() as PCHAR, ((*ModeInformation).WindowsAttributesB) as usize);
    TRACE(b"WindowGranularity = %huKB\n\0".as_ptr() as PCHAR, ((*ModeInformation).WindowGranularity) as usize);
    TRACE(b"WindowSize = %huKB\n\0".as_ptr() as PCHAR, ((*ModeInformation).WindowSize) as usize);
    TRACE(b"WindowAStartSegment = 0x%04x\n\0".as_ptr() as PCHAR, ((*ModeInformation).WindowAStartSegment) as usize);
    TRACE(b"WindowBStartSegment = 0x%04x\n\0".as_ptr() as PCHAR, ((*ModeInformation).WindowBStartSegment) as usize);
    TRACE(b"WindowPositioningFunction = 0x%x\n\0".as_ptr() as PCHAR, ((*ModeInformation).WindowPositioningFunction) as usize);
    TRACE(b"BytesPerScanLine = %hu\n\0".as_ptr() as PCHAR, ((*ModeInformation).BytesPerScanLine) as usize);
    TRACE(b"WidthInPixels = %hu\n\0".as_ptr() as PCHAR, ((*ModeInformation).WidthInPixels) as usize);
    TRACE(b"HeightInPixels = %hu\n\0".as_ptr() as PCHAR, ((*ModeInformation).HeightInPixels) as usize);
    TRACE(b"CharacterWidthInPixels = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).CharacterWidthInPixels) as usize);
    TRACE(b"CharacterHeightInPixels = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).CharacterHeightInPixels) as usize);
    TRACE(b"NumberOfMemoryPlanes = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).NumberOfMemoryPlanes) as usize);
    TRACE(b"BitsPerPixel = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).BitsPerPixel) as usize);
    TRACE(b"NumberOfBanks = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).NumberOfBanks) as usize);
    TRACE(b"MemoryModel = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).MemoryModel) as usize);
    TRACE(b"BankSize = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).BankSize) as usize);
    TRACE(b"NumberOfImagePanes = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).NumberOfImagePanes) as usize);
    TRACE(b"Reserved1 = 0x%02x\n\0".as_ptr() as PCHAR, ((*ModeInformation).Reserved1) as usize);
    TRACE(b"---VBE v1.2+ ---\n\0".as_ptr() as PCHAR);
    TRACE(b"RedMaskSize = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).RedMaskSize) as usize);
    TRACE(b"RedMaskPosition = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).RedMaskPosition) as usize);
    TRACE(b"GreenMaskSize = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).GreenMaskSize) as usize);
    TRACE(b"GreenMaskPosition = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).GreenMaskPosition) as usize);
    TRACE(b"BlueMaskSize = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).BlueMaskSize) as usize);
    TRACE(b"BlueMaskPosition = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).BlueMaskPosition) as usize);
    TRACE(b"ReservedMaskSize = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).ReservedMaskSize) as usize);
    TRACE(b"ReservedMaskPosition = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).ReservedMaskPosition) as usize);
    TRACE(b"DirectColorModeInfo = 0x%02x\n\0".as_ptr() as PCHAR, ((*ModeInformation).DirectColorModeInfo) as usize);
    TRACE(b"---VBE v2.0+ ---\n\0".as_ptr() as PCHAR);
    TRACE(b"LinearVideoBufferAddress = 0x%x\n\0".as_ptr() as PCHAR, ((*ModeInformation).LinearVideoBufferAddress) as usize);
    TRACE(b"OffscreenMemoryPointer = 0x%x\n\0".as_ptr() as PCHAR, ((*ModeInformation).OffscreenMemoryPointer) as usize);
    TRACE(b"OffscreenMemorySize = %huKB\n\0".as_ptr() as PCHAR, ((*ModeInformation).OffscreenMemorySize) as usize);
    TRACE(b"---VBE v3.0 ---\n\0".as_ptr() as PCHAR);
    TRACE(b"LinearBytesPerScanLine = %hu\n\0".as_ptr() as PCHAR, ((*ModeInformation).LinearBytesPerScanLine) as usize);
    TRACE(b"BankedNumberOfImages = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).BankedNumberOfImages) as usize);
    TRACE(b"LinearNumberOfImages = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).LinearNumberOfImages) as usize);
    TRACE(b"LinearRedMaskSize = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).LinearRedMaskSize) as usize);
    TRACE(b"LinearRedMaskPosition = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).LinearRedMaskPosition) as usize);
    TRACE(b"LinearGreenMaskSize = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).LinearGreenMaskSize) as usize);
    TRACE(b"LinearGreenMaskPosition = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).LinearGreenMaskPosition) as usize);
    TRACE(b"LinearBlueMaskSize = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).LinearBlueMaskSize) as usize);
    TRACE(b"LinearBlueMaskPosition = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).LinearBlueMaskPosition) as usize);
    TRACE(b"LinearReservedMaskSize = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).LinearReservedMaskSize) as usize);
    TRACE(b"LinearReservedMaskPosition = %u\n\0".as_ptr() as PCHAR, ((*ModeInformation).LinearReservedMaskPosition) as usize);
    TRACE(b"MaximumPixelClock = %luHz\n\0".as_ptr() as PCHAR, ((*ModeInformation).MaximumPixelClock) as usize);
    TRACE(b"\n\0".as_ptr() as PCHAR);

    1u8
}

unsafe fn PcVideoVesaGetCurrentSVGAMode(
    Mode: PUSHORT
) -> BOOLEAN {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.w.ax = 0x4F03;
    Int386(0x10, &mut Regs, &mut Regs);

    if Regs.w.ax != 0x004F {
        return 0u8;
    }

    *Mode = Regs.w.bx;
    1u8
}

unsafe fn PcVideoGetBiosMode(
    Mode: PUSHORT
) -> BOOLEAN {
    let mut Regs: REGS = core::mem::zeroed();

    (*addr_of_mut!(Regs.b.ah)) = 0x0F;
    Int386(0x10, &mut Regs, &mut Regs);

    if (*addr_of!(Regs.b.ah)) == 0 {
        return 0u8;
    }

    *Mode = (*addr_of!(Regs.b.al)) as u16;
    1u8
}

unsafe fn PcVideoGetDisplayMode() {
    let mut Mode: USHORT = 0;

    if PcVideoGetBiosMode(&mut Mode) != 0 &&
       PcVideoVesaGetSVGAModeInformation(Mode, &mut VesaVideoModeInformation) != 0
    {
        TRACE(b"VESA mode detected\n\0".as_ptr() as *const i8);
        ScreenWidth = (VesaVideoModeInformation.WidthInPixels) as u32 ;
        ScreenHeight = (VesaVideoModeInformation.HeightInPixels) as u32 ;
        BytesPerScanLine = (VesaVideoModeInformation.BytesPerScanLine) as u32 ;
        BiosVideoMode = Mode;
        DisplayMode = if 0x0108 <= Mode && Mode <= 0x010C {
            VideoTextMode
        } else {
            VideoGraphicsMode
        };
        VesaVideoMode = 1u8;
    }
    else if PcVideoGetBiosMode(&mut Mode) != 0
    {

        let _BiosMode = (*(0x449 as *const UCHAR)) & 0x7F;
        BiosVideoMode = Mode;
        TRACE(b"BIOS mode detected\n\0".as_ptr() as *const i8);

        ScreenWidth = (*(0x44A as *const USHORT)) as u32 ;
        ScreenHeight = 1 + *(0x484 as *const UCHAR) as ULONG;

        // Text mode: one byte for the character, one for the attribute. (Upstream
        // bd881161ac2: BDA byte 0x489 holds the vertical scan-line count, not a row stride.)
        BytesPerScanLine = ScreenWidth * 2;

        DisplayMode = VideoTextMode;
        VesaVideoMode = 0u8;
    }
    else
    {
        TRACE(b"Fallback mode detected\n\0".as_ptr() as *const i8);
        BiosVideoMode = VIDEOMODE_NORMAL_TEXT;
        ScreenWidth = 80;
        ScreenHeight = 25;
        BytesPerScanLine = ScreenWidth * 2;
        DisplayMode = VideoTextMode;
        VesaVideoMode = 0u8;
    }

    TRACE(
        b"This is a %s video mode\n\0".as_ptr() as *const i8, (if VesaVideoMode != 0 { b"VESA\0".as_ptr() } else { b"BIOS\0".as_ptr() }) as usize);
    TRACE(b"BiosVideoMode    = 0x%x\n\0".as_ptr() as *const i8, (BiosVideoMode) as usize);
    TRACE(
        b"DisplayMode      = %sMode\n\0".as_ptr() as *const i8, (if DisplayMode == VideoTextMode { b"Text\0".as_ptr() } else { b"Graphics\0".as_ptr() }) as usize);
    TRACE(b"ScreenWidth      = %lu\n\0".as_ptr() as *const i8, (ScreenWidth) as usize);
    TRACE(b"ScreenHeight     = %lu\n\0".as_ptr() as *const i8, (ScreenHeight) as usize);
    TRACE(b"BytesPerScanLine = %lu\n\0".as_ptr() as *const i8, (BytesPerScanLine) as usize);
}

unsafe fn PcVideoSetBiosVesaMode(Mode: USHORT) -> BOOLEAN {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.w.ax = 0x4F02;
    Regs.w.bx = Mode;
    Int386(0x10, &mut Regs, &mut Regs);
    if Regs.w.ax == 0x004F { 1u8 } else { 0u8 }
}

unsafe fn PcVideoSetBiosMode(VideoMode: UCHAR) {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.b.ah = 0x00;
    Regs.b.al = VideoMode;
    Int386(0x10, &mut Regs, &mut Regs);
}

unsafe fn PcVideoSetFont8x8() {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.w.ax = 0x1112;
    Regs.b.bl = 0x00;
    Int386(0x10, &mut Regs, &mut Regs);
}

unsafe fn PcVideoSetFont8x14() {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.w.ax = 0x1111;
    Regs.b.bl = 0;
    Int386(0x10, &mut Regs, &mut Regs);
}

unsafe fn PcVideoSelectAlternatePrintScreen() {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.b.ah = 0x12;
    Regs.b.bl = 0x20;
    Int386(0x10, &mut Regs as *mut REGS, &mut Regs as *mut REGS);
}

unsafe fn PcVideoDisableCursorEmulation() {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.b.ah = 0x12;
    Regs.b.bl = 0x34;
    Regs.b.al = 0x01;
    Int386(0x10, &mut Regs, &mut Regs);
}

unsafe fn PcVideoDefineCursor(StartScanLine: UCHAR, EndScanLine: UCHAR) {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.b.ah = 0x01;
    Regs.b.al = 0x03;
    Regs.b.ch = StartScanLine;
    Regs.b.cl = EndScanLine;
    Int386(0x10, &mut Regs, &mut Regs);
}

unsafe fn PcVideoSetVerticalResolution(VerticalResolutionMode: u8) {
    let mut Regs: REGS = core::mem::zeroed();
    Regs.b.ah = 0x12;
    Regs.b.bl = 0x30;
    Regs.b.al = VerticalResolutionMode;
    Int386(0x10, &mut Regs, &mut Regs);
}

unsafe fn PcVideoSet480ScanLines() {
    let mut CRTC: INT_PTR;

    CRTC = READ_PORT_UCHAR(0x03CC as *mut u8) as INT_PTR;
    if (CRTC & 1) != 0 {
        CRTC = 0x3D4;
    } else {
        CRTC = 0x3B4;
    }

    WRITE_PORT_UCHAR(CRTC as *mut u8, 0x11);
    WRITE_PORT_UCHAR((CRTC + 1) as *mut u8, 0x0C);

    WRITE_PORT_UCHAR(CRTC as *mut u8, 0x06);
    WRITE_PORT_UCHAR((CRTC + 1) as *mut u8, 0x0B);

    WRITE_PORT_UCHAR(CRTC as *mut u8, 0x07);
    WRITE_PORT_UCHAR((CRTC + 1) as *mut u8, 0x3E);

    WRITE_PORT_UCHAR(CRTC as *mut u8, 0x10);
    WRITE_PORT_UCHAR((CRTC + 1) as *mut u8, 0xEA);

    WRITE_PORT_UCHAR(CRTC as *mut u8, 0x12);
    WRITE_PORT_UCHAR((CRTC + 1) as *mut u8, 0xDF);

    WRITE_PORT_UCHAR(CRTC as *mut u8, 0x15);
    WRITE_PORT_UCHAR((CRTC + 1) as *mut u8, 0xE7);

    WRITE_PORT_UCHAR(CRTC as *mut u8, 0x16);
    WRITE_PORT_UCHAR((CRTC + 1) as *mut u8, 0x04);

    CRTC = READ_PORT_UCHAR(0x03CC as *mut u8) as INT_PTR;

    CRTC = CRTC & 0x0D;
    CRTC = CRTC | 0xE2;

    WRITE_PORT_UCHAR(0x03C2 as *mut u8, CRTC as u8);
}

unsafe fn PcVideoSetDisplayEnd() {

    let mut CRTC: INT_PTR;

    CRTC = READ_PORT_UCHAR(0x03CC as *mut u8) as INT_PTR;
    if CRTC & 1 != 0 {
        CRTC = 0x3D4;
    } else {
        CRTC = 0x3B4;
    }

    WRITE_PORT_UCHAR(CRTC as *mut u8, 0x12);
    WRITE_PORT_UCHAR((CRTC + 1) as *mut u8, 0xDF);
}

unsafe fn PcVideoSetMode80x25() -> BOOLEAN {
    PcVideoSetBiosMode(0x03);
    ScreenWidth = 80;
    ScreenHeight = 25;
    BytesPerScanLine = ScreenWidth * 2;
    return 1u8;
}

unsafe fn PcVideoSetMode80x50_80x43() -> BOOLEAN {
    if VideoCard == VIDEOCARD_VGA {
        PcVideoSetBiosMode(0x03);
        PcVideoSetFont8x8();
        PcVideoSelectAlternatePrintScreen();
        PcVideoDisableCursorEmulation();
        PcVideoDefineCursor(6, 7);
        ScreenWidth = 80;
        ScreenHeight = 50;
        BytesPerScanLine = ScreenWidth * 2;
    } else if VideoCard == VIDEOCARD_EGA {
        PcVideoSetBiosMode(0x03);
        PcVideoSetFont8x8();
        PcVideoSelectAlternatePrintScreen();
        PcVideoDisableCursorEmulation();
        PcVideoDefineCursor(6, 7);
        ScreenWidth = 80;
        ScreenHeight = 43;
        BytesPerScanLine = ScreenWidth * 2;
    } else {
        return 0u8;
    }

    return 1u8;
}

unsafe fn PcVideoSetMode80x28() -> BOOLEAN {
    PcVideoSetMode80x25();
    PcVideoSetFont8x14();
    PcVideoDefineCursor(11, 12);
    ScreenWidth = 80;
    ScreenHeight = 28;
    BytesPerScanLine = ScreenWidth * 2;
    return 1u8;
}

unsafe fn PcVideoSetMode80x30() -> BOOLEAN {
    PcVideoSetMode80x25();
    PcVideoSet480ScanLines();
    ScreenWidth = 80;
    ScreenHeight = 30;
    BytesPerScanLine = ScreenWidth * 2;
    return 1u8;
}

unsafe fn PcVideoSetMode80x34() -> BOOLEAN {
    PcVideoSetMode80x25();
    PcVideoSet480ScanLines();
    PcVideoSetFont8x14();
    PcVideoDefineCursor(11, 12);
    PcVideoSetDisplayEnd();
    ScreenWidth = 80;
    ScreenHeight = 34;
    BytesPerScanLine = ScreenWidth * 2;
    return 1u8;
}

unsafe fn PcVideoSetMode80x43() -> BOOLEAN {
    PcVideoSetVerticalResolution(VERTRES_350_SCANLINES);
    PcVideoSetMode80x25();
    PcVideoSetFont8x8();
    PcVideoSelectAlternatePrintScreen();
    PcVideoDisableCursorEmulation();
    PcVideoDefineCursor(6, 7);
    ScreenWidth = 80;
    ScreenHeight = 43;
    BytesPerScanLine = ScreenWidth * 2;
    return 1u8;
}

unsafe fn PcVideoSetMode80x60() -> BOOLEAN {
    PcVideoSetMode80x25();
    PcVideoSet480ScanLines();
    PcVideoSetFont8x8();
    PcVideoSelectAlternatePrintScreen();
    PcVideoDisableCursorEmulation();
    PcVideoDefineCursor(6, 7);
    PcVideoSetDisplayEnd();
    ScreenWidth = 80;
    ScreenHeight = 60;
    BytesPerScanLine = ScreenWidth * 2;
    return 1u8;
}

unsafe fn PcVideoSetMode(NewMode: USHORT) -> BOOLEAN {
    CurrentMemoryBank = 0;

    BiosVideoMode = NewMode;
    ScreenWidth = 80;
    ScreenHeight = 25;
    BytesPerScanLine = 160;
    DisplayMode = VideoTextMode;
    VesaVideoMode = 0u8;

    match NewMode {
        VIDEOMODE_NORMAL_TEXT | 0x03 => {
            return PcVideoSetMode80x25();
        }
        VIDEOMODE_EXTENDED_TEXT => {
            return PcVideoSetMode80x50_80x43();
        }
        VIDEOMODE_80X28 => {
            return PcVideoSetMode80x28();
        }
        VIDEOMODE_80X30 => {
            return PcVideoSetMode80x30();
        }
        VIDEOMODE_80X34 => {
            return PcVideoSetMode80x34();
        }
        VIDEOMODE_80X43 => {
            return PcVideoSetMode80x43();
        }
        VIDEOMODE_80X60 => {
            return PcVideoSetMode80x60();
        }
        _ => {}
    }

    if NewMode == 0x12 {
        PcVideoSetBiosMode(NewMode as UCHAR);
        WRITE_PORT_USHORT(0x03CE as *mut USHORT, 0x0F01);
        ScreenWidth = 640;
        ScreenHeight = 480;
        BytesPerScanLine = 80;
        BiosVideoMode = NewMode;
        DisplayMode = VideoGraphicsMode;

        return 1u8;
    } else if NewMode == 0x13 {
        PcVideoSetBiosMode(NewMode as UCHAR);
        ScreenWidth = 320;
        ScreenHeight = 200;
        BytesPerScanLine = 320;
        BiosVideoMode = NewMode;
        DisplayMode = VideoGraphicsMode;

        return 1u8;
    } else {
        if PcVideoVesaGetSVGAModeInformation(NewMode, core::ptr::addr_of_mut!(VesaVideoModeInformation)) == 0 {
            return 0u8;
        }

        if PcVideoSetBiosVesaMode(NewMode) == 0 {
            return 0u8;
        }

        ScreenWidth = (VesaVideoModeInformation.WidthInPixels) as u32 ;
        ScreenHeight = (VesaVideoModeInformation.HeightInPixels) as u32 ;
        BytesPerScanLine = (VesaVideoModeInformation.BytesPerScanLine) as u32 ;
        BiosVideoMode = NewMode;
        DisplayMode = if 0x0108 <= NewMode && NewMode <= 0x010C { VideoTextMode } else { VideoGraphicsMode };
        VesaVideoMode = 1u8;

        return 1u8;
    }
}

unsafe fn PcVideoSetBlinkBit(Enable: BOOLEAN) {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.w.ax = 0x1003;
    Regs.w.bx = if Enable != 0 { 0x0001 } else { 0x0000 };
    Int386(0x10, &mut Regs, &mut Regs);
}

unsafe fn PcVideoSetMemoryBank(BankNumber: USHORT) {
    let mut Regs: REGS = core::mem::zeroed();

    if CurrentMemoryBank == ((BankNumber) as u32) {
        return;
    }

    Regs.w.ax = 0x4F05;
    Regs.w.bx = 0x0000;
    Regs.w.dx = BankNumber;
    Int386(0x10, &mut Regs, &mut Regs);

    if Regs.w.ax == 0x004F {
        CurrentMemoryBank = (BankNumber) as u32 ;
    }
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoInit() {
    VideoCard = PcVideoDetectVideoCard();

    PcVideoGetDisplayMode();

    if !BootMgrInfo.VideoOptions.is_null() && *BootMgrInfo.VideoOptions != 0 {
        PcVideoSetDisplayMode(BootMgrInfo.VideoOptions, 1u8);
    }

    if DisplayMode != VideoTextMode {
        PcVideoSetMode(VIDEOMODE_NORMAL_TEXT);
    }
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoSetDisplayMode(DisplayModeName: PCSTR, Init: BOOLEAN) -> VIDEODISPLAYMODE {
    let mut VideoMode: USHORT = VIDEOMODE_NORMAL_TEXT;

    if DisplayModeName.is_null() || *DisplayModeName == 0 {
        PcVideoSetBlinkBit(if Init != 0 { 0u8 } else { 1u8 });
        return DisplayMode;
    }

    if VideoCard == VIDEOCARD_CGA_OR_OTHER {
        TRACE_printf(b"CGA or other display adapter detected.\n\0".as_ptr() as PCSTR);
        TRACE_printf(b"Using 80x25 text mode.\n\0".as_ptr() as PCSTR);
        VideoMode = VIDEOMODE_NORMAL_TEXT;
    } else if VideoCard == VIDEOCARD_EGA {
        TRACE_printf(b"EGA display adapter detected.\n\0".as_ptr() as PCSTR);
        TRACE_printf(b"Using 80x25 text mode.\n\0".as_ptr() as PCSTR);
        VideoMode = VIDEOMODE_NORMAL_TEXT;
    } else {
        TRACE_printf(b"VGA display adapter detected.\n\0".as_ptr() as PCSTR);

        let NameLen: usize = strcspn(DisplayModeName, b",\0".as_ptr() as PCSTR);
        if NameLen == 0 {
            PcVideoSetBlinkBit(if Init != 0 { 0u8 } else { 1u8 });
            return DisplayMode;
        }

        if _strnicmp(DisplayModeName, b"NORMAL_VGA\0".as_ptr() as PCSTR, NameLen) == 0 {
            VideoMode = VIDEOMODE_NORMAL_TEXT;
        } else if _strnicmp(DisplayModeName, b"EXTENDED_VGA\0".as_ptr() as PCSTR, NameLen) == 0 {
            VideoMode = VIDEOMODE_EXTENDED_TEXT;
        } else {
            VideoMode = strtoul(DisplayModeName, core::ptr::null_mut(), 0) as USHORT;
        }
    }

    if PcVideoSetMode(VideoMode) == 0 {
        TRACE_printf(b"Error: unable to set video display mode 0x%x\n\0".as_ptr() as PCSTR, VideoMode as u32);
        TRACE_printf(b"Defaulting to 80x25 text mode.\n\0".as_ptr() as PCSTR);
        printf(b"Press any key to continue.\n\0".as_ptr() as PCSTR);
        PcConsGetCh();

        PcVideoSetMode(VIDEOMODE_NORMAL_TEXT);
    }

    PcVideoSetBlinkBit(if Init != 0 { 0u8 } else { 1u8 });
    DisplayMode
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoGetDisplaySize(Width: PULONG, Height: PULONG, Depth: PULONG) {
    *Width = ScreenWidth;
    *Height = ScreenHeight;
    if VideoGraphicsMode == DisplayMode && VesaVideoMode != 0 {
        if VesaVideoModeInformation.BitsPerPixel == 16 {
            *Depth = if VesaVideoModeInformation.GreenMaskSize == 6 { 16 } else { 15 };
        } else {
            *Depth = VesaVideoModeInformation.BitsPerPixel as u32;
        }
    } else {
        *Depth = 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoGetBufferSize() -> ULONG {
    ScreenHeight * BytesPerScanLine
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoGetFontsFromFirmware(RomFontPointers: PULONG) {
    let mut BiosRegs: REGS = core::mem::zeroed();

    BiosRegs.d.eax = 0x1130;
    BiosRegs.b.bh = ROM_8x14CharacterFont;
    Int386(0x10, &mut BiosRegs, &mut BiosRegs);
    *RomFontPointers.offset(0) = ((BiosRegs.w.es as u32) << 4) | (BiosRegs.w.bp as u32);

    BiosRegs.b.bh = ROM_8x8DoubleDotFontLo;
    Int386(0x10, &mut BiosRegs, &mut BiosRegs);
    *RomFontPointers.offset(1) = ((BiosRegs.w.es as u32) << 16) | (BiosRegs.w.bp as u32);

    BiosRegs.b.bh = ROM_8x8DoubleDotFontHi;
    Int386(0x10, &mut BiosRegs, &mut BiosRegs);
    *RomFontPointers.offset(2) = ((BiosRegs.w.es as u32) << 16) | (BiosRegs.w.bp as u32);

    BiosRegs.b.bh = ROM_AlphaAlternate;
    Int386(0x10, &mut BiosRegs, &mut BiosRegs);
    *RomFontPointers.offset(3) = ((BiosRegs.w.es as u32) << 16) | (BiosRegs.w.bp as u32);

    BiosRegs.b.bh = ROM_8x16Font;
    Int386(0x10, &mut BiosRegs, &mut BiosRegs);
    *RomFontPointers.offset(4) = ((BiosRegs.w.es as u32) << 16) | (BiosRegs.w.bp as u32);

    BiosRegs.b.bh = ROM_Alternate9x16Font;
    Int386(0x10, &mut BiosRegs, &mut BiosRegs);
    *RomFontPointers.offset(5) = ((BiosRegs.w.es as u32) << 16) | (BiosRegs.w.bp as u32);
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoSetTextCursorPosition(X: UCHAR, Y: UCHAR) {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.b.ah = 0x02;
    Regs.b.bh = 0x00;
    Regs.b.dh = Y;
    Regs.b.dl = X;
    Int386(0x10, &mut Regs, &mut Regs);
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoHideShowTextCursor(Show: BOOLEAN) {
    if Show != 0 {
        PcVideoDefineCursor(0x0D, 0x0E);
    } else {
        PcVideoDefineCursor(0x20, 0x00);
    }
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoCopyOffScreenBufferToVRAM(Buffer: PVOID) {
    let BanksToCopy: USHORT;
    let BytesInLastBank: ULONG;
    let mut CurrentBank: USHORT;
    let BankSize: ULONG;

    if VideoTextMode == DisplayMode {
        RtlCopyMemory(
            VIDEOTEXT_MEM_ADDRESS as PVOID,
            Buffer,
            PcVideoGetBufferSize() as usize,
        );
    } else if VideoGraphicsMode == DisplayMode && VesaVideoMode != 0 {
        BankSize = (VesaVideoModeInformation.WindowGranularity as ULONG) << 10;
        BanksToCopy = ((VesaVideoModeInformation.HeightInPixels as ULONG
            * VesaVideoModeInformation.BytesPerScanLine as ULONG)
            / BankSize) as USHORT;
        BytesInLastBank = (VesaVideoModeInformation.HeightInPixels as ULONG
            * VesaVideoModeInformation.BytesPerScanLine as ULONG)
            % BankSize;

        CurrentBank = 0;
        while CurrentBank < BanksToCopy {
            PcVideoSetMemoryBank(CurrentBank);
            RtlCopyMemory(
                VIDEOVGA_MEM_ADDRESS as PVOID,
                (Buffer as *mut u8).add((CurrentBank as ULONG * BankSize) as usize) as PVOID,
                BankSize as usize,
            );
            CurrentBank = CurrentBank.wrapping_add(1);
        }

        PcVideoSetMemoryBank(CurrentBank);
        RtlCopyMemory(
            VIDEOVGA_MEM_ADDRESS as PVOID,
            (Buffer as *mut u8).add((CurrentBank as ULONG * BankSize) as usize) as PVOID,
            BytesInLastBank as usize,
        );
    } else {
        unimplemented!();
    }
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoClearScreen(Attr: UCHAR) {
    let AttrChar: USHORT = ((Attr as USHORT) << 8) | b' ' as USHORT;
    let mut BufPtr = VIDEOTEXT_MEM_ADDRESS as *mut USHORT;
    while BufPtr < (VIDEOTEXT_MEM_ADDRESS + VIDEOTEXT_MEM_SIZE) as *mut USHORT {
        *BufPtr = AttrChar;
        BufPtr = BufPtr.offset(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoPutChar(Ch: i32, Attr: u8, X: u32, Y: u32) {
    let BufPtr: *mut u16;

    BufPtr = (VIDEOTEXT_MEM_ADDRESS + Y * BytesPerScanLine + X * 2) as *mut u16;
    *BufPtr = ((Attr as u16) << 8) | ((Ch & 0xff) as u16);
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoIsPaletteFixed() -> u8 {
    0u8
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoSetPaletteColor(Color: UCHAR, Red: UCHAR, Green: UCHAR, Blue: UCHAR) {
    
    WRITE_PORT_UCHAR(VIDEOPORT_PALETTE_WRITE as *mut UCHAR, Color);
    WRITE_PORT_UCHAR(VIDEOPORT_PALETTE_DATA as *mut UCHAR, Red);
    WRITE_PORT_UCHAR(VIDEOPORT_PALETTE_DATA as *mut UCHAR, Green);
    WRITE_PORT_UCHAR(VIDEOPORT_PALETTE_DATA as *mut UCHAR, Blue);
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoGetPaletteColor(Color: UCHAR, Red: *mut UCHAR, Green: *mut UCHAR, Blue: *mut UCHAR) {
    
    WRITE_PORT_UCHAR(VIDEOPORT_PALETTE_READ as *mut UCHAR, Color);
    *Red = READ_PORT_UCHAR(VIDEOPORT_PALETTE_DATA as *mut UCHAR);
    *Green = READ_PORT_UCHAR(VIDEOPORT_PALETTE_DATA as *mut UCHAR);
    *Blue = READ_PORT_UCHAR(VIDEOPORT_PALETTE_DATA as *mut UCHAR);
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoSync() {
    while (READ_PORT_UCHAR(VIDEOPORT_VERTICAL_RETRACE as *mut u8) & 0x08) != 0 {
    }

    while (READ_PORT_UCHAR(VIDEOPORT_VERTICAL_RETRACE as *mut u8) & 0x08) == 0 {
    }
}

#[no_mangle]
pub unsafe extern "C" fn PcVideoPrepareForReactOS() {
    PcVideoSetMode80x25();
    PcVideoHideShowTextCursor(0u8);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #include <freeldr.h>
// #include <suppress.h>
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(UI);
//
// #define VIDEOPORT_PALETTE_READ      0x03C7
// #define VIDEOPORT_PALETTE_WRITE     0x03C8
// #define VIDEOPORT_PALETTE_DATA      0x03C9
// #define VIDEOPORT_VERTICAL_RETRACE  0x03DA
//
// #define VIDEOVGA_MEM_ADDRESS  0xA0000
// #define VIDEOTEXT_MEM_ADDRESS 0xB8000
// #define VIDEOTEXT_MEM_SIZE    0x8000
//
// #define VIDEOCARD_CGA_OR_OTHER 0
// #define VIDEOCARD_EGA          1
// #define VIDEOCARD_VGA          2
//
// #define VIDEOMODE_NORMAL_TEXT   0
// #define VIDEOMODE_EXTENDED_TEXT 1
// #define    VIDEOMODE_80X28         0x501C
// #define    VIDEOMODE_80X30         0x501E
// #define    VIDEOMODE_80X34         0x5022
// #define    VIDEOMODE_80X43         0x502B
// #define    VIDEOMODE_80X60         0x503C
// #define    VIDEOMODE_132X25        0x8419
// #define    VIDEOMODE_132X43        0x842B
// #define    VIDEOMODE_132X50        0x8432
// #define    VIDEOMODE_132X60        0x843C
//
// #define VERTRES_200_SCANLINES        0x00
// #define VERTRES_350_SCANLINES        0x01
// #define VERTRES_400_SCANLINES        0x02
//
// #include <pshpack2.h>
// typedef struct
// {
//     USHORT ModeAttributes;             /* Mode attributes (see #00080) */
//     UCHAR  WindowAttributesA;          /* Window attributes, window A (see #00081) */
//     UCHAR  WindowsAttributesB;         /* Window attributes, window B (see #00081) */
//     USHORT WindowGranularity;          /* Window granularity in KB */
//     USHORT WindowSize;                 /* Window size in KB */
//     USHORT WindowAStartSegment;        /* Start segment of window A (0000h if not supported) */
//     USHORT WindowBStartSegment;        /* Start segment of window B (0000h if not supported) */
//     ULONG  WindowPositioningFunction;  /* -> FAR window positioning function (equivalent to AX=4F05h) */
//     USHORT BytesPerScanLine;           /* Bytes per scan line */
//     /* ---remainder is optional for VESA modes in v1.0/1.1, needed for OEM modes--- */
//     USHORT WidthInPixels;              /* Width in pixels (graphics) or characters (text) */
//     USHORT HeightInPixels;             /* Height in pixels (graphics) or characters (text) */
//     UCHAR  CharacterWidthInPixels;     /* Width of character cell in pixels */
//     UCHAR  CharacterHeightInPixels;    /* Height of character cell in pixels */
//     UCHAR  NumberOfMemoryPlanes;       /* Number of memory planes */
//     UCHAR  BitsPerPixel;               /* Number of bits per pixel */
//     UCHAR  NumberOfBanks;              /* Number of banks */
//     UCHAR  MemoryModel;                /* Memory model type (see #00082) */
//     UCHAR  BankSize;                   /* Size of bank in KB */
//     UCHAR  NumberOfImagePanes;         /* Number of image pages (less one) that will fit in video RAM */
//     UCHAR  Reserved1;                  /* Reserved (00h for VBE 1.0-2.0, 01h for VBE 3.0) */
//     /* ---VBE v1.2+ --- */
//     UCHAR  RedMaskSize;                /* Red mask size */
//     UCHAR  RedMaskPosition;            /* Red field position */
//     UCHAR  GreenMaskSize;              /* Green mask size */
//     UCHAR  GreenMaskPosition;          /* Green field size */
//     UCHAR  BlueMaskSize;               /* Blue mask size */
//     UCHAR  BlueMaskPosition;           /* Blue field size */
//     UCHAR  ReservedMaskSize;           /* Reserved mask size */
//     UCHAR  ReservedMaskPosition;       /* Reserved mask position */
//     UCHAR  DirectColorModeInfo;        /* Direct color mode info
//                                         * bit 0:Color ramp is programmable
//                                         * bit 1:Bytes in reserved field may be used by application */
//     /* ---VBE v2.0+ --- */
//     ULONG  LinearVideoBufferAddress;   /* Physical address of linear video buffer */
//     ULONG  OffscreenMemoryPointer;     /* Pointer to start of offscreen memory */
//     USHORT OffscreenMemorySize;        /* KB of offscreen memory */
//     /* ---VBE v3.0 --- */
//     USHORT LinearBytesPerScanLine;     /* Bytes per scan line in linear modes */
//     UCHAR  BankedNumberOfImages;       /* Number of images (less one) for banked video modes */
//     UCHAR  LinearNumberOfImages;       /* Number of images (less one) for linear video modes */
//     UCHAR  LinearRedMaskSize;          /* Linear modes:Size of direct color red mask (in bits) */
//     UCHAR  LinearRedMaskPosition;      /* Linear modes:Bit position of red mask LSB (e.g. shift count) */
//     UCHAR  LinearGreenMaskSize;        /* Linear modes:Size of direct color green mask (in bits) */
//     UCHAR  LinearGreenMaskPosition;    /* Linear modes:Bit position of green mask LSB (e.g. shift count) */
//     UCHAR  LinearBlueMaskSize;         /* Linear modes:Size of direct color blue mask (in bits) */
//     UCHAR  LinearBlueMaskPosition;     /* Linear modes:Bit position of blue mask LSB (e.g. shift count) */
//     UCHAR  LinearReservedMaskSize;     /* Linear modes:Size of direct color reserved mask (in bits) */
//     UCHAR  LinearReservedMaskPosition; /* Linear modes:Bit position of reserved mask LSB */
//     ULONG  MaximumPixelClock;          /* Maximum pixel clock for graphics video mode, in Hz */
//     UCHAR  Reserved2[190];             /* 190 BYTEs  reserved (0) */
// } SVGA_MODE_INFORMATION, *PSVGA_MODE_INFORMATION;
// C_ASSERT(sizeof(SVGA_MODE_INFORMATION) == 256);
// #include <poppack.h>
//
// UCHAR MachDefaultTextColor = COLOR_GRAY;
//
// static ULONG VideoCard = VIDEOCARD_CGA_OR_OTHER;        /* Type of video card installed on the system */
// static USHORT BiosVideoMode = VIDEOMODE_NORMAL_TEXT;    /* Current video mode as known by BIOS */
// static ULONG ScreenWidth = 80;                          /* Screen Width in characters */
// static ULONG ScreenHeight = 25;                         /* Screen Height in characters */
// static ULONG BytesPerScanLine = 160;                    /* Number of bytes per scanline (delta) */
// static VIDEODISPLAYMODE DisplayMode = VideoTextMode;    /* Current display mode */
// static BOOLEAN VesaVideoMode = FALSE;                   /* Are we using a VESA mode? */
// static SVGA_MODE_INFORMATION VesaVideoModeInformation;  /* Only valid when in VESA mode */
// static ULONG CurrentMemoryBank = 0;                     /* Currently selected VESA bank */
//
// enum
// {
//     INT1FhFont = 0x00,
//     INT43hFont = 0x01,
//     ROM_8x14CharacterFont = 0x02,
//     ROM_8x8DoubleDotFontLo = 0x03,
//     ROM_8x8DoubleDotFontHi = 0x04,
//     ROM_AlphaAlternate = 0x05,
//     ROM_8x16Font = 0x06,
//     ROM_Alternate9x16Font = 0x07,
//     UltraVision_8x20Font = 0x11,
//     UltraVision_8x10Font = 0x12,
// };
//
// static ULONG
// PcVideoDetectVideoCard(VOID)
// {
//     REGS Regs;
//
//     /* Int 10h AH=12h BL=10h
//      * VIDEO - ALTERNATE FUNCTION SELECT (PS,EGA,VGA,MCGA) - GET EGA INFO
//      *
//      * AH = 12h
//      * BL = 10h
//      * Return:
//      * BH = video state
//      * 00h color mode in effect (I/O port 3Dxh)
//      * 01h mono mode in effect (I/O port 3Bxh)
//      * BL = installed memory (00h = 64K, 01h = 128K, 02h = 192K, 03h = 256K)
//      * CH = feature connector bits
//      * CL = switch settings
//      * AH destroyed (at least by Tseng ET4000 BIOS v8.00n)
//      *
//      * Installation check;EGA
//      */
//     Regs.b.ah = 0x12;
//     Regs.b.bl = 0x10;
//     Int386(0x10, &Regs, &Regs);
//
//     /* If BL is still equal to 0x10 then there is no EGA/VGA present */
//     if (Regs.b.bl == 0x10)
//         return VIDEOCARD_CGA_OR_OTHER;
//
//     /* Int 10h AX=1A00h
//      * VIDEO - GET DISPLAY COMBINATION CODE (PS,VGA/MCGA)
//      *
//      * AX = 1A00h
//      * Return:
//      * AL = 1Ah if function was supported
//      * BL = active display code
//      * BH = alternate display code
//      *
//      * This function is commonly used to check for the presence of a VGA.
//      *
//      * Installation check;VGA
//      *
//      * Values for display combination code:
//      * 00h    no display
//      * 01h    monochrome adapter w/ monochrome display
//      * 02h    CGA w/ color display
//      * 03h    reserved
//      * 04h    EGA w/ color display
//      * 05h    EGA w/ monochrome display
//      * 06h    PGA w/ color display
//      * 07h    VGA w/ monochrome analog display
//      * 08h    VGA w/ color analog display
//      * 09h    reserved
//      * 0Ah    MCGA w/ digital color display
//      * 0Bh    MCGA w/ monochrome analog display
//      * 0Ch    MCGA w/ color analog display
//      * FFh    unknown display type
//      */
//     Regs.w.ax = 0x1A00;
//     Int386(0x10, &Regs, &Regs);
//
//     if (Regs.b.al == 0x1a)
//         return VIDEOCARD_VGA;
//     else
//         return VIDEOCARD_EGA;
// }
//
// static BOOLEAN
// PcVideoVesaGetSVGAModeInformation(
//     _In_ USHORT Mode,
//     _Out_ PSVGA_MODE_INFORMATION ModeInformation)
// {
//     REGS Regs;
//
//     RtlZeroMemory((PVOID)BIOSCALLBUFFER, sizeof(*ModeInformation));
//
//     /* VESA SuperVGA BIOS - GET SuperVGA MODE INFORMATION
//      * AX = 4F01h
//      * CX = SuperVGA video mode (see #04082 for bitfields)
//      * ES:DI -> 256-byte buffer for mode information (see #00079)
//      * Return:
//      * AL = 4Fh if function supported
//      * AH = status
//      * 00h successful
//      * ES:DI buffer filled
//      * 01h failed
//      *
//      * Desc: Determine the attributes of the specified video mode.
//      *
//      * Note: While VBE 1.1 and higher will zero out all unused bytes
//      * of the buffer, v1.0 did not, so applications that want to be
//      * backward compatible should clear the buffer before calling.
//      */
//     Regs.w.ax = 0x4F01;
//     Regs.w.cx = Mode;
//     Regs.w.es = BIOSCALLBUFSEGMENT;
//     Regs.w.di = BIOSCALLBUFOFFSET;
//     Int386(0x10, &Regs, &Regs);
//
//     if (Regs.w.ax != 0x004F)
//         return FALSE;
//
//     RtlCopyMemory(ModeInformation, (PVOID)BIOSCALLBUFFER, sizeof(*ModeInformation));
//
//     TRACE("\n");
//     TRACE("BiosVesaGetSVGAModeInformation(Mode 0x%x)\n", (Mode) as usize);
//     TRACE("ModeAttributes = 0x%04x\n", (ModeInformation->ModeAttributes) as usize);
//     TRACE("WindowAttributesA = 0x%02x\n", (ModeInformation->WindowAttributesA) as usize);
//     TRACE("WindowAttributesB = 0x%02x\n", (ModeInformation->WindowsAttributesB) as usize);
//     TRACE("WindowGranularity = %huKB\n", (ModeInformation->WindowGranularity) as usize);
//     TRACE("WindowSize = %huKB\n", (ModeInformation->WindowSize) as usize);
//     TRACE("WindowAStartSegment = 0x%04x\n", (ModeInformation->WindowAStartSegment) as usize);
//     TRACE("WindowBStartSegment = 0x%04x\n", (ModeInformation->WindowBStartSegment) as usize);
//     TRACE("WindowPositioningFunction = 0x%x\n", (ModeInformation->WindowPositioningFunction) as usize);
//     TRACE("BytesPerScanLine = %hu\n", (ModeInformation->BytesPerScanLine) as usize);
//     TRACE("WidthInPixels = %hu\n", (ModeInformation->WidthInPixels) as usize);
//     TRACE("HeightInPixels = %hu\n", (ModeInformation->HeightInPixels) as usize);
//     TRACE("CharacterWidthInPixels = %u\n", (ModeInformation->CharacterWidthInPixels) as usize);
//     TRACE("CharacterHeightInPixels = %u\n", (ModeInformation->CharacterHeightInPixels) as usize);
//     TRACE("NumberOfMemoryPlanes = %u\n", (ModeInformation->NumberOfMemoryPlanes) as usize);
//     TRACE("BitsPerPixel = %u\n", (ModeInformation->BitsPerPixel) as usize);
//     TRACE("NumberOfBanks = %u\n", (ModeInformation->NumberOfBanks) as usize);
//     TRACE("MemoryModel = %u\n", (ModeInformation->MemoryModel) as usize);
//     TRACE("BankSize = %u\n", (ModeInformation->BankSize) as usize);
//     TRACE("NumberOfImagePanes = %u\n", (ModeInformation->NumberOfImagePanes) as usize);
//     TRACE("Reserved1 = 0x%02x\n", (ModeInformation->Reserved1) as usize);
//     TRACE("---VBE v1.2+ ---\n");
//     TRACE("RedMaskSize = %u\n", (ModeInformation->RedMaskSize) as usize);
//     TRACE("RedMaskPosition = %u\n", (ModeInformation->RedMaskPosition) as usize);
//     TRACE("GreenMaskSize = %u\n", (ModeInformation->GreenMaskSize) as usize);
//     TRACE("GreenMaskPosition = %u\n", (ModeInformation->GreenMaskPosition) as usize);
//     TRACE("BlueMaskSize = %u\n", (ModeInformation->BlueMaskSize) as usize);
//     TRACE("BlueMaskPosition = %u\n", (ModeInformation->BlueMaskPosition) as usize);
//     TRACE("ReservedMaskSize = %u\n", (ModeInformation->ReservedMaskSize) as usize);
//     TRACE("ReservedMaskPosition = %u\n", (ModeInformation->ReservedMaskPosition) as usize);
//     TRACE("DirectColorModeInfo = 0x%02x\n", (ModeInformation->DirectColorModeInfo) as usize);
//     TRACE("---VBE v2.0+ ---\n");
//     TRACE("LinearVideoBufferAddress = 0x%x\n", (ModeInformation->LinearVideoBufferAddress) as usize);
//     TRACE("OffscreenMemoryPointer = 0x%x\n", (ModeInformation->OffscreenMemoryPointer) as usize);
//     TRACE("OffscreenMemorySize = %huKB\n", (ModeInformation->OffscreenMemorySize) as usize);
//     TRACE("---VBE v3.0 ---\n");
//     TRACE("LinearBytesPerScanLine = %hu\n", (ModeInformation->LinearBytesPerScanLine) as usize);
//     TRACE("BankedNumberOfImages = %u\n", (ModeInformation->BankedNumberOfImages) as usize);
//     TRACE("LinearNumberOfImages = %u\n", (ModeInformation->LinearNumberOfImages) as usize);
//     TRACE("LinearRedMaskSize = %u\n", (ModeInformation->LinearRedMaskSize) as usize);
//     TRACE("LinearRedMaskPosition = %u\n", (ModeInformation->LinearRedMaskPosition) as usize);
//     TRACE("LinearGreenMaskSize = %u\n", (ModeInformation->LinearGreenMaskSize) as usize);
//     TRACE("LinearGreenMaskPosition = %u\n", (ModeInformation->LinearGreenMaskPosition) as usize);
//     TRACE("LinearBlueMaskSize = %u\n", (ModeInformation->LinearBlueMaskSize) as usize);
//     TRACE("LinearBlueMaskPosition = %u\n", (ModeInformation->LinearBlueMaskPosition) as usize);
//     TRACE("LinearReservedMaskSize = %u\n", (ModeInformation->LinearReservedMaskSize) as usize);
//     TRACE("LinearReservedMaskPosition = %u\n", (ModeInformation->LinearReservedMaskPosition) as usize);
//     TRACE("MaximumPixelClock = %luHz\n", (ModeInformation->MaximumPixelClock) as usize);
//     TRACE("\n");
//
//     return TRUE;
// }
//
// static BOOLEAN
// PcVideoVesaGetCurrentSVGAMode(
//     _Out_ PUSHORT Mode)
// {
//     REGS Regs;
//
//     /*
//      * VESA SuperVGA BIOS - GET CURRENT VIDEO MODE
//      *
//      * AX = 4F03h
//      * Return:
//      * AL = 4Fh if function supported
//      * AH = status
//      * 00h successful
//      * BX = video mode (see #00083,#00084)
//      *
//      * bit 13:
//      * VBE/AF v1.0P accelerated video mode
//      *
//      * bit 14:
//      * Linear frame buffer enabled (VBE v2.0+)
//      *
//      * bit 15:
//      * Don't clear video memory
//      * 01h failed
//      */
//     Regs.w.ax = 0x4F03;
//     Int386(0x10, &Regs, &Regs);
//
//     if (Regs.w.ax != 0x004F)
//         return FALSE;
//
//     *Mode = Regs.w.bx;
//     return TRUE;
// }
//
// static BOOLEAN
// PcVideoGetBiosMode(
//     _Out_ PUSHORT Mode)
// {
//     REGS Regs;
//
//     /* Int 10h AH=0Fh
//      * VIDEO - GET CURRENT VIDEO MODE
//      *
//      * AH = 0Fh
//      * Return:
//      * AH = number of character columns
//      * AL = display mode (see #00010 at AH=00h)
//      * BH = active page (see AH=05h)
//      *
//      * Notes: If mode was set with bit 7 set ("no blanking"), the returned
//      * mode will also have bit 7 set. EGA, VGA, and UltraVision return either
//      * AL=03h (color) or AL=07h (monochrome) in all extended-row text modes.
//      * HP 200LX returns AL=07h (monochrome) if mode was set to AL=21h and
//      * always 80 resp. 40 columns in all text modes regardless of current
//      * zoom setting (see AH=D0h). When using a Hercules Graphics Card,
//      * additional checks are necessary:
//      *
//      * mode 05h:
//      * If WORD 0040h:0063h is 03B4h, may be in graphics page 1
//      * (as set by DOSSHELL and other Microsoft software)
//      *
//      * mode 06h:
//      * If WORD 0040h:0063h is 03B4h, may be in graphics page 0
//      * (as set by DOSSHELL and other Microsoft software)
//      *
//      * mode 07h:
//      * If BYTE 0040h:0065h bit 1 is set, Hercules card is in
//      * graphics mode, with bit 7 indicating the page (mode set by
//      * Hercules driver for Borland Turbo C).
//      * The Tandy 2000 BIOS is only documented as returning AL, not AH or BH
//      */
//     Regs.b.ah = 0x0F;
//     Int386(0x10, &Regs, &Regs);
//
//     if (Regs.b.ah == 0)
//         return FALSE;
//
//     *Mode = Regs.b.al;
//     return TRUE;
// }
//
// static VOID
// PcVideoGetDisplayMode(VOID)
// {
//     USHORT Mode = 0;
//
//     /* Is the current mode VESA? */
//     if (PcVideoVesaGetCurrentSVGAMode(&Mode) &&
//         PcVideoVesaGetSVGAModeInformation(Mode, &VesaVideoModeInformation))
//     {
//         TRACE("VESA mode detected\n");
//         ScreenWidth = VesaVideoModeInformation.WidthInPixels;
//         ScreenHeight = VesaVideoModeInformation.HeightInPixels;
//         BytesPerScanLine = VesaVideoModeInformation.BytesPerScanLine;
//         BiosVideoMode = Mode;
//         DisplayMode = (0x0108 <= Mode && Mode <= 0x010C) ? VideoTextMode : VideoGraphicsMode;
//         VesaVideoMode = TRUE;
//     }
//     /* If not, it should be a regular BIOS mode */
//     else if (PcVideoGetBiosMode(&Mode))
//     {
//         /*
//          * The BIOS data area 0x400 holds information about the current video mode.
//          * Infos at: https://web.archive.org/web/20240119203029/http://www.bioscentral.com/misc/bda.htm
//          * https://stanislavs.org/helppc/bios_data_area.html
//          */
//
//         UCHAR BiosMode = (*(PUCHAR)0x449) & 0x7F; /* Current video mode */
//         ASSERT(BiosMode == Mode);
//         BiosVideoMode = Mode;
//         TRACE("BIOS mode detected\n");
//
//         ScreenWidth = *(PUSHORT)0x44A; /* Number of screen columns */
//         ScreenHeight = 1 + *(PUCHAR)0x484; /* 1 + "Rows on the screen (less 1, EGA+)" */
//
//         /*
//          * Select bits 7 and 4 of "Video display data area (MCGA and VGA)"
//          * |7|6|5|4|3|2|1|0|
//          *  |     `------ see table below
//          *  `--------- alphanumeric scan lines (see table below)
//          *
//          * then convert to a number and map it to the scanline number:
//          * Bit7  Bit4  Scan Lines
//          * 0     0     350 line mode
//          * 0     1     400 line mode
//          * 1     0     200 line mode
//          * 1     1     reserved
//          */
//         BytesPerScanLine = (*(PUCHAR)0x489) & 0x90;
//         BytesPerScanLine = ((BytesPerScanLine & 0x80) >> 6) | ((BytesPerScanLine & 0x10) >> 4);
//         switch (BytesPerScanLine)
//         {
//             case VERTRES_200_SCANLINES:
//                 BytesPerScanLine = 200; break;
//             case VERTRES_350_SCANLINES:
//                 BytesPerScanLine = 350; break;
//             case VERTRES_400_SCANLINES:
//                 BytesPerScanLine = 400; break;
//             default:
//                 BytesPerScanLine = 160; break;
//         }
//
//         DisplayMode = VideoTextMode;
//         VesaVideoMode = FALSE;
//     }
//     else
//     {
//         /* Set the values for the default text mode.
//          * If a graphics mode is set, these values will be changed. */
//         TRACE("Fallback mode detected\n");
//         BiosVideoMode = VIDEOMODE_NORMAL_TEXT;
//         ScreenWidth = 80;
//         ScreenHeight = 25;
//         BytesPerScanLine = 160;
//         DisplayMode = VideoTextMode;
//         VesaVideoMode = FALSE;
//     }
//
//     TRACE("This is a %s video mode\n", (VesaVideoMode ? "VESA" : "BIOS") as usize);
//     TRACE("BiosVideoMode    = 0x%x\n", (BiosVideoMode) as usize);
//     TRACE("DisplayMode      = %sMode\n", (DisplayMode == VideoTextMode ? "Text" : "Graphics") as usize);
//     TRACE("ScreenWidth      = %lu\n", (ScreenWidth) as usize);
//     TRACE("ScreenHeight     = %lu\n", (ScreenHeight) as usize);
//     TRACE("BytesPerScanLine = %lu\n", (BytesPerScanLine) as usize);
// }
//
// static BOOLEAN
// PcVideoSetBiosVesaMode(USHORT Mode)
// {
//     REGS Regs;
//
//     /* Int 10h AX=4F02h
//      * VESA SuperVGA BIOS - SET SuperVGA VIDEO MODE
//      *
//      * AX = 4F02h
//      * BX = new video mode
//      * ES:DI -> (VBE 3.0+) CRTC information block, bit mode bit 11 set
//      * Return:
//      * AL = 4Fh if function supported
//      * AH = status
//      * 00h successful
//      * 01h failed
//      *
//      * Values for VESA video mode:
//      * 00h-FFh OEM video modes (see #00010 at AH=00h)
//      * 100h   640x400x256
//      * 101h   640x480x256
//      * 102h   800x600x16
//      * 103h   800x600x256
//      * 104h   1024x768x16
//      * 105h   1024x768x256
//      * 106h   1280x1024x16
//      * 107h   1280x1024x256
//      * 108h   80x60 text
//      * 109h   132x25 text
//      * 10Ah   132x43 text
//      * 10Bh   132x50 text
//      * 10Ch   132x60 text
//      * ---VBE v1.2+ ---
//      * 10Dh   320x200x32K
//      * 10Eh   320x200x64K
//      * 10Fh   320x200x16M
//      * 110h   640x480x32K
//      * 111h   640x480x64K
//      * 112h   640x480x16M
//      * 113h   800x600x32K
//      * 114h   800x600x64K
//      * 115h   800x600x16M
//      * 116h   1024x768x32K
//      * 117h   1024x768x64K
//      * 118h   1024x768x16M
//      * 119h   1280x1024x32K (1:5:5:5)
//      * 11Ah   1280x1024x64K (5:6:5)
//      * 11Bh   1280x1024x16M
//      * ---VBE 2.0+ ---
//      * 120h   1600x1200x256
//      * 121h   1600x1200x32K
//      * 122h   1600x1200x64K
//      * 81FFh   special full-memory access mode
//      *
//      * Notes: The special mode 81FFh preserves the contents of the video memory and gives
//      * access to all of the memory; VESA recommends that the special mode be a packed-pixel
//      * mode. For VBE 2.0+, it is required that the VBE implement the mode, but not place it
//      * in the list of available modes (mode information for this mode can be queried
//      * directly, however). As of VBE 2.0, VESA will no longer define video mode numbers.
//      */
//     Regs.w.ax = 0x4F02;
//     Regs.w.bx = Mode;
//     Int386(0x10, &Regs, &Regs);
//     return (Regs.w.ax == 0x004F);
// }
//
// static VOID
// PcVideoSetBiosMode(UCHAR VideoMode)
// {
//     REGS Regs;
//
//     /* Int 10h AH=00h
//      * VIDEO - SET VIDEO MODE
//      *
//      * AH = 00h
//      * AL = desired video mode
//      * Return:
//      * AL = video mode flag (Phoenix, AMI BIOS)
//      * 20h mode > 7
//      * 30h modes 0-5 and 7
//      * 3Fh mode 6
//      * AL = CRT controller mode byte (Phoenix 386 BIOS v1.10)
//      */
//     Regs.b.ah = 0x00;
//     Regs.b.al = VideoMode;
//     Int386(0x10, &Regs, &Regs);
// }
//
// static VOID
// PcVideoSetFont8x8(VOID)
// {
//     REGS Regs;
//
//     /* Int 10h AX=1112h
//      * VIDEO - TEXT-MODE CHARGEN - LOAD ROM 8x8 DBL-DOT PATTERNS (PS,EGA,VGA)
//      *
//      * AX = 1112h
//      * BL = block to load
//      * Return:
//      * Nothing
//      */
//     Regs.w.ax = 0x1112;
//     Regs.b.bl = 0x00;
//     Int386(0x10, &Regs, &Regs);
// }
//
// static VOID
// PcVideoSetFont8x14(VOID)
// {
//     REGS Regs;
//
//     /* Int 10h AX=1111h
//      * VIDEO - TEXT-MODE CHARGEN - LOAD ROM MONOCHROME PATTERNS (PS,EGA,VGA)
//      *
//      * AX = 1111h
//      * BL = block to load
//      * Return:
//      * Nothing
//      */
//     Regs.w.ax = 0x1111;
//     Regs.b.bl = 0;
//     Int386(0x10, &Regs, &Regs);
// }
//
// static VOID
// PcVideoSelectAlternatePrintScreen(VOID)
// {
//     REGS Regs;
//
//     /* Int 10h AH=12h BL=20h
//      * VIDEO - ALTERNATE FUNCTION SELECT (PS,EGA,VGA,MCGA) - ALTERNATE PRTSC
//      *
//      * AH = 12h
//      * BL = 20h select alternate print screen routine
//      * Return:
//      * Nothing
//      *
//      * Installs a PrtSc routine from the video card's BIOS to replace the
//      * default PrtSc handler from the ROM BIOS, which usually does not
//      * understand screen heights other than 25 lines.
//      *
//      * Some adapters disable print-screen instead of enhancing it.
//      */
//     Regs.b.ah = 0x12;
//     Regs.b.bl = 0x20;
//     Int386(0x10, &Regs, &Regs);
// }
//
// static VOID
// PcVideoDisableCursorEmulation(VOID)
// {
//     REGS Regs;
//
//     /* Int 10h AH=12h BL=34h
//      * VIDEO - ALTERNATE FUNCTION SELECT (VGA) - CURSOR EMULATION
//      *
//      * AH = 12h
//      * BL = 34h
//      * AL = new state
//      * 00h enable alphanumeric cursor emulation
//      * 01h disable alphanumeric cursor emulation
//      * Return:
//      * AL = 12h if function supported
//      *
//      * Specify whether the BIOS should automatically remap cursor start/end
//      * according to the current character height in text modes.
//      */
//     Regs.b.ah = 0x12;
//     Regs.b.bl = 0x34;
//     Regs.b.al = 0x01;
//     Int386(0x10, &Regs, &Regs);
// }
//
// static VOID
// PcVideoDefineCursor(UCHAR StartScanLine, UCHAR EndScanLine)
// {
//     REGS Regs;
//
//     /* Int 10h AH=01h
//      * VIDEO - SET TEXT-MODE CURSOR SHAPE
//      *
//      * AH = 01h
//      * CH = cursor start and options
//      * CL = bottom scan line containing cursor (bits 0-4)
//      * Return:
//      * Nothing
//      *
//      * Specify the starting and ending scan lines to be occupied
//      * by the hardware cursor in text modes.
//      *
//      * AMI 386 BIOS and AST Premier 386 BIOS will lock up the
//      * system if AL is not equal to the current video mode.
//      *
//      * Bitfields for cursor start and options:
//      *
//      * Bit(s)    Description
//      * 7         should be zero
//      * 6,5       cursor blink
//      * (00=normal, 01=invisible, 10=erratic, 11=slow).
//      * (00=normal, other=invisible on EGA/VGA)
//      * 4-0       topmost scan line containing cursor
//      */
//     Regs.b.ah = 0x01;
//     Regs.b.al = 0x03;
//     Regs.b.ch = StartScanLine;
//     Regs.b.cl = EndScanLine;
//     Int386(0x10, &Regs, &Regs);
// }
//
// static VOID
// PcVideoSetVerticalResolution(UCHAR VerticalResolutionMode)
// {
//     REGS Regs;
//
//     /* Int 10h AH=12h BL=30h
//      * VIDEO - ALTERNATE FUNCTION SELECT (VGA) - SELECT VERTICAL RESOLUTION
//      *
//      * AH = 12h
//      * BL = 30h
//      * AL = vertical resolution
//      * 00h 200 scan lines
//      * 01h 350 scan lines
//      * 02h 400 scan lines
//      * Return:
//      * AL = 12h if function supported
//      *
//      * Specify the number of scan lines used to display text modes.
//      *
//      * The specified resolution will take effect on the next mode set.
//      */
//     Regs.b.ah = 0x12;
//     Regs.b.bl = 0x30;
//     Regs.b.al = VerticalResolutionMode;
//     Int386(0x10, &Regs, &Regs);
// }
//
// static VOID
// PcVideoSet480ScanLines(VOID)
// {
//     INT_PTR CRTC;
//
//     /* Read CRTC port */
//     CRTC = READ_PORT_UCHAR((PUCHAR)0x03CC);
//     if (CRTC & 1)
//         CRTC = 0x3D4;
//     else
//         CRTC = 0x3B4;
//
//     /* Vertical sync end (also unlocks CR0-7) */
//     WRITE_PORT_UCHAR((PUCHAR)CRTC, 0x11);
//     WRITE_PORT_UCHAR((PUCHAR)CRTC+1, 0x0C);
//
//     /* Vertical total */
//     WRITE_PORT_UCHAR((PUCHAR)CRTC, 0x06);
//     WRITE_PORT_UCHAR((PUCHAR)CRTC+1, 0x0B);
//
//     /* (vertical) overflow */
//     WRITE_PORT_UCHAR((PUCHAR)CRTC, 0x07);
//     WRITE_PORT_UCHAR((PUCHAR)CRTC+1, 0x3E);
//
//     /* Vertical sync start */
//     WRITE_PORT_UCHAR((PUCHAR)CRTC, 0x10);
//     WRITE_PORT_UCHAR((PUCHAR)CRTC+1, 0xEA);
//
//     /* Vertical display end */
//     WRITE_PORT_UCHAR((PUCHAR)CRTC, 0x12);
//     WRITE_PORT_UCHAR((PUCHAR)CRTC+1, 0xDF);
//
//     /* Vertical blank start */
//     WRITE_PORT_UCHAR((PUCHAR)CRTC, 0x15);
//     WRITE_PORT_UCHAR((PUCHAR)CRTC+1, 0xE7);
//
//     /* Vertical blank end */
//     WRITE_PORT_UCHAR((PUCHAR)CRTC, 0x16);
//     WRITE_PORT_UCHAR((PUCHAR)CRTC+1, 0x04);
//
//     /* Misc output register (read) */
//     CRTC = READ_PORT_UCHAR((PUCHAR)0x03CC);
//
//     /* Preserve clock select bits and color bit */
//     CRTC = (CRTC & 0x0D);
//     /* Set correct sync polarity */
//     CRTC = (CRTC | 0xE2);
//
//     /* (write) */
//     WRITE_PORT_UCHAR((PUCHAR)0x03C2, (UCHAR)CRTC);
// }
//
// static VOID
// PcVideoSetDisplayEnd(VOID)
// {
//     INT_PTR CRTC;
//
//     /* Read CRTC port */
//     CRTC = READ_PORT_UCHAR((PUCHAR)0x03CC);
//     if (CRTC & 1)
//         CRTC = 0x3D4;
//     else
//         CRTC = 0x3B4;
//
//     /* Vertical display end */
//     WRITE_PORT_UCHAR((PUCHAR)CRTC, 0x12);
//     WRITE_PORT_UCHAR((PUCHAR)CRTC+1, 0xDF);
// }
//
// static BOOLEAN
// PcVideoSetMode80x25(VOID)
// {
//     PcVideoSetBiosMode(0x03);
//     ScreenWidth = 80;
//     ScreenHeight = 25;
//     return TRUE;
// }
//
// static BOOLEAN
// PcVideoSetMode80x50_80x43(VOID)
// {
//     if (VideoCard == VIDEOCARD_VGA)
//     {
//         PcVideoSetBiosMode(0x03);
//         PcVideoSetFont8x8();
//         PcVideoSelectAlternatePrintScreen();
//         PcVideoDisableCursorEmulation();
//         PcVideoDefineCursor(6, 7);
//         ScreenWidth = 80;
//         ScreenHeight = 50;
//     }
//     else if (VideoCard == VIDEOCARD_EGA)
//     {
//         PcVideoSetBiosMode(0x03);
//         PcVideoSetFont8x8();
//         PcVideoSelectAlternatePrintScreen();
//         PcVideoDisableCursorEmulation();
//         PcVideoDefineCursor(6, 7);
//         ScreenWidth = 80;
//         ScreenHeight = 43;
//     }
//     else /* VIDEOCARD_CGA_OR_OTHER */
//     {
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// static BOOLEAN
// PcVideoSetMode80x28(VOID)
// {
//     /* FIXME: Is this VGA-only? */
//     PcVideoSetMode80x25();
//     PcVideoSetFont8x14();
//     PcVideoDefineCursor(11, 12);
//     ScreenWidth = 80;
//     ScreenHeight = 28;
//     return TRUE;
// }
//
// static BOOLEAN
// PcVideoSetMode80x30(VOID)
// {
//     /* FIXME: Is this VGA-only? */
//     PcVideoSetMode80x25();
//     PcVideoSet480ScanLines();
//     ScreenWidth = 80;
//     ScreenHeight = 30;
//     return TRUE;
// }
//
// static BOOLEAN
// PcVideoSetMode80x34(VOID)
// {
//     /* FIXME: Is this VGA-only? */
//     PcVideoSetMode80x25();
//     PcVideoSet480ScanLines();
//     PcVideoSetFont8x14();
//     PcVideoDefineCursor(11, 12);
//     PcVideoSetDisplayEnd();
//     ScreenWidth = 80;
//     ScreenHeight = 34;
//     return TRUE;
// }
//
// static BOOLEAN
// PcVideoSetMode80x43(VOID)
// {
//     /* FIXME: Is this VGA-only? */
//     PcVideoSetVerticalResolution(VERTRES_350_SCANLINES);
//     PcVideoSetMode80x25();
//     PcVideoSetFont8x8();
//     PcVideoSelectAlternatePrintScreen();
//     PcVideoDisableCursorEmulation();
//     PcVideoDefineCursor(6, 7);
//     ScreenWidth = 80;
//     ScreenHeight = 43;
//     return TRUE;
// }
//
// static BOOLEAN
// PcVideoSetMode80x60(VOID)
// {
//     /* FIXME: Is this VGA-only? */
//     PcVideoSetMode80x25();
//     PcVideoSet480ScanLines();
//     PcVideoSetFont8x8();
//     PcVideoSelectAlternatePrintScreen();
//     PcVideoDisableCursorEmulation();
//     PcVideoDefineCursor(6, 7);
//     PcVideoSetDisplayEnd();
//     ScreenWidth = 80;
//     ScreenHeight = 60;
//     return TRUE;
// }
//
// static BOOLEAN
// PcVideoSetMode(USHORT NewMode)
// {
//     CurrentMemoryBank = 0;
//
//     /* Set the values for the default text modes
//      * If they are setting a graphics mode then
//      * these values will be changed. */
//     BiosVideoMode = NewMode;
//     ScreenWidth = 80;
//     ScreenHeight = 25;
//     BytesPerScanLine = 160;
//     DisplayMode = VideoTextMode;
//     VesaVideoMode = FALSE;
//
//     switch (NewMode)
//     {
//         case VIDEOMODE_NORMAL_TEXT:
//         case 0x03: /* BIOS 80x25 text mode number */
//             return PcVideoSetMode80x25();
//         case VIDEOMODE_EXTENDED_TEXT:
//             return PcVideoSetMode80x50_80x43();
//         case VIDEOMODE_80X28:
//             return PcVideoSetMode80x28();
//         case VIDEOMODE_80X30:
//             return PcVideoSetMode80x30();
//         case VIDEOMODE_80X34:
//             return PcVideoSetMode80x34();
//         case VIDEOMODE_80X43:
//             return PcVideoSetMode80x43();
//         case VIDEOMODE_80X60:
//             return PcVideoSetMode80x60();
//     }
//
//     if (NewMode == 0x12)
//     {
//         /* 640x480x16 */
//         PcVideoSetBiosMode((UCHAR)NewMode);
//         WRITE_PORT_USHORT((USHORT*)0x03CE, 0x0F01); /* For some reason this is necessary? */
//         ScreenWidth = 640;
//         ScreenHeight = 480;
//         BytesPerScanLine = 80;
//         BiosVideoMode = NewMode;
//         DisplayMode = VideoGraphicsMode;
//
//         return TRUE;
//     }
//     else if (NewMode == 0x13)
//     {
//         /* 320x200x256 */
//         PcVideoSetBiosMode((UCHAR)NewMode);
//         ScreenWidth = 320;
//         ScreenHeight = 200;
//         BytesPerScanLine = 320;
//         BiosVideoMode = NewMode;
//         DisplayMode = VideoGraphicsMode;
//
//         return TRUE;
//     }
//     else
//     {
//         /* VESA Text/Graphics Mode */
//         if (!PcVideoVesaGetSVGAModeInformation(NewMode, &VesaVideoModeInformation))
//             return FALSE;
//
//         if (!PcVideoSetBiosVesaMode(NewMode))
//             return FALSE;
//
//         ScreenWidth = VesaVideoModeInformation.WidthInPixels;
//         ScreenHeight = VesaVideoModeInformation.HeightInPixels;
//         BytesPerScanLine = VesaVideoModeInformation.BytesPerScanLine;
//         BiosVideoMode = NewMode;
//         DisplayMode = (0x0108 <= NewMode && NewMode <= 0x010C) ? VideoTextMode : VideoGraphicsMode;
//         VesaVideoMode = TRUE;
//
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// static VOID
// PcVideoSetBlinkBit(BOOLEAN Enable)
// {
//     REGS Regs;
//
//     /* Int 10h AX=1003h
//      * VIDEO - TOGGLE INTENSITY/BLINKING BIT (Jr, PS, TANDY 1000, EGA, VGA)
//      *
//      * AX = 1003h
//      * BL = new state
//      * 00h background intensity enabled
//      * 01h blink enabled
//      * BH = 00h to avoid problems on some adapters
//      * Return:
//      * Nothing
//      *
//      * Note: although there is no function to get
//      * the current status, bit 5 of 0040h:0065h
//      * indicates the state.
//      */
//     Regs.w.ax = 0x1003;
//     Regs.w.bx = Enable ? 0x0001 : 0x0000;
//     Int386(0x10, &Regs, &Regs);
// }
//
// static VOID
// PcVideoSetMemoryBank(USHORT BankNumber)
// {
//     REGS Regs;
//
//     if (CurrentMemoryBank == BankNumber)
//         return;
//
//     /* Int 10h AX=4F05h
//      * VESA SuperVGA BIOS - CPU VIDEO MEMORY CONTROL
//      *
//      * AX = 4F05h
//      * BH = subfunction
//      * 00h select video memory window
//      * 01h get video memory window
//      * DX = window address in video memory (in granularity units)
//      * Return:
//      * DX = window address in video memory (in gran. units)
//      * BL = window number
//      * 00h window A
//      * 01h window B.
//      * Return:
//      * AL = 4Fh if function supported
//      * AH = status
//      * 00h successful
//      * 01h failed
//      */
//     Regs.w.ax = 0x4F05;
//     Regs.w.bx = 0x0000;
//     Regs.w.dx = BankNumber;
//     Int386(0x10, &Regs, &Regs);
//
//     if (Regs.w.ax == 0x004F)
//         CurrentMemoryBank = BankNumber;
// }
//
// VOID
// PcVideoInit(VOID)
// {
//     /* Detect the installed video card */
//     VideoCard = PcVideoDetectVideoCard();
//
//     /* Retrieve the initial display mode parameters */
//     PcVideoGetDisplayMode();
//
//     /* If any video options have been specified, try to set a display mode */
//     if (BootMgrInfo.VideoOptions && *BootMgrInfo.VideoOptions)
//         PcVideoSetDisplayMode(BootMgrInfo.VideoOptions, TRUE);
//
//     // FIXME: We don't support graphics modes yet!
//     // Revert to 80x25 text mode.
//     if (DisplayMode != VideoTextMode)
//         PcVideoSetMode(VIDEOMODE_NORMAL_TEXT);
// }
//
// #define TRACE_printf(Format, ...) \
// do { \
//     TRACE(Format, (##__VA_ARGS__) as usize); \
//     printf(Format, ##__VA_ARGS__); \
// } while (0)
//
// VIDEODISPLAYMODE
// PcVideoSetDisplayMode(PCSTR DisplayModeName, BOOLEAN Init)
// {
//     USHORT VideoMode = VIDEOMODE_NORMAL_TEXT;
//
//     if (!DisplayModeName || *DisplayModeName == '\0')
//         goto Quit;
//
//     if (VideoCard == VIDEOCARD_CGA_OR_OTHER)
//     {
//         TRACE_printf("CGA or other display adapter detected.\n");
//         TRACE_printf("Using 80x25 text mode.\n");
//         VideoMode = VIDEOMODE_NORMAL_TEXT;
//     }
//     else if (VideoCard == VIDEOCARD_EGA)
//     {
//         TRACE_printf("EGA display adapter detected.\n");
//         TRACE_printf("Using 80x25 text mode.\n");
//         VideoMode = VIDEOMODE_NORMAL_TEXT;
//     }
//     else /* VIDEOCARD_VGA */
//     {
//         TRACE_printf("VGA display adapter detected.\n");
//
//         /* Get the video option separator, if any */
//         size_t NameLen = strcspn(DisplayModeName, ",");
//         if (!NameLen)
//             goto Quit;
//
//         if (_strnicmp(DisplayModeName, "NORMAL_VGA", NameLen) == 0)
//             VideoMode = VIDEOMODE_NORMAL_TEXT;
//         else if (_strnicmp(DisplayModeName, "EXTENDED_VGA", NameLen) == 0)
//             VideoMode = VIDEOMODE_EXTENDED_TEXT;
//         else
//             VideoMode = (USHORT)strtoul(DisplayModeName, NULL, 0);
//     }
//
//     if (!PcVideoSetMode(VideoMode))
//     {
//         TRACE_printf("Error: unable to set video display mode 0x%x\n", VideoMode);
//         TRACE_printf("Defaulting to 80x25 text mode.\n");
//         printf("Press any key to continue.\n");
//         PcConsGetCh();
//
//         PcVideoSetMode(VIDEOMODE_NORMAL_TEXT);
//     }
//
// Quit:
//     PcVideoSetBlinkBit(!Init);
//     return DisplayMode;
// }
//
// VOID
// PcVideoGetDisplaySize(PULONG Width, PULONG Height, PULONG Depth)
// {
//     *Width = ScreenWidth;
//     *Height = ScreenHeight;
//     if (VideoGraphicsMode == DisplayMode && VesaVideoMode)
//     {
//         if (VesaVideoModeInformation.BitsPerPixel == 16)
//         {
//             /* 16-bit color modes give green an extra bit (5:6:5)
//              * 15-bit color modes have just 5:5:5 for R:G:B */
//             *Depth = (VesaVideoModeInformation.GreenMaskSize == 6 ? 16 : 15);
//         }
//         else
//         {
//             *Depth = VesaVideoModeInformation.BitsPerPixel;
//         }
//     }
//     else
//     {
//         *Depth = 0;
//     }
// }
//
// ULONG
// PcVideoGetBufferSize(VOID)
// {
//     return ScreenHeight * BytesPerScanLine;
// }
//
// VOID
// PcVideoGetFontsFromFirmware(PULONG RomFontPointers)
// {
//     REGS BiosRegs;
//
//     /* Get the address of the BIOS ROM fonts.
//      * Int 10h, AX=1130h, BH = pointer specifier
//      * Returns: es:bp = address */
//     BiosRegs.d.eax = 0x1130;
//     BiosRegs.b.bh = ROM_8x14CharacterFont;
//     Int386(0x10, &BiosRegs, &BiosRegs);
//     RomFontPointers[0] = BiosRegs.w.es << 4 | BiosRegs.w.bp;
//
//     BiosRegs.b.bh = ROM_8x8DoubleDotFontLo;
//     Int386(0x10, &BiosRegs, &BiosRegs);
//     RomFontPointers[1] = BiosRegs.w.es << 16 | BiosRegs.w.bp;
//
//     BiosRegs.b.bh = ROM_8x8DoubleDotFontHi;
//     Int386(0x10, &BiosRegs, &BiosRegs);
//     RomFontPointers[2] = BiosRegs.w.es << 16 | BiosRegs.w.bp;
//
//     BiosRegs.b.bh = ROM_AlphaAlternate;
//     Int386(0x10, &BiosRegs, &BiosRegs);
//     RomFontPointers[3] = BiosRegs.w.es << 16 | BiosRegs.w.bp;
//
//     BiosRegs.b.bh = ROM_8x16Font;
//     Int386(0x10, &BiosRegs, &BiosRegs);
//     RomFontPointers[4] = BiosRegs.w.es << 16 | BiosRegs.w.bp;
//
//     BiosRegs.b.bh = ROM_Alternate9x16Font;
//     Int386(0x10, &BiosRegs, &BiosRegs);
//     RomFontPointers[5] = BiosRegs.w.es << 16 | BiosRegs.w.bp;
// }
//
// VOID
// PcVideoSetTextCursorPosition(UCHAR X, UCHAR Y)
// {
//     REGS Regs;
//
//     /* Int 10h AH=02h
//      * VIDEO - SET CURSOR POSITION
//      *
//      * AH = 02h
//      * BH = page number
//      * 0-3 in modes 2&3
//      * 0-7 in modes 0&1
//      * 0 in graphics modes
//      * DH = row (00h is top)
//      * DL = column (00h is left)
//      * Return:
//      * Nothing
//      */
//     Regs.b.ah = 0x02;
//     Regs.b.bh = 0x00;
//     Regs.b.dh = Y;
//     Regs.b.dl = X;
//     Int386(0x10, &Regs, &Regs);
// }
//
// VOID
// PcVideoHideShowTextCursor(BOOLEAN Show)
// {
//     if (Show)
//         PcVideoDefineCursor(0x0D, 0x0E);
//     else
//         PcVideoDefineCursor(0x20, 0x00);
// }
//
// VOID
// PcVideoCopyOffScreenBufferToVRAM(PVOID Buffer)
// {
//     USHORT BanksToCopy;
//     ULONG BytesInLastBank;
//     USHORT CurrentBank;
//     ULONG BankSize;
//
//     /* PcVideoWaitForVerticalRetrace(); */
//
//     /* Text mode (BIOS or VESA) */
//     if (VideoTextMode == DisplayMode)
//     {
//         RtlCopyMemory((PVOID)VIDEOTEXT_MEM_ADDRESS, Buffer, PcVideoGetBufferSize());
//     }
//     /* VESA graphics mode */
//     else if (VideoGraphicsMode == DisplayMode && VesaVideoMode)
//     {
//         BankSize = VesaVideoModeInformation.WindowGranularity << 10;
//         BanksToCopy = (USHORT)((VesaVideoModeInformation.HeightInPixels * VesaVideoModeInformation.BytesPerScanLine) / BankSize);
//         BytesInLastBank = (VesaVideoModeInformation.HeightInPixels * VesaVideoModeInformation.BytesPerScanLine) % BankSize;
//
//         /* Copy all the banks but the last one because
//          * it is probably a partial bank */
//         for (CurrentBank = 0; CurrentBank < BanksToCopy; CurrentBank++)
//         {
//             PcVideoSetMemoryBank(CurrentBank);
//             RtlCopyMemory((PVOID)VIDEOVGA_MEM_ADDRESS, (char*)Buffer + CurrentBank * BankSize, BankSize);
//         }
//
//         /* Copy the remaining bytes into the last bank */
//         PcVideoSetMemoryBank(CurrentBank);
//         RtlCopyMemory((PVOID)VIDEOVGA_MEM_ADDRESS, (char*)Buffer + CurrentBank * BankSize, BytesInLastBank);
//     }
//     /* BIOS graphics mode */
//     else
//     {
//         UNIMPLEMENTED;
//     }
// }
//
// VOID
// PcVideoClearScreen(UCHAR Attr)
// {
//     USHORT AttrChar;
//     USHORT *BufPtr;
//
//     AttrChar = ((USHORT)Attr << 8) | ' ';
//     for (BufPtr = (USHORT*)VIDEOTEXT_MEM_ADDRESS;
//          BufPtr < (USHORT*)(VIDEOTEXT_MEM_ADDRESS + VIDEOTEXT_MEM_SIZE);
//          BufPtr++)
//     {
//         _PRAGMA_WARNING_SUPPRESS(__WARNING_DEREF_NULL_PTR)
//         *BufPtr = AttrChar;
//     }
// }
//
// VOID
// PcVideoPutChar(int Ch, UCHAR Attr, unsigned X, unsigned Y)
// {
//     USHORT *BufPtr;
//
//     BufPtr = (USHORT*)(ULONG_PTR)(VIDEOTEXT_MEM_ADDRESS + Y * BytesPerScanLine + X * 2);
//     *BufPtr = ((USHORT)Attr << 8) | (Ch & 0xff);
// }
//
// BOOLEAN
// PcVideoIsPaletteFixed(VOID)
// {
//     return FALSE;
// }
//
// VOID
// PcVideoSetPaletteColor(UCHAR Color, UCHAR Red, UCHAR Green, UCHAR Blue)
// {
//     WRITE_PORT_UCHAR((UCHAR*)VIDEOPORT_PALETTE_WRITE, Color);
//     WRITE_PORT_UCHAR((UCHAR*)VIDEOPORT_PALETTE_DATA, Red);
//     WRITE_PORT_UCHAR((UCHAR*)VIDEOPORT_PALETTE_DATA, Green);
//     WRITE_PORT_UCHAR((UCHAR*)VIDEOPORT_PALETTE_DATA, Blue);
// }
//
// VOID
// PcVideoGetPaletteColor(UCHAR Color, UCHAR* Red, UCHAR* Green, UCHAR* Blue)
// {
//     WRITE_PORT_UCHAR((UCHAR*)VIDEOPORT_PALETTE_READ, Color);
//     *Red = READ_PORT_UCHAR((UCHAR*)VIDEOPORT_PALETTE_DATA);
//     *Green = READ_PORT_UCHAR((UCHAR*)VIDEOPORT_PALETTE_DATA);
//     *Blue = READ_PORT_UCHAR((UCHAR*)VIDEOPORT_PALETTE_DATA);
// }
//
// VOID
// PcVideoSync(VOID)
// {
//     while ((READ_PORT_UCHAR((UCHAR*)VIDEOPORT_VERTICAL_RETRACE) & 0x08))
//     {
//         /*
//          * Keep reading the port until bit 3 is clear
//          * This waits for the current retrace to end and
//          * we can catch the next one so we know we are
//          * getting a full retrace.
//          */
//     }
//
//     while (!(READ_PORT_UCHAR((UCHAR*)VIDEOPORT_VERTICAL_RETRACE) & 0x08))
//     {
//         /*
//          * Keep reading the port until bit 3 is set
//          * Now that we know we aren't doing a vertical
//          * retrace we need to wait for the next one.
//          */
//     }
// }
//
// VOID
// PcVideoPrepareForReactOS(VOID)
// {
//     // PcVideoSetMode80x50_80x43();
//     PcVideoSetMode80x25();
//     PcVideoHideShowTextCursor(FALSE);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
