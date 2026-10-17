use crate::prelude::*;
use crate::shellrecyclebin::recyclebin::EmptyRecycleBinW;
use crate::shellrecyclebin::recyclebin::GetDefaultRecycleBin;
use crate::shellrecyclebin::recyclebin_generic_enumerator_free::RecycleBinGenericEnum_Constructor;
// IRecycleBin: 7 slot(s) from the bindgen island, which agrees with recyclebin.h.
// Not re-rendered here -- a second `IRecycleBin` would be a distinct type with the same name.

// IRecycleBinEnumList: 6 slot(s) from the bindgen island, which agrees with recyclebin.h.
// Not re-rendered here -- a second `IRecycleBinEnumList` would be a distinct type with the same name.

// IRecycleBinFile: 14 slot(s) from the bindgen island, which agrees with recyclebin.h.
// Not re-rendered here -- a second `IRecycleBinFile` would be a distinct type with the same name.

// Deterministic COM scaffold for RecycleBinGeneric; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    #[link_name = "HeapAlloc"] fn cpp_com_HeapAlloc(heap: HANDLE, flags: DWORD, bytes: SIZE_T) -> *mut core::ffi::c_void;
    #[link_name = "HeapFree"] fn cpp_com_HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
trait CppComSize { fn cpp_com_size(self) -> usize; }
impl CppComSize for usize { fn cpp_com_size(self) -> usize { self } }
impl CppComSize for u32 { fn cpp_com_size(self) -> usize { self as usize } }
#[inline]
unsafe fn HeapAlloc<N: CppComSize>(heap: HANDLE, flags: DWORD, bytes: N) -> *mut core::ffi::c_void {
    cpp_com_HeapAlloc(heap, flags, bytes.cpp_com_size() as SIZE_T)
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
    QueryInterface: Some(RecycleBinGeneric_IRecycleBin_QueryInterface),
    AddRef: Some(RecycleBinGeneric_IRecycleBin_AddRef),
    Release: Some(RecycleBinGeneric_IRecycleBin_Release),
    DeleteFileW: Some(RecycleBinGeneric_IRecycleBin_DeleteFile),
    EmptyRecycleBinW: Some(RecycleBinGeneric_IRecycleBin_EmptyRecycleBin),
    EnumObjects: Some(RecycleBinGeneric_IRecycleBin_EnumObjects),
    GetDirectory: Some(RecycleBinGeneric_IRecycleBin_GetDirectory),
};

/// `new RecycleBinGeneric` — operator new, the vptr store, then the constructor body.
pub unsafe fn RecycleBinGeneric_new() -> *mut RecycleBinGeneric {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<RecycleBinGeneric>(),
    ) as *mut RecycleBinGeneric;
    if this.is_null() { return core::ptr::null_mut(); }
    (*this).IRecycleBin_iface.lpVtbl = core::ptr::addr_of!(RECYCLEBINGENERIC_IRECYCLEBIN_VTBL) as *mut _;
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
    let ref_count = InterlockedIncrement(core::ptr::addr_of_mut!((*this).m_ref));
    ref_count as u32
}

pub unsafe fn RecycleBinGeneric_dtor_body(this: *mut RecycleBinGeneric) -> () {}

pub unsafe fn RecycleBinGeneric_Release_body(this: *mut RecycleBinGeneric) -> ULONG {
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*this).m_ref)) as ULONG;
    if refCount == 0 {
        RecycleBinGeneric_delete(this);
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
                CoTaskMemFree(szFullName.cast());
            }
            return HRESULT_FROM_WIN32(GetLastError());
        } else if len < dwBufferLength {
            break;
        }
        if !szFullName.is_null() {
            CoTaskMemFree(szFullName.cast());
        }
        dwBufferLength = len;
        szFullName = CoTaskMemAlloc(((dwBufferLength as usize) * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
        if szFullName.is_null() {
            return HRESULT_FROM_WIN32(ERROR_NOT_ENOUGH_MEMORY);
        }
    }

    // Get associated volume path
    let mut szVolume: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    swprintf(
        szVolume.as_mut_ptr(),
        szVolume.len(),
        CPP_COM_WSTR_88674D0269BC.as_ptr(),
        // C default argument promotion: a WCHAR through `...` arrives as int, which is
        // what `%c` reads. Rust does not promote, so it is spelled.
        *szFullName.offset(0 as isize) as c_uint,
    );

    // Skip namespace (if any): "\\.\" or "\\?\"
    if *szVolume.get_unchecked(0 as usize) == '\\' as u16
        && *szVolume.get_unchecked(1 as usize) == '\\' as u16
        && (*szVolume.get_unchecked(2 as usize) == '.' as u16
            || *szVolume.get_unchecked(2 as usize) == '?' as u16)
        && *szVolume.get_unchecked(3 as usize) == '\\' as u16
    {
        MoveMemory!(
            szVolume.as_mut_ptr().cast(),
            (&szVolume[4 as usize] as *const u16).cast(),
            (core::mem::size_of_val(&szVolume) / core::mem::size_of::<u16>() - 4 as usize)
                * core::mem::size_of::<u16>(),
        );
    }

    let mut prb: *mut IRecycleBin = core::ptr::null_mut();
    let mut hr = GetDefaultRecycleBin(szVolume.as_ptr(), &mut prb as *mut *mut IRecycleBin);
    if hr < (0 as i32) {
        CoTaskMemFree(szFullName.cast());
        return hr;
    }

    // The island renders every vtable slot as `Option<fn>` and names this one `DeleteFileW`
    // (recyclebin.h:99 `#define DeleteFileToRecycleBin ...W`, the TCHAR family). The crate's
    // `commacros` helpers already carry both facts, including the `unwrap_unchecked`.
    hr = IRecycleBin_DeleteFile(prb, szFullName);
    CoTaskMemFree(szFullName.cast());
    IRecycleBin_Release(prb);
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

        let mut szVolumeName: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
        swprintf(
            szVolumeName.as_mut_ptr(),
            szVolumeName.len(),
            CPP_COM_WSTR_88674D0269BC.as_ptr(),
            // recyclebin_generic.cpp:151 `swprintf(szVolumeName, L"%c:\\", L'A' + i)` -- the
            // sum is an int in C (both operands promote), and `%c` reads an int.
            (0x0041u16 + i as u16) as c_uint,
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

        let _ = IRecycleBin_EmptyRecycleBin(prb);
        IRecycleBin_Release(prb);

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
