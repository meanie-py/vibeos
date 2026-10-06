use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn pOpenDeviceEx_UStr(
    mut DevicePath: PCUNICODE_STRING,
    mut DeviceHandle: PHANDLE,
    mut DesiredAccess: ACCESS_MASK,
    mut ShareAccess: ULONG,
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = DevicePath as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    return NtOpenFile(
        DeviceHandle,
        DesiredAccess | SYNCHRONIZE,
        &raw mut ObjectAttributes,
        &raw mut IoStatusBlock,
        ShareAccess,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
}
#[no_mangle]
pub unsafe extern "C" fn pOpenDevice_UStr(
    DevicePath: PCUNICODE_STRING,
    DeviceHandle: PHANDLE,
) -> NTSTATUS {
    return pOpenDeviceEx_UStr(
        DevicePath,
        DeviceHandle,
        FILE_READ_DATA | FILE_READ_ATTRIBUTES,
        FILE_SHARE_ALL,
    );
}
#[no_mangle]
pub unsafe extern "C" fn pOpenDeviceEx(
    DevicePath: PCWSTR,
    DeviceHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ShareAccess: ULONG,
) -> NTSTATUS {
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), DevicePath);
    return pOpenDeviceEx_UStr(
        core::ptr::addr_of_mut!(Name) as PCUNICODE_STRING,
        DeviceHandle,
        DesiredAccess,
        ShareAccess,
    );
}
#[no_mangle]
pub unsafe extern "C" fn pOpenDevice(
    DevicePath: PCWSTR,
    DeviceHandle: PHANDLE,
) -> NTSTATUS {
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), DevicePath);
    return pOpenDevice_UStr(core::ptr::addr_of!(Name), DeviceHandle);
}
