use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{FILE_POSITION_INFORMATION, FilePositionInformation, _IO_STATUS_BLOCK, ZwReadFile, ZwSetInformationFile};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn RosSymZwReadFile(
    file_context: *mut core::ffi::c_void,
    buffer: *mut core::ffi::c_void,
    size: u32,
) -> u8 {
    unsafe {
        let mut io_status_block: ntkrnl_sys::ddk::_IO_STATUS_BLOCK = core::mem::zeroed();
        let handle: *mut core::ffi::c_void;

        handle = *(file_context as *mut *mut core::ffi::c_void);

        let iosb_ptr = core::ptr::addr_of_mut!(io_status_block);

        let status = ntkrnl_sys::ddk::ZwReadFile(
            handle,
            core::ptr::null_mut::<core::ffi::c_void>(),
            core::ptr::null_mut::<core::ffi::c_void>(),
            core::ptr::null_mut::<core::ffi::c_void>(),
            iosb_ptr,
            buffer,
            size,
            core::ptr::null_mut::<i64>(),
            core::ptr::null_mut::<u32>(),
        );

        ((ntkrnl_sys::base::NT_SUCCESS(status) != 0) && (io_status_block.Information == size as usize)) as u8
    }
}
#[no_mangle]
pub unsafe extern "C" fn RosSymZwSeekFile(FileContext: *mut core::ffi::c_void, Position: usize) -> u8 {
    unsafe {
        let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut new_position: FILE_POSITION_INFORMATION = core::mem::zeroed();
        new_position.CurrentByteOffset.u.__bindgen_anon_1.HighPart = 0;
        new_position.CurrentByteOffset.u.__bindgen_anon_1.LowPart = Position as u32;
        let status = ZwSetInformationFile(
            *(FileContext as *mut HANDLE),
            core::ptr::addr_of_mut!(io_status_block),
            core::ptr::addr_of_mut!(new_position) as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_POSITION_INFORMATION>() as u32,
            FilePositionInformation,
        );
        return NT_SUCCESS(status) as u8;
    }
}
