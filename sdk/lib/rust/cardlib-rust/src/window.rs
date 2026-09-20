//! `CardWindow` — the table. Owns the HWND and the regions, routes the mouse,
//! and paints.
//!
//! ## The mouse model
//!
//! A press is not a drag. On `WM_LBUTTONDOWN` we fire the click callback and
//! only *arm* a drag; the drag actually begins on the first `WM_MOUSEMOVE`
//! while the button is held. That split is forced by the games: Solitaire's
//! row stacks are both draggable and clickable — a click turns the top
//! face-down card over — so committing to a drag on press would make it
//! impossible to click one.
//!
//! ## Painting
//!
//! Everything is composed into an offscreen buffer and blitted once, because a
//! seven-column fan repainted in place flickers badly. The dragged cards are
//! the exception: they are blitted straight to the screen over a saved
//! background, so a drag costs two small blits per mouse move instead of a
//! full recompose.

use core::ffi::c_int;

use crate::bitmaps::{self, TABLE_GREEN};
use crate::card::CardStack;
use crate::region::*;
use crate::sys::{self, COLORREF, HBITMAP, HDC, HWND, LPARAM, LRESULT, RECT, WPARAM};

pub const MAX_CARD_REGIONS: usize = 32;
/// Two is enough for every caller we have (one table per game) and keeps the
/// static footprint near 70 KB rather than a quarter of a megabyte.
pub const MAX_CARD_WINDOWS: usize = 2;

const FLASH_TIMER_BASE: usize = 0x1000;
const ZOOM_STEPS: c_int = 12;

#[repr(C)]
pub struct CardWindow {
    pub in_use: bool,
    pub hwnd: HWND,
    pub width: c_int,
    pub height: c_int,
    pub back_idx: c_int,
    pub bk_color: COLORREF,

    pub regions: [CardRegion; MAX_CARD_REGIONS],
    pub num_regions: usize,

    // offscreen compose buffer, sized to the client area
    buf_dc: HDC,
    buf_bmp: HBITMAP,
    buf_old: HBITMAP,
    buf_w: c_int,
    buf_h: c_int,

    // --- press / drag state -------------------------------------------
    /// Region index armed by a press, before any movement.
    press_region: isize,
    press_cards: c_int,
    press_x: c_int,
    press_y: c_int,
    press_draggable: bool,

    dragging: bool,
    drag_from: isize,
    drag_stack: CardStack,
    drag_dc: HDC,
    drag_bmp: HBITMAP,
    drag_old: HBITMAP,
    save_dc: HDC,
    save_bmp: HBITMAP,
    save_old: HBITMAP,
    drag_w: c_int,
    drag_h: c_int,
    /// Cursor position within the drag image.
    drag_off_x: c_int,
    drag_off_y: c_int,
    /// Where the drag image currently sits, in client coordinates.
    drag_x: c_int,
    drag_y: c_int,
}

impl CardWindow {
    pub const fn empty() -> Self {
        CardWindow {
            in_use: false,
            hwnd: core::ptr::null_mut(),
            width: 0,
            height: 0,
            back_idx: 53,
            bk_color: TABLE_GREEN,
            regions: [const { CardRegion::empty() }; MAX_CARD_REGIONS],
            num_regions: 0,
            buf_dc: core::ptr::null_mut(),
            buf_bmp: core::ptr::null_mut(),
            buf_old: core::ptr::null_mut(),
            buf_w: 0,
            buf_h: 0,
            press_region: -1,
            press_cards: 0,
            press_x: 0,
            press_y: 0,
            press_draggable: false,
            dragging: false,
            drag_from: -1,
            drag_stack: CardStack {
                cards: [crate::card::Card::new(0); crate::card::MAX_CARDSTACK_SIZE],
                num_cards: 0,
            },
            drag_dc: core::ptr::null_mut(),
            drag_bmp: core::ptr::null_mut(),
            drag_old: core::ptr::null_mut(),
            save_dc: core::ptr::null_mut(),
            save_bmp: core::ptr::null_mut(),
            save_old: core::ptr::null_mut(),
            drag_w: 0,
            drag_h: 0,
            drag_off_x: 0,
            drag_off_y: 0,
            drag_x: 0,
            drag_y: 0,
        }
    }
}

static mut WINDOWS: [CardWindow; MAX_CARD_WINDOWS] =
    [const { CardWindow::empty() }; MAX_CARD_WINDOWS];

static mut CLASS_REGISTERED: bool = false;
static mut ZOOM_SPEED_MS: c_int = 8;

const CLASS_NAME: [u16; 16] = [
    b'C' as u16, b'a' as u16, b'r' as u16, b'd' as u16, b'L' as u16, b'i' as u16, b'b' as u16,
    b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b'3' as u16,
    b'2' as u16, 0,
];

/// # Safety
/// Single-threaded: the UI thread owns all of this.
unsafe fn windows() -> &'static mut [CardWindow; MAX_CARD_WINDOWS] {
    &mut *core::ptr::addr_of_mut!(WINDOWS)
}

pub fn register_class() -> bool {
    unsafe {
        let reg = &mut *core::ptr::addr_of_mut!(CLASS_REGISTERED);
        if *reg {
            return true;
        }
        let wc = sys::WNDCLASSEXW {
            cbSize: core::mem::size_of::<sys::WNDCLASSEXW>() as u32,
            // CS_DBLCLKS or the double-click-to-foundation shortcut never
            // arrives; we would only ever see two separate presses.
            style: sys::CS_DBLCLKS,
            lpfnWndProc: Some(card_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: sys::GetModuleHandleW(core::ptr::null()),
            hIcon: core::ptr::null_mut(),
            hCursor: sys::LoadCursorW(core::ptr::null_mut(), sys::IDC_ARROW),
            hbrBackground: core::ptr::null_mut(),
            lpszMenuName: core::ptr::null(),
            lpszClassName: CLASS_NAME.as_ptr(),
            hIconSm: core::ptr::null_mut(),
        };
        *reg = sys::RegisterClassExW(&wc) != 0;
        *reg
    }
}

fn wnd_from_hwnd(hwnd: HWND) -> *mut CardWindow {
    unsafe { sys::GetWindowPtr(hwnd, sys::GWLP_USERDATA) as *mut CardWindow }
}

// ===========================================================================
//  Painting
// ===========================================================================

impl CardWindow {
    fn ensure_buffer(&mut self, hdc: HDC) {
        if self.buf_w == self.width && self.buf_h == self.height && !self.buf_dc.is_null() {
            return;
        }
        self.free_buffer();
        if self.width <= 0 || self.height <= 0 {
            return;
        }
        unsafe {
            self.buf_dc = sys::CreateCompatibleDC(hdc);
            self.buf_bmp = sys::CreateCompatibleBitmap(hdc, self.width, self.height);
            if self.buf_dc.is_null() || self.buf_bmp.is_null() {
                self.free_buffer();
                return;
            }
            self.buf_old = sys::SelectObject(self.buf_dc, self.buf_bmp);
            self.buf_w = self.width;
            self.buf_h = self.height;
        }
    }

    fn free_buffer(&mut self) {
        unsafe {
            if !self.buf_dc.is_null() {
                if !self.buf_old.is_null() {
                    sys::SelectObject(self.buf_dc, self.buf_old);
                    self.buf_old = core::ptr::null_mut();
                }
                sys::DeleteDC(self.buf_dc);
                self.buf_dc = core::ptr::null_mut();
            }
            if !self.buf_bmp.is_null() {
                sys::DeleteObject(self.buf_bmp);
                self.buf_bmp = core::ptr::null_mut();
            }
        }
        self.buf_w = 0;
        self.buf_h = 0;
    }

    fn compose(&mut self) {
        if self.buf_dc.is_null() {
            return;
        }
        unsafe {
            let rc = RECT {
                left: 0,
                top: 0,
                right: self.width,
                bottom: self.height,
            };
            let brush = sys::CreateSolidBrush(self.bk_color);
            sys::FillRect(self.buf_dc, &rc, brush);
            sys::DeleteObject(brush);

            for i in 0..self.num_regions {
                if self.regions[i].in_use {
                    self.regions[i].render(self.buf_dc, self.back_idx, self.bk_color);
                }
            }
        }
    }

    fn paint(&mut self, hdc: HDC) {
        self.ensure_buffer(hdc);
        if self.buf_dc.is_null() {
            return;
        }
        self.compose();
        unsafe {
            sys::BitBlt(
                hdc,
                0,
                0,
                self.width,
                self.height,
                self.buf_dc,
                0,
                0,
                sys::SRCCOPY,
            );
        }
        // A repaint wiped the dragged cards off the screen, and the saved
        // background under them is now stale. Re-take both.
        if self.dragging {
            self.recapture_and_blit_drag(hdc);
        }
    }

    pub fn redraw(&mut self) {
        if self.hwnd.is_null() {
            return;
        }
        unsafe {
            sys::InvalidateRect(self.hwnd, core::ptr::null(), 0);
            sys::UpdateWindow(self.hwnd);
        }
    }
}

// ===========================================================================
//  Drag plumbing
// ===========================================================================

impl CardWindow {
    fn free_drag_bitmaps(&mut self) {
        unsafe {
            if !self.drag_dc.is_null() {
                if !self.drag_old.is_null() {
                    sys::SelectObject(self.drag_dc, self.drag_old);
                    self.drag_old = core::ptr::null_mut();
                }
                sys::DeleteDC(self.drag_dc);
                self.drag_dc = core::ptr::null_mut();
            }
            if !self.drag_bmp.is_null() {
                sys::DeleteObject(self.drag_bmp);
                self.drag_bmp = core::ptr::null_mut();
            }
            if !self.save_dc.is_null() {
                if !self.save_old.is_null() {
                    sys::SelectObject(self.save_dc, self.save_old);
                    self.save_old = core::ptr::null_mut();
                }
                sys::DeleteDC(self.save_dc);
                self.save_dc = core::ptr::null_mut();
            }
            if !self.save_bmp.is_null() {
                sys::DeleteObject(self.save_bmp);
                self.save_bmp = core::ptr::null_mut();
            }
        }
    }

    /// Build the floating image of `self.drag_stack` as it would look fanned
    /// out with `xoff`/`yoff`, on top of whatever the table shows at
    /// (`at_x`, `at_y`). Starting from the real background means a diagonal
    /// fan's uncovered corners stay correct and the cards' rounded corners
    /// work for free.
    fn build_drag_image(&mut self, hdc: HDC, at_x: c_int, at_y: c_int, xoff: c_int, yoff: c_int) {
        let (cw, ch) = (bitmaps::card_width(), bitmaps::card_height());
        let n = self.drag_stack.len() as c_int;
        let last = (n - 1).max(0);
        self.drag_w = (last * xoff).abs() + cw;
        self.drag_h = (last * yoff).abs() + ch;

        unsafe {
            self.drag_dc = sys::CreateCompatibleDC(hdc);
            self.drag_bmp = sys::CreateCompatibleBitmap(hdc, self.drag_w, self.drag_h);
            self.drag_old = sys::SelectObject(self.drag_dc, self.drag_bmp);

            self.save_dc = sys::CreateCompatibleDC(hdc);
            self.save_bmp = sys::CreateCompatibleBitmap(hdc, self.drag_w, self.drag_h);
            self.save_old = sys::SelectObject(self.save_dc, self.save_bmp);

            // seed with the table underneath, then draw the fan on top
            sys::BitBlt(
                self.drag_dc,
                0,
                0,
                self.drag_w,
                self.drag_h,
                hdc,
                at_x,
                at_y,
                sys::SRCCOPY,
            );
            let base_x = if xoff < 0 { -last * xoff } else { 0 };
            let base_y = if yoff < 0 { -last * yoff } else { 0 };
            for k in 0..n {
                let public_index = (n - 1 - k) as usize;
                bitmaps::draw_card(
                    self.drag_dc,
                    base_x + k * xoff,
                    base_y + k * yoff,
                    self.drag_stack.get(public_index).idx,
                );
            }
        }
    }

    fn save_background(&mut self, hdc: HDC, x: c_int, y: c_int) {
        unsafe {
            sys::BitBlt(
                self.save_dc,
                0,
                0,
                self.drag_w,
                self.drag_h,
                hdc,
                x,
                y,
                sys::SRCCOPY,
            );
        }
    }

    fn restore_background(&mut self, hdc: HDC) {
        unsafe {
            sys::BitBlt(
                hdc,
                self.drag_x,
                self.drag_y,
                self.drag_w,
                self.drag_h,
                self.save_dc,
                0,
                0,
                sys::SRCCOPY,
            );
        }
    }

    fn blit_drag(&mut self, hdc: HDC) {
        unsafe {
            sys::BitBlt(
                hdc,
                self.drag_x,
                self.drag_y,
                self.drag_w,
                self.drag_h,
                self.drag_dc,
                0,
                0,
                sys::SRCCOPY,
            );
        }
    }

    fn recapture_and_blit_drag(&mut self, hdc: HDC) {
        let (x, y) = (self.drag_x, self.drag_y);
        self.save_background(hdc, x, y);
        self.blit_drag(hdc);
    }

    fn move_drag_to(&mut self, hdc: HDC, x: c_int, y: c_int) {
        if x == self.drag_x && y == self.drag_y {
            return;
        }
        self.restore_background(hdc);
        self.drag_x = x;
        self.drag_y = y;
        self.save_background(hdc, x, y);
        self.blit_drag(hdc);
    }
}

// ===========================================================================
//  Hit testing and drop targeting
// ===========================================================================

impl CardWindow {
    fn region_at(&self, x: c_int, y: c_int) -> isize {
        // Later regions are on top, so search backwards.
        for i in (0..self.num_regions).rev() {
            let r = &self.regions[i];
            if r.in_use && r.visible && r.contains(x, y) {
                return i as isize;
            }
        }
        -1
    }

    /// The region overlapping the dragged image most. Overlap area rather than
    /// the cursor position, because a player aiming a fan of cards at a column
    /// lines up the CARDS, not the pointer.
    fn best_drop_target(&self, rc: &RECT) -> isize {
        let mut best = -1isize;
        let mut best_area = 0i64;
        for i in 0..self.num_regions {
            let r = &self.regions[i];
            if !r.in_use || !r.visible {
                continue;
            }
            let b = r.bounds();
            let w = (rc.right.min(b.right) - rc.left.max(b.left)).max(0) as i64;
            let h = (rc.bottom.min(b.bottom) - rc.top.max(b.top)).max(0) as i64;
            let area = w * h;
            if area > best_area {
                best_area = area;
                best = i as isize;
            }
        }
        best
    }
}

// ===========================================================================
//  Input handling
// ===========================================================================

impl CardWindow {
    fn on_lbutton_down(&mut self, x: c_int, y: c_int) {
        self.press_region = -1;
        self.press_draggable = false;

        let idx = self.region_at(x, y);
        if idx < 0 {
            return;
        }
        let i = idx as usize;

        let n = self.regions[i].cards_under_point(x, y);
        let rule = self.regions[i].drag_rule;

        // Decide draggability BEFORE the click callback runs: a click handler
        // is allowed to deal cards onto this very pile (Solitaire's stock does
        // exactly that) and we must not let it retroactively arm a drag.
        let mut draggable = match rule {
            CS_DRAG_NONE => false,
            CS_DRAG_TOP => n == 1,
            _ => n > 0,
        };
        if draggable && rule == CS_DRAG_CALLBACK {
            if let Some(cb) = self.regions[i].can_drag {
                draggable = unsafe { cb(&mut self.regions[i] as *mut CardRegion, n) != 0 };
            } else {
                draggable = false;
            }
        }

        self.press_region = idx;
        self.press_cards = n;
        self.press_x = x;
        self.press_y = y;
        self.press_draggable = draggable && n > 0;

        if let Some(cb) = self.regions[i].on_click {
            unsafe { cb(&mut self.regions[i] as *mut CardRegion, n) };
        }
    }

    fn begin_drag(&mut self, x: c_int, y: c_int) {
        let i = self.press_region as usize;
        let n = self.press_cards.min(self.regions[i].stack.len() as c_int);
        if n <= 0 {
            self.press_draggable = false;
            return;
        }

        // Where the fan being picked up currently sits.
        let a = self.regions[i].apparent_cards();
        let first_k = a - n;
        let (xoff, yoff) = (self.regions[i].xoffset, self.regions[i].yoffset);
        let (fan_x, fan_y) = self.regions[i].card_pos(first_k);
        let last = (n - 1).max(0);
        let img_x = fan_x + (last * xoff).min(0);
        let img_y = fan_y + (last * yoff).min(0);

        self.drag_stack = self.regions[i].stack.pop_n(n);
        self.regions[i].update_size();
        self.drag_from = self.press_region;
        self.dragging = true;

        // Repaint without the lifted cards, THEN capture what is underneath.
        self.redraw();

        unsafe {
            let hdc = sys::GetDC(self.hwnd);
            self.build_drag_image(hdc, img_x, img_y, xoff, yoff);
            self.drag_x = img_x;
            self.drag_y = img_y;
            self.drag_off_x = x - img_x;
            self.drag_off_y = y - img_y;
            self.save_background(hdc, img_x, img_y);
            self.blit_drag(hdc);
            sys::ReleaseDC(self.hwnd, hdc);
            sys::SetCapture(self.hwnd);
        }
    }

    fn on_mouse_move(&mut self, x: c_int, y: c_int) {
        if !self.dragging {
            if self.press_draggable && self.press_region >= 0 && (x != self.press_x || y != self.press_y)
            {
                self.begin_drag(self.press_x, self.press_y);
                if !self.dragging {
                    return;
                }
            } else {
                return;
            }
        }
        unsafe {
            let hdc = sys::GetDC(self.hwnd);
            let nx = x - self.drag_off_x;
            let ny = y - self.drag_off_y;
            self.move_drag_to(hdc, nx, ny);
            sys::ReleaseDC(self.hwnd, hdc);
        }
    }

    fn on_lbutton_up(&mut self, _x: c_int, _y: c_int) {
        if !self.dragging {
            let idx = self.press_region;
            self.press_region = -1;
            self.press_draggable = false;
            if idx >= 0 {
                let i = idx as usize;
                if let Some(cb) = self.regions[i].on_click_release {
                    let n = self.press_cards;
                    unsafe { cb(&mut self.regions[i] as *mut CardRegion, n) };
                }
            }
            return;
        }

        unsafe {
            let hdc = sys::GetDC(self.hwnd);
            self.restore_background(hdc);
            sys::ReleaseDC(self.hwnd, hdc);
            sys::ReleaseCapture();
        }
        self.free_drag_bitmaps();
        self.dragging = false;

        let rc = RECT {
            left: self.drag_x,
            top: self.drag_y,
            right: self.drag_x + self.drag_w,
            bottom: self.drag_y + self.drag_h,
        };
        // The target is chosen by how much of the dragged IMAGE covers each
        // pile, not by where the pointer happens to be: a player lining up a
        // fan of cards aims the cards.
        let target = self.best_drop_target(&rc);
        self.finish_drop(target);
    }

    /// Commit or bounce the cards currently in `drag_stack`.
    fn finish_drop(&mut self, target: isize) {
        let from = self.drag_from as usize;
        let n = self.drag_stack.len() as c_int;
        let mut accepted = false;

        if target >= 0 {
            let t = target as usize;
            let rule = self.regions[t].drop_rule;
            accepted = match rule {
                CS_DROP_ALL => true,
                CS_DROP_CALLBACK => match self.regions[t].can_drop {
                    // The callback may REWRITE drag_stack — Spider empties it
                    // when the drop completes a run it has just cleared by
                    // hand — so pass the live pile and commit what is left.
                    Some(cb) => unsafe {
                        cb(
                            &mut self.regions[t] as *mut CardRegion,
                            &mut self.drag_stack as *mut CardStack,
                        ) != 0
                    },
                    None => false,
                },
                _ => false,
            };
            if accepted {
                let dropped = self.drag_stack;
                self.regions[t].stack.push_stack(&dropped);
                self.regions[t].update_size();
                self.regions[from].update_size();
                if let Some(cb) = self.regions[t].on_add {
                    unsafe {
                        cb(
                            &mut self.regions[t] as *mut CardRegion,
                            &dropped as *const CardStack,
                        )
                    };
                }
                if from != t {
                    if let Some(cb) = self.regions[from].on_remove {
                        unsafe { cb(&mut self.regions[from] as *mut CardRegion, n) };
                    }
                }
            }
        }

        if !accepted {
            let bounced = self.drag_stack;
            self.regions[from].stack.push_stack(&bounced);
            self.regions[from].update_size();
        }

        self.drag_stack.clear();
        self.drag_from = -1;
        self.press_region = -1;
        self.press_draggable = false;
        self.redraw();
    }

    fn on_lbutton_dblclk(&mut self, x: c_int, y: c_int) {
        let idx = self.region_at(x, y);
        if idx < 0 {
            return;
        }
        let i = idx as usize;
        let n = self.regions[i].cards_under_point(x, y);
        if let Some(cb) = self.regions[i].on_dbl_click {
            unsafe { cb(&mut self.regions[i] as *mut CardRegion, n) };
        }
    }

    fn on_size(&mut self, w: c_int, h: c_int) {
        self.width = w;
        self.height = h;
        for i in 0..self.num_regions {
            if self.regions[i].in_use {
                self.regions[i].adjust_position(w, h);
            }
        }
        self.free_buffer();
    }

    fn on_timer(&mut self, id: usize) {
        if id < FLASH_TIMER_BASE {
            return;
        }
        let i = id - FLASH_TIMER_BASE;
        if i >= self.num_regions || !self.regions[i].in_use {
            return;
        }
        let r = &mut self.regions[i];
        r.flash_visible = !r.flash_visible;
        r.flash_count -= 1;
        let done = r.flash_count <= 0;
        if done {
            r.flash_visible = true;
            r.flash_count = 0;
            unsafe { sys::KillTimer(self.hwnd, id) };
            self.regions[i].flash_timer = 0;
        }
        self.redraw();
    }
}

// ===========================================================================
//  Animation
// ===========================================================================

impl CardWindow {
    /// Slide the current drag image from where it is to (`to_x`, `to_y`).
    fn animate_to(&mut self, to_x: c_int, to_y: c_int) {
        let (from_x, from_y) = (self.drag_x, self.drag_y);
        let speed = unsafe { *core::ptr::addr_of!(ZOOM_SPEED_MS) }.max(0);
        unsafe {
            let hdc = sys::GetDC(self.hwnd);
            for step in 1..=ZOOM_STEPS {
                let nx = from_x + (to_x - from_x) * step / ZOOM_STEPS;
                let ny = from_y + (to_y - from_y) * step / ZOOM_STEPS;
                self.move_drag_to(hdc, nx, ny);
                if speed > 0 {
                    sys::Sleep(speed as u32);
                }
            }
            self.restore_background(hdc);
            sys::ReleaseDC(self.hwnd, hdc);
        }
    }

    /// Where the next card placed on region `t` would land.
    fn landing_spot(&self, t: usize) -> (c_int, c_int) {
        let r = &self.regions[t];
        let a = r.apparent_cards();
        r.card_pos(a)
    }

    fn move_cards(
        &mut self,
        from: usize,
        to: usize,
        n: c_int,
        animate: bool,
        simulate: bool,
    ) -> bool {
        let n = n.min(self.regions[from].stack.len() as c_int);
        if n <= 0 || from >= self.num_regions || to >= self.num_regions {
            return false;
        }

        let a = self.regions[from].apparent_cards();
        let first_k = a - n;
        let (xoff, yoff) = (self.regions[from].xoffset, self.regions[from].yoffset);
        let (fan_x, fan_y) = self.regions[from].card_pos(first_k);
        let last = (n - 1).max(0);
        let img_x = fan_x + (last * xoff).min(0);
        let img_y = fan_y + (last * yoff).min(0);

        self.drag_stack = self.regions[from].stack.pop_n(n);
        self.regions[from].update_size();
        self.drag_from = from as isize;

        if animate && !self.hwnd.is_null() {
            self.redraw();
            unsafe {
                let hdc = sys::GetDC(self.hwnd);
                self.build_drag_image(hdc, img_x, img_y, xoff, yoff);
                self.drag_x = img_x;
                self.drag_y = img_y;
                self.save_background(hdc, img_x, img_y);
                self.blit_drag(hdc);
                sys::ReleaseDC(self.hwnd, hdc);
            }
            let (tx, ty) = self.landing_spot(to);
            self.animate_to(tx, ty);
            self.free_drag_bitmaps();
        }

        if simulate {
            // Run it through the real drop path so the destination's rule is
            // consulted and its add callback fires. Solitaire's
            // double-click-to-foundation depends on that callback to notice
            // the game has been won.
            self.finish_drop(to as isize);
            true
        } else {
            let moved = self.drag_stack;
            self.regions[to].stack.push_stack(&moved);
            self.regions[to].update_size();
            self.drag_stack.clear();
            self.drag_from = -1;
            self.redraw();
            true
        }
    }
}

// ===========================================================================
//  Window procedure
// ===========================================================================

unsafe extern "system" fn card_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    let pw = wnd_from_hwnd(hwnd);
    if pw.is_null() {
        return sys::DefWindowProcW(hwnd, msg, wp, lp);
    }
    let w = &mut *pw;

    match msg {
        sys::WM_ERASEBKGND => 1, // we paint every pixel ourselves
        sys::WM_PAINT => {
            let mut ps: sys::PAINTSTRUCT = core::mem::zeroed();
            let hdc = sys::BeginPaint(hwnd, &mut ps);
            w.paint(hdc);
            sys::EndPaint(hwnd, &ps);
            0
        }
        sys::WM_SIZE => {
            w.on_size(sys::LOWORD_SIGNED(lp), sys::HIWORD_SIGNED(lp));
            0
        }
        sys::WM_LBUTTONDOWN => {
            w.on_lbutton_down(sys::LOWORD_SIGNED(lp), sys::HIWORD_SIGNED(lp));
            0
        }
        sys::WM_MOUSEMOVE => {
            w.on_mouse_move(sys::LOWORD_SIGNED(lp), sys::HIWORD_SIGNED(lp));
            0
        }
        sys::WM_LBUTTONUP => {
            w.on_lbutton_up(sys::LOWORD_SIGNED(lp), sys::HIWORD_SIGNED(lp));
            0
        }
        sys::WM_LBUTTONDBLCLK => {
            w.on_lbutton_dblclk(sys::LOWORD_SIGNED(lp), sys::HIWORD_SIGNED(lp));
            0
        }
        sys::WM_TIMER => {
            w.on_timer(wp);
            0
        }
        _ => sys::DefWindowProcW(hwnd, msg, wp, lp),
    }
}

// ===========================================================================
//  C ABI — library lifetime
// ===========================================================================

#[no_mangle]
pub extern "C" fn CardLib_Init() -> c_int {
    if !bitmaps::init() {
        return 0;
    }
    if !register_class() {
        return 0;
    }
    1
}

#[no_mangle]
pub extern "C" fn CardLib_Term() {
    bitmaps::term();
}

#[no_mangle]
pub extern "C" fn CardLib_CardWidth() -> c_int {
    bitmaps::card_width()
}

#[no_mangle]
pub extern "C" fn CardLib_CardHeight() -> c_int {
    bitmaps::card_height()
}

#[no_mangle]
pub extern "C" fn CardLib_CardBitmapsDC() -> HDC {
    bitmaps::strip_dc()
}

#[no_mangle]
pub extern "C" fn CardLib_SetZoomSpeed(speed: c_int) {
    unsafe { *core::ptr::addr_of_mut!(ZOOM_SPEED_MS) = speed.max(0) };
}

// ===========================================================================
//  C ABI — CardWindow
// ===========================================================================

#[no_mangle]
pub extern "C" fn CardWindow_Create(
    parent: HWND,
    ex_style: u32,
    style: u32,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
) -> *mut CardWindow {
    if !register_class() {
        return core::ptr::null_mut();
    }
    unsafe {
        let all = windows();
        let slot = match all.iter().position(|w| !w.in_use) {
            Some(s) => s,
            None => return core::ptr::null_mut(),
        };
        all[slot] = CardWindow::empty();
        all[slot].in_use = true;
        all[slot].width = width;
        all[slot].height = height;

        let pw = &mut all[slot] as *mut CardWindow;
        let hwnd = sys::CreateWindowExW(
            ex_style,
            CLASS_NAME.as_ptr(),
            core::ptr::null(),
            style,
            x,
            y,
            width,
            height,
            parent,
            core::ptr::null_mut(),
            sys::GetModuleHandleW(core::ptr::null()),
            core::ptr::null_mut(),
        );
        if hwnd.is_null() {
            all[slot].in_use = false;
            return core::ptr::null_mut();
        }
        sys::SetWindowPtr(hwnd, sys::GWLP_USERDATA, pw as isize);
        all[slot].hwnd = hwnd;
        pw
    }
}

/// # Safety
/// `pw` must come from `CardWindow_Create`.
#[no_mangle]
pub unsafe extern "C" fn CardWindow_Destroy(pw: *mut CardWindow) {
    if let Some(w) = pw.as_mut() {
        w.free_buffer();
        w.free_drag_bitmaps();
        if !w.hwnd.is_null() {
            sys::DestroyWindow(w.hwnd);
            w.hwnd = core::ptr::null_mut();
        }
        w.in_use = false;
        w.num_regions = 0;
    }
}

/// # Safety
/// `pw` must come from `CardWindow_Create`.
#[no_mangle]
pub unsafe extern "C" fn CardWindow_GetHwnd(pw: *mut CardWindow) -> HWND {
    match pw.as_ref() {
        Some(w) => w.hwnd,
        None => core::ptr::null_mut(),
    }
}

/// # Safety
/// `pw` must come from `CardWindow_Create`.
#[no_mangle]
pub unsafe extern "C" fn CardWindow_CreateRegion(
    pw: *mut CardWindow,
    id: c_int,
    visible: c_int,
    x: c_int,
    y: c_int,
    xoffset: c_int,
    yoffset: c_int,
) -> *mut CardRegion {
    let w = match pw.as_mut() {
        Some(w) => w,
        None => return core::ptr::null_mut(),
    };
    if w.num_regions >= MAX_CARD_REGIONS {
        return core::ptr::null_mut();
    }
    let i = w.num_regions;
    w.regions[i] = CardRegion::empty();
    let r = &mut w.regions[i];
    r.wnd = pw;
    r.in_use = true;
    r.index = i;
    r.id = id;
    r.visible = visible != 0;
    r.base_x = x;
    r.base_y = y;
    r.x = x;
    r.y = y;
    r.xoffset = xoffset;
    r.yoffset = yoffset;
    r.update_size();
    w.num_regions += 1;
    &mut w.regions[i] as *mut CardRegion
}

/// # Safety
/// `pw` must come from `CardWindow_Create`.
#[no_mangle]
pub unsafe extern "C" fn CardWindow_EmptyStacks(pw: *mut CardWindow) {
    if let Some(w) = pw.as_mut() {
        for i in 0..w.num_regions {
            w.regions[i].stack.clear();
            w.regions[i].update_size();
        }
    }
}

/// # Safety
/// `pw` must come from `CardWindow_Create`.
#[no_mangle]
pub unsafe extern "C" fn CardWindow_Redraw(pw: *mut CardWindow) {
    if let Some(w) = pw.as_mut() {
        w.redraw();
    }
}

/// # Safety
/// `pw` must come from `CardWindow_Create`.
#[no_mangle]
pub unsafe extern "C" fn CardWindow_SetBackCardIdx(pw: *mut CardWindow, back: u32) {
    if let Some(w) = pw.as_mut() {
        w.back_idx = back as c_int;
    }
}

/// # Safety
/// `pw` must come from `CardWindow_Create`.
#[no_mangle]
pub unsafe extern "C" fn CardWindow_GetBackCardIdx(pw: *mut CardWindow) -> u32 {
    match pw.as_ref() {
        Some(w) => w.back_idx as u32,
        None => 0,
    }
}

pub fn flash_timer_id(region_index: usize) -> usize {
    FLASH_TIMER_BASE + region_index
}

/// Shared body of `CardRegion_MoveCard` / `CardRegion_SimulateDrag`.
///
/// # Safety
/// Both pointers must be regions of the same window.
pub unsafe fn region_move(
    from: *mut CardRegion,
    to: *mut CardRegion,
    num: c_int,
    animate: bool,
    simulate: bool,
) -> bool {
    let (fr, tr) = match (from.as_mut(), to.as_mut()) {
        (Some(a), Some(b)) => (a, b),
        _ => return false,
    };
    if fr.wnd.is_null() || fr.wnd != tr.wnd {
        return false;
    }
    let (fi, ti) = (fr.index, tr.index);
    let w = &mut *fr.wnd;
    w.move_cards(fi, ti, num, animate, simulate)
}
