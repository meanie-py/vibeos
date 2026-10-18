//! MI_Session: a destination (only this machine: the backend is the local wbemprox) and the
//! operations against it. Each operation function turns its arguments into a `Job`; what
//! the backend cannot do yet completes with MI_RESULT_NOT_SUPPORTED through the operation,
//! as the MI contract requires (the session functions themselves return nothing).

use crate::instance::as_inst;
use crate::operation::{start, Job};
use crate::types::*;
use crate::value::*;
use crate::wstr::*;
use alloc::boxed::Box;
use core::ffi::c_void;

pub struct SessState {
    pub application: MI_Application,
}

const SESSION_MAGIC: u64 = 0x4d49_5365_7373_696f; // "MISessio"

unsafe fn state<'a>(s: *const MI_Session) -> Option<&'a SessState> {
    if s.is_null() || (*s).reserved1 != SESSION_MAGIC || (*s).reserved2 == 0 {
        return None;
    }
    Some(&*((*s).reserved2 as *const SessState))
}

pub unsafe fn init(session: *mut MI_Session, app: MI_Application) {
    (*session).reserved1 = SESSION_MAGIC;
    (*session).reserved2 = Box::into_raw(Box::new(SessState { application: app })) as isize;
    (*session).ft = &SESSION_FT;
}

/// The namespace an operation names, `root/cimv2` when it names none.
unsafe fn ns_or_default(ns: *const MI_Char) -> WString {
    match wdup(ns) {
        Some(v) if body(&v).len() != 0 => v,
        _ => wa("root/cimv2"),
    }
}

unsafe fn begin(
    session: *mut MI_Session,
    flags: MI_Uint32,
    callbacks: *mut MI_OperationCallbacks,
    operation: *mut MI_Operation,
    job: Job,
) {
    let job = if state(session).is_none() { Job::Done(MI_RESULT_INVALID_PARAMETER) } else { job };
    start(session, flags, callbacks, operation, job);
}

unsafe extern "system" fn s_close(
    session: *mut MI_Session,
    context: *mut c_void,
    callback: Option<unsafe extern "system" fn(*mut c_void)>,
) -> MI_Result {
    if state(session).is_none() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    drop(Box::from_raw((*session).reserved2 as *mut SessState));
    (*session).reserved1 = 0;
    (*session).reserved2 = 0;
    if let Some(cb) = callback {
        cb(context);
    }
    MI_RESULT_OK
}

unsafe extern "system" fn s_get_application(session: *mut MI_Session, app: *mut MI_Application) -> MI_Result {
    let Some(st) = state(session) else { return MI_RESULT_INVALID_PARAMETER };
    if app.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    *app = st.application;
    MI_RESULT_OK
}

/// `Class.Key="value",Other=3` from an instance's key elements (all elements when none is
/// marked a key, which is how a client-built instance usually arrives).
unsafe fn object_path(inst: *const MI_Instance) -> Option<WString> {
    let i = as_inst(inst)?;
    let mut p = wfrom(body(&i.class_name));
    let any_key = i.elems.iter().any(|e| e.flags & FLAG_KEY != 0);
    let mut first = true;
    for e in &i.elems {
        if (any_key && e.flags & FLAG_KEY == 0) || e.value.null {
            continue;
        }
        push_str(&mut p, if first { "." } else { "," });
        first = false;
        push_w(&mut p, &e.name);
        push_str(&mut p, "=");
        let v = &e.value.val;
        match e.value.ty {
            MI_STRING => {
                push_str(&mut p, "\"");
                for &c in core::slice::from_raw_parts(v.string, wlen(v.string)) {
                    if c == b'"' as u16 || c == b'\\' as u16 {
                        push_str(&mut p, "\\");
                    }
                    push_w(&mut p, &[c]);
                }
                push_str(&mut p, "\"");
            }
            MI_BOOLEAN => push_str(&mut p, if v.boolean != 0 { "TRUE" } else { "FALSE" }),
            MI_UINT8 => push_dec(&mut p, v.uint8 as u64),
            MI_UINT16 | MI_CHAR16 => push_dec(&mut p, v.uint16 as u64),
            MI_UINT32 => push_dec(&mut p, v.uint32 as u64),
            MI_UINT64 => push_dec(&mut p, v.uint64),
            MI_SINT8 | MI_SINT16 | MI_SINT32 | MI_SINT64 => {
                let n: i64 = match e.value.ty {
                    MI_SINT8 => v.sint8 as i64,
                    MI_SINT16 => v.sint16 as i64,
                    MI_SINT32 => v.sint32 as i64,
                    _ => v.sint64,
                };
                if n < 0 {
                    push_str(&mut p, "-");
                }
                push_dec(&mut p, n.unsigned_abs());
            }
            _ => return None,
        }
    }
    Some(p)
}

unsafe extern "system" fn s_get_instance(
    session: *mut MI_Session,
    flags: MI_Uint32,
    _options: *mut MI_OperationOptions,
    ns: *const MI_Char,
    inbound: *const MI_Instance,
    callbacks: *mut MI_OperationCallbacks,
    operation: *mut MI_Operation,
) {
    let job = match object_path(inbound) {
        Some(path) => Job::Get { ns: ns_or_default(ns), path },
        None => Job::Done(MI_RESULT_INVALID_PARAMETER),
    };
    begin(session, flags, callbacks, operation, job)
}

unsafe extern "system" fn s_unsupported_instance_op(
    session: *mut MI_Session,
    flags: MI_Uint32,
    _options: *mut MI_OperationOptions,
    _ns: *const MI_Char,
    _inbound: *const MI_Instance,
    callbacks: *mut MI_OperationCallbacks,
    operation: *mut MI_Operation,
) {
    begin(session, flags, callbacks, operation, Job::Done(MI_RESULT_NOT_SUPPORTED))
}

unsafe extern "system" fn s_invoke(
    session: *mut MI_Session,
    flags: MI_Uint32,
    _options: *mut MI_OperationOptions,
    _ns: *const MI_Char,
    _class: *const MI_Char,
    _method: *const MI_Char,
    _inbound: *const MI_Instance,
    _params: *const MI_Instance,
    callbacks: *mut MI_OperationCallbacks,
    operation: *mut MI_Operation,
) {
    begin(session, flags, callbacks, operation, Job::Done(MI_RESULT_NOT_SUPPORTED))
}

unsafe extern "system" fn s_enumerate_instances(
    session: *mut MI_Session,
    flags: MI_Uint32,
    _options: *mut MI_OperationOptions,
    ns: *const MI_Char,
    class: *const MI_Char,
    _keys_only: MI_Boolean,
    callbacks: *mut MI_OperationCallbacks,
    operation: *mut MI_Operation,
) {
    let job = match wdup(class) {
        Some(c) if body(&c).len() != 0 => {
            let mut wql = wa("SELECT * FROM ");
            push_w(&mut wql, &c);
            Job::Query { ns: ns_or_default(ns), wql }
        }
        _ => Job::Done(MI_RESULT_INVALID_PARAMETER),
    };
    begin(session, flags, callbacks, operation, job)
}

unsafe extern "system" fn s_query_instances(
    session: *mut MI_Session,
    flags: MI_Uint32,
    _options: *mut MI_OperationOptions,
    ns: *const MI_Char,
    dialect: *const MI_Char,
    query: *const MI_Char,
    callbacks: *mut MI_OperationCallbacks,
    operation: *mut MI_Operation,
) {
    let job = match wdup(query) {
        None => Job::Done(MI_RESULT_INVALID_PARAMETER),
        // WQL is the dialect WMI speaks; Windows' mi.dll also refuses others for a DCOM session.
        Some(_) if !dialect.is_null() && !ieq_ptr(&wa("WQL"), dialect) => Job::Done(MI_RESULT_QUERY_LANGUAGE_NOT_SUPPORTED),
        Some(wql) => Job::Query { ns: ns_or_default(ns), wql },
    };
    begin(session, flags, callbacks, operation, job)
}

unsafe extern "system" fn s_associators(
    session: *mut MI_Session,
    flags: MI_Uint32,
    _options: *mut MI_OperationOptions,
    _ns: *const MI_Char,
    _keys: *const MI_Instance,
    _assoc: *const MI_Char,
    _result: *const MI_Char,
    _role: *const MI_Char,
    _result_role: *const MI_Char,
    _keys_only: MI_Boolean,
    callbacks: *mut MI_OperationCallbacks,
    operation: *mut MI_Operation,
) {
    begin(session, flags, callbacks, operation, Job::Done(MI_RESULT_NOT_SUPPORTED))
}

unsafe extern "system" fn s_references(
    session: *mut MI_Session,
    flags: MI_Uint32,
    _options: *mut MI_OperationOptions,
    _ns: *const MI_Char,
    _keys: *const MI_Instance,
    _result: *const MI_Char,
    _role: *const MI_Char,
    _keys_only: MI_Boolean,
    callbacks: *mut MI_OperationCallbacks,
    operation: *mut MI_Operation,
) {
    begin(session, flags, callbacks, operation, Job::Done(MI_RESULT_NOT_SUPPORTED))
}

unsafe extern "system" fn s_subscribe(
    session: *mut MI_Session,
    flags: MI_Uint32,
    _options: *mut MI_OperationOptions,
    _ns: *const MI_Char,
    _dialect: *const MI_Char,
    _query: *const MI_Char,
    _delivery: *const MI_SubscriptionDeliveryOptions,
    callbacks: *mut MI_OperationCallbacks,
    operation: *mut MI_Operation,
) {
    begin(session, flags, callbacks, operation, Job::Done(MI_RESULT_NOT_SUPPORTED))
}

unsafe extern "system" fn s_get_class(
    session: *mut MI_Session,
    flags: MI_Uint32,
    _options: *mut MI_OperationOptions,
    _ns: *const MI_Char,
    _class: *const MI_Char,
    callbacks: *mut MI_OperationCallbacks,
    operation: *mut MI_Operation,
) {
    begin(session, flags, callbacks, operation, Job::Done(MI_RESULT_NOT_SUPPORTED))
}

unsafe extern "system" fn s_enumerate_classes(
    session: *mut MI_Session,
    flags: MI_Uint32,
    _options: *mut MI_OperationOptions,
    _ns: *const MI_Char,
    _class: *const MI_Char,
    _names_only: MI_Boolean,
    callbacks: *mut MI_OperationCallbacks,
    operation: *mut MI_Operation,
) {
    begin(session, flags, callbacks, operation, Job::Done(MI_RESULT_NOT_SUPPORTED))
}

unsafe extern "system" fn s_test_connection(
    session: *mut MI_Session,
    flags: MI_Uint32,
    callbacks: *mut MI_OperationCallbacks,
    operation: *mut MI_Operation,
) {
    begin(session, flags, callbacks, operation, Job::Done(MI_RESULT_OK))
}

pub static SESSION_FT: MI_SessionFT = MI_SessionFT {
    Close: Some(s_close),
    GetApplication: Some(s_get_application),
    GetInstance: Some(s_get_instance),
    ModifyInstance: Some(s_unsupported_instance_op),
    CreateInstance: Some(s_unsupported_instance_op),
    DeleteInstance: Some(s_unsupported_instance_op),
    Invoke: Some(s_invoke),
    EnumerateInstances: Some(s_enumerate_instances),
    QueryInstances: Some(s_query_instances),
    AssociatorInstances: Some(s_associators),
    ReferenceInstances: Some(s_references),
    Subscribe: Some(s_subscribe),
    GetClass: Some(s_get_class),
    EnumerateClasses: Some(s_enumerate_classes),
    TestConnection: Some(s_test_connection),
};

/// A destination this backend can serve: none (the local machine), `.`, `localhost`, the
/// loopback address or this computer's name.
pub unsafe fn is_local(dest: *const MI_Char) -> bool {
    let Some(d) = wdup(dest) else { return true };
    let d = body(&d);
    if d.is_empty() || ieq(d, &wa(".")) || ieq(d, &wa("localhost")) || ieq(d, &wa("127.0.0.1")) {
        return true;
    }
    let mut name = [0u16; 64];
    let mut n: u32 = name.len() as u32;
    crate::sys::GetComputerNameW(name.as_mut_ptr(), &mut n) != 0 && ieq(d, &name[..n as usize])
}
