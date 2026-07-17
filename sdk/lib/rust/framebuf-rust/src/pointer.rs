use win32k_sys::win32k::*;

#[cfg(not(feature = "experimental_mouse_cursor_support"))]
#[no_mangle]
pub unsafe extern "system" fn DrvSetPointerShape(
    pso: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    psoColor: *mut SURFOBJ,
    pxlo: *mut XLATEOBJ,
    xHot: i32,
    yHot: i32,
    x: i32,
    y: i32,
    prcl: *mut RECTL,
    fl: FLONG,
) -> u32 {
    EngSetPointerShape(pso, psoMask, psoColor, pxlo, xHot, yHot, x, y, prcl, fl)
}

#[cfg(not(feature = "experimental_mouse_cursor_support"))]
#[no_mangle]
pub unsafe extern "system" fn DrvMovePointer(
    pso: *mut SURFOBJ,
    x: i32,
    y: i32,
    prcl: *mut RECTL,
) {
    EngMovePointer(pso, x, y, prcl);
}

#[cfg(feature = "experimental_mouse_cursor_support")]
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntHideMousePointer(
    ppdev: *mut PDEV,
    DestSurface: *mut SURFOBJ,
) {
    if (*ppdev).PointerAttributes.Enable == 0u8 {
        return;
    }

    (*ppdev).PointerAttributes.Enable = 0u8;
    if (*ppdev).PointerSaveSurface != ptr::null_mut() {
        let mut DestRect: RECTL;
        let mut SrcPoint: POINTL;

        DestRect.left = core::cmp::max((*ppdev).PointerAttributes.Column, 0);
        DestRect.top = core::cmp::max((*ppdev).PointerAttributes.Row, 0);
        DestRect.right = core::cmp::min(
            (*ppdev).PointerAttributes.Column + (*ppdev).PointerAttributes.Width,
            (*ppdev).ScreenWidth - 1,
        );
        DestRect.bottom = core::cmp::min(
            (*ppdev).PointerAttributes.Row + (*ppdev).PointerAttributes.Height,
            (*ppdev).ScreenHeight - 1,
        );

        SrcPoint.x = core::cmp::max(-(*ppdev).PointerAttributes.Column, 0);
        SrcPoint.y = core::cmp::max(-(*ppdev).PointerAttributes.Row, 0);

        let SaveSurface = EngLockSurface((*ppdev).PointerSaveSurface);
        let MaskSurface = EngLockSurface((*ppdev).PointerMaskSurface);
        EngBitBlt(
            DestSurface,
            SaveSurface,
            MaskSurface,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut DestRect,
            &mut SrcPoint,
            &mut SrcPoint,
            ptr::null_mut(),
            ptr::null_mut(),
            SRCCOPY,
        );
        EngUnlockSurface(MaskSurface);
        EngUnlockSurface(SaveSurface);
    }
}

#[cfg(feature = "experimental_mouse_cursor_support")]
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntHideMousePointer(
    ppdev: *mut PDEV,
    DestSurface: *mut SURFOBJ,
) {
    if (*ppdev).PointerAttributes.Enable == 0u8 {
        return;
    }

    (*ppdev).PointerAttributes.Enable = 0u8;
    if (*ppdev).PointerSaveSurface != ptr::null_mut() {
        let mut DestRect: RECTL;
        let mut SrcPoint: POINTL;

        DestRect.left = core::cmp::max((*ppdev).PointerAttributes.Column, 0);
        DestRect.top = core::cmp::max((*ppdev).PointerAttributes.Row, 0);
        DestRect.right = core::cmp::min(
            (*ppdev).PointerAttributes.Column + (*ppdev).PointerAttributes.Width,
            (*ppdev).ScreenWidth - 1,
        );
        DestRect.bottom = core::cmp::min(
            (*ppdev).PointerAttributes.Row + (*ppdev).PointerAttributes.Height,
            (*ppdev).ScreenHeight - 1,
        );

        SrcPoint.x = core::cmp::max(-(*ppdev).PointerAttributes.Column, 0);
        SrcPoint.y = core::cmp::max(-(*ppdev).PointerAttributes.Row, 0);

        let SaveSurface = EngLockSurface((*ppdev).PointerSaveSurface);
        let MaskSurface = EngLockSurface((*ppdev).PointerMaskSurface);
        EngBitBlt(
            DestSurface,
            SaveSurface,
            MaskSurface,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut DestRect,
            &mut SrcPoint,
            &mut SrcPoint,
            ptr::null_mut(),
            ptr::null_mut(),
            SRCCOPY,
        );
        EngUnlockSurface(MaskSurface);
        EngUnlockSurface(SaveSurface);
    }
}

#[cfg(feature = "experimental_mouse_cursor_support")]
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntShowMousePointer(
    ppdev: *mut PDEV,
    DestSurface: *mut SURFOBJ,
) {
    if (*ppdev).PointerAttributes.Enable != 0u8 {
        return;
    }

    (*ppdev).PointerAttributes.Enable = 1u8;

    if (*ppdev).PointerSaveSurface != ptr::null_mut() {
        let mut DestRect: RECTL;
        let mut SrcPoint: POINTL;

        SrcPoint.x = core::cmp::max((*ppdev).PointerAttributes.Column, 0);
        SrcPoint.y = core::cmp::max((*ppdev).PointerAttributes.Row, 0);

        DestRect.left = SrcPoint.x - (*ppdev).PointerAttributes.Column;
        DestRect.top = SrcPoint.y - (*ppdev).PointerAttributes.Row;
        DestRect.right = core::cmp::min(
            (*ppdev).PointerAttributes.Width,
            (*ppdev).ScreenWidth - (*ppdev).PointerAttributes.Column - 1,
        );
        DestRect.bottom = core::cmp::min(
            (*ppdev).PointerAttributes.Height,
            (*ppdev).ScreenHeight - (*ppdev).PointerAttributes.Row - 1,
        );

        let SaveSurface = EngLockSurface((*ppdev).PointerSaveSurface);
        EngBitBlt(
            SaveSurface,
            DestSurface,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut DestRect,
            &mut SrcPoint,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            SRCCOPY,
        );
        EngUnlockSurface(SaveSurface);
    }

    {
        let mut DestRect: RECTL;
        let mut SrcPoint: POINTL;

        DestRect.left = core::cmp::max((*ppdev).PointerAttributes.Column, 0);
        DestRect.top = core::cmp::max((*ppdev).PointerAttributes.Row, 0);
        DestRect.right = core::cmp::min(
            (*ppdev).PointerAttributes.Column + (*ppdev).PointerAttributes.Width,
            (*ppdev).ScreenWidth - 1,
        );
        DestRect.bottom = core::cmp::min(
            (*ppdev).PointerAttributes.Row + (*ppdev).PointerAttributes.Height,
            (*ppdev).ScreenHeight - 1,
        );

        SrcPoint.x = core::cmp::max(-(*ppdev).PointerAttributes.Column, 0);
        SrcPoint.y = core::cmp::max(-(*ppdev).PointerAttributes.Row, 0);

        let MaskSurf = EngLockSurface((*ppdev).PointerMaskSurface);
        if (*ppdev).PointerColorSurface != ptr::null_mut() {
            let ColorSurf = EngLockSurface((*ppdev).PointerColorSurface);
            EngBitBlt(
                DestSurface,
                ColorSurf,
                MaskSurf,
                ptr::null_mut(),
                (*ppdev).PointerXlateObject,
                &mut DestRect,
                &mut SrcPoint,
                &mut SrcPoint,
                ptr::null_mut(),
                ptr::null_mut(),
                0xAACC,
            );
            EngUnlockSurface(ColorSurf);
        } else {
            EngBitBlt(
                DestSurface,
                MaskSurf,
                ptr::null_mut(),
                ptr::null_mut(),
                (*ppdev).PointerXlateObject,
                &mut DestRect,
                &mut SrcPoint,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                SRCAND,
            );
            SrcPoint.y += (*ppdev).PointerAttributes.Height;
            EngBitBlt(
                DestSurface,
                MaskSurf,
                ptr::null_mut(),
                ptr::null_mut(),
                (*ppdev).PointerXlateObject,
                &mut DestRect,
                &mut SrcPoint,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                SRCINVERT,
            );
        }
        EngUnlockSurface(MaskSurf);
    }
}

#[cfg(feature = "experimental_mouse_cursor_support")]
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntShowMousePointer(
    ppdev: *mut PDEV,
    DestSurface: *mut SURFOBJ,
) {
    if (*ppdev).PointerAttributes.Enable != 0u8 {
        return;
    }

    (*ppdev).PointerAttributes.Enable = 1u8;

    if (*ppdev).PointerSaveSurface != ptr::null_mut() {
        let mut DestRect: RECTL;
        let mut SrcPoint: POINTL;

        SrcPoint.x = core::cmp::max((*ppdev).PointerAttributes.Column, 0);
        SrcPoint.y = core::cmp::max((*ppdev).PointerAttributes.Row, 0);

        DestRect.left = SrcPoint.x - (*ppdev).PointerAttributes.Column;
        DestRect.top = SrcPoint.y - (*ppdev).PointerAttributes.Row;
        DestRect.right = core::cmp::min(
            (*ppdev).PointerAttributes.Width,
            (*ppdev).ScreenWidth - (*ppdev).PointerAttributes.Column - 1,
        );
        DestRect.bottom = core::cmp::min(
            (*ppdev).PointerAttributes.Height,
            (*ppdev).ScreenHeight - (*ppdev).PointerAttributes.Row - 1,
        );

        let SaveSurface = EngLockSurface((*ppdev).PointerSaveSurface);
        EngBitBlt(
            SaveSurface,
            DestSurface,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut DestRect,
            &mut SrcPoint,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            SRCCOPY,
        );
        EngUnlockSurface(SaveSurface);
    }

    {
        let mut DestRect: RECTL;
        let mut SrcPoint: POINTL;

        DestRect.left = core::cmp::max((*ppdev).PointerAttributes.Column, 0);
        DestRect.top = core::cmp::max((*ppdev).PointerAttributes.Row, 0);
        DestRect.right = core::cmp::min(
            (*ppdev).PointerAttributes.Column + (*ppdev).PointerAttributes.Width,
            (*ppdev).ScreenWidth - 1,
        );
        DestRect.bottom = core::cmp::min(
            (*ppdev).PointerAttributes.Row + (*ppdev).PointerAttributes.Height,
            (*ppdev).ScreenHeight - 1,
        );

        SrcPoint.x = core::cmp::max(-(*ppdev).PointerAttributes.Column, 0);
        SrcPoint.y = core::cmp::max(-(*ppdev).PointerAttributes.Row, 0);

        let MaskSurf = EngLockSurface((*ppdev).PointerMaskSurface);
        if (*ppdev).PointerColorSurface != ptr::null_mut() {
            let ColorSurf = EngLockSurface((*ppdev).PointerColorSurface);
            EngBitBlt(
                DestSurface,
                ColorSurf,
                MaskSurf,
                ptr::null_mut(),
                (*ppdev).PointerXlateObject,
                &mut DestRect,
                &mut SrcPoint,
                &mut SrcPoint,
                ptr::null_mut(),
                ptr::null_mut(),
                0xAACC,
            );
            EngUnlockSurface(ColorSurf);
        } else {
            EngBitBlt(
                DestSurface,
                MaskSurf,
                ptr::null_mut(),
                ptr::null_mut(),
                (*ppdev).PointerXlateObject,
                &mut DestRect,
                &mut SrcPoint,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                SRCAND,
            );
            SrcPoint.y += (*ppdev).PointerAttributes.Height;
            EngBitBlt(
                DestSurface,
                MaskSurf,
                ptr::null_mut(),
                ptr::null_mut(),
                (*ppdev).PointerXlateObject,
                &mut DestRect,
                &mut SrcPoint,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                SRCINVERT,
            );
        }
        EngUnlockSurface(MaskSurf);
    }
}

#[cfg(feature = "experimental_mouse_cursor_support")]
#[no_mangle]
pub unsafe extern "system" fn DrvSetPointerShape(
    pso: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    psoColor: *mut SURFOBJ,
    pxlo: *mut XLATEOBJ,
    xHot: i32,
    yHot: i32,
    x: i32,
    y: i32,
    prcl: *mut RECTL,
    fl: FLONG,
) -> u32 {
    let ppdev = (*pso).dhpdev as *mut PDEV;

    IntHideMousePointer(ppdev, pso);

    if (*ppdev).PointerColorSurface != ptr::null_mut() {
        let TempSurfObj = EngLockSurface((*ppdev).PointerColorSurface);
        EngFreeMem((*TempSurfObj).pvBits);
        (*TempSurfObj).pvBits = ptr::null_mut();
        EngUnlockSurface(TempSurfObj);

        EngDeleteSurface((*ppdev).PointerColorSurface);
        (*ppdev).PointerColorSurface = ptr::null_mut();
    }

    if (*ppdev).PointerMaskSurface != ptr::null_mut() {
        let TempSurfObj = EngLockSurface((*ppdev).PointerMaskSurface);
        EngFreeMem((*TempSurfObj).pvBits);
        (*TempSurfObj).pvBits = ptr::null_mut();
        EngUnlockSurface(TempSurfObj);

        EngDeleteSurface((*ppdev).PointerMaskSurface);
        (*ppdev).PointerMaskSurface = ptr::null_mut();
    }

    if (*ppdev).PointerSaveSurface != ptr::null_mut() {
        EngDeleteSurface((*ppdev).PointerSaveSurface);
        (*ppdev).PointerSaveSurface = ptr::null_mut();
    }

    if psoMask.is_null() {
        return SPS_ACCEPT_EXCLUDE;
    }

    (*ppdev).PointerHotSpot.x = xHot;
    (*ppdev).PointerHotSpot.y = yHot;

    (*ppdev).PointerXlateObject = pxlo;
    (*ppdev).PointerAttributes.Column = x - xHot;
    (*ppdev).PointerAttributes.Row = y - yHot;
    (*ppdev).PointerAttributes.Width = (*psoMask).lDelta << 3;
    (*ppdev).PointerAttributes.Height = ((*psoMask).cjBits / (*psoMask).lDelta as u32) >> 1;

    if !psoColor.is_null() {
        let mut Size: SIZEL;

        Size.cx = (*ppdev).PointerAttributes.Width;
        Size.cy = (*ppdev).PointerAttributes.Height;
        let Bits = EngAllocMem(0, (*psoColor).cjBits, ALLOC_TAG) as PBYTE;
        core::ptr::copy_nonoverlapping(
            (*psoColor).pvBits as *const u8,
            Bits,
            (*psoColor).cjBits as usize,
        );

        (*ppdev).PointerColorSurface = EngCreateBitmap(
            Size,
            (*psoColor).lDelta,
            (*psoColor).iBitmapFormat,
            0,
            Bits as *mut c_void,
        ) as HSURF;
    }

    {
        let mut Size: SIZEL;

        Size.cx = (*ppdev).PointerAttributes.Width;
        Size.cy = (*ppdev).PointerAttributes.Height << 1;
        let Bits = EngAllocMem(0, (*psoMask).cjBits, ALLOC_TAG) as PBYTE;
        core::ptr::copy_nonoverlapping(
            (*psoMask).pvBits as *const u8,
            Bits,
            (*psoMask).cjBits as usize,
        );

        (*ppdev).PointerMaskSurface = EngCreateBitmap(
            Size,
            (*psoMask).lDelta,
            (*psoMask).iBitmapFormat,
            0,
            Bits as *mut c_void,
        ) as HSURF;
    }

    {
        let mut Size: SIZEL;
        let lDelta: i32;

        Size.cx = (*ppdev).PointerAttributes.Width;
        Size.cy = (*ppdev).PointerAttributes.Height;

        lDelta = match (*pso).iBitmapFormat {
            BMF_8BPP => Size.cx,
            BMF_16BPP => Size.cx << 1,
            BMF_24BPP => Size.cx * 3,
            BMF_32BPP => Size.cx << 2,
            _ => Size.cx,
        };

        (*ppdev).PointerSaveSurface = EngCreateBitmap(
            Size,
            lDelta,
            (*pso).iBitmapFormat,
            BMF_NOZEROINIT,
            ptr::null_mut(),
        ) as HSURF;
    }

    IntShowMousePointer(ppdev, pso);

    SPS_ACCEPT_EXCLUDE
}

#[cfg(feature = "experimental_mouse_cursor_support")]
#[no_mangle]
pub unsafe extern "system" fn DrvMovePointer(
    pso: *mut SURFOBJ,
    x: i32,
    y: i32,
    prcl: *mut RECTL,
) {
    let ppdev = (*pso).dhpdev as *mut PDEV;

    let WasVisible = (*ppdev).PointerAttributes.Enable;
    if WasVisible != 0u8 {
        IntHideMousePointer(ppdev, pso);
    }

    if x == -1 {
        return;
    }

    (*ppdev).PointerAttributes.Column = x - (*ppdev).PointerHotSpot.x;
    (*ppdev).PointerAttributes.Row = y - (*ppdev).PointerHotSpot.y;

    if WasVisible != 0u8 {
        IntShowMousePointer(ppdev, pso);
    }
}