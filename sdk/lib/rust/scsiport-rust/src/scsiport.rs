#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ANSI_STRING, CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_RESOURCE_LIST, CM_SCSI_DEVICE_DATA, DEVICE_DESCRIPTION, HalAllocateCommonBuffer, HalAssignSlotResources, HalGetBusData, HalGetInterruptVector, HalTranslateBusAddress, INT64, IoMapTransfer, IoQueryDeviceDescription, IoStartPacket, KeSynchronizeExecution, MAPPED_ADDRESS, MmMapIoSpace, MmUnmapIoSpace, _OBJECT_ATTRIBUTES, PCCHAR, PCI_COMMON_CONFIG, PCI_SLOT_NUMBER, PCM_FULL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PCM_SCSI_DEVICE_DATA, PCONFIGURATION_INFORMATION, PDEVICE_OBJECT, PDRIVER_OBJECT, PHYSICAL_ADDRESS, PIO_STACK_LOCATION, PIRP, PKAFFINITY, PKEY_VALUE_FULL_INFORMATION, PKINTERRUPT, PKIRQL, PMAPPED_ADDRESS, PPCI_SLOT_NUMBER, PUNICODE_STRING, RtlUnicodeStringToAnsiString, UINT8, ZwEnumerateValueKey, _KDPC, va_list, CM_FULL_RESOURCE_DESCRIPTOR, CONFIGURATION_INFORMATION, DEVICE_OBJECT, DRIVER_OBJECT, INTERFACE_TYPE, IO_ALLOCATION_ACTION, IO_STACK_LOCATION, IRP, KEY_VALUE_FULL_INFORMATION, KTIMER, _UNICODE_STRING};
use core::ffi::c_void;
use core::mem::{align_of, size_of, offset_of};
// The C-variadic entry points (ScsiDebugPrint / ScsiPortNotification) live in
// drivers/storage/port/scsiport/rust_varargs_shim.c — stable Rust cannot DEFINE one
// (c_variadic, rust-lang#44930). Each is a 3-line va_start trampoline into the
// fixed-arity `*_v` twin below, which holds ALL the logic. `VaArgs` reads the va_list.
use varargs_rust::VaArgs;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{ACCESS_RANGE, CONFIGURATION_INFO, SCSI_MAXIMUM_LOGICAL_UNITS, HW_INITIALIZATION_DATA, IO_SCSI_CAPABILITIES, PORT_CONFIGURATION_INFORMATION, RESETBUS_PARAMS, SCSI_BUS_INFO, SCSI_NOTIFICATION_TYPE, SCSI_PHYSICAL_ADDRESS, SCSI_PORT_COMMON_EXTENSION, SCSI_PORT_DEVICE_EXTENSION, SCSI_PORT_DRIVER_EXTENSION, SCSI_PORT_LUN_EXTENSION, SCSI_REQUEST_BLOCK, SCSI_REQUEST_BLOCK_INFO, SCSI_SG_ADDRESS, SENSE_DATA, PSCSI_PORT_DEVICE_EXTENSION};
// resolve_imports.py: explicit kernel/private-type imports

extern "C" {
    fn vsprintf( buffer: *mut core::ffi::c_char, format: *const core::ffi::c_char, args: *mut core::ffi::c_char, ) -> i32;
    fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32;
    // ScsiPortNotification's variadic ENTRY is the trampoline in rust_varargs_shim.c;
    // the logic is ScsiPortNotification_v below. Declaring + CALLING a C-variadic fn is
    // stable (only DEFINING one isn't), so the in-crate call sites are unchanged.
    // Same decl as scsi.rs:23. SCSI_NOTIFICATION_TYPE is a C enum -> c_uint.
    fn ScsiPortNotification(
        NotificationType: SCSI_NOTIFICATION_TYPE,
        HwDeviceExtension: *mut c_void,
        ...
    );
}
extern "C" {
    fn FdoDispatchPnp(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP) -> NTSTATUS;
    fn PdoDispatchPnp(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP) -> NTSTATUS;
    fn DbgPrint(format: *const i8, ...);
    fn GetLunByPath( DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION, PathId: u8, TargetId: u8, Lun: u8, ) -> *mut SCSI_PORT_LUN_EXTENSION;
    fn SpiGetSrbData(DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION, LunExtension: *mut SCSI_PORT_LUN_EXTENSION, QueueTag: u8) -> *mut SCSI_REQUEST_BLOCK_INFO;
    fn _strnicmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
    fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32;
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
    pub fn FdoCallHWInitialize(DeviceExtension: PSCSI_PORT_DEVICE_EXTENSION) -> NTSTATUS;
    fn FdoStartAdapter(DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION) -> NTSTATUS;
    fn FdoScanAdapter(DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION);
    pub fn FdoRemoveAdapter(DeviceExtension: PSCSI_PORT_DEVICE_EXTENSION) -> NTSTATUS;
    fn SpiInitOpenKeys(ConfigInfo: *mut CONFIGURATION_INFO, DriverExtension: *mut SCSI_PORT_DRIVER_EXTENSION);
}
// NTAPI = stdcall on i386 — extern "system" resolves per-arch.
extern "system" {
    fn HalSetBusDataByOffset( BusDataType: u32, SystemIoBusNumber: u32, SlotNumber: u32, Buffer: *mut core::ffi::c_void, Offset: u32, Length: u32, ) -> u32;
    fn HalGetAdapter( NumberOfMapRegisters: *mut DEVICE_DESCRIPTION, MapRegistersCount: *mut u32, ) -> *mut core::ffi::c_void;
    fn RtlInitUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *const u16);
    fn ZwOpenKey( KeyHandle: *mut HANDLE, DesiredAccess: u32, ObjectAttributes: *mut _OBJECT_ATTRIBUTES, ) -> NTSTATUS;
    fn ZwClose(Handle: HANDLE) -> NTSTATUS;
    fn ExAllocatePoolWithTag(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: u32) -> *mut c_void;
    fn ExFreePool(PoolHandle: *mut c_void);
    fn ExFreePoolWithTag(PoolHandle: *mut c_void, Tag: u32);
    fn IoAllocateDriverObjectExtension(DriverObject: *mut DRIVER_OBJECT, ClientIdentificationAddress: *mut c_void, ExtensionSize: u32, Extension: *mut *mut c_void) -> NTSTATUS;
    fn IoGetDriverObjectExtension(DriverObject: *mut DRIVER_OBJECT, ClientIdentificationAddress: *mut c_void) -> *mut c_void;
    fn IoCreateDevice(DriverObject: *mut DRIVER_OBJECT, DeviceExtensionSize: u32, DeviceName: *mut _UNICODE_STRING, DeviceType: u32, DeviceCharacteristics: u32, Exclusive: BOOLEAN, DeviceObject: *mut *mut DEVICE_OBJECT) -> NTSTATUS;
    fn KeInitializeTimer(Timer: *mut KTIMER);
    fn KeInitializeDpc(Dpc: *mut KDPC, DeferredRoutine: PVOID, DeferredContext: PVOID);
    fn IoInitializeTimer(DeviceObject: *mut DEVICE_OBJECT, TimerRoutine: PVOID, Context: *mut c_void);
    fn IoReportDetectedDevice(DriverObject: *mut DRIVER_OBJECT, LegacyBusType: INTERFACE_TYPE, BusNumber: u32, SlotNumber: PCI_SLOT_NUMBER, ResourceList: *mut CM_RESOURCE_LIST, ResourceRequirements: *mut c_void, DetectedAtBoot: BOOLEAN, DeviceObject: *mut *mut DEVICE_OBJECT) -> NTSTATUS;
    fn IoAttachDeviceToDeviceStack(SourceDevice: *mut DEVICE_OBJECT, TargetDevice: *mut DEVICE_OBJECT) -> *mut DEVICE_OBJECT;
    fn IoGetConfigurationInformation() -> *mut CONFIGURATION_INFORMATION;
    fn RtlCopyUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *mut _UNICODE_STRING);
    fn RtlCreateUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *const u16) -> u8;
}

// ---- file-scope preamble ----
// The LLM preamble ("values inferred from DDK" — several fabricated: NEXT_REQUEST_READY
// 0x01 vs real 0x00008, DISABLE_INTERRUPTS 'placeholder' 1 vs 0x04000, a third wrong
// STATUS_DEVICE_DOES_NOT_EXIST) was deleted; glue.rs + bindgen types are the verified
// source. InternalDebugLevel lives in glue.rs (single storage location). The dup extern
// decls for SpiGetSrbData/GetLunByPath (defined in this module) were dropped.
pub const NDEBUG: bool = true;
use crate::scsi::{ScsiPortStartPacket, ScsiPortStartIo, ScsiPortDispatchScsi, ScsiPortDpcForIsr};
use crate::ioctl::ScsiPortDeviceControl;
use crate::power::ScsiPortDispatchPower;
use crate::types::{SCSI_PORT_NEXT_REQUEST_READY, SCSI_PORT_RESET_REPORTED, SCSI_PORT_TIMER_NEEDED};

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(DriverObject: *mut DRIVER_OBJECT, RegistryPath: *mut _UNICODE_STRING) -> i32 {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortUnload(
    DriverObject: *mut DRIVER_OBJECT,
) {
    // no-op
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortDispatchPnp(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
) -> NTSTATUS {
    if ((*DeviceObject).DeviceExtension as *mut SCSI_PORT_COMMON_EXTENSION)
        .as_ref()
        .unwrap()
        .IsFDO != 0
    {
        return FdoDispatchPnp(DeviceObject, Irp);
    } else {
        return PdoDispatchPnp(DeviceObject, Irp);
    }
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortAddDevice(
    DriverObject: *mut DRIVER_OBJECT,
    PhysicalDeviceObject: *mut DEVICE_OBJECT,
) -> NTSTATUS {
    crate::glue::dprint_noop!(
        b"AddDevice no-op DriverObj: %p, PDO: %p\n\0".as_ptr() as *const i8,
        DriverObject as *mut core::ffi::c_void,
        PhysicalDeviceObject as *mut core::ffi::c_void,
    );
    return STATUS_SUCCESS;
}

/// Fixed-arity twin of the `ScsiDebugPrint` C-variadic entry point in
/// `drivers/storage/port/scsiport/rust_varargs_shim.c`. The C side exists only to do
/// the `va_start` that stable Rust cannot (`c_variadic`, rust-lang#44930). This body
/// never `va_arg`s — it forwards the whole list to `vsprintf` — so the shim hands over
/// the `va_list` BY VALUE (same arrangement as the shipping
/// sdk/lib/rtl/dbgprint_variadic.c ↔ rtl-diag pair; `va_list` == `*mut c_char` on all
/// three arches, which is exactly the file-scope `vsprintf` decl's argptr type).
#[no_mangle]
pub unsafe extern "C" fn ScsiDebugPrint_v(
    DebugPrintLevel: u32,
    DebugMessage: *const core::ffi::c_char,
    ap: va_list,
) {
    if DebugPrintLevel > InternalDebugLevel {
        return;
    }

    let mut buffer: [core::ffi::c_char; 256] = [0; 256];
    // ntoskrnl CRT export vsprintf (@ cdecl), called with the C frame's va_list.
    vsprintf(buffer.as_mut_ptr(), DebugMessage, ap);

    DbgPrint((buffer.as_ptr() as *const u8).cast());
}

#[no_mangle]
pub unsafe extern "system" fn SpiCompleteRequest(
    HwDeviceExtension: *mut core::ffi::c_void,
    SrbInfo: *mut SCSI_REQUEST_BLOCK_INFO,
    SrbStatus: u8,
) {
    let Srb: *mut SCSI_REQUEST_BLOCK;

    Srb = (*SrbInfo).Srb;

    if Srb.is_null() || ((*Srb).SrbFlags & SRB_FLAGS_IS_ACTIVE) == 0 {
        return;
    }

    (*Srb).SrbStatus = SrbStatus;
    (*Srb).DataTransferLength = 0;

    ScsiPortNotification(RequestComplete, HwDeviceExtension, Srb);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortCompleteRequest(
    HwDeviceExtension: *mut core::ffi::c_void,
    PathId: u8,
    TargetId: u8,
    Lun: u8,
    SrbStatus: u8,
) {
    // DPRINT("ScsiPortCompleteRequest() called\n");
    if InternalDebugLevel != 0 {
        crate::glue::dprint_noop!(b"ScsiPortCompleteRequest() called\n\0".as_ptr() as *const i8);
    }

    let device_extension = (HwDeviceExtension as *mut u8)
        .offset(-(core::mem::offset_of!(SCSI_PORT_DEVICE_EXTENSION, MiniPortDeviceExtension) as isize))
        as *mut SCSI_PORT_DEVICE_EXTENSION;

    let mut pathId_loop: u8 = 0;
    while pathId_loop < (*device_extension).NumberOfBuses {
        let bus = (*device_extension).Buses.add(pathId_loop as usize);
        let mut lun_entry = (*bus).LunsListHead.Flink;
        while lun_entry != core::ptr::addr_of!((*bus).LunsListHead) as *mut _ {
            let lun_extension = (lun_entry as *mut u8)
                .offset(-(core::mem::offset_of!(SCSI_PORT_LUN_EXTENSION, LunEntry) as isize))
                as *mut SCSI_PORT_LUN_EXTENSION;

            if (PathId == 0xFF /* SP_UNTAGGED */ || PathId == (*lun_extension).PathId) &&
               (TargetId == 0xFF /* SP_UNTAGGED */ || TargetId == (*lun_extension).TargetId) &&
               (Lun == 0xFF /* SP_UNTAGGED */ || Lun == (*lun_extension).Lun) {

                if !(*lun_extension).CompletedAbortRequests.is_null() {
                    // DPRINT1("Completing abort request without setting SrbStatus!\n");
                    if InternalDebugLevel != 0 {
                        DbgPrint(b"Completing abort request without setting SrbStatus!\n\0".as_ptr() as *const i8);
                    }
                    ScsiPortNotification(0 /* RequestComplete */, HwDeviceExtension, (*lun_extension).CompletedAbortRequests);
                }

                SpiCompleteRequest(HwDeviceExtension, core::ptr::addr_of_mut!((*lun_extension).SrbInfo), SrbStatus);

                let mut list_entry = (*lun_extension).SrbInfo.Requests.Flink;
                while list_entry != core::ptr::addr_of_mut!((*lun_extension).SrbInfo.Requests) {
                    let srb_info = (list_entry as *mut u8)
                        .offset(-(core::mem::offset_of!(SCSI_REQUEST_BLOCK_INFO, Requests) as isize))
                        as *mut SCSI_REQUEST_BLOCK_INFO;

                    SpiCompleteRequest(HwDeviceExtension, srb_info, SrbStatus);
                    list_entry = (*srb_info).Requests.Flink;
                }
            }

            lun_entry = (*lun_entry).Flink;
        }
        pathId_loop += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortFlushDma(HwDeviceExtension: *mut core::ffi::c_void) {
    if InternalDebugLevel != 0 {
        crate::glue::dprint_noop!(b"ScsiPortFlushDma()\n\0".as_ptr() as *const i8);
    }
    DbgPrint(b"UNIMPLEMENTED\n\0".as_ptr() as *const i8);
    DbgBreakPoint();
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortFreeDeviceBase(
    HwDeviceExtension: *mut core::ffi::c_void,
    MappedAddress: *mut core::ffi::c_void,
) {
    let device_extension: *mut SCSI_PORT_DEVICE_EXTENSION = {
        let offset = core::mem::offset_of!(SCSI_PORT_DEVICE_EXTENSION, MiniPortDeviceExtension);
        (HwDeviceExtension as *mut u8).sub(offset) as *mut SCSI_PORT_DEVICE_EXTENSION
    };
    let mut NextMa: *mut MAPPED_ADDRESS = (*device_extension).MappedAddressList;
    let mut LastMa: *mut MAPPED_ADDRESS = NextMa;
    while !NextMa.is_null() {
        if (*NextMa).MappedAddress == MappedAddress {
            MmUnmapIoSpace(MappedAddress, ((*NextMa).NumberOfBytes) as _);
            if NextMa == (*device_extension).MappedAddressList {
                (*device_extension).MappedAddressList = (*NextMa).NextMappedAddress;
            } else {
                (*LastMa).NextMappedAddress = (*NextMa).NextMappedAddress;
            }
            ExFreePool(NextMa as *mut core::ffi::c_void);
            return;
        } else {
            LastMa = NextMa;
            NextMa = (*NextMa).NextMappedAddress;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortGetBusData(
    DeviceExtension: *mut core::ffi::c_void,
    BusDataType: u32,
    SystemIoBusNumber: u32,
    SlotNumber: u32,
    Buffer: *mut core::ffi::c_void,
    Length: u32,
) -> u32 {
    unsafe {
        crate::glue::dprint_noop!((b"ScsiPortGetBusData()\n\0".as_ptr()).cast());
        if Length != 0 {
            return HalGetBusData((BusDataType) as i32, SystemIoBusNumber, SlotNumber, Buffer, Length);
        }
        DbgBreakPoint();
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortSetBusDataByOffset(
    DeviceExtension: *mut core::ffi::c_void,
    BusDataType: u32,
    SystemIoBusNumber: u32,
    SlotNumber: u32,
    Buffer: *mut core::ffi::c_void,
    Offset: u32,
    Length: u32,
) -> u32 {
    crate::glue::dprint_noop!(b"ScsiPortSetBusDataByOffset()\n\0".as_ptr() as *const i8);
    HalSetBusDataByOffset(
        BusDataType,
        SystemIoBusNumber,
        SlotNumber,
        Buffer,
        Offset,
        Length,
    )
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortGetDeviceBase(
    HwDeviceExtension: *mut core::ffi::c_void,
    BusType: INTERFACE_TYPE,
    SystemIoBusNumber: u32,
    IoAddress: SCSI_PHYSICAL_ADDRESS,
    NumberOfBytes: u32,
    InIoSpace: u8,
) -> *mut core::ffi::c_void {
    let device_extension = (HwDeviceExtension as *const u8)
        .sub(core::mem::offset_of!(SCSI_PORT_DEVICE_EXTENSION, MiniPortDeviceExtension))
        as *mut SCSI_PORT_DEVICE_EXTENSION;
    let mut address_space: u32 = InIoSpace as u32;
    let mut translated_address: PHYSICAL_ADDRESS = core::mem::zeroed();
    if HalTranslateBusAddress(
        BusType,
        SystemIoBusNumber,
        IoAddress,
        &mut address_space as *mut u32,
        &mut translated_address as *mut PHYSICAL_ADDRESS,
    ) == 0
    {
        return core::ptr::null_mut();
    }
    if address_space != 0 {
        return translated_address.QuadPart as *mut core::ffi::c_void;
    }
    let mapped_address = MmMapIoSpace(translated_address, NumberOfBytes as _, 0u32);
    let device_base = ExAllocatePoolWithTag(
        NonPagedPool,
        (core::mem::size_of::<MAPPED_ADDRESS>()) as SIZE_T,
        0 /* TAG_SCSIPORT */,
    ) as *mut MAPPED_ADDRESS;
    if device_base.is_null() {
        return mapped_address;
    }
    (*device_base).MappedAddress = mapped_address;
    (*device_base).NumberOfBytes = NumberOfBytes;
    (*device_base).IoAddress = IoAddress;
    (*device_base).BusNumber = SystemIoBusNumber;
    (*device_base).NextMappedAddress = (*device_extension).MappedAddressList;
    (*device_extension).MappedAddressList = device_base;
    mapped_address
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortGetLogicalUnit(
    HwDeviceExtension: *mut core::ffi::c_void,
    PathId: u8,
    TargetId: u8,
    Lun: u8,
) -> *mut core::ffi::c_void {
    let DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION;
    let LunExtension: *mut SCSI_PORT_LUN_EXTENSION;

    crate::glue::dprint_noop!(
        b"ScsiPortGetLogicalUnit() called\n\0".as_ptr() as *const i8,
    );

    DeviceExtension = (HwDeviceExtension as *mut u8)
        .sub(core::mem::offset_of!(
            SCSI_PORT_DEVICE_EXTENSION,
            MiniPortDeviceExtension
        )) as *mut SCSI_PORT_DEVICE_EXTENSION;

    /* Check the extension size */
    if (*DeviceExtension).LunExtensionSize == 0 {
        /* They didn't want one */
        return core::ptr::null_mut();
    }

    LunExtension = GetLunByPath(DeviceExtension, PathId, TargetId, Lun);

    /* Check that the logical unit exists */
    if LunExtension.is_null() {
        /* Nope, return NULL */
        return core::ptr::null_mut();
    }

    /* Return the logical unit miniport extension */
    return (LunExtension.add(1)) as *mut core::ffi::c_void;
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortGetPhysicalAddress(
    HwDeviceExtension: *mut core::ffi::c_void,
    Srb: *mut SCSI_REQUEST_BLOCK,
    VirtualAddress: *mut core::ffi::c_void,
    Length: *mut u32,
) -> SCSI_PHYSICAL_ADDRESS {
    crate::glue::dprint_noop!(
        b"ScsiPortGetPhysicalAddress(%p %p %p %p)\n\0".as_ptr() as *const i8,
        HwDeviceExtension as *const core::ffi::c_void,
        Srb as *const core::ffi::c_void,
        VirtualAddress as *const core::ffi::c_void,
        Length as *const core::ffi::c_void,
    );

    let DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION = (HwDeviceExtension as *mut u8)
        .sub(core::mem::offset_of!(
            SCSI_PORT_DEVICE_EXTENSION,
            MiniPortDeviceExtension
        )) as *mut SCSI_PORT_DEVICE_EXTENSION;

    let mut PhysicalAddress: SCSI_PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut BufferLength: usize = 0;

    if Srb.is_null() || (*Srb).SenseInfoBuffer == VirtualAddress {
        let Offset =
            (VirtualAddress as usize) - ((*DeviceExtension).SrbExtensionBuffer as usize);
        BufferLength = (((*DeviceExtension).CommonBufferLength - ((Offset) as usize) as u32) as u32) as usize;
        PhysicalAddress.QuadPart =
            (*DeviceExtension).PhysicalAddress.QuadPart + Offset as i64;
    } else if (*DeviceExtension).MapRegisters != 0 {
        let IoStack =
            IoGetCurrentIrpStackLocation(((*Srb).OriginalRequest).cast());
        let LunExtension =
            (*(*IoStack).DeviceObject).DeviceExtension as *mut SCSI_PORT_LUN_EXTENSION;

        ntkrnl_sys::ASSERT!(!LunExtension.is_null());
        ntkrnl_sys::ASSERT!((*LunExtension).Common.IsFDO == 0);

        let SrbInfo = SpiGetSrbData(DeviceExtension, LunExtension, (((*Srb).QueueTag) as u32) as _);
        let mut SGList = (*SrbInfo).ScatterGather;

        let mut Offset = (VirtualAddress as usize) - ((*Srb).DataBuffer as usize);
        while Offset >= (*SGList).Length as usize {
            Offset -= (*SGList).Length as usize;
            SGList = SGList.add(1);
        }

        BufferLength = (*SGList).Length as usize - Offset;
        PhysicalAddress.QuadPart =
            (*SGList).PhysicalAddress.QuadPart + Offset as i64;
    } else {
        PhysicalAddress.QuadPart = SP_UNINITIALIZED_VALUE as i64;
    }

    *Length = BufferLength as u32;
    PhysicalAddress
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortGetSrb(
    DeviceExtension: *mut core::ffi::c_void,
    PathId: u8,
    TargetId: u8,
    Lun: u8,
    QueueTag: i32,
) -> *mut SCSI_REQUEST_BLOCK {
    DbgPrint(b"ScsiPortGetSrb() unimplemented\n\0".as_ptr() as *const i8);
    DbgBreakPoint();
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortGetUncachedExtension(
    HwDeviceExtension: *mut core::ffi::c_void,
    ConfigInfo: *mut PORT_CONFIGURATION_INFORMATION,
    NumberOfBytes: u32,
) -> *mut core::ffi::c_void {
    // Forward declaration for HalGetAdapter (if not already available)
    // Forward declaration for SpiAllocateCommonBuffer (local static)

    let device_extension: *mut SCSI_PORT_DEVICE_EXTENSION;
    let mut device_description: DEVICE_DESCRIPTION = core::mem::zeroed();
    let mut map_registers_count: u32 = 0;
    let mut status: i32;

    // DPRINT call (mapped to DbgPrint)
    crate::glue::dprint_noop!(
        b"ScsiPortGetUncachedExtension(%p %p %lu)\n\0".as_ptr() as *const i8,
        HwDeviceExtension,
        ConfigInfo,
        NumberOfBytes,
    );

    // CONTAINING_RECORD expansion
    device_extension = (HwDeviceExtension as *mut u8)
        .sub(core::mem::offset_of!(SCSI_PORT_DEVICE_EXTENSION, MiniPortDeviceExtension))
        as *mut SCSI_PORT_DEVICE_EXTENSION;

    // Check for allocated common DMA buffer
    if !(*device_extension).SrbExtensionBuffer.is_null() {
        // DPRINT1 call
        DbgPrint(
            b"The HBA has already got a common DMA buffer!\n\0".as_ptr() as *const i8,
        );
        return core::ptr::null_mut();
    }

    // Check for DMA adapter object
    if (*device_extension).AdapterObject.is_null() {
        // Initialize DMA adapter description
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!(device_description) as *mut u8,
            0,
            core::mem::size_of::<DEVICE_DESCRIPTION>(),
        );

        // Set fields from ConfigInfo
        (*core::ptr::addr_of_mut!(device_description)).Version = DEVICE_DESCRIPTION_VERSION;
        (*core::ptr::addr_of_mut!(device_description)).Master = (*ConfigInfo).Master;
        (*core::ptr::addr_of_mut!(device_description)).ScatterGather = (*ConfigInfo).ScatterGather;
        (*core::ptr::addr_of_mut!(device_description)).DemandMode = (*ConfigInfo).DemandMode;
        (*core::ptr::addr_of_mut!(device_description)).Dma32BitAddresses = (*ConfigInfo).Dma32BitAddresses;
        (*core::ptr::addr_of_mut!(device_description)).BusNumber = (*ConfigInfo).SystemIoBusNumber;
        (*core::ptr::addr_of_mut!(device_description)).DmaChannel = (*ConfigInfo).DmaChannel;
        (*core::ptr::addr_of_mut!(device_description)).InterfaceType = (*ConfigInfo).AdapterInterfaceType;
        (*core::ptr::addr_of_mut!(device_description)).DmaWidth = (*ConfigInfo).DmaWidth;
        (*core::ptr::addr_of_mut!(device_description)).DmaSpeed = (*ConfigInfo).DmaSpeed;
        (*core::ptr::addr_of_mut!(device_description)).MaximumLength = (*ConfigInfo).MaximumTransferLength;
        (*core::ptr::addr_of_mut!(device_description)).DmaPort = (*ConfigInfo).DmaPort;

        // Get a DMA adapter object
        (*device_extension).AdapterObject =
            (HalGetAdapter(
                core::ptr::addr_of_mut!(device_description),
                core::ptr::addr_of_mut!(map_registers_count),
            )).cast();

        // Fail in case of error
        if (*device_extension).AdapterObject.is_null() {
            DbgPrint(
                b"HalGetAdapter() failed\n\0".as_ptr() as *const i8,
            );
            return core::ptr::null_mut();
        }

        // Set number of physical breaks
        if (*ConfigInfo).NumberOfPhysicalBreaks != 0 &&
            map_registers_count > (*ConfigInfo).NumberOfPhysicalBreaks
        {
            (*device_extension).PortCapabilities.MaximumPhysicalPages =
                (*ConfigInfo).NumberOfPhysicalBreaks;
        } else {
            (*device_extension).PortCapabilities.MaximumPhysicalPages = map_registers_count;
        }
    }

    // Update auto request sense feature
    (*device_extension).SupportsAutoSense = (*ConfigInfo).AutoRequestSense;

    // Update Srb extension size
    if (*device_extension).SrbExtensionSize != (*ConfigInfo).SrbExtensionSize {
        (*device_extension).SrbExtensionSize = (*ConfigInfo).SrbExtensionSize;
    }

    // Update Srb extension alloc flag
    if (*ConfigInfo).AutoRequestSense != 0 || (*device_extension).SrbExtensionSize != 0 {
        (*device_extension).NeedSrbExtensionAlloc = 1; // TRUE
    }

    // Allocate a common DMA buffer
    status = SpiAllocateCommonBuffer(device_extension, NumberOfBytes);

    // Check NT_SUCCESS (expanded as Status >= 0)
    if !(status >= 0) {
        DbgPrint(
            b"SpiAllocateCommonBuffer() failed with Status = 0x%08X!\n\0".as_ptr() as *const i8,
            status,
        );
        return core::ptr::null_mut();
    }

    (*device_extension).NonCachedExtension
}

// Provide the missing constant
const DEVICE_DESCRIPTION_VERSION: u32 = 0x00;

unsafe fn SpiAllocateCommonBuffer(
    DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION,
    NonCachedSize: u32,
) -> i32 {
    let mut CommonBuffer: *mut core::ffi::c_void;
    let mut CommonBufferLength: u32;
    let mut BufSize: u32;

    if (*DeviceExtension).SrbExtensionSize == 0 {
        (*DeviceExtension).SrbExtensionSize = 16;
    }

    BufSize = (*DeviceExtension).SrbExtensionSize;

    if (*DeviceExtension).SupportsAutoSense != 0 {
        BufSize = BufSize.wrapping_add(core::mem::size_of::<SENSE_DATA>() as u32);
    }

    BufSize = (BufSize.wrapping_add(7)) & !7u32;

    CommonBufferLength = ROUND_TO_PAGES(
        (NonCachedSize.wrapping_add(BufSize.wrapping_mul((*DeviceExtension).RequestsNumber))) as usize
    ) as u32;

    if (*DeviceExtension).AdapterObject.is_null() {
        CommonBuffer = ExAllocatePoolWithTag(0 /* NonPagedPool */, CommonBufferLength as SIZE_T, 0x50726F53 /* TAG_SCSIPORT */);
    } else {
        CommonBuffer = HalAllocateCommonBuffer(
            (*DeviceExtension).AdapterObject,
            CommonBufferLength,
            core::ptr::addr_of_mut!((*DeviceExtension).PhysicalAddress),
            0,
        );
    }

    if CommonBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(CommonBuffer, CommonBufferLength as usize);

    (*DeviceExtension).CommonBufferLength = CommonBufferLength;

    (*DeviceExtension).SrbExtensionBuffer = CommonBuffer;

    if NonCachedSize != 0 {
        CommonBufferLength = CommonBufferLength.wrapping_sub(NonCachedSize);
        (*DeviceExtension).NonCachedExtension = (CommonBuffer as *mut u8).add(CommonBufferLength as usize) as *mut c_void;
    } else {
        (*DeviceExtension).NonCachedExtension = core::ptr::null_mut();
    }

    if (*DeviceExtension).NeedSrbExtensionAlloc != 0 {
        (*DeviceExtension).SrbDataCount = CommonBufferLength / BufSize;

        let mut SrbExtension: *mut *mut c_void = CommonBuffer as *mut *mut c_void;
        (*DeviceExtension).FreeSrbExtensions = (SrbExtension).cast();

        while CommonBufferLength >= 2 * BufSize {
            let next_ptr = (SrbExtension as *mut u8).add(BufSize as usize) as *mut *mut c_void;
            core::ptr::write(SrbExtension, (next_ptr).cast());
            SrbExtension = core::ptr::read(SrbExtension) as *mut *mut c_void;

            CommonBufferLength = CommonBufferLength.wrapping_sub(BufSize);
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortGetVirtualAddress(
    HwDeviceExtension: *mut core::ffi::c_void,
    PhysicalAddress: SCSI_PHYSICAL_ADDRESS,
) -> *mut core::ffi::c_void {
    // DPRINT("ScsiPortGetVirtualAddress(%p %I64x)\n", HwDeviceExtension, PhysicalAddress.QuadPart);
    crate::glue::dprint_noop!(
        b"ScsiPortGetVirtualAddress(%p %I64x)\n\0".as_ptr() as *const i8,
        HwDeviceExtension,
        PhysicalAddress.QuadPart,
    );

    let device_extension = ((HwDeviceExtension as usize)
        .wrapping_sub(core::mem::offset_of!(SCSI_PORT_DEVICE_EXTENSION, MiniPortDeviceExtension)))
        as *mut SCSI_PORT_DEVICE_EXTENSION;

    if (*device_extension).PhysicalAddress.QuadPart > PhysicalAddress.QuadPart {
        return core::ptr::null_mut();
    }

    let offset: u32 =
        (PhysicalAddress.QuadPart - (*device_extension).PhysicalAddress.QuadPart) as u32;

    if offset >= (*device_extension).CommonBufferLength {
        return core::ptr::null_mut();
    }

    ((*device_extension).SrbExtensionBuffer as usize + offset as usize) as *mut core::ffi::c_void
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortInitialize(
    Argument1: *mut core::ffi::c_void,
    Argument2: *mut core::ffi::c_void,
    HwInitializationData: *mut HW_INITIALIZATION_DATA,
    HwContext: *mut core::ffi::c_void,
) -> u32 {
    unsafe {
        let DriverObject: *mut DRIVER_OBJECT = Argument1 as *mut DRIVER_OBJECT;
        let RegistryPath: *mut _UNICODE_STRING = Argument2 as *mut _UNICODE_STRING;
        let mut DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION = core::ptr::null_mut();
        let SystemConfig: *mut CONFIGURATION_INFORMATION = IoGetConfigurationInformation();
        let mut PortConfig: *mut PORT_CONFIGURATION_INFORMATION;
        let mut ConfigInfo: CONFIGURATION_INFO = core::mem::zeroed();
        let mut DeviceExtensionSize: u32;
        let mut PortConfigSize: u32;
        let mut Again: BOOLEAN;
        let mut DeviceFound: BOOLEAN = 0;
        let mut FirstConfigCall: BOOLEAN = 1;
        let mut Result: u32;
        let mut Status: NTSTATUS;
        let mut SlotNumber: PCI_SLOT_NUMBER = core::mem::zeroed();
        let mut PortDeviceObject: *mut DEVICE_OBJECT = core::ptr::null_mut();
        let mut DeviceName: _UNICODE_STRING = core::mem::zeroed();
        let mut PortCapabilities: *mut IO_SCSI_CAPABILITIES;
        let mut ResourceList: *mut CM_RESOURCE_LIST;

        crate::glue::dprint_noop!((b"ScsiPortInitialize() called!\n\0".as_ptr() as *const u8).cast());

        if (*HwInitializationData).HwInitialize.is_none() ||
            (*HwInitializationData).HwStartIo.is_none() ||
            (*HwInitializationData).HwInterrupt.is_none() ||
            (*HwInitializationData).HwFindAdapter.is_none() ||
            (*HwInitializationData).HwResetBus.is_none()
        {
            return (STATUS_REVISION_MISMATCH) as u32;
        }

        let mut driverExtension: *mut SCSI_PORT_DRIVER_EXTENSION =
            IoGetDriverObjectExtension(
                DriverObject,
                core::mem::transmute::<_, *mut c_void>((*HwInitializationData).HwInitialize),
            ) as *mut SCSI_PORT_DRIVER_EXTENSION;

        if driverExtension.is_null() {
            Status = IoAllocateDriverObjectExtension(
                DriverObject,
                core::mem::transmute::<_, *mut c_void>((*HwInitializationData).HwInitialize),
                core::mem::size_of::<SCSI_PORT_DRIVER_EXTENSION>() as u32,
                &mut driverExtension as *mut *mut SCSI_PORT_DRIVER_EXTENSION as *mut *mut c_void,
            );
            if Status < 0 {
                DbgPrint(
                    (b"Failed to allocate the driver extension! Status 0x%x\n\0".as_ptr() as *const u8).cast(),
                    Status,
                );
                return (Status) as u32;
            }
        }

        (*driverExtension).RegistryPath.Buffer = ExAllocatePoolWithTag(
            PagedPool,
            ((*RegistryPath).MaximumLength) as SIZE_T,
            TAG_SCSIPORT /* TAG_SCSIPORT */,
        ) as *mut u16;
        (*driverExtension).RegistryPath.MaximumLength = (*RegistryPath).MaximumLength;
        RtlCopyUnicodeString(&mut (*driverExtension).RegistryPath, RegistryPath);
        (*driverExtension).DriverObject = DriverObject;

        (*DriverObject).DriverUnload = Some(core::mem::transmute(ScsiPortUnload as *const ()));
        (*DriverObject).DriverStartIo = Some(core::mem::transmute(ScsiPortStartIo as *const ()));
        (*(*DriverObject).DriverExtension).AddDevice = Some(core::mem::transmute(ScsiPortAddDevice as *const ()));
        (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = Some(core::mem::transmute(ScsiPortCreateClose as *const ()));
        (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = Some(core::mem::transmute(ScsiPortCreateClose as *const ()));
        (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(core::mem::transmute(ScsiPortDeviceControl as *const ()));
        (*DriverObject).MajorFunction[IRP_MJ_SCSI as usize] = Some(core::mem::transmute(ScsiPortDispatchScsi as *const ()));
        (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = Some(core::mem::transmute(ScsiPortDispatchPnp as *const ()));
        (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = Some(core::mem::transmute(ScsiPortDispatchPower as *const ()));

        // write_bytes counts ELEMENTS — cast to *mut u8 for byte semantics (the
        // typed-pointer form zeroed size² bytes and smashed the stack frame,
        // BUGCHECK 0x1E CR2=0xC — the mountmgr element-vs-byte family).
        core::ptr::write_bytes(
            &mut ConfigInfo as *mut CONFIGURATION_INFO as *mut u8,
            0,
            core::mem::size_of::<CONFIGURATION_INFO>(),
        );
        SlotNumber.u.AsULONG = 0;

        if (*HwInitializationData).NumberOfAccessRanges != 0 {
            ConfigInfo.AccessRanges = ExAllocatePoolWithTag(
                PagedPool,
                ((*HwInitializationData).NumberOfAccessRanges as SIZE_T)
                    * core::mem::size_of::<ACCESS_RANGE>() as SIZE_T,
                TAG_SCSIPORT,
            ) as *mut ACCESS_RANGE;
            if ConfigInfo.AccessRanges.is_null() {
                return (STATUS_INSUFFICIENT_RESOURCES) as u32;
            }
        }

        SpiInitOpenKeys(&mut ConfigInfo, driverExtension);

        if (*driverExtension).IsLegacyDriver == 0 {
            __debugbreak();
        }

        ConfigInfo.LastAdapterNumber = (SP_UNINITIALIZED_VALUE) as u32;

        DeviceExtensionSize = core::mem::size_of::<SCSI_PORT_DEVICE_EXTENSION>() as u32
            + (*HwInitializationData).DeviceExtensionSize;

        crate::glue::dprint_noop!(
            (b"AdapterInterfaceType: %lu\n\0".as_ptr() as *const u8).cast(),
            (*HwInitializationData).AdapterInterfaceType as u32,
        );

        'outer: loop {
            let wformat: [u16; 20] = [
                0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C,
                0x0053, 0x0063, 0x0073, 0x0069, 0x0050, 0x006F, 0x0072, 0x0074,
                0x0025, 0x006C, 0x0075, 0x0000,
            ];
            let mut NameBuffer: [u16; 27] = [0; 27];
            swprintf(
                NameBuffer.as_mut_ptr(),
                wformat.as_ptr(),
                (*SystemConfig).ScsiPortCount as u32,
            );
            Status = if RtlCreateUnicodeString(&mut DeviceName, NameBuffer.as_ptr()) != 0 { 0 } else { STATUS_INSUFFICIENT_RESOURCES };
            if Status < 0 {
                DbgPrint((b"Failed to allocate memory for device name!\n\0".as_ptr() as *const u8).cast());
                Status = STATUS_INSUFFICIENT_RESOURCES;
                PortDeviceObject = core::ptr::null_mut();
                break 'outer;
            }

            crate::glue::dprint_noop!((b"Creating device: %wZ\n\0".as_ptr() as *const u8).cast(), &DeviceName);

            Status = IoCreateDevice(
                DriverObject,
                DeviceExtensionSize,
                &mut DeviceName,
                FILE_DEVICE_CONTROLLER,
                FILE_DEVICE_SECURE_OPEN,
                0,
                &mut PortDeviceObject,
            );
            if Status < 0 {
                DbgPrint(
                    (b"IoCreateDevice call failed! (Status 0x%lX)\n\0".as_ptr() as *const u8).cast(),
                    Status,
                );
                PortDeviceObject = core::ptr::null_mut();
                break 'outer;
            }

            (*PortDeviceObject).Flags |= DO_DIRECT_IO;
            (*PortDeviceObject).AlignmentRequirement = FILE_WORD_ALIGNMENT;

            DeviceExtension = (*PortDeviceObject).DeviceExtension as *mut SCSI_PORT_DEVICE_EXTENSION;
            core::ptr::write_bytes(DeviceExtension as *mut u8, 0, DeviceExtensionSize as usize);
            (*DeviceExtension).Common.DeviceObject = PortDeviceObject;
            (*DeviceExtension).Common.IsFDO = 1;
            (*DeviceExtension).Length = DeviceExtensionSize;
            (*DeviceExtension).PortNumber = (*SystemConfig).ScsiPortCount;
            (*DeviceExtension).DeviceName = DeviceName;

            (*DeviceExtension).HwInitialize = (*HwInitializationData).HwInitialize;
            (*DeviceExtension).HwStartIo = (*HwInitializationData).HwStartIo;
            (*DeviceExtension).HwInterrupt = (*HwInitializationData).HwInterrupt;
            (*DeviceExtension).HwResetBus = (*HwInitializationData).HwResetBus;
            (*DeviceExtension).HwDmaStarted = (*HwInitializationData).HwDmaStarted;

            (*DeviceExtension).MiniPortExtensionSize = (*HwInitializationData).DeviceExtensionSize;
            (*DeviceExtension).LunExtensionSize =
                ALIGN_UP((*HwInitializationData).SpecificLuExtensionSize, 8 /* INT64 */);
            (*DeviceExtension).SrbExtensionSize =
                ALIGN_UP((*HwInitializationData).SrbExtensionSize, 8 /* INT64 */);

            (*DeviceExtension).MaxLunCount = 8; // SCSI_MAXIMUM_LOGICAL_UNITS
            (*DeviceExtension).RequestsNumber = 16;

            KeInitializeSpinLock((&raw mut (*DeviceExtension).IrqLock).cast());
            KeInitializeSpinLock((&raw mut (*DeviceExtension).SpinLock).cast());

            IoInitializeDpcRequest(PortDeviceObject, ScsiPortDpcForIsr as *const () as *mut c_void);

            (*DeviceExtension).TimerCount = (-1i32 as u32) as i32;
            IoInitializeTimer(PortDeviceObject, ScsiPortIoTimer as *mut c_void, DeviceExtension as *mut c_void);

            KeInitializeTimer(&mut (*DeviceExtension).MiniportTimer);
            KeInitializeDpc(
                (&raw mut (*DeviceExtension).MiniportTimerDpc).cast(),
                SpiMiniportTimerDpc as *mut c_void,
                PortDeviceObject as *mut c_void,
            );

            'create_port_config: loop {
                PortConfigSize = core::mem::size_of::<PORT_CONFIGURATION_INFORMATION>() as u32
                    + (*HwInitializationData).NumberOfAccessRanges as u32
                        * core::mem::size_of::<ACCESS_RANGE>() as u32;
                PortConfigSize = ALIGN_UP(PortConfigSize, 8 /* INT64 */);
                (*DeviceExtension).PortConfig = ExAllocatePoolWithTag(
                    NonPagedPool,
                    PortConfigSize as SIZE_T,
                    TAG_SCSIPORT,
                ) as *mut PORT_CONFIGURATION_INFORMATION;
                if (*DeviceExtension).PortConfig.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'outer;
                }
                Status = SpiCreatePortConfig(
                    DeviceExtension,
                    HwInitializationData,
                    &mut ConfigInfo,
                    (*DeviceExtension).PortConfig,
                    FirstConfigCall,
                );
                if Status < 0 {
                    crate::glue::dprint_noop!(
                        (b"SpiCreatePortConfig() failed with Status 0x%08X\n\0".as_ptr() as *const u8).cast(),
                        Status,
                    );
                    break 'outer;
                }

                PortConfig = (*DeviceExtension).PortConfig;
                (*PortConfig).SpecificLuExtensionSize = (*DeviceExtension).LunExtensionSize;
                (*PortConfig).SrbExtensionSize = (*DeviceExtension).SrbExtensionSize;

                if (*HwInitializationData).NumberOfAccessRanges != 0 {
                    // C: ALIGN_UP_POINTER(PortConfig + 1, INT64) — the ranges live in the
                    // tail allocation ONE STRUCT PAST PortConfig. The LLM dropped the +1,
                    // aliasing AccessRanges onto PortConfig itself; the copy below then
                    // overwrote the struct (including the AccessRanges field at +0x38 ->
                    // NULL) and uniata's HwFindAdapter faulted at CR2=0x4.
                    (*PortConfig).AccessRanges = (ALIGN_UP_POINTER(
                        PortConfig.add(1) as *mut c_void,
                        8,
                    ) as *mut ACCESS_RANGE).cast();
                    core::ptr::copy_nonoverlapping(
                        ConfigInfo.AccessRanges,
                        ((*PortConfig).AccessRanges).cast(),
                        (*HwInitializationData).NumberOfAccessRanges as usize,
                    );
                }

                if (*HwInitializationData).AdapterInterfaceType == 5 /* PCIBus */
                    && (*HwInitializationData).VendorIdLength > 0
                    && !(*HwInitializationData).VendorId.is_null()
                    && (*HwInitializationData).DeviceIdLength > 0
                    && !(*HwInitializationData).DeviceId.is_null()
                {
                    (*PortConfig).BusInterruptLevel = 0;
                    crate::glue::dprint_noop!(
                        (b"VendorId '%.*s'  DeviceId '%.*s'\n\0".as_ptr() as *const u8).cast(),
                        (*HwInitializationData).VendorIdLength as u32,
                        (*HwInitializationData).VendorId,
                        (*HwInitializationData).DeviceIdLength as u32,
                        (*HwInitializationData).DeviceId,
                    );

                    if 0 == SpiGetPciConfigData(
                        DriverObject,
                        PortDeviceObject,
                        HwInitializationData,
                        PortConfig,
                        RegistryPath,
                        ConfigInfo.BusNumber,
                        &mut SlotNumber,
                    ) {
                        ConfigInfo.BusNumber += 1;
                        (*DeviceExtension).PortConfig = core::ptr::null_mut();
                        ExFreePool(PortConfig as *mut c_void);
                        Again = 0;
                        continue 'create_port_config;
                    }

                    if (*PortConfig).BusInterruptLevel == 0 {
                        (*DeviceExtension).PortConfig = core::ptr::null_mut();
                        ExFreePool(PortConfig as *mut c_void);
                        continue 'create_port_config;
                    }
                } else {
                    crate::glue::dprint_noop!((b"Non-pci bus\n\0".as_ptr() as *const u8).cast());
                }

                // Re-seed the AT-disk claim state from the authoritative system
                // configuration before every HwFindAdapter call. PortConfig is
                // reallocated fresh (non-zeroed) on each iteration of this loop and
                // SpiCreatePortConfig only seeds these fields when ZeroStruct
                // (FirstConfigCall) is TRUE. Without this a legacy miniport (uniata)
                // that re-enumerates per channel reads uninitialized garbage for
                // AtdiskPrimaryClaimed/AtdiskSecondaryClaimed, fails to notice a
                // channel it already claimed, keeps returning SP_RETURN_FOUND, and
                // this while-loop never terminates (BUGCHECK 0x50 — was flaky,
                // deterministic once the winsock catalog perturbs pool contents).
                {
                    let ddk = IoGetConfigurationInformation();
                    (*PortConfig).AtdiskPrimaryClaimed = (*ddk).AtDiskPrimaryAddressClaimed;
                    (*PortConfig).AtdiskSecondaryClaimed = (*ddk).AtDiskSecondaryAddressClaimed;
                }

                Again = 0;
                crate::glue::dprint_noop!(
                    (b"Calling HwFindAdapter() for Bus %lu\n\0".as_ptr() as *const u8).cast(),
                    (*PortConfig).SystemIoBusNumber,
                );

                Result = ((*HwInitializationData).HwFindAdapter.unwrap())(
                    (*DeviceExtension).MiniPortDeviceExtension.as_mut_ptr() as *mut c_void,
                    HwContext,
                    core::ptr::null_mut(), // BusInformation
                    ConfigInfo.Parameter as *mut i8,
                    PortConfig,
                    &mut Again,
                );

                crate::glue::dprint_noop!(
                    (b"HwFindAdapter() Result: %lu  Again: %s\n\0".as_ptr() as *const u8).cast(),
                    Result,
                    if Again != 0 { b"True\0".as_ptr() as *const u8 } else { b"False\0".as_ptr() as *const u8 },
                );

                if !(*DeviceExtension).MapRegisterBase.is_null() {
                    ExFreePool((*DeviceExtension).MapRegisterBase as *mut c_void);
                    (*DeviceExtension).MapRegisterBase = core::ptr::null_mut();
                }

                if Result != SP_RETURN_FOUND {
                    crate::glue::dprint_noop!(
                        (b"HwFindAdapter() Result: %lu\n\0".as_ptr() as *const u8).cast(),
                        Result,
                    );
                    if Result == SP_RETURN_NOT_FOUND {
                        ConfigInfo.BusNumber += 1;
                        Again = 0;
                        (*DeviceExtension).PortConfig = core::ptr::null_mut();
                        ExFreePool(PortConfig as *mut c_void);
                        continue 'create_port_config;
                    }
                    Status = STATUS_INTERNAL_ERROR;
                    break 'outer;
                }

                crate::glue::dprint_noop!(
                    (b"ScsiPortInitialize(): Found HBA! (%x), adapter Id %d\n\0".as_ptr() as *const u8).cast(),
                    (*PortConfig).BusInterruptVector,
                    ((*PortConfig).InitiatorBusId[0]) as i32,
                );

                if (*DeviceExtension).NonCachedExtension.is_null()
                    && (*PortConfig).SrbExtensionSize != (*DeviceExtension).SrbExtensionSize
                {
                    (*DeviceExtension).SrbExtensionSize =
                        ALIGN_UP((*PortConfig).SrbExtensionSize, 8 /* INT64 */);
                }

                if (*PortConfig).SpecificLuExtensionSize != (*DeviceExtension).LunExtensionSize {
                    (*DeviceExtension).LunExtensionSize = (*PortConfig).SpecificLuExtensionSize;
                }

                ResourceList = SpiConfigToResource(DeviceExtension, PortConfig);

                let mut LowerPDO: *mut DEVICE_OBJECT = core::ptr::null_mut();
                Status = IoReportDetectedDevice(
                    DriverObject,
                    (*HwInitializationData).AdapterInterfaceType,
                    ConfigInfo.BusNumber,
                    SlotNumber,
                    ResourceList,
                    core::ptr::null_mut(),
                    1,
                    &mut LowerPDO,
                );

                if Status < 0 {
                    DbgPrint(
                        (b"IoReportDetectedDevice failed. Status: 0x%x\n\0".as_ptr() as *const u8).cast(),
                        Status,
                    );
                    __debugbreak();
                    break 'outer;
                }

                (*DeviceExtension).Common.LowerDevice =
                    IoAttachDeviceToDeviceStack(PortDeviceObject, LowerPDO);

                if (*DeviceExtension).Common.LowerDevice.is_null() {
                    __debugbreak();
                }

                (*PortDeviceObject).Flags &= !DO_DEVICE_INITIALIZING;

                if !ResourceList.is_null() {
                    ExFreePoolWithTag(ResourceList as *mut c_void, TAG_SCSIPORT);
                }

                if (*PortConfig).MaximumNumberOfTargets > 0x80 /* SCSI_MAXIMUM_TARGETS_PER_BUS */ {
                    (*DeviceExtension).MaxTargetIds = 8;
                } else {
                    (*DeviceExtension).MaxTargetIds = ((*PortConfig).MaximumNumberOfTargets) as u32;
                }

                (*DeviceExtension).NumberOfBuses = (*PortConfig).NumberOfBuses;
                (*DeviceExtension).CachesData = (*PortConfig).CachesData;
                (*DeviceExtension).ReceiveEvent = (*PortConfig).ReceiveEvent;
                (*DeviceExtension).SupportsTaggedQueuing = (*PortConfig).TaggedQueuing;
                (*DeviceExtension).MultipleReqsPerLun = (*PortConfig).MultipleRequestPerLu;

                let BusConfigSize =
                    (*DeviceExtension).NumberOfBuses as usize * core::mem::size_of::<SCSI_BUS_INFO>();
                (*DeviceExtension).Buses = ExAllocatePoolZero(
                    NonPagedPool,
                    BusConfigSize as u32,
                    TAG_SCSIPORT,
                ) as *mut SCSI_BUS_INFO;
                if (*DeviceExtension).Buses.is_null() {
                    DbgPrint((b"Out of resources!\n\0".as_ptr() as *const u8).cast());
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    break 'outer;
                }

                for pathId in 0..(*DeviceExtension).NumberOfBuses {
                    let bus = (*DeviceExtension).Buses.add(pathId as usize);
                    (*bus).BusIdentifier = (*(*DeviceExtension).PortConfig).InitiatorBusId[pathId as usize];
                    InitializeListHead(&mut (*bus).LunsListHead);
                }

                if ConfigInfo.DisableMultipleLun != 0 {
                    (*DeviceExtension).MultipleReqsPerLun = 0;
                    (*PortConfig).MultipleRequestPerLu = 0;
                }
                if ConfigInfo.DisableTaggedQueueing != 0 {
                    (*DeviceExtension).SupportsTaggedQueuing = 0;
                    (*PortConfig).MultipleRequestPerLu = 0;
                }

                (*DeviceExtension).NeedSrbDataAlloc = if (*DeviceExtension).SupportsTaggedQueuing != 0
                    || (*DeviceExtension).MultipleReqsPerLun != 0
                {
                    1
                } else {
                    0
                };

                PortCapabilities = &mut (*DeviceExtension).PortCapabilities;
                (*DeviceExtension).MapBuffers = (*PortConfig).MapBuffers;
                (*PortCapabilities).AdapterUsesPio = (*PortConfig).MapBuffers;

                if (*DeviceExtension).AdapterObject.is_null()
                    && ((*PortConfig).DmaChannel != (SP_UNINITIALIZED_VALUE) as u32 || (*PortConfig).Master != 0)
                {
                    DbgPrint((b"DMA is not supported yet\n\0".as_ptr() as *const u8).cast());
                    if 1 != 1 {
                        __debugbreak();
                    }
                }

                if (*DeviceExtension).SrbExtensionBuffer.is_null()
                    && ((*DeviceExtension).SrbExtensionSize != 0 || (*PortConfig).AutoRequestSense != 0)
                {
                    (*DeviceExtension).SupportsAutoSense = (*PortConfig).AutoRequestSense;
                    (*DeviceExtension).NeedSrbExtensionAlloc = 1;
                    Status = SpiAllocateCommonBuffer(DeviceExtension, 0);
                    if Status < 0 {
                        break 'outer;
                    }
                }

                if (*DeviceExtension).NeedSrbDataAlloc != 0 {
                    let mut Count: u32;
                    let mut SrbData: *mut SCSI_REQUEST_BLOCK_INFO;

                    if (*DeviceExtension).SrbDataCount != 0 {
                        Count = (*DeviceExtension).SrbDataCount;
                    } else {
                        Count = (*DeviceExtension).RequestsNumber * 2;
                    }

                    SrbData = ExAllocatePoolWithTag(
                        NonPagedPool,
                        (Count as SIZE_T) * core::mem::size_of::<SCSI_REQUEST_BLOCK_INFO>() as SIZE_T,
                        TAG_SCSIPORT,
                    ) as *mut SCSI_REQUEST_BLOCK_INFO;
                    if SrbData.is_null() {
                        return (STATUS_INSUFFICIENT_RESOURCES) as u32;
                    }

                    core::ptr::write_bytes(
                        SrbData as *mut u8,
                        0,
                        (Count as usize) * core::mem::size_of::<SCSI_REQUEST_BLOCK_INFO>(),
                    );

                    (*DeviceExtension).SrbInfo = SrbData;
                    (*DeviceExtension).FreeSrbInfo = SrbData;
                    (*DeviceExtension).SrbDataCount = Count;

                    let mut local_count = Count;
                    let mut srb_ptr = SrbData;
                    while local_count > 0 {
                        (*srb_ptr).Requests.Flink =
                            (srb_ptr.add(1) as *mut c_void) as PLIST_ENTRY;
                        srb_ptr = srb_ptr.add(1);
                        local_count -= 1;
                    }
                    srb_ptr = srb_ptr.sub(1);
                    (*srb_ptr).Requests.Flink = core::ptr::null_mut();
                }

                PortCapabilities = &mut (*DeviceExtension).PortCapabilities;
                (*PortCapabilities).Length = core::mem::size_of::<IO_SCSI_CAPABILITIES>() as u32;
                (*PortCapabilities).MaximumTransferLength = (*PortConfig).MaximumTransferLength;

                if (*PortConfig).ReceiveEvent != 0 {
                    (*PortCapabilities).SupportedAsynchronousEvents |=
                        SRBEV_SCSI_ASYNC_NOTIFICATION;
                }

                (*PortCapabilities).TaggedQueuing = (*DeviceExtension).SupportsTaggedQueuing;
                (*PortCapabilities).AdapterScansDown = (*PortConfig).AdapterScansDown;

                if (*PortConfig).AlignmentMask > (*PortDeviceObject).AlignmentRequirement {
                    (*PortDeviceObject).AlignmentRequirement = (*PortConfig).AlignmentMask;
                }

                (*PortCapabilities).AlignmentMask = (*PortDeviceObject).AlignmentRequirement;

                if (*PortCapabilities).MaximumPhysicalPages == 0 {
                    (*PortCapabilities).MaximumPhysicalPages =
                        (((BYTES_TO_PAGES((((*PortCapabilities).MaximumTransferLength) as u32) as usize) as u32) as usize)) as u32;
                    if (*PortConfig).NumberOfPhysicalBreaks
                        < (*PortCapabilities).MaximumPhysicalPages
                    {
                        (*PortCapabilities).MaximumPhysicalPages =
                            (*PortConfig).NumberOfPhysicalBreaks;
                    }
                }

                FdoCallHWInitialize(DeviceExtension);
                Status = FdoStartAdapter(DeviceExtension);
                DbgPrint((b"[SCSIPORT] FdoStartAdapter st=%08x\n\0".as_ptr() as *const u8).cast(),
                    Status as u32); // ahci marker
                if Status < 0 {
                    DbgPrint(
                        (b"Failed to start the legacy adapter. Status 0x%x\n\0".as_ptr() as *const u8).cast(),
                        Status,
                    );
                    break 'outer;
                }
                FdoScanAdapter(DeviceExtension);
                DbgPrint((b"[SCSIPORT] FdoScanAdapter done\n\0".as_ptr() as *const u8).cast()); // ahci marker

                FirstConfigCall = 0;

                ConfigInfo.AdapterNumber += 1;
                if Again == 0 {
                    ConfigInfo.BusNumber += 1;
                }
                crate::glue::dprint_noop!((b"  Bus: %lu\n\0".as_ptr() as *const u8).cast(), ConfigInfo.BusNumber);
                DeviceFound = 1;
                break 'create_port_config;
            }
        }

        if Status < 0 && !PortDeviceObject.is_null() {
            FdoRemoveAdapter(DeviceExtension);
        }

        if !ConfigInfo.ServiceKey.is_null() {
            ZwClose(ConfigInfo.ServiceKey as *mut c_void);
        }
        if !ConfigInfo.DeviceKey.is_null() {
            ZwClose(ConfigInfo.DeviceKey as *mut c_void);
        }
        if !ConfigInfo.BusKey.is_null() {
            ZwClose(ConfigInfo.BusKey as *mut c_void);
        }
        if !ConfigInfo.AccessRanges.is_null() {
            ExFreePool(ConfigInfo.AccessRanges as *mut c_void);
        }
        if !ConfigInfo.Parameter.is_null() {
            ExFreePool(ConfigInfo.Parameter as *mut c_void);
        }

        crate::glue::dprint_noop!(
            (b"ScsiPortInitialize() done, Status = 0x%08X, DeviceFound = %d!\n\0".as_ptr() as *const u8).cast(),
            Status,
            DeviceFound as u32,
        );

        if DeviceFound == 0 { (Status) as u32 } else { (STATUS_SUCCESS) as u32 }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortIoMapTransfer(
    HwDeviceExtension: *mut core::ffi::c_void,
    Srb: *mut SCSI_REQUEST_BLOCK,
    LogicalAddress: *mut core::ffi::c_void,
    Length: u32,
) {
    DbgPrint(b"ScsiPortIoMapTransfer()\n\0".as_ptr() as *const i8);
    DbgPrint(b"UNIMPLEMENTED\n\0".as_ptr() as *const i8);
    DbgBreakPoint();
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortLogError(
    HwDeviceExtension: *mut core::ffi::c_void,
    Srb: *mut SCSI_REQUEST_BLOCK,
    PathId: u8,
    TargetId: u8,
    Lun: u8,
    ErrorCode: u32,
    UniqueId: u32,
) {
    // PSCSI_PORT_DEVICE_EXTENSION DeviceExtension;

    DbgPrint(
        b"ScsiPortLogError() called\n\0".as_ptr() as *const i8,
    );
    DbgPrint(
        b"PathId: 0x%02x  TargetId: 0x%02x  Lun: 0x%02x  ErrorCode: 0x%08lx  UniqueId: 0x%08lx\n\0"
            .as_ptr() as *const i8,
        PathId as u32,
        TargetId as u32,
        Lun as u32,
        ErrorCode,
        UniqueId,
    );

    // DeviceExtension = CONTAINING_RECORD(HwDeviceExtension, SCSI_PORT_DEVICE_EXTENSION, MiniPortDeviceExtension);

    crate::glue::dprint_noop!(
        b"ScsiPortLogError() done\n\0".as_ptr() as *const i8,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortMoveMemory(
    Destination: *mut core::ffi::c_void,
    Source: *const core::ffi::c_void,
    Length: u32,
) {
    core::ptr::copy(Source as *const u8, Destination as *mut u8, Length as usize);
}
/// Fixed-arity twin of the `ScsiPortNotification` C-variadic entry point in
/// `drivers/storage/port/scsiport/rust_varargs_shim.c` (see `ScsiDebugPrint_v`).
/// `ap` is a pointer to the shim frame's `va_list`; `VaArgs::arg::<T>()` mirrors
/// nightly's `VaListImpl::arg::<T>()`, so the dispatch below is byte-for-byte the
/// translation it was. All the logic stays here.
#[no_mangle]
pub unsafe extern "C" fn ScsiPortNotification_v(
    NotificationType: SCSI_NOTIFICATION_TYPE,
    HwDeviceExtension: *mut c_void,
    ap: *mut c_void,
) {

    // NDEBUG defined → DPRINT is empty; DPRINT1 stays
    // DPRINT("ScsiPortNotification() called\n");
    // DPRINT("DeviceExtension %p\n", DeviceExtension);

    let DeviceExtension = {
        let ptr = (HwDeviceExtension as *const u8).offset(
            -(core::mem::offset_of!(SCSI_PORT_DEVICE_EXTENSION, MiniPortDeviceExtension) as isize),
        ) as *mut SCSI_PORT_DEVICE_EXTENSION;
        ptr
    };

    // Real va_list walk (via the C shim's pullers) — the hand-rolled &param stack-walk
    // this replaced only worked on i386 cdecl; on win64 the params live in registers and
    // it read garbage (PathId=160 from a junk Srb).
    let mut args = VaArgs::from_ptr(ap);

    match NotificationType {
        RequestComplete => {
            let Srb: *mut SCSI_REQUEST_BLOCK = args.arg::<*mut SCSI_REQUEST_BLOCK>();

            // ASSERT(Srb->SrbStatus != SRB_STATUS_PENDING)
            if !((*Srb).SrbStatus != (SRB_STATUS_PENDING) as u8) {
                DbgBreakPoint();
            }
            // ASSERT(Srb->Function != SRB_FUNCTION_EXECUTE_SCSI ||
            //        Srb->SrbStatus != SRB_STATUS_SUCCESS ||
            //        Srb->ScsiStatus == SCSISTAT_GOOD)
            if !((*Srb).Function != (SRB_FUNCTION_EXECUTE_SCSI) as u8
                || (*Srb).SrbStatus != (SRB_STATUS_SUCCESS) as u8
                || (*Srb).ScsiStatus == (SCSISTAT_GOOD) as u8)
            {
                DbgBreakPoint();
            }

            // Check if already completed
            if ((*Srb).SrbFlags & SRB_FLAGS_IS_ACTIVE) == 0 {
                // va_end(ap) – nothing to do
                return;
            }

            // Clear active flag
            (*Srb).SrbFlags &= !SRB_FLAGS_IS_ACTIVE;

            if (*Srb).Function == (SRB_FUNCTION_ABORT_COMMAND) as u8 {
                // ASSERT(FALSE)
                if !(false) {
                    DbgBreakPoint();
                }
            } else {
                let IoStack = IoGetCurrentIrpStackLocation(((*Srb).OriginalRequest).cast());
                let LunExtension = (*(*IoStack).DeviceObject).DeviceExtension
                    as *mut SCSI_PORT_LUN_EXTENSION;

                // ASSERT(LunExtension && !LunExtension->Common.IsFDO)
                // `!IsFDO != 0` was bitwise-not and always true, so only the null half of
                // the assert had any effect.
                if !(!LunExtension.is_null() && (*LunExtension).Common.IsFDO == 0) {
                    DbgBreakPoint();
                }

                let SrbData = SpiGetSrbData(DeviceExtension, LunExtension, (((*Srb).QueueTag) as u32) as _);

                // ASSERT(SrbData->CompletedRequests == NULL && SrbData->Srb != NULL)
                if !((*SrbData).CompletedRequests.is_null() && !(*SrbData).Srb.is_null()) {
                    DbgBreakPoint();
                }

                if (*Srb).SrbStatus == (SRB_STATUS_SUCCESS) as u8
                    && ((*Srb).Cdb[0] == SCSIOP_READ || (*Srb).Cdb[0] == SCSIOP_WRITE)
                {
                    if (*Srb).DataTransferLength == 0 {
                        DbgBreakPoint();
                    }
                }

                (*SrbData).CompletedRequests = (*DeviceExtension).InterruptData.CompletedRequests;
                (*DeviceExtension).InterruptData.CompletedRequests = SrbData;
            }
        }

        NextRequest => {
            // DPRINT("Notify: NextRequest\n");
            (*DeviceExtension).InterruptData.Flags |= SCSI_PORT_NEXT_REQUEST_READY;
        }

        NextLuRequest => {
            // C: (UCHAR)va_arg(ap, int) ×3
            let PathId = args.arg::<i32>() as u8;
            let TargetId = args.arg::<i32>() as u8;
            let Lun = args.arg::<i32>() as u8;

            // DPRINT( "Notify: NextLuRequest(PathId %u  TargetId %u  Lun %u)\n", PathId, TargetId, Lun );

            (*DeviceExtension).InterruptData.Flags |= SCSI_PORT_NEXT_REQUEST_READY;

            let LunExtension = GetLunByPath(DeviceExtension, PathId, TargetId, Lun);

            if LunExtension.is_null() {
                // break from switch
            } else {
                if !(*LunExtension).ReadyLun.is_null() || !(*LunExtension).SrbInfo.Srb.is_null() {
                    // break
                } else {
                    (*LunExtension).ReadyLun = (*DeviceExtension).InterruptData.ReadyLun;
                    (*DeviceExtension).InterruptData.ReadyLun = LunExtension;
                }
            }
        }

        ResetDetected => {
            // DPRINT("Notify: ResetDetected\n");
            (*DeviceExtension).InterruptData.Flags |=
                SCSI_PORT_RESET | SCSI_PORT_RESET_REPORTED;
        }

        CallDisableInterrupts => {
            DbgPrint(
                b"UNIMPLEMENTED SCSI Notification called: CallDisableInterrupts!\n\0".as_ptr()
                    as *const i8,
            );
        }

        CallEnableInterrupts => {
            DbgPrint(
                b"UNIMPLEMENTED SCSI Notification called: CallEnableInterrupts!\n\0".as_ptr()
                    as *const i8,
            );
        }

        RequestTimerCall => {
            // DPRINT("Notify: RequestTimerCall\n");
            (*DeviceExtension).InterruptData.Flags |= SCSI_PORT_TIMER_NEEDED;

            // C: va_arg(ap, PHW_TIMER); va_arg(ap, ULONG)
            let timer_ptr = args.arg::<*mut c_void>();
            let miniport_timer_value = args.arg::<u32>();

            (*DeviceExtension).InterruptData.HwScsiTimer = core::mem::transmute(timer_ptr);
            (*DeviceExtension).InterruptData.MiniportTimerValue = miniport_timer_value;
        }

        BusChangeDetected => {
            // Unimplemented in ReactOS's scsiport too. A miniport that sees a
            // device arrive or leave (uniata on SATA hotplug) asks for a rescan:
            // flag it here, at interrupt level, and ScsiPortDpcForIsr turns it
            // into IoInvalidateDeviceRelations(BusRelations), whose handler
            // (FdoHandleDeviceRelations) re-runs FdoScanAdapter.
            (*DeviceExtension).InterruptData.Flags |= SCSI_PORT_BUS_CHANGE_DETECTED;
        }

        _ => {
            DbgPrint(
                b"Unsupported notification from WMI: %lu\n\0".as_ptr() as *const i8,
                NotificationType,
            );
        }
    }

    // va_end(ap) – nothing to do

    (*DeviceExtension).InterruptData.Flags |= SCSI_PORT_NOTIFICATION_NEEDED;
}

/// InterruptData.Flags bit for a pending BusChangeDetected (not a ReactOS
/// flag: the SCSI_PORT_* values stop at 0x10000).
pub const SCSI_PORT_BUS_CHANGE_DETECTED: u32 = 0x0002_0000;

#[no_mangle]
pub unsafe extern "system" fn ScsiPortValidateRange(
    HwDeviceExtension: *mut core::ffi::c_void,
    BusType: INTERFACE_TYPE,
    SystemIoBusNumber: u32,
    IoAddress: SCSI_PHYSICAL_ADDRESS,
    NumberOfBytes: u32,
    InIoSpace: BOOLEAN,
) -> BOOLEAN {
    crate::glue::dprint_noop!(b"ScsiPortValidateRange()\n\0".as_ptr() as *const i8);
    return 1;
}
unsafe fn SpiResourceToConfig(
    HwInitializationData: *mut HW_INITIALIZATION_DATA,
    ResourceDescriptor: *mut CM_FULL_RESOURCE_DESCRIPTOR,
    PortConfig: *mut PORT_CONFIGURATION_INFORMATION,
) {
    let mut AccessRange: *mut ACCESS_RANGE;
    let mut PartialData: *mut CM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut RangeNumber: u32 = 0;
    let mut Index: u32;
    let mut Interrupt: u32 = 0;
    let mut Dma: u32 = 0;

    let count = (*ResourceDescriptor).PartialResourceList.Count;
    for Index in 0..count {
        PartialData = ((*ResourceDescriptor).PartialResourceList.PartialDescriptors.as_mut_ptr()).add(Index as usize);

        match (*PartialData).Type {
            CmResourceTypePort => {
                if RangeNumber < (*HwInitializationData).NumberOfAccessRanges {
                    AccessRange = ((*PortConfig).AccessRanges as *mut ACCESS_RANGE).add(RangeNumber as usize);
                    (*AccessRange).RangeStart = (*PartialData).u.Port.Start;
                    (*AccessRange).RangeLength = (*PartialData).u.Port.Length;
                    (*AccessRange).RangeInMemory = 0; // FALSE
                    RangeNumber += 1;
                }
            }

            CmResourceTypeMemory => {
                if RangeNumber < (*HwInitializationData).NumberOfAccessRanges {
                    AccessRange = ((*PortConfig).AccessRanges as *mut ACCESS_RANGE).add(RangeNumber as usize);
                    (*AccessRange).RangeStart = (*PartialData).u.Memory.Start;
                    (*AccessRange).RangeLength = (*PartialData).u.Memory.Length;
                    (*AccessRange).RangeInMemory = 1; // TRUE
                    RangeNumber += 1;
                }
            }

            CmResourceTypeInterrupt => {
                if Interrupt == 0 {
                    (*PortConfig).BusInterruptLevel = (*PartialData).u.Interrupt.Level;
                    (*PortConfig).BusInterruptVector = (*PartialData).u.Interrupt.Vector;

                    if (*PartialData).Flags == (CM_RESOURCE_INTERRUPT_LATCHED) as u16 {
                        (*PortConfig).InterruptMode = Latched;
                    } else if (*PartialData).Flags == (CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE) as u16 {
                        (*PortConfig).InterruptMode = LevelSensitive;
                    }
                } else if Interrupt == 1 {
                    (*PortConfig).BusInterruptLevel2 = (*PartialData).u.Interrupt.Level;
                    (*PortConfig).BusInterruptVector2 = (*PartialData).u.Interrupt.Vector;

                    if (*PartialData).Flags == (CM_RESOURCE_INTERRUPT_LATCHED) as u16 {
                        (*PortConfig).InterruptMode2 = Latched;
                    } else if (*PartialData).Flags == (CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE) as u16 {
                        (*PortConfig).InterruptMode2 = LevelSensitive;
                    }
                }
                Interrupt += 1;
            }

            CmResourceTypeDma => {
                if Dma == 0 {
                    (*PortConfig).DmaChannel = (*PartialData).u.Dma.Channel;
                    (*PortConfig).DmaPort = (*PartialData).u.Dma.Port;

                    if (*PartialData).Flags & (CM_RESOURCE_DMA_8) as u16 != 0 {
                        (*PortConfig).DmaWidth = Width8Bits;
                    } else if (*PartialData).Flags & (CM_RESOURCE_DMA_16) as u16 != 0
                        || (*PartialData).Flags & (CM_RESOURCE_DMA_8_AND_16) as u16 != 0
                    {
                        (*PortConfig).DmaWidth = Width16Bits;
                    } else if (*PartialData).Flags & (CM_RESOURCE_DMA_32) as u16 != 0 {
                        (*PortConfig).DmaWidth = Width32Bits;
                    }
                } else if Dma == 1 {
                    (*PortConfig).DmaChannel2 = (*PartialData).u.Dma.Channel;
                    (*PortConfig).DmaPort2 = (*PartialData).u.Dma.Port;

                    if (*PartialData).Flags & (CM_RESOURCE_DMA_8) as u16 != 0 {
                        (*PortConfig).DmaWidth2 = Width8Bits;
                    } else if (*PartialData).Flags & (CM_RESOURCE_DMA_16) as u16 != 0
                        || (*PartialData).Flags & (CM_RESOURCE_DMA_8_AND_16) as u16 != 0
                    {
                        (*PortConfig).DmaWidth2 = Width16Bits;
                    } else if (*PartialData).Flags & (CM_RESOURCE_DMA_32) as u16 != 0 {
                        (*PortConfig).DmaWidth2 = Width32Bits;
                    }
                }
            }

            _ => {}
        }
    }
}

unsafe fn SpiConfigToResource(
    DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION,
    PortConfig: *mut PORT_CONFIGURATION_INFORMATION,
) -> *mut CM_RESOURCE_LIST {
    let ConfigInfo: *mut CONFIGURATION_INFORMATION;
    let mut ResourceList: *mut CM_RESOURCE_LIST;
    let mut ResourceDescriptor: *mut CM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut AccessRange: *mut ACCESS_RANGE;
    let mut ListLength: u32 = 0;
    let mut i: u32;
    let mut FullSize: u32;
    let mut Interrupt: u32;
    let mut Dma: u32;

    ConfigInfo = IoGetConfigurationInformation();

    if (*PortConfig).AtdiskPrimaryClaimed != 0 {
        (*ConfigInfo).AtDiskPrimaryAddressClaimed = 1; // TRUE
    }
    if (*PortConfig).AtdiskSecondaryClaimed != 0 {
        (*ConfigInfo).AtDiskSecondaryAddressClaimed = 1; // TRUE
    }

    if (*PortConfig).DmaChannel != (SP_UNINITIALIZED_VALUE) as u32 || (*PortConfig).DmaPort != (SP_UNINITIALIZED_VALUE) as u32 {
        Dma = 1;
        if (*PortConfig).DmaChannel2 != (SP_UNINITIALIZED_VALUE) as u32 || (*PortConfig).DmaPort2 != (SP_UNINITIALIZED_VALUE) as u32 {
            Dma += 1;
        }
    } else {
        Dma = 0;
    }
    ListLength += Dma;

    Interrupt = (*DeviceExtension).InterruptCount;
    ListLength += Interrupt;

    AccessRange = ((*PortConfig).AccessRanges).cast();
    for i in 0..(*PortConfig).NumberOfAccessRanges {
        if (*AccessRange).RangeLength != 0 {
            ListLength += 1;
        }
        AccessRange = AccessRange.offset(1);
    }

    FullSize = core::mem::size_of::<CM_RESOURCE_LIST>() as u32 + (ListLength.wrapping_sub(1)) * (core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as u32);

    ResourceList = ExAllocatePoolWithTag(PagedPool, FullSize as SIZE_T, TAG_SCSIPORT) as *mut CM_RESOURCE_LIST;

    if ResourceList.is_null() {
        return core::ptr::null_mut();
    }

    core::ptr::write_bytes(ResourceList as *mut u8, 0, FullSize as usize);

    (*ResourceList).Count = 1;
    (*ResourceList).List[0].InterfaceType = (*PortConfig).AdapterInterfaceType;
    (*ResourceList).List[0].BusNumber = (*PortConfig).SystemIoBusNumber;
    (*ResourceList).List[0].PartialResourceList.Count = ListLength;
    ResourceDescriptor = &raw mut (*ResourceList).List[0].PartialResourceList.PartialDescriptors[0];

    for i in 0..(*PortConfig).NumberOfAccessRanges {
        // .add() on the *mut [ACCESS_RANGE; 0] field is a ZERO-BYTE step (ZST array
        // element) — cast to *mut ACCESS_RANGE FIRST. The broken form always read
        // element[0], emitting a descriptor per range instead of per non-empty range
        // and overrunning the exactly-sized CM_RESOURCE_LIST pool block (BUGCHECK 0x19).
        AccessRange = ((*PortConfig).AccessRanges as *mut ACCESS_RANGE).add(i as usize);
        if (*AccessRange).RangeLength == 0 {
            continue;
        }
        if (*AccessRange).RangeInMemory != 0 {
            (*ResourceDescriptor).Type = CmResourceTypeMemory;
            (*ResourceDescriptor).Flags = (CM_RESOURCE_MEMORY_READ_WRITE) as u16;
        } else {
            (*ResourceDescriptor).Type = CmResourceTypePort;
            (*ResourceDescriptor).Flags = (CM_RESOURCE_PORT_IO) as u16;
        }
        (*ResourceDescriptor).ShareDisposition = (CmResourceShareDeviceExclusive) as u8;
        (*ResourceDescriptor).u.Memory.Start = (*AccessRange).RangeStart;
        (*ResourceDescriptor).u.Memory.Length = (*AccessRange).RangeLength;
        ResourceDescriptor = ResourceDescriptor.offset(1);
    }

    while Interrupt != 0 {
        (*ResourceDescriptor).Type = CmResourceTypeInterrupt;
        let mode = if Interrupt == 2 { (*PortConfig).InterruptMode2 } else { (*PortConfig).InterruptMode };
        if (*PortConfig).AdapterInterfaceType == MicroChannel || mode == LevelSensitive {
            (*ResourceDescriptor).ShareDisposition = (CmResourceShareShared) as u8;
            (*ResourceDescriptor).Flags = (CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE) as u16;
        } else {
            (*ResourceDescriptor).ShareDisposition = (CmResourceShareDeviceExclusive) as u8;
            (*ResourceDescriptor).Flags = (CM_RESOURCE_INTERRUPT_LATCHED) as u16;
        }
        (*ResourceDescriptor).u.Interrupt.Level = if Interrupt == 2 { (*PortConfig).BusInterruptLevel2 } else { (*PortConfig).BusInterruptLevel };
        (*ResourceDescriptor).u.Interrupt.Vector = if Interrupt == 2 { (*PortConfig).BusInterruptVector2 } else { (*PortConfig).BusInterruptVector };
        (*ResourceDescriptor).u.Interrupt.Affinity = 0;
        ResourceDescriptor = ResourceDescriptor.offset(1);
        Interrupt -= 1;
    }

    while Dma != 0 {
        (*ResourceDescriptor).Type = CmResourceTypeDma;
        (*ResourceDescriptor).ShareDisposition = (CmResourceShareDeviceExclusive) as u8;
        (*ResourceDescriptor).u.Dma.Channel = if Dma == 2 { (*PortConfig).DmaChannel2 } else { (*PortConfig).DmaChannel };
        (*ResourceDescriptor).u.Dma.Port = if Dma == 2 { (*PortConfig).DmaPort2 } else { (*PortConfig).DmaPort };
        (*ResourceDescriptor).Flags = 0;

        let width = if Dma == 2 { (*PortConfig).DmaWidth2 } else { (*PortConfig).DmaWidth };
        if width == Width8Bits {
            (*ResourceDescriptor).Flags |= (CM_RESOURCE_DMA_8) as u16;
        } else if width == Width16Bits {
            (*ResourceDescriptor).Flags |= (CM_RESOURCE_DMA_16) as u16;
        } else {
            (*ResourceDescriptor).Flags |= (CM_RESOURCE_DMA_32) as u16;
        }

        if (if Dma == 2 { (*PortConfig).DmaChannel2 } else { (*PortConfig).DmaChannel }) == (SP_UNINITIALIZED_VALUE) as u32 {
            (*ResourceDescriptor).u.Dma.Channel = 0;
        }
        if (if Dma == 2 { (*PortConfig).DmaPort2 } else { (*PortConfig).DmaPort }) == (SP_UNINITIALIZED_VALUE) as u32 {
            (*ResourceDescriptor).u.Dma.Port = 0;
        }

        ResourceDescriptor = ResourceDescriptor.offset(1);
        Dma -= 1;
    }

    return ResourceList;
}

unsafe fn SpiGetPciConfigData(
    DriverObject: *mut DRIVER_OBJECT,
    DeviceObject: *mut DEVICE_OBJECT,
    HwInitializationData: *mut HW_INITIALIZATION_DATA,
    PortConfig: *mut PORT_CONFIGURATION_INFORMATION,
    RegistryPath: *mut _UNICODE_STRING,
    BusNumber: u32,
    NextSlotNumber: *mut PCI_SLOT_NUMBER,
) -> i32 {
    const PCI_MAX_DEVICES: u32 = 32;
    const PCI_MAX_FUNCTION: u32 = 8;
    const PCI_INVALID_VENDORID: u16 = 0xFFFF;

    let mut PciConfig: crate::glue::PCI_COMMON_CONFIG_FIXED = core::mem::zeroed();
    let mut SlotNumber: PCI_SLOT_NUMBER = core::mem::zeroed();
    let mut DataSize: u32;
    let mut DeviceNumber: u32;
    let mut FunctionNumber: u32;
    let mut VendorIdString: [i8; 8] = [0; 8];
    let mut DeviceIdString: [i8; 8] = [0; 8];
    let mut UnicodeStr: _UNICODE_STRING = core::mem::zeroed();
    let mut ResourceList: *mut CM_RESOURCE_LIST = core::ptr::null_mut();
    let mut Status: i32;

    DbgPrint(b"SpiGetPciConfiguration() called\n\0".as_ptr() as *const i8);

    SlotNumber.u.AsULONG = 0;

    DeviceNumber = (*NextSlotNumber).u.bits.DeviceNumber();
    while DeviceNumber < PCI_MAX_DEVICES {
        SlotNumber.u.bits.set_DeviceNumber(DeviceNumber);

        FunctionNumber = (*NextSlotNumber).u.bits.FunctionNumber();
        while FunctionNumber < PCI_MAX_FUNCTION {
            SlotNumber.u.bits.set_FunctionNumber(FunctionNumber);

            DataSize = HalGetBusData(
                PCIConfiguration,
                BusNumber,
                SlotNumber.u.AsULONG,
                core::ptr::addr_of_mut!(PciConfig) as *mut core::ffi::c_void,
                core::mem::size_of::<u32>() as u32,
            );

            if DataSize == 0 {
                return 0;
            }

            if DataSize < core::mem::size_of::<u32>() as u32
                || PciConfig.VendorID == PCI_INVALID_VENDORID
            {
                FunctionNumber += 1;
                continue;
            }

            sprintf(
                VendorIdString.as_mut_ptr() as *mut i8,
                b"%04hx\0".as_ptr() as *const i8,
                PciConfig.VendorID as u32,
            );
            sprintf(
                DeviceIdString.as_mut_ptr() as *mut i8,
                b"%04hx\0".as_ptr() as *const i8,
                PciConfig.DeviceID as u32,
            );

            if _strnicmp(
                VendorIdString.as_ptr() as *const i8,
                (*HwInitializationData).VendorId as *const i8,
                (*HwInitializationData).VendorIdLength as usize,
            ) != 0
                || _strnicmp(
                    DeviceIdString.as_ptr() as *const i8,
                    (*HwInitializationData).DeviceId as *const i8,
                    (*HwInitializationData).DeviceIdLength as usize,
                ) != 0
            {
                FunctionNumber += 1;
                continue;
            }

            let wide_scsi_adapter: [u16; 12] = [
                0x0053, 0x0063, 0x0073, 0x0069, 0x0041, 0x0064, 0x0061, 0x0070, 0x0074, 0x0065,
                0x0072, 0x0000,
            ];

            crate::glue::dprint_noop!(
                b"Found device 0x%04hx 0x%04hx at %1lu %2lu %1lu\n\0".as_ptr() as *const i8,
                (PciConfig.VendorID) as i32,
                (PciConfig.DeviceID) as i32,
                BusNumber,
                SlotNumber.u.bits.DeviceNumber(),
                SlotNumber.u.bits.FunctionNumber(),
            );

            RtlInitUnicodeString(
                &mut UnicodeStr as *mut _UNICODE_STRING,
                wide_scsi_adapter.as_ptr(),
            );

            Status = HalAssignSlotResources(
                RegistryPath,
                &mut UnicodeStr,
                DriverObject,
                DeviceObject,
                PCIBus,
                BusNumber,
                SlotNumber.u.AsULONG,
                &mut ResourceList,
            );

            DbgPrint(b"[SCSIPORT] pci scan match %04x:%04x bus=%u slot=%u AssignSlotRes=%08x\n\0"
                .as_ptr() as *const i8,
                PciConfig.VendorID as u32, PciConfig.DeviceID as u32,
                BusNumber, DeviceNumber, Status as u32); // ahci marker

            if !NT_SUCCESS(Status) {
                break;
            }

            SpiResourceToConfig(
                HwInitializationData,
                core::ptr::addr_of_mut!((*ResourceList).List) as *mut CM_FULL_RESOURCE_DESCRIPTOR,
                PortConfig,
            );

            DbgPrint(b"[SCSIPORT] resources: irql=%u vec=%u ranges0=%08x\n\0"
                .as_ptr() as *const i8,
                (*PortConfig).BusInterruptLevel, (*PortConfig).BusInterruptVector,
                ScsiPortConvertPhysicalAddressToUlong(
                    (*((*PortConfig).AccessRanges as *mut ACCESS_RANGE)).RangeStart)); // ahci marker

            ExFreePool(ResourceList as *mut core::ffi::c_void);

            (*NextSlotNumber).u.bits.set_DeviceNumber(DeviceNumber);
            (*NextSlotNumber).u.bits.set_FunctionNumber(FunctionNumber + 1);
            (*PortConfig).SlotNumber = SlotNumber.u.AsULONG;
            return 1;
        }
        (*NextSlotNumber).u.bits.set_FunctionNumber(0);
        DeviceNumber += 1;
    }

    (*NextSlotNumber).u.bits.set_DeviceNumber(0);
    DbgPrint(b"No device found\n\0".as_ptr() as *const i8);
    return 0;
}

const IO_NO_INCREMENT: i32 = 0;
pub unsafe extern "system" fn ScsiPortCreateClose(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
) -> i32 {
    if InternalDebugLevel != 0 {
        crate::glue::dprint_noop!(b"ScsiPortCreateClose()\n\0".as_ptr() as *const i8);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, (IO_NO_INCREMENT) as i8);

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn SpiAdapterControl(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
    MapRegisterBase: PVOID,
    Context: PVOID,
) -> i32 {
    // Max SG list constant from headers
    const MAX_SG_LIST: u32 = 26;
    // SCSI request flags
    const SRB_FLAGS_DATA_OUT: u32 = 0x00000080;
    const SRB_FLAGS_SGLIST_FROM_POOL: u32 = 0x00040000;
    // IO_ALLOCATION_ACTION return value
    const DeallocateObjectKeepRegisters: i32 = 3;

    let srb: *mut SCSI_REQUEST_BLOCK;
    let mut scatter_gather_list: *mut SCSI_SG_ADDRESS;
    let mut current_irql: KIRQL = 0;
    let irp_stack: *mut IO_STACK_LOCATION;
    let mut total_length: u32 = 0;
    let srb_info: *mut SCSI_REQUEST_BLOCK_INFO;
    let device_extension: *mut SCSI_PORT_DEVICE_EXTENSION;
    let data_va: *mut u8;
    let write_to_device: u8;

    // Get pointers to SrbInfo and DeviceExtension
    srb_info = Context as *mut SCSI_REQUEST_BLOCK_INFO;
    device_extension = ((*DeviceObject).DeviceExtension).cast();

    // Get pointer to SRB
    irp_stack = IoGetCurrentIrpStackLocation(Irp);
    srb = (*irp_stack).Parameters.Others.Argument1 as *mut SCSI_REQUEST_BLOCK;

    // Depending on the map registers number, we allocate
    // either from NonPagedPool, or from our static list
    if (*srb_info).NumberOfMapRegisters > MAX_SG_LIST {
        (*srb_info).ScatterGather = ExAllocatePoolWithTag(
            NonPagedPool,
            ((*srb_info).NumberOfMapRegisters as SIZE_T)
                .wrapping_mul(core::mem::size_of::<SCSI_SG_ADDRESS>() as SIZE_T),
            // scsiport.h TAG_SCSIPORT ('ISCS'); 0 is PoolHitTag's default and breaks into
            // the pool tracker's DbgBreakPoint.
            crate::glue::TAG_SCSIPORT,
        ) as *mut SCSI_SG_ADDRESS;

        // ASSERT(FALSE) removed because NDEBUG is defined
        (*srb).SrbFlags |= SRB_FLAGS_SGLIST_FROM_POOL;
    } else {
        (*srb_info).ScatterGather = (*srb_info).ScatterGatherList.as_mut_ptr();
    }

    // Use chosen SG list source
    scatter_gather_list = (*srb_info).ScatterGather;

    // Save map registers base
    (*srb_info).BaseOfMapRegister = MapRegisterBase;

    // Determine WriteToDevice flag
    write_to_device = if (*srb).SrbFlags & SRB_FLAGS_DATA_OUT == SRB_FLAGS_DATA_OUT { 1 } else { 0 };

    // Get virtual address of the data buffer
    let base = MmGetMdlVirtualAddress((*Irp).MdlAddress) as *mut u8;
    let offset = ((*srb).DataBuffer as usize).wrapping_sub((*srb_info).DataOffset as usize);
    data_va = base.wrapping_add(offset);

    // Build the actual SG list
    while total_length < (*srb).DataTransferLength {
        if scatter_gather_list.is_null() {
            break;
        }

        (*scatter_gather_list).Length = (*srb).DataTransferLength - total_length;
        (*scatter_gather_list).PhysicalAddress = IoMapTransfer(
            (*device_extension).AdapterObject,
            (*Irp).MdlAddress,
            MapRegisterBase,
            data_va.add(total_length as usize) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*scatter_gather_list).Length),
            write_to_device,
        );

        total_length += (*scatter_gather_list).Length;
        scatter_gather_list = scatter_gather_list.add(1);
    }

    // Schedule an active request
    InterlockedIncrement(core::ptr::addr_of_mut!((*device_extension).ActiveRequestCounter) as *mut i32);
    KeAcquireSpinLock((core::ptr::addr_of_mut!((*device_extension).SpinLock)).cast(), core::ptr::addr_of_mut!(current_irql));
    KeSynchronizeExecution(
        (*device_extension).Interrupt[0],
        Some(core::mem::transmute(ScsiPortStartPacket as *const ())),
        DeviceObject as PVOID,
    );
    KeReleaseSpinLock((core::ptr::addr_of_mut!((*device_extension).SpinLock)).cast(), current_irql);

    return DeallocateObjectKeepRegisters;
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortIsr(
    Interrupt: PKINTERRUPT,
    ServiceContext: *mut core::ffi::c_void,
) -> i32 {
    let device_extension: *mut SCSI_PORT_DEVICE_EXTENSION =
        ServiceContext as *mut SCSI_PORT_DEVICE_EXTENSION;

    if InternalDebugLevel != 0 {
        DbgPrint(b"ScsiPortIsr() called!\n".as_ptr() as *const i8);
    }

    // Interrupt-storm detector (serial, exponential threshold prints only —
    // a raw per-entry print would BE the livelock).
    {
        static mut ISR_COUNT: u64 = 0;
        ISR_COUNT += 1;
        if ISR_COUNT == 1
            || ISR_COUNT == 1_000
            || ISR_COUNT == 100_000
            || ISR_COUNT == 10_000_000
        {
            DbgPrint(b"[SCSIPORT] Isr entry count=%I64u\n\0".as_ptr() as *const i8,
                ISR_COUNT); // storm marker
        }
    }

    WRITE_PORT_UCHAR(0xE9 as *mut u8, b'#');
    WRITE_PORT_UCHAR(0xE9 as *mut u8, b'I');
    WRITE_PORT_UCHAR(0xE9 as *mut u8, b'#');

    if (*device_extension).InterruptData.Flags & SCSI_PORT_DISABLE_INTERRUPTS != 0 {
        return 0; // FALSE
    }

    let hw_result = ((*device_extension).HwInterrupt.unwrap())(
        (*device_extension).MiniPortDeviceExtension.as_mut_ptr() as *mut c_void,
    );
    if hw_result == 0 {
        // false
        WRITE_PORT_UCHAR(0xE9 as *mut u8, b'#');
        WRITE_PORT_UCHAR(0xE9 as *mut u8, b'i');
        WRITE_PORT_UCHAR(0xE9 as *mut u8, b'#');
        return 0;
    }

    WRITE_PORT_UCHAR(0xE9 as *mut u8, b'#');
    WRITE_PORT_UCHAR(0xE9 as *mut u8, b'J');
    WRITE_PORT_UCHAR(0xE9 as *mut u8, b'#');

    if (*device_extension).InterruptData.Flags & SCSI_PORT_NOTIFICATION_NEEDED != 0 {
        IoRequestDpc(
            (*device_extension).Common.DeviceObject,
            (*device_extension).CurrentIrp,
            device_extension as *mut core::ffi::c_void,
        );
    }

    return 1; // TRUE
}

#[no_mangle]
pub unsafe extern "system" fn SpiProcessTimeout(
    ServiceContext: *mut core::ffi::c_void,
) -> i32 {
    let DeviceObject = ServiceContext as *mut DEVICE_OBJECT;
    let DeviceExtension = (*DeviceObject).DeviceExtension as PSCSI_PORT_DEVICE_EXTENSION;

    crate::glue::dprint_noop!(b"SpiProcessTimeout() entered\n\0".as_ptr() as *const i8);

    (*DeviceExtension).TimerCount = (core::u32::MAX) as i32; // -1

    if ((*DeviceExtension).InterruptData.Flags & SCSI_PORT_RESET) != 0 {
        (*DeviceExtension).InterruptData.Flags &= !SCSI_PORT_RESET;

        if ((*DeviceExtension).InterruptData.Flags & SCSI_PORT_RESET_REQUEST) != 0 {
            (*DeviceExtension).InterruptData.Flags &= !SCSI_PORT_RESET_REQUEST;
            ScsiPortStartPacket(ServiceContext);
        }

        return 0; // FALSE
    } else {
        crate::glue::dprint_noop!(b"Resetting the bus\n\0".as_ptr() as *const i8);

        for Bus in 0..(*DeviceExtension).NumberOfBuses {
            ((*DeviceExtension).HwResetBus.unwrap())((*DeviceExtension).MiniPortDeviceExtension.as_mut_ptr() as *mut c_void, Bus as u32);

            // Reset flags and set reset timeout to 4 seconds
            (*DeviceExtension).InterruptData.Flags |= SCSI_PORT_RESET;
            (*DeviceExtension).TimerCount = 4;
        }

        // If miniport requested - request a dpc for it
        if ((*DeviceExtension).InterruptData.Flags & SCSI_PORT_NOTIFICATION_NEEDED) != 0 {
            IoRequestDpc(
                (*DeviceExtension).Common.DeviceObject,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }
    }

    1 // TRUE
}

#[no_mangle]
pub unsafe extern "system" fn SpiResetBus(ServiceContext: *mut core::ffi::c_void) -> u8 {
    let ResetParams = ServiceContext as *mut RESETBUS_PARAMS;
    let DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION;

    DeviceExtension = (*ResetParams).DeviceExtension as *mut SCSI_PORT_DEVICE_EXTENSION;
    let hw_reset_bus: unsafe extern "system" fn(*mut core::ffi::c_void, u32) =
        core::mem::transmute((*DeviceExtension).HwResetBus);
    hw_reset_bus((*DeviceExtension).MiniPortDeviceExtension.as_mut_ptr() as *mut c_void, (*ResetParams).PathId);

    (*DeviceExtension).InterruptData.Flags |= SCSI_PORT_RESET;
    (*DeviceExtension).TimerCount = 4;

    if ((*DeviceExtension).InterruptData.Flags & SCSI_PORT_NOTIFICATION_NEEDED) != 0 {
        IoRequestDpc((*DeviceExtension).Common.DeviceObject, core::ptr::null_mut(), core::ptr::null_mut());
    }

    return 1;
}
// IO_TIMER_ROUTINE / DRIVER_DISPATCH are NTAPI (stdcall on i386). These two were
// plain Rust-ABI fns whose addresses went to the kernel — the timer dispatch then
// smashed the stack on i386 (PC jumped into the pool DeviceExtension, BUGCHECK 0x1E).
pub unsafe extern "system" fn ScsiPortIoTimer(
    DeviceObject: *mut DEVICE_OBJECT,
    Context: *mut core::ffi::c_void,
) {
    // DPRINT("ScsiPortIoTimer()\n");
    crate::glue::dprint_noop!(b"ScsiPortIoTimer()\n\0".as_ptr() as *const i8);

    let DeviceExtension = (*DeviceObject).DeviceExtension as *mut SCSI_PORT_DEVICE_EXTENSION;

    // Protect with the spinlock
    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());

    // Check timeouts
    if (*DeviceExtension).TimerCount > 0 {
        // Decrease the timeout counter
        (*DeviceExtension).TimerCount -= 1;

        if (*DeviceExtension).TimerCount == 0 {
            // Timeout, process it
            if KeSynchronizeExecution(
                (*DeviceExtension).Interrupt[0],
                Some(core::mem::transmute(SpiProcessTimeout as *const ())),
                (*DeviceExtension).Common.DeviceObject as *mut core::ffi::c_void,
            ) != 0
            {
                crate::glue::dprint_noop!(b"Error happened during processing timeout, but nothing critical\n\0".as_ptr() as *const i8);
            }
        }

        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());

        // We should exit now, since timeout is processed
        return;
    }

    // Per-Lun scanning of timeouts is needed...
    for pathId in 0u8..(*DeviceExtension).NumberOfBuses {
        let bus: *mut SCSI_BUS_INFO = (*DeviceExtension).Buses.add(pathId as usize);

        let mut lunEntry = (*bus).LunsListHead.Flink;
        while lunEntry != core::ptr::addr_of_mut!((*bus).LunsListHead) {
            let LunExtension = (lunEntry as *const u8)
                .offset(-(core::mem::offset_of!(SCSI_PORT_LUN_EXTENSION, LunEntry) as isize))
                as *mut SCSI_PORT_LUN_EXTENSION;

            if ((*LunExtension).Flags & 0x8 /* LUNEX_BUSY */) != 0 {
                if ((*LunExtension).Flags & (0x4 | 0x1) /* LUNEX_NEED_REQUEST_SENSE | LUNEX_FROZEN_QUEUE */) == 0 {
                    crate::glue::dprint_noop!(b"Retrying busy request\n\0".as_ptr() as *const i8);

                    // Clear flags, and retry busy request
                    (*LunExtension).Flags &= !(0x8 | 0x10 /* LUNEX_BUSY | LUNEX_FULL_QUEUE */);
                    let Irp = (*LunExtension).BusyRequest;

                    // Clearing busy request
                    (*LunExtension).BusyRequest = core::ptr::null_mut();

                    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());

                    IoStartPacket(DeviceObject, Irp, core::ptr::null_mut::<u32>(), None);

                    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());
                }
            } else if (*LunExtension).RequestTimeout == 0 {
                let mut ResetParams: RESETBUS_PARAMS = core::mem::zeroed();

                (*LunExtension).RequestTimeout = -1i32;

                crate::glue::dprint_noop!(b"Request timed out, resetting bus\n\0".as_ptr() as *const i8);

                // Pass params to the bus reset routine
                ResetParams.PathId = (*LunExtension).PathId as u32;
                ResetParams.DeviceExtension = DeviceExtension;

                if KeSynchronizeExecution(
                    (*DeviceExtension).Interrupt[0],
                    Some(core::mem::transmute(SpiResetBus as *const ())),
                    core::ptr::addr_of_mut!(ResetParams) as *mut core::ffi::c_void,
                ) == 0
                {
                    DbgPrint(b"Reset failed\n\0".as_ptr() as *const i8);
                }
            } else if (*LunExtension).RequestTimeout > 0 {
                // Decrement the timeout counter
                (*LunExtension).RequestTimeout -= 1;
            }

            lunEntry = (*lunEntry).Flink;
        }
    }

    // Release the spinlock
    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*DeviceExtension).SpinLock)).cast());
}

#[no_mangle]
pub unsafe extern "system" fn SpiMiniportTimerDpc(
    Dpc: *mut _KDPC,
    DeviceObject: *mut core::ffi::c_void,
    SystemArgument1: *mut core::ffi::c_void,
    SystemArgument2: *mut core::ffi::c_void,
) {


    crate::glue::dprint_noop!(b"Miniport timer DPC\n\0".as_ptr() as *const i8);

    let device_object = DeviceObject as *mut DEVICE_OBJECT;
    let device_extension: *mut SCSI_PORT_DEVICE_EXTENSION =
        (*device_object).DeviceExtension as *mut SCSI_PORT_DEVICE_EXTENSION;

    KeAcquireSpinLockAtDpcLevel((&raw mut (*device_extension).SpinLock).cast());

    if !(*device_extension).HwScsiTimer.is_none() {
        ((*device_extension).HwScsiTimer.unwrap())((*device_extension).MiniPortDeviceExtension.as_mut_ptr() as *mut c_void);
    }

    KeReleaseSpinLockFromDpcLevel((&raw mut (*device_extension).SpinLock).cast());

    if ((*device_extension).InterruptData.Flags & SCSI_PORT_NOTIFICATION_NEEDED) != 0 {
        ScsiPortDpcForIsr(
            core::ptr::null_mut(),
            (*device_extension).Common.DeviceObject,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }
}

unsafe fn SpiCreatePortConfig(
    DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION,
    HwInitData: *mut HW_INITIALIZATION_DATA,
    InternalConfigInfo: *mut CONFIGURATION_INFO,
    ConfigInfo: *mut PORT_CONFIGURATION_INFORMATION,
    ZeroStruct: u8,
) -> NTSTATUS {
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DdkConfigInformation: *mut CONFIGURATION_INFORMATION;
    let mut RootKey: HANDLE = core::ptr::null_mut();
    let mut Key: HANDLE = core::ptr::null_mut();
    let mut Found: u8;
    let mut DeviceBuffer: [u16; 16] = [0; 16];
    let mut StrBuffer: [u16; 512] = [0; 512];
    let mut Bus: u32;
    let mut Status: NTSTATUS;

    if ZeroStruct != 0 {
        core::ptr::write_bytes(
            ConfigInfo as *mut u8,
            0,
            core::mem::size_of::<PORT_CONFIGURATION_INFORMATION>(),
        );
        core::ptr::write_bytes(
            (*InternalConfigInfo).AccessRanges as *mut u8,
            0,
            ((*HwInitData).NumberOfAccessRanges as usize)
                * core::mem::size_of::<ACCESS_RANGE>(),
        );

        (*ConfigInfo).Length = core::mem::size_of::<PORT_CONFIGURATION_INFORMATION>() as u32;
        (*ConfigInfo).AdapterInterfaceType = (*HwInitData).AdapterInterfaceType;
        (*ConfigInfo).InterruptMode = Latched;
        (*ConfigInfo).DmaChannel = (SP_UNINITIALIZED_VALUE) as u32;
        (*ConfigInfo).DmaPort = (SP_UNINITIALIZED_VALUE) as u32;
        (*ConfigInfo).DmaChannel2 = (SP_UNINITIALIZED_VALUE) as u32;
        (*ConfigInfo).DmaPort2 = (SP_UNINITIALIZED_VALUE) as u32;
        (*ConfigInfo).MaximumTransferLength = (SP_UNINITIALIZED_VALUE) as u32;
        (*ConfigInfo).NumberOfAccessRanges = (*HwInitData).NumberOfAccessRanges;
        (*ConfigInfo).MaximumNumberOfTargets = (SCSI_MAXIMUM_TARGETS) as u8;

        (*ConfigInfo).NeedPhysicalAddresses = (*HwInitData).NeedPhysicalAddresses;
        (*ConfigInfo).MapBuffers = (*HwInitData).MapBuffers;
        (*ConfigInfo).AutoRequestSense = (*HwInitData).AutoRequestSense;
        (*ConfigInfo).ReceiveEvent = (*HwInitData).ReceiveEvent;
        (*ConfigInfo).TaggedQueuing = (*HwInitData).TaggedQueuing;
        (*ConfigInfo).MultipleRequestPerLu = (*HwInitData).MultipleRequestPerLu;

        DdkConfigInformation = IoGetConfigurationInformation();
        (*ConfigInfo).AtdiskPrimaryClaimed =
            (*DdkConfigInformation).AtDiskPrimaryAddressClaimed;
        (*ConfigInfo).AtdiskSecondaryClaimed =
            (*DdkConfigInformation).AtDiskSecondaryAddressClaimed;

        let initiator_len = core::mem::size_of_val(&(*ConfigInfo).InitiatorBusId)
            / core::mem::size_of_val(&(*ConfigInfo).InitiatorBusId[0]);
        for Bus in 0..initiator_len {
            (*ConfigInfo).InitiatorBusId[Bus] = (SP_UNINITIALIZED_VALUE as i8) as u8;
        }
    }

    (*ConfigInfo).NumberOfPhysicalBreaks = 17;

    (*InternalConfigInfo).DisableTaggedQueueing = 0;
    (*InternalConfigInfo).DisableMultipleLun = 0;

    (*ConfigInfo).SystemIoBusNumber = (*InternalConfigInfo).BusNumber;

    // PortConfig is reallocated fresh on every find-loop iteration, but the init
    // block above only runs when ZeroStruct (FirstConfigCall) is TRUE — i.e. only
    // before the first device is found. On later iterations AdapterInterfaceType is
    // left as uninitialized pool garbage; when it happens to read 0 (== Internal),
    // the Internal hardware-database branch below wrongly resets BusNumber to 0, so
    // the legacy find loop (uniata AtapiFindIsaController) never advances past bus 0
    // and never terminates -> BUGCHECK 0x50 (flaky; deterministic once the winsock
    // catalog perturbs the pool). Always seed it from HwInitData so the interface
    // check is correct regardless of ZeroStruct.
    (*ConfigInfo).AdapterInterfaceType = (*HwInitData).AdapterInterfaceType;

    'try_next_ad: loop {
        if (*ConfigInfo).AdapterInterfaceType == Internal {
            ObjectAttributes = core::mem::zeroed::<_OBJECT_ATTRIBUTES>();
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
            ObjectAttributes.ObjectName = ((*(*(*DeviceExtension).Common.DeviceObject).DriverObject).HardwareDatabase);
            ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = ZwOpenKey(
                core::ptr::addr_of_mut!(RootKey),
                KEY_READ,
                &mut ObjectAttributes as *mut _OBJECT_ATTRIBUTES,
            );

            if NT_SUCCESS(Status) {
                let scsi_adapter_fmt: &[u16] = &[
                    0x53, 0x63, 0x73, 0x69, 0x41, 0x64, 0x61, 0x70, 0x74, 0x65, 0x72, 0x5c,
                    0x25, 0x6c, 0x75, 0x00,
                ];
                swprintf(
                    StrBuffer.as_mut_ptr(),
                    scsi_adapter_fmt.as_ptr(),
                    (*InternalConfigInfo).AdapterNumber,
                );

                UnicodeString.Length = 0;
                UnicodeString.MaximumLength = 0;
                UnicodeString.Buffer = StrBuffer.as_mut_ptr() as PWSTR;

                ObjectAttributes = core::mem::zeroed::<_OBJECT_ATTRIBUTES>();
                ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
                ObjectAttributes.ObjectName = &mut UnicodeString as *mut _UNICODE_STRING;
                ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
                ObjectAttributes.RootDirectory = RootKey;
                ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                Status = ZwOpenKey(
                    core::ptr::addr_of_mut!(Key),
                    KEY_READ,
                    &mut ObjectAttributes as *mut _OBJECT_ATTRIBUTES,
                );

                ZwClose(RootKey);

                if NT_SUCCESS(Status) {
                    if (*InternalConfigInfo).LastAdapterNumber
                        != (*InternalConfigInfo).AdapterNumber
                    {
                        crate::glue::dprint_noop!(
                            b"Hardware info found at %S\n\0".as_ptr() as *const i8,
                            StrBuffer.as_mut_ptr(),
                        );

                        SpiParseDeviceInfo(
                            DeviceExtension,
                            Key,
                            ConfigInfo,
                            InternalConfigInfo,
                            StrBuffer.as_mut_ptr() as *mut u8,
                        );

                        (*InternalConfigInfo).BusNumber = 0;
                    } else {
                        (*InternalConfigInfo).AdapterNumber += 1;
                        continue 'try_next_ad;
                    }
                } else {
                    crate::glue::dprint_noop!(
                        b"ZwOpenKey() failed with Status=0x%08X\n\0".as_ptr() as *const i8,
                        Status,
                    );
                    return STATUS_DEVICE_DOES_NOT_EXIST;
                }
            } else {
                crate::glue::dprint_noop!(
                    b"ZwOpenKey() failed with Status=0x%08X\n\0".as_ptr() as *const i8,
                    Status,
                );
            }
        }

        Key = core::ptr::null_mut();
        if !(*InternalConfigInfo).Parameter.is_null() {
            ExFreePool((*InternalConfigInfo).Parameter);
            (*InternalConfigInfo).Parameter = core::ptr::null_mut();
        }

        if !(*InternalConfigInfo).ServiceKey.is_null() {
            let device_fmt: &[u16] = &[
                0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x25, 0x6c, 0x75, 0x00,
            ];
            swprintf(
                DeviceBuffer.as_mut_ptr(),
                device_fmt.as_ptr(),
                (*InternalConfigInfo).AdapterNumber,
            );

            UnicodeString.Length = 0;
            UnicodeString.MaximumLength = 0;
            UnicodeString.Buffer = DeviceBuffer.as_mut_ptr() as PWSTR;

            ObjectAttributes = core::mem::zeroed::<_OBJECT_ATTRIBUTES>();
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
            ObjectAttributes.ObjectName = &mut UnicodeString as *mut _UNICODE_STRING;
            ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
            ObjectAttributes.RootDirectory = (*InternalConfigInfo).ServiceKey;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = ZwOpenKey(
                core::ptr::addr_of_mut!(Key),
                KEY_READ,
                &mut ObjectAttributes as *mut _OBJECT_ATTRIBUTES,
            );
        }

        if !(*InternalConfigInfo).DeviceKey.is_null() {
            SpiParseDeviceInfo(
                DeviceExtension,
                (*InternalConfigInfo).DeviceKey,
                ConfigInfo,
                InternalConfigInfo,
                StrBuffer.as_mut_ptr() as *mut u8,
            );
        }

        if !Key.is_null() {
            if (*InternalConfigInfo).LastAdapterNumber
                != (*InternalConfigInfo).AdapterNumber
            {
                SpiParseDeviceInfo(
                    DeviceExtension,
                    Key,
                    ConfigInfo,
                    InternalConfigInfo,
                    StrBuffer.as_mut_ptr() as *mut u8,
                );

                ZwClose(Key);
            } else {
                (*InternalConfigInfo).AdapterNumber += 1;

                ZwClose(Key);

                continue 'try_next_ad;
            }
        }

        (*InternalConfigInfo).LastAdapterNumber = (*InternalConfigInfo).AdapterNumber;

        Found = 0;
        Status = ((((IoQueryDeviceDescription))))(
            core::ptr::addr_of_mut!((*HwInitData).AdapterInterfaceType),
            core::ptr::addr_of_mut!((*InternalConfigInfo).BusNumber),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            Some(core::mem::transmute(SpQueryDeviceCallout as *const ())),
            core::ptr::addr_of_mut!(Found) as *mut c_void,
        );

        if Found == 0 {
            let mut InterfaceType: INTERFACE_TYPE = Eisa;

            if (*HwInitData).AdapterInterfaceType == (crate::glue::Isa) as i32 {
                Status = ((((IoQueryDeviceDescription))))(
                    core::ptr::addr_of_mut!(InterfaceType),
                    core::ptr::addr_of_mut!((*InternalConfigInfo).BusNumber),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    Some(core::mem::transmute(SpQueryDeviceCallout as *const ())),
                    core::ptr::addr_of_mut!(Found) as *mut c_void,
                );

                if Found != 0 {
                    return STATUS_SUCCESS;
                } else {
                    return STATUS_DEVICE_DOES_NOT_EXIST;
                }
            } else {
                return STATUS_DEVICE_DOES_NOT_EXIST;
            }
        } else {
            return STATUS_SUCCESS;
        }

        // The loop only reaches this point via continue; otherwise returns above.
        // This break is unreachable, but we need to break out if for some reason control falls through.
        break;
    }

    // This point is unreachable because the loop always returns.
    // Return STATUS_DEVICE_DOES_NOT_EXIST just in case.
    STATUS_DEVICE_DOES_NOT_EXIST
}

// scsiport.c:2634-3014. Walks every value under the device node's registry key and
// applies the ones scsiport understands: eight scalar tuning knobs, DriverParameters,
// and the REG_FULL_RESOURCE_DESCRIPTOR blob whose partial descriptors carry the port /
// memory / interrupt / DMA / device-specific resources.
//
// The C's `if`s are INDEPENDENT, not an else-chain, and several of them `continue` —
// so a value whose name matches one test still falls through the remaining tests
// unless that test skipped the iteration. Kept literally.
unsafe fn SpiParseDeviceInfo(
    DeviceExtension: *mut SCSI_PORT_DEVICE_EXTENSION,
    Key: HANDLE,
    ConfigInfo: *mut PORT_CONFIGURATION_INFORMATION,
    InternalConfigInfo: *mut CONFIGURATION_INFO,
    Buffer: *mut u8,
) {
    let KeyValueInformation: *mut KEY_VALUE_FULL_INFORMATION =
        Buffer as *mut KEY_VALUE_FULL_INFORMATION;
    let mut Length: u32 = 0;
    let mut Dma: u32 = 0;
    let mut Interrupt: u32 = 0;
    let mut Index: u32 = 0;
    let mut RangeCount: u32 = 0;
    let mut Status: NTSTATUS;

    // Loop through all values in the device node
    loop {
        Status = ZwEnumerateValueKey(
            Key,
            Index,
            KeyValueFullInformation,
            (Buffer).cast(),
            512,
            core::ptr::addr_of_mut!(Length),
        );

        if !NT_SUCCESS(Status) {
            return;
        }

        Index += 1;

        // Length for DWORD is ok?
        if (*KeyValueInformation).Type == REG_DWORD
            && (*KeyValueInformation).DataLength != core::mem::size_of::<u32>() as u32
        {
            continue;
        }

        // `Name` is the trailing flexible array; NameLength is BYTES, so /2 is the
        // character count _wcsnicmp is asked to compare.
        let name = (*KeyValueInformation).Name.as_ptr();
        let namechars = ((*KeyValueInformation).NameLength / 2) as usize;
        let data = Buffer.add((*KeyValueInformation).DataOffset as usize);

        // Get MaximumLogicalUnit
        if _wcsnicmp(name, crate::wide!("MaximumLogicalUnit").as_ptr(), namechars) == 0 {
            if (*KeyValueInformation).Type != REG_DWORD {
                crate::glue::dprint_noop!(
                    b"Bad data type for MaximumLogicalUnit\n\0".as_ptr() as *const i8
                );
                continue;
            }

            // The C reads ONE byte out of the DWORD, not the DWORD. Kept.
            (*DeviceExtension).MaxLunCount = *data as ULONG;

            // Check / reset if needed
            if (*DeviceExtension).MaxLunCount > SCSI_MAXIMUM_LOGICAL_UNITS {
                (*DeviceExtension).MaxLunCount = SCSI_MAXIMUM_LOGICAL_UNITS;
            }

            crate::glue::dprint_noop!(
                b"MaximumLogicalUnit = %d\n\0".as_ptr() as *const i8,
                (*DeviceExtension).MaxLunCount
            );
        }

        // Get InitiatorTargetId
        if _wcsnicmp(name, crate::wide!("InitiatorTargetId").as_ptr(), namechars) == 0 {
            if (*KeyValueInformation).Type != REG_DWORD {
                crate::glue::dprint_noop!(
                    b"Bad data type for InitiatorTargetId\n\0".as_ptr() as *const i8
                );
                continue;
            }

            (*ConfigInfo).InitiatorBusId[0] = *data;

            // Check / reset if needed. MaximumNumberOfTargets is UCHAR and the C does
            // `- 1` in int arithmetic, so a value of 0 yields -1 and the test is always
            // true — reproduced with an explicit i32 widening rather than a u8 wrap.
            if (*ConfigInfo).InitiatorBusId[0] as i32
                > (*ConfigInfo).MaximumNumberOfTargets as i32 - 1
            {
                // (CCHAR)SP_UNINITIALIZED_VALUE — 0xFFFFFFFF narrowed to signed char
                // is -1, stored back into the UCHAR slot as 0xFF.
                (*ConfigInfo).InitiatorBusId[0] = SP_UNINITIALIZED_VALUE as i8 as u8;
            }

            crate::glue::dprint_noop!(
                b"InitiatorTargetId = %d\n\0".as_ptr() as *const i8,
                (*ConfigInfo).InitiatorBusId[0]
            );
        }

        // Get ScsiDebug
        if _wcsnicmp(name, crate::wide!("ScsiDebug").as_ptr(), namechars) == 0 {
            crate::glue::dprint_noop!(b"ScsiDebug key not supported\n\0".as_ptr() as *const i8);
        }

        // Check for a breakpoint
        if _wcsnicmp(name, crate::wide!("BreakPointOnEntry").as_ptr(), namechars) == 0 {
            crate::glue::dprint_noop!(b"Breakpoint on entry requested!\n\0".as_ptr() as *const i8);
            DbgBreakPoint();
        }

        // Get DisableSynchronousTransfers
        if _wcsnicmp(name, crate::wide!("DisableSynchronousTransfers").as_ptr(), namechars) == 0 {
            (*DeviceExtension).SrbFlags |= SRB_FLAGS_DISABLE_SYNCH_TRANSFER;
            crate::glue::dprint_noop!(b"Synch transfers disabled\n\0".as_ptr() as *const i8);
        }

        // Get DisableDisconnects
        if _wcsnicmp(name, crate::wide!("DisableDisconnects").as_ptr(), namechars) == 0 {
            (*DeviceExtension).SrbFlags |= SRB_FLAGS_DISABLE_DISCONNECT;
            crate::glue::dprint_noop!(b"Disconnects disabled\n\0".as_ptr() as *const i8);
        }

        // Get DisableTaggedQueuing
        if _wcsnicmp(name, crate::wide!("DisableTaggedQueuing").as_ptr(), namechars) == 0 {
            (*InternalConfigInfo).DisableTaggedQueueing = TRUE as BOOLEAN;
            crate::glue::dprint_noop!(b"Tagged queueing disabled\n\0".as_ptr() as *const i8);
        }

        // Get DisableMultipleRequests
        if _wcsnicmp(name, crate::wide!("DisableMultipleRequests").as_ptr(), namechars) == 0 {
            (*InternalConfigInfo).DisableMultipleLun = TRUE as BOOLEAN;
            crate::glue::dprint_noop!(b"Multiple requests disabled\n\0".as_ptr() as *const i8);
        }

        // Get DriverParameters
        if _wcsnicmp(name, crate::wide!("DriverParameters").as_ptr(), namechars) == 0 {
            // Skip if nothing
            if (*KeyValueInformation).DataLength == 0 {
                continue;
            }

            // If there was something previously allocated - free it
            if !(*InternalConfigInfo).Parameter.is_null() {
                ExFreePool((*InternalConfigInfo).Parameter);
            }

            // Allocate it
            (*InternalConfigInfo).Parameter = ExAllocatePoolWithTag(
                NonPagedPool,
                (*KeyValueInformation).DataLength as SIZE_T,
                TAG_SCSIPORT,
            );

            if !(*InternalConfigInfo).Parameter.is_null() {
                if (*KeyValueInformation).Type != REG_SZ {
                    // Just copy
                    core::ptr::copy_nonoverlapping(
                        (KeyValueInformation as *const u8)
                            .add((*KeyValueInformation).DataOffset as usize),
                        (*InternalConfigInfo).Parameter as *mut u8,
                        (*KeyValueInformation).DataLength as usize,
                    );
                } else {
                    // If it's a unicode string, convert it to ansi
                    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
                    let mut AnsiString: ANSI_STRING = core::mem::zeroed();

                    UnicodeString.Length = (*KeyValueInformation).DataLength as USHORT;
                    UnicodeString.MaximumLength = (*KeyValueInformation).DataLength as USHORT;
                    UnicodeString.Buffer = (KeyValueInformation as *mut u8)
                        .add((*KeyValueInformation).DataOffset as usize)
                        as PWSTR;

                    AnsiString.Length = 0;
                    AnsiString.MaximumLength = (*KeyValueInformation).DataLength as USHORT;
                    AnsiString.Buffer = (*InternalConfigInfo).Parameter as PCHAR;

                    Status = RtlUnicodeStringToAnsiString(
                        core::ptr::addr_of_mut!(AnsiString),
                        core::ptr::addr_of_mut!(UnicodeString),
                        FALSE as BOOLEAN,
                    );

                    // In case of error, free the allocated space
                    if !NT_SUCCESS(Status) {
                        ExFreePool((*InternalConfigInfo).Parameter);
                        (*InternalConfigInfo).Parameter = core::ptr::null_mut();
                    }
                }
            }

            crate::glue::dprint_noop!(b"Found driver parameter\n\0".as_ptr() as *const i8);
        }

        // Get MaximumSGList
        if _wcsnicmp(name, crate::wide!("MaximumSGList").as_ptr(), namechars) == 0 {
            if (*KeyValueInformation).Type != REG_DWORD {
                crate::glue::dprint_noop!(
                    b"Bad data type for MaximumSGList\n\0".as_ptr() as *const i8
                );
                continue;
            }

            (*ConfigInfo).NumberOfPhysicalBreaks = *data as ULONG;

            // Check / fix
            if (*ConfigInfo).NumberOfPhysicalBreaks > SCSI_MAXIMUM_PHYSICAL_BREAKS {
                (*ConfigInfo).NumberOfPhysicalBreaks = SCSI_MAXIMUM_PHYSICAL_BREAKS;
            } else if (*ConfigInfo).NumberOfPhysicalBreaks < SCSI_MINIMUM_PHYSICAL_BREAKS {
                (*ConfigInfo).NumberOfPhysicalBreaks = SCSI_MINIMUM_PHYSICAL_BREAKS;
            }

            crate::glue::dprint_noop!(
                b"MaximumSGList = %d\n\0".as_ptr() as *const i8,
                (*ConfigInfo).NumberOfPhysicalBreaks
            );
        }

        // Get NumberOfRequests
        if _wcsnicmp(name, crate::wide!("NumberOfRequests").as_ptr(), namechars) == 0 {
            if (*KeyValueInformation).Type != REG_DWORD {
                crate::glue::dprint_noop!(
                    b"NumberOfRequests has wrong data type\n\0".as_ptr() as *const i8
                );
                continue;
            }

            (*DeviceExtension).RequestsNumber = *data as ULONG;

            // Check / fix
            if (*DeviceExtension).RequestsNumber < 16 {
                (*DeviceExtension).RequestsNumber = 16;
            } else if (*DeviceExtension).RequestsNumber > 512 {
                (*DeviceExtension).RequestsNumber = 512;
            }

            crate::glue::dprint_noop!(
                b"Number Of Requests = %d\n\0".as_ptr() as *const i8,
                (*DeviceExtension).RequestsNumber
            );
        }

        // Get resource list
        if _wcsnicmp(name, crate::wide!("ResourceList").as_ptr(), namechars) == 0
            || _wcsnicmp(name, crate::wide!("Configuration Data").as_ptr(), namechars) == 0
        {
            // cmtypes.h:33 `#define REG_FULL_RESOURCE_DESCRIPTOR 9` is a MACRO, not a
            // type, so the C's `sizeof(REG_FULL_RESOURCE_DESCRIPTOR)` preprocesses to
            // `sizeof(9)` == sizeof(int) == 4. That is the length floor that actually
            // ships; widening it to size_of::<CM_FULL_RESOURCE_DESCRIPTOR>() would
            // reject blobs the C accepts.
            if (*KeyValueInformation).Type != REG_FULL_RESOURCE_DESCRIPTOR
                || (*KeyValueInformation).DataLength < core::mem::size_of::<core::ffi::c_int>() as u32
            {
                crate::glue::dprint_noop!(b"Bad data type for ResourceList\n\0".as_ptr() as *const i8);
                continue;
            } else {
                crate::glue::dprint_noop!(b"Found ResourceList\n\0".as_ptr() as *const i8);
            }

            let FullResource: *mut CM_FULL_RESOURCE_DESCRIPTOR =
                data as *mut CM_FULL_RESOURCE_DESCRIPTOR;

            // Copy some info from it
            (*InternalConfigInfo).BusNumber = (*FullResource).BusNumber;
            (*ConfigInfo).SystemIoBusNumber = (*FullResource).BusNumber;

            // Loop through it
            let count = (*FullResource).PartialResourceList.Count;
            for Count in 0..count {
                // Get partial descriptor
                let PartialDescriptor: *mut CM_PARTIAL_RESOURCE_DESCRIPTOR = (*FullResource)
                    .PartialResourceList
                    .PartialDescriptors
                    .as_mut_ptr()
                    .add(Count as usize);

                // Check datalength
                if (PartialDescriptor.add(1) as usize - FullResource as usize)
                    > (*KeyValueInformation).DataLength as usize
                {
                    crate::glue::dprint_noop!(
                        b"Resource data is of incorrect size\n\0".as_ptr() as *const i8
                    );
                    break;
                }

                match (*PartialDescriptor).Type {
                    CmResourceTypePort => {
                        if RangeCount >= (*ConfigInfo).NumberOfAccessRanges {
                            crate::glue::dprint_noop!(
                                b"Too many access ranges\n\0".as_ptr() as *const i8
                            );
                            // C's `continue` inside a switch inside a for continues the
                            // FOR, not the switch.
                            continue;
                        }

                        let range = (*InternalConfigInfo).AccessRanges.add(RangeCount as usize);
                        (*range).RangeInMemory = FALSE as BOOLEAN;
                        (*range).RangeStart = (*PartialDescriptor).u.Port.Start;
                        (*range).RangeLength = (*PartialDescriptor).u.Port.Length;
                        RangeCount += 1;
                    }

                    CmResourceTypeMemory => {
                        if RangeCount >= (*ConfigInfo).NumberOfAccessRanges {
                            crate::glue::dprint_noop!(
                                b"Too many access ranges\n\0".as_ptr() as *const i8
                            );
                            continue;
                        }

                        let range = (*InternalConfigInfo).AccessRanges.add(RangeCount as usize);
                        (*range).RangeInMemory = TRUE as BOOLEAN;
                        (*range).RangeStart = (*PartialDescriptor).u.Memory.Start;
                        (*range).RangeLength = (*PartialDescriptor).u.Memory.Length;
                        RangeCount += 1;
                    }

                    CmResourceTypeInterrupt => {
                        if Interrupt == 0 {
                            (*ConfigInfo).BusInterruptLevel = (*PartialDescriptor).u.Interrupt.Level;
                            (*ConfigInfo).BusInterruptVector =
                                (*PartialDescriptor).u.Interrupt.Vector;
                            (*ConfigInfo).InterruptMode = if (*PartialDescriptor).Flags
                                & CM_RESOURCE_INTERRUPT_LATCHED as u16
                                != 0
                            {
                                Latched
                            } else {
                                LevelSensitive
                            };
                        } else if Interrupt == 1 {
                            (*ConfigInfo).BusInterruptLevel2 =
                                (*PartialDescriptor).u.Interrupt.Level;
                            (*ConfigInfo).BusInterruptVector2 =
                                (*PartialDescriptor).u.Interrupt.Vector;
                            (*ConfigInfo).InterruptMode2 = if (*PartialDescriptor).Flags
                                & CM_RESOURCE_INTERRUPT_LATCHED as u16
                                != 0
                            {
                                Latched
                            } else {
                                LevelSensitive
                            };
                        }

                        Interrupt += 1;
                    }

                    CmResourceTypeDma => {
                        if Dma == 0 {
                            (*ConfigInfo).DmaChannel = (*PartialDescriptor).u.Dma.Channel;
                            (*ConfigInfo).DmaPort = (*PartialDescriptor).u.Dma.Port;

                            if (*PartialDescriptor).Flags & CM_RESOURCE_DMA_8 as u16 != 0 {
                                (*ConfigInfo).DmaWidth = Width8Bits;
                            } else if (*PartialDescriptor).Flags & CM_RESOURCE_DMA_16 as u16 != 0
                                || (*PartialDescriptor).Flags & CM_RESOURCE_DMA_8_AND_16 as u16 != 0
                            {
                                (*ConfigInfo).DmaWidth = Width16Bits;
                            } else if (*PartialDescriptor).Flags & CM_RESOURCE_DMA_32 as u16 != 0 {
                                (*ConfigInfo).DmaWidth = Width32Bits;
                            }
                        } else if Dma == 1 {
                            (*ConfigInfo).DmaChannel2 = (*PartialDescriptor).u.Dma.Channel;
                            (*ConfigInfo).DmaPort2 = (*PartialDescriptor).u.Dma.Port;

                            if (*PartialDescriptor).Flags & CM_RESOURCE_DMA_8 as u16 != 0 {
                                (*ConfigInfo).DmaWidth2 = Width8Bits;
                            } else if (*PartialDescriptor).Flags & CM_RESOURCE_DMA_16 as u16 != 0
                                || (*PartialDescriptor).Flags & CM_RESOURCE_DMA_8_AND_16 as u16 != 0
                            {
                                (*ConfigInfo).DmaWidth2 = Width16Bits;
                            } else if (*PartialDescriptor).Flags & CM_RESOURCE_DMA_32 as u16 != 0 {
                                (*ConfigInfo).DmaWidth2 = Width32Bits;
                            }
                        }

                        Dma += 1;
                    }

                    CmResourceTypeDeviceSpecific => {
                        let datasize = (*PartialDescriptor).u.DeviceSpecificData.DataSize;
                        if (datasize as usize) < core::mem::size_of::<CM_SCSI_DEVICE_DATA>()
                            || (PartialDescriptor.add(1) as usize - FullResource as usize)
                                + datasize as usize
                                > (*KeyValueInformation).DataLength as usize
                        {
                            crate::glue::dprint_noop!(
                                b"Resource data length is incorrect\0".as_ptr() as *const i8
                            );
                            // C's `break` here leaves the SWITCH, not the for loop.
                            // Falling out of the match arm is exactly that.
                        } else {
                            // Set only one field from it
                            let ScsiDeviceData =
                                PartialDescriptor.add(1) as *mut CM_SCSI_DEVICE_DATA;
                            (*ConfigInfo).InitiatorBusId[0] = (*ScsiDeviceData).HostIdentifier;
                        }
                    }

                    // The C switch has no `default:` — an unhandled resource type is
                    // simply skipped.
                    _ => {}
                }
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn SpQueryDeviceCallout(
    Context: *mut core::ffi::c_void,
    PathName: *mut core::ffi::c_void,
    BusType: i32,
    BusNumber: u32,
    BusInformation: *mut *mut core::ffi::c_void,
    ControllerType: i32,
    ControllerNumber: u32,
    ControllerInformation: *mut *mut core::ffi::c_void,
    PeripheralType: i32,
    PeripheralNumber: u32,
    PeripheralInformation: *mut *mut core::ffi::c_void,
) -> i32 {
    let found: *mut u8 = Context as *mut u8; // PBOOLEAN = BOOLEAN*
    *found = 1u8; // TRUE
    0x00000000_i32 // STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortAllocateAdapterChannel(
    device_object: *mut DEVICE_OBJECT,
    irp: *mut IRP,
    map_register_base: *mut core::ffi::c_void,
    context: *mut core::ffi::c_void,
) -> IO_ALLOCATION_ACTION {
    let mut irql: KIRQL = 0;
    let device_extension = (*device_object).DeviceExtension as *mut SCSI_PORT_DEVICE_EXTENSION;
    KeAcquireSpinLock((core::ptr::addr_of_mut!((*device_extension).SpinLock)).cast(), core::ptr::addr_of_mut!(irql));
    (*device_extension).MapRegisterBase = map_register_base;
    KeSynchronizeExecution(
        (*device_extension).Interrupt[0],
        Some(core::mem::transmute(ScsiPortStartPacket as *const ())),
        device_object as *mut core::ffi::c_void,
    );
    KeReleaseSpinLock((core::ptr::addr_of_mut!((*device_extension).SpinLock)).cast(), irql);
    KeepObject
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortConvertPhysicalAddressToUlong(Address: SCSI_PHYSICAL_ADDRESS) -> u32 {
    unsafe {
        crate::glue::dprint_noop!(b"ScsiPortConvertPhysicalAddressToUlong()\n\0".as_ptr() as *const i8);
    }
    Address.u.LowPart
}
