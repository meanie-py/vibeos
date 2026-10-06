//! atapi.c — controller-discovery + DriverEntry module of the ATAPI IDE miniport.
//!
//! Faithful transliteration of:
//!   AtapiParseArgumentString, AtapiFindController, FindBrokenController,
//!   AtapiFindNativeModeController, AtapiFindPCIController, DriverEntry.
//!
//! DriverEntry (the #[no_mangle] export that takes over the cohab atapi.sys link)
//! builds an HW_INITIALIZATION_DATA and registers it 4× with ScsiPortInitialize.

use core::mem::size_of;

use ntkrnl_sys::ddk::{
    BOOLEAN, INTERFACE_TYPE, PBOOLEAN, PCHAR, PUCHAR, PVOID, UCHAR, ULONG, ULONG_PTR, USHORT,
    // INTERFACE_TYPE enum members
    _INTERFACE_TYPE_Isa, _INTERFACE_TYPE_MicroChannel, _INTERFACE_TYPE_PCIBus,
    // KINTERRUPT_MODE enum members
    _KINTERRUPT_MODE_Latched, _KINTERRUPT_MODE_LevelSensitive,
    // BUS_DATA_TYPE enum member (PCIConfiguration)
    _BUS_DATA_TYPE_PCIConfiguration,
    // PCI config space layout. The C `PCI_COMMON_CONFIG` flattens `PCI_COMMON_HEADER`
    // (VendorID/DeviceID/ProgIf...) ahead of DeviceSpecific[]; bindgen's
    // _PCI_COMMON_CONFIG drops those header fields (it's just DeviceSpecific[192] —
    // see memory feedback_bindgen_drops_macro_spliced_struct_fields), so read the
    // named header fields through a _PCI_COMMON_HEADER view of the same bytes.
    PCI_COMMON_CONFIG, PCI_SLOT_NUMBER, PPCI_COMMON_HEADER,
};

use crate::types::{
    ACCESS_RANGE, BROKEN_CONTROLLER_INFORMATION, HW_DEVICE_EXTENSION, HW_INITIALIZATION_DATA,
    HW_LU_EXTENSION, NATIVE_MODE_CONTROLLER_INFORMATION,
    PHW_DEVICE_EXTENSION, PIDE_REGISTERS_1, PIDE_REGISTERS_2,
    PPORT_CONFIGURATION_INFORMATION, SCSI_PHYSICAL_ADDRESS,
    DFLAGS_ATAPI_DEVICE, DFLAGS_DEVICE_PRESENT, DFLAGS_TAPE_DEVICE, IDE_STATUS_BUSY,
    SP_RETURN_ERROR, SP_RETURN_FOUND, SP_RETURN_NOT_FOUND,
};

// atapi.h:305 / :319 define these const tables WITH initializers — so they lived in
// atapi.c's TU (the bindgen substrate only emits extern declarations for them). In
// the cohab build atapi.c is gone, so the data is defined here. `const` (not `static`)
// because the structs hold raw pointers (not Sync); the C-string fields point at
// 'static NUL-terminated literals. VendorId/DeviceId are PCHAR (`*mut c_char`).
const BrokenAdapters: [BROKEN_CONTROLLER_INFORMATION; 2] = [
    BROKEN_CONTROLLER_INFORMATION {
        VendorId: b"1095\0".as_ptr() as PCHAR, VendorIdLength: 4,
        DeviceId: b"0640\0".as_ptr() as PCHAR, DeviceIdLength: 4,
    },
    BROKEN_CONTROLLER_INFORMATION {
        VendorId: b"1039\0".as_ptr() as PCHAR, VendorIdLength: 4,
        DeviceId: b"0601\0".as_ptr() as PCHAR, DeviceIdLength: 4,
    },
];
const NativeModeAdapters: [NATIVE_MODE_CONTROLLER_INFORMATION; 1] = [
    NATIVE_MODE_CONTROLLER_INFORMATION {
        VendorId: b"10ad\0".as_ptr() as PCHAR, VendorIdLength: 4,
        DeviceId: b"0105\0".as_ptr() as PCHAR, DeviceIdLength: 4,
    },
];

use crate::inlines::{
    ScsiPortConvertPhysicalAddressToUlong, ScsiPortConvertUlongToPhysicalAddress,
    ScsiPortFreeDeviceBase, ScsiPortGetBusData, ScsiPortGetDeviceBase, ScsiPortInitialize,
    ScsiPortReadPortUchar, ScsiPortStallExecution, ScsiPortWritePortUchar,
};

use crate::helpers::{AtapiHexToString, AtapiStringCmp, AtapiZeroMemory};

use crate::identify::{AtapiHwInitialize, AtapiResetController, FindDevices};
use crate::startio::{AtapiInterrupt, AtapiStartIo};

// `PCI_INVALID_VENDORID` is defined only in ntkrnl-sys hal_bindgen (x86-only, gated
// behind the `hal_bindgen` feature), not in ntkrnl_sys::ddk or crate::types, so it
// is not importable here. Its value is the documented PCI sentinel 0xFFFF.
// TODO-MISSING: PCI_INVALID_VENDORID (using documented value 0xFFFF).
const PCI_INVALID_VENDORID: USHORT = 0xFFFF;

// atapi.c:1851
pub unsafe extern "system" fn AtapiParseArgumentString(String: PCHAR, KeyWord: PCHAR) -> ULONG {
    let mut cptr: PCHAR;
    let mut kptr: PCHAR;
    let mut value: ULONG;
    let mut stringLength: ULONG = 0;
    let mut keyWordLength: ULONG = 0;
    let mut index: ULONG;

    if String.is_null() {
        return 0;
    }
    if KeyWord.is_null() {
        return 0;
    }

    // Calculate the string length and lower case all characters.
    cptr = String;
    while *cptr != 0 {
        if *cptr >= b'A' as i8 && *cptr <= b'Z' as i8 {
            *cptr = *cptr + (b'a' as i8 - b'A' as i8);
        }
        cptr = cptr.add(1);
        stringLength += 1;
    }

    // Calculate the keyword length and lower case all characters.
    cptr = KeyWord;
    while *cptr != 0 {
        if *cptr >= b'A' as i8 && *cptr <= b'Z' as i8 {
            *cptr = *cptr + (b'a' as i8 - b'A' as i8);
        }
        cptr = cptr.add(1);
        keyWordLength += 1;
    }

    if keyWordLength > stringLength {
        // Can't possibly have a match.
        return 0;
    }

    // Now setup and start the compare.
    cptr = String;

    // ContinueSearch:
    'ContinueSearch: loop {
        // The input string may start with white space.  Skip it.
        while *cptr == b' ' as i8 || *cptr == b'\t' as i8 {
            cptr = cptr.add(1);
        }

        if *cptr == b'\0' as i8 {
            // end of string.
            return 0;
        }

        kptr = KeyWord;
        // while (*cptr++ == *kptr++)
        loop {
            let c = *cptr;
            cptr = cptr.add(1);
            let k = *kptr;
            kptr = kptr.add(1);
            if c != k {
                break;
            }
            if *cptr.offset(-1) == b'\0' as i8 {
                // end of string
                return 0;
            }
        }

        if *kptr.offset(-1) == b'\0' as i8 {
            // May have a match backup and check for blank or equals.
            cptr = cptr.offset(-1);

            while *cptr == b' ' as i8 || *cptr == b'\t' as i8 {
                cptr = cptr.add(1);
            }

            // Found a match.  Make sure there is an equals.
            if *cptr != b'=' as i8 {
                // Not a match so move to the next semicolon.
                while *cptr != 0 {
                    let c = *cptr;
                    cptr = cptr.add(1);
                    if c == b';' as i8 {
                        continue 'ContinueSearch;
                    }
                }
                return 0;
            }

            // Skip the equals sign.
            cptr = cptr.add(1);

            // Skip white space.
            while *cptr == b' ' as i8 || *cptr == b'\t' as i8 {
                cptr = cptr.add(1);
            }

            if *cptr == b'\0' as i8 {
                // Early end of string, return not found
                return 0;
            }

            if *cptr == b';' as i8 {
                // This isn't it either.
                cptr = cptr.add(1);
                continue 'ContinueSearch;
            }

            value = 0;
            if *cptr == b'0' as i8 && *cptr.add(1) == b'x' as i8 {
                // Value is in Hex.  Skip the "0x"
                cptr = cptr.add(2);
                index = 0;
                while *cptr.add(index as usize) != 0 {
                    if *cptr.add(index as usize) == b' ' as i8
                        || *cptr.add(index as usize) == b'\t' as i8
                        || *cptr.add(index as usize) == b';' as i8
                    {
                        break;
                    }

                    if *cptr.add(index as usize) >= b'0' as i8
                        && *cptr.add(index as usize) <= b'9' as i8
                    {
                        value = (16 * value) + (*cptr.add(index as usize) - b'0' as i8) as ULONG;
                    } else {
                        if *cptr.add(index as usize) >= b'a' as i8
                            && *cptr.add(index as usize) <= b'f' as i8
                        {
                            value = (16 * value)
                                + (*cptr.add(index as usize) - b'a' as i8 + 10) as ULONG;
                        } else {
                            // Syntax error, return not found.
                            return 0;
                        }
                    }
                    index += 1;
                }
            } else {
                // Value is in Decimal.
                index = 0;
                while *cptr.add(index as usize) != 0 {
                    if *cptr.add(index as usize) == b' ' as i8
                        || *cptr.add(index as usize) == b'\t' as i8
                        || *cptr.add(index as usize) == b';' as i8
                    {
                        break;
                    }

                    if *cptr.add(index as usize) >= b'0' as i8
                        && *cptr.add(index as usize) <= b'9' as i8
                    {
                        value = (10 * value) + (*cptr.add(index as usize) - b'0' as i8) as ULONG;
                    } else {
                        // Syntax error return not found.
                        return 0;
                    }
                    index += 1;
                }
            }

            return value;
        } else {
            // Not a match check for ';' to continue search.
            while *cptr != 0 {
                let c = *cptr;
                cptr = cptr.add(1);
                if c == b';' as i8 {
                    continue 'ContinueSearch;
                }
            }

            return 0;
        }
    }
}

// atapi.c:2104
pub unsafe extern "system" fn AtapiFindController(
    HwDeviceExtension: PVOID,
    Context: PVOID,
    BusInformation: PVOID,
    ArgumentString: PCHAR,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    Again: PBOOLEAN,
) -> ULONG {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let adapterCount: *mut ULONG = Context as *mut ULONG;
    let mut ioSpace: PUCHAR = core::ptr::null_mut();
    let mut i: ULONG;
    let mut irq: ULONG;
    let mut portBase: ULONG;
    let mut retryCount: ULONG;
    let mut slotData: PCI_SLOT_NUMBER;
    let mut pciData: PPCI_COMMON_HEADER;
    let mut pciBuffer: ULONG = 0;
    let mut atapiOnly: BOOLEAN;
    let mut statusByte: UCHAR;
    let mut preConfig: BOOLEAN = 0; // FALSE

    // The following table specifies the ports to be checked when searching for
    // an IDE controller.  A zero entry terminates the search.
    const AdapterAddresses: [ULONG; 5] = [0x1F0, 0x170, 0x1e8, 0x168, 0];

    // The following table specifies interrupt levels corresponding to the
    // port addresses in the previous table.
    const InterruptLevels: [ULONG; 5] = [14, 15, 11, 10, 0];

    if deviceExtension.is_null() {
        return SP_RETURN_ERROR;
    }

    // Check to see if this is a special configuration environment.
    portBase = 0;
    irq = 0;
    if !ArgumentString.is_null() {
        irq = AtapiParseArgumentString(ArgumentString, b"Interrupt\0".as_ptr() as PCHAR);
        if irq != 0 {
            // Both parameters must be present to proceed
            portBase = AtapiParseArgumentString(ArgumentString, b"BaseAddress\0".as_ptr() as PCHAR);
            if portBase == 0 {
                // Try a default search for the part.
                irq = 0;
            }
        }
    }

    // Scan though the adapter address looking for adapters.
    let accessRanges0 = access_range_ptr(ConfigInfo);
    if ScsiPortConvertPhysicalAddressToUlong((*accessRanges0.add(0)).RangeStart) != 0 {
        ioSpace = ScsiPortGetDeviceBase(
            HwDeviceExtension,
            (*ConfigInfo).AdapterInterfaceType,
            (*ConfigInfo).SystemIoBusNumber,
            (*accessRanges0.add(0)).RangeStart,
            (*accessRanges0.add(0)).RangeLength,
            (((*accessRanges0.add(0)).RangeInMemory) == 0) as BOOLEAN,
        ) as PUCHAR;
        *Again = 0; // FALSE
        // Since we have pre-configured information we only need to go through this loop once
        preConfig = 1; // TRUE
        portBase = ScsiPortConvertPhysicalAddressToUlong((*accessRanges0.add(0)).RangeStart);
    }

    'outer: while AdapterAddresses[*adapterCount as usize] != 0 {
        retryCount = 4;

        i = 0;
        while i < 4 {
            // Zero device fields to ensure that if earlier devices were found,
            // but not claimed, the fields are cleared.
            (*deviceExtension).DeviceFlags[i as usize] &= !((DFLAGS_ATAPI_DEVICE
                | DFLAGS_DEVICE_PRESENT
                | DFLAGS_TAPE_DEVICE) as USHORT);
            i += 1;
        }

        // Check if configInfo has the default information
        // if not, we go and find ourselves
        if preConfig == 0 {
            if portBase != 0 {
                ioSpace = ScsiPortGetDeviceBase(
                    HwDeviceExtension,
                    (*ConfigInfo).AdapterInterfaceType,
                    (*ConfigInfo).SystemIoBusNumber,
                    ScsiPortConvertUlongToPhysicalAddress(portBase as ULONG_PTR),
                    8,
                    1, // TRUE
                ) as PUCHAR;
            } else {
                ioSpace = ScsiPortGetDeviceBase(
                    HwDeviceExtension,
                    (*ConfigInfo).AdapterInterfaceType,
                    (*ConfigInfo).SystemIoBusNumber,
                    ScsiPortConvertUlongToPhysicalAddress(
                        AdapterAddresses[*adapterCount as usize] as ULONG_PTR,
                    ),
                    8,
                    1, // TRUE
                ) as PUCHAR;
            }
        } // ConfigInfo check

        // Update the adapter count.
        *adapterCount += 1;

        // Check if ioSpace accessible.
        if ioSpace.is_null() {
            continue;
        }

        // retryIdentifier:
        'retryIdentifier: loop {
            // Select master.
            ScsiPortWritePortUchar(
                &mut (*(ioSpace as PIDE_REGISTERS_1)).DriveSelect as *mut UCHAR as PUCHAR,
                0xA0,
            );

            // Check if card at this address.
            ScsiPortWritePortUchar(
                &mut (*(ioSpace as PIDE_REGISTERS_1)).CylinderLow as *mut UCHAR as PUCHAR,
                0xAA,
            );

            // Check if identifier can be read back.
            statusByte = ScsiPortReadPortUchar(
                &mut (*(ioSpace as PIDE_REGISTERS_1)).CylinderLow as *mut UCHAR as PUCHAR,
            );
            if statusByte != 0xAA {
                statusByte = ScsiPortReadPortUchar(
                    &mut (*(ioSpace as PIDE_REGISTERS_2)).AlternateStatus as *mut UCHAR as PUCHAR,
                );

                if (statusByte & IDE_STATUS_BUSY as UCHAR) != 0 {
                    i = 0;

                    // Could be the TEAC in a thinkpad. Their dos driver puts it in a sleep-mode that
                    // warm boots don't clear.
                    loop {
                        ScsiPortStallExecution(1000);
                        statusByte = ScsiPortReadPortUchar(
                            &mut (*(ioSpace as PIDE_REGISTERS_1)).Command as *mut UCHAR as PUCHAR,
                        );
                        i += 1;
                        if !((statusByte & IDE_STATUS_BUSY as UCHAR) != 0 && i < 10) {
                            break;
                        }
                    }

                    let rc = retryCount;
                    retryCount = retryCount.wrapping_sub(1);
                    if rc != 0 && (statusByte & IDE_STATUS_BUSY as UCHAR) == 0 {
                        continue 'retryIdentifier;
                    }
                }

                // Select slave.
                ScsiPortWritePortUchar(
                    &mut (*(ioSpace as PIDE_REGISTERS_1)).DriveSelect as *mut UCHAR as PUCHAR,
                    0xB0,
                );

                // See if slave is present.
                ScsiPortWritePortUchar(
                    &mut (*(ioSpace as PIDE_REGISTERS_1)).CylinderLow as *mut UCHAR as PUCHAR,
                    0xAA,
                );

                statusByte = ScsiPortReadPortUchar(
                    &mut (*(ioSpace as PIDE_REGISTERS_1)).CylinderLow as *mut UCHAR as PUCHAR,
                );
                if statusByte != 0xAA {
                    // No controller at this base address.
                    ScsiPortFreeDeviceBase(HwDeviceExtension, ioSpace as PVOID);
                    // C `continue;` — restart the outer adapter-address while loop.
                    continue 'outer;
                }
            }

            // (master matched, or slave found) — exit the retry loop and proceed.
            break 'retryIdentifier;
        }

        // Record base IO address.
        (*deviceExtension).BaseIoAddress1[0] = ioSpace as PIDE_REGISTERS_1;

        // Fill in the access array information only if default params are not in there.
        if preConfig == 0 {
            // An adapter has been found request another call, only if we didn't get preconfigured info.
            *Again = 1; // TRUE

            if portBase != 0 {
                (*access_range_ptr(ConfigInfo).add(0)).RangeStart =
                    ScsiPortConvertUlongToPhysicalAddress(portBase as ULONG_PTR);
            } else {
                (*access_range_ptr(ConfigInfo).add(0)).RangeStart =
                    ScsiPortConvertUlongToPhysicalAddress(
                        AdapterAddresses[*adapterCount as usize - 1] as ULONG_PTR,
                    );
            }

            (*access_range_ptr(ConfigInfo).add(0)).RangeLength = 8;
            (*access_range_ptr(ConfigInfo).add(0)).RangeInMemory = 0; // FALSE

            // Indicate the interrupt level corresponding to this IO range.
            if irq != 0 {
                (*ConfigInfo).BusInterruptLevel = irq;
            } else {
                (*ConfigInfo).BusInterruptLevel = InterruptLevels[*adapterCount as usize - 1];
            }

            if (*ConfigInfo).AdapterInterfaceType == _INTERFACE_TYPE_MicroChannel as INTERFACE_TYPE {
                (*ConfigInfo).InterruptMode = _KINTERRUPT_MODE_LevelSensitive;
            } else {
                (*ConfigInfo).InterruptMode = _KINTERRUPT_MODE_Latched;
            }
        }

        // Get the system physical address for the second IO range.
        if portBase != 0 {
            ioSpace = ScsiPortGetDeviceBase(
                HwDeviceExtension,
                (*ConfigInfo).AdapterInterfaceType,
                (*ConfigInfo).SystemIoBusNumber,
                ScsiPortConvertUlongToPhysicalAddress((portBase + 0x206) as ULONG_PTR),
                1,
                1, // TRUE
            ) as PUCHAR;
        } else {
            ioSpace = ScsiPortGetDeviceBase(
                HwDeviceExtension,
                (*ConfigInfo).AdapterInterfaceType,
                (*ConfigInfo).SystemIoBusNumber,
                ScsiPortConvertUlongToPhysicalAddress(
                    (AdapterAddresses[*adapterCount as usize - 1] + 0x206) as ULONG_PTR,
                ),
                1,
                1, // TRUE
            ) as PUCHAR;
        }

        (*deviceExtension).BaseIoAddress2[0] = ioSpace as PIDE_REGISTERS_2;

        (*deviceExtension).NumberChannels = 1;

        (*ConfigInfo).NumberOfBuses = 1;
        (*ConfigInfo).MaximumNumberOfTargets = 2;

        // Indicate maximum transfer length is 64k.
        (*ConfigInfo).MaximumTransferLength = 0x10000;

        // For Daytona, the atdisk driver gets the first shot at the
        // primary and secondary controllers.
        if preConfig == 0 {
            if *adapterCount - 1 < 2 {
                // Determine whether this driver is being initialized by the
                // system or as a crash dump driver.
                if !ArgumentString.is_null() {
                    if AtapiParseArgumentString(ArgumentString, b"dump\0".as_ptr() as PCHAR) == 1 {
                        atapiOnly = 0; // FALSE
                        (*deviceExtension).DriverMustPoll = 1; // TRUE
                    } else {
                        atapiOnly = 1; // TRUE
                        (*deviceExtension).DriverMustPoll = 0; // FALSE
                    }
                } else {
                    atapiOnly = 1; // TRUE
                    (*deviceExtension).DriverMustPoll = 0; // FALSE
                }
            } else {
                atapiOnly = 0; // FALSE
            }

            // If this is a PCI machine, pick up all devices.
            pciData = (&mut pciBuffer as *mut ULONG) as PPCI_COMMON_HEADER;

            slotData = core::mem::zeroed();
            slotData.u.bits.set_DeviceNumber(0);
            slotData.u.bits.set_FunctionNumber(0);

            if ScsiPortGetBusData(
                deviceExtension as PVOID,
                _BUS_DATA_TYPE_PCIConfiguration as ULONG,
                0, // BusNumber
                slotData.u.AsULONG,
                pciData as PVOID,
                size_of::<ULONG>() as ULONG,
            ) != 0
            {
                atapiOnly = 0; // FALSE

                // Wait on doing this, until a reliable method
                // of determining support is found.
                // #if 0 deviceExtension->DWordIO = TRUE; #endif
            } else {
                (*deviceExtension).DWordIO = 0; // FALSE
            }
        } else {
            atapiOnly = 0; // FALSE
            (*deviceExtension).DriverMustPoll = 0; // FALSE
        } // preConfig check

        // Save the Interrupt Mode for later use
        (*deviceExtension).InterruptMode = (*ConfigInfo).InterruptMode;

        // Search for devices on this controller.
        if FindDevices(HwDeviceExtension, atapiOnly, 0) != 0 {
            // Claim primary or secondary ATA IO range.
            if portBase != 0 {
                match portBase {
                    0x170 => {
                        (*ConfigInfo).AtdiskSecondaryClaimed = 1; // TRUE
                        (*deviceExtension).PrimaryAddress = 0; // FALSE
                    }
                    0x1f0 => {
                        (*ConfigInfo).AtdiskPrimaryClaimed = 1; // TRUE
                        (*deviceExtension).PrimaryAddress = 1; // TRUE
                    }
                    _ => {}
                }
            } else {
                if *adapterCount == 1 {
                    (*ConfigInfo).AtdiskPrimaryClaimed = 1; // TRUE
                    (*deviceExtension).PrimaryAddress = 1; // TRUE
                } else if *adapterCount == 2 {
                    (*ConfigInfo).AtdiskSecondaryClaimed = 1; // TRUE
                    (*deviceExtension).PrimaryAddress = 0; // FALSE
                }
            }

            return SP_RETURN_FOUND;
        }
    }

    // The entire table has been searched and no adapters have been found.
    // There is no need to call again and the device base can now be freed.
    // Clear the adapter count for the next bus.
    *Again = 0; // FALSE
    *adapterCount = 0;

    SP_RETURN_NOT_FOUND
}

// atapi.c:2573
pub unsafe extern "system" fn FindBrokenController(
    DeviceExtension: PVOID,
    VendorID: PUCHAR,
    VendorIDLength: ULONG,
    DeviceID: PUCHAR,
    DeviceIDLength: ULONG,
    FunctionNumber: *mut ULONG,
    SlotNumber: *mut ULONG,
    BusNumber: ULONG,
    LastSlot: PBOOLEAN,
) -> BOOLEAN {
    let mut pciBuffer: ULONG = 0;
    let mut slotNumber: ULONG;
    let mut functionNumber: ULONG;
    let mut slotData: PCI_SLOT_NUMBER;
    let pciData: PPCI_COMMON_HEADER;
    let mut vendorString: [UCHAR; 5] = [0; 5];
    let mut deviceString: [UCHAR; 5] = [0; 5];
    let mut vendorStrPtr: PUCHAR;
    let mut deviceStrPtr: PUCHAR;

    pciData = (&mut pciBuffer as *mut ULONG) as PPCI_COMMON_HEADER;

    slotData = core::mem::zeroed();
    slotData.u.AsULONG = 0;

    // Look at each device.
    slotNumber = *SlotNumber;
    while slotNumber < 32 {
        slotData.u.bits.set_DeviceNumber(slotNumber);

        // Look at each function.
        functionNumber = *FunctionNumber;
        while functionNumber < 8 {
            slotData.u.bits.set_FunctionNumber(functionNumber);

            if ScsiPortGetBusData(
                DeviceExtension,
                _BUS_DATA_TYPE_PCIConfiguration as ULONG,
                BusNumber,
                slotData.u.AsULONG,
                pciData as PVOID,
                size_of::<ULONG>() as ULONG,
            ) == 0
            {
                // Out of PCI data.
                *LastSlot = 1; // TRUE
                return 0; // FALSE
            }

            if (*pciData).VendorID == PCI_INVALID_VENDORID {
                // No PCI device, or no more functions on device
                // move to next PCI device.
                break;
            }

            // Translate hex ids to strings.
            vendorStrPtr = vendorString.as_mut_ptr();
            deviceStrPtr = deviceString.as_mut_ptr();
            AtapiHexToString((*pciData).VendorID as ULONG, core::ptr::addr_of_mut!(vendorStrPtr).cast());
            AtapiHexToString((*pciData).DeviceID as ULONG, core::ptr::addr_of_mut!(deviceStrPtr).cast());

            // Compare strings.
            if AtapiStringCmp(vendorString.as_mut_ptr() as PCHAR, VendorID as PCHAR, VendorIDLength)
                != 0
                || AtapiStringCmp(
                    deviceString.as_mut_ptr() as PCHAR,
                    DeviceID as PCHAR,
                    DeviceIDLength,
                ) != 0
            {
                // Not our PCI device. Try next device/function
                functionNumber += 1;
                continue;
            }

            *FunctionNumber = functionNumber;
            *SlotNumber = slotNumber;
            return 1; // TRUE

            // (next PCI function — unreachable due to returns/continue)
        } // next PCI function

        *FunctionNumber = 0;
        slotNumber += 1;
    } // next PCI slot

    *LastSlot = 1; // TRUE
    0 // FALSE
}

// atapi.c:2710
pub unsafe extern "system" fn AtapiFindNativeModeController(
    HwDeviceExtension: PVOID,
    Context: PVOID,
    BusInformation: PVOID,
    ArgumentString: PCHAR,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    Again: PBOOLEAN,
) -> ULONG {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let nativeModeAdapterTableIndex: ULONG = Context as ULONG_PTR as ULONG;
    let mut channel: ULONG;
    let mut ioSpace: PUCHAR;
    let mut atapiOnly: BOOLEAN;
    let mut deviceFound: BOOLEAN = 0; // FALSE
    let mut statusByte: UCHAR;
    let mut slotData: PCI_SLOT_NUMBER;
    // PCI_COMMON_CONFIG pciData (by value); bindgen drops header fields, so read
    // VendorID/DeviceID/ProgIf through a _PCI_COMMON_HEADER view of the same bytes.
    let mut pciData: PCI_COMMON_CONFIG = core::mem::zeroed();
    let pciHdr: PPCI_COMMON_HEADER = &mut pciData as *mut _ as PPCI_COMMON_HEADER;
    let mut funcNumber: ULONG;
    let mut busDataRead: ULONG;
    let mut vendorString: [UCHAR; 5] = [0; 5];
    let mut deviceString: [UCHAR; 5] = [0; 5];
    let mut vendorStrPtr: PUCHAR;
    let mut deviceStrPtr: PUCHAR;
    let mut IoBasePort1: SCSI_PHYSICAL_ADDRESS;
    let mut IoBasePort2: SCSI_PHYSICAL_ADDRESS;

    // The following table specifies the ports to be checked when searching for
    // an IDE controller.  A zero entry terminates the search.
    const AdapterAddresses: [ULONG; 3] = [0x1F0, 0x170, 0];

    if deviceExtension.is_null() {
        return SP_RETURN_ERROR;
    }

    *Again = 0; // FALSE

    slotData = core::mem::zeroed();
    slotData.u.AsULONG = 0;
    slotData.u.bits.set_DeviceNumber((*ConfigInfo).SlotNumber);

    funcNumber = 0;
    while funcNumber < 8 {
        slotData.u.bits.set_FunctionNumber(funcNumber);

        busDataRead = ScsiPortGetBusData(
            HwDeviceExtension,
            _BUS_DATA_TYPE_PCIConfiguration as ULONG,
            (*ConfigInfo).SystemIoBusNumber,
            slotData.u.AsULONG,
            &mut pciData as *mut _ as PVOID,
            size_of::<PCI_COMMON_CONFIG>() as ULONG,
        );
        if busDataRead != size_of::<PCI_COMMON_CONFIG>() as ULONG {
            return SP_RETURN_ERROR;
        }
        if (*pciHdr).VendorID == PCI_INVALID_VENDORID {
            return SP_RETURN_ERROR;
        }

        // Translate hex ids to strings.
        vendorStrPtr = vendorString.as_mut_ptr();
        deviceStrPtr = deviceString.as_mut_ptr();
        AtapiHexToString((*pciHdr).VendorID as ULONG, core::ptr::addr_of_mut!(vendorStrPtr).cast());
        AtapiHexToString((*pciHdr).DeviceID as ULONG, core::ptr::addr_of_mut!(deviceStrPtr).cast());

        // Compare strings.
        if AtapiStringCmp(
            vendorString.as_mut_ptr() as PCHAR,
            NativeModeAdapters[nativeModeAdapterTableIndex as usize].VendorId,
            NativeModeAdapters[nativeModeAdapterTableIndex as usize].VendorIdLength,
        ) != 0
            || AtapiStringCmp(
                deviceString.as_mut_ptr() as PCHAR,
                NativeModeAdapters[nativeModeAdapterTableIndex as usize].DeviceId,
                NativeModeAdapters[nativeModeAdapterTableIndex as usize].DeviceIdLength,
            ) != 0
        {
            funcNumber += 1;
            continue;
        }

        if ((*pciHdr).ProgIf as u32 & ((1u32 << 2) | (1u32 << 0))) != 0 {
            // both primary and secondary channel are in native mode
            // Found our device
            *Again = 1; // TRUE
            break;
        }

        funcNumber += 1;
    }

    if *Again != 0 {
        channel = 0;
        'channelLoop: while channel < 2 {
            IoBasePort1 = (*access_range_ptr(ConfigInfo).add((channel * 2 + 0) as usize)).RangeStart;
            IoBasePort2 = (*access_range_ptr(ConfigInfo).add((channel * 2 + 1) as usize)).RangeStart;
            IoBasePort2 = ScsiPortConvertUlongToPhysicalAddress(
                (ScsiPortConvertPhysicalAddressToUlong(IoBasePort2) + 2) as ULONG_PTR,
            );

            // Get the system physical address for this IO range.
            ioSpace = ScsiPortGetDeviceBase(
                HwDeviceExtension,
                (*ConfigInfo).AdapterInterfaceType,
                (*ConfigInfo).SystemIoBusNumber,
                IoBasePort1,
                8,
                1, // TRUE
            ) as PUCHAR;

            // Check if ioSpace accessible.
            if ioSpace.is_null() {
                channel += 1;
                continue;
            }

            // Select master.
            ScsiPortWritePortUchar(
                &mut (*(ioSpace as PIDE_REGISTERS_1)).DriveSelect as *mut UCHAR as PUCHAR,
                0xA0,
            );

            // Check if card at this address.
            ScsiPortWritePortUchar(
                &mut (*(ioSpace as PIDE_REGISTERS_1)).CylinderLow as *mut UCHAR as PUCHAR,
                0xAA,
            );

            // Check if identifier can be read back.
            statusByte = ScsiPortReadPortUchar(
                &mut (*(ioSpace as PIDE_REGISTERS_1)).CylinderLow as *mut UCHAR as PUCHAR,
            );
            if statusByte != 0xAA {
                // Select slave.
                ScsiPortWritePortUchar(
                    &mut (*(ioSpace as PIDE_REGISTERS_1)).DriveSelect as *mut UCHAR as PUCHAR,
                    0xB0,
                );

                // See if slave is present.
                ScsiPortWritePortUchar(
                    &mut (*(ioSpace as PIDE_REGISTERS_1)).CylinderLow as *mut UCHAR as PUCHAR,
                    0xAA,
                );

                statusByte = ScsiPortReadPortUchar(
                    &mut (*(ioSpace as PIDE_REGISTERS_1)).CylinderLow as *mut UCHAR as PUCHAR,
                );
                if statusByte != 0xAA {
                    // No controller at this base address.
                    ScsiPortFreeDeviceBase(HwDeviceExtension, ioSpace as PVOID);

                    // If the chip is there, but we couldn't find the primary channel, try the secondary.
                    // If we couldn't find a secondary, who cares.
                    if channel == 1 {
                        break 'channelLoop; // goto setStatusAndExit
                    } else {
                        channel += 1;
                        continue;
                    }
                }
            }

            // Record base IO address.
            (*deviceExtension).BaseIoAddress1[channel as usize] = ioSpace as PIDE_REGISTERS_1;

            // Get the system physical address for the second IO range.
            ioSpace = ScsiPortGetDeviceBase(
                HwDeviceExtension,
                (*ConfigInfo).AdapterInterfaceType,
                (*ConfigInfo).SystemIoBusNumber,
                IoBasePort2,
                1,
                1, // TRUE
            ) as PUCHAR;

            (*deviceExtension).BaseIoAddress2[channel as usize] = ioSpace as PIDE_REGISTERS_2;

            (*deviceExtension).NumberChannels = 2;

            // Indicate only one bus.
            (*ConfigInfo).NumberOfBuses = 1;

            // Indicate four devices can be attached to the adapter, since we
            // have to serialize access to the two channels.
            (*ConfigInfo).MaximumNumberOfTargets = 4;

            // Indicate maximum transfer length is 64k.
            (*ConfigInfo).MaximumTransferLength = 0x10000;

            // Since we will always pick up this part, and not atdisk, so indicate.
            atapiOnly = 0; // FALSE

            // Save the Interrupt Mode for later use
            (*deviceExtension).InterruptMode = (*ConfigInfo).InterruptMode;

            // Search for devices on this controller.
            if FindDevices(HwDeviceExtension, atapiOnly, channel) != 0 {
                deviceFound = 1; // TRUE
            }

            // Claim primary or secondary ATA IO range.
            if ScsiPortConvertPhysicalAddressToUlong(IoBasePort1) == AdapterAddresses[0] {
                (*ConfigInfo).AtdiskPrimaryClaimed = 1; // TRUE
                (*deviceExtension).PrimaryAddress = 1; // TRUE
            } else if ScsiPortConvertPhysicalAddressToUlong(IoBasePort2) == AdapterAddresses[1] {
                (*ConfigInfo).AtdiskSecondaryClaimed = 1; // TRUE
                (*deviceExtension).PrimaryAddress = 0; // FALSE
            }

            channel += 1;
        }
    }

    // setStatusAndExit:
    if deviceFound != 0 {
        *Again = 1; // TRUE
        return SP_RETURN_FOUND;
    }

    *Again = 0; // FALSE
    SP_RETURN_NOT_FOUND
}

// File-scope statics for AtapiFindPCIController (C `static ULONG functionNumber,
// slotNumber, controllers;`). The miniport is single-threaded through SCSIPORT;
// these persist across calls exactly as the C statics do.
static mut PCIC_functionNumber: ULONG = 0;
static mut PCIC_slotNumber: ULONG = 0;
static mut PCIC_controllers: ULONG = 0;

// atapi.c:3012
pub unsafe extern "system" fn AtapiFindPCIController(
    HwDeviceExtension: PVOID,
    Context: PVOID,
    BusInformation: PVOID,
    ArgumentString: PCHAR,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    Again: PBOOLEAN,
) -> ULONG {
    let deviceExtension: PHW_DEVICE_EXTENSION = HwDeviceExtension as PHW_DEVICE_EXTENSION;
    let adapterCount: *mut ULONG = Context as *mut ULONG;
    let mut channel: ULONG = 0;
    // static functionNumber, slotNumber, controllers — module statics above.
    let mut i: ULONG;
    let mut j: ULONG;
    let mut ioSpace: PUCHAR;
    let mut atapiOnly: BOOLEAN;
    let mut lastSlot: BOOLEAN;
    let mut controllerFound: BOOLEAN = 0; // FALSE
    let mut deviceFound: BOOLEAN = 0; // FALSE
    let mut statusByte: UCHAR;

    // The following table specifies the ports to be checked when searching for
    // an IDE controller.  A zero entry terminates the search.
    const AdapterAddresses: [ULONG; 5] = [0x1F0, 0x170, 0x1e8, 0x168, 0];

    // The following table specifies interrupt levels corresponding to the
    // port addresses in the previous table.
    const InterruptLevels: [ULONG; 5] = [14, 15, 11, 10, 0];

    if deviceExtension.is_null() {
        return SP_RETURN_ERROR;
    }

    // Since scsiport will call this function first before it calls AtapiFindController
    // we need to bypass it if we have data installed in ConfigInfo, by the pcmcia driver.
    if ScsiPortConvertPhysicalAddressToUlong((*access_range_ptr(ConfigInfo).add(0)).RangeStart) != 0
    {
        return AtapiFindController(
            HwDeviceExtension,
            Context,
            BusInformation,
            ArgumentString,
            ConfigInfo,
            Again,
        );
    }

    // BROKEN_ADAPTERS = sizeof(BrokenAdapters)/sizeof(BROKEN_CONTROLLER_INFORMATION)
    let BROKEN_ADAPTERS: ULONG = BrokenAdapters.len() as ULONG;

    // Gronk PCI config space looking for the broken PCI IDE controllers.
    i = PCIC_controllers;
    while i < BROKEN_ADAPTERS {
        // Determine if both channels are enabled and have devices.
        lastSlot = 0; // FALSE

        if FindBrokenController(
            deviceExtension as PVOID,
            BrokenAdapters[i as usize].VendorId as PUCHAR,
            BrokenAdapters[i as usize].VendorIdLength,
            BrokenAdapters[i as usize].DeviceId as PUCHAR,
            BrokenAdapters[i as usize].DeviceIdLength,
            &raw mut PCIC_functionNumber,
            &raw mut PCIC_slotNumber,
            (*ConfigInfo).SystemIoBusNumber,
            &mut lastSlot,
        ) != 0
        {
            PCIC_slotNumber += 1;
            PCIC_functionNumber = 0;
            controllerFound = 1; // TRUE

            if AdapterAddresses[*adapterCount as usize] != 0 {
                j = 0;
                'jLoop: while j < 2 {
                    // Get the system physical address for this IO range.
                    ioSpace = ScsiPortGetDeviceBase(
                        HwDeviceExtension,
                        (*ConfigInfo).AdapterInterfaceType,
                        (*ConfigInfo).SystemIoBusNumber,
                        ScsiPortConvertUlongToPhysicalAddress(
                            AdapterAddresses[*adapterCount as usize] as ULONG_PTR,
                        ),
                        8,
                        1, // TRUE
                    ) as PUCHAR;

                    // Update the adapter count.
                    *adapterCount += 1;

                    // Check if ioSpace accessible.
                    if ioSpace.is_null() {
                        j += 1;
                        continue;
                    }

                    // Select master.
                    ScsiPortWritePortUchar(
                        &mut (*(ioSpace as PIDE_REGISTERS_1)).DriveSelect as *mut UCHAR as PUCHAR,
                        0xA0,
                    );

                    // Check if card at this address.
                    ScsiPortWritePortUchar(
                        &mut (*(ioSpace as PIDE_REGISTERS_1)).CylinderLow as *mut UCHAR as PUCHAR,
                        0xAA,
                    );

                    // Check if identifier can be read back.
                    statusByte = ScsiPortReadPortUchar(
                        &mut (*(ioSpace as PIDE_REGISTERS_1)).CylinderLow as *mut UCHAR as PUCHAR,
                    );
                    if statusByte != 0xAA {
                        // Select slave.
                        ScsiPortWritePortUchar(
                            &mut (*(ioSpace as PIDE_REGISTERS_1)).DriveSelect as *mut UCHAR
                                as PUCHAR,
                            0xB0,
                        );

                        // See if slave is present.
                        ScsiPortWritePortUchar(
                            &mut (*(ioSpace as PIDE_REGISTERS_1)).CylinderLow as *mut UCHAR
                                as PUCHAR,
                            0xAA,
                        );

                        statusByte = ScsiPortReadPortUchar(
                            &mut (*(ioSpace as PIDE_REGISTERS_1)).CylinderLow as *mut UCHAR
                                as PUCHAR,
                        );
                        if statusByte != 0xAA {
                            // No controller at this base address.
                            ScsiPortFreeDeviceBase(HwDeviceExtension, ioSpace as PVOID);

                            // If the chip is there, but we couldn't find the primary channel, try the secondary.
                            if j == 1 {
                                break 'jLoop; // goto setStatusAndExit
                            } else {
                                j += 1;
                                continue;
                            }
                        }
                    }

                    if controllerFound != 0 {
                        // Record base IO address.
                        (*deviceExtension).BaseIoAddress1[channel as usize] =
                            ioSpace as PIDE_REGISTERS_1;

                        // Fill in the access array information.
                        (*access_range_ptr(ConfigInfo).add(channel as usize)).RangeStart =
                            ScsiPortConvertUlongToPhysicalAddress(
                                AdapterAddresses[*adapterCount as usize - 1] as ULONG_PTR,
                            );

                        (*access_range_ptr(ConfigInfo).add(channel as usize)).RangeLength = 8;
                        (*access_range_ptr(ConfigInfo).add(channel as usize)).RangeInMemory = 0; // FALSE

                        // Indicate the interrupt level corresponding to this IO range.
                        if channel == 0 {
                            (*ConfigInfo).BusInterruptLevel =
                                InterruptLevels[*adapterCount as usize - 1];
                            (*ConfigInfo).InterruptMode = _KINTERRUPT_MODE_Latched;
                        } else {
                            (*ConfigInfo).BusInterruptLevel2 =
                                InterruptLevels[*adapterCount as usize - 1];
                            (*ConfigInfo).InterruptMode2 = _KINTERRUPT_MODE_Latched;
                        }

                        // Get the system physical address for the second IO range.
                        ioSpace = ScsiPortGetDeviceBase(
                            HwDeviceExtension,
                            (*ConfigInfo).AdapterInterfaceType,
                            (*ConfigInfo).SystemIoBusNumber,
                            ScsiPortConvertUlongToPhysicalAddress(
                                (AdapterAddresses[*adapterCount as usize - 1] + 0x206) as ULONG_PTR,
                            ),
                            1,
                            1, // TRUE
                        ) as PUCHAR;

                        (*deviceExtension).BaseIoAddress2[channel as usize] =
                            ioSpace as PIDE_REGISTERS_2;

                        (*deviceExtension).NumberChannels = 2;

                        // Indicate only one bus.
                        (*ConfigInfo).NumberOfBuses = 1;

                        // Indicate four devices can be attached to the adapter.
                        (*ConfigInfo).MaximumNumberOfTargets = 4;

                        // Indicate maximum transfer length is 64k.
                        (*ConfigInfo).MaximumTransferLength = 0x10000;

                        // Since we will always pick up this part, and not atdisk, so indicate.
                        atapiOnly = 0; // FALSE

                        // Save the Interrupt Mode for later use
                        (*deviceExtension).InterruptMode = (*ConfigInfo).InterruptMode;

                        // Search for devices on this controller.
                        // FindDevices(..., channel++) — post-increment.
                        let chan_arg = channel;
                        channel += 1;
                        if FindDevices(HwDeviceExtension, atapiOnly, chan_arg) != 0 {
                            deviceFound = 1; // TRUE
                        }

                        // Claim primary or secondary ATA IO range.
                        if *adapterCount == 1 {
                            (*ConfigInfo).AtdiskPrimaryClaimed = 1; // TRUE
                            (*deviceExtension).PrimaryAddress = 1; // TRUE
                        } else if *adapterCount == 2 {
                            (*ConfigInfo).AtdiskSecondaryClaimed = 1; // TRUE
                            (*deviceExtension).PrimaryAddress = 0; // FALSE
                        }
                    }

                    j += 1;
                }
            }
        }

        // setStatusAndExit:
        if lastSlot != 0 {
            PCIC_slotNumber = 0;
            PCIC_functionNumber = 0;
        }

        PCIC_controllers = i;

        if controllerFound != 0 && deviceFound != 0 {
            *Again = 1; // TRUE
            return SP_RETURN_FOUND;
        }

        i += 1;
    }

    // The entire table has been searched and no adapters have been found.
    *Again = 0; // FALSE

    SP_RETURN_NOT_FOUND
}

// atapi.c:6175
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(DriverObject: PVOID, Argument2: PVOID) -> ULONG {
    let mut hwInitializationData: HW_INITIALIZATION_DATA = core::mem::zeroed();
    let mut adapterCount: ULONG;
    let mut i: ULONG;
    let mut statusToReturn: ULONG;
    let mut newStatus: ULONG;

    statusToReturn = 0xffffffff;

    // Zero out structure.
    AtapiZeroMemory(
        &mut hwInitializationData as *mut HW_INITIALIZATION_DATA as PUCHAR,
        size_of::<HW_INITIALIZATION_DATA>() as ULONG,
    );

    // Set size of hwInitializationData.
    hwInitializationData.HwInitializationDataSize = size_of::<HW_INITIALIZATION_DATA>() as ULONG;

    // Set entry points. The bindgen callback fields are
    //   Option<unsafe extern "stdcall" fn(...)>
    // while these sibling fns are `extern "system"` — ABI-identical on i386
    // (stdcall) and amd64/arm64 (cdecl) but a *distinct Rust type*, so transmute
    // the fn-address through *const () into the option-fn-ptr field type.
    hwInitializationData.HwInitialize = Some(core::mem::transmute(
        AtapiHwInitialize as unsafe extern "system" fn(PVOID) -> BOOLEAN,
    ));
    hwInitializationData.HwResetBus = Some(core::mem::transmute(
        AtapiResetController as unsafe extern "system" fn(PVOID, ULONG) -> BOOLEAN,
    ));
    hwInitializationData.HwStartIo = Some(core::mem::transmute(
        AtapiStartIo as unsafe extern "system" fn(PVOID, crate::types::PSCSI_REQUEST_BLOCK) -> BOOLEAN,
    ));
    hwInitializationData.HwInterrupt = Some(core::mem::transmute(
        AtapiInterrupt as unsafe extern "system" fn(PVOID) -> BOOLEAN,
    ));

    // Specify size of extensions.
    hwInitializationData.DeviceExtensionSize = size_of::<HW_DEVICE_EXTENSION>() as ULONG;
    hwInitializationData.SpecificLuExtensionSize = size_of::<HW_LU_EXTENSION>() as ULONG;

    // Indicate PIO device.
    hwInitializationData.MapBuffers = 1; // TRUE

    // Native Mode Devices.
    // NUM_NATIVE_MODE_ADAPTERS = sizeof(NativeModeAdapters)/sizeof(...) = array length.
    let NUM_NATIVE_MODE_ADAPTERS: ULONG = NativeModeAdapters.len() as ULONG;
    i = 0;
    while i < NUM_NATIVE_MODE_ADAPTERS {
        hwInitializationData.HwFindAdapter = Some(core::mem::transmute(
            AtapiFindNativeModeController
                as unsafe extern "system" fn(
                    PVOID,
                    PVOID,
                    PVOID,
                    PCHAR,
                    PPORT_CONFIGURATION_INFORMATION,
                    PBOOLEAN,
                ) -> ULONG,
        ));
        hwInitializationData.NumberOfAccessRanges = 4;
        hwInitializationData.AdapterInterfaceType = _INTERFACE_TYPE_PCIBus as INTERFACE_TYPE;

        hwInitializationData.VendorId = NativeModeAdapters[i as usize].VendorId as PVOID;
        hwInitializationData.VendorIdLength =
            NativeModeAdapters[i as usize].VendorIdLength as USHORT;
        hwInitializationData.DeviceId = NativeModeAdapters[i as usize].DeviceId as PVOID;
        hwInitializationData.DeviceIdLength =
            NativeModeAdapters[i as usize].DeviceIdLength as USHORT;

        newStatus = ScsiPortInitialize(
            DriverObject,
            Argument2,
            &mut hwInitializationData as *mut HW_INITIALIZATION_DATA as PVOID,
            (i as ULONG_PTR) as PVOID,
        );
        if newStatus < statusToReturn {
            statusToReturn = newStatus;
        }

        i += 1;
    }

    hwInitializationData.VendorId = core::ptr::null_mut();
    hwInitializationData.VendorIdLength = 0;
    hwInitializationData.DeviceId = core::ptr::null_mut();
    hwInitializationData.DeviceIdLength = 0;

    // The adapter count is used by the find adapter routine to track how
    // which adapter addresses have been tested.
    adapterCount = 0;

    hwInitializationData.HwFindAdapter = Some(core::mem::transmute(
        AtapiFindPCIController
            as unsafe extern "system" fn(
                PVOID,
                PVOID,
                PVOID,
                PCHAR,
                PPORT_CONFIGURATION_INFORMATION,
                PBOOLEAN,
            ) -> ULONG,
    ));
    hwInitializationData.NumberOfAccessRanges = 4;
    hwInitializationData.AdapterInterfaceType = _INTERFACE_TYPE_Isa as INTERFACE_TYPE;

    newStatus = ScsiPortInitialize(
        DriverObject,
        Argument2,
        &mut hwInitializationData as *mut HW_INITIALIZATION_DATA as PVOID,
        &mut adapterCount as *mut ULONG as PVOID,
    );
    if newStatus < statusToReturn {
        statusToReturn = newStatus;
    }

    // Indicate 2 access ranges and reset FindAdapter.
    hwInitializationData.NumberOfAccessRanges = 2;
    hwInitializationData.HwFindAdapter = Some(core::mem::transmute(
        AtapiFindController
            as unsafe extern "system" fn(
                PVOID,
                PVOID,
                PVOID,
                PCHAR,
                PPORT_CONFIGURATION_INFORMATION,
                PBOOLEAN,
            ) -> ULONG,
    ));

    // Indicate ISA bustype.
    hwInitializationData.AdapterInterfaceType = _INTERFACE_TYPE_Isa as INTERFACE_TYPE;

    // Call initialization for ISA bustype.
    newStatus = ScsiPortInitialize(
        DriverObject,
        Argument2,
        &mut hwInitializationData as *mut HW_INITIALIZATION_DATA as PVOID,
        &mut adapterCount as *mut ULONG as PVOID,
    );
    if newStatus < statusToReturn {
        statusToReturn = newStatus;
    }

    // Set up for MCA.
    hwInitializationData.AdapterInterfaceType = _INTERFACE_TYPE_MicroChannel as INTERFACE_TYPE;
    adapterCount = 0;

    newStatus = ScsiPortInitialize(
        DriverObject,
        Argument2,
        &mut hwInitializationData as *mut HW_INITIALIZATION_DATA as PVOID,
        &mut adapterCount as *mut ULONG as PVOID,
    );
    if newStatus < statusToReturn {
        statusToReturn = newStatus;
    }

    statusToReturn
}

/// Helper: the C code accesses `(*ConfigInfo->AccessRanges)[idx]`, where
/// `AccessRanges` is a pointer to a zero-length array `[ACCESS_RANGE; 0]`. Indexing
/// a ZST `[T; 0]` would step 0 bytes (the ZST-array `.add()` hazard), so cast the
/// pointer to a real `*mut ACCESS_RANGE` and index that — matching the C pointer
/// arithmetic exactly.
#[inline]
unsafe fn access_range_ptr(ConfigInfo: PPORT_CONFIGURATION_INFORMATION) -> *mut ACCESS_RANGE {
    (*ConfigInfo).AccessRanges as *mut ACCESS_RANGE
}
