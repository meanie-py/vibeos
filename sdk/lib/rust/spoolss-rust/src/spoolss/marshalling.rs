use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn MarshallDownStructure(pStructure: PVOID, mut pInfo: *const MARSHALLING_INFO, cbStructureSize: DWORD, bSomeBoolean: BOOL) -> BOOL {
    // Sanity checks
    if pStructure.is_null() || pInfo.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    // Loop until we reach an element with offset set to MAXDWORD.
    while (*pInfo).dwOffset != MAXDWORD as DWORD {
        let mut pCurrentField: PULONG_PTR =
            (pStructure as PBYTE).offset((*pInfo).dwOffset as isize) as PULONG_PTR;

        if (*pInfo).bAdjustAddress != 0 && *pCurrentField != 0 {
            // Make a relative offset out of the absolute pointer address.
            *pCurrentField = (*pCurrentField).wrapping_sub(pStructure as ULONG_PTR);
        }

        // Advance to the next field description.
        pInfo = pInfo.offset(1);
    }

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn MarshallDownStructuresArray(
    mut pStructuresArray: PVOID,
    mut cElements: DWORD,
    pInfo: *const MARSHALLING_INFO,
    cbStructureSize: DWORD,
    bSomeBoolean: BOOL,
) -> BOOL {
    let mut pCurrentElement: PBYTE = pStructuresArray as PBYTE;

    // Call MarshallDownStructure on all array elements given by cElements of cbStructureSize.
    while {
        let fresh = cElements;
        cElements = cElements.wrapping_sub(1);
        fresh != 0
    } {
        if MarshallDownStructure(pCurrentElement as PVOID, pInfo, cbStructureSize, bSomeBoolean) == 0 {
            return (FALSE) as BOOL;
        }

        // Advance to the next array element.
        pCurrentElement = pCurrentElement.add(cbStructureSize as usize);
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn MarshallUpStructure(cbSize: DWORD, pStructure: PVOID, pInfo: *const MARSHALLING_INFO, cbStructureSize: DWORD, bSomeBoolean: BOOL) -> BOOL {
    let mut pInfo = pInfo;

    // Sanity checks
    if pStructure.is_null() || pInfo.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    // Loop until we reach an element with offset set to MAXDWORD.
    while (*pInfo).dwOffset != MAXDWORD {
        let mut pCurrentField: PULONG_PTR =
            (pStructure as PBYTE).offset((*pInfo).dwOffset as isize) as PULONG_PTR;

        if (*pInfo).bAdjustAddress != 0 && *pCurrentField != 0 {
            // Verify that the offset in the current field is within the bounds given by cbSize.
            if (cbSize as ULONG_PTR) <= *pCurrentField {
                SetLastError(ERROR_INVALID_DATA);
                return (FALSE) as BOOL;
            }

            // Make an absolute pointer address out of the relative offset.
            *pCurrentField = (*pCurrentField).wrapping_add(pStructure as ULONG_PTR);
        }

        // Advance to the next field description.
        pInfo = pInfo.offset(1);
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn MarshallUpStructuresArray(cbSize: DWORD, pStructuresArray: PVOID, mut cElements: DWORD, pInfo: *const MARSHALLING_INFO, cbStructureSize: DWORD, bSomeBoolean: BOOL) -> BOOL {
    let mut pCurrentElement: PBYTE = pStructuresArray as PBYTE;

    // Call MarshallUpStructure on all array elements given by cElements of cbStructureSize.
    loop {
        let fresh0 = cElements;
        cElements = cElements.wrapping_sub(1);
        if !(fresh0 != 0) {
            break;
        }
        if MarshallUpStructure(cbSize, pCurrentElement as PVOID, pInfo, cbStructureSize, bSomeBoolean) == 0 {
            return (FALSE) as BOOL;
        }

        // Advance to the next array element.
        pCurrentElement = pCurrentElement.offset(cbStructureSize as isize);
    }

    return (TRUE) as BOOL;
}
