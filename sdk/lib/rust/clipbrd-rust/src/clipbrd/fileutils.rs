use crate::clipbrd::winutils::MessageBoxRes;
use crate::clipbrd::cliputils::RetrieveClipboardFormatName;
use crate::clipbrd::winutils::ShowLastWin32Error;
use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn ClipboardReadMemoryBlock(
    mut hFile: HANDLE,
    mut dwOffset: DWORD,
    mut dwLength: DWORD,
) -> HGLOBAL {
    let mut hData: HGLOBAL = core::ptr::null_mut::<core::ffi::c_void>();
    let mut lpData: LPVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut dwBytesRead: DWORD = 0;
    hData = GlobalAlloc(GHND as UINT, dwLength as SIZE_T);
    if hData.is_null() {
        return core::ptr::null_mut();
    }
    lpData = GlobalLock(hData);
    if lpData.is_null() {
        GlobalFree(hData);
        return core::ptr::null_mut();
    }
    if SetFilePointer(
        hFile,
        dwOffset as LONG,
        core::ptr::null_mut::<LONG>(),
        FILE_BEGIN as DWORD,
    ) == INVALID_SET_FILE_POINTER
    {
        GlobalUnlock(hData);
        GlobalFree(hData);
        return core::ptr::null_mut();
    }
    if ReadFile(
        hFile,
        lpData as PVOID,
        dwLength,
        core::ptr::addr_of_mut!(dwBytesRead),
        core::ptr::null_mut::<_OVERLAPPED>(),
    ) == 0
    {
        GlobalUnlock(hData);
        GlobalFree(hData);
        return core::ptr::null_mut();
    }
    GlobalUnlock(hData);
    return hData;
}

unsafe extern "C" fn ClipboardReadMemory(
    mut hFile: HANDLE,
    mut dwFormat: DWORD,
    mut dwOffset: DWORD,
    mut dwLength: DWORD,
    mut FileIdentifier: WORD,
    mut lpFormatName: PVOID,
) -> BOOL {
    let mut hData: HGLOBAL = core::ptr::null_mut::<core::ffi::c_void>();
    let mut dwTemp: DWORD = 0;
    hData = ClipboardReadMemoryBlock(hFile, dwOffset, dwLength);
    if hData.is_null() {
        return (FALSE) as BOOL;
    }
    if dwFormat >= 0xC000 && dwFormat <= 0xFFFF {
        if FileIdentifier as ::core::ffi::c_int == (CLIP_FMT_31 as i32) {
            dwTemp = RegisterClipboardFormatA(lpFormatName as LPCSTR) as DWORD;
        } else if FileIdentifier as ::core::ffi::c_int == (CLIP_FMT_NT as i32)
            || FileIdentifier as ::core::ffi::c_int == (CLIP_FMT_BK as i32)
        {
            dwTemp = RegisterClipboardFormatW(lpFormatName as LPCWSTR) as DWORD;
        }
        if dwTemp == 0 {
            GlobalFree(hData);
            return (FALSE) as BOOL;
        }
    } else {
        dwTemp = dwFormat;
    }
    if SetClipboardData(dwTemp as UINT, hData as HANDLE).is_null() {
        GlobalFree(hData);
        return (FALSE) as BOOL;
    }
    return TRUE as BOOL;
}

unsafe extern "C" fn ClipboardWriteMemory(hFile: HANDLE, dwFormat: DWORD, dwOffset: DWORD, pdwLength: PDWORD) -> BOOL {
    let mut hData: HGLOBAL;
    let mut lpData: LPVOID;
    let mut dwBytesWritten: DWORD = core::mem::zeroed();

    hData = GetClipboardData(dwFormat as UINT) as HGLOBAL;
    if hData.is_null() {
        return (FALSE) as BOOL;
    }

    lpData = GlobalLock(hData);
    if lpData.is_null() {
        return (FALSE) as BOOL;
    }

    *pdwLength = GlobalSize(hData) as DWORD;

    if SetFilePointer(hFile, dwOffset as LONG, core::ptr::null_mut::<LONG>(), FILE_BEGIN as DWORD) == INVALID_SET_FILE_POINTER {
        GlobalUnlock(hData);
        return FALSE as BOOL;
    }

    if WriteFile(hFile, lpData as LPCVOID, *pdwLength, core::ptr::addr_of_mut!(dwBytesWritten), core::ptr::null_mut::<_OVERLAPPED>()) == 0 {
        GlobalUnlock(hData);
        return FALSE as BOOL;
    }

    GlobalUnlock(hData);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn ClipboardReadPalette(hFile: HANDLE, dwOffset: DWORD, dwLength: DWORD) -> BOOL {
    let lpPalette: LPLOGPALETTE;
    let hPalette: HPALETTE;
    let hData: HGLOBAL;

    hData = ClipboardReadMemoryBlock(hFile, dwOffset, dwLength);
    if hData.is_null() {
        return (FALSE) as BOOL;
    }

    lpPalette = GlobalLock(hData) as LPLOGPALETTE;
    if lpPalette.is_null() {
        GlobalFree(hData);
        return (FALSE) as BOOL;
    }

    hPalette = CreatePalette(lpPalette as *const LOGPALETTE) as HPALETTE;
    if hPalette.is_null() {
        GlobalUnlock(hData);
        GlobalFree(hData);
        SetLastError(ERROR_OUTOFMEMORY as DWORD);
        return (FALSE) as BOOL;
    }

    GlobalUnlock(hData);
    GlobalFree(hData);

    if SetClipboardData(CF_PALETTE as UINT, hPalette as HANDLE).is_null() {
        DeleteObject(hPalette as HGDIOBJ);
        return (FALSE) as BOOL;
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn ClipboardReadMetafile(hFile: HANDLE, dwOffset: DWORD, dwLength: DWORD) -> BOOL {
    let mut hMf: HMETAFILE = core::ptr::null_mut();
    let mut hData: HGLOBAL = core::ptr::null_mut();
    let mut lpData: LPVOID = core::ptr::null_mut();

    hData = ClipboardReadMemoryBlock(hFile, dwOffset, dwLength);
    if hData.is_null() {
        return (FALSE) as BOOL;
    }

    lpData = GlobalLock(hData);
    if lpData.is_null() {
        GlobalFree(hData);
        return (FALSE) as BOOL;
    }

    hMf = SetMetaFileBitsEx(dwLength as UINT, lpData as *const BYTE);

    GlobalUnlock(hData);
    GlobalFree(hData);

    if hMf.is_null() {
        SetLastError(ERROR_OUTOFMEMORY as DWORD);
        return FALSE as BOOL;
    }

    if SetClipboardData(CF_METAFILEPICT as UINT, hMf as HANDLE).is_null() {
        DeleteMetaFile(hMf);
        return FALSE as BOOL;
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn ClipboardReadEnhMetafile(mut hFile: HANDLE, mut dwOffset: DWORD, mut dwLength: DWORD) -> BOOL {
    let mut hEmf: HENHMETAFILE = core::ptr::null_mut();
    let mut hData: HGLOBAL = core::ptr::null_mut();
    let mut lpData: LPVOID = core::ptr::null_mut();

    hData = ClipboardReadMemoryBlock(hFile, dwOffset, dwLength);
    if hData.is_null() {
        return (FALSE) as BOOL;
    }

    lpData = GlobalLock(hData);
    if lpData.is_null() {
        GlobalFree(hData);
        return (FALSE) as BOOL;
    }

    hEmf = SetEnhMetaFileBits(dwLength as UINT, lpData as *const BYTE);

    GlobalUnlock(hData);
    GlobalFree(hData);

    if hEmf.is_null() {
        SetLastError(ERROR_OUTOFMEMORY as DWORD);
        return (FALSE) as BOOL;
    }

    if SetClipboardData(CF_ENHMETAFILE as UINT, hEmf as HANDLE).is_null() {
        DeleteEnhMetaFile(hEmf);
        return (FALSE) as BOOL;
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn ClipboardReadBitmap(mut hFile: HANDLE, mut dwOffset: DWORD, mut dwLength: DWORD) -> BOOL {
    let mut hData: HGLOBAL = core::ptr::null_mut::<core::ffi::c_void>();
    let mut hBitmap: HBITMAP = core::ptr::null_mut::<HBITMAP__>();
    let mut lpBitmap: LPBITMAP = core::ptr::null_mut::<tagBITMAP>();
    hData = ClipboardReadMemoryBlock(hFile, dwOffset, dwLength);
    if hData.is_null() {
        return (FALSE) as BOOL;
    }
    lpBitmap = GlobalLock(hData) as LPBITMAP;
    if lpBitmap.is_null() {
        GlobalFree(hData);
        return (FALSE) as BOOL;
    }
    // The C's `lpBitmap + sizeof(BITMAP) + 1` is arithmetic on a BITMAP*, so it scales by
    // sizeof(BITMAP): bmBits pointed some 25 headers past a block holding one header and its
    // bits, and CreateBitmapIndirect read out of bounds. The bits follow the header; a block
    // too short to hold the header and bmHeight * bmWidthBytes of them is refused.
    let need = core::mem::size_of::<BITMAP>() as u64
        + ((*lpBitmap).bmHeight.max(0) as u64) * ((*lpBitmap).bmWidthBytes.max(0) as u64);
    if (dwLength as u64) < core::mem::size_of::<BITMAP>() as u64 || (dwLength as u64) < need {
        GlobalUnlock(hData);
        GlobalFree(hData);
        return (FALSE) as BOOL;
    }
    (*lpBitmap).bmBits = lpBitmap.add(1) as LPVOID;
    hBitmap = CreateBitmapIndirect(lpBitmap as *const BITMAP);
    GlobalUnlock(hData);
    GlobalFree(hData);
    if hBitmap.is_null() {
        SetLastError(ERROR_OUTOFMEMORY as DWORD);
        return (FALSE) as BOOL;
    }
    if SetClipboardData(CF_BITMAP as UINT, hBitmap as HANDLE).is_null() {
        DeleteObject(hBitmap as HGDIOBJ);
        return (FALSE) as BOOL;
    }
    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ReadClipboardFile(lpFileName: LPCWSTR) {
    let mut ClipFileHeader: CLIPFILEHEADER = core::mem::zeroed();
    let mut ClipFormatArray: CLIPFORMATHEADER = core::mem::zeroed();
    let mut NtClipFileHeader: NTCLIPFILEHEADER = core::mem::zeroed();
    let mut NtClipFormatArray: NTCLIPFORMATHEADER = core::mem::zeroed();
    let mut pClipFileHeader: PVOID = core::ptr::null_mut();
    let mut pClipFormatArray: PVOID = core::ptr::null_mut();
    let mut SizeOfFileHeader: DWORD = 0;
    let mut SizeOfFormatHeader: DWORD = 0;

    let mut wFileIdentifier: WORD = 0;
    let mut wFormatCount: WORD = 0;
    let mut dwFormatID: DWORD = 0;
    let mut dwLenData: DWORD = 0;
    let mut dwOffData: DWORD = 0;
    let mut szName: PVOID = core::ptr::null_mut();

    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut dwBytesRead: DWORD = 0;
    let mut bResult: BOOL = 0;
    let mut i: ::core::ffi::c_int = 0;

    'done: {
        /* Open the file for read access */
        hFile = CreateFileW(
            lpFileName,
            GENERIC_READ,
            FILE_SHARE_READ,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            core::ptr::null_mut(),
        );
        if hFile == INVALID_HANDLE_VALUE {
            ShowLastWin32Error(Globals.hMainWnd);
            break 'done;
        }

        /* Just read enough bytes to get the clipboard file format ID */
        if ReadFile(
            hFile,
            core::ptr::addr_of_mut!(wFileIdentifier) as PVOID,
            core::mem::size_of::<WORD>() as DWORD,
            core::ptr::addr_of_mut!(dwBytesRead),
            core::ptr::null_mut(),
        ) == 0
        {
            ShowLastWin32Error(Globals.hMainWnd);
            break 'done;
        }

        /* Set data according to the clipboard file format ID */
        match (wFileIdentifier) as u32 {
            CLIP_FMT_31 => {
                SizeOfFileHeader = core::mem::size_of::<CLIPFILEHEADER>() as DWORD;
                SizeOfFormatHeader = core::mem::size_of::<CLIPFORMATHEADER>() as DWORD;
                pClipFileHeader = core::ptr::addr_of_mut!(ClipFileHeader) as PVOID;
                pClipFormatArray = core::ptr::addr_of_mut!(ClipFormatArray) as PVOID;
            }
            CLIP_FMT_NT | CLIP_FMT_BK => {
                SizeOfFileHeader = core::mem::size_of::<NTCLIPFILEHEADER>() as DWORD;
                SizeOfFormatHeader = core::mem::size_of::<NTCLIPFORMATHEADER>() as DWORD;
                pClipFileHeader = core::ptr::addr_of_mut!(NtClipFileHeader) as PVOID;
                pClipFormatArray = core::ptr::addr_of_mut!(NtClipFormatArray) as PVOID;
            }
            _ => {
                MessageBoxRes(
                    Globals.hMainWnd,
                    Globals.hInstance,
                    ERROR_INVALID_FILE_FORMAT,
                    0,
                    MB_ICONSTOP | MB_OK,
                );
                break 'done;
            }
        }

        /* Completely read the header */
        SetFilePointer(
            hFile,
            0,
            core::ptr::null_mut(),
            FILE_BEGIN,
        );
        if ReadFile(
            hFile,
            pClipFileHeader,
            SizeOfFileHeader,
            core::ptr::addr_of_mut!(dwBytesRead),
            core::ptr::null_mut(),
        ) == 0
            || dwBytesRead != SizeOfFileHeader
        {
            ShowLastWin32Error(Globals.hMainWnd);
            break 'done;
        }

        /* Get header data */
        match (wFileIdentifier) as u32 {
            CLIP_FMT_31 => {
                assert!(
                    (wFileIdentifier as ::core::ffi::c_int
                        == (*(pClipFileHeader as *mut CLIPFILEHEADER)).wFileIdentifier
                            as ::core::ffi::c_int) as ::core::ffi::c_int != 0,
                );
                wFormatCount = (*(pClipFileHeader as *mut CLIPFILEHEADER)).wFormatCount;
            }
            CLIP_FMT_NT | CLIP_FMT_BK => {
                assert!(
                    (wFileIdentifier as ::core::ffi::c_int
                        == (*(pClipFileHeader as *mut NTCLIPFILEHEADER)).wFileIdentifier
                            as ::core::ffi::c_int) as ::core::ffi::c_int != 0,
                );
                wFormatCount = (*(pClipFileHeader as *mut NTCLIPFILEHEADER)).wFormatCount;
            }
            _ => {}
        }

        /* Loop through the format data array */
        i = 0;
        while i < wFormatCount as ::core::ffi::c_int {
            if SetFilePointer(
                hFile,
                (SizeOfFileHeader.wrapping_add((i as DWORD).wrapping_mul(SizeOfFormatHeader))) as LONG,
                core::ptr::null_mut(),
                FILE_BEGIN,
            ) == INVALID_SET_FILE_POINTER
            {
                ShowLastWin32Error(Globals.hMainWnd);
                break 'done;
            }

            if ReadFile(
                hFile,
                pClipFormatArray,
                SizeOfFormatHeader,
                core::ptr::addr_of_mut!(dwBytesRead),
                core::ptr::null_mut(),
            ) == 0
            {
                ShowLastWin32Error(Globals.hMainWnd);
                break 'done;
            }

            /* Get format data */
            match (wFileIdentifier) as u32 {
                CLIP_FMT_31 => {
                    dwFormatID = ((*(pClipFormatArray as *mut CLIPFORMATHEADER)).dwFormatID) as _;
                    dwLenData = (*(pClipFormatArray as *mut CLIPFORMATHEADER)).dwLenData;
                    dwOffData = (*(pClipFormatArray as *mut CLIPFORMATHEADER)).dwOffData;
                    szName = core::ptr::addr_of_mut!(
                        (*(pClipFormatArray as *mut CLIPFORMATHEADER)).szName
                    ) as PVOID;
                }
                CLIP_FMT_NT | CLIP_FMT_BK => {
                    dwFormatID = (*(pClipFormatArray as *mut NTCLIPFORMATHEADER)).dwFormatID;
                    dwLenData = (*(pClipFormatArray as *mut NTCLIPFORMATHEADER)).dwLenData;
                    dwOffData = (*(pClipFormatArray as *mut NTCLIPFORMATHEADER)).dwOffData;
                    szName = core::ptr::addr_of_mut!(
                        (*(pClipFormatArray as *mut NTCLIPFORMATHEADER)).szName
                    ) as PVOID;
                }
                _ => {}
            }

            match dwFormatID {
                CF_OWNERDISPLAY => {}
                CF_DSPBITMAP | CF_BITMAP => {
                    bResult = ClipboardReadBitmap(hFile, dwOffData, dwLenData);
                }
                CF_DSPMETAFILEPICT | CF_METAFILEPICT => {
                    bResult = ClipboardReadMetafile(hFile, dwOffData, dwLenData);
                }
                CF_DSPENHMETAFILE | CF_ENHMETAFILE => {
                    bResult = ClipboardReadEnhMetafile(hFile, dwOffData, dwLenData);
                }
                CF_PALETTE => {
                    bResult = ClipboardReadPalette(hFile, dwOffData, dwLenData);
                }
                _ => {
                    if dwFormatID < CF_PRIVATEFIRST || dwFormatID > CF_PRIVATELAST
                    {
                        bResult = ClipboardReadMemory(
                            hFile,
                            dwFormatID,
                            dwOffData,
                            dwLenData,
                            wFileIdentifier,
                            szName,
                        );
                    }
                }
            }

            if bResult == 0 {
                ShowLastWin32Error(Globals.hMainWnd);
            }

            i += 1;
        }
    }

    /* done: */
    if hFile != INVALID_HANDLE_VALUE {
        CloseHandle(hFile);
    }

    return;
}
#[no_mangle]
pub unsafe extern "C" fn WriteClipboardFile(lpFileName: LPCWSTR, wFileIdentifier: WORD) {
    let mut ClipFileHeader: CLIPFILEHEADER = core::mem::zeroed();
    let mut ClipFormatArray: CLIPFORMATHEADER = core::mem::zeroed();
    let mut NtClipFileHeader: NTCLIPFILEHEADER = core::mem::zeroed();
    let mut NtClipFormatArray: NTCLIPFORMATHEADER = core::mem::zeroed();
    let mut pClipFileHeader: PVOID = core::ptr::null_mut();
    let mut pClipFormatArray: PVOID = core::ptr::null_mut();
    let mut SizeOfFileHeader: DWORD = 0;
    let mut SizeOfFormatHeader: DWORD = 0;

    let mut wFormatCount: WORD = 0;
    let mut dwFormatID: DWORD = 0;
    let mut dwLenData: DWORD = 0;
    let mut dwOffData: DWORD = 0;

    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut dwBytesWritten: DWORD = 0;
    let mut i: i32 = 0;

    'done: {
        /* Create the file for write access */
        hFile = CreateFileW(
            lpFileName,
            GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            core::ptr::null_mut(),
        );
        if hFile == INVALID_HANDLE_VALUE {
            ShowLastWin32Error(Globals.hMainWnd);
            break 'done;
        }

        wFormatCount = CountClipboardFormats() as WORD;

        /* Select the file format and setup the header according to the clipboard file format ID */
        match (wFileIdentifier as i32) as u32 {
            CLIP_FMT_31 => {
                SizeOfFileHeader = core::mem::size_of::<CLIPFILEHEADER>() as DWORD;
                SizeOfFormatHeader = core::mem::size_of::<CLIPFORMATHEADER>() as DWORD;
                pClipFileHeader = core::ptr::addr_of_mut!(ClipFileHeader) as PVOID;
                pClipFormatArray = core::ptr::addr_of_mut!(ClipFormatArray) as PVOID;

                ClipFileHeader.wFileIdentifier = (CLIP_FMT_31 as u16); // wFileIdentifier
                ClipFileHeader.wFormatCount = wFormatCount;
            }

            CLIP_FMT_NT | CLIP_FMT_BK => {
                SizeOfFileHeader = core::mem::size_of::<NTCLIPFILEHEADER>() as DWORD;
                SizeOfFormatHeader = core::mem::size_of::<NTCLIPFORMATHEADER>() as DWORD;
                pClipFileHeader = core::ptr::addr_of_mut!(NtClipFileHeader) as PVOID;
                pClipFormatArray = core::ptr::addr_of_mut!(NtClipFormatArray) as PVOID;

                NtClipFileHeader.wFileIdentifier = (CLIP_FMT_NT as u16); // wFileIdentifier
                NtClipFileHeader.wFormatCount = wFormatCount;
            }

            _ => {
                MessageBoxRes(
                    Globals.hMainWnd,
                    Globals.hInstance,
                    ERROR_INVALID_FILE_FORMAT,
                    0,
                    MB_ICONSTOP | MB_OK,
                );
                break 'done;
            }
        }

        /* Write the header */
        SetFilePointer(
            hFile,
            0,
            core::ptr::null_mut(),
            FILE_BEGIN,
        );
        if WriteFile(
            hFile,
            pClipFileHeader,
            SizeOfFileHeader,
            core::ptr::addr_of_mut!(dwBytesWritten),
            core::ptr::null_mut(),
        ) == 0
            || dwBytesWritten != SizeOfFileHeader
        {
            ShowLastWin32Error(Globals.hMainWnd);
            break 'done;
        }

        /* Compute where the data should start (after the file header and the format array) */
        dwOffData = SizeOfFileHeader
            .wrapping_add((wFormatCount as DWORD).wrapping_mul(SizeOfFormatHeader));

        /* Loop through each format and save the data */
        i = 0;
        dwFormatID = EnumClipboardFormats(0);
        while dwFormatID != 0 {
            if i >= wFormatCount as i32 {
                /* Must never happen! */
                assert!(FALSE != 0);
                break;
            }

            'cont: {
                /* Write the clipboard data at the specified offset, and retrieve its length */
                if ClipboardWriteMemory(
                    hFile,
                    dwFormatID,
                    dwOffData,
                    core::ptr::addr_of_mut!(dwLenData),
                ) == 0
                {
                    break 'cont;
                }

                /* Write the format data header */
                match (wFileIdentifier as i32) as u32 {
                    CLIP_FMT_31 => {
                        ZeroMemory!(
                            pClipFormatArray,
                            core::mem::size_of::<CLIPFORMATHEADER>(),
                        );
                        (*(pClipFormatArray as *mut CLIPFORMATHEADER)).dwFormatID = (dwFormatID) as u16;
                        (*(pClipFormatArray as *mut CLIPFORMATHEADER)).dwLenData = dwLenData;
                        (*(pClipFormatArray as *mut CLIPFORMATHEADER)).dwOffData = dwOffData;
                        RetrieveClipboardFormatName(
                            Globals.hInstance,
                            dwFormatID,
                            (FALSE) as BOOL,
                            core::ptr::addr_of_mut!((*(pClipFormatArray as *mut CLIPFORMATHEADER)).szName)
                                as *mut CHAR as PVOID,
                            (core::mem::size_of::<[CHAR; 79]>() as usize)
                                .wrapping_div(core::mem::size_of::<CHAR>()) as UINT,
                        );
                    }

                    CLIP_FMT_NT | CLIP_FMT_BK => {
                        ZeroMemory!(
                            pClipFormatArray,
                            core::mem::size_of::<NTCLIPFORMATHEADER>(),
                        );
                        (*(pClipFormatArray as *mut NTCLIPFORMATHEADER)).dwFormatID = dwFormatID;
                        (*(pClipFormatArray as *mut NTCLIPFORMATHEADER)).dwLenData = dwLenData;
                        (*(pClipFormatArray as *mut NTCLIPFORMATHEADER)).dwOffData = dwOffData;
                        RetrieveClipboardFormatName(
                            Globals.hInstance,
                            dwFormatID,
                            (TRUE) as BOOL,
                            core::ptr::addr_of_mut!((*(pClipFormatArray as *mut NTCLIPFORMATHEADER)).szName)
                                as *mut WCHAR as PVOID,
                            (core::mem::size_of::<[WCHAR; 79]>() as usize)
                                .wrapping_div(core::mem::size_of::<WCHAR>()) as UINT,
                        );
                    }

                    _ => {}
                }

                if SetFilePointer(
                    hFile,
                    (SizeOfFileHeader.wrapping_add((i as DWORD).wrapping_mul(SizeOfFormatHeader))) as LONG,
                    core::ptr::null_mut(),
                    FILE_BEGIN,
                ) == INVALID_SET_FILE_POINTER
                {
                    ShowLastWin32Error(Globals.hMainWnd);
                    break 'done;
                }

                if WriteFile(
                    hFile,
                    pClipFormatArray,
                    SizeOfFormatHeader,
                    core::ptr::addr_of_mut!(dwBytesWritten),
                    core::ptr::null_mut(),
                ) == 0
                {
                    ShowLastWin32Error(Globals.hMainWnd);
                    break 'done;
                }

                /* Adjust the offset for the next data stream */
                dwOffData = dwOffData.wrapping_add(dwLenData);
            }

            i += 1;
            dwFormatID = EnumClipboardFormats(dwFormatID);
        }
    }

    if hFile != INVALID_HANDLE_VALUE {
        CloseHandle(hFile);
    }

    return;
}
