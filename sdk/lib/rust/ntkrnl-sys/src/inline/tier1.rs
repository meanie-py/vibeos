//! Tier 1 subsystem inline functions.
//!
//! IO IRP stack macros, LPC inlines/macros, HAL dispatch table macros,
//! IRQL wrappers (i386/amd64), SeDeleteClientSecurity, etc.

use crate::base::*;
use crate::constants::*;
use crate::ddk;

// ============================================================================
// Extern declarations for globals and functions not in bindgen
// ============================================================================

extern "C" {
    static mut HalPrivateDispatchTable: ddk::_HAL_PRIVATE_DISPATCH;
    static mut HalDispatchTable: ddk::HAL_DISPATCH;

    // LPC globals
    static mut LpcpLock: crate::ke::KGUARDED_MUTEX;
    static mut LpcpMessagesLookaside: ddk::_PAGED_LOOKASIDE_LIST;
}

// i386 KfRaiseIrql/KfLowerIrql are FASTCALL from HAL
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
    fn KfLowerIrql(NewIrql: KIRQL);
}

// amd64/arm64: KfRaiseIrql/KfLowerIrql are regular C calling convention
#[cfg(target_arch = "x86_64")]
extern "C" {
    fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
}

// ============================================================================
// LPC constants (from ntoskrnl/include/internal/lpc.h)
// ============================================================================

const LPCP_THREAD_FLAG_IS_PORT: usize = 1;
const LPCP_THREAD_FLAG_NO_IMPERSONATION: usize = 2;
const LPCP_THREAD_FLAGS: usize = LPCP_THREAD_FLAG_IS_PORT | LPCP_THREAD_FLAG_NO_IMPERSONATION;

// STATUS_USER_APC — returned by KeWaitForSingleObject when preempted by APC
const STATUS_USER_APC: NTSTATUS = 0x000000C0_u32 as NTSTATUS;

// KWAIT_REASON values for LPC wait macros
const WR_LPC_RECEIVE: ddk::KWAIT_REASON = ddk::_KWAIT_REASON_WrLpcReceive;
const WR_LPC_REPLY: ddk::KWAIT_REASON = ddk::_KWAIT_REASON_WrLpcReply;
const EXECUTIVE: ddk::KWAIT_REASON = ddk::_KWAIT_REASON_Executive;
const WR_EXECUTIVE: ddk::KWAIT_REASON = ddk::_KWAIT_REASON_WrExecutive;

// IO stack location flag
const SL_PENDING_RETURNED: u8 = 0x01;

// NOTE (2026-07-09): the aarch64 "C-truth" shims that used to live here
// (CurrentStackLocation slot at IRP+0xb8, IO_STACK_LOCATION stride 0x44) are
// GONE. They compensated for the iotypes.h pack(4) guard missing _ARM64_
// (clang aarch64-w64-mingw32 ignores per-field aligned attrs under
// #pragma pack, so the arm64 C layout was i386-packed while bindgen produced
// what NT intends). The guard is fixed and bindings regenerated: C == bindgen
// on every arch, so plain bindgen field access / pointer arithmetic is right.
// Mixed-stride survivors of this shim family wedge boot at the first
// synchronous PnP IRP — if you see IofCompleteRequest panic on a None
// CompletionRoutine, hunt for a reintroduced copy of these.

#[inline]
unsafe fn irp_current_stack_location_slot(irp: ddk::PIRP) -> *mut ddk::PIO_STACK_LOCATION {
    core::ptr::addr_of_mut!(
        (*irp)
            .Tail
            .Overlay
            .__bindgen_anon_2
            .__bindgen_anon_1
            .CurrentStackLocation
    )
}

#[inline]
unsafe fn irp_stack_location_add(
    stack: ddk::PIO_STACK_LOCATION,
    count: isize,
) -> ddk::PIO_STACK_LOCATION {
    stack.offset(count)
}

// ============================================================================
// IO IRP stack location macros (from iofuncs.h)
// ============================================================================

/// IoGetCurrentIrpStackLocation — return current IO_STACK_LOCATION pointer.
/// C: `return Irp->Tail.Overlay.CurrentStackLocation;`
#[inline]
pub unsafe fn IoGetCurrentIrpStackLocation(irp: ddk::PIRP) -> ddk::PIO_STACK_LOCATION {
    *irp_current_stack_location_slot(irp)
}

/// IoGetNextIrpStackLocation — return next (lower) IO_STACK_LOCATION.
/// C: `return Irp->Tail.Overlay.CurrentStackLocation - 1;`
#[inline]
pub unsafe fn IoGetNextIrpStackLocation(irp: ddk::PIRP) -> ddk::PIO_STACK_LOCATION {
    irp_stack_location_add(IoGetCurrentIrpStackLocation(irp), -1)
}

/// IoSetNextIrpStackLocation — decrement CurrentLocation, advance stack pointer.
/// C: `Irp->CurrentLocation--; Irp->Tail.Overlay.CurrentStackLocation--;`
#[inline]
pub unsafe fn IoSetNextIrpStackLocation(irp: ddk::PIRP) {
    (*irp).CurrentLocation -= 1;
    let p = irp_current_stack_location_slot(irp);
    *p = irp_stack_location_add(*p, -1);
}

/// IoSkipCurrentIrpStackLocation — increment CurrentLocation, retreat stack pointer.
/// C: `Irp->CurrentLocation++; Irp->Tail.Overlay.CurrentStackLocation++;`
#[inline]
pub unsafe fn IoSkipCurrentIrpStackLocation(irp: ddk::PIRP) {
    (*irp).CurrentLocation += 1;
    let p = irp_current_stack_location_slot(irp);
    *p = irp_stack_location_add(*p, 1);
}

/// IoCopyCurrentIrpStackLocationToNext — copy current stack location to next.
/// C: copies bytes up to FIELD_OFFSET(IO_STACK_LOCATION, CompletionRoutine),
/// then zeroes nextIrpSp->Control.
#[inline]
pub unsafe fn IoCopyCurrentIrpStackLocationToNext(irp: ddk::PIRP) {
    let irp_sp = IoGetCurrentIrpStackLocation(irp);
    let next_irp_sp = IoGetNextIrpStackLocation(irp);
    let count = core::mem::offset_of!(ddk::IO_STACK_LOCATION, CompletionRoutine);
    core::ptr::copy_nonoverlapping(
        irp_sp as *const u8,
        next_irp_sp as *mut u8,
        count,
    );
    (*next_irp_sp).Control = 0;
}

/// IoSetCancelRoutine — atomically set IRP cancel routine, return old value.
/// C: `return InterlockedExchangePointer(&Irp->CancelRoutine, CancelRoutine);`
#[inline]
pub unsafe fn IoSetCancelRoutine(
    irp: ddk::PIRP,
    cancel_routine: ddk::PDRIVER_CANCEL,
) -> ddk::PDRIVER_CANCEL {
    use core::sync::atomic::{AtomicPtr, Ordering};
    // CancelRoutine is Option<fn ptr>. We treat it as a pointer-sized atomic.
    let ptr = core::ptr::addr_of_mut!((*irp).CancelRoutine) as *const AtomicPtr<()>;
    let new_val = match cancel_routine {
        Some(f) => f as *mut (),
        None => core::ptr::null_mut(),
    };
    let old = (*ptr).swap(new_val, Ordering::SeqCst);
    if old.is_null() {
        None
    } else {
        Some(core::mem::transmute(old))
    }
}

/// IoMarkIrpPending — set SL_PENDING_RETURNED on current stack location.
/// C: `IoGetCurrentIrpStackLocation(Irp)->Control |= SL_PENDING_RETURNED;`
#[inline]
pub unsafe fn IoMarkIrpPending(irp: ddk::PIRP) {
    let sp = IoGetCurrentIrpStackLocation(irp);
    (*sp).Control |= SL_PENDING_RETURNED;
}

/// IoSetCompletionRoutine — set completion routine on next stack location.
#[inline]
pub unsafe fn IoSetCompletionRoutine(
    irp: ddk::PIRP,
    completion_routine: ddk::PIO_COMPLETION_ROUTINE,
    context: PVOID,
    invoke_on_success: BOOLEAN,
    invoke_on_error: BOOLEAN,
    invoke_on_cancel: BOOLEAN,
) {
    let sp = IoGetNextIrpStackLocation(irp);
    (*sp).CompletionRoutine = completion_routine;
    (*sp).Context = context;
    (*sp).Control = 0;
    if invoke_on_success != 0 {
        (*sp).Control |= 0x40; // SL_INVOKE_ON_SUCCESS
    }
    if invoke_on_error != 0 {
        (*sp).Control |= 0x80; // SL_INVOKE_ON_ERROR
    }
    if invoke_on_cancel != 0 {
        (*sp).Control |= 0x20; // SL_INVOKE_ON_CANCEL
    }
}

// ============================================================================
// LPC FORCEINLINE functions (from ntoskrnl/include/internal/lpc_x.h)
// ============================================================================

/// LpcpAllocateFromPortZone — allocate a message from the LPC lookaside list.
/// Acquires LpcpLock (guarded mutex), allocates, initializes, releases lock.
#[inline]
pub unsafe fn LpcpAllocateFromPortZone() -> ddk::PLPCP_MESSAGE {
    // Acquire LPC lock
    crate::ke::KeAcquireGuardedMutex(
        core::ptr::addr_of_mut!(LpcpLock) as crate::ke::PKGUARDED_MUTEX,
    );

    // Allocate from paged lookaside list
    let entry = super::ex::ExAllocateFromPagedLookasideList(
        core::ptr::addr_of_mut!(LpcpMessagesLookaside),
    );

    if entry.is_null() {
        crate::ke::KeReleaseGuardedMutex(
            core::ptr::addr_of_mut!(LpcpLock) as crate::ke::PKGUARDED_MUTEX,
        );
        return core::ptr::null_mut();
    }

    let message = entry as ddk::PLPCP_MESSAGE;

    // InitializeListHead(&Message->Entry)
    let list_head = core::ptr::addr_of_mut!((*message).__bindgen_anon_1.Entry)
        as *mut ddk::_LIST_ENTRY;
    (*list_head).Flink = list_head;
    (*list_head).Blink = list_head;

    // Message->RepliedToThread = NULL
    (*message).RepliedToThread = core::ptr::null_mut();

    // Message->Request.u2.ZeroInit = 0
    (*message).Request.u2.ZeroInit = 0;

    // Release lock
    crate::ke::KeReleaseGuardedMutex(
        core::ptr::addr_of_mut!(LpcpLock) as crate::ke::PKGUARDED_MUTEX,
    );

    message
}

/// LpcpGetMessageFromThread — get LPC message from thread, if flag not set.
/// C: if IS_PORT flag set → NULL; else return ptr with flags masked off.
#[inline]
pub unsafe fn LpcpGetMessageFromThread(thread: ddk::PETHREAD) -> ddk::PLPCP_MESSAGE {
    let reply = (*thread).__bindgen_anon_5.LpcReplyMessage as usize;
    if reply & LPCP_THREAD_FLAG_IS_PORT != 0 {
        core::ptr::null_mut()
    } else {
        (reply & !LPCP_THREAD_FLAGS) as ddk::PLPCP_MESSAGE
    }
}

/// LpcpGetPortFromThread — get LPC port from thread, if IS_PORT flag set.
#[inline]
pub unsafe fn LpcpGetPortFromThread(thread: ddk::PETHREAD) -> ddk::PLPCP_PORT_OBJECT {
    let reply = (*thread).__bindgen_anon_5.LpcReplyMessage as usize;
    if reply & LPCP_THREAD_FLAG_IS_PORT != 0 {
        ((*thread).__bindgen_anon_5.LpcWaitingOnPort as usize & !LPCP_THREAD_FLAGS) as ddk::PLPCP_PORT_OBJECT
    } else {
        core::ptr::null_mut()
    }
}

/// LpcpSetPortToThread — set port with IS_PORT flag on thread.
#[inline]
pub unsafe fn LpcpSetPortToThread(thread: ddk::PETHREAD, port: ddk::PLPCP_PORT_OBJECT) {
    (*thread).__bindgen_anon_5.LpcWaitingOnPort =
        ((port as usize) | LPCP_THREAD_FLAG_IS_PORT) as PVOID;
}

/// LpcpGetDataInfoFromMessage — get data info pointer from message.
/// C: `return (PLPCP_DATA_INFO)((PUCHAR)Message + Message->u2.s2.DataInfoOffset);`
#[inline]
pub unsafe fn LpcpGetDataInfoFromMessage(message: *mut ddk::_PORT_MESSAGE) -> ddk::PLPCP_DATA_INFO {
    let offset = (*message).u2.s2.DataInfoOffset as usize;
    (message as *mut u8).add(offset) as ddk::PLPCP_DATA_INFO
}

// ============================================================================
// LPC wait macros (from ntoskrnl/include/internal/lpc_x.h)
// ============================================================================

/// LpcpReceiveWait — wait for LPC receive semaphore.
/// C macro: `KeWaitForSingleObject(s, WrLpcReceive, w, FALSE, NULL)` → returns Status.
#[inline]
pub unsafe fn LpcpReceiveWait(
    semaphore: PVOID,
    wait_mode: ddk::KPROCESSOR_MODE,
) -> NTSTATUS {
    ddk::KeWaitForSingleObject(
        semaphore,
        WR_LPC_RECEIVE,
        wait_mode,
        0, // FALSE
        core::ptr::null_mut(),
    )
}

/// LpcpReplyWait — wait for LPC reply semaphore with APC retry.
/// C macro: KeWaitForSingleObject + STATUS_USER_APC retry via KeReadStateSemaphore.
/// Returns void (caller doesn't use the status).
#[inline]
pub unsafe fn LpcpReplyWait(
    semaphore: PVOID,
    wait_mode: ddk::KPROCESSOR_MODE,
) {
    let status = ddk::KeWaitForSingleObject(
        semaphore,
        WR_LPC_REPLY,
        wait_mode,
        0, // FALSE
        core::ptr::null_mut(),
    );
    if status == STATUS_USER_APC {
        // We were preempted by an APC — check if semaphore is still signaled
        if ddk::KeReadStateSemaphore(semaphore as ddk::PRKSEMAPHORE) != 0 {
            // Still signaled, wait again
            ddk::KeWaitForSingleObject(
                semaphore,
                WR_EXECUTIVE,
                KernelMode as ddk::KPROCESSOR_MODE,
                0, // FALSE
                core::ptr::null_mut(),
            );
        }
    }
}

/// LpcpConnectWait — wait for LPC connect semaphore with APC retry.
/// C macro: KeWaitForSingleObject(s, Executive, w, FALSE, NULL) + APC retry.
#[inline]
pub unsafe fn LpcpConnectWait(
    semaphore: PVOID,
    wait_mode: ddk::KPROCESSOR_MODE,
) {
    let status = ddk::KeWaitForSingleObject(
        semaphore,
        EXECUTIVE,
        wait_mode,
        0, // FALSE
        core::ptr::null_mut(),
    );
    if status == STATUS_USER_APC {
        if ddk::KeReadStateSemaphore(semaphore as ddk::PRKSEMAPHORE) != 0 {
            ddk::KeWaitForSingleObject(
                semaphore,
                WR_EXECUTIVE,
                KernelMode as ddk::KPROCESSOR_MODE,
                0, // FALSE
                core::ptr::null_mut(),
            );
        }
    }
}

/// LpcpCompleteWait — release an LPC semaphore.
/// C macro: `KeReleaseSemaphore(s, 1, 1, FALSE);`
#[inline]
pub unsafe fn LpcpCompleteWait(semaphore: PVOID) {
    ddk::KeReleaseSemaphore(
        semaphore as ddk::PRKSEMAPHORE,
        1, // Increment
        1, // Adjustment
        0, // Wait = FALSE
    );
}

// ============================================================================
// HAL dispatch table macros
// ============================================================================

/// HalHaltSystem — call HalPrivateDispatchTable.HalHaltSystem.
/// C: `HALPRIVATEDISPATCH->HalHaltSystem()`
#[inline]
pub unsafe fn HalHaltSystem() {
    if let Some(f) = HalPrivateDispatchTable.HalHaltSystem {
        f();
    }
}

/// HalSetWakeEnable — call HalPrivateDispatchTable.HalSetWakeEnable.
/// C: `HALPRIVATEDISPATCH->HalSetWakeEnable(NewState)`
#[inline]
pub unsafe fn HalSetWakeEnable(new_state: BOOLEAN) {
    if let Some(f) = HalPrivateDispatchTable.HalSetWakeEnable {
        f(new_state);
    }
}

/// HalEndOfBoot — call HalDispatchTable.HalEndOfBoot.
/// C: `HALDISPATCH->HalEndOfBoot()`
#[inline]
pub unsafe fn HalEndOfBoot() {
    if let Some(f) = HalDispatchTable.HalEndOfBoot {
        f();
    }
}

// ============================================================================
// SeDeleteClientSecurity (macro from sefuncs.h)
// ============================================================================

/// SeDeleteClientSecurity — dereference the client token.
/// C: if primary → PsDereferencePrimaryToken; else PsDereferenceImpersonationToken.
#[inline]
pub unsafe fn SeDeleteClientSecurity(ctx: *mut ddk::_SECURITY_CLIENT_CONTEXT) {
    let token = (*ctx).ClientToken;
    if ddk::SeTokenType(token) == ddk::_TOKEN_TYPE_TokenPrimary {
        ddk::PsDereferencePrimaryToken(token);
    } else {
        ddk::PsDereferenceImpersonationToken(token);
    }
}

// ============================================================================
// IoGetDevObjExtension (macro from ntoskrnl/include/internal/io.h)
// ============================================================================

/// IoGetDevObjExtension — cast DeviceObjectExtension to PEXTENDED_DEVOBJ_EXTENSION.
/// C: `((PEXTENDED_DEVOBJ_EXTENSION)(DeviceObject->DeviceObjectExtension))`
#[inline]
pub unsafe fn IoGetDevObjExtension(
    device_object: ddk::PDEVICE_OBJECT,
) -> ddk::PEXTENDED_DEVOBJ_EXTENSION {
    (*device_object).DeviceObjectExtension as ddk::PEXTENDED_DEVOBJ_EXTENSION
}

// ============================================================================
// ObReferenceObject (C macro → ObfReferenceObject)
// ============================================================================

/// ObReferenceObject — wraps ObfReferenceObject, returns old ref count.
/// C: `#define ObReferenceObject ObfReferenceObject`
/// The tier1 C shim wraps it as NTAPI returning LONG_PTR.
#[inline]
pub unsafe fn ObReferenceObject(object: PVOID) -> LONG_PTR {
    ddk::ObfReferenceObject(object) as LONG_PTR
}

// ============================================================================
// NtCurrentProcess / PsGetCurrentProcess (macros)
// ============================================================================

/// NtCurrentProcess — pseudo-handle for current process.
/// C: `((HANDLE)(LONG_PTR)-1)`
#[inline]
pub unsafe fn NtCurrentProcess() -> ddk::HANDLE {
    (-1isize) as ddk::HANDLE
}

/// PsGetCurrentProcess — returns current EPROCESS.
/// C: `#define PsGetCurrentProcess IoGetCurrentProcess`
/// C body: `return (PEPROCESS)PsGetCurrentThread()->Tcb.ApcState.Process;`
///
/// Implemented directly instead of calling ddk::IoGetCurrentProcess() to avoid
/// infinite recursion: shim export IoGetCurrentProcess → this inline →
/// ddk::IoGetCurrentProcess → shim export → cycle. LLVM detects the cycle,
/// infers noreturn, and replaces the body with ud2.
#[inline]
pub unsafe fn PsGetCurrentProcess() -> ddk::PEPROCESS {
    let thread = super::common::PsGetCurrentThread() as ddk::PKTHREAD;
    (*thread).__bindgen_anon_1.ApcState.Process as ddk::PEPROCESS
}

// ============================================================================
// RtlUnicodeStringToOemSize (macro)
// ============================================================================

/// RtlUnicodeStringToOemSize — wraps RtlxUnicodeStringToOemSize.
/// C: `#define RtlUnicodeStringToOemSize(STRING) ... RtlxUnicodeStringToOemSize(STRING)`
#[inline]
pub unsafe fn RtlUnicodeStringToOemSize(
    unicode_string: ddk::PCUNICODE_STRING,
) -> ULONG {
    ddk::RtlxUnicodeStringToOemSize(unicode_string)
}

// ============================================================================
// CONTAINING_RECORD (C macro → function for Rust)
// ============================================================================

/// CONTAINING_RECORD — subtract field offset from address.
/// Rust callers pass: (address, type_size_unused, field_offset).
#[inline]
pub unsafe fn CONTAINING_RECORD(
    address: PVOID,
    _type_size: usize,
    field_offset: usize,
) -> PVOID {
    (address as *mut u8).sub(field_offset) as PVOID
}

// ============================================================================
// KeRaiseIrql / KeLowerIrql (i386 macros wrapping FASTCALL KfRaiseIrql/KfLowerIrql)
// ============================================================================

/// KeRaiseIrql — i386 only. Wraps KfRaiseIrql.
/// C: `#define KeRaiseIrql(a,b) *(b) = KfRaiseIrql(a)`
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn KeRaiseIrql(new_irql: KIRQL, old_irql: *mut KIRQL) {
    *old_irql = KfRaiseIrql(new_irql);
}

/// KeLowerIrql — i386 only. Wraps KfLowerIrql.
/// C: `#define KeLowerIrql(a) KfLowerIrql(a)`
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn KeLowerIrql(new_irql: KIRQL) {
    KfLowerIrql(new_irql);
}

/// KeRaiseIrqlToDpcLevel — amd64 only. Wraps KfRaiseIrql(DISPATCH_LEVEL).
/// C: FORCEINLINE wrapping KfRaiseIrql on amd64.
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KeRaiseIrqlToDpcLevel() -> KIRQL {
    KfRaiseIrql(DISPATCH_LEVEL)
}

/// KeGetCurrentThread — amd64 only FORCEINLINE.
/// Wraps KeGetCurrentPrcb()->CurrentThread.
/// Other arches have a real exported function.
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KeGetCurrentThread() -> ddk::PKTHREAD {
    crate::mm::KeGetCurrentThread() as ddk::PKTHREAD
}

// ============================================================================
// Cross-arch KeRaiseIrql/KeLowerIrql helpers
// Used internally by ex.rs fast mutex functions on all arches.
// ============================================================================

// Cross-arch KeRaiseIrql/KeLowerIrql extern declarations.
// On amd64/arm64, provided by ARM3/helpers.rs (not in bindgen).
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
}

/// KeRaiseIrqlInternal — cross-arch wrapper for raising IRQL.
/// i386: KfRaiseIrql (FASTCALL). amd64/arm64: real exported KeRaiseIrql.
#[inline]
pub unsafe fn KeRaiseIrqlInternal(new_irql: KIRQL, old_irql: *mut KIRQL) {
    #[cfg(target_arch = "x86")]
    {
        *old_irql = KfRaiseIrql(new_irql);
    }
    #[cfg(not(target_arch = "x86"))]
    {
        KeRaiseIrql(new_irql, old_irql);
    }
}

/// KeLowerIrqlInternal — cross-arch wrapper for lowering IRQL.
#[inline]
pub unsafe fn KeLowerIrqlInternal(new_irql: KIRQL) {
    #[cfg(target_arch = "x86")]
    {
        KfLowerIrql(new_irql);
    }
    #[cfg(not(target_arch = "x86"))]
    {
        KeLowerIrql(new_irql);
    }
}
