use crate::prelude::*;
// Deterministic from recyclebin.h
#[repr(C)]
pub struct IRecycleBin {
    pub lpVtbl: *const IRecycleBinVtbl,
}
#[repr(C)]
pub struct IRecycleBinVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IRecycleBin, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IRecycleBin) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IRecycleBin) -> ULONG,
    pub DeleteFile: unsafe extern "system" fn(This: *mut IRecycleBin, szFileName: LPCWSTR) -> HRESULT,
    pub EmptyRecycleBin: unsafe extern "system" fn(This: *mut IRecycleBin) -> HRESULT,
    pub EnumObjects: unsafe extern "system" fn(This: *mut IRecycleBin, ppEnumList: *mut *mut IRecycleBinEnumList) -> HRESULT,
    pub GetDirectory: unsafe extern "system" fn(This: *mut IRecycleBin, szPath: LPWSTR) -> HRESULT,
}
extern "C" {
    pub static IID_IRecycleBin: IID;
}

// Deterministic from recyclebin.h
#[repr(C)]
pub struct IRecycleBinEnumList {
    pub lpVtbl: *const IRecycleBinEnumListVtbl,
}
#[repr(C)]
pub struct IRecycleBinEnumListVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IRecycleBinEnumList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IRecycleBinEnumList) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IRecycleBinEnumList) -> ULONG,
    pub Next: unsafe extern "system" fn(This: *mut IRecycleBinEnumList, celt: DWORD, rgelt: *mut *mut IRecycleBinFile, pceltFetched: *mut DWORD) -> HRESULT,
    pub Skip: unsafe extern "system" fn(This: *mut IRecycleBinEnumList, celt: DWORD) -> HRESULT,
    pub Reset: unsafe extern "system" fn(This: *mut IRecycleBinEnumList) -> HRESULT,
}
extern "C" {
    pub static IID_IRecycleBinEnumList: IID;
}

// Deterministic from recyclebin.h
#[repr(C)]
pub struct IRecycleBinFile {
    pub lpVtbl: *const IRecycleBinFileVtbl,
}
#[repr(C)]
pub struct IRecycleBinFileVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IRecycleBinFile, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IRecycleBinFile) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IRecycleBinFile) -> ULONG,
    pub IsEqualIdentity: unsafe extern "system" fn(This: *mut IRecycleBinFile, pFI: *const RECYCLEBINFILEIDENTITY) -> HRESULT,
    pub GetInfo: unsafe extern "system" fn(This: *mut IRecycleBinFile, pInfo: PDELETED_FILE_INFO) -> HRESULT,
    pub GetLastModificationTime: unsafe extern "system" fn(This: *mut IRecycleBinFile, pLastModificationTime: *mut FILETIME) -> HRESULT,
    pub GetDeletionTime: unsafe extern "system" fn(This: *mut IRecycleBinFile, pDeletionTime: *mut FILETIME) -> HRESULT,
    pub GetFileSize: unsafe extern "system" fn(This: *mut IRecycleBinFile, pFileSize: *mut ULARGE_INTEGER) -> HRESULT,
    pub GetPhysicalFileSize: unsafe extern "system" fn(This: *mut IRecycleBinFile, pPhysicalFileSize: *mut ULARGE_INTEGER) -> HRESULT,
    pub GetAttributes: unsafe extern "system" fn(This: *mut IRecycleBinFile, pAttributes: *mut DWORD) -> HRESULT,
    pub GetFileName: unsafe extern "system" fn(This: *mut IRecycleBinFile, BufferSize: SIZE_T, Buffer: LPWSTR, RequiredSize: *mut SIZE_T) -> HRESULT,
    pub Delete: unsafe extern "system" fn(This: *mut IRecycleBinFile) -> HRESULT,
    pub Restore: unsafe extern "system" fn(This: *mut IRecycleBinFile) -> HRESULT,
    pub RemoveFromDatabase: unsafe extern "system" fn(This: *mut IRecycleBinFile) -> HRESULT,
}
extern "C" {
    pub static IID_IRecycleBinFile: IID;
}

// Deterministic COM scaffold for RecycleBinGeneric; method bodies are generated separately.
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
pub struct RecycleBinGeneric {
    pub IRecycleBin_iface: IRecycleBin,
    pub m_ref: LONG,
}

unsafe extern "system" fn RecycleBinGeneric_IRecycleBin_QueryInterface(This: *mut IRecycleBin, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBinGeneric, IRecycleBin_iface)) as *mut RecycleBinGeneric;
    RecycleBinGeneric_QueryInterface_body(this, riid, ppvObject)
}
unsafe extern "system" fn RecycleBinGeneric_IRecycleBin_AddRef(This: *mut IRecycleBin) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBinGeneric, IRecycleBin_iface)) as *mut RecycleBinGeneric;
    RecycleBinGeneric_AddRef_body(this)
}
unsafe extern "system" fn RecycleBinGeneric_IRecycleBin_Release(This: *mut IRecycleBin) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBinGeneric, IRecycleBin_iface)) as *mut RecycleBinGeneric;
    RecycleBinGeneric_Release_body(this)
}
unsafe extern "system" fn RecycleBinGeneric_IRecycleBin_DeleteFile(This: *mut IRecycleBin, szFileName: LPCWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBinGeneric, IRecycleBin_iface)) as *mut RecycleBinGeneric;
    RecycleBinGeneric_DeleteFile_body(this, szFileName)
}
unsafe extern "system" fn RecycleBinGeneric_IRecycleBin_EmptyRecycleBin(This: *mut IRecycleBin) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBinGeneric, IRecycleBin_iface)) as *mut RecycleBinGeneric;
    RecycleBinGeneric_EmptyRecycleBin_body(this)
}
unsafe extern "system" fn RecycleBinGeneric_IRecycleBin_EnumObjects(This: *mut IRecycleBin, ppEnumList: *mut *mut IRecycleBinEnumList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBinGeneric, IRecycleBin_iface)) as *mut RecycleBinGeneric;
    RecycleBinGeneric_EnumObjects_body(this, ppEnumList)
}
unsafe extern "system" fn RecycleBinGeneric_IRecycleBin_GetDirectory(This: *mut IRecycleBin, szPath: LPWSTR) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBinGeneric, IRecycleBin_iface)) as *mut RecycleBinGeneric;
    RecycleBinGeneric_GetDirectory_body(this, szPath)
}

pub static RECYCLEBINGENERIC_IRECYCLEBIN_VTBL: IRecycleBinVtbl = IRecycleBinVtbl {
    QueryInterface: RecycleBinGeneric_IRecycleBin_QueryInterface,
    AddRef: RecycleBinGeneric_IRecycleBin_AddRef,
    Release: RecycleBinGeneric_IRecycleBin_Release,
    DeleteFile: RecycleBinGeneric_IRecycleBin_DeleteFile,
    EmptyRecycleBin: RecycleBinGeneric_IRecycleBin_EmptyRecycleBin,
    EnumObjects: RecycleBinGeneric_IRecycleBin_EnumObjects,
    GetDirectory: RecycleBinGeneric_IRecycleBin_GetDirectory,
};

/// `new RecycleBinGeneric` — operator new, the vptr store, then the constructor body.
pub unsafe fn RecycleBinGeneric_new() -> *mut RecycleBinGeneric {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<RecycleBinGeneric>(),
    ) as *mut RecycleBinGeneric;
    if this.is_null() { return core::ptr::null_mut(); }
    (*this).IRecycleBin_iface.lpVtbl = core::ptr::addr_of!(RECYCLEBINGENERIC_IRECYCLEBIN_VTBL);
    RecycleBinGeneric_ctor_body(this);
    this
}

/// `delete` — the destructor body, then the matching free. Pairs with
/// `RecycleBinGeneric_new`; freeing through any other allocator is a heap bug.
pub unsafe fn RecycleBinGeneric_delete(this: *mut RecycleBinGeneric) {
    if this.is_null() { return; }
    RecycleBinGeneric_dtor_body(this);
    HeapFree(GetProcessHeap(), 0, this);
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null() && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Deterministic C++ support closure selected from source identifiers.

pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;

pub const DRIVE_FIXED: DWORD = 3;

pub const ERROR_NOT_ENOUGH_MEMORY: DWORD = 8;

#[repr(C)]
pub struct ULARGE_INTEGER {
    pub QuadPart: ULONGLONG,
}

extern "system" {
    fn GetLastError() -> DWORD;
    fn CoTaskMemAlloc(bytes: SIZE_T) -> LPVOID;
    fn CoTaskMemFree(memory: LPVOID);
}

extern "C" {
    fn _wcsicmp(left: LPCWSTR, right: LPCWSTR) -> INT;
}

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_88674D0269BC: [WCHAR; 5] = [0x0025, 0x0063, 0x003A, 0x005C, 0x0000];

// --local-raii CZZWStr: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CZZWStr;

// --local-raii CStringW: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CStringW;

use core::ffi::c_void;

pub unsafe fn RecycleBinGeneric_GetDirectory_body(this: *mut RecycleBinGeneric, szPath: LPWSTR) -> HRESULT { E_UNEXPECTED }

pub unsafe fn RecycleBinGeneric_QueryInterface_body(this: *mut RecycleBinGeneric, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    if ppvObject.is_null() {
        return E_POINTER;
    }

    if cpp_com_iid_equal(riid, &IID_IUnknown) || cpp_com_iid_equal(riid, &IID_IRecycleBin) {
        *ppvObject = this as *mut core::ffi::c_void;
        RecycleBinGeneric_AddRef_body(this);
        return S_OK;
    }

    *ppvObject = core::ptr::null_mut();
    E_NOINTERFACE
}

pub unsafe fn RecycleBinGeneric_AddRef_body(this: *mut RecycleBinGeneric) -> ULONG {
    let ref_count = ntdll_sys::nt::InterlockedIncrement(
        &raw mut (*this).m_ref,
    );
    ref_count as u32
}

pub unsafe fn RecycleBinGeneric_dtor_body(this: *mut RecycleBinGeneric) -> () {}

pub unsafe fn RecycleBinGeneric_Release_body(this: *mut RecycleBinGeneric) -> ULONG {
    let refCount = InterlockedDecrement(&mut (*this).m_ref);
    if refCount == 0 {
        let _ = SENDTO_ITEM_delete(this);
    }
    refCount
}

pub unsafe fn RecycleBinGeneric_DeleteFile_body(this: *mut RecycleBinGeneric, szFileName: LPCWSTR) -> HRESULT {
    // Get full file name
    let mut szFullName: LPWSTR = core::ptr::null_mut();
    let mut dwBufferLength: u32 = 0;
    loop {
        let len = GetFullPathNameW(szFileName, dwBufferLength, szFullName, core::ptr::null_mut());
        if len == 0 {
            if !szFullName.is_null() {
                CoTaskMemFree(szFullName);
            }
            return HRESULT_FROM_WIN32(GetLastError());
        } else if len < dwBufferLength {
            break;
        }
        if !szFullName.is_null() {
            CoTaskMemFree(szFullName);
        }
        dwBufferLength = len;
        szFullName = CoTaskMemAlloc((dwBufferLength as usize) * core::mem::size_of::<u16>()) as LPWSTR;
        if szFullName.is_null() {
            return HRESULT_FROM_WIN32(ERROR_NOT_ENOUGH_MEMORY as LONG);
        }
    }

    // Get associated volume path
    let mut szVolume: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    swprintf(
        szVolume.as_mut_ptr(),
        CPP_COM_WSTR_88674D0269BC.as_ptr(),
        *szFullName.offset(0 as isize),
    );

    // Skip namespace (if any): "\\.\" or "\\?\"
    if *szVolume.get_unchecked(0 as usize) == '\\' as u16
        && *szVolume.get_unchecked(1 as usize) == '\\' as u16
        && (*szVolume.get_unchecked(2 as usize) == '.' as u16
            || *szVolume.get_unchecked(2 as usize) == '?' as u16)
        && *szVolume.get_unchecked(3 as usize) == '\\' as u16
    {
        MoveMemory(
            szVolume.as_mut_ptr().cast(),
            (&szVolume[4 as usize] as *const u16).cast(),
            (core::mem::size_of_val(&szVolume) / core::mem::size_of::<u16>() - 4 as usize)
                * core::mem::size_of::<u16>(),
        );
    }

    let mut prb: *mut IRecycleBin = core::ptr::null_mut();
    let mut hr = GetDefaultRecycleBin(szVolume.as_ptr(), &mut prb as *mut *mut IRecycleBin);
    if hr < (0 as i32) {
        CoTaskMemFree(szFullName);
        return hr;
    }

    hr = ((*(*prb).lpVtbl).DeleteFile)(prb, szFullName);
    CoTaskMemFree(szFullName);
    ((*(*prb).lpVtbl).Release)(prb);
    hr
}

pub unsafe fn RecycleBinGeneric_EmptyRecycleBin_body(this: *mut RecycleBinGeneric) -> HRESULT {
    let dwLogicalDrives = GetLogicalDrives();
    if dwLogicalDrives == 0 {
        return HRESULT_FROM_WIN32(GetLastError());
    }

    let mut i = 0;
    while i < ('Z' as u32 - 'A' as u32 + 1) {
        if (dwLogicalDrives & (1 << i)) == 0 {
            i += 1;
            continue;
        }

        let mut szVolumeName: [u16; MAX_PATH] = [0u16; MAX_PATH];
        swprintf(
            szVolumeName.as_mut_ptr(),
            CPP_COM_WSTR_88674D0269BC.as_ptr(),
            0x0041u16 + i as u16,
        );
        if GetDriveTypeW(szVolumeName.as_ptr()) != DRIVE_FIXED {
            i += 1;
            continue;
        }

        let mut prb: *mut IRecycleBin = core::ptr::null_mut();
        let hr = GetDefaultRecycleBin(szVolumeName.as_ptr(), &mut prb as *mut *mut IRecycleBin);
        if !(hr >= 0) {
            return hr;
        }

        let _ = ((*(*prb).lpVtbl).EmptyRecycleBin)(prb);
        ((*(*prb).lpVtbl).Release)(prb);

        i += 1;
    }

    S_OK
}

pub unsafe fn RecycleBinGeneric_EnumObjects_body(this: *mut RecycleBinGeneric, ppEnumList: *mut *mut IRecycleBinEnumList) -> HRESULT {
    RecycleBinGenericEnum_Constructor(ppEnumList)
}

pub unsafe fn RecycleBinGeneric_ctor_body(this: *mut RecycleBinGeneric) -> () {
    (*this).m_ref = 1;
}
