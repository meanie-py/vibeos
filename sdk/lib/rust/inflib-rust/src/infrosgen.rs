//! infrosgen.rs — ReactOS-specific INF open/close (InfSetHeap, InfOpenFile, etc.)
//! Translated from sdk/lib/inflib/infrosgen.c.
//!
//! Fixes applied:
//!  - `InfpHeap` defined here as the single #[no_mangle] static; other modules
//!    reference it via `extern "C" { static mut InfpHeap: *mut c_void; }`.
//!  - `INFSTATUS = u32` / `INF_STATUS_NO_MEMORY = 1` removed; use crate::types (signed).
//!  - `INF_STATUS_INSUFFICIENT_RESOURCES` added from types.
//!  - `memset` → `core::ptr::write_bytes`.
//!  - `RtlCreateHeap` / `RtlDestroyHeap` / `RtlIsTextUnicode` / `RtlMultiByteToUnicodeN`
//!    / `NtOpenFile` / `NtReadFile` / `NtQueryInformationFile` / `NtClose` referenced via
//!    extern "C" blocks (same pattern as rossym-rust, matches ntkrnl_sys::ddk exports).
//!  - `FileStandardInformation` const value = 5 (from bindgen).
//!  - `LARGE_INTEGER.QuadPart as u32` for file length (avoids union field path complexity).

use crate::types::*;
use crate::infcore::{InfpFreeSection, InfpParseBuffer};
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use core::ffi::c_void;

extern "C" {
    fn RtlAllocateHeap(HeapHandle: *mut c_void, Flags: ULONG, Size: SIZE_T) -> *mut c_void;
    fn RtlFreeHeap(HeapHandle: *mut c_void, Flags: ULONG, BaseAddress: *mut c_void);
    fn DbgPrint(fmt: *const u8, ...) -> i32;

    fn RtlCreateHeap(
        Flags:       ULONG,
        HeapBase:    PVOID,
        ReserveSize: SIZE_T,
        CommitSize:  SIZE_T,
        Lock:        PVOID,
        Parameters:  PVOID,
    ) -> PVOID;
    fn RtlDestroyHeap(HeapHandle: PVOID) -> PVOID;

    fn RtlIsTextUnicode(buf: *const c_void, len: i32, flags: *mut i32) -> i32;

    fn RtlMultiByteToUnicodeN(
        UnicodeString: *mut u16,
        UnicodeSize:   ULONG,
        ResultSize:    *mut ULONG,
        MbString:      *const i8,
        MbSize:        ULONG,
    ) -> NTSTATUS;

    fn NtOpenFile(
        FileHandle:       *mut HANDLE,
        DesiredAccess:    ULONG,
        ObjectAttributes: PVOID,
        IoStatusBlock:    PVOID,
        ShareAccess:      ULONG,
        OpenOptions:      ULONG,
    ) -> NTSTATUS;

    fn NtQueryInformationFile(
        FileHandle:          HANDLE,
        IoStatusBlock:       PVOID,
        FileInformation:     PVOID,
        Length:              ULONG,
        FileInformationClass: i32,
    ) -> NTSTATUS;

    fn NtReadFile(
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

/// Single definition of InfpHeap (shared via `extern "C"` across all inflib modules).
#[no_mangle]
pub static mut InfpHeap: *mut c_void = core::ptr::null_mut();

static mut InfpHeapRefCount: i32 = 0;

const HEAP_GROWABLE:               u32 = 0x00000002;
const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x00002000;
const FILE_NON_DIRECTORY_FILE:      u32 = 0x00000040;
/// _FILE_INFORMATION_CLASS_FileStandardInformation = 5
const FILE_INFORMATION_CLASS_STANDARD: i32 = 5;

unsafe fn CheckHeap() {
    if InfpHeap.is_null() {
        InfpHeap = RtlCreateHeap(
            HEAP_GROWABLE,
            core::ptr::null_mut(),
            0,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }
    if 0 <= InfpHeapRefCount {
        InfpHeapRefCount += 1;
    }
}

#[inline(always)]
unsafe fn MALLOC(size: usize) -> *mut c_void {
    RtlAllocateHeap(InfpHeap, 0, size)
}
#[inline(always)]
unsafe fn FREE(ptr: *mut c_void) {
    RtlFreeHeap(InfpHeap, 0, ptr);
}

// Minimal FILE_STANDARD_INFORMATION layout (matches C struct: AllocationSize, EndOfFile, ...)
#[repr(C)]
struct FILE_STANDARD_INFORMATION {
    AllocationSize_lo: u32,
    AllocationSize_hi: i32,
    EndOfFile_lo:      u32,
    EndOfFile_hi:      i32,
    NumberOfLinks:     u32,
    DeletePending:     u8,
    Directory:         u8,
}

// Minimal OBJECT_ATTRIBUTES layout
#[repr(C)]
struct OBJECT_ATTRIBUTES {
    Length:                   u32,
    RootDirectory:            PVOID,
    ObjectName:               PVOID,
    Attributes:               u32,
    SecurityDescriptor:       PVOID,
    SecurityQualityOfService: PVOID,
}

// Minimal IO_STATUS_BLOCK layout
#[repr(C)]
struct IO_STATUS_BLOCK {
    StatusOrPointer: usize,
    Information:     usize,
}

// Minimal LARGE_INTEGER for file offset
#[repr(C)]
struct LARGE_INTEGER {
    QuadPart: i64,
}

// ──────────────────────────────────────────────────────────────────────────────
// Public functions
// ──────────────────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn InfSetHeap(Heap: PVOID) {
    if InfpHeap.is_null() {
        InfpHeap = Heap;
        InfpHeapRefCount = -1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn InfOpenBufferedFile(
    InfHandle:  PHINF,
    Buffer:     PVOID,
    BufferSize: ULONG,
    LanguageId: LANGID,
    ErrorLine:  PULONG,
) -> NTSTATUS {
    CheckHeap();
    *InfHandle = core::ptr::null_mut();
    *ErrorLine = !0u32;

    let file_buf_size = BufferSize.wrapping_add(2);
    let FileBuffer = MALLOC(file_buf_size as usize) as *mut i8;
    if FileBuffer.is_null() {
        return INF_STATUS_INSUFFICIENT_RESOURCES as NTSTATUS;
    }
    core::ptr::copy_nonoverlapping(Buffer as *const u8, FileBuffer as *mut u8, BufferSize as usize);
    *FileBuffer.add(BufferSize as usize)     = 0;
    *FileBuffer.add(BufferSize as usize + 1) = 0;

    let Cache = MALLOC(core::mem::size_of::<INFCACHE>()) as PINFCACHE;
    if Cache.is_null() {
        FREE(FileBuffer as *mut c_void);
        return INF_STATUS_INSUFFICIENT_RESOURCES as NTSTATUS;
    }
    core::ptr::write_bytes(Cache as *mut u8, 0, core::mem::size_of::<INFCACHE>());
    (*Cache).LanguageId = LanguageId;

    let mut Status: INFSTATUS;
    if RtlIsTextUnicode(FileBuffer as *const c_void, file_buf_size as i32, core::ptr::null_mut()) == 0 {
        let new_buff = MALLOC(
            (file_buf_size as usize).wrapping_mul(core::mem::size_of::<u16>())
        ) as *mut u16;
        if !new_buff.is_null() {
            let mut len: u32 = 0;
            let _ = RtlMultiByteToUnicodeN(
                new_buff,
                (file_buf_size as usize * core::mem::size_of::<u16>()) as u32,
                &mut len,
                FileBuffer,
                file_buf_size,
            );
            Status = InfpParseBuffer(
                Cache,
                new_buff,
                new_buff.add((len as usize) / core::mem::size_of::<u16>()),
                ErrorLine,
            );
            FREE(new_buff as *mut c_void);
        } else {
            Status = INF_STATUS_INSUFFICIENT_RESOURCES;
        }
    } else {
        let mut new_buff = FileBuffer as *mut u16;
        let mut remaining = file_buf_size;
        if *new_buff == 0xfeffu16 {
            new_buff  = new_buff.add(1);
            remaining = remaining.wrapping_sub(core::mem::size_of::<u16>() as u32);
        }
        Status = InfpParseBuffer(
            Cache,
            new_buff,
            (new_buff as *mut i8).add(remaining as usize) as *mut u16,
            ErrorLine,
        );
    }

    if Status < 0 {
        FREE(Cache as *mut c_void);
        *InfHandle = core::ptr::null_mut();
    } else {
        *InfHandle = Cache as *mut c_void;
    }
    FREE(FileBuffer as *mut c_void);
    Status as NTSTATUS
}

#[no_mangle]
pub unsafe extern "C" fn InfOpenFile(
    InfHandle:  PHINF,
    FileName:   *mut _UNICODE_STRING,
    LanguageId: LANGID,
    ErrorLine:  PULONG,
) -> NTSTATUS {
    CheckHeap();
    *InfHandle = core::ptr::null_mut();
    *ErrorLine = !0u32;

    let mut ObjAttr = OBJECT_ATTRIBUTES {
        Length:                   core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory:            core::ptr::null_mut(),
        ObjectName:               FileName as PVOID,
        Attributes:               0,
        SecurityDescriptor:       core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut IoStatusBlock = IO_STATUS_BLOCK { StatusOrPointer: 0, Information: 0 };
    let mut FileHandle: HANDLE = core::ptr::null_mut();

    let mut Status = NtOpenFile(
        &mut FileHandle,
        GENERIC_READ | SYNCHRONIZE,
        &mut ObjAttr as *mut _ as PVOID,
        &mut IoStatusBlock as *mut _ as PVOID,
        FILE_SHARE_READ,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE,
    );
    if Status < 0 {
        DbgPrint(b"InfOpenFile: NtOpenFile failed %x\n\0".as_ptr(), Status);
        return Status;
    }

    let mut FileInfo: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    Status = NtQueryInformationFile(
        FileHandle,
        &mut IoStatusBlock as *mut _ as PVOID,
        &mut FileInfo as *mut _ as PVOID,
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as u32,
        FILE_INFORMATION_CLASS_STANDARD,
    );
    if Status < 0 {
        NtClose(FileHandle);
        return Status;
    }

    let FileLength    = FileInfo.EndOfFile_lo;
    let FileBufferLen = FileLength.wrapping_add(2);
    let FileBuffer    = MALLOC(FileBufferLen as usize) as *mut i8;
    if FileBuffer.is_null() {
        NtClose(FileHandle);
        return INF_STATUS_INSUFFICIENT_RESOURCES as NTSTATUS;
    }

    let mut FileOffset = LARGE_INTEGER { QuadPart: 0 };
    Status = NtReadFile(
        FileHandle,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut IoStatusBlock as *mut _ as PVOID,
        FileBuffer as PVOID,
        FileLength,
        &mut FileOffset as *mut _ as PVOID,
        core::ptr::null_mut(),
    );
    *FileBuffer.add(FileLength as usize)     = 0;
    *FileBuffer.add(FileLength as usize + 1) = 0;
    NtClose(FileHandle);

    if Status < 0 {
        FREE(FileBuffer as *mut c_void);
        return Status;
    }

    let Cache = MALLOC(core::mem::size_of::<INFCACHE>()) as PINFCACHE;
    if Cache.is_null() {
        FREE(FileBuffer as *mut c_void);
        return INF_STATUS_INSUFFICIENT_RESOURCES as NTSTATUS;
    }
    core::ptr::write_bytes(Cache as *mut u8, 0, core::mem::size_of::<INFCACHE>());
    (*Cache).LanguageId = LanguageId;

    let inf_status: INFSTATUS;
    if RtlIsTextUnicode(FileBuffer as *const c_void, FileBufferLen as i32, core::ptr::null_mut()) == 0 {
        let new_buff = MALLOC(
            (FileBufferLen as usize).wrapping_mul(core::mem::size_of::<u16>())
        ) as *mut u16;
        if !new_buff.is_null() {
            let mut len: u32 = 0;
            let _ = RtlMultiByteToUnicodeN(
                new_buff,
                (FileBufferLen as usize * core::mem::size_of::<u16>()) as u32,
                &mut len,
                FileBuffer,
                FileBufferLen,
            );
            inf_status = InfpParseBuffer(
                Cache,
                new_buff,
                new_buff.add((len as usize) / core::mem::size_of::<u16>()),
                ErrorLine,
            );
            FREE(new_buff as *mut c_void);
        } else {
            inf_status = INF_STATUS_INSUFFICIENT_RESOURCES;
        }
    } else {
        let mut new_buff = FileBuffer as *mut u16;
        let mut remaining = FileBufferLen;
        if *new_buff == 0xfeffu16 {
            new_buff  = new_buff.add(1);
            remaining = remaining.wrapping_sub(core::mem::size_of::<u16>() as u32);
        }
        inf_status = InfpParseBuffer(
            Cache,
            new_buff,
            (new_buff as *mut i8).add(remaining as usize) as *mut u16,
            ErrorLine,
        );
    }

    if inf_status < 0 {
        FREE(Cache as *mut c_void);
        *InfHandle = core::ptr::null_mut();
    } else {
        *InfHandle = Cache as *mut c_void;
    }
    FREE(FileBuffer as *mut c_void);
    inf_status as NTSTATUS
}

#[no_mangle]
pub unsafe extern "C" fn InfCloseFile(InfHandle: HINF) {
    let Cache = InfHandle as PINFCACHE;
    if Cache.is_null() { return; }
    while !(*Cache).FirstSection.is_null() {
        (*Cache).FirstSection = InfpFreeSection((*Cache).FirstSection);
    }
    (*Cache).LastSection = core::ptr::null_mut();
    FREE(Cache as *mut c_void);
    if InfpHeapRefCount > 0 {
        InfpHeapRefCount -= 1;
        if InfpHeapRefCount == 0 {
            RtlDestroyHeap(InfpHeap);
            InfpHeap = core::ptr::null_mut();
        }
    }
}
