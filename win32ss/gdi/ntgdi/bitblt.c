/*
 * COPYRIGHT:        GNU GPL, See COPYING in the top level directory
 * PROJECT:          ReactOS kernel
 * PURPOSE:          Bit blit functions
 * FILE:             win32ss/gdi/ntgdi/bitblt.c
 * PROGRAMER:        Unknown
 */

#include <win32k.h>
#include <safe_user_io.h>
DBG_DEFAULT_CHANNEL(GdiBlt);

BOOL APIENTRY
NtGdiAlphaBlend(
    HDC hDCDest,
    LONG XOriginDest,
    LONG YOriginDest,
    LONG WidthDest,
    LONG HeightDest,
    HDC hDCSrc,
    LONG XOriginSrc,
    LONG YOriginSrc,
    LONG WidthSrc,
    LONG HeightSrc,
    BLENDFUNCTION BlendFunc,
    HANDLE hcmXform)
{
    PDC DCDest;
    PDC DCSrc;
    HDC ahDC[2];
    PGDIOBJ apObj[2];
    SURFACE *BitmapDest, *BitmapSrc;
    RECTL DestRect, SourceRect;
    BOOL bResult;
    EXLATEOBJ exlo;
    BLENDOBJ BlendObj;
    BlendObj.BlendFunction = BlendFunc;

    if (WidthDest < 0 || HeightDest < 0 || WidthSrc < 0 || HeightSrc < 0)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    if ((hDCDest == NULL) || (hDCSrc == NULL))
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    TRACE("Locking DCs\n");
    ahDC[0] = hDCDest;
    ahDC[1] = hDCSrc ;
    if (!GDIOBJ_bLockMultipleObjects(2, (HGDIOBJ*)ahDC, apObj, GDIObjType_DC_TYPE))
    {
        WARN("Invalid dc handle (dest=0x%p, src=0x%p) passed to NtGdiAlphaBlend\n", hDCDest, hDCSrc);
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }
    DCDest = apObj[0];
    DCSrc = apObj[1];

    if (DCSrc->dctype == DCTYPE_INFO || DCDest->dctype == DCTYPE_INFO)
    {
        GDIOBJ_vUnlockObject(&DCSrc->BaseObject);
        GDIOBJ_vUnlockObject(&DCDest->BaseObject);
        /* Yes, Windows really returns TRUE in this case */
        return TRUE;
    }

    DestRect.left   = XOriginDest;
    DestRect.top    = YOriginDest;
    DestRect.right  = XOriginDest + WidthDest;
    DestRect.bottom = YOriginDest + HeightDest;
    IntLPtoDP(DCDest, (LPPOINT)&DestRect, 2);

    DestRect.left   += DCDest->ptlDCOrig.x;
    DestRect.top    += DCDest->ptlDCOrig.y;
    DestRect.right  += DCDest->ptlDCOrig.x;
    DestRect.bottom += DCDest->ptlDCOrig.y;

    if (DCDest->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
    {
       IntUpdateBoundsRect(DCDest, &DestRect);
    }

    SourceRect.left   = XOriginSrc;
    SourceRect.top    = YOriginSrc;
    SourceRect.right  = XOriginSrc + WidthSrc;
    SourceRect.bottom = YOriginSrc + HeightSrc;
    IntLPtoDP(DCSrc, (LPPOINT)&SourceRect, 2);

    SourceRect.left   += DCSrc->ptlDCOrig.x;
    SourceRect.top    += DCSrc->ptlDCOrig.y;
    SourceRect.right  += DCSrc->ptlDCOrig.x;
    SourceRect.bottom += DCSrc->ptlDCOrig.y;

    if (!DestRect.right ||
        !DestRect.bottom ||
        !SourceRect.right ||
        !SourceRect.bottom)
    {
        GDIOBJ_vUnlockObject(&DCSrc->BaseObject);
        GDIOBJ_vUnlockObject(&DCDest->BaseObject);
        return TRUE;
    }

    /* Prepare DCs for blit */
    TRACE("Preparing DCs for blit\n");
    DC_vPrepareDCsForBlit(DCDest, &DestRect, DCSrc, &SourceRect);

    /* Determine surfaces to be used in the bitblt */
    BitmapDest = DCDest->dclevel.pSurface;
    if (!BitmapDest)
    {
        bResult = FALSE ;
        goto leave ;
    }

    BitmapSrc = DCSrc->dclevel.pSurface;
    if (!BitmapSrc)
    {
        bResult = FALSE;
        goto leave;
    }

    /* Create the XLATEOBJ. */
    EXLATEOBJ_vInitXlateFromDCs(&exlo, DCSrc, DCDest);

    /* Perform the alpha blend operation */
    TRACE("Performing the alpha blend\n");
    bResult = IntEngAlphaBlend(&BitmapDest->SurfObj,
                               &BitmapSrc->SurfObj,
                               (CLIPOBJ *)&DCDest->co,
                               &exlo.xlo,
                               &DestRect,
                               &SourceRect,
                               &BlendObj);

    EXLATEOBJ_vCleanup(&exlo);
leave :
    TRACE("Finishing blit\n");
    DC_vFinishBlit(DCDest, DCSrc);
    GDIOBJ_vUnlockObject(&DCSrc->BaseObject);
    GDIOBJ_vUnlockObject(&DCDest->BaseObject);

    return bResult;
}

BOOL APIENTRY
NtGdiBitBlt(
    HDC hDCDest,
    INT XDest,
    INT YDest,
    INT Width,
    INT Height,
    HDC hDCSrc,
    INT XSrc,
    INT YSrc,
    DWORD dwRop,
    IN DWORD crBackColor,
    IN FLONG fl)
{

    if (dwRop & CAPTUREBLT)
    {
       return NtGdiStretchBlt(hDCDest,
                              XDest,
                              YDest,
                              Width,
                              Height,
                              hDCSrc,
                              XSrc,
                              YSrc,
                              Width,
                              Height,
                              dwRop,
                              crBackColor);
    }

    dwRop = dwRop & ~(NOMIRRORBITMAP|CAPTUREBLT);

    /* Forward to NtGdiMaskBlt */
    // TODO: What's fl for? LOL not to send this to MaskBit!
    return NtGdiMaskBlt(hDCDest,
                        XDest,
                        YDest,
                        Width,
                        Height,
                        hDCSrc,
                        XSrc,
                        YSrc,
                        NULL,
                        0,
                        0,
                        MAKEROP4(dwRop, dwRop),
                        crBackColor);
}

BOOL APIENTRY
NtGdiTransparentBlt(
    HDC hdcDst,
    INT xDst,
    INT yDst,
    INT cxDst,
    INT cyDst,
    HDC hdcSrc,
    INT xSrc,
    INT ySrc,
    INT cxSrc,
    INT cySrc,
    COLORREF TransColor)
{
    PDC DCDest, DCSrc;
    HDC ahDC[2];
    PGDIOBJ apObj[2];
    RECTL rcDest, rcSrc;
    SURFACE *BitmapDest, *BitmapSrc = NULL;
    ULONG TransparentColor = 0;
    BOOL Ret = FALSE;
    EXLATEOBJ exlo;

    if ((hdcDst == NULL) || (hdcSrc == NULL))
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    TRACE("Locking DCs\n");
    ahDC[0] = hdcDst;
    ahDC[1] = hdcSrc ;
    if (!GDIOBJ_bLockMultipleObjects(2, (HGDIOBJ*)ahDC, apObj, GDIObjType_DC_TYPE))
    {
        WARN("Invalid dc handle (dest=0x%p, src=0x%p) passed to NtGdiAlphaBlend\n", hdcDst, hdcSrc);
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }
    DCDest = apObj[0];
    DCSrc = apObj[1];

    if (DCSrc->dctype == DCTYPE_INFO || DCDest->dctype == DCTYPE_INFO)
    {
        GDIOBJ_vUnlockObject(&DCSrc->BaseObject);
        GDIOBJ_vUnlockObject(&DCDest->BaseObject);
        /* Yes, Windows really returns TRUE in this case */
        return TRUE;
    }

    rcDest.left   = xDst;
    rcDest.top    = yDst;
    rcDest.right  = rcDest.left + cxDst;
    rcDest.bottom = rcDest.top + cyDst;
    IntLPtoDP(DCDest, (LPPOINT)&rcDest, 2);

    rcDest.left   += DCDest->ptlDCOrig.x;
    rcDest.top    += DCDest->ptlDCOrig.y;
    rcDest.right  += DCDest->ptlDCOrig.x;
    rcDest.bottom += DCDest->ptlDCOrig.y;

    rcSrc.left   = xSrc;
    rcSrc.top    = ySrc;
    rcSrc.right  = rcSrc.left + cxSrc;
    rcSrc.bottom = rcSrc.top + cySrc;
    IntLPtoDP(DCSrc, (LPPOINT)&rcSrc, 2);

    rcSrc.left   += DCSrc->ptlDCOrig.x;
    rcSrc.top    += DCSrc->ptlDCOrig.y;
    rcSrc.right  += DCSrc->ptlDCOrig.x;
    rcSrc.bottom += DCSrc->ptlDCOrig.y;

    if (DCDest->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
    {
       IntUpdateBoundsRect(DCDest, &rcDest);
    }

    /* Prepare for blit */
    DC_vPrepareDCsForBlit(DCDest, &rcDest, DCSrc, &rcSrc);

    BitmapDest = DCDest->dclevel.pSurface;
    if (!BitmapDest)
    {
        goto done;
    }

    BitmapSrc = DCSrc->dclevel.pSurface;
    if (!BitmapSrc)
    {
        goto done;
    }

    /* Translate Transparent (RGB) Color to the source palette */
    EXLATEOBJ_vInitialize(&exlo, &gpalRGB, BitmapSrc->ppal, 0, 0, 0);
    TransparentColor = XLATEOBJ_iXlate(&exlo.xlo, (ULONG)TransColor);
    EXLATEOBJ_vCleanup(&exlo);

    EXLATEOBJ_vInitXlateFromDCs(&exlo, DCSrc, DCDest);

    Ret = IntEngTransparentBlt(&BitmapDest->SurfObj, &BitmapSrc->SurfObj,
        (CLIPOBJ *)&DCDest->co, &exlo.xlo, &rcDest, &rcSrc,
        TransparentColor, 0);

    EXLATEOBJ_vCleanup(&exlo);

done:
    DC_vFinishBlit(DCDest, DCSrc);
    GDIOBJ_vUnlockObject(&DCDest->BaseObject);
    GDIOBJ_vUnlockObject(&DCSrc->BaseObject);

    return Ret;
}

BOOL APIENTRY
NtGdiMaskBlt(
    HDC hdcDest,
    INT nXDest,
    INT nYDest,
    INT nWidth,
    INT nHeight,
    HDC hdcSrc,
    INT nXSrc,
    INT nYSrc,
    HBITMAP hbmMask,
    INT xMask,
    INT yMask,
    DWORD dwRop4,
    IN DWORD crBackColor)
{
    PDC DCDest;
    PDC DCSrc = NULL;
    HDC ahDC[2];
    PGDIOBJ apObj[2];
    PDC_ATTR pdcattr = NULL;
    SURFACE *BitmapDest, *BitmapSrc = NULL, *psurfMask = NULL;
    RECTL DestRect, SourceRect;
    POINTL SourcePoint, MaskPoint;
    BOOL Status = FALSE;
    EXLATEOBJ exlo;
    XLATEOBJ *XlateObj = NULL;
    BOOL UsesSource, UsesPattern;
    ROP4 rop4;

    rop4 = WIN32_ROP4_TO_ENG_ROP4(dwRop4);

    if (!hdcDest)
    {
        return FALSE;
    }

    UsesSource = ROP4_USES_SOURCE(rop4);
    UsesPattern = ROP4_USES_PATTERN(rop4);
    if (!hdcSrc && (UsesSource || UsesPattern))
        return FALSE;

    /* Check if we need a mask and have a mask bitmap */
    if (ROP4_USES_MASK(rop4) && (hbmMask != NULL))
    {
        /* Reference the mask bitmap */
        psurfMask = SURFACE_ShareLockSurface(hbmMask);
        if (psurfMask == NULL)
        {
            EngSetLastError(ERROR_INVALID_HANDLE);
            return FALSE;
        }

        /* Make sure the mask bitmap is 1 BPP */
        if (gajBitsPerFormat[psurfMask->SurfObj.iBitmapFormat] != 1)
        {
            SURFACE_ShareUnlockSurface(psurfMask);
            EngSetLastError(ERROR_INVALID_HANDLE);
            return FALSE;
        }
    }
    else
    {
        /* We use NULL, if we need a mask, the Eng function will take care of
           that and use the brushobject to get a mask */
        psurfMask = NULL;
    }

    MaskPoint.x = xMask;
    MaskPoint.y = yMask;

    /* Take care of source and destination bitmap */
    TRACE("Locking DCs\n");
    ahDC[0] = hdcDest;
    ahDC[1] = UsesSource ? hdcSrc : NULL;
    if (!GDIOBJ_bLockMultipleObjects(2, (HGDIOBJ*)ahDC, apObj, GDIObjType_DC_TYPE))
    {
        WARN("Invalid dc handle (dest=0x%p, src=0x%p) passed to NtGdiMaskBlt\n", hdcDest, hdcSrc);
        if(psurfMask) SURFACE_ShareUnlockSurface(psurfMask);
        return FALSE;
    }
    DCDest = apObj[0];
    DCSrc = apObj[1];

    ASSERT(DCDest);
    if (NULL == DCDest)
    {
        if(DCSrc) DC_UnlockDc(DCSrc);
        WARN("Invalid destination dc handle (0x%p) passed to NtGdiMaskBlt\n", hdcDest);
        if(psurfMask) SURFACE_ShareUnlockSurface(psurfMask);
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    if (DCDest->dctype == DCTYPE_INFO)
    {
        if(DCSrc) DC_UnlockDc(DCSrc);
        DC_UnlockDc(DCDest);
        /* Yes, Windows really returns TRUE in this case */
        if(psurfMask) SURFACE_ShareUnlockSurface(psurfMask);
        return TRUE;
    }

    if (UsesSource)
    {
        ASSERT(DCSrc);
        if (DCSrc->dctype == DCTYPE_INFO)
        {
            DC_UnlockDc(DCDest);
            DC_UnlockDc(DCSrc);
            /* Yes, Windows really returns TRUE in this case */
            if(psurfMask) SURFACE_ShareUnlockSurface(psurfMask);
            return TRUE;
        }
    }

    pdcattr = DCDest->pdcattr;

    DestRect.left   = nXDest;
    DestRect.top    = nYDest;
    DestRect.right  = nXDest + nWidth;
    DestRect.bottom = nYDest + nHeight;
    IntLPtoDP(DCDest, (LPPOINT)&DestRect, 2);

    DestRect.left   += DCDest->ptlDCOrig.x;
    DestRect.top    += DCDest->ptlDCOrig.y;
    DestRect.right  += DCDest->ptlDCOrig.x;
    DestRect.bottom += DCDest->ptlDCOrig.y;

    if (DCDest->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
    {
       IntUpdateBoundsRect(DCDest, &DestRect);
    }

    SourcePoint.x = nXSrc;
    SourcePoint.y = nYSrc;

    if (UsesSource)
    {
        IntLPtoDP(DCSrc, (LPPOINT)&SourcePoint, 1);

        SourcePoint.x += DCSrc->ptlDCOrig.x;
        SourcePoint.y += DCSrc->ptlDCOrig.y;
        /* Calculate Source Rect */
        SourceRect.left = SourcePoint.x;
        SourceRect.top = SourcePoint.y;
        SourceRect.right = SourcePoint.x + DestRect.right - DestRect.left;
        SourceRect.bottom = SourcePoint.y + DestRect.bottom - DestRect.top ;
    }
    else
    {
        SourceRect.left = 0;
        SourceRect.top = 0;
        SourceRect.right = 0;
        SourceRect.bottom = 0;
    }

    /* Prepare blit */
    DC_vPrepareDCsForBlit(DCDest, &DestRect, DCSrc, &SourceRect);

    if (pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
        DC_vUpdateFillBrush(DCDest);

    /* Determine surfaces to be used in the bitblt */
    BitmapDest = DCDest->dclevel.pSurface;
    if (!BitmapDest)
        goto cleanup;

    if (UsesSource)
    {
        BitmapSrc = DCSrc->dclevel.pSurface;
        if (!BitmapSrc)
            goto cleanup;

        /* Create the XLATEOBJ. */
        EXLATEOBJ_vInitXlateFromDCsEx(&exlo, DCSrc, DCDest, crBackColor);
        XlateObj = &exlo.xlo;
    }

    TRACE("DestRect: (%d,%d)-(%d,%d) and SourcePoint is (%d,%d)\n",
          DestRect.left, DestRect.top, DestRect.right, DestRect.bottom,
          SourcePoint.x, SourcePoint.y);

    TRACE("nWidth is '%d' and nHeight is '%d'.\n", nWidth, nHeight);

    /* Fix BitBlt so that it will not flip left to right */
    if ((DestRect.left > DestRect.right) && (nWidth < 0))
    {
        SourcePoint.x += nWidth;
        nWidth = -nWidth;
    }

    /* Fix BitBlt so that it will not flip top to bottom */
    if ((DestRect.top > DestRect.bottom) && (nHeight < 0))
    {
        SourcePoint.y += nHeight;
        nHeight = -nHeight;
    }

    /* Make Well Ordered so that we don't flip either way */
    RECTL_vMakeWellOrdered(&DestRect);

    /* Perform the bitblt operation */
    Status = IntEngBitBlt(&BitmapDest->SurfObj,
                          BitmapSrc ? &BitmapSrc->SurfObj : NULL,
                          psurfMask ? &psurfMask->SurfObj : NULL,
                          (CLIPOBJ *)&DCDest->co,
                          XlateObj,
                          &DestRect,
                          &SourcePoint,
                          &MaskPoint,
                          &DCDest->eboFill.BrushObject,
                          &DCDest->dclevel.pbrFill->ptOrigin,
                          rop4);

    if (UsesSource)
        EXLATEOBJ_vCleanup(&exlo);
cleanup:
    DC_vFinishBlit(DCDest, DCSrc);
    if (UsesSource)
    {
        DC_UnlockDc(DCSrc);
    }
    DC_UnlockDc(DCDest);
    if(psurfMask) SURFACE_ShareUnlockSurface(psurfMask);

    if (!Status)
        EngSetLastError(ERROR_INVALID_PARAMETER);

    return Status;
}

BOOL
APIENTRY
NtGdiPlgBlt(
    IN HDC hdcTrg,
    IN LPPOINT pptlTrg,
    IN HDC hdcSrc,
    IN INT xSrc,
    IN INT ySrc,
    IN INT cxSrc,
    IN INT cySrc,
    IN HBITMAP hbmMask,
    IN INT xMask,
    IN INT yMask,
    IN DWORD crBackColor)
{
    FIXME("NtGdiPlgBlt: unimplemented.\n");
    return FALSE;
}

BOOL
NTAPI
GreStretchBltMask(
    HDC hDCDest,
    INT XOriginDest,
    INT YOriginDest,
    INT WidthDest,
    INT HeightDest,
    HDC hDCSrc,
    INT XOriginSrc,
    INT YOriginSrc,
    INT WidthSrc,
    INT HeightSrc,
    DWORD dwRop4,
    IN DWORD dwBackColor,
    HDC hDCMask,
    INT XOriginMask,
    INT YOriginMask)
{
    PDC DCDest;
    PDC DCSrc  = NULL;
    PDC DCMask = NULL;
    HDC ahDC[3];
    PGDIOBJ apObj[3];
    PDC_ATTR pdcattr;
    SURFACE *BitmapDest, *BitmapSrc = NULL;
    SURFACE *BitmapMask = NULL;
    RECTL DestRect;
    RECTL SourceRect;
    POINTL MaskPoint;
    BOOL Status = FALSE;
    EXLATEOBJ exlo;
    XLATEOBJ *XlateObj = NULL;
    POINTL BrushOrigin;
    BOOL UsesSource;
    BOOL UsesMask;
    ROP4 rop4;
    BOOL Case0000, Case0101, Case1010, CaseExcept;

    rop4 = WIN32_ROP4_TO_ENG_ROP4(dwRop4);

    UsesSource = ROP4_USES_SOURCE(rop4);
    UsesMask = ROP4_USES_MASK(rop4);

    if (0 == WidthDest || 0 == HeightDest || 0 == WidthSrc || 0 == HeightSrc)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return TRUE;
    }

    if (!hDCDest || (UsesSource && !hDCSrc) || (UsesMask && !hDCMask))
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    ahDC[0] = hDCDest;
    ahDC[1] = UsesSource ? hDCSrc : NULL;
    ahDC[2] = UsesMask ? hDCMask : NULL;
    if (!GDIOBJ_bLockMultipleObjects(3, (HGDIOBJ*)ahDC, apObj, GDIObjType_DC_TYPE))
    {
        WARN("Invalid dc handle (dest=0x%p, src=0x%p) passed to GreStretchBltMask\n", hDCDest, hDCSrc);
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }
    DCDest = apObj[0];
    DCSrc = apObj[1];
    DCMask = apObj[2];

    if (DCDest->dctype == DCTYPE_INFO)
    {
        if(DCSrc) GDIOBJ_vUnlockObject(&DCSrc->BaseObject);
        if(DCMask) GDIOBJ_vUnlockObject(&DCMask->BaseObject);
        GDIOBJ_vUnlockObject(&DCDest->BaseObject);
        /* Yes, Windows really returns TRUE in this case */
        return TRUE;
    }

    if (UsesSource)
    {
        if (DCSrc->dctype == DCTYPE_INFO)
        {
            GDIOBJ_vUnlockObject(&DCDest->BaseObject);
            GDIOBJ_vUnlockObject(&DCSrc->BaseObject);
            if(DCMask) GDIOBJ_vUnlockObject(&DCMask->BaseObject);
            /* Yes, Windows really returns TRUE in this case */
            return TRUE;
        }
    }


    Case0000 = ((WidthDest < 0) && (HeightDest < 0) && (WidthSrc < 0) && (HeightSrc < 0));
    Case0101 = ((WidthDest < 0) && (HeightDest > 0) && (WidthSrc < 0) && (HeightSrc > 0));
    Case1010 = ((WidthDest > 0) && (HeightDest < 0) && (WidthSrc > 0) && (HeightSrc < 0));
    CaseExcept = (Case0000 || Case0101 || Case1010);

    pdcattr = DCDest->pdcattr;

    DestRect.left   = XOriginDest;
    DestRect.top    = YOriginDest;
    DestRect.right  = XOriginDest+WidthDest;
    DestRect.bottom = YOriginDest+HeightDest;

    /* Account for possible negative span values */
    if ((WidthDest < 0) && !CaseExcept)
    {
        DestRect.left++;
        DestRect.right++;
    }
    if ((HeightDest < 0) && !CaseExcept)
    {
        DestRect.top++;
        DestRect.bottom++;
    }

    IntLPtoDP(DCDest, (LPPOINT)&DestRect, 2);

    DestRect.left   += DCDest->ptlDCOrig.x;
    DestRect.top    += DCDest->ptlDCOrig.y;
    DestRect.right  += DCDest->ptlDCOrig.x;
    DestRect.bottom += DCDest->ptlDCOrig.y;

    if (DCDest->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
    {
       IntUpdateBoundsRect(DCDest, &DestRect);
    }

    SourceRect.left   = XOriginSrc;
    SourceRect.top    = YOriginSrc;
    SourceRect.right  = XOriginSrc+WidthSrc;
    SourceRect.bottom = YOriginSrc+HeightSrc;

    /* Account for possible negative span values */
    if ((WidthSrc < 0) && !CaseExcept)
    {
        SourceRect.left++;
        SourceRect.right++;
    }
    if ((HeightSrc < 0) && !CaseExcept)
    {
        SourceRect.top++;
        SourceRect.bottom++;
    }

    if (UsesSource)
    {
        IntLPtoDP(DCSrc, (LPPOINT)&SourceRect, 2);

        SourceRect.left   += DCSrc->ptlDCOrig.x;
        SourceRect.top    += DCSrc->ptlDCOrig.y;
        SourceRect.right  += DCSrc->ptlDCOrig.x;
        SourceRect.bottom += DCSrc->ptlDCOrig.y;
    }

    BrushOrigin.x = 0;
    BrushOrigin.y = 0;

    /* Only prepare Source and Dest, hdcMask represents a DIB */
    DC_vPrepareDCsForBlit(DCDest, &DestRect, DCSrc, &SourceRect);

    if (pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
        DC_vUpdateFillBrush(DCDest);

    /* Determine surfaces to be used in the bitblt */
    BitmapDest = DCDest->dclevel.pSurface;
    if (BitmapDest == NULL)
        goto failed;
    if (UsesSource)
    {
        BitmapSrc = DCSrc->dclevel.pSurface;
        if (BitmapSrc == NULL)
            goto failed;

        /* Create the XLATEOBJ. */
        EXLATEOBJ_vInitXlateFromDCsEx(&exlo, DCSrc, DCDest, dwBackColor);
        XlateObj = &exlo.xlo;
    }

    /* Offset the brush */
    BrushOrigin.x += DCDest->ptlDCOrig.x;
    BrushOrigin.y += DCDest->ptlDCOrig.y;

    /* Make mask surface for source surface */
    if (BitmapSrc && DCMask)
    {
        BitmapMask = DCMask->dclevel.pSurface;
        if (BitmapMask &&
            (BitmapMask->SurfObj.sizlBitmap.cx < WidthSrc ||
             BitmapMask->SurfObj.sizlBitmap.cy < HeightSrc))
        {
            WARN("%dx%d mask is smaller than %dx%d bitmap\n",
                    BitmapMask->SurfObj.sizlBitmap.cx, BitmapMask->SurfObj.sizlBitmap.cy,
                    WidthSrc, HeightSrc);
            EXLATEOBJ_vCleanup(&exlo);
            goto failed;
        }
        /* Create mask offset point */
        MaskPoint.x = XOriginMask;
        MaskPoint.y = YOriginMask;
        IntLPtoDP(DCMask, &MaskPoint, 1);
        MaskPoint.x += DCMask->ptlDCOrig.x;
        MaskPoint.y += DCMask->ptlDCOrig.y;
    }

    TRACE("Calling IntEngStrethBlt SourceRect: (%d,%d)-(%d,%d) and DestRect: (%d,%d)-(%d,%d).\n",
          SourceRect.left, SourceRect.top, SourceRect.right, SourceRect.bottom,
          DestRect.left, DestRect.top, DestRect.right, DestRect.bottom);

    /* Perform the bitblt operation */
    Status = IntEngStretchBlt(&BitmapDest->SurfObj,
                              BitmapSrc ? &BitmapSrc->SurfObj : NULL,
                              BitmapMask ? &BitmapMask->SurfObj : NULL,
                              (CLIPOBJ *)&DCDest->co,
                              XlateObj,
                              &DCDest->dclevel.ca,
                              &DestRect,
                              &SourceRect,
                              BitmapMask ? &MaskPoint : NULL,
                              &DCDest->eboFill.BrushObject,
                              &BrushOrigin,
                              rop4);
    if (UsesSource)
    {
        EXLATEOBJ_vCleanup(&exlo);
    }

failed:
    DC_vFinishBlit(DCDest, DCSrc);
    if (UsesSource)
    {
        DC_UnlockDc(DCSrc);
    }
    if (DCMask)
    {
        DC_UnlockDc(DCMask);
    }
    DC_UnlockDc(DCDest);

    return Status;
}


BOOL APIENTRY
NtGdiStretchBlt(
    HDC hDCDest,
    INT XOriginDest,
    INT YOriginDest,
    INT WidthDest,
    INT HeightDest,
    HDC hDCSrc,
    INT XOriginSrc,
    INT YOriginSrc,
    INT WidthSrc,
    INT HeightSrc,
    DWORD dwRop3,
    IN DWORD dwBackColor)
{
    dwRop3 = dwRop3 & ~(NOMIRRORBITMAP|CAPTUREBLT);

    return GreStretchBltMask(
                hDCDest,
                XOriginDest,
                YOriginDest,
                WidthDest,
                HeightDest,
                hDCSrc,
                XOriginSrc,
                YOriginSrc,
                WidthSrc,
                HeightSrc,
                MAKEROP4(dwRop3 & 0xFF0000, dwRop3),
                dwBackColor,
                NULL,
                0,
                0);
}


BOOL FASTCALL
IntPatBlt(
    PDC pdc,
    INT XLeft,
    INT YLeft,
    INT Width,
    INT Height,
    DWORD dwRop3,
    PEBRUSHOBJ pebo)
{
    RECTL DestRect;
    SURFACE *psurf;
    POINTL BrushOrigin;
    BOOL ret;
    PBRUSH pbrush;

    ASSERT(pebo);
    pbrush = pebo->pbrush;
    ASSERT(pbrush);

    if (pbrush->flAttrs & BR_IS_NULL)
    {
        return TRUE;
    }

    if (Width >= 0)
    {
        DestRect.left = XLeft;
        DestRect.right = XLeft + Width;
    }
    else
    {
        DestRect.left = XLeft + Width;
        DestRect.right = XLeft;
    }

    if (Height >= 0)
    {
        DestRect.top = YLeft;
        DestRect.bottom = YLeft + Height;
    }
    else
    {
        DestRect.top = YLeft + Height;
        DestRect.bottom = YLeft;
    }

    IntLPtoDP(pdc, (LPPOINT)&DestRect, 2);

    DestRect.left   += pdc->ptlDCOrig.x;
    DestRect.top    += pdc->ptlDCOrig.y;
    DestRect.right  += pdc->ptlDCOrig.x;
    DestRect.bottom += pdc->ptlDCOrig.y;

    if (pdc->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
    {
       IntUpdateBoundsRect(pdc, &DestRect);
    }

#ifdef _USE_DIBLIB_
    BrushOrigin.x = pbrush->ptOrigin.x + pdc->ptlDCOrig.x + XLeft;
    BrushOrigin.y = pbrush->ptOrigin.y + pdc->ptlDCOrig.y + YLeft;
#else
    BrushOrigin.x = pbrush->ptOrigin.x + pdc->ptlDCOrig.x;
    BrushOrigin.y = pbrush->ptOrigin.y + pdc->ptlDCOrig.y;
#endif

    DC_vPrepareDCsForBlit(pdc, &DestRect, NULL, NULL);

    psurf = pdc->dclevel.pSurface;

    ret = IntEngBitBlt(&psurf->SurfObj,
                       NULL,
                       NULL,
                       (CLIPOBJ *)&pdc->co,
                       NULL,
                       &DestRect,
                       NULL,
                       NULL,
                       &pebo->BrushObject,
                       &BrushOrigin,
                       WIN32_ROP3_TO_ENG_ROP4(dwRop3));

    DC_vFinishBlit(pdc, NULL);

    return ret;
}
/* --- pretranslation split: the remaining function groups live in bitblt.split2..2.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "bitblt.split2.c"
