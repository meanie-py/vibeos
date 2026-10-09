use crate::prelude::*;
use core::ffi::c_void;

// Genuine CRT imports (cdecl). Everything else (SysFreeString, SafeArray*,
// Var* conversions, IRecordInfo_*/IUnknown_* wrappers, HeapAlloc, memcpy, ...)
// comes from the prelude glob / sibling modules of this crate.
extern "C" {
    fn floor(x: f64) -> f64;
    fn ceil(x: f64) -> f64;
    fn fabs(x: f64) -> f64;
    fn iswdigit(c: OLECHAR) -> i32;
    fn iswspace(c: OLECHAR) -> i32;
}

// Precomputed case labels (C: `case VT_X | VT_BYREF:`; VT_BYREF = 0x4000).
const VT_DISPATCH_VT_BYREF: VARTYPE = 0x4009; // VT_DISPATCH | VT_BYREF
const VT_UNKNOWN_VT_BYREF: VARTYPE = 0x400D;  // VT_UNKNOWN | VT_BYREF
const VT_BSTR_VT_BYREF: VARTYPE = 0x4008;     // VT_BSTR | VT_BYREF
const VT_VARIANT_VT_BYREF: VARTYPE = 0x400C;  // VT_VARIANT | VT_BYREF
const VT_RECORD_VT_BYREF: VARTYPE = 0x4024;   // VT_RECORD | VT_BYREF

// ---- file-scope preamble (translated) ----
pub static mut cache_cs: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: unsafe { core::ptr::addr_of_mut!(critsect_debug) },
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};

// C: critsect_debug = { 0, 0, &cache_cs, { &critsect_debug.ProcessLocksList,
// &critsect_debug.ProcessLocksList }, 0, 0, { (DWORD_PTR)(__FILE__ ": cache_cs") } }
// (the debug-name Spare slot is dropped, matching the prelude's mirror layout).
pub static mut critsect_debug: CRITICAL_SECTION_DEBUG = CRITICAL_SECTION_DEBUG {
    Type: 0,
    CreatorBackTraceIndex: 0,
    CriticalSection: unsafe { core::ptr::addr_of_mut!(cache_cs) },
    ProcessLocksList: _LIST_ENTRY {
        Flink: unsafe { core::ptr::addr_of_mut!(critsect_debug.ProcessLocksList) as *mut _LIST_ENTRY },
        Blink: unsafe { core::ptr::addr_of_mut!(critsect_debug.ProcessLocksList) as *mut _LIST_ENTRY },
    },
    EntryCount: 0,
    ContentionCount: 0,
    // The trailing member is a union (ndk/rtltypes.h:1432-1442); zero it via the
    // pointer-sized arm, which covers every alternative.
    c2rust_unnamed: RTL_CRITICAL_SECTION_DEBUG_u { WineDebugString: 0 },
};

pub const B_PROCESSING_EXPONENT: u32 = 0x1;
pub const B_NEGATIVE_EXPONENT: u32 = 0x2;
pub const B_EXPONENT_START: u32 = 0x4;
pub const B_INEXACT_ZEROS: u32 = 0x8;
pub const B_LEADING_ZERO: u32 = 0x10;
pub const B_PROCESSING_HEX: u32 = 0x20;
pub const B_PROCESSING_OCT: u32 = 0x40;

pub const INTEGER_VTBITS: u32 = VTBIT_I1 | VTBIT_UI1 | VTBIT_I2 | VTBIT_UI2 | VTBIT_I4 | VTBIT_UI4 | VTBIT_I8 | VTBIT_UI8;
pub const REAL_VTBITS: u32 = VTBIT_R4 | VTBIT_R8 | VTBIT_CY;

unsafe fn VARIANT_Coerce(
    pd: *mut VARIANTARG,
    lcid: LCID,
    wFlags: USHORT,
    ps: *mut VARIANTARG,
    vt: VARTYPE,
) -> HRESULT {
    let mut res: HRESULT = DISP_E_TYPEMISMATCH;
    let mut vtFrom: VARTYPE = V_TYPE(ps);
    let mut dwFlags: DWORD = 0;

    TRACE!(
        b"(%s,0x%08x,0x%04x,%s,%s)\n\0".as_ptr() as PCHAR,
        debugstr_variant(pd),
        lcid,
        wFlags,
        debugstr_variant(ps),
        debugstr_vt(vt),
    );

    if vt == VT_BSTR || vtFrom == VT_BSTR {
        if wFlags as u32 & VARIANT_LOCALBOOL != 0 {
            dwFlags |= VAR_LOCALBOOL;
        }
        if wFlags as u32 & VARIANT_CALENDAR_HIJRI != 0 {
            dwFlags |= VAR_CALENDAR_HIJRI;
        }
        if wFlags as u32 & VARIANT_CALENDAR_THAI != 0 {
            dwFlags |= VAR_CALENDAR_THAI;
        }
        if wFlags as u32 & VARIANT_CALENDAR_GREGORIAN != 0 {
            dwFlags |= VAR_CALENDAR_GREGORIAN;
        }
        if wFlags as u32 & VARIANT_NOUSEROVERRIDE != 0 {
            dwFlags |= LOCALE_NOUSEROVERRIDE as u32;
        }
        if wFlags as u32 & VARIANT_USE_NLS != 0 {
            dwFlags |= LOCALE_USE_NLS;
        }
    }

    let mut vt = vt;
    if vt == VT_INT {
        vt = VT_I4;
    } else if vt == VT_UINT {
        vt = VT_UI4;
    }

    if vtFrom == VT_INT {
        vtFrom = VT_I4;
    } else if vtFrom == VT_UINT {
        vtFrom = VT_UI4;
    }

    if vt == vtFrom {
        return VariantCopy(pd, ps);
    }

    if wFlags as u32 & VARIANT_NOVALUEPROP != 0 && vtFrom == VT_DISPATCH && vt != VT_UNKNOWN {
        return DISP_E_TYPEMISMATCH;
    }

    match vt {
        VT_EMPTY => {
            if vtFrom == VT_NULL {
                return DISP_E_TYPEMISMATCH;
            }
            if vtFrom <= VT_UINT && vtFrom != 15 && vtFrom != VT_ERROR {
                res = VariantClear(pd);
                if vt == VT_NULL && SUCCEEDED(res) {
                    (*V_VT_ref(pd)) = VT_NULL;
                }
            }
            return res;
        }
        VT_NULL => {
            if vtFrom <= VT_UINT && vtFrom != 15 && vtFrom != VT_ERROR {
                res = VariantClear(pd);
                if vt == VT_NULL && SUCCEEDED(res) {
                    (*V_VT_ref(pd)) = VT_NULL;
                }
            }
            return res;
        }
        VT_I1 => match vtFrom {
            VT_EMPTY => {
                (*V_I1_ref(pd)) = 0;
                return S_OK;
            }
            VT_I2 => return VarI1FromI2(*V_I2_ref(ps) , core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            VT_I4 => return VarI1FromI4(*V_I4_ref(ps) , core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            VT_UI1 => {
                (*V_I1_ref(pd)) = (*V_UI1_ref(ps)) as i8;
                return S_OK;
            }
            VT_UI2 => return VarI1FromUI2(*V_UI2_ref(ps) , core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            VT_UI4 => return VarI1FromUI4(*V_UI4_ref(ps) , core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            VT_I8 => return VarI1FromI8(*V_I8_ref(ps) , core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            VT_UI8 => return VarI1FromUI8(*V_UI8_ref(ps) , core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            VT_R4 => return VarI1FromR4(*V_R4_ref(ps) , core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            VT_R8 => return VarI1FromR8(*V_R8_ref(ps) , core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            VT_DATE => return VarI1FromDate(*V_DATE_ref(ps) , core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            VT_BOOL => return VarI1FromBool(*V_BOOL_ref(ps) , core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            VT_CY => return VarI1FromCy(*V_CY_ref(ps) , core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            VT_DECIMAL => return VarI1FromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            VT_DISPATCH => return VarI1FromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            VT_BSTR => return VarI1FromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_I1_ref(pd)))),
            _ => {}
        },
        VT_I2 => match vtFrom {
            VT_EMPTY => {
                (*V_I2_ref(pd)) = 0;
                return S_OK;
            }
            VT_I1 => return VarI2FromI1(*V_I1_ref(ps) , core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            VT_I4 => return VarI2FromI4(*V_I4_ref(ps) , core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            VT_UI1 => return VarI2FromUI1(*V_UI1_ref(ps) , core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            VT_UI2 => {
                (*V_I2_ref(pd)) = (*V_UI2_ref(ps)) as i16;
                return S_OK;
            }
            VT_UI4 => return VarI2FromUI4(*V_UI4_ref(ps) , core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            VT_I8 => return VarI2FromI8(*V_I8_ref(ps) , core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            VT_UI8 => return VarI2FromUI8(*V_UI8_ref(ps) , core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            VT_R4 => return VarI2FromR4(*V_R4_ref(ps) , core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            VT_R8 => return VarI2FromR8(*V_R8_ref(ps) , core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            VT_DATE => return VarI2FromDate(*V_DATE_ref(ps) , core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            VT_BOOL => return VarI2FromBool(*V_BOOL_ref(ps) , core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            VT_CY => return VarI2FromCy(*V_CY_ref(ps) , core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            VT_DECIMAL => return VarI2FromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            VT_DISPATCH => return VarI2FromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            VT_BSTR => return VarI2FromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_I2_ref(pd)))),
            _ => {}
        },
        VT_I4 => match vtFrom {
            VT_EMPTY => {
                (*V_I4_ref(pd)) = 0;
                return S_OK;
            }
            VT_I1 => return VarI4FromI1(*V_I1_ref(ps) , core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            VT_I2 => return VarI4FromI2(*V_I2_ref(ps) , core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            VT_UI1 => return VarI4FromUI1(*V_UI1_ref(ps) , core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            VT_UI2 => return VarI4FromUI2(*V_UI2_ref(ps) , core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            VT_UI4 => {
                (*V_I4_ref(pd)) = (*V_UI4_ref(ps)) as i32;
                return S_OK;
            }
            VT_I8 => return VarI4FromI8(*V_I8_ref(ps) , core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            VT_UI8 => return VarI4FromUI8(*V_UI8_ref(ps) , core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            VT_R4 => return VarI4FromR4(*V_R4_ref(ps) , core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            VT_R8 => return VarI4FromR8(*V_R8_ref(ps) , core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            VT_DATE => return VarI4FromDate(*V_DATE_ref(ps) , core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            VT_BOOL => return VarI4FromBool(*V_BOOL_ref(ps) , core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            VT_CY => return VarI4FromCy(*V_CY_ref(ps) , core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            VT_DECIMAL => return VarI4FromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            VT_DISPATCH => return VarI4FromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            VT_BSTR => return VarI4FromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_I4_ref(pd)))),
            _ => {}
        },
        VT_UI1 => match vtFrom {
            VT_EMPTY => {
                (*V_UI1_ref(pd)) = 0;
                return S_OK;
            }
            VT_I1 => {
                (*V_UI1_ref(pd)) = (*V_I1_ref(ps)) as u8;
                return S_OK;
            }
            VT_I2 => return VarUI1FromI2(*V_I2_ref(ps) , core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            VT_I4 => return VarUI1FromI4(*V_I4_ref(ps) , core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            VT_UI2 => return VarUI1FromUI2(*V_UI2_ref(ps) , core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            VT_UI4 => return VarUI1FromUI4(*V_UI4_ref(ps) , core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            VT_I8 => return VarUI1FromI8(*V_I8_ref(ps) , core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            VT_UI8 => return VarUI1FromUI8(*V_UI8_ref(ps) , core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            VT_R4 => return VarUI1FromR4(*V_R4_ref(ps) , core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            VT_R8 => return VarUI1FromR8(*V_R8_ref(ps) , core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            VT_DATE => return VarUI1FromDate(*V_DATE_ref(ps) , core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            VT_BOOL => return VarUI1FromBool(*V_BOOL_ref(ps) , core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            VT_CY => return VarUI1FromCy(*V_CY_ref(ps) , core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            VT_DECIMAL => return VarUI1FromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            VT_DISPATCH => return VarUI1FromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            VT_BSTR => return VarUI1FromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_UI1_ref(pd)))),
            _ => {}
        },
        VT_UI2 => match vtFrom {
            VT_EMPTY => {
                (*V_UI2_ref(pd)) = 0;
                return S_OK;
            }
            VT_I1 => return VarUI2FromI1(*V_I1_ref(ps) , core::ptr::addr_of_mut!((*V_UI2_ref(pd)))),
            VT_I2 => {
                (*V_UI2_ref(pd)) = (*V_I2_ref(ps)) as u16;
                return S_OK;
            }
            VT_I4 => return VarUI2FromI4(*V_I4_ref(ps) , core::ptr::addr_of_mut!((*V_UI2_ref(pd)))),
            VT_UI1 => return VarUI2FromUI1(*V_UI1_ref(ps) , core::ptr::addr_of_mut!((*V_UI2_ref(pd)))),
            VT_UI4 => return VarUI2FromUI4(*V_UI4_ref(ps) , core::ptr::addr_of_mut!((*V_UI2_ref(pd)))),
            VT_I8 => return VarUI4FromI8(*V_I8_ref(ps) , core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_UI8 => return VarUI4FromUI8(*V_UI8_ref(ps) , core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_R4 => return VarUI2FromR4(*V_R4_ref(ps) , core::ptr::addr_of_mut!((*V_UI2_ref(pd)))),
            VT_R8 => return VarUI2FromR8(*V_R8_ref(ps) , core::ptr::addr_of_mut!((*V_UI2_ref(pd)))),
            VT_DATE => return VarUI2FromDate(*V_DATE_ref(ps) , core::ptr::addr_of_mut!((*V_UI2_ref(pd)))),
            VT_BOOL => return VarUI2FromBool(*V_BOOL_ref(ps) , core::ptr::addr_of_mut!((*V_UI2_ref(pd)))),
            VT_CY => return VarUI2FromCy(*V_CY_ref(ps) , core::ptr::addr_of_mut!((*V_UI2_ref(pd)))),
            VT_DECIMAL => return VarUI2FromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), core::ptr::addr_of_mut!((*V_UI2_ref(pd)))),
            VT_DISPATCH => return VarUI2FromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_UI2_ref(pd)))),
            VT_BSTR => return VarUI2FromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_UI2_ref(pd)))),
            _ => {}
        },
        VT_UI4 => match vtFrom {
            VT_EMPTY => {
                (*V_UI4_ref(pd)) = 0;
                return S_OK;
            }
            VT_I1 => return VarUI4FromI1(*V_I1_ref(ps) , core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_I2 => return VarUI4FromI2(*V_I2_ref(ps) , core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_I4 => {
                (*V_UI4_ref(pd)) = (*V_I4_ref(ps)) as u32;
                return S_OK;
            }
            VT_UI1 => return VarUI4FromUI1(*V_UI1_ref(ps) , core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_UI2 => return VarUI4FromUI2(*V_UI2_ref(ps) , core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_I8 => return VarUI4FromI8(*V_I8_ref(ps) , core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_UI8 => return VarUI4FromUI8(*V_UI8_ref(ps) , core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_R4 => return VarUI4FromR4(*V_R4_ref(ps) , core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_R8 => return VarUI4FromR8(*V_R8_ref(ps) , core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_DATE => return VarUI4FromDate(*V_DATE_ref(ps) , core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_BOOL => return VarUI4FromBool(*V_BOOL_ref(ps) , core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_CY => return VarUI4FromCy(*V_CY_ref(ps) , core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_DECIMAL => return VarUI4FromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_DISPATCH => return VarUI4FromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            VT_BSTR => return VarUI4FromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_UI4_ref(pd)))),
            _ => {}
        },
        VT_UI8 => match vtFrom {
            VT_EMPTY => {
                (*V_UI8_ref(pd)) = 0;
                return S_OK;
            }
            VT_I4 => {
                if (*V_I4_ref(ps)) < 0 {
                    return DISP_E_OVERFLOW;
                }
                (*V_UI8_ref(pd)) = (*V_I4_ref(ps)) as u64;
                return S_OK;
            }
            VT_I1 => return VarUI8FromI1(*V_I1_ref(ps) , core::ptr::addr_of_mut!((*V_UI8_ref(pd)))),
            VT_I2 => return VarUI8FromI2(*V_I2_ref(ps) , core::ptr::addr_of_mut!((*V_UI8_ref(pd)))),
            VT_UI1 => return VarUI8FromUI1(*V_UI1_ref(ps) , core::ptr::addr_of_mut!((*V_UI8_ref(pd)))),
            VT_UI2 => return VarUI8FromUI2(*V_UI2_ref(ps) , core::ptr::addr_of_mut!((*V_UI8_ref(pd)))),
            VT_UI4 => return VarUI8FromUI4(*V_UI4_ref(ps) , core::ptr::addr_of_mut!((*V_UI8_ref(pd)))),
            VT_I8 => {
                (*V_UI8_ref(pd)) = (*V_I8_ref(ps)) as u64;
                return S_OK;
            }
            VT_R4 => return VarUI8FromR4(*V_R4_ref(ps) , core::ptr::addr_of_mut!((*V_UI8_ref(pd)))),
            VT_R8 => return VarUI8FromR8(*V_R8_ref(ps) , core::ptr::addr_of_mut!((*V_UI8_ref(pd)))),
            VT_DATE => return VarUI8FromDate(*V_DATE_ref(ps) , core::ptr::addr_of_mut!((*V_UI8_ref(pd)))),
            VT_BOOL => return VarUI8FromBool(*V_BOOL_ref(ps) , core::ptr::addr_of_mut!((*V_UI8_ref(pd)))),
            VT_CY => return VarUI8FromCy(*V_CY_ref(ps) , core::ptr::addr_of_mut!((*V_UI8_ref(pd)))),
            VT_DECIMAL => return VarUI8FromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), core::ptr::addr_of_mut!((*V_UI8_ref(pd)))),
            VT_DISPATCH => return VarUI8FromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_UI8_ref(pd)))),
            VT_BSTR => return VarUI8FromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_UI8_ref(pd)))),
            _ => {}
        },
        VT_I8 => match vtFrom {
            VT_EMPTY => {
                (*V_I8_ref(pd)) = 0;
                return S_OK;
            }
            VT_I4 => {
                (*V_I8_ref(pd)) = (*V_I4_ref(ps)) as i64;
                return S_OK;
            }
            VT_I1 => return VarI8FromI1(*V_I1_ref(ps) , core::ptr::addr_of_mut!((*V_I8_ref(pd)))),
            VT_I2 => return VarI8FromI2(*V_I2_ref(ps) , core::ptr::addr_of_mut!((*V_I8_ref(pd)))),
            VT_UI1 => return VarI8FromUI1(*V_UI1_ref(ps) , core::ptr::addr_of_mut!((*V_I8_ref(pd)))),
            VT_UI2 => return VarI8FromUI2(*V_UI2_ref(ps) , core::ptr::addr_of_mut!((*V_I8_ref(pd)))),
            VT_UI4 => return VarI8FromUI4(*V_UI4_ref(ps) , core::ptr::addr_of_mut!((*V_I8_ref(pd)))),
            VT_UI8 => {
                (*V_I8_ref(pd)) = (*V_UI8_ref(ps)) as i64;
                return S_OK;
            }
            VT_R4 => return VarI8FromR4(*V_R4_ref(ps) , core::ptr::addr_of_mut!((*V_I8_ref(pd)))),
            VT_R8 => return VarI8FromR8(*V_R8_ref(ps) , core::ptr::addr_of_mut!((*V_I8_ref(pd)))),
            VT_DATE => return VarI8FromDate(*V_DATE_ref(ps) , core::ptr::addr_of_mut!((*V_I8_ref(pd)))),
            VT_BOOL => return VarI8FromBool(*V_BOOL_ref(ps) , core::ptr::addr_of_mut!((*V_I8_ref(pd)))),
            VT_CY => return VarI8FromCy(*V_CY_ref(ps) , core::ptr::addr_of_mut!((*V_I8_ref(pd)))),
            VT_DECIMAL => return VarI8FromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), core::ptr::addr_of_mut!((*V_I8_ref(pd)))),
            VT_DISPATCH => return VarI8FromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_I8_ref(pd)))),
            VT_BSTR => return VarI8FromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_I8_ref(pd)))),
            _ => {}
        },
        VT_R4 => match vtFrom {
            VT_EMPTY => {
                (*V_R4_ref(pd)) = 0.0f32;
                return S_OK;
            }
            VT_I1 => return VarR4FromI1(*V_I1_ref(ps) , core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_I2 => return VarR4FromI2(*V_I2_ref(ps) , core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_I4 => return VarR4FromI4(*V_I4_ref(ps) , core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_UI1 => return VarR4FromUI1(*V_UI1_ref(ps) , core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_UI2 => return VarR4FromUI2(*V_UI2_ref(ps) , core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_UI4 => return VarR4FromUI4(*V_UI4_ref(ps) , core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_I8 => return VarR4FromI8(*V_I8_ref(ps) , core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_UI8 => return VarR4FromUI8(*V_UI8_ref(ps) , core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_R8 => return VarR4FromR8(*V_R8_ref(ps) , core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_DATE => return VarR4FromDate(*V_DATE_ref(ps) , core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_BOOL => return VarR4FromBool(*V_BOOL_ref(ps) , core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_CY => return VarR4FromCy(*V_CY_ref(ps) , core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_DECIMAL => return VarR4FromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_DISPATCH => return VarR4FromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            VT_BSTR => return VarR4FromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_R4_ref(pd)))),
            _ => {}
        },
        VT_R8 => match vtFrom {
            VT_EMPTY => {
                (*V_R8_ref(pd)) = 0.0f64;
                return S_OK;
            }
            VT_I1 => return VarR8FromI1(*V_I1_ref(ps) , core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_I2 => return VarR8FromI2(*V_I2_ref(ps) , core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_I4 => return VarR8FromI4(*V_I4_ref(ps) , core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_UI1 => return VarR8FromUI1(*V_UI1_ref(ps) , core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_UI2 => return VarR8FromUI2(*V_UI2_ref(ps) , core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_UI4 => return VarR8FromUI4(*V_UI4_ref(ps) , core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_I8 => return VarR8FromI8(*V_I8_ref(ps) , core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_UI8 => return VarR8FromUI8(*V_UI8_ref(ps) , core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_R4 => return VarR8FromR4(*V_R4_ref(ps) , core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_DATE => return VarR8FromDate(*V_DATE_ref(ps) , core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_BOOL => return VarR8FromBool(*V_BOOL_ref(ps) , core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_CY => return VarR8FromCy(*V_CY_ref(ps) , core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_DECIMAL => return VarR8FromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_DISPATCH => return VarR8FromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            VT_BSTR => return VarR8FromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_R8_ref(pd)))),
            _ => {}
        },
        VT_DATE => match vtFrom {
            VT_EMPTY => {
                (*V_DATE_ref(pd)) = 0.0f64;
                return S_OK;
            }
            VT_I1 => return VarDateFromI1(*V_I1_ref(ps) , core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_I2 => return VarDateFromI2(*V_I2_ref(ps) , core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_I4 => return VarDateFromI4(*V_I4_ref(ps) , core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_UI1 => return VarDateFromUI1(*V_UI1_ref(ps) , core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_UI2 => return VarDateFromUI2(*V_UI2_ref(ps) , core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_UI4 => return VarDateFromUI4(*V_UI4_ref(ps) , core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_I8 => return VarDateFromI8(*V_I8_ref(ps) , core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_UI8 => return VarDateFromUI8(*V_UI8_ref(ps) , core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_R4 => return VarDateFromR4(*V_R4_ref(ps) , core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_R8 => return VarDateFromR8(*V_R8_ref(ps) , core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_BOOL => return VarDateFromBool(*V_BOOL_ref(ps) , core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_CY => return VarDateFromCy(*V_CY_ref(ps) , core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_DECIMAL => return VarDateFromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_DISPATCH => return VarDateFromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            VT_BSTR => return VarDateFromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_DATE_ref(pd)))),
            _ => {}
        },
        VT_BOOL => match vtFrom {
            VT_EMPTY => {
                (*V_BOOL_ref(pd)) = 0;
                return S_OK;
            }
            VT_I1 => return VarBoolFromI1(*V_I1_ref(ps) , core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_I2 => return VarBoolFromI2(*V_I2_ref(ps) , core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_I4 => return VarBoolFromI4(*V_I4_ref(ps) , core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_UI1 => return VarBoolFromUI1(*V_UI1_ref(ps) , core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_UI2 => return VarBoolFromUI2(*V_UI2_ref(ps) , core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_UI4 => return VarBoolFromUI4(*V_UI4_ref(ps) , core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_I8 => return VarBoolFromI8(*V_I8_ref(ps) , core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_UI8 => return VarBoolFromUI8(*V_UI8_ref(ps) , core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_R4 => return VarBoolFromR4(*V_R4_ref(ps) , core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_R8 => return VarBoolFromR8(*V_R8_ref(ps) , core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_DATE => return VarBoolFromDate(*V_DATE_ref(ps) , core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_CY => return VarBoolFromCy(*V_CY_ref(ps) , core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_DECIMAL => return VarBoolFromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_DISPATCH => return VarBoolFromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            VT_BSTR => return VarBoolFromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BOOL_ref(pd)))),
            _ => {}
        },
        VT_BSTR => match vtFrom {
            VT_EMPTY => {
                (*V_BSTR_ref(pd)) = SysAllocStringLen(core::ptr::null_mut(), 0);
                return if !(*V_BSTR_ref(pd)).is_null() { S_OK } else { E_OUTOFMEMORY };
            }
            VT_BOOL => {
                if wFlags as u32 & (VARIANT_ALPHABOOL | VARIANT_LOCALBOOL) != 0 {
                    return VarBstrFromBool(*V_BOOL_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd))));
                }
                return VarBstrFromI2(*V_BOOL_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd))));
            }
            VT_I1 => return VarBstrFromI1(*V_I1_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            VT_I2 => return VarBstrFromI2(*V_I2_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            VT_I4 => return VarBstrFromI4(*V_I4_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            VT_UI1 => return VarBstrFromUI1(*V_UI1_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            VT_UI2 => return VarBstrFromUI2(*V_UI2_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            VT_UI4 => return VarBstrFromUI4(*V_UI4_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            VT_I8 => return VarBstrFromI8(*V_I8_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            VT_UI8 => return VarBstrFromUI8(*V_UI8_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            VT_R4 => return VarBstrFromR4(*V_R4_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            VT_R8 => return VarBstrFromR8(*V_R8_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            VT_DATE => return VarBstrFromDate(*V_DATE_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            VT_CY => return VarBstrFromCy(*V_CY_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            VT_DECIMAL => return VarBstrFromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            VT_DISPATCH => return VarBstrFromDisp(*V_DISPATCH_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_BSTR_ref(pd)))),
            _ => {}
        },
        VT_CY => match vtFrom {
            VT_EMPTY => {
                (*V_CY_ref(pd)).int64 = 0;
                return S_OK;
            }
            VT_I1 => return VarCyFromI1(*V_I1_ref(ps) , core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_I2 => return VarCyFromI2(*V_I2_ref(ps) , core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_I4 => return VarCyFromI4(*V_I4_ref(ps) , core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_UI1 => return VarCyFromUI1(*V_UI1_ref(ps) , core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_UI2 => return VarCyFromUI2(*V_UI2_ref(ps) , core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_UI4 => return VarCyFromUI4(*V_UI4_ref(ps) , core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_I8 => return VarCyFromI8(*V_I8_ref(ps) , core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_UI8 => return VarCyFromUI8(*V_UI8_ref(ps) , core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_R4 => return VarCyFromR4(*V_R4_ref(ps) , core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_R8 => return VarCyFromR8(*V_R8_ref(ps) , core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_DATE => return VarCyFromDate(*V_DATE_ref(ps) , core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_BOOL => return VarCyFromBool(*V_BOOL_ref(ps) , core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_DECIMAL => return VarCyFromDec(core::ptr::addr_of_mut!((*V_DECIMAL_ref(ps))), core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_DISPATCH => return VarCyFromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            VT_BSTR => return VarCyFromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_CY_ref(pd)))),
            _ => {}
        },
        VT_DECIMAL => match vtFrom {
            VT_EMPTY | VT_BOOL => {
                (*DEC_SIGNSCALE_ref(core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd))))) = SIGNSCALE(DECIMAL_POS, 0);
                (*DEC_HI32_ref(core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd))))) = 0;
                (*DEC_MID32_ref(core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd))))) = 0;
                (*DEC_LO32_ref(core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd))))) = if vtFrom == VT_BOOL && (*V_BOOL_ref(ps)) != 0 { 1 } else { 0 };
                return S_OK;
            }
            VT_I1 => return VarDecFromI1(*V_I1_ref(ps) , core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            VT_I2 => return VarDecFromI2(*V_I2_ref(ps) , core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            VT_I4 => return VarDecFromI4(*V_I4_ref(ps) , core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            VT_UI1 => return VarDecFromUI1(*V_UI1_ref(ps) , core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            VT_UI2 => return VarDecFromUI2(*V_UI2_ref(ps) , core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            VT_UI4 => return VarDecFromUI4(*V_UI4_ref(ps) , core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            VT_I8 => return VarDecFromI8(*V_I8_ref(ps) , core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            VT_UI8 => return VarDecFromUI8(*V_UI8_ref(ps) , core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            VT_R4 => return VarDecFromR4(*V_R4_ref(ps) , core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            VT_R8 => return VarDecFromR8(*V_R8_ref(ps) , core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            VT_DATE => return VarDecFromDate(*V_DATE_ref(ps) , core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            VT_CY => return VarDecFromCy(*V_CY_ref(ps) , core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            VT_DISPATCH => return VarDecFromDisp(*V_DISPATCH_ref(ps) , lcid, core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            VT_BSTR => return VarDecFromStr(*V_BSTR_ref(ps) , lcid, dwFlags, core::ptr::addr_of_mut!((*V_DECIMAL_ref(pd)))),
            _ => {}
        },
        VT_UNKNOWN => match vtFrom {
            VT_DISPATCH => {
                if (*V_DISPATCH_ref(ps)).is_null() {
                    (*V_UNKNOWN_ref(pd)) = core::ptr::null_mut();
                    res = S_OK;
                } else {
                    res = IDispatch_QueryInterface(*V_DISPATCH_ref(ps) , &IID_IUnknown, V_UNKNOWN_ref(pd) as *mut *mut core::ffi::c_void);
                }
            }
            _ => {}
        },
        VT_DISPATCH => match vtFrom {
            VT_UNKNOWN => {
                if (*V_UNKNOWN_ref(ps)).is_null() {
                    (*V_DISPATCH_ref(pd)) = core::ptr::null_mut();
                    res = S_OK;
                } else {
                    res = IUnknown_QueryInterface(*V_UNKNOWN_ref(ps) , &IID_IDispatch, V_DISPATCH_ref(pd) as *mut *mut core::ffi::c_void);
                }
            }
            _ => {}
        },
        VT_RECORD => {}
        _ => {}
    }
    res
}

unsafe fn VARIANT_CoerceArray(pd: *mut VARIANTARG, ps: *mut VARIANTARG, vt: VARTYPE) -> HRESULT {

    if vt == VT_BSTR && (*V_VT_ref(ps)) == (VT_ARRAY | VT_UI1) {
        return BstrFromVector(*V_ARRAY_ref(ps) , core::ptr::addr_of_mut!((*V_BSTR_ref(pd))));
    }

    if (*V_VT_ref(ps)) == VT_BSTR && vt == (VT_ARRAY | VT_UI1) {
        return VectorFromBstr(*V_BSTR_ref(ps) , core::ptr::addr_of_mut!((*V_ARRAY_ref(pd))));
    }

    if (*V_VT_ref(ps)) == vt {
        return SafeArrayCopy(*V_ARRAY_ref(ps) , core::ptr::addr_of_mut!((*V_ARRAY_ref(pd))));
    }

    DISP_E_TYPEMISMATCH
}

unsafe fn VARIANT_FetchDispatchValue(pvDispatch: LPVARIANT, pValue: LPVARIANT) -> HRESULT {
    let hres: HRESULT;
    static mut emptyParams: DISPPARAMS = DISPPARAMS {
        rgvarg: core::ptr::null_mut(),
        rgdispidNamedArgs: core::ptr::null_mut(),
        cArgs: 0,
        cNamedArgs: 0,
    };

    if ((*V_VT_ref(pvDispatch)) & VT_TYPEMASK) == VT_DISPATCH {
        if (*V_DISPATCH_ref(pvDispatch)).is_null() {
            return DISP_E_TYPEMISMATCH;
        }
        hres = IDispatch_Invoke(
            *V_DISPATCH_ref(pvDispatch) ,
            DISPID_VALUE as DISPID,
            &IID_NULL,
            LOCALE_USER_DEFAULT,
            DISPATCH_PROPERTYGET as WORD,
            core::ptr::addr_of_mut!(emptyParams),
            pValue,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    } else {
        hres = DISP_E_TYPEMISMATCH;
    }
    hres
}

unsafe fn VARIANT_ValidateType(vt: VARTYPE) -> HRESULT {
    let vtExtra = vt & VT_EXTRA_TYPE as VARTYPE;
    let vt = vt & VT_TYPEMASK;

    if (vtExtra & (VT_VECTOR | VT_RESERVED)) == 0 {
        if vt < VT_VOID || vt == VT_RECORD || vt == VT_CLSID {
            if (vtExtra & (VT_BYREF | VT_ARRAY)) != 0 && vt <= VT_NULL {
                return DISP_E_BADVARTYPE;
            }
            if vt != 15 {
                return S_OK;
            }
        }
    }
    DISP_E_BADVARTYPE
}

#[no_mangle]
pub unsafe extern "system" fn VariantInit(pVarg: *mut VARIANTARG) {
    TRACE!(b"(%p)\n\0".as_ptr() as PCHAR, pVarg);

    // Wine 10.0, as Windows 8.1+: the whole VARIANT is zeroed, not just vt.
    core::ptr::write_bytes(pVarg as *mut u8, 0, core::mem::size_of::<VARIANTARG>());
    (*V_VT_ref(pVarg)) = VT_EMPTY;
}
#[no_mangle]
pub unsafe extern "C" fn VARIANT_ClearInd(pVarg: *mut VARIANTARG) -> HRESULT {
    let mut hres: HRESULT;

    TRACE!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_variant(pVarg));

    hres = VARIANT_ValidateType(*V_VT_ref(pVarg) );
    if FAILED(hres) {
        return hres;
    }

    match *V_VT_ref(pVarg)  {
        VT_DISPATCH | VT_UNKNOWN => {
            if !(*V_UNKNOWN_ref(pVarg)).is_null() {
                IUnknown_Release(*V_UNKNOWN_ref(pVarg) );
            }
        }
        VT_UNKNOWN_VT_BYREF | VT_DISPATCH_VT_BYREF => {
            if !(*(*V_UNKNOWNREF_ref(pVarg))).is_null() {
                IUnknown_Release(*(*V_UNKNOWNREF_ref(pVarg)));
            }
        }
        VT_BSTR => {
            SysFreeString(*V_BSTR_ref(pVarg) );
        }
        VT_BSTR_VT_BYREF => {
            SysFreeString(*(*V_BSTRREF_ref(pVarg)));
        }
        VT_VARIANT_VT_BYREF => {
            VariantClear(*V_VARIANTREF_ref(pVarg) );
        }
        VT_RECORD | VT_RECORD_VT_BYREF => {
            let pBr = core::ptr::addr_of_mut!((*pVarg).n1.n2.n3.brecVal);
            if !(*pBr).pRecInfo.is_null() {
                IRecordInfo_RecordClear((*pBr).pRecInfo, (*pBr).pvRecord);
                IRecordInfo_Release((*pBr).pRecInfo);
            }
        }
        _ => {
            if V_ISARRAY(pVarg) || ((*V_VT_ref(pVarg)) & !VT_BYREF) == VT_SAFEARRAY {
                if V_ISBYREF(pVarg) {
                    if !(*(*V_ARRAYREF_ref(pVarg))).is_null() {
                        hres = SafeArrayDestroy(*(*V_ARRAYREF_ref(pVarg)));
                    }
                } else if !(*V_ARRAY_ref(pVarg)).is_null() {
                    hres = SafeArrayDestroy(*V_ARRAY_ref(pVarg) );
                }
            }
        }
    }

    (*V_VT_ref(pVarg)) = VT_EMPTY;
    hres
}

#[no_mangle]
pub unsafe extern "system" fn VariantClear(pVarg: *mut VARIANTARG) -> HRESULT {

    let mut hres: HRESULT;

    TRACE!(b"(%s)\n\0".as_ptr() as *const i8, debugstr_variant(pVarg));

    hres = VARIANT_ValidateType(*V_VT_ref(pVarg) );

    if SUCCEEDED(hres) {
        if !V_ISBYREF(pVarg) {
            if V_ISARRAY(pVarg) || (*V_VT_ref(pVarg)) == VT_SAFEARRAY {
                hres = SafeArrayDestroy(*V_ARRAY_ref(pVarg) );
            } else if (*V_VT_ref(pVarg)) == VT_BSTR {
                SysFreeString(*V_BSTR_ref(pVarg) );
            } else if (*V_VT_ref(pVarg)) == VT_RECORD {
                let pBr = core::ptr::addr_of_mut!((*pVarg).n1.n2.n3.brecVal);
                if !(*pBr).pRecInfo.is_null() {
                    IRecordInfo_RecordClear((*pBr).pRecInfo, (*pBr).pvRecord);
                    IRecordInfo_Release((*pBr).pRecInfo);
                }
            } else if (*V_VT_ref(pVarg)) == VT_DISPATCH || (*V_VT_ref(pVarg)) == VT_UNKNOWN {
                if !(*V_UNKNOWN_ref(pVarg)).is_null() {
                    IUnknown_Release(*V_UNKNOWN_ref(pVarg) );
                }
            }
        }
        (*V_VT_ref(pVarg)) = VT_EMPTY;
    }
    hres
}

unsafe fn VARIANT_CopyIRecordInfo(dest: *mut VARIANT, src: *mut VARIANT) -> HRESULT {

    let dest_rec = core::ptr::addr_of_mut!((*dest).n1.n2.n3.brecVal);
    let src_rec = core::ptr::addr_of_mut!((*src).n1.n2.n3.brecVal);
    let hr: HRESULT;
    let mut size: ULONG = 0;

    if (*src_rec).pRecInfo.is_null() {
        if !(*src_rec).pvRecord.is_null() {
            return E_INVALIDARG;
        }
        return S_OK;
    }

    hr = IRecordInfo_GetSize((*src_rec).pRecInfo, &mut size);
    if FAILED(hr) {
        return hr;
    }

    // COM task memory, as RecordCreate/RecordDestroy now use (Wine 10.0).
    (*dest_rec).pvRecord = CoTaskMemAlloc(size as SIZE_T);
    if (*dest_rec).pvRecord.is_null() {
        return E_OUTOFMEMORY;
    }
    core::ptr::write_bytes((*dest_rec).pvRecord as *mut u8, 0, size as usize);

    (*dest_rec).pRecInfo = (*src_rec).pRecInfo;
    IRecordInfo_AddRef((*src_rec).pRecInfo);

    IRecordInfo_RecordCopy((*src_rec).pRecInfo, (*src_rec).pvRecord, (*dest_rec).pvRecord)
}

#[no_mangle]
pub unsafe extern "system" fn VariantCopy(pvargDest: *mut VARIANTARG, pvargSrc: *mut VARIANTARG) -> HRESULT {
    let mut hres: HRESULT = S_OK;

    TRACE!(b"(%s,%s)\n\0".as_ptr() as *const i8, debugstr_variant(pvargDest), debugstr_variant(pvargSrc));

    if V_TYPE(pvargSrc) == VT_CLSID || FAILED(VARIANT_ValidateType(*V_VT_ref(pvargSrc) )) {
        return DISP_E_BADVARTYPE;
    }

    if pvargDest != pvargSrc {
        hres = VariantClear(pvargDest);
        if SUCCEEDED(hres) {
            *pvargDest = *pvargSrc;

            if !V_ISBYREF(pvargSrc) {
                match *V_VT_ref(pvargSrc)  {
                    VT_BSTR => {
                        let bstr_src = *V_BSTR_ref(pvargSrc) ;
                        let byte_len = SysStringByteLen(bstr_src);
                        let new_bstr = SysAllocStringByteLen(bstr_src as *mut i8, byte_len);
                        (*V_BSTR_ref(pvargDest)) = new_bstr;
                        if new_bstr.is_null() {
                            hres = E_OUTOFMEMORY;
                        }
                    }
                    VT_RECORD => {
                        hres = VARIANT_CopyIRecordInfo(pvargDest, pvargSrc);
                    }
                    VT_DISPATCH | VT_UNKNOWN => {
                        let unk_src = *V_UNKNOWN_ref(pvargSrc) ;
                        (*V_UNKNOWN_ref(pvargDest)) = unk_src;
                        if !unk_src.is_null() {
                            IUnknown_AddRef(unk_src);
                        }
                    }
                    _ => {
                        if V_ISARRAY(pvargSrc) {
                            hres = SafeArrayCopy(*V_ARRAY_ref(pvargSrc) , V_ARRAY_ref(pvargDest));
                        }
                    }
                }
            }
        }
    }

    hres
}

unsafe fn VARIANT_DataSize(pv: *const VARIANT) -> usize {
    match V_TYPE(pv as *mut VARIANT) {
        VT_I1 | VT_UI1 => core::mem::size_of::<BYTE>(),
        VT_I2 | VT_UI2 => core::mem::size_of::<SHORT>(),
        VT_INT | VT_UINT | VT_I4 | VT_UI4 => core::mem::size_of::<LONG>(),
        VT_I8 | VT_UI8 => core::mem::size_of::<LONGLONG>(),
        VT_R4 => core::mem::size_of::<f32>(),
        VT_R8 => core::mem::size_of::<f64>(),
        VT_DATE => core::mem::size_of::<DATE>(),
        VT_BOOL => core::mem::size_of::<VARIANT_BOOL>(),
        VT_DISPATCH | VT_UNKNOWN | VT_BSTR => core::mem::size_of::<*mut core::ffi::c_void>(),
        VT_CY => core::mem::size_of::<CY>(),
        VT_ERROR => core::mem::size_of::<SCODE>(),
        _ => {
            TRACE!(b"Shouldn't be called for variant %s!\n\0".as_ptr() as PCHAR, debugstr_variant(pv));
            0
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn VariantCopyInd(pvargDest: *mut VARIANT, pvargSrc: *mut VARIANTARG) -> HRESULT {
    let mut vTmp: VARIANTARG;
    let mut pSrc: *mut VARIANTARG = pvargSrc;
    let vt: VARTYPE;
    let mut hres: HRESULT = S_OK;

    TRACE!(b"(%s,%s)\n\0".as_ptr() as *const i8, debugstr_variant(pvargDest), debugstr_variant(pvargSrc));

    if !V_ISBYREF(pvargSrc) {
        return VariantCopy(pvargDest, pvargSrc);
    }

    vt = V_TYPE(pvargSrc);
    if V_ISARRAY(pvargSrc) || ((*V_VT_ref(pvargSrc)) == (VT_RECORD | VT_BYREF)) ||
       (vt > VT_NULL && vt != 15 && vt < VT_VOID &&
       ((*V_VT_ref(pvargSrc)) & (VT_VECTOR | VT_RESERVED)) == 0) {
    } else {
        return E_INVALIDARG;
    }

    if pvargSrc == pvargDest {
        vTmp = *pvargSrc;
        pSrc = &mut vTmp;
        (*V_VT_ref(pvargDest)) = VT_EMPTY;
    } else {
        hres = VariantClear(pvargDest);
        if FAILED(hres) {
            TRACE!(b"VariantClear() of destination failed\n\0".as_ptr() as *const i8);
            return hres;
        }
    }

    if V_ISARRAY(pSrc) {
        hres = SafeArrayCopy(*(*V_ARRAYREF_ref(pSrc)), V_ARRAY_ref(pvargDest));
    } else if (*V_VT_ref(pSrc)) == (VT_BSTR | VT_BYREF) {
        (*V_BSTR_ref(pvargDest)) = SysAllocStringByteLen((*(*V_BSTRREF_ref(pSrc))) as *const i8, SysStringByteLen(*(*V_BSTRREF_ref(pSrc))));
    } else if (*V_VT_ref(pSrc)) == (VT_RECORD | VT_BYREF) {
        hres = VARIANT_CopyIRecordInfo(pvargDest, pvargSrc);
    } else if (*V_VT_ref(pSrc)) == (VT_DISPATCH | VT_BYREF) || (*V_VT_ref(pSrc)) == (VT_UNKNOWN | VT_BYREF) {
        (*V_UNKNOWN_ref(pvargDest)) = *(*V_UNKNOWNREF_ref(pSrc));
        if !(*(*V_UNKNOWNREF_ref(pSrc))).is_null() {
            IUnknown_AddRef(*(*V_UNKNOWNREF_ref(pSrc)));
        }
    } else if (*V_VT_ref(pSrc)) == (VT_VARIANT | VT_BYREF) {
        if (*V_VT_ref(*V_VARIANTREF_ref(pSrc) )) == (VT_VARIANT | VT_BYREF) {
            hres = E_INVALIDARG;
        } else {
            hres = VariantCopyInd(pvargDest, *V_VARIANTREF_ref(pSrc) );
        }

        if pSrc != pvargSrc {
            VariantClear(pSrc);
        }

        TRACE!(b"returning 0x%08x, %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(pvargDest));
        return hres;
    } else if (*V_VT_ref(pSrc)) == (VT_DECIMAL | VT_BYREF) {
        memcpy(
            core::ptr::addr_of_mut!((*V_DECIMAL_ref(pvargDest)).__bindgen_anon_1.__bindgen_anon_1.scale) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*(*V_DECIMALREF_ref(pSrc))).__bindgen_anon_1.__bindgen_anon_1.scale) as *const core::ffi::c_void,
            core::mem::size_of::<DECIMAL>() - core::mem::size_of::<USHORT>()
        );
    } else {
        memcpy(
            core::ptr::addr_of_mut!((*V_BYREF_ref(pvargDest))) as *mut core::ffi::c_void,
            (*V_BYREF_ref(pSrc)) as *const core::ffi::c_void,
            VARIANT_DataSize(pSrc)
        );
    }

    (*V_VT_ref(pvargDest)) = (*V_VT_ref(pSrc)) & !VT_BYREF;

    if pSrc != pvargSrc {
        VariantClear(pSrc);
    }

    TRACE!(b"returning 0x%08x, %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(pvargDest));
    return hres;
}

#[no_mangle]
pub unsafe extern "system" fn VariantChangeType(
    pvargDest: *mut VARIANTARG,
    pvargSrc: *mut VARIANTARG,
    wFlags: USHORT,
    vt: VARTYPE,
) -> HRESULT {
    VariantChangeTypeEx(pvargDest, pvargSrc, LOCALE_USER_DEFAULT, wFlags, vt)
}

#[no_mangle]
pub unsafe extern "system" fn VariantChangeTypeEx(
    pvargDest: *mut VARIANTARG,
    pvargSrc: *mut VARIANTARG,
    lcid: LCID,
    wFlags: USHORT,
    vt: VARTYPE,
) -> HRESULT {
    let mut res: HRESULT;

    TRACE!(
        b"(%s,%s,0x%08x,0x%04x,%s)\n\0".as_ptr() as *const i8,
        debugstr_variant(pvargDest),
        debugstr_variant(pvargSrc),
        lcid,
        wFlags,
        debugstr_vt(vt),
    );

    if vt == VT_CLSID {
        res = DISP_E_BADVARTYPE;
    } else {
        res = VARIANT_ValidateType(*V_VT_ref(pvargSrc) );

        if SUCCEEDED(res) {
            res = VARIANT_ValidateType(vt);

            if SUCCEEDED(res) {
                let mut vTmp: VARIANTARG = core::mem::zeroed();
                let mut vSrcDeref: VARIANTARG = core::mem::zeroed();

                if V_ISBYREF(pvargSrc) && (*V_BYREF_ref(pvargSrc)).is_null() {
                    res = DISP_E_TYPEMISMATCH;
                } else {
                    (*V_VT_ref(&mut vTmp)) = VT_EMPTY;
                    (*V_VT_ref(&mut vSrcDeref)) = VT_EMPTY;
                    VariantClear(&mut vTmp);
                    VariantClear(&mut vSrcDeref);
                }

                if SUCCEEDED(res) {
                    res = VariantCopyInd(&mut vSrcDeref, pvargSrc);
                    if SUCCEEDED(res) {
                        if V_ISARRAY(&mut vSrcDeref) || (vt & VT_ARRAY) != 0 {
                            res = VARIANT_CoerceArray(&mut vTmp, &mut vSrcDeref, vt);
                        } else {
                            res = VARIANT_Coerce(&mut vTmp, lcid, wFlags, &mut vSrcDeref, vt);
                        }

                        if SUCCEEDED(res) {
                            (*V_VT_ref(&mut vTmp)) = vt;
                            res = VariantCopy(pvargDest, &mut vTmp);
                        }
                        VariantClear(&mut vTmp);
                        VariantClear(&mut vSrcDeref);
                    }
                }
            }
        }
    }

    TRACE!(
        b"returning 0x%08x, %s\n\0".as_ptr() as *const i8,
        res,
        debugstr_variant(pvargDest),
    );
    res
}

unsafe fn VARIANT_JulianFromDate(dateIn: i32) -> i32 {
    let mut julianDays = dateIn;
    julianDays -= DATE_MIN as i32;
    julianDays += 1757585;
    julianDays
}

unsafe fn VARIANT_DateFromJulian(dateIn: i32) -> i32 {
    let mut julianDays = dateIn;
    
    julianDays -= 1757585;
    julianDays += DATE_MIN as i32;
    julianDays
}

unsafe fn VARIANT_DMYFromJulian(jd: i32, year: *mut u16, month: *mut u16, day: *mut u16) {
    let j: i32;
    let i: i32;
    let mut l: i32;
    let n: i32;

    l = jd + 68569;
    n = l * 4 / 146097;
    l -= (n * 146097 + 3) / 4;
    i = (4000 * (l + 1)) / 1461001;
    l += 31 - (i * 1461) / 4;
    j = (l * 80) / 2447;
    *day = (l - (j * 2447) / 80) as u16;
    l = j / 11;
    *month = ((j + 2) - (12 * l)) as u16;
    *year = (100 * (n - 49) + i + l) as u16;
}

unsafe fn VARIANT_JulianFromDMY(year: u16, month: u16, day: u16) -> f64 {
    let m12 = ((month as i32 - 14) / 12) as i32;
    
    ((1461 * (year as i32 + 4800 + m12)) / 4 + (367 * (month as i32 - 2 - 12 * m12)) / 12 -
     (3 * ((year as i32 + 4900 + m12) / 100)) / 4 + day as i32 - 32075) as f64
}

unsafe fn VARIANT_RollUdate(lpUd: *mut UDATE) -> HRESULT {
    static DAYS: [u8; 13] = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut iYear: i16;
    let mut iMonth: i16;
    let mut iDay: i16;
    let mut iHour: i16;
    let mut iMinute: i16;
    let mut iSecond: i16;

    iYear = (*lpUd).st.wYear as i16;
    iMonth = (*lpUd).st.wMonth as i16;
    iDay = (*lpUd).st.wDay as i16;
    iHour = (*lpUd).st.wHour as i16;
    iMinute = (*lpUd).st.wMinute as i16;
    iSecond = (*lpUd).st.wSecond as i16;

    TRACE!(b"Raw date: %d/%d/%d %d:%d:%d\n\0".as_ptr() as *const i8,
          iDay, iMonth, iYear, iHour, iMinute, iSecond);

    if iYear > 9999 || iYear < -9999 {
        return E_INVALIDARG;
    }
    // Wine 10.0, as current Windows: two-digit years 00-49 are 20xx (was 00-29).
    if iYear >= 0 && iYear <= 49 {
        iYear += 2000;
    } else if iYear >= 50 && iYear < 100 {
        iYear += 1900;
    }

    iMinute += iSecond / 60;
    iSecond = iSecond % 60;
    iHour += iMinute / 60;
    iMinute = iMinute % 60;
    iDay += iHour / 24;
    iHour = iHour % 24;
    iYear += iMonth / 12;
    iMonth = iMonth % 12;
    if iMonth <= 0 {
        iMonth += 12;
        iYear -= 1;
    }
    while iDay > DAYS[iMonth as usize] as i16 {
        if iMonth == 2 && IsLeapYear(iYear as i32) {
            iDay -= 29;
        } else {
            iDay -= DAYS[iMonth as usize] as i16;
        }
        iMonth += 1;
        iYear += iMonth / 12;
        iMonth = iMonth % 12;
    }
    while iDay <= 0 {
        iMonth -= 1;
        if iMonth <= 0 {
            iMonth += 12;
            iYear -= 1;
        }
        if iMonth == 2 && IsLeapYear(iYear as i32) {
            iDay += 29;
        } else {
            iDay += DAYS[iMonth as usize] as i16;
        }
    }

    if iSecond < 0 {
        iSecond += 60;
        iMinute -= 1;
    }
    if iMinute < 0 {
        iMinute += 60;
        iHour -= 1;
    }
    if iHour < 0 {
        iHour += 24;
        iDay -= 1;
    }
    if iYear <= 0 {
        iYear += 2000;
    }

    (*lpUd).st.wYear = iYear as u16;
    (*lpUd).st.wMonth = iMonth as u16;
    (*lpUd).st.wDay = iDay as u16;
    (*lpUd).st.wHour = iHour as u16;
    (*lpUd).st.wMinute = iMinute as u16;
    (*lpUd).st.wSecond = iSecond as u16;

    TRACE!(b"Rolled date: %d/%d/%d %d:%d:%d\n\0".as_ptr() as *const i8,
          (*lpUd).st.wDay, (*lpUd).st.wMonth, (*lpUd).st.wYear,
          (*lpUd).st.wHour, (*lpUd).st.wMinute, (*lpUd).st.wSecond);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DosDateTimeToVariantTime(
    wDosDate: USHORT,
    wDosTime: USHORT,
    pDateOut: *mut f64,
) -> INT {
    let mut ud: UDATE = core::mem::zeroed();

    TRACE!(
        b"(0x%x(%d/%d/%d),0x%x(%d:%d:%d),%p)\n\0".as_ptr() as PCHAR,
        wDosDate,
        DOS_YEAR(wDosDate),
        DOS_MONTH(wDosDate),
        DOS_DAY(wDosDate),
        wDosTime,
        DOS_HOUR(wDosTime),
        DOS_MINUTE(wDosTime),
        DOS_SECOND(wDosTime),
        pDateOut,
    );

    ud.st.wYear = DOS_YEAR(wDosDate) as u16;
    ud.st.wMonth = DOS_MONTH(wDosDate) as u16;
    if ud.st.wYear > 2099 || ud.st.wMonth > 12 {
        return 0;
    }
    ud.st.wDay = DOS_DAY(wDosDate) as u16;
    ud.st.wHour = DOS_HOUR(wDosTime) as u16;
    ud.st.wMinute = DOS_MINUTE(wDosTime) as u16;
    ud.st.wSecond = DOS_SECOND(wDosTime) as u16;
    ud.st.wDayOfWeek = 0;
    ud.st.wMilliseconds = 0;
    if ud.st.wHour > 23 || ud.st.wMinute > 59 || ud.st.wSecond > 59 {
        return 0;
    }

    if VarDateFromUdate(core::ptr::addr_of_mut!(ud), 0, pDateOut) == S_OK {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn VariantTimeToDosDateTime(
    dateIn: f64,
    pwDosDate: *mut u16,
    pwDosTime: *mut u16,
) -> i32 {
    let mut ud: UDATE = core::mem::zeroed();

    TRACE!(
        b"(%g,%p,%p)\n\0".as_ptr() as *const i8,
        dateIn,
        pwDosDate,
        pwDosTime,
    );

    if FAILED(VarUdateFromDate(dateIn, 0, &mut ud)) {
        return 0;
    }

    if ud.st.wYear < 1980 || ud.st.wYear > 2099 {
        return 0;
    }

    *pwDosDate = DOS_DATE(ud.st.wDay as i32, ud.st.wMonth as i32, ud.st.wYear as i32) as u16;
    *pwDosTime = DOS_TIME(ud.st.wHour as i32, ud.st.wMinute as i32, ud.st.wSecond as i32) as u16;

    TRACE!(
        b"Returning 0x%x(%d/%d/%d), 0x%x(%d:%d:%d)\n\0".as_ptr() as *const i8,
        *pwDosDate,
        DOS_YEAR(*pwDosDate),
        DOS_MONTH(*pwDosDate),
        DOS_DAY(*pwDosDate),
        *pwDosTime,
        DOS_HOUR(*pwDosTime),
        DOS_MINUTE(*pwDosTime),
        DOS_SECOND(*pwDosTime),
    );
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SystemTimeToVariantTime(lpSt: LPSYSTEMTIME, pDateOut: *mut f64) -> i32 {
    let mut ud: UDATE = core::mem::zeroed();

    TRACE!(b"(%p->%d/%d/%d %d:%d:%d,%p)\n\0".as_ptr(), lpSt, (*lpSt).wDay, (*lpSt).wMonth,
          (*lpSt).wYear, (*lpSt).wHour, (*lpSt).wMinute, (*lpSt).wSecond, pDateOut);

    if (*lpSt).wMonth > 12 {
        return 0;
    }
    if (*lpSt).wDay > 31 {
        return 0;
    }
    if ((*lpSt).wYear as i16) < 0 {
        return 0;
    }

    ud.st = *(lpSt as *const _SYSTEMTIME);
    if VarDateFromUdate(&mut ud, 0, pDateOut) == S_OK {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn VariantTimeToSystemTime(
    dateIn: f64,
    lpSt: LPSYSTEMTIME,
) -> i32 {
    let mut ud: UDATE = core::mem::zeroed();

    TRACE!(b"(%g,%p)\n\0".as_ptr() as *const i8, dateIn, lpSt);

    if FAILED(VarUdateFromDate(dateIn, 0, &mut ud)) {
        return 0;
    }

    core::ptr::copy_nonoverlapping(core::ptr::addr_of!(ud.st) as *const SYSTEMTIME, lpSt, 1);
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromUdateEx(
    pUdateIn: *mut UDATE,
    lcid: LCID,
    dwFlags: ULONG,
    pDateOut: *mut DATE,
) -> HRESULT {
    let mut ud: UDATE;
    let mut dateVal: f64 = 0.0;

    TRACE!(
        b"(%p->%d/%d/%d %d:%d:%d:%d %d %d,0x%08x,0x%08x,%p)\n\0".as_ptr() as *const i8,
        pUdateIn,
        (*pUdateIn).st.wMonth,
        (*pUdateIn).st.wDay,
        (*pUdateIn).st.wYear,
        (*pUdateIn).st.wHour,
        (*pUdateIn).st.wMinute,
        (*pUdateIn).st.wSecond,
        (*pUdateIn).st.wMilliseconds,
        (*pUdateIn).st.wDayOfWeek,
        (*pUdateIn).wDayOfYear,
        lcid,
        dwFlags,
        pDateOut,
    );

    if lcid != MAKELCID(MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US), SORT_DEFAULT) {
        FIXME!(b"lcid possibly not handled, treating as en-us\n\0".as_ptr() as *const i8);
    }
    if (dwFlags & !(VAR_TIMEVALUEONLY | VAR_DATEVALUEONLY)) != 0 {
        FIXME!(b"unsupported flags: %x\n\0".as_ptr() as *const i8, dwFlags);
    }

    ud = *pUdateIn;

    if (dwFlags & VAR_VALIDDATE) != 0 {
        WARN!(b"Ignoring VAR_VALIDDATE\n\0".as_ptr() as *const i8);
    }

    if FAILED(VARIANT_RollUdate(&mut ud)) {
        return E_INVALIDARG;
    }

    if (dwFlags & VAR_TIMEVALUEONLY) == 0 {
        dateVal = VARIANT_DateFromJulian(VARIANT_JulianFromDMY(
            ud.st.wYear,
            ud.st.wMonth,
            ud.st.wDay,
        ) as i32) as f64;
    }

    if (dwFlags & VAR_TIMEVALUEONLY) != 0 || (dwFlags & VAR_DATEVALUEONLY) == 0 {
        let dateSign: f64 = if dateVal < 0.0 { -1.0 } else { 1.0 };

        dateVal += ud.st.wHour as f64 / 24.0 * dateSign;
        dateVal += ud.st.wMinute as f64 / 1440.0 * dateSign;
        dateVal += ud.st.wSecond as f64 / 86400.0 * dateSign;
    }

    TRACE!(b"Returning %g\n\0".as_ptr() as *const i8, dateVal);
    *pDateOut = dateVal;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn VarDateFromUdate(
    pUdateIn: *mut UDATE,
    dwFlags: ULONG,
    pDateOut: *mut DATE,
) -> HRESULT {
    let lcid: LCID = MAKELCID(MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US), SORT_DEFAULT);
    
    VarDateFromUdateEx(pUdateIn, lcid, dwFlags, pDateOut)
}

#[no_mangle]
pub unsafe extern "system" fn VarUdateFromDate(
    dateIn: f64,
    _dwFlags: u32,
    lpUdate: *mut UDATE,
) -> HRESULT {
    static CUMULATIVE_DAYS: [u16; 13] = [
        0, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334
    ];
    
    
    macro_rules! IsLeapYear {
        ($y:expr) => {
            ((($y) % 4) == 0) && (((($y) % 100) != 0) || ((($y) % 400) == 0))
        };
    }
    
    macro_rules! TRACE {
        ($($arg:tt)*) => {};
    }
    
    TRACE!("(%g,0x%08x,%p)\n", dateIn, dwFlags, lpUdate);
    
    if dateIn <= (-657434.0 - 1.0) || dateIn >= (2958465.0 + 1.0) {
        return 0x80070057u32 as i32;
    }
    
    let datePart = if dateIn < 0.0 { ceil(dateIn) } else { floor(dateIn) };
    let mut timePart = fabs(dateIn - datePart) + 0.00000000001;
    if timePart >= 1.0 {
        timePart -= 0.00000000001;
    }
    
    let julianDays = VARIANT_JulianFromDate(dateIn as i32);
    VARIANT_DMYFromJulian(
        julianDays,
        core::ptr::addr_of_mut!((*lpUdate).st.wYear),
        core::ptr::addr_of_mut!((*lpUdate).st.wMonth),
        core::ptr::addr_of_mut!((*lpUdate).st.wDay),
    );
    
    let datePart_dow = (datePart + 1.5) / 7.0;
    (*lpUdate).st.wDayOfWeek = ((datePart_dow - floor(datePart_dow)) * 7.0) as u16;
    if (*lpUdate).st.wDayOfWeek == 0 {
        (*lpUdate).st.wDayOfWeek = 5;
    } else if (*lpUdate).st.wDayOfWeek == 1 {
        (*lpUdate).st.wDayOfWeek = 6;
    } else {
        (*lpUdate).st.wDayOfWeek -= 2;
    }
    
    if (*lpUdate).st.wMonth > 2 && IsLeapYear!((*lpUdate).st.wYear) {
        (*lpUdate).wDayOfYear = 1;
    } else {
        (*lpUdate).wDayOfYear = 0;
    }
    
    (*lpUdate).wDayOfYear += CUMULATIVE_DAYS[(*lpUdate).st.wMonth as usize];
    (*lpUdate).wDayOfYear += (*lpUdate).st.wDay;
    
    timePart *= 24.0;
    (*lpUdate).st.wHour = timePart as u16;
    timePart -= (*lpUdate).st.wHour as f64;
    timePart *= 60.0;
    (*lpUdate).st.wMinute = timePart as u16;
    timePart -= (*lpUdate).st.wMinute as f64;
    timePart *= 60.0;
    (*lpUdate).st.wSecond = timePart as u16;
    timePart -= (*lpUdate).st.wSecond as f64;
    (*lpUdate).st.wMilliseconds = 0;
    
    if timePart > 0.5 {
        if (*lpUdate).st.wSecond < 59 {
            (*lpUdate).st.wSecond += 1;
        } else {
            (*lpUdate).st.wSecond = 0;
            if (*lpUdate).st.wMinute < 59 {
                (*lpUdate).st.wMinute += 1;
            } else {
                (*lpUdate).st.wMinute = 0;
                if (*lpUdate).st.wHour < 23 {
                    (*lpUdate).st.wHour += 1;
                } else {
                    (*lpUdate).st.wHour = 0;
                    (*lpUdate).st.wDay += 1;
                    if (*lpUdate).st.wDay > 28 {
                        VARIANT_RollUdate(lpUdate);
                    }
                }
            }
        }
    }
    
    0i32
}

/* The localised characters that make up a valid number (Wine 10.0 variant.c).
 * Wine 10.0 replaced the one-character cCurrencyLocal/cCurrencyLocal2 pair with
 * the whole currency string (up to 7 chars + NUL), matched with wcsncmp, and
 * dropped the lcid cache. */
#[repr(C)]
#[derive(Copy, Clone)]
pub struct VARIANT_NUMBER_CHARS {
    pub cNegativeSymbol: WCHAR,
    pub cPositiveSymbol: WCHAR,
    pub cDecimalPoint: WCHAR,
    pub cDigitSeparator: WCHAR,
    pub sCurrencyLen: DWORD,
    pub sCurrency: [WCHAR; 8],
    pub cCurrencyDecimalPoint: WCHAR,
    pub cCurrencyDigitSeparator: WCHAR,
}

extern "C" {
    #[link_name = "wcsncmp"]
    fn wcsncmp_numchars(s1: *const WCHAR, s2: *const WCHAR, n: usize) -> i32;
}

unsafe fn VARIANT_GetLocalisedNumberChars(lpChars: *mut VARIANT_NUMBER_CHARS, lcid: LCID, dwFlags: DWORD) {
    let lctype: LCTYPE = dwFlags & LOCALE_NOUSEROVERRIDE as u32;
    let mut buff: [WCHAR; 4] = [0; 4];

    *lpChars = VARIANT_NUMBER_CHARS {
        cNegativeSymbol: b'-' as u16,
        cPositiveSymbol: b'+' as u16,
        cDecimalPoint: b'.' as u16,
        cDigitSeparator: 0,
        sCurrencyLen: 1,
        sCurrency: [b'$' as u16, 0, 0, 0, 0, 0, 0, 0],
        cCurrencyDecimalPoint: 0,
        cCurrencyDigitSeparator: b',' as u16,
    };

    // GET_NUMBER_TEXT: the locale's first character, if it has one
    macro_rules! get_number_text {
        ($fld:expr, $name:ident) => {{
            buff[0] = 0;
            if GetLocaleInfoW(lcid, lctype | $fld, buff.as_mut_ptr(), buff.len() as i32) != 0
                && buff[0] != 0
            {
                (*lpChars).$name = buff[0];
            }
        }};
    }
    get_number_text!(LOCALE_SNEGATIVESIGN, cNegativeSymbol);
    get_number_text!(LOCALE_SPOSITIVESIGN, cPositiveSymbol);
    get_number_text!(LOCALE_SDECIMAL, cDecimalPoint);
    get_number_text!(LOCALE_STHOUSAND, cDigitSeparator);
    get_number_text!(LOCALE_SMONDECIMALSEP, cCurrencyDecimalPoint);
    get_number_text!(LOCALE_SMONTHOUSANDSEP, cCurrencyDigitSeparator);

    if GetLocaleInfoW(
        lcid,
        lctype | LOCALE_SCURRENCY,
        (*lpChars).sCurrency.as_mut_ptr(),
        (*lpChars).sCurrency.len() as i32,
    ) == 0
    {
        (*lpChars).sCurrency[0] = 0;
    }
    if (*lpChars).sCurrency[0] == 0 {
        (*lpChars).sCurrency[0] = b'$' as u16;
        (*lpChars).sCurrency[1] = 0;
    }
    let mut n = 0usize;
    while n < (*lpChars).sCurrency.len() && (*lpChars).sCurrency[n] != 0 {
        n += 1;
    }
    (*lpChars).sCurrencyLen = n as DWORD;
}

/* Digits are ASCII only (Wine 10.0; iswdigit also took other scripts' digits,
 * which the `- '0'` arithmetic below cannot convert). */
#[inline]
fn is_digit(c: WCHAR) -> bool {
    (b'0' as u16..=b'9' as u16).contains(&c)
}

/// VarParseNumFromStr [OLEAUT32.46], ported from Wine 10.0 (the pre-10
/// translation is replaced whole). What changed, all visible in Wine's
/// vartest: a leading decimal point is accepted; a thousands separator before
/// the first digit is DISP_E_TYPEMISMATCH; the decimal point masks an equal
/// thousands separator, and a non-breaking-space separator admits plain
/// spaces; currency decimal/thousands separators are always accepted (and set
/// NUMPRS_CURRENCY); the whole currency string is matched, before or after
/// the number; hex/octal is refused after currency or a decimal point; no
/// exponent inside currency.
#[no_mangle]
pub unsafe extern "system" fn VarParseNumFromStr(
    lpszStr: *mut OLECHAR,
    lcid: LCID,
    dwFlags: ULONG,
    pNumprs: *mut NUMPARSE,
    rgbDig: *mut BYTE,
) -> HRESULT {
    let mut chars: VARIANT_NUMBER_CHARS = core::mem::zeroed();
    let mut rgbTmp: [BYTE; 1024] = [0; 1024];
    let mut dwState: DWORD = B_EXPONENT_START | B_INEXACT_ZEROS;
    let mut iMaxDigits: i32 = rgbTmp.len() as i32;
    let mut cchUsed: i32 = 0;
    let mut s = lpszStr as *const OLECHAR;

    if pNumprs.is_null() || rgbDig.is_null() {
        return E_INVALIDARG;
    }
    let np = &mut *pNumprs;
    let in_flags = np.dwInFlags;

    if np.cDig < iMaxDigits {
        iMaxDigits = np.cDig;
    }

    np.cDig = 0;
    np.dwOutFlags = 0;
    np.cchUsed = 0;
    np.nBaseShift = 0;
    np.nPwr10 = 0;

    if s.is_null() {
        return DISP_E_TYPEMISMATCH;
    }

    VARIANT_GetLocalisedNumberChars(&mut chars, lcid, dwFlags);
    if chars.cDigitSeparator == chars.cDecimalPoint {
        /* The decimal point completely masks the digit separator */
        chars.cDigitSeparator = 0;
    }
    /* A non-breaking-space thousands separator implies regular spaces are
     * allowed too (not the converse). */
    let cDigitSeparator2: WCHAR = if chars.cDigitSeparator == 0xa0 { b' ' as u16 } else { 0 };
    let is_sep = |c: WCHAR| {
        (chars.cDigitSeparator != 0 && c == chars.cDigitSeparator)
            || (cDigitSeparator2 != 0 && c == cDigitSeparator2)
    };
    let cur_at = |p: *const OLECHAR| {
        wcsncmp_numchars(p, chars.sCurrency.as_ptr(), chars.sCurrencyLen as usize) == 0
    };
    let thousands_currency = (NUMPRS_THOUSANDS | NUMPRS_CURRENCY) as u32;

    /* First consume all the leading symbols and space from the string */
    loop {
        let c = *s;
        if in_flags & NUMPRS_DECIMAL != 0
            && (c == chars.cDecimalPoint || c == chars.cCurrencyDecimalPoint)
        {
            np.dwOutFlags |= NUMPRS_DECIMAL;
            if c == chars.cCurrencyDecimalPoint && chars.cDecimalPoint != chars.cCurrencyDecimalPoint {
                np.dwOutFlags |= NUMPRS_CURRENCY;
            }
            cchUsed += 1;
            s = s.add(1);

            /* If we have no digits so far, skip leading zeros */
            if np.cDig == 0 {
                while *s == b'0' as u16 {
                    dwState |= B_LEADING_ZERO;
                    cchUsed += 1;
                    s = s.add(1);
                    np.nPwr10 -= 1;
                }
            }
            break;
        } else if in_flags & NUMPRS_LEADING_WHITE != 0 && iswspace(c) != 0 {
            np.dwOutFlags |= NUMPRS_LEADING_WHITE;
            loop {
                cchUsed += 1;
                s = s.add(1);
                if iswspace(*s) == 0 {
                    break;
                }
            }
        } else if in_flags & NUMPRS_THOUSANDS != 0 && is_sep(c) {
            return DISP_E_TYPEMISMATCH; /* Not allowed before the first digit */
        } else if (in_flags as u32 & thousands_currency) == thousands_currency
            && chars.cCurrencyDigitSeparator != 0
            && c == chars.cCurrencyDigitSeparator
        {
            return DISP_E_TYPEMISMATCH; /* Not allowed before the first digit */
        } else if in_flags & NUMPRS_LEADING_PLUS != 0
            && c == chars.cPositiveSymbol
            && np.dwOutFlags & NUMPRS_LEADING_PLUS == 0
        {
            np.dwOutFlags |= NUMPRS_LEADING_PLUS;
            cchUsed += 1;
            s = s.add(1);
        } else if in_flags & NUMPRS_LEADING_MINUS != 0
            && c == chars.cNegativeSymbol
            && np.dwOutFlags & NUMPRS_LEADING_MINUS == 0
        {
            np.dwOutFlags |= NUMPRS_LEADING_MINUS | NUMPRS_NEG;
            cchUsed += 1;
            s = s.add(1);
        } else if in_flags & NUMPRS_CURRENCY != 0
            && np.dwOutFlags & NUMPRS_CURRENCY == 0
            && cur_at(s)
        {
            np.dwOutFlags |= NUMPRS_CURRENCY;
            cchUsed += chars.sCurrencyLen as i32;
            s = s.add(chars.sCurrencyLen as usize);
        } else if in_flags & NUMPRS_PARENS != 0
            && c == b'(' as u16
            && np.dwOutFlags & NUMPRS_PARENS == 0
        {
            np.dwOutFlags |= NUMPRS_PARENS;
            cchUsed += 1;
            s = s.add(1);
        } else {
            break;
        }
    }

    if np.dwOutFlags & (NUMPRS_CURRENCY | NUMPRS_DECIMAL) == 0 && in_flags & NUMPRS_HEX_OCT != 0 && *s == b'&' as u16 {
        let k = *s.add(1);
        if k == b'H' as u16 || k == b'h' as u16 {
            dwState |= B_PROCESSING_HEX;
            np.dwOutFlags |= NUMPRS_HEX_OCT;
            cchUsed += 2;
            s = s.add(2);
        } else if k == b'O' as u16 || k == b'o' as u16 {
            dwState |= B_PROCESSING_OCT;
            np.dwOutFlags |= NUMPRS_HEX_OCT;
            cchUsed += 2;
            s = s.add(2);
        }
    }

    /* Strip Leading zeros */
    while *s == b'0' as u16 {
        dwState |= B_LEADING_ZERO;
        cchUsed += 1;
        s = s.add(1);
    }

    while *s != 0 {
        let c = *s;
        if is_digit(c) {
            if dwState & B_PROCESSING_EXPONENT != 0 {
                let mut exponentSize: i32 = 0;
                if dwState & B_EXPONENT_START != 0 {
                    while *s == b'0' as u16 {
                        /* Skip leading zero's in the exponent */
                        cchUsed += 1;
                        s = s.add(1);
                    }
                }

                while is_digit(*s) {
                    exponentSize = exponentSize.wrapping_mul(10);
                    exponentSize = exponentSize.wrapping_add((*s - b'0' as u16) as i32);
                    cchUsed += 1;
                    s = s.add(1);
                }
                if dwState & B_NEGATIVE_EXPONENT != 0 {
                    exponentSize = -exponentSize;
                }
                /* Add the exponent into the powers of 10 */
                np.nPwr10 += exponentSize;
                dwState &= !(B_PROCESSING_EXPONENT | B_EXPONENT_START);
                s = s.sub(1); /* back up to allow processing of next char */
            } else {
                if np.cDig >= iMaxDigits
                    && dwState & B_PROCESSING_HEX == 0
                    && dwState & B_PROCESSING_OCT == 0
                {
                    np.dwOutFlags |= NUMPRS_INEXACT;

                    if c != b'0' as u16 {
                        dwState &= !B_INEXACT_ZEROS; /* Inexact number with non-trailing zeros */
                    }

                    /* This digit can't be represented, but count it in nPwr10 */
                    if np.dwOutFlags & NUMPRS_DECIMAL != 0 {
                        np.nPwr10 -= 1;
                    } else {
                        np.nPwr10 += 1;
                    }
                } else {
                    if dwState & B_PROCESSING_OCT != 0 && (c == b'8' as u16 || c == b'9' as u16) {
                        break;
                    }

                    if np.dwOutFlags & NUMPRS_DECIMAL != 0 {
                        np.nPwr10 -= 1; /* Count decimal points in nPwr10 */
                    }

                    rgbTmp[np.cDig as usize] = (c - b'0' as u16) as BYTE;
                }
                np.cDig += 1;
                cchUsed += 1;
            }
        } else if in_flags & NUMPRS_THOUSANDS != 0
            && np.dwOutFlags & NUMPRS_HEX_OCT == 0
            && is_sep(c)
        {
            np.dwOutFlags |= NUMPRS_THOUSANDS;
            cchUsed += 1;
        } else if (in_flags as u32 & thousands_currency) == thousands_currency
            && np.dwOutFlags & NUMPRS_HEX_OCT == 0
            && chars.cCurrencyDigitSeparator != 0
            && c == chars.cCurrencyDigitSeparator
        {
            np.dwOutFlags |= NUMPRS_THOUSANDS | NUMPRS_CURRENCY;
            cchUsed += 1;
        } else if in_flags & NUMPRS_DECIMAL != 0
            && (c == chars.cDecimalPoint || c == chars.cCurrencyDecimalPoint)
            && np.dwOutFlags & (NUMPRS_HEX_OCT | NUMPRS_DECIMAL | NUMPRS_EXPONENT) == 0
        {
            np.dwOutFlags |= NUMPRS_DECIMAL;
            if c == chars.cCurrencyDecimalPoint && chars.cDecimalPoint != chars.cCurrencyDecimalPoint {
                np.dwOutFlags |= NUMPRS_CURRENCY;
            }
            cchUsed += 1;

            /* If we have no digits so far, skip leading zeros */
            if np.cDig == 0 {
                while *s.add(1) == b'0' as u16 {
                    dwState |= B_LEADING_ZERO;
                    cchUsed += 1;
                    s = s.add(1);
                    np.nPwr10 -= 1;
                }
            }
        } else if ((b'a' as u16..=b'f' as u16).contains(&c) || (b'A' as u16..=b'F' as u16).contains(&c))
            && dwState & B_PROCESSING_HEX != 0
        {
            if np.cDig >= iMaxDigits {
                return DISP_E_OVERFLOW;
            }
            rgbTmp[np.cDig as usize] = if c >= b'a' as u16 {
                (c - b'a' as u16 + 10) as BYTE
            } else {
                (c - b'A' as u16 + 10) as BYTE
            };
            np.cDig += 1;
            cchUsed += 1;
        } else if (c == b'e' as u16 || c == b'E' as u16)
            && in_flags & NUMPRS_EXPONENT != 0
            && np.dwOutFlags & (NUMPRS_HEX_OCT | NUMPRS_CURRENCY | NUMPRS_EXPONENT) == 0
        {
            dwState |= B_PROCESSING_EXPONENT;
            np.dwOutFlags |= NUMPRS_EXPONENT;
            cchUsed += 1;
        } else if dwState & B_PROCESSING_EXPONENT != 0 && c == chars.cPositiveSymbol {
            cchUsed += 1; /* Ignore positive exponent */
        } else if dwState & B_PROCESSING_EXPONENT != 0 && c == chars.cNegativeSymbol {
            dwState |= B_NEGATIVE_EXPONENT;
            cchUsed += 1;
        } else {
            break; /* Stop at an unrecognised character */
        }

        s = s.add(1);
    }

    if np.cDig == 0 && dwState & B_LEADING_ZERO != 0 {
        /* Ensure a 0 on its own gets stored */
        np.cDig = 1;
        rgbTmp[0] = 0;
    }

    if np.dwOutFlags & NUMPRS_EXPONENT != 0 && dwState & B_PROCESSING_EXPONENT != 0 {
        np.cchUsed = cchUsed;
        return DISP_E_TYPEMISMATCH; /* Failed to completely parse the exponent */
    }

    if np.dwOutFlags & NUMPRS_INEXACT != 0 {
        if dwState & B_INEXACT_ZEROS != 0 {
            np.dwOutFlags &= !NUMPRS_INEXACT; /* All zeros doesn't set NUMPRS_INEXACT */
        }
    } else if in_flags & NUMPRS_HEX_OCT != 0 {
        /* copy all of the digits into the output digit buffer; this is exactly
         * what windows does although it also returns cDig of X and writes X+Y
         * where Y>=0 number of digits to rgbDig */
        core::ptr::copy_nonoverlapping(rgbTmp.as_ptr(), rgbDig, np.cDig as usize);

        if dwState & B_PROCESSING_HEX != 0 {
            /* hex numbers have always the same format */
            np.nPwr10 = 0;
            np.nBaseShift = 4;
        } else if dwState & B_PROCESSING_OCT != 0 {
            /* oct numbers have always the same format */
            np.nPwr10 = 0;
            np.nBaseShift = 3;
        } else {
            while np.cDig > 1 && rgbTmp[np.cDig as usize - 1] == 0 {
                np.nPwr10 += 1;
                np.cDig -= 1;
            }
        }
    } else {
        /* Remove trailing zeros from the last (whole number or decimal) part */
        while np.cDig > 1 && rgbTmp[np.cDig as usize - 1] == 0 {
            np.nPwr10 += 1;
            np.cDig -= 1;
        }
    }

    if np.cDig <= iMaxDigits {
        np.dwOutFlags &= !NUMPRS_INEXACT; /* Ignore stripped zeros for NUMPRS_INEXACT */
    } else {
        np.cDig = iMaxDigits; /* Only return iMaxDigits worth of digits */
    }

    /* Copy the digits we processed into rgbDig */
    core::ptr::copy_nonoverlapping(rgbTmp.as_ptr(), rgbDig, np.cDig as usize);

    /* Consume any trailing symbols and space */
    loop {
        let c = *s;
        if is_sep(c) {
            if in_flags & NUMPRS_THOUSANDS != 0 && np.dwOutFlags & NUMPRS_HEX_OCT == 0 {
                np.dwOutFlags |= NUMPRS_THOUSANDS;
                cchUsed += 1;
                s = s.add(1);
            } else {
                break; /* Not allowed, even with NUMPRS_TRAILING_WHITE */
            }
        } else if c == chars.cCurrencyDigitSeparator {
            if (in_flags as u32 & thousands_currency) == thousands_currency
                && np.dwOutFlags & NUMPRS_HEX_OCT == 0
            {
                np.dwOutFlags |= NUMPRS_THOUSANDS | NUMPRS_CURRENCY;
                cchUsed += 1;
                s = s.add(1);
            } else {
                break; /* Not allowed, even with NUMPRS_TRAILING_WHITE */
            }
        } else if in_flags & NUMPRS_TRAILING_WHITE != 0 && iswspace(c) != 0 {
            np.dwOutFlags |= NUMPRS_TRAILING_WHITE;
            loop {
                cchUsed += 1;
                s = s.add(1);
                if iswspace(*s) == 0 {
                    break;
                }
            }
        } else if in_flags & NUMPRS_TRAILING_PLUS != 0
            && np.dwOutFlags & NUMPRS_LEADING_PLUS == 0
            && c == chars.cPositiveSymbol
        {
            np.dwOutFlags |= NUMPRS_TRAILING_PLUS;
            cchUsed += 1;
            s = s.add(1);
        } else if in_flags & NUMPRS_TRAILING_MINUS != 0
            && np.dwOutFlags & NUMPRS_LEADING_MINUS == 0
            && c == chars.cNegativeSymbol
        {
            np.dwOutFlags |= NUMPRS_TRAILING_MINUS | NUMPRS_NEG;
            cchUsed += 1;
            s = s.add(1);
        } else if in_flags & NUMPRS_PARENS != 0
            && c == b')' as u16
            && np.dwOutFlags & NUMPRS_PARENS != 0
        {
            cchUsed += 1;
            s = s.add(1);
            np.dwOutFlags |= NUMPRS_NEG;
        } else if in_flags & NUMPRS_CURRENCY != 0
            && np.dwOutFlags & NUMPRS_HEX_OCT == 0
            && cur_at(s)
        {
            np.dwOutFlags |= NUMPRS_CURRENCY;
            cchUsed += chars.sCurrencyLen as i32;
            s = s.add(chars.sCurrencyLen as usize);
        } else {
            break;
        }
    }

    if np.dwOutFlags & NUMPRS_PARENS != 0 && np.dwOutFlags & NUMPRS_NEG == 0 {
        np.cchUsed = cchUsed;
        return DISP_E_TYPEMISMATCH; /* Opening parenthesis not matched */
    }

    if in_flags & NUMPRS_USE_ALL != 0 && *s != 0 {
        return DISP_E_TYPEMISMATCH; /* Not all chars were consumed */
    }

    if np.cDig == 0 {
        return DISP_E_TYPEMISMATCH; /* No Number found */
    }

    np.cchUsed = cchUsed;
    S_OK
}
#[no_mangle]
pub unsafe extern "system" fn VarNumFromParseNum(
    pNumprs: *mut NUMPARSE,
    rgbDig: *mut BYTE,
    dwVtBits: ULONG,
    pVarDst: *mut VARIANT,
) -> HRESULT {
    static dblMultipliers: [f64; 11] = [
        1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0,
        1000000.0, 10000000.0, 100000000.0, 1000000000.0, 10000000000.0
    ];
    static dblMinimums: [f64; 11] = [
        R8_MIN, R8_MIN*10.0, R8_MIN*100.0, R8_MIN*1000.0, R8_MIN*10000.0,
        R8_MIN*100000.0, R8_MIN*1000000.0, R8_MIN*10000000.0,
        R8_MIN*100000000.0, R8_MIN*1000000000.0, R8_MIN*10000000000.0
    ];
    static dblMaximums: [f64; 11] = [
        R8_MAX, R8_MAX/10.0, R8_MAX/100.0, R8_MAX/1000.0, R8_MAX/10000.0,
        R8_MAX/100000.0, R8_MAX/1000000.0, R8_MAX/10000000.0,
        R8_MAX/100000000.0, R8_MAX/1000000000.0, R8_MAX/10000000000.0
    ];

    let wholeNumberDigits: i32;
    let fractionalDigits: i32;
    let mut divisor10: i32 = 0;
    let mut multiplier10: i32 = 0;

    TRACE!(b"(%p,%p,0x%x,%p)\n\0".as_ptr() as PCHAR, pNumprs, rgbDig, dwVtBits, pVarDst);

    if (*pNumprs).nBaseShift != 0 {
        let mut ul64: ULONG64 = 0;
        let l64: LONG64;
        let mut i: i32;

        i = 0;
        while i < (*pNumprs).cDig {
            if ul64 > ((UI8_MAX >> (*pNumprs).nBaseShift) - *rgbDig.offset(i as isize) as ULONG64) {
                TRACE!(b"Overflow multiplying digits\n\0".as_ptr() as PCHAR);
                return DISP_E_OVERFLOW;
            }
            ul64 = (ul64 << (*pNumprs).nBaseShift) + *rgbDig.offset(i as isize) as ULONG64;
            i += 1;
        }

        l64 = -(ul64 as LONG64);

        if dwVtBits & VTBIT_I1 != 0 && (ul64 >> 8) == 0 {
            (*V_VT_ref(pVarDst)) = VT_I1;
            (*V_I1_ref(pVarDst)) = ul64 as i8;
            return S_OK;
        } else if dwVtBits & VTBIT_UI1 != 0 && (ul64 >> 8) == 0 {
            (*V_VT_ref(pVarDst)) = VT_UI1;
            (*V_UI1_ref(pVarDst)) = ul64 as u8;
            return S_OK;
        } else if dwVtBits & VTBIT_I2 != 0 && (ul64 >> 16) == 0 {
            (*V_VT_ref(pVarDst)) = VT_I2;
            (*V_I2_ref(pVarDst)) = ul64 as i16;
            return S_OK;
        } else if dwVtBits & VTBIT_UI2 != 0 && (ul64 >> 16) == 0 {
            (*V_VT_ref(pVarDst)) = VT_UI2;
            (*V_UI2_ref(pVarDst)) = ul64 as u16;
            return S_OK;
        } else if dwVtBits & VTBIT_I4 != 0 && FITS_AS_I4(ul64) {
            (*V_VT_ref(pVarDst)) = VT_I4;
            (*V_I4_ref(pVarDst)) = ul64 as i32;
            return S_OK;
        } else if dwVtBits & VTBIT_UI4 != 0 && FITS_AS_I4(ul64) {
            (*V_VT_ref(pVarDst)) = VT_UI4;
            (*V_UI4_ref(pVarDst)) = ul64 as u32;
            return S_OK;
        } else if dwVtBits & VTBIT_I8 != 0 && ((ul64 <= I8_MAX as ULONG64) || (l64 >= I8_MIN)) {
            (*V_VT_ref(pVarDst)) = VT_I8;
            (*V_I8_ref(pVarDst)) = ul64 as i64;
            return S_OK;
        } else if dwVtBits & VTBIT_UI8 != 0 {
            (*V_VT_ref(pVarDst)) = VT_UI8;
            (*V_UI8_ref(pVarDst)) = ul64;
            return S_OK;
        } else if (dwVtBits & VTBIT_DECIMAL) == VTBIT_DECIMAL {
            (*V_VT_ref(pVarDst)) = VT_DECIMAL;
            (*DEC_SIGNSCALE_ref(&mut (*V_DECIMAL_ref(pVarDst)))) = SIGNSCALE(DECIMAL_POS, 0);
            (*DEC_HI32_ref(&mut (*V_DECIMAL_ref(pVarDst)))) = 0;
            (*DEC_LO64_ref(&mut (*V_DECIMAL_ref(pVarDst)))) = ul64;
            return S_OK;
        } else if dwVtBits & VTBIT_R4 != 0 && ((ul64 <= I4_MAX as ULONG64) || (l64 >= I4_MIN as LONG64)) {
            (*V_VT_ref(pVarDst)) = VT_R4;
            if ul64 <= I4_MAX as ULONG64 {
                (*V_R4_ref(pVarDst)) = ul64 as f32;
            } else {
                (*V_R4_ref(pVarDst)) = l64 as f32;
            }
            return S_OK;
        } else if dwVtBits & VTBIT_R8 != 0 && ((ul64 <= I4_MAX as ULONG64) || (l64 >= I4_MIN as LONG64)) {
            (*V_VT_ref(pVarDst)) = VT_R8;
            if ul64 <= I4_MAX as ULONG64 {
                (*V_R8_ref(pVarDst)) = ul64 as f64;
            } else {
                (*V_R8_ref(pVarDst)) = l64 as f64;
            }
            return S_OK;
        }

        TRACE!(b"Overflow: possible return types: 0x%x, value: %s\n\0".as_ptr() as PCHAR, dwVtBits, wine_dbgstr_longlong(ul64));
        return DISP_E_OVERFLOW;
    }

    if (*pNumprs).nPwr10 < 0 {
        if -(*pNumprs).nPwr10 >= (*pNumprs).cDig {
            wholeNumberDigits = 0;
            fractionalDigits = (*pNumprs).cDig;
            divisor10 = -(*pNumprs).nPwr10;
        } else {
            wholeNumberDigits = (*pNumprs).cDig + (*pNumprs).nPwr10;
            fractionalDigits = (*pNumprs).cDig - wholeNumberDigits;
            divisor10 = (*pNumprs).cDig - wholeNumberDigits;
        }
    } else if (*pNumprs).nPwr10 == 0 {
        wholeNumberDigits = (*pNumprs).cDig;
        fractionalDigits = 0;
    } else {
        wholeNumberDigits = (*pNumprs).cDig;
        fractionalDigits = 0;
        multiplier10 = (*pNumprs).nPwr10;
    }

    TRACE!(b"cDig %d; nPwr10 %d, whole %d, frac %d mult %d; div %d\n\0".as_ptr() as PCHAR,
        (*pNumprs).cDig, (*pNumprs).nPwr10, wholeNumberDigits, fractionalDigits,
        multiplier10, divisor10);

    if dwVtBits & (INTEGER_VTBITS | VTBIT_DECIMAL) != 0 &&
        (fractionalDigits == 0 || (dwVtBits & (REAL_VTBITS | VTBIT_DECIMAL)) == 0) {
        let mut bOverflow: BOOL = 0;
        let bNegative: BOOL;
        let mut ul64: ULONG64 = 0;
        let mut i: i32;

        i = 0;
        while i < wholeNumberDigits {
            if ul64 > UI8_MAX / 10 || (ul64 == UI8_MAX / 10 && *rgbDig.offset(i as isize) as ULONG64 > UI8_MAX % 10) {
                TRACE!(b"Overflow multiplying digits\n\0".as_ptr() as PCHAR);
                bOverflow = 1;
                break;
            }
            ul64 = ul64 * 10 + *rgbDig.offset(i as isize) as ULONG64;
            i += 1;
        }

        if bOverflow == 0 && multiplier10 != 0 {
            i = 0;
            while i < multiplier10 {
                if ul64 > (UI8_MAX / 10) {
                    TRACE!(b"Overflow scaling number\n\0".as_ptr() as PCHAR);
                    bOverflow = 1;
                    break;
                }
                ul64 = ul64 * 10;
                i += 1;
            }
        }

        if bOverflow == 0 && fractionalDigits != 0 && divisor10 > 0 {
            let fracDig: *const BYTE = rgbDig.offset(wholeNumberDigits as isize);
            let mut bAdjust: BOOL = 0;

            TRACE!(b"first decimal value is %d\n\0".as_ptr() as PCHAR, *fracDig);

            if *fracDig > 5 {
                bAdjust = 1;
            } else if *fracDig == 5 {
                i = 1;
                while i < fractionalDigits {
                    if *fracDig.offset(i as isize) != 0 {
                        bAdjust = 1;
                        break;
                    }
                    i += 1;
                }
                if i == fractionalDigits && (ul64 & 1) != 0 {
                    bAdjust = 1;
                }
            }

            if bAdjust != 0 {
                if ul64 == UI8_MAX {
                    TRACE!(b"Overflow after rounding\n\0".as_ptr() as PCHAR);
                    bOverflow = 1;
                }
                ul64 += 1;
            }
        }

        bNegative = if (*pNumprs).dwOutFlags & NUMPRS_NEG != 0 && ul64 != 0 { 1 } else { 0 };

        TRACE!(b"Integer value is 0x%s, bNeg %d\n\0".as_ptr() as PCHAR, wine_dbgstr_longlong(ul64), bNegative);

        if bOverflow == 0 && bNegative != 0 {
            if dwVtBits & (VTBIT_I1 | VTBIT_I2 | VTBIT_I4 | VTBIT_I8) != 0 {
                if dwVtBits & VTBIT_I1 != 0 && ul64 <= (-(I1_MIN as i64)) as ULONG64 {
                    (*V_VT_ref(pVarDst)) = VT_I1;
                    (*V_I1_ref(pVarDst)) = -(ul64 as i64) as i8;
                    return S_OK;
                } else if dwVtBits & VTBIT_I2 != 0 && ul64 <= (-(I2_MIN as i64)) as ULONG64 {
                    (*V_VT_ref(pVarDst)) = VT_I2;
                    (*V_I2_ref(pVarDst)) = -(ul64 as i64) as i16;
                    return S_OK;
                } else if dwVtBits & VTBIT_I4 != 0 && ul64 <= (-(I4_MIN as LONGLONG)) as ULONG64 {
                    (*V_VT_ref(pVarDst)) = VT_I4;
                    (*V_I4_ref(pVarDst)) = -(ul64 as i64) as i32;
                    return S_OK;
                } else if dwVtBits & VTBIT_I8 != 0 && ul64 <= (I8_MAX as ULONGLONG + 1) {
                    (*V_VT_ref(pVarDst)) = VT_I8;
                    (*V_I8_ref(pVarDst)) = -(ul64 as i64);
                    return S_OK;
                } else if (dwVtBits & (REAL_VTBITS | VTBIT_DECIMAL)) == VTBIT_DECIMAL {
                    (*V_VT_ref(pVarDst)) = VT_DECIMAL;
                    (*DEC_SIGNSCALE_ref(&mut (*V_DECIMAL_ref(pVarDst)))) = SIGNSCALE(DECIMAL_NEG, 0);
                    (*DEC_HI32_ref(&mut (*V_DECIMAL_ref(pVarDst)))) = 0;
                    (*DEC_LO64_ref(&mut (*V_DECIMAL_ref(pVarDst)))) = -(ul64 as i64) as u64;
                    return S_OK;
                }
            }
        } else if bOverflow == 0 {
            if dwVtBits & VTBIT_I1 != 0 && ul64 <= I1_MAX as ULONG64 {
                (*V_VT_ref(pVarDst)) = VT_I1;
                (*V_I1_ref(pVarDst)) = ul64 as i8;
                return S_OK;
            } else if dwVtBits & VTBIT_UI1 != 0 && ul64 <= UI1_MAX as ULONG64 {
                (*V_VT_ref(pVarDst)) = VT_UI1;
                (*V_UI1_ref(pVarDst)) = ul64 as u8;
                return S_OK;
            } else if dwVtBits & VTBIT_I2 != 0 && ul64 <= I2_MAX as ULONG64 {
                (*V_VT_ref(pVarDst)) = VT_I2;
                (*V_I2_ref(pVarDst)) = ul64 as i16;
                return S_OK;
            } else if dwVtBits & VTBIT_UI2 != 0 && ul64 <= UI2_MAX as ULONG64 {
                (*V_VT_ref(pVarDst)) = VT_UI2;
                (*V_UI2_ref(pVarDst)) = ul64 as u16;
                return S_OK;
            } else if dwVtBits & VTBIT_I4 != 0 && ul64 <= I4_MAX as ULONG64 {
                (*V_VT_ref(pVarDst)) = VT_I4;
                (*V_I4_ref(pVarDst)) = ul64 as i32;
                return S_OK;
            } else if dwVtBits & VTBIT_UI4 != 0 && ul64 <= UI4_MAX as ULONG64 {
                (*V_VT_ref(pVarDst)) = VT_UI4;
                (*V_UI4_ref(pVarDst)) = ul64 as u32;
                return S_OK;
            } else if dwVtBits & VTBIT_I8 != 0 && ul64 <= I8_MAX as ULONG64 {
                (*V_VT_ref(pVarDst)) = VT_I8;
                (*V_I8_ref(pVarDst)) = ul64 as i64;
                return S_OK;
            } else if dwVtBits & VTBIT_UI8 != 0 {
                (*V_VT_ref(pVarDst)) = VT_UI8;
                (*V_UI8_ref(pVarDst)) = ul64;
                return S_OK;
            } else if (dwVtBits & (REAL_VTBITS | VTBIT_DECIMAL)) == VTBIT_DECIMAL {
                (*V_VT_ref(pVarDst)) = VT_DECIMAL;
                (*DEC_SIGNSCALE_ref(&mut (*V_DECIMAL_ref(pVarDst)))) = SIGNSCALE(DECIMAL_POS, 0);
                (*DEC_HI32_ref(&mut (*V_DECIMAL_ref(pVarDst)))) = 0;
                (*DEC_LO64_ref(&mut (*V_DECIMAL_ref(pVarDst)))) = ul64;
                return S_OK;
            }
        }
    }

    if dwVtBits & REAL_VTBITS != 0 {
        let mut bOverflow: BOOL = 0;
        let bNegative: BOOL = if (*pNumprs).dwOutFlags & NUMPRS_NEG != 0 { 1 } else { 0 };
        let mut whole: f64 = 0.0;
        let mut i: i32;
        let mut dwVtBits = dwVtBits;

        i = 0;
        while i < (*pNumprs).cDig {
            whole = whole * 10.0 + *rgbDig.offset(i as isize) as f64;
            i += 1;
        }

        TRACE!(b"Whole double value is %16.16g\n\0".as_ptr() as PCHAR, whole);

        while multiplier10 > 10 {
            if whole > dblMaximums[10] {
                dwVtBits &= !(VTBIT_R4 | VTBIT_R8 | VTBIT_CY);
                bOverflow = 1;
                break;
            }
            whole = whole * dblMultipliers[10];
            multiplier10 -= 10;
        }
        if multiplier10 != 0 && bOverflow == 0 {
            if whole > dblMaximums[multiplier10 as usize] {
                dwVtBits &= !(VTBIT_R4 | VTBIT_R8 | VTBIT_CY);
                bOverflow = 1;
            } else {
                whole = whole * dblMultipliers[multiplier10 as usize];
            }
        }

        if bOverflow == 0 {
            TRACE!(b"Scaled double value is %16.16g\n\0".as_ptr() as PCHAR, whole);
        }

        while divisor10 > 10 && bOverflow == 0 {
            if whole < dblMinimums[10] && whole != 0.0 {
                whole = 0.0;
                divisor10 = 0;
                break;
            }
            whole = whole / dblMultipliers[10];
            divisor10 -= 10;
        }
        if divisor10 != 0 && bOverflow == 0 {
            if whole < dblMinimums[divisor10 as usize] && whole != 0.0 {
                whole = 0.0;
                divisor10 = 0;
            } else {
                whole = whole / dblMultipliers[divisor10 as usize];
            }
        }
        if bOverflow == 0 {
            TRACE!(b"Final double value is %16.16g\n\0".as_ptr() as PCHAR, whole);
        }

        if dwVtBits & VTBIT_R4 != 0 &&
            ((whole <= R4_MAX as f64 && whole >= R4_MIN as f64) || whole == 0.0) {
            TRACE!(b"Set R4 to final value\n\0".as_ptr() as PCHAR);
            (*V_VT_ref(pVarDst)) = VT_R4;
            (*V_R4_ref(pVarDst)) = if (*pNumprs).dwOutFlags & NUMPRS_NEG != 0 { -whole } else { whole } as f32;
            return S_OK;
        }

        if dwVtBits & VTBIT_R8 != 0 {
            TRACE!(b"Set R8 to final value\n\0".as_ptr() as PCHAR);
            (*V_VT_ref(pVarDst)) = VT_R8;
            (*V_R8_ref(pVarDst)) = if (*pNumprs).dwOutFlags & NUMPRS_NEG != 0 { -whole } else { whole };
            return S_OK;
        }

        if dwVtBits & VTBIT_CY != 0 {
            if SUCCEEDED(VarCyFromR8(if bNegative != 0 { -whole } else { whole }, &mut (*V_CY_ref(pVarDst)))) {
                (*V_VT_ref(pVarDst)) = VT_CY;
                TRACE!(b"Set CY to final value\n\0".as_ptr() as PCHAR);
                return S_OK;
            }
            TRACE!(b"Value Overflows CY\n\0".as_ptr() as PCHAR);
        }
    }

    if dwVtBits & VTBIT_DECIMAL != 0 {
        let mut i: i32;
        let mut carry: ULONG;
        let mut tmp: ULONG64;
        let pDec: *mut DECIMAL = &mut (*V_DECIMAL_ref(pVarDst));

        DECIMAL_SETZERO(pDec);
        (*DEC_LO32_ref(pDec)) = 0;

        if (*pNumprs).dwOutFlags & NUMPRS_NEG != 0 {
            (*pDec).__bindgen_anon_1.__bindgen_anon_1.sign = DECIMAL_NEG;
        } else {
            (*pDec).__bindgen_anon_1.__bindgen_anon_1.sign = DECIMAL_POS;
        }

        i = 0;
        while i < (*pNumprs).cDig {
            tmp = (*DEC_LO32_ref(pDec)) as ULONG64 * 10 + *rgbDig.offset(i as isize) as ULONG64;
            carry = (tmp >> 32) as ULONG;
            (*DEC_LO32_ref(pDec)) = (tmp & UI4_MAX as ULONG64) as ULONG;
            tmp = (*DEC_MID32_ref(pDec)) as ULONG64 * 10 + carry as ULONG64;
            carry = (tmp >> 32) as ULONG;
            (*DEC_MID32_ref(pDec)) = (tmp & UI4_MAX as ULONG64) as ULONG;
            tmp = (*DEC_HI32_ref(pDec)) as ULONG64 * 10 + carry as ULONG64;
            (*DEC_HI32_ref(pDec)) = (tmp & UI4_MAX as ULONG64) as ULONG;

            if (tmp >> 32) & UI4_MAX as ULONG64 != 0 {
                TRACE!(b"Overflow\n\0".as_ptr() as PCHAR);
                (*DEC_LO32_ref(pDec)) = UI4_MAX as u32;
                (*DEC_MID32_ref(pDec)) = UI4_MAX as u32;
                (*DEC_HI32_ref(pDec)) = UI4_MAX as u32;
                return DISP_E_OVERFLOW;
            }
            i += 1;
        }

        while multiplier10 > 0 {
            tmp = (*DEC_LO32_ref(pDec)) as ULONG64 * 10;
            carry = (tmp >> 32) as ULONG;
            (*DEC_LO32_ref(pDec)) = (tmp & UI4_MAX as ULONG64) as ULONG;
            tmp = (*DEC_MID32_ref(pDec)) as ULONG64 * 10 + carry as ULONG64;
            carry = (tmp >> 32) as ULONG;
            (*DEC_MID32_ref(pDec)) = (tmp & UI4_MAX as ULONG64) as ULONG;
            tmp = (*DEC_HI32_ref(pDec)) as ULONG64 * 10 + carry as ULONG64;
            (*DEC_HI32_ref(pDec)) = (tmp & UI4_MAX as ULONG64) as ULONG;

            if (tmp >> 32) & UI4_MAX as ULONG64 != 0 {
                TRACE!(b"Overflow\n\0".as_ptr() as PCHAR);
                (*DEC_LO32_ref(pDec)) = UI4_MAX as u32;
                (*DEC_MID32_ref(pDec)) = UI4_MAX as u32;
                (*DEC_HI32_ref(pDec)) = UI4_MAX as u32;
                return DISP_E_OVERFLOW;
            }
            multiplier10 -= 1;
        }
        (*pDec).__bindgen_anon_1.__bindgen_anon_1.scale = divisor10 as BYTE;

        (*V_VT_ref(pVarDst)) = VT_DECIMAL;
        return S_OK;
    }
    DISP_E_OVERFLOW
}

#[no_mangle]
pub unsafe extern "system" fn VarCat(left: LPVARIANT, right: LPVARIANT, out: LPVARIANT) -> HRESULT {
    let mut left_str: BSTR = core::ptr::null_mut();
    let mut right_str: BSTR = core::ptr::null_mut();
    let leftvt: VARTYPE;
    let rightvt: VARTYPE;
    let mut hres: HRESULT;

    TRACE!(b"%s,%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(left), debugstr_variant(right), out);

    leftvt = *V_VT_ref(left) ;
    rightvt = *V_VT_ref(right) ;

    if leftvt == VT_NULL && rightvt == VT_NULL {
        (*V_VT_ref(out)) = VT_NULL;
        return S_OK;
    }

    if leftvt == VT_VARIANT && (rightvt == VT_ERROR ||
        rightvt == VT_DATE || rightvt == VT_DECIMAL) {
        hres = DISP_E_TYPEMISMATCH;
    } else if (leftvt == VT_I2 || leftvt == VT_I4 ||
        leftvt == VT_R4 || leftvt == VT_R8 ||
        leftvt == VT_CY || leftvt == VT_BOOL ||
        leftvt == VT_BSTR || leftvt == VT_I1 ||
        leftvt == VT_UI1 || leftvt == VT_UI2 ||
        leftvt == VT_UI4 || leftvt == VT_I8 ||
        leftvt == VT_UI8 || leftvt == VT_INT ||
        leftvt == VT_UINT || leftvt == VT_EMPTY ||
        leftvt == VT_NULL || leftvt == VT_DATE ||
        leftvt == VT_DECIMAL || leftvt == VT_DISPATCH)
        &&
        (rightvt == VT_I2 || rightvt == VT_I4 ||
        rightvt == VT_R4 || rightvt == VT_R8 ||
        rightvt == VT_CY || rightvt == VT_BOOL ||
        rightvt == VT_BSTR || rightvt == VT_I1 ||
        rightvt == VT_UI1 || rightvt == VT_UI2 ||
        rightvt == VT_UI4 || rightvt == VT_I8 ||
        rightvt == VT_UI8 || rightvt == VT_INT ||
        rightvt == VT_UINT || rightvt == VT_EMPTY ||
        rightvt == VT_NULL || rightvt == VT_DATE ||
        rightvt == VT_DECIMAL || rightvt == VT_DISPATCH) {
        hres = S_OK;
    } else if rightvt == VT_ERROR && leftvt < VT_VOID {
        hres = DISP_E_TYPEMISMATCH;
    } else if leftvt == VT_ERROR && (rightvt == VT_DATE ||
        rightvt == VT_ERROR || rightvt == VT_DECIMAL) {
        hres = DISP_E_TYPEMISMATCH;
    } else if rightvt == VT_DATE || rightvt == VT_ERROR ||
        rightvt == VT_DECIMAL {
        hres = DISP_E_BADVARTYPE;
    } else if leftvt == VT_ERROR || rightvt == VT_ERROR {
        hres = DISP_E_TYPEMISMATCH;
    } else if leftvt == VT_VARIANT {
        hres = DISP_E_TYPEMISMATCH;
    } else if rightvt == VT_VARIANT && (leftvt == VT_EMPTY ||
        leftvt == VT_NULL || leftvt ==  VT_I2 ||
        leftvt == VT_I4 || leftvt == VT_R4 ||
        leftvt == VT_R8 || leftvt == VT_CY ||
        leftvt == VT_DATE || leftvt == VT_BSTR ||
        leftvt == VT_BOOL ||  leftvt == VT_DECIMAL ||
        leftvt == VT_I1 || leftvt == VT_UI1 ||
        leftvt == VT_UI2 || leftvt == VT_UI4 ||
        leftvt == VT_I8 || leftvt == VT_UI8 ||
        leftvt == VT_INT || leftvt == VT_UINT) {
        hres = DISP_E_TYPEMISMATCH;
    } else {
        hres = DISP_E_BADVARTYPE;
    }

    if hres != S_OK {
        (*V_VT_ref(out)) = VT_EMPTY;
        return hres;
    }

    if leftvt == VT_BSTR {
        left_str = *V_BSTR_ref(left) ;
    } else {
        let mut converted: VARIANTARG = core::mem::zeroed();
        let mut tmp: LPVARIANT = left;

        VariantInit(&mut converted);
        if leftvt == VT_DISPATCH {
            hres = VARIANT_FetchDispatchValue(left, &mut converted);
            if FAILED(hres) {
                if (*V_VT_ref(left)) != VT_BSTR {
                    SysFreeString(left_str);
                }
                if (*V_VT_ref(right)) != VT_BSTR {
                    SysFreeString(right_str);
                }
                return hres;
            }

            tmp = &mut converted;
        }

        hres = VariantChangeTypeEx(&mut converted, tmp, 0, (VARIANT_ALPHABOOL | VARIANT_LOCALBOOL) as USHORT, VT_BSTR);
        if SUCCEEDED(hres) {
            left_str = *V_BSTR_ref(&mut converted) ;
        } else if hres != DISP_E_TYPEMISMATCH {
            VariantClear(&mut converted);
            if (*V_VT_ref(left)) != VT_BSTR {
                SysFreeString(left_str);
            }
            if (*V_VT_ref(right)) != VT_BSTR {
                SysFreeString(right_str);
            }
            return hres;
        }
    }

    if rightvt == VT_BSTR {
        right_str = *V_BSTR_ref(right) ;
    } else {
        let mut converted: VARIANTARG = core::mem::zeroed();
        let mut tmp: LPVARIANT = right;

        VariantInit(&mut converted);
        if rightvt == VT_DISPATCH {
            hres = VARIANT_FetchDispatchValue(right, &mut converted);
            if FAILED(hres) {
                if (*V_VT_ref(left)) != VT_BSTR {
                    SysFreeString(left_str);
                }
                if (*V_VT_ref(right)) != VT_BSTR {
                    SysFreeString(right_str);
                }
                return hres;
            }

            tmp = &mut converted;
        }

        hres = VariantChangeTypeEx(&mut converted, tmp, 0, (VARIANT_ALPHABOOL | VARIANT_LOCALBOOL) as USHORT, VT_BSTR);
        if SUCCEEDED(hres) {
            right_str = *V_BSTR_ref(&mut converted) ;
        } else if hres != DISP_E_TYPEMISMATCH {
            VariantClear(&mut converted);
            if (*V_VT_ref(left)) != VT_BSTR {
                SysFreeString(left_str);
            }
            if (*V_VT_ref(right)) != VT_BSTR {
                SysFreeString(right_str);
            }
            return hres;
        }
    }

    (*V_VT_ref(out)) = VT_BSTR;
    hres = VarBstrCat(left_str, right_str, core::ptr::addr_of_mut!((*V_BSTR_ref(out))));

    if (*V_VT_ref(left)) != VT_BSTR {
        SysFreeString(left_str);
    }
    if (*V_VT_ref(right)) != VT_BSTR {
        SysFreeString(right_str);
    }
    hres
}

unsafe fn _VarChangeTypeExWrap(
    pvargDest: *mut VARIANTARG,
    pvargSrc: *mut VARIANTARG,
    lcid: LCID,
    wFlags: USHORT,
    vt: VARTYPE,
) -> HRESULT {
    let mut vtmpsrc = *pvargSrc;
    (*V_VT_ref(&mut vtmpsrc)) &= !VT_RESERVED;
    VariantChangeTypeEx(pvargDest, &mut vtmpsrc, lcid, wFlags, vt)
}

#[no_mangle]
pub unsafe extern "system" fn VarCmp(
    left: *mut VARIANT,
    right: *mut VARIANT,
    lcid: LCID,
    flags: DWORD,
) -> HRESULT {
    let lvt: VARTYPE;
    let rvt: VARTYPE;
    let mut vt: VARTYPE;
    let mut rv: VARIANT = core::mem::zeroed();
    let mut lv: VARIANT = core::mem::zeroed();
    let xmask: DWORD;
    let mut rc: HRESULT;

    TRACE!(
        b"(%s,%s,0x%08x,0x%08x)\n\0".as_ptr() as *const i8,
        debugstr_variant(left),
        debugstr_variant(right),
        lcid,
        flags,
    );

    lvt = (*V_VT_ref(left)) & VT_TYPEMASK;
    rvt = (*V_VT_ref(right)) & VT_TYPEMASK;
    xmask = (1 << lvt) | (1 << rvt);

    if (((*V_VT_ref(left)) | (*V_VT_ref(right))) & !VT_TYPEMASK & !VT_RESERVED != 0)
        || lvt > VT_INT
        || rvt > VT_I8
    {
        return DISP_E_BADVARTYPE;
    }

    if rvt == VT_INT
        || (xmask
            & (VTBIT_I1
                | VTBIT_UI2
                | VTBIT_UI4
                | VTBIT_UI8
                | VTBIT_DISPATCH
                | VTBIT_VARIANT
                | VTBIT_UNKNOWN
                | VTBIT_15)
            != 0)
    {
        return DISP_E_TYPEMISMATCH;
    }

    if xmask == VTBIT_ERROR {
        return VARCMP_EQ;
    } else if xmask & VTBIT_ERROR != 0 {
        return DISP_E_TYPEMISMATCH;
    }

    if xmask & VTBIT_NULL != 0 {
        return VARCMP_NULL;
    }

    VariantInit(&mut lv);
    VariantInit(&mut rv);

    if xmask == VTBIT_BSTR {
        return VarBstrCmp(*V_BSTR_ref(left) , *V_BSTR_ref(right) , lcid, flags);
    }

    if xmask & VTBIT_BSTR != 0 {
        let bstrv: *mut VARIANT;
        let nonbv: *mut VARIANT;
        let nonbvt: VARTYPE;
        let mut swap = 0;

        if lvt == VT_BSTR {
            bstrv = left;
            nonbv = right;
            nonbvt = rvt;
        } else {
            swap = 1;
            bstrv = right;
            nonbv = left;
            nonbvt = lvt;
        }

        if nonbvt == VT_EMPTY {
            rc = if (*V_BSTR_ref(bstrv)).is_null() || *(*V_BSTR_ref(bstrv)) == 0 {
                VARCMP_EQ
            } else {
                VARCMP_GT
            };
        } else {
            let breserv = (*V_VT_ref(bstrv)) & !VT_TYPEMASK;
            let nreserv = (*V_VT_ref(nonbv)) & !VT_TYPEMASK;

            if breserv == 0 && nreserv == 0 {
                rc = VARCMP_GT;
            } else if breserv != 0 && nreserv == 0 {
                rc = VariantChangeTypeEx(&mut rv, nonbv, lcid, 0, VT_BSTR);
                if FAILED(rc) {
                    return rc;
                }
                rc = VarBstrCmp(*V_BSTR_ref(bstrv) , *V_BSTR_ref(&mut rv) , lcid, flags);
                VariantClear(&mut rv);
            } else if !(*V_BSTR_ref(bstrv)).is_null() && *(*V_BSTR_ref(bstrv)) != 0 {
                rc = _VarChangeTypeExWrap(&mut lv, bstrv, lcid, 0, VT_R8);
                if FAILED(rc) {
                    rc = VARCMP_GT;
                } else if breserv != 0 && nreserv != 0 {
                    rc = if (*V_R8_ref(&mut lv)) >= 0.0 {
                        VARCMP_GT
                    } else {
                        VARCMP_LT
                    };
                } else {
                    rc = VARCMP_NULL;
                }
                VariantClear(&mut lv);
                VariantClear(&mut rv);
            } else {
                rc = VARCMP_GT;
            }
        }

        if swap != 0 {
            if rc == VARCMP_GT {
                rc = VARCMP_LT;
            } else if rc == VARCMP_LT {
                rc = VARCMP_GT;
            }
        }
        if rc != VARCMP_NULL {
            return rc;
        }
    }

    if xmask & VTBIT_DECIMAL != 0 {
        vt = VT_DECIMAL;
    } else if xmask & VTBIT_BSTR != 0 {
        vt = VT_R8;
    } else if xmask & VTBIT_R4 != 0 {
        vt = VT_R4;
    } else if xmask & (VTBIT_R8 | VTBIT_DATE) != 0 {
        vt = VT_R8;
    } else if xmask & VTBIT_CY != 0 {
        vt = VT_CY;
    } else {
        vt = VT_I8;
    }

    rc = _VarChangeTypeExWrap(&mut lv, left, lcid, 0, vt);
    if rc == DISP_E_OVERFLOW && vt != VT_R8 {
        vt = VT_R8;
        rc = _VarChangeTypeExWrap(&mut lv, left, lcid, 0, vt);
    }
    if FAILED(rc) {
        return rc;
    }
    rc = _VarChangeTypeExWrap(&mut rv, right, lcid, 0, vt);
    if rc == DISP_E_OVERFLOW && vt != VT_R8 {
        vt = VT_R8;
        rc = _VarChangeTypeExWrap(&mut lv, left, lcid, 0, vt);
        if FAILED(rc) {
            return rc;
        }
        rc = _VarChangeTypeExWrap(&mut rv, right, lcid, 0, vt);
    }
    if FAILED(rc) {
        return rc;
    }

    match vt {
        VT_CY => VarCyCmp(*V_CY_ref(&mut lv) , *V_CY_ref(&mut rv) ),
        VT_DECIMAL => VarDecCmp(V_DECIMAL_ref(&mut lv), V_DECIMAL_ref(&mut rv)),
        VT_I8 => {
            let a = *V_I8_ref(&mut lv) ;
            let b = *V_I8_ref(&mut rv) ;
            if a == b {
                VARCMP_EQ
            } else if a < b {
                VARCMP_LT
            } else {
                VARCMP_GT
            }
        }
        VT_R4 => {
            let a = *V_R4_ref(&mut lv) ;
            let b = *V_R4_ref(&mut rv) ;
            if a == b {
                VARCMP_EQ
            } else if a < b {
                VARCMP_LT
            } else {
                VARCMP_GT
            }
        }
        VT_R8 => {
            let a = *V_R8_ref(&mut lv) ;
            let b = *V_R8_ref(&mut rv) ;
            if a == b {
                VARCMP_EQ
            } else if a < b {
                VARCMP_LT
            } else {
                VARCMP_GT
            }
        }
        _ => E_FAIL,
    }
}

#[no_mangle]
pub unsafe extern "system" fn VarAnd(left: LPVARIANT, right: LPVARIANT, result: LPVARIANT) -> HRESULT {
    let mut hres: HRESULT = S_OK;
    let mut resvt: VARTYPE;
    let leftvt: VARTYPE;
    let rightvt: VARTYPE;
    let rightExtraFlags: VARTYPE;
    let leftExtraFlags: VARTYPE;
    let ExtraFlags: VARTYPE;
    let mut varLeft: VARIANT = core::mem::zeroed();
    let mut varRight: VARIANT = core::mem::zeroed();
    let mut tempLeft: VARIANT = core::mem::zeroed();
    let mut tempRight: VARIANT = core::mem::zeroed();
    let mut left_ptr = left;
    let mut right_ptr = right;

    VariantInit(&mut varLeft);
    VariantInit(&mut varRight);
    VariantInit(&mut tempLeft);
    VariantInit(&mut tempRight);

    TRACE!(b"(%s,%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(left), debugstr_variant(right), result);

    'VarAnd_Exit: {
    if ((*V_VT_ref(left)) & VT_TYPEMASK) == VT_DISPATCH {
        hres = VARIANT_FetchDispatchValue(left, &mut tempLeft);
        if FAILED(hres) { break 'VarAnd_Exit; }
        left_ptr = &mut tempLeft;
    }
    if ((*V_VT_ref(right)) & VT_TYPEMASK) == VT_DISPATCH {
        hres = VARIANT_FetchDispatchValue(right, &mut tempRight);
        if FAILED(hres) { break 'VarAnd_Exit; }
        right_ptr = &mut tempRight;
    }

    leftvt = (*V_VT_ref(left_ptr)) & VT_TYPEMASK;
    rightvt = (*V_VT_ref(right_ptr)) & VT_TYPEMASK;
    leftExtraFlags = (*V_VT_ref(left_ptr)) & (!VT_TYPEMASK);
    rightExtraFlags = (*V_VT_ref(right_ptr)) & (!VT_TYPEMASK);

    if leftExtraFlags != rightExtraFlags {
        hres = DISP_E_BADVARTYPE;
        break 'VarAnd_Exit;
    }
    ExtraFlags = leftExtraFlags;

    if (leftvt == VT_I8 && rightvt == VT_INT) ||
       (leftvt == VT_INT && rightvt == VT_I8) ||
       ExtraFlags != 0 {
        hres = DISP_E_BADVARTYPE;
        break 'VarAnd_Exit;
    }

    if leftvt == VT_I8 || rightvt == VT_I8 {
        resvt = VT_I8;
    } else if leftvt == VT_I4 || rightvt == VT_I4 ||
        leftvt == VT_UINT || rightvt == VT_UINT ||
        leftvt == VT_INT || rightvt == VT_INT ||
        leftvt == VT_R4 || rightvt == VT_R4 ||
        leftvt == VT_R8 || rightvt == VT_R8 ||
        leftvt == VT_CY || rightvt == VT_CY ||
        leftvt == VT_DATE || rightvt == VT_DATE ||
        leftvt == VT_I1 || rightvt == VT_I1 ||
        leftvt == VT_UI2 || rightvt == VT_UI2 ||
        leftvt == VT_UI4 || rightvt == VT_UI4 ||
        leftvt == VT_UI8 || rightvt == VT_UI8 ||
        leftvt == VT_DECIMAL || rightvt == VT_DECIMAL {
        resvt = VT_I4;
    } else if leftvt == VT_UI1 || rightvt == VT_UI1 ||
        leftvt == VT_I2 || rightvt == VT_I2 ||
        leftvt == VT_EMPTY || rightvt == VT_EMPTY {
        if (leftvt == VT_NULL && rightvt == VT_UI1) ||
            (leftvt == VT_UI1 && rightvt == VT_NULL) ||
            (leftvt == VT_UI1 && rightvt == VT_UI1) {
            resvt = VT_UI1;
        } else {
            resvt = VT_I2;
        }
    } else if leftvt == VT_BOOL || rightvt == VT_BOOL ||
        (leftvt == VT_BSTR && rightvt == VT_BSTR) {
        resvt = VT_BOOL;
    } else if leftvt == VT_NULL || rightvt == VT_NULL ||
        leftvt == VT_BSTR || rightvt == VT_BSTR {
        resvt = VT_NULL;
    } else {
        hres = DISP_E_BADVARTYPE;
        break 'VarAnd_Exit;
    }

    if leftvt == VT_NULL || rightvt == VT_NULL {
        if leftvt == VT_NULL {
            let mut b: VARIANT_BOOL = 0;
            match rightvt {
                VT_I1 => { if (*V_I1_ref(right_ptr)) != 0 { resvt = VT_NULL; } }
                VT_UI1 => { if (*V_UI1_ref(right_ptr)) != 0 { resvt = VT_NULL; } }
                VT_I2 => { if (*V_I2_ref(right_ptr)) != 0 { resvt = VT_NULL; } }
                VT_UI2 => { if (*V_UI2_ref(right_ptr)) != 0 { resvt = VT_NULL; } }
                VT_I4 => { if (*V_I4_ref(right_ptr)) != 0 { resvt = VT_NULL; } }
                VT_UI4 => { if (*V_UI4_ref(right_ptr)) != 0 { resvt = VT_NULL; } }
                VT_I8 => { if (*V_I8_ref(right_ptr)) != 0 { resvt = VT_NULL; } }
                VT_UI8 => { if (*V_UI8_ref(right_ptr)) != 0 { resvt = VT_NULL; } }
                VT_INT => { if (*V_INT_ref(right_ptr)) != 0 { resvt = VT_NULL; } }
                VT_UINT => { if (*V_UINT_ref(right_ptr)) != 0 { resvt = VT_NULL; } }
                VT_BOOL => { if (*V_BOOL_ref(right_ptr)) != 0 { resvt = VT_NULL; } }
                VT_R4 => { if (*V_R4_ref(right_ptr)) != 0.0 { resvt = VT_NULL; } }
                VT_R8 => { if (*V_R8_ref(right_ptr)) != 0.0 { resvt = VT_NULL; } }
                VT_CY => {
                    if (*V_CY_ref(right_ptr)).int64 != 0 {
                        resvt = VT_NULL;
                    }
                }
                VT_DECIMAL => {
                    if (*DEC_HI32_ref(V_DECIMAL_ref(right_ptr))) != 0 ||
                        (*DEC_LO64_ref(V_DECIMAL_ref(right_ptr))) != 0 {
                        resvt = VT_NULL;
                    }
                }
                VT_BSTR => {
                    hres = VarBoolFromStr(*V_BSTR_ref(right_ptr) ,
                        LOCALE_USER_DEFAULT, VAR_LOCALBOOL, &mut b);
                    if FAILED(hres) {
                        return hres;
                    } else if b != 0 {
                        (*V_VT_ref(result)) = VT_NULL;
                    } else {
                        (*V_VT_ref(result)) = VT_BOOL;
                        (*V_BOOL_ref(result)) = b;
                    }
                    break 'VarAnd_Exit;
                }
                _ => {}
            }
        }
        (*V_VT_ref(result)) = resvt;
        break 'VarAnd_Exit;
    }

    hres = VariantCopy(&mut varLeft, left_ptr);
    if FAILED(hres) { break 'VarAnd_Exit; }

    hres = VariantCopy(&mut varRight, right_ptr);
    if FAILED(hres) { break 'VarAnd_Exit; }

    if resvt == VT_I4 && (*V_VT_ref(&mut varLeft)) == VT_UI4 {
        (*V_VT_ref(&mut varLeft)) = VT_I4;
    } else {
        let mut d: f64 = 0.0;

        if (*V_VT_ref(&mut varLeft)) == VT_BSTR &&
            FAILED(VarR8FromStr(*V_BSTR_ref(&mut varLeft) ,
            LOCALE_USER_DEFAULT, 0, &mut d)) {
            hres = VariantChangeType(&mut varLeft, &mut varLeft,
                VARIANT_LOCALBOOL as USHORT, VT_BOOL);
        }
        if SUCCEEDED(hres) && (*V_VT_ref(&mut varLeft)) != resvt {
            hres = VariantChangeType(&mut varLeft, &mut varLeft, 0, resvt);
        }
        if FAILED(hres) { break 'VarAnd_Exit; }
    }

    if resvt == VT_I4 && (*V_VT_ref(&mut varRight)) == VT_UI4 {
        (*V_VT_ref(&mut varRight)) = VT_I4;
    } else {
        let mut d: f64 = 0.0;

        if (*V_VT_ref(&mut varRight)) == VT_BSTR &&
            FAILED(VarR8FromStr(*V_BSTR_ref(&mut varRight) ,
            LOCALE_USER_DEFAULT, 0, &mut d)) {
            hres = VariantChangeType(&mut varRight, &mut varRight,
                VARIANT_LOCALBOOL as USHORT, VT_BOOL);
        }
        if SUCCEEDED(hres) && (*V_VT_ref(&mut varRight)) != resvt {
            hres = VariantChangeType(&mut varRight, &mut varRight, 0, resvt);
        }
        if FAILED(hres) { break 'VarAnd_Exit; }
    }

    (*V_VT_ref(result)) = resvt;
    match resvt {
        VT_I8 => {
            (*V_I8_ref(result)) = (*V_I8_ref(&mut varLeft)) & (*V_I8_ref(&mut varRight));
        }
        VT_I4 => {
            (*V_I4_ref(result)) = (*V_I4_ref(&mut varLeft)) & (*V_I4_ref(&mut varRight));
        }
        VT_I2 => {
            (*V_I2_ref(result)) = (*V_I2_ref(&mut varLeft)) & (*V_I2_ref(&mut varRight));
        }
        VT_UI1 => {
            (*V_UI1_ref(result)) = (*V_UI1_ref(&mut varLeft)) & (*V_UI1_ref(&mut varRight));
        }
        VT_BOOL => {
            (*V_BOOL_ref(result)) = (*V_BOOL_ref(&mut varLeft)) & (*V_BOOL_ref(&mut varRight));
        }
        _ => {
            FIXME!(b"Couldn't bitwise AND variant types %d,%d\n\0".as_ptr() as *const i8,
                leftvt, rightvt);
        }
    }

    } // 'VarAnd_Exit

    VariantClear(&mut varLeft);
    VariantClear(&mut varRight);
    VariantClear(&mut tempLeft);
    VariantClear(&mut tempRight);

    hres
}

#[no_mangle]
pub unsafe extern "system" fn VarAdd(mut left: LPVARIANT, mut right: LPVARIANT, result: LPVARIANT) -> HRESULT {
    let mut hres: HRESULT;
    let lvt: VARTYPE;
    let rvt: VARTYPE;
    let mut resvt: VARTYPE;
    let mut tvt: VARTYPE;
    let mut lv: VARIANT = core::mem::zeroed();
    let mut rv: VARIANT = core::mem::zeroed();
    let mut tv: VARIANT = core::mem::zeroed();
    let mut tempLeft: VARIANT = core::mem::zeroed();
    let mut tempRight: VARIANT = core::mem::zeroed();
    let r8res: f64;

    // variant.c:3140 declares `enum coerceprio` FUNCTION-LOCAL; the sibling ladder
    // below (variant.c:3353) is a DIFFERENT enum that reuses the same member names
    // with different values. Both are faithful -- verified member-for-member against
    // their own C enum -- so the cross-value disagreement is correct, not a bug.
    const VT_EMPTY_PRIO: u32 = 0;  // nolint: const-contradiction (function-local C enum)
    const VT_UI1_PRIO: u32 = 1;  // nolint: const-contradiction (function-local C enum)
    const VT_I2_PRIO: u32 = 2;  // nolint: const-contradiction (function-local C enum)
    const VT_I4_PRIO: u32 = 3;  // nolint: const-contradiction (function-local C enum)
    const VT_I8_PRIO: u32 = 4;  // nolint: const-contradiction (function-local C enum)
    const VT_BSTR_PRIO: u32 = 5;  // nolint: const-contradiction (function-local C enum)
    const VT_R4_PRIO: u32 = 6;  // nolint: const-contradiction (function-local C enum)
    const VT_R8_PRIO: u32 = 7;  // nolint: const-contradiction (function-local C enum)
    const VT_CY_PRIO: u32 = 8;  // nolint: const-contradiction (function-local C enum)
    const VT_DATE_PRIO: u32 = 9;  // nolint: const-contradiction (function-local C enum)
    const VT_DECIMAL_PRIO: u32 = 10;  // nolint: const-contradiction (function-local C enum)
    const VT_DISPATCH_PRIO: u32 = 11;  // nolint: const-contradiction (function-local C enum)
    const VT_NULL_PRIO: u32 = 12;  // nolint: const-contradiction (function-local C enum)
    const VT_ERROR_PRIO: u32 = 13;  // nolint: const-contradiction (function-local C enum)

    static PRIO2VT: [VARTYPE; 14] = [
        VT_EMPTY, VT_UI1, VT_I2, VT_I4, VT_I8, VT_BSTR, VT_R4,
        VT_R8, VT_CY, VT_DATE, VT_DECIMAL, VT_DISPATCH,
        VT_NULL, VT_ERROR
    ];

    static COERCE: [u32; 21] = [
        VT_EMPTY_PRIO, VT_NULL_PRIO, VT_I2_PRIO, VT_I4_PRIO, VT_R4_PRIO,
        VT_R8_PRIO, VT_CY_PRIO, VT_DATE_PRIO, VT_BSTR_PRIO, VT_DISPATCH_PRIO,
        VT_ERROR_PRIO, VT_I2_PRIO, VT_ERROR_PRIO, VT_ERROR_PRIO, VT_DECIMAL_PRIO,
        VT_ERROR_PRIO, VT_ERROR_PRIO, VT_UI1_PRIO, VT_ERROR_PRIO, VT_ERROR_PRIO, VT_I8_PRIO
    ];

    TRACE!(b"(%s,%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(left), debugstr_variant(right), result);

    VariantInit(&mut lv);
    VariantInit(&mut rv);
    VariantInit(&mut tv);
    VariantInit(&mut tempLeft);
    VariantInit(&mut tempRight);

    if ((*V_VT_ref(left)) & VT_TYPEMASK) != VT_NULL && ((*V_VT_ref(right)) & VT_TYPEMASK) != VT_NULL {
        if ((*V_VT_ref(left)) & VT_TYPEMASK) == VT_DISPATCH {
            hres = VARIANT_FetchDispatchValue(left, &mut tempLeft);
            if FAILED(hres) {
                'end: {
                    if hres != S_OK {
                        (*V_VT_ref(result)) = VT_EMPTY;
                        (*V_I4_ref(result)) = 0;
                    }
                    VariantClear(&mut lv);
                    VariantClear(&mut rv);
                    VariantClear(&mut tv);
                    VariantClear(&mut tempLeft);
                    VariantClear(&mut tempRight);
                    TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
                    return hres;
                }
            }
            left = &mut tempLeft;
        }
        if ((*V_VT_ref(right)) & VT_TYPEMASK) == VT_DISPATCH {
            hres = VARIANT_FetchDispatchValue(right, &mut tempRight);
            if FAILED(hres) {
                'end: {
                    if hres != S_OK {
                        (*V_VT_ref(result)) = VT_EMPTY;
                        (*V_I4_ref(result)) = 0;
                    }
                    VariantClear(&mut lv);
                    VariantClear(&mut rv);
                    VariantClear(&mut tv);
                    VariantClear(&mut tempLeft);
                    VariantClear(&mut tempRight);
                    TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
                    return hres;
                }
            }
            right = &mut tempRight;
        }
    }

    lvt = (*V_VT_ref(left)) & VT_TYPEMASK;
    rvt = (*V_VT_ref(right)) & VT_TYPEMASK;

    if ((*V_VT_ref(left)) & !VT_TYPEMASK) != 0 || ((*V_VT_ref(right)) & !VT_TYPEMASK) != 0 ||
        lvt > VT_I8 || rvt > VT_I8 {
        hres = DISP_E_BADVARTYPE;
        'end: {
            if hres != S_OK {
                (*V_VT_ref(result)) = VT_EMPTY;
                (*V_I4_ref(result)) = 0;
            }
            VariantClear(&mut lv);
            VariantClear(&mut rv);
            VariantClear(&mut tv);
            VariantClear(&mut tempLeft);
            VariantClear(&mut tempRight);
            TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
            return hres;
        }
    }

    if COERCE[lvt as usize] > COERCE[rvt as usize] {
        resvt = PRIO2VT[COERCE[lvt as usize] as usize];
        tvt = PRIO2VT[COERCE[rvt as usize] as usize];
    } else {
        resvt = PRIO2VT[COERCE[rvt as usize] as usize];
        tvt = PRIO2VT[COERCE[lvt as usize] as usize];
    }

    if resvt == VT_BSTR {
        if tvt == VT_EMPTY || tvt == VT_BSTR {
            resvt = VT_BSTR;
        } else {
            resvt = VT_R8;
        }
    }
    if resvt == VT_R4 && (tvt == VT_BSTR || tvt == VT_I8 || tvt == VT_I4) {
        resvt = VT_R8;
    }

    match resvt {
        VT_ERROR => {
            hres = DISP_E_BADVARTYPE;
            'end: {
                if hres != S_OK {
                    (*V_VT_ref(result)) = VT_EMPTY;
                    (*V_I4_ref(result)) = 0;
                }
                VariantClear(&mut lv);
                VariantClear(&mut rv);
                VariantClear(&mut tv);
                VariantClear(&mut tempLeft);
                VariantClear(&mut tempRight);
                TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
                return hres;
            }
        }
        VT_NULL => {
            hres = S_OK;
            (*V_VT_ref(result)) = VT_NULL;
            'end: {
                if hres != S_OK {
                    (*V_VT_ref(result)) = VT_EMPTY;
                    (*V_I4_ref(result)) = 0;
                }
                VariantClear(&mut lv);
                VariantClear(&mut rv);
                VariantClear(&mut tv);
                VariantClear(&mut tempLeft);
                VariantClear(&mut tempRight);
                TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
                return hres;
            }
        }
        VT_DISPATCH => {
            FIXME!(b"cannot handle variant type VT_DISPATCH\n\0".as_ptr() as *const i8);
            hres = DISP_E_TYPEMISMATCH;
            'end: {
                if hres != S_OK {
                    (*V_VT_ref(result)) = VT_EMPTY;
                    (*V_I4_ref(result)) = 0;
                }
                VariantClear(&mut lv);
                VariantClear(&mut rv);
                VariantClear(&mut tv);
                VariantClear(&mut tempLeft);
                VariantClear(&mut tempRight);
                TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
                return hres;
            }
        }
        VT_EMPTY => {
            resvt = VT_I2;
            tvt = VT_I8;
        }
        VT_UI1 | VT_I2 | VT_I4 | VT_I8 => {
            tvt = VT_I8;
        }
        VT_DATE | VT_R4 => {
            tvt = VT_R8;
        }
        _ => {
            tvt = resvt;
        }
    }

    hres = VariantChangeType(&mut lv, left, 0, tvt);
    if FAILED(hres) {
        'end: {
            if hres != S_OK {
                (*V_VT_ref(result)) = VT_EMPTY;
                (*V_I4_ref(result)) = 0;
            }
            VariantClear(&mut lv);
            VariantClear(&mut rv);
            VariantClear(&mut tv);
            VariantClear(&mut tempLeft);
            VariantClear(&mut tempRight);
            TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
            return hres;
        }
    }
    hres = VariantChangeType(&mut rv, right, 0, tvt);
    if FAILED(hres) {
        'end: {
            if hres != S_OK {
                (*V_VT_ref(result)) = VT_EMPTY;
                (*V_I4_ref(result)) = 0;
            }
            VariantClear(&mut lv);
            VariantClear(&mut rv);
            VariantClear(&mut tv);
            VariantClear(&mut tempLeft);
            VariantClear(&mut tempRight);
            TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
            return hres;
        }
    }

    hres = S_OK;
    (*V_VT_ref(result)) = resvt;
    match tvt {
        VT_DECIMAL => {
            hres = VarDecAdd(V_DECIMAL_ref(&mut lv), V_DECIMAL_ref(&mut rv), V_DECIMAL_ref(result));
            'end: {
                if hres != S_OK {
                    (*V_VT_ref(result)) = VT_EMPTY;
                    (*V_I4_ref(result)) = 0;
                }
                VariantClear(&mut lv);
                VariantClear(&mut rv);
                VariantClear(&mut tv);
                VariantClear(&mut tempLeft);
                VariantClear(&mut tempRight);
                TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
                return hres;
            }
        }
        VT_CY => {
            hres = VarCyAdd(*V_CY_ref(&mut lv) , *V_CY_ref(&mut rv) , V_CY_ref(result));
            'end: {
                if hres != S_OK {
                    (*V_VT_ref(result)) = VT_EMPTY;
                    (*V_I4_ref(result)) = 0;
                }
                VariantClear(&mut lv);
                VariantClear(&mut rv);
                VariantClear(&mut tv);
                VariantClear(&mut tempLeft);
                VariantClear(&mut tempRight);
                TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
                return hres;
            }
        }
        VT_BSTR => {
            hres = VarBstrCat(*V_BSTR_ref(&mut lv) , *V_BSTR_ref(&mut rv) , V_BSTR_ref(result));
            'end: {
                if hres != S_OK {
                    (*V_VT_ref(result)) = VT_EMPTY;
                    (*V_I4_ref(result)) = 0;
                }
                VariantClear(&mut lv);
                VariantClear(&mut rv);
                VariantClear(&mut tv);
                VariantClear(&mut tempLeft);
                VariantClear(&mut tempRight);
                TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
                return hres;
            }
        }
        VT_I8 => {
            r8res = (*V_I8_ref(&mut lv)) as f64 + (*V_I8_ref(&mut rv)) as f64;
            if r8res > 9223372036854775807i64 as f64 || r8res < (-9223372036854775807i64 - 1) as f64 {
                (*V_VT_ref(result)) = VT_R8;
                (*V_R8_ref(result)) = r8res;
                'end: {
                    if hres != S_OK {
                        (*V_VT_ref(result)) = VT_EMPTY;
                        (*V_I4_ref(result)) = 0;
                    }
                    VariantClear(&mut lv);
                    VariantClear(&mut rv);
                    VariantClear(&mut tv);
                    VariantClear(&mut tempLeft);
                    VariantClear(&mut tempRight);
                    TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
                    return hres;
                }
            } else {
                (*V_VT_ref(&mut tv)) = tvt;
                (*V_I8_ref(&mut tv)) = (*V_I8_ref(&mut lv)) + (*V_I8_ref(&mut rv));
            }
        }
        VT_R8 => {
            (*V_VT_ref(&mut tv)) = tvt;
            (*V_R8_ref(&mut tv)) = (*V_R8_ref(&mut lv)) + (*V_R8_ref(&mut rv));
        }
        _ => {
            ERR!(b"We shouldn't get here! tvt = %d!\n\0".as_ptr() as *const i8, tvt);
        }
    }
    if resvt != tvt {
        hres = VariantChangeType(result, &mut tv, 0, resvt);
        if hres != S_OK {
            if resvt == VT_I4 {
                resvt = VT_R8;
            } else {
                resvt = PRIO2VT[COERCE[resvt as usize] as usize + 1];
            }
            hres = VariantChangeType(result, &mut tv, 0, resvt);
        }
    } else {
        hres = VariantCopy(result, &mut tv);
    }

    if hres != S_OK {
        (*V_VT_ref(result)) = VT_EMPTY;
        (*V_I4_ref(result)) = 0;
    }
    VariantClear(&mut lv);
    VariantClear(&mut rv);
    VariantClear(&mut tv);
    VariantClear(&mut tempLeft);
    VariantClear(&mut tempRight);
    TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
    return hres;
}

#[no_mangle]
pub unsafe extern "system" fn VarMul(left: LPVARIANT, right: LPVARIANT, result: LPVARIANT) -> HRESULT {

    const VT_TYPEMASK: u16 = 0x0FFF;
    const VT_EMPTY: u16 = 0;
    const VT_NULL: u16 = 1;
    const VT_I2: u16 = 2;
    const VT_I4: u16 = 3;
    const VT_R4: u16 = 4;
    const VT_R8: u16 = 5;
    const VT_CY: u16 = 6;
    const VT_BSTR: u16 = 8;
    const VT_DISPATCH: u16 = 9;
    const VT_ERROR: u16 = 10;
    const VT_BOOL: u16 = 11;
    const VT_DECIMAL: u16 = 14;
    const VT_UI1: u16 = 17;
    const VT_I8: u16 = 20;
    const DISP_E_BADVARTYPE: HRESULT = 0x80020008u32 as i32;
    const S_OK: HRESULT = 0;
    const I8_MAX: i64 = 0x7FFFFFFFFFFFFFFF;
    const I8_MIN: i64 = -0x7FFFFFFFFFFFFFFF - 1;

    // variant.c:3353: a SECOND, shorter `enum coerceprio` (no EMPTY/BSTR/DATE/DISPATCH).
    // Same member names as the ladder above, deliberately different values. See there.
    const VT_UI1_PRIO: u16 = 0;  // nolint: const-contradiction (function-local C enum)
    const VT_I2_PRIO: u16 = 1;  // nolint: const-contradiction (function-local C enum)
    const VT_I4_PRIO: u16 = 2;  // nolint: const-contradiction (function-local C enum)
    const VT_I8_PRIO: u16 = 3;  // nolint: const-contradiction (function-local C enum)
    const VT_CY_PRIO: u16 = 4;  // nolint: const-contradiction (function-local C enum)
    const VT_R4_PRIO: u16 = 5;  // nolint: const-contradiction (function-local C enum)
    const VT_R8_PRIO: u16 = 6;  // nolint: const-contradiction (function-local C enum)
    const VT_DECIMAL_PRIO: u16 = 7;  // nolint: const-contradiction (function-local C enum)
    const VT_NULL_PRIO: u16 = 8;  // nolint: const-contradiction (function-local C enum)
    const VT_ERROR_PRIO: u16 = 9;  // nolint: const-contradiction (function-local C enum)

    let prio2vt: [VARTYPE; 10] = [VT_UI1, VT_I2, VT_I4, VT_I8, VT_CY, VT_R4, VT_R8, VT_DECIMAL, VT_NULL, VT_ERROR];

    let coerce: [u16; 21] = [
        VT_UI1_PRIO, VT_NULL_PRIO, VT_I2_PRIO, VT_I4_PRIO, VT_R4_PRIO,
        VT_R8_PRIO, VT_CY_PRIO, VT_R8_PRIO, VT_R8_PRIO, VT_ERROR_PRIO,
        VT_ERROR_PRIO, VT_I2_PRIO, VT_ERROR_PRIO, VT_ERROR_PRIO, VT_DECIMAL_PRIO,
        VT_ERROR_PRIO, VT_ERROR_PRIO, VT_UI1_PRIO, VT_ERROR_PRIO, VT_ERROR_PRIO, VT_I8_PRIO
    ];

    TRACE!(b"(%s,%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(left), debugstr_variant(right), result);

    let mut lv: VARIANTARG = core::mem::zeroed();
    let mut rv: VARIANTARG = core::mem::zeroed();
    let mut tv: VARIANTARG = core::mem::zeroed();
    let mut tempLeft: VARIANTARG = core::mem::zeroed();
    let mut tempRight: VARIANTARG = core::mem::zeroed();

    VariantInit(&mut lv);
    VariantInit(&mut rv);
    VariantInit(&mut tv);
    VariantInit(&mut tempLeft);
    VariantInit(&mut tempRight);

    let mut left = left;
    let mut right = right;
    let mut hres: HRESULT;

    'end: loop {
        if ((*V_VT_ref(left)) & VT_TYPEMASK) == VT_DISPATCH {
            hres = VARIANT_FetchDispatchValue(left, &mut tempLeft);
            if FAILED(hres) { break 'end; }
            left = &mut tempLeft;
        }
        if ((*V_VT_ref(right)) & VT_TYPEMASK) == VT_DISPATCH {
            hres = VARIANT_FetchDispatchValue(right, &mut tempRight);
            if FAILED(hres) { break 'end; }
            right = &mut tempRight;
        }

        let lvt = (*V_VT_ref(left)) & VT_TYPEMASK;
        let rvt = (*V_VT_ref(right)) & VT_TYPEMASK;

        if ((*V_VT_ref(left)) & !VT_TYPEMASK) != 0 || ((*V_VT_ref(right)) & !VT_TYPEMASK) != 0 ||
           lvt > VT_I8 || rvt > VT_I8 {
            hres = DISP_E_BADVARTYPE;
            break 'end;
        }

        let mut resvt: VARTYPE;
        let mut tvt: VARTYPE;

        if coerce[lvt as usize] > coerce[rvt as usize] {
            resvt = prio2vt[coerce[lvt as usize] as usize];
            tvt = prio2vt[coerce[rvt as usize] as usize];
        } else {
            resvt = prio2vt[coerce[rvt as usize] as usize];
            tvt = prio2vt[coerce[lvt as usize] as usize];
        }

        if resvt == VT_R4 && (tvt == VT_CY || tvt == VT_I8 || tvt == VT_I4) {
            resvt = VT_R8;
        }
        if lvt == VT_EMPTY && rvt == VT_EMPTY {
            resvt = VT_I2;
        }

        match resvt {
            VT_ERROR => {
                hres = DISP_E_BADVARTYPE;
                break 'end;
            }
            VT_NULL => {
                hres = S_OK;
                (*V_VT_ref(result)) = VT_NULL;
                break 'end;
            }
            VT_UI1 | VT_I2 | VT_I4 | VT_I8 => {
                tvt = VT_I8;
            }
            VT_R4 => {
                tvt = VT_R8;
            }
            _ => {
                tvt = resvt;
            }
        }

        hres = VariantChangeType(&mut lv, left as *mut VARIANTARG, 0, tvt);
        if FAILED(hres) {
            break 'end;
        }
        hres = VariantChangeType(&mut rv, right as *mut VARIANTARG, 0, tvt);
        if FAILED(hres) {
            break 'end;
        }

        hres = S_OK;
        (*V_VT_ref(&mut tv)) = tvt;
        (*V_VT_ref(result)) = resvt;

        match tvt {
            VT_DECIMAL => {
                hres = VarDecMul(V_DECIMAL_ref(&mut lv), V_DECIMAL_ref(&mut rv), V_DECIMAL_ref(result));
                break 'end;
            }
            VT_CY => {
                hres = VarCyMul(*V_CY_ref(&mut lv) , *V_CY_ref(&mut rv) , V_CY_ref(result));
                break 'end;
            }
            VT_I8 => {
                let r8res = (*V_I8_ref(&mut lv)) as f64 * (*V_I8_ref(&mut rv)) as f64;
                if r8res > I8_MAX as f64 || r8res < I8_MIN as f64 {
                    (*V_VT_ref(result)) = VT_R8;
                    (*V_R8_ref(result)) = r8res;
                    break 'end;
                } else {
                    (*V_I8_ref(&mut tv)) = (*V_I8_ref(&mut lv)) * (*V_I8_ref(&mut rv));
                }
            }
            VT_R8 => {
                (*V_R8_ref(&mut tv)) = (*V_R8_ref(&mut lv)) * (*V_R8_ref(&mut rv));
            }
            _ => {
                ERR!(b"We shouldn't get here! tvt = %d!\n\0".as_ptr() as *const i8, tvt);
            }
        }

        if resvt != tvt {
            loop {
                hres = VariantChangeType(result, &mut tv, 0, resvt);
                if hres == S_OK {
                    break;
                }
                if resvt == VT_I4 {
                    resvt = VT_R8;
                } else {
                    resvt = prio2vt[coerce[resvt as usize] as usize + 1];
                }
            }
        } else {
            hres = VariantCopy(result, &mut tv);
        }

        break 'end;
    }

    if hres != S_OK {
        (*V_VT_ref(result)) = VT_EMPTY;
        (*V_I4_ref(result)) = 0;
    }

    VariantClear(&mut lv);
    VariantClear(&mut rv);
    VariantClear(&mut tv);
    VariantClear(&mut tempLeft);
    VariantClear(&mut tempRight);

    TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as *const i8, hres, debugstr_variant(result));
    hres
}

#[no_mangle]
pub unsafe extern "system" fn VarDiv(left: LPVARIANT, right: LPVARIANT, result: LPVARIANT) -> HRESULT {

    let mut hres: HRESULT;
    let mut resvt: VARTYPE = VT_EMPTY;
    let leftvt: VARTYPE;
    let rightvt: VARTYPE;
    let rightExtraFlags: VARTYPE;
    let leftExtraFlags: VARTYPE;
    let ExtraFlags: VARTYPE;
    let mut lv: VARIANT = core::mem::zeroed();
    let mut rv: VARIANT = core::mem::zeroed();
    let mut tempLeft: VARIANT = core::mem::zeroed();
    let mut tempRight: VARIANT = core::mem::zeroed();
    let mut left = left;
    let mut right = right;

    VariantInit(&mut tempLeft);
    VariantInit(&mut tempRight);
    VariantInit(&mut lv);
    VariantInit(&mut rv);

    TRACE!(b"(%s,%s,%p)\n\0".as_ptr(), debugstr_variant(left), debugstr_variant(right), result);

    'end: {
        if ((*V_VT_ref(left)) & VT_TYPEMASK) == VT_DISPATCH {
            hres = VARIANT_FetchDispatchValue(left, &mut tempLeft);
            if FAILED(hres) {
                break 'end;
            }
            left = &mut tempLeft;
        }
        if ((*V_VT_ref(right)) & VT_TYPEMASK) == VT_DISPATCH {
            hres = VARIANT_FetchDispatchValue(right, &mut tempRight);
            if FAILED(hres) {
                break 'end;
            }
            right = &mut tempRight;
        }

        leftvt = (*V_VT_ref(left)) & VT_TYPEMASK;
        rightvt = (*V_VT_ref(right)) & VT_TYPEMASK;
        leftExtraFlags = (*V_VT_ref(left)) & (!VT_TYPEMASK);
        rightExtraFlags = (*V_VT_ref(right)) & (!VT_TYPEMASK);

        if leftExtraFlags != rightExtraFlags {
            hres = DISP_E_BADVARTYPE;
            break 'end;
        }
        ExtraFlags = leftExtraFlags;

        if ExtraFlags != 0 {
            hres = DISP_E_BADVARTYPE;
            break 'end;
        }

        if rightvt != VT_EMPTY {
            if leftvt == VT_NULL || rightvt == VT_NULL {
                (*result).n1.n2.vt = VT_NULL;
                hres = S_OK;
                break 'end;
            } else if leftvt == VT_DECIMAL || rightvt == VT_DECIMAL {
                resvt = VT_DECIMAL;
            } else if leftvt == VT_I8 || rightvt == VT_I8 ||
                leftvt == VT_CY || rightvt == VT_CY ||
                leftvt == VT_DATE || rightvt == VT_DATE ||
                leftvt == VT_I4 || rightvt == VT_I4 ||
                leftvt == VT_BSTR || rightvt == VT_BSTR ||
                leftvt == VT_I2 || rightvt == VT_I2 ||
                leftvt == VT_BOOL || rightvt == VT_BOOL ||
                leftvt == VT_R8 || rightvt == VT_R8 ||
                leftvt == VT_UI1 || rightvt == VT_UI1
            {
                if (leftvt == VT_UI1 && rightvt == VT_R4) ||
                    (leftvt == VT_R4 && rightvt == VT_UI1) {
                    resvt = VT_R4;
                } else if (leftvt == VT_R4 && (rightvt == VT_BOOL || rightvt == VT_I2)) ||
                    (rightvt == VT_R4 && (leftvt == VT_BOOL || leftvt == VT_I2)) {
                    resvt = VT_R4;
                } else {
                    resvt = VT_R8;
                }
            } else if leftvt == VT_R4 || rightvt == VT_R4 {
                resvt = VT_R4;
            }
        } else if leftvt == VT_NULL {
            (*result).n1.n2.vt = VT_NULL;
            hres = S_OK;
            break 'end;
        } else {
            hres = DISP_E_BADVARTYPE;
            break 'end;
        }

        hres = VariantChangeType(&mut lv, left, 0, resvt);
        if hres != S_OK {
            break 'end;
        }

        hres = VariantChangeType(&mut rv, right, 0, resvt);
        if hres != S_OK {
            break 'end;
        }

        (*result).n1.n2.vt = resvt;
        match resvt {
            VT_R4 => {
                if (*V_R4_ref(&mut lv)) == 0.0 && (*V_R4_ref(&mut rv)) == 0.0 {
                    hres = DISP_E_OVERFLOW;
                    (*result).n1.n2.vt = VT_EMPTY;
                } else if (*V_R4_ref(&mut rv)) == 0.0 {
                    hres = DISP_E_DIVBYZERO;
                    (*result).n1.n2.vt = VT_EMPTY;
                } else {
                    (*V_R4_ref(result)) = (*V_R4_ref(&mut lv)) / (*V_R4_ref(&mut rv));
                }
            }
            VT_R8 => {
                if (*V_R8_ref(&mut lv)) == 0.0 && (*V_R8_ref(&mut rv)) == 0.0 {
                    hres = DISP_E_OVERFLOW;
                    (*result).n1.n2.vt = VT_EMPTY;
                } else if (*V_R8_ref(&mut rv)) == 0.0 {
                    hres = DISP_E_DIVBYZERO;
                    (*result).n1.n2.vt = VT_EMPTY;
                } else {
                    (*V_R8_ref(result)) = (*V_R8_ref(&mut lv)) / (*V_R8_ref(&mut rv));
                }
            }
            VT_DECIMAL => {
                hres = VarDecDiv(V_DECIMAL_ref(&mut lv), V_DECIMAL_ref(&mut rv), V_DECIMAL_ref(result));
            }
            _ => {}
        }
    }

    VariantClear(&mut lv);
    VariantClear(&mut rv);
    VariantClear(&mut tempLeft);
    VariantClear(&mut tempRight);
    TRACE!(b"returning 0x%8x %s\n\0".as_ptr(), hres, debugstr_variant(result));
    hres
}

#[no_mangle]
pub unsafe extern "system" fn VarSub(mut left: LPVARIANT, mut right: LPVARIANT, result: LPVARIANT) -> HRESULT {
    let mut hres: HRESULT = S_OK;
    let resvt: VARTYPE;
    let leftvt: VARTYPE;
    let rightvt: VARTYPE;
    let rightExtraFlags: VARTYPE;
    let leftExtraFlags: VARTYPE;
    let ExtraFlags: VARTYPE;
    let mut lv: VARIANT = core::mem::zeroed();
    let mut rv: VARIANT = core::mem::zeroed();
    let mut tempLeft: VARIANT = core::mem::zeroed();
    let mut tempRight: VARIANT = core::mem::zeroed();

    VariantInit(&mut lv);
    VariantInit(&mut rv);
    VariantInit(&mut tempLeft);
    VariantInit(&mut tempRight);

    TRACE!(b"(%s,%s,%p)\n\0".as_ptr() as _, debugstr_variant(left), debugstr_variant(right), result);

    'end: loop {
        if ((*V_VT_ref(left)) & VT_TYPEMASK) == VT_DISPATCH &&
            ((*V_VT_ref(left)) & (!VT_TYPEMASK)) == 0 &&
            ((*V_VT_ref(right)) & VT_TYPEMASK) != VT_NULL
        {
            if (*V_DISPATCH_ref(left)).is_null() {
                if ((*V_VT_ref(right)) & VT_TYPEMASK) >= VT_INT_PTR {
                    hres = DISP_E_BADVARTYPE;
                } else if ((*V_VT_ref(right)) & VT_TYPEMASK) >= VT_UI8 &&
                    ((*V_VT_ref(right)) & VT_TYPEMASK) < VT_RECORD
                {
                    hres = DISP_E_BADVARTYPE;
                } else {
                    match (*V_VT_ref(right)) & VT_TYPEMASK {
                        VT_VARIANT | VT_UNKNOWN | 15 | VT_I1 | VT_UI2 | VT_UI4 => {
                            hres = DISP_E_BADVARTYPE;
                        }
                        _ => {}
                    }
                }
                if FAILED(hres) { break 'end; }
            }
            hres = VARIANT_FetchDispatchValue(left, &mut tempLeft);
            if FAILED(hres) { break 'end; }
            left = &mut tempLeft; // C re-points the operand at the fetched value
        }
        if ((*V_VT_ref(right)) & VT_TYPEMASK) == VT_DISPATCH &&
            ((*V_VT_ref(right)) & (!VT_TYPEMASK)) == 0 &&
            ((*V_VT_ref(left)) & VT_TYPEMASK) != VT_NULL
        {
            if (*V_DISPATCH_ref(right)).is_null() {
                if ((*V_VT_ref(left)) & VT_TYPEMASK) >= VT_INT_PTR {
                    hres = DISP_E_BADVARTYPE;
                } else if ((*V_VT_ref(left)) & VT_TYPEMASK) >= VT_UI8 &&
                    ((*V_VT_ref(left)) & VT_TYPEMASK) < VT_RECORD
                {
                    hres = DISP_E_BADVARTYPE;
                } else {
                    match (*V_VT_ref(left)) & VT_TYPEMASK {
                        VT_VARIANT | VT_UNKNOWN | 15 | VT_I1 | VT_UI2 | VT_UI4 => {
                            hres = DISP_E_BADVARTYPE;
                        }
                        _ => {}
                    }
                }
                if FAILED(hres) { break 'end; }
            }
            hres = VARIANT_FetchDispatchValue(right, &mut tempRight);
            if FAILED(hres) { break 'end; }
            right = &mut tempRight;
        }

        leftvt = (*V_VT_ref(left)) & VT_TYPEMASK;
        rightvt = (*V_VT_ref(right)) & VT_TYPEMASK;
        leftExtraFlags = (*V_VT_ref(left)) & (!VT_TYPEMASK);
        rightExtraFlags = (*V_VT_ref(right)) & (!VT_TYPEMASK);

        if leftExtraFlags != rightExtraFlags {
            hres = DISP_E_BADVARTYPE;
            break 'end;
        }
        ExtraFlags = leftExtraFlags;

        if ExtraFlags == (VT_VECTOR | VT_BYREF | VT_RESERVED) ||
            ExtraFlags == (VT_VECTOR | VT_RESERVED) ||
            ExtraFlags == (VT_VECTOR | VT_BYREF) ||
            ExtraFlags == (VT_BYREF | VT_RESERVED) ||
            ExtraFlags == VT_VECTOR ||
            ExtraFlags == VT_BYREF ||
            ExtraFlags == VT_RESERVED
        {
            hres = DISP_E_BADVARTYPE;
            break 'end;
        } else if ExtraFlags >= VT_ARRAY {
            hres = DISP_E_TYPEMISMATCH;
            break 'end;
        } else if leftvt == VT_CLSID || rightvt == VT_CLSID ||
            leftvt == VT_VARIANT || rightvt == VT_VARIANT ||
            leftvt == VT_I1 || rightvt == VT_I1 ||
            leftvt == VT_UI2 || rightvt == VT_UI2 ||
            leftvt == VT_UI4 || rightvt == VT_UI4 ||
            leftvt == VT_UI8 || rightvt == VT_UI8 ||
            leftvt == VT_INT || rightvt == VT_INT ||
            leftvt == VT_UINT || rightvt == VT_UINT ||
            leftvt == VT_UNKNOWN || rightvt == VT_UNKNOWN ||
            leftvt == VT_RECORD || rightvt == VT_RECORD
        {
            if leftvt == VT_RECORD && rightvt == VT_I8 {
                hres = DISP_E_TYPEMISMATCH;
            } else if leftvt < VT_UI1 && rightvt == VT_RECORD {
                hres = DISP_E_TYPEMISMATCH;
            } else if leftvt >= VT_UI1 && rightvt == VT_RECORD {
                hres = DISP_E_TYPEMISMATCH;
            } else if leftvt == VT_RECORD && rightvt <= VT_UI1 {
                hres = DISP_E_TYPEMISMATCH;
            } else if leftvt == VT_RECORD && rightvt > VT_UI1 {
                hres = DISP_E_BADVARTYPE;
            } else {
                hres = DISP_E_BADVARTYPE;
            }
            break 'end;
        } else if !((leftvt <= VT_LPWSTR || leftvt == VT_RECORD ||
            leftvt == VT_CLSID) && leftvt != 15 &&
            (leftvt < VT_VOID || leftvt > VT_LPWSTR))
        {
            hres = DISP_E_BADVARTYPE;
            break 'end;
        } else if !((rightvt <= VT_LPWSTR || rightvt == VT_RECORD ||
            rightvt == VT_CLSID) && rightvt != 15 &&
            (rightvt < VT_VOID || rightvt > VT_LPWSTR))
        {
            hres = DISP_E_BADVARTYPE;
            break 'end;
        } else if (leftvt == VT_NULL && rightvt == VT_DISPATCH) ||
            (leftvt == VT_DISPATCH && rightvt == VT_NULL)
        {
            resvt = VT_NULL;
        } else if leftvt == VT_DISPATCH || rightvt == VT_DISPATCH ||
            leftvt == VT_ERROR || rightvt == VT_ERROR
        {
            hres = DISP_E_TYPEMISMATCH;
            break 'end;
        } else if leftvt == VT_NULL || rightvt == VT_NULL {
            resvt = VT_NULL;
        } else if (leftvt == VT_EMPTY && rightvt == VT_BSTR) ||
            (leftvt == VT_DATE && rightvt == VT_DATE) ||
            (leftvt == VT_BSTR && rightvt == VT_EMPTY) ||
            (leftvt == VT_BSTR && rightvt == VT_BSTR)
        {
            resvt = VT_R8;
        } else if leftvt == VT_DECIMAL || rightvt == VT_DECIMAL {
            resvt = VT_DECIMAL;
        } else if leftvt == VT_DATE || rightvt == VT_DATE {
            resvt = VT_DATE;
        } else if leftvt == VT_CY || rightvt == VT_CY {
            resvt = VT_CY;
        } else if leftvt == VT_R8 || rightvt == VT_R8 {
            resvt = VT_R8;
        } else if leftvt == VT_BSTR || rightvt == VT_BSTR {
            resvt = VT_R8;
        } else if leftvt == VT_R4 || rightvt == VT_R4 {
            if leftvt == VT_I4 || rightvt == VT_I4 ||
                leftvt == VT_I8 || rightvt == VT_I8
            {
                resvt = VT_R8;
            } else {
                resvt = VT_R4;
            }
        } else if leftvt == VT_I8 || rightvt == VT_I8 {
            resvt = VT_I8;
        } else if leftvt == VT_I4 || rightvt == VT_I4 {
            resvt = VT_I4;
        } else if leftvt == VT_I2 || rightvt == VT_I2 ||
            leftvt == VT_BOOL || rightvt == VT_BOOL ||
            (leftvt == VT_EMPTY && rightvt == VT_EMPTY)
        {
            resvt = VT_I2;
        } else if leftvt == VT_UI1 || rightvt == VT_UI1 {
            resvt = VT_UI1;
        } else {
            hres = DISP_E_TYPEMISMATCH;
            break 'end;
        }

        if leftvt == VT_BSTR && rightvt == VT_DATE {
            hres = VariantChangeType(&mut lv, left, 0, VT_R8);
        } else {
            hres = VariantChangeType(&mut lv, left, 0, resvt);
        }
        if hres != S_OK { break 'end; }
        if leftvt == VT_DATE && rightvt == VT_BSTR {
            hres = VariantChangeType(&mut rv, right, 0, VT_R8);
        } else {
            hres = VariantChangeType(&mut rv, right, 0, resvt);
        }
        if hres != S_OK { break 'end; }

        (*V_VT_ref(result)) = resvt;
        match resvt {
            VT_NULL => {}
            VT_DATE => {
                (*V_DATE_ref(result)) = (*V_DATE_ref(&mut lv)) - (*V_DATE_ref(&mut rv));
            }
            VT_CY => {
                hres = VarCySub(*V_CY_ref(&mut lv) , *V_CY_ref(&mut rv) , V_CY_ref(result));
            }
            VT_R4 => {
                (*V_R4_ref(result)) = (*V_R4_ref(&mut lv)) - (*V_R4_ref(&mut rv));
            }
            VT_I8 => {
                (*V_I8_ref(result)) = (*V_I8_ref(&mut lv)) - (*V_I8_ref(&mut rv));
            }
            VT_I4 => {
                (*V_I4_ref(result)) = (*V_I4_ref(&mut lv)) - (*V_I4_ref(&mut rv));
            }
            VT_I2 => {
                (*V_I2_ref(result)) = (*V_I2_ref(&mut lv)) - (*V_I2_ref(&mut rv));
            }
            VT_UI1 => {
                (*V_UI1_ref(result)) = ((*V_UI2_ref(&mut lv)) as i32 - (*V_UI1_ref(&mut rv)) as i32) as u8;
            }
            VT_R8 => {
                (*V_R8_ref(result)) = (*V_R8_ref(&mut lv)) - (*V_R8_ref(&mut rv));
            }
            VT_DECIMAL => {
                hres = VarDecSub(V_DECIMAL_ref(&mut lv), V_DECIMAL_ref(&mut rv), V_DECIMAL_ref(result));
            }
            _ => {}
        }

        break;
    }

    VariantClear(&mut lv);
    VariantClear(&mut rv);
    VariantClear(&mut tempLeft);
    VariantClear(&mut tempRight);
    TRACE!(b"returning 0x%8x %s\n\0".as_ptr() as _, hres, debugstr_variant(result));
    hres
}

#[no_mangle]
pub unsafe extern "system" fn VarOr(pVarLeft: LPVARIANT, pVarRight: LPVARIANT, pVarOut: LPVARIANT) -> HRESULT {
    let mut vt: VARTYPE = VT_I4;
    let mut varLeft: VARIANT = core::mem::zeroed();
    let mut varRight: VARIANT = core::mem::zeroed();
    let mut varStr: VARIANT = core::mem::zeroed();
    let mut hRet: HRESULT;
    let mut tempLeft: VARIANT = core::mem::zeroed();
    let mut tempRight: VARIANT = core::mem::zeroed();
    let mut pVarLeft = pVarLeft;
    let mut pVarRight = pVarRight;

    VariantInit(&mut tempLeft);
    VariantInit(&mut tempRight);
    VariantInit(&mut varLeft);
    VariantInit(&mut varRight);
    VariantInit(&mut varStr);

    TRACE!(b"(%s,%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(pVarLeft), debugstr_variant(pVarRight), pVarOut);

    'VarOr_Exit: {
        if ((*V_VT_ref(pVarLeft)) & VT_TYPEMASK) == VT_DISPATCH {
            hRet = VARIANT_FetchDispatchValue(pVarLeft, &mut tempLeft);
            if FAILED(hRet) { break 'VarOr_Exit; }
            pVarLeft = &mut tempLeft;
        }
        if ((*V_VT_ref(pVarRight)) & VT_TYPEMASK) == VT_DISPATCH {
            hRet = VARIANT_FetchDispatchValue(pVarRight, &mut tempRight);
            if FAILED(hRet) { break 'VarOr_Exit; }
            pVarRight = &mut tempRight;
        }

        if V_EXTRA_TYPE(pVarLeft) != 0 || V_EXTRA_TYPE(pVarRight) != 0 ||
            (*V_VT_ref(pVarLeft)) == VT_UNKNOWN || (*V_VT_ref(pVarRight)) == VT_UNKNOWN ||
            (*V_VT_ref(pVarLeft)) == VT_DISPATCH || (*V_VT_ref(pVarRight)) == VT_DISPATCH ||
            (*V_VT_ref(pVarLeft)) == VT_RECORD || (*V_VT_ref(pVarRight)) == VT_RECORD
        {
            hRet = DISP_E_BADVARTYPE;
            break 'VarOr_Exit;
        }

        (*V_VT_ref(&mut varLeft)) = VT_EMPTY;
        (*V_VT_ref(&mut varRight)) = VT_EMPTY;
        (*V_VT_ref(&mut varStr)) = VT_EMPTY;

        if (*V_VT_ref(pVarLeft)) == VT_NULL || (*V_VT_ref(pVarRight)) == VT_NULL {
            if (*V_VT_ref(pVarLeft)) == VT_NULL {
                pVarLeft = pVarRight;
            }

            (*V_VT_ref(pVarOut)) = VT_NULL;
            (*V_I4_ref(pVarOut)) = 0;

            match *V_VT_ref(pVarLeft)  {
                VT_DATE | VT_R8 => {
                    if (*V_R8_ref(pVarLeft)) != 0.0 {
                        'VarOr_AsEmpty: {
                            match *V_VT_ref(pVarLeft)  {
                                VT_BSTR => {
                                    if (*V_BSTR_ref(pVarLeft)).is_null() {
                                        hRet = DISP_E_BADVARTYPE;
                                        break 'VarOr_Exit;
                                    }

                                    hRet = VariantCopy(&mut varStr, pVarLeft);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    pVarLeft = &mut varStr;
                                    hRet = VariantChangeType(pVarLeft, pVarLeft, 0, VT_BOOL);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_EMPTY | VT_UI1 | VT_BOOL | VT_I2 => {
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_DATE | VT_CY | VT_DECIMAL | VT_R4 | VT_R8 |
                                VT_I1 | VT_UI2 | VT_I4 | VT_UI4 |
                                VT_INT | VT_UINT | VT_UI8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I4;
                                }
                                VT_I8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I8;
                                }
                                _ => {
                                    hRet = DISP_E_BADVARTYPE;
                                    break 'VarOr_Exit;
                                }
                            }
                            hRet = VariantCopy(&mut varLeft, pVarLeft);
                            if FAILED(hRet) { break 'VarOr_Exit; }
                            pVarLeft = &mut varLeft;
                            hRet = VariantChangeType(pVarOut, pVarLeft, 0, *V_VT_ref(pVarOut) );
                            break 'VarOr_Exit;
                        }
                    }
                    hRet = S_OK;
                    break 'VarOr_Exit;
                }
                VT_BOOL => {
                    if (*V_BOOL_ref(pVarLeft)) != 0 {
                        *pVarOut = *pVarLeft;
                    }
                    hRet = S_OK;
                    break 'VarOr_Exit;
                }
                VT_I2 | VT_UI2 => {
                    if (*V_I2_ref(pVarLeft)) != 0 {
                        'VarOr_AsEmpty: {
                            match *V_VT_ref(pVarLeft)  {
                                VT_BSTR => {
                                    if (*V_BSTR_ref(pVarLeft)).is_null() {
                                        hRet = DISP_E_BADVARTYPE;
                                        break 'VarOr_Exit;
                                    }

                                    hRet = VariantCopy(&mut varStr, pVarLeft);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    pVarLeft = &mut varStr;
                                    hRet = VariantChangeType(pVarLeft, pVarLeft, 0, VT_BOOL);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_EMPTY | VT_UI1 | VT_BOOL | VT_I2 => {
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_DATE | VT_CY | VT_DECIMAL | VT_R4 | VT_R8 |
                                VT_I1 | VT_UI2 | VT_I4 | VT_UI4 |
                                VT_INT | VT_UINT | VT_UI8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I4;
                                }
                                VT_I8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I8;
                                }
                                _ => {
                                    hRet = DISP_E_BADVARTYPE;
                                    break 'VarOr_Exit;
                                }
                            }
                            hRet = VariantCopy(&mut varLeft, pVarLeft);
                            if FAILED(hRet) { break 'VarOr_Exit; }
                            pVarLeft = &mut varLeft;
                            hRet = VariantChangeType(pVarOut, pVarLeft, 0, *V_VT_ref(pVarOut) );
                            break 'VarOr_Exit;
                        }
                    }
                    hRet = S_OK;
                    break 'VarOr_Exit;
                }
                VT_I1 => {
                    if (*V_I1_ref(pVarLeft)) != 0 {
                        'VarOr_AsEmpty: {
                            match *V_VT_ref(pVarLeft)  {
                                VT_BSTR => {
                                    if (*V_BSTR_ref(pVarLeft)).is_null() {
                                        hRet = DISP_E_BADVARTYPE;
                                        break 'VarOr_Exit;
                                    }

                                    hRet = VariantCopy(&mut varStr, pVarLeft);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    pVarLeft = &mut varStr;
                                    hRet = VariantChangeType(pVarLeft, pVarLeft, 0, VT_BOOL);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_EMPTY | VT_UI1 | VT_BOOL | VT_I2 => {
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_DATE | VT_CY | VT_DECIMAL | VT_R4 | VT_R8 |
                                VT_I1 | VT_UI2 | VT_I4 | VT_UI4 |
                                VT_INT | VT_UINT | VT_UI8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I4;
                                }
                                VT_I8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I8;
                                }
                                _ => {
                                    hRet = DISP_E_BADVARTYPE;
                                    break 'VarOr_Exit;
                                }
                            }
                            hRet = VariantCopy(&mut varLeft, pVarLeft);
                            if FAILED(hRet) { break 'VarOr_Exit; }
                            pVarLeft = &mut varLeft;
                            hRet = VariantChangeType(pVarOut, pVarLeft, 0, *V_VT_ref(pVarOut) );
                            break 'VarOr_Exit;
                        }
                    }
                    hRet = S_OK;
                    break 'VarOr_Exit;
                }
                VT_UI1 => {
                    if (*V_UI1_ref(pVarLeft)) != 0 {
                        *pVarOut = *pVarLeft;
                    }
                    hRet = S_OK;
                    break 'VarOr_Exit;
                }
                VT_R4 => {
                    if (*V_R4_ref(pVarLeft)) != 0.0 {
                        'VarOr_AsEmpty: {
                            match *V_VT_ref(pVarLeft)  {
                                VT_BSTR => {
                                    if (*V_BSTR_ref(pVarLeft)).is_null() {
                                        hRet = DISP_E_BADVARTYPE;
                                        break 'VarOr_Exit;
                                    }

                                    hRet = VariantCopy(&mut varStr, pVarLeft);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    pVarLeft = &mut varStr;
                                    hRet = VariantChangeType(pVarLeft, pVarLeft, 0, VT_BOOL);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_EMPTY | VT_UI1 | VT_BOOL | VT_I2 => {
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_DATE | VT_CY | VT_DECIMAL | VT_R4 | VT_R8 |
                                VT_I1 | VT_UI2 | VT_I4 | VT_UI4 |
                                VT_INT | VT_UINT | VT_UI8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I4;
                                }
                                VT_I8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I8;
                                }
                                _ => {
                                    hRet = DISP_E_BADVARTYPE;
                                    break 'VarOr_Exit;
                                }
                            }
                            hRet = VariantCopy(&mut varLeft, pVarLeft);
                            if FAILED(hRet) { break 'VarOr_Exit; }
                            pVarLeft = &mut varLeft;
                            hRet = VariantChangeType(pVarOut, pVarLeft, 0, *V_VT_ref(pVarOut) );
                            break 'VarOr_Exit;
                        }
                    }
                    hRet = S_OK;
                    break 'VarOr_Exit;
                }
                VT_I4 | VT_UI4 | VT_INT | VT_UINT => {
                    if (*V_I4_ref(pVarLeft)) != 0 {
                        'VarOr_AsEmpty: {
                            match *V_VT_ref(pVarLeft)  {
                                VT_BSTR => {
                                    if (*V_BSTR_ref(pVarLeft)).is_null() {
                                        hRet = DISP_E_BADVARTYPE;
                                        break 'VarOr_Exit;
                                    }

                                    hRet = VariantCopy(&mut varStr, pVarLeft);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    pVarLeft = &mut varStr;
                                    hRet = VariantChangeType(pVarLeft, pVarLeft, 0, VT_BOOL);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_EMPTY | VT_UI1 | VT_BOOL | VT_I2 => {
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_DATE | VT_CY | VT_DECIMAL | VT_R4 | VT_R8 |
                                VT_I1 | VT_UI2 | VT_I4 | VT_UI4 |
                                VT_INT | VT_UINT | VT_UI8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I4;
                                }
                                VT_I8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I8;
                                }
                                _ => {
                                    hRet = DISP_E_BADVARTYPE;
                                    break 'VarOr_Exit;
                                }
                            }
                            hRet = VariantCopy(&mut varLeft, pVarLeft);
                            if FAILED(hRet) { break 'VarOr_Exit; }
                            pVarLeft = &mut varLeft;
                            hRet = VariantChangeType(pVarOut, pVarLeft, 0, *V_VT_ref(pVarOut) );
                            break 'VarOr_Exit;
                        }
                    }
                    hRet = S_OK;
                    break 'VarOr_Exit;
                }
                VT_CY => {
                    if (*V_CY_ref(pVarLeft)).int64 != 0 {
                        'VarOr_AsEmpty: {
                            match *V_VT_ref(pVarLeft)  {
                                VT_BSTR => {
                                    if (*V_BSTR_ref(pVarLeft)).is_null() {
                                        hRet = DISP_E_BADVARTYPE;
                                        break 'VarOr_Exit;
                                    }

                                    hRet = VariantCopy(&mut varStr, pVarLeft);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    pVarLeft = &mut varStr;
                                    hRet = VariantChangeType(pVarLeft, pVarLeft, 0, VT_BOOL);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_EMPTY | VT_UI1 | VT_BOOL | VT_I2 => {
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_DATE | VT_CY | VT_DECIMAL | VT_R4 | VT_R8 |
                                VT_I1 | VT_UI2 | VT_I4 | VT_UI4 |
                                VT_INT | VT_UINT | VT_UI8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I4;
                                }
                                VT_I8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I8;
                                }
                                _ => {
                                    hRet = DISP_E_BADVARTYPE;
                                    break 'VarOr_Exit;
                                }
                            }
                            hRet = VariantCopy(&mut varLeft, pVarLeft);
                            if FAILED(hRet) { break 'VarOr_Exit; }
                            pVarLeft = &mut varLeft;
                            hRet = VariantChangeType(pVarOut, pVarLeft, 0, *V_VT_ref(pVarOut) );
                            break 'VarOr_Exit;
                        }
                    }
                    hRet = S_OK;
                    break 'VarOr_Exit;
                }
                VT_I8 | VT_UI8 => {
                    if (*V_I8_ref(pVarLeft)) != 0 {
                        'VarOr_AsEmpty: {
                            match *V_VT_ref(pVarLeft)  {
                                VT_BSTR => {
                                    if (*V_BSTR_ref(pVarLeft)).is_null() {
                                        hRet = DISP_E_BADVARTYPE;
                                        break 'VarOr_Exit;
                                    }

                                    hRet = VariantCopy(&mut varStr, pVarLeft);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    pVarLeft = &mut varStr;
                                    hRet = VariantChangeType(pVarLeft, pVarLeft, 0, VT_BOOL);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_EMPTY | VT_UI1 | VT_BOOL | VT_I2 => {
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_DATE | VT_CY | VT_DECIMAL | VT_R4 | VT_R8 |
                                VT_I1 | VT_UI2 | VT_I4 | VT_UI4 |
                                VT_INT | VT_UINT | VT_UI8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I4;
                                }
                                VT_I8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I8;
                                }
                                _ => {
                                    hRet = DISP_E_BADVARTYPE;
                                    break 'VarOr_Exit;
                                }
                            }
                            hRet = VariantCopy(&mut varLeft, pVarLeft);
                            if FAILED(hRet) { break 'VarOr_Exit; }
                            pVarLeft = &mut varLeft;
                            hRet = VariantChangeType(pVarOut, pVarLeft, 0, *V_VT_ref(pVarOut) );
                            break 'VarOr_Exit;
                        }
                    }
                    hRet = S_OK;
                    break 'VarOr_Exit;
                }
                VT_DECIMAL => {
                    if (*DEC_HI32_ref(V_DECIMAL_ref(pVarLeft))) != 0 || (*DEC_LO64_ref(V_DECIMAL_ref(pVarLeft))) != 0 {
                        'VarOr_AsEmpty: {
                            match *V_VT_ref(pVarLeft)  {
                                VT_BSTR => {
                                    if (*V_BSTR_ref(pVarLeft)).is_null() {
                                        hRet = DISP_E_BADVARTYPE;
                                        break 'VarOr_Exit;
                                    }

                                    hRet = VariantCopy(&mut varStr, pVarLeft);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    pVarLeft = &mut varStr;
                                    hRet = VariantChangeType(pVarLeft, pVarLeft, 0, VT_BOOL);
                                    if FAILED(hRet) { break 'VarOr_Exit; }
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_EMPTY | VT_UI1 | VT_BOOL | VT_I2 => {
                                    (*V_VT_ref(pVarOut)) = VT_I2;
                                }
                                VT_DATE | VT_CY | VT_DECIMAL | VT_R4 | VT_R8 |
                                VT_I1 | VT_UI2 | VT_I4 | VT_UI4 |
                                VT_INT | VT_UINT | VT_UI8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I4;
                                }
                                VT_I8 => {
                                    (*V_VT_ref(pVarOut)) = VT_I8;
                                }
                                _ => {
                                    hRet = DISP_E_BADVARTYPE;
                                    break 'VarOr_Exit;
                                }
                            }
                            hRet = VariantCopy(&mut varLeft, pVarLeft);
                            if FAILED(hRet) { break 'VarOr_Exit; }
                            pVarLeft = &mut varLeft;
                            hRet = VariantChangeType(pVarOut, pVarLeft, 0, *V_VT_ref(pVarOut) );
                            break 'VarOr_Exit;
                        }
                    }
                    hRet = S_OK;
                    break 'VarOr_Exit;
                }
                VT_BSTR => {
                    let mut b: VARIANT_BOOL = 0;

                    if (*V_BSTR_ref(pVarLeft)).is_null() {
                        hRet = DISP_E_BADVARTYPE;
                        break 'VarOr_Exit;
                    }

                    hRet = VarBoolFromStr(*V_BSTR_ref(pVarLeft) , LOCALE_USER_DEFAULT, VAR_LOCALBOOL, &mut b);
                    if SUCCEEDED(hRet) && b != 0 {
                        (*V_VT_ref(pVarOut)) = VT_BOOL;
                        (*V_BOOL_ref(pVarOut)) = b;
                    }
                    break 'VarOr_Exit;
                }
                VT_NULL | VT_EMPTY => {
                    (*V_VT_ref(pVarOut)) = VT_NULL;
                    hRet = S_OK;
                    break 'VarOr_Exit;
                }
                _ => {
                    hRet = DISP_E_BADVARTYPE;
                    break 'VarOr_Exit;
                }
            }
        }

        if (*V_VT_ref(pVarLeft)) == VT_EMPTY || (*V_VT_ref(pVarRight)) == VT_EMPTY {
            if (*V_VT_ref(pVarLeft)) == VT_EMPTY {
                pVarLeft = pVarRight;
            }

            'VarOr_AsEmpty: {
                match *V_VT_ref(pVarLeft)  {
                    VT_BSTR => {
                        if (*V_BSTR_ref(pVarLeft)).is_null() {
                            hRet = DISP_E_BADVARTYPE;
                            break 'VarOr_Exit;
                        }

                        hRet = VariantCopy(&mut varStr, pVarLeft);
                        if FAILED(hRet) { break 'VarOr_Exit; }
                        pVarLeft = &mut varStr;
                        hRet = VariantChangeType(pVarLeft, pVarLeft, 0, VT_BOOL);
                        if FAILED(hRet) { break 'VarOr_Exit; }
                        (*V_VT_ref(pVarOut)) = VT_I2;
                    }
                    VT_EMPTY | VT_UI1 | VT_BOOL | VT_I2 => {
                        (*V_VT_ref(pVarOut)) = VT_I2;
                    }
                    VT_DATE | VT_CY | VT_DECIMAL | VT_R4 | VT_R8 |
                    VT_I1 | VT_UI2 | VT_I4 | VT_UI4 |
                    VT_INT | VT_UINT | VT_UI8 => {
                        (*V_VT_ref(pVarOut)) = VT_I4;
                    }
                    VT_I8 => {
                        (*V_VT_ref(pVarOut)) = VT_I8;
                    }
                    _ => {
                        hRet = DISP_E_BADVARTYPE;
                        break 'VarOr_Exit;
                    }
                }
                hRet = VariantCopy(&mut varLeft, pVarLeft);
                if FAILED(hRet) { break 'VarOr_Exit; }
                pVarLeft = &mut varLeft;
                hRet = VariantChangeType(pVarOut, pVarLeft, 0, *V_VT_ref(pVarOut) );
                break 'VarOr_Exit;
            }
        }

        if (*V_VT_ref(pVarLeft)) == VT_BOOL && (*V_VT_ref(pVarRight)) == VT_BOOL {
            (*V_VT_ref(pVarOut)) = VT_BOOL;
            (*V_BOOL_ref(pVarOut)) = (*V_BOOL_ref(pVarLeft)) | (*V_BOOL_ref(pVarRight));
            hRet = S_OK;
            break 'VarOr_Exit;
        }

        if (*V_VT_ref(pVarLeft)) == VT_UI1 && (*V_VT_ref(pVarRight)) == VT_UI1 {
            (*V_VT_ref(pVarOut)) = VT_UI1;
            (*V_UI1_ref(pVarOut)) = (*V_UI1_ref(pVarLeft)) | (*V_UI1_ref(pVarRight));
            hRet = S_OK;
            break 'VarOr_Exit;
        }

        if (*V_VT_ref(pVarLeft)) == VT_BSTR {
            hRet = VariantCopy(&mut varStr, pVarLeft);
            if FAILED(hRet) { break 'VarOr_Exit; }
            pVarLeft = &mut varStr;
            hRet = VariantChangeType(pVarLeft, pVarLeft, 0, VT_BOOL);
            if FAILED(hRet) { break 'VarOr_Exit; }
        }

        if (*V_VT_ref(pVarLeft)) == VT_BOOL &&
            ((*V_VT_ref(pVarRight)) == VT_BOOL || (*V_VT_ref(pVarRight)) == VT_BSTR)
        {
            vt = VT_BOOL;
        }
        else if ((*V_VT_ref(pVarLeft)) == VT_BOOL || (*V_VT_ref(pVarLeft)) == VT_UI1 ||
            (*V_VT_ref(pVarLeft)) == VT_I2 || (*V_VT_ref(pVarLeft)) == VT_BSTR) &&
            ((*V_VT_ref(pVarRight)) == VT_BOOL || (*V_VT_ref(pVarRight)) == VT_UI1 ||
            (*V_VT_ref(pVarRight)) == VT_I2 || (*V_VT_ref(pVarRight)) == VT_BSTR)
        {
            vt = VT_I2;
        }
        else if (*V_VT_ref(pVarLeft)) == VT_I8 || (*V_VT_ref(pVarRight)) == VT_I8 {
            if (*V_VT_ref(pVarLeft)) == VT_INT || (*V_VT_ref(pVarRight)) == VT_INT {
                hRet = DISP_E_TYPEMISMATCH;
                break 'VarOr_Exit;
            }
            vt = VT_I8;
        }

        hRet = VariantCopy(&mut varLeft, pVarLeft);
        if FAILED(hRet) { break 'VarOr_Exit; }

        hRet = VariantCopy(&mut varRight, pVarRight);
        if FAILED(hRet) { break 'VarOr_Exit; }

        if vt == VT_I4 && (*V_VT_ref(&mut varLeft)) == VT_UI4 {
            (*V_VT_ref(&mut varLeft)) = VT_I4;
        }
        else {
            let mut d: f64 = 0.0;

            if (*V_VT_ref(&mut varLeft)) == VT_BSTR &&
                FAILED(VarR8FromStr(*V_BSTR_ref(&mut varLeft) , LOCALE_USER_DEFAULT, 0, &mut d)) {
                hRet = VariantChangeType(&mut varLeft, &mut varLeft, VARIANT_LOCALBOOL as USHORT, VT_BOOL);
            }
            if SUCCEEDED(hRet) && (*V_VT_ref(&mut varLeft)) != vt {
                hRet = VariantChangeType(&mut varLeft, &mut varLeft, 0, vt);
            }
            if FAILED(hRet) { break 'VarOr_Exit; }
        }

        if vt == VT_I4 && (*V_VT_ref(&mut varRight)) == VT_UI4 {
            (*V_VT_ref(&mut varRight)) = VT_I4;
        }
        else {
            let mut d: f64 = 0.0;

            if (*V_VT_ref(&mut varRight)) == VT_BSTR &&
                FAILED(VarR8FromStr(*V_BSTR_ref(&mut varRight) , LOCALE_USER_DEFAULT, 0, &mut d)) {
                hRet = VariantChangeType(&mut varRight, &mut varRight, VARIANT_LOCALBOOL as USHORT, VT_BOOL);
            }
            if SUCCEEDED(hRet) && (*V_VT_ref(&mut varRight)) != vt {
                hRet = VariantChangeType(&mut varRight, &mut varRight, 0, vt);
            }
            if FAILED(hRet) { break 'VarOr_Exit; }
        }

        (*V_VT_ref(pVarOut)) = vt;
        if vt == VT_I8 {
            (*V_I8_ref(pVarOut)) = (*V_I8_ref(&mut varLeft)) | (*V_I8_ref(&mut varRight));
        }
        else if vt == VT_I4 {
            (*V_I4_ref(pVarOut)) = (*V_I4_ref(&mut varLeft)) | (*V_I4_ref(&mut varRight));
        }
        else {
            (*V_I2_ref(pVarOut)) = (*V_I2_ref(&mut varLeft)) | (*V_I2_ref(&mut varRight));
        }

        hRet = S_OK;
    }

    VariantClear(&mut varStr);
    VariantClear(&mut varLeft);
    VariantClear(&mut varRight);
    VariantClear(&mut tempLeft);
    VariantClear(&mut tempRight);
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarAbs(pVarIn: LPVARIANT, pVarOut: LPVARIANT) -> HRESULT {
    let mut hRet: HRESULT = S_OK;
    let mut temp: VARIANT = core::mem::zeroed();

    VariantInit(&mut temp);

    TRACE!(b"(%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(pVarIn), pVarOut);

    let mut pVarIn = pVarIn;

    if ((*V_VT_ref(pVarIn)) & VT_TYPEMASK) == VT_DISPATCH && (((*V_VT_ref(pVarIn)) & !VT_TYPEMASK) == 0) {
        hRet = VARIANT_FetchDispatchValue(pVarIn, &mut temp);
        if FAILED(hRet) {
            VariantClear(&mut temp);
            return hRet;
        }
        pVarIn = &mut temp;
    }

    if V_ISARRAY(pVarIn) || (*V_VT_ref(pVarIn)) == VT_UNKNOWN ||
        (*V_VT_ref(pVarIn)) == VT_DISPATCH || (*V_VT_ref(pVarIn)) == VT_RECORD ||
        (*V_VT_ref(pVarIn)) == VT_ERROR
    {
        hRet = DISP_E_TYPEMISMATCH;
        VariantClear(&mut temp);
        return hRet;
    }

    *pVarOut = *pVarIn;

    match *V_VT_ref(pVarIn)  {
        VT_I1 => {
            if (*V_I1_ref(pVarIn)) == I1_MIN as i8 {
                hRet = DISP_E_OVERFLOW;
            } else if (*V_I1_ref(pVarIn)) < 0 {
                (*V_I1_ref(pVarOut)) = -(*V_I1_ref(pVarIn));
            }
        }
        VT_BOOL => {
            (*V_VT_ref(pVarOut)) = VT_I2;
            if (*V_I2_ref(pVarIn)) == I2_MIN as i16 {
                hRet = DISP_E_OVERFLOW;
            } else if (*V_I2_ref(pVarIn)) < 0 {
                (*V_I2_ref(pVarOut)) = -(*V_I2_ref(pVarIn));
            }
        }
        VT_I2 => {
            if (*V_I2_ref(pVarIn)) == I2_MIN as i16 {
                hRet = DISP_E_OVERFLOW;
            } else if (*V_I2_ref(pVarIn)) < 0 {
                (*V_I2_ref(pVarOut)) = -(*V_I2_ref(pVarIn));
            }
        }
        VT_INT | VT_I4 => {
            if (*V_I4_ref(pVarIn)) == I4_MIN as i32 {
                hRet = DISP_E_OVERFLOW;
            } else if (*V_I4_ref(pVarIn)) < 0 {
                (*V_I4_ref(pVarOut)) = -(*V_I4_ref(pVarIn));
            }
        }
        VT_I8 => {
            if (*V_I8_ref(pVarIn)) == I8_MIN {
                hRet = DISP_E_OVERFLOW;
            } else if (*V_I8_ref(pVarIn)) < 0 {
                (*V_I8_ref(pVarOut)) = -(*V_I8_ref(pVarIn));
            }
        }
        // Wine 10.0. Floats have no unrepresentable minimum, so the old
        // `== R4_MIN/R8_MIN -> DISP_E_OVERFLOW` (R4_MIN/R8_MIN are the smallest
        // DENORMALS) rejected a valid input. And a BSTR is parsed straight into
        // pVarOut: the old code parsed into a local and wrote pVarOut only for a
        // negative value, so VarAbs("5") returned VT_R8 holding the shallow-
        // copied BSTR pointer's bits.
        VT_R4 => {
            if (*V_R4_ref(pVarOut)) < 0.0 {
                (*V_R4_ref(pVarOut)) = -(*V_R4_ref(pVarOut));
            }
        }
        VT_BSTR => {
            hRet = VarR8FromStr(*V_BSTR_ref(pVarIn), LOCALE_USER_DEFAULT, 0, V_R8_ref(pVarOut));
            if !FAILED(hRet) {
                (*V_VT_ref(pVarOut)) = VT_R8;
                if (*V_R8_ref(pVarOut)) < 0.0 {
                    (*V_R8_ref(pVarOut)) = -(*V_R8_ref(pVarOut));
                }
            }
        }
        VT_DATE | VT_R8 => {
            if (*V_R8_ref(pVarOut)) < 0.0 {
                (*V_R8_ref(pVarOut)) = -(*V_R8_ref(pVarOut));
            }
        }
        VT_CY => {
            hRet = VarCyAbs(*V_CY_ref(pVarIn) , core::ptr::addr_of_mut!((*V_CY_ref(pVarOut))));
        }
        VT_DECIMAL => {
            (*V_DECIMAL_ref(pVarOut)).__bindgen_anon_1.__bindgen_anon_1.sign &= !DECIMAL_NEG;
        }
        VT_UI1 | VT_UI2 | VT_UINT | VT_UI4 | VT_UI8 => {
        }
        VT_EMPTY => {
            (*V_VT_ref(pVarOut)) = VT_I2;
            (*V_I2_ref(pVarOut)) = 0;
        }
        VT_NULL => {
            (*V_I2_ref(pVarOut)) = 0;
        }
        _ => {
            hRet = DISP_E_BADVARTYPE;
        }
    }

    VariantClear(&mut temp);
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarFix(pVarIn: LPVARIANT, pVarOut: LPVARIANT) -> HRESULT {
    let mut hRet: HRESULT = S_OK;
    let mut temp: VARIANT = core::mem::zeroed();

    VariantInit(&mut temp);

    TRACE!(b"(%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(pVarIn), pVarOut);

    let mut pVarIn = pVarIn;

    if ((*V_VT_ref(pVarIn)) & VT_TYPEMASK) == VT_DISPATCH && (((*V_VT_ref(pVarIn)) & !VT_TYPEMASK) == 0) {
        hRet = VARIANT_FetchDispatchValue(pVarIn, &mut temp);
        if FAILED(hRet) {
            (*V_VT_ref(pVarOut)) = VT_EMPTY;
            VariantClear(&mut temp);
            return hRet;
        }
        pVarIn = &mut temp;
    }
    (*V_VT_ref(pVarOut)) = *V_VT_ref(pVarIn) ;

    match *V_VT_ref(pVarIn)  {
        VT_UI1 => {
            (*V_UI1_ref(pVarOut)) = *V_UI1_ref(pVarIn) ;
        }
        VT_BOOL => {
            (*V_VT_ref(pVarOut)) = VT_I2;
            (*V_I2_ref(pVarOut)) = *V_I2_ref(pVarIn) ;
        }
        VT_I2 => {
            (*V_I2_ref(pVarOut)) = *V_I2_ref(pVarIn) ;
        }
        VT_I4 => {
            (*V_I4_ref(pVarOut)) = *V_I4_ref(pVarIn) ;
        }
        VT_I8 => {
            (*V_I8_ref(pVarOut)) = *V_I8_ref(pVarIn) ;
        }
        VT_R4 => {
            if (*V_R4_ref(pVarIn)) < 0.0f32 {
                (*V_R4_ref(pVarOut)) = ceil((*V_R4_ref(pVarIn)) as f64) as f32;
            } else {
                (*V_R4_ref(pVarOut)) = floor((*V_R4_ref(pVarIn)) as f64) as f32;
            }
        }
        VT_BSTR => {
            (*V_VT_ref(pVarOut)) = VT_R8;
            hRet = VarR8FromStr(*V_BSTR_ref(pVarIn) , LOCALE_USER_DEFAULT, 0, &mut (*V_R8_ref(pVarOut)));
            pVarIn = pVarOut;
            if (*V_R8_ref(pVarIn)) < 0.0 {
                (*V_R8_ref(pVarOut)) = ceil(*V_R8_ref(pVarIn) );
            } else {
                (*V_R8_ref(pVarOut)) = floor(*V_R8_ref(pVarIn) );
            }
        }
        VT_DATE | VT_R8 => {
            if (*V_R8_ref(pVarIn)) < 0.0 {
                (*V_R8_ref(pVarOut)) = ceil(*V_R8_ref(pVarIn) );
            } else {
                (*V_R8_ref(pVarOut)) = floor(*V_R8_ref(pVarIn) );
            }
        }
        VT_CY => {
            hRet = VarCyFix(*V_CY_ref(pVarIn) , &mut (*V_CY_ref(pVarOut)));
        }
        VT_DECIMAL => {
            hRet = VarDecFix(&(*V_DECIMAL_ref(pVarIn)), &mut (*V_DECIMAL_ref(pVarOut)));
        }
        VT_EMPTY => {
            (*V_VT_ref(pVarOut)) = VT_I2;
            (*V_I2_ref(pVarOut)) = 0;
        }
        VT_NULL => {
        }
        _ => {
            if V_TYPE(pVarIn) == VT_CLSID || FAILED(VARIANT_ValidateType(*V_VT_ref(pVarIn) )) {
                hRet = DISP_E_BADVARTYPE;
            } else {
                hRet = DISP_E_TYPEMISMATCH;
            }
        }
    }

    if FAILED(hRet) {
        (*V_VT_ref(pVarOut)) = VT_EMPTY;
    }
    VariantClear(&mut temp);

    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarInt(pVarIn: LPVARIANT, pVarOut: LPVARIANT) -> HRESULT {

    let mut hRet: HRESULT = 0;
    let mut temp: VARIANT = core::mem::zeroed();
    let mut pVarIn = pVarIn;

    VariantInit(&mut temp);

    TRACE!(b"(%s,%p)\n\0".as_ptr(), debugstr_variant(pVarIn), pVarOut);

    if ((*V_VT_ref(pVarIn)) & 0xfff) == 9 && (((*V_VT_ref(pVarIn)) & !0xfff) == 0) {
        hRet = VARIANT_FetchDispatchValue(pVarIn, &mut temp);
        if FAILED(hRet) {
            VariantClear(&mut temp);
            return hRet;
        }
        pVarIn = &mut temp;
    }
    (*V_VT_ref(pVarOut)) = *V_VT_ref(pVarIn) ;

    match *V_VT_ref(pVarIn)  {
        4 => {
            (*V_R4_ref(pVarOut)) = floor((*V_R4_ref(pVarIn)) as f64) as f32;
        }
        8 => {
            (*V_VT_ref(pVarOut)) = 5;
            hRet = VarR8FromStr(*V_BSTR_ref(pVarIn) , 0x0400, 0, V_R8_ref(pVarOut));
            pVarIn = pVarOut;
            (*V_R8_ref(pVarOut)) = floor(*V_R8_ref(pVarIn) );
        }
        7 | 5 => {
            (*V_R8_ref(pVarOut)) = floor(*V_R8_ref(pVarIn) );
        }
        6 => {
            hRet = VarCyInt(*V_CY_ref(pVarIn) , V_CY_ref(pVarOut));
        }
        14 => {
            hRet = VarDecInt(V_DECIMAL_ref(pVarIn), V_DECIMAL_ref(pVarOut));
        }
        _ => {
            hRet = VarFix(pVarIn, pVarOut);
        }
    }

    VariantClear(&mut temp);

    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarXor(pVarLeft: LPVARIANT, pVarRight: LPVARIANT, pVarOut: LPVARIANT) -> HRESULT {
    let vt: VARTYPE;
    let mut varLeft: VARIANT = core::mem::zeroed();
    let mut varRight: VARIANT = core::mem::zeroed();
    let mut tempLeft: VARIANT = core::mem::zeroed();
    let mut tempRight: VARIANT = core::mem::zeroed();
    let mut d: f64 = 0.0;
    let mut hRet: HRESULT;

    TRACE!(b"(%s,%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(pVarLeft), debugstr_variant(pVarRight), pVarOut);

    if V_EXTRA_TYPE(pVarLeft) != 0 || V_EXTRA_TYPE(pVarRight) != 0 ||
        (*V_VT_ref(pVarLeft)) > VT_UINT || (*V_VT_ref(pVarRight)) > VT_UINT ||
        (*V_VT_ref(pVarLeft)) == VT_VARIANT || (*V_VT_ref(pVarRight)) == VT_VARIANT ||
        (*V_VT_ref(pVarLeft)) == VT_UNKNOWN || (*V_VT_ref(pVarRight)) == VT_UNKNOWN ||
        (*V_VT_ref(pVarLeft)) == 15 || (*V_VT_ref(pVarRight)) == 15 ||
        (*V_VT_ref(pVarLeft)) == VT_ERROR || (*V_VT_ref(pVarRight)) == VT_ERROR {
        return DISP_E_BADVARTYPE;
    }

    if (*V_VT_ref(pVarLeft)) == VT_NULL || (*V_VT_ref(pVarRight)) == VT_NULL {
        (*V_VT_ref(pVarOut)) = VT_NULL;
        return S_OK;
    }

    VariantInit(&mut tempLeft);
    VariantInit(&mut tempRight);

    let mut pVarLeft = pVarLeft;
    let mut pVarRight = pVarRight;

    if ((*V_VT_ref(pVarLeft)) & VT_TYPEMASK) == VT_DISPATCH {
        hRet = VARIANT_FetchDispatchValue(pVarLeft, &mut tempLeft);
        if FAILED(hRet) {
            VariantClear(&mut varLeft);
            VariantClear(&mut varRight);
            VariantClear(&mut tempLeft);
            VariantClear(&mut tempRight);
            return hRet;
        }
        pVarLeft = &mut tempLeft;
    }
    if ((*V_VT_ref(pVarRight)) & VT_TYPEMASK) == VT_DISPATCH {
        hRet = VARIANT_FetchDispatchValue(pVarRight, &mut tempRight);
        if FAILED(hRet) {
            VariantClear(&mut varLeft);
            VariantClear(&mut varRight);
            VariantClear(&mut tempLeft);
            VariantClear(&mut tempRight);
            return hRet;
        }
        pVarRight = &mut tempRight;
    }

    (*V_VT_ref(&mut varLeft)) = VT_EMPTY;
    (*V_VT_ref(&mut varRight)) = VT_EMPTY;

    hRet = VariantCopy(&mut varLeft, pVarLeft);
    if FAILED(hRet) {
        VariantClear(&mut varLeft);
        VariantClear(&mut varRight);
        VariantClear(&mut tempLeft);
        VariantClear(&mut tempRight);
        return hRet;
    }

    hRet = VariantCopy(&mut varRight, pVarRight);
    if FAILED(hRet) {
        VariantClear(&mut varLeft);
        VariantClear(&mut varRight);
        VariantClear(&mut tempLeft);
        VariantClear(&mut tempRight);
        return hRet;
    }

    if (*V_VT_ref(&mut varLeft)) == VT_BSTR {
        hRet = VarR8FromStr(*V_BSTR_ref(&mut varLeft) , LOCALE_USER_DEFAULT, 0, &mut d);
        hRet = VariantChangeType(&mut varLeft, &mut varLeft, VARIANT_LOCALBOOL as USHORT,
                                 if FAILED(hRet) { VT_BOOL } else { VT_I4 });
        if FAILED(hRet) {
            VariantClear(&mut varLeft);
            VariantClear(&mut varRight);
            VariantClear(&mut tempLeft);
            VariantClear(&mut tempRight);
            return hRet;
        }
    }

    if (*V_VT_ref(&mut varRight)) == VT_BSTR {
        hRet = VarR8FromStr(*V_BSTR_ref(&mut varRight) , LOCALE_USER_DEFAULT, 0, &mut d);
        hRet = VariantChangeType(&mut varRight, &mut varRight, VARIANT_LOCALBOOL as USHORT,
                                 if FAILED(hRet) { VT_BOOL } else { VT_I4 });
        if FAILED(hRet) {
            VariantClear(&mut varLeft);
            VariantClear(&mut varRight);
            VariantClear(&mut tempLeft);
            VariantClear(&mut tempRight);
            return hRet;
        }
    }

    if (*V_VT_ref(&mut varLeft)) == VT_I8 || (*V_VT_ref(&mut varRight)) == VT_I8 {
        if (*V_VT_ref(pVarLeft)) == VT_INT || (*V_VT_ref(pVarRight)) == VT_INT {
            hRet = DISP_E_TYPEMISMATCH;
            VariantClear(&mut varLeft);
            VariantClear(&mut varRight);
            VariantClear(&mut tempLeft);
            VariantClear(&mut tempRight);
            return hRet;
        }
        vt = VT_I8;
    } else {
        match ((*V_VT_ref(&mut varLeft)) as u32) << 16 | (*V_VT_ref(&mut varRight)) as u32 {
            720907 /* (720907 /* (720907 /* ((VT_BOOL << 16) | VT_BOOL) */) */) */ => {
                vt = VT_BOOL;
            }
            1114129 /* (1114129 /* (1114129 /* ((VT_UI1 << 16) | VT_UI1) */) */) */ => {
                vt = VT_UI1;
            }
            0 /* (0 /* (0 /* ((VT_EMPTY << 16) | VT_EMPTY) */) */) */ |
            17 /* (17 /* (17 /* ((VT_EMPTY << 16) | VT_UI1) */) */) */ |
            2 /* (2 /* (2 /* ((VT_EMPTY << 16) | VT_I2) */) */) */ |
            11 /* (11 /* (11 /* ((VT_EMPTY << 16) | VT_BOOL) */) */) */ |
            1114112 /* (1114112 /* (1114112 /* ((VT_UI1 << 16) | VT_EMPTY) */) */) */ |
            1114114 /* (1114114 /* (1114114 /* ((VT_UI1 << 16) | VT_I2) */) */) */ |
            1114123 /* (1114123 /* (1114123 /* ((VT_UI1 << 16) | VT_BOOL) */) */) */ |
            131072 /* (131072 /* (131072 /* ((VT_I2 << 16) | VT_EMPTY) */) */) */ |
            131089 /* (131089 /* (131089 /* ((VT_I2 << 16) | VT_UI1) */) */) */ |
            131074 /* (131074 /* (131074 /* ((VT_I2 << 16) | VT_I2) */) */) */ |
            131083 /* (131083 /* (131083 /* ((VT_I2 << 16) | VT_BOOL) */) */) */ |
            720896 /* (720896 /* (720896 /* ((VT_BOOL << 16) | VT_EMPTY) */) */) */ |
            720913 /* (720913 /* (720913 /* ((VT_BOOL << 16) | VT_UI1) */) */) */ |
            720898 /* (720898 /* (720898 /* ((VT_BOOL << 16) | VT_I2) */) */) */ => {
                vt = VT_I2;
            }
            _ => {
                vt = VT_I4;
            }
        }
    }

    if vt != VT_I8 {
        if (*V_VT_ref(&mut varLeft)) == VT_UI4 {
            (*V_VT_ref(&mut varLeft)) = VT_I4;
        }
        if (*V_VT_ref(&mut varRight)) == VT_UI4 {
            (*V_VT_ref(&mut varRight)) = VT_I4;
        }
    }

    if (*V_VT_ref(&mut varLeft)) != vt {
        hRet = VariantChangeType(&mut varLeft, &mut varLeft, 0, vt);
    }
    if FAILED(hRet) {
        VariantClear(&mut varLeft);
        VariantClear(&mut varRight);
        VariantClear(&mut tempLeft);
        VariantClear(&mut tempRight);
        return hRet;
    }

    if (*V_VT_ref(&mut varRight)) != vt {
        hRet = VariantChangeType(&mut varRight, &mut varRight, 0, vt);
    }
    if FAILED(hRet) {
        VariantClear(&mut varLeft);
        VariantClear(&mut varRight);
        VariantClear(&mut tempLeft);
        VariantClear(&mut tempRight);
        return hRet;
    }

    (*V_VT_ref(pVarOut)) = vt;

    match vt {
        VT_I8 => {
            (*V_I8_ref(pVarOut)) = (*V_I8_ref(&mut varLeft)) ^ (*V_I8_ref(&mut varRight));
        }
        VT_I4 => {
            (*V_I4_ref(pVarOut)) = (*V_I4_ref(&mut varLeft)) ^ (*V_I4_ref(&mut varRight));
        }
        VT_BOOL | VT_I2 => {
            (*V_I2_ref(pVarOut)) = (*V_I2_ref(&mut varLeft)) ^ (*V_I2_ref(&mut varRight));
        }
        VT_UI1 => {
            (*V_UI1_ref(pVarOut)) = (*V_UI1_ref(&mut varLeft)) ^ (*V_UI1_ref(&mut varRight));
        }
        _ => {}
    }

    VariantClear(&mut varLeft);
    VariantClear(&mut varRight);
    VariantClear(&mut tempLeft);
    VariantClear(&mut tempRight);
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarEqv(pVarLeft: LPVARIANT, pVarRight: LPVARIANT, pVarOut: LPVARIANT) -> HRESULT {
    let hRet: HRESULT;

    TRACE!(b"(%s,%s,%p)\n\0".as_ptr(), debugstr_variant(pVarLeft), debugstr_variant(pVarRight), pVarOut);

    hRet = VarXor(pVarLeft, pVarRight, pVarOut);
    if SUCCEEDED(hRet) {
        if (*V_VT_ref(pVarOut)) == VT_I8 {
            (*V_I8_ref(pVarOut)) = !(*V_I8_ref(pVarOut));
        } else {
            (*V_UI4_ref(pVarOut)) = !(*V_UI4_ref(pVarOut));
        }
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn VarNeg(pVarIn: LPVARIANT, pVarOut: LPVARIANT) -> HRESULT {
    let mut hRet: HRESULT = S_OK;
    let mut temp: VARIANT = core::mem::zeroed();

    VariantInit(&mut temp);

    TRACE!(b"(%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(pVarIn), pVarOut);

    let mut pVarIn = pVarIn;
    if ((*V_VT_ref(pVarIn)) & VT_TYPEMASK) == VT_DISPATCH && (((*V_VT_ref(pVarIn)) & !VT_TYPEMASK) == 0) {
        hRet = VARIANT_FetchDispatchValue(pVarIn, &mut temp);
        if FAILED(hRet) {
            (*V_VT_ref(pVarOut)) = VT_EMPTY;
            VariantClear(&mut temp);
            return hRet;
        }
        pVarIn = &mut temp;
    }
    (*V_VT_ref(pVarOut)) = *V_VT_ref(pVarIn) ;

    match *V_VT_ref(pVarIn)  {
        VT_UI1 => {
            (*V_VT_ref(pVarOut)) = VT_I2;
            (*V_I2_ref(pVarOut)) = -((*V_UI1_ref(pVarIn)) as i16);
        }
        VT_BOOL => {
            (*V_VT_ref(pVarOut)) = VT_I2;
            if (*V_I2_ref(pVarIn)) == I2_MIN as i16 {
                (*V_VT_ref(pVarOut)) = VT_I4;
                (*V_I4_ref(pVarOut)) = -((*V_I2_ref(pVarIn)) as i32);
            } else {
                (*V_I2_ref(pVarOut)) = -(*V_I2_ref(pVarIn));
            }
        }
        VT_I2 => {
            if (*V_I2_ref(pVarIn)) == I2_MIN as i16 {
                (*V_VT_ref(pVarOut)) = VT_I4;
                (*V_I4_ref(pVarOut)) = -((*V_I2_ref(pVarIn)) as i32);
            } else {
                (*V_I2_ref(pVarOut)) = -(*V_I2_ref(pVarIn));
            }
        }
        VT_I4 => {
            if (*V_I4_ref(pVarIn)) == I4_MIN as i32 {
                (*V_VT_ref(pVarOut)) = VT_R8;
                (*V_R8_ref(pVarOut)) = -((*V_I4_ref(pVarIn)) as f64);
            } else {
                (*V_I4_ref(pVarOut)) = -(*V_I4_ref(pVarIn));
            }
        }
        VT_I8 => {
            if (*V_I8_ref(pVarIn)) == I8_MIN {
                (*V_VT_ref(pVarOut)) = VT_R8;
                hRet = VarR8FromI8(*V_I8_ref(pVarIn) , &mut (*V_R8_ref(pVarOut)));
                (*V_R8_ref(pVarOut)) *= -1.0;
            } else {
                (*V_I8_ref(pVarOut)) = -(*V_I8_ref(pVarIn));
            }
        }
        VT_R4 => {
            (*V_R4_ref(pVarOut)) = -(*V_R4_ref(pVarIn));
        }
        VT_DATE | VT_R8 => {
            (*V_R8_ref(pVarOut)) = -(*V_R8_ref(pVarIn));
        }
        VT_CY => {
            hRet = VarCyNeg(*V_CY_ref(pVarIn) , &mut (*V_CY_ref(pVarOut)));
        }
        VT_DECIMAL => {
            hRet = VarDecNeg(&(*V_DECIMAL_ref(pVarIn)), &mut (*V_DECIMAL_ref(pVarOut)));
        }
        VT_BSTR => {
            (*V_VT_ref(pVarOut)) = VT_R8;
            hRet = VarR8FromStr(*V_BSTR_ref(pVarIn) , LOCALE_USER_DEFAULT, 0, &mut (*V_R8_ref(pVarOut)));
            (*V_R8_ref(pVarOut)) = -(*V_R8_ref(pVarOut));
        }
        VT_EMPTY => {
            (*V_VT_ref(pVarOut)) = VT_I2;
            (*V_I2_ref(pVarOut)) = 0;
        }
        VT_NULL => {
        }
        _ => {
            if V_TYPE(pVarIn) == VT_CLSID || FAILED(VARIANT_ValidateType(*V_VT_ref(pVarIn) )) {
                hRet = DISP_E_BADVARTYPE;
            } else {
                hRet = DISP_E_TYPEMISMATCH;
            }
        }
    }

    if FAILED(hRet) {
        (*V_VT_ref(pVarOut)) = VT_EMPTY;
    }
    VariantClear(&mut temp);

    hRet
}

#[no_mangle]
pub unsafe extern "system" fn VarNot(pVarIn: LPVARIANT, pVarOut: LPVARIANT) -> HRESULT {
    let mut varIn: VARIANT = core::mem::zeroed();
    let mut hRet: HRESULT = S_OK;
    let mut temp: VARIANT = core::mem::zeroed();

    VariantInit(&mut temp);

    TRACE!(b"(%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(pVarIn), pVarOut);

    let mut pVarIn = pVarIn;

    if ((*V_VT_ref(pVarIn)) & VT_TYPEMASK) == VT_DISPATCH && (((*V_VT_ref(pVarIn)) & !VT_TYPEMASK) == 0) {
        hRet = VARIANT_FetchDispatchValue(pVarIn, &mut temp);
        if FAILED(hRet) {
            if FAILED(hRet) {
                (*V_VT_ref(pVarOut)) = VT_EMPTY;
            }
            VariantClear(&mut temp);
            return hRet;
        }
        pVarIn = &mut temp;
    }

    if (*V_VT_ref(pVarIn)) == VT_BSTR {
        (*V_VT_ref(&mut varIn)) = VT_R8;
        hRet = VarR8FromStr(*V_BSTR_ref(pVarIn) , LOCALE_USER_DEFAULT, 0, V_R8_ref(&mut varIn));
        if FAILED(hRet) {
            (*V_VT_ref(&mut varIn)) = VT_BOOL;
            hRet = VarBoolFromStr(*V_BSTR_ref(pVarIn) , LOCALE_USER_DEFAULT, VAR_LOCALBOOL, V_BOOL_ref(&mut varIn));
        }
        if FAILED(hRet) {
            if FAILED(hRet) {
                (*V_VT_ref(pVarOut)) = VT_EMPTY;
            }
            VariantClear(&mut temp);
            return hRet;
        }
        pVarIn = &mut varIn;
    }

    (*V_VT_ref(pVarOut)) = *V_VT_ref(pVarIn) ;

    match *V_VT_ref(pVarIn)  {
        VT_I1 => {
            (*V_I4_ref(pVarOut)) = !(*V_I1_ref(pVarIn)) as i32;
            (*V_VT_ref(pVarOut)) = VT_I4;
        }
        VT_UI1 => {
            (*V_UI1_ref(pVarOut)) = !(*V_UI1_ref(pVarIn));
        }
        VT_BOOL | VT_I2 => {
            (*V_I2_ref(pVarOut)) = !(*V_I2_ref(pVarIn));
        }
        VT_UI2 => {
            // C promotes the USHORT to int before `~`: ~0 is -1, not 0xFFFF
            (*V_I4_ref(pVarOut)) = !(*V_UI2_ref(pVarIn) as i32);
            (*V_VT_ref(pVarOut)) = VT_I4;
        }
        VT_DECIMAL => {
            hRet = VarI4FromDec(V_DECIMAL_ref(pVarIn), V_I4_ref(&mut varIn));
            if FAILED(hRet) {
                if FAILED(hRet) {
                    (*V_VT_ref(pVarOut)) = VT_EMPTY;
                }
                VariantClear(&mut temp);
                return hRet;
            }
            pVarIn = &mut varIn;
            (*V_VT_ref(pVarOut)) = VT_I4;
            (*V_I4_ref(pVarOut)) = !(*V_I4_ref(pVarIn));
        }
        VT_INT => {
            (*V_VT_ref(pVarOut)) = VT_I4;
            (*V_I4_ref(pVarOut)) = !(*V_I4_ref(pVarIn));
        }
        VT_I4 => {
            (*V_I4_ref(pVarOut)) = !(*V_I4_ref(pVarIn));
        }
        VT_UINT | VT_UI4 => {
            (*V_I4_ref(pVarOut)) = !(*V_UI4_ref(pVarIn)) as i32;
            (*V_VT_ref(pVarOut)) = VT_I4;
        }
        VT_I8 => {
            (*V_I8_ref(pVarOut)) = !(*V_I8_ref(pVarIn));
        }
        VT_UI8 => {
            (*V_I4_ref(pVarOut)) = !(*V_UI8_ref(pVarIn)) as i32;
            (*V_VT_ref(pVarOut)) = VT_I4;
        }
        VT_R4 => {
            hRet = VarI4FromR4(*V_R4_ref(pVarIn) , V_I4_ref(pVarOut));
            (*V_I4_ref(pVarOut)) = !(*V_I4_ref(pVarOut));
            (*V_VT_ref(pVarOut)) = VT_I4;
        }
        VT_DATE | VT_R8 => {
            hRet = VarI4FromR8(*V_R8_ref(pVarIn) , V_I4_ref(pVarOut));
            (*V_I4_ref(pVarOut)) = !(*V_I4_ref(pVarOut));
            (*V_VT_ref(pVarOut)) = VT_I4;
        }
        VT_CY => {
            hRet = VarI4FromCy(*V_CY_ref(pVarIn) , V_I4_ref(pVarOut));
            (*V_I4_ref(pVarOut)) = !(*V_I4_ref(pVarOut));
            (*V_VT_ref(pVarOut)) = VT_I4;
        }
        VT_EMPTY => {
            (*V_I2_ref(pVarOut)) = !0i16;
            (*V_VT_ref(pVarOut)) = VT_I2;
        }
        VT_NULL => {
            // No-Op
        }
        _ => {
            if V_TYPE(pVarIn) == VT_CLSID || FAILED(VARIANT_ValidateType(*V_VT_ref(pVarIn) )) {
                hRet = DISP_E_BADVARTYPE;
            } else {
                hRet = DISP_E_TYPEMISMATCH;
            }
        }
    }

    if FAILED(hRet) {
        (*V_VT_ref(pVarOut)) = VT_EMPTY;
    }
    VariantClear(&mut temp);

    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn VarRound(pVarIn: LPVARIANT, deci: i32, pVarOut: LPVARIANT) -> HRESULT {
    let mut varIn: VARIANT = core::mem::zeroed();
    let mut hRet: HRESULT = S_OK;
    let factor: f32;
    let mut temp: VARIANT = core::mem::zeroed();

    VariantInit(&mut temp);

    TRACE!(b"(%s,%d)\n\0".as_ptr() as *const i8, debugstr_variant(pVarIn), deci);

    let mut pVarIn = pVarIn;

    if ((*V_VT_ref(pVarIn)) & VT_TYPEMASK) == VT_DISPATCH && (((*V_VT_ref(pVarIn)) & !VT_TYPEMASK) == 0) {
        hRet = VARIANT_FetchDispatchValue(pVarIn, &mut temp);
        if FAILED(hRet) {
            VarRound_Exit(hRet, pVarOut, &mut temp);
            return hRet;
        }
        pVarIn = &mut temp;
    }

    match *V_VT_ref(pVarIn)  {
        VT_I1 | VT_I8 | VT_UI2 | VT_UI4 => {
            hRet = DISP_E_BADVARTYPE;
        }

        VT_UI1 => {
            (*V_VT_ref(pVarOut)) = *V_VT_ref(pVarIn) ;
            (*V_UI1_ref(pVarOut)) = *V_UI1_ref(pVarIn) ;
        }
        VT_I2 => {
            (*V_VT_ref(pVarOut)) = *V_VT_ref(pVarIn) ;
            (*V_I2_ref(pVarOut)) = *V_I2_ref(pVarIn) ;
        }
        VT_I4 => {
            (*V_VT_ref(pVarOut)) = *V_VT_ref(pVarIn) ;
            (*V_I4_ref(pVarOut)) = *V_I4_ref(pVarIn) ;
        }
        VT_NULL => {
            (*V_VT_ref(pVarOut)) = *V_VT_ref(pVarIn) ;
        }

        VT_EMPTY => {
            (*V_VT_ref(pVarOut)) = VT_I2;
            (*V_I2_ref(pVarOut)) = 0;
        }
        VT_BOOL => {
            (*V_VT_ref(pVarOut)) = VT_I2;
            (*V_I2_ref(pVarOut)) = *V_BOOL_ref(pVarIn) ;
        }
        VT_BSTR => {
            hRet = VarR8FromStr(*V_BSTR_ref(pVarIn) , LOCALE_USER_DEFAULT, 0, V_R8_ref(&mut varIn));
            if FAILED(hRet) {
            } else {
                (*V_VT_ref(&mut varIn)) = VT_R8;
                pVarIn = &mut varIn;

                if (*V_R8_ref(pVarIn)) > 0.0 {
                    (*V_R8_ref(pVarOut)) = floor((*V_R8_ref(pVarIn)) * pow(10.0, deci as f64) + 0.5) / pow(10.0, deci as f64);
                } else {
                    (*V_R8_ref(pVarOut)) = ceil((*V_R8_ref(pVarIn)) * pow(10.0, deci as f64) - 0.5) / pow(10.0, deci as f64);
                }
                (*V_VT_ref(pVarOut)) = *V_VT_ref(pVarIn) ;
            }
        }

        VT_R8 => {
            if (*V_R8_ref(pVarIn)) > 0.0 {
                (*V_R8_ref(pVarOut)) = floor((*V_R8_ref(pVarIn)) * pow(10.0, deci as f64) + 0.5) / pow(10.0, deci as f64);
            } else {
                (*V_R8_ref(pVarOut)) = ceil((*V_R8_ref(pVarIn)) * pow(10.0, deci as f64) - 0.5) / pow(10.0, deci as f64);
            }
            (*V_VT_ref(pVarOut)) = *V_VT_ref(pVarIn) ;
        }
        VT_R4 => {
            if (*V_R4_ref(pVarIn)) > 0.0 {
                (*V_R4_ref(pVarOut)) = (floor(((*V_R4_ref(pVarIn)) * pow(10.0, deci as f64) as f32 + 0.5) as f64) / pow(10.0, deci as f64)) as f32;
            } else {
                (*V_R4_ref(pVarOut)) = (ceil(((*V_R4_ref(pVarIn)) * pow(10.0, deci as f64) as f32 - 0.5) as f64) / pow(10.0, deci as f64)) as f32;
            }
            (*V_VT_ref(pVarOut)) = *V_VT_ref(pVarIn) ;
        }
        VT_DATE => {
            if (*V_DATE_ref(pVarIn)) > 0.0 {
                (*V_DATE_ref(pVarOut)) = floor((*V_DATE_ref(pVarIn)) * pow(10.0, deci as f64) + 0.5) / pow(10.0, deci as f64);
            } else {
                (*V_DATE_ref(pVarOut)) = ceil((*V_DATE_ref(pVarIn)) * pow(10.0, deci as f64) - 0.5) / pow(10.0, deci as f64);
            }
            (*V_VT_ref(pVarOut)) = *V_VT_ref(pVarIn) ;
        }
        VT_CY => {
            if deci > 3 {
                factor = 1.0;
            } else {
                factor = pow(10.0, (4 - deci) as f64) as f32;
            }

            if ((*V_CY_ref(pVarIn))).int64 > 0 {
                ((*V_CY_ref(pVarOut))).int64 = (floor(((*V_CY_ref(pVarIn))).int64 as f64 / factor as f64) * factor as f64) as i64;
            } else {
                ((*V_CY_ref(pVarOut))).int64 = (ceil(((*V_CY_ref(pVarIn))).int64 as f64 / factor as f64) * factor as f64) as i64;
            }
            (*V_VT_ref(pVarOut)) = *V_VT_ref(pVarIn) ;
        }
        VT_DECIMAL => {
            let mut dbl: f64 = 0.0;

            hRet = VarR8FromDec(V_DECIMAL_ref(pVarIn), &mut dbl);
            if FAILED(hRet) {
            } else {
                if dbl > 0.0 {
                    dbl = floor(dbl * pow(10.0, deci as f64) + 0.5);
                } else {
                    dbl = ceil(dbl * pow(10.0, deci as f64) - 0.5);
                }

                (*V_VT_ref(pVarOut)) = VT_DECIMAL;
                hRet = VarDecFromR8(dbl, V_DECIMAL_ref(pVarOut));
            }
        }

        _ => {
            FIXME!(b"unimplemented part, (*V_VT_ref(pVarIn)) == 0x%X, deci == %d\n\0".as_ptr() as *const i8,
                  (*V_VT_ref(pVarIn)) & VT_TYPEMASK, deci);
            hRet = DISP_E_BADVARTYPE;
        }
    }

    VarRound_Exit(hRet, pVarOut, &mut temp);
    hRet
}

unsafe fn VarRound_Exit(hRet: HRESULT, pVarOut: LPVARIANT, temp: *mut VARIANT) {
    if FAILED(hRet) {
        (*V_VT_ref(pVarOut)) = VT_EMPTY;
    }
    VariantClear(temp);

    TRACE!(b"returning 0x%08x %s\n\0".as_ptr() as *const i8, hRet, debugstr_variant(pVarOut));
}

#[no_mangle]
pub unsafe extern "system" fn VarIdiv(left: LPVARIANT, right: LPVARIANT, result: LPVARIANT) -> HRESULT {
    let mut hres: HRESULT;
    let resvt: VARTYPE;
    let leftvt: VARTYPE;
    let rightvt: VARTYPE;
    let rightExtraFlags: VARTYPE;
    let leftExtraFlags: VARTYPE;
    let _ExtraFlags: VARTYPE;
    let mut lv: VARIANT = core::mem::zeroed();
    let mut rv: VARIANT = core::mem::zeroed();
    let mut tempLeft: VARIANT = core::mem::zeroed();
    let mut tempRight: VARIANT = core::mem::zeroed();

    TRACE!(b"(%s,%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(left), debugstr_variant(right), result);

    VariantInit(&mut lv);
    VariantInit(&mut rv);
    VariantInit(&mut tempLeft);
    VariantInit(&mut tempRight);

    leftvt = (*V_VT_ref(left)) & VT_TYPEMASK;
    rightvt = (*V_VT_ref(right)) & VT_TYPEMASK;
    leftExtraFlags = (*V_VT_ref(left)) & (!VT_TYPEMASK);
    rightExtraFlags = (*V_VT_ref(right)) & (!VT_TYPEMASK);

    'end: loop {
        if leftExtraFlags != rightExtraFlags {
            hres = DISP_E_BADVARTYPE;
            break 'end;
        }
        let ExtraFlags = leftExtraFlags;

        if (leftvt == VT_I8 && rightvt == VT_INT) ||
           (leftvt == VT_INT && rightvt == VT_I8) ||
           (rightvt == VT_EMPTY && leftvt != VT_NULL) ||
           ExtraFlags != 0 {
            hres = DISP_E_BADVARTYPE;
            break 'end;
        } else if leftvt == VT_NULL || rightvt == VT_NULL {
            (*V_VT_ref(result)) = VT_NULL;
            hres = S_OK;
            break 'end;
        } else if leftvt == VT_I8 || rightvt == VT_I8 {
            resvt = VT_I8;
        } else if leftvt == VT_I4 || rightvt == VT_I4 ||
                  leftvt == VT_INT || rightvt == VT_INT ||
                  leftvt == VT_UINT || rightvt == VT_UINT ||
                  leftvt == VT_UI8 || rightvt == VT_UI8 ||
                  leftvt == VT_UI4 || rightvt == VT_UI4 ||
                  leftvt == VT_UI2 || rightvt == VT_UI2 ||
                  leftvt == VT_I1 || rightvt == VT_I1 ||
                  leftvt == VT_BSTR || rightvt == VT_BSTR ||
                  leftvt == VT_DATE || rightvt == VT_DATE ||
                  leftvt == VT_CY || rightvt == VT_CY ||
                  leftvt == VT_DECIMAL || rightvt == VT_DECIMAL ||
                  leftvt == VT_R8 || rightvt == VT_R8 ||
                  leftvt == VT_R4 || rightvt == VT_R4 {
            resvt = VT_I4;
        } else if leftvt == VT_I2 || rightvt == VT_I2 ||
                  leftvt == VT_BOOL || rightvt == VT_BOOL ||
                  leftvt == VT_EMPTY {
            resvt = VT_I2;
        } else if leftvt == VT_UI1 || rightvt == VT_UI1 {
            resvt = VT_UI1;
        } else {
            hres = DISP_E_BADVARTYPE;
            break 'end;
        }

        hres = VariantChangeType(&mut lv, left, 0, resvt);
        if hres != S_OK { break 'end; }
        hres = VariantChangeType(&mut rv, right, 0, resvt);
        if hres != S_OK { break 'end; }

        (*V_VT_ref(result)) = resvt;
        match resvt {
            VT_UI1 => {
                if (*V_UI1_ref(&mut rv)) == 0 {
                    hres = DISP_E_DIVBYZERO;
                    (*V_VT_ref(result)) = VT_EMPTY;
                } else {
                    (*V_UI1_ref(result)) = (*V_UI1_ref(&mut lv)) / (*V_UI1_ref(&mut rv));
                }
            }
            VT_I2 => {
                if (*V_I2_ref(&mut rv)) == 0 {
                    hres = DISP_E_DIVBYZERO;
                    (*V_VT_ref(result)) = VT_EMPTY;
                } else {
                    (*V_I2_ref(result)) = (*V_I2_ref(&mut lv)) / (*V_I2_ref(&mut rv));
                }
            }
            VT_I4 => {
                if (*V_I4_ref(&mut rv)) == 0 {
                    hres = DISP_E_DIVBYZERO;
                    (*V_VT_ref(result)) = VT_EMPTY;
                } else {
                    (*V_I4_ref(result)) = (*V_I4_ref(&mut lv)) / (*V_I4_ref(&mut rv));
                }
            }
            VT_I8 => {
                if (*V_I8_ref(&mut rv)) == 0 {
                    hres = DISP_E_DIVBYZERO;
                    (*V_VT_ref(result)) = VT_EMPTY;
                } else {
                    (*V_I8_ref(result)) = (*V_I8_ref(&mut lv)) / (*V_I8_ref(&mut rv));
                }
            }
            _ => {
                FIXME!(b"Couldn't integer divide variant types %d,%d\n\0".as_ptr() as *const i8, leftvt, rightvt);
            }
        }

        break 'end;
    }

    VariantClear(&mut lv);
    VariantClear(&mut rv);
    VariantClear(&mut tempLeft);
    VariantClear(&mut tempRight);

    hres
}

#[no_mangle]
pub unsafe extern "system" fn VarMod(left: LPVARIANT, right: LPVARIANT, result: LPVARIANT) -> HRESULT {
    let lOk: BOOL;
    let mut rc: HRESULT;
    let resT: i32;
    let mut lv: VARIANT = core::mem::zeroed();
    let mut rv: VARIANT = core::mem::zeroed();
    let mut tempLeft: VARIANT = core::mem::zeroed();
    let mut tempRight: VARIANT = core::mem::zeroed();
    let mut left_ptr = left;
    let mut right_ptr = right;

    VariantInit(&mut tempLeft);
    VariantInit(&mut tempRight);
    VariantInit(&mut lv);
    VariantInit(&mut rv);
    // Wine 10.0: the early-error exits below return `result` untouched, so
    // give it a defined type first.
    (*V_VT_ref(result)) = VT_EMPTY;

    TRACE!(b"(%s,%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(left), debugstr_variant(right), result);

    'end: {
        if ((*V_VT_ref(left_ptr)) & VT_TYPEMASK) == VT_DISPATCH {
            rc = VARIANT_FetchDispatchValue(left_ptr, &mut tempLeft);
            if FAILED(rc) { break 'end; }
            left_ptr = &mut tempLeft;
        }
        if ((*V_VT_ref(right_ptr)) & VT_TYPEMASK) == VT_DISPATCH {
            rc = VARIANT_FetchDispatchValue(right_ptr, &mut tempRight);
            if FAILED(rc) { break 'end; }
            right_ptr = &mut tempRight;
        }

        lOk = 1;
        match (*V_VT_ref(left_ptr)) & VT_TYPEMASK {
            VT_BOOL | VT_I1 | VT_I2 | VT_I4 | VT_I8 | VT_INT | VT_UI1 | VT_UI2 | VT_UI4 | VT_UI8 |
            VT_UINT | VT_R4 | VT_R8 | VT_CY | VT_EMPTY | VT_DATE | VT_BSTR | VT_DECIMAL => {}
            VT_VARIANT | VT_UNKNOWN => {
                (*V_VT_ref(result)) = VT_EMPTY;
                rc = DISP_E_TYPEMISMATCH;
                break 'end;
            }
            VT_ERROR => {
                rc = DISP_E_TYPEMISMATCH;
                break 'end;
            }
            VT_RECORD => {
                (*V_VT_ref(result)) = VT_EMPTY;
                rc = DISP_E_TYPEMISMATCH;
                break 'end;
            }
            VT_NULL => {}
            _ => {
                (*V_VT_ref(result)) = VT_EMPTY;
                rc = DISP_E_BADVARTYPE;
                break 'end;
            }
        }

        // C: these cases FALL THROUGH into each other (BOOL..I8 -> INT -> UI1..CY
        // -> EMPTY/DATE/DECIMAL -> BSTR), so every one of them ends at BSTR's
        // "NULL % x is NULL" check. The translation gave each arm only its own
        // check, so VarMod(NULL, <any non-BSTR>) went on to convert NULL to I8
        // and failed DISP_E_TYPEMISMATCH (Wine's vartest). `stage` replays the
        // fallthrough: an arm runs its own check and every later one.
        let rvt = (*V_VT_ref(right_ptr)) & VT_TYPEMASK;
        let stage: u8 = match rvt {
            VT_BOOL | VT_I1 | VT_I2 | VT_I4 | VT_I8 => 1,
            VT_INT => 2,
            VT_UI1 | VT_UI2 | VT_UI4 | VT_UI8 | VT_UINT | VT_R4 | VT_R8 | VT_CY => 3,
            VT_EMPTY | VT_DATE | VT_DECIMAL => 4,
            VT_BSTR => 5,
            _ => 0,
        };
        if stage != 0 {
            if stage <= 1 && ((*V_VT_ref(left_ptr)) == VT_INT) && ((*V_VT_ref(right_ptr)) == VT_I8) {
                (*V_VT_ref(result)) = VT_EMPTY;
                rc = DISP_E_TYPEMISMATCH;
                break 'end;
            }
            if stage <= 2 && ((*V_VT_ref(right_ptr)) == VT_INT) && ((*V_VT_ref(left_ptr)) == VT_I8) {
                (*V_VT_ref(result)) = VT_EMPTY;
                rc = DISP_E_TYPEMISMATCH;
                break 'end;
            }
            if stage <= 3 && (*V_VT_ref(left_ptr)) == VT_EMPTY {
                // Wine 10.0: and a VALUE (the result held whatever the caller had).
                (*V_VT_ref(result)) = VT_I4;
                (*V_I4_ref(result)) = 0;
                rc = S_OK;
                break 'end;
            }
            if stage <= 4 && (*V_VT_ref(left_ptr)) == VT_ERROR {
                (*V_VT_ref(result)) = VT_EMPTY;
                rc = DISP_E_TYPEMISMATCH;
                break 'end;
            }
            if (*V_VT_ref(left_ptr)) == VT_NULL {
                (*V_VT_ref(result)) = VT_NULL;
                rc = S_OK;
                break 'end;
            }
        }

        match rvt {
            VT_BOOL | VT_I1 | VT_I2 | VT_I4 | VT_I8 | VT_INT | VT_UI1 | VT_UI2 | VT_UI4 | VT_UI8
            | VT_UINT | VT_R4 | VT_R8 | VT_CY | VT_EMPTY | VT_DATE | VT_DECIMAL | VT_BSTR => {}
            VT_VOID => {
                (*V_VT_ref(result)) = VT_EMPTY;
                rc = DISP_E_BADVARTYPE;
                break 'end;
            }
            VT_NULL => {
                if (*V_VT_ref(left_ptr)) == VT_VOID {
                    (*V_VT_ref(result)) = VT_EMPTY;
                    rc = DISP_E_BADVARTYPE;
                } else if ((*V_VT_ref(left_ptr)) == VT_NULL) || ((*V_VT_ref(left_ptr)) == VT_EMPTY) || ((*V_VT_ref(left_ptr)) == VT_ERROR) || (lOk != 0) {
                    (*V_VT_ref(result)) = VT_NULL;
                    rc = S_OK;
                } else {
                    (*V_VT_ref(result)) = VT_NULL;
                    rc = DISP_E_BADVARTYPE;
                }
                break 'end;
            }
            VT_VARIANT | VT_UNKNOWN => {
                (*V_VT_ref(result)) = VT_EMPTY;
                rc = DISP_E_TYPEMISMATCH;
                break 'end;
            }
            VT_ERROR => {
                rc = DISP_E_TYPEMISMATCH;
                break 'end;
            }
            VT_RECORD => {
                if ((*V_VT_ref(left_ptr)) == 15) || (((*V_VT_ref(left_ptr)) >= 24) && ((*V_VT_ref(left_ptr)) <= 35)) || (lOk == 0) {
                    (*V_VT_ref(result)) = VT_EMPTY;
                    rc = DISP_E_BADVARTYPE;
                } else {
                    (*V_VT_ref(result)) = VT_EMPTY;
                    rc = DISP_E_TYPEMISMATCH;
                }
                break 'end;
            }
            _ => {
                (*V_VT_ref(result)) = VT_EMPTY;
                rc = DISP_E_BADVARTYPE;
                break 'end;
            }
        }

        if ((*V_VT_ref(left_ptr)) == VT_I8) || ((*V_VT_ref(right_ptr)) == VT_I8) {
            resT = VT_I8 as i32;
        } else if ((*V_VT_ref(left_ptr)) == VT_UI1) && ((*V_VT_ref(right_ptr)) == VT_BOOL) {
            resT = VT_I2 as i32;
        } else if ((*V_VT_ref(left_ptr)) == VT_UI1) && ((*V_VT_ref(right_ptr)) == VT_UI1) {
            resT = VT_UI1 as i32;
        } else if ((*V_VT_ref(left_ptr)) == VT_UI1) && ((*V_VT_ref(right_ptr)) == VT_I2) {
            resT = VT_I2 as i32;
        } else if ((*V_VT_ref(left_ptr)) == VT_I2) && ((*V_VT_ref(right_ptr)) == VT_BOOL) {
            resT = VT_I2 as i32;
        } else if ((*V_VT_ref(left_ptr)) == VT_I2) && ((*V_VT_ref(right_ptr)) == VT_UI1) {
            resT = VT_I2 as i32;
        } else if ((*V_VT_ref(left_ptr)) == VT_I2) && ((*V_VT_ref(right_ptr)) == VT_I2) {
            resT = VT_I2 as i32;
        } else if ((*V_VT_ref(left_ptr)) == VT_BOOL) && ((*V_VT_ref(right_ptr)) == VT_BOOL) {
            resT = VT_I2 as i32;
        } else if ((*V_VT_ref(left_ptr)) == VT_BOOL) && ((*V_VT_ref(right_ptr)) == VT_UI1) {
            resT = VT_I2 as i32;
        } else if ((*V_VT_ref(left_ptr)) == VT_BOOL) && ((*V_VT_ref(right_ptr)) == VT_I2) {
            resT = VT_I2 as i32;
        } else {
            resT = VT_I4 as i32;
        }

        rc = VariantChangeType(&mut lv, left_ptr, 0, VT_I8);
        if FAILED(rc) {
            FIXME!(b"Could not convert left type %d to %d? rc == 0x%X\n\0".as_ptr() as *const i8, (*V_VT_ref(left_ptr)), VT_I8, rc);
            break 'end;
        }

        rc = VariantChangeType(&mut rv, right_ptr, 0, VT_I8);
        if FAILED(rc) {
            FIXME!(b"Could not convert right type %d to %d? rc == 0x%X\n\0".as_ptr() as *const i8, (*V_VT_ref(right_ptr)), VT_I8, rc);
            break 'end;
        }

        if (*V_I8_ref(&mut rv)) == 0 {
            (*V_VT_ref(result)) = VT_EMPTY;
            rc = DISP_E_DIVBYZERO;
            break 'end;
        }

        (*V_VT_ref(result)) = VT_I8;
        (*V_I8_ref(result)) = (*V_I8_ref(&mut lv)) % (*V_I8_ref(&mut rv));

        TRACE!(b"(*V_I8_ref(left)) == %s, (*V_I8_ref(right)) == %s, (*V_I8_ref(result)) == %s\n\0".as_ptr() as *const i8,
              wine_dbgstr_longlong((*V_I8_ref(&mut lv))), wine_dbgstr_longlong((*V_I8_ref(&mut rv))),
              wine_dbgstr_longlong((*V_I8_ref(result))));

        rc = VariantChangeType(result, result, 0, resT as u16);
        if FAILED(rc) {
            FIXME!(b"Could not convert 0x%x to %d?\n\0".as_ptr() as *const i8, (*V_VT_ref(result)), resT);
        }
    }

    VariantClear(&mut lv);
    VariantClear(&mut rv);
    VariantClear(&mut tempLeft);
    VariantClear(&mut tempRight);
    rc
}

#[no_mangle]
pub unsafe extern "system" fn VarPow(left: LPVARIANT, right: LPVARIANT, result: LPVARIANT) -> HRESULT {
    let mut hr: HRESULT;
    let mut dl: VARIANT = core::mem::zeroed();
    let mut dr: VARIANT = core::mem::zeroed();
    let resvt: VARTYPE;
    let leftvt: VARTYPE;
    let rightvt: VARTYPE;
    let rightExtraFlags: VARTYPE;
    let leftExtraFlags: VARTYPE;
    let ExtraFlags: VARTYPE;
    let mut tempLeft: VARIANT = core::mem::zeroed();
    let mut tempRight: VARIANT = core::mem::zeroed();
    let mut left = left;
    let mut right = right;

    TRACE!(b"(%s,%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(left), debugstr_variant(right), result);

    VariantInit(&mut dl);
    VariantInit(&mut dr);
    VariantInit(&mut tempLeft);
    VariantInit(&mut tempRight);

    if ((*V_VT_ref(left)) & VT_TYPEMASK) == VT_DISPATCH {
        hr = VARIANT_FetchDispatchValue(left, &mut tempLeft);
        if FAILED(hr) {
            VariantClear(&mut dl);
            VariantClear(&mut dr);
            VariantClear(&mut tempLeft);
            VariantClear(&mut tempRight);
            return hr;
        }
        left = &mut tempLeft;
    }
    if ((*V_VT_ref(right)) & VT_TYPEMASK) == VT_DISPATCH {
        hr = VARIANT_FetchDispatchValue(right, &mut tempRight);
        if FAILED(hr) {
            VariantClear(&mut dl);
            VariantClear(&mut dr);
            VariantClear(&mut tempLeft);
            VariantClear(&mut tempRight);
            return hr;
        }
        right = &mut tempRight;
    }

    leftvt = (*V_VT_ref(left)) & VT_TYPEMASK;
    rightvt = (*V_VT_ref(right)) & VT_TYPEMASK;
    leftExtraFlags = (*V_VT_ref(left)) & (!VT_TYPEMASK);
    rightExtraFlags = (*V_VT_ref(right)) & (!VT_TYPEMASK);

    if leftExtraFlags != rightExtraFlags {
        hr = DISP_E_BADVARTYPE;
        VariantClear(&mut dl);
        VariantClear(&mut dr);
        VariantClear(&mut tempLeft);
        VariantClear(&mut tempRight);
        return hr;
    }
    ExtraFlags = leftExtraFlags;

    if ExtraFlags != 0 {
        hr = DISP_E_BADVARTYPE;
        VariantClear(&mut dl);
        VariantClear(&mut dr);
        VariantClear(&mut tempLeft);
        VariantClear(&mut tempRight);
        return hr;
    } else if leftvt == VT_NULL || rightvt == VT_NULL {
        *(&mut (*result).n1.n2.vt) = VT_NULL;
        hr = S_OK;
        VariantClear(&mut dl);
        VariantClear(&mut dr);
        VariantClear(&mut tempLeft);
        VariantClear(&mut tempRight);
        return hr;
    } else if (leftvt == VT_EMPTY || leftvt == VT_I2 ||
        leftvt == VT_I4 || leftvt == VT_R4 ||
        leftvt == VT_R8 || leftvt == VT_CY ||
        leftvt == VT_DATE || leftvt == VT_BSTR ||
        leftvt == VT_BOOL || leftvt == VT_DECIMAL ||
        (leftvt >= VT_I1 && leftvt <= VT_UINT)) &&
        (rightvt == VT_EMPTY || rightvt == VT_I2 ||
        rightvt == VT_I4 || rightvt == VT_R4 ||
        rightvt == VT_R8 || rightvt == VT_CY ||
        rightvt == VT_DATE || rightvt == VT_BSTR ||
        rightvt == VT_BOOL || rightvt == VT_DECIMAL ||
        (rightvt >= VT_I1 && rightvt <= VT_UINT)) {
        resvt = VT_R8;
    } else {
        hr = DISP_E_BADVARTYPE;
        VariantClear(&mut dl);
        VariantClear(&mut dr);
        VariantClear(&mut tempLeft);
        VariantClear(&mut tempRight);
        return hr;
    }

    hr = VariantChangeType(&mut dl, left, 0, resvt);
    if FAILED(hr) {
        ERR!(b"Could not change passed left argument to VT_R8, handle it differently.\n\0".as_ptr() as *const i8);
        hr = E_FAIL;
        VariantClear(&mut dl);
        VariantClear(&mut dr);
        VariantClear(&mut tempLeft);
        VariantClear(&mut tempRight);
        return hr;
    }

    hr = VariantChangeType(&mut dr, right, 0, resvt);
    if FAILED(hr) {
        ERR!(b"Could not change passed right argument to VT_R8, handle it differently.\n\0".as_ptr() as *const i8);
        hr = E_FAIL;
        VariantClear(&mut dl);
        VariantClear(&mut dr);
        VariantClear(&mut tempLeft);
        VariantClear(&mut tempRight);
        return hr;
    }

    *(&mut (*result).n1.n2.vt) = VT_R8;
    *(&mut (*result).n1.n2.n3.dblVal) = pow(*V_R8_ref(&mut dl) , *V_R8_ref(&mut dr) );

    VariantClear(&mut dl);
    VariantClear(&mut dr);
    VariantClear(&mut tempLeft);
    VariantClear(&mut tempRight);

    hr
}

#[no_mangle]
pub unsafe extern "system" fn VarImp(left: LPVARIANT, right: LPVARIANT, result: LPVARIANT) -> HRESULT {
    let mut hres: HRESULT = S_OK;
    let mut resvt: VARTYPE = VT_EMPTY;
    let leftvt: VARTYPE;
    let rightvt: VARTYPE;
    let rightExtraFlags: VARTYPE;
    let leftExtraFlags: VARTYPE;
    let ExtraFlags: VARTYPE;
    let mut lv: VARIANT = core::mem::zeroed();
    let mut rv: VARIANT = core::mem::zeroed();
    let mut d: f64 = 0.0;
    let mut tempLeft: VARIANT = core::mem::zeroed();
    let mut tempRight: VARIANT = core::mem::zeroed();
    let mut left_ptr = left;
    let mut right_ptr = right;

    VariantInit(&mut lv);
    VariantInit(&mut rv);
    VariantInit(&mut tempLeft);
    VariantInit(&mut tempRight);

    TRACE!(b"(%s,%s,%p)\n\0".as_ptr() as *const i8, debugstr_variant(left), debugstr_variant(right), result);

    'VarImp_Exit: {
    if ((*V_VT_ref(left_ptr)) & VT_TYPEMASK) == VT_DISPATCH {
        hres = VARIANT_FetchDispatchValue(left_ptr, &mut tempLeft);
        if FAILED(hres) { break 'VarImp_Exit; }
        left_ptr = &mut tempLeft;
    }
    if ((*V_VT_ref(right_ptr)) & VT_TYPEMASK) == VT_DISPATCH {
        hres = VARIANT_FetchDispatchValue(right_ptr, &mut tempRight);
        if FAILED(hres) { break 'VarImp_Exit; }
        right_ptr = &mut tempRight;
    }

    leftvt = (*V_VT_ref(left_ptr)) & VT_TYPEMASK;
    rightvt = (*V_VT_ref(right_ptr)) & VT_TYPEMASK;
    leftExtraFlags = (*V_VT_ref(left_ptr)) & (!VT_TYPEMASK);
    rightExtraFlags = (*V_VT_ref(right_ptr)) & (!VT_TYPEMASK);

    if leftExtraFlags != rightExtraFlags {
        hres = DISP_E_BADVARTYPE;
        break 'VarImp_Exit;
    }
    ExtraFlags = leftExtraFlags;

    if (leftvt == VT_I8 && rightvt == VT_INT) || ExtraFlags != 0 {
        hres = DISP_E_BADVARTYPE;
        break 'VarImp_Exit;
    } else if (leftvt == VT_NULL && rightvt == VT_NULL) || (leftvt == VT_NULL && rightvt == VT_EMPTY) {
        (*V_VT_ref(result)) = VT_NULL;
        hres = S_OK;
        break 'VarImp_Exit;
    } else if leftvt == VT_I8 || rightvt == VT_I8 {
        resvt = VT_I8;
    } else if leftvt == VT_I4 || rightvt == VT_I4 || leftvt == VT_INT || rightvt == VT_INT ||
              leftvt == VT_UINT || rightvt == VT_UINT || leftvt == VT_UI4 || rightvt == VT_UI4 ||
              leftvt == VT_UI8 || rightvt == VT_UI8 || leftvt == VT_UI2 || rightvt == VT_UI2 ||
              leftvt == VT_DECIMAL || rightvt == VT_DECIMAL || leftvt == VT_DATE || rightvt == VT_DATE ||
              leftvt == VT_CY || rightvt == VT_CY || leftvt == VT_R8 || rightvt == VT_R8 ||
              leftvt == VT_R4 || rightvt == VT_R4 || leftvt == VT_I1 || rightvt == VT_I1 {
        resvt = VT_I4;
    } else if (leftvt == VT_UI1 && rightvt == VT_UI1) || (leftvt == VT_UI1 && rightvt == VT_NULL) ||
              (leftvt == VT_NULL && rightvt == VT_UI1) {
        resvt = VT_UI1;
    } else if leftvt == VT_EMPTY || rightvt == VT_EMPTY || leftvt == VT_I2 || rightvt == VT_I2 ||
              leftvt == VT_UI1 || rightvt == VT_UI1 {
        resvt = VT_I2;
    } else if leftvt == VT_BOOL || rightvt == VT_BOOL || leftvt == VT_BSTR || rightvt == VT_BSTR {
        resvt = VT_BOOL;
    }

    if leftvt == VT_NULL {
        let mut b: VARIANT_BOOL = 0;
        match rightvt {
            VT_I1 => { if (*V_I1_ref(right_ptr)) == 0 { resvt = VT_NULL; } }
            VT_UI1 => { if (*V_UI1_ref(right_ptr)) == 0 { resvt = VT_NULL; } }
            VT_I2 => { if (*V_I2_ref(right_ptr)) == 0 { resvt = VT_NULL; } }
            VT_UI2 => { if (*V_UI2_ref(right_ptr)) == 0 { resvt = VT_NULL; } }
            VT_I4 => { if (*V_I4_ref(right_ptr)) == 0 { resvt = VT_NULL; } }
            VT_UI4 => { if (*V_UI4_ref(right_ptr)) == 0 { resvt = VT_NULL; } }
            VT_I8 => { if (*V_I8_ref(right_ptr)) == 0 { resvt = VT_NULL; } }
            VT_UI8 => { if (*V_UI8_ref(right_ptr)) == 0 { resvt = VT_NULL; } }
            VT_INT => { if (*V_INT_ref(right_ptr)) == 0 { resvt = VT_NULL; } }
            VT_UINT => { if (*V_UINT_ref(right_ptr)) == 0 { resvt = VT_NULL; } }
            VT_BOOL => { if (*V_BOOL_ref(right_ptr)) == 0 { resvt = VT_NULL; } }
            VT_R4 => { if (*V_R4_ref(right_ptr)) == 0.0 { resvt = VT_NULL; } }
            VT_R8 => { if (*V_R8_ref(right_ptr)) == 0.0 { resvt = VT_NULL; } }
            VT_DATE => { if (*V_DATE_ref(right_ptr)) == 0.0 { resvt = VT_NULL; } }
            VT_CY => { if (*V_CY_ref(right_ptr)).int64 == 0 { resvt = VT_NULL; } }
            VT_DECIMAL => {
                if ((*DEC_HI32_ref(V_DECIMAL_ref(right_ptr))) == 0) && ((*DEC_LO64_ref(V_DECIMAL_ref(right_ptr))) == 0) {
                    resvt = VT_NULL;
                }
            }
            VT_BSTR => {
                hres = VarBoolFromStr(*V_BSTR_ref(right_ptr) , LOCALE_USER_DEFAULT, VAR_LOCALBOOL, &mut b);
                if FAILED(hres) { break 'VarImp_Exit; }
                else if b == 0 {
                    (*V_VT_ref(result)) = VT_NULL;
                } else {
                    (*V_VT_ref(result)) = VT_BOOL;
                    (*V_BOOL_ref(result)) = b;
                }
                break 'VarImp_Exit;
            }
            _ => {}
        }
        if resvt == VT_NULL {
            (*V_VT_ref(result)) = resvt;
            break 'VarImp_Exit;
        } else {
            hres = VariantChangeType(result, right_ptr, 0, resvt);
            break 'VarImp_Exit;
        }
    } else if rightvt == VT_NULL {
        let mut b: VARIANT_BOOL = 0;
        match leftvt {
            VT_I1 => { if (*V_I1_ref(left_ptr)) == -1 { resvt = VT_NULL; } }
            VT_UI1 => { if (*V_UI1_ref(left_ptr)) == 0xff { resvt = VT_NULL; } }
            VT_I2 => { if (*V_I2_ref(left_ptr)) == -1 { resvt = VT_NULL; } }
            VT_UI2 => { if (*V_UI2_ref(left_ptr)) == 0xffff { resvt = VT_NULL; } }
            VT_INT => { if (*V_INT_ref(left_ptr)) == -1 { resvt = VT_NULL; } }
            VT_UINT => { if (*V_UINT_ref(left_ptr)) == !0u32 { resvt = VT_NULL; } }
            VT_I4 => { if (*V_I4_ref(left_ptr)) == -1 { resvt = VT_NULL; } }
            VT_UI4 => { if (*V_UI4_ref(left_ptr)) == !0u32 { resvt = VT_NULL; } }
            VT_I8 => { if (*V_I8_ref(left_ptr)) == -1 { resvt = VT_NULL; } }
            VT_UI8 => { if (*V_UI8_ref(left_ptr)) == !0u64 { resvt = VT_NULL; } }
            VT_BOOL => { if (*V_BOOL_ref(left_ptr)) == VARIANT_TRUE as i16 { resvt = VT_NULL; } }
            VT_R4 => { if (*V_R4_ref(left_ptr)) == -1.0 { resvt = VT_NULL; } }
            VT_R8 => { if (*V_R8_ref(left_ptr)) == -1.0 { resvt = VT_NULL; } }
            VT_CY => { if (*V_CY_ref(left_ptr)).int64 == -1 { resvt = VT_NULL; } }
            VT_DECIMAL => {
                if (*DEC_HI32_ref(V_DECIMAL_ref(left_ptr))) == 0xffffffff {
                    resvt = VT_NULL;
                }
            }
            VT_BSTR => {
                hres = VarBoolFromStr(*V_BSTR_ref(left_ptr) , LOCALE_USER_DEFAULT, VAR_LOCALBOOL, &mut b);
                if FAILED(hres) { break 'VarImp_Exit; }
                else if b == VARIANT_TRUE as i16 {
                    resvt = VT_NULL;
                }
            }
            _ => {}
        }
        if resvt == VT_NULL {
            (*V_VT_ref(result)) = resvt;
            break 'VarImp_Exit;
        }
    }

    hres = VariantCopy(&mut lv, left_ptr);
    if FAILED(hres) { break 'VarImp_Exit; }

    if rightvt == VT_NULL {
        memset(&mut rv as *mut _ as LPVOID, 0, core::mem::size_of::<VARIANT>());
        (*V_VT_ref(&mut rv)) = resvt;
    } else {
        hres = VariantCopy(&mut rv, right_ptr);
        if FAILED(hres) { break 'VarImp_Exit; }
    }

    if (*V_VT_ref(&mut lv)) == VT_BSTR && FAILED(VarR8FromStr(*V_BSTR_ref(&mut lv) , LOCALE_USER_DEFAULT, 0, &mut d)) {
        hres = VariantChangeType(&mut lv, &mut lv, VARIANT_LOCALBOOL as USHORT, VT_BOOL);
    }
    if SUCCEEDED(hres) && (*V_VT_ref(&mut lv)) != resvt {
        hres = VariantChangeType(&mut lv, &mut lv, 0, resvt);
    }
    if FAILED(hres) { break 'VarImp_Exit; }

    if (*V_VT_ref(&mut rv)) == VT_BSTR && FAILED(VarR8FromStr(*V_BSTR_ref(&mut rv) , LOCALE_USER_DEFAULT, 0, &mut d)) {
        hres = VariantChangeType(&mut rv, &mut rv, VARIANT_LOCALBOOL as USHORT, VT_BOOL);
    }
    if SUCCEEDED(hres) && (*V_VT_ref(&mut rv)) != resvt {
        hres = VariantChangeType(&mut rv, &mut rv, 0, resvt);
    }
    if FAILED(hres) { break 'VarImp_Exit; }

    (*V_VT_ref(result)) = resvt;
    match resvt {
        VT_I8 => {
            (*V_I8_ref(result)) = (!(*V_I8_ref(&mut lv))) | (*V_I8_ref(&mut rv));
        }
        VT_I4 => {
            (*V_I4_ref(result)) = (!(*V_I4_ref(&mut lv))) | (*V_I4_ref(&mut rv));
        }
        VT_I2 => {
            (*V_I2_ref(result)) = (!(*V_I2_ref(&mut lv))) | (*V_I2_ref(&mut rv));
        }
        VT_UI1 => {
            (*V_UI1_ref(result)) = (!(*V_UI1_ref(&mut lv))) | (*V_UI1_ref(&mut rv));
        }
        VT_BOOL => {
            (*V_BOOL_ref(result)) = (!(*V_BOOL_ref(&mut lv))) | (*V_BOOL_ref(&mut rv));
        }
        _ => {
            FIXME!(b"Couldn't perform bitwise implication on variant types %d,%d\n\0".as_ptr() as *const i8, leftvt, rightvt);
        }
    }

    } // 'VarImp_Exit

    VariantClear(&mut lv);
    VariantClear(&mut rv);
    VariantClear(&mut tempLeft);
    VariantClear(&mut tempRight);

    hres
}
