//! ddk_shims.rs — the DDK/WDF macros + FORCEINLINEs the cdrom C calls as if they were
//! exported functions. They have no kernel export symbol (header inlines/macros), so the
//! cohab link needs the crate to provide them. Kernel-inline impls mirror the battle-tested
//! classpnp-rust/src/ddk_shims.rs (boots ×3). WDF *_INIT macros are 1:1 with <wdf.h> (zero +
//! Size [+ fields]). WDF typed-context accessors route through the kmdf-sys dispatch helper
//! using the WDF_DECLARE_CONTEXT_TYPE type-info globals (defined by data.c). WPP/Trace are
//! no-ops (telemetry, not boot-critical).
#![allow(non_snake_case, unused_variables, dead_code)]
use ntkrnl_sys::ddk::*;
use kmdf_sys::*;
use crate::types::*;
use core::ffi::c_void;

// ============================ IRP stack inlines (mirror classpnp) ============================
#[no_mangle]
pub unsafe extern "C" fn IoGetNextIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION {
    (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation.offset(-1)
}
#[no_mangle]
pub unsafe extern "C" fn IoSetCompletionRoutine(
    Irp: PIRP,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    Context: PVOID,
    Success: u8,
    Error: u8,
    Cancel: u8,
) {
    let stk = IoGetNextIrpStackLocation(Irp);
    (*stk).CompletionRoutine = CompletionRoutine;
    (*stk).Context = Context;
    let mut ctl: u8 = 0;
    if Success != 0 { ctl |= 0x40; } // SL_INVOKE_ON_SUCCESS
    if Error != 0 { ctl |= 0x80; } // SL_INVOKE_ON_ERROR
    if Cancel != 0 { ctl |= 0x20; } // SL_INVOKE_ON_CANCEL
    (*stk).Control = ctl;
}

// aarch64-windows LLVM codegen emits a reference to the *user-mode-named*
// `IsProcessorFeaturePresent` (kernel mode only exports the `Ex`-prefixed form);
// cdrom-rust trips it while fastfat/scsiport/etc. don't, so cdrom.sys fails its
// arm64 link with `undefined symbol: IsProcessorFeaturePresent`. i386/amd64
// codegen does not emit the reference (cdrom.sys links clean there). Forward it
// to the real kernel export so the feature query stays faithful. arm64-only.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "system" fn IsProcessorFeaturePresent(feature: ULONG) -> i32 {
    ExIsProcessorFeaturePresent(feature) as i32
}

// ===================== IRQL / tick (cfg-gated like classpnp: real exports on i386) =====================
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn KeGetCurrentIrql() -> u8 {
    let irql: u64;
    core::arch::asm!("mov {}, cr8", out(reg) irql, options(nomem, nostack, preserves_flags));
    irql as u8
}
// KeQueryTickCount: a real export on i386, an inline on amd64/arm64. KI_USER_SHARED_DATA is at
// the fixed VA 0xFFFFF78000000000 on both; TickCount (KSYSTEM_TIME) at +0x320. Lockless read.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn KeQueryTickCount(Count: *mut LARGE_INTEGER) {
    const LOW: usize = 0xFFFFF78000000000 + 0x320;
    loop {
        let high1 = core::ptr::read_volatile((LOW + 4) as *const i32);
        let low = core::ptr::read_volatile(LOW as *const u32);
        let high2 = core::ptr::read_volatile((LOW + 8) as *const i32);
        if high1 == high2 {
            (*Count).QuadPart = ((high1 as i64) << 32) | (low as i64 & 0xFFFF_FFFF);
            return;
        }
    }
}

// ============================ C macros c2rust emitted as extern fns ============================
#[no_mangle]
pub unsafe extern "C" fn FREE_POOL(Ptr: *mut c_void) {
    if !Ptr.is_null() {
        ExFreePool(Ptr);
    }
}
#[no_mangle]
pub unsafe extern "system" fn PagedCode() {}
#[no_mangle]
pub unsafe extern "C" fn BootEnvironmentIsWinPE() -> BOOLEAN {
    0 // FALSE — cdrom is not running under WinPE in our boot path
}
#[no_mangle]
pub unsafe extern "system" fn WPP_CLEANUP(_DriverObject: PDRIVER_OBJECT) {}
#[no_mangle]
pub unsafe extern "system" fn WPP_INIT_TRACING(
    _DriverObject: PDRIVER_OBJECT,
    _RegistryPath: PUNICODE_STRING,
) {
}
// TracePrint is a macro now (see lib.rs) — no C trampoline, no _v twin.
// EXCLUSIVE_MODE(_CdData) := (_CdData->ExclusiveOwner != NULL)
#[no_mangle]
pub unsafe extern "C" fn EXCLUSIVE_MODE(CdData: *mut CDROM_DATA) -> i32 {
    (!(*CdData).ExclusiveOwner.is_null()) as i32
}
// EXCLUSIVE_OWNER(_CdData, _FileObject) := (_CdData->ExclusiveOwner == _FileObject)
#[no_mangle]
pub unsafe extern "C" fn EXCLUSIVE_OWNER(CdData: *mut CDROM_DATA, FileObject: *mut c_void) -> u8 {
    ((*CdData).ExclusiveOwner == FileObject as WDFFILEOBJECT) as u8
}

// ============================ WDF *_INIT struct-init macros (<wdf.h>) ============================
#[no_mangle]
pub unsafe extern "system" fn WDF_DRIVER_CONFIG_INIT(
    Config: *mut WDF_DRIVER_CONFIG,
    EvtDriverDeviceAdd: PFN_WDF_DRIVER_DEVICE_ADD,
) {
    core::ptr::write_bytes(Config as *mut u8, 0, core::mem::size_of::<WDF_DRIVER_CONFIG>());
    (*Config).Size = core::mem::size_of::<WDF_DRIVER_CONFIG>() as u32;
    (*Config).EvtDriverDeviceAdd = EvtDriverDeviceAdd;
}
#[no_mangle]
pub unsafe extern "system" fn WDF_PNPPOWER_EVENT_CALLBACKS_INIT(
    Callbacks: *mut WDF_PNPPOWER_EVENT_CALLBACKS,
) {
    core::ptr::write_bytes(
        Callbacks as *mut u8,
        0,
        core::mem::size_of::<WDF_PNPPOWER_EVENT_CALLBACKS>(),
    );
    (*Callbacks).Size = core::mem::size_of::<WDF_PNPPOWER_EVENT_CALLBACKS>() as u32;
}
#[no_mangle]
pub unsafe extern "system" fn WDF_REQUEST_PARAMETERS_INIT(Parameters: *mut WDF_REQUEST_PARAMETERS) {
    core::ptr::write_bytes(
        Parameters as *mut u8,
        0,
        core::mem::size_of::<WDF_REQUEST_PARAMETERS>(),
    );
    (*Parameters).Size = core::mem::size_of::<WDF_REQUEST_PARAMETERS>() as u16;
}
// WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE = WDF_OBJECT_ATTRIBUTES_INIT + set ContextTypeInfo.
// INIT: zero, Size, ExecutionLevel=WdfExecutionLevelInheritFromParent(1),
//       SynchronizationScope=WdfSynchronizationScopeInheritFromParent(1).
#[no_mangle]
pub unsafe extern "system" fn WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
    Attributes: *mut WDF_OBJECT_ATTRIBUTES,
    ContextType: PCWDF_OBJECT_CONTEXT_TYPE_INFO,
) {
    core::ptr::write_bytes(
        Attributes as *mut u8,
        0,
        core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>(),
    );
    (*Attributes).Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;
    (*Attributes).ExecutionLevel = 1; // WdfExecutionLevelInheritFromParent
    (*Attributes).SynchronizationScope = 1; // WdfSynchronizationScopeInheritFromParent
    (*Attributes).ContextTypeInfo = (*ContextType).UniqueType;
}

// ===================== WDF typed-context type-info globals (WDF_DECLARE_CONTEXT_TYPE) =====================
// cdrom.c instantiated these via WDF_DECLARE_CONTEXT_TYPE_WITH_NAME; that .c is now Rust, so
// define them here. Layout: Size, ContextName, ContextSize (the context struct's size),
// UniqueType (self-pointer — WDF identifies a context type by this address),
// EvtDriverGetUniqueContextType (NULL).
// WDF_OBJECT_CONTEXT_TYPE_INFO holds raw pointers (not Sync); a `static` requires Sync. Wrap in a
// repr(transparent) newtype with a manual Sync impl — the type-info is read-only after image load.
#[repr(transparent)]
pub struct CtxTypeInfo(pub WDF_OBJECT_CONTEXT_TYPE_INFO);
unsafe impl Sync for CtxTypeInfo {}
#[no_mangle]
pub static CDROM_DEVICE_EXTENSION_TYPE_INFO: CtxTypeInfo = CtxTypeInfo(WDF_OBJECT_CONTEXT_TYPE_INFO {
    Size: core::mem::size_of::<WDF_OBJECT_CONTEXT_TYPE_INFO>() as u32,
    ContextName: b"CDROM_DEVICE_EXTENSION\0".as_ptr() as PCHAR,
    ContextSize: core::mem::size_of::<CDROM_DEVICE_EXTENSION>(),
    UniqueType: &CDROM_DEVICE_EXTENSION_TYPE_INFO as *const CtxTypeInfo as PCWDF_OBJECT_CONTEXT_TYPE_INFO,
    EvtDriverGetUniqueContextType: None,
});
// Driver-extension context type-info. cdrom.c declared this via WDF_DECLARE_CONTEXT_TYPE_WITH_NAME
// (symbol _WDF_CDROM_DRIVER_EXTENSION_TYPE_INFO) — now that cdrom.c is Rust, define it here with the
// correct ContextSize so WdfDriverCreate allocates the right driver-extension size and
// DriverGetExtension retrieves it by the same UniqueType identity.
#[no_mangle]
pub static CDROM_DRIVER_EXTENSION_TYPE_INFO: CtxTypeInfo = CtxTypeInfo(WDF_OBJECT_CONTEXT_TYPE_INFO {
    Size: core::mem::size_of::<WDF_OBJECT_CONTEXT_TYPE_INFO>() as u32,
    ContextName: b"CDROM_DRIVER_EXTENSION\0".as_ptr() as PCHAR,
    ContextSize: core::mem::size_of::<CDROM_DRIVER_EXTENSION>(),
    UniqueType: &CDROM_DRIVER_EXTENSION_TYPE_INFO as *const CtxTypeInfo as PCWDF_OBJECT_CONTEXT_TYPE_INFO,
    EvtDriverGetUniqueContextType: None,
});
#[no_mangle]
pub static CDROM_REQUEST_CONTEXT_TYPE_INFO: CtxTypeInfo = CtxTypeInfo(WDF_OBJECT_CONTEXT_TYPE_INFO {
    Size: core::mem::size_of::<WDF_OBJECT_CONTEXT_TYPE_INFO>() as u32,
    ContextName: b"CDROM_REQUEST_CONTEXT\0".as_ptr() as PCHAR,
    ContextSize: core::mem::size_of::<CDROM_REQUEST_CONTEXT>(),
    UniqueType: &CDROM_REQUEST_CONTEXT_TYPE_INFO as *const CtxTypeInfo as PCWDF_OBJECT_CONTEXT_TYPE_INFO,
    EvtDriverGetUniqueContextType: None,
});
// init.rs (translated from cdrom.c) references the CDROM_REQUEST_CONTEXT type-info under the
// canonical WDF macro-expanded symbol `WdfObjectContextTypeInfo_CDROM_REQUEST_CONTEXT` and under
// the bare `CDROM_REQUEST_CONTEXT` name (WDF_DECLARE_CONTEXT_TYPE_WITH_NAME emits
// `#define CDROM_REQUEST_CONTEXT &WdfObjectContextTypeInfo_CDROM_REQUEST_CONTEXT`). Both must be
// the SAME context identity as the lookup path (RequestGetContext, above) which uses
// CDROM_REQUEST_CONTEXT_TYPE_INFO. WDF compares the `UniqueType` self-pointer, so we make all
// three structs carry the *same* UniqueType (the canonical CDROM_REQUEST_CONTEXT_TYPE_INFO
// address) — that keeps registration (init.rs) and lookup (RequestGetContext) matched.
#[no_mangle]
pub static WdfObjectContextTypeInfo_CDROM_REQUEST_CONTEXT: CtxTypeInfo = CtxTypeInfo(WDF_OBJECT_CONTEXT_TYPE_INFO {
    Size: core::mem::size_of::<WDF_OBJECT_CONTEXT_TYPE_INFO>() as u32,
    ContextName: b"CDROM_REQUEST_CONTEXT\0".as_ptr() as PCHAR,
    ContextSize: core::mem::size_of::<CDROM_REQUEST_CONTEXT>(),
    UniqueType: &CDROM_REQUEST_CONTEXT_TYPE_INFO as *const CtxTypeInfo as PCWDF_OBJECT_CONTEXT_TYPE_INFO,
    EvtDriverGetUniqueContextType: None,
});
#[no_mangle]
pub static CDROM_REQUEST_CONTEXT: CtxTypeInfo = CtxTypeInfo(WDF_OBJECT_CONTEXT_TYPE_INFO {
    Size: core::mem::size_of::<WDF_OBJECT_CONTEXT_TYPE_INFO>() as u32,
    ContextName: b"CDROM_REQUEST_CONTEXT\0".as_ptr() as PCHAR,
    ContextSize: core::mem::size_of::<CDROM_REQUEST_CONTEXT>(),
    UniqueType: &CDROM_REQUEST_CONTEXT_TYPE_INFO as *const CtxTypeInfo as PCWDF_OBJECT_CONTEXT_TYPE_INFO,
    EvtDriverGetUniqueContextType: None,
});
#[no_mangle]
pub static FILE_OBJECT_CONTEXT_TYPE_INFO: CtxTypeInfo = CtxTypeInfo(WDF_OBJECT_CONTEXT_TYPE_INFO {
    Size: core::mem::size_of::<WDF_OBJECT_CONTEXT_TYPE_INFO>() as u32,
    ContextName: b"FILE_OBJECT_CONTEXT\0".as_ptr() as PCHAR,
    ContextSize: core::mem::size_of::<FILE_OBJECT_CONTEXT>(),
    UniqueType: &FILE_OBJECT_CONTEXT_TYPE_INFO as *const CtxTypeInfo as PCWDF_OBJECT_CONTEXT_TYPE_INFO,
    EvtDriverGetUniqueContextType: None,
});
#[no_mangle]
pub unsafe extern "system" fn DeviceGetExtension(Device: WDFDEVICE) -> PCDROM_DEVICE_EXTENSION {
    WdfObjectGetTypedContext(Device as WDFOBJECT, &CDROM_DEVICE_EXTENSION_TYPE_INFO.0)
        as PCDROM_DEVICE_EXTENSION
}
#[no_mangle]
pub unsafe extern "system" fn RequestGetContext(Request: WDFREQUEST) -> *mut CDROM_REQUEST_CONTEXT {
    WdfObjectGetTypedContext(Request as WDFOBJECT, &CDROM_REQUEST_CONTEXT_TYPE_INFO.0)
        as *mut CDROM_REQUEST_CONTEXT
}
#[no_mangle]
pub unsafe extern "system" fn FileObjectGetContext(
    FileObject: WDFFILEOBJECT,
) -> *mut FILE_OBJECT_CONTEXT {
    WdfObjectGetTypedContext(FileObject as WDFOBJECT, &FILE_OBJECT_CONTEXT_TYPE_INFO.0)
        as *mut FILE_OBJECT_CONTEXT
}
#[no_mangle]
pub unsafe extern "system" fn DriverGetExtension(Driver: WDFDRIVER) -> *mut CDROM_DRIVER_EXTENSION {
    WdfObjectGetTypedContext(Driver as WDFOBJECT, &CDROM_DRIVER_EXTENSION_TYPE_INFO.0)
        as *mut CDROM_DRIVER_EXTENSION
}

// ============================ more WDF *_INIT macros ============================
#[no_mangle]
pub unsafe extern "system" fn WDF_OBJECT_ATTRIBUTES_INIT(Attributes: *mut WDF_OBJECT_ATTRIBUTES) {
    core::ptr::write_bytes(Attributes as *mut u8, 0, core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>());
    (*Attributes).Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;
    (*Attributes).ExecutionLevel = 1;       // WdfExecutionLevelInheritFromParent
    (*Attributes).SynchronizationScope = 1; // WdfSynchronizationScopeInheritFromParent
}
unsafe fn io_queue_config_init(Config: *mut WDF_IO_QUEUE_CONFIG, DispatchType: i32, default_queue: u8) {
    core::ptr::write_bytes(Config as *mut u8, 0, core::mem::size_of::<WDF_IO_QUEUE_CONFIG>());
    (*Config).Size = core::mem::size_of::<WDF_IO_QUEUE_CONFIG>() as u32;
    (*Config).PowerManaged = 2; // WdfUseDefault
    (*Config).DefaultQueue = default_queue;
    (*Config).DispatchType = DispatchType as u32;
    if DispatchType == 2 { // WdfIoQueueDispatchParallel
        (*Config).Settings.Parallel.as_mut().NumberOfPresentedRequests = u32::MAX;
    }
}
#[no_mangle]
pub unsafe extern "system" fn WDF_IO_QUEUE_CONFIG_INIT(Config: *mut WDF_IO_QUEUE_CONFIG, DispatchType: i32) {
    io_queue_config_init(Config, DispatchType, 0);
}
#[no_mangle]
pub unsafe extern "system" fn WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE(Config: *mut WDF_IO_QUEUE_CONFIG, DispatchType: i32) {
    io_queue_config_init(Config, DispatchType, 1);
}
#[no_mangle]
pub unsafe extern "system" fn WDF_FILEOBJECT_CONFIG_INIT(
    Config: *mut WDF_FILEOBJECT_CONFIG,
    EvtDeviceFileCreate: PFN_WDF_DEVICE_FILE_CREATE,
    EvtFileClose: PFN_WDF_FILE_CLOSE,
    EvtFileCleanup: PFN_WDF_FILE_CLEANUP,
) {
    core::ptr::write_bytes(Config as *mut u8, 0, core::mem::size_of::<WDF_FILEOBJECT_CONFIG>());
    (*Config).Size = core::mem::size_of::<WDF_FILEOBJECT_CONFIG>() as u32;
    (*Config).EvtDeviceFileCreate = EvtDeviceFileCreate;
    (*Config).EvtFileClose = EvtFileClose;
    (*Config).EvtFileCleanup = EvtFileCleanup;
    (*Config).AutoForwardCleanupClose = 2; // WdfUseDefault
}
#[no_mangle]
pub unsafe extern "system" fn WDF_WORKITEM_CONFIG_INIT(Config: *mut WDF_WORKITEM_CONFIG, EvtWorkItemFunc: PFN_WDF_WORKITEM) {
    core::ptr::write_bytes(Config as *mut u8, 0, core::mem::size_of::<WDF_WORKITEM_CONFIG>());
    (*Config).Size = core::mem::size_of::<WDF_WORKITEM_CONFIG>() as u32;
    (*Config).EvtWorkItemFunc = EvtWorkItemFunc;
    (*Config).AutomaticSerialization = 1; // TRUE
}
#[no_mangle]
pub unsafe extern "system" fn WDF_REMOVE_LOCK_OPTIONS_INIT(Options: *mut WDF_REMOVE_LOCK_OPTIONS, Flags: u32) {
    core::ptr::write_bytes(Options as *mut u8, 0, core::mem::size_of::<WDF_REMOVE_LOCK_OPTIONS>());
    (*Options).Size = core::mem::size_of::<WDF_REMOVE_LOCK_OPTIONS>() as u32;
    (*Options).Flags = Flags;
}
#[no_mangle]
pub unsafe extern "system" fn WDF_IO_TARGET_OPEN_PARAMS_INIT_EXISTING_DEVICE(
    Params: *mut WDF_IO_TARGET_OPEN_PARAMS,
    DeviceObject: PDEVICE_OBJECT,
) {
    core::ptr::write_bytes(Params as *mut u8, 0, core::mem::size_of::<WDF_IO_TARGET_OPEN_PARAMS>());
    (*Params).Size = core::mem::size_of::<WDF_IO_TARGET_OPEN_PARAMS>() as u32;
    (*Params).Type = 1; // WdfIoTargetOpenUseExistingDevice (WDF_IO_TARGET_OPEN_TYPE: 0=Undef,1=UseExisting,2=ByName,3=Reopen)
    (*Params).TargetDeviceObject = DeviceObject;
}

// ============================ more C macros + small inlines ============================
#[no_mangle]
pub unsafe extern "C" fn NT_SUCCESS(Status: NTSTATUS) -> bool { Status >= 0 }
#[no_mangle]
pub unsafe extern "C" fn IoGetCurrentIrpStackLocation(Irp: PIRP) -> PIO_STACK_LOCATION {
    (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation
}
#[no_mangle]
pub unsafe extern "C" fn DEVICE_TYPE_FROM_CTL_CODE(ControlCode: u32) -> u32 { (ControlCode >> 16) & 0xFFFF }
// PORT_ALLOCATED_SENSE / RequestGetCurrentStackLocationFlags: diagnostic/flag accessors; the
// conservative value (0 / no-flags) is safe — sense buffers aren't port-allocated in our path.
// ARITY: the scsi.h macro is PORT_ALLOCATED_SENSE(devExt, srb) and both call sites pass two
// arguments (autorun.rs:973, and the decls in autorun.rs/common.rs spell two). This shim took
// ONE, so the callee bound `Srb` to the DeviceExtension. Harmless only because the body ignores
// its parameter — but it is an arity mismatch in live driver code, and any future body would
// have read the wrong pointer. Signature now matches the declarations.
#[no_mangle]
pub unsafe extern "C" fn PORT_ALLOCATED_SENSE(
    _DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    _Srb: *mut SCSI_REQUEST_BLOCK,
) -> u8 {
    0
}
#[no_mangle]
pub unsafe extern "system" fn RequestGetCurrentStackLocationFlags(_Request: WDFREQUEST) -> u8 { 0 }
// RtlStringCchPrintfW: ntstrsafe inline (no export). Stub returns STATUS_SUCCESS; building the
// formatted device-interface ref-string is non-fatal to boot (the GUID path still resolves).
// TODO: real swprintf impl for correct symbolic-link suffixes.
#[no_mangle]
pub unsafe extern "C" fn RtlStringCchPrintfW(_Dst: *mut u16, _CchDst: usize, _Format: *const u16) -> NTSTATUS { 0 }
// ===== translated scratch.h FORCEINLINEs the c2rust pass left declared-not-defined =====
// ScratchBuffer_ExecuteCdb := ExecuteCdbEx(.., TimeoutValue=0) (scratch.h FORCEINLINE).
#[no_mangle]
pub unsafe extern "system" fn ScratchBuffer_ExecuteCdb(
    DeviceExtension: PCDROM_DEVICE_EXTENSION,
    OriginalRequest: WDFREQUEST,
    TransferSize: u32,
    GetDataFromDevice: u8,
    Cdb: PCDB,
    OprationLength: u8,
) -> NTSTATUS {
    crate::scratch::ScratchBuffer_ExecuteCdbEx(
        DeviceExtension,
        OriginalRequest as *mut c_void,
        TransferSize,
        GetDataFromDevice,
        Cdb as *mut CDB,
        OprationLength,
        0,
    )
}
// ScratchBuffer_BeginUse / EndUse: the scratch-buffer "soft lock" (NOT atomic per the C comment).
#[no_mangle]
pub unsafe extern "system" fn ScratchBuffer_BeginUse(DeviceExtension: PCDROM_DEVICE_EXTENSION) {
    (*DeviceExtension).ScratchContext.ScratchInUse = usize::MAX as PVOID; // mark in-use
    (*DeviceExtension).ScratchContext.ScratchInUseFileName = core::ptr::null();
    (*DeviceExtension).ScratchContext.ScratchInUseLineNumber = u32::MAX;
    crate::scratch::ScratchBuffer_ResetItems(DeviceExtension, 1);
    crate::common::RequestClearSendTime((*DeviceExtension).ScratchContext.ScratchRequest);
}
#[no_mangle]
pub unsafe extern "system" fn ScratchBuffer_EndUse(DeviceExtension: PCDROM_DEVICE_EXTENSION) {
    (*DeviceExtension).ScratchContext.ScratchInUseFileName = core::ptr::null();
    (*DeviceExtension).ScratchContext.ScratchInUseLineNumber = 0;
    if (*DeviceExtension).ScratchContext.PartialMdlIsBuilt != 0 {
        MmPrepareMdlForReuse((*DeviceExtension).ScratchContext.PartialMdl);
        (*DeviceExtension).ScratchContext.PartialMdlIsBuilt = 0;
    }
    (*DeviceExtension).ScratchContext.ScratchInUse = core::ptr::null_mut(); // release
}
// IsVolumeMounted := (DeviceObject->Vpb->Flags & VPB_MOUNTED)
#[no_mangle]
pub unsafe extern "C" fn IsVolumeMounted(DeviceObject: PDEVICE_OBJECT) -> BOOLEAN {
    let vpb = (*DeviceObject).Vpb;
    (!vpb.is_null() && ((*vpb).Flags & 0x1 /* VPB_MOUNTED */) != 0) as BOOLEAN
}
// CompareStorageDuids: its only caller feeds the result to a no-op NT_ASSERT, so the value is
// discarded — return DuidExactMatch(0). (Full sub-id comparison is unnecessary for boot.)
#[no_mangle]
pub unsafe extern "C" fn CompareStorageDuids(
    _Duid1: *const STORAGE_DEVICE_UNIQUE_IDENTIFIER,
    _Duid2: *const STORAGE_DEVICE_UNIQUE_IDENTIFIER,
) -> i32 {
    0 // DuidExactMatch
}

// ============================ mechanical DDK/CRT macro + intrinsic glue ============================
const STATUS_INTEGER_OVERFLOW_: NTSTATUS = 0xC0000095u32 as NTSTATUS;
#[no_mangle] pub unsafe extern "C" fn REVERSE_BYTES(dest: *mut u32, src: *const u8) {
    *dest = u32::from_be_bytes([*src, *src.add(1), *src.add(2), *src.add(3)]);
}
#[no_mangle] pub unsafe extern "C" fn REVERSE_BYTES_SHORT(dest: *mut c_void, src: *const c_void) {
    let s = src as *const u8; let d = dest as *mut u8;
    *d = *s.add(1); *d.add(1) = *s;
}
#[no_mangle] pub unsafe extern "C" fn REVERSE_LONG(p: *mut u32) { *p = (*p).swap_bytes(); }
#[no_mangle] pub unsafe extern "C" fn REVERSE_SHORT(p: *mut u16) { *p = (*p).swap_bytes(); }
#[no_mangle] pub unsafe extern "C" fn MSF_TO_LBA(m: u8, s: u8, f: u8) -> u32 {
    (m as u32) * 60 * 75 + (s as u32) * 75 + (f as u32) - 150
}
#[no_mangle] pub unsafe extern "system" fn TEST_FLAG(flags: u32, flag: u32) -> i32 { ((flags & flag) != 0) as i32 }
#[no_mangle] pub unsafe extern "C" fn WHICH_BIT(bps: u32, shift: *mut u8) { *shift = bps.trailing_zeros() as u8; }
#[no_mangle] pub unsafe extern "C" fn CountOfSetBitsUChar(byte: u8) -> u8 { byte.count_ones() as u8 }
#[no_mangle] pub unsafe extern "C" fn Int64ShrlMod32(value: i64, shift: u32) -> i64 { ((value as u64) >> shift) as i64 }
#[no_mangle] pub unsafe extern "C" fn InterlockedCompareExchange(Destination: *mut i32, Exchange: i32, Comperand: i32) -> i32 {
    let a = &*(Destination as *const core::sync::atomic::AtomicI32);
    match a.compare_exchange(Comperand, Exchange, core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst) {
        Ok(v) => v, Err(v) => v,
    }
}
#[no_mangle] pub unsafe extern "C" fn IoSetNextIrpStackLocation(Irp: PIRP) {
    let p = &mut (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation;
    *p = (*p).offset(-1);
    (*Irp).CurrentLocation = ((*Irp).CurrentLocation as i8 - 1) as i8;
}
#[no_mangle] pub unsafe extern "system" fn MmGetMdlByteCount(Mdl: PMDL) -> u32 { (*Mdl).ByteCount }
#[no_mangle] pub unsafe extern "system" fn MmGetMdlVirtualAddress(Mdl: PMDL) -> *mut c_void {
    ((*Mdl).StartVa as *mut u8).add((*Mdl).ByteOffset as usize) as *mut c_void
}
#[no_mangle] pub unsafe extern "C" fn RtlULongAdd(ul1: u32, ul2: u32, result: *mut u32) -> NTSTATUS {
    match ul1.checked_add(ul2) { Some(s) => { *result = s; 0 } None => STATUS_INTEGER_OVERFLOW_ }
}
#[no_mangle] pub unsafe extern "C" fn RtlULongSub(a: u32, b: u32, result: *mut u32) -> i32 {
    match a.checked_sub(b) { Some(s) => { *result = s; 0 } None => STATUS_INTEGER_OVERFLOW_ }
}
// ValidChar: c2rust device-name char validation — printable ASCII (approx; TODO confirm macro).
#[no_mangle] pub unsafe extern "C" fn ValidChar(c: u8) -> i32 { ((c >= 0x20) && (c < 0x7F)) as i32 }
// PLAY_ACTIVE(dev) := &dev->DeviceAdditionalData.PlayActive (lvalue pointer).
#[no_mangle] pub unsafe extern "C" fn PLAY_ACTIVE(DeviceExtension: PCDROM_DEVICE_EXTENSION) -> *mut i32 {
    core::ptr::addr_of_mut!((*DeviceExtension).DeviceAdditionalData.PlayActive) as *mut i32
}
// TimeOutValueGetCapValue: caps a per-request timeout; no-cap passthrough (TODO confirm cap policy).
#[no_mangle] pub unsafe extern "C" fn TimeOutValueGetCapValue(_timeOutValue: u32, value: u32) -> u32 { value }
#[no_mangle] pub static D3COLD_SUPPORT_INTERFACE_VERSION: u16 = 1;
#[no_mangle] pub unsafe extern "C" fn min(a: u32, b: u32) -> u32 { if a < b { a } else { b } }
#[no_mangle] pub unsafe extern "C" fn IoIsErrorUserInduced(Status: NTSTATUS) -> BOOLEAN {
    matches!(Status as u32, 0xC000009A|0xC0000010|0xC0000013|0xC0000022|0xC000000E|0xC0000056|0xC0000288|0xC000020C) as BOOLEAN
}
#[no_mangle] pub unsafe extern "system" fn __SEH2_ProbeAndLockPages(mdl: *mut MDL, mode: u32, access: u32) -> i32 {
    // Returns the SEH "exception code": callers do `if status != 0 { break }`, i.e. 0 = success,
    // non-zero = the code the C's _SEH2_EXCEPT would have caught. The status form of the probe
    // gives that code without a raise (which nothing here could catch).
    // Never return a success status other than 0: a returned 1 made DeviceSendSrbSynchronously
    // treat a successful MDL lock as an exception and bail before sending the SRB (st=1, SrbStatus
    // PENDING, DataTransferLength 0 → INQUIRY STATUS_DEVICE_PROTOCOL_ERROR → no SCSI commands ever run).
    let status = ntkrnl_sys::probe::MmTryProbeAndLockPages(mdl as _, mode as i8, access as _);
    if status < 0 { status } else { 0 }
}
// CLASSP_REG_* registry key/value names (wide strings). TODO confirm exact strings vs classpnp.
#[no_mangle] pub static CLASSP_REG_SUBKEY_NAME: [u16; 6] = [b'C' as u16, b'd' as u16, b'R' as u16, b'o' as u16, b'm' as u16, 0];
#[no_mangle] pub static CLASSP_REG_REMOVAL_POLICY_VALUE_NAME: [u16; 18] =
    [b'U' as u16,b's' as u16,b'e' as u16,b'r' as u16,b'R' as u16,b'e' as u16,b'm' as u16,b'o' as u16,b'v' as u16,b'a' as u16,b'l' as u16,b'P' as u16,b'o' as u16,b'l' as u16,b'i' as u16,b'c' as u16,b'y' as u16,0];

// ============================ final mechanical glue (whole-archive complete gap) ============================
#[no_mangle] pub unsafe extern "system" fn SET_FLAG(flags: *mut u32, flag: u32) { *flags |= flag; }
#[no_mangle] pub unsafe extern "C" fn SRB_STATUS(srb_status: u8) -> u32 { (srb_status & 0x3F) as u32 }
#[no_mangle] pub unsafe extern "C" fn IS_SCSIOP_READ(opcode: u8) -> u8 {
    matches!(opcode, 0x08 | 0x28 | 0xA8 | 0x88) as u8 // READ(6/10/12/16)
}
#[no_mangle] pub unsafe extern "C" fn IS_SCSIOP_READWRITE(opCode: u8) -> i32 {
    matches!(opCode, 0x08 | 0x28 | 0xA8 | 0x88 | 0x0A | 0x2A | 0xAA | 0x8A) as i32 // READ/WRITE(6/10/12/16)
}
#[no_mangle] pub unsafe extern "C" fn FREE_PORT_ALLOCATED_SENSE_BUFFER(_DeviceExtension: PCDROM_DEVICE_EXTENSION, _Srb: *mut SCSI_REQUEST_BLOCK) {}
#[no_mangle] pub unsafe extern "C" fn SECONDS_TO_100NS_UNITS(seconds: i32) -> i64 { (seconds as i64) * 10_000_000 }
#[no_mangle] pub unsafe extern "C" fn RTL_SIZEOF_THROUGH_FIELD_SENSE_DATA_AdditionalSenseCodeQualifier() -> usize {
    core::mem::offset_of!(SENSE_DATA, AdditionalSenseCodeQualifier) + 1
}
#[no_mangle] pub unsafe extern "C" fn max(x: i64, y: i64) -> i64 { if x > y { x } else { y } }
#[no_mangle] pub unsafe extern "C" fn InterlockedIncrement(p: *mut i32) -> i32 {
    (*(p as *const core::sync::atomic::AtomicI32)).fetch_add(1, core::sync::atomic::Ordering::SeqCst) + 1
}
// MmPrepareMdlForReuse: clear the partial-mapped flag so the MDL can be rebuilt (ntddk.h inline).
#[no_mangle] pub unsafe extern "C" fn MmPrepareMdlForReuse(Mdl: PMDL) {
    (*Mdl).MdlFlags &= !(0x0008i16); // ~MDL_PARTIAL_HAS_BEEN_MAPPED
}
// KeQuerySystemTime: KUSER_SHARED_DATA.SystemTime @ fixed VA +0x14, lockless read (amd64/arm64 inline).
#[cfg(not(target_arch = "x86"))]
#[no_mangle] pub unsafe extern "system" fn KeQuerySystemTime(CurrentTime: *mut LARGE_INTEGER) {
    const ST: usize = 0xFFFFF78000000000 + 0x14;
    loop {
        let high1 = core::ptr::read_volatile((ST + 4) as *const i32);
        let low = core::ptr::read_volatile(ST as *const u32);
        let high2 = core::ptr::read_volatile((ST + 8) as *const i32);
        if high1 == high2 { (*CurrentTime).QuadPart = ((high1 as i64) << 32) | (low as i64 & 0xFFFF_FFFF); return; }
    }
}
// KeRaiseIrql/KeLowerIrql: cr8 on amd64 (inlines; not exported). i386 has real exports.
#[cfg(target_arch = "x86_64")]
#[no_mangle] pub unsafe extern "system" fn KeRaiseIrql(NewIrql: u8, OldIrql: *mut u8) {
    let cur: u64;
    core::arch::asm!("mov {}, cr8", out(reg) cur, options(nomem, nostack, preserves_flags));
    *OldIrql = cur as u8;
    core::arch::asm!("mov cr8, {}", in(reg) NewIrql as u64, options(nomem, nostack, preserves_flags));
}
#[cfg(target_arch = "x86_64")]
#[no_mangle] pub unsafe extern "system" fn KeLowerIrql(NewIrql: u8) {
    core::arch::asm!("mov cr8, {}", in(reg) NewIrql as u64, options(nomem, nostack, preserves_flags));
}

// ============================ more WDF *_INIT macros ============================
#[no_mangle] pub unsafe extern "system" fn WDF_REQUEST_REUSE_PARAMS_INIT(Params: *mut WDF_REQUEST_REUSE_PARAMS, Flags: u32, Status: NTSTATUS) {
    core::ptr::write_bytes(Params as *mut u8, 0, core::mem::size_of::<WDF_REQUEST_REUSE_PARAMS>());
    (*Params).Size = core::mem::size_of::<WDF_REQUEST_REUSE_PARAMS>() as u32;
    (*Params).Flags = Flags;
    (*Params).Status = Status;
}
#[no_mangle] pub unsafe extern "system" fn WDF_REQUEST_SEND_OPTIONS_INIT(Options: *mut WDF_REQUEST_SEND_OPTIONS, Flags: u32) {
    core::ptr::write_bytes(Options as *mut u8, 0, core::mem::size_of::<WDF_REQUEST_SEND_OPTIONS>());
    (*Options).Size = core::mem::size_of::<WDF_REQUEST_SEND_OPTIONS>() as u32;
    (*Options).Flags = Flags;
}
#[no_mangle] pub unsafe extern "system" fn WDF_MEMORY_DESCRIPTOR_INIT_BUFFER(Descriptor: *mut WDF_MEMORY_DESCRIPTOR, Buffer: *mut c_void, BufferLength: u32) {
    core::ptr::write_bytes(Descriptor as *mut u8, 0, core::mem::size_of::<WDF_MEMORY_DESCRIPTOR>());
    (*Descriptor).Type = 1; // WdfMemoryDescriptorTypeBuffer
    let bt = (*Descriptor).u.BufferType.as_mut();
    bt.Buffer = Buffer;
    bt.Length = BufferLength;
}

// ============================ remaining IRP-stack + kernel inlines ============================
// (cdrom already defines IoGet{Current,Next}IrpStackLocation / IoSetNextIrpStackLocation /
//  IoSetCompletionRoutine above — these are the rest the cohab link still wants.)
#[no_mangle] pub unsafe extern "C" fn IoSkipCurrentIrpStackLocation(Irp: PIRP) {
    let p = &mut (*Irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation;
    *p = (*p).offset(1);
    (*Irp).CurrentLocation = ((*Irp).CurrentLocation as i8 + 1) as i8;
}
#[no_mangle] pub unsafe extern "C" fn IoCopyCurrentIrpStackLocationToNext(Irp: PIRP) {
    let cur = IoGetCurrentIrpStackLocation(Irp);
    let next = IoGetNextIrpStackLocation(Irp);
    // C macro copies FIELD_OFFSET(IO_STACK_LOCATION, CompletionRoutine) bytes — cross-arch
    // via offset_of (NOT classpnp's i386-only 0x24 hardcode), then clears Control.
    let n = core::mem::offset_of!(IO_STACK_LOCATION, CompletionRoutine);
    core::ptr::copy_nonoverlapping(cur as *const u8, next as *mut u8, n);
    (*next).Control = 0;
}
#[no_mangle] pub unsafe extern "C" fn IoMarkIrpPending(Irp: PIRP) {
    (*IoGetCurrentIrpStackLocation(Irp)).Control |= 0x01; // SL_PENDING_RETURNED
}
// KeInitializeSpinLock(lock) => *lock = 0
#[no_mangle] pub unsafe extern "C" fn KeInitializeSpinLock(SpinLock: *mut usize) { *SpinLock = 0; } // CC = ntkrnl-sys bindgen_i386 decl class (§9.99; one-liner form dodged the 07-18 sweep)

// c2rust panic/UB paths emit a call to libc `abort`; there is no CRT abort in the
// kernel. Halt (matches the crate panic_handler).
#[no_mangle] pub extern "C" fn abort() -> ! { loop {} }

// FORCEINLINE from cdrom.h: 64k-write time in 100ns units. 320,000,000 / SectorsPerSecond
// (no floats — kernel-safe). SectorsPerSecond is guaranteed >= 1 by the caller (_In_range_).
// FORCEINLINE (cdecl, not NTAPI) — callers declare it `extern "C"`, so define it
// `extern "C"` too (matters on i386 where system=stdcall != C=cdecl).
#[no_mangle] pub unsafe extern "C" fn ConvertSectorsPerSecondTo100nsUnitsFor64kWrite(SectorsPerSecond: u32) -> i64 {
    320_000_000i64 / SectorsPerSecond as i64
}
