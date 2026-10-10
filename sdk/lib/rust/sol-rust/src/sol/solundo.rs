use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut source_id: i32 = -1;
pub static mut destination_id: i32 = -1;
pub static mut number_of_cards: i32 = 0;
pub static mut prev_score: i32 = 0;
pub static mut prev_visible_pile_cards: i32 = 0;
#[no_mangle]
pub unsafe extern "C" fn SetUndo(
    set_source_id: i32,
    set_destination_id: i32,
    set_number_of_cards: i32,
    set_prev_score: i32,
    set_prev_visible_pile_cards: i32,
) {
    if (set_source_id == set_destination_id) || (set_number_of_cards == 0) {
        return;
    }

    source_id = set_source_id;
    destination_id = set_destination_id;
    number_of_cards = set_number_of_cards;
    prev_score = set_prev_score;
    prev_visible_pile_cards = set_prev_visible_pile_cards;
    SetUndoMenuState((true) as i32);
}
#[no_mangle]
pub unsafe extern "C" fn ClearUndo() {
    source_id = -1;
    destination_id = -1;
    number_of_cards = 0;
    SetUndoMenuState((false) as i32);
}
#[no_mangle]
pub unsafe extern "C" fn Undo() {
    let mut source: *mut CARDREGION = core::ptr::null_mut();
    let mut destination: *mut CARDREGION = core::ptr::null_mut();

    if (source_id < 1)
        || ((source_id) as u32 > (ROW_ID + NUM_ROW_STACKS - 1))
        || (destination_id < 1)
        || ((destination_id) as u32 > (ROW_ID + NUM_ROW_STACKS - 1))
        || (number_of_cards < 1)
    {
        ClearUndo();
        return;
    }

    if source_id >= (ROW_ID as i32) {
        source = pRowStack[(source_id - (ROW_ID as i32)) as usize];
    } else if (source_id >= (SUIT_ID as i32)) && (((source_id) as u32) < SUIT_ID + 4) {
        source = pSuitStack[(source_id - (SUIT_ID as i32)) as usize];
    } else if source_id == (PILE_ID as i32) {
        source = pPile;
    } else if source_id == (DECK_ID as i32) {
        source = pDeck;
    }

    if destination_id >= (ROW_ID as i32) {
        destination = pRowStack[(destination_id - (ROW_ID as i32)) as usize];
    } else if (destination_id >= (SUIT_ID as i32)) && (((destination_id) as u32) < SUIT_ID + 4) {
        destination = pSuitStack[(destination_id - (SUIT_ID as i32)) as usize];
    } else if destination_id == (PILE_ID as i32) {
        destination = pPile;
    } else if destination_id == (DECK_ID as i32) {
        destination = pDeck;
    }

    if destination.is_null() || source.is_null() {
        ClearUndo();
        return;
    }

    // If the player clicked on the deck.
    if destination == pPile && source == pDeck {
        // Put back the cards on the deck in reversed order.
        let mut tmp: CARDSTACK = core::mem::zeroed();
        CardStack_PopN(core::ptr::addr_of_mut!(activepile), number_of_cards, core::ptr::addr_of_mut!(tmp));
        CardStack_Reverse(core::ptr::addr_of_mut!(tmp));
        CardRegion_PushStack(source, core::ptr::addr_of_mut!(tmp));
        // Restore the pile to be the top cards in the active pile.
        CardRegion_Clear(destination);
        if prev_visible_pile_cards <= 1 {
            CardRegion_SetOffsets(destination, 0, 0);
            CardRegion_SetCardStack(destination, core::ptr::addr_of_mut!(activepile));
        } else {
            CardStack_TopN(core::ptr::addr_of_mut!(activepile), prev_visible_pile_cards, core::ptr::addr_of_mut!(tmp));
            CardRegion_SetOffsets(destination, (CS_DEFXOFF) as INT, 1);
            CardRegion_PushStack(destination, core::ptr::addr_of_mut!(tmp));
        }
        VisiblePileCards = prev_visible_pile_cards;
    }

    // If the player clicked on the empty deck.
    else if source == pPile && destination == pDeck {
        // Put back all the cards from the deck to the active pile in reversed order.
        CardRegion_Reverse(destination);
        CardStack_PushStack(core::ptr::addr_of_mut!(activepile), CardRegion_GetCardStack(destination));
        CardRegion_Clear(destination);
        if prev_visible_pile_cards <= 1 {
            CardRegion_SetOffsets(source, 0, 0);
            CardRegion_SetCardStack(source, core::ptr::addr_of_mut!(activepile));
        } else {
            let mut tmp: CARDSTACK = core::mem::zeroed();
            CardStack_TopN(core::ptr::addr_of_mut!(activepile), prev_visible_pile_cards, core::ptr::addr_of_mut!(tmp));
            CardRegion_SetOffsets(source, (CS_DEFXOFF) as INT, 1);
            CardRegion_PushStack(source, core::ptr::addr_of_mut!(tmp));
        }
        VisiblePileCards = prev_visible_pile_cards;
    }

    // If the player moved one card from the pile.
    else if source == pPile {
        let mut tmp: CARDSTACK = core::mem::zeroed();
        CardRegion_PopN(destination, 1, core::ptr::addr_of_mut!(tmp));
        CardStack_PushStack(core::ptr::addr_of_mut!(activepile), core::ptr::addr_of_mut!(tmp));
        if prev_visible_pile_cards <= 1 {
            CardRegion_PushStack(source, core::ptr::addr_of_mut!(tmp));
        } else {
            CardRegion_Clear(source);
            CardStack_TopN(core::ptr::addr_of_mut!(activepile), prev_visible_pile_cards, core::ptr::addr_of_mut!(tmp));
            CardRegion_PushStack(source, core::ptr::addr_of_mut!(tmp));
            CardRegion_SetOffsets(source, (CS_DEFXOFF) as INT, 1);
        }
        VisiblePileCards = prev_visible_pile_cards;
    }

    // If the player moved cards between row stacks / suit stacks.
    else {
        CardRegion_MoveCard(destination, source, number_of_cards, (false) as i32);
    }

    lScore = prev_score;

    // -2 points for the undo in standard score mode.
    if GetScoreMode() == (SCORE_STD as i32) {
        lScore = if lScore >= 2 { lScore - 2 } else { 0 };
    }

    UpdateStatusBar();

    CardWindow_Redraw(SolWnd);
    ClearUndo();
}
