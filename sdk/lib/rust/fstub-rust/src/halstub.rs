//! ntoskrnl/fstub/halstub.c — I/O Stub HAL Routines.
//!
//! Defines the HalDispatchTable and HalPrivateDispatchTable globals plus
//! the no-op / NOT_IMPLEMENTED stub bodies they point at. The stubs are
//! overwritten by the actual HAL during boot; until then they catch any
//! caller that hits the dispatch table early.
//!
//! NTAPI on i386 = `extern "stdcall"`; FASTCALL on i386 = `extern "fastcall"`.
//! On amd64/arm64 every ABI collapses to `extern "C"`.

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::STATUS_NOT_SUPPORTED;

use ntkrnl_sys::ddk::{
    HAL_DISPATCH,
    HAL_PRIVATE_DISPATCH,
    HAL_QUERY_INFORMATION_CLASS,
    HAL_SET_INFORMATION_CLASS,
    PADAPTER_OBJECT,
    PBUS_HANDLER,
    PDEBUG_DEVICE_DESCRIPTOR,
    PDEVICE_OBJECT,
    PINSTALL_BUS_HANDLER,
    PMAP_REGISTER_ENTRY,
    PPHYSICAL_ADDRESS,
    PPM_DISPATCH_TABLE,
    PTIME_FIELDS,
    PTRANSLATOR_INTERFACE,
    BUS_DATA_TYPE,
    INTERFACE_TYPE,
    LARGE_INTEGER,
    PHYSICAL_ADDRESS,
    PSTRING,
    PUCHAR,
    KeBugCheckEx,
    _LOADER_PARAMETER_BLOCK,
    _DRIVE_LAYOUT_INFORMATION,
};

// HAL_DISPATCH_VERSION / HAL_PRIVATE_DISPATCH_VERSION — _WIN32_WINNT=0x502 (WS03).
//
//   NTDDI_WS03 < NTDDI_WIN7      → HAL_DISPATCH_VERSION         = 3
//   NTDDI_WS03 < NTDDI_LONGHORN  → HAL_PRIVATE_DISPATCH_VERSION = 2
const HAL_DISPATCH_VERSION: ULONG = 3;
const HAL_PRIVATE_DISPATCH_VERSION: ULONG = 2;

// HAL_INITIALIZATION_FAILED bug check code (sdk/include/reactos/mc/bugcodes.mc).
const HAL_INITIALIZATION_FAILED: ULONG = 0x5C;

// STATUS_INVALID_LEVEL (sdk/include/psdk/ntstatus.h).
const STATUS_INVALID_LEVEL: NTSTATUS = 0xC0000148_u32 as NTSTATUS;

// xHal* stubs that live in disksup.rs (FASTCALL in C, extern "C" in the
// existing Rust translation on amd64/arm64). Declared here so the dispatch
// table can take their address.
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn xHalExamineMBR(
        DeviceObject: PDEVICE_OBJECT,
        SectorSize: ULONG,
        MBRTypeIdentifier: ULONG,
        Buffer: *mut PVOID,
    );
    fn xHalIoAssignDriveLetters(
        LoaderBlock: *mut _LOADER_PARAMETER_BLOCK,
        NtDeviceName: PSTRING,
        NtSystemPath: PUCHAR,
        NtSystemPathString: PSTRING,
    );
    fn xHalIoReadPartitionTable(
        DeviceObject: PDEVICE_OBJECT,
        SectorSize: ULONG,
        ReturnRecognizedPartitions: BOOLEAN,
        PartitionBuffer: *mut *mut _DRIVE_LAYOUT_INFORMATION,
    ) -> NTSTATUS;
    fn xHalIoSetPartitionInformation(
        DeviceObject: PDEVICE_OBJECT,
        SectorSize: ULONG,
        PartitionNumber: ULONG,
        PartitionType: ULONG,
    ) -> NTSTATUS;
    fn xHalIoWritePartitionTable(
        DeviceObject: PDEVICE_OBJECT,
        SectorSize: ULONG,
        SectorsPerTrack: ULONG,
        NumberOfHeads: ULONG,
        PartitionBuffer: *mut _DRIVE_LAYOUT_INFORMATION,
    ) -> NTSTATUS;
}

// On i386 the bindgen pHal* function pointer types are FASTCALL.
// disksup.rs defines these symbols as `extern "C"` on non-x86 only, so
// on i386 they're absent — but halstub.c was already in this state and
// the build links thanks to disksup.c also being present elsewhere.
// We declare them with the FASTCALL ABI that the dispatch table expects.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn xHalExamineMBR(
        DeviceObject: PDEVICE_OBJECT,
        SectorSize: ULONG,
        MBRTypeIdentifier: ULONG,
        Buffer: *mut PVOID,
    );
    fn xHalIoAssignDriveLetters(
        LoaderBlock: *mut _LOADER_PARAMETER_BLOCK,
        NtDeviceName: PSTRING,
        NtSystemPath: PUCHAR,
        NtSystemPathString: PSTRING,
    );
    fn xHalIoReadPartitionTable(
        DeviceObject: PDEVICE_OBJECT,
        SectorSize: ULONG,
        ReturnRecognizedPartitions: BOOLEAN,
        PartitionBuffer: *mut *mut _DRIVE_LAYOUT_INFORMATION,
    ) -> NTSTATUS;
    fn xHalIoSetPartitionInformation(
        DeviceObject: PDEVICE_OBJECT,
        SectorSize: ULONG,
        PartitionNumber: ULONG,
        PartitionType: ULONG,
    ) -> NTSTATUS;
    fn xHalIoWritePartitionTable(
        DeviceObject: PDEVICE_OBJECT,
        SectorSize: ULONG,
        SectorsPerTrack: ULONG,
        NumberOfHeads: ULONG,
        PartitionBuffer: *mut _DRIVE_LAYOUT_INFORMATION,
    ) -> NTSTATUS;
}

// pHalGetInterruptTranslator: bindgen takes a USHORT for Size and Version.
// The C internal hal.h declares xHalGetInterruptTranslator the same way.
// That symbol is provided by HAL itself / not in this file, so we don't
// install it in the dispatch table (set to None — original C had the entry
// but referenced an extern; reading the C carefully: it DOES set it to
// xHalGetInterruptTranslator). The C declares it in include/internal/hal.h
// but the implementation lives in hal.dll. Declare it as extern.
extern "system" {
    fn xHalGetInterruptTranslator(
        ParentInterfaceType: INTERFACE_TYPE,
        ParentBusNumber: ULONG,
        BridgeInterfaceType: INTERFACE_TYPE,
        Size: USHORT,
        Version: USHORT,
        Translator: PTRANSLATOR_INTERFACE,
        BridgeBusNumber: PULONG,
    ) -> NTSTATUS;
}

// ============================================================================
// Function bodies — NTAPI (stdcall on i386, "C" elsewhere).
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn xHalVectorToIDTEntry(Vector: ULONG) -> UCHAR {
    Vector as UCHAR
}

#[no_mangle]
pub unsafe extern "system" fn xHalHaltSystem() {
    loop {}
}

#[no_mangle]
pub unsafe extern "system" fn xHalEndOfBoot() {
    // PAGED_CODE();
}

#[no_mangle]
pub unsafe extern "system" fn xHalSetWakeEnable(_Enable: BOOLEAN) {}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn xHalHandlerForBus(
    _InterfaceType: INTERFACE_TYPE,
    _BusNumber: ULONG,
) -> PBUS_HANDLER {
    core::ptr::null_mut()
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn xHalHandlerForBus(
    _InterfaceType: INTERFACE_TYPE,
    _BusNumber: ULONG,
) -> PBUS_HANDLER {
    core::ptr::null_mut()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn xHalReferenceHandler(_BusHandler: PBUS_HANDLER) {}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn xHalReferenceHandler(_BusHandler: PBUS_HANDLER) {}

#[no_mangle]
pub unsafe extern "system" fn xHalInitPnpDriver() -> NTSTATUS {
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn xHalInitPowerManagement(
    _PmDriverDispatchTable: PPM_DISPATCH_TABLE,
    _PmHalDispatchTable: *mut PPM_DISPATCH_TABLE,
) -> NTSTATUS {
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn xHalStartMirroring() -> NTSTATUS {
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn xHalEndMirroring(_PassNumber: ULONG) -> NTSTATUS {
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn xHalMirrorPhysicalMemory(
    _PhysicalAddress: PHYSICAL_ADDRESS,
    _NumberOfBytes: LARGE_INTEGER,
) -> NTSTATUS {
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn xHalQueryBusSlots(
    _BusHandler: PBUS_HANDLER,
    _BufferSize: ULONG,
    _SlotNumbers: PULONG,
    _ReturnedLength: PULONG,
) -> NTSTATUS {
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn xHalSetSystemInformation(
    _InformationClass: HAL_SET_INFORMATION_CLASS,
    _BufferSize: ULONG,
    _Buffer: PVOID,
) -> NTSTATUS {
    STATUS_INVALID_LEVEL
}

#[no_mangle]
pub unsafe extern "system" fn xHalQuerySystemInformation(
    _InformationClass: HAL_QUERY_INFORMATION_CLASS,
    _BufferSize: ULONG,
    _Buffer: PVOID,
    _ReturnedLength: PULONG,
) -> NTSTATUS {
    STATUS_INVALID_LEVEL
}

#[no_mangle]
pub unsafe extern "system" fn xHalLocateHiberRanges(_MemoryMap: PVOID) {}

#[no_mangle]
pub unsafe extern "system" fn xHalRegisterBusHandler(
    _InterfaceType: INTERFACE_TYPE,
    _ConfigSpace: BUS_DATA_TYPE,
    _BusNumber: ULONG,
    _ParentInterfaceType: INTERFACE_TYPE,
    _ParentBusNumber: ULONG,
    _ContextSize: ULONG,
    _InstallCallback: PINSTALL_BUS_HANDLER,
    _BusHandler: *mut PBUS_HANDLER,
) -> NTSTATUS {
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn xHalSetWakeAlarm(
    _AlartTime: ULONGLONG,
    _TimeFields: PTIME_FIELDS,
) {
}

#[no_mangle]
pub unsafe extern "system" fn xHalTranslateBusAddress(
    _InterfaceType: INTERFACE_TYPE,
    _BusNumber: ULONG,
    _BusAddress: PHYSICAL_ADDRESS,
    _AddressSpace: PULONG,
    _TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    KeBugCheckEx(HAL_INITIALIZATION_FAILED, 0, 0, 0, 0);
}

#[no_mangle]
pub unsafe extern "system" fn xHalAllocateMapRegisters(
    _AdapterObject: PADAPTER_OBJECT,
    _Unknown: ULONG,
    _Unknown2: ULONG,
    _Registers: PMAP_REGISTER_ENTRY,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn xKdSetupPciDeviceForDebugging(
    _LoaderBlock: PVOID,
    _PciDevice: PDEBUG_DEVICE_DESCRIPTOR,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn xKdReleasePciDeviceForDebugging(
    _PciDevice: PDEBUG_DEVICE_DESCRIPTOR,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn xKdGetAcpiTablePhase(
    _LoaderBlock: *mut _LOADER_PARAMETER_BLOCK,
    _Signature: ULONG,
) -> PVOID {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn MatchAll(
    _PhysicalAddress: PHYSICAL_ADDRESS,
    _NumberPages: ULONG,
    _FlushCurrentTLB: BOOLEAN,
) -> PVOID {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn xKdUnmapVirtualAddress(
    _VirtualAddress: PVOID,
    _NumberPages: ULONG,
    _FlushCurrentTLB: BOOLEAN,
) {
}

// ============================================================================
// i386-specific stubs for HalPrivateDispatchTable to fix argument count
// and ABI mismatches that cause stack corruption.
// ============================================================================

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn xKdCheckPowerButtonStub() -> BOOLEAN {
    // Slot expects 0 args. Original C cast xHalReferenceHandler to this.
    // xHalReferenceHandler is FASTCALL and takes 1 arg (BusHandler),
    // which it ignores. We can just do nothing.
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn MatchAllStub(
    PhysicalAddress: PHYSICAL_ADDRESS,
    NumberPages: ULONG,
) -> PVOID {
    // Slot expects 2 args, MatchAll takes 3.
    MatchAll(PhysicalAddress, NumberPages, 0)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn xKdUnmapVirtualAddressStub(
    VirtualAddress: PVOID,
    NumberPages: ULONG,
) {
    // Slot expects 2 args, xKdUnmapVirtualAddress takes 3.
    xKdUnmapVirtualAddress(VirtualAddress, NumberPages, 0)
}

// ============================================================================
// Dispatch table globals.
//
// Bindgen field names differ from the C struct comments; they're matched here
// exactly. C uses positional init, Rust uses named init. All three arches
// share the same struct layout.
// ============================================================================

#[no_mangle]
pub static mut HalDispatchTable: HAL_DISPATCH = HAL_DISPATCH {
    Version: HAL_DISPATCH_VERSION,
    HalQuerySystemInformation: Some(xHalQuerySystemInformation),
    HalSetSystemInformation: Some(xHalSetSystemInformation),
    HalQueryBusSlots: Some(xHalQueryBusSlots),
    Spare1: 0,
    HalExamineMBR: Some(xHalExamineMBR),
    HalIoAssignDriveLetters: Some(xHalIoAssignDriveLetters),
    HalIoReadPartitionTable: Some(xHalIoReadPartitionTable),
    HalIoSetPartitionInformation: Some(xHalIoSetPartitionInformation),
    HalIoWritePartitionTable: Some(xHalIoWritePartitionTable),
    HalReferenceHandlerForBus: Some(xHalHandlerForBus),
    HalReferenceBusHandler: Some(xHalReferenceHandler),
    HalDereferenceBusHandler: Some(xHalReferenceHandler),
    HalInitPnpDriver: Some(xHalInitPnpDriver),
    HalInitPowerManagement: Some(xHalInitPowerManagement),
    HalGetDmaAdapter: None, // C: (pHalGetDmaAdapter)NULL
    HalGetInterruptTranslator: Some(xHalGetInterruptTranslator),
    HalStartMirroring: Some(xHalStartMirroring),
    HalEndMirroring: Some(xHalEndMirroring),
    HalMirrorPhysicalMemory: Some(xHalMirrorPhysicalMemory),
    HalEndOfBoot: Some(xHalEndOfBoot),
    HalMirrorVerify: Some(xHalMirrorPhysicalMemory),
    // Trailing fields beyond what the C positional initializer covers
    // — the C source (NTDDI_WS03 layout) doesn't initialize these; the
    // bindgen struct (always full size) defaults them to None.
    HalGetCachedAcpiTable: None,
    HalSetPciErrorHandlerCallback: None,
};

#[no_mangle]
pub static mut HalPrivateDispatchTable: HAL_PRIVATE_DISPATCH = HAL_PRIVATE_DISPATCH {
    Version: HAL_PRIVATE_DISPATCH_VERSION,
    HalHandlerForBus: Some(xHalHandlerForBus),
    // C cast: (pHalHandlerForConfigSpace)xHalHandlerForBus — same prototype
    // shape, different INTERFACE_TYPE/BUS_DATA_TYPE on the first arg.
    // transmute over the Option<fn> to keep the cast intent.
    HalHandlerForConfigSpace: unsafe {
        core::mem::transmute::<
            ntkrnl_sys::ddk::pHalHandlerForBus,
            ntkrnl_sys::ddk::pHalHandlerForConfigSpace,
        >(Some(xHalHandlerForBus))
    },
    HalLocateHiberRanges: Some(xHalLocateHiberRanges),
    HalRegisterBusHandler: Some(xHalRegisterBusHandler),
    HalSetWakeEnable: Some(xHalSetWakeEnable),
    HalSetWakeAlarm: Some(xHalSetWakeAlarm),
    HalPciTranslateBusAddress: Some(xHalTranslateBusAddress),
    // C cast: (pHalAssignSlotResources)xHalTranslateBusAddress.
    HalPciAssignSlotResources: unsafe {
        core::mem::transmute::<
            ntkrnl_sys::ddk::pHalTranslateBusAddress,
            ntkrnl_sys::ddk::pHalAssignSlotResources,
        >(Some(xHalTranslateBusAddress))
    },
    HalHaltSystem: Some(xHalHaltSystem),
    HalFindBusAddressTranslation: None, // C: (pHalFindBusAddressTranslation)NULL
    HalResetDisplay: None,               // C: (pHalResetDisplay)NULL
    HalAllocateMapRegisters: Some(xHalAllocateMapRegisters),
    KdSetupPciDeviceForDebugging: Some(xKdSetupPciDeviceForDebugging),
    KdReleasePciDeviceforDebugging: Some(xKdReleasePciDeviceForDebugging),
    KdGetAcpiTablePhase0: Some(xKdGetAcpiTablePhase),
    KdCheckPowerButton: unsafe {
        #[cfg(target_arch = "x86")]
        {
            core::mem::transmute::<
                unsafe extern "stdcall" fn() -> BOOLEAN,
                ntkrnl_sys::ddk::pKdCheckPowerButton,
            >(xKdCheckPowerButtonStub)
        }
        #[cfg(not(target_arch = "x86"))]
        {
            core::mem::transmute::<
                ntkrnl_sys::ddk::pHalReferenceBusHandler,
                ntkrnl_sys::ddk::pKdCheckPowerButton,
            >(Some(xHalReferenceHandler))
        }
    },
    HalVectorToIDTEntry: Some(xHalVectorToIDTEntry),
    // C cast: (pKdMapPhysicalMemory64)MatchAll. MatchAll takes 3 args
    // (PhysicalAddress, NumberPages, FlushCurrentTLB) but the dispatch
    // slot's prototype is 2 args (PhysicalAddress, NumberPages).
    // Use an Option<fn> transmute keyed on the actual fn item ABI per arch.
    #[cfg(not(target_arch = "x86"))]
    KdMapPhysicalMemory64: unsafe {
        core::mem::transmute::<
            Option<unsafe extern "system" fn(PHYSICAL_ADDRESS, ULONG, BOOLEAN) -> PVOID>,
            ntkrnl_sys::ddk::pKdMapPhysicalMemory64,
        >(Some(MatchAll))
    },
    #[cfg(target_arch = "x86")]
    KdMapPhysicalMemory64: unsafe {
        core::mem::transmute::<
            unsafe extern "stdcall" fn(PHYSICAL_ADDRESS, ULONG) -> PVOID,
            ntkrnl_sys::ddk::pKdMapPhysicalMemory64,
        >(MatchAllStub)
    },
    // C cast: (pKdUnmapVirtualAddress)xKdUnmapVirtualAddress. Same shape
    // mismatch: the stub takes a 3rd FlushCurrentTLB argument that the
    // dispatch slot's prototype doesn't pass.
    #[cfg(not(target_arch = "x86"))]
    KdUnmapVirtualAddress: unsafe {
        core::mem::transmute::<
            Option<unsafe extern "system" fn(PVOID, ULONG, BOOLEAN)>,
            ntkrnl_sys::ddk::pKdUnmapVirtualAddress,
        >(Some(xKdUnmapVirtualAddress))
    },
    #[cfg(target_arch = "x86")]
    KdUnmapVirtualAddress: unsafe {
        core::mem::transmute::<
            unsafe extern "stdcall" fn(PVOID, ULONG),
            ntkrnl_sys::ddk::pKdUnmapVirtualAddress,
            >(xKdUnmapVirtualAddressStub)
    },
};

// ============================================================================
// Original C source (ntoskrnl/fstub/halstub.c).
// ============================================================================
//
// /*
// * PROJECT:         ReactOS Kernel
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            ntoskrnl/fstub/halstub.c
// * PURPOSE:         I/O Stub HAL Routines
// * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//                    Pierre Schweitzer (pierre.schweitzer@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// HAL_DISPATCH HalDispatchTable =
// {
//     HAL_DISPATCH_VERSION,
//     xHalQuerySystemInformation,
//     xHalSetSystemInformation,
//     xHalQueryBusSlots,
//     0,
//     xHalExamineMBR,
//     xHalIoAssignDriveLetters,
//     xHalIoReadPartitionTable,
//     xHalIoSetPartitionInformation,
//     xHalIoWritePartitionTable,
//     xHalHandlerForBus,
//     xHalReferenceHandler,
//     xHalReferenceHandler,
//     xHalInitPnpDriver,
//     xHalInitPowerManagement,
//     (pHalGetDmaAdapter) NULL,
//     xHalGetInterruptTranslator,
//     xHalStartMirroring,
//     xHalEndMirroring,
//     xHalMirrorPhysicalMemory,
//     xHalEndOfBoot,
//     xHalMirrorPhysicalMemory
// };
//
// HAL_PRIVATE_DISPATCH HalPrivateDispatchTable =
// {
//     HAL_PRIVATE_DISPATCH_VERSION,
//     xHalHandlerForBus,
//     (pHalHandlerForConfigSpace)xHalHandlerForBus,
//     xHalLocateHiberRanges,
//     xHalRegisterBusHandler,
//     xHalSetWakeEnable,
//     xHalSetWakeAlarm,
//     xHalTranslateBusAddress,
//     (pHalAssignSlotResources)xHalTranslateBusAddress,
//     xHalHaltSystem,
//     (pHalFindBusAddressTranslation)NULL,
//     (pHalResetDisplay)NULL,
//     xHalAllocateMapRegisters,
//     xKdSetupPciDeviceForDebugging,
//     xKdReleasePciDeviceForDebugging,
//     xKdGetAcpiTablePhase,
//     (pKdCheckPowerButton)xHalReferenceHandler,
//     xHalVectorToIDTEntry,
//     (pKdMapPhysicalMemory64)MatchAll,
//     (pKdUnmapVirtualAddress)xKdUnmapVirtualAddress
// };
//
// /* FUNCTIONS *****************************************************************/
//
// UCHAR
// NTAPI
// xHalVectorToIDTEntry(IN ULONG Vector)
// {
//     /* Return the vector */
//     return (UCHAR)Vector;
// }
//
// VOID
// NTAPI
// xHalHaltSystem(VOID)
// {
//     /* Halt execution */
//     while (TRUE);
// }
//
// VOID
// NTAPI
// xHalEndOfBoot(VOID)
// {
//     PAGED_CODE();
//
//     /* Nothing */
//     return;
// }
//
// VOID
// NTAPI
// xHalSetWakeEnable(IN BOOLEAN Enable)
// {
//     /* Nothing */
//     return;
// }
//
// PBUS_HANDLER
// FASTCALL
// xHalHandlerForBus(IN INTERFACE_TYPE InterfaceType,
//                   IN ULONG BusNumber)
// {
//     return NULL;
// }
//
// VOID
// FASTCALL
// xHalReferenceHandler(IN PBUS_HANDLER BusHandler)
// {
//     /* Nothing */
//     return;
// }
//
// NTSTATUS
// NTAPI
// xHalInitPnpDriver(VOID)
// {
//     return STATUS_NOT_SUPPORTED;
// }
//
// NTSTATUS
// NTAPI
// xHalInitPowerManagement(IN PPM_DISPATCH_TABLE PmDriverDispatchTable,
//                         OUT PPM_DISPATCH_TABLE *PmHalDispatchTable)
// {
//     return STATUS_NOT_SUPPORTED;
// }
//
// NTSTATUS
// NTAPI
// xHalStartMirroring(VOID)
// {
//     PAGED_CODE();
//
//     return STATUS_NOT_SUPPORTED;
// }
//
// NTSTATUS
// NTAPI
// xHalEndMirroring(IN ULONG PassNumber)
// {
//     return STATUS_NOT_SUPPORTED;
// }
//
// NTSTATUS
// NTAPI
// xHalMirrorPhysicalMemory(IN PHYSICAL_ADDRESS PhysicalAddress,
//                          IN LARGE_INTEGER NumberOfBytes)
// {
//     return STATUS_NOT_SUPPORTED;
// }
//
// NTSTATUS
// NTAPI
// xHalQueryBusSlots(IN PBUS_HANDLER BusHandler,
//                   IN ULONG BufferSize,
//                   OUT PULONG SlotNumbers,
//                   OUT PULONG ReturnedLength)
// {
//     PAGED_CODE();
//
//     return STATUS_NOT_SUPPORTED;
// }
//
// NTSTATUS
// NTAPI
// xHalSetSystemInformation(IN HAL_SET_INFORMATION_CLASS InformationClass,
//                          IN ULONG BufferSize,
//                          IN PVOID Buffer)
// {
//     PAGED_CODE();
//
//     return STATUS_INVALID_LEVEL;
// }
//
// NTSTATUS
// NTAPI
// xHalQuerySystemInformation(IN HAL_QUERY_INFORMATION_CLASS InformationClass,
//                            IN ULONG BufferSize,
//                            IN OUT PVOID Buffer,
//                            OUT PULONG ReturnedLength)
// {
//     PAGED_CODE();
//
//     return STATUS_INVALID_LEVEL;
// }
//
// VOID
// NTAPI
// xHalLocateHiberRanges(IN PVOID MemoryMap)
// {
//     /* Nothing */
//     return;
// }
//
// NTSTATUS
// NTAPI
// xHalRegisterBusHandler(IN INTERFACE_TYPE InterfaceType,
//                        IN BUS_DATA_TYPE ConfigSpace,
//                        IN ULONG BusNumber,
//                        IN INTERFACE_TYPE ParentInterfaceType,
//                        IN ULONG ParentBusNumber,
//                        IN ULONG ContextSize,
//                        IN PINSTALL_BUS_HANDLER InstallCallback,
//                        OUT PBUS_HANDLER *BusHandler)
// {
//     PAGED_CODE();
//
//     return STATUS_NOT_SUPPORTED;
// }
//
// VOID
// NTAPI
// xHalSetWakeAlarm(IN ULONGLONG AlartTime,
//                  IN PTIME_FIELDS TimeFields)
// {
//     /* Nothing */
//     return;
// }
//
// BOOLEAN
// NTAPI
// xHalTranslateBusAddress(IN INTERFACE_TYPE InterfaceType,
//                         IN ULONG BusNumber,
//                         IN PHYSICAL_ADDRESS BusAddress,
//                         IN OUT PULONG AddressSpace,
//                         OUT PPHYSICAL_ADDRESS TranslatedAddress)
// {
// #ifdef _M_IX86
//     {
//         /* Dump return address to debugcon via GAS inline asm */
//         ULONG_PTR retaddr = (ULONG_PTR)__builtin_return_address(0);
//         int i;
//         __asm__ __volatile__("outb %%al, $0xe9" :: "a"((UCHAR)'S'));
//         for (i = 28; i >= 0; i -= 4)
//         {
//             UCHAR d = (UCHAR)((retaddr >> i) & 0xF);
//             UCHAR ch = d < 10 ? '0' + d : 'a' + d - 10;
//             __asm__ __volatile__("outb %%al, $0xe9" :: "a"(ch));
//         }
//         __asm__ __volatile__("outb %%al, $0xe9" :: "a"((UCHAR)'!'));
//     }
// #endif
//     KeBugCheckEx(HAL_INITIALIZATION_FAILED, 0, 0, 0, 0);
//
//     return FALSE;
// }
//
// NTSTATUS
// NTAPI
// xHalAllocateMapRegisters(IN PADAPTER_OBJECT AdapterObject,
//                          IN ULONG Unknown,
//                          IN ULONG Unknown2,
//                          PMAP_REGISTER_ENTRY Registers)
// {
//     PAGED_CODE();
//
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// xKdSetupPciDeviceForDebugging(IN PVOID LoaderBlock OPTIONAL,
//                               IN OUT PDEBUG_DEVICE_DESCRIPTOR PciDevice)
// {
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// xKdReleasePciDeviceForDebugging(IN OUT PDEBUG_DEVICE_DESCRIPTOR PciDevice)
// {
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// PVOID
// NTAPI
// xKdGetAcpiTablePhase(IN struct _LOADER_PARAMETER_BLOCK *LoaderBlock,
//                      IN ULONG Signature)
// {
//     return NULL;
// }
//
// PVOID
// NTAPI
// MatchAll(IN PHYSICAL_ADDRESS PhysicalAddress,
//          IN ULONG NumberPages,
//          IN BOOLEAN FlushCurrentTLB)
// {
//     return NULL;
// }
//
// VOID
// NTAPI
// xKdUnmapVirtualAddress(IN PVOID VirtualAddress,
//                        IN ULONG NumberPages,
//                        IN BOOLEAN FlushCurrentTLB)
// {
//     /* Nothing */
//     return;
// }
