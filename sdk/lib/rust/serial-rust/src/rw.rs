use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, IoAllocateWorkItem, IoFreeWorkItem, IoQueueWorkItem, KTIMER, KeCancelTimer, KeInitializeTimer, KeSetTimer, KeWaitForMultipleObjects, LARGE_INTEGER, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIO_WORKITEM, PIRP};
// IoGetCurrentIrpStackLocation, IoMarkIrpPending come from crate::glue (ntkrnl_sys::inline::tier1)
use core::ffi::c_void;

extern "C" {
    fn DbgPrintEx( ComponentId: ULONG, Level: ULONG, Format: PCSTR, ... ) -> ULONG;
    fn IsCircularBufferEmpty(CircularBuffer: PVOID) -> u8;
    fn PopCircularBufferEntry(CircularBuffer: PVOID, Byte: *mut UCHAR) -> u32;
    fn PushCircularBufferEntry(Buffer: PVOID, Entry: UCHAR) -> NTSTATUS;
}
unsafe extern "system" {
    fn SerialSendByte(Dpc: PVOID, Context: PVOID, Arg1: PVOID, Arg2: PVOID);
}

// ---- file-scope preamble (translated) ----
// static IO_WORKITEM_ROUTINE SerialReadWorkItem; — forward declaration of a function; no data declaration emitted.

unsafe fn SerialGetUserBuffer(mut Irp: PIRP) -> PVOID {
    if Irp.is_null() {
        RtlAssert(
            b"Irp\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/serial/serial/rw.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            19 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if !(*Irp).MdlAddress.is_null() {
        return (*Irp).MdlAddress as PVOID;
    } else {
        return (*Irp).AssociatedIrp.SystemBuffer;
    };
}
unsafe fn ReadBytes(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
    mut WorkItemData: PWORKITEM_DATA,
) {

    const DPFLTR_SERIAL_ID: u32 = 39;
    const DPFLTR_INFO_LEVEL: u32 = 3;
    const DPFLTR_TRACE_LEVEL: u32 = 2;

    let mut DeviceExtension: PSERIAL_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut Buffer: PUCHAR = core::ptr::null_mut();
    let mut ReceivedByte: UCHAR = 0;
    let mut TotalTimeoutTimer: KTIMER = core::mem::zeroed();
    let mut Irql: KIRQL = 0;
    let mut ObjectCount: ULONG = 0;
    let mut ObjectsArray: [PVOID; 2] = [core::ptr::null_mut(); 2];
    let mut Information: ULONG_PTR = 0 as ULONG_PTR;
    let mut Status: NTSTATUS = 0;

    if DeviceObject.is_null() {
        RtlAssert(
            b"DeviceObject\0".as_ptr() as PVOID,
            b"drivers/serial/serial/rw.c\0".as_ptr() as PVOID,
            44 as ULONG,
            core::ptr::null_mut(),
        );
    } else {
    };
    if WorkItemData.is_null() {
        RtlAssert(
            b"WorkItemData\0".as_ptr() as PVOID,
            b"drivers/serial/serial/rw.c\0".as_ptr() as PVOID,
            45 as ULONG,
            core::ptr::null_mut(),
        );
    } else {
    };

    DeviceExtension = (*DeviceObject).DeviceExtension as PSERIAL_DEVICE_EXTENSION;
    Length = (*IoGetCurrentIrpStackLocation(Irp)).Parameters.Read.Length;
    Buffer = SerialGetUserBuffer(Irp) as PUCHAR;

    DbgPrintEx(
        DPFLTR_SERIAL_ID as ULONG,
        DPFLTR_INFO_LEVEL as ULONG,
        b"(%s:%d) UseIntervalTimeout = %s, IntervalTimeout = %lu\n\0".as_ptr() as PCSTR,
        b"drivers/serial/serial/rw.c\0".as_ptr() as *const core::ffi::c_char,
        53 as core::ffi::c_int,
        if (*WorkItemData).UseIntervalTimeout as core::ffi::c_int != 0 {
            b"YES\0".as_ptr() as *const core::ffi::c_char
        } else {
            b"NO\0".as_ptr() as *const core::ffi::c_char
        },
        if (*WorkItemData).UseIntervalTimeout as core::ffi::c_int != 0 {
            (*WorkItemData).IntervalTimeout.QuadPart as core::ffi::c_longlong
        } else {
            0 as core::ffi::c_longlong
        },
    );
    DbgPrintEx(
        DPFLTR_SERIAL_ID as ULONG,
        DPFLTR_INFO_LEVEL as ULONG,
        b"(%s:%d) UseTotalTimeout = %s\n\0".as_ptr() as PCSTR,
        b"drivers/serial/serial/rw.c\0".as_ptr() as *const core::ffi::c_char,
        55 as core::ffi::c_int,
        if (*WorkItemData).UseTotalTimeout as core::ffi::c_int != 0 {
            b"YES\0".as_ptr() as *const core::ffi::c_char
        } else {
            b"NO\0".as_ptr() as *const core::ffi::c_char
        },
    );

    ObjectCount = 1 as ULONG;
    ObjectsArray[0 as usize] = core::ptr::addr_of_mut!((*DeviceExtension).InputBufferNotEmpty) as PVOID;
    if (*WorkItemData).UseTotalTimeout != 0 {
        KeInitializeTimer(core::ptr::addr_of_mut!(TotalTimeoutTimer));
        KeSetTimer(
            core::ptr::addr_of_mut!(TotalTimeoutTimer),
            (*WorkItemData).TotalTimeoutTime,
            core::ptr::null_mut(),
        );
        ObjectsArray[ObjectCount as usize] = core::ptr::addr_of_mut!(TotalTimeoutTimer) as PVOID;
        ObjectCount = ObjectCount.wrapping_add(1);
    }

    loop {
        Irql = KeAcquireSpinLockRaiseToDpc(core::ptr::addr_of_mut!((*DeviceExtension).InputBufferLock) as PKSPIN_LOCK as PVOID);
        while IsCircularBufferEmpty(core::ptr::addr_of_mut!((*DeviceExtension).InputBuffer) as PVOID) == 0
            && Length > 0 as core::ffi::c_ulong
        {
            PopCircularBufferEntry(
                core::ptr::addr_of_mut!((*DeviceExtension).InputBuffer) as PVOID,
                core::ptr::addr_of_mut!(ReceivedByte),
            );
            DbgPrintEx(
                DPFLTR_SERIAL_ID as ULONG,
                DPFLTR_INFO_LEVEL as ULONG,
                b"(%s:%d) Reading byte from buffer: 0x%02x\n\0".as_ptr() as PCSTR,
                b"drivers/serial/serial/rw.c\0".as_ptr() as *const core::ffi::c_char,
                76 as core::ffi::c_int,
                ReceivedByte as core::ffi::c_int,
            );
            let fresh1 = Information;
            Information = Information.wrapping_add(1);
            *Buffer.offset(fresh1 as isize) = ReceivedByte as core::ffi::c_uchar;
            Length = Length.wrapping_sub(1);
        }
        KeClearEvent(core::ptr::addr_of_mut!((*DeviceExtension).InputBufferNotEmpty) as PVOID);
        KeReleaseSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).InputBufferLock) as PKSPIN_LOCK, Irql);

        if Length == 0 as core::ffi::c_ulong {
            DbgPrintEx(
                DPFLTR_SERIAL_ID as ULONG,
                DPFLTR_INFO_LEVEL as ULONG,
                b"(%s:%d) All bytes read\n\0".as_ptr() as PCSTR,
                b"drivers/serial/serial/rw.c\0".as_ptr() as *const core::ffi::c_char,
                86 as core::ffi::c_int,
            );
            break;
        } else if (*WorkItemData).DontWait as core::ffi::c_int != 0
            && !((*WorkItemData).ReadAtLeastOneByte as core::ffi::c_int != 0
                && Information == 0 as ULONG_PTR)
        {
            DbgPrintEx(
                DPFLTR_SERIAL_ID as ULONG,
                DPFLTR_INFO_LEVEL as ULONG,
                b"(%s:%d) Buffer empty. Don't wait more bytes\n\0".as_ptr() as PCSTR,
                b"drivers/serial/serial/rw.c\0".as_ptr() as *const core::ffi::c_char,
                93 as core::ffi::c_int,
            );
            break;
        } else {
            Status = KeWaitForMultipleObjects(
                ObjectCount,
                core::ptr::addr_of_mut!(ObjectsArray) as *mut PVOID,
                WaitAny,
                _KWAIT_REASON_Executive,
                0i32 as core::ffi::c_int as KPROCESSOR_MODE,
                0 as BOOLEAN,
                if (*WorkItemData).UseIntervalTimeout as core::ffi::c_int != 0
                    && Information > 0 as ULONG_PTR
                {
                    core::ptr::addr_of_mut!((*WorkItemData).IntervalTimeout)
                } else {
                    core::ptr::null_mut()
                },
                core::ptr::null_mut(),
            );
            if !(Status == STATUS_TIMEOUT || Status == STATUS_WAIT_1) {
                continue;
            }
            DbgPrintEx(
                DPFLTR_SERIAL_ID as ULONG,
                DPFLTR_TRACE_LEVEL as ULONG,
                b"(%s:%d) Timeout when reading bytes. Status = 0x%08lx\n\0".as_ptr() as PCSTR,
                b"drivers/serial/serial/rw.c\0".as_ptr() as *const core::ffi::c_char,
                110 as core::ffi::c_int,
                Status,
            );
            break;
        }
    }

    if (*WorkItemData).UseTotalTimeout != 0 {
        KeCancelTimer(core::ptr::addr_of_mut!(TotalTimeoutTimer));
    }

    (*Irp).IoStatus.Information = Information as _;
    if Information == 0 as ULONG_PTR {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_TIMEOUT;
    } else {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    };
}

unsafe extern "system" fn SerialReadWorkItem(
    mut DeviceObject: PDEVICE_OBJECT,
    mut pWorkItemData: PVOID,
) {
    const DPFLTR_SERIAL_ID: u32 = 39;
    const DPFLTR_TRACE_LEVEL: u32 = 2;
    const SERIAL_TAG: u32 = 0x6c726553; // 'lreS'

    let mut WorkItemData: PWORKITEM_DATA = core::ptr::null_mut();
    let mut Irp: PIRP = core::ptr::null_mut();

    DbgPrintEx(
        DPFLTR_SERIAL_ID as ULONG,
        DPFLTR_TRACE_LEVEL as ULONG,
        b"(%s:%d) SerialReadWorkItem() called\n\0" as *const u8 as PCSTR,
        b"rw.c\0" as *const u8 as *const ::core::ffi::c_char,
        134 as ::core::ffi::c_int,
    );

    WorkItemData = pWorkItemData as PWORKITEM_DATA;
    Irp = (*WorkItemData).Irp;

    ReadBytes(DeviceObject, Irp, WorkItemData);

    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);

    IoFreeWorkItem((*WorkItemData).IoWorkItem);
    ExFreePoolWithTag(pWorkItemData, SERIAL_TAG as ULONG);
}

#[no_mangle]
pub unsafe extern "system" fn SerialRead(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {

    const DPFLTR_SERIAL_ID: u32 = 39;
    const DPFLTR_TRACE_LEVEL: u32 = 2;
    const DPFLTR_INFO_LEVEL: u32 = 3;
    const INFINITE: ULONG = 0xFFFFFFFF;
    const TRUE: i32 = 1;
    const SERIAL_TAG: ULONG = 0x6c726553; // 'Serl'

    let mut Stack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut DeviceExtension: PSERIAL_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Length: ULONG;
    let mut Buffer: PUCHAR;
    let mut WorkItemData: PWORKITEM_DATA = core::ptr::null_mut();
    let mut WorkItem: PIO_WORKITEM;
    let mut Status: NTSTATUS = 0;

    DbgPrintEx(
        DPFLTR_SERIAL_ID as ULONG,
        DPFLTR_TRACE_LEVEL as ULONG,
        b"(%s:%d) IRP_MJ_READ\n\0".as_ptr() as PCSTR,
        b"rw.c\0".as_ptr() as *const core::ffi::c_char,
        160 as core::ffi::c_int,
    );

    Stack = IoGetCurrentIrpStackLocation(Irp);
    Length = (*Stack).Parameters.Read.Length;
    Buffer = SerialGetUserBuffer(Irp) as PUCHAR;
    DeviceExtension = (*DeviceObject).DeviceExtension as PSERIAL_DEVICE_EXTENSION;

    if (*Stack).Parameters.Read.ByteOffset.QuadPart != 0 || Buffer.is_null() {
        Status = STATUS_INVALID_PARAMETER;
    } else if Length == 0 {
        Status = STATUS_SUCCESS;
    } else {
        WorkItemData = ExAllocatePoolWithTag(
            PagedPool,
            core::mem::size_of::<WORKITEM_DATA>() as SIZE_T,
            SERIAL_TAG,
        ) as PWORKITEM_DATA;
        if WorkItemData.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
        } else {
            RtlZeroMemory(
                WorkItemData as *mut core::ffi::c_void,
                core::mem::size_of::<WORKITEM_DATA>(),
            );
            (*WorkItemData).Irp = Irp;

            if (*DeviceExtension).SerialTimeOuts.ReadIntervalTimeout == INFINITE
                && (*DeviceExtension).SerialTimeOuts.ReadTotalTimeoutMultiplier == INFINITE
                && (*DeviceExtension).SerialTimeOuts.ReadTotalTimeoutConstant > 0
                && (*DeviceExtension).SerialTimeOuts.ReadTotalTimeoutConstant < INFINITE
            {
                (*WorkItemData).DontWait = TRUE as BOOLEAN;
                (*WorkItemData).ReadAtLeastOneByte = TRUE as BOOLEAN;
            } else if (*DeviceExtension).SerialTimeOuts.ReadIntervalTimeout == INFINITE
                && (*DeviceExtension).SerialTimeOuts.ReadTotalTimeoutConstant == 0
                && (*DeviceExtension).SerialTimeOuts.ReadTotalTimeoutMultiplier == 0
            {
                (*WorkItemData).DontWait = TRUE as BOOLEAN;
            } else {
                if (*DeviceExtension).SerialTimeOuts.ReadIntervalTimeout != 0 {
                    (*WorkItemData).UseIntervalTimeout = TRUE as BOOLEAN;
                    (*WorkItemData).IntervalTimeout.QuadPart =
                        (*DeviceExtension).SerialTimeOuts.ReadIntervalTimeout as LONGLONG;
                }
                if (*DeviceExtension).SerialTimeOuts.ReadTotalTimeoutMultiplier != 0
                    || (*DeviceExtension).SerialTimeOuts.ReadTotalTimeoutConstant != 0
                {
                    let mut TotalTimeout: ULONG;
                    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();

                    (*WorkItemData).UseTotalTimeout = TRUE as BOOLEAN;
                    TotalTimeout = (*DeviceExtension)
                        .SerialTimeOuts
                        .ReadTotalTimeoutConstant
                        .wrapping_add(
                            (*DeviceExtension)
                                .SerialTimeOuts
                                .ReadTotalTimeoutMultiplier
                                .wrapping_mul(Length),
                        );
                    *(core::ptr::addr_of_mut!(SystemTime) as *mut u64) =
                        *(0xFFFFF78000000014u64 as *mut u64);
                    (*WorkItemData).TotalTimeoutTime.QuadPart = (SystemTime.QuadPart
                        + (TotalTimeout.wrapping_mul(10000)) as core::ffi::c_longlong)
                        as LONGLONG;
                }
            }

            WorkItem = IoAllocateWorkItem(DeviceObject);
            if !WorkItem.is_null() {
                (*WorkItemData).IoWorkItem = WorkItem;
                IoMarkIrpPending(Irp);
                IoQueueWorkItem(
                    WorkItem,
                    Some(SerialReadWorkItem),
                    DelayedWorkQueue,
                    WorkItemData as PVOID,
                );
                return STATUS_PENDING;
            }

            DbgPrintEx(
                DPFLTR_SERIAL_ID as ULONG,
                DPFLTR_INFO_LEVEL as ULONG,
                b"(%s:%d) Insufficient resources\n\0".as_ptr() as PCSTR,
                b"rw.c\0".as_ptr() as *const core::ffi::c_char,
                240 as core::ffi::c_int,
            );
            Status = IoAcquireRemoveLockEx(
                core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                ULongToPtr((*DeviceExtension).ComPort as ULONG),
                b"rw.c\0".as_ptr() as PCSTR,
                241 as ULONG,
                core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
            );
            if !NT_SUCCESS(Status) {
                ExFreePoolWithTag(WorkItemData as PVOID, SERIAL_TAG);
            } else {
                ReadBytes(DeviceObject, Irp, WorkItemData);
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
                IoReleaseRemoveLockEx(
                    core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
                    ULongToPtr((*DeviceExtension).ComPort as ULONG),
                    core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
                );
            }
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn SerialWrite(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let mut Stack: PIO_STACK_LOCATION;
    let mut DeviceExtension: PSERIAL_DEVICE_EXTENSION;
    let mut Length: ULONG;
    let mut Information: ULONG_PTR = 0 as ULONG_PTR;
    let mut Buffer: PUCHAR;
    let mut Irql: KIRQL = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    // TRACE_!(SERIAL, "IRP_MJ_WRITE\n") → DbgPrintEx(DPFLTR_SERIAL_ID, DPFLTR_TRACE_LEVEL, ...)
    DbgPrintEx(
        DPFLTR_SERIAL_ID,
        DPFLTR_TRACE_LEVEL,
        b"IRP_MJ_WRITE\n\0".as_ptr() as PCSTR,
    );

    /* FIXME: pend operation if possible */
    /* FIXME: use write timeouts */

    Stack = IoGetCurrentIrpStackLocation(Irp);
    Length = (*Stack).Parameters.Write.Length;
    Buffer = SerialGetUserBuffer(Irp) as PUCHAR;
    DeviceExtension = (*DeviceObject).DeviceExtension as PSERIAL_DEVICE_EXTENSION;

    'ByeBye: loop {
        if (*Stack).Parameters.Write.ByteOffset.QuadPart != 0 || Buffer.is_null() {
            Status = STATUS_INVALID_PARAMETER;
            break 'ByeBye;
        }

        Status = IoAcquireRemoveLock(
            core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
            ULongToPtr((*DeviceExtension).ComPort),
        );
        if !NT_SUCCESS(Status) {
            break 'ByeBye;
        }

        /* push bytes into output buffer */
        KeAcquireSpinLock(
            core::ptr::addr_of_mut!((*DeviceExtension).OutputBufferLock) as PKSPIN_LOCK,
            core::ptr::addr_of_mut!(Irql),
        );
        while Information < Length as ULONG_PTR {
            Status = PushCircularBufferEntry(
                core::ptr::addr_of_mut!((*DeviceExtension).OutputBuffer) as PVOID,
                *Buffer.offset(Information as isize),
            );
            if !NT_SUCCESS(Status) {
                if Status == STATUS_BUFFER_TOO_SMALL {
                    KeReleaseSpinLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).OutputBufferLock) as PKSPIN_LOCK,
                        Irql,
                    );
                    SerialSendByte(
                        core::ptr::null_mut(),
                        DeviceExtension as PVOID,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                    KeAcquireSpinLock(
                        core::ptr::addr_of_mut!((*DeviceExtension).OutputBufferLock) as PKSPIN_LOCK,
                        core::ptr::addr_of_mut!(Irql),
                    );
                    continue;
                } else {
                    // WARN_(SERIAL, "Buffer overrun on COM%lu\n", ComPort) → DbgPrintEx
                    DbgPrintEx(
                        DPFLTR_SERIAL_ID,
                        DPFLTR_TRACE_LEVEL,
                        b"Buffer overrun on COM%lu\n\0".as_ptr() as PCSTR,
                        (*DeviceExtension).ComPort,
                    );
                    (*DeviceExtension).SerialPerfStats.BufferOverrunErrorCount += 1;
                    break;
                }
            }
            Information += 1;
        }
        KeReleaseSpinLock(
            core::ptr::addr_of_mut!((*DeviceExtension).OutputBufferLock) as PKSPIN_LOCK,
            Irql,
        );
        IoReleaseRemoveLock(
            core::ptr::addr_of_mut!((*DeviceExtension).RemoveLock) as PVOID,
            ULongToPtr((*DeviceExtension).ComPort),
        );

        /* send bytes */
        SerialSendByte(
            core::ptr::null_mut(),
            DeviceExtension as PVOID,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        break 'ByeBye;
    }

    (*Irp).IoStatus.Information = Information as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Serial port driver
//  * FILE:            drivers/dd/serial/create.c
//  * PURPOSE:         Serial IRP_MJ_READ/IRP_MJ_WRITE operations
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "serial.h"
//
// #include <debug.h>
//
// static IO_WORKITEM_ROUTINE SerialReadWorkItem;
//
// static PVOID
// SerialGetUserBuffer(IN PIRP Irp)
// {
// 	ASSERT(Irp);
//
// 	if (Irp->MdlAddress)
// 		return Irp->MdlAddress;
// 	else
// 		return Irp->AssociatedIrp.SystemBuffer;
// }
//
// static VOID
// ReadBytes(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp,
// 	PWORKITEM_DATA WorkItemData)
// {
// 	PSERIAL_DEVICE_EXTENSION DeviceExtension;
// 	ULONG Length;
// 	PUCHAR Buffer;
// 	UCHAR ReceivedByte;
// 	KTIMER TotalTimeoutTimer;
// 	KIRQL Irql;
// 	ULONG ObjectCount;
// 	PVOID ObjectsArray[2];
// 	ULONG_PTR Information = 0;
// 	NTSTATUS Status;
//
// 	ASSERT(DeviceObject);
// 	ASSERT(WorkItemData);
//
// 	DeviceExtension = (PSERIAL_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
// 	Length = IoGetCurrentIrpStackLocation(Irp)->Parameters.Read.Length;
// 	Buffer = SerialGetUserBuffer(Irp);
//
// 	INFO_(SERIAL, "UseIntervalTimeout = %s, IntervalTimeout = %lu\n",
// 		WorkItemData->UseIntervalTimeout ? "YES" : "NO",
// 		WorkItemData->UseIntervalTimeout ? WorkItemData->IntervalTimeout.QuadPart : 0);
// 	INFO_(SERIAL, "UseTotalTimeout = %s\n",
// 		WorkItemData->UseTotalTimeout ? "YES" : "NO");
//
// 	ObjectCount = 1;
// 	ObjectsArray[0] = &DeviceExtension->InputBufferNotEmpty;
// 	if (WorkItemData->UseTotalTimeout)
// 	{
// 		KeInitializeTimer(&TotalTimeoutTimer);
// 		KeSetTimer(&TotalTimeoutTimer, WorkItemData->TotalTimeoutTime, NULL);
// 		ObjectsArray[ObjectCount] = &TotalTimeoutTimer;
// 		ObjectCount++;
// 	}
//
// 	/* while buffer is not fully filled */
// 	while (TRUE)
// 	{
// 		/* read already received bytes from buffer */
// 		KeAcquireSpinLock(&DeviceExtension->InputBufferLock, &Irql);
// 		while (!IsCircularBufferEmpty(&DeviceExtension->InputBuffer)
// 			&& Length > 0)
// 		{
// 			PopCircularBufferEntry(&DeviceExtension->InputBuffer, &ReceivedByte);
// 			INFO_(SERIAL, "Reading byte from buffer: 0x%02x\n", ReceivedByte);
//
// 			Buffer[Information++] = ReceivedByte;
// 			Length--;
// 		}
// 		KeClearEvent(&DeviceExtension->InputBufferNotEmpty);
// 		KeReleaseSpinLock(&DeviceExtension->InputBufferLock, Irql);
//
// 		if (Length == 0)
// 		{
// 			INFO_(SERIAL, "All bytes read\n");
// 			break;
// 		}
//
// 		if (WorkItemData->DontWait
// 			&& !(WorkItemData->ReadAtLeastOneByte && Information == 0))
// 		{
// 			INFO_(SERIAL, "Buffer empty. Don't wait more bytes\n");
// 			break;
// 		}
//
// 		Status = KeWaitForMultipleObjects(
// 			ObjectCount,
// 			ObjectsArray,
// 			WaitAny,
// 			Executive,
// 			KernelMode,
// 			FALSE,
// 			(WorkItemData->UseIntervalTimeout && Information > 0) ? &WorkItemData->IntervalTimeout : NULL,
// 			NULL);
//
// 		if (Status == STATUS_TIMEOUT /* interval timeout */
// 			|| Status == STATUS_WAIT_1) /* total timeout */
// 		{
// 			TRACE_!(SERIAL, "Timeout when reading bytes. Status = 0x%08lx\n", Status);
// 			break;
// 		}
// 	}
//
// 	/* stop total timeout timer */
// 	if (WorkItemData->UseTotalTimeout)
// 		KeCancelTimer(&TotalTimeoutTimer);
//
// 	Irp->IoStatus.Information = Information;
// 	if (Information == 0)
// 		Irp->IoStatus.Status = STATUS_TIMEOUT;
// 	else
// 		Irp->IoStatus.Status = STATUS_SUCCESS;
// }
//
// static VOID NTAPI
// SerialReadWorkItem(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PVOID pWorkItemData /* real type PWORKITEM_DATA */)
// {
// 	PWORKITEM_DATA WorkItemData;
// 	PIRP Irp;
//
// 	TRACE_!(SERIAL, "SerialReadWorkItem() called\n");
//
// 	WorkItemData = (PWORKITEM_DATA)pWorkItemData;
// 	Irp = WorkItemData->Irp;
//
// 	ReadBytes(DeviceObject, Irp, WorkItemData);
//
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
// 	IoFreeWorkItem(WorkItemData->IoWorkItem);
// 	ExFreePoolWithTag(pWorkItemData, SERIAL_TAG);
// }
//
// NTSTATUS NTAPI
// SerialRead(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PIO_STACK_LOCATION Stack;
// 	PSERIAL_DEVICE_EXTENSION DeviceExtension;
// 	ULONG Length;
// 	PUCHAR Buffer;
// 	PWORKITEM_DATA WorkItemData;
// 	PIO_WORKITEM WorkItem;
// 	NTSTATUS Status;
//
// 	TRACE_!(SERIAL, "IRP_MJ_READ\n");
//
// 	Stack = IoGetCurrentIrpStackLocation(Irp);
// 	Length = Stack->Parameters.Read.Length;
// 	Buffer = SerialGetUserBuffer(Irp);
// 	DeviceExtension = (PSERIAL_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
// 	if (Stack->Parameters.Read.ByteOffset.QuadPart != 0 || Buffer == NULL)
// 	{
// 		Status = STATUS_INVALID_PARAMETER;
// 		goto ByeBye;
// 	}
//
// 	if (Length == 0)
// 	{
// 		Status = STATUS_SUCCESS;
// 		goto ByeBye;
// 	}
//
// 	/* Allocate memory for parameters */
// 	WorkItemData = ExAllocatePoolWithTag(PagedPool, sizeof(WORKITEM_DATA), SERIAL_TAG);
// 	if (!WorkItemData)
// 	{
// 		Status = STATUS_INSUFFICIENT_RESOURCES;
// 		goto ByeBye;
// 	}
// 	RtlZeroMemory(WorkItemData, sizeof(WORKITEM_DATA));
// 	WorkItemData->Irp = Irp;
//
// 	/* Calculate time outs */
// 	if (DeviceExtension->SerialTimeOuts.ReadIntervalTimeout == INFINITE &&
// 		 DeviceExtension->SerialTimeOuts.ReadTotalTimeoutMultiplier == INFINITE &&
// 		 DeviceExtension->SerialTimeOuts.ReadTotalTimeoutConstant > 0 &&
// 		 DeviceExtension->SerialTimeOuts.ReadTotalTimeoutConstant < INFINITE)
// 	{
// 		/* read at least one byte, and at most bytes already received */
// 		WorkItemData->DontWait = TRUE;
// 		WorkItemData->ReadAtLeastOneByte = TRUE;
// 	}
// 	else if (DeviceExtension->SerialTimeOuts.ReadIntervalTimeout == INFINITE &&
// 	         DeviceExtension->SerialTimeOuts.ReadTotalTimeoutConstant == 0 &&
// 	         DeviceExtension->SerialTimeOuts.ReadTotalTimeoutMultiplier == 0)
// 	{
// 		/* read only bytes that are already in buffer */
// 		WorkItemData->DontWait = TRUE;
// 	}
// 	else
// 	{
// 		/* use timeouts */
// 		if (DeviceExtension->SerialTimeOuts.ReadIntervalTimeout != 0)
// 			{
// 				WorkItemData->UseIntervalTimeout = TRUE;
// 				WorkItemData->IntervalTimeout.QuadPart = DeviceExtension->SerialTimeOuts.ReadIntervalTimeout;
// 		}
// 		if (DeviceExtension->SerialTimeOuts.ReadTotalTimeoutMultiplier != 0 ||
// 			 DeviceExtension->SerialTimeOuts.ReadTotalTimeoutConstant != 0)
// 		{
// 			ULONG TotalTimeout;
// 			LARGE_INTEGER SystemTime;
//
// 			WorkItemData->UseTotalTimeout = TRUE;
// 			TotalTimeout = DeviceExtension->SerialTimeOuts.ReadTotalTimeoutConstant +
// 				DeviceExtension->SerialTimeOuts.ReadTotalTimeoutMultiplier * Length;
// 			KeQuerySystemTime(&SystemTime);
// 			WorkItemData->TotalTimeoutTime.QuadPart = SystemTime.QuadPart +
// 				TotalTimeout * 10000;
// 		}
// 	}
//
// 	/* Pend IRP */
// 	WorkItem = IoAllocateWorkItem(DeviceObject);
// 	if (WorkItem)
// 	{
// 		WorkItemData->IoWorkItem = WorkItem;
// 		IoMarkIrpPending(Irp);
// 		IoQueueWorkItem(WorkItem, SerialReadWorkItem, DelayedWorkQueue, WorkItemData);
// 		return STATUS_PENDING;
// 	}
//
// 	/* Insufficient resources, we can't pend the Irp */
// 	INFO_(SERIAL, "Insufficient resources\n");
// 	Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 	if (!NT_SUCCESS(Status))
// 	{
// 		ExFreePoolWithTag(WorkItemData, SERIAL_TAG);
// 		goto ByeBye;
// 	}
// 	ReadBytes(DeviceObject, Irp, WorkItemData);
// 	Status = Irp->IoStatus.Status;
//
// 	IoReleaseRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
//
// ByeBye:
// 	Irp->IoStatus.Status = Status;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return Status;
// }
//
// NTSTATUS NTAPI
// SerialWrite(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PIO_STACK_LOCATION Stack;
// 	PSERIAL_DEVICE_EXTENSION DeviceExtension;
// 	ULONG Length;
// 	ULONG_PTR Information = 0;
// 	PUCHAR Buffer;
// 	KIRQL Irql;
// 	NTSTATUS Status = STATUS_SUCCESS;
//
// 	TRACE_!(SERIAL, "IRP_MJ_WRITE\n");
//
// 	/* FIXME: pend operation if possible */
// 	/* FIXME: use write timeouts */
//
// 	Stack = IoGetCurrentIrpStackLocation(Irp);
// 	Length = Stack->Parameters.Write.Length;
// 	Buffer = SerialGetUserBuffer(Irp);
// 	DeviceExtension = (PSERIAL_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
// 	if (Stack->Parameters.Write.ByteOffset.QuadPart != 0 || Buffer == NULL)
// 	{
// 		Status = STATUS_INVALID_PARAMETER;
// 		goto ByeBye;
// 	}
//
// 	Status = IoAcquireRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
// 	if (!NT_SUCCESS(Status))
// 		goto ByeBye;
//
// 	/* push  bytes into output buffer */
// 	KeAcquireSpinLock(&DeviceExtension->OutputBufferLock, &Irql);
// 	while (Information < Length)
// 	{
// 		Status = PushCircularBufferEntry(&DeviceExtension->OutputBuffer, Buffer[Information]);
// 		if (!NT_SUCCESS(Status))
// 		{
// 			if (Status == STATUS_BUFFER_TOO_SMALL)
// 			{
// 				KeReleaseSpinLock(&DeviceExtension->OutputBufferLock, Irql);
// 				SerialSendByte(NULL, DeviceExtension, NULL, NULL);
// 				KeAcquireSpinLock(&DeviceExtension->OutputBufferLock, &Irql);
// 				continue;
// 			}
// 			else
// 			{
// 				WARN_(SERIAL, "Buffer overrun on COM%lu\n", DeviceExtension->ComPort);
// 				DeviceExtension->SerialPerfStats.BufferOverrunErrorCount++;
// 				break;
// 			}
// 		}
// 		Information++;
// 	}
// 	KeReleaseSpinLock(&DeviceExtension->OutputBufferLock, Irql);
// 	IoReleaseRemoveLock(&DeviceExtension->RemoveLock, ULongToPtr(DeviceExtension->ComPort));
//
// 	/* send bytes */
// 	SerialSendByte(NULL, DeviceExtension, NULL, NULL);
//
// ByeBye:
// 	Irp->IoStatus.Information = Information;
// 	Irp->IoStatus.Status = Status;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
