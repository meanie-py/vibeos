use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{KeInitializeSpinLock, KeInsertQueueDpc, KeStallExecutionProcessor, LARGE_INTEGER, PDEVICE_OBJECT};
use core::ffi::c_void;

extern "system" {
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, );
    fn USBPORT_CancelSplitTransfer(Transfer: PUSBPORT_TRANSFER);
    pub fn USBPORT_FreeCommonBuffer( FdoDevice: PDEVICE_OBJECT, HeaderBuffer: PUSBPORT_COMMON_BUFFER_HEADER, );
    fn USBPORT_AllocateCommonBuffer( FdoDevice: PDEVICE_OBJECT, BufferLength: SIZE_T, ) -> PUSBPORT_COMMON_BUFFER_HEADER;
    fn USBPORT_Wait(MiniPortExtension: PVOID, Milliseconds: ULONG,) -> NTSTATUS;
    fn USBPORT_FreeBandwidthUSB2(FdoDevice: PDEVICE_OBJECT, Endpoint: PUSBPORT_ENDPOINT);
    fn USBPORT_SignalWorkerThread(FdoDevice: PDEVICE_OBJECT);
    fn USBPORT_UpdateAllocatedBwTt(TtExtension: PUSB2_TT_EXTENSION);
    fn USBPORT_AllocateBandwidthUSB2( FdoDevice: PDEVICE_OBJECT, Endpoint: PUSBPORT_ENDPOINT, ) -> BOOLEAN;
    pub fn USBPORT_USBDStatusToNtStatus(Urb: PURB, USBDStatus: USBD_STATUS) -> NTSTATUS;
    fn USBPORT_FlushCancelList(Endpoint: PUSBPORT_ENDPOINT);
    fn USBPORT_FlushAbortList(Endpoint: PUSBPORT_ENDPOINT);
    fn USBPORT_RootHubEndpointWorker(Endpoint: PUSBPORT_ENDPOINT);
}
extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();

pub const NDEBUG_USBPORT_CORE: () = ();

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CalculateUsbBandwidth(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
) -> ULONG {
    let EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES;
    let mut Bandwidth: ULONG;
    let Overhead: ULONG;

    EndpointProperties = core::ptr::addr_of_mut!((*Endpoint).EndpointProperties)
        as PUSBPORT_ENDPOINT_PROPERTIES;

    match (*EndpointProperties).TransferType {
        USBPORT_TRANSFER_TYPE_ISOCHRONOUS => {
            Overhead = USB2_FS_ISOCHRONOUS_OVERHEAD as ULONG;
        }
        USBPORT_TRANSFER_TYPE_INTERRUPT => {
            Overhead = USB2_FS_INTERRUPT_OVERHEAD as ULONG;
        }
        _ => {
            Overhead = 0 as ULONG;
        }
    }

    if Overhead == 0 {
        Bandwidth = 0 as ULONG;
    } else {
        Bandwidth = ((*EndpointProperties).TotalMaxPacketSize as ULONG)
            .wrapping_add(Overhead)
            .wrapping_mul(USB2_BIT_STUFFING_OVERHEAD as ULONG);
    }

    if (*EndpointProperties).DeviceSpeed as u32 == UsbLowSpeed as u32 {
        Bandwidth = Bandwidth.wrapping_mul(8 as ULONG);
    }

    return Bandwidth;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AllocateBandwidth(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
) -> BOOLEAN {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES;
    let mut TransferType: ULONG;
    let mut TotalBusBandwidth: ULONG;
    let mut EndpointBandwidth: ULONG;
    let mut MinBandwidth: ULONG;
    let mut Bandwidth: PULONG;
    let mut MaxBandwidth: ULONG = 0;
    let mut ix: ULONG;
    let mut Offset: ULONG;
    let mut ScheduleOffset: LONG = -1;
    let mut Period: ULONG;
    let mut Factor: ULONG;
    let mut Bit: UCHAR;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    EndpointProperties = core::ptr::addr_of_mut!((*Endpoint).EndpointProperties) as PUSBPORT_ENDPOINT_PROPERTIES;
    TransferType = (*EndpointProperties).TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_BULK
        || TransferType == USBPORT_TRANSFER_TYPE_CONTROL
        || (*Endpoint).Flags as ULONG & ENDPOINT_FLAG_ROOTHUB_EP0 as ULONG != 0
    {
        (*EndpointProperties).ScheduleOffset = 0;
        return 1;
    }

    TotalBusBandwidth = (*FdoExtension).TotalBusBandwidth;
    EndpointBandwidth = (*EndpointProperties).UsbBandwidth;

    Period = (*EndpointProperties).Period as ULONG;
    // ASSERT(Period != 0)
    Factor = (USB2_FRAMES as ULONG) / Period;

    Offset = 0;
    while Offset < Period {
        MinBandwidth = TotalBusBandwidth;
        Bandwidth = (core::ptr::addr_of_mut!((*FdoExtension).Bandwidth) as *mut ULONG)
            .offset((Offset.wrapping_mul(Factor)) as isize) as PULONG;

        ix = 1;
        while *Bandwidth >= EndpointBandwidth {
            MinBandwidth = if MinBandwidth < *Bandwidth { MinBandwidth } else { *Bandwidth };

            Bandwidth = Bandwidth.offset(1);

            if Factor <= ix {
                if MinBandwidth > MaxBandwidth {
                    MaxBandwidth = MinBandwidth;
                    ScheduleOffset = Offset as LONG;
                }

                break;
            }

            ix = ix.wrapping_add(1);
        }

        Offset = Offset.wrapping_add(1);
    }

    if ScheduleOffset != -1 {
        (*EndpointProperties).ScheduleOffset = ScheduleOffset as ULONG;

        Bandwidth = (core::ptr::addr_of_mut!((*FdoExtension).Bandwidth) as *mut ULONG)
            .offset(((ScheduleOffset as ULONG).wrapping_mul(Factor)) as isize) as PULONG;

        Factor = (USB2_FRAMES as ULONG) / Period;
        while Factor != 0 {
            (*FdoExtension).Bandwidth[(ScheduleOffset as ULONG).wrapping_mul(Factor) as usize] =
                ((*FdoExtension).Bandwidth[(ScheduleOffset as ULONG).wrapping_mul(Factor) as usize])
                    .wrapping_sub(EndpointBandwidth);
            Factor = Factor.wrapping_sub(1);
        }

        if TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT {
            Bit = 0x80;
            while Bit != 0 {
                if (Period & Bit as ULONG) != 0 {
                    Period = Bit as ULONG;
                    break;
                }
                Bit = (Bit as i32 >> 1) as UCHAR;
            }
        } else {
        }
    }

    return (ScheduleOffset != -1) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FreeBandwidth(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
) {
    const USBPORT_TRANSFER_TYPE_CONTROL: ULONG = 0x00000001; // FIXED 2026-07-05: translation invented 0x0; verified vs usbport.h/usbmport.h
    const USBPORT_TRANSFER_TYPE_ISOCHRONOUS: ULONG = 0x00000000; // FIXED 2026-07-05: translation invented 0x1; verified vs usbport.h/usbmport.h
    const USBPORT_TRANSFER_TYPE_BULK: ULONG = 2;
    const USBPORT_TRANSFER_TYPE_INTERRUPT: ULONG = 3;
    const ENDPOINT_FLAG_ROOTHUB_EP0: ULONG = 0x00000002; // FIXED 2026-07-05: translation invented 0x1; verified vs usbport.h/usbmport.h
    const USB2_FRAMES: ULONG = 0x00000020; // FIXED 2026-07-05: translation invented 0x8; verified vs usbport.h/usbmport.h

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES;
    let mut TransferType: ULONG;
    let mut Offset: ULONG;
    let mut EndpointBandwidth: ULONG;
    let mut Period: ULONG;
    let mut Factor: ULONG;
    let mut Bit: UCHAR;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    EndpointProperties =
        core::ptr::addr_of_mut!((*Endpoint).EndpointProperties) as PUSBPORT_ENDPOINT_PROPERTIES;
    TransferType = (*EndpointProperties).TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_BULK
        || TransferType == USBPORT_TRANSFER_TYPE_CONTROL
        || ((*Endpoint).Flags as ULONG & ENDPOINT_FLAG_ROOTHUB_EP0) != 0
    {
        return;
    }

    Offset = (*Endpoint).EndpointProperties.ScheduleOffset;
    EndpointBandwidth = (*Endpoint).EndpointProperties.UsbBandwidth;

    Period = (*Endpoint).EndpointProperties.Period as ULONG;
    if !(Period != 0) {
        RtlAssert(
            b"Period != 0\0".as_ptr() as *const i8 as PVOID,
            b"endpoint.c\0".as_ptr() as *const i8 as PVOID,
            203 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    Factor = (USB2_FRAMES as ULONG).wrapping_div(Period);
    while Factor != 0 {
        (*FdoExtension).Bandwidth[Offset.wrapping_mul(Factor) as usize] =
            ((*FdoExtension).Bandwidth[Offset.wrapping_mul(Factor) as usize])
                .wrapping_add(EndpointBandwidth);
        Factor = Factor.wrapping_sub(1);
    }

    if TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT {
        Bit = 0x80 as UCHAR;
        while Bit as i32 != 0 {
            if (Period & Bit as ULONG) != 0 {
                Period = Bit as ULONG;
                break;
            } else {
                Bit = (Bit as i32 >> 1) as UCHAR;
            }
        }

        if !(Period != 0) {
            RtlAssert(
                b"Period != 0\0".as_ptr() as *const i8 as PVOID,
                b"endpoint.c\0".as_ptr() as *const i8 as PVOID,
                221 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }

    if DbgPrint(
        b"(%s:%d) USBPORT_FreeBandwidth: FIXME USBPORT_UpdateAllocatedBw\n\0".as_ptr() as PCSTR,
        b"endpoint.c\0".as_ptr() as *const i8,
        230 as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"endpoint.c\0".as_ptr() as *const i8,
            230 as i32,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_NormalizeHsInterval(mut Interval: UCHAR) -> UCHAR {
    let mut interval: UCHAR;

    interval = Interval;

    if Interval != 0 {
        interval = (Interval as i32 - 1) as UCHAR;
    }

    if interval as i32 > 5 {
        interval = 5 as UCHAR;
    }

    return ((1 as i32) << interval as i32) as UCHAR;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_EndpointHasQueuedTransfers(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
    TransferCount: PULONG,
) -> BOOLEAN {
    let mut Entry: PLIST_ENTRY;
    let mut Transfer: PUSBPORT_TRANSFER = core::ptr::null_mut();
    let mut Result: BOOLEAN = 0;

    DPRINT_CORE!(b"USBPORT_EndpointHasQueuedTransfers: ... \n\0".as_ptr() as *const i8);

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
    );

    if !IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).PendingTransferList)) {
        Result = 1;
    }

    if !IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).TransferList)) {
        Result = 1;

        if !TransferCount.is_null() {
            *TransferCount = 0;

            Entry = (*Endpoint).TransferList.Flink;
            while !Entry.is_null()
                && Entry != core::ptr::addr_of_mut!((*Endpoint).TransferList)
            {
                Transfer = (Entry as usize)
                    .wrapping_sub(core::mem::offset_of!(USBPORT_TRANSFER, TransferLink))
                    as *mut USBPORT_TRANSFER;

                if (*Transfer).Flags as u32 & TRANSFER_FLAG_SUBMITED as u32 != 0 {
                    *TransferCount = (*TransferCount).wrapping_add(1);
                }

                Entry = (*Transfer).TransferLink.Flink;
            }
        }
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        (*Endpoint).EndpointOldIrql,
    );

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_NukeAllEndpoints(FdoDevice: PDEVICE_OBJECT) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut EndpointList: PLIST_ENTRY;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut OldIrql: KIRQL = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );

    EndpointList = (*FdoExtension).EndpointList.Flink;

    while !EndpointList.is_null()
        && EndpointList != core::ptr::addr_of_mut!((*FdoExtension).EndpointList)
    {
        Endpoint = (EndpointList as usize)
            .wrapping_sub(core::mem::offset_of!(USBPORT_ENDPOINT, EndpointLink))
            as *mut USBPORT_ENDPOINT;

        if (*Endpoint).Flags & ENDPOINT_FLAG_ROOTHUB_EP0 == 0 {
            (*Endpoint).Flags |= ENDPOINT_FLAG_NUKE;
        }

        EndpointList = (*Endpoint).EndpointLink.Flink;
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
        OldIrql,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_GetEndpointState(Endpoint: PUSBPORT_ENDPOINT) -> ULONG {
    let mut State: ULONG;

    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize));

    if (*Endpoint).StateLast != (*Endpoint).StateNext {
        State = USBPORT_ENDPOINT_UNKNOWN as ULONG;
    } else {
        State = (*Endpoint).StateLast;
    }

    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize));

    if State != USBPORT_ENDPOINT_ACTIVE as ULONG {
        // DPRINT("USBPORT_GetEndpointState: Endpoint - %p, State - %x\n", Endpoint, State);
    }

    return State;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_SetEndpointState(
    Endpoint: PUSBPORT_ENDPOINT,
    State: ULONG,
) {
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut OldIrql: KIRQL = 0;

    FdoDevice = (*Endpoint).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize),
        core::ptr::addr_of_mut!((*Endpoint).EndpointStateOldIrql),
    );

    if (*Endpoint).Flags as ULONG & ENDPOINT_FLAG_ROOTHUB_EP0 as ULONG == 0 {
        if (*Endpoint).Flags as ULONG & ENDPOINT_FLAG_NUKE as ULONG != 0 {
            (*Endpoint).StateLast = State;
            (*Endpoint).StateNext = State;

            KeReleaseSpinLock(
                (core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize),
                (*Endpoint).EndpointStateOldIrql,
            );

            USBPORT_InvalidateEndpointHandler(
                FdoDevice,
                Endpoint,
                INVALIDATE_ENDPOINT_WORKER_THREAD as ULONG,
            );
            return;
        }

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize),
            (*Endpoint).EndpointStateOldIrql,
        );

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );
        ((*Packet).SetEndpointState.expect("non-null function pointer"))(
            (*FdoExtension).MiniPortExt,
            Endpoint.offset(1) as PVOID,
            State,
        );
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            OldIrql,
        );

        (*Endpoint).StateNext = State;

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );
        (*Endpoint).FrameNumber =
            ((*Packet).Get32BitFrameNumber.expect("non-null function pointer"))(
                (*FdoExtension).MiniPortExt,
            );
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            OldIrql,
        );

        let mut StateChangeOldIrql: KIRQL = 0;
        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EpStateChangeSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(StateChangeOldIrql),
        );

        if (*Endpoint).StateChangeLink.Flink.is_null()
            && (*Endpoint).StateChangeLink.Blink.is_null()
        {
            InsertTailList(
                core::ptr::addr_of_mut!((*FdoExtension).EpStateChangeList),
                core::ptr::addr_of_mut!((*Endpoint).StateChangeLink),
            );
        }

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EpStateChangeSpinLock) as *mut usize),
            StateChangeOldIrql,
        );

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );
        ((*Packet).InterruptNextSOF.expect("non-null function pointer"))(
            (*FdoExtension).MiniPortExt,
        );
        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
            OldIrql,
        );
    } else {
        (*Endpoint).StateLast = State;
        (*Endpoint).StateNext = State;

        if State == USBPORT_ENDPOINT_REMOVE as ULONG {
            KeReleaseSpinLock(
                (core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize),
                (*Endpoint).EndpointStateOldIrql,
            );

            USBPORT_InvalidateEndpointHandler(
                FdoDevice,
                Endpoint,
                INVALIDATE_ENDPOINT_WORKER_THREAD as ULONG,
            );
            return;
        }

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize),
            (*Endpoint).EndpointStateOldIrql,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_AddPipeHandle(
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
    PipeHandle: PUSBPORT_PIPE_HANDLE,
) {
    InsertTailList(
        core::ptr::addr_of_mut!((*DeviceHandle).PipeHandleList),
        core::ptr::addr_of_mut!((*PipeHandle).PipeLink),
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_RemovePipeHandle(
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
    PipeHandle: PUSBPORT_PIPE_HANDLE,
) {
    RemoveEntryList(core::ptr::addr_of_mut!((*PipeHandle).PipeLink));

    (*PipeHandle).PipeLink.Flink = core::ptr::null_mut();
    (*PipeHandle).PipeLink.Blink = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_ValidatePipeHandle(
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
    PipeHandle: PUSBPORT_PIPE_HANDLE,
) -> BOOLEAN {
    let mut HandleList: PLIST_ENTRY;
    let mut CurrentHandle: PUSBPORT_PIPE_HANDLE;

    HandleList = (*DeviceHandle).PipeHandleList.Flink;

    while HandleList != core::ptr::addr_of_mut!((*DeviceHandle).PipeHandleList) {
        CurrentHandle = (HandleList as usize).wrapping_sub(
            core::mem::offset_of!(USBPORT_PIPE_HANDLE, PipeLink),
        ) as *mut USBPORT_PIPE_HANDLE;

        HandleList = (*HandleList).Flink;

        if CurrentHandle == PipeHandle {
            return 1;
        }
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DeleteEndpoint(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
) -> BOOLEAN {
    const USB_PORT_TAG: u32 = 0x70627375;

    let FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Result: BOOLEAN;
    let mut OldIrql: KIRQL = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    let mut StateChangeOldIrql: KIRQL = 0;
    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).EpStateChangeSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(StateChangeOldIrql),
    );

    if !(*Endpoint).StateChangeLink.Flink.is_null() && !(*Endpoint).StateChangeLink.Blink.is_null()
    {
        RemoveEntryList(core::ptr::addr_of_mut!((*Endpoint).StateChangeLink));
    }

    (*Endpoint).StateChangeLink.Flink = core::ptr::null_mut();
    (*Endpoint).StateChangeLink.Blink = core::ptr::null_mut();

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).EpStateChangeSpinLock) as *mut usize),
        StateChangeOldIrql,
    );

    if (!(*Endpoint).WorkerLink.Flink.is_null() && !(*Endpoint).WorkerLink.Blink.is_null())
        || (*Endpoint).LockCounter != -1
    {
        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );

        ExInterlockedInsertTailList(
            core::ptr::addr_of_mut!((*FdoExtension).EndpointClosedList),
            core::ptr::addr_of_mut!((*Endpoint).CloseLink),
            core::ptr::addr_of_mut!((*FdoExtension).EndpointClosedSpinLock),
        );

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
            OldIrql,
        );

        Result = 0;
    } else {
        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );

        RemoveEntryList(core::ptr::addr_of_mut!((*Endpoint).EndpointLink));
        (*Endpoint).EndpointLink.Flink = core::ptr::null_mut();
        (*Endpoint).EndpointLink.Blink = core::ptr::null_mut();

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
            OldIrql,
        );

        MiniportCloseEndpoint(FdoDevice, Endpoint);

        if !(*Endpoint).HeaderBuffer.is_null() {
            USBPORT_FreeCommonBuffer(FdoDevice, (((*Endpoint).HeaderBuffer as *mut c_void)) as _);
        }

        ExFreePoolWithTag(Endpoint as *mut core::ffi::c_void, USB_PORT_TAG);

        Result = 1;
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn MiniportCloseEndpoint(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let IsDoDisablePeriodic: BOOLEAN;
    let TransferType: ULONG;
    let mut OldIrql: KIRQL = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );

    if (*Endpoint).Flags as u32 & ENDPOINT_FLAG_OPENED as u32 != 0 {
        TransferType = (*Endpoint).EndpointProperties.TransferType;

        if TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as u32
            || TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as u32
        {
            (*FdoExtension).PeriodicEndpoints =
                (*FdoExtension).PeriodicEndpoints.wrapping_sub(1);
        }

        IsDoDisablePeriodic = ((*FdoExtension).PeriodicEndpoints == 0) as BOOLEAN;

        ((*Packet).CloseEndpoint.expect("non-null function pointer"))(
            (*FdoExtension).MiniPortExt,
            Endpoint.offset(1) as PVOID,
            IsDoDisablePeriodic,
        );

        (*Endpoint).Flags &= !ENDPOINT_FLAG_OPENED as u32;
        (*Endpoint).Flags |= ENDPOINT_FLAG_CLOSED as u32;
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
        OldIrql,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_ClosePipe(
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
    FdoDevice: PDEVICE_OBJECT,
    PipeHandle: PUSBPORT_PIPE_HANDLE,
) {

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let Endpoint: PUSBPORT_ENDPOINT;
    let Packet: PUSBPORT_REGISTRATION_PACKET;
    let TtExtension: PUSB2_TT_EXTENSION;
    let mut ix: ULONG;
    let mut IsReady: BOOLEAN;
    let mut OldIrql: KIRQL = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    if (*PipeHandle).Flags & PIPE_HANDLE_FLAG_CLOSED != 0 {
        return;
    }

    USBPORT_RemovePipeHandle(DeviceHandle, PipeHandle);

    (*PipeHandle).Flags |= PIPE_HANDLE_FLAG_CLOSED;

    if (*PipeHandle).Flags & PIPE_HANDLE_FLAG_NULL_PACKET_SIZE != 0 {
        (*PipeHandle).Flags &= !PIPE_HANDLE_FLAG_NULL_PACKET_SIZE;
        return;
    }

    Endpoint = (*PipeHandle).Endpoint as PUSBPORT_ENDPOINT;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );

    if (*Endpoint).Flags & ENDPOINT_FLAG_ROOTHUB_EP0 != 0
        && (*Endpoint).EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT
    {
        PdoExtension = (*(*FdoExtension).RootHubPdo).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;
        (*PdoExtension).Endpoint = core::ptr::null_mut();
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
        OldIrql,
    );

    loop {
        IsReady = 1;

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
        );

        if !IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).PendingTransferList)) {
            IsReady = 0;
        }

        if !IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).TransferList)) {
            IsReady = 0;
        }

        if !IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).CancelList)) {
            IsReady = 0;
        }

        if !IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).AbortList)) {
            IsReady = 0;
        }

        KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize));
        if (*Endpoint).StateLast != (*Endpoint).StateNext {
            IsReady = 0;
        }
        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize));

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            (*Endpoint).EndpointOldIrql,
        );

        if InterlockedIncrement(core::ptr::addr_of_mut!((*Endpoint).LockCounter)) != 0 {
            IsReady = 0;
        }
        InterlockedDecrement(core::ptr::addr_of_mut!((*Endpoint).LockCounter));

        if IsReady as i32 == 1 {
            break;
        }

        USBPORT_Wait(FdoDevice as PVOID, 1);
    }

    (*Endpoint).DeviceHandle = core::ptr::null_mut();
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    if (*Packet).MiniPortFlags & USB_MINIPORT_FLAGS_USB2 != 0 {
        USBPORT_FreeBandwidthUSB2(FdoDevice, Endpoint);

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).TtSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );

        TtExtension = (*Endpoint).TtExtension as PUSB2_TT_EXTENSION;

        if !TtExtension.is_null() {
            RemoveEntryList(core::ptr::addr_of_mut!((*Endpoint).TtLink));

            (*Endpoint).TtLink.Flink = core::ptr::null_mut();
            (*Endpoint).TtLink.Blink = core::ptr::null_mut();

            if (*TtExtension).Flags & USB2_TT_EXTENSION_FLAG_DELETED != 0 {
                if IsListEmpty(core::ptr::addr_of_mut!((*TtExtension).EndpointList)) {
                    USBPORT_UpdateAllocatedBwTt(TtExtension);

                    ix = 0;
                    while ix < USB2_FRAMES {
                        (*FdoExtension).Bandwidth[ix as usize] =
                            ((*FdoExtension).Bandwidth[ix as usize] as ULONG)
                                .wrapping_add((*TtExtension).MaxBandwidth as ULONG);
                        ix = ix.wrapping_add(1);
                    }

                    ExFreePoolWithTag(TtExtension as PVOID, USB_PORT_TAG);
                }
            }
        }

        KeReleaseSpinLock((core::ptr::addr_of_mut!((*FdoExtension).TtSpinLock) as *mut usize), OldIrql);
    } else {
        USBPORT_FreeBandwidth(FdoDevice, Endpoint);
    }

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
    );
    USBPORT_SetEndpointState(Endpoint, USBPORT_ENDPOINT_REMOVE);
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        (*Endpoint).EndpointOldIrql,
    );

    USBPORT_SignalWorkerThread(FdoDevice);
}

#[no_mangle]
pub unsafe extern "system" fn MiniportOpenEndpoint(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
) -> MPSTATUS {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut OldIrql: KIRQL = 0;
    let mut TransferType: ULONG;
    let mut MpStatus: MPSTATUS;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );

    (*Endpoint).Flags &= !(ENDPOINT_FLAG_CLOSED as ULONG);

    MpStatus = ((*Packet).OpenEndpoint.expect("non-null function pointer"))(
        (*FdoExtension).MiniPortExt,
        core::ptr::addr_of_mut!((*Endpoint).EndpointProperties),
        Endpoint.offset(1) as PVOID,
    );

    if MpStatus == 0 {
        TransferType = (*Endpoint).EndpointProperties.TransferType;

        if TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG
            || TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG
        {
            (*FdoExtension).PeriodicEndpoints =
                (*FdoExtension).PeriodicEndpoints.wrapping_add(1);
        }

        (*Endpoint).Flags |= ENDPOINT_FLAG_OPENED as ULONG;
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
        OldIrql,
    );
    return MpStatus;
}


/// A USB 2.0 miniport addresses a device by its USB address and, for a
/// split transaction, its TT hub. An xHCI controller addresses it by where it
/// sits: root port, route string (one nibble per hub below the root port,
/// nearest first) and, for a low/full-speed device below a high-speed hub,
/// that hub and the hub port leading to the device. usbxhci reads this from
/// fields every other miniport leaves alone:
///   Reserved3: route string (bits 0..19) | root port (bits 24..31)
///   Reserved6: TT port (bits 0..7) | TT hub's USB address (bits 8..15), 0 if none
unsafe fn USBPORT_SetXhciTopology(
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
) {
    const DEVICE_HANDLE_FLAG_ROOTHUB: ULONG = 0x00000002;
    let low_or_full = (*DeviceHandle).DeviceSpeed != UsbHighSpeed;
    let mut hub_ports = [0u32; 5];
    let mut depth = 0usize;
    let mut root_port = 0u32;
    let mut tt: u32 = 0;
    let mut dh = DeviceHandle;
    while !dh.is_null() && (*dh).Flags & DEVICE_HANDLE_FLAG_ROOTHUB == 0 {
        let hub = (*dh).HubDeviceHandle;
        let port = (*dh).PortNumber as u32;
        if hub.is_null() || (*hub).Flags & DEVICE_HANDLE_FLAG_ROOTHUB != 0 {
            root_port = port;
            break;
        }
        if low_or_full && tt == 0 && (*hub).DeviceSpeed == UsbHighSpeed {
            tt = (port & 0xFF) | (((*hub).DeviceAddress as u32 & 0xFF) << 8);
        }
        if depth < hub_ports.len() {
            hub_ports[depth] = port.min(15);
            depth += 1;
        }
        dh = hub;
    }
    let mut route = 0u32;
    for (tier, &port) in hub_ports[..depth].iter().rev().enumerate() {
        route |= port << (4 * tier);
    }
    (*EndpointProperties).Reserved3 = route | (root_port << 24);
    (*EndpointProperties).Reserved6 = tt;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_OpenPipe(
    FdoDevice: PDEVICE_OBJECT,
    DeviceHandle: PUSBPORT_DEVICE_HANDLE,
    PipeHandle: PUSBPORT_PIPE_HANDLE,
    UsbdStatus: PUSBD_STATUS,
) -> NTSTATUS {

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut PdoExtension: PUSBPORT_RHDEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut EndpointSize: SIZE_T;
    let mut Endpoint: PUSBPORT_ENDPOINT;
    let mut EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES;
    let mut EndpointDescriptor: PUSB_ENDPOINT_DESCRIPTOR;
    let mut Direction: UCHAR;
    let mut Interval: UCHAR;
    let mut Period: UCHAR;
    let mut EndpointRequirements: USBPORT_ENDPOINT_REQUIREMENTS =
        core::mem::zeroed::<USBPORT_ENDPOINT_REQUIREMENTS>();
    let mut HeaderBuffer: PUSBPORT_COMMON_BUFFER_HEADER;
    let mut MpStatus: MPSTATUS;
    let mut USBDStatus: USBD_STATUS = USBD_STATUS_SUCCESS;
    let mut Status: NTSTATUS = 0;
    let mut OldIrql: KIRQL = 0;
    let mut MaxPacketSize: USHORT;
    let mut AdditionalTransaction: USHORT;
    let mut IsAllocatedBandwidth: BOOLEAN = 0;
    let mut RetryCount: ULONG;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);

    EndpointSize = (core::mem::size_of::<USBPORT_ENDPOINT>() as SIZE_T)
        .wrapping_add((*Packet).MiniPortEndpointSize as SIZE_T);

    if (*Packet).MiniPortFlags as u32 & USB_MINIPORT_FLAGS_USB2 != 0 {
        EndpointSize =
            EndpointSize.wrapping_add(core::mem::size_of::<USB2_TT_ENDPOINT>() as SIZE_T);
    }

    if (*PipeHandle).EndpointDescriptor.wMaxPacketSize == 0 {
        USBPORT_AddPipeHandle(DeviceHandle, PipeHandle);

        (*PipeHandle).Flags = ((*PipeHandle).Flags & !PIPE_HANDLE_FLAG_CLOSED)
            | PIPE_HANDLE_FLAG_NULL_PACKET_SIZE;

        (*PipeHandle).Endpoint = -1isize as PUSBPORT_ENDPOINT;

        return STATUS_SUCCESS;
    }

    Endpoint =
        ExAllocatePoolWithTag(NonPagedPool, EndpointSize, USB_PORT_TAG) as PUSBPORT_ENDPOINT;

    if Endpoint.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
        return Status;
    }

    RtlZeroMemory(Endpoint as PVOID, EndpointSize);

    (*Endpoint).FdoDevice = FdoDevice;
    (*Endpoint).DeviceHandle = DeviceHandle;
    (*Endpoint).LockCounter = -1;

    (*Endpoint).TtExtension = (*DeviceHandle).TtExtension;

    if !(*DeviceHandle).TtExtension.is_null() {
        ExInterlockedInsertTailList(
            core::ptr::addr_of_mut!((*(*DeviceHandle).TtExtension).EndpointList),
            core::ptr::addr_of_mut!((*Endpoint).TtLink),
            core::ptr::addr_of_mut!((*FdoExtension).TtSpinLock),
        );
    }

    if (*Packet).MiniPortFlags as u32 & USB_MINIPORT_FLAGS_USB2 != 0 {
        (*Endpoint).TtEndpoint = (Endpoint as usize)
            .wrapping_add(core::mem::size_of::<USBPORT_ENDPOINT>())
            .wrapping_add((*Packet).MiniPortEndpointSize as usize)
            as PUSB2_TT_ENDPOINT;
    } else {
        (*Endpoint).TtEndpoint = core::ptr::null_mut();
    }

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock));

    InitializeListHead(core::ptr::addr_of_mut!((*Endpoint).PendingTransferList));
    InitializeListHead(core::ptr::addr_of_mut!((*Endpoint).TransferList));
    InitializeListHead(core::ptr::addr_of_mut!((*Endpoint).CancelList));
    InitializeListHead(core::ptr::addr_of_mut!((*Endpoint).AbortList));

    EndpointProperties = core::ptr::addr_of_mut!((*Endpoint).EndpointProperties);
    EndpointDescriptor = core::ptr::addr_of_mut!((*PipeHandle).EndpointDescriptor);

    MaxPacketSize = ((*EndpointDescriptor).wMaxPacketSize as i32 & 0x7FF) as USHORT;
    AdditionalTransaction =
        (((*EndpointDescriptor).wMaxPacketSize as i32 >> 11) & 3) as USHORT;

    (*EndpointProperties).DeviceAddress = (*DeviceHandle).DeviceAddress;
    (*EndpointProperties).DeviceSpeed = (*DeviceHandle).DeviceSpeed;
    (*EndpointProperties).Period = 0;
    (*EndpointProperties).EndpointAddress = (*EndpointDescriptor).bEndpointAddress as USHORT;
    (*EndpointProperties).TransactionPerMicroframe = (AdditionalTransaction as i32 + 1) as UCHAR;
    (*EndpointProperties).MaxPacketSize = MaxPacketSize as ULONG;
    (*EndpointProperties).TotalMaxPacketSize =
        (MaxPacketSize as i32 * (AdditionalTransaction as i32 + 1)) as USHORT;

    if !(*Endpoint).TtExtension.is_null() {
        (*EndpointProperties).HubAddr = (*(*Endpoint).TtExtension).DeviceAddress;
    } else {
        (*EndpointProperties).HubAddr = -1i32 as USHORT;
    }

    (*EndpointProperties).PortNumber = (*DeviceHandle).PortNumber;
    USBPORT_SetXhciTopology(DeviceHandle, EndpointProperties);

    match (*EndpointDescriptor).bmAttributes as u32 & (USB_ENDPOINT_TYPE_MASK as u32) {
        USB_ENDPOINT_TYPE_CONTROL => {
            (*EndpointProperties).TransferType = USBPORT_TRANSFER_TYPE_CONTROL;

            if (*EndpointProperties).EndpointAddress == 0 {
                (*EndpointProperties).MaxTransferSize = 0x1000;
            } else {
                (*EndpointProperties).MaxTransferSize = 0x10000;
            }
        }
        USB_ENDPOINT_TYPE_ISOCHRONOUS => {
            (*EndpointProperties).TransferType = USBPORT_TRANSFER_TYPE_ISOCHRONOUS;
            (*EndpointProperties).MaxTransferSize = 0x1000000;
        }
        USB_ENDPOINT_TYPE_BULK => {
            (*EndpointProperties).TransferType = USBPORT_TRANSFER_TYPE_BULK;
            (*EndpointProperties).MaxTransferSize = 0x10000;
        }
        USB_ENDPOINT_TYPE_INTERRUPT => {
            (*EndpointProperties).TransferType = USBPORT_TRANSFER_TYPE_INTERRUPT;
            (*EndpointProperties).MaxTransferSize = 0x400;
        }
        _ => {}
    }

    if (*EndpointProperties).TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT {
        if (*EndpointProperties).DeviceSpeed == UsbHighSpeed as USB_DEVICE_SPEED {
            Interval = USBPORT_NormalizeHsInterval((*EndpointDescriptor).bInterval);
        } else {
            Interval = (*EndpointDescriptor).bInterval;
        }

        (*EndpointProperties).Period = ENDPOINT_INTERRUPT_32ms as UCHAR;

        if Interval as i32 != 0 && (Interval as i32) < (USB2_FRAMES as i32) {
            if (*EndpointProperties).DeviceSpeed != UsbLowSpeed as USB_DEVICE_SPEED
                || Interval as i32 >= (ENDPOINT_INTERRUPT_8ms as i32)
            {
                if Interval as i32 & (ENDPOINT_INTERRUPT_32ms as i32) == 0 {
                    Period = (*EndpointProperties).Period;

                    loop {
                        Period = (Period as i32 >> 1) as UCHAR;
                        if !(Period as i32 & Interval as i32 == 0) {
                            break;
                        }
                    }

                    (*EndpointProperties).Period = Period;
                }
            } else {
                (*EndpointProperties).Period = ENDPOINT_INTERRUPT_8ms as UCHAR;
            }
        }
    }

    if (*EndpointProperties).TransferType == USB_ENDPOINT_TYPE_ISOCHRONOUS as ULONG {
        if (*EndpointProperties).DeviceSpeed == UsbHighSpeed as USB_DEVICE_SPEED {
            (*EndpointProperties).Period =
                USBPORT_NormalizeHsInterval((*EndpointDescriptor).bInterval);
        } else {
            (*EndpointProperties).Period = ENDPOINT_INTERRUPT_1ms as UCHAR;
        }
    }

    if (*DeviceHandle).Flags & DEVICE_HANDLE_FLAG_ROOTHUB != 0 {
        (*Endpoint).Flags |= ENDPOINT_FLAG_ROOTHUB_EP0;
    }

    if (*Packet).MiniPortFlags as u32 & USB_MINIPORT_FLAGS_USB2 != 0 {
        IsAllocatedBandwidth = USBPORT_AllocateBandwidthUSB2(FdoDevice, Endpoint);
    } else {
        (*EndpointProperties).UsbBandwidth =
            USBPORT_CalculateUsbBandwidth(FdoDevice, Endpoint);

        IsAllocatedBandwidth = USBPORT_AllocateBandwidth(FdoDevice, Endpoint);
    }

    if IsAllocatedBandwidth == 0 {
        Status = USBPORT_USBDStatusToNtStatus(core::ptr::null_mut(), USBD_STATUS_NO_BANDWIDTH);

        if !UsbdStatus.is_null() {
            *UsbdStatus = USBD_STATUS_NO_BANDWIDTH;
        }
    } else {
        Direction = (USB_ENDPOINT_DIRECTION_OUT((*EndpointDescriptor).bEndpointAddress) as u8);
        (*EndpointProperties).Direction = Direction as ULONG;

        if (*DeviceHandle).IsRootHub != 0 {
            (*Endpoint).EndpointWorker = 0;

            (*Endpoint).Flags |= ENDPOINT_FLAG_ROOTHUB_EP0;

            (*Endpoint).StateLast = USBPORT_ENDPOINT_ACTIVE;
            (*Endpoint).StateNext = USBPORT_ENDPOINT_ACTIVE;

            PdoExtension =
                (*(*FdoExtension).RootHubPdo).DeviceExtension as PUSBPORT_RHDEVICE_EXTENSION;

            if (*EndpointProperties).TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT {
                (*PdoExtension).Endpoint = Endpoint;
            }

            USBDStatus = USBD_STATUS_SUCCESS;
        } else {
            (*Endpoint).EndpointWorker = 1;

            KeAcquireSpinLock(
                (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                core::ptr::addr_of_mut!(OldIrql),
            );

            ((*Packet).QueryEndpointRequirements.expect("non-null function pointer"))(
                (*FdoExtension).MiniPortExt,
                core::ptr::addr_of_mut!((*Endpoint).EndpointProperties),
                core::ptr::addr_of_mut!(EndpointRequirements),
            );

            KeReleaseSpinLock(
                (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                OldIrql,
            );

            if (*EndpointProperties).TransferType == USBPORT_TRANSFER_TYPE_BULK
                || (*EndpointProperties).TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT
            {
                (*EndpointProperties).MaxTransferSize = EndpointRequirements.MaxTransferSize;
            }

            if EndpointRequirements.HeaderBufferSize != 0 {
                HeaderBuffer = USBPORT_AllocateCommonBuffer(
                    FdoDevice,
                    EndpointRequirements.HeaderBufferSize as SIZE_T,
                );
            } else {
                HeaderBuffer = core::ptr::null_mut();
            }

            if !HeaderBuffer.is_null() || EndpointRequirements.HeaderBufferSize == 0 {
                (*Endpoint).HeaderBuffer = HeaderBuffer;

                if !HeaderBuffer.is_null() {
                    (*EndpointProperties).BufferVA = (*HeaderBuffer).VirtualAddress;
                    (*EndpointProperties).BufferPA = (*HeaderBuffer).PhysicalAddress;
                    (*EndpointProperties).BufferLength = (*HeaderBuffer).BufferLength as ULONG;
                }

                MpStatus = MiniportOpenEndpoint(FdoDevice, Endpoint);

                (*Endpoint).Flags |= ENDPOINT_FLAG_DMA_TYPE;
                (*Endpoint).Flags |= ENDPOINT_FLAG_QUEUENE_EMPTY;

                if MpStatus == 0 {
                    let mut State: ULONG;

                    KeAcquireSpinLock(
                        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
                        core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
                    );

                    (*Endpoint).StateLast = USBPORT_ENDPOINT_PAUSED;
                    (*Endpoint).StateNext = USBPORT_ENDPOINT_PAUSED;

                    USBPORT_SetEndpointState(Endpoint, USBPORT_ENDPOINT_ACTIVE);

                    KeReleaseSpinLock(
                        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
                        (*Endpoint).EndpointOldIrql,
                    );

                    State = 0;
                    RetryCount = 0;
                    while RetryCount < 1000 {
                        KeAcquireSpinLock(
                            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
                            core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
                        );

                        State = USBPORT_GetEndpointState(Endpoint);

                        KeReleaseSpinLock(
                            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
                            (*Endpoint).EndpointOldIrql,
                        );

                        if State == USBPORT_ENDPOINT_ACTIVE {
                            break;
                        }

                        USBPORT_Wait(FdoDevice as PVOID, 1);

                        RetryCount = RetryCount.wrapping_add(1);
                    }
                    if State != USBPORT_ENDPOINT_ACTIVE {
                        USBDStatus = USBD_STATUS_TIMEOUT;
                    }
                }
            } else {
                MpStatus = MP_STATUS_NO_RESOURCES as MPSTATUS;
                (*Endpoint).HeaderBuffer = core::ptr::null_mut();
            }

            if MpStatus != 0 {
                USBDStatus = USBD_STATUS_INSUFFICIENT_RESOURCES;
            }
        }

        if !UsbdStatus.is_null() {
            *UsbdStatus = USBDStatus;
        }

        Status = USBPORT_USBDStatusToNtStatus(core::ptr::null_mut(), USBDStatus);

        if NT_SUCCESS(Status) {
            USBPORT_AddPipeHandle(DeviceHandle, PipeHandle);

            ExInterlockedInsertTailList(
                core::ptr::addr_of_mut!((*FdoExtension).EndpointList),
                core::ptr::addr_of_mut!((*Endpoint).EndpointLink),
                core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock),
            );

            (*PipeHandle).Endpoint = Endpoint;
            (*PipeHandle).Flags &= !PIPE_HANDLE_FLAG_CLOSED;

            return Status;
        }
    }

    if !Endpoint.is_null() {
        /* A timed out endpoint is still queued for the ISR DPC and opened in the miniport */
        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EpStateChangeSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );

        if !(*Endpoint).StateChangeLink.Flink.is_null() && !(*Endpoint).StateChangeLink.Blink.is_null()
        {
            RemoveEntryList(core::ptr::addr_of_mut!((*Endpoint).StateChangeLink));
        }

        (*Endpoint).StateChangeLink.Flink = core::ptr::null_mut();
        (*Endpoint).StateChangeLink.Blink = core::ptr::null_mut();

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EpStateChangeSpinLock) as *mut usize),
            OldIrql,
        );

        MiniportCloseEndpoint(FdoDevice, Endpoint);

        if !(*Endpoint).HeaderBuffer.is_null() {
            USBPORT_FreeCommonBuffer(FdoDevice, (((*Endpoint).HeaderBuffer as *mut c_void)) as _);
        }

        if IsAllocatedBandwidth != 0 {
            if (*Packet).MiniPortFlags as u32 & USB_MINIPORT_FLAGS_USB2 != 0 {
                USBPORT_FreeBandwidthUSB2(FdoDevice, Endpoint);
            } else {
                USBPORT_FreeBandwidth(FdoDevice, Endpoint);
            }
        }

        if !(*Endpoint).TtExtension.is_null() {
            KeAcquireSpinLock(
                (core::ptr::addr_of_mut!((*FdoExtension).TtSpinLock) as *mut usize),
                core::ptr::addr_of_mut!(OldIrql),
            );
            RemoveEntryList(core::ptr::addr_of_mut!((*Endpoint).TtLink));
            KeReleaseSpinLock((core::ptr::addr_of_mut!((*FdoExtension).TtSpinLock) as *mut usize), OldIrql);
        }

        ExFreePoolWithTag(Endpoint as PVOID, USB_PORT_TAG);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_ReopenPipe(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
) -> NTSTATUS {

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut HeaderBuffer: PUSBPORT_COMMON_BUFFER_HEADER;
    let mut EndpointRequirements: USBPORT_ENDPOINT_REQUIREMENTS =
        core::mem::zeroed();
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut MiniportOldIrql: KIRQL = 0;
    let mut Status: NTSTATUS;

    // DPRINT1("USBPORT_ReopenPipe ... \n");

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    loop {
        if InterlockedIncrement(
            core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
        ) == 0
        {
            break;
        }

        InterlockedDecrement(
            core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
        );
        USBPORT_Wait(FdoDevice as PVOID, 1 as ULONG);
    }

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
    );
    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize));

    ((*Packet).SetEndpointState.expect("non-null function pointer"))(
        (*FdoExtension).MiniPortExt,
        Endpoint.offset(1) as PVOID,
        USBPORT_ENDPOINT_REMOVE as ULONG,
    );

    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize));
    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        (*Endpoint).EndpointOldIrql,
    );

    KeStallExecutionProcessor(2000);
    Status = STATUS_SUCCESS;

    MiniportCloseEndpoint(FdoDevice, Endpoint);

    RtlZeroMemory(
        Endpoint.offset(1) as *mut core::ffi::c_void,
        (*Packet).MiniPortEndpointSize as usize,
    );

    if !(*Endpoint).HeaderBuffer.is_null() {
        USBPORT_FreeCommonBuffer(FdoDevice, (((*Endpoint).HeaderBuffer as *mut c_void)) as _);
        (*Endpoint).HeaderBuffer = core::ptr::null_mut();
    }

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(MiniportOldIrql),
    );

    ((*Packet)
        .QueryEndpointRequirements
        .expect("non-null function pointer"))(
        (*FdoExtension).MiniPortExt,
        core::ptr::addr_of_mut!((*Endpoint).EndpointProperties),
        core::ptr::addr_of_mut!(EndpointRequirements),
    );

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
        MiniportOldIrql,
    );

    if EndpointRequirements.HeaderBufferSize != 0 {
        HeaderBuffer = USBPORT_AllocateCommonBuffer(
            FdoDevice,
            EndpointRequirements.HeaderBufferSize as SIZE_T,
        );
    } else {
        HeaderBuffer = core::ptr::null_mut();
    }

    if !HeaderBuffer.is_null() || EndpointRequirements.HeaderBufferSize == 0 {
        (*Endpoint).HeaderBuffer = HeaderBuffer;
        Status = STATUS_SUCCESS;
    } else {
        (*Endpoint).HeaderBuffer = core::ptr::null_mut();
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    if !(*Endpoint).HeaderBuffer.is_null() && !HeaderBuffer.is_null() {
        (*Endpoint).EndpointProperties.BufferVA = (*HeaderBuffer).VirtualAddress;
        (*Endpoint).EndpointProperties.BufferPA = (*HeaderBuffer).PhysicalAddress;
        (*Endpoint).EndpointProperties.BufferLength = (*HeaderBuffer).BufferLength as ULONG;
    }

    if NT_SUCCESS(Status) {
        MiniportOpenEndpoint(FdoDevice, Endpoint);

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
        );
        KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize));

        if (*Endpoint).StateLast == USBPORT_ENDPOINT_ACTIVE as core::ffi::c_ulong {
            KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!(
                (*Endpoint).StateChangeSpinLock
            ) as *mut usize));
            KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!(
                (*FdoExtension).MiniportSpinLock
            ) as *mut usize));

            ((*Packet).SetEndpointState.expect("non-null function pointer"))(
                (*FdoExtension).MiniPortExt,
                Endpoint.offset(1) as PVOID,
                USBPORT_ENDPOINT_ACTIVE as ULONG,
            );

            KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!(
                (*FdoExtension).MiniportSpinLock
            ) as *mut usize));
        } else {
            KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!(
                (*Endpoint).StateChangeSpinLock
            ) as *mut usize));
        }

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
            (*Endpoint).EndpointOldIrql,
        );
    }

    InterlockedDecrement(
        core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
    );

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_FlushClosedEndpointList(FdoDevice: PDEVICE_OBJECT) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut OldIrql: KIRQL = 0;
    let ClosedList: PLIST_ENTRY;
    let mut Endpoint: PUSBPORT_ENDPOINT;

    // DPRINT_CORE!("USBPORT_FlushClosedEndpointList: ... \n");

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).EndpointClosedSpinLock) as *mut usize),
        core::ptr::addr_of_mut!(OldIrql),
    );
    ClosedList = core::ptr::addr_of_mut!((*FdoExtension).EndpointClosedList) as PLIST_ENTRY;

    while !IsListEmpty(ClosedList) {
        Endpoint = ((*ClosedList).Flink as usize)
            .wrapping_sub(core::mem::offset_of!(USBPORT_ENDPOINT, CloseLink))
            as *mut USBPORT_ENDPOINT as PUSBPORT_ENDPOINT;

        RemoveHeadList(ClosedList);
        (*Endpoint).CloseLink.Flink = core::ptr::null_mut();
        (*Endpoint).CloseLink.Blink = core::ptr::null_mut();

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EndpointClosedSpinLock) as *mut usize),
            OldIrql,
        );

        USBPORT_DeleteEndpoint(FdoDevice, Endpoint);

        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EndpointClosedSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*FdoExtension).EndpointClosedSpinLock) as *mut usize),
        OldIrql,
    );
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_InvalidateEndpointHandler(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
    mut Type: ULONG,
) {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut Entry: PLIST_ENTRY;
    let mut WorkerLink: PLIST_ENTRY;
    let mut endpoint: PUSBPORT_ENDPOINT;
    let mut OldIrql: KIRQL = 0;
    let mut IsAddEntry: BOOLEAN = 0;

    DPRINT_CORE!(
        b"USBPORT_InvalidateEndpointHandler: Endpoint - %p, Type - %x\n\0".as_ptr() as *const i8,
        Endpoint,
        Type,
    );

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    if !Endpoint.is_null() {
        WorkerLink = core::ptr::addr_of_mut!((*Endpoint).WorkerLink) as PLIST_ENTRY;
        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );
        DPRINT_CORE!(
            b"USBPORT_InvalidateEndpointHandler: KeAcquireSpinLock \n\0".as_ptr() as *const i8,
        );

        if ((*WorkerLink).Flink.is_null() || (*WorkerLink).Blink.is_null())
            && ((*Endpoint).Flags & ENDPOINT_FLAG_IDLE) == 0
            && USBPORT_GetEndpointState(Endpoint) != USBPORT_ENDPOINT_CLOSED
        {
            DPRINT_CORE!(
                b"USBPORT_InvalidateEndpointHandler: InsertTailList \n\0".as_ptr() as *const i8,
            );
            InsertTailList(core::ptr::addr_of_mut!((*FdoExtension).WorkerList), WorkerLink);
            IsAddEntry = 1;
        }

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
            OldIrql,
        );

        if ((*Endpoint).Flags & ENDPOINT_FLAG_ROOTHUB_EP0) != 0 {
            Type = INVALIDATE_ENDPOINT_WORKER_THREAD;
        }
    } else {
        KeAcquireSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
            core::ptr::addr_of_mut!(OldIrql),
        );

        Entry = (*FdoExtension).EndpointList.Flink as PLIST_ENTRY;
        while !Entry.is_null()
            && Entry != core::ptr::addr_of_mut!((*FdoExtension).EndpointList) as PLIST_ENTRY
        {
            endpoint = (Entry as usize
                - core::mem::offset_of!(USBPORT_ENDPOINT, EndpointLink))
                as PUSBPORT_ENDPOINT;

            if (*endpoint).WorkerLink.Flink.is_null() || (*endpoint).WorkerLink.Blink.is_null() {
                if ((*endpoint).Flags & ENDPOINT_FLAG_IDLE) == 0
                    && ((*endpoint).Flags & ENDPOINT_FLAG_ROOTHUB_EP0) == 0
                    && USBPORT_GetEndpointState(endpoint) != USBPORT_ENDPOINT_CLOSED
                {
                    DPRINT_CORE!(
                        b"USBPORT_InvalidateEndpointHandler: InsertTailList \n\0".as_ptr()
                            as *const i8,
                    );
                    InsertTailList(
                        core::ptr::addr_of_mut!((*FdoExtension).WorkerList),
                        core::ptr::addr_of_mut!((*endpoint).WorkerLink),
                    );
                    IsAddEntry = 1;
                }
            }

            Entry = (*Entry).Flink as PLIST_ENTRY;
        }

        KeReleaseSpinLock(
            (core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize),
            OldIrql,
        );
    }

    if ((*FdoExtension).Flags & USBPORT_FLAG_HC_SUSPEND) != 0 {
        Type = INVALIDATE_ENDPOINT_WORKER_THREAD;
    } else if IsAddEntry as i32 == 0 && Type == INVALIDATE_ENDPOINT_INT_NEXT_SOF {
        Type = INVALIDATE_ENDPOINT_ONLY;
    }

    match Type {
        INVALIDATE_ENDPOINT_WORKER_THREAD => {
            USBPORT_SignalWorkerThread(FdoDevice);
        }
        INVALIDATE_ENDPOINT_WORKER_DPC => {
            KeInsertQueueDpc(
                core::ptr::addr_of_mut!((*FdoExtension).WorkerRequestDpc),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }
        INVALIDATE_ENDPOINT_INT_NEXT_SOF => {
            KeAcquireSpinLock(
                (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                core::ptr::addr_of_mut!(OldIrql),
            );
            ((*Packet).InterruptNextSOF).expect("non-null function pointer")(
                (*FdoExtension).MiniPortExt,
            );
            KeReleaseSpinLock(
                (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                OldIrql,
            );
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DmaEndpointPaused(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
) -> ULONG {

    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut Entry: PLIST_ENTRY;
    let mut Transfer: PUSBPORT_TRANSFER;
    let mut Urb: PURB;
    let mut Frame: ULONG;
    let mut CurrentFrame: ULONG;
    let mut CompletedLen: ULONG = 0 as ULONG;
    let mut OldIrql: KIRQL = 0;

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet =
        &mut (*(*FdoExtension).MiniPortInterface).Packet as *mut _ as PUSBPORT_REGISTRATION_PACKET;

    Entry = (*Endpoint).TransferList.Flink as PLIST_ENTRY;

    if Entry == &mut (*Endpoint).TransferList as *mut _ as PLIST_ENTRY {
        return USBPORT_ENDPOINT_ACTIVE as ULONG;
    }

    while !Entry.is_null() && Entry != &mut (*Endpoint).TransferList as *mut _ as PLIST_ENTRY {
        Transfer = (Entry as usize)
            .wrapping_sub(core::mem::offset_of!(USBPORT_TRANSFER, TransferLink))
            as *mut USBPORT_TRANSFER as PUSBPORT_TRANSFER;

        if (*Transfer).Flags & (TRANSFER_FLAG_CANCELED | TRANSFER_FLAG_ABORTED) != 0 {
            if (*Transfer).Flags & TRANSFER_FLAG_ISO != 0
                && (*Transfer).Flags & TRANSFER_FLAG_SUBMITED != 0
                && (*Endpoint).Flags & ENDPOINT_FLAG_NUKE == 0
            {
                Urb = (*Transfer).Urb;

                Frame = (*Urb)
                    .__bindgen_anon_1.UrbIsochronousTransfer
                    .StartFrame
                    .wrapping_add((*Urb).__bindgen_anon_1.UrbIsochronousTransfer.NumberOfPackets);

                KeAcquireSpinLock(
                    core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize,
                    &mut OldIrql,
                );
                CurrentFrame = ((*Packet).Get32BitFrameNumber.expect("non-null function pointer"))(
                    (*FdoExtension).MiniPortExt,
                );
                KeReleaseSpinLock(core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize, OldIrql);

                if Frame.wrapping_add(1) > CurrentFrame {
                    return USBPORT_GetEndpointState(Endpoint);
                }
            }

            if (*Transfer).Flags & TRANSFER_FLAG_SUBMITED != 0
                && (*Endpoint).Flags & ENDPOINT_FLAG_NUKE == 0
            {
                KeAcquireSpinLock(
                    core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize,
                    &mut OldIrql,
                );

                ((*Packet).AbortTransfer.expect("non-null function pointer"))(
                    (*FdoExtension).MiniPortExt,
                    Endpoint.offset(1) as PVOID,
                    (*Transfer).MiniportTransfer,
                    &mut CompletedLen,
                );

                KeReleaseSpinLock(core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize, OldIrql);

                if (*Transfer).Flags & TRANSFER_FLAG_ISO != 0 {
                    if DbgPrint(
                        b"(%s:%d) USBPORT_DmaEndpointActive: FIXME call USBPORT_FlushIsoTransfer\n\0"
                            as *const u8 as PCSTR,
                        b"endpoint.c\0" as *const u8 as *const ::core::ffi::c_char,
                        1506 as ::core::ffi::c_int,
                    ) != 0
                    {
                        DbgPrint(
                            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                            b"endpoint.c\0" as *const u8 as *const ::core::ffi::c_char,
                            1506 as ::core::ffi::c_int,
                        );
                    }
                    if 0 == 0 {
                        RtlAssert(
                            b"FALSE\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                            b"endpoint.c\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
                            1507 as ULONG,
                            ::core::ptr::null_mut::<CHAR>(),
                        );
                    } else {
                    };
                } else {
                    (*Transfer).CompletedTransferLen = CompletedLen;
                }
            }

            RemoveEntryList(&mut (*Transfer).TransferLink);
            Entry = (*Transfer).TransferLink.Flink as PLIST_ENTRY;

            if (*Transfer).Flags & TRANSFER_FLAG_SPLITED != 0 {
                USBPORT_CancelSplitTransfer(Transfer);
            } else {
                InsertTailList(
                    &mut (*Endpoint).CancelList,
                    &mut (*Transfer).TransferLink,
                );
            }
        } else {
            Entry = (*Transfer).TransferLink.Flink as PLIST_ENTRY;
        }
    }

    return USBPORT_ENDPOINT_ACTIVE as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DmaEndpointActive(
    FdoDevice: PDEVICE_OBJECT,
    Endpoint: PUSBPORT_ENDPOINT,
) -> ULONG {
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Packet: PUSBPORT_REGISTRATION_PACKET = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut Transfer: PUSBPORT_TRANSFER = core::ptr::null_mut();
    let mut TimeOut: LARGE_INTEGER = core::mem::zeroed();
    let mut MpStatus: MPSTATUS;
    let mut OldIrql: KIRQL = 0;

    DPRINT_CORE!(b"USBPORT_DmaEndpointActive \n\0".as_ptr() as *const i8);

    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;

    Entry = (*Endpoint).TransferList.Flink;

    while !Entry.is_null() && Entry != core::ptr::addr_of_mut!((*Endpoint).TransferList) {
        Transfer = (Entry as usize)
            .wrapping_sub(core::mem::offset_of!(USBPORT_TRANSFER, TransferLink))
            as *mut USBPORT_TRANSFER;

        if ((*Transfer).Flags & TRANSFER_FLAG_SUBMITED) == 0
            && ((*Endpoint).Flags & ENDPOINT_FLAG_NUKE) == 0
        {
            KeAcquireSpinLock(
                (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                core::ptr::addr_of_mut!(OldIrql),
            );

            Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
                as PUSBPORT_REGISTRATION_PACKET;

            if ((*Transfer).Flags & TRANSFER_FLAG_ISO) != 0 {
                DbgPrint(b"USBPORT_DmaEndpointActive: FIXME call SubmitIsoTransfer\n\0".as_ptr() as *const i8);

                MpStatus = ((*Packet).SubmitIsoTransfer).expect("non-null function pointer")(
                    (*FdoExtension).MiniPortExt,
                    Endpoint.offset(1) as PVOID,
                    core::ptr::addr_of_mut!((*Transfer).TransferParameters),
                    (*Transfer).MiniportTransfer,
                    core::ptr::null_mut(),
                );
            } else {
                MpStatus = ((*Packet).SubmitTransfer).expect("non-null function pointer")(
                    (*FdoExtension).MiniPortExt,
                    Endpoint.offset(1) as PVOID,
                    core::ptr::addr_of_mut!((*Transfer).TransferParameters),
                    (*Transfer).MiniportTransfer,
                    core::ptr::addr_of_mut!((*Transfer).SgList),
                );
            }

            KeReleaseSpinLock(
                (core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize),
                OldIrql,
            );

            if MpStatus != 0 {
                if MpStatus != MP_STATUS_FAILURE && ((*Transfer).Flags & TRANSFER_FLAG_ISO) != 0 {
                    DbgPrint(
                        b"USBPORT_DmaEndpointActive: FIXME call USBPORT_ErrorCompleteIsoTransfer\n\0".as_ptr()
                            as *const i8,
                    );
                    ASSERT(0);
                }

                return USBPORT_ENDPOINT_ACTIVE;
            }

            (*Transfer).Flags |= TRANSFER_FLAG_SUBMITED;
            KeQuerySystemTime(core::ptr::addr_of_mut!((*Transfer).Time));

            TimeOut.QuadPart = 10000 * (*Transfer).TimeOut as i64;
            (*Transfer).Time.QuadPart += TimeOut.QuadPart;
        }

        if ((*Transfer).Flags & (TRANSFER_FLAG_CANCELED | TRANSFER_FLAG_ABORTED)) != 0 {
            return USBPORT_ENDPOINT_PAUSED;
        }

        Entry = (*Transfer).TransferLink.Flink;
    }

    return USBPORT_ENDPOINT_ACTIVE;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_DmaEndpointWorker(Endpoint: PUSBPORT_ENDPOINT) {
    let FdoDevice: PDEVICE_OBJECT;
    let PrevState: ULONG;
    let EndpointState: ULONG;
    let mut IsPaused: BOOLEAN = 0;

    FdoDevice = (*Endpoint).FdoDevice;

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
    );

    PrevState = USBPORT_GetEndpointState(Endpoint);

    if PrevState == USBPORT_ENDPOINT_PAUSED as ULONG {
        EndpointState = USBPORT_DmaEndpointPaused(FdoDevice, Endpoint);
    } else if PrevState == USBPORT_ENDPOINT_ACTIVE as ULONG {
        EndpointState = USBPORT_DmaEndpointActive(FdoDevice, Endpoint);
    } else {
        EndpointState = USBPORT_ENDPOINT_UNKNOWN as ULONG;
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        (*Endpoint).EndpointOldIrql,
    );

    USBPORT_FlushCancelList(Endpoint);

    KeAcquireSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        core::ptr::addr_of_mut!((*Endpoint).EndpointOldIrql),
    );

    if EndpointState == PrevState {
        if EndpointState == USBPORT_ENDPOINT_PAUSED as ULONG {
            IsPaused = 1;
        }
    } else {
        USBPORT_SetEndpointState(Endpoint, EndpointState);
    }

    KeReleaseSpinLock(
        (core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize),
        (*Endpoint).EndpointOldIrql,
    );

    if IsPaused != 0 {
        USBPORT_InvalidateEndpointHandler(
            FdoDevice,
            Endpoint,
            INVALIDATE_ENDPOINT_WORKER_THREAD as ULONG,
        );
    }
}

// Note: I need to declare the externs and constants used. Here they are above the function:

const USBPORT_ENDPOINT_PAUSED: ULONG = 0x00000002; // FIXED 2026-07-05: translation invented 0x4; verified vs usbport.h/usbmport.h
const USBPORT_ENDPOINT_ACTIVE: ULONG = 0x00000003; // FIXED 2026-07-05: translation invented 0x2; verified vs usbport.h/usbmport.h
const USBPORT_ENDPOINT_UNKNOWN: ULONG = 0;
const INVALIDATE_ENDPOINT_WORKER_THREAD: ULONG = 0x00000001; // FIXED 2026-07-05: translation invented 0x3; verified vs usbport.h/usbmport.h

#[no_mangle]
pub unsafe extern "system" fn USBPORT_EndpointWorker(
    Endpoint: PUSBPORT_ENDPOINT,
    LockNotChecked: BOOLEAN,
) -> BOOLEAN {
    let mut FdoDevice: PDEVICE_OBJECT;
    let mut FdoExtension: PUSBPORT_DEVICE_EXTENSION;
    let mut Packet: PUSBPORT_REGISTRATION_PACKET;
    let mut EndpointState: ULONG;

    // DPRINT_CORE!("USBPORT_EndpointWorker: Endpoint - %p, LockNotChecked - %x\n", Endpoint, LockNotChecked);

    FdoDevice = (*Endpoint).FdoDevice;
    FdoExtension = (*FdoDevice).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
    Packet = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet)
        as PUSBPORT_REGISTRATION_PACKET;

    if LockNotChecked as i32 == 0 {
        if InterlockedIncrement(
            core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
        ) != 0
        {
            InterlockedDecrement(
                core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
            );
            // DPRINT_CORE!("USBPORT_EndpointWorker: LockCounter > 0\n");
            return 1 as BOOLEAN;
        }
    }

    if !(KeGetCurrentIrql() as i32 == DISPATCH_LEVEL as i32) {
        RtlAssert(
            b"KeGetCurrentIrql() == DISPATCH_LEVEL\0".as_ptr() as *const i8 as PVOID,
            b"endpoint.c\0".as_ptr() as *const i8 as PVOID,
            1711 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));

    if USBPORT_GetEndpointState(Endpoint) == USBPORT_ENDPOINT_CLOSED as ULONG {
        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));
        InterlockedDecrement(
            core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
        );
        // DPRINT_CORE!("USBPORT_EndpointWorker: State == USBPORT_ENDPOINT_CLOSED. return FALSE\n");
        return 0 as BOOLEAN;
    }

    if (*Endpoint).Flags as ULONG
        & (ENDPOINT_FLAG_ROOTHUB_EP0 | ENDPOINT_FLAG_NUKE) as ULONG
        == 0 as ULONG
    {
        KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize));
        ((*Packet).PollEndpoint).expect("non-null function pointer")(
            (*FdoExtension).MiniPortExt,
            Endpoint.offset(1) as PVOID,
        );
        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).MiniportSpinLock) as *mut usize));
    }

    EndpointState = USBPORT_GetEndpointState(Endpoint);

    if EndpointState == USBPORT_ENDPOINT_REMOVE as ULONG {
        KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize));
        (*Endpoint).StateLast = USBPORT_ENDPOINT_CLOSED as ULONG;
        (*Endpoint).StateNext = USBPORT_ENDPOINT_CLOSED as ULONG;
        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize));

        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));

        KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize));

        ExInterlockedInsertTailList(
            core::ptr::addr_of_mut!((*FdoExtension).EndpointClosedList),
            core::ptr::addr_of_mut!((*Endpoint).CloseLink),
            core::ptr::addr_of_mut!((*FdoExtension).EndpointClosedSpinLock),
        );

        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*FdoExtension).EndpointListSpinLock) as *mut usize));

        InterlockedDecrement(
            core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
        );
        // DPRINT_CORE!("USBPORT_EndpointWorker: State == USBPORT_ENDPOINT_REMOVE. return FALSE\n");
        return 0 as BOOLEAN;
    }

    if !IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).PendingTransferList))
        || !IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).TransferList))
        || !IsListEmpty(core::ptr::addr_of_mut!((*Endpoint).CancelList))
    {
        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));

        EndpointState = USBPORT_GetEndpointState(Endpoint);

        KeAcquireSpinLockAtDpcLevel((core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize));
        if EndpointState == (*Endpoint).StateNext {
            KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize));

            if (*Endpoint).EndpointWorker != 0 {
                USBPORT_DmaEndpointWorker(Endpoint);
            } else {
                USBPORT_RootHubEndpointWorker(Endpoint);
            }

            USBPORT_FlushAbortList(Endpoint);

            InterlockedDecrement(
                core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
            );
            // DPRINT_CORE!("USBPORT_EndpointWorker: return FALSE\n");
            return 0 as BOOLEAN;
        }

        KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).StateChangeSpinLock) as *mut usize));
        InterlockedDecrement(
            core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
        );

        // DPRINT_CORE!("USBPORT_EndpointWorker: return TRUE\n");
        return 1 as BOOLEAN;
    }

    KeReleaseSpinLockFromDpcLevel((core::ptr::addr_of_mut!((*Endpoint).EndpointSpinLock) as *mut usize));

    USBPORT_FlushAbortList(Endpoint);

    InterlockedDecrement(
        core::ptr::addr_of_mut!((*Endpoint).LockCounter) as *mut core::ffi::c_long,
    );
    // DPRINT_CORE!("USBPORT_EndpointWorker: return FALSE\n");
    return 0 as BOOLEAN;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Port Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBPort endpoint functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbport.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_USBPORT_CORE
// #include "usbdebug.h"
//
// ULONG
// NTAPI
// USBPORT_CalculateUsbBandwidth(IN PDEVICE_OBJECT FdoDevice,
//                               IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties;
//     ULONG Bandwidth;
//     ULONG Overhead;
//
//     DPRINT("USBPORT_CalculateUsbBandwidth ... \n");
//
//     EndpointProperties = &Endpoint->EndpointProperties;
//
//     switch (EndpointProperties->TransferType)
//     {
//         case USBPORT_TRANSFER_TYPE_ISOCHRONOUS:
//             Overhead = USB2_FS_ISOCHRONOUS_OVERHEAD;
//             break;
//
//         case USBPORT_TRANSFER_TYPE_INTERRUPT:
//             Overhead = USB2_FS_INTERRUPT_OVERHEAD;
//             break;
//
//         default: //USBPORT_TRANSFER_TYPE_CONTROL or USBPORT_TRANSFER_TYPE_BULK
//             Overhead = 0;
//             break;
//     }
//
//     if (Overhead == 0)
//     {
//         Bandwidth = 0;
//     }
//     else
//     {
//         Bandwidth = (EndpointProperties->TotalMaxPacketSize + Overhead) * USB2_BIT_STUFFING_OVERHEAD;
//     }
//
//     if (EndpointProperties->DeviceSpeed == UsbLowSpeed)
//     {
//         Bandwidth *= 8;
//     }
//
//     return Bandwidth;
// }
//
// BOOLEAN
// NTAPI
// USBPORT_AllocateBandwidth(IN PDEVICE_OBJECT FdoDevice,
//                           IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties;
//     ULONG TransferType;
//     ULONG TotalBusBandwidth;
//     ULONG EndpointBandwidth;
//     ULONG MinBandwidth;
//     PULONG Bandwidth;
//     ULONG MaxBandwidth = 0;
//     ULONG ix;
//     ULONG Offset;
//     LONG ScheduleOffset = -1;
//     ULONG Period;
//     ULONG Factor;
//     UCHAR Bit;
//
//     DPRINT("USBPORT_AllocateBandwidth: FdoDevice - %p, Endpoint - %p\n",
//            FdoDevice,
//            Endpoint);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     EndpointProperties = &Endpoint->EndpointProperties;
//     TransferType = EndpointProperties->TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_BULK ||
//         TransferType == USBPORT_TRANSFER_TYPE_CONTROL ||
//         Endpoint->Flags & ENDPOINT_FLAG_ROOTHUB_EP0)
//     {
//         EndpointProperties->ScheduleOffset = 0;
//         return TRUE;
//     }
//
//     TotalBusBandwidth = FdoExtension->TotalBusBandwidth;
//     EndpointBandwidth = EndpointProperties->UsbBandwidth;
//
//     Period = EndpointProperties->Period;
//     ASSERT(Period != 0);
//     Factor = USB2_FRAMES / Period;
//
//     for (Offset = 0; Offset < Period; Offset++)
//     {
//         MinBandwidth = TotalBusBandwidth;
//         Bandwidth = &FdoExtension->Bandwidth[Offset * Factor];
//
//         for (ix = 1; *Bandwidth >= EndpointBandwidth; ix++)
//         {
//             MinBandwidth = min(MinBandwidth, *Bandwidth);
//
//             Bandwidth++;
//
//             if (Factor <= ix)
//             {
//                 if (MinBandwidth > MaxBandwidth)
//                 {
//                     MaxBandwidth = MinBandwidth;
//                     ScheduleOffset = Offset;
//
//                     DPRINT("USBPORT_AllocateBandwidth: ScheduleOffset - %X\n",
//                            ScheduleOffset);
//                 }
//
//                 break;
//             }
//         }
//     }
//
//     DPRINT("USBPORT_AllocateBandwidth: ScheduleOffset - %X\n", ScheduleOffset);
//
//     if (ScheduleOffset != -1)
//     {
//         EndpointProperties->ScheduleOffset = ScheduleOffset;
//
//         Bandwidth = &FdoExtension->Bandwidth[ScheduleOffset * Factor];
//
//         for (Factor = USB2_FRAMES / Period; Factor; Factor--)
//         {
//             FdoExtension->Bandwidth[ScheduleOffset * Factor] -= EndpointBandwidth;
//         }
//
//         if (TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//         {
//             for (Bit = 0x80; Bit != 0; Bit >>= 1)
//             {
//                 if ((Period & Bit) != 0)
//                 {
//                     Period = Bit;
//                     break;
//                 }
//             }
//
//             DPRINT("USBPORT_AllocateBandwidth: FIXME AllocatedInterrupt_XXms\n");
//         }
//         else
//         {
//             DPRINT("USBPORT_AllocateBandwidth: FIXME AllocatedIso\n");
//         }
//     }
//
//     DPRINT("USBPORT_AllocateBandwidth: FIXME USBPORT_UpdateAllocatedBw\n");
//
//     DPRINT("USBPORT_AllocateBandwidth: ScheduleOffset - %X\n", ScheduleOffset);
//     return ScheduleOffset != -1;
// }
//
// VOID
// NTAPI
// USBPORT_FreeBandwidth(IN PDEVICE_OBJECT FdoDevice,
//                       IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties;
//     ULONG TransferType;
//     ULONG Offset;
//     ULONG EndpointBandwidth;
//     ULONG Period;
//     ULONG Factor;
//     UCHAR Bit;
//
//     DPRINT("USBPORT_FreeBandwidth: FdoDevice - %p, Endpoint - %p\n",
//            FdoDevice,
//            Endpoint);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     EndpointProperties = &Endpoint->EndpointProperties;
//     TransferType = EndpointProperties->TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_BULK ||
//         TransferType == USBPORT_TRANSFER_TYPE_CONTROL ||
//         (Endpoint->Flags & ENDPOINT_FLAG_ROOTHUB_EP0))
//     {
//         return;
//     }
//
//     Offset = Endpoint->EndpointProperties.ScheduleOffset;
//     EndpointBandwidth = Endpoint->EndpointProperties.UsbBandwidth;
//
//     Period = Endpoint->EndpointProperties.Period;
//     ASSERT(Period != 0);
//
//     for (Factor = USB2_FRAMES / Period; Factor; Factor--)
//     {
//         FdoExtension->Bandwidth[Offset * Factor] += EndpointBandwidth;
//     }
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//         for (Bit = 0x80; Bit != 0; Bit >>= 1)
//         {
//             if ((Period & Bit) != 0)
//             {
//                 Period = Bit;
//                 break;
//             }
//         }
//
//         ASSERT(Period != 0);
//
//         DPRINT("USBPORT_AllocateBandwidth: FIXME AllocatedInterrupt_XXms\n");
//     }
//     else
//     {
//         DPRINT("USBPORT_AllocateBandwidth: FIXME AllocatedIso\n");
//     }
//
//     DPRINT1("USBPORT_FreeBandwidth: FIXME USBPORT_UpdateAllocatedBw\n");
// }
//
// UCHAR
// NTAPI
// USBPORT_NormalizeHsInterval(UCHAR Interval)
// {
//     UCHAR interval;
//
//     DPRINT("USBPORT_NormalizeHsInterval: Interval - %x\n", Interval);
//
//     interval = Interval;
//
//     if (Interval)
//        interval = Interval - 1;
//
//     if (interval > 5)
//        interval = 5;
//
//     return 1 << interval;
// }
//
// BOOLEAN
// NTAPI
// USBPORT_EndpointHasQueuedTransfers(IN PDEVICE_OBJECT FdoDevice,
//                                    IN PUSBPORT_ENDPOINT Endpoint,
//                                    IN PULONG TransferCount)
// {
//     PLIST_ENTRY Entry;
//     PUSBPORT_TRANSFER Transfer;
//     BOOLEAN Result = FALSE;
//
//     DPRINT_CORE!("USBPORT_EndpointHasQueuedTransfers: ... \n");
//
//     KeAcquireSpinLock(&Endpoint->EndpointSpinLock, &Endpoint->EndpointOldIrql);
//
//     if (!IsListEmpty(&Endpoint->PendingTransferList))
//         Result = TRUE;
//
//     if (!IsListEmpty(&Endpoint->TransferList))
//     {
//         Result = TRUE;
//
//         if (TransferCount)
//         {
//             *TransferCount = 0;
//
//             for (Entry = Endpoint->TransferList.Flink;
//                  Entry && Entry != &Endpoint->TransferList;
//                  Entry = Transfer->TransferLink.Flink)
//             {
//                 Transfer = CONTAINING_RECORD(Entry,
//                                              USBPORT_TRANSFER,
//                                              TransferLink);
//
//                 if (Transfer->Flags & TRANSFER_FLAG_SUBMITED)
//                 {
//                     ++*TransferCount;
//                 }
//             }
//         }
//     }
//
//     KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//
//     return Result;
// }
//
// VOID
// NTAPI
// USBPORT_NukeAllEndpoints(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//     PLIST_ENTRY EndpointList;
//     PUSBPORT_ENDPOINT Endpoint;
//     KIRQL OldIrql;
//
//     DPRINT("USBPORT_NukeAllEndpoints \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeAcquireSpinLock(&FdoExtension->EndpointListSpinLock, &OldIrql);
//
//     EndpointList = FdoExtension->EndpointList.Flink;
//
//     while (EndpointList && (EndpointList != &FdoExtension->EndpointList))
//     {
//         Endpoint = CONTAINING_RECORD(EndpointList,
//                                      USBPORT_ENDPOINT,
//                                      EndpointLink);
//
//         if (!(Endpoint->Flags & ENDPOINT_FLAG_ROOTHUB_EP0))
//             Endpoint->Flags |= ENDPOINT_FLAG_NUKE;
//
//         EndpointList = Endpoint->EndpointLink.Flink;
//     }
//
//     KeReleaseSpinLock(&FdoExtension->EndpointListSpinLock, OldIrql);
// }
//
// ULONG
// NTAPI
// USBPORT_GetEndpointState(IN PUSBPORT_ENDPOINT Endpoint)
// {
//     ULONG State;
//
//     //DPRINT("USBPORT_GetEndpointState \n");
//
//     KeAcquireSpinLockAtDpcLevel(&Endpoint->StateChangeSpinLock);
//
//     if (Endpoint->StateLast != Endpoint->StateNext)
//     {
//         State = USBPORT_ENDPOINT_UNKNOWN;
//     }
//     else
//     {
//         State = Endpoint->StateLast;
//     }
//
//     KeReleaseSpinLockFromDpcLevel(&Endpoint->StateChangeSpinLock);
//
//     if (State != USBPORT_ENDPOINT_ACTIVE)
//     {
//         DPRINT("USBPORT_GetEndpointState: Endpoint - %p, State - %x\n",
//                Endpoint,
//                State);
//     }
//
//     return State;
// }
//
// VOID
// NTAPI
// USBPORT_SetEndpointState(IN PUSBPORT_ENDPOINT Endpoint,
//                          IN ULONG State)
// {
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     KIRQL OldIrql;
//
//     DPRINT("USBPORT_SetEndpointState: Endpoint - %p, State - %x\n",
//            Endpoint,
//            State);
//
//     FdoDevice = Endpoint->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     KeAcquireSpinLock(&Endpoint->StateChangeSpinLock,
//                       &Endpoint->EndpointStateOldIrql);
//
//     if (!(Endpoint->Flags & ENDPOINT_FLAG_ROOTHUB_EP0))
//     {
//         if (Endpoint->Flags & ENDPOINT_FLAG_NUKE)
//         {
//             Endpoint->StateLast = State;
//             Endpoint->StateNext = State;
//
//             KeReleaseSpinLock(&Endpoint->StateChangeSpinLock,
//                               Endpoint->EndpointStateOldIrql);
//
//             USBPORT_InvalidateEndpointHandler(FdoDevice,
//                                               Endpoint,
//                                               INVALIDATE_ENDPOINT_WORKER_THREAD);
//             return;
//         }
//
//         KeReleaseSpinLock(&Endpoint->StateChangeSpinLock,
//                           Endpoint->EndpointStateOldIrql);
//
//         KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//         Packet->SetEndpointState(FdoExtension->MiniPortExt,
//                                  Endpoint + 1,
//                                  State);
//         KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//
//         Endpoint->StateNext = State;
//
//         KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//         Endpoint->FrameNumber = Packet->Get32BitFrameNumber(FdoExtension->MiniPortExt);
//         KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//
//         KIRQL StateChangeOldIrql;
//         KeAcquireSpinLock(&FdoExtension->EpStateChangeSpinLock, &StateChangeOldIrql);
//
//         if (Endpoint->StateChangeLink.Flink == NULL &&
//             Endpoint->StateChangeLink.Blink == NULL)
//         {
//             InsertTailList(&FdoExtension->EpStateChangeList,
//                           &Endpoint->StateChangeLink);
//         }
//
//         KeReleaseSpinLock(&FdoExtension->EpStateChangeSpinLock, StateChangeOldIrql);
//
//         KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//         Packet->InterruptNextSOF(FdoExtension->MiniPortExt);
//         KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//     }
//     else
//     {
//         Endpoint->StateLast = State;
//         Endpoint->StateNext = State;
//
//         if (State == USBPORT_ENDPOINT_REMOVE)
//         {
//             KeReleaseSpinLock(&Endpoint->StateChangeSpinLock,
//                               Endpoint->EndpointStateOldIrql);
//
//             USBPORT_InvalidateEndpointHandler(FdoDevice,
//                                               Endpoint,
//                                               INVALIDATE_ENDPOINT_WORKER_THREAD);
//             return;
//         }
//
//         KeReleaseSpinLock(&Endpoint->StateChangeSpinLock,
//                           Endpoint->EndpointStateOldIrql);
//     }
// }
//
// VOID
// NTAPI
// USBPORT_AddPipeHandle(IN PUSBPORT_DEVICE_HANDLE DeviceHandle,
//                       IN PUSBPORT_PIPE_HANDLE PipeHandle)
// {
//     DPRINT("USBPORT_AddPipeHandle: DeviceHandle - %p, PipeHandle - %p\n",
//            DeviceHandle,
//            PipeHandle);
//
//     InsertTailList(&DeviceHandle->PipeHandleList, &PipeHandle->PipeLink);
// }
//
// VOID
// NTAPI
// USBPORT_RemovePipeHandle(IN PUSBPORT_DEVICE_HANDLE DeviceHandle,
//                          IN PUSBPORT_PIPE_HANDLE PipeHandle)
// {
//     DPRINT("USBPORT_RemovePipeHandle: PipeHandle - %p\n", PipeHandle);
//
//     RemoveEntryList(&PipeHandle->PipeLink);
//
//     PipeHandle->PipeLink.Flink = NULL;
//     PipeHandle->PipeLink.Blink = NULL;
// }
//
// BOOLEAN
// NTAPI
// USBPORT_ValidatePipeHandle(IN PUSBPORT_DEVICE_HANDLE DeviceHandle,
//                            IN PUSBPORT_PIPE_HANDLE PipeHandle)
// {
//     PLIST_ENTRY HandleList;
//     PUSBPORT_PIPE_HANDLE CurrentHandle;
//
//     //DPRINT("USBPORT_ValidatePipeHandle: DeviceHandle - %p, PipeHandle - %p\n",
//     //       DeviceHandle,
//     //       PipeHandle);
//
//     HandleList = DeviceHandle->PipeHandleList.Flink;
//
//     while (HandleList != &DeviceHandle->PipeHandleList)
//     {
//         CurrentHandle = CONTAINING_RECORD(HandleList,
//                                           USBPORT_PIPE_HANDLE,
//                                           PipeLink);
//
//         HandleList = HandleList->Flink;
//
//         if (CurrentHandle == PipeHandle)
//             return TRUE;
//     }
//
//     return FALSE;
// }
//
// BOOLEAN
// NTAPI
// USBPORT_DeleteEndpoint(IN PDEVICE_OBJECT FdoDevice,
//                        IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//     BOOLEAN Result;
//     KIRQL OldIrql;
//
//     DPRINT1("USBPORT_DeleteEndpoint: Endpoint - %p\n", Endpoint);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     /* 
//      * Remove endpoint from EpStateChangeList if it's still there.
//      * This prevents the DPC handler from accessing a deleted endpoint.
//      */
//     KIRQL StateChangeOldIrql;
//     KeAcquireSpinLock(&FdoExtension->EpStateChangeSpinLock, &StateChangeOldIrql);
//
//     if (Endpoint->StateChangeLink.Flink != NULL &&
//         Endpoint->StateChangeLink.Blink != NULL)
//     {
//         RemoveEntryList(&Endpoint->StateChangeLink);
//     }
//     
//     Endpoint->StateChangeLink.Flink = NULL;
//     Endpoint->StateChangeLink.Blink = NULL;
//     
//     KeReleaseSpinLock(&FdoExtension->EpStateChangeSpinLock, StateChangeOldIrql);
//
//     if ((Endpoint->WorkerLink.Flink && Endpoint->WorkerLink.Blink) ||
//         Endpoint->LockCounter != -1)
//     {
//         KeAcquireSpinLock(&FdoExtension->EndpointListSpinLock, &OldIrql);
//
//         ExInterlockedInsertTailList(&FdoExtension->EndpointClosedList,
//                                     &Endpoint->CloseLink,
//                                     &FdoExtension->EndpointClosedSpinLock);
//
//         KeReleaseSpinLock(&FdoExtension->EndpointListSpinLock, OldIrql);
//
//         Result = FALSE;
//     }
//     else
//     {
//         KeAcquireSpinLock(&FdoExtension->EndpointListSpinLock, &OldIrql);
//
//         RemoveEntryList(&Endpoint->EndpointLink);
//         Endpoint->EndpointLink.Flink = NULL;
//         Endpoint->EndpointLink.Blink = NULL;
//
//         KeReleaseSpinLock(&FdoExtension->EndpointListSpinLock, OldIrql);
//
//         MiniportCloseEndpoint(FdoDevice, Endpoint);
//
//         if (Endpoint->HeaderBuffer)
//         {
//             USBPORT_FreeCommonBuffer(FdoDevice, Endpoint->HeaderBuffer);
//         }
//
//         ExFreePoolWithTag(Endpoint, USB_PORT_TAG);
//
//         Result = TRUE;
//     }
//
//     return Result;
// }
//
// VOID
// NTAPI
// MiniportCloseEndpoint(IN PDEVICE_OBJECT FdoDevice,
//                       IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     BOOLEAN IsDoDisablePeriodic;
//     ULONG TransferType;
//     KIRQL OldIrql;
//
//     DPRINT("MiniportCloseEndpoint: Endpoint - %p\n", Endpoint);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//
//     if (Endpoint->Flags & ENDPOINT_FLAG_OPENED)
//     {
//         TransferType = Endpoint->EndpointProperties.TransferType;
//
//         if (TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT ||
//             TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//         {
//             --FdoExtension->PeriodicEndpoints;
//         }
//
//         IsDoDisablePeriodic = FdoExtension->PeriodicEndpoints == 0;
//
//         Packet->CloseEndpoint(FdoExtension->MiniPortExt,
//                               Endpoint + 1,
//                               IsDoDisablePeriodic);
//
//         Endpoint->Flags &= ~ENDPOINT_FLAG_OPENED;
//         Endpoint->Flags |= ENDPOINT_FLAG_CLOSED;
//     }
//
//     KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
// }
//
// VOID
// NTAPI
// USBPORT_ClosePipe(IN PUSBPORT_DEVICE_HANDLE DeviceHandle,
//                   IN PDEVICE_OBJECT FdoDevice,
//                   IN PUSBPORT_PIPE_HANDLE PipeHandle)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PUSBPORT_ENDPOINT Endpoint;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PUSB2_TT_EXTENSION TtExtension;
//     ULONG ix;
//     BOOLEAN IsReady;
//     KIRQL OldIrql;
//
//     DPRINT1("USBPORT_ClosePipe \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     if (PipeHandle->Flags & PIPE_HANDLE_FLAG_CLOSED)
//         return;
//
//     USBPORT_RemovePipeHandle(DeviceHandle, PipeHandle);
//
//     PipeHandle->Flags |= PIPE_HANDLE_FLAG_CLOSED;
//
//     if (PipeHandle->Flags & PIPE_HANDLE_FLAG_NULL_PACKET_SIZE)
//     {
//         PipeHandle->Flags &= ~PIPE_HANDLE_FLAG_NULL_PACKET_SIZE;
//         return;
//     }
//
//     Endpoint = PipeHandle->Endpoint;
//
//     KeAcquireSpinLock(&FdoExtension->EndpointListSpinLock, &OldIrql);
//
//     if ((Endpoint->Flags & ENDPOINT_FLAG_ROOTHUB_EP0) &&
//         (Endpoint->EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT))
//     {
//         PdoExtension = FdoExtension->RootHubPdo->DeviceExtension;
//         PdoExtension->Endpoint = NULL;
//     }
//
//     KeReleaseSpinLock(&FdoExtension->EndpointListSpinLock, OldIrql);
//
//     while (TRUE)
//     {
//         IsReady = TRUE;
//
//         KeAcquireSpinLock(&Endpoint->EndpointSpinLock,
//                           &Endpoint->EndpointOldIrql);
//
//         if (!IsListEmpty(&Endpoint->PendingTransferList))
//             IsReady = FALSE;
//
//         if (!IsListEmpty(&Endpoint->TransferList))
//             IsReady = FALSE;
//
//         if (!IsListEmpty(&Endpoint->CancelList))
//             IsReady = FALSE;
//
//         if (!IsListEmpty(&Endpoint->AbortList))
//             IsReady = FALSE;
//
//         KeAcquireSpinLockAtDpcLevel(&Endpoint->StateChangeSpinLock);
//         if (Endpoint->StateLast != Endpoint->StateNext)
//             IsReady = FALSE;
//         KeReleaseSpinLockFromDpcLevel(&Endpoint->StateChangeSpinLock);
//
//         KeReleaseSpinLock(&Endpoint->EndpointSpinLock,
//                           Endpoint->EndpointOldIrql);
//
//         if (InterlockedIncrement(&Endpoint->LockCounter))
//             IsReady = FALSE;
//         InterlockedDecrement(&Endpoint->LockCounter);
//
//         if (IsReady == TRUE)
//             break;
//
//         USBPORT_Wait(FdoDevice, 1);
//     }
//
//     Endpoint->DeviceHandle = NULL;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     if (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2)
//     {
//         USBPORT_FreeBandwidthUSB2(FdoDevice, Endpoint);
//
//         KeAcquireSpinLock(&FdoExtension->TtSpinLock, &OldIrql);
//
//         TtExtension = Endpoint->TtExtension;
//         DPRINT1("USBPORT_ClosePipe: TtExtension - %p\n", TtExtension);
//
//         if (TtExtension)
//         {
//             RemoveEntryList(&Endpoint->TtLink);
//
//             Endpoint->TtLink.Flink = NULL;
//             Endpoint->TtLink.Blink = NULL;
//
//             if (TtExtension->Flags & USB2_TT_EXTENSION_FLAG_DELETED)
//             {
//                 if (IsListEmpty(&TtExtension->EndpointList))
//                 {
//                     USBPORT_UpdateAllocatedBwTt(TtExtension);
//
//                     for (ix = 0; ix < USB2_FRAMES; ix++)
//                     {
//                         FdoExtension->Bandwidth[ix] += TtExtension->MaxBandwidth;
//                     }
//
//                     DPRINT1("USBPORT_ClosePipe: ExFreePoolWithTag TtExtension - %p\n", TtExtension);
//                     ExFreePoolWithTag(TtExtension, USB_PORT_TAG);
//                 }
//             }
//         }
//
//         KeReleaseSpinLock(&FdoExtension->TtSpinLock, OldIrql);
//     }
//     else
//     {
//         USBPORT_FreeBandwidth(FdoDevice, Endpoint);
//     }
//
//     KeAcquireSpinLock(&Endpoint->EndpointSpinLock, &Endpoint->EndpointOldIrql);
//     USBPORT_SetEndpointState(Endpoint, USBPORT_ENDPOINT_REMOVE);
//     KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//
//     USBPORT_SignalWorkerThread(FdoDevice);
// }
//
// MPSTATUS
// NTAPI
// MiniportOpenEndpoint(IN PDEVICE_OBJECT FdoDevice,
//                      IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     KIRQL OldIrql;
//     ULONG TransferType;
//     MPSTATUS MpStatus;
//
//     DPRINT("MiniportOpenEndpoint: Endpoint - %p\n", Endpoint);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//
//     Endpoint->Flags &= ~ENDPOINT_FLAG_CLOSED;
//
//     MpStatus = Packet->OpenEndpoint(FdoExtension->MiniPortExt,
//                                     &Endpoint->EndpointProperties,
//                                     Endpoint + 1);
//
//     if (!MpStatus)
//     {
//         TransferType = Endpoint->EndpointProperties.TransferType;
//
//         if (TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT ||
//             TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//         {
//             ++FdoExtension->PeriodicEndpoints;
//         }
//
//         Endpoint->Flags |= ENDPOINT_FLAG_OPENED;
//     }
//
//     KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//     return MpStatus;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_OpenPipe(IN PDEVICE_OBJECT FdoDevice,
//                  IN PUSBPORT_DEVICE_HANDLE DeviceHandle,
//                  IN PUSBPORT_PIPE_HANDLE PipeHandle,
//                  IN OUT PUSBD_STATUS UsbdStatus)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_RHDEVICE_EXTENSION PdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     SIZE_T EndpointSize;
//     PUSBPORT_ENDPOINT Endpoint;
//     PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties;
//     PUSB_ENDPOINT_DESCRIPTOR EndpointDescriptor;
//     UCHAR Direction;
//     UCHAR Interval;
//     UCHAR Period;
//     USBPORT_ENDPOINT_REQUIREMENTS EndpointRequirements = {0};
//     PUSBPORT_COMMON_BUFFER_HEADER HeaderBuffer;
//     MPSTATUS MpStatus;
//     USBD_STATUS USBDStatus = USBD_STATUS_SUCCESS;
//     NTSTATUS Status;
//     KIRQL OldIrql;
//     USHORT MaxPacketSize;
//     USHORT AdditionalTransaction;
//     BOOLEAN IsAllocatedBandwidth;
//     ULONG RetryCount;
//
//     DPRINT1("USBPORT_OpenPipe: DeviceHandle - %p, FdoDevice - %p, PipeHandle - %p\n",
//            DeviceHandle,
//            FdoDevice,
//            PipeHandle);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     EndpointSize = sizeof(USBPORT_ENDPOINT) + Packet->MiniPortEndpointSize;
//
//     if (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2)
//     {
//         EndpointSize += sizeof(USB2_TT_ENDPOINT);
//     }
//
//     if (PipeHandle->EndpointDescriptor.wMaxPacketSize == 0)
//     {
//         USBPORT_AddPipeHandle(DeviceHandle, PipeHandle);
//
//         PipeHandle->Flags = (PipeHandle->Flags & ~PIPE_HANDLE_FLAG_CLOSED) |
//                              PIPE_HANDLE_FLAG_NULL_PACKET_SIZE;
//
//         PipeHandle->Endpoint = (PUSBPORT_ENDPOINT)-1;
//
//         return STATUS_SUCCESS;
//     }
//
//     Endpoint = ExAllocatePoolWithTag(NonPagedPool, EndpointSize, USB_PORT_TAG);
//
//     if (!Endpoint)
//     {
//         DPRINT1("USBPORT_OpenPipe: Not allocated Endpoint!\n");
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         return Status;
//     }
//
//     RtlZeroMemory(Endpoint, EndpointSize);
//
//     Endpoint->FdoDevice = FdoDevice;
//     Endpoint->DeviceHandle = DeviceHandle;
//     Endpoint->LockCounter = -1;
//
//     Endpoint->TtExtension = DeviceHandle->TtExtension;
//
//     if (DeviceHandle->TtExtension)
//     {
//         ExInterlockedInsertTailList(&DeviceHandle->TtExtension->EndpointList,
//                                     &Endpoint->TtLink,
//                                     &FdoExtension->TtSpinLock);
//     }
//
//     if (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2)
//     {
//         Endpoint->TtEndpoint = (PUSB2_TT_ENDPOINT)((ULONG_PTR)Endpoint +
//                                                    sizeof(USBPORT_ENDPOINT) +
//                                                    Packet->MiniPortEndpointSize);
//     }
//     else
//     {
//         Endpoint->TtEndpoint = NULL;
//     }
//
//     KeInitializeSpinLock(&Endpoint->EndpointSpinLock);
//     KeInitializeSpinLock(&Endpoint->StateChangeSpinLock);
//
//     InitializeListHead(&Endpoint->PendingTransferList);
//     InitializeListHead(&Endpoint->TransferList);
//     InitializeListHead(&Endpoint->CancelList);
//     InitializeListHead(&Endpoint->AbortList);
//
//     EndpointProperties = &Endpoint->EndpointProperties;
//     EndpointDescriptor = &PipeHandle->EndpointDescriptor;
//
//     MaxPacketSize = EndpointDescriptor->wMaxPacketSize & 0x7FF;
//     AdditionalTransaction = (EndpointDescriptor->wMaxPacketSize >> 11) & 3;
//
//     EndpointProperties->DeviceAddress = DeviceHandle->DeviceAddress;
//     EndpointProperties->DeviceSpeed = DeviceHandle->DeviceSpeed;
//     EndpointProperties->Period = 0;
//     EndpointProperties->EndpointAddress = EndpointDescriptor->bEndpointAddress;
//     EndpointProperties->TransactionPerMicroframe = AdditionalTransaction + 1;
//     EndpointProperties->MaxPacketSize = MaxPacketSize;
//     EndpointProperties->TotalMaxPacketSize = MaxPacketSize *
//                                              (AdditionalTransaction + 1);
//
//     if (Endpoint->TtExtension)
//     {
//         EndpointProperties->HubAddr = Endpoint->TtExtension->DeviceAddress;
//     }
//     else
//     {
//         EndpointProperties->HubAddr = -1;
//     }
//
//     EndpointProperties->PortNumber = DeviceHandle->PortNumber;
//
//     switch (EndpointDescriptor->bmAttributes & USB_ENDPOINT_TYPE_MASK)
//     {
//         case USB_ENDPOINT_TYPE_CONTROL:
//             EndpointProperties->TransferType = USBPORT_TRANSFER_TYPE_CONTROL;
//
//             if (EndpointProperties->EndpointAddress == 0)
//             {
//                 EndpointProperties->MaxTransferSize = 0x1000; // OUT Ep0
//             }
//             else
//             {
//                 EndpointProperties->MaxTransferSize = 0x10000;
//             }
//
//             break;
//
//         case USB_ENDPOINT_TYPE_ISOCHRONOUS:
//             DPRINT1("USBPORT_OpenPipe: USB_ENDPOINT_TYPE_ISOCHRONOUS UNIMPLEMENTED. FIXME. \n");
//             EndpointProperties->TransferType = USBPORT_TRANSFER_TYPE_ISOCHRONOUS;
//             EndpointProperties->MaxTransferSize = 0x1000000;
//             break;
//
//         case USB_ENDPOINT_TYPE_BULK:
//             EndpointProperties->TransferType = USBPORT_TRANSFER_TYPE_BULK;
//             EndpointProperties->MaxTransferSize = 0x10000;
//             break;
//
//         case USB_ENDPOINT_TYPE_INTERRUPT:
//             EndpointProperties->TransferType = USBPORT_TRANSFER_TYPE_INTERRUPT;
//             EndpointProperties->MaxTransferSize = 0x400;
//             break;
//     }
//
//     if (EndpointProperties->TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//         if (EndpointProperties->DeviceSpeed == UsbHighSpeed)
//         {
//             Interval = USBPORT_NormalizeHsInterval(EndpointDescriptor->bInterval);
//         }
//         else
//         {
//             Interval = EndpointDescriptor->bInterval;
//         }
//
//         EndpointProperties->Period = ENDPOINT_INTERRUPT_32ms;
//
//         if (Interval && (Interval < USB2_FRAMES))
//         {
//             if ((EndpointProperties->DeviceSpeed != UsbLowSpeed) ||
//                 (Interval >= ENDPOINT_INTERRUPT_8ms))
//             {
//                 if (!(Interval & ENDPOINT_INTERRUPT_32ms))
//                 {
//                     Period = EndpointProperties->Period;
//
//                     do
//                     {
//                         Period >>= 1;
//                     }
//                     while (!(Period & Interval));
//
//                     EndpointProperties->Period = Period;
//                 }
//             }
//             else
//             {
//                 EndpointProperties->Period = ENDPOINT_INTERRUPT_8ms;
//             }
//         }
//     }
//
//     if (EndpointProperties->TransferType == USB_ENDPOINT_TYPE_ISOCHRONOUS)
//     {
//         if (EndpointProperties->DeviceSpeed == UsbHighSpeed)
//         {
//             EndpointProperties->Period =
//                 USBPORT_NormalizeHsInterval(EndpointDescriptor->bInterval);
//         }
//         else
//         {
//             EndpointProperties->Period = ENDPOINT_INTERRUPT_1ms;
//         }
//     }
//
//     if ((DeviceHandle->Flags & DEVICE_HANDLE_FLAG_ROOTHUB) != 0)
//     {
//         Endpoint->Flags |= ENDPOINT_FLAG_ROOTHUB_EP0;
//     }
//
//     if (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2)
//     {
//         IsAllocatedBandwidth = USBPORT_AllocateBandwidthUSB2(FdoDevice, Endpoint);
//     }
//     else
//     {
//         EndpointProperties->UsbBandwidth = USBPORT_CalculateUsbBandwidth(FdoDevice,
//                                                                          Endpoint);
//
//         IsAllocatedBandwidth = USBPORT_AllocateBandwidth(FdoDevice, Endpoint);
//     }
//
//     if (!IsAllocatedBandwidth)
//     {
//         Status = USBPORT_USBDStatusToNtStatus(NULL, USBD_STATUS_NO_BANDWIDTH);
//
//         if (UsbdStatus)
//         {
//             *UsbdStatus = USBD_STATUS_NO_BANDWIDTH;
//         }
//
//         goto ExitWithError;
//     }
//
//     Direction = USB_ENDPOINT_DIRECTION_OUT(EndpointDescriptor->bEndpointAddress);
//     EndpointProperties->Direction = Direction;
//
//     if (DeviceHandle->IsRootHub)
//     {
//         Endpoint->EndpointWorker = 0; // USBPORT_RootHubEndpointWorker;
//
//         Endpoint->Flags |= ENDPOINT_FLAG_ROOTHUB_EP0;
//
//         Endpoint->StateLast = USBPORT_ENDPOINT_ACTIVE;
//         Endpoint->StateNext = USBPORT_ENDPOINT_ACTIVE;
//
//         PdoExtension = FdoExtension->RootHubPdo->DeviceExtension;
//
//         if (EndpointProperties->TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//         {
//             PdoExtension->Endpoint = Endpoint;
//         }
//
//         USBDStatus = USBD_STATUS_SUCCESS;
//     }
//     else
//     {
//         Endpoint->EndpointWorker = 1; // USBPORT_DmaEndpointWorker;
//
//         KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//
//         Packet->QueryEndpointRequirements(FdoExtension->MiniPortExt,
//                                           &Endpoint->EndpointProperties,
//                                           &EndpointRequirements);
//
//         KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//
//         if ((EndpointProperties->TransferType == USBPORT_TRANSFER_TYPE_BULK) ||
//             (EndpointProperties->TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT))
//         {
//             EndpointProperties->MaxTransferSize = EndpointRequirements.MaxTransferSize;
//         }
//
//         if (EndpointRequirements.HeaderBufferSize)
//         {
//             HeaderBuffer = USBPORT_AllocateCommonBuffer(FdoDevice,
//                                                         EndpointRequirements.HeaderBufferSize);
//         }
//         else
//         {
//             HeaderBuffer = NULL;
//         }
//
//         if (HeaderBuffer || (EndpointRequirements.HeaderBufferSize == 0))
//         {
//             Endpoint->HeaderBuffer = HeaderBuffer;
//
//             if (HeaderBuffer)
//             {
//                 EndpointProperties->BufferVA = HeaderBuffer->VirtualAddress;
//                 EndpointProperties->BufferPA = HeaderBuffer->PhysicalAddress;
//                 EndpointProperties->BufferLength = HeaderBuffer->BufferLength; // BufferLength + LengthPadded;
//             }
//
//             MpStatus = MiniportOpenEndpoint(FdoDevice, Endpoint);
//
//             Endpoint->Flags |= ENDPOINT_FLAG_DMA_TYPE;
//             Endpoint->Flags |= ENDPOINT_FLAG_QUEUENE_EMPTY;
//
//             if (MpStatus == 0)
//             {
//                 ULONG State;
//
//                 KeAcquireSpinLock(&Endpoint->EndpointSpinLock,
//                                   &Endpoint->EndpointOldIrql);
//
//                 Endpoint->StateLast = USBPORT_ENDPOINT_PAUSED;
//                 Endpoint->StateNext = USBPORT_ENDPOINT_PAUSED;
//
//                 USBPORT_SetEndpointState(Endpoint, USBPORT_ENDPOINT_ACTIVE);
//
//                 KeReleaseSpinLock(&Endpoint->EndpointSpinLock,
//                                   Endpoint->EndpointOldIrql);
//
//                 /* Wait maximum 1 second for the endpoint to be active */
//                 for (RetryCount = 0; RetryCount < 1000; RetryCount++)
//                 {
//                     KeAcquireSpinLock(&Endpoint->EndpointSpinLock,
//                                       &Endpoint->EndpointOldIrql);
//
//                     State = USBPORT_GetEndpointState(Endpoint);
//
//                     KeReleaseSpinLock(&Endpoint->EndpointSpinLock,
//                                       Endpoint->EndpointOldIrql);
//
//                     if (State == USBPORT_ENDPOINT_ACTIVE)
//                     {
//                         break;
//                     }
//
//                     USBPORT_Wait(FdoDevice, 1); // 1 msec.
//                 }
//                 if (State != USBPORT_ENDPOINT_ACTIVE)
//                 {
//                     DPRINT1("Timeout State %x\n", State);
//                     USBDStatus = USBD_STATUS_TIMEOUT;
//                 }
//             }
//         }
//         else
//         {
//             MpStatus = MP_STATUS_NO_RESOURCES;
//             Endpoint->HeaderBuffer = NULL;
//         }
//
//         if (MpStatus)
//         {
//             USBDStatus = USBD_STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     if (UsbdStatus)
//     {
//         *UsbdStatus = USBDStatus;
//     }
//
//     Status = USBPORT_USBDStatusToNtStatus(NULL, USBDStatus);
//
//     if (NT_SUCCESS(Status))
//     {
//         USBPORT_AddPipeHandle(DeviceHandle, PipeHandle);
//
//         ExInterlockedInsertTailList(&FdoExtension->EndpointList,
//                                     &Endpoint->EndpointLink,
//                                     &FdoExtension->EndpointListSpinLock);
//
//         PipeHandle->Endpoint = Endpoint;
//         PipeHandle->Flags &= ~PIPE_HANDLE_FLAG_CLOSED;
//
//         return Status;
//     }
//
// ExitWithError:
//
//     if (Endpoint)
//     {
//         if (IsAllocatedBandwidth)
//         {
//             if (Packet->MiniPortFlags & USB_MINIPORT_FLAGS_USB2)
//             {
//                 USBPORT_FreeBandwidthUSB2(FdoDevice, Endpoint);
//             }
//             else
//             {
//                 USBPORT_FreeBandwidth(FdoDevice, Endpoint);
//             }
//         }
//
//         if (Endpoint->TtExtension)
//         {
//             KeAcquireSpinLock(&FdoExtension->TtSpinLock, &OldIrql);
//             RemoveEntryList(&Endpoint->TtLink);
//             KeReleaseSpinLock(&FdoExtension->TtSpinLock, OldIrql);
//         }
//
//         ExFreePoolWithTag(Endpoint, USB_PORT_TAG);
//     }
//
//     DPRINT1("USBPORT_OpenPipe: Status - %lx\n", Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// USBPORT_ReopenPipe(IN PDEVICE_OBJECT FdoDevice,
//                    IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_COMMON_BUFFER_HEADER HeaderBuffer;
//     USBPORT_ENDPOINT_REQUIREMENTS EndpointRequirements = {0};
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     KIRQL MiniportOldIrql;
//     NTSTATUS Status;
//
//     DPRINT1("USBPORT_ReopenPipe ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     while (TRUE)
//     {
//         if (!InterlockedIncrement(&Endpoint->LockCounter))
//             break;
//
//         InterlockedDecrement(&Endpoint->LockCounter);
//         USBPORT_Wait(FdoDevice, 1);
//     }
//
//     KeAcquireSpinLock(&Endpoint->EndpointSpinLock, &Endpoint->EndpointOldIrql);
//     KeAcquireSpinLockAtDpcLevel(&FdoExtension->MiniportSpinLock);
//
//     Packet->SetEndpointState(FdoExtension->MiniPortExt,
//                              Endpoint + 1,
//                              USBPORT_ENDPOINT_REMOVE);
//
//     KeReleaseSpinLockFromDpcLevel(&FdoExtension->MiniportSpinLock);
//     KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//
//     USBPORT_Wait(FdoDevice, 2);
//
//     MiniportCloseEndpoint(FdoDevice, Endpoint);
//
//     RtlZeroMemory(Endpoint + 1,
//                   Packet->MiniPortEndpointSize);
//
//     if (Endpoint->HeaderBuffer)
//     {
//         USBPORT_FreeCommonBuffer(FdoDevice, Endpoint->HeaderBuffer);
//         Endpoint->HeaderBuffer = NULL;
//     }
//
//     KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &MiniportOldIrql);
//
//     Packet->QueryEndpointRequirements(FdoExtension->MiniPortExt,
//                                       &Endpoint->EndpointProperties,
//                                       &EndpointRequirements);
//
//     KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, MiniportOldIrql);
//
//     if (EndpointRequirements.HeaderBufferSize)
//     {
//         HeaderBuffer = USBPORT_AllocateCommonBuffer(FdoDevice,
//                                                     EndpointRequirements.HeaderBufferSize);
//     }
//     else
//     {
//         HeaderBuffer = NULL;
//     }
//
//     if (HeaderBuffer || EndpointRequirements.HeaderBufferSize == 0)
//     {
//         Endpoint->HeaderBuffer = HeaderBuffer;
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         Endpoint->HeaderBuffer = 0;
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     if (Endpoint->HeaderBuffer && HeaderBuffer)
//     {
//         Endpoint->EndpointProperties.BufferVA = HeaderBuffer->VirtualAddress;
//         Endpoint->EndpointProperties.BufferPA = HeaderBuffer->PhysicalAddress;
//         Endpoint->EndpointProperties.BufferLength = HeaderBuffer->BufferLength;
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         MiniportOpenEndpoint(FdoDevice, Endpoint);
//
//         KeAcquireSpinLock(&Endpoint->EndpointSpinLock, &Endpoint->EndpointOldIrql);
//         KeAcquireSpinLockAtDpcLevel(&Endpoint->StateChangeSpinLock);
//
//         if (Endpoint->StateLast == USBPORT_ENDPOINT_ACTIVE)
//         {
//             KeReleaseSpinLockFromDpcLevel(&Endpoint->StateChangeSpinLock);
//             KeAcquireSpinLockAtDpcLevel(&FdoExtension->MiniportSpinLock);
//
//             Packet->SetEndpointState(FdoExtension->MiniPortExt,
//                                      Endpoint + 1,
//                                      USBPORT_ENDPOINT_ACTIVE);
//
//             KeReleaseSpinLockFromDpcLevel(&FdoExtension->MiniportSpinLock);
//         }
//         else
//         {
//             KeReleaseSpinLockFromDpcLevel(&Endpoint->StateChangeSpinLock);
//         }
//
//         KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//     }
//
//     InterlockedDecrement(&Endpoint->LockCounter);
//
//     return Status;
// }
//
// VOID
// NTAPI
// USBPORT_FlushClosedEndpointList(IN PDEVICE_OBJECT FdoDevice)
// {
//     PUSBPORT_DEVICE_EXTENSION  FdoExtension;
//     KIRQL OldIrql;
//     PLIST_ENTRY ClosedList;
//     PUSBPORT_ENDPOINT Endpoint;
//
//     DPRINT_CORE!("USBPORT_FlushClosedEndpointList: ... \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     KeAcquireSpinLock(&FdoExtension->EndpointClosedSpinLock, &OldIrql);
//     ClosedList = &FdoExtension->EndpointClosedList;
//
//     while (!IsListEmpty(ClosedList))
//     {
//         Endpoint = CONTAINING_RECORD(ClosedList->Flink,
//                                      USBPORT_ENDPOINT,
//                                      CloseLink);
//
//         RemoveHeadList(ClosedList);
//         Endpoint->CloseLink.Flink = NULL;
//         Endpoint->CloseLink.Blink = NULL;
//
//         KeReleaseSpinLock(&FdoExtension->EndpointClosedSpinLock, OldIrql);
//
//         USBPORT_DeleteEndpoint(FdoDevice, Endpoint);
//
//         KeAcquireSpinLock(&FdoExtension->EndpointClosedSpinLock, &OldIrql);
//     }
//
//     KeReleaseSpinLock(&FdoExtension->EndpointClosedSpinLock, OldIrql);
// }
//
// VOID
// NTAPI
// USBPORT_InvalidateEndpointHandler(IN PDEVICE_OBJECT FdoDevice,
//                                   IN PUSBPORT_ENDPOINT Endpoint,
//                                   IN ULONG Type)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PLIST_ENTRY Entry;
//     PLIST_ENTRY WorkerLink;
//     PUSBPORT_ENDPOINT endpoint;
//     KIRQL OldIrql;
//     BOOLEAN IsAddEntry = FALSE;
//
//     DPRINT_CORE!("USBPORT_InvalidateEndpointHandler: Endpoint - %p, Type - %x\n",
//                 Endpoint,
//                 Type);
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     if (Endpoint)
//     {
//         WorkerLink = &Endpoint->WorkerLink;
//         KeAcquireSpinLock(&FdoExtension->EndpointListSpinLock, &OldIrql);
//         DPRINT_CORE!("USBPORT_InvalidateEndpointHandler: KeAcquireSpinLock \n");
//
//         if ((!WorkerLink->Flink || !WorkerLink->Blink) &&
//             !(Endpoint->Flags & ENDPOINT_FLAG_IDLE) &&
//             USBPORT_GetEndpointState(Endpoint) != USBPORT_ENDPOINT_CLOSED)
//         {
//             DPRINT_CORE!("USBPORT_InvalidateEndpointHandler: InsertTailList \n");
//             InsertTailList(&FdoExtension->WorkerList, WorkerLink);
//             IsAddEntry = TRUE;
//         }
//
//         KeReleaseSpinLock(&FdoExtension->EndpointListSpinLock, OldIrql);
//
//         if (Endpoint->Flags & ENDPOINT_FLAG_ROOTHUB_EP0)
//             Type = INVALIDATE_ENDPOINT_WORKER_THREAD;
//     }
//     else
//     {
//         KeAcquireSpinLock(&FdoExtension->EndpointListSpinLock, &OldIrql);
//
//         for (Entry = FdoExtension->EndpointList.Flink;
//              Entry && Entry != &FdoExtension->EndpointList;
//              Entry = Entry->Flink)
//         {
//             endpoint = CONTAINING_RECORD(Entry,
//                                          USBPORT_ENDPOINT,
//                                          EndpointLink);
//
//             if (!endpoint->WorkerLink.Flink || !endpoint->WorkerLink.Blink)
//             {
//                 if (!(endpoint->Flags & ENDPOINT_FLAG_IDLE) &&
//                     !(endpoint->Flags & ENDPOINT_FLAG_ROOTHUB_EP0) &&
//                     USBPORT_GetEndpointState(endpoint) != USBPORT_ENDPOINT_CLOSED)
//                 {
//                     DPRINT_CORE!("USBPORT_InvalidateEndpointHandler: InsertTailList \n");
//                     InsertTailList(&FdoExtension->WorkerList, &endpoint->WorkerLink);
//                     IsAddEntry = TRUE;
//                 }
//             }
//         }
//
//         KeReleaseSpinLock(&FdoExtension->EndpointListSpinLock, OldIrql);
//     }
//
//     if (FdoExtension->Flags & USBPORT_FLAG_HC_SUSPEND)
//     {
//         Type = INVALIDATE_ENDPOINT_WORKER_THREAD;
//     }
//     else if (IsAddEntry == FALSE && Type == INVALIDATE_ENDPOINT_INT_NEXT_SOF)
//     {
//         Type = INVALIDATE_ENDPOINT_ONLY;
//     }
//
//     switch (Type)
//     {
//         case INVALIDATE_ENDPOINT_WORKER_THREAD:
//             USBPORT_SignalWorkerThread(FdoDevice);
//             break;
//
//         case INVALIDATE_ENDPOINT_WORKER_DPC:
//             KeInsertQueueDpc(&FdoExtension->WorkerRequestDpc, NULL, NULL);
//             break;
//
//         case INVALIDATE_ENDPOINT_INT_NEXT_SOF:
//             KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//             Packet->InterruptNextSOF(FdoExtension->MiniPortExt);
//             KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//             break;
//     }
// }
//
// ULONG
// NTAPI
// USBPORT_DmaEndpointPaused(IN PDEVICE_OBJECT FdoDevice,
//                           IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PLIST_ENTRY Entry;
//     PUSBPORT_TRANSFER Transfer;
//     PURB Urb;
//     ULONG Frame;
//     ULONG CurrentFrame;
//     ULONG CompletedLen = 0;
//     KIRQL OldIrql;
//
//     DPRINT_CORE!("USBPORT_DmaEndpointPaused \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     Entry = Endpoint->TransferList.Flink;
//
//     if (Entry == &Endpoint->TransferList)
//         return USBPORT_ENDPOINT_ACTIVE;
//
//     while (Entry && Entry != &Endpoint->TransferList)
//     {
//         Transfer = CONTAINING_RECORD(Entry,
//                                      USBPORT_TRANSFER,
//                                      TransferLink);
//
//         if (Transfer->Flags & (TRANSFER_FLAG_CANCELED | TRANSFER_FLAG_ABORTED))
//         {
//             if (Transfer->Flags & TRANSFER_FLAG_ISO &&
//                 Transfer->Flags & TRANSFER_FLAG_SUBMITED &&
//                 !(Endpoint->Flags & ENDPOINT_FLAG_NUKE))
//             {
//                 Urb = Transfer->Urb;
//
//                 Frame = Urb->UrbIsochronousTransfer.StartFrame +
//                         Urb->UrbIsochronousTransfer.NumberOfPackets;
//
//                 KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//                 CurrentFrame = Packet->Get32BitFrameNumber(FdoExtension->MiniPortExt);
//                 KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//
//                 if (Frame + 1 > CurrentFrame)
//                 {
//                     return USBPORT_GetEndpointState(Endpoint);
//                 }
//             }
//
//             if ((Transfer->Flags & TRANSFER_FLAG_SUBMITED) &&
//                  !(Endpoint->Flags & ENDPOINT_FLAG_NUKE))
//             {
//                 KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//
//                 Packet->AbortTransfer(FdoExtension->MiniPortExt,
//                                       Endpoint + 1,
//                                       Transfer->MiniportTransfer,
//                                       &CompletedLen);
//
//                 KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//
//                 if (Transfer->Flags & TRANSFER_FLAG_ISO)
//                 {
//                     DPRINT1("USBPORT_DmaEndpointActive: FIXME call USBPORT_FlushIsoTransfer\n");
//                     ASSERT(FALSE); //USBPORT_FlushIsoTransfer();
//                 }
//                 else
//                 {
//                     Transfer->CompletedTransferLen = CompletedLen;
//                 }
//             }
//
//             RemoveEntryList(&Transfer->TransferLink);
//             Entry = Transfer->TransferLink.Flink;
//
//             if (Transfer->Flags & TRANSFER_FLAG_SPLITED)
//             {
//                 USBPORT_CancelSplitTransfer(Transfer);
//             }
//             else
//             {
//                 InsertTailList(&Endpoint->CancelList, &Transfer->TransferLink);
//             }
//         }
//         else
//         {
//             Entry = Transfer->TransferLink.Flink;
//         }
//     }
//
//     return USBPORT_ENDPOINT_ACTIVE;
// }
//
// ULONG
// NTAPI
// USBPORT_DmaEndpointActive(IN PDEVICE_OBJECT FdoDevice,
//                           IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     PLIST_ENTRY Entry;
//     PUSBPORT_TRANSFER Transfer;
//     LARGE_INTEGER TimeOut;
//     MPSTATUS MpStatus;
//     KIRQL OldIrql;
//
//     DPRINT_CORE!("USBPORT_DmaEndpointActive \n");
//
//     FdoExtension = FdoDevice->DeviceExtension;
//
//     Entry = Endpoint->TransferList.Flink;
//
//     while (Entry && Entry != &Endpoint->TransferList)
//     {
//         Transfer = CONTAINING_RECORD(Entry,
//                                      USBPORT_TRANSFER,
//                                      TransferLink);
//
//         if (!(Transfer->Flags & TRANSFER_FLAG_SUBMITED) &&
//              !(Endpoint->Flags & ENDPOINT_FLAG_NUKE))
//         {
//             KeAcquireSpinLock(&FdoExtension->MiniportSpinLock, &OldIrql);
//
//             Packet = &FdoExtension->MiniPortInterface->Packet;
//
//             if (Transfer->Flags & TRANSFER_FLAG_ISO)
//             {
//                 DPRINT1("USBPORT_DmaEndpointActive: FIXME call SubmitIsoTransfer\n");
//
//                 MpStatus = Packet->SubmitIsoTransfer(FdoExtension->MiniPortExt,
//                                                      Endpoint + 1,
//                                                      &Transfer->TransferParameters,
//                                                      Transfer->MiniportTransfer,
//                                                      NULL);//&Transfer->IsoTransferParameters);
//             }
//             else
//             {
//                 MpStatus = Packet->SubmitTransfer(FdoExtension->MiniPortExt,
//                                                   Endpoint + 1,
//                                                   &Transfer->TransferParameters,
//                                                   Transfer->MiniportTransfer,
//                                                   &Transfer->SgList);
//             }
//
//             KeReleaseSpinLock(&FdoExtension->MiniportSpinLock, OldIrql);
//
//             if (MpStatus)
//             {
//                 if ((MpStatus != MP_STATUS_FAILURE) && Transfer->Flags & TRANSFER_FLAG_ISO)
//                 {
//                     DPRINT1("USBPORT_DmaEndpointActive: FIXME call USBPORT_ErrorCompleteIsoTransfer\n");
//                     ASSERT(FALSE); //USBPORT_ErrorCompleteIsoTransfer();
//                 }
//
//                 return USBPORT_ENDPOINT_ACTIVE;
//             }
//
//             Transfer->Flags |= TRANSFER_FLAG_SUBMITED;
//             KeQuerySystemTime(&Transfer->Time);
//
//             TimeOut.QuadPart = 10000 * Transfer->TimeOut;
//             Transfer->Time.QuadPart += TimeOut.QuadPart;
//         }
//
//         if (Transfer->Flags & (TRANSFER_FLAG_CANCELED | TRANSFER_FLAG_ABORTED))
//         {
//             return USBPORT_ENDPOINT_PAUSED;
//         }
//
//         Entry = Transfer->TransferLink.Flink;
//     }
//
//     return USBPORT_ENDPOINT_ACTIVE;
// }
//
// VOID
// NTAPI
// USBPORT_DmaEndpointWorker(IN PUSBPORT_ENDPOINT Endpoint)
// {
//     PDEVICE_OBJECT FdoDevice;
//     ULONG PrevState;
//     ULONG EndpointState;
//     BOOLEAN IsPaused = FALSE;
//
//     DPRINT_CORE!("USBPORT_DmaEndpointWorker ... \n");
//
//     FdoDevice = Endpoint->FdoDevice;
//
//     KeAcquireSpinLock(&Endpoint->EndpointSpinLock, &Endpoint->EndpointOldIrql);
//
//     PrevState = USBPORT_GetEndpointState(Endpoint);
//
//     if (PrevState == USBPORT_ENDPOINT_PAUSED)
//     {
//         EndpointState = USBPORT_DmaEndpointPaused(FdoDevice, Endpoint);
//     }
//     else if (PrevState == USBPORT_ENDPOINT_ACTIVE)
//     {
//         EndpointState = USBPORT_DmaEndpointActive(FdoDevice, Endpoint);
//     }
//     else
//     {
// #ifndef NDEBUG_USBPORT_CORE
//         DPRINT1("USBPORT_DmaEndpointWorker: DbgBreakPoint. EndpointState - %x\n",
//                 EndpointState);
//         DbgBreakPoint();
// #endif
//         EndpointState = USBPORT_ENDPOINT_UNKNOWN;
//     }
//
//     KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//
//     USBPORT_FlushCancelList(Endpoint);
//
//     KeAcquireSpinLock(&Endpoint->EndpointSpinLock, &Endpoint->EndpointOldIrql);
//
//     if (EndpointState == PrevState)
//     {
//         if (EndpointState == USBPORT_ENDPOINT_PAUSED)
//         {
//             IsPaused = TRUE;
//         }
//     }
//     else
//     {
//         USBPORT_SetEndpointState(Endpoint, EndpointState);
//     }
//
//     KeReleaseSpinLock(&Endpoint->EndpointSpinLock, Endpoint->EndpointOldIrql);
//
//     if (IsPaused)
//     {
//        USBPORT_InvalidateEndpointHandler(FdoDevice,
//                                          Endpoint,
//                                          INVALIDATE_ENDPOINT_WORKER_THREAD);
//     }
//
//     DPRINT_CORE!("USBPORT_DmaEndpointWorker exit \n");
// }
//
// BOOLEAN
// NTAPI
// USBPORT_EndpointWorker(IN PUSBPORT_ENDPOINT Endpoint,
//                        IN BOOLEAN LockNotChecked)
// {
//     PDEVICE_OBJECT FdoDevice;
//     PUSBPORT_DEVICE_EXTENSION FdoExtension;
//     PUSBPORT_REGISTRATION_PACKET Packet;
//     ULONG EndpointState;
//
//     DPRINT_CORE!("USBPORT_EndpointWorker: Endpoint - %p, LockNotChecked - %x\n",
//            Endpoint,
//            LockNotChecked);
//
//     FdoDevice = Endpoint->FdoDevice;
//     FdoExtension = FdoDevice->DeviceExtension;
//     Packet = &FdoExtension->MiniPortInterface->Packet;
//
//     if (LockNotChecked == FALSE)
//     {
//         if (InterlockedIncrement(&Endpoint->LockCounter))
//         {
//             InterlockedDecrement(&Endpoint->LockCounter);
//             DPRINT_CORE!("USBPORT_EndpointWorker: LockCounter > 0\n");
//             return TRUE;
//         }
//     }
//
//     ASSERT(KeGetCurrentIrql() == DISPATCH_LEVEL);
//
//     KeAcquireSpinLockAtDpcLevel(&Endpoint->EndpointSpinLock);
//
//     if (USBPORT_GetEndpointState(Endpoint) == USBPORT_ENDPOINT_CLOSED)
//     {
//         KeReleaseSpinLockFromDpcLevel(&Endpoint->EndpointSpinLock);
//         InterlockedDecrement(&Endpoint->LockCounter);
//         DPRINT_CORE!("USBPORT_EndpointWorker: State == USBPORT_ENDPOINT_CLOSED. return FALSE\n");
//         return FALSE;
//     }
//
//     if ((Endpoint->Flags & (ENDPOINT_FLAG_ROOTHUB_EP0 | ENDPOINT_FLAG_NUKE)) == 0)
//     {
//         KeAcquireSpinLockAtDpcLevel(&FdoExtension->MiniportSpinLock);
//         Packet->PollEndpoint(FdoExtension->MiniPortExt, Endpoint + 1);
//         KeReleaseSpinLockFromDpcLevel(&FdoExtension->MiniportSpinLock);
//     }
//
//     EndpointState = USBPORT_GetEndpointState(Endpoint);
//
//     if (EndpointState == USBPORT_ENDPOINT_REMOVE)
//     {
//         KeAcquireSpinLockAtDpcLevel(&Endpoint->StateChangeSpinLock);
//         Endpoint->StateLast = USBPORT_ENDPOINT_CLOSED;
//         Endpoint->StateNext = USBPORT_ENDPOINT_CLOSED;
//         KeReleaseSpinLockFromDpcLevel(&Endpoint->StateChangeSpinLock);
//
//         KeReleaseSpinLockFromDpcLevel(&Endpoint->EndpointSpinLock);
//
//         KeAcquireSpinLockAtDpcLevel(&FdoExtension->EndpointListSpinLock);
//
//         ExInterlockedInsertTailList(&FdoExtension->EndpointClosedList,
//                                     &Endpoint->CloseLink,
//                                     &FdoExtension->EndpointClosedSpinLock);
//
//         KeReleaseSpinLockFromDpcLevel(&FdoExtension->EndpointListSpinLock);
//
//         InterlockedDecrement(&Endpoint->LockCounter);
//         DPRINT_CORE!("USBPORT_EndpointWorker: State == USBPORT_ENDPOINT_REMOVE. return FALSE\n");
//         return FALSE;
//     }
//
//     if (!IsListEmpty(&Endpoint->PendingTransferList) ||
//         !IsListEmpty(&Endpoint->TransferList) ||
//         !IsListEmpty(&Endpoint->CancelList))
//     {
//         KeReleaseSpinLockFromDpcLevel(&Endpoint->EndpointSpinLock);
//
//         EndpointState = USBPORT_GetEndpointState(Endpoint);
//
//         KeAcquireSpinLockAtDpcLevel(&Endpoint->StateChangeSpinLock);
//         if (EndpointState == Endpoint->StateNext)
//         {
//             KeReleaseSpinLockFromDpcLevel(&Endpoint->StateChangeSpinLock);
//
//             if (Endpoint->EndpointWorker)
//             {
//                 USBPORT_DmaEndpointWorker(Endpoint);
//             }
//             else
//             {
//                 USBPORT_RootHubEndpointWorker(Endpoint);
//             }
//
//             USBPORT_FlushAbortList(Endpoint);
//
//             InterlockedDecrement(&Endpoint->LockCounter);
//             DPRINT_CORE!("USBPORT_EndpointWorker: return FALSE\n");
//             return FALSE;
//         }
//
//         KeReleaseSpinLockFromDpcLevel(&Endpoint->StateChangeSpinLock);
//         InterlockedDecrement(&Endpoint->LockCounter);
//
//         DPRINT_CORE!("USBPORT_EndpointWorker: return TRUE\n");
//         return TRUE;
//     }
//
//     KeReleaseSpinLockFromDpcLevel(&Endpoint->EndpointSpinLock);
//
//     USBPORT_FlushAbortList(Endpoint);
//
//     InterlockedDecrement(&Endpoint->LockCounter);
//     DPRINT_CORE!("USBPORT_EndpointWorker: return FALSE\n");
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
