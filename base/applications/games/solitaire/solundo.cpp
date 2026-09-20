/*
 * PROJECT:      Solitaire
 * LICENSE:      See COPYING in top level directory
 * FILE:         base/applications/games/solitaire/solundo.cpp
 * PURPOSE:      Undo module for Solitaire
 * PROGRAMMER:   Tibor Lajos Füzi
 */

#include "solitaire.h"

// source_id and destination_id store the source and destination of the cards
// that were moved. These ids are defined in solitaire.h and can be DECK_ID, PILE_ID,
// [SUIT_ID..SUIT_ID + 3], [ROW_ID..ROW_ID + NUM_ROW_STACKS - 1].
// -1 means that there is no action stored in the undo module.
static int source_id = -1;
static int destination_id = -1;

// Number of cards that were moved.
static int number_of_cards = 0;

// The score before the action was taken.
static int prev_score = 0;

// The number of visible pile cards before the action was taken.
static int prev_visible_pile_cards = 0;

void SetUndo(
    int set_source_id,
    int set_destination_id,
    int set_number_of_cards,
    int set_prev_score,
    int set_prev_visible_pile_cards)
{
    if ((set_source_id == set_destination_id) || (set_number_of_cards == 0))
        return;

    source_id = set_source_id;
    destination_id = set_destination_id;
    number_of_cards = set_number_of_cards;
    prev_score = set_prev_score;
    prev_visible_pile_cards = set_prev_visible_pile_cards;
    SetUndoMenuState(true);
}

void ClearUndo(void)
{
    source_id = -1;
    destination_id = -1;
    number_of_cards = 0;
    SetUndoMenuState(false);
}

void Undo(void)
{
    CARDREGION *source = NULL;
    CARDREGION *destination = NULL;

    if ((source_id < 1)                                  ||
        (source_id > (ROW_ID + NUM_ROW_STACKS - 1))      ||
        (destination_id < 1)                             ||
        (destination_id > (ROW_ID + NUM_ROW_STACKS - 1)) ||
        (number_of_cards < 1))
    {
        ClearUndo();
        return;
    }

    if (source_id >= ROW_ID)
        source = pRowStack[source_id - ROW_ID];
    else if ((source_id >= SUIT_ID) && (source_id < SUIT_ID + 4))
        source = pSuitStack[source_id - SUIT_ID];
    else if (source_id == PILE_ID)
        source = pPile;
    else if (source_id == DECK_ID)
        source = pDeck;

    if (destination_id >= ROW_ID)
        destination = pRowStack[destination_id - ROW_ID];
    else if ((destination_id >= SUIT_ID) && (destination_id < SUIT_ID + 4))
        destination = pSuitStack[destination_id - SUIT_ID];
    else if (destination_id == PILE_ID)
        destination = pPile;
    else if (destination_id == DECK_ID)
        destination = pDeck;

    if (destination == NULL || source == NULL)
    {
        ClearUndo();
        return;
    }

    // If the player clicked on the deck.
    if (destination == pPile && source == pDeck)
    {
        // Put back the cards on the deck in reversed order.
        CARDSTACK tmp;
        CardStack_PopN(&activepile, number_of_cards, &tmp);
        CardStack_Reverse(&tmp);
        CardRegion_PushStack(source, &tmp);
        // Restore the pile to be the top cards in the active pile.
        CardRegion_Clear(destination);
        if (prev_visible_pile_cards <= 1)
        {
            CardRegion_SetOffsets(destination, 0,0);
            CardRegion_SetCardStack(destination, &activepile);
        }
        else
        {
            CardStack_TopN(&activepile, prev_visible_pile_cards, &tmp);
            CardRegion_SetOffsets(destination, CS_DEFXOFF, 1);
            CardRegion_PushStack(destination, &tmp);
        }
        VisiblePileCards = prev_visible_pile_cards;
    }

    // If the player clicked on the empty deck.
    else if (source == pPile && destination == pDeck)
    {
        // Put back all the cards from the deck to the active pile in reversed order.
        CardRegion_Reverse(destination);
        CardStack_PushStack(&activepile, CardRegion_GetCardStack(destination));
        CardRegion_Clear(destination);
        if (prev_visible_pile_cards <= 1)
        {
            CardRegion_SetOffsets(source, 0,0);
            CardRegion_SetCardStack(source, &activepile);
        }
        else
        {
            CARDSTACK tmp;
            CardStack_TopN(&activepile, prev_visible_pile_cards, &tmp);
            CardRegion_SetOffsets(source, CS_DEFXOFF, 1);
            CardRegion_PushStack(source, &tmp);
        }
        VisiblePileCards = prev_visible_pile_cards;
    }

    // If the player moved one card from the pile.
    else if (source == pPile)
    {
        CARDSTACK tmp;
        CardRegion_PopN(destination, 1, &tmp);
        CardStack_PushStack(&activepile, &tmp);
        if (prev_visible_pile_cards <= 1)
        {
            CardRegion_PushStack(source, &tmp);
        }
        else
        {
            CardRegion_Clear(source);
            CardStack_TopN(&activepile, prev_visible_pile_cards, &tmp);
            CardRegion_PushStack(source, &tmp);
            CardRegion_SetOffsets(source, CS_DEFXOFF, 1);
        }
        VisiblePileCards = prev_visible_pile_cards;
    }

    // If the player moved cards between row stacks / suit stacks.
    else
    {
        CardRegion_MoveCard(destination, source, number_of_cards, false);
    }

    lScore = prev_score;

    // -2 points for the undo in standard score mode.
    if (GetScoreMode() == SCORE_STD)
        lScore = lScore >= 2 ? lScore - 2 : 0;

    UpdateStatusBar();

    CardWindow_Redraw(SolWnd);
    ClearUndo();
}
