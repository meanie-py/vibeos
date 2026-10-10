use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn AttachHBITMAP(phbm: *mut HBITMAP, hbmNew: HBITMAP) {
    if hbmNew.is_null() {
        return;
    }
    DeleteObject((*phbm).cast());
    *phbm = hbmNew;
}

impl SelectionModel {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hbmColor = core::ptr::null_mut();
        (*this_).m_hbmMask = core::ptr::null_mut();
        (*this_).m_rgbBack = RGB!(255, 255, 255);
        (*this_).m_bShow = (FALSE) as i32;
        (*this_).m_bContentChanged = (FALSE) as i32;
        {
            (*this_).m_rc.SetRectEmpty();
            (*this_).m_rcOld.SetRectEmpty();
            (*this_).m_ptHit = POINT { x: -1, y: -1 };
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).ClearColorImage();
        (*this_).ClearMaskImage();
    }
    pub unsafe fn SetRectFromPoints(&mut self, ptFrom: &POINT, ptTo: &POINT) {
        let this_: *mut Self = self;
        (*this_).m_rc = CRect__make_5(*ptFrom, *ptTo);
        (*this_).m_rc.NormalizeRect();
    }
    pub unsafe fn setMask(&mut self, rc: &CRect, hbmMask: HBITMAP) {
        let this_: *mut Self = self;
        if !(*this_).m_hbmMask.is_null() {
            DeleteObject((*this_).m_hbmMask as HGDIOBJ);
        }
    
        (*this_).m_hbmMask = hbmMask;
        (*this_).m_rcOld = core::ptr::read(rc as *const CRect);
        (*this_).m_rc = core::ptr::read(rc as *const CRect);
    }
    pub unsafe fn TakeOff(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).IsLanded() == 0 || (*this_).m_rc.IsRectEmpty() != 0 {
            return (FALSE) as BOOL;
        }
    
        // The background color is needed for transparency of selection
        (*this_).m_rgbBack = paletteModel.GetBgColor();
    
        // Get the contents of the selection area
        (*this_).ClearColorImage();
        (*this_).m_hbmColor = (*this_).GetSelectionContents();
    
        // RectSel doesn't need the mask image
        if toolsModel.GetActiveTool() == TOOL_RECTSEL {
            (*this_).ClearMaskImage();
        }
    
        // Save the selection area
        (*this_).m_rcOld = (*this_).m_rc;
    
        imageModel.NotifyImageChanged();
        return (TRUE) as BOOL;
    }
    pub unsafe fn Landing(&mut self) {
        let this_: *mut Self = self;
        if (*this_).IsLanded() != 0 && (*this_).m_bShow == 0 {
            imageModel.NotifyImageChanged();
            return;
        }
    
        if (*this_).m_bContentChanged != 0
            || ((*this_).m_rc.EqualRect((*this_).m_rcOld.op_to_tagRECT()) == 0
                && (*this_).m_rc.IsRectEmpty() == 0
                && (*this_).m_rcOld.IsRectEmpty() == 0)
        {
            let mut rc: CRect = CRect__make_0();
            rc.UnionRect((*this_).m_rc.op_to_tagRECT(), (*this_).m_rcOld.op_to_tagRECT());
            imageModel.PushImageForUndo__2(&((rc).__base0));
    
            canvasWindow.m_drawing = (FALSE) as i32;
            toolsModel.OnDrawOverlayOnImage(imageModel.GetDC());
        }
    
        (*this_).HideSelection();
    }
    pub unsafe fn IsLanded(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return ((*this_).m_hbmColor.is_null()) as BOOL;
    }
    pub unsafe fn HideSelection(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_bShow = (FALSE) as i32;
        (*this_).m_bContentChanged = (FALSE) as i32;
        (*this_).ClearColorImage();
        (*this_).ClearMaskImage();
        (*this_).m_rc.SetRectEmpty();
        (*this_).m_rcOld.SetRectEmpty();
        (*core::ptr::addr_of_mut!(imageModel)).NotifyImageChanged();
    }
    pub unsafe fn DeleteSelection(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_bShow == 0 {
            return;
        }
    
        (*this_).TakeOff();
        imageModel.PushImageForUndo__0();
        (*this_).DrawBackground(imageModel.GetDC(), paletteModel.GetBgColor());
    
        (*this_).HideSelection();
    }
    pub unsafe fn hitTest(&mut self, ptCanvas: POINT) -> HITTEST {
        let this_: *mut Self = self;
        if (*this_).m_bShow == 0 {
            return HIT_NONE;
        }
    
        let mut rcSelection: CRect = {
            let mut tmp: CRect = CRect__make_0();
            tmp = (*this_).m_rc;
            tmp
        };
        canvasWindow.ImageToCanvas__1(&mut rcSelection.__base0);
        rcSelection.InflateRect__0((GRIP_SIZE as i32), (GRIP_SIZE as i32));
        return getSizeBoxHitTest(ptCanvas, &rcSelection.__base0);
    }
    pub unsafe fn drawFrameOnCanvas(&mut self, hCanvasDC: HDC) {
        let this_: *mut Self = self;
        if (*this_).m_bShow == 0 {
            return;
        }
    
        let mut rcSelection: CRect = CRect__make_0();
        rcSelection = (*this_).m_rc;
        canvasWindow.ImageToCanvas__1((rcSelection).__base0);
        rcSelection.InflateRect__0((GRIP_SIZE as i32), (GRIP_SIZE as i32));
        drawSizeBoxes(hCanvasDC, core::ptr::addr_of!(rcSelection.__base0) as LPCRECT, (TRUE) as BOOL, core::ptr::null());
    }
    pub unsafe fn moveSelection(&mut self, xDelta: INT, yDelta: INT) {
        let this_: *mut Self = self;
        if (*this_).m_bShow == 0 {
            return;
        }
    
        (*this_).TakeOff();
        (*this_).m_rc.OffsetRect__0(xDelta, yDelta);
        canvasWindow.__base0.Invalidate((TRUE) as BOOL);
    }
    pub unsafe fn GetSelectionContents(&mut self) -> HBITMAP {
        let this_: *mut Self = self;
        let hbmWhole = imageModel.LockBitmap();
        let hbmPart = getSubImage(hbmWhole, &((if (*this_).IsLanded() != 0 { (*this_).m_rc } else { (*this_).m_rcOld }).__base0));
        imageModel.UnlockBitmap(hbmWhole);
        if hbmPart.is_null() {
            return core::ptr::null_mut();
        }
    
        let mut rc: CRect = CRect__make_1(0, 0, (*this_).m_rc.Width(), (*this_).m_rc.Height());
    
        let hdcMem = CreateCompatibleDC(core::ptr::null_mut());
        let hbmNew = CreateColorDIB(rc.Width(), rc.Height(), paletteModel.GetBgColor());
        let hbmOld = SelectObject(hdcMem, (hbmNew).cast());
        selectionModel.DrawSelection__0(hdcMem, paletteModel.GetBgColor(), (TRUE) as BOOL, &rc, hbmPart);
        SelectObject(hdcMem, hbmOld);
        DeleteDC(hdcMem);
    
        DeleteObject((hbmPart).cast());
        return hbmNew;
    }
    pub unsafe fn DrawBackground(&mut self, hDCImage: HDC, crBg: COLORREF) {
        let this_: *mut Self = self;
        if toolsModel.GetActiveTool() == TOOL_FREESEL {
            (*this_).DrawBackgroundPoly(hDCImage, crBg);
        } else {
            (*this_).DrawBackgroundRect(hDCImage, crBg);
        }
    }
    pub unsafe fn DrawBackgroundPoly(&mut self, hDCImage: HDC, crBg: COLORREF) {
        let this_: *mut Self = self;
        if (*this_).m_rcOld.IsRectEmpty() != 0 {
            return;
        }
    
        let hbrOld: HGDIOBJ = SelectObject(hDCImage, GetStockObject((DC_BRUSH as i32)));
        SetDCBrushColor(hDCImage, crBg);
        MaskBlt(hDCImage, (*this_).m_rcOld.left, (*this_).m_rcOld.top, (*this_).m_rcOld.Width(), (*this_).m_rcOld.Height(),
                  hDCImage, (*this_).m_rcOld.left, (*this_).m_rcOld.top, (*this_).m_hbmMask, 0, 0, MAKEROP4!(PATCOPY, SRCCOPY));
        SelectObject(hDCImage, hbrOld);
    }
    pub unsafe fn DrawBackgroundRect(&mut self, hDCImage: HDC, crBg: COLORREF) {
        let this_: *mut Self = self;
        if (*this_).m_rcOld.IsRectEmpty() != 0 {
            return;
        }
    
        Rect(hDCImage, (*this_).m_rcOld.left, (*this_).m_rcOld.top, (*this_).m_rcOld.right, (*this_).m_rcOld.bottom, crBg, crBg, 0, 1);
    }
    pub unsafe fn DrawSelection__0(&mut self, hDCImage: HDC, crBg: COLORREF, bBgTransparent: BOOL, rc: &CRect, hbm: HBITMAP) {
        let this_: *mut Self = self;
        let rc_mut: *mut CRect = rc as *const CRect as *mut CRect;
        if (*rc_mut).IsRectEmpty() != 0 {
            return;
        }
    
        let mut bm: BITMAP = core::mem::zeroed();
        if GetObjectW((hbm).cast(), core::mem::size_of::<BITMAP>() as i32, core::ptr::addr_of_mut!(bm) as *mut core::ffi::c_void) == 0 {
            return;
        }
    
        let keyColor: COLORREF = if bBgTransparent != 0 { crBg } else { CLR_INVALID as COLORREF };
    
        let hMemDC: HDC = CreateCompatibleDC(hDCImage);
        let hbmOld: HGDIOBJ = SelectObject(hMemDC, (hbm).cast());
        ColorKeyedMaskBlt(hDCImage, (*rc_mut).__base0.left, (*rc_mut).__base0.top, (*rc_mut).Width(), (*rc_mut).Height(),
                          hMemDC, 0, 0, bm.bmWidth, bm.bmHeight, (*this_).m_hbmMask, keyColor);
        SelectObject(hMemDC, hbmOld);
        DeleteDC(hMemDC);
    }
    pub unsafe fn InsertFromHBITMAP(&mut self, hbmColor: HBITMAP, x: i32, y: i32, hbmMask: HBITMAP) {
        let this_: *mut Self = self;
        DeleteObject(((*this_).m_hbmColor).cast());
        (*this_).m_hbmColor = hbmColor;
    
        (*this_).m_rc.left = x;
        (*this_).m_rc.top = y;
        (*this_).m_rc.right = x + GetDIBWidth(hbmColor);
        (*this_).m_rc.bottom = y + GetDIBHeight(hbmColor);
    
        if !hbmMask.is_null() {
            DeleteObject(((*this_).m_hbmMask).cast());
            (*this_).m_hbmMask = hbmMask;
        } else {
            (*this_).ClearMaskImage();
        }
    
        (*this_).NotifyContentChanged();
    }
    pub unsafe fn FlipHorizontally(&mut self) {
        let this_: *mut Self = self;
        (*this_).TakeOff();
    
        let hdcMem: HDC = CreateCompatibleDC(core::ptr::null_mut());
        if ((*this_).m_hbmMask as usize) != 0 {
            SelectObject(hdcMem, ((*this_).m_hbmMask).cast());
            StretchBlt(hdcMem, (*this_).m_rc.Width() - 1, 0, -(*this_).m_rc.Width(), (*this_).m_rc.Height(),
                         hdcMem, 0, 0, (*this_).m_rc.Width(), (*this_).m_rc.Height(), SRCCOPY);
        }
        if ((*this_).m_hbmColor as usize) != 0 {
            SelectObject(hdcMem, ((*this_).m_hbmColor).cast());
            StretchBlt(hdcMem, (*this_).m_rc.Width() - 1, 0, -(*this_).m_rc.Width(), (*this_).m_rc.Height(),
                         hdcMem, 0, 0, (*this_).m_rc.Width(), (*this_).m_rc.Height(), SRCCOPY);
        }
        DeleteDC(hdcMem);
    
        (*this_).NotifyContentChanged();
    }
    pub unsafe fn FlipVertically(&mut self) {
        let this_: *mut Self = self;
        (*this_).TakeOff();
    
        let hdcMem: HDC = CreateCompatibleDC(core::ptr::null_mut());
        if (*this_).m_hbmMask as usize != 0 {
            SelectObject(hdcMem, ((*this_).m_hbmMask).cast());
            StretchBlt(hdcMem, 0, (*this_).m_rc.Height() - 1, (*this_).m_rc.Width(), -(*this_).m_rc.Height(),
                         hdcMem, 0, 0, (*this_).m_rc.Width(), (*this_).m_rc.Height(), SRCCOPY);
        }
        if (*this_).m_hbmColor as usize != 0 {
            SelectObject(hdcMem, ((*this_).m_hbmColor).cast());
            StretchBlt(hdcMem, 0, (*this_).m_rc.Height() - 1, (*this_).m_rc.Width(), -(*this_).m_rc.Height(),
                         hdcMem, 0, 0, (*this_).m_rc.Width(), (*this_).m_rc.Height(), SRCCOPY);
        }
        DeleteDC(hdcMem);
    
        (*this_).NotifyContentChanged();
    }
    pub unsafe fn RotateNTimes90Degrees(&mut self, iN: i32) {
        let this_: *mut Self = self;
        let mut hbm: HBITMAP = core::mem::zeroed();
        let mut hbmOld: HGDIOBJ = core::mem::zeroed();
        let hdcMem: HDC = CreateCompatibleDC(core::ptr::null_mut());
    
        match iN {
            1 | 3 => {
                (*this_).TakeOff();
    
                if !(*this_).m_hbmColor.is_null() {
                    hbmOld = SelectObject(hdcMem, ((*this_).m_hbmColor).cast());
                    hbm = Rotate90DegreeBlt(hdcMem, (*this_).m_rc.Width(), (*this_).m_rc.Height(), (iN == 1) as i32, (FALSE) as BOOL);
                    SelectObject(hdcMem, hbmOld);
                    DeleteObject(((*this_).m_hbmColor).cast());
                    (*this_).m_hbmColor = hbm;
                }
                if !(*this_).m_hbmMask.is_null() {
                    hbmOld = SelectObject(hdcMem, ((*this_).m_hbmMask).cast());
                    hbm = Rotate90DegreeBlt(hdcMem, (*this_).m_rc.Width(), (*this_).m_rc.Height(), (iN == 1) as i32, (TRUE) as BOOL);
                    SelectObject(hdcMem, hbmOld);
                    DeleteObject(((*this_).m_hbmMask).cast());
                    (*this_).m_hbmMask = hbm;
                }
    
                (*this_).SwapWidthAndHeight();
            }
    
            2 => {
                (*this_).TakeOff();
    
                if !(*this_).m_hbmColor.is_null() {
                    hbmOld = SelectObject(hdcMem, ((*this_).m_hbmColor).cast());
                    StretchBlt(hdcMem, (*this_).m_rc.Width() - 1, (*this_).m_rc.Height() - 1, -(*this_).m_rc.Width(), -(*this_).m_rc.Height(),
                                 hdcMem, 0, 0, (*this_).m_rc.Width(), (*this_).m_rc.Height(), SRCCOPY);
                    SelectObject(hdcMem, hbmOld);
                }
                if !(*this_).m_hbmMask.is_null() {
                    hbmOld = SelectObject(hdcMem, ((*this_).m_hbmMask).cast());
                    StretchBlt(hdcMem, (*this_).m_rc.Width() - 1, (*this_).m_rc.Height() - 1, -(*this_).m_rc.Width(), -(*this_).m_rc.Height(),
                                 hdcMem, 0, 0, (*this_).m_rc.Width(), (*this_).m_rc.Height(), SRCCOPY);
                    SelectObject(hdcMem, hbmOld);
                }
            }
    
            _ => {}
        }
    
        DeleteDC(hdcMem);
        (*this_).NotifyContentChanged();
    }
    pub unsafe fn StretchSkew(&mut self, nStretchPercentX: i32, nStretchPercentY: i32, nSkewDegX: i32, nSkewDegY: i32) {
        let this_: *mut Self = self;
        if nStretchPercentX == 100 && nStretchPercentY == 100 && nSkewDegX == 0 && nSkewDegY == 0 {
            return;
        }
    
        (*this_).TakeOff();
    
        let oldWidth: i32 = (*this_).m_rc.Width();
        let oldHeight: i32 = (*this_).m_rc.Height();
        let newWidth: i32 = oldWidth * nStretchPercentX / 100;
        let newHeight: i32 = oldHeight * nStretchPercentY / 100;
    
        let mut hbmColor: HBITMAP = (*this_).m_hbmColor;
        let mut hbmMask: HBITMAP = (*this_).m_hbmMask;
    
        if hbmMask.is_null() {
            hbmMask = CreateMonoBitmap(oldWidth, oldHeight, (TRUE) as BOOL);
        }
    
        if oldWidth != newWidth || oldHeight != newHeight {
            AttachHBITMAP(core::ptr::addr_of_mut!(hbmColor), CopyDIBImage(hbmColor, newWidth, newHeight, (STRETCH_HALFTONE as i32)));
            AttachHBITMAP(core::ptr::addr_of_mut!(hbmMask), CopyMonoImage(hbmMask, newWidth, newHeight, (STRETCH_HALFTONE) as INT));
        }
    
        let mut hbmOld: HGDIOBJ = core::ptr::null_mut();
        let hDC: HDC = CreateCompatibleDC(core::ptr::null_mut());
    
        if nSkewDegX != 0 {
            hbmOld = SelectObject(hDC, (hbmColor).cast());
            AttachHBITMAP(core::ptr::addr_of_mut!(hbmColor), SkewDIB(hDC, hbmColor, nSkewDegX, (FALSE) as BOOL, (FALSE) as BOOL));
            SelectObject(hDC, (hbmMask).cast());
            AttachHBITMAP(core::ptr::addr_of_mut!(hbmMask), SkewDIB(hDC, hbmMask, nSkewDegX, (FALSE) as BOOL, (TRUE) as BOOL));
            SelectObject(hDC, hbmOld);
        }

        if nSkewDegY != 0 {
            hbmOld = SelectObject(hDC, (hbmColor).cast());
            AttachHBITMAP(core::ptr::addr_of_mut!(hbmColor), SkewDIB(hDC, hbmColor, nSkewDegY, (TRUE) as BOOL, (FALSE) as BOOL));
            SelectObject(hDC, (hbmMask).cast());
            AttachHBITMAP(core::ptr::addr_of_mut!(hbmMask), SkewDIB(hDC, hbmMask, nSkewDegY, (TRUE) as BOOL, (TRUE) as BOOL));
            SelectObject(hDC, hbmOld);
        }
    
        DeleteDC(hDC);
    
        (*this_).InsertFromHBITMAP(hbmColor, (*this_).m_rc.left, (*this_).m_rc.top, hbmMask);
    
        (*this_).m_bShow = (TRUE) as i32;
        (*this_).NotifyContentChanged();
    }
    pub unsafe fn InvertSelection(&mut self) {
        let this_: *mut Self = self;
        (*this_).TakeOff();
    
        let mut bm: BITMAP = core::mem::zeroed();
        GetObjectW(((*this_).m_hbmColor).cast(), core::mem::size_of::<BITMAP>() as i32, &mut bm as *mut BITMAP as *mut core::ffi::c_void);
    
        let hdc: HDC = CreateCompatibleDC(core::ptr::null_mut());
        let hbmOld: HGDIOBJ = SelectObject(hdc, ((*this_).m_hbmColor).cast());
        let mut rc: RECT = RECT {
            left: 0,
            top: 0,
            right: bm.bmWidth,
            bottom: bm.bmHeight,
        };
        InvertRect(hdc, &mut rc as *mut RECT);
        SelectObject(hdc, hbmOld);
        DeleteDC(hdc);
    
        (*this_).NotifyContentChanged();
    }
    pub unsafe fn Dragging(&mut self, hit: HITTEST, pt: POINT) {
        let this_: *mut Self = self;
        if hit == HIT_NONE {
            // no-op
        } else if hit == HIT_UPPER_LEFT {
            (*this_).m_rc.left += pt.x - (*this_).m_ptHit.x;
            (*this_).m_rc.top += pt.y - (*this_).m_ptHit.y;
        } else if hit == HIT_UPPER_CENTER {
            (*this_).m_rc.top += pt.y - (*this_).m_ptHit.y;
        } else if hit == HIT_UPPER_RIGHT {
            (*this_).m_rc.right += pt.x - (*this_).m_ptHit.x;
            (*this_).m_rc.top += pt.y - (*this_).m_ptHit.y;
        } else if hit == HIT_MIDDLE_LEFT {
            (*this_).m_rc.left += pt.x - (*this_).m_ptHit.x;
        } else if hit == HIT_MIDDLE_RIGHT {
            (*this_).m_rc.right += pt.x - (*this_).m_ptHit.x;
        } else if hit == HIT_LOWER_LEFT {
            (*this_).m_rc.left += pt.x - (*this_).m_ptHit.x;
            (*this_).m_rc.bottom += pt.y - (*this_).m_ptHit.y;
        } else if hit == HIT_LOWER_CENTER {
            (*this_).m_rc.bottom += pt.y - (*this_).m_ptHit.y;
        } else if hit == HIT_LOWER_RIGHT {
            (*this_).m_rc.right += pt.x - (*this_).m_ptHit.x;
            (*this_).m_rc.bottom += pt.y - (*this_).m_ptHit.y;
        } else if hit == HIT_BORDER || hit == HIT_INNER {
            (*this_).m_rc.OffsetRect__0(pt.x - (*this_).m_ptHit.x, pt.y - (*this_).m_ptHit.y);
        }
        (*this_).m_ptHit = pt;
    }
    pub unsafe fn ClearMaskImage(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_hbmMask.is_null() {
            DeleteObject(((*this_).m_hbmMask).cast());
            (*this_).m_hbmMask = core::ptr::null_mut();
        }
    }
    pub unsafe fn ClearColorImage(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_hbmColor.is_null() {
            DeleteObject(((*this_).m_hbmColor).cast());
            (*this_).m_hbmColor = core::ptr::null_mut();
        }
    }
    pub unsafe fn NotifyContentChanged(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_bContentChanged = (TRUE) as i32;
        imageModel.NotifyImageChanged();
    }
    pub unsafe fn StretchSelection(&mut self, bShrink: BOOL) {
        let this_: *mut Self = self;
        if (*this_).m_bShow == 0 {
            return;
        }
    
        (*this_).TakeOff();
    
        let cx: INT = (*this_).m_rc.Width();
        let cy: INT = (*this_).m_rc.Height();
    
        if bShrink != 0 {
            (*this_).m_rc.InflateRect__0(-cx / 4, -cy / 4);
        } else {
            (*this_).m_rc.InflateRect__0(cx / 2, cy / 2);
        }
    
        // The selection area must exist there
        if (*this_).m_rc.Width() <= 0 {
            (*this_).m_rc.right = (*this_).m_rc.left + 1;
        }
        if (*this_).m_rc.Height() <= 0 {
            (*this_).m_rc.bottom = (*this_).m_rc.top + 1;
        }
    
        imageModel.NotifyImageChanged();
    }
    pub unsafe fn SwapWidthAndHeight(&mut self) {
        let this_: *mut Self = self;
        let cx: INT = (*this_).m_rc.Width();
        let cy: INT = (*this_).m_rc.Height();
        (*this_).m_rc.right = (*this_).m_rc.left + cy;
        (*this_).m_rc.bottom = (*this_).m_rc.top + cx;
    }
}
