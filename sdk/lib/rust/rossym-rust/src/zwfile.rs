//! [correctness-review 2026-06-07 · sonnet; FIXED 2026-06-07 · opus] Reviewed vs sdk/lib/rossym/zwfile.c.
//! Risk: MED → RESOLVED. FIXED: RosSymZwReadFile compares `Information == size as usize` (full
//! ULONG_PTR width, matching C) instead of the truncating `Information as u32 == size`.
//! Risk: MED. Behavior: 1 truncation concern, otherwise preserved. [original finding]
//! Functions reviewed: RosSymZwReadFile, RosSymZwSeekFile (2/2).
//! Findings:
//!   CONCERN — RosSymZwReadFile: `io_status_block.Information as u32 == size`.
//!     `Information` is ULONG_PTR (usize: 32-bit on i386, 64-bit on amd64/arm64).
//!     `size` is u32. Casting Information to u32 silently truncates on 64-bit if
//!     Information > 0xFFFFFFFF — impossible in practice for any file read that fit in
//!     a u32 `size` arg, but semantically incorrect. The correct comparison is:
//!     `io_status_block.Information == size as usize`. No real-world breakage expected.
//!   NOTE — RosSymZwSeekFile: `Position as u32` assigned to LowPart with HighPart=0.
//!     C does the same: `NewPosition.CurrentByteOffset.u.LowPart = Position` where
//!     LowPart is ULONG (u32), implicitly truncating ULONG_PTR on 64-bit. Rust
//!     faithfully reproduces the C behavior (which is itself a latent 64-bit bug in C).
//!     Not a Rust translation defect.
//!   NOTE — ABI: both functions are `pub unsafe extern "C"` with `#[no_mangle]`.
//!     The C declarations in rossympriv.h use default (cdecl) linkage. Correct.
//!   NOTE — ZwReadFile ByteOffset and Key args: Rust passes null *mut LARGE_INTEGER and
//!     null *mut u32 respectively, matching C's NULL, NULL. Correct.
//! Unverified: Whether `_FILE_INFORMATION_CLASS_FilePositionInformation` resolves to the
//!   correct enum discriminant (value 14) from ntkrnl_sys::ddk at compile time.
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{FILE_POSITION_INFORMATION, _FILE_INFORMATION_CLASS_FilePositionInformation, _IO_STATUS_BLOCK, _LARGE_INTEGER, HANDLE, ZwReadFile, ZwSetInformationFile};
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
            None,
            core::ptr::null_mut::<core::ffi::c_void>(),
            iosb_ptr,
            buffer,
            size,
            core::ptr::null_mut::<_LARGE_INTEGER>(),
            core::ptr::null_mut::<u32>(),
        );

        // C compares ULONG_PTR Information against ULONG Size at full width (Size promotes).
        // Information's bindgen type is arch-dependent (u32 on i386, usize on 64-bit), so widen
        // both to usize — faithful and non-truncating, vs the original truncating `as u32`.
        (NT_SUCCESS(status) && (io_status_block.Information as usize == size as usize)) as u8
    }
}
#[no_mangle]
pub unsafe extern "C" fn RosSymZwSeekFile(FileContext: *mut core::ffi::c_void, Position: usize) -> u8 {
    unsafe {
        let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
        let mut new_position: FILE_POSITION_INFORMATION = core::mem::zeroed();
        new_position.CurrentByteOffset.u.HighPart = 0;
        new_position.CurrentByteOffset.u.LowPart = Position as u32;
        let status = ZwSetInformationFile(
            *(FileContext as *mut HANDLE),
            core::ptr::addr_of_mut!(io_status_block),
            core::ptr::addr_of_mut!(new_position) as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_POSITION_INFORMATION>() as u32,
            _FILE_INFORMATION_CLASS_FilePositionInformation,
        );
        return NT_SUCCESS(status) as u8;
    }
}
