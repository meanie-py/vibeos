//! Vista+ extended name resolution: `GetAddrInfoExW`, `GetAddrInfoExCancel`,
//! `GetAddrInfoExOverlappedResult`, `FreeAddrInfoExW` / `FreeAddrInfoEx`.
//!
//! Resolution itself is the existing `GetAddrInfoW` (DNS only). Its ADDRINFOW chain is converted
//! to an ADDRINFOEXW chain by MOVING the canonname/address allocations into the new nodes (no
//! copy), so every allocation in the result — and therefore `FreeAddrInfoExW` — lives on
//! `WsSockHeap`.
//!
//! Asynchronous requests (an OVERLAPPED and/or completion routine) run on a thread-pool work item
//! (`QueueUserWorkItem`), like Wine. The cancel handle is the request object; requests that have
//! not yet been delivered sit on a spin-locked registry so a stale handle is rejected instead of
//! dereferenced.
//!
//! Divergences from Windows (all honest, none faked):
//!   * the `timeout` argument is not enforced and `NamespaceId` is ignored (DNS is the only
//!     provider);
//!   * `GetAddrInfoExCancel` completes the request immediately with WSA_E_CANCELLED, but cannot
//!     interrupt a resolver call already in flight — its result is simply discarded.

use crate::prelude::*;
use crate::src::addrinfo::{freeaddrinfo, GetAddrInfoW};
use crate::types::*;
use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

// winsock error codes (winerror.h).
const E_INVALID_HANDLE: u32 = 6; // WSA_INVALID_HANDLE
const E_NOT_ENOUGH_MEMORY: u32 = 8; // WSA_NOT_ENOUGH_MEMORY
const E_IO_PENDING: u32 = 997; // WSA_IO_PENDING == ERROR_IO_PENDING
const E_INVAL: u32 = 10022; // WSAEINVAL
const E_INPROGRESS: u32 = 10036; // WSAEINPROGRESS
const E_CANCELLED: u32 = 10111; // WSA_E_CANCELLED

const NS_ALL: u32 = 0;
const NS_DNS: u32 = 12;
const HEAP_ZERO_MEMORY: u32 = 0x8;

/// C: `ADDRINFOEXW` (ws2def.h). Measured: 72 bytes on x86_64, 44 on i686.
#[repr(C)]
pub struct ADDRINFOEXW {
    pub ai_flags: i32,
    pub ai_family: i32,
    pub ai_socktype: i32,
    pub ai_protocol: i32,
    pub ai_addrlen: usize,
    pub ai_canonname: *mut u16,
    pub ai_addr: *mut sockaddr,
    pub ai_blob: *mut c_void,
    pub ai_bloblen: usize,
    pub ai_provider: *mut GUID,
    pub ai_next: *mut ADDRINFOEXW,
}

const _: () = assert!(core::mem::offset_of!(ADDRINFOEXW, ai_family) == 4);
const _: () = assert!(core::mem::offset_of!(ADDRINFOEXW, ai_socktype) == 8);
const _: () = assert!(core::mem::offset_of!(ADDRINFOEXW, ai_protocol) == 12);
const _: () = assert!(core::mem::offset_of!(ADDRINFOEXW, ai_addrlen) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<ADDRINFOEXW>() == 72);
    assert!(core::mem::offset_of!(ADDRINFOEXW, ai_canonname) == 24);
    assert!(core::mem::offset_of!(ADDRINFOEXW, ai_addr) == 32);
    assert!(core::mem::offset_of!(ADDRINFOEXW, ai_blob) == 40);
    assert!(core::mem::offset_of!(ADDRINFOEXW, ai_bloblen) == 48);
    assert!(core::mem::offset_of!(ADDRINFOEXW, ai_provider) == 56);
    assert!(core::mem::offset_of!(ADDRINFOEXW, ai_next) == 64);
};
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<ADDRINFOEXW>() == 44);
    assert!(core::mem::offset_of!(ADDRINFOEXW, ai_canonname) == 20);
    assert!(core::mem::offset_of!(ADDRINFOEXW, ai_addr) == 24);
    assert!(core::mem::offset_of!(ADDRINFOEXW, ai_blob) == 28);
    assert!(core::mem::offset_of!(ADDRINFOEXW, ai_bloblen) == 32);
    assert!(core::mem::offset_of!(ADDRINFOEXW, ai_provider) == 36);
    assert!(core::mem::offset_of!(ADDRINFOEXW, ai_next) == 40);
};

/// `LPLOOKUPSERVICE_COMPLETION_ROUTINE`: `VOID CALLBACK (DWORD dwError, DWORD dwBytes, LPWSAOVERLAPPED)`.
type CompletionFn = unsafe extern "system" fn(u32, u32, *mut OVERLAPPED);

extern "system" {
    fn QueueUserWorkItem(
        function: unsafe extern "system" fn(*mut c_void) -> u32,
        context: *mut c_void,
        flags: u32,
    ) -> BOOL;
}
extern "C" {
    fn wcslen(s: *const u16) -> usize;
}

// ---------------------------------------------------------------------------------------------
// Freeing.
// ---------------------------------------------------------------------------------------------

unsafe fn free_chain(mut ai: *mut ADDRINFOEXW) {
    while !ai.is_null() {
        let next = (*ai).ai_next;
        if !(*ai).ai_canonname.is_null() {
            HeapFree(WsSockHeap, 0, (*ai).ai_canonname as PVOID);
        }
        if !(*ai).ai_addr.is_null() {
            HeapFree(WsSockHeap, 0, (*ai).ai_addr as PVOID);
        }
        HeapFree(WsSockHeap, 0, ai as PVOID);
        ai = next;
    }
}

/// C: `void FreeAddrInfoExW(PADDRINFOEXW pAddrInfoEx)`.
#[no_mangle]
pub unsafe extern "system" fn FreeAddrInfoExW(pAddrInfoEx: *mut ADDRINFOEXW) {
    free_chain(pAddrInfoEx);
}

/// C: `void FreeAddrInfoEx(PADDRINFOEXA pAddrInfoEx)`. ADDRINFOEXA differs from the W form only in
/// the canonname character width, which does not affect freeing.
#[no_mangle]
pub unsafe extern "system" fn FreeAddrInfoEx(pAddrInfoEx: *mut ADDRINFOEXW) {
    free_chain(pAddrInfoEx);
}

// ---------------------------------------------------------------------------------------------
// ADDRINFOW -> ADDRINFOEXW.
// ---------------------------------------------------------------------------------------------

/// Converts (and consumes) a GetAddrInfoW result chain. On allocation failure everything is
/// freed and WSA_NOT_ENOUGH_MEMORY returned.
unsafe fn convert_chain(w: *mut ADDRINFOW) -> Result<*mut ADDRINFOEXW, u32> {
    let mut head: *mut ADDRINFOEXW = core::ptr::null_mut();
    let mut tail: *mut *mut ADDRINFOEXW = &mut head;
    let mut cur = w;
    while !cur.is_null() {
        let next = (*cur).ai_next;
        let n = HeapAlloc(
            WsSockHeap,
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<ADDRINFOEXW>() as SIZE_T,
        ) as *mut ADDRINFOEXW;
        if n.is_null() {
            freeaddrinfo(cur as PADDRINFOA); // the unconverted remainder
            free_chain(head);
            return Err(E_NOT_ENOUGH_MEMORY);
        }
        (*n).ai_flags = (*cur).ai_flags;
        (*n).ai_family = (*cur).ai_family;
        (*n).ai_socktype = (*cur).ai_socktype;
        (*n).ai_protocol = (*cur).ai_protocol;
        (*n).ai_addrlen = (*cur).ai_addrlen;
        (*n).ai_canonname = (*cur).ai_canonname; // ownership moves
        (*n).ai_addr = (*cur).ai_addr; // ownership moves
        HeapFree(WsSockHeap, 0, cur as PVOID);
        *tail = n;
        tail = &mut (*n).ai_next;
        cur = next;
    }
    Ok(head)
}

/// Resolves synchronously through GetAddrInfoW. `hints` may be null.
unsafe fn resolve(
    name: PCWSTR,
    service: PCWSTR,
    hints: Option<&ADDRINFOW>,
) -> Result<*mut ADDRINFOEXW, u32> {
    let mut res: PADDRINFOW = core::ptr::null_mut();
    let hp = match hints {
        Some(h) => h as *const ADDRINFOW,
        None => core::ptr::null(),
    };
    let r = GetAddrInfoW(name, service, hp, &mut res);
    if r != 0 {
        return Err(r as u32);
    }
    convert_chain(res)
}

fn hints_from_ex(h: &ADDRINFOEXW) -> ADDRINFOW {
    ADDRINFOW {
        ai_flags: h.ai_flags,
        ai_family: h.ai_family,
        ai_socktype: h.ai_socktype,
        ai_protocol: h.ai_protocol,
        ai_addrlen: 0,
        ai_canonname: core::ptr::null_mut(),
        ai_addr: core::ptr::null_mut(),
        ai_next: core::ptr::null_mut(),
    }
}

// ---------------------------------------------------------------------------------------------
// Asynchronous requests.
// ---------------------------------------------------------------------------------------------

const ST_PENDING: u32 = 0;
const ST_DONE: u32 = 1;
const ST_CANCELLED: u32 = 2;

#[repr(C)]
struct Request {
    refs: AtomicU32,
    state: AtomicU32,
    overlapped: *mut OVERLAPPED,
    completion: Option<CompletionFn>,
    event: HANDLE,
    result: *mut *mut ADDRINFOEXW,
    name: *mut u16,
    service: *mut u16,
    hints: ADDRINFOW,
    has_hints: bool,
    next: *mut Request,
}

static REG_LOCK: AtomicBool = AtomicBool::new(false);
static mut REG_HEAD: *mut Request = core::ptr::null_mut();

unsafe fn reg_lock() {
    while REG_LOCK
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}
unsafe fn reg_unlock() {
    REG_LOCK.store(false, Ordering::Release);
}

/// Removes `req` from the registry if present (idempotent).
unsafe fn reg_remove(req: *mut Request) {
    reg_lock();
    let mut pp: *mut *mut Request = &raw mut REG_HEAD;
    while !(*pp).is_null() {
        if *pp == req {
            *pp = (*req).next;
            break;
        }
        pp = &mut (**pp).next;
    }
    reg_unlock();
}

unsafe fn req_release(req: *mut Request) {
    if (*req).refs.fetch_sub(1, Ordering::AcqRel) == 1 {
        if !(*req).name.is_null() {
            HeapFree(WsSockHeap, 0, (*req).name as PVOID);
        }
        if !(*req).service.is_null() {
            HeapFree(WsSockHeap, 0, (*req).service as PVOID);
        }
        HeapFree(WsSockHeap, 0, req as PVOID);
    }
}

/// Publishes the final status to the caller: OVERLAPPED.Internal, then the completion routine
/// and/or the event (both saved at submission, as the OVERLAPPED may be gone after the routine).
unsafe fn deliver(req: *mut Request, code: u32) {
    let ov = (*req).overlapped;
    (*ov).Internal = code as ULONG_PTR;
    if let Some(cb) = (*req).completion {
        cb(code, 0, ov);
    }
    if !(*req).event.is_null() {
        SetEvent((*req).event);
    }
}

unsafe fn dup_wide(s: PCWSTR) -> Result<*mut u16, u32> {
    if s.is_null() {
        return Ok(core::ptr::null_mut());
    }
    let n = wcslen(s) + 1;
    let p = HeapAlloc(WsSockHeap, 0, (n * 2) as SIZE_T) as *mut u16;
    if p.is_null() {
        return Err(E_NOT_ENOUGH_MEMORY);
    }
    core::ptr::copy_nonoverlapping(s, p, n);
    Ok(p)
}

unsafe extern "system" fn worker(ctx: *mut c_void) -> u32 {
    let req = ctx as *mut Request;
    let hints = if (*req).has_hints { Some(&(*req).hints) } else { None };
    let (ex, code) = match resolve((*req).name, (*req).service, hints) {
        Ok(p) => (p, 0u32),
        Err(e) => (core::ptr::null_mut(), e),
    };
    if (*req)
        .state
        .compare_exchange(ST_PENDING, ST_DONE, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        *(*req).result = ex;
        reg_remove(req);
        deliver(req, code);
    } else {
        // Cancelled while resolving: the caller already got WSA_E_CANCELLED.
        free_chain(ex);
        reg_remove(req);
    }
    req_release(req);
    0
}

/// C: `INT GetAddrInfoExW(PCWSTR pName, PCWSTR pServiceName, DWORD dwNameSpace,
/// LPGUID lpNspId, const ADDRINFOEXW *hints, PADDRINFOEXW *ppResult, struct timeval *timeout,
/// LPOVERLAPPED lpOverlapped, LPLOOKUPSERVICE_COMPLETION_ROUTINE lpCompletionRoutine,
/// LPHANDLE lpHandle)`.
///
/// Synchronous when neither an OVERLAPPED nor a completion routine is given; otherwise the
/// request is queued, WSA_IO_PENDING returned and `*ppResult` filled at completion.
#[no_mangle]
pub unsafe extern "system" fn GetAddrInfoExW(
    pName: PCWSTR,
    pServiceName: PCWSTR,
    dwNameSpace: DWORD,
    lpNspId: *mut GUID,
    hints: *const ADDRINFOEXW,
    ppResult: *mut *mut ADDRINFOEXW,
    timeout: *mut timeval,
    lpOverlapped: *mut OVERLAPPED,
    lpCompletionRoutine: Option<CompletionFn>,
    lpHandle: *mut HANDLE,
) -> i32 {
    if ppResult.is_null() || (dwNameSpace != NS_ALL && dwNameSpace != NS_DNS) {
        SetLastError(E_INVAL);
        return E_INVAL as i32;
    }
    *ppResult = core::ptr::null_mut();

    let hw = if hints.is_null() { None } else { Some(hints_from_ex(&*hints)) };

    if lpOverlapped.is_null() && lpCompletionRoutine.is_none() {
        if !lpHandle.is_null() {
            *lpHandle = core::ptr::null_mut(); // nothing to cancel
        }
        return match resolve(pName, pServiceName, hw.as_ref()) {
            Ok(p) => {
                *ppResult = p;
                0
            }
            Err(e) => {
                SetLastError(e);
                e as i32
            }
        };
    }

    // A completion routine needs an OVERLAPPED to carry the status; both an event and a routine
    // is ambiguous (Windows and Wine reject it).
    if lpOverlapped.is_null() || (!(*lpOverlapped).hEvent.is_null() && lpCompletionRoutine.is_some()) {
        SetLastError(E_INVAL);
        return E_INVAL as i32;
    }

    let req = HeapAlloc(WsSockHeap, HEAP_ZERO_MEMORY, core::mem::size_of::<Request>() as SIZE_T)
        as *mut Request;
    if req.is_null() {
        SetLastError(E_NOT_ENOUGH_MEMORY);
        return E_NOT_ENOUGH_MEMORY as i32;
    }
    (*req).refs = AtomicU32::new(1);
    (*req).state = AtomicU32::new(ST_PENDING);
    (*req).overlapped = lpOverlapped;
    (*req).completion = lpCompletionRoutine;
    (*req).event = (*lpOverlapped).hEvent;
    (*req).result = ppResult;
    match (dup_wide(pName), dup_wide(pServiceName)) {
        (Ok(a), Ok(b)) => {
            (*req).name = a;
            (*req).service = b;
        }
        (a, b) => {
            if let Ok(p) = a {
                if !p.is_null() {
                    HeapFree(WsSockHeap, 0, p as PVOID);
                }
            }
            if let Ok(p) = b {
                if !p.is_null() {
                    HeapFree(WsSockHeap, 0, p as PVOID);
                }
            }
            HeapFree(WsSockHeap, 0, req as PVOID);
            SetLastError(E_NOT_ENOUGH_MEMORY);
            return E_NOT_ENOUGH_MEMORY as i32;
        }
    }
    if let Some(h) = hw {
        (*req).hints = h;
        (*req).has_hints = true;
    }

    (*lpOverlapped).Internal = E_INPROGRESS as ULONG_PTR;
    reg_lock();
    (*req).next = REG_HEAD;
    REG_HEAD = req;
    reg_unlock();
    if !lpHandle.is_null() {
        *lpHandle = req as HANDLE;
    }

    if QueueUserWorkItem(worker, req as *mut c_void, 0) == 0 {
        let e = GetLastError();
        reg_remove(req);
        if !lpHandle.is_null() {
            *lpHandle = core::ptr::null_mut();
        }
        req_release(req);
        let e = if e != 0 { e } else { E_NOT_ENOUGH_MEMORY };
        SetLastError(e);
        return e as i32;
    }
    SetLastError(E_IO_PENDING);
    E_IO_PENDING as i32
}

/// C: `INT GetAddrInfoExOverlappedResult(LPOVERLAPPED lpOverlapped)`. Returns the status the
/// request finished with (WSAEINPROGRESS while still pending).
#[no_mangle]
pub unsafe extern "system" fn GetAddrInfoExOverlappedResult(lpOverlapped: *mut OVERLAPPED) -> i32 {
    if lpOverlapped.is_null() {
        return E_INVAL as i32;
    }
    (*lpOverlapped).Internal as i32
}

/// C: `INT GetAddrInfoExCancel(LPHANDLE lpHandle)`. Completes a still-pending request with
/// WSA_E_CANCELLED (completion routine / event fire from this call). WSA_INVALID_HANDLE if the
/// handle is NULL, unknown, or its request already completed.
#[no_mangle]
pub unsafe extern "system" fn GetAddrInfoExCancel(lpHandle: *mut HANDLE) -> i32 {
    if lpHandle.is_null() || (*lpHandle).is_null() {
        SetLastError(E_INVALID_HANDLE);
        return E_INVALID_HANDLE as i32;
    }
    let want = *lpHandle as *mut Request;
    let mut found: *mut Request = core::ptr::null_mut();
    reg_lock();
    let mut p = REG_HEAD;
    while !p.is_null() {
        if p == want {
            (*p).refs.fetch_add(1, Ordering::AcqRel);
            found = p;
            break;
        }
        p = (*p).next;
    }
    reg_unlock();
    if found.is_null() {
        SetLastError(E_INVALID_HANDLE);
        return E_INVALID_HANDLE as i32;
    }
    let won = (*found)
        .state
        .compare_exchange(ST_PENDING, ST_CANCELLED, Ordering::AcqRel, Ordering::Acquire)
        .is_ok();
    if won {
        reg_remove(found);
        deliver(found, E_CANCELLED);
    }
    req_release(found);
    if won {
        0
    } else {
        SetLastError(E_INVALID_HANDLE);
        E_INVALID_HANDLE as i32
    }
}
