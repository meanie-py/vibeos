use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut ProcessHeapHandle: HANDLE = NULL;

static mut CurrentAllocations: UINT = 0 as UINT;
#[no_mangle]
pub unsafe extern "C" fn AllocateMemory(Size: UINT) -> PVOID {
    let mut Pointer: PVOID = core::ptr::null_mut();

    if ProcessHeapHandle.is_null() {
        ProcessHeapHandle = GetProcessHeap();
    }

    Pointer = HeapAlloc(ProcessHeapHandle, HEAP_ZERO_MEMORY as DWORD, Size as SIZE_T);

    if Pointer.is_null() {
        return core::ptr::null_mut();
    }

    CurrentAllocations = CurrentAllocations.wrapping_add(1);

    return Pointer;
}
#[no_mangle]
pub unsafe extern "C" fn FreeMemory(Pointer: PVOID) {
    SND_ASSERT!(ProcessHeapHandle);
    SND_ASSERT!(Pointer);

    HeapFree(ProcessHeapHandle, 0, Pointer);

    CurrentAllocations = CurrentAllocations.wrapping_sub(1);
}
#[no_mangle]
pub unsafe extern "C" fn GetMemoryAllocationCount() -> UINT {
    return CurrentAllocations;
}
#[no_mangle]
pub unsafe extern "C" fn GetDigitCount(mut Number: UINT) -> UINT {
    let mut Value: UINT = Number;
    let mut Digits: ULONG = 1;

    while Value > 9 {
        Value /= 10;
        Digits += 1;
    }

    return Digits as UINT;
}
#[no_mangle]
pub unsafe extern "C" fn Win32ErrorToMmResult(ErrorCode: UINT) -> MMRESULT {
    match ErrorCode {
        NO_ERROR | ERROR_IO_PENDING => {
            return MMSYSERR_NOERROR as MMRESULT;
        }
        ERROR_BUSY => {
            return MMSYSERR_ALLOCATED as MMRESULT;
        }
        ERROR_NOT_SUPPORTED | ERROR_INVALID_FUNCTION => {
            return MMSYSERR_NOTSUPPORTED as MMRESULT;
        }
        ERROR_NOT_ENOUGH_MEMORY => {
            return MMSYSERR_NOMEM as MMRESULT;
        }
        ERROR_ACCESS_DENIED => {
            return MMSYSERR_BADDEVICEID as MMRESULT;
        }
        ERROR_INSUFFICIENT_BUFFER => {
            return MMSYSERR_INVALPARAM as MMRESULT;
        }
        ERROR_INVALID_PARAMETER => {
            return MMSYSERR_INVALPARAM as MMRESULT;
        }
        _ => {
            return MMSYSERR_ERROR as MMRESULT;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn TranslateInternalMmResult(mut Result: MMRESULT) -> MMRESULT {
    match Result {
        MMSYSERR_INVALPARAM | MMSYSERR_INVALFLAG => {
            return MMSYSERR_ERROR as MMRESULT;
        }
        _ => {}
    }

    return Result;
}
