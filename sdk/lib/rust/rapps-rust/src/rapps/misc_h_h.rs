use crate::prelude::*;
use core::ffi::c_void;










pub unsafe extern "C" fn HResultFromWin32(Error: UINT) -> HRESULT {
    HRESULT_FROM_WIN32!(Error)
}

pub unsafe extern "C" fn ErrorFromHResult(hr: HRESULT) -> UINT {
    // Attempt to extract the original Win32 error code from the HRESULT
    if HIWORD!(hr as u32) == HIWORD!(HRESULT_FROM_WIN32!(!0)) {
        return LOWORD!(hr as u32) as UINT;
    } else {
        return if hr >= 0 { ERROR_SUCCESS } else { hr as UINT };
    }
}

pub unsafe extern "C" fn NotifyFileExtractCallback(ItemPath: *const CStringW, UncompressedSize: UINT64, FileAttributes: UINT, Callback: EXTRACTCALLBACK, Cookie: *mut core::ffi::c_void) -> BOOL {
    let eci = EXTRACTCALLBACKINFO {
        ItemPath: ItemPath as *const ATL_CStringW,
        UncompressedSize: UncompressedSize,
        FileAttributes: FileAttributes,
    };
    return if Callback.is_some() { (Callback.unwrap_unchecked())(&eci, Cookie) } else { (TRUE as i32) };
}

pub unsafe fn BuildPath__T0(Base: *const CStringW, mut Append: LPCWSTR) -> CStringW {
    let mut path = CStringW::from_ptr((*Base).GetString());
    let len: usize = path.GetLength() as usize;
    if len != 0 && path.GetAt((len - 1) as i32) != 0x005Cu16 && path.GetAt((len - 1) as i32) != 0x002Fu16 {
        path.AppendChar(0x005Cu16);
    }
    while *Append == 0x005Cu16 || *Append == 0x002Fu16 {
        Append = Append.add(1);
    }
    path.Append(Append);
    return path;
}

pub unsafe fn BuildPath__T1(Base: *const *const wchar_t, mut Append: LPCWSTR) -> CStringW {
    let mut path = CStringW::from_ptr(*Base);
    let len: usize = path.GetLength() as usize;
    if len != 0 && path.GetAt((len - 1) as i32) != 0x005Cu16 && path.GetAt((len - 1) as i32) != 0x002Fu16 {
        path.AppendChar(0x005Cu16);
    }
    while *Append == 0x005Cu16 || *Append == 0x002Fu16 {
        Append = Append.add(1);
    }
    path += Append;
    path
}

pub unsafe fn BuildPath__T2(Base: *const wchar_t, mut Append: LPCWSTR) -> CStringW {
    let mut path = CStringW::from_ptr(Base);
    let mut len: SIZE_T = path.GetLength() as SIZE_T;
    if len != 0 && path.GetAt((len - 1) as i32) != 0x005Cu16 && path.GetAt((len - 1) as i32) != 0x002Fu16 {
        path.AppendChar(0x005Cu16);
    }
    while *Append == 0x005Cu16 || *Append == 0x002Fu16 {
        Append = Append.add(1);
    }
    let mut result = path;
    result += Append;
    return result;
}

pub unsafe fn BuildPath__T3(Base: *const wchar_t, mut Append: LPCWSTR) -> CStringW {
    let mut path = CStringW::from_ptr(Base);
    let mut len: usize = path.GetLength() as usize;
    if len != 0
        && path.GetAt((len - 1) as i32) != 0x005Cu16
        && path.GetAt((len - 1) as i32) != 0x002Fu16
    {
        path.AppendChar(0x005Cu16);
    }
    while *Append == 0x005Cu16 || *Append == 0x002Fu16 {
        Append = Append.add(1);
    }
    path += Append;
    return path;
}

impl CScopedMutex {
    pub unsafe fn __ctor_0(&mut self, Name: LPCWSTR, Timeout: UINT, InitialOwner: BOOL) {
        let this_: *mut Self = self;
        {
            (*this_).m_hMutex = CreateMutexW(core::ptr::null_mut(), InitialOwner, Name);
            if !(*this_).m_hMutex.is_null() && InitialOwner == 0 {
                let wait: DWORD = WaitForSingleObject((*this_).m_hMutex, Timeout);
                if wait != WAIT_OBJECT_0 && wait != WAIT_ABANDONED {
                    CloseHandle((*this_).m_hMutex);
                    (*this_).m_hMutex = core::ptr::null_mut();
                }
            }
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_hMutex as usize != 0 {
            ReleaseMutex((*this_).m_hMutex);
            CloseHandle((*this_).m_hMutex);
        }
    }
    pub unsafe fn Acquired(&mut self) -> bool {
        let this_: *mut Self = self;
        return (*this_).m_hMutex != core::ptr::null_mut();
    }
}

impl Deleter_CAvailableApplicationInfo {
    pub unsafe fn __ctor_0(&mut self, ptr: *mut CAvailableApplicationInfo) {
        let this_: *mut Self = self;
        (*this_).p = ptr;
        {}
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        CAvailableApplicationInfo__delete((*this_).p);
    }
}

impl CLocalPtr_char {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CLocalPtr_wchar_t {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
