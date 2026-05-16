/* path.split2.c — #include fragment of path.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original path.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


BOOL
FASTCALL
PATH_PolyBezierTo(
    PDC dc,
    const POINT *pts,
    DWORD cbPoints)
{
    PPATH pPath;
    BOOL ret;

    ASSERT(dc);
    ASSERT(pts);
    ASSERT(cbPoints);

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath) return FALSE;

    ret = add_log_points_new_stroke( dc, pPath, pts, cbPoints, PT_BEZIERTO );

    PATH_UnlockPath(pPath);
    return ret;
}

BOOL
FASTCALL
PATH_PolyBezier(
    PDC dc,
    const POINT *pts,
    DWORD cbPoints)
{
    PPATH pPath;
    BYTE *type;

    ASSERT(dc);
    ASSERT(pts);
    ASSERT(cbPoints);

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath) return FALSE;

    type = add_log_points( dc, pPath, pts, cbPoints, PT_BEZIERTO );
    if (!type) return FALSE;

    type[0] = PT_MOVETO;

    PATH_UnlockPath(pPath);
    return TRUE;
}

BOOL
FASTCALL
PATH_PolyDraw(
    PDC dc,
    const POINT *pts,
    const BYTE *types,
    DWORD cbPoints)
{
    PPATH pPath;
    POINT orig_pos, cur_pos;
    ULONG i, lastmove = 0;

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath) return FALSE;

    if (pPath->state != PATH_Open)
    {
        PATH_UnlockPath(pPath);
        return FALSE;
    }

    for (i = 0; i < pPath->numEntriesUsed; i++) if (pPath->pFlags[i] == PT_MOVETO) lastmove = i;
    orig_pos = pPath->pos;

    IntGetCurrentPositionEx(dc, &cur_pos);

    TRACE("PPD : Current pos X %d Y %d\n",pPath->pos.x, pPath->pos.y);
    TRACE("PPD : last %d pos X %d Y %d\n",lastmove, pPath->pPoints[lastmove].x, pPath->pPoints[lastmove].y);


    for(i = 0; i < cbPoints; i++)
    {
        switch (types[i])
        {
        case PT_MOVETO:
            pPath->newStroke = TRUE;
            pPath->pos = pts[i];
            IntLPtoDP( dc, &pPath->pos, 1);
            lastmove = pPath->numEntriesUsed;
            break;
        case PT_LINETO:
        case PT_LINETO | PT_CLOSEFIGURE:
            if (!add_log_points_new_stroke( dc, pPath, &pts[i], 1, PT_LINETO ))
            {
               PATH_UnlockPath(pPath);
               return FALSE;
            }
            break;
        case PT_BEZIERTO:
            if ((i + 2 < cbPoints) && (types[i + 1] == PT_BEZIERTO) &&
                (types[i + 2] & ~PT_CLOSEFIGURE) == PT_BEZIERTO)
            {
                if (!add_log_points_new_stroke( dc, pPath, &pts[i], 3, PT_BEZIERTO ))
                {
                   PATH_UnlockPath(pPath);
                   return FALSE;
                }
                i += 2;
                break;
            }
            /* fall through */
        default:
            /* restore original position */
            pPath->pos = orig_pos;

            TRACE("PPD Bad   : pos X %d Y %d\n",pPath->pos.x, pPath->pos.y);

            IntGdiMoveToEx(dc, cur_pos.x, cur_pos.y, NULL);

            PATH_UnlockPath(pPath);
            return FALSE;
        }

        if (types[i] & PT_CLOSEFIGURE)
        {
            close_figure( pPath );
            pPath->pos = pPath->pPoints[lastmove];
            TRACE("PPD close : pos X %d Y %d\n",pPath->pos.x, pPath->pos.y);
        }
    }
    PATH_UnlockPath(pPath);
    return TRUE;
}

BOOL
FASTCALL
PATH_PolylineTo(
    PDC dc,
    const POINT *pts,
    DWORD cbPoints)
{
    PPATH pPath;
    BOOL ret;

    ASSERT(dc);
    ASSERT(pts);
    ASSERT(cbPoints);

    if (cbPoints < 1) return FALSE;

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath) return FALSE;

    ret = add_log_points_new_stroke( dc, pPath, pts, cbPoints, PT_LINETO );
    PATH_UnlockPath(pPath);
    return ret;
}

BOOL
FASTCALL
PATH_PolyPolygon(
    PDC dc,
    const POINT* pts,
    const INT* counts,
    UINT polygons)
{
    UINT poly, count;
    BYTE *type;
    PPATH pPath;

    ASSERT(dc);
    ASSERT(pts);
    ASSERT(counts);
    ASSERT(polygons);

    if (!polygons) return FALSE;

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath) return FALSE;


    for (poly = count = 0; poly < polygons; poly++)
    {
        if (counts[poly] < 2)
        {
           PATH_UnlockPath(pPath);
           return FALSE;
        }
        count += counts[poly];
    }

    type = add_log_points( dc, pPath, pts, count, PT_LINETO );
    if (!type)
    {
       PATH_UnlockPath(pPath);
       return FALSE;
    }

    /* make the first point of each polyline a PT_MOVETO, and close the last one */
    for (poly = 0; poly < polygons; type += counts[poly++])
    {
        type[0] = PT_MOVETO;
        type[counts[poly] - 1] = PT_LINETO | PT_CLOSEFIGURE;
    }
    PATH_UnlockPath(pPath);
    return TRUE;
}

BOOL
FASTCALL
PATH_PolyPolyline(
    PDC dc,
    const POINT* pts,
    const DWORD* counts,
    DWORD polylines)
{
    POINT pt;
    ULONG poly, point, i;
    PPATH pPath;

    ASSERT(dc);
    ASSERT(pts);
    ASSERT(counts);
    ASSERT(polylines);

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath)
    {
       return FALSE;
    }

    for (i = 0, poly = 0; poly < polylines; poly++)
    {
        for (point = 0; point < counts[poly]; point++, i++)
        {
            pt = pts[i];
            CoordLPtoDP(dc, &pt);
            PATH_AddEntry(pPath, &pt, (point == 0) ? PT_MOVETO : PT_LINETO);
        }
    }
    TRACE("PATH_PolyPolyline end count %d\n",pPath->numEntriesUsed);
    PATH_UnlockPath(pPath);
    return TRUE;
}

/* PATH_AddFlatBezier
 *
 */
BOOL
FASTCALL
PATH_AddFlatBezier(
    PPATH pPath,
    POINT *pt,
    BOOL closed)
{
    POINT *pts;
    BOOL ret = FALSE;
    INT no, i;

    pts = GDI_Bezier(pt, 4, &no);
    if (!pts) return FALSE;

    for (i = 1; i < no; i++)
    {
        if (!(ret = PATH_AddEntry(pPath, &pts[i],  (i == no - 1 && closed) ? PT_LINETO | PT_CLOSEFIGURE : PT_LINETO)))
           break;
    }

    ExFreePoolWithTag(pts, TAG_BEZIER);
    return ret;
}

/* PATH_FlattenPath
 *
 * Replaces Beziers with line segments
 *
 */
PPATH
FASTCALL
PATH_FlattenPath(PPATH pPath)
{
    PPATH newPath;
    INT srcpt;
    TRACE("PATH_FlattenPath\n");
    if (!(newPath = PATH_CreatePath(pPath->numEntriesUsed))) return NULL;

    for (srcpt = 0; srcpt < pPath->numEntriesUsed; srcpt++)
    {
        switch(pPath->pFlags[srcpt] & ~PT_CLOSEFIGURE)
        {
            case PT_MOVETO:
            case PT_LINETO:
                if (!PATH_AddEntry(newPath, &pPath->pPoints[srcpt], pPath->pFlags[srcpt]))
                {
                   PATH_UnlockPath(newPath);
                   PATH_Delete(newPath->BaseObject.hHmgr);
                   return NULL;
                }
                break;
            case PT_BEZIERTO:
                if(!PATH_AddFlatBezier(newPath, &pPath->pPoints[srcpt - 1], pPath->pFlags[srcpt + 2] & PT_CLOSEFIGURE))
                {
                   PATH_UnlockPath(newPath);
                   PATH_Delete(newPath->BaseObject.hHmgr);
                   return NULL;
                }
                srcpt += 2;
                break;
        }
    }
    TRACE("PATH_FlattenPath good\n");
    newPath->state = pPath->state;
    return newPath;
}

/* PATH_PathToRegion
 *
 * Fills Rgn from the specified path using the specified polygon
 * filling mode. The path is left unchanged.
 */
BOOL
FASTCALL
PATH_PathToRegion(
    PPATH pPath,
    INT Mode,
    PREGION Rgn)
{
    int i, pos, polygons;
    PULONG counts;
    int Ret;

    if (!pPath->numEntriesUsed) return FALSE;

    counts = ExAllocatePoolWithTag(PagedPool, (pPath->numEntriesUsed / 2) * sizeof(*counts), TAG_PATH);
    if (!counts)
    {
        ERR("Failed to allocate %lu strokes\n", (pPath->numEntriesUsed / 2) * sizeof(*counts));
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE;
    }

    pos = polygons = 0;
    ASSERT( pPath->pFlags[0] == PT_MOVETO );
    for (i = 1; i < pPath->numEntriesUsed; i++)
    {
        if (pPath->pFlags[i] != PT_MOVETO) continue;
        counts[polygons++] = i - pos;
        pos = i;
    }
    if (i > pos + 1) counts[polygons++] = i - pos;

    ASSERT( polygons <= pPath->numEntriesUsed / 2 );

    /* Fill the region with the strokes */
    Ret = REGION_SetPolyPolygonRgn(Rgn,
                                   pPath->pPoints,
                                   counts,
                                   polygons,
                                   Mode);
    if (!Ret)
    {
        ERR("REGION_SetPolyPolygonRgn failed\n");
    }

    ExFreePoolWithTag(counts, TAG_PATH);

    /* Success! */
    return Ret;
}

/* PATH_FillPath
 *
 * You can play with this as long as you like, but if you break Area.exe the purge will Begain on Path!!!
 *
 */
BOOL
FASTCALL
PATH_FillPath(
    PDC dc,
    PPATH pPath)
{
    return PATH_FillPathEx(dc, pPath, NULL);
}

BOOL
FASTCALL
PATH_FillPathEx(
    PDC dc,
    PPATH pPath,
    PBRUSH pbrFill)
{
    INT   mapMode, graphicsMode;
    SIZE  ptViewportExt, ptWindowExt;
    POINTL ptViewportOrg, ptWindowOrg;
    XFORML xform;
    PREGION  Rgn;
    PDC_ATTR pdcattr = dc->pdcattr;

    /* Allocate a temporary region */
    Rgn = IntSysCreateRectpRgn(0, 0, 0, 0);
    if (!Rgn)
    {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE;
    }

    if (!PATH_PathToRegion(pPath, pdcattr->jFillMode, Rgn))
    {
        TRACE("PFP : Fail P2R\n");
        /* EngSetLastError ? */
        REGION_Delete(Rgn);
        return FALSE;
    }

    /* Since PaintRgn interprets the region as being in logical coordinates
     * but the points we store for the path are already in device
     * coordinates, we have to set the mapping mode to MM_TEXT temporarily.
     * Using SaveDC to save information about the mapping mode / world
     * transform would be easier but would require more overhead, especially
     * now that SaveDC saves the current path.
     */

    /* Save the information about the old mapping mode */
    mapMode = pdcattr->iMapMode;
    ptViewportExt = pdcattr->szlViewportExt;
    ptViewportOrg = pdcattr->ptlViewportOrg;
    ptWindowExt   = pdcattr->szlWindowExt;
    ptWindowOrg   = pdcattr->ptlWindowOrg;

    /* Save world transform
     * NB: The Windows documentation on world transforms would lead one to
     * believe that this has to be done only in GM_ADVANCED; however, my
     * tests show that resetting the graphics mode to GM_COMPATIBLE does
     * not reset the world transform.
     */
    MatrixS2XForm(&xform, &dc->pdcattr->mxWorldToPage);

    /* Set MM_TEXT */
    IntGdiSetMapMode(dc, MM_TEXT);
    pdcattr->ptlViewportOrg.x = 0;
    pdcattr->ptlViewportOrg.y = 0;
    pdcattr->ptlWindowOrg.x = 0;
    pdcattr->ptlWindowOrg.y = 0;

    graphicsMode = pdcattr->iGraphicsMode;
    pdcattr->iGraphicsMode = GM_ADVANCED;
    GreModifyWorldTransform(dc, &xform, MWT_IDENTITY);
    pdcattr->iGraphicsMode =  graphicsMode;

    /* Paint the region */
    IntGdiFillRgn(dc, Rgn, pbrFill);
    REGION_Delete(Rgn);
    /* Restore the old mapping mode */
    IntGdiSetMapMode(dc, mapMode);
    pdcattr->szlViewportExt = ptViewportExt;
    pdcattr->ptlViewportOrg = ptViewportOrg;
    pdcattr->szlWindowExt   = ptWindowExt;
    pdcattr->ptlWindowOrg   = ptWindowOrg;

    /* Go to GM_ADVANCED temporarily to restore the world transform */
    graphicsMode = pdcattr->iGraphicsMode;
    pdcattr->iGraphicsMode = GM_ADVANCED;
    GreModifyWorldTransform(dc, &xform, MWT_SET);
    pdcattr->iGraphicsMode = graphicsMode;
    return TRUE;
}

BOOL
FASTCALL
PATH_StrokePath(
    DC *dc,
    PPATH pPath)
{
    BOOL ret = FALSE;
    INT nLinePts, nAlloc, jOldFillMode, i = 0;
    POINT *pLinePts = NULL;
    POINT ptViewportOrg, ptWindowOrg;
    SIZE szViewportExt, szWindowExt;
    DWORD mapMode, graphicsMode;
    XFORM xform;
    PDC_ATTR pdcattr = dc->pdcattr;
    PBRUSH pbrLine;
    PPATH pNewPath;

    TRACE("Enter %s\n", __FUNCTION__);

    pbrLine = dc->dclevel.pbrLine;
    if (IntIsEffectiveWidePen(pbrLine))
    {
        pNewPath = PATH_WidenPathEx(dc, pPath);
        if (pNewPath)
        {
            /* Fill the path with the WINDING fill mode */
            jOldFillMode = pdcattr->jFillMode;
            pdcattr->jFillMode = WINDING;
            PATH_FillPathEx(dc, pNewPath, pbrLine);
            pdcattr->jFillMode = jOldFillMode;

            PATH_Delete(pNewPath->BaseObject.hHmgr);
            return TRUE;
        }
    }

    /* Save the mapping mode info */
    mapMode = pdcattr->iMapMode;

    szViewportExt = *DC_pszlViewportExt(dc);
    ptViewportOrg = dc->pdcattr->ptlViewportOrg;
    szWindowExt = dc->pdcattr->szlWindowExt;
    ptWindowOrg = dc->pdcattr->ptlWindowOrg;

    MatrixS2XForm(&xform, &dc->pdcattr->mxWorldToPage);

    /* Set MM_TEXT */
    pdcattr->iMapMode = MM_TEXT;
    pdcattr->ptlViewportOrg.x = 0;
    pdcattr->ptlViewportOrg.y = 0;
    pdcattr->ptlWindowOrg.x = 0;
    pdcattr->ptlWindowOrg.y = 0;
    graphicsMode = pdcattr->iGraphicsMode;
    pdcattr->iGraphicsMode = GM_ADVANCED;
    GreModifyWorldTransform(dc, (XFORML*)&xform, MWT_IDENTITY);
    pdcattr->iGraphicsMode = graphicsMode;

    /* Allocate enough memory for the worst case without beziers (one PT_MOVETO
     * and the rest PT_LINETO with PT_CLOSEFIGURE at the end) plus some buffer
     * space in case we get one to keep the number of reallocations small. */
    nAlloc = pPath->numEntriesUsed + 1 + 300;
    pLinePts = ExAllocatePoolWithTag(PagedPool, nAlloc * sizeof(POINT), TAG_PATH);
    if (!pLinePts)
    {
        ERR("Can't allocate pool!\n");
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        goto end;
    }
    nLinePts = 0;

    for (i = 0; i < pPath->numEntriesUsed; i++)
    {
        if ((i == 0 || (pPath->pFlags[i - 1] & PT_CLOSEFIGURE))
                && (pPath->pFlags[i] != PT_MOVETO))
        {
            ERR("Expected PT_MOVETO %s, got path flag %d\n",
                    i == 0 ? "as first point" : "after PT_CLOSEFIGURE",
                    (INT)pPath->pFlags[i]);
            goto end;
        }

        switch(pPath->pFlags[i])
        {
            case PT_MOVETO:
                TRACE("Got PT_MOVETO (%ld, %ld)\n",
                       pPath->pPoints[i].x, pPath->pPoints[i].y);
                if (nLinePts >= 2) IntGdiPolyline(dc, pLinePts, nLinePts);
                nLinePts = 0;
                pLinePts[nLinePts++] = pPath->pPoints[i];
                break;
            case PT_LINETO:
            case (PT_LINETO | PT_CLOSEFIGURE):
                TRACE("Got PT_LINETO (%ld, %ld)\n",
                       pPath->pPoints[i].x, pPath->pPoints[i].y);
                pLinePts[nLinePts++] = pPath->pPoints[i];
                break;
            case PT_BEZIERTO:
                TRACE("Got PT_BEZIERTO\n");
                if (pPath->pFlags[i + 1] != PT_BEZIERTO ||
                        (pPath->pFlags[i + 2] & ~PT_CLOSEFIGURE) != PT_BEZIERTO)
                {
                    ERR("Path didn't contain 3 successive PT_BEZIERTOs\n");
                    ret = FALSE;
                    goto end;
                }
                else
                {
                    INT nBzrPts, nMinAlloc;
                    POINT *pBzrPts = GDI_Bezier(&pPath->pPoints[i - 1], 4, &nBzrPts);
                    /* Make sure we have allocated enough memory for the lines of
                     * this bezier and the rest of the path, assuming we won't get
                     * another one (since we won't reallocate again then). */
                    nMinAlloc = nLinePts + (pPath->numEntriesUsed - i) + nBzrPts;
                    if (nAlloc < nMinAlloc)
                    {
                        // Reallocate memory

                        POINT *Realloc = NULL;
                        nAlloc = nMinAlloc * 2;

                        Realloc = ExAllocatePoolWithTag(PagedPool,
                                                        nAlloc * sizeof(POINT),
                                                        TAG_PATH);

                        if (!Realloc)
                        {
                            ERR("Can't allocate pool!\n");
                            ExFreePoolWithTag(pBzrPts, TAG_BEZIER);
                            goto end;
                        }

                        memcpy(Realloc, pLinePts, nLinePts * sizeof(POINT));
                        ExFreePoolWithTag(pLinePts, TAG_PATH);
                        pLinePts = Realloc;
                    }
                    memcpy(&pLinePts[nLinePts], &pBzrPts[1], (nBzrPts - 1) * sizeof(POINT));
                    nLinePts += nBzrPts - 1;
                    ExFreePoolWithTag(pBzrPts, TAG_BEZIER);
                    i += 2;
                }
                break;
            default:
                ERR("Got path flag %d (not supported)\n", (INT)pPath->pFlags[i]);
                goto end;
        }

        if (pPath->pFlags[i] & PT_CLOSEFIGURE)
        {
            pLinePts[nLinePts++] = pLinePts[0];
        }
    }
    if (nLinePts >= 2)
        IntGdiPolyline(dc, pLinePts, nLinePts);

    ret = TRUE;

end:
    if (pLinePts) ExFreePoolWithTag(pLinePts, TAG_PATH);

    /* Restore the old mapping mode */
    pdcattr->iMapMode =  mapMode;
    pdcattr->szlWindowExt.cx = szWindowExt.cx;
    pdcattr->szlWindowExt.cy = szWindowExt.cy;
    pdcattr->ptlWindowOrg.x = ptWindowOrg.x;
    pdcattr->ptlWindowOrg.y = ptWindowOrg.y;

    pdcattr->szlViewportExt.cx = szViewportExt.cx;
    pdcattr->szlViewportExt.cy = szViewportExt.cy;
    pdcattr->ptlViewportOrg.x = ptViewportOrg.x;
    pdcattr->ptlViewportOrg.y = ptViewportOrg.y;

    /* Restore the world transform */
    XForm2MatrixS(&dc->pdcattr->mxWorldToPage, &xform);

    /* If we've moved the current point then get its new position
       which will be in device (MM_TEXT) co-ords, convert it to
       logical co-ords and re-set it.  This basically updates
       dc->CurPosX|Y so that their values are in the correct mapping
       mode.
    */
    if (i > 0)
    {
        POINT pt;
        IntGetCurrentPositionEx(dc, &pt);
        IntDPtoLP(dc, &pt, 1);
        IntGdiMoveToEx(dc, pt.x, pt.y, NULL);
    }
    TRACE("Leave %s, ret=%d\n", __FUNCTION__, ret);
    return ret;
}

#define round(x) ((int)((x)>0?(x)+0.5:(x)-0.5))

PPATH FASTCALL
IntGdiWidenPath(PPATH pPath, UINT penWidth, UINT penStyle, FLOAT eMiterLimit)
{
    INT i, j, numStrokes, numOldStrokes, penWidthIn, penWidthOut;
    PPATH flat_path, pNewPath = NULL, *pStrokes = NULL, *pOldStrokes, pUpPath, pDownPath;
    BYTE *type;
    DWORD joint, endcap;
    KFLOATING_SAVE fpsave;

    endcap = (PS_ENDCAP_MASK & penStyle);
    joint = (PS_JOIN_MASK & penStyle);

    if (!(flat_path = PATH_FlattenPath(pPath)))
    {
        ERR("PATH_FlattenPath\n");
        return NULL;
    }

    penWidthIn = penWidth / 2;
    penWidthOut = penWidth / 2;
    if (penWidthIn + penWidthOut < penWidth)
        penWidthOut++;

    numStrokes = 0;

    for (i = 0, j = 0; i < flat_path->numEntriesUsed; i++, j++)
    {
        POINT point;
        if ((i == 0 || (flat_path->pFlags[i - 1] & PT_CLOSEFIGURE)) &&
                (flat_path->pFlags[i] != PT_MOVETO))
        {
            ERR("Expected PT_MOVETO %s, got path flag %c\n",
                    i == 0 ? "as first point" : "after PT_CLOSEFIGURE",
                    flat_path->pFlags[i]);
            goto Exit;
        }
        switch(flat_path->pFlags[i])
        {
            case PT_MOVETO:
                if (numStrokes > 0)
                {
                    pStrokes[numStrokes - 1]->state = PATH_Closed;
                }
                numOldStrokes = numStrokes;
                numStrokes++;
                j = 0;
                if (numStrokes == 1)
                    pStrokes = ExAllocatePoolWithTag(PagedPool, sizeof(*pStrokes), TAG_PATH);
                else
                {
                    pOldStrokes = pStrokes; // Save old pointer.
                    pStrokes = ExAllocatePoolWithTag(PagedPool, numStrokes * sizeof(*pStrokes), TAG_PATH);
                    if (!pStrokes)
                    {
                       ExFreePoolWithTag(pOldStrokes, TAG_PATH);
                       goto Exit;
                    }
                    RtlCopyMemory(pStrokes, pOldStrokes, numOldStrokes * sizeof(PPATH));
                    ExFreePoolWithTag(pOldStrokes, TAG_PATH); // Free old pointer.
                }
                if (!pStrokes)
                {
                   goto Exit;
                }
                pStrokes[numStrokes - 1] = ExAllocatePoolWithTag(PagedPool, sizeof(PATH), TAG_PATH);
                if (!pStrokes[numStrokes - 1])
                {
                    ASSERT(FALSE); // FIXME
                }
                PATH_InitGdiPath(pStrokes[numStrokes - 1]);
                pStrokes[numStrokes - 1]->state = PATH_Open;
            case PT_LINETO:
            case (PT_LINETO | PT_CLOSEFIGURE):
                point.x = flat_path->pPoints[i].x;
                point.y = flat_path->pPoints[i].y;
                PATH_AddEntry(pStrokes[numStrokes - 1], &point, flat_path->pFlags[i]);
                break;
            case PT_BEZIERTO:
                /* Should never happen because of the FlattenPath call */
                ERR("Should never happen\n");
                break;
            default:
                ERR("Got path flag %c\n", flat_path->pFlags[i]);
                goto Exit;
        }
    }

    pNewPath = PATH_CreatePath( flat_path->numEntriesUsed );
    if (pNewPath == NULL)
    {
        ERR("PATH_CreatePath\n");
        goto Exit;
    }

    KeSaveFloatingPointState(&fpsave);

    for (i = 0; i < numStrokes; i++)
    {
        pUpPath = ExAllocatePoolWithTag(PagedPool, sizeof(PATH), TAG_PATH);
        PATH_InitGdiPath(pUpPath);
        pUpPath->state = PATH_Open;
        pDownPath = ExAllocatePoolWithTag(PagedPool, sizeof(PATH), TAG_PATH);
        PATH_InitGdiPath(pDownPath);
        pDownPath->state = PATH_Open;

        for (j = 0; j < pStrokes[i]->numEntriesUsed; j++)
        {
            /* Beginning or end of the path if not closed */
            if ((!(pStrokes[i]->pFlags[pStrokes[i]->numEntriesUsed - 1] & PT_CLOSEFIGURE)) && (j == 0 || j == pStrokes[i]->numEntriesUsed - 1))
            {
                /* Compute segment angle */
                INT xo, yo, xa, ya;
                double theta;
                POINT pt;
                POINT corners[2];
                if (j == 0)
                {
                    xo = pStrokes[i]->pPoints[j].x;
                    yo = pStrokes[i]->pPoints[j].y;
                    xa = pStrokes[i]->pPoints[1].x;
                    ya = pStrokes[i]->pPoints[1].y;
                }
                else
                {
                    xa = pStrokes[i]->pPoints[j - 1].x;
                    ya = pStrokes[i]->pPoints[j - 1].y;
                    xo = pStrokes[i]->pPoints[j].x;
                    yo = pStrokes[i]->pPoints[j].y;
                }
                theta = atan2(ya - yo, xa - xo);
                switch(endcap)
                {
                    case PS_ENDCAP_SQUARE :
                        pt.x = xo + round(sqrt(2) * penWidthOut * cos(M_PI_4 + theta));
                        pt.y = yo + round(sqrt(2) * penWidthOut * sin(M_PI_4 + theta));
                        PATH_AddEntry(pUpPath, &pt, (j == 0 ? PT_MOVETO : PT_LINETO));
                        pt.x = xo + round(sqrt(2) * penWidthIn * cos(- M_PI_4 + theta));
                        pt.y = yo + round(sqrt(2) * penWidthIn * sin(- M_PI_4 + theta));
                        PATH_AddEntry(pUpPath, &pt, PT_LINETO);
                        break;
                    case PS_ENDCAP_FLAT :
                        pt.x = xo + round(penWidthOut * cos(theta + M_PI_2));
                        pt.y = yo + round(penWidthOut * sin(theta + M_PI_2));
                        PATH_AddEntry(pUpPath, &pt, (j == 0 ? PT_MOVETO : PT_LINETO));
                        pt.x = xo - round(penWidthIn * cos(theta + M_PI_2));
                        pt.y = yo - round(penWidthIn * sin(theta + M_PI_2));
                        PATH_AddEntry(pUpPath, &pt, PT_LINETO);
                        break;
                    case PS_ENDCAP_ROUND :
                    default :
                        corners[0].x = xo - penWidthIn;
                        corners[0].y = yo - penWidthIn;
                        corners[1].x = xo + penWidthOut;
                        corners[1].y = yo + penWidthOut;
                        PATH_DoArcPart(pUpPath , corners, theta + M_PI_2 , theta + 3 * M_PI_4, (j == 0 ? PT_MOVETO : FALSE));
                        PATH_DoArcPart(pUpPath , corners, theta + 3 * M_PI_4 , theta + M_PI, FALSE);
                        PATH_DoArcPart(pUpPath , corners, theta + M_PI, theta +  5 * M_PI_4, FALSE);
                        PATH_DoArcPart(pUpPath , corners, theta + 5 * M_PI_4 , theta + 3 * M_PI_2, FALSE);
                        break;
                }
            }
            /* Corpse of the path */
            else
            {
                /* Compute angle */
                INT previous, next;
                double xa, ya, xb, yb, xo, yo;
                double alpha, theta, miterWidth;
                DWORD _joint = joint;
                POINT pt;
                PPATH pInsidePath, pOutsidePath;
                if (j > 0 && j < pStrokes[i]->numEntriesUsed - 1)
                {
                    previous = j - 1;
                    next = j + 1;
                }
                else if (j == 0)
                {
                    previous = pStrokes[i]->numEntriesUsed - 1;
                    next = j + 1;
                }
                else
                {
                    previous = j - 1;
                    next = 0;
                }
                xo = pStrokes[i]->pPoints[j].x;
                yo = pStrokes[i]->pPoints[j].y;
                xa = pStrokes[i]->pPoints[previous].x;
                ya = pStrokes[i]->pPoints[previous].y;
                xb = pStrokes[i]->pPoints[next].x;
                yb = pStrokes[i]->pPoints[next].y;
                theta = atan2(yo - ya, xo - xa);
                alpha = atan2(yb - yo, xb - xo) - theta;
                if (alpha > 0) alpha -= M_PI;
                else alpha += M_PI;
                if (_joint == PS_JOIN_MITER && eMiterLimit < fabs(1 / sin(alpha / 2)))
                {
                    _joint = PS_JOIN_BEVEL;
                }
                if (alpha > 0)
                {
                    pInsidePath = pUpPath;
                    pOutsidePath = pDownPath;
                }
                else if (alpha < 0)
                {
                    pInsidePath = pDownPath;
                    pOutsidePath = pUpPath;
                }
                else
                {
                    continue;
                }
                /* Inside angle points */
                if (alpha > 0)
                {
                    pt.x = xo - round(penWidthIn * cos(theta + M_PI_2));
                    pt.y = yo - round(penWidthIn * sin(theta + M_PI_2));
                }
                else
                {
                    pt.x = xo + round(penWidthIn * cos(theta + M_PI_2));
                    pt.y = yo + round(penWidthIn * sin(theta + M_PI_2));
                }
                PATH_AddEntry(pInsidePath, &pt, PT_LINETO);
                if (alpha > 0)
                {
                    pt.x = xo + round(penWidthIn * cos(M_PI_2 + alpha + theta));
                    pt.y = yo + round(penWidthIn * sin(M_PI_2 + alpha + theta));
                }
                else
                {
                    pt.x = xo - round(penWidthIn * cos(M_PI_2 + alpha + theta));
                    pt.y = yo - round(penWidthIn * sin(M_PI_2 + alpha + theta));
                }
                PATH_AddEntry(pInsidePath, &pt, PT_LINETO);
                /* Outside angle point */
                switch(_joint)
                {
                    case PS_JOIN_MITER :
                        miterWidth = fabs(penWidthOut / cos(M_PI_2 - fabs(alpha) / 2));
                        pt.x = xo + round(miterWidth * cos(theta + alpha / 2));
                        pt.y = yo + round(miterWidth * sin(theta + alpha / 2));
                        PATH_AddEntry(pOutsidePath, &pt, PT_LINETO);
                        break;
                    case PS_JOIN_BEVEL :
                        if (alpha > 0)
                        {
                            pt.x = xo + round(penWidthOut * cos(theta + M_PI_2));
                            pt.y = yo + round(penWidthOut * sin(theta + M_PI_2));
                        }
                        else
                        {
                            pt.x = xo - round(penWidthOut * cos(theta + M_PI_2));
                            pt.y = yo - round(penWidthOut * sin(theta + M_PI_2));
                        }
                        PATH_AddEntry(pOutsidePath, &pt, PT_LINETO);
                        if (alpha > 0)
                        {
                            pt.x = xo - round(penWidthOut * cos(M_PI_2 + alpha + theta));
                            pt.y = yo - round(penWidthOut * sin(M_PI_2 + alpha + theta));
                        }
                        else
                        {
                            pt.x = xo + round(penWidthOut * cos(M_PI_2 + alpha + theta));
                            pt.y = yo + round(penWidthOut * sin(M_PI_2 + alpha + theta));
                        }
                        PATH_AddEntry(pOutsidePath, &pt, PT_LINETO);
                        break;
                    case PS_JOIN_ROUND :
                    default :
                        if (alpha > 0)
                        {
                            pt.x = xo + round(penWidthOut * cos(theta + M_PI_2));
                            pt.y = yo + round(penWidthOut * sin(theta + M_PI_2));
                        }
                        else
                        {
                            pt.x = xo - round(penWidthOut * cos(theta + M_PI_2));
                            pt.y = yo - round(penWidthOut * sin(theta + M_PI_2));
                        }
                        PATH_AddEntry(pOutsidePath, &pt, PT_BEZIERTO);
                        pt.x = xo + round(penWidthOut * cos(theta + alpha / 2));
                        pt.y = yo + round(penWidthOut * sin(theta + alpha / 2));
                        PATH_AddEntry(pOutsidePath, &pt, PT_BEZIERTO);
                        if (alpha > 0)
                        {
                            pt.x = xo - round(penWidthOut * cos(M_PI_2 + alpha + theta));
                            pt.y = yo - round(penWidthOut * sin(M_PI_2 + alpha + theta));
                        }
                        else
                        {
                            pt.x = xo + round(penWidthOut * cos(M_PI_2 + alpha + theta));
                            pt.y = yo + round(penWidthOut * sin(M_PI_2 + alpha + theta));
                        }
                        PATH_AddEntry(pOutsidePath, &pt, PT_BEZIERTO);
                        break;
                }
            }
        }
        type = add_points( pNewPath, pUpPath->pPoints, pUpPath->numEntriesUsed, PT_LINETO );
        type[0] = PT_MOVETO;
        reverse_points( pDownPath->pPoints, pDownPath->numEntriesUsed );
        type = add_points( pNewPath, pDownPath->pPoints, pDownPath->numEntriesUsed, PT_LINETO );
        if (pStrokes[i]->pFlags[pStrokes[i]->numEntriesUsed - 1] & PT_CLOSEFIGURE) type[0] = PT_MOVETO;

        PATH_DestroyGdiPath(pStrokes[i]);
        ExFreePoolWithTag(pStrokes[i], TAG_PATH);
        PATH_DestroyGdiPath(pUpPath);
        ExFreePoolWithTag(pUpPath, TAG_PATH);
        PATH_DestroyGdiPath(pDownPath);
        ExFreePoolWithTag(pDownPath, TAG_PATH);
    }

    pNewPath->state = PATH_Closed;
    PATH_UnlockPath(pNewPath);

    KeRestoreFloatingPointState(&fpsave);

Exit:
    if (pStrokes) ExFreePoolWithTag(pStrokes, TAG_PATH);
    HPATH hpathToDelete = flat_path->BaseObject.hHmgr;
    PATH_UnlockPath(flat_path);
    PATH_Delete(hpathToDelete);

    return pNewPath;
}