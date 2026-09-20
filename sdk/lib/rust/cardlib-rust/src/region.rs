//! `CardRegion` — one pile on the table: where it sits, how it fans out, which
//! of its cards show their face, and what the game is allowed to do with it.
//!
//! A region never owns a window; it is stored inside its [`CardWindow`] and
//! keeps a raw pointer back to it. The array is `static`, so those pointers
//! stay valid for the life of the process.

use core::ffi::c_int;

use crate::bitmaps::{self, draw_card};
use crate::card::{Card, CardStack};
use crate::color::color_darker;
use crate::sys::{self, COLORREF, HDC, RECT};
use crate::window::CardWindow;

// Drag rules
pub const CS_DRAG_NONE: u32 = 0;
pub const CS_DRAG_TOP: u32 = 1;
pub const CS_DRAG_ALL: u32 = 2;
pub const CS_DRAG_CALLBACK: u32 = 3;

// Drop rules
pub const CS_DROP_NONE: u32 = 0;
pub const CS_DROP_ALL: u32 = 1;
pub const CS_DROP_CALLBACK: u32 = 2;

// Justification
pub const CS_XJUST_NONE: u32 = 0;
pub const CS_XJUST_RIGHT: u32 = 1;
pub const CS_XJUST_CENTER: u32 = 2;
pub const CS_YJUST_NONE: u32 = 0;
pub const CS_YJUST_BOTTOM: u32 = 1;
pub const CS_YJUST_CENTER: u32 = 2;

// Face direction
pub const CS_FACE_UP: u32 = 0;
pub const CS_FACE_DOWN: u32 = 1;
pub const CS_FACE_DOWNUP: u32 = 2;
pub const CS_FACE_UPDOWN: u32 = 3;
pub const CS_FACE_ANY: u32 = 4;

// Empty-pile artwork
pub const CS_EI_NONE: u32 = 0;
pub const CS_EI_SUNK: u32 = 1;

pub type PfnCanDrag = Option<unsafe extern "system" fn(*mut CardRegion, c_int) -> c_int>;
pub type PfnCanDrop = Option<unsafe extern "system" fn(*mut CardRegion, *mut CardStack) -> c_int>;
pub type PfnClick = Option<unsafe extern "system" fn(*mut CardRegion, c_int)>;
pub type PfnAdd = Option<unsafe extern "system" fn(*mut CardRegion, *const CardStack)>;
pub type PfnRemove = Option<unsafe extern "system" fn(*mut CardRegion, c_int)>;

#[repr(C)]
pub struct CardRegion {
    pub wnd: *mut CardWindow,
    pub in_use: bool,
    /// Our own slot in the window's region array — MoveCard and SimulateDrag
    /// hand it back to the window, which owns the drag machinery.
    pub index: usize,
    pub id: c_int,
    pub visible: bool,

    /// Where the region was asked to sit, before justification.
    pub base_x: c_int,
    pub base_y: c_int,
    /// Where it actually sits now.
    pub x: c_int,
    pub y: c_int,

    pub xoffset: c_int,
    pub yoffset: c_int,

    /// Bounding box of the fan, recomputed by [`CardRegion::update_size`].
    pub width: c_int,
    pub height: c_int,

    pub xjustify: u32,
    pub yjustify: u32,
    pub xadjust: c_int,
    pub yadjust: c_int,

    /// How many real cards one drawn card stands for. 1 means "draw them
    /// all"; Solitaire's stock uses 6 so a 24-card deck looks like a slab
    /// four cards thick instead of a 24-card staircase.
    pub threed: c_int,

    pub face_dir: u32,
    pub face_option: c_int,

    pub empty_image: u32,
    /// -1 means "whatever the window says".
    pub back_idx: c_int,

    pub drag_rule: u32,
    pub drop_rule: u32,

    pub can_drag: PfnCanDrag,
    pub can_drop: PfnCanDrop,
    pub on_click: PfnClick,
    pub on_click_release: PfnClick,
    pub on_dbl_click: PfnClick,
    pub on_add: PfnAdd,
    pub on_remove: PfnRemove,

    pub stack: CardStack,

    pub flash_count: c_int,
    pub flash_visible: bool,
    pub flash_timer: usize,
}

impl CardRegion {
    pub const fn empty() -> Self {
        CardRegion {
            wnd: core::ptr::null_mut(),
            in_use: false,
            index: 0,
            id: 0,
            visible: false,
            base_x: 0,
            base_y: 0,
            x: 0,
            y: 0,
            xoffset: 0,
            yoffset: 0,
            width: 0,
            height: 0,
            xjustify: CS_XJUST_NONE,
            yjustify: CS_YJUST_NONE,
            xadjust: 0,
            yadjust: 0,
            threed: 1,
            face_dir: CS_FACE_UP,
            face_option: 0,
            empty_image: CS_EI_NONE,
            back_idx: -1,
            drag_rule: CS_DRAG_NONE,
            drop_rule: CS_DROP_NONE,
            can_drag: None,
            can_drop: None,
            on_click: None,
            on_click_release: None,
            on_dbl_click: None,
            on_add: None,
            on_remove: None,
            stack: CardStack {
                cards: [Card::new(0); crate::card::MAX_CARDSTACK_SIZE],
                num_cards: 0,
            },
            flash_count: 0,
            flash_visible: true,
            flash_timer: 0,
        }
    }

    /// How many card images this pile draws.
    pub fn apparent_cards(&self) -> c_int {
        let n = self.stack.len() as c_int;
        if n == 0 {
            return 0;
        }
        let t = self.threed.max(1);
        // Round up, so a pile never draws fewer than one card.
        ((n + t - 1) / t).max(1)
    }

    /// Top-left of the k'th drawn card, k = 0 being the bottom of the fan.
    pub fn card_pos(&self, k: c_int) -> (c_int, c_int) {
        (self.x + k * self.xoffset, self.y + k * self.yoffset)
    }

    /// Recompute the bounding box. Offsets may be negative — Spider's stock
    /// fans leftwards — so the box can extend on either side of (x, y).
    pub fn update_size(&mut self) {
        let (cw, ch) = (bitmaps::card_width(), bitmaps::card_height());
        let a = self.apparent_cards();
        let last = if a > 0 { a - 1 } else { 0 };
        let dx = last * self.xoffset;
        let dy = last * self.yoffset;
        self.width = dx.abs() + cw;
        self.height = dy.abs() + ch;
    }

    /// Bounding box in client coordinates.
    pub fn bounds(&self) -> RECT {
        let (cw, ch) = (bitmaps::card_width(), bitmaps::card_height());
        let a = self.apparent_cards();
        let last = if a > 0 { a - 1 } else { 0 };
        let dx = last * self.xoffset;
        let dy = last * self.yoffset;
        RECT {
            left: self.x + dx.min(0),
            top: self.y + dy.min(0),
            right: self.x + dx.max(0) + cw,
            bottom: self.y + dy.max(0) + ch,
        }
    }

    /// Re-place the region for a window of this size.
    ///
    /// Justification positions the whole fan and `adjust` then displaces it,
    /// which is how Solitaire spreads seven centred columns: every one is
    /// `CS_XJUST_CENTER` with a different adjust.
    pub fn adjust_position(&mut self, win_w: c_int, win_h: c_int) {
        self.update_size();
        self.x = match self.xjustify {
            CS_XJUST_CENTER => (win_w - self.width) / 2 + self.xadjust,
            CS_XJUST_RIGHT => win_w - self.width + self.xadjust,
            _ => self.base_x,
        };
        self.y = match self.yjustify {
            CS_YJUST_CENTER => (win_h - self.height) / 2 + self.yadjust,
            CS_YJUST_BOTTOM => win_h - self.height + self.yadjust,
            _ => self.base_y,
        };
    }

    /// Should the card at public index `i` show its face?
    ///
    /// "Bottom N" counts from the bottom of the pile, which is the HIGH end of
    /// the public index, so the comparisons look inverted and are not.
    pub fn is_face_up(&self, i: usize) -> bool {
        let n = self.stack.len();
        let opt = self.face_option.max(0) as usize;
        match self.face_dir {
            CS_FACE_DOWN => false,
            CS_FACE_DOWNUP => i < n.saturating_sub(opt),
            CS_FACE_UPDOWN => i >= n.saturating_sub(opt),
            CS_FACE_ANY => self.stack.get(i).face_up != 0,
            _ => true,
        }
    }

    /// Which strip slot draws the card at public index `i`.
    pub fn slot_for(&self, i: usize, window_back: c_int) -> c_int {
        if self.is_face_up(i) {
            self.stack.get(i).idx
        } else if self.back_idx >= 0 {
            self.back_idx
        } else {
            window_back
        }
    }

    pub fn render(&self, hdc: HDC, window_back: c_int, bk_color: COLORREF) {
        if !self.visible || (self.flash_count > 0 && !self.flash_visible) {
            return;
        }
        let a = self.apparent_cards();
        if a == 0 {
            self.render_empty(hdc, bk_color);
            return;
        }
        // Bottom of the fan first, so the top card lands on top.
        for k in 0..a {
            let (cx, cy) = self.card_pos(k);
            let public_index = (a - 1 - k) as usize;
            draw_card(hdc, cx, cy, self.slot_for(public_index, window_back));
        }
    }

    fn render_empty(&self, hdc: HDC, bk_color: COLORREF) {
        let (cw, ch) = (bitmaps::card_width(), bitmaps::card_height());
        match self.empty_image {
            CS_EI_NONE => {}
            CS_EI_SUNK => draw_sunken(hdc, self.x, self.y, cw, ch, bk_color),
            slot => draw_card(hdc, self.x, self.y, slot as c_int),
        }
    }

    pub fn contains(&self, px: c_int, py: c_int) -> bool {
        let r = self.bounds();
        px >= r.left && px < r.right && py >= r.top && py < r.bottom
    }

    /// How many cards a press at (px, py) would pick up: the card under the
    /// cursor plus everything resting on it. 0 means "nothing draggable here".
    pub fn cards_under_point(&self, px: c_int, py: c_int) -> c_int {
        let (cw, ch) = (bitmaps::card_width(), bitmaps::card_height());
        let a = self.apparent_cards();
        // Front to back: the visually topmost card that covers the point wins.
        for k in (0..a).rev() {
            let (cx, cy) = self.card_pos(k);
            if px >= cx && px < cx + cw && py >= cy && py < cy + ch {
                return a - k;
            }
        }
        0
    }
}

/// A recessed outline for an empty pile: two darker edges, corners left out so
/// it reads as rounded like the cards do.
pub fn draw_sunken(hdc: HDC, x: c_int, y: c_int, w: c_int, h: c_int, bk: COLORREF) {
    unsafe {
        let dark = color_darker(bk);
        let pen = sys::CreatePen(sys::PS_SOLID, 1, dark);
        let old_pen = sys::SelectObject(hdc, pen);

        // top and bottom, inset by one so the corners stay bare
        sys::MoveToEx(hdc, x + 1, y, core::ptr::null_mut());
        sys::LineTo(hdc, x + w - 1, y);
        sys::MoveToEx(hdc, x + 1, y + h - 1, core::ptr::null_mut());
        sys::LineTo(hdc, x + w - 1, y + h - 1);
        // left and right
        sys::MoveToEx(hdc, x, y + 1, core::ptr::null_mut());
        sys::LineTo(hdc, x, y + h - 1);
        sys::MoveToEx(hdc, x + w - 1, y + 1, core::ptr::null_mut());
        sys::LineTo(hdc, x + w - 1, y + h - 1);

        sys::SelectObject(hdc, old_pen);
        sys::DeleteObject(pen);
    }
}

// ===========================================================================
//  C ABI
// ===========================================================================

/// Repaint the window this region belongs to, if it has one yet.
unsafe fn redraw_parent(r: &mut CardRegion) {
    if let Some(w) = r.wnd.as_mut() {
        w.redraw();
    }
}

macro_rules! with_region {
    ($p:expr, $r:ident => $body:expr) => {
        if let Some($r) = $p.as_mut() {
            $body
        }
    };
    ($p:expr, $r:ident => $body:expr, $default:expr) => {
        match $p.as_mut() {
            Some($r) => $body,
            None => $default,
        }
    };
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_Id(p: *mut CardRegion) -> c_int {
    with_region!(p, r => r.id, 0)
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SetOffsets(p: *mut CardRegion, x: c_int, y: c_int) {
    with_region!(p, r => {
        r.xoffset = x;
        r.yoffset = y;
        r.update_size();
    })
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SetPlacement(
    p: *mut CardRegion,
    xjust: u32,
    yjust: u32,
    xadjust: c_int,
    yadjust: c_int,
) {
    with_region!(p, r => {
        r.xjustify = xjust;
        r.yjustify = yjust;
        r.xadjust = xadjust;
        r.yadjust = yadjust;
        if let Some(w) = r.wnd.as_ref() {
            let (ww, wh) = (w.width, w.height);
            r.adjust_position(ww, wh);
        }
    })
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SetEmptyImage(p: *mut CardRegion, image: u32) {
    with_region!(p, r => r.empty_image = image)
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SetThreedCount(p: *mut CardRegion, count: c_int) -> c_int {
    with_region!(p, r => {
        if count < 1 {
            return 0;
        }
        r.threed = count;
        r.update_size();
        1
    }, 0)
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SetFaceDirection(p: *mut CardRegion, dir: u32, option: c_int) {
    with_region!(p, r => {
        r.face_dir = dir;
        r.face_option = option;
    })
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`; `option` may be null.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_GetFaceDirection(
    p: *mut CardRegion,
    option: *mut c_int,
) -> u32 {
    with_region!(p, r => {
        if let Some(o) = option.as_mut() {
            *o = r.face_option;
        }
        r.face_dir
    }, {
        if let Some(o) = option.as_mut() {
            *o = 0;
        }
        CS_FACE_UP
    })
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SetDragRule(
    p: *mut CardRegion,
    rule: u32,
    pfn: PfnCanDrag,
) -> c_int {
    with_region!(p, r => {
        // A callback rule with no callback would silently make the pile
        // undraggable; say no instead.
        if rule == CS_DRAG_CALLBACK && pfn.is_none() {
            return 0;
        }
        r.drag_rule = rule;
        r.can_drag = pfn;
        1
    }, 0)
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SetDropRule(
    p: *mut CardRegion,
    rule: u32,
    pfn: PfnCanDrop,
) -> c_int {
    with_region!(p, r => {
        if rule == CS_DROP_CALLBACK && pfn.is_none() {
            return 0;
        }
        r.drop_rule = rule;
        r.can_drop = pfn;
        1
    }, 0)
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SetClickProc(p: *mut CardRegion, pfn: PfnClick) {
    with_region!(p, r => r.on_click = pfn)
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SetClickReleaseProc(p: *mut CardRegion, pfn: PfnClick) {
    with_region!(p, r => r.on_click_release = pfn)
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SetDblClickProc(p: *mut CardRegion, pfn: PfnClick) {
    with_region!(p, r => r.on_dbl_click = pfn)
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SetAddCardProc(p: *mut CardRegion, pfn: PfnAdd) {
    with_region!(p, r => r.on_add = pfn)
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SetRemoveCardProc(p: *mut CardRegion, pfn: PfnRemove) {
    with_region!(p, r => r.on_remove = pfn)
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`; `stack` must be valid.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SetCardStack(p: *mut CardRegion, stack: *const CardStack) {
    with_region!(p, r => {
        if let Some(s) = stack.as_ref() {
            r.stack = *s;
            r.update_size();
        }
    })
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`. The returned pointer is owned
/// by the region and is invalidated by the next call that changes it.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_GetCardStack(p: *mut CardRegion) -> *const CardStack {
    match p.as_ref() {
        Some(r) => &r.stack as *const CardStack,
        None => core::ptr::null(),
    }
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_NumCards(p: *mut CardRegion) -> c_int {
    with_region!(p, r => r.stack.len() as c_int, 0)
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_Clear(p: *mut CardRegion) {
    with_region!(p, r => {
        r.stack.clear();
        r.update_size();
    })
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_Reverse(p: *mut CardRegion) {
    with_region!(p, r => r.stack.reverse())
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_PushCard(p: *mut CardRegion, card: Card) {
    with_region!(p, r => {
        r.stack.push_card(card);
        r.update_size();
    })
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`; `push` must be valid.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_PushStack(p: *mut CardRegion, push: *const CardStack) {
    with_region!(p, r => {
        if let Some(s) = push.as_ref() {
            let copy = *s;
            r.stack.push_stack(&copy);
            r.update_size();
        }
    })
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_PopCard(p: *mut CardRegion) -> Card {
    with_region!(p, r => {
        let c = r.stack.pop_card();
        r.update_size();
        c
    }, Card::new(0))
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`; `out` must be valid.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_PopN(p: *mut CardRegion, items: c_int, out: *mut CardStack) {
    let popped = match p.as_mut() {
        Some(r) => {
            let s = r.stack.pop_n(items);
            r.update_size();
            s
        }
        None => CardStack::new(),
    };
    if let Some(o) = out.as_mut() {
        *o = popped;
    }
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`; `out` must be valid.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_TopN(p: *mut CardRegion, items: c_int, out: *mut CardStack) {
    let top = match p.as_ref() {
        Some(r) => r.stack.top_n(items),
        None => CardStack::new(),
    };
    if let Some(o) = out.as_mut() {
        *o = top;
    }
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_Update(p: *mut CardRegion) {
    with_region!(p, r => {
        r.update_size();
        if let Some(w) = r.wnd.as_ref() {
            let (ww, wh) = (w.width, w.height);
            r.adjust_position(ww, wh);
        }
        redraw_parent(r);
    })
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_Redraw(p: *mut CardRegion) {
    with_region!(p, r => redraw_parent(r))
}

/// # Safety
/// `p` must come from `CardWindow_CreateRegion`.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_Flash(p: *mut CardRegion, count: c_int, timeout: c_int) {
    with_region!(p, r => {
        if count <= 0 || timeout <= 0 {
            return;
        }
        let idx = r.index;
        r.flash_count = count;
        r.flash_visible = true;
        if let Some(w) = r.wnd.as_mut() {
            let id = crate::window::flash_timer_id(idx);
            r.flash_timer = id;
            sys::SetTimer(w.hwnd, id, timeout as u32, None);
        }
    })
}

/// # Safety
/// Both pointers must come from `CardWindow_CreateRegion`, on the same window.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_MoveCard(
    p: *mut CardRegion,
    dest: *mut CardRegion,
    num: c_int,
    animate: c_int,
) -> c_int {
    crate::window::region_move(p, dest, num, animate != 0, false) as c_int
}

/// # Safety
/// Both pointers must come from `CardWindow_CreateRegion`, on the same window.
#[no_mangle]
pub unsafe extern "C" fn CardRegion_SimulateDrag(
    p: *mut CardRegion,
    dest: *mut CardRegion,
    num: c_int,
    animate: c_int,
) -> c_int {
    crate::window::region_move(p, dest, num, animate != 0, true) as c_int
}
