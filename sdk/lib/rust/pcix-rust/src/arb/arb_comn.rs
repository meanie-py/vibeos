#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{ARBITER_INTERFACE, PARBITER_INSTANCE, PARBITER_INTERFACE, PCM_RESOURCE_LIST};
use ntkrnl_sys::driver_ddk::swprintf;
use ntkrnl_sys::interlocked::{InterlockedDecrement, InterlockedIncrement};
use core::ffi::c_void;
use core::ffi::c_char;

// ---- file-scope preamble (translated) ----
pub static mut PciArbiterNames: [PCHAR; 4] = [
    b"I/O Port\0".as_ptr() as PCHAR,
    b"Memory\0".as_ptr() as PCHAR,
    b"Interrupt\0".as_ptr() as PCHAR,
    b"Bus Number\0".as_ptr() as PCHAR,
];

/// `PciArbiterNames[ArbiterType - PciArb_Io]`
unsafe fn arbiter_name(ArbiterType: PCI_SIGNATURE) -> PCHAR {
    PciArbiterNames[(ArbiterType as u32 - PciArb_Io as u32) as usize]
}

#[no_mangle]
pub unsafe extern "system" fn PciArbiter_Reference(Context: PVOID) {
    let Arbiter = Context as PARBITER_INSTANCE;

    InterlockedIncrement(core::ptr::addr_of_mut!((*Arbiter).ReferenceCount) as *mut LONG);
}

#[no_mangle]
pub unsafe extern "system" fn PciArbiter_Dereference(Context: PVOID) {
    let Arbiter = Context as PARBITER_INSTANCE;

    InterlockedDecrement(core::ptr::addr_of_mut!((*Arbiter).ReferenceCount) as *mut LONG);
}

#[no_mangle]
pub unsafe extern "system" fn PciArbiterConstructor(
    FdoExtension: PPCI_FDO_EXTENSION,
    ArbiterType: PCI_SIGNATURE,
    Interface: PARBITER_INTERFACE,
) -> NTSTATUS {
    PAGED_CODE!();

    if (*FdoExtension).ArbitersInitialized == 0 {
        return STATUS_NOT_SUPPORTED;
    }

    // Find the instance this bus built for the requested resource type
    let Arbiter = PciFindNextSecondaryExtension(
        (*FdoExtension).SecondaryExtension.Next,
        ArbiterType,
    ) as PPCI_ARBITER_INSTANCE;
    if Arbiter.is_null() {
        win32macros_rust::DPRINT1!(
            "PCI - FDO ext 0x%p has no %s arbiter to hand out.\n",
            FdoExtension,
            arbiter_name(ArbiterType),
        );
        return STATUS_NOT_SUPPORTED;
    }

    (*Interface).Size = core::mem::size_of::<ARBITER_INTERFACE>() as USHORT;
    (*Interface).Version = ARBITER_INTERFACE_VERSION as USHORT;
    (*Interface).Context = core::ptr::addr_of_mut!((*Arbiter).CommonInstance) as PVOID;
    (*Interface).InterfaceReference = Some(PciArbiter_Reference);
    (*Interface).InterfaceDereference = Some(PciArbiter_Dereference);
    (*Interface).ArbiterHandler = Some(ArbiterLibHandler);
    (*Interface).Flags = 0;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PciArbiterDestructor(Arbiter: PPCI_ARBITER_INSTANCE) {
    PAGED_CODE!();

    ArbiterLibDeleteInstance(core::ptr::addr_of_mut!((*Arbiter).CommonInstance));
}

#[no_mangle]
pub unsafe extern "system" fn PciInitializeArbiters(FdoExtension: PPCI_FDO_EXTENSION) -> NTSTATUS {
    let mut CurrentInterface: PPCI_INTERFACE = core::ptr::null_mut();
    ntkrnl_sys::ASSERT!((*FdoExtension).ExtensionType == PciFdoExtensionType);

    // A bus that ends up needing no arbiter at all is not a failure
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    // Loop all the arbiters
    let mut ArbiterType = PciArb_Io;
    while ArbiterType as u32 <= PciArb_BusNumber as u32 {
        let This = ArbiterType;
        ArbiterType = (ArbiterType as u32 + 1) as PCI_SIGNATURE;

        // Check if this is the extension for the Root PCI Bus
        if (*FdoExtension).BusRootFdoExtension != FdoExtension {
            // Get the PDO extension
            let PdoExtension =
                (*(*FdoExtension).PhysicalDeviceObject).DeviceExtension as PPCI_PDO_EXTENSION;
            ntkrnl_sys::ASSERT!((*PdoExtension).ExtensionType == PciPdoExtensionType);

            // Skip this bus if it does subtractive decode
            if (*PdoExtension).Dependent.type1.as_ref().SubtractiveDecode() != 0 {
                win32macros_rust::DPRINT1!(
                    "PCI Not creating arbiters for subtractive bus %u\n",
                    (*PdoExtension).Dependent.type1.as_ref().SubtractiveDecode() as u32,
                );
                continue;
            }
        }

        // Query all the registered arbiter interfaces
        let mut Interfaces = core::ptr::addr_of_mut!(PciInterfaces) as *mut PPCI_INTERFACE;
        while !(*Interfaces).is_null() {
            // Find the one that matches the arbiter currently being setup
            CurrentInterface = *Interfaces;
            if (*CurrentInterface).Signature as u32 == This as u32 {
                break;
            }
            Interfaces = Interfaces.add(1);
        }

        // Check if the required arbiter was not found in the list
        if (*Interfaces).is_null() {
            // Skip this arbiter and try the next one
            win32macros_rust::DPRINT1!(
                "PCI - FDO ext 0x%p no %s arbiter.\n",
                FdoExtension,
                arbiter_name(This),
            );
            continue;
        }

        // An arbiter was found, allocate an instance for it
        Status = STATUS_INSUFFICIENT_RESOURCES;
        let ArbiterInterface = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<PCI_ARBITER_INSTANCE>() as SIZE_T,
            PCI_POOL_TAG as ULONG,
        ) as PPCI_ARBITER_INSTANCE;
        if ArbiterInterface.is_null() {
            break;
        }

        // Setup the instance
        (*ArbiterInterface).BusFdoExtension = FdoExtension;
        (*ArbiterInterface).Interface = CurrentInterface;
        swprintf(
            core::ptr::addr_of_mut!((*ArbiterInterface).InstanceName) as *mut u16,
            w!("PCI %S (b=%02x)").as_ptr(),
            arbiter_name(This),
            (*FdoExtension).BaseBus as u32,
        );

        // Call the interface initializer for it
        Status = ((*CurrentInterface).Initializer.unwrap())(ArbiterInterface as PVOID);
        if !NT_SUCCESS(Status) {
            break;
        }

        // Link it with this FDO
        PcipLinkSecondaryExtension(
            core::ptr::addr_of_mut!((*FdoExtension).SecondaryExtension),
            core::ptr::addr_of_mut!((*FdoExtension).SecondaryExtLock) as PVOID,
            core::ptr::addr_of_mut!((*ArbiterInterface).Header),
            This,
            PciArbiterDestructor as PVOID,
        );

        // This arbiter is now initialized, move to the next one
        win32macros_rust::DPRINT1!(
            "PCI - FDO ext 0x%p %S arbiter initialized (context 0x%p).\n",
            FdoExtension,
            w!("ARBITER HEADER MISSING").as_ptr(),
            ArbiterInterface,
        );
        Status = STATUS_SUCCESS;
    }

    // Return to caller
    Status
}

#[no_mangle]
pub unsafe extern "system" fn PciInitializeArbiterRanges(
    DeviceExtension: PPCI_FDO_EXTENSION,
    Resources: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    // Arbiters should not already be initialized
    if (*DeviceExtension).ArbitersInitialized != 0 {
        // Duplicated start request, fail initialization
        win32macros_rust::DPRINT1!(
            "PCI Warning hot start FDOx %p, resource ranges not checked.\n",
            DeviceExtension,
        );
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    // Check for non-root FDO
    if (*DeviceExtension).BusRootFdoExtension != DeviceExtension {
        // Grab the PDO
        let PdoExtension =
            (*(*DeviceExtension).PhysicalDeviceObject).DeviceExtension as PPCI_PDO_EXTENSION;
        ntkrnl_sys::ASSERT!((*PdoExtension).ExtensionType == PciPdoExtensionType);

        // Check if this is a subtractive bus
        if (*PdoExtension).Dependent.type1.as_ref().SubtractiveDecode() != 0 {
            // There is nothing to do regarding arbitration of resources
            win32macros_rust::DPRINT1!(
                "PCI Skipping arbiter initialization for subtractive bridge FDOX %p\n",
                DeviceExtension,
            );
            return STATUS_SUCCESS;
        }
    }

    // Loop the arbiters that hand out the ranges a bus decodes
    let mut ArbiterType = PciArb_Io;
    while ArbiterType as u32 <= PciArb_Memory as u32 {
        // Find an arbiter of this type
        let Instance = PciFindNextSecondaryExtension(
            (*DeviceExtension).SecondaryExtension.Next,
            ArbiterType,
        ) as PPCI_ARBITER_INSTANCE;
        if !Instance.is_null() {
            // Hand it the resources this bus was started with. They describe
            // the windows the bus actually decodes, which is what bounds the
            // addresses it may hand out to the devices behind it.
            let Status = ((*Instance).CommonInstance.StartArbiter.unwrap())(
                core::ptr::addr_of_mut!((*Instance).CommonInstance),
                Resources,
            );
            if !NT_SUCCESS(Status) {
                win32macros_rust::DPRINT1!(
                    "PCI - FDO ext 0x%p %s arbiter failed to start: %X\n",
                    DeviceExtension,
                    arbiter_name(ArbiterType),
                    Status,
                );
                return Status;
            }
        } else {
            // The arbiter was not found, this is an error!
            win32macros_rust::DPRINT1!(
                "PCI - FDO ext 0x%p %s arbiter (REQUIRED) is missing.\n",
                DeviceExtension,
                arbiter_name(ArbiterType),
            );
        }
        ArbiterType = (ArbiterType as u32 + 1) as PCI_SIGNATURE;
    }

    // Arbiters are now initialized
    (*DeviceExtension).ArbitersInitialized = TRUE as BOOLEAN;
    STATUS_SUCCESS
}
