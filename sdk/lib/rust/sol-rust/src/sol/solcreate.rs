use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// Wrapper functions to convert extern "C" callbacks to extern "system" for cardlib-rust
unsafe extern "system" fn DeckClickProc_Wrapper(pRegion: *mut CARDREGION, nNumCards: INT) {
    DeckClickProc(pRegion, nNumCards)
}
unsafe extern "system" fn PileDragProc_Wrapper(pRegion: *mut CARDREGION, nNumDragging: INT) -> BOOL {
    PileDragProc(pRegion, nNumDragging)
}
unsafe extern "system" fn PileDblClickProc_Wrapper(pRegion: *mut CARDREGION, nNumCards: INT) {
    PileDblClickProc(pRegion, nNumCards)
}
unsafe extern "system" fn PileRemoveProc_Wrapper(pRegion: *mut CARDREGION, nRemoved: INT) {
    PileRemoveProc(pRegion, nRemoved)
}
unsafe extern "system" fn SuitStackDragProc_Wrapper(pRegion: *mut CARDREGION, nNumDragging: INT) -> BOOL {
    SuitStackDragProc(pRegion, nNumDragging)
}
unsafe extern "system" fn SuitStackDropProc_Wrapper(pRegion: *mut CARDREGION, pDragCards: *mut CARDSTACK) -> BOOL {
    SuitStackDropProc(pRegion, pDragCards)
}
unsafe extern "system" fn SuitStackAddProc_Wrapper(pRegion: *mut CARDREGION, pAdded: *const CARDSTACK) {
    SuitStackAddProc(pRegion, pAdded)
}
unsafe extern "system" fn RowStackDragProc_Wrapper(pRegion: *mut CARDREGION, nNumDragging: INT) -> BOOL {
    RowStackDragProc(pRegion, nNumDragging)
}
unsafe extern "system" fn RowStackDropProc_Wrapper(pRegion: *mut CARDREGION, pDragCards: *mut CARDSTACK) -> BOOL {
    RowStackDropProc(pRegion, pDragCards)
}
unsafe extern "system" fn RowStackClickProc_Wrapper(pRegion: *mut CARDREGION, nNumCards: INT) {
    RowStackClickProc(pRegion, nNumCards)
}
unsafe extern "system" fn RowStackDblClickProc_Wrapper(pRegion: *mut CARDREGION, nNumCards: INT) {
    RowStackDblClickProc(pRegion, nNumCards)
}
unsafe extern "system" fn RowStackRemoveProc_Wrapper(pRegion: *mut CARDREGION, nRemoved: INT) {
    RowStackRemoveProc(pRegion, nRemoved)
}

#[no_mangle]
pub static mut pDeck: *mut CARDREGION = core::ptr::null_mut();
#[no_mangle]
pub static mut pPile: *mut CARDREGION = core::ptr::null_mut();
#[no_mangle]
pub static mut pSuitStack: [*mut CARDREGION; (NUM_SUIT_STACKS as usize)] = [core::ptr::null_mut(); (NUM_SUIT_STACKS as usize)];
#[no_mangle]
pub static mut pRowStack: [*mut CARDREGION; (NUM_ROW_STACKS as usize)] = [core::ptr::null_mut(); (NUM_ROW_STACKS as usize)];
#[no_mangle]
pub static mut yRowStackCardOffset: i32 = 0;
#[no_mangle]
pub static mut xPileAdjust: i32 = 0;

unsafe extern "C" fn ColumnAdjust(nColumn: INT) -> INT {
    return (nColumn - (NUM_ROW_STACKS / 2) as i32) * (CardLib_CardWidth() + (X_BORDER as i32));
}

#[no_mangle]
pub unsafe extern "C" fn CreateSol() {
    let mut i: i32;
    let yTopRow: i32 = (Y_BORDER as i32);
    let yTableau: i32 = (Y_BORDER + (CardLib_CardHeight()) as u32 + Y_BORDER) as i32;

    /*
     * How far each buried tableau card peeks out from under the one above it.
     * A sixth of the card height leaves the rank and suit in the top-left
     * corner readable, and derives from the artwork so the big deck fans
     * proportionally rather than overlapping into an unreadable stripe.
     */
    yRowStackCardOffset = CardLib_CardHeight() / 6;
    if yRowStackCardOffset < 1 {
        yRowStackCardOffset = 1;
    }

    xPileAdjust = ColumnAdjust(1);

    /* ---- the stock -------------------------------------------------- *
     * No fan and no drag: the only thing a player does to it is click it.
     * The threed count makes a 24-card stock four blits deep instead of
     * twenty-four, all landing in the same place.
     */
    pDeck = CardWindow_CreateRegion(SolWnd, (DECK_ID) as INT, (TRUE) as BOOL, 0, yTopRow, 0, 0);
    CardRegion_SetFaceDirection(pDeck, CS_FACE_DOWN, 0);
    CardRegion_SetPlacement(pDeck, CS_XJUST_CENTER, CS_YJUST_NONE, ColumnAdjust(0), 0);
    CardRegion_SetEmptyImage(pDeck, CS_EI_CIRC);
    CardRegion_SetThreedCount(pDeck, 6);
    CardRegion_SetDragRule(pDeck, CS_DRAG_NONE, None);
    CardRegion_SetDropRule(pDeck, CS_DROP_NONE, None);
    CardRegion_SetClickProc(pDeck, Some(DeckClickProc_Wrapper));

    /* ---- the waste --------------------------------------------------- *
     * Offsets are set per deal by UpdatePileDisplay: squared up when one card
     * is turned at a time, fanned right when three are.  Only the top card
     * ever leaves, and nothing may be dropped back on it.
     */
    pPile = CardWindow_CreateRegion(SolWnd, (PILE_ID) as INT, (TRUE) as BOOL, 0, yTopRow, 0, 0);
    CardRegion_SetFaceDirection(pPile, CS_FACE_UP, 0);
    CardRegion_SetPlacement(pPile, CS_XJUST_CENTER, CS_YJUST_NONE, xPileAdjust, 0);
    CardRegion_SetEmptyImage(pPile, CS_EI_NONE);
    CardRegion_SetDragRule(pPile, CS_DRAG_CALLBACK, Some(PileDragProc_Wrapper));
    CardRegion_SetDropRule(pPile, CS_DROP_NONE, None);
    CardRegion_SetDblClickProc(pPile, Some(PileDblClickProc_Wrapper));
    CardRegion_SetRemoveCardProc(pPile, Some(PileRemoveProc_Wrapper));

    /* ---- the foundations --------------------------------------------- *
     * Ace up to King in suit.  Squared up, so a threed count of a full suit
     * reduces a thirteen-card pile to the single image you can actually see.
     * The add callback is where the win is noticed, which is why the
     * double-click shortcut has to go through CardRegion_SimulateDrag.
     */
    i = 0;
    while i < (NUM_SUIT_STACKS as i32) {
        pSuitStack[i as usize] = CardWindow_CreateRegion(SolWnd, (SUIT_ID + (i) as u32) as INT, (TRUE) as BOOL, 0, yTopRow, 0, 0);
        CardRegion_SetFaceDirection(pSuitStack[i as usize], CS_FACE_UP, 0);
        CardRegion_SetPlacement(pSuitStack[i as usize], CS_XJUST_CENTER, CS_YJUST_NONE,
                                ColumnAdjust((NUM_ROW_STACKS - NUM_SUIT_STACKS + (i) as u32) as INT), 0);
        CardRegion_SetEmptyImage(pSuitStack[i as usize], CS_EI_SUNK);
        CardRegion_SetThreedCount(pSuitStack[i as usize], 13);
        CardRegion_SetDragRule(pSuitStack[i as usize], CS_DRAG_CALLBACK, Some(SuitStackDragProc_Wrapper));
        CardRegion_SetDropRule(pSuitStack[i as usize], CS_DROP_CALLBACK, Some(SuitStackDropProc_Wrapper));
        CardRegion_SetAddCardProc(pSuitStack[i as usize], Some(SuitStackAddProc_Wrapper));
        i += 1;
    }

    /* ---- the tableau -------------------------------------------------- *
     * Created last so they sit on top: a long column fans downwards and must
     * win the hit test against anything it grows over.  CS_FACE_DOWNUP with
     * the count of still-hidden cards is the whole face-up/face-down model;
     * the remove callback decrements it when a column is stripped bare.
     */
    i = 0;
    while i < (NUM_ROW_STACKS as i32) {
        pRowStack[i as usize] = CardWindow_CreateRegion(SolWnd, (ROW_ID + (i) as u32) as INT, (TRUE) as BOOL,
                                               0, yTableau, 0, yRowStackCardOffset);
        CardRegion_SetFaceDirection(pRowStack[i as usize], CS_FACE_DOWNUP, 0);
        CardRegion_SetPlacement(pRowStack[i as usize], CS_XJUST_CENTER, CS_YJUST_NONE, ColumnAdjust(i), 0);
        CardRegion_SetEmptyImage(pRowStack[i as usize], CS_EI_SUNK);
        CardRegion_SetDragRule(pRowStack[i as usize], CS_DRAG_CALLBACK, Some(RowStackDragProc_Wrapper));
        CardRegion_SetDropRule(pRowStack[i as usize], CS_DROP_CALLBACK, Some(RowStackDropProc_Wrapper));
        CardRegion_SetClickProc(pRowStack[i as usize], Some(RowStackClickProc_Wrapper));
        CardRegion_SetDblClickProc(pRowStack[i as usize], Some(RowStackDblClickProc_Wrapper));
        CardRegion_SetRemoveCardProc(pRowStack[i as usize], Some(RowStackRemoveProc_Wrapper));
        i += 1;
    }
}
