/* bitblt.split2.c — #include fragment of bitblt.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original bitblt.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


BOOL FASTCALL
IntGdiPolyPatBlt(
    HDC hDC,
    DWORD dwRop,
    PPATRECT pRects,
    INT cRects,
    ULONG Reserved)
{
    INT i;
    PBRUSH pbrush;
    PDC pdc;
    EBRUSHOBJ eboFill;

    pdc = DC_LockDc(hDC);
    if (!pdc)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    if (pdc->dctype == DCTYPE_INFO)
    {
        DC_UnlockDc(pdc);
        /* Yes, Windows really returns TRUE in this case */
        return TRUE;
    }

    for (i = 0; i < cRects; i++)
    {
        pbrush = BRUSH_ShareLockBrush(pRects->hBrush);

        /* Check if we could lock the brush */
        if (pbrush != NULL)
        {
            /* Initialize a brush object */
            EBRUSHOBJ_vInitFromDC(&eboFill, pbrush, pdc);

            IntPatBlt(
                pdc,
                pRects->r.left,
                pRects->r.top,
                pRects->r.right,
                pRects->r.bottom,
                dwRop,
                &eboFill);

            /* Cleanup the brush object and unlock the brush */
            EBRUSHOBJ_vCleanup(&eboFill);
            BRUSH_ShareUnlockBrush(pbrush);
        }
        pRects++;
    }

    DC_UnlockDc(pdc);

    return TRUE;
}

BOOL
APIENTRY
NtGdiPatBlt(
    _In_ HDC hdcDest,
    _In_ INT x,
    _In_ INT y,
    _In_ INT cx,
    _In_ INT cy,
    _In_ DWORD dwRop)
{
    BOOL bResult;
    PDC pdc;

    /* Convert the ROP3 to a ROP4 */
    dwRop = MAKEROP4(dwRop & 0xFF0000, dwRop);

    /* Check if the rop uses a source */
    if (WIN32_ROP4_USES_SOURCE(dwRop))
    {
        /* This is not possible */
        return FALSE;
    }

    /* Lock the DC */
    pdc = DC_LockDc(hdcDest);
    if (pdc == NULL)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    /* Check if the DC has no surface (empty mem or info DC) */
    if (pdc->dclevel.pSurface == NULL)
    {
        /* Nothing to do, Windows returns TRUE! */
        DC_UnlockDc(pdc);
        return TRUE;
    }

    /* Update the fill brush, if necessary */
    if (pdc->pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
        DC_vUpdateFillBrush(pdc);

    /* Call the internal function */
    bResult = IntPatBlt(pdc, x, y, cx, cy, dwRop, &pdc->eboFill);

    /* Unlock the DC and return the result */
    DC_UnlockDc(pdc);
    return bResult;
}

BOOL
APIENTRY
NtGdiPolyPatBlt(
    HDC hDC,
    DWORD dwRop,
    IN PPOLYPATBLT pRects,
    IN DWORD cRects,
    IN DWORD Mode)
{
    PPATRECT rb = NULL;
    NTSTATUS Status = STATUS_SUCCESS;
    BOOL Ret;

    if (cRects > 0)
    {
        rb = ExAllocatePoolWithTag(PagedPool, sizeof(PATRECT) * cRects, GDITAG_PLGBLT_DATA);
        if (!rb)
        {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return FALSE;
        }
        Status = SafeProbeForRead(pRects, cRects * sizeof(PATRECT), 1);
        if (NT_SUCCESS(Status))
        {
            RtlCopyMemory(rb,
                pRects,
                cRects * sizeof(PATRECT));
        }

        if (!NT_SUCCESS(Status))
        {
            ExFreePoolWithTag(rb, GDITAG_PLGBLT_DATA);
            SetLastNtError(Status);
            return FALSE;
        }
    }

    Ret = IntGdiPolyPatBlt(hDC, dwRop, rb, cRects, Mode);

    if (cRects > 0)
        ExFreePoolWithTag(rb, GDITAG_PLGBLT_DATA);

    return Ret;
}

static
BOOL
FASTCALL
REGION_LPTODP(
    _In_ PDC pdc,
    _Inout_ PREGION prgnDest,
    _In_ PREGION prgnSrc)
{
    if (IntGdiCombineRgn(prgnDest, prgnSrc, NULL, RGN_COPY) == ERROR)
        return FALSE;

    return REGION_bXformRgn(prgnDest, DC_pmxWorldToDevice(pdc));
}

BOOL
APIENTRY
IntGdiBitBltRgn(
    _In_ PDC pdc,
    _In_ PREGION prgn,
    _In_opt_ BRUSHOBJ *pbo,
    _In_opt_ POINTL *pptlBrush,
    _In_ ROP4 rop4)
{
    PREGION prgnClip;
    XCLIPOBJ xcoClip;
    BOOL bResult;
    NT_ASSERT((pdc != NULL) && (prgn != NULL));

    /* Check if we have a surface */
    if (pdc->dclevel.pSurface == NULL)
    {
        return TRUE;
    }

    /* Create an empty clip region */
    prgnClip = IntSysCreateRectpRgn(0, 0, 0, 0);
    if (prgnClip == NULL)
    {
        return FALSE;
    }

    /* Transform given region into device coordinates */
    if (!REGION_LPTODP(pdc, prgnClip, prgn))
    {
        REGION_Delete(prgnClip);
        return FALSE;
    }

    /* Intersect with the system or RAO region (these are (atm) without DC-origin) */
    if (pdc->prgnRao)
        IntGdiCombineRgn(prgnClip, prgnClip, pdc->prgnRao, RGN_AND);
    else
        IntGdiCombineRgn(prgnClip, prgnClip, pdc->prgnVis, RGN_AND);

    /* Now account for the DC-origin */
    if (!REGION_bOffsetRgn(prgnClip, pdc->ptlDCOrig.x, pdc->ptlDCOrig.y))
    {
        REGION_Delete(prgnClip);
        return FALSE;
    }

    if (pdc->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
    {
        RECTL rcrgn;
        REGION_GetRgnBox(prgnClip, &rcrgn);
        IntUpdateBoundsRect(pdc, &rcrgn);
    }

    /* Prepare the DC */
    DC_vPrepareDCsForBlit(pdc, &prgnClip->rdh.rcBound, NULL, NULL);

    /* Initialize a clip object */
    IntEngInitClipObj(&xcoClip);
    IntEngUpdateClipRegion(&xcoClip,
                           prgnClip->rdh.nCount,
                           prgnClip->Buffer,
                           &prgnClip->rdh.rcBound);

    /* Call the Eng or Drv function */
    bResult = IntEngBitBlt(&pdc->dclevel.pSurface->SurfObj,
                           NULL,
                           NULL,
                           (CLIPOBJ *)&xcoClip,
                           NULL,
                           &prgnClip->rdh.rcBound,
                           NULL,
                           NULL,
                           pbo,
                           pptlBrush,
                           rop4);

    /* Cleanup */
    DC_vFinishBlit(pdc, NULL);
    REGION_Delete(prgnClip);
    IntEngFreeClipResources(&xcoClip);

    /* Return the result */
    return bResult;
}

BOOL
IntGdiFillRgn(
    _In_ PDC pdc,
    _In_ PREGION prgn,
    _In_opt_ PBRUSH pbrFill)
{
    PREGION prgnClip;
    XCLIPOBJ xcoClip;
    EBRUSHOBJ eboFill;
    BRUSHOBJ *pbo;
    BOOL bRet;
    DWORD rop2Fg;
    MIX mix;
    NT_ASSERT((pdc != NULL) && (prgn != NULL));

    if (pdc->dclevel.pSurface == NULL)
    {
        return TRUE;
    }

    prgnClip = IntSysCreateRectpRgn(0, 0, 0, 0);
    if (prgnClip == NULL)
    {
        return FALSE;
    }

    /* Transform region into device coordinates */
    if (!REGION_LPTODP(pdc, prgnClip, prgn))
    {
        REGION_Delete(prgnClip);
        return FALSE;
    }

    /* Intersect with the system or RAO region (these are (atm) without DC-origin) */
    if (pdc->prgnRao)
        IntGdiCombineRgn(prgnClip, prgnClip, pdc->prgnRao, RGN_AND);
    else
        IntGdiCombineRgn(prgnClip, prgnClip, pdc->prgnVis, RGN_AND);

    /* Now account for the DC-origin */
    if (!REGION_bOffsetRgn(prgnClip, pdc->ptlDCOrig.x, pdc->ptlDCOrig.y))
    {
        REGION_Delete(prgnClip);
        return FALSE;
    }

    if (pdc->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
    {
        RECTL rcrgn;
        REGION_GetRgnBox(prgnClip, &rcrgn);
        IntUpdateBoundsRect(pdc, &rcrgn);
    }

    IntEngInitClipObj(&xcoClip);
    IntEngUpdateClipRegion(&xcoClip,
                           prgnClip->rdh.nCount,
                           prgnClip->Buffer,
                           &prgnClip->rdh.rcBound );

    /* Get the FG rop and create a MIX based on the BK mode */
    rop2Fg = FIXUP_ROP2(pdc->pdcattr->jROP2);
    mix = rop2Fg | (pdc->pdcattr->jBkMode == OPAQUE ? rop2Fg : R2_NOP) << 8;

    /* Prepare DC for blit */
    DC_vPrepareDCsForBlit(pdc, &prgnClip->rdh.rcBound, NULL, NULL);

    /* Check if we have a fill brush */
    if (pbrFill != NULL)
    {
        /* Initialize the brush object */
        /// \todo Check parameters
        EBRUSHOBJ_vInit(&eboFill, pbrFill, pdc->dclevel.pSurface, 0x00FFFFFF, 0, NULL);
        pbo = &eboFill.BrushObject;
    }
    else
    {
        /* Update the fill brush if needed */
        if (pdc->pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
            DC_vUpdateFillBrush(pdc);

        /* Use the DC brush object */
        pbo = &pdc->eboFill.BrushObject;
    }

    /* Call the internal function */
    bRet = IntEngPaint(&pdc->dclevel.pSurface->SurfObj,
                       (CLIPOBJ *)&xcoClip,
                       pbo,
                       &pdc->pdcattr->ptlBrushOrigin,
                       mix);

    DC_vFinishBlit(pdc, NULL);
    REGION_Delete(prgnClip);
    IntEngFreeClipResources(&xcoClip);

    // Fill the region
    return bRet;
}

BOOL
FASTCALL
IntGdiPaintRgn(
    _In_ PDC pdc,
    _In_ PREGION prgn)
{
    return IntGdiFillRgn(pdc, prgn, NULL);
}

BOOL
APIENTRY
NtGdiFillRgn(
    _In_ HDC hdc,
    _In_ HRGN hrgn,
    _In_ HBRUSH hbrush)
{
    PDC pdc;
    PREGION prgn;
    PBRUSH pbrFill;
    BOOL bResult;

    /* Lock the DC */
    pdc = DC_LockDc(hdc);
    if (pdc == NULL)
    {
        ERR("Failed to lock hdc %p\n", hdc);
        return FALSE;
    }

    /* Check if the DC has no surface (empty mem or info DC) */
    if (pdc->dclevel.pSurface == NULL)
    {
        DC_UnlockDc(pdc);
        return TRUE;
    }

    /* Lock the region */
    prgn = REGION_LockRgn(hrgn);
    if (prgn == NULL)
    {
        ERR("Failed to lock hrgn %p\n", hrgn);
        DC_UnlockDc(pdc);
        return FALSE;
    }

    /* Lock the brush */
    pbrFill = BRUSH_ShareLockBrush(hbrush);
    if (pbrFill == NULL)
    {
        ERR("Failed to lock hbrush %p\n", hbrush);
        REGION_UnlockRgn(prgn);
        DC_UnlockDc(pdc);
        return FALSE;
    }

    /* Call the internal function */
    bResult = IntGdiFillRgn(pdc, prgn, pbrFill);

    /* Cleanup locks */
    BRUSH_ShareUnlockBrush(pbrFill);
    REGION_UnlockRgn(prgn);
    DC_UnlockDc(pdc);

    return bResult;
}

BOOL
APIENTRY
NtGdiFrameRgn(
    _In_ HDC hdc,
    _In_ HRGN hrgn,
    _In_ HBRUSH hbrush,
    _In_ INT xWidth,
    _In_ INT yHeight)
{
    HRGN hrgnFrame;
    BOOL bResult;

    hrgnFrame = GreCreateFrameRgn(hrgn, xWidth, yHeight);
    if (hrgnFrame == NULL)
    {
        return FALSE;
    }

    bResult = NtGdiFillRgn(hdc, hrgnFrame, hbrush);

    GreDeleteObject(hrgnFrame);
    return bResult;
}

BOOL
APIENTRY
NtGdiInvertRgn(
    _In_ HDC hdc,
    _In_ HRGN hrgn)
{
    BOOL bResult;
    PDC pdc;
    PREGION prgn;

    /* Lock the DC */
    pdc = DC_LockDc(hdc);
    if (pdc == NULL)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    /* Check if the DC has no surface (empty mem or info DC) */
    if (pdc->dclevel.pSurface == NULL)
    {
        /* Nothing to do, Windows returns TRUE! */
        DC_UnlockDc(pdc);
        return TRUE;
    }

    /* Lock the region */
    prgn = REGION_LockRgn(hrgn);
    if (prgn == NULL)
    {
        DC_UnlockDc(pdc);
        return FALSE;
    }

    /* Call the internal function */
    bResult = IntGdiBitBltRgn(pdc,
                              prgn,
                              NULL, // pbo
                              NULL, // pptlBrush,
                              ROP4_DSTINVERT);

    /* Unlock the region and DC and return the result */
    REGION_UnlockRgn(prgn);
    DC_UnlockDc(pdc);
    return bResult;
}

COLORREF
APIENTRY
NtGdiSetPixel(
    _In_ HDC hdc,
    _In_ INT x,
    _In_ INT y,
    _In_ COLORREF crColor)
{
    PDC pdc;
    ULONG iOldColor, iSolidColor;
    BOOL bResult;
    PEBRUSHOBJ pebo;
    ULONG ulDirty;
    EXLATEOBJ exlo;

    /* Lock the DC */
    pdc = DC_LockDc(hdc);
    if (!pdc)
    {
        return CLR_INVALID;
    }

    /* Check if the DC has no surface (empty mem or info DC) */
    if (pdc->dclevel.pSurface == NULL)
    {
        /* Fail! */
        DC_UnlockDc(pdc);
        return CLR_INVALID;
    }

    if (pdc->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
    {
       RECTL rcDst;

       RECTL_vSetRect(&rcDst, x, y, x+1, y+1);

       IntLPtoDP(pdc, (LPPOINT)&rcDst, 2);

       rcDst.left   += pdc->ptlDCOrig.x;
       rcDst.top    += pdc->ptlDCOrig.y;
       rcDst.right  += pdc->ptlDCOrig.x;
       rcDst.bottom += pdc->ptlDCOrig.y;

       IntUpdateBoundsRect(pdc, &rcDst);
    }

    /* Translate the color to the target format */
    iSolidColor = TranslateCOLORREF(pdc, crColor);

    /* Use the DC's text brush, which is always a solid brush */
    pebo = &pdc->eboText;

    /* Save the old solid color and set the one for the pixel */
    iOldColor = EBRUSHOBJ_iSetSolidColor(pebo, iSolidColor);

    /* Save dirty flags and reset dirty text brush flag */
    ulDirty = pdc->pdcattr->ulDirty_;
    pdc->pdcattr->ulDirty_ &= ~DIRTY_TEXT;

    /* Call the internal function */
    bResult = IntPatBlt(pdc, x, y, 1, 1, PATCOPY, pebo);

    /* Restore old text brush color and dirty flags */
    EBRUSHOBJ_iSetSolidColor(pebo, iOldColor);
    pdc->pdcattr->ulDirty_ = ulDirty;

    /// FIXME: we shouldn't dereference pSurface while the PDEV is not locked!
    /* Initialize an XLATEOBJ from the target surface to RGB without using BkColor */
    EXLATEOBJ_vInitialize(&exlo,
                          pdc->dclevel.pSurface->ppal,
                          &gpalRGB,
                          CLR_INVALID,
                          CLR_INVALID,
                          CLR_INVALID);

    /* Translate the color back to RGB */
    crColor = XLATEOBJ_iXlate(&exlo.xlo, iSolidColor);

    /* Cleanup and return the target format color */
    EXLATEOBJ_vCleanup(&exlo);

    /* Unlock the DC */
    DC_UnlockDc(pdc);

    /* Return the new RGB color or CLR_INVALID (-1) on failure */
    return bResult ? crColor : CLR_INVALID;
}

COLORREF
APIENTRY
NtGdiGetPixel(
    _In_ HDC hdc,
    _In_ INT x,
    _In_ INT y)
{
    PDC pdc;
    ULONG ulRGBColor = CLR_INVALID;
    POINTL ptlSrc;
    RECT rcDest;
    PSURFACE psurfSrc, psurfDest;

    /* Lock the DC */
    pdc = DC_LockDc(hdc);
    if (!pdc)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID;
    }

    /* Check if the DC has no surface (empty mem or info DC) */
    if (pdc->dclevel.pSurface == NULL)
    {
        /* Fail! */
        goto leave;
    }

    /* Get the logical coordinates */
    ptlSrc.x = x;
    ptlSrc.y = y;

    /* Translate coordinates to device coordinates */
    IntLPtoDP(pdc, &ptlSrc, 1);
    ptlSrc.x += pdc->ptlDCOrig.x;
    ptlSrc.y += pdc->ptlDCOrig.y;

    rcDest.left = x;
    rcDest.top = y;
    rcDest.right = x + 1;
    rcDest.bottom = y + 1;

    /* Prepare DC for blit */
    DC_vPrepareDCsForBlit(pdc, &rcDest, NULL, NULL);

    /* Check if the pixel is outside the surface */
    psurfSrc = pdc->dclevel.pSurface;
    if ((ptlSrc.x >= psurfSrc->SurfObj.sizlBitmap.cx) ||
        (ptlSrc.y >= psurfSrc->SurfObj.sizlBitmap.cy) ||
        (ptlSrc.x < 0) ||
        (ptlSrc.y < 0))
    {
        /* Fail! */
        goto leave;
    }

    /* Allocate a surface */
    psurfDest = SURFACE_AllocSurface(STYPE_BITMAP,
                                     1,
                                     1,
                                     BMF_32BPP,
                                     0,
                                     0,
                                     0,
                                     &ulRGBColor);
    if (psurfDest)
    {
        RECTL rclDest = {0, 0, 1, 1};
        EXLATEOBJ exlo;

        /* Translate from the source palette to RGB color without BkColor */
        EXLATEOBJ_vInitialize(&exlo,
                              psurfSrc->ppal,
                              &gpalRGB,
                              CLR_INVALID,
                              CLR_INVALID,
                              CLR_INVALID);

        /* Call the copy bits function */
        EngCopyBits(&psurfDest->SurfObj,
                    &psurfSrc->SurfObj,
                    NULL,
                    &exlo.xlo,
                    &rclDest,
                    &ptlSrc);

        /* Cleanup the XLATEOBJ */
        EXLATEOBJ_vCleanup(&exlo);

        /* Delete the surface */
        GDIOBJ_vDeleteObject(&psurfDest->BaseObject);

        /* The top byte is zero */
        ulRGBColor &= 0x00FFFFFF;
    }

leave:

    /* Unlock the DC */
    DC_vFinishBlit(pdc, NULL);
    DC_UnlockDc(pdc);

    /* Return the new RGB color or -1 on failure */
    return ulRGBColor;
}

