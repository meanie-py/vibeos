use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{KeInsertQueueDpc, PDEVICE_OBJECT, PIRP, PKINTERRUPT};
// IoSkipCurrentIrpStackLocation → crate::glue (tier1 FORCEINLINE)
// SER_LSR/THR/IER/IIR/MSR → crate::glue inline fns
// READ_PORT_UCHAR, WRITE_PORT_UCHAR, ULongToPtr, RtlAssert → crate::glue inline fns
use core::ffi::c_void;

extern "C" {
    fn PushCircularBufferEntry(Buffer: PVOID, Entry: UCHAR) -> NTSTATUS;
    fn DbgPrintEx( ComponentId: ULONG, Level: ULONG, Format: PCSTR, ... ) -> ULONG;
    fn IsCircularBufferEmpty(Buffer: PVOID) -> u8;
    fn PopCircularBufferEntry(Buffer: PVOID, Byte: PUCHAR) -> NTSTATUS;
    fn GetNumberOfElementsInCircularBuffer(CircularBuffer: PVOID) -> ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn ForwardIrpAndForget(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let LowerDevice: PDEVICE_OBJECT =
        (*((*DeviceObject).DeviceExtension as PSERIAL_DEVICE_EXTENSION)).LowerDevice;
    if LowerDevice.is_null() {
        RtlAssert(
            b"LowerDevice\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/serial/serial/misc.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            23 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    IoSkipCurrentIrpStackLocation(Irp);
    return IoCallDriver(LowerDevice, Irp);
}

#[no_mangle]
pub unsafe extern "system" fn SerialReceiveByte(
    Dpc: PKDPC,
    pDeviceExtension: PVOID,
    Unused1: PVOID,
    Unused2: PVOID,
) {

    const SR_LSR_DATA_RECEIVED: ::core::ffi::c_int = 0x01;
    const SR_IER_DATA_RECEIVED: ::core::ffi::c_int = 0x01;
    const DPFLTR_SERIAL_ID: ::core::ffi::c_int = 39; // DPFLTR_SERIAL_ID
    const DPFLTR_INFO_LEVEL: ULONG = 3; // DPFLTR_INFO_LEVEL

    let mut DeviceExtension: PSERIAL_DEVICE_EXTENSION =
        ::core::ptr::null_mut::<_SERIAL_DEVICE_EXTENSION>();
    let mut ComPortBase: PUCHAR = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    let mut Byte: UCHAR = 0;
    let mut Irql: KIRQL = 0;
    let mut IER: UCHAR = 0;
    let mut Status: NTSTATUS = 0;

    DeviceExtension = pDeviceExtension as PSERIAL_DEVICE_EXTENSION;
    ComPortBase = ULongToPtr((*DeviceExtension).BaseAddress as ULONG) as PUCHAR;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*DeviceExtension).InputBufferLock) as PKSPIN_LOCK,
        core::ptr::addr_of_mut!(Irql),
    );
    while READ_PORT_UCHAR(ComPortBase.offset(5)) as ::core::ffi::c_int & SR_LSR_DATA_RECEIVED != 0 {
        Byte = READ_PORT_UCHAR(ComPortBase.offset(0));
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_INFO_LEVEL,
            b"(%s:%d) Byte received on COM%lu: 0x%02x\n\0" as *const u8 as PCSTR,
            b"drivers/serial/serial/misc.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            51 as ::core::ffi::c_int,
            (*DeviceExtension).ComPort,
            Byte as ::core::ffi::c_int,
        );
        Status = PushCircularBufferEntry(
            core::ptr::addr_of_mut!((*DeviceExtension).InputBuffer) as PVOID,
            Byte,
        );
        if NT_SUCCESS(Status) {
            (*DeviceExtension).SerialPerfStats.ReceivedCount = (*DeviceExtension)
                .SerialPerfStats
                .ReceivedCount
                .wrapping_add(1);
        } else {
            (*DeviceExtension).SerialPerfStats.BufferOverrunErrorCount = (*DeviceExtension)
                .SerialPerfStats
                .BufferOverrunErrorCount
                .wrapping_add(1);
        }
    }
    KeSetEvent(
        core::ptr::addr_of_mut!((*DeviceExtension).InputBufferNotEmpty) as PVOID,
        0,
        0 as BOOLEAN,
    );
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*DeviceExtension).InputBufferLock) as PKSPIN_LOCK,
        Irql,
    );

    IER = READ_PORT_UCHAR(ComPortBase.offset(1));
    WRITE_PORT_UCHAR(
        ComPortBase.offset(1),
        (IER as ::core::ffi::c_int | SR_IER_DATA_RECEIVED) as UCHAR,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SerialSendByte(
    mut Dpc: PKDPC,
    mut pDeviceExtension: PVOID,
    mut Unused1: PVOID,
    mut Unused2: PVOID,
) {
    const SR_LSR_THR_EMPTY: ::core::ffi::c_int = 0x20;
    const SR_IER_THR_EMPTY: ::core::ffi::c_int = 0x02;
    const DPFLTR_SERIAL_ID: ::core::ffi::c_int = 39;
    const DPFLTR_INFO_LEVEL: ULONG = 3;

    let mut DeviceExtension: PSERIAL_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut ComPortBase: PUCHAR = core::ptr::null_mut();
    let mut Byte: UCHAR = 0;
    let mut Irql: KIRQL = 0;
    let mut IER: UCHAR = 0;
    let mut Status: NTSTATUS = 0;

    DeviceExtension = pDeviceExtension as PSERIAL_DEVICE_EXTENSION;
    ComPortBase = ULongToPtr((*DeviceExtension).BaseAddress as ULONG) as PUCHAR;

    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*DeviceExtension).OutputBufferLock) as PKSPIN_LOCK,
        core::ptr::addr_of_mut!(Irql),
    );
    while IsCircularBufferEmpty(core::ptr::addr_of_mut!((*DeviceExtension).OutputBuffer) as PVOID) == 0
        && READ_PORT_UCHAR(SER_LSR(ComPortBase)) as ::core::ffi::c_int & SR_LSR_THR_EMPTY != 0
    {
        Status = PopCircularBufferEntry(
            core::ptr::addr_of_mut!((*DeviceExtension).OutputBuffer) as PVOID,
            core::ptr::addr_of_mut!(Byte),
        );
        if !NT_SUCCESS(Status) {
            break;
        }
        WRITE_PORT_UCHAR(SER_THR(ComPortBase), Byte);
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_INFO_LEVEL,
            b"(%s:%d) Byte sent to COM%lu: 0x%02x\n\0" as *const u8 as PCSTR,
            b"drivers/serial/serial/misc.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            92 as ::core::ffi::c_int,
            (*DeviceExtension).ComPort,
            Byte as ::core::ffi::c_int,
        );
        (*DeviceExtension).SerialPerfStats.TransmittedCount =
            (*DeviceExtension).SerialPerfStats.TransmittedCount.wrapping_add(1);
    }
    if IsCircularBufferEmpty(core::ptr::addr_of_mut!((*DeviceExtension).OutputBuffer) as PVOID) == 0 {
        /* allow new interrupts */
        IER = READ_PORT_UCHAR(SER_IER(ComPortBase));
        WRITE_PORT_UCHAR(
            SER_IER(ComPortBase),
            (IER as ::core::ffi::c_int | SR_IER_THR_EMPTY) as UCHAR,
        );
    }
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*DeviceExtension).OutputBufferLock) as PKSPIN_LOCK,
        Irql,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SerialCompleteIrp(
    Dpc: PKDPC,
    pDeviceExtension: PVOID,
    pIrp: PVOID,
    Unused: PVOID,
) {
    IoCompleteRequest(pIrp as PIRP, IO_NO_INCREMENT as CCHAR);
}

#[no_mangle]
pub unsafe extern "system" fn SerialInterruptService(
    Interrupt: PKINTERRUPT,
    ServiceContext: PVOID,
) -> BOOLEAN {

    const SR_IIR_ID_MASK: i32 = 0x07;
    const SR_IIR_MSR_CHANGE: i32 = 0x00;
    const SR_IIR_THR_EMPTY: i32 = 0x02;
    const SR_IIR_DATA_RECEIVED: i32 = 0x04;
    const SR_IIR_ERROR: i32 = 0x06;

    const SR_MSR_CTS_CHANGED: i32 = 0x01;
    const SR_MSR_DSR_CHANGED: i32 = 0x02;
    const SR_MSR_RI_CHANGED: i32 = 0x04;
    const SR_MSR_DCD_CHANGED: i32 = 0x08;
    const SR_MSR_CTS: i32 = 0x10;
    const SR_MSR_DSR: i32 = 0x20;
    const SI_MSR_RI: i32 = 0x40;
    const SR_MSR_DCD: i32 = 0x80;

    const SR_IER_MSR_CHANGE: i32 = 0x08;

    const SR_LSR_OVERRUN_ERROR: i32 = 0x02;
    const SR_LSR_PARITY_ERROR: i32 = 0x04;
    const SR_LSR_FRAMING_ERROR: i32 = 0x08;
    const SR_LSR_BREAK_INT: i32 = 0x10;

    const SERIAL_EV_RXCHAR: u32 = 0x0001;
    const SERIAL_EV_RXFLAG: u32 = 0x0002;
    const SERIAL_EV_TXEMPTY: u32 = 0x0004;
    const SERIAL_EV_CTS: u32 = 0x0008;
    const SERIAL_EV_DSR: u32 = 0x0010;
    const SERIAL_EV_RLSD: u32 = 0x0020;
    const SERIAL_EV_BREAK: u32 = 0x0040;
    const SERIAL_EV_ERR: u32 = 0x0080;
    const SERIAL_EV_RING: u32 = 0x0100;
    const SERIAL_EV_RX80FULL: u32 = 0x0400;

    const TRUE: BOOLEAN = 1;
    const FALSE: BOOLEAN = 0;

    let mut DeviceObject: PDEVICE_OBJECT;
    let mut DeviceExtension: PSERIAL_DEVICE_EXTENSION;
    let mut ComPortBase: PUCHAR;
    let mut Iir: UCHAR;
    let mut Events: ULONG = 0 as ULONG;
    let mut ret: BOOLEAN = FALSE;

    DeviceObject = ServiceContext as PDEVICE_OBJECT;
    DeviceExtension = (*DeviceObject).DeviceExtension as PSERIAL_DEVICE_EXTENSION;
    ComPortBase = ULongToPtr((*DeviceExtension).BaseAddress as ULONG) as PUCHAR;

    Iir = READ_PORT_UCHAR(SER_IIR(ComPortBase));
    if Iir as i32 == 0xff {
        return TRUE;
    }
    Iir = (Iir as i32 & SR_IIR_ID_MASK) as UCHAR;
    if (Iir as u32 & SR_IIR_SELF) != 0 {
        return FALSE;
    }

    match Iir as i32 {
        SR_IIR_MSR_CHANGE => {
            let mut MSR: UCHAR;
            let mut IER: UCHAR;

            MSR = READ_PORT_UCHAR(SER_MSR(ComPortBase));
            if MSR as i32 & SR_MSR_CTS_CHANGED != 0 {
                if MSR as i32 & SR_MSR_CTS != 0 {
                    KeInsertQueueDpc(
                        core::ptr::addr_of_mut!((*DeviceExtension).SendByteDpc),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                } else {
                    /* FIXME: stop transmission */
                }
                Events |= SERIAL_EV_CTS;
            }
            if MSR as i32 & SR_MSR_DSR_CHANGED != 0 {
                if MSR as i32 & SR_MSR_DSR != 0 {
                    KeInsertQueueDpc(
                        core::ptr::addr_of_mut!((*DeviceExtension).ReceivedByteDpc),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                } else {
                    /* FIXME: stop reception */
                }
                Events |= SERIAL_EV_DSR;
            }
            if MSR as i32 & SR_MSR_RI_CHANGED != 0 {
                Events |= SERIAL_EV_RING;
            }
            if MSR as i32 & SR_MSR_DCD_CHANGED != 0 {
                Events |= SERIAL_EV_RLSD;
            }
            IER = READ_PORT_UCHAR(SER_IER(ComPortBase));
            WRITE_PORT_UCHAR(
                SER_IER(ComPortBase),
                (IER as i32 | SR_IER_MSR_CHANGE) as UCHAR,
            );

            ret = TRUE;
        }
        SR_IIR_THR_EMPTY => {
            KeInsertQueueDpc(
                core::ptr::addr_of_mut!((*DeviceExtension).SendByteDpc),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            Events |= SERIAL_EV_TXEMPTY;

            ret = TRUE;
        }
        SR_IIR_DATA_RECEIVED => {
            let mut AlreadyReceivedBytes: ULONG;
            let mut Limit: ULONG;

            KeInsertQueueDpc(
                core::ptr::addr_of_mut!((*DeviceExtension).ReceivedByteDpc),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            Events |= SERIAL_EV_RXCHAR;

            AlreadyReceivedBytes = (GetNumberOfElementsInCircularBuffer(
                core::ptr::addr_of_mut!((*DeviceExtension).InputBuffer) as PVOID,
            ) as ULONG)
                .wrapping_mul(5 as ULONG);
            Limit = ((*DeviceExtension).InputBuffer.Length as ULONG).wrapping_mul(4 as ULONG);
            if AlreadyReceivedBytes < Limit
                && AlreadyReceivedBytes.wrapping_add(1 as ULONG) >= Limit
            {
                Events |= SERIAL_EV_RX80FULL;
            }

            ret = TRUE;
        }
        SR_IIR_ERROR => {
            let mut LSR: UCHAR;

            LSR = READ_PORT_UCHAR(SER_LSR(ComPortBase));
            if LSR as i32 & SR_LSR_OVERRUN_ERROR != 0 {
                InterlockedIncrement(
                    core::ptr::addr_of_mut!((*DeviceExtension).SerialPerfStats.SerialOverrunErrorCount) as PLONG,
                );
                Events |= SERIAL_EV_ERR;
            }
            if LSR as i32 & SR_LSR_PARITY_ERROR != 0 {
                InterlockedIncrement(
                    core::ptr::addr_of_mut!((*DeviceExtension).SerialPerfStats.ParityErrorCount) as PLONG,
                );
                Events |= SERIAL_EV_ERR;
            }
            if LSR as i32 & SR_LSR_FRAMING_ERROR != 0 {
                InterlockedIncrement(
                    core::ptr::addr_of_mut!((*DeviceExtension).SerialPerfStats.FrameErrorCount) as PLONG,
                );
                Events |= SERIAL_EV_ERR;
            }
            if LSR as i32 & SR_LSR_BREAK_INT != 0 {
                InterlockedIncrement(
                    core::ptr::addr_of_mut!((*DeviceExtension).BreakInterruptErrorCount) as PLONG,
                );
                Events |= SERIAL_EV_BREAK;
            }

            ret = TRUE;
        }
        _ => {}
    }

    if ret == 0 {
        return FALSE;
    }
    if !(*DeviceExtension).WaitOnMaskIrp.is_null()
        && (Events & (*DeviceExtension).WaitMask) != 0
    {
        let pEvents: PULONG =
            (*(*DeviceExtension).WaitOnMaskIrp).AssociatedIrp.SystemBuffer as PULONG;

        (*(*DeviceExtension).WaitOnMaskIrp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*(*DeviceExtension).WaitOnMaskIrp).IoStatus.Information =
            core::mem::size_of::<ULONG>() as _;
        *pEvents = Events;
        KeInsertQueueDpc(
            core::ptr::addr_of_mut!((*DeviceExtension).CompleteIrpDpc),
            (*DeviceExtension).WaitOnMaskIrp as PVOID,
            core::ptr::null_mut(),
        );

        (*DeviceExtension).WaitOnMaskIrp = core::ptr::null_mut();
        return TRUE;
    }
    return TRUE;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Serial port driver
//  * FILE:            drivers/dd/serial/misc.c
//  * PURPOSE:         Miscellaneous operations
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
// /* FIXME: call IoAcquireRemoveLock/IoReleaseRemoveLock around each I/O operation */
//
// #include "serial.h"
//
// #include <debug.h>
//
//
// NTSTATUS NTAPI
// ForwardIrpAndForget(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PDEVICE_OBJECT LowerDevice = ((PSERIAL_DEVICE_EXTENSION)DeviceObject->DeviceExtension)->LowerDevice;
//
// 	ASSERT(LowerDevice);
//
// 	IoSkipCurrentIrpStackLocation(Irp);
// 	return IoCallDriver(LowerDevice, Irp);
// }
//
// VOID NTAPI
// SerialReceiveByte(
// 	IN PKDPC Dpc,
// 	IN PVOID pDeviceExtension, // real type PSERIAL_DEVICE_EXTENSION
// 	IN PVOID Unused1,
// 	IN PVOID Unused2)
// {
// 	PSERIAL_DEVICE_EXTENSION DeviceExtension;
// 	PUCHAR ComPortBase;
// 	UCHAR Byte;
// 	KIRQL Irql;
// 	UCHAR IER;
// 	NTSTATUS Status;
//
// 	DeviceExtension = (PSERIAL_DEVICE_EXTENSION)pDeviceExtension;
// 	ComPortBase = ULongToPtr(DeviceExtension->BaseAddress);
//
// 	KeAcquireSpinLock(&DeviceExtension->InputBufferLock, &Irql);
// 	while (READ_PORT_UCHAR(SER_LSR(ComPortBase)) & SR_LSR_DATA_RECEIVED)
// 	{
// 		Byte = READ_PORT_UCHAR(SER_RBR(ComPortBase));
// 		INFO_(SERIAL, "Byte received on COM%lu: 0x%02x\n",
// 			DeviceExtension->ComPort, Byte);
// 		Status = PushCircularBufferEntry(&DeviceExtension->InputBuffer, Byte);
// 		if (NT_SUCCESS(Status))
// 			DeviceExtension->SerialPerfStats.ReceivedCount++;
// 		else
// 			DeviceExtension->SerialPerfStats.BufferOverrunErrorCount++;
// 	}
// 	KeSetEvent(&DeviceExtension->InputBufferNotEmpty, 0, FALSE);
// 	KeReleaseSpinLock(&DeviceExtension->InputBufferLock, Irql);
//
// 	/* allow new interrupts */
// 	IER = READ_PORT_UCHAR(SER_IER(ComPortBase));
// 	WRITE_PORT_UCHAR(SER_IER(ComPortBase), IER | SR_IER_DATA_RECEIVED);
// }
//
// VOID NTAPI
// SerialSendByte(
// 	IN PKDPC Dpc,
// 	IN PVOID pDeviceExtension, // real type PSERIAL_DEVICE_EXTENSION
// 	IN PVOID Unused1,
// 	IN PVOID Unused2)
// {
// 	PSERIAL_DEVICE_EXTENSION DeviceExtension;
// 	PUCHAR ComPortBase;
// 	UCHAR Byte;
// 	KIRQL Irql;
// 	UCHAR IER;
// 	NTSTATUS Status;
//
// 	DeviceExtension = (PSERIAL_DEVICE_EXTENSION)pDeviceExtension;
// 	ComPortBase = ULongToPtr(DeviceExtension->BaseAddress);
//
// 	KeAcquireSpinLock(&DeviceExtension->OutputBufferLock, &Irql);
// 	while (!IsCircularBufferEmpty(&DeviceExtension->OutputBuffer)
// 		&& READ_PORT_UCHAR(SER_LSR(ComPortBase)) & SR_LSR_THR_EMPTY)
// 	{
// 		Status = PopCircularBufferEntry(&DeviceExtension->OutputBuffer, &Byte);
// 		if (!NT_SUCCESS(Status))
// 			break;
// 		WRITE_PORT_UCHAR(SER_THR(ComPortBase), Byte);
// 		INFO_(SERIAL, "Byte sent to COM%lu: 0x%02x\n",
// 			DeviceExtension->ComPort, Byte);
// 		DeviceExtension->SerialPerfStats.TransmittedCount++;
// 	}
// 	if (!IsCircularBufferEmpty(&DeviceExtension->OutputBuffer))
// 	{
// 		/* allow new interrupts */
// 		IER = READ_PORT_UCHAR(SER_IER(ComPortBase));
// 		WRITE_PORT_UCHAR(SER_IER(ComPortBase), IER | SR_IER_THR_EMPTY);
// 	}
// 	KeReleaseSpinLock(&DeviceExtension->OutputBufferLock, Irql);
// }
//
// VOID NTAPI
// SerialCompleteIrp(
// 	IN PKDPC Dpc,
// 	IN PVOID pDeviceExtension, // real type PSERIAL_DEVICE_EXTENSION
// 	IN PVOID pIrp, // real type PIRP
// 	IN PVOID Unused)
// {
// 	IoCompleteRequest((PIRP)pIrp, IO_NO_INCREMENT);
// }
//
// BOOLEAN NTAPI
// SerialInterruptService(
// 	IN PKINTERRUPT Interrupt,
// 	IN OUT PVOID ServiceContext)
// {
// 	PDEVICE_OBJECT DeviceObject;
// 	PSERIAL_DEVICE_EXTENSION DeviceExtension;
// 	PUCHAR ComPortBase;
// 	UCHAR Iir;
// 	ULONG Events = 0;
// 	BOOLEAN ret = FALSE;
//
// 	/* FIXME: sometimes, produce SERIAL_EV_RXFLAG event */
//
// 	DeviceObject = (PDEVICE_OBJECT)ServiceContext;
// 	DeviceExtension = (PSERIAL_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
// 	ComPortBase = ULongToPtr(DeviceExtension->BaseAddress);
//
// 	Iir = READ_PORT_UCHAR(SER_IIR(ComPortBase));
// 	if (Iir == 0xff)
// 		return TRUE;
// 	Iir &= SR_IIR_ID_MASK;
// 	if ((Iir & SR_IIR_SELF) != 0) { return FALSE; }
//
// 	switch (Iir)
// 	{
// 		case SR_IIR_MSR_CHANGE:
// 		{
// 			UCHAR MSR, IER;
// 			TRACE_!(SERIAL, "SR_IIR_MSR_CHANGE\n");
//
// 			MSR = READ_PORT_UCHAR(SER_MSR(ComPortBase));
// 			if (MSR & SR_MSR_CTS_CHANGED)
// 			{
// 				if (MSR & SR_MSR_CTS)
// 					KeInsertQueueDpc(&DeviceExtension->SendByteDpc, NULL, NULL);
// 				else
// 				{
// 					; /* FIXME: stop transmission */
// 				}
// 				Events |= SERIAL_EV_CTS;
// 			}
// 			if (MSR & SR_MSR_DSR_CHANGED)
// 			{
// 				if (MSR & SR_MSR_DSR)
// 					KeInsertQueueDpc(&DeviceExtension->ReceivedByteDpc, NULL, NULL);
// 				else
// 				{
// 					; /* FIXME: stop reception */
// 				}
// 				Events |= SERIAL_EV_DSR;
// 			}
// 			if (MSR & SR_MSR_RI_CHANGED)
// 			{
// 				INFO_(SERIAL, "SR_MSR_RI_CHANGED changed: now %d\n", MSR & SI_MSR_RI);
// 				Events |= SERIAL_EV_RING;
// 			}
// 			if (MSR & SR_MSR_DCD_CHANGED)
// 			{
// 				INFO_(SERIAL, "SR_MSR_DCD_CHANGED changed: now %d\n", MSR & SR_MSR_DCD);
// 				Events |= SERIAL_EV_RLSD;
// 			}
// 			IER = READ_PORT_UCHAR(SER_IER(ComPortBase));
// 			WRITE_PORT_UCHAR(SER_IER(ComPortBase), IER | SR_IER_MSR_CHANGE);
//
// 			ret = TRUE;
// 			goto done;
// 		}
// 		case SR_IIR_THR_EMPTY:
// 		{
// 			TRACE_!(SERIAL, "SR_IIR_THR_EMPTY\n");
//
// 			KeInsertQueueDpc(&DeviceExtension->SendByteDpc, NULL, NULL);
// 			Events |= SERIAL_EV_TXEMPTY;
//
// 			ret = TRUE;
// 			goto done;
// 		}
// 		case SR_IIR_DATA_RECEIVED:
// 		{
// 			ULONG AlreadyReceivedBytes, Limit;
// 			TRACE_!(SERIAL, "SR_IIR_DATA_RECEIVED\n");
//
// 			KeInsertQueueDpc(&DeviceExtension->ReceivedByteDpc, NULL, NULL);
// 			Events |= SERIAL_EV_RXCHAR;
//
// 			/* Check if buffer will be 80% full */
// 			AlreadyReceivedBytes = GetNumberOfElementsInCircularBuffer(
// 				&DeviceExtension->InputBuffer) * 5;
// 			Limit = DeviceExtension->InputBuffer.Length * 4;
// 			if (AlreadyReceivedBytes < Limit && AlreadyReceivedBytes + 1 >= Limit)
// 			{
// 				/* Buffer is full at 80% */
// 				Events |= SERIAL_EV_RX80FULL;
// 			}
//
// 			ret = TRUE;
// 			goto done;
// 		}
// 		case SR_IIR_ERROR:
// 		{
// 			UCHAR LSR;
// 			TRACE_!(SERIAL, "SR_IIR_ERROR\n");
//
// 			LSR = READ_PORT_UCHAR(SER_LSR(ComPortBase));
// 			if (LSR & SR_LSR_OVERRUN_ERROR)
// 			{
// 				InterlockedIncrement((PLONG)&DeviceExtension->SerialPerfStats.SerialOverrunErrorCount);
// 				Events |= SERIAL_EV_ERR;
// 			}
// 			if (LSR & SR_LSR_PARITY_ERROR)
// 			{
// 				InterlockedIncrement((PLONG)&DeviceExtension->SerialPerfStats.ParityErrorCount);
// 				Events |= SERIAL_EV_ERR;
// 			}
// 			if (LSR & SR_LSR_FRAMING_ERROR)
// 			{
// 				InterlockedIncrement((PLONG)&DeviceExtension->SerialPerfStats.FrameErrorCount);
// 				Events |= SERIAL_EV_ERR;
// 			}
// 			if (LSR & SR_LSR_BREAK_INT)
// 			{
// 				InterlockedIncrement((PLONG)&DeviceExtension->BreakInterruptErrorCount);
// 				Events |= SERIAL_EV_BREAK;
// 			}
//
// 			ret = TRUE;
// 			goto done;
// 		}
// 	}
//
// done:
// 	if (!ret)
// 		return FALSE;
// 	if (DeviceExtension->WaitOnMaskIrp && (Events & DeviceExtension->WaitMask))
// 	{
// 		/* Finish pending IRP */
// 		PULONG pEvents = (PULONG)DeviceExtension->WaitOnMaskIrp->AssociatedIrp.SystemBuffer;
//
// 		DeviceExtension->WaitOnMaskIrp->IoStatus.Status = STATUS_SUCCESS;
// 		DeviceExtension->WaitOnMaskIrp->IoStatus.Information = sizeof(ULONG);
// 		*pEvents = Events;
// 		KeInsertQueueDpc(&DeviceExtension->CompleteIrpDpc, DeviceExtension->WaitOnMaskIrp, NULL);
//
// 		/* We are now ready to handle another IRP, even if this one is not completed */
// 		DeviceExtension->WaitOnMaskIrp = NULL;
// 		return TRUE;
// 	}
// 	return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
