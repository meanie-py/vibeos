use crate::crypt32::main::CryptMemAlloc;
use crate::crypt32::main::CryptMemFree;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(crypt): debug channel declaration; handled by the DPRINT1!/ERR!/WARN! macros.

#[repr(C)]
pub struct _CONTEXT_PROPERTY {
    pub propID: DWORD,
    pub cbData: DWORD,
    pub pbData: LPBYTE,
    pub entry: list,
}

pub type CONTEXT_PROPERTY = _CONTEXT_PROPERTY;
#[no_mangle]
pub unsafe extern "C" fn ContextPropertyList_Create() -> *mut CONTEXT_PROPERTY_LIST {
    let mut list: *mut CONTEXT_PROPERTY_LIST = CryptMemAlloc(core::mem::size_of::<CONTEXT_PROPERTY_LIST>() as ULONG) as *mut CONTEXT_PROPERTY_LIST;
    if !list.is_null() {
        InitializeCriticalSectionEx(
            core::ptr::addr_of_mut!((*list).cs),
            0 as DWORD,
            RTL_CRITICAL_SECTION_FLAG_FORCE_DEBUG_INFO as DWORD,
        );
        (*(*list).cs.DebugInfo).__bindgen_anon_1.Spare[0 as usize] = concat!(file!(), "\0").as_ptr() as *const u8 as *const ::core::ffi::c_char as DWORD_PTR;
        list_init(core::ptr::addr_of_mut!((*list).properties));
    }
    return list;
}
#[no_mangle]
pub unsafe extern "C" fn ContextPropertyList_Free(list: *mut CONTEXT_PROPERTY_LIST) {
    let mut prop: *mut CONTEXT_PROPERTY = core::ptr::null_mut();
    let mut next: *mut CONTEXT_PROPERTY = core::ptr::null_mut();
    prop = ((*list).properties.next as *mut u8)
        .sub(core::mem::offset_of!(CONTEXT_PROPERTY, entry)) as *mut CONTEXT_PROPERTY;
    next = ((*prop).entry.next as *mut u8)
        .sub(core::mem::offset_of!(CONTEXT_PROPERTY, entry)) as *mut CONTEXT_PROPERTY;
    while core::ptr::addr_of_mut!((*prop).entry) != core::ptr::addr_of_mut!((*list).properties) {
        list_remove(core::ptr::addr_of_mut!((*prop).entry));
        CryptMemFree((*prop).pbData as LPVOID);
        CryptMemFree(prop as LPVOID);
        prop = next;
        next = ((*prop).entry.next as *mut u8)
            .sub(core::mem::offset_of!(CONTEXT_PROPERTY, entry)) as *mut CONTEXT_PROPERTY;
    }
    (*(*list).cs.DebugInfo).__bindgen_anon_1.Spare[0 as usize] = 0 as DWORD_PTR;
    DeleteCriticalSection(core::ptr::addr_of_mut!((*list).cs));
    CryptMemFree(list as LPVOID);
}
#[no_mangle]
pub unsafe extern "C" fn ContextPropertyList_FindProperty(list: *mut CONTEXT_PROPERTY_LIST, id: DWORD, blob: PCRYPT_DATA_BLOB) -> BOOL {
    let _ = core::mem::size_of::<CONTEXT_PROPERTY>();
    let mut prop: *mut CONTEXT_PROPERTY = core::ptr::null_mut::<CONTEXT_PROPERTY>();
    let mut ret: BOOL = (FALSE as i32);

    EnterCriticalSection(core::ptr::addr_of_mut!((*list).cs));
    prop = ((*list).properties.next as *mut u8)
        .offset(-(core::mem::offset_of!(CONTEXT_PROPERTY, entry) as isize))
        as *mut CONTEXT_PROPERTY;
    while core::ptr::addr_of_mut!((*prop).entry) != core::ptr::addr_of_mut!((*list).properties) {
        if (*prop).propID == id {
            (*blob).cbData = (*prop).cbData;
            (*blob).pbData = (*prop).pbData;
            ret = (TRUE as i32);
            break;
        } else {
            prop = ((*prop).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(CONTEXT_PROPERTY, entry) as isize))
                as *mut CONTEXT_PROPERTY;
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*list).cs));
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn ContextPropertyList_SetProperty(list: *mut CONTEXT_PROPERTY_LIST, id: DWORD, pbData: *const BYTE, cbData: size_t) -> BOOL {
    let mut data: LPBYTE = core::ptr::null_mut::<BYTE>();
    let mut ret: BOOL = (FALSE as i32);

    if cbData != 0 {
        data = CryptMemAlloc(cbData as ULONG) as LPBYTE;
        if !data.is_null() {
            memcpy(
                data as *mut core::ffi::c_void,
                pbData as *const core::ffi::c_void,
                cbData,
            );
        }
    } else {
        data = core::ptr::null_mut::<BYTE>();
    }
    if cbData == 0 || !data.is_null() {
        let mut prop: *mut CONTEXT_PROPERTY = core::ptr::null_mut::<CONTEXT_PROPERTY>();
        let mut found: BOOL = (FALSE as i32);

        EnterCriticalSection(&raw mut (*list).cs);
        prop = ((*list).properties.next as *mut core::ffi::c_char).offset(
            -(core::mem::offset_of!(CONTEXT_PROPERTY, entry) as isize),
        ) as *mut CONTEXT_PROPERTY;
        while &raw mut (*prop).entry != &raw mut (*list).properties {
            if (*prop).propID == id {
                found = (TRUE as i32);
                break;
            } else {
                prop = ((*prop).entry.next as *mut core::ffi::c_char).offset(
                    -(core::mem::offset_of!(CONTEXT_PROPERTY, entry) as isize),
                ) as *mut CONTEXT_PROPERTY;
            }
        }
        if found != 0 {
            CryptMemFree((*prop).pbData as LPVOID);
            (*prop).cbData = cbData as DWORD;
            (*prop).pbData = data;
            ret = (TRUE as i32);
        } else {
            prop = CryptMemAlloc(
                core::mem::size_of::<CONTEXT_PROPERTY>() as ULONG,
            ) as *mut CONTEXT_PROPERTY;
            if !prop.is_null() {
                (*prop).propID = id;
                (*prop).cbData = cbData as DWORD;
                (*prop).pbData = data;
                list_add_tail(&raw mut (*list).properties, &raw mut (*prop).entry);
                ret = (TRUE as i32);
            } else {
                CryptMemFree(data as LPVOID);
            }
        }
        LeaveCriticalSection(&raw mut (*list).cs);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn ContextPropertyList_RemoveProperty(list: *mut CONTEXT_PROPERTY_LIST, id: DWORD) {
    let mut prop: *mut CONTEXT_PROPERTY = core::ptr::null_mut();
    EnterCriticalSection(&raw mut (*list).cs);
    prop = ((*list).properties.next as *mut u8).offset(
        -(core::mem::offset_of!(CONTEXT_PROPERTY, entry) as isize),
    ) as *mut CONTEXT_PROPERTY;
    while &raw mut (*prop).entry != &raw mut (*list).properties {
        if (*prop).propID == id {
            list_remove(&raw mut (*prop).entry);
            CryptMemFree((*prop).pbData as LPVOID);
            CryptMemFree(prop as LPVOID);
            break;
        } else {
            prop = ((*prop).entry.next as *mut u8).offset(
                -(core::mem::offset_of!(CONTEXT_PROPERTY, entry) as isize),
            ) as *mut CONTEXT_PROPERTY;
        }
    }
    LeaveCriticalSection(&raw mut (*list).cs);
}
#[no_mangle]
pub unsafe extern "C" fn ContextPropertyList_EnumPropIDs(list: *mut CONTEXT_PROPERTY_LIST, id: DWORD) -> DWORD {
    let mut ret: DWORD = 0;
    EnterCriticalSection(core::ptr::addr_of_mut!((*list).cs));
    if id != 0 {
        let mut cursor: *mut CONTEXT_PROPERTY = core::ptr::null_mut();
        let mut prop: *mut CONTEXT_PROPERTY;
        prop = LIST_ENTRY!((*list).properties.next, CONTEXT_PROPERTY, entry);
        while core::ptr::addr_of_mut!((*prop).entry) != core::ptr::addr_of_mut!((*list).properties) {
            if (*prop).propID == id {
                cursor = prop;
                break;
            }
            prop = LIST_ENTRY!((*prop).entry.next, CONTEXT_PROPERTY, entry);
        }
        if !cursor.is_null() {
            if (*cursor).entry.next != core::ptr::addr_of_mut!((*list).properties) {
                ret = (*LIST_ENTRY!((*cursor).entry.next, CONTEXT_PROPERTY, entry)).propID;
            } else {
                ret = 0;
            }
        } else {
            ret = 0;
        }
    } else if list_empty(core::ptr::addr_of_mut!((*list).properties)) == 0 {
        ret = (*LIST_ENTRY!((*list).properties.next, CONTEXT_PROPERTY, entry)).propID;
    } else {
        ret = 0;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*list).cs));
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn ContextPropertyList_Copy(to: *mut CONTEXT_PROPERTY_LIST, from: *mut CONTEXT_PROPERTY_LIST) {
    let mut prop: *mut CONTEXT_PROPERTY = core::ptr::null_mut();
    EnterCriticalSection(&raw mut (*from).cs);
    prop = ((*from).properties.next as *mut u8).offset(-(core::mem::offset_of!(CONTEXT_PROPERTY, entry) as isize)) as *mut CONTEXT_PROPERTY;
    while &raw mut (*prop).entry != &raw mut (*from).properties {
        ContextPropertyList_SetProperty(to, (*prop).propID, (*prop).pbData as *const u8, (*prop).cbData as usize);
        prop = ((*prop).entry.next as *mut u8).offset(-(core::mem::offset_of!(CONTEXT_PROPERTY, entry) as isize)) as *mut CONTEXT_PROPERTY;
    }
    LeaveCriticalSection(&raw mut (*from).cs);
}
