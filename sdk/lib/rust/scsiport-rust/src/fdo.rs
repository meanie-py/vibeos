#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
use crate::glue::{IRP_MN_QUERY_ID}; // explicit: disambiguate vs ntkrnl_sys::driver_ddk glob
#[allow(unused_imports)] use crate::inlines::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{DEVICE_RELATIONS, ExFreePool, _KWAIT_REASON_Executive, HalFreeCommonBuffer, HalGetInterruptVector, _IO_STATUS_BLOCK, IoBuildDeviceIoControlRequest, IoConnectInterrupt, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, IoDisconnectInterrupt, IoForwardIrpSynchronously, IoGetConfigurationInformation, IoRegisterDeviceInterface, IoSetDeviceInterfaceState, IoStartTimer, IoStopTimer, KAFFINITY, KEVENT, KeInitializeEvent, KeSynchronizeExecution, KeWaitForSingleObject, MmUnmapIoSpace, _EVENT_TYPE_NotificationEvent, PCONFIGURATION_INFORMATION, PDEVICE_OBJECT, PDEVICE_RELATIONS, PIO_STACK_LOCATION, PIRP, PZZWSTR, RtlFreeUnicodeString, RtlInitUnicodeString, UINT32, UINT8, ZwClose, ZwDeleteKey, DEVICE_OBJECT, KINTERRUPT_MODE, MAPPED_ADDRESS, PKSYNCHRONIZE_ROUTINE, _DEVICE_OBJECT, _IRP, _KDPC, _KINTERRUPT, _UNICODE_STRING};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::scsi::ScsiPortDpcForIsr;
use crate::scsiport::ScsiPortIsr;
use crate::types::{ACCESS_RANGE, CDB, DEVICE_QUALIFIER_NOT_SUPPORTED, SCSI_PORT_SCAN_IN_PROGRESS, INQUIRYDATA, INQUIRYDATABUFFERSIZE, IOCTL_SCSI_EXECUTE_IN, LUNEX_FROZEN_QUEUE, PINQUIRYDATA, PSCSI_PORT_DEVICE_EXTENSION, PSENSE_DATA, SCSIOP_INQUIRY, SCSI_BUS_INFO, SCSI_PORT_DEVICE_EXTENSION, SCSI_PORT_DISCONNECT_ALLOWED, SCSI_PORT_LUN_EXTENSION, SCSI_PORT_NOTIFICATION_NEEDED, SCSI_REQUEST_BLOCK, SCSI_SENSE_ILLEGAL_REQUEST, _PORT_CONFIGURATION_INFORMATION as PORT_CONFIGURATION_INFORMATION, SRB_FLAGS_DATA_IN, SRB_FLAGS_DISABLE_SYNCH_TRANSFER, SRB_FUNCTION_EXECUTE_SCSI, SRB_STATUS_AUTOSENSE_VALID, SRB_STATUS_BAD_FUNCTION, SRB_STATUS_BAD_SRB_BLOCK_LENGTH, SRB_STATUS_DATA_OVERRUN, SRB_STATUS_NO_DEVICE, SRB_STATUS_QUEUE_FROZEN, SRB_STATUS_SELECTION_TIMEOUT, SRB_STATUS_SUCCESS};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{LPVOID, STATUS_ADAPTER_HARDWARE_ERROR, STATUS_IO_DEVICE_ERROR};

extern "C" {
    fn wcslen(s: *const u16) -> usize;
}
extern "C" {
    fn SpiGetNextRequestFromLun( DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION, LunExtension: *mut SCSI_PORT_LUN_EXTENSION, Irql: *mut KIRQL, );
    fn DbgPrint(format: *const u8, ...) -> i32;
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
    pub fn RegistryInitAdapterKey(DeviceExtension: PSCSI_PORT_DEVICE_EXTENSION) -> NTSTATUS;
    fn GetLunByPath( PortExtension: *mut SCSI_PORT_DEVICE_EXTENSION, PathId: u8, TargetId: u8, Lun: u8, ) -> *mut SCSI_PORT_LUN_EXTENSION;
    fn PdoCreateLunDevice( PortExtension: *mut SCSI_PORT_DEVICE_EXTENSION, ) -> *mut DEVICE_OBJECT;
}

// ---- file-scope preamble ----
// Local LLM consts deleted: SCSI_PORT_SCAN_IN_PROGRESS was fabricated as 0x1 (real
// 0x8000) — equal to LUNEX_FROZEN_QUEUE, so every FOUND device's LUN queue was left
// permanently frozen after the scan and all later class I/O parked forever (the boot
// wedge). DEVICE_QUALIFIER_NOT_SUPPORTED was fabricated as 0x1f (real 0x03). The
// bindgen types are the verified source.
pub const NDEBUG: bool = true;



unsafe fn FdoSendInquiry(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    IoStatusBlock.__bindgen_anon_1.Status = 0;
    IoStatusBlock.Information = 0;
    let mut IrpStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Event: KEVENT = core::mem::zeroed();
    let mut Irql: KIRQL = 0;
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut InquiryBuffer: PINQUIRYDATA = core::ptr::null_mut();
    let mut SenseBuffer: PSENSE_DATA = core::ptr::null_mut();
    let mut KeepTrying: BOOLEAN = 1;
    let mut RetryCount: ULONG = 0;
    let mut Srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    let mut Cdb: *mut CDB = core::ptr::null_mut();

    // DPRINT removed

    let LunExtension = (*DeviceObject).DeviceExtension as *mut SCSI_PORT_LUN_EXTENSION;
    let DeviceExtension = (*(*LunExtension).Common.LowerDevice).DeviceExtension as *mut SCSI_PORT_DEVICE_EXTENSION;

    InquiryBuffer = ExAllocatePoolWithTag(NonPagedPool, INQUIRYDATABUFFERSIZE as SIZE_T, TAG_SCSIPORT) as PINQUIRYDATA;
    if InquiryBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    SenseBuffer = ExAllocatePoolWithTag(NonPagedPool, SENSE_BUFFER_SIZE as SIZE_T, TAG_SCSIPORT) as PSENSE_DATA;
    if SenseBuffer.is_null() {
        ExFreePoolWithTag(InquiryBuffer as LPVOID, TAG_SCSIPORT);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    while KeepTrying != 0 {
        KeInitializeEvent(&raw mut Event, _EVENT_TYPE_NotificationEvent, 0);

        Irp = IoBuildDeviceIoControlRequest(
            IOCTL_SCSI_EXECUTE_IN,
            DeviceObject,
            core::ptr::null_mut(),
            0,
            InquiryBuffer as LPVOID,
            INQUIRYDATABUFFERSIZE,
            1,
            &raw mut Event,
            &raw mut IoStatusBlock,
        );
        if Irp.is_null() {
            // DPRINT removed
            Status = STATUS_INSUFFICIENT_RESOURCES;
            KeepTrying = 0;
            continue;
        }

        // Prepare SRB
        core::ptr::write_bytes(
            &raw mut Srb as *mut SCSI_REQUEST_BLOCK as *mut u8,
            0,
            core::mem::size_of::<SCSI_REQUEST_BLOCK>(),
        );

        Srb.Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
        Srb.OriginalRequest = Irp as LPVOID;
        Srb.PathId = (*LunExtension).PathId;
        Srb.TargetId = (*LunExtension).TargetId;
        Srb.Lun = (*LunExtension).Lun;
        Srb.Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
        Srb.SrbFlags = (SRB_FLAGS_DATA_IN | SRB_FLAGS_DISABLE_SYNCH_TRANSFER);
        Srb.TimeOutValue = 4;
        Srb.CdbLength = 6;

        Srb.SenseInfoBuffer = SenseBuffer as LPVOID;
        Srb.SenseInfoBufferLength = (SENSE_BUFFER_SIZE) as u8;

        Srb.DataBuffer = InquiryBuffer as LPVOID;
        Srb.DataTransferLength = INQUIRYDATABUFFERSIZE;

        IrpStack = IoGetNextIrpStackLocation(Irp);
        (*IrpStack).Parameters.Scsi.Srb = &raw mut Srb as *mut _ as *mut ntkrnl_sys::ddk::_SCSI_REQUEST_BLOCK;

        Cdb = &raw mut Srb.Cdb as *mut UCHAR as *mut CDB;
        (*Cdb).CDB6INQUIRY.as_mut().OperationCode = (SCSIOP_INQUIRY) as u8;
        (*Cdb).CDB6INQUIRY.as_mut().set_LogicalUnitNumber((*LunExtension).Lun as UCHAR);
        (*Cdb).CDB6INQUIRY.as_mut().AllocationLength = INQUIRYDATABUFFERSIZE as UCHAR;

        Status = IoCallDriver(DeviceObject, Irp);

        if Status == STATUS_PENDING {
            // DPRINT removed
            KeWaitForSingleObject(
                &raw mut Event as LPVOID,
                _KWAIT_REASON_Executive,
                (0i32) as i8,
                0,
                core::ptr::null_mut(),
            );
            Status = IoStatusBlock.__bindgen_anon_1.Status;
        }

        if (Srb.SrbStatus as i32 & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32) == (SRB_STATUS_SUCCESS) as i32 {
            core::ptr::copy_nonoverlapping(
                InquiryBuffer as *const u8,
                &raw mut (*LunExtension).InquiryData as *mut u8,
                INQUIRYDATABUFFERSIZE as usize,
            );
            Status = STATUS_SUCCESS;
            KeepTrying = 0;
            continue;
        }

        // DPRINT removed

        if Srb.SrbStatus & (SRB_STATUS_QUEUE_FROZEN) as u8 != 0 {
            KeepTrying = 0;

            // DPRINT removed

            (*LunExtension).Flags &= !LUNEX_FROZEN_QUEUE;

            KeAcquireSpinLock((&raw mut (*DeviceExtension).SpinLock).cast(), &raw mut Irql);
            SpiGetNextRequestFromLun(DeviceExtension, LunExtension, &raw mut Irql);
        }

        if (Srb.SrbStatus as i32 & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32) == (SRB_STATUS_DATA_OVERRUN) as i32 {
            // DPRINT removed

            let copy_size = if Srb.DataTransferLength > INQUIRYDATABUFFERSIZE {
                INQUIRYDATABUFFERSIZE
            } else {
                Srb.DataTransferLength
            };
            core::ptr::copy_nonoverlapping(
                InquiryBuffer as *const u8,
                &raw mut (*LunExtension).InquiryData as *mut u8,
                copy_size as usize,
            );
            Status = STATUS_SUCCESS;
            KeepTrying = 0;
        } else if Srb.SrbStatus & (SRB_STATUS_AUTOSENSE_VALID) as u8 != 0
            && (*SenseBuffer).SenseKey() == (SCSI_SENSE_ILLEGAL_REQUEST) as u8
        {
            Status = STATUS_INVALID_DEVICE_REQUEST;
            KeepTrying = 0;
        } else if RetryCount < 2
            && (Srb.SrbStatus as i32 & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32) != (SRB_STATUS_NO_DEVICE) as i32
            && (Srb.SrbStatus as i32 & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32) != (SRB_STATUS_SELECTION_TIMEOUT) as i32
        {
            RetryCount = RetryCount.wrapping_add(1);
            KeepTrying = 1;
        } else {
            KeepTrying = 0;
            if (Srb.SrbStatus as i32 & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32) == (SRB_STATUS_BAD_FUNCTION) as i32
                || (Srb.SrbStatus as i32 & (!(SRB_STATUS_AUTOSENSE_VALID | SRB_STATUS_QUEUE_FROZEN)) as i32) == (SRB_STATUS_BAD_SRB_BLOCK_LENGTH) as i32
            {
                Status = STATUS_INVALID_DEVICE_REQUEST;
            } else {
                Status = STATUS_IO_DEVICE_ERROR;
            }
        }
    }

    ExFreePoolWithTag(InquiryBuffer as LPVOID, TAG_SCSIPORT);
    ExFreePoolWithTag(SenseBuffer as LPVOID, TAG_SCSIPORT);

    // DPRINT removed

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn FdoScanAdapter(
    PortExtension: *mut SCSI_PORT_DEVICE_EXTENSION,
) {
    let mut status: i32; // NTSTATUS
    let mut totalLUNs: u32 = (*PortExtension).TotalLUCount as u32;

    // Scan all buses
    for pathId in 0..(*PortExtension).NumberOfBuses {
        // DPRINT equivalent (omitted as in original? original had DPRINT, anchor uses no print for this)
        // Not printed, we skip to be faithful to the C that had DPRINT but anchor omitted.
        // However the anchor has no print there. The original C had DPRINT("    Scanning bus/pathID %u\n", pathId);
        // But the anchor does not include that call. We must follow anchor. So we omit.

        let currentBus: *mut SCSI_BUS_INFO =
            (*PortExtension).Buses.add(pathId as usize);

        // Scan all targets
        for targetId in 0..(*(*PortExtension).PortConfig).MaximumNumberOfTargets {
            let mut targetFound: i32 = 0; // BOOLEAN, 0=FALSE

            // Skip if it's the same address
            if targetId == (*currentBus).BusIdentifier {
                continue;
            }

            // Scan all logical units
            for lun in 0..(*PortExtension).MaxLunCount {
                let mut lunExt: *mut SCSI_PORT_LUN_EXTENSION = core::ptr::null_mut();

                // Skip invalid lun values
                if lun >= ((*(*PortExtension).PortConfig).MaximumNumberOfLogicalUnits) as u32 {
                    continue;
                }

                // Try to find an existing device
                lunExt = GetLunByPath(PortExtension, pathId, targetId, (lun) as u8);

                if !lunExt.is_null() {
                    // Check if the device still exists
                    status = FdoSendInquiry((*lunExt).Common.DeviceObject);
                    if status < 0 {
                        // UNIMPLEMENTED; __debugbreak
                        DbgPrint(
                            (b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8).cast(),
                            b"FdoScanAdapter\0".as_ptr() as *const i8,
                            b"drivers/storage/port/scsiport/fdo.c\0".as_ptr() as *const i8,
                            269i32,
                        );
                        __debugbreak();
                    }

                    if (*lunExt).InquiryData.DeviceTypeQualifier() as i32
                        == DEVICE_QUALIFIER_NOT_SUPPORTED as i32
                    {
                        // UNIMPLEMENTED; __debugbreak
                        DbgPrint(
                            (b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8).cast(),
                            b"FdoScanAdapter\0".as_ptr() as *const i8,
                            b"drivers/storage/port/scsiport/fdo.c\0".as_ptr() as *const i8,
                            276i32,
                        );
                        __debugbreak();
                    }

                    // Decide whether we are continuing or not
                    if status == STATUS_INVALID_DEVICE_REQUEST {
                        continue;
                    } else {
                        break;
                    }
                } else {
                    // Create a new LUN device
                    let lunPDO: *mut DEVICE_OBJECT = PdoCreateLunDevice(PortExtension);
                    if lunPDO.is_null() {
                        continue;
                    }

                    lunExt = (*lunPDO).DeviceExtension as *mut SCSI_PORT_LUN_EXTENSION;

                    (*lunExt).PathId = pathId;
                    (*lunExt).TargetId = targetId;
                    (*lunExt).Lun = (lun) as u8;

                    // DPRINT omitted (anchor has no print)

                    // Set flag to prevent race conditions
                    (*lunExt).Flags |= SCSI_PORT_SCAN_IN_PROGRESS as u32;

                    // Finally send the inquiry command
                    status = FdoSendInquiry(lunPDO);

                    if status >= 0 {
                        let InquiryData: *mut INQUIRYDATA = &raw mut (*lunExt).InquiryData;

                        // Check if this device is unsupported
                        if (*InquiryData).DeviceTypeQualifier() as i32
                            == DEVICE_QUALIFIER_NOT_SUPPORTED as i32
                        {
                            IoDeleteDevice(lunPDO);
                            continue;
                        }

                        // Clear the "in scan" flag
                        (*lunExt).Flags &= !SCSI_PORT_SCAN_IN_PROGRESS as u32;

                        // DPRINT1 equivalent (with DbgPrint return check)
                        if DbgPrint(
                            (b"(%s:%d) Found device of type %d at controller %d bus %d tid %d lun %d, PDO: %p\n\0"
                                .as_ptr() as *const i8).cast(),
                            b"drivers/storage/port/scsiport/fdo.c\0".as_ptr() as *const i8,
                            324i32,
                            (*InquiryData).DeviceType() as i32,
                            (*PortExtension).PortNumber,
                            pathId as i32,
                            targetId as i32,
                            lun as i32,
                            lunPDO,
                        ) != 0
                        {
                            DbgPrint(
                                (b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8).cast(),
                                b"drivers/storage/port/scsiport/fdo.c\0".as_ptr() as *const i8,
                                324i32,
                            );
                        }

                        InsertTailList(
                            &raw mut (*currentBus).LunsListHead,
                            &raw mut (*lunExt).LunEntry,
                        );

                        totalLUNs = totalLUNs.wrapping_add(1);
                        (*currentBus).LogicalUnitsCount = (*currentBus).LogicalUnitsCount.wrapping_add(1);
                        targetFound = 1; // TRUE
                    } else if status != STATUS_INVALID_DEVICE_REQUEST {
                        break;
                    }
                }
            } // end lun loop

            if targetFound != 0 {
                (*currentBus).TargetsCount = (*currentBus).TargetsCount.wrapping_add(1);
            }
        } // end targetId loop
    } // end pathId loop

    (*PortExtension).TotalLUCount = totalLUNs as u8;
}
#[no_mangle]
pub unsafe extern "C" fn FdoCallHWInitialize(
    DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let PortConfig: *mut PORT_CONFIGURATION_INFORMATION = (*DeviceExtension).PortConfig;
    let mut Status: NTSTATUS = 0;
    let mut OldIrql: KIRQL = 0;

    if (*DeviceExtension).HwInterrupt.is_none()
        || ((*PortConfig).BusInterruptLevel == 0u32
            && (*PortConfig).BusInterruptVector == 0u32)
    {
        (*DeviceExtension).InterruptCount = 0u32;
        if DbgPrint(
            b"(%s:%d) Interrupt Count: 0\n\0" as *const u8 as *const u8,
            b"drivers/storage/port/scsiport/fdo.c\0"
                as *const u8 as *const i8,
            376i32,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const u8,
                b"drivers/storage/port/scsiport/fdo.c\0"
                    as *const u8 as *const i8,
                376i32,
            );
        }
        DbgPrint(
            b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const u8,
            b"FdoCallHWInitialize\0" as *const u8 as *const i8,
            b"drivers/storage/port/scsiport/fdo.c\0"
                as *const u8 as *const i8,
            378i32,
        );
        __debugbreak();
    } else {
        let mut InterruptShareable: BOOLEAN = 0;
        let mut InterruptMode: [KINTERRUPT_MODE; 2] = [LevelSensitive; 2];
        let mut InterruptVector: [ULONG; 2] = [0; 2];
        let mut i: ULONG = 0;
        let mut MappedIrq: [ULONG; 2] = [0; 2];
        let mut Dirql: [KIRQL; 2] = [0; 2];
        let mut MaxDirql: KIRQL = 0;
        let mut Affinity: [KAFFINITY; 2] = [0; 2];

        (*DeviceExtension).InterruptLevel[0usize] = (*PortConfig).BusInterruptLevel;
        (*DeviceExtension).InterruptLevel[1usize] = (*PortConfig).BusInterruptLevel2;
        InterruptVector[0usize] = (*PortConfig).BusInterruptVector;
        InterruptVector[1usize] = (*PortConfig).BusInterruptVector2;
        InterruptMode[0usize] = (*PortConfig).InterruptMode;
        InterruptMode[1usize] = (*PortConfig).InterruptMode2;

        (*DeviceExtension).InterruptCount = if (*PortConfig).BusInterruptLevel2 != 0u32
            || (*PortConfig).BusInterruptVector2 != 0u32
        {
            2u32
        } else {
            1u32
        };

        i = 0u32;
        while i < (*DeviceExtension).InterruptCount {
            MappedIrq[i as usize] = HalGetInterruptVector(
                (*PortConfig).AdapterInterfaceType,
                (*PortConfig).SystemIoBusNumber,
                (*DeviceExtension).InterruptLevel[i as usize],
                InterruptVector[i as usize],
                (core::ptr::addr_of_mut!(Dirql) as *mut KIRQL).offset(i as isize) as *mut KIRQL,
                (core::ptr::addr_of_mut!(Affinity) as *mut KAFFINITY).offset(i as isize)
                    as *mut KAFFINITY,
            );
            i = i.wrapping_add(1);
        }

        if (*DeviceExtension).InterruptCount == 1u32
            || Dirql[0usize] as i32 > Dirql[1usize] as i32
        {
            MaxDirql = Dirql[0usize];
        } else {
            MaxDirql = Dirql[1usize];
        }

        i = 0u32;
        while i < (*DeviceExtension).InterruptCount {
            if (*PortConfig).AdapterInterfaceType as i32 == MicroChannel as i32
                || InterruptMode[i as usize] as u32 == LevelSensitive as i32 as u32
            {
                InterruptShareable = 1u8;
            } else {
                InterruptShareable = 0u8;
            }

            Status = IoConnectInterrupt(
                ((&raw mut (*DeviceExtension).Interrupt) as *mut *mut _KINTERRUPT)
                    .offset(i as isize) as *mut *mut _KINTERRUPT,
                Some(core::mem::transmute(ScsiPortIsr as *const ())),
                DeviceExtension as PVOID,
                &raw mut (*DeviceExtension).IrqLock,
                MappedIrq[i as usize],
                Dirql[i as usize],
                MaxDirql,
                InterruptMode[i as usize],
                InterruptShareable,
                Affinity[i as usize],
                (0i32) as u8,
            );
            DbgPrint(b"[SCSIPORT] connect intr: level=%u vec=%u mapped=%u dirql=%u mode=%u share=%u st=%08x\n\0"
                .as_ptr(),
                (*DeviceExtension).InterruptLevel[i as usize],
                InterruptVector[i as usize], MappedIrq[i as usize],
                Dirql[i as usize] as u32, InterruptMode[i as usize] as u32,
                InterruptShareable as u32, Status as u32); // ahci marker

            if !(Status >= 0) {
                if DbgPrint(
                    b"(%s:%d) Could not connect interrupt %d\n\0" as *const u8 as *const u8,
                    b"drivers/storage/port/scsiport/fdo.c\0"
                        as *const u8 as *const i8,
                    448i32,
                    InterruptVector[i as usize],
                ) != 0
                {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const u8,
                        b"drivers/storage/port/scsiport/fdo.c\0"
                            as *const u8 as *const i8,
                        448i32,
                    );
                }
                (*DeviceExtension).Interrupt[i as usize] = core::ptr::null_mut::<_KINTERRUPT>();
                return Status;
            }
            i = i.wrapping_add(1);
        }
    }

    if (*PortConfig).NumberOfAccessRanges != 0u32 {
        (*DeviceExtension).IoAddress = (*(&raw mut *(*PortConfig).AccessRanges
            as *mut ACCESS_RANGE)
            .offset(0isize))
        .RangeStart
        .__bindgen_anon_1
        .LowPart;
    }

    (*DeviceExtension).Flags |= SCSI_PORT_DISCONNECT_ALLOWED as u32;
    (*DeviceExtension).ActiveRequestCounter = -(1i32) as i32;

    if !(*DeviceExtension).AdapterObject.is_null()
        && (*PortConfig).Master as i32 != 0
        && (*PortConfig).NeedPhysicalAddresses as i32 != 0
    {
        (*DeviceExtension).MapRegisters = 1u8;
    } else {
        (*DeviceExtension).MapRegisters = 0u8;
    }

    KeRaiseIrql(DISPATCH_LEVEL, &mut OldIrql as *mut KIRQL);

    if KeSynchronizeExecution(
        (*DeviceExtension).Interrupt[0usize],
        (*DeviceExtension).HwInitialize as PKSYNCHRONIZE_ROUTINE,
        &raw mut (*DeviceExtension).MiniPortDeviceExtension as *mut u8 as PVOID,
    ) == 0
    {
        if DbgPrint(
            b"(%s:%d) HwInitialize() failed!\n\0" as *const u8 as *const u8,
            b"drivers/storage/port/scsiport/fdo.c\0"
                as *const u8 as *const i8,
            487i32,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const u8,
                b"drivers/storage/port/scsiport/fdo.c\0"
                    as *const u8 as *const i8,
                487i32,
            );
        }
        KeLowerIrql(OldIrql);
        return STATUS_ADAPTER_HARDWARE_ERROR;
    }

    if (*DeviceExtension).InterruptData.Flags as u32 & SCSI_PORT_NOTIFICATION_NEEDED as u32 != 0 {
        ScsiPortDpcForIsr(
            core::ptr::null_mut(),
            (*DeviceExtension).Common.DeviceObject as *mut _DEVICE_OBJECT,
            core::ptr::null_mut::<_IRP>(),
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }

    KeLowerIrql(OldIrql);
    DbgPrint(b"[SCSIPORT] FdoCallHWInitialize done\n\0".as_ptr()); // ahci marker
    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn FdoRemoveAdapter(
    DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    IoStopTimer((*DeviceExtension).Common.DeviceObject);

    if !(*DeviceExtension).InterfaceName.Buffer.is_null() {
        IoSetDeviceInterfaceState(&raw mut (*DeviceExtension).InterfaceName, 0u8);
        RtlFreeUnicodeString(&raw mut (*DeviceExtension).InterfaceName);
        RtlInitUnicodeString(
            &raw mut (*DeviceExtension).InterfaceName,
            core::ptr::null::<u16>(),
        );
    }

    let mut dosNameBuffer: [u16; 12] = [0u16; 12];
    let mut dosDeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<u16>(),
    };
    const DOS_FMT: [u16; 13] = [
        0x005C, 0x003F, 0x003F, 0x005C, 0x0053, 0x0063, 0x0073, 0x0069,
        0x0025, 0x006C, 0x0075, 0x003A, 0x0000,
    ];
    swprintf(
        dosNameBuffer.as_mut_ptr(),
        DOS_FMT.as_ptr(),
        (*DeviceExtension).PortNumber,
    );
    RtlInitUnicodeString(
        &raw mut dosDeviceName,
        dosNameBuffer.as_mut_ptr() as *const u16,
    );
    IoDeleteSymbolicLink(&raw mut dosDeviceName);

    if (*DeviceExtension).DeviceStarted != 0 {
        let sysConfig: PCONFIGURATION_INFORMATION = IoGetConfigurationInformation();
        (*sysConfig).ScsiPortCount = (*sysConfig).ScsiPortCount.wrapping_sub(1);
    }

    while (*DeviceExtension).InterruptCount != 0 {
        (*DeviceExtension).InterruptCount = (*DeviceExtension).InterruptCount.wrapping_sub(1);
        if !(*DeviceExtension).Interrupt[(*DeviceExtension).InterruptCount as usize].is_null() {
            IoDisconnectInterrupt(
                (*DeviceExtension).Interrupt[(*DeviceExtension).InterruptCount as usize],
            );
        }
    }

    if !(*DeviceExtension).Buses.is_null() {
        for pathId in 0u8..(*DeviceExtension).NumberOfBuses {
            let bus: *mut SCSI_BUS_INFO = (*DeviceExtension)
                .Buses
                .add(pathId as usize) as *mut SCSI_BUS_INFO;
            if !(*bus).RegistryMapKey.is_null() {
                ZwDeleteKey((*bus).RegistryMapKey);
                ZwClose((*bus).RegistryMapKey);
                (*bus).RegistryMapKey = core::ptr::null_mut();
            }
        }
        ExFreePoolWithTag((*DeviceExtension).Buses as *mut core::ffi::c_void, TAG_SCSIPORT);
    }

    if !(*DeviceExtension).PortConfig.is_null() {
        ExFreePoolWithTag((*DeviceExtension).PortConfig as *mut core::ffi::c_void, TAG_SCSIPORT);
    }

    if !(*DeviceExtension).SrbExtensionBuffer.is_null()
        && (*DeviceExtension).CommonBufferLength != 0
    {
        if (*DeviceExtension).AdapterObject.is_null() {
            ExFreePoolWithTag((*DeviceExtension).SrbExtensionBuffer, TAG_SCSIPORT);
        } else {
            HalFreeCommonBuffer(
                ((*DeviceExtension).AdapterObject as *mut core::ffi::c_void).cast(),
                (*DeviceExtension).CommonBufferLength,
                (*DeviceExtension).PhysicalAddress,
                (*DeviceExtension).SrbExtensionBuffer,
                0u8,
            );
        }
    }

    if !(*DeviceExtension).SrbInfo.is_null() {
        ExFreePoolWithTag((*DeviceExtension).SrbInfo as *mut core::ffi::c_void, TAG_SCSIPORT);
    }

    while !(*DeviceExtension).MappedAddressList.is_null() {
        MmUnmapIoSpace(
            (*(*DeviceExtension).MappedAddressList).MappedAddress,
            ((*(*DeviceExtension).MappedAddressList).NumberOfBytes as usize) as _,
        );
        let ptr = (*DeviceExtension).MappedAddressList as *mut core::ffi::c_void;
        (*DeviceExtension).MappedAddressList =
            (*(*DeviceExtension).MappedAddressList).NextMappedAddress as *mut MAPPED_ADDRESS;
        ExFreePoolWithTag(ptr, TAG_SCSIPORT);
    }

    IoDeleteDevice((*DeviceExtension).Common.DeviceObject);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn FdoStartAdapter(
    PortExtension: PSCSI_PORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut dosNameBuffer: [u16; 12] = [0; 12];
    let mut dosDeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut status: NTSTATUS = 0;

    IoStartTimer((*PortExtension).Common.DeviceObject);

    const WIDE_FMT: [u16; 12] = [
        0x005Cu16, 0x003F, 0x003F, 0x005C,
        0x0053, 0x0063, 0x0073, 0x0069,
        0x0025, 0x0075, 0x003A, 0x0000,
    ];
    swprintf(
        dosNameBuffer.as_mut_ptr(),
        WIDE_FMT.as_ptr(),
        (*PortExtension).PortNumber,
    );

    RtlInitUnicodeString(
        &raw mut dosDeviceName,
        dosNameBuffer.as_ptr() as PCWSTR,
    );
    status = IoCreateSymbolicLink(
        &raw mut dosDeviceName,
        &raw mut (*PortExtension).DeviceName,
    );
    if !(status >= 0) {
        return status;
    }

    RegistryInitAdapterKey(PortExtension);

    let mut sysConfig: PCONFIGURATION_INFORMATION = IoGetConfigurationInformation();
    (*sysConfig).ScsiPortCount = (*sysConfig).ScsiPortCount.wrapping_add(1);

    status = IoRegisterDeviceInterface(
        (*PortExtension).Common.DeviceObject,
        &raw const GUID_DEVINTERFACE_STORAGEPORT,
        core::ptr::null_mut::<_UNICODE_STRING>(),
        &raw mut (*PortExtension).InterfaceName,
    );

    if status >= 0 {
        IoSetDeviceInterfaceState(&raw mut (*PortExtension).InterfaceName, 1);
    }

    (*PortExtension).DeviceStarted = 1;

    return STATUS_SUCCESS;
}

unsafe fn FdoHandleDeviceRelations(
    PortExtension: PSCSI_PORT_DEVICE_EXTENSION,
    Irp: PIRP,
) -> NTSTATUS {
    let ioStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    if (*ioStack).Parameters.QueryDeviceRelations.Type as u32 == BusRelations as u32 {
        FdoScanAdapter(PortExtension);
        if !((*Irp).IoStatus.Information == (0 as ULONG_PTR) as _) {
            RtlAssert(
                b"Irp->IoStatus.Information == 0\0".as_ptr() as *const core::ffi::c_char as *const core::ffi::c_void,
                b"drivers/storage/port/scsiport/fdo.c\0".as_ptr() as *const core::ffi::c_char as *const core::ffi::c_void,
                664,
                core::ptr::null_mut::<i8>(),
            );
        }
        let deviceRelations: *mut DEVICE_RELATIONS = ExAllocatePoolWithTag(
            PagedPool,
            (core::mem::size_of::<DEVICE_RELATIONS>() as usize).wrapping_add(
                (core::mem::size_of::<PDEVICE_OBJECT>() as usize).wrapping_mul(
                    ((*PortExtension).TotalLUCount as i32 - 1) as usize,
                ),
            ),
            TAG_SCSIPORT,
        ) as *mut DEVICE_RELATIONS;
        if deviceRelations.is_null() {
            (*Irp).IoStatus.Information = (0 as ULONG_PTR) as _;
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
            IoCompleteRequest(Irp, IO_NO_INCREMENT);
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        (*deviceRelations).Count = 0;
        let mut pathId: u8 = 0;
        while (pathId as i32) < (*PortExtension).NumberOfBuses as i32 {
            let bus: *mut SCSI_BUS_INFO = (*PortExtension).Buses.offset(pathId as isize) as *mut SCSI_BUS_INFO;
            let mut lunEntry: *mut LIST_ENTRY = (*bus).LunsListHead.Flink;
            while lunEntry != core::ptr::addr_of_mut!((*bus).LunsListHead) {
                let lunExt: *mut SCSI_PORT_LUN_EXTENSION = (lunEntry as usize).wrapping_sub(
                    core::mem::offset_of!(SCSI_PORT_LUN_EXTENSION, LunEntry),
                ) as *mut SCSI_PORT_LUN_EXTENSION;
                let idx = (*deviceRelations).Count;
                (*deviceRelations).Count = (*deviceRelations).Count.wrapping_add(1);
                let target_ptr: *mut PDEVICE_OBJECT =
                    core::ptr::addr_of_mut!((*deviceRelations).Objects) as *mut PDEVICE_OBJECT;
                *target_ptr.offset(idx as isize) = (*lunExt).Common.DeviceObject;
                ObReferenceObject((*lunExt).Common.DeviceObject as *mut core::ffi::c_void);
                lunEntry = (*lunEntry).Flink;
            }
            pathId = pathId.wrapping_add(1);
        }
        if !((*deviceRelations).Count == (*PortExtension).TotalLUCount as u32) {
            RtlAssert(
                b"deviceRelations->Count == PortExtension->TotalLUCount\0".as_ptr() as *const core::ffi::c_char as *const core::ffi::c_void,
                b"drivers/storage/port/scsiport/fdo.c\0".as_ptr() as *const core::ffi::c_char as *const core::ffi::c_void,
                698,
                core::ptr::null_mut::<i8>(),
            );
        }
        (*Irp).IoStatus.Information = (deviceRelations as ULONG_PTR) as _;
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    }
    IoSkipCurrentIrpStackLocation(Irp);
    IoCallDriver((*PortExtension).Common.LowerDevice, Irp)
}

unsafe fn FdoHandleQueryCompatibleId(PwIds: *mut *mut u16) -> NTSTATUS {
    static GenScsiAdapterId: [u16; 16] = unsafe {
        core::mem::transmute::<[u8; 32], [u16; 16]>(
            *b"G\0E\0N\0_\0S\0C\0S\0I\0A\0D\0A\0P\0T\0E\0R\0\0\0",
        )
    };

    let mut Ids: *mut u16 = *PwIds;
    let mut Length: ULONG = 0u32;

    if !Ids.is_null() {
        while *Ids != 0u16 {
            Ids = Ids.add(wcslen(Ids as *const u16).wrapping_add(1));
        }
        Length = Ids.offset_from(*PwIds) as ULONG;
        Ids = *PwIds;
    }

    let NewIds: *mut u16 = ExAllocatePoolZero(
        PagedPool,
        ((Length as usize)
            .wrapping_mul(core::mem::size_of::<u16>())
            .wrapping_add(core::mem::size_of::<[u16; 16]>())
            .wrapping_add(core::mem::size_of::<u16>())) as _,
        TAG_SCSIPORT,
    ) as *mut u16;

    if NewIds.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if Length != 0 {
        core::ptr::copy_nonoverlapping(  // nolint: wide-byte-copy (Length counts WCHARs; C copies Length * sizeof(WCHAR))
            Ids as *const u16,
            NewIds as *mut u16,
            Length as usize,
        );
    }
    core::ptr::copy_nonoverlapping(
        GenScsiAdapterId.as_ptr(),
        NewIds.add(Length as usize),
        core::mem::size_of::<[u16; 16]>() / core::mem::size_of::<u16>(),
    );

    if !Ids.is_null() {
        ExFreePool(Ids as *mut core::ffi::c_void);
    }
    *PwIds = NewIds;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn FdoDispatchPnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let ioStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let portExt: PSCSI_PORT_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PSCSI_PORT_DEVICE_EXTENSION;
    let mut status: NTSTATUS = 0;

    if (*portExt).Common.IsFDO == 0 {
        RtlAssert(
            b"portExt->Common.IsFDO\0".as_ptr() as *const i8 as PVOID,
            b"drivers/storage/port/scsiport/fdo.c\0"
                .as_ptr() as *const i8 as PVOID,
            762 as ULONG,
            core::ptr::null_mut::<i8>() as PCHAR,
        );
    }

    let mut current_block_14: u64;

    match (*ioStack).MinorFunction {
        IRP_MN_START_DEVICE => {
            status = STATUS_SUCCESS;
            current_block_14 = 1054647088692577877;
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            return FdoHandleDeviceRelations(portExt, Irp);
        }
        IRP_MN_QUERY_ID => {
            if (*ioStack).Parameters.QueryId.IdType as u32
                == BusQueryCompatibleIDs as u32
            {
                (*Irp).IoStatus.Information = (0usize as ULONG_PTR) as _;
                IoForwardIrpSynchronously((*portExt).Common.LowerDevice, Irp);
                status = FdoHandleQueryCompatibleId(
                    &raw mut (*Irp).IoStatus.Information as *mut PZZWSTR,
                );
                current_block_14 = 1054647088692577877;
            } else {
                current_block_14 = 3278574377432250588;
            }
        }
        _ => {
            current_block_14 = 3278574377432250588;
        }
    }

    match current_block_14 {
        1054647088692577877 => {}
        _ => {
            IoCopyCurrentIrpStackLocationToNext(Irp);
            return IoCallDriver((*portExt).Common.LowerDevice, Irp);
        }
    }

    if status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
    }

    return status;
}
