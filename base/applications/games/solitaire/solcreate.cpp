/*
 * PROJECT:     Solitaire
 * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
 * FILE:        base/applications/games/solitaire/solcreate.cpp
 * PURPOSE:     Builds the Klondike table: the thirteen card regions, where
 *              they sit and what the player may do with each of them
 * PROGRAMMER:  Copyright 2026 vibeos contributors
 */

#include "solitaire.h"

CARDREGION *pDeck;
CARDREGION *pPile;
CARDREGION *pSuitStack[NUM_SUIT_STACKS];
CARDREGION *pRowStack[NUM_ROW_STACKS];

INT yRowStackCardOffset;
INT xPileAdjust;

/*
 * The table is one row of seven columns wide, and everything lines up with
 * those columns:
 *
 *      col 0     col 1     col 2     col 3  col 4  col 5  col 6
 *      [stock]   [waste]             [ 4 foundations, one per suit ]
 *      [ 1 ]     [ 2 ]     [ 3 ]     [ 4 ]  [ 5 ]  [ 6 ]  [ 7 ]      tableau
 *
 * Every region is CS_XJUST_CENTER with a per-column displacement, so the whole
 * layout stays centred and evenly spaced whatever the window size, and card
 * dimensions come from the library rather than being hardcoded - the artwork
 * has a second, much larger deck style.
 */
static INT ColumnAdjust(INT nColumn)
{
    return (nColumn - NUM_ROW_STACKS / 2) * (CardLib_CardWidth() + X_BORDER);
}

void CreateSol(void)
{
    INT i;
    INT yTopRow = Y_BORDER;
    INT yTableau = Y_BORDER + CardLib_CardHeight() + Y_BORDER;

    /*
     * How far each buried tableau card peeks out from under the one above it.
     * A sixth of the card height leaves the rank and suit in the top-left
     * corner readable, and derives from the artwork so the big deck fans
     * proportionally rather than overlapping into an unreadable stripe.
     */
    yRowStackCardOffset = CardLib_CardHeight() / 6;
    if (yRowStackCardOffset < 1)
        yRowStackCardOffset = 1;

    xPileAdjust = ColumnAdjust(1);

    /* ---- the stock -------------------------------------------------- *
     * No fan and no drag: the only thing a player does to it is click it.
     * The threed count makes a 24-card stock four blits deep instead of
     * twenty-four, all landing in the same place.
     */
    pDeck = CardWindow_CreateRegion(SolWnd, DECK_ID, TRUE, 0, yTopRow, 0, 0);
    CardRegion_SetFaceDirection(pDeck, CS_FACE_DOWN, 0);
    CardRegion_SetPlacement(pDeck, CS_XJUST_CENTER, CS_YJUST_NONE, ColumnAdjust(0), 0);
    CardRegion_SetEmptyImage(pDeck, CS_EI_CIRC);
    CardRegion_SetThreedCount(pDeck, 6);
    CardRegion_SetDragRule(pDeck, CS_DRAG_NONE, NULL);
    CardRegion_SetDropRule(pDeck, CS_DROP_NONE, NULL);
    CardRegion_SetClickProc(pDeck, DeckClickProc);

    /* ---- the waste --------------------------------------------------- *
     * Offsets are set per deal by UpdatePileDisplay: squared up when one card
     * is turned at a time, fanned right when three are.  Only the top card
     * ever leaves, and nothing may be dropped back on it.
     */
    pPile = CardWindow_CreateRegion(SolWnd, PILE_ID, TRUE, 0, yTopRow, 0, 0);
    CardRegion_SetFaceDirection(pPile, CS_FACE_UP, 0);
    CardRegion_SetPlacement(pPile, CS_XJUST_CENTER, CS_YJUST_NONE, xPileAdjust, 0);
    CardRegion_SetEmptyImage(pPile, CS_EI_NONE);
    CardRegion_SetDragRule(pPile, CS_DRAG_CALLBACK, PileDragProc);
    CardRegion_SetDropRule(pPile, CS_DROP_NONE, NULL);
    CardRegion_SetDblClickProc(pPile, PileDblClickProc);
    CardRegion_SetRemoveCardProc(pPile, PileRemoveProc);

    /* ---- the foundations --------------------------------------------- *
     * Ace up to King in suit.  Squared up, so a threed count of a full suit
     * reduces a thirteen-card pile to the single image you can actually see.
     * The add callback is where the win is noticed, which is why the
     * double-click shortcut has to go through CardRegion_SimulateDrag.
     */
    for (i = 0; i < NUM_SUIT_STACKS; i++)
    {
        pSuitStack[i] = CardWindow_CreateRegion(SolWnd, SUIT_ID + i, TRUE, 0, yTopRow, 0, 0);
        CardRegion_SetFaceDirection(pSuitStack[i], CS_FACE_UP, 0);
        CardRegion_SetPlacement(pSuitStack[i], CS_XJUST_CENTER, CS_YJUST_NONE,
                                ColumnAdjust(NUM_ROW_STACKS - NUM_SUIT_STACKS + i), 0);
        CardRegion_SetEmptyImage(pSuitStack[i], CS_EI_SUNK);
        CardRegion_SetThreedCount(pSuitStack[i], 13);
        CardRegion_SetDragRule(pSuitStack[i], CS_DRAG_CALLBACK, SuitStackDragProc);
        CardRegion_SetDropRule(pSuitStack[i], CS_DROP_CALLBACK, SuitStackDropProc);
        CardRegion_SetAddCardProc(pSuitStack[i], SuitStackAddProc);
    }

    /* ---- the tableau -------------------------------------------------- *
     * Created last so they sit on top: a long column fans downwards and must
     * win the hit test against anything it grows over.  CS_FACE_DOWNUP with
     * the count of still-hidden cards is the whole face-up/face-down model;
     * the remove callback decrements it when a column is stripped bare.
     */
    for (i = 0; i < NUM_ROW_STACKS; i++)
    {
        pRowStack[i] = CardWindow_CreateRegion(SolWnd, ROW_ID + i, TRUE,
                                               0, yTableau, 0, yRowStackCardOffset);
        CardRegion_SetFaceDirection(pRowStack[i], CS_FACE_DOWNUP, 0);
        CardRegion_SetPlacement(pRowStack[i], CS_XJUST_CENTER, CS_YJUST_NONE, ColumnAdjust(i), 0);
        CardRegion_SetEmptyImage(pRowStack[i], CS_EI_SUNK);
        CardRegion_SetDragRule(pRowStack[i], CS_DRAG_CALLBACK, RowStackDragProc);
        CardRegion_SetDropRule(pRowStack[i], CS_DROP_CALLBACK, RowStackDropProc);
        CardRegion_SetClickProc(pRowStack[i], RowStackClickProc);
        CardRegion_SetDblClickProc(pRowStack[i], RowStackDblClickProc);
        CardRegion_SetRemoveCardProc(pRowStack[i], RowStackRemoveProc);
    }
}
