//! `CARD` and `CARDSTACK` — the value types from `cardlib.h`.
//!
//! Storage runs BOTTOM-FIRST (`cards[0]` is the bottom of the pile) because
//! that makes push and pop O(1). The public index is the other way up —
//! index 0 is the TOP, the card a player takes next — so every public accessor
//! flips through [`CardStack::slot`]. Getting that backwards is silent: the
//! games compare `stack[0]` against the card being dropped, so a flipped index
//! makes every drop rule consult the wrong end of the pile and the game merely
//! plays wrong rather than crashing.

use core::ffi::c_int;

use crate::rng;

pub const MAX_CARDSTACK_SIZE: usize = 128;

/// Mirrors `CARD` in cardlib.h: a 0-51 index plus a face-up flag, two plain
/// ints so it passes in registers.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub idx: c_int,
    pub face_up: c_int,
}

impl Card {
    pub const fn new(idx: c_int) -> Self {
        Card { idx, face_up: 1 }
    }

    /// `(value - 1) * 4 + suit`, value-major: 0..3 are the aces.
    ///
    /// The clamping is part of the contract, not defensiveness — Spider builds
    /// its 1/2/4-suit decks by walking a counter in strides of 4, 2 or 1 and
    /// relies on out-of-range values folding rather than trapping.
    pub fn make(value: c_int, suit: c_int) -> Self {
        let mut v = value;
        if v < 1 {
            v = 1;
        }
        if v == 14 {
            v = 1;
        }
        if v > 13 {
            v = 13;
        }
        let s = suit.clamp(0, 3);
        Card::new((v - 1) * 4 + s)
    }

    pub fn from_index(index: c_int) -> Self {
        Card::new(index.clamp(0, 51))
    }

    pub fn suit(self) -> c_int {
        self.idx % 4
    }

    /// Ace low: 1..13.
    pub fn lo_val(self) -> c_int {
        self.idx / 4 + 1
    }

    /// Ace high: an ace reads 14, everything else reads its face value.
    pub fn hi_val(self) -> c_int {
        if self.idx < 4 {
            14
        } else {
            self.idx / 4 + 1
        }
    }

    /// Clubs and spades. Matches `eSuit`, where 0 = clubs and 3 = spades.
    pub fn is_black(self) -> bool {
        let s = self.suit();
        s == 0 || s == 3
    }

    pub fn is_red(self) -> bool {
        !self.is_black()
    }
}

impl Default for Card {
    fn default() -> Self {
        Card::new(0)
    }
}

/// Mirrors `CARDSTACK` in cardlib.h. Copied by value all over the games, which
/// is why it is a fixed array and not a slice.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CardStack {
    pub cards: [Card; MAX_CARDSTACK_SIZE],
    pub num_cards: c_int,
}

impl Default for CardStack {
    fn default() -> Self {
        CardStack {
            cards: [Card::new(0); MAX_CARDSTACK_SIZE],
            num_cards: 0,
        }
    }
}

impl CardStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        // Clamp rather than trust: the struct is public and callers memset it.
        (self.num_cards.max(0) as usize).min(MAX_CARDSTACK_SIZE)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Public index (0 = top) to storage index (0 = bottom).
    fn slot(&self, index: usize) -> Option<usize> {
        let n = self.len();
        if index >= n {
            None
        } else {
            Some(n - 1 - index)
        }
    }

    /// Card at public index `index`, 0 being the top. Out of range reads the
    /// ace of clubs, which is what a C caller indexing an empty pile got.
    pub fn get(&self, index: usize) -> Card {
        match self.slot(index) {
            Some(s) => self.cards[s],
            None => Card::new(0),
        }
    }

    pub fn set_face_up(&mut self, index: usize, face_up: bool) {
        if let Some(s) = self.slot(index) {
            self.cards[s].face_up = face_up as c_int;
        }
    }

    pub fn clear(&mut self) {
        self.num_cards = 0;
    }

    /// 52 cards, ace of clubs at the bottom through king of spades on top.
    pub fn new_deck(&mut self) {
        for i in 0..52 {
            self.cards[i] = Card::new(i as c_int);
        }
        self.num_cards = 52;
    }

    pub fn shuffle(&mut self) {
        let n = self.len();
        if n < 2 {
            return;
        }
        // Fisher-Yates downward, so every permutation is equally likely.
        for i in (1..n).rev() {
            let j = (rng::next() as usize) % (i + 1);
            self.cards.swap(i, j);
        }
    }

    pub fn reverse(&mut self) {
        let n = self.len();
        self.cards[..n].reverse();
    }

    pub fn push_card(&mut self, card: Card) {
        let n = self.len();
        if n < MAX_CARDSTACK_SIZE {
            self.cards[n] = card;
            self.num_cards = (n + 1) as c_int;
        }
    }

    /// Put `other` on top of self, keeping its order — so `other`'s top card
    /// ends up as self's top card.
    pub fn push_stack(&mut self, other: &CardStack) {
        let mut n = self.len();
        let m = other.len();
        for i in 0..m {
            if n >= MAX_CARDSTACK_SIZE {
                break;
            }
            self.cards[n] = other.cards[i];
            n += 1;
        }
        self.num_cards = n as c_int;
    }

    /// Remove and return the top card. An empty pile yields the ace of clubs,
    /// matching what the games already tolerate from an over-pop.
    pub fn pop_card(&mut self) -> Card {
        let n = self.len();
        if n == 0 {
            return Card::new(0);
        }
        self.num_cards = (n - 1) as c_int;
        self.cards[n - 1]
    }

    /// Remove the top `items` cards. The result keeps the source's order, so
    /// what was on top is still on top of the result.
    pub fn pop_n(&mut self, items: c_int) -> CardStack {
        let out = self.top_n(items);
        let n = self.len();
        self.num_cards = (n - out.len()) as c_int;
        out
    }

    pub fn top_card(&self) -> Card {
        self.get(0)
    }

    /// The top `items` cards without removing them.
    pub fn top_n(&self, items: c_int) -> CardStack {
        let n = self.len();
        let k = (items.max(0) as usize).min(n);
        let mut out = CardStack::new();
        out.cards[..k].copy_from_slice(&self.cards[n - k..n]);
        out.num_cards = k as c_int;
        out
    }

    /// Remove the card at public index `index` (0 = top) and return it.
    pub fn remove_card(&mut self, index: usize) -> Card {
        match self.slot(index) {
            None => Card::new(0),
            Some(s) => {
                let n = self.len();
                let card = self.cards[s];
                self.cards.copy_within(s + 1..n, s);
                self.num_cards = (n - 1) as c_int;
                card
            }
        }
    }
}

// ===========================================================================
//  C ABI
// ===========================================================================

#[no_mangle]
pub extern "C" fn Card_FromIndex(index: c_int) -> Card {
    Card::from_index(index)
}

#[no_mangle]
pub extern "C" fn Card_Make(value: c_int, suit: c_int) -> Card {
    Card::make(value, suit)
}

#[no_mangle]
pub extern "C" fn Card_Suit(card: Card) -> c_int {
    card.suit()
}

#[no_mangle]
pub extern "C" fn Card_LoVal(card: Card) -> c_int {
    card.lo_val()
}

#[no_mangle]
pub extern "C" fn Card_HiVal(card: Card) -> c_int {
    card.hi_val()
}

#[no_mangle]
pub extern "C" fn Card_Idx(card: Card) -> c_int {
    card.idx
}

#[no_mangle]
pub extern "C" fn Card_IsRed(card: Card) -> c_int {
    card.is_red() as c_int
}

#[no_mangle]
pub extern "C" fn Card_IsBlack(card: Card) -> c_int {
    card.is_black() as c_int
}

/// # Safety
/// `stack` must point to a valid `CARDSTACK`.
#[no_mangle]
pub unsafe extern "C" fn CardStack_Clear(stack: *mut CardStack) {
    if let Some(s) = stack.as_mut() {
        s.clear();
    }
}

/// # Safety
/// `stack` must point to a valid `CARDSTACK`.
#[no_mangle]
pub unsafe extern "C" fn CardStack_NewDeck(stack: *mut CardStack) {
    if let Some(s) = stack.as_mut() {
        s.new_deck();
    }
}

/// # Safety
/// `stack` must point to a valid `CARDSTACK`.
#[no_mangle]
pub unsafe extern "C" fn CardStack_Shuffle(stack: *mut CardStack) {
    if let Some(s) = stack.as_mut() {
        s.shuffle();
    }
}

/// # Safety
/// `stack` must point to a valid `CARDSTACK`.
#[no_mangle]
pub unsafe extern "C" fn CardStack_Reverse(stack: *mut CardStack) {
    if let Some(s) = stack.as_mut() {
        s.reverse();
    }
}

/// # Safety
/// `stack` must point to a valid `CARDSTACK`.
#[no_mangle]
pub unsafe extern "C" fn CardStack_NumCards(stack: *const CardStack) -> c_int {
    match stack.as_ref() {
        Some(s) => s.len() as c_int,
        None => 0,
    }
}

/// # Safety
/// `stack` must point to a valid `CARDSTACK`.
#[no_mangle]
pub unsafe extern "C" fn CardStack_PushCard(stack: *mut CardStack, card: Card) {
    if let Some(s) = stack.as_mut() {
        s.push_card(card);
    }
}

/// # Safety
/// Both pointers must be valid `CARDSTACK`s.
#[no_mangle]
pub unsafe extern "C" fn CardStack_PushStack(stack: *mut CardStack, push: *const CardStack) {
    if let (Some(s), Some(p)) = (stack.as_mut(), push.as_ref()) {
        // Copy first: a caller may legitimately push a pile onto itself.
        let copy = *p;
        s.push_stack(&copy);
    }
}

/// # Safety
/// `stack` must point to a valid `CARDSTACK`.
#[no_mangle]
pub unsafe extern "C" fn CardStack_PopCard(stack: *mut CardStack) -> Card {
    match stack.as_mut() {
        Some(s) => s.pop_card(),
        None => Card::new(0),
    }
}

/// # Safety
/// `stack` and `out` must point to valid `CARDSTACK`s.
#[no_mangle]
pub unsafe extern "C" fn CardStack_PopN(stack: *mut CardStack, items: c_int, out: *mut CardStack) {
    let popped = match stack.as_mut() {
        Some(s) => s.pop_n(items),
        None => CardStack::new(),
    };
    if let Some(o) = out.as_mut() {
        *o = popped;
    }
}

/// # Safety
/// `stack` must point to a valid `CARDSTACK`.
#[no_mangle]
pub unsafe extern "C" fn CardStack_TopCard(stack: *const CardStack) -> Card {
    match stack.as_ref() {
        Some(s) => s.top_card(),
        None => Card::new(0),
    }
}

/// # Safety
/// `stack` and `out` must point to valid `CARDSTACK`s.
#[no_mangle]
pub unsafe extern "C" fn CardStack_TopN(
    stack: *const CardStack,
    items: c_int,
    out: *mut CardStack,
) {
    let top = match stack.as_ref() {
        Some(s) => s.top_n(items),
        None => CardStack::new(),
    };
    if let Some(o) = out.as_mut() {
        *o = top;
    }
}

/// # Safety
/// `stack` must point to a valid `CARDSTACK`.
#[no_mangle]
pub unsafe extern "C" fn CardStack_Get(stack: *const CardStack, index: c_int) -> Card {
    match stack.as_ref() {
        Some(s) if index >= 0 => s.get(index as usize),
        _ => Card::new(0),
    }
}

/// # Safety
/// `stack` must point to a valid `CARDSTACK`.
#[no_mangle]
pub unsafe extern "C" fn CardStack_RemoveCard(stack: *mut CardStack, index: c_int) -> Card {
    match stack.as_mut() {
        Some(s) if index >= 0 => s.remove_card(index as usize),
        _ => Card::new(0),
    }
}
