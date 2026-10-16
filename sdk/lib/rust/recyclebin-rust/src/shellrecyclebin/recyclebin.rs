// rustconv-assembly: whole-tu
use crate::prelude::*;

// COBJMACROS for IUnknown and the winuser.h LOWORD macro. The crate's `commacros` cover the
// four recycle-bin interfaces but not IUnknown, and `win32macros` is not a dependency here.
#[inline]
unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    // The island types IUnknownVtbl's riid against its own GUID rendering; same 16 bytes.
    ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid.cast(), ppv)
}
#[inline]
unsafe fn IUnknown_Release(This: *mut IUnknown) -> ULONG {
    ((*(*This).lpVtbl).Release.unwrap_unchecked())(This)
}
/// `LOWORD(l)` (winuser.h) -- the low 16 bits, as a WORD.
#[inline]
fn LOWORD(l: DWORD) -> WORD {
    (l & 0xFFFF) as WORD
}
// Both constructors exist TWICE in this crate: in the superseded whole-TU modules
// (recyclebin_generic.rs / recyclebin_v5.rs, both ORPHAN-BY-DESIGN and unwired) and in the
// live class-atomic free halves. Import the LIVE ones.
use crate::shellrecyclebin::recyclebin_generic_free::RecycleBinGeneric_Constructor;
use crate::shellrecyclebin::recyclebin_v5_free::RecycleBin5_Constructor;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _BBENUMFILECONTEXT {
    pub pFI: *const _RECYCLEBINFILEIDENTITY,
    pub hDelFile: HDELFILE,
}
pub type BBENUMFILECONTEXT = _BBENUMFILECONTEXT;
#[no_mangle]
pub unsafe extern "system" fn CloseRecycleBinHandle(hDeletedFile: HDELFILE) -> BOOL {
    let rbf = IRecycleBinFileFromHDELFILE(hDeletedFile);
    let hr: HRESULT = (IRecycleBinFile_Release(rbf)) as i32;
    if hr >= 0 {
        return (TRUE) as i32;
    }
    if HRESULT_FACILITY(hr) == (FACILITY_WIN32) as i32 {
        SetLastError((HRESULT_CODE(hr)) as DWORD);
    } else {
        SetLastError(ERROR_GEN_FAILURE);
    }
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn DeleteFileToRecycleBinA(FileName: *const i8) -> i32 {
    let mut len: i32;
    let mut FileNameW: *mut u16 = core::ptr::null_mut();
    let mut ret: i32 = (FALSE) as i32;

    'cleanup: {
        if FileName.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
            break 'cleanup;
        }

        len = MultiByteToWideChar(
            CP_ACP,
            0,
            FileName,
            -1,
            core::ptr::null_mut(),
            0,
        );

        if len == 0 {
            break 'cleanup;
        }

        FileNameW = HeapAlloc(
            GetProcessHeap(),
            0,
            ((len as usize) * 2) as SIZE_T,
        ) as *mut u16;

        if FileNameW.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        }

        if MultiByteToWideChar(
            CP_ACP,
            0,
            FileName,
            -1,
            FileNameW,
            len,
        ) == 0
        {
            break 'cleanup;
        }

        ret = DeleteFileToRecycleBinW(FileNameW);
    }

    if !FileNameW.is_null() {
        HeapFree(GetProcessHeap(), 0, (FileNameW).cast());
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn DeleteFileToRecycleBinW(FileName: LPCWSTR) -> i32 {
    let mut hr: i32;
    let mut prb: *mut IRecycleBin = core::ptr::null_mut();
    hr = unsafe { GetDefaultRecycleBin(core::ptr::null_mut(), &mut prb) };
    if hr >= 0 {
        hr = unsafe { IRecycleBin_DeleteFile(prb, FileName) };
        unsafe { IRecycleBin_Release(prb) };
    }
    if hr >= 0 {
        return (TRUE) as i32;
    }
    if unsafe { HRESULT_FACILITY(hr) } == (FACILITY_WIN32) as i32 {
        unsafe { SetLastError((HRESULT_CODE(hr)) as DWORD) };
    } else {
        unsafe { SetLastError(ERROR_GEN_FAILURE) };
    }
    return (FALSE) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn DeleteFileInRecycleBin(hDeletedFile: HDELFILE) -> BOOL {
    let rbf: *mut IRecycleBinFile = IRecycleBinFileFromHDELFILE(hDeletedFile);
    let hr: HRESULT = IRecycleBinFile_Delete(rbf);

    if hr >= 0 {
        return (TRUE) as i32;
    }
    if HRESULT_FACILITY(hr) == (FACILITY_WIN32) as i32 {
        SetLastError((HRESULT_CODE(hr)) as DWORD);
    } else {
        SetLastError(ERROR_GEN_FAILURE);
    }
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn EmptyRecycleBinA(pszRoot: LPCSTR) -> i32 {
    let mut ret: i32 = (FALSE) as i32;
    let mut szRootW: LPWSTR = core::ptr::null_mut();
    let len: i32;

    'cleanup: loop {
        if !pszRoot.is_null() {
            len = MultiByteToWideChar(
                CP_ACP,
                0,
                pszRoot,
                -1,
                core::ptr::null_mut(),
                0,
            );
            if len == 0 {
                break 'cleanup;
            }
            szRootW = (HeapAlloc(
                GetProcessHeap(),
                0,
                (len * (core::mem::size_of::<u16>()) as i32) as SIZE_T,
            )).cast();
            if szRootW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            if MultiByteToWideChar(
                CP_ACP,
                0,
                pszRoot,
                -1,
                szRootW,
                len,
            ) == 0
            {
                break 'cleanup;
            }
        }
        ret = EmptyRecycleBinW(szRootW);
        break 'cleanup;
    }

    HeapFree(
        GetProcessHeap(),
        0,
        (szRootW).cast(),
    );
    ret
}

#[no_mangle]
pub unsafe extern "system" fn EmptyRecycleBinW(pszRoot: LPCWSTR) -> i32 {
    let mut hr: HRESULT;
    let mut prb: *mut IRecycleBin = core::ptr::null_mut();

    'main_block: {
        hr = GetDefaultRecycleBin(pszRoot, core::ptr::addr_of_mut!(prb));
        if hr < 0 {
            break 'main_block;
        }
        hr = IRecycleBin_EmptyRecycleBin(prb);
        IRecycleBin_Release(prb);
    }

    if hr >= 0 {
        return (TRUE) as i32;
    }
    if HRESULT_FACILITY(hr) == (FACILITY_WIN32) as i32 {
        SetLastError((HRESULT_CODE(hr)) as DWORD);
    } else {
        SetLastError(ERROR_GEN_FAILURE);
    }
    return (FALSE) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn EnumerateRecycleBinA(
    pszRoot: LPCSTR,
    pFnCallback: PENUMERATE_RECYCLEBIN_CALLBACK,
    Context: PVOID,
) -> BOOL {
    let mut len: i32;
    let mut szRootW: LPWSTR = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE) as i32;

    'cleanup: {
        if !pszRoot.is_null() {
            len = MultiByteToWideChar(
                CP_ACP,
                0,
                pszRoot,
                -1,
                core::ptr::null_mut(),
                0,
            );
            if len == 0 {
                break 'cleanup;
            }
            szRootW = HeapAlloc(
                GetProcessHeap(),
                0,
                (len as usize * core::mem::size_of::<u16>()) as SIZE_T,
            ) as LPWSTR;
            if szRootW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            if MultiByteToWideChar(
                CP_ACP,
                0,
                pszRoot,
                -1,
                szRootW,
                len,
            ) == 0
            {
                break 'cleanup;
            }
        }
        ret = EnumerateRecycleBinW(szRootW, pFnCallback, Context);
    }

    HeapFree(GetProcessHeap(), 0, (szRootW).cast());
    ret
}

#[no_mangle]
pub unsafe extern "system" fn EnumerateRecycleBinW(
    pszRoot: LPCWSTR,
    pFnCallback: PENUMERATE_RECYCLEBIN_CALLBACK,
    Context: PVOID,
) -> BOOL {
    let mut prb: *mut IRecycleBin = core::ptr::null_mut();
    let mut prbel: *mut IRecycleBinEnumList = core::ptr::null_mut();
    let mut prbf: *mut IRecycleBinFile = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut error: UINT;

    hr = GetDefaultRecycleBin(core::ptr::null_mut(), core::ptr::addr_of_mut!(prb));

    'cleanup: {
        if !(hr >= 0) {
            break 'cleanup;
        }
        hr = IRecycleBin_EnumObjects(prb, core::ptr::addr_of_mut!(prbel));
        if !(hr >= 0) {
            break 'cleanup;
        }
        loop {
            hr = IRecycleBinEnumList_Next(prbel, 1, core::ptr::addr_of_mut!(prbf), core::ptr::null_mut());
            if hr == S_FALSE {
                hr = S_OK;
                break 'cleanup;
            } else if !(hr >= 0) {
                break 'cleanup;
            }
            if pFnCallback.unwrap_unchecked()(Context, prbf as HDELFILE) == 0 {
                error = GetLastError();
                hr = HRESULT_FROM_WIN32(error);
                break 'cleanup;
            }
        }
    }

    if !prb.is_null() {
        IRecycleBin_Release(prb);
    }
    if !prbel.is_null() {
        IRecycleBinEnumList_Release(prbel);
    }
    if hr >= 0 {
        return (TRUE) as i32;
    }
    if HRESULT_FACILITY(hr) == (FACILITY_WIN32) as i32 {
        SetLastError((HRESULT_CODE(hr)) as DWORD);
    } else {
        SetLastError(ERROR_GEN_FAILURE);
    }
    return (FALSE) as i32;
}
#[cfg(target_arch = "x86")]
unsafe extern "system" fn GetRecycleBinFileHandleCallback(Context: PVOID, hDeletedFile: HDELFILE) -> BOOL {
    let pCtx = Context as *mut BBENUMFILECONTEXT;
    let pRBF = IRecycleBinFileFromHDELFILE(hDeletedFile);
    if IRecycleBinFile_IsEqualIdentity(pRBF, (*pCtx).pFI) == S_OK {
        (*pCtx).hDelFile = hDeletedFile;
        return FALSE as BOOL;
    }
    CloseRecycleBinHandle(hDeletedFile);
    // `BOOL` is i32 while `TRUE`/`FALSE` are u32 consts -- an i386-only mismatch, since the
    // other two arches infer it away. Cast to the declared return type.
    return TRUE as BOOL;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "system" fn GetRecycleBinFileHandleCallback(Context: PVOID, hDeletedFile: HDELFILE) -> BOOL {
    let pCtx = Context as *mut BBENUMFILECONTEXT;
    let pRBF = IRecycleBinFileFromHDELFILE(hDeletedFile);
    if IRecycleBinFile_IsEqualIdentity(pRBF, (*pCtx).pFI) == S_OK {
        (*pCtx).hDelFile = hDeletedFile;
        return (FALSE) as i32;
    }
    CloseRecycleBinHandle(hDeletedFile);
    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn GetRecycleBinFileHandle(
    pszRoot: LPCWSTR,
    pFI: *const RECYCLEBINFILEIDENTITY,
) -> HDELFILE {
    let mut context = BBENUMFILECONTEXT {
        pFI: pFI,
        hDelFile: core::ptr::null_mut(),
    };
    EnumerateRecycleBinW(
        pszRoot,
        Some(GetRecycleBinFileHandleCallback),
        &mut context as *mut _ as PVOID,
    );
    context.hDelFile
}
#[no_mangle]
pub unsafe extern "C" fn RemoveFromRecycleBinDatabase(pFI: *const RECYCLEBINFILEIDENTITY) -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;
    let hDelFile = GetRecycleBinFileHandle(core::ptr::null_mut(), pFI);
    if !hDelFile.is_null() {
        let rbf = IRecycleBinFileFromHDELFILE(hDelFile);
        ret = (IRecycleBinFile_RemoveFromDatabase(rbf) >= 0) as BOOL;
        CloseRecycleBinHandle(hDelFile);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn RestoreFileFromRecycleBin(hDeletedFile: HDELFILE) -> i32 {
    let rbf: *mut IRecycleBinFile = IRecycleBinFileFromHDELFILE(hDeletedFile);
    let hr: HRESULT = IRecycleBinFile_Restore(rbf);
    if hr >= 0 {
        return TRUE as i32;
    }
    if HRESULT_FACILITY(hr) == (FACILITY_WIN32) as i32 {
        SetLastError((HRESULT_CODE(hr)) as DWORD);
    } else {
        SetLastError(ERROR_GEN_FAILURE);
    }
    return FALSE as i32;
}
#[no_mangle]
pub unsafe extern "C" fn GetDefaultRecycleBin(
    pszVolume: LPCWSTR,
    pprb: *mut *mut IRecycleBin,
) -> HRESULT {
    let mut pUnk: *mut IUnknown = core::ptr::null_mut();
    let mut hr: HRESULT;

    if pprb.is_null() {
        return E_POINTER;
    }

    if pszVolume.is_null() {
        hr = RecycleBinGeneric_Constructor(core::ptr::addr_of_mut!(pUnk).cast());
    } else {
        let szRootPath: [WCHAR; 4] = [*pszVolume, ':' as u16, '\\' as u16, 0];
        let drive = PathGetDriveNumberW(pszVolume);
        if drive < 0 || *pszVolume.offset(2 as isize) != '\\' as u16 {
            return HRESULT_FROM_WIN32(ERROR_INVALID_NAME);
        }

        /* For now, only support this type of recycle bins... */
        hr = RecycleBin5_Constructor(szRootPath.as_ptr(), core::ptr::addr_of_mut!(pUnk).cast());
    }

    if !(hr >= 0) {
        return hr;
    }

    hr = IUnknown_QueryInterface(
        pUnk,
        &IID_IRecycleBin,
        pprb as *mut *mut core::ffi::c_void,
    );
    IUnknown_Release(pUnk);

    hr
}
#[no_mangle]
pub unsafe extern "C" fn GetRecycleBinPathFromDriveNumber(Drive: u32, Path: *mut u16) -> i32 {
    let volume: [u16; 4] = [
        LOWORD(('A' as u32 + Drive) as u32),
        ':' as u16,
        '\\' as u16,
        0u16,
    ];
    let mut pRB: *mut IRecycleBin = core::ptr::null_mut();
    let mut hr: i32 = GetDefaultRecycleBin(volume.as_ptr() as LPCWSTR, &mut pRB as *mut *mut IRecycleBin);
    if hr >= 0 {
        hr = IRecycleBin_GetDirectory(pRB, Path);
        IRecycleBin_Release(pRB);
    }
    hr
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Recycle bin management
//  * LICENSE:     GPL v2 - See COPYING in the top level directory
//  * FILE:        lib/recyclebin/recyclebin.c
//  * PURPOSE:     Public interface
//  * PROGRAMMERS: Copyright 2006-2007 Hervé Poussineau (hpoussin@reactos.org)
//  */
//
// #include "recyclebin_private.h"
//
// BOOL WINAPI
// CloseRecycleBinHandle(
//      HDELFILE hDeletedFile)
// {
//     IRecycleBinFile *rbf = IRecycleBinFileFromHDELFILE(hDeletedFile);
//     HRESULT hr;
//
//     TRACE("(%p)\n", hDeletedFile);
//
//     hr = IRecycleBinFile_Release(rbf);
//     if (SUCCEEDED(hr))
//         return TRUE;
//     if (HRESULT_FACILITY(hr) == FACILITY_WIN32)
//         SetLastError(HRESULT_CODE(hr));
//     else
//         SetLastError(ERROR_GEN_FAILURE);
//     return FALSE;
// }
//
// BOOL WINAPI
// DeleteFileToRecycleBinA(
//      LPCSTR FileName)
// {
//     int len;
//     LPWSTR FileNameW = NULL;
//     BOOL ret = FALSE;
//
//     TRACE("(%s)\n", debugstr_a(FileName));
//
//     /* Check parameters */
//     if (FileName == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         goto cleanup;
//     }
//
//     len = MultiByteToWideChar(CP_ACP, 0, FileName, -1, NULL, 0);
//     if (len == 0)
//         goto cleanup;
//     FileNameW = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//     if (!FileNameW)
//     {
//         SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         goto cleanup;
//     }
//     if (MultiByteToWideChar(CP_ACP, 0, FileName, -1, FileNameW, len) == 0)
//         goto cleanup;
//
//     ret = DeleteFileToRecycleBinW(FileNameW);
//
// cleanup:
//     HeapFree(GetProcessHeap(), 0, FileNameW);
//     return ret;
// }
//
// BOOL WINAPI
// DeleteFileToRecycleBinW(
//      LPCWSTR FileName)
// {
//     IRecycleBin *prb;
//     HRESULT hr;
//
//     TRACE("(%s)\n", debugstr_w(FileName));
//
//     hr = GetDefaultRecycleBin(NULL, &prb);
//     if (!SUCCEEDED(hr))
//         goto cleanup;
//
//     hr = IRecycleBin_DeleteFile(prb, FileName);
//     IRecycleBin_Release(prb);
//
// cleanup:
//     if (SUCCEEDED(hr))
//         return TRUE;
//     if (HRESULT_FACILITY(hr) == FACILITY_WIN32)
//         SetLastError(HRESULT_CODE(hr));
//     else
//         SetLastError(ERROR_GEN_FAILURE);
//     return FALSE;
// }
//
// BOOL WINAPI
// DeleteFileInRecycleBin(
//      HDELFILE hDeletedFile)
// {
//     IRecycleBinFile *rbf = IRecycleBinFileFromHDELFILE(hDeletedFile);
//     HRESULT hr;
//
//     TRACE("(%p)\n", hDeletedFile);
//
//     hr = IRecycleBinFile_Delete(rbf);
//
//     if (SUCCEEDED(hr))
//         return TRUE;
//     if (HRESULT_FACILITY(hr) == FACILITY_WIN32)
//         SetLastError(HRESULT_CODE(hr));
//     else
//         SetLastError(ERROR_GEN_FAILURE);
//     return FALSE;
// }
//
// BOOL WINAPI
// EmptyRecycleBinA(
//      LPCSTR pszRoot )
// {
//     int len;
//     LPWSTR szRootW = NULL;
//     BOOL ret = FALSE;
//
//     TRACE("(%s)\n", debugstr_a(pszRoot));
//
//     if (pszRoot)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, pszRoot, -1, NULL, 0);
//         if (len == 0)
//             goto cleanup;
//         szRootW = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!szRootW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         if (MultiByteToWideChar(CP_ACP, 0, pszRoot, -1, szRootW, len) == 0)
//             goto cleanup;
//     }
//
//     ret = EmptyRecycleBinW(szRootW);
//
// cleanup:
//     HeapFree(GetProcessHeap(), 0, szRootW);
//     return ret;
// }
//
// BOOL WINAPI
// EmptyRecycleBinW(
//      LPCWSTR pszRoot )
// {
//     IRecycleBin *prb;
//     HRESULT hr;
//
//     TRACE("(%s)\n", debugstr_w(pszRoot));
//
//     hr = GetDefaultRecycleBin(pszRoot, &prb);
//     if (!SUCCEEDED(hr))
//         goto cleanup;
//
//     hr = IRecycleBin_EmptyRecycleBin(prb);
//     IRecycleBin_Release(prb);
//
// cleanup:
//     if (SUCCEEDED(hr))
//         return TRUE;
//     if (HRESULT_FACILITY(hr) == FACILITY_WIN32)
//         SetLastError(HRESULT_CODE(hr));
//     else
//         SetLastError(ERROR_GEN_FAILURE);
//     return FALSE;
// }
//
// BOOL WINAPI
// EnumerateRecycleBinA(
//      LPCSTR pszRoot ,
//      PENUMERATE_RECYCLEBIN_CALLBACK pFnCallback,
//      PVOID Context )
// {
//     int len;
//     LPWSTR szRootW = NULL;
//     BOOL ret = FALSE;
//
//     TRACE("(%s, %p, %p)\n", debugstr_a(pszRoot), pFnCallback, Context);
//
//     if (pszRoot)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, pszRoot, -1, NULL, 0);
//         if (len == 0)
//             goto cleanup;
//         szRootW = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//         if (!szRootW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         if (MultiByteToWideChar(CP_ACP, 0, pszRoot, -1, szRootW, len) == 0)
//             goto cleanup;
//     }
//
//     ret = EnumerateRecycleBinW(szRootW, pFnCallback, Context);
//
// cleanup:
//     HeapFree(GetProcessHeap(), 0, szRootW);
//     return ret;
// }
//
// BOOL WINAPI
// EnumerateRecycleBinW(
//      LPCWSTR pszRoot ,
//      PENUMERATE_RECYCLEBIN_CALLBACK pFnCallback,
//      PVOID Context )
// {
//     IRecycleBin *prb = NULL;
//     IRecycleBinEnumList *prbel = NULL;
//     IRecycleBinFile *prbf;
//     HRESULT hr;
//
//     TRACE("(%s, %p, %p)\n", debugstr_w(pszRoot), pFnCallback, Context);
//
//     hr = GetDefaultRecycleBin(NULL, &prb);
//     if (!SUCCEEDED(hr))
//         goto cleanup;
//
//     hr = IRecycleBin_EnumObjects(prb, &prbel);
//     if (!SUCCEEDED(hr))
//         goto cleanup;
//     while (TRUE)
//     {
//         hr = IRecycleBinEnumList_Next(prbel, 1, &prbf, NULL);
//         if (hr == S_FALSE)
//         {
//             hr = S_OK;
//             goto cleanup;
//         }
//         else if (!SUCCEEDED(hr))
//             goto cleanup;
//         if (!pFnCallback(Context, (HDELFILE)prbf))
//         {
//             UINT error = GetLastError();
//             hr = HRESULT_FROM_WIN32(error);
//             goto cleanup;
//         }
//     }
//
// cleanup:
//     if (prb)
//         IRecycleBin_Release(prb);
//     if (prbel)
//         IRecycleBinEnumList_Release(prbel);
//     if (SUCCEEDED(hr))
//         return TRUE;
//     if (HRESULT_FACILITY(hr) == FACILITY_WIN32)
//         SetLastError(HRESULT_CODE(hr));
//     else
//         SetLastError(ERROR_GEN_FAILURE);
//     return FALSE;
// }
//
// typedef struct _BBENUMFILECONTEXT
// {
//     const RECYCLEBINFILEIDENTITY *pFI;
//     HDELFILE hDelFile;
// } BBENUMFILECONTEXT;
//
// static BOOL __stdcall
// GetRecycleBinFileHandleCallback( PVOID Context,  HDELFILE hDeletedFile)
// {
//     BBENUMFILECONTEXT *pCtx = (BBENUMFILECONTEXT*)Context;
//     IRecycleBinFile *pRBF = IRecycleBinFileFromHDELFILE(hDeletedFile);
//     if (IRecycleBinFile_IsEqualIdentity(pRBF, pCtx->pFI) == S_OK)
//     {
//         pCtx->hDelFile = hDeletedFile;
//         return FALSE;
//     }
//     CloseRecycleBinHandle(hDeletedFile);
//     return TRUE;
// }
//
// extern HDELFILE
// GetRecycleBinFileHandle(
//      LPCWSTR pszRoot ,
//      const RECYCLEBINFILEIDENTITY *pFI)
// {
//     BBENUMFILECONTEXT context = { pFI, NULL };
//     EnumerateRecycleBinW(pszRoot, GetRecycleBinFileHandleCallback, &context);
//     return context.hDelFile;
// }
//
// extern BOOL
// RemoveFromRecycleBinDatabase(
//      const RECYCLEBINFILEIDENTITY *pFI)
// {
//     BOOL ret = FALSE;
//     HDELFILE hDelFile = GetRecycleBinFileHandle(NULL, pFI);
//     if (hDelFile)
//     {
//         IRecycleBinFile *rbf = IRecycleBinFileFromHDELFILE(hDelFile);
//         ret = SUCCEEDED(IRecycleBinFile_RemoveFromDatabase(rbf));
//         CloseRecycleBinHandle(hDelFile);
//     }
//     return ret;
// }
//
// BOOL WINAPI
// RestoreFileFromRecycleBin(
//      HDELFILE hDeletedFile)
// {
//     IRecycleBinFile *rbf = IRecycleBinFileFromHDELFILE(hDeletedFile);
//     HRESULT hr;
//
//     TRACE("(%p)\n", hDeletedFile);
//
//     hr = IRecycleBinFile_Restore(rbf);
//     if (SUCCEEDED(hr))
//         return TRUE;
//     if (HRESULT_FACILITY(hr) == FACILITY_WIN32)
//         SetLastError(HRESULT_CODE(hr));
//     else
//         SetLastError(ERROR_GEN_FAILURE);
//     return FALSE;
// }
//
// extern HRESULT
// GetDefaultRecycleBin(
//      LPCWSTR pszVolume ,
//      IRecycleBin **pprb)
// {
//     IUnknown *pUnk;
//     HRESULT hr;
//
//     TRACE("(%s, %p)\n", debugstr_w(pszVolume), pprb);
//
//     if (!pprb)
//         return E_POINTER;
//
//     if (!pszVolume)
//         hr = RecycleBinGeneric_Constructor(&pUnk);
//     else
//     {
//         /* FIXME: do a better validation! */
//         if (wcslen(pszVolume) != 3 || pszVolume[1] != ':' || pszVolume[2] != '\\')
//             return HRESULT_FROM_WIN32(ERROR_INVALID_NAME);
//
//         /* For now, only support this type of recycle bins... */
//         hr = RecycleBin5_Constructor(pszVolume, &pUnk);
//     }
//     if (!SUCCEEDED(hr))
//         return hr;
//     let mut hr = IUnknown_QueryInterface(pUnk, &IID_IRecycleBin, (void **)pprb);
//     IUnknown_Release(pUnk);
//     return hr;
// }
//
// extern HRESULT
// GetRecycleBinPathFromDriveNumber(UINT Drive, LPWSTR Path)
// {
//     const WCHAR volume[] = { LOWORD('A' + Drive), ':', '\\', '\0' };
//     IRecycleBin *pRB;
//     HRESULT hr = GetDefaultRecycleBin(volume, &pRB);
//     if (SUCCEEDED(hr))
//     {
//         hr = IRecycleBin_GetDirectory(pRB, Path);
//         IRecycleBin_Release(pRB);
//     }
//     return hr;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
