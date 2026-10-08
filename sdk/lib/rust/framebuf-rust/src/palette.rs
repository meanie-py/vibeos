use win32k_sys::win32k::*;
use crate::framebuf::*;
use core::ptr;

const BASEPALETTE: [PALETTEENTRY; 20] = [
    PALETTEENTRY { peRed: 0x00, peGreen: 0x00, peBlue: 0x00, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0x80, peGreen: 0x00, peBlue: 0x00, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0x00, peGreen: 0x80, peBlue: 0x00, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0x80, peGreen: 0x80, peBlue: 0x00, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0x00, peGreen: 0x00, peBlue: 0x80, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0x80, peGreen: 0x00, peBlue: 0x80, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0x00, peGreen: 0x80, peBlue: 0x80, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0xC0, peGreen: 0xC0, peBlue: 0xC0, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0xC0, peGreen: 0xDC, peBlue: 0xC0, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0xD4, peGreen: 0xD0, peBlue: 0xC8, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0xFF, peGreen: 0xFB, peBlue: 0xF0, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0x3A, peGreen: 0x6E, peBlue: 0xA5, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0x80, peGreen: 0x80, peBlue: 0x80, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0xFF, peGreen: 0x00, peBlue: 0x00, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0x00, peGreen: 0xFF, peBlue: 0x00, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0xFF, peGreen: 0xFF, peBlue: 0x00, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0x00, peGreen: 0x00, peBlue: 0xFF, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0xFF, peGreen: 0x00, peBlue: 0xFF, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0x00, peGreen: 0xFF, peBlue: 0xFF, peFlags: 0x00 },
    PALETTEENTRY { peRed: 0xFF, peGreen: 0xFF, peBlue: 0xFF, peFlags: 0x00 },
];

#[no_mangle]
pub unsafe extern "C" fn IntInitDefaultPalette(
    ppdev: *mut PDEV,
    pDevInfo: *mut DEVINFO,
) -> BOOL {
    if (*ppdev).BitsPerPixel > 8 {
        (*ppdev).DefaultPalette = EngCreatePalette(
            PAL_BITFIELDS,
            0,
            ptr::null_mut(),
            (*ppdev).RedMask,
            (*ppdev).GreenMask,
            (*ppdev).BlueMask,
        );
        (*pDevInfo).hpalDefault = (*ppdev).DefaultPalette;
    } else {
        (*ppdev).PaletteEntries = EngAllocMem(
            0,
            (core::mem::size_of::<PALETTEENTRY>() << 8) as u32,
            ALLOC_TAG,
        ) as PPALETTEENTRY;
        if (*ppdev).PaletteEntries.is_null() {
            return 0;
        }

        let mut ColorLoop = 256;
        let mut PaletteEntryPtr = (*ppdev).PaletteEntries;
        while ColorLoop != 0 {
            (*PaletteEntryPtr).peRed = (((ColorLoop >> 5) & 7) * 255 / 7) as u8;
            (*PaletteEntryPtr).peGreen = (((ColorLoop >> 3) & 3) * 255 / 3) as u8;
            (*PaletteEntryPtr).peBlue = ((ColorLoop & 7) * 255 / 7) as u8;
            (*PaletteEntryPtr).peFlags = 0;
            ColorLoop -= 1;
            PaletteEntryPtr = PaletteEntryPtr.add(1);
        }

        core::ptr::copy_nonoverlapping(
            BASEPALETTE.as_ptr() as *const u8,
            (*ppdev).PaletteEntries as *mut u8,
            10 * core::mem::size_of::<PALETTEENTRY>(),
        );
        core::ptr::copy_nonoverlapping(
            BASEPALETTE.as_ptr().add(10) as *const u8,
            (*ppdev).PaletteEntries.add(246) as *mut u8,
            10 * core::mem::size_of::<PALETTEENTRY>(),
        );

        (*ppdev).DefaultPalette = EngCreatePalette(
            PAL_INDEXED,
            256,
            (*ppdev).PaletteEntries as *mut u32,
            0,
            0,
            0,
        );
        (*pDevInfo).hpalDefault = (*ppdev).DefaultPalette;
    }

    if (*ppdev).DefaultPalette.is_null() {
        0
    } else {
        1
    }
}

#[no_mangle]
pub unsafe extern "C" fn IntSetPalette(
    dhpdev: DHPDEV,
    ppalent: PPALETTEENTRY,
    iStart: u32,
    cColors: u32,
) -> BOOL {
    let mut cColors = cColors;
    let ClutSize = core::mem::size_of::<VIDEO_CLUT>() + (cColors as usize * core::mem::size_of::<u32>());
    let pClut = EngAllocMem(0, ClutSize as u32, ALLOC_TAG) as PVIDEO_CLUT;

    (*pClut).FirstEntry = iStart as u16;
    (*pClut).NumEntries = cColors as u16;
    core::ptr::copy_nonoverlapping(
        ppalent as *const u8,
        core::ptr::addr_of_mut!((*pClut).LookupTable) as *mut u8,
        core::mem::size_of::<u32>() * cColors as usize ,
    );

    // LookupTable is a C flexible array (`LookupTable[1]`, psdk/ntddvdeo.h) in a buffer
    // EngAllocMem'd for cColors entries above. bindgen's [_; 1] panicked on the FIRST
    // iteration of both walks below -- they run downward from cColors - 1 -- so every
    // palette of more than one colour took the display driver down. Constant index 0 is
    // checked at compile time.
    let lut = &raw mut (*pClut).LookupTable[0];
    if (*(dhpdev as *mut PDEV)).PaletteShift != 0 {
        let mut idx = cColors;
        while idx > 0 {
            idx -= 1;
            (*lut.add(idx as usize)).RgbArray.Red >>= (*(dhpdev as *mut PDEV)).PaletteShift;
            (*lut.add(idx as usize)).RgbArray.Green >>= (*(dhpdev as *mut PDEV)).PaletteShift;
            (*lut.add(idx as usize)).RgbArray.Blue >>= (*(dhpdev as *mut PDEV)).PaletteShift;
            (*lut.add(idx as usize)).RgbArray.Unused = 0;
        }
    } else {
        let mut idx = cColors;
        while idx > 0 {
            idx -= 1;
            (*lut.add(idx as usize)).RgbArray.Unused = 0;
        }
    }

    if EngDeviceIoControl(
        (*(dhpdev as *mut PDEV)).hDriver,
        IOCTL_VIDEO_SET_COLOR_REGISTERS,
        pClut as *mut core::ffi::c_void,
        ClutSize as u32,
        ptr::null_mut(),
        0,
        &mut cColors as *mut u32,
    ) != 0
    {
        EngFreeMem(pClut as *mut core::ffi::c_void);
        return 0;
    }

    EngFreeMem(pClut as *mut core::ffi::c_void);
    1
}

#[no_mangle]
pub unsafe extern "C" fn DrvSetPalette(
    dhpdev: DHPDEV,
    ppalo: *mut PALOBJ,
    _fl: FLONG,
    iStart: u32,
    cColors: u32,
) -> BOOL {
    if cColors == 0 {
        return 0;
    }

    let PaletteEntries = EngAllocMem(
        0,
        cColors * core::mem::size_of::<u32>() as u32 ,
        ALLOC_TAG,
    ) as PPALETTEENTRY;
    if PaletteEntries.is_null() {
        return 0;
    }

    if PALOBJ_cGetColors(ppalo, iStart, cColors, PaletteEntries as *mut u32) != cColors {
        EngFreeMem(PaletteEntries as *mut core::ffi::c_void);
        return 0;
    }

    let bRet = IntSetPalette(dhpdev, PaletteEntries, iStart, cColors);
    EngFreeMem(PaletteEntries as *mut core::ffi::c_void);
    bRet
}