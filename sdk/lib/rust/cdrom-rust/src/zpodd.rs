use crate::types::*;
use kmdf_sys::*;
use ntkrnl_sys::ddk::{IRP, PIRP, IO_STACK_LOCATION, PIO_STACK_LOCATION, GUID, LPGUID, LPCGUID, DEVICE_OBJECT, PDEVICE_OBJECT, MDL, PMDL};
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::base::STATUS_SUCCESS;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{D3COLD_SUPPORT_INTERFACE, DEVICE_WAKE_DEPTH, _DEVICE_WAKE_DEPTH_DeviceWakeDepthD3cold as DeviceWakeDepthD3cold, _DEVICE_WAKE_DEPTH_DeviceWakeDepthNotWakeable as DeviceWakeDepthNotWakeable, _KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoBuildDeviceIoControlRequest, IoBuildSynchronousFsdRequest, IoGetAttachedDeviceReference, IoGetNextIrpStackLocation, KEVENT, KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PD3COLD_SUPPORT_INTERFACE, PINTERFACE, PSTORAGE_IDLE_POWERUP_REASON, STORAGE_IDLE_POWER, STORAGE_IDLE_POWERUP_REASON};
use core::ffi::c_void;
use ntkrnl_sys::ddk::RtlAssert;

extern "C" {
    static GUID_D3COLD_SUPPORT_INTERFACE: GUID;
    static D3COLD_SUPPORT_INTERFACE_VERSION: u16;
    fn FREE_POOL(ptr: *mut core::ffi::c_void);
}
extern "C" {
}
extern "system" {
    fn DeviceSendSrbSynchronously(device: PDEVICE_OBJECT, srb: *mut SCSI_REQUEST_BLOCK, buffer: *mut core::ffi::c_void, length: u32, request_sense: u8, sense_info: *mut u8) -> NTSTATUS;
    fn DeviceGetConfigurationWithAlloc( Device: PDEVICE_OBJECT, Buffer: *mut *mut core::ffi::c_void, BufferSize: *mut ULONG, Feature: u32, RequestType: u32, ) -> NTSTATUS;
}
unsafe extern "C" {
}
unsafe extern "C" {
    fn DeviceSendIoctlAsynchronously( DeviceExtension: PCDROM_DEVICE_EXTENSION, IoControlCode: u32, LowerPdo: PDEVICE_OBJECT, );
}
unsafe extern "C" {
    fn DeviceFindFeaturePage(buffer: *mut u8, buffer_size: u32, feature_code: u32) -> *mut FEATURE_DATA_REMOVABLE_MEDIUM;
}

const IRP_MN_QUERY_INTERFACE: u8 = 0x08u8;

// cdrom.h:68: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
const NonPagedPoolNx: u32 = 0;
const CDROM_TAG_ZERO_POWER_ODD: u32 = 0x5A436353;
const FeatureRemovableMedium: u32 = 0x0003; // ntddmmc.h:78
const LOADING_MECHANISM_TRAY: u32 = 1;
const LOADING_MECHANISM_CADDY: u32 = 0x0;  // was 2 (LLM-invented); bindgen-correct
const DELAY_TIME_TO_ENTER_ZERO_POWER_IN_MS: u32 = 0xea60;  // was 0 (LLM-invented); bindgen-correct
const FeatureProfileList: u32 = 0;
const SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL: u32 = 0;
const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
const TRACE_FLAG_POWER: u32 = 2; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
const PowerSystemWorking: u32 = 1;
// CTL_CODE(IOCTL_STORAGE_BASE=0x2d, code, METHOD_BUFFERED=0, FILE_ANY_ACCESS=0)
const IOCTL_STORAGE_GET_IDLE_POWERUP_REASON: u32 = 0x002D1C84;
const IOCTL_STORAGE_POWER_ACTIVE: u32 = 0x002D1C88;
const IOCTL_STORAGE_POWER_IDLE: u32 = 0x002D1C8C;
#[no_mangle]
pub unsafe extern "C" fn DeviceInitializeZPODD(DeviceExtension: *mut CDROM_DEVICE_EXTENSION) -> NTSTATUS {
    PAGED_CODE!();

    let mut status: NTSTATUS = STATUS_SUCCESS;
    let mut tempStatus: NTSTATUS;
    let mut zpoddInfo: *mut ZERO_POWER_ODD_INFO = core::ptr::null_mut();
    let mut removableMediumHeader: *mut FEATURE_DATA_REMOVABLE_MEDIUM;
    let mut ZPODDEnabledInRegistry: u32;
    let mut deepestWakeableDstate: DEVICE_WAKE_DEPTH = DeviceWakeDepthNotWakeable;
    let mut inHomePosition: i32;

    'cleanup: {
    if !(*DeviceExtension).ZeroPowerODDInfo.is_null() {
        // Already initialized.
        break 'cleanup;
    }

    ZPODDEnabledInRegistry = DeviceGetZPODDEnabledFromRegistry();

    if ZPODDEnabledInRegistry == 0 {
        status = STATUS_NOT_SUPPORTED;
        TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
                   b"DeviceInitializeZPODD: ZPODD not enabled due to registry settings.\n\0".as_ptr() as *const i8);
        break 'cleanup;
    }

    zpoddInfo = ExAllocatePoolWithTag(
        NonPagedPoolNx,
        core::mem::size_of::<ZERO_POWER_ODD_INFO>(),
        CDROM_TAG_ZERO_POWER_ODD,
    ) as *mut ZERO_POWER_ODD_INFO;

    if zpoddInfo.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
        break 'cleanup;
    }

    core::ptr::write_bytes((zpoddInfo) as *mut u8, 0, core::mem::size_of::<ZERO_POWER_ODD_INFO>());

    // Check for DA & AN
    if (*DeviceExtension).PowerDescriptor.is_null()
        || (*(*DeviceExtension).PowerDescriptor).DeviceAttentionSupported == 0
        || (*(*DeviceExtension).PowerDescriptor).AsynchronousNotificationSupported == 0
    {
        status = STATUS_NOT_SUPPORTED;
        TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
                   b"DeviceInitializeZPODD: ZPODD not enabled due to SATA features not present.\n\0".as_ptr() as *const i8);
        break 'cleanup;
    }

    // Check for LoChange / MediaRemoval
    let capabilities_buffer: *mut u8 = (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBuffer as *mut u8;
    let capabilities_buffer_size: u32 = (*DeviceExtension).DeviceAdditionalData.Mmc.CapabilitiesBufferSize;
    removableMediumHeader = DeviceFindFeaturePage(capabilities_buffer, capabilities_buffer_size, FeatureRemovableMedium);

    if removableMediumHeader.is_null()
        || !(
            ((*removableMediumHeader).LoadingMechanism() as u32 == LOADING_MECHANISM_TRAY)
                && ((*removableMediumHeader).Load() == 0)
                && ((*removableMediumHeader).DBML() != 0)
        )
        && !(
            ((*removableMediumHeader).LoadingMechanism() as u32 == LOADING_MECHANISM_CADDY)
                && ((*removableMediumHeader).Load() == 0)
                && ((*(*DeviceExtension).MediaChangeDetectionInfo).Gesn.Supported != 0)
        )
    {
        status = STATUS_NOT_SUPPORTED;
        TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
                   b"DeviceInitializeZPODD: ZPODD not enabled due to ODD features not present.\n\0".as_ptr() as *const i8);
        break 'cleanup;
    }

    (*zpoddInfo).LoadingMechanism = (*removableMediumHeader).LoadingMechanism();
    (*zpoddInfo).Load = (*removableMediumHeader).Load();

    // Check for ACPI
    status = DeviceQueryD3ColdInterface(
        DeviceExtension,
        &raw mut (*zpoddInfo).D3ColdInterface as *mut D3COLD_SUPPORT_INTERFACE,
    );

    if status < 0 {
        TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
                   b"DeviceInitializeZPODD: Query D3Cold support interface failed.\n\0".as_ptr() as *const i8);
        break 'cleanup;
    }

    let d3ColdInterface: *mut D3COLD_SUPPORT_INTERFACE = &raw mut (*zpoddInfo).D3ColdInterface;
    let d3cold_if = &*d3ColdInterface;
    let get_idle_wake_info: unsafe extern "system" fn(*mut c_void, u32, *mut DEVICE_WAKE_DEPTH) -> NTSTATUS =
        core::mem::transmute_copy(&d3cold_if.GetIdleWakeInfo);
    status = get_idle_wake_info(d3cold_if.Context, PowerSystemWorking, &raw mut deepestWakeableDstate);

    if status < 0 {
        break 'cleanup;
    }

    if deepestWakeableDstate != DeviceWakeDepthD3cold
        || (*(*DeviceExtension).PowerDescriptor).D3ColdSupported == 0
    {
        status = STATUS_NOT_SUPPORTED;
        TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
                   b"DeviceInitializeZPODD: ZPODD not enabled due to ACPI support not present.\n\0".as_ptr() as *const i8);
        break 'cleanup;
    }

    status = DeviceSendEnableIdlePowerIoctl(DeviceExtension, 1, 1, DELAY_TIME_TO_ENTER_ZERO_POWER_IN_MS);
    if status < 0 {
        TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
                   b"DeviceInitializeZPODD: ZPODD not enabled due to runtime power framework.\n\0".as_ptr() as *const i8);
        break 'cleanup;
    }

    TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
               b"DeviceInitializeZPODD: ZPODD is enabled.\n\0".as_ptr() as *const i8);
    (*DeviceExtension).ZeroPowerODDInfo = zpoddInfo;

    inHomePosition = DeviceZPODDIsInHomePosition(DeviceExtension) as i32;
    if inHomePosition == 0 {
        TracePrint!(TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
                   b"DeviceInitializeZPODD: not ready to power off, device marked as active\n\0".as_ptr() as *const i8);
        DeviceMarkActive(DeviceExtension, 1, 0);
    } else {
        if (*zpoddInfo).GetConfigurationBuffer.is_null() {
            tempStatus = DeviceGetConfigurationWithAlloc(
                (*DeviceExtension).Device as PDEVICE_OBJECT,
                &raw mut (*zpoddInfo).GetConfigurationBuffer as *mut *mut c_void,
                &raw mut (*zpoddInfo).GetConfigurationBufferSize,
                FeatureProfileList,
                SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL,
            );
            let _ = tempStatus; // UNREFERENCED_PARAMETER
        }
    }
    } // 'cleanup

    if status < 0 {
        tempStatus = DeviceSendEnableIdlePowerIoctl(DeviceExtension, 0, 0, DELAY_TIME_TO_ENTER_ZERO_POWER_IN_MS);
        if tempStatus >= 0 {
            DeviceMarkActive(DeviceExtension, 1, 0);
        }
        FREE_POOL(zpoddInfo as *mut c_void);
    }

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "C" fn DeviceGetZPODDEnabledFromRegistry() -> u32 {
    #[allow(non_snake_case)]

    const WSTR: [u16; 20] = [
        90, 101, 114, 111, 80, 111, 119, 101, 114, 79, 68, 68, 69, 110, 97, 98, 108, 101, 100, 0,
    ];

    let mut status: i32; // STATUS_SUCCESS
    let mut registryKey: WDFKEY = core::ptr::null_mut();
    let mut ZPODDEnabled: u32 = 0;

    let registryValueName = ntkrnl_sys::ddk::UNICODE_STRING {
        Length: 38,
        MaximumLength: 38,
        Buffer: &WSTR[0] as *const u16 as *mut u16,
    };

    PAGED_CODE!();

    status = WdfDriverOpenParametersRegistryKey(
        WdfGetDriver(),
        0x20019, // KEY_READ
        core::ptr::null_mut(),
        &mut registryKey as *mut WDFKEY,
    );

    if status >= 0 {
        status = WdfRegistryQueryULong(
            registryKey,
            &raw const registryValueName,
            &mut ZPODDEnabled,
        );
        WdfRegistryClose(registryKey);
    }

    if status < 0 {
        ZPODDEnabled = 1;
    }

    ZPODDEnabled
}
#[no_mangle]
pub unsafe extern "C" fn DeviceQueryD3ColdInterface(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    D3ColdInterface: *mut D3COLD_SUPPORT_INTERFACE,
) -> NTSTATUS {
    let mut irp: PIRP;
    let mut event: KEVENT = core::mem::zeroed();
    let mut status: NTSTATUS;
    let mut targetDevice: PDEVICE_OBJECT;
    let mut ioStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut irpStack: PIO_STACK_LOCATION;

    // PAGED_CODE();

    core::ptr::write_bytes((D3ColdInterface) as *mut u8, 0, core::mem::size_of::<D3COLD_SUPPORT_INTERFACE>());
    KeInitializeEvent(&raw mut event, _EVENT_TYPE_NotificationEvent, 0);

    targetDevice = IoGetAttachedDeviceReference((*DeviceExtension).DeviceObject);

    irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as u32,
        targetDevice,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        &raw mut event,
        &raw mut ioStatus,
    );

    'cleanup: {
        if irp.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            break 'cleanup;
        }

        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
        (*irp).IoStatus.Information = 0;

        irpStack = IoGetNextIrpStackLocation(irp);
        (*irpStack).MinorFunction = IRP_MN_QUERY_INTERFACE;
        (*irpStack).Parameters.QueryInterface.InterfaceType = 
            &raw const GUID_D3COLD_SUPPORT_INTERFACE as *const GUID as *mut GUID;
        (*irpStack).Parameters.QueryInterface.Size = 
            core::mem::size_of::<D3COLD_SUPPORT_INTERFACE>() as u16;
        (*irpStack).Parameters.QueryInterface.Version = D3COLD_SUPPORT_INTERFACE_VERSION;
        (*irpStack).Parameters.QueryInterface.Interface = D3ColdInterface as PINTERFACE;
        (*irpStack).Parameters.QueryInterface.InterfaceSpecificData = core::ptr::null_mut();

        status = IoCallDriver(targetDevice, irp);

        if status == STATUS_PENDING {
            KeWaitForSingleObject(&raw mut event as *mut c_void, _KWAIT_REASON_Executive, 0i8, 0, core::ptr::null_mut());
            status = ioStatus.__bindgen_anon_1.Status;
        }

        if !NT_SUCCESS(status) {
            break 'cleanup;
        }

        // NT_ASSERT equivalent
        if (*D3ColdInterface).SetD3ColdSupport.is_none() {
            RtlAssert(
                b"D3ColdInterface->SetD3ColdSupport != NULL\0".as_ptr() as *mut core::ffi::c_void,
                b"zpodd.rs\0".as_ptr() as *mut core::ffi::c_void,
                line!(),
                core::ptr::null_mut::<i8>(),
            );
        }
        if (*D3ColdInterface).GetIdleWakeInfo.is_none() {
            RtlAssert(
                b"D3ColdInterface->GetIdleWakeInfo != NULL\0".as_ptr() as *mut core::ffi::c_void,
                b"zpodd.rs\0".as_ptr() as *mut core::ffi::c_void,
                line!(),
                core::ptr::null_mut::<i8>(),
            );
        }
        if (*D3ColdInterface).GetD3ColdCapability.is_none() {
            RtlAssert(
                b"D3ColdInterface->GetD3ColdCapability != NULL\0".as_ptr() as *mut core::ffi::c_void,
                b"zpodd.rs\0".as_ptr() as *mut core::ffi::c_void,
                line!(),
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    ObDereferenceObject(targetDevice as *mut c_void);

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceSendEnableIdlePowerIoctl(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    WakeCapable: u8,
    Enable: u8,
    D3IdleTimeout: u32,
) -> i32 {
    let mut status: i32;
    let mut idlePower: STORAGE_IDLE_POWER = unsafe { core::mem::zeroed() };
    let mut ioStatus: _IO_STATUS_BLOCK = unsafe { core::mem::zeroed() };
    let mut irp: *mut IRP;
    let mut event: KEVENT = unsafe { core::mem::zeroed() };

    // PAGED_CODE();  (omitted, no equivalent needed)

    idlePower.Version = 1;
    idlePower.Size = core::mem::size_of::<STORAGE_IDLE_POWER>() as u32;
    idlePower.set_WakeCapableHint(WakeCapable as u32);
    idlePower.set_D3ColdSupported(Enable as u32);
    idlePower.D3IdleTimeout = D3IdleTimeout;

    KeInitializeEvent(&raw mut event, _EVENT_TYPE_NotificationEvent, 0);

    irp = IoBuildDeviceIoControlRequest(
        0x2D1C80, /* IOCTL_STORAGE_ENABLE_IDLE_POWER */
        (*DeviceExtension).LowerPdo,
        core::ptr::addr_of_mut!(idlePower) as *mut core::ffi::c_void,
        core::mem::size_of::<STORAGE_IDLE_POWER>() as u32,
        core::ptr::null_mut(),
        0,
        0,
        &raw mut event,
        &raw mut ioStatus,
    );

    if irp.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        status = IoCallDriver((*DeviceExtension).LowerPdo, irp);

        if status == STATUS_PENDING {
            KeWaitForSingleObject(
                &raw mut event as *mut c_void,
                _KWAIT_REASON_Executive,
                0i8,
                0,
                core::ptr::null_mut(),
            );

            status = ioStatus.__bindgen_anon_1.Status;
        }
    }

    TracePrint!(
        0x00000004, /* TRACE_LEVEL_INFORMATION */
        0x00004000, /* TRACE_FLAG_POWER */
        b"DeviceSendEnableIdlePowerIoctl: Port driver returned status (%x) for FDO (%p)\n\tD3ColdSupported: %u\n\tD3IdleTimeout: %u (ms)\0"
            .as_ptr() as *const i8,
        status as i32,
        (*DeviceExtension).DeviceObject as *const core::ffi::c_void,
        Enable as u32,
        0, /* DELAY_TIME_TO_ENTER_ZERO_POWER_IN_MS */
    );

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceReleaseZPODDResources(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
) {
    // PAGED_CODE(); // macro omitted — no behavioral effect in free builds
    let zpoddInfo: *mut ZERO_POWER_ODD_INFO = (*DeviceExtension).ZeroPowerODDInfo;

    if !zpoddInfo.is_null() {
        FREE_POOL((*zpoddInfo).GetConfigurationBuffer as *mut core::ffi::c_void);
        FREE_POOL(zpoddInfo as *mut core::ffi::c_void);
    }

    return;
}
#[no_mangle]
pub unsafe extern "C" fn DeviceZPODDGetPowerupReason(
    DeviceExtension: *mut CDROM_DEVICE_EXTENSION,
    PowerupReason: *mut STORAGE_IDLE_POWERUP_REASON,
) -> i32 {
    let mut status: i32;
    let irp: *mut IRP;
    let mut ioStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut event: KEVENT = core::mem::zeroed();

    RtlZeroMemory(
        PowerupReason as *mut c_void,
        core::mem::size_of::<STORAGE_IDLE_POWERUP_REASON>(),
    );

    (*PowerupReason).Size = core::mem::size_of::<STORAGE_IDLE_POWERUP_REASON>() as u32;
    (*PowerupReason).Version = STORAGE_IDLE_POWERUP_REASON_VERSION_V1 as u32;

    KeInitializeEvent(&mut event, _EVENT_TYPE_NotificationEvent, 0);

    irp = IoBuildDeviceIoControlRequest(
        IOCTL_STORAGE_GET_IDLE_POWERUP_REASON as u32,
        (*DeviceExtension).LowerPdo,
        PowerupReason as *mut c_void,
        core::mem::size_of::<STORAGE_IDLE_POWERUP_REASON>() as u32,
        PowerupReason as *mut c_void,
        core::mem::size_of::<STORAGE_IDLE_POWERUP_REASON>() as u32,
        0, // InternalDeviceIoControl = FALSE
        &mut event,
        &mut ioStatus,
    );

    if irp.is_null() {
        status = STATUS_INSUFFICIENT_RESOURCES as i32;
    } else {
        status = IoCallDriver((*DeviceExtension).LowerPdo, irp);

        if status == STATUS_PENDING as i32 {
            KeWaitForSingleObject(
                &mut event as *mut _ as *mut c_void,
                _KWAIT_REASON_Executive,
                0i8,
                0, // Alertable = FALSE
                core::ptr::null_mut(), // Timeout = NULL
            );
            status = ioStatus.__bindgen_anon_1.Status;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn DeviceZPODDIsInHomePosition(DeviceExtension: *mut CDROM_DEVICE_EXTENSION) -> u8 {

    let mut status: NTSTATUS;
    let zpoddInfo: *mut ZERO_POWER_ODD_INFO = (*DeviceExtension).ZeroPowerODDInfo;
    let mut srb: SCSI_REQUEST_BLOCK = core::mem::zeroed();
    let cdb: *mut CDB = core::ptr::addr_of_mut!(srb.Cdb) as *mut CDB;
    let mut inHomePosition: u8 = 0;

    if !zpoddInfo.is_null() {
        (*zpoddInfo).SenseKey = 0;
        (*zpoddInfo).AdditionalSenseCode = 0;
        (*zpoddInfo).AdditionalSenseCodeQualifier = 0;

        core::ptr::write_bytes(core::ptr::addr_of_mut!(srb) as *mut u8, 0, core::mem::size_of::<SCSI_REQUEST_BLOCK>());

        srb.CdbLength = 6;
        (*cdb).CDB6GENERIC.as_mut().OperationCode = SCSIOP_TEST_UNIT_READY as u8;
        srb.TimeOutValue = CDROM_TEST_UNIT_READY_TIMEOUT;

        let device = (*DeviceExtension).Device;
        status = DeviceSendSrbSynchronously(
            device as PDEVICE_OBJECT,
            core::ptr::addr_of_mut!(srb),
            core::ptr::null_mut(),
            0,
            0,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(status) {
            return 0;
        }

        let loading_mech = (*zpoddInfo).LoadingMechanism;
        let load = (*zpoddInfo).Load;
        let sense_key = (*zpoddInfo).SenseKey;
        let asc = (*zpoddInfo).AdditionalSenseCode;
        let ascq = (*zpoddInfo).AdditionalSenseCodeQualifier;

        let condition_tray = (u32::from(loading_mech) == LOADING_MECHANISM_TRAY)
            && (load == 0)
            && (u32::from(sense_key) == SCSI_SENSE_NOT_READY)
            && (u32::from(asc) == SCSI_ADSENSE_NO_MEDIA_IN_DEVICE)
            && (ascq == 0x01);
        let condition_caddy = (u32::from(loading_mech) == LOADING_MECHANISM_CADDY)
            && (load == 0)
            && (u32::from(sense_key) == SCSI_SENSE_NOT_READY)
            && (u32::from(asc) == SCSI_ADSENSE_NO_MEDIA_IN_DEVICE);

        if condition_tray || condition_caddy {
            inHomePosition = 1;
        }
    }

    inHomePosition
}
#[no_mangle]
pub unsafe extern "C" fn DeviceMarkActive(
    DeviceExtension: PCDROM_DEVICE_EXTENSION,
    IsActive: BOOLEAN,
    SetIdleTimeout: BOOLEAN,
) {
    let mut status: NTSTATUS = STATUS_SUCCESS;
    let zpoddInfo = (*DeviceExtension).ZeroPowerODDInfo;

    PAGED_CODE!();

    if (*DeviceExtension).IsActive != IsActive {
        if IsActive == FALSE && !zpoddInfo.is_null() {
            if (*zpoddInfo).GetConfigurationBuffer.is_null() {
                let _ = DeviceGetConfigurationWithAlloc(
                    (*DeviceExtension).Device as PDEVICE_OBJECT,
                    core::ptr::addr_of_mut!((*zpoddInfo).GetConfigurationBuffer) as *mut *mut c_void,
                    core::ptr::addr_of_mut!((*zpoddInfo).GetConfigurationBufferSize),
                    FeatureProfileList,
                    SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL,
                );
            }
        }

        if SetIdleTimeout != FALSE {
            status = DeviceSendEnableIdlePowerIoctl(
                DeviceExtension,
                FALSE,
                FALSE,
                if IsActive != FALSE {
                    DELAY_TIME_TO_ENTER_ZERO_POWER_IN_MS
                } else {
                    DELAY_TIME_TO_ENTER_AOAC_IDLE_POWER_IN_MS
                },
            );
        }

        if (status as i32) >= 0 {
            DeviceSendIoctlAsynchronously(
                DeviceExtension,
                if IsActive != FALSE {
                    IOCTL_STORAGE_POWER_ACTIVE
                } else {
                    IOCTL_STORAGE_POWER_IDLE
                },
                (*DeviceExtension).LowerPdo,
            );
        }

        (*DeviceExtension).IsActive = IsActive;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (C) Microsoft Corporation. All rights reserved.
//
// Module Name:
//
//     zpodd.c
//
// Abstract:
//
//     Code for Zero Power ODD support.
//
// Environment:
//
//     kernel mode only
//
// Notes:
//
//
// Revision History:
//
// --*/
//
// #include "ntddk.h"
// #include "ntddstor.h"
// #include "wdmguid.h"
// #include "cdrom.h"
// #include "mmc.h"
// #include "ioctl.h"
// #include "scratch.h"
//
// #ifdef DEBUG_USE_WPP
// #include "zpodd.tmh"
// #endif
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// ULONG
// DeviceGetZPODDEnabledFromRegistry();
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceQueryD3ColdInterface(
//         PCDROM_DEVICE_EXTENSION     DeviceExtension,
//        PD3COLD_SUPPORT_INTERFACE   D3ColdInterface
//     );
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceSendEnableIdlePowerIoctl(
//         PCDROM_DEVICE_EXTENSION     DeviceExtension,
//         BOOLEAN                     WakeCapable,
//         BOOLEAN                     Enable,
//         ULONG                       D3IdleTimeout
//     );
//
// #if ALLOC_PRAGMA
//
// #pragma alloc_text(PAGE, DeviceInitializeZPODD)
// #pragma alloc_text(PAGE, DeviceGetZPODDEnabledFromRegistry)
// #pragma alloc_text(PAGE, DeviceQueryD3ColdInterface)
// #pragma alloc_text(PAGE, DeviceSendEnableIdlePowerIoctl)
// #pragma alloc_text(PAGE, DeviceReleaseZPODDResources)
// #pragma alloc_text(PAGE, DeviceZPODDIsInHomePosition)
// #pragma alloc_text(PAGE, DeviceMarkActive)
//
// #endif
//
// #pragma warning(push)
// #pragma warning(disable:4152) // nonstandard extension, function/data pointer conversion in expression
// #pragma warning(disable:26000) // read overflow reported because of pointer type conversion
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceInitializeZPODD(
//      PCDROM_DEVICE_EXTENSION  DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     This routine initialize the contents of ZPODD structure.
//
// Arguments:
//
//     DeviceExtension - the device extension
//
// Return Value:
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                            status = STATUS_SUCCESS;
//     NTSTATUS                            tempStatus = STATUS_SUCCESS;
//     PZERO_POWER_ODD_INFO                zpoddInfo = NULL;
//     PFEATURE_DATA_REMOVABLE_MEDIUM      removableMediumHeader = NULL;
//     ULONG                               ZPODDEnabledInRegistry = 0;
//     PD3COLD_SUPPORT_INTERFACE           d3ColdInterface = NULL;
//     DEVICE_WAKE_DEPTH                   deepestWakeableDstate = DeviceWakeDepthNotWakeable;
//     BOOLEAN                             inHomePosition = FALSE;
//
//     PAGED_CODE();
//
//     if (DeviceExtension->ZeroPowerODDInfo != NULL)
//     {
//         //
//         // Already initialized.
//         //
//
//         goto Cleanup;
//     }
//
//     ZPODDEnabledInRegistry = DeviceGetZPODDEnabledFromRegistry();
//
//     if (ZPODDEnabledInRegistry == 0)
//     {
//         //
//         // User has explicitly disabled Zero Power ODD.
//         //
//
//         status = STATUS_NOT_SUPPORTED;
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                     "DeviceInitializeZPODD: ZPODD not enabled due to registry settings.\n"
//                     ));
//
//         goto Cleanup;
//     }
//
//     zpoddInfo = ExAllocatePoolWithTag(NonPagedPoolNx,
//                                       sizeof(ZERO_POWER_ODD_INFO),
//                                       CDROM_TAG_ZERO_POWER_ODD);
//
//     if (zpoddInfo == NULL)
//     {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//
//         goto Cleanup;
//     }
//
//     RtlZeroMemory(zpoddInfo, sizeof (ZERO_POWER_ODD_INFO));
//
//     //
//     // Check the system for the following prerequisites:
//     //
//     // 1. SATA: Device Attention line
//     // 2. SATA: Asynchronous Notification
//     // 3. ODD:  LoChange / MediaRemoval
//     // 4. ACPI: Wake capable
//     //
//     // Only drawer and slot loading types have well defined behaviors in the spec, so only these two
//     // types are supported.
//     //
//
//     //
//     // Check for DA & AN
//     //
//
//     if ((DeviceExtension->PowerDescriptor == NULL) ||
//         (DeviceExtension->PowerDescriptor->DeviceAttentionSupported == FALSE) ||
//         (DeviceExtension->PowerDescriptor->AsynchronousNotificationSupported == FALSE))
//     {
//         status = STATUS_NOT_SUPPORTED;
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                     "DeviceInitializeZPODD: ZPODD not enabled due to SATA features not present.\n"
//                     ));
//
//         goto Cleanup;
//     }
//
//     //
//     // Check for LoChange / MediaRemoval
//     //
//
//     removableMediumHeader = (PFEATURE_DATA_REMOVABLE_MEDIUM)
//                             DeviceFindFeaturePage(DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBuffer,
//                                                   DeviceExtension->DeviceAdditionalData.Mmc.CapabilitiesBufferSize,
//                                                   FeatureRemovableMedium);
//
//     if ((removableMediumHeader == NULL) ||
//         (!((removableMediumHeader->LoadingMechanism == LOADING_MECHANISM_TRAY) && (removableMediumHeader->Load == 0) &&     // Drawer ...
//          (removableMediumHeader->DBML != FALSE)) &&                                                                         // requires LoChange/NotBusy
//         !((removableMediumHeader->LoadingMechanism == LOADING_MECHANISM_CADDY) && (removableMediumHeader->Load == 0) &&     // Slot ...
//          (DeviceExtension->MediaChangeDetectionInfo->Gesn.Supported != FALSE))))                                            // requires MediaRemoval
//     {
//         status = STATUS_NOT_SUPPORTED;
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                     "DeviceInitializeZPODD: ZPODD not enabled due to ODD features not present.\n"
//                     ));
//
//         goto Cleanup;
//     }
//
//     zpoddInfo->LoadingMechanism = removableMediumHeader->LoadingMechanism;
//     zpoddInfo->Load = removableMediumHeader->Load;
//
//     //
//     // Check for ACPI
//     //
//
//     status = DeviceQueryD3ColdInterface(DeviceExtension, &zpoddInfo->D3ColdInterface);
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                     "DeviceInitializeZPODD: Query D3Cold support interface failed.\n"
//                     ));
//
//         goto Cleanup;
//     }
//
//     //
//     // If the platform supports Zero Power ODD, the following conditions must be met:
//     //
//     // 1. The deepest wakeable D-state for the device is D3Cold;
//     // 2. The platform supports D3Cold for the device.
//     //
//
//     d3ColdInterface = &zpoddInfo->D3ColdInterface;
//
//     status = d3ColdInterface->GetIdleWakeInfo(d3ColdInterface->Context,
//                                               PowerSystemWorking,
//                                               &deepestWakeableDstate);
//
//     if (!NT_SUCCESS(status))
//     {
//         goto Cleanup;
//     }
//
//     //
//     // DeviceExtension->PowerDescriptor->D3ColdSupported is retrieved from lower layer.
//     // It has more accurate supportive information than just uses d3ColdInterface->GetD3ColdCapability
//     //
//     if ((deepestWakeableDstate != DeviceWakeDepthD3cold) ||
//         (DeviceExtension->PowerDescriptor->D3ColdSupported == FALSE))
//     {
//         status = STATUS_NOT_SUPPORTED;
//
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                     "DeviceInitializeZPODD: ZPODD not enabled due to ACPI support not present.\n"
//                     ));
//
//         goto Cleanup;
//     }
//
//     //
//     // The system meets all requirements. Go ahead and enable ZPODD.
//     //
//
//     //
//     // Register with the runtime power framework.
//     // Note that no un-registration is needed during tear-down.
//     // D3Cold will be enabled (success case of following call) or disabled by port driver during processing Enable Idle Power IOCTL.
//     //
//
//     status = DeviceSendEnableIdlePowerIoctl(DeviceExtension, TRUE, TRUE, DELAY_TIME_TO_ENTER_ZERO_POWER_IN_MS);
//
//     if (!NT_SUCCESS(status))
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                     "DeviceInitializeZPODD: ZPODD not enabled due to runtime power framework.\n"
//                     ));
//
//         goto Cleanup;
//     }
//
//     TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                 "DeviceInitializeZPODD: ZPODD is enabled.\n"
//                 ));
//
//     DeviceExtension->ZeroPowerODDInfo = zpoddInfo;
//
//     //
//     // If device is not in home position, then we should take an active reference here
//     // to prevent it from being powered off.
//     //
//
//     inHomePosition = DeviceZPODDIsInHomePosition(DeviceExtension);
//
//     if (inHomePosition == FALSE)
//     {
//         TracePrint!((TRACE_LEVEL_INFORMATION, TRACE_FLAG_POWER,
//                    "DeviceInitializeZPODD: not ready to power off, device marked as active\n"));
//
//         DeviceMarkActive(DeviceExtension, TRUE, FALSE);
//     }
//     else
//     {
//         //
//         // cache get configuration response.
//         // failing is not critical, so we don't want to check for status here.
//         //
//
//         if (zpoddInfo->GetConfigurationBuffer == NULL)
//         {
//             tempStatus = DeviceGetConfigurationWithAlloc(DeviceExtension->Device,
//                                                          &zpoddInfo->GetConfigurationBuffer,
//                                                          &zpoddInfo->GetConfigurationBufferSize,
//                                                          FeatureProfileList,
//                                                          SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL);
//
//             UNREFERENCED_PARAMETER(tempStatus); // Avoid PREFAST warning.
//         }
//     }
//
// Cleanup:
//
//     if (!NT_SUCCESS(status))
//     {
//         //
//         // We register always even in non-ZPODD case, per request from storport.
//         //
//
//         tempStatus = DeviceSendEnableIdlePowerIoctl(DeviceExtension, FALSE, FALSE, DELAY_TIME_TO_ENTER_ZERO_POWER_IN_MS);
//
//         if (NT_SUCCESS(tempStatus))
//         {
//             //
//             // Mark the device active; this reference will never be released unless the system enters a
//             // low power state.
//             //
//
//             DeviceMarkActive(DeviceExtension, TRUE, FALSE);
//         }
//
//         FREE_POOL(zpoddInfo);
//     }
//
//     //
//     // If Zero Power ODD is not supported, we should not block the device init sequence.
//     //
//
//     return STATUS_SUCCESS;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// ULONG
// DeviceGetZPODDEnabledFromRegistry()
// /*++
//
// Routine Description:
//
//     Get the ZeroPowerODDEnabled value from registry, which dictates if Zero Power ODD
//     should be enabled or not. If the value is not in registry, by default Zero
//     Power ODD is enabled.
//
// Arguments:
//
//     None
//
// Return Value:
//
//     ULONG
//
// --*/
// {
//     NTSTATUS    status = STATUS_SUCCESS;
//     WDFKEY      registryKey = NULL;
//     ULONG       ZPODDEnabled = 0;
//
//     DECLARE_CONST_UNICODE_STRING(registryValueName, L"ZeroPowerODDEnabled");
//
//     PAGED_CODE();
//
//     //
//     // open the Parameters key under the service key.
//     //
//
//     status = WdfDriverOpenParametersRegistryKey(WdfGetDriver(),
//                                                 KEY_READ,
//                                                 WDF_NO_OBJECT_ATTRIBUTES,
//                                                 &registryKey);
//
//     if (NT_SUCCESS(status))
//     {
//         status = WdfRegistryQueryULong(registryKey,
//                                        &registryValueName,
//                                        &ZPODDEnabled);
//
//         WdfRegistryClose(registryKey);
//     }
//
//     if (!NT_SUCCESS(status))
//     {
//         //
//         // By default, Zero Power ODD is enabled
//         //
//
//         ZPODDEnabled = 1;
//     }
//
//     return ZPODDEnabled;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceQueryD3ColdInterface(
//         PCDROM_DEVICE_EXTENSION     DeviceExtension,
//        PD3COLD_SUPPORT_INTERFACE   D3ColdInterface
//     )
// /*++
//
// Routine Description:
//
//     Queries ACPI for the D3Cold support interface.
//
// Arguments:
//
//     DeviceExtension - the device extension
//     D3ColdInterface - output buffer receiving the interface
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     PIRP                irp = NULL;
//     KEVENT              event;
//     NTSTATUS            status = STATUS_SUCCESS;
//     PDEVICE_OBJECT      targetDevice = NULL;
//     IO_STATUS_BLOCK     ioStatus = {0};
//     PIO_STACK_LOCATION  irpStack = NULL;
//
//     PAGED_CODE();
//
//     RtlZeroMemory(D3ColdInterface, sizeof(D3COLD_SUPPORT_INTERFACE));
//
//     //
//     // Query D3COLD support interface synchronously
//     //
//
//     KeInitializeEvent(&event, NotificationEvent, FALSE);
//
//     targetDevice = IoGetAttachedDeviceReference(DeviceExtension->DeviceObject);
//
//     irp = IoBuildSynchronousFsdRequest(IRP_MJ_PNP,
//                                        targetDevice,
//                                        NULL,
//                                        0,
//                                        0,
//                                        &event,
//                                        &ioStatus);
//
//     if (irp == NULL)
//     {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//
//         goto Cleanup;
//     }
//
//     irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//     irp->IoStatus.Information = 0;
//
//     irpStack = IoGetNextIrpStackLocation(irp);
//     irpStack->MinorFunction = IRP_MN_QUERY_INTERFACE;
//     irpStack->Parameters.QueryInterface.InterfaceType = (LPGUID) &GUID_D3COLD_SUPPORT_INTERFACE;
//     irpStack->Parameters.QueryInterface.Size = sizeof (D3COLD_SUPPORT_INTERFACE);
//     irpStack->Parameters.QueryInterface.Version = D3COLD_SUPPORT_INTERFACE_VERSION;
//     irpStack->Parameters.QueryInterface.Interface = (PINTERFACE) D3ColdInterface;
//     irpStack->Parameters.QueryInterface.InterfaceSpecificData = NULL;
//
//     status = IoCallDriver(targetDevice, irp);
//
//     if (status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&event, Executive, KernelMode, FALSE, NULL);
//
//         status = ioStatus.Status;
//     }
//
//     if (!NT_SUCCESS(status))
//     {
//         goto Cleanup;
//     }
//
//     NT_ASSERT(D3ColdInterface->SetD3ColdSupport != NULL);
//     NT_ASSERT(D3ColdInterface->GetIdleWakeInfo != NULL);
//     NT_ASSERT(D3ColdInterface->GetD3ColdCapability != NULL);
//
// Cleanup:
//
//     ObDereferenceObject(targetDevice);
//
//     return status;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// NTSTATUS
// DeviceSendEnableIdlePowerIoctl(
//         PCDROM_DEVICE_EXTENSION     DeviceExtension,
//         BOOLEAN                     WakeCapable,
//         BOOLEAN                     Enable,
//         ULONG                       D3IdleTimeout
//     )
// /*++
//
// Routine Description:
//
//     Enables idle power support.
//
// Arguments:
//
//     DeviceExtension - the device extension
//     WakeCapable - whether the device is wake capable
//     Enable - enable / disable idle power management
//
// Return Value:
//
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS            status = STATUS_SUCCESS;
//     STORAGE_IDLE_POWER  idlePower = {0};
//     IO_STATUS_BLOCK     ioStatus = {0};
//     PIRP                irp = NULL;
//     KEVENT              event;
//
//     PAGED_CODE();
//
//     idlePower.Version = 1;
//     idlePower.Size = sizeof (STORAGE_IDLE_POWER);
//     idlePower.WakeCapableHint = WakeCapable;
//     idlePower.D3ColdSupported = Enable;
//     idlePower.D3IdleTimeout = D3IdleTimeout;
//
//     KeInitializeEvent(&event, NotificationEvent, FALSE);
//
//     irp = IoBuildDeviceIoControlRequest(IOCTL_STORAGE_ENABLE_IDLE_POWER,
//                                         DeviceExtension->LowerPdo,
//                                         &idlePower,
//                                         sizeof(STORAGE_IDLE_POWER),
//                                         NULL,
//                                         0,
//                                         FALSE,
//                                         &event,
//                                         &ioStatus);
//
//     if (irp == NULL)
//     {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//     else
//     {
//         //
//         // Send the synchronous request to port driver.
//         //
//
//         status = IoCallDriver(DeviceExtension->LowerPdo, irp);
//
//         if (status == STATUS_PENDING)
//         {
//             KeWaitForSingleObject(&event, Executive, KernelMode, FALSE, NULL);
//
//             status = ioStatus.Status;
//         }
//     }
//
//     TracePrint!((TRACE_LEVEL_INFORMATION,
//                 TRACE_FLAG_POWER,
//                 "DeviceSendEnableIdlePowerIoctl: Port driver returned status (%x) for FDO (%p)\n"
//                 "\tD3ColdSupported: %u\n"
//                 "\tD3IdleTimeout: %u (ms)",
//                 status,
//                 DeviceExtension->DeviceObject,
//                 Enable,
//                 DELAY_TIME_TO_ENTER_ZERO_POWER_IN_MS));
//
//     return status;
// }
//
// _IRQL_requires_max_(APC_LEVEL)
// VOID
// DeviceReleaseZPODDResources(
//      PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     This routine will cleanup any resources allocated for ZPODD.
//
// Arguments:
//
//     DeviceExtension - the device context
//
// Return Value:
//     None.
//
// --*/
// {
//     PZERO_POWER_ODD_INFO zpoddInfo = DeviceExtension->ZeroPowerODDInfo;
//
//     PAGED_CODE()
//
//     if (zpoddInfo != NULL)
//     {
//         FREE_POOL(zpoddInfo->GetConfigurationBuffer);
//         FREE_POOL(zpoddInfo);
//     }
//
//     return;
// }
//
// NTSTATUS
// DeviceZPODDGetPowerupReason(
//         PCDROM_DEVICE_EXTENSION DeviceExtension,
//        PSTORAGE_IDLE_POWERUP_REASON PowerupReason
//     )
// /*++
//
// Routine Description:
//
//     This routine queries the port driver for what caused the power up.
//
// Arguments:
//
//     DeviceExtension - device extension.
//     PowerupReason - what caused the power up.
//
// Return Value:
//     NTSTATUS
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PIRP                    irp = NULL;
//     IO_STATUS_BLOCK         ioStatus = {0};
//     KEVENT                  event;
//
//     RtlZeroMemory(PowerupReason, sizeof (STORAGE_IDLE_POWERUP_REASON));
//
//     PowerupReason->Size = sizeof (STORAGE_IDLE_POWERUP_REASON);
//     PowerupReason->Version = STORAGE_IDLE_POWERUP_REASON_VERSION_V1;
//
//     //
//     // Setup a synchronous irp.
//     //
//
//     KeInitializeEvent(&event, NotificationEvent, FALSE);
//
//     irp = IoBuildDeviceIoControlRequest(IOCTL_STORAGE_GET_IDLE_POWERUP_REASON,
//                                         DeviceExtension->LowerPdo,
//                                         PowerupReason,
//                                         sizeof (STORAGE_IDLE_POWERUP_REASON),
//                                         PowerupReason,
//                                         sizeof (STORAGE_IDLE_POWERUP_REASON),
//                                         FALSE,
//                                         &event,
//                                         &ioStatus);
//
//     if (irp == NULL)
//     {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//     else
//     {
//         //
//         // Send the synchronous request to port driver.
//         //
//
//         status = IoCallDriver(DeviceExtension->LowerPdo, irp);
//
//         if (status == STATUS_PENDING)
//         {
//             KeWaitForSingleObject(&event, Executive, KernelMode, FALSE, NULL);
//
//             status = ioStatus.Status;
//         }
//     }
//
//     return status;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// BOOLEAN
// DeviceZPODDIsInHomePosition(
//         PCDROM_DEVICE_EXTENSION DeviceExtension
//     )
// /*++
//
// Routine Description:
//
//     Checks to see if the device is ready to be powered off.
//     Requirements are: 1. tray closed 2. no media present.
//
// Arguments:
//
//     DeviceExtension - device extension.
//
// Return Value:
//     BOOLEAN
//
// --*/
// {
//     NTSTATUS             status = STATUS_SUCCESS;
//     PZERO_POWER_ODD_INFO zpoddInfo = DeviceExtension->ZeroPowerODDInfo;
//     SCSI_REQUEST_BLOCK   srb = {0};
//     PCDB                 cdb = (PCDB) srb.Cdb;
//     BOOLEAN              inHomePosition = FALSE;
//
//     PAGED_CODE();
//
//     if (zpoddInfo != NULL)
//     {
//         //
//         // Clear sense data.
//         //
//
//         zpoddInfo->SenseKey = 0;
//         zpoddInfo->AdditionalSenseCode = 0;
//         zpoddInfo->AdditionalSenseCodeQualifier = 0;
//
//         //
//         // Send a Test Unit Ready to check media & tray status.
//         //
//
//         RtlZeroMemory(&srb, sizeof(SCSI_REQUEST_BLOCK));
//
//         srb.CdbLength = 6;
//         cdb->CDB6GENERIC.OperationCode = SCSIOP_TEST_UNIT_READY;
//
//         srb.TimeOutValue = CDROM_TEST_UNIT_READY_TIMEOUT;
//
//         status = DeviceSendSrbSynchronously(DeviceExtension->Device,
//                                             &srb,
//                                             NULL,
//                                             0,
//                                             FALSE,
//                                             NULL);
//
// #ifdef __REACTOS__
//         if (!NT_SUCCESS(status))
//         {
//             return FALSE;
//         }
// #endif
//
//         //
//         // At this time, sense data, if available, is already copied into zpoddInfo.
//         //
//         // We don't check for status because we expect it to fail in case there is no media in device.
//         //
//         // Should enter Zero Power state if:
//         //
//         // Drawer:  02/3A/01
//         // Slot:    02/3A/xx
//         //
//
//         if (((zpoddInfo->LoadingMechanism == LOADING_MECHANISM_TRAY) && (zpoddInfo->Load == 0) && // Drawer
//              (zpoddInfo->SenseKey                       == SCSI_SENSE_NOT_READY) &&
//              (zpoddInfo->AdditionalSenseCode            == SCSI_ADSENSE_NO_MEDIA_IN_DEVICE) &&
//              (zpoddInfo->AdditionalSenseCodeQualifier   == 0x01)) ||
//             ((zpoddInfo->LoadingMechanism == LOADING_MECHANISM_CADDY) && (zpoddInfo->Load == 0) && // Slot
//              (zpoddInfo->SenseKey                       == SCSI_SENSE_NOT_READY) &&
//              (zpoddInfo->AdditionalSenseCode            == SCSI_ADSENSE_NO_MEDIA_IN_DEVICE)))
//         {
//             inHomePosition = TRUE;
//         }
//     }
//
//     return inHomePosition;
// }
//
// _IRQL_requires_max_(PASSIVE_LEVEL)
// VOID
// DeviceMarkActive(
//      PCDROM_DEVICE_EXTENSION DeviceExtension,
//      BOOLEAN                 IsActive,
//      BOOLEAN                 SetIdleTimeout
//     )
// /*++
//
// Routine Description:
//
//     This routine will mark the device as active / idle.
//
// Arguments:
//
//     DeviceExtension - the device context
//     IsActive - if the device should be marked as active
//
// Return Value:
//     None.
//
// --*/
// {
//     NTSTATUS                status = STATUS_SUCCESS;
//     PZERO_POWER_ODD_INFO    zpoddInfo = DeviceExtension->ZeroPowerODDInfo;
//
//     PAGED_CODE()
//
//     if (DeviceExtension->IsActive != IsActive)
//     {
//         if ((IsActive == FALSE) && (zpoddInfo != NULL))
//         {
//             // cache get configuration response.
//             // failing is not critical, so we don't want to check for status here.
//             if (zpoddInfo->GetConfigurationBuffer == NULL)
//             {
//                 (VOID)DeviceGetConfigurationWithAlloc(DeviceExtension->Device,
//                                                       &zpoddInfo->GetConfigurationBuffer,
//                                                       &zpoddInfo->GetConfigurationBufferSize,
//                                                       FeatureProfileList,
//                                                       SCSI_GET_CONFIGURATION_REQUEST_TYPE_ALL);
//             }
//         }
//
//         if (SetIdleTimeout)
//         {
//             status = DeviceSendEnableIdlePowerIoctl(DeviceExtension,
//                                                     FALSE,
//                                                     FALSE,
//                                                     IsActive ? DELAY_TIME_TO_ENTER_ZERO_POWER_IN_MS : DELAY_TIME_TO_ENTER_AOAC_IDLE_POWER_IN_MS);
//         }
//
//         if (NT_SUCCESS(status))
//         {
//             DeviceSendIoctlAsynchronously(DeviceExtension,
//                                           IsActive ? IOCTL_STORAGE_POWER_ACTIVE : IOCTL_STORAGE_POWER_IDLE,
//                                           DeviceExtension->LowerPdo);
//         }
//
//         DeviceExtension->IsActive = IsActive;
//     }
// }
//
// #pragma warning(pop) // un-sets any local warning changes
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
