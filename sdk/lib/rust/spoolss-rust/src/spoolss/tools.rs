use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn PackStrings(
    mut pSource: *mut PCWSTR,
    mut pDest: PBYTE,
    mut DestOffsets: *const DWORD,
    mut pEnd: PBYTE,
) -> PBYTE {
    let mut cbString: DWORD;
    let mut StringAddress: ULONG_PTR;

    // Loop until we reach an element with offset set to MAXDWORD.
    while *DestOffsets != MAXDWORD {
        StringAddress = 0 as ULONG_PTR;

        if !(*pSource).is_null() {
            // Determine the length of the source string.
            cbString = (wcslen(*pSource) as DWORD + 1)
                * core::mem::size_of::<WCHAR>() as DWORD;

            // Copy it before the last string.
            pEnd = pEnd.offset(-(cbString as isize));
            StringAddress = pEnd as ULONG_PTR;
            CopyMemory!(
                pEnd as *mut core::ffi::c_void,
                *pSource as *const core::ffi::c_void,
                cbString as usize,
            );
        }

        // Copy the address of the copied string to the location given by the offset.
        CopyMemory!(
            pDest.offset(*DestOffsets as isize) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(StringAddress) as *const core::ffi::c_void,
            core::mem::size_of::<ULONG_PTR>(),
        );

        // Advance to the next source string and destination offset.
        pSource = pSource.offset(1);
        DestOffsets = DestOffsets.offset(1);
    }

    // pEnd is now at the last string we copied. Return this value as a pointer to the beginning of all strings in the output buffer.
    return pEnd;
}
