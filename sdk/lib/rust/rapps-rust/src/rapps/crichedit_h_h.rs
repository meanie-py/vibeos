use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_589443647ECA: [WCHAR; 12] = [0x0052, 0x0069, 0x0063, 0x0068, 0x0045, 0x0064, 0x0069, 0x0074, 0x0032, 0x0030, 0x0057, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_A5D19B228349: [WCHAR; 13] = [0x0072, 0x0069, 0x0063, 0x0068, 0x0065, 0x0064, 0x0032, 0x0030, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];

use core::ffi::c_void;













impl CRichEdit {
    pub unsafe fn GenericInsertText(&mut self, lpszText: LPCWSTR, InsertedTextLen: SIZE_T, dwEffects: DWORD) {
        let this_: *mut Self = self;
        let mut SetText: SETTEXTEX = core::mem::zeroed();
        let Len: SIZE_T = (*this_).GetTextLen() as SIZE_T;

        if InsertedTextLen != 0 {
            /* Insert new text */
            SetText.flags = ST_SELECTION;
            SetText.codepage = 1200;

            (*this_).__base0.SendMessageW__0(EM_SETTEXTEX, (core::ptr::addr_of_mut!(SetText) as usize) as WPARAM, lpszText as LPARAM);

            if (dwEffects == CFM_LINK) && (PathIsURLW(lpszText) == 0) {
                // if text is not an URL, no styling is used
                (*this_).SetRangeFormatting(Len, Len + InsertedTextLen, 0);
            } else {
                (*this_).SetRangeFormatting(Len, Len + InsertedTextLen, dwEffects);
            }
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_LoadedLibrary = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn SetRangeFormatting(&mut self, Start: SIZE_T, End: SIZE_T, dwEffects: DWORD) {
        let this_: *mut Self = self;
        let mut CharFormat: CHARFORMAT2W = core::mem::zeroed();

        (*this_).__base0.SendMessageW__0(EM_SETSEL, Start, (End) as LPARAM);

        ZeroMemory!(core::ptr::addr_of_mut!(CharFormat) as *mut _, core::mem::size_of::<CHARFORMAT2W>());

        CharFormat.cbSize = core::mem::size_of::<CHARFORMAT2W>() as u32;
        CharFormat.dwMask = dwEffects;
        CharFormat.dwEffects = dwEffects;

        (*this_).__base0.SendMessageW__0(EM_SETCHARFORMAT, (SCF_WORD | SCF_SELECTION) as WPARAM, core::ptr::addr_of_mut!(CharFormat) as LPARAM);

        (*this_).__base0.SendMessageW__0(EM_SETSEL, End, (End + 1) as LPARAM);
    }
    pub unsafe fn GetTextLen(&mut self) -> LONG {
        let this_: *mut Self = self;
            let mut TxtLenStruct: GETTEXTLENGTHEX = core::mem::zeroed();

            TxtLenStruct.flags = GTL_NUMCHARS;
            TxtLenStruct.codepage = 1200;

            return (*this_).__base0.SendMessageW__0(EM_GETTEXTLENGTHEX, core::ptr::addr_of_mut!(TxtLenStruct) as WPARAM, 0) as LONG;
        }
    pub unsafe fn InsertText__0(&mut self, lpszText: LPCWSTR, dwEffects: DWORD) {
        let this_: *mut Self = self;
        (*this_).GenericInsertText(lpszText, (wcslen(lpszText)) as SIZE_T, dwEffects);
    }
    pub unsafe fn InsertText__1(&mut self, szText: &CStringW, dwEffects: DWORD) {
        let this_: *mut Self = self;
        (*this_).GenericInsertText(szText.GetString(), szText.GetLength() as SIZE_T, dwEffects);
    }
    pub unsafe fn SetText__0(&mut self, lpszText: LPCWSTR, dwEffects: DWORD) {
        let this_: *mut Self = self;
        (*this_).__base0.SetWindowTextW(CPP_COM_WSTR_96A296D224F2.as_ptr());
        (*this_).InsertText__0(lpszText, dwEffects);
    }
    pub unsafe fn SetText__1(&mut self, szText: &CStringW, dwEffects: DWORD) {
        let this_: *mut Self = self;
        (*this_).SetText__0(szText.GetString(), dwEffects);
    }
    pub unsafe fn Create(&mut self, hwndParent: HWND) -> HWND {
        let this_: *mut Self = self;
        (*this_).m_LoadedLibrary = (LoadLibraryW(CPP_COM_WSTR_A5D19B228349.as_ptr())).cast();

        (*this_).__base0.m_hWnd = CreateWindowExW(
            0, CPP_COM_WSTR_589443647ECA.as_ptr(), core::ptr::null_mut(), WS_CHILD | WS_VISIBLE | ES_MULTILINE | ES_LEFT | ES_READONLY, 205, 28, 465, 100,
            hwndParent, core::ptr::null_mut(), _AtlBaseModule.GetModuleInstance(), core::ptr::null_mut());

        if (*this_).__base0.m_hWnd as usize != 0 {
            (*this_).__base0.SendMessageW__0(EM_SETBKGNDCOLOR, 0, (GetSysColor((COLOR_BTNFACE as i32))) as LPARAM);
            (*this_).__base0.SendMessageW__0(WM_SETFONT, GetStockObject((DEFAULT_GUI_FONT as i32)) as WPARAM, 0);
            (*this_).__base0.SendMessageW__0(EM_SETEVENTMASK, 0, (ENM_LINK | ENM_MOUSEEVENTS) as LPARAM);
            (*this_).__base0.SendMessageW__0(EM_SHOWSCROLLBAR, (SB_VERT) as WPARAM, (TRUE) as LPARAM);
        }

        return (*this_).__base0.m_hWnd;
    }
    pub unsafe fn OnLink__body(&mut self, Link: *mut ENLINK) {
        let this_: *mut Self = self;}
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_LoadedLibrary.is_null() {
            FreeLibrary((*this_).m_LoadedLibrary);
        }
    }
}
