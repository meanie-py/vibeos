use crate::prelude::*;
// Deterministic COM scaffold for CDirectoryItem; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: usize) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size())
}
#[inline]
unsafe fn HeapFree<T>(heap: HANDLE, flags: DWORD, memory: *mut T) -> BOOL {
    cpp_com_HeapFree(heap, flags, memory.cast())
}
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct CDirectoryItem {
    pub m_pszPath: LPWSTR,
}

/// `new CDirectoryItem` — operator new, the vptr store, then the constructor body.
pub unsafe fn CDirectoryItem_new() -> *mut CDirectoryItem {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CDirectoryItem>(),
    ) as *mut CDirectoryItem;
    if this.is_null() { return core::ptr::null_mut(); }
    CDirectoryItem_ctor_body(this);
    this
}

/// `delete` — the destructor body, then the matching free. Pairs with
/// `CDirectoryItem_new`; freeing through any other allocator is a heap bug.
pub unsafe fn CDirectoryItem_delete(this: *mut CDirectoryItem) {
    if this.is_null() { return; }
    CDirectoryItem_dtor_body(this);
    HeapFree(GetProcessHeap(), 0, this);
}

// Deterministic C++ support closure selected from source identifiers.

pub const FILE_ATTRIBUTE_DIRECTORY: DWORD = 0x00000010;

pub use crate::island::WIN32_FIND_DATAW;

extern "system" {
    fn lstrlenW(text: LPCWSTR) -> INT;
}

extern "C" {
    fn _wcsdup(text: LPCWSTR) -> LPWSTR;
}

extern "C" {
    #[link_name = "free"] fn cpp_com_free(memory: *mut core::ffi::c_void);
}
#[inline]
unsafe fn free<T>(memory: *mut T) { cpp_com_free(memory.cast()); }

use core::ffi::c_void;

pub unsafe fn CDirectoryItem_ctor_body(this: *mut CDirectoryItem) -> () {}

pub unsafe fn CDirectoryItem_ctor_2_body(this: *mut CDirectoryItem, pszPath: LPCWSTR) -> () {
    (*this).m_pszPath = _wcsdup(pszPath);
}

pub unsafe fn CDirectoryItem_ctor_3_body(this: *mut CDirectoryItem, item: &CDirectoryItem) -> () {
    (*this).m_pszPath = _wcsdup((*item).m_pszPath as *const _);
}

pub unsafe fn CDirectoryItem_dtor_body(this: *mut CDirectoryItem) -> () {
    free((*this).m_pszPath);
}

pub unsafe fn CDirectoryItem_IsEmpty_body(this: *mut CDirectoryItem) -> BOOL {
    if (*this).m_pszPath.is_null() { 1 } else { 0 }
}

pub unsafe fn CDirectoryItem_GetPath_body(this: *mut CDirectoryItem) -> LPCWSTR {
    return (*this).m_pszPath;
}

pub unsafe fn CDirectoryItem_SetPath_body(this: *mut CDirectoryItem, pszPath: LPCWSTR) -> () {
    free((*this).m_pszPath as *mut core::ffi::c_void);
    (*this).m_pszPath = _wcsdup(pszPath);
}

pub unsafe fn CDirectoryItem_EqualPath_body(this: *mut CDirectoryItem, pszPath: LPCWSTR) -> BOOL {
    if (*this).m_pszPath != core::ptr::null_mut()
        && lstrcmpiW((*this).m_pszPath, pszPath) == 0
    {
        1
    } else {
        0
    }
}
