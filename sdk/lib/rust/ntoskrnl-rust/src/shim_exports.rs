//! `#[no_mangle]` export wrappers replacing C `*_rust_shims.c` files.
//!
//! Each function provides a linker symbol that existing Rust `extern`
//! declarations (and any remaining C code) resolve to. The implementations
//! delegate to `ntkrnl_sys::inline::*` for the actual logic.

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// ============================================================================
// SE subsystem exports were extracted to the `se-inlines` sibling rlib
// (see sdk/lib/rust/se-inlines/) to defuse cross-CGU captures(none) contagion.
// ============================================================================

// ============================================================================
// Common shim_exports wrappers were extracted to the `common-inlines` sibling
// rlib (see sdk/lib/rust/common-inlines/) to defuse cross-CGU captures(none)
// contagion. 34 functions in total — see common-inlines for the canonical
// list (Probe* family, splay-link, list-ops, RTL string helpers,
// interlocked, FsRtl region, init helpers, PsGetCurrentThread).
// ============================================================================

// ============================================================================
// KD64 subsystem exports (was: kd64/kd64_rust_shims.c)
// ============================================================================

// --- KE functions (was NTAPI, now cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeSetContextPc(
    ctx: *mut _CONTEXT,
    pc: ULONG_PTR,
) {
    let ctx = opaque_ptr(ctx);
    ntkrnl_sys::inline::ke::KeSetContextPc(ctx, pc as usize)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeSetContextReturnRegister(
    ctx: *mut _CONTEXT,
    val: ULONG_PTR,
) {
    let ctx = opaque_ptr(ctx);
    ntkrnl_sys::inline::ke::KeSetContextReturnRegister(ctx, val as usize)
}

// KeGetContextPc — moved to PS section (all arches, cdecl)

#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeGetPcr() -> PKPCR {
    ntkrnl_sys::inline::ke::KeGetPcr()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeGetCurrentProcessorNumber() -> ULONG {
    ntkrnl_sys::inline::ke::KeGetCurrentProcessorNumber()
}

/// The exported way (Windows 7's) for a driver to ask which processor it is on:
/// KeGetCurrentProcessorNumber is a header inline that reads the PCR, which a
/// driver built once for the UP and MP kernels cannot do. One group: the index is
/// the processor number.
#[no_mangle]
pub unsafe extern "system" fn KeGetCurrentProcessorNumberEx(ProcNumber: ntkrnl_sys::ddk::PPROCESSOR_NUMBER) -> ULONG {
    let number = ntkrnl_sys::inline::ke::KeGetCurrentProcessorNumber();
    if !ProcNumber.is_null() {
        (*ProcNumber).Group = 0;
        (*ProcNumber).Number = number as u8;
        (*ProcNumber).Reserved = 0;
    }
    number
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeDisableInterrupts() -> BOOLEAN {
    ntkrnl_sys::inline::ke::KeDisableInterrupts()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeRestoreInterrupts(were_enabled: BOOLEAN) {
    ntkrnl_sys::inline::ke::KeRestoreInterrupts(were_enabled)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeSweepICache(
    base: PVOID,
    size: usize,
) {
    let base = opaque_ptr(base);
    ntkrnl_sys::inline::ke::KeSweepICache(base, size)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeGetTrapFrameInterruptState(
    tf: *const _KTRAP_FRAME,
) -> BOOLEAN {
    ntkrnl_sys::inline::ke::KeGetTrapFrameInterruptState(tf)
}

// --- MM functions ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsMemoryTypeInvisible(
    mem_type: TYPE_OF_MEMORY,
) -> BOOLEAN {
    ntkrnl_sys::inline::mm::MiIsMemoryTypeInvisible(mem_type)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MmGetMdlPfnArray(mdl: PMDL) -> *mut ULONG_PTR {
    let mdl = opaque_ptr(mdl);
    ntkrnl_sys::inline::mm::MmGetMdlPfnArray(mdl) as *mut ULONG_PTR
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MmGetSystemAddressForMdl(mdl: PMDL) -> PVOID {
    let mdl = opaque_ptr(mdl);
    ntkrnl_sys::inline::mm::MmGetSystemAddressForMdl(mdl)
}

// --- EX functions ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExAllocateFromNPagedLookasideList(
    lookaside: PNPAGED_LOOKASIDE_LIST,
) -> PVOID {
    ntkrnl_sys::inline::ex::ExAllocateFromNPagedLookasideList(lookaside)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExFreeToNPagedLookasideList(
    lookaside: PNPAGED_LOOKASIDE_LIST,
    entry: PVOID,
) {
    let entry = opaque_ptr(entry);
    ntkrnl_sys::inline::ex::ExFreeToNPagedLookasideList(lookaside, entry)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn ExAllocateFromPagedLookasideList(
    lookaside: PPAGED_LOOKASIDE_LIST,
) -> PVOID {
    ntkrnl_sys::inline::ex::ExAllocateFromPagedLookasideList(lookaside)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExInitializeWorkItem(
    item: *mut _WORK_QUEUE_ITEM,
    routine: PWORKER_THREAD_ROUTINE,
    context: PVOID,
) {
    let item = opaque_ptr(item);
    let context = opaque_ptr(context);
    ntkrnl_sys::inline::ex::ExInitializeWorkItem(item, routine, context)
}

// FsRtlEnterFileSystem/FsRtlExitFileSystem, CcBugCheck, splay-link helpers,
// RtlLongLongAdd, ExceptionRecord32To64 → moved to the `common-inlines`
// sibling rlib.

// ============================================================================
// Tier 1 subsystem exports (was: tier1_rust_shims.c)
// ============================================================================

// --- CONTAINING_RECORD (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn CONTAINING_RECORD(
    address: PVOID,
    type_size: usize,
    field_offset: usize,
) -> PVOID {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::tier1::CONTAINING_RECORD(address, type_size, field_offset)
}

// ProbeForWritePointer / ProbeForWriteUlong → moved to `common-inlines`.

// --- IoGetDevObjExtension (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoGetDevObjExtension(
    device_object: PDEVICE_OBJECT,
) -> PEXTENDED_DEVOBJ_EXTENSION {
    let device_object = opaque_ptr(device_object);
    ntkrnl_sys::inline::tier1::IoGetDevObjExtension(device_object)
}

// --- SeDeleteClientSecurity moved to the `se-inlines` sibling rlib. ---

// --- ObReferenceObject (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObReferenceObject(object: PVOID) -> LONG_PTR {
    let object = opaque_ptr(object);
    ntkrnl_sys::inline::tier1::ObReferenceObject(object) as LONG_PTR
}

// --- NtCurrentProcess (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn NtCurrentProcess() -> HANDLE {
    ntkrnl_sys::inline::tier1::NtCurrentProcess()
}

// --- PsGetCurrentProcess (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn PsGetCurrentProcess() -> PEPROCESS {
    ntkrnl_sys::inline::tier1::PsGetCurrentProcess()
}

// --- _rs_PsCurrentProcessTokenAddr ---
// Returns &CurrentProcess->Token as a *mut PVOID. Lets header-free C
// trampolines (e.g. ntoskrnl/lto_workaround.c) initialise the process
// token without needing the EPROCESS layout in C.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_PsCurrentProcessTokenAddr() -> *mut PVOID {
    use ntkrnl_sys::ddk::_EPROCESS;
    core::ptr::addr_of_mut!(
        (*(PsGetCurrentProcess() as *mut _EPROCESS)).Token
    ) as *mut PVOID
}

// --- RtlUnicodeStringToOemSize (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn RtlUnicodeStringToOemSize(
    unicode_string: PCUNICODE_STRING,
) -> ULONG {
    ntkrnl_sys::inline::tier1::RtlUnicodeStringToOemSize(unicode_string)
}

// RtlInitEmptyAnsiString / InsertTailList → moved to `common-inlines`.

// --- ExFreeToPagedLookasideList (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn ExFreeToPagedLookasideList(
    lookaside: PPAGED_LOOKASIDE_LIST,
    entry: PVOID,
) {
    let entry = opaque_ptr(entry);
    ntkrnl_sys::inline::ex::ExFreeToPagedLookasideList(lookaside, entry)
}

// --- LPC wait macros (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpReceiveWait(
    semaphore: PVOID,
    wait_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let semaphore = opaque_ptr(semaphore);
    ntkrnl_sys::inline::tier1::LpcpReceiveWait(semaphore, wait_mode)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn LpcpReplyWait(
    semaphore: PVOID,
    wait_mode: KPROCESSOR_MODE,
) {
    let semaphore = opaque_ptr(semaphore);
    ntkrnl_sys::inline::tier1::LpcpReplyWait(semaphore, wait_mode)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn LpcpConnectWait(
    semaphore: PVOID,
    wait_mode: KPROCESSOR_MODE,
) {
    let semaphore = opaque_ptr(semaphore);
    ntkrnl_sys::inline::tier1::LpcpConnectWait(semaphore, wait_mode)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn LpcpCompleteWait(semaphore: PVOID) {
    let semaphore = opaque_ptr(semaphore);
    ntkrnl_sys::inline::tier1::LpcpCompleteWait(semaphore)
}

// --- LPC FORCEINLINE functions ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn LpcpAllocateFromPortZone() -> PLPCP_MESSAGE {
    ntkrnl_sys::inline::tier1::LpcpAllocateFromPortZone()
}

// LpcpGetMessageFromThread / LpcpGetPortFromThread / LpcpSetPortToThread:
// cdecl (no NTAPI in C shim) — Rust DDK bindgen declares them as extern "C".

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpGetMessageFromThread(thread: PETHREAD) -> PLPCP_MESSAGE {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::tier1::LpcpGetMessageFromThread(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpGetPortFromThread(thread: PETHREAD) -> PLPCP_PORT_OBJECT {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::tier1::LpcpGetPortFromThread(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpSetPortToThread(thread: PETHREAD, port: PLPCP_PORT_OBJECT) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::tier1::LpcpSetPortToThread(thread, port)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpGetDataInfoFromMessage(
    message: *mut _PORT_MESSAGE,
) -> PLPCP_DATA_INFO {
    let message = opaque_ptr(message);
    ntkrnl_sys::inline::tier1::LpcpGetDataInfoFromMessage(message)
}

// --- HAL dispatch table macros (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn HalHaltSystem() {
    ntkrnl_sys::inline::tier1::HalHaltSystem()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn HalSetWakeEnable(new_state: BOOLEAN) {
    ntkrnl_sys::inline::tier1::HalSetWakeEnable(new_state)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn HalEndOfBoot() {
    ntkrnl_sys::inline::tier1::HalEndOfBoot()
}

// --- IO IRP stack macros (cdecl — no NTAPI in C shim) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoGetNextIrpStackLocation(irp: PIRP) -> PIO_STACK_LOCATION {
    let irp = opaque_ptr(irp);
    ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation(irp)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoGetCurrentIrpStackLocation(irp: PIRP) -> PIO_STACK_LOCATION {
    let irp = opaque_ptr(irp);
    ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation(irp)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoSetNextIrpStackLocation(irp: PIRP) {
    let irp = opaque_ptr(irp);
    ntkrnl_sys::inline::tier1::IoSetNextIrpStackLocation(irp)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoSkipCurrentIrpStackLocation(irp: PIRP) {
    let irp = opaque_ptr(irp);
    ntkrnl_sys::inline::tier1::IoSkipCurrentIrpStackLocation(irp)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoSetCancelRoutine(
    irp: PIRP,
    cancel_routine: PDRIVER_CANCEL,
) -> PDRIVER_CANCEL {
    let irp = opaque_ptr(irp);
    ntkrnl_sys::inline::tier1::IoSetCancelRoutine(irp, cancel_routine)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoMarkIrpPending(irp: PIRP) {
    let irp = opaque_ptr(irp);
    ntkrnl_sys::inline::tier1::IoMarkIrpPending(irp)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoSetCompletionRoutine(
    irp: PIRP,
    completion_routine: PIO_COMPLETION_ROUTINE,
    context: PVOID,
    invoke_on_success: BOOLEAN,
    invoke_on_error: BOOLEAN,
    invoke_on_cancel: BOOLEAN,
) {
    let irp = opaque_ptr(irp);
    let context = opaque_ptr(context);
    ntkrnl_sys::inline::tier1::IoSetCompletionRoutine(
        irp,
        completion_routine,
        context,
        invoke_on_success,
        invoke_on_error,
        invoke_on_cancel,
    )
}

// --- KeRaiseIrqlToDpcLevel (NTAPI, amd64 only) ---

#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeRaiseIrqlToDpcLevel() -> KIRQL {
    ntkrnl_sys::inline::tier1::KeRaiseIrqlToDpcLevel()
}

// --- KeRaiseIrql / KeLowerIrql (NTAPI, i386 only) ---

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeRaiseIrql(new_irql: KIRQL, old_irql: *mut KIRQL) {
    let old_irql = opaque_ptr(old_irql);
    ntkrnl_sys::inline::tier1::KeRaiseIrql(new_irql, old_irql)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeLowerIrql(new_irql: KIRQL) {
    ntkrnl_sys::inline::tier1::KeLowerIrql(new_irql)
}

// KeGetCurrentThread — provided by ke/thrdobj.rs, not needed here.

// --- SRW locks: NOT here. rtl-sync (srw.rs, the translation of ReactOS srw.c) is the
// one provider. A CAS spin-lock stub used to be exported from this file under the same
// four names; --allow-multiple-definition let it shadow rtl-sync's on amd64/arm64, so the
// kernel ran a lock whose word encoding differs from the one rtl-sync's condition
// variables read. scan_duplicate_symbols gates a second provider now.

// --- Tier 1 globals ---

/// LdrpShutdownInProgress — dummy for kernel mode (sdk/lib/rtl/critical.c references it).
#[no_mangle]
pub static mut LdrpShutdownInProgress: BOOLEAN = 0;

/// LdrpShutdownThreadId — dummy for kernel mode.
#[no_mangle]
pub static mut LdrpShutdownThreadId: HANDLE = core::ptr::null_mut();

/// VdmState — i386 only. Points to FIXED_NTVDMSTATE_LINEAR_PC_AT (0x714).
#[cfg(target_arch = "x86")]
#[no_mangle]
pub static mut VdmState: *mut ULONG = 0x714 as *mut ULONG;

// ============================================================================
// EX subsystem exports (was: ex/ex_rust_shims.c)
// ============================================================================

// --- KeGetContextSwitches (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeGetContextSwitches(prcb: PKPRCB) -> ULONG {
    ntkrnl_sys::inline::ke::KeGetContextSwitches(prcb)
}

// --- MmGetSystemAddressForMdlSafe (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MmGetSystemAddressForMdlSafe(mdl: PMDL, priority: ULONG) -> PVOID {
    let mdl = opaque_ptr(mdl);
    ntkrnl_sys::inline::mm::MmGetSystemAddressForMdlSafe(mdl, priority)
}

// --- ObpGetHandleObject (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpGetHandleObject(entry: PHANDLE_TABLE_ENTRY) -> POBJECT_HEADER {
    let entry = opaque_ptr(entry);
    ntkrnl_sys::macros::ObpGetHandleObject(entry)
}

// ProbeForWriteBoolean / ProbeForWriteLangId / InterlockedExchangeAdd64 /
// PopEntryList → moved to `common-inlines`.

// --- Fast Mutex functions (cdecl — no NTAPI in C shim) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _ExAcquireFastMutex(fast_mutex: PFAST_MUTEX) {
    ntkrnl_sys::inline::ex::_ExAcquireFastMutex(fast_mutex)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _ExReleaseFastMutex(fast_mutex: PFAST_MUTEX) {
    ntkrnl_sys::inline::ex::_ExReleaseFastMutex(fast_mutex)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _ExAcquireFastMutexUnsafe(fast_mutex: PFAST_MUTEX) {
    ntkrnl_sys::inline::ex::_ExAcquireFastMutexUnsafe(fast_mutex)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _ExReleaseFastMutexUnsafe(fast_mutex: PFAST_MUTEX) {
    ntkrnl_sys::inline::ex::_ExReleaseFastMutexUnsafe(fast_mutex)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _ExEnterCriticalRegionAndAcquireFastMutexUnsafe(fast_mutex: PFAST_MUTEX) {
    ntkrnl_sys::inline::ex::_ExEnterCriticalRegionAndAcquireFastMutexUnsafe(fast_mutex)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(fast_mutex: PFAST_MUTEX) {
    ntkrnl_sys::inline::ex::_ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(fast_mutex)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _ExTryToAcquireFastMutex(fast_mutex: PFAST_MUTEX) -> BOOLEAN {
    ntkrnl_sys::inline::ex::_ExTryToAcquireFastMutex(fast_mutex)
}

// --- ExGetRunRefForGivenProcessor (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExGetRunRefForGivenProcessor(
    run_ref_cache_aware: PEX_RUNDOWN_REF_CACHE_AWARE,
    proc_number: ULONG,
) -> PEX_RUNDOWN_REF {
    ntkrnl_sys::inline::ex::ExGetRunRefForGivenProcessor(run_ref_cache_aware, proc_number)
}

// --- ExWaitOnPushLock (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExWaitOnPushLock(push_lock: PEX_PUSH_LOCK) {
    ntkrnl_sys::inline::ex::ExWaitOnPushLock(push_lock)
}

// RtlStringCbCopyA → moved to `common-inlines`.

// --- KxAcquireSpinLock / KxReleaseSpinLock (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxAcquireSpinLock(spin_lock: *mut ULONG_PTR) {
    let spin_lock = opaque_ptr(spin_lock);
    ntkrnl_sys::inline::ke::KxAcquireSpinLock(spin_lock as *mut usize)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxReleaseSpinLock(spin_lock: *mut ULONG_PTR) {
    let spin_lock = opaque_ptr(spin_lock);
    ntkrnl_sys::inline::ke::KxReleaseSpinLock(spin_lock as *mut usize)
}

// ============================================================================
// OB subsystem exports (was: ob/ob_rust_shims.c)
// ============================================================================

// --- KE critical region (NTAPI = stdcall on i386) ---
// Must be extern "system" to match calling convention of ntkrnl-sys extern declarations.

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn KeEnterCriticalRegion() {
    ntkrnl_sys::inline::ke::KeEnterCriticalRegion()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn KeLeaveCriticalRegion() {
    ntkrnl_sys::inline::ke::KeLeaveCriticalRegion()
}

// --- KE KeGetCurrentPrcb (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}

// --- KE KeQueryInterruptTime (NTAPI, amd64/arm64 only) ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeQueryInterruptTime() -> ULONGLONG {
    ntkrnl_sys::inline::ke::KeQueryInterruptTime()
}

// PsGetCurrentThread (extern "system" = NTAPI) and RtlInitEmptyUnicodeString
// → moved to `common-inlines`.

// --- EX Rundown protection (FASTCALL = fastcall on i386, cdecl on 64-bit) ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExAcquireRundownProtection(
    run_ref: PEX_RUNDOWN_REF,
) -> BOOLEAN {
    ntkrnl_sys::inline::ex::ExAcquireRundownProtection(run_ref)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn ExAcquireRundownProtection(
    run_ref: PEX_RUNDOWN_REF,
) -> BOOLEAN {
    ntkrnl_sys::inline::ex::ExAcquireRundownProtection(run_ref)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExReleaseRundownProtection(
    run_ref: PEX_RUNDOWN_REF,
) {
    ntkrnl_sys::inline::ex::ExReleaseRundownProtection(run_ref)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn ExReleaseRundownProtection(
    run_ref: PEX_RUNDOWN_REF,
) {
    ntkrnl_sys::inline::ex::ExReleaseRundownProtection(run_ref)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExWaitForRundownProtectionRelease(
    run_ref: PEX_RUNDOWN_REF,
) {
    ntkrnl_sys::inline::ex::ExWaitForRundownProtectionRelease(run_ref)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn ExWaitForRundownProtectionRelease(
    run_ref: PEX_RUNDOWN_REF,
) {
    ntkrnl_sys::inline::ex::ExWaitForRundownProtectionRelease(run_ref)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExRundownCompleted(
    run_ref: PEX_RUNDOWN_REF,
) {
    ntkrnl_sys::inline::ex::ExRundownCompleted(run_ref)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn ExRundownCompleted(
    run_ref: PEX_RUNDOWN_REF,
) {
    ntkrnl_sys::inline::ex::ExRundownCompleted(run_ref)
}

// --- ExInitializeRundownProtection (FASTCALL on i386, cdecl on 64-bit) ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExInitializeRundownProtection(
    run_ref: PEX_RUNDOWN_REF,
) {
    ntkrnl_sys::inline::ex::ExInitializeRundownProtection(run_ref)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn ExInitializeRundownProtection(
    run_ref: PEX_RUNDOWN_REF,
) {
    ntkrnl_sys::inline::ex::ExInitializeRundownProtection(run_ref)
}

// --- EX Push lock functions ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn ExInitializePushLock(
    push_lock: PEX_PUSH_LOCK,
) {
    ntkrnl_sys::inline::ex::ExInitializePushLock(push_lock)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExAcquirePushLockExclusive(
    push_lock: PEX_PUSH_LOCK,
) {
    // lockdep §4: site = this #[inline(never)] shim's own PC (code_addr!) —
    // shim-identity, not the C caller (Windows-x64 has no frame chain to
    // walk on stable; see lockdep.rs code_addr! history). Rust callers get
    // exact sites via the *Sited inline routes. Symbolize with addr2sym.sh.
    #[cfg(feature = "lockdep")]
    let site = crate::code_addr!();
    #[cfg(not(feature = "lockdep"))]
    let site = 0usize;
    ntkrnl_sys::inline::ex::ExAcquirePushLockExclusiveSited(push_lock, site)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExAcquirePushLockShared(
    push_lock: PEX_PUSH_LOCK,
) {
    // lockdep §4: see ExAcquirePushLockExclusive above.
    #[cfg(feature = "lockdep")]
    let site = crate::code_addr!();
    #[cfg(not(feature = "lockdep"))]
    let site = 0usize;
    ntkrnl_sys::inline::ex::ExAcquirePushLockSharedSited(push_lock, site)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExReleasePushLock(
    push_lock: PEX_PUSH_LOCK,
) {
    ntkrnl_sys::inline::ex::ExReleasePushLock(push_lock)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExReleasePushLockExclusive(
    push_lock: PEX_PUSH_LOCK,
) {
    ntkrnl_sys::inline::ex::ExReleasePushLockExclusive(push_lock)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExConvertPushLockSharedToExclusive(
    push_lock: PEX_PUSH_LOCK,
) -> BOOLEAN {
    ntkrnl_sys::inline::ex::ExConvertPushLockSharedToExclusive(push_lock)
}

// --- EX Fast reference functions (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExGetObjectFastReference(
    fast_ref: _EX_FAST_REF,
) -> PVOID {
    let value: usize = core::mem::transmute(fast_ref);
    ntkrnl_sys::inline::ex::ExGetObjectFastReference(value)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExGetCountFastReference(
    fast_ref: _EX_FAST_REF,
) -> ULONG {
    let value: usize = core::mem::transmute(fast_ref);
    ntkrnl_sys::inline::ex::ExGetCountFastReference(value)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExInitializeFastReference(
    fast_ref: PEX_FAST_REF,
    object: PVOID,
) {
    let object = opaque_ptr(object);
    ntkrnl_sys::inline::ex::ExInitializeFastReference(fast_ref, object)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExAcquireFastReference(
    fast_ref: PEX_FAST_REF,
) -> _EX_FAST_REF {
    let result = ntkrnl_sys::inline::ex::ExAcquireFastReference(fast_ref);
    core::mem::transmute(result)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExInsertFastReference(
    fast_ref: PEX_FAST_REF,
    object: PVOID,
) -> BOOLEAN {
    let object = opaque_ptr(object);
    ntkrnl_sys::inline::ex::ExInsertFastReference(fast_ref, object)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExReleaseFastReference(
    fast_ref: PEX_FAST_REF,
    object: PVOID,
) -> BOOLEAN {
    let object = opaque_ptr(object);
    ntkrnl_sys::inline::ex::ExReleaseFastReference(fast_ref, object)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExSwapFastReference(
    fast_ref: PEX_FAST_REF,
    object: PVOID,
) -> _EX_FAST_REF {
    let object = opaque_ptr(object);
    let result = ntkrnl_sys::inline::ex::ExSwapFastReference(fast_ref, object);
    core::mem::transmute(result)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExCompareSwapFastReference(
    fast_ref: PEX_FAST_REF,
    object: PVOID,
    old_object: PVOID,
) -> _EX_FAST_REF {
    let object = opaque_ptr(object);
    let old_object = opaque_ptr(old_object);
    let result = ntkrnl_sys::inline::ex::ExCompareSwapFastReference(
        fast_ref,
        object,
        old_object,
    );
    core::mem::transmute(result)
}

// --- OB ob_x.h FORCEINLINE wrappers ---
// Extracted into the `ob-inlines` rlib to defuse the cross-CGU
// captures(none) contagion risk that killed the original cm Pilot 1.
// `ObReferenceObject` and `ObpGetHandleObject` stay above — different bodies
// (`tier1`, `macros`).

// ============================================================================
// PS subsystem exports (was: ps/ps_rust_shims.c)
// ============================================================================

// --- PS ps_x.h FORCEINLINE wrappers (macro trampolines + lock helpers) ---
// Extracted into the `ps-inlines` rlib (sibling pattern).
// `PspClearCrossThreadFlag` lives in `common-inlines` (its body is in
// `inline::common`, not `inline::ps`).

// --- KeGetContextPc / KeGetContextReturnRegister (cdecl — no NTAPI) ---
// Note: KeGetContextPc is also exported from KD64 section for i386.
// These cdecl exports cover all arches for PS callers.

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeGetContextPc(ctx: *const _CONTEXT) -> ULONG_PTR {
    ntkrnl_sys::inline::ke::KeGetContextPc(ctx) as ULONG_PTR
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeGetContextReturnRegister(
    ctx: *mut _CONTEXT,
) -> ULONG_PTR {
    let ctx = opaque_ptr(ctx);
    ntkrnl_sys::inline::ke::KeGetContextReturnRegister(ctx) as ULONG_PTR
}

// InterlockedDecrementUL → moved to `common-inlines`.

// --- KeQuerySystemTime (NTAPI, amd64/arm64 only — i386 has real HAL function) ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeQuerySystemTime(current_time: PLARGE_INTEGER) {
    let current_time = opaque_ptr(current_time);
    ntkrnl_sys::inline::ke::KeQuerySystemTime(current_time)
}

// --- KeGetCurrentIrql / KeLowerIrql / KeRaiseIrql (NTAPI, amd64 only) ---

#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeGetCurrentIrql() -> KIRQL {
    ntkrnl_sys::inline::ke::KeGetCurrentIrql_amd64()
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeLowerIrql(new_irql: KIRQL) {
    ntkrnl_sys::inline::ke::KeLowerIrql_amd64(new_irql)
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeRaiseIrql(new_irql: KIRQL, old_irql: *mut KIRQL) {
    let old_irql = opaque_ptr(old_irql);
    *old_irql = ntkrnl_sys::inline::ke::KfRaiseIrql_amd64(new_irql);
}

// --- KeEnterGuardedRegion / KeLeaveGuardedRegion (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn KeEnterGuardedRegion() {
    ntkrnl_sys::inline::ke::KeEnterGuardedRegion()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn KeLeaveGuardedRegion() {
    ntkrnl_sys::inline::ke::KeLeaveGuardedRegion()
}

// --- KeEnterGuardedRegionThread / KeLeaveGuardedRegionThread (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeEnterGuardedRegionThread(thread: PKTHREAD) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KeEnterGuardedRegionThread(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeLeaveGuardedRegionThread(thread: PKTHREAD) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KeLeaveGuardedRegionThread(thread)
}

// --- ExRaiseStatus (NTAPI) ---
//
// Spec entry was historically a forwarder to RtlRaiseStatus; we now export this
// shim instead so future instrumentation hooks land on the kernel-side caller
// (e.g. fastfat under USE_DUMMY_PSEH).

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn ExRaiseStatus(status: NTSTATUS) -> ! {
    ntkrnl_sys::inline::ke::ExRaiseStatus(status)
}

// InterlockedCompareExchangePointer → moved to `common-inlines`.
//
// InterlockedCompareExchange — already provided by ex/exintrin.rs with correct
// calling conventions (fastcall on i386, cdecl on 64-bit).

// --- KeGetTrapFrame (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeGetTrapFrame(thread: PKTHREAD) -> PKTRAP_FRAME {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KeGetTrapFrame(thread)
}

// --- KeGetExceptionFrame (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeGetExceptionFrame(thread: PKTHREAD) -> PKEXCEPTION_FRAME {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KeGetExceptionFrame(thread)
}

// --- PS ps_x.h FORCEINLINE function wrappers — extracted into ps-inlines rlib. ---

// --- NtCurrentTeb (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn NtCurrentTeb() -> PVOID {
    ntkrnl_sys::inline::ke::NtCurrentTeb()
}

// --- KeInitializeSpinLock (cdecl) ---
// Note: already exists in KE subsystem; this provides the PS export.

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeInitializeSpinLock(spin_lock: *mut ULONG_PTR) {
    let spin_lock = opaque_ptr(spin_lock);
    ntkrnl_sys::inline::ke::KeInitializeSpinLock(spin_lock as *mut usize)
}

// --- ExReleasePushLockShared (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExReleasePushLockShared(push_lock: PEX_PUSH_LOCK) {
    ntkrnl_sys::inline::ex::ExReleasePushLockShared(push_lock)
}

// --- ExInitializeFastMutex (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExInitializeFastMutex(fast_mutex: PFAST_MUTEX) {
    ntkrnl_sys::inline::ex::ExInitializeFastMutex(fast_mutex)
}

// --- PspDumpThreads (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn PspDumpThreads(include_system: BOOLEAN) {
    ntkrnl_sys::inline::ke::PspDumpThreads(include_system)
}

// --- Ki386PerfEnd (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn Ki386PerfEnd() {
    ntkrnl_sys::inline::ke::Ki386PerfEnd()
}

// ============================================================================
// MM subsystem exports (was: mm/mm_rust_shims.c)
// ============================================================================

// MiIsMemoryTypeInvisible, MiIsMemoryTypeFree — already in KD64 section
// MmGetMdlPfnArray, MmGetSystemAddressForMdl, MmGetSystemAddressForMdlSafe — already in KD64 section

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsMemoryTypeFree(
    mem_type: ntkrnl_sys::ddk::TYPE_OF_MEMORY,
) -> BOOLEAN {
    ntkrnl_sys::inline::mm::MiIsMemoryTypeFree(mem_type)
}

// --- PTE portable macros (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiMakeDirtyPage(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiMakeDirtyPage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiMakeCleanPage(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiMakeCleanPage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiMakeAccessedPage(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiMakeAccessedPage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiPageDisableCache(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiPageDisableCache(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiPageWriteThrough(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiPageWriteThrough(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiPageWriteCombined(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiPageWriteCombined(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsPageWriteable(pte: PMMPTE) -> BOOLEAN {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiIsPageWriteable(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsPageDirty(pte: PMMPTE) -> BOOLEAN {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiIsPageDirty(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiMakeOwnerPage(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiMakeOwnerPage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiMakeWritePage(pte: PMMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiMakeWritePage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsMappedPte(pte: PMMPTE) -> BOOLEAN {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiIsMappedPte(pte)
}

// --- PTE atomics (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiInterlockedCompareExchangePte(
    pte: PMMPTE,
    exchange: ntkrnl_sys::base::LONG_PTR,
    comperand: ntkrnl_sys::base::LONG_PTR,
) -> ntkrnl_sys::base::LONG_PTR {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiInterlockedCompareExchangePte(pte, exchange, comperand)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiInterlockedExchangePte(
    pte: PMMPTE,
    value: ntkrnl_sys::base::LONG_PTR,
) -> ntkrnl_sys::base::LONG_PTR {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MiInterlockedExchangePte(pte, value)
}

// --- TLB operations (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeInvalidateTlbEntry(address: PVOID) {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::mm::KeInvalidateTlbEntry(address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeFlushProcessTb() {
    ntkrnl_sys::inline::mm::KeFlushProcessTb()
}

// --- Working set lock check (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MM_ANY_WS_LOCK_HELD(thread: PETHREAD) -> BOOLEAN {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MM_ANY_WS_LOCK_HELD(thread)
}

// --- PDE hierarchy check (64-bit only, cdecl) ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsPdeForAddressValid(address: PVOID) -> BOOLEAN {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::mm::MiIsPdeForAddressValid(address)
}

// --- Working set locks (cdecl) ---
// Replaces the C trampolines in mm_rust_shims.c. The historical
// `MmDbgLast{Safe,Excl}{Locker,Unlocker}` debug instrumentation
// (volatile pointer stores tracking the last `__builtin_return_address`)
// was dead — its only consumer was `MmDebugWsFaultCheck` which has no
// callers. ARM64 IRQL UART traces in the C trampolines were
// debugging-only for a fixed-since IRQL violation; if needed again
// they can be re-added inside `ntkrnl_sys::inline::mm`.

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiLockProcessWorkingSet(process: PEPROCESS, thread: PETHREAD) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiLockProcessWorkingSet(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiUnlockProcessWorkingSet(process: PEPROCESS, thread: PETHREAD) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiUnlockProcessWorkingSet(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiLockProcessWorkingSetUnsafe(process: PEPROCESS, thread: PETHREAD) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiLockProcessWorkingSetUnsafe(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiUnlockProcessWorkingSetUnsafe(process: PEPROCESS, thread: PETHREAD) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiUnlockProcessWorkingSetUnsafe(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiLockProcessWorkingSetShared(process: PEPROCESS, thread: PETHREAD) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiLockProcessWorkingSetShared(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiUnlockProcessWorkingSetShared(process: PEPROCESS, thread: PETHREAD) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiUnlockProcessWorkingSetShared(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiLockWorkingSet(thread: PETHREAD, ws: PMMSUPPORT) {
    let thread = opaque_ptr(thread);
    let ws = opaque_ptr(ws);
    ntkrnl_sys::inline::mm::MiLockWorkingSet(thread, ws)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiUnlockWorkingSet(thread: PETHREAD, ws: PMMSUPPORT) {
    let thread = opaque_ptr(thread);
    let ws = opaque_ptr(ws);
    ntkrnl_sys::inline::mm::MiUnlockWorkingSet(thread, ws)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiLockWorkingSetShared(thread: PETHREAD, ws: PMMSUPPORT) {
    let thread = opaque_ptr(thread);
    let ws = opaque_ptr(ws);
    ntkrnl_sys::inline::mm::MiLockWorkingSetShared(thread, ws)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiUnlockWorkingSetShared(thread: PETHREAD, ws: PMMSUPPORT) {
    let thread = opaque_ptr(thread);
    let ws = opaque_ptr(ws);
    ntkrnl_sys::inline::mm::MiUnlockWorkingSetShared(thread, ws)
}

// --- Working set lock (fault path) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiLockProcessWorkingSetForFault(
    process: PEPROCESS,
    thread: PETHREAD,
    safe: BOOLEAN,
    shared: BOOLEAN,
) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiLockProcessWorkingSetForFault(process, thread, safe, shared)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiUnlockProcessWorkingSetForFault(
    process: PEPROCESS,
    thread: PETHREAD,
    safe: *mut BOOLEAN,
    shared: *mut BOOLEAN,
) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    let safe = opaque_ptr(safe);
    let shared = opaque_ptr(shared);
    ntkrnl_sys::inline::mm::MiUnlockProcessWorkingSetForFault(process, thread, safe, shared)
}

// --- Shared→exclusive conversion ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiConvertSharedWorkingSetLockToExclusive(
    thread: PETHREAD,
    vm: PMMSUPPORT,
) -> BOOLEAN {
    let thread = opaque_ptr(thread);
    let vm = opaque_ptr(vm);
    ntkrnl_sys::inline::mm::MiConvertSharedWorkingSetLockToExclusive(thread, vm)
}

// --- PFN lock (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiAcquirePfnLock() -> ntkrnl_sys::base::KIRQL {
    ntkrnl_sys::mm::MiAcquirePfnLock()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiReleasePfnLock(old_irql: ntkrnl_sys::base::KIRQL) {
    ntkrnl_sys::mm::MiReleasePfnLock(old_irql)
}

// --- Expansion lock (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiAcquireExpansionLock() -> ntkrnl_sys::base::KIRQL {
    ntkrnl_sys::inline::mm::MiAcquireExpansionLock()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiReleaseExpansionLock(old_irql: ntkrnl_sys::base::KIRQL) {
    ntkrnl_sys::inline::mm::MiReleaseExpansionLock(old_irql)
}

// --- User PTE/PDE checks (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsUserPte(address: PVOID) -> BOOLEAN {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::mm::MiIsUserPte(address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsUserPde(address: PVOID) -> BOOLEAN {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::mm::MiIsUserPde(address)
}

// --- PTE creation (was NTAPI, now cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MI_MAKE_HARDWARE_PTE_KERNEL(
    new_pte: PMMPTE,
    mapping_pte: PMMPTE,
    protection_mask: ntkrnl_sys::base::ULONG_PTR,
    page_frame_number: ntkrnl_sys::base::ULONG_PTR,
) {
    let new_pte = opaque_ptr(new_pte);
    let mapping_pte = opaque_ptr(mapping_pte);
    ntkrnl_sys::inline::mm::MI_MAKE_HARDWARE_PTE_KERNEL(
        new_pte, mapping_pte, protection_mask, page_frame_number,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MI_MAKE_HARDWARE_PTE_USER(
    new_pte: PMMPTE,
    mapping_pte: PMMPTE,
    protection_mask: ntkrnl_sys::base::ULONG_PTR,
    page_frame_number: ntkrnl_sys::base::ULONG_PTR,
) {
    let new_pte = opaque_ptr(new_pte);
    let mapping_pte = opaque_ptr(mapping_pte);
    ntkrnl_sys::inline::mm::MI_MAKE_HARDWARE_PTE_USER(
        new_pte, mapping_pte, protection_mask, page_frame_number,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MI_MAKE_HARDWARE_PTE(
    new_pte: PMMPTE,
    mapping_pte: PMMPTE,
    protection_mask: ntkrnl_sys::base::ULONG_PTR,
    page_frame_number: ntkrnl_sys::base::ULONG_PTR,
) {
    let new_pte = opaque_ptr(new_pte);
    let mapping_pte = opaque_ptr(mapping_pte);
    ntkrnl_sys::inline::mm::MI_MAKE_HARDWARE_PTE(
        new_pte, mapping_pte, protection_mask, page_frame_number,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MI_UPDATE_VALID_PTE(pte: PMMPTE, temp_pte: _MMPTE) {
    let pte = opaque_ptr(pte);
    ntkrnl_sys::inline::mm::MI_UPDATE_VALID_PTE(pte, temp_pte)
}

// --- User/global PTE mask (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiDetermineUserGlobalPteMask(
    pointer_pte: PMMPTE,
) -> ntkrnl_sys::base::ULONG_PTR {
    let pointer_pte = opaque_ptr(pointer_pte);
    ntkrnl_sys::inline::mm::MiDetermineUserGlobalPteMask(pointer_pte)
}

// --- Macro wrappers (was NTAPI, now cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RMAP_IS_SEGMENT(address: PVOID) -> BOOLEAN {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::mm::RMAP_IS_SEGMENT(address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PAGE_ROUND_DOWN(address: PVOID) -> ntkrnl_sys::base::ULONG_PTR {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::mm::PAGE_ROUND_DOWN(address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PFN_FROM_SSE(
    entry: ntkrnl_sys::base::ULONG_PTR,
) -> ntkrnl_sys::base::ULONG_PTR {
    ntkrnl_sys::inline::mm::PFN_FROM_SSE(entry)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MM_IS_WAIT_PTE(entry: ntkrnl_sys::base::ULONG_PTR) -> BOOLEAN {
    ntkrnl_sys::inline::mm::MM_IS_WAIT_PTE(entry)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IS_SWAP_FROM_SSE(entry: ntkrnl_sys::base::ULONG_PTR) -> BOOLEAN {
    ntkrnl_sys::inline::mm::IS_SWAP_FROM_SSE(entry)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SWAPENTRY_FROM_SSE(
    entry: ntkrnl_sys::base::ULONG_PTR,
) -> ntkrnl_sys::base::ULONG_PTR {
    ntkrnl_sys::inline::mm::SWAPENTRY_FROM_SSE(entry)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MmSetDirtyPage(process: PEPROCESS, address: PVOID) {
    let process = opaque_ptr(process);
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::mm::MmSetDirtyPage(process, address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InterlockedIncrementUL(addend: *mut ULONG) -> ULONG {
    let addend = opaque_ptr(addend);
    ntkrnl_sys::inline::mm::InterlockedIncrementUL(addend)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InterlockedExchangeAddUL(
    addend: *mut ULONG,
    value: ULONG,
) -> ULONG {
    let addend = opaque_ptr(addend);
    ntkrnl_sys::inline::mm::InterlockedExchangeAddUL(addend, value)
}

// --- Pool and segment helpers (was NTAPI, now cdecl) ---

// ExAllocatePoolZero (cdecl) folded into mm/util_inlines.rs alongside the
// stdcall variant 2026-05-09 — see comment there for the #[export_name]
// trick that lets both decorations coexist within ntoskrnl-rust.

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MmDereferenceSegment(segment: PMM_SECTION_SEGMENT) {
    ntkrnl_sys::inline::mm::MmDereferenceSegment(segment)
}

// --- Address space helpers (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MmGetAddressSpaceOwner(address_space: PMMSUPPORT) -> PEPROCESS {
    ntkrnl_sys::inline::mm::MmGetAddressSpaceOwner(address_space)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MmGetKernelAddressSpace() -> PMMSUPPORT {
    ntkrnl_sys::inline::mm::MmGetKernelAddressSpace()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MmLockAddressSpace(address_space: PMMSUPPORT) {
    ntkrnl_sys::inline::mm::MmLockAddressSpace(address_space)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MmUnlockAddressSpace(address_space: PMMSUPPORT) {
    ntkrnl_sys::inline::mm::MmUnlockAddressSpace(address_space)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MmGetCurrentAddressSpace() -> PMMSUPPORT {
    ntkrnl_sys::inline::mm::MmGetCurrentAddressSpace()
}

// --- Section / page helpers (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsRosSectionObject(section: PSECTION) -> BOOLEAN {
    ntkrnl_sys::inline::mm::MiIsRosSectionObject(section)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MmInitializeMdl(mdl: PMDL, base_va: PVOID, length: ULONG) {
    let mdl = opaque_ptr(mdl);
    let base_va = opaque_ptr(base_va);
    ntkrnl_sys::inline::mm::MmInitializeMdl(mdl, base_va, length)
}

// --- PFN reference counting (cdecl) ---

/// MiIncrementPageTableReferences — uses OriginalPte.u.Soft.UsedPageTableEntries
/// from OriginalPte, matching the C FORCEINLINE in miarm.h.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIncrementPageTableReferences(address: PVOID) -> u16 {
    let address = opaque_ptr(address);
    #[cfg(target_arch = "x86")]
    {
        // i386 (2-level paging): MmWorkingSetList->UsedPageTableEntries[pde_offset]
        ntkrnl_sys::inline::mm::MiIncrementPageTableReferences(address)
    }
    #[cfg(not(target_arch = "x86"))]
    {
        use crate::ARM3::helpers::{mmpfn_used_page_table_entries, mmpfn_set_used_page_table_entries};
        let pde = ntkrnl_sys::mm::MiAddressToPde(address);
        let pfn = ntkrnl_sys::mm::MiGetPfnEntry(ntkrnl_sys::mm::PFN_FROM_PTE(pde));
        let val = mmpfn_used_page_table_entries(pfn);
        let new_val = val + 1;
        mmpfn_set_used_page_table_entries(pfn, new_val);
        ASSERT!(new_val <= ntkrnl_sys::mm::PTE_PER_PAGE as u32);
        new_val as u16
    }
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiDropLockCount(pfn: PMMPFN) {
    let pfn = opaque_ptr(pfn);
    ntkrnl_sys::inline::mm::MiDropLockCount(pfn)
}

// MiDereferencePfnAndDropLockCount — already exported from ARM3/helpers.rs

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiReferenceProbedPageAndBumpLockCount(pfn: PMMPFN) {
    let pfn = opaque_ptr(pfn);
    ntkrnl_sys::inline::mm::MiReferenceProbedPageAndBumpLockCount(pfn)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiReferenceUnusedPageAndBumpLockCount(pfn: PMMPFN) {
    let pfn = opaque_ptr(pfn);
    ntkrnl_sys::inline::mm::MiReferenceUnusedPageAndBumpLockCount(pfn)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiReferenceUsedPageAndBumpLockCount(pfn: PMMPFN) {
    let pfn = opaque_ptr(pfn);
    ntkrnl_sys::inline::mm::MiReferenceUsedPageAndBumpLockCount(pfn)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiRemoveZeroPageSafe(color: ULONG) -> ntkrnl_sys::base::ULONG_PTR {
    ntkrnl_sys::inline::mm::MiRemoveZeroPageSafe(color)
}

// --- System PDE sync (i386 only) ---

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiSynchronizeSystemPde(pde: PMMPDE) -> BOOLEAN {
    ntkrnl_sys::inline::mm::MiSynchronizeSystemPde(pde)
}

// --- Process lock helpers (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireProcessLockRaiseToSynch(
    process: PKPROCESS,
    handle: PKLOCK_QUEUE_HANDLE,
) {
    let process = opaque_ptr(process);
    ntkrnl_sys::inline::mm::KiAcquireProcessLockRaiseToSynch(process, handle)
}

// KiReleaseProcessLock Rust shim REMOVED — the C shim in mm_rust_shims.c
// provides this symbol. Having both caused a duplicate symbol where the linker
// could pick the Rust version, whose tail-call codegen was broken on i386
// (crashed during early boot in KeStartThread → KeReleaseInStackQueuedSpinLock).

// --- Probe helpers (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForWriteSize_t(ptr: *mut ntkrnl_sys::base::ULONG_PTR) {
    let ptr = opaque_ptr(ptr);
    ntkrnl_sys::inline::mm::ProbeForWriteSize_t(ptr)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForWriteIoStatusBlock(ptr: *mut IO_STATUS_BLOCK) {
    let ptr = opaque_ptr(ptr);
    ntkrnl_sys::inline::mm::ProbeForWriteIoStatusBlock(ptr)
}

// --- UpdateTotalCommittedPages (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn UpdateTotalCommittedPages(delta: i32) {
    ntkrnl_sys::inline::mm::UpdateTotalCommittedPages(delta)
}

// --- KfRaiseIrql/KfLowerIrql (amd64 only, cdecl) ---

#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KfRaiseIrql(new_irql: ntkrnl_sys::base::KIRQL) -> ntkrnl_sys::base::KIRQL {
    ntkrnl_sys::inline::mm::KfRaiseIrql(new_irql)
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KfLowerIrql(new_irql: ntkrnl_sys::base::KIRQL) {
    ntkrnl_sys::inline::mm::KfLowerIrql(new_irql)
}

// ============================================================================
// KE subsystem exports — generic ke_x.h functions (was: ke/ke_rust_shims.c)
// ============================================================================

// --- UP no-op lock functions ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireThreadLock(thread: PKTHREAD) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiAcquireThreadLock(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleaseThreadLock(thread: PKTHREAD) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiReleaseThreadLock(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireDispatcherObject(object: *mut _DISPATCHER_HEADER) {
    let object = opaque_ptr(object);
    ntkrnl_sys::inline::ke::KiAcquireDispatcherObject(object)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleaseDispatcherObject(object: *mut _DISPATCHER_HEADER) {
    let object = opaque_ptr(object);
    ntkrnl_sys::inline::ke::KiReleaseDispatcherObject(object)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireDispatcherLockAtSynchLevel() {
    ntkrnl_sys::inline::ke::KiAcquireDispatcherLockAtSynchLevel()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleaseDispatcherLockFromSynchLevel() {
    ntkrnl_sys::inline::ke::KiReleaseDispatcherLockFromSynchLevel()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiRescheduleThread(new_thread: ntkrnl_sys::base::BOOLEAN, cpu: ntkrnl_sys::base::ULONG) {
    ntkrnl_sys::inline::ke::KiRescheduleThread(new_thread, cpu)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiSetThreadSwapBusy(thread: PKTHREAD) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiSetThreadSwapBusy(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquirePrcbLock(prcb: PKPRCB) {
    ntkrnl_sys::inline::ke::KiAcquirePrcbLock(prcb)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleasePrcbLock(prcb: PKPRCB) {
    ntkrnl_sys::inline::ke::KiReleasePrcbLock(prcb)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiTryThreadLock(thread: PKTHREAD) -> ntkrnl_sys::base::BOOLEAN {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiTryThreadLock(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiCheckDeferredReadyList(prcb: PKPRCB) {
    ntkrnl_sys::inline::ke::KiCheckDeferredReadyList(prcb)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiRequestApcInterrupt(need_apc: ntkrnl_sys::base::BOOLEAN, processor: ntkrnl_sys::base::UCHAR) {
    ntkrnl_sys::inline::ke::KiRequestApcInterrupt(need_apc, processor)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireTimerLock(hand: ntkrnl_sys::base::ULONG) -> PKSPIN_LOCK_QUEUE {
    ntkrnl_sys::inline::ke::KiAcquireTimerLock(hand)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleaseTimerLock(lock_queue: PKSPIN_LOCK_QUEUE) {
    ntkrnl_sys::inline::ke::KiReleaseTimerLock(lock_queue)
}

// --- Dispatcher lock ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireDispatcherLock() -> ntkrnl_sys::base::KIRQL {
    ntkrnl_sys::inline::ke::KiAcquireDispatcherLock()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleaseDispatcherLock(old_irql: ntkrnl_sys::base::KIRQL) {
    ntkrnl_sys::inline::ke::KiReleaseDispatcherLock(old_irql)
}

// --- Deferred ready ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiInsertDeferredReadyList(thread: PKTHREAD) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiInsertDeferredReadyList(thread)
}

// --- APC lock functions ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireApcLockRaiseToSynch(
    thread: PKTHREAD,
    handle: PKLOCK_QUEUE_HANDLE,
) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiAcquireApcLockRaiseToSynch(thread, handle)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireApcLockAtSynchLevel(
    thread: PKTHREAD,
    handle: PKLOCK_QUEUE_HANDLE,
) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiAcquireApcLockAtSynchLevel(thread, handle)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireApcLockRaiseToDpc(
    thread: PKTHREAD,
    handle: PKLOCK_QUEUE_HANDLE,
) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiAcquireApcLockRaiseToDpc(thread, handle)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleaseApcLock(handle: PKLOCK_QUEUE_HANDLE) {
    ntkrnl_sys::inline::ke::KiReleaseApcLock(handle)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleaseApcLockFromSynchLevel(handle: PKLOCK_QUEUE_HANDLE) {
    ntkrnl_sys::inline::ke::KiReleaseApcLockFromSynchLevel(handle)
}

// --- Process lock ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleaseProcessLockFromSynchLevel(handle: PKLOCK_QUEUE_HANDLE) {
    ntkrnl_sys::inline::ke::KiReleaseProcessLockFromSynchLevel(handle)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleaseProcessLock(handle: PKLOCK_QUEUE_HANDLE) {
    ntkrnl_sys::ddk::KeReleaseInStackQueuedSpinLock(handle)
}

// --- Device queue lock ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireDeviceQueueLock(
    device_queue: PKDEVICE_QUEUE,
    device_lock: PKLOCK_QUEUE_HANDLE,
) {
    ntkrnl_sys::inline::ke::KiAcquireDeviceQueueLock(device_queue, device_lock)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleaseDeviceQueueLock(device_lock: PKLOCK_QUEUE_HANDLE) {
    ntkrnl_sys::inline::ke::KiReleaseDeviceQueueLock(device_lock)
}

// --- NMI list lock ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireNmiListLock(old_irql: *mut ntkrnl_sys::base::KIRQL) {
    let old_irql = opaque_ptr(old_irql);
    ntkrnl_sys::inline::ke::KiAcquireNmiListLock(old_irql)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleaseNmiListLock(old_irql: ntkrnl_sys::base::KIRQL) {
    ntkrnl_sys::inline::ke::KiReleaseNmiListLock(old_irql)
}

// --- Timer/wait helpers ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiRecalculateDueTime(
    original_due_time: *mut _LARGE_INTEGER,
    due_time: *mut _LARGE_INTEGER,
    new_due_time: *mut _LARGE_INTEGER,
) -> *mut _LARGE_INTEGER {
    let original_due_time = opaque_ptr(original_due_time);
    let due_time = opaque_ptr(due_time);
    let new_due_time = opaque_ptr(new_due_time);
    ntkrnl_sys::inline::ke::KiRecalculateDueTime(original_due_time, due_time, new_due_time)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiCheckThreadStackSwap(
    thread: PKTHREAD,
    wait_mode: KPROCESSOR_MODE,
) -> ntkrnl_sys::base::BOOLEAN {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiCheckThreadStackSwap(thread, wait_mode)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiCheckAlertability(
    thread: PKTHREAD,
    alertable: ntkrnl_sys::base::BOOLEAN,
    wait_mode: KPROCESSOR_MODE,
) -> ntkrnl_sys::base::NTSTATUS {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiCheckAlertability(thread, alertable, wait_mode)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiComputeTimerTableIndex(
    due_time: u64,
) -> ntkrnl_sys::base::ULONG {
    ntkrnl_sys::inline::ke::KiComputeTimerTableIndex(due_time)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiRemoveEntryTimer(timer: PKTIMER) {
    let timer = opaque_ptr(timer);
    ntkrnl_sys::inline::ke::KiRemoveEntryTimer(timer)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxInsertTimer(timer: PKTIMER, hand: ntkrnl_sys::base::ULONG) {
    let timer = opaque_ptr(timer);
    ntkrnl_sys::inline::ke::KxInsertTimer(timer, hand)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiComputeDueTime(
    timer: PKTIMER,
    due_time: _LARGE_INTEGER,
    hand: *mut ntkrnl_sys::base::ULONG,
) -> ntkrnl_sys::base::BOOLEAN {
    let timer = opaque_ptr(timer);
    let hand = opaque_ptr(hand);
    ntkrnl_sys::inline::ke::KiComputeDueTime(timer, due_time, hand)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxRemoveTreeTimer(timer: PKTIMER) {
    let timer = opaque_ptr(timer);
    ntkrnl_sys::inline::ke::KxRemoveTreeTimer(timer)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxSetTimerForThreadWait(
    timer: PKTIMER,
    interval: _LARGE_INTEGER,
    hand: *mut ntkrnl_sys::base::ULONG,
) {
    let timer = opaque_ptr(timer);
    let hand = opaque_ptr(hand);
    ntkrnl_sys::inline::ke::KxSetTimerForThreadWait(timer, interval, hand)
}

// --- Unwait functions ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxUnwaitThread(
    object: *mut _DISPATCHER_HEADER,
    increment: KPRIORITY,
) {
    let object = opaque_ptr(object);
    ntkrnl_sys::inline::ke::KxUnwaitThread(object, increment)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxUnwaitThread_impl(
    header: *mut _DISPATCHER_HEADER,
    increment: KPRIORITY,
) {
    let header = opaque_ptr(header);
    ntkrnl_sys::inline::ke::KxUnwaitThread(header, increment)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxUnwaitThreadForEvent(
    event: PKEVENT,
    increment: KPRIORITY,
) {
    let event = opaque_ptr(event);
    ntkrnl_sys::inline::ke::KxUnwaitThreadForEvent(event, increment)
}

// --- Ready queue ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxQueueReadyThread(thread: PKTHREAD, prcb: PKPRCB) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KxQueueReadyThread(thread, prcb)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiSelectReadyThread(
    priority: KPRIORITY,
    prcb: PKPRCB,
) -> PKTHREAD {
    ntkrnl_sys::inline::ke::KiSelectReadyThread(priority, prcb)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiComputeNewPriority(thread: PKTHREAD, adjustment: i8) -> i8 {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiComputeNewPriority(thread, adjustment)
}

// --- Guarded mutex ---
// NOTE: KeInitializeGuardedMutex, KeAcquireGuardedMutex, KeReleaseGuardedMutex,
// KeAcquireGuardedMutexUnsafe, KeReleaseGuardedMutexUnsafe, KeTryToAcquireGuardedMutex
// are exported by ke/gmutex.rs (with correct i386 fastcall calling convention).

// --- Macro wrappers ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiSatisfyMutantWait(object: *mut _KMUTANT, thread: PKTHREAD) {
    let object = opaque_ptr(object);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiSatisfyMutantWait(object, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiSatisfyNonMutantWait(object: *mut _KMUTANT) {
    let object = opaque_ptr(object);
    ntkrnl_sys::inline::ke::KiSatisfyNonMutantWait(object)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiSatisfyObjectWait(object: *mut _KMUTANT, thread: PKTHREAD) {
    let object = opaque_ptr(object);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiSatisfyObjectWait(object, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAddThreadToWaitList(
    thread: PKTHREAD,
    swappable: ntkrnl_sys::base::BOOLEAN,
) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiAddThreadToWaitList(thread, swappable)
}

// --- Statement macro wrappers ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxSingleThreadWait(
    object: ntkrnl_sys::base::PVOID,
    timeout: *mut _LARGE_INTEGER,
    wait_mode: KPROCESSOR_MODE,
    wait_reason: _KWAIT_REASON,
    alertable: ntkrnl_sys::base::BOOLEAN,
    swappable: *mut ntkrnl_sys::base::BOOLEAN,
    due_time: *mut _LARGE_INTEGER,
    hand: *mut ntkrnl_sys::base::ULONG,
) {
    let object = opaque_ptr(object);
    let timeout = opaque_ptr(timeout);
    let swappable = opaque_ptr(swappable);
    let due_time = opaque_ptr(due_time);
    let hand = opaque_ptr(hand);
    ntkrnl_sys::inline::ke::KxSingleThreadWait(
        object, timeout, wait_mode, wait_reason, alertable, swappable, due_time, hand,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxDelayThreadWait(
    interval: *mut _LARGE_INTEGER,
    wait_mode: KPROCESSOR_MODE,
    alertable: ntkrnl_sys::base::BOOLEAN,
    swappable: *mut ntkrnl_sys::base::BOOLEAN,
    due_time: *mut _LARGE_INTEGER,
    hand: *mut ntkrnl_sys::base::ULONG,
) {
    let interval = opaque_ptr(interval);
    let swappable = opaque_ptr(swappable);
    let due_time = opaque_ptr(due_time);
    let hand = opaque_ptr(hand);
    ntkrnl_sys::inline::ke::KxDelayThreadWait(interval, wait_mode, alertable, swappable, due_time, hand)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxMultiThreadWait(
    wait_block_array: *mut _KWAIT_BLOCK,
    objects: *mut ntkrnl_sys::base::PVOID,
    count: ntkrnl_sys::base::ULONG,
    timeout: *mut _LARGE_INTEGER,
    wait_type: _WAIT_TYPE,
    wait_mode: KPROCESSOR_MODE,
    wait_reason: _KWAIT_REASON,
    alertable: ntkrnl_sys::base::BOOLEAN,
    swappable: *mut ntkrnl_sys::base::BOOLEAN,
    due_time: *mut _LARGE_INTEGER,
    hand: *mut ntkrnl_sys::base::ULONG,
) {
    let wait_block_array = opaque_ptr(wait_block_array);
    let objects = opaque_ptr(objects);
    let timeout = opaque_ptr(timeout);
    let swappable = opaque_ptr(swappable);
    let due_time = opaque_ptr(due_time);
    let hand = opaque_ptr(hand);
    ntkrnl_sys::inline::ke::KxMultiThreadWait(
        wait_block_array, objects, count, timeout, wait_type, wait_mode, wait_reason,
        alertable, swappable, due_time, hand,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxQueueThreadWait(
    queue: ntkrnl_sys::base::PVOID,
    timeout: *mut _LARGE_INTEGER,
    wait_mode: KPROCESSOR_MODE,
    swappable: *mut ntkrnl_sys::base::BOOLEAN,
    due_time: *mut _LARGE_INTEGER,
    hand: *mut ntkrnl_sys::base::ULONG,
) {
    let queue = opaque_ptr(queue);
    let timeout = opaque_ptr(timeout);
    let swappable = opaque_ptr(swappable);
    let due_time = opaque_ptr(due_time);
    let hand = opaque_ptr(hand);
    ntkrnl_sys::inline::ke::KxQueueThreadWait(
        queue, timeout, wait_mode, swappable, due_time, hand,
    )
}

// --- Utility functions ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn YieldProcessor() {
    ntkrnl_sys::inline::ke::YieldProcessor()
}

// KeGetTrapFramePc moved to ke-macros (2026-05-08 — promoted from the
// _rs_* trampoline pattern to a bare-name `extern "system"` export).

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiRundownThread(thread: PKTHREAD) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ke::KiRundownThread(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiGetLinkedTrapFrame(trap_frame: PKTRAP_FRAME) -> PKTRAP_FRAME {
    ntkrnl_sys::inline::ke::KiGetLinkedTrapFrame(trap_frame)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiGetUserModeStackAddress() -> *mut ntkrnl_sys::base::ULONG_PTR {
    ntkrnl_sys::inline::ke::KiGetUserModeStackAddress()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InterlockedBitTestAndSet(base: *mut i32, bit: i32) -> ntkrnl_sys::base::BOOLEAN {
    let base = opaque_ptr(base);
    ntkrnl_sys::inline::ke::InterlockedBitTestAndSet(base, bit)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InterlockedBitTestAndReset(base: *mut i32, bit: i32) -> ntkrnl_sys::base::BOOLEAN {
    let base = opaque_ptr(base);
    ntkrnl_sys::inline::ke::InterlockedBitTestAndReset(base, bit)
}

// ============================================================================
// KE subsystem exports — amd64 intrinsics (was: ke/ke_rust_shims.c amd64 section)
// ============================================================================

#[cfg(target_arch = "x86_64")]
mod ke_amd64_exports {
    use ntkrnl_sys::ddk::*;
    use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

    #[no_mangle]
    pub unsafe extern "C" fn __readcr0() -> ntkrnl_sys::base::ULONG_PTR {
        ntkrnl_sys::inline::ke_amd64::__readcr0()
    }

    #[no_mangle]
    pub unsafe extern "C" fn __readcr2() -> ntkrnl_sys::base::ULONG_PTR {
        ntkrnl_sys::inline::ke_amd64::__readcr2()
    }

    #[no_mangle]
    pub unsafe extern "C" fn __readcr3() -> ntkrnl_sys::base::ULONG_PTR {
        ntkrnl_sys::inline::ke_amd64::__readcr3()
    }

    #[no_mangle]
    pub unsafe extern "C" fn __readcr4() -> ntkrnl_sys::base::ULONG_PTR {
        ntkrnl_sys::inline::ke_amd64::__readcr4()
    }

    #[no_mangle]
    pub unsafe extern "C" fn __readcr8() -> ntkrnl_sys::base::ULONG_PTR {
        ntkrnl_sys::inline::ke_amd64::__readcr8()
    }

    #[no_mangle]
    pub unsafe extern "C" fn __writecr0(val: ntkrnl_sys::base::ULONG_PTR) {
        ntkrnl_sys::inline::ke_amd64::__writecr0(val)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __writecr3(val: ntkrnl_sys::base::ULONG_PTR) {
        ntkrnl_sys::inline::ke_amd64::__writecr3(val)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __writecr4(val: ntkrnl_sys::base::ULONG_PTR) {
        ntkrnl_sys::inline::ke_amd64::__writecr4(val)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __writecr8(val: ntkrnl_sys::base::ULONG_PTR) {
        ntkrnl_sys::inline::ke_amd64::__writecr8(val)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __readdr(reg: ntkrnl_sys::base::ULONG) -> ntkrnl_sys::base::ULONG_PTR {
        ntkrnl_sys::inline::ke_amd64::__readdr(reg)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __writedr(reg: ntkrnl_sys::base::ULONG, val: ntkrnl_sys::base::ULONG_PTR) {
        ntkrnl_sys::inline::ke_amd64::__writedr(reg, val)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __readmsr(reg: ntkrnl_sys::base::ULONG) -> u64 {
        ntkrnl_sys::inline::ke_amd64::__readmsr(reg)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __writemsr(reg: ntkrnl_sys::base::ULONG, val: u64) {
        ntkrnl_sys::inline::ke_amd64::__writemsr(reg, val)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __lidt(source: ntkrnl_sys::base::PVOID) {
    let source = opaque_ptr(source);
        ntkrnl_sys::inline::ke_amd64::__lidt(source)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __lgdt(source: ntkrnl_sys::base::PVOID) {
    let source = opaque_ptr(source);
        ntkrnl_sys::inline::ke_amd64::__lgdt(source)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __sidt(dest: ntkrnl_sys::base::PVOID) {
    let dest = opaque_ptr(dest);
        ntkrnl_sys::inline::ke_amd64::__sidt(dest)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __sgdt(dest: ntkrnl_sys::base::PVOID) {
    let dest = opaque_ptr(dest);
        ntkrnl_sys::inline::ke_amd64::__sgdt(dest)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __sldt(ldtr: *mut u16) {
    let ldtr = opaque_ptr(ldtr);
        ntkrnl_sys::inline::ke_amd64::__sldt(ldtr)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __str(tr: *mut u16) {
    let tr = opaque_ptr(tr);
        ntkrnl_sys::inline::ke_amd64::__str(tr)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __wbinvd() {
        ntkrnl_sys::inline::ke_amd64::__wbinvd()
    }

    #[no_mangle]
    pub unsafe extern "C" fn __swapgs() {
        ntkrnl_sys::inline::ke_amd64::__swapgs()
    }

    #[no_mangle]
    pub unsafe extern "C" fn KeMemoryBarrier() {
        ntkrnl_sys::inline::ke_amd64::KeMemoryBarrier()
    }

    #[no_mangle]
    pub unsafe extern "C" fn KeQueryInterruptHandler(vector: ntkrnl_sys::base::ULONG) -> ntkrnl_sys::base::PVOID {
        ntkrnl_sys::inline::ke_amd64::KeQueryInterruptHandler(vector)
    }

    #[no_mangle]
    pub unsafe extern "C" fn KeRegisterInterruptHandler(vector: ntkrnl_sys::base::ULONG, handler: ntkrnl_sys::base::PVOID) {
    let handler = opaque_ptr(handler);
        ntkrnl_sys::inline::ke_amd64::KeRegisterInterruptHandler(vector, handler)
    }

    #[no_mangle]
    pub unsafe extern "C" fn KiGetGdtEntry(gdt: ntkrnl_sys::base::PVOID, selector: u16) -> *mut _KGDTENTRY64 {
    let gdt = opaque_ptr(gdt);
        ntkrnl_sys::inline::ke_amd64::KiGetGdtEntry(gdt, selector)
    }

    #[no_mangle]
    pub unsafe extern "C" fn KiGetGdtDescriptorBase(entry: *mut _KGDTENTRY64) -> ntkrnl_sys::base::PVOID {
    let entry = opaque_ptr(entry);
        ntkrnl_sys::inline::ke_amd64::KiGetGdtDescriptorBase(entry)
    }

    #[no_mangle]
    pub unsafe extern "C" fn KiInitGdtEntry(
        entry: *mut _KGDTENTRY64,
        base: u64,
        size: ntkrnl_sys::base::ULONG,
        type_: u8,
        dpl: u8,
    ) {
    let entry = opaque_ptr(entry);
        ntkrnl_sys::inline::ke_amd64::KiInitGdtEntry(entry, base, size, type_, dpl)
    }

    #[no_mangle]
    pub unsafe extern "C" fn KiSetGdtDescriptorBase(entry: *mut _KGDTENTRY64, base: u64) {
    let entry = opaque_ptr(entry);
        ntkrnl_sys::inline::ke_amd64::KiSetGdtDescriptorBase(entry, base)
    }

    #[no_mangle]
    pub unsafe extern "C" fn KiSetGdtDescriptorLimit(entry: *mut _KGDTENTRY64, limit: ntkrnl_sys::base::ULONG) {
    let entry = opaque_ptr(entry);
        ntkrnl_sys::inline::ke_amd64::KiSetGdtDescriptorLimit(entry, limit)
    }

    // KiSaveXState / KiRestoreXState — provided by ke_rust_shims.c (C version)
    // which properly checks KeFeatureBits and uses xsaves64/xsaveopt64/xsave64.
    // The Rust inline version in ntkrnl-sys is simplified and incorrect for KVM
    // (doesn't mask XSTATE_MASK_LEGACY_SSE, doesn't check KF_XSAVES/KF_XSAVEOPT).

    #[no_mangle]
    pub unsafe extern "C" fn KiSendEOI() {
        ntkrnl_sys::inline::ke_amd64::KiSendEOI()
    }

    // RtlFillMemoryUlong: the `mem` crate (rtl/mem.c) is its one amd64 provider. A
    // `rep stosd` copy here used to shadow it via --allow-multiple-definition.
}

// ============================================================================
// KE subsystem exports — i386 KeMemoryBarrier (ke_i386.rs inline fn)
// ============================================================================

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeMemoryBarrier() {
    ntkrnl_sys::inline::ke_i386::KeMemoryBarrier()
}

// ============================================================================
// KE subsystem exports — i386 intrinsics + GDT helpers
// ============================================================================

#[cfg(target_arch = "x86")]
mod ke_i386_exports {
    use ntkrnl_sys::ddk::*;

    use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
    #[no_mangle]
    #[inline(never)]
    pub unsafe extern "C" fn KiGetGdtEntry(
        gdt: ntkrnl_sys::base::PVOID,
        selector: u16,
    ) -> *mut _KGDTENTRY {
    let gdt = opaque_ptr(gdt);
        ntkrnl_sys::inline::ke_i386::KiGetGdtEntry(gdt, selector)
    }

    #[no_mangle]
    #[inline(never)]
    pub unsafe extern "C" fn KiSetGdtDescriptorBase(
        entry: *mut _KGDTENTRY,
        base: ntkrnl_sys::base::ULONG_PTR,
    ) {
    let entry = opaque_ptr(entry);
        ntkrnl_sys::inline::ke_i386::KiSetGdtDescriptorBase(entry, base)
    }

    // RtlFillMemoryUlong is sdk/lib/rtl/i386/rtlmem.s's (`_RtlFillMemoryUlong@12`,
    // the same `rep stosd`), which the kernel links with the rest of the Rtl
    // memory family. A copy here was a duplicate symbol kept by link order.
}

// ============================================================================
// ARM64_UART_TRACE / ARM64_UART_TRACE_HEX
//
// Direct UART pokes used at bringup-path callsites where DbgPrint isn't wired
// up yet (PsCreateProcess, config init, etc.). Real implementation on ARM64;
// no-op stub on i386/amd64 so cross-arch Rust callers (cminit.rs, kill.rs,
// process.rs, psmgr.rs, …) can declare them unconditionally.
//
// Was: ntoskrnl/arm64_uart.c (now retired).
// ============================================================================

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ARM64_UART_TRACE(msg: *const u8) {
    #[cfg(target_arch = "aarch64")]
    {
        let u = core::ptr::read_volatile(
            core::ptr::addr_of!(crate::ke::arm64::stubs::Arm64UartKernelBase),
        ) as *mut u32;
        if u.is_null() { return; }
        let daif = crate::arm64_dbg::uart_line_lock();
        let mut p = msg;
        while *p != 0 {
            crate::arm64_dbg::uart_putc_default(*p as u8);
            p = p.add(1);
        }
        crate::arm64_dbg::uart_line_unlock(daif);
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = msg;
    }
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ARM64_UART_TRACE_HEX(prefix: *const u8, val: ntkrnl_sys::base::ULONG_PTR) {
    #[cfg(target_arch = "aarch64")]
    {
        const HEX: &[u8; 16] = b"0123456789ABCDEF";
        let u = core::ptr::read_volatile(
            core::ptr::addr_of!(crate::ke::arm64::stubs::Arm64UartKernelBase),
        ) as *mut u32;
        if u.is_null() { return; }
        let daif = crate::arm64_dbg::uart_line_lock();
        let mut p = prefix;
        while *p != 0 {
            crate::arm64_dbg::uart_putc_default(*p as u8);
            p = p.add(1);
        }
        crate::arm64_dbg::uart_putc_default(b'0' as u8);
        crate::arm64_dbg::uart_putc_default(b'x' as u8);
        let nibbles = (core::mem::size_of::<ntkrnl_sys::base::ULONG_PTR>() * 2) as i32;
        let mut i = nibbles - 1;
        while i >= 0 {
            let nib = ((val >> (i * 4)) & 0xF) as usize;
            crate::arm64_dbg::uart_putc_default(HEX[nib] as u8);
            i -= 1;
        }
        crate::arm64_dbg::uart_line_unlock(daif);
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = prefix;
        let _ = val;
    }
}

// ============================================================================
// StopChecking global (was in C traphdlr.c, referenced by ke_rust_shims.c)
// ============================================================================

#[cfg(target_arch = "x86")]
#[no_mangle]
pub static mut StopChecking: ntkrnl_sys::base::BOOLEAN = 0;

// ============================================================================
// i386 stdcall aliases for 0-arg functions that also have cdecl callers.
// On i386, stdcall and cdecl for 0-arg functions are ABI-identical (both
// use `ret`), so we can safely alias the stdcall decorated name to the
// cdecl implementation.
// ============================================================================

#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    ".intel_syntax noprefix",

    // KeEnterCriticalRegion: now extern "system" = stdcall = _KeEnterCriticalRegion@0 (primary)
    ".globl _KeEnterCriticalRegion",
    ".set _KeEnterCriticalRegion, _KeEnterCriticalRegion@0",

    // KeLeaveCriticalRegion: now extern "system" = stdcall = _KeLeaveCriticalRegion@0 (primary)
    ".globl _KeLeaveCriticalRegion",
    ".set _KeLeaveCriticalRegion, _KeLeaveCriticalRegion@0",

    // PsGetCurrentThread alias moved to common-inlines crate (target symbol
    // _PsGetCurrentThread@0 is now defined there, not in this rlib).

    // KeEnterGuardedRegion: stdcall = _KeEnterGuardedRegion@0 (definition), cdecl = _KeEnterGuardedRegion (alias)
    ".globl _KeEnterGuardedRegion",
    ".set _KeEnterGuardedRegion, _KeEnterGuardedRegion@0",

    // KeLeaveGuardedRegion: stdcall = _KeLeaveGuardedRegion@0 (definition), cdecl = _KeLeaveGuardedRegion (alias)
    ".globl _KeLeaveGuardedRegion",
    ".set _KeLeaveGuardedRegion, _KeLeaveGuardedRegion@0",

    // IoGetCurrentProcess: now extern "system" = stdcall = _IoGetCurrentProcess@0 (primary)
    // Alias cdecl name for any C code that uses cdecl calling convention
    ".globl _IoGetCurrentProcess",
    ".set _IoGetCurrentProcess, _IoGetCurrentProcess@0",
);

// ============================================================================
// i386 Queued Spinlock functions — replaces HAL import with local Rust impl.
//
// On i386, KeAcquireQueuedSpinLock et al. are exported by HAL as FASTCALL.
// We provide our own implementation here to avoid HAL import issues.
// These MUST be in shim_exports.rs (not a separate file) so the linker
// always includes them — archive members in libkernel_rust.a are only
// pulled in if needed, and HAL imports would otherwise satisfy the symbols.
// amd64 version is in ke/amd64/spinlock.rs.
// ============================================================================

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn KeAcquireQueuedSpinLock(
    LockNumber: ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE_NUMBER,
) -> KIRQL {
    let mut old_irql: KIRQL = 0;
    ntkrnl_sys::inline::tier1::KeRaiseIrql(ntkrnl_sys::base::DISPATCH_LEVEL, &mut old_irql);
    let prcb = ntkrnl_sys::inline::ke::KeGetCurrentPrcb();
    ntkrnl_sys::inline::ke::KxAcquireSpinLock(
        (*prcb).LockQueue[LockNumber as usize].Lock as *mut _,
    );
    old_irql
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn KeAcquireQueuedSpinLockRaiseToSynch(
    LockNumber: ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE_NUMBER,
) -> KIRQL {
    let mut old_irql: KIRQL = 0;
    ntkrnl_sys::inline::tier1::KeRaiseIrql(ntkrnl_sys::inline::ke::SYNCH_LEVEL, &mut old_irql);
    let prcb = ntkrnl_sys::inline::ke::KeGetCurrentPrcb();
    ntkrnl_sys::inline::ke::KxAcquireSpinLock(
        (*prcb).LockQueue[LockNumber as usize].Lock as *mut _,
    );
    old_irql
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn KeReleaseQueuedSpinLock(
    LockNumber: ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE_NUMBER,
    OldIrql: KIRQL,
) {
    let prcb = ntkrnl_sys::inline::ke::KeGetCurrentPrcb();
    ntkrnl_sys::inline::ke::KxReleaseSpinLock(
        (*prcb).LockQueue[LockNumber as usize].Lock as *mut _,
    );
    ntkrnl_sys::inline::tier1::KeLowerIrql(OldIrql);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn KeAcquireInStackQueuedSpinLock(
    SpinLock: PKSPIN_LOCK,
    LockHandle: PKLOCK_QUEUE_HANDLE,
) {
    (*LockHandle).LockQueue.Next = core::ptr::null_mut();
    (*LockHandle).LockQueue.Lock = SpinLock as *mut u32;
    ntkrnl_sys::inline::tier1::KeRaiseIrql(ntkrnl_sys::base::DISPATCH_LEVEL, &mut (*LockHandle).OldIrql);
    ntkrnl_sys::inline::ke::KxAcquireSpinLock((*LockHandle).LockQueue.Lock as *mut _);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn KeAcquireInStackQueuedSpinLockRaiseToSynch(
    SpinLock: PKSPIN_LOCK,
    LockHandle: PKLOCK_QUEUE_HANDLE,
) {
    (*LockHandle).LockQueue.Next = core::ptr::null_mut();
    (*LockHandle).LockQueue.Lock = SpinLock as *mut u32;
    ntkrnl_sys::inline::tier1::KeRaiseIrql(ntkrnl_sys::inline::ke::SYNCH_LEVEL, &mut (*LockHandle).OldIrql);
    ntkrnl_sys::inline::ke::KxAcquireSpinLock((*LockHandle).LockQueue.Lock as *mut _);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn KeReleaseInStackQueuedSpinLock(
    LockHandle: PKLOCK_QUEUE_HANDLE,
) {
    ntkrnl_sys::inline::ke::KxReleaseSpinLock((*LockHandle).LockQueue.Lock as *mut _);
    ntkrnl_sys::inline::tier1::KeLowerIrql((*LockHandle).OldIrql);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn KeTryToAcquireQueuedSpinLockRaiseToSynch(
    LockNumber: ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE_NUMBER,
    OldIrql: *mut KIRQL,
) -> BOOLEAN {
    let OldIrql = opaque_ptr(OldIrql);
    ntkrnl_sys::inline::tier1::KeRaiseIrql(ntkrnl_sys::inline::ke::SYNCH_LEVEL, OldIrql);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    1u8
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "fastcall" fn KeTryToAcquireQueuedSpinLock(
    LockNumber: ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE_NUMBER,
    OldIrql: *mut KIRQL,
) -> u32 {
    let OldIrql = opaque_ptr(OldIrql);
    ntkrnl_sys::inline::tier1::KeRaiseIrql(ntkrnl_sys::base::DISPATCH_LEVEL, OldIrql);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    1u32
}

// ============================================================================
// IoGetCurrentProcess — macro in C (#define IoGetCurrentProcess PsGetCurrentProcess)
// Must be extern "system" (NTAPI = stdcall on i386) to match calling convention
// of ntkrnl-sys extern declarations. Cdecl/stdcall mismatch in same LLVM CGU = UB.
// ============================================================================

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn IoGetCurrentProcess() -> PEPROCESS {
    ntkrnl_sys::inline::tier1::PsGetCurrentProcess()
}
