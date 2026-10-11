use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_B6D752404602: [WCHAR; 10] = [0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x002E, 0x0032, 0x0073, 0x0029, 0x0000];

use core::ffi::c_void;



#[no_mangle]
pub unsafe extern "C" fn SelectDrive(Drive: &mut WCHAR) {
    let mut dlgSelectDrive: CSelectDriveDlg = CSelectDriveDlg__make_0();
    if dlgSelectDrive.DoModal(GetActiveWindow(), 0) == IDOK as INT_PTR {
        *Drive = dlgSelectDrive.m_SelectedDrive;
    }
}

impl CSelectDriveDlg {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_SelectedDrive = UNICODE_NULL;
        {
        }
    }
    pub unsafe fn OnInitDialog(&mut self, arg0: UINT, arg1: WPARAM, arg2: LPARAM, arg3: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        // Try to find an existing instance of this dialog
        let mut buf: [WCHAR; 300] = core::mem::zeroed();
        (*this_).__base0.GetWindowTextW__0(buf.as_mut_ptr(), (_countof!(buf)) as i32);
        let mut hNext: HWND = core::ptr::null_mut();
        let mut hFind: HWND;
        loop {
            hFind = FindWindowExW(core::ptr::null_mut(), hNext, core::ptr::null_mut(), buf.as_mut_ptr());
            if hFind.is_null() {
                break;
            }
            if hFind != (*this_).op_to_HWND() && IsWindowVisible(hFind) != 0 {
                SetForegroundWindow(hFind);
                (*this_).EndDialog(IDCANCEL as INT_PTR);
                return (FALSE) as LRESULT;
            }
            hNext = hFind;
        }
    
        let mut cbo = CWindow { m_hWnd: (*this_).__base0.GetDlgItem((IDC_DRIVES as i32)) };
        let mut VolumeNameBuffer: [WCHAR; (MAX_PATH + 1) as _] = core::mem::zeroed();
        let mut Tmp = CStringW::new();
        let mut Drive: WCHAR = 'A' as u16;
        while Drive <= 'Z' as u16 {
            let mut RootPathName: [WCHAR; 4] = [Drive, ':' as u16, '\\' as u16, 0];
            let Type: UINT = GetDriveTypeW(RootPathName.as_ptr());
            if Type == DRIVE_FIXED {
                GetVolumeInformationW(RootPathName.as_ptr(), VolumeNameBuffer.as_mut_ptr(), _countof!(VolumeNameBuffer), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), 0);
                atl_format!(Tmp, CPP_COM_WSTR_B6D752404602.as_ptr(), VolumeNameBuffer.as_ptr(), RootPathName.as_ptr());
    
                let index = cbo.SendMessageW__0(CB_ADDSTRING, 0 as WPARAM, Tmp.GetString() as LPARAM) as i32;
                cbo.SendMessageW__0(CB_SETITEMDATA, index as WPARAM, Drive as LPARAM);
            }
            Drive += 1;
        }
        cbo.SendMessageW__0(CB_SETCURSEL, 0, 0);
        return 1;
    }
    pub unsafe fn OnEndDialog(&mut self, arg0: WORD, wID: WORD, arg2: HWND, arg3: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut cbo: CWindow = CWindow { m_hWnd: (*this_).__base0.GetDlgItem((IDC_DRIVES as i32)) };
        let __inner: LRESULT = cbo.SendMessageW__0(CB_GETCURSEL, 0, 0);
        (*this_).m_SelectedDrive = cbo.SendMessageW__0(CB_GETITEMDATA, (__inner) as WPARAM, 0) as WCHAR;
        (*this_).EndDialog((wID) as _);
        return 0;
    }
}
