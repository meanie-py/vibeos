use crate::prelude::*;
use core::ffi::c_void;




impl COptionsDialog {
    pub unsafe fn __ctor_0(&mut self, MainWnd: *mut CMainWnd, console: *mut CConsoleWnd) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        {
            (*this_).m_MainWnd = MainWnd;
            (*this_).m_Console = console;
            (*this_).m_DocumentMode = (*(*this_).m_MainWnd).GetDocumentMode();
            (*this_).m_LogicalReadOnly = (*(*this_).m_MainWnd).GetLogicalReadOnly();
            (*this_).m_PreventViewCustomization = (*(*this_).m_MainWnd).GetPreventViewCustomization();
        }
    }
    pub unsafe fn OnInitDialog(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.CenterWindow((*(*this_).m_Console).__base0.m_hWnd);
    
        (*this_).m_ConsoleIcon.Attach((*this_).__base0.GetDlgItem((IDC_CONSOLEICON as i32)));
        (*this_).m_ConsoleName.Attach((*this_).__base0.GetDlgItem((IDC_CONSOLENAME as i32)));
        (*this_).m_ConsoleSymbol.Attach((*this_).__base0.GetDlgItem((IDC_CONSOLESYMBOL as i32)));
        (*this_).m_ConsoleMode.Attach((*this_).__base0.GetDlgItem((IDC_CONSOLEMODE as i32)));
        (*this_).m_ConsoleDescription.Attach((*this_).__base0.GetDlgItem((IDC_CONSOLEDESCRIPTION as i32)));
        (*this_).m_ConsoleNoSave.Attach((*this_).__base0.GetDlgItem((IDC_CONSOLENOSAVE as i32)));
        (*this_).m_ConsoleCustom.Attach((*this_).__base0.GetDlgItem((IDC_CONSOLECUSTOM as i32)));
    
        (*this_).m_AppIcon = LoadIcon(_AtlBaseModule.GetModuleInstance(), MAKEINTRESOURCE!(IDI_MAINAPP));
        (*this_).m_ConsoleIcon.SendMessageW__0(STM_SETIMAGE, (IMAGE_ICON) as WPARAM, (*this_).m_AppIcon as LPARAM);
    
        (*this_).m_ConsoleName.SetWindowTextW((*(*(*this_).m_MainWnd).GetConsoleTitleW()).GetString());
    
        (*this_).m_ConsoleSymbol.EnableWindow((FALSE as i32));
    
        let mut AuthorMode = CStringW::from_ptr(MAKEINTRESOURCE!(IDS_AUTHORMODE));
        (*this_).m_ConsoleMode.SendMessageW__0(CB_ADDSTRING, 0, AuthorMode.GetString() as LPARAM);
    
        let mut UserModeFull = CStringW::from_ptr(MAKEINTRESOURCE!(IDS_USERMODE_FULL));
        (*this_).m_ConsoleMode.SendMessageW__0(CB_ADDSTRING, 0, UserModeFull.GetString() as LPARAM);
    
        let mut UserModeMultiple = CStringW::from_ptr(MAKEINTRESOURCE!(IDS_USERMODE_MULTIPLE));
        (*this_).m_ConsoleMode.SendMessageW__0(CB_ADDSTRING, 0, UserModeMultiple.GetString() as LPARAM);
    
        let mut UserModeSingle = CStringW::from_ptr(MAKEINTRESOURCE!(IDS_USERMODE_SINGLE));
        (*this_).m_ConsoleMode.SendMessageW__0(CB_ADDSTRING, 0, UserModeSingle.GetString() as LPARAM);
    
        (*this_).m_ConsoleMode.SendMessageW__0(CB_SETCURSEL, (*this_).m_DocumentMode as WPARAM, 0);
    
        let mut ModeDescription = CStringW::from_ptr(MAKEINTRESOURCE!(IDS_AUTHORMODE_DESC + (*this_).m_DocumentMode as UINT));
        (*this_).m_ConsoleDescription.SetWindowTextW(ModeDescription.GetString());
    
        (*this_).m_ConsoleNoSave.EnableWindow(((*this_).m_DocumentMode != DocumentMode_Author) as BOOL);
        if (*this_).m_DocumentMode != DocumentMode_Author {
            (*this_).__base0.CheckDlgButton((IDC_CONSOLENOSAVE as i32), if (*this_).m_LogicalReadOnly != 0 { BST_UNCHECKED } else { BST_CHECKED });
        }
    
        (*this_).m_ConsoleCustom.EnableWindow(((*this_).m_DocumentMode != DocumentMode_Author) as BOOL);
        if (*this_).m_DocumentMode != DocumentMode_Author {
            (*this_).__base0.CheckDlgButton((IDC_CONSOLECUSTOM as i32), if (*this_).m_PreventViewCustomization != 0 { BST_UNCHECKED } else { BST_CHECKED });
        }
    
        return 0;
    }
    pub unsafe fn OnCommand(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        match (wID) as u32 {
            IDC_CONSOLEMODE => {
                {
                    (*this_).m_DocumentMode = (*this_).m_ConsoleMode.SendMessageW__0(CB_GETCURSEL, 0, 0) as DOCUMENT_MODE;
    
                    let mut ModeDescription = CStringW::from_resource((IDS_AUTHORMODE_DESC + (*this_).m_DocumentMode as UINT) as UINT);
                    (*this_).m_ConsoleDescription.SetWindowTextW(ModeDescription.GetString());
                    (*this_).m_ConsoleNoSave.EnableWindow(((*this_).m_DocumentMode != DocumentMode_Author) as BOOL);
                    if (*this_).m_DocumentMode == DocumentMode_Author {
                        (*this_).__base0.CheckDlgButton((IDC_CONSOLENOSAVE as i32), BST_UNCHECKED);
                    }
                    (*this_).m_ConsoleCustom.EnableWindow(((*this_).m_DocumentMode != DocumentMode_Author) as BOOL);
                    if (*this_).m_DocumentMode == DocumentMode_Author {
                        (*this_).__base0.CheckDlgButton((IDC_CONSOLECUSTOM as i32), BST_UNCHECKED);
                    }
                }
                return 0;
            }
    
            IDOK => {
                {
                    let mut consoleTitle = CStringW::new();
                    (*this_).m_ConsoleName.GetWindowTextW__1(&mut consoleTitle);
                    (*(*this_).m_MainWnd).SetConsoleTitleW(consoleTitle);
    
                    (*(*this_).m_MainWnd).SetDocumentMode((*this_).m_DocumentMode);
                    if (*this_).m_DocumentMode == DocumentMode_Author {
                        (*(*this_).m_MainWnd).SetLogicalReadOnly((FALSE) as BOOL);
                        (*(*this_).m_MainWnd).SetPreventViewCustomization((FALSE) as BOOL);
                    } else {
                        (*(*this_).m_MainWnd).SetLogicalReadOnly(
                            if (*this_).__base0.IsDlgButtonChecked((IDC_CONSOLENOSAVE as i32)) == BST_CHECKED {
                                (FALSE as i32)
                            } else {
                                (TRUE as i32)
                            },
                        );
                        (*(*this_).m_MainWnd).SetPreventViewCustomization(
                            if (*this_).__base0.IsDlgButtonChecked((IDC_CONSOLECUSTOM as i32)) == BST_CHECKED {
                                (FALSE as i32)
                            } else {
                                (TRUE as i32)
                            },
                        );
                    }
    
                    (*this_).EndDialog((IDOK as _));
                }
                return 0;
            }
    
            IDCANCEL => {
                (*this_).EndDialog((IDCANCEL as _));
                return 0;
            }
    
            _ => {}
        }
    
        return 0;
    }
}
