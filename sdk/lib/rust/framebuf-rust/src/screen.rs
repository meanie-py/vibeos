use win32k_sys::win32k::*;
use crate::framebuf::*;

static mut SystemFont: LOGFONTW = LOGFONTW {
    lfHeight: 16,
    lfWidth: 7,
    lfEscapement: 0,
    lfOrientation: 0,
    lfWeight: 700,
    lfItalic: 0,
    lfUnderline: 0,
    lfStrikeOut: 0,
    lfCharSet: ANSI_CHARSET as u8,
    lfOutPrecision: OUT_DEFAULT_PRECIS as u8,
    lfClipPrecision: CLIP_DEFAULT_PRECIS as u8,
    lfQuality: DEFAULT_QUALITY as u8,
    lfPitchAndFamily: (VARIABLE_PITCH | FF_DONTCARE) as u8,
    lfFaceName: [
        b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
};

static mut AnsiVariableFont: LOGFONTW = LOGFONTW {
    lfHeight: 12,
    lfWidth: 9,
    lfEscapement: 0,
    lfOrientation: 0,
    lfWeight: 400,
    lfItalic: 0,
    lfUnderline: 0,
    lfStrikeOut: 0,
    lfCharSet: ANSI_CHARSET as u8,
    lfOutPrecision: OUT_DEFAULT_PRECIS as u8,
    lfClipPrecision: CLIP_STROKE_PRECIS as u8,
    lfQuality: PROOF_QUALITY as u8,
    lfPitchAndFamily: (VARIABLE_PITCH | FF_DONTCARE) as u8,
    lfFaceName: [
        b'M' as u16, b'S' as u16, b' ' as u16, b'S' as u16, b'a' as u16, b'n' as u16,
        b's' as u16, b' ' as u16, b'S' as u16, b'e' as u16, b'r' as u16, b'i' as u16,
        b'f' as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
};

static mut AnsiFixedFont: LOGFONTW = LOGFONTW {
    lfHeight: 12,
    lfWidth: 9,
    lfEscapement: 0,
    lfOrientation: 0,
    lfWeight: 400,
    lfItalic: 0,
    lfUnderline: 0,
    lfStrikeOut: 0,
    lfCharSet: ANSI_CHARSET as u8,
    lfOutPrecision: OUT_DEFAULT_PRECIS as u8,
    lfClipPrecision: CLIP_STROKE_PRECIS as u8,
    lfQuality: PROOF_QUALITY as u8,
    lfPitchAndFamily: (FIXED_PITCH | FF_DONTCARE) as u8,
    lfFaceName: [
        b'C' as u16, b'o' as u16, b'u' as u16, b'r' as u16, b'i' as u16, b'e' as u16,
        b'r' as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0,
    ],
};

unsafe fn GetAvailableModes(
    hDriver: HANDLE,
    ModeInfo: *mut PVIDEO_MODE_INFORMATION,
    ModeInfoSize: *mut DWORD,
) -> DWORD {
    let mut ulTemp: ULONG = 0;
    let mut Modes: VIDEO_NUM_MODES = core::mem::zeroed();
    let mut ModeInfoPtr: PVIDEO_MODE_INFORMATION;

    if EngDeviceIoControl(
        hDriver,
        IOCTL_VIDEO_QUERY_NUM_AVAIL_MODES,
        core::ptr::null_mut(),
        0,
        &mut Modes as *mut VIDEO_NUM_MODES as *mut core::ffi::c_void,
        core::mem::size_of::<VIDEO_NUM_MODES>() as ULONG,
        &mut ulTemp,
    ) != 0
    {
        return 0;
    }

    if Modes.NumModes == 0 {
        return 0;
    }

    *ModeInfoSize = Modes.ModeInformationLength;

    *ModeInfo = EngAllocMem(
        0,
        (Modes.NumModes * Modes.ModeInformationLength) as ULONG,
        ALLOC_TAG,
    ) as PVIDEO_MODE_INFORMATION;

    if (*ModeInfo).is_null() {
        return 0;
    }

    if EngDeviceIoControl(
        hDriver,
        IOCTL_VIDEO_QUERY_AVAIL_MODES,
        core::ptr::null_mut(),
        0,
        *ModeInfo as *mut core::ffi::c_void,
        Modes.NumModes * Modes.ModeInformationLength,
        &mut ulTemp,
    ) != 0
    {
        EngFreeMem(*ModeInfo as *mut core::ffi::c_void);
        *ModeInfo = core::ptr::null_mut();
        return 0;
    }

    ulTemp = Modes.NumModes;
    ModeInfoPtr = *ModeInfo;

    while ulTemp > 0 {
        ulTemp -= 1;
        if ((*ModeInfoPtr).NumberOfPlanes != 1)
            || (((*ModeInfoPtr).AttributeFlags & VIDEO_MODE_GRAPHICS) == 0)
            || (((*ModeInfoPtr).BitsPerPlane != 8)
                && ((*ModeInfoPtr).BitsPerPlane != 16)
                && ((*ModeInfoPtr).BitsPerPlane != 24)
                && ((*ModeInfoPtr).BitsPerPlane != 32))
        {
            (*ModeInfoPtr).Length = 0;
        }

        ModeInfoPtr = ((ModeInfoPtr as *mut u8).add(Modes.ModeInformationLength as usize))
            as PVIDEO_MODE_INFORMATION;
    }

    Modes.NumModes
}

#[no_mangle]
pub unsafe extern "C" fn IntInitScreenInfo(
    ppdev: PPDEV,
    pDevMode: LPDEVMODEW,
    pGdiInfo: PGDIINFO,
    pDevInfo: PDEVINFO,
) -> BOOL {
    let mut ModeCount: ULONG;
    let mut ModeInfoSize: ULONG = 0;
    let mut ModeInfo: PVIDEO_MODE_INFORMATION = core::ptr::null_mut();
    let mut ModeInfoPtr: PVIDEO_MODE_INFORMATION;
    let mut SelectedMode: PVIDEO_MODE_INFORMATION = core::ptr::null_mut();
    let mut ColorCapabilities: VIDEO_COLOR_CAPABILITIES = core::mem::zeroed();
    let mut Temp: ULONG = 0;

    ModeCount = GetAvailableModes((*ppdev).hDriver, &mut ModeInfo, &mut ModeInfoSize);
    if ModeCount == 0 {
        return 0;
    }

    if (*pDevMode).dmPelsWidth == 0
        && (*pDevMode).dmPelsHeight == 0
        && (*pDevMode).dmBitsPerPel == 0
        && (*pDevMode).dmDisplayFrequency == 0
    {
        ModeInfoPtr = ModeInfo;
        while ModeCount > 0 {
            ModeCount -= 1;
            if (*ModeInfoPtr).Length == 0 {
                ModeInfoPtr = ((ModeInfoPtr as *mut u8).add(ModeInfoSize as usize))
                    as PVIDEO_MODE_INFORMATION;
                continue;
            }
            SelectedMode = ModeInfoPtr;
            break;
        }
    } else {
        ModeInfoPtr = ModeInfo;
        while ModeCount > 0 {
            ModeCount -= 1;
            if (*ModeInfoPtr).Length > 0
                && (*pDevMode).dmPelsWidth == (*ModeInfoPtr).VisScreenWidth
                && (*pDevMode).dmPelsHeight == (*ModeInfoPtr).VisScreenHeight
                && (*pDevMode).dmBitsPerPel
                    == ((*ModeInfoPtr).BitsPerPlane * (*ModeInfoPtr).NumberOfPlanes)
                && (*pDevMode).dmDisplayFrequency == (*ModeInfoPtr).Frequency
            {
                SelectedMode = ModeInfoPtr;
                break;
            }

            ModeInfoPtr = ((ModeInfoPtr as *mut u8).add(ModeInfoSize as usize))
                as PVIDEO_MODE_INFORMATION;
        }
    }

    if SelectedMode.is_null() {
        EngFreeMem(ModeInfo as *mut core::ffi::c_void);
        return 0;
    }

    (*ppdev).ModeIndex = (*SelectedMode).ModeIndex;
    (*ppdev).ScreenWidth = (*SelectedMode).VisScreenWidth;
    (*ppdev).ScreenHeight = (*SelectedMode).VisScreenHeight;
    (*ppdev).ScreenDelta = (*SelectedMode).ScreenStride;
    (*ppdev).BitsPerPixel =
        ((*SelectedMode).BitsPerPlane * (*SelectedMode).NumberOfPlanes) as UCHAR;

    (*ppdev).MemWidth = (*SelectedMode).VideoMemoryBitmapWidth;
    (*ppdev).MemHeight = (*SelectedMode).VideoMemoryBitmapHeight;

    (*ppdev).RedMask = (*SelectedMode).RedMask;
    (*ppdev).GreenMask = (*SelectedMode).GreenMask;
    (*ppdev).BlueMask = (*SelectedMode).BlueMask;

    (*pGdiInfo).ulVersion = GDI_DRIVER_VERSION;
    (*pGdiInfo).ulTechnology = DT_RASDISPLAY;
    (*pGdiInfo).ulHorzSize = (*SelectedMode).XMillimeter;
    (*pGdiInfo).ulVertSize = (*SelectedMode).YMillimeter;
    (*pGdiInfo).ulHorzRes = (*SelectedMode).VisScreenWidth;
    (*pGdiInfo).ulVertRes = (*SelectedMode).VisScreenHeight;
    (*pGdiInfo).ulPanningHorzRes = (*SelectedMode).VisScreenWidth;
    (*pGdiInfo).ulPanningVertRes = (*SelectedMode).VisScreenHeight;
    (*pGdiInfo).cBitsPixel = (*SelectedMode).BitsPerPlane;
    (*pGdiInfo).cPlanes = (*SelectedMode).NumberOfPlanes;
    (*pGdiInfo).ulVRefresh = (*SelectedMode).Frequency;
    (*pGdiInfo).ulBltAlignment = 1;
    (*pGdiInfo).ulLogPixelsX = (*pDevMode).dmLogPixels as ULONG;
    (*pGdiInfo).ulLogPixelsY = (*pDevMode).dmLogPixels as ULONG;
    (*pGdiInfo).flTextCaps = TC_RA_ABLE;
    (*pGdiInfo).flRaster = 0;
    (*pGdiInfo).ulDACRed = (*SelectedMode).NumberRedBits;
    (*pGdiInfo).ulDACGreen = (*SelectedMode).NumberGreenBits;
    (*pGdiInfo).ulDACBlue = (*SelectedMode).NumberBlueBits;
    (*pGdiInfo).ulAspectX = 0x24;
    (*pGdiInfo).ulAspectY = 0x24;
    (*pGdiInfo).ulAspectXY = 0x33;
    (*pGdiInfo).xStyleStep = 1;
    (*pGdiInfo).yStyleStep = 1;
    (*pGdiInfo).denStyleStep = 3;
    (*pGdiInfo).ptlPhysOffset.x = 0;
    (*pGdiInfo).ptlPhysOffset.y = 0;
    (*pGdiInfo).szlPhysSize.cx = 0;
    (*pGdiInfo).szlPhysSize.cy = 0;

    if EngDeviceIoControl(
        (*ppdev).hDriver,
        IOCTL_VIDEO_QUERY_COLOR_CAPABILITIES,
        core::ptr::null_mut(),
        0,
        &mut ColorCapabilities as *mut VIDEO_COLOR_CAPABILITIES as *mut core::ffi::c_void,
        core::mem::size_of::<VIDEO_COLOR_CAPABILITIES>() as ULONG,
        &mut Temp,
    ) == 0
    {
        (*pGdiInfo).ciDevice.Red.x = ColorCapabilities.RedChromaticity_x;
        (*pGdiInfo).ciDevice.Red.y = ColorCapabilities.RedChromaticity_y;
        (*pGdiInfo).ciDevice.Green.x = ColorCapabilities.GreenChromaticity_x;
        (*pGdiInfo).ciDevice.Green.y = ColorCapabilities.GreenChromaticity_y;
        (*pGdiInfo).ciDevice.Blue.x = ColorCapabilities.BlueChromaticity_x;
        (*pGdiInfo).ciDevice.Blue.y = ColorCapabilities.BlueChromaticity_y;
        (*pGdiInfo).ciDevice.AlignmentWhite.x = ColorCapabilities.WhiteChromaticity_x;
        (*pGdiInfo).ciDevice.AlignmentWhite.y = ColorCapabilities.WhiteChromaticity_y;
        (*pGdiInfo).ciDevice.AlignmentWhite.Y = ColorCapabilities.WhiteChromaticity_Y;
        if (ColorCapabilities.AttributeFlags & VIDEO_DEVICE_COLOR) != 0 {
            (*pGdiInfo).ciDevice.RedGamma = ColorCapabilities.RedGamma;
            (*pGdiInfo).ciDevice.GreenGamma = ColorCapabilities.GreenGamma;
            (*pGdiInfo).ciDevice.BlueGamma = ColorCapabilities.BlueGamma;
        } else {
            (*pGdiInfo).ciDevice.RedGamma = ColorCapabilities.WhiteGamma;
            (*pGdiInfo).ciDevice.GreenGamma = ColorCapabilities.WhiteGamma;
            (*pGdiInfo).ciDevice.BlueGamma = ColorCapabilities.WhiteGamma;
        }
    } else {
        (*pGdiInfo).ciDevice.Red.x = 6700;
        (*pGdiInfo).ciDevice.Red.y = 3300;
        (*pGdiInfo).ciDevice.Green.x = 2100;
        (*pGdiInfo).ciDevice.Green.y = 7100;
        (*pGdiInfo).ciDevice.Blue.x = 1400;
        (*pGdiInfo).ciDevice.Blue.y = 800;
        (*pGdiInfo).ciDevice.AlignmentWhite.x = 3127;
        (*pGdiInfo).ciDevice.AlignmentWhite.y = 3290;
        (*pGdiInfo).ciDevice.AlignmentWhite.Y = 0;
        (*pGdiInfo).ciDevice.RedGamma = 20000;
        (*pGdiInfo).ciDevice.GreenGamma = 20000;
        (*pGdiInfo).ciDevice.BlueGamma = 20000;
    }

    (*pGdiInfo).ciDevice.Red.Y = 0;
    (*pGdiInfo).ciDevice.Green.Y = 0;
    (*pGdiInfo).ciDevice.Blue.Y = 0;
    (*pGdiInfo).ciDevice.Cyan.x = 0;
    (*pGdiInfo).ciDevice.Cyan.y = 0;
    (*pGdiInfo).ciDevice.Cyan.Y = 0;
    (*pGdiInfo).ciDevice.Magenta.x = 0;
    (*pGdiInfo).ciDevice.Magenta.y = 0;
    (*pGdiInfo).ciDevice.Magenta.Y = 0;
    (*pGdiInfo).ciDevice.Yellow.x = 0;
    (*pGdiInfo).ciDevice.Yellow.y = 0;
    (*pGdiInfo).ciDevice.Yellow.Y = 0;
    (*pGdiInfo).ciDevice.MagentaInCyanDye = 0;
    (*pGdiInfo).ciDevice.YellowInCyanDye = 0;
    (*pGdiInfo).ciDevice.CyanInMagentaDye = 0;
    (*pGdiInfo).ciDevice.YellowInMagentaDye = 0;
    (*pGdiInfo).ciDevice.CyanInYellowDye = 0;
    (*pGdiInfo).ciDevice.MagentaInYellowDye = 0;
    (*pGdiInfo).ulDevicePelsDPI = 0;
    (*pGdiInfo).ulPrimaryOrder = PRIMARY_ORDER_CBA;
    (*pGdiInfo).ulHTPatternSize = HT_PATSIZE_4x4_M;
    (*pGdiInfo).flHTFlags = HT_FLAG_ADDITIVE_PRIMS;

    (*pDevInfo).flGraphicsCaps = 0;
    (*pDevInfo).lfDefaultFont = SystemFont;
    (*pDevInfo).lfAnsiVarFont = AnsiVariableFont;
    (*pDevInfo).lfAnsiFixFont = AnsiFixedFont;
    (*pDevInfo).cFonts = 0;
    (*pDevInfo).cxDither = 0;
    (*pDevInfo).cyDither = 0;
    (*pDevInfo).hpalDefault = core::ptr::null_mut();
    (*pDevInfo).flGraphicsCaps2 = 0;

    if (*ppdev).BitsPerPixel == 8 {
        (*pGdiInfo).ulNumColors = 20;
        (*pGdiInfo).ulNumPalReg = 1 << (*ppdev).BitsPerPixel;
        (*pGdiInfo).ulHTOutputFormat = HT_FORMAT_8BPP;
        (*pDevInfo).flGraphicsCaps |= GCAPS_PALMANAGED;
        (*pDevInfo).iDitherFormat = BMF_8BPP;
        (*ppdev).PaletteShift = (8 - (*pGdiInfo).ulDACRed) as UCHAR;
    } else {
        (*pGdiInfo).ulNumColors = 0xFFFFFFFF;
        (*pGdiInfo).ulNumPalReg = 0;
        match (*ppdev).BitsPerPixel {
            16 => {
                (*pGdiInfo).ulHTOutputFormat = HT_FORMAT_16BPP;
                (*pDevInfo).iDitherFormat = BMF_16BPP;
            }
            24 => {
                (*pGdiInfo).ulHTOutputFormat = HT_FORMAT_24BPP;
                (*pDevInfo).iDitherFormat = BMF_24BPP;
            }
            _ => {
                (*pGdiInfo).ulHTOutputFormat = HT_FORMAT_32BPP;
                (*pDevInfo).iDitherFormat = BMF_32BPP;
            }
        }
    }

    EngFreeMem(ModeInfo as *mut core::ffi::c_void);
    1
}

#[no_mangle]
pub unsafe extern "system" fn DrvGetModes(
    hDriver: HANDLE,
    _cjSize: ULONG,
    pdm: *mut DEVMODEW,
) -> ULONG {
    let mut ModeCount: ULONG;
    let mut ModeInfoSize: ULONG = 0;
    let mut ModeInfo: PVIDEO_MODE_INFORMATION = core::ptr::null_mut();
    let mut ModeInfoPtr: PVIDEO_MODE_INFORMATION;
    let mut OutputSize: ULONG;
    let mut pdm_ptr: *mut DEVMODEW;

    ModeCount = GetAvailableModes(hDriver, &mut ModeInfo, &mut ModeInfoSize);
    if ModeCount == 0 {
        return 0;
    }

    if pdm.is_null() {
        EngFreeMem(ModeInfo as *mut core::ffi::c_void);
        return ModeCount * core::mem::size_of::<DEVMODEW>() as ULONG;
    }

    OutputSize = 0;
    ModeInfoPtr = ModeInfo;
    pdm_ptr = pdm;

    while ModeCount > 0 {
        ModeCount -= 1;
        if (*ModeInfoPtr).Length == 0 {
            ModeInfoPtr = ((ModeInfoPtr as usize) + (ModeInfoSize as usize))
                as PVIDEO_MODE_INFORMATION;
            continue;
        }

        core::ptr::write_bytes(pdm_ptr as *mut u8, 0, core::mem::size_of::<DEVMODEW>());
        core::ptr::copy_nonoverlapping(
            DEVICE_NAME.as_ptr() as *const u8,
            (*pdm_ptr).dmDeviceName.as_mut_ptr() as *mut u8,
            core::mem::size_of_val(&DEVICE_NAME),
        );
        (*pdm_ptr).dmSpecVersion = DM_SPECVERSION as u16;
        (*pdm_ptr).dmDriverVersion = DM_SPECVERSION as u16;
        (*pdm_ptr).dmSize = core::mem::size_of::<DEVMODEW>() as u16;
        (*pdm_ptr).dmDriverExtra = 0;
        (*pdm_ptr).dmBitsPerPel = (*ModeInfoPtr).NumberOfPlanes * (*ModeInfoPtr).BitsPerPlane;
        (*pdm_ptr).dmPelsWidth = (*ModeInfoPtr).VisScreenWidth;
        (*pdm_ptr).dmPelsHeight = (*ModeInfoPtr).VisScreenHeight;
        (*pdm_ptr).dmDisplayFrequency = (*ModeInfoPtr).Frequency;
        // dmDisplayFlags lives in an anonymous union (with dmNup) in the
        // bindgen DEVMODEW; the C writes 0 here, but write_bytes() above
        // already zeroed the whole struct, so this is a redundant store.
        // Omitted to avoid the anon-union field-path coupling.
        (*pdm_ptr).dmFields = DM_BITSPERPEL
            | DM_PELSWIDTH
            | DM_PELSHEIGHT
            | DM_DISPLAYFREQUENCY
            | DM_DISPLAYFLAGS;

        ModeInfoPtr =
            ((ModeInfoPtr as usize) + (ModeInfoSize as usize)) as PVIDEO_MODE_INFORMATION;
        pdm_ptr = ((pdm_ptr as usize) + core::mem::size_of::<DEVMODEW>()) as *mut DEVMODEW;
        OutputSize += core::mem::size_of::<DEVMODEW>() as ULONG;
    }

    EngFreeMem(ModeInfo as *mut core::ffi::c_void);
    OutputSize
}