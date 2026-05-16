//! csrsrv-internal macros (subsystems/csr/csrsrv/api.h + csrsrv.h),
//! translated once as inline fns — the ntoskrnl-rust `ps/inlines.rs`
//! / win32k "intra-component primitive translated once" analog.
//! Every csrsrv module `use super::srv_inlines::*;`. NOT exported
//! (these are C macros, not csrsrv.spec symbols — no #[no_mangle]).

use ntdll_sys::nt::*;
// The lock globals are defined (once) in the module that owned the C
// `RTL_CRITICAL_SECTION X;` definition; api.h declared them `extern`.
use super::procsup::CsrProcessLock;
use super::session::CsrNtSessionLock;
use super::wait::CsrWaitListsLock;

// api.h: #define CsrAcquireProcessLock() RtlEnterCriticalSection(&CsrProcessLock)
#[inline]
pub unsafe fn CsrAcquireProcessLock() {
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(CsrProcessLock));
}
#[inline]
pub unsafe fn CsrReleaseProcessLock() {
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(CsrProcessLock));
}
// api.h: #define ProcessStructureListLocked()
//        (CsrProcessLock.OwningThread == NtCurrentTeb()->ClientId.UniqueThread)
#[inline]
pub unsafe fn ProcessStructureListLocked() -> bool {
    CsrProcessLock.OwningThread == (*NtCurrentTeb()).ClientId.UniqueThread
}
#[inline]
pub unsafe fn CsrAcquireWaitLock() {
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(CsrWaitListsLock));
}
#[inline]
pub unsafe fn CsrReleaseWaitLock() {
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(CsrWaitListsLock));
}
#[inline]
pub unsafe fn CsrAcquireNtSessionLock() {
    RtlEnterCriticalSection(core::ptr::addr_of_mut!(CsrNtSessionLock));
}
#[inline]
pub unsafe fn CsrReleaseNtSessionLock() {
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(CsrNtSessionLock));
}

// csrsrv.h: #define CsrGetClientThread() ((PCSR_THREAD)(NtCurrentTeb()->CsrClientThread))
#[inline]
pub unsafe fn CsrGetClientThread() -> PCSR_THREAD {
    (*NtCurrentTeb()).CsrClientThread as PCSR_THREAD
}

// srv.h: #define InterlockedIncrementUL(v) (ULONG)_InterlockedIncrement((PLONG)v)
//        #define InterlockedDecrementUL(v) (ULONG)_InterlockedDecrement((PLONG)v)
// `_Interlocked{In,De}crement` return the POST-operation value.
#[inline]
pub unsafe fn InterlockedIncrementUL(value: *mut ULONG) -> ULONG {
    (*(value as *const core::sync::atomic::AtomicU32))
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
        .wrapping_add(1)
}
#[inline]
pub unsafe fn InterlockedDecrementUL(value: *mut ULONG) -> ULONG {
    (*(value as *const core::sync::atomic::AtomicU32))
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        .wrapping_sub(1)
}
