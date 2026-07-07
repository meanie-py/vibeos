//! Per-crate write barrier for cc-rust's `pub static mut` globals — a visible,
//! un-eliminable write site so LLVM can't const-fold a global that's 0-initialized
//! in Rust but written from C or another crate. NEVER CALLED; `#[no_mangle]` +
//! pulled into the link via kernel-rust's `extern crate cc_rust`. Relocated from
//! ntoskrnl-rust/src/global_barrier.rs during the crate-split (Phase 1b).
//!
//! Hand-written for the crate-split. When `tools/gen_global_barrier.py` is made
//! crate-aware this becomes generated output — see
//! docs/engineering/ntoskrnl-crate-split-plan.md.
#[allow(unused_unsafe)]
#[allow(dead_code)]
/// Write barrier — touches cc-rust's pub static mut globals. NEVER CALLED.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _cc_global_write_barrier() {
    use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
    write_volatile(addr_of_mut!(crate::cacheman::CcPfGlobals), read_volatile(addr_of!(crate::cacheman::CcPfGlobals)));
    write_volatile(addr_of_mut!(crate::cacheman::CcPfEnablePrefetcher), read_volatile(addr_of!(crate::cacheman::CcPfEnablePrefetcher)));
    write_volatile(addr_of_mut!(crate::cacheman::CcCapturedSystemSize), read_volatile(addr_of!(crate::cacheman::CcCapturedSystemSize)));
    write_volatile(addr_of_mut!(crate::copy::CcRosTraceLevel), read_volatile(addr_of!(crate::copy::CcRosTraceLevel)));
    write_volatile(addr_of_mut!(crate::copy::CcFastMdlReadWait), read_volatile(addr_of!(crate::copy::CcFastMdlReadWait)));
    write_volatile(addr_of_mut!(crate::copy::CcFastMdlReadNotPossible), read_volatile(addr_of!(crate::copy::CcFastMdlReadNotPossible)));
    write_volatile(addr_of_mut!(crate::copy::CcFastReadNotPossible), read_volatile(addr_of!(crate::copy::CcFastReadNotPossible)));
    write_volatile(addr_of_mut!(crate::copy::CcFastReadWait), read_volatile(addr_of!(crate::copy::CcFastReadWait)));
    write_volatile(addr_of_mut!(crate::copy::CcFastReadNoWait), read_volatile(addr_of!(crate::copy::CcFastReadNoWait)));
    write_volatile(addr_of_mut!(crate::copy::CcFastReadResourceMiss), read_volatile(addr_of!(crate::copy::CcFastReadResourceMiss)));
    write_volatile(addr_of_mut!(crate::copy::CcDataPages), read_volatile(addr_of!(crate::copy::CcDataPages)));
    write_volatile(addr_of_mut!(crate::copy::CcDataFlushes), read_volatile(addr_of!(crate::copy::CcDataFlushes)));
    write_volatile(addr_of_mut!(crate::lazywrite::CcLazyWritePages), read_volatile(addr_of!(crate::lazywrite::CcLazyWritePages)));
    write_volatile(addr_of_mut!(crate::lazywrite::CcLazyWriteIos), read_volatile(addr_of!(crate::lazywrite::CcLazyWriteIos)));
    write_volatile(addr_of_mut!(crate::lazywrite::LazyWriter), read_volatile(addr_of!(crate::lazywrite::LazyWriter)));
    write_volatile(addr_of_mut!(crate::lazywrite::CcTwilightLookasideList), read_volatile(addr_of!(crate::lazywrite::CcTwilightLookasideList)));
    write_volatile(addr_of_mut!(crate::lazywrite::CcExpressWorkQueue), read_volatile(addr_of!(crate::lazywrite::CcExpressWorkQueue)));
    write_volatile(addr_of_mut!(crate::lazywrite::CcRegularWorkQueue), read_volatile(addr_of!(crate::lazywrite::CcRegularWorkQueue)));
    write_volatile(addr_of_mut!(crate::lazywrite::CcIdleWorkerThreadList), read_volatile(addr_of!(crate::lazywrite::CcIdleWorkerThreadList)));
    write_volatile(addr_of_mut!(crate::lazywrite::CcPostTickWorkQueue), read_volatile(addr_of!(crate::lazywrite::CcPostTickWorkQueue)));
    write_volatile(addr_of_mut!(crate::lazywrite::CcQueueThrottle), read_volatile(addr_of!(crate::lazywrite::CcQueueThrottle)));
    write_volatile(addr_of_mut!(crate::lazywrite::CcNumberActiveWorkerThreads), read_volatile(addr_of!(crate::lazywrite::CcNumberActiveWorkerThreads)));
    write_volatile(addr_of_mut!(crate::lazywrite::CcFirstDelay), read_volatile(addr_of!(crate::lazywrite::CcFirstDelay)));
    write_volatile(addr_of_mut!(crate::lazywrite::CcIdleDelay), read_volatile(addr_of!(crate::lazywrite::CcIdleDelay)));
    write_volatile(addr_of_mut!(crate::lazywrite::CcNoDelay), read_volatile(addr_of!(crate::lazywrite::CcNoDelay)));
    write_volatile(addr_of_mut!(crate::lazywrite::CcNumberWorkerThreads), read_volatile(addr_of!(crate::lazywrite::CcNumberWorkerThreads)));
    write_volatile(addr_of_mut!(crate::pin::CcMapDataWait), read_volatile(addr_of!(crate::pin::CcMapDataWait)));
    write_volatile(addr_of_mut!(crate::pin::CcMapDataNoWait), read_volatile(addr_of!(crate::pin::CcMapDataNoWait)));
    write_volatile(addr_of_mut!(crate::pin::CcPinReadWait), read_volatile(addr_of!(crate::pin::CcPinReadWait)));
    write_volatile(addr_of_mut!(crate::pin::CcPinReadNoWait), read_volatile(addr_of!(crate::pin::CcPinReadNoWait)));
    write_volatile(addr_of_mut!(crate::pin::CcPinMappedDataCount), read_volatile(addr_of!(crate::pin::CcPinMappedDataCount)));
    write_volatile(addr_of_mut!(crate::view::DirtyVacbListHead), read_volatile(addr_of!(crate::view::DirtyVacbListHead)));
    write_volatile(addr_of_mut!(crate::view::iBcbLookasideList), read_volatile(addr_of!(crate::view::iBcbLookasideList)));
    write_volatile(addr_of_mut!(crate::view::CcDirtyPageThreshold), read_volatile(addr_of!(crate::view::CcDirtyPageThreshold)));
    write_volatile(addr_of_mut!(crate::view::CcTotalDirtyPages), read_volatile(addr_of!(crate::view::CcTotalDirtyPages)));
    write_volatile(addr_of_mut!(crate::view::CcDeferredWrites), read_volatile(addr_of!(crate::view::CcDeferredWrites)));
    write_volatile(addr_of_mut!(crate::view::CcDeferredWriteSpinLock), read_volatile(addr_of!(crate::view::CcDeferredWriteSpinLock)));
}
