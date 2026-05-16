/* path.split3.c — #include fragment of path.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original path.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


static
PPATH
FASTCALL
PATH_WidenPath(DC *dc)
{
    PPATH pPath, pNewPath;

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath)
    {
        EngSetLastError( ERROR_CAN_NOT_COMPLETE );
        return NULL;
    }

    pNewPath = PATH_WidenPathEx(dc, pPath);
    PATH_UnlockPath(pPath);
    return pNewPath;
}

PPATH
FASTCALL
PATH_WidenPathEx(DC *dc, PPATH pPath)
{
    INT size;
    UINT penWidth, penStyle;
    DWORD obj_type;
    LPEXTLOGPEN elp;
    PDC_ATTR pdcattr = dc->pdcattr;

    if (pPath->state != PATH_Closed)
    {
        TRACE("PWP 1\n");
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        return NULL;
    }

    size = GreGetObject(pdcattr->hpen, 0, NULL);
    if (!size)
    {
        TRACE("PWP 2\n");
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        return NULL;
    }

    elp = ExAllocatePoolWithTag(PagedPool, size, TAG_PATH);
    if (elp == NULL)
    {
        TRACE("PWP 3\n");
        EngSetLastError(ERROR_OUTOFMEMORY);
        return NULL;
    }

    GreGetObject(pdcattr->hpen, size, elp);

    obj_type = GDI_HANDLE_GET_TYPE(pdcattr->hpen);
    if (obj_type == GDI_OBJECT_TYPE_PEN)
    {
        penStyle = ((LOGPEN*)elp)->lopnStyle;
    }
    else if (obj_type == GDI_OBJECT_TYPE_EXTPEN)
    {
        penStyle = elp->elpPenStyle;
    }
    else
    {
        TRACE("PWP 4\n");
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        ExFreePoolWithTag(elp, TAG_PATH);
        return NULL;
    }

    penWidth = elp->elpWidth;
    ExFreePoolWithTag(elp, TAG_PATH);

    /* The function cannot apply to cosmetic pens */
    if (obj_type == GDI_OBJECT_TYPE_EXTPEN &&
        (PS_TYPE_MASK & penStyle) == PS_COSMETIC)
    {
        TRACE("PWP 5\n");
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        return FALSE;
    }

    return IntGdiWidenPath(pPath, penWidth, penStyle, dc->dclevel.laPath.eMiterLimit);
}

static inline INT int_from_fixed(FIXED f)
{
    return (f.fract >= 0x8000) ? (f.value + 1) : f.value;
}

/**********************************************************************
 *      PATH_BezierTo
 *
 * Internally used by PATH_add_outline
 */
static
VOID
FASTCALL
PATH_BezierTo(
    PPATH pPath,
    POINT *lppt,
    INT n)
{
    if (n < 2) return;

    if (n == 2)
    {
        PATH_AddEntry(pPath, &lppt[1], PT_LINETO);
    }
    else if (n == 3)
    {
        add_points( pPath, lppt, 3, PT_BEZIERTO );
    }
    else
    {
        POINT pt[3];
        INT i = 0;

        pt[2] = lppt[0];
        n--;

        while (n > 2)
        {
            pt[0] = pt[2];
            pt[1] = lppt[i + 1];
            pt[2].x = (lppt[i + 2].x + lppt[i + 1].x) / 2;
            pt[2].y = (lppt[i + 2].y + lppt[i + 1].y) / 2;
            add_points( pPath, pt, 3, PT_BEZIERTO );
            n--;
            i++;
        }

        pt[0] = pt[2];
        pt[1] = lppt[i + 1];
        pt[2] = lppt[i + 2];
        add_points( pPath, pt, 3, PT_BEZIERTO );
    }
}

static
BOOL
FASTCALL
PATH_add_outline(
    PDC dc,
    PPATH pPath,
    INT x,
    INT y,
    TTPOLYGONHEADER *header,
    DWORD size)
{
    TTPOLYGONHEADER *start;
    POINT pt;
    BOOL bResult = FALSE;

    start = header;

    while ((char *)header < (char *)start + size)
    {
        TTPOLYCURVE *curve;

        if (header->dwType != TT_POLYGON_TYPE)
        {
            ERR("Unknown header type %lu\n", header->dwType);
            goto cleanup;
        }

        pt.x = x + int_from_fixed(header->pfxStart.x);
        pt.y = y - int_from_fixed(header->pfxStart.y);
        PATH_AddEntry(pPath, &pt, PT_MOVETO);

        curve = (TTPOLYCURVE *)(header + 1);

        while ((char *)curve < (char *)header + header->cb)
        {
            TRACE("curve->wType %d\n", curve->wType);

            switch(curve->wType)
            {
                case TT_PRIM_LINE:
                {
                    WORD i;

                    for (i = 0; i < curve->cpfx; i++)
                    {
                        pt.x = x + int_from_fixed(curve->apfx[i].x);
                        pt.y = y - int_from_fixed(curve->apfx[i].y);
                        PATH_AddEntry(pPath, &pt, PT_LINETO);
                    }
                    break;
                }

                case TT_PRIM_QSPLINE:
                case TT_PRIM_CSPLINE:
                {
                    WORD i;
                    POINTFX ptfx;
                    POINT *pts = ExAllocatePoolWithTag(PagedPool, (curve->cpfx + 1) * sizeof(POINT), TAG_PATH);

                    if (!pts) goto cleanup;

                    ptfx = *(POINTFX *)((char *)curve - sizeof(POINTFX));

                    pts[0].x = x + int_from_fixed(ptfx.x);
                    pts[0].y = y - int_from_fixed(ptfx.y);

                    for (i = 0; i < curve->cpfx; i++)
                    {
                        pts[i + 1].x = x + int_from_fixed(curve->apfx[i].x);
                        pts[i + 1].y = y - int_from_fixed(curve->apfx[i].y);
                    }

                    PATH_BezierTo(pPath, pts, curve->cpfx + 1);

                    ExFreePoolWithTag(pts, TAG_PATH);
                    break;
                }

                default:
                    ERR("Unknown curve type %04x\n", curve->wType);
                    goto cleanup;
            }

            curve = (TTPOLYCURVE *)&curve->apfx[curve->cpfx];
        }
        header = (TTPOLYGONHEADER *)((char *)header + header->cb);
    }

    bResult = TRUE;

cleanup:
    IntGdiCloseFigure(pPath);
    return bResult;
}

/**********************************************************************
 *      PATH_ExtTextOut
 */
BOOL
FASTCALL
PATH_ExtTextOut(
    PDC dc,
    INT x,
    INT y,
    UINT flags,
    const RECTL *lprc,
    LPCWSTR str,
    UINT count,
    const INT *dx)
{
    PPATH pPath;
    unsigned int idx, ggo_flags = GGO_NATIVE;
    POINT offset = {0, 0};

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath)
    {
        return FALSE;
    }

    if (pPath->state != PATH_Open)
    {
        ERR("PATH_ExtTextOut not open\n");
        return FALSE;
    }

    if (!count) return TRUE;
    if (flags & ETO_GLYPH_INDEX) ggo_flags |= GGO_GLYPH_INDEX;

    for (idx = 0; idx < count; idx++)
    {
        MAT2 identity = { {0, 1}, {0, 0}, {0, 0}, {0, 1} };
        GLYPHMETRICS gm;
        DWORD dwSize;
        void *outline;

        dwSize = ftGdiGetGlyphOutline(dc,
                                      str[idx],
                                      ggo_flags,
                                      &gm,
                                      0,
                                      NULL,
                                      &identity,
                                      TRUE);
        if (dwSize == GDI_ERROR)
        {
           // With default DC font,,, bitmap font?
           // ExtTextOut on a path with bitmap font selected shouldn't fail.
           // This just leads to empty path generated.
           // Ref : test_emf_ExtTextOut_on_path
           continue;
        }

        /* Add outline only if char is printable */
        if (dwSize)
        {
            outline = ExAllocatePoolWithTag(PagedPool, dwSize, TAG_PATH);
            if (!outline)
            {
               PATH_UnlockPath(pPath);
               return FALSE;
            }

            ftGdiGetGlyphOutline(dc,
                                 str[idx],
                                 ggo_flags,
                                 &gm,
                                 dwSize,
                                 outline,
                                 &identity,
                                 TRUE);

            PATH_add_outline(dc, pPath, x + offset.x, y + offset.y, outline, dwSize);

            ExFreePoolWithTag(outline, TAG_PATH);
        }

        if (dx)
        {
            if (flags & ETO_PDY)
            {
                offset.x += dx[idx * 2];
                offset.y += dx[idx * 2 + 1];
            }
            else
                offset.x += dx[idx];
        }
        else
        {
            offset.x += gm.gmCellIncX;
            offset.y += gm.gmCellIncY;
        }
    }
    PATH_UnlockPath(pPath);
    return TRUE;
}


/***********************************************************************
 * Exported functions
 */

BOOL
APIENTRY
NtGdiAbortPath(HDC hDC)
{
    PDC dc = DC_LockDc(hDC);
    if (!dc)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    if (!dc->dclevel.hPath)
    {
       DC_UnlockDc(dc);
       return TRUE;
    }

    if (!PATH_Delete(dc->dclevel.hPath))
    {
       DC_UnlockDc(dc);
       return FALSE;
    }

    dc->dclevel.hPath = 0;
    dc->dclevel.flPath &= ~DCPATH_ACTIVE;

    DC_UnlockDc(dc);
    return TRUE;
}

BOOL
APIENTRY
NtGdiBeginPath(HDC  hDC)
{
    PPATH pPath;
    PDC dc;

    dc = DC_LockDc(hDC);
    if (!dc)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    /* If path is already open, do nothing. Check if not Save DC state */
    if ((dc->dclevel.flPath & DCPATH_ACTIVE) && !(dc->dclevel.flPath & DCPATH_SAVE))
    {
        DC_UnlockDc(dc);
        return TRUE;
    }

    if (dc->dclevel.hPath)
    {
        TRACE("BeginPath 1 0x%p\n", dc->dclevel.hPath);
        if (!(dc->dclevel.flPath & DCPATH_SAVE))
        {
            // Remove previous handle.
            if (!PATH_Delete(dc->dclevel.hPath))
            {
                DC_UnlockDc(dc);
                return FALSE;
            }
        }
        else
        {
            // Clear flags and Handle.
            dc->dclevel.flPath &= ~(DCPATH_SAVE | DCPATH_ACTIVE);
            dc->dclevel.hPath = NULL;
        }
    }
    pPath = PATH_CreatePath(NUM_ENTRIES_INITIAL);
    dc->dclevel.flPath |= DCPATH_ACTIVE; // Set active ASAP!
    dc->dclevel.hPath = pPath->BaseObject.hHmgr;
    IntGetCurrentPositionEx(dc, &pPath->pos);
    IntLPtoDP( dc, &pPath->pos, 1 );
    TRACE("BP : Current pos X %d Y %d\n",pPath->pos.x, pPath->pos.y);
    PATH_UnlockPath(pPath);
    DC_UnlockDc(dc);

    if (!pPath)
    {
        return FALSE;
    }
    return TRUE;
}

BOOL
APIENTRY
NtGdiCloseFigure(HDC hDC)
{
    BOOL Ret = FALSE; // Default to failure
    PDC pDc;
    PPATH pPath;

    TRACE("Enter %s\n", __FUNCTION__);

    pDc = DC_LockDc(hDC);
    if (!pDc)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    pPath = PATH_LockPath(pDc->dclevel.hPath);
    if (!pPath)
    {
        DC_UnlockDc(pDc);
        return FALSE;
    }

    if (pPath->state == PATH_Open)
    {
        IntGdiCloseFigure(pPath);
        Ret = TRUE;
    }
    else
    {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
    }

    PATH_UnlockPath(pPath);
    DC_UnlockDc(pDc);
    return Ret;
}

BOOL
APIENTRY
NtGdiEndPath(HDC  hDC)
{
    BOOL ret = TRUE;
    PPATH pPath;
    PDC dc;

    dc = DC_LockDc(hDC);
    if (!dc)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath)
    {
        DC_UnlockDc(dc);
        return FALSE;
    }

    /* Check that path is currently being constructed */
    if ((pPath->state != PATH_Open) || !(dc->dclevel.flPath & DCPATH_ACTIVE))
    {
        TRACE("EndPath ERROR! 0x%p\n", dc->dclevel.hPath);
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        ret = FALSE;
    }
    /* Set flag to indicate that path is finished */
    else
    {
        TRACE("EndPath 0x%p\n", dc->dclevel.hPath);
        pPath->state = PATH_Closed;
        dc->dclevel.flPath &= ~DCPATH_ACTIVE;
    }

    PATH_UnlockPath(pPath);
    DC_UnlockDc(dc);
    return ret;
}

BOOL
APIENTRY
NtGdiFillPath(HDC  hDC)
{
    BOOL ret = FALSE;
    PPATH pPath, pNewPath;
    PDC_ATTR pdcattr;
    PDC dc;

    dc = DC_LockDc(hDC);
    if (!dc)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath)
    {
        DC_UnlockDc(dc);
        return FALSE;
    }

    DC_vPrepareDCsForBlit(dc, NULL, NULL, NULL);

    pdcattr = dc->pdcattr;

    if (pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
        DC_vUpdateLineBrush(dc);

    if (pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
        DC_vUpdateFillBrush(dc);

    pNewPath = PATH_FlattenPath(pPath);

    if (pNewPath->state != PATH_Closed)
    {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
    }
    else if (pNewPath->numEntriesUsed)
    {
       ret = PATH_FillPath(dc, pNewPath);
    }
    else ret = TRUE;

    PATH_UnlockPath(pNewPath);
    PATH_Delete(pNewPath->BaseObject.hHmgr);

    PATH_UnlockPath(pPath);
    PATH_Delete(pPath->BaseObject.hHmgr);
    dc->dclevel.hPath = 0;
    dc->dclevel.flPath &= ~DCPATH_ACTIVE;

    DC_vFinishBlit(dc, NULL);
    DC_UnlockDc(dc);
    return ret;
}

BOOL
APIENTRY
NtGdiFlattenPath(HDC hDC)
{
    BOOL Ret = FALSE;
    DC *pDc;
    PPATH pPath, pNewPath = NULL;

    TRACE("Enter %s\n", __FUNCTION__);

    pDc = DC_LockDc(hDC);
    if (!pDc)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    pPath = PATH_LockPath(pDc->dclevel.hPath);
    if (!pPath)
    {
        EngSetLastError( ERROR_CAN_NOT_COMPLETE );
        DC_UnlockDc(pDc);
        return FALSE;
    }

    if (pPath->state == PATH_Closed)
    {
        pNewPath = PATH_FlattenPath(pPath);
    }

    PATH_UnlockPath(pPath);

    if (pNewPath)
    {
       PATH_Delete(pDc->dclevel.hPath);
       pDc->dclevel.hPath = pNewPath->BaseObject.hHmgr;
       PATH_UnlockPath(pNewPath);
       Ret = TRUE;
    }

    DC_UnlockDc(pDc);
    return Ret;
}

_Success_(return != FALSE)
BOOL
APIENTRY
NtGdiGetMiterLimit(
    _In_ HDC hdc,
    _Out_ PDWORD pdwOut)
{
    DC *pDc;
    BOOL bResult = TRUE;
    NTSTATUS Status;

    if (!(pDc = DC_LockDc(hdc)))
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    Status = SafeProbeForWrite(pdwOut, sizeof(DWORD), 1);
    if (!NT_SUCCESS(Status))
    {
        SetLastNtError(Status);
        bResult = FALSE;
    }
    else
    {
        *pdwOut = pDc->dclevel.laPath.eMiterLimit;
    }

    DC_UnlockDc(pDc);
    return bResult;

}


INT
APIENTRY
NtGdiGetPath(
    HDC hDC,
    LPPOINT Points,
    LPBYTE Types,
    INT nSize)
{
    INT ret = -1;
    PPATH pPath;
    DC *dc;
    NTSTATUS Status;

    Status = SafeProbeForWrite(Points, nSize * sizeof(*Points), sizeof(ULONG));
    if (NT_SUCCESS(Status)) Status = SafeProbeForWrite(Types, nSize, 1);
    if (!NT_SUCCESS(Status))
    {
        SetLastNtError(Status);
        return -1;
    }

    dc = DC_LockDc(hDC);
    TRACE("NtGdiGetPath start\n");
    if (!dc)
    {
        ERR("Can't lock dc!\n");
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return -1;
    }

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath)
    {
        DC_UnlockDc(dc);
        return -1;
    }

    if (pPath->state != PATH_Closed)
    {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
        goto done;
    }

    if (nSize == 0)
    {
        ret = pPath->numEntriesUsed;
    }
    else if (nSize < pPath->numEntriesUsed)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        goto done;
    }
    else
    {
        _SEH2_TRY
        {
            memcpy(Points, pPath->pPoints, sizeof(POINT)*pPath->numEntriesUsed);
            memcpy(Types, pPath->pFlags, sizeof(BYTE)*pPath->numEntriesUsed);

            /* Convert the points to logical coordinates */
            if (!GdiPathDPtoLP(dc, Points, pPath->numEntriesUsed))
            {
                EngSetLastError(ERROR_ARITHMETIC_OVERFLOW);
                _SEH2_LEAVE;
            }

            ret = pPath->numEntriesUsed;
        }
        _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
        {
            SetLastNtError(_SEH2_GetExceptionCode());
        }
        _SEH2_END
    }

done:
    TRACE("NtGdiGetPath exit %d\n",ret);
    PATH_UnlockPath(pPath);
    DC_UnlockDc(dc);
    return ret;
}


HRGN
APIENTRY
NtGdiPathToRegion(HDC  hDC)
{
    PPATH pPath, pNewPath;
    HRGN  hrgnRval = 0;
    int Ret;
    PREGION Rgn;
    DC *pDc;
    PDC_ATTR pdcattr;

    TRACE("Enter %s\n", __FUNCTION__);

    pDc = DC_LockDc(hDC);
    if (!pDc)
    {
        ERR("Failed to lock DC %p\n", hDC);
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return NULL;
    }

    pdcattr = pDc->pdcattr;

    pPath = PATH_LockPath(pDc->dclevel.hPath);
    if (!pPath)
    {
        ERR("Failed to lock DC path %p\n", pDc->dclevel.hPath);
        DC_UnlockDc(pDc);
        return NULL;
    }

    if (pPath->state != PATH_Closed)
    {
        // FIXME: Check that setlasterror is being called correctly
        ERR("Path is not closed!\n");
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
    }
    else
    {
        /* Create the region and fill it with the path strokes */
        Rgn = REGION_AllocUserRgnWithHandle(1);
        if (!Rgn)
        {
            ERR("Failed to allocate a region\n");
            PATH_UnlockPath(pPath);
            DC_UnlockDc(pDc);
            return NULL;
        }
        hrgnRval = Rgn->BaseObject.hHmgr;

        pNewPath = PATH_FlattenPath(pPath);
        if (pNewPath == NULL)
        {
            ERR("Failed to flatten path %p\n", pDc->dclevel.hPath);
            REGION_Delete(Rgn);
            PATH_UnlockPath(pPath);
            DC_UnlockDc(pDc);
            return NULL;
        }

        Ret = PATH_PathToRegion(pNewPath, pdcattr->jFillMode, Rgn);

        PATH_UnlockPath(pNewPath);
        PATH_Delete(pNewPath->BaseObject.hHmgr);

        if (!Ret)
        {
            ERR("PATH_PathToRegion failed\n");
            REGION_Delete(Rgn);
            hrgnRval = NULL;
        }
        else
            REGION_UnlockRgn(Rgn);
    }

    PATH_UnlockPath(pPath);
    PATH_Delete(pDc->dclevel.hPath);
    pDc->dclevel.hPath = NULL;
    pDc->dclevel.flPath &= ~DCPATH_ACTIVE;

    DC_UnlockDc(pDc);
    return hrgnRval;
}

BOOL
APIENTRY
NtGdiSetMiterLimit(
    IN HDC hdc,
    IN DWORD dwNew,
    IN OUT OPTIONAL PDWORD pdwOut)
{
    DC *pDc;
    gxf_long worker, worker1;
    BOOL bResult = TRUE;
    NTSTATUS Status;

    if (!(pDc = DC_LockDc(hdc)))
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    worker.l  = dwNew;
    worker1.f = pDc->dclevel.laPath.eMiterLimit;
    pDc->dclevel.laPath.eMiterLimit = worker.f;

    if (pdwOut)
    {
        Status = SafeProbeForWrite(pdwOut, sizeof(DWORD), 1);
        if (!NT_SUCCESS(Status))
        {
            SetLastNtError(Status);
            bResult = FALSE;
        }
        else
        {
            *pdwOut = worker1.l;
        }
    }

    DC_UnlockDc(pDc);
    return bResult;
}


BOOL
APIENTRY
NtGdiStrokeAndFillPath(HDC hDC)
{
    DC *pDc;
    PDC_ATTR pdcattr;
    PPATH pPath, pNewPath;
    BOOL bRet = FALSE;

    TRACE("Enter %s\n", __FUNCTION__);

    if (!(pDc = DC_LockDc(hDC)))
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    pPath = PATH_LockPath(pDc->dclevel.hPath);
    if (!pPath)
    {
        DC_UnlockDc(pDc);
        return FALSE;
    }

    DC_vPrepareDCsForBlit(pDc, NULL, NULL, NULL);

    pdcattr = pDc->pdcattr;

    if (pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
        DC_vUpdateFillBrush(pDc);

    if (pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
        DC_vUpdateLineBrush(pDc);

    pNewPath = PATH_FlattenPath(pPath);

    if (pNewPath->state != PATH_Closed)
    {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
    }
    else if (pNewPath->numEntriesUsed)
    {
       bRet = PATH_FillPath(pDc, pNewPath);
       if (bRet) bRet = PATH_StrokePath(pDc, pNewPath);
    }
    else bRet = TRUE;

    PATH_UnlockPath(pNewPath);
    PATH_Delete(pNewPath->BaseObject.hHmgr);

    PATH_UnlockPath(pPath);
    PATH_Delete(pPath->BaseObject.hHmgr);
    pDc->dclevel.hPath = 0;
    pDc->dclevel.flPath &= ~DCPATH_ACTIVE;

    DC_vFinishBlit(pDc, NULL);
    DC_UnlockDc(pDc);
    return bRet;
}

BOOL
APIENTRY
NtGdiStrokePath(HDC hDC)
{
    DC *pDc;
    PDC_ATTR pdcattr;
    PPATH pPath, pNewPath;
    BOOL bRet = FALSE;

    TRACE("Enter %s\n", __FUNCTION__);

    if (!(pDc = DC_LockDc(hDC)))
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    pPath = PATH_LockPath(pDc->dclevel.hPath);
    if (!pPath)
    {
        DC_UnlockDc(pDc);
        return FALSE;
    }

    DC_vPrepareDCsForBlit(pDc, NULL, NULL, NULL);

    pdcattr = pDc->pdcattr;

    if (pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
        DC_vUpdateLineBrush(pDc);

    pNewPath = PATH_FlattenPath(pPath);

    if (pNewPath->state != PATH_Closed)
    {
        EngSetLastError(ERROR_CAN_NOT_COMPLETE);
    }
    else bRet = PATH_StrokePath(pDc, pNewPath);

    PATH_UnlockPath(pNewPath);
    PATH_Delete(pNewPath->BaseObject.hHmgr);

    DC_vFinishBlit(pDc, NULL);

    PATH_UnlockPath(pPath);
    PATH_Delete(pPath->BaseObject.hHmgr);
    pDc->dclevel.hPath = 0;
    pDc->dclevel.flPath &= ~DCPATH_ACTIVE;

    DC_UnlockDc(pDc);
    return bRet;
}

BOOL
APIENTRY
NtGdiWidenPath(HDC  hDC)
{
    PPATH pPath;
    BOOL Ret = FALSE;
    PDC pdc = DC_LockDc(hDC);
    TRACE("NtGdiWidenPat Enter\n");
    if (!pdc)
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    pPath = PATH_WidenPath(pdc);
    if (pPath)
    {
       TRACE("WindenPath New Path\n");
       PATH_Delete(pdc->dclevel.hPath);
       pdc->dclevel.hPath = pPath->BaseObject.hHmgr;
       Ret = TRUE;
    }
    DC_UnlockDc(pdc);
    TRACE("NtGdiWidenPat Ret %d\n",Ret);
    return Ret;
}

/* EOF */
