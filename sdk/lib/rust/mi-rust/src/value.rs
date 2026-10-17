//! Owned MI values. An element (of an instance or an options bag) keeps a deep copy of what
//! it was given; `get` hands out an MI_Value whose pointers borrow that copy, which is the
//! MI contract (valid until the element is changed or its owner deleted).
//!
//! WMI's side: a property arrives as a VARIANT plus its CIMTYPE. The VARIANT's own vt is
//! what decides how to read it (WMI stores uint32 in VT_I4 and 64-bit integers in VT_BSTR),
//! the CIMTYPE what MI_Type it becomes.

use crate::types::*;
use crate::sys::*;
use crate::wstr::*;
use alloc::vec::Vec;
use core::ffi::c_void;

pub const MI_RESULT_OK: MI_Result = _MI_Result_MI_RESULT_OK;
pub const MI_RESULT_FAILED: MI_Result = _MI_Result_MI_RESULT_FAILED;
pub const MI_RESULT_ACCESS_DENIED: MI_Result = _MI_Result_MI_RESULT_ACCESS_DENIED;
pub const MI_RESULT_INVALID_NAMESPACE: MI_Result = _MI_Result_MI_RESULT_INVALID_NAMESPACE;
pub const MI_RESULT_INVALID_PARAMETER: MI_Result = _MI_Result_MI_RESULT_INVALID_PARAMETER;
pub const MI_RESULT_INVALID_CLASS: MI_Result = _MI_Result_MI_RESULT_INVALID_CLASS;
pub const MI_RESULT_NOT_FOUND: MI_Result = _MI_Result_MI_RESULT_NOT_FOUND;
pub const MI_RESULT_NOT_SUPPORTED: MI_Result = _MI_Result_MI_RESULT_NOT_SUPPORTED;
pub const MI_RESULT_ALREADY_EXISTS: MI_Result = _MI_Result_MI_RESULT_ALREADY_EXISTS;
pub const MI_RESULT_NO_SUCH_PROPERTY: MI_Result = _MI_Result_MI_RESULT_NO_SUCH_PROPERTY;
pub const MI_RESULT_TYPE_MISMATCH: MI_Result = _MI_Result_MI_RESULT_TYPE_MISMATCH;
pub const MI_RESULT_QUERY_LANGUAGE_NOT_SUPPORTED: MI_Result =
    _MI_Result_MI_RESULT_QUERY_LANGUAGE_NOT_SUPPORTED;
pub const MI_RESULT_INVALID_QUERY: MI_Result = _MI_Result_MI_RESULT_INVALID_QUERY;
pub const MI_RESULT_METHOD_NOT_FOUND: MI_Result = _MI_Result_MI_RESULT_METHOD_NOT_FOUND;

pub const MI_BOOLEAN: MI_Type = _MI_Type_MI_BOOLEAN;
pub const MI_UINT8: MI_Type = _MI_Type_MI_UINT8;
pub const MI_SINT8: MI_Type = _MI_Type_MI_SINT8;
pub const MI_UINT16: MI_Type = _MI_Type_MI_UINT16;
pub const MI_SINT16: MI_Type = _MI_Type_MI_SINT16;
pub const MI_UINT32: MI_Type = _MI_Type_MI_UINT32;
pub const MI_SINT32: MI_Type = _MI_Type_MI_SINT32;
pub const MI_UINT64: MI_Type = _MI_Type_MI_UINT64;
pub const MI_SINT64: MI_Type = _MI_Type_MI_SINT64;
pub const MI_REAL32: MI_Type = _MI_Type_MI_REAL32;
pub const MI_REAL64: MI_Type = _MI_Type_MI_REAL64;
pub const MI_CHAR16: MI_Type = _MI_Type_MI_CHAR16;
pub const MI_DATETIME: MI_Type = _MI_Type_MI_DATETIME;
pub const MI_STRING: MI_Type = _MI_Type_MI_STRING;
pub const MI_REFERENCE: MI_Type = _MI_Type_MI_REFERENCE;
pub const MI_INSTANCE: MI_Type = _MI_Type_MI_INSTANCE;
pub const MI_STRINGA: MI_Type = _MI_Type_MI_STRINGA;
pub const MI_REFERENCEA: MI_Type = _MI_Type_MI_REFERENCEA;
pub const MI_INSTANCEA: MI_Type = _MI_Type_MI_INSTANCEA;
/// MI_ARRAY is the array bit: MI_UINT8A == MI_UINT8 | MI_ARRAY.
pub const MI_ARRAY_BIT: MI_Type = _MI_Type_MI_ARRAY;

pub const FLAG_NULL: u32 = MI_FLAG_NULL;
pub const FLAG_KEY: u32 = MI_FLAG_KEY;
pub const FLAG_BORROW: u32 = MI_FLAG_BORROW;

/// Bytes per element of a scalar MI type (and of its array).
pub fn scalar_size(t: MI_Type) -> usize {
    match t & !MI_ARRAY_BIT {
        MI_BOOLEAN | MI_UINT8 | MI_SINT8 => 1,
        MI_UINT16 | MI_SINT16 | MI_CHAR16 => 2,
        MI_UINT32 | MI_SINT32 | MI_REAL32 => 4,
        MI_UINT64 | MI_SINT64 | MI_REAL64 => 8,
        MI_DATETIME => core::mem::size_of::<MI_Datetime>(),
        _ => core::mem::size_of::<*mut c_void>(),
    }
}

pub fn valid_type(t: MI_Type) -> bool {
    t <= MI_INSTANCEA
}

/// The storage an MI_Value's pointers point into. Never read back: it is held only so the
/// pointers in `Owned::val` stay valid.
#[allow(dead_code)]
pub enum Store {
    None,
    Str(WString),
    Inst(*mut MI_Instance),
    Bytes(Vec<u8>),
    Strs(Vec<WString>, Vec<*mut u16>),
    Insts(Vec<*mut MI_Instance>),
}

pub struct Owned {
    pub ty: MI_Type,
    pub null: bool,
    pub val: MI_Value,
    pub store: Store,
}

unsafe fn delete_instance(i: *mut MI_Instance) {
    if !i.is_null() && !(*i).ft.is_null() {
        if let Some(d) = (*(*i).ft).Delete {
            d(i);
        }
    }
}

pub unsafe fn clone_instance(i: *const MI_Instance) -> Result<*mut MI_Instance, MI_Result> {
    if i.is_null() {
        return Ok(core::ptr::null_mut());
    }
    if (*i).ft.is_null() {
        return Err(MI_RESULT_INVALID_PARAMETER);
    }
    let mut out = core::ptr::null_mut();
    match (*(*i).ft).Clone {
        Some(c) => {
            let r = c(i, &mut out);
            if r == MI_RESULT_OK {
                Ok(out)
            } else {
                Err(r)
            }
        }
        None => Err(MI_RESULT_NOT_SUPPORTED),
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        unsafe {
            match &self.store {
                Store::Inst(i) => delete_instance(*i),
                Store::Insts(v) => {
                    for &i in v {
                        delete_instance(i)
                    }
                }
                _ => {}
            }
        }
    }
}

impl Owned {
    pub fn null(ty: MI_Type) -> Owned {
        Owned { ty, null: true, val: unsafe { core::mem::zeroed() }, store: Store::None }
    }

    fn scalar(ty: MI_Type, val: MI_Value) -> Owned {
        Owned { ty, null: false, val, store: Store::None }
    }

    pub fn string(ty: MI_Type, s: WString) -> Owned {
        let mut o = Owned { ty, null: false, val: unsafe { core::mem::zeroed() }, store: Store::Str(s) };
        if let Store::Str(s) = &mut o.store {
            o.val.string = s.as_mut_ptr();
        }
        o
    }

    pub fn instance(ty: MI_Type, i: *mut MI_Instance) -> Owned {
        let mut o = Owned::null(ty);
        if !i.is_null() {
            o.null = false;
            o.val.instance = i;
            o.store = Store::Inst(i);
        }
        o
    }

    pub fn bytes(ty: MI_Type, mut b: Vec<u8>, count: u32) -> Owned {
        let mut o = Owned::null(ty);
        o.null = false;
        o.val.array = MI_Array { data: b.as_mut_ptr() as *mut c_void, size: count };
        o.store = Store::Bytes(b);
        o
    }

    pub fn strings(ty: MI_Type, v: Vec<WString>) -> Owned {
        let mut o = Owned::null(ty);
        o.null = false;
        let mut v = v;
        let mut ptrs: Vec<*mut u16> = v.iter_mut().map(|s| s.as_mut_ptr()).collect();
        o.val.stringa = MI_StringA { data: ptrs.as_mut_ptr(), size: ptrs.len() as u32 };
        o.store = Store::Strs(v, ptrs);
        o
    }

    pub fn instances(ty: MI_Type, mut v: Vec<*mut MI_Instance>) -> Owned {
        let mut o = Owned::null(ty);
        o.null = false;
        o.val.instancea = MI_InstanceA { data: v.as_mut_ptr(), size: v.len() as u32 };
        o.store = Store::Insts(v);
        o
    }

    /// A deep copy of a caller's value. NULL `value` (or MI_FLAG_NULL) is a null element.
    pub unsafe fn copy_in(ty: MI_Type, value: *const MI_Value, flags: u32) -> Result<Owned, MI_Result> {
        if !valid_type(ty) {
            return Err(MI_RESULT_INVALID_PARAMETER);
        }
        if value.is_null() || flags & FLAG_NULL != 0 {
            return Ok(Owned::null(ty));
        }
        let v = &*value;
        Ok(match ty {
            MI_STRING => match wdup(v.string) {
                Some(s) => Owned::string(ty, s),
                None => Owned::null(ty),
            },
            MI_INSTANCE | MI_REFERENCE => Owned::instance(ty, clone_instance(v.instance)?),
            MI_STRINGA => {
                let a = v.stringa;
                let mut out = Vec::with_capacity(a.size as usize);
                for k in 0..a.size as usize {
                    out.push(wdup(*a.data.add(k)).unwrap_or_else(|| alloc::vec![0]));
                }
                Owned::strings(ty, out)
            }
            MI_INSTANCEA | MI_REFERENCEA => {
                let a = v.instancea;
                let mut out = Vec::with_capacity(a.size as usize);
                for k in 0..a.size as usize {
                    match clone_instance(*a.data.add(k)) {
                        Ok(i) => out.push(i),
                        Err(e) => {
                            for i in out {
                                delete_instance(i);
                            }
                            return Err(e);
                        }
                    }
                }
                Owned::instances(ty, out)
            }
            t if t & MI_ARRAY_BIT != 0 => {
                let a = v.array;
                let n = a.size as usize * scalar_size(t);
                let mut b = Vec::with_capacity(n);
                if n != 0 {
                    b.extend_from_slice(core::slice::from_raw_parts(a.data as *const u8, n));
                }
                Owned::bytes(ty, b, a.size)
            }
            _ => Owned::scalar(ty, *v),
        })
    }

    pub unsafe fn deep_clone(&self) -> Result<Owned, MI_Result> {
        if self.null {
            return Ok(Owned::null(self.ty));
        }
        Owned::copy_in(self.ty, &self.val, 0)
    }

    /// The element's flags bit for nullness.
    pub fn null_flag(&self) -> u32 {
        if self.null {
            FLAG_NULL
        } else {
            0
        }
    }
}

// ---- WMI -> MI ---------------------------------------------------------------------------

pub const CIM_SINT8: i32 = tag_CIMTYPE_ENUMERATION_CIM_SINT8 as i32;
pub const CIM_UINT8: i32 = tag_CIMTYPE_ENUMERATION_CIM_UINT8 as i32;
pub const CIM_SINT16: i32 = tag_CIMTYPE_ENUMERATION_CIM_SINT16 as i32;
pub const CIM_UINT16: i32 = tag_CIMTYPE_ENUMERATION_CIM_UINT16 as i32;
pub const CIM_SINT32: i32 = tag_CIMTYPE_ENUMERATION_CIM_SINT32 as i32;
pub const CIM_UINT32: i32 = tag_CIMTYPE_ENUMERATION_CIM_UINT32 as i32;
pub const CIM_SINT64: i32 = tag_CIMTYPE_ENUMERATION_CIM_SINT64 as i32;
pub const CIM_UINT64: i32 = tag_CIMTYPE_ENUMERATION_CIM_UINT64 as i32;
pub const CIM_REAL32: i32 = tag_CIMTYPE_ENUMERATION_CIM_REAL32 as i32;
pub const CIM_REAL64: i32 = tag_CIMTYPE_ENUMERATION_CIM_REAL64 as i32;
pub const CIM_BOOLEAN: i32 = tag_CIMTYPE_ENUMERATION_CIM_BOOLEAN as i32;
pub const CIM_DATETIME: i32 = tag_CIMTYPE_ENUMERATION_CIM_DATETIME as i32;
pub const CIM_REFERENCE: i32 = tag_CIMTYPE_ENUMERATION_CIM_REFERENCE as i32;
pub const CIM_CHAR16: i32 = tag_CIMTYPE_ENUMERATION_CIM_CHAR16 as i32;
pub const CIM_OBJECT: i32 = tag_CIMTYPE_ENUMERATION_CIM_OBJECT as i32;
pub const CIM_FLAG_ARRAY: i32 = tag_CIMTYPE_ENUMERATION_CIM_FLAG_ARRAY as i32;

/// The MI type a CIM type becomes.
pub fn mi_type_of(cim: i32) -> MI_Type {
    let base = match cim & !CIM_FLAG_ARRAY {
        CIM_SINT8 => MI_SINT8,
        CIM_UINT8 => MI_UINT8,
        CIM_SINT16 => MI_SINT16,
        CIM_UINT16 => MI_UINT16,
        CIM_SINT32 => MI_SINT32,
        CIM_UINT32 => MI_UINT32,
        CIM_SINT64 => MI_SINT64,
        CIM_UINT64 => MI_UINT64,
        CIM_REAL32 => MI_REAL32,
        CIM_REAL64 => MI_REAL64,
        CIM_BOOLEAN => MI_BOOLEAN,
        CIM_DATETIME => MI_DATETIME,
        CIM_REFERENCE => MI_REFERENCE,
        CIM_CHAR16 => MI_CHAR16,
        CIM_OBJECT => MI_INSTANCE,
        _ => MI_STRING,
    };
    if cim & CIM_FLAG_ARRAY != 0 {
        base | MI_ARRAY_BIT
    } else {
        base
    }
}

/// One scalar read out of VARIANT storage of type `vt` at `p`, as the widest integer, a
/// float, or a string, whichever it holds.
enum Raw {
    Int(i128),
    Float(f64),
    Str(WString),
    Unk(*mut IUnknown),
    None,
}

unsafe fn read_raw(vt: u16, p: *const u8) -> Raw {
    match vt {
        VT_I1 => Raw::Int(*(p as *const i8) as i128),
        VT_UI1 => Raw::Int(*p as i128),
        VT_I2 => Raw::Int(*(p as *const i16) as i128),
        VT_UI2 => Raw::Int(*(p as *const u16) as i128),
        VT_I4 | VT_INT => Raw::Int(*(p as *const i32) as i128),
        VT_UI4 | VT_UINT => Raw::Int(*(p as *const u32) as i128),
        VT_I8 => Raw::Int(*(p as *const i64) as i128),
        VT_UI8 => Raw::Int(*(p as *const u64) as i128),
        VT_BOOL => Raw::Int((*(p as *const i16) != 0) as i128),
        VT_R4 => Raw::Float(*(p as *const f32) as f64),
        VT_R8 => Raw::Float(*(p as *const f64)),
        VT_BSTR => {
            let b = *(p as *const BSTR);
            if b.is_null() {
                Raw::Str(alloc::vec![0])
            } else {
                Raw::Str(wfrom(core::slice::from_raw_parts(b, SysStringLen(b) as usize)))
            }
        }
        VT_UNKNOWN | VT_DISPATCH => Raw::Unk(*(p as *const *mut IUnknown)),
        _ => Raw::None,
    }
}

fn vt_size(vt: u16) -> usize {
    match vt {
        VT_I1 | VT_UI1 => 1,
        VT_I2 | VT_UI2 | VT_BOOL => 2,
        VT_I4 | VT_UI4 | VT_INT | VT_UINT | VT_R4 => 4,
        VT_I8 | VT_UI8 | VT_R8 => 8,
        _ => core::mem::size_of::<*mut c_void>(),
    }
}

fn as_int(r: &Raw) -> Option<i128> {
    match r {
        Raw::Int(n) => Some(*n),
        Raw::Float(f) => Some(*f as i128),
        Raw::Str(s) => parse_i128(s),
        _ => None,
    }
}

fn as_float(r: &Raw) -> Option<f64> {
    match r {
        Raw::Int(n) => Some(*n as f64),
        Raw::Float(f) => Some(*f),
        _ => None,
    }
}

fn digits(s: &[u16], at: usize, n: usize) -> Option<u32> {
    let mut v = 0u32;
    for k in 0..n {
        let c = *s.get(at + k)?;
        if c == b'*' as u16 {
            return Some(0);
        }
        if !(b'0' as u16..=b'9' as u16).contains(&c) {
            return None;
        }
        v = v * 10 + (c - b'0' as u16) as u32;
    }
    Some(v)
}

/// CIM DMTF datetime: `yyyymmddHHMMSS.mmmmmmsUUU` (s = '+'/'-', UUU minutes from UTC) for a
/// timestamp, `ddddddddHHMMSS.mmmmmm:000` for an interval.
pub fn parse_datetime(s: &[u16]) -> MI_Datetime {
    let s = body(s);
    let mut d: MI_Datetime = unsafe { core::mem::zeroed() };
    if s.len() >= 25 && s[21] == b':' as u16 {
        d.isTimestamp = 0;
        d.u.interval = MI_Interval {
            days: digits(s, 0, 8).unwrap_or(0),
            hours: digits(s, 8, 2).unwrap_or(0),
            minutes: digits(s, 10, 2).unwrap_or(0),
            seconds: digits(s, 12, 2).unwrap_or(0),
            microseconds: digits(s, 15, 6).unwrap_or(0),
            __padding1: 0,
            __padding2: 0,
            __padding3: 0,
        };
    } else {
        d.isTimestamp = 1;
        let off = digits(s, 22, 3).unwrap_or(0) as i32;
        d.u.timestamp = MI_Timestamp {
            year: digits(s, 0, 4).unwrap_or(0),
            month: digits(s, 4, 2).unwrap_or(0),
            day: digits(s, 6, 2).unwrap_or(0),
            hour: digits(s, 8, 2).unwrap_or(0),
            minute: digits(s, 10, 2).unwrap_or(0),
            second: digits(s, 12, 2).unwrap_or(0),
            microseconds: digits(s, 15, 6).unwrap_or(0),
            utc: if s.get(21) == Some(&(b'-' as u16)) { -off } else { off },
        };
    }
    d
}

unsafe fn scalar_from_raw(
    ty: MI_Type,
    r: Raw,
    ns: &[u16],
    server: &[u16],
) -> Result<Owned, MI_Result> {
    let mut v: MI_Value = core::mem::zeroed();
    let bad = || MI_RESULT_TYPE_MISMATCH;
    match ty {
        MI_BOOLEAN => v.boolean = (as_int(&r).ok_or_else(bad)? != 0) as u8,
        MI_UINT8 => v.uint8 = as_int(&r).ok_or_else(bad)? as u8,
        MI_SINT8 => v.sint8 = as_int(&r).ok_or_else(bad)? as i8,
        MI_UINT16 => v.uint16 = as_int(&r).ok_or_else(bad)? as u16,
        MI_SINT16 => v.sint16 = as_int(&r).ok_or_else(bad)? as i16,
        MI_CHAR16 => v.char16 = as_int(&r).ok_or_else(bad)? as u16,
        MI_UINT32 => v.uint32 = as_int(&r).ok_or_else(bad)? as u32,
        MI_SINT32 => v.sint32 = as_int(&r).ok_or_else(bad)? as i32,
        MI_UINT64 => v.uint64 = as_int(&r).ok_or_else(bad)? as u64,
        MI_SINT64 => v.sint64 = as_int(&r).ok_or_else(bad)? as i64,
        MI_REAL32 => v.real32 = as_float(&r).ok_or_else(bad)? as f32,
        MI_REAL64 => v.real64 = as_float(&r).ok_or_else(bad)?,
        MI_DATETIME => match r {
            Raw::Str(s) => v.datetime = parse_datetime(&s),
            _ => return Err(bad()),
        },
        MI_STRING => {
            return Ok(match r {
                Raw::Str(s) => Owned::string(ty, s),
                Raw::Int(n) => {
                    let mut s = WString::new();
                    if n < 0 {
                        push_str(&mut s, "-");
                    }
                    push_dec(&mut s, n.unsigned_abs() as u64);
                    Owned::string(ty, s)
                }
                _ => return Err(bad()),
            })
        }
        MI_REFERENCE => {
            return match r {
                Raw::Str(s) => Ok(Owned::instance(ty, crate::instance::from_path(&s, ns, server))),
                _ => Err(bad()),
            }
        }
        MI_INSTANCE => {
            return match r {
                Raw::Unk(u) if !u.is_null() => {
                    let mut obj: *mut c_void = core::ptr::null_mut();
                    let hr = com!(u, QueryInterface(&IID_IWBEM_CLASS_OBJECT, &mut obj));
                    if hr < 0 || obj.is_null() {
                        return Err(bad());
                    }
                    let obj = ComPtr(obj as *mut IWbemClassObject);
                    let i = crate::instance::from_wbem(obj.0, ns, server)?;
                    Ok(Owned::instance(ty, i))
                }
                _ => Ok(Owned::null(ty)),
            }
        }
        _ => return Err(bad()),
    }
    Ok(Owned::scalar(ty, v))
}

/// A WMI property value (VARIANT + CIMTYPE) as an owned MI value.
pub unsafe fn from_variant(var: &VARIANT, cim: i32, ns: &[u16], server: &[u16]) -> Result<Owned, MI_Result> {
    let ty = mi_type_of(cim);
    let vt = var.n1.n2.vt;
    if vt == VT_EMPTY || vt == VT_NULL {
        return Ok(Owned::null(ty));
    }
    let payload = core::ptr::addr_of!(var.n1.n2.n3) as *const u8;
    if vt & VT_ARRAY == 0 {
        if ty & MI_ARRAY_BIT != 0 {
            return Err(MI_RESULT_TYPE_MISMATCH);
        }
        return scalar_from_raw(ty, read_raw(vt, payload), ns, server);
    }
    if ty & MI_ARRAY_BIT == 0 {
        return Err(MI_RESULT_TYPE_MISMATCH);
    }
    let sa = var.n1.n2.n3.parray;
    if sa.is_null() {
        return Ok(Owned::null(ty));
    }
    let evt = vt & VT_TYPEMASK;
    let (mut lb, mut ub) = (0, -1);
    SafeArrayGetLBound(sa, 1, &mut lb);
    SafeArrayGetUBound(sa, 1, &mut ub);
    let n = if ub >= lb { (ub - lb + 1) as usize } else { 0 };
    let mut data: *mut c_void = core::ptr::null_mut();
    if n != 0 && SafeArrayAccessData(sa, &mut data) < 0 {
        return Err(MI_RESULT_FAILED);
    }
    let esize = vt_size(evt);
    let elem = ty & !MI_ARRAY_BIT;
    let mut strs = Vec::new();
    let mut insts = Vec::new();
    let mut bytes = Vec::new();
    let mut convert = || -> Result<(), MI_Result> {
        for k in 0..n {
            let raw = read_raw(evt, (data as *const u8).add(k * esize));
            let mut o = scalar_from_raw(elem, raw, ns, server)?;
            match (elem, core::mem::replace(&mut o.store, Store::None)) {
                (MI_STRING, Store::Str(s)) => strs.push(s),
                (MI_INSTANCE | MI_REFERENCE, Store::Inst(i)) => insts.push(i),
                (MI_STRING | MI_INSTANCE | MI_REFERENCE, _) => {}
                _ => {
                    let p = core::ptr::addr_of!(o.val) as *const u8;
                    bytes.extend_from_slice(core::slice::from_raw_parts(p, scalar_size(elem)));
                }
            }
        }
        Ok(())
    };
    let result = convert();
    if n != 0 {
        SafeArrayUnaccessData(sa);
    }
    if let Err(e) = result {
        for i in insts {
            if let Some(d) = (*(*i).ft).Delete {
                d(i);
            }
        }
        return Err(e);
    }
    Ok(match elem {
        MI_STRING => Owned::strings(ty, strs),
        MI_INSTANCE | MI_REFERENCE => Owned::instances(ty, insts),
        _ => Owned::bytes(ty, bytes, n as u32),
    })
}
