use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut SpInfExports: SPINF_EXPORTS = _SPINF_EXPORTS {
    SpInfCloseInfFile: None,
    SpInfFindFirstLine: None,
    SpInfFindNextLine: None,
    SpInfGetFieldCount: None,
    SpInfGetBinaryField: None,
    SpInfGetIntField: None,
    SpInfGetMultiSzField: None,
    SpInfGetStringField: None,
    SpInfGetField: None,
    SpInfOpenInfFile: None,
};

#[no_mangle]
pub unsafe extern "C" fn INF_GetDataField(
    Context: PINFCONTEXT,
    FieldIndex: ULONG,
    Data: *mut PCWSTR,
) -> BOOLEAN {
    *Data = SpInfGetField!(Context, FieldIndex);
    return !(*Data).is_null() as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn INF_GetData(
    Context: PINFCONTEXT,
    Key: *mut PCWSTR,
    Data: *mut PCWSTR,
) -> BOOLEAN {
    let mut Success: BOOL;
    let mut InfData: [PCWSTR; 2] = [core::ptr::null_mut(), core::ptr::null_mut()];

    if !Key.is_null() {
        *Key = core::ptr::null_mut();
    }

    if !Data.is_null() {
        *Data = core::ptr::null_mut();
    }

    if SpInfGetFieldCount!(Context) != 1 {
        DPRINT1!("SpInfGetFieldCount != 1\n");
        return (FALSE) as u8;
    }

    if !Key.is_null() {
        Success = (INF_GetDataField(
            Context,
            0,
            &mut InfData[0],
        )) as _;
        if Success == 0 {
            return (FALSE) as u8;
        }
    }

    if !Data.is_null() {
        Success = (INF_GetDataField(
            Context,
            1,
            &mut InfData[1],
        )) as _;
        if Success == 0 {
            INF_FreeData(InfData[0]);
            return (FALSE) as u8;
        }
    }

    if !Key.is_null() {
        *Key = InfData[0];
    }

    if !Data.is_null() {
        *Data = InfData[1];
    }

    return (TRUE) as u8;
}
