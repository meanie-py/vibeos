use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{KeStallExecutionProcessor};
use core::ffi::c_void;

extern "C" {
    fn i8042ReadKeyboardData(DeviceExtension: PPORT_DEVICE_EXTENSION, Data: PUCHAR) -> NTSTATUS;
    fn READ_PORT_UCHAR(Port: PUCHAR) -> UCHAR;
    fn WRITE_PORT_UCHAR(Port: PUCHAR, Value: UCHAR);
}

unsafe fn i8042Flush(DeviceExtension: PPORT_DEVICE_EXTENSION) {
    let mut Ignore: UCHAR = 0;

    while NT_SUCCESS(i8042ReadData(DeviceExtension, KBD_OBF as UCHAR, &mut Ignore)) {
        KeStallExecutionProcessor(50);
    }

    while NT_SUCCESS(i8042ReadData(DeviceExtension, KBD_IBF as UCHAR, &mut Ignore)) {
        KeStallExecutionProcessor(50);
    }
}

unsafe fn i8042IsrWritePort(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    Value: UCHAR,
    SelectCmd: UCHAR,
) -> BOOLEAN {
    if SelectCmd != 0 {
        if i8042Write(DeviceExtension, (*DeviceExtension).ControlPort, SelectCmd) == 0 {
            return 0u8;
        }
    }

    i8042Write(DeviceExtension, (*DeviceExtension).DataPort, Value)
}

unsafe fn i8042ReadData(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    StatusFlags: UCHAR,
    Data: PUCHAR,
) -> NTSTATUS {
    let mut PortStatus: UCHAR = 0;
    let Status: NTSTATUS;

    Status = i8042ReadStatus(DeviceExtension, &mut PortStatus);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if PortStatus & StatusFlags != 0 {
        *Data = READ_PORT_UCHAR((*DeviceExtension).DataPort);

        if (PortStatus & KBD_PERR as UCHAR) == 0 {
            return STATUS_SUCCESS;
        }
    }
    STATUS_UNSUCCESSFUL
}

unsafe fn i8042ReadStatus(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    Status: PUCHAR,
) -> NTSTATUS {
    *Status = READ_PORT_UCHAR((*DeviceExtension).ControlPort);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn i8042ReadDataWait(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    Data: PUCHAR,
) -> NTSTATUS {
    let mut Counter: ULONG;
    let mut Status: NTSTATUS;

    Counter = (*DeviceExtension).Settings.PollingIterations;

    while Counter != 0 {
        Counter -= 1;

        Status = i8042ReadKeyboardData(DeviceExtension, Data);

        if NT_SUCCESS(Status) {
            return Status;
        }

        KeStallExecutionProcessor(50);
    }

    /* Timed out */
    STATUS_IO_TIMEOUT
}

#[no_mangle]
pub unsafe extern "system" fn i8042SynchReadPort(
    Context: PVOID,
    Value: PUCHAR,
    WaitForAck: BOOLEAN,
) -> NTSTATUS {
    let DeviceExtension: PPORT_DEVICE_EXTENSION;

    let _ = WaitForAck;

    DeviceExtension = Context as PPORT_DEVICE_EXTENSION;

    i8042ReadDataWait(DeviceExtension, Value)
}

#[no_mangle]
pub unsafe extern "system" fn i8042SynchWritePort(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    Port: UCHAR,
    Value: UCHAR,
    WaitForAck: BOOLEAN,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut Ack: UCHAR = 0;
    let mut ResendIterations: ULONG;

    ResendIterations = (*DeviceExtension).Settings.ResendIterations + 1;

    loop {
        if Port != 0 {
            if i8042Write(DeviceExtension, (*DeviceExtension).DataPort, Port) == 0 {
                return STATUS_IO_TIMEOUT;
            }
        }

        if i8042Write(DeviceExtension, (*DeviceExtension).DataPort, Value) == 0 {
            return STATUS_IO_TIMEOUT;
        }

        if WaitForAck != 0 {
            Status = i8042ReadDataWait(DeviceExtension, core::ptr::addr_of_mut!(Ack));
            if !NT_SUCCESS(Status) {
                return Status;
            }
            if Ack == KBD_ACK as UCHAR {
                return STATUS_SUCCESS;
            } else if Ack == KBD_RESEND as UCHAR {
            }
        } else {
            return STATUS_SUCCESS;
        }

        ResendIterations -= 1;
        if ResendIterations == 0 {
            break;
        }
    }

    STATUS_IO_TIMEOUT
}

unsafe fn i8042Write(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    addr: PUCHAR,
    data: UCHAR,
) -> BOOLEAN {
    let mut Counter: ULONG;

    Counter = (*DeviceExtension).Settings.PollingIterations;

    while (KBD_IBF & READ_PORT_UCHAR((*DeviceExtension).ControlPort)) != 0 && {
        let old = Counter;
        Counter = Counter.wrapping_sub(1);
        old != 0
    } {
        KeStallExecutionProcessor(50);
    }

    if Counter != 0 {
        WRITE_PORT_UCHAR(addr, data);
        return 1u8;
    }
    0u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS i8042 (ps/2 keyboard-mouse controller) driver
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        drivers/input/i8042prt/readwrite.c
//  * PURPOSE:     Read/write port functions
//  * PROGRAMMERS: Copyright Victor Kirhenshtein (sauros@iname.com)
//                 Copyright Jason Filby (jasonfilby@yahoo.com)
//                 Copyright Martijn Vernooij (o112w8r02@sneakemail.com)
//                 Copyright 2006-2007 Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "i8042prt.h"
//
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// i8042Flush(
// 	IN PPORT_DEVICE_EXTENSION DeviceExtension)
// {
// 	UCHAR Ignore;
//
// 	/* Flush output buffer */
// 	while (NT_SUCCESS(i8042ReadData(DeviceExtension, KBD_OBF /* | MOU_OBF*/, &Ignore))) {
// 		KeStallExecutionProcessor(50);
// 		TRACE_(I8042PRT, "Output data flushed\n");
// 	}
//
// 	/* Flush input buffer */
// 	while (NT_SUCCESS(i8042ReadData(DeviceExtension, KBD_IBF, &Ignore))) {
// 		KeStallExecutionProcessor(50);
// 		TRACE_(I8042PRT, "Input data flushed\n");
// 	}
// }
//
// BOOLEAN
// i8042IsrWritePort(
// 	IN PPORT_DEVICE_EXTENSION DeviceExtension,
// 	IN UCHAR Value,
// 	IN UCHAR SelectCmd OPTIONAL)
// {
// 	if (SelectCmd)
// 		if (!i8042Write(DeviceExtension, DeviceExtension->ControlPort, SelectCmd))
// 			return FALSE;
//
// 	return i8042Write(DeviceExtension, DeviceExtension->DataPort, Value);
// }
//
// /*
//  * FUNCTION: Read data from port 0x60
//  */
// NTSTATUS
// i8042ReadData(
// 	IN PPORT_DEVICE_EXTENSION DeviceExtension,
// 	IN UCHAR StatusFlags,
// 	OUT PUCHAR Data)
// {
// 	UCHAR PortStatus;
// 	NTSTATUS Status;
//
// 	Status = i8042ReadStatus(DeviceExtension, &PortStatus);
// 	if (!NT_SUCCESS(Status))
// 		return Status;
//
// 	// If data is available
// 	if (PortStatus & StatusFlags)
// 	{
// 		*Data = READ_PORT_UCHAR(DeviceExtension->DataPort);
// 		INFO_(I8042PRT, "Read: 0x%02x (status: 0x%x)\n", Data[0], PortStatus);
//
// 		// If the data is valid (not timeout, not parity error)
// 		if ((PortStatus & KBD_PERR) == 0)
// 			return STATUS_SUCCESS;
// 	}
// 	return STATUS_UNSUCCESSFUL;
// }
//
// NTSTATUS
// i8042ReadStatus(
// 	IN PPORT_DEVICE_EXTENSION DeviceExtension,
// 	OUT PUCHAR Status)
// {
// 	ASSERT(DeviceExtension->ControlPort != NULL);
// 	*Status = READ_PORT_UCHAR(DeviceExtension->ControlPort);
// 	return STATUS_SUCCESS;
// }
//
// /*
//  * FUNCTION: Read data from data port
//  */
// NTSTATUS
// i8042ReadDataWait(
// 	IN PPORT_DEVICE_EXTENSION DeviceExtension,
// 	OUT PUCHAR Data)
// {
// 	ULONG Counter;
// 	NTSTATUS Status;
//
// 	Counter = DeviceExtension->Settings.PollingIterations;
//
// 	while (Counter--)
// 	{
// 		Status = i8042ReadKeyboardData(DeviceExtension, Data);
//
// 		if (NT_SUCCESS(Status))
// 			return Status;
//
// 		KeStallExecutionProcessor(50);
// 	}
//
// 	/* Timed out */
// 	return STATUS_IO_TIMEOUT;
// }
//
// /*
//  * This one reads a value from the port; You don't have to specify
//  * which one, it'll always be from the one you talked to, so one function
//  * is enough this time. Note how MSDN specifies the
//  * WaitForAck parameter to be ignored.
//  */
// NTSTATUS NTAPI
// i8042SynchReadPort(
// 	IN PVOID Context,
// 	OUT PUCHAR Value,
// 	IN BOOLEAN WaitForAck)
// {
// 	PPORT_DEVICE_EXTENSION DeviceExtension;
//
// 	UNREFERENCED_PARAMETER(WaitForAck);
//
// 	DeviceExtension = (PPORT_DEVICE_EXTENSION)Context;
//
// 	return i8042ReadDataWait(DeviceExtension, Value);
// }
//
// /*
//  * These functions are callbacks for filter driver custom
//  * initialization routines.
//  */
// NTSTATUS NTAPI
// i8042SynchWritePort(
// 	IN PPORT_DEVICE_EXTENSION DeviceExtension,
// 	IN UCHAR Port,
// 	IN UCHAR Value,
// 	IN BOOLEAN WaitForAck)
// {
// 	NTSTATUS Status;
// 	UCHAR Ack;
// 	ULONG ResendIterations;
//
// 	ResendIterations = DeviceExtension->Settings.ResendIterations + 1;
//
// 	do
// 	{
// 		if (Port)
// 			if (!i8042Write(DeviceExtension, DeviceExtension->DataPort, Port))
// 			{
// 				WARN_(I8042PRT, "Failed to write Port\n");
// 				return STATUS_IO_TIMEOUT;
// 			}
//
// 		if (!i8042Write(DeviceExtension, DeviceExtension->DataPort, Value))
// 		{
// 			WARN_(I8042PRT, "Failed to write Value\n");
// 			return STATUS_IO_TIMEOUT;
// 		}
//
// 		if (WaitForAck)
// 		{
// 			Status = i8042ReadDataWait(DeviceExtension, &Ack);
// 			if (!NT_SUCCESS(Status))
// 			{
// 				WARN_(I8042PRT, "Failed to read Ack\n");
// 				return Status;
// 			}
// 			if (Ack == KBD_ACK)
// 				return STATUS_SUCCESS;
// 			else if (Ack == KBD_RESEND)
// 				INFO_(I8042PRT, "i8042 asks for a data resend\n");
// 		}
// 		else
// 		{
// 			return STATUS_SUCCESS;
// 		}
// 		TRACE_(I8042PRT, "Reiterating\n");
// 		ResendIterations--;
// 	} while (ResendIterations);
//
// 	return STATUS_IO_TIMEOUT;
// }
//
// /*
//  * FUNCTION: Write data to a port, waiting first for it to become ready
//  */
// BOOLEAN
// i8042Write(
// 	IN PPORT_DEVICE_EXTENSION DeviceExtension,
// 	IN PUCHAR addr,
// 	IN UCHAR data)
// {
// 	ULONG Counter;
//
// 	ASSERT(addr);
// 	ASSERT(DeviceExtension->ControlPort != NULL);
//
// 	Counter = DeviceExtension->Settings.PollingIterations;
//
// 	while ((KBD_IBF & READ_PORT_UCHAR(DeviceExtension->ControlPort)) &&
// 	       (Counter--))
// 	{
// 		KeStallExecutionProcessor(50);
// 	}
//
// 	if (Counter)
// 	{
// 		WRITE_PORT_UCHAR(addr, data);
// 		INFO_(I8042PRT, "Sent 0x%x to port %p\n", data, addr);
// 		return TRUE;
// 	}
// 	return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
