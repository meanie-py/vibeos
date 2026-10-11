use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut activepile: CARDSTACK = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut VisiblePileCards: INT = 0;
pub static mut pDragSource: *mut CARDREGION = core::ptr::null_mut();

unsafe extern "C" fn StartPlay() {
    if ((fGameStarted) == 0) {
        fGameStarted = (TRUE) as i32;
        StartPlayTimer();
    }
}

unsafe extern "C" fn UpdateDeckImage() {
    CardRegion_SetEmptyImage(pDeck, if CardStack_NumCards(core::ptr::addr_of_mut!(activepile)) > 0 { CS_EI_CIRC } else { CS_EI_X });
}
#[no_mangle]
pub unsafe extern "C" fn UpdatePilePlacement() {
    let mut nAdjust: i32 = xPileAdjust;

    if VisiblePileCards > 1 {
        nAdjust += ((VisiblePileCards - 1) * (CS_DEFXOFF as i32)) / 2;
    }

    CardRegion_SetPlacement(pPile, CS_XJUST_CENTER, CS_YJUST_NONE, nAdjust, 0);
}
#[no_mangle]
pub unsafe extern "C" fn UpdatePileDisplay(mut nVisible: i32) {
    let mut tmp: CARDSTACK = core::mem::zeroed();
    let nTotal: i32 = CardStack_NumCards(core::ptr::addr_of_mut!(activepile));

    if nVisible > nTotal {
        nVisible = nTotal;
    }
    if nVisible < 1 {
        nVisible = if nTotal > 0 {
            1
        } else {
            0
        };
    }

    CardRegion_Clear(pPile);

    if nVisible <= 1 {
        CardRegion_SetOffsets(pPile, 0, 0);
        CardRegion_SetCardStack(pPile, core::ptr::addr_of_mut!(activepile));
    } else {
        CardStack_TopN(core::ptr::addr_of_mut!(activepile), nVisible, core::ptr::addr_of_mut!(tmp));
        CardRegion_SetOffsets(pPile, (CS_DEFXOFF) as INT, 1);
        CardRegion_PushStack(pPile, core::ptr::addr_of_mut!(tmp));
    }

    VisiblePileCards = nVisible;
    UpdatePilePlacement();
    UpdateDeckImage();
}
#[no_mangle]
pub unsafe extern "C" fn TurnRowStackCard(pRegion: *mut CARDREGION) {
    let mut nFaceDown: INT = 0;

    CardRegion_GetFaceDirection(pRegion, core::ptr::addr_of_mut!(nFaceDown));

    if nFaceDown > 0 && CardRegion_NumCards(pRegion) <= nFaceDown {
        nFaceDown -= 1;
        CardRegion_SetFaceDirection(pRegion, CS_FACE_DOWNUP, nFaceDown);

        if GetScoreMode() == (SCORE_STD as i32) {
            lScore += (SCORE_STD_TURNOVER as i32);
            UpdateStatusBar();
        }

        CardRegion_Redraw(pRegion);
    }
}

unsafe extern "C" fn IsSuitStackId(nId: i32) -> i32 {
    return ((nId >= (SUIT_ID as i32)) && (((nId) as u32) < SUIT_ID + NUM_SUIT_STACKS)) as i32;
}

unsafe extern "C" fn CanDropOnSuitStack(pRegion: *mut CARDREGION, card: CARD) -> BOOL {
    let mut top: CARD;

    if CardRegion_NumCards(pRegion) == 0 {
        return (Card_LoVal(card) == 1 /* Ace */) as BOOL;
    }

    top = CardStack_TopCard(CardRegion_GetCardStack(pRegion));

    return ((Card_Suit(top) == Card_Suit(card)) &&
            (Card_LoVal(top) + 1 == Card_LoVal(card))) as BOOL;
}

unsafe extern "C" fn ScoreMove(nSrcId: INT, nDestId: INT, nCards: INT) {
    let fToSuit: BOOL = IsSuitStackId(nDestId);
    let fFromSuit: BOOL = IsSuitStackId(nSrcId);

    match (GetScoreMode()) as u32 {
        SCORE_STD => {
            if fToSuit != 0 {
                lScore += (SCORE_STD_TO_SUIT * (nCards) as u32) as i32;
            } else if fFromSuit != 0 {
                lScore -= (SCORE_STD_FROM_SUIT * (nCards) as u32) as i32;
            } else if nSrcId == (PILE_ID as i32) {
                lScore += (SCORE_STD_PILE_TO_ROW * (nCards) as u32) as i32;
            }

            /* Standard scoring has a floor of zero, which is also the floor
               solundo.cpp clamps to when it charges for an undo. */
            if lScore < 0 {
                lScore = 0;
            }
        }

        SCORE_VEGAS => {
            /* Vegas is a bank, not a tally: it may legitimately go negative
               and must not be clamped. */
            if fToSuit != 0 {
                lScore += (SCORE_VEGAS_PER_CARD * (nCards) as u32) as i32;
            } else if fFromSuit != 0 {
                lScore -= (SCORE_VEGAS_PER_CARD * (nCards) as u32) as i32;
            }
        }

        _ => {}
    }

    UpdateStatusBar();
}

unsafe extern "C" fn CommitMove(pSrc: *mut CARDREGION, pDest: *mut CARDREGION, nCards: INT) {
    let mut nSrcId: INT;
    let mut nDestId: INT;

    if pSrc.is_null() || pSrc == pDest {
        return;
    }

    nSrcId = CardRegion_Id(pSrc);
    nDestId = CardRegion_Id(pDest);

    SetUndo(nSrcId, nDestId, nCards, lScore, VisiblePileCards);
    ScoreMove(nSrcId, nDestId, nCards);
    StartPlay();
}

#[no_mangle]
pub unsafe extern "C" fn NewGame() {
    let mut deck: CARDSTACK = core::mem::zeroed();
    let mut tmp: CARDSTACK = core::mem::zeroed();
    let mut i: INT;
    let mut j: INT;

    StopPlayTimer();
    ClearUndo();

    if GetScoreMode() == (SCORE_VEGAS as i32) {
        if fKeepScore == 0 {
            lScore = 0;
        }
        lScore -= (SCORE_VEGAS_STAKE as i32);
    } else {
        lScore = 0;
    }

    CardWindow_EmptyStacks(SolWnd);
    CardStack_Clear(core::ptr::addr_of_mut!(activepile));
    VisiblePileCards = 0;

    CardStack_NewDeck(core::ptr::addr_of_mut!(deck));
    CardStack_Shuffle(core::ptr::addr_of_mut!(deck));

    i = 0;
    while i < (NUM_ROW_STACKS as i32) {
        CardStack_Clear(core::ptr::addr_of_mut!(tmp));
        j = 0;
        while j <= i {
            CardStack_PushCard(
                core::ptr::addr_of_mut!(tmp),
                CardStack_PopCard(core::ptr::addr_of_mut!(deck)),
            );
            j += 1;
        }

        CardRegion_SetFaceDirection(pRowStack[i as usize], CS_FACE_DOWNUP, i);
        CardRegion_SetCardStack(pRowStack[i as usize], core::ptr::addr_of_mut!(tmp));
        i += 1;
    }

    i = 0;
    while i < (NUM_SUIT_STACKS as i32) {
        CardRegion_Clear(pSuitStack[i as usize]);
        i += 1;
    }

    CardRegion_SetCardStack(pDeck, core::ptr::addr_of_mut!(deck));

    UpdatePileDisplay(0);

    fGameStarted = (FALSE) as i32;
    dwElapsed = 0;

    UpdateStatusBar();
    CardWindow_Redraw(SolWnd);
}
#[no_mangle]
pub unsafe extern "C" fn DeckClickProc(pRegion: *mut CARDREGION, nNumCards: INT) {
    let mut tmp: CARDSTACK = core::mem::zeroed();
    let nDeckCards: INT = CardRegion_NumCards(pDeck);

    UNREFERENCED_PARAMETER!(pRegion);
    UNREFERENCED_PARAMETER!(nNumCards);

    if nDeckCards > 0 {
        let mut nDraw: INT = nDrawCards;

        if nDraw > nDeckCards {
            nDraw = nDeckCards;
        }

        StartPlay();
        SetUndo((DECK_ID as i32), (PILE_ID as i32), nDraw, lScore, VisiblePileCards);

        CardRegion_PopN(pDeck, nDraw, core::ptr::addr_of_mut!(tmp));

        // PopN keeps the stock's order, so tmp's top is the card that came off
        // FIRST. Reversing makes the last one dealt the top of the waste -
        // the one the player may take - and it is the order solundo.cpp
        // reverses again to put them back.
        CardStack_Reverse(core::ptr::addr_of_mut!(tmp));
        CardStack_PushStack(core::ptr::addr_of_mut!(activepile), core::ptr::addr_of_mut!(tmp));

        UpdatePileDisplay(nDraw);
    } else {
        // Stock exhausted: the whole waste goes back under it, in the reverse
        // of the order it came out, so the cycle repeats.
        let nTotal: INT = CardStack_NumCards(core::ptr::addr_of_mut!(activepile));

        if nTotal == 0 {
            return;
        }

        StartPlay();
        SetUndo((PILE_ID as i32), (DECK_ID as i32), nTotal, lScore, VisiblePileCards);

        tmp = activepile;
        CardStack_Reverse(core::ptr::addr_of_mut!(tmp));
        CardRegion_SetCardStack(pDeck, core::ptr::addr_of_mut!(tmp));
        CardStack_Clear(core::ptr::addr_of_mut!(activepile));

        UpdatePileDisplay(0);
    }

    CardRegion_Update(pDeck);
    CardWindow_Redraw(SolWnd);
}
#[no_mangle]
pub unsafe extern "C" fn PileDragProc(pRegion: *mut CARDREGION, nNumDragging: INT) -> BOOL {
    /* Only the card on top of the waste is in play. */
    if nNumDragging != 1 {
        return (FALSE) as BOOL;
    }

    pDragSource = pRegion;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SuitStackDragProc(pRegion: *mut CARDREGION, nNumDragging: INT) -> BOOL {
    /* A card may come back off a foundation, but only the top one. */
    if nNumDragging != 1 {
        return (FALSE) as BOOL;
    }

    pDragSource = pRegion;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn RowStackDragProc(pRegion: *mut CARDREGION, nNumDragging: INT) -> BOOL {
    let mut pStack: *const CARDSTACK;
    let mut nFaceDown: INT = 0;
    let mut i: INT;

    if nNumDragging < 1 {
        return (FALSE) as BOOL;
    }

    CardRegion_GetFaceDirection(pRegion, core::ptr::addr_of_mut!(nFaceDown));

    /* Never lift a card that is still face down. */
    if nNumDragging > CardRegion_NumCards(pRegion) - nFaceDown {
        return (FALSE) as BOOL;
    }

    /* A run only travels as a unit if it already IS one: descending in rank
       and alternating in colour.  Index 0 is the top of the pile, so card i
       rests on card i + 1 and must be one lower and the other colour. */
    pStack = CardRegion_GetCardStack(pRegion);

    i = 0;
    while i < nNumDragging - 1 {
        let upper: CARD = CardStack_Get(pStack, i);
        let lower: CARD = CardStack_Get(pStack, i + 1);

        if Card_LoVal(upper) + 1 != Card_LoVal(lower) {
            return (FALSE) as BOOL;
        }
        if Card_IsRed(upper) == Card_IsRed(lower) {
            return (FALSE) as BOOL;
        }
        i += 1;
    }

    pDragSource = pRegion;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SuitStackDropProc(pRegion: *mut CARDREGION, pDragCards: *mut CARDSTACK) -> BOOL {
    if CardStack_NumCards(pDragCards) != 1 {
        return (FALSE) as BOOL;
    }

    if CanDropOnSuitStack(pRegion, CardStack_Get(pDragCards, 0)) == 0 {
        return (FALSE) as BOOL;
    }

    CommitMove(pDragSource, pRegion, 1);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn RowStackDropProc(pRegion: *mut CARDREGION, pDragCards: *mut CARDSTACK) -> BOOL {
    let cardLanding: CARD;
    let nDrag: INT = CardStack_NumCards(pDragCards);

    if nDrag < 1 {
        return (FALSE) as BOOL;
    }

    /* Index 0 is the top of a pile, so the card that actually touches the
       column is the one at the  end of the run being carried. */
    cardLanding = CardStack_Get(pDragCards, nDrag - 1);

    if CardRegion_NumCards(pRegion) == 0 {
        /* An empty column is a King's seat and nothing else's. */
        if (Card_LoVal(cardLanding)) as u32 != King {
            return (FALSE) as BOOL;
        }
    } else {
        let cardTop: CARD = CardStack_TopCard(CardRegion_GetCardStack(pRegion));

        if Card_LoVal(cardLanding) + 1 != Card_LoVal(cardTop) {
            return (FALSE) as BOOL;
        }
        if Card_IsRed(cardLanding) == Card_IsRed(cardTop) {
            return (FALSE) as BOOL;
        }
    }

    CommitMove(pDragSource, pRegion, nDrag);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn RowStackClickProc(pRegion: *mut CARDREGION, nNumCards: INT) {
    UNREFERENCED_PARAMETER!(nNumCards);
    TurnRowStackCard(pRegion);
}

unsafe extern "C" fn PlayToSuitStack(pSrc: *mut CARDREGION) -> BOOL {
    let mut card: CARD;
    let mut i: INT;

    if CardRegion_NumCards(pSrc) == 0 {
        return (FALSE) as BOOL;
    }

    card = CardStack_TopCard(CardRegion_GetCardStack(pSrc));

    i = 0;
    while i < (NUM_SUIT_STACKS as i32) {
        if ((CanDropOnSuitStack(pSuitStack[i as usize], card)) == 0) {
            i += 1;
            continue;
        }

        pDragSource = pSrc;
        CardRegion_SimulateDrag(pSrc, pSuitStack[i as usize], 1, (TRUE) as BOOL);
        CardWindow_Redraw(SolWnd);
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn PileDblClickProc(pRegion: *mut CARDREGION, nNumCards: INT) {
    UNREFERENCED_PARAMETER!(nNumCards);
    PlayToSuitStack(pRegion);
}
#[no_mangle]
pub unsafe extern "C" fn RowStackDblClickProc(pRegion: *mut CARDREGION, nNumCards: INT) {
    let mut nFaceDown: INT = core::mem::zeroed();

    /* Only the exposed card of a column plays itself; a double-click further
       down the fan is picking out a buried card and means nothing here. */
    if nNumCards != 1 {
        return;
    }

    CardRegion_GetFaceDirection(pRegion, core::ptr::addr_of_mut!(nFaceDown));
    if CardRegion_NumCards(pRegion) <= nFaceDown {
        return;
    }

    PlayToSuitStack(pRegion);
}
#[no_mangle]
pub unsafe extern "C" fn SuitStackAddProc(pRegion: *mut CARDREGION, pAdded: *const CARDSTACK) {
    let mut i: INT;
    let mut nTotal: INT = 0;

    UNREFERENCED_PARAMETER!(pRegion);
    UNREFERENCED_PARAMETER!(pAdded);

    i = 0;
    while i < (NUM_SUIT_STACKS as i32) {
        nTotal += CardRegion_NumCards(pSuitStack[i as usize]);
        i += 1;
    }

    /* Posted, not called: see WM_SOL_GAMEWON in solitaire.h.  We are inside
       the library's drop handling and it is not finished with us yet. */
    if nTotal == (NUM_CARDS as i32) {
        PostMessage(hwndMain, WM_SOL_GAMEWON, 0, 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn PileRemoveProc(pRegion: *mut CARDREGION, nRemoved: INT) {
    let mut tmp: CARDSTACK = core::mem::zeroed();

    UNREFERENCED_PARAMETER!(pRegion);

    CardStack_PopN(core::ptr::addr_of_mut!(activepile), nRemoved, core::ptr::addr_of_mut!(tmp));

    if VisiblePileCards > 1 {
        VisiblePileCards -= nRemoved;
    }

    UpdatePileDisplay(VisiblePileCards);
}
#[no_mangle]
pub unsafe extern "C" fn RowStackRemoveProc(pRegion: *mut CARDREGION, nRemoved: INT) {
    UNREFERENCED_PARAMETER!(nRemoved);
    TurnRowStackCard(pRegion);
}
#[no_mangle]
pub unsafe extern "C" fn GameFinished() {
    let mut i: INT;

    StopPlayTimer();

    /* The time bonus only applies to a timed standard game, and only if the
       deal took long enough for the division to mean anything. */
    if GetScoreMode() == (SCORE_STD as i32) && fShowTime != 0 && dwElapsed > SCORE_STD_BONUS_MINTIME {
        lScore += (SCORE_STD_BONUS_BASE / dwElapsed) as INT;
    }

    UpdateStatusBar();
    ClearUndo();

    i = 0;
    while i < (NUM_SUIT_STACKS as i32) {
        CardRegion_Flash(pSuitStack[i as usize], 6, 100);
        i += 1;
    }

    MessageBox(hwndMain, (MsgWin).as_ptr(), (szAppName).as_ptr(), MB_OK | MB_ICONINFORMATION);

    if (MessageBox(hwndMain, (MsgDeal).as_ptr(), (szAppName).as_ptr(), MB_YESNO | MB_ICONQUESTION)) as u32 == IDYES {
        NewGame();
    } else {
        fGameStarted = (FALSE) as i32;
    }
}
