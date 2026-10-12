use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_04F93279A3AB: [WCHAR; 11] = [0x0055, 0x0073, 0x0065, 0x0072, 0x0020, 0x0053, 0x0074, 0x006F, 0x0072, 0x0065, 0x0000];
pub static CPP_COM_WSTR_0BCE0F081BC5: [WCHAR; 16] = [0x0045, 0x0078, 0x0070, 0x0069, 0x0072, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020, 0x0044, 0x0061, 0x0074, 0x0065, 0x0000];
pub static CPP_COM_WSTR_9C4B70748645: [WCHAR; 10] = [0x0049, 0x0073, 0x0073, 0x0075, 0x0065, 0x0064, 0x0020, 0x0042, 0x0079, 0x0000];
pub static CPP_COM_WSTR_BA52125E4714: [WCHAR; 14] = [0x0046, 0x0072, 0x0069, 0x0065, 0x006E, 0x0064, 0x006C, 0x0079, 0x0020, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_D29EB886FF27: [WCHAR; 15] = [0x0043, 0x006F, 0x006D, 0x0070, 0x0075, 0x0074, 0x0065, 0x0072, 0x0020, 0x0053, 0x0074, 0x006F, 0x0072, 0x0065, 0x0000];
pub static CPP_COM_WSTR_D90381C6A684: [WCHAR; 10] = [0x0049, 0x0073, 0x0073, 0x0075, 0x0065, 0x0064, 0x0020, 0x0054, 0x006F, 0x0000];
pub static CPP_COM_WSTR_E37B97C1CDF3: [WCHAR; 13] = [0x0043, 0x0065, 0x0072, 0x0074, 0x0069, 0x0066, 0x0069, 0x0063, 0x0061, 0x0074, 0x0065, 0x0073, 0x0000];

use core::ffi::c_void;









#[no_mangle]
pub unsafe extern "system" fn wWinMain(_hInstance: HINSTANCE, _hPrevInstance: HINSTANCE, _lpCmdLine: LPWSTR, nShowCmd: i32) -> i32 {
    __cxx_global_init();
    return (*core::ptr::addr_of_mut!(_AtlModule)).WinMain(nShowCmd);
}

impl CCertMgrProperties {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_UserStoreList.__ctor_0(User);
        (*this_).m_ComputerStoreList.__ctor_0(Computer);
        {
    
        }
    }
    pub unsafe fn OnInitDialog(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let hIcon: HICON = LoadImageW(_AtlBaseModule.GetResourceInstance(), MAKEINTRESOURCEW!(IDI_CERTMGR), IMAGE_ICON, 0, 0, LR_DEFAULTSIZE | LR_SHARED) as HICON;
        (*this_).__base0.SetIcon(hIcon, (TRUE) as BOOL);
        (*this_).__base0.SetIcon(hIcon, (FALSE) as BOOL);
    
        (*this_).m_StoreTree.Attach((*this_).__base0.GetDlgItem((IDC_STORE_TREE as i32)));
        (*this_).m_CertList.Attach((*this_).__base0.GetDlgItem((IDC_CERT_LIST as i32)));
        (*this_).m_Splitter.Init(CWindow__make_0((*this_).__base0.m_hWnd), CWindow__make_0((*this_).m_StoreTree._base.m_hWnd), CWindow__make_0((*this_).m_CertList._base.m_hWnd));
    
        ListView_SetExtendedListViewStyleEx((*this_).m_CertList._base.m_hWnd, LVS_EX_FULLROWSELECT, LVS_EX_FULLROWSELECT);
    
        let certRoot = (*this_).m_StoreTree.AddItem(core::ptr::null_mut(), CPP_COM_WSTR_E37B97C1CDF3.as_ptr() as LPWSTR, 0, 0, 0);
        let userRoot = (*this_).m_StoreTree.AddItem(certRoot, CPP_COM_WSTR_04F93279A3AB.as_ptr() as LPWSTR, 0, 0, 0);
        (*this_).m_UserStoreList.ForEach(
            |current: *mut CStore| {
                (*this_).m_StoreTree.AddItem(userRoot, (*current).GetStoreName() as LPWSTR, 0, 0, current as LPARAM);
            });
        let computerRoot = (*this_).m_StoreTree.AddItem(certRoot, CPP_COM_WSTR_D29EB886FF27.as_ptr() as LPWSTR, 0, 0, 0);
        (*this_).m_ComputerStoreList.ForEach(
            |current: *mut CStore| {
                (*this_).m_StoreTree.AddItem(computerRoot, (*current).GetStoreName() as LPWSTR, 0, 0, current as LPARAM);
            });
    
        (*this_).m_StoreTree.Expand(certRoot, TVE_EXPAND);
    
        (*this_).m_CertList.InsertColumn(0, CPP_COM_WSTR_D90381C6A684.as_ptr() as LPWSTR, LVCFMT_LEFT as c_int, 200);
        (*this_).m_CertList.InsertColumn(1, CPP_COM_WSTR_9C4B70748645.as_ptr() as LPWSTR, LVCFMT_LEFT as c_int, 200);
        (*this_).m_CertList.InsertColumn(2, CPP_COM_WSTR_0BCE0F081BC5.as_ptr() as LPWSTR, LVCFMT_LEFT as c_int, 100);
        (*this_).m_CertList.InsertColumn(3, CPP_COM_WSTR_BA52125E4714.as_ptr() as LPWSTR, LVCFMT_LEFT as c_int, 200);
    
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnEndDialog(&mut self, arg0: WORD, wID: WORD, arg2: HWND, arg3: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).EndDialog((wID as i32) as _);
        return 0;
    }
    pub unsafe fn OnStoreChanged(&mut self, idCtrl: i32, pnmh: LPNMHDR, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if idCtrl == (IDC_STORE_TREE as i32) {
            let pntv: LPNMTREEVIEW = pnmh as LPNMTREEVIEW;
            let store: *mut CStore = (*pntv).itemNew.lParam as *mut CStore;
            if !store.is_null() {
                (*this_).m_CertList.DeleteAllItems();
                (*store).Expand();
                (*store).ForEach(|current: *mut CCert| {
                    let mut item: LV_ITEM = core::mem::zeroed();
                    item.mask = LVIF_TEXT | LVIF_PARAM;
                    item.lParam = current as LPARAM;
                    item.pszText = ((*(*current).GetSubjectName()).as_ptr()) as LPWSTR;
                    item.iItem = (*this_).m_CertList.GetItemCount();
                    item.iItem = (*this_).m_CertList.InsertItem(core::ptr::addr_of_mut!(item));
                    (*this_).m_CertList.SetItemText(item.iItem, 1, (*(*current).GetIssuerName()).as_ptr());
                    let mut notAfter: FILETIME = (*current).GetNotAfter();
                    let mut st: SYSTEMTIME = core::mem::zeroed();
                    FileTimeToSystemTime(core::ptr::addr_of_mut!(notAfter), core::ptr::addr_of_mut!(st));
                    let mut dateStr: [u16; 100] = [0u16; 100];
                    GetDateFormatW(LOCALE_USER_DEFAULT, 0, core::ptr::addr_of_mut!(st), core::ptr::null_mut(), dateStr.as_mut_ptr(), (_countof!(dateStr)) as i32);
                    (*this_).m_CertList.SetItemText(item.iItem, 2, dateStr.as_ptr());
                    (*this_).m_CertList.SetItemText(item.iItem, 3, (*(*current).GetFriendlyName()).as_ptr());
                });
                (*this_).m_SortableHeader.ReapplySorting(&mut (*this_).m_CertList);
            }
        }
        return 0;
    }
    pub unsafe fn OnCertDoubleClicked(&mut self, idCtrl: i32, pnmh: LPNMHDR, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if idCtrl == (IDC_CERT_LIST as i32) {
            let lpnmitem: LPNMITEMACTIVATE = pnmh as LPNMITEMACTIVATE;
            if (*lpnmitem).iItem < 0 {
                return 0;
            }
    
            let current: *mut CCert = (*this_).m_CertList.GetItemData((*lpnmitem).iItem) as *mut CCert;
    
            let mut CertViewInfo: CRYPTUI_VIEWCERTIFICATE_STRUCTW = core::mem::zeroed::<CRYPTUI_VIEWCERTIFICATE_STRUCTW>();
            CertViewInfo.dwSize = core::mem::size_of::<CRYPTUI_VIEWCERTIFICATE_STRUCTW>() as u32;
            CertViewInfo.pCertContext = (*current).GetCertContext();
            CertViewInfo.hwndParent = (*this_).__base0.m_hWnd;
            CryptUIDlgViewCertificateW(core::ptr::addr_of_mut!(CertViewInfo), core::ptr::null_mut());
        }
        return 0;
    }
    pub unsafe fn OnHeaderClicked(&mut self, idCtrl: i32, pnmh: LPNMHDR, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let pnmv: LPNMLISTVIEW = pnmh as LPNMLISTVIEW;
        (*this_).m_SortableHeader.ColumnClick(&mut (*this_).m_CertList, pnmv);
        return 0;
    }
}

impl CCertMgrModule {
    pub unsafe fn Run(&mut self, nShowCmd: i32) -> HRESULT {
        let this_: *mut Self = self;
        let mut Title = CStringW::from_ptr(MAKEINTRESOURCEW!(IDS_PROPERTIES_MAIN_TITLE) as *const WCHAR);
        let mut certMgr = CCertMgrProperties__make_0();
        certMgr.DoModal(GetActiveWindow(), 0);
        return S_OK;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
    }
}
