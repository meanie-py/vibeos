//! Card artwork: loading it out of cards.dll and putting it on screen.
//!
//! The 68 bitmaps are composed once into a single wide "strip" DC, and
//! everything afterwards is a BitBlt out of it at `slot * card_width`. Slot
//! numbering is `resource id - 1`:
//!
//! | slot  | content                                        |
//! |-------|------------------------------------------------|
//! | 0..51 | the faces, addressed directly by `Card::idx()` |
//! | 52    | the free-cell placeholder (unused by us)       |
//! | 53..64| the twelve card backs (`ecbCROSSHATCH`..)      |
//! | 65    | the joker                                      |
//! | 66    | the red X placeholder (`CS_EI_X`)              |
//! | 67    | the green O placeholder (`CS_EI_CIRC`)         |
//!
//! The faces are stored in the DLL suit-major (clubs ace..king, then
//! diamonds...) while `Card::idx()` is value-major, hence the transposition
//! `(i % 4) * 13 + (i / 4)` when filling slots 0..51.

use core::ffi::c_int;

use crate::sys::{self, HBITMAP, HDC, HMODULE};

pub const NUM_CARD_BITMAPS: usize = 68;

/// Slots that are not faces. Public because the header exposes them.
pub const SLOT_X: c_int = 66;
pub const SLOT_CIRC: c_int = 67;

/// The table green the X and O placeholder artwork is painted on. Their
/// backgrounds are baked into the bitmaps, so a window with any other
/// background colour would show a green rectangle around them.
pub const TABLE_GREEN: sys::COLORREF = sys::RGB(0, 128, 0);

pub struct Artwork {
    pub strip_dc: HDC,
    pub strip_bmp: HBITMAP,
    pub old_bmp: HBITMAP,
    pub cards_dll: HMODULE,
    pub card_width: c_int,
    pub card_height: c_int,
}

static mut ARTWORK: Artwork = Artwork {
    strip_dc: core::ptr::null_mut(),
    strip_bmp: core::ptr::null_mut(),
    old_bmp: core::ptr::null_mut(),
    cards_dll: core::ptr::null_mut(),
    card_width: 0,
    card_height: 0,
};

/// # Safety
/// Single-threaded by construction: everything here runs on the UI thread.
pub unsafe fn artwork() -> &'static mut Artwork {
    &mut *core::ptr::addr_of_mut!(ARTWORK)
}

pub fn card_width() -> c_int {
    unsafe { artwork().card_width }
}

pub fn card_height() -> c_int {
    unsafe { artwork().card_height }
}

pub fn strip_dc() -> HDC {
    unsafe { artwork().strip_dc }
}

pub fn is_initialised() -> bool {
    !strip_dc().is_null()
}

const CARDS_DLL: [u16; 10] = [
    b'c' as u16, b'a' as u16, b'r' as u16, b'd' as u16, b's' as u16, b'.' as u16, b'd' as u16,
    b'l' as u16, b'l' as u16, 0,
];

/// Load cards.dll and compose the strip. Returns false and leaves nothing
/// behind if the artwork is missing — a caller that ignores that and carries
/// on would draw an empty window rather than crash, which is exactly the kind
/// of quiet degradation that wastes an afternoon, so the header says to stop.
pub fn init() -> bool {
    unsafe {
        let art = artwork();
        if !art.strip_dc.is_null() {
            return true;
        }

        art.cards_dll = sys::LoadLibraryW(CARDS_DLL.as_ptr());
        if art.cards_dll.is_null() {
            return false;
        }

        // Take the card size from the artwork itself rather than hardcoding
        // 72x112: the same DLL ships a 110x198 Bavarian deck, and every
        // layout number in the games is derived from these two values.
        let first = sys::LoadBitmapW(art.cards_dll, 1);
        if first.is_null() {
            sys::FreeLibrary(art.cards_dll);
            art.cards_dll = core::ptr::null_mut();
            return false;
        }
        let mut bm: sys::BITMAP = core::mem::zeroed();
        sys::GetObjectW(
            first,
            core::mem::size_of::<sys::BITMAP>() as c_int,
            core::ptr::addr_of_mut!(bm).cast(),
        );
        art.card_width = bm.bmWidth;
        art.card_height = bm.bmHeight;
        sys::DeleteObject(first);

        if art.card_width <= 0 || art.card_height <= 0 {
            sys::FreeLibrary(art.cards_dll);
            art.cards_dll = core::ptr::null_mut();
            return false;
        }

        let screen = sys::GetDC(core::ptr::null_mut());
        art.strip_dc = sys::CreateCompatibleDC(screen);
        art.strip_bmp = sys::CreateCompatibleBitmap(
            screen,
            art.card_width * NUM_CARD_BITMAPS as c_int,
            art.card_height,
        );
        sys::ReleaseDC(core::ptr::null_mut(), screen);

        if art.strip_dc.is_null() || art.strip_bmp.is_null() {
            term();
            return false;
        }
        art.old_bmp = sys::SelectObject(art.strip_dc, art.strip_bmp);

        let src = sys::CreateCompatibleDC(art.strip_dc);
        for slot in 0..NUM_CARD_BITMAPS {
            // Faces are suit-major in the DLL, value-major in Card::idx().
            let res_id = if slot < 52 {
                (slot % 4) * 13 + (slot / 4) + 1
            } else {
                slot + 1
            };
            let bmp = sys::LoadBitmapW(art.cards_dll, res_id);
            if bmp.is_null() {
                continue;
            }
            let old = sys::SelectObject(src, bmp);
            sys::BitBlt(
                art.strip_dc,
                slot as c_int * art.card_width,
                0,
                art.card_width,
                art.card_height,
                src,
                0,
                0,
                sys::SRCCOPY,
            );
            sys::SelectObject(src, old);
            sys::DeleteObject(bmp);
        }
        sys::DeleteDC(src);

        true
    }
}

pub fn term() {
    unsafe {
        let art = artwork();
        if !art.strip_dc.is_null() {
            if !art.old_bmp.is_null() {
                sys::SelectObject(art.strip_dc, art.old_bmp);
                art.old_bmp = core::ptr::null_mut();
            }
            sys::DeleteDC(art.strip_dc);
            art.strip_dc = core::ptr::null_mut();
        }
        if !art.strip_bmp.is_null() {
            sys::DeleteObject(art.strip_bmp);
            art.strip_bmp = core::ptr::null_mut();
        }
        if !art.cards_dll.is_null() {
            sys::FreeLibrary(art.cards_dll);
            art.cards_dll = core::ptr::null_mut();
        }
        art.card_width = 0;
        art.card_height = 0;
    }
}

/// Blit one slot of the strip, rounding the corners.
///
/// The artwork has square corners — white on the faces, yellow on the backs —
/// so a plain blit puts four wrong-coloured pixels on the table. Saving the
/// three pixels at each corner and putting them back afterwards is how
/// cards.dll's own `cdtDrawExt` does it, and it costs nothing.
pub fn draw_card(hdc: HDC, x: c_int, y: c_int, slot: c_int) {
    if !is_initialised() {
        return;
    }
    let (w, h) = (card_width(), card_height());
    let slot = slot.clamp(0, NUM_CARD_BITMAPS as c_int - 1);

    // The three pixels that make each corner look square.
    let corners = [
        (x, y),
        (x + 1, y),
        (x, y + 1),
        (x + w - 1, y),
        (x + w - 2, y),
        (x + w - 1, y + 1),
        (x, y + h - 1),
        (x + 1, y + h - 1),
        (x, y + h - 2),
        (x + w - 1, y + h - 1),
        (x + w - 2, y + h - 1),
        (x + w - 1, y + h - 2),
    ];

    unsafe {
        let mut saved = [0u32; 12];
        for (i, &(cx, cy)) in corners.iter().enumerate() {
            saved[i] = sys::GetPixel(hdc, cx, cy);
        }
        sys::BitBlt(hdc, x, y, w, h, strip_dc(), slot * w, 0, sys::SRCCOPY);
        for (i, &(cx, cy)) in corners.iter().enumerate() {
            // CLR_INVALID comes back when the pixel was outside the clip; put
            // nothing back rather than painting it black.
            if saved[i] != 0xFFFF_FFFF {
                sys::SetPixel(hdc, cx, cy, saved[i]);
            }
        }
    }
}
