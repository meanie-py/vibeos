use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];

use core::ffi::c_void;

extern "C" {
    fn _AtlModule___ctor_0();
}
extern "system" {
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn EnumWindows( lpEnumFunc: Option<unsafe extern "system" fn(HWND, LPARAM) -> BOOL>, lParam: LPARAM, ) -> BOOL;
}

// LVM_GETITEMW = LVM_FIRST (0x1000) + 75













#[no_mangle]
pub unsafe extern "system" fn wWinMain(_hInstance: HINSTANCE, _hPrevInstance: HINSTANCE, _lpCmdLine: LPWSTR, nShowCmd: i32) -> i32 {
    __cxx_global_init();
    return (*core::ptr::addr_of_mut!(_AtlModule)).WinMain(nShowCmd);
}
#[no_mangle]
pub unsafe extern "C" fn __cxx_global_init() {
    // `CCleanMgrModule _AtlModule;` -- its constructor, which cppapp generates.
    (*core::ptr::addr_of_mut!(_AtlModule)).__ctor_0();
}

impl CCleanMgrProperties {
    pub unsafe fn __ctor_0(&mut self, Drive: WCHAR, TotalSpaceUsed: DWORDLONG, handlerList: *mut CCleanupHandlerList) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_Drive = Drive;
        (*this_).m_TotalSpaceUsed = TotalSpaceUsed;
        (*this_).m_HandlerList = handlerList;
        (*this_).m_IgnoreChanges = true;
        {
        }
    }
    pub unsafe fn OnApply(&mut self) -> i32 {
        let this_: *mut Self = self;
        let Title = CStringW::from_resource(IDS_DISK_CLEANUP);
        let Text = CStringW::from_resource(IDS_CONFIRM_DELETE);
    
        if (*this_).__base0.MessageBoxW(Text.GetString(), Title.GetString(), MB_YESNO | MB_ICONQUESTION) != (IDYES as i32) {
            return (PSNRET_INVALID as i32);
        }
    
        return (PSNRET_NOERROR as i32);
    }
    pub unsafe fn OnInitDialog(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let hIcon: HICON = LoadImageW(
            _AtlBaseModule.GetResourceInstance(), MAKEINTRESOURCEW!(IDI_CLEANMGR), IMAGE_ICON, 0, 0,
            LR_DEFAULTSIZE | LR_SHARED) as HICON;
        (*this_).__base0.SendDlgItemMessageW((IDC_DISKICON as i32), STM_SETICON, hIcon as WPARAM, 0);
    
        (*this_).m_HandlerListControl = CWindow__make_0((*this_).__base0.GetDlgItem((IDC_HANDLERLIST as i32)));
        let mut rc: RECT = core::mem::zeroed();
        (*this_).m_HandlerListControl.GetClientRect(core::ptr::addr_of_mut!(rc));
        rc.right -= GetSystemMetrics((SM_CXVSCROLL as i32));
    
        let mut column: LV_COLUMN = core::mem::zeroed();
        column.mask = LVCF_FMT | LVCF_WIDTH;
        column.fmt = (LVCFMT_LEFT as i32);
        column.cx = rc.right * 80 / 100;
        ListView_InsertColumn!((*this_).m_HandlerListControl.op_to_HWND(), 0, core::ptr::addr_of_mut!(column));
        column.fmt = (LVCFMT_RIGHT as i32);
        column.cx = rc.right * 20 / 100;
    
        ListView_InsertColumn!((*this_).m_HandlerListControl.op_to_HWND(), 1, core::ptr::addr_of_mut!(column));
        let hImagelist: HIMAGELIST = ImageList_Create(16, 16, ILC_MASK | ILC_COLOR32, 1, 1);
        ListView_SetImageList!((*this_).m_HandlerListControl.op_to_HWND(), hImagelist, LVSIL_SMALL);
    
        ListView_SetExtendedListViewStyleEx!((*this_).m_HandlerListControl.op_to_HWND(), LVS_EX_CHECKBOXES | LVS_EX_FULLROWSELECT, LVS_EX_CHECKBOXES | LVS_EX_FULLROWSELECT);
    
        (*(*this_).m_HandlerList).ForEach(|current: *mut CCleanupHandler| {
            if !(*current).ShowHandler {
                return;
            }
    
            let mut item: LV_ITEM = core::mem::zeroed();
            item.mask = LVIF_TEXT | LVIF_IMAGE | LVIF_PARAM;
            item.lParam = current as LPARAM;
            item.pszText = (*current).wszDisplayName.op_to_LPWSTR();
            item.iItem = ListView_GetItemCount!((*this_).m_HandlerListControl.op_to_HWND());
            item.iImage = ImageList_AddIcon!(hImagelist, (*current).hIcon);
            item.iItem = ListView_InsertItem!((*this_).m_HandlerListControl.op_to_HWND(), core::ptr::addr_of_mut!(item));
            ListView_SetCheckState!(
                (*this_).m_HandlerListControl.op_to_HWND(), item.iItem, ((((*current).StateFlags & (HANDLER_STATE_SELECTED as u32)) != 0) as BOOL));
    
            item.mask = LVIF_TEXT;
            let mut ByteSize: [WCHAR; 100] = [0; 100];
            StrFormatByteSizeW(((*current).SpaceUsed) as LONGLONG, ByteSize.as_mut_ptr(), _countof!(ByteSize));
            ListView_SetItemText!((*this_).m_HandlerListControl.op_to_HWND(), item.iItem, 1, ByteSize.as_mut_ptr());
        });
    
        // Now we should start responding to changes
        (*this_).m_IgnoreChanges = false;
    
        // Select the first item
        ListView_SetItemState!((*this_).m_HandlerListControl.op_to_HWND(), 0, LVIS_SELECTED | LVIS_FOCUSED, LVIS_SELECTED | LVIS_FOCUSED);
    
        (*this_).UpdateSpaceUsed();
        return (TRUE) as LRESULT;
    }
    pub unsafe fn GetHandler(&mut self, Index: i32) -> *mut CCleanupHandler {
        let this_: *mut Self = self;
        let mut item: LVITEMW = core::mem::zeroed();
        item.iItem = Index;
        if item.iItem >= 0 {
            item.mask = LVIF_PARAM;
            // (ListView_GetItem)(this_->m_HandlerListControl, &item) expands to a LVM_GETITEMW message send
            SendMessageW(
                (*this_).m_HandlerListControl.op_to_HWND(),
                LVM_GETITEMW,
                0,
                &mut item as *mut LVITEMW as LPARAM,
            );
            return item.lParam as *mut CCleanupHandler;
        }
        core::ptr::null_mut()
    }
    pub unsafe fn OnDetails(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        // ListView_GetNextItem(hwnd, i, flags) == SendMessageW(hwnd, LVM_GETNEXTITEM, (WPARAM)(int)i, MAKELPARAM(flags, 0))
        // LVM_GETNEXTITEM = LVM_FIRST (0x1000) + 12 = 0x100C
        let handler_index: i32 = SendMessageW(
            (*this_).m_HandlerListControl.m_hWnd,
            0x100C,
            (-(1 as i32)) as WPARAM,
            LVIS_FOCUSED as LPARAM,
        ) as i32;
        let handler: *mut CCleanupHandler = (*this_).GetHandler(handler_index);
        if !handler.is_null() {
            let _ = (*(*handler).Handler.p).ShowProperties((*this_).__base0.m_hWnd);
        }
        return 0 as LRESULT;
    }
    pub unsafe fn OnHandlerItemchanged(&mut self, idCtrl: i32, pnmh: LPNMHDR, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if idCtrl == (IDC_HANDLERLIST as i32) {
            // We are still initializing, don't respond to changes just yet!
            if (*this_).m_IgnoreChanges {
                return 0;
            }
    
            let pnic: LPNMLISTVIEW = pnmh as LPNMLISTVIEW;
    
            // We only care about state changes
            if !(((*pnic).uChanged & LVIF_STATE) != 0) {
                return 0;
            }
    
            let ItemIndex: INT = (*pnic).iItem;
            if ItemIndex == -1 || ItemIndex >= ListView_GetItemCount!((*pnic).hdr.hwndFrom) {
                return 0;
            }
    
            let GotSelected: bool = (((*pnic).uNewState & LVIS_SELECTED) != 0) && !(((*pnic).uOldState & LVIS_SELECTED) != 0);
            if GotSelected {
                let mut DetailsButton: CWindow = CWindow__make_0((*this_).__base0.GetDlgItem((IDC_DETAILS as i32)));
                let handler: *mut CCleanupHandler = (*pnic).lParam as *mut CCleanupHandler;
    
                let description = if !(*handler).wszDescription.is_null() {
                    (*handler).wszDescription.m_pData as *const u16
                } else {
                    CPP_COM_WSTR_96A296D224F2.as_ptr()
                };
                (*this_).__base0.SetDlgItemTextW((IDC_DESCRIPTION as i32), description);
                if (*handler).HasSettings() != 0 {
                    DetailsButton.ShowWindow((SW_SHOW as i32));
                    DetailsButton.SetWindowTextW((*handler).wszBtnText.m_pData);
                } else {
                    DetailsButton.ShowWindow((SW_HIDE as i32));
                }
            }
    
            let iOldState: i32 = STATEIMAGETOINDEX!((*pnic).uOldState) as i32;
            let iNewState: i32 = STATEIMAGETOINDEX!((*pnic).uNewState) as i32;
    
            if (iOldState ^ iNewState) == (STATEIMAGE_UNCHECKED ^ STATEIMAGE_CHECKED) {
                let handler: *mut CCleanupHandler = (*pnic).lParam as *mut CCleanupHandler;
                if iNewState == STATEIMAGE_CHECKED {
                    (*handler).StateFlags |= (HANDLER_STATE_SELECTED as u32);
                } else {
                    (*handler).StateFlags &= (!HANDLER_STATE_SELECTED) as u32;
                }
                (*this_).UpdateSpaceUsed();
            }
        }
        return 0;
    }
    pub unsafe fn UpdateSpaceUsed(&mut self) {
        let this_: *mut Self = self;
            let mut tmp = CStringW::new();
            let mut ByteSize: [WCHAR; 100] = [0; 100];
            StrFormatByteSizeW(((*this_).m_TotalSpaceUsed) as LONGLONG, ByteSize.as_mut_ptr(), _countof!(ByteSize));
    
            atl_format_id!(tmp, IDS_TOTAL_CLEANABLE_CAPTION, ByteSize.as_ptr(), (*this_).m_Drive);
            (*this_).__base0.SetDlgItemTextW((IDC_TOTAL_CLEANABLE as i32), tmp.GetString());
    
            let mut SelectedGained: DWORDLONG = 0;
    
            (*(*this_).m_HandlerList).ForEach(
                |current: *mut CCleanupHandler| {
                    if ((*current).StateFlags & (HANDLER_STATE_SELECTED as u32)) != 0 {
                        SelectedGained += (*current).SpaceUsed;
                    }
                });
    
            StrFormatByteSizeW((SelectedGained) as LONGLONG, ByteSize.as_mut_ptr(), _countof!(ByteSize));
            (*this_).__base0.SetDlgItemTextW((IDC_SELECTED_GAINED as i32), ByteSize.as_ptr());
        }
}

impl CCleanMgrModule {
    pub unsafe fn ParseCommandLine(&mut self, lpCmdLine: LPCTSTR, pnRetCode: *mut HRESULT) -> bool {
        let this_: *mut Self = self;
        let mut argc: i32 = 0;
        // cleanmgr.h's `CLocalPtr<T>` is `CHeapPtr<T, CLocalAllocator>` plus its constructors:
        // the argv block is LocalFree'd when it goes out of scope, as CommandLineToArgvW wants.
        let argv: CHeapPtr<LPWSTR, CLocalAllocator> =
            CHeapPtr::from_raw(CommandLineToArgvW(lpCmdLine, core::ptr::addr_of_mut!(argc)));
        // `argv[n][i]`
        let arg = |n: i32, i: usize| -> WCHAR { *(*argv.m_pData.add(n as usize)).add(i) };
    
        let mut n: i32 = 1;
        while n < argc {
            'body: {
                if ((arg(n, 0) == '/' as u16) || (arg(n, 0) == '-' as u16)) && (towlower(arg(n, 1)) == 'd' as u16) {
                    if iswalpha(arg(n, 2)) != 0 {
                        (*this_).m_Drive = (towupper(arg(n, 2) as _)) as u16;
                        break 'body;
                    }
                    if (n + 1) < argc {
                        (*this_).m_Drive = (towupper(arg(n + 1, 0) as _)) as u16;
                        n += 1;
                        break 'body;
                    }
                }
            }
            n += 1;
        }
        *pnRetCode = S_OK;
        return true;
    }
    pub unsafe fn GetWindowProcessId(hWnd: HWND) -> UINT {
        let mut pid: DWORD = 0;
        if GetWindowThreadProcessId(hWnd, core::ptr::addr_of_mut!(pid)) != 0 {
            pid
        } else {
            0
        }
    }
    pub unsafe extern "system" fn EnumSingleInstanceCallback(hWnd: HWND, lParam: LPARAM) -> BOOL {
        if IsWindowVisible(hWnd) != 0 && (CCleanMgrModule::GetWindowProcessId(hWnd) as LPARAM) == lParam {
            SetForegroundWindow(hWnd);
            return (FALSE) as BOOL;
        }
        return (TRUE) as BOOL;
    }
    pub unsafe fn Run(&mut self, nShowCmd: i32) -> HRESULT {
        let this_: *mut Self = self;
        let _ = nShowCmd;
        if (*this_).m_Drive == UNICODE_NULL {
            SelectDrive(&mut (*this_).m_Drive);
        }
    
        if (*this_).m_Drive == UNICODE_NULL {
            return E_FAIL;
        }
    
        let mut Title = CStringW::new();
        atl_format_id!(Title, IDS_PROPERTIES_MAIN_TITLE, (*this_).m_Drive);
    
        let hWndInstance: HWND = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            WC_STATIC,
            Title.GetString(),
            0,
            0,
            0,
            0,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        let mut hNext: HWND = core::ptr::null_mut();
        let mut hFind: HWND;
        loop {
            hFind = FindWindowExW(
                core::ptr::null_mut(),
                hNext,
                WC_STATIC,
                Title.GetString(),
            );
            if hFind.is_null() {
                break;
            }
            if hFind != hWndInstance {
                EnumWindows(
                    Some(CCleanMgrModule::EnumSingleInstanceCallback),
                    (CCleanMgrModule::GetWindowProcessId(hFind)) as _,
                );
                return S_FALSE;
            }
            hNext = hFind;
        }
    
        let mut Handlers: CCleanupHandlerList = CCleanupHandlerList__make_0();
        let mut CacheCallBack: CEmptyVolumeCacheCallBack = CEmptyVolumeCacheCallBack__make_0();
    
        Handlers.LoadHandlers((*this_).m_Drive);
        let TotalSpaceUsed: DWORDLONG =
            Handlers.ScanDrive(core::ptr::addr_of_mut!(CacheCallBack.__base0));
    
        let mut cleanMgr: CCleanMgrProperties = CCleanMgrProperties__make_0(
            (*this_).m_Drive,
            TotalSpaceUsed,
            core::ptr::addr_of_mut!(Handlers),
        );
        let mut hpsp: [HPROPSHEETPAGE; 1] = core::mem::zeroed();
        hpsp[0] = cleanMgr.Create();
    
        let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
        psh.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as u32;
        psh.dwFlags = PSH_NOAPPLYNOW | PSH_USEICONID | PSH_NOCONTEXTHELP;
        psh.hInstance = _AtlBaseModule.GetResourceInstance();
        psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDI_CLEANMGR);
        psh.pszCaption = Title.GetString();
        psh.nPages = _countof!(hpsp) as u32;
        psh.__bindgen_anon_3.phpage = hpsp.as_mut_ptr();
    
        if PropertySheetW(core::ptr::addr_of_mut!(psh)) >= 1 {
            // Allow new "cleanmgr /D" without waiting for these handlers
            DestroyWindow(hWndInstance);
            Handlers.ExecuteCleanup(core::ptr::addr_of_mut!(CacheCallBack.__base0));
        } else {
            DestroyWindow(hWndInstance);
        }
        return S_OK;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_Drive = UNICODE_NULL;
    }
}
