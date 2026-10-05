use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::PsInitialSystemProcess;
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    HANDLE, ACCESS_MASK, PKTHREAD,
    _KPRCB,
    _EPROCESS,
    _ETHREAD,
    _DEVICETREE_TRAVERSE_CONTEXT,
    _DEVICE_NODE,
    _OBJECT_ATTRIBUTES,
    _CALLBACK_OBJECT,
    _PROCESSOR_POWER_STATE,
    PPROCESSOR_POWER_STATE,
    PCALLBACK_OBJECT,
    PEPROCESS,
    PDEVICE_NODE,
    PDEVICE_OBJECT,
    PIRP,
    PIO_STACK_LOCATION,
    PKDPC,
    PWORK_QUEUE_ITEM,
    WORK_QUEUE_ITEM,
    PREQUEST_POWER_COMPLETE,
    POWER_STATE,
    SYSTEM_POWER_STATE,
    POWER_ACTION,
    DEVICE_POWER_STATE,
    POWER_STATE_TYPE,
    EXECUTION_STATE,
    LATENCY_TIME,
    POWER_INFORMATION_LEVEL,
    _IO_STATUS_BLOCK,
    IO_STATUS_BLOCK,
    KEVENT,
    KPROCESSOR_MODE,
    SYSTEM_POWER_CAPABILITIES,
    SYSTEM_BATTERY_STATE,
    PROCESSOR_POWER_INFORMATION,
    BATTERY_QUERY_INFORMATION,
    BATTERY_INFORMATION,
    BATTERY_STATUS,
    BATTERY_WAIT_STATUS,
    WORK_QUEUE_TYPE,
    BATTERY_QUERY_INFORMATION_LEVEL,
    GUID,
    _EVENT_TYPE_NotificationEvent,
    _KWAIT_REASON_Executive,
    _WORK_QUEUE_TYPE_DelayedWorkQueue,
    _WORK_QUEUE_TYPE_CriticalWorkQueue,
    IoAllocateIrp,
    IoBuildSynchronousFsdRequest,
    IofCallDriver,
    IoFreeIrp,
    IoGetAttachedDeviceReference,
    IoGetCurrentIrpStackLocation,
    IoGetNextIrpStackLocation,
    IoRegisterPlugPlayNotification,
    IoSetCompletionRoutine,
    IoSetNextIrpStackLocation,
    IoSkipCurrentIrpStackLocation,
    IopGetSystemPowerDeviceObject,
    IopTraverseDeviceTree,
    KeInitializeDpc,
    KeInitializeSpinLock,
    KeInitializeEvent,
    KeInitializeTimerEx,
    KeSetTargetProcessorDpc,
    KeSuspendThread,
    KeWaitForSingleObject,
    ExQueueWorkItem,
    ExNotifyCallback,
    ExSwapinWorkerThreads,
    HalProcessorIdle,
    CcRosFlushDirtyPages,
    CmSetLazyFlushState,
    _LOADER_PARAMETER_BLOCK,
    _LOADER_PARAMETER_EXTENSION,
};

// KeLoaderBlock: ke global read across the crate boundary as a link symbol
// (#[no_mangle] in ntoskrnl-rust ke/krnlinit.rs). crate-split Phase 1b.
extern "C" {
    static mut KeLoaderBlock: ntkrnl_sys::ddk::PLOADER_PARAMETER_BLOCK;
}
extern "system" {
    static mut IopRootDeviceNode: PDEVICE_NODE;
    static mut PowerStateCallback: PCALLBACK_OBJECT;
    static SeShutdownPrivilege: LUID;

    fn KeGetCurrentThread() -> PKTHREAD;
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
    fn ProbeForRead(address: PVOID, length: SIZE_T, alignment: ULONG);
    fn ProbeForWrite(address: PVOID, length: SIZE_T, alignment: ULONG);
    fn SeSinglePrivilegeCheck(privilege: LUID, previous_mode: KPROCESSOR_MODE) -> BOOLEAN;
    fn ZwOpenFile(
        file_handle: *mut HANDLE,
        desired_access: ACCESS_MASK,
        object_attributes: *mut _OBJECT_ATTRIBUTES,
        io_status_block: *mut _IO_STATUS_BLOCK,
        share_access: ULONG,
        open_options: ULONG,
    ) -> NTSTATUS;
    fn ZwClose(handle: HANDLE) -> NTSTATUS;
    fn ZwDeviceIoControlFile(
        file_handle: HANDLE,
        event: HANDLE,
        apc_routine: PVOID,
        apc_context: PVOID,
        io_status_block: *mut _IO_STATUS_BLOCK,
        io_control_code: ULONG,
        input_buffer: PVOID,
        input_buffer_length: ULONG,
        output_buffer: PVOID,
        output_buffer_length: ULONG,
    ) -> NTSTATUS;
    fn ZwSetSystemPowerState(
        system_action: POWER_ACTION,
        min_system_state: SYSTEM_POWER_STATE,
        flags: ULONG,
    ) -> NTSTATUS;
    fn PopInitShutdownList();
    fn PopReadShutdownPolicy();
    fn PopFlushVolumes(shutdown: BOOLEAN);
    fn PopGracefulShutdown(parameter: PVOID);
    static mut PopVolumeDevices: LIST_ENTRY;
    static mut PopVolumeLock: ntkrnl_sys::ke::KGUARDED_MUTEX;
    static mut PopDopeGlobalLock: KSPIN_LOCK;

    static GUID_DEVICE_SYS_BUTTON: GUID;
    static GUID_DEVICE_LID: GUID;
    static GUID_DEVICE_BATTERY: GUID;
}
extern "C" {
}
// NTAPI (stdcall on i386) — defs in shim_exports/common_inlines are extern "system"
extern "system" {
}

extern "system" {
    fn PopAddRemoveSysCapsCallback(
        notification_structure: PVOID,
        context: PVOID,
    ) -> NTSTATUS;
}

/// ExInitializeWorkItem -- C macro, inline implementation.
#[no_mangle]
#[inline]
pub unsafe extern "system" fn ExInitializeWorkItem_inline(
    item: PWORK_QUEUE_ITEM,
    routine: ntkrnl_sys::ddk::PWORKER_THREAD_ROUTINE,
    context: PVOID,
) {
    let context = opaque_ptr(context);
    (*item).WorkerRoutine = routine;
    (*item).Parameter = context;
    (*item).List.Flink = core::ptr::null_mut();
}

/// IopInitDeviceTreeTraverseContext -- C macro, inline implementation.
#[no_mangle]
#[inline]
pub unsafe extern "system" fn IopInitDeviceTreeTraverseContext(
    ctx: *mut _DEVICETREE_TRAVERSE_CONTEXT,
    device_node: PDEVICE_NODE,
    action: ntkrnl_sys::ddk::DEVICETREE_TRAVERSE_ROUTINE,
    context: PVOID,
) {
    let ctx = opaque_ptr(ctx);
    let context = opaque_ptr(context);
    (*ctx).FirstDeviceNode = device_node;
    (*ctx).Action = action;
    (*ctx).Context = context;
}

extern "system" {
    fn IoCallDriver(device: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS;
}


#[repr(C)]
#[derive(Copy, Clone)]
struct LUID {
    LowPart: ULONG,
    HighPart: LONG,
}

#[repr(C)]
struct POWER_STATE_TRAVERSE_CONTEXT {
    SystemPowerState: SYSTEM_POWER_STATE,
    PowerAction: POWER_ACTION,
    PowerDevice: PDEVICE_OBJECT,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct POP_POWER_ACTION {
    pub Action: POWER_ACTION,
    pub Flags: ULONG,
    pub IrpMinor: UCHAR,
    pub Shutdown: BOOLEAN,
}

#[no_mangle]
pub static mut PopSystemPowerDeviceNode: PDEVICE_NODE = core::ptr::null_mut();
#[no_mangle]
pub static mut PopAcpiPresent: BOOLEAN = 0;
#[no_mangle]
pub static mut PopAction: POP_POWER_ACTION = POP_POWER_ACTION {
    Action: 0,
    Flags: 0,
    IrpMinor: 0,
    Shutdown: 0,
};
#[no_mangle]
pub static mut PopShutdownWorkItem: ntkrnl_sys::ddk::WORK_QUEUE_ITEM = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut PopCapabilities: SYSTEM_POWER_CAPABILITIES = unsafe { core::mem::zeroed() };

#[no_mangle]
pub unsafe extern "system" fn PopPassivePowerCall(Parameter: PVOID) {
    let Parameter = opaque_ptr(Parameter);
    PopPassivePowerCallInner(Parameter)
}
#[no_mangle]
#[inline(always)]
pub unsafe extern "system" fn PopPassivePowerCallInner(Parameter: PVOID) {
    let Parameter = opaque_ptr(Parameter);
    let Irp = Parameter as PIRP;
    let IoStack: PIO_STACK_LOCATION;

    IoStack = IoGetNextIrpStackLocation(Irp);

    IofCallDriver((*IoStack).DeviceObject, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn PopPresentIrp(NextStack: PIO_STACK_LOCATION, Irp: PIRP) -> NTSTATUS {
    let NextStack = opaque_ptr(NextStack);
    let Irp = opaque_ptr(Irp);
    let Status: NTSTATUS;
    let mut CallAtPassiveLevel: BOOLEAN;
    let DeviceObject: PDEVICE_OBJECT;
    let WorkQueueItem: PWORK_QUEUE_ITEM;

    DeviceObject = (*NextStack).DeviceObject;

    CallAtPassiveLevel = 1;
    if ((*NextStack).MinorFunction == IRP_MN_SET_POWER) &&
        ((*DeviceObject).Flags & DO_POWER_PAGABLE) == 0
    {
        if (*NextStack).Parameters.Power.Type == DevicePowerState &&
            (*NextStack).Parameters.Power.State.DeviceState == PowerDeviceD0
        {
            CallAtPassiveLevel = 0;
        }
        if (*NextStack).Parameters.Power.Type == SystemPowerState &&
            (*NextStack).Parameters.Power.State.SystemState == PowerSystemWorking
        {
            CallAtPassiveLevel = 0;
        }
    }

    if CallAtPassiveLevel != 0 {
        if KeGetCurrentIrql() == PASSIVE_LEVEL {
            return IofCallDriver(DeviceObject, Irp);
        }

        (*NextStack).Control |= SL_PENDING_RETURNED;

        WorkQueueItem = core::ptr::addr_of_mut!((*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext) as PWORK_QUEUE_ITEM;
        ExInitializeWorkItem_inline(WorkQueueItem, Some(PopPassivePowerCall), Irp as PVOID);
        ExQueueWorkItem(WorkQueueItem, _WORK_QUEUE_TYPE_DelayedWorkQueue);

        return STATUS_PENDING;
    }

    Status = IofCallDriver(DeviceObject, Irp);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PopRequestPowerIrpCompletion(
    DeviceObject: PDEVICE_OBJECT, Irp: PIRP, Context: PVOID,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    let Context = opaque_ptr(Context);
    PopRequestPowerIrpCompletionInner(DeviceObject, Irp, Context)
}
#[no_mangle]
#[inline(always)]
pub unsafe extern "system" fn PopRequestPowerIrpCompletionInner(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    let Context = opaque_ptr(Context);
    let Stack: PIO_STACK_LOCATION;
    let CompletionRoutine: PREQUEST_POWER_COMPLETE;
    let mut PowerState: POWER_STATE = core::mem::zeroed();

    Stack = IoGetCurrentIrpStackLocation(Irp);
    CompletionRoutine = core::mem::transmute::<PVOID, PREQUEST_POWER_COMPLETE>(Context);

    PowerState.DeviceState = (*Stack).Parameters.Others.Argument3 as usize as u32;

    if CompletionRoutine.is_some() {
        (CompletionRoutine.unwrap())(
            (*Stack).Parameters.Others.Argument1 as PDEVICE_OBJECT,
            (*Stack).Parameters.Others.Argument2 as u8,
            PowerState,
            (*Stack).Parameters.Others.Argument4,
            core::ptr::addr_of_mut!((*Irp).IoStatus),
        );
    }

    IoSkipCurrentIrpStackLocation(Irp);
    IoFreeIrp(Irp);
    ObDereferenceObject(DeviceObject as PVOID);

    STATUS_MORE_PROCESSING_REQUIRED
}

#[no_mangle]
pub unsafe extern "system" fn PopCleanupPowerState(PowerState: *mut POWER_STATE) {
    let PowerState = opaque_ptr(PowerState);
}

#[no_mangle]
pub unsafe extern "C" fn PopSendQuerySystemPowerState(
    DeviceObject: PDEVICE_OBJECT,
    SystemState: SYSTEM_POWER_STATE,
    PowerAction: POWER_ACTION,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let IrpSp: PIO_STACK_LOCATION;
    let Irp: PIRP;
    let Status: NTSTATUS;

    KeInitializeEvent(
        core::ptr::addr_of_mut!(Event),
        _EVENT_TYPE_NotificationEvent,
        0,
    );

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_POWER as ULONG,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    IrpSp = IoGetNextIrpStackLocation(Irp);
    (*IrpSp).MinorFunction = IRP_MN_QUERY_POWER;
    (*IrpSp).Parameters.Power.Type = SystemPowerState;
    (*IrpSp).Parameters.Power.State.SystemState = SystemState;
    (*IrpSp).Parameters.Power.ShutdownType = PowerAction;

    let status = PoCallDriver(DeviceObject, Irp);
    if status == STATUS_PENDING {
        // RunToken pilot: synchronous-IRP wait via the CPS token API; the
        // IOSB read runs in the continuation, after the re-brand.
        Status = ntkrnl_sys::token::with_token::<ntkrnl_sys::token::Passive, _>(|tok| {
            ntkrnl_sys::token::wait_for(
                tok,
                core::ptr::addr_of_mut!(Event) as PVOID,
                _KWAIT_REASON_Executive as u32,
                KernelMode as i8,
                0,
                core::ptr::null_mut(),
                |_tok, _wait_status| IoStatusBlock.__bindgen_anon_1.Status,
            )
        });
    } else {
        Status = status;
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn PopSendSetSystemPowerState(
    DeviceObject: PDEVICE_OBJECT,
    SystemState: SYSTEM_POWER_STATE,
    PowerAction: POWER_ACTION,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let IrpSp: PIO_STACK_LOCATION;
    let Irp: PIRP;
    let Status: NTSTATUS;

    KeInitializeEvent(
        core::ptr::addr_of_mut!(Event),
        _EVENT_TYPE_NotificationEvent,
        0,
    );

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_POWER as ULONG,
        DeviceObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatusBlock),
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    IrpSp = IoGetNextIrpStackLocation(Irp);
    (*IrpSp).MinorFunction = IRP_MN_SET_POWER;
    (*IrpSp).Parameters.Power.Type = SystemPowerState;
    (*IrpSp).Parameters.Power.State.SystemState = SystemState;
    (*IrpSp).Parameters.Power.ShutdownType = PowerAction;

    let status = PoCallDriver(DeviceObject, Irp);
    if status == STATUS_PENDING {
        // RunToken pilot: synchronous-IRP wait via the CPS token API; the
        // IOSB read runs in the continuation, after the re-brand.
        Status = ntkrnl_sys::token::with_token::<ntkrnl_sys::token::Passive, _>(|tok| {
            ntkrnl_sys::token::wait_for(
                tok,
                core::ptr::addr_of_mut!(Event) as PVOID,
                _KWAIT_REASON_Executive as u32,
                KernelMode as i8,
                0,
                core::ptr::null_mut(),
                |_tok, _wait_status| IoStatusBlock.__bindgen_anon_1.Status,
            )
        });
    } else {
        Status = status;
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn PopQuerySystemPowerStateTraverse(
    DeviceNode: PDEVICE_NODE,
    Context: PVOID,
) -> NTSTATUS {
    let Context = opaque_ptr(Context);
    let PowerStateContext = Context as *mut POWER_STATE_TRAVERSE_CONTEXT;
    let TopDeviceObject: PDEVICE_OBJECT;
    let Status: NTSTATUS;

    if DeviceNode == IopRootDeviceNode {
        return STATUS_SUCCESS;
    }

    if ((*DeviceNode).Flags & DNF_LEGACY_DRIVER) != 0 {
        return STATUS_SUCCESS;
    }

    TopDeviceObject = IoGetAttachedDeviceReference((*DeviceNode).PhysicalDeviceObject);

    Status = PopSendQuerySystemPowerState(
        TopDeviceObject,
        (*PowerStateContext).SystemPowerState,
        (*PowerStateContext).PowerAction,
    );

    ObDereferenceObject(TopDeviceObject as PVOID);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn PopSetSystemPowerStateTraverse(
    DeviceNode: PDEVICE_NODE,
    Context: PVOID,
) -> NTSTATUS {
    let Context = opaque_ptr(Context);
    let PowerStateContext = Context as *mut POWER_STATE_TRAVERSE_CONTEXT;
    let TopDeviceObject: PDEVICE_OBJECT;
    let Status: NTSTATUS;

    if DeviceNode == IopRootDeviceNode {
        return STATUS_SUCCESS;
    }

    if (*DeviceNode).PhysicalDeviceObject == (*PowerStateContext).PowerDevice {
        return STATUS_SUCCESS;
    }

    if ((*DeviceNode).Flags & DNF_LEGACY_DRIVER) != 0 {
        return STATUS_SUCCESS;
    }

    TopDeviceObject = IoGetAttachedDeviceReference((*DeviceNode).PhysicalDeviceObject);
    if TopDeviceObject == (*PowerStateContext).PowerDevice {
        ObDereferenceObject(TopDeviceObject as PVOID);
        return STATUS_SUCCESS;
    }

    Status = PopSendSetSystemPowerState(
        TopDeviceObject,
        (*PowerStateContext).SystemPowerState,
        (*PowerStateContext).PowerAction,
    );

    ObDereferenceObject(TopDeviceObject as PVOID);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PopSetSystemPowerState(
    PowerState: SYSTEM_POWER_STATE,
    PowerAction: POWER_ACTION,
) -> NTSTATUS {
    let DeviceObject: PDEVICE_OBJECT;
    let Fdo: PDEVICE_OBJECT;
    let Status: NTSTATUS;
    let mut Context: _DEVICETREE_TRAVERSE_CONTEXT = core::mem::zeroed();
    let mut PowerContext: POWER_STATE_TRAVERSE_CONTEXT = core::mem::zeroed();

    let mut dev_obj = core::mem::MaybeUninit::uninit();
    Status = IopGetSystemPowerDeviceObject(dev_obj.as_mut_ptr());
    if Status < 0 {
        Fdo = core::ptr::null_mut();
    } else {
        DeviceObject = dev_obj.assume_init();
        Fdo = IoGetAttachedDeviceReference(DeviceObject);
        if Fdo == DeviceObject {
            return STATUS_UNSUCCESSFUL;
        }
    }

    PowerContext.PowerAction = PowerAction;
    PowerContext.SystemPowerState = PowerState;
    PowerContext.PowerDevice = Fdo;

    IopInitDeviceTreeTraverseContext(
        core::ptr::addr_of_mut!(Context),
        IopRootDeviceNode,
        Some(PopQuerySystemPowerStateTraverse),
        core::ptr::addr_of_mut!(PowerContext) as PVOID,
    );

    let status = IopTraverseDeviceTree(core::ptr::addr_of_mut!(Context));

    IopInitDeviceTreeTraverseContext(
        core::ptr::addr_of_mut!(Context),
        IopRootDeviceNode,
        Some(PopSetSystemPowerStateTraverse),
        core::ptr::addr_of_mut!(PowerContext) as PVOID,
    );

    IopTraverseDeviceTree(core::ptr::addr_of_mut!(Context));

    if PopAcpiPresent == 0 {
        return STATUS_NOT_IMPLEMENTED;
    }

    if !Fdo.is_null() {
        if PowerAction != PowerActionShutdownReset {
            PopSendSetSystemPowerState(Fdo, PowerState, PowerAction);
        }

        ObDereferenceObject(Fdo as PVOID);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn PoInitSystem(BootPhase: ULONG) -> BOOLEAN {
    let mut NotificationEntry = core::mem::MaybeUninit::uninit();
    let CommandLine: PCHAR;
    let mut ForceAcpiDisable: BOOLEAN = 0;

    if BootPhase == 1 {
        let mut Status: NTSTATUS;

        Status = IoRegisterPlugPlayNotification(
            EventCategoryDeviceInterfaceChange,
            PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES,
            core::ptr::addr_of!(GUID_DEVICE_SYS_BUTTON) as PVOID,
            (*(*IopRootDeviceNode).PhysicalDeviceObject).DriverObject,
            Some(PopAddRemoveSysCapsCallback),
            PolicyDeviceSystemButton as usize as PVOID,
            NotificationEntry.as_mut_ptr(),
        );
        if Status < 0 {
            return 0;
        }

        Status = IoRegisterPlugPlayNotification(
            EventCategoryDeviceInterfaceChange,
            PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES,
            core::ptr::addr_of!(GUID_DEVICE_LID) as PVOID,
            (*(*IopRootDeviceNode).PhysicalDeviceObject).DriverObject,
            Some(PopAddRemoveSysCapsCallback),
            PolicyDeviceSystemButton as usize as PVOID,
            NotificationEntry.as_mut_ptr(),
        );
        if Status < 0 {
            return 0;
        }

        Status = IoRegisterPlugPlayNotification(
            EventCategoryDeviceInterfaceChange,
            PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES,
            core::ptr::addr_of!(GUID_DEVICE_BATTERY) as PVOID,
            (*(*IopRootDeviceNode).PhysicalDeviceObject).DriverObject,
            Some(PopAddRemoveSysCapsCallback),
            PolicyDeviceBattery as usize as PVOID,
            NotificationEntry.as_mut_ptr(),
        );

        return if Status >= 0 { 1 } else { 0 };
    }

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(PopCapabilities) as *mut u8,
        0,
        core::mem::size_of::<SYSTEM_POWER_CAPABILITIES>(),
    );

    CommandLine = (*KeLoaderBlock).LoadOptions;

    _strupr(CommandLine);

    if !strstr(CommandLine, b"NOACPI\0".as_ptr() as PCHAR).is_null() {
        ForceAcpiDisable = 1;
    }

    if ForceAcpiDisable != 0 {
        PopAcpiPresent = 0;
    } else {
        PopAcpiPresent = if !(*(*KeLoaderBlock).Extension).AcpiTable.is_null() { 1 } else { 0 };
    }

    if PopAcpiPresent != 0 {
        PopCapabilities.SystemS5 = 1;
    }

    InitializeListHead(core::ptr::addr_of_mut!(PopVolumeDevices));
    KeInitializeGuardedMutex(core::ptr::addr_of_mut!(PopVolumeLock) as *mut ntkrnl_sys::ke::KGUARDED_MUTEX);

    KeInitializeSpinLock(core::ptr::addr_of_mut!(PopDopeGlobalLock) as *mut _);

    PopInitShutdownList();

    1
}

#[no_mangle]
pub unsafe extern "system" fn PopPerfIdle(PowerState: PPROCESSOR_POWER_STATE) {
}

#[no_mangle]
pub unsafe extern "system" fn PopPerfIdleDpc(
    Dpc: PKDPC, DeferredContext: PVOID, SystemArgument1: PVOID, SystemArgument2: PVOID,
) {
    let Dpc = opaque_ptr(Dpc);
    let DeferredContext = opaque_ptr(DeferredContext);
    let SystemArgument1 = opaque_ptr(SystemArgument1);
    let SystemArgument2 = opaque_ptr(SystemArgument2);
    PopPerfIdle(core::ptr::addr_of_mut!((*(DeferredContext as *mut _KPRCB)).PowerState));
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PopIdle0(PowerState: PPROCESSOR_POWER_STATE) {
    HalProcessorIdle();
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PopIdle0(PowerState: PPROCESSOR_POWER_STATE) {
    HalProcessorIdle();
}

#[no_mangle]
pub unsafe extern "system" fn PoInitializePrcb(Prcb: *mut _KPRCB) {
    let Prcb = opaque_ptr(Prcb);
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*Prcb).PowerState) as *mut u8,
        0,
        core::mem::size_of::<_PROCESSOR_POWER_STATE>(),
    );
    (*Prcb).PowerState.Idle0KernelTimeLimit = 0xFFFFFFFF;
    (*Prcb).PowerState.CurrentThrottle = 100;
    (*Prcb).PowerState.CurrentThrottleIndex = 0;
    (*Prcb).PowerState.IdleFunction = Some(PopIdle0);

    KeInitializeDpc(
        core::ptr::addr_of_mut!((*Prcb).PowerState.PerfDpc),
        Some(PopPerfIdleDpc),
        Prcb as PVOID,
    );
    KeSetTargetProcessorDpc(
        core::ptr::addr_of_mut!((*Prcb).PowerState.PerfDpc),
        (*Prcb).Number as CCHAR,
    );
    KeInitializeTimerEx(
        core::ptr::addr_of_mut!((*Prcb).PowerState.PerfTimer),
        SynchronizationTimer,
    );
}

#[no_mangle]
pub unsafe extern "system" fn PoCancelDeviceNotify(NotifyBlock: PVOID) -> NTSTATUS {
    let NotifyBlock = opaque_ptr(NotifyBlock);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn PoRegisterDeviceNotify(
    Unknown0: PVOID,
    Unknown1: ULONG,
    Unknown2: ULONG,
    Unknown3: ULONG,
    Unknown4: PVOID,
    Unknown5: PVOID,
) -> NTSTATUS {
    let Unknown0 = opaque_ptr(Unknown0);
    let Unknown4 = opaque_ptr(Unknown4);
    let Unknown5 = opaque_ptr(Unknown5);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn PoShutdownBugCheck(
    LogError: BOOLEAN,
    BugCheckCode: ULONG,
    BugCheckParameter1: ULONG_PTR,
    BugCheckParameter2: ULONG_PTR,
    BugCheckParameter3: ULONG_PTR,
    BugCheckParameter4: ULONG_PTR,
) {
    KeBugCheckEx(
        BugCheckCode,
        BugCheckParameter1,
        BugCheckParameter2,
        BugCheckParameter3,
        BugCheckParameter4,
    );
}

#[no_mangle]
pub unsafe extern "system" fn PoSetHiberRange(
    HiberContext: PVOID,
    Flags: ULONG,
    StartPage: PVOID,
    Length: ULONG,
    PageTag: ULONG,
) {
    let HiberContext = opaque_ptr(HiberContext);
    let StartPage = opaque_ptr(StartPage);
}

#[no_mangle]
pub unsafe extern "system" fn PoCallDriver(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    let NextStack: PIO_STACK_LOCATION;

    NextStack = IoGetNextIrpStackLocation(Irp);

    (*NextStack).DeviceObject = DeviceObject;

    if (*NextStack).MinorFunction != IRP_MN_SET_POWER &&
        (*NextStack).MinorFunction != IRP_MN_QUERY_POWER
    {
        return IoCallDriver(DeviceObject, Irp);
    }

    PopPresentIrp(NextStack, Irp)
}

#[no_mangle]
pub unsafe extern "system" fn PoRegisterDeviceForIdleDetection(
    DeviceObject: PDEVICE_OBJECT,
    ConservationIdleTime: ULONG,
    PerformanceIdleTime: ULONG,
    State: DEVICE_POWER_STATE,
) -> *mut ULONG {
    let DeviceObject = opaque_ptr(DeviceObject);
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn PoRegisterSystemState(
    StateHandle: PVOID,
    Flags: EXECUTION_STATE,
) -> PVOID {
    let StateHandle = opaque_ptr(StateHandle);
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn PoRequestPowerIrp(
    DeviceObject: PDEVICE_OBJECT,
    MinorFunction: UCHAR,
    PowerState: POWER_STATE,
    CompletionFunction: PREQUEST_POWER_COMPLETE,
    Context: PVOID,
    pIrp: *mut PIRP,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Context = opaque_ptr(Context);
    let pIrp = opaque_ptr(pIrp);
    let TopDeviceObject: PDEVICE_OBJECT;
    let mut Stack: PIO_STACK_LOCATION;
    let Irp: PIRP;

    if MinorFunction != IRP_MN_QUERY_POWER &&
        MinorFunction != IRP_MN_SET_POWER &&
        MinorFunction != IRP_MN_WAIT_WAKE
    {
        return STATUS_INVALID_PARAMETER_2;
    }

    TopDeviceObject = IoGetAttachedDeviceReference(DeviceObject);

    Irp = IoAllocateIrp(((*TopDeviceObject).StackSize as i32 + 2) as CCHAR, 0);
    if Irp.is_null() {
        ObDereferenceObject(TopDeviceObject as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    (*Irp).IoStatus.Information = 0;

    IoSetNextIrpStackLocation(Irp);

    Stack = IoGetNextIrpStackLocation(Irp);
    (*Stack).Parameters.Others.Argument1 = DeviceObject as PVOID;
    (*Stack).Parameters.Others.Argument2 = MinorFunction as usize as PVOID;
    (*Stack).Parameters.Others.Argument3 = PowerState.DeviceState as PVOID;
    (*Stack).Parameters.Others.Argument4 = Context;
    (*Stack).DeviceObject = TopDeviceObject;
    IoSetNextIrpStackLocation(Irp);

    Stack = IoGetNextIrpStackLocation(Irp);
    (*Stack).MajorFunction = IRP_MJ_POWER;
    (*Stack).MinorFunction = MinorFunction;
    if MinorFunction == IRP_MN_WAIT_WAKE {
        (*Stack).Parameters.WaitWake.PowerState = PowerState.SystemState;
    } else {
        (*Stack).Parameters.Power.Type = DevicePowerState;
        (*Stack).Parameters.Power.State = PowerState;
    }

    if !pIrp.is_null() {
        *pIrp = Irp;
    }

    IoSetCompletionRoutine(
        Irp,
        Some(PopRequestPowerIrpCompletion),
        core::mem::transmute::<PREQUEST_POWER_COMPLETE, PVOID>(CompletionFunction),
        1,
        1,
        1,
    );
    PoCallDriver(TopDeviceObject, Irp);

    STATUS_PENDING
}

#[no_mangle]
pub unsafe extern "system" fn PoSetPowerState(
    DeviceObject: PDEVICE_OBJECT,
    Type: POWER_STATE_TYPE,
    State: POWER_STATE,
) -> POWER_STATE {
    let DeviceObject = opaque_ptr(DeviceObject);
    let mut ps: POWER_STATE = core::mem::zeroed();

    ps.SystemState = PowerSystemWorking;
    ps.DeviceState = PowerDeviceD0;

    ps
}

#[no_mangle]
pub unsafe extern "system" fn PoSetSystemState(Flags: EXECUTION_STATE) {
}

#[no_mangle]
pub unsafe extern "system" fn PoStartNextPowerIrp(Irp: PIRP) {
    let Irp = opaque_ptr(Irp);
}

#[no_mangle]
pub unsafe extern "system" fn PoUnregisterSystemState(StateHandle: PVOID) {
    let StateHandle = opaque_ptr(StateHandle);
}

#[no_mangle]
pub unsafe extern "system" fn NtInitiatePowerAction(
    SystemAction: POWER_ACTION,
    MinSystemState: SYSTEM_POWER_STATE,
    Flags: ULONG,
    Asynchronous: BOOLEAN,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn PopQueryBatteryState(BatteryState: *mut SYSTEM_BATTERY_STATE) -> NTSTATUS {
    let BatteryState = opaque_ptr(BatteryState);
    static COMP_BATT_DEVICE_NAME: [u16; 26] = [
        92, 68, 101, 118, 105, 99, 101, 92, 67, 111, 109, 112, 111, 115, 105, 116, 101, 66, 97,
        116, 116, 101, 114, 121, 0, 0,
    ];

    let mut device_name = UNICODE_STRING {
        Length: 48,
        MaximumLength: 50,
        Buffer: COMP_BATT_DEVICE_NAME.as_ptr() as *mut u16,
    };

    let mut object_attributes = _OBJECT_ATTRIBUTES {
        Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::addr_of_mut!(device_name) as *mut ntkrnl_sys::ddk::_UNICODE_STRING,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };

    let mut CompBattHandle: HANDLE = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut BatteryQueryInfo: BATTERY_QUERY_INFORMATION = core::mem::zeroed();
    let mut BatteryInfo: BATTERY_INFORMATION = core::mem::zeroed();
    let mut BatteryStatus: BATTERY_STATUS = core::mem::zeroed();
    let Status: NTSTATUS;

    Status = ZwOpenFile(
        core::ptr::addr_of_mut!(CompBattHandle),
        GENERIC_READ | SYNCHRONIZE,
        core::ptr::addr_of_mut!(object_attributes),
        core::ptr::addr_of_mut!(IoStatusBlock),
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if Status < 0 {
        return Status;
    }

    let mut Wait: ULONG = 0;
    let status = ZwDeviceIoControlFile(
        CompBattHandle,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        IOCTL_BATTERY_QUERY_TAG,
        core::ptr::addr_of_mut!(Wait) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::addr_of_mut!(BatteryQueryInfo.BatteryTag) as PVOID,
        core::mem::size_of_val(&BatteryQueryInfo.BatteryTag) as ULONG,
    );
    if status < 0 {
        ZwClose(CompBattHandle);
        return status;
    }

    BatteryQueryInfo.InformationLevel = BatteryInformation as u32;
    let status = ZwDeviceIoControlFile(
        CompBattHandle,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        IOCTL_BATTERY_QUERY_INFORMATION,
        core::ptr::addr_of_mut!(BatteryQueryInfo) as PVOID,
        core::mem::size_of::<BATTERY_QUERY_INFORMATION>() as ULONG,
        core::ptr::addr_of_mut!(BatteryInfo) as PVOID,
        core::mem::size_of::<BATTERY_INFORMATION>() as ULONG,
    );

    let mut BatteryWait: BATTERY_WAIT_STATUS = core::mem::zeroed();
    BatteryWait.BatteryTag = BatteryQueryInfo.BatteryTag;
    BatteryWait.PowerState = 0xF;
    BatteryWait.HighCapacity = MAXULONG;
    let status = ZwDeviceIoControlFile(
        CompBattHandle,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(IoStatusBlock),
        IOCTL_BATTERY_QUERY_STATUS,
        core::ptr::addr_of_mut!(BatteryWait) as PVOID,
        core::mem::size_of::<BATTERY_WAIT_STATUS>() as ULONG,
        core::ptr::addr_of_mut!(BatteryStatus) as PVOID,
        core::mem::size_of::<BATTERY_STATUS>() as ULONG,
    );
    if status < 0 {
        ZwClose(CompBattHandle);
        return status;
    }

    core::ptr::write_bytes(BatteryState as *mut u8, 0, core::mem::size_of::<SYSTEM_BATTERY_STATE>());
    (*BatteryState).BatteryPresent = 1;
    (*BatteryState).AcOnLine = if (BatteryStatus.PowerState & BATTERY_POWER_ON_LINE) != 0 { 1 } else { 0 };
    (*BatteryState).Charging = if (BatteryStatus.PowerState & BATTERY_CHARGING) != 0 { 1 } else { 0 };
    (*BatteryState).Discharging = if (BatteryStatus.PowerState & BATTERY_DISCHARGING) != 0 { 1 } else { 0 };
    (*BatteryState).MaxCapacity = BatteryInfo.FullChargedCapacity;
    (*BatteryState).RemainingCapacity = BatteryStatus.Capacity;
    (*BatteryState).Rate = BatteryStatus.Rate as ULONG;
    if (*BatteryState).Discharging != 0 && BatteryStatus.Rate < 0 {
        (*BatteryState).EstimatedTime =
            3600 * BatteryInfo.FullChargedCapacity / (-(BatteryStatus.Rate) as ULONG);
    } else {
        (*BatteryState).EstimatedTime = 0;
    }

    ZwClose(CompBattHandle);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtPowerInformation(
    PowerInformationLevel: POWER_INFORMATION_LEVEL,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
) -> NTSTATUS {
    let InputBuffer = opaque_ptr(InputBuffer);
    let OutputBuffer = opaque_ptr(OutputBuffer);
    let mut Status: NTSTATUS;
    let PreviousMode: KPROCESSOR_MODE = KeGetPreviousMode();

    if PreviousMode != KernelMode {
        ProbeForRead(InputBuffer, InputBufferLength as SIZE_T, 1);
        ProbeForWrite(OutputBuffer, OutputBufferLength as SIZE_T, core::mem::size_of::<ULONG>() as ULONG);
    }

    match PowerInformationLevel {
        SystemBatteryState => {
            let mut BatteryState: SYSTEM_BATTERY_STATE = core::mem::zeroed();

            if !InputBuffer.is_null() {
                return STATUS_INVALID_PARAMETER;
            }
            if OutputBufferLength < core::mem::size_of::<SYSTEM_BATTERY_STATE>() as ULONG {
                return STATUS_BUFFER_TOO_SMALL;
            }

            if PopCapabilities.SystemBatteriesPresent != 0 {
                Status = PopQueryBatteryState(core::ptr::addr_of_mut!(BatteryState));
                if Status < 0 {
                    return Status;
                }
            } else {
                core::ptr::write_bytes(
                    core::ptr::addr_of_mut!(BatteryState) as *mut u8,
                    0,
                    core::mem::size_of::<SYSTEM_BATTERY_STATE>(),
                );
                BatteryState.AcOnLine = 1;
            }

            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!(BatteryState) as *const u8,
                OutputBuffer as *mut u8,
                core::mem::size_of::<SYSTEM_BATTERY_STATE>(),
            );
            Status = STATUS_SUCCESS;
        }

        SystemPowerCapabilities => {
            let PowerCapabilities = OutputBuffer as *mut SYSTEM_POWER_CAPABILITIES;

            if !InputBuffer.is_null() {
                return STATUS_INVALID_PARAMETER;
            }
            if OutputBufferLength < core::mem::size_of::<SYSTEM_POWER_CAPABILITIES>() as ULONG {
                return STATUS_BUFFER_TOO_SMALL;
            }

            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!(PopCapabilities) as *const u8,
                PowerCapabilities as *mut u8,
                core::mem::size_of::<SYSTEM_POWER_CAPABILITIES>(),
            );

            Status = STATUS_SUCCESS;
        }

        ProcessorInformation => {
            let PowerInformation = OutputBuffer as *mut PROCESSOR_POWER_INFORMATION;

            if !InputBuffer.is_null() {
                return STATUS_INVALID_PARAMETER;
            }
            if OutputBufferLength < core::mem::size_of::<PROCESSOR_POWER_INFORMATION>() as ULONG {
                return STATUS_BUFFER_TOO_SMALL;
            }

            (*PowerInformation).Number = 0;
            (*PowerInformation).MaxMhz = 1000;
            (*PowerInformation).CurrentMhz = (*(KeGetCurrentPrcb() as *mut _KPRCB)).MHz;
            (*PowerInformation).MhzLimit = 1000;
            (*PowerInformation).MaxIdleState = 0;
            (*PowerInformation).CurrentIdleState = 0;

            Status = STATUS_SUCCESS;
        }

        _ => {
            Status = STATUS_NOT_IMPLEMENTED;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtGetDevicePowerState(
    Device: ntkrnl_sys::ddk::HANDLE,
    PowerState: *mut DEVICE_POWER_STATE,
) -> NTSTATUS {
    let Device = opaque_ptr(Device);
    let PowerState = opaque_ptr(PowerState);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtIsSystemResumeAutomatic() -> BOOLEAN {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NtRequestWakeupLatency(Latency: LATENCY_TIME) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtSetThreadExecutionState(
    esFlags: EXECUTION_STATE,
    PreviousFlags: *mut EXECUTION_STATE,
) -> NTSTATUS {
    let PreviousFlags = opaque_ptr(PreviousFlags);
    let Thread = KeGetCurrentThread();
    let PreviousMode: KPROCESSOR_MODE = KeGetPreviousMode();
    let PreviousState: EXECUTION_STATE;

    if (esFlags & !(ES_CONTINUOUS | ES_USER_PRESENT)) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    // C: inside _SEH2_TRY, handler returns the exception code. No SEH here.
    if PreviousMode != KernelMode {
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(PreviousFlags as *mut ULONG, 1, PreviousMode) {
            return status;
        }
    }

    #[cfg(target_arch = "x86")]
    {
        PreviousState = (*(Thread as *mut _ETHREAD)).Tcb.__bindgen_anon_9.__bindgen_anon_6.PowerState as EXECUTION_STATE | ES_CONTINUOUS;
    }
    #[cfg(not(target_arch = "x86"))]
    {
        PreviousState = (*(Thread as *mut _ETHREAD)).Tcb.__bindgen_anon_7.__bindgen_anon_6.PowerState as EXECUTION_STATE | ES_CONTINUOUS;
    }

    if (esFlags & ES_CONTINUOUS) != 0 {
        #[cfg(target_arch = "x86")]
        {
            (*(Thread as *mut _ETHREAD)).Tcb.__bindgen_anon_9.__bindgen_anon_6.PowerState = esFlags as UCHAR;
        }
        #[cfg(not(target_arch = "x86"))]
        {
            (*(Thread as *mut _ETHREAD)).Tcb.__bindgen_anon_7.__bindgen_anon_6.PowerState = esFlags as UCHAR;
        }
    }

    *PreviousFlags = PreviousState;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtSetSystemPowerState(
    SystemAction: POWER_ACTION,
    MinSystemState: SYSTEM_POWER_STATE,
    Flags: ULONG,
) -> NTSTATUS {
    let PreviousMode: KPROCESSOR_MODE = KeGetPreviousMode();
    let mut Action: POP_POWER_ACTION = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Dummy: ULONG = 0;

    if (MinSystemState >= PowerSystemMaximum) ||
        (MinSystemState <= PowerSystemUnspecified) ||
        (SystemAction > PowerActionWarmEject) ||
        (SystemAction < PowerActionReserved) ||
        (Flags & !(POWER_ACTION_QUERY_ALLOWED |
                   POWER_ACTION_UI_ALLOWED |
                   POWER_ACTION_OVERRIDE_APPS |
                   POWER_ACTION_LIGHTEST_FIRST |
                   POWER_ACTION_LOCK_CONSOLE |
                   POWER_ACTION_DISABLE_WAKES |
                   POWER_ACTION_CRITICAL)) != 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    if PreviousMode != KernelMode {
        if SeSinglePrivilegeCheck(core::ptr::read(&SeShutdownPrivilege), PreviousMode) == 0 {
            return STATUS_PRIVILEGE_NOT_HELD;
        }

        return ZwSetSystemPowerState(SystemAction, MinSystemState, Flags);
    }

    if SystemAction == PowerActionShutdown {
        PopReadShutdownPolicy();
    }

    CmSetLazyFlushState(0);

    Action.Action = SystemAction;
    Action.Flags = Flags;

    ExNotifyCallback(PowerStateCallback as PCALLBACK_OBJECT, 3 as usize as PVOID, core::ptr::null_mut());

    ExSwapinWorkerThreads(0);

    PopAction = Action;

    Status = STATUS_CANCELLED;
    loop {
        if Action.Action == PowerActionNone {
            break;
        }

        if Status == STATUS_CANCELLED {
            if (PopAction.Action == PowerActionShutdown) ||
                (PopAction.Action == PowerActionShutdownReset) ||
                (PopAction.Action == PowerActionShutdownOff)
            {
                PopAction.Shutdown = 1;
            }

            Status = STATUS_SUCCESS;
        }

        if Status < 0 {
            break;
        }

        PopFlushVolumes(PopAction.Shutdown);

        CcRosFlushDirtyPages(MAXULONG, core::ptr::addr_of_mut!(Dummy), 1, 0);

        PopAction.IrpMinor = IRP_MN_SET_POWER;
        if PopAction.Shutdown != 0 {
            if PsGetCurrentProcess() as PVOID != PsInitialSystemProcess as PVOID {
                ExInitializeWorkItem_inline(
                    core::ptr::addr_of_mut!(PopShutdownWorkItem),
                    core::mem::transmute(PopGracefulShutdown as usize),
                    core::ptr::null_mut(),
                );

                ExQueueWorkItem(
                    core::ptr::addr_of_mut!(PopShutdownWorkItem),
                    _WORK_QUEUE_TYPE_CriticalWorkQueue,
                );

                KeSuspendThread(KeGetCurrentThread());
                Status = STATUS_SYSTEM_SHUTDOWN;
                break;
            } else {
                PopGracefulShutdown(core::ptr::null_mut());
            }
        }

        break;
    }

    Status
}

extern "C" {
    fn _strupr(s: PCHAR) -> PCHAR;
    fn strstr(haystack: PCHAR, needle: PCHAR) -> PCHAR;
}


const IRP_MJ_POWER: UCHAR = 0x16;
const IRP_MN_QUERY_POWER: UCHAR = 0x03;
const IRP_MN_SET_POWER: UCHAR = 0x02;
const IRP_MN_WAIT_WAKE: UCHAR = 0x00;
const DO_POWER_PAGABLE: ULONG = 0x2000;
const SL_PENDING_RETURNED: UCHAR = 0x01;
const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as i32;
const KernelMode: KPROCESSOR_MODE = 0;
const EventCategoryDeviceInterfaceChange: u32 = 2;
const PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES: ULONG = 0x00000001;
const PolicyDeviceSystemButton: ULONG = 0;
const PolicyDeviceBattery: ULONG = 1;
const PowerSystemWorking: SYSTEM_POWER_STATE = 0;
const PowerDeviceD0: DEVICE_POWER_STATE = 1;
const PowerActionNone: POWER_ACTION = 0;
const PowerActionShutdown: POWER_ACTION = 4;
const PowerActionShutdownReset: POWER_ACTION = 5;
const PowerActionShutdownOff: POWER_ACTION = 6;
const PowerActionWarmEject: POWER_ACTION = 7;
const PowerActionReserved: POWER_ACTION = 0;
const SynchronizationTimer: u32 = 0x01;
const SystemBatteryState: POWER_INFORMATION_LEVEL = 5;
const SystemPowerCapabilities: POWER_INFORMATION_LEVEL = 4;
const ProcessorInformation: POWER_INFORMATION_LEVEL = 11;
const BatteryInformation: u32 = 0;
const IOCTL_BATTERY_QUERY_TAG: ULONG = 0x00294040;
const IOCTL_BATTERY_QUERY_INFORMATION: ULONG = 0x00294044;
const IOCTL_BATTERY_QUERY_STATUS: ULONG = 0x0029404C;
const BATTERY_POWER_ON_LINE: ULONG = 0x00000001;
const BATTERY_CHARGING: ULONG = 0x00000004;
const BATTERY_DISCHARGING: ULONG = 0x00000002;
const GENERIC_READ: ULONG = 0x80000000;
const SYNCHRONIZE: ULONG = 0x00100000;
const FILE_SHARE_READ: ULONG = 0x00000001;
const FILE_SHARE_WRITE: ULONG = 0x00000002;
const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x00000020;
const MAXULONG: ULONG = 0xFFFFFFFF;
const ES_CONTINUOUS: EXECUTION_STATE = 0x80000000;
const ES_USER_PRESENT: EXECUTION_STATE = 0x00000004;
const POWER_ACTION_QUERY_ALLOWED: ULONG = 0x00000001;
const POWER_ACTION_UI_ALLOWED: ULONG = 0x00000002;
const POWER_ACTION_OVERRIDE_APPS: ULONG = 0x00000004;
const POWER_ACTION_LIGHTEST_FIRST: ULONG = 0x10000000;
const POWER_ACTION_LOCK_CONSOLE: ULONG = 0x20000000;
const POWER_ACTION_DISABLE_WAKES: ULONG = 0x40000000;
const POWER_ACTION_CRITICAL: ULONG = 0x80000000;
const STATUS_SYSTEM_SHUTDOWN: NTSTATUS = 0xC00002EB_u32 as i32;
const DNF_LEGACY_DRIVER: ULONG = 0x1000;
const PowerSystemUnspecified: SYSTEM_POWER_STATE = 0;
const PowerSystemMaximum: SYSTEM_POWER_STATE = 7;
const SystemPowerState: POWER_STATE_TYPE = 0;
const DevicePowerState: POWER_STATE_TYPE = 1;

type CCHAR = i8;


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/po/power.c
//  * PURPOSE:         Power Manager
//  * PROGRAMMERS:     Casper S. Hornstrup (chorns@users.sourceforge.net)
//  *                  Hervé Poussineau (hpoussin@reactos.com)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// typedef struct _POWER_STATE_TRAVERSE_CONTEXT
// {
//     SYSTEM_POWER_STATE SystemPowerState;
//     POWER_ACTION PowerAction;
//     PDEVICE_OBJECT PowerDevice;
// } POWER_STATE_TRAVERSE_CONTEXT, *PPOWER_STATE_TRAVERSE_CONTEXT;
//
// PDEVICE_NODE PopSystemPowerDeviceNode = NULL;
// BOOLEAN PopAcpiPresent = FALSE;
// POP_POWER_ACTION PopAction;
// WORK_QUEUE_ITEM PopShutdownWorkItem;
// SYSTEM_POWER_CAPABILITIES PopCapabilities;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// static WORKER_THREAD_ROUTINE PopPassivePowerCall;
// _Use_decl_annotations_
// static
// VOID
// NTAPI
// PopPassivePowerCall(
//     PVOID Parameter)
// {
//     PIRP Irp = Parameter;
//     PIO_STACK_LOCATION IoStack;
//
//     ASSERT(KeGetCurrentIrql() == PASSIVE_LEVEL);
//
//     _Analysis_assume_(Irp != NULL);
//     IoStack = IoGetNextIrpStackLocation(Irp);
//
//     (VOID)IoCallDriver(IoStack->DeviceObject, Irp);
// }
//
// _IRQL_requires_max_(DISPATCH_LEVEL)
// _IRQL_requires_same_
// static
// NTSTATUS
// PopPresentIrp(
//     _In_ PIO_STACK_LOCATION NextStack,
//     _In_ PIRP Irp)
// {
//     NTSTATUS Status;
//     BOOLEAN CallAtPassiveLevel;
//     PDEVICE_OBJECT DeviceObject;
//     PWORK_QUEUE_ITEM WorkQueueItem;
//
//     ASSERT(NextStack->MajorFunction == IRP_MJ_POWER);
//
//     DeviceObject = NextStack->DeviceObject;
//
//     /* Determine whether the IRP must be handled at PASSIVE_LEVEL.
//      * Only SET_POWER to working state can happen at raised IRQL. */
//     CallAtPassiveLevel = TRUE;
//     if ((NextStack->MinorFunction == IRP_MN_SET_POWER) &&
//         !(DeviceObject->Flags & DO_POWER_PAGABLE))
//     {
//         if (NextStack->Parameters.Power.Type == DevicePowerState &&
//             NextStack->Parameters.Power.State.DeviceState == PowerDeviceD0)
//         {
//             CallAtPassiveLevel = FALSE;
//         }
//         if (NextStack->Parameters.Power.Type == SystemPowerState &&
//             NextStack->Parameters.Power.State.SystemState == PowerSystemWorking)
//         {
//             CallAtPassiveLevel = FALSE;
//         }
//     }
//
//     if (CallAtPassiveLevel)
//     {
//         /* We need to fit a work item into the DriverContext below */
//         C_ASSERT(sizeof(Irp->Tail.Overlay.DriverContext) >= sizeof(WORK_QUEUE_ITEM));
//
//         if (KeGetCurrentIrql() == PASSIVE_LEVEL)
//         {
//             /* Already at passive, call next driver directly */
//             return IoCallDriver(DeviceObject, Irp);
//         }
//
//         /* Need to schedule a work item and return pending */
//         NextStack->Control |= SL_PENDING_RETURNED;
//
//         WorkQueueItem = (PWORK_QUEUE_ITEM)&Irp->Tail.Overlay.DriverContext;
//         ExInitializeWorkItem(WorkQueueItem,
//                              PopPassivePowerCall,
//                              Irp);
//         ExQueueWorkItem(WorkQueueItem, DelayedWorkQueue);
//
//         return STATUS_PENDING;
//     }
//
//     /* Direct call. Raise IRQL in debug to catch invalid paged memory access. */
// #if DBG
//     {
//     KIRQL OldIrql;
//     KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
// #endif
//
//     Status = IoCallDriver(DeviceObject, Irp);
//
// #if DBG
//     KeLowerIrql(OldIrql);
//     }
// #endif
//
//     return Status;
// }
//
// static IO_COMPLETION_ROUTINE PopRequestPowerIrpCompletion;
//
// static
// NTSTATUS
// NTAPI
// PopRequestPowerIrpCompletion(
//     _In_ PDEVICE_OBJECT DeviceObject,
//     _In_ PIRP Irp,
//     _In_reads_opt_(_Inexpressible_("varies")) PVOID Context)
// {
//     PIO_STACK_LOCATION Stack;
//     PREQUEST_POWER_COMPLETE CompletionRoutine;
//     POWER_STATE PowerState;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//     CompletionRoutine = Context;
//
//     PowerState.DeviceState = (ULONG_PTR)Stack->Parameters.Others.Argument3;
//
//     if (CompletionRoutine)
//     {
//         CompletionRoutine(Stack->Parameters.Others.Argument1,
//                           (UCHAR)(ULONG_PTR)Stack->Parameters.Others.Argument2,
//                           PowerState,
//                           Stack->Parameters.Others.Argument4,
//                           &Irp->IoStatus);
//     }
//
//     IoSkipCurrentIrpStackLocation(Irp);
//     IoFreeIrp(Irp);
//     ObDereferenceObject(DeviceObject);
//
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// VOID
// NTAPI
// PopCleanupPowerState(IN PPOWER_STATE PowerState)
// {
//     //UNIMPLEMENTED;
// }
//
// NTSTATUS
// PopSendQuerySystemPowerState(PDEVICE_OBJECT DeviceObject, SYSTEM_POWER_STATE SystemState, POWER_ACTION PowerAction)
// {
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIO_STACK_LOCATION IrpSp;
//     PIRP Irp;
//     NTSTATUS Status;
//
//     KeInitializeEvent(&Event,
//                       NotificationEvent,
//                       FALSE);
//
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_POWER,
//                                        DeviceObject,
//                                        NULL,
//                                        0,
//                                        NULL,
//                                        &Event,
//                                        &IoStatusBlock);
//     if (!Irp) return STATUS_INSUFFICIENT_RESOURCES;
//
//     IrpSp = IoGetNextIrpStackLocation(Irp);
//     IrpSp->MinorFunction = IRP_MN_QUERY_POWER;
//     IrpSp->Parameters.Power.Type = SystemPowerState;
//     IrpSp->Parameters.Power.State.SystemState = SystemState;
//     IrpSp->Parameters.Power.ShutdownType = PowerAction;
//
//     Status = PoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event,
//                               Executive,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// PopSendSetSystemPowerState(PDEVICE_OBJECT DeviceObject, SYSTEM_POWER_STATE SystemState, POWER_ACTION PowerAction)
// {
//     KEVENT Event;
//     IO_STATUS_BLOCK IoStatusBlock;
//     PIO_STACK_LOCATION IrpSp;
//     PIRP Irp;
//     NTSTATUS Status;
//
//     KeInitializeEvent(&Event,
//                       NotificationEvent,
//                       FALSE);
//
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_POWER,
//                                        DeviceObject,
//                                        NULL,
//                                        0,
//                                        NULL,
//                                        &Event,
//                                        &IoStatusBlock);
//     if (!Irp) return STATUS_INSUFFICIENT_RESOURCES;
//
//     IrpSp = IoGetNextIrpStackLocation(Irp);
//     IrpSp->MinorFunction = IRP_MN_SET_POWER;
//     IrpSp->Parameters.Power.Type = SystemPowerState;
//     IrpSp->Parameters.Power.State.SystemState = SystemState;
//     IrpSp->Parameters.Power.ShutdownType = PowerAction;
//
//     Status = PoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event,
//                               Executive,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//         Status = IoStatusBlock.Status;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// PopQuerySystemPowerStateTraverse(PDEVICE_NODE DeviceNode,
//                                  PVOID Context)
// {
//     PPOWER_STATE_TRAVERSE_CONTEXT PowerStateContext = Context;
//     PDEVICE_OBJECT TopDeviceObject;
//     NTSTATUS Status;
//
//     DPRINT("PopQuerySystemPowerStateTraverse(%p, %p)\n", DeviceNode, Context);
//
//     if (DeviceNode == IopRootDeviceNode)
//         return STATUS_SUCCESS;
//
//     if (DeviceNode->Flags & DNF_LEGACY_DRIVER)
//         return STATUS_SUCCESS;
//
//     TopDeviceObject = IoGetAttachedDeviceReference(DeviceNode->PhysicalDeviceObject);
//
//     Status = PopSendQuerySystemPowerState(TopDeviceObject,
//                                           PowerStateContext->SystemPowerState,
//                                           PowerStateContext->PowerAction);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Device '%wZ' failed IRP_MN_QUERY_POWER\n", &DeviceNode->InstancePath);
//     }
//     ObDereferenceObject(TopDeviceObject);
//
// #if 0
//     return Status;
// #else
//     return STATUS_SUCCESS;
// #endif
// }
//
// NTSTATUS
// PopSetSystemPowerStateTraverse(PDEVICE_NODE DeviceNode,
//                                PVOID Context)
// {
//     PPOWER_STATE_TRAVERSE_CONTEXT PowerStateContext = Context;
//     PDEVICE_OBJECT TopDeviceObject;
//     NTSTATUS Status;
//
//     DPRINT("PopSetSystemPowerStateTraverse(%p, %p)\n", DeviceNode, Context);
//
//     if (DeviceNode == IopRootDeviceNode)
//         return STATUS_SUCCESS;
//
//     if (DeviceNode->PhysicalDeviceObject == PowerStateContext->PowerDevice)
//         return STATUS_SUCCESS;
//
//     if (DeviceNode->Flags & DNF_LEGACY_DRIVER)
//         return STATUS_SUCCESS;
//
//     TopDeviceObject = IoGetAttachedDeviceReference(DeviceNode->PhysicalDeviceObject);
//     if (TopDeviceObject == PowerStateContext->PowerDevice)
//     {
//         ObDereferenceObject(TopDeviceObject);
//         return STATUS_SUCCESS;
//     }
//
//     Status = PopSendSetSystemPowerState(TopDeviceObject,
//                                         PowerStateContext->SystemPowerState,
//                                         PowerStateContext->PowerAction);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Device '%wZ' failed IRP_MN_SET_POWER\n", &DeviceNode->InstancePath);
//     }
//
//     ObDereferenceObject(TopDeviceObject);
//
// #if 0
//     return Status;
// #else
//     return STATUS_SUCCESS;
// #endif
// }
//
// NTSTATUS
// NTAPI
// PopSetSystemPowerState(SYSTEM_POWER_STATE PowerState, POWER_ACTION PowerAction)
// {
//     PDEVICE_OBJECT DeviceObject;
//     PDEVICE_OBJECT Fdo;
//     NTSTATUS Status;
//     DEVICETREE_TRAVERSE_CONTEXT Context;
//     POWER_STATE_TRAVERSE_CONTEXT PowerContext;
//
//     Status = IopGetSystemPowerDeviceObject(&DeviceObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("No system power driver available\n");
//         Fdo = NULL;
//     }
//     else
//     {
//         Fdo = IoGetAttachedDeviceReference(DeviceObject);
//         if (Fdo == DeviceObject)
//         {
//             DPRINT("An FDO was not attached\n");
//             return STATUS_UNSUCCESSFUL;
//         }
//     }
//
//     /* Set up context */
//     PowerContext.PowerAction = PowerAction;
//     PowerContext.SystemPowerState = PowerState;
//     PowerContext.PowerDevice = Fdo;
//
//     /* Query for system power change */
//     IopInitDeviceTreeTraverseContext(&Context,
//                                      IopRootDeviceNode,
//                                      PopQuerySystemPowerStateTraverse,
//                                      &PowerContext);
//
//     Status = IopTraverseDeviceTree(&Context);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Query system power state failed; changing state anyway\n");
//     }
//
//     /* Set system power change */
//     IopInitDeviceTreeTraverseContext(&Context,
//                                      IopRootDeviceNode,
//                                      PopSetSystemPowerStateTraverse,
//                                      &PowerContext);
//
//     IopTraverseDeviceTree(&Context);
//
//     if (!PopAcpiPresent) return STATUS_NOT_IMPLEMENTED;
//
//     if (Fdo != NULL)
//     {
//         if (PowerAction != PowerActionShutdownReset)
//             PopSendSetSystemPowerState(Fdo, PowerState, PowerAction);
//
//         ObDereferenceObject(Fdo);
//     }
//
//     return Status;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// PoInitSystem(IN ULONG BootPhase)
// {
//     PVOID NotificationEntry;
//     PCHAR CommandLine;
//     BOOLEAN ForceAcpiDisable = FALSE;
//
//     /* Check if this is phase 1 init */
//     if (BootPhase == 1)
//     {
//         NTSTATUS Status;
//         /* Register power button notification */
//         Status = IoRegisterPlugPlayNotification(EventCategoryDeviceInterfaceChange,
//                                                 PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES,
//                                                 (PVOID)&GUID_DEVICE_SYS_BUTTON,
//                                                 IopRootDeviceNode->PhysicalDeviceObject->DriverObject,
//                                                 PopAddRemoveSysCapsCallback,
//                                                 (PVOID)(ULONG_PTR)PolicyDeviceSystemButton,
//                                                 &NotificationEntry);
//         if (!NT_SUCCESS(Status))
//             return FALSE;
//
//         /* Register lid notification */
//         Status = IoRegisterPlugPlayNotification(EventCategoryDeviceInterfaceChange,
//                                                 PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES,
//                                                 (PVOID)&GUID_DEVICE_LID,
//                                                 IopRootDeviceNode->PhysicalDeviceObject->DriverObject,
//                                                 PopAddRemoveSysCapsCallback,
//                                                 (PVOID)(ULONG_PTR)PolicyDeviceSystemButton,
//                                                 &NotificationEntry);
//         if (!NT_SUCCESS(Status))
//             return FALSE;
//
//         /* Register battery notification */
//         Status = IoRegisterPlugPlayNotification(EventCategoryDeviceInterfaceChange,
//                                                 PNPNOTIFY_DEVICE_INTERFACE_INCLUDE_EXISTING_INTERFACES,
//                                                 (PVOID)&GUID_DEVICE_BATTERY,
//                                                 IopRootDeviceNode->PhysicalDeviceObject->DriverObject,
//                                                 PopAddRemoveSysCapsCallback,
//                                                 (PVOID)(ULONG_PTR)PolicyDeviceBattery,
//                                                 &NotificationEntry);
//
//         return NT_SUCCESS(Status);
//     }
//
//     /* Initialize the power capabilities */
//     RtlZeroMemory(&PopCapabilities, sizeof(SYSTEM_POWER_CAPABILITIES));
//
//     /* Get the Command Line */
//     CommandLine = KeLoaderBlock->LoadOptions;
//
//     /* Upcase it */
//     _strupr(CommandLine);
//
//     /* Check for ACPI disable */
//     if (strstr(CommandLine, "NOACPI")) ForceAcpiDisable = TRUE;
//
//     if (ForceAcpiDisable)
//     {
//         /* Set the ACPI State to False if it's been forced that way */
//         PopAcpiPresent = FALSE;
//     }
//     else
//     {
//         /* Otherwise check if the LoaderBlock has a ACPI Table  */
//         PopAcpiPresent = KeLoaderBlock->Extension->AcpiTable != NULL ? TRUE : FALSE;
//     }
//
//     /* Enable shutdown by power button */
//     if (PopAcpiPresent)
//         PopCapabilities.SystemS5 = TRUE;
//
//     /* Initialize volume support */
//     InitializeListHead(&PopVolumeDevices);
//     KeInitializeGuardedMutex(&PopVolumeLock);
//
//     /* Initialize support for dope */
//     KeInitializeSpinLock(&PopDopeGlobalLock);
//
//     /* Initialize support for shutdown waits and work-items */
//     PopInitShutdownList();
//
//     return TRUE;
// }
//
// VOID
// NTAPI
// PopPerfIdle(PPROCESSOR_POWER_STATE PowerState)
// {
//     DPRINT1("PerfIdle function: %p\n", PowerState);
// }
//
// VOID
// NTAPI
// PopPerfIdleDpc(IN PKDPC Dpc,
//                IN PVOID DeferredContext,
//                IN PVOID SystemArgument1,
//                IN PVOID SystemArgument2)
// {
//     /* Call the Perf Idle function */
//     PopPerfIdle(&((PKPRCB)DeferredContext)->PowerState);
// }
//
// VOID
// FASTCALL
// PopIdle0(IN PPROCESSOR_POWER_STATE PowerState)
// {
//     /* FIXME: Extremly naive implementation */
//     HalProcessorIdle();
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// PoInitializePrcb(IN PKPRCB Prcb)
// {
//     /* Initialize the Power State */
//     RtlZeroMemory(&Prcb->PowerState, sizeof(Prcb->PowerState));
//     Prcb->PowerState.Idle0KernelTimeLimit = 0xFFFFFFFF;
//     Prcb->PowerState.CurrentThrottle = 100;
//     Prcb->PowerState.CurrentThrottleIndex = 0;
//     Prcb->PowerState.IdleFunction = PopIdle0;
//
//     /* Initialize the Perf DPC and Timer */
//     KeInitializeDpc(&Prcb->PowerState.PerfDpc, PopPerfIdleDpc, Prcb);
//     KeSetTargetProcessorDpc(&Prcb->PowerState.PerfDpc, Prcb->Number);
//     KeInitializeTimerEx(&Prcb->PowerState.PerfTimer, SynchronizationTimer);
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// PoCancelDeviceNotify(IN PVOID NotifyBlock)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// PoRegisterDeviceNotify(OUT PVOID Unknown0,
//                        IN ULONG Unknown1,
//                        IN ULONG Unknown2,
//                        IN ULONG Unknown3,
//                        IN PVOID Unknown4,
//                        IN PVOID Unknown5)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// PoShutdownBugCheck(IN BOOLEAN LogError,
//                    IN ULONG BugCheckCode,
//                    IN ULONG_PTR BugCheckParameter1,
//                    IN ULONG_PTR BugCheckParameter2,
//                    IN ULONG_PTR BugCheckParameter3,
//                    IN ULONG_PTR BugCheckParameter4)
// {
//     DPRINT1("PoShutdownBugCheck called\n");
//
//     /* FIXME: Log error if requested */
//     /* FIXME: Initiate a shutdown */
//
//     /* Bugcheck the system */
//     KeBugCheckEx(BugCheckCode,
//                  BugCheckParameter1,
//                  BugCheckParameter2,
//                  BugCheckParameter3,
//                  BugCheckParameter4);
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// PoSetHiberRange(IN PVOID HiberContext,
//                 IN ULONG Flags,
//                 IN OUT PVOID StartPage,
//                 IN ULONG Length,
//                 IN ULONG PageTag)
// {
//     UNIMPLEMENTED;
//     return;
// }
//
// /*
//  * @implemented
//  */
// _IRQL_requires_max_(DISPATCH_LEVEL)
// NTSTATUS
// NTAPI
// PoCallDriver(
//     _In_ PDEVICE_OBJECT DeviceObject,
//     _Inout_ __drv_aliasesMem PIRP Irp)
// {
//     PIO_STACK_LOCATION NextStack;
//
//     ASSERT(KeGetCurrentIrql() <= DISPATCH_LEVEL);
//
//     ASSERT(DeviceObject);
//     ASSERT(Irp);
//
//     NextStack = IoGetNextIrpStackLocation(Irp);
//     ASSERT(NextStack->MajorFunction == IRP_MJ_POWER);
//
//     /* Set DeviceObject for PopPresentIrp */
//     NextStack->DeviceObject = DeviceObject;
//
//     /* Only QUERY_POWER and SET_POWER use special handling */
//     if (NextStack->MinorFunction != IRP_MN_SET_POWER &&
//         NextStack->MinorFunction != IRP_MN_QUERY_POWER)
//     {
//         return IoCallDriver(DeviceObject, Irp);
//     }
//
//     /* Call the next driver, either directly or at PASSIVE_LEVEL */
//     return PopPresentIrp(NextStack, Irp);
// }
//
// /*
//  * @unimplemented
//  */
// PULONG
// NTAPI
// PoRegisterDeviceForIdleDetection(IN PDEVICE_OBJECT DeviceObject,
//                                  IN ULONG ConservationIdleTime,
//                                  IN ULONG PerformanceIdleTime,
//                                  IN DEVICE_POWER_STATE State)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @unimplemented
//  */
// PVOID
// NTAPI
// PoRegisterSystemState(IN PVOID StateHandle,
//                       IN EXECUTION_STATE Flags)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// PoRequestPowerIrp(
//     _In_ PDEVICE_OBJECT DeviceObject,
//     _In_ UCHAR MinorFunction,
//     _In_ POWER_STATE PowerState,
//     _In_opt_ PREQUEST_POWER_COMPLETE CompletionFunction,
//     _In_opt_ __drv_aliasesMem PVOID Context,
//     _Outptr_opt_ PIRP *pIrp)
// {
//     PDEVICE_OBJECT TopDeviceObject;
//     PIO_STACK_LOCATION Stack;
//     PIRP Irp;
//
//     if (MinorFunction != IRP_MN_QUERY_POWER
//         && MinorFunction != IRP_MN_SET_POWER
//         && MinorFunction != IRP_MN_WAIT_WAKE)
//         return STATUS_INVALID_PARAMETER_2;
//
//     /* Always call the top of the device stack */
//     TopDeviceObject = IoGetAttachedDeviceReference(DeviceObject);
//
//     Irp = IoAllocateIrp(TopDeviceObject->StackSize + 2, FALSE);
//     if (!Irp)
//     {
//         ObDereferenceObject(TopDeviceObject);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//     Irp->IoStatus.Information = 0;
//
//     IoSetNextIrpStackLocation(Irp);
//
//     Stack = IoGetNextIrpStackLocation(Irp);
//     Stack->Parameters.Others.Argument1 = DeviceObject;
//     Stack->Parameters.Others.Argument2 = (PVOID)(ULONG_PTR)MinorFunction;
//     Stack->Parameters.Others.Argument3 = (PVOID)(ULONG_PTR)PowerState.DeviceState;
//     Stack->Parameters.Others.Argument4 = Context;
//     Stack->DeviceObject = TopDeviceObject;
//     IoSetNextIrpStackLocation(Irp);
//
//     Stack = IoGetNextIrpStackLocation(Irp);
//     Stack->MajorFunction = IRP_MJ_POWER;
//     Stack->MinorFunction = MinorFunction;
//     if (MinorFunction == IRP_MN_WAIT_WAKE)
//     {
//         Stack->Parameters.WaitWake.PowerState = PowerState.SystemState;
//     }
//     else
//     {
//         Stack->Parameters.Power.Type = DevicePowerState;
//         Stack->Parameters.Power.State = PowerState;
//     }
//
//     if (pIrp != NULL)
//         *pIrp = Irp;
//
//     IoSetCompletionRoutine(Irp, PopRequestPowerIrpCompletion, CompletionFunction, TRUE, TRUE, TRUE);
//     PoCallDriver(TopDeviceObject, Irp);
//
//     /* Always return STATUS_PENDING. The completion routine
//      * will call CompletionFunction and complete the Irp.
//      */
//     return STATUS_PENDING;
// }
//
// /*
//  * @unimplemented
//  */
// POWER_STATE
// NTAPI
// PoSetPowerState(IN PDEVICE_OBJECT DeviceObject,
//                 IN POWER_STATE_TYPE Type,
//                 IN POWER_STATE State)
// {
//     POWER_STATE ps;
//
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     ps.SystemState = PowerSystemWorking;  // Fully on
//     ps.DeviceState = PowerDeviceD0;       // Fully on
//
//     return ps;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// PoSetSystemState(IN EXECUTION_STATE Flags)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// PoStartNextPowerIrp(IN PIRP Irp)
// {
//     UNIMPLEMENTED_ONCE;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// NTAPI
// PoUnregisterSystemState(IN PVOID StateHandle)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// NtInitiatePowerAction(IN POWER_ACTION SystemAction,
//                       IN SYSTEM_POWER_STATE MinSystemState,
//                       IN ULONG Flags,
//                       IN BOOLEAN Asynchronous)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// static
// NTSTATUS
// PopQueryBatteryState(
//     _Out_ PSYSTEM_BATTERY_STATE BatteryState)
// {
//     static const UNICODE_STRING CompBattDeviceName =
//         RTL_CONSTANT_STRING(L"\\Device\\CompositeBattery");
//     OBJECT_ATTRIBUTES ObjectAttributes = RTL_CONSTANT_OBJECT_ATTRIBUTES(
//         &CompBattDeviceName, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE);
//     HANDLE CompBattHandle;
//     IO_STATUS_BLOCK IoStatusBlock;
//     BATTERY_QUERY_INFORMATION BatteryQueryInfo = {0};
//     BATTERY_INFORMATION BatteryInfo = { 0 };
//     BATTERY_STATUS BatteryStatus;
//     NTSTATUS Status;
//
//     /* Open the CompositeBattery device */
//     Status = ZwOpenFile(&CompBattHandle,
//                         GENERIC_READ | SYNCHRONIZE,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE,
//                         FILE_SYNCHRONOUS_IO_NONALERT);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to open CompositeBattery device: Status: 0x%08lX\n", Status);
//         return Status;
//     }
//
//     /* Query the battery tag */
//     ULONG Wait = 0;
//     Status = ZwDeviceIoControlFile(CompBattHandle,
//                                    NULL,
//                                    NULL,
//                                    NULL,
//                                    &IoStatusBlock,
//                                    IOCTL_BATTERY_QUERY_TAG,
//                                    &Wait,
//                                    sizeof(Wait),
//                                    &BatteryQueryInfo.BatteryTag,
//                                    sizeof(BatteryQueryInfo.BatteryTag));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to query battery tag: Status: 0x%08lX\n", Status);
//         goto Exit;
//     }
//
//     /* Query the battery information */
//     BatteryQueryInfo.InformationLevel = BatteryInformation as u32;
//     Status = ZwDeviceIoControlFile(CompBattHandle,
//                                    NULL,
//                                    NULL,
//                                    NULL,
//                                    &IoStatusBlock,
//                                    IOCTL_BATTERY_QUERY_INFORMATION,
//                                    &BatteryQueryInfo,
//                                    sizeof(BatteryQueryInfo),
//                                    &BatteryInfo,
//                                    sizeof(BatteryInfo));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to query battery information: Status: 0x%08lX\n", Status);
//     }
//
//     /* Query the battery status */
//     BATTERY_WAIT_STATUS BatteryWait = {0};
//     BatteryWait.BatteryTag = BatteryQueryInfo.BatteryTag;
//     BatteryWait.PowerState = 0xF ; // Wait for all power states
//     BatteryWait.HighCapacity = MAXULONG;
//     Status = ZwDeviceIoControlFile(CompBattHandle,
//                                    NULL,
//                                    NULL,
//                                    NULL,
//                                    &IoStatusBlock,
//                                    IOCTL_BATTERY_QUERY_STATUS,
//                                    &BatteryWait,
//                                    sizeof(BatteryWait),
//                                    &BatteryStatus,
//                                    sizeof(BatteryStatus));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to query battery status: Status: 0x%08lX\n", Status);
//         goto Exit;
//     }
//
//     RtlZeroMemory(BatteryState, sizeof(*BatteryState));
//     BatteryState->BatteryPresent = TRUE;
//     BatteryState->AcOnLine = BooleanFlagOn(BatteryStatus.PowerState, BATTERY_POWER_ON_LINE);
//     BatteryState->Charging = BooleanFlagOn(BatteryStatus.PowerState, BATTERY_CHARGING);
//     BatteryState->Discharging = BooleanFlagOn(BatteryStatus.PowerState, BATTERY_DISCHARGING);
//     BatteryState->MaxCapacity = BatteryInfo.FullChargedCapacity;
//     BatteryState->RemainingCapacity = BatteryStatus.Capacity;
//     BatteryState->Rate = BatteryStatus.Rate;
//     if (BatteryState->Discharging && BatteryStatus.Rate < 0)
//     {
//         BatteryState->EstimatedTime =
//             3600 * BatteryInfo.FullChargedCapacity / -BatteryStatus.Rate;
//     }
//     else
//     {
//         BatteryState->EstimatedTime = 0;
//     }
//
// Exit:
//     ZwClose(CompBattHandle);
//     return Status;
// }
//     
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// NtPowerInformation(IN POWER_INFORMATION_LEVEL PowerInformationLevel,
//                    IN PVOID InputBuffer  OPTIONAL,
//                    IN ULONG InputBufferLength,
//                    OUT PVOID OutputBuffer  OPTIONAL,
//                    IN ULONG OutputBufferLength)
// {
//     NTSTATUS Status;
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//
//     PAGED_CODE();
//
//     DPRINT("NtPowerInformation(PowerInformationLevel 0x%x, InputBuffer 0x%p, "
//            "InputBufferLength 0x%x, OutputBuffer 0x%p, OutputBufferLength 0x%x)\n",
//            PowerInformationLevel,
//            InputBuffer, InputBufferLength,
//            OutputBuffer, OutputBufferLength);
//
//     if (PreviousMode != KernelMode)
//     {
//         _SEH2_TRY
//         {
//             ProbeForRead(InputBuffer, InputBufferLength, 1);
//             ProbeForWrite(OutputBuffer, OutputBufferLength, sizeof(ULONG));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     switch (PowerInformationLevel)
//     {
//         case SystemBatteryState:
//         {
//             SYSTEM_BATTERY_STATE BatteryState;
//
//             if (InputBuffer != NULL)
//                 return STATUS_INVALID_PARAMETER;
//             if (OutputBufferLength < sizeof(SYSTEM_BATTERY_STATE))
//                 return STATUS_BUFFER_TOO_SMALL;
//
//             if (PopCapabilities.SystemBatteriesPresent)
//             {
//                 /* Open the battery driver and query the battery state */
//                 Status = PopQueryBatteryState(&BatteryState);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     DPRINT1("Failed to query battery state: Status: 0x%08X\n", Status);
//                     return Status;
//                 }
//             }
//             else
//             {
//                 /* No batteries present, set defaults */
//                 RtlZeroMemory(&BatteryState, sizeof(BatteryState));
//                 BatteryState.AcOnLine = TRUE;
//             }
//
//             _SEH2_TRY
//             {
//                 /* Copy the battery state to the output buffer */
//                 RtlCopyMemory(OutputBuffer, &BatteryState, sizeof(BatteryState));
//                 Status = STATUS_SUCCESS;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             break;
//         }
//
//         case SystemPowerCapabilities:
//         {
//             PSYSTEM_POWER_CAPABILITIES PowerCapabilities = (PSYSTEM_POWER_CAPABILITIES)OutputBuffer;
//
//             if (InputBuffer != NULL)
//                 return STATUS_INVALID_PARAMETER;
//             if (OutputBufferLength < sizeof(SYSTEM_POWER_CAPABILITIES))
//                 return STATUS_BUFFER_TOO_SMALL;
//
//             _SEH2_TRY
//             {
//                 RtlCopyMemory(PowerCapabilities,
//                               &PopCapabilities,
//                               sizeof(SYSTEM_POWER_CAPABILITIES));
//
//                 Status = STATUS_SUCCESS;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             break;
//         }
//
//         case ProcessorInformation:
//         {
//             PPROCESSOR_POWER_INFORMATION PowerInformation = (PPROCESSOR_POWER_INFORMATION)OutputBuffer;
//
//             if (InputBuffer != NULL)
//                 return STATUS_INVALID_PARAMETER;
//             if (OutputBufferLength < sizeof(PROCESSOR_POWER_INFORMATION))
//                 return STATUS_BUFFER_TOO_SMALL;
//
//             /* FIXME: return structures for all processors */
//
//             _SEH2_TRY
//             {
//                 /* FIXME: some values are hardcoded */
//                 PowerInformation->Number = 0;
//                 PowerInformation->MaxMhz = 1000;
//                 PowerInformation->CurrentMhz = KeGetCurrentPrcb()->MHz;
//                 PowerInformation->MhzLimit = 1000;
//                 PowerInformation->MaxIdleState = 0;
//                 PowerInformation->CurrentIdleState = 0;
//
//                 Status = STATUS_SUCCESS;
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             break;
//         }
//
//         default:
//             Status = STATUS_NOT_IMPLEMENTED;
//             DPRINT1("PowerInformationLevel 0x%x is UNIMPLEMENTED! Have a nice day.\n",
//                     PowerInformationLevel);
//             break;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtGetDevicePowerState(IN HANDLE Device,
//                       IN PDEVICE_POWER_STATE PowerState)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// BOOLEAN
// NTAPI
// NtIsSystemResumeAutomatic(VOID)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// NTSTATUS
// NTAPI
// NtRequestWakeupLatency(IN LATENCY_TIME Latency)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// NtSetThreadExecutionState(IN EXECUTION_STATE esFlags,
//                           OUT EXECUTION_STATE *PreviousFlags)
// {
//     PKTHREAD Thread = KeGetCurrentThread();
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//     EXECUTION_STATE PreviousState;
//     PAGED_CODE();
//
//     /* Validate flags */
//     if (esFlags & ~(ES_CONTINUOUS | ES_USER_PRESENT))
//     {
//         /* Fail the request */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check for user parameters */
//     if (PreviousMode != KernelMode)
//     {
//         /* Protect the probes */
//         _SEH2_TRY
//         {
//             /* Check if the pointer is valid */
//             ProbeForWriteUlong(PreviousFlags);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* It isn't -- fail */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Save the previous state, always masking in the continous flag */
//     PreviousState = Thread->PowerState | ES_CONTINUOUS;
//
//     /* Check if we need to update the power state */
//     if (esFlags & ES_CONTINUOUS) Thread->PowerState = (UCHAR)esFlags;
//
//     /* Protect the write back to user mode */
//     _SEH2_TRY
//     {
//         /* Return the previous flags */
//         *PreviousFlags = PreviousState;
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Something's wrong, fail */
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     /* All is good */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// NtSetSystemPowerState(IN POWER_ACTION SystemAction,
//                       IN SYSTEM_POWER_STATE MinSystemState,
//                       IN ULONG Flags)
// {
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//     POP_POWER_ACTION Action = {0};
//     NTSTATUS Status;
//     ULONG Dummy;
//
//     /* Check for invalid parameter combinations */
//     if ((MinSystemState >= PowerSystemMaximum) ||
//         (MinSystemState <= PowerSystemUnspecified) ||
//         (SystemAction > PowerActionWarmEject) ||
//         (SystemAction < PowerActionReserved) ||
//         (Flags & ~(POWER_ACTION_QUERY_ALLOWED  |
//                    POWER_ACTION_UI_ALLOWED     |
//                    POWER_ACTION_OVERRIDE_APPS  |
//                    POWER_ACTION_LIGHTEST_FIRST |
//                    POWER_ACTION_LOCK_CONSOLE   |
//                    POWER_ACTION_DISABLE_WAKES  |
//                    POWER_ACTION_CRITICAL)))
//     {
//         DPRINT1("NtSetSystemPowerState: Bad parameters!\n");
//         DPRINT1("                       SystemAction: 0x%x\n", SystemAction);
//         DPRINT1("                       MinSystemState: 0x%x\n", MinSystemState);
//         DPRINT1("                       Flags: 0x%x\n", Flags);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check for user caller */
//     if (PreviousMode != KernelMode)
//     {
//         /* Check for shutdown permission */
//         if (!SeSinglePrivilegeCheck(SeShutdownPrivilege, PreviousMode))
//         {
//             /* Not granted */
//             DPRINT1("ERROR: Privilege not held for shutdown\n");
//             return STATUS_PRIVILEGE_NOT_HELD;
//         }
//
//         /* Do it as a kernel-mode caller for consistency with system state */
//         return ZwSetSystemPowerState(SystemAction, MinSystemState, Flags);
//     }
//
//     /* Read policy settings (partial shutdown vs. full shutdown) */
//     if (SystemAction == PowerActionShutdown) PopReadShutdownPolicy();
//
//     /* Disable lazy flushing of registry */
//     DPRINT("Stopping lazy flush\n");
//     CmSetLazyFlushState(FALSE);
//
//     /* Setup the power action */
//     Action.Action = SystemAction;
//     Action.Flags = Flags;
//
//     /* Notify callbacks */
//     DPRINT("Notifying callbacks\n");
//     ExNotifyCallback(PowerStateCallback, (PVOID)3, NULL);
//
//     /* Swap in any worker thread stacks */
//     DPRINT("Swapping worker threads\n");
//     ExSwapinWorkerThreads(FALSE);
//
//     /* Make our action global */
//     PopAction = Action;
//
//     /* Start power loop */
//     Status = STATUS_CANCELLED;
//     while (TRUE)
//     {
//         /* Break out if there's nothing to do */
//         if (Action.Action == PowerActionNone) break;
//
//         /* Check for first-pass or restart */
//         if (Status == STATUS_CANCELLED)
//         {
//             /* Check for shutdown action */
//             if ((PopAction.Action == PowerActionShutdown) ||
//                 (PopAction.Action == PowerActionShutdownReset) ||
//                 (PopAction.Action == PowerActionShutdownOff))
//             {
//                 /* Set the action */
//                 PopAction.Shutdown = TRUE;
//             }
//
//             /* Now we are good to go */
//             Status = STATUS_SUCCESS;
//         }
//
//         /* Check if we're still in an invalid status */
//         if (!NT_SUCCESS(Status)) break;
//
//         /* Flush all volumes and the registry */
//         DPRINT("Flushing volumes\n");
//         PopFlushVolumes(PopAction.Shutdown);
//
// #ifndef NEWCC
//         /* Flush dirty cache pages */
//         /* XXX: Is that still mandatory? As now we'll wait on lazy writer to complete? */
//         CcRosFlushDirtyPages(MAXULONG, &Dummy, TRUE, FALSE);
//         DPRINT("Cache flushed %lu pages\n", Dummy);
// #else
//         Dummy = 0;
// #endif
//
//         /* Set IRP for drivers */
//         PopAction.IrpMinor = IRP_MN_SET_POWER;
//         if (PopAction.Shutdown)
//         {
//             DPRINT("Queueing shutdown thread\n");
//             /* Check if we are running in the system context */
//             if (PsGetCurrentProcess() != PsInitialSystemProcess)
//             {
//                 /* We're not, so use a worker thread for shutdown */
//                 ExInitializeWorkItem(&PopShutdownWorkItem,
//                                      &PopGracefulShutdown,
//                                      NULL);
//
//                 ExQueueWorkItem(&PopShutdownWorkItem, CriticalWorkQueue);
//
//                 /* Spend us -- when we wake up, the system is good to go down */
//                 KeSuspendThread(KeGetCurrentThread());
//                 Status = STATUS_SYSTEM_SHUTDOWN;
//                 goto Exit;
//
//             }
//             else
//             {
//                 /* Do the shutdown inline */
//                 PopGracefulShutdown(NULL);
//             }
//         }
//
//         /* You should not have made it this far */
//         // ASSERTMSG("System is still up and running?!\n", FALSE);
//         DPRINT1("System is still up and running, you may not have chosen a yet supported power option: %u\n", PopAction.Action);
//         break;
//     }
//
// Exit:
//     /* We're done, return */
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
