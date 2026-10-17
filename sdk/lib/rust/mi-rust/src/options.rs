//! Operation and destination options: named, typed values the client sets and reads back.
//! The WMI backend honours none of them yet (a local session has no transport to tune), but
//! the bag has to work: the CIM .NET layer builds one for every call and reads its own
//! settings back out of it.

use crate::instance::Elem;
use crate::types::*;
use crate::value::*;
use crate::wstr::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ptr::null;

pub struct Bag {
    pub elems: Vec<Elem>,
}

const BAG_MAGIC: u64 = 0x4d49_4f70_7442_6167; // "MIOptBag"

impl Bag {
    unsafe fn set(&mut self, name: *const u16, ty: MI_Type, value: *const MI_Value, flags: u32) -> MI_Result {
        let Some(n) = wdup(name) else { return MI_RESULT_INVALID_PARAMETER };
        let v = match Owned::copy_in(ty, value, flags) {
            Ok(v) => v,
            Err(e) => return e,
        };
        match self.elems.iter().position(|e| ieq(&e.name, &n)) {
            Some(k) => self.elems[k] = Elem { name: n, flags, value: v },
            None => self.elems.push(Elem { name: n, flags, value: v }),
        }
        MI_RESULT_OK
    }

    unsafe fn get(&self, name: *const u16) -> Option<(usize, &Elem)> {
        self.elems.iter().enumerate().find(|(_, e)| ieq_ptr(&e.name, name))
    }

    unsafe fn clone(&self) -> Result<Bag, MI_Result> {
        let mut out = Vec::with_capacity(self.elems.len());
        for e in &self.elems {
            out.push(Elem { name: e.name.clone(), flags: e.flags, value: e.value.deep_clone()? });
        }
        Ok(Bag { elems: out })
    }
}

/// The three option handles share a layout: reserved1, reserved2 (our Bag), ft.
#[repr(C)]
pub struct Handle<FT> {
    pub reserved1: MI_Uint64,
    pub reserved2: isize,
    pub ft: *const FT,
}

pub unsafe fn bag<'a, FT>(h: *const Handle<FT>) -> Option<&'a mut Bag> {
    if h.is_null() || (*h).reserved1 != BAG_MAGIC || (*h).reserved2 == 0 {
        return None;
    }
    Some(&mut *((*h).reserved2 as *mut Bag))
}

pub unsafe fn init<FT>(h: *mut Handle<FT>, ft: *const FT, b: Bag) {
    (*h).reserved1 = BAG_MAGIC;
    (*h).reserved2 = Box::into_raw(Box::new(b)) as isize;
    (*h).ft = ft;
}

unsafe fn delete<FT>(h: *mut Handle<FT>) {
    if bag(h).is_some() {
        drop(Box::from_raw((*h).reserved2 as *mut Bag));
        (*h).reserved1 = 0;
        (*h).reserved2 = 0;
    }
}

macro_rules! bag_or {
    ($h:expr) => {
        match bag($h as *const _ as *const Handle<()>) {
            Some(b) => b,
            None => return MI_RESULT_INVALID_PARAMETER,
        }
    };
}

unsafe fn set_string<FT>(h: *mut Handle<FT>, name: *const u16, value: *const u16, flags: u32) -> MI_Result {
    let b = bag_or!(h);
    let mut v: MI_Value = core::mem::zeroed();
    v.string = value as *mut u16;
    b.set(name, MI_STRING, if value.is_null() { null() } else { &v }, flags)
}

unsafe fn set_number<FT>(h: *mut Handle<FT>, name: *const u16, value: u32, flags: u32) -> MI_Result {
    let b = bag_or!(h);
    let mut v: MI_Value = core::mem::zeroed();
    v.uint32 = value;
    b.set(name, MI_UINT32, &v, flags)
}

unsafe fn set_interval<FT>(h: *mut Handle<FT>, name: *const u16, value: *const MI_Interval, flags: u32) -> MI_Result {
    let b = bag_or!(h);
    if value.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    let mut v: MI_Value = core::mem::zeroed();
    v.datetime.isTimestamp = 0;
    v.datetime.u.interval = *value;
    b.set(name, MI_DATETIME, &v, flags)
}

unsafe fn get_typed<FT>(
    h: *const Handle<FT>,
    name: *const u16,
    want: MI_Type,
    index: *mut u32,
    flags: *mut u32,
) -> Result<MI_Value, MI_Result> {
    let b = match bag(h as *const Handle<()>) {
        Some(b) => b,
        None => return Err(MI_RESULT_INVALID_PARAMETER),
    };
    let Some((k, e)) = b.get(name) else { return Err(MI_RESULT_NO_SUCH_PROPERTY) };
    if e.value.ty != want {
        return Err(MI_RESULT_TYPE_MISMATCH);
    }
    if !index.is_null() {
        *index = k as u32;
    }
    if !flags.is_null() {
        *flags = e.flags;
    }
    Ok(e.value.val)
}

unsafe fn get_string<FT>(h: *const Handle<FT>, name: *const u16, value: *mut *const u16, index: *mut u32, flags: *mut u32) -> MI_Result {
    match get_typed(h, name, MI_STRING, index, flags) {
        Ok(v) => {
            if !value.is_null() {
                *value = v.string;
            }
            MI_RESULT_OK
        }
        Err(e) => e,
    }
}

unsafe fn get_number<FT>(h: *const Handle<FT>, name: *const u16, value: *mut u32, index: *mut u32, flags: *mut u32) -> MI_Result {
    match get_typed(h, name, MI_UINT32, index, flags) {
        Ok(v) => {
            if !value.is_null() {
                *value = v.uint32;
            }
            MI_RESULT_OK
        }
        Err(e) => e,
    }
}

unsafe fn get_interval<FT>(h: *const Handle<FT>, name: *const u16, value: *mut MI_Interval, index: *mut u32, flags: *mut u32) -> MI_Result {
    match get_typed(h, name, MI_DATETIME, index, flags) {
        Ok(v) if v.datetime.isTimestamp == 0 => {
            if !value.is_null() {
                *value = v.datetime.u.interval;
            }
            MI_RESULT_OK
        }
        Ok(_) => MI_RESULT_TYPE_MISMATCH,
        Err(e) => e,
    }
}

unsafe fn count<FT>(h: *const Handle<FT>, out: *mut u32) -> MI_Result {
    let b = bag_or!(h);
    if out.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    *out = b.elems.len() as u32;
    MI_RESULT_OK
}

unsafe fn get_at<FT>(h: *const Handle<FT>, index: u32, name: *mut *const u16, value: *mut MI_Value, ty: *mut MI_Type, flags: *mut u32) -> MI_Result {
    let b = bag_or!(h);
    let Some(e) = b.elems.get(index as usize) else { return MI_RESULT_NOT_FOUND };
    if !name.is_null() {
        *name = e.name.as_ptr();
    }
    if !value.is_null() {
        *value = e.value.val;
    }
    if !ty.is_null() {
        *ty = e.value.ty;
    }
    if !flags.is_null() {
        *flags = e.flags | e.value.null_flag();
    }
    MI_RESULT_OK
}

unsafe fn get_option<FT>(h: *const Handle<FT>, name: *const u16, value: *mut MI_Value, ty: *mut MI_Type, index: *mut u32, flags: *mut u32) -> MI_Result {
    let b = bag_or!(h);
    let Some((k, _)) = b.get(name) else { return MI_RESULT_NO_SUCH_PROPERTY };
    if !index.is_null() {
        *index = k as u32;
    }
    get_at(h, k as u32, core::ptr::null_mut(), value, ty, flags)
}

unsafe fn clone_into<FT>(src: *const Handle<FT>, dst: *mut Handle<FT>) -> MI_Result {
    let b = bag_or!(src);
    if dst.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    match b.clone() {
        Ok(n) => {
            init(dst, (*src).ft, n);
            MI_RESULT_OK
        }
        Err(e) => e,
    }
}

// ---- MI_OperationOptions -------------------------------------------------------------------

type OO = MI_OperationOptions;
unsafe fn oo(p: *const OO) -> *mut Handle<MI_OperationOptionsFT> {
    p as *mut Handle<MI_OperationOptionsFT>
}
unsafe extern "system" fn oo_delete(o: *mut OO) {
    delete(oo(o))
}
unsafe extern "system" fn oo_set_string(o: *mut OO, n: *const MI_Char, v: *const MI_Char, f: MI_Uint32) -> MI_Result {
    set_string(oo(o), n, v, f)
}
unsafe extern "system" fn oo_set_number(o: *mut OO, n: *const MI_Char, v: MI_Uint32, f: MI_Uint32) -> MI_Result {
    set_number(oo(o), n, v, f)
}
unsafe extern "system" fn oo_set_custom(o: *mut OO, n: *const MI_Char, t: MI_Type, v: *const MI_Value, _must: MI_Boolean, f: MI_Uint32) -> MI_Result {
    match bag(oo(o)) {
        Some(b) => b.set(n, t, v, f),
        None => MI_RESULT_INVALID_PARAMETER,
    }
}
unsafe extern "system" fn oo_get_string(o: *const OO, n: *const MI_Char, v: *mut *const MI_Char, i: *mut MI_Uint32, f: *mut MI_Uint32) -> MI_Result {
    get_string(oo(o), n, v, i, f)
}
unsafe extern "system" fn oo_get_number(o: *const OO, n: *const MI_Char, v: *mut MI_Uint32, i: *mut MI_Uint32, f: *mut MI_Uint32) -> MI_Result {
    get_number(oo(o), n, v, i, f)
}
unsafe extern "system" fn oo_count(o: *const OO, c: *mut MI_Uint32) -> MI_Result {
    count(oo(o), c)
}
unsafe extern "system" fn oo_get_at(o: *const OO, i: MI_Uint32, n: *mut *const MI_Char, v: *mut MI_Value, t: *mut MI_Type, f: *mut MI_Uint32) -> MI_Result {
    get_at(oo(o), i, n, v, t, f)
}
unsafe extern "system" fn oo_get(o: *const OO, n: *const MI_Char, v: *mut MI_Value, t: *mut MI_Type, i: *mut MI_Uint32, f: *mut MI_Uint32) -> MI_Result {
    get_option(oo(o), n, v, t, i, f)
}
unsafe extern "system" fn oo_channels(o: *const OO, _n: *const MI_Char, _c: *mut MI_Uint32, _len: MI_Uint32, count: *mut MI_Uint32, _f: *mut MI_Uint32) -> MI_Result {
    if bag(oo(o)).is_none() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    if !count.is_null() {
        *count = 0;
    }
    MI_RESULT_OK
}
unsafe extern "system" fn oo_clone(o: *const OO, n: *mut OO) -> MI_Result {
    clone_into(oo(o), oo(n))
}
unsafe extern "system" fn oo_set_interval(o: *mut OO, n: *const MI_Char, v: *const MI_Interval, f: MI_Uint32) -> MI_Result {
    set_interval(oo(o), n, v, f)
}
unsafe extern "system" fn oo_get_interval(o: *const OO, n: *const MI_Char, v: *mut MI_Interval, i: *mut MI_Uint32, f: *mut MI_Uint32) -> MI_Result {
    get_interval(oo(o), n, v, i, f)
}

pub static OPERATION_OPTIONS_FT: MI_OperationOptionsFT = MI_OperationOptionsFT {
    Delete: Some(oo_delete),
    SetString: Some(oo_set_string),
    SetNumber: Some(oo_set_number),
    SetCustomOption: Some(oo_set_custom),
    GetString: Some(oo_get_string),
    GetNumber: Some(oo_get_number),
    GetOptionCount: Some(oo_count),
    GetOptionAt: Some(oo_get_at),
    GetOption: Some(oo_get),
    GetEnabledChannels: Some(oo_channels),
    Clone: Some(oo_clone),
    SetInterval: Some(oo_set_interval),
    GetInterval: Some(oo_get_interval),
};

// ---- MI_DestinationOptions -----------------------------------------------------------------

type DO = MI_DestinationOptions;
unsafe fn dop(p: *const DO) -> *mut Handle<MI_DestinationOptionsFT> {
    p as *mut Handle<MI_DestinationOptionsFT>
}
unsafe extern "system" fn do_delete(o: *mut DO) {
    delete(dop(o))
}
unsafe extern "system" fn do_set_string(o: *mut DO, n: *const MI_Char, v: *const MI_Char, f: MI_Uint32) -> MI_Result {
    set_string(dop(o), n, v, f)
}
unsafe extern "system" fn do_set_number(o: *mut DO, n: *const MI_Char, v: MI_Uint32, f: MI_Uint32) -> MI_Result {
    set_number(dop(o), n, v, f)
}
unsafe extern "system" fn do_add_credentials(o: *mut DO, _n: *const MI_Char, _c: *const MI_UserCredentials, _f: MI_Uint32) -> MI_Result {
    // Credentials only matter to a remote destination, and sessions here are local.
    if bag(dop(o)).is_none() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    MI_RESULT_NOT_SUPPORTED
}
unsafe extern "system" fn do_get_string(o: *const DO, n: *const MI_Char, v: *mut *const MI_Char, i: *mut MI_Uint32, f: *mut MI_Uint32) -> MI_Result {
    get_string(dop(o), n, v, i, f)
}
unsafe extern "system" fn do_get_number(o: *const DO, n: *const MI_Char, v: *mut MI_Uint32, i: *mut MI_Uint32, f: *mut MI_Uint32) -> MI_Result {
    get_number(dop(o), n, v, i, f)
}
unsafe extern "system" fn do_count(o: *const DO, c: *mut MI_Uint32) -> MI_Result {
    count(dop(o), c)
}
unsafe extern "system" fn do_get_at(o: *const DO, i: MI_Uint32, n: *mut *const MI_Char, v: *mut MI_Value, t: *mut MI_Type, f: *mut MI_Uint32) -> MI_Result {
    get_at(dop(o), i, n, v, t, f)
}
unsafe extern "system" fn do_get(o: *const DO, n: *const MI_Char, v: *mut MI_Value, t: *mut MI_Type, i: *mut MI_Uint32, f: *mut MI_Uint32) -> MI_Result {
    get_option(dop(o), n, v, t, i, f)
}
unsafe extern "system" fn do_cred_count(o: *const DO, c: *mut MI_Uint32) -> MI_Result {
    if bag(dop(o)).is_none() || c.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    *c = 0;
    MI_RESULT_OK
}
unsafe extern "system" fn do_cred_at(_o: *const DO, _i: MI_Uint32, _n: *mut *const MI_Char, _c: *mut MI_UserCredentials, _f: *mut MI_Uint32) -> MI_Result {
    MI_RESULT_NOT_FOUND
}
unsafe extern "system" fn do_cred_pw_at(_o: *const DO, _i: MI_Uint32, _n: *mut *const MI_Char, _p: *mut MI_Char, _len: MI_Uint32, _plen: *mut MI_Uint32, _f: *mut MI_Uint32) -> MI_Result {
    MI_RESULT_NOT_FOUND
}
unsafe extern "system" fn do_clone(o: *const DO, n: *mut DO) -> MI_Result {
    clone_into(dop(o), dop(n))
}
unsafe extern "system" fn do_set_interval(o: *mut DO, n: *const MI_Char, v: *const MI_Interval, f: MI_Uint32) -> MI_Result {
    set_interval(dop(o), n, v, f)
}
unsafe extern "system" fn do_get_interval(o: *const DO, n: *const MI_Char, v: *mut MI_Interval, i: *mut MI_Uint32, f: *mut MI_Uint32) -> MI_Result {
    get_interval(dop(o), n, v, i, f)
}

pub static DESTINATION_OPTIONS_FT: MI_DestinationOptionsFT = MI_DestinationOptionsFT {
    Delete: Some(do_delete),
    SetString: Some(do_set_string),
    SetNumber: Some(do_set_number),
    AddCredentials: Some(do_add_credentials),
    GetString: Some(do_get_string),
    GetNumber: Some(do_get_number),
    GetOptionCount: Some(do_count),
    GetOptionAt: Some(do_get_at),
    GetOption: Some(do_get),
    GetCredentialsCount: Some(do_cred_count),
    GetCredentialsAt: Some(do_cred_at),
    GetCredentialsPasswordAt: Some(do_cred_pw_at),
    Clone: Some(do_clone),
    SetInterval: Some(do_set_interval),
    GetInterval: Some(do_get_interval),
};

pub fn new_bag() -> Bag {
    Bag { elems: Vec::new() }
}
