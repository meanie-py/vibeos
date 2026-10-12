use crate::prelude::*;
use core::ffi::c_void;











impl CEmptyVolumeCacheCallBack {
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        2
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        1
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        if IsEqualGUID!(&(*riid), &(IID_IUnknown)) || IsEqualGUID!(&(*riid), &(IID_IEmptyVolumeCacheCallBack)) {
            *ppvObject = this_ as *mut core::ffi::c_void;
            return S_OK;
        }
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }
    pub unsafe fn ScanProgress(&mut self, dwlSpaceUsed: DWORDLONG,
        dwFlags: DWORD,
        pcwszStatus: LPCWSTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // DPRINT("dwlSpaceUsed: %lld, dwFlags: %x\n", dwlSpaceUsed, dwFlags);
        S_OK
    }
    pub unsafe fn PurgeProgress(&mut self, dwlSpaceFreed: DWORDLONG,
        dwlSpaceToFree: DWORDLONG,
        dwFlags: DWORD,
        pcwszStatus: LPCWSTR,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // DPRINT("dwlSpaceFreed: %lld, dwlSpaceToFree: %lld, dwFlags: %x\n", dwlSpaceFreed, dwlSpaceToFree, dwFlags);
        S_OK
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
    }
}
