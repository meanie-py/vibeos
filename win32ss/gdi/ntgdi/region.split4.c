/* region.split4.c — #include fragment of region.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original region.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


HRGN
NTAPI
GreCreatePolyPolygonRgn(
    _In_ const POINT *ppt,
    _In_ const ULONG *pcPoints,
    _In_ ULONG cPolygons,
    _In_ INT iMode)
{
    PREGION prgn;
    HRGN hrgn;

    /* Allocate a new region */
    prgn = REGION_AllocUserRgnWithHandle(0);
    if (prgn == NULL)
    {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return NULL;
    }

    /* Call the internal function and check for success */
    if (REGION_SetPolyPolygonRgn(prgn, ppt, pcPoints, cPolygons, iMode))
    {
        /* Success, get the handle and unlock the region */
        hrgn = prgn->BaseObject.hHmgr;
        REGION_UnlockRgn(prgn);
    }
    else
    {
        /* Failure, delete the region */
        REGION_Delete(prgn);
        hrgn = NULL;
    }

    return hrgn;
}

BOOL
FASTCALL
IntRectInRegion(
    HRGN  hRgn,
    LPRECTL rc)
{
    PREGION Rgn;
    BOOL Ret;

    Rgn = REGION_LockRgn(hRgn);
    if (Rgn == NULL)
    {
        return ERROR;
    }

    Ret = REGION_RectInRegion(Rgn, rc);
    REGION_UnlockRgn(Rgn);
    return Ret;
}

//
// NtGdi Exported Functions
//
INT
APIENTRY
NtGdiCombineRgn(
    IN HRGN hrgnDst,
    IN HRGN hrgnSrc1,
    IN HRGN hrgnSrc2,
    IN INT iMode)
{
    HRGN ahrgn[3];
    PREGION aprgn[3];
    INT iResult;

    /* Validate the combine mode */
    if ((iMode < RGN_AND) || (iMode > RGN_COPY))
    {
        return ERROR;
    }

    /* Validate that we have the required regions */
    if ((hrgnDst == NULL) ||
        (hrgnSrc1 == NULL) ||
        ((iMode != RGN_COPY) && (hrgnSrc2 == NULL)))
    {
        DPRINT1("NtGdiCombineRgn invalid parameters: %p, %p, %p, %d\n",
                hrgnDst, hrgnSrc1, hrgnSrc2, iMode);
        EngSetLastError(ERROR_INVALID_HANDLE);
        return ERROR;
    }

    /* Lock all regions */
    ahrgn[0] = hrgnDst;
    ahrgn[1] = hrgnSrc1;
    ahrgn[2] = iMode != RGN_COPY ? hrgnSrc2 : NULL;
    if (!GDIOBJ_bLockMultipleObjects(3, (HGDIOBJ*)ahrgn, (PVOID*)aprgn, GDIObjType_RGN_TYPE))
    {
        DPRINT1("NtGdiCombineRgn failed to lock regions: %p, %p, %p, %d\n",
                hrgnDst, hrgnSrc1, hrgnSrc2, iMode);
        return ERROR;
    }

    /* HACK: Sync usermode attributes */
    REGION_vSyncRegion(aprgn[0]);
    if (aprgn[1] != aprgn[0])
        REGION_vSyncRegion(aprgn[1]);
    if ((aprgn[2] != NULL) && (aprgn[2] != aprgn[0]) && (aprgn[2] != aprgn[1]))
        REGION_vSyncRegion(aprgn[2]);

    /* Call the internal function */
    iResult = IntGdiCombineRgn(aprgn[0], aprgn[1], aprgn[2], iMode);

    /* Unlock and return */
    REGION_UnlockRgn(aprgn[0]);
    REGION_UnlockRgn(aprgn[1]);
    if (aprgn[2] != NULL)
        REGION_UnlockRgn(aprgn[2]);

    return iResult;
}

HRGN
APIENTRY
NtGdiCreateEllipticRgn(
    INT Left,
    INT Top,
    INT Right,
    INT Bottom)
{
    return NtGdiCreateRoundRectRgn(Left,
                                   Top,
                                   Right, Bottom,
                                   Right - Left,
                                   Bottom - Top);
}

HRGN
APIENTRY
NtGdiCreateRectRgn(
    INT LeftRect,
    INT TopRect,
    INT RightRect,
    INT BottomRect)
{
    PREGION pRgn;
    HRGN hRgn;

    /* Allocate region data structure with space for 1 RECTL */
    pRgn = REGION_AllocUserRgnWithHandle(1);
    if (pRgn == NULL)
    {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return NULL;
    }

    hRgn = pRgn->BaseObject.hHmgr;

    REGION_SetRectRgn(pRgn, LeftRect, TopRect, RightRect, BottomRect);
    REGION_UnlockRgn(pRgn);

    DPRINT("Returning %p.\n", hRgn);

    return hRgn;
}

HRGN
APIENTRY
NtGdiCreateRoundRectRgn(
    INT left,
    INT top,
    INT right,
    INT bottom,
    INT ellipse_width,
    INT ellipse_height)
{
    PREGION obj;
    HRGN hrgn;
    int a, b, i, x, y;
    INT64 asq, bsq, dx, dy, err;
    RECT *rects;

    /* Make the dimensions sensible */
    if (left > right)
    {
        INT tmp = left;
        left = right;
        right = tmp;
    }

    if (top > bottom)
    {
        INT tmp = top;
        top = bottom;
        bottom = tmp;
    }

    /* the region is for the rectangle interior, but only at right and bottom for some reason */
    right--;
    bottom--;

    ellipse_width = min( right - left, abs( ellipse_width ));
    ellipse_height = min( bottom - top, abs( ellipse_height ));

    /* Check if we can do a normal rectangle instead */

    if ((ellipse_width < 2) || (ellipse_height < 2))
        return NtGdiCreateRectRgn(left, top, right, bottom);

    obj = REGION_AllocUserRgnWithHandle( ellipse_height );
    if (obj == NULL)
        return 0;

    hrgn = obj->BaseObject.hHmgr;

    obj->rdh.rcBound.left   = left;
    obj->rdh.rcBound.top    = top;
    obj->rdh.rcBound.right  = right;
    obj->rdh.rcBound.bottom = bottom;
    rects = obj->Buffer;

    /* based on an algorithm by Alois Zingl */

    a = ellipse_width - 1;
    b = ellipse_height - 1;
    asq = (INT64)8 * a * a;
    bsq = (INT64)8 * b * b;
    dx  = (INT64)4 * b * b * (1 - a);
    dy  = (INT64)4 * a * a * (1 + (b % 2));
    err = dx + dy + a * a * (b % 2);

    x = 0;
    y = ellipse_height / 2;

    rects[y].left = left;
    rects[y].right = right;

    while (x <= ellipse_width / 2)
    {
        INT64 e2 = 2 * err;
        if (e2 >= dx)
        {
            x++;
            err += dx += bsq;
        }
        if (e2 <= dy)
        {
            y++;
            err += dy += asq;
            rects[y].left = left + x;
            rects[y].right = right - x;
        }
    }
    for (i = 0; i < ellipse_height / 2; i++)
    {
        rects[i].left = rects[b - i].left;
        rects[i].right = rects[b - i].right;
        rects[i].top = top + i;
        rects[i].bottom = rects[i].top + 1;
    }
    for (; i < ellipse_height; i++)
    {
        rects[i].top = bottom - ellipse_height + i;
        rects[i].bottom = rects[i].top + 1;
    }
    rects[ellipse_height / 2].top = top + ellipse_height / 2;  /* extend to top of rectangle */

    REGION_UnlockRgn(obj);
    return hrgn;
}

BOOL
APIENTRY
NtGdiEqualRgn(
    HRGN  hSrcRgn1,
    HRGN  hSrcRgn2)
{
    HRGN ahrgn[2];
    PREGION aprgn[2];
    PREGION rgn1, rgn2;
    PRECTL tRect1, tRect2;
    ULONG i;
    BOOL bRet = FALSE;

    /* Check if we got 2 regions */
    if ((hSrcRgn1 == NULL) || (hSrcRgn2 == NULL))
    {
        return FALSE;
    }

    /* Check if these are the same regions */
    if (hSrcRgn1 == hSrcRgn2)
    {
        /* Make sure this region is valid */
        if ((GDI_HANDLE_GET_TYPE(hSrcRgn1) == GDILoObjType_LO_REGION_TYPE) &&
            GreIsHandleValid(hSrcRgn1))
        {
            return TRUE;
        }
        return FALSE;
    }

    /* Lock both regions */
    ahrgn[0] = hSrcRgn1;
    ahrgn[1] = hSrcRgn2;
    if (!GDIOBJ_bLockMultipleObjects(2, (HGDIOBJ*)ahrgn, (PVOID*)aprgn, GDIObjType_RGN_TYPE))
    {
        DPRINT1("NtGdiEqualRgn failed to lock regions: %p, %p\n",
                hSrcRgn1, hSrcRgn2);
        return FALSE;
    }

    REGION_vSyncRegion(aprgn[0]);
    REGION_vSyncRegion(aprgn[1]);

    rgn1 = aprgn[0];
    rgn2 = aprgn[1];

    if (rgn1->rdh.nCount != rgn2->rdh.nCount)
        goto exit;

    if (rgn1->rdh.nCount == 0)
    {
        bRet = TRUE;
        goto exit;
    }

    if ((rgn1->rdh.rcBound.left   != rgn2->rdh.rcBound.left)  ||
        (rgn1->rdh.rcBound.right  != rgn2->rdh.rcBound.right) ||
        (rgn1->rdh.rcBound.top    != rgn2->rdh.rcBound.top)   ||
        (rgn1->rdh.rcBound.bottom != rgn2->rdh.rcBound.bottom))
        goto exit;

    tRect1 = rgn1->Buffer;
    tRect2 = rgn2->Buffer;

    if ((tRect1 == NULL) || (tRect2 == NULL))
        goto exit;

    for (i=0; i < rgn1->rdh.nCount; i++)
    {
        if ((tRect1[i].left   != tRect2[i].left)  ||
            (tRect1[i].right  != tRect2[i].right) ||
            (tRect1[i].top    != tRect2[i].top)   ||
            (tRect1[i].bottom != tRect2[i].bottom))
            goto exit;
    }

    bRet = TRUE;

exit:
    REGION_UnlockRgn(rgn1);
    REGION_UnlockRgn(rgn2);
    return bRet;
}

HRGN
APIENTRY
NtGdiExtCreateRegion(
    OPTIONAL LPXFORM Xform,
    DWORD Count,
    LPRGNDATA RgnData)
{
    HRGN hRgn;
    PREGION Region;
    DWORD nCount = 0;
    DWORD iType = 0;
    DWORD dwSize = 0;
    UINT i;
    RECT* rects;
    NTSTATUS Status = STATUS_SUCCESS;
    MATRIX matrix;
    XFORMOBJ xo;

    DPRINT("NtGdiExtCreateRegion\n");
    Status = SafeProbeForRead(RgnData, Count, 1);
    if (NT_SUCCESS(Status))
    {
        nCount = RgnData->rdh.nCount;
        iType = RgnData->rdh.iType;
        dwSize = RgnData->rdh.dwSize;
        rects = (RECT*)RgnData->Buffer;
    }

    if (!NT_SUCCESS(Status))
    {
        SetLastNtError(Status);
        return NULL;
    }

    /* Check parameters, but don't set last error here */
    if ((Count < sizeof(RGNDATAHEADER) + nCount * sizeof(RECT)) ||
        (iType != RDH_RECTANGLES) ||
        (dwSize != sizeof(RGNDATAHEADER)))
    {
        return NULL;
    }

    Region = REGION_AllocUserRgnWithHandle(nCount);

    if (Region == NULL)
    {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return FALSE;
    }
    hRgn = Region->BaseObject.hHmgr;

    _SEH2_TRY
    {
        /* Insert the rectangles one by one */
        for(i=0; i<nCount; i++)
        {
            if ( rects[i].left < rects[i].right && rects[i].top < rects[i].bottom )
            {
                if (!REGION_UnionRectWithRgn(Region, &rects[i]))
                {
                   REGION_UnlockRgn(Region);
                   GreDeleteObject(hRgn);
                   hRgn = NULL;
                   _SEH2_LEAVE;
                }
            }
        }

        if (Xform != NULL)
        {
            ULONG ret;

            /* Init the XFORMOBJ from the Xform struct */
            Status = STATUS_INVALID_PARAMETER;
            XFORMOBJ_vInit(&xo, &matrix);
            ret = XFORMOBJ_iSetXform(&xo, (XFORML*)Xform);

            /* Check for error */
            if (ret != DDI_ERROR)
            {
                /* Apply the coordinate transformation on the rects */
                if (XFORMOBJ_bApplyXform(&xo,
                                         XF_LTOL,
                                         Region->rdh.nCount * 2,
                                         Region->Buffer,
                                         Region->Buffer))
                {
                    Status = STATUS_SUCCESS;
                }
            }
        }
    }
    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
    {
        Status = _SEH2_GetExceptionCode();
    }
    _SEH2_END;
    if (!NT_SUCCESS(Status))
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        REGION_UnlockRgn(Region);
        GreDeleteObject(hRgn);
        return NULL;
    }

    if (hRgn) REGION_UnlockRgn(Region);

    return hRgn;
}

INT
APIENTRY
NtGdiGetRgnBox(
    HRGN hRgn,
    PRECTL pRect)
{
    PREGION Rgn;
    RECTL SafeRect;
    DWORD ret;
    NTSTATUS Status = STATUS_SUCCESS;

    Rgn = REGION_LockRgn(hRgn);
    if (Rgn == NULL)
    {
        return ERROR;
    }

    ret = REGION_GetRgnBox(Rgn, &SafeRect);
    REGION_UnlockRgn(Rgn);
    if (ret == ERROR)
    {
        return ret;
    }

    Status = SafeProbeForWrite(pRect, sizeof(RECT), 1);
    if (NT_SUCCESS(Status))
    {
        *pRect = SafeRect;
    }
    if (!NT_SUCCESS(Status))
    {
        return ERROR;
    }

    return ret;
}

INT
APIENTRY
NtGdiOffsetRgn(
    _In_ HRGN hrgn,
    _In_ INT cx,
    _In_ INT cy)
{
    PREGION prgn;
    INT iResult;

    DPRINT("NtGdiOffsetRgn: hrgn %p cx %d cy %d\n", hrgn, cx, cy);

    /* Lock the region */
    prgn = REGION_LockRgn(hrgn);
    if (prgn == NULL)
    {
        DPRINT1("NtGdiOffsetRgn: failed to lock region %p\n", hrgn);
        return ERROR;
    }

    /* Call the internal function */
    if (!REGION_bOffsetRgn(prgn, cx, cy))
    {
        iResult = ERROR;
    }
    else
    {
        iResult = REGION_Complexity(prgn);
    }

    /* Unlock and return the result */
    REGION_UnlockRgn(prgn);
    return iResult;
}

BOOL
APIENTRY
NtGdiPtInRegion(
    _In_ HRGN hrgn,
    _In_ INT x,
    _In_ INT y)
{
    PREGION prgn;
    BOOL bResult;

    /* Lock the region */
    prgn = REGION_LockRgn(hrgn);
    if (prgn == NULL)
    {
        DPRINT1("NtGdiPtInRegion: hrgn error\n");
        return FALSE;
    }

    /* Call the internal function */
    bResult = REGION_PtInRegion(prgn, x, y);

    /* Unlock and return the result */
    REGION_UnlockRgn(prgn);
    return bResult;
}

__kernel_entry
BOOL
APIENTRY
NtGdiRectInRegion(
    _In_ HRGN hrgn,
    _Inout_ LPRECT prclUnsafe)
{
    RECTL rcTemp;

    /* Probe and copy the rect */
    {
        NTSTATUS __SehStatus = SafeProbeForRead(prclUnsafe, sizeof(RECT), 1);
        if (!NT_SUCCESS(__SehStatus))
        {
            DPRINT1("NtGdiRectInRegion: Exception accessing the rect\n");
            return FALSE;
        }
    }
    rcTemp = *prclUnsafe;

    /* Call the internal function */
    return IntRectInRegion(hrgn, &rcTemp);
}

BOOL
APIENTRY
NtGdiSetRectRgn(
    _In_ HRGN hrgn,
    _In_ INT xLeft,
    _In_ INT yTop,
    _In_ INT xRight,
    _In_ INT yBottom)
{
    PREGION prgn;

    /* Lock the region */
    prgn = REGION_LockRgn(hrgn);
    if (prgn == NULL)
    {
        return FALSE;
    }

    /* Call the internal API */
    REGION_SetRectRgn(prgn, xLeft, yTop, xRight, yBottom);

    /* Unlock the region and return success */
    REGION_UnlockRgn(prgn);
    return TRUE;
}

/*!
 * MSDN: GetRegionData, Return Values:
 *
 * "If the function succeeds and dwCount specifies an adequate number of bytes,
 * the return value is always dwCount. If dwCount is too small or the function
 * fails, the return value is 0. If lpRgnData is NULL, the return value is the
 * required number of bytes.
 *
 * If the function fails, the return value is zero."
 */
_Success_(return!=0)
__kernel_entry
ULONG
APIENTRY
NtGdiGetRegionData(
    _In_ HRGN hrgn,
    _In_ ULONG cjBuffer,
    _Out_writes_bytes_to_opt_(cjBuffer, return) LPRGNDATA lpRgnData)
{
    ULONG cjRects, cjSize;
    PREGION prgn;
    NTSTATUS Status;

    /* Lock the region */
    prgn = REGION_LockRgn(hrgn);
    if (prgn == NULL)
    {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    /* Calculate the region sizes */
    cjRects = prgn->rdh.nCount * sizeof(RECT);
    cjSize = cjRects + sizeof(RGNDATAHEADER);

    /* Check if region data is requested */
    if (lpRgnData)
    {
        /* Check if the buffer is large enough */
        if (cjBuffer >= cjSize)
        {
            /* Probe the buffer and copy the data */
            Status = SafeProbeForWrite(lpRgnData, cjSize, sizeof(ULONG));
            if (!NT_SUCCESS(Status))
            {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                cjSize = 0;
            }
            else
            {
                RtlCopyMemory(lpRgnData, &prgn->rdh, sizeof(RGNDATAHEADER));
                RtlCopyMemory(lpRgnData->Buffer, prgn->Buffer, cjRects);
                lpRgnData->rdh.iType = RDH_RECTANGLES;
                lpRgnData->rdh.nRgnSize = cjRects;
            }
        }
        else
        {
            /* Buffer is too small */
            EngSetLastError(ERROR_INVALID_PARAMETER);
            cjSize = 0;
        }
    }

    /* Unlock the region and return the size */
    REGION_UnlockRgn(prgn);
    return cjSize;
}


/* EOF */
