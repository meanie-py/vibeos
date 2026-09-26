use crate::prelude::*;
use crate::shell32::CExtractIcon::{
    CExtractIcon_CreateInstance, IDefaultExtractIconInit, IID_IDefaultExtractIconInit,
};
use core::ffi::c_void;

/// The `CComPtr` destructor: release on every exit path and NULL the slot.
pub struct CppComPtrGuard<T> { slot: *mut *mut T }
impl<T> CppComPtrGuard<T> {
    pub const fn new(slot: *mut *mut T) -> Self { Self { slot } }
}
impl<T> Drop for CppComPtrGuard<T> {
    fn drop(&mut self) {
        unsafe {
            let value = *self.slot;
            if !value.is_null() {
                let unknown = value as *mut IUnknown;
                ((*(*unknown).lpVtbl).Release)(unknown);
                *self.slot = core::ptr::null_mut();
            }
        }
    }
}

// The class half owns `IconLocation` / `CExtractIcon`; the generated preamble repeated both
// here, and a second copy is a distinct nominal type that drifts from the first in silence.
#[no_mangle]
pub unsafe extern "C" fn DuplicateString(Source: LPCWSTR, Destination: *mut LPWSTR) {
    let mut cb: usize;

    if !(*Destination).is_null() {
        CoTaskMemFree((*Destination).cast());
    }

    cb = (wcslen(Source) + 1) * core::mem::size_of::<u16>();
    *Destination = CoTaskMemAlloc(cb as SIZE_T) as LPWSTR;
    if (*Destination).is_null() {
        return;
    }
    // `CopyMemory(*Destination, Source, cb)` counts BYTES; `copy_nonoverlapping` counts
    // ELEMENTS, and these are `*mut u16`. Passing `cb` straight through copied twice the
    // allocation. Byte pointers keep the C's unit.
    core::ptr::copy_nonoverlapping(Source.cast::<u8>(), (*Destination).cast::<u8>(), cb);
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateDefaultExtractIcon(riid: REFIID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    // CExtractIcon.cpp:339 `ShellObjectCreator<CExtractIcon>(riid, ppv)` -- the internal,
    // non-aggregating factory the scaffold generated for this class.
    CExtractIcon_CreateInstance(core::ptr::null_mut(), riid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateFileExtractIconW(
    pszFile: LPCWSTR,
    dwFileAttributes: u32,
    riid: REFIID,
    ppv: *mut *mut core::ffi::c_void
) -> HRESULT {
    let mut shfi: SHFILEINFOW = core::mem::zeroed();
    let firet: DWORD_PTR = SHGetFileInfoW(
        pszFile,
        dwFileAttributes,
        &mut shfi,
        core::mem::size_of::<SHFILEINFOW>() as u32,
        SHGFI_USEFILEATTRIBUTES | SHGFI_ICONLOCATION
    );
    let mut hr: HRESULT = E_FAIL;
    if firet != 0 {
        let mut iconInit: *mut IDefaultExtractIconInit = core::ptr::null_mut();
        let _iconInit_cpp_com_guard = CppComPtrGuard::new(core::ptr::addr_of_mut!(iconInit));
        hr = SHCreateDefaultExtractIcon(
            core::ptr::addr_of!(IID_IDefaultExtractIconInit),
            (&mut iconInit as *mut *mut IDefaultExtractIconInit) as *mut *mut core::ffi::c_void
        );
        if ((hr) < 0) {
            if !iconInit.is_null() {
                ((*(*iconInit).lpVtbl).Release)(iconInit);
                iconInit = core::ptr::null_mut();
            }
            return hr;
        }

        hr = ((*(*iconInit).lpVtbl).SetNormalIcon)(
            iconInit,
            // SHFILEINFOW is `#[repr(packed)]` on i386, so `.as_ptr()` on a field builds a
            // reference to an unaligned place -- UB in Rust even if never dereferenced, and
            // an error only i386 reports. `addr_of!` takes the address without the reference.
            core::ptr::addr_of!(shfi.szDisplayName).cast::<WCHAR>(),
            shfi.iIcon
        );
        if ((hr) < 0) {
            if !iconInit.is_null() {
                ((*(*iconInit).lpVtbl).Release)(iconInit);
                iconInit = core::ptr::null_mut();
            }
            return hr;
        }

        let result: HRESULT = ((*(*iconInit).lpVtbl).QueryInterface)(iconInit, riid, ppv);
        if !iconInit.is_null() {
            ((*(*iconInit).lpVtbl).Release)(iconInit);
            iconInit = core::ptr::null_mut();
        }
        return result;
    }
    if ((hr) < 0) {
        return hr;
    }
    return hr;
}
