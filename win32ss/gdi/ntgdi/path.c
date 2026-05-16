/*
 * PROJECT:         ReactOS win32 kernel mode subsystem
 * LICENSE:         GPL - See COPYING in the top level directory
 * FILE:            win32ss/gdi/ntgdi/path.c
 * PURPOSE:         Graphics paths (BeginPath, EndPath etc.)
 * PROGRAMMER:      Copyright 1997, 1998 Martin Boehme
 *                            1999 Huw D M Davies
 *                            2005 Dmitry Timoshkov
 *                            2018 Katayama Hirofumi MZ
 */

#include <win32k.h>
#include <safe_user_io.h>
#include <suppress.h>

DBG_DEFAULT_CHANNEL(GdiPath);

#ifdef _MSC_VER
#pragma warning(disable:4244)
#endif

#define NUM_ENTRIES_INITIAL 16  /* Initial size of points / flags arrays  */

#define GROW_FACTOR_NUMER    2  /* Numerator of grow factor for the array */
#define GROW_FACTOR_DENOM    1  /* Denominator of grow factor             */

#if DBG
static int PathCount = 0;
#endif

/***********************************************************************
 * Internal functions
 */

PPATH FASTCALL
PATH_CreatePath(int count)
{
    PPATH pPath = PATH_AllocPathWithHandle();

    if (!pPath)
    {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return NULL;
    }

    TRACE("CreatePath p 0x%p\n", pPath);
    // Path handles are shared. Also due to recursion with in the same thread.
    GDIOBJ_vUnlockObject((POBJ)pPath);       // Unlock
    pPath = PATH_LockPath(pPath->BaseObject.hHmgr); // Share Lock.

    /* Make sure that path is empty */
    PATH_EmptyPath(pPath);

    count = max( NUM_ENTRIES_INITIAL, count );

    pPath->numEntriesAllocated = count;

    pPath->pPoints = (POINT *)ExAllocatePoolWithTag(PagedPool, count * sizeof(POINT), TAG_PATH);
    RtlZeroMemory( pPath->pPoints, count * sizeof(POINT));
    pPath->pFlags  =  (BYTE *)ExAllocatePoolWithTag(PagedPool, count * sizeof(BYTE),  TAG_PATH);
    RtlZeroMemory( pPath->pFlags, count * sizeof(BYTE));

    /* Initialize variables for new path */
    pPath->numEntriesUsed = 0;
    pPath->newStroke = TRUE;
    pPath->state     = PATH_Open;
    pPath->pos.x = pPath->pos.y = 0;
#if DBG
    PathCount++;
    TRACE("Create Path %d\n",PathCount);
#endif
    return pPath;
}

/* PATH_DestroyGdiPath
 *
 * Destroys a GdiPath structure (frees the memory in the arrays).
 */
VOID
FASTCALL
PATH_DestroyGdiPath(PPATH pPath)
{
    ASSERT(pPath != NULL);

    if (pPath->pPoints) ExFreePoolWithTag(pPath->pPoints, TAG_PATH);
    if (pPath->pFlags) ExFreePoolWithTag(pPath->pFlags, TAG_PATH);
}

BOOL
FASTCALL
PATH_Delete(HPATH hPath)
{
    PPATH pPath;
    if (!hPath) return FALSE;
    pPath = PATH_LockPath(hPath);
    if (!pPath) return FALSE;
    PATH_DestroyGdiPath(pPath);
    GDIOBJ_vDeleteObject(&pPath->BaseObject);
#if DBG
    PathCount--;
    TRACE("Delete Path %d\n",PathCount);
#endif
    return TRUE;
}


VOID
FASTCALL
IntGdiCloseFigure(PPATH pPath)
{
    ASSERT(pPath->state == PATH_Open);

    // FIXME: Shouldn't we draw a line to the beginning of the figure?
    // Set PT_CLOSEFIGURE on the last entry and start a new stroke
    if (pPath->numEntriesUsed)
    {
        pPath->pFlags[pPath->numEntriesUsed - 1] |= PT_CLOSEFIGURE;
        pPath->newStroke = TRUE;
    }
}

/* MSDN: This fails if the device coordinates exceed 27 bits, or if the converted
         logical coordinates exceed 32 bits. */
BOOL
FASTCALL
GdiPathDPtoLP(
    PDC pdc,
    PPOINT ppt,
    INT count)
{
    XFORMOBJ xo;

    XFORMOBJ_vInit(&xo, &pdc->pdcattr->mxDeviceToWorld);
    return XFORMOBJ_bApplyXform(&xo, XF_LTOL, count, (PPOINTL)ppt, (PPOINTL)ppt);
}

/* PATH_InitGdiPath
 *
 * Initializes the GdiPath structure.
 */
VOID
FASTCALL
PATH_InitGdiPath(
    PPATH pPath)
{
    ASSERT(pPath != NULL);

    pPath->state = PATH_Null;
    pPath->pPoints = NULL;
    pPath->pFlags = NULL;
    pPath->numEntriesUsed = 0;
    pPath->numEntriesAllocated = 0;
}

/* PATH_AssignGdiPath
 *
 * Copies the GdiPath structure "pPathSrc" to "pPathDest". A deep copy is
 * performed, i.e. the contents of the pPoints and pFlags arrays are copied,
 * not just the pointers. Since this means that the arrays in pPathDest may
 * need to be resized, pPathDest should have been initialized using
 * PATH_InitGdiPath (in C++, this function would be an assignment operator,
 * not a copy constructor).
 * Returns TRUE if successful, else FALSE.
 */
BOOL
FASTCALL
PATH_AssignGdiPath(
    PPATH pPathDest,
    const PPATH pPathSrc)
{
    ASSERT(pPathDest != NULL && pPathSrc != NULL);

    /* Make sure destination arrays are big enough */
    if (!PATH_ReserveEntries(pPathDest, pPathSrc->numEntriesUsed))
        return FALSE;

    /* Perform the copy operation */
    memcpy(pPathDest->pPoints, pPathSrc->pPoints, sizeof(POINT)*pPathSrc->numEntriesUsed);
    memcpy(pPathDest->pFlags,  pPathSrc->pFlags,  sizeof(BYTE)*pPathSrc->numEntriesUsed);

    pPathDest->pos = pPathSrc->pos;
    pPathDest->state = pPathSrc->state;
    pPathDest->numEntriesUsed = pPathSrc->numEntriesUsed;
    pPathDest->newStroke = pPathSrc->newStroke;
    return TRUE;
}

BOOL PATH_SavePath( DC *dst, DC *src )
{
    PPATH pdstPath, psrcPath = PATH_LockPath(src->dclevel.hPath);
    TRACE("PATH_SavePath\n");
    if (psrcPath)
    {
       TRACE("PATH_SavePath 1\n");

       pdstPath = PATH_CreatePath(psrcPath->numEntriesAllocated);

       dst->dclevel.flPath = src->dclevel.flPath;

       dst->dclevel.hPath = pdstPath->BaseObject.hHmgr;

       PATH_AssignGdiPath(pdstPath, psrcPath);

       PATH_UnlockPath(pdstPath);
       PATH_UnlockPath(psrcPath);
    }
    return TRUE;
}

BOOL PATH_RestorePath( DC *dst, DC *src )
{
    TRACE("PATH_RestorePath\n");

    if (dst->dclevel.hPath == NULL)
    {
       PPATH pdstPath, psrcPath = PATH_LockPath(src->dclevel.hPath);
       TRACE("PATH_RestorePath 1\n");
       pdstPath = PATH_CreatePath(psrcPath->numEntriesAllocated);
       dst->dclevel.flPath = src->dclevel.flPath;
       dst->dclevel.hPath = pdstPath->BaseObject.hHmgr;

       PATH_AssignGdiPath(pdstPath, psrcPath);

       PATH_UnlockPath(pdstPath);
       PATH_UnlockPath(psrcPath);
    }
    else
    {
       PPATH pdstPath, psrcPath = PATH_LockPath(src->dclevel.hPath);
       pdstPath = PATH_LockPath(dst->dclevel.hPath);
       TRACE("PATH_RestorePath 2\n");
       dst->dclevel.flPath = src->dclevel.flPath & (DCPATH_CLOCKWISE|DCPATH_ACTIVE);
       PATH_AssignGdiPath(pdstPath, psrcPath);

       PATH_UnlockPath(pdstPath);
       PATH_UnlockPath(psrcPath);
    }
    return TRUE;
}

/* PATH_EmptyPath
 *
 * Removes all entries from the path and sets the path state to PATH_Null.
 */
VOID
FASTCALL
PATH_EmptyPath(PPATH pPath)
{
    ASSERT(pPath != NULL);

    pPath->state = PATH_Null;
    pPath->numEntriesUsed = 0;
}

/* PATH_AddEntry
 *
 * Adds an entry to the path. For "flags", pass either PT_MOVETO, PT_LINETO
 * or PT_BEZIERTO, optionally ORed with PT_CLOSEFIGURE. Returns TRUE if
 * successful, FALSE otherwise (e.g. if not enough memory was available).
 */
BOOL
FASTCALL
PATH_AddEntry(
    PPATH pPath,
    const POINT *pPoint,
    BYTE flags)
{
    ASSERT(pPath != NULL);

    /* FIXME: If newStroke is true, perhaps we want to check that we're
     * getting a PT_MOVETO
     */
    TRACE("(%d,%d) - %d\n", pPoint->x, pPoint->y, flags);

    /* Reserve enough memory for an extra path entry */
    if (!PATH_ReserveEntries(pPath, pPath->numEntriesUsed + 1))
        return FALSE;

    /* Store information in path entry */
    pPath->pPoints[pPath->numEntriesUsed] = *pPoint;
    pPath->pFlags[pPath->numEntriesUsed] = flags;

    /* Increment entry count */
    pPath->numEntriesUsed++;

    return TRUE;
}

/* PATH_ReserveEntries
 *
 * Ensures that at least "numEntries" entries (for points and flags) have
 * been allocated; allocates larger arrays and copies the existing entries
 * to those arrays, if necessary. Returns TRUE if successful, else FALSE.
 */
BOOL
FASTCALL
PATH_ReserveEntries(
    PPATH pPath,
    INT numEntries)
{
    INT numEntriesToAllocate;
    POINT *pPointsNew;
    BYTE *pFlagsNew;

    ASSERT(pPath != NULL);
    ASSERT(numEntries >= 0);

    /* Do we have to allocate more memory? */
    if (numEntries > pPath->numEntriesAllocated)
    {
        /* Find number of entries to allocate. We let the size of the array
         * grow exponentially, since that will guarantee linear time
         * complexity. */
        if (pPath->numEntriesAllocated)
        {
            numEntriesToAllocate = pPath->numEntriesAllocated;
            while (numEntriesToAllocate < numEntries)
                numEntriesToAllocate = numEntriesToAllocate * GROW_FACTOR_NUMER / GROW_FACTOR_DENOM;
        }
        else
            numEntriesToAllocate = numEntries;

        /* Allocate new arrays */
        pPointsNew = (POINT *)ExAllocatePoolWithTag(PagedPool, numEntriesToAllocate * sizeof(POINT), TAG_PATH);
        if (!pPointsNew)
            return FALSE;

        pFlagsNew = (BYTE *)ExAllocatePoolWithTag(PagedPool, numEntriesToAllocate * sizeof(BYTE), TAG_PATH);
        if (!pFlagsNew)
        {
            ExFreePoolWithTag(pPointsNew, TAG_PATH);
            return FALSE;
        }

        /* Copy old arrays to new arrays and discard old arrays */
        if (pPath->pPoints)
        {
            ASSERT(pPath->pFlags);

            memcpy(pPointsNew, pPath->pPoints, sizeof(POINT)*pPath->numEntriesUsed);
            memcpy(pFlagsNew, pPath->pFlags, sizeof(BYTE)*pPath->numEntriesUsed);

            ExFreePoolWithTag(pPath->pPoints, TAG_PATH);
            ExFreePoolWithTag(pPath->pFlags, TAG_PATH);
        }

        pPath->pPoints = pPointsNew;
        pPath->pFlags = pFlagsNew;
        pPath->numEntriesAllocated = numEntriesToAllocate;
    }

    return TRUE;
}

/* PATH_ScaleNormalizedPoint
 *
 * Scales a normalized point (x, y) with respect to the box whose corners are
 * passed in "corners". The point is stored in "*pPoint". The normalized
 * coordinates (-1.0, -1.0) correspond to corners[0], the coordinates
 * (1.0, 1.0) correspond to corners[1].
 */
static
BOOLEAN
PATH_ScaleNormalizedPoint(
    POINT corners[],
    FLOATL x,
    FLOATL y,
    POINT *pPoint)
{
    FLOATOBJ tmp;

    ASSERT(corners);
    ASSERT(pPoint);

    /* pPoint->x = (double)corners[0].x + (double)(corners[1].x - corners[0].x) * 0.5 * (x + 1.0); */
    FLOATOBJ_SetFloat(&tmp, x);
    FLOATOBJ_Add(&tmp, (FLOATOBJ*)&gef1);
    FLOATOBJ_Div(&tmp, (FLOATOBJ*)&gef2);
    FLOATOBJ_MulLong(&tmp, corners[1].x - corners[0].x);
    FLOATOBJ_AddLong(&tmp, corners[0].x);
    if (!FLOATOBJ_bConvertToLong(&tmp, &pPoint->x))
        return FALSE;

    /* pPoint->y = (double)corners[0].y + (double)(corners[1].y - corners[0].y) * 0.5 * (y + 1.0); */
    FLOATOBJ_SetFloat(&tmp, y);
    FLOATOBJ_Add(&tmp, (FLOATOBJ*)&gef1);
    FLOATOBJ_Div(&tmp, (FLOATOBJ*)&gef2);
    FLOATOBJ_MulLong(&tmp, corners[1].y - corners[0].y);
    FLOATOBJ_AddLong(&tmp, corners[0].y);
    if (!FLOATOBJ_bConvertToLong(&tmp, &pPoint->y))
        return FALSE;
    return TRUE;
}

/* PATH_NormalizePoint
 *
 * Normalizes a point with respect to the box whose corners are passed in
 * corners. The normalized coordinates are stored in *pX and *pY.
 */
static
VOID
PATH_NormalizePoint(
    POINTL corners[],
    const POINTL *pPoint,
    FLOATL *pX,
    FLOATL *pY)
{
    FLOATOBJ tmp;

    ASSERT(corners);
    ASSERT(pPoint);
    ASSERT(pX);
    ASSERT(pY);

    /* *pX = (float)(pPoint->x - corners[0].x) / (float)(corners[1].x - corners[0].x) * 2.0 - 1.0; */
    FLOATOBJ_SetLong(&tmp, (pPoint->x - corners[0].x) * 2);
    FLOATOBJ_DivLong(&tmp, corners[1].x - corners[0].x);
    FLOATOBJ_Sub(&tmp, (PFLOATOBJ)&gef1);
    *pX = FLOATOBJ_GetFloat(&tmp);

    /* *pY = (float)(pPoint->y - corners[0].y) / (float)(corners[1].y - corners[0].y) * 2.0 - 1.0; */
    FLOATOBJ_SetLong(&tmp, (pPoint->y - corners[0].y) * 2);
    FLOATOBJ_DivLong(&tmp, corners[1].y - corners[0].y);
    FLOATOBJ_Sub(&tmp, (PFLOATOBJ)&gef1);
    *pY = FLOATOBJ_GetFloat(&tmp);
}

/* PATH_CheckCorners
 *
 * Helper function for PATH_RoundRect() and PATH_Rectangle()
 */
static
BOOL
PATH_CheckRect(
    DC *dc,
    RECTL* rect,
    INT x1,
    INT y1,
    INT x2,
    INT y2)
{
    PDC_ATTR pdcattr = dc->pdcattr;

    /* Convert points to device coordinates */
    RECTL_vSetRect(rect, x1, y1, x2, y2);
    IntLPtoDP(dc, (PPOINT)rect, 2);

    /* Make sure first corner is top left and second corner is bottom right */
    RECTL_vMakeWellOrdered(rect);

    /* In GM_COMPATIBLE, don't include bottom and right edges */
    if (pdcattr->iGraphicsMode == GM_COMPATIBLE)
    {
        if (rect->left == rect->right) return FALSE;
        if (rect->top == rect->bottom) return FALSE;
        rect->right--;
        rect->bottom--;
    }
    return TRUE;
}

/* add a number of points, converting them to device coords */
/* return a pointer to the first type byte so it can be fixed up if necessary */
static BYTE *add_log_points( DC *dc, PPATH path, const POINT *points,
                             DWORD count, BYTE type )
{
    BYTE *ret;

    if (!PATH_ReserveEntries( path, path->numEntriesUsed + count )) return NULL;

    ret = &path->pFlags[path->numEntriesUsed];
    memcpy( &path->pPoints[path->numEntriesUsed], points, count * sizeof(*points) );
    IntLPtoDP( dc, &path->pPoints[path->numEntriesUsed], count );
    memset( ret, type, count );
    path->numEntriesUsed += count;
    return ret;
}

/* add a number of points that are already in device coords */
/* return a pointer to the first type byte so it can be fixed up if necessary */
static BYTE *add_points( PPATH path, const POINT *points, DWORD count, BYTE type )
{
    BYTE *ret;

    if (!PATH_ReserveEntries( path, path->numEntriesUsed + count )) return NULL;

    ret = &path->pFlags[path->numEntriesUsed];
    memcpy( &path->pPoints[path->numEntriesUsed], points, count * sizeof(*points) );
    memset( ret, type, count );
    path->numEntriesUsed += count;
    return ret;
}

/* reverse the order of an array of points */
static void reverse_points( POINT *points, UINT count )
{
    UINT i;
    for (i = 0; i < count / 2; i++)
    {
        POINT pt = points[i];
        points[i] = points[count - i - 1];
        points[count - i - 1] = pt;
    }
}

/* start a new path stroke if necessary */
static BOOL start_new_stroke( PPATH path )
{
    if (!path->newStroke && path->numEntriesUsed &&
        !(path->pFlags[path->numEntriesUsed - 1] & PT_CLOSEFIGURE) &&
        path->pPoints[path->numEntriesUsed - 1].x == path->pos.x &&
        path->pPoints[path->numEntriesUsed - 1].y == path->pos.y)
        return TRUE;

    path->newStroke = FALSE;
    return add_points( path, &path->pos, 1, PT_MOVETO ) != NULL;
}

/* set current position to the last point that was added to the path */
static void update_current_pos( PPATH path )
{
    ASSERT(path->numEntriesUsed);
    path->pos = path->pPoints[path->numEntriesUsed - 1];
}

/* close the current figure */
static void close_figure( PPATH path )
{
    ASSERT(path->numEntriesUsed);
    path->pFlags[path->numEntriesUsed - 1] |= PT_CLOSEFIGURE;
}

/* add a number of points, starting a new stroke if necessary */
static BOOL add_log_points_new_stroke( DC *dc, PPATH path, const POINT *points,
                                       DWORD count, BYTE type )
{
    if (!start_new_stroke( path )) return FALSE;
    if (!add_log_points( dc, path, points, count, type )) return FALSE;
    update_current_pos( path );

    TRACE("ALPNS : Pos X %d Y %d\n",path->pos.x, path->pos.y);
    IntGdiMoveToEx(dc, path->pos.x, path->pos.y, NULL);

    return TRUE;
}

/* PATH_MoveTo
 *
 * Should be called when a MoveTo is performed on a DC that has an
 * open path. This starts a new stroke. Returns TRUE if successful, else
 * FALSE.
 */
BOOL
FASTCALL
PATH_MoveTo(
    PDC dc,
    PPATH pPath)
{
    if (!pPath) return FALSE;

    // GDI32 : Signal from user space of a change in position.
    if (dc->pdcattr->ulDirty_ & DIRTY_STYLESTATE)
    {
       TRACE("MoveTo has changed\n");
       pPath->newStroke = TRUE;
       // Set position and clear the signal flag.
       IntGetCurrentPositionEx(dc, &pPath->pos);
       IntLPtoDP( dc, &pPath->pos, 1 );
       return TRUE;
    }

    return FALSE;
}

/* PATH_LineTo
 *
 * Should be called when a LineTo is performed on a DC that has an
 * open path. This adds a PT_LINETO entry to the path (and possibly
 * a PT_MOVETO entry, if this is the first LineTo in a stroke).
 * Returns TRUE if successful, else FALSE.
 */
BOOL
FASTCALL
PATH_LineTo(
    PDC dc,
    INT x,
    INT y)
{
    BOOL Ret;
    PPATH pPath;
    POINT point, pointCurPos;

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath) return FALSE;

    point.x = x;
    point.y = y;

    // Coalesce a MoveTo point.
    if ( !PATH_MoveTo(dc, pPath) )
    {
       /* Add a PT_MOVETO if necessary */
       if (pPath->newStroke)
       {
           TRACE("Line To : New Stroke\n");
           pPath->newStroke = FALSE;
           IntGetCurrentPositionEx(dc, &pointCurPos);
           CoordLPtoDP(dc, &pointCurPos);
           if (!PATH_AddEntry(pPath, &pointCurPos, PT_MOVETO))
           {
               PATH_UnlockPath(pPath);
               return FALSE;
           }
       }
    }
    Ret = add_log_points_new_stroke( dc, pPath, &point, 1, PT_LINETO );
    PATH_UnlockPath(pPath);
    return Ret;
}

/* PATH_Rectangle
 *
 * Should be called when a call to Rectangle is performed on a DC that has
 * an open path. Returns TRUE if successful, else FALSE.
 */
BOOL
FASTCALL
PATH_Rectangle(
    PDC dc,
    INT x1,
    INT y1,
    INT x2,
    INT y2)
{
    PPATH pPath;
    RECTL rect;
    POINTL points[4];
    BYTE *type;

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath) return FALSE;

    if (!PATH_CheckRect(dc, &rect, x1, y1, x2, y2))
    {
        PATH_UnlockPath(pPath);
        return TRUE;
    }

    points[0].x = rect.right; points[0].y = rect.top;
    points[1].x = rect.left; points[1].y = rect.top;
    points[2].x = rect.left; points[2].y = rect.bottom;
    points[3].x = rect.right; points[3].y = rect.bottom;

    if (dc->dclevel.flPath & DCPATH_CLOCKWISE) reverse_points(points, 4 );

    if (!(type = add_points( pPath, points, 4, PT_LINETO )))
    {
       PATH_UnlockPath(pPath);
       return FALSE;
    }
    type[0] = PT_MOVETO;

    /* Close the rectangle figure */
    IntGdiCloseFigure(pPath) ;
    PATH_UnlockPath(pPath);
    return TRUE;
}

/* PATH_RoundRect
 *
 * Should be called when a call to RoundRect is performed on a DC that has
 * an open path. Returns TRUE if successful, else FALSE.
 *
 */
BOOL
PATH_RoundRect(
    DC *dc,
    INT x1,
    INT y1,
    INT x2,
    INT y2,
    INT ell_width,
    INT ell_height)
{
    PPATH pPath;
    RECTL rect, ellipse;
    POINT points[16];
    BYTE *type;
    INT xOffset, yOffset;

    if (!ell_width || !ell_height) return PATH_Rectangle( dc, x1, y1, x2, y2 );

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath) return FALSE;

    if (!PATH_CheckRect(dc, &rect, x1, y1, x2, y2))
    {
        PATH_UnlockPath(pPath);
        return TRUE;
    }

    RECTL_vSetRect(&ellipse, 0, 0, ell_width, ell_height);
    IntLPtoDP( dc, (PPOINT)&ellipse, 2 );
    RECTL_vMakeWellOrdered(&ellipse);
    ell_width = min(RECTL_lGetWidth(&ellipse), RECTL_lGetWidth(&rect));
    ell_height = min(RECTL_lGetHeight(&ellipse), RECTL_lGetHeight(&rect));

    /*
     * See here to understand what's happening
     * https://stackoverflow.com/questions/1734745/how-to-create-circle-with-b%C3%A9zier-curves
     */
    xOffset = EngMulDiv(ell_width, 44771525, 200000000); /* w * (1 - 0.5522847) / 2 */
    yOffset = EngMulDiv(ell_height, 44771525, 200000000); /* h * (1 - 0.5522847) / 2 */
    TRACE("xOffset %d, yOffset %d, Rect WxH: %dx%d.\n",
        xOffset, yOffset, RECTL_lGetWidth(&rect), RECTL_lGetHeight(&rect));

    /*
     * Get half width & height.
     * Do not use integer division, we need the rounding made by EngMulDiv.
     */
    ell_width = EngMulDiv(ell_width, 1, 2);
    ell_height = EngMulDiv(ell_height, 1, 2);

    /* starting point */
    points[0].x  = rect.right;
    points[0].y  = rect.top + ell_height;
    /* first curve */
    points[1].x = rect.right;
    points[1].y = rect.top + yOffset;
    points[2].x = rect.right - xOffset;
    points[2].y = rect.top;
    points[3].x  = rect.right - ell_width;
    points[3].y  = rect.top;
    /* horizontal line */
    points[4].x  = rect.left + ell_width;
    points[4].y  = rect.top;
    /* second curve */
    points[5].x  = rect.left + xOffset;
    points[5].y  = rect.top;
    points[6].x  = rect.left;
    points[6].y  = rect.top + yOffset;
    points[7].x  = rect.left;
    points[7].y  = rect.top + ell_height;
    /* vertical line */
    points[8].x  = rect.left;
    points[8].y  = rect.bottom - ell_height;
    /* third curve */
    points[9].x  = rect.left;
    points[9].y  = rect.bottom - yOffset;
    points[10].x = rect.left + xOffset;
    points[10].y = rect.bottom;
    points[11].x = rect.left + ell_width;
    points[11].y = rect.bottom;
    /* horizontal line */
    points[12].x = rect.right - ell_width;
    points[12].y = rect.bottom;
    /* fourth curve */
    points[13].x = rect.right - xOffset;
    points[13].y = rect.bottom;
    points[14].x = rect.right;
    points[14].y = rect.bottom - yOffset;
    points[15].x = rect.right;
    points[15].y = rect.bottom - ell_height;

    if (dc->dclevel.flPath & DCPATH_CLOCKWISE) reverse_points( points, 16 );
    if (!(type = add_points( pPath, points, 16, PT_BEZIERTO )))
    {
       PATH_UnlockPath(pPath);
       return FALSE;
    }
    type[0] = PT_MOVETO;
    type[4] = type[8] = type[12] = PT_LINETO;

    IntGdiCloseFigure(pPath);
    PATH_UnlockPath(pPath);
    return TRUE;
}

/* PATH_Ellipse
 *
 */
BOOL
PATH_Ellipse(
    PDC dc,
    INT x1,
    INT y1,
    INT x2,
    INT y2)
{
    PPATH pPath;
    POINT points[13];
    RECTL rect;
    BYTE *type;
    LONG xRadius, yRadius, xOffset, yOffset;
    POINT left, top, right, bottom;

    TRACE("PATH_Ellipse: %p -> (%d, %d) - (%d, %d)\n",
            dc, x1, y1, x2, y2);

    if (!PATH_CheckRect(dc, &rect, x1, y1, x2, y2))
    {
        return TRUE;
    }

    xRadius = RECTL_lGetWidth(&rect) / 2;
    yRadius = RECTL_lGetHeight(&rect) / 2;

    /* Get the four points which box our ellipse */
    left.x = rect.left; left.y = rect.top + yRadius;
    top.x = rect.left + xRadius; top.y = rect.top;
    right.x = rect.right; right.y = rect.bottom - yRadius;
    bottom.x = rect.right - xRadius; bottom.y = rect.bottom;

    /*
     * See here to understand what's happening
     * https://stackoverflow.com/questions/1734745/how-to-create-circle-with-b%C3%A9zier-curves
     */
    xOffset = EngMulDiv(RECTL_lGetWidth(&rect), 55428475, 200000000); /* w * 0.55428475 / 2 */
    yOffset = EngMulDiv(RECTL_lGetHeight(&rect), 55428475, 200000000); /* h * 0.55428475 / 2 */
    TRACE("xOffset %d, yOffset %d, Rect WxH: %dx%d.\n",
        xOffset, yOffset, RECTL_lGetWidth(&rect), RECTL_lGetHeight(&rect));

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath)
        return FALSE;

    /* Starting point: Right */
    points[0] = right;

    /* first curve - going up, left */
    points[1] = right;
    points[1].y -= yOffset;
    points[2] = top;
    points[2].x += xOffset;

    /* top */
    points[3] = top;

    /* second curve - going left, down*/
    points[4] = top;
    points[4].x -= xOffset;
    points[5] = left;
    points[5].y -= yOffset;

    /* Left */
    points[6] = left;

    /* Third curve - going down, right */
    points[7] = left;
    points[7].y += yOffset;
    points[8] = bottom;
    points[8].x -= xOffset;

    /* bottom */
    points[9] = bottom;

    /* Fourth curve - Going right, up */
    points[10] = bottom;
    points[10].x += xOffset;
    points[11] = right;
    points[11].y += yOffset;

    /* Back to starting point */
    points[12] = right;

    if (dc->dclevel.flPath & DCPATH_CLOCKWISE) reverse_points( points, 13 );
    if (!(type = add_points( pPath, points, 13, PT_BEZIERTO )))
    {
        ERR("PATH_Ellipse No add\n");
        PATH_UnlockPath(pPath);
        return FALSE;
    }
    type[0] = PT_MOVETO;

    IntGdiCloseFigure(pPath);
    PATH_UnlockPath(pPath);
    return TRUE;
}

/* PATH_DoArcPart
 *
 * Creates a Bezier spline that corresponds to part of an arc and appends the
 * corresponding points to the path. The start and end angles are passed in
 * "angleStart" and "angleEnd"; these angles should span a quarter circle
 * at most. If "startEntryType" is non-zero, an entry of that type for the first
 * control point is added to the path; otherwise, it is assumed that the current
 * position is equal to the first control point.
 */
static
BOOL
PATH_DoArcPart(
    PPATH pPath,
    POINT corners[],
    double angleStart,
    double angleEnd,
    BYTE startEntryType)
{
    double  halfAngle, a;
    float  xNorm[4], yNorm[4];
    POINT points[4];
    BYTE *type;
    int i, start;

    ASSERT(fabs(angleEnd - angleStart) <= M_PI_2);

    /* FIXME: Is there an easier way of computing this? */

    /* Compute control points */
    halfAngle = (angleEnd - angleStart) / 2.0;
    if (fabs(halfAngle) > 1e-8)
    {
        a = 4.0 / 3.0 * (1 - cos(halfAngle)) / sin(halfAngle);
        xNorm[0] = cos(angleStart);
        yNorm[0] = sin(angleStart);
        xNorm[1] = xNorm[0] - a * yNorm[0];
        yNorm[1] = yNorm[0] + a * xNorm[0];
        xNorm[3] = cos(angleEnd);
        yNorm[3] = sin(angleEnd);
        xNorm[2] = xNorm[3] + a * yNorm[3];
        yNorm[2] = yNorm[3] - a * xNorm[3];
    }
    else
        for (i = 0; i < 4; i++)
        {
            xNorm[i] = cos(angleStart);
            yNorm[i] = sin(angleStart);
        }

    /* Add starting point to path if desired */
    start = !startEntryType;

    /* Add remaining control points */
    for (i = start; i < 4; i++)
    {
        if (!PATH_ScaleNormalizedPoint(corners, *(FLOATL*)&xNorm[i], *(FLOATL*)&yNorm[i], &points[i]))
            return FALSE;
    }
    if (!(type = add_points( pPath, points + start, 4 - start, PT_BEZIERTO ))) return FALSE;
    if (!start) type[0] = startEntryType;

    return TRUE;
}

/* PATH_Arc
 *
 * Should be called when a call to Arc is performed on a DC that has
 * an open path. This adds up to five Bezier splines representing the arc
 * to the path. When 'lines' is 1, we add 1 extra line to get a chord,
 * when 'lines' is 2, we add 2 extra lines to get a pie, and when 'lines' is
 * -1 we add 1 extra line from the current DC position to the starting position
 * of the arc before drawing the arc itself (arcto). Returns TRUE if successful,
 * else FALSE.
 */
BOOL
FASTCALL
PATH_Arc(
    PDC dc,
    INT x1,
    INT y1,
    INT x2,
    INT y2,
    INT xStart,
    INT yStart,
    INT xEnd,
    INT yEnd,
    INT direction,
    INT lines)
{
    double  angleStart, angleEnd, angleStartQuadrant, angleEndQuadrant = 0.0;
    /* Initialize angleEndQuadrant to silence gcc's warning */
    FLOATL  x, y;
    POINT corners[2], pointStart, pointEnd;
    POINT   centre, pointCurPos;
    BOOL    start, end, Ret = TRUE;
    INT     temp;
    BOOL    clockwise;
    PPATH   pPath;

    /* FIXME: This function should check for all possible error returns */
    /* FIXME: Do we have to respect newStroke? */

    ASSERT(dc);

    pPath = PATH_LockPath(dc->dclevel.hPath);
    if (!pPath) return FALSE;

    if (direction)
       clockwise = ((direction == AD_CLOCKWISE) !=0 );
    else
       clockwise = ((dc->dclevel.flPath & DCPATH_CLOCKWISE) != 0);

    /* Check for zero height / width */
    /* FIXME: Only in GM_COMPATIBLE? */
    if (x1 == x2 || y1 == y2)
    {
        Ret = TRUE;
        goto ArcExit;
    }
    /* Convert points to device coordinates */
    corners[0].x = x1; corners[0].y = y1;
    corners[1].x = x2; corners[1].y = y2;
    pointStart.x = xStart; pointStart.y = yStart;
    pointEnd.x = xEnd; pointEnd.y = yEnd;
    INTERNAL_LPTODP(dc, corners, 2);
    INTERNAL_LPTODP(dc, &pointStart, 1);
    INTERNAL_LPTODP(dc, &pointEnd, 1);

    /* Make sure first corner is top left and second corner is bottom right */
    if (corners[0].x > corners[1].x)
    {
        temp = corners[0].x;
        corners[0].x = corners[1].x;
        corners[1].x = temp;
    }
    if (corners[0].y > corners[1].y)
    {
        temp = corners[0].y;
        corners[0].y = corners[1].y;
        corners[1].y = temp;
    }

    /* Compute start and end angle */
    PATH_NormalizePoint(corners, &pointStart, &x, &y);
    angleStart = atan2(*(FLOAT*)&y, *(FLOAT*)&x);
    PATH_NormalizePoint(corners, &pointEnd, &x, &y);
    angleEnd = atan2(*(FLOAT*)&y, *(FLOAT*)&x);

    /* Make sure the end angle is "on the right side" of the start angle */
    if (clockwise)
    {
        if (angleEnd <= angleStart)
        {
            angleEnd += 2 * M_PI;
            ASSERT(angleEnd >= angleStart);
        }
    }
    else
    {
        if (angleEnd >= angleStart)
        {
            angleEnd -= 2 * M_PI;
            ASSERT(angleEnd <= angleStart);
        }
    }

    /* In GM_COMPATIBLE, don't include bottom and right edges */
    if (dc->pdcattr->iGraphicsMode == GM_COMPATIBLE)
    {
        corners[1].x--;
        corners[1].y--;
    }

    /* arcto: Add a PT_MOVETO only if this is the first entry in a stroke */
    if (lines == GdiTypeArcTo && pPath->newStroke) // -1
    {
        pPath->newStroke = FALSE;
        IntGetCurrentPositionEx(dc, &pointCurPos);
        CoordLPtoDP(dc, &pointCurPos);
        if (!PATH_AddEntry(pPath, &pointCurPos, PT_MOVETO))
        {
            Ret = FALSE;
            goto ArcExit;
        }
    }

    /* Add the arc to the path with one Bezier spline per quadrant that the
     * arc spans */
    start = TRUE;
    end = FALSE;
    do
    {
        /* Determine the start and end angles for this quadrant */
        if (start)
        {
            angleStartQuadrant = angleStart;
            if (clockwise)
                angleEndQuadrant = (floor(angleStart / M_PI_2) + 1.0) * M_PI_2;
            else
                angleEndQuadrant = (ceil(angleStart / M_PI_2) - 1.0) * M_PI_2;
        }
        else
        {
            angleStartQuadrant = angleEndQuadrant;
            if (clockwise)
                angleEndQuadrant += M_PI_2;
            else
                angleEndQuadrant -= M_PI_2;
        }

        /* Have we reached the last part of the arc? */
        if ((clockwise && angleEnd < angleEndQuadrant) ||
            (!clockwise && angleEnd > angleEndQuadrant))
        {
            /* Adjust the end angle for this quadrant */
            angleEndQuadrant = angleEnd;
            end = TRUE;
        }

        /* Add the Bezier spline to the path */
        PATH_DoArcPart(pPath,
                       corners,
                       angleStartQuadrant,
                       angleEndQuadrant,
                       start ? (lines == GdiTypeArcTo ? PT_LINETO : PT_MOVETO) : FALSE); // -1
        start = FALSE;
    }
    while (!end);

    if (lines == GdiTypeArcTo)
    {
       update_current_pos( pPath );
    }
    else /* chord: close figure. pie: add line and close figure */
    if (lines == GdiTypeChord) // 1
    {
        IntGdiCloseFigure(pPath);
    }
    else if (lines == GdiTypePie) // 2
    {
        centre.x = (corners[0].x + corners[1].x) / 2;
        centre.y = (corners[0].y + corners[1].y) / 2;
        if (!PATH_AddEntry(pPath, &centre, PT_LINETO | PT_CLOSEFIGURE))
            Ret = FALSE;
    }
ArcExit:
    PATH_UnlockPath(pPath);
    return Ret;
}
/* --- pretranslation split: the remaining function groups live in path.split2..3.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "path.split2.c"
#include "path.split3.c"
