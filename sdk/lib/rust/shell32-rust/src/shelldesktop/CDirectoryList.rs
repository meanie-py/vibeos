use crate::prelude::*;
use crate::shelldesktop::CDirectoryItem::CDirectoryItem_EqualPath_body;
use crate::shelldesktop::CDirectoryItem::CDirectoryItem_IsEmpty_body;
use crate::shelldesktop::CDirectoryItem::CDirectoryItem_SetPath_body;
#[repr(C)]
pub struct CDirectoryItem {
    pub m_pszPath: LPWSTR,
}

// Deterministic COM scaffold for CDirectoryList; method bodies are generated separately.
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
pub struct CDirectoryList {
    pub m_fRecursive: BOOL,
    pub m_items: CSimpleArray<CDirectoryItem>,
}

/// `new CDirectoryList` — operator new, the vptr store, then the constructor body.
pub unsafe fn CDirectoryList_new() -> *mut CDirectoryList {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<CDirectoryList>(),
    ) as *mut CDirectoryList;
    if this.is_null() { return core::ptr::null_mut(); }
    CDirectoryList_ctor_body(this);
    this
}

/// `delete` — the destructor body, then the matching free. Pairs with
/// `CDirectoryList_new`; freeing through any other allocator is a heap bug.
pub unsafe fn CDirectoryList_delete(this: *mut CDirectoryList) {
    if this.is_null() { return; }
    CDirectoryList_dtor_body(this);
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

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_E8A4B2EE7EDE: [WCHAR; 2] = [0x002A, 0x0000];

use core::ffi::c_void;

pub unsafe fn CDirectoryList_dtor_body(this: *mut CDirectoryList) -> () {}

pub unsafe fn CDirectoryList_ContainsPath_body(this: *mut CDirectoryList, pszPath: LPCWSTR) -> BOOL {
    assert!(PathIsRelativeW(pszPath) == 0);

    for i in 0..(*this).m_items.GetSize() {
        let pItem = (*this).m_items.GetAt(i);

        if CDirectoryItem_IsEmpty_body(pItem.cast()) != 0 {
            continue;
        }

        if CDirectoryItem_EqualPath_body(pItem.cast(), pszPath) != 0 {
            return TRUE;
        }
    }

    return FALSE;
}

pub unsafe fn CDirectoryList_AddPath_body(this: *mut CDirectoryList, pszPath: LPCWSTR) -> BOOL {
    assert!(PathIsRelativeW(pszPath) == 0);
    if CDirectoryList_ContainsPath_body(this, pszPath) != 0 {
        return FALSE;
    }
    let mut i: INT = 0;
    while i < (*this).m_items.GetSize() {
        let item = (*this).m_items.GetAt(i);
        if CDirectoryItem_IsEmpty_body(item.cast()) != 0 {
            CDirectoryItem_SetPath_body(item.cast(), pszPath);
            return TRUE;
        }
        i += 1;
    }
    let new_item = CDirectoryItem { m_pszPath: _wcsdup(pszPath) };
    return (*this).m_items.Add(new_item);
}

pub unsafe fn CDirectoryList_RenamePath_body(this: *mut CDirectoryList, pszPath1: LPCWSTR, pszPath2: LPCWSTR) -> BOOL {
    assert!(PathIsRelativeW(pszPath1) == 0);
    assert!(PathIsRelativeW(pszPath2) == 0);

    let mut i: i32 = 0;
    while i < (*this).m_items.GetSize() {
        if CDirectoryItem_EqualPath_body((*this).m_items.GetAt(i).cast(), pszPath1) != 0 {
            CDirectoryItem_SetPath_body((*this).m_items.GetAt(i).cast(), pszPath2);
            return TRUE;
        }
        i += 1;
    }
    return FALSE;
}

pub unsafe fn CDirectoryList_DeletePath_body(this: *mut CDirectoryList, pszPath: LPCWSTR) -> BOOL {
    assert!(PathIsRelativeW(pszPath) == 0);
    let mut i: i32 = 0;
    while i < (*this).m_items.GetSize() {
        let pItem = (*this).m_items.GetAt(i);
        if CDirectoryItem_EqualPath_body(pItem.cast(), pszPath) != 0 {
            CDirectoryItem_SetPath_body(pItem.cast(), core::ptr::null::<u16>());
            return TRUE;
        }
        i += 1;
    }
    return FALSE;
}

pub unsafe fn CDirectoryList_ctor_body(this: *mut CDirectoryList) -> () {}

pub unsafe fn CDirectoryList_ctor_2_body(this: *mut CDirectoryList, pszDirectoryPath: LPCWSTR, fRecursive: BOOL) -> () {
    (*this).m_fRecursive = fRecursive;
    CDirectoryList_AddPathsFromDirectory_body(this, pszDirectoryPath);
}

pub unsafe fn CDirectoryList_AddPathsFromDirectory_body(this: *mut CDirectoryList, pszDirectoryPath: LPCWSTR) -> BOOL {
    let mut szPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    lstrcpynW(szPath.as_mut_ptr(), pszDirectoryPath, szPath.len() as INT);
    assert!(PathIsRelativeW(szPath.as_ptr()) == 0);

    if PathIsDirectoryW(szPath.as_ptr()) == 0 {
        return FALSE;
    }

    if CDirectoryList_AddPath_body(this, szPath.as_ptr()) == 0 {
        return FALSE;
    }

    PathAppendW(szPath.as_mut_ptr(), CPP_COM_WSTR_E8A4B2EE7EDE.as_ptr());
    let mut find: WIN32_FIND_DATAW = core::mem::zeroed();
    let hFind: HANDLE = FindFirstFileW(szPath.as_ptr(), core::ptr::addr_of_mut!(find).cast());
    if hFind == INVALID_HANDLE_VALUE {
        return FALSE;
    }

    let mut first = true;
    while first || FindNextFileW(hFind, core::ptr::addr_of_mut!(find).cast()) != 0 {
        first = false;

        let pch: *const u16 = find.cFileName.as_ptr();
        if *pch.offset(0 as isize) == 0x002Eu16 &&
           (*pch.offset(1 as isize) == 0 ||
            (*pch.offset(1 as isize) == 0x002Eu16 &&
             *pch.offset(2 as isize) == 0))
        {
            continue;
        }

        PathRemoveFileSpecW(szPath.as_mut_ptr());
        if (lstrlenW(szPath.as_ptr()) + lstrlenW(find.cFileName.as_ptr()) + 1) > MAX_PATH as INT {
            continue;
        }
        PathAppendW(szPath.as_mut_ptr(), find.cFileName.as_ptr());

        if (find.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
            if (*this).m_fRecursive != 0 {
                CDirectoryList_AddPathsFromDirectory_body(this, szPath.as_ptr());
            } else {
                CDirectoryList_AddPath_body(this, szPath.as_ptr());
            }
        }
    }

    FindClose(hFind);

    return TRUE;
}

pub unsafe fn CDirectoryList_RemoveAll_body(this: *mut CDirectoryList) -> () {
    (*this).m_items.RemoveAll();
}
