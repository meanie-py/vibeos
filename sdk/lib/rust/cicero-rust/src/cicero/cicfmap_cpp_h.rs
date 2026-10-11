use crate::prelude::*;
use core::ffi::c_void;











impl CicFileMappingStatic {
    pub unsafe fn _Map(&mut self) -> LPVOID {
        let this_: *mut Self = self;
        (*this_).m_pView = MapViewOfFile((*this_).m_hMapping, FILE_MAP_WRITE, 0, 0, 0);
        if (*this_).m_pView.is_null() {
            (*this_).Close();
            return core::ptr::null_mut();
        }
        return (*this_).m_pView;
    }
    pub unsafe fn Init(&mut self, pszName: LPCTSTR, pMutex: *mut CicMutex) {
        let this_: *mut Self = self;
        if !pMutex.is_null() {
            (*this_).m_pMutex = pMutex;
        }
    
        (*this_).m_bCreated = (FALSE) as i32;
        (*this_).m_pszName = pszName;
        (*this_).m_bHasMutex = (pMutex != core::ptr::null_mut()) as BOOL;
    }
    pub unsafe fn Create(&mut self, pSA: LPSECURITY_ATTRIBUTES, dwMaximumSizeLow: DWORD, pbAlreadyExists: LPBOOL) -> LPVOID {
        let this_: *mut Self = self;
        if (*this_).m_pszName.is_null() {
            return core::ptr::null_mut();
        }
    
        (*this_).m_hMapping = CreateFileMapping(INVALID_HANDLE_VALUE,
                                         pSA,
                                         PAGE_READWRITE,
                                         0,
                                         dwMaximumSizeLow,
                                         (*this_).m_pszName);
        if !pbAlreadyExists.is_null() {
            *pbAlreadyExists = ((GetLastError() == ERROR_ALREADY_EXISTS) as i32);
        }
        if (*this_).m_hMapping.is_null() {
            return core::ptr::null_mut();
        }
    
        (*this_).m_bCreated = (TRUE) as i32;
        return (*this_)._Map();
    }
    pub unsafe fn Open(&mut self) -> LPVOID {
        let this_: *mut Self = self;
        if (*this_).m_pszName.is_null() {
            return core::ptr::null_mut();
        }
        (*this_).m_hMapping = OpenFileMapping(FILE_MAP_ALL_ACCESS, (FALSE) as i32, (*this_).m_pszName);
        if (*this_).m_hMapping as usize == 0 {
            return core::ptr::null_mut();
        }
    
        return (*this_)._Map();
    }
    pub unsafe fn Close(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pView.is_null() {
            UnmapViewOfFile((*this_).m_pView);
            (*this_).m_pView = core::ptr::null_mut();
        }
    
        if !(*this_).m_hMapping.is_null() {
            CloseHandle((*this_).m_hMapping);
            (*this_).m_hMapping = core::ptr::null_mut();
        }
    
        (*this_).m_bCreated = (FALSE) as i32;
    }
    pub unsafe fn Enter(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_bHasMutex == 0 {
            return (TRUE) as BOOL;
        }
        return (*(*this_).m_pMutex).Enter();
    }
    pub unsafe fn Leave(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_bHasMutex == 0 {
            return;
        }
        (*(*this_).m_pMutex).Leave();
    }
    pub unsafe fn Flush(&mut self, dwNumberOfBytesToFlush: SIZE_T) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_pView.is_null() {
            return (FALSE) as BOOL;
        }
        return FlushViewOfFile((*this_).m_pView, dwNumberOfBytesToFlush);
    }
    pub unsafe fn Finalize(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_bHasMutex == 0 {
            return;
        }
    
        (*this_).Close();
        (*this_).Leave();
    }
}

impl CicFileMapping {
    pub unsafe fn __ctor_0(&mut self, pszName: LPCTSTR, pMutex: *mut CicMutex) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
            (*this_).__base0.m_pszName = core::ptr::null_mut();
            (*this_).__base0.m_pView = core::ptr::null_mut();
            (*this_).__base0.m_hMapping = core::ptr::null_mut();
            (*this_).__base0.m_bCreated = (FALSE) as i32;
            (*this_).__base0.m_bHasMutex = (FALSE) as i32;
            (*this_).__base0.m_pMutex = core::ptr::null_mut();
            (*this_).__base0.Init(pszName, pMutex);
        }
    }
}
