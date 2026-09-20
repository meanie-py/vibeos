#include "solitaire.h"

#if 1
#define TRACE(s)
#else
#define TRACE(s) printf("%s(%i): %s",__FILE__,__LINE__,s)
#endif

extern TCHAR MsgWin[128];
extern TCHAR MsgDeal[128];

CARDSTACK activepile;
int VisiblePileCards;
int LastId;
bool fGameStarted = false;
bool bAutoroute = false;

void NewGame(void)
{
    TRACE("ENTER NewGame()\n");
    int i, j;

    if (GetScoreMode() == SCORE_VEGAS)
    {
        if ((dwOptions & OPTION_KEEP_SCORE) && (dwPrevMode == SCORE_VEGAS))
            lScore = lScore - 52;
        else
            lScore = -52;

        if (dwOptions & OPTION_THREE_CARDS)
            dwWasteTreshold = 2;
        else
            dwWasteTreshold = 0;

    }
    else
    {
        if (dwOptions & OPTION_THREE_CARDS)
            dwWasteTreshold = 3;
        else
            dwWasteTreshold = 0;

        lScore = 0;
    }

    dwTime = 0;
    dwWasteCount = 0;
    LastId = 0;

    CardWindow_EmptyStacks(SolWnd);

    //create a new card-stack
    CARDSTACK deck;
    CardStack_NewDeck(&deck);
    CardStack_Shuffle(&deck);
    CardStack_Clear(&activepile);
    VisiblePileCards = 0;

    //deal to each row stack..
    for(i = 0; i < NUM_ROW_STACKS; i++)
    {
        CARDSTACK temp;
        CardStack_Clear(&temp);

        CardRegion_SetFaceDirection(pRowStack[i], CS_FACE_DOWNUP, i);

        for(j = 0; j <= i; j++)
        {
            CardStack_PushCard(&temp, CardStack_PopCard(&deck));
        }

        CardRegion_SetCardStack(pRowStack[i], &temp);
    }

    //put the other cards onto the deck
    CardRegion_SetCardStack(pDeck, &deck);
    CardRegion_Update(pDeck);

    // For the 1-card-mode, all cards need to be completely overlapped
    if(!(dwOptions & OPTION_THREE_CARDS))
        CardRegion_SetOffsets(pPile, 0, 0);

    CardWindow_Redraw(SolWnd);

    fGameStarted = false;

    dwPrevMode = GetScoreMode();

    UpdateStatusBar();
    ClearUndo();

    TRACE("EXIT NewGame()\n");

}

//
//    Now follow the stack callback functions. This is where we
//  provide the game functionality and rules
//

//
//    Can only drag face-up cards
//
BOOL CARDLIBPROC RowStackDragProc(CARDREGION *stackobj, int iNumDragCards)
{
    TRACE("ENTER RowStackDragProc()\n");
    int numfacedown;
    int numcards;

    SetPlayTimer();

    CardRegion_GetFaceDirection(stackobj, &numfacedown);

    numcards = CardRegion_NumCards(stackobj);

    TRACE("EXIT RowStackDragProc()\n");
    if(iNumDragCards <= numcards-numfacedown)
        return true;
    else
        return false;

}

//
//    Row a row-stack, we can only drop cards
//    that are lower / different colour
//
BOOL CARDLIBPROC RowStackDropProc(CARDREGION *stackobj, CARDSTACK *dragcards)
{
    TRACE("ENTER RowStackDropProc()\n");
    CARD dragcard = CardStack_Get(dragcards, CardStack_NumCards(dragcards) - 1);

    SetPlayTimer();

    //if we are empty, can only drop a stack with a King at bottom
    if(CardRegion_NumCards(stackobj) == 0)
    {
        if(Card_LoVal(dragcard) != 13)
        {
            TRACE("EXIT RowStackDropProc(false)\n");
            return false;
        }
    }
    else
    {
        const CARDSTACK *mystack = CardRegion_GetCardStack(stackobj);

        //can only drop if card is 1 less
        if(Card_LoVal(CardStack_Get(mystack, 0)) != Card_LoVal(dragcard) + 1)
        {
            TRACE("EXIT RowStackDropProc(false)\n");
            return false;
        }

        //can only drop if card is different colour
        if( (Card_IsBlack(CardStack_Get(mystack, 0)) && !Card_IsRed(dragcard)) ||
           (!Card_IsBlack(CardStack_Get(mystack, 0)) &&  Card_IsRed(dragcard)) )
        {
            TRACE("EXIT RowStackDropProc(false)\n");
            return false;
        }
    }

    fGameStarted = true;

    SetUndo(LastId, CardRegion_Id(stackobj), CardStack_NumCards(dragcards), lScore, VisiblePileCards);

    if (LastId == PILE_ID)
    {
        if (GetScoreMode() == SCORE_STD)
        {
            lScore = lScore + 5;
        }
    }
    else if ((LastId >= SUIT_ID) && (LastId <= SUIT_ID + 3))
    {
        if (GetScoreMode() == SCORE_STD)
        {
            lScore = lScore >= 15 ? lScore - 15 : 0;
        }
        else if (GetScoreMode() == SCORE_VEGAS)
        {
            lScore = lScore >= -47 ? lScore - 5 : -52;
        }
    }

    UpdateStatusBar();

    TRACE("EXIT RowStackDropProc(true)\n");
    return true;
}

//
//    Can only drop a card onto a suit-stack if the
//  card is 1 higher, and is the same suit
//
bool CanDrop(CARDREGION *stackobj, CARD card)
{
    TRACE("ENTER CanDrop()\n");
    int topval;

    const CARDSTACK *cardstack = CardRegion_GetCardStack(stackobj);

    SetPlayTimer();

    if(CardStack_NumCards(cardstack) > 0)
    {
        if(Card_Suit(card) != Card_Suit(CardStack_Get(cardstack, 0)))
        {
            TRACE("EXIT CanDrop()\n");
            return false;
        }

        topval = Card_LoVal(CardStack_Get(cardstack, 0));
    }
    else
    {
        topval = 0;
    }

    //make sure 1 higher
    if(Card_LoVal(card) != (topval + 1))
    {
        TRACE("EXIT CanDrop()\n");
        return false;
    }

    TRACE("EXIT CanDrop()\n");
    return true;
}

//
//    Can only drop a card onto suit stack if it is same suit, and 1 higher
//
BOOL CARDLIBPROC SuitStackDropProc(CARDREGION *stackobj, CARDSTACK *dragcards)
{
    TRACE("ENTER SuitStackDropProc()\n");

    SetPlayTimer();

    //only drop 1 card at a time
    if (!bAutoroute && CardStack_NumCards(dragcards) != 1)
    {
        TRACE("EXIT SuitStackDropProc()\n");
        return false;
    }

    bool b = CanDrop(stackobj, CardStack_Get(dragcards, 0));
    TRACE("EXIT SuitStackDropProc()\n");

    if (b)
    {
        SetUndo(LastId, CardRegion_Id(stackobj), 1, lScore, VisiblePileCards);

        if ((LastId == PILE_ID) || (LastId >= ROW_ID))
        {
            if (GetScoreMode() == SCORE_VEGAS)
            {
                lScore = lScore + 5;
            }
            else if (GetScoreMode() == SCORE_STD)
            {
                lScore = lScore + 10;
            }

            UpdateStatusBar();
        }
    }

    return b;
}

//
//    Single-click on one of the suit-stacks
//
void CARDLIBPROC SuitStackClickProc(CARDREGION *stackobj, int iNumClicked)
{
    TRACE("ENTER SuitStackClickProc()\n");

    fGameStarted = true;

    LastId = CardRegion_Id(stackobj);

    TRACE("EXIT SuitStackClickProc()\n");
}

//
//    Single-click on one of the row-stacks
//    Turn the top-card over if they are all face-down
//
void CARDLIBPROC RowStackClickProc(CARDREGION *stackobj, int iNumClicked)
{
    TRACE("ENTER RowStackClickProc()\n");
    int numfacedown;

    CardRegion_GetFaceDirection(stackobj, &numfacedown);

    //if all face-down, then make top card face-up
    if(CardRegion_NumCards(stackobj) == numfacedown)
    {
        if(numfacedown > 0) numfacedown--;
        CardRegion_SetFaceDirection(stackobj, CS_FACE_DOWNUP, numfacedown);
        CardRegion_Redraw(stackobj);

        if (GetScoreMode() == SCORE_STD)
        {
            lScore = lScore + 5;
            UpdateStatusBar();
        }
        ClearUndo();
    }

    LastId = CardRegion_Id(stackobj);

    fGameStarted = true;

    TRACE("EXIT RowStackClickProc()\n");
}

//
//    Find the suit-stack that can accept the specified card
//
CARDREGION *FindSuitStackFromCard(CARD card)
{
    TRACE("ENTER FindSuitStackFromCard()\n");

    for(int i = 0; i < 4; i++)
    {
        if(CanDrop(pSuitStack[i], card))
        {
            TRACE("EXIT FindSuitStackFromCard()\n");
            return pSuitStack[i];
        }
    }

    TRACE("EXIT FindSuitStackFromCard()\n");
    return 0;
}

//
//    What happens when we add a card to one of the suit stacks?
//  Well, nothing (it is already added), but we need to
//  check all four stacks (not just this one) to see if
//  the game has finished.
//
void CARDLIBPROC SuitStackAddProc(CARDREGION *stackobj, const CARDSTACK *added)
{
    TRACE("ENTER SuitStackAddProc()\n");
    bool fGameOver = true;

    SetPlayTimer();

    for(int i = 0; i < 4; i++)
    {
        if(CardRegion_NumCards(pSuitStack[i]) != 13)
        {
            fGameOver = false;

            break;
        }
    }

    if(fGameOver)
    {
        KillTimer(hwndMain, IDT_PLAYTIMER);
        PlayTimer = 0;

        if ((dwOptions & OPTION_SHOW_TIME) && (GetScoreMode() == SCORE_STD))
        {
            lScore = lScore + (700000 / dwTime);
        }

        UpdateStatusBar();
        ClearUndo();

        MessageBox(CardWindow_GetHwnd(SolWnd), MsgWin, szAppName, MB_OK | MB_ICONINFORMATION);

        for(int i = 0; i < 4; i++)
        {
            CardRegion_Flash(pSuitStack[i], 11, 100);
        }

        if( IDYES == MessageBox(CardWindow_GetHwnd(SolWnd), MsgDeal, szAppName, MB_YESNO | MB_ICONQUESTION) )
        {
            NewGame();
        }
        else
        {
            CardWindow_EmptyStacks(SolWnd);

            fGameStarted = false;
        }
    }

    TRACE("EXIT SuitStackAddProc()\n");
}

//
//    Double-click on one of the row stacks
//    The aim is to find a suit-stack to move the
//  double-clicked card to.
//
void CARDLIBPROC RowStackDblClickProc(CARDREGION *stackobj, int iNumClicked)
{
    TRACE("ENTER RowStackDblClickProc()\n");

    SetPlayTimer();

    //can only move 1 card at a time
    if(iNumClicked != 1)
    {
        TRACE("EXIT RowStackDblClickProc()\n");
        return;
    }

    //find a suit-stack to move the card to...
    const CARDSTACK *cardstack = CardRegion_GetCardStack(stackobj);
    CARDREGION *pDest = FindSuitStackFromCard(CardStack_Get(cardstack, 0));

    if(pDest != 0)
    {
        fGameStarted = true;
        SetPlayTimer();

        //stackobj.MoveCards(pDest, 1, true);
        //use the SimulateDrag function, because we get the
        //AddProc callbacks called for us on the destination stacks...
        bAutoroute = true;
        CardRegion_SimulateDrag(stackobj, pDest, 1, true);
        bAutoroute = false;
    }
    TRACE("EXIT RowStackDblClickProc()\n");
}

//
//    Face-up pile single-click
//
void CARDLIBPROC PileClickProc(CARDREGION *stackobj, int iNumClicked)
{
    TRACE("ENTER SuitStackClickProc()\n");

    fGameStarted = true;

    LastId = CardRegion_Id(stackobj);

    TRACE("EXIT SuitStackClickProc()\n");
}

//
//    Face-up pile double-click
//
void CARDLIBPROC PileDblClickProc(CARDREGION *stackobj, int iNumClicked)
{
    TRACE("ENTER PileDblClickProc()\n");

    SetPlayTimer();

    RowStackDblClickProc(stackobj, iNumClicked);
    TRACE("EXIT PileDblClickProc()\n");
}

//
//    Fix for the 3-card play when only 1 card left on the pile.
//
void FixIfOneCardLeft(void)
{
    // If there is just 1 card left, then modify the
    // stack to contain ALL the face-up cards. The effect
    // will be, the next time a card is dragged, all the
    // previous card-triplets will be available underneath.
    if ((dwOptions & OPTION_THREE_CARDS) && CardRegion_NumCards(pPile) == 1)
    {
        CardRegion_SetOffsets(pPile, 0, 0);
        CardRegion_SetCardStack(pPile, &activepile);
    }
}

//
//    What happens when a card is removed from face-up pile?
//
void CARDLIBPROC PileRemoveProc(CARDREGION *stackobj, int iItems)
{
    CARDSTACK discarded;

    TRACE("ENTER PileRemoveProc()\n");

    SetPlayTimer();

    //modify our "virtual" pile by removing the same card
    //that was removed from the physical card stack
    CardStack_PopN(&activepile, iItems, &discarded);
    if ((dwOptions & OPTION_THREE_CARDS) && (VisiblePileCards > 1))
    {
        --VisiblePileCards;
    }

    FixIfOneCardLeft();

    TRACE("EXIT PileRemoveProc()\n");
}

//
//    Double-click on the deck
//    Move 3 cards to the face-up pile
//
void CARDLIBPROC DeckClickProc(CARDREGION *stackobj, int iNumClicked)
{
    TRACE("ENTER DeckClickProc()\n");

    SetPlayTimer();

    CARDSTACK cardstack = *CardRegion_GetCardStack(stackobj);
    CARDSTACK pile      = *CardRegion_GetCardStack(pPile);

    fGameStarted = true;
    SetPlayTimer();

    //reset the face-up pile to represent 3 cards
    if(dwOptions & OPTION_THREE_CARDS)
        CardRegion_SetOffsets(pPile, CS_DEFXOFF, 1);

    if(CardStack_NumCards(&cardstack) == 0)
    {
        if (GetScoreMode() == SCORE_VEGAS)
        {
            if (dwWasteCount < dwWasteTreshold)
            {
                CardStack_Clear(&pile);

                CardStack_Reverse(&activepile);
                CardStack_PushStack(&cardstack, &activepile);
                CardStack_Clear(&activepile);
                SetUndo(PILE_ID, DECK_ID, CardStack_NumCards(&cardstack), lScore, VisiblePileCards);
                VisiblePileCards = 0;
            }
        }
        else if (GetScoreMode() == SCORE_STD)
        {
            SetUndo(PILE_ID, DECK_ID, CardStack_NumCards(&activepile), lScore, VisiblePileCards);
            if ((dwWasteCount >= dwWasteTreshold) && (CardStack_NumCards(&activepile) != 0))
            {
                if (dwOptions & OPTION_THREE_CARDS)
                    lScore = lScore >= 20 ? lScore - 20 : 0;
                else
                    lScore = lScore >= 100 ? lScore - 100 : 0;
            }

            CardStack_Clear(&pile);

            CardStack_Reverse(&activepile);
            CardStack_PushStack(&cardstack, &activepile);
            CardStack_Clear(&activepile);
            VisiblePileCards = 0;

            UpdateStatusBar();
        }
        else
        {
            CardStack_Clear(&pile);

            CardStack_Reverse(&activepile);
            CardStack_PushStack(&cardstack, &activepile);
            CardStack_Clear(&activepile);
            SetUndo(PILE_ID, DECK_ID, CardStack_NumCards(&cardstack), lScore, VisiblePileCards);
            VisiblePileCards = 0;
        }

        dwWasteCount++;
    }
    else
    {
        int numcards = min((dwOptions & OPTION_THREE_CARDS) ? 3 : 1, CardStack_NumCards(&cardstack));

        SetUndo(DECK_ID, PILE_ID, numcards, lScore, VisiblePileCards);

        //make a "visible" copy of these cards
        CARDSTACK temp;
        CardStack_PopN(&cardstack, numcards, &temp);
        CardStack_Reverse(&temp);

        if(dwOptions & OPTION_THREE_CARDS)
            CardStack_Clear(&pile);

        CardStack_PushStack(&pile, &temp);

        //remove the top 3 from deck
        CardStack_PushStack(&activepile, &temp);

        VisiblePileCards = numcards;
    }

    CardRegion_SetCardStack(pDeck, &cardstack);
    CardRegion_SetCardStack(pPile, &pile);

    FixIfOneCardLeft();

    CardWindow_Redraw(SolWnd);
    TRACE("EXIT DeckClickProc()\n");
}
