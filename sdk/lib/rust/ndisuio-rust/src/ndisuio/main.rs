use crate::prelude::*;
use ntkrnl_sys::ddk::{IoCreateDevice, IoCreateSymbolicLink, IoDeleteDevice, IoDeleteSymbolicLink, LIST_ENTRY, PDEVICE_OBJECT, PDRIVER_OBJECT, PUNICODE_STRING};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_C51043CB7165: [u16; 8] = [0x004Eu16, 0x0044, 0x0049, 0x0053, 0x0055, 0x0049, 0x004F, 0x0000];

#[no_mangle]
pub static mut GlobalDeviceObject: PDEVICE_OBJECT =
    ::core::ptr::null::<_DEVICE_OBJECT>() as *mut _DEVICE_OBJECT;

#[no_mangle]
pub static mut GlobalProtocolHandle: NDIS_HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut GlobalAdapterListLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut GlobalAdapterList: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

#[no_mangle]
pub static mut ProtocolName: NDIS_STRING = _UNICODE_STRING {
        Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 8]>() as usize)
            .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
            as USHORT,
        MaximumLength: (::core::mem::size_of::<[::core::ffi::c_ushort; 8]>() as usize)
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>() as usize)
            as USHORT,
        Buffer: RUSTCONV_C2R_WSTR_C51043CB7165.as_ptr() as PWCH,
    };

// #define NDEBUG (empty object-like macro, no value)

#[no_mangle]
pub unsafe extern "system" fn NduUnload(DriverObject: PDRIVER_OBJECT) {
    // DPRINT("NDISUIO: Unloaded\n");
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NDIS_STATUS = 0;
    let mut Chars: NDIS_PROTOCOL_CHARACTERISTICS = core::mem::zeroed();
    let mut NtDeviceName: _UNICODE_STRING = RTL_CONSTANT_STRING!(NDISUIO_DEVICE_NAME_NT);
    let mut DosDeviceName: _UNICODE_STRING = RTL_CONSTANT_STRING!(NDISUIO_DEVICE_NAME_DOS);

    /* Setup dispatch functions */
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        Some(NduDispatchCreate);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
        Some(NduDispatchClose);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
        Some(NduDispatchDeviceControl);
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] =
        Some(NduDispatchRead);
    (*DriverObject).MajorFunction[IRP_MJ_WRITE as usize] =
        Some(NduDispatchWrite);
    (*DriverObject).DriverUnload =
        Some(NduUnload);

    /* Setup global state */
    InitializeListHead(core::ptr::addr_of_mut!(GlobalAdapterList));
    KeInitializeSpinLock(core::ptr::addr_of_mut!(GlobalAdapterListLock));

    /* Create the NDISUIO device object */
    Status = IoCreateDevice(
        DriverObject,
        0 as ULONG,
        core::ptr::addr_of_mut!(NtDeviceName),
        FILE_DEVICE_NETWORK as DEVICE_TYPE,
        FILE_DEVICE_SECURE_OPEN as ULONG,
        FALSE as BOOLEAN,
        core::ptr::addr_of_mut!(GlobalDeviceObject),
    ) as NDIS_STATUS;
    if !NT_SUCCESS!(Status as NTSTATUS) {
        DPRINT1!("Failed to create device object with status 0x%x\n", Status);
        return Status as NTSTATUS;
    }

    /* NduDispatchRead/Write use Irp->AssociatedIrp.SystemBuffer, which only buffered
     * I/O fills in: without the flag every WriteFile handed NDIS a NULL buffer.
     * (DeviceType and Characteristics above were also passed swapped.) */
    (*GlobalDeviceObject).Flags |= DO_BUFFERED_IO as ULONG;

    /* Create a symbolic link into the DOS devices namespace */
    Status = IoCreateSymbolicLink(
        core::ptr::addr_of_mut!(DosDeviceName),
        core::ptr::addr_of_mut!(NtDeviceName),
    ) as NDIS_STATUS;
    if !NT_SUCCESS!(Status as NTSTATUS) {
        DPRINT1!("Failed to create symbolic link with status 0x%x\n", Status);
        IoDeleteDevice(GlobalDeviceObject);
        return Status as NTSTATUS;
    }

    /* Register the protocol with NDIS */
    RtlZeroMemory!(
        core::ptr::addr_of_mut!(Chars) as *mut core::ffi::c_void,
        core::mem::size_of::<NDIS_PROTOCOL_CHARACTERISTICS>() as SIZE_T,
    );
    Chars.__bindgen_anon_1.MajorNdisVersion = NDIS_MAJOR_VERSION as UCHAR;
    Chars.__bindgen_anon_1.MinorNdisVersion = NDIS_MINOR_VERSION as UCHAR;
    Chars.__bindgen_anon_1.OpenAdapterCompleteHandler = Some(NduOpenAdapterComplete);
    Chars.__bindgen_anon_1.CloseAdapterCompleteHandler =
        Some(NduCloseAdapterComplete);
    Chars.__bindgen_anon_1.PnPEventHandler =
        Some(NduNetPnPEvent);
    Chars.__bindgen_anon_1.__bindgen_anon_2.SendCompleteHandler =
        Some(NduSendComplete);
    Chars


        .__bindgen_anon_1.__bindgen_anon_3.TransferDataCompleteHandler = Some(NduTransferDataComplete);
    Chars.__bindgen_anon_1.ResetCompleteHandler =
        Some(NduResetComplete);
    Chars.__bindgen_anon_1.RequestCompleteHandler = Some(NduRequestComplete);
    Chars.__bindgen_anon_1.__bindgen_anon_4.ReceiveHandler = Some(NduReceive);
    Chars.__bindgen_anon_1.ReceiveCompleteHandler =
        Some(NduReceiveComplete);
    Chars.__bindgen_anon_1.StatusHandler =
        Some(NduStatus);
    Chars.__bindgen_anon_1.StatusCompleteHandler =
        Some(NduStatusComplete);
    Chars.__bindgen_anon_1.Name = ProtocolName;
    Chars.__bindgen_anon_1.BindAdapterHandler = Some(NduBindAdapter);
    Chars.__bindgen_anon_1.UnbindAdapterHandler = Some(NduUnbindAdapter);

    NdisRegisterProtocol(
        core::ptr::addr_of_mut!(Status),
        core::ptr::addr_of_mut!(GlobalProtocolHandle),
        core::ptr::addr_of_mut!(Chars),
        core::mem::size_of::<NDIS_PROTOCOL_CHARACTERISTICS>() as UINT,
    );
    if Status != NDIS_STATUS_SUCCESS {
        DPRINT1!("Failed to register protocol with status 0x%x\n", Status);
        IoDeleteSymbolicLink(core::ptr::addr_of_mut!(DosDeviceName));
        IoDeleteDevice(GlobalDeviceObject);
        return Status as NTSTATUS;
    }

    DPRINT!("NDISUIO: Loaded\n");

    return STATUS_SUCCESS;
}
