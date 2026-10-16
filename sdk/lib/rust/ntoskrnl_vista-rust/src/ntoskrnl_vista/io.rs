use crate::ntoskrnl_vista::fsrtl::PECP_LIST;
use crate::prelude::*;
use ntkrnl_sys::ddk::{DEVPROPKEY, DEVPROPTYPE, IoQueueWorkItem, LCID, MmGetSystemRoutineAddress, PDEVICE_OBJECT, PIO_WORKITEM, PIO_WORKITEM_ROUTINE_EX, PIRP, PUNICODE_STRING, RtlInitUnicodeString};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct EX_WORKITEM_CONTEXT {
    pub WorkItem: PIO_WORKITEM,
    pub WorkItemRoutineEx: PIO_WORKITEM_ROUTINE_EX,
    pub Context: PVOID,
}

pub type PEX_WORKITEM_CONTEXT = *mut EX_WORKITEM_CONTEXT;

pub const TAG_IOWI: u32 = 0x49574F49; // 'IWOI'
#[no_mangle]
pub unsafe extern "system" fn IoGetIrpExtraCreateParameter(Irp: PIRP, ExtraCreateParameter: *mut PECP_LIST) -> NTSTATUS {
    if BooleanFlagOn!((*Irp).Flags, IRP_CREATE_OPERATION) == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    *ExtraCreateParameter = (*Irp).UserBuffer as PECP_LIST;
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn IopWorkItemExCallback(DeviceObject: PDEVICE_OBJECT, Ctx: PVOID) {
    let mut context: PEX_WORKITEM_CONTEXT = Ctx as PEX_WORKITEM_CONTEXT;
    (*context).WorkItemRoutineEx.unwrap_unchecked()(
        DeviceObject as PVOID,
        (*context).Context,
        (*context).WorkItem,
    );
    ExFreePoolWithTag(context as PVOID, TAG_IOWI as ULONG);
}

#[no_mangle]
pub unsafe extern "system" fn IoQueueWorkItemEx(
    mut IoWorkItem: PIO_WORKITEM,
    mut WorkerRoutine: PIO_WORKITEM_ROUTINE_EX,
    mut QueueType: WORK_QUEUE_TYPE,
    mut Context: PVOID,
) {
    let mut newContext: PEX_WORKITEM_CONTEXT = ExAllocatePoolWithTag(
        NonPagedPoolMustSucceed,
        ::core::mem::size_of::<_EX_WORKITEM_CONTEXT>() as SIZE_T,
        TAG_IOWI as ULONG,
    ) as PEX_WORKITEM_CONTEXT;
    (*newContext).WorkItem = IoWorkItem;
    (*newContext).WorkItemRoutineEx = WorkerRoutine;
    (*newContext).Context = Context;
    IoQueueWorkItem(
        IoWorkItem,
        Some(IopWorkItemExCallback),
        QueueType,
        newContext as PVOID,
    );
}

// IoSet/GetDevicePropertyData are ntoskrnl exports now (pnpmgr-rust).

type PSET_INTERFACE_PROPERTY_ROUTINE = unsafe extern "system" fn(
    PUNICODE_STRING,
    *const DEVPROPKEY,
    LCID,
    ULONG,
    DEVPROPTYPE,
    ULONG,
    PVOID,
) -> NTSTATUS;
type PGET_INTERFACE_PROPERTY_ROUTINE = unsafe extern "system" fn(
    PUNICODE_STRING,
    *const DEVPROPKEY,
    LCID,
    ULONG,
    ULONG,
    PVOID,
    PULONG,
    *mut DEVPROPTYPE,
) -> NTSTATUS;

/// The kernel export `name`, looked up once (upstream 3cc9aa99292: "it only exists on
/// NT 6.2+", so a driver linking this library still loads on a kernel without it).
unsafe fn system_routine(cache: *mut PVOID, name: &[u16]) -> PVOID {
    if (*cache).is_null() {
        let mut RoutineName: UNICODE_STRING = core::mem::zeroed();
        RtlInitUnicodeString(&mut RoutineName, name.as_ptr());
        *cache = MmGetSystemRoutineAddress(&mut RoutineName);
    }
    *cache
}

static mut SetRoutine: PVOID = core::ptr::null_mut();
static mut GetRoutine: PVOID = core::ptr::null_mut();

/// Forward to the kernel export when present, it only exists on NT 6.2+.
#[no_mangle]
pub unsafe extern "system" fn IoSetDeviceInterfacePropertyData(
    SymbolicLinkName: PUNICODE_STRING,
    PropertyKey: *const DEVPROPKEY,
    Lcid: LCID,
    Flags: ULONG,
    Type: DEVPROPTYPE,
    Size: ULONG,
    Data: PVOID,
) -> NTSTATUS {
    let routine = system_routine(&raw mut SetRoutine, ntkrnl_sys::w!("IoSetDeviceInterfacePropertyData"));
    if routine.is_null() {
        return STATUS_NOT_IMPLEMENTED;
    }
    let routine: PSET_INTERFACE_PROPERTY_ROUTINE = core::mem::transmute(routine);
    routine(SymbolicLinkName, PropertyKey, Lcid, Flags, Type, Size, Data)
}

#[no_mangle]
pub unsafe extern "system" fn IoGetDeviceInterfacePropertyData(
    SymbolicLinkName: PUNICODE_STRING,
    PropertyKey: *const DEVPROPKEY,
    Lcid: LCID,
    Flags: ULONG,
    Size: ULONG,
    Data: PVOID,
    RequiredSize: PULONG,
    Type: *mut DEVPROPTYPE,
) -> NTSTATUS {
    let routine = system_routine(&raw mut GetRoutine, ntkrnl_sys::w!("IoGetDeviceInterfacePropertyData"));
    if routine.is_null() {
        return STATUS_NOT_IMPLEMENTED;
    }
    let routine: PGET_INTERFACE_PROPERTY_ROUTINE = core::mem::transmute(routine);
    routine(SymbolicLinkName, PropertyKey, Lcid, Flags, Size, Data, RequiredSize, Type)
}
#[no_mangle]
pub unsafe extern "system" fn IoGetIoPriorityHint(Irp: PIRP) -> IO_PRIORITY_HINT {
    let _ = Irp;
    return IoPriorityNormal;
}
#[no_mangle]
pub unsafe extern "C" fn IoSetMasterIrpStatus(MasterIrp: PIRP, Status: NTSTATUS) {
    let mut MasterStatus: NTSTATUS = (*MasterIrp).IoStatus.__bindgen_anon_1.Status;

    if Status == STATUS_FT_READ_FROM_COPY
    {
        return;
    }

    if (Status == STATUS_VERIFY_REQUIRED) ||
        (MasterStatus == STATUS_SUCCESS && !(Status >= 0 as NTSTATUS)) ||
        (!(MasterStatus >= 0 as NTSTATUS) && !(Status >= 0 as NTSTATUS) && Status > MasterStatus)
    {
        (*MasterIrp).IoStatus.__bindgen_anon_1.Status = Status;
    }
}
