//! marshal.rs — translation of dll/win32/ole32/marshal.c (Wine marshalling
//! library: proxy manager, standard marshaler, CoMarshalInterface family).
//!
//! Faithful 1:1 port: identical HRESULTs, identical refcount/interlocked
//! behavior, identical lock ordering, identical list manipulation. The
//! inter-thread path (CoMarshalInterThreadInterfaceInStream /
//! CoGetInterfaceAndReleaseStream) is live at desktop runtime.
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    unused_variables,
    unused_unsafe,
    unused_mut,
    unused_assignments,
    static_mut_refs,
    clippy::all
)]

use super::types::*;

/// number of refs given out for normal marshaling
const NORMALEXTREFS: ULONG = 5;

// (struct proxy_manager lives in types.rs — see the note there.)

#[inline]
unsafe fn impl_from_IMultiQI(iface: *mut IMultiQI) -> *mut proxy_manager {
    containing_record!(iface, proxy_manager, IMultiQI_iface)
}
#[inline]
unsafe fn impl_from_IMarshal(iface: *mut IMarshal) -> *mut proxy_manager {
    containing_record!(iface, proxy_manager, IMarshal_iface)
}
#[inline]
unsafe fn impl_from_IClientSecurity(iface: *mut IClientSecurity) -> *mut proxy_manager {
    containing_record!(iface, proxy_manager, IClientSecurity_iface)
}

/* Marshalling just passes a unique identifier to the remote client,
 * that makes it possible to find the passed interface again.
 *
 * OXIDs identify an apartment and are network scoped
 * OIDs identify a stub manager and are apartment scoped
 * IPIDs identify an interface stub and are apartment scoped
 */

unsafe fn get_facbuf_for_iid(riid: REFIID, facbuf: *mut *mut IPSFactoryBuffer) -> HRESULT {
    let mut clsid = GUID_NULL;
    let hr = CoGetPSClsid(riid, &mut clsid);
    if hr != S_OK {
        return hr;
    }
    CoGetClassObject(
        &clsid,
        CLSCTX_INPROC_SERVER | WINE_CLSCTX_DONT_HOST,
        core::ptr::null_mut(),
        &IID_IPSFactoryBuffer,
        facbuf as *mut *mut c_void,
    )
}

/// marshals an object into a STDOBJREF structure
///
/// (compobj_private.h: HRESULT marshal_object(...))
#[no_mangle]
pub unsafe extern "C" fn marshal_object(
    apt: *mut apartment,
    stdobjref: *mut STDOBJREF,
    riid: REFIID,
    object: *mut IUnknown,
    dest_context: DWORD,
    dest_context_data: *mut c_void,
    mshlflags: MSHLFLAGS,
) -> HRESULT {
    let mut hr: HRESULT;

    hr = apartment_getoxid(apt, &mut (*stdobjref).oxid);
    if hr != S_OK {
        return hr;
    }

    hr = apartment_createwindowifneeded(apt);
    if hr != S_OK {
        return hr;
    }

    let manager = get_stub_manager_from_object(apt, object, TRUE);
    if manager.is_null() {
        return E_OUTOFMEMORY;
    }

    (*stdobjref).flags = SORF_NULL;
    if mshlflags & MSHLFLAGS_TABLEWEAK != 0 {
        (*stdobjref).flags |= SORFP_TABLEWEAK;
    }
    if mshlflags & MSHLFLAGS_NOPING != 0 {
        (*stdobjref).flags |= SORF_NOPING;
    }
    (*stdobjref).oid = (*manager).oid;

    let tablemarshal =
        (mshlflags & MSHLFLAGS_TABLESTRONG != 0) || (mshlflags & MSHLFLAGS_TABLEWEAK != 0);

    /* make sure ifstub that we are creating is unique */
    let mut ifstub_p = stub_manager_find_ifstub(manager, riid, mshlflags);
    if ifstub_p.is_null() {
        let mut stub: *mut IRpcStubBuffer = core::ptr::null_mut();

        /* IUnknown doesn't require a stub buffer, because it never goes out on
         * the wire */
        if !IsEqualIID(riid, &IID_IUnknown) {
            let mut psfb: *mut IPSFactoryBuffer = core::ptr::null_mut();

            hr = get_facbuf_for_iid(riid, &mut psfb);
            if hr == S_OK {
                hr = psfb.CreateStub(riid, (*manager).object, &mut stub);
                psfb.Release();
                if hr != S_OK {
                    ERR!("Failed to create an IRpcStubBuffer from IPSFactory for %s with error 0x%08x\n");
                }
            } else {
                ERR!("couldn't get IPSFactory buffer for interface %s\n");
                hr = E_NOINTERFACE;
            }
        }

        if hr == S_OK {
            ifstub_p = stub_manager_new_ifstub(
                manager,
                stub,
                riid,
                dest_context,
                dest_context_data,
                mshlflags,
            );
            if ifstub_p.is_null() {
                hr = E_OUTOFMEMORY;
            }
        }
        if !stub.is_null() {
            stub.Release();
        }

        if hr != S_OK {
            stub_manager_int_release(manager);
            /* destroy the stub manager if it has no ifstubs by releasing
             * zero external references */
            stub_manager_ext_release(manager, 0, FALSE, TRUE);
            return hr;
        }
    }

    if !tablemarshal {
        (*stdobjref).cPublicRefs = NORMALEXTREFS;
        stub_manager_ext_addref(manager, (*stdobjref).cPublicRefs, FALSE);
    } else {
        (*stdobjref).cPublicRefs = 0;
        if mshlflags & MSHLFLAGS_TABLESTRONG != 0 {
            stub_manager_ext_addref(manager, 1, FALSE);
        } else {
            stub_manager_ext_addref(manager, 0, TRUE);
        }
    }

    /* FIXME: check return value */
    RPC_RegisterInterface(riid);

    (*stdobjref).ipid = (*ifstub_p).ipid;

    stub_manager_int_release(manager);
    S_OK
}

// ---------------------------------------------------------------------------
// Client-side identity of the server object (IMultiQI on proxy_manager)
// ---------------------------------------------------------------------------

unsafe extern "system" fn ClientIdentity_QueryInterface(
    iface: *mut IMultiQI,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let mut mqi = MULTI_QI {
        pIID: riid,
        pItf: core::ptr::null_mut(),
        hr: S_OK,
    };

    TRACE!("%s\n", debugstr_guid(riid));

    let hr = iface.QueryMultipleInterfaces(1, &mut mqi);
    *ppv = mqi.pItf as *mut c_void;

    hr
}

unsafe extern "system" fn ClientIdentity_AddRef(iface: *mut IMultiQI) -> ULONG {
    let This = impl_from_IMultiQI(iface);
    TRACE!("%p - before %d\n", iface, (*This).refs);
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).refs)) as ULONG
}

unsafe extern "system" fn ClientIdentity_Release(iface: *mut IMultiQI) -> ULONG {
    let This = impl_from_IMultiQI(iface);
    let refs = InterlockedDecrement(core::ptr::addr_of_mut!((*This).refs)) as ULONG;
    TRACE!("%p - after %d\n", iface, refs);
    if refs == 0 {
        proxy_manager_destroy(This);
    }
    refs
}

unsafe extern "system" fn ClientIdentity_QueryMultipleInterfaces(
    iface: *mut IMultiQI,
    cMQIs: ULONG,
    pMQIs: *mut MULTI_QI,
) -> HRESULT {
    let This = impl_from_IMultiQI(iface);
    let mut qiresults: *mut REMQIRESULT = core::ptr::null_mut();
    let mut nonlocal_mqis: ULONG = 0;
    let mut successful_mqis: ULONG = 0;
    // NOTE: the C allocates these unchecked and indexes straight into them;
    // faithfully preserved (allocation failure here would crash the C too).
    let iids =
        HeapAlloc(GetProcessHeap(), 0, cMQIs as usize * core::mem::size_of::<IID>()) as *mut IID;
    /* mapping of RemQueryInterface index to QueryMultipleInterfaces index */
    let mapping = HeapAlloc(GetProcessHeap(), 0, cMQIs as usize * core::mem::size_of::<ULONG>())
        as *mut ULONG;

    TRACE!("cMQIs: %d\n", cMQIs);

    /* try to get a local interface - this includes already active proxy
     * interfaces and also interfaces exposed by the proxy manager */
    for i in 0..cMQIs {
        let mqi = pMQIs.add(i as usize);
        TRACE!("iid[%d] = %s\n", i, debugstr_guid((*mqi).pIID));
        (*mqi).hr = proxy_manager_query_local_interface(
            This,
            (*mqi).pIID,
            core::ptr::addr_of_mut!((*mqi).pItf) as *mut *mut c_void,
        );
        if (*mqi).hr == S_OK {
            successful_mqis += 1;
        } else {
            *iids.add(nonlocal_mqis as usize) = *(*mqi).pIID;
            *mapping.add(nonlocal_mqis as usize) = i;
            nonlocal_mqis += 1;
        }
    }

    TRACE!("%d interfaces not found locally\n", nonlocal_mqis);

    /* if we have more than one interface not found locally then we must try
     * to query the remote object for it */
    if nonlocal_mqis != 0 {
        let mut remunk: *mut IRemUnknown = core::ptr::null_mut();
        let mut hr: HRESULT;

        /* get the ipid of the first entry */
        /* FIXME: should we implement ClientIdentity on the ifproxies instead
         * of the proxy_manager so we use the correct ipid here? */
        let first =
            containing_record!(list_head(core::ptr::addr_of!((*This).interfaces)), ifproxy, entry);
        let ipid: *mut IPID = core::ptr::addr_of_mut!((*first).stdobjref.ipid);

        /* get IRemUnknown proxy so we can communicate with the remote object */
        hr = proxy_manager_get_remunknown(This, &mut remunk);

        if SUCCEEDED(hr) {
            hr = remunk.RemQueryInterface(
                ipid,
                NORMALEXTREFS,
                nonlocal_mqis as USHORT,
                iids,
                &mut qiresults,
            );
            remunk.Release();
            if FAILED(hr) {
                ERR!("IRemUnknown_RemQueryInterface failed with error 0x%08x\n");
            }
        }

        /* IRemUnknown_RemQueryInterface can return S_FALSE if only some of
         * the interfaces were returned */
        if SUCCEEDED(hr) {
            let apt = apartment_get_current_or_mta();

            /* try to unmarshal each object returned to us */
            for i in 0..nonlocal_mqis {
                let index = *mapping.add(i as usize);
                let qi = qiresults.add(i as usize);
                let mqi = pMQIs.add(index as usize);
                let mut hrobj = (*qi).hResult;
                if hrobj == S_OK {
                    hrobj = unmarshal_object(
                        core::ptr::addr_of!((*qi).std),
                        apt,
                        (*This).dest_context,
                        (*This).dest_context_data,
                        (*mqi).pIID,
                        core::ptr::addr_of!((*This).oxid_info),
                        core::ptr::addr_of_mut!((*mqi).pItf) as *mut *mut c_void,
                    );
                }

                if hrobj == S_OK {
                    successful_mqis += 1;
                } else {
                    ERR!("Failed to get pointer to interface %s\n");
                }
                (*mqi).hr = hrobj;
            }

            apartment_release(apt);
        }

        /* free the memory allocated by the proxy */
        CoTaskMemFree(qiresults as PVOID);
    }

    TRACE!("%d/%d successfully queried\n", successful_mqis, cMQIs);

    HeapFree(GetProcessHeap(), 0, iids as PVOID);
    HeapFree(GetProcessHeap(), 0, mapping as PVOID);

    if successful_mqis == cMQIs {
        S_OK /* we got all requested interfaces */
    } else if successful_mqis == 0 {
        E_NOINTERFACE /* we didn't get any interfaces */
    } else {
        S_FALSE /* we got some interfaces */
    }
}

static ClientIdentity_Vtbl: IMultiQIVtbl = IMultiQIVtbl {
    QueryInterface: ClientIdentity_QueryInterface,
    AddRef: ClientIdentity_AddRef,
    Release: ClientIdentity_Release,
    QueryMultipleInterfaces: ClientIdentity_QueryMultipleInterfaces,
};

// ---------------------------------------------------------------------------
// IMarshal facet of proxy_manager
// ---------------------------------------------------------------------------

unsafe extern "system" fn Proxy_QueryInterface(
    iface: *mut IMarshal,
    riid: REFIID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);
    core::ptr::addr_of_mut!((*This).IMultiQI_iface).QueryInterface(riid, ppvObject)
}

unsafe extern "system" fn Proxy_AddRef(iface: *mut IMarshal) -> ULONG {
    let This = impl_from_IMarshal(iface);
    core::ptr::addr_of_mut!((*This).IMultiQI_iface).AddRef()
}

unsafe extern "system" fn Proxy_Release(iface: *mut IMarshal) -> ULONG {
    let This = impl_from_IMarshal(iface);
    core::ptr::addr_of_mut!((*This).IMultiQI_iface).Release()
}

unsafe extern "system" fn Proxy_GetUnmarshalClass(
    iface: *mut IMarshal,
    riid: REFIID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    pCid: *mut CLSID,
) -> HRESULT {
    *pCid = CLSID_StdMarshal;
    S_OK
}

unsafe extern "system" fn Proxy_GetMarshalSizeMax(
    iface: *mut IMarshal,
    riid: REFIID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    pSize: *mut DWORD,
) -> HRESULT {
    *pSize = OBJREF_STANDARD_SIZE as DWORD;
    S_OK
}

unsafe fn fill_std_objref(objref: *mut OBJREF, iid: *const GUID, std: *const STDOBJREF) {
    (*objref).signature = OBJREF_SIGNATURE;
    (*objref).flags = OBJREF_STANDARD;
    (*objref).iid = *iid;
    if !std.is_null() {
        (*objref).u_objref.u_standard.std = *std;
    }
    memset(
        core::ptr::addr_of_mut!((*objref).u_objref.u_standard.saResAddr) as PVOID,
        0,
        core::mem::size_of::<DUALSTRINGARRAY>(),
    );
}

unsafe extern "system" fn Proxy_MarshalInterface(
    iface: LPMARSHAL,
    pStm: *mut IStream,
    riid: REFIID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);
    let mut hr: HRESULT;
    let mut ifproxy_p: *mut ifproxy = core::ptr::null_mut();

    TRACE!("(...,%s,...)\n", debugstr_guid(riid));

    hr = proxy_manager_find_ifproxy(This, riid, &mut ifproxy_p);
    if SUCCEEDED(hr) {
        let mut stdobjref: STDOBJREF = (*ifproxy_p).stdobjref;

        stdobjref.cPublicRefs = 0;

        if (mshlflags != MSHLFLAGS_TABLEWEAK) && (mshlflags != MSHLFLAGS_TABLESTRONG) {
            let mut cPublicRefs: ULONG = (*ifproxy_p).refs;
            let mut cPublicRefsOld: ULONG;
            /* optimization - share out proxy's public references if possible
             * instead of making new proxy do a roundtrip through the server */
            loop {
                cPublicRefsOld = cPublicRefs;
                stdobjref.cPublicRefs = cPublicRefs / 2;
                let cPublicRefsNew = cPublicRefs - stdobjref.cPublicRefs;
                cPublicRefs = InterlockedCompareExchange(
                    core::ptr::addr_of_mut!((*ifproxy_p).refs) as *mut LONG,
                    cPublicRefsNew as LONG,
                    cPublicRefsOld as LONG,
                ) as ULONG;
                if cPublicRefs == cPublicRefsOld {
                    break;
                }
            }
        }

        /* normal and table-strong marshaling need at least one reference */
        if stdobjref.cPublicRefs == 0 && (mshlflags != MSHLFLAGS_TABLEWEAK) {
            let mut remunk: *mut IRemUnknown = core::ptr::null_mut();
            hr = proxy_manager_get_remunknown(This, &mut remunk);
            if hr == S_OK {
                let mut hrref: HRESULT = S_OK;
                let mut rif = REMINTERFACEREF {
                    ipid: (*ifproxy_p).stdobjref.ipid,
                    cPublicRefs: if mshlflags == MSHLFLAGS_TABLESTRONG {
                        1
                    } else {
                        NORMALEXTREFS
                    },
                    cPrivateRefs: 0,
                };
                hr = remunk.RemAddRef(1, &mut rif, &mut hrref);
                remunk.Release();
                if hr == S_OK && hrref == S_OK {
                    /* table-strong marshaling doesn't give the refs to the
                     * client that unmarshals the STDOBJREF */
                    if mshlflags != MSHLFLAGS_TABLESTRONG {
                        stdobjref.cPublicRefs = rif.cPublicRefs;
                    }
                } else {
                    ERR!("IRemUnknown_RemAddRef returned with 0x%08x, hrref = 0x%08x\n");
                }
            }
        }

        if SUCCEEDED(hr) {
            let mut objref: OBJREF = core::mem::zeroed();

            TRACE!("writing stdobjref: flags = %04x cPublicRefs = %d oxid = %s oid = %s ipid = %s\n");
            fill_std_objref(&mut objref, riid, &stdobjref);
            hr = pStm.Write(
                core::ptr::addr_of!(objref) as *const c_void,
                OBJREF_STANDARD_SIZE as ULONG,
                core::ptr::null_mut(),
            );
        }
    } else {
        /* we don't have the interface already unmarshaled so we have to
         * request the object from the server */
        let mut remunk: *mut IRemUnknown = core::ptr::null_mut();
        let mut qiresults: *mut REMQIRESULT = core::ptr::null_mut();
        let mut iid: IID = *riid;

        /* get the ipid of the first entry */
        /* FIXME: should we implement ClientIdentity on the ifproxies instead
         * of the proxy_manager so we use the correct ipid here? */
        let first =
            containing_record!(list_head(core::ptr::addr_of!((*This).interfaces)), ifproxy, entry);
        let ipid: *mut IPID = core::ptr::addr_of_mut!((*first).stdobjref.ipid);

        /* get IRemUnknown proxy so we can communicate with the remote object */
        hr = proxy_manager_get_remunknown(This, &mut remunk);

        if hr == S_OK {
            hr = remunk.RemQueryInterface(ipid, NORMALEXTREFS, 1, &mut iid, &mut qiresults);
            if SUCCEEDED(hr) {
                let mut objref: OBJREF = core::mem::zeroed();

                fill_std_objref(&mut objref, riid, core::ptr::addr_of!((*qiresults).std));
                hr = pStm.Write(
                    core::ptr::addr_of!(objref) as *const c_void,
                    OBJREF_STANDARD_SIZE as ULONG,
                    core::ptr::null_mut(),
                );
                if FAILED(hr) {
                    let mut rif = REMINTERFACEREF {
                        ipid: (*qiresults).std.ipid,
                        cPublicRefs: (*qiresults).std.cPublicRefs,
                        cPrivateRefs: 0,
                    };
                    remunk.RemRelease(1, &mut rif);
                }
                CoTaskMemFree(qiresults as PVOID);
            } else {
                ERR!("IRemUnknown_RemQueryInterface failed with error 0x%08x\n");
            }
            remunk.Release();
        }
    }

    hr
}

unsafe extern "system" fn Proxy_UnmarshalInterface(
    iface: *mut IMarshal,
    pStm: *mut IStream,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IMarshal(iface);
    let mut marshal: *mut IMarshal = core::ptr::null_mut();

    TRACE!("(%p, %p, %s, %p)\n", This, pStm, wine_dbgstr_guid(riid), ppv);

    let mut hr = StdMarshalImpl_Construct(
        &IID_IMarshal,
        (*This).dest_context,
        (*This).dest_context_data,
        core::ptr::addr_of_mut!(marshal) as *mut *mut c_void,
    );
    if FAILED(hr) {
        return hr;
    }

    hr = marshal.UnmarshalInterface(pStm, riid, ppv);
    marshal.Release();
    hr
}

unsafe extern "system" fn Proxy_ReleaseMarshalData(iface: *mut IMarshal, pStm: *mut IStream) -> HRESULT {
    let This = impl_from_IMarshal(iface);
    let mut marshal: *mut IMarshal = core::ptr::null_mut();

    TRACE!("(%p, %p)\n", This, pStm);

    let mut hr = StdMarshalImpl_Construct(
        &IID_IMarshal,
        (*This).dest_context,
        (*This).dest_context_data,
        core::ptr::addr_of_mut!(marshal) as *mut *mut c_void,
    );
    if FAILED(hr) {
        return hr;
    }

    hr = marshal.ReleaseMarshalData(pStm);
    marshal.Release();
    hr
}

unsafe extern "system" fn Proxy_DisconnectObject(iface: *mut IMarshal, dwReserved: DWORD) -> HRESULT {
    let This = impl_from_IMarshal(iface);
    let mut marshal: *mut IMarshal = core::ptr::null_mut();

    TRACE!("(%p, %x)\n", This, dwReserved);

    let mut hr = StdMarshalImpl_Construct(
        &IID_IMarshal,
        (*This).dest_context,
        (*This).dest_context_data,
        core::ptr::addr_of_mut!(marshal) as *mut *mut c_void,
    );
    if FAILED(hr) {
        return hr;
    }

    hr = marshal.DisconnectObject(dwReserved);
    marshal.Release();
    hr
}

static ProxyMarshal_Vtbl: IMarshalVtbl = IMarshalVtbl {
    QueryInterface: Proxy_QueryInterface,
    AddRef: Proxy_AddRef,
    Release: Proxy_Release,
    GetUnmarshalClass: Proxy_GetUnmarshalClass,
    GetMarshalSizeMax: Proxy_GetMarshalSizeMax,
    MarshalInterface: Proxy_MarshalInterface,
    UnmarshalInterface: Proxy_UnmarshalInterface,
    ReleaseMarshalData: Proxy_ReleaseMarshalData,
    DisconnectObject: Proxy_DisconnectObject,
};

// ---------------------------------------------------------------------------
// IClientSecurity facet of proxy_manager
// ---------------------------------------------------------------------------

unsafe extern "system" fn ProxyCliSec_QueryInterface(
    iface: *mut IClientSecurity,
    riid: REFIID,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_IClientSecurity(iface);
    core::ptr::addr_of_mut!((*This).IMultiQI_iface).QueryInterface(riid, ppvObject)
}

unsafe extern "system" fn ProxyCliSec_AddRef(iface: *mut IClientSecurity) -> ULONG {
    let This = impl_from_IClientSecurity(iface);
    core::ptr::addr_of_mut!((*This).IMultiQI_iface).AddRef()
}

unsafe extern "system" fn ProxyCliSec_Release(iface: *mut IClientSecurity) -> ULONG {
    let This = impl_from_IClientSecurity(iface);
    core::ptr::addr_of_mut!((*This).IMultiQI_iface).Release()
}

unsafe extern "system" fn ProxyCliSec_QueryBlanket(
    iface: *mut IClientSecurity,
    pProxy: *mut IUnknown,
    pAuthnSvc: *mut DWORD,
    pAuthzSvc: *mut DWORD,
    ppServerPrincName: *mut *mut OLECHAR,
    pAuthnLevel: *mut DWORD,
    pImpLevel: *mut DWORD,
    pAuthInfo: *mut *mut c_void,
    pCapabilities: *mut DWORD,
) -> HRESULT {
    FIXME!("(%p, %p, %p, %p, %p, %p, %p, %p): stub\n");

    if !pAuthnSvc.is_null() {
        *pAuthnSvc = 0;
    }
    if !pAuthzSvc.is_null() {
        *pAuthzSvc = 0;
    }
    if !ppServerPrincName.is_null() {
        *ppServerPrincName = core::ptr::null_mut();
    }
    if !pAuthnLevel.is_null() {
        *pAuthnLevel = RPC_C_AUTHN_LEVEL_DEFAULT;
    }
    if !pImpLevel.is_null() {
        *pImpLevel = RPC_C_IMP_LEVEL_DEFAULT;
    }
    if !pAuthInfo.is_null() {
        *pAuthInfo = core::ptr::null_mut();
    }
    if !pCapabilities.is_null() {
        *pCapabilities = EOAC_NONE;
    }

    E_NOTIMPL
}

unsafe extern "system" fn ProxyCliSec_SetBlanket(
    iface: *mut IClientSecurity,
    pProxy: *mut IUnknown,
    AuthnSvc: DWORD,
    AuthzSvc: DWORD,
    pServerPrincName: *mut OLECHAR,
    AuthnLevel: DWORD,
    ImpLevel: DWORD,
    pAuthInfo: *mut c_void,
    Capabilities: DWORD,
) -> HRESULT {
    FIXME!("(%p, %d, %d, %s, %d, %d, %p, 0x%x): stub\n");
    E_NOTIMPL
}

unsafe extern "system" fn ProxyCliSec_CopyProxy(
    iface: *mut IClientSecurity,
    pProxy: *mut IUnknown,
    ppCopy: *mut *mut IUnknown,
) -> HRESULT {
    FIXME!("(%p, %p): stub\n");
    *ppCopy = core::ptr::null_mut();
    E_NOTIMPL
}

static ProxyCliSec_Vtbl: IClientSecurityVtbl = IClientSecurityVtbl {
    QueryInterface: ProxyCliSec_QueryInterface,
    AddRef: ProxyCliSec_AddRef,
    Release: ProxyCliSec_Release,
    QueryBlanket: ProxyCliSec_QueryBlanket,
    SetBlanket: ProxyCliSec_SetBlanket,
    CopyProxy: ProxyCliSec_CopyProxy,
};

// ---------------------------------------------------------------------------
// ifproxy helpers
// ---------------------------------------------------------------------------

unsafe fn ifproxy_get_public_ref(This: *mut ifproxy) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if WAIT_OBJECT_0 != WaitForSingleObject((*(*This).parent).remoting_mutex, INFINITE) {
        ERR!("Wait failed for ifproxy %p\n");
        return E_UNEXPECTED;
    }

    if (*This).refs == 0 {
        let mut remunk: *mut IRemUnknown = core::ptr::null_mut();

        TRACE!("getting public ref for ifproxy %p\n", This);

        hr = proxy_manager_get_remunknown((*This).parent, &mut remunk);
        if hr == S_OK {
            let mut hrref: HRESULT = S_OK;
            let mut rif = REMINTERFACEREF {
                ipid: (*This).stdobjref.ipid,
                cPublicRefs: NORMALEXTREFS,
                cPrivateRefs: 0,
            };
            hr = remunk.RemAddRef(1, &mut rif, &mut hrref);
            remunk.Release();
            if hr == S_OK && hrref == S_OK {
                InterlockedExchangeAdd(
                    core::ptr::addr_of_mut!((*This).refs) as *mut LONG,
                    NORMALEXTREFS as LONG,
                );
            } else {
                ERR!("IRemUnknown_RemAddRef returned with 0x%08x, hrref = 0x%08x\n");
            }
        }
    }
    ReleaseMutex((*(*This).parent).remoting_mutex);

    hr
}

unsafe fn ifproxy_release_public_refs(This: *mut ifproxy) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    if WAIT_OBJECT_0 != WaitForSingleObject((*(*This).parent).remoting_mutex, INFINITE) {
        ERR!("Wait failed for ifproxy %p\n");
        return E_UNEXPECTED;
    }

    let public_refs: LONG = (*This).refs as LONG;
    if public_refs > 0 {
        let mut remunk: *mut IRemUnknown = core::ptr::null_mut();

        TRACE!("releasing %d refs\n", public_refs);

        hr = proxy_manager_get_remunknown((*This).parent, &mut remunk);
        if hr == S_OK {
            let mut rif = REMINTERFACEREF {
                ipid: (*This).stdobjref.ipid,
                cPublicRefs: public_refs as ULONG,
                cPrivateRefs: 0,
            };
            hr = remunk.RemRelease(1, &mut rif);
            remunk.Release();
            if hr == S_OK {
                InterlockedExchangeAdd(
                    core::ptr::addr_of_mut!((*This).refs) as *mut LONG,
                    -public_refs,
                );
            } else if hr == RPC_E_DISCONNECTED {
                WARN!(
                    "couldn't release references because object was disconnected: oxid = %s, oid = %s\n"
                );
            } else {
                ERR!("IRemUnknown_RemRelease failed with error 0x%08x\n");
            }
        }
    }
    ReleaseMutex((*(*This).parent).remoting_mutex);

    hr
}

/// should be called inside This->parent->cs critical section
unsafe fn ifproxy_disconnect(This: *mut ifproxy) {
    ifproxy_release_public_refs(This);
    if !(*This).proxy.is_null() {
        (*This).proxy.Disconnect();
    }

    (*This).chan.Release();
    (*This).chan = core::ptr::null_mut();
}

/// should be called in This->parent->cs critical section if it is an entry in
/// parent's list
unsafe fn ifproxy_destroy(This: *mut ifproxy) {
    TRACE!("%p\n", This);

    /* release public references to this object so that the stub can know
     * when to destroy itself */
    ifproxy_release_public_refs(This);

    list_remove(core::ptr::addr_of_mut!((*This).entry));

    if !(*This).chan.is_null() {
        (*This).chan.Release();
        (*This).chan = core::ptr::null_mut();
    }

    if !(*This).proxy.is_null() {
        (*This).proxy.Release();
    }

    HeapFree(GetProcessHeap(), 0, This as PVOID);
}

// ---------------------------------------------------------------------------
// proxy_manager
// ---------------------------------------------------------------------------

unsafe fn proxy_manager_construct(
    apt: *mut apartment,
    sorflags: ULONG,
    oxid: OXID,
    oid: OID,
    oxid_info: *const OXID_INFO,
    proxy_manager_out: *mut *mut proxy_manager,
) -> HRESULT {
    let This =
        HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<proxy_manager>()) as *mut proxy_manager;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).remoting_mutex = CreateMutexW(core::ptr::null_mut(), FALSE, core::ptr::null());
    if (*This).remoting_mutex.is_null() {
        HeapFree(GetProcessHeap(), 0, This as PVOID);
        return HRESULT_FROM_WIN32(GetLastError());
    }

    if !oxid_info.is_null() {
        (*This).oxid_info.dwPid = (*oxid_info).dwPid;
        (*This).oxid_info.dwTid = (*oxid_info).dwTid;
        (*This).oxid_info.ipidRemUnknown = (*oxid_info).ipidRemUnknown;
        (*This).oxid_info.dwAuthnHint = (*oxid_info).dwAuthnHint;
        (*This).oxid_info.psa = core::ptr::null_mut() /* FIXME: copy from oxid_info */;
    } else {
        let hr = RPC_ResolveOxid(oxid, core::ptr::addr_of_mut!((*This).oxid_info));
        if FAILED(hr) {
            CloseHandle((*This).remoting_mutex);
            HeapFree(GetProcessHeap(), 0, This as PVOID);
            return hr;
        }
    }

    (*This).IMultiQI_iface.lpVtbl = &ClientIdentity_Vtbl;
    (*This).IMarshal_iface.lpVtbl = &ProxyMarshal_Vtbl;
    (*This).IClientSecurity_iface.lpVtbl = &ProxyCliSec_Vtbl;

    list_init(core::ptr::addr_of_mut!((*This).entry));
    list_init(core::ptr::addr_of_mut!((*This).interfaces));

    InitializeCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    DEBUG_SET_CRITSEC_NAME(core::ptr::addr_of_mut!((*This).cs), "proxy_manager");

    /* the apartment the object was unmarshaled into */
    (*This).parent = apt;

    /* the source apartment and id of the object */
    (*This).oxid = oxid;
    (*This).oid = oid;

    (*This).refs = 1;

    /* the DCOM draft specification states that the SORF_NOPING flag is
     * proxy manager specific, not ifproxy specific, so this implies that we
     * should store the STDOBJREF flags here in the proxy manager. */
    (*This).sorflags = sorflags;

    /* we create the IRemUnknown proxy on demand */
    (*This).remunk = core::ptr::null_mut();

    /* initialise these values to the weakest values and they will be
     * overwritten in proxy_manager_set_context */
    (*This).dest_context = MSHCTX_INPROC;
    (*This).dest_context_data = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!((*apt).cs));
    /* FIXME: we are dependent on the ordering in here to make sure a proxy's
     * IRemUnknown proxy doesn't get destroyed before the regular proxy does
     * because we need the IRemUnknown proxy during the destruction of the
     * regular proxy. Ideally, we should maintain a separate list for the
     * IRemUnknown proxies that need late destruction */
    list_add_tail(
        core::ptr::addr_of_mut!((*apt).proxies),
        core::ptr::addr_of_mut!((*This).entry),
    );
    LeaveCriticalSection(core::ptr::addr_of_mut!((*apt).cs));

    TRACE!("%p created for OXID %s, OID %s\n");

    *proxy_manager_out = This;
    S_OK
}

#[inline]
unsafe fn proxy_manager_set_context(
    This: *mut proxy_manager,
    dest_context: MSHCTX,
    dest_context_data: *mut c_void,
) {
    let mut old_dest_context: MSHCTX;
    let mut new_dest_context: MSHCTX;

    loop {
        old_dest_context = (*This).dest_context;
        new_dest_context = old_dest_context;
        /* "stronger" values overwrite "weaker" values. stronger values are
         * ones that disable more optimisations */
        match old_dest_context {
            MSHCTX_INPROC => {
                new_dest_context = dest_context;
            }
            MSHCTX_CROSSCTX => match dest_context {
                MSHCTX_INPROC => {}
                _ => new_dest_context = dest_context,
            },
            MSHCTX_LOCAL => match dest_context {
                MSHCTX_INPROC | MSHCTX_CROSSCTX => {}
                _ => new_dest_context = dest_context,
            },
            MSHCTX_NOSHAREDMEM => match dest_context {
                MSHCTX_DIFFERENTMACHINE => new_dest_context = dest_context,
                _ => {}
            },
            _ => {}
        }

        if old_dest_context == new_dest_context {
            break;
        }

        new_dest_context = InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*This).dest_context) as *mut LONG,
            new_dest_context as LONG,
            old_dest_context as LONG,
        ) as MSHCTX;
        if new_dest_context == old_dest_context {
            break;
        }
    }

    if !dest_context_data.is_null() {
        InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*This).dest_context_data),
            dest_context_data,
        );
    }
}

unsafe fn proxy_manager_query_local_interface(
    This: *mut proxy_manager,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    TRACE!("%s\n", debugstr_guid(riid));

    if IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IMultiQI) {
        *ppv = core::ptr::addr_of_mut!((*This).IMultiQI_iface) as *mut c_void;
        core::ptr::addr_of_mut!((*This).IMultiQI_iface).AddRef();
        return S_OK;
    }
    if IsEqualIID(riid, &IID_IMarshal) {
        *ppv = core::ptr::addr_of_mut!((*This).IMarshal_iface) as *mut c_void;
        core::ptr::addr_of_mut!((*This).IMarshal_iface).AddRef();
        return S_OK;
    }
    if IsEqualIID(riid, &IID_IClientSecurity) {
        *ppv = core::ptr::addr_of_mut!((*This).IClientSecurity_iface) as *mut c_void;
        core::ptr::addr_of_mut!((*This).IClientSecurity_iface).AddRef();
        return S_OK;
    }

    let mut ifproxy_p: *mut ifproxy = core::ptr::null_mut();
    let hr = proxy_manager_find_ifproxy(This, riid, &mut ifproxy_p);
    if hr == S_OK {
        *ppv = (*ifproxy_p).iface;
        ((*ppv) as *mut IUnknown).AddRef();
        return S_OK;
    }

    *ppv = core::ptr::null_mut();
    E_NOINTERFACE
}

unsafe fn proxy_manager_create_ifproxy(
    This: *mut proxy_manager,
    stdobjref: *const STDOBJREF,
    riid: REFIID,
    channel: *mut IRpcChannelBuffer,
    iif_out: *mut *mut ifproxy,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut psfb: *mut IPSFactoryBuffer = core::ptr::null_mut();
    let ifproxy_p = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<ifproxy>()) as *mut ifproxy;
    if ifproxy_p.is_null() {
        return E_OUTOFMEMORY;
    }

    list_init(core::ptr::addr_of_mut!((*ifproxy_p).entry));

    (*ifproxy_p).parent = This;
    (*ifproxy_p).stdobjref = *stdobjref;
    (*ifproxy_p).iid = *riid;
    (*ifproxy_p).refs = 0;
    (*ifproxy_p).proxy = core::ptr::null_mut();

    c_assert!(!channel.is_null());
    (*ifproxy_p).chan = channel; /* FIXME: we should take the binding strings and construct the channel in this function */

    /* the IUnknown interface is special because it does not have a
     * proxy associated with the ifproxy as we handle IUnknown ourselves */
    if IsEqualIID(riid, &IID_IUnknown) {
        (*ifproxy_p).iface = core::ptr::addr_of_mut!((*This).IMultiQI_iface) as *mut c_void;
        core::ptr::addr_of_mut!((*This).IMultiQI_iface).AddRef();
        hr = S_OK;
    } else {
        hr = get_facbuf_for_iid(riid, &mut psfb);
        if hr == S_OK {
            /* important note: the outer unknown is set to the proxy manager.
             * This ensures the COM identity rules are not violated, by having a
             * one-to-one mapping of objects on the proxy side to objects on the
             * stub side, no matter which interface you view the object through */
            hr = psfb.CreateProxy(
                core::ptr::addr_of_mut!((*This).IMultiQI_iface) as *mut IUnknown,
                riid,
                core::ptr::addr_of_mut!((*ifproxy_p).proxy),
                core::ptr::addr_of_mut!((*ifproxy_p).iface),
            );
            psfb.Release();
            if hr != S_OK {
                ERR!("Could not create proxy for interface %s, error 0x%08x\n");
            }
        } else {
            ERR!("Could not get IPSFactoryBuffer for interface %s, error 0x%08x\n");
        }

        if hr == S_OK {
            hr = (*ifproxy_p).proxy.Connect((*ifproxy_p).chan);
        }
    }

    if hr == S_OK {
        EnterCriticalSection(core::ptr::addr_of_mut!((*This).cs));
        list_add_tail(
            core::ptr::addr_of_mut!((*This).interfaces),
            core::ptr::addr_of_mut!((*ifproxy_p).entry),
        );
        LeaveCriticalSection(core::ptr::addr_of_mut!((*This).cs));

        *iif_out = ifproxy_p;
        TRACE!("ifproxy %p created for IPID %s, interface %s with %u public refs\n");
    } else {
        ifproxy_destroy(ifproxy_p);
    }

    hr
}

unsafe fn proxy_manager_find_ifproxy(
    This: *mut proxy_manager,
    riid: REFIID,
    ifproxy_found: *mut *mut ifproxy,
) -> HRESULT {
    let mut hr: HRESULT = E_NOINTERFACE; /* assume not found */

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    let head = core::ptr::addr_of_mut!((*This).interfaces);
    let mut cursor = (*head).next;
    while cursor != head {
        let ifproxy_p = containing_record!(cursor, ifproxy, entry);
        if IsEqualIID(riid, core::ptr::addr_of!((*ifproxy_p).iid)) {
            *ifproxy_found = ifproxy_p;
            hr = S_OK;
            break;
        }
        cursor = (*cursor).next;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).cs));

    hr
}

unsafe fn proxy_manager_disconnect(This: *mut proxy_manager) {
    TRACE!("oxid = %s, oid = %s\n");

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).cs));

    /* SORFP_NOLIFTIMEMGMT proxies (for IRemUnknown) shouldn't be
     * disconnected - it won't do anything anyway, except cause
     * problems for other objects that depend on this proxy always
     * working */
    if (*This).sorflags & SORFP_NOLIFETIMEMGMT == 0 {
        let head = core::ptr::addr_of_mut!((*This).interfaces);
        let mut cursor = (*head).next;
        while cursor != head {
            let ifproxy_p = containing_record!(cursor, ifproxy, entry);
            ifproxy_disconnect(ifproxy_p);
            cursor = (*cursor).next;
        }
    }

    /* apartment is being destroyed so don't keep a pointer around to it */
    (*This).parent = core::ptr::null_mut();

    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).cs));
}

unsafe fn proxy_manager_get_remunknown(
    This: *mut proxy_manager,
    remunk: *mut *mut IRemUnknown,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    /* we don't want to try and unmarshal or use IRemUnknown if we don't want
     * lifetime management */
    if (*This).sorflags & SORFP_NOLIFETIMEMGMT != 0 {
        return S_FALSE;
    }

    let apt = apartment_get_current_or_mta();
    if apt.is_null() {
        return CO_E_NOTINITIALIZED;
    }

    let called_in_original_apt = !(*This).parent.is_null() && ((*(*This).parent).oxid == (*apt).oxid);

    EnterCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    /* only return the cached object if called from the original apartment.
     * in future, we might want to make the IRemUnknown proxy callable from any
     * apartment to avoid these checks */
    if !(*This).remunk.is_null() && called_in_original_apt {
        /* already created - return existing object */
        *remunk = (*This).remunk;
        (*remunk).AddRef();
    } else if (*This).parent.is_null() {
        /* disconnected - we can't create IRemUnknown */
        *remunk = core::ptr::null_mut();
        hr = S_FALSE;
    } else {
        let mut stdobjref: STDOBJREF = core::mem::zeroed();
        /* Don't want IRemUnknown lifetime management as this is IRemUnknown!
         * We also don't care about whether or not the stub is still alive */
        stdobjref.flags = SORFP_NOLIFETIMEMGMT | SORF_NOPING;
        stdobjref.cPublicRefs = 1;
        /* oxid of destination object */
        stdobjref.oxid = (*This).oxid;
        /* FIXME: what should be used for the oid? The DCOM draft doesn't say */
        stdobjref.oid = !0u64; /* (OID)-1 */
        stdobjref.ipid = (*This).oxid_info.ipidRemUnknown;

        /* do the unmarshal */
        hr = unmarshal_object(
            &stdobjref,
            apt,
            (*This).dest_context,
            (*This).dest_context_data,
            &IID_IRemUnknown,
            core::ptr::addr_of!((*This).oxid_info),
            remunk as *mut *mut c_void,
        );
        if hr == S_OK && called_in_original_apt {
            (*This).remunk = *remunk;
            (*This).remunk.AddRef();
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*This).cs));
    apartment_release(apt);

    TRACE!("got IRemUnknown* pointer %p, hr = 0x%08x\n");

    hr
}

/// destroys a proxy manager, freeing the memory it used.
/// Note: this function should not be called from a list iteration in the
/// apartment, due to the fact that it removes itself from the apartment and
/// it could add a proxy to IRemUnknown into the apartment.
unsafe fn proxy_manager_destroy(This: *mut proxy_manager) {
    TRACE!("oxid = %s, oid = %s\n");

    if !(*This).parent.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!((*(*This).parent).cs));

        /* remove ourself from the list of proxy objects in the apartment */
        let head = core::ptr::addr_of_mut!((*(*This).parent).proxies);
        let mut cursor = (*head).next;
        while cursor != head {
            if cursor == core::ptr::addr_of_mut!((*This).entry) {
                list_remove(core::ptr::addr_of_mut!((*This).entry));
                break;
            }
            cursor = (*cursor).next;
        }

        LeaveCriticalSection(core::ptr::addr_of_mut!((*(*This).parent).cs));
    }

    /* destroy all of the interface proxies */
    loop {
        let cursor = list_head(core::ptr::addr_of!((*This).interfaces));
        if cursor.is_null() {
            break;
        }
        let ifproxy_p = containing_record!(cursor, ifproxy, entry);
        ifproxy_destroy(ifproxy_p);
    }

    if !(*This).remunk.is_null() {
        (*This).remunk.Release();
    }
    CoTaskMemFree((*This).oxid_info.psa as PVOID);

    DEBUG_CLEAR_CRITSEC_NAME(core::ptr::addr_of_mut!((*This).cs));
    DeleteCriticalSection(core::ptr::addr_of_mut!((*This).cs));

    CloseHandle((*This).remoting_mutex);

    HeapFree(GetProcessHeap(), 0, This as PVOID);
}

/// finds the proxy manager corresponding to a given OXID and OID that has
/// been unmarshaled in the specified apartment. The caller must release the
/// reference to the proxy_manager when the object is no longer used.
unsafe fn find_proxy_manager(
    apt: *mut apartment,
    oxid: OXID,
    oid: OID,
    proxy_found: *mut *mut proxy_manager,
) -> BOOL {
    let mut found: BOOL = FALSE;

    EnterCriticalSection(core::ptr::addr_of_mut!((*apt).cs));
    let head = core::ptr::addr_of_mut!((*apt).proxies);
    let mut cursor = (*head).next;
    while cursor != head {
        let proxy = containing_record!(cursor, proxy_manager, entry);
        if (oxid == (*proxy).oxid) && (oid == (*proxy).oid) {
            /* be careful of a race with ClientIdentity_Release, which would
             * cause us to return a proxy which is in the process of being
             * destroyed */
            if core::ptr::addr_of_mut!((*proxy).IMultiQI_iface).AddRef() != 0 {
                *proxy_found = proxy;
                found = TRUE;
                break;
            }
        }
        cursor = (*cursor).next;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*apt).cs));
    found
}

/// (compobj_private.h: HRESULT apartment_disconnectproxies(struct apartment*))
#[no_mangle]
pub unsafe extern "C" fn apartment_disconnectproxies(apt: *mut apartment) -> HRESULT {
    let head = core::ptr::addr_of_mut!((*apt).proxies);
    let mut cursor = (*head).next;
    while cursor != head {
        let proxy = containing_record!(cursor, proxy_manager, entry);
        proxy_manager_disconnect(proxy);
        cursor = (*cursor).next;
    }

    S_OK
}

// ---------------------------------------------------------------------------
// StdMarshal implementation
// ---------------------------------------------------------------------------
#[repr(C)]
struct StdMarshalImpl {
    IMarshal_iface: IMarshal,
    ref_: LONG,
    dest_context: DWORD,
    dest_context_data: *mut c_void,
}

#[inline]
unsafe fn impl_from_StdMarshal(iface: *mut IMarshal) -> *mut StdMarshalImpl {
    containing_record!(iface, StdMarshalImpl, IMarshal_iface)
}

unsafe extern "system" fn StdMarshalImpl_QueryInterface(
    iface: *mut IMarshal,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    *ppv = core::ptr::null_mut();
    if IsEqualIID(&IID_IUnknown, riid) || IsEqualIID(&IID_IMarshal, riid) {
        *ppv = iface as *mut c_void;
        iface.AddRef();
        return S_OK;
    }
    FIXME!("No interface for %s.\n");
    E_NOINTERFACE
}

unsafe extern "system" fn StdMarshalImpl_AddRef(iface: *mut IMarshal) -> ULONG {
    let This = impl_from_StdMarshal(iface);
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG
}

unsafe extern "system" fn StdMarshalImpl_Release(iface: *mut IMarshal) -> ULONG {
    let This = impl_from_StdMarshal(iface);
    let ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref_)) as ULONG;

    if ref_ == 0 {
        HeapFree(GetProcessHeap(), 0, This as PVOID);
    }
    ref_
}

unsafe extern "system" fn StdMarshalImpl_GetUnmarshalClass(
    iface: *mut IMarshal,
    riid: REFIID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    pCid: *mut CLSID,
) -> HRESULT {
    *pCid = CLSID_StdMarshal;
    S_OK
}

unsafe extern "system" fn StdMarshalImpl_GetMarshalSizeMax(
    iface: *mut IMarshal,
    riid: REFIID,
    pv: *mut c_void,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlflags: DWORD,
    pSize: *mut DWORD,
) -> HRESULT {
    *pSize = OBJREF_STANDARD_SIZE as DWORD;
    S_OK
}

unsafe extern "system" fn StdMarshalImpl_MarshalInterface(
    iface: *mut IMarshal,
    pStm: *mut IStream,
    riid: REFIID,
    pv: *mut c_void,
    dest_context: DWORD,
    dest_context_data: *mut c_void,
    mshlflags: DWORD,
) -> HRESULT {
    let mut res: ULONG = 0;
    let mut objref: OBJREF = core::mem::zeroed();

    TRACE!("(...,%s,...)\n", debugstr_guid(riid));

    let apt = apartment_get_current_or_mta();
    if apt.is_null() {
        ERR!("Apartment not initialized\n");
        return CO_E_NOTINITIALIZED;
    }

    /* make sure this apartment can be reached from other threads / processes */
    RPC_StartRemoting(apt);

    fill_std_objref(&mut objref, riid, core::ptr::null());
    let hres = marshal_object(
        apt,
        core::ptr::addr_of_mut!(objref.u_objref.u_standard.std),
        riid,
        pv as *mut IUnknown,
        dest_context,
        dest_context_data,
        mshlflags,
    );
    apartment_release(apt);
    if hres != S_OK {
        ERR!("Failed to create ifstub, hres=0x%x\n");
        return hres;
    }

    pStm.Write(
        core::ptr::addr_of!(objref) as *const c_void,
        OBJREF_STANDARD_SIZE as ULONG,
        &mut res,
    )
}

/// helper for StdMarshalImpl_UnmarshalInterface - does the unmarshaling with
/// no questions asked about the rules surrounding same-apartment unmarshals
/// and table marshaling
unsafe fn unmarshal_object(
    stdobjref: *const STDOBJREF,
    apt: *mut apartment,
    dest_context: MSHCTX,
    dest_context_data: *mut c_void,
    riid: REFIID,
    oxid_info: *const OXID_INFO,
    object: *mut *mut c_void,
) -> HRESULT {
    let mut proxy_manager_p: *mut proxy_manager = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    c_assert!(!apt.is_null());

    TRACE!("stdobjref: flags = %04x cPublicRefs = %d oxid = %s oid = %s ipid = %s\n");

    /* create a new proxy manager if one doesn't already exist for the
     * object */
    if find_proxy_manager(apt, (*stdobjref).oxid, (*stdobjref).oid, &mut proxy_manager_p) == FALSE {
        hr = proxy_manager_construct(
            apt,
            (*stdobjref).flags,
            (*stdobjref).oxid,
            (*stdobjref).oid,
            oxid_info,
            &mut proxy_manager_p,
        );
    } else {
        TRACE!("proxy manager already created, using\n");
    }

    if hr == S_OK {
        let mut ifproxy_p: *mut ifproxy = core::ptr::null_mut();

        proxy_manager_set_context(proxy_manager_p, dest_context, dest_context_data);

        hr = proxy_manager_find_ifproxy(proxy_manager_p, riid, &mut ifproxy_p);
        if hr == E_NOINTERFACE {
            let mut chanbuf: *mut IRpcChannelBuffer = core::ptr::null_mut();
            hr = RPC_CreateClientChannel(
                core::ptr::addr_of!((*stdobjref).oxid),
                core::ptr::addr_of!((*stdobjref).ipid),
                core::ptr::addr_of!((*proxy_manager_p).oxid_info),
                riid,
                (*proxy_manager_p).dest_context,
                (*proxy_manager_p).dest_context_data,
                &mut chanbuf,
                apt,
            );
            if hr == S_OK {
                hr = proxy_manager_create_ifproxy(
                    proxy_manager_p,
                    stdobjref,
                    riid,
                    chanbuf,
                    &mut ifproxy_p,
                );
            }
        } else {
            ((*ifproxy_p).iface as *mut IUnknown).AddRef();
        }

        if hr == S_OK {
            InterlockedExchangeAdd(
                core::ptr::addr_of_mut!((*ifproxy_p).refs) as *mut LONG,
                (*stdobjref).cPublicRefs as LONG,
            );
            /* get at least one external reference to the object to keep it alive */
            hr = ifproxy_get_public_ref(ifproxy_p);
            if FAILED(hr) {
                ifproxy_destroy(ifproxy_p);
            }
        }

        if hr == S_OK {
            *object = (*ifproxy_p).iface;
        }
    }

    /* release our reference to the proxy manager - the client/apartment
     * will hold on to the remaining reference for us */
    if !proxy_manager_p.is_null() {
        core::ptr::addr_of_mut!((*proxy_manager_p).IMultiQI_iface).Release();
    }

    hr
}

unsafe fn std_unmarshal_interface(
    mut dest_context: MSHCTX,
    mut dest_context_data: *mut c_void,
    pStm: *mut IStream,
    riid: REFIID,
    ppv: *mut *mut c_void,
    dest_context_known: BOOL,
) -> HRESULT {
    let mut stubmgr: *mut stub_manager = core::ptr::null_mut();
    let mut obj: OR_STANDARD = core::mem::zeroed();
    let mut res: ULONG = 0;
    let mut hres: HRESULT;
    let mut oxid: OXID = 0;

    TRACE!("(...,%s,....)\n", debugstr_guid(riid));

    /* we need an apartment to unmarshal into */
    let apt = apartment_get_current_or_mta();
    if apt.is_null() {
        ERR!("Apartment not initialized\n");
        return CO_E_NOTINITIALIZED;
    }

    /* read STDOBJREF from wire */
    hres = pStm.Read(
        core::ptr::addr_of_mut!(obj) as *mut c_void,
        OR_STANDARD_SIZE as ULONG,
        &mut res,
    );
    if hres != S_OK {
        apartment_release(apt);
        return STG_E_READFAULT;
    }

    hres = apartment_getoxid(apt, &mut oxid);
    if hres != S_OK {
        apartment_release(apt);
        return hres;
    }

    if obj.saResAddr.wNumEntries != 0 {
        // NOTE: the C leaks the apartment reference on this path; preserved.
        ERR!("unsupported size of DUALSTRINGARRAY\n");
        return E_NOTIMPL;
    }

    /* check if we're marshalling back to ourselves */
    if oxid == obj.std.oxid {
        stubmgr = get_stub_manager(apt, obj.std.oid);
        if !stubmgr.is_null() {
            TRACE!("Unmarshalling object marshalled in same apartment for iid %s, returning original object %p\n");

            hres = (*stubmgr).object.QueryInterface(riid, ppv);

            /* unref the ifstub. FIXME: only do this on success? */
            if stub_manager_is_table_marshaled(stubmgr, core::ptr::addr_of!(obj.std.ipid)) == FALSE
            {
                stub_manager_ext_release(
                    stubmgr,
                    obj.std.cPublicRefs,
                    (obj.std.flags & SORFP_TABLEWEAK) as BOOL,
                    FALSE,
                );
            }

            stub_manager_int_release(stubmgr);
            apartment_release(apt);
            return hres;
        }
    }

    /* notify stub manager about unmarshal if process-local object.
     * note: if the oxid is not found then we and native will quite happily
     * ignore table marshaling and normal marshaling rules regarding number of
     * unmarshals, etc, but if you abuse these rules then your proxy could end
     * up returning RPC_E_DISCONNECTED. */
    let stub_apt = apartment_findfromoxid(obj.std.oxid, TRUE);
    if !stub_apt.is_null() {
        stubmgr = get_stub_manager(stub_apt, obj.std.oid);
        if !stubmgr.is_null() {
            if stub_manager_notify_unmarshal(stubmgr, core::ptr::addr_of!(obj.std.ipid)) == FALSE {
                hres = CO_E_OBJNOTCONNECTED;
            }
            if SUCCEEDED(hres) && dest_context_known == FALSE {
                hres = super::stubmanager::ipid_get_dest_context(
                    core::ptr::addr_of!(obj.std.ipid),
                    &mut dest_context,
                    &mut dest_context_data,
                );
            }
        } else {
            WARN!("Couldn't find object for OXID %s, OID %s, assuming disconnected\n");
            hres = CO_E_OBJNOTCONNECTED;
        }
    } else {
        TRACE!("Treating unmarshal from OXID %s as inter-process\n");
    }

    if hres == S_OK {
        hres = unmarshal_object(
            core::ptr::addr_of!(obj.std),
            apt,
            dest_context,
            dest_context_data,
            riid,
            if !stubmgr.is_null() {
                core::ptr::addr_of!((*stubmgr).oxid_info)
            } else {
                core::ptr::null()
            },
            ppv,
        );
    }

    if !stubmgr.is_null() {
        stub_manager_int_release(stubmgr);
    }
    if !stub_apt.is_null() {
        apartment_release(stub_apt);
    }

    if hres != S_OK {
        WARN!("Failed with error 0x%08x\n");
    } else {
        TRACE!("Successfully created proxy %p\n");
    }

    apartment_release(apt);
    hres
}

unsafe extern "system" fn StdMarshalImpl_UnmarshalInterface(
    iface: *mut IMarshal,
    pStm: *mut IStream,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    let This = impl_from_StdMarshal(iface);
    let mut objref: OBJREF = core::mem::zeroed();
    let mut res: ULONG = 0;

    let hr = pStm.Read(
        core::ptr::addr_of_mut!(objref) as *mut c_void,
        OBJREF_HDR_SIZE as ULONG,
        &mut res,
    );
    if hr != S_OK || (res as usize != OBJREF_HDR_SIZE) {
        ERR!("Failed to read common OBJREF header, 0x%08x\n");
        return STG_E_READFAULT;
    }

    if objref.signature != OBJREF_SIGNATURE {
        ERR!("Bad OBJREF signature 0x%08x\n");
        return RPC_E_INVALID_OBJREF;
    }

    if objref.flags & OBJREF_STANDARD == 0 {
        FIXME!("unsupported objref.flags = %x\n");
        return E_NOTIMPL;
    }

    std_unmarshal_interface((*This).dest_context, (*This).dest_context_data, pStm, riid, ppv, TRUE)
}

unsafe fn std_release_marshal_data(pStm: *mut IStream) -> HRESULT {
    let mut obj: OR_STANDARD = core::mem::zeroed();
    let mut res: ULONG = 0;

    let hres = pStm.Read(
        core::ptr::addr_of_mut!(obj) as *mut c_void,
        OR_STANDARD_SIZE as ULONG,
        &mut res,
    );
    if hres != S_OK {
        return STG_E_READFAULT;
    }

    if obj.saResAddr.wNumEntries != 0 {
        ERR!("unsupported size of DUALSTRINGARRAY\n");
        return E_NOTIMPL;
    }

    TRACE!("oxid = %s, oid = %s, ipid = %s\n");

    let apt = apartment_findfromoxid(obj.std.oxid, TRUE);
    if apt.is_null() {
        WARN!("Could not map OXID %s to apartment object\n");
        return RPC_E_INVALID_OBJREF;
    }

    let stubmgr = get_stub_manager(apt, obj.std.oid);
    if stubmgr.is_null() {
        apartment_release(apt);
        ERR!("could not map object ID to stub manager, oxid=%s, oid=%s\n");
        return RPC_E_INVALID_OBJREF;
    }

    stub_manager_release_marshal_data(
        stubmgr,
        obj.std.cPublicRefs,
        core::ptr::addr_of!(obj.std.ipid),
        (obj.std.flags & SORFP_TABLEWEAK) as BOOL,
    );

    stub_manager_int_release(stubmgr);
    apartment_release(apt);

    S_OK
}

unsafe extern "system" fn StdMarshalImpl_ReleaseMarshalData(
    iface: *mut IMarshal,
    pStm: *mut IStream,
) -> HRESULT {
    let mut objref: OBJREF = core::mem::zeroed();
    let mut res: ULONG = 0;

    TRACE!("iface=%p, pStm=%p\n", iface, pStm);

    let hr = pStm.Read(
        core::ptr::addr_of_mut!(objref) as *mut c_void,
        OBJREF_HDR_SIZE as ULONG,
        &mut res,
    );
    if hr != S_OK || (res as usize != OBJREF_HDR_SIZE) {
        ERR!("Failed to read common OBJREF header, 0x%08x\n");
        return STG_E_READFAULT;
    }

    if objref.signature != OBJREF_SIGNATURE {
        ERR!("Bad OBJREF signature 0x%08x\n");
        return RPC_E_INVALID_OBJREF;
    }

    if objref.flags & OBJREF_STANDARD == 0 {
        FIXME!("unsupported objref.flags = %x\n");
        return E_NOTIMPL;
    }

    std_release_marshal_data(pStm)
}

unsafe extern "system" fn StdMarshalImpl_DisconnectObject(
    iface: *mut IMarshal,
    dwReserved: DWORD,
) -> HRESULT {
    FIXME!("(), stub!\n");
    S_OK
}

static StdMarshalVtbl: IMarshalVtbl = IMarshalVtbl {
    QueryInterface: StdMarshalImpl_QueryInterface,
    AddRef: StdMarshalImpl_AddRef,
    Release: StdMarshalImpl_Release,
    GetUnmarshalClass: StdMarshalImpl_GetUnmarshalClass,
    GetMarshalSizeMax: StdMarshalImpl_GetMarshalSizeMax,
    MarshalInterface: StdMarshalImpl_MarshalInterface,
    UnmarshalInterface: StdMarshalImpl_UnmarshalInterface,
    ReleaseMarshalData: StdMarshalImpl_ReleaseMarshalData,
    DisconnectObject: StdMarshalImpl_DisconnectObject,
};

unsafe fn StdMarshalImpl_Construct(
    riid: REFIID,
    dest_context: DWORD,
    dest_context_data: *mut c_void,
    ppvObject: *mut *mut c_void,
) -> HRESULT {
    let pStdMarshal =
        HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<StdMarshalImpl>()) as *mut StdMarshalImpl;
    if pStdMarshal.is_null() {
        return E_OUTOFMEMORY;
    }

    (*pStdMarshal).IMarshal_iface.lpVtbl = &StdMarshalVtbl;
    (*pStdMarshal).ref_ = 0;
    (*pStdMarshal).dest_context = dest_context;
    (*pStdMarshal).dest_context_data = dest_context_data;

    let hr = core::ptr::addr_of_mut!((*pStdMarshal).IMarshal_iface).QueryInterface(riid, ppvObject);
    if FAILED(hr) {
        HeapFree(GetProcessHeap(), 0, pStdMarshal as PVOID);
    }

    hr
}

/***********************************************************************
 *		CoGetStandardMarshal	[OLE32.@]
 *
 * Gets or creates a standard marshal object.
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetStandardMarshal(
    riid: REFIID,
    pUnk: *mut IUnknown,
    dwDestContext: DWORD,
    pvDestContext: LPVOID,
    mshlflags: DWORD,
    ppMarshal: *mut LPMARSHAL,
) -> HRESULT {
    if pUnk.is_null() {
        FIXME!("(%s,NULL,%x,%p,%x,%p), unimplemented yet.\n");
        return E_NOTIMPL;
    }
    TRACE!("(%s,%p,%x,%p,%x,%p)\n");

    StdMarshalImpl_Construct(
        &IID_IMarshal,
        dwDestContext,
        pvDestContext,
        ppMarshal as *mut *mut c_void,
    )
}

/***********************************************************************
 *		get_marshaler	[internal]
 *
 * Retrieves an IMarshal interface for an object.
 */
unsafe fn get_marshaler(
    riid: REFIID,
    pUnk: *mut IUnknown,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlFlags: DWORD,
    pMarshal: *mut LPMARSHAL,
) -> HRESULT {
    if pUnk.is_null() {
        return E_POINTER;
    }
    let mut hr = pUnk.QueryInterface(&IID_IMarshal, pMarshal as *mut *mut c_void);
    if hr != S_OK {
        hr = CoGetStandardMarshal(riid, pUnk, dwDestContext, pvDestContext, mshlFlags, pMarshal);
    }
    hr
}

/***********************************************************************
 *		get_unmarshaler_from_stream	[internal]
 *
 * Creates an IMarshal* object according to the data marshaled to the stream.
 * The function leaves the stream pointer at the start of the data written
 * to the stream by the IMarshal* object.
 */
unsafe fn get_unmarshaler_from_stream(
    stream: *mut IStream,
    marshal: *mut *mut IMarshal,
    iid: *mut IID,
) -> HRESULT {
    let mut res: ULONG = 0;
    let mut objref: OBJREF = core::mem::zeroed();

    /* read common OBJREF header */
    let mut hr = stream.Read(
        core::ptr::addr_of_mut!(objref) as *mut c_void,
        OBJREF_HDR_SIZE as ULONG,
        &mut res,
    );
    if hr != S_OK || (res as usize != OBJREF_HDR_SIZE) {
        ERR!("Failed to read common OBJREF header, 0x%08x\n");
        return STG_E_READFAULT;
    }

    /* sanity check on header */
    if objref.signature != OBJREF_SIGNATURE {
        ERR!("Bad OBJREF signature 0x%08x\n");
        return RPC_E_INVALID_OBJREF;
    }

    if !iid.is_null() {
        *iid = objref.iid;
    }

    /* FIXME: handler marshaling */
    if objref.flags & OBJREF_STANDARD != 0 {
        TRACE!("Using standard unmarshaling\n");
        *marshal = core::ptr::null_mut();
        return S_FALSE;
    } else if objref.flags & OBJREF_CUSTOM != 0 {
        let custom_header_size = OBJREF_CUSTOM_HDR_SIZE - OBJREF_HDR_SIZE;
        TRACE!("Using custom unmarshaling\n");
        /* read constant sized OR_CUSTOM data from stream */
        hr = stream.Read(
            core::ptr::addr_of_mut!(objref.u_objref.u_custom) as *mut c_void,
            custom_header_size as ULONG,
            &mut res,
        );
        if hr != S_OK || (res as usize != custom_header_size) {
            ERR!("Failed to read OR_CUSTOM header, 0x%08x\n");
            return STG_E_READFAULT;
        }
        /* now create the marshaler specified in the stream */
        hr = CoCreateInstance(
            core::ptr::addr_of!(objref.u_objref.u_custom.clsid),
            core::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &IID_IMarshal,
            marshal as *mut *mut c_void,
        );
    } else {
        FIXME!("Invalid or unimplemented marshaling type specified: %x\n");
        return RPC_E_INVALID_OBJREF;
    }

    if hr != S_OK {
        ERR!("Failed to create marshal, 0x%08x\n");
    }

    hr
}

/***********************************************************************
 *		CoGetMarshalSizeMax	[OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetMarshalSizeMax(
    pulSize: *mut ULONG,
    riid: REFIID,
    pUnk: *mut IUnknown,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlFlags: DWORD,
) -> HRESULT {
    let mut pMarshal: LPMARSHAL = core::ptr::null_mut();
    let mut std_marshal = FALSE;

    if pUnk.is_null() {
        return E_POINTER;
    }

    let mut hr = pUnk.QueryInterface(&IID_IMarshal, core::ptr::addr_of_mut!(pMarshal) as *mut *mut c_void);
    if hr != S_OK {
        std_marshal = TRUE;
        hr = CoGetStandardMarshal(
            riid,
            pUnk,
            dwDestContext,
            pvDestContext,
            mshlFlags,
            &mut pMarshal,
        );
    }
    if hr != S_OK {
        return hr;
    }

    hr = pMarshal.GetMarshalSizeMax(riid, pUnk as *mut c_void, dwDestContext, pvDestContext, mshlFlags, pulSize);
    if std_marshal == FALSE {
        /* add on the size of the whole OBJREF structure like native does */
        *pulSize += core::mem::size_of::<OBJREF>() as ULONG;
    }

    pMarshal.Release();
    hr
}

unsafe fn dump_MSHLFLAGS(flags: MSHLFLAGS) {
    if flags & MSHLFLAGS_TABLESTRONG != 0 {
        TRACE!(" MSHLFLAGS_TABLESTRONG");
    }
    if flags & MSHLFLAGS_TABLEWEAK != 0 {
        TRACE!(" MSHLFLAGS_TABLEWEAK");
    }
    if flags & (MSHLFLAGS_TABLESTRONG | MSHLFLAGS_TABLEWEAK) == 0 {
        TRACE!(" MSHLFLAGS_NORMAL");
    }
    if flags & MSHLFLAGS_NOPING != 0 {
        TRACE!(" MSHLFLAGS_NOPING");
    }
}

/***********************************************************************
 *		CoMarshalInterface	[OLE32.@]
 *
 * Marshals an interface into a stream so that the object can then be
 * unmarshaled from another COM apartment and used remotely.
 */
#[no_mangle]
pub unsafe extern "system" fn CoMarshalInterface(
    pStream: *mut IStream,
    riid: REFIID,
    pUnk: *mut IUnknown,
    dwDestContext: DWORD,
    pvDestContext: *mut c_void,
    mshlFlags: DWORD,
) -> HRESULT {
    let mut marshaler_clsid: CLSID = GUID_NULL;
    let mut pMarshal: LPMARSHAL = core::ptr::null_mut();

    TRACE!("(%p, %s, %p, %x, %p, ");
    dump_MSHLFLAGS(mshlFlags);
    TRACE!(")\n");

    if pUnk.is_null() || pStream.is_null() {
        return E_INVALIDARG;
    }

    /* get the marshaler for the specified interface */
    let mut hr = get_marshaler(riid, pUnk, dwDestContext, pvDestContext, mshlFlags, &mut pMarshal);
    if hr != S_OK {
        ERR!("Failed to get marshaller, 0x%08x\n");
        return hr;
    }

    'cleanup: {
        hr = pMarshal.GetUnmarshalClass(
            riid,
            pUnk as *mut c_void,
            dwDestContext,
            pvDestContext,
            mshlFlags,
            &mut marshaler_clsid,
        );
        if hr != S_OK {
            ERR!("IMarshal::GetUnmarshalClass failed, 0x%08x\n");
            break 'cleanup;
        }

        /* FIXME: implement handler marshaling too */
        if IsEqualCLSID(&marshaler_clsid, &CLSID_StdMarshal) {
            TRACE!("Using standard marshaling\n");
        } else {
            let mut objref: OBJREF = core::mem::zeroed();

            TRACE!("Using custom marshaling\n");
            objref.signature = OBJREF_SIGNATURE;
            objref.iid = *riid;
            objref.flags = OBJREF_CUSTOM;
            objref.u_objref.u_custom.clsid = marshaler_clsid;
            objref.u_objref.u_custom.cbExtension = 0;
            objref.u_objref.u_custom.size = 0;
            hr = pMarshal.GetMarshalSizeMax(
                riid,
                pUnk as *mut c_void,
                dwDestContext,
                pvDestContext,
                mshlFlags,
                core::ptr::addr_of_mut!(objref.u_objref.u_custom.size),
            );
            if hr != S_OK {
                ERR!("Failed to get max size of marshal data, error 0x%08x\n");
                break 'cleanup;
            }
            /* write constant sized common header and OR_CUSTOM data into stream */
            hr = pStream.Write(
                core::ptr::addr_of!(objref) as *const c_void,
                OBJREF_CUSTOM_HDR_SIZE as ULONG,
                core::ptr::null_mut(),
            );
            if hr != S_OK {
                ERR!("Failed to write OR_CUSTOM header to stream with 0x%08x\n");
                break 'cleanup;
            }
        }

        TRACE!("Calling IMarshal::MarshalInterface\n");
        /* call helper object to do the actual marshaling */
        hr = pMarshal.MarshalInterface(
            pStream,
            riid,
            pUnk as *mut c_void,
            dwDestContext,
            pvDestContext,
            mshlFlags,
        );

        if hr != S_OK {
            ERR!("Failed to marshal the interface %s, %x\n");
            break 'cleanup;
        }
    }

    /* cleanup: */
    pMarshal.Release();

    TRACE!("completed with hr 0x%08x\n");

    hr
}

/***********************************************************************
 *		CoUnmarshalInterface	[OLE32.@]
 *
 * Unmarshals an object from a stream by creating a proxy to the remote
 * object, if necessary.
 */
#[no_mangle]
pub unsafe extern "system" fn CoUnmarshalInterface(
    pStream: *mut IStream,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut pMarshal: LPMARSHAL = core::ptr::null_mut();
    let mut iid: IID = GUID_NULL;
    let mut object: *mut IUnknown = core::ptr::null_mut();

    TRACE!("(%p, %s, %p)\n");

    if pStream.is_null() || ppv.is_null() {
        return E_INVALIDARG;
    }

    let mut hr = get_unmarshaler_from_stream(pStream, &mut pMarshal, &mut iid);
    if hr == S_FALSE {
        hr = std_unmarshal_interface(
            0,
            core::ptr::null_mut(),
            pStream,
            &iid,
            core::ptr::addr_of_mut!(object) as *mut *mut c_void,
            FALSE,
        );
        if hr != S_OK {
            ERR!("StdMarshal UnmarshalInterface failed, 0x%08x\n");
        }
    } else if hr == S_OK {
        /* call the helper object to do the actual unmarshaling */
        hr = pMarshal.UnmarshalInterface(
            pStream,
            &iid,
            core::ptr::addr_of_mut!(object) as *mut *mut c_void,
        );
        pMarshal.Release();
        if hr != S_OK {
            ERR!("IMarshal::UnmarshalInterface failed, 0x%08x\n");
        }
    }

    if hr == S_OK {
        /* IID_NULL means use the interface ID of the marshaled object */
        if !IsEqualIID(riid, &IID_NULL) && !IsEqualIID(riid, &iid) {
            TRACE!("requested interface != marshalled interface, additional QI needed\n");
            hr = object.QueryInterface(riid, ppv as *mut *mut c_void);
            if hr != S_OK {
                ERR!("Couldn't query for interface %s, hr = 0x%08x\n");
            }
            object.Release();
        } else {
            *ppv = object as *mut c_void;
        }
    }

    TRACE!("completed with hr 0x%x\n");

    hr
}

/***********************************************************************
 *		CoReleaseMarshalData	[OLE32.@]
 *
 * Releases resources associated with an object that has been marshaled into
 * a stream.
 */
#[no_mangle]
pub unsafe extern "system" fn CoReleaseMarshalData(pStream: *mut IStream) -> HRESULT {
    let mut pMarshal: LPMARSHAL = core::ptr::null_mut();

    TRACE!("(%p)\n", pStream);

    let mut hr = get_unmarshaler_from_stream(pStream, &mut pMarshal, core::ptr::null_mut());
    if hr == S_FALSE {
        hr = std_release_marshal_data(pStream);
        if hr != S_OK {
            ERR!("StdMarshal ReleaseMarshalData failed with error 0x%08x\n");
        }
        return hr;
    }
    if hr != S_OK {
        return hr;
    }

    /* call the helper object to do the releasing of marshal data */
    hr = pMarshal.ReleaseMarshalData(pStream);
    if hr != S_OK {
        ERR!("IMarshal::ReleaseMarshalData failed with error 0x%08x\n");
    }

    pMarshal.Release();
    hr
}

/***********************************************************************
 *		CoMarshalInterThreadInterfaceInStream	[OLE32.@]
 *
 * Marshal an interface across threads in the same process.
 */
#[no_mangle]
pub unsafe extern "system" fn CoMarshalInterThreadInterfaceInStream(
    riid: REFIID,
    pUnk: *mut IUnknown,
    ppStm: *mut LPSTREAM,
) -> HRESULT {
    let mut xpos: ULARGE_INTEGER = 0;
    let seekto: LARGE_INTEGER = 0;

    TRACE!("(%s, %p, %p)\n");

    let mut hres = CreateStreamOnHGlobal(core::ptr::null_mut(), TRUE, ppStm);
    if FAILED(hres) {
        return hres;
    }
    hres = CoMarshalInterface(
        *ppStm,
        riid,
        pUnk,
        MSHCTX_INPROC,
        core::ptr::null_mut(),
        MSHLFLAGS_NORMAL,
    );

    if SUCCEEDED(hres) {
        (*ppStm).Seek(seekto, STREAM_SEEK_SET, &mut xpos);
    } else {
        (*ppStm).Release();
        *ppStm = core::ptr::null_mut();
    }

    hres
}

/***********************************************************************
 *		CoGetInterfaceAndReleaseStream	[OLE32.@]
 *
 * Unmarshalls an interface from a stream and then releases the stream.
 */
#[no_mangle]
pub unsafe extern "system" fn CoGetInterfaceAndReleaseStream(
    pStm: LPSTREAM,
    riid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    TRACE!("(%p, %s, %p)\n");

    if pStm.is_null() {
        return E_INVALIDARG;
    }
    let hres = CoUnmarshalInterface(pStm, riid, ppv);
    pStm.Release();
    hres
}

// ---------------------------------------------------------------------------
// StdMarshal class factory
// ---------------------------------------------------------------------------

unsafe extern "system" fn StdMarshalCF_QueryInterface(
    iface: *mut IClassFactory,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    *ppv = core::ptr::null_mut();
    if IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IClassFactory) {
        *ppv = iface as *mut c_void;
        return S_OK;
    }
    E_NOINTERFACE
}

unsafe extern "system" fn StdMarshalCF_AddRef(iface: *mut IClassFactory) -> ULONG {
    2 /* non-heap based object */
}

unsafe extern "system" fn StdMarshalCF_Release(iface: *mut IClassFactory) -> ULONG {
    1 /* non-heap based object */
}

unsafe extern "system" fn StdMarshalCF_CreateInstance(
    iface: *mut IClassFactory,
    pUnk: *mut IUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IMarshal) {
        return StdMarshalImpl_Construct(riid, 0, core::ptr::null_mut(), ppv);
    }

    FIXME!("(%s), not supported.\n");
    E_NOINTERFACE
}

unsafe extern "system" fn StdMarshalCF_LockServer(iface: *mut IClassFactory, fLock: BOOL) -> HRESULT {
    FIXME!("(%d), stub!\n");
    S_OK
}

static StdMarshalCFVtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: StdMarshalCF_QueryInterface,
    AddRef: StdMarshalCF_AddRef,
    Release: StdMarshalCF_Release,
    CreateInstance: StdMarshalCF_CreateInstance,
    LockServer: StdMarshalCF_LockServer,
};
/// C: `static const IClassFactoryVtbl *StdMarshalCF = &StdMarshalCFVtbl;`
/// The ADDRESS of this pointer is the IClassFactory object handed out by
/// MARSHAL_GetStandardMarshalCF (identical memory shape to the C).
static StdMarshalCF: ComStatic<IClassFactory> = ComStatic(IClassFactory {
    lpVtbl: &StdMarshalCFVtbl,
});

/// (compobj_private.h: HRESULT MARSHAL_GetStandardMarshalCF(LPVOID*))
#[no_mangle]
pub unsafe extern "C" fn MARSHAL_GetStandardMarshalCF(ppv: *mut LPVOID) -> HRESULT {
    *ppv = core::ptr::addr_of!(StdMarshalCF.0) as *mut c_void;
    S_OK
}

/***********************************************************************
 *		CoMarshalHresult	[OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoMarshalHresult(pStm: LPSTREAM, hresult: HRESULT) -> HRESULT {
    pStm.Write(
        core::ptr::addr_of!(hresult) as *const c_void,
        core::mem::size_of::<HRESULT>() as ULONG,
        core::ptr::null_mut(),
    )
}

/***********************************************************************
 *		CoUnmarshalHresult	[OLE32.@]
 */
#[no_mangle]
pub unsafe extern "system" fn CoUnmarshalHresult(pStm: LPSTREAM, phresult: *mut HRESULT) -> HRESULT {
    pStm.Read(
        phresult as *mut c_void,
        core::mem::size_of::<HRESULT>() as ULONG,
        core::ptr::null_mut(),
    )
}
