use crate::prelude::*;
use ntkrnl_sys::ddk::{IoGetDeviceProperty, PCCHAR, va_list};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut PciIdeDebug: ULONG = 0 as ULONG;
#[no_mangle]
pub unsafe extern "C" fn PciIdeXChannelState(
    FdoExtension: *mut FDO_DEVICE_EXTENSION,
    Channel: u32,
) -> IDE_CHANNEL_STATE {
    let mut MiniportChannelEnabled: PCIIDE_CHANNEL_ENABLED;

    PAGED_CODE!();

    MiniportChannelEnabled = (*FdoExtension).Properties.PciIdeChannelEnabled;
    if !MiniportChannelEnabled.is_none() {
        return (MiniportChannelEnabled.unwrap_unchecked())(
            (*FdoExtension).MiniControllerExtension.as_ptr() as *mut core::ffi::c_void,
            Channel,
        );
    }

    return ChannelStateUnknown;
}

// HAND-PORTED (miniport.c PciIdeXDebugPrint): a C-variadic definition is `c_variadic`, which
// stable Rust does not have. The exported variadic symbol is a 3-line trampoline in
// drivers/storage/ide/pciidex/rust_varargs_shim.c; it hands this fixed-arity twin a POINTER
// to its va_list, and the level filter -- the only policy -- stays here. vDbgPrintEx takes the
// va_list by value, so the shim's vibeos_vDbgPrintEx dereferences the pointer for it.
extern "C" {
    fn vibeos_vDbgPrintEx(ComponentId: ULONG, Level: ULONG, Format: PCCH, ap: *mut c_void) -> ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn PciIdeXDebugPrint_v(
    mut DebugPrintLevel: ULONG,
    DebugMessage: PCCHAR,
    ap: *mut c_void,
) {
    /* Check if we can print anything */
    if DebugPrintLevel <= PciIdeDebug {
        DebugPrintLevel = 0;
    }
    vibeos_vDbgPrintEx(DPFLTR_PCIIDE_ID as ULONG, DebugPrintLevel, DebugMessage as PCCH, ap);
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXGetBusData(
    DeviceExtension: PVOID,
    Buffer: PVOID,
    ConfigDataOffset: ULONG,
    BufferLength: ULONG,
) -> NTSTATUS {
    let mut FdoExt: PFDO_DEVICE_EXTENSION;
    let mut Controller: PATA_CONTROLLER;
    let mut BytesRead: ULONG;

    INFO!(
        b"PciIdeXGetBusData(%p %p 0x%lx 0x%lx)\n\0".as_ptr() as PCCHAR,
        DeviceExtension,
        Buffer,
        ConfigDataOffset,
        BufferLength,
    );

    FdoExt = CONTAINING_RECORD!(
        DeviceExtension,
        FDO_DEVICE_EXTENSION,
        MiniControllerExtension,
    );
    Controller = &raw mut (*FdoExt).Controller as PATA_CONTROLLER;

    BytesRead = (*Controller).GetBusData.unwrap_unchecked()(
        (*Controller).BusInterfaceContext,
        PCI_WHICHSPACE_CONFIG,
        Buffer,
        ConfigDataOffset,
        BufferLength,
    );
    if BytesRead != BufferLength {
        return STATUS_UNSUCCESSFUL;
    }

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn PciIdeXSetBusData(
    DeviceExtension: PVOID,
    Buffer: PVOID,
    DataMask: PVOID,
    ConfigDataOffset: ULONG,
    BufferLength: ULONG,
) -> NTSTATUS {
    let mut FdoExt: *mut FDO_DEVICE_EXTENSION;
    let mut Controller: *mut ATA_CONTROLLER;
    let mut LocalBuffer: [u8; 4] = [0u8; 4];
    let mut i: ULONG;
    let mut BytesWritten: ULONG;
    let mut CurrentBuffer: *mut u8;
    let mut OldIrql: KIRQL = 0;
    let mut Status: NTSTATUS;

    let info_fmt = b"PciIdeXSetBusData(%p %p %p 0x%lx 0x%lx)\n\0".as_ptr() as *const i8;
    INFO!(info_fmt, DeviceExtension, Buffer, DataMask, ConfigDataOffset, BufferLength);

    // Optimize common case: use stack buffer if small enough
    if BufferLength <= LocalBuffer.len() as ULONG {
        CurrentBuffer = LocalBuffer.as_mut_ptr();
    } else {
        CurrentBuffer = ExAllocatePoolUninitialized(NonPagedPool, (BufferLength) as _, TAG_PCIIDEX) as *mut u8;
        if CurrentBuffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    // Compute containing record manually (equivalent to CONTAINING_RECORD)
    FdoExt = (DeviceExtension as usize)
        .wrapping_sub(core::mem::offset_of!(FDO_DEVICE_EXTENSION, MiniControllerExtension))
        as *mut FDO_DEVICE_EXTENSION;
    Controller = core::ptr::addr_of_mut!((*FdoExt).Controller);

    let lock_ptr = core::ptr::addr_of_mut!((*Controller).Lock);
    let old_irql_ptr = core::ptr::addr_of_mut!(OldIrql);

    // Labeled block to allow goto-like break
    let _: () = {
        KeAcquireSpinLock(lock_ptr, old_irql_ptr);

        Status = PciIdeXGetBusData(DeviceExtension, Buffer, ConfigDataOffset, BufferLength);
        if !NT_SUCCESS!(Status) {
            // goto Cleanup
            Status = Status; // break out of block, continue after block
        } else {
            i = 0;
            while i < BufferLength {
                let idx = i as usize;
                let data_mask_byte = *((DataMask as *const u8).add(idx)) as i32;
                let buffer_byte = *((Buffer as *const u8).add(idx)) as i32;
                let current_byte = *CurrentBuffer.add(idx) as i32;
                let result = (current_byte & !data_mask_byte) | (data_mask_byte & buffer_byte);
                *CurrentBuffer.add(idx) = result as u8;
                i = i.wrapping_add(1);
            }

            let set_bus_data_fn = (*Controller).SetBusData;
            BytesWritten = (set_bus_data_fn.unwrap_unchecked())(
                (*Controller).BusInterfaceContext,
                PCI_WHICHSPACE_CONFIG,
                CurrentBuffer as *mut c_void,
                ConfigDataOffset,
                BufferLength,
            );

            if BytesWritten != BufferLength {
                Status = STATUS_UNSUCCESSFUL;
            } else {
                Status = STATUS_SUCCESS;
            }
        }
    };

    // Cleanup:
    KeReleaseSpinLock(lock_ptr, OldIrql);

    if CurrentBuffer != LocalBuffer.as_mut_ptr() {
        ExFreePoolWithTag((CurrentBuffer).cast(), TAG_PCIIDEX);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PciIdexGetBusLocation(
    DeviceExtension: *mut core::ffi::c_void,
    BusLocation: *mut u32,
) -> i32 {
    let mut FdoExt: *mut FDO_DEVICE_EXTENSION;
    let mut Length: u32 = core::mem::zeroed();
    let mut Status: i32;

    FdoExt = CONTAINING_RECORD!(
        DeviceExtension,
        FDO_DEVICE_EXTENSION,
        MiniControllerExtension,
    );

    Status = IoGetDeviceProperty(
        (*FdoExt).Pdo,
        DevicePropertyAddress,
        core::mem::size_of::<*mut u32>() as u32,
        BusLocation as *mut core::ffi::c_void,
        &mut Length,
    );
    return Status;
}
