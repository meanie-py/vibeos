use crate::prelude::*;
use core::ffi::c_void;













impl CPaintToolBar {
    pub unsafe fn DoCreate(&mut self, hwndParent: HWND) -> BOOL {
        let this_: *mut Self = self;
        // NOTE: The horizontal line above the toolbar is hidden by CCS_NODIVIDER style.
        let toolbarPos: RECT = RECT {
            left: 0,
            top: 0,
            right: (CX_TOOLBAR + (2 * GetSystemMetrics((SM_CXBORDER) as i32)) as u32) as i32,
            bottom: (CY_TOOLBAR + (2 * GetSystemMetrics((SM_CYBORDER) as i32)) as u32) as i32,
        };
        let style: DWORD = WS_CHILD | WS_VISIBLE | CCS_NOPARENTALIGN | CCS_VERT | CCS_NORESIZE |
                      TBSTYLE_TOOLTIPS | TBSTYLE_FLAT;
        if (!(*this_).__base0.Create(TOOLBARCLASSNAMEW, hwndParent, toolbarPos, core::ptr::null_mut(), style, 0, 0, core::ptr::null_mut()).is_null()) == false {
            return (FALSE) as BOOL;
        }
    
        let hImageList: HIMAGELIST = ImageList_Create(16, 16, ILC_COLOR24 | ILC_MASK, 16, 0);
        (*this_).__base0.SendMessageW__0(TB_SETIMAGELIST, 0 as WPARAM, hImageList as LPARAM);
    
        let hbmIcons: HBITMAP = LoadImageW(g_hinstExe, MAKEINTRESOURCEW!(IDB_TOOLBARICONS),
                                                 IMAGE_BITMAP, 256, 16, 0) as HBITMAP;
        ImageList_AddMasked(hImageList, hbmIcons, RGB!(255, 0, 255));
        DeleteObject((hbmIcons).cast());
    
        (*this_).__base0.SendMessageW__0(TB_BUTTONSTRUCTSIZE, core::mem::size_of::<TBBUTTON>() as WPARAM, 0);
    
        let mut szToolTip: [u16; 30] = [0; 30];
        let mut tbbutton: TBBUTTON = core::mem::zeroed();
        ZeroMemory!(core::ptr::addr_of_mut!(tbbutton) as *mut core::ffi::c_void, core::mem::size_of_val(&tbbutton));
        tbbutton.fsStyle = (TBSTYLE_CHECKGROUP as u8);
        let mut i: INT = 0;
        while i < (NUM_TOOLS as i32) {
            LoadStringW(g_hinstExe, IDS_TOOLTIP1 + (i) as u32, szToolTip.as_mut_ptr(), (_countof!(szToolTip)) as i32);
            tbbutton.iString = szToolTip.as_mut_ptr() as INT_PTR;
            tbbutton.fsState = (TBSTATE_ENABLED | (if i % 2 == 1 { TBSTATE_WRAP } else { 0 })) as u8;
            tbbutton.idCommand = (ID_FREESEL + (i) as u32) as i32;
            tbbutton.iBitmap = i;
            (*this_).__base0.SendMessageW__0((TB_ADDBUTTONS as u32), 1 as WPARAM, core::ptr::addr_of_mut!(tbbutton) as LPARAM);
            i += 1;
        }
    
        (*this_).__base0.SendMessageW__0(TB_CHECKBUTTON, ID_PEN as WPARAM, MAKELPARAM!(TRUE, 0));
        (*this_).__base0.SendMessageW__0(TB_SETMAXTEXTROWS, 0 as WPARAM, 0);
        (*this_).__base0.SendMessageW__0(TB_SETBUTTONSIZE, 0 as WPARAM, MAKELPARAM!(CXY_TB_BUTTON, CXY_TB_BUTTON));
    
        (*this_).__base0.SetWindowLongPtr(GWLP_USERDATA, (*this_).__base0.SetWindowLongPtr(GWLP_WNDPROC, CPaintToolBar::ToolBarWndProc as LONG_PTR));
        return (TRUE) as BOOL;
    }
    pub unsafe fn ToolBarWndProc(hwnd: HWND, uMsg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let oldWndProc: WNDPROC = core::mem::transmute::<isize, WNDPROC>((GetWindowLongPtr(hwnd, GWLP_USERDATA)) as _);
        if uMsg == WM_LBUTTONUP {
            // We have to detect clicking on toolbar even if no change of pressed button
            let mut pt: POINT = POINT { x: GET_X_LPARAM!(lParam), y: GET_Y_LPARAM!(lParam) };
            let index: i32 = SendMessageW(hwnd, TB_HITTEST, 0, core::ptr::addr_of_mut!(pt) as LPARAM) as i32;
            if index >= 0 {
                let mut button: TBBUTTON = core::mem::zeroed();
                if SendMessageW(hwnd, TB_GETBUTTON, index as WPARAM, core::ptr::addr_of_mut!(button) as LPARAM) != 0 {
                    PostMessageW(GetParent(hwnd), WM_COMMAND, button.idCommand as WPARAM, 0);
                }
            }
        }
        return CallWindowProc(oldWndProc, hwnd, uMsg, wParam, lParam);
    }
}

impl CToolBox {
    pub unsafe fn DoCreate(&mut self, hwndParent: HWND) -> BOOL {
        let this_: *mut Self = self;
        // Rely on mainWindow's WM_SIZE
        let mut rcToolBox: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let style: DWORD = WS_CHILD
            | (if registrySettings.ShowToolBox != 0 {
                WS_VISIBLE
            } else {
                0
            });
        return ((*this_).Create(hwndParent, &mut rcToolBox, core::ptr::null_mut(), style) != core::ptr::null_mut()) as BOOL;
    }
    pub unsafe fn OnCreate(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).toolbar.DoCreate((*this_).__base0.m_hWnd);
        toolSettingsWindow.DoCreate((*this_).__base0.m_hWnd);
        return 0;
    }
    pub unsafe fn OnSysColorChange(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).toolbar.__base0.SendMessageW__0(WM_SYSCOLORCHANGE, 0, 0);
        return 0;
    }
    pub unsafe fn OnCommand(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let id: UINT = LOWORD!(wParam) as UINT;
        let mut i: usize = 0;
        while i < _countof!(CommandToToolMapping) {
            if CommandToToolMapping[i].id == id {
                toolsModel.SetActiveTool(CommandToToolMapping[i].tool);
                break;
            }
            i += 1;
        }
        return 0;
    }
    pub unsafe fn OnLButtonDown(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.SetCapture();
        return 0;
    }
    pub unsafe fn OnMouseMove(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if GetCapture() != (*this_).__base0.m_hWnd {
            return 0;
        }
        let mut pt: POINT = POINT { x: GET_X_LPARAM!(lParam), y: GET_Y_LPARAM!(lParam) };
        (*this_).__base0.ClientToScreen__0(core::ptr::addr_of_mut!(pt));
    
        let mut rc: RECT = core::mem::zeroed();
        mainWindow.__base0.GetWindowRect(core::ptr::addr_of_mut!(rc));
    
        let ptCenter: POINT = POINT {
            x: (rc.left + rc.right) / 2,
            y: (rc.top + rc.bottom) / 2,
        };
    
        let dwExpectedBar2ID: DWORD = if pt.x < ptCenter.x { BAR2ID_LEFT } else { BAR2ID_RIGHT };
    
        if registrySettings.Bar2ID != dwExpectedBar2ID {
            registrySettings.Bar2ID = dwExpectedBar2ID;
            mainWindow.__base0.PostMessageW(WM_SIZE, 0, 0);
        }
    
        return 0;
    }
    pub unsafe fn OnLButtonUp(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if GetCapture() != (*this_).__base0.m_hWnd {
            return 0;
        }
    
        ReleaseCapture();
        return 0;
    }
    pub unsafe fn OnToolsModelToolChanged(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        selectionModel.HideSelection();
        toolsModel.resetTool(); // resets the point-buffer of the polygon and bezier functions
    
        // Check the toolbar button
        let tool: TOOLTYPE = toolsModel.GetActiveTool();
        let mut i: usize = 0;
        while i < _countof!(CommandToToolMapping) {
            if CommandToToolMapping[i].tool == tool {
                (*this_).toolbar.__base0.SendMessageW__0(TB_CHECKBUTTON, ((CommandToToolMapping[i].id) as UINT) as WPARAM, ((TRUE) as WPARAM) as LPARAM);
                break;
            }
            i += 1;
        }
    
        return 0;
    }
}

impl COMMAND_TO_TOOL {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
