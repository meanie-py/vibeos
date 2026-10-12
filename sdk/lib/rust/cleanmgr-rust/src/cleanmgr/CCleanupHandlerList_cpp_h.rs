use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_5E350CC9351A: [WCHAR; 4] = [0x0025, 0x0063, 0x003A, 0x0000];
pub static CPP_COM_WSTR_9F38BEEF6450: [WCHAR; 64] = [0x0053, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0056, 0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x0043, 0x0061, 0x0063, 0x0068, 0x0065, 0x0073, 0x0000];

use core::ffi::c_void;




impl CCleanupHandlerList {
    pub unsafe fn LoadHandlers(&mut self, Drive: WCHAR) {
        let this_: *mut Self = self;
        atl_format!((*this_).m_DriveStr, CPP_COM_WSTR_5E350CC9351A.as_ptr(), Drive);
    
        let mut VolumeCaches = CRegKey__make_0();
        if VolumeCaches.Open(HKEY_LOCAL_MACHINE, CPP_COM_WSTR_9F38BEEF6450.as_ptr(), KEY_READ) != (ERROR_SUCCESS as i32) {
            return;
        }
    
        let mut ItemIndex: LONG = 0;
        let mut szKeyName = [0u16; (MAX_PATH) as _];
    
        let mut wszVolume: [WCHAR; 4] = [Drive, 0x003Au16, 0x005Cu16, UNICODE_NULL];
    
        while TRUE != 0 {
            let mut dwSize: DWORD = _countof!(szKeyName) as DWORD;
            let __idx: DWORD = ItemIndex as DWORD;
            ItemIndex += 1;
            if VolumeCaches.EnumKey(__idx, szKeyName.as_mut_ptr(), core::ptr::addr_of_mut!(dwSize), core::ptr::null_mut()) != (ERROR_SUCCESS as i32) {
                break;
            }
    
            let mut hSubKey = CRegKey__make_0();
            if hSubKey.Open((VolumeCaches).op_to_HKEY(), szKeyName.as_ptr(), KEY_READ) == (ERROR_SUCCESS as i32) {
                let mut GuidStr = [0u16; 50];
                dwSize = _countof!(GuidStr) as DWORD;
                if hSubKey.QueryStringValue(core::ptr::null(), GuidStr.as_mut_ptr(), core::ptr::addr_of_mut!(dwSize)) != (ERROR_SUCCESS as i32) {
                    continue;
                }
    
                let mut guid: GUID = core::mem::zeroed();
                if FAILED_UNEXPECTEDLY!(CLSIDFromString(GuidStr.as_ptr(), core::ptr::addr_of_mut!(guid))) {
                    continue;
                }
    
                let handler: *mut CCleanupHandler = CCleanupHandler__new_0(&mut hSubKey, &CStringW::from_ptr(szKeyName.as_ptr()), &guid);
    
                if !(*handler).Initialize(wszVolume.as_ptr()) {
                    CCleanupHandler__delete(handler);
                    continue;
                }
    
                (*this_).m_Handlers.AddTail(handler);
            }
        }
    
        // Sort handlers
        let mut fChanged: BOOL = ((*this_).m_Handlers.GetCount() > 0) as BOOL;
        while fChanged != 0 {
            fChanged = (FALSE as i32);
    
            let mut n: usize = 0 as usize;
            while n < ((*this_).m_Handlers.GetCount() as usize) - (1 as usize) {
                let leftPos = (*this_).m_Handlers.FindIndex((n) as i32);
                let rightPos = (*this_).m_Handlers.FindIndex((n + (1 as usize)) as i32);
                let left: *mut CCleanupHandler = ((*this_).m_Handlers.GetAt(leftPos)).cast();
                let right: *mut CCleanupHandler = ((*this_).m_Handlers.GetAt(rightPos)).cast();
    
                if (*right).Priority < (*left).Priority {
                    (*this_).m_Handlers.SwapElements(leftPos, rightPos);
                    fChanged = (TRUE as i32);
                } else if (*right).Priority == (*left).Priority {
                    let mut leftStr = CStringW::from_ptr((*left).wszDisplayName.as_ptr());
                    if leftStr.Compare((*right).wszDisplayName.as_ptr()) > 0 {
                        (*this_).m_Handlers.SwapElements(leftPos, rightPos);
                        fChanged = (TRUE as i32);
                    }
                }
                n += 1;
            }
        }
    }
    pub unsafe fn ScanDrive(&mut self, picb: *mut IEmptyVolumeCacheCallBack) -> DWORDLONG {
        let this_: *mut Self = self;
        let mut progress = CProgressDlg__make_0();
        let mut Caption = CStringW::new();
        atl_format_id!(Caption, IDS_CALCULATING, (*this_).m_DriveStr.GetString());
        let mut Title = CStringW::from_ptr(MAKEINTRESOURCE!(IDS_DISK_CLEANUP) as *const WCHAR);
        progress.Start((*this_).m_Handlers.GetCount() as DWORD, Title.GetString(), Caption.GetString());
        let mut ItemIndex: i32 = 0;
        let mut TotalSpaceUsed: DWORDLONG = 0;
        let mut cb = |current: *mut CCleanupHandler| {
            atl_format_id!(Caption, IDS_SCANNING, (*current).wszDisplayName.m_pData);
            progress.Step({ ItemIndex += 1; ItemIndex as DWORD }, Caption.GetString());
    
            let hr: HRESULT = (*(*current).Handler.p).GetSpaceUsed(core::ptr::addr_of_mut!((*current).SpaceUsed), picb);
    
            if FAILED_UNEXPECTEDLY!(hr) != false {
                (*current).ShowHandler = false;
                (*current).StateFlags &= (!HANDLER_STATE_SELECTED) as u32;
                return;
            }
    
            if (*current).SpaceUsed == 0 && (*current).DontShowIfZero() != 0 {
                (*current).ShowHandler = false;
                (*current).StateFlags &= (!HANDLER_STATE_SELECTED) as u32;
            }
            TotalSpaceUsed += (*current).SpaceUsed;
        };
        (*this_).ForEach(&mut cb);
        progress.Stop();
    
        return TotalSpaceUsed;
    }
    pub unsafe fn ExecuteCleanup(&mut self, picb: *mut IEmptyVolumeCacheCallBack) {
        let this_: *mut Self = self;
        let mut progress = CProgressDlg__make_0();
        let mut Caption = CStringW::new();
        atl_format_id!(Caption, IDS_CLEANING_CAPTION, (*this_).m_DriveStr.GetString());
    
        let mut TotalSelected: DWORD = 0;
        (*this_).ForEach(|current: *mut CCleanupHandler| {
            if ((*current).StateFlags & (HANDLER_STATE_SELECTED as u32)) != 0 {
                TotalSelected += 1;
            }
        });
    
        let Title = CStringW::from_ptr(MAKEINTRESOURCE!(IDS_DISK_CLEANUP));
        progress.Start(TotalSelected, Title.GetString(), Caption.GetString());
        let mut ItemIndex: i32 = 0;
        (*this_).ForEach(|current: *mut CCleanupHandler| {
            if ((*current).StateFlags & (HANDLER_STATE_SELECTED as u32)) == 0 {
                return;
            }
    
            atl_format_id!(Caption, IDS_CLEANING, (*current).wszDisplayName.m_pData);
            ItemIndex += 1;
            progress.Step(ItemIndex as DWORD, Caption.GetString());
    
            // If there is nothing to clean, we might get STG_E_NOMOREFILES
            if (*current).SpaceUsed > 0 {
                let hr: HRESULT = (*(*current).Handler.p).Purge((1 as u64).wrapping_neg(), picb);
                if FAILED_UNEXPECTEDLY!(hr) {
                    return;
                }
            }
        });
        progress.Stop();
    }
}
