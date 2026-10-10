use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NUM_DECK_CARDS: i32 = 5;

pub const NUM_SMALLER_STACKS: i32 = 4;

pub const NUM_CARD_COLORS: i32 = 4;

pub const NUM_ONECOLOR_CARDS: i32 = 13;

pub const NUM_STD_CARDS: i32 = 52;

pub const NUM_SPIDER_CARDS: i32 = 104;

pub static mut deck: CARDSTACK = unsafe { core::mem::zeroed() };

pub static mut from: *mut CARDREGION = core::ptr::null_mut();

pub static mut pDeck: *mut CARDREGION = core::ptr::null_mut();

pub static mut pStack: [*mut CARDREGION; NUM_STACKS as usize] =
    [core::ptr::null_mut(); NUM_STACKS as usize];

#[no_mangle]
pub static mut fGameStarted: bool = false;

#[no_mangle]
pub static mut yRowStackCardOffset: i32 = 0;

pub static mut cardsFinished: i32 = 0;

// C: extern TCHAR MsgDeal[]; / extern TCHAR MsgWin[]; (incomplete array types, defined elsewhere)
#[no_mangle]
pub unsafe extern "C" fn CreatePlayDeck() -> CARDSTACK {
    let mut newStack: CARDSTACK = core::mem::zeroed();
    let mut i: i32;
    let mut colors: i32 = 1;
    let mut num: i32 = 0;

    CardStack_Clear(core::ptr::addr_of_mut!(newStack));

    match dwDifficulty {
        IDC_DIF_ONECOLOR => {
            colors = 1;
        }
        IDC_DIF_TWOCOLORS => {
            colors = 2;
        }
        IDC_DIF_FOURCOLORS => {
            colors = 4;
        }
        _ => {}
    }
    i = 0;
    while i < NUM_SPIDER_CARDS {
        num += NUM_CARD_COLORS / colors;
        let newCard: CARD = Card_FromIndex(num % NUM_STD_CARDS);
        CardStack_PushCard(core::ptr::addr_of_mut!(newStack), newCard);
        i += 1;
    }
    return newStack;
}
#[no_mangle]
pub unsafe extern "C" fn NewGame() {
    let mut i: i32;
    let mut j: i32;
    /* First four stack with five, all other with 4 */
    let mut covCards: i32 = 5;
    let mut fakeDeck: CARDSTACK = core::mem::zeroed();
    let mut temp: CARDSTACK = core::mem::zeroed();

    CardWindow_EmptyStacks(SpiderWnd);

    /* Create a new card-deck, fake deck */
    deck = CreatePlayDeck();
    CardStack_Shuffle(core::ptr::addr_of_mut!(deck));
    CardStack_NewDeck(core::ptr::addr_of_mut!(fakeDeck));
    CardStack_Shuffle(core::ptr::addr_of_mut!(fakeDeck));

    /* Reset progress value */
    cardsFinished = 0;

    /* Deal to each stack */
    i = 0;
    while i < (NUM_STACKS as i32) {
        CardStack_Clear(core::ptr::addr_of_mut!(temp));
        if i == NUM_SMALLER_STACKS {
            covCards -= 1;
        }
        j = 0;
        while j <= covCards {
            CardStack_PushCard(
                core::ptr::addr_of_mut!(temp),
                CardStack_PopCard(core::ptr::addr_of_mut!(deck)),
            );
            j += 1;
        }
        CardRegion_SetFaceDirection(pStack[i as usize], CS_FACE_DOWNUP, covCards);
        CardRegion_SetCardStack(pStack[i as usize], core::ptr::addr_of_mut!(temp));
        i += 1;
    }
    /* Deal five fake cards to the deck */
    CardStack_PopN(
        core::ptr::addr_of_mut!(fakeDeck),
        5,
        core::ptr::addr_of_mut!(temp),
    );
    CardRegion_SetCardStack(pDeck, core::ptr::addr_of_mut!(temp));

    CardWindow_Redraw(SpiderWnd);
    fGameStarted = false;
}
#[no_mangle]
pub unsafe extern "C" fn stackLookingGood(mystack: *const CARDSTACK, numChecks: i32) -> bool {
    let mut i: i32;
    i = 0;
    while i < numChecks {
        if Card_LoVal(CardStack_Get(mystack, i))
            != Card_LoVal(CardStack_Get(mystack, i + 1)) - 1
        {
            return false;
        }
        if Card_Suit(CardStack_Get(mystack, i))
            != Card_Suit(CardStack_Get(mystack, i + 1))
        {
            return false;
        }
        i += 1;
    }
    return true;
}
#[no_mangle]
pub unsafe extern "C" fn TurnStackCard(stackobj: *mut CARDREGION) {
    let mut numfacedown: i32;
    numfacedown = 0;

    CardRegion_GetFaceDirection(stackobj, core::ptr::addr_of_mut!(numfacedown));
    if CardRegion_NumCards(stackobj) <= numfacedown {
        if numfacedown > 0 {
            numfacedown -= 1;
        }
        CardRegion_SetFaceDirection(stackobj, CS_FACE_DOWNUP, numfacedown);
        CardRegion_Redraw(stackobj);
    }
}
#[no_mangle]
pub unsafe extern "system" fn DeckClickProc(stackobj: *mut CARDREGION, NumDragCards: i32) {
    let mut temp: CARDSTACK = core::mem::zeroed();
    let mut fakeDeck: CARDSTACK = *CardRegion_GetCardStack(pDeck);
    fGameStarted = true;

    if CardStack_NumCards(core::ptr::addr_of_mut!(fakeDeck)) != 0
        && CardStack_NumCards(core::ptr::addr_of_mut!(deck)) != 0
    {
        let mut i: i32;
        let mut facedown: i32 = 0;
        let mut faceup: i32 = 0;
        /* Add one card to every stack */
        i = 0;
        while i < (NUM_STACKS as i32) {
            temp = *CardRegion_GetCardStack(pStack[i as usize]);
            CardStack_PushCard(
                core::ptr::addr_of_mut!(temp),
                CardStack_PopCard(core::ptr::addr_of_mut!(deck)),
            );

            /* Check if we accidentally finished a row */
            CardRegion_GetFaceDirection(pStack[i as usize], core::ptr::addr_of_mut!(facedown));
            faceup = CardStack_NumCards(core::ptr::addr_of_mut!(temp)) - facedown;
            if faceup >= NUM_ONECOLOR_CARDS {
                /* Check stack finished, remove cards if so */
                if stackLookingGood(
                    core::ptr::addr_of_mut!(temp),
                    NUM_ONECOLOR_CARDS - 1,
                ) != false
                {
                    let mut j: i32;
                    j = 0;
                    while j < NUM_ONECOLOR_CARDS {
                        CardStack_RemoveCard(
                            core::ptr::addr_of_mut!(temp),
                            0,
                        );
                        j += 1;
                    }
                    cardsFinished += NUM_ONECOLOR_CARDS;
                    CardRegion_SetCardStack(pStack[i as usize], core::ptr::addr_of_mut!(temp));
                    /* Turn now topmost card */
                    TurnStackCard(pStack[i as usize]);
                }
            }
            CardRegion_SetCardStack(pStack[i as usize], core::ptr::addr_of_mut!(temp));
            i += 1;
        }
        /* Remove one card from the fake ones */
        CardStack_PopN(
            core::ptr::addr_of_mut!(fakeDeck),
            CardStack_NumCards(core::ptr::addr_of_mut!(fakeDeck)) - 1,
            core::ptr::addr_of_mut!(temp),
        );
        CardRegion_SetCardStack(pDeck, core::ptr::addr_of_mut!(temp));
    }
    CardRegion_Update(pDeck);
    CardWindow_Redraw(SpiderWnd);
}
#[no_mangle]
pub unsafe extern "system" fn StackDragProc(stackobj: *mut CARDREGION, numDragCards: i32) -> BOOL {
    let mut numfacedown: i32 = 0;
    let mut numcards: i32;

    CardRegion_GetFaceDirection(stackobj, core::ptr::addr_of_mut!(numfacedown));
    numcards = CardRegion_NumCards(stackobj);

    /* Only cards facing up */
    if numDragCards <= numcards - numfacedown {
        let mystack: *const CARDSTACK = CardRegion_GetCardStack(stackobj);
        /* Don't allow to drag unsuited cards */
        if !stackLookingGood(mystack, numDragCards - 1) {
            return 0;
        }
        /* Remember where the cards come from */
        from = stackobj;
        return 1;
    } else {
        return 0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn GameFinished() {
    CardWindow_EmptyStacks(SpiderWnd);

    MessageBox(CardWindow_GetHwnd(SpiderWnd), MsgWin.as_ptr(), szAppName.as_ptr(), MB_OK | MB_ICONINFORMATION);
    if IDYES == (MessageBox(CardWindow_GetHwnd(SpiderWnd), MsgDeal.as_ptr(), szAppName.as_ptr(), MB_YESNO | MB_ICONQUESTION)) as u32
    {
        NewGame();
    }
    else
    {
        fGameStarted = false;
    }
}
#[no_mangle]
pub unsafe extern "system" fn StackAddProc(stackobj: *mut CARDREGION, added: *const CARDSTACK) {
    if cardsFinished == NUM_SPIDER_CARDS {
        GameFinished();
    }
}
#[no_mangle]
pub unsafe extern "system" fn StackDropProc(stackobj: *mut CARDREGION, dragcards: *mut CARDSTACK) -> BOOL {
    let dragcard: CARD = CardStack_Get(dragcards, CardStack_NumCards(dragcards) - 1);
    let mut faceup: i32 = 0;
    let mut facedown: i32 = 0;

    /* Only drop our cards on other stacks */
    if CardRegion_Id(stackobj) == CardRegion_Id(from) {
        return 0;
    }

    /* If stack is empty, everything can be dropped */
    if CardRegion_NumCards(stackobj) != 0 {
        let mystack: *const CARDSTACK = CardRegion_GetCardStack(stackobj);

        /* Can only drop if card is 1 less */
        if Card_LoVal(CardStack_Get(mystack, 0)) != Card_LoVal(dragcard) + 1 {
            return 0;
        }

        /* Check if stack complete */
        CardRegion_GetFaceDirection(stackobj, core::ptr::addr_of_mut!(facedown));
        faceup = CardRegion_NumCards(stackobj) - facedown;

        if faceup + CardStack_NumCards(dragcards) >= NUM_ONECOLOR_CARDS {
            let mut i: i32 = 0;
            let mut max: i32 = NUM_ONECOLOR_CARDS - CardStack_NumCards(dragcards) - 1;

            /* Dragged cards have been checked to be in order, check stack cards */
            if Card_Suit(CardStack_Get(mystack, 0)) == Card_Suit(dragcard)
                && stackLookingGood(mystack, max)
            {
                let mut s: CARDSTACK = *CardRegion_GetCardStack(stackobj);

                /* Remove from card stack */
                i = 0;
                while i < max + 1 {
                    CardStack_RemoveCard(core::ptr::addr_of_mut!(s), 0);
                    i += 1;
                }
                /* Remove dragged cards: the drop is accepted but lands
                   nothing, because the run it completed has just been taken
                   off the board above. */
                CardStack_Clear(dragcards);
                CardRegion_SetCardStack(stackobj, core::ptr::addr_of_mut!(s));
                cardsFinished += NUM_ONECOLOR_CARDS;
                /* Flip top card of the dest stack */
                TurnStackCard(stackobj);
            }
        }
    }
    /* Flip the top card of the source stack */
    TurnStackCard(from);
    fGameStarted = true;
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn CreateSpider() {
    let mut i: i32;
    let mut pos: i32;

    /* Compute the value for yRowStackCardOffset based on the height of the card, so the card number and suite isn't hidden on larger cards except Ace */
    yRowStackCardOffset = ((CardLib_CardHeight() as f64) / 6.4) as i32;

    pDeck = CardWindow_CreateRegion(SpiderWnd, 0, (true) as i32, 0, 0, -15, 0);
    CardRegion_SetFaceDirection(pDeck, CS_FACE_DOWN, 0);
    CardRegion_SetEmptyImage(pDeck, CS_EI_CIRC);
    CardRegion_SetPlacement(pDeck, CS_XJUST_RIGHT, CS_YJUST_BOTTOM, (-(X_BORDER as i32)) as INT, (-(Y_BORDER as i32)) as INT);
    CardRegion_SetDragRule(pDeck, CS_DRAG_NONE, None);
    CardRegion_SetDropRule(pDeck, CS_DROP_NONE, None);
    CardRegion_SetClickReleaseProc(pDeck, Some(DeckClickProc));

    /* Create the row stacks */
    i = 0;
    while i < (NUM_STACKS as i32) {
        pStack[i as usize] = CardWindow_CreateRegion(SpiderWnd, (NUM_STACKS + (i) as u32) as INT, (true) as i32, 0, (Y_BORDER) as INT, 0, yRowStackCardOffset);
        CardRegion_SetFaceDirection(pStack[i as usize], CS_FACE_DOWN, 0);
        pos = i - (NUM_STACKS / 2) as i32;
        CardRegion_SetPlacement(pStack[i as usize], CS_XJUST_CENTER, 0,
            pos * (CardLib_CardWidth() + (X_BORDER as i32)) + (6 * (X_BORDER + 1)) as i32 + 3, 0);
        CardRegion_SetEmptyImage(pStack[i as usize], CS_EI_SUNK);
        CardRegion_SetDragRule(pStack[i as usize], CS_DRAG_CALLBACK, Some(StackDragProc));
        CardRegion_SetDropRule(pStack[i as usize], CS_DROP_CALLBACK, Some(StackDropProc));
        CardRegion_SetAddCardProc(pStack[i as usize], Some(StackAddProc));
        i += 1;
    }
}
