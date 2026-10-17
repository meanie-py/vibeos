//! Legacy HAL bus-support layer — translated from `hal/halx86/legacy/bussupp.c`.
//!
//! i386-only.  Allocates per-interface BUS_HANDLERs (Internal/CMOS/ISA/PCI),
//! runs HalpInitializePciBus to enumerate PCI buses, and exports the public
//! Hal{Get,Set}BusData{,ByOffset} / HalGetInterruptVector /
//! HalAdjustResourceList / HalAssignSlotResources / HalTranslateBusAddress
//! wrappers that drivers + the kernel reach through.

#![cfg(target_arch = "x86")]
#![allow(non_snake_case, non_upper_case_globals, dead_code, unused_unsafe, unused_variables, unused_mut, unused_imports)]

use ntkrnl_sys::hal_ddk::{
    BOOLEAN, BUS_DATA_TYPE, HANDLE, INTERFACE_TYPE, KIRQL, KSPIN_LOCK, LIST_ENTRY, NTSTATUS,
    PBUS_HANDLER, PCM_RESOURCE_LIST, PCI_SLOT_NUMBER, PDEVICE_OBJECT, PDRIVER_OBJECT,
    PHAL_BUS_HANDLER, PHYSICAL_ADDRESS, PIO_RESOURCE_REQUIREMENTS_LIST, PKAFFINITY, PKIRQL,
    PLIST_ENTRY, PPCIPBUSDATA, PPCI_CARD_DESCRIPTOR, PCIPBUSDATA, _PCI_CARD_DESCRIPTOR,
    HAL_BUS_HANDLER, KEY_VALUE_PARTIAL_INFORMATION, PCI_COMMON_HEADER,
    _PCI_COMMON_HEADER__bindgen_ty_1 as PCI_COMMON_HEADER__u,
    _PCI_COMMON_HEADER__bindgen_ty_1__PCI_HEADER_TYPE_0 as PCI_COMMON_HEADER__u__type0,
    PPCI_REGISTRY_INFO_INTERNAL, PPHYSICAL_ADDRESS, PULONG_PTR, PUNICODE_STRING, PUSHORT, PVOID,
    // bindgen split _PCI_COMMON_CONFIG (opaque DeviceSpecific[192]) from
    // _PCI_COMMON_HEADER (the real field layout VendorID/DeviceID/...).
    // The C source uses PCI_COMMON_CONFIG but accesses HEADER fields — alias
    // them to match WDK semantics.
    _PCI_COMMON_HEADER as _PCI_COMMON_CONFIG,
    SUPPORTED_RANGES, UCHAR, ULONG, ULONG_PTR, USHORT, _UNICODE_STRING, _OBJECT_ATTRIBUTES,
    PCI_ADDRESS_IO_SPACE, PCI_ADDRESS_MEMORY_TYPE_MASK, PCI_BRIDGE_TYPE, PCI_CARDBUS_BRIDGE_TYPE,
    PCI_DEVICE_TYPE, PCI_INVALID_VENDORID, PCI_MAX_DEVICES, PCI_MULTIFUNCTION,
    PCI_STATUS_66MHZ_CAPABLE, PCI_STATUS_DEVSEL, PCI_TYPE0_ADDRESSES, PCI_TYPE1_ADDRESSES,
    PCI_TYPE1_DATA_PORT, PCI_TYPE2_ADDRESS_BASE, PCI_TYPE_32BIT, PCI_TYPE_64BIT,
    PCI_DATA_VERSION, PCI_ENABLE_BUS_MASTER, PCI_CLASS_BRIDGE_DEV, PCI_CLASS_MASS_STORAGE_CTLR,
    PCI_SUBCLASS_BR_CARDBUS, PCI_SUBCLASS_BR_PCI_TO_PCI, PCI_SUBCLASS_MSC_IDE_CTLR,
    PCI_ADDRESS_IO_ADDRESS_MASK, PCI_ADDRESS_MEMORY_ADDRESS_MASK, PCI_ADDRESS_MEMORY_PREFETCHABLE,
    PCIConfigHandler, PCIConfigHandlerType1, PCIConfigHandlerType2,
    HalpCheckPowerButton, HalpSetupPciDeviceForDebugging, HalpReleasePciDeviceForDebugging,
    HalpGetNMICrashFlag,
    HALP_CARD_FEATURE_FULL_DECODE, HALP_CHECK_CARD_REVISION_ID, HALP_CHECK_CARD_SUBSYSTEM_ID,
    HALP_CHECK_CARD_SUBVENDOR_ID, HAL_PCI_CHIP_HACK_BROKEN_ACPI_TIMER,
    HAL_PCI_CHIP_HACK_DISABLE_ACPI_IRQ_ROUTING, HAL_PCI_CHIP_HACK_DISABLE_HIBERNATE,
    HAL_PCI_CHIP_HACK_USB_SMI_DISABLE, HAL_SUPPORTED_RANGE_VERSION, MACHINE_TYPE_ISA,
    KEY_ALL_ACCESS, KEY_READ, REG_DWORD, OBJ_CASE_INSENSITIVE,
    _INTERFACE_TYPE_Internal,
    _INTERFACE_TYPE_Isa,
    _INTERFACE_TYPE_Eisa,
    _INTERFACE_TYPE_PCIBus,
    _INTERFACE_TYPE_InterfaceTypeUndefined,
    _BUS_DATA_TYPE_Cmos,
    _BUS_DATA_TYPE_PCIConfiguration,
    _BUS_DATA_TYPE_ConfigurationSpaceUndefined,
};

#[cfg(not(feature = "minihal"))]
use crate::x86::bushndlr::HalpInitBusHandler;
use crate::x86::bushndlr::{HalpNoBusData, HaliReferenceHandlerForConfigSpace};
// sysbus/memory are full-HAL modules gated out of minihal; their imports
// are only consumed by bussupp's `#[cfg(not(feature="minihal"))]` paths.
// feedback_freeldr_clean_build_regression
#[cfg(not(feature = "minihal"))]
use crate::x86::sysbus::{HalpGetSystemInterruptVector, HalpTranslateSystemBusAddress};
use crate::x86::isabus::HalpTranslateIsaBusAddress;
use crate::x86::cmosbus::{HalpcGetCmosData, HalpcSetCmosData};
#[cfg(not(feature = "minihal"))]
use crate::x86::memory::{HalpMapPhysicalMemory64Vista, HalpUnmapVirtualAddressVista};
use crate::x86::pcr::KeGetCurrentPrcb;

// HalDispatchTable / HalPrivateDispatchTable:
//
// In the full HAL build these come from `hal_ddk`, where bindgen decorates
// them with `#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]` — correct
// for the real HAL (the kernel exports them).
//
// In the FreeLDR `--features minihal` build there is NO ntoskrnl.exe; the
// loader provides these globals from its own libs as PLAIN externs. Using the
// raw-dylib decl here would force rosload.exe to spuriously import
// ntoskrnl.exe (breaks FreeLDR — feedback_freeldr_clean_build_regression).
// Mirrors bushndlr.rs. Both tables are referenced by the minihal-DEFINES
// common-region functions (HalpAllocateBusHandler / HalpAssignSlotResources /
// HaliTranslateBusAddress / HalAssignSlotResources / HalGetBusDataByOffset /
// HalTranslateBusAddress), so both need the plain-extern alternative.
#[cfg(not(feature = "minihal"))]
use ntkrnl_sys::hal_ddk::{HalDispatchTable, HalPrivateDispatchTable};

#[cfg(feature = "minihal")]
unsafe extern "C" {
    static mut HalDispatchTable: ntkrnl_sys::hal_ddk::HAL_DISPATCH;
    static mut HalPrivateDispatchTable: ntkrnl_sys::hal_ddk::_HAL_PRIVATE_DISPATCH;
}

const TAG_HAL: u32 = u32::from_le_bytes(*b"Hal ");
const PCI_DATA_TAG: u32 = u32::from_le_bytes(*b"PCI ");
const PCI_COMMON_HDR_LENGTH: usize = 64;
const PCI_MAX_FUNCTION: u32 = 8;
const PAGED_POOL: i32 = 1;
const NON_PAGED_POOL: i32 = 0;
const NonPagedPoolMustSucceed: i32 = 2;
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009Au32 as NTSTATUS;

#[inline]
fn nt_success(s: NTSTATUS) -> bool { (s as i32) >= 0 }

// _PCI_COMMON_CONFIG is aliased to _PCI_COMMON_HEADER above; restore the
// pointer type alias the draft uses everywhere.
pub type PPCI_COMMON_CONFIG = *mut _PCI_COMMON_CONFIG;

#[unsafe(no_mangle)]
pub static mut HalpPciIrqMask: ULONG = 0;

static mut WarningsGiven: [BOOLEAN; 5] = [0; 5];

// PCI_CONFIGURATION_TYPE — header type bits (0=device, 1=bridge, 2=cardbus),
// masking off the multi-function bit (0x80).
#[inline(always)]
#[allow(non_snake_case)]
unsafe fn PCI_CONFIGURATION_TYPE(pci: PPCI_COMMON_CONFIG) -> u32 {
    unsafe { ((*pci).HeaderType & 0x7F) as u32 }
}

// KeInitializeSpinLock is the kernel macro `*sl = 0` (UP + DUMMY_PSEH),
// NOT linkable in the FreeLDR mini-HAL build. Inline per
// system_prompt_hal.md (matches dma.rs). feedback_freeldr_clean_build_regression
#[allow(non_snake_case)]
#[inline]
unsafe fn KeInitializeSpinLock(SpinLock: *mut KSPIN_LOCK) {
    unsafe { *SpinLock = 0 }
}

// Pull bus-handler-context helpers from bushndlr.rs.
use crate::x86::bushndlr::{HalpAllBusHandlers, HalpContextToBusHandler};

// C-side PCI port constants — Type1 (CF8/CFC), Type2 (CF8/CFA via forward).
const PCI_TYPE1_ADDRESS_PORT: u32 = 0xCF8;
const PCI_TYPE2_CSE_PORT: u32 = 0xCF8;
const PCI_TYPE2_FORWARD_PORT: u32 = 0xCFA;
const STATUS_NOT_FOUND: NTSTATUS = 0xC0000225u32 as NTSTATUS;

// _INTERFACE_TYPE_PCIBus aliases as PCIBus (the C identifier).  Same applies
// for Internal, Isa, ConfigurationSpaceUndefined, Cmos, etc.
const PCIBus: INTERFACE_TYPE = _INTERFACE_TYPE_PCIBus;
const Internal: INTERFACE_TYPE = _INTERFACE_TYPE_Internal;
const Isa: INTERFACE_TYPE = _INTERFACE_TYPE_Isa;
const Eisa: INTERFACE_TYPE = _INTERFACE_TYPE_Eisa;
const InterfaceTypeUndefined: INTERFACE_TYPE = _INTERFACE_TYPE_InterfaceTypeUndefined;
const Cmos: BUS_DATA_TYPE = _BUS_DATA_TYPE_Cmos;
const PCIConfiguration: BUS_DATA_TYPE = _BUS_DATA_TYPE_PCIConfiguration;
const ConfigurationSpaceUndefined: BUS_DATA_TYPE = _BUS_DATA_TYPE_ConfigurationSpaceUndefined;

unsafe extern "C" {
    static HalpBusType: UCHAR;
    static mut HalpPCIConfigLock: KSPIN_LOCK;
}
unsafe extern "system" {
    // All NTAPI (stdcall on i386).  See hal/halx86/include/bus.h.
    fn HalpReadPCIConfig(BusHandler: PBUS_HANDLER, Slot: PCI_SLOT_NUMBER, Buffer: PVOID, Offset: ULONG, Length: ULONG);
    fn HalpWritePCIConfig(BusHandler: PBUS_HANDLER, Slot: PCI_SLOT_NUMBER, Buffer: PVOID, Offset: ULONG, Length: ULONG);
    fn HalpQueryPciRegistryInfo() -> PPCI_REGISTRY_INFO_INTERNAL;
    fn HalpGetPCIData(BusHandler: PBUS_HANDLER, RootHandler: PBUS_HANDLER, Slot: PCI_SLOT_NUMBER, Buffer: PVOID, Offset: ULONG, Length: ULONG) -> ULONG;
    fn HalpSetPCIData(BusHandler: PBUS_HANDLER, RootHandler: PBUS_HANDLER, Slot: PCI_SLOT_NUMBER, Buffer: PVOID, Offset: ULONG, Length: ULONG) -> ULONG;
    fn HalpAdjustPCIResourceList(BusHandler: PBUS_HANDLER, RootHandler: PBUS_HANDLER, pResourceList: *mut PIO_RESOURCE_REQUIREMENTS_LIST) -> NTSTATUS;
    fn HalpAssignPCISlotResources(BusHandler: PBUS_HANDLER, RootHandler: PBUS_HANDLER, RegistryPath: PUNICODE_STRING, DriverClassName: PUNICODE_STRING, DriverObject: PDRIVER_OBJECT, DeviceObject: PDEVICE_OBJECT, SlotNumber: ULONG, AllocatedResources: *mut PCM_RESOURCE_LIST) -> NTSTATUS;
    fn HalpGetISAFixedPCIIrq(BusHandler: PBUS_HANDLER, RootHandler: PBUS_HANDLER, PciSlot: PCI_SLOT_NUMBER, Range: *mut ntkrnl_sys::hal_ddk::PSUPPORTED_RANGE) -> NTSTATUS;
    fn HalpGetPCIIntOnISABus(BusHandler: PBUS_HANDLER, RootHandler: PBUS_HANDLER, BusInterruptLevel: ULONG, BusInterruptVector: ULONG, Irql: PKIRQL, Affinity: PKAFFINITY) -> ULONG;
    fn HalpPCIPin2ISALine(BusHandler: PBUS_HANDLER, RootHandler: PBUS_HANDLER, Slot: PCI_SLOT_NUMBER, PciData: PPCI_COMMON_CONFIG);
    fn HalpPCIISALine2Pin(BusHandler: PBUS_HANDLER, RootHandler: PBUS_HANDLER, Slot: PCI_SLOT_NUMBER, PciNewData: PPCI_COMMON_CONFIG, PciOldData: PPCI_COMMON_CONFIG);
}

unsafe extern "system" {
    fn ExAllocatePoolWithTag(PoolType: i32, NumberOfBytes: usize, Tag: u32) -> PVOID;
    fn ExFreePoolWithTag(P: PVOID, Tag: u32);
    fn ZwClose(Handle: HANDLE) -> NTSTATUS;
    fn ZwOpenKey(KeyHandle: *mut HANDLE, DesiredAccess: u32, ObjectAttributes: *mut _OBJECT_ATTRIBUTES) -> NTSTATUS;
    fn ZwQueryValueKey(KeyHandle: HANDLE, ValueName: PUNICODE_STRING, KeyValueInformationClass: i32, KeyValueInformation: PVOID, Length: ULONG, ResultLength: *mut ULONG) -> NTSTATUS;
    fn ZwSetValueKey(KeyHandle: HANDLE, ValueName: PUNICODE_STRING, TitleIndex: ULONG, Type: ULONG, Data: PVOID, DataSize: ULONG) -> NTSTATUS;
    fn RtlInitUnicodeString(DestinationString: *mut _UNICODE_STRING, SourceString: *const u16);
    fn RtlInitializeBitMap(BitMapHeader: PVOID, BitMapBuffer: *mut ULONG, SizeOfBitMap: ULONG);
    fn HalpOpenRegistryKey(KeyHandle: *mut HANDLE, RootKey: HANDLE, KeyName: PUNICODE_STRING, DesiredAccess: u32, Create: BOOLEAN) -> NTSTATUS;
}

use core::ffi::c_void;





#[no_mangle]
pub unsafe extern "system" fn HalpAllocateBusHandler(
    InterfaceType: INTERFACE_TYPE,
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    ParentBusInterfaceType: INTERFACE_TYPE,
    ParentBusNumber: ULONG,
    BusSpecificData: ULONG,
) -> PBUS_HANDLER {
    

    let mut Bus: PBUS_HANDLER = core::ptr::null_mut();

    HalPrivateDispatchTable.HalRegisterBusHandler.unwrap()(
        InterfaceType,
        BusDataType,
        BusNumber,
        ParentBusInterfaceType,
        ParentBusNumber,
        BusSpecificData,
        None,
        &mut Bus,
    );
    
    if Bus.is_null() {
        return core::ptr::null_mut();
    }

    if InterfaceType != _INTERFACE_TYPE_InterfaceTypeUndefined {
        (*Bus).BusAddresses = ExAllocatePoolWithTag(
            NonPagedPoolMustSucceed,
            core::mem::size_of::<SUPPORTED_RANGES>(),
            TAG_HAL,
        ) as *mut SUPPORTED_RANGES;
        
        core::ptr::write_bytes(
            (*Bus).BusAddresses as *mut u8,
            0,
            core::mem::size_of::<SUPPORTED_RANGES>(),
        );

        (*(*Bus).BusAddresses).Version = HAL_SUPPORTED_RANGE_VERSION as u16;
        (*(*Bus).BusAddresses).Dma.Limit = 7;
        (*(*Bus).BusAddresses).Memory.Limit = 0xFFFFFFFF;
        (*(*Bus).BusAddresses).IO.Limit = 0xFFFF;
        (*(*Bus).BusAddresses).IO.SystemAddressSpace = 1;
        (*(*Bus).BusAddresses).PrefetchMemory.Base = 1;
    }

    Bus
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpRegisterInternalBusHandlers() {
    unsafe {
        if (*KeGetCurrentPrcb()).Number != 0 {
            return;
        }

        HalpInitBusHandler();

        let mut bus = HalpAllocateBusHandler(
            _INTERFACE_TYPE_Internal,
            _BUS_DATA_TYPE_ConfigurationSpaceUndefined,
            0,
            _INTERFACE_TYPE_InterfaceTypeUndefined,
            0,
            0,
        );
        if !bus.is_null() {
            (*bus).GetInterruptVector = Some(core::mem::transmute(HalpGetSystemInterruptVector as usize));
            (*bus).TranslateBusAddress = Some(core::mem::transmute(HalpTranslateSystemBusAddress as usize));
        }

        bus = HalpAllocateBusHandler(
            _INTERFACE_TYPE_InterfaceTypeUndefined,
            _BUS_DATA_TYPE_Cmos,
            0,
            _INTERFACE_TYPE_InterfaceTypeUndefined,
            0,
            0,
        );
        if !bus.is_null() {
            (*bus).GetBusData = Some(core::mem::transmute(HalpcGetCmosData as usize));
            (*bus).SetBusData = Some(core::mem::transmute(HalpcSetCmosData as usize));
        }

        bus = HalpAllocateBusHandler(
            _INTERFACE_TYPE_InterfaceTypeUndefined,
            _BUS_DATA_TYPE_Cmos,
            1,
            _INTERFACE_TYPE_InterfaceTypeUndefined,
            0,
            0,
        );
        if !bus.is_null() {
            (*bus).GetBusData = Some(core::mem::transmute(HalpcGetCmosData as usize));
            (*bus).SetBusData = Some(core::mem::transmute(HalpcSetCmosData as usize));
        }

        bus = HalpAllocateBusHandler(
            _INTERFACE_TYPE_Isa,
            _BUS_DATA_TYPE_ConfigurationSpaceUndefined,
            0,
            _INTERFACE_TYPE_Internal,
            0,
            0,
        );
        if !bus.is_null() {
            (*bus).GetBusData = Some(core::mem::transmute(HalpNoBusData as usize));
            (*(*bus).BusAddresses).Memory.Limit = 0xFFFFFF;
            (*bus).TranslateBusAddress = Some(core::mem::transmute(HalpTranslateIsaBusAddress as usize));
        }
    }
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpMarkChipsetDecode(OverrideEnable: BOOLEAN) -> NTSTATUS {
    let mut KeyString: _UNICODE_STRING = core::mem::zeroed();
    let Data: ULONG = OverrideEnable as ULONG;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Handle: HANDLE = core::ptr::null_mut();

    // Open CCS key
    const CCS_PATH: &[u16] = &[
        b'\\' as u16, b'R' as u16, b'E' as u16, b'G' as u16, b'I' as u16, b'S' as u16,
        b'T' as u16, b'R' as u16, b'Y' as u16, b'\\' as u16, b'M' as u16, b'A' as u16,
        b'C' as u16, b'H' as u16, b'I' as u16, b'N' as u16, b'E' as u16, b'\\' as u16,
        b'S' as u16, b'Y' as u16, b'S' as u16, b'T' as u16, b'E' as u16, b'M' as u16,
        b'\\' as u16, b'C' as u16, b'U' as u16, b'R' as u16, b'R' as u16, b'E' as u16,
        b'N' as u16, b'T' as u16, b'C' as u16, b'O' as u16, b'N' as u16, b'T' as u16,
        b'R' as u16, b'O' as u16, b'L' as u16, b'S' as u16, b'E' as u16, b'T' as u16,
        0,
    ];
    KeyString.Buffer = CCS_PATH.as_ptr() as *mut u16;
    KeyString.Length = ((CCS_PATH.len() - 1) * 2) as u16;
    KeyString.MaximumLength = (CCS_PATH.len() * 2) as u16;

    let mut Status = HalpOpenRegistryKey(
        &raw mut Handle,
        core::ptr::null_mut(),
        &raw mut KeyString,
        KEY_ALL_ACCESS,
        0,
    );

    if Status >= 0 {
        // Open PNP Bios key
        const PNPBIOS_PATH: &[u16] = &[
            b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16,
            b'l' as u16, b'\\' as u16, b'B' as u16, b'i' as u16, b'o' as u16, b's' as u16,
            b'i' as u16, b'n' as u16, b'f' as u16, b'o' as u16, b'\\' as u16, b'P' as u16,
            b'N' as u16, b'P' as u16, b'B' as u16, b'i' as u16, b'o' as u16, b's' as u16,
            0,
        ];
        KeyString.Buffer = PNPBIOS_PATH.as_ptr() as *mut u16;
        KeyString.Length = ((PNPBIOS_PATH.len() - 1) * 2) as u16;
        KeyString.MaximumLength = (PNPBIOS_PATH.len() * 2) as u16;

        Status = HalpOpenRegistryKey(
            &raw mut KeyHandle,
            Handle,
            &raw mut KeyString,
            KEY_ALL_ACCESS,
            1,
        );

        // Close root key
        ZwClose(Handle);

        // Check if PNP BIOS key exists
        if Status >= 0 {
            // Set the override value
            const OVERRIDE_NAME: &[u16] = &[
                b'F' as u16, b'u' as u16, b'l' as u16, b'l' as u16, b'D' as u16, b'e' as u16,
                b'c' as u16, b'o' as u16, b'd' as u16, b'e' as u16, b'C' as u16, b'h' as u16,
                b'i' as u16, b'p' as u16, b's' as u16, b'e' as u16, b't' as u16, b'O' as u16,
                b'v' as u16, b'e' as u16, b'r' as u16, b'r' as u16, b'i' as u16, b'd' as u16,
                b'e' as u16, 0,
            ];
            KeyString.Buffer = OVERRIDE_NAME.as_ptr() as *mut u16;
            KeyString.Length = ((OVERRIDE_NAME.len() - 1) * 2) as u16;
            KeyString.MaximumLength = (OVERRIDE_NAME.len() * 2) as u16;

            Status = ZwSetValueKey(
                KeyHandle,
                &raw mut KeyString,
                0,
                REG_DWORD,
                &raw const Data as *mut _,
                core::mem::size_of::<ULONG>() as ULONG,
            );

            // Close subkey
            ZwClose(KeyHandle);
        }
    }

    // Return status
    Status
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpAllocateAndInitPciBusHandler(
    PciType: ULONG,
    BusNo: ULONG,
    TestAllocation: BOOLEAN,
) -> PBUS_HANDLER {
    unsafe extern "C" {
        fn DbgPrint(Format: *const u8, ...) -> NTSTATUS;
    }

    let Bus: PBUS_HANDLER;
    let BusData: PPCIPBUSDATA;

    Bus = HalpAllocateBusHandler(
        _INTERFACE_TYPE_PCIBus,
        _BUS_DATA_TYPE_PCIConfiguration,
        BusNo,
        _INTERFACE_TYPE_Internal,
        0,
        core::mem::size_of::<PCIPBUSDATA>() as ULONG,
    );

    (*Bus).GetBusData = Some(core::mem::transmute(HalpGetPCIData as usize));
    (*Bus).SetBusData = Some(core::mem::transmute(HalpSetPCIData as usize));
    (*Bus).GetInterruptVector = Some(core::mem::transmute(HalpGetPCIIntOnISABus as usize));
    (*Bus).AdjustResourceList = Some(core::mem::transmute(HalpAdjustPCIResourceList as usize));
    (*Bus).AssignSlotResources = Some(core::mem::transmute(HalpAssignPCISlotResources as usize));
    (*(*Bus).BusAddresses).Dma.Limit = 0;

    BusData = (*Bus).BusData as PPCIPBUSDATA;

    (*BusData).CommonData.Tag = PCI_DATA_TAG;
    (*BusData).CommonData.Version = PCI_DATA_VERSION;
    (*BusData).CommonData.ReadConfig = Some(core::mem::transmute(HalpReadPCIConfig as usize));
    (*BusData).CommonData.WriteConfig = Some(core::mem::transmute(HalpWritePCIConfig as usize));
    (*BusData).CommonData.Pin2Line = Some(core::mem::transmute(HalpPCIPin2ISALine as usize));
    (*BusData).CommonData.Line2Pin = Some(core::mem::transmute(HalpPCIISALine2Pin as usize));
    (*BusData).MaxDevice = PCI_MAX_DEVICES;
    (*BusData).GetIrqRange = Some(core::mem::transmute(HalpGetISAFixedPCIIrq as usize));

    RtlInitializeBitMap(
        &raw mut (*BusData).DeviceConfigured as PVOID,
        (*BusData).ConfiguredBits.as_mut_ptr(),
        256,
    );

    match PciType {
        1 => {
            core::ptr::copy_nonoverlapping(
                &raw const PCIConfigHandlerType1 as *const u8,
                &raw mut PCIConfigHandler as *mut u8,
                core::mem::size_of_val(&PCIConfigHandler),
            );

            (*BusData).Config.Type1.Address = PCI_TYPE1_ADDRESS_PORT as *mut ULONG;
            (*BusData).Config.Type1.Data = PCI_TYPE1_DATA_PORT as ULONG;
        }
        2 => {
            core::ptr::copy_nonoverlapping(
                &raw const PCIConfigHandlerType2 as *const u8,
                &raw mut PCIConfigHandler as *mut u8,
                core::mem::size_of_val(&PCIConfigHandler),
            );

            (*BusData).Config.Type2.CSE = PCI_TYPE2_CSE_PORT as *mut UCHAR;
            (*BusData).Config.Type2.Forward = PCI_TYPE2_FORWARD_PORT as *mut UCHAR;
            (*BusData).Config.Type2.Base = PCI_TYPE2_ADDRESS_BASE as ULONG;

            (*BusData).MaxDevice = 16;
        }
        _ => {
            DbgPrint(b"HAL: Unknown PCI type\n\0".as_ptr());
        }
    }

    Bus
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpIsValidPCIDevice(
    BusHandler: PBUS_HANDLER,
    Slot: PCI_SLOT_NUMBER,
) -> BOOLEAN {
    

    let mut DataBuffer: [u8; 64] = core::mem::zeroed();
    let PciHeader: PPCI_COMMON_CONFIG = DataBuffer.as_mut_ptr() as PPCI_COMMON_CONFIG;

    HalpReadPCIConfig(BusHandler, Slot, PciHeader as *mut core::ffi::c_void, 0, 64);

    if (*PciHeader).VendorID == PCI_INVALID_VENDORID as u16 ||
       PCI_CONFIGURATION_TYPE(PciHeader) != PCI_DEVICE_TYPE
    {
        return 0;
    }

    if ((*PciHeader).u.type0.InterruptPin != 0 &&
        (*PciHeader).u.type0.InterruptPin > 4) ||
       ((*PciHeader).u.type0.InterruptLine & 0x70) != 0
    {
        return 0;
    }

    let mut i: ULONG = 0;
    while i < PCI_TYPE0_ADDRESSES {
        let Address: ULONG_PTR = (*PciHeader).u.type0.BaseAddresses[i as usize] as ULONG_PTR;
        
        if (Address & PCI_ADDRESS_IO_SPACE as ULONG_PTR) != 0 {
            if Address > 0xFFFF {
                return 0;
            }
        } else {
            if Address > 0xF && Address < 0x80000 {
                return 0;
            }
        }

        if (Address & PCI_ADDRESS_IO_SPACE as ULONG_PTR) == 0 &&
           (Address & PCI_ADDRESS_MEMORY_TYPE_MASK as ULONG_PTR) == PCI_TYPE_64BIT as ULONG_PTR
        {
            i += 1;
        }

        i += 1;
    }

    1
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpGetChipHacks(
    VendorId: USHORT,
    DeviceId: USHORT,
    RevisionId: UCHAR,
    HackFlags: *mut ULONG,
) -> NTSTATUS {
    unsafe extern "C" {
        fn DbgPrint(Format: *const u8, ...) -> ULONG;
    }

    #[inline]
    unsafe fn halp_hack_flags(flags: ULONG) -> ULONG {
        flags & 0xFFFF
    }

    #[inline]
    unsafe fn halp_revision_from_hack_flags(flags: ULONG) -> ULONG {
        (flags >> 16) & 0xFF
    }

    #[inline]
    unsafe fn halp_revision_hack_flags(flags: ULONG) -> ULONG {
        (flags >> 24) & 0xFF
    }

    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Buffer: [u16; 32] = [0; 32];
    let mut PartialInfo: KEY_VALUE_PARTIAL_INFORMATION = core::mem::zeroed();
    let ResultLength: *mut ULONG = &mut 0;

    let key_str: &[u16] = &[
        0x005C, 0x0052, 0x0045, 0x0047, 0x0049, 0x0053, 0x0054, 0x0052,
        0x0059, 0x005C, 0x004D, 0x0041, 0x0043, 0x0048, 0x0049, 0x004E,
        0x0045, 0x005C, 0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D,
        0x005C, 0x0043, 0x0055, 0x0052, 0x0052, 0x0045, 0x004E, 0x0054,
        0x0043, 0x004F, 0x004E, 0x0054, 0x0052, 0x004F, 0x004C, 0x0053,
        0x0045, 0x0054, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072,
        0x006F, 0x006C, 0x005C, 0x0048, 0x0041, 0x004C, 0x0000,
    ];

    RtlInitUnicodeString(&raw mut KeyName, key_str.as_ptr());

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &raw mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let mut Status = ZwOpenKey(&raw mut KeyHandle, KEY_READ, &raw mut ObjectAttributes);
    if Status < 0 {
        return Status;
    }

    let fmt: &[u16] = &[
        0x0025, 0x0030, 0x0034, 0x0058, 0x0025, 0x0030, 0x0034, 0x0058, 0x0000,
    ];
    
    unsafe extern "C" {
        fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
    }
    
    swprintf(Buffer.as_mut_ptr(), fmt.as_ptr(), VendorId as u32, DeviceId as u32);
    
    RtlInitUnicodeString(&raw mut ValueName, Buffer.as_ptr());
    
    Status = ZwQueryValueKey(
        KeyHandle,
        &raw mut ValueName,
        1,
        &raw mut PartialInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG,
        ResultLength,
    );
    
    if Status >= 0 {
        let msg1: &[u8] = b"\tFound HackFlags for your chipset\n\0";
        DbgPrint(msg1.as_ptr());
        
        *HackFlags = *(PartialInfo.Data.as_ptr() as *const ULONG);
        
        let msg2: &[u8] = b"\t\tHack Flags: %lx (Hack Revision: %lx-Your Revision: %lx)\n\0";
        DbgPrint(
            msg2.as_ptr(),
            *HackFlags,
            halp_revision_from_hack_flags(*HackFlags),
            RevisionId as ULONG,
        );

        if (RevisionId != 0) && (RevisionId as ULONG >= halp_revision_from_hack_flags(*HackFlags)) {
            *HackFlags = halp_revision_hack_flags(*HackFlags);
        }

        *HackFlags = halp_hack_flags(*HackFlags);
        
        if *HackFlags == 0 {
            let msg3: &[u8] = b"\tNo HackFlags for your chipset's revision!\n\0";
            DbgPrint(msg3.as_ptr());
        }
    }

    ZwClose(KeyHandle);
    Status
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpIsRecognizedCard(
    PciRegistryInfo: PPCI_REGISTRY_INFO_INTERNAL,
    PciData: PPCI_COMMON_CONFIG,
    Flags: ULONG,
) -> BOOLEAN {
    let mut ElementCount: ULONG;
    let mut i: ULONG;
    let mut CardDescriptor: PPCI_CARD_DESCRIPTOR;
    let mut PciData = PciData;

    

    ElementCount = (*PciRegistryInfo).ElementCount;
    if ElementCount == 0 {
        return 0;
    }

    CardDescriptor = (&raw mut (*PciRegistryInfo).CardList).cast::<_PCI_CARD_DESCRIPTOR>().add(0);
    i = 0;
    while i < ElementCount {
        if (*CardDescriptor).Flags != Flags {
            CardDescriptor = CardDescriptor.add(1);
            i += 1;
            continue;
        }

        if ((*CardDescriptor).VendorID != (*PciData).VendorID) ||
           ((*CardDescriptor).DeviceID != (*PciData).DeviceID)
        {
            CardDescriptor = CardDescriptor.add(1);
            i += 1;
            continue;
        }

        if ((*CardDescriptor).Flags & HALP_CHECK_CARD_REVISION_ID) != 0 &&
           ((*CardDescriptor).RevisionID != (*PciData).RevisionID as u16)
        {
            CardDescriptor = CardDescriptor.add(1);
            i += 1;
            continue;
        }

        match unsafe { PCI_CONFIGURATION_TYPE(PciData) } {
            x if x == PCI_CARDBUS_BRIDGE_TYPE => {
                PciData = (PciData as *mut u8).add(core::mem::size_of::<_PCI_COMMON_CONFIG>()) as PPCI_COMMON_CONFIG;

                if ((*CardDescriptor).Flags & HALP_CHECK_CARD_SUBVENDOR_ID) != 0 &&
                   ((*CardDescriptor).SubsystemVendorID != (*PciData).u.type0.SubVendorID)
                {
                    CardDescriptor = CardDescriptor.add(1);
                    i += 1;
                    continue;
                }

                if ((*CardDescriptor).Flags & HALP_CHECK_CARD_SUBSYSTEM_ID) != 0 &&
                   ((*CardDescriptor).SubsystemID != (*PciData).u.type0.SubSystemID)
                {
                    CardDescriptor = CardDescriptor.add(1);
                    i += 1;
                    continue;
                }

                return 1;
            }
            x if x == PCI_DEVICE_TYPE => {
                if ((*CardDescriptor).Flags & HALP_CHECK_CARD_SUBVENDOR_ID) != 0 &&
                   ((*CardDescriptor).SubsystemVendorID != (*PciData).u.type0.SubVendorID)
                {
                    CardDescriptor = CardDescriptor.add(1);
                    i += 1;
                    continue;
                }

                if ((*CardDescriptor).Flags & HALP_CHECK_CARD_SUBSYSTEM_ID) != 0 &&
                   ((*CardDescriptor).SubsystemID != (*PciData).u.type0.SubSystemID)
                {
                    CardDescriptor = CardDescriptor.add(1);
                    i += 1;
                    continue;
                }

                return 1;
            }
            x if x == PCI_BRIDGE_TYPE => {
                return 1;
            }
            _ => {
                return 1;
            }
        }
    }

    0
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpIsIdeDevice(
    PciData: PPCI_COMMON_CONFIG,
) -> BOOLEAN {
    if (*PciData).BaseClass == PCI_CLASS_MASS_STORAGE_CTLR as u8 &&
       (*PciData).SubClass == PCI_SUBCLASS_MSC_IDE_CTLR as u8
    {
        return 1;
    }

    if (*PciData).VendorID == 0x1C1C {
        return 1;
    }

    if (*PciData).VendorID == 0x10B9 &&
       ((*PciData).DeviceID == 0x5215 || (*PciData).DeviceID == 0x5219)
    {
        return 1;
    }

    if (*PciData).VendorID == 0x1097 && (*PciData).DeviceID == 0x38 {
        return 1;
    }

    if (*PciData).VendorID == 0xE11 && (*PciData).DeviceID == 0xAE33 {
        return 1;
    }

    if (*PciData).VendorID == 0x1042 && (*PciData).DeviceID == 0x1000 {
        return 1;
    }

    if (*PciData).VendorID == 0x1039 &&
       ((*PciData).DeviceID == 0x601 || (*PciData).DeviceID == 0x5513)
    {
        return 1;
    }

    if (*PciData).VendorID == 0x10AD &&
       ((*PciData).DeviceID == 0x1 || (*PciData).DeviceID == 0x150)
    {
        return 1;
    }

    if (*PciData).VendorID == 0x1060 && (*PciData).DeviceID == 0x101 {
        return 1;
    }

    0
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpIsBridgeDevice(PciData: PPCI_COMMON_CONFIG) -> BOOLEAN {
    

    ((PCI_CONFIGURATION_TYPE(PciData) == PCI_BRIDGE_TYPE &&
      (*PciData).BaseClass == PCI_CLASS_BRIDGE_DEV as u8 &&
      (*PciData).SubClass == PCI_SUBCLASS_BR_PCI_TO_PCI as u8) ||
     (PCI_CONFIGURATION_TYPE(PciData) == PCI_CARDBUS_BRIDGE_TYPE &&
      (*PciData).BaseClass == PCI_CLASS_BRIDGE_DEV as u8 &&
      (*PciData).SubClass == PCI_SUBCLASS_BR_CARDBUS as u8)) as BOOLEAN
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpGetPciBridgeConfig(
    PciType: ULONG,
    BusCount: *mut USHORT,
) -> BOOLEAN {
    

    let mut data_buffer: [u8; PCI_COMMON_HDR_LENGTH as usize] = core::mem::zeroed();
    let pci_data: PPCI_COMMON_CONFIG = data_buffer.as_mut_ptr() as PPCI_COMMON_CONFIG;
    let mut pci_slot: PCI_SLOT_NUMBER = core::mem::zeroed();

    pci_slot.u.bits.set_Reserved(0);

    for i in 0..(*BusCount as ULONG) {
        let bus_handler = HalPrivateDispatchTable.HalHandlerForBus.unwrap()(PCIBus, i);

        for j in 0..PCI_MAX_DEVICES {
            pci_slot.u.bits.set_DeviceNumber(j as u32);

            for k in 0..PCI_MAX_FUNCTION {
                pci_slot.u.bits.set_FunctionNumber(k as u32);

                HalpReadPCIConfig(
                    bus_handler,
                    pci_slot,
                    pci_data as *mut core::ffi::c_void,
                    0,
                    PCI_COMMON_HDR_LENGTH as ULONG,
                );

                if (*pci_data).VendorID == PCI_INVALID_VENDORID as u16 {
                    continue;
                }

                if HalpIsBridgeDevice(pci_data) == 0 {
                    continue;
                }

                continue;
            }
        }
    }

    0
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpFixupPciSupportedRanges(BusCount: ULONG) {
    

    let mut i: ULONG;
    let mut Bus: PBUS_HANDLER;
    let mut ParentBus: PBUS_HANDLER;

    i = 0;
    while i < BusCount {
        Bus = HalPrivateDispatchTable.HalHandlerForBus.unwrap()(PCIBus, i);

        ParentBus = (*Bus).ParentHandler;
        while !ParentBus.is_null() {
            if WarningsGiven[0] == 0 {
                WarningsGiven[0] += 1;
            }

            ParentBus = (*ParentBus).ParentHandler;
        }

        i += 1;
    }

    i = 0;
    while i < BusCount {
        Bus = HalPrivateDispatchTable.HalHandlerForBus.unwrap()(PCIBus, i);

        if (*((*Bus).BusData as PPCIPBUSDATA)).Subtractive == 0 {
            ParentBus = (*Bus).ParentHandler;
            while !ParentBus.is_null() {
                if (*ParentBus).InterfaceType == PCIBus {
                    if WarningsGiven[1] == 0 {
                        WarningsGiven[1] += 1;
                    }
                }

                ParentBus = (*ParentBus).ParentHandler;
            }
        }

        i += 1;
    }

    i = 0;
    while i < BusCount {
        Bus = HalPrivateDispatchTable.HalHandlerForBus.unwrap()(PCIBus, i);

        i += 1;
    }
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn ShowSize(x: ULONG) {
    if x == 0 {
        return;
    }
    
    unsafe extern "C" {
        fn DbgPrint(Format: *const u8, ...) -> ULONG;
    }
    
    unsafe {
        DbgPrint(b" [size=\0".as_ptr());
        if x < 1024 {
            DbgPrint(b"%d\0".as_ptr(), x as i32);
        } else if x < 1048576 {
            DbgPrint(b"%dK\0".as_ptr(), (x / 1024) as i32);
        } else if x < 0x80000000 {
            DbgPrint(b"%dM\0".as_ptr(), (x / 1048576) as i32);
        } else {
            DbgPrint(b"%d\0".as_ptr(), x);
        }
        DbgPrint(b"]\0".as_ptr());
    }
}

// PCI vendor/class name tables (INITDATA segment, NUL-terminated).
// i386: defined in pcidata.rs via include_bytes! of the .ids files.
// amd64: never compiled (no legacy.cmake on amd64) — the references below
// sit on a debug path the linker dead-strips, so a bare extern decl with
// no definition is the existing, working arrangement; keep it identical.
#[cfg(target_arch = "x86")]
use crate::x86::pcidata::{ClassTable, VendorTable};
#[cfg(target_arch = "x86_64")]
unsafe extern "C" {
    static ClassTable: [u8; 0];
    static VendorTable: [u8; 0];
}

/// `strlen` for a NUL-terminated byte string.
unsafe fn byte_strlen(s: *const u8) -> usize {
    let mut n = 0;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// `strstr(hay, needle)` — both NUL-terminated. Returns NULL if not found.
unsafe fn byte_strstr(hay: *const u8, needle: *const u8) -> *const u8 {
    let nlen = byte_strlen(needle);
    if nlen == 0 {
        return hay;
    }
    let mut p = hay;
    loop {
        if *p == 0 {
            return core::ptr::null();
        }
        let mut k = 0;
        while k < nlen && *p.add(k) != 0 && *p.add(k) == *needle.add(k) {
            k += 1;
        }
        if k == nlen {
            return p;
        }
        p = p.add(1);
    }
}

/// `strpbrk(s, "\n")` — returns pointer to first '\n' in s, or NULL.
/// bussupp.c only ever uses strpbrk against the single-char NEWLINE "\n".
unsafe fn byte_find_newline(s: *const u8) -> *const u8 {
    let mut p = s;
    while *p != 0 {
        if *p == b'\n' {
            return p;
        }
        p = p.add(1);
    }
    core::ptr::null()
}

/// `strncpy(dst, src, n)` then NUL-terminate at position n.
unsafe fn byte_strncpy(dst: *mut u8, src: *const u8, n: usize) {
    for i in 0..n {
        *dst.add(i) = *src.add(i);
    }
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpDebugPciDumpBus(
    BusHandler: PBUS_HANDLER,
    PciSlot: PCI_SLOT_NUMBER,
    i: ULONG,
    j: ULONG,
    k: ULONG,
    PciData: PPCI_COMMON_CONFIG,
) {
    unsafe extern "C" {
        fn DbgPrint(Format: *const u8, ...) -> ULONG;
        fn sprintf(Buffer: *mut u8, Format: *const u8, ...) -> i32;
    }
    unsafe extern "system" {
        fn HalpReadPCIConfig(
            BusHandler: PBUS_HANDLER,
            Slot: PCI_SLOT_NUMBER,
            Buffer: PVOID,
            Offset: ULONG,
            Length: ULONG,
        );
        fn HalpWritePCIConfig(
            BusHandler: PBUS_HANDLER,
            Slot: PCI_SLOT_NUMBER,
            Buffer: PVOID,
            Offset: ULONG,
            Length: ULONG,
        );
    }

    let header_type = ((*PciData).HeaderType & !(PCI_MULTIFUNCTION as u8)) as u8;

    let mut lookup: [u8; 16] = [0; 16];
    let mut b_subclass: [u8; 64] = *b"Unknown\0                                                        ";
    let mut b_vendor: [u8; 64] = [0; 64];
    let mut b_product: [u8; 128] = *b"Unknown device\0                                                                                                                 ";
    let mut b_subvendor: [u8; 128] = *b"Unknown\0                                                                                                                        ";

    let class_table = (&raw const ClassTable) as *const u8;
    let vendor_table = (&raw const VendorTable) as *const u8;

    // --- Isolate the class/subclass name ---
    sprintf(
        lookup.as_mut_ptr(),
        b"C %02x  \0".as_ptr(),
        (*PciData).BaseClass as i32,
    );
    let class_name = byte_strstr(class_table, lookup.as_ptr());
    if !class_name.is_null() {
        // Walk past "C 00  " (6 bytes) to the line content.
        let class_name = class_name.add(b"C 00  ".len());
        let boundary = byte_strstr(class_name, b"\nC \0".as_ptr());
        sprintf(
            lookup.as_mut_ptr(),
            b"\n\t%02x  \0".as_ptr(),
            (*PciData).SubClass as i32,
        );
        let mut subclass_name = byte_strstr(class_name, lookup.as_ptr());
        if !boundary.is_null() && !subclass_name.is_null() && subclass_name >= boundary {
            subclass_name = core::ptr::null();
        }
        let subclass_name = if subclass_name.is_null() {
            class_name
        } else {
            subclass_name.add(b"\n\t00  ".len())
        };
        let p = byte_find_newline(subclass_name);
        if !p.is_null() {
            let mut len = p as usize - subclass_name as usize;
            if len > b_subclass.len() - 1 {
                len = b_subclass.len() - 1;
            }
            byte_strncpy(b_subclass.as_mut_ptr(), subclass_name, len);
            b_subclass[len] = 0;
        }
    }

    // --- Isolate the vendor / product / subvendor names ---
    sprintf(
        lookup.as_mut_ptr(),
        b"\n%04x  \0".as_ptr(),
        (*PciData).VendorID as i32,
    );
    let vendor_name = byte_strstr(vendor_table, lookup.as_ptr());
    if !vendor_name.is_null() {
        let vendor_name = vendor_name.add(b"\n0000  ".len());
        let p = byte_find_newline(vendor_name);
        if !p.is_null() {
            let mut len = p as usize - vendor_name as usize;
            if len > b_vendor.len() - 1 {
                len = b_vendor.len() - 1;
            }
            byte_strncpy(b_vendor.as_mut_ptr(), vendor_name, len);
            b_vendor[len] = 0;
        }
        // Walk past device comments ('\t' or '#' lines) to find boundary
        // (next vendor block).
        let mut p = if p.is_null() { core::ptr::null() } else { p.add(1) };
        while !p.is_null() && (*p == b'\t' || *p == b'#') {
            p = byte_find_newline(p);
            if !p.is_null() {
                p = p.add(1);
            }
        }
        let boundary = p;

        // --- Product name lookup ---
        sprintf(
            lookup.as_mut_ptr(),
            b"\t%04x  \0".as_ptr(),
            (*PciData).DeviceID as i32,
        );
        let mut product_name = byte_strstr(vendor_name, lookup.as_ptr());
        if !boundary.is_null() && !product_name.is_null() && product_name >= boundary {
            product_name = core::ptr::null();
        }
        if !product_name.is_null() {
            let product_name = product_name.add(b"\t0000  ".len());
            let p = byte_find_newline(product_name);
            if !p.is_null() {
                let mut len = p as usize - product_name as usize;
                if len > b_product.len() - 1 {
                    len = b_product.len() - 1;
                }
                byte_strncpy(b_product.as_mut_ptr(), product_name, len);
                b_product[len] = 0;
            }
            // Skip over subsystem lines that aren't ours.
            let mut p = if p.is_null() { core::ptr::null() } else { p.add(1) };
            while !p.is_null() && ((*p == b'\t' && *p.add(1) == b'\t') || *p == b'#') {
                p = byte_find_newline(p);
                if !p.is_null() {
                    p = p.add(1);
                }
            }
            let inner_boundary = p;

            // --- Subvendor / subsystem name (PCI type-0 devices only) ---
            if header_type == PCI_DEVICE_TYPE as u8 {
                sprintf(
                    lookup.as_mut_ptr(),
                    b"\t\t%04x %04x  \0".as_ptr(),
                    (*PciData).u.type0.SubVendorID as i32,
                    (*PciData).u.type0.SubSystemID as i32,
                );
                let mut subvendor_name = byte_strstr(product_name, lookup.as_ptr());
                if !inner_boundary.is_null()
                    && !subvendor_name.is_null()
                    && subvendor_name >= inner_boundary
                {
                    subvendor_name = core::ptr::null();
                }
                if !subvendor_name.is_null() {
                    let subvendor_name = subvendor_name.add(b"\t\t0000 0000  ".len());
                    let p = byte_find_newline(subvendor_name);
                    if !p.is_null() {
                        let mut len = p as usize - subvendor_name as usize;
                        if len > b_subvendor.len() - 1 {
                            len = b_subvendor.len() - 1;
                        }
                        byte_strncpy(b_subvendor.as_mut_ptr(), subvendor_name, len);
                        b_subvendor[len] = 0;
                    }
                }
            }
        }
    }

    // --- Print out the data ---
    DbgPrint(
        b"%02x:%02x.%x %s [%02x%02x]: %s %s [%04x:%04x] (rev %02x)\n\0".as_ptr(),
        i,
        j,
        k,
        b_subclass.as_ptr(),
        (*PciData).BaseClass as i32,
        (*PciData).SubClass as i32,
        b_vendor.as_ptr(),
        b_product.as_ptr(),
        (*PciData).VendorID as i32,
        (*PciData).DeviceID as i32,
        (*PciData).RevisionID as i32,
    );

    if header_type == PCI_DEVICE_TYPE as u8 {
        DbgPrint(
            b"\tSubsystem: %s [%04x:%04x]\n\0".as_ptr(),
            b_subvendor.as_ptr(),
            (*PciData).u.type0.SubVendorID as i32,
            (*PciData).u.type0.SubSystemID as i32,
        );
    }

    // --- Flags ---
    DbgPrint(b"\tFlags:\0".as_ptr());
    if (*PciData).Command as u32 & PCI_ENABLE_BUS_MASTER != 0 {
        DbgPrint(b" bus master,\0".as_ptr());
    }
    if (*PciData).Status as u32 & PCI_STATUS_66MHZ_CAPABLE != 0 {
        DbgPrint(b" 66MHz,\0".as_ptr());
    }
    match (*PciData).Status as u32 & PCI_STATUS_DEVSEL {
        0x000 => { DbgPrint(b" fast devsel,\0".as_ptr()); }
        0x200 => { DbgPrint(b" medium devsel,\0".as_ptr()); }
        0x400 => { DbgPrint(b" slow devsel,\0".as_ptr()); }
        0x600 => { DbgPrint(b" unknown devsel,\0".as_ptr()); }
        _ => {}
    }
    DbgPrint(b" latency %d\0".as_ptr(), (*PciData).LatencyTimer as i32);
    if (*PciData).u.type0.InterruptPin != 0
        && (*PciData).u.type0.InterruptLine != 0
        && (*PciData).u.type0.InterruptLine != 0xFF
    {
        DbgPrint(b", IRQ %02d\0".as_ptr(), (*PciData).u.type0.InterruptLine as i32);
    } else if (*PciData).u.type0.InterruptPin != 0 {
        DbgPrint(b", IRQ assignment required\0".as_ptr());
    }
    DbgPrint(b"\n\0".as_ptr());

    if header_type == PCI_BRIDGE_TYPE as u8 {
        DbgPrint(b"\tBridge:\0".as_ptr());
        DbgPrint(
            b" primary bus %d,\0".as_ptr(),
            (*PciData).u.type1.PrimaryBus as i32,
        );
        DbgPrint(
            b" secondary bus %d,\0".as_ptr(),
            (*PciData).u.type1.SecondaryBus as i32,
        );
        DbgPrint(
            b" subordinate bus %d,\0".as_ptr(),
            (*PciData).u.type1.SubordinateBus as i32,
        );
        DbgPrint(
            b" secondary latency %d\0".as_ptr(),
            (*PciData).u.type1.SecondaryLatency as i32,
        );
        DbgPrint(b"\n\0".as_ptr());
    }

    // --- Scan addresses (BARs) ---
    let bar_count: u32 = if header_type == PCI_DEVICE_TYPE as u8 {
        PCI_TYPE0_ADDRESSES
    } else {
        PCI_TYPE1_ADDRESSES
    };
    // Field offset of u.type0.BaseAddresses[0] within PCI_COMMON_HEADER.
    let bar_field_offset =
        core::mem::offset_of!(_PCI_COMMON_CONFIG, u) + core::mem::offset_of!(PCI_COMMON_HEADER__u__type0, BaseAddresses);

    for b in 0..bar_count {
        let mut mem: ULONG = if header_type != PCI_CARDBUS_BRIDGE_TYPE as u8 {
            (*PciData).u.type0.BaseAddresses[b as usize]
        } else {
            0
        };
        if mem != 0 {
            let mut pci_bar: ULONG = 0xFFFFFFFF;
            let off = (bar_field_offset + (b as usize) * core::mem::size_of::<ULONG>()) as ULONG;
            HalpWritePCIConfig(
                BusHandler,
                PciSlot,
                &raw mut pci_bar as PVOID,
                off,
                core::mem::size_of::<ULONG>() as ULONG,
            );
            HalpReadPCIConfig(
                BusHandler,
                PciSlot,
                &raw mut pci_bar as PVOID,
                off,
                core::mem::size_of::<ULONG>() as ULONG,
            );
            HalpWritePCIConfig(
                BusHandler,
                PciSlot,
                &raw mut mem as PVOID,
                off,
                core::mem::size_of::<ULONG>() as ULONG,
            );

            // Decode the address type
            if pci_bar & PCI_ADDRESS_IO_SPACE != 0 {
                let mut size: ULONG = 1 << 2;
                while pci_bar & size == 0 && size != 0 {
                    size <<= 1;
                }
                DbgPrint(
                    b"\tI/O ports at %04lx\0".as_ptr(),
                    mem & PCI_ADDRESS_IO_ADDRESS_MASK,
                );
                ShowSize(size);
            } else {
                let mut size: ULONG = 1 << 4;
                while pci_bar & size == 0 && size != 0 {
                    size <<= 1;
                }
                let prefetch: *const u8 = if mem & PCI_ADDRESS_MEMORY_PREFETCHABLE != 0 {
                    b"\0".as_ptr()
                } else {
                    b"non-\0".as_ptr()
                };
                let bits: i32 = if mem & PCI_ADDRESS_MEMORY_TYPE_MASK == PCI_TYPE_32BIT as ULONG {
                    32
                } else {
                    64
                };
                DbgPrint(
                    b"\tMemory at %08lx (%d-bit, %sprefetchable)\0".as_ptr(),
                    mem & PCI_ADDRESS_MEMORY_ADDRESS_MASK,
                    bits,
                    prefetch,
                );
                ShowSize(size);
            }
            DbgPrint(b"\n\0".as_ptr());
        }
    }
}


// C bussupp.c:1020 HalpInitializePciBus is a common-region function whose
// ENTIRE body is `#ifndef _MINIHAL_` (bussupp.c:1025-1256). Under -D_MINIHAL_
// the C compiler emits the symbol with an EMPTY body. FreeLDR links the
// symbol, so the minihal build must export it (it is in the minihal-DEFINES
// set). Reproduce the C minihal face: empty body, references nothing.
// feedback_freeldr_clean_build_regression
#[cfg(feature = "minihal")]
#[no_mangle]
pub unsafe extern "system" fn HalpInitializePciBus() {}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpInitializePciBus() {
    unsafe extern "C" {
        fn DbgPrint(Format: *const u8, ...) -> NTSTATUS;
    }

    let PciRegistryInfo: PPCI_REGISTRY_INFO_INTERNAL;
    let mut PciType: UCHAR;
    let mut PciSlot: PCI_SLOT_NUMBER;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut k: ULONG;
    let mut DataBuffer: [u8; 64] = core::mem::zeroed();
    let PciData: PPCI_COMMON_CONFIG = DataBuffer.as_mut_ptr() as PPCI_COMMON_CONFIG;
    let mut BusHandler: PBUS_HANDLER;
    let mut HackFlags: ULONG = 0;
    let mut ExtendedAddressDecoding: BOOLEAN = 0;
    let mut Status: NTSTATUS;

    PciRegistryInfo = HalpQueryPciRegistryInfo();
    if PciRegistryInfo.is_null() {
        KeInitializeSpinLock(&raw mut HalpPCIConfigLock);
        DbgPrint(b"HAL: No PCI registry info, probing type 1 bus 0\n\0".as_ptr());
        HalpAllocateAndInitPciBusHandler(1, 0, 0);
        HalpFixupPciSupportedRanges(1);
        HalpGetNMICrashFlag();
        HalpMarkChipsetDecode(0);
        return;
    }

    KeInitializeSpinLock(&raw mut HalpPCIConfigLock);

    PciType = (*PciRegistryInfo).HardwareMechanism & 0xF;

    if ((*PciRegistryInfo).NoBuses != 0) && (PciType == 2) {
        PciSlot = core::mem::zeroed();
        PciSlot.u.bits.set_Reserved(0);
        PciSlot.u.bits.set_FunctionNumber(0);

        i = 0;
        while i < 32 {
            PciType = 2;
            BusHandler = HalpAllocateAndInitPciBusHandler(2, 0, 1);
            if BusHandler.is_null() {
                break;
            }

            if HalpIsValidPCIDevice(BusHandler, PciSlot) != 0 {
                break;
            }

            PciType = 1;
            BusHandler = HalpAllocateAndInitPciBusHandler(1, 0, 1);
            if BusHandler.is_null() {
                break;
            }

            if HalpIsValidPCIDevice(BusHandler, PciSlot) != 0 {
                break;
            }

            PciType = 2;
            i += 1;
        }

        HalpAllocateAndInitPciBusHandler(PciType as ULONG, 0, 0);
    }

    loop {
        i = 0;
        while i < (*PciRegistryInfo).NoBuses as ULONG {
            if HalPrivateDispatchTable.HalHandlerForBus.unwrap()(_INTERFACE_TYPE_PCIBus, i).is_null() {
                HalpAllocateAndInitPciBusHandler(PciType as ULONG, i, 0);
            }
            i += 1;
        }

        if HalpGetPciBridgeConfig(PciType as ULONG, &raw mut (*PciRegistryInfo).NoBuses) == 0 {
            break;
        }
    }

    HalpFixupPciSupportedRanges((*PciRegistryInfo).NoBuses as ULONG);

    DbgPrint(b"\n====== PCI BUS HARDWARE DETECTION =======\n\n\0".as_ptr());
    PciSlot = core::mem::zeroed();
    PciSlot.u.bits.set_Reserved(0);

    i = 0;
    while i < (*PciRegistryInfo).NoBuses as ULONG {
        BusHandler = HalPrivateDispatchTable.HalHandlerForBus.unwrap()(_INTERFACE_TYPE_PCIBus, i);

        j = 0;
        while j < 32 {
            PciSlot.u.bits.set_DeviceNumber(j as ULONG);

            k = 0;
            while k < 8 {
                PciSlot.u.bits.set_FunctionNumber(k as ULONG);

                HalpReadPCIConfig(
                    BusHandler,
                    PciSlot,
                    PciData as *mut core::ffi::c_void,
                    0,
                    64,
                );

                if (*PciData).VendorID == PCI_INVALID_VENDORID as u16 {
                    k += 1;
                    continue;
                }

                HalpDebugPciDumpBus(BusHandler, PciSlot, i, j, k, PciData);

                if PCI_CONFIGURATION_TYPE(PciData) == PCI_CARDBUS_BRIDGE_TYPE {
                    DbgPrint(b"\tDevice is a PCI Cardbus Bridge. It will not work!\n\0".as_ptr());
                    k += 1;
                    continue;
                }

                if PCI_CONFIGURATION_TYPE(PciData) != PCI_BRIDGE_TYPE {
                    if (((*PciData).u.type1.InterruptPin as u32) != 0) &&
                       (((*PciData).u.type1.InterruptLine as u32) != 0)
                    {
                        if ((*PciData).u.type1.InterruptLine as u32) < 16 {
                            if HalpIsIdeDevice(PciData) == 0 {
                                DbgPrint(
                                    b"\tDevice is using IRQ %d! ISA Cards using that IRQ may fail!\n\0".as_ptr(),
                                    ((*PciData).u.type1.InterruptLine as u32),
                                );
                                HalpPciIrqMask |= 1 << ((*PciData).u.type1.InterruptLine as u32);
                            }
                        }
                    }
                }

                if (*PciData).VendorID == 0x8086 {
                    if ((*PciData).DeviceID == 0x04A3) &&
                       ((*PciData).RevisionID < 0x11)
                    {
                        DbgPrint(b"\tDevice is a broken Intel 82430 PCI Controller. It will not work!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }

                    if ((*PciData).DeviceID == 0x0484) &&
                       ((*PciData).RevisionID <= 3)
                    {
                        DbgPrint(b"\tDevice is a broken Intel 82378 PCI-to-ISA Bridge. It will not work!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }

                    if ((*PciData).DeviceID == 0x84C4) &&
                       ((*PciData).RevisionID <= 4)
                    {
                        DbgPrint(b"\tDevice is a Intel Orion 82450 PCI Bridge. It will not work!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }
                }

                if ExtendedAddressDecoding == 0 {
                    if HalpIsRecognizedCard(
                        PciRegistryInfo,
                        PciData,
                        HALP_CARD_FEATURE_FULL_DECODE,
                    ) != 0
                    {
                        DbgPrint(b"\tDevice has Extended Address Decoding. It may fail to work on older BIOSes!\n\0".as_ptr());
                        ExtendedAddressDecoding = 1;
                    }
                }

                Status = HalpGetChipHacks(
                    (*PciData).VendorID,
                    (*PciData).DeviceID,
                    (*PciData).RevisionID,
                    &raw mut HackFlags,
                );

                if Status >= 0 {
                    if (HackFlags & HAL_PCI_CHIP_HACK_DISABLE_ACPI_IRQ_ROUTING) != 0 {
                        DbgPrint(b"This chipset has broken ACPI IRQ Routing! Be aware!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }

                    if (HackFlags & HAL_PCI_CHIP_HACK_BROKEN_ACPI_TIMER) != 0 {
                        DbgPrint(b"This chipset has a broken ACPI timer! Be aware!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }

                    if (HackFlags & HAL_PCI_CHIP_HACK_DISABLE_HIBERNATE) != 0 {
                        DbgPrint(b"This chipset has a broken PCI device which is incompatible with hibernation. Be aware!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }

                    if (HackFlags & HAL_PCI_CHIP_HACK_USB_SMI_DISABLE) != 0 {
                        DbgPrint(b"This chipset has a USB controller which generates SMIs. ReactOS will likely fail to boot!\n\n\0".as_ptr());
                        k += 1;
                        continue;
                    }
                }

                DbgPrint(b"\n\0".as_ptr());
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }

    HalpGetNMICrashFlag();

    ExFreePoolWithTag(PciRegistryInfo as PVOID, TAG_HAL);

    HalpMarkChipsetDecode(ExtendedAddressDecoding);
    DbgPrint(b"====== PCI BUS DETECTION COMPLETE =======\n\n\0".as_ptr());
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpInitBusHandlers() {
    HalpRegisterInternalBusHandlers();
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalpRegisterKdSupportFunctions() {
    unsafe {
        HalPrivateDispatchTable.KdSetupPciDeviceForDebugging =
            Some(core::mem::transmute(HalpSetupPciDeviceForDebugging as usize));
        HalPrivateDispatchTable.KdReleasePciDeviceforDebugging =
            Some(core::mem::transmute(HalpReleasePciDeviceForDebugging as usize));
        HalPrivateDispatchTable.KdMapPhysicalMemory64 =
            Some(core::mem::transmute(HalpMapPhysicalMemory64Vista as usize));
        HalPrivateDispatchTable.KdUnmapVirtualAddress =
            Some(core::mem::transmute(HalpUnmapVirtualAddressVista as usize));
        HalPrivateDispatchTable.KdCheckPowerButton =
            Some(core::mem::transmute(HalpCheckPowerButton as usize));
    }
}

// C common region — required in minihal (bound by HalAssignSlotResources
// call site in this module). feedback_freeldr_clean_build_regression
#[no_mangle]
pub unsafe extern "system" fn HalpAssignSlotResources(
    RegistryPath: PUNICODE_STRING,
    DriverClassName: PUNICODE_STRING,
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    BusType: INTERFACE_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    AllocatedResources: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    

    let Handler: PBUS_HANDLER;
    let Status: NTSTATUS;

    Handler = unsafe { HalDispatchTable.HalReferenceHandlerForBus.unwrap()(BusType, BusNumber) };
    if Handler.is_null() {
        return STATUS_NOT_FOUND;
    }

    Status = unsafe {
        ((*Handler).AssignSlotResources.unwrap())(
            Handler,
            Handler,
            RegistryPath,
            DriverClassName,
            DriverObject,
            DeviceObject,
            SlotNumber,
            AllocatedResources,
        )
    };

    unsafe { HalDispatchTable.HalDereferenceBusHandler.unwrap()(Handler) };
    Status
}

// feedback_freeldr_clean_build_regression: C bussupp.c:1329 HaliFindBusAddressTranslation
// is common-region (outside every #ifndef _MINIHAL_; the nearest guard #endif is at
// bussupp.c:1292, the next #ifndef _MINIHAL_ starts at 1444). HalpInitBusHandler
// (bushndlr.c:470, also common) references it unconditionally under -D_MINIHAL_, so the
// definition must exist in the minihal build too. Earlier translation incorrectly gated
// it #[cfg(not(feature = "minihal"))]; un-gate to mirror C truth (no minihal-only deps).
#[no_mangle]
pub unsafe extern "system" fn HaliFindBusAddressTranslation(
    BusAddress: PHYSICAL_ADDRESS,
    AddressSpace: *mut ULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
    Context: PULONG_PTR,
    NextBus: BOOLEAN,
) -> BOOLEAN {
    let mut BusHandler: PHAL_BUS_HANDLER;
    let mut Handler: PBUS_HANDLER;
    let mut NextEntry: PLIST_ENTRY;

    // Make sure we have a context
    if Context.is_null() {
        return 0;
    }

    // If we have data in the context, then this shouldn't be a new lookup
    if (*Context != 0) && (NextBus != 0) {
        return 0;
    }

    // Find the bus handler
    Handler = HalpContextToBusHandler(*Context);
    if Handler.is_null() {
        return 0;
    }

    // Check if this is an ongoing lookup
    if NextBus != 0 {
        // Get the HAL bus handler
        BusHandler = ((Handler as ULONG_PTR) - core::mem::offset_of!(HAL_BUS_HANDLER, Handler) as ULONG_PTR) as *mut HAL_BUS_HANDLER;
        NextEntry = &raw mut (*BusHandler).AllHandlers;

        // Get the next one if we were already with one
        if *Context != 0 {
            NextEntry = (*NextEntry).Flink;
        }

        // Start scanning
        loop {
            // Check if this is the last one
            if NextEntry == &raw mut HalpAllBusHandlers {
                // Quit
                *Context = 1;
                return 0;
            }

            // Call this translator
            BusHandler = ((NextEntry as ULONG_PTR) - core::mem::offset_of!(HAL_BUS_HANDLER, AllHandlers) as ULONG_PTR) as *mut HAL_BUS_HANDLER;
            if HalTranslateBusAddress(
                (*BusHandler).Handler.InterfaceType,
                (*BusHandler).Handler.BusNumber,
                BusAddress,
                AddressSpace,
                TranslatedAddress,
            ) != 0
            {
                break;
            }

            // Try the next one
            NextEntry = (*NextEntry).Flink;
        }

        // If we made it, we're done
        *Context = &raw mut (*BusHandler).Handler as ULONG_PTR;
        return 1;
    }

    // Try the first one through
    if HalTranslateBusAddress(
        (*Handler).InterfaceType,
        (*Handler).BusNumber,
        BusAddress,
        AddressSpace,
        TranslatedAddress,
    ) == 0
    {
        return 0;
    }

    // Remember for next time
    *Context = Handler as ULONG_PTR;
    return 1;
}

// C common region (bussupp.c after `#endif // _MINIHAL_`) — required in
// minihal (called from HalpInitBusHandler). feedback_freeldr_clean_build_regression
#[no_mangle]
pub unsafe extern "system" fn HaliTranslateBusAddress(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
    BusAddress: PHYSICAL_ADDRESS,
    AddressSpace: *mut ULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    let Handler: PBUS_HANDLER;
    let Status: BOOLEAN;

    Handler = unsafe { HalDispatchTable.HalReferenceHandlerForBus.unwrap()(InterfaceType, BusNumber) };
    if Handler.is_null() || unsafe { (*Handler).TranslateBusAddress.is_none() } {
        return 0;
    }

    Status = unsafe {
        ((*Handler).TranslateBusAddress.unwrap())(
            Handler,
            Handler,
            BusAddress,
            AddressSpace,
            TranslatedAddress,
        )
    };

    unsafe { HalDispatchTable.HalDereferenceBusHandler.unwrap()(Handler) };
    Status
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalAdjustResourceList(
    ResourceList: *mut PIO_RESOURCE_REQUIREMENTS_LIST,
) -> NTSTATUS {
    

    let handler: PBUS_HANDLER = unsafe {
        HalDispatchTable.HalReferenceHandlerForBus.unwrap()(
            (**ResourceList).InterfaceType,
            (**ResourceList).BusNumber,
        )
    };
    
    if handler.is_null() {
        return STATUS_SUCCESS;
    }

    let status: ULONG = unsafe {
        ((*handler).AdjustResourceList.unwrap())(handler, handler, ResourceList) as ULONG
    };

    unsafe { HalDispatchTable.HalDereferenceBusHandler.unwrap()(handler) };
    status as NTSTATUS
}

// C legacy/bussupp.c common region (after `#endif // _MINIHAL_`):
// compiled identically under -D_MINIHAL_. Must stay in the minihal
// feature — FreeLDR links it. See memory
// feedback_freeldr_clean_build_regression.
#[no_mangle]
pub unsafe extern "system" fn HalAssignSlotResources(
    RegistryPath: PUNICODE_STRING,
    DriverClassName: PUNICODE_STRING,
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    BusType: INTERFACE_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    AllocatedResources: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    

    if BusType != _INTERFACE_TYPE_PCIBus {
        HalpAssignSlotResources(
            RegistryPath,
            DriverClassName,
            DriverObject,
            DeviceObject,
            BusType,
            BusNumber,
            SlotNumber,
            AllocatedResources,
        )
    } else {
        HalPrivateDispatchTable.HalPciAssignSlotResources.unwrap()(
            RegistryPath,
            DriverClassName,
            DriverObject,
            DeviceObject,
            _INTERFACE_TYPE_PCIBus,
            BusNumber,
            SlotNumber,
            AllocatedResources,
        )
    }
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalGetBusData(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    unsafe {
        HalGetBusDataByOffset(
            BusDataType,
            BusNumber,
            SlotNumber,
            Buffer,
            0,
            Length,
        )
    }
}

// C common region — required in minihal (FreeLDR links it).
#[no_mangle]
pub unsafe extern "system" fn HalGetBusDataByOffset(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    

    let Handler: PBUS_HANDLER = HaliReferenceHandlerForConfigSpace(BusDataType, BusNumber);
    if Handler.is_null() {
        return 0;
    }

    let Status: ULONG = ((*Handler).GetBusData.unwrap())(
        Handler,
        Handler,
        SlotNumber,
        Buffer,
        Offset,
        Length,
    );

    HalDispatchTable.HalDereferenceBusHandler.unwrap()(Handler);
    Status
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalGetInterruptVector(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
    BusInterruptLevel: ULONG,
    BusInterruptVector: ULONG,
    Irql: PKIRQL,
    Affinity: PKAFFINITY,
) -> ULONG {
    unsafe extern "fastcall" {
        fn HalpVectorToIrql(Vector: UCHAR) -> KIRQL;
    }

    *Irql = 0;
    core::ptr::write_unaligned(Affinity, 0);

    // A message token is not a line on any bus: the HAL routes it (crate::message).
    if crate::message::is_message_token(BusInterruptVector) {
        return crate::message::allocate(Irql, Affinity);
    }

    let Handler = HalDispatchTable.HalReferenceHandlerForBus.unwrap()(InterfaceType, BusNumber);
    if Handler.is_null() {
        return 0;
    }

    let Vector = ((*Handler).GetInterruptVector.unwrap())(
        Handler,
        Handler,
        BusInterruptLevel,
        BusInterruptVector,
        Irql,
        Affinity,
    );

    HalDispatchTable.HalDereferenceBusHandler.unwrap()(Handler);
    Vector
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalSetBusData(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Length: ULONG,
) -> ULONG {
    HalSetBusDataByOffset(
        BusDataType,
        BusNumber,
        SlotNumber,
        Buffer,
        0,
        Length,
    )
}

#[cfg(not(feature = "minihal"))]
#[no_mangle]
pub unsafe extern "system" fn HalSetBusDataByOffset(
    BusDataType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    

    let Handler: PBUS_HANDLER;
    let Status: ULONG;

    Handler = HaliReferenceHandlerForConfigSpace(BusDataType, BusNumber);
    if Handler.is_null() {
        return 0;
    }

    Status = ((*Handler).SetBusData.unwrap())(
        Handler,
        Handler,
        SlotNumber,
        Buffer,
        Offset,
        Length,
    );

    HalDispatchTable.HalDereferenceBusHandler.unwrap()(Handler);
    Status
}

// C common region — required in minihal (FreeLDR links it).
#[no_mangle]
pub unsafe extern "system" fn HalTranslateBusAddress(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
    BusAddress: PHYSICAL_ADDRESS,
    AddressSpace: *mut ULONG,
    TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    if InterfaceType == 5 {
        unsafe {
            HalPrivateDispatchTable.HalPciTranslateBusAddress.unwrap()(
                5,
                BusNumber,
                BusAddress,
                AddressSpace,
                TranslatedAddress,
            )
        }
    } else {
        unsafe {
            HaliTranslateBusAddress(
                InterfaceType,
                BusNumber,
                BusAddress,
                AddressSpace,
                TranslatedAddress,
            )
        }
    }
}
