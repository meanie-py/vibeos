use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub Flags: DWORD,
    pub CreatorBackTraceIndexHigh: WORD,
    pub SpareWORD: WORD,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub WineDebugString: DWORD_PTR,
    pub Spare: [DWORD_PTR; 1],
    pub c2rust_unnamed: C2RustUnnamed_0,
}

static mut registered_ept_entry_list: list = unsafe {
    list {
        next: &raw const registered_ept_entry_list as *mut list,
        prev: &raw const registered_ept_entry_list as *mut list,
    }
};

static mut critsect_debug: CRITICAL_SECTION_DEBUG = _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut csEpm,
        ProcessLocksList: _LIST_ENTRY {
            Flink: unsafe { &raw mut critsect_debug.ProcessLocksList },
            Blink: unsafe { &raw mut critsect_debug.ProcessLocksList },
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        // The C's `(DWORD_PTR)(__FILE__ ": cs")` debugger label is zeroed: a pointer cannot become
        // an integer in a Rust static initializer (msvcrt's exit.rs takes the same divergence).
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    };

static mut csEpm: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const critsect_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as ::core::ffi::c_int) as LONG,
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

static mut nil_object: UUID = _GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

// WINE_DEFAULT_DEBUG_CHANNEL(ole): debug channel name, emitted as a constant.
pub const WINE_DEFAULT_DEBUG_CHANNEL_NAME: &str = "ole";

// Not defined in the preamble; value assumed from the Wine source.

#[repr(C)]
pub struct registered_ept_entry {
    pub entry: list,
    pub object: GUID,
    pub iface: RPC_SYNTAX_IDENTIFIER,
    pub syntax: RPC_SYNTAX_IDENTIFIER,
    pub protseq: *mut i8,
    pub endpoint: *mut i8,
    pub address: *mut i8,
    pub annotation: [i8; (ept_max_annotation_size as _)],
}

// LIST_INIT(registered_ept_entry_list): empty circular list pointing at itself.

// The __FILE__ ": csEpm" debug string pointer cast to DWORD_PTR cannot be
// evaluated in a const context; Spare[0] is left at 0 (debug-only field).

unsafe extern "C" fn delete_registered_ept_entry(entry: *mut registered_ept_entry) {
    I_RpcFree((*entry).protseq as *mut ::core::ffi::c_void);
    I_RpcFree((*entry).endpoint as *mut ::core::ffi::c_void);
    I_RpcFree((*entry).address as *mut ::core::ffi::c_void);
    list_remove(core::ptr::addr_of_mut!((*entry).entry));
    free(entry as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn find_ept_entry(
    iface: *const RPC_SYNTAX_IDENTIFIER,
    syntax: *const RPC_SYNTAX_IDENTIFIER,
    protseq: *const core::ffi::c_char,
    endpoint: *const core::ffi::c_char,
    address: *const core::ffi::c_char,
    object: *const UUID,
) -> *mut registered_ept_entry {
    let mut entry: *mut registered_ept_entry = core::ptr::null_mut::<registered_ept_entry>();
    entry = (registered_ept_entry_list.next as *mut core::ffi::c_char).offset(
        -(core::mem::offset_of!(registered_ept_entry, entry) as isize),
    ) as *mut registered_ept_entry;
    while core::ptr::addr_of_mut!((*entry).entry) != core::ptr::addr_of_mut!(registered_ept_entry_list) {
        if !(memcmp(
            core::ptr::addr_of_mut!((*entry).iface) as *const core::ffi::c_void,
            iface as *const core::ffi::c_void,
            core::mem::size_of::<RPC_SYNTAX_IDENTIFIER>() as usize,
        ) != 0)
        {
            if !(memcmp(
                core::ptr::addr_of_mut!((*entry).syntax) as *const core::ffi::c_void,
                syntax as *const core::ffi::c_void,
                core::mem::size_of::<RPC_SYNTAX_IDENTIFIER>() as usize,
            ) != 0)
            {
                if !(strcmp((*entry).protseq, protseq) != 0) {
                    if !(memcmp(
                        core::ptr::addr_of_mut!((*entry).object) as *const core::ffi::c_void,
                        object as *const core::ffi::c_void,
                        core::mem::size_of::<UUID>() as usize,
                    ) != 0)
                    {
                        WINE_TRACE!(
                            b"found entry with iface %d.%d %s, syntax %d.%d %s, protseq %s, object %s\n\0"
                                .as_ptr() as *const core::ffi::c_char,
                            (*entry).iface.SyntaxVersion.MajorVersion as core::ffi::c_int,
                            (*entry).iface.SyntaxVersion.MinorVersion as core::ffi::c_int,
                            wine_dbgstr_guid(core::ptr::addr_of_mut!((*entry).iface.SyntaxGUID)),
                            (*entry).syntax.SyntaxVersion.MajorVersion as core::ffi::c_int,
                            (*entry).syntax.SyntaxVersion.MinorVersion as core::ffi::c_int,
                            wine_dbgstr_guid(core::ptr::addr_of_mut!((*entry).syntax.SyntaxGUID)),
                            protseq,
                            wine_dbgstr_guid(core::ptr::addr_of_mut!((*entry).object)),
                        );
                        return entry;
                    }
                }
            }
        }
        entry = ((*entry).entry.next as *mut core::ffi::c_char).offset(
            -(core::mem::offset_of!(registered_ept_entry, entry) as isize),
        ) as *mut registered_ept_entry;
    }
    WINE_TRACE!(b"not found\n\0".as_ptr() as *const core::ffi::c_char);
    return core::ptr::null_mut::<registered_ept_entry>();
}

#[no_mangle]
pub unsafe extern "system" fn ept_lookup_handle_t_rundown(entry_handle: ept_lookup_handle_t) {
    // WINE_FIXME("%p\n", entry_handle);
}

#[no_mangle]
pub unsafe extern "C" fn ept_insert(
    h: handle_t,
    num_ents: unsigned32,
    entries: *mut ept_entry_t,
    replace: boolean32,
    status: *mut error_status_t,
) {
    let mut i: unsigned32;
    let mut rpc_status: RPC_STATUS;

    WINE_TRACE!(b"(%p, %lu, %p, %lu, %p)\n\0".as_ptr() as *const i8, h, num_ents, entries, replace, status);

    *status = RPC_S_OK as error_status_t;

    EnterCriticalSection(core::ptr::addr_of_mut!(csEpm));

    i = 0 as unsigned32;
    while i < num_ents {
        let entry: *mut registered_ept_entry =
            malloc(core::mem::size_of::<registered_ept_entry>() as size_t) as *mut registered_ept_entry;
        if entry.is_null() {
            /* FIXME: cleanup code to delete added entries */
            *status = EPT_S_CANT_PERFORM_OP as error_status_t;
            break;
        }
        memcpy(
            core::ptr::addr_of_mut!((*entry).annotation) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*entries.add(i as usize)).annotation) as *const core::ffi::c_void,
            core::mem::size_of::<[core::ffi::c_char; 64]>() as size_t,
        );
        rpc_status = TowerExplode(
            (*entries.add(i as usize)).tower,
            core::ptr::addr_of_mut!((*entry).iface),
            core::ptr::addr_of_mut!((*entry).syntax),
            core::ptr::addr_of_mut!((*entry).protseq),
            core::ptr::addr_of_mut!((*entry).endpoint),
            core::ptr::addr_of_mut!((*entry).address),
        );
        if rpc_status != RPC_S_OK as RPC_STATUS {
            WINE_WARN!(b"TowerExplode failed %lu\n\0".as_ptr() as *const i8, rpc_status);
            *status = rpc_status as error_status_t;
            free(entry as *mut core::ffi::c_void);
            break; /* FIXME: more cleanup? */
        }

        (*entry).object = (*entries.add(i as usize)).object;

        if replace != 0 {
            /* FIXME: correct find algorithm */
            let old_entry: *mut registered_ept_entry = find_ept_entry(
                core::ptr::addr_of_mut!((*entry).iface),
                core::ptr::addr_of_mut!((*entry).syntax),
                (*entry).protseq,
                (*entry).endpoint,
                (*entry).address,
                core::ptr::addr_of_mut!((*entry).object),
            );
            if !old_entry.is_null() {
                delete_registered_ept_entry(old_entry);
            }
        }
        list_add_tail(
            core::ptr::addr_of_mut!(registered_ept_entry_list),
            core::ptr::addr_of_mut!((*entry).entry),
        );
        i = i.wrapping_add(1);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(csEpm));
}
#[no_mangle]
pub unsafe extern "C" fn ept_delete(
    h: handle_t,
    num_ents: unsigned32,
    entries: *mut ept_entry_t,
    status: *mut error_status_t,
) {
    let mut i: unsigned32;
    let mut rpc_status: RPC_STATUS;

    *status = (RPC_S_OK as u32);

    WINE_TRACE!("(%p, %lu, %p, %p)\n", h, num_ents, entries, status);

    EnterCriticalSection(core::ptr::addr_of_mut!(csEpm));

    i = 0;
    while i < num_ents {
        let mut entry: *mut registered_ept_entry;
        let mut iface: RPC_SYNTAX_IDENTIFIER = core::mem::zeroed();
        let mut syntax: RPC_SYNTAX_IDENTIFIER = core::mem::zeroed();
        let mut protseq: *mut core::ffi::c_char = core::ptr::null_mut();
        let mut endpoint: *mut core::ffi::c_char = core::ptr::null_mut();
        let mut address: *mut core::ffi::c_char = core::ptr::null_mut();

        rpc_status = TowerExplode(
            (*entries.offset(i as isize)).tower,
            core::ptr::addr_of_mut!(iface),
            core::ptr::addr_of_mut!(syntax),
            core::ptr::addr_of_mut!(protseq),
            core::ptr::addr_of_mut!(endpoint),
            core::ptr::addr_of_mut!(address),
        );
        if rpc_status != RPC_S_OK {
            break;
        }
        entry = find_ept_entry(
            core::ptr::addr_of_mut!(iface),
            core::ptr::addr_of_mut!(syntax),
            protseq,
            endpoint,
            address,
            core::ptr::addr_of_mut!((*entries.offset(i as isize)).object),
        );

        I_RpcFree(protseq as *mut core::ffi::c_void);
        I_RpcFree(endpoint as *mut core::ffi::c_void);
        I_RpcFree(address as *mut core::ffi::c_void);

        if !entry.is_null() {
            delete_registered_ept_entry(entry);
            i = i.wrapping_add(1);
        } else {
            *status = EPT_S_NOT_REGISTERED;
            break;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(csEpm));
}

#[no_mangle]
pub unsafe extern "C" fn ept_lookup(
    h: handle_t,
    inquiry_type: unsigned32,
    object: uuid_p_t,
    interface_id: rpc_if_id_p_t,
    vers_option: unsigned32,
    entry_handle: *mut ept_lookup_handle_t,
    max_ents: unsigned32,
    num_ents: *mut unsigned32,
    entries: *mut ept_entry_t,
    status: *mut error_status_t,
) {
    WINE_FIXME!(
        b"(%p, %p, %p): stub\n\0".as_ptr() as *const i8,
        h,
        entry_handle,
        status,
    );

    *status = EPT_S_CANT_PERFORM_OP;
}
#[no_mangle]
pub unsafe extern "C" fn ept_map(
    h: handle_t,
    object: uuid_p_t,
    map_tower: twr_p_t,
    entry_handle: *mut ept_lookup_handle_t,
    max_towers: unsigned32,
    num_towers: *mut unsigned32,
    towers: *mut twr_p_t,
    status: *mut error_status_t,
) {
    let mut rpc_status: RPC_STATUS;
    let mut iface: RPC_SYNTAX_IDENTIFIER = core::mem::zeroed();
    let mut syntax: RPC_SYNTAX_IDENTIFIER = core::mem::zeroed();
    let mut protseq: *mut core::ffi::c_char = core::ptr::null_mut();
    let mut entry: *mut registered_ept_entry;

    *status = RPC_S_OK as error_status_t;
    *num_towers = 0 as unsigned32;

    WINE_TRACE!(
        b"(%p, %p, %p, %p, %lu, %p, %p, %p)\n\0".as_ptr() as *const core::ffi::c_char,
        h,
        object,
        map_tower,
        entry_handle,
        max_towers,
        num_towers,
        towers,
        status,
    );

    rpc_status = TowerExplode(
        map_tower,
        core::ptr::addr_of_mut!(iface),
        core::ptr::addr_of_mut!(syntax),
        core::ptr::addr_of_mut!(protseq),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if rpc_status != RPC_S_OK as RPC_STATUS {
        *status = rpc_status as error_status_t;
        return;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!(csEpm));

    // LIST_FOR_EACH_ENTRY(entry, &registered_ept_entry_list, struct registered_ept_entry, entry)
    entry = (registered_ept_entry_list.next as *mut core::ffi::c_char).offset(
        -(core::mem::offset_of!(registered_ept_entry, entry) as isize),
    ) as *mut registered_ept_entry;
    while core::ptr::addr_of_mut!((*entry).entry) != core::ptr::addr_of_mut!(registered_ept_entry_list) {
        if IsEqualGUID!(
            core::ptr::addr_of_mut!((*entry).iface.SyntaxGUID),
            core::ptr::addr_of_mut!(iface.SyntaxGUID),
        ) != false
            && (*entry).iface.SyntaxVersion.MajorVersion == iface.SyntaxVersion.MajorVersion
            && (*entry).iface.SyntaxVersion.MinorVersion >= iface.SyntaxVersion.MinorVersion
            && memcmp(
                core::ptr::addr_of_mut!((*entry).syntax) as *const core::ffi::c_void,
                core::ptr::addr_of_mut!(syntax) as *const core::ffi::c_void,
                core::mem::size_of::<RPC_SYNTAX_IDENTIFIER>() as size_t,
            ) == 0
            && strcmp((*entry).protseq, protseq) == 0
            && ((object.is_null()
                && IsEqualGUID!(
                    core::ptr::addr_of_mut!((*entry).object),
                    core::ptr::addr_of!(nil_object) as *mut GUID,
                ) != false)
                || IsEqualGUID!(object, core::ptr::addr_of_mut!((*entry).object)) != false)
        {
            if *num_towers < max_towers {
                rpc_status = TowerConstruct(
                    core::ptr::addr_of_mut!((*entry).iface),
                    core::ptr::addr_of_mut!((*entry).syntax),
                    (*entry).protseq,
                    (*entry).endpoint,
                    (*entry).address,
                    towers.offset(*num_towers as isize),
                );
                if rpc_status != RPC_S_OK as RPC_STATUS {
                    *status = rpc_status as error_status_t;
                    break;
                }
            }
            *num_towers = (*num_towers).wrapping_add(1);
        }
        entry = ((*entry).entry.next as *mut core::ffi::c_char).offset(
            -(core::mem::offset_of!(registered_ept_entry, entry) as isize),
        ) as *mut registered_ept_entry;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(csEpm));

    I_RpcFree(protseq as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn ept_lookup_handle_free(
    h: handle_t,
    entry_handle: *mut ept_lookup_handle_t,
    status: *mut error_status_t,
) {
    WINE_FIXME!(
        b"(%p, %p, %p): stub\n\0".as_ptr() as *const i8,
        h,
        entry_handle,
        status,
    );

    *status = EPT_S_CANT_PERFORM_OP as error_status_t;
}
#[no_mangle]
pub unsafe extern "C" fn ept_inq_object(h: handle_t, ept_object: *mut GUID, status: *mut error_status_t) {
    WINE_FIXME!("(%p, %p, %p): stub\n", h, ept_object, status);

    *status = EPT_S_CANT_PERFORM_OP;
}

#[no_mangle]
pub unsafe extern "C" fn ept_mgmt_delete(
    h: handle_t,
    object_speced: boolean32,
    object: uuid_p_t,
    tower: twr_p_t,
    status: *mut error_status_t,
) {
    WINE_FIXME!("(%p, %ld, %p, %p, %p): stub\n", h, object_speced, object, tower, status);

    *status = EPT_S_CANT_PERFORM_OP as error_status_t;
}
