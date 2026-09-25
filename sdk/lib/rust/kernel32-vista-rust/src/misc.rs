//! GetTickCount64.c, GetNamedPipeClientProcessId.c, SetThreadDescription.c, InitOnce.c.

use crate::common::*;
use core::ffi::c_int;

// ===========================================================================
// GetTickCount64.c
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn GetTickCount64() -> ULONGLONG {
    let mut high: ULONG;
    let mut low: ULONG;
    // Volatile seqlock capture — plain loads get hoisted out of the retry loop
    // (spin_loop() is a CPU hint, not a compiler memory barrier).
    let tc = core::ptr::addr_of!((*SharedUserData).__bindgen_anon_2.TickCount);
    loop {
        high = core::ptr::read_volatile(core::ptr::addr_of!((*tc).High1Time)) as ULONG;
        low = core::ptr::read_volatile(core::ptr::addr_of!((*tc).LowPart));
        if high == core::ptr::read_volatile(core::ptr::addr_of!((*tc).High2Time)) as ULONG {
            break;
        }
        core::hint::spin_loop(); // YieldProcessor
    }
    // UInt32x32To64(x, mult): 32x32 -> 64
    let mult = (*SharedUserData).TickCountMultiplier as u64;
    ((low as u64 * mult) >> 24) + ((high as u64 * mult) << 8)
}

// ===========================================================================
// GetNamedPipeClientProcessId.c
// ===========================================================================
const FILE_DEVICE_NAMED_PIPE: ULONG = 0x0000_0011;
const METHOD_BUFFERED: ULONG = 0;
const FILE_ANY_ACCESS: ULONG = 0;
// CTL_CODE(FILE_DEVICE_NAMED_PIPE, 12, METHOD_BUFFERED, FILE_ANY_ACCESS)
const FSCTL_PIPE_GET_CONNECTION_ATTRIBUTE: ULONG =
    (FILE_DEVICE_NAMED_PIPE << 16) | (FILE_ANY_ACCESS << 14) | (12u32 << 2) | METHOD_BUFFERED;

#[inline]
unsafe fn set_ntstatus(status: NTSTATUS) -> BOOL {
    if status != 0 {
        SetLastError(RtlNtStatusToDosError(status));
    }
    (status == 0) as BOOL // C: return !status;
}

#[no_mangle]
pub unsafe extern "system" fn GetNamedPipeClientProcessId(pipe: HANDLE, id: *mut ULONG) -> BOOL {
    let mut iosb: IO_STATUS_BLOCK = core::mem::zeroed();
    // sizeof("ClientProcessId") includes the NUL = 16
    const ATTR: &[u8] = b"ClientProcessId\0";
    set_ntstatus(NtFsControlFile(
        pipe,
        core::ptr::null_mut(),
        None,
        core::ptr::null_mut(),
        &mut iosb,
        FSCTL_PIPE_GET_CONNECTION_ATTRIBUTE,
        ATTR.as_ptr() as PVOID,
        ATTR.len() as ULONG,
        id as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    ))
}

// ===========================================================================
// SetThreadDescription.c
// ===========================================================================
const USHRT_MAX: c_int = 0xFFFF;

#[inline]
fn HRESULT_FROM_NT(x: NTSTATUS) -> HRESULT {
    // ((HRESULT)(x) | FACILITY_NT_BIT)
    (x | 0x1000_0000) as HRESULT
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadDescription(thread: HANDLE, description: PCWSTR) -> HRESULT {
    let mut info: THREAD_NAME_INFORMATION = core::mem::zeroed();

    let length: c_int = if !description.is_null() {
        lstrlenW(description) * core::mem::size_of::<WCHAR>() as c_int
    } else {
        0
    };

    if length > USHRT_MAX {
        return HRESULT_FROM_NT(STATUS_INVALID_PARAMETER);
    }

    info.ThreadName.Length = length as USHORT;
    info.ThreadName.MaximumLength = length as USHORT;
    info.ThreadName.Buffer = description as *mut WCHAR;

    HRESULT_FROM_NT(NtSetInformationThread(
        thread,
        ThreadNameInformation,
        &mut info as *mut THREAD_NAME_INFORMATION as PVOID,
        core::mem::size_of::<THREAD_NAME_INFORMATION>() as ULONG,
    ))
}

// ===========================================================================
// InitOnce.c
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn InitOnceExecuteOnce(
    InitOnce: PRTL_RUN_ONCE,
    InitFn: PRTL_RUN_ONCE_INIT_FN,
    Parameter: PVOID,
    Context: *mut PVOID,
) -> BOOL {
    NT_SUCCESS(RtlRunOnceExecuteOnce(InitOnce, InitFn, Parameter, Context)) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn InitOnceBeginInitialize(
    lpInitOnce: PRTL_RUN_ONCE,
    dwFlags: DWORD,
    fPending: *mut BOOL,
    lpContext: *mut PVOID,
) -> BOOL {
    let Status = RtlRunOnceBeginInitialize(lpInitOnce, dwFlags, lpContext);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }
    *fPending = (Status == STATUS_PENDING) as BOOL;
    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn InitOnceComplete(
    lpInitOnce: PRTL_RUN_ONCE,
    dwFlags: DWORD,
    lpContext: PVOID,
) -> BOOL {
    let Status = RtlRunOnceComplete(lpInitOnce, dwFlags, lpContext);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }
    TRUE as BOOL
}

// ===========================================================================
// ThreadDescription.c — GetThreadDescription
// ===========================================================================
// Added by the 2026-09-20 upstream sync. Upstream moved SetThreadDescription out of
// SetThreadDescription.c into a new ThreadDescription.c and added its Get counterpart
// (`b0f7cb6`-era work by Hermès Bélusca-Maïto), exporting it from kernel32_vista.spec.
// Ported here rather than building their C, because that TU also redefines
// SetThreadDescription, which the Rust above already owns — linking it would have
// silently taken the C over the Rust.
//
// The retry loop is upstream's and is load-bearing: a concurrent SetThreadDescription can
// grow the name between the sizing query and the fetch.
use core::ffi::c_void;

extern "C" {
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

extern "system" {
    fn NtQueryInformationThread(
        ThreadHandle: HANDLE,
        ThreadInformationClass: THREADINFOCLASS,
        ThreadInformation: PVOID,
        ThreadInformationLength: ULONG,
        ReturnLength: *mut ULONG,
    ) -> NTSTATUS;
    fn LocalAlloc(uFlags: UINT, uBytes: usize) -> *mut c_void;
    fn LocalReAlloc(hMem: *mut c_void, uBytes: usize, uFlags: UINT) -> *mut c_void;
    fn LocalFree(hMem: *mut c_void) -> *mut c_void;
}

const LMEM_FIXED: UINT = 0x0000;
const LMEM_MOVEABLE: UINT = 0x0002;

#[no_mangle]
pub unsafe extern "system" fn GetThreadDescription(
    hThread: HANDLE,
    ppszThreadDescription: *mut PWSTR,
) -> HRESULT {
    if ppszThreadDescription.is_null() {
        return HRESULT_FROM_NT(STATUS_INVALID_PARAMETER);
    }
    *ppszThreadDescription = core::ptr::null_mut();

    let mut name_info: *mut THREAD_NAME_INFORMATION = core::ptr::null_mut();
    let mut length: ULONG = 0;

    let mut status = NtQueryInformationThread(
        hThread,
        ThreadNameInformation,
        core::ptr::null_mut(),
        0,
        &mut length,
    );
    if status != STATUS_BUFFER_TOO_SMALL {
        return HRESULT_FROM_NT(status);
    }

    while status == STATUS_INFO_LENGTH_MISMATCH || status == STATUS_BUFFER_TOO_SMALL {
        let ptr = if name_info.is_null() {
            LocalAlloc(LMEM_FIXED, length as usize)
        } else {
            LocalReAlloc(name_info as *mut c_void, length as usize, LMEM_MOVEABLE)
        };
        if ptr.is_null() {
            status = STATUS_NO_MEMORY;
            break;
        }
        name_info = ptr as *mut THREAD_NAME_INFORMATION;
        status = NtQueryInformationThread(
            hThread,
            ThreadNameInformation,
            name_info as PVOID,
            length,
            &mut length,
        );
    }
    if !NT_SUCCESS(status) {
        if !name_info.is_null() {
            LocalFree(name_info as *mut c_void);
        }
        return HRESULT_FROM_NT(status);
    }

    // The buffer holds a UNICODE_STRING followed by its characters. Slide the characters
    // to the front and shrink: sizeof(UNICODE_STRING) > sizeof(WCHAR), so the NUL fits.
    let out = name_info as PWSTR;
    let chars = (*name_info).ThreadName.Length;
    if chars != 0 {
        memmove(
            out as *mut c_void,
            (*name_info).ThreadName.Buffer as *const c_void,
            chars as usize,
        );
    }
    *out.add(chars as usize / core::mem::size_of::<WCHAR>()) = 0;

    // Shrink to fit. On failure the oversized buffer is still perfectly valid.
    let shrunk = LocalReAlloc(
        out as *mut c_void,
        chars as usize + core::mem::size_of::<WCHAR>(),
        LMEM_FIXED,
    );
    *ppszThreadDescription = if shrunk.is_null() { out } else { shrunk as PWSTR };

    HRESULT_FROM_NT(status)
}
