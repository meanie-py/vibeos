/*
 *  ReactOS W32 Subsystem
 *  Copyright (C) 1998, 1999, 2000, 2001, 2002, 2003 ReactOS Team
 *
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; either version 2 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License along
 *  with this program; if not, write to the Free Software Foundation, Inc.,
 *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

/*
 * GDI region objects. Shamelessly ripped out from the X11 distribution
 * Thanks for the nice licence.
 *
 * Copyright 1993, 1994, 1995 Alexandre Julliard
 * Modifications and additions: Copyright 1998 Huw Davies
 *                      1999 Alex Korobka
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */

/************************************************************************

Copyright (c) 1987, 1988  X Consortium

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL THE
X CONSORTIUM BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN
AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

Except as contained in this notice, the name of the X Consortium shall not be
used in advertising or otherwise to promote the sale, use or other dealings
in this Software without prior written authorization from the X Consortium.


Copyright 1987, 1988 by Digital Equipment Corporation, Maynard, Massachusetts.

            All Rights Reserved

Permission to use, copy, modify, and distribute this software and its
documentation for any purpose and without fee is hereby granted,
provided that the above copyright notice appear in all copies and that
both that copyright notice and this permission notice appear in
supporting documentation, and that the name of Digital not be
used in advertising or publicity pertaining to distribution of the
software without specific, written prior permission.

DIGITAL DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE, INCLUDING
ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS, IN NO EVENT SHALL
DIGITAL BE LIABLE FOR ANY SPECIAL, INDIRECT OR CONSEQUENTIAL DAMAGES OR
ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS,
WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS
SOFTWARE.

************************************************************************/
/*
 * The functions in this file implement the Region abstraction, similar to one
 * used in the X11 sample server. A Region is simply an area, as the name
 * implies, and is implemented as a "y-x-banded" array of rectangles. To
 * explain: Each Region is made up of a certain number of rectangles sorted
 * by y coordinate first, and then by x coordinate.
 *
 * Furthermore, the rectangles are banded such that every rectangle with a
 * given upper-left y coordinate (y1) will have the same lower-right y
 * coordinate (y2) and vice versa. If a rectangle has scanlines in a band, it
 * will span the entire vertical distance of the band. This means that some
 * areas that could be merged into a taller rectangle will be represented as
 * several shorter rectangles to account for shorter rectangles to its left
 * or right but within its "vertical scope".
 *
 * An added constraint on the rectangles is that they must cover as much
 * horizontal area as possible. E.g. no two rectangles in a band are allowed
 * to touch.
 *
 * Whenever possible, bands will be merged together to cover a greater vertical
 * distance (and thus reduce the number of rectangles). Two bands can be merged
 * only if the bottom of one touches the top of the other and they have
 * rectangles in the same places (of the same width, of course). This maintains
 * the y-x-banding that's so nice to have...
 */

// X11 sources for ReactOS region processing.
//
// libX11/src/PolyReg.c
// libX11/src/Region.c
//
//

#include <win32k.h>
#include <safe_user_io.h>
#include <suppress.h>

#define NDEBUG
#include <debug.h>

PREGION prgnDefault = NULL;
HRGN    hrgnDefault = NULL;

// Internal Functions

#if 1
#define COPY_RECTS(dest, src, nRects) \
  do {                                \
    PRECTL xDest = (dest);            \
    PRECTL xSrc = (src);              \
    UINT xRects = (nRects);           \
    while (xRects-- > 0) {            \
      *(xDest++) = *(xSrc++);         \
    }                                 \
  } while (0)
#else
#define COPY_RECTS(dest, src, nRects) RtlCopyMemory(dest, src, (nRects) * sizeof(RECTL))
#endif

#define EMPTY_REGION(pReg) { \
  (pReg)->rdh.nCount = 0; \
  (pReg)->rdh.rcBound.left = (pReg)->rdh.rcBound.top = 0; \
  (pReg)->rdh.rcBound.right = (pReg)->rdh.rcBound.bottom = 0; \
  (pReg)->rdh.iType = RDH_RECTANGLES; \
}

#define REGION_NOT_EMPTY(pReg) pReg->rdh.nCount

#define INRECT(r, x, y) \
      ( ( ((r).right >  x)) && \
        ( ((r).left <= x)) && \
        ( ((r).bottom >  y)) && \
        ( ((r).top <= y)) )

/*  1 if two RECTs overlap.
 *  0 if two RECTs do not overlap.
 */
#define EXTENTCHECK(r1, r2) \
    ((r1)->right > (r2)->left && \
     (r1)->left < (r2)->right && \
     (r1)->bottom > (r2)->top && \
     (r1)->top < (r2)->bottom)

/*
 *  In scan converting polygons, we want to choose those pixels
 *  which are inside the polygon.  Thus, we add .5 to the starting
 *  x coordinate for both left and right edges.  Now we choose the
 *  first pixel which is inside the pgon for the left edge and the
 *  first pixel which is outside the pgon for the right edge.
 *  Draw the left pixel, but not the right.
 *
 *  How to add .5 to the starting x coordinate:
 *      If the edge is moving to the right, then subtract dy from the
 *  error term from the general form of the algorithm.
 *      If the edge is moving to the left, then add dy to the error term.
 *
 *  The reason for the difference between edges moving to the left
 *  and edges moving to the right is simple:  If an edge is moving
 *  to the right, then we want the algorithm to flip immediately.
 *  If it is moving to the left, then we don't want it to flip until
 *  we traverse an entire pixel.
 */
#define BRESINITPGON(dy, x1, x2, xStart, d, m, m1, incr1, incr2) { \
    int dx;      /* Local storage */ \
\
    /* \
     *  If the edge is horizontal, then it is ignored \
     *  and assumed not to be processed.  Otherwise, do this stuff. \
     */ \
    if ((dy) != 0) { \
        xStart = (x1); \
        dx = (x2) - xStart; \
        if (dx < 0) { \
            m = dx / (dy); \
            m1 = m - 1; \
            incr1 = -2 * dx + 2 * (dy) * m1; \
            incr2 = -2 * dx + 2 * (dy) * m; \
            d = 2 * m * (dy) - 2 * dx - 2 * (dy); \
        } else { \
            m = dx / (dy); \
            m1 = m + 1; \
            incr1 = 2 * dx - 2 * (dy) * m1; \
            incr2 = 2 * dx - 2 * (dy) * m; \
            d = -2 * m * (dy) + 2 * dx; \
        } \
    } \
}

#define BRESINCRPGON(d, minval, m, m1, incr1, incr2) { \
    if (m1 > 0) { \
        if (d > 0) { \
            minval += m1; \
            d += incr1; \
        } \
        else { \
            minval += m; \
            d += incr2; \
        } \
    } else {\
        if (d >= 0) { \
            minval += m1; \
            d += incr1; \
        } \
        else { \
            minval += m; \
            d += incr2; \
        } \
    } \
}

/*
 * This structure contains all of the information needed
 * to run the bresenham algorithm.
 * The variables may be hardcoded into the declarations
 * instead of using this structure to make use of
 * register declarations.
 */
typedef struct
{
    INT minor_axis;   /* Minor axis        */
    INT d;            /* Decision variable */
    INT m, m1;        /* Slope and slope+1 */
    INT incr1, incr2; /* Error increments */
} BRESINFO;

#define BRESINITPGONSTRUCT(dmaj, min1, min2, bres) \
    BRESINITPGON(dmaj, min1, min2, bres.minor_axis, bres.d, \
                     bres.m, bres.m1, bres.incr1, bres.incr2)

#define BRESINCRPGONSTRUCT(bres) \
        BRESINCRPGON(bres.d, bres.minor_axis, bres.m, bres.m1, bres.incr1, bres.incr2)

/*
 *     These are the data structures needed to scan
 *     convert regions.  Two different scan conversion
 *     methods are available -- the even-odd method, and
 *     the winding number method.
 *     The even-odd rule states that a point is inside
 *     the polygon if a ray drawn from that point in any
 *     direction will pass through an odd number of
 *     path segments.
 *     By the winding number rule, a point is decided
 *     to be inside the polygon if a ray drawn from that
 *     point in any direction passes through a different
 *     number of clockwise and counter-clockwise path
 *     segments.
 *
 *     These data structures are adapted somewhat from
 *     the algorithm in (Foley/Van Dam) for scan converting
 *     polygons.
 *     The basic algorithm is to start at the top (smallest y)
 *     of the polygon, stepping down to the bottom of
 *     the polygon by incrementing the y coordinate.  We
 *     keep a list of edges which the current scanline crosses,
 *     sorted by x.  This list is called the Active Edge Table (AET)
 *     As we change the y-coordinate, we update each entry in
 *     in the active edge table to reflect the edges new xcoord.
 *     This list must be sorted at each scanline in case
 *     two edges intersect.
 *     We also keep a data structure known as the Edge Table (ET),
 *     which keeps track of all the edges which the current
 *     scanline has not yet reached.  The ET is basically a
 *     list of SCANLINE_LIST structures containing a list of
 *     edges which are entered at a given scanline.  There is one
 *     SCANLINE_LIST per scanline at which an edge is entered.
 *     When we enter a new edge, we move it from the ET to the AET.
 *
 *     From the AET, we can implement the even-odd rule as in
 *     (Foley/Van Dam).
 *     The winding number rule is a little trickier.  We also
 *     keep the EDGE_TABLEEntries in the AET linked by the
 *     nextWETE (winding EDGE_TABLE_ENTRY) link.  This allows
 *     the edges to be linked just as before for updating
 *     purposes, but only uses the edges linked by the nextWETE
 *     link as edges representing spans of the polygon to
 *     drawn (as with the even-odd rule).
 */

/*
 * For the winding number rule
 */
#define CLOCKWISE          1
#define COUNTERCLOCKWISE  -1

typedef struct _EDGE_TABLE_ENTRY
{
    INT ymax;             /* ycoord at which we exit this edge. */
    BRESINFO bres;        /* Bresenham info to run the edge     */
    struct _EDGE_TABLE_ENTRY *next;       /* Next in the list     */
    struct _EDGE_TABLE_ENTRY *back;       /* For insertion sort   */
    struct _EDGE_TABLE_ENTRY *nextWETE;   /* For winding num rule */
    INT ClockWise;        /* Flag for winding number rule       */
} EDGE_TABLE_ENTRY;

typedef struct _SCANLINE_LIST
{
    INT scanline;                /* The scanline represented */
    EDGE_TABLE_ENTRY *edgelist;    /* Header node              */
    struct _SCANLINE_LIST *next;  /* Next in the list       */
} SCANLINE_LIST;

typedef struct
{
    INT ymax;                 /* ymax for the polygon     */
    INT ymin;                 /* ymin for the polygon     */
    SCANLINE_LIST scanlines;   /* Header node              */
} EDGE_TABLE;

/*
 * Here is a struct to help with storage allocation
 * so we can allocate a big chunk at a time, and then take
 * pieces from this heap when we need to.
 */
#define SLLSPERBLOCK 25

typedef struct _SCANLINE_LISTBLOCK
{
    SCANLINE_LIST SLLs[SLLSPERBLOCK];
    struct _SCANLINE_LISTBLOCK *next;
} SCANLINE_LISTBLOCK;

/*
 *     A few macros for the inner loops of the fill code where
 *     performance considerations don't allow a procedure call.
 *
 *     Evaluate the given edge at the given scanline.
 *     If the edge has expired, then we leave it and fix up
 *     the active edge table; otherwise, we increment the
 *     x value to be ready for the next scanline.
 *     The winding number rule is in effect, so we must notify
 *     the caller when the edge has been removed so he
 *     can reorder the Winding Active Edge Table.
 */
#define EVALUATEEDGEWINDING(pAET, pPrevAET, y, fixWAET) { \
   if (pAET->ymax == y) {          /* Leaving this edge */ \
      pPrevAET->next = pAET->next; \
      pAET = pPrevAET->next; \
      fixWAET = 1; \
      if (pAET) \
         pAET->back = pPrevAET; \
   } \
   else { \
      BRESINCRPGONSTRUCT(pAET->bres); \
      pPrevAET = pAET; \
      pAET = pAET->next; \
   } \
}

/*
 *     Evaluate the given edge at the given scanline.
 *     If the edge has expired, then we leave it and fix up
 *     the active edge table; otherwise, we increment the
 *     x value to be ready for the next scanline.
 *     The even-odd rule is in effect.
 */
#define EVALUATEEDGEEVENODD(pAET, pPrevAET, y) { \
   if (pAET->ymax == y) {          /* Leaving this edge */ \
      pPrevAET->next = pAET->next; \
      pAET = pPrevAET->next; \
      if (pAET) \
         pAET->back = pPrevAET; \
   } \
   else { \
      BRESINCRPGONSTRUCT(pAET->bres); \
      pPrevAET = pAET; \
      pAET = pAET->next; \
   } \
}

/**************************************************************************
 *
 *    Poly Regions
 *
 *************************************************************************/

#define LARGE_COORDINATE  INT_MAX
#define SMALL_COORDINATE  INT_MIN

static
BOOL
REGION_bGrowBufferSize(
    _Inout_ PREGION prgn,
    _In_ UINT cRects)
{
    ULONG cjNewSize;
    PVOID pvBuffer;
    NT_ASSERT(cRects > 0);

    /* Make sure we don't overflow */
    if (cRects > MAXULONG / sizeof(RECTL))
    {
        return FALSE;
    }

    /* Calculate new buffer size */
    cjNewSize = cRects * sizeof(RECTL);

    /* Avoid allocating too often, by duplicating the old buffer size
       Note: we don't do an overflow check, since the old size will never
       get that large before running out of memory. */
    if (2 * prgn->rdh.nRgnSize > cjNewSize)
    {
        cjNewSize = 2 * prgn->rdh.nRgnSize;
    }

    /* Allocate the new buffer */
    pvBuffer = ExAllocatePoolWithTag(PagedPool, cjNewSize, TAG_REGION);
    if (pvBuffer == NULL)
    {
        return FALSE;
    }

    /* Copy the rects into the new buffer */
    COPY_RECTS(pvBuffer, prgn->Buffer, prgn->rdh.nCount);

    /* Free the old buffer */
    if (prgn->Buffer != &prgn->rdh.rcBound)
    {
        ExFreePoolWithTag(prgn->Buffer, TAG_REGION);
    }

    /* Set the new buffer */
    prgn->Buffer = pvBuffer;
    prgn->rdh.nRgnSize = cjNewSize;

    return TRUE;
}

static __inline
BOOL
REGION_bEnsureBufferSize(
    _Inout_ PREGION prgn,
    _In_ UINT cRects)
{
    /* Check if the current region size is too small */
    if (cRects > prgn->rdh.nRgnSize / sizeof(RECTL))
    {
        /* Allocate a new buffer */
        return REGION_bGrowBufferSize(prgn, cRects);
    }

    return TRUE;
}

FORCEINLINE
VOID
REGION_vAddRect(
    _Inout_ PREGION prgn,
    _In_ LONG left,
    _In_ LONG top,
    _In_ LONG right,
    _In_ LONG bottom)
{
    PRECTL prcl;
    NT_ASSERT((prgn->rdh.nCount + 1) * sizeof(RECT) <= prgn->rdh.nRgnSize);

    prcl = &prgn->Buffer[prgn->rdh.nCount];
    prcl->left = left;
    prcl->top = top;
    prcl->right = right;
    prcl->bottom = bottom;
    prgn->rdh.nCount++;
}

static __inline
BOOL
REGION_bAddRect(
    _Inout_ PREGION prgn,
    _In_ LONG left,
    _In_ LONG top,
    _In_ LONG right,
    _In_ LONG bottom)
{
    if (!REGION_bEnsureBufferSize(prgn, prgn->rdh.nCount + 1))
    {
        return FALSE;
    }

    REGION_vAddRect(prgn, left, top, right, bottom);
    return TRUE;
}

typedef BOOL (FASTCALL *overlapProcp)(PREGION, PRECT, PRECT, PRECT, PRECT, INT, INT);
typedef BOOL (FASTCALL *nonOverlapProcp)(PREGION, PRECT, PRECT, INT, INT);

// Number of points to buffer before sending them off to scanlines() :  Must be an even number
#define NUMPTSTOBUFFER 200

#define RGN_DEFAULT_RECTS    2

// Used to allocate buffers for points and link the buffers together
typedef struct _POINTBLOCK
{
    POINT pts[NUMPTSTOBUFFER];
    struct _POINTBLOCK *next;
} POINTBLOCK;

#ifndef NDEBUG
/*
 * This function is left there for debugging purposes.
 */
VOID
FASTCALL
IntDumpRegion(HRGN hRgn)
{
    PREGION Data;

    Data = REGION_LockRgn(hRgn);
    if (Data == NULL)
    {
        DbgPrint("IntDumpRegion called with invalid region!\n");
        return;
    }

    DbgPrint("IntDumpRegion(%x): %d,%d-%d,%d %d\n",
             hRgn,
             Data->rdh.rcBound.left,
             Data->rdh.rcBound.top,
             Data->rdh.rcBound.right,
             Data->rdh.rcBound.bottom,
             Data->rdh.iType);

    REGION_UnlockRgn(Data);
}
#endif /* Not NDEBUG */

INT
FASTCALL
REGION_Complexity(PREGION prgn)
{
    if (prgn == NULL)
        return NULLREGION;

    DPRINT("Region Complexity: %lu\n", prgn->rdh.nCount);
    switch (prgn->rdh.nCount)
    {
        case 0:
            return NULLREGION;
        case 1:
            return SIMPLEREGION;
        default:
            return COMPLEXREGION;
    }
}

static
BOOL
FASTCALL
REGION_CopyRegion(
    PREGION dst,
    PREGION src)
{
    /* Only copy if source and dest are not equal */
    if (dst != src)
    {
        /* Check if we need to increase our buffer */
        if (dst->rdh.nRgnSize < src->rdh.nCount * sizeof(RECT))
        {
            PRECTL temp;

            /* Allocate a new buffer */
            temp = ExAllocatePoolWithTag(PagedPool,
                                         src->rdh.nCount * sizeof(RECT),
                                         TAG_REGION);
            if (temp == NULL)
                return FALSE;

            /* Free the old buffer */
            if ((dst->Buffer != NULL) && (dst->Buffer != &dst->rdh.rcBound))
                ExFreePoolWithTag(dst->Buffer, TAG_REGION);

            /* Set the new buffer and the size */
            dst->Buffer = temp;
            dst->rdh.nRgnSize = src->rdh.nCount * sizeof(RECT);
        }

        dst->rdh.nCount = src->rdh.nCount;
        dst->rdh.rcBound.left = src->rdh.rcBound.left;
        dst->rdh.rcBound.top = src->rdh.rcBound.top;
        dst->rdh.rcBound.right = src->rdh.rcBound.right;
        dst->rdh.rcBound.bottom = src->rdh.rcBound.bottom;
        dst->rdh.iType = src->rdh.iType;
        COPY_RECTS(dst->Buffer, src->Buffer, src->rdh.nCount);
    }

    return TRUE;
}

static
VOID
FASTCALL
REGION_SetExtents(
    PREGION pReg)
{
    RECTL *pRect, *pRectEnd, *pExtents;

    /* Quick check for NULLREGION */
    if (pReg->rdh.nCount == 0)
    {
        pReg->rdh.rcBound.left = 0;
        pReg->rdh.rcBound.top = 0;
        pReg->rdh.rcBound.right = 0;
        pReg->rdh.rcBound.bottom = 0;
        pReg->rdh.iType = RDH_RECTANGLES;
        return;
    }

    pExtents = &pReg->rdh.rcBound;
    pRect = pReg->Buffer;
    pRectEnd = pReg->Buffer + pReg->rdh.nCount - 1;

    /* Since pRect is the first rectangle in the region, it must have the
     * smallest top and since pRectEnd is the last rectangle in the region,
     * it must have the largest bottom, because of banding. Initialize left and
     * right from pRect and pRectEnd, resp., as good things to initialize them
     * to... */
    pExtents->left = pRect->left;
    pExtents->top = pRect->top;
    pExtents->right = pRectEnd->right;
    pExtents->bottom = pRectEnd->bottom;

    while (pRect <= pRectEnd)
    {
        if (pRect->left < pExtents->left)
            pExtents->left = pRect->left;
        if (pRect->right > pExtents->right)
            pExtents->right = pRect->right;
        pRect++;
    }

    pReg->rdh.iType = RDH_RECTANGLES;
}

// FIXME: This function needs review and testing
/***********************************************************************
 *           REGION_CropRegion
 */
INT
FASTCALL
REGION_CropRegion(
    PREGION rgnDst,
    PREGION rgnSrc,
    const RECTL *rect)
{
    PRECTL lpr, rpr;
    ULONG i, j, clipa, clipb, nRgnSize;
    INT left = MAXLONG;
    INT right = MINLONG;
    INT top = MAXLONG;
    INT bottom = MINLONG;

    if ((rect->left >= rect->right) ||
        (rect->top >= rect->bottom) ||
        (EXTENTCHECK(rect, &rgnSrc->rdh.rcBound) == 0))
    {
        goto empty;
    }

    /* Skip all rects that are completely above our intersect rect */
    for (clipa = 0; clipa < rgnSrc->rdh.nCount; clipa++)
    {
        /* bottom is exclusive, so break when we go above it */
        if (rgnSrc->Buffer[clipa].bottom > rect->top) break;
    }

    /* Bail out, if there is nothing left */
    if (clipa == rgnSrc->rdh.nCount) goto empty;

    /* Find the last rect that is still within the intersect rect (exclusive) */
    for (clipb = clipa; clipb < rgnSrc->rdh.nCount; clipb++)
    {
        /* bottom is exclusive, so stop, when we start at that y pos */
        if (rgnSrc->Buffer[clipb].top >= rect->bottom) break;
    }

    /* Bail out, if there is nothing left */
    if (clipb == clipa) goto empty;

    // clipa - index of the first rect in the first intersecting band
    // clipb - index of the last rect in the last intersecting band plus 1

    /* Check if the buffer in the dest region is large enough,
       otherwise allocate a new one */
    nRgnSize = (clipb - clipa) * sizeof(RECT);
    if ((rgnDst != rgnSrc) && (rgnDst->rdh.nRgnSize < nRgnSize))
    {
        PRECTL temp;
        temp = ExAllocatePoolWithTag(PagedPool, nRgnSize, TAG_REGION);
        if (temp == NULL)
            return ERROR;

        /* Free the old buffer */
        if (rgnDst->Buffer && (rgnDst->Buffer != &rgnDst->rdh.rcBound))
            ExFreePoolWithTag(rgnDst->Buffer, TAG_REGION);

        rgnDst->Buffer = temp;
        rgnDst->rdh.nCount = 0;
        rgnDst->rdh.nRgnSize = nRgnSize;
        rgnDst->rdh.iType = RDH_RECTANGLES;
    }

    /* Loop all rects within the intersect rect from the y perspective */
    for (i = clipa, j = 0; i < clipb ; i++)
    {
        /* i - src index, j - dst index, j is always <= i for obvious reasons */

        lpr = &rgnSrc->Buffer[i];

        /* Make sure the source rect is not retarded */
        ASSERT(lpr->bottom > lpr->top);
        ASSERT(lpr->right > lpr->left);

        /* We already checked above, this should hold true */
        ASSERT(lpr->bottom > rect->top);
        ASSERT(lpr->top < rect->bottom);

        /* Check if this rect is really inside the intersect rect */
        if ((lpr->left < rect->right) && (lpr->right > rect->left))
        {
            rpr = &rgnDst->Buffer[j];

            /* Crop the rect with the intersect rect */
            rpr->top = max(lpr->top, rect->top);
            rpr->bottom = min(lpr->bottom, rect->bottom);
            rpr->left = max(lpr->left, rect->left);
            rpr->right = min(lpr->right, rect->right);

            /* Make sure the resulting rect is not retarded */
            ASSERT(rpr->bottom > rpr->top);
            ASSERT(rpr->right > rpr->left);

            /* Track new bounds */
            if (rpr->left < left) left = rpr->left;
            if (rpr->right > right) right = rpr->right;
            if (rpr->top < top) top = rpr->top;
            if (rpr->bottom > bottom) bottom = rpr->bottom;

            /* Next target rect */
            j++;
        }
    }

    if (j == 0) goto empty;

    /* Update the bounds rect */
    rgnDst->rdh.rcBound.left = left;
    rgnDst->rdh.rcBound.right = right;
    rgnDst->rdh.rcBound.top = top;
    rgnDst->rdh.rcBound.bottom = bottom;

    /* Set new rect count */
    rgnDst->rdh.nCount = j;

    return REGION_Complexity(rgnDst);

empty:
    if (rgnDst->Buffer == NULL)
    {
        rgnDst->Buffer = &rgnDst->rdh.rcBound;
    }

    EMPTY_REGION(rgnDst);
    return NULLREGION;
}

/*!
 *      Attempt to merge the rects in the current band with those in the
 *      previous one. Used only by REGION_RegionOp.
 *
 * Results:
 *      The new index for the previous band.
 *
 * \note Side Effects:
 *      If coalescing takes place:
 *          - rectangles in the previous band will have their bottom fields
 *            altered.
 *          - pReg->numRects will be decreased.
 *
 */
static
INT
FASTCALL
REGION_Coalesce(
    PREGION pReg,  /* Region to coalesce */
    INT prevStart, /* Index of start of previous band */
    INT curStart)  /* Index of start of current band */
{
    RECTL *pPrevRect;  /* Current rect in previous band */
    RECTL *pCurRect;   /* Current rect in current band */
    RECTL *pRegEnd;    /* End of region */
    INT curNumRects;   /* Number of rectangles in current band */
    INT prevNumRects;  /* Number of rectangles in previous band */
    INT bandtop;       /* Top coordinate for current band */

    pRegEnd = pReg->Buffer + pReg->rdh.nCount;
    pPrevRect = pReg->Buffer + prevStart;
    prevNumRects = curStart - prevStart;

    /* Figure out how many rectangles are in the current band. Have to do
     * this because multiple bands could have been added in REGION_RegionOp
     * at the end when one region has been exhausted. */
    pCurRect = pReg->Buffer + curStart;
    bandtop = pCurRect->top;
    for (curNumRects = 0;
         (pCurRect != pRegEnd) && (pCurRect->top == bandtop);
         curNumRects++)
    {
        pCurRect++;
    }

    if (pCurRect != pRegEnd)
    {
        /* If more than one band was added, we have to find the start
         * of the last band added so the next coalescing job can start
         * at the right place... (given when multiple bands are added,
         * this may be pointless -- see above). */
        pRegEnd--;
        while ((pRegEnd-1)->top == pRegEnd->top)
        {
            pRegEnd--;
        }

        curStart = pRegEnd - pReg->Buffer;
        pRegEnd = pReg->Buffer + pReg->rdh.nCount;
    }

    if ((curNumRects == prevNumRects) && (curNumRects != 0))
    {
        pCurRect -= curNumRects;

        /* The bands may only be coalesced if the bottom of the previous
         * matches the top scanline of the current. */
        if (pPrevRect->bottom == pCurRect->top)
        {
            /* Make sure the bands have rects in the same places. This
             * assumes that rects have been added in such a way that they
             * cover the most area possible. I.e. two rects in a band must
             * have some horizontal space between them. */
            do
            {
                if ((pPrevRect->left != pCurRect->left) ||
                    (pPrevRect->right != pCurRect->right))
                {
                    /* The bands don't line up so they can't be coalesced. */
                    return (curStart);
                }

                pPrevRect++;
                pCurRect++;
                prevNumRects -= 1;
            }
            while (prevNumRects != 0);

            pReg->rdh.nCount -= curNumRects;
            pCurRect -= curNumRects;
            pPrevRect -= curNumRects;

            /* The bands may be merged, so set the bottom of each rect
             * in the previous band to that of the corresponding rect in
             * the current band. */
            do
            {
                pPrevRect->bottom = pCurRect->bottom;
                pPrevRect++;
                pCurRect++;
                curNumRects -= 1;
            }
            while (curNumRects != 0);

            /* If only one band was added to the region, we have to backup
             * curStart to the start of the previous band.
             *
             * If more than one band was added to the region, copy the
             * other bands down. The assumption here is that the other bands
             * came from the same region as the current one and no further
             * coalescing can be done on them since it's all been done
             * already... curStart is already in the right place. */
            if (pCurRect == pRegEnd)
            {
                curStart = prevStart;
            }
            else
            {
                do
                {
                    *pPrevRect++ = *pCurRect++;
                }
                while (pCurRect != pRegEnd);
            }
        }
    }

    return (curStart);
}

/*!
 *      Apply an operation to two regions. Called by REGION_Union,
 *      REGION_Inverse, REGION_Subtract, REGION_Intersect...
 *
 * Results:
 *      None.
 *
 * Side Effects:
 *      The new region is overwritten.
 *
 *\note The idea behind this function is to view the two regions as sets.
 *      Together they cover a rectangle of area that this function divides
 *      into horizontal bands where points are covered only by one region
 *      or by both. For the first case, the nonOverlapFunc is called with
 *      each the band and the band's upper and lower extents. For the
 *      second, the overlapFunc is called to process the entire band. It
 *      is responsible for clipping the rectangles in the band, though
 *      this function provides the boundaries.
 *      At the end of each band, the new region is coalesced, if possible,
 *      to reduce the number of rectangles in the region.
 *
 */
static
BOOL
FASTCALL
REGION_RegionOp(
    PREGION newReg, /* Place to store result */
    PREGION reg1,   /* First region in operation */
    PREGION reg2,   /* 2nd region in operation */
    overlapProcp overlapFunc,     /* Function to call for over-lapping bands */
    nonOverlapProcp nonOverlap1Func, /* Function to call for non-overlapping bands in region 1 */
    nonOverlapProcp nonOverlap2Func)  /* Function to call for non-overlapping bands in region 2 */
{
    RECTL *r1;                         /* Pointer into first region */
    RECTL *r2;                         /* Pointer into 2d region */
    RECTL *r1End;                      /* End of 1st region */
    RECTL *r2End;                      /* End of 2d region */
    INT ybot;                          /* Bottom of intersection */
    INT ytop;                          /* Top of intersection */
    RECTL *oldRects;                   /* Old rects for newReg */
    ULONG prevBand;                    /* Index of start of
                                        * Previous band in newReg */
    ULONG curBand;                     /* Index of start of current band in newReg */
    RECTL *r1BandEnd;                  /* End of current band in r1 */
    RECTL *r2BandEnd;                  /* End of current band in r2 */
    ULONG top;                         /* Top of non-overlapping band */
    ULONG bot;                         /* Bottom of non-overlapping band */

    /* Initialization:
     *  set r1, r2, r1End and r2End appropriately, preserve the important
     * parts of the destination region until the end in case it's one of
     * the two source regions, then mark the "new" region empty, allocating
     * another array of rectangles for it to use. */
    r1 = reg1->Buffer;
    r2 = reg2->Buffer;
    r1End = r1 + reg1->rdh.nCount;
    r2End = r2 + reg2->rdh.nCount;

    /* newReg may be one of the src regions so we can't empty it. We keep a
     * note of its rects pointer (so that we can free them later), preserve its
     * extents and simply set numRects to zero. */
    oldRects = newReg->Buffer;
    newReg->rdh.nCount = 0;

    /* Allocate a reasonable number of rectangles for the new region. The idea
     * is to allocate enough so the individual functions don't need to
     * reallocate and copy the array, which is time consuming, yet we don't
     * have to worry about using too much memory. I hope to be able to
     * nuke the Xrealloc() at the end of this function eventually. */
    newReg->rdh.nRgnSize = max(reg1->rdh.nCount + 1, reg2->rdh.nCount) * 2 * sizeof(RECT);

    newReg->Buffer = ExAllocatePoolWithTag(PagedPool,
                                           newReg->rdh.nRgnSize,
                                           TAG_REGION);
    if (newReg->Buffer == NULL)
    {
        newReg->rdh.nRgnSize = 0;
        return FALSE;
    }

    /* Initialize ybot and ytop.
     * In the upcoming loop, ybot and ytop serve different functions depending
     * on whether the band being handled is an overlapping or non-overlapping
     * band.
     *  In the case of a non-overlapping band (only one of the regions
     * has points in the band), ybot is the bottom of the most recent
     * intersection and thus clips the top of the rectangles in that band.
     * ytop is the top of the next intersection between the two regions and
     * serves to clip the bottom of the rectangles in the current band.
     *  For an overlapping band (where the two regions intersect), ytop clips
     * the top of the rectangles of both regions and ybot clips the bottoms. */
    if (reg1->rdh.rcBound.top < reg2->rdh.rcBound.top)
        ybot = reg1->rdh.rcBound.top;
    else
        ybot = reg2->rdh.rcBound.top;

    /* prevBand serves to mark the start of the previous band so rectangles
     * can be coalesced into larger rectangles. qv. miCoalesce, above.
     * In the beginning, there is no previous band, so prevBand == curBand
     * (curBand is set later on, of course, but the first band will always
     * start at index 0). prevBand and curBand must be indices because of
     * the possible expansion, and resultant moving, of the new region's
     * array of rectangles. */
    prevBand = 0;
    do
    {
        curBand = newReg->rdh.nCount;

        /* This algorithm proceeds one source-band (as opposed to a
         * destination band, which is determined by where the two regions
         * intersect) at a time. r1BandEnd and r2BandEnd serve to mark the
         * rectangle after the last one in the current band for their
         * respective regions. */
        r1BandEnd = r1;
        while ((r1BandEnd != r1End) && (r1BandEnd->top == r1->top))
        {
            r1BandEnd++;
        }

        r2BandEnd = r2;
        while ((r2BandEnd != r2End) && (r2BandEnd->top == r2->top))
        {
            r2BandEnd++;
        }

        /* First handle the band that doesn't intersect, if any.
         *
         * Note that attention is restricted to one band in the
         * non-intersecting region at once, so if a region has n
         * bands between the current position and the next place it overlaps
         * the other, this entire loop will be passed through n times. */
        if (r1->top < r2->top)
        {
            top = max(r1->top,ybot);
            bot = min(r1->bottom,r2->top);

            if ((top != bot) && (nonOverlap1Func != NULL))
            {
                if (!(*nonOverlap1Func)(newReg, r1, r1BandEnd, top, bot)) return FALSE;
            }

            ytop = r2->top;
        }
        else if (r2->top < r1->top)
        {
            top = max(r2->top,ybot);
            bot = min(r2->bottom,r1->top);

            if ((top != bot) && (nonOverlap2Func != NULL))
            {
                if (!(*nonOverlap2Func)(newReg, r2, r2BandEnd, top, bot) ) return FALSE;
            }

            ytop = r1->top;
        }
        else
        {
            ytop = r1->top;
        }

        /* If any rectangles got added to the region, try and coalesce them
         * with rectangles from the previous band. Note we could just do
         * this test in miCoalesce, but some machines incur a not
         * inconsiderable cost for function calls, so... */
        if (newReg->rdh.nCount != curBand)
        {
            prevBand = REGION_Coalesce(newReg, prevBand, curBand);
        }

        /* Now see if we've hit an intersecting band. The two bands only
         * intersect if ybot > ytop */
        ybot = min(r1->bottom, r2->bottom);
        curBand = newReg->rdh.nCount;
        if (ybot > ytop)
        {
            if (!(*overlapFunc)(newReg, r1, r1BandEnd, r2, r2BandEnd, ytop, ybot)) return FALSE;
        }

        if (newReg->rdh.nCount != curBand)
        {
            prevBand = REGION_Coalesce(newReg, prevBand, curBand);
        }

        /* If we've finished with a band (bottom == ybot) we skip forward
         * in the region to the next band. */
        if (r1->bottom == ybot)
        {
            r1 = r1BandEnd;
        }
        if (r2->bottom == ybot)
        {
            r2 = r2BandEnd;
        }
    }
    while ((r1 != r1End) && (r2 != r2End));

    /* Deal with whichever region still has rectangles left. */
    curBand = newReg->rdh.nCount;
    if (r1 != r1End)
    {
        if (nonOverlap1Func != NULL)
        {
            do
            {
                r1BandEnd = r1;
                while ((r1BandEnd < r1End) && (r1BandEnd->top == r1->top))
                {
                    r1BandEnd++;
                }

                if (!(*nonOverlap1Func)(newReg,
                                   r1,
                                   r1BandEnd,
                                   max(r1->top,ybot),
                                   r1->bottom))
                    return FALSE;
                r1 = r1BandEnd;
            }
            while (r1 != r1End);
        }
    }
    else if ((r2 != r2End) && (nonOverlap2Func != NULL))
    {
        do
        {
            r2BandEnd = r2;
            while ((r2BandEnd < r2End) && (r2BandEnd->top == r2->top))
            {
                r2BandEnd++;
            }

            if (!(*nonOverlap2Func)(newReg,
                               r2,
                               r2BandEnd,
                               max(r2->top,ybot),
                               r2->bottom))
                return FALSE;
            r2 = r2BandEnd;
        }
        while (r2 != r2End);
    }

    if (newReg->rdh.nCount != curBand)
    {
        (VOID)REGION_Coalesce(newReg, prevBand, curBand);
    }

    /* A bit of cleanup. To keep regions from growing without bound,
     * we shrink the array of rectangles to match the new number of
     * rectangles in the region. This never goes to 0, however...
     *
     * Only do this stuff if the number of rectangles allocated is more than
     * twice the number of rectangles in the region (a simple optimization...). */
    if ((newReg->rdh.nRgnSize > (2 * newReg->rdh.nCount * sizeof(RECT))) &&
        (newReg->rdh.nCount > 2))
    {
        if (REGION_NOT_EMPTY(newReg))
        {
            RECTL *prev_rects = newReg->Buffer;
            newReg->Buffer = ExAllocatePoolWithTag(PagedPool,
                                                   newReg->rdh.nCount * sizeof(RECT),
                                                   TAG_REGION);

            if (newReg->Buffer == NULL)
            {
                newReg->Buffer = prev_rects;
            }
            else
            {
                newReg->rdh.nRgnSize = newReg->rdh.nCount*sizeof(RECT);
                COPY_RECTS(newReg->Buffer, prev_rects, newReg->rdh.nCount);
                if (prev_rects != &newReg->rdh.rcBound)
                    ExFreePoolWithTag(prev_rects, TAG_REGION);
            }
        }
        else
        {
            /* No point in doing the extra work involved in an Xrealloc if
             * the region is empty */
            newReg->rdh.nRgnSize = sizeof(RECT);
            if (newReg->Buffer != &newReg->rdh.rcBound)
                ExFreePoolWithTag(newReg->Buffer, TAG_REGION);

            newReg->Buffer = ExAllocatePoolWithTag(PagedPool,
                                                   sizeof(RECT),
                                                   TAG_REGION);
            ASSERT(newReg->Buffer);
        }
    }

    newReg->rdh.iType = RDH_RECTANGLES;

    if (oldRects != &newReg->rdh.rcBound)
        ExFreePoolWithTag(oldRects, TAG_REGION);
    return TRUE;
}

/***********************************************************************
 *          Region Intersection
 ***********************************************************************/


/*!
 * Handle an overlapping band for REGION_Intersect.
 *
 * Results:
 *      None.
 *
 * \note Side Effects:
 *      Rectangles may be added to the region.
 *
 */
static
BOOL
FASTCALL
REGION_IntersectO(
    PREGION pReg,
    PRECTL  r1,
    PRECTL  r1End,
    PRECTL  r2,
    PRECTL  r2End,
    INT     top,
    INT     bottom)
{
    INT left, right;

    while ((r1 != r1End) && (r2 != r2End))
    {
        left = max(r1->left, r2->left);
        right = min(r1->right, r2->right);

        /* If there's any overlap between the two rectangles, add that
         * overlap to the new region.
         * There's no need to check for subsumption because the only way
         * such a need could arise is if some region has two rectangles
         * right next to each other. Since that should never happen... */
        if (left < right)
        {
            if (!REGION_bAddRect(pReg, left, top, right, bottom))
            {
                return FALSE;
            }
        }

        /* Need to advance the pointers. Shift the one that extends
         * to the right the least, since the other still has a chance to
         * overlap with that region's next rectangle, if you see what I mean. */
        if (r1->right < r2->right)
        {
            r1++;
        }
        else if (r2->right < r1->right)
        {
            r2++;
        }
        else
        {
            r1++;
            r2++;
        }
    }

    return TRUE;
}
/* --- pretranslation split: the remaining function groups live in region.split2..4.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "region.split2.c"
#include "region.split3.c"
#include "region.split4.c"
