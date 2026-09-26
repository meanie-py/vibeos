//! `sdk/include/reactos/ui/rosctrls.h` + the `CWindow` methods it inherits from `atl/atlwin.h`.
//!
//! These are C++ CLASS METHODS, so the bindgen island renders only the STRUCT — `CListView` is
//! `{ _base: ATL_CWindow }` and `ATL_CWindow` is `{ m_hWnd: HWND }`. Every method below is a
//! thin `SendMessage` wrapper that existed only in the header, which means no symbol exists to
//! import and every translated TU that touches a list view needs this file instead.
//!
//! Ported straight from the header, one method at a time, with the message id and the
//! WPARAM/LPARAM packing kept EXACTLY as the C++ writes them. The packing is the whole risk
//! here: `SetItemPosition` puts the point in the LPARAM via MAKELPARAM while `GetItemPosition`
//! passes a POINTER, and `SortItems` puts the comparator in the LPARAM and the context in the
//! WPARAM — the opposite order from every other method in the file. Getting one of those
//! backwards compiles cleanly and fails at runtime.
//!
//! Overload note: rosctrls.h declares `CListView`, `CTreeView` and `CToolbar` in one header and
//! several method NAMES are shared between them with different messages (`GetItemCount` is
//! LVM_GETITEMCOUNT on the list view and TVM_GETCOUNT on the tree view). Only the `CListView`
//! (LVM_*) forms are ported here; do not merge a tree-view overload into one of these.

use crate::prelude::*;

/// `class CListView: public CWindow` — C++ INHERITS the base, bindgen COMPOSES it as `_base`,
/// so `m_hWnd` stops being reachable by name and every `m_ListView.m_hWnd` in a translated TU
/// stops compiling. Deref restores the inherited-member spelling (Rust auto-derefs for field
/// access), which is the closest thing the language has to a public base class.
impl core::ops::Deref for CListView {
    type Target = ATL_CWindow;
    #[inline]
    fn deref(&self) -> &ATL_CWindow {
        &self._base
    }
}

impl core::ops::DerefMut for CListView {
    #[inline]
    fn deref_mut(&mut self) -> &mut ATL_CWindow {
        &mut self._base
    }
}

/// `m_hWnd` is the entire object — ATL's window wrappers are handle-sized by design, which is
/// why they are passed and stored by value and why `core::mem::zeroed()` is a valid empty one.
#[inline]
unsafe fn hwnd_of(view: &CListView) -> HWND {
    view._base.m_hWnd
}

/// `WC_LISTVIEW` is `L"SysListView32"`. The island renders it as a BYTE string, which is the
/// ANSI spelling and would make CreateWindowExW read garbage — so the wide form is written out
/// here as an explicit array, terminator included, per the crate's wide-literal rule.
static WC_LISTVIEW_W: [u16; 14] = [
    'S' as u16, 'y' as u16, 's' as u16, 'L' as u16, 'i' as u16, 's' as u16, 't' as u16,
    'V' as u16, 'i' as u16, 'e' as u16, 'w' as u16, '3' as u16, '2' as u16, 0,
];

#[inline]
fn make_lparam(low: u16, high: u16) -> LPARAM {
    (((high as u32) << 16) | (low as u32)) as i32 as LPARAM
}

impl CListView {
    // -- CWindow basics -------------------------------------------------------------------

    /// The C++ tests a `CListView` for emptiness through its `operator HWND`, i.e. against the
    /// HANDLE, not against the wrapper.
    #[inline]
    pub unsafe fn is_null(&self) -> bool {
        hwnd_of(self).is_null()
    }

    #[inline]
    pub unsafe fn SendMessageW(&self, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        crate::island::SendMessageW(hwnd_of(self), msg, wParam, lParam)
    }

    /// shell32 is a Unicode build, so ATL's `SendMessage` IS `SendMessageW` here.
    #[inline]
    pub unsafe fn SendMessage(&self, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        self.SendMessageW(msg, wParam, lParam)
    }

    #[inline]
    pub unsafe fn IsWindow(&self) -> BOOL {
        IsWindow(hwnd_of(self))
    }

    #[inline]
    pub unsafe fn IsChild(&self, hWnd: HWND) -> BOOL {
        IsChild(hwnd_of(self), hWnd)
    }

    #[inline]
    pub unsafe fn SetFocus(&self) -> HWND {
        SetFocus(hwnd_of(self))
    }

    #[inline]
    pub unsafe fn UpdateWindow(&self) -> BOOL {
        UpdateWindow(hwnd_of(self))
    }

    #[inline]
    pub unsafe fn InvalidateRect(&self, lpRect: LPCRECT, bErase: BOOL) -> BOOL {
        InvalidateRect(hwnd_of(self), lpRect, bErase)
    }

    #[inline]
    pub unsafe fn ClientToScreen(&self, lpPoint: LPPOINT) -> BOOL {
        ClientToScreen(hwnd_of(self), lpPoint)
    }

    /// Win32 defines `GetWindowLongPtr` as a MACRO onto `GetWindowLong` in 32-bit builds, so
    /// there is no such export to import there — hence the split rather than one extern.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    pub unsafe fn GetWindowLongPtr(&self, nIndex: c_int) -> LONG_PTR {
        GetWindowLongPtrW(hwnd_of(self), nIndex)
    }

    #[cfg(target_pointer_width = "32")]
    #[inline]
    pub unsafe fn GetWindowLongPtr(&self, nIndex: c_int) -> LONG_PTR {
        GetWindowLongW(hwnd_of(self), nIndex) as LONG_PTR
    }

    /// ATL reads the style with `::GetWindowLong`, not the Ptr form — a window style is 32 bits
    /// on every target, so widening it would only invite a sign-extension question.
    #[inline]
    pub unsafe fn GetStyle(&self) -> DWORD {
        GetWindowLongW(hwnd_of(self), GWL_STYLE as c_int) as DWORD
    }

    /// `nFlags == 0` means "do not reposition", which is the common case and the reason the
    /// SetWindowPos call is conditional rather than unconditional-with-no-op-flags.
    pub unsafe fn ModifyStyle(&self, dwRemove: DWORD, dwAdd: DWORD, nFlags: UINT) -> BOOL {
        let hwnd = hwnd_of(self);
        let style = (GetWindowLongW(hwnd, GWL_STYLE as c_int) as DWORD & !dwRemove) | dwAdd;
        SetWindowLongW(hwnd, GWL_STYLE as c_int, style as LONG);
        if nFlags != 0 {
            return SetWindowPos(
                hwnd,
                core::ptr::null_mut(),
                0,
                0,
                0,
                0,
                nFlags | SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        TRUE
    }

    // -- list-view messages ---------------------------------------------------------------

    pub unsafe fn Create(
        &mut self,
        hWndParent: HWND,
        rect: *const RECT,
        szWindowName: LPCWSTR,
        dwStyle: DWORD,
        dwExStyle: DWORD,
        MenuOrID: HMENU,
        lpCreateParam: LPVOID,
    ) -> HWND {
        self._base.m_hWnd = CreateWindowExW(
            dwExStyle,
            WC_LISTVIEW_W.as_ptr(),
            szWindowName,
            dwStyle,
            (*rect).left,
            (*rect).top,
            (*rect).right - (*rect).left,
            (*rect).bottom - (*rect).top,
            hWndParent,
            MenuOrID,
            // `_AtlBaseModule.GetModuleInstance()` is shell32's OWN HINSTANCE, not the
            // process image — see the note on atl::LoadStringW for why that distinction bites.
            crate::win32::shell32_hInstance as HINSTANCE,
            lpCreateParam,
        );
        self._base.m_hWnd
    }

    #[inline]
    pub unsafe fn SetRedraw(&self, redraw: BOOL) {
        self.SendMessage(WM_SETREDRAW, redraw as WPARAM, 0);
    }

    #[inline]
    pub unsafe fn SetExtendedListViewStyle(&self, dw: DWORD, dwMask: DWORD) -> DWORD {
        self.SendMessage(LVM_SETEXTENDEDLISTVIEWSTYLE, dwMask as WPARAM, dw as LPARAM) as DWORD
    }

    #[inline]
    pub unsafe fn SetImageList(&self, himl: HIMAGELIST, iImageList: c_int) -> HIMAGELIST {
        self.SendMessage(LVM_SETIMAGELIST, iImageList as WPARAM, himl as LPARAM) as HIMAGELIST
    }

    #[inline]
    pub unsafe fn InsertItem(&self, pitem: *const LVITEMW) -> c_int {
        self.SendMessage(LVM_INSERTITEMW, 0, pitem as LPARAM) as c_int
    }

    #[inline]
    pub unsafe fn DeleteItem(&self, i: c_int) -> BOOL {
        self.SendMessage(LVM_DELETEITEM, i as WPARAM, 0) as BOOL
    }

    #[inline]
    pub unsafe fn DeleteAllItems(&self) -> BOOL {
        self.SendMessage(LVM_DELETEALLITEMS, 0, 0) as BOOL
    }

    #[inline]
    pub unsafe fn GetItem(&self, pitem: *mut LVITEMW) -> BOOL {
        self.SendMessage(LVM_GETITEMW, 0, pitem as LPARAM) as BOOL
    }

    #[inline]
    pub unsafe fn SetItem(&self, pitem: *const LVITEMW) -> BOOL {
        self.SendMessage(LVM_SETITEMW, 0, pitem as LPARAM) as BOOL
    }

    #[inline]
    pub unsafe fn GetItemCount(&self) -> c_int {
        self.SendMessage(LVM_GETITEMCOUNT, 0, 0) as c_int
    }

    /// Reads the item's LPARAM through a LVIF_PARAM query, exactly as the header does — there
    /// is no LVM_GETITEMDATA message.
    pub unsafe fn GetItemData(&self, i: c_int) -> DWORD_PTR {
        let mut lvItem: LVITEMW = core::mem::zeroed();
        lvItem.mask = LVIF_PARAM;
        lvItem.iItem = i;
        if self.GetItem(&mut lvItem) != 0 {
            lvItem.lParam as DWORD_PTR
        } else {
            0
        }
    }

    #[inline]
    pub unsafe fn GetItemPosition(&self, nItem: c_int, pPoint: *mut POINT) -> BOOL {
        self.SendMessage(LVM_GETITEMPOSITION, nItem as WPARAM, pPoint as LPARAM) as BOOL
    }

    /// Note the asymmetry with `GetItemPosition`: the point travels PACKED IN THE LPARAM here,
    /// not as a pointer. That is the header's behaviour, and it caps each coordinate at 16 bits.
    #[inline]
    pub unsafe fn SetItemPosition(&self, nItem: c_int, pPoint: *const POINT) -> BOOL {
        self.SendMessage(
            LVM_SETITEMPOSITION,
            nItem as WPARAM,
            make_lparam((*pPoint).x as u16, (*pPoint).y as u16),
        ) as BOOL
    }

    /// LVM_GETITEMSPACING returns the pair packed in one DWORD rather than filling a SIZE.
    pub unsafe fn GetItemSpacing(&self, spacing: &mut SIZE, bSmallIconView: BOOL) {
        let ret = self.SendMessage(LVM_GETITEMSPACING, bSmallIconView as WPARAM, 0) as DWORD;
        spacing.cx = (ret & 0xFFFF) as LONG;
        spacing.cy = ((ret >> 16) & 0xFFFF) as LONG;
    }

    #[inline]
    pub unsafe fn GetItemState(&self, i: c_int, mask: UINT) -> UINT {
        self.SendMessage(LVM_GETITEMSTATE, i as WPARAM, mask as LPARAM) as UINT
    }

    /// Only `state`/`stateMask` are read by LVM_SETITEMSTATE, which is why the header leaves the
    /// rest of the LV_ITEM uninitialized; zeroing it is harmless and avoids a garbage read if
    /// that ever stops being true.
    pub unsafe fn SetItemState(&self, i: c_int, state: UINT, mask: UINT) {
        let mut item: LVITEMW = core::mem::zeroed();
        item.stateMask = mask;
        item.state = state;
        self.SendMessage(LVM_SETITEMSTATE, i as WPARAM, core::ptr::addr_of!(item) as LPARAM);
    }

    pub unsafe fn SetItemText(&self, i: c_int, subItem: c_int, text: LPCWSTR) -> BOOL {
        let mut item: LVITEMW = core::mem::zeroed();
        item.iSubItem = subItem;
        item.pszText = text as LPWSTR;
        self.SendMessage(LVM_SETITEMTEXTW, i as WPARAM, core::ptr::addr_of!(item) as LPARAM) as BOOL
    }

    #[inline]
    pub unsafe fn GetNextItem(&self, i: c_int, flags: WORD) -> c_int {
        self.SendMessage(LVM_GETNEXTITEM, i as WPARAM, make_lparam(flags, 0)) as c_int
    }

    #[inline]
    pub unsafe fn GetSelectedCount(&self) -> UINT {
        self.SendMessage(LVM_GETSELECTEDCOUNT, 0, 0) as UINT
    }

    #[inline]
    pub unsafe fn GetSelectionMark(&self) -> c_int {
        self.SendMessage(LVM_GETSELECTIONMARK, 0, 0) as c_int
    }

    #[inline]
    pub unsafe fn HitTest(&self, phtInfo: *mut LVHITTESTINFO) -> c_int {
        self.SendMessage(LVM_HITTEST, 0, phtInfo as LPARAM) as c_int
    }

    #[inline]
    pub unsafe fn EnsureVisible(&self, i: c_int, fPartialOK: BOOL) -> BOOL {
        self.SendMessage(LVM_ENSUREVISIBLE, i as WPARAM, make_lparam(fPartialOK as u16, 0)) as BOOL
    }

    #[inline]
    pub unsafe fn EditLabel(&self, i: c_int) -> HWND {
        self.SendMessage(LVM_EDITLABELW, i as WPARAM, 0) as HWND
    }

    #[inline]
    pub unsafe fn Arrange(&self, nCode: UINT) -> BOOL {
        self.SendMessage(LVM_ARRANGE, nCode as WPARAM, 0) as BOOL
    }

    /// The ONE method in this file that puts the callback in the LPARAM and the caller's context
    /// in the WPARAM. Every other wrapper here is the other way round; do not "normalize" it.
    #[inline]
    pub unsafe fn SortItems(&self, pfnCompare: PFNLVCOMPARE, lParam: PVOID) -> BOOL {
        self.SendMessage(LVM_SORTITEMS, lParam as WPARAM, pfnCompare_as_lparam(pfnCompare)) as BOOL
    }

    #[inline]
    pub unsafe fn GetScrollPos(&self, nBar: c_int) -> c_int {
        GetScrollPos(hwnd_of(self), nBar)
    }

    #[inline]
    pub unsafe fn Update(&self, i: c_int) -> BOOL {
        self.SendMessage(LVM_UPDATE, i as WPARAM, 0) as BOOL
    }
}

#[inline]
fn pfnCompare_as_lparam(pfnCompare: PFNLVCOMPARE) -> LPARAM {
    match pfnCompare {
        Some(f) => f as usize as LPARAM,
        None => 0,
    }
}

// ---------------------------------------------------------------------------------------------
// commctrl.h ListView_* / Header_* macros
// ---------------------------------------------------------------------------------------------
//
// Free functions taking an HWND, matching the macros' own shape. The `W` message ids are used
// wherever commctrl resolves through __MINGW_NAME_AW, because shell32 is a Unicode build.

#[inline]
unsafe fn snd(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    crate::island::SendMessageW(hwnd, msg, w, l)
}

/// NOTE the side effect the macro hides: it writes the wanted portion code into `prc->left`
/// BEFORE sending, because LVM_GETITEMRECT takes its input that way. Dropping that write
/// silently returns the wrong rectangle rather than failing.
#[inline]
pub unsafe fn ListView_GetItemRect(hwnd: HWND, i: c_int, prc: *mut RECT, code: c_int) -> BOOL {
    if prc.is_null() {
        return snd(hwnd, LVM_GETITEMRECT, i as WPARAM, 0) as BOOL;
    }
    (*prc).left = code as LONG;
    snd(hwnd, LVM_GETITEMRECT, i as WPARAM, prc as LPARAM) as BOOL
}

#[inline]
pub unsafe fn ListView_GetHeader(hwnd: HWND) -> HWND {
    snd(hwnd, LVM_GETHEADER, 0, 0) as HWND
}

#[inline]
pub unsafe fn ListView_GetOrigin(hwnd: HWND, ppt: *mut POINT) -> BOOL {
    snd(hwnd, LVM_GETORIGIN, 0, ppt as LPARAM) as BOOL
}

#[inline]
pub unsafe fn ListView_GetItemSpacing(hwnd: HWND, fSmall: BOOL) -> DWORD {
    snd(hwnd, LVM_GETITEMSPACING, fSmall as WPARAM, 0) as DWORD
}

#[inline]
pub unsafe fn ListView_DeleteColumn(hwnd: HWND, iCol: c_int) -> BOOL {
    snd(hwnd, LVM_DELETECOLUMN, iCol as WPARAM, 0) as BOOL
}

#[inline]
pub unsafe fn ListView_InsertColumn(hwnd: HWND, iCol: c_int, pcol: *const LVCOLUMNW) -> c_int {
    snd(hwnd, LVM_INSERTCOLUMNW, iCol as WPARAM, pcol as LPARAM) as c_int
}

#[inline]
pub unsafe fn ListView_SetColumn(hwnd: HWND, iCol: c_int, pcol: *const LVCOLUMNW) -> BOOL {
    snd(hwnd, LVM_SETCOLUMNW, iCol as WPARAM, pcol as LPARAM) as BOOL
}

#[inline]
pub unsafe fn ListView_HitTest(hwnd: HWND, pinfo: *mut LVHITTESTINFO) -> c_int {
    snd(hwnd, LVM_HITTEST, 0, pinfo as LPARAM) as c_int
}

#[inline]
pub unsafe fn ListView_GetNextItem(hwnd: HWND, i: c_int, flags: UINT) -> c_int {
    snd(hwnd, LVM_GETNEXTITEM, i as WPARAM, make_lparam(flags as u16, 0)) as c_int
}

#[inline]
pub unsafe fn ListView_RedrawItems(hwnd: HWND, iFirst: c_int, iLast: c_int) -> BOOL {
    snd(hwnd, LVM_REDRAWITEMS, iFirst as WPARAM, iLast as LPARAM) as BOOL
}

#[inline]
pub unsafe fn ListView_CancelEditLabel(hwnd: HWND) {
    snd(hwnd, LVM_CANCELEDITLABEL, 0, 0);
}

#[inline]
pub unsafe fn ListView_GetEditControl(hwnd: HWND) -> HWND {
    snd(hwnd, LVM_GETEDITCONTROL, 0, 0) as HWND
}

#[inline]
pub unsafe fn ListView_IsGroupViewEnabled(hwnd: HWND) -> BOOL {
    snd(hwnd, LVM_ISGROUPVIEWENABLED, 0, 0) as BOOL
}

#[inline]
pub unsafe fn Header_GetItem(hwndHD: HWND, i: c_int, phdi: *mut HDITEMW) -> BOOL {
    snd(hwndHD, HDM_GETITEMW, i as WPARAM, phdi as LPARAM) as BOOL
}

#[inline]
pub unsafe fn Header_SetItem(hwndHD: HWND, i: c_int, phdi: *const HDITEMW) -> BOOL {
    snd(hwndHD, HDM_SETITEMW, i as WPARAM, phdi as LPARAM) as BOOL
}

#[inline]
pub unsafe fn Header_GetItemCount(hwndHD: HWND) -> c_int {
    snd(hwndHD, HDM_GETITEMCOUNT, 0, 0) as c_int
}

// ---------------------------------------------------------------------------------------------
// `CListView_*_body` bridges
// ---------------------------------------------------------------------------------------------
//
// rustconv names a translated C++ method `<Class>_<Method>_body` and calls it with the receiver
// as the first argument. CListView's methods are header-only, so they were never translated —
// these forward the translator's spelling onto the real wrappers above, keeping the assembled
// module's call sites intact instead of rewriting hundreds of them.

#[inline]
pub unsafe fn CListView_SetTextColor_body(this: *mut CListView, cr: COLORREF) -> BOOL {
    (*this).SendMessage(LVM_SETTEXTCOLOR, 0, cr as LPARAM) as BOOL
}

#[inline]
pub unsafe fn CListView_SetTextBkColor_body(this: *mut CListView, cr: COLORREF) -> BOOL {
    (*this).SendMessage(LVM_SETTEXTBKCOLOR, 0, cr as LPARAM) as BOOL
}

#[inline]
pub unsafe fn CListView_SetBkColor_body(this: *mut CListView, cr: COLORREF) -> BOOL {
    (*this).SendMessage(LVM_SETBKCOLOR, 0, cr as LPARAM) as BOOL
}

#[inline]
pub unsafe fn CListView_SetExtendedListViewStyle_body(this: *mut CListView, dw: DWORD, dwMask: DWORD) -> DWORD {
    (*this).SetExtendedListViewStyle(dw, dwMask)
}

#[inline]
pub unsafe fn CListView_SetRedraw_body(this: *mut CListView, redraw: BOOL) {
    (*this).SetRedraw(redraw);
}

#[inline]
pub unsafe fn CListView_GetItemCount_body(this: *mut CListView) -> c_int {
    (*this).GetItemCount()
}

#[inline]
pub unsafe fn CListView_GetSelectedCount_body(this: *mut CListView) -> UINT {
    (*this).GetSelectedCount()
}

#[inline]
pub unsafe fn CListView_SetItem_body(this: *mut CListView, pitem: *const LVITEMW) -> BOOL {
    (*this).SetItem(pitem)
}

#[inline]
pub unsafe fn CListView_SetItemState_body(this: *mut CListView, i: c_int, state: UINT, mask: UINT) {
    (*this).SetItemState(i, state, mask);
}

#[inline]
pub unsafe fn CListView_EditLabel_body(this: *mut CListView, i: c_int) -> HWND {
    (*this).EditLabel(i)
}

#[inline]
pub unsafe fn CListView_EnsureVisible_body(this: *mut CListView, i: c_int, fPartialOK: BOOL) -> BOOL {
    (*this).EnsureVisible(i, fPartialOK)
}

#[inline]
pub unsafe fn CListView_Update_body(this: *mut CListView, i: c_int) -> BOOL {
    (*this).Update(i)
}

/// `operator BOOL()` on the CWindow base: the wrapper is true when it HOLDS a window. The
/// translator hands this the address of `m_hWnd` rather than the wrapper.
#[inline]
pub unsafe fn CListView_operator_BOOL_body(hWnd: *const HWND) -> BOOL {
    (!(*hWnd).is_null()) as BOOL
}

// ---------------------------------------------------------------------------------------
// commctrl.h list-view + image-list MACROS. Same rationale as everything above: these exist
// only in the header, so no symbol exists to import and every TU that touches a list view
// needs them written out. Message ids come from the island; the WPARAM/LPARAM packing is
// kept exactly as commctrl.h writes it.
// ---------------------------------------------------------------------------------------

/// `ListView_SetItem(hwnd, pitem)` — commctrl.h. The item goes in the LPARAM.
#[inline]
pub unsafe fn ListView_SetItem(hwnd: HWND, pitem: *const LVITEMW) -> BOOL {
    snd(hwnd, LVM_SETITEMW, 0, pitem as LPARAM) as BOOL
}

/// `ListView_GetItem(hwnd, pitem)` — commctrl.h.
#[inline]
pub unsafe fn ListView_GetItem(hwnd: HWND, pitem: *mut LVITEMW) -> BOOL {
    snd(hwnd, LVM_GETITEMW, 0, pitem as LPARAM) as BOOL
}

/// `ListView_DeleteItem(hwnd, i)` — commctrl.h. The index is the WPARAM here, not the LPARAM.
#[inline]
pub unsafe fn ListView_DeleteItem(hwnd: HWND, i: c_int) -> BOOL {
    snd(hwnd, LVM_DELETEITEM, i as WPARAM, 0) as BOOL
}

/// `ListView_SetItemState(hwnd, i, data, mask)` — commctrl.h.
///
/// The macro builds a STACK LVITEMW holding only `state`/`stateMask` and sends its address;
/// `i` rides in the WPARAM. Passing the flags directly in the LPARAM would be read as a
/// pointer, so the temporary is load-bearing rather than a convenience.
#[inline]
pub unsafe fn ListView_SetItemState(hwnd: HWND, i: c_int, data: UINT, mask: UINT) -> BOOL {
    let mut item: LVITEMW = core::mem::zeroed();
    item.state = data;
    item.stateMask = mask;
    snd(hwnd, LVM_SETITEMSTATE, i as WPARAM, core::ptr::addr_of_mut!(item) as LPARAM) as BOOL
}


/// `ListView_FindItem(hwnd, iStart, plvfi)` — commctrl.h. The START INDEX is the WPARAM and
/// is routinely -1 ("search from the beginning"), which is why it is signed.
#[inline]
pub unsafe fn ListView_FindItem(hwnd: HWND, iStart: c_int, plvfi: *const LVFINDINFOW) -> c_int {
    snd(hwnd, LVM_FINDITEMW, iStart as WPARAM, plvfi as LPARAM) as c_int
}

/// `ListView_GetItemText(hwnd, i, iSubItem, pszText, cchTextMax)` — commctrl.h.
///
/// The macro fills a STACK LVITEMW and calls LVM_GETITEMTEXTW; `iSubItem`, the buffer and its
/// length travel inside that struct, not as message parameters. Only `i` is the WPARAM.
#[inline]
pub unsafe fn ListView_GetItemText(
    hwnd: HWND,
    i: c_int,
    iSubItem: c_int,
    pszText: LPWSTR,
    cchTextMax: c_int,
) -> c_int {
    let mut item: LVITEMW = core::mem::zeroed();
    item.iSubItem = iSubItem;
    item.cchTextMax = cchTextMax;
    item.pszText = pszText;
    snd(hwnd, LVM_GETITEMTEXTW, i as WPARAM, core::ptr::addr_of_mut!(item) as LPARAM) as c_int
}

/// `ListView_DeleteAllItems(hwnd)` — commctrl.h.
#[inline]
pub unsafe fn ListView_DeleteAllItems(hwnd: HWND) -> BOOL {
    snd(hwnd, LVM_DELETEALLITEMS, 0, 0) as BOOL
}

/// `ListView_EnsureVisible(hwnd, i, fPartialOK)` — commctrl.h. `fPartialOK` is the LPARAM.
#[inline]
pub unsafe fn ListView_EnsureVisible(hwnd: HWND, i: c_int, fPartialOK: BOOL) -> BOOL {
    snd(hwnd, LVM_ENSUREVISIBLE, i as WPARAM, fPartialOK as LPARAM) as BOOL
}

/// `ListView_SetImageList(hwnd, himl, iImageList)` — commctrl.h. Returns the PREVIOUS list,
/// which the caller may need to destroy; discarding it silently leaks the old image list.
#[inline]
pub unsafe fn ListView_SetImageList(hwnd: HWND, himl: HIMAGELIST, iImageList: c_int) -> HIMAGELIST {
    snd(hwnd, LVM_SETIMAGELIST, iImageList as WPARAM, himl as LPARAM) as HIMAGELIST
}

/// `ListView_SortItems(hwnd, pfnCompare, lPrm)` — commctrl.h.
///
/// NOTE the order: the COMPARATOR goes in the LPARAM and the caller's context in the WPARAM,
/// the opposite of nearly every other list-view message. Swapping them calls an arbitrary
/// pointer as code.
#[inline]
pub unsafe fn ListView_SortItems(hwnd: HWND, pfnCompare: PFNLVCOMPARE, lPrm: LPARAM) -> BOOL {
    snd(hwnd, LVM_SORTITEMS, lPrm as WPARAM, pfnCompare.map_or(0, |f| f as usize) as LPARAM) as BOOL
}

/// `ListView_SetExtendedListViewStyleEx(hwnd, dwMask, dw)` — commctrl.h.
#[inline]
pub unsafe fn ListView_SetExtendedListViewStyleEx(hwnd: HWND, dwMask: DWORD, dw: DWORD) -> DWORD {
    snd(hwnd, LVM_SETEXTENDEDLISTVIEWSTYLE, dwMask as WPARAM, dw as LPARAM) as DWORD
}

/// `ImageList_RemoveAll(himl)` — commctrl.h:`ImageList_Remove(himl, -1)`.
#[inline]
pub unsafe fn ImageList_RemoveAll(himl: HIMAGELIST) -> BOOL {
    crate::island::ImageList_Remove(himl, -1)
}
