#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{PCM_RESOURCE_LIST};
use ntkrnl_sys::ddk::{PCSTR, RtlAssert};
use ntkrnl_sys::driver_ddk::{DbgPrint, swprintf};
use core::ffi::c_void;
use core::ffi::c_char;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut PciArbiterNames: [PCHAR; 4] = [
    b"I/O Port\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"Memory\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"Interrupt\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
    b"Bus Number\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
];

pub const NDEBUG: bool = true;
#[no_mangle]
pub unsafe extern "system" fn PciArbiterDestructor(Arbiter: PPCI_ARBITER_INSTANCE) {
    DbgPrint(b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR, b"PciArbiterDestructor\0".as_ptr() as *const c_char, b"drivers/bus/pcix/arb/arb_comn.c\0".as_ptr() as *const c_char, 34);
    loop {}
}
#[no_mangle]
pub unsafe extern "system" fn PciInitializeArbiters(
    FdoExtension: PPCI_FDO_EXTENSION,
) -> NTSTATUS {
    const FILE_PATH: &[u8] = b"drivers/bus/pcix/arb/arb_comn.c\0";
    const FORMAT_SWPRINTF: [u16; 16] = [
        0x50, 0x43, 0x49, 0x20, 0x25, 0x53, 0x20, 0x28,
        0x62, 0x3D, 0x25, 0x30, 0x32, 0x78, 0x29, 0x00,
    ];
    const ARBITER_HEADER_MISSING: [u16; 23] = [
        // "ARBITER HEADER MISSING\0"
        0x41, 0x52, 0x42, 0x49, 0x54, 0x45, 0x52, 0x20,
        0x48, 0x45, 0x41, 0x44, 0x45, 0x52, 0x20, 0x4D,
        0x49, 0x53, 0x53, 0x49, 0x4E, 0x47, 0x00,
    ];

    // ASSERT_FDO(FdoExtension)
    if (*FdoExtension).ExtensionType != PciFdoExtensionType as u32 {
        RtlAssert(
            b"(FdoExtension)->ExtensionType == PciFdoExtensionType\0".as_ptr()
                as *const i8 as PVOID,
            FILE_PATH.as_ptr() as *const i8 as PVOID,
            47 as ULONG,
            core::ptr::null_mut(),
        );
    }

    let mut CurrentInterface: PPCI_INTERFACE = core::ptr::null_mut();
    let mut Interfaces: *mut PPCI_INTERFACE;
    let mut PdoExtension: PPCI_PDO_EXTENSION;
    let mut ArbiterInterface: PPCI_ARBITER_INSTANCE;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut ArbiterType: PCI_SIGNATURE = PciArb_Io;

    while ArbiterType as u32 <= PciArb_BusNumber as u32 {
        if (*FdoExtension).BusRootFdoExtension != FdoExtension {
            PdoExtension = (*(*FdoExtension).PhysicalDeviceObject)
                .DeviceExtension as PPCI_PDO_EXTENSION;

            // ASSERT_PDO(PdoExtension)
            if (*PdoExtension).ExtensionType != PciPdoExtensionType as u32 {
                RtlAssert(
                    b"(PdoExtension)->ExtensionType == PciPdoExtensionType\0".as_ptr()
                        as *const i8 as PVOID,
                    FILE_PATH.as_ptr() as *const i8 as PVOID,
                    57 as ULONG,
                    core::ptr::null_mut(),
                );
            }

            if (*PdoExtension).Dependent.type1.as_ref().SubtractiveDecode() != 0 {
                if DbgPrint(
                    b"(%s:%d) PCI Not creating arbiters for subtractive bus %u\n\0".as_ptr()
                        as PCSTR,
                    FILE_PATH.as_ptr() as PCSTR,
                    63 as i32,
                    (*PdoExtension).Dependent.type1.as_ref().SubtractiveDecode() as i32,
                ) != 0
                {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                        FILE_PATH.as_ptr() as PCSTR,
                        63 as i32,
                    );
                }
                ArbiterType = ((ArbiterType as u32) + 1) as PCI_SIGNATURE;
                continue;
            }
        }

        Interfaces = core::ptr::addr_of_mut!(PciInterfaces) as *mut PPCI_INTERFACE;
        while !(*Interfaces).is_null() {
            CurrentInterface = *Interfaces;
            if (*CurrentInterface).Signature as u32 == ArbiterType as u32 {
                break;
            }
            Interfaces = Interfaces.offset(1);
        }

        if (*Interfaces).is_null() {
            if DbgPrint(
                b"(%s:%d) PCI - FDO ext 0x%p no %s arbiter.\n\0".as_ptr() as PCSTR,
                FILE_PATH.as_ptr() as PCSTR,
                84 as i32,
                FdoExtension,
                PciArbiterNames[(ArbiterType as u32 - PciArb_Io as u32) as usize],
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                    FILE_PATH.as_ptr() as PCSTR,
                    84 as i32,
                );
            }
            ArbiterType = ((ArbiterType as u32) + 1) as PCI_SIGNATURE;
            continue;
        }

        Status = STATUS_INSUFFICIENT_RESOURCES;
        ArbiterInterface = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<PCI_ARBITER_INSTANCE>() as SIZE_T,
            PCI_POOL_TAG as ULONG,
        ) as PPCI_ARBITER_INSTANCE;

        if ArbiterInterface.is_null() {
            break;
        }

        (*ArbiterInterface).BusFdoExtension = FdoExtension;
        (*ArbiterInterface).Interface = CurrentInterface;

        swprintf(
            core::ptr::addr_of_mut!((*ArbiterInterface).InstanceName) as *mut u16,
            FORMAT_SWPRINTF.as_ptr() as *const u16,
            PciArbiterNames[(ArbiterType as u32 - PciArb_Io as u32) as usize],
            (*FdoExtension).BaseBus as i32,
        );

        Status = (*CurrentInterface)
            .Initializer
            .expect("non-null function pointer")(ArbiterInterface as PVOID);

        if !NT_SUCCESS(Status) {
            break;
        }

        PcipLinkSecondaryExtension(
            core::ptr::addr_of_mut!((*FdoExtension).SecondaryExtension),
            core::ptr::addr_of_mut!((*FdoExtension).SecondaryExtLock) as PVOID,
            core::ptr::addr_of_mut!((*ArbiterInterface).Header),
            ArbiterType,
            PciArbiterDestructor as PVOID,
        );

        if DbgPrint(
            b"(%s:%d) PCI - FDO ext 0x%p %S arbiter initialized (context 0x%p).\n\0".as_ptr()
                as PCSTR,
            FILE_PATH.as_ptr() as PCSTR,
            118 as i32,
            FdoExtension,
            ARBITER_HEADER_MISSING.as_ptr() as *const u16,
            ArbiterInterface,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                FILE_PATH.as_ptr() as PCSTR,
                118 as i32,
            );
        }

        Status = STATUS_SUCCESS;
        ArbiterType = ((ArbiterType as u32) + 1) as PCI_SIGNATURE;
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn PciInitializeArbiterRanges(
    DeviceExtension: PPCI_FDO_EXTENSION,
    Resources: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let _ = Resources;
    let mut PdoExtension: PPCI_PDO_EXTENSION;
    let mut Instance: PVOID;
    if (*DeviceExtension).ArbitersInitialized != 0 {
        if DbgPrint(
            b"(%s:%d) PCI Warning hot start FDOx %p, resource ranges not checked.\n\0"
                .as_ptr() as *const core::ffi::c_char,
            b"drivers/bus/pcix/arb/arb_comn.c\0"
                .as_ptr() as *const core::ffi::c_char,
            142 as core::ffi::c_int,
            DeviceExtension,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const core::ffi::c_char,
                b"drivers/bus/pcix/arb/arb_comn.c\0"
                    .as_ptr() as *const core::ffi::c_char,
                142 as core::ffi::c_int,
            );
        }
        return STATUS_INVALID_DEVICE_REQUEST;
    }
    if !((*DeviceExtension).BusRootFdoExtension == DeviceExtension) {
        PdoExtension =
            (*(*DeviceExtension).PhysicalDeviceObject).DeviceExtension as PPCI_PDO_EXTENSION;
        if ((*PdoExtension).ExtensionType != PciPdoExtensionType as core::ffi::c_ulong) {
            RtlAssert(
                b"(PdoExtension)->ExtensionType == PciPdoExtensionType\0"
                    .as_ptr() as *const core::ffi::c_char as PVOID,
                b"drivers/bus/pcix/arb/arb_comn.c\0"
                    .as_ptr() as *const core::ffi::c_char as PVOID,
                151 as ULONG,
                core::ptr::null_mut::<core::ffi::c_char>(),
            );
        }
        if (*PdoExtension).Dependent.type1.as_ref().SubtractiveDecode() != 0 {
            if DbgPrint(
                b"(%s:%d) PCI Skipping arbiter initialization for subtractive bridge FDOX %p\n\0"
                    .as_ptr() as *const core::ffi::c_char,
                b"drivers/bus/pcix/arb/arb_comn.c\0"
                    .as_ptr() as *const core::ffi::c_char,
                157 as core::ffi::c_int,
                DeviceExtension,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const core::ffi::c_char,
                    b"drivers/bus/pcix/arb/arb_comn.c\0"
                        .as_ptr() as *const core::ffi::c_char,
                    157 as core::ffi::c_int,
                );
            }
            return STATUS_SUCCESS;
        }
    }
    let mut ArbiterType: PCI_SIGNATURE = PciArb_Io;
    while (ArbiterType as core::ffi::c_uint) <= (PciArb_Memory as core::ffi::c_uint) {
        if ArbiterType != PciArb_Io && ArbiterType != PciArb_Memory {
            ArbiterType += 1;
            continue;
        }
        Instance = PciFindNextSecondaryExtension(
            &raw mut (*DeviceExtension).SecondaryExtension,
            ArbiterType,
        ) as PVOID;
        if !Instance.is_null() {
            DbgPrint(
                b"WARNING:  %s at %s:%d is unimplemented!()\n\0"
                    .as_ptr() as *const core::ffi::c_char,
                b"PciInitializeArbiterRanges\0".as_ptr() as *const core::ffi::c_char,
                b"drivers/bus/pcix/arb/arb_comn.c\0"
                    .as_ptr() as *const core::ffi::c_char,
                191 as core::ffi::c_int,
            );
        } else if DbgPrint(
            b"(%s:%d) PCI - FDO ext 0x%p %s arbiter (REQUIRED) is missing.\n\0"
                .as_ptr() as *const core::ffi::c_char,
            b"drivers/bus/pcix/arb/arb_comn.c\0"
                .as_ptr() as *const core::ffi::c_char,
            199 as core::ffi::c_int,
            DeviceExtension,
            PciArbiterNames[(ArbiterType as core::ffi::c_uint)
                .wrapping_sub(PciArb_Io as core::ffi::c_uint) as usize],
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const core::ffi::c_char,
                b"drivers/bus/pcix/arb/arb_comn.c\0"
                    .as_ptr() as *const core::ffi::c_char,
                199 as core::ffi::c_int,
            );
        }
        ArbiterType = ArbiterType.wrapping_add(1);
    }
    (*DeviceExtension).ArbitersInitialized = 1u8;
    STATUS_SUCCESS
}
