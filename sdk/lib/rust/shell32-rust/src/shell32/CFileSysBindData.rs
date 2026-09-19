// rustconv-assembly: class:CFileSysBindData
use crate::prelude::*;
// Deterministic from shobjidl.h
#[repr(C)]
pub struct IFileSystemBindData {
    pub lpVtbl: *const IFileSystemBindDataVtbl,
}
#[repr(C)]
pub struct IFileSystemBindDataVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IFileSystemBindData, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IFileSystemBindData) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IFileSystemBindData) -> ULONG,
    pub SetFindData: unsafe extern "system" fn(This: *mut IFileSystemBindData, pfd: *const WIN32_FIND_DATAW) -> HRESULT,
    pub GetFindData: unsafe extern "system" fn(This: *mut IFileSystemBindData, pfd: *mut WIN32_FIND_DATAW) -> HRESULT,
}
extern "C" {
    pub static IID_IFileSystemBindData: IID;
}

// Deterministic COM scaffold for CFileSysBindData; method bodies are generated separately.
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
pub struct CFileSysBindData {
    pub IFileSystemBindData_iface: IFileSystemBindData,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub findFile: WIN32_FIND_DATAW,
}

unsafe extern "system" fn CFileSysBindData_IFileSystemBindData_QueryInterface(This: *mut IFileSystemBindData, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileSysBindData, IFileSystemBindData_iface)) as *mut CFileSysBindData;
    CFileSysBindData_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CFileSysBindData_IFileSystemBindData_AddRef(This: *mut IFileSystemBindData) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileSysBindData, IFileSystemBindData_iface)) as *mut CFileSysBindData;
    CFileSysBindData_AddRef(this)
}
unsafe extern "system" fn CFileSysBindData_IFileSystemBindData_Release(This: *mut IFileSystemBindData) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileSysBindData, IFileSystemBindData_iface)) as *mut CFileSysBindData;
    CFileSysBindData_Release(this)
}
unsafe extern "system" fn CFileSysBindData_IFileSystemBindData_SetFindData(This: *mut IFileSystemBindData, pfd: *const WIN32_FIND_DATAW) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileSysBindData, IFileSystemBindData_iface)) as *mut CFileSysBindData;
    CFileSysBindData_SetFindData_body(this, pfd)
}
unsafe extern "system" fn CFileSysBindData_IFileSystemBindData_GetFindData(This: *mut IFileSystemBindData, pfd: *mut WIN32_FIND_DATAW) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CFileSysBindData, IFileSystemBindData_iface)) as *mut CFileSysBindData;
    CFileSysBindData_GetFindData_body(this, pfd)
}

pub static CFILESYSBINDDATA_IFILESYSTEMBINDDATA_VTBL: IFileSystemBindDataVtbl = IFileSystemBindDataVtbl {
    QueryInterface: CFileSysBindData_IFileSystemBindData_QueryInterface,
    AddRef: CFileSysBindData_IFileSystemBindData_AddRef,
    Release: CFileSysBindData_IFileSystemBindData_Release,
    SetFindData: CFileSysBindData_IFileSystemBindData_SetFindData,
    GetFindData: CFileSysBindData_IFileSystemBindData_GetFindData,
};

unsafe fn CFileSysBindData_QueryInterface(
    this: *mut CFileSysBindData, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IFileSystemBindData)) {
        *ppv = core::ptr::addr_of_mut!((*this).IFileSystemBindData_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CFileSysBindData_AddRef(this);
    S_OK
}

unsafe fn CFileSysBindData_AddRef(this: *mut CFileSysBindData) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CFileSysBindData_Release(this: *mut CFileSysBindData) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CFileSysBindData_dtor_body(this);
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null() && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Internal activation: this class has no CLSID; the DLL constructs it directly through
// shellutils.h ShellObjectCreator<T> or ATL's own _CreatorClass::CreateInstance, both of
// which take (outer, riid, ppv) — the signature of this factory.
pub unsafe fn CFileSysBindData_CreateInstance(
    outer: *mut IUnknown,
    riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<CFileSysBindData>()) as *mut CFileSysBindData;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CFileSysBindData {
        IFileSystemBindData_iface: IFileSystemBindData { lpVtbl: core::ptr::addr_of!(CFILESYSBINDDATA_IFILESYSTEMBINDDATA_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        findFile: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CFileSysBindData_ctor_body(this);
    let hr = CFileSysBindData_QueryInterface(this, riid, ppv);
    CFileSysBindData_Release(this);
    hr
}

// Deterministic C++ support closure selected from source identifiers.

pub const STGM_CREATE: DWORD = 0x00001000;

#[repr(C)]
pub struct BIND_OPTS {
    pub cbStruct: DWORD,
    pub grfFlags: DWORD,
    pub grfMode: DWORD,
    pub dwTickCountDeadline: DWORD,
}

#[repr(C)]
pub struct WIN32_FIND_DATAW {
    pub dwFileAttributes: DWORD,
    pub ftCreationTime: FILETIME,
    pub ftLastAccessTime: FILETIME,
    pub ftLastWriteTime: FILETIME,
    pub nFileSizeHigh: DWORD,
    pub nFileSizeLow: DWORD,
    pub dwReserved0: DWORD,
    pub dwReserved1: DWORD,
    pub cFileName: [WCHAR; 260],
    pub cAlternateFileName: [WCHAR; 14],
}

extern "C" {
    #[link_name = "memcpy"] fn cpp_com_memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
}

#[inline]
unsafe fn memcpy<D, S, N: CppComSize>(dst: *mut D, src: *const S, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_memcpy(dst.cast(), src.cast(), bytes.cpp_com_size())
}

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

use core::ffi::c_void;

pub unsafe fn CFileSysBindData_ctor_body(this: *mut CFileSysBindData) -> () {
    memset(
        core::ptr::addr_of_mut!((*this).findFile) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<WIN32_FIND_DATAW>(),
    );
}

pub unsafe fn CFileSysBindData_dtor_body(this: *mut CFileSysBindData) -> () {}

pub unsafe fn CFileSysBindData_GetFindData_body(this: *mut CFileSysBindData, pfd: *mut WIN32_FIND_DATAW) -> HRESULT {
    if pfd.is_null() {
        return E_INVALIDARG;
    }

    memcpy(
        pfd,
        core::ptr::addr_of!((*this).findFile),
        core::mem::size_of::<WIN32_FIND_DATAW>(),
    );

    return S_OK;
}

pub unsafe fn CFileSysBindData_SetFindData_body(this: *mut CFileSysBindData, pfd: *const WIN32_FIND_DATAW) -> HRESULT {
    if !pfd.is_null() {
        unsafe {
            memcpy(
                &mut (*this).findFile as *mut _,
                pfd as *const _,
                core::mem::size_of::<WIN32_FIND_DATAW>()
            );
        }
    } else {
        unsafe {
            memset(
                core::ptr::addr_of_mut!((*this).findFile).cast(),
                0,
                core::mem::size_of::<WIN32_FIND_DATAW>()
            );
        }
    }
    return S_OK;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * File System Bind Data object to use as parameter for the bind context to
//  * IShellFolder_ParseDisplayName
//  *
//  * Copyright 2003 Rolf Kalbermatter
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  *
//  */
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(pidl);
//
// /***********************************************************************
//  * IFileSystemBindData implementation
//  */
// class CFileSysBindData :
//     public CComObjectRootEx<CComMultiThreadModelNoCS>,
//     public IFileSystemBindData
// {
// private:
//     WIN32_FIND_DATAW findFile;
// public:
//     CFileSysBindData();
//     ~CFileSysBindData();
//
//     // *** IFileSystemBindData methods ***
//     STDMETHOD(SetFindData)(const WIN32_FIND_DATAW *pfd) override;
//     STDMETHOD(GetFindData)(WIN32_FIND_DATAW *pfd) override;
//
// DECLARE_NOT_AGGREGATABLE(CFileSysBindData)
// DECLARE_PROTECT_FINAL_CONSTRUCT()
//
// BEGIN_COM_MAP(CFileSysBindData)
//     COM_INTERFACE_ENTRY_IID(IID_IFileSystemBindData, IFileSystemBindData)
// END_COM_MAP()
// };
//
// HRESULT WINAPI CFileSysBindData::GetFindData(WIN32_FIND_DATAW *pfd)
// {
//     TRACE("(%p), %p\n", this, pfd);
//
//     if (!pfd)
//         return E_INVALIDARG;
//
//     memcpy(pfd, &findFile, sizeof(WIN32_FIND_DATAW));
//     return S_OK;
// }
// HRESULT WINAPI CFileSysBindData::SetFindData(const WIN32_FIND_DATAW *pfd)
// {
//     TRACE("(%p), %p\n", this, pfd);
//
//     if (pfd)
//         memcpy(&findFile, pfd, sizeof(WIN32_FIND_DATAW));
//     else
//         memset(&findFile, 0, sizeof(WIN32_FIND_DATAW));
//     return S_OK;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
