//! `#[no_mangle]` export wrappers replacing C `*_rust_shims.c` files.
//!
//! Each function provides a linker symbol that existing Rust `extern`
//! declarations (and any remaining C code) resolve to. The implementations
//! delegate to `ntkrnl_sys::inline::*` for the actual logic.

use ntkrnl_sys::ddk::*;

// ============================================================================
// SE subsystem exports (was: se/se_rust_shims.c)
// ============================================================================

// --- Security descriptor accessors (cdecl — no NTAPI in C shim) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepGetGroupFromDescriptor(
    descriptor: PSECURITY_DESCRIPTOR,
) -> PSID {
    ntkrnl_sys::inline::se::SepGetGroupFromDescriptor(
        descriptor as PISECURITY_DESCRIPTOR,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepGetOwnerFromDescriptor(
    descriptor: PSECURITY_DESCRIPTOR,
) -> PSID {
    ntkrnl_sys::inline::se::SepGetOwnerFromDescriptor(
        descriptor as PISECURITY_DESCRIPTOR,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepGetDaclFromDescriptor(
    descriptor: PSECURITY_DESCRIPTOR,
) -> PACL {
    ntkrnl_sys::inline::se::SepGetDaclFromDescriptor(
        descriptor as PISECURITY_DESCRIPTOR,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepGetSaclFromDescriptor(
    descriptor: PSECURITY_DESCRIPTOR,
) -> PACL {
    ntkrnl_sys::inline::se::SepGetSaclFromDescriptor(
        descriptor as PISECURITY_DESCRIPTOR,
    )
}

// --- Token locks (was NTAPI, now cdecl for Rust-only linkage) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepAcquireTokenLockExclusive(token: PVOID) {
    ntkrnl_sys::inline::se::SepAcquireTokenLockExclusive(token as PTOKEN)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepAcquireTokenLockShared(token: PVOID) {
    ntkrnl_sys::inline::se::SepAcquireTokenLockShared(token as PTOKEN)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SepReleaseTokenLock(token: PVOID) {
    ntkrnl_sys::inline::se::SepReleaseTokenLock(token as PTOKEN)
}

// --- SeQuerySubjectContextToken (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SeQuerySubjectContextToken(
    ctx: *const _SECURITY_SUBJECT_CONTEXT,
) -> PVOID {
    ntkrnl_sys::inline::se::SeQuerySubjectContextToken(ctx)
}

// --- SeLengthSid (cdecl — no NTAPI in C shim) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SeLengthSid(sid: PSID) -> ULONG {
    ntkrnl_sys::inline::se::SeLengthSid(sid)
}

// ============================================================================
// Common exports used by SE (and shared across subsystems)
// ============================================================================

// --- DefaultQueryInfoBufferCheck (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn DefaultQueryInfoBufferCheck(
    class: ULONG,
    class_list: *const _INFORMATION_CLASS_INFO,
    class_list_entries: ULONG,
    flags: ULONG,
    buffer: PVOID,
    buffer_length: ULONG,
    return_length: *mut ULONG,
    return_length_ptr: *mut ULONG_PTR,
    previous_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    ntkrnl_sys::inline::common::DefaultQueryInfoBufferCheck(
        class,
        class_list,
        class_list_entries,
        flags,
        buffer,
        buffer_length,
        return_length,
        return_length_ptr as *mut usize,
        previous_mode,
    )
}

// --- DefaultSetInfoBufferCheck (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn DefaultSetInfoBufferCheck(
    class: ULONG,
    class_list: *const _INFORMATION_CLASS_INFO,
    class_list_entries: ULONG,
    buffer: PVOID,
    buffer_length: ULONG,
    previous_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    ntkrnl_sys::inline::common::DefaultSetInfoBufferCheck(
        class,
        class_list,
        class_list_entries,
        buffer,
        buffer_length,
        previous_mode,
    )
}

// --- Probe functions (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForReadLargeInteger(
    addr: PLARGE_INTEGER,
) -> _LARGE_INTEGER {
    ntkrnl_sys::inline::common::ProbeForReadLargeInteger(addr)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForReadUchar(addr: *const u8) -> u8 {
    ntkrnl_sys::inline::common::ProbeForReadUchar(addr)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForReadUshort(addr: *const u16) -> u16 {
    ntkrnl_sys::inline::common::ProbeForReadUshort(addr)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForWriteHandle(ptr: *mut HANDLE) {
    ntkrnl_sys::inline::common::ProbeForWriteHandle(ptr)
}

// --- ProbeAndCaptureUnicodeString (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeAndCaptureUnicodeString(
    dest: PUNICODE_STRING,
    mode: KPROCESSOR_MODE,
    src: *const _UNICODE_STRING,
) -> NTSTATUS {
    ntkrnl_sys::inline::common::ProbeAndCaptureUnicodeString(dest, mode, src)
}

// --- ReleaseCapturedUnicodeString (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ReleaseCapturedUnicodeString(
    str: PUNICODE_STRING,
    mode: KPROCESSOR_MODE,
) {
    ntkrnl_sys::inline::common::ReleaseCapturedUnicodeString(str, mode)
}

// --- InitializeObjectAttributes (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InitializeObjectAttributes(
    p: *mut _OBJECT_ATTRIBUTES,
    n: PUNICODE_STRING,
    a: ULONG,
    r: HANDLE,
    s: PVOID,
) {
    ntkrnl_sys::inline::common::InitializeObjectAttributes(p, n, a, r, s)
}

// --- ExInitializeResource (NTAPI) ---
// Maps to ExInitializeResourceLite. C shim returned void, but the underlying
// function returns NTSTATUS. We return void to match the C shim ABI.

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExInitializeResource(resource: PERESOURCE) {
    ntkrnl_sys::inline::common::ExInitializeResource(resource);
}

// --- PspClearCrossThreadFlag (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspClearCrossThreadFlag(thread: PETHREAD, flag: ULONG) {
    ntkrnl_sys::inline::common::PspClearCrossThreadFlag(thread, flag)
}

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
    ntkrnl_sys::inline::ke::KeSetContextPc(ctx, pc as usize)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeSetContextReturnRegister(
    ctx: *mut _CONTEXT,
    val: ULONG_PTR,
) {
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
    ntkrnl_sys::inline::mm::MmGetMdlPfnArray(mdl) as *mut ULONG_PTR
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MmGetSystemAddressForMdl(mdl: PMDL) -> PVOID {
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
    ntkrnl_sys::inline::ex::ExFreeToNPagedLookasideList(lookaside, entry)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExAllocateFromPagedLookasideList(
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
    ntkrnl_sys::inline::ex::ExInitializeWorkItem(item, routine, context)
}

// --- FSRTL functions (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn FsRtlEnterFileSystem() {
    ntkrnl_sys::inline::common::FsRtlEnterFileSystem()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn FsRtlExitFileSystem() {
    ntkrnl_sys::inline::common::FsRtlExitFileSystem()
}

// --- CC functions (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn CcBugCheck(a: ULONG, b: ULONG, c: ULONG) {
    ntkrnl_sys::inline::common::CcBugCheck(a, b, c)
}

// --- RTL splay link functions (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlInitializeSplayLinks(links: PRTL_SPLAY_LINKS) {
    ntkrnl_sys::inline::common::RtlInitializeSplayLinks(links)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlInsertAsLeftChild(
    parent: PRTL_SPLAY_LINKS,
    child: PRTL_SPLAY_LINKS,
) {
    ntkrnl_sys::inline::common::RtlInsertAsLeftChild(parent, child)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlInsertAsRightChild(
    parent: PRTL_SPLAY_LINKS,
    child: PRTL_SPLAY_LINKS,
) {
    ntkrnl_sys::inline::common::RtlInsertAsRightChild(parent, child)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlIsLeftChild(links: PRTL_SPLAY_LINKS) -> BOOLEAN {
    ntkrnl_sys::inline::common::RtlIsLeftChild(links)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlParent(links: PRTL_SPLAY_LINKS) -> PRTL_SPLAY_LINKS {
    ntkrnl_sys::inline::common::RtlParent(links)
}

// --- RTL intsafe (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlLongLongAdd(
    augend: i64,
    addend: i64,
    result: *mut i64,
) -> NTSTATUS {
    ntkrnl_sys::inline::common::RtlLongLongAdd(augend, addend, result)
}

// --- Exception record conversion (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExceptionRecord32To64(
    ex32: *const _EXCEPTION_RECORD32,
    ex64: *mut _EXCEPTION_RECORD64,
) {
    ntkrnl_sys::inline::common::ExceptionRecord32To64(ex32, ex64)
}

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
    ntkrnl_sys::inline::tier1::CONTAINING_RECORD(address, type_size, field_offset)
}

// --- ProbeForWritePointer (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForWritePointer(address: *mut PVOID) {
    ntkrnl_sys::inline::common::ProbeForWritePointer(address)
}

// --- ProbeForWriteUlong (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForWriteUlong(address: *mut ULONG) {
    ntkrnl_sys::inline::common::ProbeForWriteUlong(address)
}

// --- IoGetDevObjExtension (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoGetDevObjExtension(
    device_object: PDEVICE_OBJECT,
) -> PEXTENDED_DEVOBJ_EXTENSION {
    ntkrnl_sys::inline::tier1::IoGetDevObjExtension(device_object)
}

// --- SeDeleteClientSecurity (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn SeDeleteClientSecurity(
    ctx: *mut _SECURITY_CLIENT_CONTEXT,
) {
    ntkrnl_sys::inline::tier1::SeDeleteClientSecurity(ctx)
}

// --- ObReferenceObject (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObReferenceObject(object: PVOID) -> LONG_PTR {
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
pub unsafe extern "C" fn PsGetCurrentProcess() -> PEPROCESS {
    ntkrnl_sys::inline::tier1::PsGetCurrentProcess()
}

// --- RtlUnicodeStringToOemSize (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlUnicodeStringToOemSize(
    unicode_string: PCUNICODE_STRING,
) -> ULONG {
    ntkrnl_sys::inline::tier1::RtlUnicodeStringToOemSize(unicode_string)
}

// --- RtlInitEmptyAnsiString (cdecl — no NTAPI in C shim) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlInitEmptyAnsiString(
    ansi_string: PANSI_STRING,
    buffer: *mut i8,
    maximum_length: u16,
) {
    ntkrnl_sys::inline::common::RtlInitEmptyAnsiString(ansi_string, buffer, maximum_length)
}

// --- InsertTailList (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InsertTailList(
    list_head: PLIST_ENTRY,
    entry: PLIST_ENTRY,
) {
    ntkrnl_sys::inline::common::InsertTailList(list_head, entry)
}

// --- ExFreeToPagedLookasideList (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExFreeToPagedLookasideList(
    lookaside: PPAGED_LOOKASIDE_LIST,
    entry: PVOID,
) {
    ntkrnl_sys::inline::ex::ExFreeToPagedLookasideList(lookaside, entry)
}

// --- LPC wait macros (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpReceiveWait(
    semaphore: PVOID,
    wait_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    ntkrnl_sys::inline::tier1::LpcpReceiveWait(semaphore, wait_mode)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpReplyWait(
    semaphore: PVOID,
    wait_mode: KPROCESSOR_MODE,
) {
    ntkrnl_sys::inline::tier1::LpcpReplyWait(semaphore, wait_mode)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpConnectWait(
    semaphore: PVOID,
    wait_mode: KPROCESSOR_MODE,
) {
    ntkrnl_sys::inline::tier1::LpcpConnectWait(semaphore, wait_mode)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpCompleteWait(semaphore: PVOID) {
    ntkrnl_sys::inline::tier1::LpcpCompleteWait(semaphore)
}

// --- LPC FORCEINLINE functions ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpAllocateFromPortZone() -> PLPCP_MESSAGE {
    ntkrnl_sys::inline::tier1::LpcpAllocateFromPortZone()
}

// LpcpGetMessageFromThread / LpcpGetPortFromThread / LpcpSetPortToThread:
// cdecl (no NTAPI in C shim) — Rust DDK bindgen declares them as extern "C".

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpGetMessageFromThread(thread: PETHREAD) -> PLPCP_MESSAGE {
    ntkrnl_sys::inline::tier1::LpcpGetMessageFromThread(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpGetPortFromThread(thread: PETHREAD) -> PLPCP_PORT_OBJECT {
    ntkrnl_sys::inline::tier1::LpcpGetPortFromThread(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpSetPortToThread(thread: PETHREAD, port: PLPCP_PORT_OBJECT) {
    ntkrnl_sys::inline::tier1::LpcpSetPortToThread(thread, port)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn LpcpGetDataInfoFromMessage(
    message: *mut _PORT_MESSAGE,
) -> PLPCP_DATA_INFO {
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
    ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation(irp)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoGetCurrentIrpStackLocation(irp: PIRP) -> PIO_STACK_LOCATION {
    ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation(irp)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoSetNextIrpStackLocation(irp: PIRP) {
    ntkrnl_sys::inline::tier1::IoSetNextIrpStackLocation(irp)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoSkipCurrentIrpStackLocation(irp: PIRP) {
    ntkrnl_sys::inline::tier1::IoSkipCurrentIrpStackLocation(irp)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoSetCancelRoutine(
    irp: PIRP,
    cancel_routine: PDRIVER_CANCEL,
) -> PDRIVER_CANCEL {
    ntkrnl_sys::inline::tier1::IoSetCancelRoutine(irp, cancel_routine)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn IoMarkIrpPending(irp: PIRP) {
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
    ntkrnl_sys::inline::tier1::KeRaiseIrql(new_irql, old_irql)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeLowerIrql(new_irql: KIRQL) {
    ntkrnl_sys::inline::tier1::KeLowerIrql(new_irql)
}

// KeGetCurrentThread — provided by ke/thrdobj.rs, not needed here.

// --- SRW Lock stubs (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlAcquireSRWLockExclusive(srw_lock: PRTL_SRWLOCK) {
    ntkrnl_sys::inline::tier1::RtlAcquireSRWLockExclusive(srw_lock)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlReleaseSRWLockExclusive(srw_lock: PRTL_SRWLOCK) {
    ntkrnl_sys::inline::tier1::RtlReleaseSRWLockExclusive(srw_lock)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlAcquireSRWLockShared(srw_lock: PRTL_SRWLOCK) {
    ntkrnl_sys::inline::tier1::RtlAcquireSRWLockShared(srw_lock)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlReleaseSRWLockShared(srw_lock: PRTL_SRWLOCK) {
    ntkrnl_sys::inline::tier1::RtlReleaseSRWLockShared(srw_lock)
}

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
    ntkrnl_sys::inline::mm::MmGetSystemAddressForMdlSafe(mdl, priority)
}

// --- ObpGetHandleObject (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpGetHandleObject(entry: PHANDLE_TABLE_ENTRY) -> POBJECT_HEADER {
    ntkrnl_sys::macros::ObpGetHandleObject(entry)
}

// --- ProbeForWriteBoolean (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForWriteBoolean(address: *mut BOOLEAN) {
    ntkrnl_sys::inline::common::ProbeForWriteBoolean(address)
}

// --- ProbeForWriteLangId (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForWriteLangId(address: *mut u16) {
    ntkrnl_sys::inline::common::ProbeForWriteLangId(address)
}

// --- InterlockedExchangeAdd64 (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InterlockedExchangeAdd64(addend: *mut i64, value: i64) -> i64 {
    ntkrnl_sys::inline::common::InterlockedExchangeAdd64(addend, value)
}

// --- PopEntryList (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PopEntryList(
    list_head: *mut ntkrnl_sys::ddk::_SINGLE_LIST_ENTRY,
) -> *mut ntkrnl_sys::ddk::_SINGLE_LIST_ENTRY {
    ntkrnl_sys::inline::common::PopEntryList(list_head)
}

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

// --- RtlStringCbCopyA (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlStringCbCopyA(
    psz_dest: *mut u8,
    cb_dest: usize,
    psz_src: *const u8,
) -> NTSTATUS {
    ntkrnl_sys::inline::common::RtlStringCbCopyA(psz_dest, cb_dest, psz_src)
}

// --- KxAcquireSpinLock / KxReleaseSpinLock (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxAcquireSpinLock(spin_lock: *mut ULONG_PTR) {
    ntkrnl_sys::inline::ke::KxAcquireSpinLock(spin_lock as *mut usize)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxReleaseSpinLock(spin_lock: *mut ULONG_PTR) {
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

// --- PS PsGetCurrentThread (NTAPI = stdcall on i386) ---
// Must be extern "system" to match calling convention of ntkrnl-sys extern declarations.
// Mismatch (cdecl def vs stdcall call) within the same LLVM CGU is UB → noreturn cascade.

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn PsGetCurrentThread() -> PETHREAD {
    ntkrnl_sys::inline::common::PsGetCurrentThread()
}

// --- RTL RtlInitEmptyUnicodeString (cdecl — FORCEINLINE, not NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlInitEmptyUnicodeString(
    unicode_string: PUNICODE_STRING,
    buffer: *mut u16,
    buffer_size: u16,
) {
    ntkrnl_sys::inline::common::RtlInitEmptyUnicodeString(
        unicode_string,
        buffer,
        buffer_size,
    )
}

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
    ntkrnl_sys::inline::ex::ExAcquirePushLockExclusive(push_lock)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExAcquirePushLockShared(
    push_lock: PEX_PUSH_LOCK,
) {
    ntkrnl_sys::inline::ex::ExAcquirePushLockShared(push_lock)
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
    ntkrnl_sys::inline::ex::ExInsertFastReference(fast_ref, object)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExReleaseFastReference(
    fast_ref: PEX_FAST_REF,
    object: PVOID,
) -> BOOLEAN {
    ntkrnl_sys::inline::ex::ExReleaseFastReference(fast_ref, object)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExSwapFastReference(
    fast_ref: PEX_FAST_REF,
    object: PVOID,
) -> _EX_FAST_REF {
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
    let result = ntkrnl_sys::inline::ex::ExCompareSwapFastReference(
        fast_ref,
        object,
        old_object,
    );
    core::mem::transmute(result)
}

// --- OB functions (cdecl — no NTAPI in C shim) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpSelectObjectLockSlot(
    header: POBJECT_HEADER,
) -> ULONG {
    ntkrnl_sys::inline::ob::ObpSelectObjectLockSlot(header)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpValidateAttributes(
    attributes: ULONG,
    previous_mode: KPROCESSOR_MODE,
) -> ULONG {
    ntkrnl_sys::inline::ob::ObpValidateAttributes(attributes, previous_mode)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpAcquireObjectLock(
    header: POBJECT_HEADER,
) {
    ntkrnl_sys::inline::ob::ObpAcquireObjectLock(header)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpReleaseObjectLock(
    header: POBJECT_HEADER,
) {
    ntkrnl_sys::inline::ob::ObpReleaseObjectLock(header)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpAcquireObjectLockShared(
    header: POBJECT_HEADER,
) {
    ntkrnl_sys::inline::ob::ObpAcquireObjectLockShared(header)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpEnterObjectTypeMutex(
    object_type: POBJECT_TYPE,
) {
    ntkrnl_sys::inline::ob::ObpEnterObjectTypeMutex(object_type)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpLeaveObjectTypeMutex(
    object_type: POBJECT_TYPE,
) {
    ntkrnl_sys::inline::ob::ObpLeaveObjectTypeMutex(object_type)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpCalloutStart(
    callout_irql: *mut u8,
) {
    ntkrnl_sys::inline::ob::ObpCalloutStart(callout_irql)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpCalloutEnd(
    callout_irql: u8,
    procedure: *const u8,
    object_type: POBJECT_TYPE,
    object: PVOID,
) {
    ntkrnl_sys::inline::ob::ObpCalloutEnd(
        callout_irql,
        procedure,
        object_type,
        object,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpReferenceNameInfo(
    header: POBJECT_HEADER,
) -> POBJECT_HEADER_NAME_INFO {
    ntkrnl_sys::inline::ob::ObpReferenceNameInfo(header)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpDereferenceNameInfo(
    name_info: POBJECT_HEADER_NAME_INFO,
) {
    ntkrnl_sys::inline::ob::ObpDereferenceNameInfo(name_info)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpInitializeLookupContext(
    context: POBP_LOOKUP_CONTEXT,
) {
    ntkrnl_sys::inline::ob::ObpInitializeLookupContext(context)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpAcquireDirectoryLockShared(
    directory: POBJECT_DIRECTORY,
    context: POBP_LOOKUP_CONTEXT,
) {
    ntkrnl_sys::inline::ob::ObpAcquireDirectoryLockShared(directory, context)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpAcquireDirectoryLockExclusive(
    directory: POBJECT_DIRECTORY,
    context: POBP_LOOKUP_CONTEXT,
) {
    ntkrnl_sys::inline::ob::ObpAcquireDirectoryLockExclusive(directory, context)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpReleaseDirectoryLock(
    directory: POBJECT_DIRECTORY,
    context: POBP_LOOKUP_CONTEXT,
) {
    ntkrnl_sys::inline::ob::ObpReleaseDirectoryLock(directory, context)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpAcquireLookupContextLock(
    context: POBP_LOOKUP_CONTEXT,
    directory: POBJECT_DIRECTORY,
) {
    ntkrnl_sys::inline::ob::ObpAcquireLookupContextLock(context, directory)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpReleaseLookupContextObject(
    context: POBP_LOOKUP_CONTEXT,
) {
    ntkrnl_sys::inline::ob::ObpReleaseLookupContextObject(context)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpReleaseLookupContext(
    context: POBP_LOOKUP_CONTEXT,
) {
    ntkrnl_sys::inline::ob::ObpReleaseLookupContext(context)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpReleaseObjectCreateInformation(
    create_info: POBJECT_CREATE_INFORMATION,
) {
    ntkrnl_sys::inline::ob::ObpReleaseObjectCreateInformation(create_info)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpAllocateObjectCreateInfoBuffer(
    lookaside_type: PP_NPAGED_LOOKASIDE_NUMBER,
) -> PVOID {
    ntkrnl_sys::inline::ob::ObpAllocateObjectCreateInfoBuffer(lookaside_type)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpFreeCapturedAttributes(
    buffer: PVOID,
    lookaside_type: PP_NPAGED_LOOKASIDE_NUMBER,
) {
    ntkrnl_sys::inline::ob::ObpFreeCapturedAttributes(buffer, lookaside_type)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpFreeObjectCreateInformation(
    create_info: POBJECT_CREATE_INFORMATION,
) {
    ntkrnl_sys::inline::ob::ObpFreeObjectCreateInformation(create_info)
}

// ============================================================================
// PS subsystem exports (was: ps/ps_rust_shims.c)
// ============================================================================

// --- Macro trampolines (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspSetCrossThreadFlag(thread: PETHREAD, flag: ULONG) {
    ntkrnl_sys::inline::ps::PspSetCrossThreadFlag(thread, flag)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspSetProcessFlag(process: PEPROCESS, flag: ULONG) {
    ntkrnl_sys::inline::ps::PspSetProcessFlag(process, flag)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspClearProcessFlag(process: PEPROCESS, flag: ULONG) {
    ntkrnl_sys::inline::ps::PspClearProcessFlag(process, flag)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspPrioritySeparationFromMask(mask: ULONG) -> ULONG {
    ntkrnl_sys::inline::ps::PspPrioritySeparationFromMask(mask)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspQuantumTypeFromMask(mask: ULONG) -> ULONG {
    ntkrnl_sys::inline::ps::PspQuantumTypeFromMask(mask)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspQuantumLengthFromMask(mask: ULONG) -> ULONG {
    ntkrnl_sys::inline::ps::PspQuantumLengthFromMask(mask)
}

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
    ntkrnl_sys::inline::ke::KeGetContextReturnRegister(ctx) as ULONG_PTR
}

// --- InterlockedDecrementUL (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InterlockedDecrementUL(addend: *mut ULONG) -> ULONG {
    ntkrnl_sys::inline::common::InterlockedDecrementUL(addend)
}

// --- KeQuerySystemTime (NTAPI, amd64/arm64 only — i386 has real HAL function) ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeQuerySystemTime(current_time: PLARGE_INTEGER) {
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
    ntkrnl_sys::inline::ke::KeEnterGuardedRegionThread(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeLeaveGuardedRegionThread(thread: PKTHREAD) {
    ntkrnl_sys::inline::ke::KeLeaveGuardedRegionThread(thread)
}

// --- ExRaiseStatus (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn ExRaiseStatus(status: NTSTATUS) -> ! {
    ntkrnl_sys::inline::ke::ExRaiseStatus(status)
}

// --- InterlockedCompareExchangePointer (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InterlockedCompareExchangePointer(
    destination: *mut PVOID,
    exchange: PVOID,
    comparand: PVOID,
) -> PVOID {
    ntkrnl_sys::inline::common::InterlockedCompareExchangePointer(
        destination, exchange, comparand,
    )
}

// InterlockedCompareExchange — already provided by ex/exintrin.rs with correct
// calling conventions (fastcall on i386, cdecl on 64-bit).

// --- KeGetTrapFrame (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeGetTrapFrame(thread: PKTHREAD) -> PKTRAP_FRAME {
    ntkrnl_sys::inline::ke::KeGetTrapFrame(thread)
}

// --- KeGetExceptionFrame (NTAPI) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeGetExceptionFrame(thread: PKTHREAD) -> PKEXCEPTION_FRAME {
    ntkrnl_sys::inline::ke::KeGetExceptionFrame(thread)
}

// --- FORCEINLINE function wrappers (cdecl — no NTAPI in C shim) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspRunCreateThreadNotifyRoutines(
    current_thread: PETHREAD,
    create: BOOLEAN,
) {
    ntkrnl_sys::inline::ps::PspRunCreateThreadNotifyRoutines(current_thread, create)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspRunCreateProcessNotifyRoutines(
    current_process: PEPROCESS,
    create: BOOLEAN,
) {
    ntkrnl_sys::inline::ps::PspRunCreateProcessNotifyRoutines(current_process, create)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspRunLoadImageNotifyRoutines(
    full_image_name: PUNICODE_STRING,
    process_id: HANDLE,
    image_info: *mut _IMAGE_INFO,
) {
    ntkrnl_sys::inline::ps::PspRunLoadImageNotifyRoutines(
        full_image_name, process_id, image_info,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspRunLegoRoutine(thread: PKTHREAD) {
    ntkrnl_sys::inline::ps::PspRunLegoRoutine(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspLockProcessSecurityShared(process: PEPROCESS) {
    ntkrnl_sys::inline::ps::PspLockProcessSecurityShared(process)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspUnlockProcessSecurityShared(process: PEPROCESS) {
    ntkrnl_sys::inline::ps::PspUnlockProcessSecurityShared(process)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspLockProcessSecurityExclusive(process: PEPROCESS) {
    ntkrnl_sys::inline::ps::PspLockProcessSecurityExclusive(process)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspUnlockProcessSecurityExclusive(process: PEPROCESS) {
    ntkrnl_sys::inline::ps::PspUnlockProcessSecurityExclusive(process)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspLockThreadSecurityShared(thread: PETHREAD) {
    ntkrnl_sys::inline::ps::PspLockThreadSecurityShared(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspUnlockThreadSecurityShared(thread: PETHREAD) {
    ntkrnl_sys::inline::ps::PspUnlockThreadSecurityShared(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspLockThreadSecurityExclusive(thread: PETHREAD) {
    ntkrnl_sys::inline::ps::PspLockThreadSecurityExclusive(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspUnlockThreadSecurityExclusive(thread: PETHREAD) {
    ntkrnl_sys::inline::ps::PspUnlockThreadSecurityExclusive(thread)
}

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
    ntkrnl_sys::inline::mm::MiMakeDirtyPage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiMakeCleanPage(pte: PMMPTE) {
    ntkrnl_sys::inline::mm::MiMakeCleanPage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiMakeAccessedPage(pte: PMMPTE) {
    ntkrnl_sys::inline::mm::MiMakeAccessedPage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiPageDisableCache(pte: PMMPTE) {
    ntkrnl_sys::inline::mm::MiPageDisableCache(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiPageWriteThrough(pte: PMMPTE) {
    ntkrnl_sys::inline::mm::MiPageWriteThrough(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiPageWriteCombined(pte: PMMPTE) {
    ntkrnl_sys::inline::mm::MiPageWriteCombined(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsPageWriteable(pte: PMMPTE) -> BOOLEAN {
    ntkrnl_sys::inline::mm::MiIsPageWriteable(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsPageDirty(pte: PMMPTE) -> BOOLEAN {
    ntkrnl_sys::inline::mm::MiIsPageDirty(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiMakeOwnerPage(pte: PMMPTE) {
    ntkrnl_sys::inline::mm::MiMakeOwnerPage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiMakeWritePage(pte: PMMPTE) {
    ntkrnl_sys::inline::mm::MiMakeWritePage(pte)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsMappedPte(pte: PMMPTE) -> BOOLEAN {
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
    ntkrnl_sys::inline::mm::MiInterlockedCompareExchangePte(pte, exchange, comperand)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiInterlockedExchangePte(
    pte: PMMPTE,
    value: ntkrnl_sys::base::LONG_PTR,
) -> ntkrnl_sys::base::LONG_PTR {
    ntkrnl_sys::inline::mm::MiInterlockedExchangePte(pte, value)
}

// --- TLB operations (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KeInvalidateTlbEntry(address: PVOID) {
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
    ntkrnl_sys::inline::mm::MM_ANY_WS_LOCK_HELD(thread)
}

// --- PDE hierarchy check (64-bit only, cdecl) ---

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsPdeForAddressValid(address: PVOID) -> BOOLEAN {
    ntkrnl_sys::inline::mm::MiIsPdeForAddressValid(address)
}

// --- Working set locks (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiLockProcessWorkingSet(process: PEPROCESS, thread: PETHREAD) {
    ntkrnl_sys::inline::mm::MiLockProcessWorkingSet(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiUnlockProcessWorkingSet(process: PEPROCESS, thread: PETHREAD) {
    ntkrnl_sys::inline::mm::MiUnlockProcessWorkingSet(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiLockProcessWorkingSetShared(process: PEPROCESS, thread: PETHREAD) {
    ntkrnl_sys::inline::mm::MiLockProcessWorkingSetShared(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiUnlockProcessWorkingSetShared(process: PEPROCESS, thread: PETHREAD) {
    ntkrnl_sys::inline::mm::MiUnlockProcessWorkingSetShared(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiLockProcessWorkingSetUnsafe(process: PEPROCESS, thread: PETHREAD) {
    ntkrnl_sys::inline::mm::MiLockProcessWorkingSetUnsafe(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiUnlockProcessWorkingSetUnsafe(process: PEPROCESS, thread: PETHREAD) {
    ntkrnl_sys::inline::mm::MiUnlockProcessWorkingSetUnsafe(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiLockWorkingSet(thread: PETHREAD, ws: PMMSUPPORT) {
    ntkrnl_sys::inline::mm::MiLockWorkingSet(thread, ws)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiUnlockWorkingSet(thread: PETHREAD, ws: PMMSUPPORT) {
    ntkrnl_sys::inline::mm::MiUnlockWorkingSet(thread, ws)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiLockWorkingSetShared(thread: PETHREAD, ws: PMMSUPPORT) {
    ntkrnl_sys::inline::mm::MiLockWorkingSetShared(thread, ws)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiUnlockWorkingSetShared(thread: PETHREAD, ws: PMMSUPPORT) {
    ntkrnl_sys::inline::mm::MiUnlockWorkingSetShared(thread, ws)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiLockProcessWorkingSetForFault(
    process: PEPROCESS,
    thread: PETHREAD,
    safe: BOOLEAN,
    shared: BOOLEAN,
) {
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
    ntkrnl_sys::inline::mm::MiUnlockProcessWorkingSetForFault(process, thread, safe, shared)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiConvertSharedWorkingSetLockToExclusive(
    thread: PETHREAD,
    vm: PMMSUPPORT,
) -> BOOLEAN {
    ntkrnl_sys::inline::mm::MiConvertSharedWorkingSetLockToExclusive(thread, vm)
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
    ntkrnl_sys::inline::mm::MiIsUserPte(address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIsUserPde(address: PVOID) -> BOOLEAN {
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
    ntkrnl_sys::inline::mm::MI_MAKE_HARDWARE_PTE(
        new_pte, mapping_pte, protection_mask, page_frame_number,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MI_UPDATE_VALID_PTE(pte: PMMPTE, temp_pte: _MMPTE) {
    ntkrnl_sys::inline::mm::MI_UPDATE_VALID_PTE(pte, temp_pte)
}

// --- User/global PTE mask (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiDetermineUserGlobalPteMask(
    pointer_pte: PMMPTE,
) -> ntkrnl_sys::base::ULONG_PTR {
    ntkrnl_sys::inline::mm::MiDetermineUserGlobalPteMask(pointer_pte)
}

// --- Macro wrappers (was NTAPI, now cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RMAP_IS_SEGMENT(address: PVOID) -> BOOLEAN {
    ntkrnl_sys::inline::mm::RMAP_IS_SEGMENT(address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PAGE_ROUND_DOWN(address: PVOID) -> ntkrnl_sys::base::ULONG_PTR {
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
    ntkrnl_sys::inline::mm::MmSetDirtyPage(process, address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InterlockedIncrementUL(addend: *mut ULONG) -> ULONG {
    ntkrnl_sys::inline::mm::InterlockedIncrementUL(addend)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InterlockedExchangeAddUL(
    addend: *mut ULONG,
    value: ULONG,
) -> ULONG {
    ntkrnl_sys::inline::mm::InterlockedExchangeAddUL(addend, value)
}

// --- Pool and segment helpers (was NTAPI, now cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExAllocatePoolZero(
    pool_type: POOL_TYPE,
    number_of_bytes: ntkrnl_sys::base::ULONG_PTR,
    tag: ULONG,
) -> PVOID {
    ntkrnl_sys::inline::mm::ExAllocatePoolZero(pool_type, number_of_bytes, tag)
}

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
    ntkrnl_sys::inline::mm::MmInitializeMdl(mdl, base_va, length)
}

// --- PFN reference counting (cdecl) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiIncrementPageTableReferences(address: PVOID) -> u16 {
    ntkrnl_sys::inline::mm::MiIncrementPageTableReferences(address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiDropLockCount(pfn: PMMPFN) {
    ntkrnl_sys::inline::mm::MiDropLockCount(pfn)
}

// MiDereferencePfnAndDropLockCount — already exported from ARM3/helpers.rs

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiReferenceProbedPageAndBumpLockCount(pfn: PMMPFN) {
    ntkrnl_sys::inline::mm::MiReferenceProbedPageAndBumpLockCount(pfn)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiReferenceUnusedPageAndBumpLockCount(pfn: PMMPFN) {
    ntkrnl_sys::inline::mm::MiReferenceUnusedPageAndBumpLockCount(pfn)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn MiReferenceUsedPageAndBumpLockCount(pfn: PMMPFN) {
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
    ntkrnl_sys::inline::mm::ProbeForWriteSize_t(ptr)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForWriteIoStatusBlock(ptr: *mut IO_STATUS_BLOCK) {
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
    ntkrnl_sys::inline::ke::KiAcquireThreadLock(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleaseThreadLock(thread: PKTHREAD) {
    ntkrnl_sys::inline::ke::KiReleaseThreadLock(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireDispatcherObject(object: *mut _DISPATCHER_HEADER) {
    ntkrnl_sys::inline::ke::KiAcquireDispatcherObject(object)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiReleaseDispatcherObject(object: *mut _DISPATCHER_HEADER) {
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
    ntkrnl_sys::inline::ke::KiInsertDeferredReadyList(thread)
}

// --- APC lock functions ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireApcLockRaiseToSynch(
    thread: PKTHREAD,
    handle: PKLOCK_QUEUE_HANDLE,
) {
    ntkrnl_sys::inline::ke::KiAcquireApcLockRaiseToSynch(thread, handle)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireApcLockAtSynchLevel(
    thread: PKTHREAD,
    handle: PKLOCK_QUEUE_HANDLE,
) {
    ntkrnl_sys::inline::ke::KiAcquireApcLockAtSynchLevel(thread, handle)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAcquireApcLockRaiseToDpc(
    thread: PKTHREAD,
    handle: PKLOCK_QUEUE_HANDLE,
) {
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
    ntkrnl_sys::inline::ke::KiRecalculateDueTime(original_due_time, due_time, new_due_time)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiCheckThreadStackSwap(
    thread: PKTHREAD,
    wait_mode: KPROCESSOR_MODE,
) -> ntkrnl_sys::base::BOOLEAN {
    ntkrnl_sys::inline::ke::KiCheckThreadStackSwap(thread, wait_mode)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiCheckAlertability(
    thread: PKTHREAD,
    alertable: ntkrnl_sys::base::BOOLEAN,
    wait_mode: KPROCESSOR_MODE,
) -> ntkrnl_sys::base::NTSTATUS {
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
    ntkrnl_sys::inline::ke::KiRemoveEntryTimer(timer)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxInsertTimer(timer: PKTIMER, hand: ntkrnl_sys::base::ULONG) {
    ntkrnl_sys::inline::ke::KxInsertTimer(timer, hand)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiComputeDueTime(
    timer: PKTIMER,
    due_time: _LARGE_INTEGER,
    hand: *mut ntkrnl_sys::base::ULONG,
) -> ntkrnl_sys::base::BOOLEAN {
    ntkrnl_sys::inline::ke::KiComputeDueTime(timer, due_time, hand)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxRemoveTreeTimer(timer: PKTIMER) {
    ntkrnl_sys::inline::ke::KxRemoveTreeTimer(timer)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxSetTimerForThreadWait(
    timer: PKTIMER,
    interval: _LARGE_INTEGER,
    hand: *mut ntkrnl_sys::base::ULONG,
) {
    ntkrnl_sys::inline::ke::KxSetTimerForThreadWait(timer, interval, hand)
}

// --- Unwait functions ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxUnwaitThread(
    object: *mut _DISPATCHER_HEADER,
    increment: KPRIORITY,
) {
    ntkrnl_sys::inline::ke::KxUnwaitThread(object, increment)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxUnwaitThread_impl(
    header: *mut _DISPATCHER_HEADER,
    increment: KPRIORITY,
) {
    ntkrnl_sys::inline::ke::KxUnwaitThread(header, increment)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxUnwaitThreadForEvent(
    event: PKEVENT,
    increment: KPRIORITY,
) {
    ntkrnl_sys::inline::ke::KxUnwaitThreadForEvent(event, increment)
}

// --- Ready queue ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxQueueReadyThread(thread: PKTHREAD, prcb: PKPRCB) {
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
    ntkrnl_sys::inline::ke::KiSatisfyMutantWait(object, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiSatisfyNonMutantWait(object: *mut _KMUTANT) {
    ntkrnl_sys::inline::ke::KiSatisfyNonMutantWait(object)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiSatisfyObjectWait(object: *mut _KMUTANT, thread: PKTHREAD) {
    ntkrnl_sys::inline::ke::KiSatisfyObjectWait(object, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiAddThreadToWaitList(
    thread: PKTHREAD,
    swappable: ntkrnl_sys::base::BOOLEAN,
) {
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
    ntkrnl_sys::inline::ke::KxSingleThreadWait(
        object, timeout, wait_mode, wait_reason, alertable, swappable, due_time, hand,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KxDelayThreadWait(
    interval: *mut _LARGE_INTEGER,
    wait_mode: KPROCESSOR_MODE,
    swappable: *mut ntkrnl_sys::base::BOOLEAN,
    due_time: *mut _LARGE_INTEGER,
    hand: *mut ntkrnl_sys::base::ULONG,
) {
    ntkrnl_sys::inline::ke::KxDelayThreadWait(interval, wait_mode, swappable, due_time, hand)
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

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn KeGetTrapFramePc(
    trap_frame: PKTRAP_FRAME,
) -> ntkrnl_sys::base::ULONG_PTR {
    ntkrnl_sys::inline::ke::KeGetTrapFramePc(trap_frame)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn KiRundownThread(thread: PKTHREAD) {
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
    ntkrnl_sys::inline::ke::InterlockedBitTestAndSet(base, bit)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InterlockedBitTestAndReset(base: *mut i32, bit: i32) -> ntkrnl_sys::base::BOOLEAN {
    ntkrnl_sys::inline::ke::InterlockedBitTestAndReset(base, bit)
}

// ============================================================================
// KE subsystem exports — amd64 intrinsics (was: ke/ke_rust_shims.c amd64 section)
// ============================================================================

#[cfg(target_arch = "x86_64")]
mod ke_amd64_exports {
    use ntkrnl_sys::ddk::*;

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
        ntkrnl_sys::inline::ke_amd64::__lidt(source)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __lgdt(source: ntkrnl_sys::base::PVOID) {
        ntkrnl_sys::inline::ke_amd64::__lgdt(source)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __sidt(dest: ntkrnl_sys::base::PVOID) {
        ntkrnl_sys::inline::ke_amd64::__sidt(dest)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __sgdt(dest: ntkrnl_sys::base::PVOID) {
        ntkrnl_sys::inline::ke_amd64::__sgdt(dest)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __sldt(ldtr: *mut u16) {
        ntkrnl_sys::inline::ke_amd64::__sldt(ldtr)
    }

    #[no_mangle]
    pub unsafe extern "C" fn __str(tr: *mut u16) {
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
        ntkrnl_sys::inline::ke_amd64::KeRegisterInterruptHandler(vector, handler)
    }

    #[no_mangle]
    pub unsafe extern "C" fn KiGetGdtEntry(gdt: ntkrnl_sys::base::PVOID, selector: u16) -> *mut _KGDTENTRY64 {
        ntkrnl_sys::inline::ke_amd64::KiGetGdtEntry(gdt, selector)
    }

    #[no_mangle]
    pub unsafe extern "C" fn KiGetGdtDescriptorBase(entry: *mut _KGDTENTRY64) -> ntkrnl_sys::base::PVOID {
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
        ntkrnl_sys::inline::ke_amd64::KiInitGdtEntry(entry, base, size, type_, dpl)
    }

    #[no_mangle]
    pub unsafe extern "C" fn KiSetGdtDescriptorBase(entry: *mut _KGDTENTRY64, base: u64) {
        ntkrnl_sys::inline::ke_amd64::KiSetGdtDescriptorBase(entry, base)
    }

    #[no_mangle]
    pub unsafe extern "C" fn KiSetGdtDescriptorLimit(entry: *mut _KGDTENTRY64, limit: ntkrnl_sys::base::ULONG) {
        ntkrnl_sys::inline::ke_amd64::KiSetGdtDescriptorLimit(entry, limit)
    }

    #[no_mangle]
    pub unsafe extern "C" fn KiSaveXState(buffer: ntkrnl_sys::base::PVOID, component_mask: u64) {
        ntkrnl_sys::inline::ke_amd64::KiSaveXState(buffer, component_mask)
    }

    #[no_mangle]
    pub unsafe extern "C" fn KiRestoreXState(buffer: ntkrnl_sys::base::PVOID, component_mask: u64) {
        ntkrnl_sys::inline::ke_amd64::KiRestoreXState(buffer, component_mask)
    }

    #[no_mangle]
    pub unsafe extern "C" fn KiSendEOI() {
        ntkrnl_sys::inline::ke_amd64::KiSendEOI()
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFillMemoryUlong(dest: ntkrnl_sys::base::PVOID, length: usize, pattern: ntkrnl_sys::base::ULONG) {
        ntkrnl_sys::inline::ke_amd64::RtlFillMemoryUlong(dest, length, pattern)
    }
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

    #[no_mangle]
    #[inline(never)]
    pub unsafe extern "C" fn KiGetGdtEntry(
        gdt: ntkrnl_sys::base::PVOID,
        selector: u16,
    ) -> *mut _KGDTENTRY {
        ntkrnl_sys::inline::ke_i386::KiGetGdtEntry(gdt, selector)
    }

    #[no_mangle]
    #[inline(never)]
    pub unsafe extern "C" fn KiSetGdtDescriptorBase(
        entry: *mut _KGDTENTRY,
        base: ntkrnl_sys::base::ULONG_PTR,
    ) {
        ntkrnl_sys::inline::ke_i386::KiSetGdtDescriptorBase(entry, base)
    }

    #[no_mangle]
    #[inline(never)]
    pub unsafe extern "C" fn RtlFillMemoryUlong(
        dest: ntkrnl_sys::base::PVOID,
        length: usize,
        pattern: ntkrnl_sys::base::ULONG,
    ) {
        ntkrnl_sys::inline::ke_i386::RtlFillMemoryUlong(dest, length, pattern)
    }
}

// ============================================================================
// ARM64_UART_TRACE stubs (C macros → no-op functions for Rust callers)
// ============================================================================

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ARM64_UART_TRACE(_msg: *const u8) {
    // No-op: ARM64_UART_TRACE is a C macro that expands to no-op on non-ARM64.
    // On ARM64, actual UART tracing is done via volatile MMIO in C.
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ARM64_UART_TRACE_HEX(_prefix: *const u8, _val: ntkrnl_sys::base::ULONG_PTR) {
    // No-op stub.
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

    // PsGetCurrentThread: now extern "system" = stdcall = _PsGetCurrentThread@0 (primary)
    // Alias cdecl name for any C code that uses cdecl calling convention
    ".globl _PsGetCurrentThread",
    ".set _PsGetCurrentThread, _PsGetCurrentThread@0",

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
    ntkrnl_sys::inline::tier1::KeRaiseIrql(12, &mut old_irql); // SYNCH_LEVEL
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
    ntkrnl_sys::inline::tier1::KeRaiseIrql(12, &mut (*LockHandle).OldIrql); // SYNCH_LEVEL
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
    ntkrnl_sys::inline::tier1::KeRaiseIrql(12, OldIrql); // SYNCH_LEVEL
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

