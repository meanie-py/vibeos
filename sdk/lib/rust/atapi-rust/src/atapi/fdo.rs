// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_01D8877A5F07: [u16; 23] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0049, 0x0064,
    0x0065, 0x005C, 0x0049, 0x0064, 0x0065, 0x0050, 0x006F, 0x0072, 0x0074, 0x0025,
    0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_30996DDBD6F9: [u16; 21] = [
    0x005Cu16, 0x0044, 0x006F, 0x0073, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065,
    0x0073, 0x005C, 0x0053, 0x0063, 0x0073, 0x0069, 0x0025, 0x006C, 0x0075, 0x003A,
    0x0000,
];
pub static RUSTCONV_WSTR_499DA777F3BC: [u16; 20] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0053, 0x0063,
    0x0073, 0x0069, 0x0050, 0x006F, 0x0072, 0x0074, 0x0025, 0x006C, 0x0075, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, IoDetachDevice, IoForwardIrpSynchronously, IoGetConfigurationInformation, IoInitializeTimer, IoRegisterDeviceInterface, IoSetDeviceInterfaceState, IoStartTimer, IoStopTimer, KeInitializeDpc, KeInitializeEvent, KeWaitForSingleObject, MmAllocateContiguousMemorySpecifyCache, MmAllocateMappingAddress, MmFreeContiguousMemorySpecifyCache, MmFreeMappingAddress, MmGetPhysicalAddress, _EVENT_TYPE_NotificationEvent, _OBJECT_ATTRIBUTES, ObReferenceObjectByHandle, PCM_RESOURCE_LIST, PHYSICAL_ADDRESS, PIO_STACK_LOCATION, PIRP, PsCreateSystemThread, RtlFreeUnicodeString, RtlInitUnicodeString, ZwClose};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut AtapDevSymLinkFormat: *const u16 = RUSTCONV_WSTR_499DA777F3BC.as_ptr();
pub static mut AtapDosSymLinkFormat: *const u16 = RUSTCONV_WSTR_30996DDBD6F9.as_ptr();

unsafe extern "C" fn AtaFdoCreateSymLinks(
    ChanExt: *mut ATAPORT_CHANNEL_EXTENSION,
) -> NTSTATUS {
    let mut ChannelName: _UNICODE_STRING = core::mem::zeroed();
    let mut ScsiAdapter: u32;
    let mut ScsiPortCount: u32;
    let mut ChannelNameBuffer: [u16; 28] = [0; 28];
    let mut Status: NTSTATUS;
    let FdoFormat: *const u16 = RUSTCONV_WSTR_01D8877A5F07.as_ptr();

    PAGED_CODE!();

    if (*ChanExt).PortData.PortFlags & PORT_FLAG_SYMLINK_CREATED != 0 {
        return STATUS_SUCCESS;
    }

    Status = RtlStringCbPrintfW!(
        ChannelNameBuffer.as_mut_ptr(),
        core::mem::size_of::<[u16; 28]>() as u32,
        FdoFormat,
        (*ChanExt).DeviceObjectNumber,
    );
    ASSERT!(NT_SUCCESS!(Status));

    RtlInitUnicodeString(&mut ChannelName, ChannelNameBuffer.as_mut_ptr());

    ScsiPortCount = (*IoGetConfigurationInformation()).ScsiPortCount;

    ScsiAdapter = 0;
    while ScsiAdapter <= ScsiPortCount {
        let mut SymLinkNameBuffer: [u16; 24] = [0; 24];
        let mut SymLinkName: _UNICODE_STRING = core::mem::zeroed();

        Status = RtlStringCbPrintfW!(
            SymLinkNameBuffer.as_mut_ptr(),
            core::mem::size_of::<[u16; 24]>() as u32,
            AtapDevSymLinkFormat,
            ScsiAdapter,
        );
        ASSERT!(NT_SUCCESS!(Status));

        RtlInitUnicodeString(&mut SymLinkName, SymLinkNameBuffer.as_mut_ptr());

        Status = IoCreateSymbolicLink(&mut SymLinkName, &mut ChannelName);
        if !NT_SUCCESS!(Status) {
            ScsiAdapter += 1;
            continue;
        }

        INFO!(
            b"Symlink created '%wZ' -> '%wZ'\n\0".as_ptr() as *const i8,
            &SymLinkName,
            &ChannelName,
        );

        Status = RtlStringCbPrintfW!(
            SymLinkNameBuffer.as_mut_ptr(),
            core::mem::size_of::<[u16; 24]>() as u32,
            AtapDosSymLinkFormat,
            ScsiAdapter,
        );
        ASSERT!(NT_SUCCESS!(Status));

        RtlInitUnicodeString(&mut SymLinkName, SymLinkNameBuffer.as_mut_ptr());

        Status = IoCreateSymbolicLink(&mut SymLinkName, &mut ChannelName);
        if NT_SUCCESS!(Status) {
            INFO!(
                b"Symlink created '%wZ' -> '%wZ'\n\0".as_ptr() as *const i8,
                &SymLinkName,
                &ChannelName,
            );
        }

        (*IoGetConfigurationInformation()).ScsiPortCount += 1;

        (*ChanExt).ScsiPortNumber = ScsiAdapter;
        (*ChanExt).PortData.PortFlags |= PORT_FLAG_SYMLINK_CREATED;
        break;
    }

    Status
}

unsafe extern "C" fn AtaFdoAllocateLocalBuffer(
    PortData: PATAPORT_PORT_DATA,
) -> NTSTATUS {
    let mut PhysicalAddress: PHYSICAL_ADDRESS;
    let mut HighestAcceptableAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut LowestAcceptableAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut BoundaryAddressMultiple: PHYSICAL_ADDRESS = core::mem::zeroed();

    PAGED_CODE!();

    if !(*PortData).LocalBuffer.is_null() {
        return STATUS_SUCCESS;
    }

    LowestAcceptableAddress.QuadPart = 0;
    HighestAcceptableAddress.QuadPart = 0xFFFFFFFF; // 32-bit DMA
    BoundaryAddressMultiple.QuadPart = 0x10000; // 64k, for PATA compability
    (*PortData).LocalBuffer = MmAllocateContiguousMemorySpecifyCache(
        (ATA_LOCAL_BUFFER_SIZE) as SIZE_T,
        LowestAcceptableAddress,
        HighestAcceptableAddress,
        BoundaryAddressMultiple,
        MmNonCached,
    );
    if (*PortData).LocalBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    PhysicalAddress = MmGetPhysicalAddress((*PortData).LocalBuffer);

    (*PortData).LocalSgList.NumberOfElements = 1;
    (*PortData).LocalSgList.Elements[0].Length = ATA_LOCAL_BUFFER_SIZE;
    (*PortData).LocalSgList.Elements[0].Address.QuadPart = PhysicalAddress.QuadPart;

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaFdoCreatePortThread(
    PortData: *mut ATAPORT_PORT_DATA,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = RTL_CONSTANT_OBJECT_ATTRIBUTES(core::ptr::null_mut(), OBJ_KERNEL_HANDLE);
    let mut ThreadHandle: HANDLE = core::mem::zeroed();
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    if (*PortData).Worker.Thread != core::ptr::null_mut() {
        return STATUS_SUCCESS;
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!((*PortData).Worker.ThreadEvent),
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );

    Status = PsCreateSystemThread(
        core::ptr::addr_of_mut!(ThreadHandle),
        THREAD_ALL_ACCESS,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Some(AtaPortWorkerThread),
        (PortData).cast(),
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    Status = ObReferenceObjectByHandle(
        ThreadHandle,
        THREAD_ALL_ACCESS,
        core::ptr::null_mut(),
        (0i32) as KPROCESSOR_MODE,
        core::ptr::addr_of_mut!((*PortData).Worker.Thread) as *mut PVOID,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    ZwClose(ThreadHandle);

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaFdoDestroyPortThread(
    ChanExt: *mut ATAPORT_CHANNEL_EXTENSION,
) {
    let PortData: *mut ATAPORT_PORT_DATA = &mut (*ChanExt).PortData;

    PAGED_CODE!();

    if (*PortData).Worker.Thread.is_null() {
        return;
    }

    (*PortData).PortFlags |= PORT_FLAG_EXIT_THREAD;
    AtaPortSignalWorkerThread((PortData).cast());

    KeWaitForSingleObject(
        ((*PortData).Worker.Thread).cast(),
        _KWAIT_REASON_Executive,
        (0i32) as KPROCESSOR_MODE,
        FALSE,
        core::ptr::null_mut(),
    );

    ObDereferenceObject(((*PortData).Worker.Thread).cast());

    (*PortData).Worker.Thread = core::ptr::null_mut();
}

unsafe extern "C" fn AtaFdoRemoveSymLinks(
    ChanExt: *mut ATAPORT_CHANNEL_EXTENSION,
) {
    let mut Status: NTSTATUS;
    let mut SymLinkName: _UNICODE_STRING = core::mem::zeroed();
    let mut SymLinkNameBuffer: [u16; 22] = [0; 22];

    PAGED_CODE!();

    if ((*ChanExt).PortData.PortFlags & PORT_FLAG_SYMLINK_CREATED) == 0 {
        return;
    }

    /* Delete the '\DosDevices\\ScsiX:' symbolic link */
    Status = RtlStringCbPrintfW!(
        SymLinkNameBuffer.as_mut_ptr(),
        core::mem::size_of::<[u16; 22]>() as usize,
        AtapDevSymLinkFormat,
        (*ChanExt).ScsiPortNumber,
    );
    ASSERT!(NT_SUCCESS!(Status));

    RtlInitUnicodeString(
        &mut SymLinkName,
        SymLinkNameBuffer.as_mut_ptr(),
    );
    let _ = IoDeleteSymbolicLink(&mut SymLinkName);

    /* Delete the '\Device\\ScsiPortX' symbolic link */
    Status = RtlStringCbPrintfW!(
        SymLinkNameBuffer.as_mut_ptr(),
        core::mem::size_of::<[u16; 22]>() as usize,
        AtapDosSymLinkFormat,
        (*ChanExt).ScsiPortNumber,
    );
    ASSERT!(NT_SUCCESS!(Status));

    RtlInitUnicodeString(
        &mut SymLinkName,
        SymLinkNameBuffer.as_mut_ptr(),
    );
    let _ = IoDeleteSymbolicLink(&mut SymLinkName);

    /* Unregister the SCSI port adapter */
    (*IoGetConfigurationInformation()).ScsiPortCount -= 1;
}
#[no_mangle]
pub unsafe extern "C" fn AtaFdoStartDevice(
    ChanExt: PATAPORT_CHANNEL_EXTENSION,
    ResourcesTranslated: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let PortData: PATAPORT_PORT_DATA = &mut (*ChanExt).PortData;
    let mut ChannelInterface: PCIIDEX_CHANNEL_INTERFACE = core::mem::zeroed();
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    INFO!(
        b"Starting channel %lu\n\0".as_ptr() as *const i8,
        (*ChanExt).DeviceObjectNumber,
    );

    Status = AtaPnpQueryInterface(
        &mut (*ChanExt).Common,
        &GUID_PCIIDE_INTERFACE_ROS,
        &mut ChannelInterface as *mut PCIIDEX_CHANNEL_INTERFACE as *mut core::ffi::c_void,
        PCIIDEX_INTERFACE_VERSION,
        (core::mem::size_of::<PCIIDEX_CHANNEL_INTERFACE>()) as _,
    );
    if !NT_SUCCESS!(Status) {
        ERR!(b"Failed to query channel interface %lx\n\0".as_ptr() as *const i8, Status);
        return Status;
    }
    (*PortData).ChannelContext = ChannelInterface.ChannelContext;
    (*PortData).AttachChannel = ChannelInterface.AttachChannel;
    (*PortData).SetDeviceData = ChannelInterface.SetDeviceData;
    (*PortData).GetInitTaskFile = ChannelInterface.GetInitTaskFile;
    (*PortData).DowngradeInterfaceSpeed = ChannelInterface.DowngradeInterfaceSpeed;
    (*PortData).InterruptObject = ChannelInterface.InterruptObject;
    (*PortData).AbortChannel = ChannelInterface.AbortChannel;
    (*PortData).ResetChannel = ChannelInterface.ResetChannel;
    (*PortData).EnumerateChannel = ChannelInterface.EnumerateChannel;
    (*PortData).IdentifyDevice = ChannelInterface.IdentifyDevice;
    (*PortData).SetTransferMode = ChannelInterface.SetTransferMode;
    (*PortData).AllocateSlot = ChannelInterface.AllocateSlot;
    (*PortData).PreparePrdTable = ChannelInterface.PreparePrdTable;
    (*PortData).PrepareIo = ChannelInterface.PrepareIo;
    (*PortData).StartIo = ChannelInterface.StartIo;
    (*PortData).MaxTargetId = ChannelInterface.MaxTargetId;
    (*PortData).MaximumTransferLength = ChannelInterface.MaximumTransferLength;
    (*PortData).MaximumPhysicalPages = ChannelInterface.MaximumPhysicalPages;
    (*PortData).QueueDepth = ChannelInterface.QueueDepth;
    (*PortData).PortNumber = ChannelInterface.Channel;
    (*PortData).DmaAdapter = ChannelInterface.DmaAdapter;
    (*PortData).ChannelObject = ChannelInterface.ChannelObject;

    (*PortData).InterruptFlags = (PORT_INT_FLAG_IS_IO_ACTIVE) as i32;
    (*PortData).FreeSlotsBitmap = NUM_TO_BITMAP!((*PortData).QueueDepth);
    (*PortData).MaxSlotsBitmap = (*PortData).FreeSlotsBitmap;
    (*PortData).LastUsedSlot = (RTL_BITS_OF!(ULONG) - 1) as _;

    if (((ChannelInterface.TransferModeSupported & !PIO_ALL)) == 0) {
        (*PortData).PortFlags |= PORT_FLAG_PIO_ONLY;
    }

    if ChannelInterface.Flags & ATA_CHANNEL_FLAG_PIO_VIA_DMA != 0 {
        (*PortData).PortFlags |= PORT_FLAG_PIO_VIA_DMA;
    }

    if ChannelInterface.Flags & ATA_CHANNEL_FLAG_NCQ != 0 {
        (*PortData).PortFlags |= PORT_FLAG_NCQ;
    }

    if ChannelInterface.Flags & ATA_CHANNEL_FLAG_IS_AHCI != 0 {
        (*PortData).PortFlags |= PORT_FLAG_IS_AHCI;
    }

    if ChannelInterface.Flags & ATA_CHANNEL_FLAG_IS_EXTERNAL != 0 {
        (*PortData).PortFlags |= PORT_FLAG_IS_EXTERNAL;
    }

    if ChannelInterface.Flags & ATA_CHANNEL_FLAG_PIO_FOR_LBA48_XFER != 0 {
        (*PortData).PortFlags |= PORT_FLAG_PIO_FOR_LBA48_XFER;
    }

    if !ChannelInterface.HwSyncObject.is_null() {
        (*PortData).PortFlags |= PORT_FLAG_IS_SIMPLEX;
        (*PortData).HwSyncObject = ChannelInterface.HwSyncObject;
    }

    if !(ChannelInterface.Flags & ATA_CHANNEL_FLAG_PIO_VIA_DMA != 0) && (*PortData).ReservedVaSpace.is_null() {
        (*PortData).ReservedVaSpace = MmAllocateMappingAddress(
            (ATA_RESERVED_PAGES * PAGE_SIZE) as SIZE_T,
            ATAPORT_TAG,
        );
    }

    Status = AtaFdoAllocateLocalBuffer(PortData);
    if !NT_SUCCESS!(Status) {
        ERR!(b"CH %lu: Failed to allocate local buffer 0x%lx\n\0".as_ptr() as *const i8, (*PortData).PortNumber, Status);
        return Status;
    }

    InitializeListHead(&mut (*PortData).PortQueueList);
    KeInitializeEvent(
        &mut (*PortData).QueueStoppedEvent,
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );

    KeInitializeDpc(
        &mut (*PortData).Worker.Dpc,
        Some(AtaPortWorkerSignalDpc),
        (PortData).cast(),
    );
    KeInitializeDpc(
        &mut (*PortData).Worker.NotificationDpc,
        Some(AtaStorageNotificationlDpc),
        (PortData).cast(),
    );
    KeInitializeSpinLock(&mut (*PortData).Worker.Lock);
    KeInitializeEvent(
        &mut (*PortData).Worker.EnumerationEvent,
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );

    KeInitializeEvent(
        &mut (*PortData).Worker.CompletionEvent,
        _EVENT_TYPE_NotificationEvent,
        FALSE,
    );
    (*PortData).Worker.InternalRequest.Complete = Some(AtaPortCompleteInternalRequest);
    (*PortData).Worker.InternalRequest.Signature = ATA_DEVICE_REQUEST_SIGNATURE;

    Status = AtaFdoCreatePortThread((PortData).cast());
    if !NT_SUCCESS!(Status) {
        ERR!(b"CH %lu: Failed to create port thread 0x%lx\n\0".as_ptr() as *const i8, (*PortData).PortNumber, Status);
        return Status;
    }

    Status = AtaFdoCreateSymLinks((ChanExt).cast());
    if !NT_SUCCESS!(Status) {
        ERR!(b"CH %lu: Failed to create symbolic links 0x%lx\n\0".as_ptr() as *const i8, (*PortData).PortNumber, Status);
        return Status;
    }

    AtaSetPortRegistryKey((ChanExt).cast(), DD_ATA_REG_MAX_TARGET_ID.as_ptr(), (*PortData).MaxTargetId);

    Status = IoRegisterDeviceInterface(
        (*ChanExt).Pdo,
        &GUID_DEVINTERFACE_STORAGEPORT,
        core::ptr::null_mut(),
        &mut (*ChanExt).StorageInterfaceName,
    );
    if NT_SUCCESS!(Status) {
        INFO!(
            b"InterfaceName: '%wZ'\n\0".as_ptr() as *const i8,
            &(*ChanExt).StorageInterfaceName,
        );

        Status = IoSetDeviceInterfaceState(
            &raw mut (*ChanExt).StorageInterfaceName,
            TRUE,
        );
        if !NT_SUCCESS!(Status) {
            RtlFreeUnicodeString(&mut (*ChanExt).StorageInterfaceName);
            (*ChanExt).StorageInterfaceName.Buffer = core::ptr::null_mut();
        }
    }

    *ChannelInterface.PortContext = (PortData).cast();
    *ChannelInterface.PortNotification = Some(AtaPortNotification);
    *ChannelInterface.Slots = ((*PortData).Slots).as_mut_ptr();

    Status = IoInitializeTimer((*ChanExt).Common.Self_, Some(AtaPortIoTimer), (PortData).cast());
    if !NT_SUCCESS!(Status) {
        return Status;
    }

    IoStartTimer((*ChanExt).Common.Self_);
    (*PortData).PortFlags |= PORT_FLAG_IO_TIMER_ACTIVE;

    Status = (((*PortData).AttachChannel).unwrap_unchecked())((*PortData).ChannelContext, TRUE);
    if !NT_SUCCESS!(Status) {
        ERR!(b"CH %lu: Failed to attach channel %lx\n\0".as_ptr() as *const i8, (*PortData).PortNumber, Status);
        return Status;
    }
    (*PortData).PortFlags |= PORT_FLAG_CHANNEL_ATTACHED;

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaFdoDetachChannel(
    ChanExt: *mut ATAPORT_CHANNEL_EXTENSION,
) {
    let PortData: *mut ATAPORT_PORT_DATA = &mut (*ChanExt).PortData;

    PAGED_CODE!();

    if !(*PortData).AttachChannel.is_none() {
        (((*PortData).AttachChannel).unwrap_unchecked())((*PortData).ChannelContext, FALSE);
        (*PortData).AttachChannel = None;
    }
}

unsafe extern "C" fn AtaFdoStopDevice(
    ChanExt: *mut ATAPORT_CHANNEL_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    PAGED_CODE!();

    AtaFdoDetachChannel(ChanExt);

    if !(*ChanExt).StorageInterfaceName.Buffer.is_null() {
        IoSetDeviceInterfaceState(
            core::ptr::addr_of_mut!((*ChanExt).StorageInterfaceName),
            FALSE,
        );
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn AtaFdoRemoveDevice(
    ChanExt: *mut ATAPORT_CHANNEL_EXTENSION,
    Irp: PIRP,
    FinalRemove: u8,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    PAGED_CODE!();

    if !(*ChanExt).StorageInterfaceName.Buffer.is_null() {
        IoSetDeviceInterfaceState(
            &mut (*ChanExt).StorageInterfaceName,
            FALSE,
        );

        RtlFreeUnicodeString(&mut (*ChanExt).StorageInterfaceName);
        (*ChanExt).StorageInterfaceName.Buffer = core::ptr::null_mut();
    }

    AtaFdoDestroyPortThread(ChanExt);

    if (*ChanExt).PortData.PortFlags & PORT_FLAG_IO_TIMER_ACTIVE != 0 {
        IoStopTimer((*ChanExt).Common.Self_);
        (*ChanExt).PortData.PortFlags &= !PORT_FLAG_IO_TIMER_ACTIVE;
    }

    if (*ChanExt).PortData.PortFlags & PORT_FLAG_CHANNEL_ATTACHED != 0 {
        AtaFdoDetachChannel(ChanExt);
        (*ChanExt).PortData.PortFlags &= !PORT_FLAG_CHANNEL_ATTACHED;
    }

    if FinalRemove != 0 {
        let mut AtaScsiAddress: ATA_SCSI_ADDRESS = core::mem::zeroed();

        IoReleaseRemoveLockAndWait(&mut (*ChanExt).Common.RemoveLock, (Irp).cast());

        AtaScsiAddress.AsULONG = 0;
        loop {
            let mut DevExt: *mut ATAPORT_DEVICE_EXTENSION;

            DevExt = AtaFdoFindNextDeviceByPath(
                ChanExt,
                &mut AtaScsiAddress,
                TRUE,
                core::ptr::null_mut(),
            );
            if DevExt.is_null() {
                break;
            }

            AtaReqFlushDeviceQueue(&mut (*DevExt).Device);
            AtaPdoFreeDevice((DevExt).cast());
        }

        AtaFdoRemoveSymLinks(ChanExt);

        if !(*ChanExt).PortData.LocalBuffer.is_null() {
            MmFreeContiguousMemorySpecifyCache(
                (*ChanExt).PortData.LocalBuffer,
                (ATA_LOCAL_BUFFER_SIZE) as SIZE_T,
                MmNonCached,
            );
            (*ChanExt).PortData.LocalBuffer = core::ptr::null_mut();
        }

        if !(*ChanExt).PortData.ReservedVaSpace.is_null() {
            MmFreeMappingAddress(
                (*ChanExt).PortData.ReservedVaSpace,
                ATAPORT_TAG,
            );
            (*ChanExt).PortData.ReservedVaSpace = core::ptr::null_mut();
        }

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        IoSkipCurrentIrpStackLocation(Irp);
        Status = IoCallDriver((*ChanExt).Common.__bindgen_anon_1.LowerDeviceObject, Irp);

        IoDetachDevice((*ChanExt).Common.__bindgen_anon_1.LowerDeviceObject);
        IoDeleteDevice((*ChanExt).Common.Self_);
    } else {
        IoSkipCurrentIrpStackLocation(Irp);
        Status = IoCallDriver((*ChanExt).Common.__bindgen_anon_1.LowerDeviceObject, Irp);

        IoReleaseRemoveLock(&mut (*ChanExt).Common.RemoveLock, (Irp).cast());
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn AtaFdoPnp(
    ChanExt: PATAPORT_CHANNEL_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let IoStack: PIO_STACK_LOCATION;

    PAGED_CODE!();

    INFO!(
        b"(%p, %p) Ch.%lu %s\n\0".as_ptr() as *const i8,
        (*ChanExt).Common.Self,
        Irp,
        (*ChanExt).DeviceObjectNumber,
        GetIRPMinorFunctionString(IoGetCurrentIrpStackLocation(Irp).MinorFunction),
    );

    Status = IoAcquireRemoveLock(&mut (*ChanExt).Common.RemoveLock, (Irp).cast());
    if !NT_SUCCESS!(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

        return Status;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    match (*IoStack).MinorFunction {
        IRP_MN_START_DEVICE => {
            if ((NT_VERIFY!(IoForwardIrpSynchronously((*ChanExt).Common.__bindgen_anon_1.LowerDeviceObject, Irp))) == 0) {
                Status = STATUS_UNSUCCESSFUL;
                'CompleteIrp: {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                    IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

                    IoReleaseRemoveLock(&mut (*ChanExt).Common.RemoveLock, (Irp).cast());

                    return Status;
                }
            }
            Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
            if !NT_SUCCESS!(Status) {
                'CompleteIrp: {
                    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                    IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

                    IoReleaseRemoveLock(&mut (*ChanExt).Common.RemoveLock, (Irp).cast());

                    return Status;
                }
            }

            Status = AtaFdoStartDevice(
                ChanExt,
                (*IoStack).Parameters.StartDevice.AllocatedResourcesTranslated,
            );
            'CompleteIrp: {
                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

                IoReleaseRemoveLock(&mut (*ChanExt).Common.RemoveLock, (Irp).cast());

                return Status;
            }
        }

        IRP_MN_STOP_DEVICE => {
            Status = AtaFdoStopDevice((ChanExt).cast(), Irp);
        }

        IRP_MN_REMOVE_DEVICE | IRP_MN_SURPRISE_REMOVAL => {
            return AtaFdoRemoveDevice(
                (ChanExt).cast(),
                Irp,
                ((*IoStack).MinorFunction == IRP_MN_REMOVE_DEVICE) as u8,
            );
        }

        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            Status = AtaPnpQueryPnpDeviceState(&mut (*ChanExt).Common, Irp);
        }

        IRP_MN_QUERY_DEVICE_RELATIONS => {
            if (*IoStack).Parameters.QueryDeviceRelations.Type != BusRelations {
                // break out of match
            } else {
                Status = AtaFdoQueryBusRelations((ChanExt).cast(), Irp);
                if !NT_SUCCESS!(Status) {
                    'CompleteIrp: {
                        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
                        IoCompleteRequest(Irp, (IO_NO_INCREMENT) as CCHAR);

                        IoReleaseRemoveLock(&mut (*ChanExt).Common.RemoveLock, (Irp).cast());

                        return Status;
                    }
                }

                (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            }
        }

        IRP_MN_DEVICE_USAGE_NOTIFICATION => {
            Status = AtaPnpQueryDeviceUsageNotification(&mut (*ChanExt).Common, Irp);
        }

        IRP_MN_QUERY_STOP_DEVICE
        | IRP_MN_QUERY_REMOVE_DEVICE
        | IRP_MN_CANCEL_STOP_DEVICE
        | IRP_MN_CANCEL_REMOVE_DEVICE => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        }

        _ => {}
    }

    IoSkipCurrentIrpStackLocation(Irp);
    Status = IoCallDriver((*ChanExt).Common.__bindgen_anon_1.LowerDeviceObject, Irp);

    IoReleaseRemoveLock(&mut (*ChanExt).Common.RemoveLock, (Irp).cast());

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn AtaFdoFindDeviceByPath(
    ChanExt: PATAPORT_CHANNEL_EXTENSION,
    AtaScsiAddress: ATA_SCSI_ADDRESS,
    ReferenceTag: *mut core::ffi::c_void,
) -> PATAPORT_DEVICE_EXTENSION {
    let mut DevExt: PATAPORT_DEVICE_EXTENSION;
    let mut Result: PATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Entry: PSINGLE_LIST_ENTRY;
    let mut OldLevel: KIRQL = 0;
    let mut Status: NTSTATUS;

    KeAcquireSpinLock(&mut (*ChanExt).PdoListLock, &mut OldLevel);

    Entry = (*ChanExt).PdoList.Next;
    while !Entry.is_null() {
        DevExt = CONTAINING_RECORD!(Entry, ATAPORT_DEVICE_EXTENSION, ListEntry);

        if (*DevExt).Device.AtaScsiAddress.AsULONG != AtaScsiAddress.AsULONG {
            Entry = (*Entry).Next;
            continue;
        }

        if (*DevExt).ReportedMissing != 0 || (*DevExt).RemovalPending != 0 {
            Entry = (*Entry).Next;
            continue;
        }

        if !ReferenceTag.is_null() {
            Status = IoAcquireRemoveLock(&mut (*DevExt).Common.RemoveLock, ReferenceTag);
            if NT_SUCCESS!(Status) == false {
                break;
            }
        }

        Result = DevExt;
        break;
    }

    KeReleaseSpinLock(&mut (*ChanExt).PdoListLock, OldLevel);

    Result
}
#[no_mangle]
pub unsafe extern "C" fn AtaFdoFindNextDeviceByPath(
    ChanExt: *mut ATAPORT_CHANNEL_EXTENSION,
    AtaScsiAddress: *mut ATA_SCSI_ADDRESS,
    SearchRemoveDev: u8,
    ReferenceTag: *mut core::ffi::c_void,
) -> *mut ATAPORT_DEVICE_EXTENSION {
    let mut DevExt: *mut ATAPORT_DEVICE_EXTENSION;
    let mut Result: *mut ATAPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Entry: PSINGLE_LIST_ENTRY;
    let mut OldLevel: KIRQL = core::mem::zeroed();
    let mut Status: i32;

    KeAcquireSpinLock((&mut (*ChanExt).PdoListLock as *mut _ as *mut usize).cast(), &mut OldLevel);

    Entry = (*ChanExt).PdoList.Next;
    while !Entry.is_null() {
        DevExt = CONTAINING_RECORD!(Entry, ATAPORT_DEVICE_EXTENSION, ListEntry);

        if (*DevExt).Device.AtaScsiAddress.AsULONG <= (*AtaScsiAddress).AsULONG {
            Entry = (*Entry).Next;
            continue;
        }

        if (*DevExt).ReportedMissing != 0 {
            Entry = (*Entry).Next;
            continue;
        }

        if SearchRemoveDev == 0 && (*DevExt).RemovalPending != 0 {
            Entry = (*Entry).Next;
            continue;
        }

        *AtaScsiAddress = (*DevExt).Device.AtaScsiAddress;

        if !ReferenceTag.is_null() {
            Status = IoAcquireRemoveLock(&mut (*DevExt).Common.RemoveLock, ReferenceTag);
            if NT_SUCCESS!(Status) == false {
                Entry = (*Entry).Next;
                continue;
            }
        }
        Result = DevExt;
        break;
    }

    KeReleaseSpinLock((&mut (*ChanExt).PdoListLock as *mut _ as *mut usize).cast(), OldLevel);

    Result
}
#[no_mangle]
pub unsafe extern "C" fn AtaFdoDeviceListInsert(
    ChanExt: *mut ATAPORT_CHANNEL_EXTENSION,
    DevExt: *mut ATAPORT_DEVICE_EXTENSION,
    DoInsert: u8,
) {
    let mut Entry: PSINGLE_LIST_ENTRY;
    let mut PrevEntry: PSINGLE_LIST_ENTRY;
    let mut OldLevel: KIRQL = core::mem::zeroed();
    let Address: u32 = (*DevExt).Device.AtaScsiAddress.AsULONG;

    PAGED_CODE!();

    KeAcquireSpinLock((&mut (*ChanExt).PdoListLock as *mut _ as *mut usize).cast(), &mut OldLevel);

    Entry = (*ChanExt).PdoList.Next;
    PrevEntry = core::ptr::null_mut();
    while !Entry.is_null() {
        let CurrentDevExt: *mut ATAPORT_DEVICE_EXTENSION;

        CurrentDevExt = CONTAINING_RECORD!(Entry, ATAPORT_DEVICE_EXTENSION, ListEntry);

        if DoInsert != 0 {
            if (*CurrentDevExt).Device.AtaScsiAddress.AsULONG > Address {
                break;
            }
        } else {
            if (*CurrentDevExt).Device.AtaScsiAddress.AsULONG == Address {
                break;
            }
        }

        PrevEntry = Entry;
        Entry = (*Entry).Next;
    }

    if !PrevEntry.is_null() {
        if DoInsert != 0 {
            (*DevExt).ListEntry.Next = (*PrevEntry).Next;
            (*PrevEntry).Next = &mut (*DevExt).ListEntry;
        } else {
            (*PrevEntry).Next = (*DevExt).ListEntry.Next;
        }
    } else {
        if DoInsert != 0 {
            PushEntryList(&mut (*ChanExt).PdoList, &mut (*DevExt).ListEntry);
        } else {
            PopEntryList(&mut (*DevExt).ListEntry);
        }
    }

    KeReleaseSpinLock((&mut (*ChanExt).PdoListLock as *mut _ as *mut usize).cast(), OldLevel);
}
