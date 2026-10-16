use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// File-scope preamble of stubs.c: no types, constants, or tables to emit.
// `#define NDEBUG` has no value and only gates assert behavior, so it is omitted.
// The STORPORT_API prototype fragments name no identifiers and are not translatable here.

unsafe extern "system" fn placeholder_unused() {}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadPortBufferUchar(
    HwDeviceExtension: PVOID,
    Port: PUCHAR,
    Buffer: PUCHAR,
    Count: ULONG,
) {
    READ_PORT_BUFFER_UCHAR(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadPortBufferUlong(
    HwDeviceExtension: PVOID,
    Port: PULONG,
    Buffer: PULONG,
    Count: ULONG,
) {
    READ_PORT_BUFFER_ULONG(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadPortBufferUshort(
    HwDeviceExtension: PVOID,
    Port: PUSHORT,
    Buffer: PUSHORT,
    Count: ULONG,
) {
    let _ = HwDeviceExtension;
    READ_PORT_BUFFER_USHORT(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadPortUchar(
    HwDeviceExtension: PVOID,
    Port: PUCHAR,
) -> UCHAR {
    return READ_PORT_UCHAR(Port);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadPortUlong(
    HwDeviceExtension: PVOID,
    Port: PULONG,
) -> ULONG {
    return READ_PORT_ULONG(Port);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadPortUshort(
    HwDeviceExtension: PVOID,
    Port: PUSHORT,
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
    let _ = HwDeviceExtension;
    READ_REGISTER_BUFFER_UCHAR(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadRegisterBufferUlong(
    HwDeviceExtension: PVOID,
    Register: PULONG,
    Buffer: PULONG,
    Count: ULONG,
) {
    let _ = HwDeviceExtension;
    READ_REGISTER_BUFFER_ULONG(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadRegisterBufferUshort(
    HwDeviceExtension: PVOID,
    Register: PUSHORT,
    Buffer: PUSHORT,
    Count: ULONG,
) {
    let _ = HwDeviceExtension;
    READ_REGISTER_BUFFER_USHORT(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadRegisterUchar(
    HwDeviceExtension: PVOID,
    Register: PUCHAR,
) -> UCHAR {
    return READ_REGISTER_UCHAR(Register);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadRegisterUlong(
    HwDeviceExtension: PVOID,
    Register: PULONG,
) -> ULONG {
    return READ_REGISTER_ULONG(Register);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReadRegisterUshort(
    HwDeviceExtension: *mut core::ffi::c_void,
    Register: *mut u16,
) -> u16 {
    return READ_REGISTER_USHORT(Register);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWritePortBufferUchar(
    HwDeviceExtension: PVOID,
    Port: PUCHAR,
    Buffer: PUCHAR,
    Count: ULONG,
) {
    WRITE_PORT_BUFFER_UCHAR(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWritePortBufferUlong(
    HwDeviceExtension: PVOID,
    Port: PULONG,
    Buffer: PULONG,
    Count: ULONG,
) {
    WRITE_PORT_BUFFER_ULONG(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWritePortBufferUshort(
    HwDeviceExtension: PVOID,
    Port: PUSHORT,
    Buffer: PUSHORT,
    Count: ULONG,
) {
    WRITE_PORT_BUFFER_USHORT(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWritePortUchar(
    HwDeviceExtension: PVOID,
    Port: PUCHAR,
    Value: UCHAR,
) {
    WRITE_PORT_UCHAR(Port, Value);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWritePortUlong(
    HwDeviceExtension: PVOID,
    Port: PULONG,
    Value: ULONG,
) {
    WRITE_PORT_ULONG(Port, Value);
}
#[no_mangle]
pub unsafe extern "system" fn StorPortWritePortUshort(HwDeviceExtension: PVOID, Port: PUSHORT, Value: USHORT) {
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
    HwDeviceExtension: *mut core::ffi::c_void,
    Register: *mut u16,
    Buffer: *mut u16,
    Count: u32,
) {
    WRITE_REGISTER_BUFFER_USHORT(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWriteRegisterUchar(
    HwDeviceExtension: *mut core::ffi::c_void,
    Register: *mut u8,
    Value: u8,
) {
    WRITE_REGISTER_UCHAR(Register, Value);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWriteRegisterUlong(
    HwDeviceExtension: PVOID,
    Register: PULONG,
    Value: ULONG,
) {
    let _ = HwDeviceExtension;
    WRITE_REGISTER_ULONG(Register, Value);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortWriteRegisterUshort(
    HwDeviceExtension: PVOID,
    Register: PUSHORT,
    Value: USHORT,
) {
    WRITE_REGISTER_USHORT(Register, Value);
}
