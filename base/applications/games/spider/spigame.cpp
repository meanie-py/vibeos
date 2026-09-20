/*
 * PROJECT:      Spider Solitaire
 * LICENSE:      See COPYING in top level directory
 * FILE:         base/applications/games/spider/spigame.cpp
 * PURPOSE:      Spider Solitaire game functions
 * PROGRAMMER:   Gregor Schneider
 */

#include "spider.h"

#define NUM_DECK_CARDS     5
#define NUM_SMALLER_STACKS 4
#define NUM_CARD_COLORS    4
#define NUM_ONECOLOR_CARDS 13
#define NUM_STD_CARDS      52
#define NUM_SPIDER_CARDS   104

CARDSTACK deck;
CARDREGION *from;
CARDREGION *pDeck;
CARDREGION *pStack[NUM_STACKS];
bool fGameStarted = false;
int  yRowStackCardOffset;
int cardsFinished;
extern TCHAR MsgDeal[];
extern TCHAR MsgWin[];

CARDSTACK CreatePlayDeck()
{
    CARDSTACK newStack;
    int i, colors = 1, num = 0;

    CardStack_Clear(&newStack);

    switch (dwDifficulty)
    {
        case IDC_DIF_ONECOLOR:
            colors = 1;
            break;
        case IDC_DIF_TWOCOLORS:
            colors = 2;
            break;
        case IDC_DIF_FOURCOLORS:
            colors = 4;
            break;
    }
    for (i = 0; i < NUM_SPIDER_CARDS; i++)
    {
        num += NUM_CARD_COLORS / colors;
        CARD newCard = Card_FromIndex(num % NUM_STD_CARDS);
        CardStack_PushCard(&newStack, newCard);
    }
    return newStack;
}

void NewGame(void)
{
    int i, j;
    /* First four stack with five, all other with 4 */
    int covCards = 5;
    CARDSTACK fakeDeck, temp;

    CardWindow_EmptyStacks(SpiderWnd);

    /* Create a new card-deck, fake deck */
    deck = CreatePlayDeck();
    CardStack_Shuffle(&deck);
    CardStack_NewDeck(&fakeDeck);
    CardStack_Shuffle(&fakeDeck);

    /* Reset progress value */
    cardsFinished = 0;

    /* Deal to each stack */
    for (i = 0; i < NUM_STACKS; i++)
    {
        CardStack_Clear(&temp);
        if (i == NUM_SMALLER_STACKS)
        {
            covCards--;
        }
        for (j = 0; j <= covCards; j++)
        {
            CardStack_PushCard(&temp, CardStack_PopCard(&deck));
        }
        CardRegion_SetFaceDirection(pStack[i], CS_FACE_DOWNUP, covCards);
        CardRegion_SetCardStack(pStack[i], &temp);
    }
    /* Deal five fake cards to the deck */
    CardStack_PopN(&fakeDeck, 5, &temp);
    CardRegion_SetCardStack(pDeck, &temp);

    CardWindow_Redraw(SpiderWnd);
    fGameStarted = false;
}

bool stackLookingGood(const CARDSTACK *mystack, int numChecks)
{
    int i;
    for (i = 0; i < numChecks; i++)
    {
        if (Card_LoVal(CardStack_Get(mystack, i)) != Card_LoVal(CardStack_Get(mystack, i + 1)) - 1)
        {
            return false;
        }
        if (Card_Suit(CardStack_Get(mystack, i)) != Card_Suit(CardStack_Get(mystack, i + 1)))
        {
            return false;
        }
    }
    return true;
}

/* CARD to be turned from a stack */
void TurnStackCard(CARDREGION *stackobj)
{
    int numfacedown;

    CardRegion_GetFaceDirection(stackobj, &numfacedown);
    if (CardRegion_NumCards(stackobj) <= numfacedown)
    {
        if (numfacedown > 0) numfacedown--;
        CardRegion_SetFaceDirection(stackobj, CS_FACE_DOWNUP, numfacedown);
        CardRegion_Redraw(stackobj);
    }
}

/* Click on the deck */
void CARDLIBPROC DeckClickProc(CARDREGION *stackobj, int NumDragCards)
{
    CARDSTACK temp, fakeDeck = *CardRegion_GetCardStack(pDeck);
    fGameStarted = true;

    if (CardStack_NumCards(&fakeDeck) != 0 && CardStack_NumCards(&deck) != 0)
    {
        int i, facedown, faceup;
        /* Add one card to every stack */
        for (i = 0; i < NUM_STACKS; i++)
        {
            temp = *CardRegion_GetCardStack(pStack[i]);
            CardStack_PushCard(&temp, CardStack_PopCard(&deck));

            /* Check if we accidentally finished a row */
            CardRegion_GetFaceDirection(pStack[i], &facedown);
            faceup = CardStack_NumCards(&temp) - facedown;
            if (faceup >= NUM_ONECOLOR_CARDS)
            {
                /* Check stack finished, remove cards if so */
                if (stackLookingGood(&temp, NUM_ONECOLOR_CARDS - 1))
                {
                    int j;
                    for (j = 0; j < NUM_ONECOLOR_CARDS; j++)
                    {
                        CardStack_RemoveCard(&temp, 0);
                    }
                    cardsFinished += NUM_ONECOLOR_CARDS;
                    CardRegion_SetCardStack(pStack[i], &temp);
                    /* Turn now topmost card */
                    TurnStackCard(pStack[i]);
                }
            }
            CardRegion_SetCardStack(pStack[i], &temp);
        }
        /* Remove one card from the fake ones */
        CardStack_PopN(&fakeDeck, CardStack_NumCards(&fakeDeck) - 1, &temp);
        CardRegion_SetCardStack(pDeck, &temp);
    }
    CardRegion_Update(pDeck);
    CardWindow_Redraw(SpiderWnd);
}

/* Cards dragged from a stack */
BOOL CARDLIBPROC StackDragProc(CARDREGION *stackobj, int numDragCards)
{
    int numfacedown, numcards;

    CardRegion_GetFaceDirection(stackobj, &numfacedown);
    numcards = CardRegion_NumCards(stackobj);

    /* Only cards facing up */
    if (numDragCards <= numcards - numfacedown)
    {
        const CARDSTACK *mystack = CardRegion_GetCardStack(stackobj);
        /* Don't allow to drag unsuited cards */
        if (!stackLookingGood(mystack, numDragCards - 1))
        {
            return false;
        }
        /* Remember where the cards come from */
        from = stackobj;
        return true;
    }
    else
    {
        return false;
    }
}

/* Game finished successfully */
void GameFinished()
{
    CardWindow_EmptyStacks(SpiderWnd);

    MessageBox(CardWindow_GetHwnd(SpiderWnd), MsgWin, szAppName, MB_OK | MB_ICONINFORMATION);
    if( IDYES == MessageBox(CardWindow_GetHwnd(SpiderWnd), MsgDeal, szAppName, MB_YESNO | MB_ICONQUESTION) )
    {
        NewGame();
    }
    else
    {
        fGameStarted = false;
    }
}

/* CARD added, check for win situation */
void CARDLIBPROC StackAddProc(CARDREGION *stackobj, const CARDSTACK *added)
{
    if (cardsFinished == NUM_SPIDER_CARDS)
    {
        GameFinished();
    }
}

/* Cards dropped to a stack */
BOOL CARDLIBPROC StackDropProc(CARDREGION *stackobj, CARDSTACK *dragcards)
{
    CARD dragcard = CardStack_Get(dragcards, CardStack_NumCards(dragcards) - 1);
    int faceup, facedown;

    /* Only drop our cards on other stacks */
    if (CardRegion_Id(stackobj) == CardRegion_Id(from))
    {
        return false;
    }

    /* If stack is empty, everything can be dropped */
    if (CardRegion_NumCards(stackobj) != 0)
    {
        const CARDSTACK *mystack = CardRegion_GetCardStack(stackobj);

        /* Can only drop if card is 1 less */
        if (Card_LoVal(CardStack_Get(mystack, 0)) != Card_LoVal(dragcard) + 1)
        {
            return false;
        }

        /* Check if stack complete */
        CardRegion_GetFaceDirection(stackobj, &facedown);
        faceup = CardRegion_NumCards(stackobj) - facedown;

        if (faceup + CardStack_NumCards(dragcards) >= NUM_ONECOLOR_CARDS)
        {
            int i, max = NUM_ONECOLOR_CARDS - CardStack_NumCards(dragcards) - 1;

            /* Dragged cards have been checked to be in order, check stack cards */
            if (Card_Suit(CardStack_Get(mystack, 0)) == Card_Suit(dragcard) &&
                stackLookingGood(mystack, max))
            {
                CARDSTACK s = *CardRegion_GetCardStack(stackobj);

                /* Remove from card stack */
                for (i = 0; i < max + 1; i++)
                {
                    CardStack_RemoveCard(&s, 0);
                }
                /* Remove dragged cards: the drop is accepted but lands
                   nothing, because the run it completed has just been taken
                   off the board above. */
                CardStack_Clear(dragcards);
                CardRegion_SetCardStack(stackobj, &s);
                cardsFinished += NUM_ONECOLOR_CARDS;
                /* Flip top card of the dest stack */
                TurnStackCard(stackobj);
            }
        }
    }
    /* Flip the top card of the source stack */
    TurnStackCard(from);
    fGameStarted = true;
    return true;
}

/* Create card regions */
void CreateSpider()
{
    int i, pos;

    /* Compute the value for yRowStackCardOffset based on the height of the card, so the card number and suite isn't hidden on larger cards except Ace */
    yRowStackCardOffset = (int)(CardLib_CardHeight() / 6.4);

    pDeck = CardWindow_CreateRegion(SpiderWnd, 0, true, 0, 0, -15, 0);
    CardRegion_SetFaceDirection(pDeck, CS_FACE_DOWN, 0);
    CardRegion_SetEmptyImage(pDeck, CS_EI_CIRC);
    CardRegion_SetPlacement(pDeck, CS_XJUST_RIGHT, CS_YJUST_BOTTOM, - X_BORDER, - Y_BORDER);
    CardRegion_SetDragRule(pDeck, CS_DRAG_NONE, 0);
    CardRegion_SetDropRule(pDeck, CS_DROP_NONE, 0);
    CardRegion_SetClickReleaseProc(pDeck, DeckClickProc);

    /* Create the row stacks */
    for (i = 0; i < NUM_STACKS; i++)
    {
        pStack[i] = CardWindow_CreateRegion(SpiderWnd, NUM_STACKS+i, true, 0, Y_BORDER, 0, yRowStackCardOffset);
        CardRegion_SetFaceDirection(pStack[i], CS_FACE_DOWN, 0);
        pos = i - NUM_STACKS/2;
        CardRegion_SetPlacement(pStack[i], CS_XJUST_CENTER, 0,
            pos * (CardLib_CardWidth() + X_BORDER) + 6 * (X_BORDER + 1) + 3, 0);
        CardRegion_SetEmptyImage(pStack[i], CS_EI_SUNK);
        CardRegion_SetDragRule(pStack[i], CS_DRAG_CALLBACK, StackDragProc);
        CardRegion_SetDropRule(pStack[i], CS_DROP_CALLBACK, StackDropProc);
        CardRegion_SetAddCardProc(pStack[i], StackAddProc);
    }
}

