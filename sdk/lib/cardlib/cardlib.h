/*
 * PROJECT:     CardLib
 * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
 * PURPOSE:     Public interface of the playing-card rendering library
 * COPYRIGHT:   Copyright 2026 vibeos contributors
 *
 * A clean-room reimplementation of the interface of "CardLib" by J Brown
 * (catch22.net), which this project previously carried as freeware.  Credit for
 * the original design of this interface is his; none of its source was read or
 * consulted while writing this replacement.  See README.md in this directory.
 *
 * Implementation lives in sdk/lib/rust/cardlib-rust/ and is plain C ABI so that
 * it links into ordinary C and C++ callers.
 */

#ifndef CARDLIB_INCLUDED
#define CARDLIB_INCLUDED

#include <stdarg.h>
#include <stdlib.h>

#define WIN32_NO_STATUS
#include <windef.h>
#include <winbase.h>
#include <wingdi.h>
#include <winuser.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Calling conventions.  Two DIFFERENT ones on purpose, and mixing them up costs
 * a day of link errors on i386:
 *
 *   CARDLIBAPI  (cdecl)   - every function exported BY NAME from the library.
 *                           Rust's #[no_mangle] does not apply __stdcall's
 *                           `@n` decoration, so a stdcall export would be
 *                           spelled `CardLib_Foo` by Rust and looked up as
 *                           `_CardLib_Foo@8` by a 32-bit C caller.
 *   CARDLIBPROC (stdcall) - callbacks the caller hands to us as function
 *                           POINTERS.  No symbol name is involved, so only the
 *                           ABI has to agree, and stdcall is what the games
 *                           have always declared.
 */
#define CARDLIBAPI  __cdecl
#define CARDLIBPROC __stdcall

/* ------------------------------------------------------------------------ *
 *  Card
 * ------------------------------------------------------------------------ */

enum eSuit  { Clubs = 0, Diamonds = 1, Hearts = 2, Spades = 3 };
enum eValue { Ace = 1, Two = 2, Three = 3, Four = 4, Five = 5, Six = 6,
              Seven = 7, Eight = 8, Nine = 9, Ten = 10, Jack = 11,
              Queen = 12, King = 13 };

/*
 * A card is its 0-51 index plus a face-up flag.  The index packs value and suit
 * as `(value - 1) * 4 + suit`, i.e. VALUE-major: indices 0..3 are the four aces,
 * 4..7 the twos, and so on.  That encoding is load bearing - the card artwork is
 * addressed from it (see CardLib_CardBitmapsDC) - so do not "tidy" it.
 */
typedef struct _CARD
{
    INT nIdx;       /* 0..51 */
    INT fFaceUp;    /* BOOL, but sized so the struct is two plain ints */
} CARD;

#define CARD_SUIT(idx)      ((idx) % 4)
#define CARD_LOVAL(idx)     ((idx) / 4 + 1)

CARD CARDLIBAPI Card_FromIndex(INT nIndex);           /* clamps to 0..51 */
CARD CARDLIBAPI Card_Make(INT nValue, INT nSuit);     /* value 1..13, suit 0..3 */

INT  CARDLIBAPI Card_Suit(CARD card);                 /* 0..3, see eSuit */
INT  CARDLIBAPI Card_LoVal(CARD card);                /* ace low:  1..13 */
INT  CARDLIBAPI Card_HiVal(CARD card);                /* ace high: 2..14 */
INT  CARDLIBAPI Card_Idx(CARD card);                  /* 0..51 */
BOOL CARDLIBAPI Card_IsRed(CARD card);
BOOL CARDLIBAPI Card_IsBlack(CARD card);

/* ------------------------------------------------------------------------ *
 *  CardStack - an ordered pile, by value
 * ------------------------------------------------------------------------ */

#define MAX_CARDSTACK_SIZE 128

/*
 * Plain value type: callers copy it with `=` and keep their own scratch piles on
 * the stack.  1 KB a piece, which is deliberate - it keeps the whole library
 * allocation-free.
 *
 * INDEX 0 IS THE TOP OF THE PILE, i.e. the card a player would take next, and
 * index NumCards()-1 is the bottom.  Every caller assumes this.
 */
typedef struct _CARDSTACK
{
    CARD cards[MAX_CARDSTACK_SIZE];
    INT  nNumCards;
} CARDSTACK;

void CARDLIBAPI CardStack_Clear   (CARDSTACK *pStack);
void CARDLIBAPI CardStack_NewDeck (CARDSTACK *pStack);  /* 52 cards, ordered */
void CARDLIBAPI CardStack_Shuffle (CARDSTACK *pStack);
void CARDLIBAPI CardStack_Reverse (CARDSTACK *pStack);
INT  CARDLIBAPI CardStack_NumCards(const CARDSTACK *pStack);

void CARDLIBAPI CardStack_PushCard (CARDSTACK *pStack, CARD card);
void CARDLIBAPI CardStack_PushStack(CARDSTACK *pStack, const CARDSTACK *pPush);

/* Pop removes from the TOP.  PopN's output keeps the source's order, so the
 * card that was on top is still at index 0 of the result. */
CARD CARDLIBAPI CardStack_PopCard(CARDSTACK *pStack);
void CARDLIBAPI CardStack_PopN   (CARDSTACK *pStack, INT nItems, CARDSTACK *pOut);

/* Top* peek without removing. */
CARD CARDLIBAPI CardStack_TopCard(const CARDSTACK *pStack);
void CARDLIBAPI CardStack_TopN   (const CARDSTACK *pStack, INT nItems, CARDSTACK *pOut);

CARD CARDLIBAPI CardStack_Get       (const CARDSTACK *pStack, INT nIndex);
CARD CARDLIBAPI CardStack_RemoveCard(CARDSTACK *pStack, INT nIndex);

/* ------------------------------------------------------------------------ *
 *  Opaque objects
 * ------------------------------------------------------------------------ */

typedef struct _CARDWINDOW CARDWINDOW;
typedef struct _CARDREGION CARDREGION;

/* ------------------------------------------------------------------------ *
 *  Callbacks
 * ------------------------------------------------------------------------ */

typedef BOOL (CARDLIBPROC *PFNCARDCANDRAG)(CARDREGION *pRegion, INT nNumDragging);

/*
 * NOTE pDragCards is NOT const, and that is the whole point: a drop handler is
 * allowed to rewrite the cards being dropped before they land.  Spider empties
 * it (leaving nNumCards == 0) when the drop completes a K-to-A run that it has
 * just removed from the board by hand.  Returning TRUE then commits whatever is
 * left, including nothing.
 */
typedef BOOL (CARDLIBPROC *PFNCARDCANDROP)(CARDREGION *pRegion, CARDSTACK *pDragCards);

typedef void (CARDLIBPROC *PFNCARDCLICK) (CARDREGION *pRegion, INT nNumCards);
typedef void (CARDLIBPROC *PFNCARDADD)   (CARDREGION *pRegion, const CARDSTACK *pAdded);
typedef void (CARDLIBPROC *PFNCARDREMOVE)(CARDREGION *pRegion, INT nNumRemoved);

/* ------------------------------------------------------------------------ *
 *  Region behaviour constants
 * ------------------------------------------------------------------------ */

/* Empty-pile artwork.  0 and 1 are drawn by us; any other value is an index
 * into the card bitmap strip (so CS_EI_X and CS_EI_CIRC below are strip slots,
 * not resource ids).  This means strip slots 0 and 1 - the ace of clubs and the
 * ace of diamonds - cannot be used as empty images.  Nobody wants to. */
#define CS_EI_NONE          0
#define CS_EI_SUNK          1
#define CS_EI_X             66      /* red X on the table green */
#define CS_EI_CIRC          67      /* green O on the table green */

#define CS_DEFXOFF          12
#define CS_DEFYOFF          18
#define CS_NO3D             1
#define CS_DEF3D            10

#define CS_DRAG_NONE        0
#define CS_DRAG_TOP         1
#define CS_DRAG_ALL         2
#define CS_DRAG_CALLBACK    3

#define CS_DROP_NONE        0
#define CS_DROP_ALL         1
#define CS_DROP_CALLBACK    2

#define CS_XJUST_NONE       0
#define CS_XJUST_RIGHT      1
#define CS_XJUST_CENTER     2

#define CS_YJUST_NONE       0
#define CS_YJUST_BOTTOM     1
#define CS_YJUST_CENTER     2

#define CS_FACE_UP          0   /* every card face-up */
#define CS_FACE_DOWN        1   /* every card face-down */
#define CS_FACE_DOWNUP      2   /* bottom N face-down, the rest face-up */
#define CS_FACE_UPDOWN      3   /* bottom N face-up, the rest face-down */
#define CS_FACE_ANY         4   /* leave each card as it is */

/*
 * Card backs, as slots in the bitmap strip.  The twelve consecutive values are
 * what the games' back-picker dialogs walk, and they are the only part of the
 * tail of the strip anyone uses.
 */
#define ecbCROSSHATCH       53
#define ecbWEAVE1           54
#define ecbWEAVE2           55
#define ecbROBOT            56
#define ecbFLOWERS          57
#define ecbVINE1            58
#define ecbVINE2            59
#define ecbFISH1            60
#define ecbFISH2            61
#define ecbSHELLS           62
#define ecbCASTLE           63
#define ecbISLAND           64

/* ------------------------------------------------------------------------ *
 *  Library lifetime and shared state
 * ------------------------------------------------------------------------ */

/*
 * Loads cards.dll, composes its 68 bitmaps into one wide device context and
 * registers the window class.  Call once before anything else; returns FALSE if
 * the artwork could not be loaded, and the caller should not continue.
 */
BOOL CARDLIBAPI CardLib_Init(void);
void CARDLIBAPI CardLib_Term(void);

INT CARDLIBAPI CardLib_CardWidth(void);
INT CARDLIBAPI CardLib_CardHeight(void);

/*
 * The composed strip: 68 cards side by side, each CardLib_CardWidth() wide, so
 * slot N starts at x = N * CardLib_CardWidth().  Slots 0..51 are the faces
 * addressed by Card_Idx(); 52 is the free-cell placeholder; 53..64 are the card
 * backs; 65 is the joker; 66 and 67 are the X and O placeholders.  Callers blit
 * out of it directly to draw card previews.
 */
HDC CARDLIBAPI CardLib_CardBitmapsDC(void);

/* Milliseconds a move animation takes.  Default is fine; exposed for tests. */
void CARDLIBAPI CardLib_SetZoomSpeed(INT nSpeed);

/* ------------------------------------------------------------------------ *
 *  CardWindow
 * ------------------------------------------------------------------------ */

CARDWINDOW *CARDLIBAPI CardWindow_Create(HWND hwndParent, DWORD dwExStyle, DWORD dwStyle,
                                         INT x, INT y, INT nWidth, INT nHeight);
void        CARDLIBAPI CardWindow_Destroy(CARDWINDOW *pWnd);

HWND CARDLIBAPI CardWindow_GetHwnd(CARDWINDOW *pWnd);

CARDREGION *CARDLIBAPI CardWindow_CreateRegion(CARDWINDOW *pWnd, INT nId, BOOL fVisible,
                                               INT x, INT y, INT xOffset, INT yOffset);

/* Empties every region's pile.  Does not destroy the regions. */
void CARDLIBAPI CardWindow_EmptyStacks(CARDWINDOW *pWnd);

void CARDLIBAPI CardWindow_Redraw(CARDWINDOW *pWnd);

void CARDLIBAPI CardWindow_SetBackCardIdx(CARDWINDOW *pWnd, UINT uBackIdx);
UINT CARDLIBAPI CardWindow_GetBackCardIdx(CARDWINDOW *pWnd);

/* ------------------------------------------------------------------------ *
 *  CardRegion
 * ------------------------------------------------------------------------ */

INT CARDLIBAPI CardRegion_Id(CARDREGION *pRegion);

/* Layout */
void CARDLIBAPI CardRegion_SetOffsets     (CARDREGION *pRegion, INT x, INT y);
void CARDLIBAPI CardRegion_SetPlacement   (CARDREGION *pRegion, UINT xJustify, UINT yJustify,
                                           INT xAdjust, INT yAdjust);
void CARDLIBAPI CardRegion_SetEmptyImage  (CARDREGION *pRegion, UINT uImage);
BOOL CARDLIBAPI CardRegion_SetThreedCount (CARDREGION *pRegion, INT nCount);

/* Which cards show their face.  nOption is the count for CS_FACE_DOWNUP and
 * CS_FACE_UPDOWN, and ignored otherwise. */
void CARDLIBAPI CardRegion_SetFaceDirection(CARDREGION *pRegion, UINT uDirType, INT nOption);
UINT CARDLIBAPI CardRegion_GetFaceDirection(CARDREGION *pRegion, INT *pnOption);

/* Policy */
BOOL CARDLIBAPI CardRegion_SetDragRule(CARDREGION *pRegion, UINT uDragType, PFNCARDCANDRAG pfn);
BOOL CARDLIBAPI CardRegion_SetDropRule(CARDREGION *pRegion, UINT uDropType, PFNCARDCANDROP pfn);

void CARDLIBAPI CardRegion_SetClickProc       (CARDREGION *pRegion, PFNCARDCLICK pfn);
void CARDLIBAPI CardRegion_SetClickReleaseProc(CARDREGION *pRegion, PFNCARDCLICK pfn);
void CARDLIBAPI CardRegion_SetDblClickProc    (CARDREGION *pRegion, PFNCARDCLICK pfn);
void CARDLIBAPI CardRegion_SetAddCardProc     (CARDREGION *pRegion, PFNCARDADD pfn);
void CARDLIBAPI CardRegion_SetRemoveCardProc  (CARDREGION *pRegion, PFNCARDREMOVE pfn);

/* Contents.  The pointer from GetCardStack is owned by the region and is valid
 * until the next call that changes it - copy it if you intend to keep it. */
void             CARDLIBAPI CardRegion_SetCardStack(CARDREGION *pRegion, const CARDSTACK *pStack);
const CARDSTACK *CARDLIBAPI CardRegion_GetCardStack(CARDREGION *pRegion);

INT  CARDLIBAPI CardRegion_NumCards (CARDREGION *pRegion);
void CARDLIBAPI CardRegion_Clear    (CARDREGION *pRegion);
void CARDLIBAPI CardRegion_Reverse  (CARDREGION *pRegion);
void CARDLIBAPI CardRegion_PushCard (CARDREGION *pRegion, CARD card);
void CARDLIBAPI CardRegion_PushStack(CARDREGION *pRegion, const CARDSTACK *pPush);
CARD CARDLIBAPI CardRegion_PopCard  (CARDREGION *pRegion);
void CARDLIBAPI CardRegion_PopN     (CARDREGION *pRegion, INT nItems, CARDSTACK *pOut);
void CARDLIBAPI CardRegion_TopN     (CARDREGION *pRegion, INT nItems, CARDSTACK *pOut);

/* Painting.  Update recomputes the region's size from its contents and repaints
 * it; Redraw just repaints. */
void CARDLIBAPI CardRegion_Update(CARDREGION *pRegion);
void CARDLIBAPI CardRegion_Redraw(CARDREGION *pRegion);

/* Blink the pile nCount times, nTimeout ms apart.  Used for the win flourish. */
void CARDLIBAPI CardRegion_Flash(CARDREGION *pRegion, INT nCount, INT nTimeout);

/*
 * Move nNumCards off the top of pRegion onto pDest.
 *
 *   MoveCard     - moves them and repaints.  No drag/drop callbacks fire.
 *   SimulateDrag - runs the cards through the same path a mouse drag takes, so
 *                  the destination's drop rule is consulted and its add
 *                  callback fires.  Solitaire's double-click-to-foundation
 *                  depends on that callback to notice the game is won.
 */
BOOL CARDLIBAPI CardRegion_MoveCard   (CARDREGION *pRegion, CARDREGION *pDest,
                                       INT nNumCards, BOOL fAnimate);
BOOL CARDLIBAPI CardRegion_SimulateDrag(CARDREGION *pRegion, CARDREGION *pDest,
                                        INT nNumCards, BOOL fAnimate);

#ifdef __cplusplus
}   /* extern "C" */
#endif

#endif /* CARDLIB_INCLUDED */
