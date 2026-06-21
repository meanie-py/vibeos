use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::ddk::{PUCHAR, PULONG, PUSHORT, UCHAR, ULONG, USHORT, PVOID};
use core::ffi::c_void;

// PORT I/O and REGISTER functions are provided by glue.rs (via the glob import above).
// On i386/arm64 they are HAL extern symbols; on amd64 they are inline asm/volatile impls.

// ---- file-scope preamble (translated) ----
// #define NDEBUG (skipped: configuration macro)

#[no_mangle]
pub unsafe extern "system" fn StorPortReadPortBufferUchar(
    mut HwDeviceExtension: PVOID,
    mut Port: PUCHAR,
    mut Buffer: PUCHAR,
    mut Count: ULONG,
) {
    READ_PORT_BUFFER_UCHAR(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadPortBufferUlong(
    mut HwDeviceExtension: PVOID,
    mut Port: PULONG,
    mut Buffer: PULONG,
    mut Count: ULONG,
) {
    READ_PORT_BUFFER_ULONG(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadPortBufferUshort(
    mut HwDeviceExtension: PVOID,
    mut Port: PUSHORT,
    mut Buffer: PUSHORT,
    mut Count: ULONG,
) {
    READ_PORT_BUFFER_USHORT(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadPortUchar(
    mut HwDeviceExtension: PVOID,
    mut Port: PUCHAR,
) -> UCHAR {
    return READ_PORT_UCHAR(Port);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadPortUlong(
    mut HwDeviceExtension: PVOID,
    mut Port: PULONG,
) -> ULONG {
    return READ_PORT_ULONG(Port);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadPortUshort(
    mut HwDeviceExtension: PVOID,
    mut Port: PUSHORT,
) -> USHORT {
    return READ_PORT_USHORT(Port);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadRegisterBufferUchar(
    HwDeviceExtension: PVOID,
    Register: PUCHAR,
    Buffer: PUCHAR,
    Count: ULONG,
) {
    READ_REGISTER_BUFFER_UCHAR(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadRegisterBufferUlong(
    HwDeviceExtension: PVOID,
    Register: PULONG,
    Buffer: PULONG,
    Count: ULONG,
) {
    READ_REGISTER_BUFFER_ULONG(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadRegisterBufferUshort(
    HwDeviceExtension: PVOID,
    Register: PUSHORT,
    Buffer: PUSHORT,
    Count: ULONG,
) {
    READ_REGISTER_BUFFER_USHORT(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadRegisterUchar(
    HwDeviceExtension: PVOID,
    Register: PUCHAR,
) -> UCHAR {
    return READ_REGISTER_UCHAR(Register as *mut UCHAR);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadRegisterUlong(
    HwDeviceExtension: PVOID,
    Register: PULONG,
) -> ULONG {
    return READ_REGISTER_ULONG(Register as *mut ULONG);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadRegisterUshort(
    HwDeviceExtension: PVOID,
    Register: PUSHORT,
) -> USHORT {
    return READ_REGISTER_USHORT(Register as *mut USHORT);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWritePortBufferUchar(
    mut HwDeviceExtension: PVOID,
    mut Port: PUCHAR,
    mut Buffer: PUCHAR,
    mut Count: ULONG,
) {
    WRITE_PORT_BUFFER_UCHAR(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWritePortBufferUlong(
    mut HwDeviceExtension: PVOID,
    mut Port: PULONG,
    mut Buffer: PULONG,
    mut Count: ULONG,
) {
    WRITE_PORT_BUFFER_ULONG(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWritePortBufferUshort(
    mut HwDeviceExtension: PVOID,
    mut Port: PUSHORT,
    mut Buffer: PUSHORT,
    mut Count: ULONG,
) {
    WRITE_PORT_BUFFER_USHORT(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWritePortUchar(
    mut HwDeviceExtension: PVOID,
    mut Port: PUCHAR,
    mut Value: UCHAR,
) {
    WRITE_PORT_UCHAR(Port, Value);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWritePortUlong(
    mut HwDeviceExtension: PVOID,
    mut Port: PULONG,
    mut Value: ULONG,
) {
    WRITE_PORT_ULONG(Port, Value);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWritePortUshort(
    mut HwDeviceExtension: PVOID,
    mut Port: PUSHORT,
    mut Value: USHORT,
) {
    WRITE_PORT_USHORT(Port, Value);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWriteRegisterBufferUchar(
    HwDeviceExtension: PVOID,
    Register: PUCHAR,
    Buffer: PUCHAR,
    Count: ULONG,
) {
    WRITE_REGISTER_BUFFER_UCHAR(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWriteRegisterBufferUlong(
    HwDeviceExtension: PVOID,
    Register: PULONG,
    Buffer: PULONG,
    Count: ULONG,
) {
    WRITE_REGISTER_BUFFER_ULONG(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWriteRegisterBufferUshort(
    HwDeviceExtension: PVOID,
    Register: PUSHORT,
    Buffer: PUSHORT,
    Count: ULONG,
) {
    WRITE_REGISTER_BUFFER_USHORT(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWriteRegisterUchar(
    HwDeviceExtension: PVOID,
    Register: PUCHAR,
    Value: UCHAR,
) {
    WRITE_REGISTER_UCHAR(Register as *mut UCHAR, Value);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWriteRegisterUlong(
    mut HwDeviceExtension: PVOID,
    mut Register: PULONG,
    mut Value: ULONG,
) {
    WRITE_REGISTER_ULONG(Register as *mut ULONG, Value);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWriteRegisterUshort(
    HwDeviceExtension: PVOID,
    Register: PUSHORT,
    Value: USHORT,
) {
    WRITE_REGISTER_USHORT(Register as *mut u16, Value);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Storport Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     Storport driver stub functions
//  * COPYRIGHT:   Copyright 2017 Eric Kohl (eric.kohl@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* FUNCTIONS ******************************************************************/
//
// STORPORT_API
// VOID
// NTAPI
// StorPortReadPortBufferUchar(
//      PVOID HwDeviceExtension,
//      PUCHAR Port,
//      PUCHAR Buffer,
//      ULONG Count)
// {
//     READ_PORT_BUFFER_UCHAR(Port, Buffer, Count);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortReadPortBufferUlong(
//      PVOID HwDeviceExtension,
//      PULONG Port,
//      PULONG Buffer,
//      ULONG Count)
// {
//     READ_PORT_BUFFER_ULONG(Port, Buffer, Count);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortReadPortBufferUshort(
//      PVOID HwDeviceExtension,
//      PUSHORT Port,
//      PUSHORT Buffer,
//      ULONG Count)
// {
//     READ_PORT_BUFFER_USHORT(Port, Buffer, Count);
// }
//
//
// STORPORT_API
// UCHAR
// NTAPI
// StorPortReadPortUchar(
//      PVOID HwDeviceExtension,
//      PUCHAR Port)
// {
//     return READ_PORT_UCHAR(Port);
// }
//
//
// STORPORT_API
// ULONG
// NTAPI
// StorPortReadPortUlong(
//      PVOID HwDeviceExtension,
//      PULONG Port)
// {
//     return READ_PORT_ULONG(Port);
// }
//
//
// STORPORT_API
// USHORT
// NTAPI
// StorPortReadPortUshort(
//      PVOID HwDeviceExtension,
//      PUSHORT Port)
// {
//     return READ_PORT_USHORT(Port);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortReadRegisterBufferUchar(
//      PVOID HwDeviceExtension,
//      PUCHAR Register,
//      PUCHAR Buffer,
//      ULONG Count)
// {
//     READ_REGISTER_BUFFER_UCHAR(Register, Buffer, Count);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortReadRegisterBufferUlong(
//      PVOID HwDeviceExtension,
//      PULONG Register,
//      PULONG Buffer,
//      ULONG Count)
// {
//     READ_REGISTER_BUFFER_ULONG(Register, Buffer, Count);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortReadRegisterBufferUshort(
//      PVOID HwDeviceExtension,
//      PUSHORT Register,
//      PUSHORT Buffer,
//      ULONG Count)
// {
//     READ_REGISTER_BUFFER_USHORT(Register, Buffer, Count);
// }
//
//
// STORPORT_API
// UCHAR
// NTAPI
// StorPortReadRegisterUchar(
//      PVOID HwDeviceExtension,
//      PUCHAR Register)
// {
//     return READ_REGISTER_UCHAR(Register);
// }
//
//
// STORPORT_API
// ULONG
// NTAPI
// StorPortReadRegisterUlong(
//      PVOID HwDeviceExtension,
//      PULONG Register)
// {
//     return READ_REGISTER_ULONG(Register);
// }
//
//
// STORPORT_API
// USHORT
// NTAPI
// StorPortReadRegisterUshort(
//      PVOID HwDeviceExtension,
//      PUSHORT Register)
// {
//     return READ_REGISTER_USHORT(Register);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortWritePortBufferUchar(
//      PVOID HwDeviceExtension,
//      PUCHAR Port,
//      PUCHAR Buffer,
//      ULONG Count)
// {
//     WRITE_PORT_BUFFER_UCHAR(Port, Buffer, Count);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortWritePortBufferUlong(
//      PVOID HwDeviceExtension,
//      PULONG Port,
//      PULONG Buffer,
//      ULONG Count)
// {
//     WRITE_PORT_BUFFER_ULONG(Port, Buffer, Count);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortWritePortBufferUshort(
//      PVOID HwDeviceExtension,
//      PUSHORT Port,
//      PUSHORT Buffer,
//      ULONG Count)
// {
//     WRITE_PORT_BUFFER_USHORT(Port, Buffer, Count);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortWritePortUchar(
//      PVOID HwDeviceExtension,
//      PUCHAR Port,
//      UCHAR Value)
// {
//     WRITE_PORT_UCHAR(Port, Value);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortWritePortUlong(
//      PVOID HwDeviceExtension,
//      PULONG Port,
//      ULONG Value)
// {
//     WRITE_PORT_ULONG(Port, Value);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortWritePortUshort(
//      PVOID HwDeviceExtension,
//      PUSHORT Port,
//      USHORT Value)
// {
//     WRITE_PORT_USHORT(Port, Value);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortWriteRegisterBufferUchar(
//      PVOID HwDeviceExtension,
//      PUCHAR Register,
//      PUCHAR Buffer,
//      ULONG Count)
// {
//     WRITE_REGISTER_BUFFER_UCHAR(Register, Buffer, Count);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortWriteRegisterBufferUlong(
//      PVOID HwDeviceExtension,
//      PULONG Register,
//      PULONG Buffer,
//      ULONG Count)
// {
//     WRITE_REGISTER_BUFFER_ULONG(Register, Buffer, Count);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortWriteRegisterBufferUshort(
//      PVOID HwDeviceExtension,
//      PUSHORT Register,
//      PUSHORT Buffer,
//      ULONG Count)
// {
//     WRITE_REGISTER_BUFFER_USHORT(Register, Buffer, Count);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortWriteRegisterUchar(
//      PVOID HwDeviceExtension,
//      PUCHAR Register,
//      UCHAR Value)
// {
//     WRITE_REGISTER_UCHAR(Register, Value);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortWriteRegisterUlong(
//      PVOID HwDeviceExtension,
//      PULONG Register,
//      ULONG Value)
// {
//     WRITE_REGISTER_ULONG(Register, Value);
// }
//
//
// STORPORT_API
// VOID
// NTAPI
// StorPortWriteRegisterUshort(
//      PVOID HwDeviceExtension,
//      PUSHORT Register,
//      USHORT Value)
// {
//     WRITE_REGISTER_USHORT(Register, Value);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
