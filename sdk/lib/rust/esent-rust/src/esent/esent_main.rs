use crate::prelude::*;
use core::ffi::c_void;

/// `struct _JET_RETINFO;` -- only forward-declared by esent_main.c, which never touches
/// its fields: an opaque type for the pointer parameter.
#[repr(C)]
pub struct _JET_RETINFO {
    _opaque: [u8; 0],
}

// ---- file-scope preamble (translated) ----
pub type JET_ERR = i32;
pub type JET_GRBIT = u32;
pub type JET_DBID = u32;
pub type JET_COLUMNID = u32;
pub type JET_API_PTR = usize;
pub type JET_INSTANCE = JET_API_PTR;
pub type JET_SESID = JET_API_PTR;
pub type JET_TABLEID = JET_API_PTR;
pub type JET_PCWSTR = *const u16;

// struct _JET_RETINFO: body not present in the preamble excerpt; definition omitted.

pub const JET_errSuccess: JET_ERR = 0;
pub const JET_errFeatureNotAvailable: JET_ERR = -1001;
pub const JET_errInvalidParameter: JET_ERR = -1003;

unsafe extern "system" fn jet_not_available() -> JET_ERR {
    return JET_errFeatureNotAvailable;
}
#[no_mangle]
pub unsafe extern "system" fn JetAttachDatabase2W(
    sesid: JET_SESID,
    szFilename: *const WCHAR,
    cpgDatabaseSizeMax: core::ffi::c_ulong,
    grbit: JET_GRBIT,
) -> JET_ERR {
    return jet_not_available();
}
#[no_mangle]
pub unsafe extern "system" fn JetBeginSessionW(instance: JET_INSTANCE, psesid: *mut JET_SESID, szUserName: JET_PCWSTR, szPassword: JET_PCWSTR) -> JET_ERR {
    if !psesid.is_null() {
        *psesid = 0;
    }
    return jet_not_available();
}
#[no_mangle]
pub unsafe extern "system" fn JetCloseTable(sesid: JET_SESID, tableid: JET_TABLEID) -> JET_ERR {
    let _ = sesid;
    let _ = tableid;
    return JET_errInvalidParameter as JET_ERR;
}
#[no_mangle]
pub unsafe extern "system" fn JetCreateInstanceW(
    pinstance: *mut JET_INSTANCE,
    szInstanceName: *const WCHAR,
) -> JET_ERR {
    if !pinstance.is_null() {
        *pinstance = 0 as JET_INSTANCE;
    }
    return jet_not_available();
}
#[no_mangle]
pub unsafe extern "system" fn JetGetTableColumnInfoW(
    sesid: JET_SESID,
    tableid: JET_TABLEID,
    szColumnName: *const WCHAR,
    pvResult: *mut core::ffi::c_void,
    cbMax: core::ffi::c_ulong,
    InfoLevel: core::ffi::c_ulong,
) -> JET_ERR {
    if !pvResult.is_null() && cbMax != 0 {
        memset(pvResult, 0 as core::ffi::c_int, cbMax as usize);
    }
    return jet_not_available();
}
#[no_mangle]
pub unsafe extern "system" fn JetInit(pinstance: *mut JET_INSTANCE) -> JET_ERR {
    if !pinstance.is_null() {
        *pinstance = 0 as JET_INSTANCE;
    }
    return jet_not_available();
}
#[no_mangle]
pub unsafe extern "system" fn JetMove(
    sesid: JET_SESID,
    tableid: JET_TABLEID,
    cRow: core::ffi::c_long,
    grbit: JET_GRBIT,
) -> JET_ERR {
    return jet_not_available();
}
#[no_mangle]
pub unsafe extern "system" fn JetOpenDatabaseW(sesid: JET_SESID, szFilename: *const WCHAR, szConnect: *const WCHAR, pdbid: *mut JET_DBID, grbit: JET_GRBIT) -> JET_ERR {
    if !pdbid.is_null() {
        *pdbid = 0;
    }
    return jet_not_available();
}
#[no_mangle]
pub unsafe extern "system" fn JetOpenTableW(sesid: JET_SESID, dbid: JET_DBID, szTableName: *const WCHAR, pvParameters: *const core::ffi::c_void, cbParameters: core::ffi::c_ulong, grbit: JET_GRBIT, ptableid: *mut JET_TABLEID) -> JET_ERR {
    if !ptableid.is_null() {
        *ptableid = 0;
    }
    return jet_not_available();
}
#[no_mangle]
pub unsafe extern "system" fn JetRetrieveColumn(
    mut sesid: JET_SESID,
    mut tableid: JET_TABLEID,
    mut columnid: JET_COLUMNID,
    mut pvData: *mut ::core::ffi::c_void,
    mut cbData: ::core::ffi::c_ulong,
    mut pcbActual: *mut ::core::ffi::c_ulong,
    mut grbit: JET_GRBIT,
    mut pretinfo: *mut _JET_RETINFO,
) -> JET_ERR {
    if !pcbActual.is_null() {
        *pcbActual = 0 as ::core::ffi::c_ulong;
    }
    return jet_not_available();
}
#[no_mangle]
pub unsafe extern "system" fn JetSetSystemParameterW(
    pinstance: *mut JET_INSTANCE,
    sesid: JET_SESID,
    paramid: ::core::ffi::c_ulong,
    lParam: JET_API_PTR,
    szParam: JET_PCWSTR,
) -> JET_ERR {
    return jet_not_available();
}
#[no_mangle]
pub unsafe extern "system" fn JetTerm(instance: JET_INSTANCE) -> JET_ERR {
    let _ = instance;
    return JET_errInvalidParameter as JET_ERR;
}
