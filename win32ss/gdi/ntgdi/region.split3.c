/* region.split3.c — #include fragment of region.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original region.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


//
// Allocate User Space Region Handle.
//
PREGION
FASTCALL
REGION_AllocUserRgnWithHandle(
    INT nRgn)
{
    PREGION prgn;

    prgn = REGION_AllocRgnWithHandle(nRgn);
    if (prgn == NULL)
    {
        return NULL;
    }

    if (!REGION_bAllocRgnAttr(prgn))
    {
        ASSERT(FALSE);
    }

    return prgn;
}

static
VOID
REGION_vSyncRegion(
    _In_ PREGION prgn)
{
    PRGN_ATTR prgnattr;

    NT_ASSERT(prgn != NULL);
    NT_ASSERT(prgn->prgnattr != NULL);
    NT_ASSERT((prgn->prgnattr == &prgn->rgnattr) ||
              (prgn->prgnattr->AttrFlags & ATTR_RGN_VALID));

    /* Get the region attribute and check if it's dirty (modified) */
    prgnattr = prgn->prgnattr;
    if (prgnattr->AttrFlags & ATTR_RGN_DIRTY)
    {
        NT_ASSERT(GreGetObjectOwner(prgn->BaseObject.hHmgr) == GDI_OBJ_HMGR_POWNED);
        NT_ASSERT(prgnattr != &prgn->rgnattr);

        if (prgnattr->iComplexity == NULLREGION)
        {
            EMPTY_REGION(prgn);
        }
        else if (prgnattr->iComplexity == SIMPLEREGION)
        {
            REGION_SetRectRgn(prgn,
                              prgnattr->Rect.left,
                              prgnattr->Rect.top,
                              prgnattr->Rect.right,
                              prgnattr->Rect.bottom);
        }
        else
        {
            /* Should not happen, region attribute is corrupted! */
            DPRINT1("Region attribute is corrupted, ignoring\n");
            NT_ASSERT(FALSE);
        }
    }

    /* Reset the flags */
    prgnattr->AttrFlags &= ~(ATTR_RGN_DIRTY | ATTR_RGN_VALID);
}

PREGION
FASTCALL
REGION_LockRgn(
    _In_ HRGN hrgn)
{
    PREGION prgn;

    prgn = GDIOBJ_LockObject(hrgn, GDIObjType_RGN_TYPE);
    if (prgn == NULL)
        return NULL;

    REGION_vSyncRegion(prgn);
    return prgn;
}

VOID
FASTCALL
REGION_UnlockRgn(
    _In_ PREGION prgn)
{
    PRGN_ATTR prgnattr;

    NT_ASSERT(prgn != NULL);
    NT_ASSERT(prgn->prgnattr != NULL);

    /* Get the region attribute and check if it's user mode */
    prgnattr = prgn->prgnattr;
    if (prgnattr != &prgn->rgnattr)
    {
        NT_ASSERT(GreGetObjectOwner(prgn->BaseObject.hHmgr) == GDI_OBJ_HMGR_POWNED);
        prgnattr->iComplexity = REGION_Complexity(prgn);
        prgnattr->Rect.left   = prgn->rdh.rcBound.left;
        prgnattr->Rect.top    = prgn->rdh.rcBound.top;
        prgnattr->Rect.right  = prgn->rdh.rcBound.right;
        prgnattr->Rect.bottom = prgn->rdh.rcBound.bottom;
        prgnattr->AttrFlags |= ATTR_RGN_VALID;
    }

    GDIOBJ_vUnlockObject(&prgn->BaseObject);
}

/*
  System Regions:
    These regions do not use attribute sections and when allocated, use gdiobj
    level functions.
*/
//
// System Region Functions
//
PREGION
FASTCALL
IntSysCreateRectpRgn(
    INT LeftRect,
    INT TopRect,
    INT RightRect,
    INT BottomRect)
{
    PREGION prgn;

    /* Allocate a region, without a handle */
    prgn = (PREGION)GDIOBJ_AllocateObject(GDIObjType_RGN_TYPE, sizeof(REGION), BASEFLAG_LOOKASIDE);
    if (prgn == NULL)
    {
        return NULL;
    }

    /* Initialize it */
    prgn->Buffer = &prgn->rdh.rcBound;
    prgn->prgnattr = &prgn->rgnattr;
    prgn->prgnattr->AttrFlags = ATTR_RGN_VALID;
    REGION_SetRectRgn(prgn, LeftRect, TopRect, RightRect, BottomRect);

    return prgn;
}

VOID
NTAPI
REGION_vCleanup(PVOID ObjectBody)
{
    PREGION pRgn = (PREGION)ObjectBody;
    PPROCESSINFO ppi = PsGetCurrentProcessWin32Process();
    ASSERT(ppi);

    ASSERT(pRgn->prgnattr);
    if (pRgn->prgnattr != &pRgn->rgnattr)
        GdiPoolFree(ppi->pPoolRgnAttr, pRgn->prgnattr);

    if (pRgn->Buffer && pRgn->Buffer != &pRgn->rdh.rcBound)
        ExFreePoolWithTag(pRgn->Buffer, TAG_REGION);
}

VOID
FASTCALL
REGION_Delete(PREGION pRgn)
{
    if (pRgn == prgnDefault)
        return;

    GDIOBJ_vDeleteObject(&pRgn->BaseObject);
}

BOOL
FASTCALL
IntGdiSetRegionOwner(HRGN hRgn, DWORD OwnerMask)
{
    PREGION prgn;
    PRGN_ATTR prgnattr;
    PPROCESSINFO ppi;

    prgn = REGION_LockRgn(hRgn);
    if (prgn == NULL)
    {
        return FALSE;
    }

    prgnattr = prgn->prgnattr;
    if (prgnattr != &prgn->rgnattr)
    {
        GDIOBJ_vSetObjectAttr(&prgn->BaseObject, NULL);
        prgn->prgnattr = &prgn->rgnattr;
        ppi = PsGetCurrentProcessWin32Process();
        GdiPoolFree(ppi->pPoolRgnAttr, prgnattr);
    }

    REGION_UnlockRgn(prgn);

    return GreSetObjectOwner(hRgn, OwnerMask);
}

INT
FASTCALL
IntGdiCombineRgn(
    PREGION prgnDest,
    PREGION prgnSrc1,
    PREGION prgnSrc2,
    INT iCombineMode)
{
    BOOL Ret = TRUE;

    if (prgnDest == NULL)
    {
        DPRINT("IntGdiCombineRgn: hDest unavailable\n");
        return ERROR;
    }

    if (prgnSrc1 == NULL)
    {
        DPRINT("IntGdiCombineRgn: hSrc1 unavailable\n");
        return ERROR;
    }

    if (iCombineMode == RGN_COPY)
    {
        if (!REGION_CopyRegion(prgnDest, prgnSrc1))
            return ERROR;

        return REGION_Complexity(prgnDest);
    }

    if (prgnSrc2 == NULL)
    {
        DPRINT1("IntGdiCombineRgn requires hSrc2 != NULL for combine mode %d!\n", iCombineMode);
        ASSERT(FALSE);
        return ERROR;
    }

    switch (iCombineMode)
    {
        case RGN_AND:
            Ret = REGION_IntersectRegion(prgnDest, prgnSrc1, prgnSrc2);
            break;
        case RGN_OR:
            Ret = REGION_UnionRegion(prgnDest, prgnSrc1, prgnSrc2);
            break;
        case RGN_XOR:
            Ret = REGION_XorRegion(prgnDest, prgnSrc1, prgnSrc2);
            break;
        case RGN_DIFF:
            Ret = REGION_SubtractRegion(prgnDest, prgnSrc1, prgnSrc2);
            break;
    }

    return Ret ? REGION_Complexity(prgnDest) : ERROR;
}

INT
FASTCALL
REGION_GetRgnBox(
    PREGION Rgn,
    PRECTL pRect)
{
    DWORD ret;

    if (Rgn != NULL)
    {
        *pRect = Rgn->rdh.rcBound;
        ret = REGION_Complexity(Rgn);

        return ret;
    }
    return 0; // If invalid region return zero
}

INT
APIENTRY
IntGdiGetRgnBox(
    HRGN hRgn,
    PRECTL pRect)
{
    PREGION Rgn;
    DWORD ret;

    Rgn = REGION_LockRgn(hRgn);
    if (Rgn == NULL)
    {
        return ERROR;
    }

    ret = REGION_GetRgnBox(Rgn, pRect);
    REGION_UnlockRgn(Rgn);

    return ret;
}

BOOL
FASTCALL
REGION_PtInRegion(
    PREGION prgn,
    INT X,
    INT Y)
{
    ULONG i;
    PRECT r;

    if (prgn->rdh.nCount > 0 && INRECT(prgn->rdh.rcBound, X, Y))
    {
        r =  prgn->Buffer;
        for (i = 0; i < prgn->rdh.nCount; i++)
        {
            if (INRECT(r[i], X, Y))
                return TRUE;
        }
    }

    return FALSE;
}

BOOL
FASTCALL
REGION_RectInRegion(
    PREGION Rgn,
    const RECTL *rect)
{
    PRECTL pCurRect, pRectEnd;
    RECT rc;

    /* Swap the coordinates to make right >= left and bottom >= top */
    /* (region building rectangles are normalized the same way) */
    if (rect->top > rect->bottom)
    {
        rc.top = rect->bottom;
        rc.bottom = rect->top;
    }
    else
    {
        rc.top = rect->top;
        rc.bottom = rect->bottom;
    }

    if (rect->right < rect->left)
    {
        rc.right = rect->left;
        rc.left = rect->right;
    }
    else
    {
        rc.right = rect->right;
        rc.left = rect->left;
    }

    /* This is (just) a useful optimization */
    if ((Rgn->rdh.nCount > 0) && EXTENTCHECK(&Rgn->rdh.rcBound, &rc))
    {
        for (pCurRect = Rgn->Buffer, pRectEnd = pCurRect +
                                                Rgn->rdh.nCount; pCurRect < pRectEnd; pCurRect++)
        {
            if (pCurRect->bottom <= rc.top)
                continue;             /* Not far enough down yet */

            if (pCurRect->top >= rc.bottom)
                break;                /* Too far down */

            if (pCurRect->right <= rc.left)
                continue;              /* Not far enough over yet */

            if (pCurRect->left >= rc.right)
            {
                continue;
            }

            return TRUE;
        }
    }

    return FALSE;
}

VOID
FASTCALL
REGION_SetRectRgn(
    PREGION rgn,
    INT LeftRect,
    INT TopRect,
    INT RightRect,
    INT BottomRect)
{
    PRECTL firstRect;

    if (LeftRect > RightRect)
    {
        INT tmp = LeftRect;
        LeftRect = RightRect;
        RightRect = tmp;
    }

    if (TopRect > BottomRect)
    {
        INT tmp = TopRect;
        TopRect = BottomRect;
        BottomRect = tmp;
    }

    if ((LeftRect != RightRect) && (TopRect != BottomRect))
    {
        firstRect = rgn->Buffer;
        ASSERT(firstRect);
        firstRect->left = rgn->rdh.rcBound.left = LeftRect;
        firstRect->top = rgn->rdh.rcBound.top = TopRect;
        firstRect->right = rgn->rdh.rcBound.right = RightRect;
        firstRect->bottom = rgn->rdh.rcBound.bottom = BottomRect;
        rgn->rdh.nCount = 1;
        rgn->rdh.iType = RDH_RECTANGLES;
    }
    else
    {
        EMPTY_REGION(rgn);
    }
}

BOOL
FASTCALL
REGION_bOffsetRgn(
    _Inout_ PREGION prgn,
    _In_ INT cx,
    _In_ INT cy)
{
    PRECTL prcl;
    UINT i;

    NT_ASSERT(prgn != NULL);

    /* Check for trivial case */
    if ((cx == 0) && (cy == 0))
    {
        return TRUE;
    }

    /* Check for empty regions, we ignore the offset values here */
    if (prgn->rdh.nCount == 0)
    {
        return TRUE;
    }

    /* Make sure the offset is within the legal range */
    if ((cx > MAX_COORD) || (cx < MIN_COORD) ||
        (cy > MAX_COORD) || (cy < MIN_COORD))
    {
        return FALSE;
    }

    /* Are we moving right? */
    if (cx > 0)
    {
        /* Check if we stay inside the bounds on the right side */
        if (prgn->rdh.rcBound.right > (MAX_COORD - cx))
        {
            return FALSE;
        }
    }
    else
    {
        /* Check if we stay inside the bounds on the left side */
        if (prgn->rdh.rcBound.left < (MIN_COORD - cx))
        {
            return FALSE;
        }
    }

    /* Are we moving down? */
    if (cy > 0)
    {
        /* Check if we stay inside the bounds on the right side */
        if (prgn->rdh.rcBound.bottom > (MAX_COORD - cy))
        {
            return FALSE;
        }
    }
    else
    {
        /* Check if we stay inside the bounds on the left side */
        if (prgn->rdh.rcBound.top < (MIN_COORD - cy))
        {
            return FALSE;
        }
    }

    /* Loop to move the rects */
    prcl = prgn->Buffer;
    for (i = 0; i < prgn->rdh.nCount; i++)
    {
        prcl[i].left += cx;
        prcl[i].right += cx;
        prcl[i].top += cy;
        prcl[i].bottom += cy;
    }

    /* Finally update the bounds rect */
    if (prgn->Buffer != &prgn->rdh.rcBound)
    {
        prgn->rdh.rcBound.left += cx;
        prgn->rdh.rcBound.right += cx;
        prgn->rdh.rcBound.top += cy;
        prgn->rdh.rcBound.bottom += cy;
    }

    return TRUE;
}

/***********************************************************************
 *     REGION_InsertEdgeInET
 *
 *     Insert the given edge into the edge table.
 *     First we must find the correct bucket in the
 *     Edge table, then find the right slot in the
 *     bucket.  Finally, we can insert it.
 *
 */
static
VOID
FASTCALL
REGION_InsertEdgeInET(
    EDGE_TABLE *ET,
    EDGE_TABLE_ENTRY *ETE,
    INT scanline,
    SCANLINE_LISTBLOCK **SLLBlock,
    INT *iSLLBlock)
{
    EDGE_TABLE_ENTRY *start, *prev;
    SCANLINE_LIST *pSLL, *pPrevSLL;
    SCANLINE_LISTBLOCK *tmpSLLBlock;

    /* Find the right bucket to put the edge into */
    pPrevSLL = &ET->scanlines;
    pSLL = pPrevSLL->next;
    while (pSLL && (pSLL->scanline < scanline))
    {
        pPrevSLL = pSLL;
        pSLL = pSLL->next;
    }

    /* Reassign pSLL (pointer to SCANLINE_LIST) if necessary */
    if ((!pSLL) || (pSLL->scanline > scanline))
    {
        if (*iSLLBlock > SLLSPERBLOCK-1)
        {
            tmpSLLBlock = ExAllocatePoolWithTag(PagedPool,
                                                sizeof(SCANLINE_LISTBLOCK),
                                                TAG_REGION);
            if (tmpSLLBlock == NULL)
            {
                DPRINT1("REGION_InsertEdgeInETL(): Can't alloc SLLB\n");
                /* FIXME: Free resources? */
                return;
            }

            (*SLLBlock)->next = tmpSLLBlock;
            tmpSLLBlock->next = (SCANLINE_LISTBLOCK *)NULL;
            *SLLBlock = tmpSLLBlock;
            *iSLLBlock = 0;
        }

        pSLL = &((*SLLBlock)->SLLs[(*iSLLBlock)++]);

        pSLL->next = pPrevSLL->next;
        pSLL->edgelist = (EDGE_TABLE_ENTRY *)NULL;
        pPrevSLL->next = pSLL;
    }

    pSLL->scanline = scanline;

    /* Now insert the edge in the right bucket */
    prev = (EDGE_TABLE_ENTRY *)NULL;
    start = pSLL->edgelist;
    while (start && (start->bres.minor_axis < ETE->bres.minor_axis))
    {
        prev = start;
        start = start->next;
    }

    ETE->next = start;

    if (prev)
        prev->next = ETE;
    else
        pSLL->edgelist = ETE;
}

/***********************************************************************
 *     REGION_loadAET
 *
 *     This routine moves EDGE_TABLEEntries from the
 *     EDGE_TABLE into the Active Edge Table,
 *     leaving them sorted by smaller x coordinate.
 *
 */
static
VOID
FASTCALL
REGION_loadAET(
    EDGE_TABLE_ENTRY *AET,
    EDGE_TABLE_ENTRY *ETEs)
{
    EDGE_TABLE_ENTRY *pPrevAET;
    EDGE_TABLE_ENTRY *tmp;

    pPrevAET = AET;
    AET = AET->next;
    while (ETEs)
    {
        while (AET && (AET->bres.minor_axis < ETEs->bres.minor_axis))
        {
            pPrevAET = AET;
            AET = AET->next;
        }

        tmp = ETEs->next;
        ETEs->next = AET;
        if (AET)
            AET->back = ETEs;

        ETEs->back = pPrevAET;
        pPrevAET->next = ETEs;
        pPrevAET = ETEs;

        ETEs = tmp;
    }
}

/***********************************************************************
 *     REGION_computeWAET
 *
 *     This routine links the AET by the
 *     nextWETE (winding EDGE_TABLE_ENTRY) link for
 *     use by the winding number rule.  The final
 *     Active Edge Table (AET) might look something
 *     like:
 *
 *     AET
 *     ----------  ---------   ---------
 *     |ymax    |  |ymax    |  |ymax    |
 *     | ...    |  |...     |  |...     |
 *     |next    |->|next    |->|next    |->...
 *     |nextWETE|  |nextWETE|  |nextWETE|
 *     ---------   ---------   ^--------
 *         |                   |       |
 *         V------------------->       V---> ...
 *
 */
static
VOID
FASTCALL
REGION_computeWAET(
    EDGE_TABLE_ENTRY *AET)
{
    register EDGE_TABLE_ENTRY *pWETE;
    register INT inside = 1;
    register INT isInside = 0;

    AET->nextWETE = (EDGE_TABLE_ENTRY *)NULL;
    pWETE = AET;
    AET = AET->next;
    while (AET)
    {
        if (AET->ClockWise)
            isInside++;
        else
            isInside--;

        if ((!inside && !isInside) ||
            ( inside &&  isInside))
        {
            pWETE->nextWETE = AET;
            pWETE = AET;
            inside = !inside;
        }
        AET = AET->next;
    }

    pWETE->nextWETE = (EDGE_TABLE_ENTRY *)NULL;
}

/***********************************************************************
 *     REGION_InsertionSort
 *
 *     Just a simple insertion sort using
 *     pointers and back pointers to sort the Active
 *     Edge Table.
 *
 */
static
BOOL
FASTCALL
REGION_InsertionSort(
    EDGE_TABLE_ENTRY *AET)
{
    EDGE_TABLE_ENTRY *pETEchase;
    EDGE_TABLE_ENTRY *pETEinsert;
    EDGE_TABLE_ENTRY *pETEchaseBackTMP;
    BOOL changed = FALSE;

    AET = AET->next;
    while (AET)
    {
        pETEinsert = AET;
        pETEchase = AET;
        while (pETEchase->back->bres.minor_axis > AET->bres.minor_axis)
            pETEchase = pETEchase->back;

        AET = AET->next;
        if (pETEchase != pETEinsert)
        {
            pETEchaseBackTMP = pETEchase->back;
            pETEinsert->back->next = AET;
            if (AET)
                AET->back = pETEinsert->back;

            pETEinsert->next = pETEchase;
            pETEchase->back->next = pETEinsert;
            pETEchase->back = pETEinsert;
            pETEinsert->back = pETEchaseBackTMP;
            changed = TRUE;
        }
    }

    return changed;
}

/***********************************************************************
 *     REGION_FreeStorage
 *
 *     Clean up our act.
 */
static
VOID
FASTCALL
REGION_FreeStorage(
    SCANLINE_LISTBLOCK *pSLLBlock)
{
    SCANLINE_LISTBLOCK   *tmpSLLBlock;

    while (pSLLBlock)
    {
        tmpSLLBlock = pSLLBlock->next;
        ExFreePoolWithTag(pSLLBlock, TAG_REGION);
        pSLLBlock = tmpSLLBlock;
    }
}

/***********************************************************************
 *     REGION_PtsToRegion
 *
 *     Create an array of rectangles from a list of points.
 */
static
INT
FASTCALL
REGION_PtsToRegion(
    INT numFullPtBlocks,
    INT iCurPtBlock,
    POINTBLOCK *FirstPtBlock,
    PREGION reg)
{
    RECTL *rects;
    POINT *pts;
    POINTBLOCK *CurPtBlock;
    INT i;
    RECTL *extents, *temp;
    INT numRects;

    extents = &reg->rdh.rcBound;

    numRects = ((numFullPtBlocks * NUMPTSTOBUFFER) + iCurPtBlock) >> 1;

    /* Make sure, we have at least one rect */
    if (numRects == 0)
    {
        numRects = 1;
    }

    temp = ExAllocatePoolWithTag(PagedPool, numRects * sizeof(RECT), TAG_REGION);
    if (temp == NULL)
    {
        return 0;
    }

    if (reg->Buffer != NULL)
    {
        COPY_RECTS(temp, reg->Buffer, reg->rdh.nCount);
        if (reg->Buffer != &reg->rdh.rcBound)
            ExFreePoolWithTag(reg->Buffer, TAG_REGION);
    }
    reg->Buffer = temp;

    reg->rdh.nCount = numRects;
    CurPtBlock = FirstPtBlock;
    rects = reg->Buffer - 1;
    numRects = 0;
    extents->left = LARGE_COORDINATE,  extents->right = SMALL_COORDINATE;

    for ( ; numFullPtBlocks >= 0; numFullPtBlocks--)
    {
        /* The loop uses 2 points per iteration */
        i = NUMPTSTOBUFFER >> 1;
        if (numFullPtBlocks == 0)
            i = iCurPtBlock >> 1;

        for (pts = CurPtBlock->pts; i--; pts += 2)
        {
            if (pts->x == pts[1].x)
                continue;

            if ((numRects && pts->x == rects->left) &&
                (pts->y == rects->bottom) &&
                (pts[1].x == rects->right) &&
                ((numRects == 1) || (rects[-1].top != rects->top)) &&
                (i && pts[2].y > pts[1].y))
            {
                rects->bottom = pts[1].y + 1;
                continue;
            }

            numRects++;
            rects++;
            rects->left = pts->x;
            rects->top = pts->y;
            rects->right = pts[1].x;
            rects->bottom = pts[1].y + 1;

            if (rects->left < extents->left)
                extents->left = rects->left;
            if (rects->right > extents->right)
                extents->right = rects->right;
        }

        CurPtBlock = CurPtBlock->next;
    }

    if (numRects)
    {
        extents->top = reg->Buffer->top;
        extents->bottom = rects->bottom;
    }
    else
    {
        extents->left = 0;
        extents->top = 0;
        extents->right = 0;
        extents->bottom = 0;
    }

    reg->rdh.nCount = numRects;

    return(TRUE);
}

/***********************************************************************
 *     REGION_CreateETandAET
 *
 *     This routine creates the edge table for
 *     scan converting polygons.
 *     The Edge Table (ET) looks like:
 *
 *    EDGE_TABLE
 *     --------
 *    |  ymax  |        SCANLINE_LISTs
 *    |scanline|-->------------>-------------->...
 *     --------   |scanline|   |scanline|
 *                |edgelist|   |edgelist|
 *                ---------    ---------
 *                    |             |
 *                    |             |
 *                    V             V
 *              list of ETEs   list of ETEs
 *
 *     where ETE is an EDGE_TABLE_ENTRY data structure,
 *     and there is one SCANLINE_LIST per scanline at
 *     which an edge is initially entered.
 *
 */
static
VOID
FASTCALL
REGION_CreateETandAET(
    const ULONG *Count,
    INT nbpolygons,
    const POINT *pts,
    EDGE_TABLE *ET,
    EDGE_TABLE_ENTRY *AET,
    EDGE_TABLE_ENTRY *pETEs,
    SCANLINE_LISTBLOCK *pSLLBlock)
{
    const POINT *top, *bottom;
    const POINT *PrevPt, *CurrPt, *EndPt;
    INT poly, count;
    INT iSLLBlock = 0;
    INT dy;

    /* Initialize the Active Edge Table */
    AET->next = (EDGE_TABLE_ENTRY *)NULL;
    AET->back = (EDGE_TABLE_ENTRY *)NULL;
    AET->nextWETE = (EDGE_TABLE_ENTRY *)NULL;
    AET->bres.minor_axis = SMALL_COORDINATE;

    /* Initialize the Edge Table. */
    ET->scanlines.next = (SCANLINE_LIST *)NULL;
    ET->ymax = SMALL_COORDINATE;
    ET->ymin = LARGE_COORDINATE;
    pSLLBlock->next = (SCANLINE_LISTBLOCK *)NULL;

    EndPt = pts - 1;
    for (poly = 0; poly < nbpolygons; poly++)
    {
        count = Count[poly];
        EndPt += count;
        if (count < 2)
            continue;

        PrevPt = EndPt;

        /*  For each vertex in the array of points.
         *  In this loop we are dealing with two vertices at
         *  a time -- these make up one edge of the polygon. */
        while (count--)
        {
            CurrPt = pts++;

            /*  Find out which point is above and which is below. */
            if (PrevPt->y > CurrPt->y)
            {
                bottom = PrevPt, top = CurrPt;
                pETEs->ClockWise = 0;
            }
            else
            {
                bottom = CurrPt, top = PrevPt;
                pETEs->ClockWise = 1;
            }

            /* Don't add horizontal edges to the Edge table. */
            if (bottom->y != top->y)
            {
                /* -1 so we don't get last scanline */
                pETEs->ymax = bottom->y - 1;

                /*  Initialize integer edge algorithm */
                dy = bottom->y - top->y;
                BRESINITPGONSTRUCT(dy, top->x, bottom->x, pETEs->bres);

                REGION_InsertEdgeInET(ET,
                                      pETEs,
                                      top->y,
                                      &pSLLBlock,
                                      &iSLLBlock);

                if (PrevPt->y > ET->ymax)
                    ET->ymax = PrevPt->y;
                if (PrevPt->y < ET->ymin)
                    ET->ymin = PrevPt->y;
                pETEs++;
            }

            PrevPt = CurrPt;
        }
    }
}

BOOL
FASTCALL
REGION_SetPolyPolygonRgn(
    _Inout_ PREGION prgn,
    _In_ const POINT *ppt,
    _In_ const ULONG *pcPoints,
    _In_ ULONG cPolygons,
    _In_ INT iMode)
{
    EDGE_TABLE_ENTRY *pAET;               /* Active Edge Table        */
    INT y;                                /* Current scanline         */
    INT iPts = 0;                         /* Number of pts in buffer  */
    EDGE_TABLE_ENTRY *pWETE;              /* Winding Edge Table Entry */
    SCANLINE_LIST *pSLL;                  /* Current SCANLINE_LIST    */
    POINT *pts;                           /* Output buffer            */
    EDGE_TABLE_ENTRY *pPrevAET;           /* Pointer to previous AET  */
    EDGE_TABLE ET;                        /* Header node for ET       */
    EDGE_TABLE_ENTRY AET;                 /* Header node for AET      */
    EDGE_TABLE_ENTRY *pETEs;              /* EDGE_TABLEEntries pool   */
    SCANLINE_LISTBLOCK SLLBlock;          /* Header for SCANLINE_LIST */
    INT fixWAET = FALSE;
    POINTBLOCK FirstPtBlock, *curPtBlock; /* PtBlock buffers          */
    POINTBLOCK *tmpPtBlock;
    UINT numFullPtBlocks = 0;
    UINT poly, total;
    BOOL bResult = FALSE;

    /* Check if iMode is valid */
    if ((iMode != ALTERNATE) && (iMode != WINDING))
    {
        DPRINT1("Invalid iMode: %lu\n", iMode);
        return FALSE;
    }

    /* Special case a rectangle */
    if (((cPolygons == 1) && ((pcPoints[0] == 4) ||
         ((pcPoints[0] == 5) && (ppt[4].x == ppt[0].x) && (ppt[4].y == ppt[0].y)))) &&
        (((ppt[0].y == ppt[1].y) &&
          (ppt[1].x == ppt[2].x) &&
          (ppt[2].y == ppt[3].y) &&
          (ppt[3].x == ppt[0].x)) ||
         ((ppt[0].x == ppt[1].x) &&
          (ppt[1].y == ppt[2].y) &&
          (ppt[2].x == ppt[3].x) &&
          (ppt[3].y == ppt[0].y))))
    {
        REGION_SetRectRgn(prgn,
                          min(ppt[0].x, ppt[2].x),
                          min(ppt[0].y, ppt[2].y),
                          max(ppt[0].x, ppt[2].x),
                          max(ppt[0].y, ppt[2].y));
        return TRUE;
    }

    for (poly = total = 0; poly < cPolygons; poly++)
        total += pcPoints[poly];

    pETEs = ExAllocatePoolWithTag(PagedPool,
                                  sizeof(EDGE_TABLE_ENTRY) * total,
                                  TAG_REGION);
    if (pETEs == NULL)
    {
        DPRINT1("Failed to allocate %lu edge entries\n", total);
        return FALSE;
    }

    pts = FirstPtBlock.pts;
    REGION_CreateETandAET(pcPoints, cPolygons, ppt, &ET, &AET, pETEs, &SLLBlock);
    pSLL = ET.scanlines.next;
    curPtBlock = &FirstPtBlock;

    if (iMode != WINDING)
    {
        /*  For each scanline */
        for (y = ET.ymin; y < ET.ymax; y++)
        {
            /*  Add a new edge to the active edge table when we
             *  get to the next edge. */
            if (pSLL != NULL && y == pSLL->scanline)
            {
                REGION_loadAET(&AET, pSLL->edgelist);
                pSLL = pSLL->next;
            }
            pPrevAET = &AET;
            pAET = AET.next;

            /*  For each active edge */
            while (pAET)
            {
                pts->x = pAET->bres.minor_axis,  pts->y = y;
                pts++, iPts++;

                /* Send out the buffer */
                if (iPts == NUMPTSTOBUFFER)
                {
                    tmpPtBlock = ExAllocatePoolWithTag(PagedPool,
                                                       sizeof(POINTBLOCK),
                                                       TAG_REGION);
                    if (tmpPtBlock == NULL)
                    {
                        DPRINT1("Can't alloc tmpPtBlock\n");
                        goto Cleanup;
                    }

                    curPtBlock->next = tmpPtBlock;
                    curPtBlock = tmpPtBlock;
                    pts = curPtBlock->pts;
                    numFullPtBlocks++;
                    iPts = 0;
                }

                EVALUATEEDGEEVENODD(pAET, pPrevAET, y);
            }

            REGION_InsertionSort(&AET);
        }
    }
    else
    {
        /* For each scanline */
        for (y = ET.ymin; y < ET.ymax; y++)
        {
            /*  Add a new edge to the active edge table when we
             *  get to the next edge. */
            if (pSLL != NULL && y == pSLL->scanline)
            {
                REGION_loadAET(&AET, pSLL->edgelist);
                REGION_computeWAET(&AET);
                pSLL = pSLL->next;
            }

            pPrevAET = &AET;
            pAET = AET.next;
            pWETE = pAET;

            /* For each active edge */
            while (pAET)
            {
                /* Add to the buffer only those edges that
                 * are in the Winding active edge table. */
                if (pWETE == pAET)
                {
                    pts->x = pAET->bres.minor_axis;
                    pts->y = y;
                    pts++;
                    iPts++;

                    /* Send out the buffer */
                    if (iPts == NUMPTSTOBUFFER)
                    {
                        tmpPtBlock = ExAllocatePoolWithTag(PagedPool,
                                                           sizeof(POINTBLOCK),
                                                           TAG_REGION);
                        if (tmpPtBlock == NULL)
                        {
                            DPRINT1("Can't alloc tPB\n");
                            goto Cleanup;
                        }
                        curPtBlock->next = tmpPtBlock;
                        curPtBlock = tmpPtBlock;
                        pts = curPtBlock->pts;
                        numFullPtBlocks++;
                        iPts = 0;
                    }

                    pWETE = pWETE->nextWETE;
                }

                EVALUATEEDGEWINDING(pAET, pPrevAET, y, fixWAET);
            }

            /* Recompute the winding active edge table if
             * we just resorted or have exited an edge. */
            if (REGION_InsertionSort(&AET) || fixWAET)
            {
                REGION_computeWAET(&AET);
                fixWAET = FALSE;
            }
        }
    }

    REGION_PtsToRegion(numFullPtBlocks, iPts, &FirstPtBlock, prgn);
    bResult = TRUE;

Cleanup:
    REGION_FreeStorage(SLLBlock.next);

    for (curPtBlock = FirstPtBlock.next; numFullPtBlocks-- > 0;)
    {
        tmpPtBlock = curPtBlock->next;
        ExFreePoolWithTag(curPtBlock, TAG_REGION);
        curPtBlock = tmpPtBlock;
    }

    ExFreePoolWithTag(pETEs, TAG_REGION);
    return bResult;
}