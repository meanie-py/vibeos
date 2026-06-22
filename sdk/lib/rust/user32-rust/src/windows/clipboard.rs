use win32k_sys::win32k::*;
// ntdll Rtl* heap/string helpers + RtlGetProcessHeap inline live in the substrate.
use crate::internal::{
    RtlAllocateHeap, RtlCreateUnicodeStringFromAsciiz, RtlFreeHeap, RtlFreeUnicodeString,
    RtlGetProcessHeap, RtlInitUnicodeString,
};
// NtUserx* syscall forwarder.
use crate::ntwrapper::NtUserxEnumClipboardFormats;
#[allow(unused_imports)]
use core::ffi::c_void;

// ERR/FIXME debug-channel macros — no-ops (WINE_DEFAULT_DEBUG_CHANNEL).
macro_rules! ERR { ($($t:tt)*) => {{ }}; }
macro_rules! FIXME { ($($t:tt)*) => {{ }}; }

const CP_ACP: UINT = 0;
const GMEM_MOVEABLE: UINT = 0x0002;
const GMEM_DDESHARE: UINT = 0x2000;

// kernel32 / gdi32 imports (extern "system" = stdcall on i386).
extern "system" {
    fn GdiConvertEnhMetaFile(h: HANDLE) -> HANDLE;
    fn GdiConvertMetaFilePict(h: HANDLE) -> HANDLE;
    fn GdiCreateLocalEnhMetaFile(h: HANDLE) -> HANDLE;
    fn GdiCreateLocalMetaFilePict(h: HANDLE) -> HANDLE;
    fn GlobalAlloc(uFlags: UINT, dwBytes: SIZE_T) -> HANDLE;
    fn GlobalLock(h: HANDLE) -> LPVOID;
    fn GlobalSize(h: HANDLE) -> DWORD;
    fn GlobalUnlock(h: HANDLE) -> i32;
    fn GlobalFree(h: HANDLE) -> HANDLE;
    fn GlobalHandle(pMem: LPCVOID) -> HANDLE;
    fn SetLastError(dwErrCode: DWORD);
    fn OemToCharBuffA(lpszSrc: LPCSTR, lpszDst: LPSTR, cchDstLength: DWORD) -> BOOL;
    fn CharToOemBuffA(lpszSrc: LPCSTR, lpszDst: LPSTR, cchDstLength: DWORD) -> BOOL;
    fn WideCharToMultiByte( CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: core::ffi::c_int, lpMultiByteStr: LPSTR, cbMultiByte: core::ffi::c_int, lpDefaultChar: LPCSTR, lpUsedDefaultChar: *mut core::ffi::c_int, ) -> core::ffi::c_int;
    fn MultiByteToWideChar( CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: core::ffi::c_int, lpWideCharStr: LPWSTR, cchWideChar: core::ffi::c_int, ) -> core::ffi::c_int;
}

// ---- file-scope preamble (translated) ----
pub type GdiConvertMetaFilePictFn = unsafe extern "system" fn(HANDLE) -> HANDLE;
pub type GdiConvertEnhMetaFileFn = unsafe extern "system" fn(HANDLE) -> HANDLE;
pub type GdiCreateLocalEnhMetaFileFn = unsafe extern "system" fn(HANDLE) -> HANDLE;
pub type GdiCreateLocalMetaFilePictFn = unsafe extern "system" fn(HANDLE) -> HANDLE;

#[no_mangle]
pub unsafe extern "system" fn OpenClipboard(hWndNewOwner: HWND) -> BOOL {
    NtUserOpenClipboard(hWndNewOwner, 0)
}

#[no_mangle]
pub unsafe extern "system" fn EnumClipboardFormats(format: UINT) -> UINT {
    SetLastError(NO_ERROR);
    NtUserxEnumClipboardFormats(format)
}

#[no_mangle]
pub unsafe extern "system" fn GetClipboardFormatNameA(
    format: UINT,
    lpszFormatName: LPSTR,
    cchMaxCount: core::ffi::c_int,
) -> INT {
    let lpBuffer: LPWSTR = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (cchMaxCount as SIZE_T) * core::mem::size_of::<WCHAR>() as SIZE_T,
    ) as LPWSTR;
    if lpBuffer.is_null() {
        SetLastError(ERROR_OUTOFMEMORY);
        return 0;
    }

    let mut Length: INT = NtUserGetClipboardFormatName(format, lpBuffer, cchMaxCount);

    if Length != 0 {
        if WideCharToMultiByte(
            CP_ACP,
            0,
            lpBuffer,
            Length,
            lpszFormatName,
            cchMaxCount,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            Length = 0;
        }
        *lpszFormatName.offset(Length as isize) = 0;
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, lpBuffer as PVOID);
    Length
}

#[no_mangle]
pub unsafe extern "system" fn GetClipboardFormatNameW(
    uFormat: UINT,
    lpszFormatName: LPWSTR,
    cchMaxCount: INT,
) -> INT {
    NtUserGetClipboardFormatName(uFormat, lpszFormatName, cchMaxCount)
}

#[no_mangle]
pub unsafe extern "system" fn RegisterClipboardFormatA(lpszFormat: LPCSTR) -> UINT {
    let ret: UINT;
    let mut usFormat: _UNICODE_STRING = core::mem::zeroed();

    if lpszFormat.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if *lpszFormat == 0 {
        SetLastError(ERROR_INVALID_NAME);
        return 0;
    }

    if RtlCreateUnicodeStringFromAsciiz(&mut usFormat, lpszFormat as *mut i8) == 0 {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    ret = NtUserRegisterWindowMessage(&mut usFormat);

    RtlFreeUnicodeString(&mut usFormat);

    ret
}

#[no_mangle]
pub unsafe extern "system" fn RegisterClipboardFormatW(lpszFormat: LPCWSTR) -> UINT {
    let mut usFormat: _UNICODE_STRING = core::mem::zeroed();

    if lpszFormat.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if *lpszFormat == 0 {
        SetLastError(ERROR_INVALID_NAME);
        return 0;
    }

    RtlInitUnicodeString(&mut usFormat, lpszFormat);
    NtUserRegisterWindowMessage(&mut usFormat)
}

unsafe fn IntSynthesizeMultiByte(pwStr: PVOID, cbStr: DWORD, bOem: BOOL) -> PVOID {

    const CP_ACP: UINT = 0;
    const CP_OEMCP: UINT = 1;
    const GMEM_DDESHARE: UINT = 0x2000;
    const GMEM_MOVEABLE: UINT = 0x0002;

    let hGlobal: HANDLE;
    let pGlobal: PVOID;
    let cbGlobal: core::ffi::c_int;

    cbGlobal = WideCharToMultiByte(
        if bOem != 0 { CP_OEMCP } else { CP_ACP },
        0,
        pwStr as LPCWSTR,
        (cbStr / core::mem::size_of::<u16>() as DWORD) as core::ffi::c_int,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    hGlobal = GlobalAlloc(GMEM_DDESHARE | GMEM_MOVEABLE, cbGlobal as SIZE_T);
    if hGlobal.is_null() {
        return core::ptr::null_mut();
    }

    pGlobal = GlobalLock(hGlobal);
    WideCharToMultiByte(
        if bOem != 0 { CP_OEMCP } else { CP_ACP },
        0,
        pwStr as LPCWSTR,
        (cbStr / core::mem::size_of::<u16>() as DWORD) as core::ffi::c_int,
        pGlobal as LPSTR,
        cbGlobal,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    pGlobal
}

unsafe fn IntSynthesizeWideChar(pwStr: PVOID, cbStr: DWORD, bOem: BOOL) -> PVOID {

    const CP_ACP: UINT = 0;
    const CP_OEMCP: UINT = 1;
    const GMEM_MOVEABLE: UINT = 0x0002;
    const GMEM_DDESHARE: UINT = 0x2000;

    let codepage = if bOem != 0 { CP_OEMCP } else { CP_ACP };

    let cbGlobal: INT = MultiByteToWideChar(
        codepage,
        0,
        pwStr as LPCSTR,
        cbStr as core::ffi::c_int,
        core::ptr::null_mut(),
        0,
    ) * core::mem::size_of::<u16>() as core::ffi::c_int;

    let hGlobal: HANDLE = GlobalAlloc(GMEM_DDESHARE | GMEM_MOVEABLE, cbGlobal as SIZE_T);
    if hGlobal.is_null() {
        return core::ptr::null_mut();
    }

    let pGlobal: PVOID = GlobalLock(hGlobal);
    MultiByteToWideChar(
        codepage,
        0,
        pwStr as LPCSTR,
        cbStr as core::ffi::c_int,
        pGlobal as LPWSTR,
        cbGlobal,
    );
    pGlobal
}

#[no_mangle]
pub unsafe extern "system" fn GetClipboardData(uFormat: UINT) -> HANDLE {
    let mut hData: HANDLE = core::ptr::null_mut();
    let mut pData: PVOID = core::ptr::null_mut();
    let mut cbData: DWORD = 0;
    let mut gcd: GETCLIPBDATA = core::mem::zeroed();

    hData = NtUserGetClipboardData(uFormat, &mut gcd);
    if hData.is_null() {
        return core::ptr::null_mut();
    }

    match uFormat {
        CF_DSPMETAFILEPICT | CF_METAFILEPICT => {
            return GdiCreateLocalMetaFilePict(hData);
        }
        CF_DSPENHMETAFILE | CF_ENHMETAFILE => {
            return GdiCreateLocalEnhMetaFile(hData);
        }
        _ => {}
    }

    if gcd.fGlobalHandle != 0 {
        let hGlobal: HANDLE;

        NtUserCreateLocalMemHandle(hData, core::ptr::null_mut(), 0, &mut cbData);
        hGlobal = GlobalAlloc(GMEM_DDESHARE | GMEM_MOVEABLE, cbData as SIZE_T);
        pData = GlobalLock(hGlobal);
        NtUserCreateLocalMemHandle(hData, pData, cbData, core::ptr::null_mut());
        hData = hGlobal;
    }

    if gcd.uFmtRet != uFormat {
        let mut scd: SETCLIPBDATA = core::mem::zeroed();
        let mut hNewData: HANDLE = core::ptr::null_mut();
        let mut pNewData: PVOID = core::ptr::null_mut();

        match uFormat {
            CF_TEXT => {
                if gcd.uFmtRet == CF_UNICODETEXT {
                    pNewData = IntSynthesizeMultiByte(pData, cbData, (uFormat == CF_OEMTEXT) as BOOL);
                } else {
                    OemToCharBuffA(pData as LPCSTR, pData as LPSTR, cbData);
                }
            }
            CF_OEMTEXT => {
                if gcd.uFmtRet == CF_UNICODETEXT {
                    pNewData = IntSynthesizeMultiByte(pData, cbData, (uFormat == CF_OEMTEXT) as BOOL);
                } else {
                    CharToOemBuffA(pData as LPCSTR, pData as LPSTR, cbData);
                }
            }
            CF_UNICODETEXT => {
                pNewData = IntSynthesizeWideChar(pData, cbData, (gcd.uFmtRet == CF_OEMTEXT) as BOOL);
            }
            _ => {
                FIXME!(b"Format: %u != %u\n\0".as_ptr(), uFormat, gcd.uFmtRet);
            }
        }

        if !pNewData.is_null() {
            hNewData = GlobalHandle(pNewData);
        }

        if !hNewData.is_null() {
            if !pData.is_null() {
                GlobalUnlock(hData);
                GlobalFree(hData);
            }
            hData = hNewData;
            pData = pNewData;
        }

        if !pData.is_null() {
            let hMem: HANDLE;

            scd.fGlobalHandle = 1;
            hMem = NtUserConvertMemHandle(pData, GlobalSize(hData) as DWORD);
            NtUserSetClipboardData(uFormat, hMem, &mut scd);
        } else if !hData.is_null() {
            NtUserSetClipboardData(uFormat, hData, &mut scd);
        }
    }

    if !pData.is_null() {
        GlobalUnlock(hData);
    }

    hData
}

#[no_mangle]
pub unsafe extern "system" fn SetClipboardData(uFormat: UINT, hMem: HANDLE) -> HANDLE {
    let dwSize: DWORD;
    let hGlobal: HANDLE;
    let pMem: LPVOID;
    let mut hRet: HANDLE = core::ptr::null_mut();
    let hTemp: HANDLE;
    let mut scd: SETCLIPBDATA = SETCLIPBDATA {
        fGlobalHandle: 0,
        fIncSerialNumber: 0,
    };

    if hMem.is_null() {
        return NtUserSetClipboardData(uFormat, core::ptr::null_mut(), &mut scd);
    }

    if hMem <= 4 as HANDLE {
        SetLastError(ERROR_INVALID_PARAMETER);
    } else if uFormat == CF_BITMAP || uFormat == CF_DSPBITMAP || uFormat == CF_PALETTE {
        hRet = NtUserSetClipboardData(uFormat, hMem, &mut scd);
    } else if uFormat == CF_DSPMETAFILEPICT || uFormat == CF_METAFILEPICT {
        hTemp = GdiConvertMetaFilePict(hMem);
        hRet = NtUserSetClipboardData(uFormat, hTemp, &mut scd);
        if hRet == hTemp {
            hRet = hMem;
        }
    } else if uFormat == CF_DSPENHMETAFILE || uFormat == CF_ENHMETAFILE {
        hTemp = GdiConvertEnhMetaFile(hMem);
        hRet = NtUserSetClipboardData(uFormat, hTemp, &mut scd);
        if hRet == hTemp {
            hRet = hMem;
        }
    } else {
        pMem = GlobalLock(hMem);
        dwSize = GlobalSize(hMem);

        if !pMem.is_null()
            || uFormat == CF_DIB
            || uFormat == CF_DIBV5
            || uFormat == CF_DSPTEXT
            || uFormat == CF_LOCALE
            || uFormat == CF_OEMTEXT
            || uFormat == CF_TEXT
            || uFormat == CF_UNICODETEXT
        {
            if !pMem.is_null() {
                hGlobal = NtUserConvertMemHandle(pMem, dwSize);

                GlobalUnlock(hMem);

                if !hGlobal.is_null() {
                    scd.fGlobalHandle = 1;
                    hRet = NtUserSetClipboardData(uFormat, hGlobal, &mut scd);
                }

                if hRet == hGlobal {
                    hRet = hMem;
                }
            } else {
                SetLastError(ERROR_INVALID_HANDLE);
            }
        } else {
            hRet = NtUserSetClipboardData(uFormat, hMem, &mut scd);
        }
    }

    if hRet.is_null() {
        ERR!(
            b"SetClipboardData(%u, %p) failed\n\0".as_ptr() as *const i8,
            uFormat,
            hMem,
        );
    }

    hRet
}

#[no_mangle]
pub unsafe extern "system" fn AddClipboardFormatListener(hwnd: HWND) -> BOOL {
    0
}

#[no_mangle]
pub unsafe extern "system" fn RemoveClipboardFormatListener(hwnd: HWND) -> u32 {
    0
}

#[no_mangle]
pub unsafe extern "system" fn GetUpdatedClipboardFormats(
    lpuiFormats: PUINT,
    cFormats: UINT,
    pcFormatsOut: PUINT,
) -> BOOL {
    0
}
