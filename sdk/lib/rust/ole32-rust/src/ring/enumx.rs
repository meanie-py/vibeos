use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
//
// enumx.c:36 — `struct tagEnumSTATPROPSETSTG_impl` (the private enumerator object;
// enumx.h:22 typedefs it as the opaque `enumx_impl`).
#[repr(C)]
pub struct tagEnumSTATPROPSETSTG_impl {
    pub vtbl: *const c_void,
    pub ref_: LONG,
    pub elements: list,
    pub current: *mut list,
    pub elem_size: ULONG,
    pub riid: GUID,
    pub parent: *mut IUnknown,
    pub copy_cb: enumx_copy_cb,
}

/// enumx.h:22 — `typedef struct tagEnumSTATPROPSETSTG_impl enumx_impl;`
pub type enumx_impl = tagEnumSTATPROPSETSTG_impl;

pub type PtagEnumSTATPROPSETSTG_impl = *mut tagEnumSTATPROPSETSTG_impl;

/************************************************************************
 * enumx_QueryInterface
 */
#[no_mangle]
pub unsafe extern "system" fn enumx_QueryInterface(
    This: *mut enumx_impl,
    riid: *const GUID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualGUID(core::ptr::addr_of!(IID_IUnknown), riid)
        || IsEqualGUID(core::ptr::addr_of!((*This).riid), riid)
    {
        IUnknown_AddRef(This as *mut IUnknown);
        *ppvObject = This as *mut c_void;
        return S_OK;
    }

    E_NOINTERFACE
}

/************************************************************************
 * enumx_AddRef
 */
#[no_mangle]
pub unsafe extern "system" fn enumx_AddRef(This: *mut enumx_impl) -> ULONG {
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}

/************************************************************************
 * enumx_Release
 */
#[no_mangle]
pub unsafe extern "system" fn enumx_Release(This: *mut enumx_impl) -> ULONG {
    let ref_count: ULONG;

    ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;
    if ref_count == 0 {
        while list_empty(core::ptr::addr_of!((*This).elements)) == 0 {
            let x = list_head(core::ptr::addr_of!((*This).elements));
            list_remove(x);
            HeapFree(GetProcessHeap(), 0, x as LPVOID);
        }
        IUnknown_Release((*This).parent);
        HeapFree(GetProcessHeap(), 0, This as LPVOID);
    }
    ref_count
}

/************************************************************************
 * enumx_Next
 */
#[no_mangle]
pub unsafe extern "system" fn enumx_Next(
    This: *mut enumx_impl,
    celt: ULONG,
    rgelt: *mut c_void,
    pceltFetched: *mut ULONG,
) -> HRESULT {
    let mut p: *mut u8;
    let mut count: ULONG = 0;

    TRACE!("%p %u %p\n", This, celt, pceltFetched);

    if (*This).current.is_null() {
        (*This).current = list_head(core::ptr::addr_of!((*This).elements));
    }
    p = rgelt as *mut u8;
    while count < celt
        && !(*This).current.is_null()
        && (*This).current != core::ptr::addr_of_mut!((*This).elements)
    {
        // C: `&This->current[1]` — the element payload immediately after the
        // list node, i.e. the list pointer advanced by one `struct list`.
        let data = (*This).current.offset(1) as *mut c_void;
        if (*This).copy_cb.is_some() {
            ((*This).copy_cb.unwrap_unchecked())((*This).parent, data, p as *mut c_void);
        } else {
            core::ptr::copy_nonoverlapping(
                data as *const u8,
                p,
                (*This).elem_size as usize,
            );
        }
        p = p.offset((*This).elem_size as isize);
        (*This).current = (*(*This).current).next;
        count += 1;
    }
    if !pceltFetched.is_null() {
        *pceltFetched = count;
    }
    if count < celt {
        return S_FALSE;
    }
    S_OK
}

/************************************************************************
 * enumx_Skip
 */
#[no_mangle]
pub unsafe extern "system" fn enumx_Skip(This: *mut enumx_impl, celt: ULONG) -> HRESULT {
    let mut count: ULONG = 0;

    TRACE!("%p %u\n", This, celt);

    if (*This).current.is_null() {
        (*This).current = list_head(core::ptr::addr_of!((*This).elements));
    }

    // NOTE: faithful to the C oracle (dll/win32/ole32/enumx.c:145) — which never
    // advances `This->current` inside the loop. Upstream bug (the cursor should
    // step like enumx_Next does); NOT a translation error. Kept bug-compatible on
    // purpose; do not "fix" here without fixing the C oracle too.
    while count < celt
        && !(*This).current.is_null()
        && (*This).current != core::ptr::addr_of_mut!((*This).elements)
    {
        count += 1;
    }

    S_OK
}

/************************************************************************
 * enumx_Reset
 */
#[no_mangle]
pub unsafe extern "system" fn enumx_Reset(This: *mut enumx_impl) -> HRESULT {
    TRACE!("\n");

    (*This).current = core::ptr::null_mut();
    S_OK
}

/************************************************************************
 * enumx_fnClone
 */
#[no_mangle]
pub unsafe extern "system" fn enumx_Clone(
    _iface: *mut enumx_impl,
    _ppenum: *mut *mut enumx_impl,
) -> HRESULT {
    FIXME!("\n");
    E_NOTIMPL
}

/************************************************************************
 * enumx_allocate
 *
 * Allocate a generic enumerator
 */
#[no_mangle]
pub unsafe extern "C" fn enumx_allocate(
    riid: *const GUID,
    vtbl: *const c_void,
    elem_size: ULONG,
    parent: *mut IUnknown,
    copy_cb: enumx_copy_cb,
) -> *mut enumx_impl {
    let enumx = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<enumx_impl>() as usize,
    ) as *mut enumx_impl;

    if !enumx.is_null() {
        (*enumx).vtbl = vtbl;
        (*enumx).ref_ = 1;
        (*enumx).current = core::ptr::null_mut();
        (*enumx).elem_size = elem_size;
        (*enumx).riid = *riid;
        (*enumx).parent = parent;
        (*enumx).copy_cb = copy_cb;

        IUnknown_AddRef(parent);

        list_init(core::ptr::addr_of_mut!((*enumx).elements));
    }

    enumx
}

/************************************************************************
 * enumx_add_element
 *
 * Add an element to the enumeration.
 */
#[no_mangle]
pub unsafe extern "C" fn enumx_add_element(
    enumx: *mut enumx_impl,
    data: *const c_void,
) -> *mut c_void {
    let element: *mut list;

    element = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<list>() + (*enumx).elem_size as usize) as usize,
    ) as *mut list;
    if element.is_null() {
        return core::ptr::null_mut();
    }
    // C: `memcpy(&element[1], data, enumx->elem_size)` — payload sits right
    // after the embedded list node.
    core::ptr::copy_nonoverlapping(
        data as *const u8,
        element.offset(1) as *mut u8,
        (*enumx).elem_size as usize,
    );
    list_add_tail(core::ptr::addr_of_mut!((*enumx).elements), element);
    element.offset(1) as *mut c_void
}
