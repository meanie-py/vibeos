use win32k_sys::win32k::*;

// LFONT_ShareUnlockFont is a macro (text.h): GDIOBJ_vDereferenceObject((POBJ)plfnt)
#[inline(always)] unsafe fn LFONT_ShareUnlockFont(plfnt: PLFONT) { GDIOBJ_vDereferenceObject(plfnt as POBJ); }
#[inline(always)] unsafe fn SURFACE_ShareUnlockSurface(p: PSURFACE) { GDIOBJ_vDereferenceObject(p as POBJ); }
#[inline(always)] unsafe fn PALETTE_ShareUnlockPalette(p: PPALETTE) { GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*p).BaseObject)); }
// DC_vSelect*/BRUSH_ShareUnlockBrush are FORCEINLINE (dc.h/brush.h) — inline locally.
#[inline(always)] unsafe fn BRUSH_ShareUnlockBrush(pbr: PBRUSH) { GDIOBJ_vDereferenceObject(pbr as POBJ); }
#[inline(always)] unsafe fn DC_vSelectSurface(pdc: PDC, psurfNew: PSURFACE) {
    let psurfOld = (*pdc).dclevel.pSurface;
    if !psurfOld.is_null() { (*psurfOld).hdc = core::ptr::null_mut(); SURFACE_ShareUnlockSurface(psurfOld); }
    if !psurfNew.is_null() { GDIOBJ_vReferenceObjectByPointer(psurfNew as POBJ); }
    (*pdc).dclevel.pSurface = psurfNew;
}
#[inline(always)] unsafe fn DC_vSelectFillBrush(pdc: PDC, pbrFill: PBRUSH) {
    let o = (*pdc).dclevel.pbrFill;
    if !o.is_null() { BRUSH_ShareUnlockBrush(o); }
    if !pbrFill.is_null() { GDIOBJ_vReferenceObjectByPointer(pbrFill as POBJ); }
    (*pdc).dclevel.pbrFill = pbrFill;
}
#[inline(always)] unsafe fn DC_vSelectLineBrush(pdc: PDC, pbrLine: PBRUSH) {
    let o = (*pdc).dclevel.pbrLine;
    if !o.is_null() { BRUSH_ShareUnlockBrush(o); }
    if !pbrLine.is_null() { GDIOBJ_vReferenceObjectByPointer(pbrLine as POBJ); }
    (*pdc).dclevel.pbrLine = pbrLine;
}
#[inline(always)] unsafe fn DC_vSelectPalette(pdc: PDC, ppal: PPALETTE) {
    let o = (*pdc).dclevel.ppal;
    if !o.is_null() { PALETTE_ShareUnlockPalette(o); }
    if !ppal.is_null() { GDIOBJ_vReferenceObjectByPointer(ppal as POBJ); }
    (*pdc).dclevel.ppal = ppal;
}

// ---- file-scope preamble (translated) ----

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn DC_vCopyState(pdcSrc: PDC, pdcDst: PDC, To: BOOL) {
    *(*pdcDst).pdcattr = *(*pdcSrc).pdcattr;

    (*(*pdcDst).pdcattr).ulDirty_ |= (DIRTY_FILL | DIRTY_LINE | DIRTY_TEXT | DIRTY_BACKGROUND | DIRTY_CHARSET | DC_ICM_NOT_CALIBRATED | DC_ICM_NOT_SET) as u32;

    (*pdcDst).dclevel.pColorSpace = (*pdcSrc).dclevel.pColorSpace;
    (*pdcDst).dclevel.laPath = (*pdcSrc).dclevel.laPath;
    (*pdcDst).dclevel.ca = (*pdcSrc).dclevel.ca;
    (*pdcDst).dclevel.mxWorldToDevice = (*pdcSrc).dclevel.mxWorldToDevice;
    (*pdcDst).dclevel.mxDeviceToWorld = (*pdcSrc).dclevel.mxDeviceToWorld;
    (*pdcDst).dclevel.mxWorldToPage = (*pdcSrc).dclevel.mxWorldToPage;
    (*pdcDst).dclevel.efM11PtoD = (*pdcSrc).dclevel.efM11PtoD;
    (*pdcDst).dclevel.efM22PtoD = (*pdcSrc).dclevel.efM22PtoD;
    (*pdcDst).dclevel.sizl = (*pdcSrc).dclevel.sizl;
    (*pdcDst).dclevel.hpal = (*pdcSrc).dclevel.hpal;

    DC_vSelectFillBrush(pdcDst, (*pdcSrc).dclevel.pbrFill);
    DC_vSelectLineBrush(pdcDst, (*pdcSrc).dclevel.pbrLine);
    DC_vSelectPalette(pdcDst, (*pdcSrc).dclevel.ppal);

    if !(*pdcDst).dclevel.plfnt.is_null() {
        LFONT_ShareUnlockFont((*pdcDst).dclevel.plfnt);
    }
    GDIOBJ_vReferenceObjectByPointer(core::ptr::addr_of_mut!((*(*pdcSrc).dclevel.plfnt).BaseObject));
    (*pdcDst).dclevel.plfnt = (*pdcSrc).dclevel.plfnt;

    if To == 0 {
        IntGdiExtSelectClipRgn(pdcDst, (*pdcSrc).dclevel.prgnClip, RGN_COPY as INT);
        if !(*pdcDst).dclevel.prgnMeta.is_null() {
            REGION_Delete((*pdcDst).dclevel.prgnMeta);
            (*pdcDst).dclevel.prgnMeta = core::ptr::null_mut();
        }
        if !(*pdcSrc).dclevel.prgnMeta.is_null() {
            (*pdcDst).dclevel.prgnMeta = IntSysCreateRectpRgn(0, 0, 0, 0);
            IntGdiCombineRgn((*pdcDst).dclevel.prgnMeta, (*pdcSrc).dclevel.prgnMeta, core::ptr::null_mut(), RGN_COPY as INT);
        }
        (*pdcDst).fs |= DC_DIRTY_RAO as i32;
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn DC_vCopyState(pdcSrc: PDC, pdcDst: PDC, To: BOOL) {
    *(*pdcDst).pdcattr = *(*pdcSrc).pdcattr;

    (*(*pdcDst).pdcattr).ulDirty_ |= (DIRTY_FILL | DIRTY_LINE | DIRTY_TEXT | DIRTY_BACKGROUND | DIRTY_CHARSET | DC_ICM_NOT_CALIBRATED | DC_ICM_NOT_SET) as u32;

    (*pdcDst).dclevel.pColorSpace = (*pdcSrc).dclevel.pColorSpace;
    (*pdcDst).dclevel.laPath = (*pdcSrc).dclevel.laPath;
    (*pdcDst).dclevel.ca = (*pdcSrc).dclevel.ca;
    (*pdcDst).dclevel.mxWorldToDevice = (*pdcSrc).dclevel.mxWorldToDevice;
    (*pdcDst).dclevel.mxDeviceToWorld = (*pdcSrc).dclevel.mxDeviceToWorld;
    (*pdcDst).dclevel.mxWorldToPage = (*pdcSrc).dclevel.mxWorldToPage;
    (*pdcDst).dclevel.efM11PtoD = (*pdcSrc).dclevel.efM11PtoD;
    (*pdcDst).dclevel.efM22PtoD = (*pdcSrc).dclevel.efM22PtoD;
    (*pdcDst).dclevel.sizl = (*pdcSrc).dclevel.sizl;
    (*pdcDst).dclevel.hpal = (*pdcSrc).dclevel.hpal;

    DC_vSelectFillBrush(pdcDst, (*pdcSrc).dclevel.pbrFill);
    DC_vSelectLineBrush(pdcDst, (*pdcSrc).dclevel.pbrLine);
    DC_vSelectPalette(pdcDst, (*pdcSrc).dclevel.ppal);

    if !(*pdcDst).dclevel.plfnt.is_null() {
        LFONT_ShareUnlockFont((*pdcDst).dclevel.plfnt);
    }
    GDIOBJ_vReferenceObjectByPointer(core::ptr::addr_of_mut!((*(*pdcSrc).dclevel.plfnt).BaseObject));
    (*pdcDst).dclevel.plfnt = (*pdcSrc).dclevel.plfnt;

    if To == 0 {
        IntGdiExtSelectClipRgn(pdcDst, (*pdcSrc).dclevel.prgnClip, RGN_COPY as INT);
        if !(*pdcDst).dclevel.prgnMeta.is_null() {
            REGION_Delete((*pdcDst).dclevel.prgnMeta);
            (*pdcDst).dclevel.prgnMeta = core::ptr::null_mut();
        }
        if !(*pdcSrc).dclevel.prgnMeta.is_null() {
            (*pdcDst).dclevel.prgnMeta = IntSysCreateRectpRgn(0, 0, 0, 0);
            IntGdiCombineRgn((*pdcDst).dclevel.prgnMeta, (*pdcSrc).dclevel.prgnMeta, core::ptr::null_mut(), RGN_COPY as INT);
        }
        (*pdcDst).fs |= DC_DIRTY_RAO as i32;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn IntGdiCleanDC(hDC: HDC) -> BOOL {
    let dc: PDC;
    if hDC.is_null() {
        return 0;
    }
    dc = DC_LockDc(hDC);
    if dc.is_null() {
        return 0;
    }
    
    if !defaultDCstate.is_null() {
        DC_vCopyState(defaultDCstate, dc, 0);
        DC_vUpdateFillBrush(dc);
        DC_vUpdateBackgroundBrush(dc);
        DC_vUpdateLineBrush(dc);
        DC_vUpdateTextBrush(dc);
    }

    if !(*dc).dclevel.hPath.is_null() {
        if PATH_Delete((*dc).dclevel.hPath) == 0 {
        }
        (*dc).dclevel.hPath = core::ptr::null_mut();
        (*dc).dclevel.flPath = 0;
    }

    if !(*dc).prgnRao.is_null() {
        REGION_Delete((*dc).prgnRao);
    }
    if !(*dc).prgnAPI.is_null() {
        REGION_Delete((*dc).prgnAPI);
    }
    (*dc).prgnRao = core::ptr::null_mut();
    (*dc).prgnAPI = core::ptr::null_mut();

    (*dc).fs |= DC_DIRTY_RAO as i32;

    DC_UnlockDc(dc);

    1}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IntGdiCleanDC(hDC: HDC) -> BOOL {
    let dc: PDC;
    if hDC.is_null() {
        return 0;
    }
    dc = DC_LockDc(hDC);
    if dc.is_null() {
        return 0;
    }
    
    if !defaultDCstate.is_null() {
        DC_vCopyState(defaultDCstate, dc, 0);
        DC_vUpdateFillBrush(dc);
        DC_vUpdateBackgroundBrush(dc);
        DC_vUpdateLineBrush(dc);
        DC_vUpdateTextBrush(dc);
    }

    if !(*dc).dclevel.hPath.is_null() {
        if PATH_Delete((*dc).dclevel.hPath) == 0 {
        }
        (*dc).dclevel.hPath = core::ptr::null_mut();
        (*dc).dclevel.flPath = 0;
    }

    if !(*dc).prgnRao.is_null() {
        REGION_Delete((*dc).prgnRao);
    }
    if !(*dc).prgnAPI.is_null() {
        REGION_Delete((*dc).prgnAPI);
    }
    (*dc).prgnRao = core::ptr::null_mut();
    (*dc).prgnAPI = core::ptr::null_mut();

    (*dc).fs |= DC_DIRTY_RAO as i32;

    DC_UnlockDc(dc);

    1}
#[no_mangle]
pub unsafe extern "system" fn NtGdiResetDC(
    hdc: HDC,
    pdm: LPDEVMODEW,
    pbBanding: PBOOL,
    pDriverInfo2: *mut DRIVER_INFO_2W,
    ppUMdhpdev: *mut core::ffi::c_void,
) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn DC_vRestoreDC(
    pdc: PDC,
    iSaveLevel: INT) {
    let mut hdcSave: HDC;
    let mut pdcSave: PDC;

    while (*pdc).dclevel.lSaveDepth > iSaveLevel
    {
        hdcSave = (*pdc).dclevel.hdcSave as HDC;

        if GreSetObjectOwner(hdcSave as HGDIOBJ, GDI_OBJ_HMGR_POWNED) == 0
        {
            return;
        }

        pdcSave = DC_LockDc(hdcSave);
        if pdcSave.is_null()
        {
            return;
        }

        (*pdc).dclevel.hdcSave = (*pdcSave).dclevel.hdcSave;

        (*pdc).dclevel.lSaveDepth -= 1;

        if (*pdc).dclevel.lSaveDepth == iSaveLevel
        {
            DC_vCopyState(pdcSave, pdc, 0);

            if (*pdc).dctype == DCTYPE_MEMORY
            {
                DC_vSelectSurface(pdc, (*pdcSave).dclevel.pSurface);
            }

            if !(*pdcSave).dclevel.hPath.is_null()
            {
                PATH_RestorePath(pdc, pdcSave);
            }
        }

        (*pdcSave).dclevel.lSaveDepth = 1;

        DC_UnlockDc(pdcSave);
        GreDeleteObject(hdcSave as HGDIOBJ);
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiRestoreDC(
    hdc: HDC,
    iSaveLevel: INT) -> BOOL {
    let pdc: PDC;

    pdc = DC_LockDc(hdc);
    if pdc.is_null()
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    let mut iSaveLevel = iSaveLevel;
    if iSaveLevel < 0
    {
        iSaveLevel = (*pdc).dclevel.lSaveDepth + iSaveLevel;
    }

    if iSaveLevel <= 0 || iSaveLevel >= (*pdc).dclevel.lSaveDepth
    {
        DC_UnlockDc(pdc);
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    DC_vRestoreDC(pdc, iSaveLevel);

    DC_UnlockDc(pdc);

    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSaveDC(hDC: HDC) -> INT {
    let mut hdcSave: HDC;
    let mut pdc: PDC;
    let mut pdcSave: PDC;
    let mut lSaveDepth: INT;

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    pdcSave = DC_AllocDcWithHandle(GDILoObjType_LO_DC_TYPE);
    if pdcSave.is_null() {
        DC_UnlockDc(pdc);
        return 0;
    }
    hdcSave = (*pdcSave).BaseObject.hHmgr as HDC;

    PDEVOBJ_vReference((*pdc).ppdev);
    DC_vInitDc(pdcSave, DCTYPE_MEMORY, (*pdc).ppdev);

    GDIOBJ_vSetObjectOwner(core::ptr::addr_of_mut!((*pdcSave).BaseObject), GDI_OBJ_HMGR_PUBLIC);

    DC_vCopyState(pdc, pdcSave, 0);

    if (*pdc).dctype == DCTYPE_MEMORY {
        DC_vSelectSurface(pdcSave, (*pdc).dclevel.pSurface);
    }

    if (*pdc).dclevel.hPath != core::ptr::null_mut() {
        PATH_SavePath(pdcSave, pdc);
    }
    (*pdcSave).dclevel.flPath = (*pdc).dclevel.flPath | DCPATH_SAVESTATE;

    (*pdcSave).dclevel.hdcSave = (*pdc).dclevel.hdcSave;
    (*pdc).dclevel.hdcSave = hdcSave as HGDIOBJ;

    lSaveDepth = (*pdc).dclevel.lSaveDepth;
    (*pdc).dclevel.lSaveDepth += 1;

    DC_UnlockDc(pdcSave);
    DC_UnlockDc(pdc);

    lSaveDepth
}
