/*
 * PROJECT:     Solitaire
 * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
 * FILE:        base/applications/games/solitaire/solgame.cpp
 * PURPOSE:     The rules of Klondike: dealing, the drag/drop and click
 *              callbacks the card library asks the game, scoring, and the win
 * PROGRAMMER:  Copyright 2026 vibeos contributors
 */

#include "solitaire.h"

CARDSTACK activepile;
INT       VisiblePileCards;

/*
 * Where the cards currently in flight came from.
 *
 * The library hands a drop rule the DESTINATION and the cards, never the
 * source, so every draggable pile records itself here from its drag rule.
 * CardRegion_SimulateDrag does not run the drag rule, so the double-click
 * shortcut sets this by hand before it calls it.
 */
static CARDREGION *pDragSource;

/* ---------------------------------------------------------------------- *
 *  Small helpers
 * ---------------------------------------------------------------------- */

static void StartPlay(void)
{
    if (!fGameStarted)
    {
        fGameStarted = TRUE;
        StartPlayTimer();
    }
}

/*
 * The stock shows a circle while there is still a waste to recycle, and a
 * cross once the cycle is genuinely over.
 */
static void UpdateDeckImage(void)
{
    CardRegion_SetEmptyImage(pDeck,
        (CardStack_NumCards(&activepile) > 0) ? CS_EI_CIRC : CS_EI_X);
}

/*
 * CS_XJUST_CENTER centres a region's BOUNDING BOX, and a three-card fan is two
 * offsets wider than a single card - so without compensating, the waste would
 * slide left as it fans and back right as it empties.  Called after every
 * change to the waste, and again after an undo, which restores
 * VisiblePileCards behind our back.
 */
void UpdatePilePlacement(void)
{
    INT nAdjust = xPileAdjust;

    if (VisiblePileCards > 1)
        nAdjust += ((VisiblePileCards - 1) * CS_DEFXOFF) / 2;

    CardRegion_SetPlacement(pPile, CS_XJUST_CENTER, CS_YJUST_NONE, nAdjust, 0);
}

/*
 * Rebuild the pPile region from activepile so that nVisible cards show.
 *
 * The two shapes here are exactly the two solundo.cpp rebuilds after an undo,
 * and they have to agree: with one card showing the region holds the WHOLE
 * waste with no offset (so it draws as one square pile), and with more it
 * holds only the fanned top few.  Either way activepile is the truth and the
 * region is the view of it.
 */
void UpdatePileDisplay(INT nVisible)
{
    CARDSTACK tmp;
    INT nTotal = CardStack_NumCards(&activepile);

    if (nVisible > nTotal)
        nVisible = nTotal;
    if (nVisible < 1)
        nVisible = (nTotal > 0) ? 1 : 0;

    CardRegion_Clear(pPile);

    if (nVisible <= 1)
    {
        CardRegion_SetOffsets(pPile, 0, 0);
        CardRegion_SetCardStack(pPile, &activepile);
    }
    else
    {
        CardStack_TopN(&activepile, nVisible, &tmp);
        CardRegion_SetOffsets(pPile, CS_DEFXOFF, 1);
        CardRegion_PushStack(pPile, &tmp);
    }

    VisiblePileCards = nVisible;
    UpdatePilePlacement();
    UpdateDeckImage();
}

/*
 * A column whose face-up cards have all been played shows its next card.
 * Driven from the remove callback, so it fires for a mouse drag and for the
 * double-click shortcut alike; the click callback calls it too, as a backstop
 * for a column that somehow ends up presenting a face-down card.
 */
void TurnRowStackCard(CARDREGION *pRegion)
{
    INT nFaceDown;

    CardRegion_GetFaceDirection(pRegion, &nFaceDown);

    if (nFaceDown > 0 && CardRegion_NumCards(pRegion) <= nFaceDown)
    {
        nFaceDown--;
        CardRegion_SetFaceDirection(pRegion, CS_FACE_DOWNUP, nFaceDown);

        if (GetScoreMode() == SCORE_STD)
        {
            lScore += SCORE_STD_TURNOVER;
            UpdateStatusBar();
        }

        CardRegion_Redraw(pRegion);
    }
}

static BOOL IsSuitStackId(INT nId)
{
    return (nId >= SUIT_ID) && (nId < SUIT_ID + NUM_SUIT_STACKS);
}

/* Would this card be legal on this foundation right now? */
static BOOL CanDropOnSuitStack(CARDREGION *pRegion, CARD card)
{
    CARD top;

    if (CardRegion_NumCards(pRegion) == 0)
        return (Card_LoVal(card) == Ace);

    top = CardStack_TopCard(CardRegion_GetCardStack(pRegion));

    return (Card_Suit(top) == Card_Suit(card)) &&
           (Card_LoVal(top) + 1 == Card_LoVal(card));
}

static void ScoreMove(INT nSrcId, INT nDestId, INT nCards)
{
    BOOL fToSuit   = IsSuitStackId(nDestId);
    BOOL fFromSuit = IsSuitStackId(nSrcId);

    switch (GetScoreMode())
    {
        case SCORE_STD:
            if (fToSuit)
                lScore += SCORE_STD_TO_SUIT * nCards;
            else if (fFromSuit)
                lScore -= SCORE_STD_FROM_SUIT * nCards;
            else if (nSrcId == PILE_ID)
                lScore += SCORE_STD_PILE_TO_ROW * nCards;

            /* Standard scoring has a floor of zero, which is also the floor
               solundo.cpp clamps to when it charges for an undo. */
            if (lScore < 0)
                lScore = 0;
            break;

        case SCORE_VEGAS:
            /* Vegas is a bank, not a tally: it may legitimately go negative
               and must not be clamped. */
            if (fToSuit)
                lScore += SCORE_VEGAS_PER_CARD * nCards;
            else if (fFromSuit)
                lScore -= SCORE_VEGAS_PER_CARD * nCards;
            break;

        default:
            break;
    }

    UpdateStatusBar();
}

/*
 * Bank a legal move: one step of undo, then the score.  Called from a drop
 * rule, i.e. BEFORE the library commits the cards, so lScore and
 * VisiblePileCards still hold the values an undo has to restore.
 */
static void CommitMove(CARDREGION *pSrc, CARDREGION *pDest, INT nCards)
{
    INT nSrcId, nDestId;

    /* Dropping a run back where it came from is a legal no-op, and neither
       worth scoring nor recordable (SetUndo rejects it anyway). */
    if (pSrc == NULL || pSrc == pDest)
        return;

    nSrcId  = CardRegion_Id(pSrc);
    nDestId = CardRegion_Id(pDest);

    SetUndo(nSrcId, nDestId, nCards, lScore, VisiblePileCards);
    ScoreMove(nSrcId, nDestId, nCards);
    StartPlay();
}

/* ---------------------------------------------------------------------- *
 *  Dealing
 * ---------------------------------------------------------------------- */

void NewGame(void)
{
    CARDSTACK deck, tmp;
    INT i, j;

    StopPlayTimer();
    ClearUndo();

    /* Vegas buys the deal; "keep score" is what makes the bank run on from
       the last one instead of starting fresh. */
    if (GetScoreMode() == SCORE_VEGAS)
    {
        if (!fKeepScore)
            lScore = 0;
        lScore -= SCORE_VEGAS_STAKE;
    }
    else
    {
        lScore = 0;
    }

    CardWindow_EmptyStacks(SolWnd);
    CardStack_Clear(&activepile);
    VisiblePileCards = 0;

    CardStack_NewDeck(&deck);
    CardStack_Shuffle(&deck);

    /* Column i gets i face-down cards and one face-up on top, so column 0
       holds one card and column 6 holds seven.  CS_FACE_DOWNUP counts from
       the BOTTOM, so the option is exactly i. */
    for (i = 0; i < NUM_ROW_STACKS; i++)
    {
        CardStack_Clear(&tmp);
        for (j = 0; j <= i; j++)
            CardStack_PushCard(&tmp, CardStack_PopCard(&deck));

        CardRegion_SetFaceDirection(pRowStack[i], CS_FACE_DOWNUP, i);
        CardRegion_SetCardStack(pRowStack[i], &tmp);
    }

    for (i = 0; i < NUM_SUIT_STACKS; i++)
        CardRegion_Clear(pSuitStack[i]);

    /* The 24 cards nobody was dealt are the stock. */
    CardRegion_SetCardStack(pDeck, &deck);

    UpdatePileDisplay(0);

    fGameStarted = FALSE;
    dwElapsed = 0;

    UpdateStatusBar();
    CardWindow_Redraw(SolWnd);
}

/* ---------------------------------------------------------------------- *
 *  The stock
 * ---------------------------------------------------------------------- */

void CARDLIBPROC DeckClickProc(CARDREGION *pRegion, INT nNumCards)
{
    CARDSTACK tmp;
    INT nDeckCards = CardRegion_NumCards(pDeck);

    UNREFERENCED_PARAMETER(pRegion);
    UNREFERENCED_PARAMETER(nNumCards);

    if (nDeckCards > 0)
    {
        INT nDraw = nDrawCards;

        if (nDraw > nDeckCards)
            nDraw = nDeckCards;

        StartPlay();
        SetUndo(DECK_ID, PILE_ID, nDraw, lScore, VisiblePileCards);

        CardRegion_PopN(pDeck, nDraw, &tmp);

        /* PopN keeps the stock's order, so tmp's top is the card that came off
           FIRST.  Reversing makes the last one dealt the top of the waste -
           the one the player may take - and it is the order solundo.cpp
           reverses again to put them back. */
        CardStack_Reverse(&tmp);
        CardStack_PushStack(&activepile, &tmp);

        UpdatePileDisplay(nDraw);
    }
    else
    {
        /* Stock exhausted: the whole waste goes back under it, in the reverse
           of the order it came out, so the cycle repeats. */
        INT nTotal = CardStack_NumCards(&activepile);

        if (nTotal == 0)
            return;

        StartPlay();
        SetUndo(PILE_ID, DECK_ID, nTotal, lScore, VisiblePileCards);

        tmp = activepile;
        CardStack_Reverse(&tmp);
        CardRegion_SetCardStack(pDeck, &tmp);
        CardStack_Clear(&activepile);

        UpdatePileDisplay(0);
    }

    CardRegion_Update(pDeck);
    CardWindow_Redraw(SolWnd);
}

/* ---------------------------------------------------------------------- *
 *  What may be picked up
 * ---------------------------------------------------------------------- */

BOOL CARDLIBPROC PileDragProc(CARDREGION *pRegion, INT nNumDragging)
{
    /* Only the card on top of the waste is in play. */
    if (nNumDragging != 1)
        return FALSE;

    pDragSource = pRegion;
    return TRUE;
}

BOOL CARDLIBPROC SuitStackDragProc(CARDREGION *pRegion, INT nNumDragging)
{
    /* A card may come back off a foundation, but only the top one. */
    if (nNumDragging != 1)
        return FALSE;

    pDragSource = pRegion;
    return TRUE;
}

BOOL CARDLIBPROC RowStackDragProc(CARDREGION *pRegion, INT nNumDragging)
{
    const CARDSTACK *pStack;
    INT nFaceDown, i;

    if (nNumDragging < 1)
        return FALSE;

    CardRegion_GetFaceDirection(pRegion, &nFaceDown);

    /* Never lift a card that is still face down. */
    if (nNumDragging > CardRegion_NumCards(pRegion) - nFaceDown)
        return FALSE;

    /* A run only travels as a unit if it already IS one: descending in rank
       and alternating in colour.  Index 0 is the top of the pile, so card i
       rests on card i + 1 and must be one lower and the other colour. */
    pStack = CardRegion_GetCardStack(pRegion);

    for (i = 0; i < nNumDragging - 1; i++)
    {
        CARD upper = CardStack_Get(pStack, i);
        CARD lower = CardStack_Get(pStack, i + 1);

        if (Card_LoVal(upper) + 1 != Card_LoVal(lower))
            return FALSE;
        if (Card_IsRed(upper) == Card_IsRed(lower))
            return FALSE;
    }

    pDragSource = pRegion;
    return TRUE;
}

/* ---------------------------------------------------------------------- *
 *  Where they may land
 * ---------------------------------------------------------------------- */

BOOL CARDLIBPROC SuitStackDropProc(CARDREGION *pRegion, CARDSTACK *pDragCards)
{
    if (CardStack_NumCards(pDragCards) != 1)
        return FALSE;

    if (!CanDropOnSuitStack(pRegion, CardStack_Get(pDragCards, 0)))
        return FALSE;

    CommitMove(pDragSource, pRegion, 1);
    return TRUE;
}

BOOL CARDLIBPROC RowStackDropProc(CARDREGION *pRegion, CARDSTACK *pDragCards)
{
    CARD cardLanding;
    INT nDrag = CardStack_NumCards(pDragCards);

    if (nDrag < 1)
        return FALSE;

    /* Index 0 is the top of a pile, so the card that actually touches the
       column is the one at the FAR end of the run being carried. */
    cardLanding = CardStack_Get(pDragCards, nDrag - 1);

    if (CardRegion_NumCards(pRegion) == 0)
    {
        /* An empty column is a King's seat and nothing else's. */
        if (Card_LoVal(cardLanding) != King)
            return FALSE;
    }
    else
    {
        CARD cardTop = CardStack_TopCard(CardRegion_GetCardStack(pRegion));

        if (Card_LoVal(cardLanding) + 1 != Card_LoVal(cardTop))
            return FALSE;
        if (Card_IsRed(cardLanding) == Card_IsRed(cardTop))
            return FALSE;
    }

    CommitMove(pDragSource, pRegion, nDrag);
    return TRUE;
}

/* ---------------------------------------------------------------------- *
 *  Clicks
 * ---------------------------------------------------------------------- */

void CARDLIBPROC RowStackClickProc(CARDREGION *pRegion, INT nNumCards)
{
    UNREFERENCED_PARAMETER(nNumCards);
    TurnRowStackCard(pRegion);
}

/*
 * Send the top card of a pile to whichever foundation will take it.
 *
 * SimulateDrag rather than MoveCard, deliberately: only the drag path consults
 * the foundation's drop rule - which is where the move is scored and recorded
 * for undo - and fires its add callback, which is where the win is noticed.
 */
static BOOL PlayToSuitStack(CARDREGION *pSrc)
{
    CARD card;
    INT i;

    if (CardRegion_NumCards(pSrc) == 0)
        return FALSE;

    card = CardStack_TopCard(CardRegion_GetCardStack(pSrc));

    for (i = 0; i < NUM_SUIT_STACKS; i++)
    {
        if (!CanDropOnSuitStack(pSuitStack[i], card))
            continue;

        pDragSource = pSrc;
        CardRegion_SimulateDrag(pSrc, pSuitStack[i], 1, TRUE);
        CardWindow_Redraw(SolWnd);
        return TRUE;
    }

    return FALSE;
}

void CARDLIBPROC PileDblClickProc(CARDREGION *pRegion, INT nNumCards)
{
    UNREFERENCED_PARAMETER(nNumCards);
    PlayToSuitStack(pRegion);
}

void CARDLIBPROC RowStackDblClickProc(CARDREGION *pRegion, INT nNumCards)
{
    INT nFaceDown;

    /* Only the exposed card of a column plays itself; a double-click further
       down the fan is picking out a buried card and means nothing here. */
    if (nNumCards != 1)
        return;

    CardRegion_GetFaceDirection(pRegion, &nFaceDown);
    if (CardRegion_NumCards(pRegion) <= nFaceDown)
        return;

    PlayToSuitStack(pRegion);
}

/* ---------------------------------------------------------------------- *
 *  Cards arriving and leaving
 * ---------------------------------------------------------------------- */

void CARDLIBPROC SuitStackAddProc(CARDREGION *pRegion, const CARDSTACK *pAdded)
{
    INT i, nTotal = 0;

    UNREFERENCED_PARAMETER(pRegion);
    UNREFERENCED_PARAMETER(pAdded);

    for (i = 0; i < NUM_SUIT_STACKS; i++)
        nTotal += CardRegion_NumCards(pSuitStack[i]);

    /* Posted, not called: see WM_SOL_GAMEWON in solitaire.h.  We are inside
       the library's drop handling and it is not finished with us yet. */
    if (nTotal == NUM_CARDS)
        PostMessage(hwndMain, WM_SOL_GAMEWON, 0, 0);
}

void CARDLIBPROC PileRemoveProc(CARDREGION *pRegion, INT nRemoved)
{
    CARDSTACK tmp;

    UNREFERENCED_PARAMETER(pRegion);

    /* The region is only a view; take the card out of the waste itself, then
       rebuild the view.  The fan shrinks by one as it is played off, and
       collapses to a square pile once a single card is left showing. */
    CardStack_PopN(&activepile, nRemoved, &tmp);

    if (VisiblePileCards > 1)
        VisiblePileCards -= nRemoved;

    UpdatePileDisplay(VisiblePileCards);
}

void CARDLIBPROC RowStackRemoveProc(CARDREGION *pRegion, INT nRemoved)
{
    UNREFERENCED_PARAMETER(nRemoved);
    TurnRowStackCard(pRegion);
}

/* ---------------------------------------------------------------------- *
 *  Winning
 * ---------------------------------------------------------------------- */

void GameFinished(void)
{
    INT i;

    StopPlayTimer();

    /* The time bonus only applies to a timed standard game, and only if the
       deal took long enough for the division to mean anything. */
    if (GetScoreMode() == SCORE_STD && fShowTime && dwElapsed > SCORE_STD_BONUS_MINTIME)
        lScore += (INT)(SCORE_STD_BONUS_BASE / dwElapsed);

    UpdateStatusBar();
    ClearUndo();

    for (i = 0; i < NUM_SUIT_STACKS; i++)
        CardRegion_Flash(pSuitStack[i], 6, 100);

    MessageBox(hwndMain, MsgWin, szAppName, MB_OK | MB_ICONINFORMATION);

    if (MessageBox(hwndMain, MsgDeal, szAppName, MB_YESNO | MB_ICONQUESTION) == IDYES)
        NewGame();
    else
        fGameStarted = FALSE;
}
