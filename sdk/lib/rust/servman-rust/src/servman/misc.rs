// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_78A49CBB7CDB: [u16; 7] = [0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0021, 0x0000];
pub static RUSTCONV_WSTR_E872F95E8F59: [u16; 6] = [0x004Eu16, 0x006F, 0x0074, 0x0065, 0x0021, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const BUFFERSIZE: usize = 512;

unsafe extern "C" fn LengthOfStrResource(mut hInst: HINSTANCE, mut uID: UINT) -> INT {
    let mut hrSrc: HRSRC = ::core::ptr::null_mut();
    let mut hRes: HGLOBAL = ::core::ptr::null_mut();
    let mut lpName: LPWSTR = ::core::ptr::null_mut();
    let mut lpStr: LPWSTR = ::core::ptr::null_mut();
    if hInst.is_null() {
        return -(1 as INT);
    }
    /* There are always blocks of 16 strings */
    lpName = MAKEINTRESOURCE!((uID >> 4) + 1) as LPWSTR;
    /* Find the string table block */
    if {
        hrSrc = FindResourceW(
            hInst as HMODULE,
            lpName as LPCWSTR,
            (RT_STRING as usize as LPWSTR) as LPCWSTR,
        );
        !hrSrc.is_null()
    } && {
        hRes = LoadResource(hInst, hrSrc);
        !hRes.is_null()
    } && {
        lpStr = LockResource(hRes) as *mut WCHAR as LPWSTR;
        !lpStr.is_null()
    } {
        let mut x: UINT = 0;
        /* Find the string we're looking for */
        uID &= 0xF as UINT; /* position in the block, same as % 16 */
        x = 0 as UINT;
        while x < uID {
            lpStr = lpStr.offset((*lpStr as isize) + (1 as isize));
            x = x.wrapping_add(1);
        }
        /* Found the string */
        return *lpStr as INT;
    }
    return -(1 as INT);
}
#[no_mangle]
pub unsafe extern "C" fn AllocAndLoadString(lpTarget: *mut LPWSTR, hInst: HINSTANCE, uID: UINT) -> INT {
    let mut ln: INT;
    ln = LengthOfStrResource(hInst, uID);
    let fresh = ln;
    ln = ln + 1;
    if fresh > 0 {
        *lpTarget = LocalAlloc(
            LMEM_FIXED as UINT,
            (ln as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as LPWSTR;
        if !(*lpTarget).is_null() {
            let mut Ret: INT;
            Ret = LoadStringW(hInst, uID, *lpTarget, ln) as INT;
            if Ret == 0 {
                LocalFree(*lpTarget as HLOCAL);
            }
            return Ret;
        }
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn LoadAndFormatString_v(
    hInst: HINSTANCE,
    uID: UINT,
    lpTarget: *mut LPWSTR,
    ap_list: *mut ::core::ffi::c_void,
) -> DWORD {
    let mut Ret: DWORD = 0;
    let mut lpFormat: LPWSTR = core::ptr::null_mut();
    let mut lArgs: va_list;

    if AllocAndLoadString(&raw mut lpFormat, hInst, uID) > 0 {
        lArgs = *(ap_list as *mut va_list);
        /* let's use Format to format it because it has the ability to allocate
           memory automatically */
        Ret = FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_STRING,
            lpFormat as LPCVOID,
            0,
            0,
            lpTarget as LPWSTR,
            0,
            (&raw mut lArgs as *mut core::ffi::c_void).cast(),
        );

        LocalFree(lpFormat as HLOCAL);
    }

    return Ret;
}


#[no_mangle]
pub unsafe extern "C" fn StatusBarLoadAndFormatString_v(
    mut hStatusBar: HWND,
    mut PartId: INT,
    mut hInstance_0: HINSTANCE,
    mut uID: UINT,
    ap_list: *mut ::core::ffi::c_void,
) -> BOOL {
    let mut Ret: BOOL = (FALSE) as i32;
    let mut lpFormat: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut lpStr: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut lArgs: varargs_rust::VaArgs;
    if AllocAndLoadString(&raw mut lpFormat, hInstance_0, uID) > 0 {
        lArgs = varargs_rust::VaArgs::from_ptr(ap_list);
        Ret = FormatMessageW(
            (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_STRING) as DWORD,
            lpFormat as LPCVOID,
            0 as DWORD,
            0 as DWORD,
            &raw mut lpStr as *mut ::core::ffi::c_void as LPWSTR,
            0 as DWORD,
            (&raw mut lArgs).cast(),
        ) as BOOL;
        if !lpStr.is_null() {
            Ret = SendMessageW(
                hStatusBar,
                SB_SETTEXT as UINT,
                PartId as WPARAM,
                lpStr as LPARAM,
            ) as BOOL;
            LocalFree(lpStr as HLOCAL);
        }
        LocalFree(lpFormat as HLOCAL);
    }
    return Ret;
}


#[no_mangle]
pub unsafe extern "C" fn StatusBarLoadString(hStatusBar: HWND, PartId: INT, hInst: HINSTANCE, uID: UINT) -> BOOL {
    let mut Ret: BOOL = (FALSE) as i32;
    let mut lpStr: LPWSTR = core::ptr::null_mut();

    if AllocAndLoadString(core::ptr::addr_of_mut!(lpStr), hInst, uID) > 0 {
        Ret = SendMessageW(
            hStatusBar,
            SB_SETTEXT as UINT,
            PartId as WPARAM,
            lpStr as LPARAM,
        ) as BOOL;
        LocalFree(lpStr as HLOCAL);
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn GetTextFromEdit(
    mut lpString: LPWSTR,
    mut hDlg: HWND,
    mut Res: UINT,
) -> INT {
    let mut len: INT = GetWindowTextLengthW(GetDlgItem(hDlg, Res as ::core::ffi::c_int)) as INT;
    if len > 0 {
        GetDlgItemTextW(
            hDlg,
            Res as ::core::ffi::c_int,
            lpString,
            len + 1,
        );
    } else {
        lpString = core::ptr::null_mut();
    }

    return len;
}
#[no_mangle]
pub unsafe extern "C" fn GetError() {
    let mut lpMsgBuf: LPWSTR = core::ptr::null_mut();

    FormatMessageW((FORMAT_MESSAGE_ALLOCATE_BUFFER |
                   FORMAT_MESSAGE_FROM_SYSTEM |
                   FORMAT_MESSAGE_IGNORE_INSERTS) as DWORD,
                   core::ptr::null(),
                   GetLastError(),
                   MAKELANGID!(LANG_NEUTRAL, SUBLANG_DEFAULT) as DWORD,
                   core::ptr::addr_of_mut!(lpMsgBuf) as *mut core::ffi::c_void as LPWSTR,
                   0 as DWORD,
                   core::ptr::null_mut() );

    MessageBoxW(core::ptr::null_mut(), lpMsgBuf, RUSTCONV_WSTR_78A49CBB7CDB.as_ptr() as LPCWSTR, (MB_OK | MB_ICONERROR) as UINT);

    LocalFree(lpMsgBuf as HLOCAL);
}
#[no_mangle]
pub unsafe extern "C" fn DisplayString(Msg: PWCHAR) {
    MessageBoxW(
        core::ptr::null_mut(),
        Msg as *const u16,
        RUSTCONV_WSTR_E872F95E8F59.as_ptr(),
        (MB_ICONEXCLAMATION | MB_OK) as UINT,
    );
}
#[no_mangle]
pub unsafe extern "C" fn InitImageList(mut StartResource: UINT, mut EndResource: UINT, mut Width: UINT, mut Height: UINT, mut type_0: ULONG) -> HIMAGELIST {
    let mut hImage: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut himl: HIMAGELIST = core::ptr::null_mut::<_IMAGELIST>();
    let mut i: UINT = 0;
    let mut ret: INT = 0;

    /* Create the toolbar icon image list */
    himl = ImageList_Create(
        Width as core::ffi::c_int,
        Height as core::ffi::c_int,
        (ILC_MASK | ILC_COLOR32) as UINT,
        EndResource.wrapping_sub(StartResource) as core::ffi::c_int,
        0,
    );
    if himl.is_null() {
        return core::ptr::null_mut::<_IMAGELIST>();
    }

    ret = 0 as INT;
    i = StartResource;
    while i <= EndResource && ret != -(1 as INT) {
        hImage = LoadImageW(
            hInstance,
            MAKEINTRESOURCEW!(i),
            type_0 as UINT,
            Width as core::ffi::c_int,
            Height as core::ffi::c_int,
            LR_LOADTRANSPARENT as UINT,
        );
        if hImage.is_null() {
            ret = -(1 as INT);
            break;
        }

        if type_0 == (IMAGE_BITMAP as ULONG) {
            ret = ImageList_AddMasked(
                himl,
                hImage as HBITMAP,
                RGB!(255, 0, 128),
            ) as INT;
        } else if type_0 == (IMAGE_ICON as ULONG) {
            ret = ImageList_AddIcon!(himl, hImage as HICON) as INT;
        }

        DeleteObject(hImage as HGDIOBJ);
        i = i.wrapping_add(1);
    }

    if ret == -(1 as INT) {
        ImageList_Destroy(himl);
        himl = core::ptr::null_mut::<_IMAGELIST>();
    }

    return himl;
}
#[no_mangle]
pub unsafe extern "C" fn ResourceMessageBox(hInst: HINSTANCE, hwnd: HWND, uType: UINT, uCaptionId: UINT, uMessageId: UINT) {
    let mut szErrorText: [WCHAR; BUFFERSIZE] = [0; BUFFERSIZE];
    let mut szErrorCaption: [WCHAR; BUFFERSIZE] = [0; BUFFERSIZE];

    LoadStringW(
        hInst,
        uMessageId,
        core::ptr::addr_of_mut!(szErrorText) as *mut WCHAR as LPWSTR,
        (core::mem::size_of::<[WCHAR; BUFFERSIZE]>() / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
    );
    LoadStringW(
        hInst,
        uCaptionId,
        core::ptr::addr_of_mut!(szErrorCaption) as *mut WCHAR as LPWSTR,
        (core::mem::size_of::<[WCHAR; BUFFERSIZE]>() / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
    );

    MessageBoxW(
        hwnd,
        core::ptr::addr_of_mut!(szErrorText) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(szErrorCaption) as *mut WCHAR as LPCWSTR,
        uType,
    );
}
