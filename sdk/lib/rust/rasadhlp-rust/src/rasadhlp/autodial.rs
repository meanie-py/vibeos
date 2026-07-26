use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn AcsHlpSendCommand(Command: PAUTODIAL_COMMAND) -> BOOLEAN {
    // C: RTL_CONSTANT_STRING(L"\\Device\\RasAcd") — WIDE. The translation dropped the
    // `L`, and a UNICODE_STRING over ANSI bytes compiles but cannot open the device.
    let mut DriverName: _UNICODE_STRING = RTL_CONSTANT_STRING!("\\Device\\RasAcd");
    let mut Status: NTSTATUS;
    let mut DriverHandle: HANDLE = core::ptr::null_mut();
    let mut EventHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut DriverName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateFile(
        &mut DriverHandle,
        FILE_READ_DATA | FILE_WRITE_DATA,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        FILE_ATTRIBUTE_NORMAL,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_OPEN_IF,
        0,
        core::ptr::null_mut(),
        0,
    );
    if !NT_SUCCESS(Status) {
        return (FALSE) as u8;
    }

    EventHandle = CreateEvent(
        core::ptr::null_mut(),
        FALSE as BOOL,
        FALSE as BOOL,
        core::ptr::null(),
    );
    if EventHandle.is_null() {
        CloseHandle(DriverHandle);
        return (FALSE) as u8;
    }

    Status = NtDeviceIoControlFile(
        DriverHandle,
        EventHandle,
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        IOCTL_ACD_CONNECT_ADDRESS,
        Command as *mut core::ffi::c_void,
        core::mem::size_of::<AUTODIAL_COMMAND>() as ULONG,
        core::ptr::null_mut(),
        0,
    );

    if Status == STATUS_PENDING {
        Status = WaitForSingleObject(EventHandle, INFINITE) as NTSTATUS;
        Status = IoStatusBlock.__bindgen_anon_1.Status;
    }

    CloseHandle(EventHandle);
    CloseHandle(DriverHandle);
    NT_SUCCESS(Status) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn AcsHlpAttemptConnection(ConnectionAddress: PAUTODIAL_ADDR) -> BOOLEAN {
    let mut Command: AUTODIAL_COMMAND = core::mem::zeroed();

    core::ptr::write_bytes(
        &mut Command as *mut _ as *mut u8,
        0,
        core::mem::size_of::<AUTODIAL_COMMAND>(),
    );

    core::ptr::copy_nonoverlapping(
        ConnectionAddress as *const u8,
        core::ptr::addr_of_mut!(Command.Address) as *mut u8,
        core::mem::size_of::<AUTODIAL_ADDR>(),
    );

    AcsHlpSendCommand(&mut Command)
}

#[no_mangle]
pub unsafe extern "system" fn AcsHlpNoteNewConnection(
    ConnectionAddress: PAUTODIAL_ADDR,
    Connection: PAUTODIAL_CONN,
) -> BOOLEAN {
    let mut Command: AUTODIAL_COMMAND = core::mem::zeroed();

    core::ptr::copy_nonoverlapping(
        ConnectionAddress as *const u8,
        core::ptr::addr_of_mut!(Command.Address) as *mut u8,
        core::mem::size_of::<AUTODIAL_ADDR>(),
    );

    Command.NewConnection = (TRUE) as i32;
    core::ptr::copy_nonoverlapping(
        Connection as *const u8,
        core::ptr::addr_of_mut!(Command.Connection) as *mut u8,
        core::mem::size_of::<AUTODIAL_CONN>(),
    );

    AcsHlpSendCommand(core::ptr::addr_of_mut!(Command))
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:   See COPYING in the top level directory
//  * PROJECT:     ReactOS Winsock 2 SPI
//  * FILE:        lib/mswsock/lib/init.c
//  * PURPOSE:     DLL Initialization
//  */
//
// #include "precomp.h"
//
// #include <ndk/iofuncs.h>
// #include <ndk/rtltypes.h>
//
// /* FUNCTIONS *****************************************************************/
//
// BOOLEAN
// WINAPI
// AcsHlpSendCommand( PAUTODIAL_COMMAND Command)
// {
//     UNICODE_STRING DriverName = RTL_CONSTANT_STRING(L"\\Device\\RasAcd");
//     NTSTATUS Status;
//     HANDLE DriverHandle;
//     HANDLE EventHandle = NULL;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//
//     /* Initialize the object attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DriverName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     /* Open a handle to it */
//     Status = NtCreateFile(&DriverHandle,
//                           FILE_READ_DATA | FILE_WRITE_DATA,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           FILE_ATTRIBUTE_NORMAL,
//                           FILE_SHARE_READ | FILE_SHARE_WRITE,
//                           FILE_OPEN_IF,
//                           0,
//                           NULL,
//                           0);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Create an event */
//     EventHandle = CreateEvent(NULL, FALSE, FALSE, NULL);
//     if (!EventHandle)
//     {
//         /* Event failed, fail us */
//         CloseHandle(DriverHandle);
//         return FALSE;
//     }
//
//     /* Connect to the driver */
//     Status = NtDeviceIoControlFile(DriverHandle,
//                                    EventHandle,
//                                    NULL,
//                                    NULL,
//                                    &IoStatusBlock,
//                                    IOCTL_ACD_CONNECT_ADDRESS,
//                                    Command,
//                                    sizeof(AUTODIAL_COMMAND),
//                                    NULL,
//                                    0);
//
//     /* Check if we need to wait */
//     if (Status == STATUS_PENDING)
//     {
//         /* Wait for the driver */
//         Status = WaitForSingleObject(EventHandle, INFINITE);
//
//         /* Update status */
//         Status = IoStatusBlock.Status;
//     }
//
//     /* Close handles and return */
//     CloseHandle(EventHandle);
//     CloseHandle(DriverHandle);
//     return NT_SUCCESS(Status);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// WINAPI
// AcsHlpAttemptConnection( PAUTODIAL_ADDR ConnectionAddress)
// {
//     AUTODIAL_COMMAND Command;
//
//     /* Clear the command packet */
//     RtlZeroMemory(&Command, sizeof(AUTODIAL_COMMAND));
//
//     /* Copy the address into the command packet */
//     RtlCopyMemory(&Command.Address, ConnectionAddress, sizeof(AUTODIAL_ADDR));
//
//     /* Send it to the driver */
//     return AcsHlpSendCommand(&Command);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// WINAPI
// AcsHlpNoteNewConnection( PAUTODIAL_ADDR ConnectionAddress,
//                          PAUTODIAL_CONN Connection)
// {
//     AUTODIAL_COMMAND Command;
//
//     /* Copy the address into the command packet */
//     RtlCopyMemory(&Command.Address, ConnectionAddress, sizeof(AUTODIAL_ADDR));
//
//     /* Set the New Connection flag and copy the connection data */
//     Command.NewConnection = TRUE;
//     RtlCopyMemory(&Command.Connection, Connection, sizeof(AUTODIAL_CONN));
//
//     /* Send it to the driver */
//     return AcsHlpSendCommand(&Command);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
