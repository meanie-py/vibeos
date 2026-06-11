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

// ===================== IRQL / tick (cfg-gated like classpnp: real exports on i386) =====================
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentIrql() -> u8 {
    let irql: u64;
    core::arch::asm!("mov {}, cr8", out(reg) irql, options(nomem, nostack, preserves_flags));
    irql as u8
}
// KeQueryTickCount: a real export on i386, an inline on amd64/arm64. KI_USER_SHARED_DATA is at
// the fixed VA 0xFFFFF78000000000 on both; TickCount (KSYSTEM_TIME) at +0x320. Lockless read.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeQueryTickCount(Count: *mut LARGE_INTEGER) {
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
pub unsafe extern "C" fn PagedCode() {}
#[no_mangle]
pub unsafe extern "C" fn BootEnvironmentIsWinPE() -> BOOLEAN {
    0 // FALSE — cdrom is not running under WinPE in our boot path
}
#[no_mangle]
pub unsafe extern "C" fn WPP_CLEANUP(_DriverObject: PDRIVER_OBJECT) {}
#[no_mangle]
pub unsafe extern "C" fn WPP_INIT_TRACING(
    _DriverObject: PDRIVER_OBJECT,
    _RegistryPath: PUNICODE_STRING,
) {
}
// TracePrint is a C-variadic WPP macro; the callers use the cdecl variadic ABI (caller cleans),
// so a fixed-arity no-op definition resolves the symbol and is ABI-compatible. No-op = telemetry.
#[no_mangle]
pub unsafe extern "C" fn TracePrint(_level: u32, _flag: u32, _fmt: *const i8) {}
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
pub unsafe extern "C" fn WDF_DRIVER_CONFIG_INIT(
    Config: *mut WDF_DRIVER_CONFIG,
    EvtDriverDeviceAdd: PFN_WDF_DRIVER_DEVICE_ADD,
) {
    core::ptr::write_bytes(Config as *mut u8, 0, core::mem::size_of::<WDF_DRIVER_CONFIG>());
    (*Config).Size = core::mem::size_of::<WDF_DRIVER_CONFIG>() as u32;
    (*Config).EvtDriverDeviceAdd = EvtDriverDeviceAdd;
}
#[no_mangle]
pub unsafe extern "C" fn WDF_PNPPOWER_EVENT_CALLBACKS_INIT(
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
pub unsafe extern "C" fn WDF_REQUEST_PARAMETERS_INIT(Parameters: *mut WDF_REQUEST_PARAMETERS) {
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
pub unsafe extern "C" fn WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
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
// EvtDriverGetUniqueContextType (NULL). The driver-ext one stays defined elsewhere.
extern "C" {
    static _WDF_CDROM_DRIVER_EXTENSION_TYPE_INFO: WDF_OBJECT_CONTEXT_TYPE_INFO;
}
// WDF_OBJECT_CONTEXT_TYPE_INFO holds raw pointers (not Sync); a `static` requires Sync. Wrap in a
// repr(transparent) newtype with a manual Sync impl — the type-info is read-only after image load.
#[repr(transparent)]
struct CtxTypeInfo(WDF_OBJECT_CONTEXT_TYPE_INFO);
unsafe impl Sync for CtxTypeInfo {}
#[no_mangle]
pub static CDROM_DEVICE_EXTENSION_TYPE_INFO: CtxTypeInfo = CtxTypeInfo(WDF_OBJECT_CONTEXT_TYPE_INFO {
    Size: core::mem::size_of::<WDF_OBJECT_CONTEXT_TYPE_INFO>() as u32,
    ContextName: b"CDROM_DEVICE_EXTENSION\0".as_ptr() as PCHAR,
    ContextSize: core::mem::size_of::<CDROM_DEVICE_EXTENSION>(),
    UniqueType: &CDROM_DEVICE_EXTENSION_TYPE_INFO as *const CtxTypeInfo as PCWDF_OBJECT_CONTEXT_TYPE_INFO,
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
#[no_mangle]
pub static FILE_OBJECT_CONTEXT_TYPE_INFO: CtxTypeInfo = CtxTypeInfo(WDF_OBJECT_CONTEXT_TYPE_INFO {
    Size: core::mem::size_of::<WDF_OBJECT_CONTEXT_TYPE_INFO>() as u32,
    ContextName: b"FILE_OBJECT_CONTEXT\0".as_ptr() as PCHAR,
    ContextSize: core::mem::size_of::<FILE_OBJECT_CONTEXT>(),
    UniqueType: &FILE_OBJECT_CONTEXT_TYPE_INFO as *const CtxTypeInfo as PCWDF_OBJECT_CONTEXT_TYPE_INFO,
    EvtDriverGetUniqueContextType: None,
});
#[no_mangle]
pub unsafe extern "C" fn DeviceGetExtension(Device: WDFDEVICE) -> PCDROM_DEVICE_EXTENSION {
    WdfObjectGetTypedContext(Device as WDFOBJECT, &CDROM_DEVICE_EXTENSION_TYPE_INFO.0)
        as PCDROM_DEVICE_EXTENSION
}
#[no_mangle]
pub unsafe extern "C" fn RequestGetContext(Request: WDFREQUEST) -> *mut CDROM_REQUEST_CONTEXT {
    WdfObjectGetTypedContext(Request as WDFOBJECT, &CDROM_REQUEST_CONTEXT_TYPE_INFO.0)
        as *mut CDROM_REQUEST_CONTEXT
}
#[no_mangle]
pub unsafe extern "C" fn FileObjectGetContext(
    FileObject: WDFFILEOBJECT,
) -> *mut FILE_OBJECT_CONTEXT {
    WdfObjectGetTypedContext(FileObject as WDFOBJECT, &FILE_OBJECT_CONTEXT_TYPE_INFO.0)
        as *mut FILE_OBJECT_CONTEXT
}
#[no_mangle]
pub unsafe extern "C" fn DriverGetExtension(Driver: WDFDRIVER) -> *mut CDROM_DRIVER_EXTENSION {
    WdfObjectGetTypedContext(Driver as WDFOBJECT, &_WDF_CDROM_DRIVER_EXTENSION_TYPE_INFO)
        as *mut CDROM_DRIVER_EXTENSION
}

// ============================ more WDF *_INIT macros ============================
#[no_mangle]
pub unsafe extern "C" fn WDF_OBJECT_ATTRIBUTES_INIT(Attributes: *mut WDF_OBJECT_ATTRIBUTES) {
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
pub unsafe extern "C" fn WDF_IO_QUEUE_CONFIG_INIT(Config: *mut WDF_IO_QUEUE_CONFIG, DispatchType: i32) {
    io_queue_config_init(Config, DispatchType, 0);
}
#[no_mangle]
pub unsafe extern "C" fn WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE(Config: *mut WDF_IO_QUEUE_CONFIG, DispatchType: i32) {
    io_queue_config_init(Config, DispatchType, 1);
}
#[no_mangle]
pub unsafe extern "C" fn WDF_FILEOBJECT_CONFIG_INIT(
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
pub unsafe extern "C" fn WDF_WORKITEM_CONFIG_INIT(Config: *mut WDF_WORKITEM_CONFIG, EvtWorkItemFunc: PFN_WDF_WORKITEM) {
    core::ptr::write_bytes(Config as *mut u8, 0, core::mem::size_of::<WDF_WORKITEM_CONFIG>());
    (*Config).Size = core::mem::size_of::<WDF_WORKITEM_CONFIG>() as u32;
    (*Config).EvtWorkItemFunc = EvtWorkItemFunc;
    (*Config).AutomaticSerialization = 1; // TRUE
}
#[no_mangle]
pub unsafe extern "C" fn WDF_REMOVE_LOCK_OPTIONS_INIT(Options: *mut WDF_REMOVE_LOCK_OPTIONS, Flags: u32) {
    core::ptr::write_bytes(Options as *mut u8, 0, core::mem::size_of::<WDF_REMOVE_LOCK_OPTIONS>());
    (*Options).Size = core::mem::size_of::<WDF_REMOVE_LOCK_OPTIONS>() as u32;
    (*Options).Flags = Flags;
}
#[no_mangle]
pub unsafe extern "C" fn WDF_IO_TARGET_OPEN_PARAMS_INIT_EXISTING_DEVICE(
    Params: *mut WDF_IO_TARGET_OPEN_PARAMS,
    DeviceObject: PDEVICE_OBJECT,
) {
    core::ptr::write_bytes(Params as *mut u8, 0, core::mem::size_of::<WDF_IO_TARGET_OPEN_PARAMS>());
    (*Params).Size = core::mem::size_of::<WDF_IO_TARGET_OPEN_PARAMS>() as u32;
    (*Params).Type = 2; // WdfIoTargetOpenUseExistingDevice
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
#[no_mangle]
pub unsafe extern "C" fn PORT_ALLOCATED_SENSE(_Srb: PVOID) -> u8 { 0 }
#[no_mangle]
pub unsafe extern "C" fn RequestGetCurrentStackLocationFlags(_Request: WDFREQUEST) -> u8 { 0 }
// RtlStringCchPrintfW: ntstrsafe inline (no export). Stub returns STATUS_SUCCESS; building the
// formatted device-interface ref-string is non-fatal to boot (the GUID path still resolves).
// TODO: real swprintf impl for correct symbolic-link suffixes.
#[no_mangle]
pub unsafe extern "C" fn RtlStringCchPrintfW(_Dst: *mut u16, _CchDst: usize, _Format: *const u16) -> NTSTATUS { 0 }
// ScratchBuffer_EndUse: pairs with ScratchBuffer_BeginUse (releases the scratch in-use flag).
// TODO: confirm the field; no-op stub unblocks the link — flag for the smoke pass.
#[no_mangle]
pub unsafe extern "C" fn ScratchBuffer_EndUse(_DeviceExtension: PCDROM_DEVICE_EXTENSION) {}
