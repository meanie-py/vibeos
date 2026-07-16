use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch_ole: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"ole\0\0\0\0\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_ole as *mut __wine_debug_channel };

pub const MAXSINKS: i32 = 10;

#[repr(C)]
pub struct ConnectionPointImpl {
    pub IConnectionPoint_iface: IConnectionPoint,
    pub Obj: *mut IUnknown,
    pub ref_: LONG,
    pub iid: IID,
    pub sinks: *mut *mut IUnknown,
    pub maxSinks: DWORD,
    pub nSinks: DWORD,
}
pub type PConnectionPointImpl = *mut ConnectionPointImpl;

#[repr(C)]
pub struct EnumConnectionsImpl {
    pub IEnumConnections_iface: IEnumConnections,
    pub ref_: LONG,
    pub pUnk: *mut IUnknown,
    pub pCD: *mut CONNECTDATA,
    pub nConns: DWORD,
    pub nCur: DWORD,
}
pub type PEnumConnectionsImpl = *mut EnumConnectionsImpl;
unsafe extern "C" fn impl_from_IConnectionPoint(iface: *mut IConnectionPoint) -> *mut ConnectionPointImpl {
    (iface as ULONG_PTR).wrapping_sub(
        core::ptr::addr_of_mut!((*core::ptr::null_mut::<ConnectionPointImpl>()).IConnectionPoint_iface) as ULONG_PTR
    ) as *mut ConnectionPointImpl
}
unsafe extern "C" fn impl_from_IEnumConnections(
    iface: *mut IEnumConnections,
) -> *mut EnumConnectionsImpl {
    return (iface as ULONG_PTR).wrapping_sub(
        core::ptr::addr_of_mut!((*core::ptr::null_mut::<EnumConnectionsImpl>()).IEnumConnections_iface)
            as ULONG_PTR,
    ) as *mut EnumConnectionsImpl;
}
unsafe extern "C" fn ConnectionPointImpl_Destroy(Obj: *mut ConnectionPointImpl) {

    let mut i: DWORD = 0;
    while i < (*Obj).maxSinks {
        if !(*(*Obj).sinks.offset(i as isize)).is_null() {
            IUnknown_Release(*(*Obj).sinks.offset(i as isize));
            *(*Obj).sinks.offset(i as isize) = core::ptr::null_mut();
        }
        i = i.wrapping_add(1);
    }
    HeapFree(GetProcessHeap(), 0, (*Obj).sinks as LPVOID);
    HeapFree(GetProcessHeap(), 0, Obj as LPVOID);
}
stdmethod! {
fn ConnectionPointImpl_QueryInterface(
    iface: *mut IConnectionPoint,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut ConnectionPointImpl = impl_from_IConnectionPoint(iface);
    TRACE!(b"(%p)->(%s, %p)\n\0".as_ptr() as *const i8, This, debugstr_guid(riid), ppvObject);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut::<core::ffi::c_void>();

    if IsEqualIID(&IID_IConnectionPoint as *const IID, riid) || IsEqualIID(&IID_IUnknown as *const IID, riid) {
        *ppvObject = iface as *mut core::ffi::c_void;
    }

    if (*ppvObject).is_null() {
        FIXME!(b"() : asking for unsupported interface %s\n\0".as_ptr() as *const i8, debugstr_guid(riid));
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppvObject as *mut IUnknown);

    return S_OK;
}
}
stdmethod! {
fn ConnectionPointImpl_AddRef(iface: *mut IConnectionPoint) -> ULONG {
    let This: *mut ConnectionPointImpl = impl_from_IConnectionPoint(iface);
    let refCount: ULONG = InterlockedIncrement(&mut (*This).ref_) as ULONG;

    TRACE!(
        b"(%p)->(ref before=%d)\n\0".as_ptr() as *const i8,
        This,
        refCount.wrapping_sub(1)
    );

    return refCount;
}
}
stdmethod! {
fn ConnectionPointImpl_Release(
    iface: *mut IConnectionPoint
) -> ULONG {
    let This: *mut ConnectionPointImpl = impl_from_IConnectionPoint(iface);
    let refCount: ULONG = InterlockedDecrement(&mut (*This).ref_) as ULONG;

    TRACE!(b"(%p)->(ref before=%d)\n\0".as_ptr() as *const i8, This, refCount.wrapping_add(1));

    if refCount == 0 {
        ConnectionPointImpl_Destroy(This);
    }

    refCount
}
}
stdmethod! {
fn ConnectionPointImpl_GetConnectionInterface(
    iface: *mut IConnectionPoint,
    piid: *mut IID,
) -> HRESULT {
    let This: *mut ConnectionPointImpl = impl_from_IConnectionPoint(iface);
    TRACE!(
        b"(%p)->(%p) returning %s\n\0".as_ptr() as *const i8,
        This,
        piid,
        debugstr_guid(core::ptr::addr_of_mut!((*This).iid)),
    );
    *piid = (*This).iid;
    S_OK
}
}
stdmethod! {
fn ConnectionPointImpl_GetConnectionPointContainer(
    iface: *mut IConnectionPoint,
    ppCPC: *mut *mut IConnectionPointContainer,
) -> HRESULT {
    let This: *mut ConnectionPointImpl = impl_from_IConnectionPoint(iface);
    
    if (*__wine_dbch___default).flags as i32
        & (1i32 << __WINE_DBCL_TRACE as i32)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/connpt.c\0"
                as *const u8 as *const i8,
            b"ConnectionPointImpl_GetConnectionPointContainer\0" as *const u8
                as *const i8,
            223i32,
            b"(%p)->(%p)\n\0" as *const u8 as *const i8,
            This,
            ppCPC,
        );
    }
    
    return (*(*(*This).Obj).lpVtbl)
        .QueryInterface
        .expect("non-null function pointer")(
        (*This).Obj,
        core::ptr::addr_of!(IID_IConnectionPointContainer),
        ppCPC as *mut *mut core::ffi::c_void,
    );
}
}
stdmethod! {
fn ConnectionPointImpl_Advise(
    iface: *mut IConnectionPoint,
    lpUnk: *mut IUnknown,
    pdwCookie: *mut DWORD,
) -> HRESULT {
    let mut i: DWORD;
    let This: *mut ConnectionPointImpl = impl_from_IConnectionPoint(iface);
    let mut lpSink: *mut IUnknown = core::ptr::null_mut();
    
    TRACE!(b"(%p)->(%p, %p)\n\0".as_ptr() as *const i8, This, lpUnk, pdwCookie);
    
    *pdwCookie = 0;
    
    if FAILED(IUnknown_QueryInterface(
        lpUnk,
        core::ptr::addr_of_mut!((*This).iid),
        core::ptr::addr_of_mut!(lpSink) as *mut *mut core::ffi::c_void,
    )) {
        return CONNECT_E_CANNOTCONNECT;
    }
    
    i = 0;
    while i < (*This).maxSinks {
        if (*(*This).sinks.offset(i as isize)).is_null() {
            break;
        }
        i = i.wrapping_add(1);
    }
    
    if i == (*This).maxSinks {
        (*This).maxSinks = (*This).maxSinks.wrapping_add(MAXSINKS as DWORD);
        (*This).sinks = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*This).sinks as *mut core::ffi::c_void,
            ((*This).maxSinks as SIZE_T)
                .wrapping_mul(core::mem::size_of::<*mut IUnknown>() as SIZE_T),
        ) as *mut *mut IUnknown;
    }
    
    *(*This).sinks.offset(i as isize) = lpSink;
    (*This).nSinks = (*This).nSinks.wrapping_add(1);
    *pdwCookie = i.wrapping_add(1);
    
    S_OK
}
}
stdmethod! {
fn ConnectionPointImpl_Unadvise(
    iface: *mut IConnectionPoint,
    dwCookie: DWORD,
) -> HRESULT {
    let This: *mut ConnectionPointImpl = impl_from_IConnectionPoint(iface);
    
    
    TRACE!(b"(%p)->(%d)\n\0".as_ptr(), This, dwCookie);

    if dwCookie == 0 || dwCookie > (*This).maxSinks {
        return E_INVALIDARG;
    }

    if (*(*This).sinks.offset(dwCookie.wrapping_sub(1) as isize)).is_null() {
        return CONNECT_E_NOCONNECTION;
    }

    IUnknown_Release(*(*This).sinks.offset(dwCookie.wrapping_sub(1) as isize));
    *(*This).sinks.offset(dwCookie.wrapping_sub(1) as isize) = core::ptr::null_mut();
    (*This).nSinks = (*This).nSinks.wrapping_sub(1);
    
    S_OK
}
}
stdmethod! {
fn ConnectionPointImpl_EnumConnections(
    iface: *mut IConnectionPoint,
    ppEnum: *mut LPENUMCONNECTIONS,
) -> HRESULT {
    let This: *mut ConnectionPointImpl = impl_from_IConnectionPoint(iface);
    let mut pCD: *mut CONNECTDATA;
    let mut i: DWORD;
    let mut nextslot: DWORD;
    let EnumObj: *mut EnumConnectionsImpl;
    let hr: HRESULT;

    TRACE!(
        b"(%p)->(%p)\n\0".as_ptr() as *const i8,
        This,
        ppEnum,
    );

    *ppEnum = core::ptr::null_mut();

    if (*This).nSinks == 0 {
        return OLE_E_NOCONNECTION;
    }

    pCD = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<CONNECTDATA>() * (*This).nSinks as usize) as SIZE_T,
    ) as *mut CONNECTDATA;

    i = 0;
    nextslot = 0;
    while i < (*This).maxSinks {
        if !(*(*This).sinks.offset(i as isize)).is_null() {
            (*pCD.offset(nextslot as isize)).pUnk = *(*This).sinks.offset(i as isize);
            (*pCD.offset(nextslot as isize)).dwCookie = i + 1;
            nextslot = nextslot + 1;
        }
        i = i + 1;
    }

    assert(nextslot == (*This).nSinks);

    IConnectionPoint_AddRef(iface);

    EnumObj = EnumConnectionsImpl_Construct(iface as *mut IUnknown, (*This).nSinks, pCD);
    hr = IEnumConnections_QueryInterface(
        core::ptr::addr_of_mut!((*EnumObj).IEnumConnections_iface),
        core::ptr::addr_of!(IID_IEnumConnections),
        ppEnum as *mut *mut core::ffi::c_void,
    );
    IEnumConnections_Release(core::ptr::addr_of_mut!((*EnumObj).IEnumConnections_iface));

    HeapFree(GetProcessHeap(), 0, pCD as PVOID);
    return hr;
}
}
unsafe extern "C" fn EnumConnectionsImpl_Construct(
    pUnk: *mut IUnknown,
    nSinks: DWORD,
    pCD: *mut CONNECTDATA,
) -> *mut EnumConnectionsImpl {

    let Obj: *mut EnumConnectionsImpl = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<EnumConnectionsImpl>() as SIZE_T,
    ) as *mut EnumConnectionsImpl;

    (*Obj).IEnumConnections_iface.lpVtbl =
        core::ptr::addr_of!(EnumConnectionsImpl_VTable) as *mut IEnumConnectionsVtbl;
    (*Obj).ref_ = 1;
    (*Obj).pUnk = pUnk;
    (*Obj).pCD = HeapAlloc(
        GetProcessHeap(),
        0,
        (nSinks as SIZE_T).wrapping_mul(core::mem::size_of::<CONNECTDATA>() as SIZE_T),
    ) as *mut CONNECTDATA;
    (*Obj).nConns = nSinks;
    (*Obj).nCur = 0;

    let mut i: DWORD = 0;
    while i < nSinks {
        *(*Obj).pCD.offset(i as isize) = *pCD.offset(i as isize);
        IUnknown_AddRef((*(*Obj).pCD.offset(i as isize)).pUnk);
        i = i.wrapping_add(1);
    }

    Obj
}
unsafe extern "C" fn EnumConnectionsImpl_Destroy(Obj: *mut EnumConnectionsImpl) {
    let mut i: DWORD = 0;
    
    while i < (*Obj).nConns {
        IUnknown_Release((*(*Obj).pCD.offset(i as isize)).pUnk);
        i = i.wrapping_add(1);
    }
    
    HeapFree(GetProcessHeap(), 0, (*Obj).pCD as PVOID);
    HeapFree(GetProcessHeap(), 0, Obj as PVOID);
}
stdmethod! {
fn EnumConnectionsImpl_QueryInterface(
    iface: *mut IEnumConnections,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut ConnectionPointImpl = iface as *mut ConnectionPointImpl;
    TRACE!(
        b"(%p)->(%s, %p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_guid(riid),
        ppvObject,
    );

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(riid, core::ptr::addr_of!(IID_IEnumConnections))
        || IsEqualIID(riid, core::ptr::addr_of!(IID_IUnknown))
    {
        *ppvObject = iface as *mut core::ffi::c_void;
    }

    if (*ppvObject).is_null() {
        FIXME!(
            b"() : asking for unsupported interface %s\n\0".as_ptr() as *const i8,
            debugstr_guid(riid),
        );
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*ppvObject as *mut IUnknown);

    return S_OK;
}
}
stdmethod! {
fn EnumConnectionsImpl_AddRef(iface: *mut IEnumConnections) -> ULONG {
    let This: *mut EnumConnectionsImpl = impl_from_IEnumConnections(iface);
    let refCount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    TRACE!(
        b"(%p)->(ref before=%d)\n\0".as_ptr() as *const i8,
        This,
        refCount.wrapping_sub(1),
    );

    IUnknown_AddRef((*This).pUnk);
    return refCount;
}
}
stdmethod! {
fn EnumConnectionsImpl_Release(iface: *mut IEnumConnections) -> ULONG {
    let This: *mut EnumConnectionsImpl = impl_from_IEnumConnections(iface);
    let refCount: ULONG = InterlockedDecrement(&mut (*This).ref_) as ULONG;

    TRACE!(b"(%p)->(ref before=%d)\n\0".as_ptr() as *const i8, This, refCount.wrapping_add(1));

    IUnknown_Release((*This).pUnk);

    if refCount == 0 {
        EnumConnectionsImpl_Destroy(This);
    }

    refCount
}
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn EnumConnectionsImpl_Next(
    iface: *mut IEnumConnections,
    mut cConn: ULONG,
    mut pCD: LPCONNECTDATA,
    pEnum: *mut ULONG,
) -> HRESULT {
    let This: *mut EnumConnectionsImpl = impl_from_IEnumConnections(iface);
    let mut nRet: DWORD = 0;
    TRACE!(b"(%p)->(%d, %p, %p)\n\0".as_ptr(), This, cConn, pCD, pEnum);

    if pEnum.is_null() {
        if cConn != 1 {
            return E_POINTER;
        }
    } else {
        *pEnum = 0;
    }

    if (*This).nCur >= (*This).nConns {
        return S_FALSE;
    }

    while (*This).nCur < (*This).nConns && cConn != 0 {
        let fresh_pCD = pCD;
        pCD = pCD.offset(1);
        *fresh_pCD = *(*This).pCD.offset((*This).nCur as isize);
        IUnknown_AddRef((*(*This).pCD.offset((*This).nCur as isize)).pUnk);
        (*This).nCur = (*This).nCur.wrapping_add(1);
        cConn = cConn.wrapping_sub(1);
        nRet = nRet.wrapping_add(1);
    }

    if !pEnum.is_null() {
        *pEnum = nRet;
    }

    S_OK
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn EnumConnectionsImpl_Next(
    iface: *mut IEnumConnections,
    mut cConn: ULONG,
    mut pCD: LPCONNECTDATA,
    pEnum: *mut ULONG,
) -> HRESULT {
    let This: *mut EnumConnectionsImpl = impl_from_IEnumConnections(iface);
    let mut nRet: DWORD = 0;
    TRACE!(b"(%p)->(%d, %p, %p)\n\0".as_ptr(), This, cConn, pCD, pEnum);

    if pEnum.is_null() {
        if cConn != 1 {
            return E_POINTER;
        }
    } else {
        *pEnum = 0;
    }

    if (*This).nCur >= (*This).nConns {
        return S_FALSE;
    }

    while (*This).nCur < (*This).nConns && cConn != 0 {
        let fresh_pCD = pCD;
        pCD = pCD.offset(1);
        *fresh_pCD = *(*This).pCD.offset((*This).nCur as isize);
        IUnknown_AddRef((*(*This).pCD.offset((*This).nCur as isize)).pUnk);
        (*This).nCur = (*This).nCur.wrapping_add(1);
        cConn = cConn.wrapping_sub(1);
        nRet = nRet.wrapping_add(1);
    }

    if !pEnum.is_null() {
        *pEnum = nRet;
    }

    S_OK
}
stdmethod! {
fn EnumConnectionsImpl_Skip(
    iface: *mut IEnumConnections,
    cSkip: ULONG,
) -> HRESULT {
    let This: *mut EnumConnectionsImpl = impl_from_IEnumConnections(iface);
    TRACE!(b"(%p)->(%d)\n\0".as_ptr() as *const i8, This, cSkip);

    if ((*This).nCur as ULONG).wrapping_add(cSkip) >= (*This).nConns {
        return S_FALSE;
    }

    (*This).nCur = ((*This).nCur as ULONG).wrapping_add(cSkip) as DWORD;

    S_OK
}
}
stdmethod! {
fn EnumConnectionsImpl_Reset(iface: *mut IEnumConnections) -> HRESULT {
    let This: *mut EnumConnectionsImpl = impl_from_IEnumConnections(iface);
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, This);

    (*This).nCur = 0;

    S_OK
}
}
stdmethod! {
fn EnumConnectionsImpl_Clone(
    iface: *mut IEnumConnections,
    ppEnum: *mut LPENUMCONNECTIONS,
) -> HRESULT {
    let This: *mut EnumConnectionsImpl = impl_from_IEnumConnections(iface);
    let newObj: *mut EnumConnectionsImpl;
    
    TRACE!(
        b"(%p)->(%p)\n\0".as_ptr() as *const i8,
        This,
        ppEnum,
    );
    
    newObj = EnumConnectionsImpl_Construct((*This).pUnk, (*This).nConns, (*This).pCD);
    (*newObj).nCur = (*This).nCur;
    *ppEnum = core::ptr::addr_of_mut!((*newObj).IEnumConnections_iface) as LPENUMCONNECTIONS;
    IUnknown_AddRef((*This).pUnk);
    
    S_OK
}
}
#[no_mangle]
pub unsafe extern "C" fn CreateConnectionPoint(
    pUnk: *mut IUnknown,
    riid: *const IID,
    pCP: *mut *mut IConnectionPoint,
) -> HRESULT {

    TRACE!(
        b"(%p %s %p)\n\0".as_ptr() as PCSTR,
        pUnk,
        debugstr_guid(riid),
        pCP,
    );

    *pCP = core::ptr::null_mut();
    let Obj = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<ConnectionPointImpl>() as SIZE_T,
    ) as *mut ConnectionPointImpl;
    if Obj.is_null() {
        return E_OUTOFMEMORY;
    }

    (*Obj).IConnectionPoint_iface.lpVtbl =
        core::ptr::addr_of!(ConnectionPointImpl_VTable) as *mut IConnectionPointVtbl;
    (*Obj).Obj = pUnk;
    (*Obj).ref_ = 1;
    (*Obj).iid = *riid;
    (*Obj).maxSinks = MAXSINKS as DWORD;
    (*Obj).sinks = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<*mut IUnknown>() * MAXSINKS as usize) as SIZE_T,
    ) as *mut *mut IUnknown;
    (*Obj).nSinks = 0;

    *pCP = &mut (*Obj).IConnectionPoint_iface;
    S_OK
}

// COM vtable tables — generated from connpt.c by vtbl_gen.py
/// connpt.c:325 `static const IConnectionPointVtbl ConnectionPointImpl_VTable`  (slot order: com-sys/bindgen_i386.rs)
pub static ConnectionPointImpl_VTable: IConnectionPointVtbl = IConnectionPointVtbl {
    QueryInterface: Some(ConnectionPointImpl_QueryInterface),
    AddRef: Some(ConnectionPointImpl_AddRef),
    Release: Some(ConnectionPointImpl_Release),
    GetConnectionInterface: Some(ConnectionPointImpl_GetConnectionInterface),
    GetConnectionPointContainer: Some(ConnectionPointImpl_GetConnectionPointContainer),
    Advise: Some(ConnectionPointImpl_Advise),
    Unadvise: Some(ConnectionPointImpl_Unadvise),
    EnumConnections: Some(ConnectionPointImpl_EnumConnections),
};

/// connpt.c:547 `static const IEnumConnectionsVtbl EnumConnectionsImpl_VTable`  (slot order: com-sys/bindgen_i386.rs)
pub static EnumConnectionsImpl_VTable: IEnumConnectionsVtbl = IEnumConnectionsVtbl {
    QueryInterface: Some(EnumConnectionsImpl_QueryInterface),
    AddRef: Some(EnumConnectionsImpl_AddRef),
    Release: Some(EnumConnectionsImpl_Release),
    Next: Some(EnumConnectionsImpl_Next),
    Skip: Some(EnumConnectionsImpl_Skip),
    Reset: Some(EnumConnectionsImpl_Reset),
    Clone: Some(EnumConnectionsImpl_Clone),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Implementation of a generic ConnectionPoint object.
//  *
//  * Copyright 2000 Huw D M Davies for CodeWeavers
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
//  * NOTES:
//  * See one exported function here is CreateConnectionPoint, see
//  * comments just above that function for information.
//  */
//
// #include <assert.h>
// #include <stdarg.h>
// #include <string.h>
//
// #define COBJMACROS
//
// #include "winerror.h"
// #include "windef.h"
// #include "winbase.h"
// #include "wingdi.h"
// #include "winuser.h"
// #include "ole2.h"
// #include "olectl.h"
// #include "connpt.h"
//
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(ole);
//
// #define MAXSINKS 10
//
// /************************************************************************
//  * Implementation of IConnectionPoint
//  */
// typedef struct ConnectionPointImpl {
//
//   IConnectionPoint IConnectionPoint_iface;
//
//   /* IUnknown of our main object*/
//   IUnknown *Obj;
//
//   /* Reference count */
//   LONG ref;
//
//   /* IID of sink interface */
//   IID iid;
//
//   /* Array of sink IUnknowns */
//   IUnknown **sinks;
//   DWORD maxSinks;
//
//   DWORD nSinks;
// } ConnectionPointImpl;
//
// /************************************************************************
//  * Implementation of IEnumConnections
//  */
// typedef struct EnumConnectionsImpl {
//
//   IEnumConnections IEnumConnections_iface;
//
//   LONG ref;
//
//   /* IUnknown of ConnectionPoint, used for ref counting */
//   IUnknown *pUnk;
//
//   /* Connection Data */
//   CONNECTDATA *pCD;
//   DWORD nConns;
//
//   /* Next connection to enumerate from */
//   DWORD nCur;
//
// } EnumConnectionsImpl;
//
// static EnumConnectionsImpl *EnumConnectionsImpl_Construct(IUnknown *pUnk,
// 							  DWORD nSinks,
// 							  CONNECTDATA *pCD);
//
// static inline ConnectionPointImpl *impl_from_IConnectionPoint(IConnectionPoint *iface)
// {
//   return CONTAINING_RECORD(iface, ConnectionPointImpl, IConnectionPoint_iface);
// }
//
// static inline EnumConnectionsImpl *impl_from_IEnumConnections(IEnumConnections *iface)
// {
//   return CONTAINING_RECORD(iface, EnumConnectionsImpl, IEnumConnections_iface);
// }
//
// /************************************************************************
//  * ConnectionPointImpl_Destroy
//  */
// static void ConnectionPointImpl_Destroy(ConnectionPointImpl *Obj)
// {
//   DWORD i;
//   for(i = 0; i < Obj->maxSinks; i++) {
//     if(Obj->sinks[i]) {
//       IUnknown_Release(Obj->sinks[i]);
//       Obj->sinks[i] = NULL;
//     }
//   }
//   HeapFree(GetProcessHeap(), 0, Obj->sinks);
//   HeapFree(GetProcessHeap(), 0, Obj);
//   return;
// }
//
// /************************************************************************
//  * ConnectionPointImpl_QueryInterface (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static HRESULT WINAPI ConnectionPointImpl_QueryInterface(
//   IConnectionPoint*  iface,
//   REFIID  riid,
//   void**  ppvObject)
// {
//   ConnectionPointImpl *This = impl_from_IConnectionPoint(iface);
//   TRACE("(%p)->(%s, %p)\n", This, debugstr_guid(riid), ppvObject);
//
//   /*
//    * Perform a sanity check on the parameters.
//    */
//   if (!ppvObject)
//     return E_INVALIDARG;
//
//   /*
//    * Initialize the return parameter.
//    */
//   *ppvObject = 0;
//
//
//   if (IsEqualIID(&IID_IConnectionPoint, riid) || IsEqualIID(&IID_IUnknown, riid))
//     *ppvObject = iface;
//
//   /*
//    * Check that we obtained an interface.
//    */
//   if ((*ppvObject)==0)
//   {
//     FIXME("() : asking for unsupported interface %s\n", debugstr_guid(riid));
//     return E_NOINTERFACE;
//   }
//
//   IUnknown_AddRef((IUnknown*)*ppvObject);
//
//   return S_OK;
// }
//
//
// /************************************************************************
//  * ConnectionPointImpl_AddRef (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static ULONG WINAPI ConnectionPointImpl_AddRef(IConnectionPoint* iface)
// {
//   ConnectionPointImpl *This = impl_from_IConnectionPoint(iface);
//   ULONG refCount = InterlockedIncrement(&This->ref);
//
//   TRACE("(%p)->(ref before=%d)\n", This, refCount - 1);
//
//   return refCount;
// }
//
// /************************************************************************
//  * ConnectionPointImpl_Release (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static ULONG WINAPI ConnectionPointImpl_Release(
//       IConnectionPoint* iface)
// {
//   ConnectionPointImpl *This = impl_from_IConnectionPoint(iface);
//   ULONG refCount = InterlockedDecrement(&This->ref);
//
//   TRACE("(%p)->(ref before=%d)\n", This, refCount + 1);
//
//   /*
//    * If the reference count goes down to 0, perform suicide.
//    */
//   if (!refCount) ConnectionPointImpl_Destroy(This);
//
//   return refCount;
// }
//
// /************************************************************************
//  * ConnectionPointImpl_GetConnectionInterface (IConnectionPoint)
//  *
//  */
// static HRESULT WINAPI ConnectionPointImpl_GetConnectionInterface(
// 					       IConnectionPoint *iface,
// 					       IID              *piid)
// {
//   ConnectionPointImpl *This = impl_from_IConnectionPoint(iface);
//   TRACE("(%p)->(%p) returning %s\n", This, piid, debugstr_guid(&(This->iid)));
//   *piid = This->iid;
//   return S_OK;
// }
//
// /************************************************************************
//  * ConnectionPointImpl_GetConnectionPointContainer (IConnectionPoint)
//  *
//  */
// static HRESULT WINAPI ConnectionPointImpl_GetConnectionPointContainer(
// 				      IConnectionPoint           *iface,
// 				      IConnectionPointContainer  **ppCPC)
// {
//   ConnectionPointImpl *This = impl_from_IConnectionPoint(iface);
//   TRACE("(%p)->(%p)\n", This, ppCPC);
//
//   return IUnknown_QueryInterface(This->Obj, &IID_IConnectionPointContainer, (void**)ppCPC);
// }
//
// /************************************************************************
//  * ConnectionPointImpl_Advise (IConnectionPoint)
//  *
//  */
// static HRESULT WINAPI ConnectionPointImpl_Advise(IConnectionPoint *iface,
// 						 IUnknown *lpUnk,
// 						 DWORD *pdwCookie)
// {
//   DWORD i;
//   ConnectionPointImpl *This = impl_from_IConnectionPoint(iface);
//   IUnknown *lpSink;
//   TRACE("(%p)->(%p, %p)\n", This, lpUnk, pdwCookie);
//
//   *pdwCookie = 0;
//   if(FAILED(IUnknown_QueryInterface(lpUnk, &This->iid, (void**)&lpSink)))
//     return CONNECT_E_CANNOTCONNECT;
//
//   for(i = 0; i < This->maxSinks; i++) {
//     if(This->sinks[i] == NULL)
//       break;
//   }
//   if(i == This->maxSinks) {
//     This->maxSinks += MAXSINKS;
//     This->sinks = HeapReAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, This->sinks,
// 			      This->maxSinks * sizeof(IUnknown *));
//   }
//   This->sinks[i] = lpSink;
//   This->nSinks++;
//   *pdwCookie = i + 1;
//   return S_OK;
// }
//
//
// /************************************************************************
//  * ConnectionPointImpl_Unadvise (IConnectionPoint)
//  *
//  */
// static HRESULT WINAPI ConnectionPointImpl_Unadvise(IConnectionPoint *iface,
// 						   DWORD dwCookie)
// {
//   ConnectionPointImpl *This = impl_from_IConnectionPoint(iface);
//   TRACE("(%p)->(%d)\n", This, dwCookie);
//
//   if(dwCookie == 0 || dwCookie > This->maxSinks) return E_INVALIDARG;
//
//   if(This->sinks[dwCookie-1] == NULL) return CONNECT_E_NOCONNECTION;
//
//   IUnknown_Release(This->sinks[dwCookie-1]);
//   This->sinks[dwCookie-1] = NULL;
//   This->nSinks--;
//   return S_OK;
// }
//
// /************************************************************************
//  * ConnectionPointImpl_EnumConnections (IConnectionPoint)
//  *
//  */
// static HRESULT WINAPI ConnectionPointImpl_EnumConnections(
// 						    IConnectionPoint *iface,
// 						    LPENUMCONNECTIONS *ppEnum)
// {
//   ConnectionPointImpl *This = impl_from_IConnectionPoint(iface);
//   CONNECTDATA *pCD;
//   DWORD i, nextslot;
//   EnumConnectionsImpl *EnumObj;
//   HRESULT hr;
//
//   TRACE("(%p)->(%p)\n", This, ppEnum);
//
//   *ppEnum = NULL;
//
//   if(This->nSinks == 0) return OLE_E_NOCONNECTION;
//
//   pCD = HeapAlloc(GetProcessHeap(), 0, sizeof(CONNECTDATA) * This->nSinks);
//
//   for(i = 0, nextslot = 0; i < This->maxSinks; i++) {
//     if(This->sinks[i] != NULL) {
//       pCD[nextslot].pUnk = This->sinks[i];
//       pCD[nextslot].dwCookie = i + 1;
//       nextslot++;
//     }
//   }
//   assert(nextslot == This->nSinks);
//
//   /* Bump the ref count of this object up by one.  It gets Released in
//      IEnumConnections_Release */
//   IConnectionPoint_AddRef(iface);
//
//   EnumObj = EnumConnectionsImpl_Construct((IUnknown*)iface, This->nSinks, pCD);
//   hr = IEnumConnections_QueryInterface(&EnumObj->IEnumConnections_iface,
//                                        &IID_IEnumConnections, (void**)ppEnum);
//   IEnumConnections_Release(&EnumObj->IEnumConnections_iface);
//
//   HeapFree(GetProcessHeap(), 0, pCD);
//   return hr;
// }
//
// static const IConnectionPointVtbl ConnectionPointImpl_VTable =
// {
//   ConnectionPointImpl_QueryInterface,
//   ConnectionPointImpl_AddRef,
//   ConnectionPointImpl_Release,
//   ConnectionPointImpl_GetConnectionInterface,
//   ConnectionPointImpl_GetConnectionPointContainer,
//   ConnectionPointImpl_Advise,
//   ConnectionPointImpl_Unadvise,
//   ConnectionPointImpl_EnumConnections
// };
//
//
// static const IEnumConnectionsVtbl EnumConnectionsImpl_VTable;
//
// /************************************************************************
//  * EnumConnectionsImpl_Construct
//  */
// static EnumConnectionsImpl *EnumConnectionsImpl_Construct(IUnknown *pUnk,
// 							  DWORD nSinks,
// 							  CONNECTDATA *pCD)
// {
//   EnumConnectionsImpl *Obj = HeapAlloc(GetProcessHeap(), 0, sizeof(*Obj));
//   DWORD i;
//
//   Obj->IEnumConnections_iface.lpVtbl = &EnumConnectionsImpl_VTable;
//   Obj->ref = 1;
//   Obj->pUnk = pUnk;
//   Obj->pCD = HeapAlloc(GetProcessHeap(), 0, nSinks * sizeof(CONNECTDATA));
//   Obj->nConns = nSinks;
//   Obj->nCur = 0;
//
//   for(i = 0; i < nSinks; i++) {
//     Obj->pCD[i] = pCD[i];
//     IUnknown_AddRef(Obj->pCD[i].pUnk);
//   }
//   return Obj;
// }
//
// /************************************************************************
//  * EnumConnectionsImpl_Destroy
//  */
// static void EnumConnectionsImpl_Destroy(EnumConnectionsImpl *Obj)
// {
//   DWORD i;
//
//   for(i = 0; i < Obj->nConns; i++)
//     IUnknown_Release(Obj->pCD[i].pUnk);
//
//   HeapFree(GetProcessHeap(), 0, Obj->pCD);
//   HeapFree(GetProcessHeap(), 0, Obj);
//   return;
// }
//
// /************************************************************************
//  * EnumConnectionsImpl_QueryInterface (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static HRESULT WINAPI EnumConnectionsImpl_QueryInterface(
//   IEnumConnections*  iface,
//   REFIID  riid,
//   void**  ppvObject)
// {
//   ConnectionPointImpl *This = (ConnectionPointImpl *)iface;
//   TRACE("(%p)->(%s, %p)\n", This, debugstr_guid(riid), ppvObject);
//
//   /*
//    * Perform a sanity check on the parameters.
//    */
//   if (!ppvObject)
//     return E_INVALIDARG;
//
//   /*
//    * Initialize the return parameter.
//    */
//   *ppvObject = 0;
//
//   if (IsEqualIID(&IID_IEnumConnections, riid) || IsEqualIID(&IID_IUnknown, riid))
//     *ppvObject = iface;
//
//   /*
//    * Check that we obtained an interface.
//    */
//   if ((*ppvObject)==0)
//   {
//     FIXME("() : asking for unsupported interface %s\n", debugstr_guid(riid));
//     return E_NOINTERFACE;
//   }
//
//   IUnknown_AddRef((IUnknown*)*ppvObject);
//
//   return S_OK;
// }
//
//
// /************************************************************************
//  * EnumConnectionsImpl_AddRef (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static ULONG WINAPI EnumConnectionsImpl_AddRef(IEnumConnections* iface)
// {
//   EnumConnectionsImpl *This = impl_from_IEnumConnections(iface);
//   ULONG refCount = InterlockedIncrement(&This->ref);
//
//   TRACE("(%p)->(ref before=%d)\n", This, refCount - 1);
//
//   IUnknown_AddRef(This->pUnk);
//   return refCount;
// }
//
// /************************************************************************
//  * EnumConnectionsImpl_Release (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static ULONG WINAPI EnumConnectionsImpl_Release(IEnumConnections* iface)
// {
//   EnumConnectionsImpl *This = impl_from_IEnumConnections(iface);
//   ULONG refCount = InterlockedDecrement(&This->ref);
//
//   TRACE("(%p)->(ref before=%d)\n", This, refCount + 1);
//
//   IUnknown_Release(This->pUnk);
//
//   /*
//    * If the reference count goes down to 0, perform suicide.
//    */
//   if (!refCount) EnumConnectionsImpl_Destroy(This);
//
//   return refCount;
// }
//
// /************************************************************************
//  * EnumConnectionsImpl_Next (IEnumConnections)
//  *
//  */
// static HRESULT WINAPI EnumConnectionsImpl_Next(IEnumConnections* iface,
// 					       ULONG cConn, LPCONNECTDATA pCD,
// 					       ULONG *pEnum)
// {
//   EnumConnectionsImpl *This = impl_from_IEnumConnections(iface);
//   DWORD nRet = 0;
//   TRACE("(%p)->(%d, %p, %p)\n", This, cConn, pCD, pEnum);
//
//   if(pEnum == NULL) {
//     if(cConn != 1)
//       return E_POINTER;
//   } else
//     *pEnum = 0;
//
//   if(This->nCur >= This->nConns)
//     return S_FALSE;
//
//   while(This->nCur < This->nConns && cConn) {
//     *pCD++ = This->pCD[This->nCur];
//     IUnknown_AddRef(This->pCD[This->nCur].pUnk);
//     This->nCur++;
//     cConn--;
//     nRet++;
//   }
//
//   if(pEnum)
//     *pEnum = nRet;
//
//   return S_OK;
// }
//
//
// /************************************************************************
//  * EnumConnectionsImpl_Skip (IEnumConnections)
//  *
//  */
// static HRESULT WINAPI EnumConnectionsImpl_Skip(IEnumConnections* iface,
// 					       ULONG cSkip)
// {
//   EnumConnectionsImpl *This = impl_from_IEnumConnections(iface);
//   TRACE("(%p)->(%d)\n", This, cSkip);
//
//   if(This->nCur + cSkip >= This->nConns)
//     return S_FALSE;
//
//   This->nCur += cSkip;
//
//   return S_OK;
// }
//
//
// /************************************************************************
//  * EnumConnectionsImpl_Reset (IEnumConnections)
//  *
//  */
// static HRESULT WINAPI EnumConnectionsImpl_Reset(IEnumConnections* iface)
// {
//   EnumConnectionsImpl *This = impl_from_IEnumConnections(iface);
//   TRACE("(%p)\n", This);
//
//   This->nCur = 0;
//
//   return S_OK;
// }
//
//
// /************************************************************************
//  * EnumConnectionsImpl_Clone (IEnumConnections)
//  *
//  */
// static HRESULT WINAPI EnumConnectionsImpl_Clone(IEnumConnections* iface,
// 						LPENUMCONNECTIONS *ppEnum)
// {
//   EnumConnectionsImpl *This = impl_from_IEnumConnections(iface);
//   EnumConnectionsImpl *newObj;
//   TRACE("(%p)->(%p)\n", This, ppEnum);
//
//   newObj = EnumConnectionsImpl_Construct(This->pUnk, This->nConns, This->pCD);
//   newObj->nCur = This->nCur;
//   *ppEnum = &newObj->IEnumConnections_iface;
//   IUnknown_AddRef(This->pUnk);
//   return S_OK;
// }
//
// static const IEnumConnectionsVtbl EnumConnectionsImpl_VTable =
// {
//   EnumConnectionsImpl_QueryInterface,
//   EnumConnectionsImpl_AddRef,
//   EnumConnectionsImpl_Release,
//   EnumConnectionsImpl_Next,
//   EnumConnectionsImpl_Skip,
//   EnumConnectionsImpl_Reset,
//   EnumConnectionsImpl_Clone
// };
//
// /************************************************************************
//  *
//  *  The exported function to create the connection point.
//  *  NB not a windows API
//  *
//  * PARAMS
//  * pUnk [in] IUnknown of object to which the ConnectionPoint is associated.
//  *           Needed to access IConnectionPointContainer.
//  *
//  * riid [in] IID of sink interface that this ConnectionPoint manages
//  *
//  * pCP [out] returns IConnectionPoint
//  *
//  */
// HRESULT CreateConnectionPoint(IUnknown *pUnk, REFIID riid,
// 			      IConnectionPoint **pCP)
// {
//   ConnectionPointImpl *Obj;
//
//   TRACE("(%p %s %p)\n", pUnk, debugstr_guid(riid), pCP);
//
//   *pCP = NULL;
//   Obj = HeapAlloc(GetProcessHeap(), 0, sizeof(*Obj));
//   if (!Obj)
//     return E_OUTOFMEMORY;
//
//   Obj->IConnectionPoint_iface.lpVtbl = &ConnectionPointImpl_VTable;
//   Obj->Obj = pUnk;
//   Obj->ref = 1;
//   Obj->iid = *riid;
//   Obj->maxSinks = MAXSINKS;
//   Obj->sinks = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(IUnknown*) * MAXSINKS);
//   Obj->nSinks = 0;
//
//   *pCP = &Obj->IConnectionPoint_iface;
//   return S_OK;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
