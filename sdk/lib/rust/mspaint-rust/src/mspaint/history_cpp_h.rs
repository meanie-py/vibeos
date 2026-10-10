use crate::prelude::*;
use core::ffi::c_void;































impl IMAGE_PART {
    pub unsafe fn clear(&mut self) {
        let this_: *mut Self = self;
        DeleteObject(((*this_).m_hbmImage).cast());
        (*this_).m_hbmImage = core::ptr::null_mut();
        (*this_).m_rcPart.SetRectEmpty();
        (*this_).m_bPartial = (FALSE) as i32;
    }
}

impl ImageModel {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_hDrawingDC = CreateCompatibleDC(core::ptr::null_mut());
        (*this_).m_currInd = 0;
        (*this_).m_undoSteps = 0;
        (*this_).m_redoSteps = 0;
        {
        (*this_).m_hbmMaster = CreateColorDIB(1, 1, RGB!(255, 255, 255));
        (*this_).m_hbmOld = SelectObject((*this_).m_hDrawingDC, ((*this_).m_hbmMaster).cast());
    
        g_imageSaved = (TRUE) as i32;
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        SelectObject((*this_).m_hDrawingDC, (*this_).m_hbmOld); // De-select
        DeleteDC((*this_).m_hDrawingDC);
        DeleteObject(((*this_).m_hbmMaster).cast());
        (*this_).ClearHistory();
    }
    pub unsafe fn GetDC(&mut self) -> HDC {
        let this_: *mut Self = self;
        (*this_).m_hDrawingDC
    }
    pub unsafe fn PushImageForUndo__0(&mut self) {
        let this_: *mut Self = self;
        let hbm: HBITMAP = (*this_).CopyBitmap();
        if hbm == core::ptr::null_mut() {
            ShowOutOfMemory();
            (*this_).ClearHistory();
            return;
        }
    
        (*this_).PushImageForUndo__1(hbm);
    }
    pub unsafe fn PushImageForUndo__1(&mut self, hbm: HBITMAP) {
        let this_: *mut Self = self;
        ATLTRACE!(b"%s: %d\n\0".as_ptr() as *const i8, __FUNCTION__, (*this_).m_currInd);
    
        if hbm.is_null() {
            ShowOutOfMemory();
            (*this_).ClearHistory();
            return;
        }
    
        let part: *mut IMAGE_PART = core::ptr::addr_of_mut!((*this_).m_historyItems[(*this_).m_currInd as usize]);
        (*part).clear();
        (*part).m_hbmImage = (*this_).m_hbmMaster;
        (*this_).m_hbmMaster = hbm;
        SelectObject((*this_).m_hDrawingDC, ((*this_).m_hbmMaster).cast()); // Re-select
    
        (*this_).PushDone();
    }
    pub unsafe fn PushImageForUndo__2(&mut self, rcPartial: &RECT) {
        let this_: *mut Self = self;
        ATLTRACE!(b"%s: %d\n\0".as_ptr() as *const i8, __FUNCTION__, (*this_).m_currInd);
    
        let part: &mut IMAGE_PART = &mut (*this_).m_historyItems[(*this_).m_currInd as usize];
        (*part).clear();
        (*part).m_bPartial = (TRUE) as i32;
        (*part).m_rcPart.__base0 = *rcPartial;
    
        let rcImage: CRect = CRect__make_1(0, 0, (*this_).GetWidth(), (*this_).GetHeight());
        let rc: &mut CRect = &mut (*part).m_rcPart;
        if rc.IntersectRect(
            core::ptr::addr_of!(rc.__base0),
            core::ptr::addr_of!(rcImage.__base0),
        ) == 0
        {
            rc.SetRect(-1, -1, 0, 0);
        }
    
        let hbmMaster: HBITMAP = (*this_).LockBitmap();
        (*part).m_hbmImage = getSubImage(hbmMaster, &rc.__base0);
        (*this_).UnlockBitmap(hbmMaster);
    
        (*this_).PushDone();
    }
    pub unsafe fn Undo(&mut self, bClearRedo: BOOL) {
        let this_: *mut Self = self;
        ATLTRACE!(b"%s: %d\n\0".as_ptr() as *const i8, __FUNCTION__, (*this_).m_undoSteps);
        if (*this_).CanUndo() == 0 {
            return;
        }
    
        selectionModel.HideSelection();
    
        (*this_).m_currInd = ((*this_).m_currInd + (HISTORYSIZE as i32) - 1) % (HISTORYSIZE as i32); // Go previous
        ATLASSERT!((*this_).m_hbmMaster != core::ptr::null_mut());
        (*this_).SwapPart();
        SelectObject((*this_).m_hDrawingDC, ((*this_).m_hbmMaster).cast()); // Re-select
    
        (*this_).m_undoSteps -= 1;
        if bClearRedo != 0 {
            (*this_).m_redoSteps = 0;
        } else if (((*this_).m_redoSteps) as u32) < HISTORYSIZE - 1 {
            (*this_).m_redoSteps += 1;
        }
    
        (*this_).NotifyImageChanged();
    }
    pub unsafe fn Redo(&mut self) {
        let this_: *mut Self = self;
        ATLTRACE!(b"%s: %d\n\0".as_ptr() as *const i8, __FUNCTION__, (*this_).m_redoSteps);
        if (*this_).CanRedo() == 0 {
            return;
        }
    
        selectionModel.HideSelection();
    
        ATLASSERT!((*this_).m_hbmMaster != core::ptr::null_mut());
        (*this_).SwapPart();
        (*this_).m_currInd = ((*this_).m_currInd + 1) % (HISTORYSIZE as i32); // Go next
        SelectObject((*this_).m_hDrawingDC, ((*this_).m_hbmMaster).cast()); // Re-select
    
        (*this_).m_redoSteps -= 1;
        if (((*this_).m_undoSteps) as u32) < HISTORYSIZE - 1 {
            (*this_).m_undoSteps += 1;
        }
    
        (*this_).NotifyImageChanged();
    }
    pub unsafe fn ClearHistory(&mut self) {
        let this_: *mut Self = self;
        let mut i: i32 = 0;
        while i < (HISTORYSIZE as i32) {
            (*this_).m_historyItems[i as usize].clear();
            i += 1;
        }
    
        (*this_).m_undoSteps = 0;
        (*this_).m_redoSteps = 0;
    }
    pub unsafe fn Crop(&mut self, mut nWidth: i32, mut nHeight: i32, nOffsetX: i32, nOffsetY: i32) {
        let this_: *mut Self = self;
        // We cannot create bitmaps of size zero
        if nWidth <= 0 {
            nWidth = 1;
        }
        if nHeight <= 0 {
            nHeight = 1;
        }
    
        // Create a white HBITMAP
        let hbmNew: HBITMAP = CreateColorDIB(nWidth, nHeight, RGB!(255, 255, 255));
        if hbmNew.is_null() {
            ShowOutOfMemory();
            (*this_).ClearHistory();
            return;
        }
    
        // Put the master image as a sub-image
        let rcPart: RECT = RECT {
            left: -nOffsetX,
            top: -nOffsetY,
            right: (*this_).GetWidth() - nOffsetX,
            bottom: (*this_).GetHeight() - nOffsetY,
        };
        let hbmOld: HBITMAP = imageModel.LockBitmap();
        putSubImage(hbmNew, &rcPart, hbmOld);
        imageModel.UnlockBitmap(hbmOld);
    
        // Push it
        (*this_).PushImageForUndo__1(hbmNew);
    
        (*this_).NotifyImageChanged();
    }
    pub unsafe fn SaveImage(&mut self, lpFileName: LPCWSTR) {
        let this_: *mut Self = self;
        SaveDIBToFile((*this_).m_hbmMaster, lpFileName, (TRUE) as BOOL, GUID_NULL);
    }
    pub unsafe fn IsImageSaved(&mut self) -> i32 {
        let this_: *mut Self = self;
        return g_imageSaved;
    }
    pub unsafe fn StretchSkew(&mut self, nStretchPercentX: i32, nStretchPercentY: i32, nSkewDegX: i32, nSkewDegY: i32) {
        let this_: *mut Self = self;
        let oldWidth: i32 = (*this_).GetWidth();
        let oldHeight: i32 = (*this_).GetHeight();
        let newWidth: i32 = oldWidth * nStretchPercentX / 100;
        let newHeight: i32 = oldHeight * nStretchPercentY / 100;
        if oldWidth != newWidth || oldHeight != newHeight {
            let hbm0: HBITMAP = CopyDIBImage((*this_).m_hbmMaster, newWidth, newHeight, (STRETCH_HALFTONE as i32));
            (*this_).PushImageForUndo__1(hbm0);
        }
        if nSkewDegX != 0 {
            let hbm1: HBITMAP = SkewDIB((*this_).m_hDrawingDC, (*this_).m_hbmMaster, nSkewDegX, (FALSE) as BOOL, (FALSE) as BOOL);
            (*this_).PushImageForUndo__1(hbm1);
        }
        if nSkewDegY != 0 {
            let hbm2: HBITMAP = SkewDIB((*this_).m_hDrawingDC, (*this_).m_hbmMaster, nSkewDegY, (TRUE) as BOOL, (FALSE) as BOOL);
            (*this_).PushImageForUndo__1(hbm2);
        }
        (*this_).NotifyImageChanged();
    }
    pub unsafe fn GetWidth(&mut self) -> i32 {
        let this_: *mut Self = self;
        return GetDIBWidth((*this_).m_hbmMaster);
    }
    pub unsafe fn GetHeight(&mut self) -> i32 {
        let this_: *mut Self = self;
        return GetDIBHeight((*this_).m_hbmMaster);
    }
    pub unsafe fn CopyBitmap(&mut self) -> HBITMAP {
        let this_: *mut Self = self;
        let hBitmap: HBITMAP = (*this_).LockBitmap();
        let ret: HBITMAP = CopyDIBImage(hBitmap, 0, 0, (STRETCH_HALFTONE as i32));
        (*this_).UnlockBitmap(hBitmap);
        return ret;
    }
    pub unsafe fn LockBitmap(&mut self) -> HBITMAP {
        let this_: *mut Self = self;
        // NOTE: An app cannot select a bitmap into more than one device context at a time.
        SelectObject((*this_).m_hDrawingDC, (*this_).m_hbmOld); // De-select
        let mut hbmLocked: HBITMAP = (*this_).m_hbmMaster;
        (*this_).m_hbmMaster = core::ptr::null_mut();
        return hbmLocked;
    }
    pub unsafe fn UnlockBitmap(&mut self, hbmLocked: HBITMAP) {
        let this_: *mut Self = self;
        (*this_).m_hbmMaster = hbmLocked;
        (*this_).m_hbmOld = SelectObject((*this_).m_hDrawingDC, ((*this_).m_hbmMaster).cast()); // Re-select
    }
    pub unsafe fn InvertColors(&mut self) {
        let this_: *mut Self = self;
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: (*this_).GetWidth(),
            bottom: (*this_).GetHeight(),
        };
        (*this_).PushImageForUndo__0();
        InvertRect((*this_).m_hDrawingDC, core::ptr::addr_of_mut!(rect));
        (*this_).NotifyImageChanged();
    }
    pub unsafe fn FlipHorizontally(&mut self) {
        let this_: *mut Self = self;
        (*this_).PushImageForUndo__0();
        StretchBlt((*this_).m_hDrawingDC, (*this_).GetWidth() - 1, 0, -(*this_).GetWidth(), (*this_).GetHeight(), (*this_).GetDC(), 0, 0,
                   (*this_).GetWidth(), (*this_).GetHeight(), SRCCOPY);
        (*this_).NotifyImageChanged();
    }
    pub unsafe fn FlipVertically(&mut self) {
        let this_: *mut Self = self;
        (*this_).PushImageForUndo__0();
        StretchBlt((*this_).m_hDrawingDC, 0, (*this_).GetHeight() - 1, (*this_).GetWidth(), -(*this_).GetHeight(), (*this_).GetDC(), 0, 0, (*this_).GetWidth(), (*this_).GetHeight(), SRCCOPY);
        (*this_).NotifyImageChanged();
    }
    pub unsafe fn RotateNTimes90Degrees(&mut self, iN: i32) {
        let this_: *mut Self = self;
        match iN {
            1 | 3 => {
                let hbm: HBITMAP = Rotate90DegreeBlt((*this_).m_hDrawingDC, (*this_).GetWidth(), (*this_).GetHeight(), (iN == 1) as i32, (FALSE) as BOOL);
                (*this_).PushImageForUndo__1(hbm);
            }
            2 => {
                (*this_).PushImageForUndo__0();
                StretchBlt((*this_).m_hDrawingDC, (*this_).GetWidth() - 1, (*this_).GetHeight() - 1, -(*this_).GetWidth(), -(*this_).GetHeight(),
                    (*this_).m_hDrawingDC, 0, 0, (*this_).GetWidth(), (*this_).GetHeight(), SRCCOPY);
            }
            _ => {}
        }
        (*this_).NotifyImageChanged();
    }
    pub unsafe fn Clamp(&mut self, pt: &mut POINT) {
        let this_: *mut Self = self;
        pt.x = max(0, min(pt.x, (*(*(*(*this_).this_).this_).this_).GetWidth()));
        pt.y = max(0, min(pt.y, (*(*(*(*this_).this_).this_).this_).GetHeight()));
    }
    pub unsafe fn NotifyImageChanged(&mut self) {
        let this_: *mut Self = self;
        let _ = this_;
        if (canvasWindow.__base0.IsWindow()) != 0 {
            canvasWindow.updateScrollRange();
            canvasWindow.__base0.Invalidate((TRUE) as BOOL);
        }
    
        if (miniature.__base0.IsWindow()) != 0 {
            miniature.__base0.Invalidate((TRUE) as BOOL);
        }
    }
    pub unsafe fn IsBlackAndWhite(&mut self) -> BOOL {
        let this_: *mut Self = self;
        let hBitmap: HBITMAP = (*this_).LockBitmap();
        let bBlackAndWhite: BOOL = IsBitmapBlackAndWhite(hBitmap);
        (*this_).UnlockBitmap(hBitmap);
        return bBlackAndWhite;
    }
    pub unsafe fn PushBlackAndWhite(&mut self) {
        let this_: *mut Self = self;
        let hBitmap: HBITMAP = (*this_).LockBitmap();
        let hNewBitmap: HBITMAP = ConvertToBlackAndWhite(hBitmap);
        (*this_).UnlockBitmap(hBitmap);
    
        (*this_).PushImageForUndo__1(hNewBitmap);
    }
    pub unsafe fn SwapPart(&mut self) {
        let this_: *mut Self = self;
        let part: *mut IMAGE_PART = core::ptr::addr_of_mut!((*this_).m_historyItems[(*this_).m_currInd as usize]);
        if (*part).m_bPartial == 0 {
            Swap((*this_).m_hbmMaster, (*part).m_hbmImage);
            return;
        }
    
        let hbmMaster: HBITMAP = (*this_).LockBitmap();
        let hbmPart: HBITMAP = getSubImage(hbmMaster, &(*part).m_rcPart.__base0);
        putSubImage(hbmMaster, &(*part).m_rcPart.__base0, (*part).m_hbmImage);
        DeleteObject(((*part).m_hbmImage).cast());
        (*part).m_hbmImage = hbmPart;
        (*this_).UnlockBitmap(hbmMaster);
    }
    pub unsafe fn PushDone(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_currInd = ((*this_).m_currInd + 1) % (HISTORYSIZE as i32); // Go next
    
        if (((*this_).m_undoSteps) as u32) < HISTORYSIZE - 1 {
            (*this_).m_undoSteps += 1;
        }
        (*this_).m_redoSteps = 0;
    
        g_imageSaved = (FALSE) as i32;
        (*this_).NotifyImageChanged();
    }
}
