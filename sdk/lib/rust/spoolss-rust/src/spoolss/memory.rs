use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: PCHAR);
}

#[no_mangle]
pub unsafe extern "system" fn AlignRpcPtr(mut pBuffer: PVOID, mut pcbBuffer: PDWORD) -> PVOID {
    if pcbBuffer.is_null() {
        RtlAssert(
            b"pcbBuffer\0" as *const u8 as *const i8 as PVOID,
            b"win32ss/printing/base/spoolss/memory.c\0"
                as *const u8 as *const i8 as PVOID,
            31 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    *pcbBuffer = *pcbBuffer - (*pcbBuffer % (core::mem::size_of::<DWORD>() as DWORD));
    if (pBuffer as usize) % core::mem::size_of::<DWORD>() != 0 {
        pBuffer = DllAllocSplMem(*pcbBuffer);
    }
    return pBuffer;
}

#[no_mangle]
pub unsafe extern "system" fn AllocSplStr(pwszInput: PCWSTR) -> PWSTR {
    let mut cbInput: DWORD;
    let mut pwszOutput: PWSTR;

    // Sanity check
    if pwszInput.is_null() {
        return core::ptr::null_mut();
    }

    // Get the length of the input string.
    cbInput = (wcslen(pwszInput) as DWORD + 1)
        * core::mem::size_of::<WCHAR>() as DWORD;

    // Allocate it. We don't use DllAllocSplMem here, because it unnecessarily zeroes the memory.
    pwszOutput = HeapAlloc(
        hProcessHeap,
        0,
        cbInput as SIZE_T,
    ) as PWSTR;
    if pwszOutput.is_null() {

        return core::ptr::null_mut();
    }

    // Copy the string and return it.
    CopyMemory!(
        pwszOutput as *mut core::ffi::c_void,
        pwszInput as *const core::ffi::c_void,
        cbInput as usize,
    );
    return pwszOutput;
}

#[no_mangle]
pub unsafe extern "system" fn DllAllocSplMem(dwBytes: DWORD) -> PVOID {
    return HeapAlloc(hProcessHeap, HEAP_ZERO_MEMORY as DWORD, dwBytes as SIZE_T);
}

#[no_mangle]
pub unsafe extern "system" fn DllFreeSplMem(mut pMem: PVOID) -> BOOL {
    if pMem.is_null() {
        return TRUE as BOOL;
    }
    return HeapFree(hProcessHeap, 0, pMem);
}

#[no_mangle]
pub unsafe extern "system" fn DllFreeSplStr(mut pwszString: PWSTR) -> BOOL {
    if !pwszString.is_null() {
        return HeapFree(hProcessHeap, 0, pwszString as PVOID);
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ReallocSplMem(pOldMem: PVOID, cbOld: DWORD, cbNew: DWORD) -> PVOID {
    let mut pNewMem: PVOID;

    // Always allocate the new block of memory.
    pNewMem = DllAllocSplMem(cbNew);
    if pNewMem.is_null() {
        return core::ptr::null_mut();
    }

    // Copy the old memory into the new block and free it.
    if !pOldMem.is_null() {
        CopyMemory!(pNewMem, pOldMem, min(cbOld, cbNew));
        DllFreeSplMem(pOldMem);
    }

    return pNewMem;
}

#[no_mangle]
pub unsafe extern "system" fn ReallocSplStr(ppwszString: *mut PWSTR, pwszInput: PCWSTR) -> BOOL {
    if !(*ppwszString).is_null() {
        DllFreeSplStr(*ppwszString);
    }

    *ppwszString = AllocSplStr(pwszInput);

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn UndoAlignRpcPtr(
    pDestinationBuffer: PVOID,
    pSourceBuffer: PVOID,
    cbBuffer: DWORD,
    pcbNeeded: PDWORD,
) -> PDWORD {
    if !(!pDestinationBuffer.is_null() || pSourceBuffer == pDestinationBuffer || cbBuffer == 0) {
        RtlAssert(
            b"pDestinationBuffer || pSourceBuffer == pDestinationBuffer || cbBuffer == 0\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"win32ss/printing/base/spoolss/memory.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            242 as ULONG,
            ::core::ptr::null_mut::<CHAR>() as PCHAR,
        );
    } else {
    }
    if !pSourceBuffer.is_null() && pSourceBuffer != pDestinationBuffer {
        CopyMemory!(pDestinationBuffer, pSourceBuffer, cbBuffer as usize);
        DllFreeSplMem(pSourceBuffer);
    }
    if !pcbNeeded.is_null()
        && (*pcbNeeded as usize).wrapping_rem(::core::mem::size_of::<DWORD>()) != 0
    {
        *pcbNeeded = (*pcbNeeded).wrapping_add(
            (::core::mem::size_of::<DWORD>() as DWORD)
                .wrapping_sub(*pcbNeeded % (::core::mem::size_of::<DWORD>() as DWORD)),
        );
    }
    return pcbNeeded;
}
