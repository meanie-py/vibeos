//! generic/kdpci.c — kernel-debugger PCI configurator.
//!
//! Walks the PCI bus looking for a device matching a DEBUG_DEVICE_DESCRIPTOR,
//! probes its BARs, maps memory regions, enables decodes, and registers it
//! under \\Registry\\Machine\\System\\CurrentControlSet\\Services\\PCI\\Debug.
//!
//! Compiled for both i386 and amd64.
//! DPRINT0 is the EARLY_DEBUG hook; we drop the call sites entirely.
//! The PCI resource allocator (PCI_BRIDGE_TYPE, PCI_CARDBUS_BRIDGE_TYPE
//! cases) is still a TODO — same as the C source.

use core::ptr;

use ntkrnl_sys::ddk::{ZwClose, ZwSetValueKey};
use ntkrnl_sys::hal_ddk::{
    HalpAllocPhysicalMemory, HalpMapPhysicalMemory64, HalpOpenRegistryKey,
    HalpUnmapVirtualAddress, BOOLEAN,
    CmResourceTypeMemory, CmResourceTypePort, MAXIMUM_DEBUG_BARS, NTSTATUS, PCI_ADDRESS_IO_SPACE,
    PCI_ADDRESS_IO_ADDRESS_MASK, PCI_ADDRESS_MEMORY_ADDRESS_MASK, PCI_ADDRESS_MEMORY_TYPE_MASK,
    PCI_BRIDGE_TYPE, PCI_CARDBUS_BRIDGE_TYPE, PCI_DEVICE_TYPE, PCI_ENABLE_BUS_MASTER,
    PCI_ENABLE_IO_SPACE, PCI_ENABLE_MEMORY_SPACE, PCI_INVALID_VENDORID, PCI_MAX_DEVICES,
    PCI_MAX_FUNCTION, PCI_TYPE_64BIT, PDEBUG_DEVICE_DESCRIPTOR, PHYSICAL_ADDRESS, PUCHAR,
    PVOID, ULONG, ULONG64, USHORT, _PCI_COMMON_HEADER as PciCommonHeader,
    _PCI_SLOT_NUMBER as PciSlotNumber, _PCI_TYPE1_CFG_CYCLE_BITS as PciType1CfgCycleBits,
};
// Phase0 PCI accessors: use the in-crate stdcall defs directly — the hal_ddk
// bindgen decl is cdecl and folds against the in-module stdcall def (Â§9.99).
use super::pcibus::{HalpPhase0GetPciDataByOffset, HalpPhase0SetPciDataByOffset};
use ntkrnl_sys::ddk::{HANDLE, PHANDLE, RtlIntegerToUnicodeString, UNICODE_STRING};

// RtlInitEmptyUnicodeString is FORCEINLINE in NDK — no exported symbol.
// Inline it manually.
#[inline]
unsafe fn rtl_init_empty_unicode_string(
    s: *mut UNICODE_STRING,
    buffer: *mut u16,
    buffer_size: USHORT,
) {
    unsafe {
        (*s).Length = 0;
        (*s).MaximumLength = buffer_size;
        (*s).Buffer = buffer;
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC000_0001u32 as NTSTATUS;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000_009Au32 as NTSTATUS;
const STATUS_DEVICE_DOES_NOT_EXIST: NTSTATUS = 0xC000_00C0u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER_1: NTSTATUS = 0xC000_00EFu32 as NTSTATUS;
const KEY_ALL_ACCESS: ULONG = 0xF003F;
const REG_DWORD: ULONG = 4;
const PCI_MULTIFUNCTION: u8 = 0x80;
const PAGE_SIZE: ULONG = 4096;

// FIELD_OFFSET(PCI_COMMON_CONFIG, DeviceSpecific) = sizeof(PCI_COMMON_HEADER).
// PCI_COMMON_HEADER is 64 bytes (16-byte top + 48-byte type-specific union).
const PCI_COMMON_HDR_LENGTH: ULONG = 64;

#[inline]
fn nt_success(s: NTSTATUS) -> bool {
    s >= 0
}

#[inline]
fn bytes_to_pages(bytes: ULONG) -> ULONG {
    (bytes + PAGE_SIZE - 1) / PAGE_SIZE
}

// FIELD_OFFSET(PCI_COMMON_HEADER, Command) — 4 bytes in (after VendorID + DeviceID).
const PCI_COMMAND_OFFSET: ULONG = 4;

// FIELD_OFFSET(PCI_COMMON_HEADER, u.type0.BaseAddresses[i]) — header is 16 bytes,
// then BaseAddresses array starts at byte 16, each entry 4 bytes.
#[inline]
fn pci_bar_offset(i: ULONG) -> ULONG {
    16 + i * 4
}

// ---------------------------------------------------------------------------
// HalpPciDebuggingDevice — global, sized [PCI_TYPE1_CFG_CYCLE_BITS; 2].
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut HalpPciDebuggingDevice: [PciType1CfgCycleBits; 2] = [
    unsafe { core::mem::zeroed() },
    unsafe { core::mem::zeroed() },
];

// ---------------------------------------------------------------------------
// HalpPciBarLength — figure out the BAR length from the probed value.
// ---------------------------------------------------------------------------

unsafe fn halp_pci_bar_length(current_bar: ULONG, next_bar: ULONG) -> ULONG {
    let mut bar: ULONG64 = current_bar as ULONG64;
    let mut length: ULONG;

    if current_bar & PCI_ADDRESS_IO_SPACE != 0 {
        length = 1 << 2;
    } else {
        if (current_bar & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_64BIT {
            bar = ((next_bar as ULONG64) << 32) | (current_bar as ULONG64);
        }
        length = 1 << 4;
    }

    while (bar & length as ULONG64) == 0 && length != 0 {
        length <<= 1;
    }
    length
}

// ---------------------------------------------------------------------------
// HalpConfigureDebuggingDevice — configure a matched PCI debug device.
// ---------------------------------------------------------------------------

unsafe fn halp_configure_debugging_device(
    pci_device: PDEBUG_DEVICE_DESCRIPTOR,
    pci_bus: ULONG,
    pci_slot: PciSlotNumber,
    pci_config: *mut PciCommonHeader,
) -> bool {
    unsafe {
        let mut register: ULONG;

        // Disable IO+memory decode while we probe BAR sizes.
        register = ((*pci_config).Command as ULONG)
            & !(PCI_ENABLE_MEMORY_SPACE | PCI_ENABLE_IO_SPACE);
        let mut reg_word = register as USHORT;
        HalpPhase0SetPciDataByOffset(
            pci_bus,
            pci_slot,
            &mut reg_word as *mut _ as PVOID,
            PCI_COMMAND_OFFSET,
            core::mem::size_of::<USHORT>() as ULONG,
        );

        let mut i: ULONG = 0;
        while i < MAXIMUM_DEBUG_BARS {
            let device_address = (*pci_device).BaseAddress.as_mut_ptr().add(i as usize);
            (*device_address).Valid = 0;

            // Probe BAR length: write all-ones, read back masked value.
            register = 0xFFFF_FFFF;
            HalpPhase0SetPciDataByOffset(
                pci_bus,
                pci_slot,
                &mut register as *mut _ as PVOID,
                pci_bar_offset(i),
                core::mem::size_of::<ULONG>() as ULONG,
            );
            HalpPhase0GetPciDataByOffset(
                pci_bus,
                pci_slot,
                &mut register as *mut _ as PVOID,
                pci_bar_offset(i),
                core::mem::size_of::<ULONG>() as ULONG,
            );
            // Restore the original BAR.
            let mut original_bar = (*pci_config).u.type0.BaseAddresses[i as usize];
            HalpPhase0SetPciDataByOffset(
                pci_bus,
                pci_slot,
                &mut original_bar as *mut _ as PVOID,
                pci_bar_offset(i),
                core::mem::size_of::<ULONG>() as ULONG,
            );

            let next_bar = if i < MAXIMUM_DEBUG_BARS - 1 {
                (*pci_config).u.type0.BaseAddresses[(i + 1) as usize]
            } else {
                0
            };

            let length = halp_pci_bar_length(register, next_bar);
            if register == 0 || length == 0 {
                i += 1;
                continue;
            }

            if register & PCI_ADDRESS_IO_SPACE != 0 {
                // I/O space.
                (*device_address).Type = CmResourceTypePort as u8;
                (*device_address).Length = length;
                (*device_address).Valid = 1;
                let bar = (*pci_config).u.type0.BaseAddresses[i as usize];
                (*device_address).TranslatedAddress =
                    (bar & PCI_ADDRESS_IO_ADDRESS_MASK) as usize as PUCHAR;
            } else {
                // Memory space (32- or 64-bit).
                let mut physical_address: PHYSICAL_ADDRESS = core::mem::zeroed();
                let mut skip_bar = false;

                (*device_address).Type = CmResourceTypeMemory as u8;
                (*device_address).Length = length;
                (*device_address).Valid = 1;

                physical_address.QuadPart = ((*pci_config).u.type0.BaseAddresses[i as usize]
                    & PCI_ADDRESS_MEMORY_ADDRESS_MASK) as i64;
                if (register & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_64BIT {
                    let high = next_bar;
                    physical_address.QuadPart |= (high as i64) << 32;
                    skip_bar = true;
                }

                if skip_bar {
                    i += 1;
                }

                (*device_address).TranslatedAddress = HalpMapPhysicalMemory64(
                    physical_address,
                    bytes_to_pages(length),
                ) as PUCHAR;
            }

            i += 1;
        }

        (*pci_device).Bus = pci_bus;
        (*pci_device).Slot = pci_slot.u.AsULONG;
        (*pci_device).VendorID = (*pci_config).VendorID;
        (*pci_device).DeviceID = (*pci_config).DeviceID;
        (*pci_device).BaseClass = (*pci_config).BaseClass;
        (*pci_device).SubClass = (*pci_config).SubClass;
        (*pci_device).ProgIf = (*pci_config).ProgIf;

        // Enable decodes.
        (*pci_config).Command |= (PCI_ENABLE_MEMORY_SPACE
            | PCI_ENABLE_IO_SPACE
            | PCI_ENABLE_BUS_MASTER) as USHORT;
        let mut cmd = (*pci_config).Command;
        HalpPhase0SetPciDataByOffset(
            pci_bus,
            pci_slot,
            &mut cmd as *mut _ as PVOID,
            PCI_COMMAND_OFFSET,
            core::mem::size_of::<USHORT>() as ULONG,
        );

        true
    }
}

// ---------------------------------------------------------------------------
// HalpMatchDebuggingDevice — does this PCI config match the descriptor?
// ---------------------------------------------------------------------------

unsafe fn halp_match_debugging_device(
    pci_device: PDEBUG_DEVICE_DESCRIPTOR,
    pci_bus: ULONG,
    pci_slot: PciSlotNumber,
    pci_config: *const PciCommonHeader,
) -> bool {
    unsafe {
        // Wildcard bus/slot?
        if (*pci_device).Bus == 0xFFFF_FFFF && (*pci_device).Slot == 0xFFFF_FFFF {
            if (*pci_device).DeviceID == 0xFFFF && (*pci_device).VendorID == 0xFFFF {
                if (*pci_device).BaseClass == (*pci_config).BaseClass
                    && (*pci_device).SubClass == (*pci_config).SubClass
                {
                    if (*pci_device).ProgIf == 0xFF
                        || (*pci_device).ProgIf == (*pci_config).ProgIf
                    {
                        return true;
                    }
                }
            } else if (*pci_device).DeviceID == (*pci_config).DeviceID
                && (*pci_device).VendorID == (*pci_config).VendorID
            {
                return true;
            }
        } else if (*pci_device).Bus == pci_bus && (*pci_device).Slot == pci_slot.u.AsULONG {
            return true;
        }
        false
    }
}

// ---------------------------------------------------------------------------
// HalpFindMatchingDebuggingDevice — full PCI bus walk.
// ---------------------------------------------------------------------------

unsafe fn halp_find_matching_debugging_device(pci_device: PDEBUG_DEVICE_DESCRIPTOR) -> bool {
    unsafe {
        for bus_number in 0u32..0xFF {
            for device_number in 0u32..PCI_MAX_DEVICES {
                let mut function_number = 0u32;
                while function_number < PCI_MAX_FUNCTION {
                    let mut pci_slot: PciSlotNumber = core::mem::zeroed();
                    pci_slot.u.bits.set_DeviceNumber(device_number);
                    pci_slot.u.bits.set_FunctionNumber(function_number);
                    pci_slot.u.bits.set_Reserved(0);

                    let mut pci_config: PciCommonHeader = core::mem::zeroed();
                    let bytes = HalpPhase0GetPciDataByOffset(
                        bus_number,
                        pci_slot,
                        &mut pci_config as *mut _ as PVOID,
                        0,
                        PCI_COMMON_HDR_LENGTH,
                    );
                    if bytes != PCI_COMMON_HDR_LENGTH
                        || pci_config.VendorID as ULONG == PCI_INVALID_VENDORID
                        || pci_config.VendorID == 0
                    {
                        if function_number == 0 {
                            // Slot has neither single nor multi-func device.
                            break;
                        } else {
                            function_number += 1;
                            continue;
                        }
                    }

                    let header_type = pci_config.HeaderType & !PCI_MULTIFUNCTION;
                    match header_type as u32 {
                        x if x == PCI_DEVICE_TYPE => {
                            if halp_match_debugging_device(
                                pci_device,
                                bus_number,
                                pci_slot,
                                &pci_config,
                            ) && halp_configure_debugging_device(
                                pci_device,
                                bus_number,
                                pci_slot,
                                &mut pci_config,
                            ) {
                                return true;
                            }
                        }
                        x if x == PCI_BRIDGE_TYPE => {
                            // FIXME: PCI resource allocator not implemented.
                        }
                        x if x == PCI_CARDBUS_BRIDGE_TYPE => {
                            // FIXME: same.
                        }
                        _ => {}
                    }

                    if pci_config.HeaderType & PCI_MULTIFUNCTION == 0 {
                        // Single-function device — done with this slot.
                        break;
                    }

                    function_number += 1;
                }
            }
        }
        false
    }
}

// ---------------------------------------------------------------------------
// HalpRegisterPciDebuggingDeviceInfo
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpRegisterPciDebuggingDeviceInfo() {
    unsafe {
        let dbg_base: *mut PciType1CfgCycleBits =
            (&raw mut HalpPciDebuggingDevice).cast();

        // Any device claimed?
        let mut has_debugging_device = false;
        for i in 0..2usize {
            let bits = (*dbg_base.add(i)).__bindgen_anon_1;
            if bits.InUse() != 0 {
                has_debugging_device = true;
                break;
            }
        }
        if !has_debugging_device {
            return;
        }

        // L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\PCI\\Debug"
        //
        // Built at runtime rather than hand-encoded. A `const KEY_NAME: [u16; 60]` used to sit
        // here and was TRUNCATED — it spelled "...\\PCI\\Debu", missing the final 'g' (the
        // SERVICES_KEY class, field-manual §9.6-services). It was dead (`let _ = KEY_NAME;`)
        // because the author hit the bug, left a "g missing — pad it" note, and worked around it
        // with build_wstr instead of fixing it. Deleted 2026-07-15 after scan_wide_literals
        // decoded it and reported the drop; the live path below was always correct.
        let key_name_buf = build_wstr(b"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\PCI\\Debug\0");
        // build_wstr returns a fixed [u16; 64]: the string length is up to its
        // NUL, not the array length. (Length was (64 - 1) * 2, so the key name
        // carried three trailing NULs: "...\\PCI\\Debug\0\0\0".)
        let key_name_chars = key_name_buf
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(key_name_buf.len());
        let mut key_name: UNICODE_STRING = core::mem::zeroed();
        key_name.Length = (key_name_chars * 2) as USHORT;
        key_name.MaximumLength = (key_name_buf.len() * 2) as USHORT;
        key_name.Buffer = key_name_buf.as_ptr() as *mut u16;

        let mut handle: HANDLE = ptr::null_mut();
        let status = HalpOpenRegistryKey(
            &mut handle as PHANDLE,
            ptr::null_mut(),
            &mut key_name as *mut _ as _,
            KEY_ALL_ACCESS,
            1, // TRUE
        );
        if !nt_success(status) {
            return;
        }

        for i in 0..2u32 {
            let dbg = dbg_base.add(i as usize);
            if (*dbg).__bindgen_anon_1.InUse() == 0 {
                continue;
            }

            let mut string_buffer = [0u16; 8];
            let mut sub_key_name: UNICODE_STRING = core::mem::zeroed();
            rtl_init_empty_unicode_string(
                &mut sub_key_name,
                string_buffer.as_mut_ptr(),
                (string_buffer.len() * 2) as USHORT,
            );
            RtlIntegerToUnicodeString(i, 10, &mut sub_key_name);

            let mut key_handle: HANDLE = ptr::null_mut();
            let status = HalpOpenRegistryKey(
                &mut key_handle as PHANDLE,
                handle,
                &mut sub_key_name as *mut _ as _,
                KEY_ALL_ACCESS,
                1,
            );
            if !nt_success(status) {
                continue;
            }

            // Bus value
            let mut value: ULONG = (*dbg).__bindgen_anon_1.BusNumber();
            let bus_str = build_wstr(b"Bus\0");
            let mut value_name: UNICODE_STRING = core::mem::zeroed();
            value_name.Length = (3 * 2) as USHORT;
            value_name.MaximumLength = (4 * 2) as USHORT;
            value_name.Buffer = bus_str.as_ptr() as *mut u16;
            ZwSetValueKey(
                key_handle,
                &mut value_name,
                0,
                REG_DWORD,
                &mut value as *mut _ as PVOID,
                core::mem::size_of::<ULONG>() as ULONG,
            );

            // Slot value (DeviceNumber/FunctionNumber packed into PCI_SLOT_NUMBER)
            let mut pci_slot: PciSlotNumber = core::mem::zeroed();
            pci_slot.u.AsULONG = 0;
            pci_slot.u.bits.set_DeviceNumber((*dbg).__bindgen_anon_1.DeviceNumber());
            pci_slot.u.bits.set_FunctionNumber((*dbg).__bindgen_anon_1.FunctionNumber());
            value = pci_slot.u.AsULONG;

            let slot_str = build_wstr(b"Slot\0");
            let mut value_name: UNICODE_STRING = core::mem::zeroed();
            value_name.Length = (4 * 2) as USHORT;
            value_name.MaximumLength = (5 * 2) as USHORT;
            value_name.Buffer = slot_str.as_ptr() as *mut u16;
            ZwSetValueKey(
                key_handle,
                &mut value_name,
                0,
                REG_DWORD,
                &mut value as *mut _ as PVOID,
                core::mem::size_of::<ULONG>() as ULONG,
            );

            ZwClose(key_handle);
        }
        ZwClose(handle);
    }
}

// Tiny helper — convert ASCII byte string (with trailing NUL) to a UTF-16
// buffer for kernel UNICODE_STRINGs. Allocates on the stack via array.
// Returned slice is valid until end of caller's scope.
#[inline]
fn build_wstr(s: &[u8]) -> [u16; 64] {
    let mut buf = [0u16; 64];
    let n = s.len().min(64);
    let mut i = 0;
    while i < n {
        buf[i] = s[i] as u16;
        i += 1;
    }
    buf
}

// ---------------------------------------------------------------------------
// HalpReleasePciDeviceForDebugging
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpReleasePciDeviceForDebugging(
    pci_device: PDEBUG_DEVICE_DESCRIPTOR,
) -> NTSTATUS {
    unsafe {
        for i in 0..MAXIMUM_DEBUG_BARS {
            let device_address = (*pci_device).BaseAddress.as_mut_ptr().add(i as usize);
            if (*device_address).Type == CmResourceTypeMemory as u8
                && (*device_address).Valid != 0
            {
                HalpUnmapVirtualAddress(
                    (*device_address).TranslatedAddress as PVOID,
                    bytes_to_pages((*device_address).Length),
                );
                (*device_address).Valid = 0;
            }
        }
        STATUS_SUCCESS
    }
}

// ---------------------------------------------------------------------------
// HalpSetupPciDeviceForDebugging
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpSetupPciDeviceForDebugging(
    loader_block: PVOID,
    pci_device: PDEBUG_DEVICE_DESCRIPTOR,
) -> NTSTATUS {
    unsafe {
        if !halp_find_matching_debugging_device(pci_device) {
            return STATUS_DEVICE_DOES_NOT_EXIST;
        }

        if (*pci_device).Initialized != 0 {
            return STATUS_SUCCESS;
        }

        let mut pci_slot: PciSlotNumber = core::mem::zeroed();
        pci_slot.u.AsULONG = (*pci_device).Slot;

        let dbg_base: *mut PciType1CfgCycleBits =
            (&raw mut HalpPciDebuggingDevice).cast();

        // Already-present check.
        for i in 0..2usize {
            let dbg = dbg_base.add(i);
            let bits = (*dbg).__bindgen_anon_1;
            if bits.InUse() != 0
                && bits.DeviceNumber() == pci_slot.u.bits.DeviceNumber()
                && bits.FunctionNumber() == pci_slot.u.bits.FunctionNumber()
                && bits.BusNumber() == (*pci_device).Bus
            {
                return STATUS_UNSUCCESSFUL;
            }
        }

        // Save device location.
        let mut placed = false;
        for i in 0..2usize {
            let dbg = dbg_base.add(i);
            if (*dbg).__bindgen_anon_1.InUse() == 0 {
                (*dbg).__bindgen_anon_1.set_DeviceNumber(pci_slot.u.bits.DeviceNumber());
                (*dbg).__bindgen_anon_1.set_FunctionNumber(pci_slot.u.bits.FunctionNumber());
                (*dbg).__bindgen_anon_1.set_BusNumber((*pci_device).Bus);
                (*dbg).__bindgen_anon_1.set_InUse(1);
                (*pci_device).Initialized = 1;
                placed = true;
                break;
            }
        }
        if !placed {
            return STATUS_UNSUCCESSFUL;
        }

        if (*pci_device).Memory.Length == 0 {
            return STATUS_SUCCESS;
        }
        if loader_block.is_null() {
            return STATUS_INVALID_PARAMETER_1;
        }
        if (*pci_device).Memory.MaxEnd.QuadPart == 0 {
            (*pci_device).Memory.MaxEnd.QuadPart = -1; // ULONG64 -1
        }
        let max_address = core::cmp::min(
            (*pci_device).Memory.MaxEnd.QuadPart as ULONG64,
            0xFFFF_FFFFu64,
        );
        let page_count = bytes_to_pages((*pci_device).Memory.Length);

        let raw_phys =
            HalpAllocPhysicalMemory(loader_block as _, max_address, page_count as _, 0);
        let mut physical_address: PHYSICAL_ADDRESS = core::mem::zeroed();
        physical_address.QuadPart = raw_phys as i64;
        (*pci_device).Memory.Start = physical_address;
        if raw_phys == 0 {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        (*pci_device).Memory.VirtualAddress =
            HalpMapPhysicalMemory64(physical_address, page_count) as PVOID;

        STATUS_SUCCESS
    }
}
