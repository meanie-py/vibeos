//! MI_Operation: one request to WMI, run on its own thread.
//!
//! Every operation gets a worker thread that initialises COM as MTA, connects to the
//! namespace and runs the request. The caller's thread never touches COM, so MI works the
//! same whatever apartment (or none) the client is in, and no interface pointer crosses a
//! thread. Results reach the client one of two ways:
//!   - synchronous (no callbacks): queued here, pulled with MI_Operation_GetInstance;
//!   - asynchronous: the worker calls `callbacks.instanceResult` itself, and with
//!     MI_OPERATIONFLAGS_MANUAL_ACK_RESULTS waits for the client's acknowledgement before
//!     delivering the next one.
//! Either way the last result carries moreResults = FALSE and the final MI_Result.

use crate::instance;
use crate::sys::*;
use crate::types::*;
use crate::value::*;
use crate::wstr::*;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::ptr::{null, null_mut};
use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

pub enum Job {
    /// A WQL query (also what EnumerateInstances becomes).
    Query { ns: WString, wql: WString },
    /// One object by path (GetInstance).
    Get { ns: WString, path: WString },
    /// Completes at once with this result (unsupported requests, TestConnection).
    Done(MI_Result),
}

/// Shared by the client's thread and the worker. Everything either side changes lives in
/// `inner` and is touched only under `cs`; the state is reached through `&OpState` (never
/// `&mut`), so the compiler cannot assume a field is unchanged across a wait.
pub struct OpState {
    refs: AtomicI32,
    cs: UnsafeCell<CRITICAL_SECTION>,
    changed: HANDLE,
    inner: UnsafeCell<Inner>,
    cancel: AtomicBool,
    closed: AtomicBool,
    callbacks: Option<MI_OperationCallbacks>,
    flags: u32,
    session: MI_Session,
    handle: *mut MI_Operation,
    job: UnsafeCell<Option<Job>>,
    server: WString,
}

struct Inner {
    queue: VecDeque<*mut MI_Instance>,
    done: bool,
    result: MI_Result,
    error: Option<WString>,
    current: *mut MI_Instance,
    acked: bool,
}

const OP_MAGIC: u64 = 0x4d49_4f70_6572_6174; // "MIOperat"

unsafe fn state<'a>(op: *const MI_Operation) -> Option<&'a OpState> {
    if op.is_null() || (*op).reserved1 != OP_MAGIC || (*op).reserved2 == 0 {
        return None;
    }
    Some(&*((*op).reserved2 as *const OpState))
}

unsafe fn release(s: *const OpState) {
    if (*s).refs.fetch_sub(1, Ordering::AcqRel) == 1 {
        let st = Box::from_raw(s as *mut OpState);
        let inner = &*st.inner.get();
        for &i in &inner.queue {
            free_instance(i);
        }
        free_instance(inner.current);
        CloseHandle(st.changed);
        DeleteCriticalSection(st.cs.get());
    }
}

unsafe fn free_instance(i: *mut MI_Instance) {
    if !i.is_null() {
        if let Some(d) = (*(*i).ft).Delete {
            d(i);
        }
    }
}

struct Lock(*mut CRITICAL_SECTION);
impl Lock {
    unsafe fn new(st: &OpState) -> Lock {
        EnterCriticalSection(st.cs.get());
        Lock(st.cs.get())
    }
}

/// The shared fields. Only call with the lock held.
#[allow(clippy::mut_from_ref)]
unsafe fn inner(st: &OpState) -> &mut Inner {
    &mut *st.inner.get()
}
impl Drop for Lock {
    fn drop(&mut self) {
        unsafe { LeaveCriticalSection(self.0) }
    }
}

/// Wait until `ready` holds. `changed` is manual-reset and set by every state change; it is
/// reset under the lock only after `ready` was seen false, so no wakeup can be lost.
unsafe fn wait_until(st: &OpState, ready: impl Fn(&OpState, &Inner) -> bool) {
    loop {
        {
            let _l = Lock::new(st);
            if ready(st, inner(st)) {
                return;
            }
            ResetEvent(st.changed);
        }
        WaitForSingleObject(st.changed, INFINITE);
    }
}

pub fn hr_to_result(hr: HRESULT) -> MI_Result {
    match hr as u32 {
        0x8004_1002 => MI_RESULT_NOT_FOUND,          // WBEM_E_NOT_FOUND
        0x8004_1003 => MI_RESULT_ACCESS_DENIED,      // WBEM_E_ACCESS_DENIED
        0x8004_100E => MI_RESULT_INVALID_NAMESPACE,  // WBEM_E_INVALID_NAMESPACE
        0x8004_1010 => MI_RESULT_INVALID_CLASS,      // WBEM_E_INVALID_CLASS
        0x8004_1017 => MI_RESULT_INVALID_QUERY,      // WBEM_E_INVALID_QUERY
        0x8004_1018 => MI_RESULT_QUERY_LANGUAGE_NOT_SUPPORTED, // WBEM_E_INVALID_QUERY_TYPE
        0x8004_100C => MI_RESULT_NOT_SUPPORTED,      // WBEM_E_NOT_SUPPORTED
        0x8004_1008 | 0x8007_0057 => MI_RESULT_INVALID_PARAMETER, // WBEM_E_INVALID_PARAMETER, E_INVALIDARG
        0x8004_1055 => MI_RESULT_METHOD_NOT_FOUND,   // WBEM_E_METHOD_NOT_IMPLEMENTED
        0x8007_0005 => MI_RESULT_ACCESS_DENIED,      // E_ACCESSDENIED
        _ => MI_RESULT_FAILED,
    }
}

/// `root/cimv2` (MI) -> `root\cimv2` (WMI).
fn wmi_ns(ns: &[u16]) -> WString {
    let mut v = wfrom(body(ns));
    for c in v.iter_mut() {
        if *c == b'/' as u16 {
            *c = b'\\' as u16;
        }
    }
    v
}

/// Hand one result to the client. Returns false once the client has gone away.
unsafe fn deliver(st: &OpState, inst: *mut MI_Instance) -> bool {
    if st.closed.load(Ordering::Acquire) || st.cancel.load(Ordering::Acquire) {
        free_instance(inst);
        return false;
    }
    match st.callbacks {
        None => {
            let _l = Lock::new(st);
            inner(st).queue.push_back(inst);
            SetEvent(st.changed);
        }
        Some(cb) => {
            if let Some(f) = cb.instanceResult {
                let manual = st.flags & MI_OPERATIONFLAGS_MANUAL_ACK_RESULTS != 0;
                {
                    let _l = Lock::new(st);
                    inner(st).acked = false;
                }
                f(st.handle, cb.callbackContext, inst, 1, MI_RESULT_OK, null(), null(), if manual { Some(op_ack) } else { None });
                if manual {
                    wait_until(st, |s, i| i.acked || s.closed.load(Ordering::Acquire) || s.cancel.load(Ordering::Acquire));
                }
            }
            free_instance(inst);
        }
    }
    true
}

unsafe extern "system" fn op_ack(op: *mut MI_Operation) -> MI_Result {
    let Some(st) = state(op) else { return MI_RESULT_INVALID_PARAMETER };
    let _l = Lock::new(st);
    inner(st).acked = true;
    SetEvent(st.changed);
    MI_RESULT_OK
}

unsafe fn run(st: &OpState, job: Job) -> MI_Result {
    let (ns, request) = match job {
        Job::Done(r) => return r,
        Job::Query { ns, wql } => (ns, Ok(wql)),
        Job::Get { ns, path } => (ns, Err(path)),
    };
    let mut loc = ComPtr::<IWbemLocator>::null();
    let hr = CoCreateInstance(&CLSID_WBEM_LOCATOR, null_mut(), CLSCTX_INPROC_SERVER, &IID_IWBEM_LOCATOR, &mut loc.0 as *mut _ as *mut *mut c_void);
    if hr < 0 || loc.is_null() {
        return MI_RESULT_FAILED;
    }
    let ns_bstr = Bstr::new(&wmi_ns(&ns));
    let mut svc = ComPtr::<IWbemServices>::null();
    let hr = com!(loc.0, ConnectServer(ns_bstr.0, null_mut(), null_mut(), null_mut(), 0, null_mut(), null_mut(), &mut svc.0));
    if hr < 0 || svc.is_null() {
        return hr_to_result(hr);
    }
    impersonate(svc.0);
    match request {
        Ok(wql) => {
            let lang = Bstr::new(&wa("WQL"));
            let q = Bstr::new(&wql);
            let mut e = ComPtr::<IEnumWbemClassObject>::null();
            let flags = (tag_WBEM_GENERIC_FLAG_TYPE_WBEM_FLAG_FORWARD_ONLY | tag_WBEM_GENERIC_FLAG_TYPE_WBEM_FLAG_RETURN_IMMEDIATELY) as i32;
            let hr = com!(svc.0, ExecQuery(lang.0, q.0, flags, null_mut(), &mut e.0));
            if hr < 0 || e.is_null() {
                return hr_to_result(hr);
            }
            impersonate(e.0);
            loop {
                if st.cancel.load(Ordering::Acquire) {
                    return MI_RESULT_FAILED;
                }
                let mut obj: *mut IWbemClassObject = null_mut();
                let mut got: ULONG = 0;
                let hr = com!(e.0, Next(tag_WBEM_TIMEOUT_TYPE_WBEM_INFINITE as i32, 1, &mut obj, &mut got));
                if hr < 0 {
                    return hr_to_result(hr);
                }
                if got == 0 || obj.is_null() {
                    return MI_RESULT_OK;
                }
                let obj = ComPtr(obj);
                match instance::from_wbem(obj.0, &ns, &st.server) {
                    Ok(i) => {
                        if !deliver(st, i) {
                            return MI_RESULT_FAILED;
                        }
                    }
                    Err(r) => return r,
                }
            }
        }
        Err(path) => {
            let p = Bstr::new(&path);
            let mut obj = ComPtr::<IWbemClassObject>::null();
            // `GetObjectA`: wingdi.h's GetObject macro renamed the slot in the island; its position is what counts.
            let hr = com!(svc.0, GetObjectA(p.0, 0, null_mut(), &mut obj.0, null_mut()));
            if hr < 0 || obj.is_null() {
                return hr_to_result(hr);
            }
            match instance::from_wbem(obj.0, &ns, &st.server) {
                Ok(i) => {
                    deliver(st, i);
                    MI_RESULT_OK
                }
                Err(r) => r,
            }
        }
    }
}

unsafe extern "system" fn worker(p: *mut c_void) -> DWORD {
    let st = &*(p as *const OpState);
    let job = (*st.job.get()).take();
    let init = CoInitializeEx(null_mut(), COINIT_MULTITHREADED);
    let mut result = match job {
        Some(j) => run(st, j),
        None => MI_RESULT_FAILED,
    };
    if init >= 0 {
        CoUninitialize();
    }
    if st.cancel.load(Ordering::Acquire) && result == MI_RESULT_OK {
        result = MI_RESULT_FAILED;
    }
    {
        let _l = Lock::new(st);
        let i = inner(st);
        i.result = result;
        i.done = true;
        SetEvent(st.changed);
    }
    if let Some(cb) = st.callbacks {
        if let Some(f) = cb.instanceResult {
            if !st.closed.load(Ordering::Acquire) {
                f(st.handle, cb.callbackContext, null(), 0, result, null(), null(), None);
            }
        }
    }
    release(st);
    0
}

/// Start an operation and fill the client's MI_Operation. Never fails to fill it: a request
/// that cannot even start completes with its error through the normal result path.
pub unsafe fn start(
    session: *const MI_Session,
    flags: u32,
    callbacks: *const MI_OperationCallbacks,
    operation: *mut MI_Operation,
    job: Job,
) {
    if operation.is_null() {
        return;
    }
    let mut server = [0u16; 64];
    let mut n: DWORD = server.len() as DWORD;
    let server = if GetComputerNameW(server.as_mut_ptr(), &mut n) != 0 { wfrom(&server[..n as usize]) } else { wa("localhost") };
    let st = Box::new(OpState {
        refs: AtomicI32::new(2), // the client's handle and the worker
        cs: UnsafeCell::new(core::mem::zeroed()),
        changed: CreateEventW(null_mut(), 1, 0, null()),
        inner: UnsafeCell::new(Inner {
            queue: VecDeque::new(),
            done: false,
            result: MI_RESULT_OK,
            error: None,
            current: null_mut(),
            acked: false,
        }),
        cancel: AtomicBool::new(false),
        closed: AtomicBool::new(false),
        callbacks: if callbacks.is_null() || (*callbacks).instanceResult.is_none() { None } else { Some(*callbacks) },
        flags,
        session: if session.is_null() { core::mem::zeroed() } else { *session },
        handle: operation,
        job: UnsafeCell::new(Some(job)),
        server,
    });
    let s = Box::into_raw(st);
    InitializeCriticalSection((*s).cs.get());
    (*operation).reserved1 = OP_MAGIC;
    (*operation).reserved2 = s as isize;
    (*operation).ft = &OPERATION_FT;
    let t = CreateThread(null_mut(), 0, Some(worker), s as *mut c_void, 0, null_mut());
    if t.is_null() {
        // No thread: complete inline with the failure.
        *(*s).job.get() = Some(Job::Done(MI_RESULT_FAILED));
        worker(s as *mut c_void);
    } else {
        CloseHandle(t);
    }
}

unsafe extern "system" fn op_close(op: *mut MI_Operation) -> MI_Result {
    let Some(st) = state(op) else { return MI_RESULT_INVALID_PARAMETER };
    st.closed.store(true, Ordering::Release);
    st.cancel.store(true, Ordering::Release);
    {
        let _l = Lock::new(st);
        SetEvent(st.changed);
    }
    (*op).reserved1 = 0;
    (*op).reserved2 = 0;
    release(st);
    MI_RESULT_OK
}

unsafe extern "system" fn op_cancel(op: *mut MI_Operation, _reason: MI_CancellationReason) -> MI_Result {
    let Some(st) = state(op) else { return MI_RESULT_INVALID_PARAMETER };
    st.cancel.store(true, Ordering::Release);
    let _l = Lock::new(st);
    SetEvent(st.changed);
    MI_RESULT_OK
}

unsafe extern "system" fn op_get_session(op: *mut MI_Operation, session: *mut MI_Session) -> MI_Result {
    let Some(st) = state(op) else { return MI_RESULT_INVALID_PARAMETER };
    if session.is_null() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    *session = st.session;
    MI_RESULT_OK
}

unsafe extern "system" fn op_get_instance(
    op: *mut MI_Operation,
    instance: *mut *const MI_Instance,
    more: *mut MI_Boolean,
    result: *mut MI_Result,
    error_message: *mut *const MI_Char,
    details: *mut *const MI_Instance,
) -> MI_Result {
    let Some(st) = state(op) else { return MI_RESULT_INVALID_PARAMETER };
    if instance.is_null() || more.is_null() || st.callbacks.is_some() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    if !error_message.is_null() {
        *error_message = null();
    }
    if !details.is_null() {
        *details = null();
    }
    let prev = {
        let _l = Lock::new(st);
        core::mem::replace(&mut inner(st).current, null_mut())
    };
    free_instance(prev);
    // Whether a result is the last one is only known once the next one (or the end) is in.
    wait_until(st, |_, i| i.queue.len() >= 2 || i.done);
    let _l = Lock::new(st);
    let i = inner(st);
    let next = i.queue.pop_front().unwrap_or(null_mut());
    i.current = next;
    *instance = next;
    let last = i.done && i.queue.is_empty();
    *more = (!last) as u8;
    if !result.is_null() {
        *result = if last { i.result } else { MI_RESULT_OK };
    }
    if last && !error_message.is_null() {
        *error_message = i.error.as_ref().map_or(null(), |m| m.as_ptr());
    }
    MI_RESULT_OK
}

unsafe extern "system" fn op_get_indication(
    op: *mut MI_Operation,
    instance: *mut *const MI_Instance,
    _bookmark: *mut *const MI_Char,
    _machine: *mut *const MI_Char,
    more: *mut MI_Boolean,
    result: *mut MI_Result,
    error_message: *mut *const MI_Char,
    details: *mut *const MI_Instance,
) -> MI_Result {
    if state(op).is_none() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    finish_empty(instance, more, result, error_message, details, MI_RESULT_NOT_SUPPORTED)
}

unsafe extern "system" fn op_get_class(
    op: *mut MI_Operation,
    class: *mut *const MI_Class,
    more: *mut MI_Boolean,
    result: *mut MI_Result,
    error_message: *mut *const MI_Char,
    details: *mut *const MI_Instance,
) -> MI_Result {
    if state(op).is_none() {
        return MI_RESULT_INVALID_PARAMETER;
    }
    finish_empty(class as *mut *const MI_Instance, more, result, error_message, details, MI_RESULT_NOT_SUPPORTED)
}

unsafe fn finish_empty(
    out: *mut *const MI_Instance,
    more: *mut MI_Boolean,
    result: *mut MI_Result,
    error_message: *mut *const MI_Char,
    details: *mut *const MI_Instance,
    r: MI_Result,
) -> MI_Result {
    if !out.is_null() {
        *out = null();
    }
    if !more.is_null() {
        *more = 0;
    }
    if !result.is_null() {
        *result = r;
    }
    if !error_message.is_null() {
        *error_message = null();
    }
    if !details.is_null() {
        *details = null();
    }
    MI_RESULT_OK
}

pub static OPERATION_FT: MI_OperationFT = MI_OperationFT {
    Close: Some(op_close),
    Cancel: Some(op_cancel),
    GetSession: Some(op_get_session),
    GetInstance: Some(op_get_instance),
    GetIndication: Some(op_get_indication),
    GetClass: Some(op_get_class),
};
