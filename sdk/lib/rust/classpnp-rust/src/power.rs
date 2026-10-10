use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_IO_STATUS_BLOCK, IoCopyCurrentIrpStackLocationToNext, IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation, IoMarkIrpPending, IoReuseIrp, IoSetCompletionRoutine, IoSetNextIrpStackLocation, IoSkipCurrentIrpStackLocation, LPCGUID, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, STORAGE_IDLE_POWER,
    POWER_ACTION_PowerActionHibernate as PowerActionHibernate,
    POWER_ACTION_PowerActionNone as PowerActionNone,
    POWER_ACTION_PowerActionSleep as PowerActionSleep,
    _DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0,
    _DEVICE_POWER_STATE_PowerDeviceD3 as PowerDeviceD3,
    _MONITOR_DISPLAY_STATE_PowerMonitorOff as PowerMonitorOff,
    _POWER_STATE_TYPE_SystemPowerState as SystemPowerState,
    _POWER_STATE_TYPE_DevicePowerState as DevicePowerState,
};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{DO_VERIFY_VOLUME, IRP_MJ_SCSI, STATUS_DEVICE_BUSY, STATUS_DEVICE_DOES_NOT_EXIST, TRACE_LEVEL_ERROR, TRACE_LEVEL_INFORMATION, TRACE_LEVEL_WARNING, VPB_MOUNTED};
use crate::types::{CDB, CLASS_POWER_CONTEXT, CLASS_POWER_OPTIONS, CLASS_POWER_DOWN_STATE3_PowerDownDeviceInitial3, CLASS_POWER_UP_STATE_PowerUpDeviceInitial, CLASS_POWER_UP_STATE_PowerUpDeviceLocked, CLASS_POWER_UP_STATE_PowerUpDeviceOn, CLASS_POWER_UP_STATE_PowerUpDeviceStarted, CLASS_POWER_UP_STATE_PowerUpDeviceUnlocked, FDO_HACK_NO_SYNC_CACHE, IDLE_POWER_FDO_LIST_ENTRY, MAXIMUM_RETRIES, PCDB, PCLASS_POWER_CONTEXT, PCOMMON_DEVICE_EXTENSION, PFUNCTIONAL_DEVICE_EXTENSION, PIDLE_POWER_FDO_LIST_ENTRY, PSCSI_REQUEST_BLOCK, PSTORAGE_REQUEST_BLOCK, PSTORAGE_REQUEST_BLOCK_HEADER, SCSIOP_START_STOP_UNIT, SCSIOP_SYNCHRONIZE_CACHE, SCSI_ADSENSE_LUN_NOT_READY, SCSI_REQUEST_BLOCK, SCSI_SENSEQ_FORMAT_IN_PROGRESS, SCSI_SENSE_NOT_READY, SRB_FLAGS_BYPASS_LOCKED_QUEUE, SRB_FLAGS_D3_PROCESSING, SRB_FLAGS_DISABLE_AUTOSENSE, SRB_FLAGS_DISABLE_SYNCH_TRANSFER, SRB_FLAGS_FREE_SENSE_BUFFER, SRB_FLAGS_NO_DATA_TRANSFER, SRB_FLAGS_NO_QUEUE_FREEZE, SRB_FLAGS_PORT_DRIVER_ALLOCSENSE, SRB_FUNCTION_EXECUTE_SCSI, SRB_FUNCTION_LOCK_QUEUE, SRB_FUNCTION_QUIESCE_DEVICE, SRB_FUNCTION_UNLOCK_QUEUE, SRB_STATUS_AUTOSENSE_VALID, SRB_STATUS_QUEUE_FROZEN, SRB_STATUS_SUCCESS, SRB_TYPE_STORAGE_REQUEST_BLOCK, START_UNIT_TIMEOUT, STORAGE_ADDRESS_TYPE_BTL8, STORAGE_REQUEST_BLOCK_HEADER, _CDB, _CLASS_POWER_CONTEXT, _COMMON_DEVICE_EXTENSION, _FUNCTIONAL_DEVICE_EXTENSION, _SRBEXDATATYPE_SrbExDataTypeScsiCdb16 as SrbExDataTypeScsiCdb16, _STORAGE_REQUEST_BLOCK, _STORAGE_REQUEST_BLOCK_HEADER};
use ntkrnl_sys::ddk::{DEVICE_POWER_STATE, PCCHAR, PCSTR};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED, STATUS_MORE_PROCESSING_REQUIRED};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_POWER};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{NonPagedPoolNx};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CLASS_SRBEX_NO_SRBEX_DATA_BUFFER_SIZE, CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE, FailurePredictionNone, IOCTL_STORAGE_ENABLE_IDLE_POWER};
use crate::shims::*;
// Local override for ClassAcquireRemoveLock which returns LONG (shim has no return type)
unsafe extern "C" {
    #[allow(improper_ctypes)]
    fn ClassAcquireRemoveLock(DeviceObject: PDEVICE_OBJECT, Tag: PIRP) -> LONG;
}
// NEEDS SHARED SUBSTRATE: Po* and sense APIs not yet in ntkrnl-sys or driver_ddk
unsafe extern "system" {
    fn PoSetPowerState(
        DeviceObject: PDEVICE_OBJECT,
        Type: ntkrnl_sys::ddk::POWER_STATE_TYPE,
        State: ntkrnl_sys::ddk::POWER_STATE,
    ) -> ntkrnl_sys::ddk::POWER_STATE;
    fn PoQueryWatchdogTime(Pdo: PDEVICE_OBJECT, SecondsRemaining: *mut ULONG) -> BOOLEAN;
    
    
    
}
// IsEqualGUID is a macro (memcmp on GUID bytes)
#[allow(non_snake_case)]
#[inline(always)]
unsafe fn IsEqualGUID(guid1: LPCGUID, guid2: *const ntkrnl_sys::ddk::GUID) -> bool {
    core::mem::transmute::<ntkrnl_sys::ddk::GUID, u128>(*guid1) ==
    core::mem::transmute::<ntkrnl_sys::ddk::GUID, u128>(*guid2)
}
// StorDebugPrint is TracePrint alias in this context
// SET_FLAG is |= inline expansion — handled at each call site
// resolve_imports.py: explicit kernel/private-type imports
use crate::autorun::{ClasspUpdateTimerNoWakeTolerance};
use crate::types::{ClassCheckMediaState, ClassGetDeviceParameter, ClassGetVpb, ClassRetryRequest, ClassSendDeviceIoControlSynchronous, ClasspDisableTimer, ClasspEnableTimer, ClasspFailurePredictionPeriodMissed, InitializeStorageRequestBlock};
// resolve_imports.py: explicit kernel/private-type imports
use crate::guid::{GUID_CONSOLE_DISPLAY_STATE, GUID_DISK_IDLE_TIMEOUT};
use crate::types::{ClasspScreenOff, DiskIdleTimeoutInMS, IdlePowerFDOList, IdlePowerFDOListMutex, PowerSettingNotificationHandle};
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_SCSI_REQUEST_BLOCK};
// Disambiguate _STORAGE_ADAPTER_DESCRIPTOR — the local types_arch definition has SrbType,
// the ntkrnl_sys::ddk one (older header) does not. Explicit import wins over glob.
use crate::types::_STORAGE_ADAPTER_DESCRIPTOR;
// Disambiguate _SCSI_REQUEST_BLOCK — use the ntkrnl_sys one for IO stack Scsi.Srb assignment
use ntkrnl_sys::ddk::_SCSI_REQUEST_BLOCK as DdkSrb;
// resolve_imports.py: explicit kernel/private-type imports
use crate::srblib::{SrbGetSenseInfoBuffer, SrbGetSenseInfoBufferLength, SrbGetSrbFlags, SrbSetScsiStatus};

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const CLASSP_REG_SUBKEY_NAME: [::core::ffi::c_ushort; 9] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_ushort; 9]>(*b"C\0l\0a\0s\0s\0p\0n\0p\0\0\0")
};

pub const CLASSP_REG_IDLE_TIMEOUT_IN_SECONDS: [::core::ffi::c_ushort; 21] = unsafe {
    ::core::mem::transmute::<[u8; 42], [::core::ffi::c_ushort; 21]>(
        *b"I\0d\0l\0e\0T\0i\0m\0e\0o\0u\0t\0I\0n\0S\0e\0c\0o\0n\0d\0s\0\0\0",
    )
};

pub const CLASSP_REG_DISABLE_D3COLD: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"D\0i\0s\0a\0b\0l\0e\0D\x003\0C\0o\0l\0d\0\0\0",
    )
};

pub const CLASS_TAG_POWER: u32 = 0x574C6353;
pub const DEFAULT_POWER_IRP_TIMEOUT_VALUE: u32 = 10 * 60;
pub const TIME_LEFT_FOR_LOWER_DRIVERS: u32 = 30;
pub const TIME_LEFT_FOR_UPPER_DRIVERS: u32 = 5;
pub const DEFAULT_IO_TIMEOUT_VALUE: u32 = 10;
pub const MINIMUM_STOP_UNIT_TIMEOUT_VALUE: u32 = 2;
pub const MINIMAL_START_UNIT_TIMEOUT_VALUE: u32 = 60;
pub const MINIMUM_START_UNIT_TIMEOUT_VALUE: u32 = 30;
#[no_mangle]
pub unsafe extern "system" fn ClassDispatchPower(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let isRemoved: LONG;

    if (*commonExtension).__bindgen_anon_1.IsInitialized() == 0 {
        PoStartNextPowerIrp(Irp);
        IoSkipCurrentIrpStackLocation(Irp);
        return PoCallDriver((*commonExtension).LowerDeviceObject, Irp);
    }

    isRemoved = ClassAcquireRemoveLock(DeviceObject, Irp);

    if isRemoved != 0 {
        ClassReleaseRemoveLock(DeviceObject, Irp);
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;
        PoStartNextPowerIrp(Irp);
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as i8) as _);
        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    (*(*commonExtension).DevInfo).ClassPowerDevice
        .expect("non-null function pointer")(DeviceObject, Irp)
}
#[no_mangle]
pub unsafe extern "system" fn ClasspPowerUpCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let PowerContext = Context as *mut _CLASS_POWER_CONTEXT;
    if PowerContext.is_null() {
        // NT_ASSERT
        DbgPrint(
            b"Assertion failed: PowerContext != NULL\n\0".as_ptr() as PCSTR,
        );
        __int2c();
        return STATUS_INVALID_PARAMETER;
    }

    let commonExtension = (*PowerContext).DeviceObject.read().DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let fdoExtension = (*PowerContext).DeviceObject.read().DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let OriginalIrp = (*PowerContext).Irp;
    let currentStack = IoGetCurrentIrpStackLocation(OriginalIrp);
    let nextStack = IoGetNextIrpStackLocation((*(*fdoExtension).PrivateFdoData).PowerProcessIrp);

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_POWER,
        b"ClasspPowerUpCompletion: Device Object %p, Irp %p, Context %p\n\0".as_ptr() as PCCHAR,
        (*PowerContext).DeviceObject,
        Irp,
        Context,
    );

    // Determine SRB header
    let mut srbHeader = if (*((*fdoExtension).AdapterDescriptor as *mut _STORAGE_ADAPTER_DESCRIPTOR)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        let srb = core::ptr::addr_of_mut!((*(*fdoExtension).PrivateFdoData).PowerSrb.SrbEx) as *mut _STORAGE_REQUEST_BLOCK_HEADER;
        if (*PowerContext).Srb.Length as usize == core::mem::size_of::<SCSI_REQUEST_BLOCK>() {
            core::ptr::addr_of_mut!((*PowerContext).Srb) as *mut _STORAGE_REQUEST_BLOCK_HEADER
        } else {
            srb
        }
    } else {
        core::ptr::addr_of_mut!((*PowerContext).Srb) as *mut _STORAGE_REQUEST_BLOCK_HEADER
    };

    let srbFlags = SrbGetSrbFlags(srbHeader as PVOID);
    // NT_ASSERT checks (kept as assertions)
    if srbFlags & SRB_FLAGS_FREE_SENSE_BUFFER != 0 {
        DbgPrint(b"Assertion: !(srbFlags & SRB_FLAGS_FREE_SENSE_BUFFER)\n\0".as_ptr() as PCSTR);
        __int2c();
    }
    if srbFlags & SRB_FLAGS_PORT_DRIVER_ALLOCSENSE != 0 {
        DbgPrint(b"Assertion: !(srbFlags & SRB_FLAGS_PORT_DRIVER_ALLOCSENSE)\n\0".as_ptr() as PCSTR);
        __int2c();
    }
    if (*PowerContext).Options.PowerDown() != (FALSE) as u32 {
        DbgPrint(b"Assertion: PowerContext->Options.PowerDown == FALSE\n\0".as_ptr() as PCSTR);
        __int2c();
    }
    if (*PowerContext).Options.HandleSpinUp() == (FALSE) as u32 {
        DbgPrint(b"Assertion: PowerContext->Options.HandleSpinUp == TRUE\n\0".as_ptr() as PCSTR);
        __int2c();
    }

    if Irp == OriginalIrp && (*Irp).PendingReturned != 0 {
        IoMarkIrpPending(Irp);
    }

    // Increment state counter and start the state machine loop
    (*PowerContext).PowerChangeState.PowerUp += 1;

    // State machine (PowerUpDeviceLocked=1, PowerUpDeviceOn=2, PowerUpDeviceStarted=3, PowerUpDeviceUnlocked=4)
    //
    // power.c reaches PowerUpDeviceStarted two ways and they are NOT equivalent: falling
    // into `case PowerUpDeviceStarted` runs the SRB error check first, while
    // `goto ClasspPowerUpCompletionFailure` from PowerUpDeviceOn lands PAST it, on the
    // "Previously spun device up" print. The label is inside the case, so re-dispatching
    // on the state alone cannot express it -- this flag carries which entry it was.
    let mut viaFailureGoto = false;
    loop {
        match (*PowerContext).PowerChangeState.PowerUp {
            1 /* PowerUpDeviceLocked */ => {
                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER, b"(%p)\tPreviously sent power lock\n\0".as_ptr() as PCCHAR, Irp);
                IoCopyCurrentIrpStackLocationToNext(OriginalIrp);
                if (*PowerContext).Options.LockQueue() == (TRUE) as u32 && !NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER, b"(%p)\tIrp status was %lx\n\0".as_ptr() as PCCHAR, Irp, (*Irp).IoStatus.__bindgen_anon_1.Status);
                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER, b"(%p)\tSrb status was %lx\n\0".as_ptr() as PCCHAR, Irp, (*srbHeader).SrbStatus as i32);
                    IoSetCompletionRoutine(
                        OriginalIrp,
                        Some(abi_cast!(ClasspDeviceLockFailurePowerIrpCompletion)),
                        PowerContext as PVOID,
                        TRUE,
                        TRUE,
                        TRUE,
                    );
                    PoCallDriver((*commonExtension).LowerDeviceObject, OriginalIrp);
                    return STATUS_MORE_PROCESSING_REQUIRED;
                } else {
                    (*PowerContext).QueueLocked = ((*PowerContext).Options.LockQueue()) as u8;
                }
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
                (*PowerContext).PowerChangeState.PowerUp = CLASS_POWER_UP_STATE_PowerUpDeviceLocked;
                IoSetCompletionRoutine(
                    OriginalIrp,
                    Some(abi_cast!(ClasspPowerUpCompletion)),
                    PowerContext as PVOID,
                    TRUE,
                    TRUE,
                    TRUE,
                );
                let status = PoCallDriver((*commonExtension).LowerDeviceObject, OriginalIrp);
                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER, b"(%p)\tPoCallDriver returned %lx\n\0".as_ptr() as PCCHAR, OriginalIrp, status);
                break; // return STATUS_MORE_PROCESSING_REQUIRED below
            }

            2 /* PowerUpDeviceOn */ => {
                if NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
                    let mut cdb: *mut _CDB;
                    let mut secondsRemaining: ULONG = 0;
                    let mut timeoutValue: ULONG = 0;
                    let mut startUnitTimeout: ULONG;

                    if PoQueryWatchdogTime((*fdoExtension).LowerPdo, &mut secondsRemaining) != FALSE {
                        secondsRemaining = secondsRemaining.min(DEFAULT_POWER_IRP_TIMEOUT_VALUE);
                        if secondsRemaining >= START_UNIT_TIMEOUT {
                            startUnitTimeout = START_UNIT_TIMEOUT;
                        } else {
                            startUnitTimeout = MINIMAL_START_UNIT_TIMEOUT_VALUE;
                        }
                        if secondsRemaining >= (TIME_LEFT_FOR_UPPER_DRIVERS + startUnitTimeout) {
                            (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount =
                                (secondsRemaining - TIME_LEFT_FOR_UPPER_DRIVERS) / startUnitTimeout;
                            (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount -= 1;
                            if (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount != 0 {
                                timeoutValue = startUnitTimeout;
                            } else {
                                timeoutValue = secondsRemaining - TIME_LEFT_FOR_UPPER_DRIVERS;
                            }
                        } else {
                            // NT_ASSERT
                            if secondsRemaining < DEFAULT_IO_TIMEOUT_VALUE {
                                DbgPrint(b"secondsRemaining >= DEFAULT_IO_TIMEOUT_VALUE\n\0".as_ptr() as PCSTR);
                                __int2c();
                            }
                            (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount = 0;
                            timeoutValue = MINIMUM_START_UNIT_TIMEOUT_VALUE;
                        }
                    } else {
                        (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount =
                            DEFAULT_POWER_IRP_TIMEOUT_VALUE / START_UNIT_TIMEOUT - 1;
                        timeoutValue = START_UNIT_TIMEOUT;
                    }

                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER, b"(%p)\tSending start unit to device\n\0".as_ptr() as PCCHAR, Irp);
                    (*PowerContext).RetryCount = (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount;

                    if (*((*fdoExtension).AdapterDescriptor as *mut _STORAGE_ADAPTER_DESCRIPTOR)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
                        let status = InitializeStorageRequestBlock(
                            srbHeader as *mut _STORAGE_REQUEST_BLOCK,
                            STORAGE_ADDRESS_TYPE_BTL8 as u16,
                            CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as ULONG,
                            1,
                            SrbExDataTypeScsiCdb16 as i32,
                        );
                        if NT_SUCCESS(status) {
                            (*(srbHeader as *mut _STORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI as ULONG;
                            (*PowerContext).Srb.Length = 0;
                        } else {
                            // NT_ASSERT(FALSE)
                            DbgPrint(b"FALSE\n\0".as_ptr() as PCSTR);
                            __int2c();
                            srbHeader = core::ptr::addr_of_mut!((*PowerContext).Srb) as *mut _STORAGE_REQUEST_BLOCK_HEADER;
                            core::ptr::write_bytes(srbHeader as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                            (*srbHeader).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;
                            (*srbHeader).Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
                        }
                    } else {
                        core::ptr::write_bytes(srbHeader as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                        (*srbHeader).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;
                        (*srbHeader).Function = SRB_FUNCTION_EXECUTE_SCSI as u8;
                    }

                    SrbSetOriginalRequest(srbHeader as PVOID, ((*(*fdoExtension).PrivateFdoData).PowerProcessIrp as PVOID).cast());
                    SrbSetSenseInfoBuffer(srbHeader as PVOID, (*(*commonExtension).PartitionZeroExtension).SenseData as PVOID);
                    SrbSetSenseInfoBufferLength(srbHeader as PVOID,
                        (GET_FDO_EXTENSON_SENSE_DATA_LENGTH((*commonExtension).PartitionZeroExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION) as u32) as _);
                    SrbSetTimeOutValue(srbHeader as PVOID, timeoutValue);
                    SrbAssignSrbFlags(srbHeader as PVOID,
                        SRB_FLAGS_NO_DATA_TRANSFER | SRB_FLAGS_DISABLE_AUTOSENSE | SRB_FLAGS_DISABLE_SYNCH_TRANSFER | SRB_FLAGS_NO_QUEUE_FREEZE );
                    if (*PowerContext).Options.LockQueue() != 0 {
                        SrbSetSrbFlags(srbHeader as PVOID, SRB_FLAGS_BYPASS_LOCKED_QUEUE);
                    }
                    SrbSetCdbLength(srbHeader as PVOID, 6);
                    cdb = SrbGetCdb(srbHeader as PVOID);
                    core::ptr::write_bytes(cdb as *mut u8, 0, core::mem::size_of::<_CDB>());
                    (*cdb).START_STOP.as_mut().OperationCode = SCSIOP_START_STOP_UNIT as u8;
                    (*cdb).START_STOP.as_mut().set_Start(1);

                    (*PowerContext).PowerChangeState.PowerUp = CLASS_POWER_UP_STATE_PowerUpDeviceOn;
                    IoSetCompletionRoutine(
                        (*(*fdoExtension).PrivateFdoData).PowerProcessIrp,
                        Some(abi_cast!(ClasspPowerUpCompletion)),
                        PowerContext as PVOID,
                        TRUE,
                        TRUE,
                        TRUE,
                    );
                    (*nextStack).Parameters.Scsi.Srb = srbHeader as *mut DdkSrb;
                    (*nextStack).MajorFunction = IRP_MJ_SCSI as u8;
                    let status = IoCallDriver((*commonExtension).LowerDeviceObject, (*(*fdoExtension).PrivateFdoData).PowerProcessIrp);
                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER, b"(%p)\tIoCallDriver returned %lx\n\0".as_ptr() as PCCHAR,
                        (*(*fdoExtension).PrivateFdoData).PowerProcessIrp, status);
                    break;
                } else {
                    // power irp is failed by lower driver. we're done.
                    // `goto ClasspPowerUpCompletionFailure` -- the label sits inside
                    // case PowerUpDeviceStarted, after its SRB error check.
                    (*PowerContext).FinalStatus = (*Irp).IoStatus.__bindgen_anon_1.Status;
                    (*PowerContext).PowerChangeState.PowerUp = CLASS_POWER_UP_STATE_PowerUpDeviceStarted;
                    viaFailureGoto = true;
                    continue;
                }
            }

            3 /* PowerUpDeviceStarted */ => {
                // First deal with an error if one occurred -- but only on the fall-in
                // path; `goto ClasspPowerUpCompletionFailure` enters below this block.
                // SRB_STATUS() masks off AUTOSENSE_VALID|QUEUE_FROZEN (srb.h:149); comparing
                // the raw SrbStatus made a SUCCESS that merely carried sense data look
                // like a failure.
                if !viaFailureGoto
                    && SRB_STATUS((*srbHeader).SrbStatus) != (SRB_STATUS_SUCCESS) as u8
                {
                    let retry: BOOLEAN;
                    let mut delta100nsUnits: LONGLONG = 0;
                    let mut secondsRemaining: ULONG = 0;
                    let mut startUnitTimeout: ULONG = START_UNIT_TIMEOUT;
                    let mut status: NTSTATUS = 0;

                    TracePrint(
                        TRACE_LEVEL_ERROR,
                        TRACE_FLAG_POWER,
                        b"%p\tError occured when issuing START_UNIT command to device. Srb %p, Status %x\n\0"
                            .as_ptr() as PCCHAR,
                        Irp,
                        srbHeader,
                        (*srbHeader).SrbStatus as i32,
                    );

                    NT_ASSERT!(TEST_FLAG((*srbHeader).SrbStatus as ULONG, SRB_STATUS_QUEUE_FROZEN as ULONG) == 0);
                    NT_ASSERT!(
                        (*srbHeader).Function as u32 == SRB_FUNCTION_EXECUTE_SCSI
                            || (*(srbHeader as *mut _STORAGE_REQUEST_BLOCK)).SrbFunction
                                == SRB_FUNCTION_EXECUTE_SCSI
                    );

                    (*PowerContext).RetryInterval = 0;
                    retry = InterpretSenseInfoWithoutHistory(
                        // classpnp.h:867-874 -- DeviceObject is the anonymous union's
                        // member, at the same offset as CommonExtension.DeviceObject.
                        (*fdoExtension).__bindgen_anon_1.__bindgen_anon_1.as_ref().DeviceObject,
                        Irp,
                        srbHeader as *mut SCSI_REQUEST_BLOCK,
                        IRP_MJ_SCSI as u8,
                        IRP_MJ_POWER as u32,
                        (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount
                            - (*PowerContext).RetryCount,
                        core::ptr::addr_of_mut!(status),
                        core::ptr::addr_of_mut!(delta100nsUnits),
                    );

                    // NOTE: Power context is a public structure, and thus cannot be
                    //       updated to use 100ns units.  Therefore, must store the
                    //       one-second equivalent.  Round up to ensure minimum delay
                    //       requirements have been met.
                    delta100nsUnits += (10 * 1000 * 1000) - 1;
                    delta100nsUnits /= 10 * 1000 * 1000;
                    // guaranteed not to have high bits set per SAL annotations
                    (*PowerContext).RetryInterval = delta100nsUnits as ULONG;

                    // The C is `PowerContext->RetryCount-- != 0`: it tests the OLD value
                    // and decrements either way, but only when `retry` is TRUE (&& is
                    // short-circuiting, so a FALSE retry leaves RetryCount alone).
                    let mut takeRetry = false;
                    if retry == TRUE as BOOLEAN {
                        let oldRetryCount = (*PowerContext).RetryCount;
                        (*PowerContext).RetryCount = oldRetryCount.wrapping_sub(1);
                        takeRetry = oldRetryCount != 0;
                    }

                    if takeRetry {
                        TracePrint(
                            TRACE_LEVEL_INFORMATION,
                            TRACE_FLAG_POWER,
                            b"(%p)\tRetrying failed request\n\0".as_ptr() as PCCHAR,
                            Irp,
                        );

                        // Decrement the state so we come back through here the next time.
                        (*PowerContext).PowerChangeState.PowerUp -= 1;

                        // Adjust start unit timeout based on remaining time if needed.
                        if PoQueryWatchdogTime(
                            (*fdoExtension).LowerPdo,
                            core::ptr::addr_of_mut!(secondsRemaining),
                        ) != 0
                        {
                            if secondsRemaining >= TIME_LEFT_FOR_UPPER_DRIVERS {
                                secondsRemaining -= TIME_LEFT_FOR_UPPER_DRIVERS;
                            }

                            // MINIMAL (60) and MINIMUM (30) are two different constants
                            // (power.c:52-53) and the branches do NOT use the same one.
                            if secondsRemaining < MINIMAL_START_UNIT_TIMEOUT_VALUE {
                                startUnitTimeout = MINIMUM_START_UNIT_TIMEOUT_VALUE;
                            } else if secondsRemaining < START_UNIT_TIMEOUT {
                                startUnitTimeout = MINIMAL_START_UNIT_TIMEOUT_VALUE;
                            }
                        }

                        SrbSetTimeOutValue(srbHeader as PVOID, startUnitTimeout);

                        RetryPowerRequest((*commonExtension).DeviceObject, Irp, PowerContext);

                        break;
                    }

                    // reset retry count for UNLOCK command.
                    (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount =
                        MAXIMUM_RETRIES;
                    (*PowerContext).RetryCount = MAXIMUM_RETRIES;
                }

                // ClasspPowerUpCompletionFailure:
                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER, b"(%p)\tPreviously spun device up\n\0".as_ptr() as PCCHAR, Irp);

                // If QueueLocked, perform unlock and send new IRP, return.
                if (*PowerContext).QueueLocked != 0 {
                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER, b"(%p)\tUnlocking queue\n\0".as_ptr() as PCCHAR, Irp);

                    if (*((*fdoExtension).AdapterDescriptor as *mut _STORAGE_ADAPTER_DESCRIPTOR)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
                        let status = InitializeStorageRequestBlock(
                            srbHeader as *mut _STORAGE_REQUEST_BLOCK,
                            STORAGE_ADDRESS_TYPE_BTL8 as u16,
                            CLASS_SRBEX_NO_SRBEX_DATA_BUFFER_SIZE as ULONG,
                            0,
                            0, // SrbExType unused (NumSrbExData=0)
                        );
                        if NT_SUCCESS(status) {
                            (*(srbHeader as *mut _STORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_UNLOCK_QUEUE as ULONG;
                            (*PowerContext).Srb.Length = 0;
                        } else {
                            DbgPrint(b"FALSE\n\0".as_ptr() as PCSTR);
                            __int2c();
                            srbHeader = core::ptr::addr_of_mut!((*PowerContext).Srb) as *mut _STORAGE_REQUEST_BLOCK_HEADER;
                            core::ptr::write_bytes(srbHeader as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                            (*srbHeader).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;
                            (*srbHeader).Function = SRB_FUNCTION_UNLOCK_QUEUE as u8;
                        }
                    } else {
                        core::ptr::write_bytes(srbHeader as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                        (*srbHeader).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u16;
                        (*srbHeader).Function = SRB_FUNCTION_UNLOCK_QUEUE as u8;
                    }
                    SrbAssignSrbFlags(srbHeader as PVOID, SRB_FLAGS_BYPASS_LOCKED_QUEUE);
                    SrbSetOriginalRequest(srbHeader as PVOID, ((*(*fdoExtension).PrivateFdoData).PowerProcessIrp as PVOID).cast());
                    (*nextStack).Parameters.Scsi.Srb = srbHeader as *mut DdkSrb;
                    (*nextStack).MajorFunction = IRP_MJ_SCSI as u8;
                    (*PowerContext).PowerChangeState.PowerUp = CLASS_POWER_UP_STATE_PowerUpDeviceStarted;
                    IoSetCompletionRoutine(
                        (*(*fdoExtension).PrivateFdoData).PowerProcessIrp,
                        Some(abi_cast!(ClasspPowerUpCompletion)),
                        PowerContext as PVOID,
                        TRUE,
                        TRUE,
                        TRUE,
                    );
                    let status = IoCallDriver((*commonExtension).LowerDeviceObject, (*(*fdoExtension).PrivateFdoData).PowerProcessIrp);
                    TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER, b"(%p)\tIoCallDriver returned %lx\n\0".as_ptr() as PCCHAR,
                        (*(*fdoExtension).PrivateFdoData).PowerProcessIrp, status);
                    break; // return
                }
                // If not QueueLocked, fall through to PowerUpDeviceUnlocked
                (*PowerContext).PowerChangeState.PowerUp = CLASS_POWER_UP_STATE_PowerUpDeviceUnlocked;
                continue;
            }

            4 /* PowerUpDeviceUnlocked */ => {
                // Final completion
                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER, b"(%p)\tFall-through (queue not locked)\n\0".as_ptr() as PCCHAR, OriginalIrp);
                TracePrint(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER, b"(%p)\tFreeing srb and completing\n\0".as_ptr() as PCCHAR, OriginalIrp);

                let status = (*PowerContext).FinalStatus;
                (*OriginalIrp).IoStatus.__bindgen_anon_1.Status = status;

                if NT_SUCCESS(status) {
                    (*fdoExtension).DevicePowerState = (*currentStack).Parameters.Power.State.DeviceState;
                }

                let mut FailurePredictionEnabled = FALSE;
                if !(*fdoExtension).FailurePredictionInfo.is_null() &&
                    (*(*fdoExtension).FailurePredictionInfo).Method != FailurePredictionNone {
                    FailurePredictionEnabled = TRUE;
                }

                if (*(*commonExtension).DriverExtension).InitData.ClassTick.is_some() ||
                    (!(*fdoExtension).MediaChangeDetectionInfo.is_null() &&
                     !(*fdoExtension).FunctionSupportInfo.is_null() &&
                     (*(*fdoExtension).FunctionSupportInfo).AsynchronousNotificationSupported == FALSE) ||
                    FailurePredictionEnabled != 0 {

                    if FailurePredictionEnabled != 0 && ClasspFailurePredictionPeriodMissed(fdoExtension) != 0 {
                        (*(*fdoExtension).FailurePredictionInfo).CountDown = 1;
                    }
                    ClasspEnableTimer(fdoExtension);
                }

                PoSetPowerState(
                    (*PowerContext).DeviceObject,
                    (*currentStack).Parameters.Power.Type,
                    (*currentStack).Parameters.Power.State,
                );

                ClassReleaseRemoveLock((*PowerContext).DeviceObject, OriginalIrp);
                (*PowerContext).InUse = FALSE;
                PoStartNextPowerIrp(OriginalIrp);

                // Prevent completing the IRP we allocated
                if !(*fdoExtension).PrivateFdoData.is_null() && Irp == (*(*fdoExtension).PrivateFdoData).PowerProcessIrp {
                    ClassCompleteRequest((*commonExtension).DeviceObject, OriginalIrp, (IO_NO_INCREMENT as i8) as _);
                    return STATUS_MORE_PROCESSING_REQUIRED;
                }

                return status;
            }

            _ => break,
        }
    }
    STATUS_MORE_PROCESSING_REQUIRED
}
#[no_mangle]
pub unsafe extern "system" fn ClasspPowerDownCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let PowerContext = Context as PCLASS_POWER_CONTEXT;
    let fdoExtension = (*(*PowerContext).DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let commonExtension = (*(*PowerContext).DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let OriginalIrp = (*PowerContext).Irp;

    let currentStack = IoGetCurrentIrpStackLocation(OriginalIrp);
    let nextStack = IoGetNextIrpStackLocation((*(*fdoExtension).PrivateFdoData).PowerProcessIrp);
    let mut status: NTSTATUS = STATUS_MORE_PROCESSING_REQUIRED;

    // UNREFERENCED_PARAMETER
    let _ = DeviceObject;

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_POWER,
        b"ClasspPowerDownCompletion: Device Object %p, Irp %p, Context %p\n\0".as_ptr() as *const i8,
        (*PowerContext).DeviceObject,
        Irp,
        Context,
    );

    let mut srbHeader: PSTORAGE_REQUEST_BLOCK_HEADER;
    if (*((*fdoExtension).AdapterDescriptor as *mut _STORAGE_ADAPTER_DESCRIPTOR)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srbHeader = core::ptr::addr_of_mut!((*(*fdoExtension).PrivateFdoData).PowerSrb.SrbEx)
            as PSTORAGE_REQUEST_BLOCK_HEADER;
        if (*PowerContext).Srb.Length as usize == core::mem::size_of::<SCSI_REQUEST_BLOCK>() {
            srbHeader = core::ptr::addr_of_mut!((*PowerContext).Srb) as PSTORAGE_REQUEST_BLOCK_HEADER;
        }
    } else {
        srbHeader = core::ptr::addr_of_mut!((*PowerContext).Srb) as PSTORAGE_REQUEST_BLOCK_HEADER;
    }

    let srbFlags = SrbGetSrbFlags(srbHeader as PVOID);
    ntkrnl_sys::ASSERT!(
        srbFlags & SRB_FLAGS_FREE_SENSE_BUFFER == 0,
        "!TEST_FLAG(srbFlags, SRB_FLAGS_FREE_SENSE_BUFFER)"
    );
    ntkrnl_sys::ASSERT!(
        srbFlags & SRB_FLAGS_PORT_DRIVER_ALLOCSENSE == 0,
        "!TEST_FLAG(srbFlags, SRB_FLAGS_PORT_DRIVER_ALLOCSENSE)"
    );
    ntkrnl_sys::ASSERT!((*PowerContext).Options.PowerDown() == (TRUE) as u32, "PowerContext->Options.PowerDown == TRUE");
    ntkrnl_sys::ASSERT!((*PowerContext).Options.HandleSpinDown() != 0, "PowerContext->Options.HandleSpinDown");

    if Irp == OriginalIrp && (*Irp).PendingReturned != 0 {
        IoMarkIrpPending(Irp);
    }

    (*PowerContext).PowerChangeState.PowerDown3 += 1;
    let mut state = (*PowerContext).PowerChangeState.PowerDown3;

    loop {
        match state {
            1 => {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_POWER,
                    b"(%p)\tPreviously sent power lock\n\0".as_ptr() as *const i8,
                    Irp,
                );

                if (*PowerContext).Options.LockQueue() == (TRUE) as u32 && !NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tIrp status was %lx\n\0".as_ptr() as *const i8,
                        Irp,
                        (*Irp).IoStatus.__bindgen_anon_1.Status,
                    );
                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tSrb status was %lx\n\0".as_ptr() as *const i8,
                        Irp,
                        (*srbHeader).SrbStatus as i32,
                    );

                    (*fdoExtension).DevicePowerState =(*currentStack).Parameters.Power.State.DeviceState;

                    IoCopyCurrentIrpStackLocationToNext(OriginalIrp);
                    IoSetCompletionRoutine(
                        OriginalIrp,
                        Some(abi_cast!(ClasspStartNextPowerIrpCompletion)),
                        PowerContext as PVOID,
                        TRUE,
                        TRUE,
                        TRUE,
                    );
                    PoSetPowerState(
                        (*PowerContext).DeviceObject,
                        (*currentStack).Parameters.Power.Type,
                        (*currentStack).Parameters.Power.State,
                    );
                    (*fdoExtension).PowerDownInProgress =FALSE;
                    ClassReleaseRemoveLock((*commonExtension).DeviceObject, OriginalIrp);
                    PoCallDriver((*commonExtension).LowerDeviceObject, OriginalIrp);
                    return STATUS_MORE_PROCESSING_REQUIRED;
                } else {
                    (*PowerContext).QueueLocked = (*PowerContext).Options.LockQueue() as BOOLEAN;
                    (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount = 0;
                    (*PowerContext).RetryCount = 0;

                    if (*((*fdoExtension).AdapterDescriptor as *mut _STORAGE_ADAPTER_DESCRIPTOR)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
                        srbHeader = core::ptr::addr_of_mut!(
                            (*(*fdoExtension).PrivateFdoData).PowerSrb.SrbEx
                        ) as PSTORAGE_REQUEST_BLOCK_HEADER;
                        status = InitializeStorageRequestBlock(
                            srbHeader as PSTORAGE_REQUEST_BLOCK,
                            STORAGE_ADDRESS_TYPE_BTL8 as USHORT,
                            CLASS_SRBEX_NO_SRBEX_DATA_BUFFER_SIZE as ULONG,
                            0,
                            0, // SrbExType unused (NumSrbExData=0)
                        );
                        if NT_SUCCESS(status) {
                            (*(srbHeader as PSTORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_QUIESCE_DEVICE;
                        } else {
                            assert!(false, "FALSE");
                            srbHeader = core::ptr::addr_of_mut!((*PowerContext).Srb) as PSTORAGE_REQUEST_BLOCK_HEADER;
                            (*srbHeader).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
                            (*srbHeader).Function = (SRB_FUNCTION_QUIESCE_DEVICE) as u8;
                        }
                    } else {
                        srbHeader = core::ptr::addr_of_mut!((*PowerContext).Srb) as PSTORAGE_REQUEST_BLOCK_HEADER;
                        (*srbHeader).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
                        (*srbHeader).Function = (SRB_FUNCTION_QUIESCE_DEVICE) as u8;
                    }

                    SrbSetOriginalRequest(srbHeader as PVOID, (*(*fdoExtension).PrivateFdoData).PowerProcessIrp as PIRP);
                    SrbSetTimeOutValue(srbHeader as PVOID, (*fdoExtension).TimeOutValue);
                    SrbAssignSrbFlags(
                        srbHeader as PVOID,
                        SRB_FLAGS_NO_DATA_TRANSFER
                            | SRB_FLAGS_DISABLE_AUTOSENSE
                            | SRB_FLAGS_DISABLE_SYNCH_TRANSFER
                            | SRB_FLAGS_NO_QUEUE_FREEZE
                            | SRB_FLAGS_BYPASS_LOCKED_QUEUE
                            | SRB_FLAGS_D3_PROCESSING ,
                    );

                    IoSetCompletionRoutine(
                        (*(*fdoExtension).PrivateFdoData).PowerProcessIrp,
                        Some(abi_cast!(ClasspPowerDownCompletion)),
                        PowerContext as PVOID,
                        TRUE,
                        TRUE,
                        TRUE,
                    );
                    (*nextStack).Parameters.Scsi.Srb = srbHeader as *mut DdkSrb;
                    (*nextStack).MajorFunction = (IRP_MJ_SCSI) as u8;

                    status = IoCallDriver((*commonExtension).LowerDeviceObject, (*(*fdoExtension).PrivateFdoData).PowerProcessIrp);

                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tIoCallDriver returned %lx\n\0".as_ptr() as *const i8,
                        (*(*fdoExtension).PrivateFdoData).PowerProcessIrp,
                        status,
                    );

                    return STATUS_MORE_PROCESSING_REQUIRED;
                }
            }

            2 => {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_POWER,
                    b"(%p)\tPreviously sent device quiesce\n\0".as_ptr() as *const i8,
                    Irp,
                );

                if TEST_FLAG(
                    ((*(*fdoExtension).PrivateFdoData).HackFlags) as u32,
                    FDO_HACK_NO_SYNC_CACHE,
                ) == 0 {
                    (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount = MAXIMUM_RETRIES;
                    (*PowerContext).RetryCount = MAXIMUM_RETRIES;

                    if (*((*fdoExtension).AdapterDescriptor as *mut _STORAGE_ADAPTER_DESCRIPTOR)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
                        status = InitializeStorageRequestBlock(
                            srbHeader as PSTORAGE_REQUEST_BLOCK,
                            STORAGE_ADDRESS_TYPE_BTL8 as USHORT,
                            CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as ULONG,
                            1,
                            SrbExDataTypeScsiCdb16,
                        );
                        if NT_SUCCESS(status) {
                            (*(srbHeader as PSTORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
                            (*PowerContext).Srb.Length = 0;
                        } else {
                            assert!(false, "FALSE");
                            srbHeader = core::ptr::addr_of_mut!((*PowerContext).Srb) as PSTORAGE_REQUEST_BLOCK_HEADER;
                            core::ptr::write_bytes(srbHeader as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                            (*srbHeader).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
                            (*srbHeader).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
                        }
                    } else {
                        core::ptr::write_bytes(srbHeader as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                        (*srbHeader).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
                        (*srbHeader).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
                    }

                    SrbSetOriginalRequest(srbHeader as PVOID, (*(*fdoExtension).PrivateFdoData).PowerProcessIrp as PIRP);
                    SrbSetSenseInfoBuffer(srbHeader as PVOID, (*(*commonExtension).PartitionZeroExtension).SenseData as PVOID);
                    SrbSetSenseInfoBufferLength(
                        srbHeader as PVOID,
                        (GET_FDO_EXTENSON_SENSE_DATA_LENGTH((*commonExtension).PartitionZeroExtension) as u32) as _,
                    );
                    SrbSetTimeOutValue(srbHeader as PVOID, (*fdoExtension).TimeOutValue);
                    SrbAssignSrbFlags(
                        srbHeader as PVOID,
                        SRB_FLAGS_NO_DATA_TRANSFER
                            | SRB_FLAGS_DISABLE_AUTOSENSE
                            | SRB_FLAGS_DISABLE_SYNCH_TRANSFER
                            | SRB_FLAGS_NO_QUEUE_FREEZE
                            | SRB_FLAGS_BYPASS_LOCKED_QUEUE
                            | SRB_FLAGS_D3_PROCESSING ,
                    );
                    SrbSetCdbLength(srbHeader as PVOID, 10u8);

                    let cdb: PCDB = SrbGetCdb(srbHeader as PVOID);
                    core::ptr::write_bytes(cdb as *mut u8, 0, core::mem::size_of::<CDB>());
                    (*cdb).SYNCHRONIZE_CACHE10.as_mut().OperationCode = (SCSIOP_SYNCHRONIZE_CACHE) as u8;

                    IoSetCompletionRoutine(
                        (*(*fdoExtension).PrivateFdoData).PowerProcessIrp,
                        Some(abi_cast!(ClasspPowerDownCompletion)),
                        PowerContext as PVOID,
                        TRUE,
                        TRUE,
                        TRUE,
                    );
                    (*nextStack).Parameters.Scsi.Srb = srbHeader as *mut DdkSrb;
                    (*nextStack).MajorFunction = (IRP_MJ_SCSI) as u8;

                    status = IoCallDriver((*commonExtension).LowerDeviceObject, (*(*fdoExtension).PrivateFdoData).PowerProcessIrp);

                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tIoCallDriver returned %lx\n\0".as_ptr() as *const i8,
                        (*(*fdoExtension).PrivateFdoData).PowerProcessIrp,
                        status,
                    );

                    return STATUS_MORE_PROCESSING_REQUIRED;
                } else {
                    TracePrint(
                        TRACE_LEVEL_WARNING,
                        TRACE_FLAG_POWER,
                        b"(%p)\tPower Down: not sending SYNCH_CACHE\n\0".as_ptr() as *const i8,
                        (*PowerContext).DeviceObject,
                    );
                    (*PowerContext).PowerChangeState.PowerDown3 += 1;
                    state += 1;
                    (*srbHeader).SrbStatus = (SRB_STATUS_SUCCESS) as u8;
                    status = STATUS_SUCCESS;
                    continue;
                }
            }

            3 => {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_POWER,
                    b"(%p)\tPreviously send SCSIOP_SYNCHRONIZE_CACHE\n\0".as_ptr() as *const i8,
                    Irp,
                );

                if SRB_STATUS((*srbHeader).SrbStatus) != (SRB_STATUS_SUCCESS) as u8 {
                    let mut delta100nsUnits: LONGLONG = 0;
                    (*PowerContext).RetryInterval = 0;
                    let retry = InterpretSenseInfoWithoutHistory(
                        (*commonExtension).DeviceObject,
                        Irp,
                        srbHeader as PSCSI_REQUEST_BLOCK,
                        IRP_MJ_SCSI as u8,
                        IRP_MJ_POWER as u32,
                        (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount - (*PowerContext).RetryCount,
                        &mut status,
                        &mut delta100nsUnits,
                    );
                    delta100nsUnits += (10 * 1000 * 1000) - 1;
                    delta100nsUnits /= 10 * 1000 * 1000;
                    (*PowerContext).RetryInterval = delta100nsUnits as ULONG;

                    if retry == TRUE {
                        let retry_count = (*PowerContext).RetryCount;
                        (*PowerContext).RetryCount -= 1;
                        if retry_count != 0 {
                            TracePrint(
                                TRACE_LEVEL_INFORMATION,
                                TRACE_FLAG_POWER,
                                b"(%p)\tRetrying failed request\n\0".as_ptr() as *const i8,
                                Irp,
                            );
                            (*PowerContext).PowerChangeState.PowerDown3 -= 1;
                            state -= 1;
                            RetryPowerRequest((*commonExtension).DeviceObject, Irp, PowerContext);
                            return STATUS_MORE_PROCESSING_REQUIRED;
                        }
                    }
                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tSYNCHRONIZE_CACHE not retried\n\0".as_ptr() as *const i8,
                        Irp,
                    );
                    (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount = MAXIMUM_RETRIES;
                    (*PowerContext).RetryCount = MAXIMUM_RETRIES;
                }

                if (*currentStack).Parameters.Power.State.DeviceState == PowerDeviceD3
                    && (*currentStack).Parameters.Power.ShutdownType == PowerActionHibernate
                    && (*commonExtension).HibernationPathCount != 0
                {
                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tPower Down: not sending SPIN DOWN due to hibernation path\n\0".as_ptr() as *const i8,
                        (*PowerContext).DeviceObject,
                    );
                    (*PowerContext).PowerChangeState.PowerDown3 += 1;
                    state += 1;
                    (*srbHeader).SrbStatus = (SRB_STATUS_SUCCESS) as u8;
                    status = STATUS_SUCCESS;
                    continue;
                } else {
                    // Send STOP UNIT
                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tSending stop unit to device\n\0".as_ptr() as *const i8,
                        Irp,
                    );

                    let mut secondsRemaining: ULONG = 0;
                    let timeoutValue: ULONG;
                    if PoQueryWatchdogTime((*fdoExtension).LowerPdo, &mut secondsRemaining) != 0 {
                        if secondsRemaining >= (TIME_LEFT_FOR_LOWER_DRIVERS + DEFAULT_IO_TIMEOUT_VALUE) {
                            (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount =
                                (secondsRemaining - TIME_LEFT_FOR_LOWER_DRIVERS) / DEFAULT_IO_TIMEOUT_VALUE;
                            (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount -= 1;
                            if (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount > 0 {
                                timeoutValue = DEFAULT_IO_TIMEOUT_VALUE;
                            } else {
                                timeoutValue = secondsRemaining - TIME_LEFT_FOR_LOWER_DRIVERS;
                            }
                            if (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount > MAXIMUM_RETRIES {
                                (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount = MAXIMUM_RETRIES;
                            }
                        } else {
                            (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount = 0;
                            if secondsRemaining > 2 * MINIMUM_STOP_UNIT_TIMEOUT_VALUE {
                                timeoutValue = secondsRemaining - MINIMUM_STOP_UNIT_TIMEOUT_VALUE;
                            } else {
                                timeoutValue = MINIMUM_STOP_UNIT_TIMEOUT_VALUE;
                            }
                        }
                    } else {
                        (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount = MAXIMUM_RETRIES;
                        timeoutValue = DEFAULT_IO_TIMEOUT_VALUE;
                    }
                    (*PowerContext).RetryCount = (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount;

                    if (*((*fdoExtension).AdapterDescriptor as *mut _STORAGE_ADAPTER_DESCRIPTOR)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
                        status = InitializeStorageRequestBlock(
                            srbHeader as PSTORAGE_REQUEST_BLOCK,
                            STORAGE_ADDRESS_TYPE_BTL8 as USHORT,
                            CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE as ULONG,
                            1,
                            SrbExDataTypeScsiCdb16,
                        );
                        if NT_SUCCESS(status) {
                            (*(srbHeader as PSTORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_EXECUTE_SCSI;
                            (*PowerContext).Srb.Length = 0;
                        } else {
                            assert!(false, "FALSE");
                            srbHeader = core::ptr::addr_of_mut!((*PowerContext).Srb) as PSTORAGE_REQUEST_BLOCK_HEADER;
                            core::ptr::write_bytes(srbHeader as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                            (*srbHeader).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
                            (*srbHeader).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
                        }
                    } else {
                        core::ptr::write_bytes(srbHeader as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                        (*srbHeader).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
                        (*srbHeader).Function = (SRB_FUNCTION_EXECUTE_SCSI) as u8;
                    }

                    SrbSetOriginalRequest(srbHeader as PVOID, (*(*fdoExtension).PrivateFdoData).PowerProcessIrp as PIRP);
                    SrbSetSenseInfoBuffer(srbHeader as PVOID, (*(*commonExtension).PartitionZeroExtension).SenseData as PVOID);
                    SrbSetSenseInfoBufferLength(
                        srbHeader as PVOID,
                        (GET_FDO_EXTENSON_SENSE_DATA_LENGTH((*commonExtension).PartitionZeroExtension) as u32) as _,
                    );
                    SrbSetTimeOutValue(srbHeader as PVOID, timeoutValue);
                    SrbAssignSrbFlags(
                        srbHeader as PVOID,
                        SRB_FLAGS_NO_DATA_TRANSFER
                            | SRB_FLAGS_DISABLE_AUTOSENSE
                            | SRB_FLAGS_DISABLE_SYNCH_TRANSFER
                            | SRB_FLAGS_NO_QUEUE_FREEZE
                            | SRB_FLAGS_BYPASS_LOCKED_QUEUE
                            | SRB_FLAGS_D3_PROCESSING ,
                    );
                    SrbSetCdbLength(srbHeader as PVOID, 6u8);

                    let cdb: PCDB = SrbGetCdb(srbHeader as PVOID);
                    core::ptr::write_bytes(cdb as *mut u8, 0, core::mem::size_of::<CDB>());
                    (*cdb).START_STOP.as_mut().OperationCode = (SCSIOP_START_STOP_UNIT) as u8;
                    (*cdb).START_STOP.as_mut().set_Start(0);
                    (*cdb).START_STOP.as_mut().set_Immediate(1);

                    IoSetCompletionRoutine(
                        (*(*fdoExtension).PrivateFdoData).PowerProcessIrp,
                        Some(abi_cast!(ClasspPowerDownCompletion)),
                        PowerContext as PVOID,
                        TRUE,
                        TRUE,
                        TRUE,
                    );
                    (*nextStack).Parameters.Scsi.Srb = srbHeader as *mut DdkSrb;
                    (*nextStack).MajorFunction = (IRP_MJ_SCSI) as u8;

                    status = IoCallDriver((*commonExtension).LowerDeviceObject, (*(*fdoExtension).PrivateFdoData).PowerProcessIrp);

                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tIoCallDriver returned %lx\n\0".as_ptr() as *const i8,
                        (*(*fdoExtension).PrivateFdoData).PowerProcessIrp,
                        status,
                    );

                    return STATUS_MORE_PROCESSING_REQUIRED;
                }
            }

            4 => {
                let mut ignoreError: BOOLEAN = TRUE;

                if SRB_STATUS((*srbHeader).SrbStatus) != (SRB_STATUS_SUCCESS) as u8 {
                    let mut delta100nsUnits: LONGLONG = 0;
                    (*PowerContext).RetryInterval = 0;
                    let retry = InterpretSenseInfoWithoutHistory(
                        (*commonExtension).DeviceObject,
                        Irp,
                        srbHeader as PSCSI_REQUEST_BLOCK,
                        IRP_MJ_SCSI as u8,
                        IRP_MJ_POWER as u32,
                        (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount - (*PowerContext).RetryCount,
                        &mut status,
                        &mut delta100nsUnits,
                    );
                    delta100nsUnits += (10 * 1000 * 1000) - 1;
                    delta100nsUnits /= 10 * 1000 * 1000;
                    (*PowerContext).RetryInterval = delta100nsUnits as ULONG;

                    if retry == TRUE {
                        let retry_count = (*PowerContext).RetryCount;
                        (*PowerContext).RetryCount -= 1;
                        if retry_count != 0 {
                            TracePrint(
                                TRACE_LEVEL_INFORMATION,
                                TRACE_FLAG_POWER,
                                b"(%p)\tRetrying failed request\n\0".as_ptr() as *const i8,
                                Irp,
                            );
                            (*PowerContext).PowerChangeState.PowerDown3 -= 1;
                            state -= 1;
                            SrbSetTimeOutValue(srbHeader as PVOID, DEFAULT_IO_TIMEOUT_VALUE);
                            RetryPowerRequest((*commonExtension).DeviceObject, Irp, PowerContext);
                            return STATUS_MORE_PROCESSING_REQUIRED;
                        }
                    }
                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tSTOP_UNIT not retried\n\0".as_ptr() as *const i8,
                        Irp,
                    );
                    (*(*fdoExtension).PrivateFdoData).MaxPowerOperationRetryCount = MAXIMUM_RETRIES;
                    (*PowerContext).RetryCount = MAXIMUM_RETRIES;
                }

                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_POWER,
                    b"(%p)\tPreviously sent stop unit\n\0".as_ptr() as *const i8,
                    Irp,
                );

                if !NT_SUCCESS(status) {
                    let senseBuffer = SrbGetSenseInfoBuffer(srbHeader as PVOID);
                    if ((*srbHeader).SrbStatus & (SRB_STATUS_AUTOSENSE_VALID) as u8) != 0
                        && senseBuffer != core::ptr::null_mut()
                    {
                        let mut senseKey: u8 = 0;
                        let mut additionalSenseCode: u8 = 0;
                        let mut additionalSenseCodeQualifier: u8 = 0;
                        let validSense = ScsiGetSenseKeyAndCodes(
                            senseBuffer,
                            (SrbGetSenseInfoBufferLength(srbHeader as PVOID)) as _,
                            SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED,
                            &mut senseKey,
                            &mut additionalSenseCode,
                            &mut additionalSenseCodeQualifier,
                        );
                        if validSense != 0 {
                            if senseKey == (SCSI_SENSE_NOT_READY) as u8
                                && additionalSenseCode == (SCSI_ADSENSE_LUN_NOT_READY) as u8
                                && additionalSenseCodeQualifier == (SCSI_SENSEQ_FORMAT_IN_PROGRESS) as u8
                            {
                                ignoreError = FALSE;
                                (*PowerContext).FinalStatus = STATUS_DEVICE_BUSY;
                                status = (*PowerContext).FinalStatus;
                            }
                        }
                    }
                }

                if NT_SUCCESS(status) || ignoreError != FALSE {
                    IoCopyCurrentIrpStackLocationToNext(OriginalIrp);
                    IoSetCompletionRoutine(
                        OriginalIrp,
                        Some(abi_cast!(ClasspPowerDownCompletion)),
                        PowerContext as PVOID,
                        TRUE,
                        TRUE,
                        TRUE,
                    );
                    status = PoCallDriver((*commonExtension).LowerDeviceObject, OriginalIrp);

                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tPoCallDriver returned %lx\n\0".as_ptr() as *const i8,
                        OriginalIrp,
                        status,
                    );

                    return STATUS_MORE_PROCESSING_REQUIRED;
                } else {
                    // fall through to case 5
                    (*PowerContext).PowerChangeState.PowerDown3 += 1;
                    state = 5;
                    continue;
                }
            }

            5 => {
                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_POWER,
                    b"(%p)\tPreviously sent power irp\n\0".as_ptr() as *const i8,
                    OriginalIrp,
                );

                if (*PowerContext).QueueLocked != 0 {
                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tUnlocking queue\n\0".as_ptr() as *const i8,
                        OriginalIrp,
                    );

                    if (*((*fdoExtension).AdapterDescriptor as *mut _STORAGE_ADAPTER_DESCRIPTOR)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
                        status = InitializeStorageRequestBlock(
                            srbHeader as PSTORAGE_REQUEST_BLOCK,
                            STORAGE_ADDRESS_TYPE_BTL8 as USHORT,
                            CLASS_SRBEX_NO_SRBEX_DATA_BUFFER_SIZE as ULONG,
                            0,
                            0, // SrbExType unused (NumSrbExData=0)
                        );
                        if NT_SUCCESS(status) {
                            (*(srbHeader as PSTORAGE_REQUEST_BLOCK)).SrbFunction = SRB_FUNCTION_UNLOCK_QUEUE;
                            (*PowerContext).Srb.Length = 0;
                        } else {
                            assert!(false, "FALSE");
                            srbHeader = core::ptr::addr_of_mut!((*PowerContext).Srb) as PSTORAGE_REQUEST_BLOCK_HEADER;
                            core::ptr::write_bytes(srbHeader as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                            (*srbHeader).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
                            (*srbHeader).Function = (SRB_FUNCTION_UNLOCK_QUEUE) as u8;
                        }
                    } else {
                        core::ptr::write_bytes(srbHeader as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());
                        (*srbHeader).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
                        (*srbHeader).Function = (SRB_FUNCTION_UNLOCK_QUEUE) as u8;
                    }

                    SrbSetOriginalRequest(srbHeader as PVOID, (*(*fdoExtension).PrivateFdoData).PowerProcessIrp as PIRP);
                    SrbAssignSrbFlags(srbHeader as PVOID, SRB_FLAGS_BYPASS_LOCKED_QUEUE | SRB_FLAGS_D3_PROCESSING);

                    (*nextStack).Parameters.Scsi.Srb = srbHeader as *mut DdkSrb;
                    (*nextStack).MajorFunction = (IRP_MJ_SCSI) as u8;

                    IoSetCompletionRoutine(
                        (*(*fdoExtension).PrivateFdoData).PowerProcessIrp,
                        Some(abi_cast!(ClasspPowerDownCompletion)),
                        PowerContext as PVOID,
                        TRUE,
                        TRUE,
                        TRUE,
                    );

                    status = IoCallDriver((*commonExtension).LowerDeviceObject, (*(*fdoExtension).PrivateFdoData).PowerProcessIrp);

                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tIoCallDriver returned %lx\n\0".as_ptr() as *const i8,
                        (*(*fdoExtension).PrivateFdoData).PowerProcessIrp,
                        status,
                    );

                    return STATUS_MORE_PROCESSING_REQUIRED;
                } else {
                    // fall through to 6
                    (*PowerContext).PowerChangeState.PowerDown3 += 1;
                    state = 6;
                    continue;
                }
            }

            6 => {
                if (*PowerContext).QueueLocked == FALSE {
                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tFall through (queue not locked)\n\0".as_ptr() as *const i8,
                        OriginalIrp,
                    );
                } else {
                    TracePrint(
                        TRACE_LEVEL_INFORMATION,
                        TRACE_FLAG_POWER,
                        b"(%p)\tPreviously unlocked queue\n\0".as_ptr() as *const i8,
                        OriginalIrp,
                    );
                    ntkrnl_sys::ASSERT!(NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status), "NT_SUCCESS(Irp->IoStatus.__bindgen_anon_1.Status)");
                    ntkrnl_sys::ASSERT!((*srbHeader).SrbStatus == (SRB_STATUS_SUCCESS) as u8, "srbHeader->SrbStatus == SRB_STATUS_SUCCESS");
                    if NT_SUCCESS((*Irp).IoStatus.__bindgen_anon_1.Status) {
                        (*PowerContext).QueueLocked = FALSE;
                    }
                }

                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_POWER,
                    b"(%p)\tFreeing srb and completing\n\0".as_ptr() as *const i8,
                    OriginalIrp,
                );
                status = (*PowerContext).FinalStatus;
                (*OriginalIrp).IoStatus.__bindgen_anon_1.Status = status;
                (*OriginalIrp).IoStatus.Information = 0;

                if NT_SUCCESS(status) {
                    (*fdoExtension).DevicePowerState =(*currentStack).Parameters.Power.State.DeviceState;
                }

                TracePrint(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_POWER,
                    b"(%p)\tStarting next power irp\n\0".as_ptr() as *const i8,
                    OriginalIrp,
                );

                ClassReleaseRemoveLock((*PowerContext).DeviceObject, OriginalIrp);
                (*PowerContext).InUse = FALSE;
                PoStartNextPowerIrp(OriginalIrp);
                (*fdoExtension).PowerDownInProgress =FALSE;

                if Irp == (*(*fdoExtension).PrivateFdoData).PowerProcessIrp {
                    ClassCompleteRequest((*commonExtension).DeviceObject, OriginalIrp, (IO_NO_INCREMENT) as _);
                    return STATUS_MORE_PROCESSING_REQUIRED;
                }

                return status;
            }

            _ => return STATUS_MORE_PROCESSING_REQUIRED,
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClasspPowerHandler(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Options: CLASS_POWER_OPTIONS,
) -> NTSTATUS {
    let common_extension = (*DeviceObject).DeviceExtension as *mut _COMMON_DEVICE_EXTENSION;
    let lower_device = (*common_extension).LowerDeviceObject;
    let irp_stack = IoGetCurrentIrpStackLocation(Irp);
    let fdo_extension = (*DeviceObject).DeviceExtension as *mut _FUNCTIONAL_DEVICE_EXTENSION;
    let fdo_data = (*fdo_extension).PrivateFdoData;
    let mut next_irp_stack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut context: *mut _CLASS_POWER_CONTEXT = core::ptr::null_mut();
    let mut srb_header: *mut _STORAGE_REQUEST_BLOCK_HEADER = core::ptr::null_mut();
    let mut srb_flags: ULONG = 0;
    let mut status: NTSTATUS = 0;

    TracePrint(
        4,
        TRACE_FLAG_POWER,
        b"ClasspPowerHandler: Power irp %p to %s %p\n\0".as_ptr() as *const i8,
        Irp,
        if (*common_extension).__bindgen_anon_1.IsFdo() != 0 {
            b"fdo\0".as_ptr() as *const i8
        } else {
            b"pdo\0".as_ptr() as *const i8
        },
        DeviceObject,
    );

    if (*common_extension).__bindgen_anon_1.IsFdo() == 0 {
        TracePrint(
            2,
            TRACE_FLAG_POWER,
            b"ClasspPowerHandler: Called for PDO %p???\n\0".as_ptr() as *const i8,
            DeviceObject,
        );
        // NT_ASSERT(!"PDO using ClasspPowerHandler")
        if !(b"PDO using ClasspPowerHandler\0".as_ptr() as *const i8).is_null() {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/power.c\0"
                    .as_ptr() as *const i8,
                1585,
                b"!\"PDO using ClasspPowerHandler\"\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        ClassReleaseRemoveLock(DeviceObject, Irp);
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
        PoStartNextPowerIrp(Irp);
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as i8) as _);
        return STATUS_NOT_SUPPORTED;
    }

    // IRP_MN_SET_POWER special handling
    if (*irp_stack).MinorFunction as u32 == IRP_MN_SET_POWER as u32 {
        TracePrint(
            4,
            TRACE_FLAG_POWER,
            b"(%p)\tIRP_MN_SET_POWER\n\0".as_ptr() as *const i8,
            Irp,
        );
        TracePrint(
            4,
            TRACE_FLAG_POWER,
            b"(%p)\tSetting %s state to %d\n\0".as_ptr() as *const i8,
            Irp,
            if (*irp_stack).Parameters.Power.Type as u32 == SystemPowerState as u32 {
                b"System\0".as_ptr() as *const i8
            } else {
                b"Device\0".as_ptr() as *const i8
            },
            (*irp_stack).Parameters.Power.State.SystemState as u32,
        );

        let shutdown_type = (*irp_stack).Parameters.Power.ShutdownType;
        let mut do_hotplug = false;
        if shutdown_type as u32 == PowerActionNone as u32 {
            if !(*fdo_extension).FunctionSupportInfo.is_null()
                && (*(*fdo_extension).FunctionSupportInfo)
                    .IdlePower
                    .NoVerifyDuringIdlePower() != 0
            {
                // skip hotplug logic
            } else {
                do_hotplug = true;
            }
        } else if shutdown_type as u32 == PowerActionSleep as u32
            || shutdown_type as u32 == PowerActionHibernate as u32
        {
            do_hotplug = true;
        }

        if do_hotplug {
            if (*fdo_data).HotplugInfo.MediaRemovable != 0
                || (*fdo_data).HotplugInfo.MediaHotplug != 0
            {
                (*DeviceObject).Flags |= 0x2; // DO_VERIFY_VOLUME
                InterlockedIncrement(
                    &raw mut (*fdo_extension).MediaChangeCount as *mut i32,
                );
            }
        }
    } else {
        TracePrint(
            4,
            TRACE_FLAG_POWER,
            b"(%p)\tIrp minor code = %#x\n\0".as_ptr() as *const i8,
            Irp,
            (*irp_stack).MinorFunction as u32,
        );
    }

    // Begin the decision tree that leads to cleanup or main logic
    'cleanup: {
        // Not a device power set IRP or different type -> send down
        if (*irp_stack).Parameters.Power.Type as u32 != DevicePowerState as u32
            || (*irp_stack).MinorFunction as u32 != IRP_MN_SET_POWER as u32
        {
            TracePrint(
                4,
                TRACE_FLAG_POWER,
                b"(%p)\tSending to lower device\n\0".as_ptr() as *const i8,
                Irp,
            );
            break 'cleanup;
        }

        // Already in the same device state
        if (*irp_stack).Parameters.Power.State.DeviceState as u32
            == (*fdo_extension).DevicePowerState as u32
        {
            TracePrint(
                4,
                TRACE_FLAG_POWER,
                b"(%p)\tAlready in device state %x\n\0".as_ptr() as *const i8,
                Irp,
                (*fdo_extension).DevicePowerState as u32,
            );
            break 'cleanup;
        }

        // Powering down from non-D0 and already non-D0
        if (*irp_stack).Parameters.Power.State.DeviceState as u32 != PowerDeviceD0 as u32
            && (*fdo_extension).DevicePowerState as u32 != PowerDeviceD0 as u32
        {
            TracePrint(
                4,
                TRACE_FLAG_POWER,
                b"(%p)\tAlready powered down to %x???\n\0".as_ptr() as *const i8,
                Irp,
                (*fdo_extension).DevicePowerState as u32,
            );
            (*fdo_extension).DevicePowerState =
                (*irp_stack).Parameters.Power.State.DeviceState as DEVICE_POWER_STATE;
            break 'cleanup;
        }

        // Not handling spin-up and powering up
        if Options.HandleSpinUp() == 0
            && (*irp_stack).Parameters.Power.State.DeviceState as u32 == PowerDeviceD0 as u32
        {
            TracePrint(
                4,
                TRACE_FLAG_POWER,
                b"(%p)\tNot handling spinup to state %x\n\0".as_ptr() as *const i8,
                Irp,
                (*fdo_extension).DevicePowerState as u32,
            );
            (*fdo_extension).DevicePowerState =
                (*irp_stack).Parameters.Power.State.DeviceState as DEVICE_POWER_STATE;
            break 'cleanup;
        }

        // Not handling spin-down and powering down
        if Options.HandleSpinDown() == 0
            && (*irp_stack).Parameters.Power.State.DeviceState as u32 != PowerDeviceD0 as u32
        {
            TracePrint(
                4,
                TRACE_FLAG_POWER,
                b"(%p)\tNot handling spindown to state %x\n\0".as_ptr() as *const i8,
                Irp,
                (*fdo_extension).DevicePowerState as u32,
            );
            (*fdo_extension).DevicePowerState =
                (*irp_stack).Parameters.Power.State.DeviceState as DEVICE_POWER_STATE;
            break 'cleanup;
        }

        // --- Real power handling starts here ---
        let power_process_irp = (*fdo_data).PowerProcessIrp;
        IoReuseIrp(power_process_irp, STATUS_SUCCESS);
        IoSetNextIrpStackLocation(power_process_irp);
        next_irp_stack = IoGetNextIrpStackLocation(power_process_irp);

        context = &raw mut (*fdo_extension).PowerContext;
        // NT_ASSERT(context->InUse == FALSE)
        if (*context).InUse != 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/power.c\0"
                    .as_ptr() as *const i8,
                1731,
                b"context->InUse == FALSE\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        core::ptr::write_bytes(
            context as *mut u8,
            0,
            core::mem::size_of::<CLASS_POWER_CONTEXT>(),
        );
        (*context).InUse = 1u8;

        if (*((*fdo_extension).AdapterDescriptor as *mut _STORAGE_ADAPTER_DESCRIPTOR)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK
        {
            srb_header = &raw mut (*fdo_data).PowerSrb.SrbEx as *mut _STORAGE_REQUEST_BLOCK_HEADER;
            status = InitializeStorageRequestBlock(
                srb_header as PSTORAGE_REQUEST_BLOCK,
                STORAGE_ADDRESS_TYPE_BTL8 as USHORT,
                CLASS_SRBEX_NO_SRBEX_DATA_BUFFER_SIZE as ULONG,
                0,
                0, // SrbExType unused (NumSrbExData=0)
            );
            if NT_SUCCESS(status) {
                (*(srb_header as PSTORAGE_REQUEST_BLOCK)).SrbFunction =
                    SRB_FUNCTION_LOCK_QUEUE as ULONG;
            } else {
                // NT_ASSERT(FALSE)
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                    b"drivers/storage/class/classpnp/power.c\0"
                        .as_ptr() as *const i8,
                    1752,
                    b"FALSE\0".as_ptr() as *const i8,
                );
                __int2c();
                srb_header = &raw mut (*context).Srb as *mut _STORAGE_REQUEST_BLOCK_HEADER;
                (*srb_header).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
                (*srb_header).Function = SRB_FUNCTION_LOCK_QUEUE as UCHAR;
            }
        } else {
            srb_header = &raw mut (*context).Srb as *mut _STORAGE_REQUEST_BLOCK_HEADER;
            (*srb_header).Length = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as USHORT;
            (*srb_header).Function = SRB_FUNCTION_LOCK_QUEUE as UCHAR;
        }

        (*next_irp_stack).Parameters.Scsi.Srb = srb_header as *mut DdkSrb;
        (*next_irp_stack).MajorFunction = IRP_MJ_SCSI as UCHAR;

        (*context).FinalStatus = STATUS_SUCCESS;

        SrbSetOriginalRequest(srb_header as PVOID, (power_process_irp as PVOID).cast());
        SrbSetSrbFlags(
            srb_header as PVOID,
            (SRB_FLAGS_BYPASS_LOCKED_QUEUE | SRB_FLAGS_NO_QUEUE_FREEZE) as ULONG,
        );

        (*fdo_data).MaxPowerOperationRetryCount = MAXIMUM_RETRIES as ULONG;
        (*context).RetryCount = MAXIMUM_RETRIES as ULONG;

        (*context).Options = Options;
        (*context).DeviceObject = DeviceObject;
        (*context).Irp = Irp;

        if (*irp_stack).Parameters.Power.State.DeviceState as u32 == PowerDeviceD0 as u32 {
            // NT_ASSERT(Options.HandleSpinUp)
            if (*context).Options.HandleSpinUp() == 0 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                    b"drivers/storage/class/classpnp/power.c\0"
                        .as_ptr() as *const i8,
                    1779,
                    b"Options.HandleSpinUp\0".as_ptr() as *const i8,
                );
                __int2c();
            }
            TracePrint(
                4,
                TRACE_FLAG_POWER,
                b"(%p)\tpower up - locking queue\n\0".as_ptr() as *const i8,
                Irp,
            );
            (*context).Options.set_PowerDown(0u32);
            (*context).PowerChangeState.PowerUp = CLASS_POWER_UP_STATE_PowerUpDeviceInitial;
            (*context).CompletionRoutine = Some(abi_cast!(ClasspPowerUpCompletion));
        } else {
            // NT_ASSERT(Options.HandleSpinDown)
            if (*context).Options.HandleSpinDown() == 0 {
                DbgPrint(
                    b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                    b"drivers/storage/class/classpnp/power.c\0"
                        .as_ptr() as *const i8,
                    1795,
                    b"Options.HandleSpinDown\0".as_ptr() as *const i8,
                );
                __int2c();
            }
            (*fdo_extension).PowerDownInProgress = 1u8;

            TracePrint(
                4,
                TRACE_FLAG_POWER,
                b"(%p)\tPowering down - locking queue\n\0".as_ptr() as *const i8,
                Irp,
            );
            if (*fdo_data).TickTimerEnabled != 0 {
                ClasspDisableTimer(fdo_extension);
            }
            PoSetPowerState(
                DeviceObject,
                (*irp_stack).Parameters.Power.Type,
                (*irp_stack).Parameters.Power.State,
            );
            (*context).Options.set_PowerDown(1u32);
            (*context).PowerChangeState.PowerDown3 = CLASS_POWER_DOWN_STATE3_PowerDownDeviceInitial3;
            (*context).CompletionRoutine = Some(abi_cast!(ClasspPowerDownCompletion));
        }

        srb_flags = SrbGetSrbFlags(srb_header as PVOID);
        // NT_ASSERT(!TEST_FLAG(srbFlags, SRB_FLAGS_FREE_SENSE_BUFFER))
        if srb_flags & 0x400 != 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/power.c\0"
                    .as_ptr() as *const i8,
                1823,
                b"!TEST_FLAG(srbFlags, SRB_FLAGS_FREE_SENSE_BUFFER)\0".as_ptr() as *const i8,
            );
            __int2c();
        }
        // NT_ASSERT(!TEST_FLAG(srbFlags, SRB_FLAGS_PORT_DRIVER_ALLOCSENSE))
        if srb_flags & 0x200000 != 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/power.c\0"
                    .as_ptr() as *const i8,
                1824,
                b"!TEST_FLAG(srbFlags, SRB_FLAGS_PORT_DRIVER_ALLOCSENSE)\0".as_ptr() as *const i8,
            );
            __int2c();
        }

        IoMarkIrpPending(Irp);

        if (*context).Options.LockQueue() != 0 {
            IoSetCompletionRoutine(
                power_process_irp,
                (*context).CompletionRoutine,
                context as PVOID,
                1u8,
                1u8,
                1u8,
            );
            IoCallDriver(lower_device, power_process_irp);
        } else {
            (*context)
                .CompletionRoutine
                .expect("non-null function pointer")(
                DeviceObject,
                power_process_irp,
                context as PVOID,
            );
        }

        return STATUS_PENDING;
    }

    // Cleanup path: forward the original IRP
    ClassReleaseRemoveLock(DeviceObject, Irp);
    TracePrint(
        4,
        TRACE_FLAG_POWER,
        b"(%p)\tStarting next power irp\n\0".as_ptr() as *const i8,
        Irp,
    );
    IoCopyCurrentIrpStackLocationToNext(Irp);
    IoSetCompletionRoutine(
        Irp,
        Some(abi_cast!(ClasspStartNextPowerIrpCompletion)),
        core::ptr::null_mut::<c_void>(),
        1u8,
        1u8,
        1u8,
    );
    PoCallDriver(lower_device, Irp)
}

#[no_mangle]
pub unsafe extern "system" fn ClassMinimalPowerHandler(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let commonExtension: PCOMMON_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut status: NTSTATUS;

    ClassReleaseRemoveLock(DeviceObject, Irp);
    PoStartNextPowerIrp(Irp);

    let minor = (*irpStack).MinorFunction as u32;
    if minor == IRP_MN_SET_POWER as u32 {
        let shutdown_type = (*irpStack).Parameters.Power.ShutdownType;
        if shutdown_type == PowerActionNone || shutdown_type == PowerActionSleep || shutdown_type == PowerActionHibernate {
            if TEST_FLAG((*DeviceObject).Characteristics, FILE_REMOVABLE_MEDIA) != 0 {
                let vpb = ClassGetVpb(DeviceObject);
                if !vpb.is_null() && ((*vpb).Flags & (VPB_MOUNTED) as u16) != 0 {
                    (*DeviceObject).Flags |= DO_VERIFY_VOLUME as u32;
                }
            }
        }
        // Fall through to IRP_MN_QUERY_POWER handling
        if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            (*Irp).IoStatus.Information = 0;
        }
    } else if minor == IRP_MN_QUERY_POWER as u32 {
        if (*commonExtension).__bindgen_anon_1.IsFdo() == 0 {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
            (*Irp).IoStatus.Information = 0;
        }
    }

    if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
        IoCopyCurrentIrpStackLocationToNext(Irp);
        status = PoCallDriver((*commonExtension).LowerDeviceObject, Irp);
    } else {
        status = (*Irp).IoStatus.__bindgen_anon_1.Status;
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT) as _);
    }

    status
}
#[no_mangle]
pub unsafe extern "system" fn ClassSpinDownPowerHandler(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut options: CLASS_POWER_OPTIONS = core::mem::zeroed();

    if (*fdoExtension).ScanForSpecialFlags & 0x1u32 == 0 {
        options.set_HandleSpinDown(1u32);
    }
    if (*fdoExtension).ScanForSpecialFlags & 0x2u32 == 0 {
        options.set_HandleSpinUp(1u32);
    }
    if (*fdoExtension).ScanForSpecialFlags & 0x8u32 == 0 {
        options.set_LockQueue(1u32);
    }

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_POWER,
        b"ClasspPowerHandler: Devobj %p\n\t%shandling spin down\n\t%shandling spin up\n\t%slocking queue\n\0"
            .as_ptr() as PCHAR,
        DeviceObject,
        if options.HandleSpinDown() != 0 {
            b"\0".as_ptr() as PCHAR
        } else {
            b"not \0".as_ptr() as PCHAR
        },
        if options.HandleSpinUp() != 0 {
            b"\0".as_ptr() as PCHAR
        } else {
            b"not \0".as_ptr() as PCHAR
        },
        if options.LockQueue() != 0 {
            b"\0".as_ptr() as PCHAR
        } else {
            b"not \0".as_ptr() as PCHAR
        },
    );

    ClasspPowerHandler(DeviceObject, Irp, options)
}
#[no_mangle]
pub unsafe extern "system" fn ClassStopUnitPowerHandler(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION;

    TracePrint(
        TRACE_LEVEL_ERROR,
        TRACE_FLAG_POWER,
        b"ClassStopUnitPowerHandler - Devobj %p using outdated call\nDrivers should set the following flags in ScanForSpecialFlags in the FDO extension:\n\tCLASS_SPECIAL_DISABLE_SPIN_UP\n\tCLASS_SPECIAL_NO_QUEUE_LOCK\nThis will provide equivalent functionality if the power routine is then set to ClassSpinDownPowerHandler\n\n\0"
            .as_ptr() as PCHAR,
        DeviceObject,
    );

    fdoExtension = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;

    (*fdoExtension).ScanForSpecialFlags |= 0x2u32;
    (*fdoExtension).ScanForSpecialFlags |= 0x8u32;

    return ClassSpinDownPowerHandler(DeviceObject, Irp);
}
#[no_mangle]
pub unsafe extern "C" fn RetryPowerRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PCLASS_POWER_CONTEXT,
) {
    let next_irp_stack: PIO_STACK_LOCATION = IoGetNextIrpStackLocation(Irp);
    let fdo_extension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*(*Context).DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut srb: PSTORAGE_REQUEST_BLOCK_HEADER = core::ptr::null_mut::<STORAGE_REQUEST_BLOCK_HEADER>();
    let mut due_time: LONGLONG = 0;
    let srb_flags: ULONG;
    let mut srb_function: ULONG;

    TracePrint(
        (TRACE_LEVEL_INFORMATION as i32) as u32,
        TRACE_FLAG_POWER,
        b"(%p)\tDelaying retry by queueing DPC\n\0".as_ptr() as PCCHAR,
        Irp,
    );

    if (*((*fdo_extension).AdapterDescriptor as *mut _STORAGE_ADAPTER_DESCRIPTOR)).SrbType as u32 == SRB_TYPE_STORAGE_REQUEST_BLOCK {
        srb = &raw mut (*(*fdo_extension).PrivateFdoData).PowerSrb.SrbEx
            as PSTORAGE_REQUEST_BLOCK_HEADER;
        if (*Context).Srb.Length as usize == core::mem::size_of::<SCSI_REQUEST_BLOCK>() {
            srb = &raw mut (*Context).Srb as PSTORAGE_REQUEST_BLOCK_HEADER;
            srb_function = (*srb).Function as ULONG;
        } else {
            srb_function = (*(srb as PSTORAGE_REQUEST_BLOCK)).SrbFunction;
        }
    } else {
        srb = &raw mut (*Context).Srb as PSTORAGE_REQUEST_BLOCK_HEADER;
        srb_function = (*srb).Function as ULONG;
    }

    let cond1 = (*Context).DeviceObject == DeviceObject;
    if !cond1 {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/classpnp/power.c\0".as_ptr()
                as *const i8,
            2131,
            b"Context->DeviceObject == DeviceObject\0".as_ptr() as *const i8,
        );
        __int2c();
    }

    srb_flags = SrbGetSrbFlags(srb as PVOID);
    if srb_flags & SRB_FLAGS_FREE_SENSE_BUFFER != 0 {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/classpnp/power.c\0".as_ptr()
                as *const i8,
            2133,
            b"!TEST_FLAG(srbFlags, SRB_FLAGS_FREE_SENSE_BUFFER)\0".as_ptr() as *const i8,
        );
        __int2c();
    }
    if srb_flags & SRB_FLAGS_PORT_DRIVER_ALLOCSENSE != 0 {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/classpnp/power.c\0".as_ptr()
                as *const i8,
            2134,
            b"!TEST_FLAG(srbFlags, SRB_FLAGS_PORT_DRIVER_ALLOCSENSE)\0".as_ptr() as *const i8,
        );
        __int2c();
    }

    if (*Context).RetryInterval == 0 {
        TracePrint(
            (TRACE_LEVEL_INFORMATION as i32) as u32,
            TRACE_FLAG_POWER,
            b"(%p)\tDelaying minimum time (.2 sec)\n\0".as_ptr() as PCCHAR,
            Irp,
        );
        due_time = 1000000i64 * 2;
    } else {
        TracePrint(
            (TRACE_LEVEL_INFORMATION as i32) as u32,
            TRACE_FLAG_POWER,
            b"(%p)\tDelaying %x seconds\n\0".as_ptr() as PCCHAR,
            Irp,
            (*Context).RetryInterval,
        );
        due_time = 1000000i64 * 10 * (*Context).RetryInterval as i64;
    }

    (*Context).RetryInterval = 0;

    SrbSetDataTransferLength(srb as PVOID, 0);
    (*srb).SrbStatus = 0u8;
    if srb_function == SRB_FUNCTION_EXECUTE_SCSI {
        SrbSetScsiStatus(srb as PVOID, 0u8);
    }

    (*next_irp_stack).MajorFunction = IRP_MJ_SCSI as u8;
    (*next_irp_stack).Parameters.Scsi.Srb = srb as *mut DdkSrb;

    IoSetCompletionRoutine(
        Irp,
        (*Context).CompletionRoutine,
        Context as PVOID,
        1u8,
        1u8,
        1u8,
    );

    ClassRetryRequest(DeviceObject, Irp, due_time);
}
#[no_mangle]
pub unsafe extern "system" fn ClasspStartNextPowerIrpCompletion(
    _device_object: PDEVICE_OBJECT,
    irp: PIRP,
    context: PVOID,
) -> NTSTATUS {
    let power_context = context as PCLASS_POWER_CONTEXT;

    if (*irp).PendingReturned != 0 {
        IoMarkIrpPending(irp);
    }

    if !power_context.is_null() {
        (*power_context).InUse = 0u8;
    }

    PoStartNextPowerIrp(irp);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ClasspDeviceLockFailurePowerIrpCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let PowerContext = Context as PCLASS_POWER_CONTEXT;
    let commonExtension = (*(*PowerContext).DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let fdoExtension = (*(*PowerContext).DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let currentStack = IoGetCurrentIrpStackLocation(Irp);
    let mut FailurePredictionEnabled: BOOLEAN = 0;

    (*fdoExtension).DevicePowerState = (*currentStack).Parameters.Power.State.DeviceState;

    if (*fdoExtension).DevicePowerState as u32 == PowerDeviceD0 as u32 {
        if !(*fdoExtension).FailurePredictionInfo.is_null()
            && (*(*fdoExtension).FailurePredictionInfo).Method as u32 != FailurePredictionNone as u32
        {
            FailurePredictionEnabled = 1;
        }

        if (*(*commonExtension).DriverExtension).InitData.ClassTick.is_some()
            || (!(*fdoExtension).MediaChangeDetectionInfo.is_null()
                && !(*fdoExtension).FunctionSupportInfo.is_null()
                && (*(*fdoExtension).FunctionSupportInfo).AsynchronousNotificationSupported as i32 == 0)
            || FailurePredictionEnabled != 0
        {
            if FailurePredictionEnabled != 0
                && ClasspFailurePredictionPeriodMissed(fdoExtension) != 0
            {
                (*(*fdoExtension).FailurePredictionInfo).CountDown = 1;
            }
            ClasspEnableTimer(fdoExtension);
        }
    }

    PoSetPowerState(
        (*PowerContext).DeviceObject,
        (*currentStack).Parameters.Power.Type,
        (*currentStack).Parameters.Power.State,
    );

    (*PowerContext).InUse = 0;
    ClassReleaseRemoveLock((*commonExtension).DeviceObject, Irp);

    if (*Irp).PendingReturned != 0 {
        IoMarkIrpPending(Irp);
    }

    PoStartNextPowerIrp(Irp);
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ClasspSendEnableIdlePowerIoctl(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let mut idlePower: STORAGE_IDLE_POWER = core::mem::zeroed();
    let mut ioStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let commonExtension: PCOMMON_DEVICE_EXTENSION = (&raw mut (*fdoExtension).__bindgen_anon_1.CommonExtension).cast();

    idlePower.Version = 1u32;
    idlePower.Size = core::mem::size_of::<STORAGE_IDLE_POWER>() as u32;
    idlePower.set_WakeCapableHint((*(*fdoExtension).FunctionSupportInfo).IdlePower.DeviceWakeable() as u32);
    idlePower.set_D3ColdSupported((*(*fdoExtension).FunctionSupportInfo).IdlePower.D3ColdSupported() as u32);
    idlePower.D3IdleTimeout = (*(*fdoExtension).FunctionSupportInfo).IdlePower.D3IdleTimeout;

    ClassSendDeviceIoControlSynchronous(
        IOCTL_STORAGE_ENABLE_IDLE_POWER,
        (*commonExtension).LowerDeviceObject,
        &raw mut idlePower as *mut c_void,
        core::mem::size_of::<STORAGE_IDLE_POWER>() as u32,
        0u32,
        0u8, // FALSE
        &raw mut ioStatus,
    );

    let status = ioStatus.__bindgen_anon_1.Status;

    TracePrint(
        TRACE_LEVEL_INFORMATION,
        TRACE_FLAG_POWER,
        b"ClasspSendEnableIdlePowerIoctl: Port driver returned status (%x) for FDO (%p)\n\tWakeCapableHint: %u\n\tD3ColdSupported: %u\n\tD3IdleTimeout: %u (ms)\0".as_ptr() as PCCHAR,
        status,
        DeviceObject,
        idlePower.WakeCapableHint() as i32,
        idlePower.D3ColdSupported() as i32,
        idlePower.D3IdleTimeout,
    );

    status
}
#[no_mangle]
pub unsafe extern "system" fn ClasspPowerSettingCallback(
    SettingGuid: LPCGUID,
    Value: PVOID,
    ValueLength: ULONG,
    Context: PVOID,
) -> NTSTATUS {
    let mut fdoEntry: PIDLE_POWER_FDO_LIST_ENTRY = core::ptr::null_mut();
    let removeLockTag: PIRP = &ClasspPowerSettingCallback as *const _ as PIRP;

    if KeGetCurrentIrql() as ::core::ffi::c_int > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as ::core::ffi::c_int,
        );
        if 0 as ::core::ffi::c_int == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
                b"drivers/storage/class/classpnp/power.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                2436u32 as ULONG,
                b"0\0".as_ptr() as *const ::core::ffi::c_char,
            );
            __int2c();
        }
    }

    if IsEqualGUID(SettingGuid, &raw const GUID_DISK_IDLE_TIMEOUT) {
        if ValueLength as usize != core::mem::size_of::<ULONG>() || Value.is_null() {
            return STATUS_INVALID_PARAMETER;
        }
        DiskIdleTimeoutInMS = *(Value as PULONG);
        KeAcquireGuardedMutex((&raw mut IdlePowerFDOListMutex).cast());
        fdoEntry = IdlePowerFDOList.Flink as PIDLE_POWER_FDO_LIST_ENTRY;
        while fdoEntry as PLIST_ENTRY != &raw mut IdlePowerFDOList {
            let isRemoved = ClassAcquireRemoveLock((*fdoEntry).Fdo, removeLockTag);
            if isRemoved == 0 {
                let fdoExtension =
                    (*(*fdoEntry).Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
                let support_info = (*fdoExtension).FunctionSupportInfo;
                if (*support_info).IdlePower.D3IdleTimeoutOverridden() as u8 == 0 {
                    (*support_info).IdlePower.D3IdleTimeout = DiskIdleTimeoutInMS;
                    ClasspSendEnableIdlePowerIoctl((*fdoEntry).Fdo);
                }
            }
            ClassReleaseRemoveLock((*fdoEntry).Fdo, removeLockTag);
            fdoEntry = (*fdoEntry).ListEntry.Flink as PIDLE_POWER_FDO_LIST_ENTRY;
        }
        KeReleaseGuardedMutex((&raw mut IdlePowerFDOListMutex).cast());
    } else if IsEqualGUID(SettingGuid, &raw const GUID_CONSOLE_DISPLAY_STATE) {
        if ValueLength as usize == core::mem::size_of::<ULONG>() && !Value.is_null() {
            if *(Value as PULONG) == PowerMonitorOff as ULONG {
                ClasspScreenOff = TRUE as BOOLEAN;
            } else {
                ClasspScreenOff = FALSE as BOOLEAN;
            }
            KeAcquireGuardedMutex((&raw mut IdlePowerFDOListMutex).cast());
            fdoEntry = IdlePowerFDOList.Flink as PIDLE_POWER_FDO_LIST_ENTRY;
            while fdoEntry as PLIST_ENTRY != &raw mut IdlePowerFDOList {
                let isRemoved = ClassAcquireRemoveLock((*fdoEntry).Fdo, removeLockTag);
                if isRemoved == 0 {
                    let fdoExtension =
                        (*(*fdoEntry).Fdo).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
                    if ClasspScreenOff as ::core::ffi::c_int == FALSE as ::core::ffi::c_int {
                        if (*fdoExtension).DevicePowerState as u32 != PowerDeviceD0 as u32
                            && !(*fdoExtension).MediaChangeDetectionInfo.is_null()
                            && (*((*fdoExtension).FunctionSupportInfo))
                                .AsynchronousNotificationSupported as ::core::ffi::c_int
                                == FALSE as ::core::ffi::c_int
                        {
                            ClassCheckMediaState(fdoExtension);
                        }
                        if !(*fdoExtension).FailurePredictionInfo.is_null()
                            && (*((*fdoExtension).FailurePredictionInfo)).Method as u32
                                != FailurePredictionNone as u32
                        {
                            if ClasspFailurePredictionPeriodMissed(fdoExtension) != 0 {
                                (*((*fdoExtension).FailurePredictionInfo)).CountDown = 1u32;
                            }
                        }
                    }
                    ClasspUpdateTimerNoWakeTolerance(fdoExtension);
                }
                ClassReleaseRemoveLock((*fdoEntry).Fdo, removeLockTag);
                fdoEntry = (*fdoEntry).ListEntry.Flink as PIDLE_POWER_FDO_LIST_ENTRY;
            }
            KeReleaseGuardedMutex((&raw mut IdlePowerFDOListMutex).cast());
        }
    }
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ClasspEnableIdlePower(DeviceObject: PDEVICE_OBJECT) -> NTSTATUS {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut d3ColdDisabledByUser: ULONG = 0u32;
    let fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFUNCTIONAL_DEVICE_EXTENSION;
    let mut idleTimeoutOverrideInSeconds: ULONG = 0u32;

    // Assert that the function is called only once (IdlePowerEnabled is FALSE)
    if (*(*fdoExtension).FunctionSupportInfo).IdlePower.IdlePowerEnabled() != 0 {
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0".as_ptr() as PCSTR,
            b"drivers/storage/class/classpnp/power.c\0"
                .as_ptr() as *const ::core::ffi::c_char,
            2582i32,
            b"fdoExtension->FunctionSupportInfo->IdlePower.IdlePowerEnabled == FALSE\0"
                .as_ptr() as *const ::core::ffi::c_char,
        );
        __int2c();
    }

    ClassGetDeviceParameter(
        fdoExtension,
        CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
        CLASSP_REG_DISABLE_D3COLD.as_ptr() as PWSTR,
        &raw mut d3ColdDisabledByUser,
    );

    if d3ColdDisabledByUser != 0
        || (*(*fdoExtension).PrivateFdoData).HotplugInfo.DeviceHotplug as i32 != 0
    {
        (*(*fdoExtension)
            .FunctionSupportInfo).IdlePower
            .set_D3ColdSupported(0u32);
    }

    ClassGetDeviceParameter(
        fdoExtension,
        CLASSP_REG_SUBKEY_NAME.as_ptr() as PWSTR,
        CLASSP_REG_IDLE_TIMEOUT_IN_SECONDS.as_ptr() as PWSTR,
        &raw mut idleTimeoutOverrideInSeconds,
    );

    if idleTimeoutOverrideInSeconds != 0 {
        (*(*fdoExtension)
            .FunctionSupportInfo).IdlePower
            .D3IdleTimeout = (idleTimeoutOverrideInSeconds.wrapping_mul(1000u32)) as ULONG;
        (*(*fdoExtension)
            .FunctionSupportInfo).IdlePower
            .set_D3IdleTimeoutOverridden(1u32);
    } else {
        (*(*fdoExtension)
            .FunctionSupportInfo).IdlePower
            .D3IdleTimeout = DiskIdleTimeoutInMS;
    }

    (*(*fdoExtension)
        .FunctionSupportInfo).IdlePower
        .set_DeviceWakeable(0u32);

    status = ClasspSendEnableIdlePowerIoctl(DeviceObject);

    if status >= 0 {
        let fdoEntry: PIDLE_POWER_FDO_LIST_ENTRY = ExAllocatePoolWithTag(
            NonPagedPoolNx,
            ::core::mem::size_of::<IDLE_POWER_FDO_LIST_ENTRY>() as SIZE_T,
            CLASS_TAG_POWER as ULONG,
        ) as PIDLE_POWER_FDO_LIST_ENTRY;

        if !fdoEntry.is_null() {
            (*(*fdoExtension)
                .FunctionSupportInfo).IdlePower
                .set_IdlePowerEnabled(1u32);

            (*fdoEntry).Fdo = DeviceObject;

            KeAcquireGuardedMutex((&raw mut IdlePowerFDOListMutex).cast());
            InsertHeadList(&raw mut IdlePowerFDOList, &raw mut (*fdoEntry).ListEntry);
            KeReleaseGuardedMutex((&raw mut IdlePowerFDOListMutex).cast());

            if PowerSettingNotificationHandle.is_null() {
                PoRegisterPowerSettingCallback(
                    DeviceObject,
                    &raw const GUID_DISK_IDLE_TIMEOUT as LPCGUID,
                    Some(abi_cast!(ClasspPowerSettingCallback)),
                    core::ptr::null_mut(),
                    &raw mut PowerSettingNotificationHandle,
                );
            }
        } else {
            (*(*fdoExtension)
                .FunctionSupportInfo).IdlePower
                .set_IdlePowerEnabled(0u32);
            status = STATUS_UNSUCCESSFUL;
        }
    }

    status
}


// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
extern "system" {
    fn PoRegisterPowerSettingCallback(
        DeviceObject: PDEVICE_OBJECT,
        SettingGuid: LPCGUID,
        Callback: ntkrnl_sys::ddk::PPOWER_SETTING_CALLBACK,
        Context: PVOID,
        Handle: *mut PVOID,
    ) -> NTSTATUS;
}
extern "system" {
    fn ScsiGetSenseKeyAndCodes(SenseInfoBuffer: PVOID, SenseInfoBufferLength: u32, Options: ULONG, SenseKey: *mut UCHAR, AdditionalSenseCode: *mut UCHAR, AdditionalSenseCodeQualifier: *mut UCHAR) -> BOOLEAN;
    fn GET_FDO_EXTENSON_SENSE_DATA_LENGTH(fdoExtension: PFUNCTIONAL_DEVICE_EXTENSION) -> UCHAR;
}
