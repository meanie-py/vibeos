use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CCHAR, ExFreePool, GUID, IRP, IoGetNextIrpStackLocation, IoInitializeIrp, IoSetCompletionRoutine, KeInitializeSpinLock, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PWORKER_THREAD_ROUTINE, RtlAssert, WORK_QUEUE_ITEM, _WORK_QUEUE_TYPE_CriticalWorkQueue as CriticalWorkQueue};
use core::ffi::c_void;
use crate::types::*;
use crate::glue::*;
use crate::cb;

// ---- local inline helpers (not in any ntkrnl_sys module) ----

#[allow(non_snake_case)]
#[inline(always)]
fn IoSizeOfIrp(stack_size: CCHAR) -> USHORT {
    (core::mem::size_of::<IRP>()
        + stack_size as u8 as usize * core::mem::size_of::<IO_STACK_LOCATION>()) as USHORT
}

#[allow(non_snake_case)]
#[inline(always)]
unsafe fn IsEqualGUIDAligned(g1: *const GUID, g2: *const GUID) -> bool {
    core::ptr::read_unaligned(g1 as *const u64)
        == core::ptr::read_unaligned(g2 as *const u64)
        && core::ptr::read_unaligned((g1 as *const u64).add(1))
            == core::ptr::read_unaligned((g2 as *const u64).add(1))
}

#[allow(non_snake_case)]
#[inline(always)]
unsafe fn KsGateTurnInputOn(Gate: PKSGATE) {
    (*Gate).Count = (*Gate).Count.wrapping_add(1);
}

#[allow(non_snake_case)]
#[inline(always)]
unsafe fn KsGateTurnInputOff(Gate: PKSGATE) {
    (*Gate).Count = (*Gate).Count.wrapping_sub(1);
}

#[allow(non_snake_case)]
#[inline(always)]
unsafe fn ExInitializeWorkItem(
    item: *mut WORK_QUEUE_ITEM,
    routine: PWORKER_THREAD_ROUTINE,
    context: PVOID,
) {
    (*item).List.Flink = core::ptr::null_mut();
    (*item).WorkerRoutine = routine;
    (*item).Parameter = context;
}

#[allow(non_snake_case)]
#[inline(always)]
fn GET_ISO_URB_SIZE(n: ULONG) -> ULONG {
    (core::mem::size_of::<_URB_ISOCH_TRANSFER>()
        + n as usize * core::mem::size_of::<USBD_ISO_PACKET_DESCRIPTOR>()) as ULONG
}

#[allow(non_snake_case)]
#[inline(always)]
fn GET_SELECT_INTERFACE_REQUEST_SIZE(numPipes: UCHAR) -> ULONG {
    (core::mem::size_of::<_URB_SELECT_INTERFACE>()
        + numPipes.saturating_sub(1) as usize * core::mem::size_of::<USBD_PIPE_INFORMATION>())
        as ULONG
}

#[allow(non_snake_case)]
#[inline(always)]
unsafe fn UsbBuildSelectInterfaceRequest(
    Urb: PURB,
    Length: USHORT,
    ConfigurationHandle: USBD_CONFIGURATION_HANDLE,
    InterfaceNumber: UCHAR,
    AlternateSetting: UCHAR,
) {
    let s = (*Urb).__bindgen_anon_1.UrbSelectInterface.as_mut();
    s.Hdr.Function = URB_FUNCTION_SELECT_INTERFACE as USHORT;
    s.Hdr.Length = Length;
    s.ConfigurationHandle = ConfigurationHandle;
    s.Interface.InterfaceNumber = InterfaceNumber;
    s.Interface.AlternateSetting = AlternateSetting;
    s.Interface.Length = ((Length as usize)
        .wrapping_sub(core::mem::size_of::<_URB_SELECT_INTERFACE>())
        .wrapping_add(core::mem::size_of::<USBD_INTERFACE_INFORMATION>()))
        as USHORT;
}

#[allow(non_snake_case)]
#[inline(always)]
fn PtrToUlong(ptr: *mut c_void) -> ULONG {
    ptr as usize as ULONG
}

#[allow(non_snake_case)]
#[inline(always)]
fn UlongToPtr(val: ULONG) -> *mut c_void {
    val as usize as *mut c_void
}

macro_rules! ASSERT {
    ($e:expr) => { debug_assert!($e); };
}
macro_rules! NT_ASSERT {
    ($e:expr) => { debug_assert!($e); };
}

const MAXULONG: ULONG = 0xFFFF_FFFFu32;

// Shadow ke::KeAcquireSpinLock / ke::KeReleaseSpinLock to accept ddk's KSPIN_LOCK type.
// ddk::KSPIN_LOCK (the struct field type) and ke::KSPIN_LOCK differ as Rust types but
// have identical width per arch (u32 on i386, usize on amd64/arm64); the pointer cast
// bridges them portably.
#[allow(non_snake_case)]
#[inline(always)]
unsafe fn KeAcquireSpinLock(SpinLock: *mut ntkrnl_sys::ddk::KSPIN_LOCK, OldIrql: *mut KIRQL) {
    ntkrnl_sys::ke::KeAcquireSpinLock(SpinLock as *mut ntkrnl_sys::ke::KSPIN_LOCK, OldIrql);
}
#[allow(non_snake_case)]
#[inline(always)]
unsafe fn KeReleaseSpinLock(SpinLock: *mut ntkrnl_sys::ddk::KSPIN_LOCK, NewIrql: KIRQL) {
    ntkrnl_sys::ke::KeReleaseSpinLock(SpinLock as *mut ntkrnl_sys::ke::KSPIN_LOCK, NewIrql);
}

// Helper: get PIRP from a LIST_ENTRY pointer embedded in IRP.Tail.Overlay.__bindgen_anon_2.ListEntry
#[inline(always)]
unsafe fn irp_from_list_entry(entry: PLIST_ENTRY) -> PIRP {
    (entry as *mut u8).sub(
        core::mem::offset_of!(IRP, Tail.Overlay.__bindgen_anon_2.ListEntry)
    ) as PIRP
}

// Helper: transmute extern "system" completion routine to extern "system" (same ABI on i686-windows)
macro_rules! completion_stdcall {
    ($fn:expr) => {{ core::mem::transmute($fn as usize) }};
}

macro_rules! worker_stdcall {
    ($fn:expr) => {{ core::mem::transmute($fn as usize) }};
}

// ---- file-scope preamble (translated) ----
pub const PACKET_COUNT: u32 = 10;
#[no_mangle]
pub unsafe extern "C" fn GetMaxPacketSizeForInterface(
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR,
    DataFlow: KSPIN_DATAFLOW,
) -> NTSTATUS {
    let mut CommonDescriptor: PUSB_COMMON_DESCRIPTOR;
    let mut EndpointDescriptor: PUSB_ENDPOINT_DESCRIPTOR;

    CommonDescriptor = (InterfaceDescriptor as ULONG_PTR)
        .wrapping_add((*InterfaceDescriptor).bLength as ULONG_PTR)
        as PUSB_COMMON_DESCRIPTOR;
    debug_assert!((*InterfaceDescriptor).bNumEndpoints > 0);
    loop {
        if CommonDescriptor.is_null() {
            break;
        }
        if (*CommonDescriptor).bDescriptorType == USB_ENDPOINT_DESCRIPTOR_TYPE as UCHAR {
            EndpointDescriptor = CommonDescriptor as PUSB_ENDPOINT_DESCRIPTOR;
            return (*EndpointDescriptor).wMaxPacketSize as NTSTATUS;
        }

        if (*CommonDescriptor).bDescriptorType == USB_INTERFACE_DESCRIPTOR_TYPE as UCHAR {
            break;
        }

        if (CommonDescriptor as ULONG_PTR)
            .wrapping_add((*CommonDescriptor).bLength as ULONG_PTR)
            >= (ConfigurationDescriptor as ULONG_PTR)
                .wrapping_add((*ConfigurationDescriptor).wTotalLength as ULONG_PTR)
        {
            break;
        }

        CommonDescriptor = (CommonDescriptor as ULONG_PTR)
            .wrapping_add((*CommonDescriptor).bLength as ULONG_PTR)
            as PUSB_COMMON_DESCRIPTOR;
    }

    100
}
#[no_mangle]
pub unsafe extern "C" fn UsbAudioAllocCaptureUrbIso(
    PipeHandle: USBD_PIPE_HANDLE,
    MaxPacketSize: ULONG,
    Buffer: PVOID,
    BufferLength: ULONG,
    OutUrb: *mut PURB,
) -> NTSTATUS {
    let UrbSize: ULONG = (core::mem::size_of::<_URB_ISOCH_TRANSFER>() as usize)
        .wrapping_add(
            (core::mem::size_of::<USBD_ISO_PACKET_DESCRIPTOR>() as usize)
                .wrapping_mul(PACKET_COUNT as usize),
        ) as ULONG;

    let Urb = AllocFunction(UrbSize) as PURB;
    if Urb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    {
        let u = (*Urb).__bindgen_anon_1.UrbIsochronousTransfer.as_mut();
        u.Hdr.Function = URB_FUNCTION_ISOCH_TRANSFER as USHORT;
        u.Hdr.Length = UrbSize as USHORT;
        u.PipeHandle = PipeHandle;
        u.TransferFlags = (USBD_TRANSFER_DIRECTION_IN | USBD_START_ISO_TRANSFER_ASAP) as ULONG;
        u.TransferBufferLength = BufferLength;
        u.TransferBuffer = Buffer;
        u.NumberOfPackets = PACKET_COUNT as ULONG;
    }

    let mut Index: ULONG = 0;
    while Index < PACKET_COUNT as ULONG {
        let u = (*Urb).__bindgen_anon_1.UrbIsochronousTransfer.as_mut();
        (*(&mut u.IsoPacket[0] as *mut USBD_ISO_PACKET_DESCRIPTOR).offset(Index as isize))
            .Offset = Index.wrapping_mul(MaxPacketSize);
        Index = Index.wrapping_add(1);
    }

    *OutUrb = Urb;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn UsbAudioSetFormat(
    Pin: PKSPIN,
) -> NTSTATUS {
    let mut Urb: PURB = core::ptr::null_mut();
    let mut SampleRateBuffer: PUCHAR = core::ptr::null_mut();
    let mut PinContext: PPIN_CONTEXT = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut WaveFormatEx: PKSDATAFORMAT_WAVEFORMATEX = core::ptr::null_mut();

    SampleRateBuffer = AllocFunction(core::mem::size_of::<ULONG>() as ULONG) as PUCHAR;
    if SampleRateBuffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if IsEqualGUIDAligned(
        &(*(*Pin).ConnectionFormat).__bindgen_anon_1.as_ref().MajorFormat,
        &KSDATAFORMAT_TYPE_AUDIO,
    )
        && IsEqualGUIDAligned(
            &(*(*Pin).ConnectionFormat).__bindgen_anon_1.as_ref().SubFormat,
            &KSDATAFORMAT_SUBTYPE_PCM,
        )
        && IsEqualGUIDAligned(
            &(*(*Pin).ConnectionFormat).__bindgen_anon_1.as_ref().Specifier,
            &KSDATAFORMAT_SPECIFIER_WAVEFORMATEX,
        )
    {
        WaveFormatEx = (*Pin).ConnectionFormat as PKSDATAFORMAT_WAVEFORMATEX;
        *SampleRateBuffer.offset(2) = ((*WaveFormatEx).WaveFormatEx.nSamplesPerSec >> 16) as u8;
        *SampleRateBuffer.offset(1) = ((*WaveFormatEx).WaveFormatEx.nSamplesPerSec >> 8) as u8;
        *SampleRateBuffer.offset(0) = ((*WaveFormatEx).WaveFormatEx.nSamplesPerSec >> 0) as u8;
    } else {
        FreeFunction(SampleRateBuffer as PVOID);
        return STATUS_INVALID_PARAMETER;
    }

    Urb = AllocFunction(core::mem::size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as ULONG) as PURB;
    if Urb.is_null() {
        FreeFunction(SampleRateBuffer as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    PinContext = (*Pin).Context as PPIN_CONTEXT;

    (*Urb).__bindgen_anon_1.UrbHeader.as_mut().Function = 0x1c;
    (*Urb).__bindgen_anon_1.UrbHeader.as_mut().Length = core::mem::size_of::<_URB_CONTROL_VENDOR_OR_CLASS_REQUEST>() as USHORT;
    (*Urb).__bindgen_anon_1.UrbControlVendorClassRequest.as_mut().TransferBufferLength = 3;
    (*Urb).__bindgen_anon_1.UrbControlVendorClassRequest.as_mut().TransferBufferMDL = core::ptr::null_mut();
    (*Urb).__bindgen_anon_1.UrbControlVendorClassRequest.as_mut().TransferBuffer = SampleRateBuffer as PVOID;
    (*Urb).__bindgen_anon_1.UrbControlVendorClassRequest.as_mut().RequestTypeReservedBits = 0;
    (*Urb).__bindgen_anon_1.UrbControlVendorClassRequest.as_mut().Request = 0x01;
    (*Urb).__bindgen_anon_1.UrbControlVendorClassRequest.as_mut().Value = 0x100;
    (*Urb).__bindgen_anon_1.UrbControlVendorClassRequest.as_mut().Index = (*(*(*PinContext).DeviceExtension).InterfaceInfo).Pipes[0].EndpointAddress as USHORT;
    (*Urb).__bindgen_anon_1.UrbControlVendorClassRequest.as_mut().TransferFlags = 0;
    (*Urb).__bindgen_anon_1.UrbControlVendorClassRequest.as_mut().UrbLink = core::ptr::null_mut();

    Status = SubmitUrbSync((*PinContext).LowerDevice, Urb);

    FreeFunction(Urb as PVOID);
    FreeFunction(SampleRateBuffer as PVOID);
    Status
}
#[no_mangle]
pub unsafe extern "C" fn USBAudioSelectAudioStreamingInterface(
    Pin: PKSPIN,
    PinContext: PPIN_CONTEXT,
    DeviceExtension: PDEVICE_EXTENSION,
    ConfigurationDescriptor: PUSB_CONFIGURATION_DESCRIPTOR,
    FormatDescriptorIndex: ULONG,
) -> NTSTATUS {
    let mut Urb: PURB = core::ptr::null_mut();
    let mut InterfaceDescriptor: PUSB_INTERFACE_DESCRIPTOR = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut Found: ULONG = 0;
    let mut Index: ULONG = 0;
    let mut StreamingInterfaceDescriptor: PUSB_AUDIO_STREAMING_INTERFACE_DESCRIPTOR = core::ptr::null_mut();
    let TerminalDescriptor: PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR;

    TerminalDescriptor = UsbAudioGetStreamingTerminalDescriptorByIndex(
        (*DeviceExtension).ConfigurationDescriptor,
        (*Pin).Id,
    );
    if TerminalDescriptor.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(
        ConfigurationDescriptor,
        ConfigurationDescriptor as PVOID,
        -1i32 as LONG,
        -1i32 as LONG,
        USB_DEVICE_CLASS_AUDIO as LONG,
        -1i32 as LONG,
        -1i32 as LONG,
    );
    if InterfaceDescriptor.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Found = 0;
    Index = 0;

    while !InterfaceDescriptor.is_null() {
        if (*InterfaceDescriptor).bInterfaceSubClass as i32 == 0x02
            && (*InterfaceDescriptor).bNumEndpoints as i32 > 0
        {
            StreamingInterfaceDescriptor = USBD_ParseDescriptors(
                ConfigurationDescriptor as PVOID,
                (*ConfigurationDescriptor).wTotalLength as ULONG,
                InterfaceDescriptor as PVOID,
                USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE as LONG,
            ) as PUSB_AUDIO_STREAMING_INTERFACE_DESCRIPTOR;

            if !StreamingInterfaceDescriptor.is_null() {
                if (*StreamingInterfaceDescriptor).bTerminalLink as i32
                    == (*TerminalDescriptor).bTerminalID as i32
                {
                    if FormatDescriptorIndex == Index {
                        Found = 1;
                        break;
                    }
                    Index = Index.wrapping_add(1);
                }
            }
        }
        InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(
            ConfigurationDescriptor,
            (InterfaceDescriptor as ULONG_PTR)
                .wrapping_add((*InterfaceDescriptor).bLength as ULONG_PTR)
                as PVOID,
            -1i32 as LONG,
            -1i32 as LONG,
            USB_DEVICE_CLASS_AUDIO as LONG,
            -1i32 as LONG,
            -1i32 as LONG,
        );
    }

    if Found == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    Urb = AllocFunction(GET_SELECT_INTERFACE_REQUEST_SIZE((*InterfaceDescriptor).bNumEndpoints)) as PURB;
    if Urb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    UsbBuildSelectInterfaceRequest(
        Urb,
        GET_SELECT_INTERFACE_REQUEST_SIZE((*InterfaceDescriptor).bNumEndpoints) as USHORT,
        (*DeviceExtension).ConfigurationHandle,
        (*InterfaceDescriptor).bInterfaceNumber,
        (*InterfaceDescriptor).bAlternateSetting,
    );

    Status = SubmitUrbSync((*DeviceExtension).LowerDevice, Urb);

    if NT_SUCCESS(Status) {
        if !(*DeviceExtension).InterfaceInfo.is_null() {
            FreeFunction((*DeviceExtension).InterfaceInfo as PVOID);
        }

        (*DeviceExtension).InterfaceInfo = AllocFunction(
            (*Urb).__bindgen_anon_1.UrbSelectInterface.as_ref().Interface.Length as ULONG,
        ) as PUSBD_INTERFACE_INFORMATION;

        if (*DeviceExtension).InterfaceInfo.is_null() {
            FreeFunction(Urb as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::copy_nonoverlapping(
            &(*Urb).__bindgen_anon_1.UrbSelectInterface.as_ref().Interface as *const USBD_INTERFACE_INFORMATION as *const u8,
            (*DeviceExtension).InterfaceInfo as *mut u8,
            (*Urb).__bindgen_anon_1.UrbSelectInterface.as_ref().Interface.Length as usize,
        );
        (*PinContext).InterfaceDescriptor = InterfaceDescriptor;
    }

    FreeFunction(Urb as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn CaptureGateOnWorkItem(Context: PVOID) {
    let Pin: PKSPIN = Context as PKSPIN;
    let PinContext: PPIN_CONTEXT = (*Pin).Context as PPIN_CONTEXT;
    loop {
        KsPinAcquireProcessingMutex(Pin);
        let Gate: PKSGATE = KsPinGetAndGate(Pin);
        KsGateTurnInputOn(Gate);
        KsPinAttemptProcessing(Pin, 1u8);
        KsPinReleaseProcessingMutex(Pin);
        let Count: ULONG = KsDecrementCountedWorker((*PinContext).CaptureWorker);
        if Count == 0 {
            break;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn RenderInitializeUrbAndIrp(
    Pin: PKSPIN,
    PinContext: PPIN_CONTEXT,
    Irp: PIRP,
    TransferBuffer: PVOID,
    TransferBufferSize: ULONG,
    PacketSize: ULONG,
) -> NTSTATUS {
    let mut Index: ULONG;
    let PacketCount: ULONG;
    let Urb: PURB;
    let IoStack: PIO_STACK_LOCATION;

    IoInitializeIrp(
        Irp,
        IoSizeOfIrp((*(*(*PinContext).DeviceExtension).LowerDevice).StackSize),
        (*(*(*PinContext).DeviceExtension).LowerDevice).StackSize,
    );

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    (*Irp).IoStatus.Information = 0;
    (*Irp).Flags = 0;
    (*Irp).UserBuffer = core::ptr::null_mut();

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).DeviceObject = (*(*PinContext).DeviceExtension).LowerDevice;
    (*IoStack).Parameters.Others.Argument2 = core::ptr::null_mut();
    (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;

    IoSetCompletionRoutine(
        Irp,
        Some(completion_stdcall!(UsbAudioRenderComplete)),
        Pin as PVOID,
        1u8,
        1u8,
        1u8,
    );

    PacketCount = TransferBufferSize / PacketSize;

    Urb = AllocFunction(GET_ISO_URB_SIZE(PacketCount)) as PURB;
    if Urb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    {
        let u = (*Urb).__bindgen_anon_1.UrbIsochronousTransfer.as_mut();
        u.Hdr.Function = URB_FUNCTION_ISOCH_TRANSFER as USHORT;
        u.Hdr.Length = GET_ISO_URB_SIZE(PacketCount) as USHORT;
        u.PipeHandle = (*(*(*PinContext).DeviceExtension).InterfaceInfo).Pipes[0].PipeHandle;
        u.TransferFlags = (USBD_TRANSFER_DIRECTION_OUT | USBD_START_ISO_TRANSFER_ASAP) as ULONG;
        u.TransferBufferLength = TransferBufferSize;
        u.TransferBuffer = TransferBuffer;
        u.NumberOfPackets = PacketCount;
        u.StartFrame = 0;
    }

    Index = 0;
    while Index < PacketCount {
        let u = (*Urb).__bindgen_anon_1.UrbIsochronousTransfer.as_mut();
        (*(&mut u.IsoPacket[0] as *mut USBD_ISO_PACKET_DESCRIPTOR).offset(Index as isize))
            .Offset = Index * PacketSize;
        Index += 1;
    }

    (*IoStack).Parameters.Others.Argument1 = Urb as PVOID;
    (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] = Urb as PVOID;

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn CaptureInitializeUrbAndIrp(
    Pin: PKSPIN,
    Irp: PIRP,
) {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Urb: PURB;
    let mut TransferBuffer: PUCHAR;
    let mut Index: ULONG;
    let mut PinContext: PPIN_CONTEXT;

    PinContext = (*Pin).Context as PPIN_CONTEXT;

    Urb = (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] as PURB;
    TransferBuffer = (*Urb).__bindgen_anon_1.UrbIsochronousTransfer.as_ref().TransferBuffer as PUCHAR;

    IoInitializeIrp(
        Irp,
        (core::mem::size_of::<IRP>() as usize)
            .wrapping_add(
                ((*(*(*PinContext).DeviceExtension).LowerDevice).StackSize as usize)
                    .wrapping_mul(core::mem::size_of::<IO_STACK_LOCATION>() as usize),
            ) as USHORT,
        (*(*(*PinContext).DeviceExtension).LowerDevice).StackSize,
    );

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    (*Irp).IoStatus.Information = 0;
    (*Irp).Flags = 0;
    (*Irp).UserBuffer = core::ptr::null_mut();
    (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] = Urb as PVOID;
    (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1] = core::ptr::null_mut();

    IoStack = IoGetNextIrpStackLocation(Irp);
    (*IoStack).DeviceObject = (*(*PinContext).DeviceExtension).LowerDevice;
    (*IoStack).Parameters.Others.Argument1 = Urb as PVOID;
    (*IoStack).Parameters.Others.Argument2 = core::ptr::null_mut();
    (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
    (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;

    IoSetCompletionRoutine(
        Irp,
        Some(completion_stdcall!(UsbAudioCaptureComplete)),
        Pin as PVOID,
        1u8,
        1u8,
        1u8,
    );

    core::ptr::write_bytes(
        Urb as *mut core::ffi::c_void,
        0,
        (core::mem::size_of::<_URB_ISOCH_TRANSFER>() as usize)
            .wrapping_add(
                (core::mem::size_of::<USBD_ISO_PACKET_DESCRIPTOR>() as usize)
                    .wrapping_mul(PACKET_COUNT as usize),
            ),
    );

    {
        let u = (*Urb).__bindgen_anon_1.UrbIsochronousTransfer.as_mut();
        u.Hdr.Function = URB_FUNCTION_ISOCH_TRANSFER as USHORT;
        u.Hdr.Length = (core::mem::size_of::<_URB_ISOCH_TRANSFER>()
            .wrapping_add(core::mem::size_of::<USBD_ISO_PACKET_DESCRIPTOR>()
                .wrapping_mul(PACKET_COUNT as usize))) as USHORT;
        u.PipeHandle = (*(*(*PinContext).DeviceExtension).InterfaceInfo).Pipes[0].PipeHandle;
        u.TransferFlags = (USBD_TRANSFER_DIRECTION_IN | USBD_START_ISO_TRANSFER_ASAP) as ULONG;
        u.TransferBufferLength = (*(*(*PinContext).DeviceExtension).InterfaceInfo).Pipes[0]
            .MaximumPacketSize as ULONG * PACKET_COUNT as ULONG;
        u.TransferBuffer = TransferBuffer as PVOID;
        u.NumberOfPackets = PACKET_COUNT;
        u.StartFrame = 0;
    }

    Index = 0;
    while Index < PACKET_COUNT {
        let u = (*Urb).__bindgen_anon_1.UrbIsochronousTransfer.as_mut();
        (*(&mut u.IsoPacket[0] as *mut USBD_ISO_PACKET_DESCRIPTOR).offset(Index as isize))
            .Offset = Index.wrapping_mul(
                (*(*(*PinContext).DeviceExtension).InterfaceInfo).Pipes[0].MaximumPacketSize as ULONG,
            );
        Index = Index.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CaptureAvoidPipeStarvationWorker(Context: PVOID) {
    let Pin: PKSPIN = Context as PKSPIN;
    let PinContext: PPIN_CONTEXT = (*Pin).Context as PPIN_CONTEXT;
    let mut OldLevel: KIRQL = 0;
    let mut CurEntry: PLIST_ENTRY;
    let mut Irp: PIRP;

    KeAcquireSpinLock(&mut (*PinContext).IrpListLock, &mut OldLevel);

    if !IsListEmpty(&(*PinContext).IrpListHead) {
        CurEntry = RemoveHeadList(&mut (*PinContext).IrpListHead);
        KeReleaseSpinLock(&mut (*PinContext).IrpListLock, OldLevel);

        Irp = irp_from_list_entry(CurEntry);
        CaptureInitializeUrbAndIrp(Pin, Irp);
        KsDecrementCountedWorker((*PinContext).StarvationWorker);
        IoCallDriver((*(*PinContext).DeviceExtension).LowerDevice, Irp);
    } else {
        KeReleaseSpinLock(&mut (*PinContext).IrpListLock, OldLevel);
        KsDecrementCountedWorker((*PinContext).StarvationWorker);
    }
}
#[no_mangle]
pub unsafe extern "C" fn InitCapturePin(
    Pin: PKSPIN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Index: ULONG;
    let mut BufferSize: ULONG;
    let mut MaximumPacketSize: ULONG;
    let mut Irp: PIRP;
    let mut Urb: PURB = core::ptr::null_mut();
    let mut PinContext: PPIN_CONTEXT;
    let mut IoStack: PIO_STACK_LOCATION;
    let mut Framing: PKSALLOCATOR_FRAMING_EX;
    let mut Gate: PKSGATE;

    Status = UsbAudioSetFormat(Pin);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    PinContext = (*Pin).Context as PPIN_CONTEXT;

    MaximumPacketSize = GetMaxPacketSizeForInterface(
        (*(*PinContext).DeviceExtension).ConfigurationDescriptor,
        (*PinContext).InterfaceDescriptor,
        (*Pin).DataFlow,
    ) as ULONG;

    ExInitializeWorkItem(
        &mut (*PinContext).CaptureWorkItem,
        Some(worker_stdcall!(CaptureGateOnWorkItem)),
        Pin as PVOID,
    );

    Status = KsRegisterCountedWorker(
        CriticalWorkQueue,
        &mut (*PinContext).CaptureWorkItem,
        &mut (*PinContext).CaptureWorker,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    ExInitializeWorkItem(
        &mut (*PinContext).StarvationWorkItem,
        Some(worker_stdcall!(CaptureAvoidPipeStarvationWorker)),
        Pin as PVOID,
    );

    Status = KsRegisterCountedWorker(
        CriticalWorkQueue,
        &mut (*PinContext).StarvationWorkItem,
        &mut (*PinContext).StarvationWorker,
    );
    if !NT_SUCCESS(Status) {
        KsUnregisterWorker((*PinContext).CaptureWorker);
    }

    Framing = (*(*Pin).Descriptor).AllocatorFraming as PKSALLOCATOR_FRAMING_EX;
    (*Framing).FramingItem[0].PhysicalRange.MinFrameSize = MaximumPacketSize;
    (*Framing).FramingItem[0].PhysicalRange.MaxFrameSize = MaximumPacketSize;
    (*Framing).FramingItem[0].FramingRange.Range.MinFrameSize = MaximumPacketSize;
    (*Framing).FramingItem[0].FramingRange.Range.MaxFrameSize = MaximumPacketSize;

    BufferSize = (8 * PACKET_COUNT) as ULONG * MaximumPacketSize;

    (*PinContext).BufferSize = BufferSize;
    (*PinContext).Buffer = AllocFunction(BufferSize) as PUCHAR;
    if (*PinContext).Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    KsAddItemToObjectBag(
        (*Pin).Bag,
        (*PinContext).Buffer as PVOID,
        Some(core::mem::transmute(ExFreePool as usize)),
    );

    Index = 0;
    while Index < 8 {
        Irp = AllocFunction(IoSizeOfIrp((*(*(*PinContext).DeviceExtension).LowerDevice).StackSize) as ULONG) as PIRP;
        if Irp.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        IoInitializeIrp(
            Irp,
            IoSizeOfIrp((*(*(*PinContext).DeviceExtension).LowerDevice).StackSize),
            (*(*(*PinContext).DeviceExtension).LowerDevice).StackSize,
        );

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
        (*Irp).IoStatus.Information = 0;
        (*Irp).Flags = 0;
        (*Irp).UserBuffer = core::ptr::null_mut();

        IoStack = IoGetNextIrpStackLocation(Irp);
        (*IoStack).DeviceObject = (*(*PinContext).DeviceExtension).LowerDevice;
        (*IoStack).Parameters.Others.Argument2 = core::ptr::null_mut();
        (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
        (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;

        IoSetCompletionRoutine(
            Irp,
            Some(completion_stdcall!(UsbAudioCaptureComplete)),
            Pin as PVOID,
            1u8,
            1u8,
            1u8,
        );

        InsertTailList(
            &mut (*PinContext).IrpListHead,
            &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry,
        );

        KsAddItemToObjectBag(
            (*Pin).Bag,
            Irp as PVOID,
            Some(core::mem::transmute(ExFreePool as usize)),
        );

        Status = UsbAudioAllocCaptureUrbIso(
            (*(*(*PinContext).DeviceExtension).InterfaceInfo).Pipes[0].PipeHandle,
            MaximumPacketSize,
            (*PinContext).Buffer.add((MaximumPacketSize * PACKET_COUNT as ULONG * Index) as usize) as PVOID,
            MaximumPacketSize * PACKET_COUNT as ULONG,
            &mut Urb,
        );

        if NT_SUCCESS(Status) {
            IoStack = IoGetNextIrpStackLocation(Irp);
            (*IoStack).Parameters.Others.Argument1 = Urb as PVOID;
            (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] = Urb as PVOID;
        } else {
            return Status;
        }

        Index += 1;
    }

    Gate = KsPinGetAndGate(Pin);
    KsGateTurnInputOff(Gate);

    Status
}
#[no_mangle]
pub unsafe extern "C" fn InitStreamPin(
    Pin: PKSPIN,
) -> NTSTATUS {
    let mut Index: ULONG;
    let mut Irp: PIRP;
    let mut PinContext: PPIN_CONTEXT;
    let mut WaveFormatEx: PKSDATAFORMAT_WAVEFORMATEX;
    let mut IoStack: PIO_STACK_LOCATION;

    PinContext = (*Pin).Context as PPIN_CONTEXT;
    WaveFormatEx = (*Pin).ConnectionFormat as PKSDATAFORMAT_WAVEFORMATEX;

    (*PinContext).Buffer = AllocFunction((*WaveFormatEx).WaveFormatEx.nAvgBytesPerSec) as PUCHAR;
    if (*PinContext).Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*PinContext).BufferSize = (*WaveFormatEx).WaveFormatEx.nAvgBytesPerSec;
    (*PinContext).BufferOffset = 0;
    (*PinContext).BufferLength = 0;

    Index = 0;
    while Index < 12 {
        Irp = AllocFunction(IoSizeOfIrp((*(*(*PinContext).DeviceExtension).LowerDevice).StackSize) as ULONG) as PIRP;
        if Irp.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        IoInitializeIrp(
            Irp,
            IoSizeOfIrp((*(*(*PinContext).DeviceExtension).LowerDevice).StackSize),
            (*(*(*PinContext).DeviceExtension).LowerDevice).StackSize,
        );

        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
        (*Irp).IoStatus.Information = 0;
        (*Irp).Flags = 0;
        (*Irp).UserBuffer = core::ptr::null_mut();

        IoStack = IoGetNextIrpStackLocation(Irp);
        (*IoStack).DeviceObject = (*(*PinContext).DeviceExtension).LowerDevice;
        (*IoStack).Parameters.Others.Argument2 = core::ptr::null_mut();
        (*IoStack).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
        (*IoStack).Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;

        IoSetCompletionRoutine(
            Irp,
            Some(completion_stdcall!(UsbAudioRenderComplete)),
            Pin as PVOID,
            1u8,
            1u8,
            1u8,
        );

        InsertTailList(
            &mut (*PinContext).IrpListHead,
            &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry,
        );

        KsAddItemToObjectBag(
            (*Pin).Bag,
            Irp as PVOID,
            Some(core::mem::transmute(ExFreePool as usize)),
        );

        Index += 1;
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn GetDataRangeIndexForFormat(
    ConnectionFormat: PKSDATARANGE,
    DataRanges: *const PKSDATARANGE,
    DataRangesCount: ULONG,
) -> ULONG {
    let mut Index: ULONG;
    let mut CurrentDataRange: PKSDATARANGE;
    let mut CurrentAudioDataRange: PKSDATARANGE_AUDIO;
    let mut ConnectionDataFormat: PKSDATAFORMAT_WAVEFORMATEX;

    if (*ConnectionFormat).__bindgen_anon_1.as_ref().FormatSize != core::mem::size_of::<KSDATAFORMAT>() as ULONG + core::mem::size_of::<WAVEFORMATEX>() as ULONG {
        return MAXULONG;
    }

    ConnectionDataFormat = ConnectionFormat as PKSDATAFORMAT_WAVEFORMATEX;

    Index = 0;
    while Index < DataRangesCount {
        CurrentDataRange = *DataRanges.offset(Index as isize);

        if !IsEqualGUIDAligned(
                &(*CurrentDataRange).__bindgen_anon_1.as_ref().MajorFormat,
                &(*ConnectionFormat).__bindgen_anon_1.as_ref().MajorFormat)
           || !IsEqualGUIDAligned(
                &(*CurrentDataRange).__bindgen_anon_1.as_ref().SubFormat,
                &(*ConnectionFormat).__bindgen_anon_1.as_ref().SubFormat)
           || !IsEqualGUIDAligned(
                &(*CurrentDataRange).__bindgen_anon_1.as_ref().Specifier,
                &(*ConnectionFormat).__bindgen_anon_1.as_ref().Specifier)
        {
            Index = Index.wrapping_add(1);
            continue;
        }

        CurrentAudioDataRange = CurrentDataRange as PKSDATARANGE_AUDIO;

        if (*CurrentAudioDataRange).MaximumChannels != (*ConnectionDataFormat).WaveFormatEx.nChannels as ULONG {
            Index = Index.wrapping_add(1);
            continue;
        }

        if (*CurrentAudioDataRange).MinimumSampleFrequency > (*ConnectionDataFormat).WaveFormatEx.nSamplesPerSec {
            Index = Index.wrapping_add(1);
            continue;
        }

        if (*CurrentAudioDataRange).MaximumSampleFrequency < (*ConnectionDataFormat).WaveFormatEx.nSamplesPerSec {
            Index = Index.wrapping_add(1);
            continue;
        }

        return Index;
    }

    MAXULONG
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPinCreate(
    Pin: PKSPIN,
    Irp: PIRP,
) -> NTSTATUS {
    let Filter: PKSFILTER;
    let FilterContext: PFILTER_CONTEXT;
    let PinContext: PPIN_CONTEXT;
    let mut Status: NTSTATUS;
    let mut FormatIndex: ULONG;

    Filter = KsPinGetParentFilter(Pin);
    if Filter.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    FilterContext = (*Filter).Context as PFILTER_CONTEXT;
    PinContext = AllocFunction(core::mem::size_of::<PIN_CONTEXT>() as ULONG) as PPIN_CONTEXT;
    if PinContext.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*PinContext).DeviceExtension = (*FilterContext).DeviceExtension;
    (*PinContext).LowerDevice = (*FilterContext).LowerDevice;
    InitializeListHead(&mut (*PinContext).IrpListHead);
    InitializeListHead(&mut (*PinContext).DoneIrpListHead);
    KeInitializeSpinLock(&mut (*PinContext).IrpListLock);

    (*Pin).Context = PinContext as PVOID;

    Status = _KsEdit(
        (*Pin).Bag,
        &mut (*Pin).Descriptor as *mut _ as *mut PVOID,
        core::mem::size_of::<KSPIN_DESCRIPTOR_EX>() as ULONG,
        core::mem::size_of::<KSPIN_DESCRIPTOR_EX>() as ULONG,
        USBAUDIO_TAG as ULONG,
    );
    if NT_SUCCESS(Status) {
        Status = _KsEdit(
            (*Pin).Bag,
            &mut (*((*Pin).Descriptor as *mut KSPIN_DESCRIPTOR_EX)).AllocatorFraming as *mut _ as *mut PVOID,
            core::mem::size_of::<KSALLOCATOR_FRAMING_EX>() as ULONG,
            core::mem::size_of::<KSALLOCATOR_FRAMING_EX>() as ULONG,
            USBAUDIO_TAG as ULONG,
        );
        if Status != STATUS_SUCCESS {
            RtlAssert(
                b"Status == STATUS_SUCCESS\0".as_ptr() as *mut u8 as PVOID,
                b"drivers/usb/usbaudio/pin.c\0".as_ptr() as *mut u8 as PVOID,
                807u32,
                core::ptr::null_mut(),
            );
        }
    }

    FormatIndex = GetDataRangeIndexForFormat(
        (*Pin).ConnectionFormat as PKSDATARANGE,
        (*(*Pin).Descriptor).PinDescriptor.DataRanges,
        (*(*Pin).Descriptor).PinDescriptor.DataRangesCount,
    );
    if FormatIndex == MAXULONG {
        return STATUS_NO_MATCH;
    }

    Status = USBAudioSelectAudioStreamingInterface(
        Pin,
        PinContext,
        (*PinContext).DeviceExtension,
        (*(*PinContext).DeviceExtension).ConfigurationDescriptor,
        FormatIndex,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
        return Status;
    }

    if (*Pin).DataFlow == KSPIN_DATAFLOW_OUT {
        Status = InitCapturePin(Pin);
    } else {
        Status = InitStreamPin(Pin);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPinClose(
    Pin: PKSPIN,
    Irp: PIRP,
) -> NTSTATUS {
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn UsbAudioRenderComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let Pin: PKSPIN = Context as PKSPIN;
    let PinContext: PPIN_CONTEXT = (*Pin).Context as PPIN_CONTEXT;
    let Status: NTSTATUS = (*Irp).IoStatus.__bindgen_anon_1.Status;
    let StreamPointerClone: PKSSTREAM_POINTER = (*Irp)
        .Tail
        .Overlay
        .__bindgen_anon_1
        .__bindgen_anon_1
        .DriverContext[1]
        as PKSSTREAM_POINTER;
    let Urb: PURB = (*Irp)
        .Tail
        .Overlay
        .__bindgen_anon_1
        .__bindgen_anon_1
        .DriverContext[0]
        as PURB;

    FreeFunction(Urb as PVOID);

    let mut OldLevel: KIRQL = 0;
    KeAcquireSpinLock(&mut (*PinContext).IrpListLock, &mut OldLevel);
    InsertTailList(
        &mut (*PinContext).IrpListHead,
        &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry,
    );
    KeReleaseSpinLock(&mut (*PinContext).IrpListLock, OldLevel);

    if !NT_SUCCESS(Status) && !StreamPointerClone.is_null() {
        KsStreamPointerSetStatusCode(StreamPointerClone, STATUS_DEVICE_DATA_ERROR);
    }

    if !StreamPointerClone.is_null() {
        KsStreamPointerDelete(StreamPointerClone);
    }

    STATUS_MORE_PROCESSING_REQUIRED
}

#[no_mangle]
pub unsafe extern "system" fn UsbAudioCaptureComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let Pin: PKSPIN;
    let PinContext: PPIN_CONTEXT;
    let Urb: PURB;

    Pin = Context as PKSPIN;
    PinContext = (*Pin).Context as PPIN_CONTEXT;
    Urb = (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] as PURB;

    let mut OldLevel: KIRQL = 0;
    KeAcquireSpinLock(&mut (*PinContext).IrpListLock, &mut OldLevel);

    if !((*Urb).__bindgen_anon_1.UrbIsochronousTransfer.as_ref().Hdr.Status >= 0) {
        InsertTailList(
            &mut (*PinContext).IrpListHead,
            &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry,
        );
        KeReleaseSpinLock(&mut (*PinContext).IrpListLock, OldLevel);
        KsIncrementCountedWorker((*PinContext).StarvationWorker);
    } else {
        InsertTailList(
            &mut (*PinContext).DoneIrpListHead,
            &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry,
        );
        KeReleaseSpinLock(&mut (*PinContext).IrpListLock, OldLevel);
        KsIncrementCountedWorker((*PinContext).CaptureWorker);
    }

    STATUS_MORE_PROCESSING_REQUIRED
}
#[no_mangle]
pub unsafe extern "C" fn PinGetIrpFromReadyList(
    Pin: PKSPIN,
) -> PIRP {
    let PinContext: PPIN_CONTEXT;
    let CurEntry: PLIST_ENTRY;
    let mut OldLevel: KIRQL = 0;
    let mut Irp: PIRP = core::ptr::null_mut();

    PinContext = (*Pin).Context as PPIN_CONTEXT;

    KeAcquireSpinLock(&mut (*PinContext).IrpListLock, &mut OldLevel);

    if !IsListEmpty(&mut (*PinContext).IrpListHead) {
        CurEntry = RemoveHeadList(&mut (*PinContext).IrpListHead);
        Irp = irp_from_list_entry(CurEntry);
    }

    KeReleaseSpinLock(&mut (*PinContext).IrpListLock, OldLevel);

    Irp
}
#[no_mangle]
pub unsafe extern "C" fn PinRenderProcess(
    Pin: PKSPIN,
) -> NTSTATUS {
    let LeadingStreamPointer: PKSSTREAM_POINTER;
    let mut CloneStreamPointer: PKSSTREAM_POINTER = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let PinContext: PPIN_CONTEXT;
    let mut PacketCount: ULONG;
    let mut TotalPacketSize: ULONG;
    let mut Offset: ULONG;
    let WaveFormatEx: PKSDATAFORMAT_WAVEFORMATEX;
    let TransferBuffer: PUCHAR;
    let mut Irp: PIRP = core::ptr::null_mut();

    LeadingStreamPointer = KsPinGetLeadingEdgeStreamPointer(Pin, KSSTREAM_POINTER_STATE_LOCKED);
    if LeadingStreamPointer.is_null() {
        return STATUS_SUCCESS;
    }

    if (*(*LeadingStreamPointer).StreamHeader).Data.is_null() {
        Status = KsStreamPointerAdvance(LeadingStreamPointer);
    }

    PinContext = (*Pin).Context as PPIN_CONTEXT;

    Irp = PinGetIrpFromReadyList(Pin);
    if Irp.is_null() {
        KsStreamPointerUnlock(LeadingStreamPointer, 1u8);
        return STATUS_SUCCESS;
    }

    Status = KsStreamPointerClone(
        LeadingStreamPointer,
        None,
        0,
        &mut CloneStreamPointer,
    );
    if !NT_SUCCESS(Status) {
        KsStreamPointerUnlock(LeadingStreamPointer, 1u8);
        return STATUS_SUCCESS;
    }

    WaveFormatEx = (*Pin).ConnectionFormat as PKSDATAFORMAT_WAVEFORMATEX;
    TotalPacketSize = (*WaveFormatEx).WaveFormatEx.nAvgBytesPerSec / 1000;

    TransferBuffer = (*(*CloneStreamPointer).StreamHeader).Data as PUCHAR;

    Offset = 0;

    if (*PinContext).BufferLength != 0 {
        Offset = TotalPacketSize - (*PinContext).BufferLength;

        if (*PinContext).BufferOffset + TotalPacketSize >= (*PinContext).BufferSize {
            core::ptr::copy_nonoverlapping(
                (*PinContext).Buffer.offset(
                    ((*PinContext).BufferOffset - (*PinContext).BufferLength) as isize,
                ),
                (*PinContext).Buffer,
                (*PinContext).BufferLength as usize,
            );
            (*PinContext).BufferOffset = (*PinContext).BufferLength;
        }

        core::ptr::copy_nonoverlapping(
            TransferBuffer,
            (*PinContext).Buffer.offset((*PinContext).BufferOffset as isize),
            Offset as usize,
        );

        Status = RenderInitializeUrbAndIrp(
            Pin,
            PinContext,
            Irp,
            (*PinContext).Buffer.offset(
                ((*PinContext).BufferOffset - (*PinContext).BufferLength) as isize,
            ) as PVOID,
            TotalPacketSize,
            TotalPacketSize,
        );
        if NT_SUCCESS(Status) {
            Status = IoCallDriver((*PinContext).LowerDevice, Irp);
        }

        (*PinContext).BufferLength = 0;
        (*PinContext).BufferOffset += Offset;

        Irp = PinGetIrpFromReadyList(Pin);
    }

    PacketCount = ((*CloneStreamPointer).OffsetIn.Remaining - Offset) / TotalPacketSize;

    Status = RenderInitializeUrbAndIrp(
        Pin,
        PinContext,
        Irp,
        TransferBuffer.offset(Offset as isize) as PVOID,
        PacketCount * TotalPacketSize,
        TotalPacketSize,
    );
    if NT_SUCCESS(Status) {
        (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1] = CloneStreamPointer as PVOID;

        if (PacketCount * TotalPacketSize) + Offset < (*CloneStreamPointer).OffsetIn.Remaining {
            (*PinContext).BufferLength = (*CloneStreamPointer).OffsetIn.Remaining
                - ((PacketCount * TotalPacketSize) + Offset);

            if (*PinContext).BufferOffset + TotalPacketSize >= (*PinContext).BufferSize {
                (*PinContext).BufferOffset = 0;
            }

            core::ptr::copy_nonoverlapping(
                TransferBuffer.offset(((PacketCount * TotalPacketSize) + Offset) as isize),
                (*PinContext).Buffer.offset((*PinContext).BufferOffset as isize),
                (*PinContext).BufferLength as usize,
            );
            (*PinContext).BufferOffset += (*PinContext).BufferLength;
        }

        Status = IoCallDriver((*PinContext).LowerDevice, Irp);
    }

    KsStreamPointerUnlock(LeadingStreamPointer, 1u8);
    STATUS_PENDING
}
#[no_mangle]
pub unsafe extern "C" fn PinCaptureProcess(
    Pin: PKSPIN,
) -> NTSTATUS {
    let mut LeadingStreamPointer: PKSSTREAM_POINTER;
    let mut OldLevel: KIRQL = 0;
    let mut PinContext: PPIN_CONTEXT;
    let mut CurEntry: PLIST_ENTRY;
    let mut Irp: PIRP;
    let mut Urb: PURB;
    let mut TransferBuffer: PUCHAR;
    let mut OutBuffer: PUCHAR;
    let mut Offset: ULONG;
    let mut Length: ULONG;
    let mut Status: NTSTATUS;
    let mut Gate: PKSGATE;

    LeadingStreamPointer = KsPinGetLeadingEdgeStreamPointer(Pin, KSSTREAM_POINTER_STATE_LOCKED);
    if LeadingStreamPointer.is_null() {
        Gate = KsPinGetAndGate(Pin);
        KsGateTurnInputOff(Gate);
        return STATUS_SUCCESS;
    }

    PinContext = (*Pin).Context as PPIN_CONTEXT;

    KeAcquireSpinLock(&mut (*PinContext).IrpListLock, &mut OldLevel);

    while !IsListEmpty(&mut (*PinContext).DoneIrpListHead) {
        CurEntry = RemoveHeadList(&mut (*PinContext).DoneIrpListHead);

        KeReleaseSpinLock(&mut (*PinContext).IrpListLock, OldLevel);

        Irp = irp_from_list_entry(CurEntry);

        Urb = (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[0] as PURB;
        ASSERT!(!Urb.is_null());

        Offset = PtrToUlong((*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1] as *mut c_void);

        TransferBuffer = (*Urb).__bindgen_anon_1.UrbIsochronousTransfer.as_ref().TransferBuffer as PUCHAR;

        OutBuffer = (*(*LeadingStreamPointer).StreamHeader).Data as PUCHAR;

        Length = core::cmp::min(
            (*LeadingStreamPointer).OffsetOut.Count - (*(*LeadingStreamPointer).StreamHeader).DataUsed,
            (*Urb).__bindgen_anon_1.UrbIsochronousTransfer.as_ref().TransferBufferLength - Offset,
        );

        RtlCopyMemory(
            OutBuffer.offset((*(*LeadingStreamPointer).StreamHeader).DataUsed as isize) as *mut c_void,
            TransferBuffer.offset(Offset as isize) as *const c_void,
            Length as usize,
        );

        (*(*LeadingStreamPointer).StreamHeader).DataUsed += Length;

        if Length == (*LeadingStreamPointer).OffsetOut.Remaining {
            KsStreamPointerAdvanceOffsetsAndUnlock(LeadingStreamPointer, 0, Length, 1u8);

            KeAcquireSpinLock(&mut (*PinContext).IrpListLock, &mut OldLevel);

            (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext[1] = UlongToPtr(Length) as *mut c_void;

            InsertHeadList(
                &mut (*PinContext).DoneIrpListHead,
                &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry,
            );

            KeReleaseSpinLock(&mut (*PinContext).IrpListLock, OldLevel);

            LeadingStreamPointer = KsPinGetLeadingEdgeStreamPointer(Pin, KSSTREAM_POINTER_STATE_LOCKED);
            if LeadingStreamPointer.is_null() {
                return STATUS_PENDING;
            } else {
                continue;
            }
        } else {
            Status = KsStreamPointerAdvanceOffsets(LeadingStreamPointer, 0, Length, 0u8);
            NT_ASSERT!(NT_SUCCESS(Status));
            ASSERT!(Length == (*Urb).__bindgen_anon_1.UrbIsochronousTransfer.as_ref().TransferBufferLength - Offset);
        }

        KeAcquireSpinLock(&mut (*PinContext).IrpListLock, &mut OldLevel);

        InsertTailList(
            &mut (*PinContext).IrpListHead,
            &mut (*Irp).Tail.Overlay.__bindgen_anon_2.ListEntry,
        );
    }

    while !IsListEmpty(&mut (*PinContext).IrpListHead) {
        CurEntry = RemoveHeadList(&mut (*PinContext).IrpListHead);

        KeReleaseSpinLock(&mut (*PinContext).IrpListLock, OldLevel);

        Irp = irp_from_list_entry(CurEntry);

        CaptureInitializeUrbAndIrp(Pin, Irp);

        IoCallDriver((*(*PinContext).DeviceExtension).LowerDevice, Irp);

        KeAcquireSpinLock(&mut (*PinContext).IrpListLock, &mut OldLevel);
    }

    KeReleaseSpinLock(&mut (*PinContext).IrpListLock, OldLevel);

    if !LeadingStreamPointer.is_null() {
        KsStreamPointerUnlock(LeadingStreamPointer, 0u8);
    }

    Gate = KsPinGetAndGate(Pin);
    KsGateTurnInputOff(Gate);

    return STATUS_PENDING;
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPinProcess(
    Pin: PKSPIN,
) -> NTSTATUS {
    let Status: NTSTATUS;

    if (*Pin).DataFlow == KSPIN_DATAFLOW_OUT {
        Status = PinCaptureProcess(Pin);
    } else {
        Status = PinRenderProcess(Pin);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPinReset(
    Pin: PKSPIN,
) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"USBAudioPinReset\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/usb/usbaudio/pin.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        1293i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPinSetDataFormat(
    Pin: PKSPIN,
    OldFormat: PKSDATAFORMAT,
    _OldAttributeList: PKSMULTIPLE_ITEM,
    _DataRange: *const KSDATARANGE,
    _AttributeRange: *const KSATTRIBUTE_LIST,
) -> NTSTATUS {
    if OldFormat.is_null() {
        return STATUS_SUCCESS;
    }

    UsbAudioSetFormat(Pin)
}
#[no_mangle]
pub unsafe extern "C" fn StartCaptureIsocTransfer(
    Pin: PKSPIN,
) -> NTSTATUS {
    let PinContext: PPIN_CONTEXT;
    let mut CurEntry: PLIST_ENTRY;
    let mut Irp: PIRP;

    PinContext = (*Pin).Context as PPIN_CONTEXT;

    let mut OldLevel: KIRQL = 0;
    KeAcquireSpinLock(&mut (*PinContext).IrpListLock, &mut OldLevel);

    while !IsListEmpty(&(*PinContext).IrpListHead) {
        CurEntry = RemoveHeadList(&mut (*PinContext).IrpListHead);

        Irp = irp_from_list_entry(CurEntry);

        KeReleaseSpinLock(&mut (*PinContext).IrpListLock, OldLevel);

        CaptureInitializeUrbAndIrp(Pin, Irp);

        IoCallDriver((*(*PinContext).DeviceExtension).LowerDevice, Irp);

        KeAcquireSpinLock(&mut (*PinContext).IrpListLock, &mut OldLevel);
    }

    KeReleaseSpinLock(&mut (*PinContext).IrpListLock, OldLevel);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn CapturePinStateChange(
    Pin: PKSPIN,
    ToState: KSSTATE,
    FromState: KSSTATE,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    if FromState != ToState {
        if ToState != 0 {
            if ToState == KSSTATE_PAUSE {
                if FromState == KSSTATE_RUN {
                    /* wait until pin processing is finished*/
                }
            } else if ToState == KSSTATE_RUN {
                Status = StartCaptureIsocTransfer(Pin);
            }
        }
    }
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBAudioPinSetDeviceState(
    Pin: PKSPIN,
    ToState: KSSTATE,
    FromState: KSSTATE,
) -> NTSTATUS {
    let Status: NTSTATUS;
    if (*Pin).DataFlow as u32 == KSPIN_DATAFLOW_OUT as u32 {
        Status = CapturePinStateChange(Pin, ToState, FromState);
    } else {
        Status = STATUS_SUCCESS;
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn UsbAudioPinDataIntersect(
    Context: PVOID,
    Irp: PIRP,
    Pin: PKSP_PIN,
    DataRange: PKSDATARANGE,
    MatchingDataRange: PKSDATARANGE,
    DataBufferSize: ULONG,
    Data: PVOID,
    DataSize: PULONG,
) -> NTSTATUS {
    let Filter = KsGetFilterFromIrp(Irp);
    if Filter.is_null() {
        return STATUS_NO_MATCH;
    }

    let PinDescriptor = (*(*Filter).Descriptor).PinDescriptors.add((*Pin).PinId as usize) as PKSPIN_DESCRIPTOR_EX;

    *DataSize = core::mem::size_of::<KSDATAFORMAT_WAVEFORMATEX>() as ULONG;
    if DataBufferSize == 0 {
        return STATUS_BUFFER_OVERFLOW;
    }

    let DataRangeAudio = *(*PinDescriptor).PinDescriptor.DataRanges.offset(0) as PKSDATARANGE_AUDIO;

    let DataFormat = Data as PKSDATAFORMAT_WAVEFORMATEX;
    (*DataFormat).WaveFormatEx.wFormatTag = WAVE_FORMAT_PCM as WORD;
    (*DataFormat).WaveFormatEx.nChannels = (*DataRangeAudio).MaximumChannels as WORD;
    (*DataFormat).WaveFormatEx.nSamplesPerSec = (*DataRangeAudio).MaximumSampleFrequency as DWORD;
    (*DataFormat).WaveFormatEx.nAvgBytesPerSec = ((*DataRangeAudio).MaximumSampleFrequency * ((*DataRangeAudio).MaximumBitsPerSample / 8) * (*DataRangeAudio).MaximumChannels) as DWORD;
    (*DataFormat).WaveFormatEx.nBlockAlign = (((*DataRangeAudio).MaximumBitsPerSample / 8) * (*DataRangeAudio).MaximumChannels) as WORD;
    (*DataFormat).WaveFormatEx.wBitsPerSample = (*DataRangeAudio).MaximumBitsPerSample as WORD;
    (*DataFormat).WaveFormatEx.cbSize = 0 as WORD;

    {
        let df = (*DataFormat).DataFormat.__bindgen_anon_1.as_mut();
        df.FormatSize = (core::mem::size_of::<KSDATAFORMAT>() + core::mem::size_of::<WAVEFORMATEX>()) as ULONG;
        df.Flags = 0;
        df.Reserved = 0;
        df.MajorFormat = KSDATAFORMAT_TYPE_AUDIO;
        df.SubFormat = KSDATAFORMAT_SUBTYPE_PCM;
        df.Specifier = KSDATAFORMAT_SPECIFIER_WAVEFORMATEX;
        df.SampleSize = ((*DataRangeAudio).MaximumBitsPerSample / 8) * (*DataRangeAudio).MaximumChannels;
    }

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:     ReactOS Universal Audio Class Driver
// * LICENSE:     GPL - See COPYING in the top level directory
// * FILE:        drivers/usb/usbaudio/pin.c
// * PURPOSE:     USB Audio device driver.
// * PROGRAMMERS:
// *              Johannes Anderwald (johannes.anderwald@reactos.org)
// */
//
// #include "usbaudio.h"
//
// #define PACKET_COUNT 10
//
//
// NTSTATUS
// GetMaxPacketSizeForInterface(
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     IN PUSB_INTERFACE_DESCRIPTOR InterfaceDescriptor,
//     KSPIN_DATAFLOW DataFlow)
// {
//     PUSB_COMMON_DESCRIPTOR CommonDescriptor;
//     PUSB_ENDPOINT_DESCRIPTOR EndpointDescriptor;
//
//     /* loop descriptors */
//     CommonDescriptor = (PUSB_COMMON_DESCRIPTOR)((ULONG_PTR)InterfaceDescriptor + InterfaceDescriptor->bLength);
//     ASSERT(InterfaceDescriptor->bNumEndpoints > 0);
//     while (CommonDescriptor)
//     {
//         if (CommonDescriptor->bDescriptorType == USB_ENDPOINT_DESCRIPTOR_TYPE)
//         {
//             EndpointDescriptor = (PUSB_ENDPOINT_DESCRIPTOR)CommonDescriptor;
//             return EndpointDescriptor->wMaxPacketSize;
//         }
//
//         if (CommonDescriptor->bDescriptorType == USB_INTERFACE_DESCRIPTOR_TYPE)
//         {
//             /* reached next interface descriptor */
//             break;
//         }
//
//         if ((ULONG_PTR)CommonDescriptor + CommonDescriptor->bLength >= ((ULONG_PTR)ConfigurationDescriptor + ConfigurationDescriptor->wTotalLength))
//             break;
//
//         CommonDescriptor = (PUSB_COMMON_DESCRIPTOR)((ULONG_PTR)CommonDescriptor + CommonDescriptor->bLength);
//     }
//
//     /* default to 100 */
//     return 100;
// }
//
// NTSTATUS
// UsbAudioAllocCaptureUrbIso(
//     IN USBD_PIPE_HANDLE PipeHandle,
//     IN ULONG MaxPacketSize,
//     IN PVOID Buffer,
//     IN ULONG BufferLength,
//     OUT PURB * OutUrb)
// {
//     PURB Urb;
//     ULONG UrbSize;
//     ULONG Index;
//
//     /* calculate urb size*/
//     UrbSize = GET_ISO_URB_SIZE(PACKET_COUNT);
//
//     /* allocate urb */
//     Urb = AllocFunction(UrbSize);
//     if (!Urb)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* init urb */
//     Urb->UrbIsochronousTransfer.Hdr.Function = URB_FUNCTION_ISOCH_TRANSFER;
//     Urb->UrbIsochronousTransfer.Hdr.Length = UrbSize;
//     Urb->UrbIsochronousTransfer.PipeHandle = PipeHandle;
//     Urb->UrbIsochronousTransfer.TransferFlags = USBD_TRANSFER_DIRECTION_IN | USBD_START_ISO_TRANSFER_ASAP;
//     Urb->UrbIsochronousTransfer.TransferBufferLength = BufferLength;
//     Urb->UrbIsochronousTransfer.TransferBuffer = Buffer;
//     Urb->UrbIsochronousTransfer.NumberOfPackets = PACKET_COUNT;
//
//     for (Index = 0; Index < PACKET_COUNT; Index++)
//     {
//         Urb->UrbIsochronousTransfer.IsoPacket[Index].Offset = Index * MaxPacketSize;
//     }
//
//     *OutUrb = Urb;
//     return STATUS_SUCCESS;
//
// }
//
// NTSTATUS
// UsbAudioSetFormat(
//     IN PKSPIN Pin)
// {
//     PURB Urb;
//     PUCHAR SampleRateBuffer;
//     PPIN_CONTEXT PinContext;
//     NTSTATUS Status;
//     PKSDATAFORMAT_WAVEFORMATEX WaveFormatEx;
//
//     /* allocate sample rate buffer */
//     SampleRateBuffer = AllocFunction(sizeof(ULONG));
//     if (!SampleRateBuffer)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     if (IsEqualGUIDAligned(&Pin->ConnectionFormat->MajorFormat, &KSDATAFORMAT_TYPE_AUDIO) &&
//         IsEqualGUIDAligned(&Pin->ConnectionFormat->SubFormat, &KSDATAFORMAT_SUBTYPE_PCM) &&
//         IsEqualGUIDAligned(&Pin->ConnectionFormat->Specifier, &KSDATAFORMAT_SPECIFIER_WAVEFORMATEX))
//     {
//         WaveFormatEx = (PKSDATAFORMAT_WAVEFORMATEX)Pin->ConnectionFormat;
//         SampleRateBuffer[2] = (WaveFormatEx->WaveFormatEx.nSamplesPerSec >> 16) & 0xFF;
//         SampleRateBuffer[1] = (WaveFormatEx->WaveFormatEx.nSamplesPerSec >> 8) & 0xFF;
//         SampleRateBuffer[0] = (WaveFormatEx->WaveFormatEx.nSamplesPerSec >> 0) & 0xFF;
//
//         /* TODO: verify connection format */
//     }
//     else
//     {
//         /* not supported yet*/
//         UNIMPLEMENTED;
//         FreeFunction(SampleRateBuffer);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* allocate urb */
//     Urb = AllocFunction(sizeof(struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST));
//     if (!Urb)
//     {
//         /* no memory */
//         FreeFunction(SampleRateBuffer);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* get pin context */
//     PinContext = Pin->Context;
//
//     /* FIXME: determine controls and format urb */
//     UsbBuildVendorRequest(Urb,
//         URB_FUNCTION_CLASS_ENDPOINT,
//         sizeof(struct _URB_CONTROL_VENDOR_OR_CLASS_REQUEST),
//         USBD_TRANSFER_DIRECTION_OUT,
//         0,
//         0x01, // SET_CUR
//         0x100,
//         PinContext->DeviceExtension->InterfaceInfo->Pipes[0].EndpointAddress,
//         SampleRateBuffer,
//         NULL,
//         3,
//         NULL);
//
//
//
//     /* submit urb */
//     Status = SubmitUrbSync(PinContext->LowerDevice, Urb);
//
//     DPRINT1("USBAudioPinSetDataFormat Pin %p Status %x\n", Pin, Status);
//     FreeFunction(Urb);
//     FreeFunction(SampleRateBuffer);
//     return Status;
// }
//
// NTSTATUS
// USBAudioSelectAudioStreamingInterface(
//     IN PKSPIN Pin,
//     IN PPIN_CONTEXT PinContext,
//     IN PDEVICE_EXTENSION DeviceExtension,
//     IN PUSB_CONFIGURATION_DESCRIPTOR ConfigurationDescriptor,
//     IN ULONG FormatDescriptorIndex)
// {
//     PURB Urb;
//     PUSB_INTERFACE_DESCRIPTOR InterfaceDescriptor;
//     NTSTATUS Status;
//     ULONG Found, Index;
//
//     PUSB_AUDIO_STREAMING_INTERFACE_DESCRIPTOR StreamingInterfaceDescriptor;
//     PUSB_AUDIO_CONTROL_INPUT_TERMINAL_DESCRIPTOR TerminalDescriptor = NULL;
//
//     /* search for terminal descriptor of that irp sink / irp source */
//     TerminalDescriptor = UsbAudioGetStreamingTerminalDescriptorByIndex(DeviceExtension->ConfigurationDescriptor, Pin->Id);
//     ASSERT(TerminalDescriptor != NULL);
//
//     /* grab interface descriptor */
//     InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, ConfigurationDescriptor, -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1);
//     if (!InterfaceDescriptor)
//     {
//         /* no such interface */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Found = FALSE;
//     Index = 0;
//
//     /* selects the interface which has an audio streaming interface descriptor associated to the input / output terminal at the given format index */
//     while (InterfaceDescriptor != NULL)
//     {
//         if (InterfaceDescriptor->bInterfaceSubClass == 0x02 /* AUDIO_STREAMING */ && InterfaceDescriptor->bNumEndpoints > 0)
//         {
//             StreamingInterfaceDescriptor = (PUSB_AUDIO_STREAMING_INTERFACE_DESCRIPTOR)USBD_ParseDescriptors(ConfigurationDescriptor, ConfigurationDescriptor->wTotalLength, InterfaceDescriptor, USB_AUDIO_CONTROL_TERMINAL_DESCRIPTOR_TYPE);
//             if (StreamingInterfaceDescriptor != NULL)
//             {
//                 ASSERT(StreamingInterfaceDescriptor->bDescriptorSubtype == 0x01);
//                 ASSERT(StreamingInterfaceDescriptor->wFormatTag == WAVE_FORMAT_PCM);
//                 if (StreamingInterfaceDescriptor->bTerminalLink == TerminalDescriptor->bTerminalID)
//                 {
//                     if (FormatDescriptorIndex == Index)
//                     {
//                         Found = TRUE;
//                         break;
//                     }
//                     Index++;
//                 }
//             }
//         }
//         InterfaceDescriptor = USBD_ParseConfigurationDescriptorEx(ConfigurationDescriptor, (PVOID)((ULONG_PTR)InterfaceDescriptor + InterfaceDescriptor->bLength), -1, -1, USB_DEVICE_CLASS_AUDIO, -1, -1);
//     }
//
//     if (!Found)
//     {
//         /* no such interface */
//         DPRINT1("No Interface found\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     Urb = AllocFunction(GET_SELECT_INTERFACE_REQUEST_SIZE(InterfaceDescriptor->bNumEndpoints));
//     if (!Urb)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//      /* now prepare interface urb */
//      UsbBuildSelectInterfaceRequest(Urb, GET_SELECT_INTERFACE_REQUEST_SIZE(InterfaceDescriptor->bNumEndpoints), DeviceExtension->ConfigurationHandle, InterfaceDescriptor->bInterfaceNumber, InterfaceDescriptor->bAlternateSetting);
//
//      /* now select the interface */
//      Status = SubmitUrbSync(DeviceExtension->LowerDevice, Urb);
//
//      DPRINT1("USBAudioSelectAudioStreamingInterface Status %x UrbStatus %x InterfaceNumber %x AlternateSetting %x\n", Status, Urb->UrbSelectInterface.Hdr.Status, InterfaceDescriptor->bInterfaceNumber, InterfaceDescriptor->bAlternateSetting);
//
//      /* did it succeeed */
//      if (NT_SUCCESS(Status))
//      {
//          /* free old interface info */
//          if (DeviceExtension->InterfaceInfo)
//          {
//              /* free old info */
//              FreeFunction(DeviceExtension->InterfaceInfo);
//          }
//
//          /* alloc interface info */
//          DeviceExtension->InterfaceInfo = AllocFunction(Urb->UrbSelectInterface.Interface.Length);
//          if (DeviceExtension->InterfaceInfo == NULL)
//          {
//              /* no memory */
//              FreeFunction(Urb);
//              return STATUS_INSUFFICIENT_RESOURCES;
//          }
//
//          /* copy interface info */
//          RtlCopyMemory(DeviceExtension->InterfaceInfo, &Urb->UrbSelectInterface.Interface, Urb->UrbSelectInterface.Interface.Length);
//          PinContext->InterfaceDescriptor = InterfaceDescriptor;
//      }
//
//      /* free urb */
//      FreeFunction(Urb);
//      return Status;
// }
//
// VOID
// NTAPI
// CaptureGateOnWorkItem(
//      PVOID Context)
// {
//     PKSPIN Pin;
//     PPIN_CONTEXT PinContext;
//     PKSGATE Gate;
//     ULONG Count;
//
//     /* get pin */
//     Pin = Context;
//
//     /* get pin context */
//     PinContext = Pin->Context;
//
//     do
//     {
//         /* acquire processing mutex */
//         KsPinAcquireProcessingMutex(Pin);
//
//         /* get pin control gate */
//         Gate = KsPinGetAndGate(Pin);
//
//         /* turn input on */
//         KsGateTurnInputOn(Gate);
//
//         /* schedule processing */
//         KsPinAttemptProcessing(Pin, TRUE);
//
//         /* release processing mutex */
//         KsPinReleaseProcessingMutex(Pin);
//
//         /* decrement worker count */
//         Count = KsDecrementCountedWorker(PinContext->CaptureWorker);
//     } while (Count);
// }
//
// NTSTATUS
// RenderInitializeUrbAndIrp(
//     IN PKSPIN Pin,
//     IN PPIN_CONTEXT PinContext,
//     IN OUT PIRP Irp,
//     IN PVOID TransferBuffer,
//     IN ULONG TransferBufferSize,
//     IN ULONG PacketSize)
// {
//     ULONG Index, PacketCount;
//     PURB Urb;
//     PIO_STACK_LOCATION IoStack;
//
//     /* initialize irp */
//     IoInitializeIrp(Irp, IoSizeOfIrp(PinContext->DeviceExtension->LowerDevice->StackSize), PinContext->DeviceExtension->LowerDevice->StackSize);
//
//     /* set irp members */
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//     Irp->IoStatus.Information = 0;
//     Irp->Flags = 0;
//     Irp->UserBuffer = NULL;
//
//     /* init stack location */
//     IoStack = IoGetNextIrpStackLocation(Irp);
//     IoStack->DeviceObject = PinContext->DeviceExtension->LowerDevice;
//     IoStack->Parameters.Others.Argument2 = NULL;
//     IoStack->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//     IoStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;
//
//     /* set completion routine */
//     IoSetCompletionRoutine(Irp, UsbAudioRenderComplete, Pin, TRUE, TRUE, TRUE);
//
//     /* calculate packet count */
//     PacketCount = TransferBufferSize / PacketSize;
//     ASSERT(TransferBufferSize % PacketSize == 0);
//
//     /* lets allocate urb */
//     Urb = (PURB)AllocFunction(GET_ISO_URB_SIZE(PacketCount));
//     if (!Urb)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* init urb */
//     Urb->UrbIsochronousTransfer.Hdr.Function = URB_FUNCTION_ISOCH_TRANSFER;
//     Urb->UrbIsochronousTransfer.Hdr.Length = GET_ISO_URB_SIZE(PacketCount);
//     Urb->UrbIsochronousTransfer.PipeHandle = PinContext->DeviceExtension->InterfaceInfo->Pipes[0].PipeHandle;
//     Urb->UrbIsochronousTransfer.TransferFlags = USBD_TRANSFER_DIRECTION_OUT | USBD_START_ISO_TRANSFER_ASAP;
//     Urb->UrbIsochronousTransfer.TransferBufferLength = TransferBufferSize;
//     Urb->UrbIsochronousTransfer.TransferBuffer = TransferBuffer;
//     Urb->UrbIsochronousTransfer.NumberOfPackets = PacketCount;
//     Urb->UrbIsochronousTransfer.StartFrame = 0;
//
//     for (Index = 0; Index < PacketCount; Index++)
//     {
//         Urb->UrbIsochronousTransfer.IsoPacket[Index].Offset = Index * PacketSize;
//     }
//
//     /* store urb */
//     IoStack->Parameters.Others.Argument1 = Urb;
//     Irp->Tail.Overlay.DriverContext[0] = Urb;
//
//
//     /* done */
//     return STATUS_SUCCESS;
// }
//
// VOID
// CaptureInitializeUrbAndIrp(
//     IN PKSPIN Pin,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PURB Urb;
//     PUCHAR TransferBuffer;
//     ULONG Index;
//     PPIN_CONTEXT PinContext;
//
//     /* get pin context */
//     PinContext = Pin->Context;
//
//     /* backup urb and transferbuffer */
//     Urb = Irp->Tail.Overlay.DriverContext[0];
//     TransferBuffer = Urb->UrbIsochronousTransfer.TransferBuffer;
//
//     /* initialize irp */
//     IoInitializeIrp(Irp, IoSizeOfIrp(PinContext->DeviceExtension->LowerDevice->StackSize), PinContext->DeviceExtension->LowerDevice->StackSize);
//
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//     Irp->IoStatus.Information = 0;
//     Irp->Flags = 0;
//     Irp->UserBuffer = NULL;
//     Irp->Tail.Overlay.DriverContext[0] = Urb;
//     Irp->Tail.Overlay.DriverContext[1] = NULL;
//
//     /* init stack location */
//     IoStack = IoGetNextIrpStackLocation(Irp);
//     IoStack->DeviceObject = PinContext->DeviceExtension->LowerDevice;
//     IoStack->Parameters.Others.Argument1 = Urb;
//     IoStack->Parameters.Others.Argument2 = NULL;
//     IoStack->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//     IoStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;
//
//     IoSetCompletionRoutine(Irp, UsbAudioCaptureComplete, Pin, TRUE, TRUE, TRUE);
//
//     RtlZeroMemory(Urb, GET_ISO_URB_SIZE(PACKET_COUNT));
//
//     /* init urb */
//     Urb->UrbIsochronousTransfer.Hdr.Function = URB_FUNCTION_ISOCH_TRANSFER;
//     Urb->UrbIsochronousTransfer.Hdr.Length = GET_ISO_URB_SIZE(10);
//     Urb->UrbIsochronousTransfer.PipeHandle = PinContext->DeviceExtension->InterfaceInfo->Pipes[0].PipeHandle;
//     Urb->UrbIsochronousTransfer.TransferFlags = USBD_TRANSFER_DIRECTION_IN | USBD_START_ISO_TRANSFER_ASAP;
//     Urb->UrbIsochronousTransfer.TransferBufferLength = PinContext->DeviceExtension->InterfaceInfo->Pipes[0].MaximumPacketSize * 10;
//     Urb->UrbIsochronousTransfer.TransferBuffer = TransferBuffer;
//     Urb->UrbIsochronousTransfer.NumberOfPackets = PACKET_COUNT;
//     Urb->UrbIsochronousTransfer.StartFrame = 0;
//
//     for (Index = 0; Index < PACKET_COUNT; Index++)
//     {
//         Urb->UrbIsochronousTransfer.IsoPacket[Index].Offset = Index * PinContext->DeviceExtension->InterfaceInfo->Pipes[0].MaximumPacketSize;
//     }
// }
//
//
// VOID
// NTAPI
// CaptureAvoidPipeStarvationWorker(
//      PVOID Context)
// {
//     PKSPIN Pin;
//     PPIN_CONTEXT PinContext;
//     KIRQL OldLevel;
//     PLIST_ENTRY CurEntry;
//     PIRP Irp;
//
//     /* get pin */
//     Pin = Context;
//
//     /* get pin context */
//     PinContext = Pin->Context;
//
//     /* acquire spin lock */
//     KeAcquireSpinLock(&PinContext->IrpListLock, &OldLevel);
//
//     if (!IsListEmpty(&PinContext->IrpListHead))
//     {
//         /* sanity check */
//         ASSERT(!IsListEmpty(&PinContext->IrpListHead));
//
//         /* remove entry from list */
//         CurEntry = RemoveHeadList(&PinContext->IrpListHead);
//
//         /* release lock */
//         KeReleaseSpinLock(&PinContext->IrpListLock, OldLevel);
//
//         /* get irp offset */
//         Irp = (PIRP)CONTAINING_RECORD(CurEntry, IRP, Tail.Overlay.ListEntry);
//
//         /* reinitialize irp and urb */
//         CaptureInitializeUrbAndIrp(Pin, Irp);
//
//         KsDecrementCountedWorker(PinContext->StarvationWorker);
//
//         /* call driver */
//         IoCallDriver(PinContext->DeviceExtension->LowerDevice, Irp);
//     }
//     else
//     {
//         /* release lock */
//         KeReleaseSpinLock(&PinContext->IrpListLock, OldLevel);
//
//         KsDecrementCountedWorker(PinContext->StarvationWorker);
//     }
// }
//
//
//
// NTSTATUS
// InitCapturePin(
//     IN PKSPIN Pin)
// {
//     NTSTATUS Status;
//     ULONG Index;
//     ULONG BufferSize;
//     ULONG MaximumPacketSize;
//     PIRP Irp;
//     PURB Urb;
//     PPIN_CONTEXT PinContext;
//     PIO_STACK_LOCATION IoStack;
//     PKSALLOCATOR_FRAMING_EX Framing;
//     PKSGATE Gate;
//
//
//     /* set sample rate */
//     Status = UsbAudioSetFormat(Pin);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         return Status;
//     }
//
//     /* get pin context */
//     PinContext = Pin->Context;
//
//     /* lets get maximum packet size */
//     MaximumPacketSize = GetMaxPacketSizeForInterface(PinContext->DeviceExtension->ConfigurationDescriptor, PinContext->InterfaceDescriptor, Pin->DataFlow);
//
//     /* initialize work item for capture worker */
//     ExInitializeWorkItem(&PinContext->CaptureWorkItem, CaptureGateOnWorkItem, (PVOID)Pin);
//
//     /* register worker */
//     Status = KsRegisterCountedWorker(CriticalWorkQueue, &PinContext->CaptureWorkItem, &PinContext->CaptureWorker);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         return Status;
//     }
//
//     /* initialize work item */
//     ExInitializeWorkItem(&PinContext->StarvationWorkItem, CaptureAvoidPipeStarvationWorker, (PVOID)Pin);
//
//     /* register worker */
//     Status = KsRegisterCountedWorker(CriticalWorkQueue, &PinContext->StarvationWorkItem, &PinContext->StarvationWorker);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         KsUnregisterWorker(PinContext->CaptureWorker);
//     }
//
//     /* lets edit framing struct */
//     Framing = (PKSALLOCATOR_FRAMING_EX)Pin->Descriptor->AllocatorFraming;
//     Framing->FramingItem[0].PhysicalRange.MinFrameSize =
//         Framing->FramingItem[0].PhysicalRange.MaxFrameSize =
//         Framing->FramingItem[0].FramingRange.Range.MinFrameSize =
//         Framing->FramingItem[0].FramingRange.Range.MaxFrameSize =
//     MaximumPacketSize;
//
//     /* calculate buffer size 8 irps * 10 iso packets * max packet size */
//     BufferSize = 8 * PACKET_COUNT * MaximumPacketSize;
//
//     /* allocate pin capture buffer */
//     PinContext->BufferSize = BufferSize;
//     PinContext->Buffer = AllocFunction(BufferSize);
//     if (!PinContext->Buffer)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     KsAddItemToObjectBag(Pin->Bag, PinContext->Buffer, ExFreePool);
//
//     /* init irps */
//     for (Index = 0; Index < 8; Index++)
//     {
//         /* allocate irp */
//         Irp = AllocFunction(IoSizeOfIrp(PinContext->DeviceExtension->LowerDevice->StackSize));
//         if (!Irp)
//         {
//             /* no memory */
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* initialize irp */
//         IoInitializeIrp(Irp, IoSizeOfIrp(PinContext->DeviceExtension->LowerDevice->StackSize), PinContext->DeviceExtension->LowerDevice->StackSize);
//
//         Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//         Irp->IoStatus.Information = 0;
//         Irp->Flags = 0;
//         Irp->UserBuffer = NULL;
//
//         IoStack = IoGetNextIrpStackLocation(Irp);
//         IoStack->DeviceObject = PinContext->DeviceExtension->LowerDevice;
//         IoStack->Parameters.Others.Argument2 = NULL;
//         IoStack->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//         IoStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;
//
//         IoSetCompletionRoutine(Irp, UsbAudioCaptureComplete, Pin, TRUE, TRUE, TRUE);
//
//         /* insert into irp list */
//         InsertTailList(&PinContext->IrpListHead, &Irp->Tail.Overlay.ListEntry);
//
//         /* add to object bag*/
//         KsAddItemToObjectBag(Pin->Bag, Irp, ExFreePool);
//
//         /* FIXME select correct pipe handle */
//         Status = UsbAudioAllocCaptureUrbIso(PinContext->DeviceExtension->InterfaceInfo->Pipes[0].PipeHandle,
//                                             MaximumPacketSize,
//                                             &PinContext->Buffer[MaximumPacketSize * PACKET_COUNT * Index],
//                                             MaximumPacketSize * PACKET_COUNT,
//                                             &Urb);
//
//         DPRINT1("InitCapturePin Irp %p Urb %p\n", Irp, Urb);
//
//         if (NT_SUCCESS(Status))
//         {
//             /* get next stack location */
//             IoStack = IoGetNextIrpStackLocation(Irp);
//
//             /* store urb */
//             IoStack->Parameters.Others.Argument1 = Urb;
//             Irp->Tail.Overlay.DriverContext[0] = Urb;
//         }
//         else
//         {
//             /* failed */
//             return Status;
//         }
//     }
//
//     /* get process control gate */
//     Gate = KsPinGetAndGate(Pin);
//
//     /* turn input off */
//     KsGateTurnInputOff(Gate);
//
//     return Status;
// }
//
// NTSTATUS
// InitStreamPin(
//     IN PKSPIN Pin)
// {
//     ULONG Index;
//     PIRP Irp;
//     PPIN_CONTEXT PinContext;
//     PKSDATAFORMAT_WAVEFORMATEX WaveFormatEx;
//     PIO_STACK_LOCATION IoStack;
//
//     DPRINT1("InitStreamPin\n");
//
//     /* get pin context */
//     PinContext = Pin->Context;
//
//     /* allocate 1 sec buffer */
//     WaveFormatEx = (PKSDATAFORMAT_WAVEFORMATEX)Pin->ConnectionFormat;
//     PinContext->Buffer = AllocFunction(WaveFormatEx->WaveFormatEx.nAvgBytesPerSec);
//     if (!PinContext->Buffer)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* init buffer size*/
//     PinContext->BufferSize = WaveFormatEx->WaveFormatEx.nAvgBytesPerSec;
//     PinContext->BufferOffset = 0;
//     PinContext->BufferLength = 0;
//
//     /* init irps */
//     for (Index = 0; Index < 12; Index++)
//     {
//         /* allocate irp */
//         Irp = AllocFunction(IoSizeOfIrp(PinContext->DeviceExtension->LowerDevice->StackSize));
//         if (!Irp)
//         {
//             /* no memory */
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         DPRINT1("InitStreamPin Irp %p\n", Irp);
//
//         /* initialize irp */
//         IoInitializeIrp(Irp, IoSizeOfIrp(PinContext->DeviceExtension->LowerDevice->StackSize), PinContext->DeviceExtension->LowerDevice->StackSize);
//
//         Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//         Irp->IoStatus.Information = 0;
//         Irp->Flags = 0;
//         Irp->UserBuffer = NULL;
//
//         IoStack = IoGetNextIrpStackLocation(Irp);
//         IoStack->DeviceObject = PinContext->DeviceExtension->LowerDevice;
//         IoStack->Parameters.Others.Argument2 = NULL;
//         IoStack->MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
//         IoStack->Parameters.DeviceIoControl.IoControlCode = IOCTL_INTERNAL_USB_SUBMIT_URB;
//
//         IoSetCompletionRoutine(Irp, UsbAudioRenderComplete, Pin, TRUE, TRUE, TRUE);
//
//         /* insert into irp list */
//         InsertTailList(&PinContext->IrpListHead, &Irp->Tail.Overlay.ListEntry);
//
//         /* add to object bag*/
//         KsAddItemToObjectBag(Pin->Bag, Irp, ExFreePool);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// ULONG
// GetDataRangeIndexForFormat(
//     IN PKSDATARANGE ConnectionFormat,
//     IN const PKSDATARANGE * DataRanges,
//     IN ULONG DataRangesCount)
// {
//     ULONG Index;
//     PKSDATARANGE CurrentDataRange;
//     PKSDATARANGE_AUDIO CurrentAudioDataRange;
//     PKSDATAFORMAT_WAVEFORMATEX ConnectionDataFormat;
//
//     if (ConnectionFormat->FormatSize != sizeof(KSDATAFORMAT) + sizeof(WAVEFORMATEX))
//     {
//         /* unsupported connection format */
//         DPRINT1("GetDataRangeIndexForFormat expected KSDATARANGE_AUDIO\n");
//         return MAXULONG;
//     }
//
//     /* cast to right type */
//     ConnectionDataFormat = (PKSDATAFORMAT_WAVEFORMATEX)ConnectionFormat;
//
//     for (Index = 0; Index < DataRangesCount; Index++)
//     {
//          /* get current data range */
//          CurrentDataRange = DataRanges[Index];
//
//          /* compare guids */
//          if (!IsEqualGUIDAligned(&CurrentDataRange->MajorFormat, &ConnectionFormat->MajorFormat) ||
//              !IsEqualGUIDAligned(&CurrentDataRange->SubFormat, &ConnectionFormat->SubFormat) ||
//              !IsEqualGUIDAligned(&CurrentDataRange->Specifier, &ConnectionFormat->Specifier))
//          {
//              /* no match */
//              continue;
//          }
//
//          /* all pin data ranges are KSDATARANGE_AUDIO */
//          CurrentAudioDataRange = (PKSDATARANGE_AUDIO)CurrentDataRange;
//
//          /* check if number of channel match */
//          if (CurrentAudioDataRange->MaximumChannels != ConnectionDataFormat->WaveFormatEx.nChannels)
//          {
//              /* number of channels mismatch */
//              continue;
//          }
//
//          if (CurrentAudioDataRange->MinimumSampleFrequency > ConnectionDataFormat->WaveFormatEx.nSamplesPerSec)
//          {
//              /* channel frequency too low */
//              continue;
//          }
//
//          if (CurrentAudioDataRange->MaximumSampleFrequency < ConnectionDataFormat->WaveFormatEx.nSamplesPerSec)
//          {
//              /* channel frequency too high */
//              continue;
//          }
//
//          /* FIXME add checks for bitrate / sample size etc */
//          return Index;
//     }
//
//     /* no datarange found */
//     return MAXULONG;
// }
//
// NTSTATUS
// NTAPI
// USBAudioPinCreate(
//      PKSPIN Pin,
//      PIRP Irp)
// {
//     PKSFILTER Filter;
//     PFILTER_CONTEXT FilterContext;
//     PPIN_CONTEXT PinContext;
//     NTSTATUS Status;
//     ULONG FormatIndex;
//
//     Filter = KsPinGetParentFilter(Pin);
//     if (Filter == NULL)
//     {
//         /* invalid parameter */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* get filter context */
//     FilterContext = Filter->Context;
//
//     /* allocate pin context */
//     PinContext = AllocFunction(sizeof(PIN_CONTEXT));
//     if (!PinContext)
//     {
//         /* no memory*/
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* init pin context */
//     PinContext->DeviceExtension = FilterContext->DeviceExtension;
//     PinContext->LowerDevice = FilterContext->LowerDevice;
//     InitializeListHead(&PinContext->IrpListHead);
//     InitializeListHead(&PinContext->DoneIrpListHead);
//     KeInitializeSpinLock(&PinContext->IrpListLock);
//
//     /* store pin context*/
//     Pin->Context = PinContext;
//
//     /* lets edit allocator framing struct */
//     Status = _KsEdit(Pin->Bag, (PVOID*)&Pin->Descriptor, sizeof(KSPIN_DESCRIPTOR_EX), sizeof(KSPIN_DESCRIPTOR_EX), USBAUDIO_TAG);
//     if (NT_SUCCESS(Status))
//     {
//         Status = _KsEdit(Pin->Bag, (PVOID*)&Pin->Descriptor->AllocatorFraming, sizeof(KSALLOCATOR_FRAMING_EX), sizeof(KSALLOCATOR_FRAMING_EX), USBAUDIO_TAG);
//         ASSERT(Status == STATUS_SUCCESS);
//     }
//
//     /* choose correct dataformat */
//     FormatIndex = GetDataRangeIndexForFormat(Pin->ConnectionFormat, Pin->Descriptor->PinDescriptor.DataRanges, Pin->Descriptor->PinDescriptor.DataRangesCount);
//     if (FormatIndex == MAXULONG)
//     {
//         /* no format match */
//         return STATUS_NO_MATCH;
//     }
//
//     /* select streaming interface */
//     Status = USBAudioSelectAudioStreamingInterface(Pin, PinContext, PinContext->DeviceExtension, PinContext->DeviceExtension->ConfigurationDescriptor, FormatIndex);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         DPRINT1("USBAudioSelectAudioStreamingInterface failed with %x\n", Status);
//         return Status;
//     }
//
//     if (Pin->DataFlow == KSPIN_DATAFLOW_OUT)
//     {
//         /* init capture pin */
//         Status = InitCapturePin(Pin);
//     }
//     else
//     {
//         /* audio streaming pin*/
//         Status = InitStreamPin(Pin);
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBAudioPinClose(
//      PKSPIN Pin,
//      PIRP Irp)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// UsbAudioRenderComplete(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN PVOID Context)
// {
//     PKSPIN Pin;
//     PPIN_CONTEXT PinContext;
//     KIRQL OldLevel;
//     PKSSTREAM_POINTER StreamPointerClone;
//     NTSTATUS Status;
//     PURB Urb;
//
//     /* get pin context */
//     Pin = Context;
//     PinContext = Pin->Context;
//
//     /* get status */
//     Status = Irp->IoStatus.Status;
//
//     /* get streampointer */
//     StreamPointerClone = Irp->Tail.Overlay.DriverContext[1];
//
//     /* get urb */
//     Urb = Irp->Tail.Overlay.DriverContext[0];
//
//     /* and free it */
//     FreeFunction(Urb);
//
//     /* acquire lock */
//     KeAcquireSpinLock(&PinContext->IrpListLock, &OldLevel);
//
//     /* insert entry into ready list */
//     InsertTailList(&PinContext->IrpListHead, &Irp->Tail.Overlay.ListEntry);
//
//     /* release lock */
//     KeReleaseSpinLock(&PinContext->IrpListLock, OldLevel);
//
//     if (!NT_SUCCESS(Status) && StreamPointerClone)
//     {
//         /* set status code because it failed */
//         KsStreamPointerSetStatusCode(StreamPointerClone, STATUS_DEVICE_DATA_ERROR);
//         DPRINT1("UsbAudioRenderComplete failed with %x\n", Status);
//     }
//
//     if (StreamPointerClone)
//     {
//         /* lets delete the stream pointer clone */
//         KsStreamPointerDelete(StreamPointerClone);
//     }
//
//     /* done */
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// NTSTATUS
// NTAPI
// UsbAudioCaptureComplete(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN PVOID Context)
// {
//     PKSPIN Pin;
//     PPIN_CONTEXT PinContext;
//     KIRQL OldLevel;
//     PURB Urb;
//
//     /* get pin context */
//     Pin = Context;
//     PinContext = Pin->Context;
//
//     /* get urb */
//     Urb = Irp->Tail.Overlay.DriverContext[0];
//
//     /* acquire lock */
//     KeAcquireSpinLock(&PinContext->IrpListLock, &OldLevel);
//
//     if (!NT_SUCCESS(Urb->UrbIsochronousTransfer.Hdr.Status))
//     {
//         //DPRINT("UsbAudioCaptureComplete Irp %p Urb %p Status %x Packet Status %x\n", Irp, Urb, Urb->UrbIsochronousTransfer.Hdr.Status, Urb->UrbIsochronousTransfer.IsoPacket[0].Status);
//
//         /* insert entry into ready list */
//         InsertTailList(&PinContext->IrpListHead, &Irp->Tail.Overlay.ListEntry);
//
//         /* release lock */
//         KeReleaseSpinLock(&PinContext->IrpListLock, OldLevel);
//
//         KsIncrementCountedWorker(PinContext->StarvationWorker);
//     }
//     else
//     {
//         /* insert entry into done list */
//         InsertTailList(&PinContext->DoneIrpListHead, &Irp->Tail.Overlay.ListEntry);
//
//         /* release lock */
//         KeReleaseSpinLock(&PinContext->IrpListLock, OldLevel);
//
//         KsIncrementCountedWorker(PinContext->CaptureWorker);
//     }
//
//     /* done */
//     return STATUS_MORE_PROCESSING_REQUIRED;
// }
//
// PIRP
// PinGetIrpFromReadyList(
//     IN PKSPIN Pin)
// {
//     PPIN_CONTEXT PinContext;
//     PLIST_ENTRY CurEntry;
//     KIRQL OldLevel;
//     PIRP Irp = NULL;
//
//     /* get pin context */
//     PinContext = Pin->Context;
//
//     /* acquire spin lock */
//     KeAcquireSpinLock(&PinContext->IrpListLock, &OldLevel);
//
//     if (!IsListEmpty(&PinContext->IrpListHead))
//     {
//         /* remove entry from list */
//         CurEntry = RemoveHeadList(&PinContext->IrpListHead);
//
//         /* get irp offset */
//         Irp = (PIRP)CONTAINING_RECORD(CurEntry, IRP, Tail.Overlay.ListEntry);
//     }
//
//     /* release lock */
//     KeReleaseSpinLock(&PinContext->IrpListLock, OldLevel);
//
//     return Irp;
// }
//
// NTSTATUS
// PinRenderProcess(
//     IN PKSPIN Pin)
// {
//     PKSSTREAM_POINTER LeadingStreamPointer;
//     PKSSTREAM_POINTER CloneStreamPointer;
//     NTSTATUS Status;
//     PPIN_CONTEXT PinContext;
//     ULONG PacketCount, TotalPacketSize, Offset;
//     PKSDATAFORMAT_WAVEFORMATEX WaveFormatEx;
//     PUCHAR TransferBuffer;
//     PIRP Irp = NULL;
//
//     //DPRINT1("PinRenderProcess\n");
//
//     LeadingStreamPointer = KsPinGetLeadingEdgeStreamPointer(Pin, KSSTREAM_POINTER_STATE_LOCKED);
//     if (LeadingStreamPointer == NULL)
//     {
//         return STATUS_SUCCESS;
//     }
//
//     if (NULL == LeadingStreamPointer->StreamHeader->Data)
//     {
//         Status = KsStreamPointerAdvance(LeadingStreamPointer);
//         DPRINT1("Advancing Streampointer\n");
//     }
//
//
//     /* get pin context */
//     PinContext = Pin->Context;
//
//     /* get irp from ready list */
//     Irp = PinGetIrpFromReadyList(Pin);
//
//     if (!Irp)
//     {
//         /* no irps available */
//         DPRINT1("No irps available");
//         KsStreamPointerUnlock(LeadingStreamPointer, TRUE);
//         return STATUS_SUCCESS;
//     }
//
//     /* clone stream pointer */
//     Status = KsStreamPointerClone(LeadingStreamPointer, NULL, 0, &CloneStreamPointer);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         KsStreamPointerUnlock(LeadingStreamPointer, TRUE);
//         DPRINT1("Leaking Irp %p\n", Irp);
//         return STATUS_SUCCESS;
//     }
//
//     /* calculate packet count */
//     /* FIXME support various sample rates */
//     WaveFormatEx = (PKSDATAFORMAT_WAVEFORMATEX)Pin->ConnectionFormat;
//     TotalPacketSize = WaveFormatEx->WaveFormatEx.nAvgBytesPerSec / 1000;
//
//     /* init transfer buffer*/
//     TransferBuffer = CloneStreamPointer->StreamHeader->Data;
//
//     Offset = 0;
//
//     /* are there bytes from previous request*/
//     if (PinContext->BufferLength)
//     {
//         ASSERT(PinContext->BufferLength < TotalPacketSize);
//
//         /* calculate offset*/
//         Offset = TotalPacketSize - PinContext->BufferLength;
//
//         if (PinContext->BufferOffset + TotalPacketSize >= PinContext->BufferSize)
//         {
//             RtlMoveMemory(PinContext->Buffer, &PinContext->Buffer[PinContext->BufferOffset - PinContext->BufferLength], PinContext->BufferLength);
//             PinContext->BufferOffset = PinContext->BufferLength;
//         }
//
//         /* copy audio bytes */
//         RtlCopyMemory(&PinContext->Buffer[PinContext->BufferOffset], TransferBuffer, Offset);
//
//         /* init irp*/
//         Status = RenderInitializeUrbAndIrp(Pin, PinContext, Irp, &PinContext->Buffer[PinContext->BufferOffset-PinContext->BufferLength], TotalPacketSize, TotalPacketSize);
//         if (NT_SUCCESS(Status))
//         {
//             /* render audio bytes */
//             Status = IoCallDriver(PinContext->LowerDevice, Irp);
//         }
//         else
//         {
//             ASSERT(FALSE);
//         }
//
//         PinContext->BufferLength = 0;
//         PinContext->BufferOffset += Offset;
//
//         /* get new irp from ready list */
//         Irp = PinGetIrpFromReadyList(Pin);
//         ASSERT(Irp);
//
//     }
//
//     /* FIXME correct MaximumPacketSize ? */
//     PacketCount = (CloneStreamPointer->OffsetIn.Remaining - Offset) / TotalPacketSize;
//
//     Status = RenderInitializeUrbAndIrp(Pin, PinContext, Irp, &TransferBuffer[Offset], PacketCount * TotalPacketSize, TotalPacketSize);
//     if (NT_SUCCESS(Status))
//     {
//         /* store in irp context */
//         Irp->Tail.Overlay.DriverContext[1] = CloneStreamPointer;
//
//         if ((PacketCount * TotalPacketSize) + Offset < CloneStreamPointer->OffsetIn.Remaining)
//         {
//             /* calculate remaining buffer bytes */
//             PinContext->BufferLength = CloneStreamPointer->OffsetIn.Remaining - ((PacketCount * TotalPacketSize) + Offset);
//
//             /* check for overflow */
//             if (PinContext->BufferOffset + TotalPacketSize >= PinContext->BufferSize)
//             {
//                 /* reset buffer offset*/
//                 PinContext->BufferOffset = 0;
//             }
//             RtlCopyMemory(&PinContext->Buffer[PinContext->BufferOffset], &TransferBuffer[(PacketCount * TotalPacketSize) + Offset], PinContext->BufferLength);
//             PinContext->BufferOffset += PinContext->BufferLength;
//         }
//
//         /* render audio bytes */
//         Status = IoCallDriver(PinContext->LowerDevice, Irp);
//     }
//
//
//     /* unlock stream pointer and finish*/
//     KsStreamPointerUnlock(LeadingStreamPointer, TRUE);
//     return STATUS_PENDING;
// }
//
// NTSTATUS
// PinCaptureProcess(
//     IN PKSPIN Pin)
// {
//     PKSSTREAM_POINTER LeadingStreamPointer;
//     KIRQL OldLevel;
//     PPIN_CONTEXT PinContext;
//     PLIST_ENTRY CurEntry;
//     PIRP Irp;
//     PURB Urb;
//     PUCHAR TransferBuffer, OutBuffer;
//     ULONG Offset, Length;
//     NTSTATUS Status;
//     PKSGATE Gate;
//
//     //DPRINT1("PinCaptureProcess\n");
//     LeadingStreamPointer = KsPinGetLeadingEdgeStreamPointer(Pin, KSSTREAM_POINTER_STATE_LOCKED);
//     if (LeadingStreamPointer == NULL)
//     {
//         /* get process control gate */
//         Gate = KsPinGetAndGate(Pin);
//
//         /* shutdown processing */
//         KsGateTurnInputOff(Gate);
//
//         return STATUS_SUCCESS;
//     }
//
//     /* get pin context */
//     PinContext = Pin->Context;
//
//     /* acquire spin lock */
//     KeAcquireSpinLock(&PinContext->IrpListLock, &OldLevel);
//
//     while (!IsListEmpty(&PinContext->DoneIrpListHead))
//     {
//         /* remove entry from list */
//         CurEntry = RemoveHeadList(&PinContext->DoneIrpListHead);
//
//         /* release lock */
//         KeReleaseSpinLock(&PinContext->IrpListLock, OldLevel);
//
//         /* get irp offset */
//         Irp = (PIRP)CONTAINING_RECORD(CurEntry, IRP, Tail.Overlay.ListEntry);
//
//         /* get urb from irp */
//         Urb = (PURB)Irp->Tail.Overlay.DriverContext[0];
//         ASSERT(Urb);
//
//         Offset = PtrToUlong(Irp->Tail.Overlay.DriverContext[1]);
//
//         /* get transfer buffer */
//         TransferBuffer = Urb->UrbIsochronousTransfer.TransferBuffer;
//
//         /* get target buffer */
//         OutBuffer = (PUCHAR)LeadingStreamPointer->StreamHeader->Data;
//
//         /* calculate length */
//         Length = min(LeadingStreamPointer->OffsetOut.Count - LeadingStreamPointer->StreamHeader->DataUsed, Urb->UrbIsochronousTransfer.TransferBufferLength - Offset);
//
//         /* FIXME copy each packet extra */
//         /* copy audio bytes */
//         RtlCopyMemory((PUCHAR)&OutBuffer[LeadingStreamPointer->StreamHeader->DataUsed], &TransferBuffer[Offset], Length);
//
//         //DPRINT1("Irp %p Urb %p OutBuffer %p TransferBuffer %p Offset %lu Remaining %lu TransferBufferLength %lu Length %lu\n", Irp, Urb, OutBuffer, TransferBuffer, Offset, LeadingStreamPointer->OffsetOut.Remaining, Urb->UrbIsochronousTransfer.TransferBufferLength, Length);
//
//         /* adjust streampointer */
//         LeadingStreamPointer->StreamHeader->DataUsed += Length;
//
//         if (Length == LeadingStreamPointer->OffsetOut.Remaining)
//         {
//             KsStreamPointerAdvanceOffsetsAndUnlock(LeadingStreamPointer, 0, Length, TRUE);
//
//             /* acquire spin lock */
//             KeAcquireSpinLock(&PinContext->IrpListLock, &OldLevel);
//
//             /* adjust offset */
//             Irp->Tail.Overlay.DriverContext[1] = UlongToPtr(Length);
//
//             /* reinsert into processed list */
//             InsertHeadList(&PinContext->DoneIrpListHead, &Irp->Tail.Overlay.ListEntry);
//
//             /* release lock */
//             KeReleaseSpinLock(&PinContext->IrpListLock, OldLevel);
//
//             LeadingStreamPointer = KsPinGetLeadingEdgeStreamPointer(Pin, KSSTREAM_POINTER_STATE_LOCKED);
//             if (LeadingStreamPointer == NULL)
//             {
//                 /* no more work to be done*/
//                 return STATUS_PENDING;
//             }
//             else
//             {
//                 /* resume work on this irp */
//                 continue;
//             }
//         }
//         else
//         {
//             Status = KsStreamPointerAdvanceOffsets(LeadingStreamPointer, 0, Length, FALSE);
//             NT_ASSERT(NT_SUCCESS(Status));
//             ASSERT(Length == Urb->UrbIsochronousTransfer.TransferBufferLength - Offset);
//         }
//
//
//         /* acquire spin lock */
//         KeAcquireSpinLock(&PinContext->IrpListLock, &OldLevel);
//
//         InsertTailList(&PinContext->IrpListHead, &Irp->Tail.Overlay.ListEntry);
//     }
//
//     while (!IsListEmpty(&PinContext->IrpListHead))
//     {
//         /* remove entry from list */
//         CurEntry = RemoveHeadList(&PinContext->IrpListHead);
//
//         /* release lock */
//         KeReleaseSpinLock(&PinContext->IrpListLock, OldLevel);
//
//         /* get irp offset */
//         Irp = (PIRP)CONTAINING_RECORD(CurEntry, IRP, Tail.Overlay.ListEntry);
//
//         /* reinitialize irp and urb */
//         CaptureInitializeUrbAndIrp(Pin, Irp);
//
//         IoCallDriver(PinContext->DeviceExtension->LowerDevice, Irp);
//
//         /* acquire spin lock */
//         KeAcquireSpinLock(&PinContext->IrpListLock, &OldLevel);
//
//     }
//
//     /* release lock */
//     KeReleaseSpinLock(&PinContext->IrpListLock, OldLevel);
//
//     if (LeadingStreamPointer != NULL)
//         KsStreamPointerUnlock(LeadingStreamPointer, FALSE);
//
//     /* get process control gate */
//     Gate = KsPinGetAndGate(Pin);
//
//     /* shutdown processing */
//     KsGateTurnInputOff(Gate);
//
//     return STATUS_PENDING;
// }
//
//
// NTSTATUS
// NTAPI
// USBAudioPinProcess(
//      PKSPIN Pin)
// {
//     NTSTATUS Status;
//
//     if (Pin->DataFlow == KSPIN_DATAFLOW_OUT)
//     {
//         Status = PinCaptureProcess(Pin);
//     }
//     else
//     {
//         Status = PinRenderProcess(Pin);
//     }
//
//     return Status;
// }
//
//
// VOID
// NTAPI
// USBAudioPinReset(
//      PKSPIN Pin)
// {
//     UNIMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// USBAudioPinSetDataFormat(
//      PKSPIN Pin,
//      PKSDATAFORMAT OldFormat,
//      PKSMULTIPLE_ITEM OldAttributeList,
//      const KSDATARANGE* DataRange,
//      const KSATTRIBUTE_LIST* AttributeRange)
// {
//     if (OldFormat == NULL)
//     {
//         /* TODO: verify connection format */
//         UNIMPLEMENTED;
//         return STATUS_SUCCESS;
//     }
//
//     return UsbAudioSetFormat(Pin);
// }
//
// NTSTATUS
// StartCaptureIsocTransfer(
//     IN PKSPIN Pin)
// {
//     PPIN_CONTEXT PinContext;
//     PLIST_ENTRY CurEntry;
//     PIRP Irp;
//     KIRQL OldLevel;
//
//     /* get pin context */
//     PinContext = Pin->Context;
//
//     /* acquire spin lock */
//     KeAcquireSpinLock(&PinContext->IrpListLock, &OldLevel);
//
//     while(!IsListEmpty(&PinContext->IrpListHead))
//     {
//         /* remove entry from list */
//         CurEntry = RemoveHeadList(&PinContext->IrpListHead);
//
//         /* get irp offset */
//         Irp = (PIRP)CONTAINING_RECORD(CurEntry, IRP, Tail.Overlay.ListEntry);
//
//         /* release lock */
//         KeReleaseSpinLock(&PinContext->IrpListLock, OldLevel);
//
//         /* reinitialize irp and urb */
//         CaptureInitializeUrbAndIrp(Pin, Irp);
//
//         DPRINT("StartCaptureIsocTransfer Irp %p\n", Irp);
//         IoCallDriver(PinContext->DeviceExtension->LowerDevice, Irp);
//
//         /* acquire spin lock */
//         KeAcquireSpinLock(&PinContext->IrpListLock, &OldLevel);
//
//     }
//
//     /* release lock */
//     KeReleaseSpinLock(&PinContext->IrpListLock, OldLevel);
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// CapturePinStateChange(
//      PKSPIN Pin,
//      KSSTATE ToState,
//      KSSTATE FromState)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     if (FromState != ToState)
//     {
//         if (ToState)
//         {
//             if (ToState == KSSTATE_PAUSE)
//             {
//                 if (FromState == KSSTATE_RUN)
//                 {
//                     /* wait until pin processing is finished*/
//                 }
//             }
//             else
//             {
//                 if (ToState == KSSTATE_RUN)
//                 {
//                     Status = StartCaptureIsocTransfer(Pin);
//                 }
//             }
//         }
//     }
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// USBAudioPinSetDeviceState(
//      PKSPIN Pin,
//      KSSTATE ToState,
//      KSSTATE FromState)
// {
//     NTSTATUS Status;
//
//     if (Pin->DataFlow == KSPIN_DATAFLOW_OUT)
//     {
//         /* handle capture state changes */
//         Status = CapturePinStateChange(Pin, ToState, FromState);
//     }
//     else
//     {
//         UNIMPLEMENTED;
//         Status = STATUS_SUCCESS;
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// UsbAudioPinDataIntersect(
//       PVOID        Context,
//       PIRP         Irp,
//       PKSP_PIN     Pin,
//       PKSDATARANGE DataRange,
//       PKSDATARANGE MatchingDataRange,
//       ULONG        DataBufferSize,
//      PVOID        Data,
//      PULONG       DataSize)
// {
//     PKSFILTER Filter;
//     PKSPIN_DESCRIPTOR_EX PinDescriptor;
//     PKSDATAFORMAT_WAVEFORMATEX DataFormat;
//     PKSDATARANGE_AUDIO DataRangeAudio;
//
//     /* get filter from irp*/
//     Filter = KsGetFilterFromIrp(Irp);
//     if (!Filter)
//     {
//         /* no match*/
//         return STATUS_NO_MATCH;
//     }
//
//     /* get pin descriptor */
//     PinDescriptor = (PKSPIN_DESCRIPTOR_EX)&Filter->Descriptor->PinDescriptors[Pin->PinId];
//
//     *DataSize = sizeof(KSDATAFORMAT_WAVEFORMATEX);
//     if (DataBufferSize == 0)
//     {
//         /* buffer too small */
//         return STATUS_BUFFER_OVERFLOW;
//     }
//
//     /* sanity checks*/
//     ASSERT(PinDescriptor->PinDescriptor.DataRangesCount >= 0);
//     ASSERT(PinDescriptor->PinDescriptor.DataRanges[0]->FormatSize == sizeof(KSDATARANGE_AUDIO));
//
//     DataRangeAudio = (PKSDATARANGE_AUDIO)PinDescriptor->PinDescriptor.DataRanges[0];
//
//     DataFormat = Data;
//     DataFormat->WaveFormatEx.wFormatTag = WAVE_FORMAT_PCM;
//     DataFormat->WaveFormatEx.nChannels = DataRangeAudio->MaximumChannels;
//     DataFormat->WaveFormatEx.nSamplesPerSec = DataRangeAudio->MaximumSampleFrequency;
//     DataFormat->WaveFormatEx.nAvgBytesPerSec = DataRangeAudio->MaximumSampleFrequency * (DataRangeAudio->MaximumBitsPerSample / 8) * DataRangeAudio->MaximumChannels;
//     DataFormat->WaveFormatEx.nBlockAlign = (DataRangeAudio->MaximumBitsPerSample / 8) * DataRangeAudio->MaximumChannels;
//     DataFormat->WaveFormatEx.wBitsPerSample = DataRangeAudio->MaximumBitsPerSample;
//     DataFormat->WaveFormatEx.cbSize = 0;
//
//     DataFormat->DataFormat.FormatSize = sizeof(KSDATAFORMAT) + sizeof(WAVEFORMATEX);
//     DataFormat->DataFormat.Flags = 0;
//     DataFormat->DataFormat.Reserved = 0;
//     DataFormat->DataFormat.MajorFormat = KSDATAFORMAT_TYPE_AUDIO;
//     DataFormat->DataFormat.SubFormat = KSDATAFORMAT_SUBTYPE_PCM;
//     DataFormat->DataFormat.Specifier = KSDATAFORMAT_SPECIFIER_WAVEFORMATEX;
//     DataFormat->DataFormat.SampleSize = (DataRangeAudio->MaximumBitsPerSample / 8) * DataRangeAudio->MaximumChannels;
//
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
