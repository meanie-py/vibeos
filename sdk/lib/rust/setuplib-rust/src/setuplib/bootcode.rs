use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn ReadBootCodeByHandle(
    BootCodeInfo: *mut BOOTCODE,
    FileHandle: HANDLE,
    Length: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut BootCode: PVOID;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut Length = Length;

    if Length == 0 || Length == (0u32).wrapping_sub(1) {
        Length = SECTORSIZE;
    }

    BootCode = RtlAllocateHeap(ProcessHeap, HEAP_ZERO_MEMORY, (Length) as SIZE_T);
    if BootCode.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    FileOffset.QuadPart = 0;
    Status = NtReadFile(
        FileHandle,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut IoStatusBlock,
        BootCode,
        Length,
        &mut FileOffset,
        core::ptr::null_mut(),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        RtlFreeHeap(ProcessHeap, 0, BootCode);
        return Status;
    }

    if !(*BootCodeInfo).BootCode.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*BootCodeInfo).BootCode);
    }
    (*BootCodeInfo).BootCode = BootCode;
    (*BootCodeInfo).Length = Length;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn ReadBootCodeFromFile(
    BootCodeInfo: *mut BOOTCODE,
    FilePath: PUNICODE_STRING,
    Length: u32,
) -> i32 {
    let mut Status: i32;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileHandle: *mut core::ffi::c_void = core::mem::zeroed();

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = FilePath;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenFile(
        &mut FileHandle,
        GENERIC_READ | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return Status;
    }

    Status = ReadBootCodeByHandle(BootCodeInfo, FileHandle, Length);

    NtClose(FileHandle);
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn FreeBootCode(BootCodeInfo: *mut BOOTCODE) {
    if !(*BootCodeInfo).BootCode.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*BootCodeInfo).BootCode);
    }
    (*BootCodeInfo).BootCode = core::ptr::null_mut();
    (*BootCodeInfo).Length = 0;
}
