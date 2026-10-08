#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{KeStallExecutionProcessor};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{SCSI_PHYSICAL_ADDRESS};

// NTAPI = stdcall on i386 — extern "system" resolves per-arch.

#[no_mangle]
pub unsafe extern "system" fn ScsiPortConvertUlongToPhysicalAddress(
    UlongAddress: ULONG_PTR,
) -> SCSI_PHYSICAL_ADDRESS {
    let mut Address: SCSI_PHYSICAL_ADDRESS = core::mem::zeroed();
    Address.QuadPart = UlongAddress as i64;
    return Address;
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortReadPortBufferUchar(
    Port: *mut u8,
    Buffer: *mut u8,
    Count: u32,
) {
    READ_PORT_BUFFER_UCHAR(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortReadPortBufferUshort(
    Port: *mut u16,
    Buffer: *mut u16,
    Count: u32,
) {
    READ_PORT_BUFFER_USHORT(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortReadPortBufferUlong(
    Port: *mut u32,
    Buffer: *mut u32,
    Count: u32,
) {
    READ_PORT_BUFFER_ULONG(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortReadPortUchar(Port: *mut u8) -> u8 {
    return READ_PORT_UCHAR(Port);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortReadPortUshort(Port: *mut u16) -> u16 {
    READ_PORT_USHORT(Port)
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortReadPortUlong(Port: *mut u32) -> u32 {
    return READ_PORT_ULONG(Port);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortReadRegisterBufferUchar(
    Register: *mut u8,
    Buffer: *mut u8,
    Count: ULONG,
) {
    READ_REGISTER_BUFFER_UCHAR(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortReadRegisterBufferUshort(
    mut Register: *mut u16,
    mut Buffer: *mut u16,
    mut Count: u32,
) -> () {
    READ_REGISTER_BUFFER_USHORT(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortReadRegisterBufferUlong(
    Register: *mut ULONG,
    Buffer: *mut ULONG,
    Count: ULONG,
) {
    READ_REGISTER_BUFFER_ULONG(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortReadRegisterUchar(Register: PUCHAR) -> UCHAR {
    return READ_REGISTER_UCHAR(Register);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortReadRegisterUshort(Register: *mut u16) -> u16 {
    return READ_REGISTER_USHORT(Register);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortReadRegisterUlong(Register: *mut u32) -> u32 {
    return READ_REGISTER_ULONG(Register as *mut u32);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortWritePortBufferUchar(
    Port: *mut u8,
    Buffer: *mut u8,
    Count: u32,
) {
    WRITE_PORT_BUFFER_UCHAR(Port, Buffer as *const u8, Count);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortWritePortBufferUshort(
    Port: *mut u16,
    Buffer: *mut u16,
    Count: u32,
) {
    WRITE_PORT_BUFFER_USHORT(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortWritePortBufferUlong(
    Port: PULONG,
    Buffer: PULONG,
    Count: ULONG,
) {
    WRITE_PORT_BUFFER_ULONG(Port, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortWritePortUchar(mut Port: *mut u8, mut Value: u8) {
    WRITE_PORT_UCHAR(Port, Value);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortWritePortUshort(Port: PUSHORT, Value: USHORT) {
    WRITE_PORT_USHORT(Port, Value);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortWritePortUlong(Port: *mut u32, Value: u32) {
    WRITE_PORT_ULONG(Port, Value);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortWriteRegisterBufferUchar(
    Register: *mut u8,
    Buffer: *mut u8,
    Count: u32,
) {
    WRITE_REGISTER_BUFFER_UCHAR(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortWriteRegisterBufferUshort(
    Register: *mut u16,
    Buffer: *mut u16,
    Count: u32,
) {
    WRITE_REGISTER_BUFFER_USHORT(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortWriteRegisterBufferUlong(
    Register: *mut u32,
    Buffer: *mut u32,
    Count: u32,
) {
    WRITE_REGISTER_BUFFER_ULONG(Register, Buffer, Count);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortWriteRegisterUchar(
    Register: *mut u8,
    Value: u8,
) {
    WRITE_REGISTER_UCHAR(Register, Value);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortWriteRegisterUshort(mut Register: *mut u16, mut Value: u16) {
    WRITE_REGISTER_USHORT(Register as *mut u16, Value);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortWriteRegisterUlong(
    Register: *mut ULONG,
    Value: ULONG,
) {
    WRITE_REGISTER_ULONG(Register, Value);
}

#[no_mangle]
pub unsafe extern "system" fn ScsiPortStallExecution(Delay: u32) {
    KeStallExecutionProcessor(Delay);
}
