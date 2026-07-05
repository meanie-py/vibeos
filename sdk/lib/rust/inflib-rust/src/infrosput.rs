//! infrosput.rs — ReactOS public INF write/add API.
//! Translated from sdk/lib/inflib/infrosput.c.
//!
//! Fixes applied:
//!  - `extern "C" fn InfpAddLineWithKey` extern block removed; call via Rust path.
//!  - `InfFindOrAddSection` return: was `as i32` (wrong for BOOLEAN) → `as BOOLEAN`.
//!  - `InfHostAddLine` / `InfHostAddField` corrected to public API names:
//!    `InfAddLine` / `InfAddField` (from infros.h lines 73-74).
//!  - INFSTATUS = signed i32 from crate::types.
//!  - NtOpenFile / NtWriteFile / NtClose via extern "C" blocks (plain, no ntkrnl_sys ddk).

use crate::types::*;
use crate::infput::{InfpFindOrAddSection, InfpAddLineWithKey, InfpAddField, InfpBuildFileBuffer};
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use core::ffi::c_void;

extern "C" {
    pub static mut InfpHeap: *mut c_void;
    fn DbgPrint(fmt: *const u8, ...) -> i32;
    fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
}

extern "system" {
    fn RtlAllocateHeap(HeapHandle: *mut c_void, Flags: ULONG, Size: SIZE_T) -> *mut c_void;
    fn RtlFreeHeap(HeapHandle: *mut c_void, Flags: ULONG, BaseAddress: *mut c_void);

    fn NtOpenFile(
        FileHandle:       *mut HANDLE,
        DesiredAccess:    ULONG,
        ObjectAttributes: PVOID,
        IoStatusBlock:    PVOID,
        ShareAccess:      ULONG,
        OpenOptions:      ULONG,
    ) -> NTSTATUS;

    fn NtWriteFile(
        FileHandle:    HANDLE,
        Event:         HANDLE,
        ApcRoutine:    PVOID,
        ApcContext:    PVOID,
        IoStatusBlock: PVOID,
        Buffer:        PVOID,
        Length:        ULONG,
        ByteOffset:    PVOID,
        Key:           PVOID,
    ) -> NTSTATUS;

    fn NtClose(Handle: HANDLE) -> NTSTATUS;
}

#[inline(always)]
unsafe fn MALLOC(size: usize) -> *mut c_void {
    RtlAllocateHeap(InfpHeap, 0, size)
}
#[inline(always)]
unsafe fn FREE(ptr: *mut c_void) {
    RtlFreeHeap(InfpHeap, 0, ptr);
}

// Minimal OBJECT_ATTRIBUTES / IO_STATUS_BLOCK
#[repr(C)]
struct OBJECT_ATTRIBUTES {
    Length:                   u32,
    RootDirectory:            PVOID,
    ObjectName:               PVOID,
    Attributes:               u32,
    SecurityDescriptor:       PVOID,
    SecurityQualityOfService: PVOID,
}
#[repr(C)]
struct IO_STATUS_BLOCK {
    StatusOrPointer: usize,
    Information:     usize,
}

const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x20;
const FILE_NON_DIRECTORY_FILE:      u32 = 0x00000040;

#[no_mangle]
pub unsafe extern "C" fn InfWriteFile(
    InfHandle:     HINF,
    FileName:      *mut _UNICODE_STRING,
    HeaderComment: *mut _UNICODE_STRING,
) -> NTSTATUS {
    let mut Buffer:     PWCHAR = core::ptr::null_mut();
    let mut BufferSize: ULONG  = 0;

    let InfStatus = InfpBuildFileBuffer(
        InfHandle as PINFCACHE,
        &mut Buffer,
        &mut BufferSize,
    );
    if InfStatus < 0 {
        return InfStatus as NTSTATUS;
    }

    let mut ObjAttr = OBJECT_ATTRIBUTES {
        Length:                   core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
        ObjectName:               FileName as PVOID,
        RootDirectory:            core::ptr::null_mut(),
        Attributes:               0,
        SecurityDescriptor:       core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut IoStatusBlock = IO_STATUS_BLOCK { StatusOrPointer: 0, Information: 0 };
    let mut FileHandle: HANDLE = core::ptr::null_mut();

    let mut Status = NtOpenFile(
        &mut FileHandle,
        (GENERIC_WRITE | SYNCHRONIZE) as ULONG,
        &mut ObjAttr as *mut _ as PVOID,
        &mut IoStatusBlock as *mut _ as PVOID,
        0,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE,
    );
    if Status < 0 {
        DbgPrint(b"InfWriteFile: NtOpenFile failed %x\n\0".as_ptr(), Status);
        FREE(Buffer as *mut c_void);
        return Status;
    }

    if !HeaderComment.is_null() && (*HeaderComment).Length != 0 {
        let hdr_chars = (*HeaderComment).Length as usize / core::mem::size_of::<u16>();
        let hdr_size  = (hdr_chars + 7) * core::mem::size_of::<u16>();
        let hdr_buf   = MALLOC(hdr_size) as PWCHAR;
        if !hdr_buf.is_null() {
            // "; "
            let semi_space: [u16; 3] = [b';' as u16, b' ' as u16, 0u16];
            wcscpy(hdr_buf, semi_space.as_ptr());
            for i in 0..hdr_chars {
                *hdr_buf.add(2 + i) = *(*HeaderComment).Buffer.add(i);
            }
            // "\r\n\r\n"
            let crlf2: [u16; 5] = [b'\r' as u16, b'\n' as u16, b'\r' as u16, b'\n' as u16, 0u16];
            wcscpy(hdr_buf.add(2 + hdr_chars), crlf2.as_ptr());
            let _ = NtWriteFile(
                FileHandle, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
                &mut IoStatusBlock as *mut _ as PVOID,
                hdr_buf as PVOID, hdr_size as ULONG,
                core::ptr::null_mut(), core::ptr::null_mut(),
            );
            FREE(hdr_buf as *mut c_void);
        }
    }

    Status = NtWriteFile(
        FileHandle, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(),
        &mut IoStatusBlock as *mut _ as PVOID,
        Buffer as PVOID, BufferSize,
        core::ptr::null_mut(), core::ptr::null_mut(),
    );
    NtClose(FileHandle);
    FREE(Buffer as *mut c_void);

    if Status < 0 {
        DbgPrint(b"InfWriteFile: NtWriteFile failed %x\n\0".as_ptr(), Status);
        return Status;
    }
    STATUS_SUCCESS
}

/// FIXED: was returning `as i32` (wrong for BOOLEAN = u8 on most targets).
#[no_mangle]
pub unsafe extern "C" fn InfFindOrAddSection(
    InfHandle: HINF,
    Section:   PCWSTR,
    Context:   *mut PINFCONTEXT,
) -> BOOLEAN {
    (0 <= InfpFindOrAddSection(InfHandle as PINFCACHE, Section, Context)) as BOOLEAN
}

/// FIXED: public API name from infros.h is `InfAddLine` (not InfHostAddLine).
#[no_mangle]
pub unsafe extern "C" fn InfAddLine(Context: PINFCONTEXT, Key: PCWSTR) -> BOOLEAN {
    (0 <= InfpAddLineWithKey(Context, Key)) as BOOLEAN
}

/// FIXED: public API name from infros.h is `InfAddField` (not InfHostAddField).
#[no_mangle]
pub unsafe extern "C" fn InfAddField(Context: PINFCONTEXT, Data: PCWSTR) -> BOOLEAN {
    (0 <= InfpAddField(Context, Data)) as BOOLEAN
}
