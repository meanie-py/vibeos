#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{PsGetCurrentThreadWin32Thread, ExAllocatePoolWithTag, ExFreePoolWithTag};
// NtCurrentTeb is a FORCEINLINE per-arch TIB read (no kernel symbol) — use the intrinsic.
use ntkrnl_sys::inline::ke::NtCurrentTeb;
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::rtl::RtlCopyMemory;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS};
use ntkrnl_sys::constants::STATUS_INVALID_HANDLE;

// USERTAG_CLIPBOARD: C `'bcsU'` = ('b'<<24)|('c'<<16)|('s'<<8)|'U' = 0x62637355,
// stored little-endian = bytes "Uscb".
const USERTAG_CLIPBOARD: u32 = u32::from_le_bytes(*b"Uscb");

extern "C" {
    fn memcpy(dest: PVOID, src: PVOID, n: usize) -> PVOID;
}

// UserHMGetHandle(obj) = obj->head.h (object handle at offset 0). FORCEINLINE
// macro, no kernel symbol — reimplement inline (matches desktop.rs/winsta.rs).
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HWND {
    *(obj as *const HWND)
}

// SafeProbeForRead/Write — NTAPI (stdcall on i386 → extern "system"), per nm `_SafeProbeForRead@12`.
extern "system" {
    fn SafeProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}

// fastcall (`@Name@N` in nm) — extern "fastcall" on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
    fn DIB_FreeConvertedBitmapInfo(converted: *mut BITMAPINFO, orig: *mut BITMAPINFO, Usage: DWORD);
    // nm: `@co_IntSendMessage@16` / `@co_IntSendMessageNoWait@16` (message.c)
    fn co_IntSendMessage(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> isize;
    fn co_IntSendMessageNoWait(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> isize;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
    fn DIB_BitmapInfoSize(info: *const BITMAPINFO, coloruse: WORD) -> INT;
    fn DIB_ConvertBitmapInfo(bmi: *const BITMAPINFO, Usage: DWORD) -> *mut BITMAPINFO;
    fn DIB_FreeConvertedBitmapInfo(converted: *mut BITMAPINFO, orig: *mut BITMAPINFO, Usage: DWORD);
    fn co_IntSendMessage(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> isize;
    fn co_IntSendMessageNoWait(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> isize;
}

// ---- file-scope preamble (translated) ----
pub const DATA_DELAYED: HANDLE = core::ptr::null_mut();

pub const DATA_SYNTH_USER: HANDLE = 1 as HANDLE;

pub const DATA_SYNTH_KRNL: HANDLE = 2 as HANDLE;

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetWinStaForCbAccess() -> PWINSTATION_OBJECT {
    IntGetWinStaForCbAccessImpl()
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetWinStaForCbAccess() -> PWINSTATION_OBJECT {
    IntGetWinStaForCbAccessImpl()
}

unsafe fn IntGetWinStaForCbAccessImpl() -> PWINSTATION_OBJECT {
    let hWinSta: HWINSTA;
    let mut pWinStaObj: PWINSTATION_OBJECT = core::ptr::null_mut();
    let Status: NTSTATUS;

    hWinSta = UserGetProcessWindowStation();
    Status = IntValidateWindowStationHandle(
        hWinSta,
        1i32 as i8,
        WINSTA_ACCESSCLIPBOARD,
        &mut pWinStaObj,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        ERR!("Cannot open winsta\n");
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }

    pWinStaObj
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetFormatElement(pWinStaObj: PWINSTATION_OBJECT, fmt: UINT) -> PCLIP {
    let mut i: DWORD;

    i = 0;
    while i < (*pWinStaObj).cNumClipFormats {
        if (*(*pWinStaObj).pClipBase.add(i as usize)).fmt == fmt {
            return core::ptr::addr_of_mut!(*(*pWinStaObj).pClipBase.add(i as usize));
        }
        i += 1;
    }

    core::ptr::null_mut()
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetFormatElement(pWinStaObj: PWINSTATION_OBJECT, fmt: UINT) -> PCLIP {
    let mut i: DWORD;

    i = 0;
    while i < (*pWinStaObj).cNumClipFormats {
        if (*(*pWinStaObj).pClipBase.add(i as usize)).fmt == fmt {
            return core::ptr::addr_of_mut!(*(*pWinStaObj).pClipBase.add(i as usize));
        }
        i += 1;
    }

    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntIsFormatAvailable(pWinStaObj: PWINSTATION_OBJECT, fmt: UINT) -> BOOL {
    (IntGetFormatElement(pWinStaObj, fmt) != core::ptr::null_mut()) as BOOL
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntIsFormatAvailable(pWinStaObj: PWINSTATION_OBJECT, fmt: UINT) -> BOOL {
    (IntGetFormatElement(pWinStaObj, fmt) != core::ptr::null_mut()) as BOOL
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntFreeElementData(pElement: PCLIP) {
    IntFreeElementData_impl(pElement)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntFreeElementData(pElement: PCLIP) {
    IntFreeElementData_impl(pElement)
}

unsafe fn IntFreeElementData_impl(pElement: PCLIP) {
    if (*pElement).hData != DATA_DELAYED as HANDLE
        && (*pElement).hData != DATA_SYNTH_USER as HANDLE
        && (*pElement).hData != DATA_SYNTH_KRNL as HANDLE
    {
        if (*pElement).fGlobalHandle != 0 {
            UserDeleteObject((*pElement).hData, TYPE_CLIPDATA);
        } else if (*pElement).fmt == CF_BITMAP
            || (*pElement).fmt == CF_PALETTE
            || (*pElement).fmt == CF_DSPBITMAP
            || (*pElement).fmt == CF_METAFILEPICT
            || (*pElement).fmt == CF_DSPMETAFILEPICT
            || (*pElement).fmt == CF_DSPENHMETAFILE
            || (*pElement).fmt == CF_ENHMETAFILE
        {
            GreSetObjectOwner((*pElement).hData, GDI_OBJ_HMGR_POWNED);
            GreDeleteObject((*pElement).hData);
        }
    }
}

unsafe extern "system" fn IntAddFormatedData(
    pWinStaObj: PWINSTATION_OBJECT,
    fmt: UINT,
    hData: HANDLE,
    fGlobalHandle: BOOLEAN,
    bEnd: BOOL,
) -> PCLIP {
    IntAddFormatedData_impl(pWinStaObj, fmt, hData, fGlobalHandle, bEnd)
}

unsafe fn IntAddFormatedData_impl(
    pWinStaObj: PWINSTATION_OBJECT,
    fmt: UINT,
    hData: HANDLE,
    fGlobalHandle: BOOLEAN,
    bEnd: BOOL,
) -> PCLIP {
    let mut pElement: PCLIP = core::ptr::null_mut();

    if bEnd == 0 {
        pElement = IntGetFormatElement(pWinStaObj, fmt);
    }

    if pElement.is_null() {
        if (*pWinStaObj).cNumClipFormats % 4 == 0 {
            let pNewClip: PCLIP = ExAllocatePoolWithTag(
                PagedPool,
                (((*pWinStaObj).cNumClipFormats + 4) as usize * core::mem::size_of::<CLIP>()) as _,
                USERTAG_CLIPBOARD,
            ) as PCLIP;
            if pNewClip.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                return core::ptr::null_mut();
            }

            memcpy(
                pNewClip as *mut core::ffi::c_void,
                (*pWinStaObj).pClipBase as *mut core::ffi::c_void,
                (*pWinStaObj).cNumClipFormats as usize * core::mem::size_of::<CLIP>(),
            );

            if !(*pWinStaObj).pClipBase.is_null() {
                ExFreePoolWithTag((*pWinStaObj).pClipBase as *mut core::ffi::c_void, USERTAG_CLIPBOARD);
            }

            (*pWinStaObj).pClipBase = pNewClip;
        }

        pElement = core::ptr::addr_of_mut!(
            *(*pWinStaObj).pClipBase.add((*pWinStaObj).cNumClipFormats as usize)
        );
        (*pElement).fmt = fmt;
        (*pWinStaObj).cNumClipFormats += 1;
    } else {
        IntFreeElementData(pElement);
    }

    (*pElement).hData = hData;
    (*pElement).fGlobalHandle = fGlobalHandle as _;

    pElement
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntIsClipboardOpenByMe(pWinSta: PWINSTATION_OBJECT) -> BOOL {
    (!(*pWinSta).ptiClipLock.is_null()
        && (*pWinSta).ptiClipLock as *mut _ == PsGetCurrentThreadWin32Thread() as *mut _) as BOOL
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntIsClipboardOpenByMe(pWinSta: PWINSTATION_OBJECT) -> BOOL {
    (!(*pWinSta).ptiClipLock.is_null()
        && (*pWinSta).ptiClipLock as *mut _ == PsGetCurrentThreadWin32Thread() as *mut _) as BOOL
}

#[repr(C)]
struct IntSynthesizeDib_bmiBuffer {
    bmih: BITMAPINFOHEADER,
    rgbColors: [RGBQUAD; 256],
}

unsafe extern "system" fn IntSynthesizeDib(
    pWinStaObj: PWINSTATION_OBJECT,
    hbm: HBITMAP,
) {
    let hdc: HDC;
    let cjInfoSize: ULONG;
    let cjDataSize: ULONG;
    let pClipboardData: PCLIPBOARDDATA;
    let mut hMem: HANDLE = core::ptr::null_mut();
    let mut iResult: INT;
    let mut bmiBuffer: IntSynthesizeDib_bmiBuffer = core::mem::zeroed();
    let pbmi: PBITMAPINFO = core::ptr::addr_of_mut!(bmiBuffer) as PBITMAPINFO;

    /* Get the display DC */
    hdc = UserGetDCEx(core::ptr::null_mut(), core::ptr::null_mut(), DCX_USESTYLE);
    if hdc.is_null() {
        return;
    }

    /* Get information about the bitmap format */
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(bmiBuffer) as *mut u8,
        0,
        core::mem::size_of::<IntSynthesizeDib_bmiBuffer>(),
    );
    (*pbmi).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as _;
    iResult = GreGetDIBitsInternal(
        hdc,
        hbm,
        0,
        0,
        core::ptr::null_mut(),
        pbmi,
        DIB_RGB_COLORS,
        0,
        core::mem::size_of::<IntSynthesizeDib_bmiBuffer>() as _,
    );

    'cleanup: {
        if iResult == 0 {
            break 'cleanup;
        }

        /* Get the size for a full BITMAPINFO */
        cjInfoSize = DIB_BitmapInfoSize(pbmi, DIB_RGB_COLORS as _) as _;

        /* Calculate the size of the clipboard data, which is a packed DIB */
        cjDataSize = cjInfoSize + (*pbmi).bmiHeader.biSizeImage;

        /* Create the clipboard data */
        pClipboardData = UserCreateObject(
            gHandleTable,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hMem),
            TYPE_CLIPDATA as _,
            (core::mem::size_of::<CLIPBOARDDATA>() as ULONG + cjDataSize) as _,
        ) as PCLIPBOARDDATA;
        if pClipboardData.is_null() {
            break 'cleanup;
        }

        /* Set the data size */
        (*pClipboardData).cbData = cjDataSize;

        /* Copy the BITMAPINFOHEADER */
        core::ptr::copy_nonoverlapping(
            pbmi as *const u8,
            core::ptr::addr_of_mut!((*pClipboardData).Data) as *mut u8,
            core::mem::size_of::<BITMAPINFOHEADER>(),
        );

        /* Get the bitmap bits and the color table */
        iResult = GreGetDIBitsInternal(
            hdc,
            hbm,
            0,
            (*pbmi).bmiHeader.biHeight.abs() as _,
            (core::ptr::addr_of_mut!((*pClipboardData).Data) as LPBYTE).add(cjInfoSize as usize),
            core::ptr::addr_of_mut!((*pClipboardData).Data) as LPBITMAPINFO,
            DIB_RGB_COLORS,
            (*pbmi).bmiHeader.biSizeImage,
            cjInfoSize,
        );
        let _ = iResult;

        /* Add the clipboard data */
        IntAddFormatedData(pWinStaObj, CF_DIB, hMem, 1u8, 1);

        /* Release the extra reference (UserCreateObject added 2 references) */
        UserDereferenceObject(pClipboardData as _);
    }

    UserReleaseDC(core::ptr::null_mut(), hdc, 0u8 as _);
}

unsafe extern "system" fn IntSynthesizeBitmap(pWinStaObj: PWINSTATION_OBJECT, pBmEl: PCLIP) {
    let mut hdc: HDC = core::ptr::null_mut();
    let pBmi: PBITMAPINFO;
    let mut pConvertedBmi: PBITMAPINFO = core::ptr::null_mut();
    let mut hBm: HBITMAP = core::ptr::null_mut();
    let pMemObj: PCLIPBOARDDATA;
    let pDibEl: PCLIP;
    let Offset: ULONG;

    pDibEl = IntGetFormatElement(pWinStaObj, CF_DIB as UINT);
    if (*pDibEl).fGlobalHandle == 0 {
        return;
    }

    pMemObj = UserGetObject(gHandleTable, (*pDibEl).hData, TYPE_CLIPDATA) as PCLIPBOARDDATA;
    if pMemObj.is_null() {
        return;
    }

    pBmi = core::ptr::addr_of_mut!((*pMemObj).Data) as *mut u8 as PBITMAPINFO;

    'cleanup: {
        if ((*pMemObj).cbData as usize) < core::mem::size_of::<DWORD>()
            && ((*pMemObj).cbData) < (*pBmi).bmiHeader.biSize
        {
            break 'cleanup;
        }

        pConvertedBmi = DIB_ConvertBitmapInfo(pBmi, DIB_RGB_COLORS);
        if pConvertedBmi.is_null() {
            break 'cleanup;
        }

        Offset = DIB_BitmapInfoSize(pBmi, DIB_RGB_COLORS as _) as _;

        hdc = UserGetDCEx(core::ptr::null_mut(), core::ptr::null_mut(), DCX_USESTYLE);
        if hdc.is_null() {
            break 'cleanup;
        }

        hBm = GreCreateDIBitmapInternal(
            hdc,
            (*pConvertedBmi).bmiHeader.biWidth,
            (*pConvertedBmi).bmiHeader.biHeight,
            CBM_INIT,
            (core::ptr::addr_of_mut!((*pMemObj).Data) as *mut u8).add(Offset as usize),
            pConvertedBmi,
            DIB_RGB_COLORS,
            0,
            (*pMemObj).cbData - Offset,
            core::ptr::null_mut(),
        );

        if !hBm.is_null() {
            GreSetObjectOwner(hBm as _, GDI_OBJ_HMGR_PUBLIC);
            (*pBmEl).hData = hBm as _;
        }
    }

    if !hdc.is_null() {
        UserReleaseDC(core::ptr::null_mut(), hdc, 0u8 as BOOL);
    }

    if !pConvertedBmi.is_null() {
        DIB_FreeConvertedBitmapInfo(pConvertedBmi, pBmi, (-1i32) as _);
    }
}

unsafe extern "system" fn IntAddSynthesizedFormats(pWinStaObj: PWINSTATION_OBJECT) {
    let bHaveText: BOOL;
    let bHaveUniText: BOOL;
    let bHaveOemText: BOOL;
    let bHaveLocale: BOOL;
    let bHaveBm: BOOL;
    let bHaveDib: BOOL;
    let bHaveMFP: BOOL;
    let bHaveEMF: BOOL;

    bHaveText = IntIsFormatAvailable(pWinStaObj, CF_TEXT);
    bHaveOemText = IntIsFormatAvailable(pWinStaObj, CF_OEMTEXT);
    bHaveUniText = IntIsFormatAvailable(pWinStaObj, CF_UNICODETEXT);
    bHaveLocale = IntIsFormatAvailable(pWinStaObj, CF_LOCALE);
    bHaveBm = IntIsFormatAvailable(pWinStaObj, CF_BITMAP);
    bHaveDib = IntIsFormatAvailable(pWinStaObj, CF_DIB);
    bHaveMFP = IntIsFormatAvailable(pWinStaObj, CF_METAFILEPICT);
    bHaveEMF = IntIsFormatAvailable(pWinStaObj, CF_ENHMETAFILE);

    /* Add CF_LOCALE format if we have CF_TEXT, CF_OEMTEXT or CF_UNICODETEXT */
    if bHaveLocale == 0 && (bHaveText != 0 || bHaveOemText != 0 || bHaveUniText != 0) {
        let pMemObj: PCLIPBOARDDATA;
        let mut hMem: HANDLE = core::ptr::null_mut();

        pMemObj = UserCreateObject(
            gHandleTable,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hMem),
            TYPE_CLIPDATA,
            (core::mem::size_of::<CLIPBOARDDATA>() + core::mem::size_of::<LCID>()) as _,
        ) as PCLIPBOARDDATA;
        if !pMemObj.is_null() {
            (*pMemObj).cbData = core::mem::size_of::<LCID>() as _;
            *((*pMemObj).Data.as_mut_ptr() as *mut LCID) =
                (*(NtCurrentTeb() as *mut _TEB)).CurrentLocale;
            IntAddFormatedData(pWinStaObj, CF_LOCALE, hMem, 1u8, 1);

            /* Release the extra reference (UserCreateObject added 2 references) */
            UserDereferenceObject(pMemObj as _);
        }
    }

    /* Add CF_TEXT. Note: it is synthesized in user32.dll */
    if bHaveText == 0 && (bHaveUniText != 0 || bHaveOemText != 0) {
        IntAddFormatedData(pWinStaObj, CF_TEXT, 1 as HANDLE, 0u8, 1);
    }

    /* Add CF_OEMTEXT. Note: it is synthesized in user32.dll */
    if bHaveOemText == 0 && (bHaveUniText != 0 || bHaveText != 0) {
        IntAddFormatedData(pWinStaObj, CF_OEMTEXT, 1 as HANDLE, 0u8, 1);
    }

    /* Add CF_UNICODETEXT. Note: it is synthesized in user32.dll */
    if bHaveUniText == 0 && (bHaveText != 0 || bHaveOemText != 0) {
        IntAddFormatedData(pWinStaObj, CF_UNICODETEXT, 1 as HANDLE, 0u8, 1);
    }

    /* Add CF_BITMAP. Note: it is synthesized on demand */
    if bHaveBm == 0 && bHaveDib != 0 {
        IntAddFormatedData(pWinStaObj, CF_BITMAP, 2 as HANDLE, 0u8, 1);
    }

    /* Add CF_ENHMETAFILE. Note: it is synthesized in gdi32.dll */
    if bHaveMFP != 0 && bHaveEMF == 0 {
        IntAddFormatedData(pWinStaObj, CF_ENHMETAFILE, 1 as HANDLE, 0u8, 1);
    }

    /* Add CF_METAFILEPICT. Note: it is synthesized in gdi32.dll */
    if bHaveEMF != 0 && bHaveMFP == 0 {
        IntAddFormatedData(pWinStaObj, CF_METAFILEPICT, 1 as HANDLE, 0u8, 1);
    }

    /* Note: We need to render the DIB or DIBV5 format as soon as possible
       because palette information may change */
    if bHaveDib == 0 && bHaveBm != 0 {
        IntSynthesizeDib(pWinStaObj, (*IntGetFormatElement(pWinStaObj, CF_BITMAP)).hData as HBITMAP);
    }
}
#[no_mangle]
pub unsafe extern "system" fn UserEmptyClipboardData(pWinSta: PWINSTATION_OBJECT) {
    let mut i: DWORD = 0;
    while i < (*pWinSta).cNumClipFormats {
        let pElement: PCLIP = core::ptr::addr_of_mut!(*(*pWinSta).pClipBase.add(i as usize));
        IntFreeElementData(pElement);
        i += 1;
    }

    if !(*pWinSta).pClipBase.is_null() {
        ExFreePoolWithTag((*pWinSta).pClipBase as *mut core::ffi::c_void, USERTAG_CLIPBOARD);
    }

    (*pWinSta).pClipBase = core::ptr::null_mut();
    (*pWinSta).cNumClipFormats = 0;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserClipboardRelease(pWindow: PWND) {
    UserClipboardReleaseImpl(pWindow)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserClipboardRelease(pWindow: PWND) {
    UserClipboardReleaseImpl(pWindow)
}

unsafe fn UserClipboardReleaseImpl(pWindow: PWND) {

    let pWinStaObj: PWINSTATION_OBJECT;

    if pWindow.is_null() {
        return;
    }

    pWinStaObj = IntGetWinStaForCbAccess();
    if pWinStaObj.is_null() {
        return;
    }

    co_IntSendMessage(
        UserHMGetHandle((*pWinStaObj).spwndClipOwner) as HWND,
        WM_RENDERALLFORMATS,
        0,
        0,
    );

    /* If the window being destroyed is the current clipboard owner... */
    if pWindow == (*pWinStaObj).spwndClipOwner {
        /* ... make it release the clipboard */
        (*pWinStaObj).spwndClipOwner = core::ptr::null_mut();
    }

    if (*pWinStaObj).fClipboardChanged() != 0 {
        /* Add synthesized formats - they are rendered later */
        IntAddSynthesizedFormats(pWinStaObj);

        /* Notify viewer windows in chain */
        (*pWinStaObj).set_fClipboardChanged(0);
        if !(*pWinStaObj).spwndClipViewer.is_null() {
            TRACE!(
                b"Clipboard: sending WM_DRAWCLIPBOARD to %p\n\0".as_ptr() as PCHAR,
                UserHMGetHandle((*pWinStaObj).spwndClipViewer),
            );
            // For 32-bit applications this message is sent as a notification
            co_IntSendMessageNoWait(
                UserHMGetHandle((*pWinStaObj).spwndClipViewer) as HWND,
                WM_DRAWCLIPBOARD,
                0,
                0,
            );
        }
    }

    ObDereferenceObject(pWinStaObj as PVOID);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserClipboardFreeWindow(pWindow: PWND) {
    UserClipboardFreeWindowImpl(pWindow)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserClipboardFreeWindow(pWindow: PWND) {
    UserClipboardFreeWindowImpl(pWindow)
}

unsafe fn UserClipboardFreeWindowImpl(pWindow: PWND) {
    let pWinStaObj: PWINSTATION_OBJECT;

    pWinStaObj = IntGetWinStaForCbAccess();
    if pWinStaObj.is_null() {
        return;
    }

    if pWindow == (*pWinStaObj).spwndClipOwner {
        (*pWinStaObj).spwndClipOwner = core::ptr::null_mut();
    }

    if pWindow == (*pWinStaObj).spwndClipOpen {
        (*pWinStaObj).spwndClipOpen = core::ptr::null_mut();
        (*pWinStaObj).ptiClipLock = core::ptr::null_mut();
    }

    if pWindow == (*pWinStaObj).spwndClipViewer {
        (*pWinStaObj).spwndClipViewer = core::ptr::null_mut();
    }

    ObDereferenceObject(pWinStaObj as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "system" fn UserEnumClipboardFormats(fmt: UINT) -> UINT {
    let mut Ret: UINT = 0;
    let mut pElement: PCLIP;
    let pWinStaObj: PWINSTATION_OBJECT;

    'cleanup: {
        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        if IntIsClipboardOpenByMe(pWinStaObj) == 0 {
            EngSetLastError(ERROR_CLIPBOARD_NOT_OPEN);
            break 'cleanup;
        }

        if fmt == 0 {
            if !(*pWinStaObj).pClipBase.is_null() {
                Ret = (*(*pWinStaObj).pClipBase.add(0)).fmt;
            }
        } else {
            pElement = IntGetFormatElement(pWinStaObj, fmt);
            if !pElement.is_null() {
                pElement = pElement.add(1);
                if pElement < (*pWinStaObj).pClipBase.add((*pWinStaObj).cNumClipFormats as usize) {
                    Ret = (*pElement).fmt;
                }
            }
        }
    }

    if !pWinStaObj.is_null() {
        ObDereferenceObject(pWinStaObj as *mut _);
    }

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn UserOpenClipboard(hWnd: HWND) -> BOOL {
    let mut pWindow: PWND = core::ptr::null_mut();
    let mut bRet: BOOL = 0;
    let mut pWinStaObj: PWINSTATION_OBJECT = core::ptr::null_mut();

    'cleanup: {
        if !hWnd.is_null() {
            pWindow = UserGetWindowObject(hWnd);
            if pWindow.is_null() {
                break 'cleanup;
            }
        }

        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        /* Check if we already opened the clipboard */
        if pWindow == (*pWinStaObj).spwndClipOpen && IntIsClipboardOpenByMe(pWinStaObj) != 0 {
            bRet = 1;
            break 'cleanup;
        }

        /* If the clipboard was already opened by somebody else, bail out */
        if pWindow != (*pWinStaObj).spwndClipOpen && !(*pWinStaObj).ptiClipLock.is_null() {
            EngSetLastError(ERROR_ACCESS_DENIED);
            break 'cleanup;
        }

        /* Open the clipboard */
        (*pWinStaObj).spwndClipOpen = pWindow;
        (*pWinStaObj).ptiClipLock = PsGetCurrentThreadWin32Thread() as *mut _;
        bRet = 1;
    }

    if !pWinStaObj.is_null() {
        ObDereferenceObject(pWinStaObj as *mut _);
    }

    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserOpenClipboard(hWnd: HWND, Unknown1: DWORD) -> BOOL {
    let bRet: BOOL;

    UserEnterExclusive();
    bRet = UserOpenClipboard(hWnd);
    UserLeave();

    bRet
}
#[no_mangle]
pub unsafe extern "system" fn UserCloseClipboard() -> BOOL {
    let mut bRet: BOOL = FALSE as BOOL;
    let pWinStaObj: PWINSTATION_OBJECT;

    'cleanup: {
        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        /* Check if the clipboard has been opened */
        if IntIsClipboardOpenByMe(pWinStaObj) == 0 {
            EngSetLastError(ERROR_CLIPBOARD_NOT_OPEN);
            break 'cleanup;
        }

        /* Clipboard is no longer open */
        (*pWinStaObj).spwndClipOpen = core::ptr::null_mut();
        (*pWinStaObj).ptiClipLock = core::ptr::null_mut();
        bRet = TRUE as BOOL;

        if (*pWinStaObj).fClipboardChanged() != 0 {
            /* Add synthesized formats - they are rendered later */
            IntAddSynthesizedFormats(pWinStaObj);

            /* Notify viewer windows in chain */
            (*pWinStaObj).set_fClipboardChanged(FALSE as _);
            if !(*pWinStaObj).spwndClipViewer.is_null() {
                TRACE!(
                    b"Clipboard: sending WM_DRAWCLIPBOARD to %p\n\0".as_ptr() as PCHAR,
                    UserHMGetHandle((*pWinStaObj).spwndClipViewer),
                );
                co_IntSendMessageNoWait(
                    UserHMGetHandle((*pWinStaObj).spwndClipViewer),
                    WM_DRAWCLIPBOARD,
                    0,
                    0,
                );
            }
        }
    }

    if !pWinStaObj.is_null() {
        ObDereferenceObject(pWinStaObj as *mut _);
    }

    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCloseClipboard() -> BOOL {
    let bRet: BOOL;

    UserEnterExclusive();
    bRet = UserCloseClipboard();
    UserLeave();

    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetOpenClipboardWindow() -> HWND {
    let mut hWnd: HWND = core::ptr::null_mut();
    let pWinStaObj: PWINSTATION_OBJECT;

    UserEnterShared();

    'cleanup: {
        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        if !(*pWinStaObj).spwndClipOpen.is_null() {
            hWnd = UserHMGetHandle((*pWinStaObj).spwndClipOpen as *mut _);
        }

        ObDereferenceObject(pWinStaObj as *mut _);
    }

    UserLeave();

    hWnd
}
#[no_mangle]
pub unsafe extern "system" fn NtUserChangeClipboardChain(hWndRemove: HWND, hWndNewNext: HWND) -> BOOL {
    let mut bRet: BOOL = 0;
    let pWindowRemove: PWND;
    let pWinStaObj: PWINSTATION_OBJECT;

    TRACE!(b"NtUserChangeClipboardChain(%p, %p)\n\0".as_ptr() as *const i8, hWndRemove, hWndNewNext);

    UserEnterExclusive();

    'cleanup: {
        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        pWindowRemove = UserGetWindowObject(hWndRemove);

        if !pWindowRemove.is_null() && !(*pWinStaObj).spwndClipViewer.is_null() {
            if pWindowRemove == (*pWinStaObj).spwndClipViewer {
                (*pWinStaObj).spwndClipViewer = UserGetWindowObject(hWndNewNext);
            }

            if !(*pWinStaObj).spwndClipViewer.is_null() {
                bRet = co_IntSendMessage(UserHMGetHandle((*pWinStaObj).spwndClipViewer), WM_CHANGECBCHAIN, hWndRemove as WPARAM, hWndNewNext as LPARAM) as BOOL;
            }
        }

        ObDereferenceObject(pWinStaObj as *mut core::ffi::c_void);
    }

    UserLeave();

    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCountClipboardFormats() -> DWORD {
    let mut cFormats: DWORD = 0;
    let pWinStaObj: PWINSTATION_OBJECT;

    UserEnterShared();

    'cleanup: {
        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        cFormats = (*pWinStaObj).cNumClipFormats;

        ObDereferenceObject(pWinStaObj as *mut core::ffi::c_void);
    }

    UserLeave();

    cFormats
}
#[no_mangle]
pub unsafe extern "system" fn UserEmptyClipboard() -> BOOL {
    let mut bRet: BOOL = 0;
    let pWinStaObj: PWINSTATION_OBJECT;

    pWinStaObj = IntGetWinStaForCbAccess();
    if pWinStaObj.is_null() {
        return 0;
    }

    'cleanup: {
        if IntIsClipboardOpenByMe(pWinStaObj) == 0 {
            EngSetLastError(ERROR_CLIPBOARD_NOT_OPEN);
            break 'cleanup;
        }

        UserEmptyClipboardData(pWinStaObj);

        if !(*pWinStaObj).spwndClipOwner.is_null() {
            TRACE!(
                b"Clipboard: WM_DESTROYCLIPBOARD to %p\n\0".as_ptr() as *const i8,
                UserHMGetHandle((*pWinStaObj).spwndClipOwner),
            );
            co_IntSendMessage(
                UserHMGetHandle((*pWinStaObj).spwndClipOwner),
                WM_DESTROYCLIPBOARD,
                0,
                0,
            );
        }

        (*pWinStaObj).spwndClipOwner = (*pWinStaObj).spwndClipOpen;

        (*pWinStaObj).iClipSerialNumber += 1;
        (*pWinStaObj).iClipSequenceNumber += 1;
        (*pWinStaObj).set_fClipboardChanged(1);
        (*pWinStaObj).set_fInDelayedRendering(0);

        bRet = 1;
    }

    if !pWinStaObj.is_null() {
        ObDereferenceObject(pWinStaObj as *mut core::ffi::c_void);
    }

    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserEmptyClipboard() -> BOOL {
    let bRet: BOOL;

    TRACE!(b"NtUserEmptyClipboard()\n\0".as_ptr() as PCHAR);

    UserEnterExclusive();
    bRet = UserEmptyClipboard();
    UserLeave();

    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetClipboardFormatName(
    fmt: UINT,
    lpszFormatName: LPWSTR,
    cchMaxCount: INT,
) -> INT {
    let mut iRet: INT = 0;

    UserEnterShared();

    'cleanup: {
        if fmt < 0xc000 || fmt > 0xffff {
            break 'cleanup;
        }

        if cchMaxCount < 1 || lpszFormatName.is_null() {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'cleanup;
        }

        iRet = IntGetAtomName(
            fmt as RTL_ATOM,
            lpszFormatName,
            (cchMaxCount as u32) * core::mem::size_of::<u16>() as u32,
        ) as INT;
        iRet /= core::mem::size_of::<u16>() as INT;
    }

    UserLeave();

    iRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetClipboardOwner() -> HWND {
    let mut hWnd: HWND = core::ptr::null_mut();
    let pWinStaObj: PWINSTATION_OBJECT;

    UserEnterShared();

    'cleanup: {
        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        if !(*pWinStaObj).spwndClipOwner.is_null() {
            hWnd = UserHMGetHandle((*pWinStaObj).spwndClipOwner);
        }

        ObDereferenceObject(pWinStaObj as *mut core::ffi::c_void);
    }

    UserLeave();

    hWnd
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetClipboardViewer() -> HWND {
    let mut hWnd: HWND = core::ptr::null_mut();
    let pWinStaObj: PWINSTATION_OBJECT;

    UserEnterShared();

    'cleanup: {
        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        if !(*pWinStaObj).spwndClipViewer.is_null() {
            hWnd = UserHMGetHandle((*pWinStaObj).spwndClipViewer);
        }

        ObDereferenceObject(pWinStaObj as *mut core::ffi::c_void);
    }

    UserLeave();

    hWnd
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetPriorityClipboardFormat(
    paFormatPriorityList: *mut UINT,
    cFormats: INT,
) -> INT {
    let mut i: INT;
    let mut iRet: INT = 0;
    let pWinStaObj: PWINSTATION_OBJECT;

    UserEnterShared();

    'cleanup: {
        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        if (*pWinStaObj).pClipBase.is_null() {
            iRet = 0;
        } else {
            iRet = -1;

            i = 0;
            while i < cFormats {
                if IntIsFormatAvailable(pWinStaObj, *paFormatPriorityList.offset(i as isize)) != 0 {
                    iRet = *paFormatPriorityList.offset(i as isize) as INT;
                    break;
                }
                i += 1;
            }
        }

        ObDereferenceObject(pWinStaObj as *mut _);
    }

    UserLeave();

    iRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserIsClipboardFormatAvailable(fmt: UINT) -> BOOL {
    let mut bRet: BOOL = 0;
    let pWinStaObj: PWINSTATION_OBJECT;

    UserEnterShared();

    'cleanup: {
        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        if IntIsFormatAvailable(pWinStaObj, fmt) != 0 {
            bRet = 1;
        }

        ObDereferenceObject(pWinStaObj as *mut _);
    }

    UserLeave();

    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetClipboardData(fmt: UINT, pgcd: PGETCLIPBDATA) -> HANDLE {
    let mut hRet: HANDLE = core::ptr::null_mut();
    let mut pElement: PCLIP;
    let mut pWinStaObj: PWINSTATION_OBJECT;
    let mut uSourceFmt: UINT = fmt;

    UserEnterShared();

    'cleanup: {
        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        if IntIsClipboardOpenByMe(pWinStaObj) == 0 {
            EngSetLastError(ERROR_CLIPBOARD_NOT_OPEN);
            break 'cleanup;
        }

        pElement = IntGetFormatElement(pWinStaObj, fmt);
        if pElement.is_null() {
            break 'cleanup;
        }

        if (*pElement).hData == 1 as HANDLE || (*pElement).hData == 2 as HANDLE {
            match fmt {
                x if x == CF_UNICODETEXT as UINT
                    || x == CF_TEXT as UINT
                    || x == CF_OEMTEXT as UINT =>
                {
                    uSourceFmt = CF_UNICODETEXT as UINT;
                    pElement = IntGetFormatElement(pWinStaObj, uSourceFmt);
                    if (*pElement).hData == 1 as HANDLE || (*pElement).hData == 2 as HANDLE {
                        uSourceFmt = CF_TEXT as UINT;
                        pElement = IntGetFormatElement(pWinStaObj, uSourceFmt);
                    }
                    if (*pElement).hData == 1 as HANDLE || (*pElement).hData == 2 as HANDLE {
                        uSourceFmt = CF_OEMTEXT as UINT;
                        pElement = IntGetFormatElement(pWinStaObj, uSourceFmt);
                    }
                }
                x if x == CF_BITMAP as UINT => {
                    IntSynthesizeBitmap(pWinStaObj, pElement);
                }
                x if x == CF_METAFILEPICT as UINT => {
                    uSourceFmt = CF_ENHMETAFILE as UINT;
                    pElement = IntGetFormatElement(pWinStaObj, uSourceFmt);
                }
                x if x == CF_ENHMETAFILE as UINT => {
                    uSourceFmt = CF_METAFILEPICT as UINT;
                    pElement = IntGetFormatElement(pWinStaObj, uSourceFmt);
                }
                _ => {}
            }
        }

        if !pElement.is_null()
            && (*pElement).hData == 0 as HANDLE
            && !(*pWinStaObj).spwndClipOwner.is_null()
        {
            (*pWinStaObj).set_fInDelayedRendering(1);
            co_IntSendMessage(
                UserHMGetHandle((*pWinStaObj).spwndClipOwner),
                WM_RENDERFORMAT,
                uSourceFmt as WPARAM,
                0,
            );
            (*pWinStaObj).set_fInDelayedRendering(0);

            pElement = IntGetFormatElement(pWinStaObj, uSourceFmt);
        }

        if pElement.is_null() || (*pElement).hData == 0 as HANDLE {
            break 'cleanup;
        }

        // _SEH2_TRY body (handler is dead code)
        let _ = core::ptr::read_volatile(pgcd as *const u8);
        (*pgcd).uFmtRet = (*pElement).fmt;
        (*pgcd).fGlobalHandle = (*pElement).fGlobalHandle;

        if fmt == CF_TEXT as UINT {
            let pLocaleEl: PCLIP = IntGetFormatElement(pWinStaObj, CF_LOCALE as UINT);
            if !pLocaleEl.is_null() && (*pLocaleEl).hData != 0 as HANDLE {
                (*pgcd).__bindgen_anon_1.hLocale = (*pLocaleEl).hData;
            }
        } else if fmt == CF_BITMAP as UINT {
            let pPaletteEl: PCLIP = IntGetFormatElement(pWinStaObj, CF_PALETTE as UINT);
            if !pPaletteEl.is_null() && (*pPaletteEl).hData != 0 as HANDLE {
                (*pgcd).__bindgen_anon_1.hPalette = (*pPaletteEl).hData;
            }
        }

        hRet = (*pElement).hData;
    }

    if !pWinStaObj.is_null() {
        ObDereferenceObject(pWinStaObj as *mut _);
    }

    UserLeave();

    hRet
}
#[no_mangle]
pub unsafe extern "system" fn UserSetClipboardData(
    fmt: UINT,
    hData: HANDLE,
    scd: PSETCLIPBDATA,
) -> HANDLE {
    let mut hRet: HANDLE = core::ptr::null_mut();
    let pWinStaObj: PWINSTATION_OBJECT;

    'cleanup: {
        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        if fmt == 0 || (*pWinStaObj).ptiClipLock.is_null() {
            EngSetLastError(ERROR_CLIPBOARD_NOT_OPEN);
            break 'cleanup;
        }

        if (*scd).fIncSerialNumber != 0 {
            (*pWinStaObj).iClipSerialNumber += 1;
        }

        if !hData.is_null() {
            if fmt == CF_BITMAP {
                GreSetObjectOwner(hData, GDI_OBJ_HMGR_PUBLIC);
            }

            IntAddFormatedData(
                pWinStaObj,
                fmt,
                hData,
                (*scd).fGlobalHandle as BOOLEAN,
                0,
            );

            if (*scd).fIncSerialNumber != 0 {
                (*pWinStaObj).iClipSequenceNumber += 1;
            }

            (*pWinStaObj).set_fClipboardChanged(1);
        } else {
            IntAddFormatedData(pWinStaObj, fmt, core::ptr::null_mut(), 0, 0);
        }

        hRet = hData;
    }

    if !pWinStaObj.is_null() {
        ObDereferenceObject(pWinStaObj as *mut _);
    }

    hRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetClipboardData(
    fmt: UINT,
    hData: HANDLE,
    pUnsafeScd: PSETCLIPBDATA,
) -> HANDLE {
    let mut scd: SETCLIPBDATA = core::mem::zeroed();
    let hRet: HANDLE;

    TRACE!(
        b"NtUserSetClipboardData(%x %p %p)\n\0".as_ptr() as *const i8,
        fmt,
        hData,
        pUnsafeScd,
    );

    {
        let __SehStatus = SafeProbeForRead(
            pUnsafeScd as *mut core::ffi::c_void,
            core::mem::size_of::<SETCLIPBDATA>() as _,
            1,
        );
        if !NT_SUCCESS(__SehStatus) {
            SetLastNtError(__SehStatus);
            return core::ptr::null_mut();
        }
    }
    RtlCopyMemory(
        core::ptr::addr_of_mut!(scd) as *mut core::ffi::c_void,
        pUnsafeScd as *const core::ffi::c_void,
        core::mem::size_of::<SETCLIPBDATA>(),
    );

    UserEnterExclusive();

    hRet = UserSetClipboardData(fmt, hData, core::ptr::addr_of_mut!(scd));

    UserLeave();

    hRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetClipboardViewer(hWndNewViewer: HWND) -> HWND {
    let mut hWndNext: HWND = core::ptr::null_mut();
    let mut pWinStaObj: PWINSTATION_OBJECT;
    let pWindow: PWND;

    UserEnterExclusive();

    'cleanup: {
        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        pWindow = UserGetWindowObject(hWndNewViewer);
        if pWindow.is_null() {
            EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
            break 'cleanup;
        }

        if !(*pWinStaObj).spwndClipViewer.is_null() {
            hWndNext = UserHMGetHandle((*pWinStaObj).spwndClipViewer);
        }

        (*pWinStaObj).spwndClipViewer = pWindow;

        (*pWinStaObj).set_fClipboardChanged(0);
        if !(*pWinStaObj).spwndClipViewer.is_null() {
            TRACE!(
                b"Clipboard: sending WM_DRAWCLIPBOARD to %p\n\0".as_ptr() as *const i8,
                UserHMGetHandle((*pWinStaObj).spwndClipViewer),
            );
            co_IntSendMessageNoWait(
                UserHMGetHandle((*pWinStaObj).spwndClipViewer),
                WM_DRAWCLIPBOARD,
                0,
                0,
            );
        }
    }

    if !pWinStaObj.is_null() {
        ObDereferenceObject(pWinStaObj as *mut _);
    }

    UserLeave();

    hWndNext
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetClipboardSequenceNumber() -> DWORD {
    let mut dwRet: DWORD = 0;
    let pWinStaObj: PWINSTATION_OBJECT;

    UserEnterShared();

    'cleanup: {
        pWinStaObj = IntGetWinStaForCbAccess();
        if pWinStaObj.is_null() {
            break 'cleanup;
        }

        dwRet = (*pWinStaObj).iClipSequenceNumber as DWORD;

        ObDereferenceObject(pWinStaObj as *mut _);
    }

    UserLeave();

    dwRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserConvertMemHandle(
    pData: PVOID,
    cbData: DWORD,
) -> HANDLE {
    let mut hMem: HANDLE = core::ptr::null_mut();
    let pMemObj: PCLIPBOARDDATA;
    let Status: NTSTATUS;

    UserEnterExclusive();

    'cleanup: {
        /* Create Clipboard data object */
        pMemObj = UserCreateObject(
            gHandleTable,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut hMem,
            TYPE_CLIPDATA,
            (core::mem::size_of::<CLIPBOARDDATA>() as DWORD + cbData) as _,
        ) as PCLIPBOARDDATA;
        if pMemObj.is_null() {
            break 'cleanup;
        }

        (*pMemObj).cbData = cbData;

        /* Copy data */
        // Signal the failure through Status, NOT by nulling pMemObj: the
        // UserDereferenceObject below runs unconditionally and dereferenced the null
        // we had just written -- a BSOD any user-mode caller could trigger with a bad
        // pData. Upstream feb9febaae3 (#7878) made the same split.
        Status = SafeProbeForRead(pData, cbData as _, 1);
        if NT_SUCCESS(Status) {
            memcpy(
                core::ptr::addr_of_mut!((*pMemObj).Data) as *mut _,
                pData,
                cbData as _,
            );
        }

        /* Release the extra reference (UserCreateObject added 2 references) */
        UserDereferenceObject(pMemObj as _);

        /* If we failed to copy data, remove handle */
        if !NT_SUCCESS(Status) {
            UserDeleteObject(hMem, TYPE_CLIPDATA);
            hMem = core::ptr::null_mut();
        }
    }

    UserLeave();

    hMem
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserCreateLocalMemHandle(
    hMem: HANDLE,
    pData: PVOID,
    mut cbData: DWORD,
    pcbData: *mut DWORD,
) -> NTSTATUS {

    let pMemObj: PCLIPBOARDDATA;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    UserEnterShared();

    'cleanup: {
        /* Get Clipboard data object */
        pMemObj = UserGetObject(gHandleTable, hMem, TYPE_CLIPDATA) as PCLIPBOARDDATA;
        if pMemObj.is_null() {
            Status = STATUS_INVALID_HANDLE;
            break 'cleanup;
        }

        /* Don't overrun */
        if cbData > (*pMemObj).cbData {
            cbData = (*pMemObj).cbData;
        }

        /* Copy data to usermode */
        Status = SafeProbeForWrite(
            pcbData as PVOID,
            core::mem::size_of::<DWORD>() as ULONG,
            1,
        );
        if NT_SUCCESS(Status) {
            Status = SafeProbeForWrite(pData, cbData, 1);
        }
        if NT_SUCCESS(Status) {
            if !pcbData.is_null() {
                core::ptr::write_volatile(pcbData, (*pMemObj).cbData);
            }

            memcpy(
                pData,
                core::ptr::addr_of_mut!((*pMemObj).Data) as PVOID,
                cbData as usize,
            );
        }
    }

    UserLeave();

    Status
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserCreateLocalMemHandle(
    hMem: HANDLE,
    pData: PVOID,
    mut cbData: DWORD,
    pcbData: *mut DWORD,
) -> NTSTATUS {

    let pMemObj: PCLIPBOARDDATA;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    UserEnterShared();

    'cleanup: {
        /* Get Clipboard data object */
        pMemObj = UserGetObject(gHandleTable, hMem, TYPE_CLIPDATA) as PCLIPBOARDDATA;
        if pMemObj.is_null() {
            Status = STATUS_INVALID_HANDLE;
            break 'cleanup;
        }

        /* Don't overrun */
        if cbData > (*pMemObj).cbData {
            cbData = (*pMemObj).cbData;
        }

        /* Copy data to usermode */
        Status = SafeProbeForWrite(
            pcbData as PVOID,
            core::mem::size_of::<DWORD>() as _,
            1,
        );
        if NT_SUCCESS(Status) {
            Status = SafeProbeForWrite(pData, cbData as _, 1);
        }
        if NT_SUCCESS(Status) {
            if !pcbData.is_null() {
                core::ptr::write_volatile(pcbData, (*pMemObj).cbData);
            }

            memcpy(
                pData,
                core::ptr::addr_of_mut!((*pMemObj).Data) as PVOID,
                cbData as usize,
            );
        }
    }

    UserLeave();

    Status
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn DIB_BitmapInfoSize(info: *const BITMAPINFO, coloruse: WORD) -> INT;
    fn DIB_ConvertBitmapInfo(bmi: *const BITMAPINFO, Usage: DWORD) -> *mut BITMAPINFO;
}
