#include "solitaire.h"

CARDREGION *pDeck;
CARDREGION *pPile;
CARDREGION *pSuitStack[4];
CARDREGION *pRowStack[NUM_ROW_STACKS];

HBITMAP hbmBitmap;
HDC     hdcBitmap;
int     yRowStackCardOffset;

void CreateSol()
{
    int i;

//    hbmBitmap = (HBITMAP)LoadImage(0,"test.bmp",IMAGE_BITMAP,0,0,LR_LOADFROMFILE);
//    SolWnd.SetBackImage(hbmBitmap);

    CardStack_Clear(&activepile);

    // Compute the value for yRowStackCardOffset based on the height of the card, so the card number isn't hidden on larger cards
    yRowStackCardOffset = (int)(CardLib_CardHeight() / 6.7);

    pDeck = CardWindow_CreateRegion(SolWnd, DECK_ID, true, X_BORDER, Y_BORDERWITHFRAME, 2, 1);
    CardRegion_SetEmptyImage(pDeck, CS_EI_CIRC);
    CardRegion_SetThreedCount(pDeck, 6);
    CardRegion_SetDragRule(pDeck, CS_DRAG_NONE, 0);
    CardRegion_SetDropRule(pDeck, CS_DROP_NONE, 0);
    CardRegion_SetClickProc(pDeck, DeckClickProc);
    CardRegion_SetDblClickProc(pDeck, DeckClickProc);
    CardRegion_SetFaceDirection(pDeck, CS_FACE_DOWN, 0);

    pPile = CardWindow_CreateRegion(SolWnd, PILE_ID, true, X_BORDER + CardLib_CardWidth() + X_PILE_BORDER, Y_BORDERWITHFRAME, CS_DEFXOFF, 1);
    CardRegion_SetEmptyImage(pPile, CS_EI_NONE);
    CardRegion_SetDragRule(pPile, CS_DRAG_TOP, 0);
    CardRegion_SetDropRule(pPile, CS_DROP_NONE, 0);
    CardRegion_SetDblClickProc(pPile, PileDblClickProc);
    CardRegion_SetRemoveCardProc(pPile, PileRemoveProc);
    CardRegion_SetClickProc(pPile, PileClickProc);

    //
    //    Create the suit stacks
    //
    for(i = 0; i < 4; i++)
    {
        pSuitStack[i] = CardWindow_CreateRegion(SolWnd, SUIT_ID+i, true, 0, Y_BORDERWITHFRAME, 0, 0);
        CardRegion_SetEmptyImage(pSuitStack[i], CS_EI_SUNK);
        CardRegion_SetPlacement(pSuitStack[i], CS_XJUST_CENTER, 0, i * (CardLib_CardWidth() + X_SUITSTACK_BORDER) , 0);

        CardRegion_SetDropRule(pSuitStack[i], CS_DROP_CALLBACK, SuitStackDropProc);
        CardRegion_SetDragRule(pSuitStack[i], CS_DRAG_TOP, 0);
        CardRegion_SetClickProc(pSuitStack[i], SuitStackClickProc);

        CardRegion_SetAddCardProc(pSuitStack[i], SuitStackAddProc);
    }

    //
    //    Create the row stacks
    //
    for(i = 0; i < NUM_ROW_STACKS; i++)
    {
        pRowStack[i] = CardWindow_CreateRegion(SolWnd, ROW_ID+i, true, 0, Y_BORDERWITHFRAME + CardLib_CardHeight() + Y_ROWSTACK_BORDER, 0, yRowStackCardOffset);
        CardRegion_SetEmptyImage(pRowStack[i], CS_EI_SUNK);
        CardRegion_SetFaceDirection(pRowStack[i], CS_FACE_DOWNUP, i);

        CardRegion_SetPlacement(pRowStack[i], CS_XJUST_CENTER, 0,
            (i - NUM_ROW_STACKS/2) * (CardLib_CardWidth() + X_ROWSTACK_BORDER),     0);

        CardRegion_SetEmptyImage(pRowStack[i], CS_EI_NONE);

        CardRegion_SetDragRule(pRowStack[i], CS_DRAG_CALLBACK, RowStackDragProc);
        CardRegion_SetDropRule(pRowStack[i], CS_DROP_CALLBACK, RowStackDropProc);
        CardRegion_SetClickProc(pRowStack[i], RowStackClickProc);
        CardRegion_SetDblClickProc(pRowStack[i], RowStackDblClickProc);
    }
}
