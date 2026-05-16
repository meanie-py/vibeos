/* coord.split2.c — #include fragment of coord.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original coord.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


//
// Mirror Window function.
//
VOID
FASTCALL
IntMirrorWindowOrg(PDC dc)
{
    PDC_ATTR pdcattr;
    LONG X, cx;

    pdcattr = dc->pdcattr;

    if (!(pdcattr->dwLayout & LAYOUT_RTL))
    {
        pdcattr->ptlWindowOrg.x = pdcattr->lWindowOrgx; // Flip it back.
        return;
    }

    /* Copy the window extension, so no one can mess with it */
    cx = pdcattr->szlViewportExt.cx;
    if (cx == 0) return;
    //
    // WOrgx = wox - (Width - 1) * WExtx / VExtx
    //
    X = (dc->erclWindow.right - dc->erclWindow.left) - 1; // Get device width - 1

    X = (X * pdcattr->szlWindowExt.cx) / cx;

    pdcattr->ptlWindowOrg.x = pdcattr->lWindowOrgx - X; // Now set the inverted win origion.
    pdcattr->flXform |= PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID;

    return;
}

VOID
NTAPI
DC_vSetLayout(
    IN PDC pdc,
    IN LONG wox,
    IN DWORD dwLayout)
{
    PDC_ATTR pdcattr = pdc->pdcattr;

    pdcattr->dwLayout = dwLayout;

    if (!(dwLayout & LAYOUT_ORIENTATIONMASK)) return;

    if (dwLayout & LAYOUT_RTL)
    {
        pdcattr->iMapMode = MM_ANISOTROPIC;
    }

    //pdcattr->szlWindowExt.cy = -pdcattr->szlWindowExt.cy;
    //pdcattr->ptlWindowOrg.x  = -pdcattr->ptlWindowOrg.x;

    //if (wox == -1)
    //    IntMirrorWindowOrg(pdc);
    //else
    //    pdcattr->ptlWindowOrg.x = wox - pdcattr->ptlWindowOrg.x;

    if (!(pdcattr->flTextAlign & TA_CENTER)) pdcattr->flTextAlign |= TA_RIGHT;

    if (pdc->dclevel.flPath & DCPATH_CLOCKWISE)
        pdc->dclevel.flPath &= ~DCPATH_CLOCKWISE;
    else
        pdc->dclevel.flPath |= DCPATH_CLOCKWISE;

    pdcattr->flXform |= (PAGE_EXTENTS_CHANGED |
                         INVALIDATE_ATTRIBUTES |
                         WORLD_XFORM_CHANGED |
                         DEVICE_TO_WORLD_INVALID);
}

// NtGdiSetLayout
//
// The default is left to right. This function changes it to right to left, which
// is the standard in Arabic and Hebrew cultures.
//
/*
 * @implemented
 */
DWORD
APIENTRY
NtGdiSetLayout(
    IN HDC hdc,
    IN LONG wox,
    IN DWORD dwLayout)
{
    PDC pdc;
    DWORD dwOldLayout;

    pdc = DC_LockDc(hdc);
    if (!pdc)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return GDI_ERROR;
    }

    dwOldLayout = pdc->pdcattr->dwLayout;
    DC_vSetLayout(pdc, wox, dwLayout);

    DC_UnlockDc(pdc);
    return dwOldLayout;
}

/*
 * @implemented
 */
LONG
APIENTRY
NtGdiGetDeviceWidth(
    IN HDC hdc)
{
    PDC dc;
    LONG Ret;
    dc = DC_LockDc(hdc);
    if (!dc)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    Ret = dc->erclWindow.right - dc->erclWindow.left;
    DC_UnlockDc(dc);
    return Ret;
}

/*
 * @implemented
 */
BOOL
APIENTRY
NtGdiMirrorWindowOrg(
    IN HDC hdc)
{
    PDC dc;
    dc = DC_LockDc(hdc);
    if (!dc)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }
    IntMirrorWindowOrg(dc);
    DC_UnlockDc(dc);
    return TRUE;
}

/*
 * @implemented
 */
BOOL
APIENTRY
NtGdiSetSizeDevice(
    IN HDC hdc,
    IN INT cxVirtualDevice,
    IN INT cyVirtualDevice)
{
    PDC dc;
    PDC_ATTR pdcattr;

    if (!cxVirtualDevice || !cyVirtualDevice)
    {
        return FALSE;
    }

    dc = DC_LockDc(hdc);
    if (!dc) return FALSE;

    pdcattr = dc->pdcattr;

    pdcattr->szlVirtualDeviceSize.cx = cxVirtualDevice;
    pdcattr->szlVirtualDeviceSize.cy = cyVirtualDevice;

    DC_UnlockDc(dc);

    return TRUE;
}

/*
 * @implemented
 */
BOOL
APIENTRY
NtGdiSetVirtualResolution(
    IN HDC hdc,
    IN INT cxVirtualDevicePixel,
    IN INT cyVirtualDevicePixel,
    IN INT cxVirtualDeviceMm,
    IN INT cyVirtualDeviceMm)
{
    PDC dc;
    PDC_ATTR pdcattr;

    /* Check parameters (all zeroes resets to real resolution) */
    if (cxVirtualDevicePixel == 0 && cyVirtualDevicePixel == 0 &&
        cxVirtualDeviceMm == 0 && cyVirtualDeviceMm == 0)
    {
        cxVirtualDevicePixel = NtGdiGetDeviceCaps(hdc, HORZRES);
        cyVirtualDevicePixel = NtGdiGetDeviceCaps(hdc, VERTRES);
        cxVirtualDeviceMm = NtGdiGetDeviceCaps(hdc, HORZSIZE);
        cyVirtualDeviceMm = NtGdiGetDeviceCaps(hdc, VERTSIZE);
    }
    else if (cxVirtualDevicePixel == 0 || cyVirtualDevicePixel == 0 ||
             cxVirtualDeviceMm == 0 || cyVirtualDeviceMm == 0)
    {
        return FALSE;
    }

    dc = DC_LockDc(hdc);
    if (!dc) return FALSE;

    pdcattr = dc->pdcattr;

    pdcattr->szlVirtualDevicePixel.cx = cxVirtualDevicePixel;
    pdcattr->szlVirtualDevicePixel.cy = cyVirtualDevicePixel;
    pdcattr->szlVirtualDeviceMm.cx = cxVirtualDeviceMm;
    pdcattr->szlVirtualDeviceMm.cy = cyVirtualDeviceMm;

//    DC_vUpdateXforms(dc);
    DC_UnlockDc(dc);
    return TRUE;
}

static
VOID FASTCALL
DC_vGetAspectRatioFilter(PDC pDC, LPSIZE AspectRatio)
{
    if (pDC->pdcattr->flFontMapper & 1) // TRUE assume 1.
    {
        // "This specifies that Windows should only match fonts that have the
        // same aspect ratio as the display.", Programming Windows, Fifth Ed.
        AspectRatio->cx = pDC->ppdev->gdiinfo.ulLogPixelsX;
        AspectRatio->cy = pDC->ppdev->gdiinfo.ulLogPixelsY;
    }
    else
    {
        AspectRatio->cx = 0;
        AspectRatio->cy = 0;
    }
}

BOOL APIENTRY
GreGetDCPoint(
    HDC hDC,
    UINT iPoint,
    PPOINTL Point)
{
    BOOL Ret = TRUE;
    DC *pdc;
    SIZE Size;
    PSIZEL pszlViewportExt;

    if (!Point)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    pdc = DC_LockDc(hDC);
    if (!pdc)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    switch (iPoint)
    {
        case GdiGetViewPortExt:
            pszlViewportExt = DC_pszlViewportExt(pdc);
            Point->x = pszlViewportExt->cx;
            Point->y = pszlViewportExt->cy;
            break;

        case GdiGetWindowExt:
            Point->x = pdc->pdcattr->szlWindowExt.cx;
            Point->y = pdc->pdcattr->szlWindowExt.cy;
            break;

        case GdiGetViewPortOrg:
            *Point = pdc->pdcattr->ptlViewportOrg;
            break;

        case GdiGetWindowOrg:
            *Point = pdc->pdcattr->ptlWindowOrg;
            break;

        case GdiGetDCOrg:
            *Point = pdc->ptlDCOrig;
            break;

        case GdiGetAspectRatioFilter:
            DC_vGetAspectRatioFilter(pdc, &Size);
            Point->x = Size.cx;
            Point->y = Size.cy;
            break;

        default:
            EngSetLastError(ERROR_INVALID_PARAMETER);
            Ret = FALSE;
            break;
    }

    DC_UnlockDc(pdc);
    return Ret;
}

BOOL
WINAPI
GreSetDCOrg(
    _In_ HDC hdc,
    _In_ LONG x,
    _In_ LONG y,
    _In_opt_ PRECTL Rect)
{
    PDC dc;

    dc = DC_LockDc(hdc);
    if (!dc) return FALSE;

    /* Set DC Origin */
    dc->ptlDCOrig.x = x;
    dc->ptlDCOrig.y = y;

    /* Recalculate Fill Origin */
    dc->ptlFillOrigin.x = dc->dclevel.ptlBrushOrigin.x + x;
    dc->ptlFillOrigin.y = dc->dclevel.ptlBrushOrigin.y + y;

    /* Set DC Window Rectangle */
    if (Rect)
        dc->erclWindow = *Rect;

    DC_UnlockDc(dc);
    return TRUE;
}

BOOL
WINAPI
GreGetDCOrgEx(
    _In_ HDC hdc,
    _Out_ PPOINTL Point,
    _Out_ PRECTL Rect)
{
    PDC dc;

    dc = DC_LockDc(hdc);
    if (!dc) return FALSE;

    /* Retrieve DC Window Rectangle without a check */
    *Rect = dc->erclWindow;

    DC_UnlockDc(dc);

    /* Use default call for DC Origin and parameter checking */
    return GreGetDCPoint( hdc, GdiGetDCOrg, Point);
}

BOOL
WINAPI
GreGetWindowExtEx(
    _In_ HDC hdc,
    _Out_ LPSIZE lpSize)
{
    return GreGetDCPoint(hdc, GdiGetWindowExt, (PPOINTL)lpSize);
}

BOOL
WINAPI
GreGetViewportExtEx(
    _In_ HDC hdc,
    _Out_ LPSIZE lpSize)
{
    return GreGetDCPoint(hdc, GdiGetViewPortExt, (PPOINTL)lpSize);
}

BOOL APIENTRY
NtGdiGetDCPoint(
    HDC hDC,
    UINT iPoint,
    PPOINTL Point)
{
    BOOL Ret;
    POINTL SafePoint;
    NTSTATUS Status;

    if (!Point)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    Ret = GreGetDCPoint(hDC, iPoint, &SafePoint);
    if (Ret)
    {
        Status = SafeProbeForWrite(Point, sizeof(POINT), 1);
        if (!NT_SUCCESS(Status))
        {
            Ret = FALSE;
        }
        else
        {
            *Point = SafePoint;
        }
    }

    return Ret;
}


/* EOF */
