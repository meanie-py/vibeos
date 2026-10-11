use crate::prelude::*;
use core::ffi::c_void;









impl CSortableHeader {
    pub unsafe extern "system" fn s_CompareFunc(lParam1: LPARAM, lParam2: LPARAM, lParamSort: LPARAM) -> INT {
        let mut ctx: *mut SortContext = lParamSort as *mut SortContext;
        return (*(*ctx).hdr).CompareFunc(lParam1, lParam2, ctx);
    }
    pub unsafe fn CompareFunc(&mut self, lParam1: LPARAM, lParam2: LPARAM, ctx: *mut SortContext) -> INT {
        let this_: *mut Self = self;
        let iSubItem: INT = (*ctx).iSubItem;
    
        let mut Item1 = CStringW::new();
        let mut Item2 = CStringW::new();
        let mut IndexInfo: LVFINDINFOW = core::mem::zeroed();
        let mut Index: INT;
    
        IndexInfo.flags = LVFI_PARAM;
    
        IndexInfo.lParam = lParam1;
        Index = (*(*ctx).lvw).FindItem(-(1), core::ptr::addr_of_mut!(IndexInfo));
        (*(*ctx).lvw).GetItemText(Index, iSubItem, Item1.GetBuffer((MAX_STR_LEN as i32)), (MAX_STR_LEN as i32));
        Item1.ReleaseBuffer();
    
        IndexInfo.lParam = lParam2;
        Index = (*(*ctx).lvw).FindItem(-(1), core::ptr::addr_of_mut!(IndexInfo));
        (*(*ctx).lvw).GetItemText(Index, iSubItem, Item2.GetBuffer((MAX_STR_LEN as i32)), (MAX_STR_LEN as i32));
        Item2.ReleaseBuffer();
    
        return if (*this_).m_Ascending {
            Item1.Compare(Item2.GetString())
        } else {
            Item2.Compare(Item1.GetString())
        };
    }
    pub unsafe fn ColumnClick(&mut self, listView: *mut CListView, pnmv: LPNMLISTVIEW) {
        let this_: *mut Self = self;
        if (((*listView).GetWindowLongPtr(GWL_STYLE) as u32) & !LVS_NOSORTHEADER) == 0 {
            return;
        }
    
        (*this_).ApplySorting(&mut *listView, (*pnmv).iSubItem);
    
        /* Save new values */
        (*this_).m_LastHeaderID = (*pnmv).iSubItem;
        (*this_).m_Ascending = !(*this_).m_Ascending;
    }
    pub unsafe fn ReapplySorting(&mut self, listView: &mut CListView) {
        let this_: *mut Self = self;
        if (*this_).m_LastHeaderID != -1 {
            (*this_).ApplySorting(listView, (*this_).m_LastHeaderID);
        }
    }
    pub unsafe fn ApplySorting(&mut self, listView: &mut CListView, nHeaderID: INT) {
        let this_: *mut Self = self;
        let hHeader: HWND = listView.SendMessage(LVM_GETHEADER, 0, 0) as HWND;
        let mut hColumn: HDITEMW = core::mem::zeroed();
        ZeroMemory!(core::ptr::addr_of_mut!(hColumn) as *mut core::ffi::c_void, core::mem::size_of::<HDITEMW>());
    
        /* If the sorting column changed, remove the sorting style from the old column */
        if ((*this_).m_LastHeaderID != -1) && ((*this_).m_LastHeaderID != nHeaderID) {
            (*this_).m_Ascending = ((TRUE) != 0); // also reset sorting method to ascending
            hColumn.mask = HDI_FORMAT;
            Header_GetItem!(hHeader, (*this_).m_LastHeaderID, core::ptr::addr_of_mut!(hColumn));
            hColumn.fmt &= (!(HDF_SORTUP | HDF_SORTDOWN)) as i32;
            Header_SetItem!(hHeader, (*this_).m_LastHeaderID, core::ptr::addr_of_mut!(hColumn));
        }
    
        /* Set the sorting style to the new column */
        hColumn.mask = HDI_FORMAT;
        Header_GetItem!(hHeader, nHeaderID, core::ptr::addr_of_mut!(hColumn));
    
        hColumn.fmt &= (!(HDF_SORTDOWN | HDF_SORTUP)) as i32;
        hColumn.fmt |= (if (*this_).m_Ascending { HDF_SORTUP } else { HDF_SORTDOWN }) as i32;
        Header_SetItem!(hHeader, nHeaderID, core::ptr::addr_of_mut!(hColumn));
    
        /* Sort the list, using the current values of nHeaderID and bIsAscending */
        let mut ctx: SortContext = SortContext {
            hdr: this_,
            lvw: listView as *mut CListView,
            iSubItem: nHeaderID,
        };
        listView.SortItems(Some(CSortableHeader::s_CompareFunc), core::ptr::addr_of_mut!(ctx) as *mut core::ffi::c_void);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_LastHeaderID = 0;
        (*this_).m_Ascending = true;
    }
}

impl SortContext {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

pub const MAX_STR_LEN: c_int = 256;
