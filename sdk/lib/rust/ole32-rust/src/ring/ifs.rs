use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

/// NT `LIST_ENTRY` — only needed for the statically-initialized
/// `CRITICAL_SECTION_DEBUG` below (ifs.c:65). Not in the shared surface
/// (`crate::win32` treats CRITICAL_SECTION.DebugInfo as opaque `*mut c_void`).
#[repr(C)]
pub struct LIST_ENTRY {
    pub Flink: *mut LIST_ENTRY,
    pub Blink: *mut LIST_ENTRY,
}

/// NDK `RTL_CRITICAL_SECTION_DEBUG` (sdk/include/ndk/rtltypes.h:1424). The trailing
/// union is `{ ULONG_PTR WineDebugString; ULONG_PTR Spare[1]; struct { ULONG Flags;
/// USHORT CreatorBackTraceIndexHigh; USHORT SpareWORD; } }` — the C initializer in
/// ifs.c uses the `WineDebugString` arm (`(DWORD_PTR)(__FILE__ ": IMalloc32_SpyCS")`),
/// so we model that arm directly (ULONG_PTR-sized, so the layout is unchanged).
#[repr(C)]
pub struct CRITICAL_SECTION_DEBUG {
    pub Type: u16,
    pub CreatorBackTraceIndex: u16,
    pub CriticalSection: *mut CRITICAL_SECTION,
    pub ProcessLocksList: LIST_ENTRY,
    pub EntryCount: u32,
    pub ContentionCount: u32,
    pub WineDebugString: *const c_char,
}

#[repr(C)]
pub struct _Malloc32 {
    pub IMalloc_iface: IMalloc,
    pub dummy: DWORD,
    pub pSpy: *mut IMallocSpy,
    pub SpyedAllocationsLeft: DWORD,
    pub SpyReleasePending: BOOL,
    pub SpyedBlocks: *mut LPVOID,
    pub SpyedBlockTableLength: DWORD,
}

pub type P_Malloc32 = *mut _Malloc32;

/// ifs.c:61 — the static object instance.
pub static mut Malloc32: _Malloc32 = _Malloc32 {
    IMalloc_iface: IMalloc {
        lpVtbl: core::ptr::addr_of!(VT_IMalloc32) as *mut IMallocVtbl,
    },
    dummy: 0,
    pSpy: core::ptr::null_mut(),
    SpyedAllocationsLeft: 0,
    SpyReleasePending: 0,
    SpyedBlocks: core::ptr::null_mut(),
    SpyedBlockTableLength: 0,
};

static IMALLOC32_SPYCS_NAME: &[u8; 24] = b"ifs.c: IMalloc32_SpyCS\0\0";

/// ifs.c:65
pub static mut critsect_debug: CRITICAL_SECTION_DEBUG = CRITICAL_SECTION_DEBUG {
    Type: 0,
    CreatorBackTraceIndex: 0,
    CriticalSection: core::ptr::addr_of_mut!(IMalloc32_SpyCS),
    ProcessLocksList: LIST_ENTRY {
        Flink: unsafe { core::ptr::addr_of_mut!(critsect_debug.ProcessLocksList) as *mut LIST_ENTRY },
        Blink: unsafe { core::ptr::addr_of_mut!(critsect_debug.ProcessLocksList) as *mut LIST_ENTRY },
    },
    EntryCount: 0,
    ContentionCount: 0,
    WineDebugString: IMALLOC32_SPYCS_NAME.as_ptr() as *const c_char,
};

/// ifs.c:71 — with a spy active all calls from pre to post methods are threadsafe.
pub static mut IMalloc32_SpyCS: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: core::ptr::addr_of_mut!(critsect_debug) as *mut c_void,
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};

/// ifs.c:117 `mallocspy_is_allocation_spyed` — hand-translated: rustconv could not
/// extract this body (its `static void**` signature defeated the extractor), so it
/// emitted an extern decl for a symbol that does not exist. Faithful to the C:
/// linear-scan the spy table for `mem`, returning the SLOT (so callers can clear it)
/// or NULL. Note the C walks off the end deliberately and bounds-checks after the
/// increment — the table is not NULL-terminated.
unsafe fn mallocspy_is_allocation_spyed(mem: LPCVOID) -> *mut LPVOID {
    let mut current: *mut LPVOID = Malloc32.SpyedBlocks;

    while *current != mem as LPVOID {
        current = current.add(1);
        if current >= Malloc32.SpyedBlocks.add(Malloc32.SpyedBlockTableLength as usize) {
            return core::ptr::null_mut();
        }
    }

    current
}

/// ifs.c:74 — resize the old table.
unsafe fn SetSpyedBlockTableLength(NewLength: DWORD) -> BOOL {
    let NewSpyedBlocks: *mut LPVOID;

    if Malloc32.SpyedBlocks.is_null() {
        NewSpyedBlocks = LocalAlloc(
            LMEM_ZEROINIT,
            (NewLength as SIZE_T) * (core::mem::size_of::<LPVOID>() as SIZE_T),
        ) as *mut LPVOID;
    } else {
        NewSpyedBlocks = LocalReAlloc(
            Malloc32.SpyedBlocks as HLOCAL,
            (NewLength as SIZE_T) * (core::mem::size_of::<LPVOID>() as SIZE_T),
            LMEM_ZEROINIT | LMEM_MOVEABLE,
        ) as *mut LPVOID;
    }

    if !NewSpyedBlocks.is_null() {
        Malloc32.SpyedBlocks = NewSpyedBlocks;
        Malloc32.SpyedBlockTableLength = NewLength;
    }

    if !NewSpyedBlocks.is_null() {
        1
    } else {
        0
    }
}

/// ifs.c:89 — add a location to the table.
unsafe fn AddMemoryLocation(pMem: LPVOID) -> BOOL {
    let mut Current: *mut LPVOID;

    /* allocate the table if not already allocated */
    if Malloc32.SpyedBlockTableLength == 0 && SetSpyedBlockTableLength(0x1000) == 0 {
        return 0;
    }

    /* find a free location */
    Current = Malloc32.SpyedBlocks;
    while !(*Current).is_null() {
        Current = Current.add(1);
        if Current >= Malloc32.SpyedBlocks.add(Malloc32.SpyedBlockTableLength as usize) {
            /* no more space in table, grow it */
            let old_length: DWORD = Malloc32.SpyedBlockTableLength;
            if SetSpyedBlockTableLength(Malloc32.SpyedBlockTableLength + 0x1000) == 0 {
                return 0;
            }
            Current = Malloc32.SpyedBlocks.add(old_length as usize);
        }
    }

    /* put the location in our table */
    *Current = pMem;
    Malloc32.SpyedAllocationsLeft = Malloc32.SpyedAllocationsLeft.wrapping_add(1);
    1
}

/// ifs.c:131
unsafe fn RemoveMemoryLocation(pMem: LPCVOID) -> BOOL {
    let Current: *mut LPVOID;

    /* allocate the table if not already allocated */
    if Malloc32.SpyedBlockTableLength == 0 && SetSpyedBlockTableLength(0x1000) == 0 {
        return 0;
    }

    Current = mallocspy_is_allocation_spyed(pMem);
    if Current.is_null() {
        return 0;
    }

    /* location found */
    Malloc32.SpyedAllocationsLeft = Malloc32.SpyedAllocationsLeft.wrapping_sub(1);
    *Current = core::ptr::null_mut();
    1
}

/// com-sys spells the IMallocVtbl slots `unsafe extern "stdcall" fn` on i386 and
/// `unsafe extern "C" fn` on amd64/arm64 — i.e. exactly what STDMETHODCALLTYPE
/// compiles to per arch. `extern "system"` *lowers* to the same ABI, but rustc keeps
/// it a DISTINCT fn type that will not unify with the bindgen slot type, so the vtbl
/// impls must spell the ABI the way com-sys does. This macro does that once, so no
/// method has to carry a per-arch cfg.
macro_rules! stdmethod {
    ($(
        $(#[$attr:meta])*
        unsafe fn $name:ident ( $($arg:ident : $ty:ty),* $(,)? ) -> $ret:ty $body:block
    )*) => {$(
        $(#[$attr])*
        unsafe extern "system" fn $name($($arg: $ty),*) -> $ret $body
    )*};
}

stdmethod! {

/// ifs.c:152 IMalloc32_QueryInterface [VTABLE]
unsafe fn IMalloc_fnQueryInterface(
    iface: *mut IMalloc,
    refiid: *const IID,
    obj: *mut *mut c_void,
) -> HRESULT {
    let _ = iface;

    TRACE!(b"(%s,%p)\n\0".as_ptr(), debugstr_guid(refiid), obj);

    if IsEqualIID(core::ptr::addr_of!(IID_IUnknown), refiid)
        || IsEqualIID(core::ptr::addr_of!(IID_IMalloc), refiid)
    {
        *obj = core::ptr::addr_of_mut!(Malloc32) as *mut c_void;
        return S_OK;
    }
    E_NOINTERFACE
}

/// ifs.c:166 IMalloc32_AddRefRelease [VTABLE]
unsafe fn IMalloc_fnAddRefRelease(iface: *mut IMalloc) -> ULONG {
    let _ = iface;
    1
}

/// ifs.c:174 IMalloc32_Alloc [VTABLE]
unsafe fn IMalloc_fnAlloc(iface: *mut IMalloc, cb: SIZE_T) -> LPVOID {
    let _ = iface;
    let mut addr: LPVOID;

    TRACE!(b"(%ld)\n\0".as_ptr(), cb);

    if !Malloc32.pSpy.is_null() {
        let preAllocResult: SIZE_T;

        EnterCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
        preAllocResult = IMallocSpy_PreAlloc(Malloc32.pSpy, cb);
        if (cb != 0) && (preAllocResult == 0) {
            /* PreAlloc can force Alloc to fail, but not if cb == 0 */
            TRACE!(b"returning null\n\0".as_ptr());
            LeaveCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
            return core::ptr::null_mut();
        }
    }

    addr = HeapAlloc(GetProcessHeap(), 0, (cb) as usize);

    if !Malloc32.pSpy.is_null() {
        addr = IMallocSpy_PostAlloc(Malloc32.pSpy, addr);
        if !addr.is_null() {
            AddMemoryLocation(addr);
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
    }

    TRACE!(b"--(%p)\n\0".as_ptr(), addr);
    addr
}

/// ifs.c:208 IMalloc32_Realloc [VTABLE]
unsafe fn IMalloc_fnRealloc(iface: *mut IMalloc, pv: LPVOID, cb: SIZE_T) -> LPVOID {
    let _ = iface;
    let mut pNewMemory: LPVOID;
    let mut pv = pv;
    let mut cb = cb;

    TRACE!(b"(%p,%ld)\n\0".as_ptr(), pv, cb);

    if !Malloc32.pSpy.is_null() {
        let mut pRealMemory: LPVOID = core::ptr::null_mut();
        let fSpyed: BOOL;

        EnterCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
        fSpyed = RemoveMemoryLocation(pv as LPCVOID);
        cb = IMallocSpy_PreRealloc(
            Malloc32.pSpy,
            pv,
            cb,
            core::ptr::addr_of_mut!(pRealMemory),
            fSpyed,
        );

        /* check if can release the spy */
        if Malloc32.SpyReleasePending != 0 && Malloc32.SpyedAllocationsLeft == 0 {
            IMallocSpy_Release(Malloc32.pSpy);
            Malloc32.SpyReleasePending = 0;
            Malloc32.pSpy = core::ptr::null_mut();
            LeaveCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
        }

        if cb == 0 {
            /* PreRealloc can force Realloc to fail */
            if !Malloc32.pSpy.is_null() {
                LeaveCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
            }
            return core::ptr::null_mut();
        }

        pv = pRealMemory;
    }

    if pv.is_null() {
        pNewMemory = HeapAlloc(GetProcessHeap(), 0, (cb) as usize);
    } else if cb != 0 {
        pNewMemory = HeapReAlloc(GetProcessHeap(), 0, pv, cb);
    } else {
        HeapFree(GetProcessHeap(), 0, pv);
        pNewMemory = core::ptr::null_mut();
    }

    if !Malloc32.pSpy.is_null() {
        pNewMemory = IMallocSpy_PostRealloc(Malloc32.pSpy, pNewMemory, 1);
        if !pNewMemory.is_null() {
            AddMemoryLocation(pNewMemory);
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
    }

    TRACE!(b"--(%p)\n\0".as_ptr(), pNewMemory);
    pNewMemory
}

/// ifs.c:260 IMalloc32_Free [VTABLE]
unsafe fn IMalloc_fnFree(iface: *mut IMalloc, pv: LPVOID) -> () {
    let _ = iface;
    let mut fSpyed: BOOL = 0;
    let mut pv = pv;

    TRACE!(b"(%p)\n\0".as_ptr(), pv);

    if pv.is_null() {
        return;
    }

    if !Malloc32.pSpy.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
        fSpyed = RemoveMemoryLocation(pv as LPCVOID);
        pv = IMallocSpy_PreFree(Malloc32.pSpy, pv, fSpyed);
    }

    HeapFree(GetProcessHeap(), 0, pv);

    if !Malloc32.pSpy.is_null() {
        IMallocSpy_PostFree(Malloc32.pSpy, fSpyed);

        /* check if can release the spy */
        if Malloc32.SpyReleasePending != 0 && Malloc32.SpyedAllocationsLeft == 0 {
            IMallocSpy_Release(Malloc32.pSpy);
            Malloc32.SpyReleasePending = 0;
            Malloc32.pSpy = core::ptr::null_mut();
        }

        LeaveCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
    }
}

/// ifs.c:299 IMalloc32_GetSize [VTABLE]
unsafe fn IMalloc_fnGetSize(iface: *mut IMalloc, mem: LPVOID) -> SIZE_T {
    let _ = iface;
    let mut spyed_block: BOOL = 0;
    let mut spy_active: BOOL = 0;
    let mut size: SIZE_T;
    let mut mem = mem;

    TRACE!(b"(%p)\n\0".as_ptr(), mem);

    if mem.is_null() {
        return !0; /* (SIZE_T)-1 */
    }

    if !Malloc32.pSpy.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
        spyed_block = (!mallocspy_is_allocation_spyed(mem as LPCVOID).is_null()) as BOOL;
        spy_active = 1;
        mem = IMallocSpy_PreGetSize(Malloc32.pSpy, mem, spyed_block);
    }

    size = HeapSize(GetProcessHeap(), 0, mem as LPCVOID);

    if spy_active != 0 {
        size = IMallocSpy_PostGetSize(Malloc32.pSpy, size, spyed_block);
        LeaveCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
    }
    size
}

/// ifs.c:330 IMalloc32_DidAlloc [VTABLE]
unsafe fn IMalloc_fnDidAlloc(iface: *mut IMalloc, mem: LPVOID) -> c_int {
    let _ = iface;
    let mut spyed_block: BOOL = 0;
    let mut spy_active: BOOL = 0;
    let mut did_alloc: c_int;
    let mut mem = mem;

    TRACE!(b"(%p)\n\0".as_ptr(), mem);

    if mem.is_null() {
        return -1;
    }

    if !Malloc32.pSpy.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
        spyed_block = (!mallocspy_is_allocation_spyed(mem as LPCVOID).is_null()) as BOOL;
        spy_active = 1;
        mem = IMallocSpy_PreDidAlloc(Malloc32.pSpy, mem, spyed_block);
    }

    did_alloc = HeapValidate(GetProcessHeap(), 0, mem as LPCVOID) as c_int;

    if spy_active != 0 {
        did_alloc = IMallocSpy_PostDidAlloc(Malloc32.pSpy, mem, spyed_block, did_alloc);
        LeaveCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
    }

    did_alloc
}

/// ifs.c:362 IMalloc32_HeapMinimize [VTABLE]
unsafe fn IMalloc_fnHeapMinimize(iface: *mut IMalloc) -> () {
    let _ = iface;

    TRACE!(b"()\n\0".as_ptr());

    if !Malloc32.pSpy.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
        IMallocSpy_PreHeapMinimize(Malloc32.pSpy);
    }

    if !Malloc32.pSpy.is_null() {
        IMallocSpy_PostHeapMinimize(Malloc32.pSpy);
        LeaveCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));
    }
}

} // stdmethod!

/// ifs.c:377
pub static VT_IMalloc32: IMallocVtbl = IMallocVtbl {
    QueryInterface: Some(IMalloc_fnQueryInterface),
    AddRef: Some(IMalloc_fnAddRefRelease),
    Release: Some(IMalloc_fnAddRefRelease),
    Alloc: Some(IMalloc_fnAlloc),
    Realloc: Some(IMalloc_fnRealloc),
    Free: Some(IMalloc_fnFree),
    GetSize: Some(IMalloc_fnGetSize),
    DidAlloc: Some(IMalloc_fnDidAlloc),
    HeapMinimize: Some(IMalloc_fnHeapMinimize),
};

/// ifs.c:403 CoGetMalloc [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn CoGetMalloc(context: DWORD, imalloc: *mut *mut IMalloc) -> HRESULT {
    if context != MEMCTX_TASK {
        *imalloc = core::ptr::null_mut();
        return E_INVALIDARG;
    }

    *imalloc = core::ptr::addr_of_mut!(Malloc32.IMalloc_iface);
    S_OK
}

/// ifs.c:426 CoTaskMemAlloc [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID {
    IMalloc_Alloc(core::ptr::addr_of_mut!(Malloc32.IMalloc_iface), size)
}

/// ifs.c:442 CoTaskMemFree [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn CoTaskMemFree(ptr: LPVOID) {
    IMalloc_Free(core::ptr::addr_of_mut!(Malloc32.IMalloc_iface), ptr);
}

/// ifs.c:460 CoTaskMemRealloc [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn CoTaskMemRealloc(pvOld: LPVOID, size: SIZE_T) -> LPVOID {
    IMalloc_Realloc(
        core::ptr::addr_of_mut!(Malloc32.IMalloc_iface),
        pvOld,
        size,
    )
}

/// ifs.c:482 CoRegisterMallocSpy [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn CoRegisterMallocSpy(pMallocSpy: LPMALLOCSPY) -> HRESULT {
    let mut pSpy: *mut IMallocSpy = core::ptr::null_mut();
    let mut hres: HRESULT = E_INVALIDARG;

    TRACE!(b"%p\n\0".as_ptr(), pMallocSpy);

    if pMallocSpy.is_null() {
        return E_INVALIDARG;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));

    if !Malloc32.pSpy.is_null() {
        hres = CO_E_OBJISREG;
    } else if SUCCEEDED(IMallocSpy_QueryInterface(
        pMallocSpy,
        core::ptr::addr_of!(IID_IMallocSpy),
        core::ptr::addr_of_mut!(pSpy) as *mut *mut c_void,
    )) {
        Malloc32.pSpy = pSpy;
        hres = S_OK;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));

    hres
}

/// ifs.c:522 CoRevokeMallocSpy [OLE32.@]
#[no_mangle]
pub unsafe extern "system" fn CoRevokeMallocSpy() -> HRESULT {
    let mut hres: HRESULT = S_OK;
    TRACE!(b"\n\0".as_ptr());

    EnterCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));

    if Malloc32.pSpy.is_null() {
        hres = CO_E_OBJNOTREG;
    } else if Malloc32.SpyedAllocationsLeft != 0 {
        TRACE!(
            b"SpyReleasePending with %u allocations left\n\0".as_ptr(),
            Malloc32.SpyedAllocationsLeft
        );
        Malloc32.SpyReleasePending = 1;
        hres = E_ACCESSDENIED;
    } else {
        IMallocSpy_Release(Malloc32.pSpy);
        Malloc32.pSpy = core::ptr::null_mut();
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(IMalloc32_SpyCS));

    hres
}

/// ifs.c:555 IsValidInterface [OLE32.@]
///
/// The C is a single short-circuiting `!( A || B || C || D )`: each dereference is
/// guarded by the IsBadReadPtr that precedes it. Preserve that order — do NOT hoist
/// the `punk->lpVtbl` / `->QueryInterface` loads above their guards.
#[no_mangle]
pub unsafe extern "system" fn IsValidInterface(punk: LPUNKNOWN) -> BOOL {
    if IsBadReadPtr(punk as LPCVOID, 4) != 0 {
        return 0;
    }

    let vtbl = (*punk).lpVtbl;
    if IsBadReadPtr(vtbl as LPCVOID, 4) != 0 {
        return 0;
    }

    /* the raw vtbl slot value: an empty Option is a NULL function pointer */
    let query_interface: *const c_void = match (*vtbl).QueryInterface {
        Some(f) => f as *const () as *const c_void,
        None => core::ptr::null(),
    };
    if IsBadReadPtr(query_interface as LPCVOID, 9) != 0 {
        return 0;
    }
    if IsBadCodePtr(query_interface as *mut c_void as FARPROC) != 0 {
        return 0;
    }

    1
}
