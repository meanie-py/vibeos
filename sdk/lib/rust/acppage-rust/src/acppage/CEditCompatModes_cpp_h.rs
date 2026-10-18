use crate::prelude::*;
use core::ffi::c_void;









#[no_mangle]
pub unsafe extern "C" fn ShowEditCompatModes(hWnd: HWND, page: *mut CLayerUIPropPage) -> BOOL {
    let mut modes = CEditCompatModes__make_0(page);
    let Result: INT_PTR = modes.DoModal(hWnd, 0);
    return if Result == IDOK as INT_PTR { 1 } else { 0 };
}

impl CEditCompatModes {
    pub unsafe fn GetListText(&mut self, ListBox: HWND, Cur: i32) -> CStringW {
        let this_: *mut Self = self;
        let mut Str = CStringW::new();
        let Length = ListBox_GetTextLen!(ListBox, Cur);
        let Buffer: LPWSTR = Str.GetBuffer(Length + 1);
        ListBox_GetText!(ListBox, Cur, Buffer);
        Str.ReleaseBufferSetLength(Length);
        return Str;
    }
    pub unsafe fn __ctor_0(&mut self, page: *mut CLayerUIPropPage) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_pPage = page;
        {
            (*(*this_).m_pPage).AddRef();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*(*this_).m_pPage).Release();
    }
    pub unsafe fn OnInitDialog(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
            (*this_).__base0.CenterWindow((*this_).__base0.GetParent().op_to_HWND());

            (*this_).m_hListActive = (*this_).__base0.GetDlgItem((IDC_COMPATIBILITYMODE as i32));
            (*this_).m_hListAdd = (*this_).__base0.GetDlgItem((IDC_NEWCOMPATIBILITYMODE as i32));

            // `CComObject<CLayerStringList> pList;` on the stack: the class itself, constructed in
            // place (its COM glue is its own; nothing takes a reference to it), plus what
            // CComObject adds: the module lock, from its constructor to its destructor.
            let mut pList: CLayerStringList = CLayerStringList__make_0();
            (*core::ptr::addr_of_mut!(gModule)).Lock();

            loop {
                let mut str: CComHeapPtr<OLECHAR> = CComHeapPtr::<OLECHAR>::new();
                let hr: HRESULT = pList.Next(1, str.GetAddressOf(), core::ptr::null_mut());
                if hr != S_OK {
                    break;
                }
                ListBox_AddString!((*this_).m_hListAdd, str.op_to_LPCWSTR());
            }

            let mut n: i32 = 0;
            while n < (*(*this_).m_pPage).m_CustomLayers.GetSize() {
                let Str: *const u16 = (&(*(*this_).m_pPage).m_CustomLayers)[n as usize].GetString();
                let mut Index: i32 = ListBox_FindStringExact!((*this_).m_hListActive, -1, Str);
                if Index == LB_ERR {
                    Index = ListBox_AddString!((*this_).m_hListActive, Str);
                }
                n += 1;
            }

            (*this_).OnListboxUpdated(0, 0, core::ptr::null_mut(), bHandled);
            (*core::ptr::addr_of_mut!(gModule)).Unlock(); // ~CComObject, before pList drops
            return 0;
        }
    pub unsafe fn OnButton(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if wID == (IDOK as u16) {
            let Count: i32 = ListBox_GetCount!((*this_).m_hListActive);
            (*(*this_).m_pPage).m_CustomLayers.RemoveAll();
            let mut Cur: i32 = 0;
            while Cur < Count {
                let mut Str: CStringW = (*this_).GetListText((*this_).m_hListActive, Cur);
                (*(*this_).m_pPage).m_CustomLayers.Add(Str);
                Cur += 1;
            }
        }
        (*this_).EndDialog((wID) as _);
        return 0;
    }
    pub unsafe fn OnAdd(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
            let mut Sel: i32 = ListBox_GetCurSel!((*this_).m_hListAdd) as i32;
            let mut Str: CStringW = (*this_).GetListText((*this_).m_hListAdd, Sel);

            let mut Index: i32 = ListBox_FindStringExact!((*this_).m_hListActive, -1, Str.GetString() as LPARAM) as i32;
            if Index == LB_ERR {
                Index = ListBox_AddString!((*this_).m_hListActive, Str.GetString() as LPARAM) as i32;
            }

            SetFocus((*this_).m_hListAdd);
            return 0;
        }
    pub unsafe fn OnRemove(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut Sel: i32 = ListBox_GetCurSel!((*this_).m_hListActive);
        let mut Str: CStringW = (*this_).GetListText((*this_).m_hListActive, Sel);

        ListBox_DeleteString!((*this_).m_hListActive, Sel);
        let mut Index: i32 = ListBox_FindStringExact!((*this_).m_hListAdd, -1, Str.GetString());
        if Index != LB_ERR {
            Index = ListBox_SetCurSel!((*this_).m_hListAdd, Index);
        }
        (*this_).OnListboxUpdated(wNotifyCode, wID, hWndCtl, bHandled);
        return 0;
    }
    pub unsafe fn OnRemoveAll(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
            ListBox_ResetContent!((*this_).m_hListActive);
            (*this_).OnListboxUpdated(wNotifyCode, wID, hWndCtl, bHandled);
            return 0;
        }
    pub unsafe fn OnListboxUpdated(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if (*this_).m_hListAdd == hWndCtl {
            ListBox_SetCurSel!((*this_).m_hListActive, -1);
        } else if (*this_).m_hListActive == hWndCtl {
            ListBox_SetCurSel!((*this_).m_hListAdd, -1);
        }

        EnableWindow((*this_).__base0.GetDlgItem((IDC_ADD as i32)), (ListBox_GetCurSel!((*this_).m_hListAdd) >= 0) as BOOL);
        EnableWindow((*this_).__base0.GetDlgItem((IDC_REMOVE as i32)), (ListBox_GetCurSel!((*this_).m_hListActive) >= 0) as BOOL);
        EnableWindow((*this_).__base0.GetDlgItem((IDC_REMOVEALL as i32)), (ListBox_GetCount!((*this_).m_hListActive) > 0) as BOOL);
        *bHandled = (TRUE as i32);
        return 0;
    }
}
