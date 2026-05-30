use win32k_sys::win32k::*;

use crate::framebuf::*;
use crate::palette::IntSetPalette;
#[cfg(feature = "experimental_mouse_cursor")]
use crate::pointer::DrvSetPointerShape;

#[no_mangle]
pub unsafe extern "system" fn DrvEnableSurface(dhpdev: DHPDEV) -> HSURF {
    let ppdev = dhpdev as PPDEV;
    let mut ulTemp: ULONG = 0;

    if EngDeviceIoControl(
        (*ppdev).hDriver,
        IOCTL_VIDEO_SET_CURRENT_MODE,
        core::ptr::addr_of!((*ppdev).ModeIndex) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(ulTemp),
    ) != 0
    {
        return core::ptr::null_mut();
    }

    let mut VideoMemory = VIDEO_MEMORY {
        RequestedVirtualAddress: core::ptr::null_mut(),
    };
    let mut VideoMemoryInfo: VIDEO_MEMORY_INFORMATION = core::mem::zeroed();

    if EngDeviceIoControl(
        (*ppdev).hDriver,
        IOCTL_VIDEO_MAP_VIDEO_MEMORY,
        core::ptr::addr_of_mut!(VideoMemory) as PVOID,
        core::mem::size_of::<VIDEO_MEMORY>() as ULONG,
        core::ptr::addr_of_mut!(VideoMemoryInfo) as PVOID,
        core::mem::size_of::<VIDEO_MEMORY_INFORMATION>() as ULONG,
        core::ptr::addr_of_mut!(ulTemp),
    ) != 0
    {
        return core::ptr::null_mut();
    }

    (*ppdev).ScreenPtr = VideoMemoryInfo.FrameBufferBase;

    let BitmapType = match (*ppdev).BitsPerPixel {
        8 => {
            IntSetPalette(dhpdev, (*ppdev).PaletteEntries, 0, 256);
            BMF_8BPP
        }
        16 => BMF_16BPP,
        24 => BMF_24BPP,
        32 => BMF_32BPP,
        _ => return core::ptr::null_mut(),
    };

    (*ppdev).iDitherFormat = BitmapType;

    let ScreenSize = SIZEL {
        cx: (*ppdev).ScreenWidth as LONG,
        cy: (*ppdev).ScreenHeight as LONG,
    };

    let hSurface = EngCreateBitmap(
        ScreenSize,
        (*ppdev).ScreenDelta as LONG,
        BitmapType,
        if (*ppdev).ScreenDelta > 0 { BMF_TOPDOWN } else { 0 },
        (*ppdev).ScreenPtr,
    ) as HSURF;

    if hSurface.is_null() {
        return core::ptr::null_mut();
    }

    if EngAssociateSurface(hSurface, (*ppdev).hDevEng, 0) == 0 {
        EngDeleteSurface(hSurface);
        return core::ptr::null_mut();
    }

    (*ppdev).hSurfEng = hSurface;

    hSurface
}

#[no_mangle]
pub unsafe extern "system" fn DrvDisableSurface(dhpdev: DHPDEV) {
    let ppdev = dhpdev as PPDEV;
    let mut ulTemp: ULONG = 0;

    EngDeleteSurface((*ppdev).hSurfEng);
    (*ppdev).hSurfEng = core::ptr::null_mut();

    #[cfg(feature = "experimental_mouse_cursor")]
    DrvSetPointerShape(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
        0,
        0,
        0,
        core::ptr::null_mut(),
        0,
    );

    let mut VideoMemory = VIDEO_MEMORY {
        RequestedVirtualAddress: (*ppdev).ScreenPtr,
    };

    EngDeviceIoControl(
        (*ppdev).hDriver,
        IOCTL_VIDEO_UNMAP_VIDEO_MEMORY,
        core::ptr::addr_of_mut!(VideoMemory) as PVOID,
        core::mem::size_of::<VIDEO_MEMORY>() as ULONG,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(ulTemp),
    );
}

#[no_mangle]
pub unsafe extern "system" fn DrvAssertMode(dhpdev: DHPDEV, bEnable: BOOL) -> BOOL {
    let ppdev = dhpdev as PPDEV;
    let mut ulTemp: ULONG = 0;

    if bEnable != 0 {
        if EngDeviceIoControl(
            (*ppdev).hDriver,
            IOCTL_VIDEO_SET_CURRENT_MODE,
            core::ptr::addr_of!((*ppdev).ModeIndex) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(ulTemp),
        ) != 0
        {
            return 0 as BOOL;
        }

        if (*ppdev).BitsPerPixel == 8 {
            IntSetPalette(dhpdev, (*ppdev).PaletteEntries, 0, 256);
        }

        1 as BOOL
    } else {
        if EngDeviceIoControl(
            (*ppdev).hDriver,
            IOCTL_VIDEO_RESET_DEVICE,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(ulTemp),
        ) == 0
        {
            1 as BOOL
        } else {
            0 as BOOL
        }
    }
}