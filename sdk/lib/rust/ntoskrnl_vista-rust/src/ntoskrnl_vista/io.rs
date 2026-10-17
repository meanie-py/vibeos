use crate::ntoskrnl_vista::fsrtl::PECP_LIST;
use crate::prelude::*;
use ntkrnl_sys::ddk::{DEVPROPKEY, DEVPROPTYPE, IoQueueWorkItem, LCID, PDEVICE_OBJECT, PIO_WORKITEM, PIO_WORKITEM_ROUTINE_EX, PIRP, PUNICODE_STRING};
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

// IoSet/GetDeviceInterfacePropertyData are ntoskrnl exports too (pnpprop.rs). Upstream's
// io.c forwards to the kernel export through MmGetSystemRoutineAddress because it may be
// absent (it is NT 6.2+); this kernel always has it, so a copy here would only shadow the
// import (scan_export_shadows), as the IoSet/GetDevicePropertyData ones did.
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
