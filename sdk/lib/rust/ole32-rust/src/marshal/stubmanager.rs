//! stubmanager.rs — translation of dll/win32/ole32/stubmanager.c.
//!
//! A stub manager is an object that controls interface stubs. It is
//! identified by an OID (object identifier) and acts as the network identity
//! of the object. There can be many stub managers in a process or apartment.
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

extern "C" {
    /// dll/win32/ole32/stubmanager_seh.c: IUnknown_Release under a native __try
    /// (amd64/arm64); FALSE if the release faulted and the fault was swallowed.
    fn ole32_release_object_guarded(object: *mut IUnknown) -> BOOL;
}

/// generates an ipid in the following format (similar to native version):
/// Data1 = apartment-local ipid counter
/// Data2 = apartment creator thread ID, or 0 for an MTA (wine-10)
/// Data3 = process ID
/// Data4 = random value
unsafe fn generate_ipid(m: *mut stub_manager, ipid: *mut IPID) -> HRESULT {
    let hr = UuidCreate(ipid);
    if FAILED(hr) {
        ERR!("couldn't create IPID for stub manager %p\n");
        UuidCreateNil(ipid);
        return hr;
    }

    (*ipid).Data1 =
        InterlockedIncrement(core::ptr::addr_of_mut!((*(*m).apt).ipidc)) as u32;
    // wine-10: an MTA outlives its creator's membership (that thread can leave and
    // become an STA), so its ipids must not be routed by the creator's tid.
    (*ipid).Data2 = if (*(*m).apt).multi_threaded == FALSE { (*(*m).apt).tid as USHORT } else { 0 };
    (*ipid).Data3 = GetCurrentProcessId() as USHORT;
    S_OK
}

/// registers a new interface stub COM object with the stub manager and
/// returns registration record
#[no_mangle]
pub unsafe extern "C" fn stub_manager_new_ifstub(
    m: *mut stub_manager,
    sb: *mut IRpcStubBuffer,
    iid: REFIID,
    dest_context: DWORD,
    dest_context_data: *mut c_void,
    flags: MSHLFLAGS,
) -> *mut ifstub {
    let mut hr: HRESULT;

    TRACE!("oid=%s, stubbuffer=%p, iid=%s, dest_context=%x\n");

    let stub = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<ifstub>())
        as *mut ifstub;
    if stub.is_null() {
        return core::ptr::null_mut();
    }

    hr = (*m).object.QueryInterface(
        iid,
        core::ptr::addr_of_mut!((*stub).iface) as *mut *mut c_void,
    );
    if hr != S_OK {
        HeapFree(GetProcessHeap(), 0, stub as PVOID);
        return core::ptr::null_mut();
    }

    hr = RPC_CreateServerChannel(
        dest_context,
        dest_context_data,
        core::ptr::addr_of_mut!((*stub).chan),
    );
    if hr != S_OK {
        (*stub).iface.Release();
        HeapFree(GetProcessHeap(), 0, stub as PVOID);
        return core::ptr::null_mut();
    }

    (*stub).stubbuffer = sb;
    if !sb.is_null() {
        sb.AddRef();
    }

    (*stub).flags = flags;
    (*stub).iid = *iid;

    /* FIXME: find a cleaner way of identifying that we are creating an ifstub
     * for the remunknown interface */
    if flags & MSHLFLAGSP_REMUNKNOWN != 0 {
        (*stub).ipid = (*m).oxid_info.ipidRemUnknown;
    } else {
        generate_ipid(m, core::ptr::addr_of_mut!((*stub).ipid));
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*m).lock));
    list_add_head(
        core::ptr::addr_of_mut!((*m).ifstubs),
        core::ptr::addr_of_mut!((*stub).entry),
    );
    /* every normal marshal is counted so we don't allow more than we should */
    // NOTE (faithful): MSHLFLAGS_NORMAL == 0, so `flags & MSHLFLAGS_NORMAL`
    // is always 0 — this is dead code in the C as well; kept as-is.
    if flags & MSHLFLAGS_NORMAL != 0 {
        (*m).norm_refs += 1;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*m).lock));

    TRACE!("ifstub %p created with ipid %s\n");

    stub
}

unsafe fn stub_manager_delete_ifstub(m: *mut stub_manager, ifstub_p: *mut ifstub) {
    TRACE!("m=%p, m->oid=%s, ipid=%s\n");

    list_remove(core::ptr::addr_of_mut!((*ifstub_p).entry));

    if (*m).disconnected == FALSE {
        RPC_UnregisterInterface(core::ptr::addr_of!((*ifstub_p).iid), TRUE);
    }

    if !(*ifstub_p).stubbuffer.is_null() {
        (*ifstub_p).stubbuffer.Release();
    }
    (*ifstub_p).iface.Release();
    (*ifstub_p).chan.Release();

    HeapFree(GetProcessHeap(), 0, ifstub_p as PVOID);
}

unsafe fn stub_manager_ipid_to_ifstub(m: *mut stub_manager, ipid: *const IPID) -> *mut ifstub {
    let mut result: *mut ifstub = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!((*m).lock));
    let head = core::ptr::addr_of_mut!((*m).ifstubs);
    let mut cursor = (*head).next;
    while cursor != head {
        let ifstub_p = containing_record!(cursor, ifstub, entry);
        if IsEqualGUID(ipid, core::ptr::addr_of!((*ifstub_p).ipid)) {
            result = ifstub_p;
            break;
        }
        cursor = (*cursor).next;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*m).lock));

    result
}

#[no_mangle]
pub unsafe extern "C" fn stub_manager_find_ifstub(
    m: *mut stub_manager,
    iid: REFIID,
    flags: MSHLFLAGS,
) -> *mut ifstub {
    let mut result: *mut ifstub = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!((*m).lock));
    let head = core::ptr::addr_of_mut!((*m).ifstubs);
    let mut cursor = (*head).next;
    while cursor != head {
        let ifstub_p = containing_record!(cursor, ifstub, entry);
        if IsEqualIID(iid, core::ptr::addr_of!((*ifstub_p).iid)) && ((*ifstub_p).flags == flags) {
            result = ifstub_p;
            break;
        }
        cursor = (*cursor).next;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*m).lock));

    result
}

/// creates a new stub manager and adds it into the apartment. caller must
/// release stub manager when it is no longer required. the apartment and
/// external refs together take one implicit ref
unsafe fn new_stub_manager(apt: *mut apartment, object: *mut IUnknown) -> *mut stub_manager {
    c_assert!(!apt.is_null());

    let sm = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<stub_manager>(),
    ) as *mut stub_manager;
    if sm.is_null() {
        return core::ptr::null_mut();
    }

    list_init(core::ptr::addr_of_mut!((*sm).ifstubs));

    InitializeCriticalSection(core::ptr::addr_of_mut!((*sm).lock));
    DEBUG_SET_CRITSEC_NAME(core::ptr::addr_of_mut!((*sm).lock), "stub_manager");

    object.AddRef();
    (*sm).object = object;
    (*sm).apt = apt;

    /* start off with 2 references because the stub is in the apartment
     * and the caller will also hold a reference */
    (*sm).refs = 2;
    (*sm).weakrefs = 0;

    (*sm).oxid_info.dwPid = GetCurrentProcessId();
    (*sm).oxid_info.dwTid = GetCurrentThreadId();
    /*
     * FIXME: this is a hack for marshalling IRemUnknown. In real
     * DCOM, the IPID of the IRemUnknown interface is generated like
     * any other and passed to the OXID resolver which then returns it
     * when queried. We don't have an OXID resolver yet so instead we
     * use a magic IPID reserved for IRemUnknown.
     */
    (*sm).oxid_info.ipidRemUnknown.Data1 = 0xffffffff;
    (*sm).oxid_info.ipidRemUnknown.Data2 = 0xffff;
    (*sm).oxid_info.ipidRemUnknown.Data3 = 0xffff;
    const _: () = assert!(core::mem::size_of::<[u8; 8]>() == core::mem::size_of::<OXID>());
    memcpy(
        core::ptr::addr_of_mut!((*sm).oxid_info.ipidRemUnknown.Data4) as PVOID,
        core::ptr::addr_of!((*apt).oxid) as *const c_void,
        core::mem::size_of::<OXID>(),
    );
    (*sm).oxid_info.dwAuthnHint = RPC_C_AUTHN_LEVEL_NONE;
    (*sm).oxid_info.psa = core::ptr::null_mut() /* FIXME */;

    /* Yes, that's right, this starts at zero. that's zero EXTERNAL
     * refs, i.e., nobody has unmarshalled anything yet. We can't have
     * negative refs because the stub manager cannot be explicitly
     * killed, it has to die by somebody unmarshalling then releasing
     * the marshalled ifptr.
     */
    (*sm).extrefs = 0;
    (*sm).disconnected = FALSE;

    let hres = object.QueryInterface(
        &IID_IExternalConnection,
        core::ptr::addr_of_mut!((*sm).extern_conn) as *mut *mut c_void,
    );
    if FAILED(hres) {
        (*sm).extern_conn = core::ptr::null_mut();
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*apt).cs));
    (*sm).oid = (*apt).oidc;
    (*apt).oidc += 1;
    list_add_head(
        core::ptr::addr_of_mut!((*apt).stubmgrs),
        core::ptr::addr_of_mut!((*sm).entry),
    );
    LeaveCriticalSection(core::ptr::addr_of_mut!((*apt).cs));

    TRACE!("Created new stub manager (oid=%s) at %p for object with IUnknown %p\n");

    sm
}

#[no_mangle]
pub unsafe extern "C" fn stub_manager_disconnect(m: *mut stub_manager) {
    EnterCriticalSection(core::ptr::addr_of_mut!((*m).lock));
    if (*m).disconnected == FALSE {
        let head = core::ptr::addr_of_mut!((*m).ifstubs);
        let mut cursor = (*head).next;
        while cursor != head {
            let ifstub_p = containing_record!(cursor, ifstub, entry);
            RPC_UnregisterInterface(core::ptr::addr_of!((*ifstub_p).iid), FALSE);
            cursor = (*cursor).next;
        }

        (*m).disconnected = TRUE;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*m).lock));
}

/// caller must remove stub manager from apartment prior to calling this
/// function
unsafe fn stub_manager_delete(m: *mut stub_manager) {
    TRACE!("destroying %p (oid=%s)\n");

    /* release every ifstub */
    loop {
        let cursor = list_head(core::ptr::addr_of!((*m).ifstubs));
        if cursor.is_null() {
            break;
        }
        let ifstub_p = containing_record!(cursor, ifstub, entry);
        stub_manager_delete_ifstub(m, ifstub_p);
    }

    if !(*m).extern_conn.is_null() {
        (*m).extern_conn.Release();
    }

    CoTaskMemFree((*m).oxid_info.psa as PVOID);

    /* C: __TRY / __EXCEPT_PAGE_FAULT around the release ("some broken apps
     * crash in object destructors"; XP+ swallows the fault). The guard lives in
     * a native-SEH C thunk (dll/win32/ole32/stubmanager_seh.c), since Rust has
     * no __try and DUMMY_PSEH had compiled the C's out. */
    if ole32_release_object_guarded((*m).object) == FALSE {
        ERR!("Got page fault when releasing stub!\n");
    }

    DEBUG_CLEAR_CRITSEC_NAME(core::ptr::addr_of_mut!((*m).lock));
    DeleteCriticalSection(core::ptr::addr_of_mut!((*m).lock));

    HeapFree(GetProcessHeap(), 0, m as PVOID);
}

/// increments the internal refcount
unsafe fn stub_manager_int_addref(This: *mut stub_manager) -> ULONG {
    EnterCriticalSection(core::ptr::addr_of_mut!((*(*This).apt).cs));
    (*This).refs += 1;
    let refs = (*This).refs;
    LeaveCriticalSection(core::ptr::addr_of_mut!((*(*This).apt).cs));

    TRACE!("before %d\n");

    refs
}

/// decrements the internal refcount
#[no_mangle]
pub unsafe extern "C" fn stub_manager_int_release(This: *mut stub_manager) -> ULONG {
    let apt = (*This).apt;

    EnterCriticalSection(core::ptr::addr_of_mut!((*apt).cs));
    (*This).refs -= 1;
    let refs = (*This).refs;

    TRACE!("after %d\n");

    /* remove from apartment so no other thread can access it... */
    if refs == 0 {
        list_remove(core::ptr::addr_of_mut!((*This).entry));
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*apt).cs));

    /* ... so now we can delete it without being inside the apartment critsec */
    if refs == 0 {
        stub_manager_delete(This);
    }

    refs
}

/// gets the stub manager associated with an object - caller must have
/// a reference to the apartment while a reference to the stub manager is
/// held. it must also call release on the stub manager when it is no longer
/// needed
#[no_mangle]
pub unsafe extern "C" fn get_stub_manager_from_object(
    apt: *mut apartment,
    obj: *mut IUnknown,
    alloc: BOOL,
) -> *mut stub_manager {
    let mut result: *mut stub_manager = core::ptr::null_mut();
    let mut object: *mut IUnknown = core::ptr::null_mut();

    let hres = obj.QueryInterface(
        &IID_IUnknown,
        core::ptr::addr_of_mut!(object) as *mut *mut c_void,
    );
    if FAILED(hres) {
        ERR!("QueryInterface(IID_IUnknown failed): %08x\n");
        return core::ptr::null_mut();
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*apt).cs));
    let head = core::ptr::addr_of_mut!((*apt).stubmgrs);
    let mut cursor = (*head).next;
    while cursor != head {
        let m = containing_record!(cursor, stub_manager, entry);
        if (*m).object == object {
            result = m;
            stub_manager_int_addref(result);
            break;
        }
        cursor = (*cursor).next;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*apt).cs));

    if !result.is_null() {
        TRACE!("found %p for object %p\n");
    } else if alloc != FALSE {
        TRACE!("not found, creating new stub manager...\n");
        result = new_stub_manager(apt, object);
    } else {
        TRACE!("not found for object %p\n");
    }

    object.Release();
    result
}

/// gets the stub manager associated with an object id - caller must have
/// a reference to the apartment while a reference to the stub manager is
/// held. it must also call release on the stub manager when it is no longer
/// needed
#[no_mangle]
pub unsafe extern "C" fn get_stub_manager(apt: *mut apartment, oid: OID) -> *mut stub_manager {
    let mut result: *mut stub_manager = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!((*apt).cs));
    let head = core::ptr::addr_of_mut!((*apt).stubmgrs);
    let mut cursor = (*head).next;
    while cursor != head {
        let m = containing_record!(cursor, stub_manager, entry);
        if (*m).oid == oid {
            result = m;
            stub_manager_int_addref(result);
            break;
        }
        cursor = (*cursor).next;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*apt).cs));

    if !result.is_null() {
        TRACE!("found %p for oid %s\n");
    } else {
        TRACE!("not found for oid %s\n");
    }

    result
}

/// add some external references (ie from a client that unmarshaled an ifptr)
#[no_mangle]
pub unsafe extern "C" fn stub_manager_ext_addref(
    m: *mut stub_manager,
    mut refs: ULONG,
    tableweak: BOOL,
) -> ULONG {
    EnterCriticalSection(core::ptr::addr_of_mut!((*m).lock));

    let first_extern_ref = refs != 0 && (*m).extrefs == 0;

    /* make sure we don't overflow extrefs */
    refs = min_u32(refs, u32::MAX - 1 - (*m).extrefs);
    (*m).extrefs += refs;
    let mut rc = (*m).extrefs;

    if tableweak != FALSE {
        (*m).weakrefs += 1;
        rc += (*m).weakrefs;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*m).lock));

    TRACE!("added %u refs to %p (oid %s), rc is now %u\n");

    /*
     * NOTE: According to tests, creating a stub causes two AddConnection
     * calls followed by one ReleaseConnection call (with
     * fLastReleaseCloses=FALSE).
     */
    if first_extern_ref && !(*m).extern_conn.is_null() {
        (*m).extern_conn.AddConnection(EXTCONN_STRONG, 0);
    }

    rc
}

/// remove some external references
#[no_mangle]
pub unsafe extern "C" fn stub_manager_ext_release(
    m: *mut stub_manager,
    mut refs: ULONG,
    tableweak: BOOL,
    last_unlock_releases: BOOL,
) -> ULONG {
    EnterCriticalSection(core::ptr::addr_of_mut!((*m).lock));

    /* make sure we don't underflow extrefs */
    refs = min_u32(refs, (*m).extrefs);
    (*m).extrefs -= refs;
    let mut rc = (*m).extrefs;

    if tableweak != FALSE {
        (*m).weakrefs -= 1;
    }
    if last_unlock_releases == FALSE {
        rc += (*m).weakrefs;
    }

    let last_extern_ref = refs != 0 && (*m).extrefs == 0;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*m).lock));

    TRACE!("removed %u refs from %p (oid %s), rc is now %u\n");

    if last_extern_ref && !(*m).extern_conn.is_null() {
        (*m).extern_conn
            .ReleaseConnection(EXTCONN_STRONG, 0, last_unlock_releases);
    }

    if rc == 0 {
        if !(!(*m).extern_conn.is_null() && last_unlock_releases != FALSE && (*m).weakrefs != 0) {
            stub_manager_int_release(m);
        }
    }

    rc
}

/// gets the stub manager associated with an ipid - caller must have
/// a reference to the apartment while a reference to the stub manager is
/// held. it must also call release on the stub manager when it is no longer
/// needed
unsafe fn get_stub_manager_from_ipid(
    apt: *mut apartment,
    ipid: *const IPID,
    ifstub_out: *mut *mut ifstub,
) -> *mut stub_manager {
    let mut result: *mut stub_manager = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!((*apt).cs));
    let head = core::ptr::addr_of_mut!((*apt).stubmgrs);
    let mut cursor = (*head).next;
    while cursor != head {
        let m = containing_record!(cursor, stub_manager, entry);
        *ifstub_out = stub_manager_ipid_to_ifstub(m, ipid);
        if !(*ifstub_out).is_null() {
            result = m;
            stub_manager_int_addref(result);
            break;
        }
        cursor = (*cursor).next;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*apt).cs));

    if !result.is_null() {
        TRACE!("found %p for ipid %s\n");
    } else {
        ERR!("not found for ipid %s\n");
    }

    result
}

unsafe fn ipid_to_ifstub(
    ipid: *const IPID,
    stub_apt: *mut *mut apartment,
    stubmgr_ret: *mut *mut stub_manager,
    ifstub_out: *mut *mut ifstub,
) -> HRESULT {
    /* FIXME: hack for IRemUnknown */
    if (*ipid).Data2 == 0xffff {
        /* C: *(const OXID *)ipid->Data4 — Data4 is only 4-aligned, so use an
         * unaligned read (same bytes, no UB) */
        *stub_apt = apartment_findfromoxid(
            core::ptr::read_unaligned(core::ptr::addr_of!((*ipid).Data4) as *const OXID),
            TRUE,
        );
    } else if (*ipid).Data2 == 0 && (*ipid).Data3 == GetCurrentProcessId() as USHORT {
        *stub_apt = apartment_get_mta();
    } else {
        *stub_apt = apartment_findfromtid((*ipid).Data2 as DWORD);
    }
    if (*stub_apt).is_null() {
        TRACE!("Couldn't find apartment corresponding to TID 0x%04x\n");
        return RPC_E_INVALID_OBJECT;
    }
    *stubmgr_ret = get_stub_manager_from_ipid(*stub_apt, ipid, ifstub_out);
    if (*stubmgr_ret).is_null() {
        apartment_release(*stub_apt);
        *stub_apt = core::ptr::null_mut();
        return RPC_E_INVALID_OBJECT;
    }
    S_OK
}

unsafe fn ipid_to_stub_manager(
    ipid: *const IPID,
    stub_apt: *mut *mut apartment,
    stub: *mut *mut stub_manager,
) -> HRESULT {
    let mut ifstub_p: *mut ifstub = core::ptr::null_mut();
    ipid_to_ifstub(ipid, stub_apt, stub, &mut ifstub_p)
}

/// wine-10 combase: the destination context of the channel serving `ipid`. A proxy
/// unmarshaled in the object's own process inherits it, so its channel reports
/// MSHCTX_INPROC instead of the MSHCTX_LOCAL default.
pub(crate) unsafe fn ipid_get_dest_context(
    ipid: *const IPID,
    dest_context: *mut MSHCTX,
    dest_context_data: *mut *mut c_void,
) -> HRESULT {
    let mut apt: *mut apartment = core::ptr::null_mut();
    let mut stubmgr: *mut stub_manager = core::ptr::null_mut();
    let mut ifstub_p: *mut ifstub = core::ptr::null_mut();
    let mut ctx: DWORD = 0;
    let mut data: *mut c_void = core::ptr::null_mut();

    if ipid_to_ifstub(ipid, &mut apt, &mut stubmgr, &mut ifstub_p) != S_OK {
        return RPC_E_DISCONNECTED;
    }

    let hr = (*ifstub_p).chan.GetDestCtx(&mut ctx, &mut data);
    if SUCCEEDED(hr) {
        *dest_context = ctx as MSHCTX;
        *dest_context_data = data;
    }

    stub_manager_int_release(stubmgr);
    apartment_release(apt);
    hr
}

/// gets the apartment, stub and channel of an object. the caller must
/// release the references to all objects (except iface) if the function
/// returned success, otherwise no references are returned.
#[no_mangle]
pub unsafe extern "C" fn ipid_get_dispatch_params(
    ipid: *const IPID,
    stub_apt: *mut *mut apartment,
    manager: *mut *mut stub_manager,
    stub: *mut *mut IRpcStubBuffer,
    chan: *mut *mut IRpcChannelBuffer,
    iid: *mut IID,
    iface: *mut *mut IUnknown,
) -> HRESULT {
    let mut stubmgr: *mut stub_manager = core::ptr::null_mut();
    let mut ifstub_p: *mut ifstub = core::ptr::null_mut();
    let mut apt: *mut apartment = core::ptr::null_mut();

    let hr = ipid_to_ifstub(ipid, &mut apt, &mut stubmgr, &mut ifstub_p);
    if hr != S_OK {
        return RPC_E_DISCONNECTED;
    }

    *stub = (*ifstub_p).stubbuffer;
    (*stub).AddRef();
    *chan = (*ifstub_p).chan;
    (*chan).AddRef();
    *stub_apt = apt;
    *iid = (*ifstub_p).iid;
    *iface = (*ifstub_p).iface;

    if !manager.is_null() {
        *manager = stubmgr;
    } else {
        stub_manager_int_release(stubmgr);
    }
    S_OK
}

/// returns TRUE if it is possible to unmarshal, FALSE otherwise.
#[no_mangle]
pub unsafe extern "C" fn stub_manager_notify_unmarshal(
    m: *mut stub_manager,
    ipid: *const IPID,
) -> BOOL {
    let mut ret: BOOL = TRUE;

    let ifstub_p = stub_manager_ipid_to_ifstub(m, ipid);
    if ifstub_p.is_null() {
        ERR!("attempted unmarshal of unknown IPID %s\n");
        return FALSE;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*m).lock));

    /* track normal marshals so we can enforce rules whilst in-process */
    // NOTE (faithful): MSHLFLAGS_NORMAL == 0, so this branch is dead in the
    // C too; kept as-is.
    if (*ifstub_p).flags & MSHLFLAGS_NORMAL != 0 {
        if (*m).norm_refs != 0 {
            (*m).norm_refs -= 1;
        } else {
            ERR!("attempted invalid normal unmarshal, norm_refs is zero\n");
            ret = FALSE;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*m).lock));

    ret
}

/// handles refcounting for CoReleaseMarshalData
#[no_mangle]
pub unsafe extern "C" fn stub_manager_release_marshal_data(
    m: *mut stub_manager,
    mut refs: ULONG,
    ipid: *const IPID,
    tableweak: BOOL,
) {
    let ifstub_p = stub_manager_ipid_to_ifstub(m, ipid);
    if ifstub_p.is_null() {
        return;
    }

    if (*ifstub_p).flags & MSHLFLAGS_TABLEWEAK != 0 {
        refs = 0;
    } else if (*ifstub_p).flags & MSHLFLAGS_TABLESTRONG != 0 {
        refs = 1;
    }

    stub_manager_ext_release(m, refs, tableweak, (tableweak == FALSE) as BOOL);
}

/// is an ifstub table marshaled?
#[no_mangle]
pub unsafe extern "C" fn stub_manager_is_table_marshaled(
    m: *mut stub_manager,
    ipid: *const IPID,
) -> BOOL {
    let ifstub_p = stub_manager_ipid_to_ifstub(m, ipid);

    c_assert!(!ifstub_p.is_null());

    ((*ifstub_p).flags & (MSHLFLAGS_TABLESTRONG | MSHLFLAGS_TABLEWEAK) != 0) as BOOL
}

/*****************************************************************************
 *
 * IRemUnknown implementation
 *
 *
 * Note: this object is not related to the lifetime of a stub_manager, but it
 * interacts with stub managers.
 */

#[repr(C)]
struct RemUnknown {
    IRemUnknown_iface: IRemUnknown,
    refs: LONG,
}

#[inline]
unsafe fn impl_from_IRemUnknown(iface: *mut IRemUnknown) -> *mut RemUnknown {
    containing_record!(iface, RemUnknown, IRemUnknown_iface)
}

/// construct an IRemUnknown object with one outstanding reference
unsafe fn RemUnknown_Construct(ppRemUnknown: *mut *mut IRemUnknown) -> HRESULT {
    let This =
        HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<RemUnknown>()) as *mut RemUnknown;

    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).IRemUnknown_iface.lpVtbl = &RemUnknown_Vtbl;
    (*This).refs = 1;

    *ppRemUnknown = core::ptr::addr_of_mut!((*This).IRemUnknown_iface);
    S_OK
}

unsafe extern "system" fn RemUnknown_QueryInterface(
    iface: *mut IRemUnknown,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    TRACE!("(%p)->(%s, %p)\n");

    if IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IRemUnknown) {
        *ppv = iface as *mut c_void;
        iface.AddRef();
        return S_OK;
    }

    if !IsEqualIID(riid, &IID_IExternalConnection) {
        FIXME!("No interface for iid %s\n");
    }

    *ppv = core::ptr::null_mut();
    E_NOINTERFACE
}

unsafe extern "system" fn RemUnknown_AddRef(iface: *mut IRemUnknown) -> ULONG {
    let This = impl_from_IRemUnknown(iface);
    let refs = InterlockedIncrement(core::ptr::addr_of_mut!((*This).refs)) as ULONG;
    TRACE!("%p before: %d\n");
    refs
}

unsafe extern "system" fn RemUnknown_Release(iface: *mut IRemUnknown) -> ULONG {
    let This = impl_from_IRemUnknown(iface);
    let refs = InterlockedDecrement(core::ptr::addr_of_mut!((*This).refs)) as ULONG;
    if refs == 0 {
        HeapFree(GetProcessHeap(), 0, This as PVOID);
    }

    TRACE!("%p after: %d\n");
    refs
}

unsafe extern "system" fn RemUnknown_RemQueryInterface(
    iface: *mut IRemUnknown,
    ripid: REFIPID,
    cRefs: ULONG,
    cIids: USHORT,
    iids: *mut IID, /* [size_is(cIids)] */
    ppQIResults: *mut *mut REMQIRESULT, /* [size_is(,cIids)] */
) -> HRESULT {
    let mut successful_qis: USHORT = 0;
    let mut apt: *mut apartment = core::ptr::null_mut();
    let mut stubmgr: *mut stub_manager = core::ptr::null_mut();
    let mut ifstub_p: *mut ifstub = core::ptr::null_mut();
    let mut dest_context: DWORD = 0;
    let mut dest_context_data: *mut c_void = core::ptr::null_mut();

    TRACE!("(%p)->(%s, %d, %d, %p, %p)\n");

    let hr = ipid_to_ifstub(ripid, &mut apt, &mut stubmgr, &mut ifstub_p);
    if hr != S_OK {
        return hr;
    }

    (*ifstub_p)
        .chan
        .GetDestCtx(&mut dest_context, &mut dest_context_data);

    *ppQIResults = CoTaskMemAlloc(core::mem::size_of::<REMQIRESULT>() * cIids as usize)
        as *mut REMQIRESULT;

    for i in 0..cIids as usize {
        let hrobj = marshal_object(
            apt,
            core::ptr::addr_of_mut!((*(*ppQIResults).add(i)).std),
            iids.add(i),
            (*stubmgr).object,
            dest_context,
            dest_context_data,
            MSHLFLAGS_NORMAL,
        );
        if hrobj == S_OK {
            successful_qis += 1;
        }
        (*(*ppQIResults).add(i)).hResult = hrobj;
    }

    stub_manager_int_release(stubmgr);
    apartment_release(apt);

    if successful_qis == cIids {
        S_OK /* we got all requested interfaces */
    } else if successful_qis == 0 {
        E_NOINTERFACE /* we didn't get any interfaces */
    } else {
        S_FALSE /* we got some interfaces */
    }
}

unsafe extern "system" fn RemUnknown_RemAddRef(
    iface: *mut IRemUnknown,
    cInterfaceRefs: USHORT,
    InterfaceRefs: *mut REMINTERFACEREF, /* [size_is(cInterfaceRefs)] */
    pResults: *mut HRESULT,              /* [size_is(cInterfaceRefs)] */
) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    TRACE!("(%p)->(%d, %p, %p)\n");

    for i in 0..cInterfaceRefs as usize {
        let mut apt: *mut apartment = core::ptr::null_mut();
        let mut stubmgr: *mut stub_manager = core::ptr::null_mut();

        *pResults.add(i) = ipid_to_stub_manager(
            core::ptr::addr_of!((*InterfaceRefs.add(i)).ipid),
            &mut apt,
            &mut stubmgr,
        );
        if *pResults.add(i) != S_OK {
            hr = S_FALSE;
            continue;
        }

        stub_manager_ext_addref(stubmgr, (*InterfaceRefs.add(i)).cPublicRefs, FALSE);
        if (*InterfaceRefs.add(i)).cPrivateRefs != 0 {
            FIXME!("Adding %d refs securely not implemented\n");
        }

        stub_manager_int_release(stubmgr);
        apartment_release(apt);
    }

    hr
}

unsafe extern "system" fn RemUnknown_RemRelease(
    iface: *mut IRemUnknown,
    cInterfaceRefs: USHORT,
    InterfaceRefs: *mut REMINTERFACEREF, /* [size_is(cInterfaceRefs)] */
) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    TRACE!("(%p)->(%d, %p)\n");

    for i in 0..cInterfaceRefs as usize {
        let mut apt: *mut apartment = core::ptr::null_mut();
        let mut stubmgr: *mut stub_manager = core::ptr::null_mut();

        hr = ipid_to_stub_manager(
            core::ptr::addr_of!((*InterfaceRefs.add(i)).ipid),
            &mut apt,
            &mut stubmgr,
        );
        if hr != S_OK {
            hr = E_INVALIDARG;
            /* FIXME: we should undo any changes already made in this function */
            break;
        }

        stub_manager_ext_release(stubmgr, (*InterfaceRefs.add(i)).cPublicRefs, FALSE, TRUE);
        if (*InterfaceRefs.add(i)).cPrivateRefs != 0 {
            FIXME!("Releasing %d refs securely not implemented\n");
        }

        stub_manager_int_release(stubmgr);
        apartment_release(apt);
    }

    hr
}

static RemUnknown_Vtbl: IRemUnknownVtbl = IRemUnknownVtbl {
    QueryInterface: RemUnknown_QueryInterface,
    AddRef: RemUnknown_AddRef,
    Release: RemUnknown_Release,
    RemQueryInterface: RemUnknown_RemQueryInterface,
    RemAddRef: RemUnknown_RemAddRef,
    RemRelease: RemUnknown_RemRelease,
};

/// starts the IRemUnknown listener for the current apartment
#[no_mangle]
pub unsafe extern "C" fn start_apartment_remote_unknown(apt: *mut apartment) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*apt).cs));
    if (*apt).remunk_exported == FALSE {
        /* create the IRemUnknown object */
        let mut pRemUnknown: *mut IRemUnknown = core::ptr::null_mut();
        hr = RemUnknown_Construct(&mut pRemUnknown);
        if hr == S_OK {
            let mut stdobjref: STDOBJREF = core::mem::zeroed(); /* dummy - not used */
            /* register it with the stub manager */
            hr = marshal_object(
                apt,
                &mut stdobjref,
                &IID_IRemUnknown,
                pRemUnknown as *mut IUnknown,
                MSHCTX_DIFFERENTMACHINE,
                core::ptr::null_mut(),
                MSHLFLAGS_NORMAL | MSHLFLAGSP_REMUNKNOWN,
            );
            /* release our reference to the object as the stub manager will
             * manage the life cycle for us */
            pRemUnknown.Release();
            if hr == S_OK {
                (*apt).remunk_exported = TRUE;
            }
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*apt).cs));
    hr
}
