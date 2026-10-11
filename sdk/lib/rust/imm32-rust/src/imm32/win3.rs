use crate::imm32::utils::IchAnsiFromWide;
use crate::imm32::utils::IchWideFromAnsi;
use crate::imm32::ime::ImmGetDefaultIMEWnd;
use crate::imm32::utils::ImmLocalAlloc;
use crate::imm32::imm::ImmLockIMC;
use crate::imm32::utils::ImmLockIMCC;
use crate::imm32::imm::ImmUnlockIMC;
use crate::imm32::utils::ImmUnlockIMCC;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const KOR_LEAD_BYTE_FIRST: u8 = 0xB0;
pub const KOR_LEAD_BYTE_LAST: u8 = 0xC8;
pub const KOR_SCAN_CODE_SBCS: LPARAM = 0xFFF10001u32 as LPARAM;
pub const KOR_SCAN_CODE_DBCS: LPARAM = 0xFFF20001u32 as LPARAM;
pub const KOR_KEYDOWN_FLAGS: LPARAM = 0xE0001u32 as LPARAM;
pub const KOR_INTERIM_FLAGS: LPARAM = 0xF00001u32 as LPARAM;

pub type FN_SendMessage = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;
pub type FN_PostMessage = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> BOOL;

unsafe extern "C" fn Imm32WideToAnsi(
    pchWide: PCWCH,
    cchWide: INT,
    pszAnsi: PSTR,
    cchAnsi: INT,
) -> INT {
    let mut usedDefaultChar: BOOL = (FALSE) as i32;
    let mut len: INT = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        pchWide as LPCWSTR,
        cchWide as ::core::ffi::c_int,
        pszAnsi as LPSTR,
        cchAnsi as ::core::ffi::c_int,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(usedDefaultChar),
    ) as INT;
    if !pszAnsi.is_null() && len >= 0 && len < cchAnsi {
        *pszAnsi.offset(len as isize) = ANSI_NULL;
    }
    return len;
}

unsafe extern "C" fn Imm32AnsiToWide(pchAnsi: PCCH, cchAnsi: INT, pszWide: PWSTR, cchWide: INT) -> INT {
    let mut len: INT = MultiByteToWideChar(CP_ACP, MB_PRECOMPOSED, pchAnsi, cchAnsi, pszWide, cchWide);
    if !pszWide.is_null() && len >= 0 && len < cchWide {
        *pszWide.offset(len as isize) = UNICODE_NULL;
    }
    return len;
}

unsafe extern "C" fn Imm32CompStrWToStringA(pCS: *const COMPOSITIONSTRING, pch: PCHAR) -> DWORD {
    let mut bUsedDef: BOOL = 0;
    let mut pwch: PCWSTR = (pCS as *const BYTE).offset((*pCS).dwResultStrOffset as isize) as PCWSTR;
    let mut cchNeed: INT = (WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        pwch as LPCWSTR,
        (*pCS).dwResultStrLen as ::core::ffi::c_int,
        pch as LPSTR,
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<CHAR>(),
        &raw mut bUsedDef,
    ) as INT) + 1 as INT;
    if pch.is_null() {
        return cchNeed as DWORD;
    }
    let mut cch: INT = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        pwch as LPCWSTR,
        (*pCS).dwResultStrLen as ::core::ffi::c_int,
        pch as LPSTR,
        cchNeed as ::core::ffi::c_int,
        ::core::ptr::null_mut::<CHAR>(),
        &raw mut bUsedDef,
    ) as INT;
    *pch.offset(cch as isize) = ANSI_NULL as CHAR;
    return ((cch as ::core::ffi::c_int) + 1 as ::core::ffi::c_int) as DWORD;
}

unsafe extern "C" fn Imm32CompStrWToStringW(pCS: *const COMPOSITIONSTRING, pch: PWCHAR) -> DWORD {
    let dwResultStrLen: DWORD = (*pCS).dwResultStrLen;
    if !pch.is_null() {
        let cb: DWORD = dwResultStrLen.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        RtlCopyMemory!(
            pch as *mut core::ffi::c_void,
            (pCS as *const BYTE).offset((*pCS).dwResultStrOffset as isize) as *const core::ffi::c_void,
            cb as usize,
        );
        *pch.offset(dwResultStrLen as isize) = UNICODE_NULL;
    }
    (dwResultStrLen.wrapping_add(1)).wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD)
}

unsafe extern "C" fn Imm32CompStrWToUndetW(
    dwGCS: DWORD,
    pCS: *const COMPOSITIONSTRING,
    pDet: PUNDETERMINESTRUCT,
    dwSize: DWORD,
) -> DWORD {
    let cbRequired: DWORD =
        core::mem::size_of::<UNDETERMINESTRUCT>() as DWORD
        + ALIGN_WSTR_SIZE!(((*pCS).dwCompStrLen as usize)) as DWORD
        + ALIGN_DWORD!((*pCS).dwCompAttrLen)
        + ALIGN_WSTR_SIZE!(((*pCS).dwResultStrLen as usize)) as DWORD
        + ALIGN_DWORD!((*pCS).dwResultClauseLen)
        + ALIGN_WSTR_SIZE!(((*pCS).dwResultReadStrLen as usize)) as DWORD
        + ALIGN_DWORD!((*pCS).dwResultReadClauseLen);

    if pDet.is_null() {
        return cbRequired;
    }

    if dwSize < cbRequired {
        return 0;
    }

    (*pDet).dwSize = cbRequired;

    let pbDet: PBYTE = pDet as PBYTE;
    let mut ib: DWORD = (core::mem::size_of::<UNDETERMINESTRUCT>()) as _;
    let pbCS: *const BYTE = pCS as *const BYTE;

    if (dwGCS & GCS_COMPSTR) != 0 {
        (*pDet).uUndetTextPos = ib as UINT;
        (*pDet).uUndetTextLen = (*pCS).dwCompStrLen as UINT;
        let cb: DWORD = ((*pCS).dwCompStrLen as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
        RtlCopyMemory!(
            pbDet.offset(ib as isize) as *mut core::ffi::c_void,
            pbCS.offset((*pCS).dwCompStrOffset as isize) as *const core::ffi::c_void,
            cb as usize,
        );
        *(pbDet.offset(ib as isize) as PWCHAR).offset((*pCS).dwCompStrLen as isize) =
            UNICODE_NULL;

        ib = ib.wrapping_add(ALIGN_WSTR_SIZE!(((*pCS).dwCompStrLen as usize)) as DWORD);
    }

    if (dwGCS & GCS_COMPATTR) != 0 || (*pCS).dwCompAttrLen != 0 {
        (*pDet).uUndetAttrPos = ib as UINT;
        RtlCopyMemory!(
            pbDet.offset(ib as isize) as *mut core::ffi::c_void,
            pbCS.offset((*pCS).dwCompAttrOffset as isize) as *const core::ffi::c_void,
            (*pCS).dwCompAttrLen as usize,
        );

        ib = ib.wrapping_add(ALIGN_DWORD!((*pCS).dwCompAttrLen));
    }

    if (dwGCS & GCS_CURSORPOS) != 0 {
        (*pDet).uCursorPos = (*pCS).dwCursorPos as UINT;
    }

    if (dwGCS & GCS_DELTASTART) != 0 {
        (*pDet).uDeltaStart = (*pCS).dwDeltaStart as UINT;
    }

    if (dwGCS & GCS_RESULTSTR) != 0 {
        (*pDet).uDetermineTextPos = ib as UINT;
        (*pDet).uDetermineTextLen = (*pCS).dwResultStrLen as UINT;
        let cb: DWORD = ((*pCS).dwResultStrLen as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
        RtlCopyMemory!(
            pbDet.offset(ib as isize) as *mut core::ffi::c_void,
            pbCS.offset((*pCS).dwResultStrOffset as isize) as *const core::ffi::c_void,
            cb as usize,
        );
        *(pbDet.offset(ib as isize) as PWCHAR).offset((*pCS).dwResultStrLen as isize) =
            UNICODE_NULL;

        ib = ib.wrapping_add(ALIGN_WSTR_SIZE!(((*pCS).dwResultStrLen as usize)) as DWORD);
    }

    if (dwGCS & GCS_RESULTCLAUSE) != 0 && (*pCS).dwResultClauseLen != 0 {
        (*pDet).uDetermineDelimPos = ib as UINT;
        RtlCopyMemory!(
            pbDet.offset(ib as isize) as *mut core::ffi::c_void,
            pbCS.offset((*pCS).dwResultClauseOffset as isize) as *const core::ffi::c_void,
            (*pCS).dwResultClauseLen as usize,
        );

        ib = ib.wrapping_add(ALIGN_DWORD!((*pCS).dwResultClauseLen));
    }

    if (dwGCS & GCS_RESULTREADSTR) != 0 {
        (*pDet).uYomiTextPos = ib as UINT;
        (*pDet).uYomiTextLen = (*pCS).dwResultReadStrLen as UINT;
        let cb: DWORD = ((*pCS).dwResultReadStrLen as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
        RtlCopyMemory!(
            pbDet.offset(ib as isize) as *mut core::ffi::c_void,
            pbCS.offset((*pCS).dwResultReadStrOffset as isize) as *const core::ffi::c_void,
            cb as usize,
        );
        *(pbDet.offset(ib as isize) as PWCHAR).offset((*pCS).dwResultReadStrLen as isize) =
            UNICODE_NULL;

        ib = ib.wrapping_add(ALIGN_WSTR_SIZE!(((*pCS).dwResultReadStrLen as usize)) as DWORD);
    }

    if (dwGCS & GCS_RESULTREADCLAUSE) != 0 && (*pCS).dwResultReadClauseLen != 0 {
        (*pDet).uYomiDelimPos = ib as UINT;
        RtlCopyMemory!(
            pbDet.offset(ib as isize) as *mut core::ffi::c_void,
            pbCS.offset((*pCS).dwResultReadClauseOffset as isize) as *const core::ffi::c_void,
            (*pCS).dwResultReadClauseLen as usize,
        );

        //ib += ALIGN_DWORD(pCS->dwResultReadClauseLen); // The last one (ineffective)
    }

    return cbRequired;
}

unsafe extern "C" fn Imm32CompStrWToUndetA(
    mut dwGCS: DWORD,
    pCS: *const COMPOSITIONSTRING,
    pDet: PUNDETERMINESTRUCT,
    dwSize: DWORD,
) -> DWORD {
    let cbRequired: DWORD = (core::mem::size_of::<UNDETERMINESTRUCT>() as DWORD)
        .wrapping_add(ALIGN_ASTR_SIZE!((*pCS).dwCompStrLen as usize) as DWORD)
        .wrapping_add(ALIGN_ASTR_SIZE!((*pCS).dwCompAttrLen as usize) as DWORD)
        .wrapping_add(ALIGN_ASTR_SIZE!((*pCS).dwResultStrLen as usize) as DWORD)
        .wrapping_add(ALIGN_DWORD!((*pCS).dwResultClauseLen))
        .wrapping_add(ALIGN_ASTR_SIZE!((*pCS).dwResultReadStrLen as usize) as DWORD)
        .wrapping_add(ALIGN_DWORD!((*pCS).dwResultReadClauseLen));

    if pDet.is_null() {
        return cbRequired;
    }

    if dwSize < cbRequired {
        return 0 as DWORD;
    }

    (*pDet).dwSize = cbRequired;

    let mut ib: DWORD = core::mem::size_of::<UNDETERMINESTRUCT>() as DWORD;
    let pbDet: PBYTE = pDet as PBYTE;
    let pbCS: *const BYTE = pCS as *const BYTE;
    let pCompStrW: PCWSTR = pbCS.offset((*pCS).dwCompStrOffset as isize) as PCWSTR;
    let pResultStr: PCWSTR = pbCS.offset((*pCS).dwResultStrOffset as isize) as PCWSTR;
    let pResultReadStr: PCWSTR = pbCS.offset((*pCS).dwResultReadStrOffset as isize) as PCWSTR;

    if (dwGCS & (GCS_COMPSTR as DWORD)) != 0 {
        let pchA: PSTR = pbDet.offset(ib as isize) as PSTR;
        let cchAnsi: INT = Imm32WideToAnsi(
            pCompStrW as PCWCH,
            (*pCS).dwCompStrLen as INT,
            pchA,
            cbRequired.wrapping_sub(ib) as INT,
        );
        (*pDet).uUndetTextPos = ib as UINT;
        (*pDet).uUndetTextLen = cchAnsi as UINT;
        *pbDet.offset(ib as isize).offset(cchAnsi as isize) = ANSI_NULL as BYTE;

        ib = ib.wrapping_add(
            ALIGN_DWORD!(cchAnsi + 1 as INT) as DWORD,
        );

        if (dwGCS & (GCS_COMPATTR as DWORD)) != 0 || (*pCS).dwCompAttrLen != 0 {
            (*pDet).uUndetAttrPos = ib as UINT;

            let pSrcAttr: *const BYTE = pbCS.offset((*pCS).dwCompAttrOffset as isize);
            let pDestAttr: PBYTE = pbDet.offset(ib as isize);

            let mut ibIndex: UINT = 0 as UINT;
            let mut i: UINT = 0 as UINT;
            while (i as DWORD) < (*pCS).dwCompAttrLen {
                if IsDBCSLeadByte(*pchA.offset(ibIndex as isize) as BYTE) != 0 {
                    let fresh37 = ibIndex;
                    ibIndex = ibIndex.wrapping_add(1);
                    *pDestAttr.offset(fresh37 as isize) = *pSrcAttr.offset(i as isize);
                    let fresh38 = ibIndex;
                    ibIndex = ibIndex.wrapping_add(1);
                    *pDestAttr.offset(fresh38 as isize) = *pSrcAttr.offset(i as isize);
                } else {
                    let fresh39 = ibIndex;
                    ibIndex = ibIndex.wrapping_add(1);
                    *pDestAttr.offset(fresh39 as isize) = *pSrcAttr.offset(i as isize);
                }
                i = i.wrapping_add(1);
            }

            ib = ib.wrapping_add(ALIGN_DWORD!(ibIndex as DWORD) as DWORD);
        }
    }

    if (dwGCS & (GCS_CURSORPOS as DWORD)) != 0 {
        if (*pCS).dwCursorPos == (1 as DWORD).wrapping_neg() {
            (*pDet).uCursorPos = (1 as UINT).wrapping_neg();
        } else {
            (*pDet).uCursorPos = IchAnsiFromWide(
                (*pCS).dwCursorPos as LONG,
                pCompStrW as LPCWSTR,
                CP_ACP as UINT,
            ) as UINT;
        }
    }

    if (dwGCS & (GCS_DELTASTART as DWORD)) != 0 {
        if (*pCS).dwDeltaStart == (1 as DWORD).wrapping_neg() {
            (*pDet).uDeltaStart = (1 as UINT).wrapping_neg();
        } else {
            (*pDet).uDeltaStart = IchAnsiFromWide(
                (*pCS).dwDeltaStart as LONG,
                pCompStrW as LPCWSTR,
                CP_ACP as UINT,
            ) as UINT;
        }
    }

    if (dwGCS & (GCS_RESULTSTR as DWORD)) != 0 {
        let cchAnsi: INT = Imm32WideToAnsi(
            pResultStr as PCWCH,
            (*pCS).dwResultStrLen as INT,
            pbDet.offset(ib as isize) as PSTR,
            cbRequired.wrapping_sub(ib) as INT,
        );
        (*pDet).uDetermineTextPos = ib as UINT;
        (*pDet).uDetermineTextLen = cchAnsi as UINT;

        ib = ib.wrapping_add(ALIGN_DWORD!(
            cchAnsi + core::mem::size_of_val(&ANSI_NULL) as INT,
        ) as DWORD);

        if (dwGCS & (GCS_RESULTCLAUSE as DWORD)) != 0 && (*pCS).dwResultClauseLen != 0 {
            (*pDet).uDetermineDelimPos = ib as UINT;
            let pSrcClause: *const DWORD =
                pbCS.offset((*pCS).dwResultClauseOffset as isize) as *const DWORD;
            let pDestClause: PDWORD = pbDet.offset(ib as isize) as PDWORD;
            let nClauses: DWORD = (*pCS).dwResultClauseLen / core::mem::size_of::<DWORD>() as DWORD;

            let mut i: DWORD = 0 as DWORD;
            while i < nClauses {
                *pDestClause.offset(i as isize) = IchAnsiFromWide(
                    *pSrcClause.offset(i as isize) as LONG,
                    pResultStr as LPCWSTR,
                    CP_ACP as UINT,
                ) as DWORD;
                i = i.wrapping_add(1);
            }

            ib = ib.wrapping_add(ALIGN_DWORD!((*pCS).dwResultClauseLen) as DWORD);
        }
    }

    if (dwGCS & (GCS_RESULTREADSTR as DWORD)) != 0 {
        let cchAnsi: INT = Imm32WideToAnsi(
            pResultReadStr as PCWCH,
            (*pCS).dwResultReadStrLen as INT,
            pbDet.offset(ib as isize) as PSTR,
            cbRequired.wrapping_sub(ib) as INT,
        );
        (*pDet).uYomiTextPos = ib as UINT;
        (*pDet).uYomiTextLen = cchAnsi as UINT;

        ib = ib.wrapping_add(ALIGN_DWORD!(
            cchAnsi + core::mem::size_of_val(&ANSI_NULL) as INT,
        ) as DWORD);

        if (dwGCS & (GCS_RESULTREADCLAUSE as DWORD)) != 0 && (*pCS).dwResultReadClauseLen != 0 {
            (*pDet).uYomiDelimPos = ib as UINT;
            let pSrcClause: *const DWORD =
                pbCS.offset((*pCS).dwResultReadClauseOffset as isize) as *const DWORD;
            let pDestClause: PDWORD = pbDet.offset(ib as isize) as PDWORD;
            let nClauses: DWORD = (*pCS).dwResultReadClauseLen / core::mem::size_of::<DWORD>() as DWORD;
            let pReadW: PCWSTR = pbCS.offset((*pCS).dwResultReadStrOffset as isize) as PCWSTR;

            let mut i: DWORD = 0 as DWORD;
            while i < nClauses {
                *pDestClause.offset(i as isize) = IchAnsiFromWide(
                    *pSrcClause.offset(i as isize) as LONG,
                    pReadW as LPCWSTR,
                    CP_ACP as UINT,
                ) as DWORD;
                i = i.wrapping_add(1);
            }

            //ib += ALIGN_DWORD(pCS->dwResultReadClauseLen); // The last one (ineffective)
        }
    }

    return cbRequired;
}

unsafe extern "C" fn Imm32CompStrWToStringExW(
    mut dwGCS: DWORD,
    mut pCS: *const COMPOSITIONSTRING,
    mut pSX: LPSTRINGEXSTRUCT,
    mut dwSize: DWORD,
) -> DWORD {
    let cbRequired: DWORD = (core::mem::size_of::<STRINGEXSTRUCT>() as DWORD)
        .wrapping_add(ALIGN_WSTR_SIZE!((*pCS).dwResultStrLen))
        .wrapping_add(ALIGN_DWORD!((*pCS).dwResultClauseLen))
        .wrapping_add(ALIGN_WSTR_SIZE!((*pCS).dwResultReadStrLen))
        .wrapping_add(ALIGN_DWORD!((*pCS).dwResultReadClauseLen));

    if pSX.is_null() {
        return cbRequired;
    }

    if dwSize < cbRequired {
        return 0 as DWORD;
    }

    (*pSX).dwSize = dwSize;

    let mut ib: DWORD = core::mem::size_of::<STRINGEXSTRUCT>() as DWORD;
    let mut pBase: PBYTE = pSX as PBYTE;
    let mut pbCS: *const BYTE = pCS as *const BYTE;

    {
        (*pSX).uDeterminePos = ib as UINT;
        let mut cbResultStr: DWORD = (*pCS).dwResultStrLen.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        RtlCopyMemory!(
            pBase.add(ib as usize) as *mut core::ffi::c_void,
            pbCS.add((*pCS).dwResultStrOffset as usize) as *const core::ffi::c_void,
            cbResultStr as usize,
        );
        *(pBase.add(ib as usize) as PWSTR).add((*pCS).dwResultStrLen as usize) = UNICODE_NULL as WCHAR;

        ib = ib.wrapping_add(ALIGN_WSTR_SIZE!((*pCS).dwResultStrLen));
    }

    if (dwGCS & (GCS_RESULTCLAUSE as DWORD)) != 0 && (*pCS).dwResultClauseLen != 0 {
        (*pSX).uDetermineDelimPos = ib as UINT;
        RtlCopyMemory!(
            pBase.add(ib as usize) as *mut core::ffi::c_void,
            pbCS.add((*pCS).dwResultClauseOffset as usize) as *const core::ffi::c_void,
            (*pCS).dwResultClauseLen as usize,
        );

        ib = ib.wrapping_add(ALIGN_DWORD!((*pCS).dwResultClauseLen));
    }

    {
        (*pSX).uYomiPos = ib as UINT;
        let mut cbResultReadStr: DWORD = (*pCS).dwResultReadStrLen.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD);
        RtlCopyMemory!(
            pBase.add(ib as usize) as *mut core::ffi::c_void,
            pbCS.add((*pCS).dwResultReadStrOffset as usize) as *const core::ffi::c_void,
            cbResultReadStr as usize,
        );
        *(pBase.add(ib as usize) as PWSTR).add((*pCS).dwResultReadStrLen as usize) = UNICODE_NULL as WCHAR;

        ib = ib.wrapping_add(ALIGN_WSTR_SIZE!((*pCS).dwResultReadStrLen));
    }

    if (dwGCS & (GCS_RESULTREADCLAUSE as DWORD)) != 0 && (*pCS).dwResultReadClauseLen != 0 {
        (*pSX).uYomiDelimPos = ib as UINT;
        RtlCopyMemory!(
            pBase.add(ib as usize) as *mut core::ffi::c_void,
            pbCS.add((*pCS).dwResultReadClauseOffset as usize) as *const core::ffi::c_void,
            (*pCS).dwResultReadClauseLen as usize,
        );

        //ib += ALIGN_DWORD(pCS->dwResultReadClauseLen); // The last one (ineffective)
    }

    return dwSize;
}

unsafe extern "C" fn Imm32CompStrWToStringExA(
    mut dwGCS: DWORD,
    mut pCS: *const COMPOSITIONSTRING,
    mut pSX: LPSTRINGEXSTRUCT,
    mut dwSize: DWORD,
) -> DWORD {
    let cbRequired: DWORD = (core::mem::size_of::<STRINGEXSTRUCT>() as DWORD)
        .wrapping_add(ALIGN_ASTR_SIZE!((*pCS).dwResultStrLen as usize) as DWORD)
        .wrapping_add(ALIGN_DWORD!((*pCS).dwResultClauseLen))
        .wrapping_add(ALIGN_ASTR_SIZE!((*pCS).dwResultReadStrLen as usize) as DWORD)
        .wrapping_add(ALIGN_DWORD!((*pCS).dwResultReadClauseLen));

    if pSX.is_null() {
        return cbRequired;
    }

    if dwSize < cbRequired {
        return 0;
    }

    (*pSX).dwSize = dwSize;

    let mut ib: DWORD = core::mem::size_of::<STRINGEXSTRUCT>() as DWORD;
    let mut pbSX: PBYTE = pSX as PBYTE;
    let mut pbCS: *const BYTE = pCS as *const BYTE;
    let mut pResultStr: PCWSTR = pbCS.offset((*pCS).dwResultStrOffset as isize) as PCWSTR;
    let mut pResultReadStr: PCWSTR = pbCS.offset((*pCS).dwResultReadStrOffset as isize) as PCWSTR;

    if (*pCS).dwResultStrLen != 0 {
        (*pSX).uDeterminePos = ib as UINT;
        let mut nAnsiResultLen: INT = Imm32WideToAnsi(
            pResultStr as PCWCH,
            (*pCS).dwResultStrLen as INT,
            pbSX.offset(ib as isize) as PSTR,
            dwSize.wrapping_sub(ib) as INT,
        );
        if nAnsiResultLen <= 0 {
            return 0;
        }

        ib = ib.wrapping_add(ALIGN_DWORD!(
            (nAnsiResultLen + 1) as DWORD,
        ));

        if (dwGCS & GCS_RESULTCLAUSE as DWORD) != 0 && (*pCS).dwResultClauseLen != 0 {
            (*pSX).uDetermineDelimPos = ib as UINT;
            let mut pSrcClause: *const DWORD =
                pbCS.offset((*pCS).dwResultClauseOffset as isize) as *const DWORD;
            let mut pDestClause: PDWORD = pbSX.offset(ib as isize) as PDWORD;
            let mut nClauses: DWORD = ((*pCS).dwResultClauseLen as usize)
                .wrapping_div(core::mem::size_of::<DWORD>() as usize) as DWORD;
            let mut pResultW: PCWSTR = pbCS.offset((*pCS).dwResultStrOffset as isize) as PCWSTR;
            let mut i: DWORD = 0;
            while i < nClauses {
                *pDestClause.offset(i as isize) = IchAnsiFromWide(
                    *pSrcClause.offset(i as isize) as LONG,
                    pResultW as LPCWSTR,
                    CP_ACP as UINT,
                ) as DWORD;
                i = i.wrapping_add(1);
            }

            ib = ib.wrapping_add(ALIGN_DWORD!((*pCS).dwResultClauseLen));
        }
    }

    if (*pCS).dwResultReadStrLen != 0 {
        (*pSX).uYomiPos = ib as UINT;
        let mut nAnsiReadLen: INT = Imm32WideToAnsi(
            pResultReadStr as PCWCH,
            (*pCS).dwResultReadStrLen as INT,
            pbSX.offset(ib as isize) as PSTR,
            dwSize.wrapping_sub(ib) as INT,
        );
        if nAnsiReadLen <= 0 {
            return 0;
        }

        ib = ib.wrapping_add(ALIGN_DWORD!(
            (nAnsiReadLen + 1) as DWORD,
        ));

        if (dwGCS & GCS_RESULTREADCLAUSE as DWORD) != 0 && (*pCS).dwResultReadClauseLen != 0 {
            (*pSX).uYomiDelimPos = ib as UINT;
            let mut pSrcClause_0: *const DWORD =
                pbCS.offset((*pCS).dwResultReadClauseOffset as isize) as *const DWORD;
            let mut pDestClause_0: PDWORD = pbSX.offset(ib as isize) as PDWORD;
            let mut nClauses_0: DWORD = ((*pCS).dwResultReadClauseLen as usize)
                .wrapping_div(core::mem::size_of::<DWORD>() as usize) as DWORD;
            let mut pReadW: PCWSTR = pbCS.offset((*pCS).dwResultReadStrOffset as isize) as PCWSTR;
            let mut i_0: DWORD = 0;
            while i_0 < nClauses_0 {
                *pDestClause_0.offset(i_0 as isize) = IchAnsiFromWide(
                    *pSrcClause_0.offset(i_0 as isize) as LONG,
                    pReadW as LPCWSTR,
                    CP_ACP as UINT,
                ) as DWORD;
                i_0 = i_0.wrapping_add(1);
            }

            //ib += ALIGN_DWORD(pCS->dwResultReadClauseLen); // The last one (ineffective)
        }
    }

    return dwSize;
}

unsafe extern "C" fn Imm32CompStrAToUndetA(
    mut dwGCS: DWORD,
    pCS: *const COMPOSITIONSTRING,
    pDet: PUNDETERMINESTRUCT,
    dwSize: DWORD,
) -> DWORD {
    let cbRequired: DWORD = (core::mem::size_of::<UNDETERMINESTRUCT>() as DWORD)
        .wrapping_add(ALIGN_DWORD!(
            (*pCS).dwCompStrLen.wrapping_add(core::mem::size_of_val(&ANSI_NULL) as DWORD),
        ))
        .wrapping_add(ALIGN_DWORD!((*pCS).dwCompAttrLen))
        .wrapping_add(ALIGN_DWORD!(
            (*pCS).dwResultStrLen.wrapping_add(core::mem::size_of_val(&ANSI_NULL) as DWORD),
        ))
        .wrapping_add(ALIGN_DWORD!((*pCS).dwResultClauseLen))
        .wrapping_add(ALIGN_DWORD!(
            (*pCS).dwResultReadStrLen.wrapping_add(core::mem::size_of_val(&ANSI_NULL) as DWORD),
        ))
        .wrapping_add(ALIGN_DWORD!((*pCS).dwResultReadClauseLen));

    if pDet.is_null() {
        return cbRequired;
    }

    if dwSize < cbRequired {
        return 0 as DWORD;
    }

    (*pDet).dwSize = dwSize;

    let mut ib: DWORD = core::mem::size_of::<UNDETERMINESTRUCT>() as DWORD;
    let pbDet: PBYTE = pDet as PBYTE;
    let pbCS: PBYTE = pCS as PBYTE;

    if (dwGCS & (GCS_COMPSTR as DWORD)) != 0 {
        (*pDet).uUndetTextPos = ib as UINT;
        (*pDet).uUndetTextLen = (*pCS).dwCompStrLen as UINT;
        RtlCopyMemory!(
            pbDet.add(ib as usize) as *mut core::ffi::c_void,
            pbCS.add((*pCS).dwCompStrOffset as usize) as *const core::ffi::c_void,
            (*pCS).dwCompStrLen as usize,
        );
        *pbDet.add(ib.wrapping_add((*pCS).dwCompStrLen) as usize) = ANSI_NULL as BYTE;

        ib = ib.wrapping_add(ALIGN_DWORD!(
            (*pCS).dwCompStrLen.wrapping_add(1 as DWORD),
        ));

        if (*pCS).dwCompAttrLen != 0 {
            dwGCS |= GCS_COMPATTR as DWORD;
        }
    }

    if (dwGCS & (GCS_COMPATTR as DWORD)) != 0 {
        (*pDet).uUndetAttrPos = ib as UINT;
        RtlCopyMemory!(
            pbDet.add(ib as usize) as *mut core::ffi::c_void,
            pbCS.add((*pCS).dwCompAttrOffset as usize) as *const core::ffi::c_void,
            (*pCS).dwCompAttrLen as usize,
        );

        ib = ib.wrapping_add(ALIGN_DWORD!((*pCS).dwCompAttrLen));
    }

    if (dwGCS & (GCS_CURSORPOS as DWORD)) != 0 {
        (*pDet).uCursorPos = (*pCS).dwCursorPos as UINT;
    }

    if (dwGCS & (GCS_DELTASTART as DWORD)) != 0 {
        (*pDet).uDeltaStart = (*pCS).dwDeltaStart as UINT;
    }

    if (dwGCS & (GCS_RESULTSTR as DWORD)) != 0 {
        (*pDet).uDetermineTextPos = ib as UINT;
        (*pDet).uDetermineTextLen = (*pCS).dwResultStrLen as UINT;
        RtlCopyMemory!(
            pbDet.add(ib as usize) as *mut core::ffi::c_void,
            pbCS.add((*pCS).dwResultStrOffset as usize) as *const core::ffi::c_void,
            (*pCS).dwResultStrLen as usize,
        );
        *pbDet.add(ib.wrapping_add((*pCS).dwResultStrLen) as usize) = ANSI_NULL as BYTE;

        ib = ib.wrapping_add(ALIGN_DWORD!(
            (*pCS).dwResultStrLen.wrapping_add(1 as DWORD),
        ));
    }

    if ((dwGCS & (GCS_RESULTCLAUSE as DWORD)) != 0) && (*pCS).dwResultClauseLen != 0 {
        (*pDet).uDetermineDelimPos = ib as UINT;
        RtlCopyMemory!(
            pbDet.add(ib as usize) as *mut core::ffi::c_void,
            pbCS.add((*pCS).dwResultClauseOffset as usize) as *const core::ffi::c_void,
            (*pCS).dwResultClauseLen as usize,
        );

        ib = ib.wrapping_add(ALIGN_DWORD!((*pCS).dwResultClauseLen));
    }

    if (dwGCS & (GCS_RESULTREADSTR as DWORD)) != 0 {
        (*pDet).uYomiTextPos = ib as UINT;
        (*pDet).uYomiTextLen = (*pCS).dwResultReadStrLen as UINT;
        RtlCopyMemory!(
            pbDet.add(ib as usize) as *mut core::ffi::c_void,
            pbCS.add((*pCS).dwResultReadStrOffset as usize) as *const core::ffi::c_void,
            (*pCS).dwResultReadStrLen as usize,
        );
        *pbDet.add(ib.wrapping_add((*pCS).dwResultReadStrLen) as usize) = ANSI_NULL as BYTE;

        ib = ib.wrapping_add(ALIGN_DWORD!(
            (*pCS).dwResultReadStrLen.wrapping_add(1 as DWORD),
        ));
    }

    if ((dwGCS & (GCS_RESULTREADCLAUSE as DWORD)) != 0) && (*pCS).dwResultReadClauseLen != 0 {
        (*pDet).uYomiDelimPos = ib as UINT;
        RtlCopyMemory!(
            pbDet.add(ib as usize) as *mut core::ffi::c_void,
            pbCS.add((*pCS).dwResultReadClauseOffset as usize) as *const core::ffi::c_void,
            (*pCS).dwResultReadClauseLen as usize,
        );

        //ib += ALIGN_DWORD(pCS->dwResultReadClauseLen); // The last one (ineffective)
    }

    return dwSize;
}

unsafe extern "C" fn Imm32CompStrAToUndetW(
    mut dwGCS: DWORD,
    mut pCS: *const COMPOSITIONSTRING,
    mut pUndet: PUNDETERMINESTRUCT,
    mut dwSize: DWORD,
) -> DWORD {
    let cbRequired: DWORD = (core::mem::size_of::<UNDETERMINESTRUCT>() as DWORD)
        .wrapping_add(ALIGN_DWORD!((*pCS).dwResultClauseLen))
        .wrapping_add(ALIGN_DWORD!((*pCS).dwCompAttrLen))
        .wrapping_add(ALIGN_WSTR_SIZE!((*pCS).dwResultStrLen))
        .wrapping_add(ALIGN_WSTR_SIZE!((*pCS).dwCompStrLen))
        .wrapping_add(ALIGN_WSTR_SIZE!((*pCS).dwResultReadStrLen))
        .wrapping_add(ALIGN_DWORD!((*pCS).dwResultReadClauseLen));

    if pUndet.is_null() {
        return cbRequired;
    }

    if dwSize < cbRequired {
        return 0 as DWORD;
    }

    (*pUndet).dwSize = dwSize;

    let mut ib: DWORD = core::mem::size_of::<UNDETERMINESTRUCT>() as DWORD;
    let mut pbUndet: PBYTE = pUndet as PBYTE;
    let mut pbCS: *const BYTE = pCS as *const BYTE;
    let mut pCompStrA: PCSTR = pbCS.offset((*pCS).dwCompStrOffset as isize) as PCSTR;
    let mut pResultStr: PCSTR = pbCS.offset((*pCS).dwResultStrOffset as isize) as PCSTR;
    let mut pResultReadStr: PCSTR = pbCS.offset((*pCS).dwResultReadStrOffset as isize) as PCSTR;

    if dwGCS & (GCS_COMPSTR as DWORD) != 0 {
        let mut nWideLen: INT = Imm32AnsiToWide(
            pCompStrA as PCCH,
            (*pCS).dwCompStrLen as INT,
            pbUndet.offset(ib as isize) as PWSTR,
            (dwSize.wrapping_sub(ib) as usize)
                .wrapping_div(core::mem::size_of::<WCHAR>() as usize) as INT,
        );
        if nWideLen <= 0 as INT {
            return 0 as DWORD;
        }

        (*pUndet).uUndetTextPos = ib as UINT;
        (*pUndet).uUndetTextLen = nWideLen as UINT;

        ib = ib.wrapping_add(ALIGN_DWORD!(
            ((nWideLen + 1 as INT) as usize
                * core::mem::size_of::<WCHAR>() as usize) as DWORD,
        ));

        if (dwGCS & (GCS_COMPATTR as DWORD) != 0) || (*pCS).dwCompAttrLen != 0 {
            (*pUndet).uUndetAttrPos = ib as UINT;
            let mut pSrcAttr: *const BYTE = pbCS.offset((*pCS).dwCompAttrOffset as isize);
            let mut pDestAttr: PBYTE = pbUndet.offset(ib as isize);
            let mut pWStr: PCWSTR = pbUndet.offset((*pUndet).uUndetTextPos as isize) as PCWSTR;

            let mut i: UINT = 0 as UINT;
            while i < (*pUndet).uUndetTextLen {
                *pDestAttr.offset(i as isize) = *pSrcAttr;
                let mut wChar: USHORT = *pWStr.offset(i as isize) as USHORT;
                let mut cbMultiByte: ULONG = 0 as ULONG;
                RtlUnicodeToMultiByteSize(
                    core::ptr::addr_of_mut!(cbMultiByte),
                    core::ptr::addr_of_mut!(wChar) as PCWCH,
                    core::mem::size_of::<WCHAR>() as ULONG,
                );
                pSrcAttr = pSrcAttr.offset(
                    (if cbMultiByte == 2 as ULONG {
                        2 as isize
                    } else {
                        1 as isize
                    }),
                );
                i = i.wrapping_add(1);
            }

            ib = ib.wrapping_add(ALIGN_DWORD!((*pUndet).uUndetTextLen as DWORD));
        }
    }

    if dwGCS & (GCS_CURSORPOS as DWORD) != 0 {
        if (*pCS).dwCursorPos == (-(1 as ::core::ffi::c_int)) as DWORD {
            (*pUndet).uCursorPos = (-(1 as ::core::ffi::c_int)) as UINT;
        } else {
            (*pUndet).uCursorPos = IchWideFromAnsi(
                (*pCS).dwCursorPos as LONG,
                pCompStrA as LPCSTR,
                CP_ACP as UINT,
            ) as UINT;
        }
    }

    if dwGCS & (GCS_DELTASTART as DWORD) != 0 {
        if (*pCS).dwDeltaStart == (-(1 as ::core::ffi::c_int)) as DWORD {
            (*pUndet).uDeltaStart = (-(1 as ::core::ffi::c_int)) as UINT;
        } else {
            (*pUndet).uDeltaStart = IchWideFromAnsi(
                (*pCS).dwDeltaStart as LONG,
                pCompStrA as LPCSTR,
                CP_ACP as UINT,
            ) as UINT;
        }
    }

    if dwGCS & (GCS_RESULTSTR as DWORD) != 0 {
        let mut nWideLen_0: INT = Imm32AnsiToWide(
            pResultStr as PCCH,
            (*pCS).dwResultStrLen as INT,
            pbUndet.offset(ib as isize) as PWSTR,
            (dwSize.wrapping_sub(ib) as usize)
                .wrapping_div(core::mem::size_of::<WCHAR>() as usize) as INT,
        );
        if nWideLen_0 > 0 as INT {
            (*pUndet).uDetermineTextPos = ib as UINT;
            (*pUndet).uDetermineTextLen = nWideLen_0 as UINT;

            ib = ib.wrapping_add(ALIGN_DWORD!(
                ((nWideLen_0 + 1 as INT) as usize
                    * core::mem::size_of::<WCHAR>() as usize) as DWORD,
            ));

            if (dwGCS & (GCS_RESULTCLAUSE as DWORD) != 0) && (*pCS).dwResultClauseLen != 0 {
                (*pUndet).uDetermineDelimPos = ib as UINT;
                let mut pSrcClause: PDWORD =
                    pbCS.offset((*pCS).dwResultClauseOffset as isize) as PDWORD;
                let mut pDestClause: PDWORD = pbUndet.offset(ib as isize) as PDWORD;
                let mut nClauses: DWORD =
                    ((*pCS).dwResultClauseLen as usize / core::mem::size_of::<DWORD>() as usize)
                        as DWORD;

                let mut i_0: DWORD = 0 as DWORD;
                while i_0 < nClauses {
                    *pDestClause.offset(i_0 as isize) = IchWideFromAnsi(
                        *pSrcClause.offset(i_0 as isize) as LONG,
                        pResultStr as LPCSTR,
                        CP_ACP as UINT,
                    ) as DWORD;
                    i_0 = i_0.wrapping_add(1);
                }

                ib = ib.wrapping_add(ALIGN_DWORD!((*pCS).dwResultClauseLen));
            }
        }
    }

    if dwGCS & (GCS_RESULTREADSTR as DWORD) != 0 {
        let mut nWideLen_1: INT = Imm32AnsiToWide(
            pResultReadStr as PCCH,
            (*pCS).dwResultReadStrLen as INT,
            pbUndet.offset(ib as isize) as PWSTR,
            (dwSize.wrapping_sub(ib) as usize)
                .wrapping_div(core::mem::size_of::<WCHAR>() as usize) as INT,
        );
        if nWideLen_1 > 0 as INT {
            (*pUndet).uYomiTextPos = ib as UINT;
            (*pUndet).uYomiTextLen = nWideLen_1 as UINT;

            ib = ib.wrapping_add(ALIGN_DWORD!(
                ((nWideLen_1 + 1 as INT) as usize
                    * core::mem::size_of::<WCHAR>() as usize) as DWORD,
            ));

            if (dwGCS & (GCS_RESULTREADCLAUSE as DWORD) != 0) && (*pCS).dwResultReadClauseLen != 0 {
                (*pUndet).uYomiDelimPos = ib as UINT;
                let mut pSrcClause_0: PDWORD =
                    pbCS.offset((*pCS).dwResultReadClauseOffset as isize) as PDWORD;
                let mut pDestClause_0: PDWORD = pbUndet.offset(ib as isize) as PDWORD;
                let mut nClauses_0: DWORD = ((*pCS).dwResultReadClauseLen as usize
                    / core::mem::size_of::<DWORD>() as usize)
                    as DWORD;

                let mut i_1: DWORD = 0 as DWORD;
                while i_1 < nClauses_0 {
                    *pDestClause_0.offset(i_1 as isize) = IchWideFromAnsi(
                        *pSrcClause_0.offset(i_1 as isize) as LONG,
                        pResultReadStr as LPCSTR,
                        CP_ACP as UINT,
                    ) as DWORD;
                    i_1 = i_1.wrapping_add(1);
                }

                //ib += ALIGN_DWORD(pCS->dwResultReadClauseLen); // The last one (ineffective)
            }
        }
    }

    return dwSize;
}

unsafe extern "C" fn Imm32CompStrAToStringExA(
    dwGCS: DWORD,
    pCS: *const COMPOSITIONSTRING,
    pSX: LPSTRINGEXSTRUCT,
    dwSize: DWORD,
) -> DWORD {
    let mut cbRequired: DWORD =
        core::mem::size_of::<STRINGEXSTRUCT>() as DWORD
        + ALIGN_DWORD!((*pCS).dwResultStrLen + core::mem::size_of_val(&ANSI_NULL) as DWORD)
        + ALIGN_DWORD!((*pCS).dwResultClauseLen)
        + ALIGN_DWORD!((*pCS).dwResultReadStrLen + core::mem::size_of_val(&ANSI_NULL) as DWORD)
        + ALIGN_DWORD!((*pCS).dwResultReadClauseLen);

    if pSX.is_null() {
        return cbRequired;
    }

    if dwSize < cbRequired {
        return 0;
    }

    (*pSX).dwSize = dwSize;

    let mut ib: DWORD = core::mem::size_of::<STRINGEXSTRUCT>() as DWORD;
    let pbSX: PBYTE = pSX as PBYTE;
    let pbCS: *const BYTE = pCS as *const BYTE;

    {
        (*pSX).uDeterminePos = ib;
        core::ptr::copy_nonoverlapping(
            pbSX.add(ib as usize) as *const u8,
            pbCS.add((*pCS).dwResultStrOffset as usize) as *const u8 as *mut u8,
            (*pCS).dwResultStrLen as usize,
        );
        *pbSX.add((ib + (*pCS).dwResultStrLen) as usize) = ANSI_NULL as BYTE;

        ib += ALIGN_DWORD!((*pCS).dwResultStrLen + 1 as DWORD);
    }

    if (dwGCS & GCS_RESULTCLAUSE as DWORD) != 0 && (*pCS).dwResultClauseLen != 0 {
        (*pSX).uDetermineDelimPos = ib;
        core::ptr::copy_nonoverlapping(
            pbSX.add(ib as usize) as *const u8,
            pbCS.add((*pCS).dwResultClauseOffset as usize) as *const u8 as *mut u8,
            (*pCS).dwResultClauseLen as usize,
        );

        ib += ALIGN_DWORD!((*pCS).dwResultClauseLen);
    }

    {
        (*pSX).uYomiPos = ib;
        core::ptr::copy_nonoverlapping(
            pbSX.add(ib as usize) as *const u8,
            pbCS.add((*pCS).dwResultReadStrOffset as usize) as *const u8 as *mut u8,
            (*pCS).dwResultReadStrLen as usize,
        );
        *pbSX.add((ib + (*pCS).dwResultReadStrLen) as usize) = ANSI_NULL as BYTE;

        ib += ALIGN_DWORD!((*pCS).dwResultReadStrLen + 1 as DWORD);
    }

    if (dwGCS & GCS_RESULTREADCLAUSE as DWORD) != 0 && (*pCS).dwResultReadClauseLen != 0 {
        (*pSX).uYomiDelimPos = ib;
        core::ptr::copy_nonoverlapping(
            pbSX.add(ib as usize) as *const u8,
            pbCS.add((*pCS).dwResultReadClauseOffset as usize) as *const u8 as *mut u8,
            (*pCS).dwResultReadClauseLen as usize,
        );

        //ib += ALIGN_DWORD(pCS->dwResultReadClauseLen); // The last one (ineffective)
    }

    return dwSize;
}

unsafe extern "C" fn Imm32CompStrAToStringExW(
    dwGCS: DWORD,
    pCS: *const COMPOSITIONSTRING,
    pSX: LPSTRINGEXSTRUCT,
    dwSize: DWORD,
) -> DWORD {
    let cbRequired: DWORD = (core::mem::size_of::<STRINGEXSTRUCT>() as DWORD)
        .wrapping_add(ALIGN_WSTR_SIZE!((*pCS).dwResultStrLen))
        .wrapping_add(ALIGN_DWORD!((*pCS).dwResultClauseLen))
        .wrapping_add(ALIGN_WSTR_SIZE!((*pCS).dwResultReadStrLen))
        .wrapping_add(ALIGN_DWORD!((*pCS).dwResultReadClauseLen));

    if pSX.is_null() {
        return cbRequired;
    }

    if dwSize < cbRequired {
        return 0;
    }

    (*pSX).dwSize = dwSize;

    let mut ib: DWORD = core::mem::size_of::<STRINGEXSTRUCT>() as DWORD;
    let pbSX: PBYTE = pSX as PBYTE;
    let pbCS: *const BYTE = pCS as *const BYTE;
    let pResultStr: PCSTR = pbCS.offset((*pCS).dwResultStrOffset as isize) as PCSTR;
    let pResultReadStr: PCSTR = pbCS.offset((*pCS).dwResultReadStrOffset as isize) as PCSTR;

    if (*pCS).dwResultStrLen != 0 {
        (*pSX).uDeterminePos = ib as UINT;
        let nWideResultLen: INT = Imm32AnsiToWide(
            pResultStr as PCCH,
            (*pCS).dwResultStrLen as INT,
            pbSX.offset(ib as isize) as PWSTR,
            (dwSize.wrapping_sub(ib) / (core::mem::size_of::<WCHAR>() as DWORD)) as INT,
        );
        if nWideResultLen <= 0 {
            return 0;
        }

        ib = ib.wrapping_add(ALIGN_DWORD!(
            ((nWideResultLen + 1) as DWORD)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
        ));

        if (dwGCS & GCS_RESULTCLAUSE) != 0 && (*pCS).dwResultClauseLen != 0 {
            (*pSX).uDetermineDelimPos = ib as UINT;
            let pSrcClause: *const DWORD =
                pbCS.offset((*pCS).dwResultClauseOffset as isize) as *const DWORD;
            let pDestClause: PDWORD = pbSX.offset(ib as isize) as PDWORD;
            let nClauses: DWORD =
                (*pCS).dwResultClauseLen / (core::mem::size_of::<DWORD>() as DWORD);

            let mut i: DWORD = 0;
            while i < nClauses {
                *pDestClause.offset(i as isize) = IchWideFromAnsi(
                    *pSrcClause.offset(i as isize) as LONG,
                    pbCS.offset((*pCS).dwResultStrOffset as isize) as PCSTR,
                    CP_ACP,
                ) as DWORD;
                i = i.wrapping_add(1);
            }

            ib = ib.wrapping_add(ALIGN_DWORD!((*pCS).dwResultClauseLen));
        }
    }

    if (*pCS).dwResultReadStrLen != 0 {
        (*pSX).uYomiPos = ib as UINT;
        let nWideReadLen: INT = Imm32AnsiToWide(
            pResultReadStr as PCCH,
            (*pCS).dwResultReadStrLen as INT,
            pbSX.offset(ib as isize) as PWSTR,
            (dwSize.wrapping_sub(ib) / (core::mem::size_of::<WCHAR>() as DWORD)) as INT,
        );
        if nWideReadLen <= 0 {
            return 0;
        }

        ib = ib.wrapping_add(ALIGN_DWORD!(
            ((nWideReadLen + 1) as DWORD)
                .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD),
        ));

        if (dwGCS & GCS_RESULTREADCLAUSE) != 0 && (*pCS).dwResultReadClauseLen != 0 {
            (*pSX).uYomiDelimPos = ib as UINT;
            let pSrcClause: *const DWORD =
                pbCS.offset((*pCS).dwResultReadClauseOffset as isize) as *const DWORD;
            let pDestClause: PDWORD = pbSX.offset(ib as isize) as PDWORD;
            let nClauses: DWORD =
                (*pCS).dwResultReadClauseLen / (core::mem::size_of::<DWORD>() as DWORD);

            let mut i: DWORD = 0;
            while i < nClauses {
                *pDestClause.offset(i as isize) = IchWideFromAnsi(
                    *pSrcClause.offset(i as isize) as LONG,
                    pResultReadStr as PCSTR,
                    CP_ACP,
                ) as DWORD;
                i = i.wrapping_add(1);
            }

            //ib += ALIGN_DWORD(pCS->dwResultReadClauseLen); // The last one (ineffective)
        }
    }

    return dwSize;
}

unsafe extern "C" fn Imm32CompStrAToStringA(pCS: *const COMPOSITIONSTRING, pszString: PSTR, dwSize: DWORD) -> DWORD {
    let dwResultStrLen: DWORD = (*pCS).dwResultStrLen;
    if pszString.is_null() {
        return dwResultStrLen.wrapping_add(1);
    }
    if dwSize < dwResultStrLen.wrapping_add(1) {
        return 0;
    }
    core::ptr::copy_nonoverlapping(
        (pCS as *const BYTE).add((*pCS).dwResultStrOffset as usize),
        pszString as *mut BYTE,
        dwResultStrLen as usize,
    );
    *pszString.offset(dwResultStrLen as isize) = ANSI_NULL;
    return dwResultStrLen.wrapping_add(1);
}

unsafe extern "C" fn Imm32CompStrAToStringW(pCS: *const COMPOSITIONSTRING, pszString: PWSTR, dwSize: DWORD) -> DWORD {
    let pch: PCSTR = (pCS as PCSTR).offset((*pCS).dwResultStrOffset as isize);
    let cchWide: INT = Imm32AnsiToWide(pch as PCCH, (*pCS).dwResultStrLen as INT, pszString, 0);
    let dwResultStrLen: DWORD = ((cchWide + 1) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
    if pszString.is_null() {
        return dwResultStrLen;
    }
    if dwSize < dwResultStrLen {
        return 0;
    }
    let cch: INT = Imm32AnsiToWide(pch as PCCH, (*pCS).dwResultStrLen as INT, pszString, cchWide);
    if cch < cchWide {
        return 0;
    }
    *pszString.offset(cch as isize) = UNICODE_NULL;
    return ((cch + 1) as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
}

unsafe extern "C" fn Imm32CompStrAToCharA(hWnd: HWND, pCS: *const COMPOSITIONSTRING) {
    let mut pch: PCSTR = (pCS as *const BYTE).offset((*pCS).dwResultStrOffset as isize) as PCSTR;

    let mut bImeSupportDBCS: BOOL = FALSE as BOOL;
    if (*GetWin32ClientInfo!()).dwExpWinVer >= 0x30A as DWORD {
        bImeSupportDBCS = SendMessageA(
            hWnd,
            WM_IME_REPORT as UINT,
            IR_DBCSCHAR as WPARAM,
            0 as LPARAM,
        ) as BOOL;
    }

    PostMessageA(
        hWnd,
        WM_IME_REPORT as UINT,
        IR_STRINGSTART as WPARAM,
        0 as LPARAM,
    );

    if *pch == 0 {
        PostMessageA(
            hWnd,
            WM_IME_REPORT as UINT,
            IR_STRINGEND as WPARAM,
            0 as LPARAM,
        );
        return;
    }

    let mut pNext: PCSTR = core::ptr::null();
    while *pch != 0 {
        let bFirst: BYTE = (*pch) as BYTE;
        pNext = CharNextA(pch) as PCSTR;

        if *pNext == 0 {
            PostMessageA(
                hWnd,
                WM_IME_REPORT as UINT,
                IR_STRINGEND as WPARAM,
                0 as LPARAM,
            );
        }

        if IsDBCSLeadByte(bFirst) == 0 {
            PostMessageA(
                hWnd,
                WM_CHAR as UINT,
                bFirst as WPARAM,
                1 as LPARAM,
            );
            pch = pNext;
            continue;
        }

        let bSecond: BYTE = (*pch.offset(1 as isize)) as BYTE;
        if bImeSupportDBCS != 0 {
            let wParamDBCS: WPARAM = (0x80000000 as WPARAM)
                | (MAKEWORD!(bSecond, bFirst) as WPARAM);
            PostMessageA(
                hWnd,
                WM_CHAR as UINT,
                wParamDBCS,
                1 as LPARAM,
            );
        } else {
            PostMessageA(
                hWnd,
                WM_CHAR as UINT,
                bFirst as WPARAM,
                1 as LPARAM,
            );
            PostMessageA(
                hWnd,
                WM_CHAR as UINT,
                bSecond as WPARAM,
                1 as LPARAM,
            );
        }

        pch = pNext;
    }
}

unsafe extern "C" fn Imm32CompStrAToCharW(mut hWnd: HWND, mut pCS: *const COMPOSITIONSTRING) {
    let mut pCurrent: PCSTR =
        (pCS as *const BYTE).offset((*pCS).dwResultStrOffset as isize) as PCSTR;

    PostMessageW(
        hWnd,
        WM_IME_REPORT as UINT,
        IR_STRINGSTART as WPARAM,
        0 as LPARAM,
    );

    while *pCurrent != 0 {
        let mut bTestChar: BYTE = *pCurrent as BYTE;
        let mut pNext: PCHAR = CharNextA(pCurrent as LPCSTR) as PCHAR;

        if *pNext == 0 {
            PostMessageW(
                hWnd,
                WM_IME_REPORT as UINT,
                IR_STRINGEND as WPARAM,
                0 as LPARAM,
            );
        }

        let mut szWide: [WCHAR; 2] = [0; 2];
        let mut nConverted: INT = 0 as INT;

        if IsDBCSLeadByte(bTestChar) != 0 {
            nConverted = Imm32AnsiToWide(
                pCurrent as PCCH,
                2 as INT,
                core::ptr::addr_of_mut!(szWide) as PWSTR,
                _countof!(szWide) as INT,
            );
        } else {
            nConverted = Imm32AnsiToWide(
                pCurrent as PCCH,
                1 as INT,
                core::ptr::addr_of_mut!(szWide) as PWSTR,
                _countof!(szWide) as INT,
            );
        }

        if nConverted > 0 as INT {
            PostMessageW(
                hWnd,
                WM_CHAR as UINT,
                szWide[0 as usize] as WPARAM,
                1 as LPARAM,
            );
        }

        pCurrent = pNext as PCSTR;
    }
}

unsafe extern "C" fn Imm32CompStrWToCharA(hWnd: HWND, pCS: *const COMPOSITIONSTRING) {
    let mut pCurrent: PCWSTR = (pCS as *const BYTE).add((*pCS).dwResultStrOffset as usize) as PCWSTR;

    let mut bSupportDBCSReport: BOOL = (FALSE) as i32;
    if (*GetWin32ClientInfo!()).dwExpWinVer >= 0x30A {
        bSupportDBCSReport = SendMessageA(hWnd, WM_IME_REPORT, (IR_DBCSCHAR) as WPARAM, 0) as BOOL;
    }

    PostMessageA(hWnd, WM_IME_REPORT, (IR_STRINGSTART) as WPARAM, 0);

    let mut pNext: PCWSTR = core::ptr::null();
    while *pCurrent != 0 {
        'body: {
            pNext = CharNextW(pCurrent) as PCWSTR;
            if *pNext == 0 {
                PostMessageA(hWnd, WM_IME_REPORT, (IR_STRINGEND) as WPARAM, 0);
            }

            let mut szAnsi: [CHAR; 3] = [0 as CHAR; 3];
            if Imm32WideToAnsi(pCurrent, 1, szAnsi.as_mut_ptr(), _countof!(szAnsi) as INT) == 0 {
                break 'body;
            }

            if IsDBCSLeadByte(szAnsi[0] as BYTE) == 0 {
                PostMessageA(hWnd, WM_CHAR, szAnsi[0] as BYTE as WPARAM, 1);
                break 'body;
            }

            if bSupportDBCSReport != 0 {
                let mut wParam: WPARAM = (0x80000000 as WPARAM)
                    | (MAKEWORD!(szAnsi[1] as BYTE, szAnsi[0] as BYTE) as WPARAM);
                PostMessageA(hWnd, WM_CHAR, wParam, 1);
            } else {
                PostMessageA(hWnd, WM_CHAR, szAnsi[0] as BYTE as WPARAM, 1);
                PostMessageA(hWnd, WM_CHAR, szAnsi[1] as BYTE as WPARAM, 1);
            }
        }
        pCurrent = pNext;
    }
}

unsafe extern "C" fn Imm32CompStrWToCharW(hWnd: HWND, pCS: *const COMPOSITIONSTRING) {
    let mut pCurrent: PCWSTR = (pCS as *const BYTE).add((*pCS).dwResultStrOffset as usize) as PCWSTR;

    PostMessageW(hWnd, WM_IME_REPORT, (IR_STRINGSTART) as WPARAM, 0);

    let mut pNext: PCWSTR = core::ptr::null_mut();
    while *pCurrent != 0 {
        pNext = CharNextW(pCurrent) as PCWSTR;

        if *pNext == 0 {
            PostMessageW(hWnd, WM_IME_REPORT, (IR_STRINGEND) as WPARAM, 0);
        }

        PostMessageW(hWnd, WM_CHAR, *pCurrent as WPARAM, 1);

        pCurrent = pNext;
    }
}

unsafe extern "C" fn Imm32JTransCompositionA(
    pIC: *const INPUTCONTEXTDX,
    pCS: *const COMPOSITIONSTRING,
    pCurrent: *const TRANSMSG,
    pEntries: PTRANSMSG,
) -> INT {
    let hWnd: HWND = (*pIC).__bindgen_anon_1.hWnd;
    if IsWindow(hWnd) == 0 {
        return 0 as INT;
    }

    let bIsUnicodeWnd: BOOL = IsWindowUnicode(hWnd);
    let dwGCS: DWORD = (*pCurrent).lParam as DWORD;
    let mut cMessages: INT = 0 as INT;
    let mut bUndeterminedProcessed: BOOL = FALSE as BOOL;
    let mut bResultProcessed: BOOL = FALSE as BOOL;
    let mut hUndet: HGLOBAL = core::ptr::null_mut();
    let mut cbUndet: DWORD = 0 as DWORD;

    if ((*pIC).dwUIFlags & (_IME_UI_HIDDEN as DWORD)) != 0 {
        if bIsUnicodeWnd == 0 {
            cbUndet = Imm32CompStrAToUndetA(dwGCS, pCS, core::ptr::null_mut(), 0 as DWORD);
            if cbUndet != 0 {
                hUndet = GlobalAlloc(
                    (GHND | GMEM_SHARE) as UINT,
                    cbUndet as SIZE_T,
                );
                if !hUndet.is_null() {
                    let pUndet: PUNDETERMINESTRUCT = GlobalLock(hUndet) as PUNDETERMINESTRUCT;
                    if !pUndet.is_null() {
                        cbUndet = Imm32CompStrAToUndetA(dwGCS, pCS, pUndet, cbUndet);
                        GlobalUnlock(hUndet);
                        if cbUndet != 0 {
                            if SendMessageA(
                                hWnd,
                                WM_IME_REPORT as UINT,
                                IR_UNDETERMINE as WPARAM,
                                hUndet as LPARAM,
                            ) == 0 as LRESULT
                            {
                                bUndeterminedProcessed = TRUE as BOOL;
                            }
                        }
                    }
                }
            }
        } else {
            cbUndet = Imm32CompStrAToUndetW(dwGCS, pCS, core::ptr::null_mut(), 0 as DWORD);
            if cbUndet != 0 {
                hUndet = GlobalAlloc(
                    (GHND | GMEM_SHARE) as UINT,
                    cbUndet as SIZE_T,
                );
                if !hUndet.is_null() {
                    let pUndet: PUNDETERMINESTRUCT = GlobalLock(hUndet) as PUNDETERMINESTRUCT;
                    if !pUndet.is_null() {
                        cbUndet = Imm32CompStrAToUndetW(dwGCS, pCS, pUndet, cbUndet);
                        GlobalUnlock(hUndet);
                        if cbUndet != 0 {
                            if SendMessageW(
                                hWnd,
                                WM_IME_REPORT as UINT,
                                IR_UNDETERMINE as WPARAM,
                                hUndet as LPARAM,
                            ) == 0 as LRESULT
                            {
                                bUndeterminedProcessed = TRUE as BOOL;
                            }
                        }
                    }
                }
            }
        }

        if !hUndet.is_null() {
            GlobalFree(hUndet);
        }
        if bUndeterminedProcessed != 0 {
            return 0 as INT;
        }
    }

    let mut hEx: HGLOBAL = core::ptr::null_mut();
    let mut hStr: HGLOBAL = core::ptr::null_mut();
    let mut pEx: PVOID = core::ptr::null_mut();
    let mut pStr: PVOID = core::ptr::null_mut();
    let mut bExSuccess: BOOL = FALSE as BOOL;
    let mut bStrSuccess: BOOL = FALSE as BOOL;
    let mut res: LRESULT = 0;
    let mut exSize: DWORD = 0;
    let mut strSize: DWORD = 0;

    'FINALIZE: {
        if (dwGCS & (GCS_RESULTSTR as DWORD)) != 0 {
            if (dwGCS & (GCS_RESULTREADSTR as DWORD)) != 0 {
                exSize = if bIsUnicodeWnd != 0 {
                    Imm32CompStrAToStringExW(dwGCS, pCS, core::ptr::null_mut(), 0 as DWORD)
                } else {
                    Imm32CompStrAToStringExA(dwGCS, pCS, core::ptr::null_mut(), 0 as DWORD)
                };
                if exSize != 0 {
                    hEx = GlobalAlloc(
                        (GHND | GMEM_SHARE) as UINT,
                        exSize as SIZE_T,
                    );
                    if !hEx.is_null() {
                        pEx = GlobalLock(hEx) as PVOID;
                        if !pEx.is_null() {
                            bExSuccess = (if bIsUnicodeWnd != 0 {
                                Imm32CompStrAToStringExW(dwGCS, pCS, pEx as LPSTRINGEXSTRUCT, exSize)
                            } else {
                                Imm32CompStrAToStringExA(dwGCS, pCS, pEx as LPSTRINGEXSTRUCT, exSize)
                            }) as BOOL;
                            GlobalUnlock(hEx);
                            if bExSuccess != 0 {
                                res = if bIsUnicodeWnd != 0 {
                                    SendMessageW(
                                        hWnd,
                                        WM_IME_REPORT as UINT,
                                        IR_STRINGEX as WPARAM,
                                        hEx as LPARAM,
                                    )
                                } else {
                                    SendMessageA(
                                        hWnd,
                                        WM_IME_REPORT as UINT,
                                        IR_STRINGEX as WPARAM,
                                        hEx as LPARAM,
                                    )
                                };
                                if res != 0 {
                                    GlobalFree(hEx);
                                    bResultProcessed = TRUE as BOOL;
                                    break 'FINALIZE;
                                }
                            }
                        }
                        GlobalFree(hEx);
                    }
                }
            }

            strSize = if bIsUnicodeWnd != 0 {
                Imm32CompStrAToStringW(pCS, core::ptr::null_mut(), 0 as DWORD)
            } else {
                (*pCS).dwResultStrLen.wrapping_add(1 as DWORD)
            };
            if strSize != 0 && strSize != (0u32.wrapping_sub(1 as DWORD)) {
                hStr = GlobalAlloc(
                    (GHND | GMEM_SHARE) as UINT,
                    strSize as SIZE_T,
                );
                if !hStr.is_null() {
                    pStr = GlobalLock(hStr) as PVOID;
                    if !pStr.is_null() {
                        bStrSuccess = (if bIsUnicodeWnd != 0 {
                            Imm32CompStrAToStringW(pCS, pStr as PWSTR, strSize)
                        } else {
                            Imm32CompStrAToStringA(pCS, pStr as PSTR, strSize)
                        }) as BOOL;
                        GlobalUnlock(hStr);
                        if bStrSuccess != 0 {
                            res = if bIsUnicodeWnd != 0 {
                                SendMessageW(
                                    hWnd,
                                    WM_IME_REPORT as UINT,
                                    IR_STRING as WPARAM,
                                    hStr as LPARAM,
                                )
                            } else {
                                SendMessageA(
                                    hWnd,
                                    WM_IME_REPORT as UINT,
                                    IR_STRING as WPARAM,
                                    hStr as LPARAM,
                                )
                            };
                            if res != 0 {
                                GlobalFree(hStr);
                                bResultProcessed = TRUE as BOOL;
                                break 'FINALIZE;
                            }
                        }
                    }
                    GlobalFree(hStr);
                }
            }

            if bIsUnicodeWnd != 0 {
                Imm32CompStrAToCharW(hWnd, pCS);
            } else {
                Imm32CompStrAToCharA(hWnd, pCS);
            }

            bResultProcessed = TRUE as BOOL;
        }
    }

    if bUndeterminedProcessed == 0 {
        if bResultProcessed != 0 {
            if (dwGCS & (GCS_COMPSTR as DWORD)) != 0 {
                (*pEntries).message = (*pCurrent).message;
                (*pEntries).wParam = (*pCurrent).wParam;
                (*pEntries).lParam = (dwGCS
                    & !((GCS_RESULTCLAUSE | GCS_RESULTSTR | GCS_RESULTREADCLAUSE | GCS_RESULTREADSTR) as DWORD))
                    as LPARAM;
                cMessages = 1 as INT;
            }
        } else {
            *pEntries = *pCurrent;
            cMessages = 1 as INT;
        }
    }

    return cMessages;
}

unsafe extern "C" fn Imm32JTransCompositionW(
    pIC: *const INPUTCONTEXTDX,
    pCS: *const COMPOSITIONSTRING,
    pCurrent: *const TRANSMSG,
    pEntries: PTRANSMSG,
) -> INT {
    let hWnd: HWND = (*pIC).__bindgen_anon_1.hWnd;
    if IsWindow(hWnd) == 0 {
        return 0 as INT;
    }

    let bIsUnicodeWnd: BOOL = IsWindowUnicode(hWnd);
    let dwGCS: DWORD = (*pCurrent).lParam as DWORD;
    let mut cMessages: INT = 0 as INT;
    let mut bUndeterminedProcessed: BOOL = FALSE as BOOL;
    let mut bResultProcessed: BOOL = FALSE as BOOL;

    if ((*pIC).dwUIFlags & _IME_UI_HIDDEN as DWORD) != 0 {
        let mut hUndet: HGLOBAL = core::ptr::null_mut();
        let mut cbUndet: DWORD = 0 as DWORD;

        if bIsUnicodeWnd == 0 {
            cbUndet = Imm32CompStrWToUndetA(
                dwGCS,
                pCS,
                core::ptr::null_mut(),
                0 as DWORD,
            );
            if cbUndet != 0 {
                hUndet = GlobalAlloc(
                    (GHND | GMEM_SHARE) as UINT,
                    cbUndet as SIZE_T,
                );
                if !hUndet.is_null() {
                    let pUndet: PUNDETERMINESTRUCT =
                        GlobalLock(hUndet) as PUNDETERMINESTRUCT;
                    if !pUndet.is_null() {
                        cbUndet = Imm32CompStrWToUndetA(dwGCS, pCS, pUndet, cbUndet);
                        GlobalUnlock(hUndet);
                        if cbUndet != 0 {
                            if SendMessageA(
                                hWnd,
                                WM_IME_REPORT as UINT,
                                IR_UNDETERMINE as WPARAM,
                                hUndet as LPARAM,
                            ) == 0 as LRESULT
                            {
                                bUndeterminedProcessed = TRUE as BOOL;
                            }
                        }
                    }
                }
            }
        } else {
            cbUndet = Imm32CompStrWToUndetW(
                dwGCS,
                pCS,
                core::ptr::null_mut(),
                0 as DWORD,
            );
            if cbUndet != 0 {
                hUndet = GlobalAlloc(
                    (GHND | GMEM_SHARE) as UINT,
                    cbUndet as SIZE_T,
                );
                if !hUndet.is_null() {
                    let pUndet: PUNDETERMINESTRUCT =
                        GlobalLock(hUndet) as PUNDETERMINESTRUCT;
                    if !pUndet.is_null() {
                        cbUndet = Imm32CompStrWToUndetW(dwGCS, pCS, pUndet, cbUndet);
                        GlobalUnlock(hUndet);
                        if cbUndet != 0 {
                            if SendMessageW(
                                hWnd,
                                WM_IME_REPORT as UINT,
                                IR_UNDETERMINE as WPARAM,
                                hUndet as LPARAM,
                            ) == 0 as LRESULT
                            {
                                bUndeterminedProcessed = TRUE as BOOL;
                            }
                        }
                    }
                }
            }
        }

        if !hUndet.is_null() {
            GlobalFree(hUndet);
        }
        if bUndeterminedProcessed != 0 {
            return 0 as INT;
        }
    }

    let mut hEx: HGLOBAL = core::ptr::null_mut();
    let mut hStr: HGLOBAL = core::ptr::null_mut();
    let mut pEx: PVOID = core::ptr::null_mut();
    let mut pStr: PVOID = core::ptr::null_mut();
    let mut bExSuccess: BOOL = FALSE as BOOL;
    let mut bStrSuccess: BOOL = FALSE as BOOL;
    let mut res: LRESULT = 0;
    let mut exSize: DWORD = 0;
    let mut strSize: DWORD = 0;

    'FINALIZE: {
        if (dwGCS & GCS_RESULTSTR as DWORD) != 0 {
            'FINALIZE: {
                if (dwGCS & GCS_RESULTREADSTR as DWORD) != 0 {
                    exSize = if bIsUnicodeWnd != 0 {
                        Imm32CompStrWToStringExW(
                            dwGCS,
                            pCS,
                            core::ptr::null_mut(),
                            0 as DWORD,
                        )
                    } else {
                        Imm32CompStrWToStringExA(
                            dwGCS,
                            pCS,
                            core::ptr::null_mut(),
                            0 as DWORD,
                        )
                    };
                    if exSize != 0 {
                        hEx = GlobalAlloc(
                            (GHND | GMEM_SHARE) as UINT,
                            exSize as SIZE_T,
                        );
                        if !hEx.is_null() {
                            pEx = GlobalLock(hEx) as PVOID;
                            if !pEx.is_null() {
                                bExSuccess = (if bIsUnicodeWnd != 0 {
                                    Imm32CompStrWToStringExW(dwGCS, pCS, pEx as LPSTRINGEXSTRUCT, exSize)
                                } else {
                                    Imm32CompStrWToStringExA(dwGCS, pCS, pEx as LPSTRINGEXSTRUCT, exSize)
                                }) as BOOL;
                                GlobalUnlock(hEx);
                                if bExSuccess != 0 {
                                    res = if bIsUnicodeWnd != 0 {
                                        SendMessageW(
                                            hWnd,
                                            WM_IME_REPORT as UINT,
                                            IR_STRINGEX as WPARAM,
                                            hEx as LPARAM,
                                        )
                                    } else {
                                        SendMessageA(
                                            hWnd,
                                            WM_IME_REPORT as UINT,
                                            IR_STRINGEX as WPARAM,
                                            hEx as LPARAM,
                                        )
                                    };
                                    if res != 0 {
                                        GlobalFree(hEx);
                                        bResultProcessed = TRUE as BOOL;
                                        break 'FINALIZE;
                                    }
                                }
                            }
                            GlobalFree(hEx);
                        }
                    }
                }
            }

            'FINALIZE: {
                strSize = if bIsUnicodeWnd != 0 {
                    Imm32CompStrWToStringW(pCS, core::ptr::null_mut())
                } else {
                    Imm32CompStrWToStringA(pCS, core::ptr::null_mut())
                };
                if strSize != 0
                    && strSize != (0 as DWORD).wrapping_sub(1 as DWORD)
                {
                    hStr = GlobalAlloc(
                        (GHND | GMEM_SHARE) as UINT,
                        strSize as SIZE_T,
                    );
                    if !hStr.is_null() {
                        pStr = GlobalLock(hStr) as PVOID;
                        if !pStr.is_null() {
                            bStrSuccess = (if bIsUnicodeWnd != 0 {
                                Imm32CompStrWToStringW(pCS, pStr as PWCHAR)
                            } else {
                                Imm32CompStrWToStringA(pCS, pStr as PCHAR)
                            }) as BOOL;
                            GlobalUnlock(hStr);
                            if bStrSuccess != 0 {
                                res = if bIsUnicodeWnd != 0 {
                                    SendMessageW(
                                        hWnd,
                                        WM_IME_REPORT as UINT,
                                        IR_STRING as WPARAM,
                                        hStr as LPARAM,
                                    )
                                } else {
                                    SendMessageA(
                                        hWnd,
                                        WM_IME_REPORT as UINT,
                                        IR_STRING as WPARAM,
                                        hStr as LPARAM,
                                    )
                                };
                                if res != 0 {
                                    GlobalFree(hStr);
                                    bResultProcessed = TRUE as BOOL;
                                    break 'FINALIZE;
                                }
                            }
                        }
                        GlobalFree(hStr);
                    }
                }
            }

            if bIsUnicodeWnd != 0 {
                Imm32CompStrWToCharW(hWnd, pCS);
            } else {
                Imm32CompStrWToCharA(hWnd, pCS);
            }

            bResultProcessed = TRUE as BOOL;
        }
    }

    // COMMENT_TOKEN_ECHO____: FINALIZE
    if bUndeterminedProcessed == 0 {
        if bResultProcessed != 0 {
            if (dwGCS & GCS_COMPSTR as DWORD) != 0 {
                (*pEntries).message = (*pCurrent).message;
                (*pEntries).wParam = (*pCurrent).wParam;
                (*pEntries).lParam = (dwGCS
                    & !((GCS_RESULTCLAUSE
                        | GCS_RESULTSTR
                        | GCS_RESULTREADCLAUSE
                        | GCS_RESULTREADSTR) as DWORD)) as LPARAM;
                cMessages = 1 as INT;
            }
        } else {
            *pEntries = *pCurrent;
            cMessages = 1 as INT;
        }
    }

    return cMessages;
}

unsafe extern "C" fn WINNLSTranslateMessageJ(
    mut cEntries: INT,
    mut pEntries: PTRANSMSG,
    pIC: *const INPUTCONTEXTDX,
    pCS: *const COMPOSITIONSTRING,
    bAnsi: BOOL,
) -> DWORD {
    // Clone the message list with growing
    let mut dwBufSize: DWORD = ((cEntries + 1) as usize
        * core::mem::size_of::<TRANSMSG>()) as DWORD;
    let mut pBuf: PTRANSMSG = ImmLocalAlloc(HEAP_ZERO_MEMORY, dwBufSize) as PTRANSMSG;
    if pBuf.is_null() {
        return 0 as DWORD;
    }
    RtlCopyMemory!(
        pBuf as *mut core::ffi::c_void,
        pEntries as *const core::ffi::c_void,
        (cEntries as usize) * core::mem::size_of::<TRANSMSG>(),
    );

    let hWnd: HWND = (*pIC).__bindgen_anon_1.hWnd;
    let hwndIme: HWND = ImmGetDefaultIMEWnd(hWnd);

    let mut pCurrent: PTRANSMSG = pBuf;
    if ((*pIC).dwUIFlags & _IME_UI_HIDDEN) != 0 {
        // Find WM_IME_ENDCOMPOSITION
        let mut pEndComp: PTRANSMSG = core::ptr::null_mut();
        let mut i: INT = 0;
        while i < cEntries {
            if (*pBuf.offset(i as isize)).message == WM_IME_ENDCOMPOSITION {
                pEndComp = pBuf.offset(i as isize);
                break;
            }
            i += 1;
        }

        if !pEndComp.is_null() {
            // Move WM_IME_ENDCOMPOSITION to the end of the list
            let mut pSrc: PTRANSMSG = pEndComp.offset(1 as isize);
            let mut pDest: PTRANSMSG = pEndComp;
            let mut iEntry: INT = 0;
            while iEntry < cEntries - 1 && (*pSrc).message != WM_NULL {
                *pDest = *pSrc;
                pDest = pDest.offset(1);
                pSrc = pSrc.offset(1);
                iEntry += 1;
            }

            (*pDest).message = WM_IME_ENDCOMPOSITION;
            (*pDest).lParam = 0 as LPARAM;
            (*pDest).wParam = (*pDest).lParam as WPARAM;
        }

        pCurrent = pBuf;
    }

    let mut dwResult: DWORD;
    dwResult = 0 as DWORD;
    while (*pCurrent).message != WM_NULL {
        match (*pCurrent).message {
            WM_IME_STARTCOMPOSITION => {
                if ((*pIC).dwUIFlags & _IME_UI_HIDDEN) == 0 {
                    // Send IR_OPENCONVERT
                    if (*pIC).__bindgen_anon_1.cfCompForm.dwStyle != 0 {
                        SendMessageW(
                            hWnd,
                            WM_IME_REPORT as UINT,
                            IR_OPENCONVERT as WPARAM,
                            0 as LPARAM,
                        );
                    }

                    *pEntries = *pCurrent;
                    pEntries = pEntries.offset(1);
                    dwResult += 1;
                }
            }

            WM_IME_ENDCOMPOSITION => {
                if ((*pIC).dwUIFlags & _IME_UI_HIDDEN) == 0 {
                    // Send IR_CLOSECONVERT
                    if (*pIC).__bindgen_anon_1.cfCompForm.dwStyle != 0 {
                        SendMessageW(
                            hWnd,
                            WM_IME_REPORT as UINT,
                            IR_CLOSECONVERT as WPARAM,
                            0 as LPARAM,
                        );
                    }

                    *pEntries = *pCurrent;
                    pEntries = pEntries.offset(1);
                    dwResult += 1;
                } else {
                    // Send IR_UNDETERMINE
                    let hMem: HGLOBAL = GlobalAlloc(
                        (GHND | GMEM_SHARE) as UINT,
                        core::mem::size_of::<UNDETERMINESTRUCT>() as SIZE_T,
                    );
                    if !hMem.is_null() {
                        if IsWindowUnicode(hWnd) != 0 {
                            SendMessageW(
                                hWnd,
                                WM_IME_REPORT as UINT,
                                IR_UNDETERMINE as WPARAM,
                                hMem as LPARAM,
                            );
                        } else {
                            SendMessageA(
                                hWnd,
                                WM_IME_REPORT as UINT,
                                IR_UNDETERMINE as WPARAM,
                                hMem as LPARAM,
                            );
                        }
                        GlobalFree(hMem);
                    }
                }
            }

            WM_IME_COMPOSITION => {
                let mut cMessages: INT;
                if bAnsi != 0 {
                    cMessages = Imm32JTransCompositionA(pIC, pCS, pCurrent as *const TRANSMSG, pEntries);
                } else {
                    cMessages = Imm32JTransCompositionW(pIC, pCS, pCurrent as *const TRANSMSG, pEntries);
                }

                dwResult += cMessages as DWORD;
                pEntries = pEntries.offset(cMessages as isize);

                // Send IR_CHANGECONVERT
                if ((*pIC).dwUIFlags & _IME_UI_HIDDEN) == 0 && (*pIC).__bindgen_anon_1.cfCompForm.dwStyle != 0 {
                    SendMessageW(
                        hWnd,
                        WM_IME_REPORT as UINT,
                        IR_CHANGECONVERT as WPARAM,
                        0 as LPARAM,
                    );
                }
            }

            WM_IME_NOTIFY => {
                if (*pCurrent).wParam == IMN_OPENCANDIDATE as WPARAM {
                    // Send IMC_SETCANDIDATEPOS
                    if IsWindow(hWnd) != 0 && ((*pIC).dwUIFlags & _IME_UI_HIDDEN) != 0 {
                        let mut CandForm: CANDIDATEFORM = core::mem::zeroed();
                        let lParam: LPARAM = (*pCurrent).lParam;
                        let mut iCandForm: DWORD = 0 as DWORD;
                        while iCandForm < MAX_CANDIDATEFORM as DWORD {
                            if (lParam & (((1 as DWORD) << iCandForm) as LPARAM)) != 0 {
                                CandForm.dwIndex = iCandForm;
                                CandForm.dwStyle = CFS_EXCLUDE as DWORD;
                                CandForm.ptCurrentPos = (*pIC).__bindgen_anon_1.cfCompForm.ptCurrentPos;
                                CandForm.rcArea = (*pIC).__bindgen_anon_1.cfCompForm.rcArea;
                                SendMessageW(
                                    hwndIme,
                                    WM_IME_CONTROL as UINT,
                                    IMC_SETCANDIDATEPOS as WPARAM,
                                    core::ptr::addr_of_mut!(CandForm) as LPARAM,
                                );
                            }
                            iCandForm += 1;
                        }
                    }
                }

                if ((*pIC).dwUIFlags & _IME_UI_HIDDEN) == 0 {
                    *pEntries = *pCurrent;
                    pEntries = pEntries.offset(1);
                    dwResult += 1;
                } else {
                    SendMessageW(
                        hwndIme,
                        (*pCurrent).message,
                        (*pCurrent).wParam,
                        (*pCurrent).lParam,
                    );
                }
            }

            _ => {
                *pEntries = *pCurrent;
                pEntries = pEntries.offset(1);
                dwResult += 1;
            }
        }
        pCurrent = pCurrent.offset(1);
    }

    ImmLocalFree!(pBuf as PVOID);
    return dwResult;
}

unsafe extern "C" fn WINNLSTranslateMessageK(
    cEntries: INT,
    pEntries: PTRANSMSG,
    pIC: *const INPUTCONTEXTDX,
    pCS: *const COMPOSITIONSTRING,
    bSrcIsAnsi: BOOL,
) -> DWORD {
    let hWnd: HWND = (*pIC).__bindgen_anon_1.hWnd;
    let hwndIme: HWND = ImmGetDefaultIMEWnd(hWnd);
    let bDestIsUnicode: BOOL = IsWindowUnicode(hWnd);
    let pPostMessage: FN_PostMessage = if bDestIsUnicode != 0 { PostMessageW } else { PostMessageA };
    let pSendMessage: FN_SendMessage = if bDestIsUnicode != 0 { SendMessageW } else { SendMessageA };
    static mut s_chKorean: WORD = 0 as WORD;

    if cEntries <= 0 as INT {
        return 0 as DWORD;
    }

    let mut i: INT = 0 as INT;
    while i < cEntries {
        let uMsg: UINT = (*pEntries.offset(i as isize)).message;
        let wParam: WPARAM = (*pEntries.offset(i as isize)).wParam;
        let lParam: LPARAM = (*pEntries.offset(i as isize)).lParam;

        match uMsg {
            WM_IME_COMPOSITION => {
                if (lParam & (GCS_RESULTSTR as LPARAM)) != 0 {
                    pPostMessage(
                        hWnd,
                        WM_IME_REPORT as UINT,
                        IR_STRINGSTART as WPARAM,
                        0 as LPARAM,
                    );

                    if (*pCS).dwResultStrLen != 0 {
                        let mut dwProcessedLen: DWORD = 0 as DWORD;
                        while dwProcessedLen < (*pCS).dwResultStrLen {
                            let mut lKeyData: LPARAM = 1 as LPARAM;
                            let mut szWide: [WCHAR; 2] =
                                [0 as WCHAR; 2];
                            let mut szMBStr: [CHAR; 4] =
                                [0 as CHAR; 4];

                            if bSrcIsAnsi != 0 {
                                let pResStr: PSTR =
                                    ((pCS as *mut BYTE).offset((*pCS).dwResultStrOffset as isize)) as PSTR;
                                let mut bChar: BYTE = *pResStr.offset(dwProcessedLen as isize) as BYTE;

                                if bDestIsUnicode != 0 {
                                    let mut cbMB: INT = 1 as INT;

                                    szMBStr[0 as usize] = bChar as CHAR;
                                    if IsDBCSLeadByte(bChar) != 0
                                        && dwProcessedLen.wrapping_add(1 as DWORD)
                                            < (*pCS).dwResultStrLen
                                    {
                                        szMBStr[1 as usize] = *pResStr.offset(
                                            dwProcessedLen.wrapping_add(1 as DWORD)
                                                as isize,
                                        );
                                        cbMB = 2 as INT;
                                        dwProcessedLen = dwProcessedLen.wrapping_add(1);
                                    }
                                    Imm32AnsiToWide(
                                        core::ptr::addr_of_mut!(szMBStr) as PCCH,
                                        cbMB,
                                        core::ptr::addr_of_mut!(szWide) as PWSTR,
                                        _countof!(szWide) as INT,
                                    );
                                    PostMessageW(
                                        hWnd,
                                        WM_CHAR as UINT,
                                        szWide[0 as usize] as WPARAM,
                                        1 as LPARAM,
                                    );
                                } else {
                                    if IsDBCSLeadByte(bChar) != 0 {
                                        if KOR_IS_LEAD_BYTE!(bChar) != false {
                                            lKeyData = KOR_SCAN_CODE_DBCS as LPARAM;
                                        } else {
                                            lKeyData = KOR_SCAN_CODE_SBCS as LPARAM;
                                        }

                                        PostMessageA(
                                            hWnd,
                                            WM_CHAR as UINT,
                                            bChar as WPARAM,
                                            lKeyData,
                                        );
                                        dwProcessedLen = dwProcessedLen.wrapping_add(1);
                                        bChar = *pResStr.offset(dwProcessedLen as isize) as BYTE;
                                    }
                                    PostMessageA(
                                        hWnd,
                                        WM_CHAR as UINT,
                                        bChar as WPARAM,
                                        lKeyData,
                                    );
                                }
                            } else {
                                let pResStrW: PWSTR =
                                    ((pCS as PBYTE).offset((*pCS).dwResultStrOffset as isize)) as PWSTR;
                                szWide[0 as usize] =
                                    *pResStrW.offset(dwProcessedLen as isize);

                                if bDestIsUnicode != 0 {
                                    PostMessageW(
                                        hWnd,
                                        WM_CHAR as UINT,
                                        szWide[0 as usize] as WPARAM,
                                        1 as LPARAM,
                                    );
                                } else {
                                    Imm32WideToAnsi(
                                        core::ptr::addr_of_mut!(szWide) as PCWCH,
                                        1 as INT,
                                        core::ptr::addr_of_mut!(szMBStr) as PSTR,
                                        _countof!(szMBStr) as INT,
                                    );
                                    let bLead: BYTE = szMBStr[0 as usize] as BYTE;
                                    let bChar: BYTE;
                                    if IsDBCSLeadByte(bLead) != 0 {
                                        if KOR_IS_LEAD_BYTE!(bLead) != false {
                                            lKeyData = KOR_SCAN_CODE_DBCS as LPARAM;
                                        } else {
                                            lKeyData = KOR_SCAN_CODE_SBCS as LPARAM;
                                        }

                                        PostMessageA(
                                            hWnd,
                                            WM_CHAR as UINT,
                                            bLead as WPARAM,
                                            lKeyData,
                                        );
                                        bChar = szMBStr[1 as usize] as BYTE;
                                    } else {
                                        bChar = szMBStr[0 as usize] as BYTE;
                                    }

                                    PostMessageA(
                                        hWnd,
                                        WM_CHAR as UINT,
                                        bChar as WPARAM,
                                        lKeyData,
                                    );
                                }
                            }
                            dwProcessedLen = dwProcessedLen.wrapping_add(1);
                        }
                    }
                    pPostMessage(
                        hWnd,
                        WM_IME_REPORT as UINT,
                        IR_STRINGEND as WPARAM,
                        0 as LPARAM,
                    );
                }

                if wParam != 0 {
                    pPostMessage(
                        hWnd,
                        WM_IME_REPORT as UINT,
                        IR_STRINGSTART as WPARAM,
                        0 as LPARAM,
                    );

                    let bFirst: BYTE = HIBYTE!(wParam) as BYTE;
                    let bSecond: BYTE = LOBYTE!(wParam) as BYTE;
                    s_chKorean = MAKEWORD!(bFirst, bSecond) as WORD;

                    if bSrcIsAnsi != 0 {
                        if bDestIsUnicode != 0 {
                            let mut szTmp: [CHAR; 2] =
                                [bFirst as CHAR, bSecond as CHAR];
                            let mut wTmp: [WCHAR; 2] =
                                [0 as WCHAR; 2];
                            if Imm32AnsiToWide(
                                core::ptr::addr_of_mut!(szTmp) as PCCH,
                                2 as INT,
                                core::ptr::addr_of_mut!(wTmp) as PWSTR,
                                _countof!(wTmp) as INT,
                            ) != 0
                            {
                                PostMessageW(
                                    hWnd,
                                    WM_INTERIM as UINT,
                                    wTmp[0 as usize] as WPARAM,
                                    KOR_INTERIM_FLAGS as LPARAM,
                                );
                            }
                        } else {
                            PostMessageA(
                                hWnd,
                                WM_INTERIM as UINT,
                                bFirst as WPARAM,
                                KOR_INTERIM_FLAGS as LPARAM,
                            );
                            PostMessageA(
                                hWnd,
                                WM_INTERIM as UINT,
                                bSecond as WPARAM,
                                KOR_INTERIM_FLAGS as LPARAM,
                            );
                        }
                    } else {
                        if bDestIsUnicode != 0 {
                            PostMessageW(
                                hWnd,
                                WM_INTERIM as UINT,
                                wParam,
                                KOR_INTERIM_FLAGS as LPARAM,
                            );
                        } else {
                            let mut wTmp_0: [WCHAR; 2] =
                                [wParam as WCHAR, 0 as WCHAR];
                            let mut szTmp_0: [CHAR; 2] =
                                [0 as CHAR; 2];
                            Imm32WideToAnsi(
                                core::ptr::addr_of_mut!(wTmp_0) as PCWCH,
                                1 as INT,
                                core::ptr::addr_of_mut!(szTmp_0) as PSTR,
                                _countof!(szTmp_0) as INT,
                            );
                            PostMessageA(
                                hWnd,
                                WM_INTERIM as UINT,
                                szTmp_0[0 as usize] as BYTE as WPARAM,
                                KOR_INTERIM_FLAGS as LPARAM,
                            );
                            PostMessageA(
                                hWnd,
                                WM_INTERIM as UINT,
                                szTmp_0[1 as usize] as BYTE as WPARAM,
                                KOR_INTERIM_FLAGS as LPARAM,
                            );
                        }
                    }

                    if bDestIsUnicode != 0 {
                        PostMessageW(
                            hWnd,
                            WM_IME_REPORT as UINT,
                            IR_STRINGEND as WPARAM,
                            0 as LPARAM,
                        );
                    } else {
                        PostMessageA(
                            hWnd,
                            WM_IME_REPORT as UINT,
                            IR_STRINGEND as WPARAM,
                            0 as LPARAM,
                        );
                    }

                    pSendMessage(
                        hwndIme,
                        WM_IME_ENDCOMPOSITION as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    );
                } else {
                    pPostMessage(
                        hWnd,
                        WM_IME_REPORT as UINT,
                        IR_STRINGSTART as WPARAM,
                        0 as LPARAM,
                    );
                    if bSrcIsAnsi != 0 {
                        if bDestIsUnicode != 0 {
                            let mut szTmp_1: [CHAR; 2] =
                                [0 as CHAR; 2];
                            szTmp_1[0 as usize] = LOBYTE!(s_chKorean) as CHAR;
                            szTmp_1[1 as usize] = HIBYTE!(s_chKorean) as CHAR;
                            let mut wTmp_1: [WCHAR; 2] =
                                [0 as WCHAR; 2];
                            if Imm32AnsiToWide(
                                core::ptr::addr_of_mut!(szTmp_1) as PCCH,
                                2 as INT,
                                core::ptr::addr_of_mut!(wTmp_1) as PWSTR,
                                _countof!(wTmp_1) as INT,
                            ) != 0
                            {
                                PostMessageW(
                                    hWnd,
                                    WM_CHAR as UINT,
                                    wTmp_1[0 as usize] as WPARAM,
                                    KOR_SCAN_CODE_SBCS as LPARAM,
                                );
                            }
                            PostMessageW(
                                hWnd,
                                WM_IME_REPORT as UINT,
                                IR_STRINGEND as WPARAM,
                                0 as LPARAM,
                            );
                            PostMessageW(
                                hWnd,
                                WM_KEYDOWN as UINT,
                                VK_BACK as WPARAM,
                                KOR_KEYDOWN_FLAGS as LPARAM,
                            );
                        } else {
                            PostMessageA(
                                hWnd,
                                WM_CHAR as UINT,
                                LOBYTE!(s_chKorean) as WPARAM,
                                KOR_SCAN_CODE_SBCS as LPARAM,
                            );
                            PostMessageA(
                                hWnd,
                                WM_CHAR as UINT,
                                HIBYTE!(s_chKorean) as WPARAM,
                                KOR_SCAN_CODE_SBCS as LPARAM,
                            );
                            PostMessageA(
                                hWnd,
                                WM_IME_REPORT as UINT,
                                IR_STRINGEND as WPARAM,
                                0 as LPARAM,
                            );
                            PostMessageA(
                                hWnd,
                                WM_KEYDOWN as UINT,
                                VK_BACK as WPARAM,
                                KOR_KEYDOWN_FLAGS as LPARAM,
                            );
                        }
                    } else {
                        if bDestIsUnicode != 0 {
                            PostMessageW(
                                hWnd,
                                WM_CHAR as UINT,
                                (s_chKorean as WCHAR) as WPARAM,
                                KOR_SCAN_CODE_SBCS as LPARAM,
                            );
                            PostMessageW(
                                hWnd,
                                WM_IME_REPORT as UINT,
                                IR_STRINGEND as WPARAM,
                                0 as LPARAM,
                            );
                            PostMessageW(
                                hWnd,
                                WM_KEYDOWN as UINT,
                                VK_BACK as WPARAM,
                                KOR_KEYDOWN_FLAGS as LPARAM,
                            );
                        } else {
                            let mut szTmp_2: [CHAR; 2] =
                                [0 as CHAR; 2];
                            let mut wTmp_2: [WCHAR; 2] =
                                [s_chKorean, 0 as WCHAR];
                            Imm32WideToAnsi(
                                core::ptr::addr_of_mut!(wTmp_2) as PCWCH,
                                1 as INT,
                                core::ptr::addr_of_mut!(szTmp_2) as PSTR,
                                _countof!(szTmp_2) as INT,
                            );
                            PostMessageA(
                                hWnd,
                                WM_CHAR as UINT,
                                szTmp_2[0 as usize] as BYTE as WPARAM,
                                KOR_SCAN_CODE_SBCS as LPARAM,
                            );
                            PostMessageA(
                                hWnd,
                                WM_CHAR as UINT,
                                szTmp_2[1 as usize] as BYTE as WPARAM,
                                KOR_SCAN_CODE_SBCS as LPARAM,
                            );
                            PostMessageA(
                                hWnd,
                                WM_IME_REPORT as UINT,
                                IR_STRINGEND as WPARAM,
                                0 as LPARAM,
                            );
                            PostMessageA(
                                hWnd,
                                WM_KEYDOWN as UINT,
                                VK_BACK as WPARAM,
                                KOR_KEYDOWN_FLAGS as LPARAM,
                            );
                        }
                    }
                }
            }

            WM_IMEKEYDOWN => {
                pPostMessage(
                    hWnd,
                    WM_KEYDOWN as UINT,
                    LOWORD!(wParam) as WPARAM,
                    lParam,
                );
            }

            WM_IMEKEYUP => {
                pPostMessage(
                    hWnd,
                    WM_KEYUP as UINT,
                    LOWORD!(wParam) as WPARAM,
                    lParam,
                );
            }

            _ => {
                pSendMessage(hwndIme, uMsg, wParam, lParam);
            }
        }
        i += 1;
    }

    return 0 as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn WINNLSTranslateMessage(
    cEntries: INT,
    pEntries: PTRANSMSG,
    hIMC: HIMC,
    bAnsi: BOOL,
    wLang: WORD,
) -> DWORD {
    let pIC: PINPUTCONTEXTDX = ImmLockIMC(hIMC) as PINPUTCONTEXTDX;
    if pIC.is_null() {
        return 0;
    }

    let pCompStr: PCOMPOSITIONSTRING = ImmLockIMCC((*pIC).__bindgen_anon_1.hCompStr) as PCOMPOSITIONSTRING;
    if pCompStr.is_null() {
        ImmUnlockIMC(hIMC);
        return 0;
    }

    let mut ret: DWORD;
    if (wLang as INT) as u32 == LANG_KOREAN {
        ret = WINNLSTranslateMessageK(
            cEntries,
            pEntries,
            pIC as *const INPUTCONTEXTDX,
            pCompStr as *const COMPOSITIONSTRING,
            bAnsi,
        );
    } else if (wLang as INT) as u32 == LANG_JAPANESE {
        ret = WINNLSTranslateMessageJ(
            cEntries,
            pEntries,
            pIC as *const INPUTCONTEXTDX,
            pCompStr as *const COMPOSITIONSTRING,
            bAnsi,
        );
    } else {
        ret = 0;
    }

    ImmUnlockIMCC((*pIC).__bindgen_anon_1.hCompStr);
    ImmUnlockIMC(hIMC);
    return ret;
}
