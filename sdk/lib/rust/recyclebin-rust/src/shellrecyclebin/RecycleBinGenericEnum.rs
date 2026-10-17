use crate::prelude::*;
use crate::shellrecyclebin::recyclebin::GetDefaultRecycleBin;
// IRecycleBinEnumList: 6 slot(s) from the bindgen island, which agrees with recyclebin.h.
// Not re-rendered here -- a second `IRecycleBinEnumList` would be a distinct type with the same name.

// IRecycleBinFile: 14 slot(s) from the bindgen island, which agrees with recyclebin.h.
// Not re-rendered here -- a second `IRecycleBinFile` would be a distinct type with the same name.

// Deterministic COM scaffold for RecycleBinGenericEnum; method bodies are generated separately.
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
pub struct RecycleBinGenericEnum {
    pub IRecycleBinEnumList_iface: IRecycleBinEnumList,
    pub m_ref: LONG,
    pub m_current: *mut IRecycleBinEnumList,
    pub m_dwLogicalDrives: DWORD,
    pub m_skip: SIZE_T,
}

unsafe extern "system" fn RecycleBinGenericEnum_IRecycleBinEnumList_QueryInterface(This: *mut IRecycleBinEnumList, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBinGenericEnum, IRecycleBinEnumList_iface)) as *mut RecycleBinGenericEnum;
    RecycleBinGenericEnum_QueryInterface_body(this, riid, ppvObject)
}
unsafe extern "system" fn RecycleBinGenericEnum_IRecycleBinEnumList_AddRef(This: *mut IRecycleBinEnumList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBinGenericEnum, IRecycleBinEnumList_iface)) as *mut RecycleBinGenericEnum;
    RecycleBinGenericEnum_AddRef_body(this)
}
unsafe extern "system" fn RecycleBinGenericEnum_IRecycleBinEnumList_Release(This: *mut IRecycleBinEnumList) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBinGenericEnum, IRecycleBinEnumList_iface)) as *mut RecycleBinGenericEnum;
    RecycleBinGenericEnum_Release_body(this)
}
unsafe extern "system" fn RecycleBinGenericEnum_IRecycleBinEnumList_Next(This: *mut IRecycleBinEnumList, celt: DWORD, rgelt: *mut *mut IRecycleBinFile, pceltFetched: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBinGenericEnum, IRecycleBinEnumList_iface)) as *mut RecycleBinGenericEnum;
    RecycleBinGenericEnum_Next_body(this, celt, rgelt, pceltFetched)
}
unsafe extern "system" fn RecycleBinGenericEnum_IRecycleBinEnumList_Skip(This: *mut IRecycleBinEnumList, celt: DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBinGenericEnum, IRecycleBinEnumList_iface)) as *mut RecycleBinGenericEnum;
    RecycleBinGenericEnum_Skip_body(this, celt)
}
unsafe extern "system" fn RecycleBinGenericEnum_IRecycleBinEnumList_Reset(This: *mut IRecycleBinEnumList) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(RecycleBinGenericEnum, IRecycleBinEnumList_iface)) as *mut RecycleBinGenericEnum;
    RecycleBinGenericEnum_Reset_body(this)
}

pub static RECYCLEBINGENERICENUM_IRECYCLEBINENUMLIST_VTBL: IRecycleBinEnumListVtbl = IRecycleBinEnumListVtbl {
    QueryInterface: Some(RecycleBinGenericEnum_IRecycleBinEnumList_QueryInterface),
    AddRef: Some(RecycleBinGenericEnum_IRecycleBinEnumList_AddRef),
    Release: Some(RecycleBinGenericEnum_IRecycleBinEnumList_Release),
    Next: Some(RecycleBinGenericEnum_IRecycleBinEnumList_Next),
    Skip: Some(RecycleBinGenericEnum_IRecycleBinEnumList_Skip),
    Reset: Some(RecycleBinGenericEnum_IRecycleBinEnumList_Reset),
};

/// `new RecycleBinGenericEnum` — operator new, the vptr store, then the constructor body.
pub unsafe fn RecycleBinGenericEnum_new() -> *mut RecycleBinGenericEnum {
    let this = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<RecycleBinGenericEnum>(),
    ) as *mut RecycleBinGenericEnum;
    if this.is_null() { return core::ptr::null_mut(); }
    (*this).IRecycleBinEnumList_iface.lpVtbl = core::ptr::addr_of!(RECYCLEBINGENERICENUM_IRECYCLEBINENUMLIST_VTBL) as *mut _;
    RecycleBinGenericEnum_ctor_body(this);
    this
}

/// `delete` — the destructor body, then the matching free. Pairs with
/// `RecycleBinGenericEnum_new`; freeing through any other allocator is a heap bug.
pub unsafe fn RecycleBinGenericEnum_delete(this: *mut RecycleBinGenericEnum) {
    if this.is_null() { return; }
    RecycleBinGenericEnum_dtor_body(this);
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

pub const DRIVE_FIXED: DWORD = 3;

// --local-raii CZZWStr: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CZZWStr;

// --local-raii CStringW: owned by crate::shellrecyclebin::recyclebin_support in this crate; imported, not copied (workers received this API as a sealed contract either way)
use crate::shellrecyclebin::recyclebin_support::CStringW;

use core::ffi::c_void;

pub unsafe fn RecycleBinGenericEnum_QueryInterface_body(this: *mut RecycleBinGenericEnum, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    if ppvObject.is_null() {
        return E_POINTER;
    }
    if cpp_com_iid_equal(riid, &IID_IUnknown) || cpp_com_iid_equal(riid, &IID_IRecycleBinEnumList) {
        *ppvObject = this as *mut core::ffi::c_void;
    } else {
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }
    RecycleBinGenericEnum_AddRef_body(this);
    S_OK
}

pub unsafe fn RecycleBinGenericEnum_AddRef_body(this: *mut RecycleBinGenericEnum) -> ULONG {
    let refCount = InterlockedIncrement(core::ptr::addr_of_mut!((*this).m_ref));
    refCount as u32
}

pub unsafe fn RecycleBinGenericEnum_dtor_body(this: *mut RecycleBinGenericEnum) -> () {
    if !((*this).m_current as *mut ()).is_null() {
        IRecycleBinEnumList_Release((*this).m_current);
    }
}

pub unsafe fn RecycleBinGenericEnum_Release_body(this: *mut RecycleBinGenericEnum) -> ULONG {
    let refCount = InterlockedDecrement(&mut (*this).m_ref);
    if refCount == 0 {
        RecycleBinGenericEnum_delete(this);
    }
    refCount as u32
}

pub unsafe fn RecycleBinGenericEnum_Next_body(this: *mut RecycleBinGenericEnum, celt: DWORD, rgelt: *mut *mut IRecycleBinFile, pceltFetched: *mut DWORD) -> HRESULT {
    if rgelt.is_null() {
        return E_POINTER;
    }
    if pceltFetched.is_null() && celt > 1 {
        return E_INVALIDARG;
    }

    let mut hr: HRESULT;
    let mut fetched: u32 = 0;

    loop {
        if (*this).m_current.is_null() && (*this).m_dwLogicalDrives != 0 {
            let mut i: u32 = 0;
            while i < (0x005Au32 - 0x0041u32 + 1) {
                if ((*this).m_dwLogicalDrives & (1 << i)) != 0 {
                    let mut szVolumeName: [u16; 4] = [0; 4];
                    szVolumeName[0] = (0x0041u16 + i as u16);
                    szVolumeName[1] = 0x003Au16;
                    szVolumeName[2] = 0x005Cu16;
                    szVolumeName[3] = UNICODE_NULL;

                    if GetDriveTypeW(core::ptr::addr_of!(szVolumeName[0])) != DRIVE_FIXED {
                        (*this).m_dwLogicalDrives &= !(1 << i);
                        i += 1;
                        continue;
                    }

                    let mut prb: *mut IRecycleBin = core::ptr::null_mut();
                    hr = GetDefaultRecycleBin(
                        core::ptr::addr_of!(szVolumeName[0]),
                        core::ptr::addr_of_mut!(prb),
                    );
                    if hr < 0 {
                        return hr;
                    }

                    hr = IRecycleBin_EnumObjects(prb, core::ptr::addr_of_mut!((*this).m_current));
                    IRecycleBin_Release(prb);

                    if hr < 0 {
                        return hr;
                    }

                    (*this).m_dwLogicalDrives &= !(1 << i);
                    break;
                }
                i += 1;
            }
        }

        if (*this).m_current.is_null() {
            if !pceltFetched.is_null() {
                (*pceltFetched) = fetched;
            }
            return S_FALSE;
        }

        while (*this).m_skip > 0 {
            let mut rbf: *mut IRecycleBinFile = core::ptr::null_mut();
            hr = IRecycleBinEnumList_Next(
                (*this).m_current,
                1,
                core::ptr::addr_of_mut!(rbf),
                core::ptr::null_mut(),
            );
            if hr == S_OK {
                IRecycleBinFile_Release(rbf);
            } else if hr == S_FALSE {
                break;
            } else if hr < 0 {
                return hr;
            }
        }

        if (*this).m_skip > 0 {
            continue;
        }

        let mut newFetched: u32 = 0;
        hr = IRecycleBinEnumList_Next(
            (*this).m_current,
            celt - fetched,
            rgelt.add(fetched as usize),
            core::ptr::addr_of_mut!(newFetched),
        );
        if hr >= 0 {
            fetched += newFetched;
        }

        if hr == S_FALSE || newFetched == 0 {
            IRecycleBinEnumList_Release((*this).m_current);
            (*this).m_current = core::ptr::null_mut();
        } else if hr < 0 {
            return hr;
        }

        if fetched == celt {
            if !pceltFetched.is_null() {
                (*pceltFetched) = fetched;
            }
            return S_OK;
        }
    }

    // recyclebin_generic_enumerator.cpp:176 `UNREACHABLE;` needs NO Rust statement. The C is
    // `while (TRUE)` (:93) with `return` on every path, and this is `loop {}` with no `break`,
    // so rustc already knows the fall-out is unreachable and types the function accordingly.
    // `unreachable_unchecked()` would be the literal lowering of `__builtin_unreachable()`, but
    // it is real UB for zero gain here, and `scan_assert_to_ub` rightly flagged it. Deleting the
    // statement is self-checking: if that loop could ever exit, this stops compiling.
}

pub unsafe fn RecycleBinGenericEnum_Skip_body(this: *mut RecycleBinGenericEnum, celt: DWORD) -> HRESULT {
    (*this).m_skip += celt as SIZE_T;
    S_OK
}

pub unsafe fn RecycleBinGenericEnum_Reset_body(this: *mut RecycleBinGenericEnum) -> HRESULT {
    let current = (*this).m_current;
    if !current.is_null() {
        // Release the current enumerator through its vtable
        IRecycleBinEnumList_Release(current);
        (*this).m_current = ::core::ptr::null_mut();
    }
    (*this).m_skip = 0;
    (*this).m_dwLogicalDrives = GetLogicalDrives();
    S_OK
}

pub unsafe fn RecycleBinGenericEnum_ctor_body(this: *mut RecycleBinGenericEnum) -> () {
    (*this).m_ref = 1;
    (*this).m_current = core::ptr::null_mut();
    (*this).m_dwLogicalDrives = 0;
    (*this).m_skip = 0;
}
