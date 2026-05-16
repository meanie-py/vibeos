/* region.split2.c — #include fragment of region.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original region.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


/***********************************************************************
 * REGION_IntersectRegion
 */
static
BOOL
FASTCALL
REGION_IntersectRegion(
    PREGION newReg,
    PREGION reg1,
    PREGION reg2)
{
    /* Check for trivial reject */
    if ((reg1->rdh.nCount == 0) ||
        (reg2->rdh.nCount == 0) ||
        (EXTENTCHECK(&reg1->rdh.rcBound, &reg2->rdh.rcBound) == 0))
    {
        newReg->rdh.nCount = 0;
    }
    else
    {
        if (!REGION_RegionOp(newReg,
                        reg1,
                        reg2,
                        REGION_IntersectO,
                        NULL,
                        NULL))
            return FALSE;
    }

    /* Can't alter newReg's extents before we call miRegionOp because
     * it might be one of the source regions and miRegionOp depends
     * on the extents of those regions being the same. Besides, this
     * way there's no checking against rectangles that will be nuked
     * due to coalescing, so we have to examine fewer rectangles. */
    REGION_SetExtents(newReg);
    return TRUE;
}

/***********************************************************************
 * Region Union
 ***********************************************************************/

/*!
 *      Handle a non-overlapping band for the union operation. Just
 *      Adds the rectangles into the region. Doesn't have to check for
 *      subsumption or anything.
 *
 * Results:
 *      None.
 *
 * \note Side Effects:
 *      pReg->numRects is incremented and the final rectangles overwritten
 *      with the rectangles we're passed.
 *
 */
static
BOOL
FASTCALL
REGION_UnionNonO(
    PREGION pReg,
    PRECTL  r,
    PRECTL  rEnd,
    INT     top,
    INT     bottom)
{
    if (r != rEnd)
    {
        if (!REGION_bEnsureBufferSize(pReg, pReg->rdh.nCount + (rEnd - r)))
        {
            return FALSE;
        }

        do
        {
            REGION_vAddRect(pReg, r->left, top, r->right, bottom);
            r++;
        }
        while (r != rEnd);
    }

    return TRUE;
}

static __inline
BOOL
REGION_bMergeRect(
    _Inout_ PREGION prgn,
    _In_ LONG left,
    _In_ LONG top,
    _In_ LONG right,
    _In_ LONG bottom)
{
    if ((prgn->rdh.nCount != 0) &&
        (prgn->Buffer[prgn->rdh.nCount - 1].top == top) &&
        (prgn->Buffer[prgn->rdh.nCount - 1].bottom == bottom) &&
        (prgn->Buffer[prgn->rdh.nCount - 1].right >= left))
    {
        if (prgn->Buffer[prgn->rdh.nCount - 1].right < right)
        {
            prgn->Buffer[prgn->rdh.nCount - 1].right = right;
        }
    }
    else
    {
        if (!REGION_bAddRect(prgn, left, top, right, bottom))
        {
            return FALSE;
        }
    }

    return TRUE;
}

/*!
 *      Handle an overlapping band for the union operation. Picks the
 *      left-most rectangle each time and merges it into the region.
 *
 * Results:
 *      None.
 *
 * \note Side Effects:
 *      Rectangles are overwritten in pReg->rects and pReg->numRects will
 *      be changed.
 *
 */
static
BOOL
FASTCALL
REGION_UnionO (
    PREGION pReg,
    PRECTL  r1,
    PRECTL  r1End,
    PRECTL  r2,
    PRECTL  r2End,
    INT     top,
    INT     bottom)
{
    while ((r1 != r1End) && (r2 != r2End))
    {
        if (r1->left < r2->left)
        {
            if (!REGION_bMergeRect(pReg, r1->left, top, r1->right, bottom)) return FALSE;
            r1++;
        }
        else
        {
            if (!REGION_bMergeRect(pReg, r2->left, top, r2->right, bottom)) return FALSE;
            r2++;
        }
    }

    if (r1 != r1End)
    {
        do
        {
            if (!REGION_bMergeRect(pReg, r1->left, top, r1->right, bottom)) return FALSE;
            r1++;
        }
        while (r1 != r1End);
    }
    else
    {
        while (r2 != r2End)
        {
            if (!REGION_bMergeRect(pReg, r2->left, top, r2->right, bottom)) return FALSE;
            r2++;
        }
    }

    return TRUE;
}

/***********************************************************************
 * REGION_UnionRegion
 */
static
BOOL
FASTCALL
REGION_UnionRegion(
    PREGION newReg,
    PREGION reg1,
    PREGION reg2)
{
    BOOL ret = TRUE;

    /* Checks all the simple cases
     * Region 1 and 2 are the same or region 1 is empty */
    if ((reg1 == reg2) || (reg1->rdh.nCount == 0) ||
        (reg1->rdh.rcBound.right <= reg1->rdh.rcBound.left) ||
        (reg1->rdh.rcBound.bottom <= reg1->rdh.rcBound.top))
    {
        if (newReg != reg2)
        {
            ret = REGION_CopyRegion(newReg, reg2);
        }

        return ret;
    }

    /* If nothing to union (region 2 empty) */
    if ((reg2->rdh.nCount == 0) ||
        (reg2->rdh.rcBound.right <= reg2->rdh.rcBound.left) ||
        (reg2->rdh.rcBound.bottom <= reg2->rdh.rcBound.top))
    {
        if (newReg != reg1)
        {
            ret = REGION_CopyRegion(newReg, reg1);
        }

        return ret;
    }

    /* Region 1 completely subsumes region 2 */
    if ((reg1->rdh.nCount == 1) &&
        (reg1->rdh.rcBound.left <= reg2->rdh.rcBound.left) &&
        (reg1->rdh.rcBound.top <= reg2->rdh.rcBound.top) &&
        (reg2->rdh.rcBound.right <= reg1->rdh.rcBound.right) &&
        (reg2->rdh.rcBound.bottom <= reg1->rdh.rcBound.bottom))
    {
        if (newReg != reg1)
        {
            ret = REGION_CopyRegion(newReg, reg1);
        }

        return ret;
    }

    /* Region 2 completely subsumes region 1 */
    if ((reg2->rdh.nCount == 1) &&
        (reg2->rdh.rcBound.left <= reg1->rdh.rcBound.left) &&
        (reg2->rdh.rcBound.top <= reg1->rdh.rcBound.top) &&
        (reg1->rdh.rcBound.right <= reg2->rdh.rcBound.right) &&
        (reg1->rdh.rcBound.bottom <= reg2->rdh.rcBound.bottom))
    {
        if (newReg != reg2)
        {
            ret = REGION_CopyRegion(newReg, reg2);
        }

        return ret;
    }

    if ((ret = REGION_RegionOp(newReg,
                    reg1,
                    reg2,
                    REGION_UnionO,
                    REGION_UnionNonO,
                    REGION_UnionNonO)))
    {
    newReg->rdh.rcBound.left = min(reg1->rdh.rcBound.left, reg2->rdh.rcBound.left);
    newReg->rdh.rcBound.top = min(reg1->rdh.rcBound.top, reg2->rdh.rcBound.top);
    newReg->rdh.rcBound.right = max(reg1->rdh.rcBound.right, reg2->rdh.rcBound.right);
    newReg->rdh.rcBound.bottom = max(reg1->rdh.rcBound.bottom, reg2->rdh.rcBound.bottom);
    }
    return ret;
}

/***********************************************************************
 * Region Subtraction
 ***********************************************************************/

/*!
 *      Deal with non-overlapping band for subtraction. Any parts from
 *      region 2 we discard. Anything from region 1 we add to the region.
 *
 * Results:
 *      None.
 *
 * \note Side Effects:
 *      pReg may be affected.
 *
 */
static
BOOL
FASTCALL
REGION_SubtractNonO1(
    PREGION pReg,
    PRECTL  r,
    PRECTL  rEnd,
    INT     top,
    INT     bottom)
{
    if (r != rEnd)
    {
        if (!REGION_bEnsureBufferSize(pReg, pReg->rdh.nCount + (rEnd - r)))
        {
            return FALSE;
        }

        do
        {
            REGION_vAddRect(pReg, r->left, top, r->right, bottom);
            r++;
        }
        while (r != rEnd);
    }

    return TRUE;
}

/*!
 *      Overlapping band subtraction. x1 is the left-most point not yet
 *      checked.
 *
 * Results:
 *      None.
 *
 * \note Side Effects:
 *      pReg may have rectangles added to it.
 *
 */
static
BOOL
FASTCALL
REGION_SubtractO(
    PREGION pReg,
    PRECTL  r1,
    PRECTL  r1End,
    PRECTL  r2,
    PRECTL  r2End,
    INT     top,
    INT     bottom)
{
    INT left;

    left = r1->left;

    while ((r1 != r1End) && (r2 != r2End))
    {
        if (r2->right <= left)
        {
            /* Subtrahend missed the boat: go to next subtrahend. */
            r2++;
        }
        else if (r2->left <= left)
        {
            /* Subtrahend preceeds minuend: nuke left edge of minuend. */
            left = r2->right;
            if (left >= r1->right)
            {
                /* Minuend completely covered: advance to next minuend and
                 * reset left fence to edge of new minuend. */
                r1++;
                if (r1 != r1End)
                    left = r1->left;
            }
            else
            {
                /* Subtrahend now used up since it doesn't extend beyond
                 * minuend */
                r2++;
            }
        }
        else if (r2->left < r1->right)
        {
            /* Left part of subtrahend covers part of minuend: add uncovered
             * part of minuend to region and skip to next subtrahend. */
            if (!REGION_bAddRect(pReg, left, top, r2->left, bottom))
            {
                return FALSE;
            }

            left = r2->right;
            if (left >= r1->right)
            {
                /* Minuend used up: advance to new... */
                r1++;
                if (r1 != r1End)
                    left = r1->left;
            }
            else
            {
                /* Subtrahend used up */
                r2++;
            }
        }
        else
        {
            /* Minuend used up: add any remaining piece before advancing. */
            if (r1->right > left)
            {
                if (!REGION_bAddRect(pReg, left, top, r1->right, bottom))
                {
                    return FALSE;
                }
            }

            r1++;
            if (r1 != r1End)
                left = r1->left;
        }
    }

    /* Make sure the buffer is large enough for all remaining operations */
    if (r1 != r1End)
    {
        if (!REGION_bEnsureBufferSize(pReg, pReg->rdh.nCount + (r1End - r1)))
        {
            return FALSE;
        }

        /* Add remaining minuend rectangles to region. */
        do
        {
            REGION_vAddRect(pReg, left, top, r1->right, bottom);
            r1++;
            if (r1 != r1End)
            {
                left = r1->left;
            }
        }
        while (r1 != r1End);
    }

    return TRUE;
}

/*!
 *      Subtract regS from regM and leave the result in regD.
 *      S stands for subtrahend, M for minuend and D for difference.
 *
 * Results:
 *      TRUE.
 *
 * \note Side Effects:
 *      regD is overwritten.
 *
 */
static
BOOL
FASTCALL
REGION_SubtractRegion(
    PREGION regD,
    PREGION regM,
    PREGION regS)
{
    /* Check for trivial reject */
    if ((regM->rdh.nCount == 0) ||
        (regS->rdh.nCount == 0) ||
        (EXTENTCHECK(&regM->rdh.rcBound, &regS->rdh.rcBound) == 0))
    {
        return REGION_CopyRegion(regD, regM);
    }

    if (!REGION_RegionOp(regD,
                    regM,
                    regS,
                    REGION_SubtractO,
                    REGION_SubtractNonO1,
                    NULL))
        return FALSE;

    /* Can't alter newReg's extents before we call miRegionOp because
     * it might be one of the source regions and miRegionOp depends
     * on the extents of those regions being the unaltered. Besides, this
     * way there's no checking against rectangles that will be nuked
     * due to coalescing, so we have to examine fewer rectangles. */
    REGION_SetExtents(regD);
    return TRUE;
}

/***********************************************************************
 * REGION_XorRegion
 */
static
BOOL
FASTCALL
REGION_XorRegion(
    PREGION dr,
    PREGION sra,
    PREGION srb)
{
    HRGN htra, htrb;
    PREGION tra, trb;
    BOOL ret;

    // FIXME: Don't use a handle
    tra = REGION_AllocRgnWithHandle(sra->rdh.nCount + 1);
    if (tra == NULL)
    {
        return FALSE;
    }
    htra = tra->BaseObject.hHmgr;

    // FIXME: Don't use a handle
    trb = REGION_AllocRgnWithHandle(srb->rdh.nCount + 1);
    if (trb == NULL)
    {
        REGION_UnlockRgn(tra);
        GreDeleteObject(htra);
        return FALSE;
    }
    htrb = trb->BaseObject.hHmgr;

    ret = REGION_SubtractRegion(tra, sra, srb) &&
          REGION_SubtractRegion(trb, srb, sra) &&
          REGION_UnionRegion(dr, tra, trb);
    REGION_UnlockRgn(tra);
    REGION_UnlockRgn(trb);

    GreDeleteObject(htra);
    GreDeleteObject(htrb);
    return ret;
}

/*!
 * Adds a rectangle to a REGION
 */
BOOL
FASTCALL
REGION_UnionRectWithRgn(
    PREGION rgn,
    const RECTL *rect)
{
    REGION region;

    region.Buffer = &region.rdh.rcBound;
    region.rdh.nCount = 1;
    region.rdh.nRgnSize = sizeof(RECT);
    region.rdh.rcBound = *rect;
    return REGION_UnionRegion(rgn, rgn, &region);
}

INT
FASTCALL
REGION_SubtractRectFromRgn(
    PREGION prgnDest,
    PREGION prgnSrc,
    const RECTL *prcl)
{
    REGION rgnLocal;

    rgnLocal.Buffer = &rgnLocal.rdh.rcBound;
    rgnLocal.rdh.nCount = 1;
    rgnLocal.rdh.nRgnSize = sizeof(RECT);
    rgnLocal.rdh.rcBound = *prcl;
    REGION_SubtractRegion(prgnDest, prgnSrc, &rgnLocal);
    return REGION_Complexity(prgnDest);
}

BOOL
FASTCALL
REGION_bCopy(
    PREGION dst,
    PREGION src)
{
    if ( !dst || !src ) return FALSE;
    return REGION_CopyRegion( dst, src);
}

BOOL
FASTCALL
REGION_bIntersectRegion(
    PREGION newReg,
    PREGION reg1,
    PREGION reg2)
{
    if ( !newReg || !reg1 || !reg2 ) return FALSE;
    return REGION_IntersectRegion( newReg, reg1, reg2);
}

static
BOOL
REGION_bMakeSimpleFrameRgn(
    _Inout_ PREGION prgn,
    _In_ PRECTL prclSrc,
    _In_ INT cx,
    _In_ INT cy)
{
    RECTL arcl[4];
    UINT i;

    NT_ASSERT((cx >= 0) && (cy >= 0));
    NT_ASSERT((prclSrc->bottom > prclSrc->top) &&
              (prclSrc->right > prclSrc->left));

    /* Start with an empty region */
    EMPTY_REGION(prgn);

    /* Check for the case where the frame covers the whole rect */
    if (((prclSrc->bottom - prclSrc->top) <= cy * 2) ||
        ((prclSrc->right - prclSrc->left) <= cx * 2))
    {
        prgn->rdh.rcBound = *prclSrc;
        prgn->Buffer[0] = *prclSrc;
        prgn->rdh.nCount = 1;
        return TRUE;
    }

    i = 0;

    if (cy != 0)
    {
        /* Top rectangle */
        arcl[i].left = prclSrc->left;
        arcl[i].top = prclSrc->top;
        arcl[i].right = prclSrc->right;
        arcl[i].bottom = prclSrc->top + cy;
        i++;
    }

    if (cx != 0)
    {
        /* Left rectangle */
        arcl[i].left = prclSrc->left;
        arcl[i].top = prclSrc->top + cy;
        arcl[i].right = prclSrc->left + cx;
        arcl[i].bottom = prclSrc->bottom - cy;
        i++;

        /* Right rectangle */
        arcl[i].left = prclSrc->right - cx;
        arcl[i].top = prclSrc->top + cy;
        arcl[i].right = prclSrc->right;
        arcl[i].bottom = prclSrc->bottom - cy;
        i++;
    }

    if (cy != 0)
    {
        /* Bottom rectangle */
        arcl[i].left = prclSrc->left;
        arcl[i].top = prclSrc->bottom - cy;
        arcl[i].right = prclSrc->right;
        arcl[i].bottom = prclSrc->bottom;
        i++;
    }

    if (i != 0)
    {
        /* The frame results in a complex region. rcBounds remains
           the same, though. */
        prgn->rdh.nCount = i;
        NT_ASSERT(prgn->rdh.nCount > 1);
        prgn->rdh.nRgnSize = prgn->rdh.nCount * sizeof(RECT);
        NT_ASSERT(prgn->Buffer == &prgn->rdh.rcBound);
        prgn->Buffer = ExAllocatePoolWithTag(PagedPool,
                                            prgn->rdh.nRgnSize,
                                            TAG_REGION);
        if (prgn->Buffer == NULL)
        {
            prgn->rdh.nRgnSize = 0;
            return FALSE;
        }

        _PRAGMA_WARNING_SUPPRESS(__WARNING_MAYBE_UNINIT_VAR) // arcl is initialized
        COPY_RECTS(prgn->Buffer, arcl, prgn->rdh.nCount);
    }

    return TRUE;
}

static
BOOL
REGION_bMakeFrameRegion(
    _Inout_ PREGION prgnDest,
    _Inout_ PREGION prgnSrc,
    _In_ INT cx,
    _In_ INT cy)
{
    /* Handle negative cx / cy */
    cx = abs(cx);
    cy = abs(cy);

    /* Check border size (the cast is necessary to catch cx/cy == INT_MIN!) */
    if (((UINT)cx > MAX_COORD) || ((UINT)cy > MAX_COORD))
    {
        return FALSE;
    }

    /* Fail on empty source region */
    if (!REGION_NOT_EMPTY(prgnSrc))
    {
        return FALSE;
    }

    /* Handle trivial case */
    if ((cx == 0) && (cy == 0))
    {
        EMPTY_REGION(prgnDest);
        return TRUE;
    }

    /* Handle simple source region */
    if (REGION_Complexity(prgnSrc) == SIMPLEREGION)
    {
        return REGION_bMakeSimpleFrameRgn(prgnDest, &prgnSrc->rdh.rcBound, cx, cy);
    }

    /* Check if we can move the region to create the frame region */
    if ((prgnSrc->rdh.rcBound.left < (MIN_COORD + cx)) ||
        (prgnSrc->rdh.rcBound.top < (MIN_COORD + cy)) ||
        (prgnSrc->rdh.rcBound.right > (MAX_COORD - cx)) ||
        (prgnSrc->rdh.rcBound.bottom > (MAX_COORD - cy)))
    {
        return FALSE;
    }

    /* Copy the source region */
    if (!REGION_CopyRegion(prgnDest, prgnSrc))
    {
        return FALSE;
    }

    /* Move the source region to the bottom-right */
    NT_VERIFY(REGION_bOffsetRgn(prgnSrc, cx, cy));

    /* Intersect with the source region (this crops the top-left frame) */
    REGION_IntersectRegion(prgnDest, prgnDest, prgnSrc);

    /* Move the source region to the bottom-left */
    NT_VERIFY(REGION_bOffsetRgn(prgnSrc, -2 * cx, 0));

    /* Intersect with the source region (this crops the top-right frame) */
    REGION_IntersectRegion(prgnDest, prgnDest, prgnSrc);

    /* Move the source region to the top-left */
    NT_VERIFY(REGION_bOffsetRgn(prgnSrc, 0, -2 * cy));

    /* Intersect with the source region (this crops the bottom-right frame) */
    REGION_IntersectRegion(prgnDest, prgnDest, prgnSrc);

    /* Move the source region to the top-right  */
    NT_VERIFY(REGION_bOffsetRgn(prgnSrc, 2 * cx, 0));

    /* Intersect with the source region (this crops the bottom-left frame) */
    REGION_IntersectRegion(prgnDest, prgnDest, prgnSrc);

    /* Move the source region back to the original position */
    NT_VERIFY(REGION_bOffsetRgn(prgnSrc, -cx, cy));

    /* Finally subtract the cropped region from the source */
    REGION_SubtractRegion(prgnDest, prgnSrc, prgnDest);

    return TRUE;
}

HRGN
FASTCALL
GreCreateFrameRgn(
    HRGN hrgn,
    INT cx,
    INT cy)
{
    PREGION prgnFrame, prgnSrc;
    HRGN hrgnFrame;

    /* Allocate a new region */
    prgnFrame = REGION_AllocUserRgnWithHandle(1);
    if (prgnFrame == NULL)
    {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return NULL;
    }

    /* Lock the source region */
    prgnSrc = REGION_LockRgn(hrgn);
    if (prgnSrc == NULL)
    {
        REGION_Delete(prgnFrame);
        return FALSE;
    }

    if (REGION_bMakeFrameRegion(prgnFrame, prgnSrc, cx, cy))
    {
        hrgnFrame = prgnFrame->BaseObject.hHmgr;
        REGION_UnlockRgn(prgnFrame);
    }
    else
    {
        REGION_Delete(prgnFrame);
        hrgnFrame = NULL;
    }

    REGION_UnlockRgn(prgnSrc);
    return hrgnFrame;
}

BOOL
FASTCALL
REGION_bXformRgn(
    _Inout_ PREGION prgn,
    _In_ PMATRIX pmx)
{
    XFORMOBJ xo;
    ULONG i, cjSize;
    PPOINT ppt;
    PULONG pcPoints;
    RECT rect;
    BOOL bResult;

    /* Check for zero rectangles and return TRUE for translation only matrices */
    if (prgn->rdh.nCount < 1)
        return (pmx->flAccel & XFORM_UNITY) != 0;

    /* Check if this is a scaling only matrix (off-diagonal elements are 0 */
    if (pmx->flAccel & XFORM_SCALE)
    {
        /* Check if this is a translation only matrix */
        if (pmx->flAccel & XFORM_UNITY)
        {
            /* Just offset the region */
            return REGION_bOffsetRgn(prgn, (pmx->fxDx + 8) / 16, (pmx->fxDy + 8) / 16);
        }
        else
        {
            /* Initialize the xform object */
            XFORMOBJ_vInit(&xo, pmx);

            /* Scaling can move the rects out of the coordinate space, so
             * we first need to check whether we can apply the transformation
             * on the bounds rect without modifying the region */
            if (!XFORMOBJ_bApplyXform(&xo, XF_LTOL, 2, &prgn->rdh.rcBound, &rect))
            {
                return FALSE;
            }

            /* Apply the xform to the rects in the region */
            if (!XFORMOBJ_bApplyXform(&xo,
                                      XF_LTOL,
                                      prgn->rdh.nCount * 2,
                                      prgn->Buffer,
                                      prgn->Buffer))
            {
                /* This can not happen, since we already checked the bounds! */
                NT_ASSERT(FALSE);
            }

            /* Reset bounds */
            RECTL_vSetEmptyRect(&prgn->rdh.rcBound);

            /* Loop all rects in the region */
            for (i = 0; i < prgn->rdh.nCount; i++)
            {
                /* Make sure the rect is well-ordered after the xform */
                RECTL_vMakeWellOrdered(&prgn->Buffer[i]);

                /* Update bounds */
                if (!RECTL_bUnionRect(&prgn->rdh.rcBound,
                                 &prgn->rdh.rcBound,
                                 &prgn->Buffer[i]))
                {
                    DPRINT1("NULL Set in Union Rects\n");
                    return FALSE;
                }
            }

            /* Loop all rects in the region */
            for (i = 0; i < prgn->rdh.nCount - 1; i++)
            {
                NT_ASSERT(prgn->Buffer[i].top <= prgn->Buffer[i].bottom);
                NT_ASSERT(prgn->Buffer[i + 1].top >= prgn->Buffer[i].top);
            }

            return TRUE;
        }
    }
    else
    {
        /* Allocate a buffer for the polygons */
        cjSize = prgn->rdh.nCount * (4 * sizeof(POINT) + sizeof(ULONG));
        ppt = ExAllocatePoolWithTag(PagedPool, cjSize, GDITAG_REGION);
        if (ppt == NULL)
        {
            return FALSE;
        }

        /* Fill the buffer with the rects */
        pcPoints = (PULONG)&ppt[4 * prgn->rdh.nCount];
        for (i = 0; i < prgn->rdh.nCount; i++)
        {
            /* Make sure the rect is within the legal range */
            pcPoints[i] = 4;
            ppt[4 * i + 0].x = prgn->Buffer[i].left;
            ppt[4 * i + 0].y = prgn->Buffer[i].top;
            ppt[4 * i + 1].x = prgn->Buffer[i].right;
            ppt[4 * i + 1].y = prgn->Buffer[i].top;
            ppt[4 * i + 2].x = prgn->Buffer[i].right;
            ppt[4 * i + 2].y = prgn->Buffer[i].bottom;
            ppt[4 * i + 3].x = prgn->Buffer[i].left;
            ppt[4 * i + 3].y = prgn->Buffer[i].bottom;
        }

        /* Initialize the xform object */
        XFORMOBJ_vInit(&xo, pmx);

        /* Apply the xform to the rects in the buffer */
        if (!XFORMOBJ_bApplyXform(&xo,
                                  XF_LTOL,
                                  prgn->rdh.nCount * 2,
                                  ppt,
                                  ppt))
        {
            /* This means, there were coordinates that would go outside of
               the coordinate space after the transformation */
            ExFreePoolWithTag(ppt, GDITAG_REGION);
            return FALSE;
        }

        /* Now use the polygons to create a polygon region */
        bResult = REGION_SetPolyPolygonRgn(prgn,
                                           ppt,
                                           pcPoints,
                                           prgn->rdh.nCount,
                                           WINDING);

        /* Free the polygon buffer */
        ExFreePoolWithTag(ppt, GDITAG_REGION);

        return bResult;
    }
}

PREGION
FASTCALL
REGION_AllocRgnWithHandle(
    INT nReg)
{
    //HRGN hReg;
    PREGION pReg;

    pReg = (PREGION)GDIOBJ_AllocateObject(GDIObjType_RGN_TYPE,
                                          sizeof(REGION),
                                          BASEFLAG_LOOKASIDE);
    if (pReg == NULL)
    {
        DPRINT1("Could not allocate a palette.\n");
        return NULL;
    }

    //hReg = pReg->BaseObject.hHmgr;

    if ((nReg == 0) || (nReg == 1))
    {
        /* Testing shows that > 95% of all regions have only 1 rect.
           Including that here saves us from having to do another allocation */
        pReg->Buffer = &pReg->rdh.rcBound;
    }
    else
    {
        pReg->Buffer = ExAllocatePoolWithTag(PagedPool,
                                             nReg * sizeof(RECT),
                                             TAG_REGION);
        if (pReg->Buffer == NULL)
        {
            DPRINT1("Could not allocate region buffer\n");
            GDIOBJ_vDeleteObject(&pReg->BaseObject);
            return NULL;
        }
    }

    EMPTY_REGION(pReg);
    pReg->rdh.dwSize = sizeof(RGNDATAHEADER);
    pReg->rdh.nCount = nReg;
    pReg->rdh.nRgnSize = nReg * sizeof(RECT);
    pReg->prgnattr = &pReg->rgnattr;

    /* Initialize the region attribute */
    pReg->rgnattr.AttrFlags = 0;
    pReg->rgnattr.iComplexity = SIMPLEREGION;
    pReg->rgnattr.Rect = pReg->rdh.rcBound;

    /* Finally insert the region into the handle table */
    if (!GDIOBJ_hInsertObject(&pReg->BaseObject, GDI_OBJ_HMGR_POWNED))
    {
        DPRINT1("Could not insert palette into handle table.\n");
        GDIOBJ_vFreeObject(&pReg->BaseObject);
        return NULL;
    }

    return pReg;
}

BOOL
NTAPI
REGION_bAllocRgnAttr(
    PREGION prgn)
{
    PPROCESSINFO ppi;
    PRGN_ATTR prgnattr;

    NT_ASSERT(prgn->prgnattr == &prgn->rgnattr);

    ppi = PsGetCurrentProcessWin32Process();
    ASSERT(ppi);

    prgnattr = GdiPoolAllocate(ppi->pPoolRgnAttr);
    if (prgnattr == NULL)
    {
        DPRINT1("Could not allocate RGN attr\n");
        return FALSE;
    }

    /* Copy the current region attribute */
    *prgnattr = prgn->rgnattr;

    /* Set the object attribute in the handle table */
    prgn->prgnattr = prgnattr;
    GDIOBJ_vSetObjectAttr(&prgn->BaseObject, prgnattr);

    return TRUE;
}