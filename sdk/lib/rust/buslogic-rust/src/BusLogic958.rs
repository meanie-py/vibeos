use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PCCHAR};
use core::ffi::c_void;
// Driver-private structs (BusLogic_*, _HW_DEVICE_EXTENSION, the full _SCSI_REQUEST_BLOCK,
// HW_INITIALIZATION_DATA, SCSI_PHYSICAL_ADDRESS, ...) and the `pub fn ScsiPort*` decls
// come from the gen_driver_types.sh substrate. crate::types re-exports only those
// (the primitive aliases are blocklisted) → no glob clash with base::*.
use crate::types::*;
// The `static` header helpers (register/statistics) hand-translated in inlines.rs;
// bare-named enum members + NULL/INTERFACE_TYPE aliases the translation references in glue.rs.
use crate::inlines::*;
use crate::glue::*;

// SCSIPORT services — NTAPI = __stdcall on i386, so `extern "system"` (NOT the `extern "C"`
// the translation emitted, which would expect a cdecl symbol vs scsiport's `_Name@N` and
// fail the i386 link). On amd64/arm64 "system" == "C", so this is the i386 correctness fix.
extern "system" {
    fn ScsiPortInitialize( DriverObject: PVOID, Argument2: PVOID, HwInitializationData: *mut HW_INITIALIZATION_DATA, HwContext: PVOID, ) -> ULONG;
    fn ScsiPortStallExecution(Delay: ULONG);
    fn ScsiPortGetPhysicalAddress( HwDeviceExtension: PVOID, Srb: *mut _SCSI_REQUEST_BLOCK, VirtualAddress: PVOID, Length: *mut ULONG, ) -> SCSI_PHYSICAL_ADDRESS;
    fn ScsiPortConvertPhysicalAddressToUlong(Address: SCSI_PHYSICAL_ADDRESS) -> ULONG;
    fn ScsiPortCompleteRequest( HwDeviceExtension: PVOID, PathId: UCHAR, TargetId: UCHAR, Lun: UCHAR, SrbStatus: UCHAR, );
    fn ScsiPortLogError( HwDeviceExtension: PVOID, Srb: PSCSI_REQUEST_BLOCK, PathId: UCHAR, TargetId: UCHAR, Lun: UCHAR, ErrorCode: ULONG, UniqueId: ULONG, );
    fn ScsiPortGetUncachedExtension( HwDeviceExtension: PVOID, ConfigInfo: PPORT_CONFIGURATION_INFORMATION, NumberOfBytes: ULONG, ) -> PVOID;
    fn ScsiPortGetSrb( DeviceExtension: PVOID, PathId: UCHAR, TargetId: UCHAR, Lun: UCHAR, QueueTag: LONG, ) -> PSCSI_REQUEST_BLOCK;
    fn ScsiPortValidateRange( HwDeviceExtension: PVOID, BusType: INTERFACE_TYPE, SystemIoBusNumber: ULONG, IoAddress: SCSI_PHYSICAL_ADDRESS, NumberOfBytes: ULONG, InIoSpace: BOOLEAN, ) -> BOOLEAN;
    fn ScsiPortGetDeviceBase( HwDeviceExtension: PVOID, BusType: INTERFACE_TYPE, SystemIoBusNumber: ULONG, IoAddress: SCSI_PHYSICAL_ADDRESS, NumberOfBytes: ULONG, InIoSpace: BOOLEAN, ) -> PVOID;
}

// cdecl varargs (ScsiDebugPrint = the `DebugPrint` macro target; ScsiPortNotification) +
// CRT primitives (provided by libcntpr at the cohab link). These stay `extern "C"`.
extern "C" {
    fn ScsiDebugPrint(DebugPrintLevel: ULONG, DebugMessage: PCCHAR, ...);
    fn ScsiPortNotification( NotificationType: SCSI_NOTIFICATION_TYPE, HwDeviceExtension: PVOID, ... );
    fn memset( Dst: *mut core::ffi::c_void, Val: core::ffi::c_int, Size: usize, ) -> *mut core::ffi::c_void;
    fn memmove( dest: *mut ::core::ffi::c_void, src: *const ::core::ffi::c_void, n: usize, ) -> *mut ::core::ffi::c_void;
    fn strcmp(s1: *const core::ffi::c_char, s2: *const core::ffi::c_char) -> core::ffi::c_int;
    fn strcpy( dest: *mut core::ffi::c_char, src: *const core::ffi::c_char, ) -> *mut core::ffi::c_char;
    fn strcat( dest: *mut core::ffi::c_char, src: *const core::ffi::c_char, ) -> *mut core::ffi::c_char;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut BusLogic_HostAdapterBusTypes: [BusLogic_HostAdapterBusType_T; 6] = [
    BusLogic_VESA_Bus,
    BusLogic_ISA_Bus,
    BusLogic_MCA_Bus,
    BusLogic_EISA_Bus,
    BusLogic_Unknown_Bus,
    BusLogic_PCI_Bus,
];

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(mut DriverObject: PVOID, mut Argument2: PVOID) -> ULONG {

    const TRACE: ULONG = 3;
    const INFO: ULONG = 4;
    const TRUE: i32 = 1;

    let mut hwInitializationData: HW_INITIALIZATION_DATA = core::mem::zeroed();
    let mut Status: ULONG = 0;
    let mut i: ULONG = 0;
    let mut HwContext: ULONG = 0;

    let mut VendorId: [UCHAR; 4] = [
        '1' as i32 as UCHAR,
        '0' as i32 as UCHAR,
        '4' as i32 as UCHAR,
        'b' as i32 as UCHAR,
    ];
    let mut DeviceId: [UCHAR; 4] = [
        '1' as i32 as UCHAR,
        '0' as i32 as UCHAR,
        '4' as i32 as UCHAR,
        '0' as i32 as UCHAR,
    ];

    ScsiDebugPrint(
        TRACE,
        b"\n BusLogic -  Inside the DriverEntry function \n\0" as *const u8 as *const i8 as PCCHAR,
    );

    i = 0;
    while (i as usize) < core::mem::size_of::<HW_INITIALIZATION_DATA>() {
        *(core::ptr::addr_of_mut!(hwInitializationData) as PUCHAR).offset(i as isize) = 0;
        i = i.wrapping_add(1);
    }

    hwInitializationData.HwInitializationDataSize =
        core::mem::size_of::<HW_INITIALIZATION_DATA>() as ULONG;

    // NTAPI callback slots. These went through a `transmute(f as usize)` until the bindgen
    // islands were normalized to `extern "system"` (tools/rustconv/normalize_island_abi.py) —
    // and that transmute erased the calling convention, which is exactly how all six of these
    // shipped defined `extern "C"` into slots scsiport dereferences as stdcall. Plain `Some(f)`
    // now type-checks the convention, so that bug can only ever be a compile error again.
    hwInitializationData.HwInitialize = Some(BT958HwInitialize);
    hwInitializationData.HwResetBus = Some(BT958HwResetBus);
    hwInitializationData.HwStartIo = Some(BT958HwStartIO);
    hwInitializationData.HwInterrupt = Some(BT958HwInterrupt);
    hwInitializationData.HwAdapterControl = Some(BT958HwAdapterControl);
    hwInitializationData.HwFindAdapter = Some(BT958HwFindAdapter);

    hwInitializationData.NeedPhysicalAddresses = TRUE as BOOLEAN;

    hwInitializationData.AutoRequestSense = TRUE as BOOLEAN;
    hwInitializationData.MultipleRequestPerLu = TRUE as BOOLEAN;

    hwInitializationData.TaggedQueuing = TRUE as BOOLEAN;

    hwInitializationData.AdapterInterfaceType = PCIBus;

    hwInitializationData.VendorId = core::ptr::addr_of_mut!(VendorId) as PVOID;
    hwInitializationData.VendorIdLength = 4 as USHORT;
    hwInitializationData.DeviceId = core::ptr::addr_of_mut!(DeviceId) as PVOID;
    hwInitializationData.DeviceIdLength = 4 as USHORT;

    hwInitializationData.NumberOfAccessRanges = 2 as ULONG;

    hwInitializationData.DeviceExtensionSize =
        core::mem::size_of::<HW_DEVICE_EXTENSION>() as ULONG;

    hwInitializationData.SrbExtensionSize = core::mem::size_of::<BusLogic_CCB_T>() as ULONG;

    HwContext = 0;

    ScsiDebugPrint(
        TRACE,
        b"\n BusLogic -  Calling the ScsiPortInitialize Routine\n\0" as *const u8 as *const i8
            as PCCHAR,
    );

    Status = ScsiPortInitialize(
        DriverObject,
        Argument2,
        core::ptr::addr_of_mut!(hwInitializationData),
        core::ptr::addr_of_mut!(HwContext) as PVOID,
    );

    ScsiDebugPrint(
        TRACE,
        b"\n BusLogic -  Exiting the DriverEntry function \n\0" as *const u8 as *const i8 as PCCHAR,
    );
    ScsiDebugPrint(
        INFO,
        b"\n BusLogic - Status = %ul \n\0" as *const u8 as *const i8 as PCCHAR,
        Status,
    );
    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn BT958HwFindAdapter(
    HwDeviceExtension: PVOID,
    Context: PVOID,
    BusInformation: PVOID,
    ArgumentString: PCHAR,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    Again: PBOOLEAN,
) -> ULONG {

    const TRACE: ULONG = 3;
    const INFO: ULONG = 4;
    const TRUE: BOOLEAN = 1;
    const FALSE: BOOLEAN = 0;
    const SP_RETURN_NOT_FOUND: ULONG = 0;
    const SP_RETURN_ERROR: ULONG = 2;
    const SP_RETURN_FOUND: ULONG = 1;
    const MAX_TRANSFER_SIZE: ULONG = 0x10000; // MAX_TRANSFER_SIZE
    const SP_INTERNAL_ADAPTER_ERROR: ULONG = 0x06; // SP_INTERNAL_ADAPTER_ERROR

    let mut deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut hcsp: *mut BusLogic_HostAdapter_T = core::ptr::addr_of_mut!((*deviceExtension).hcs);
    let mut accessRange: PACCESS_RANGE = core::ptr::null_mut();
    let mut pciAddress: PUCHAR = core::ptr::null_mut();
    let mut NumPort: core::ffi::c_char = 0;

    ScsiDebugPrint(
        TRACE,
        b"\n BusLogic -  Inside the Find Adapter Routine\n\0" as *const u8 as PCCHAR,
    );

    *Again = FALSE as core::ffi::c_uchar;

    accessRange = (core::ptr::addr_of_mut!(*(*ConfigInfo).AccessRanges) as *mut ACCESS_RANGE)
        .offset(0) as PACCESS_RANGE;

    (*ConfigInfo).WmiDataProvider = FALSE;

    if (*(core::ptr::addr_of_mut!(*(*ConfigInfo).AccessRanges) as *mut ACCESS_RANGE).offset(0))
        .RangeLength
        != 0
    {
        if ScsiPortValidateRange(
            deviceExtension as PVOID,
            (*ConfigInfo).AdapterInterfaceType,
            (*ConfigInfo).SystemIoBusNumber,
            (*accessRange).RangeStart,
            (*accessRange).RangeLength,
            TRUE,
        ) != 0
        {
            ScsiDebugPrint(
                INFO,
                b"\n BusLogic - Validate Range function succeeded \n\0" as *const u8 as PCCHAR,
            );

            pciAddress = ScsiPortGetDeviceBase(
                deviceExtension as PVOID,
                (*ConfigInfo).AdapterInterfaceType,
                (*ConfigInfo).SystemIoBusNumber,
                (*accessRange).RangeStart,
                (*accessRange).RangeLength,
                TRUE,
            ) as PUCHAR;

            if !pciAddress.is_null() {
                ScsiDebugPrint(
                    INFO,
                    b"\n BusLogic -  Get Device Base  function succeeded \n\0" as *const u8
                        as PCCHAR,
                );

                memset(
                    hcsp as *mut core::ffi::c_void,
                    0,
                    core::mem::size_of::<BusLogic_HostAdapter_T>() as usize,
                );

                (*hcsp).IO_Address = pciAddress;
                (*hcsp).IRQ_Channel = (*ConfigInfo).BusInterruptLevel as UCHAR;
                NumPort += 1;
            }
        }
    }

    if NumPort as core::ffi::c_int == 0 {
        return SP_RETURN_NOT_FOUND;
    }

    (*ConfigInfo).NumberOfBuses = 1 as UCHAR;
    (*ConfigInfo).MaximumTransferLength = MAX_TRANSFER_SIZE;

    (*ConfigInfo).ScatterGather = TRUE;

    (*ConfigInfo).Master = TRUE;
    (*ConfigInfo).NeedPhysicalAddresses = TRUE;
    (*ConfigInfo).Dma32BitAddresses = TRUE;
    (*ConfigInfo).InterruptMode = LevelSensitive;

    (*ConfigInfo).TaggedQueuing = TRUE;

    (*ConfigInfo).AlignmentMask = 0;

    if Buslogic_InitBT958(deviceExtension, ConfigInfo) == 0 {
        ScsiPortLogError(
            deviceExtension as PVOID,
            core::ptr::null_mut(),
            0 as UCHAR,
            0 as UCHAR,
            0 as UCHAR,
            SP_INTERNAL_ADAPTER_ERROR,
            (7 << 8) as ULONG,
        );

        return SP_RETURN_ERROR;
    }

    if NumPort as core::ffi::c_int != 0 {
        *Again = TRUE as core::ffi::c_uchar;
    }

    return SP_RETURN_FOUND;
}

#[no_mangle]
pub unsafe extern "C" fn Buslogic_InitBT958(
    mut deviceExtension: PHW_DEVICE_EXTENSION,
    mut ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
) -> BOOLEAN {
    const SP_INTERNAL_ADAPTER_ERROR: u32 = 0x06;

    let mut ch: CHAR = 0;
    let mut HostAdapter: *mut BusLogic_HostAdapter_T = core::ptr::addr_of_mut!((*deviceExtension).hcs);
    if BusLogic_ProbeHostAdapter(HostAdapter) == 0 {
        return 0 as BOOLEAN;
    }
    if BusLogic_HardwareResetHostAdapter(HostAdapter, 1 as BOOLEAN) == 0 {
        return 0 as BOOLEAN;
    }
    ch = 0 as CHAR;
    if BusLogic_Command(
        HostAdapter,
        BusLogic_DisableHostAdapterInterrupt,
        core::ptr::addr_of_mut!(ch) as *mut core::ffi::c_void,
        core::mem::size_of::<CHAR>() as core::ffi::c_int,
        core::ptr::null_mut(),
        0 as core::ffi::c_int,
    ) < 0 as core::ffi::c_int
    {
        ScsiDebugPrint(
            2 as ULONG,
            b"\n BusLogic - Could not disable interrupts!\n\0" as *const u8 as *const core::ffi::c_char as PCCHAR,
        );
    } else {
        ScsiDebugPrint(
            4 as ULONG,
            b"\n BusLogic - Disabled interrupts.\n\0" as *const u8 as *const core::ffi::c_char as PCCHAR,
        );
    }
    if BusLogic_CheckHostAdapter(HostAdapter) == 0 {
        return 0 as BOOLEAN;
    }
    (*deviceExtension).NoncachedExtension = ScsiPortGetUncachedExtension(
        deviceExtension as PVOID,
        ConfigInfo,
        core::mem::size_of::<NONCACHED_EXTENSION>() as ULONG,
    ) as PNONCACHED_EXTENSION;
    if (*deviceExtension).NoncachedExtension.is_null() {
        ScsiPortLogError(
            deviceExtension as PVOID,
            core::ptr::null_mut::<_SCSI_REQUEST_BLOCK>(),
            0 as UCHAR,
            0 as UCHAR,
            0 as UCHAR,
            SP_INTERNAL_ADAPTER_ERROR as ULONG,
            ((7 as core::ffi::c_int) << 8 as core::ffi::c_int) as ULONG,
        );
        return 0 as BOOLEAN;
    }
    if BusLogic_ReadHostAdapterConfiguration(HostAdapter) as core::ffi::c_int != 0
        && BusLogic_InitializeHostAdapter(deviceExtension, ConfigInfo) as core::ffi::c_int != 0
        && BusLogic_TargetDeviceInquiry(HostAdapter) as core::ffi::c_int != 0
    {
        (*ConfigInfo).MaximumNumberOfTargets = (*HostAdapter).MaxTargetDevices;
        (*ConfigInfo).MaximumNumberOfLogicalUnits = (*HostAdapter).MaxLogicalUnits;
        (*ConfigInfo).InitiatorBusId[0 as core::ffi::c_int as usize] = (*HostAdapter).SCSI_ID;
        (*ConfigInfo).NumberOfPhysicalBreaks = (*HostAdapter).DriverScatterGatherLimit as ULONG;
    } else {
        return 0 as BOOLEAN;
    }
    return 1 as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn BusLogic_ProbeHostAdapter(
    mut HostAdapter: *mut BusLogic_HostAdapter_T,
) -> BOOLEAN {
    let mut StatusRegister: BusLogic_StatusRegister_T = core::mem::zeroed();
    let mut InterruptRegister: BusLogic_InterruptRegister_T = core::mem::zeroed();
    let mut GeometryRegister: BusLogic_GeometryRegister_T = core::mem::zeroed();
    ScsiDebugPrint(
        3 as ULONG,
        b"\n BusLogic -  Inside ProbeHostaAdapter function \n\0" as *const u8
            as *const ::core::ffi::c_char as PCCHAR,
    );
    *InterruptRegister.All.as_mut() = BusLogic_ReadInterruptRegister(HostAdapter);
    *GeometryRegister.All.as_mut() = BusLogic_ReadGeometryRegister(HostAdapter);
    *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
    if (*StatusRegister.All.as_ref()) as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || StatusRegister.Bits.as_ref().DiagnosticActive() as ::core::ffi::c_int != 0
        || StatusRegister.Bits.as_ref().CommandParameterRegisterBusy() as ::core::ffi::c_int != 0
        || StatusRegister.Bits.as_ref().Reserved() as ::core::ffi::c_int != 0
        || StatusRegister.Bits.as_ref().CommandInvalid() as ::core::ffi::c_int != 0
        || InterruptRegister.Bits.as_ref().Reserved() as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        return 0 as BOOLEAN;
    }
    if (*GeometryRegister.All.as_ref()) as ::core::ffi::c_int == 0xff as ::core::ffi::c_int {
        return 0 as BOOLEAN;
    }
    return 1 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "C" fn BusLogic_HardwareResetHostAdapter(
    HostAdapter: *mut BusLogic_HostAdapter_T,
    HardReset: BOOLEAN,
) -> BOOLEAN {
    const ERROR: ULONG = 1; // nolint: const-contradiction (BusLogic958.h:55 `#define ERROR 1` debug level, not wingdi ERROR=0)

    let mut StatusRegister: BusLogic_StatusRegister_T = core::mem::zeroed();
    let mut TimeoutCounter: ::core::ffi::c_int;

    if HardReset != 0 {
        BusLogic_HardReset(HostAdapter);
    } else {
        BusLogic_SoftReset(HostAdapter);
    }

    TimeoutCounter = 100;
    loop {
        TimeoutCounter -= 1;
        if !(TimeoutCounter >= 0) {
            break;
        }
        *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
        if StatusRegister.Bits.as_ref().DiagnosticActive() != 0 {
            break;
        }
    }

    if TimeoutCounter < 0 {
        return 0;
    }

    ScsiPortStallExecution(100 as ULONG);

    TimeoutCounter = 10 * 10000;
    loop {
        TimeoutCounter -= 1;
        if !(TimeoutCounter >= 0) {
            break;
        }
        *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
        if StatusRegister.Bits.as_ref().DiagnosticActive() == 0 {
            break;
        }
        ScsiPortStallExecution(100 as ULONG);
    }

    if TimeoutCounter < 0 {
        return 0;
    }

    TimeoutCounter = 10000;
    loop {
        TimeoutCounter -= 1;
        if !(TimeoutCounter >= 0) {
            break;
        }
        *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
        if StatusRegister.Bits.as_ref().DiagnosticFailure() as ::core::ffi::c_int != 0
            || StatusRegister.Bits.as_ref().HostAdapterReady() as ::core::ffi::c_int != 0
            || StatusRegister.Bits.as_ref().DataInRegisterReady() as ::core::ffi::c_int != 0
        {
            break;
        }
        ScsiPortStallExecution(100 as ULONG);
    }

    if TimeoutCounter < 0 {
        return 0;
    }

    if StatusRegister.Bits.as_ref().DiagnosticFailure() as ::core::ffi::c_int != 0
        || StatusRegister.Bits.as_ref().HostAdapterReady() == 0
    {
        ScsiDebugPrint(
            ERROR,
            b"\n BusLogic - Failure - HOST ADAPTER STATUS REGISTER = %02X\n\0" as *const u8
                as *const ::core::ffi::c_char as PCCHAR,
            (*StatusRegister.All.as_ref()) as ::core::ffi::c_int,
        );

        if StatusRegister.Bits.as_ref().DataInRegisterReady() != 0 {
            ScsiDebugPrint(
                ERROR,
                b"HOST ADAPTER ERROR CODE = %d\n\0" as *const u8 as *const ::core::ffi::c_char
                    as PCCHAR,
                BusLogic_ReadDataInRegister(HostAdapter) as ::core::ffi::c_int,
            );
        }
        return 0;
    }

    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn BusLogic_CheckHostAdapter(
    HostAdapter: *mut BusLogic_HostAdapter_T,
) -> BOOLEAN {
    let mut ExtendedSetupInformation: BusLogic_ExtendedSetupInformation_T =
        core::mem::zeroed();
    let mut RequestedReplyLength: BusLogic_RequestedReplyLength_T = 0;
    let mut Result: BOOLEAN = TRUE as BOOLEAN;
    ScsiDebugPrint(
        3 as ULONG,
        b"\n BusLogic -  Inside BusLogic_CheckHostAdapter function \n\0" as *const u8
            as *const core::ffi::c_char as PCCHAR,
    );
    RequestedReplyLength = core::mem::size_of::<BusLogic_ExtendedSetupInformation_T>()
        as BusLogic_RequestedReplyLength_T;
    if BusLogic_Command(
        HostAdapter,
        BusLogic_InquireExtendedSetupInformation,
        core::ptr::addr_of_mut!(RequestedReplyLength) as *mut core::ffi::c_void,
        core::mem::size_of::<BusLogic_RequestedReplyLength_T>() as core::ffi::c_int,
        core::ptr::addr_of_mut!(ExtendedSetupInformation) as *mut core::ffi::c_void,
        core::mem::size_of::<BusLogic_ExtendedSetupInformation_T>() as core::ffi::c_int,
    ) as usize
        != core::mem::size_of::<BusLogic_ExtendedSetupInformation_T>() as usize
    {
        Result = FALSE as BOOLEAN;
    }
    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn BusLogic_Command(
    HostAdapter: *mut BusLogic_HostAdapter_T,
    OperationCode: BusLogic_OperationCode_T,
    ParameterData: *mut core::ffi::c_void,
    mut ParameterLength: core::ffi::c_int,
    ReplyData: *mut core::ffi::c_void,
    ReplyLength: core::ffi::c_int,
) -> core::ffi::c_int {

    let mut ParameterPointer: *mut UCHAR = ParameterData as *mut UCHAR;
    let mut ReplyPointer: *mut UCHAR = ReplyData as *mut UCHAR;
    let mut StatusRegister: BusLogic_StatusRegister_T = core::mem::zeroed();
    let mut InterruptRegister: BusLogic_InterruptRegister_T = core::mem::zeroed();
    let mut ReplyBytes: core::ffi::c_int = 0;
    let Result: core::ffi::c_int;
    let mut TimeoutCounter: core::ffi::c_long;

    //  Clear out the Reply Data if provided.
    if ReplyLength > 0 {
        memset(ReplyData, 0, ReplyLength as usize);
    }

    //  Wait for the Host Adapter Ready bit to be set and the Command/Parameter
    //  Register Busy bit to be reset in the Status Register.
    TimeoutCounter = 10000;
    loop {
        TimeoutCounter -= 1;
        if !(TimeoutCounter >= 0) {
            break;
        }
        *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
        if StatusRegister.Bits.as_ref().HostAdapterReady() as core::ffi::c_int != 0
            && StatusRegister.Bits.as_ref().CommandParameterRegisterBusy() == 0
        {
            break;
        }
        ScsiPortStallExecution(100);
    }
    if TimeoutCounter < 0 {
        Result = -2;
    } else {
        //  Write the OperationCode to the Command/Parameter Register.
        (*HostAdapter).HostAdapterCommandCompleted = FALSE as BOOLEAN;
        BusLogic_WriteCommandParameterRegister(HostAdapter, OperationCode as UCHAR);

        // Write any additional Parameter Bytes.
        TimeoutCounter = 10000;
        while ParameterLength > 0 && {
            TimeoutCounter -= 1;
            TimeoutCounter >= 0
        } {
            ScsiPortStallExecution(100);
            *InterruptRegister.All.as_mut() = BusLogic_ReadInterruptRegister(HostAdapter);
            *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
            if InterruptRegister.Bits.as_ref().CommandComplete() != 0 {
                break;
            }
            if (*HostAdapter).HostAdapterCommandCompleted != 0 {
                break;
            }
            if StatusRegister.Bits.as_ref().DataInRegisterReady() != 0 {
                break;
            }
            if StatusRegister.Bits.as_ref().CommandParameterRegisterBusy() != 0 {
                continue;
            }
            let fresh0 = ParameterPointer;
            ParameterPointer = ParameterPointer.offset(1);
            BusLogic_WriteCommandParameterRegister(HostAdapter, *fresh0);
            ParameterLength -= 1;
        }
        if TimeoutCounter < 0 {
            Result = -2;
        } else if OperationCode as core::ffi::c_uint
            == BusLogic_ModifyIOAddress as core::ffi::c_int as core::ffi::c_uint
        {
            // The Modify I/O Address command does not cause a Command Complete Interrupt.
            *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
            if StatusRegister.Bits.as_ref().CommandInvalid() != 0 {
                Result = -1;
            } else {
                Result = 0;
            }
        } else {
            //  Select an appropriate timeout value for awaiting command completion.
            match OperationCode as core::ffi::c_uint {
                10 | 35 | 36 => {
                    TimeoutCounter = (60 * 10000) as core::ffi::c_long;
                }
                _ => {
                    TimeoutCounter = 10000;
                }
            }
            //  Receive any Reply Bytes.
            loop {
                TimeoutCounter -= 1;
                if !(TimeoutCounter >= 0) {
                    break;
                }
                *InterruptRegister.All.as_mut() = BusLogic_ReadInterruptRegister(HostAdapter);
                *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
                if InterruptRegister.Bits.as_ref().CommandComplete() != 0 {
                    break;
                }
                if (*HostAdapter).HostAdapterCommandCompleted != 0 {
                    break;
                }
                if StatusRegister.Bits.as_ref().DataInRegisterReady() != 0 {
                    ReplyBytes += 1;
                    if ReplyBytes <= ReplyLength {
                        let fresh1 = ReplyPointer;
                        ReplyPointer = ReplyPointer.offset(1);
                        *fresh1 = BusLogic_ReadDataInRegister(HostAdapter);
                    } else {
                        BusLogic_ReadDataInRegister(HostAdapter);
                    }
                }
                if OperationCode as core::ffi::c_uint
                    == BusLogic_FetchHostAdapterLocalRAM as core::ffi::c_int as core::ffi::c_uint
                    && StatusRegister.Bits.as_ref().HostAdapterReady() as core::ffi::c_int != 0
                {
                    break;
                }
                ScsiPortStallExecution(100);
            }
            if TimeoutCounter < 0 {
                Result = -2;
            } else {
                //  Clear any pending Command Complete Interrupt.
                BusLogic_InterruptReset(HostAdapter);

                //  Process Command Invalid conditions.
                if StatusRegister.Bits.as_ref().CommandInvalid() != 0 {
                    ScsiPortStallExecution(1000);
                    *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
                    if StatusRegister.Bits.as_ref().CommandInvalid() as core::ffi::c_int != 0
                        || StatusRegister.Bits.as_ref().Reserved() as core::ffi::c_int != 0
                        || StatusRegister.Bits.as_ref().DataInRegisterReady() as core::ffi::c_int != 0
                        || StatusRegister.Bits.as_ref().CommandParameterRegisterBusy() as core::ffi::c_int
                            != 0
                        || StatusRegister.Bits.as_ref().HostAdapterReady() == 0
                        || StatusRegister.Bits.as_ref().InitializationRequired() == 0
                        || StatusRegister.Bits.as_ref().DiagnosticActive() as core::ffi::c_int != 0
                        || StatusRegister.Bits.as_ref().DiagnosticFailure() as core::ffi::c_int != 0
                    {
                        BusLogic_SoftReset(HostAdapter);
                        ScsiPortStallExecution(1000);
                    }
                    Result = -1;
                } else if ParameterLength > 0 {
                    // Handle Excess Parameters Supplied conditions.
                    Result = -1;
                } else {
                    //  Indicate the command completed successfully.
                    Result = ReplyBytes;
                }
            }
        }
    }
    return Result;
}

#[no_mangle]
pub unsafe extern "C" fn BusLogic_ReadHostAdapterConfiguration(
    HostAdapter: *mut BusLogic_HostAdapter_T,
) -> BOOLEAN {

    let mut BoardID: BusLogic_BoardID_T = core::mem::zeroed();
    let mut Configuration: BusLogic_Configuration_T = core::mem::zeroed();
    let mut SetupInformation: BusLogic_SetupInformation_T = core::mem::zeroed();
    let mut ExtendedSetupInformation: BusLogic_ExtendedSetupInformation_T = core::mem::zeroed();
    let mut HostAdapterModelNumber: BusLogic_HostAdapterModelNumber_T = core::mem::zeroed();
    let mut FirmwareVersion3rdDigit: BusLogic_FirmwareVersion3rdDigit_T;
    let mut FirmwareVersionLetter: BusLogic_FirmwareVersionLetter_T = 0;
    let mut PCIHostAdapterInformation: BusLogic_PCIHostAdapterInformation_T = core::mem::zeroed();
    let mut FetchHostAdapterLocalRAMRequest: BusLogic_FetchHostAdapterLocalRAMRequest_T =
        core::mem::zeroed();
    let mut AutoSCSIData: BusLogic_AutoSCSIData_T = core::mem::zeroed();
    let mut GeometryRegister: BusLogic_GeometryRegister_T = core::mem::zeroed();
    let mut RequestedReplyLength: BusLogic_RequestedReplyLength_T;
    let mut TargetPointer: *mut UCHAR;
    let mut Character: UCHAR;
    let mut i: ULONG;

    if BusLogic_Command(
        HostAdapter,
        BusLogic_InquireBoardID,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(BoardID) as *mut core::ffi::c_void,
        core::mem::size_of::<BusLogic_BoardID_T>() as core::ffi::c_int,
    ) as usize
        != core::mem::size_of::<BusLogic_BoardID_T>() as usize
    {
        ScsiDebugPrint(
            1 as ULONG,
            b"\n BusLogic - Failure: INQUIRE BOARD ID\n\0" as *const u8 as *const core::ffi::c_char
                as PCCHAR,
        );
        return 0 as BOOLEAN;
    }
    if BusLogic_Command(
        HostAdapter,
        BusLogic_InquireConfiguration,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(Configuration) as *mut core::ffi::c_void,
        core::mem::size_of::<BusLogic_Configuration_T>() as core::ffi::c_int,
    ) as usize
        != core::mem::size_of::<BusLogic_Configuration_T>() as usize
    {
        ScsiDebugPrint(
            1 as ULONG,
            b"\n BusLogic - Failure: INQUIRE CONFIGURATION\n\0" as *const u8
                as *const core::ffi::c_char as PCCHAR,
        );
        return 0 as BOOLEAN;
    }
    RequestedReplyLength =
        core::mem::size_of::<BusLogic_SetupInformation_T>() as BusLogic_RequestedReplyLength_T;
    if BusLogic_Command(
        HostAdapter,
        BusLogic_InquireSetupInformation,
        core::ptr::addr_of_mut!(RequestedReplyLength) as *mut core::ffi::c_void,
        core::mem::size_of::<BusLogic_RequestedReplyLength_T>() as core::ffi::c_int,
        core::ptr::addr_of_mut!(SetupInformation) as *mut core::ffi::c_void,
        core::mem::size_of::<BusLogic_SetupInformation_T>() as core::ffi::c_int,
    ) as usize
        != core::mem::size_of::<BusLogic_SetupInformation_T>() as usize
    {
        ScsiDebugPrint(
            1 as ULONG,
            b"\n BusLogic - Failure: INQUIRE SETUP INFORMATION\n\0" as *const u8
                as *const core::ffi::c_char as PCCHAR,
        );
        return 0 as BOOLEAN;
    }
    RequestedReplyLength = core::mem::size_of::<BusLogic_ExtendedSetupInformation_T>()
        as BusLogic_RequestedReplyLength_T;
    if BusLogic_Command(
        HostAdapter,
        BusLogic_InquireExtendedSetupInformation,
        core::ptr::addr_of_mut!(RequestedReplyLength) as *mut core::ffi::c_void,
        core::mem::size_of::<BusLogic_RequestedReplyLength_T>() as core::ffi::c_int,
        core::ptr::addr_of_mut!(ExtendedSetupInformation) as *mut core::ffi::c_void,
        core::mem::size_of::<BusLogic_ExtendedSetupInformation_T>() as core::ffi::c_int,
    ) as usize
        != core::mem::size_of::<BusLogic_ExtendedSetupInformation_T>() as usize
    {
        ScsiDebugPrint(
            1 as ULONG,
            b"\n BusLogic - Failure: INQUIRE EXTENDED SETUP INFORMATION\n\0" as *const u8
                as *const core::ffi::c_char as PCCHAR,
        );
        return 0 as BOOLEAN;
    }
    FirmwareVersion3rdDigit = b'\0' as BusLogic_FirmwareVersion3rdDigit_T;
    if BoardID.FirmwareVersion1stDigit as core::ffi::c_int > '0' as i32 {
        if BusLogic_Command(
            HostAdapter,
            BusLogic_InquireFirmwareVersion3rdDigit,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(FirmwareVersion3rdDigit) as *mut core::ffi::c_void,
            core::mem::size_of::<BusLogic_FirmwareVersion3rdDigit_T>() as core::ffi::c_int,
        ) as usize
            != core::mem::size_of::<BusLogic_FirmwareVersion3rdDigit_T>() as usize
        {
            ScsiDebugPrint(
                1 as ULONG,
                b"\n BusLogic - Failure: INQUIRE FIRMWARE 3RD DIGIT\n\0" as *const u8
                    as *const core::ffi::c_char as PCCHAR,
            );
            return 0 as BOOLEAN;
        }
    }
    RequestedReplyLength = core::mem::size_of::<BusLogic_HostAdapterModelNumber_T>()
        as BusLogic_RequestedReplyLength_T;
    if BusLogic_Command(
        HostAdapter,
        BusLogic_InquireHostAdapterModelNumber,
        core::ptr::addr_of_mut!(RequestedReplyLength) as *mut core::ffi::c_void,
        core::mem::size_of::<BusLogic_RequestedReplyLength_T>() as core::ffi::c_int,
        core::ptr::addr_of_mut!(HostAdapterModelNumber) as *mut core::ffi::c_void,
        core::mem::size_of::<BusLogic_HostAdapterModelNumber_T>() as core::ffi::c_int,
    ) as usize
        != core::mem::size_of::<BusLogic_HostAdapterModelNumber_T>() as usize
    {
        ScsiDebugPrint(
            1 as ULONG,
            b"\n BusLogic - Failure: INQUIRE HOST ADAPTER MODEL NUMBER\n\0" as *const u8
                as *const core::ffi::c_char as PCCHAR,
        );
        return 0 as BOOLEAN;
    }
    TargetPointer = core::ptr::addr_of_mut!((*HostAdapter).ModelName) as *mut UCHAR;
    let fresh2 = TargetPointer;
    TargetPointer = TargetPointer.offset(1);
    *fresh2 = 'B' as i32 as UCHAR;
    let fresh3 = TargetPointer;
    TargetPointer = TargetPointer.offset(1);
    *fresh3 = 'T' as i32 as UCHAR;
    let fresh4 = TargetPointer;
    TargetPointer = TargetPointer.offset(1);
    *fresh4 = '-' as i32 as UCHAR;
    i = 0 as ULONG;
    while (i as usize) < core::mem::size_of::<BusLogic_HostAdapterModelNumber_T>() as usize {
        Character = HostAdapterModelNumber[i as usize];
        if Character as core::ffi::c_int == ' ' as i32
            || Character as core::ffi::c_int == '\0' as i32
        {
            break;
        }
        let fresh5 = TargetPointer;
        TargetPointer = TargetPointer.offset(1);
        *fresh5 = Character;
        i = i.wrapping_add(1);
    }
    let fresh6 = TargetPointer;
    TargetPointer = TargetPointer.offset(1);
    *fresh6 = '\0' as i32 as UCHAR;
    TargetPointer = core::ptr::addr_of_mut!((*HostAdapter).FirmwareVersion) as *mut UCHAR;
    let fresh7 = TargetPointer;
    TargetPointer = TargetPointer.offset(1);
    *fresh7 = BoardID.FirmwareVersion1stDigit;
    let fresh8 = TargetPointer;
    TargetPointer = TargetPointer.offset(1);
    *fresh8 = '.' as i32 as UCHAR;
    let fresh9 = TargetPointer;
    TargetPointer = TargetPointer.offset(1);
    *fresh9 = BoardID.FirmwareVersion2ndDigit;
    if FirmwareVersion3rdDigit as core::ffi::c_int != ' ' as i32
        && FirmwareVersion3rdDigit as core::ffi::c_int != '\0' as i32
    {
        let fresh10 = TargetPointer;
        TargetPointer = TargetPointer.offset(1);
        *fresh10 = FirmwareVersion3rdDigit as UCHAR;
    }
    *TargetPointer = '\0' as i32 as UCHAR;
    if strcmp(
        core::ptr::addr_of_mut!((*HostAdapter).FirmwareVersion) as *mut UCHAR
            as *mut core::ffi::c_char,
        b"3.3\0" as *const u8 as *const core::ffi::c_char,
    ) >= 0
    {
        if BusLogic_Command(
            HostAdapter,
            BusLogic_InquireFirmwareVersionLetter,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(FirmwareVersionLetter) as *mut core::ffi::c_void,
            core::mem::size_of::<BusLogic_FirmwareVersionLetter_T>() as core::ffi::c_int,
        ) as usize
            != core::mem::size_of::<BusLogic_FirmwareVersionLetter_T>() as usize
        {
            ScsiDebugPrint(
                1 as ULONG,
                b"\n BusLogic - Failure: INQUIRE FIRMWARE VERSION LETTER\n\0" as *const u8
                    as *const core::ffi::c_char as PCCHAR,
            );
            return 0 as BOOLEAN;
        }
        if FirmwareVersionLetter as core::ffi::c_int != ' ' as i32
            && FirmwareVersionLetter as core::ffi::c_int != '\0' as i32
        {
            let fresh11 = TargetPointer;
            TargetPointer = TargetPointer.offset(1);
            *fresh11 = FirmwareVersionLetter as UCHAR;
        }
        *TargetPointer = '\0' as i32 as UCHAR;
    }
    (*HostAdapter).SCSI_ID = Configuration.HostAdapterID();
    (*HostAdapter).HostAdapterBusType = BusLogic_HostAdapterBusTypes
        [((*HostAdapter).ModelName[3 as usize] as core::ffi::c_int - '4' as i32) as usize];
    *GeometryRegister.All.as_mut() = BusLogic_ReadGeometryRegister(HostAdapter);
    (*HostAdapter).set_ExtendedTranslationEnabled((*GeometryRegister.All.as_ref()) as BOOLEAN);
    (*HostAdapter).HostAdapterScatterGatherLimit = ExtendedSetupInformation.ScatterGatherLimit;
    (*HostAdapter).DriverScatterGatherLimit = (*HostAdapter).HostAdapterScatterGatherLimit;
    if (*HostAdapter).HostAdapterScatterGatherLimit as core::ffi::c_int
        > BusLogic_ScatterGatherLimit as i32
    {
        (*HostAdapter).DriverScatterGatherLimit = BusLogic_ScatterGatherLimit as core::ffi::c_ushort;
    }
    if ExtendedSetupInformation.Misc.LevelSensitiveInterrupt() != 0 {
        (*HostAdapter).set_LevelSensitiveInterrupt(1 as BOOLEAN);
    }
    (*HostAdapter).set_HostWideSCSI(ExtendedSetupInformation.HostWideSCSI() as BOOLEAN);
    (*HostAdapter)
        .set_HostDifferentialSCSI(ExtendedSetupInformation.HostDifferentialSCSI() as BOOLEAN);
    (*HostAdapter).set_HostSupportsSCAM(ExtendedSetupInformation.HostSupportsSCAM() as BOOLEAN);
    (*HostAdapter).set_HostUltraSCSI(ExtendedSetupInformation.HostUltraSCSI() as BOOLEAN);
    if (*HostAdapter).FirmwareVersion[0 as usize] as core::ffi::c_int == '5' as i32
        || (*HostAdapter).FirmwareVersion[0 as usize] as core::ffi::c_int == '4' as i32
            && (*HostAdapter).HostWideSCSI() as core::ffi::c_int != 0
    {
        (*HostAdapter).set_ExtendedLUNSupport(1 as BOOLEAN);
    }
    if (*HostAdapter).FirmwareVersion[0 as usize] as core::ffi::c_int == '5' as i32 {
        if BusLogic_Command(
            HostAdapter,
            BusLogic_InquirePCIHostAdapterInformation,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(PCIHostAdapterInformation) as *mut core::ffi::c_void,
            core::mem::size_of::<BusLogic_PCIHostAdapterInformation_T>() as core::ffi::c_int,
        ) as usize
            != core::mem::size_of::<BusLogic_PCIHostAdapterInformation_T>() as usize
        {
            ScsiDebugPrint(
                1 as ULONG,
                b"\n BusLogic - Failure: INQUIRE PCI HOST ADAPTER INFORMATION\n\0" as *const u8
                    as *const core::ffi::c_char as PCCHAR,
            );
            return 0 as BOOLEAN;
        }
        if PCIHostAdapterInformation.GenericInfoValid() != 0 {
            (*HostAdapter).set_TerminationInfoValid(1 as BOOLEAN);
            (*HostAdapter)
                .set_LowByteTerminated(PCIHostAdapterInformation.LowByteTerminated() as BOOLEAN);
            (*HostAdapter)
                .set_HighByteTerminated(PCIHostAdapterInformation.HighByteTerminated() as BOOLEAN);
        }
    }
    if (*HostAdapter).FirmwareVersion[0 as usize] as core::ffi::c_int >= '4' as i32 {
        FetchHostAdapterLocalRAMRequest.ByteOffset = BusLogic_AutoSCSI_BaseOffset as UCHAR;
        FetchHostAdapterLocalRAMRequest.ByteCount =
            core::mem::size_of::<BusLogic_AutoSCSIData_T>() as UCHAR;
        if BusLogic_Command(
            HostAdapter,
            BusLogic_FetchHostAdapterLocalRAM,
            core::ptr::addr_of_mut!(FetchHostAdapterLocalRAMRequest) as *mut core::ffi::c_void,
            core::mem::size_of::<BusLogic_FetchHostAdapterLocalRAMRequest_T>() as core::ffi::c_int,
            core::ptr::addr_of_mut!(AutoSCSIData) as *mut core::ffi::c_void,
            core::mem::size_of::<BusLogic_AutoSCSIData_T>() as core::ffi::c_int,
        ) as usize
            != core::mem::size_of::<BusLogic_AutoSCSIData_T>() as usize
        {
            ScsiDebugPrint(
                1 as ULONG,
                b"\n BusLogic - Failure: FETCH HOST ADAPTER LOCAL RAM\n\0" as *const u8
                    as *const core::ffi::c_char as PCCHAR,
            );
            return 0 as BOOLEAN;
        }
        (*HostAdapter).set_ParityCheckingEnabled(AutoSCSIData.ParityCheckingEnabled() as BOOLEAN);
        (*HostAdapter).set_BusResetEnabled(AutoSCSIData.BusResetEnabled() as BOOLEAN);
        (*HostAdapter).WidePermitted = AutoSCSIData.WidePermitted();
        (*HostAdapter).FastPermitted = AutoSCSIData.FastPermitted();
        (*HostAdapter).SynchronousPermitted = AutoSCSIData.SynchronousPermitted();
        (*HostAdapter).DisconnectPermitted = AutoSCSIData.DisconnectPermitted();
        if (*HostAdapter).HostUltraSCSI() != 0 {
            (*HostAdapter).UltraPermitted = AutoSCSIData.UltraPermitted();
        }
        if (*HostAdapter).HostSupportsSCAM() != 0 {
            (*HostAdapter).set_SCAM_Enabled(AutoSCSIData.SCAM_Enabled() as BOOLEAN);
            (*HostAdapter).set_SCAM_Level2(AutoSCSIData.SCAM_Level2() as BOOLEAN);
        }
    }
    (*HostAdapter).MaxTargetDevices = (if (*HostAdapter).HostWideSCSI() as core::ffi::c_int != 0 {
        16 as core::ffi::c_int
    } else {
        8 as core::ffi::c_int
    }) as UCHAR;
    (*HostAdapter).MaxLogicalUnits =
        (if (*HostAdapter).ExtendedLUNSupport() as core::ffi::c_int != 0 {
            32 as core::ffi::c_int
        } else {
            8 as core::ffi::c_int
        }) as UCHAR;
    if (*HostAdapter).FirmwareVersion[0 as usize] as core::ffi::c_int == '5' as i32 {
        (*HostAdapter).HostAdapterQueueDepth = 192 as core::ffi::c_ushort;
    }
    if strcmp(
        core::ptr::addr_of_mut!((*HostAdapter).FirmwareVersion) as *mut UCHAR
            as *mut core::ffi::c_char,
        b"3.31\0" as *const u8 as *const core::ffi::c_char,
    ) >= 0
    {
        (*HostAdapter).set_StrictRoundRobinModeSupport(1 as BOOLEAN);
        (*HostAdapter).MailboxCount = BusLogic_MaxMailboxes as UCHAR;
    }
    (*HostAdapter).TaggedQueuingPermitted = 0xffff as core::ffi::c_ushort;
    (*HostAdapter).BIOS_Address = ((ExtendedSetupInformation.BIOS_Address as core::ffi::c_int)
        << 12 as core::ffi::c_int) as BusLogic_BusAddress_T;
    strcpy(
        core::ptr::addr_of_mut!((*HostAdapter).FullModelName) as *mut UCHAR
            as *mut core::ffi::c_char,
        b"BusLogic \0" as *const u8 as *const core::ffi::c_char,
    );
    strcat(
        core::ptr::addr_of_mut!((*HostAdapter).FullModelName) as *mut UCHAR
            as *mut core::ffi::c_char,
        core::ptr::addr_of_mut!((*HostAdapter).ModelName) as *mut UCHAR as *mut core::ffi::c_char,
    );
    (*HostAdapter).TaggedQueuingPermitted = ((*HostAdapter).TaggedQueuingPermitted
        as core::ffi::c_int
        & (*HostAdapter).DisconnectPermitted as core::ffi::c_int)
        as core::ffi::c_ushort;
    (*HostAdapter).BusSettleTime = BusLogic_DefaultBusSettleTime as core::ffi::c_ushort;
    return 1 as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn BusLogic_InitializeHostAdapter(
    deviceExtension: PHW_DEVICE_EXTENSION,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
) -> BOOLEAN {

    let HostAdapter: *mut BusLogic_HostAdapter_T = core::ptr::addr_of_mut!((*deviceExtension).hcs);
    let mut ExtendedMailboxRequest: BusLogic_ExtendedMailboxRequest_T =
        BusLogic_ExtendedMailboxRequest_T {
            MailboxCount: 0,
            BaseMailboxAddress: 0,
        };
    let mut RoundRobinModeRequest: UCHAR;
    let mut SetCCBFormatRequest: UCHAR;
    let mut TargetID: core::ffi::c_int;
    let mut LunID: core::ffi::c_int;
    let mut length: ULONG = 0;

    (*HostAdapter).FirstCompletedCCB = core::ptr::null_mut();
    (*HostAdapter).LastCompletedCCB = core::ptr::null_mut();

    TargetID = 0;
    while TargetID < (*HostAdapter).MaxTargetDevices as core::ffi::c_int {
        (*HostAdapter).BusDeviceResetPendingCCB[TargetID as usize] = core::ptr::null_mut();
        (*HostAdapter).TargetFlags[TargetID as usize].set_TaggedQueuingActive(0 as BOOLEAN);
        (*HostAdapter).TargetFlags[TargetID as usize].set_CommandSuccessfulFlag(0 as BOOLEAN);

        (*HostAdapter).ActiveCommandsPerTarget[TargetID as usize] = 0;
        LunID = 0;
        while LunID < (*HostAdapter).MaxLogicalUnits as core::ffi::c_int {
            (*HostAdapter).ActiveCommandsPerLun[TargetID as usize][LunID as usize] = 0;
            LunID += 1;
        }
        (*HostAdapter).CommandsSinceReset[TargetID as usize] = 0;
        TargetID += 1;
    }

    (*(*deviceExtension).NoncachedExtension).MailboxPA = ScsiPortGetPhysicalAddress(
        deviceExtension as PVOID,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*(*deviceExtension).NoncachedExtension).MailboxOut) as *mut UCHAR
            as PVOID,
        core::ptr::addr_of_mut!(length),
    )
    .__bindgen_anon_1
    .LowPart;

    (*HostAdapter).FirstOutgoingMailbox =
        core::ptr::addr_of_mut!((*(*deviceExtension).NoncachedExtension).MailboxOut) as *mut UCHAR
            as *mut BusLogic_OutgoingMailbox_T;
    (*HostAdapter).LastOutgoingMailbox = (*HostAdapter)
        .FirstOutgoingMailbox
        .offset((*HostAdapter).MailboxCount as core::ffi::c_int as isize)
        .offset(-1);
    (*HostAdapter).NextOutgoingMailbox = (*HostAdapter).FirstOutgoingMailbox;

    (*HostAdapter).FirstIncomingMailbox =
        core::ptr::addr_of_mut!((*(*deviceExtension).NoncachedExtension).MailboxIn) as *mut UCHAR
            as *mut BusLogic_IncomingMailbox_T;
    (*HostAdapter).LastIncomingMailbox = (*HostAdapter)
        .FirstIncomingMailbox
        .offset((*HostAdapter).MailboxCount as core::ffi::c_int as isize)
        .offset(-1);
    (*HostAdapter).NextIncomingMailbox = (*HostAdapter).FirstIncomingMailbox;

    memset(
        (*HostAdapter).FirstOutgoingMailbox as *mut core::ffi::c_void,
        0,
        ((*HostAdapter).MailboxCount as usize)
            .wrapping_mul(core::mem::size_of::<BusLogic_OutgoingMailbox_T>() as usize),
    );
    memset(
        (*HostAdapter).FirstIncomingMailbox as *mut core::ffi::c_void,
        0,
        ((*HostAdapter).MailboxCount as usize)
            .wrapping_mul(core::mem::size_of::<BusLogic_IncomingMailbox_T>() as usize),
    );

    ExtendedMailboxRequest.MailboxCount = (*HostAdapter).MailboxCount;
    ExtendedMailboxRequest.BaseMailboxAddress = (*(*deviceExtension).NoncachedExtension).MailboxPA;

    if BusLogic_Command(
        HostAdapter,
        BusLogic_InitializeExtendedMailbox,
        core::ptr::addr_of_mut!(ExtendedMailboxRequest) as *mut core::ffi::c_void,
        core::mem::size_of::<BusLogic_ExtendedMailboxRequest_T>() as core::ffi::c_int,
        core::ptr::null_mut(),
        0,
    ) < 0
    {
        ScsiDebugPrint(
            1 as ULONG,
            b"\n BusLogic - Failure: MAILBOX INITIALIZATION\n\0" as *const u8 as *const i8
                as PCCHAR,
        );
        return 0 as BOOLEAN;
    }

    if (*HostAdapter).StrictRoundRobinModeSupport() != 0 {
        RoundRobinModeRequest = BusLogic_StrictRoundRobinMode as UCHAR;
        if BusLogic_Command(
            HostAdapter,
            BusLogic_EnableStrictRoundRobinMode,
            core::ptr::addr_of_mut!(RoundRobinModeRequest) as *mut core::ffi::c_void,
            core::mem::size_of::<UCHAR>() as core::ffi::c_int,
            core::ptr::null_mut(),
            0,
        ) < 0
        {
            ScsiDebugPrint(
                1 as ULONG,
                b"\n BusLogic - Failure: ENABLE STRICT ROUND ROBIN MODE\n\0" as *const u8
                    as *const i8 as PCCHAR,
            );
            return 0 as BOOLEAN;
        }
    }

    if (*HostAdapter).ExtendedLUNSupport() != 0 {
        SetCCBFormatRequest = BusLogic_ExtendedLUNFormatCCB as core::ffi::c_int as UCHAR;
        if BusLogic_Command(
            HostAdapter,
            BusLogic_SetCCBFormat,
            core::ptr::addr_of_mut!(SetCCBFormatRequest) as *mut core::ffi::c_void,
            core::mem::size_of::<UCHAR>() as core::ffi::c_int,
            core::ptr::null_mut(),
            0,
        ) < 0
        {
            ScsiDebugPrint(
                1 as ULONG,
                b"\n BusLogic - Failure: SET CCB FORMAT\n\0" as *const u8 as *const i8 as PCCHAR,
            );
            return 0 as BOOLEAN;
        }
    }

    if (*HostAdapter).HostAdapterInitialized() == 0 {
        ScsiDebugPrint(
            4 as ULONG,
            b"\n BusLogic - %s Initialized Successfully\n\0" as *const u8 as *const i8 as PCCHAR,
            HostAdapter,
            core::ptr::addr_of_mut!((*HostAdapter).FullModelName) as *mut UCHAR,
        );
    } else {
        ScsiDebugPrint(
            2 as ULONG,
            b"\n BusLogic - %s not initialized Successfully\n\0" as *const u8 as *const i8
                as PCCHAR,
            HostAdapter,
            core::ptr::addr_of_mut!((*HostAdapter).FullModelName) as *mut UCHAR,
        );
    }
    (*HostAdapter).set_HostAdapterInitialized(1 as BOOLEAN);
    return 1 as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn BusLogic_TargetDeviceInquiry(
    HostAdapter: *mut BusLogic_HostAdapter_T,
) -> BOOLEAN {
    const ERROR: ULONG = 1; // nolint: const-contradiction (BusLogic958.h:55 `#define ERROR 1` debug level, not wingdi ERROR=0)
    let mut InstalledDevices: BusLogic_InstalledDevices_T = 0;
    let mut SetupInformation: BusLogic_SetupInformation_T = core::mem::zeroed();
    let mut SynchronousPeriod: BusLogic_SynchronousPeriod_T = [0; 16];
    let mut RequestedReplyLength: BusLogic_RequestedReplyLength_T = 0;
    let mut TargetID: ::core::ffi::c_int = 0;

    ScsiPortStallExecution((*HostAdapter).BusSettleTime as ULONG);

    if strcmp(
        core::ptr::addr_of_mut!((*HostAdapter).FirmwareVersion) as *mut UCHAR as *mut ::core::ffi::c_char,
        b"4.25\0" as *const u8 as *const ::core::ffi::c_char,
    ) >= 0
    {
        if BusLogic_Command(
            HostAdapter,
            BusLogic_InquireTargetDevices,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(InstalledDevices) as *mut ::core::ffi::c_void,
            core::mem::size_of::<BusLogic_InstalledDevices_T>() as ::core::ffi::c_int,
        ) as usize
            != core::mem::size_of::<BusLogic_InstalledDevices_T>() as usize
        {
            ScsiDebugPrint(
                ERROR,
                b"\n BusLogic - Failure: INQUIRE TARGET DEVICES\n\0" as *const u8
                    as *const ::core::ffi::c_char as PCCHAR,
            );
            return 0 as BOOLEAN;
        }
        TargetID = 0;
        while TargetID < (*HostAdapter).MaxTargetDevices as ::core::ffi::c_int {
            (*HostAdapter).TargetFlags[TargetID as usize].set_TargetExists(
                (if InstalledDevices as ::core::ffi::c_int & (1 << TargetID) != 0 {
                    1
                } else {
                    0
                }) as BOOLEAN,
            );
            TargetID += 1;
        }
    }

    RequestedReplyLength =
        core::mem::size_of::<BusLogic_SetupInformation_T>() as BusLogic_RequestedReplyLength_T;
    if BusLogic_Command(
        HostAdapter,
        BusLogic_InquireSetupInformation,
        core::ptr::addr_of_mut!(RequestedReplyLength) as *mut ::core::ffi::c_void,
        core::mem::size_of::<BusLogic_RequestedReplyLength_T>() as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(SetupInformation) as *mut ::core::ffi::c_void,
        core::mem::size_of::<BusLogic_SetupInformation_T>() as ::core::ffi::c_int,
    ) as usize
        != core::mem::size_of::<BusLogic_SetupInformation_T>() as usize
    {
        ScsiDebugPrint(
            ERROR,
            b"\n BusLogic - Failure: INQUIRE SETUP INFORMATION\n\0" as *const u8
                as *const ::core::ffi::c_char as PCCHAR,
        );
        return 0 as BOOLEAN;
    }

    TargetID = 0;
    while TargetID < (*HostAdapter).MaxTargetDevices as ::core::ffi::c_int {
        (*HostAdapter).SynchronousOffset[TargetID as usize] = (if TargetID < 8 {
            SetupInformation.SynchronousValuesID0to7[TargetID as usize].Offset() as ::core::ffi::c_int
        } else {
            SetupInformation.SynchronousValuesID8to15[(TargetID - 8) as usize].Offset()
                as ::core::ffi::c_int
        }) as UCHAR;
        TargetID += 1;
    }

    if strcmp(
        core::ptr::addr_of_mut!((*HostAdapter).FirmwareVersion) as *mut UCHAR as *mut ::core::ffi::c_char,
        b"5.06L\0" as *const u8 as *const ::core::ffi::c_char,
    ) >= 0
    {
        TargetID = 0;
        while TargetID < (*HostAdapter).MaxTargetDevices as ::core::ffi::c_int {
            (*HostAdapter).TargetFlags[TargetID as usize].set_WideTransfersActive(
                (if TargetID < 8 {
                    if SetupInformation.WideTransfersActiveID0to7 as ::core::ffi::c_int
                        & (1 << TargetID)
                        != 0
                    {
                        1
                    } else {
                        0
                    }
                } else if SetupInformation.WideTransfersActiveID8to15 as ::core::ffi::c_int
                    & (1 << (TargetID - 8))
                    != 0
                {
                    1
                } else {
                    0
                }) as BOOLEAN,
            );
            TargetID += 1;
        }
    }

    if (*HostAdapter).FirmwareVersion[0] as ::core::ffi::c_int >= '3' as i32 {
        RequestedReplyLength =
            core::mem::size_of::<BusLogic_SynchronousPeriod_T>() as BusLogic_RequestedReplyLength_T;
        if BusLogic_Command(
            HostAdapter,
            BusLogic_InquireSynchronousPeriod,
            core::ptr::addr_of_mut!(RequestedReplyLength) as *mut ::core::ffi::c_void,
            core::mem::size_of::<BusLogic_RequestedReplyLength_T>() as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(SynchronousPeriod) as *mut ::core::ffi::c_void,
            core::mem::size_of::<BusLogic_SynchronousPeriod_T>() as ::core::ffi::c_int,
        ) as usize
            != core::mem::size_of::<BusLogic_SynchronousPeriod_T>() as usize
        {
            ScsiDebugPrint(
                ERROR,
                b"\n BusLogic - Failure: INQUIRE SYNCHRONOUS PERIOD\n\0" as *const u8
                    as *const ::core::ffi::c_char as PCCHAR,
            );
            return 0 as BOOLEAN;
        }
        TargetID = 0;
        while TargetID < (*HostAdapter).MaxTargetDevices as ::core::ffi::c_int {
            (*HostAdapter).SynchronousPeriod[TargetID as usize] =
                SynchronousPeriod[TargetID as usize];
            TargetID += 1;
        }
    }

    return 1 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "C" fn BusLogic_InitializeCCB(mut CCB: PBuslogic_CCB_T) {
    (*CCB).Opcode = BusLogic_InitiatorCCB as UCHAR;
    (*CCB).set_DataDirection(0 as UCHAR);
    (*CCB).set_TagEnable(0 as UCHAR);
    (*CCB).set_QueueTag(0 as UCHAR);
    (*CCB).CDB_Length = 0 as UCHAR;
    (*CCB).SenseDataLength = 0 as UCHAR;
    (*CCB).DataLength = 0 as BusLogic_ByteCount_T;
    (*CCB).DataPointer = 0 as BusLogic_BusAddress_T;
    (*CCB).HostAdapterStatus = 0 as UCHAR;
    (*CCB).TargetDeviceStatus = 0 as UCHAR;
    (*CCB).TargetID = 0 as UCHAR;
    (*CCB).set_LogicalUnit(0 as UCHAR);
    (*CCB).set_LegacyTagEnable(0 as UCHAR);
    (*CCB).set_LegacyQueueTag(0 as UCHAR);
    (*CCB).SenseDataPointer = 0 as BusLogic_BusAddress_T;
    (*CCB).Status = BusLogic_CCB_Free;
    (*CCB).SerialNumber = 0 as core::ffi::c_ulong;
    (*CCB).Next = core::ptr::null_mut::<BusLogic_CCB>();
    (*CCB).HostAdapter = core::ptr::null_mut::<BusLogic_HostAdapter>();
    (*CCB).CompletionCode = BusLogic_IncomingMailboxFree;
    (*CCB).SrbAddress = core::ptr::null_mut() as PVOID;
    (*CCB).AbortSrb = core::ptr::null_mut() as PVOID;
}
#[no_mangle]
pub unsafe extern "system" fn BT958HwStartIO(
    mut HwDeviceExtension: PVOID,
    mut Srb: PSCSI_REQUEST_BLOCK,
) -> BOOLEAN {

    const BusLogic_CDB_MaxLength: ::core::ffi::c_int = 12;

    let mut deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut HostAdapter: *mut BusLogic_HostAdapter_T = core::ptr::addr_of_mut!((*deviceExtension).hcs);
    let mut AbortSRB: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut ccb: PBuslogic_CCB_T = core::ptr::null_mut();
    let mut TargetFlags: *mut BusLogic_TargetFlags_T =
        (core::ptr::addr_of_mut!((*HostAdapter).TargetFlags) as *mut BusLogic_TargetFlags_T)
            .offset((*Srb).TargetId as isize) as *mut BusLogic_TargetFlags_T;

    ScsiDebugPrint(
        3 as ULONG,
        b"\n BusLogic - Inside Start IO routine\n\0" as *const u8 as *const ::core::ffi::c_char as PCCHAR,
    );

    if BusLogic_CDB_MaxLength < (*Srb).CdbLength as ::core::ffi::c_int {
        ScsiDebugPrint(
            2 as ULONG,
            b"\n BusLogic - Srb->CdbLength [%d] > MaxCdbLength [%d]. Invalid request\n\0"
                as *const u8 as *const ::core::ffi::c_char as PCCHAR,
            (*Srb).CdbLength as ::core::ffi::c_int,
            12 as ::core::ffi::c_int,
        );
        (*Srb).SrbStatus = SRB_STATUS_INVALID_REQUEST as UCHAR;
        ScsiPortNotification(RequestComplete, deviceExtension as PVOID, Srb);
        ScsiPortNotification(NextRequest, deviceExtension as PVOID, NULL);
        return TRUE as BOOLEAN;
    }

    match (*Srb).Function as u32 {
        SRB_FUNCTION_EXECUTE_SCSI => {
            ccb = (*Srb).SrbExtension as PBuslogic_CCB_T;
            BusLogic_InitializeCCB(ccb);
            (*ccb).SrbAddress = Srb as PVOID;
            BusLogic_QueueCommand(HwDeviceExtension, Srb, ccb);
            if BusLogic_WriteOutgoingMailbox(
                deviceExtension,
                BusLogic_MailboxStartCommand,
                ccb as *mut BusLogic_CCB_T,
            ) == 0
            {
                ScsiDebugPrint(
                    1 as ULONG,
                    b"\n BusLogic - Unable to write Outgoing Mailbox - Pausing for 1 second\n\0"
                        as *const u8 as *const ::core::ffi::c_char as PCCHAR,
                );
                ScsiPortStallExecution(1000 as ULONG);
                if BusLogic_WriteOutgoingMailbox(
                    deviceExtension,
                    BusLogic_MailboxStartCommand,
                    ccb as *mut BusLogic_CCB_T,
                ) == 0
                {
                    ScsiDebugPrint(
                        1 as ULONG,
                        b"\n BusLogic - Still unable to write OutgoingMailbox - Host Adapter Dead?\n\0"
                            as *const u8 as *const ::core::ffi::c_char as PCCHAR,
                    );
                    (*Srb).SrbStatus = SRB_STATUS_ERROR as UCHAR;
                    ScsiPortNotification(RequestComplete, deviceExtension as PVOID, Srb);
                    ScsiPortNotification(NextRequest, deviceExtension as PVOID, NULL);
                }
            } else if (*TargetFlags).TaggedQueuingActive() as ::core::ffi::c_int != 0
                && ((*HostAdapter).ActiveCommandsPerLun[(*Srb).TargetId as usize]
                    [(*Srb).Lun as usize] as ::core::ffi::c_int)
                    < BUSLOGIC_MAXIMUM_TAGS as ::core::ffi::c_int
            {
                ScsiPortNotification(
                    NextLuRequest,
                    HwDeviceExtension,
                    (*Srb).PathId as ::core::ffi::c_int,
                    (*Srb).TargetId as ::core::ffi::c_int,
                    (*Srb).Lun as ::core::ffi::c_int,
                );
            } else {
                ScsiPortNotification(NextRequest, deviceExtension as PVOID, NULL);
            }
            return TRUE as BOOLEAN;
        }
        SRB_FUNCTION_ABORT_COMMAND => {
            ccb = (*(*Srb).NextSrb).SrbExtension as PBuslogic_CCB_T;
            (*ccb).AbortSrb = Srb as PVOID;
            AbortSRB = ScsiPortGetSrb(
                HwDeviceExtension,
                (*Srb).PathId,
                (*Srb).TargetId,
                (*Srb).Lun,
                (*Srb).QueueTag as LONG,
            );
            if AbortSRB != (*Srb).NextSrb
                || (*AbortSRB).SrbStatus as u32 != SRB_STATUS_PENDING
            {
                (*Srb).SrbStatus = SRB_STATUS_ABORT_FAILED as UCHAR;
                ScsiPortNotification(RequestComplete, HwDeviceExtension, Srb);
                ScsiPortNotification(NextRequest, HwDeviceExtension, NULL);
                return TRUE as BOOLEAN;
            }
            if BusLogic_WriteOutgoingMailbox(
                deviceExtension,
                BusLogic_MailboxAbortCommand,
                ccb as *mut BusLogic_CCB_T,
            ) != 0
            {
                ScsiDebugPrint(
                    2 as ULONG,
                    b"\n BusLogic - Aborting CCB #%ld to Target %d\n\0" as *const u8
                        as *const ::core::ffi::c_char as PCCHAR,
                    (*ccb).SerialNumber,
                    (*Srb).TargetId as ::core::ffi::c_int,
                );
                if (*TargetFlags).TaggedQueuingActive() as ::core::ffi::c_int != 0
                    && ((*HostAdapter).ActiveCommandsPerLun[(*Srb).TargetId as usize]
                        [(*Srb).Lun as usize] as ::core::ffi::c_int)
                        < BUSLOGIC_MAXIMUM_TAGS as ::core::ffi::c_int
                {
                    ScsiPortNotification(
                        NextLuRequest,
                        HwDeviceExtension,
                        (*Srb).PathId as ::core::ffi::c_int,
                        (*Srb).TargetId as ::core::ffi::c_int,
                        (*Srb).Lun as ::core::ffi::c_int,
                    );
                } else {
                    ScsiPortNotification(NextRequest, deviceExtension as PVOID, NULL);
                }
            } else {
                ScsiDebugPrint(
                    2 as ULONG,
                    b"\n BusLogic - Unable to Abort CCB #%ld to Target %d - No Outgoing Mailboxes\n\0"
                        as *const u8 as *const ::core::ffi::c_char as PCCHAR,
                    (*ccb).SerialNumber,
                    (*Srb).TargetId as ::core::ffi::c_int,
                );
                (*Srb).SrbStatus = SRB_STATUS_ERROR as UCHAR;
                ScsiPortNotification(RequestComplete, HwDeviceExtension, Srb);
                ScsiPortNotification(NextRequest, HwDeviceExtension, NULL);
            }
            return TRUE as BOOLEAN;
        }
        SRB_FUNCTION_RESET_BUS => {
            ScsiDebugPrint(
                4 as ULONG,
                b"\n BusLogic - SRB_FUNCTION_RESET_BUS, srb=%x \n\0" as *const u8
                    as *const ::core::ffi::c_char as PCCHAR,
                Srb,
            );
            BT958HwResetBus(HwDeviceExtension, (*Srb).PathId as ULONG);
            (*Srb).SrbStatus = SRB_STATUS_SUCCESS as UCHAR;
            ScsiPortNotification(RequestComplete, HwDeviceExtension, Srb);
            ScsiPortNotification(NextRequest, HwDeviceExtension, NULL);
            return TRUE as BOOLEAN;
        }
        SRB_FUNCTION_RESET_DEVICE => {
            ccb = (*Srb).SrbExtension as PBuslogic_CCB_T;
            BusLogic_InitializeCCB(ccb);
            (*ccb).SrbAddress = Srb as PVOID;
            if BusLogic_SendBusDeviceReset(HostAdapter as PVOID, Srb) == 0 {
                BT958HwResetBus(HwDeviceExtension, SP_UNTAGGED as ULONG);
                (*Srb).SrbStatus = SRB_STATUS_SUCCESS as UCHAR;
                ScsiPortNotification(RequestComplete, HwDeviceExtension, Srb);
                ScsiPortNotification(NextRequest, HwDeviceExtension, NULL);
                return TRUE as BOOLEAN;
            }
            if (*TargetFlags).TaggedQueuingActive() as ::core::ffi::c_int != 0
                && ((*HostAdapter).ActiveCommandsPerLun[(*Srb).TargetId as usize]
                    [(*Srb).Lun as usize] as ::core::ffi::c_int)
                    < BUSLOGIC_MAXIMUM_TAGS as ::core::ffi::c_int
            {
                ScsiPortNotification(
                    NextLuRequest,
                    HwDeviceExtension,
                    (*Srb).PathId as ::core::ffi::c_int,
                    (*Srb).TargetId as ::core::ffi::c_int,
                    (*Srb).Lun as ::core::ffi::c_int,
                );
            } else {
                ScsiPortNotification(NextRequest, deviceExtension as PVOID, NULL);
            }
            return TRUE as BOOLEAN;
        }
        _ => {
            (*Srb).SrbStatus = SRB_STATUS_INVALID_REQUEST as UCHAR;
            ScsiPortNotification(RequestComplete, deviceExtension as PVOID, Srb);
            ScsiPortNotification(NextRequest, deviceExtension as PVOID, NULL);
            return TRUE as BOOLEAN;
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn BusLogic_WriteOutgoingMailbox(
    deviceExtension: PHW_DEVICE_EXTENSION,
    ActionCode: BusLogic_ActionCode_T,
    CCB: *mut BusLogic_CCB_T,
) -> BOOLEAN {

    let HostAdapter: *mut BusLogic_HostAdapter_T = core::ptr::addr_of_mut!((*deviceExtension).hcs);
    let mut NextOutgoingMailbox: *mut BusLogic_OutgoingMailbox_T;
    let mut length: ULONG = 0;

    NextOutgoingMailbox = (*HostAdapter).NextOutgoingMailbox;

    if (*NextOutgoingMailbox).ActionCode as i32 == BusLogic_OutgoingMailboxFree as i32 {
        (*CCB).Status = BusLogic_CCB_Active;

        (*NextOutgoingMailbox).CCB = ScsiPortConvertPhysicalAddressToUlong(
            ScsiPortGetPhysicalAddress(
                deviceExtension as PVOID,
                core::ptr::null_mut::<_SCSI_REQUEST_BLOCK>(),
                CCB as PVOID,
                core::ptr::addr_of_mut!(length),
            ),
        ) as BusLogic_BusAddress_T;

        (*NextOutgoingMailbox).ActionCode = ActionCode as UCHAR;

        BusLogic_StartMailboxCommand(HostAdapter);

        NextOutgoingMailbox = NextOutgoingMailbox.offset(1);
        if NextOutgoingMailbox > (*HostAdapter).LastOutgoingMailbox {
            NextOutgoingMailbox = (*HostAdapter).FirstOutgoingMailbox;
        }

        (*HostAdapter).NextOutgoingMailbox = NextOutgoingMailbox;
        if ActionCode as u32 == BusLogic_MailboxStartCommand as i32 as u32 {
            (*HostAdapter).ActiveCommandsPerTarget[(*CCB).TargetID as usize] =
                (*HostAdapter).ActiveCommandsPerTarget[(*CCB).TargetID as usize].wrapping_add(1);
            (*HostAdapter).ActiveCommandsPerLun[(*CCB).TargetID as usize]
                [(*CCB).LogicalUnit() as usize] = (*HostAdapter).ActiveCommandsPerLun
                [(*CCB).TargetID as usize][(*CCB).LogicalUnit() as usize]
                .wrapping_add(1);
            if (*CCB).Opcode as i32 != BusLogic_BusDeviceReset as i32 {
                (*HostAdapter).TargetStatistics[(*CCB).TargetID as usize].CommandsAttempted =
                    (*HostAdapter).TargetStatistics[(*CCB).TargetID as usize]
                        .CommandsAttempted
                        .wrapping_add(1);
            }
        }
        return 1 as BOOLEAN;
    }
    return 0 as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn BusLogic_QueueCommand(
    mut HwDeviceExtension: PVOID,
    mut Srb: PSCSI_REQUEST_BLOCK,
    mut CCB: PBuslogic_CCB_T,
) -> ::core::ffi::c_int {
    let mut deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let mut HostAdapter: *mut BusLogic_HostAdapter_T = &raw mut (*deviceExtension).hcs;
    let mut TargetFlags: *mut BusLogic_TargetFlags_T =
        (&raw mut (*HostAdapter).TargetFlags as *mut BusLogic_TargetFlags_T)
            .offset((*Srb).TargetId as isize) as *mut BusLogic_TargetFlags_T;
    let mut TargetStatistics: *mut BusLogic_TargetStatistics_T =
        &raw mut (*HostAdapter).TargetStatistics as *mut BusLogic_TargetStatistics_T;
    let mut length: ULONG = 0;
    let mut CDB_Length: UCHAR = (*Srb).CdbLength;
    let mut TargetID: UCHAR = (*Srb).TargetId;
    let mut LogicalUnit: UCHAR = (*Srb).Lun;
    let mut BufferLength: ::core::ffi::c_int = (*Srb).DataTransferLength as ::core::ffi::c_int;
    if (*Srb).DataTransferLength > 0 as ::core::ffi::c_ulong {
        (*CCB).DataLength = (*Srb).DataTransferLength as BusLogic_ByteCount_T;
        let mut xferLength: ULONG = 0;
        let mut remainLength: ULONG = 0;
        let mut virtualAddress: PVOID = ::core::ptr::null_mut::<::core::ffi::c_void>();
        let mut i: UCHAR = 0 as UCHAR;
        virtualAddress = (*Srb).DataBuffer;
        xferLength = (*Srb).DataTransferLength;
        remainLength = xferLength;
        loop {
            (*CCB).ScatterGatherList[i as usize].SegmentDataPointer =
                ScsiPortConvertPhysicalAddressToUlong(ScsiPortGetPhysicalAddress(
                    HwDeviceExtension,
                    Srb,
                    virtualAddress,
                    &raw mut length,
                )) as BusLogic_BusAddress_T;
            if length > remainLength {
                length = remainLength;
            }
            (*CCB).ScatterGatherList[i as usize].SegmentByteCount =
                length as BusLogic_ByteCount_T;
            virtualAddress = (virtualAddress as PUCHAR).offset(length as isize) as PVOID;
            if length >= remainLength {
                remainLength = 0 as ULONG;
            } else {
                remainLength = (remainLength as ::core::ffi::c_ulong)
                    .wrapping_sub(length as ::core::ffi::c_ulong)
                    as ULONG;
            }
            i = i.wrapping_add(1);
            if !(remainLength > 0 as ::core::ffi::c_ulong) {
                break;
            }
        }
        if i as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
            (*CCB).Opcode = BusLogic_InitiatorCCB_ScatterGather as ::core::ffi::c_int as UCHAR;
            (*CCB).DataLength = (i as usize)
                .wrapping_mul(::core::mem::size_of::<BusLogic_ScatterGatherSegment_T>())
                as BusLogic_ByteCount_T;
            virtualAddress = &raw mut (*CCB).ScatterGatherList as PVOID;
            (*CCB).DataPointer = ScsiPortConvertPhysicalAddressToUlong(
                ScsiPortGetPhysicalAddress(
                    HwDeviceExtension,
                    ::core::ptr::null_mut::<_SCSI_REQUEST_BLOCK>(),
                    virtualAddress,
                    &raw mut length,
                ),
            ) as BusLogic_BusAddress_T;
        } else {
            (*CCB).Opcode = BusLogic_InitiatorCCB as ::core::ffi::c_int as UCHAR;
            (*CCB).DataLength =
                (*CCB).ScatterGatherList[0 as ::core::ffi::c_int as usize].SegmentByteCount;
            (*CCB).DataPointer =
                (*CCB).ScatterGatherList[0 as ::core::ffi::c_int as usize].SegmentDataPointer;
        }
    }
    match (*Srb).SrbFlags as ::core::ffi::c_ulong
        & SRB_FLAGS_UNSPECIFIED_DIRECTION as ::core::ffi::c_ulong
    {
        0 => {
            (*CCB).set_DataDirection(BusLogic_NoDataTransfer as ::core::ffi::c_int as UCHAR);
        }
        64 => {
            (*CCB).set_DataDirection(
                BusLogic_DataInLengthChecked as ::core::ffi::c_int as UCHAR,
            );
            let ref mut fresh12 = (*TargetStatistics.offset(TargetID as isize)).ReadCommands;
            *fresh12 = (*fresh12).wrapping_add(1);
            BusLogic_IncrementByteCounter(
                &raw mut (*TargetStatistics.offset(TargetID as isize)).TotalBytesRead,
                BufferLength as ::core::ffi::c_uint,
            );
            BusLogic_IncrementSizeBucket(
                &raw mut (*TargetStatistics.offset(TargetID as isize)).ReadCommandSizeBuckets
                    as *mut ::core::ffi::c_uint,
                BufferLength as ::core::ffi::c_uint,
            );
        }
        128 => {
            (*CCB).set_DataDirection(
                BusLogic_DataOutLengthChecked as ::core::ffi::c_int as UCHAR,
            );
            let ref mut fresh13 = (*TargetStatistics.offset(TargetID as isize)).WriteCommands;
            *fresh13 = (*fresh13).wrapping_add(1);
            BusLogic_IncrementByteCounter(
                &raw mut (*TargetStatistics.offset(TargetID as isize)).TotalBytesWritten,
                BufferLength as ::core::ffi::c_uint,
            );
            BusLogic_IncrementSizeBucket(
                &raw mut (*TargetStatistics.offset(TargetID as isize)).WriteCommandSizeBuckets
                    as *mut ::core::ffi::c_uint,
                BufferLength as ::core::ffi::c_uint,
            );
        }
        192 | _ => {
            (*CCB).set_DataDirection(
                BusLogic_UncheckedDataTransfer as ::core::ffi::c_int as UCHAR,
            );
        }
    }
    (*CCB).CDB_Length = CDB_Length;
    (*CCB).SenseDataLength = (*Srb).SenseInfoBufferLength;
    (*CCB).HostAdapterStatus = 0 as UCHAR;
    (*CCB).TargetDeviceStatus = 0 as UCHAR;
    (*CCB).TargetID = TargetID;
    (*CCB).set_LogicalUnit(LogicalUnit as UCHAR);
    (*CCB).set_TagEnable(FALSE as UCHAR);
    (*CCB).set_LegacyTagEnable(FALSE as UCHAR);
    let fresh14 = (*HostAdapter).CommandsSinceReset[TargetID as usize];
    (*HostAdapter).CommandsSinceReset[TargetID as usize] =
        (*HostAdapter).CommandsSinceReset[TargetID as usize].wrapping_add(1);
    if fresh14 >= BusLogic_MaxTaggedQueueDepth as ::core::ffi::c_uint
        && (*HostAdapter).ActiveCommandsPerTarget[TargetID as usize] as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int
        && (*TargetFlags).TaggedQueuingActive() == 0
        && (*TargetFlags).TaggedQueuingSupported() as ::core::ffi::c_int != 0
        && (*HostAdapter).TaggedQueuingPermitted as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << TargetID as ::core::ffi::c_int
            != 0
    {
        (*TargetFlags).set_TaggedQueuingActive(TRUE as BOOLEAN);
        ScsiDebugPrint(
            4 as ULONG,
            b"\n BusLogic - Tagged Queuing now active for Target %d\n\0" as *const u8
                as *const ::core::ffi::c_char as PCCHAR,
            TargetID as ::core::ffi::c_int,
        );
    }
    if (*TargetFlags).TaggedQueuingActive() != 0 {
        let mut QueueTag: BusLogic_QueueTag_T = BusLogic_SimpleQueueTag;
        if (*HostAdapter).ExtendedLUNSupport() != 0 {
            (*CCB).set_TagEnable(TRUE as UCHAR);
            (*CCB).set_QueueTag(QueueTag as UCHAR);
        } else {
            (*CCB).set_LegacyTagEnable(TRUE as UCHAR);
            (*CCB).set_LegacyQueueTag(QueueTag as UCHAR);
        }
    }
    memmove(
        &raw mut (*CCB).CDB as *mut UCHAR as *mut ::core::ffi::c_void,
        &raw mut (*Srb).Cdb as *mut UCHAR as *const ::core::ffi::c_void,
        CDB_Length as usize,
    );
    if (*Srb).SrbFlags as ::core::ffi::c_ulong
        & SRB_FLAGS_DISABLE_AUTOSENSE as ::core::ffi::c_ulong
        != 0
        || (*Srb).SenseInfoBufferLength as ::core::ffi::c_int <= 0 as ::core::ffi::c_int
    {
        (*CCB).SenseDataLength = BusLogic_DisableAutoReqSense as UCHAR;
    } else {
        (*CCB).SenseDataLength = (*Srb).SenseInfoBufferLength as UCHAR;
        (*CCB).SenseDataPointer = ScsiPortConvertPhysicalAddressToUlong(
            ScsiPortGetPhysicalAddress(
                HwDeviceExtension,
                Srb,
                (*Srb).SenseInfoBuffer,
                &raw mut length,
            ),
        ) as BusLogic_BusAddress_T;
        if (*Srb).SenseInfoBufferLength as ::core::ffi::c_ulong > length {
            (*CCB).SenseDataLength = length as ::core::ffi::c_uchar as UCHAR;
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "system" fn BT958HwInterrupt(HwDeviceExtension: PVOID) -> BOOLEAN {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let HostAdapter: *mut BusLogic_HostAdapter_T = core::ptr::addr_of_mut!((*deviceExtension).hcs);
    let mut InterruptRegister: BusLogic_InterruptRegister_T = core::mem::zeroed();

    // Read the Host Adapter Interrupt Register.
    *InterruptRegister.All.as_mut() = BusLogic_ReadInterruptRegister(HostAdapter);
    if InterruptRegister.Bits.as_ref().InterruptValid() != 0 {
        // Acknowledge the interrupt and reset the Host Adapter
        // Interrupt Register.
        BusLogic_InterruptReset(HostAdapter);
        // Process valid External SCSI Bus Reset and Incoming Mailbox
        // Loaded Interrupts.
        if InterruptRegister.Bits.as_ref().ExternalBusReset() != 0 {
            (*HostAdapter).set_HostAdapterExternalReset(TRUE as BOOLEAN);
        } else if InterruptRegister.Bits.as_ref().IncomingMailboxLoaded() != 0 {
            BusLogic_ScanIncomingMailboxes(deviceExtension);
        } else if InterruptRegister.Bits.as_ref().CommandComplete() != 0 {
            (*HostAdapter).HostAdapterCommandCompleted = TRUE as BOOLEAN;
        }
    } else {
        return FALSE as BOOLEAN;
    }

    // Process any completed CCBs.
    if !(*HostAdapter).FirstCompletedCCB.is_null() {
        BusLogic_ProcessCompletedCCBs(deviceExtension);
    }

    // Reset the Host Adapter if requested.
    if (*HostAdapter).HostAdapterExternalReset() as core::ffi::c_int != 0
        || (*HostAdapter).HostAdapterInternalError() as core::ffi::c_int != 0
    {
        BT958HwResetBus(HwDeviceExtension, SP_UNTAGGED as ULONG);
        (*HostAdapter).set_HostAdapterExternalReset(FALSE as BOOLEAN);
        (*HostAdapter).set_HostAdapterInternalError(FALSE as BOOLEAN);
    }
    return TRUE as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn BusLogic_ScanIncomingMailboxes(deviceExtension: PHW_DEVICE_EXTENSION) {
    let HostAdapter: *mut BusLogic_HostAdapter_T = core::ptr::addr_of_mut!((*deviceExtension).hcs);

    let mut NextIncomingMailbox: *mut BusLogic_IncomingMailbox_T =
        (*HostAdapter).NextIncomingMailbox;
    let mut CompletionCode: BusLogic_CompletionCode_T;

    loop {
        CompletionCode = (*NextIncomingMailbox).CompletionCode as BusLogic_CompletionCode_T;
        if !((CompletionCode as core::ffi::c_uint)
            != BusLogic_IncomingMailboxFree as core::ffi::c_uint)
        {
            break;
        }

        // Convert Physical CCB to Virtual.
        let CCB: *mut BusLogic_CCB_T = ScsiPortGetVirtualAddress(
            deviceExtension as PVOID,
            ScsiPortConvertUlongToPhysicalAddress((*NextIncomingMailbox).CCB as _),
        ) as *mut BusLogic_CCB_T;

        ScsiDebugPrint(
            4 as ULONG,
            b"\n Buslogic - Virtual CCB %lx\n\0" as *const u8 as *const core::ffi::c_char
                as PCCHAR,
            CCB,
        );

        if (CompletionCode as core::ffi::c_uint)
            != BusLogic_AbortedCommandNotFound as core::ffi::c_uint
        {
            if (*CCB).Status as core::ffi::c_uint
                == BusLogic_CCB_Active as core::ffi::c_uint
                || (*CCB).Status as core::ffi::c_uint
                    == BusLogic_CCB_Reset as core::ffi::c_uint
            {
                // Save the Completion Code for this CCB and queue the CCB
                // for completion processing.
                (*CCB).CompletionCode = CompletionCode;
                BusLogic_QueueCompletedCCB(deviceExtension, CCB);
            } else {
                // If a CCB ever appears in an Incoming Mailbox and is not marked
                // as status Active or Reset, then there is most likely a bug in
                // the Host Adapter firmware.
                ScsiDebugPrint(
                    1 as ULONG,
                    b"\n BusLogic - Illegal CCB #%ld status %d in Incoming Mailbox\n\0" as *const u8
                        as *const core::ffi::c_char as PCCHAR,
                    (*CCB).SerialNumber,
                    (*CCB).Status as core::ffi::c_uint,
                );
            }
        }

        (*NextIncomingMailbox).CompletionCode =
            BusLogic_IncomingMailboxFree as core::ffi::c_int as UCHAR;
        NextIncomingMailbox = NextIncomingMailbox.offset(1);
        if NextIncomingMailbox > (*HostAdapter).LastIncomingMailbox {
            NextIncomingMailbox = (*HostAdapter).FirstIncomingMailbox;
        }
    }

    (*HostAdapter).NextIncomingMailbox = NextIncomingMailbox;
}
#[no_mangle]
pub unsafe extern "C" fn BusLogic_QueueCompletedCCB(
    deviceExtension: PHW_DEVICE_EXTENSION,
    CCB: *mut BusLogic_CCB_T,
) {
    let HostAdapter: *mut BusLogic_HostAdapter_T = core::ptr::addr_of_mut!((*deviceExtension).hcs);
    (*CCB).Status = BusLogic_CCB_Completed;
    (*CCB).Next = core::ptr::null_mut();
    if (*HostAdapter).FirstCompletedCCB.is_null() {
        (*HostAdapter).FirstCompletedCCB = CCB;
        (*HostAdapter).LastCompletedCCB = CCB;
    } else {
        (*(*HostAdapter).LastCompletedCCB).Next = CCB;
        (*HostAdapter).LastCompletedCCB = CCB;
    }
    (*HostAdapter).ActiveCommandsPerTarget[(*CCB).TargetID as usize] =
        (*HostAdapter).ActiveCommandsPerTarget[(*CCB).TargetID as usize].wrapping_sub(1);
    (*HostAdapter).ActiveCommandsPerLun[(*CCB).TargetID as usize][(*CCB).LogicalUnit() as usize] =
        (*HostAdapter).ActiveCommandsPerLun[(*CCB).TargetID as usize]
            [(*CCB).LogicalUnit() as usize]
            .wrapping_sub(1);
}

#[no_mangle]
pub unsafe extern "C" fn BusLogic_ProcessCompletedCCBs(mut deviceExtension: PHW_DEVICE_EXTENSION) {

    let mut HostAdapter: *mut BusLogic_HostAdapter_T = &raw mut (*deviceExtension).hcs;
    let mut srb: PSCSI_REQUEST_BLOCK = core::ptr::null_mut();
    let mut RealCdb: PCDB = core::ptr::null_mut();

    if (*HostAdapter).ProcessCompletedCCBsActive != 0 {
        return;
    }
    (*HostAdapter).ProcessCompletedCCBsActive = TRUE as BOOLEAN;
    while !(*HostAdapter).FirstCompletedCCB.is_null() {
        let mut CCB: *mut BusLogic_CCB_T = (*HostAdapter).FirstCompletedCCB;

        srb = (*CCB).SrbAddress as PSCSI_REQUEST_BLOCK;

        (*HostAdapter).FirstCompletedCCB = (*CCB).Next as *mut BusLogic_CCB_T;
        if (*HostAdapter).FirstCompletedCCB.is_null() {
            (*HostAdapter).LastCompletedCCB = core::ptr::null_mut();
        }

        if (*CCB).Opcode as i32 == BusLogic_BusDeviceReset as i32 {
            let mut TargetID: i32 = (*CCB).TargetID as i32;
            let mut LunID: i32 = 0;
            ScsiDebugPrint(
                3 as ULONG,
                b"\n BusLogic - Bus Device Reset CCB #%ld to Target %d Completed\n\0".as_ptr()
                    as PCCHAR,
                (*CCB).SerialNumber,
                TargetID,
            );
            BusLogic_IncrementErrorCounter(
                &raw mut (*(&raw mut (*HostAdapter).TargetStatistics
                    as *mut BusLogic_TargetStatistics_T)
                    .offset(TargetID as isize))
                .BusDeviceResetsCompleted,
            );

            (*HostAdapter).TargetFlags[TargetID as usize]
                .set_TaggedQueuingActive(FALSE as BOOLEAN);
            (*HostAdapter).CommandsSinceReset[TargetID as usize] = 0;
            (*HostAdapter).ActiveCommandsPerTarget[TargetID as usize] = 0 as UCHAR;
            LunID = 0;
            while LunID < (*HostAdapter).MaxLogicalUnits as i32 {
                (*HostAdapter).ActiveCommandsPerLun[TargetID as usize][LunID as usize] = 0 as UCHAR;
                LunID += 1;
            }

            ScsiPortCompleteRequest(
                deviceExtension as PVOID,
                (*srb).PathId,
                (*srb).TargetId,
                0xff as UCHAR,
                SRB_STATUS_BUS_RESET as ULONG as UCHAR,
            );

            (*HostAdapter).BusDeviceResetPendingCCB[TargetID as usize] = core::ptr::null_mut();
        } else {
            match (*CCB).CompletionCode as u32 {
                0 | 5 => {
                    ScsiDebugPrint(
                        1 as ULONG,
                        b"\n BusLogic - CCB #%ld to Target %d Impossible State\n\0".as_ptr()
                            as PCCHAR,
                        (*CCB).SerialNumber,
                        (*CCB).TargetID as i32,
                    );
                }
                3 => {
                    srb = (*CCB).AbortSrb as PSCSI_REQUEST_BLOCK;
                    (*srb).SrbStatus = SRB_STATUS_ABORT_FAILED as UCHAR;
                }
                1 => {
                    (*HostAdapter).TargetStatistics[(*CCB).TargetID as usize].CommandsCompleted =
                        (*HostAdapter).TargetStatistics[(*CCB).TargetID as usize]
                            .CommandsCompleted
                            .wrapping_add(1);
                    (*HostAdapter).TargetFlags[(*CCB).TargetID as usize]
                        .set_CommandSuccessfulFlag(TRUE as BOOLEAN);
                    (*srb).SrbStatus = SRB_STATUS_SUCCESS as UCHAR;
                }
                2 => {
                    ScsiDebugPrint(
                        3 as ULONG,
                        b"\n BusLogic - CCB #%ld to Target %d Aborted\n\0".as_ptr() as PCCHAR,
                        (*CCB).SerialNumber,
                        (*CCB).TargetID as i32,
                    );
                    (*srb).SrbStatus = SRB_STATUS_ABORTED as UCHAR;
                    ScsiPortNotification(RequestComplete, deviceExtension as PVOID, srb);
                    srb = (*CCB).AbortSrb as PSCSI_REQUEST_BLOCK;
                    (*srb).SrbStatus = SRB_STATUS_SUCCESS as UCHAR;
                }
                4 => {
                    RealCdb = &raw mut (*CCB).CDB as *mut UCHAR as PCDB;
                    ScsiDebugPrint(
                        1 as ULONG,
                        b"\n BusLogic - %x Command completed with error Host - %x Target %x \n\0"
                            .as_ptr() as PCCHAR,
                        (*RealCdb).CDB6GENERIC.as_ref().OperationCode as i32,
                        (*CCB).HostAdapterStatus as i32,
                        (*CCB).TargetDeviceStatus as i32,
                    );
                    (*srb).SrbStatus = BusLogic_ComputeResultCode(
                        HostAdapter,
                        (*CCB).HostAdapterStatus as BusLogic_HostAdapterStatus_T,
                        (*CCB).TargetDeviceStatus as BusLogic_TargetDeviceStatus_T,
                        (*CCB).SenseDataLength,
                    );
                    if (*CCB).HostAdapterStatus as i32
                        != BusLogic_SCSISelectionTimeout as i32
                    {
                        (*HostAdapter).TargetStatistics[(*CCB).TargetID as usize]
                            .CommandsCompleted = (*HostAdapter).TargetStatistics
                            [(*CCB).TargetID as usize]
                            .CommandsCompleted
                            .wrapping_add(1);
                    }
                }
                _ => {
                    ScsiPortLogError(
                        deviceExtension as PVOID,
                        core::ptr::null_mut(),
                        0 as UCHAR,
                        (*srb).TargetId,
                        0 as UCHAR,
                        SP_INTERNAL_ADAPTER_ERROR as ULONG,
                        143 as ULONG,
                    );
                    ScsiDebugPrint(
                        1 as ULONG,
                        b"\n BusLogic - Unrecognized mailbox status\n\0".as_ptr() as PCCHAR,
                    );
                }
            }

            RealCdb = &raw mut (*CCB).CDB as *mut UCHAR as PCDB;
            if (*RealCdb).CDB6INQUIRY.as_ref().OperationCode as u32 == SCSIOP_INQUIRY
                && (*CCB).HostAdapterStatus as i32
                    == BusLogic_CommandCompletedNormally as i32
            {
                let mut TargetFlags: *mut BusLogic_TargetFlags_T =
                    (&raw mut (*HostAdapter).TargetFlags as *mut BusLogic_TargetFlags_T)
                        .offset((*CCB).TargetID as isize)
                        as *mut BusLogic_TargetFlags_T;
                let mut InquiryResult: *mut SCSI_Inquiry_T =
                    (*srb).DataBuffer as *mut SCSI_Inquiry_T;
                (*TargetFlags).set_TargetExists(TRUE as BOOLEAN);
                (*TargetFlags).set_TaggedQueuingSupported((*InquiryResult).CmdQue() as BOOLEAN);
                (*TargetFlags).set_WideTransfersSupported((*InquiryResult).WBus16() as BOOLEAN);
            }

            ScsiDebugPrint(
                4 as ULONG,
                b"\n BusLogic - SCSI Status %x\n\0".as_ptr() as PCCHAR,
                (*srb).ScsiStatus as i32,
            );
            ScsiDebugPrint(
                4 as ULONG,
                b"\n BusLogic - HBA Status %x\n\0".as_ptr() as PCCHAR,
                (*CCB).HostAdapterStatus as i32,
            );

            (*srb).ScsiStatus = (*CCB).TargetDeviceStatus;

            ScsiPortNotification(RequestComplete, deviceExtension as PVOID, srb);
        }
    }
    (*HostAdapter).ProcessCompletedCCBsActive = FALSE as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn BusLogic_ComputeResultCode(
    mut HostAdapter: *mut BusLogic_HostAdapter_T,
    mut HostAdapterStatus: BusLogic_HostAdapterStatus_T,
    mut TargetDeviceStatus: BusLogic_TargetDeviceStatus_T,
    mut SenseDataLength: UCHAR,
) -> UCHAR {
    let mut HostStatus: UCHAR = 0 as UCHAR;
    if TargetDeviceStatus as u32 != BusLogic_OperationGood as i32 as u32
        && (HostAdapterStatus as u32
            == BusLogic_CommandCompletedNormally as i32 as u32
            || HostAdapterStatus as u32
                == BusLogic_LinkedCommandCompleted as i32 as u32
            || HostAdapterStatus as u32
                == BusLogic_LinkedCommandCompletedWithFlag as i32 as u32)
    {
        match TargetDeviceStatus as u32 {
            2 => {
                HostStatus = SRB_STATUS_ERROR as UCHAR;
                if SenseDataLength as u32 != BusLogic_DisableAutoReqSense {
                    HostStatus =
                        (HostStatus as u32 | SRB_STATUS_AUTOSENSE_VALID) as UCHAR;
                }
            }
            8 => {
                HostStatus = SRB_STATUS_BUSY as UCHAR;
            }
            0 => {
                HostStatus = SRB_STATUS_SUCCESS as UCHAR;
            }
            _ => {}
        }
    } else {
        let mut current_block_19: u64;
        match HostAdapterStatus as u32 {
            0 | 10 | 11 => {
                HostStatus = SRB_STATUS_SUCCESS as UCHAR;
                current_block_19 = 18377268871191777778;
            }
            17 => {
                HostStatus = SRB_STATUS_SELECTION_TIMEOUT as UCHAR;
                current_block_19 = 18377268871191777778;
            }
            21 | 22 | 26 => {
                ScsiDebugPrint(
                    2 as ULONG,
                    b"\n BusLogic - Driver Protocol Error 0x%02X\n\0" as *const u8
                        as *const i8 as PCCHAR,
                    HostAdapterStatus as u32,
                );
                current_block_19 = 17407779659766490442;
            }
            12 | 18 | 23 | 28 | 36 | 38 | 39 | 48 => {
                current_block_19 = 17407779659766490442;
            }
            33 | 34 | 35 | 37 => {
                HostStatus = SRB_STATUS_BUS_RESET as UCHAR;
                current_block_19 = 18377268871191777778;
            }
            52 => {
                HostStatus = SRB_STATUS_PARITY_ERROR as UCHAR;
                current_block_19 = 18377268871191777778;
            }
            19 => {
                HostStatus = SRB_STATUS_UNEXPECTED_BUS_FREE as UCHAR;
                current_block_19 = 18377268871191777778;
            }
            20 => {
                HostStatus = SRB_STATUS_PHASE_SEQUENCE_FAILURE as UCHAR;
                current_block_19 = 18377268871191777778;
            }
            27 => {
                HostStatus = SRB_STATUS_REQUEST_SENSE_FAILED as UCHAR;
                current_block_19 = 18377268871191777778;
            }
            29 => {
                HostStatus = SRB_STATUS_INVALID_REQUEST as UCHAR;
                current_block_19 = 18377268871191777778;
            }
            32 => {
                HostStatus = SRB_STATUS_NO_HBA as UCHAR;
                current_block_19 = 18377268871191777778;
            }
            _ => {
                ScsiDebugPrint(
                    2 as ULONG,
                    b"\n BusLogic - Unknown HBA Status 0x%02X\n\0" as *const u8
                        as *const i8 as PCCHAR,
                    HostAdapterStatus as u32,
                );
                HostStatus = SRB_STATUS_ERROR as UCHAR;
                current_block_19 = 18377268871191777778;
            }
        }
        match current_block_19 {
            17407779659766490442 => {
                HostStatus = SRB_STATUS_ERROR as UCHAR;
            }
            _ => {}
        }
    }
    return HostStatus;
}
#[no_mangle]
pub unsafe extern "system" fn BT958HwResetBus(
    HwDeviceExtension: PVOID,
    PathId: ULONG,
) -> BOOLEAN {

    const TRACE: u32 = 3;
    const ERROR: u32 = 1; // nolint: const-contradiction (BusLogic958.h:55 `#define ERROR 1` debug level, not wingdi ERROR=0)
    const SRB_STATUS_BUS_RESET: u32 = 0x0E;

    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let HostAdapter: *mut BusLogic_HostAdapter_T = core::ptr::addr_of_mut!((*deviceExtension).hcs);
    let Result: BOOLEAN;
    let HardReset: BOOLEAN;

    ScsiDebugPrint(TRACE, b"\n BusLogic - Reset SCSI bus\n\0".as_ptr() as PCCHAR);

    // Complete all outstanding requests with SRB_STATUS_BUS_RESET.
    ScsiPortCompleteRequest(
        deviceExtension as PVOID,
        PathId as UCHAR,
        0xFF as UCHAR,
        0xFF as UCHAR,
        SRB_STATUS_BUS_RESET as ULONG as UCHAR,
    );

    if (*HostAdapter).HostAdapterExternalReset() != 0 {
        HardReset = 0 as BOOLEAN;
    } else if (*HostAdapter).HostAdapterInternalError() != 0 {
        HardReset = 1 as BOOLEAN;
    } else {
        HardReset = 1 as BOOLEAN;
    }

    // Attempt to Reset and Reinitialize the Host Adapter.
    if !(BusLogic_HardwareResetHostAdapter(HostAdapter, HardReset) as i32 != 0
        && BusLogic_InitializeHostAdapter(deviceExtension, core::ptr::null_mut()) as i32 != 0)
    {
        ScsiDebugPrint(
            ERROR,
            b"\n Buslogic - Resetting %s Failed\n\0".as_ptr() as PCCHAR,
            core::ptr::addr_of_mut!((*HostAdapter).FullModelName) as *mut UCHAR,
        );
        Result = 0 as BOOLEAN;
        return Result;
    }

    if HardReset != 0 {
        ScsiPortStallExecution((*HostAdapter).BusSettleTime as ULONG);
    }
    Result = 1 as BOOLEAN;

    return Result;
}

#[no_mangle]
pub unsafe extern "C" fn BusLogic_SendBusDeviceReset(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
) -> BOOLEAN {

    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let HostAdapter: *mut BusLogic_HostAdapter_T = core::ptr::addr_of_mut!((*deviceExtension).hcs);
    let TargetID: UCHAR = (*Srb).TargetId;
    let CCB: PBuslogic_CCB_T = (*Srb).SrbExtension as PBuslogic_CCB_T;
    let mut Result: BOOLEAN = 0 as BOOLEAN;

    BusLogic_IncrementErrorCounter(
        core::ptr::addr_of_mut!(
            (*(core::ptr::addr_of_mut!((*HostAdapter).TargetStatistics)
                as *mut BusLogic_TargetStatistics_T)
                .offset(TargetID as isize))
            .BusDeviceResetsRequested
        ) as *mut USHORT,
    );

    if !(*HostAdapter).BusDeviceResetPendingCCB[TargetID as usize].is_null() {
        ScsiDebugPrint(
            2 as ULONG,
            b"\n BusLogic - Unable to Reset Command to Target %d - Reset Pending\n\0" as *const u8
                as *const core::ffi::c_char as PCCHAR,
            TargetID as core::ffi::c_int,
        );
        Result = 1 as BOOLEAN;
    } else {
        ScsiDebugPrint(
            2 as ULONG,
            b"\n BusLogic - Sending Bus Device Reset CCB #%ld to Target %d\n\0" as *const u8
                as *const core::ffi::c_char as PCCHAR,
            (*CCB).SerialNumber,
            TargetID as core::ffi::c_int,
        );
        (*CCB).Opcode = BusLogic_BusDeviceReset as core::ffi::c_int as UCHAR;
        (*CCB).TargetID = TargetID;

        if BusLogic_WriteOutgoingMailbox(
            deviceExtension,
            BusLogic_MailboxStartCommand,
            CCB as *mut BusLogic_CCB_T,
        ) == 0
        {
            ScsiDebugPrint(
                2 as ULONG,
                b"\n BusLogic - Unable to write Outgoing Mailbox for Bus Device Reset\n\0"
                    as *const u8 as *const core::ffi::c_char as PCCHAR,
            );
        } else {
            BusLogic_IncrementErrorCounter(
                core::ptr::addr_of_mut!(
                    (*(core::ptr::addr_of_mut!((*HostAdapter).TargetStatistics)
                        as *mut BusLogic_TargetStatistics_T)
                        .offset(TargetID as isize))
                    .BusDeviceResetsAttempted
                ) as *mut USHORT,
            );
            (*HostAdapter).BusDeviceResetPendingCCB[TargetID as usize] = CCB as *mut BusLogic_CCB_T;
            Result = 1 as BOOLEAN;
        }
    }
    return Result;
}
#[no_mangle]
pub unsafe extern "system" fn BT958HwInitialize(HwDeviceExtension: PVOID) -> BOOLEAN {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let HostAdapter: *mut BusLogic_HostAdapter_T = core::ptr::addr_of_mut!((*deviceExtension).hcs);
    let mut Parameter: CHAR = 1 as CHAR;
    if BusLogic_Command(
        HostAdapter,
        BusLogic_DisableHostAdapterInterrupt,
        core::ptr::addr_of_mut!(Parameter) as *mut core::ffi::c_void,
        core::mem::size_of::<CHAR>() as core::ffi::c_int,
        core::ptr::null_mut(),
        0 as core::ffi::c_int,
    ) < 0 as core::ffi::c_int
    {
        return 0 as BOOLEAN;
    }
    return 1 as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "system" fn BT958HwAdapterControl(
    HwDeviceExtension: PVOID,
    ControlType: SCSI_ADAPTER_CONTROL_TYPE,
    Parameters: PVOID,
) -> SCSI_ADAPTER_CONTROL_STATUS {

    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let HostAdapter: *mut BusLogic_HostAdapter_T = core::ptr::addr_of_mut!((*deviceExtension).hcs);

    let mut ControlTypeList: PSCSI_SUPPORTED_CONTROL_TYPE_LIST = core::ptr::null_mut();
    let mut AdjustedMaxControlType: ULONG;

    let mut Index: ULONG;

    let SupportedConrolTypes: [BOOLEAN; ScsiAdapterControlMax as usize] = [
        1 as BOOLEAN, // ScsiQuerySupportedControlTypes
        1 as BOOLEAN, // ScsiStopAdapter
        1 as BOOLEAN, // ScsiRestartAdapter
        0 as BOOLEAN, // ScsiSetBootConfig
        0 as BOOLEAN, // ScsiSetRunningConfig
    ];

    ScsiDebugPrint(
        3 as ULONG,
        b"\n BusLogic -  Inside HwAdapterControl function \n\0".as_ptr() as PCCHAR,
    );

    match ControlType as core::ffi::c_uint {
        0 => {
            ControlTypeList = Parameters as PSCSI_SUPPORTED_CONTROL_TYPE_LIST;
            AdjustedMaxControlType = (if (*ControlTypeList).MaxControlType
                < ScsiAdapterControlMax as core::ffi::c_int as core::ffi::c_ulong
            {
                (*ControlTypeList).MaxControlType as core::ffi::c_ulong
            } else {
                ScsiAdapterControlMax as core::ffi::c_int as core::ffi::c_ulong
            }) as ULONG;
            Index = 0 as ULONG;
            while Index < AdjustedMaxControlType {
                *(core::ptr::addr_of_mut!((*ControlTypeList).SupportedTypeList) as *mut BOOLEAN)
                    .offset(Index as isize) = SupportedConrolTypes[Index as usize];
                Index = Index.wrapping_add(1);
            }
        }
        1 => {
            let mut Parameter: CHAR = 0 as CHAR;
            ScsiDebugPrint(
                4 as ULONG,
                b"\n BusLogic -  stopping the device \n\0".as_ptr() as PCCHAR,
            );
            if BusLogic_Command(
                HostAdapter,
                BusLogic_DisableHostAdapterInterrupt,
                core::ptr::addr_of_mut!(Parameter) as *mut core::ffi::c_void,
                core::mem::size_of::<CHAR>() as core::ffi::c_int,
                core::ptr::null_mut(),
                0 as core::ffi::c_int,
            ) < 0 as core::ffi::c_int
            {
                return ScsiAdapterControlUnsuccessful;
            }
        }
        2 => {
            if !(BusLogic_HardwareResetHostAdapter(HostAdapter, 1 as BOOLEAN) as core::ffi::c_int
                != 0
                && BusLogic_InitializeHostAdapter(deviceExtension, core::ptr::null_mut())
                    as core::ffi::c_int
                    != 0
                && BT958HwInitialize(HwDeviceExtension) as core::ffi::c_int != 0)
            {
                ScsiDebugPrint(
                    1 as ULONG,
                    b"\n Buslogic - Resetting %s Failed\n\0".as_ptr() as PCCHAR,
                    core::ptr::addr_of_mut!((*HostAdapter).FullModelName) as *mut UCHAR,
                );
                return ScsiAdapterControlUnsuccessful;
            }

            ScsiPortStallExecution((*HostAdapter).BusSettleTime as ULONG);
        }
        _ => return ScsiAdapterControlUnsuccessful,
    }
    return ScsiAdapterControlSuccess;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * vmscsi-- Miniport driver for the Buslogic BT 958 SCSI Controller
//  *          under Windows 2000/XP/Server 2003
//  *
//  *          Based in parts on the buslogic driver for the same device
//  *          available with the GNU Linux Operating System.
//  *
//  * This program is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU General Public License
//  * as published by the Free Software Foundation; either version 2
//  * of the License, or (at your option) any later version.
//  *
//  * This program is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  * GNU General Public License for more details.
//  *
//  * You should have received a copy of the GNU General Public License along
//  * with this program; if not, write to the Free Software Foundation, Inc.,
//  * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// //____________________________________________________________________________________
// //
// // File description :The driver for BusLogic-BT958 SCSI Host adapter card.
// // This driver assumes that the host adapter(HA from now on) is a PCI device.
// // This is an effort to build a driver for this card for the windows XP environment
// // since the native XP installation doesnt provide this driver.
// //
// // The technical reference for this device is at :
// //
// // Author: Namita Lal, Sirish Raghuram Calsoft Pvt Ltd
// // Date:   5th Feb 2003
// // Status: Driver version 1.2.0.0
// //         Performance tuned
// //         Correctness tested for
// //            1. Installation at OS install time
// //            2. Installation in an installed OS
// //            3. Installation by upgrading a previous version
// //         on all flavours of WinXP and Win Server 2003
// //         For Win2k however, please refer PR 22812
// /*
// Revision History:
// v1.0.0.4  // Pre final release to VMware in Sep 2001, without WMI
//     |
//     |
//     v
// v1.0.0.5  // Sep 2001 release to VMware, with WMI
//     |
//     |
//     v
// v1.0.0.6  // Fix for bug with Nero Burning ROM in XP
//     |     // where SCSI_AUTO_SENSE is turned off at times
//     |
//     |
//     |---------> v1.1.0.0    // Performance optimizations:
//     |           |           // A. WMI disabled, B. Queueing depth increased
//     |           |           // C. Control flow changed (bug)
//     |           |
//     |           v
//     |       v1.1.0.1    // Fix for .NET restart freeze with 1.1.0.0
//     |                   // Breaks on XP
//     |
//     v
// v1.2.0.0  // A. WMI disabled, B. Queueing depth increased
//     |
//     |
//     |
//     v
// v1.2.0.1  // Fix PR 40284, affects pure ACPI Win2k systems.
//     |
//     |
//     |
//     v
// v1.2.0.2  // Fix PR 40284 correctly, disable interrupts in the initialization routine.
//           // CURRENT VERSION
// */
// //____________________________________________________________________________________
//
// #include "BusLogic958.h"
//
// ULONG
// NTAPI
// DriverEntry(IN PVOID DriverObject,
//             IN PVOID Argument2
//            )
// //_________________________________________________________________________
// // Routine Description:
// //              Installable driver initialization entry point for system.
// // Arguments:
// //              Driver Object
// // Return Value:
// //              Status from ScsiPortInitialize()
// //_________________________________________________________________________
// {
//     HW_INITIALIZATION_DATA hwInitializationData;
//     ULONG Status;
//     ULONG i;
//     ULONG HwContext;
// //    static int    cardNo = 0;
//
//     UCHAR VendorId[4] = { '1', '0', '4', 'b'        };
//     UCHAR DeviceId[4] = { '1', '0', '4', '0'        };
//
//     ScsiDebugPrint((TRACE,"\n BusLogic -  Inside the DriverEntry function \n"));
//
//     // Zero out structure.
//     for (i = 0; i < sizeof(HW_INITIALIZATION_DATA); i++)
//     {
//         ((PUCHAR) & hwInitializationData)[i] = 0;
//     }
//
//     // Set size of hwInitializationData.
//     hwInitializationData.HwInitializationDataSize = sizeof(HW_INITIALIZATION_DATA);
//
//     // Set entry points.
//     hwInitializationData.HwInitialize =     BT958HwInitialize;
//     hwInitializationData.HwResetBus =       BT958HwResetBus;
//     hwInitializationData.HwStartIo =        BT958HwStartIO;
//     hwInitializationData.HwInterrupt =      BT958HwInterrupt;
//     hwInitializationData.HwAdapterControl = BT958HwAdapterControl;
//     hwInitializationData.HwFindAdapter =    BT958HwFindAdapter;
//
//     // Indicate no buffer mapping but will need physical addresses
//     hwInitializationData.NeedPhysicalAddresses = TRUE;
//
//     // Indicate Auto request sense is supported
//     hwInitializationData.AutoRequestSense = TRUE;
//     hwInitializationData.MultipleRequestPerLu = TRUE;
//
// #if TAG_QUEUING
//     hwInitializationData.TaggedQueuing = TRUE;
// #else
//     hwInitializationData.TaggedQueuing = FALSE;
// #endif
//
//     hwInitializationData.AdapterInterfaceType = PCIBus;
//
//     // Fill in the vendor id and the device id
//     hwInitializationData.VendorId = &VendorId;
//     hwInitializationData.VendorIdLength = 4;
//     hwInitializationData.DeviceId = &DeviceId;
//     hwInitializationData.DeviceIdLength = 4;
//
//
//     hwInitializationData.NumberOfAccessRanges = 2;
//
//
//     // Specify size of extensions.
//     hwInitializationData.DeviceExtensionSize = sizeof(HW_DEVICE_EXTENSION);
//
//     // logical unit extension
//     hwInitializationData.SrbExtensionSize = sizeof(BusLogic_CCB_T);
//
//     HwContext = 0;
//
//     ScsiDebugPrint((TRACE,"\n BusLogic -  Calling the ScsiPortInitialize Routine\n"));
//
//     Status = ScsiPortInitialize(DriverObject,
//                                 Argument2,
//                                 &hwInitializationData,
//                                 &HwContext);
//
//     ScsiDebugPrint((TRACE,"\n BusLogic -  Exiting the DriverEntry function \n"));
//     ScsiDebugPrint((INFO,"\n BusLogic - Status = %ul \n", Status));
//     return( Status );
//
// } // end DriverEntry()
//
//
// ULONG
// NTAPI
// BT958HwFindAdapter(IN PVOID HwDeviceExtension,
//                    IN PVOID Context,
//                    IN PVOID BusInformation,
//                    IN PCHAR ArgumentString,
//                    IN OUT PPORT_CONFIGURATION_INFORMATION ConfigInfo,
//                    OUT PBOOLEAN  Again
//                   )
// //_________________________________________________________________________________________________
// // Routine Description:
// //              This function is called by the OS-specific port driver after the necessary storage
// //              has been allocated, to gather information about the adapter's configuration.
// //
// // Arguments:
// //              HwDeviceExtension - HBA miniport driver's adapter data storage
// //              Context - Register base address
// //              ConfigInfo - Configuration information structure describing HBA
// //              This structure is defined in PORT.H.
// //
// // Return Value:
// //              HwScsiFindAdapter must return one of the following status values:
// //              SP_RETURN_FOUND: Indicates a supported HBA was found and that the HBA-relevant
// //                               configuration information was successfully determined and set in
// //                               the PORT_CONFIGURATION_INFORMATION structure.
// //              SP_RETURN_ERROR: Indicates an HBA was found but there was error obtaining the
// //                               configuration information. If possible, such an error should be
// //                               logged with ScsiPortLogError.
// //              SP_RETURN_BAD_CONFIG: Indicates the supplied configuration information was invalid
// //                                    for the adapter.
// //              SP_RETURN_NOT_FOUND: Indicates no supported HBA was found for the supplied
// //                                   configuration information.
// //________________________________________________________________________________________________
// {
//     PHW_DEVICE_EXTENSION deviceExtension = HwDeviceExtension;
//     BusLogic_HostAdapter_T *hcsp = &(deviceExtension->hcs);
// //    static UCHAR k = 0;
//     PACCESS_RANGE   accessRange;
// //    PCI_COMMON_CONFIG PCICommonConfig;
//     PUCHAR   pciAddress;
//     char    NumPort = 0;
//
//     ScsiDebugPrint((TRACE,"\n BusLogic -  Inside the Find Adapter Routine\n"));
//
//     *Again = FALSE;
//
//     accessRange = &((*(ConfigInfo->AccessRanges))[0]);
//
//     // Inform SCSIPORT that we are NOT a WMI data provider
//     // Sirish, 10th June 2002
//     ConfigInfo->WmiDataProvider = FALSE;
//     /*Sirish, 10th June 2002 BT958WmiInitialize(deviceExtension);*/
//
//     // Check for configuration information passed in form the system
//     if ((*ConfigInfo->AccessRanges)[0].RangeLength != 0)
//     {
//         // check if the system supplied bus-relative address is valid and has not been
//         // claimed by another device
//         if ( ScsiPortValidateRange(deviceExtension,
//                                    ConfigInfo->AdapterInterfaceType,
//                                    ConfigInfo->SystemIoBusNumber,
//                                    accessRange->RangeStart,
//                                    accessRange->RangeLength,
//                                    TRUE) )  // TRUE: iniospace
//         {
//             ScsiDebugPrint((INFO,"\n BusLogic - Validate Range function succeeded \n"));
//
//             // Map the Bus-relative range addresses to system-space logical range addresses
//             // so that these mapped logical addresses can be called with SciPortRead/Writexxx
//             // to determine whether the adapter is an HBA that the driver supports
//
//             pciAddress = (PUCHAR) ScsiPortGetDeviceBase(deviceExtension,
//                                                        ConfigInfo->AdapterInterfaceType,
//                                                        ConfigInfo->SystemIoBusNumber,
//                                                        accessRange->RangeStart,
//                                                        accessRange->RangeLength,
//                                                        TRUE);  // TRUE: iniospace
//
//             if(pciAddress)
//             {
//                 ScsiDebugPrint((INFO,"\n BusLogic -  Get Device Base  function succeeded \n"));
//
//                 memset(hcsp, 0, sizeof(BusLogic_HostAdapter_T));
//
//                 // points to structure of type BT958_HA which has device specific information. This needs
//                 // to be either changed or modified with our specific info.
//                 hcsp->IO_Address = pciAddress;
//                 hcsp->IRQ_Channel = (UCHAR)ConfigInfo->BusInterruptLevel;
//                 NumPort++;
//             }
//         }
//     }
//
//     if (NumPort == 0)
//     {
//         return(SP_RETURN_NOT_FOUND);
//     }
//
//     //  Hardware found, let's find out hardware configuration
//     //  and fill out ConfigInfo table for WinNT
//     ConfigInfo->NumberOfBuses = 1;
//     ConfigInfo->MaximumTransferLength = MAX_TRANSFER_SIZE;
//
// #if SG_SUPPORT
//     ConfigInfo->ScatterGather = TRUE;
// #else
//     ConfigInfo->ScatterGather = FALSE;
// #endif
//
//     ConfigInfo->Master = TRUE;
//     ConfigInfo->NeedPhysicalAddresses = TRUE;
//     ConfigInfo->Dma32BitAddresses = TRUE;
//     ConfigInfo->InterruptMode =  LevelSensitive;
//
// #if TAG_QUEUING
//     ConfigInfo->TaggedQueuing = TRUE;
// #else
//     ConfigInfo->TaggedQueuing = FALSE;
// #endif
//
//     // Should we change this to double-word aligned to increase performance
//     ConfigInfo->AlignmentMask = 0x0;
//
//     if (!Buslogic_InitBT958(deviceExtension,ConfigInfo)) // hardware specific initializations. Find what's for our card
//     {
//        ScsiPortLogError(deviceExtension,
//                          NULL,
//                          0,
//                          0,
//                          0,
//                          SP_INTERNAL_ADAPTER_ERROR,
//                          7 << 8);
//
//        return(SP_RETURN_ERROR);
//     }
//
//     if (NumPort != 0)
//         *Again = TRUE;
//
//     return(SP_RETURN_FOUND);
//
// } // end BT958FindAdapter()
//
//
// BOOLEAN
// Buslogic_InitBT958(PHW_DEVICE_EXTENSION deviceExtension,
//                    PPORT_CONFIGURATION_INFORMATION ConfigInfo)
// //_________________________________________________________________________
// // Routine Description:
// //              This routine is called from the driver's FindAdapter routine
// //              On invocation this routine probes the host adapter to check
// //              if its hardware registers are responding correctly, and
// //              initializes the device and makes it ready for IO
// // Arguments:
// //              1. deviceExtension
// //              2. Port Configuration info
// // Return Value:
// //              TRUE : Device initialized properly
// //              FALSE : Device failed to initialize
// //_________________________________________________________________________
// {
//     CHAR ch;
//
//     BusLogic_HostAdapter_T *HostAdapter = &(deviceExtension->hcs);
//
//     // Probe the Host Adapter.
//     // If unsuccessful, abort further initialization.
//     if (!BusLogic_ProbeHostAdapter(HostAdapter))
//         return   FALSE;
//
//     // Hard Reset the Host Adapter.
//     // If unsuccessful, abort further initialization.
//     if (!BusLogic_HardwareResetHostAdapter(HostAdapter, TRUE))
//         return FALSE;
//
//     /*
//      * PR 40284 -- Disable interrupts until driver initialization is complete.
//      */
//     ch = 0;
//     if (BusLogic_Command(HostAdapter, BusLogic_DisableHostAdapterInterrupt,
// 			 &ch, sizeof(ch), NULL, 0) < 0) {
//         ScsiDebugPrint((WARNING, "\n BusLogic - Could not disable interrupts!\n"));
//     } else {
//         ScsiDebugPrint((INFO, "\n BusLogic - Disabled interrupts.\n"));
//     }
//
//     // Check the Host Adapter.
//     // If unsuccessful, abort further initialization.
//     if (!BusLogic_CheckHostAdapter(HostAdapter))
//         return FALSE;
//
//     // Allocate a Noncached Extension to use for mail boxes.
//     deviceExtension->NoncachedExtension =  ScsiPortGetUncachedExtension(deviceExtension,
//                                                                       ConfigInfo,
//                                                                       sizeof(NONCACHED_EXTENSION));
//
//     if (deviceExtension->NoncachedExtension == NULL)
//     {
//         // Log error.
//         ScsiPortLogError(deviceExtension,
//                              NULL,
//                              0,
//                              0,
//                              0,
//                              SP_INTERNAL_ADAPTER_ERROR,
//                              7 << 8);
//
//         // abort further initialization
//         return FALSE;
//     }
//
//     // Read the Host Adapter Configuration, Configure the Host Adapter,
//     // Acquire the System Resources necessary to use the Host Adapter, then
//     // Create the Initial CCBs, Initialize the Host Adapter, and finally
//     // perform Target Device Inquiry.
//     if (BusLogic_ReadHostAdapterConfiguration(HostAdapter)           &&
//         //BusLogic_ReportHostAdapterConfiguration(HostAdapter)       &&
//         BusLogic_InitializeHostAdapter(deviceExtension, ConfigInfo)  &&
//         BusLogic_TargetDeviceInquiry(HostAdapter))
//     {
//         // Fill in the:
//         // 1.Maximum number of scsi target devices that are supported by our adapter
//         ConfigInfo->MaximumNumberOfTargets = HostAdapter->MaxTargetDevices;
//         // 2. Maximum number of logical units per target the HBA can control.
//         ConfigInfo->MaximumNumberOfLogicalUnits  = HostAdapter->MaxLogicalUnits;
//         ConfigInfo->InitiatorBusId[0] = HostAdapter->SCSI_ID;
//         // Maximum number of breaks between address ranges that a data buffer can
//         // have if the HBA supports scatter/gather. In other words, the number of
//         // scatter/gather lists minus one.
//         ConfigInfo->NumberOfPhysicalBreaks = HostAdapter->DriverScatterGatherLimit;
//     }
//     else
//     {
//         // An error occurred during Host Adapter Configuration Querying, Host
//         // Adapter Configuration, Host Adapter Initialization, or Target Device Inquiry,
//         // so return FALSE
//
//         return FALSE;
//     }
//     // Initialization completed successfully
//     return TRUE;
// } // end Buslogic_InitBT958
//
//
// BOOLEAN
// BusLogic_ProbeHostAdapter(BusLogic_HostAdapter_T *HostAdapter)
// //____________________________________________________________________________________
// // Routine Description: BusLogic_ProbeHostAdapter probes for a BusLogic Host Adapter.
// //                      The routine reads the status, interrupt and geometry regiter and
// //                      checks if their contents are valid
// // Arguments:
// //              1. Host Adapter structure
// //
// // Return Value:
// //              TRUE: Probe completed successfully
// //              FALSE: Probe failed
// //____________________________________________________________________________________
// {
//   BusLogic_StatusRegister_T StatusRegister;
//   BusLogic_InterruptRegister_T InterruptRegister;
//   BusLogic_GeometryRegister_T GeometryRegister;
//
//   ScsiDebugPrint((TRACE,"\n BusLogic -  Inside ProbeHostaAdapter function \n"));
//   //  Read the Status, Interrupt, and Geometry Registers to test if there are I/O
//   //  ports that respond, and to check the values to determine if they are from a
//   //  BusLogic Host Adapter.  A nonexistent I/O port will return 0xFF, in which
//   //  case there is definitely no BusLogic Host Adapter at this base I/O Address.
//   //  The test here is a subset of that used by the BusLogic Host Adapter BIOS.
//
//   *InterruptRegister.All.as_mut() = BusLogic_ReadInterruptRegister(HostAdapter);
//   *GeometryRegister.All.as_mut() = BusLogic_ReadGeometryRegister(HostAdapter);
//   *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
//
//   if (StatusRegister.All == 0 ||
//       StatusRegister.Bits.DiagnosticActive ||
//       StatusRegister.Bits.CommandParameterRegisterBusy ||
//       StatusRegister.Bits.Reserved ||
//       StatusRegister.Bits.CommandInvalid ||
//       InterruptRegister.Bits.Reserved != 0)
//     return FALSE;
//
//   //  Check the undocumented Geometry Register to test if there is an I/O port
//   //  that responded.  Adaptec Host Adapters do not implement the Geometry
//   //  Register, so this test helps serve to avoid incorrectly recognizing an
//   //  Adaptec 1542A or 1542B as a BusLogic.  Unfortunately, the Adaptec 1542C
//   //  series does respond to the Geometry Register I/O port, but it will be
//   //  rejected later when the Inquire Extended Setup Information command is
//   //  issued in BusLogic_CheckHostAdapter.  The AMI FastDisk Host Adapter is a
//   //  BusLogic clone that implements the same interface as earlier BusLogic
//   //  Host Adapters, including the undocumented commands, and is therefore
//   //  supported by this driver.  However, the AMI FastDisk always returns 0x00
//   //  upon reading the Geometry Register, so the extended translation option
//   //  should always be left disabled on the AMI FastDisk.
//
//   if (GeometryRegister.All == 0xFF) return FALSE;
//   return TRUE;
// }// end BusLogic_ProbeHostAdapter
//
// BOOLEAN
// BusLogic_HardwareResetHostAdapter(BusLogic_HostAdapter_T  *HostAdapter,
//                                   BOOLEAN HardReset)
// //_________________________________________________________________________
// // Routine Description: BusLogic_HardwareResetHostAdapter issues a Hardware
// //                      Reset to the Host Adapter and waits for Host Adapter
// //                      Diagnostics to complete.  If HardReset is TRUE, a
// //                      Hard Reset is performed which also initiates a SCSI
// //                      Bus Reset.  Otherwise, a Soft Reset is performed which
// //                      only resets the Host Adapter without forcing a SCSI
// //                      Bus Reset.
// // Arguments:
// //              1. Host Adapter structure
// //              2. Boolean HardReset - True: Do hard reset
// // Return Value:
// //              TRUE : Reset completed successfully
// //              FALSE : Reset failed
// //_________________________________________________________________________
// {
//   BusLogic_StatusRegister_T StatusRegister;
//   int TimeoutCounter;
//
//   //  Issue a Hard Reset or Soft Reset Command to the Host Adapter.  The Host
//   //  Adapter should respond by setting Diagnostic Active in the Status Register.
//   if (HardReset)
//     BusLogic_HardReset(HostAdapter);
//   else
//     BusLogic_SoftReset(HostAdapter);
//
//   // Wait until Diagnostic Active is set in the Status Register.
//   TimeoutCounter = 100;
//   while (--TimeoutCounter >= 0)
//   {
//       *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
//       if (StatusRegister.Bits.DiagnosticActive)
//           break;
//   }
//
//   // if in spite of waiting for time out period , if it didn't et set, then something is wrong-- so just return.
//   if (TimeoutCounter < 0)
//       return FALSE;
//
//   //
//   //  Wait 100 microseconds to allow completion of any initial diagnostic
//   //  activity which might leave the contents of the Status Register
//   //  unpredictable.
//   ScsiPortStallExecution(100);
//
//   //  Wait until Diagnostic Active is reset in the Status Register.
//   TimeoutCounter = 10*10000;
//   while (--TimeoutCounter >= 0)
//   {
//       *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
//       if (!StatusRegister.Bits.DiagnosticActive)
//           break;
//       ScsiPortStallExecution(100);
//   }
//
//   if (TimeoutCounter < 0)
//       return FALSE;
//
//   //  Wait until at least one of the Diagnostic Failure, Host Adapter Ready,
//   //  or Data In Register Ready bits is set in the Status Register.
//   TimeoutCounter = 10000;
//   while (--TimeoutCounter >= 0)
//   {
//       *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
//       if (StatusRegister.Bits.DiagnosticFailure ||
//           StatusRegister.Bits.HostAdapterReady ||
//           StatusRegister.Bits.DataInRegisterReady)
//       {
//         break;
//       }
//       ScsiPortStallExecution(100);
//    }
//
//   //device didn't respond to reset
//   if (TimeoutCounter < 0)
//       return FALSE;
//
//   //  If Diagnostic Failure is set or Host Adapter Ready is reset, then an
//   //  error occurred during the Host Adapter diagnostics.  If Data In Register
//   //  Ready is set, then there is an Error Code available.
//   if (StatusRegister.Bits.DiagnosticFailure || !StatusRegister.Bits.HostAdapterReady)
//   {
//       ScsiDebugPrint((ERROR, "\n BusLogic - Failure - HOST ADAPTER STATUS REGISTER = %02X\n", (*StatusRegister.All.as_ref())));
//
//       if (StatusRegister.Bits.DataInRegisterReady)
//       {
//         ScsiDebugPrint((ERROR, "HOST ADAPTER ERROR CODE = %d\n", BusLogic_ReadDataInRegister(HostAdapter)));
//       }
//       return FALSE;
//   }
//
//   //  Indicate the Host Adapter Hard Reset completed successfully.
//   return TRUE;
// }// end BusLogic_HardwareResetHostAdapter
//
//
// BOOLEAN
// BusLogic_CheckHostAdapter(BusLogic_HostAdapter_T *HostAdapter)
// //_________________________________________________________________________
// // Routine Description: BusLogic_CheckHostAdapter checks to be sure this
// //                      really is a BusLogic
// // Arguments:
// //              1. Host Adapter Structure
// // Return Value:
// //              TRUE : Buslogic adapter detected
// //              FALSE : Card is not a Buslogic adapter
// //_________________________________________________________________________
// {
//   BusLogic_ExtendedSetupInformation_T ExtendedSetupInformation;
//   BusLogic_RequestedReplyLength_T RequestedReplyLength;
//   BOOLEAN Result = TRUE;
//
//   ScsiDebugPrint((TRACE, "\n BusLogic -  Inside BusLogic_CheckHostAdapter function \n"));
//
//   //
//   //  Issue the Inquire Extended Setup Information command.  Only genuine
//   //  BusLogic Host Adapters and TRUE clones support this command.  Adaptec 1542C
//   //  series Host Adapters that respond to the Geometry Register I/O port will
//   //  fail this command.
//   RequestedReplyLength = sizeof(ExtendedSetupInformation);
//   if (BusLogic_Command(HostAdapter,
//                        BusLogic_InquireExtendedSetupInformation,
//                        &RequestedReplyLength,
//                        sizeof(RequestedReplyLength),
//                        &ExtendedSetupInformation,
//                        sizeof(ExtendedSetupInformation))
//       != sizeof(ExtendedSetupInformation))
//   {
//     Result = FALSE;
//   }
//   return Result;
// }// end BusLogic_CheckHostAdapter
//
// int
// BusLogic_Command(BusLogic_HostAdapter_T *HostAdapter,
//                  BusLogic_OperationCode_T OperationCode,
//                  void *ParameterData,
//                  int ParameterLength,
//                  void *ReplyData,
//                  int ReplyLength)
// //______________________________________________________________________________________________
// // Routine Description:
// //                      BusLogic_Command sends the command OperationCode to HostAdapter, optionally
// //                      providing ParameterLength bytes of ParameterData and receiving at most
// //                      ReplyLength bytes of ReplyData; any excess reply data is received but
// //                      discarded.
// //
// //                      BusLogic_Command is called exclusively during host adapter detection and
// //                      initialization, so performance and latency are not critical, and exclusive
// //                      access to the Host Adapter hardware is assumed.  Once the host adapter and
// //                      driver are initialized, the only Host Adapter command that is issued is the
// //                      single byte Execute Mailbox Command operation code, which does not require
// //                      waiting for the Host Adapter Ready bit to be set in the Status Register.
// // Arguments:
// //              1. HostAdapter - Host Adapter structure
// //              2. OperationCode - Operation code for the command
// //              3. ParameterData - Buffer containing parameters that needs to be passed as part
// //                 of the command
// //              4. ParameterLength - Number of parameters
// //              5. ReplyData - Buffer where reply data is copied
// //              6. ReplyLength - The length of the reply data
// // Return Value:
// //              On success, this function returns the number of reply bytes read from
// //              the Host Adapter (including any discarded data); on failure, it returns
// //              -1 if the command was invalid, or -2 if a timeout occurred.
// //_________________________________________________________________________________________________
// {
//   UCHAR *ParameterPointer = (UCHAR *) ParameterData;
//   UCHAR *ReplyPointer = (UCHAR *) ReplyData;
//   BusLogic_StatusRegister_T StatusRegister;
//   BusLogic_InterruptRegister_T InterruptRegister;
//   int ReplyBytes = 0, Result;
//   long TimeoutCounter;
//
//   //  Clear out the Reply Data if provided.
//   if (ReplyLength > 0)
//     memset(ReplyData, 0, ReplyLength);
//
//   //  If the IRQ Channel has not yet been acquired, then interrupts must be
//   //  disabled while issuing host adapter commands since a Command Complete
//   //  interrupt could occur if the IRQ Channel was previously enabled by another
//   //  BusLogic Host Adapter or another driver sharing the same IRQ Channel.
//
//   //  Wait for the Host Adapter Ready bit to be set and the Command/Parameter
//   //  Register Busy bit to be reset in the Status Register.
//   TimeoutCounter = 10000;
//   while (--TimeoutCounter >= 0)
//   {
//       *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
//       if ( StatusRegister.Bits.HostAdapterReady &&
//           !StatusRegister.Bits.CommandParameterRegisterBusy)
//       {
//             break;
//       }
//       ScsiPortStallExecution(100);
//   }
//   if (TimeoutCounter < 0)
//   {
//       //BusLogic_CommandFailureReason = "Timeout waiting for Host Adapter Ready";
//       Result = -2;
//       goto Done;
//   }
//
//   //  Write the OperationCode to the Command/Parameter Register.
//   HostAdapter->HostAdapterCommandCompleted = FALSE;
//   BusLogic_WriteCommandParameterRegister(HostAdapter, (UCHAR)OperationCode);
//
//   //Write any additional Parameter Bytes.
//   TimeoutCounter = 10000;
//   while (ParameterLength > 0 && --TimeoutCounter >= 0)
//   {
//     //
//     // Wait 100 microseconds to give the Host Adapter enough time to determine
//     // whether the last value written to the Command/Parameter Register was
//     // valid or not.  If the Command Complete bit is set in the Interrupt
//     // Register, then the Command Invalid bit in the Status Register will be
//     // reset if the Operation Code or Parameter was valid and the command
//     // has completed, or set if the Operation Code or Parameter was invalid.
//     // If the Data In Register Ready bit is set in the Status Register, then
//     // the Operation Code was valid, and data is waiting to be read back
//     // from the Host Adapter.  Otherwise, wait for the Command/Parameter
//     // Register Busy bit in the Status Register to be reset.
//
//       ScsiPortStallExecution(100);
//       *InterruptRegister.All.as_mut() = BusLogic_ReadInterruptRegister(HostAdapter);
//       *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
//       if (InterruptRegister.Bits.CommandComplete)
//           break;
//       if (HostAdapter->HostAdapterCommandCompleted)
//           break;
//       if (StatusRegister.Bits.DataInRegisterReady)
//           break;
//       if (StatusRegister.Bits.CommandParameterRegisterBusy)
//           continue;
//       BusLogic_WriteCommandParameterRegister(HostAdapter, *ParameterPointer++);
//       ParameterLength--;
//   }
//   if (TimeoutCounter < 0)
//   {
//       //BusLogic_CommandFailureReason =  "Timeout waiting for Parameter Acceptance";
//       Result = -2;
//       goto Done;
//   }
//
//   // The Modify I/O Address command does not cause a Command Complete Interrupt.
//   if (OperationCode == BusLogic_ModifyIOAddress)
//   {
//     *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
//     if (StatusRegister.Bits.CommandInvalid)
//     {
//       //BusLogic_CommandFailureReason = "Modify I/O Address Invalid";
//       Result = -1;
//       goto Done;
//     }
//     Result = 0;
//     goto Done;
//   }
//
//   //  Select an appropriate timeout value for awaiting command completion.
//   switch (OperationCode)
//   {
//     case BusLogic_InquireInstalledDevicesID0to7:
//     case BusLogic_InquireInstalledDevicesID8to15:
//     case BusLogic_InquireTargetDevices:
//       // Approximately 60 seconds.
//       TimeoutCounter = 60*10000;
//       break;
//     default:
//       // Approximately 1 second.
//       TimeoutCounter = 10000;
//       break;
//   }
//
//   //
//   //  Receive any Reply Bytes, waiting for either the Command Complete bit to
//   //  be set in the Interrupt Register, or for the Interrupt Handler to set the
//   //  Host Adapter Command Completed bit in the Host Adapter structure.
//   while (--TimeoutCounter >= 0)
//   {
//     *InterruptRegister.All.as_mut() = BusLogic_ReadInterruptRegister(HostAdapter);
//     *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
//     if (InterruptRegister.Bits.CommandComplete)
//         break;
//     if (HostAdapter->HostAdapterCommandCompleted)
//         break;
//     if (StatusRegister.Bits.DataInRegisterReady)
//     {
//       if (++ReplyBytes <= ReplyLength)
//       {
//         *ReplyPointer++ = BusLogic_ReadDataInRegister(HostAdapter);
//       }
//       else
//       {
//           BusLogic_ReadDataInRegister(HostAdapter);
//       }
//     }
//     if (OperationCode == BusLogic_FetchHostAdapterLocalRAM &&
//         StatusRegister.Bits.HostAdapterReady)
//     {
//       break;
//     }
//     ScsiPortStallExecution(100);
//   }
//   if (TimeoutCounter < 0)
//   {
//       //BusLogic_CommandFailureReason = "Timeout waiting for Command Complete";
//       Result = -2;
//       goto Done;
//   }
//
//   //  Clear any pending Command Complete Interrupt.
//   BusLogic_InterruptReset(HostAdapter);
//
//   //  Process Command Invalid conditions.
//   if (StatusRegister.Bits.CommandInvalid)
//   {
//     //
//     // Some early BusLogic Host Adapters may not recover properly from
//     // a Command Invalid condition, so if this appears to be the case,
//     // a Soft Reset is issued to the Host Adapter.  Potentially invalid
//     // commands are never attempted after Mailbox Initialization is
//     // performed, so there should be no Host Adapter state lost by a
//     // Soft Reset in response to a Command Invalid condition.
//
//     ScsiPortStallExecution(1000);
//     *StatusRegister.All.as_mut() = BusLogic_ReadStatusRegister(HostAdapter);
//     if (StatusRegister.Bits.CommandInvalid             ||
//       StatusRegister.Bits.Reserved                     ||
//       StatusRegister.Bits.DataInRegisterReady          ||
//       StatusRegister.Bits.CommandParameterRegisterBusy ||
//       !StatusRegister.Bits.HostAdapterReady            ||
//       !StatusRegister.Bits.InitializationRequired      ||
//       StatusRegister.Bits.DiagnosticActive             ||
//       StatusRegister.Bits.DiagnosticFailure)
//     {
//         BusLogic_SoftReset(HostAdapter);
//         ScsiPortStallExecution(1000);
//     }
//     //BusLogic_CommandFailureReason = "Command Invalid";
//     Result = -1;
//     goto Done;
//   }
//
//   // Handle Excess Parameters Supplied conditions.
//   if (ParameterLength > 0)
//   {
//       //BusLogic_CommandFailureReason = "Excess Parameters Supplied";
//       Result = -1;
//       goto Done;
//   }
//
//   //  Indicate the command completed successfully.
//   Result = ReplyBytes;
//
//   //  Restore the interrupt status if necessary and return.
// Done:
//   return Result;
// }// end BusLogic_Command
//
//
// BOOLEAN
// BusLogic_ReadHostAdapterConfiguration(BusLogic_HostAdapter_T  *HostAdapter)
// //___________________________________________________________________________________________
// // Routine Description:
// //                  BusLogic_ReadHostAdapterConfiguration reads the Configuration Information
// //                  from Host Adapter and initializes the Host Adapter structure.
// // Arguments:
// //              1. Host adapter structure
// // Return Value:
// //              TRUE : Configuration read properly
// //              FALSE : Encounter failure
// //___________________________________________________________________________________________
// {
//   BusLogic_BoardID_T BoardID;
//   BusLogic_Configuration_T Configuration;
//   BusLogic_SetupInformation_T SetupInformation;
//   BusLogic_ExtendedSetupInformation_T ExtendedSetupInformation;
//   BusLogic_HostAdapterModelNumber_T HostAdapterModelNumber;
//   BusLogic_FirmwareVersion3rdDigit_T FirmwareVersion3rdDigit;
//   BusLogic_FirmwareVersionLetter_T FirmwareVersionLetter;
//   BusLogic_PCIHostAdapterInformation_T PCIHostAdapterInformation;
//   BusLogic_FetchHostAdapterLocalRAMRequest_T FetchHostAdapterLocalRAMRequest;
//   BusLogic_AutoSCSIData_T AutoSCSIData;
//   BusLogic_GeometryRegister_T GeometryRegister;
//   BusLogic_RequestedReplyLength_T RequestedReplyLength;
//   UCHAR *TargetPointer, Character;
//   ULONG /*TargetID,*/ i;
//
//
//   //  Issue the Inquire Board ID command.
//   if (BusLogic_Command(HostAdapter,
//                        BusLogic_InquireBoardID,
//                        NULL,
//                        0,
//                        &BoardID, sizeof(BoardID))
//       != sizeof(BoardID))
//   {
//         ScsiDebugPrint((ERROR, "\n BusLogic - Failure: INQUIRE BOARD ID\n"));
//         return FALSE;
//   }
//
//   //  Issue the Inquire Configuration command.
//   if (BusLogic_Command(HostAdapter,
//                        BusLogic_InquireConfiguration,
//                        NULL,
//                        0,
//                        &Configuration, sizeof(Configuration))
//       != sizeof(Configuration))
//   {
//       ScsiDebugPrint((ERROR, "\n BusLogic - Failure: INQUIRE CONFIGURATION\n"));
//       return FALSE;
//   }
//
//   //  Issue the Inquire Setup Information command.
//   RequestedReplyLength = sizeof(SetupInformation);
//   if (BusLogic_Command(HostAdapter,
//                        BusLogic_InquireSetupInformation,
//                        &RequestedReplyLength,
//                        sizeof(RequestedReplyLength),
//                        &SetupInformation,
//                        sizeof(SetupInformation))
//       != sizeof(SetupInformation))
//   {
//      ScsiDebugPrint((ERROR, "\n BusLogic - Failure: INQUIRE SETUP INFORMATION\n"));
//      return FALSE;
//   }
//
//   //  Issue the Inquire Extended Setup Information command.
//   RequestedReplyLength = sizeof(ExtendedSetupInformation);
//   if (BusLogic_Command(HostAdapter,
//                        BusLogic_InquireExtendedSetupInformation,
//                        &RequestedReplyLength,
//                        sizeof(RequestedReplyLength),
//                        &ExtendedSetupInformation,
//                        sizeof(ExtendedSetupInformation))
//       != sizeof(ExtendedSetupInformation))
//   {
//       ScsiDebugPrint((ERROR, "\n BusLogic - Failure: INQUIRE EXTENDED SETUP INFORMATION\n"));
//       return FALSE;
//   }
//
//   //  Issue the Inquire Firmware Version 3rd Digit command.
//   FirmwareVersion3rdDigit = '\0';
//   if (BoardID.FirmwareVersion1stDigit > '0')
//   {
//     if (BusLogic_Command(HostAdapter,
//                          BusLogic_InquireFirmwareVersion3rdDigit,
//                          NULL,
//                          0,
//                          &FirmwareVersion3rdDigit,
//                          sizeof(FirmwareVersion3rdDigit))
//         != sizeof(FirmwareVersion3rdDigit))
//     {
//         ScsiDebugPrint((ERROR, "\n BusLogic - Failure: INQUIRE FIRMWARE 3RD DIGIT\n"));
//         return FALSE;
//     }
//   }
//
//   //  Issue the Inquire Host Adapter Model Number command.
//   RequestedReplyLength = sizeof(HostAdapterModelNumber);
//   if (BusLogic_Command(HostAdapter,
//                        BusLogic_InquireHostAdapterModelNumber,
//                        &RequestedReplyLength,
//                        sizeof(RequestedReplyLength),
//                        &HostAdapterModelNumber,
//                        sizeof(HostAdapterModelNumber))
//       != sizeof(HostAdapterModelNumber))
//   {
//       ScsiDebugPrint((ERROR, "\n BusLogic - Failure: INQUIRE HOST ADAPTER MODEL NUMBER\n"));
//       return FALSE;
//   }
//
//   //  BusLogic MultiMaster Host Adapters can be identified by their model number
//   //  and the major version number of their firmware as follows:
//   //
//   //  5.xx  BusLogic "W" Series Host Adapters:
//   //          BT-948/958/958D
//   //  Save the Model Name and Host Adapter Name in the Host Adapter structure.
//   TargetPointer = HostAdapter->ModelName;
//   *TargetPointer++ = 'B';
//   *TargetPointer++ = 'T';
//   *TargetPointer++ = '-';
//   for (i = 0; i < sizeof(HostAdapterModelNumber); i++)
//   {
//       Character = HostAdapterModelNumber[i];
//       if (Character == ' ' || Character == '\0') break;
//       *TargetPointer++ = Character;
//   }
//   *TargetPointer++ = '\0';
//
//   // Save the Firmware Version in the Host Adapter structure.
//   TargetPointer = HostAdapter->FirmwareVersion;
//   *TargetPointer++ = BoardID.FirmwareVersion1stDigit;
//   *TargetPointer++ = '.';
//   *TargetPointer++ = BoardID.FirmwareVersion2ndDigit;
//   if (FirmwareVersion3rdDigit != ' ' && FirmwareVersion3rdDigit != '\0')
//     *TargetPointer++ = FirmwareVersion3rdDigit;
//   *TargetPointer = '\0';
//
//   // Issue the Inquire Firmware Version Letter command.
//   if (strcmp((char*)HostAdapter->FirmwareVersion, "3.3") >= 0)
//   {
//       if (BusLogic_Command(HostAdapter,
//                            BusLogic_InquireFirmwareVersionLetter,
//                            NULL,
//                            0,
//                            &FirmwareVersionLetter,
//                            sizeof(FirmwareVersionLetter))
//           != sizeof(FirmwareVersionLetter))
//       {
//           ScsiDebugPrint((ERROR, "\n BusLogic - Failure: INQUIRE FIRMWARE VERSION LETTER\n"));
//           return FALSE;
//       }
//       if (FirmwareVersionLetter != ' ' && FirmwareVersionLetter != '\0')
//       {
//         *TargetPointer++ = FirmwareVersionLetter;
//       }
//       *TargetPointer = '\0';
//   }
//
//
//   //  Save the Host Adapter SCSI ID in the Host Adapter structure.
//   HostAdapter->SCSI_ID = Configuration.HostAdapterID;
//
//   //  Determine the Bus Type and save it in the Host Adapter structure, determine
//   //  and save the IRQ Channel if necessary, and determine and save the DMA
//   //  Channel for ISA Host Adapters.
//   HostAdapter->HostAdapterBusType =  BusLogic_HostAdapterBusTypes[HostAdapter->ModelName[3] - '4'];
//
//   *GeometryRegister.All.as_mut() = BusLogic_ReadGeometryRegister(HostAdapter);
//
//   //  Determine whether Extended Translation is enabled and save it in
//   //  the Host Adapter structure.
//   //  HostAdapter->ExtendedTranslationEnabled =  GeometryRegister.Bits.ExtendedTranslationEnabled;
//   HostAdapter->ExtendedTranslationEnabled =  (*GeometryRegister.All.as_ref());
//
//
//   //  Save the Scatter Gather Limits, Level Sensitive Interrupt flag, Wide
//   //  SCSI flag, Differential SCSI flag, SCAM Supported flag, and
//   //  Ultra SCSI flag in the Host Adapter structure.
//   HostAdapter->HostAdapterScatterGatherLimit =  ExtendedSetupInformation.ScatterGatherLimit;
//
//   HostAdapter->DriverScatterGatherLimit =   HostAdapter->HostAdapterScatterGatherLimit;
//
//   if (HostAdapter->HostAdapterScatterGatherLimit > BusLogic_ScatterGatherLimit)
//   {
//     HostAdapter->DriverScatterGatherLimit = BusLogic_ScatterGatherLimit;
//   }
//   if (ExtendedSetupInformation.Misc.LevelSensitiveInterrupt)
//   {
//     HostAdapter->LevelSensitiveInterrupt = TRUE;
//   }
//
//   HostAdapter->HostWideSCSI = ExtendedSetupInformation.HostWideSCSI;
//
//   HostAdapter->HostDifferentialSCSI =  ExtendedSetupInformation.HostDifferentialSCSI;
//
//   HostAdapter->HostSupportsSCAM = ExtendedSetupInformation.HostSupportsSCAM;
//
//   HostAdapter->HostUltraSCSI = ExtendedSetupInformation.HostUltraSCSI;
//
//
//   //  Determine whether Extended LUN Format CCBs are supported and save the
//   //  information in the Host Adapter structure.
//   if (HostAdapter->FirmwareVersion[0] == '5' ||
//       (HostAdapter->FirmwareVersion[0] == '4' && HostAdapter->HostWideSCSI))
//   {
//     HostAdapter->ExtendedLUNSupport = TRUE;
//   }
//   // Issue the Inquire PCI Host Adapter Information command to read the
//   // Termination Information from "W" series MultiMaster Host Adapters.
//   if (HostAdapter->FirmwareVersion[0] == '5')
//   {
//       if (BusLogic_Command(HostAdapter,
//                            BusLogic_InquirePCIHostAdapterInformation,
//                            NULL,
//                            0,
//                            &PCIHostAdapterInformation,
//                            sizeof(PCIHostAdapterInformation))
//           != sizeof(PCIHostAdapterInformation))
//       {
//           ScsiDebugPrint((ERROR, "\n BusLogic - Failure: INQUIRE PCI HOST ADAPTER INFORMATION\n"));
//           return FALSE;
//       }
//
//        // Save the Termination Information in the Host Adapter structure.
//       if (PCIHostAdapterInformation.GenericInfoValid)
//       {
//         HostAdapter->TerminationInfoValid = TRUE;
//         HostAdapter->LowByteTerminated =  PCIHostAdapterInformation.LowByteTerminated;
//         HostAdapter->HighByteTerminated = PCIHostAdapterInformation.HighByteTerminated;
//       }
//   }
//
//   //  Issue the Fetch Host Adapter Local RAM command to read the AutoSCSI data
//   //  from "W" and "C" series MultiMaster Host Adapters.
//   if (HostAdapter->FirmwareVersion[0] >= '4')
//   {
//       FetchHostAdapterLocalRAMRequest.ByteOffset =  BusLogic_AutoSCSI_BaseOffset;
//       FetchHostAdapterLocalRAMRequest.ByteCount = sizeof(AutoSCSIData);
//       if (BusLogic_Command(HostAdapter,
//                            BusLogic_FetchHostAdapterLocalRAM,
//                            &FetchHostAdapterLocalRAMRequest,
//                            sizeof(FetchHostAdapterLocalRAMRequest),
//                            &AutoSCSIData,
//                            sizeof(AutoSCSIData))
//           != sizeof(AutoSCSIData))
//       {
//           ScsiDebugPrint((ERROR, "\n BusLogic - Failure: FETCH HOST ADAPTER LOCAL RAM\n"));
//           return FALSE;
//       }
//       // Save the Parity Checking Enabled, Bus Reset Enabled, and Termination
//       // Information in the Host Adapter structure.
//       HostAdapter->ParityCheckingEnabled = AutoSCSIData.ParityCheckingEnabled;
//       HostAdapter->BusResetEnabled = AutoSCSIData.BusResetEnabled;
//
//       // Save the Wide Permitted, Fast Permitted, Synchronous Permitted,
//       // Disconnect Permitted, Ultra Permitted, and SCAM Information in the
//       // Host Adapter structure.
//       HostAdapter->WidePermitted = AutoSCSIData.WidePermitted;
//       HostAdapter->FastPermitted = AutoSCSIData.FastPermitted;
//       HostAdapter->SynchronousPermitted =   AutoSCSIData.SynchronousPermitted;
//       HostAdapter->DisconnectPermitted =    AutoSCSIData.DisconnectPermitted;
//       if (HostAdapter->HostUltraSCSI)
//       {
//         HostAdapter->UltraPermitted = AutoSCSIData.UltraPermitted;
//       }
//       if (HostAdapter->HostSupportsSCAM)
//       {
//           HostAdapter->SCAM_Enabled = AutoSCSIData.SCAM_Enabled;
//           HostAdapter->SCAM_Level2 = AutoSCSIData.SCAM_Level2;
//       }
//     }
//
//     // Determine the maximum number of Target IDs and Logical Units supported by
//     // this driver for Wide and Narrow Host Adapters.
//     HostAdapter->MaxTargetDevices = (HostAdapter->HostWideSCSI ? 16 : 8);
//     HostAdapter->MaxLogicalUnits = (HostAdapter->ExtendedLUNSupport ? 32 : 8);
//
//     // Select appropriate values for the Mailbox Count,
//     // Initial CCBs, and Incremental CCBs variables based on whether or not Strict
//     // Round Robin Mode is supported.  If Strict Round Robin Mode is supported,
//     // then there is no performance degradation in using the maximum possible
//     // number of Outgoing and Incoming Mailboxes and allowing the Tagged and
//     // Untagged Queue Depths to determine the actual utilization.  If Strict Round
//     // Robin Mode is not supported, then the Host Adapter must scan all the
//     // Outgoing Mailboxes whenever an Outgoing Mailbox entry is made, which can
//     // cause a substantial performance penalty.  The host adapters actually have
//     // room to store the following number of CCBs internally; that is, they can
//     // internally queue and manage this many active commands on the SCSI bus
//     // simultaneously.  Performance measurements demonstrate that the Driver Queue
//     // Depth should be set to the Mailbox Count, rather than the Host Adapter
//     // Queue Depth (internal CCB capacity), as it is more efficient to have the
//     // queued commands waiting in Outgoing Mailboxes if necessary than to block
//     // the process in the higher levels of the SCSI Subsystem.
//     //
//     // 192    BT-948/958/958D
//     if (HostAdapter->FirmwareVersion[0] == '5')
//     {
//         HostAdapter->HostAdapterQueueDepth = 192;
//     }
//
//     if (strcmp((char*)HostAdapter->FirmwareVersion, "3.31") >= 0)
//     {
//       HostAdapter->StrictRoundRobinModeSupport = TRUE;
//       HostAdapter->MailboxCount = BusLogic_MaxMailboxes;
//     }
//
//     //
//     // Tagged Queuing support is available and operates properly on all "W" series
//     // MultiMaster Host Adapters, on "C" series MultiMaster Host Adapters with
//     // firmware version 4.22 and above, and on "S" series MultiMaster Host
//     // Adapters with firmware version 3.35 and above.
//     HostAdapter->TaggedQueuingPermitted = 0xFFFF;
//
//     //
//     // Determine the Host Adapter BIOS Address if the BIOS is enabled and
//     ///save it in the Host Adapter structure.  The BIOS is disabled if the
//     // BIOS_Address is 0.
//     HostAdapter->BIOS_Address = ExtendedSetupInformation.BIOS_Address << 12;
//
//     //
//     //Initialize parameters for MultiMaster Host Adapters.
//
//     //
//     // Initialize the Host Adapter Full Model Name from the Model Name.
//     strcpy((char*)HostAdapter->FullModelName, "BusLogic ");
//     strcat((char*)HostAdapter->FullModelName, (char*)HostAdapter->ModelName);
//
//     // Tagged Queuing is only allowed if Disconnect/Reconnect is permitted.
//     // Therefore, mask the Tagged Queuing Permitted Default bits with the
//     // Disconnect/Reconnect Permitted bits.
//     HostAdapter->TaggedQueuingPermitted &= HostAdapter->DisconnectPermitted;
//
//     // Select an appropriate value for Bus Settle Time either from a BusLogic
//     // Driver Options specification, or from BusLogic_DefaultBusSettleTime.
//     HostAdapter->BusSettleTime = BusLogic_DefaultBusSettleTime;
//
//     // Indicate reading the Host Adapter Configuration completed successfully.
//     return TRUE;
// }// end BusLogic_ReadHostAdapterConfiguration
//
// BOOLEAN
// BusLogic_InitializeHostAdapter(PHW_DEVICE_EXTENSION deviceExtension,
//                                PPORT_CONFIGURATION_INFORMATION ConfigInfo)
// //_____________________________________________________________________________________________
// // Routine Description:
// //                  BusLogic_InitializeHostAdapter initializes Host Adapter.  This is the only
// //                  function called during SCSI Host Adapter detection which modifies the state
// //                  of the Host Adapter from its initial power on or hard reset state.
// // Arguments:
// //              1. device extension
// //              2. port config information
// // Return Value:
// //              TRUE : Host Adapter initialization completed successfully
// //              FALSE : Initialization failed
// //______________________________________________________________________________________________
// {
//   BusLogic_HostAdapter_T *HostAdapter = &(deviceExtension->hcs);
//   BusLogic_ExtendedMailboxRequest_T ExtendedMailboxRequest;
//   UCHAR RoundRobinModeRequest;
//   UCHAR SetCCBFormatRequest;
//   int TargetID, LunID;
//
//   // Used when we get the Physical address of the mail boxes
//   ULONG length;
//
//   // Initialize the pointers to the first and last CCBs that are queued for
//   // completion processing.
//   HostAdapter->FirstCompletedCCB = NULL;
//   HostAdapter->LastCompletedCCB = NULL;
//
//   //  Initialize the Bus Device Reset Pending CCB, Tagged Queuing Active,
//   //  Command Successful Flag, Active Commands, and Commands Since Reset
//   //  for each Target Device.
//   for (TargetID = 0; TargetID < HostAdapter->MaxTargetDevices; TargetID++)
//   {
//       HostAdapter->BusDeviceResetPendingCCB[TargetID] = NULL;
//       HostAdapter->TargetFlags[TargetID].TaggedQueuingActive = FALSE;
//       HostAdapter->TargetFlags[TargetID].CommandSuccessfulFlag = FALSE;
//
//       HostAdapter->ActiveCommandsPerTarget[TargetID] = 0;
//       for (LunID = 0; LunID < HostAdapter->MaxLogicalUnits; LunID++)
//         HostAdapter->ActiveCommandsPerLun[TargetID][LunID] = 0;
//
//       HostAdapter->CommandsSinceReset[TargetID] = 0;
//   }
//
//   // Convert virtual to physical mailbox address.
//   deviceExtension->NoncachedExtension->MailboxPA =   ScsiPortConvertPhysicalAddressToUlong(
//                                                                         ScsiPortGetPhysicalAddress(deviceExtension,
//                                                                                                    NULL,
//                                                                                                    deviceExtension->NoncachedExtension->MailboxOut,
//                                                                                                    &length));
//
//   HostAdapter->FirstOutgoingMailbox = (BusLogic_OutgoingMailbox_T *) deviceExtension->NoncachedExtension->MailboxOut;
//   HostAdapter->LastOutgoingMailbox =  HostAdapter->FirstOutgoingMailbox + HostAdapter->MailboxCount - 1;
//   HostAdapter->NextOutgoingMailbox =  HostAdapter->FirstOutgoingMailbox;
//
//   HostAdapter->FirstIncomingMailbox = (BusLogic_IncomingMailbox_T *) deviceExtension->NoncachedExtension->MailboxIn;
//   HostAdapter->LastIncomingMailbox =  HostAdapter->FirstIncomingMailbox + HostAdapter->MailboxCount - 1;
//   HostAdapter->NextIncomingMailbox =  HostAdapter->FirstIncomingMailbox;
//
//   //  Initialize the Outgoing and Incoming Mailbox structures.
//   memset(HostAdapter->FirstOutgoingMailbox,
//          0,
//          HostAdapter->MailboxCount * sizeof(BusLogic_OutgoingMailbox_T));
//   memset(HostAdapter->FirstIncomingMailbox,
//          0,
//          HostAdapter->MailboxCount * sizeof(BusLogic_IncomingMailbox_T));
//
//
//   ExtendedMailboxRequest.MailboxCount = HostAdapter->MailboxCount;
//   ExtendedMailboxRequest.BaseMailboxAddress = deviceExtension->NoncachedExtension->MailboxPA ;
//
//   if (BusLogic_Command(HostAdapter,
//                        BusLogic_InitializeExtendedMailbox,
//                        &ExtendedMailboxRequest,
//                        sizeof(ExtendedMailboxRequest),
//                        NULL, 0)
//        < 0)
//   {
//       ScsiDebugPrint((ERROR, "\n BusLogic - Failure: MAILBOX INITIALIZATION\n"));
//       return FALSE;
//   }
//
//   //  Enable Strict Round Robin Mode if supported by the Host Adapter.  In
//   //  Strict Round Robin Mode, the Host Adapter only looks at the next Outgoing
//   //  Mailbox for each new command, rather than scanning through all the
//   //  Outgoing Mailboxes to find any that have new commands in them.  Strict
//   //  Round Robin Mode is significantly more efficient.
//   if (HostAdapter->StrictRoundRobinModeSupport)
//   {
//         RoundRobinModeRequest = BusLogic_StrictRoundRobinMode;
//         if (BusLogic_Command(HostAdapter,
//                              BusLogic_EnableStrictRoundRobinMode,
//                              &RoundRobinModeRequest,
//                              sizeof(RoundRobinModeRequest),
//                              NULL,
//                              0)
//             < 0)
//         {
//             ScsiDebugPrint((ERROR, "\n BusLogic - Failure: ENABLE STRICT ROUND ROBIN MODE\n"));
//             return FALSE;
//         }
//    }
//
//   //  For Host Adapters that support Extended LUN Format CCBs, issue the Set CCB
//   //  Format command to allow 32 Logical Units per Target Device.
//
//   if (HostAdapter->ExtendedLUNSupport)
//   {
//       SetCCBFormatRequest = BusLogic_ExtendedLUNFormatCCB;
//       if (BusLogic_Command(HostAdapter,
//                            BusLogic_SetCCBFormat,
//                            &SetCCBFormatRequest,
//                            sizeof(SetCCBFormatRequest),
//                            NULL,
//                            0)
//           < 0)
//       {
//           ScsiDebugPrint((ERROR, "\n BusLogic - Failure: SET CCB FORMAT\n"));
//           return FALSE;
//       }
//     }
//
//   //  Announce Successful Initialization.
//   if (!HostAdapter->HostAdapterInitialized)
//   {
//       ScsiDebugPrint((INFO, "\n BusLogic - %s Initialized Successfully\n",
//                   HostAdapter, HostAdapter->FullModelName));
//   }
//   else
//   {
//       ScsiDebugPrint((WARNING, "\n BusLogic - %s not initialized Successfully\n",
//                   HostAdapter, HostAdapter->FullModelName));
//   }
//   HostAdapter->HostAdapterInitialized = TRUE;
//   //  Indicate the Host Adapter Initialization completed successfully.
//   return TRUE;
// }// end BusLogic_InitializeHostAdapter
//
//
// BOOLEAN
// BusLogic_TargetDeviceInquiry(BusLogic_HostAdapter_T *HostAdapter)
// //_________________________________________________________________________________________
// // Routine Description:
// //                  BusLogic_TargetDeviceInquiry inquires about the Target Devices accessible
// //                  through Host Adapter.
// // Arguments:
// //              1. Host Adapter structure
// //              2.
// // Return Value:
// //              TRUE : Inquiry successful
// //              FALSE : Inquiry failed
// //_________________________________________________________________________
// {
//   BusLogic_InstalledDevices_T InstalledDevices;
// //  BusLogic_InstalledDevices8_T InstalledDevicesID0to7;
//   BusLogic_SetupInformation_T SetupInformation;
//   BusLogic_SynchronousPeriod_T SynchronousPeriod;
//   BusLogic_RequestedReplyLength_T RequestedReplyLength;
//   int TargetID;
//
//   //  Wait a few seconds between the Host Adapter Hard Reset which initiates
//   //  a SCSI Bus Reset and issuing any SCSI Commands.  Some SCSI devices get
//   //  confused if they receive SCSI Commands too soon after a SCSI Bus Reset.
//   ScsiPortStallExecution(HostAdapter->BusSettleTime);
//
//   //
//   //  Issue the Inquire Target Devices command for host adapters with firmware
//   //  version 4.25 or later, or the Inquire Installed Devices ID 0 to 7 command
//   //  for older host adapters.  This is necessary to force Synchronous Transfer
//   //  Negotiation so that the Inquire Setup Information and Inquire Synchronous
//   //  Period commands will return valid data.  The Inquire Target Devices command
//   //  is preferable to Inquire Installed Devices ID 0 to 7 since it only probes
//   //  Logical Unit 0 of each Target Device.
//   if (strcmp((char*)HostAdapter->FirmwareVersion, "4.25") >= 0)
//   {
//       if (BusLogic_Command(HostAdapter,
//                            BusLogic_InquireTargetDevices,
//                            NULL,
//                            0,
//                            &InstalledDevices,
//                            sizeof(InstalledDevices))
//             != sizeof(InstalledDevices))
//       {
//           ScsiDebugPrint((ERROR, "\n BusLogic - Failure: INQUIRE TARGET DEVICES\n"));
//           return FALSE;
//       }
//       for (TargetID = 0; TargetID < HostAdapter->MaxTargetDevices; TargetID++)
//       {
//             HostAdapter->TargetFlags[TargetID].TargetExists = (InstalledDevices & (1 << TargetID) ? TRUE : FALSE);
//       }
//   }
//
//   //  Issue the Inquire Setup Information command.
//   RequestedReplyLength = sizeof(SetupInformation);
//   if (BusLogic_Command(HostAdapter,
//                        BusLogic_InquireSetupInformation,
//                        &RequestedReplyLength,
//                        sizeof(RequestedReplyLength),
//                &SetupInformation, sizeof(SetupInformation))
//       != sizeof(SetupInformation))
//   {
//       ScsiDebugPrint((ERROR, "\n BusLogic - Failure: INQUIRE SETUP INFORMATION\n"));
//       return FALSE;
//   }
//
//   for (TargetID = 0; TargetID < HostAdapter->MaxTargetDevices; TargetID++)
//   {
//       HostAdapter->SynchronousOffset[TargetID] = (TargetID < 8
//                                                   ? SetupInformation.SynchronousValuesID0to7[TargetID].Offset
//                                                   : SetupInformation.SynchronousValuesID8to15[TargetID-8].Offset);
//   }
//   if (strcmp((char*)HostAdapter->FirmwareVersion, "5.06L") >= 0)
//   {
//     for (TargetID = 0; TargetID < HostAdapter->MaxTargetDevices; TargetID++)
//     {
//       HostAdapter->TargetFlags[TargetID].WideTransfersActive = (TargetID < 8
//                                                                 ? (SetupInformation.WideTransfersActiveID0to7 & (1 << TargetID)
//                                                                     ? TRUE : FALSE)
//                                                                 : (SetupInformation.WideTransfersActiveID8to15 & (1 << (TargetID-8))
//                                                                     ? TRUE : FALSE));
//     }
//   }
//
//
//   //  Issue the Inquire Synchronous Period command.
//   if (HostAdapter->FirmwareVersion[0] >= '3')
//   {
//       RequestedReplyLength = sizeof(SynchronousPeriod);
//       if (BusLogic_Command(HostAdapter,
//                            BusLogic_InquireSynchronousPeriod,
//                            &RequestedReplyLength,
//                            sizeof(RequestedReplyLength),
//                            &SynchronousPeriod,
//                            sizeof(SynchronousPeriod))
//         != sizeof(SynchronousPeriod))
//       {
//           ScsiDebugPrint((ERROR, "\n BusLogic - Failure: INQUIRE SYNCHRONOUS PERIOD\n"));
//           return FALSE;
//       }
//       for (TargetID = 0; TargetID < HostAdapter->MaxTargetDevices; TargetID++)
//       {
//         HostAdapter->SynchronousPeriod[TargetID] = SynchronousPeriod[TargetID];
//       }
//   }
//
//   //  Indicate the Target Device Inquiry completed successfully.
//   return TRUE;
// }// end BusLogic_TargetDeviceInquiry
//
// VOID BusLogic_InitializeCCB( PBuslogic_CCB_T CCB)
// {
//   CCB->Opcode = BusLogic_InitiatorCCB;
//   CCB->DataDirection = 0;
//   CCB->TagEnable = 0;
//   CCB->QueueTag = 0 ;
//   CCB->CDB_Length = 0;
//   CCB->SenseDataLength = 0;
//   CCB->DataLength = 0;
//   CCB->DataPointer = 0;
//   CCB->HostAdapterStatus = 0;
//   CCB->TargetDeviceStatus = 0;
//   CCB->TargetID = 0;
//   CCB->LogicalUnit = 0;
//   CCB->LegacyTagEnable = 0;
//   CCB->LegacyQueueTag = 0;
//
//   CCB->SenseDataPointer = 0;
//
//   // BusLogic Driver Defined Portion
//   CCB->Status = 0;
//   CCB->SerialNumber = 0;
//   CCB->Next = NULL;
//   CCB->HostAdapter = NULL;
//
//   CCB->CompletionCode = 0;
//   // Pointer to the CCB
//   CCB->SrbAddress = NULL;
//   CCB->AbortSrb = NULL;
// }
//
// BOOLEAN
// NTAPI
// BT958HwStartIO(IN PVOID HwDeviceExtension,
//                IN PSCSI_REQUEST_BLOCK Srb
//               )
// //__________________________________________________________________________________
// // Routine Description:
// //                    As soon as it receives the initial request for a
// //                    target peripheral, the OS-specific port driver calls
// //                    the HwScsiStartIo routine with an input SRB. After
// //                    this call, the HBA miniport driver owns the request
// //                     and is expected to complete it.
// // Arguments:
// //           1. DeviceExtension: Points to the miniport driver's per-HBA storage area.
// //           2. Srb: Points to the SCSI request block to be started.
// // Return Value:
// //              TRUE : HwScsiStartIo returns TRUE to acknowledge receipt of the SRB.
// //__________________________________________________________________________________
// {
//     PHW_DEVICE_EXTENSION deviceExtension = HwDeviceExtension;
// //    PNONCACHED_EXTENSION noncachedExtension =   deviceExtension->NoncachedExtension;
//     BusLogic_HostAdapter_T *HostAdapter = &(deviceExtension->hcs);
// //    BusLogic_OutgoingMailbox_T mailboxOut;
//     PSCSI_REQUEST_BLOCK AbortSRB;
//
//     PBuslogic_CCB_T ccb;
//     BusLogic_TargetFlags_T *TargetFlags = &HostAdapter->TargetFlags[Srb->TargetId];
//
//
//     ScsiDebugPrint((TRACE, "\n BusLogic - Inside Start IO routine\n"));
//
//     // Make sure that this request isn't too long for the adapter.  If so
//     // bounce it back as an invalid request
//     if (BusLogic_CDB_MaxLength < Srb->CdbLength)
//     {
//
//         ScsiDebugPrint((WARNING, "\n BusLogic - Srb->CdbLength [%d] > MaxCdbLength [%d]."
//                     " Invalid request\n", Srb->CdbLength, BusLogic_CDB_MaxLength));
//
//         Srb->SrbStatus = SRB_STATUS_INVALID_REQUEST;
//         ScsiPortNotification(RequestComplete,deviceExtension,Srb);
//         ScsiPortNotification(NextRequest,deviceExtension,NULL);
//         return TRUE;
//     }
//
//     switch (Srb->Function)
//     {
//         /*
//         Sirish, 10th June 2002
//         // Check if command is a WMI request.
//         case SRB_FUNCTION_WMI:
//             // Process the WMI request and return.
//             return BT958WmiSrb(HwDeviceExtension, (PSCSI_WMI_REQUEST_BLOCK) Srb);
//         */
//
//         case SRB_FUNCTION_EXECUTE_SCSI:
//
//             // get the ccb structure from the extension
//             ccb = Srb->SrbExtension;
//             BusLogic_InitializeCCB(ccb);
//
//             // Save SRB back pointer in CCB.
//             ccb->SrbAddress = Srb;
//
//             // Build CCB. - Have some way of knowing that a srb has already been
//             // completed and you just need to return from here
//             BusLogic_QueueCommand(HwDeviceExtension ,Srb,   ccb);
//
//             // Place the CCB in an Outgoing Mailbox.  The higher levels of the SCSI
//             // Subsystem should not attempt to queue more commands than can be placed
//             // in Outgoing Mailboxes, so there should always be one free.  In the
//             // unlikely event that there are none available, wait 1 second and try
//             // again.  If that fails, the Host Adapter is probably hung so signal an
//             // error as a Host Adapter Hard Reset should be initiated soon.
//
//             if (!BusLogic_WriteOutgoingMailbox(deviceExtension , BusLogic_MailboxStartCommand, ccb))
//             {
//                 ScsiDebugPrint((ERROR, "\n BusLogic - Unable to write Outgoing Mailbox - "
//                             "Pausing for 1 second\n"));
//                 ScsiPortStallExecution(1000);
//                 if (!BusLogic_WriteOutgoingMailbox(deviceExtension , BusLogic_MailboxStartCommand, ccb))
//                 {
//                       ScsiDebugPrint((ERROR, "\n BusLogic - Still unable to write Outgoing"
//                                   "Mailbox - Host Adapter Dead?\n"));
//
//                       Srb->SrbStatus = SRB_STATUS_ERROR;
//                       ScsiPortNotification(RequestComplete,deviceExtension,Srb);
//                       ScsiPortNotification(NextRequest,deviceExtension,NULL);
//
//                 }
//             }
//             else
//             {
//                 /*
//                  * Reverted to pre 1.1.0.0 control flow
//                  * The 1.1.0.0 control flow causes the .NET Server freeze during installs/restarts
//                  * And the fix for that in 1.1.0.1 causes BSODs in XP
//                  * Side note: Ben: the Buslogic emulation never exports more than 1 lun
//                  */
//                 if (TargetFlags->TaggedQueuingActive
//                     && HostAdapter->ActiveCommandsPerLun[Srb->TargetId][Srb->Lun] < BUSLOGIC_MAXIMUM_TAGS)  {
//                     ScsiPortNotification(NextLuRequest, HwDeviceExtension, Srb->PathId, Srb->TargetId, Srb->Lun);
//                 }
//                 else    {
//                     ScsiPortNotification(NextRequest,deviceExtension,NULL);
//                 }
//             }
//             return TRUE;
//
//         case SRB_FUNCTION_ABORT_COMMAND:
//
//             // Get CCB to abort.
//             ccb = Srb->NextSrb->SrbExtension;
//
//             // Set abort SRB for completion.
//             ccb->AbortSrb = Srb;
//
//             AbortSRB = ScsiPortGetSrb(HwDeviceExtension,
//                                       Srb->PathId,
//                                       Srb->TargetId,
//                                       Srb->Lun,
//                                       Srb->QueueTag);
//
//             if ((AbortSRB != Srb->NextSrb) || (AbortSRB->SrbStatus != SRB_STATUS_PENDING))
//             {
//                 Srb->SrbStatus = SRB_STATUS_ABORT_FAILED;
//                 ScsiPortNotification(RequestComplete, HwDeviceExtension, Srb);
//                 ScsiPortNotification(NextRequest, HwDeviceExtension, NULL);
//                 return TRUE;
//             }
//
//             // write the abort information into a mailbox
//             if (BusLogic_WriteOutgoingMailbox( deviceExtension, BusLogic_MailboxAbortCommand, ccb))
//             {
//               ScsiDebugPrint((WARNING, "\n BusLogic - Aborting CCB #%ld to Target %d\n",
//                           ccb->SerialNumber, Srb->TargetId));
//               // Adapter ready for next request.
//
//                 /*
//                  * Reverted to pre 1.1.0.0 control flow
//                  * The 1.1.0.0 control flow causes the .NET Server freeze during installs/restarts
//                  * And the fix for that in 1.1.0.1 causes BSODs in XP
//                  * Side note: Ben: the Buslogic emulation never exports more than 1 lun
//                  */
//                 if (TargetFlags->TaggedQueuingActive
//                     && HostAdapter->ActiveCommandsPerLun[Srb->TargetId][Srb->Lun] < BUSLOGIC_MAXIMUM_TAGS)  {
//                     ScsiPortNotification(NextLuRequest, HwDeviceExtension, Srb->PathId, Srb->TargetId, Srb->Lun);
//                 }
//                 else    {
//                     ScsiPortNotification(NextRequest,deviceExtension,NULL);
//                 }
//             }
//             else
//             {
//               ScsiDebugPrint((WARNING, "\n BusLogic - Unable to Abort CCB #%ld to Target %d"
//                           " - No Outgoing Mailboxes\n", ccb->SerialNumber,
//                           Srb->TargetId));
//               Srb->SrbStatus = SRB_STATUS_ERROR;
//               ScsiPortNotification(RequestComplete, HwDeviceExtension, Srb);
//               ScsiPortNotification(NextRequest, HwDeviceExtension, NULL);
//             }
//             return TRUE;
//
//
//         case SRB_FUNCTION_RESET_BUS:
//
//             // Reset SCSI bus.
//             ScsiDebugPrint((INFO, "\n BusLogic - SRB_FUNCTION_RESET_BUS, srb=%x \n",Srb));
//             BT958HwResetBus(HwDeviceExtension, Srb->PathId);
//
//             Srb->SrbStatus = SRB_STATUS_SUCCESS;
//             ScsiPortNotification(RequestComplete, HwDeviceExtension, Srb);
//             ScsiPortNotification(NextRequest, HwDeviceExtension, NULL);
//             return TRUE;
//
//
//         case SRB_FUNCTION_RESET_DEVICE:
//
//             ccb = Srb->SrbExtension;
//             BusLogic_InitializeCCB(ccb);
//
//             // Save SRB back pointer in CCB.
//             ccb->SrbAddress = Srb;
//
//             if(!BusLogic_SendBusDeviceReset(HostAdapter, Srb))
//             {
//                 BT958HwResetBus(HwDeviceExtension, SP_UNTAGGED);
//                 Srb->SrbStatus = SRB_STATUS_SUCCESS;
//                 ScsiPortNotification(RequestComplete, HwDeviceExtension, Srb);
//                 ScsiPortNotification(NextRequest, HwDeviceExtension, NULL);
//                 return TRUE;
//             }
//
//             /*
//              * Reverted to pre 1.1.0.0 control flow
//              * The 1.1.0.0 control flow causes the .NET Server freeze during installs/restarts
//              * And the fix for that in 1.1.0.1 causes BSODs in XP
//              * Side note: Ben: the Buslogic emulation never exports more than 1 lun
//              */
//             if (TargetFlags->TaggedQueuingActive
//                 && HostAdapter->ActiveCommandsPerLun[Srb->TargetId][Srb->Lun] < BUSLOGIC_MAXIMUM_TAGS)  {
//                 ScsiPortNotification(NextLuRequest, HwDeviceExtension, Srb->PathId, Srb->TargetId, Srb->Lun);
//             }
//             else    {
//                 ScsiPortNotification(NextRequest,deviceExtension,NULL);
//             }
//             return TRUE;
//
//         default:
//
//             // Set error, complete request and signal ready for next request.
//             Srb->SrbStatus = SRB_STATUS_INVALID_REQUEST;
//             ScsiPortNotification(RequestComplete,deviceExtension,Srb);
//             ScsiPortNotification(NextRequest,deviceExtension,NULL);
//
//             return TRUE;
//
//     } // end switch
//
// }// end BusLogic_TargetDeviceInquiry
//
//
// BOOLEAN
// BusLogic_WriteOutgoingMailbox(PHW_DEVICE_EXTENSION deviceExtension ,
//                               BusLogic_ActionCode_T ActionCode,
//                               BusLogic_CCB_T *CCB)
// //______________________________________________________________________________________________
// // Routine Description:
// //                     BusLogic_WriteOutgoingMailbox places CCB and Action Code into an Outgoing
// //                     Mailbox for execution by Host Adapter.  The Host Adapter's Lock should
// //                     already have been acquired by the caller.
//
// // Arguments:
// //              1. deviceExtension: device extension
// //              2. ActionCode : action code for the mailbox which can be
// //                  BusLogic_OutgoingMailboxFree =      0x00,
// //                  BusLogic_MailboxStartCommand =      0x01,
// //                  BusLogic_MailboxAbortCommand =      0x02
// //              3. CCB :The CCB that has to be written into the mailbox
// // Return Value:
// //              TRUE : write to the mailbox was successful
// //              FALSE : write failed
// //______________________________________________________________________________________________
// {
//   BusLogic_HostAdapter_T *HostAdapter = &(deviceExtension->hcs);
//   BusLogic_OutgoingMailbox_T *NextOutgoingMailbox;
//   ULONG length;
//
//   NextOutgoingMailbox = HostAdapter->NextOutgoingMailbox;
//
//   if (NextOutgoingMailbox->ActionCode == BusLogic_OutgoingMailboxFree)
//   {
//       CCB->Status = BusLogic_CCB_Active;
//
//       // The CCB field must be written before the Action Code field since
//       // the Host Adapter is operating asynchronously and the locking code
//       // does not protect against simultaneous access by the Host Adapter.
//
//       // Get CCB physical address.
//       NextOutgoingMailbox->CCB = ScsiPortConvertPhysicalAddressToUlong( ScsiPortGetPhysicalAddress(deviceExtension,
//                                                                                     NULL,
//                                                                                     CCB,
//                                                                                     &length));
//
//       NextOutgoingMailbox->ActionCode = (UCHAR)ActionCode;
//
//       BusLogic_StartMailboxCommand(HostAdapter);
//
//       if (++NextOutgoingMailbox > HostAdapter->LastOutgoingMailbox)
//       {
//         NextOutgoingMailbox = HostAdapter->FirstOutgoingMailbox;
//       }
//
//       HostAdapter->NextOutgoingMailbox = NextOutgoingMailbox;
//       if (ActionCode == BusLogic_MailboxStartCommand)
//       {
//         HostAdapter->ActiveCommandsPerTarget[CCB->TargetID]++;
//         // check this Namita
//         HostAdapter->ActiveCommandsPerLun[CCB->TargetID][CCB->LogicalUnit]++;
//         if (CCB->Opcode != BusLogic_BusDeviceReset)
//         {
//           HostAdapter->TargetStatistics[CCB->TargetID].CommandsAttempted++;
//         }
//       }
//       return TRUE;
//   }
//   return FALSE;
// }// end BusLogic_WriteOutgoingMailbox
//
//
// int
// BusLogic_QueueCommand(IN PVOID HwDeviceExtension ,
//                       IN PSCSI_REQUEST_BLOCK Srb,
//                       PBuslogic_CCB_T CCB)
// //_________________________________________________________________________
// // Routine Description:
// //                      BusLogic_QueueCommand creates a CCB for Command
// // Arguments:
// //              1. HwDeviceExtension: device extension
// //              2. Srb: Pointe to the SRB
// //              3. CCB: POinter to the buffer containing the space for CCB
// // Return Value:
// //              Function returns 0 if successful
// //_________________________________________________________________________
// {
//   PHW_DEVICE_EXTENSION deviceExtension = HwDeviceExtension;
//   BusLogic_HostAdapter_T *HostAdapter = &(deviceExtension->hcs);
//   BusLogic_TargetFlags_T *TargetFlags = &HostAdapter->TargetFlags[Srb->TargetId];
//   BusLogic_TargetStatistics_T *TargetStatistics = HostAdapter->TargetStatistics;
//
//   ULONG length;
//
// //  UCHAR *Cdb = Srb->Cdb;
// //  PCDB RealCdb ;
//   UCHAR CDB_Length = Srb->CdbLength;
//   UCHAR TargetID = Srb->TargetId;
//   UCHAR LogicalUnit = Srb->Lun;
// //  void *BufferPointer = Srb->DataBuffer;
//   int BufferLength = Srb->DataTransferLength;
//
//   if (Srb->DataTransferLength > 0)
//   {
//     CCB->DataLength = Srb->DataTransferLength;
//
//   // Initialize the fields in the BusLogic Command Control Block (CCB).
// #if SG_SUPPORT
//     {
//         ULONG xferLength, remainLength;
//         PVOID virtualAddress;
//         UCHAR i = 0;
//
//         virtualAddress =  Srb->DataBuffer;
//         xferLength = Srb->DataTransferLength;
//         remainLength = xferLength;
//         /* Build scatter gather list  */
//         do
//         {
//             CCB->ScatterGatherList[i].SegmentDataPointer =  (ULONG)ScsiPortConvertPhysicalAddressToUlong(
//                                                                         ScsiPortGetPhysicalAddress(HwDeviceExtension,
//                                                                                                    Srb,
//                                                                                                    virtualAddress,
//                                                                                                    &length));
//             if ( length > remainLength )
//                 length = remainLength;
//             CCB->ScatterGatherList[i].SegmentByteCount = length;
//
//             virtualAddress = (PUCHAR) virtualAddress + length;
//             if (length >= remainLength)
//                 remainLength = 0;
//             else
//                 remainLength -= length;
//             i++;
//         } while ( remainLength > 0);
//
//         // For data transfers that have less than one scatter gather element, convert
//         // CCB to one transfer without using SG element. This will clear up the data
//         // overrun/underrun problem with small transfers that reak havoc with scanners
//         // and CD-ROM's etc. This is the method employed in ASPI4DOS to avoid similar
//         // problems.
//         if (i > 1)
//         {
//             CCB->Opcode = BusLogic_InitiatorCCB_ScatterGather;
//             CCB->DataLength = i * sizeof(BusLogic_ScatterGatherSegment_T);
//             virtualAddress = (PVOID) & (CCB->ScatterGatherList);
//             CCB->DataPointer =  ScsiPortConvertPhysicalAddressToUlong(
//                                     ScsiPortGetPhysicalAddress(HwDeviceExtension,
//                                                                0,
//                                                                (PVOID)virtualAddress,
//                                                                &length));
//         }
//         else /* Turn off SG */
//         {
//             CCB->Opcode = BusLogic_InitiatorCCB;
//             CCB->DataLength = CCB->ScatterGatherList[0].SegmentByteCount;
//             CCB->DataPointer = CCB->ScatterGatherList[0].SegmentDataPointer;
//
//         }
//     }
// #else
//     CCB->Opcode = BusLogic_InitiatorCCB;
//     CCB->DataLength = BufferLength;
//     CCB->DataPointer =  ScsiPortConvertPhysicalAddressToUlong(ScsiPortGetPhysicalAddress(deviceExtension,
//                                                                                        Srb,
//                                                                                        BufferPointer,
//                                                                                        &length));
// #endif
//   }
//
//   switch (Srb->SrbFlags & SRB_FLAGS_UNSPECIFIED_DIRECTION)
//   {
//     case SRB_FLAGS_NO_DATA_TRANSFER:
//         CCB->DataDirection = BusLogic_NoDataTransfer;
//         break;
//
//     case SRB_FLAGS_DATA_IN:
//         CCB->DataDirection = BusLogic_DataInLengthChecked;
//         TargetStatistics[TargetID].ReadCommands++;
//         BusLogic_IncrementByteCounter(&TargetStatistics[TargetID].TotalBytesRead,
//                                       BufferLength);
//         BusLogic_IncrementSizeBucket(TargetStatistics[TargetID].ReadCommandSizeBuckets,
//                                      BufferLength);
//         break;
//
//     case SRB_FLAGS_DATA_OUT:
//         CCB->DataDirection = BusLogic_DataOutLengthChecked;
//         TargetStatistics[TargetID].WriteCommands++;
//         BusLogic_IncrementByteCounter(&TargetStatistics[TargetID].TotalBytesWritten,
//                                       BufferLength);
//         BusLogic_IncrementSizeBucket(TargetStatistics[TargetID].WriteCommandSizeBuckets,
//                                      BufferLength);
//
//         break;
//
//     case SRB_FLAGS_UNSPECIFIED_DIRECTION:  /* let device decide direction */
//     default:
//         CCB->DataDirection = BusLogic_UncheckedDataTransfer;
//         break;
//   }
//
//
//   CCB->CDB_Length = CDB_Length;
//   CCB->SenseDataLength = Srb->SenseInfoBufferLength;
//   CCB->HostAdapterStatus = 0;
//   CCB->TargetDeviceStatus = 0;
//   CCB->TargetID = TargetID;
//   CCB->LogicalUnit = LogicalUnit;
//   CCB->TagEnable = FALSE;
//   CCB->LegacyTagEnable = FALSE;
//
//   //  BusLogic recommends that after a Reset the first couple of commands that
//   //  are sent to a Target Device be sent in a non Tagged Queue fashion so that
//   //  the Host Adapter and Target Device can establish Synchronous and Wide
//   //  Transfer before Queue Tag messages can interfere with the Synchronous and
//   //  Wide Negotiation messages.  By waiting to enable Tagged Queuing until after
//   //  the first BusLogic_MaxTaggedQueueDepth commands have been queued, it is
//   //  assured that after a Reset any pending commands are requeued before Tagged
//   //  Queuing is enabled and that the Tagged Queuing message will not occur while
//   //  the partition table is being printed.  In addition, some devices do not
//   //  properly handle the transition from non-tagged to tagged commands, so it is
//   //  necessary to wait until there are no pending commands for a target device
//   //  before queuing tagged commands.
//
//   if (HostAdapter->CommandsSinceReset[TargetID]++ >= BusLogic_MaxTaggedQueueDepth &&
//       HostAdapter->ActiveCommandsPerTarget[TargetID] == 0                         &&
//       !TargetFlags->TaggedQueuingActive                                           &&
//       TargetFlags->TaggedQueuingSupported                                         &&
//       (HostAdapter->TaggedQueuingPermitted & (1 << TargetID)))
//   {
//       TargetFlags->TaggedQueuingActive = TRUE;
//       ScsiDebugPrint((INFO, "\n BusLogic - Tagged Queuing now active for Target %d\n",
//                   TargetID));
//   }
//   if (TargetFlags->TaggedQueuingActive)
//   {
//     BusLogic_QueueTag_T QueueTag = BusLogic_SimpleQueueTag;
//
//     // When using Tagged Queuing with Simple Queue Tags, it appears that disk
//     // drive controllers do not guarantee that a queued command will not
//     // remain in a disconnected state indefinitely if commands that read or
//     // write nearer the head position continue to arrive without interruption.
//     // Therefore, for each Target Device this driver keeps track of the last
//     // time either the queue was empty or an Ordered Queue Tag was issued.  If
//     // more than 4 seconds (one fifth of the 20 second disk timeout) have
//     // elapsed since this last sequence point, this command will be issued
//     // with an Ordered Queue Tag rather than a Simple Queue Tag, which forces
//     // the Target Device to complete all previously queued commands before
//     // this command may be executed.
//     /*
//     if (HostAdapter->ActiveCommandsPerTarget[TargetID] == 0)
//         HostAdapter->LastSequencePoint[TargetID] = jiffies;
//     else if (jiffies - HostAdapter->LastSequencePoint[TargetID] > 4*HZ)
//     {
//       HostAdapter->LastSequencePoint[TargetID] = jiffies;
//       QueueTag = BusLogic_OrderedQueueTag;
//     }
//     */
//       if (HostAdapter->ExtendedLUNSupport)
//     {
//       CCB->TagEnable = TRUE;
//       CCB->QueueTag = (UCHAR)QueueTag;
//     }
//     else
//     {
//       CCB->LegacyTagEnable = TRUE;
//       CCB->LegacyQueueTag = (UCHAR)QueueTag;
//     }
//   }
//
//   ScsiPortMoveMemory(CCB->CDB, Srb->Cdb, CDB_Length);
//
//   //Fix for the XP Port driver - shuts of auto sense at times. 22nd May 2002
//   //{sirish, shobhit}@calsoftinc.com
//   if ((Srb->SrbFlags & SRB_FLAGS_DISABLE_AUTOSENSE) ||
//         (Srb->SenseInfoBufferLength <= 0)) {
//
//       //Disable auto request sense
//       CCB->SenseDataLength = BusLogic_DisableAutoReqSense;
//   } else {
//       //Enable auto request sense
//       CCB->SenseDataLength  = (unsigned char) Srb->SenseInfoBufferLength;
//
//       //Sense Buffer physical addr
//       CCB->SenseDataPointer = ScsiPortConvertPhysicalAddressToUlong(ScsiPortGetPhysicalAddress(HwDeviceExtension,
//                                                                                            Srb,
//                                                                                            Srb->SenseInfoBuffer,
//                                                                                            &length));
//       if (Srb->SenseInfoBufferLength > length ) {
//           CCB->SenseDataLength = (unsigned char) length;
//       }
//   }
//
//   return 0;
// }// end BusLogic_QueueCommand
//
//
// BOOLEAN
// NTAPI
// BT958HwInterrupt(IN PVOID HwDeviceExtension)
// //_________________________________________________________________________
// // Routine Description:
// //                      HwScsiInterrupt is called when the HBA generates an
// //                      interrupt. Miniport drivers of HBAs that do not
// //                      generate interrupts do not have this routine.
// //                      HwScsiInterrupt is responsible for completing
// //                      interrupt-driven I/O operations. This routine must
// //                      clear the interrupt on the HBA before it returns TRUE.
// // Arguments:
// //              1. HwDeviceExtension: Points to the miniport driver's per-HBA storage area.
// // Return Value:
// //              TRUE : Acknowledged the interrupts on the HBA
// //              FALSE: If the miniport finds that its HBA did not generate
// //                     the interrupt, HwScsiInterrupt should return FALSE
// //                     as soon as possible.
// //_________________________________________________________________________
// {
//     PHW_DEVICE_EXTENSION deviceExtension = HwDeviceExtension;
// //    PNONCACHED_EXTENSION noncachedExtension =    deviceExtension->NoncachedExtension;
//     BusLogic_HostAdapter_T *HostAdapter = &(deviceExtension->hcs);
// //    PBuslogic_CCB_T ccb;
// //    PSCSI_REQUEST_BLOCK srb;
//
// //    ULONG residualBytes;
// //    ULONG i;
//
//     BusLogic_InterruptRegister_T InterruptRegister;
//
//
//     // Read the Host Adapter Interrupt Register.
//     *InterruptRegister.All.as_mut() = BusLogic_ReadInterruptRegister(HostAdapter);
//     if (InterruptRegister.Bits.InterruptValid)
//     {
//
//       // Acknowledge the interrupt and reset the Host Adapter
//       // Interrupt Register.
//       BusLogic_InterruptReset(HostAdapter);
//       // Process valid External SCSI Bus Reset and Incoming Mailbox
//       // Loaded Interrupts.  Command Complete Interrupts are noted,
//       // and Outgoing Mailbox Available Interrupts are ignored, as
//       // they are never enabled.
//       if (InterruptRegister.Bits.ExternalBusReset)
//       {
//         HostAdapter->HostAdapterExternalReset = TRUE;
//       }
//       else if (InterruptRegister.Bits.IncomingMailboxLoaded)
//       {
//         BusLogic_ScanIncomingMailboxes(deviceExtension);
//       }
//       else if (InterruptRegister.Bits.CommandComplete)
//       {
//         HostAdapter->HostAdapterCommandCompleted = TRUE;
//       }
//     }
//     else
//         return FALSE;
//
//     // Process any completed CCBs.
//     if (HostAdapter->FirstCompletedCCB != NULL)
//         BusLogic_ProcessCompletedCCBs(deviceExtension);
//
//     // Reset the Host Adapter if requested.
//     if (HostAdapter->HostAdapterExternalReset ||
//         HostAdapter->HostAdapterInternalError)
//     {
//       // I have replaced the NULL with srb->pathid check if this is correct
//       BT958HwResetBus(HwDeviceExtension, SP_UNTAGGED);
//       HostAdapter->HostAdapterExternalReset = FALSE;
//       HostAdapter->HostAdapterInternalError = FALSE;
//     }
//     return TRUE;
// }// end BT958HwInterrupt
//
// void
// BusLogic_ScanIncomingMailboxes(PHW_DEVICE_EXTENSION deviceExtension)
// //________________________________________________________________________________________
// // Routine Description:
// //                  BusLogic_ScanIncomingMailboxes scans the Incoming Mailboxes saving any
// //                  Incoming Mailbox entries for completion processing.
// // Arguments:
// //              1. deviceExtension : pointer to the device extension
// //_________________________________________________________________________________________
// {
//   BusLogic_HostAdapter_T *HostAdapter = &(deviceExtension->hcs);
//
//
//   //  Scan through the Incoming Mailboxes in Strict Round Robin fashion, saving
//   //  any completed CCBs for further processing.  It is essential that for each
//   //  CCB and SCSI Command issued, command completion processing is performed
//   //  exactly once.  Therefore, only Incoming Mailboxes with completion code
//   //  Command Completed Without Error, Command Completed With Error, or Command
//   //  Aborted At Host Request are saved for completion processing.  When an
//   //  Incoming Mailbox has a completion code of Aborted Command Not Found, the
//   //  CCB had already completed or been aborted before the current Abort request
//   //  was processed, and so completion processing has already occurred and no
//   //  further action should be taken.
//   BusLogic_IncomingMailbox_T *NextIncomingMailbox =    HostAdapter->NextIncomingMailbox;
//   BusLogic_CompletionCode_T CompletionCode;
//
//   while ((CompletionCode = NextIncomingMailbox->CompletionCode) != BusLogic_IncomingMailboxFree)
//   {
//       // Convert Physical CCB to Virtual.
//       BusLogic_CCB_T *CCB = (BusLogic_CCB_T *) ScsiPortGetVirtualAddress(deviceExtension,
//                                                                          ScsiPortConvertUlongToPhysicalAddress(NextIncomingMailbox->CCB));
//
//       ScsiDebugPrint((INFO, "\n Buslogic - Virtual CCB %lx\n", CCB));
//       if (CompletionCode != BusLogic_AbortedCommandNotFound)
//       {
//         if (CCB->Status == BusLogic_CCB_Active || CCB->Status == BusLogic_CCB_Reset)
//         {
//
//             // Save the Completion Code for this CCB and queue the CCB
//             // for completion processing.
//             CCB->CompletionCode = CompletionCode;
//             BusLogic_QueueCompletedCCB(deviceExtension,CCB);
//         }
//         else
//         {
//             // If a CCB ever appears in an Incoming Mailbox and is not marked
//             // as status Active or Reset, then there is most likely a bug in
//             // the Host Adapter firmware.
//             ScsiDebugPrint((ERROR, "\n BusLogic - Illegal CCB #%ld status %d in "
//                         "Incoming Mailbox\n", CCB->SerialNumber, CCB->Status));
//         }
//       }
//       NextIncomingMailbox->CompletionCode = BusLogic_IncomingMailboxFree;
//       if (++NextIncomingMailbox > HostAdapter->LastIncomingMailbox)
//         NextIncomingMailbox = HostAdapter->FirstIncomingMailbox;
//   }
//   HostAdapter->NextIncomingMailbox = NextIncomingMailbox;
// }// end BusLogic_ScanIncomingMailboxes
//
//
// void
// BusLogic_QueueCompletedCCB(PHW_DEVICE_EXTENSION deviceExtension,
//                            BusLogic_CCB_T *CCB)
// //_________________________________________________________________________________
// // Routine Description:
// //                  BusLogic_QueueCompletedCCB queues CCB for completion processing.
// // Arguments:
// //              1. deviceExtension : pointer to device extension
// //              2. CCB: pointe t CCB that needs to be queued
// //_________________________________________________________________________________
// {
//   BusLogic_HostAdapter_T *HostAdapter = &(deviceExtension->hcs);
//
//   CCB->Status = BusLogic_CCB_Completed;
//   CCB->Next = NULL;
//   if (HostAdapter->FirstCompletedCCB == NULL)
//   {
//       HostAdapter->FirstCompletedCCB = CCB;
//       HostAdapter->LastCompletedCCB = CCB;
//   }
//   else
//   {
//       HostAdapter->LastCompletedCCB->Next = CCB;
//       HostAdapter->LastCompletedCCB = CCB;
//   }
//   HostAdapter->ActiveCommandsPerTarget[CCB->TargetID]--;
//   HostAdapter->ActiveCommandsPerLun[CCB->TargetID][CCB->LogicalUnit]--;
//
// }// end BusLogic_QueueCompletedCCB
//
//
// void
// BusLogic_ProcessCompletedCCBs(PHW_DEVICE_EXTENSION deviceExtension)
// //_________________________________________________________________________________
// // Routine Description:
// //            BusLogic_ProcessCompletedCCBs iterates over the completed CCBs for Host
// //            Adapter setting the SCSI Command Result Codes, deallocating the CCBs, and
// //            calling the SCSI Subsystem Completion Routines.  The Host Adapter's Lock
// //            should already have been acquired by the caller.
// //
// // Arguments:
// //              1. deviceExtension : pointer to device extension
// //_________________________________________________________________________________
// {
//   BusLogic_HostAdapter_T *HostAdapter = &(deviceExtension->hcs);
//   PSCSI_REQUEST_BLOCK srb;
//   PCDB RealCdb;
//
//   if (HostAdapter->ProcessCompletedCCBsActive)
//       return;
//   HostAdapter->ProcessCompletedCCBsActive = TRUE;
//   while (HostAdapter->FirstCompletedCCB != NULL)
//   {
//       BusLogic_CCB_T *CCB = HostAdapter->FirstCompletedCCB;
//
//       // Get SRB from CCB.
//       srb = CCB->SrbAddress;
//
//        HostAdapter->FirstCompletedCCB = CCB->Next;
//        if (HostAdapter->FirstCompletedCCB == NULL)
//             HostAdapter->LastCompletedCCB = NULL;
//
//       // Process the Completed CCB.
//       if (CCB->Opcode == BusLogic_BusDeviceReset)
//       {
//           int TargetID = CCB->TargetID, LunID;
//           ScsiDebugPrint((TRACE, "\n BusLogic - Bus Device Reset CCB #%ld to Target "
//                       "%d Completed\n", CCB->SerialNumber, TargetID));
//           BusLogic_IncrementErrorCounter(&HostAdapter->TargetStatistics[TargetID].BusDeviceResetsCompleted);
//
//           HostAdapter->TargetFlags[TargetID].TaggedQueuingActive = FALSE;
//           HostAdapter->CommandsSinceReset[TargetID] = 0;
//           //HostAdapter->LastResetCompleted[TargetID] = jiffies;
//           HostAdapter->ActiveCommandsPerTarget[TargetID] = 0;
//           for (LunID = 0; LunID < HostAdapter->MaxLogicalUnits; LunID++)
//             HostAdapter->ActiveCommandsPerLun[TargetID][LunID] = 0;
//
//           // Complete all outstanding requests with SRB_STATUS_BUS_RESET.
//           ScsiPortCompleteRequest(deviceExtension,
//                                   (UCHAR)srb->PathId,
//                                   srb->TargetId,
//                                   0xFF,
//                                   (ULONG)SRB_STATUS_BUS_RESET);
//
//
//           HostAdapter->BusDeviceResetPendingCCB[TargetID] = NULL;
//       }
//       else
//       {
//           // Translate the Completion Code, Host Adapter Status, and Target
//           // Device Status into a SCSI Subsystem Result Code.
//           switch (CCB->CompletionCode)
//           {
//             case BusLogic_IncomingMailboxFree:
//             case BusLogic_InvalidCCB:
//             {
//                 ScsiDebugPrint((ERROR, "\n BusLogic - CCB #%ld to Target %d Impossible "
//                             "State\n", CCB->SerialNumber, CCB->TargetID));
//                 break;
//             }
//
//             //Processing for CCB that was to be aborted
//             case BusLogic_AbortedCommandNotFound:
//             {
//                 srb = CCB->AbortSrb;
//                 srb->SrbStatus = SRB_STATUS_ABORT_FAILED;
//                 break;
//             }
//
//             case BusLogic_CommandCompletedWithoutError:
//             {
//                HostAdapter->TargetStatistics[CCB->TargetID].CommandsCompleted++;
//                HostAdapter->TargetFlags[CCB->TargetID].CommandSuccessfulFlag = TRUE;
//
//                srb->SrbStatus = SRB_STATUS_SUCCESS;
//                break;
//             }
//
//             case BusLogic_CommandAbortedAtHostRequest:
//             {
//               ScsiDebugPrint((TRACE, "\n BusLogic - CCB #%ld to Target %d Aborted\n",
//                           CCB->SerialNumber, CCB->TargetID));
//               //BusLogic_IncrementErrorCounter(&HostAdapter->TargetStatistics[CCB->TargetID].CommandAbortsCompleted);
//
//               srb->SrbStatus = SRB_STATUS_ABORTED;
//
//               // Call notification routine for the aborted SRB.
//               ScsiPortNotification(RequestComplete,
//                                    deviceExtension,
//                                    srb);
//
//               // Get the abort SRB from CCB.
//               srb = CCB->AbortSrb;
//
//                // Set status for completing abort request.
//               srb->SrbStatus = SRB_STATUS_SUCCESS;
//               break;
//             }
//
//             case BusLogic_CommandCompletedWithError:
//             {
//               RealCdb = (PCDB)CCB->CDB;
//               ScsiDebugPrint((ERROR, "\n BusLogic - %x Command completed with error Host - "
//                           "%x Target %x \n", RealCdb->CDB6GENERIC.OperationCode,
//                           CCB->HostAdapterStatus, CCB->TargetDeviceStatus));
//
//               srb->SrbStatus = BusLogic_ComputeResultCode(HostAdapter,
//                                                            CCB->HostAdapterStatus,
//                                                            CCB->TargetDeviceStatus,
//                                                            CCB->SenseDataLength);
//               if (CCB->HostAdapterStatus != BusLogic_SCSISelectionTimeout)
//               {
//                 HostAdapter->TargetStatistics[CCB->TargetID].CommandsCompleted++;
//               }
//               break;
//             }
//
//             default:
//             {
//                        // Log the error.
//                ScsiPortLogError(deviceExtension,
//                                 NULL,
//                                 0,
//                                 srb->TargetId,
//                                 0,
//                                 SP_INTERNAL_ADAPTER_ERROR,
//                                 143);
//
//                 ScsiDebugPrint((ERROR, "\n BusLogic - Unrecognized mailbox status\n"));
//             }
//
//           }// end switch
//
//
//           // When an INQUIRY command completes normally, save the CmdQue (Tagged Queuing Supported)
//           // and WBus16 (16 Bit Wide Data Transfers Supported) bits.
//           RealCdb = (PCDB) CCB->CDB;
//           if (RealCdb->CDB6INQUIRY.OperationCode == SCSIOP_INQUIRY &&
//               CCB->HostAdapterStatus == BusLogic_CommandCompletedNormally)
//           {
//               BusLogic_TargetFlags_T *TargetFlags =&HostAdapter->TargetFlags[CCB->TargetID];
//               SCSI_Inquiry_T *InquiryResult =(SCSI_Inquiry_T *) srb->DataBuffer;
//               TargetFlags->TargetExists = TRUE;
//               TargetFlags->TaggedQueuingSupported = InquiryResult->CmdQue;
//               TargetFlags->WideTransfersSupported = InquiryResult->WBus16;
//           }
//
//           ScsiDebugPrint((INFO, "\n BusLogic - SCSI Status %x\n", srb->ScsiStatus));
//           ScsiDebugPrint((INFO, "\n BusLogic - HBA Status %x\n", CCB->HostAdapterStatus));
//
//           // Update target status in SRB.
//           srb->ScsiStatus = (UCHAR)CCB->TargetDeviceStatus;
//
//           // Signal request completion.
//           ScsiPortNotification(RequestComplete, (PVOID)deviceExtension, srb);
//        }
//     }
//     HostAdapter->ProcessCompletedCCBsActive = FALSE;
// }// end BusLogic_ProcessCompletedCCBs
//
//
// UCHAR
// BusLogic_ComputeResultCode(BusLogic_HostAdapter_T *HostAdapter,
//                            BusLogic_HostAdapterStatus_T HostAdapterStatus,
//                            BusLogic_TargetDeviceStatus_T TargetDeviceStatus,
//                            UCHAR SenseDataLength)
// //_________________________________________________________________________________________
// // Routine Description:
// //                BusLogic_ComputeResultCode computes a SCSI Subsystem Result Code from
// //                the Host Adapter Status and Target Device Status.
// // Arguments:
// //              1.HostAdapter:  Pointer to the host adapter structure
// //              2.HostAdapterStatus: Host adapter status returned in the completed CCB
// //              3.TargetDeviceStatus:
// // Return Value:
// //              This function returns the error code that should be returned to port driver
// //_________________________________________________________________________________________
// {
//   UCHAR HostStatus = 0;
//
//   // Namita 2Oct CDROM issue
//   if (TargetDeviceStatus != BusLogic_OperationGood && (HostAdapterStatus == BusLogic_CommandCompletedNormally ||
//                                                        HostAdapterStatus == BusLogic_LinkedCommandCompleted   ||
//                                                        HostAdapterStatus == BusLogic_LinkedCommandCompletedWithFlag))
//   {
//     switch(TargetDeviceStatus)
//     {
//         case BusLogic_CheckCondition:
//         {
//             HostStatus = SRB_STATUS_ERROR;
//             if(SenseDataLength != BusLogic_DisableAutoReqSense)
//                 HostStatus |= SRB_STATUS_AUTOSENSE_VALID;
//             break;
//         }
//         case BusLogic_DeviceBusy:
//         {
//             HostStatus = SRB_STATUS_BUSY;
//             break;
//         }
//         case BusLogic_OperationGood:
//         {
//             HostStatus = SRB_STATUS_SUCCESS;
//             break;
//         }
// 	}
//   }
//
//   else
//   {
//
//       switch (HostAdapterStatus)
//       {
//             case BusLogic_CommandCompletedNormally:
//             case BusLogic_LinkedCommandCompleted:
//             case BusLogic_LinkedCommandCompletedWithFlag:
//             {
//               HostStatus = SRB_STATUS_SUCCESS;
//               break;
//             }
//             case BusLogic_SCSISelectionTimeout:
//             {
//               HostStatus = SRB_STATUS_SELECTION_TIMEOUT;
//               break;
//             }
//             case BusLogic_InvalidOutgoingMailboxActionCode:
//             case BusLogic_InvalidCommandOperationCode:
//             case BusLogic_InvalidCommandParameter:
//               ScsiDebugPrint((WARNING, "\n BusLogic - Driver Protocol Error 0x%02X\n",
//                           HostAdapterStatus));
//             case BusLogic_DataUnderRun:
//
//             case BusLogic_DataOverRun:
//                 // SRB_STATUS_DATA_OVERRUN
//
//             case BusLogic_LinkedCCBhasInvalidLUN:
//
//             case BusLogic_TaggedQueuingMessageRejected:
//                 // SRB_STATUS_MESSAGE_REJECTED
//             case BusLogic_TargetDeviceReconnectedImproperly:
//             case BusLogic_AbortQueueGenerated:
//             case BusLogic_HostAdapterSoftwareError:
//
//             case BusLogic_HostAdapterHardwareTimeoutError:
//                 // SRB_STATUS_TIMEOUT
//             {
//               HostStatus = SRB_STATUS_ERROR;
//               break;
//             }
//             case BusLogic_TargetFailedResponseToATN:
//             case BusLogic_HostAdapterAssertedRST:
//             case BusLogic_OtherDeviceAssertedRST:
//             case BusLogic_HostAdapterAssertedBusDeviceReset:
//             {
//               HostStatus = SRB_STATUS_BUS_RESET;
//               break;
//             }
//             case BusLogic_SCSIParityErrorDetected:
//             {
//                 HostStatus = SRB_STATUS_PARITY_ERROR;
//                 break;
//             }
//             case BusLogic_UnexpectedBusFree:
//             {
//                 HostStatus = SRB_STATUS_UNEXPECTED_BUS_FREE;
//                 break;
//             }
//             case BusLogic_InvalidBusPhaseRequested:
//             {
//                 HostStatus = SRB_STATUS_PHASE_SEQUENCE_FAILURE;
//                 break;
//             }
//             case BusLogic_AutoRequestSenseFailed:
//             {
//                 HostStatus = SRB_STATUS_REQUEST_SENSE_FAILED;
//                 break;
//             }
//             case BusLogic_UnsupportedMessageReceived:
//             {
//                 HostStatus = SRB_STATUS_INVALID_REQUEST;
//                 break;
//             }
//             case BusLogic_HostAdapterHardwareFailed:
//             {
//                 HostStatus = SRB_STATUS_NO_HBA;
//                 break;
//             }
//             default:
//             {
//               ScsiDebugPrint((WARNING, "\n BusLogic - Unknown HBA Status 0x%02X\n",
//                           HostAdapterStatus));
//               HostStatus = SRB_STATUS_ERROR;
//               break;
//             }
//         }
//   }
//   return HostStatus;
// }// end BusLogic_ComputeResultCode
//
//
// BOOLEAN
// NTAPI
// BT958HwResetBus(IN PVOID HwDeviceExtension,
//                 IN ULONG PathId)
// //_____________________________________________________________________________________
// // Routine Description:
// //                   BT958HwResetBus resets Host Adapter if possible, marking all
// //                   currently executing SCSI Commands as having been Reset.
// // Arguments:
// //           1. HwDeviceExtension: Points to the miniport driver's per-HBA storage area.
// //           2. PathId: Identifies the SCSI bus to be reset.
// // Return Value:
// //              TRUE : If the bus is successfully reset, HwScsiResetBus returns TRUE.
// //              FALSE : reset did not complete successfully
// //_____________________________________________________________________________________
// {
//   PHW_DEVICE_EXTENSION deviceExtension = HwDeviceExtension;
//   BusLogic_HostAdapter_T *HostAdapter = &(deviceExtension->hcs);
// //  BusLogic_CCB_T *CCB;
// //  int TargetID;
//   BOOLEAN Result;
//   BOOLEAN HardReset;
//
//   ScsiDebugPrint((TRACE, "\n BusLogic - Reset SCSI bus\n"));
//
//   // Complete all outstanding requests with SRB_STATUS_BUS_RESET.
//   ScsiPortCompleteRequest(deviceExtension,
//                                 (UCHAR)PathId,
//                                 0xFF,
//                                 0xFF,
//                                 (ULONG) SRB_STATUS_BUS_RESET);
//
//   if (HostAdapter->HostAdapterExternalReset)
//   {
//       //BusLogic_IncrementErrorCounter(&HostAdapter->ExternalHostAdapterResets);
//       HardReset = FALSE;
//   }
//   else if (HostAdapter->HostAdapterInternalError)
//   {
//       //BusLogic_IncrementErrorCounter(&HostAdapter->HostAdapterInternalErrors);
//       HardReset = TRUE;
//   }
//   else
//   {
//       //BusLogic_IncrementErrorCounter(&HostAdapter->TargetStatistics[srb->TargetId].HostAdapterResetsRequested);
//       HardReset = TRUE;
//   }
//
//   // Think of a way of doing this - Namita
//   /*
//   if(srb == NULL)
//   {
//     if (HostAdapter->HostAdapterInternalError)
//         ScsiDebugPrint((0,"BusLogic Warning: Resetting %s due to Host Adapter Internal Error\n",HostAdapter->FullModelName));
//     else
//         ScsiDebugPrint((0,"BUsLogic Warning: Resetting %s due to External SCSI Bus Reset\n",HostAdapter->FullModelName));
//   }
//   else
//   {
//       ScsiDebugPrint((0,"BusLogic Warning: Resetting %s due to Target %d\n", HostAdapter->FullModelName, srb->TargetId));
//       //BusLogic_IncrementErrorCounter(&HostAdapter->TargetStatistics[srb->TargetId].HostAdapterResetsAttempted);
//   }
//   */
//
//   // Attempt to Reset and Reinitialize the Host Adapter.
//   // Change the initialize routine to make allocation some place else
//   if (!(BusLogic_HardwareResetHostAdapter(HostAdapter, HardReset) && BusLogic_InitializeHostAdapter(deviceExtension, NULL)))
//   {
//       ScsiDebugPrint((ERROR, "\n Buslogic - Resetting %s Failed\n",
//                   HostAdapter->FullModelName));
//       Result = FALSE;
//       goto Done;
//   }
//
//   // Check if we have to do this, document says that the scsi port driver takes care of the reset delays - Namita
//   // Wait a few seconds between the Host Adapter Hard Reset which initiates
//   // a SCSI Bus Reset and issuing any SCSI Commands.  Some SCSI devices get
//   // confused if they receive SCSI Commands too soon after a SCSI Bus Reset.
//   // Note that a timer interrupt may occur here, but all active CCBs have
//   // already been marked Reset and so a reentrant call will return Pending.
//   if (HardReset)
//     ScsiPortStallExecution(HostAdapter->BusSettleTime);
//   Result = TRUE;
//
// Done:
//   return Result;
// }// end BT958HwResetBus
//
// BOOLEAN
// BusLogic_SendBusDeviceReset(IN PVOID HwDeviceExtension,
//                             PSCSI_REQUEST_BLOCK Srb)
// //_____________________________________________________________________________________
// // Routine Description:
// //                     BusLogic_SendBusDeviceReset causes a BUS DEVICE reset command to
// //                     be send to particular target.
// // Arguments:
// //           1. HwDeviceExtension: Points to the miniport driver's per-HBA storage area.
// //           2. Srb: pointer to the SCSI request block
// // Return Value:
// //              TRUE : If the bus is successfully reset, HwScsiResetBus returns TRUE.
// //              FALSE : reset did not complete successfully
// //_____________________________________________________________________________________
// {
//
//   PHW_DEVICE_EXTENSION deviceExtension = HwDeviceExtension;
//   BusLogic_HostAdapter_T *HostAdapter = &(deviceExtension->hcs);
//
//   UCHAR TargetID = Srb->TargetId;
//   PBuslogic_CCB_T CCB = Srb->SrbExtension;
//   BOOLEAN Result = FALSE;
//   BusLogic_IncrementErrorCounter(&HostAdapter->TargetStatistics[TargetID].BusDeviceResetsRequested);
//
//   //  If this is a Synchronous Reset and a Bus Device Reset is already pending
//   //  for this Target Device, do not send a second one.  Add this Command to
//   //  the list of Commands for which completion processing must be performed
//   //  when the Bus Device Reset CCB completes.
//
//   if (HostAdapter->BusDeviceResetPendingCCB[TargetID] != NULL)
//   {
//     ScsiDebugPrint((WARNING, "\n BusLogic - Unable to Reset Command to Target %d - "
//                 "Reset Pending\n", TargetID));
//     Result = TRUE;
//     goto Done;
//   }
//
//   ScsiDebugPrint((WARNING, "\n BusLogic - Sending Bus Device Reset CCB #%ld to Target %d\n",
//               CCB->SerialNumber, TargetID));
//   CCB->Opcode = BusLogic_BusDeviceReset;
//   CCB->TargetID = TargetID;
//
//
//   //    Attempt to write an Outgoing Mailbox with the Bus Device Reset CCB.
//   //    If sending a Bus Device Reset is impossible, attempt a full Host
//   //    Adapter Hard Reset and SCSI Bus Reset.
//   if (!(BusLogic_WriteOutgoingMailbox(deviceExtension, BusLogic_MailboxStartCommand, CCB)))
//   {
//       ScsiDebugPrint((WARNING, "\n BusLogic - Unable to write Outgoing Mailbox for "
//                   "Bus Device Reset\n"));
//       goto Done;
//   }
//
//   //  If there is a currently executing CCB in the Host Adapter for this Command
//   //  (i.e. this is an Asynchronous Reset), then an Incoming Mailbox entry may be
//   //  made with a completion code of BusLogic_HostAdapterAssertedBusDeviceReset.
//   //  If there is no active CCB for this Command (i.e. this is a Synchronous
//   //  Reset), then the Bus Device Reset CCB's Command field will have been set
//   //  to the Command so that the interrupt for the completion of the Bus Device
//   //  Reset can call the Completion Routine for the Command.  On successful
//   //  execution of a Bus Device Reset, older firmware versions did return the
//   //  pending CCBs with the appropriate completion code, but more recent firmware
//   //  versions only return the Bus Device Reset CCB itself.  This driver handles
//   //  both cases by marking all the currently executing CCBs to this Target
//   //  Device as Reset.  When the Bus Device Reset CCB is processed by the
//   //  interrupt handler, any remaining CCBs marked as Reset will have completion
//   //  processing performed.
//
//   BusLogic_IncrementErrorCounter( &HostAdapter->TargetStatistics[TargetID].BusDeviceResetsAttempted);
//   HostAdapter->BusDeviceResetPendingCCB[TargetID] = CCB;
//   //HostAdapter->LastResetAttempted[TargetID] = jiffies;
//
//   // FlashPoint Host Adapters may have already completed the Bus Device
//   // Reset and BusLogic_QueueCompletedCCB been called, or it may still be
//   // pending.
//
//   Result = TRUE;
//   //  If a Bus Device Reset was not possible for some reason, force a full
//   //  Host Adapter Hard Reset and SCSI Bus Reset.
//
// Done:
//   return Result;
// }
//
// BOOLEAN
// NTAPI
// BT958HwInitialize(IN PVOID HwDeviceExtension)
// //_______________________________________________________________________________
// // Routine Description:
// //              This routine initializes the adapter by enabling its interrupts
// // Arguments:
// //              1. device extension
// // Return Value:
// //              TRUE : initialization successful
// //              FALSE : initialization failed
// //_______________________________________________________________________________
// {
//     PHW_DEVICE_EXTENSION deviceExtension = HwDeviceExtension;
//     BusLogic_HostAdapter_T *HostAdapter = &(deviceExtension->hcs);
//     CHAR Parameter = 1;
//
//     // enable interrupts
//     if (BusLogic_Command(HostAdapter,
//                            BusLogic_DisableHostAdapterInterrupt,
//                            &Parameter,
//                            sizeof(Parameter),
//                            NULL,
//                            0)
//         < 0)
//     {
//        return FALSE;
//     }
//     return TRUE;
// }// end BT958HwInitialize
//
// SCSI_ADAPTER_CONTROL_STATUS
// NTAPI
// BT958HwAdapterControl(IN PVOID HwDeviceExtension,
//                       IN SCSI_ADAPTER_CONTROL_TYPE ControlType,
//                       IN PVOID Parameters)
// //__________________________________________________________________________________________
// // Routine Description:
// //               A miniport driver's HwScsiAdapterControl routine is called to perform
// //               synchronous operations to control the state or behavior of an HBA, such as
// //               stopping or restarting the HBA for power management.
// // Arguments:
// //              1. HwDeviceExtension: device extension
// //              2. ControlType: Specifies one of the following adapter-control operations.
// //                 ScsiQuerySupportedControlTypes : Reports the adapter-control operations
// //                                                  implemented by the miniport.
// //                 ScsiStopAdapter:  Shuts down the HBA.
// //                 ScsiRestartAdapter: Reinitializes an HBA.
// //                 ScsiSetBootConfig: Not supported
// //                 ScsiSetRunningConfig: Not supported
// //              3. Parameters:
// // Return Value:
// //              TRUE :
// //              FALSE :
// //_________________________________________________________________________
// {
//     PHW_DEVICE_EXTENSION deviceExtension = HwDeviceExtension;
//     BusLogic_HostAdapter_T *HostAdapter = &(deviceExtension->hcs);
// //	UCHAR *ParameterPointer;
// //	BusLogic_StatusRegister_T StatusRegister;
// //	BusLogic_InterruptRegister_T InterruptRegister;
// //	int Result,ParameterLength;
// //	long TimeoutCounter;
//
//     PSCSI_SUPPORTED_CONTROL_TYPE_LIST ControlTypeList;
//     ULONG AdjustedMaxControlType;
//
//     ULONG Index;
// //    UCHAR Retries;
//
//     // Default Status
// //    SCSI_ADAPTER_CONTROL_STATUS Status = ScsiAdapterControlSuccess;
//
//     //
//     // Structure defining which functions this miniport supports
//     //
//     BOOLEAN SupportedConrolTypes[ScsiAdapterControlMax] =
//     {
//         TRUE,   // ScsiQuerySupportedControlTypes
//         TRUE,   // ScsiStopAdapter
//         TRUE,   // ScsiRestartAdapter
//         FALSE,  // ScsiSetBootConfig
//         FALSE   // ScsiSetRunningConfig
//     };
//
//     ScsiDebugPrint((TRACE, "\n BusLogic -  Inside HwAdapterControl function \n"));
//     switch(ControlType)
//     {
//         case ScsiQuerySupportedControlTypes:
//         //  Reports the adapter-control operations implemented by the miniport. The port
//         //  driver calls HwScsiAdapterControl with this control type after the HBA has been
//         //  initialized but before the first I/O. The miniport fills in the
//         //  SCSI_SUPPORTED_CONTROL_TYPE_LIST structure at Parameters with the operations it
//         //  supports. After HwScsiAdapterControl returns from this call, the port driver
//         //  calls the miniport's HwScsiAdapterControl only for supported operations.
//         {
//
//             // This entry point provides the method by which SCSIPort determines the
//             // supported ControlTypes. Parameters is a pointer to a
//             // SCSI_SUPPORTED_CONTROL_TYPE_LIST structure. Fill in this structure
//             // honoring the size limits.
//             ControlTypeList = Parameters;
//             AdjustedMaxControlType =  (ControlTypeList->MaxControlType < ScsiAdapterControlMax) ?  ControlTypeList->MaxControlType :ScsiAdapterControlMax;
//             for (Index = 0; Index < AdjustedMaxControlType; Index++)
//             {
//                 ControlTypeList->SupportedTypeList[Index] = SupportedConrolTypes[Index];
//             }
//             break;
//         }
//
//         case ScsiStopAdapter:
//         // Shuts down the HBA. The port driver calls HwScsiAdapterControl with this control
//         // type when the HBA has been removed from the system, stopped for resource reconfiguration,
//         // shut down for power management, or otherwise reconfigured or disabled. The port driver
//         // ensures that there are no uncompleted requests and issues an SRB_FUNCTION_FLUSH request
//         // to the miniport before calling this routine. The miniport disables interrupts on its HBA,
//         // halts all processing, (including background processing not subject to interrupts or processing
//         // of which the port driver is unaware, such as reconstructing fault-tolerant volumes), flushes
//         // any remaining cached data to persistent storage, and puts the HBA into a state from which it
//         // can be reinitialized or restarted.
//         // The miniport should not free its resources when stopping its HBA. If the HBA was removed or
//         // stopped for PnP resource reconfiguration, the port driver releases resources on behalf of the
//         // miniport driver. If the HBA is shut down for power management, the miniport's resources are
//         // preserved so the HBA can be restarted.
//         {
//             CHAR Parameter = 0;
//             ScsiDebugPrint((INFO, "\n BusLogic -  stopping the device \n"));
//             if (BusLogic_Command(HostAdapter,
//                                    BusLogic_DisableHostAdapterInterrupt,
//                                    &Parameter,
//                                    sizeof(Parameter),
//                                    NULL,
//                                    0)
//                 < 0)
//             {
//                   return ScsiAdapterControlUnsuccessful;
//             }
//             break;
//         }
//
//         case ScsiRestartAdapter:
//         // Reinitializes an HBA. The port driver calls HwScsiAdapterControl with this control type to power
//         // up an HBA that was shut down for power management. All resources previously assigned to the miniport
//         // are still available, and its device extension and logical unit extensions, if any, are intact.
//         // The miniport performs the same operations as in its HwScsiInitialize routine, such as setting up
//         // the HBA's registers and its initial state, if any.
//         // The miniport must not call routines that can only be called from HwScsiFindAdapter or from
//         // HwScsiAdapterControl when the control type is ScsiSetRunningConfig, such as ScsiPortGetBusData and
//         // ScsiPortSetBusDataByOffset. If the miniport must call such routines to restart its HBA, it must also
//         // implement ScsiSetRunningConfig.
//         // If the miniport does not implement ScsiRestartAdapter, the port driver calls the miniport's HwScsiFindAdapter and HwScsiInitialize routines. However, because such routines might do detection work which is unnecessary for restarting the HBA, such a miniport will not power up its HBA as quickly as a miniport that implements ScsiRestartAdapter.
//
//         // See PR 69004. We were not calling BT958HwInitialize earlier if the
//         // first two calls succeeded thus causing resume from standby to fail.
//         {
//             if (!(BusLogic_HardwareResetHostAdapter(HostAdapter, TRUE) &&
//                   BusLogic_InitializeHostAdapter(deviceExtension, NULL) &&
//                   BT958HwInitialize(HwDeviceExtension))
//                )
//             {
//                 ScsiDebugPrint((ERROR, "\n Buslogic - Resetting %s Failed\n",
//                             HostAdapter->FullModelName));
//                 return ScsiAdapterControlUnsuccessful;
//             }
//
//             ScsiPortStallExecution(HostAdapter->BusSettleTime);
//             break;
//         }
//         default:
//         {
//             return ScsiAdapterControlUnsuccessful;
//         }
//     }
//     return ScsiAdapterControlSuccess;
// }// end BT958HwAdapterControl
//
// // END OF FILE BusLogic958.c
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
