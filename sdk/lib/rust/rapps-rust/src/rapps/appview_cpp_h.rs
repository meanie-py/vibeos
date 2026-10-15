use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_00FB3B10D34F: [WCHAR; 35] = [0x0056, 0x0069, 0x0065, 0x0077, 0x0020, 0x006D, 0x006F, 0x0064, 0x0065, 0x0020, 0x0069, 0x006E, 0x0076, 0x0061, 0x006C, 0x0069, 0x0064, 0x0020, 0x006F, 0x0072, 0x0020, 0x0075, 0x006E, 0x0069, 0x006D, 0x0070, 0x006C, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0065, 0x0064, 0x0000];
pub static CPP_COM_WSTR_075DE2B906DB: [WCHAR; 2] = [0x000A, 0x0000];
pub static CPP_COM_WSTR_2742CD8F7DE1: [WCHAR; 5] = [0x0045, 0x0064, 0x0069, 0x0074, 0x0000];
pub static CPP_COM_WSTR_4ACCF377958C: [WCHAR; 12] = [0x0073, 0x0063, 0x0072, 0x0065, 0x0065, 0x006E, 0x0073, 0x0068, 0x006F, 0x0074, 0x0073, 0x0000];
pub static CPP_COM_WSTR_7AFA86ACD8E9: [WCHAR; 21] = [0x0052, 0x0041, 0x0070, 0x0070, 0x0073, 0x0041, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0056, 0x0069, 0x0065, 0x0077, 0x0000];
pub static CPP_COM_WSTR_82DB442E7A60: [WCHAR; 5] = [0x006F, 0x0070, 0x0065, 0x006E, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_9D313EA1711F: [WCHAR; 3] = [0x000A, 0x000A, 0x0000];
pub static CPP_COM_WSTR_BE3990BCF6A7: [WCHAR; 13] = [0x0052, 0x0041, 0x0070, 0x0070, 0x0073, 0x0041, 0x0070, 0x0070, 0x0049, 0x006E, 0x0066, 0x006F, 0x0000];
pub static CPP_COM_WSTR_C54B7D3295AA: [WCHAR; 4] = [0x0069, 0x006D, 0x0067, 0x0000];
pub static CPP_COM_WSTR_D4C31F2B2950: [WCHAR; 4] = [0x002D, 0x002D, 0x002D, 0x0000];
pub static CPP_COM_WSTR_DE9FCDB32E36: [WCHAR; 21] = [0x0052, 0x0041, 0x0070, 0x0070, 0x0073, 0x0053, 0x0063, 0x0072, 0x006E, 0x0073, 0x0068, 0x006F, 0x0074, 0x0050, 0x0072, 0x0065, 0x0076, 0x0069, 0x0065, 0x0077, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub static mut g_hDefaultPackageIcon: HICON = core::ptr::null_mut();
pub static mut g_DefaultPackageIconILIdx: i32 = I_IMAGENONE;
pub static mut g_IconSize: UINT = 0;
pub static mut g_AsyncIconTasks: *mut CAsyncLoadIcon = core::ptr::null_mut();
pub static mut g_AsyncIconTaskId: UINT = 0;





































































































#[no_mangle]
pub unsafe extern "C" fn DeleteMenuEx(hMenu: HMENU, uPosition: UINT, uFlags: UINT) -> BOOL {
    let mut pos: INT;
    let mut mii: MENUITEMINFOW = core::mem::zeroed();
    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
    mii.fMask = MIIM_FTYPE;
    mii.fType = 0;
    let mut bIsValidItem1: bool;
    let mut bIsValidItem2: bool;
    let mut bIsSep1: bool;
    let mut bIsSep2: bool;

    if (uFlags & MF_BYPOSITION) != 0 {
        pos = uPosition as INT;
    } else {
        pos = GetMenuPosFromID(hMenu, uPosition);
    }
    if pos < 0 {
        return (FALSE) as BOOL;
    }

    bIsValidItem1 = (pos > 0)
        && (GetMenuItemInfoW(
            hMenu,
            (pos - 1) as UINT,
            (TRUE) as BOOL,
            core::ptr::addr_of_mut!(mii),
        ) != 0);
    bIsSep1 = bIsValidItem1 && ((mii.fType & MFT_SEPARATOR) != 0);

    bIsValidItem2 = GetMenuItemInfoW(
        hMenu,
        (pos + 1) as UINT,
        (TRUE) as BOOL,
        core::ptr::addr_of_mut!(mii),
    ) != 0;
    bIsSep2 = bIsValidItem2 && ((mii.fType & MFT_SEPARATOR) != 0);

    if bIsSep1 && !bIsSep2 && !bIsValidItem2 {
        pos = pos - 1; // Delete separator only if pos+1 has no item
    } else if !bIsSep1 && bIsSep2 && !bIsValidItem1 {
        pos = pos + 1; // Delete separator only if pos-1 has no item
    } else if bIsSep1 && bIsSep2 {
        pos = pos + 1;
    } else {
        pos = -1;
    }

    // Delete one of the separators if necessary
    if pos != -1 {
        DeleteMenu(hMenu, pos as UINT, MF_BYPOSITION);
    }

    // Finally, delete the menu item itself.
    return DeleteMenu(hMenu, uPosition, uFlags);
}
#[no_mangle]
pub unsafe extern "C" fn ScrnshotDownloadCallback(AsyncInet: pASYNCINET, Event: ASYNC_EVENT, wParam: WPARAM, lParam: LPARAM, Extension: *mut core::ffi::c_void) -> i32 {
    let DownloadParam: *mut ScrnshotDownloadParam = Extension as *mut ScrnshotDownloadParam;
    match Event {
        ASYNCINET_DATA => {
            let mut BytesWritten: DWORD = 0;
            WriteFile((*DownloadParam).hFile, wParam as LPCVOID, lParam as DWORD, &mut BytesWritten, core::ptr::null_mut());
        }
        ASYNCINET_COMPLETE => {
            CloseHandle((*DownloadParam).hFile);
            SendMessage(
                (*DownloadParam).hwndNotify, WM_RAPPS_DOWNLOAD_COMPLETE, ERROR_SUCCESS as WPARAM, DownloadParam as LPARAM);
        }
        ASYNCINET_CANCELLED => {
            CloseHandle((*DownloadParam).hFile);
            SendMessage(
                (*DownloadParam).hwndNotify, WM_RAPPS_DOWNLOAD_COMPLETE, ERROR_CANCELLED as WPARAM, DownloadParam as LPARAM);
        }
        ASYNCINET_ERROR => {
            CloseHandle((*DownloadParam).hFile);
            SendMessage((*DownloadParam).hwndNotify, WM_RAPPS_DOWNLOAD_COMPLETE, wParam, DownloadParam as LPARAM);
        }
        _ => {
            ATLASSERT!(FALSE);
        }
    }
    return 0;
}
unsafe extern "system" fn AsyncLoadIconProc(Param: LPVOID) -> DWORD {
    let mut task: *mut CAsyncLoadIcon = Param as *mut CAsyncLoadIcon;
    let mut old: *mut CAsyncLoadIcon = core::ptr::null_mut();
    while !task.is_null() {
        if (*task).TaskId == g_AsyncIconTaskId {
            let mut hIcon: HICON = core::ptr::null_mut();
            let phBigIcon: *mut HICON = if SettingsInfo.bSmallIcons != 0 {
                core::ptr::null_mut()
            } else {
                core::ptr::addr_of_mut!(hIcon)
            };
            let phSmallIcon: *mut HICON = if !phBigIcon.is_null() {
                core::ptr::null_mut()
            } else {
                core::ptr::addr_of_mut!(hIcon)
            };
            if !(*task).Parse {
                hIcon = LoadImageW(
                    core::ptr::null_mut(),
                    (*task).Location.as_ptr(),
                    IMAGE_ICON,
                    (g_IconSize) as i32,
                    (g_IconSize) as i32,
                    LR_LOADFROMFILE,
                ) as HICON;
            } else if ExtractIconExW(
                (*task).Location.as_ptr(),
                PathParseIconLocationW((*task).Location.as_ptr()),
                phBigIcon,
                phSmallIcon,
                1,
            ) == 0
            {
                hIcon = core::ptr::null_mut();
            }

            if !hIcon.is_null() {
                SendMessageW(
                    (*task).hAppsList,
                    WM_RAPPSLIST_ASYNCICON,
                    (hIcon) as WPARAM,
                    task as LPARAM,
                );
                DestroyIcon(hIcon);
            }
        }
        old = task;
        task = (*task).pNext;
        (*old).Free();
    }
    return 0;
}

impl CAppRichEdit {
    pub unsafe fn LoadAndInsertText__0(&mut self, uStringID: UINT, szText: &CStringW, TextFlags: DWORD) {
        let this_: *mut Self = self;
        let mut szLoadedText = CStringW::new();
        if !szText.IsEmpty() && szLoadedText.LoadStringW(uStringID) != 0 {
            let StringFlags: DWORD = CFE_BOLD;
            (*this_).__base0.__base2.InsertText__1(&szLoadedText, StringFlags);
            (*this_).__base0.__base2.InsertText__1(szText, TextFlags);
        }
    }
    pub unsafe fn LoadAndInsertText__1(&mut self, uStringID: UINT, StringFlags: DWORD) {
        let this_: *mut Self = self;
        let mut szLoadedText = CStringW::new();
        if szLoadedText.LoadStringW(uStringID) != 0 {
            (*this_).__base0.__base2.InsertText__0(CPP_COM_WSTR_075DE2B906DB.as_ptr(), 0);
            (*this_).__base0.__base2.InsertText__1(&szLoadedText, StringFlags);
            (*this_).__base0.__base2.InsertText__0(CPP_COM_WSTR_075DE2B906DB.as_ptr(), 0);
        }
    }
    pub unsafe fn InsertTextWithString(&mut self, StringID: UINT, Text: &CStringW, TextFlags: DWORD) {
        let this_: *mut Self = self;
        if !Text.IsEmpty() {
            (*this_).LoadAndInsertText__0(StringID, Text, TextFlags);
        }
    }
}

impl CAppScrnshotPreview {
    pub unsafe fn ProcessWindowMessage(&mut self, hwnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM, theResult: &mut LRESULT, dwMapId: DWORD) -> BOOL {
        let this_: *mut Self = self;
        *theResult = 0;
        'switch: {
            match Msg {
                WM_CREATE => {
                    (*this_).hBrokenImgIcon = LoadImage(hInst, MAKEINTRESOURCE!(IDI_BROKEN_IMAGE), IMAGE_ICON, (*this_).BrokenImgSize, (*this_).BrokenImgSize, 0) as HICON;
                    break 'switch;
                }
                WM_SIZE => {
                    if (*this_).BrokenImgSize != min!(min!(GET_X_LPARAM!(lParam), GET_Y_LPARAM!(lParam)), (BROKENIMG_ICON_SIZE as i32)) {
                        (*this_).BrokenImgSize = min!(min!(GET_X_LPARAM!(lParam), GET_Y_LPARAM!(lParam)), (BROKENIMG_ICON_SIZE as i32));

                        if !(*this_).hBrokenImgIcon.is_null() {
                            DeleteObject(((*this_).hBrokenImgIcon).cast());
                            (*this_).hBrokenImgIcon = LoadImage(
                                hInst, MAKEINTRESOURCE!(IDI_BROKEN_IMAGE), IMAGE_ICON, (*this_).BrokenImgSize, (*this_).BrokenImgSize, 0) as HICON;
                        }
                    }
                    break 'switch;
                }
                WM_RAPPS_DOWNLOAD_COMPLETE => {
                    let DownloadParam = lParam as *mut ScrnshotDownloadParam;
                    AsyncInetRelease((*this_).AsyncInet);
                    (*this_).AsyncInet = core::ptr::null_mut();
                    'wswitch: {
                        match wParam {
                            __c if __c == ERROR_SUCCESS as _ => {
                                if (*this_).ContentID == (*DownloadParam).ID {
                                    (*this_).DisplayFile((*DownloadParam).DownloadFileName.GetString());
                                    // send a message to trigger resizing
                                    SendMessageW(GetParent((*this_).__base0.m_hWnd), WM_RAPPS_RESIZE_CHILDREN, 0, 0);
                                    (*this_).__base0.InvalidateRect(core::ptr::null(), 0);
                                    (*this_).TempImagePath =
                                        (*DownloadParam).DownloadFileName; // record tmp file path in order to delete it when cleanup
                                } else {
                                    // the picture downloaded is already outdated. delete it.
                                    DeleteFileW((*DownloadParam).DownloadFileName.GetString());
                                }
                                break 'wswitch;
                            }
                            __c if __c == ERROR_CANCELLED as _ => {
                                DeleteFileW((*DownloadParam).DownloadFileName.GetString());
                                break 'wswitch;
                            }
                            _ => {
                                (*this_).DisplayFailed();
                                // send a message to trigger resizing
                                SendMessageW(GetParent((*this_).__base0.m_hWnd), WM_RAPPS_RESIZE_CHILDREN, 0, 0);
                                (*this_).__base0.InvalidateRect(core::ptr::null(), 0);
                                DeleteFileW((*DownloadParam).DownloadFileName.GetString());
                                break 'wswitch;
                            }
                        }
                    }
                    cpp_delete(DownloadParam);
                    break 'switch;
                }
                WM_PAINT => {
                    let mut ps: PAINTSTRUCT = core::mem::zeroed();
                    let hdc: HDC = (*this_).__base0.BeginPaint(core::ptr::addr_of_mut!(ps));
                    let mut rect: CRect = CRect__make_0();
                    (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rect.__base0));

                    (*this_).PaintOnDC(hdc, rect.Width(), rect.Height(), ps.fErase);

                    (*this_).__base0.EndPaint(core::ptr::addr_of_mut!(ps));
                    break 'switch;
                }
                WM_PRINTCLIENT => {
                    if (lParam & PRF_CHECKVISIBLE) != 0 {
                        if (((*this_).__base0.IsWindowVisible()) == 0) {
                            break 'switch;
                        }
                    }
                    let mut rect: CRect = CRect__make_0();
                    (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rect.__base0));

                    (*this_).PaintOnDC(wParam as HDC, rect.Width(), rect.Height(), (lParam & PRF_ERASEBKGND) as BOOL);
                    break 'switch;
                }
                WM_ERASEBKGND => {
                    return (TRUE) as BOOL; // do not erase to avoid blinking
                }
                WM_TIMER => {
                    'wswitch2: {
                        match wParam {
                            __c if __c == TIMER_LOADING_ANIMATION as _ => {
                                (*this_).LoadingAnimationFrame += 1;
                                (*this_).LoadingAnimationFrame %= ((LOADING_ANIMATION_PERIOD * LOADING_ANIMATION_FPS)) as i32;
                                let hdc: HDC = (*this_).__base0.GetDC();
                                let mut rect: CRect = CRect__make_0();
                                (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rect.__base0));

                                (*this_).PaintOnDC(hdc, rect.Width(), rect.Height(), (TRUE) as BOOL);
                                (*this_).__base0.ReleaseDC(hdc);
                            }
                            _ => {}
                        }
                    }
                    break 'switch;
                }
                WM_DESTROY => {
                    (*this_).PreviousDisplayCleanup();
                    DeleteObject(((*this_).hBrokenImgIcon).cast());
                    (*this_).hBrokenImgIcon = core::ptr::null_mut();
                    break 'switch;
                }
                _ => {}
            }
        }
        return (FALSE) as BOOL;
    }
    pub unsafe fn DisplayLoading(&mut self) {
        let this_: *mut Self = self;
        (*this_).SetStatus(SCRNSHOT_PREV_LOADING);
        if (*this_).bLoadingTimerOn != 0 {
            (*this_).__base0.KillTimer((TIMER_LOADING_ANIMATION as _));
        }
        (*this_).LoadingAnimationFrame = 0;
        (*this_).bLoadingTimerOn = (TRUE as i32);
        (*this_).__base0.SetTimer(TIMER_LOADING_ANIMATION, 1000 / LOADING_ANIMATION_FPS, 0);
    }
    pub unsafe fn DisplayFailed(&mut self) {
        let this_: *mut Self = self;
        InterlockedIncrement64(core::ptr::addr_of_mut!((*this_).ContentID));
        (*this_).SetStatus(SCRNSHOT_PREV_FAILED);
        (*this_).PreviousDisplayCleanup();
    }
    pub unsafe fn DisplayFile(&mut self, lpszFileName: LPCWSTR) -> BOOL {
        let this_: *mut Self = self;
        (*this_).PreviousDisplayCleanup();
        (*this_).SetStatus(SCRNSHOT_PREV_IMAGE);
        (*this_).pImage = core::ptr::addr_of_mut!((*Bitmap::FromFile(lpszFileName, 0)).__base0);
        if (*(*this_).pImage).GetLastStatus() != 0 /* Ok */ {
            (*this_).DisplayFailed();
            return (FALSE) as BOOL;
        }
        return (TRUE) as BOOL;
    }
    pub unsafe fn SetStatus(&mut self, Status: SCRNSHOT_STATUS,
    ) {
        let this_: *mut Self = self;
        (*this_).ScrnshotPrevStauts = Status;
    }
    pub unsafe fn PaintOnDC(&mut self, hdc: HDC, width: i32, height: i32, bDrawBkgnd: BOOL) {
        let this_: *mut Self = self;
        // use an off screen dc to avoid blinking
        let hdcMem: HDC = CreateCompatibleDC(hdc);
        let hBitmap: HBITMAP = CreateCompatibleBitmap(hdc, width, height);
        SelectObject(hdcMem, hBitmap as HGDIOBJ);

        if bDrawBkgnd != 0 {
            let hOldBrush: HBRUSH = SelectObject(hdcMem, GetSysColorBrush((COLOR_BTNFACE as i32)) as HGDIOBJ) as HBRUSH;
            PatBlt(hdcMem, 0, 0, width, height, PATCOPY);
            SelectObject(hdcMem, hOldBrush as HGDIOBJ);
        }

        match (*this_).ScrnshotPrevStauts {
            SCRNSHOT_PREV_EMPTY => {
            }

            SCRNSHOT_PREV_LOADING => {
                let mut graphics: Graphics = Graphics__make_1(hdcMem);
                let mut color: Color = Color__make_2(255, 0, 0);
                let mut dotBrush: SolidBrush = SolidBrush__zeroed();
                dotBrush.__ctor_0(&(Color__make_3(255, 100, 100, 100)));

                graphics.SetSmoothingMode(SmoothingMode::SmoothingModeAntiAlias);

                // Paint three dot
                let DotWidth: f32 = (*this_).GetLoadingDotWidth(width, height);
                graphics.FillEllipse__1(
                    core::ptr::addr_of!(dotBrush.__base0) as *const Brush,
                    (width as REAL) / 2.0 - (min!(width, height) as REAL) * 2.0 / 16.0 - DotWidth / 2.0,
                    (height as REAL) / 2.0
                        - (*this_).GetFrameDotShift((*this_).LoadingAnimationFrame + (LOADING_ANIMATION_FPS / 4) as i32, width, height) - DotWidth / 2.0,
                    DotWidth,
                    DotWidth,
                );

                graphics.FillEllipse__1(
                    core::ptr::addr_of!(dotBrush.__base0) as *const Brush,
                    (width as REAL) / 2.0 - DotWidth / 2.0,
                    (height as REAL) / 2.0 - (*this_).GetFrameDotShift((*this_).LoadingAnimationFrame, width, height) - DotWidth / 2.0,
                    DotWidth,
                    DotWidth,
                );

                graphics.FillEllipse__1(
                    core::ptr::addr_of!(dotBrush.__base0) as *const Brush,
                    (width as REAL) / 2.0 + (min!(width, height) as REAL) * 2.0 / 16.0 - DotWidth / 2.0,
                    (height as REAL) / 2.0
                        - (*this_).GetFrameDotShift((*this_).LoadingAnimationFrame - (LOADING_ANIMATION_FPS / 4) as i32, width, height) - DotWidth / 2.0,
                    DotWidth,
                    DotWidth,
                );
            }

            SCRNSHOT_PREV_IMAGE => {
                if !(*this_).pImage.is_null() {
                    // always draw entire image inside the window.
                    let mut graphics: Graphics = Graphics__make_1(hdcMem);
                    let ZoomRatio: f32 = min!(
                        ((width as f32) / ((*(*this_).pImage).GetWidth() as f32)),
                        ((height as f32) / ((*(*this_).pImage).GetHeight() as f32)),
                    );
                    let ZoomedImgWidth: f32 = ZoomRatio * ((*(*this_).pImage).GetWidth() as f32);
                    let ZoomedImgHeight: f32 = ZoomRatio * ((*(*this_).pImage).GetHeight() as f32);

                    graphics.DrawImage__12(
                        (*this_).pImage,
                        ((width as f32) - ZoomedImgWidth) / 2.0,
                        ((height as f32) - ZoomedImgHeight) / 2.0,
                        ZoomedImgWidth,
                        ZoomedImgHeight,
                    );
                }
            }

            SCRNSHOT_PREV_FAILED => {
                DrawIconEx(
                    hdcMem,
                    (width - (*this_).BrokenImgSize) / 2,
                    (height - (*this_).BrokenImgSize) / 2,
                    (*this_).hBrokenImgIcon,
                    (*this_).BrokenImgSize,
                    (*this_).BrokenImgSize,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    DI_NORMAL | DI_COMPAT,
                );
            }

            _ => {}
        }

        // copy the content form off-screen dc to hdc
        BitBlt(hdc, 0, 0, width, height, hdcMem, 0, 0, SRCCOPY);
        DeleteDC(hdcMem);
        DeleteObject(hBitmap as HGDIOBJ);
    }
    pub unsafe fn GetLoadingDotWidth(&mut self, width: i32, height: i32) -> f32 {
        let this_: *mut Self = self;
        let _ = this_;
        min!(width, height) as f32 / 20.0
    }
    pub unsafe fn GetFrameDotShift(&mut self, Frame: i32, width: i32, height: i32) -> f32 {
        let this_: *mut Self = self;
        return (min!(width, height) as f64 * (1.0 / 16.0) * (2.0 / (2.0 - sqrt(3.0))) * (max!(sin(((Frame as f32) * 2.0 * PI / (LOADING_ANIMATION_PERIOD as f32 * LOADING_ANIMATION_FPS)) as f64), sqrt(3.0) / 2.0) - sqrt(3.0) / 2.0)) as f32;
    }
    pub unsafe fn GetWndClassInfo() -> *mut CWndClassInfo {
        let csStyle: DWORD = CS_VREDRAW | CS_HREDRAW;
        static mut wc: CWndClassInfo = unsafe { core::mem::zeroed() };
        wc = CWndClassInfo {
            m_wc: WNDCLASSEXW {
                cbSize: core::mem::size_of::<WNDCLASSEX>() as UINT,
                style: csStyle,
                lpfnWndProc: StartWindowProc,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: core::ptr::null_mut(),
                hIcon: core::ptr::null_mut(),
                hCursor: LoadCursorW(core::ptr::null_mut(), IDC_ARROW),
                hbrBackground: (COLOR_BTNFACE + 1) as HBRUSH,
                lpszMenuName: core::ptr::null(),
                lpszClassName: CPP_COM_WSTR_DE9FCDB32E36.as_ptr(),
                hIconSm: core::ptr::null_mut(),
            },
            m_lpszOrigName: core::ptr::null_mut(),
            pWndProc: None,
            m_lpszCursorID: IDC_ARROW,
            m_bSystemCursor: (TRUE as i32),
            m_atom: 0,
            m_szAutoName: CPP_COM_WSTR_96A296D224F2.as_ptr(),
        };
        return core::ptr::addr_of_mut!(wc);
    }
    pub unsafe fn Create(&mut self, hParent: HWND) -> HWND {
        let this_: *mut Self = self;
        let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };

        return (*this_).CWindowImpl_Create(hParent, &mut r, CPP_COM_WSTR_96A296D224F2.as_ptr(), WS_CHILD | WS_VISIBLE, 0, 0);
    }
    pub unsafe fn PreviousDisplayCleanup(&mut self) {
        let this_: *mut Self = self;
        if (*this_).bLoadingTimerOn != 0 {
            (*this_).__base0.KillTimer((TIMER_LOADING_ANIMATION as _));
            (*this_).bLoadingTimerOn = (FALSE as i32);
        }
        (*this_).LoadingAnimationFrame = 0;
        if !(*this_).pImage.is_null() {
            Image__delete((*this_).pImage);
            (*this_).pImage = core::ptr::null_mut();
        }
        if !(*this_).AsyncInet.is_null() {
            AsyncInetCancel((*this_).AsyncInet);
        }
        if (*this_).TempImagePath.IsEmpty() == false {
            DeleteFileW((*this_).TempImagePath.GetString());
            (*this_).TempImagePath.Empty();
        }
    }
    pub unsafe fn DisplayEmpty(&mut self) {
        let this_: *mut Self = self;
        InterlockedIncrement64(core::ptr::addr_of_mut!((*this_).ContentID));
        (*this_).SetStatus(SCRNSHOT_PREV_EMPTY);
        (*this_).PreviousDisplayCleanup();
    }
    pub unsafe fn DisplayImage(&mut self, lpszLocation: LPCWSTR) -> BOOL {
        let this_: *mut Self = self;
        let ID = InterlockedIncrement64(core::ptr::addr_of_mut!((*this_).ContentID));
        (*this_).PreviousDisplayCleanup();

        if PathIsURLW(lpszLocation) != 0 {
            (*this_).DisplayLoading();

            let DownloadParam: *mut ScrnshotDownloadParam = cpp_new::<ScrnshotDownloadParam>();
            if DownloadParam.is_null() {
                return (FALSE) as BOOL;
            }

            (*DownloadParam).hwndNotify = (*this_).__base0.m_hWnd;
            (*DownloadParam).ID = ID;
            // generate a filename
            let mut ScrnshotFolder = CStringW::from_ptr((*this_).m_BasePath.GetString());
            PathAppendW(ScrnshotFolder.GetBuffer((MAX_PATH as i32)), CPP_COM_WSTR_4ACCF377958C.as_ptr());
            ScrnshotFolder.ReleaseBuffer();

            if PathIsDirectoryW(ScrnshotFolder.GetString()) == 0 {
                CreateDirectoryW(ScrnshotFolder.GetString(), core::ptr::null_mut());
            }

            if GetTempFileNameW(
                ScrnshotFolder.GetString(),
                CPP_COM_WSTR_C54B7D3295AA.as_ptr(),
                0,
                (*DownloadParam).DownloadFileName.GetBuffer((MAX_PATH as i32)),
            ) == 0
            {
                (*DownloadParam).DownloadFileName.ReleaseBuffer();
                cpp_delete(DownloadParam);
                (*this_).DisplayFailed();
                return (FALSE) as BOOL;
            }
            (*DownloadParam).DownloadFileName.ReleaseBuffer();

            (*DownloadParam).hFile = CreateFileW(
                (*DownloadParam).DownloadFileName.GetString(),
                GENERIC_WRITE,
                0,
                core::ptr::null_mut(),
                CREATE_ALWAYS,
                FILE_ATTRIBUTE_NORMAL,
                core::ptr::null_mut(),
            );
            if (*DownloadParam).hFile == INVALID_HANDLE_VALUE {
                cpp_delete(DownloadParam);
                (*this_).DisplayFailed();
                return (FALSE) as BOOL;
            }

            (*this_).AsyncInet = AsyncInetDownload(
                core::ptr::null(),
                INTERNET_OPEN_TYPE_PRECONFIG,
                core::ptr::null(),
                core::ptr::null(),
                lpszLocation,
                (TRUE) as BOOL,
                Some(ScrnshotDownloadCallback),
                (DownloadParam).cast(),
            );
            if (*this_).AsyncInet.is_null() {
                CloseHandle((*DownloadParam).hFile);
                DeleteFileW((*DownloadParam).DownloadFileName.GetBuffer(0));
                cpp_delete(DownloadParam);
                (*this_).DisplayFailed();
                return (FALSE) as BOOL;
            }
            return (TRUE) as BOOL;
        } else {
            return (*this_).DisplayFile(lpszLocation);
        }
    }
    pub unsafe fn GetRequestedWidth(&mut self, Height: i32) -> i32 {
        let this_: *mut Self = self;
        match (*this_).ScrnshotPrevStauts {
            SCRNSHOT_PREV_EMPTY => {
                return 0;
            }
            SCRNSHOT_PREV_LOADING => {
                return 200;
            }
            SCRNSHOT_PREV_IMAGE => {
                if !(*this_).pImage.is_null() {
                    // return the width needed to display image inside the window.
                    // and always keep window w/h ratio inside [ 1/SCRNSHOT_MAX_ASPECT_RAT, SCRNSHOT_MAX_ASPECT_RAT ]
                    return floor(
                        (Height as f32)
                            * max!(
                                min!(
                                    ((*(*this_).pImage).GetWidth() as f32)
                                        / ((*(*this_).pImage).GetHeight() as f32),
                                    (SCRNSHOT_MAX_ASPECT_RAT as f32),
                                ),
                                1.0f32 / (SCRNSHOT_MAX_ASPECT_RAT as f32),
                            ),
                    ) as i32;
                }
                return 0;
            }
            SCRNSHOT_PREV_FAILED => {
                return 200;
            }
            _ => {
                return 0;
            }
        }
    }
    pub unsafe fn __ctor_0(&mut self, BasePath: &CStringW) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_BasePath = CStringW::from_ptr((*BasePath).GetString());
        (*this_).ScrnshotPrevStauts = SCRNSHOT_PREV_EMPTY;
        (*this_).pImage = core::ptr::null_mut();
        (*this_).hBrokenImgIcon = core::ptr::null_mut();
        (*this_).bLoadingTimerOn = (FALSE as i32);
        (*this_).LoadingAnimationFrame = 0;
        (*this_).BrokenImgSize = (BROKENIMG_ICON_SIZE as i32);
        (*this_).AsyncInet = core::ptr::null_mut();
        (*this_).ContentID = 0;
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).PreviousDisplayCleanup();
    }
}

impl CAppInfoDisplay {
    pub unsafe fn ProcessWindowMessage(&mut self, hwnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM, theResult: &mut LRESULT, dwMapId: DWORD) -> BOOL {
        let this_: *mut Self = self;
        *theResult = 0;
        match message {
            WM_CREATE => {
                (*this_).RichEdit = CAppRichEdit__new_0();
                (*(*(*this_).RichEdit).__base0.__base2).Create(hwnd);

                let mut Directory: CStringW = CStringW::new();
                GetStorageDirectory(&mut Directory);
                (*this_).ScrnshotPrev = CAppScrnshotPreview__new_0(&Directory);
                (*(*this_).ScrnshotPrev).Create(hwnd);
            }
            WM_SIZE => {
                (*this_).ResizeChildren__1(GET_X_LPARAM!(lParam), GET_Y_LPARAM!(lParam));
            }
            WM_RAPPS_RESIZE_CHILDREN => {
                (*this_).ResizeChildren__0();
            }
            WM_COMMAND => {
                (*this_).OnCommand(wParam, lParam);
            }
            WM_NOTIFY => {
                let NotifyHeader: *mut NMHDR = lParam as *mut NMHDR;
                if (*NotifyHeader).hwndFrom == (*(*(*this_).RichEdit).__base0.__base2.__base0).m_hWnd {
                    match (*NotifyHeader).code {
                        EN_LINK => {
                            (*this_).OnLink(lParam as *mut ENLINK);
                        }
                        _ => {}
                    }
                }
            }
            WM_SYSCOLORCHANGE => {
                (*(*(*this_).RichEdit).__base0.__base2.__base0).SendMessageW__0(EM_SETBKGNDCOLOR, 0, GetSysColor((COLOR_BTNFACE as i32)));
            }
            _ => {}
        }

        return (FALSE) as BOOL;
    }
    pub unsafe fn OnLink(&mut self, Link: *mut ENLINK) {
        let this_: *mut Self = self;
        match (*Link).msg {
            WM_LBUTTONUP | WM_RBUTTONUP => {
                if !(*this_).pLink.is_null() {
                    HeapFree(GetProcessHeap(), 0, (*this_).pLink as LPVOID);
                }

                (*this_).pLink = HeapAlloc(
                    GetProcessHeap(), 0,
                    (((max!((*Link).chrg.cpMin, (*Link).chrg.cpMax) - min!((*Link).chrg.cpMin, (*Link).chrg.cpMax) + 1) as usize)
                        * core::mem::size_of::<u16>()) as SIZE_T
                ) as LPWSTR;
                if (*this_).pLink.is_null() {
                    /* TODO: Error message */
                    return;
                }

                (*(*this_).RichEdit).__base0.__base2.__base0.SendMessageW__0(EM_SETSEL, ((*Link).chrg.cpMin) as WPARAM, ((*Link).chrg.cpMax) as LPARAM);
                (*(*this_).RichEdit).__base0.__base2.__base0.SendMessageW__0(EM_GETSELTEXT, 0, (*this_).pLink as LPARAM);

                ShowPopupMenuEx((*this_).__base0.__base2.m_hWnd, (*this_).__base0.__base2.m_hWnd, IDR_LINKMENU, (1 as u32).wrapping_neg(), core::ptr::null_mut());
            }
            _ => {}
        }
    }
    pub unsafe fn ResizeChildren__0(&mut self) {
        let this_: *mut Self = self;
        let mut rect: CRect = CRect__make_0();
        (*this_).__base0.__base2.GetWindowRect(core::ptr::addr_of_mut!(rect.__base0));
        (*this_).ResizeChildren__1(rect.Width(), rect.Height());
    }
    pub unsafe fn ResizeChildren__1(&mut self, Width: i32, Height: i32) {
        let this_: *mut Self = self;
        let mut ScrnshotWidth: i32 = (*(*this_).ScrnshotPrev).GetRequestedWidth(Height);

        // make sure richedit always have room to display
        ScrnshotWidth = min!(ScrnshotWidth, Width - (INFO_DISPLAY_PADDING as i32) - (RICHEDIT_MIN_WIDTH as i32));

        let mut dwError: DWORD = ERROR_SUCCESS;
        let mut hDwp: HDWP = BeginDeferWindowPos(2);

        if !hDwp.is_null() {
            hDwp = DeferWindowPos(hDwp, (*(*this_).ScrnshotPrev).__base0.m_hWnd, core::ptr::null_mut(), 0, 0, ScrnshotWidth, Height, 0);

            if !hDwp.is_null() {
                // hide the padding if scrnshot window width == 0
                let RicheditPosX: i32 = if ScrnshotWidth != 0 {
                    ScrnshotWidth + (INFO_DISPLAY_PADDING as i32)
                } else {
                    0
                };

                hDwp = DeferWindowPos(hDwp, (*(*this_).RichEdit).__base0.__base2.__base0.m_hWnd, core::ptr::null_mut(), RicheditPosX, 0, Width - RicheditPosX, Height, 0);

                if !hDwp.is_null() {
                    EndDeferWindowPos(hDwp);
                } else {
                    dwError = GetLastError();
                }
            } else {
                dwError = GetLastError();
            }
        } else {
            dwError = GetLastError();
        }

        ATLASSERT!(dwError == ERROR_SUCCESS);
        UNREFERENCED_PARAMETER!(dwError);

        (*this_).__base0.__base2.UpdateWindow();
    }
    pub unsafe fn GetWndClassInfo() -> &'static mut CWndClassInfo {
        const csStyle: DWORD = CS_VREDRAW | CS_HREDRAW;
        static mut wc: CWndClassInfo = CWndClassInfo {
            m_wc: WNDCLASSEXW {
                cbSize: core::mem::size_of::<WNDCLASSEX>() as UINT,
                style: csStyle,
                lpfnWndProc: StartWindowProc,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: core::ptr::null_mut(),
                hIcon: core::ptr::null_mut(),
                hCursor: core::ptr::null_mut(),
                hbrBackground: core::ptr::without_provenance_mut::<core::ffi::c_void>(
                    (COLOR_BTNFACE + 1) as usize,
                ) as HBRUSH,
                lpszMenuName: core::ptr::null_mut(),
                lpszClassName: CPP_COM_WSTR_BE3990BCF6A7.as_ptr(),
                hIconSm: core::ptr::null_mut(),
            },
            m_lpszOrigName: core::ptr::null_mut(),
            pWndProc: None,
            m_lpszCursorID: IDC_ARROW,
            m_bSystemCursor: (TRUE as i32),
            m_atom: 0,
            m_szAutoName: CPP_COM_WSTR_96A296D224F2.as_ptr(),
        };
        return &mut *core::ptr::addr_of_mut!(wc);
    }
    pub unsafe fn Create(&mut self, hwndParent: HWND) -> HWND {
        let this_: *mut Self = self;
        let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };

        return (*this_).CWindowImpl_Create(hwndParent, &mut r, CPP_COM_WSTR_96A296D224F2.as_ptr(), WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS, 0, 0);
    }
    pub unsafe fn ShowAppInfo(&mut self, Info: &mut CAppInfo, OnlyUpdateText: bool) {
        let this_: *mut Self = self;
        if !OnlyUpdateText {
            let mut ScrnshotLocation = CStringW::new();
            if (*Info).RetrieveScreenshot(&mut ScrnshotLocation) != 0 {
                (*(*this_).ScrnshotPrev).DisplayImage((&mut ScrnshotLocation).GetString());
            } else {
                (*(*this_).ScrnshotPrev).DisplayEmpty();
            }
        }
        (*this_).ResizeChildren__0();
        Info.ShowAppInfo((*this_).RichEdit);
    }
    pub unsafe fn SetWelcomeText(&mut self, bAppwiz: bool) {
        let this_: *mut Self = self;
        let mut szText = CStringW::new();

        (*(*this_).ScrnshotPrev).DisplayEmpty();
        (*this_).ResizeChildren__0();

        // Display the standard banner in normal mode, or
        // the specific "Add/Remove Programs" in APPWIZ-mode.
        if !bAppwiz {
            szText.LoadStringW(IDS_WELCOME_TITLE);
            (*(*this_).RichEdit).__base0.__base2.SetText__1(&szText, CFE_BOLD);
            (*(*this_).RichEdit).__base0.__base2.InsertText__0(CPP_COM_WSTR_9D313EA1711F.as_ptr(), 0);

            szText.LoadStringW(IDS_WELCOME_TEXT);
            (*(*this_).RichEdit).__base0.__base2.InsertText__1(&szText, 0);

            szText.LoadStringW(IDS_WELCOME_URL);
            (*(*this_).RichEdit).__base0.__base2.InsertText__1(&szText, CFM_LINK);
        } else {
            szText.LoadStringW(IDS_APPWIZ_TITLE);
            (*(*this_).RichEdit).__base0.__base2.SetText__1(&szText, CFE_BOLD);
            (*(*this_).RichEdit).__base0.__base2.InsertText__0(CPP_COM_WSTR_9D313EA1711F.as_ptr(), 0);

            szText.LoadStringW(IDS_APPWIZ_TEXT1);
            (*(*this_).RichEdit).__base0.__base2.InsertText__1(&szText, 0);
            (*(*this_).RichEdit).__base0.__base2.InsertText__0(CPP_COM_WSTR_075DE2B906DB.as_ptr(), 0);

            szText.LoadStringW(IDS_APPWIZ_TEXT2);
            (*(*this_).RichEdit).__base0.__base2.InsertText__1(&szText, 0);
        }
    }
    pub unsafe fn OnCommand(&mut self, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        let wCommand: WORD = LOWORD!(wParam);

        match (wCommand) as u32 {
            ID_OPEN_LINK => {
                ShellExecuteW((*this_).__base0.__base2.m_hWnd, CPP_COM_WSTR_82DB442E7A60.as_ptr(), (*this_).pLink, core::ptr::null_mut(), core::ptr::null_mut(), (SW_SHOWNOACTIVATE) as INT);
                HeapFree(GetProcessHeap(), 0, ((*this_).pLink).cast());
                (*this_).pLink = core::ptr::null_mut();
            }

            ID_COPY_LINK => {
                CopyTextToClipboard((*this_).pLink);
                HeapFree(GetProcessHeap(), 0, ((*this_).pLink).cast());
                (*this_).pLink = core::ptr::null_mut();
            }

            _ => {}
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        // C source was malformed: `CAppRichEdit__delete(RichEdit)ichEdit;` and
        // `CAppScrnshotPreview__delete(ScrnshotPrev)hotPrev;`. The stray trailing
        // identifiers (`ichEdit`, `hotPrev`) are dropped; the member accesses are
        // restored through `this_` per the class mapping.
        CAppRichEdit__delete((*this_).RichEdit);
        CAppScrnshotPreview__delete((*this_).ScrnshotPrev);
    }
}

impl CAppsListView {
    pub unsafe fn OnEraseBackground(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut lRes: LRESULT = (*this_).DefWindowProc(uMsg, wParam, lParam);
        if !(*this_).m_Watermark.IsEmpty() {
            let mut rc: RECT = core::mem::zeroed();
            (*this_).__base0.__base2.__base0.GetClientRect(core::ptr::addr_of_mut!(rc));
            let oldFont: HGDIOBJ = SelectFont!(wParam as HDC, GetStockFont!(DEFAULT_GUI_FONT));
            DrawShadowText(
                wParam as HDC, (*this_).m_Watermark.GetString(), ((*this_).m_Watermark.GetLength()) as UINT, core::ptr::addr_of_mut!(rc),
                DT_CENTER | DT_VCENTER | DT_NOPREFIX | DT_SINGLELINE, GetSysColor((COLOR_GRAYTEXT as i32)),
                GetSysColor((COLOR_GRAYTEXT as i32)), 1, 1);
            SelectFont!(wParam as HDC, oldFont);
        }
        lRes
    }
    pub unsafe fn OnAsyncIcon(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let task: *mut CAsyncLoadIcon = lParam as *mut CAsyncLoadIcon;
        *bHandled = (TRUE as i32);
        if (*task).TaskId == g_AsyncIconTaskId {
            let mut lvi: LVITEM = core::mem::zeroed();
            let mut lvfi: LVFINDINFO = core::mem::zeroed();
            lvfi.flags = LVFI_PARAM;
            lvfi.lParam = (*task).AppInfo as LPARAM;
            lvi.iItem = ListView_FindItem!((*this_).__base0.__base2.__base0.m_hWnd, -1, core::ptr::addr_of_mut!(lvfi));
            if lvi.iItem != -1 {
                lvi.iImage = ImageList_AddIcon!((*this_).m_hImageListView, wParam as HICON);
                if lvi.iImage != -1 {
                    lvi.mask = LVIF_IMAGE;
                    lvi.iSubItem = 0;
                    ListView_SetItem!((*this_).__base0.__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(lvi));
                }
            }
        }
        return 0;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).bIsAscending = (TRUE as i32);
        (*this_).bHasCheckboxes = false;
        (*this_).ItemCount = 0;
        (*this_).CheckedItemCount = 0;
        (*this_).ColumnCount = 0;
        (*this_).nLastHeaderID = -1;
        (*this_).ApplicationViewType = AppViewTypeAvailableApps;
        (*this_).m_hImageListView = core::ptr::null_mut();
        {
            (*this_).m_hImageListView = core::ptr::null_mut();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_hImageListView.is_null() {
            ImageList_Destroy((*this_).m_hImageListView);
        }
        if !g_hDefaultPackageIcon.is_null() {
            DestroyIcon(g_hDefaultPackageIcon);
        }
    }
    pub unsafe fn SetWatermark(&mut self, Text: &CStringW) {
        let this_: *mut Self = self;
        (*this_).m_Watermark = Text.clone();
    }
    pub unsafe fn ShowCheckboxes(&mut self, bShow: bool) {
        let this_: *mut Self = self;
        (*this_).__base0.__base2.SetExtendedListViewStyle(
            (if bShow { LVS_EX_CHECKBOXES } else { 0 }) | LVS_EX_FULLROWSELECT,
            0,
        );
        (*this_).bHasCheckboxes = bShow;
    }
    pub unsafe fn ColumnClick(&mut self, pnmv: LPNMLISTVIEW) {
        let this_: *mut Self = self;
        let mut hHeader: HWND;
        let mut hColumn: HDITEMW = core::mem::zeroed();
        let mut nHeaderID: INT = (*pnmv).iSubItem;

        if ((*this_).__base0.__base2.__base0.GetWindowLongPtr(GWL_STYLE) & (!LVS_NOSORTHEADER) as _) == 0 {
            return;
        }

        hHeader = ((*this_).__base0.__base2.__base0.SendMessageW__0(LVM_GETHEADER, 0, 0)) as HWND;
        ZeroMemory!(core::ptr::addr_of_mut!(hColumn) as *mut _, core::mem::size_of::<HDITEMW>());

        /* If the sorting column changed, remove the sorting style from the old column */
        if ((*this_).nLastHeaderID != -1) && ((*this_).nLastHeaderID != nHeaderID) {
            (*this_).bIsAscending = (TRUE as i32); // also reset sorting method to ascending
            hColumn.mask = HDI_FORMAT;
            Header_GetItem!(hHeader, (*this_).nLastHeaderID, core::ptr::addr_of_mut!(hColumn));
            hColumn.fmt &= (!(HDF_SORTUP | HDF_SORTDOWN)) as i32;
            Header_SetItem!(hHeader, (*this_).nLastHeaderID, core::ptr::addr_of_mut!(hColumn));
        }

        /* Set the sorting style to the new column */
        hColumn.mask = HDI_FORMAT;
        Header_GetItem!(hHeader, nHeaderID, core::ptr::addr_of_mut!(hColumn));

        hColumn.fmt &= (!(HDF_SORTDOWN | HDF_SORTUP)) as i32;
        hColumn.fmt |= (if (*this_).bIsAscending != 0 { HDF_SORTUP } else { HDF_SORTDOWN }) as i32;
        Header_SetItem!(hHeader, nHeaderID, core::ptr::addr_of_mut!(hColumn));

        /* Sort the list, using the current values of nHeaderID and bIsAscending */
        let mut ctx: SortContext = SortContext { lvw: this_, iSubItem: nHeaderID };
        (*this_).__base0.__base2.SortItems(Some(CAppsListView::s_CompareFunc), (core::ptr::addr_of_mut!(ctx)).cast());

        /* Save new values */
        (*this_).nLastHeaderID = nHeaderID;
        (*this_).bIsAscending = ((*this_).bIsAscending == 0) as BOOL;
    }
    pub unsafe fn AddColumn(&mut self, Index: INT, Text: &mut CStringW, Width: INT, Format: INT) -> BOOL {
        let this_: *mut Self = self;
        let mut Column: LVCOLUMNW = core::mem::zeroed();

        ZeroMemory!(core::ptr::addr_of_mut!(Column) as *mut core::ffi::c_void, core::mem::size_of::<LVCOLUMNW>());

        Column.mask = LVCF_FMT | LVCF_TEXT | LVCF_WIDTH | LVCF_SUBITEM;
        Column.iSubItem = Index;
        Column.pszText = Text.GetString() as *mut WCHAR;
        Column.cx = Width;
        Column.fmt = Format;

        return (*this_).__base0.__base2.__base0.SendMessageW__0((LVM_INSERTCOLUMN) as UINT, Index as WPARAM, core::ptr::addr_of_mut!(Column) as LPARAM) as BOOL;
    }
    pub unsafe fn DeleteColumn(&mut self, Index: INT) {
        let this_: *mut Self = self;
        (*this_).__base0.__base2.__base0.SendMessageW__0(LVM_DELETECOLUMN, (Index) as WPARAM, 0);
        return;
    }
    pub unsafe fn AddItem(&mut self, ItemIndex: INT, IconIndex: INT, lpText: LPCWSTR, lParam: LPARAM) -> INT {
        let this_: *mut Self = self;
        let mut Item: LVITEMW = core::mem::zeroed();

        ZeroMemory!(core::ptr::addr_of_mut!(Item) as *mut core::ffi::c_void, core::mem::size_of::<LVITEMW>());

        Item.mask = LVIF_TEXT | LVIF_PARAM | LVIF_STATE;
        Item.pszText = lpText as LPWSTR;
        Item.lParam = lParam;
        Item.iItem = ItemIndex;
        Item.iImage = IconIndex;

        if IconIndex >= 0 {
            Item.iImage = IconIndex;
            Item.mask |= LVIF_IMAGE;
        }
        return (*this_).__base0.__base2.InsertItem(core::ptr::addr_of_mut!(Item));
    }
    pub unsafe fn GetImageList(&mut self, iImageList: i32) -> HIMAGELIST {
        let this_: *mut Self = self;
        return (*this_).__base0.__base2.__base0.SendMessageW__0(LVM_GETIMAGELIST, iImageList as WPARAM, 0) as HIMAGELIST;
    }
    pub unsafe extern "system" fn s_CompareFunc(lParam1: LPARAM, lParam2: LPARAM, lParamSort: LPARAM) -> INT {
        let ctx: *mut SortContext = (lParamSort as *mut SortContext);
        return (*(*ctx).lvw).CompareFunc(lParam1, lParam2, (*ctx).iSubItem);
    }
    pub unsafe fn CompareFunc(&mut self, lParam1: LPARAM, lParam2: LPARAM, iSubItem: INT) -> INT {
        let this_: *mut Self = self;
        let mut Item1: CStringW = CStringW::new();
        let mut Item2: CStringW = CStringW::new();
        let mut IndexInfo: LVFINDINFOW = core::mem::zeroed();
        let mut Index: INT;

        IndexInfo.flags = LVFI_PARAM;

        IndexInfo.lParam = lParam1;
        Index = (*this_).__base0.__base2.FindItem(-1, core::ptr::addr_of_mut!(IndexInfo));
        (*this_).__base0.__base2.GetItemText(Index, iSubItem, Item1.GetBuffer((MAX_STR_LEN as i32)), (MAX_STR_LEN as i32));
        Item1.ReleaseBuffer();

        IndexInfo.lParam = lParam2;
        Index = (*this_).__base0.__base2.FindItem(-1, core::ptr::addr_of_mut!(IndexInfo));
        (*this_).__base0.__base2.GetItemText(Index, iSubItem, Item2.GetBuffer((MAX_STR_LEN as i32)), (MAX_STR_LEN as i32));
        Item2.ReleaseBuffer();

        return if (*this_).bIsAscending != 0 {
            Item1.Compare(Item2.GetString())
        } else {
            Item2.Compare(Item1.GetString())
        };
    }
    pub unsafe fn Create(&mut self, hwndParent: HWND) -> HWND {
        let this_: *mut Self = self;
        let mut r: RECT = RECT {
            left: 205,
            top: 28,
            right: 465,
            bottom: 250,
        };
        let style: DWORD = WS_CHILD | WS_VISIBLE | LVS_SORTASCENDING | LVS_REPORT | LVS_SINGLESEL | LVS_SHOWSELALWAYS |
                      LVS_AUTOARRANGE | LVS_SHAREIMAGELISTS;

        let hwnd: HWND = (*this_).__base0.__base2.Create(
            hwndParent,
            r,
            core::ptr::null(),
            style,
            WS_EX_CLIENTEDGE,
            0,
            core::ptr::null_mut(),
        );

        if !hwnd.is_null() {
            (*this_).ShowCheckboxes(false);
        }

        // #pragma push_macro("SubclassWindow") / #undef SubclassWindow / pop_macro:
        // preprocessor-only directive, no runtime effect; the macro is not expanded here.
        (*this_).__base0.__base2.__base0.m_hWnd = core::ptr::null_mut();
        (*this_).SubclassWindow(hwnd);

        return hwnd;
    }
    pub unsafe fn GetCheckState(&mut self, item: INT) -> BOOL {
        let this_: *mut Self = self;
        return ((((*this_).__base0.__base2.GetItemState(item, LVIS_STATEIMAGEMASK)) >> 12) as BOOL) - 1;
    }
    pub unsafe fn SetCheckState(&mut self, item: INT, fCheck: BOOL) {
        let this_: *mut Self = self;
        if (*this_).bHasCheckboxes {
            (*this_).__base0.__base2.SetItemState(item, INDEXTOSTATEIMAGEMASK!(if ((fCheck) != 0) { 2 } else { 1 }), LVIS_STATEIMAGEMASK);
        }
    }
    pub unsafe fn CheckAll(&mut self) {
        let this_: *mut Self = self;
        if (*this_).bHasCheckboxes {
            if (*this_).CheckedItemCount == (*this_).ItemCount {
                // clear all
                (*this_).SetCheckState(-1, (FALSE as i32));
            } else {
                // check all
                (*this_).SetCheckState(-1, (TRUE as i32));
            }
        }
    }
    pub unsafe fn GetFocusedItemData(&mut self) -> PVOID {
        let this_: *mut Self = self;
        let item: INT = (*this_).__base0.__base2.GetSelectionMark();
        if item == -1 {
            return 0 as PVOID;
        }
        return (*this_).__base0.__base2.GetItemData(item) as PVOID;
    }
    pub unsafe fn SetDisplayAppType(&mut self, AppType: APPLICATION_VIEW_TYPE) -> BOOL {
        let this_: *mut Self = self;
        g_AsyncIconTaskId += 1; // Stop loading icons that are now invalid
        if (*this_).__base0.__base2.DeleteAllItems() == 0 {
            return (FALSE) as BOOL;
        }

        (*this_).ApplicationViewType = AppType;
        (*this_).bIsAscending = (TRUE as i32);
        (*this_).ItemCount = 0;
        (*this_).CheckedItemCount = 0;

        ListView_Scroll!((*this_).__base0.__base2.__base0.m_hWnd, 0, 0x7fff * -1); // FIXME: a bug in Wine ComCtl32 where VScroll is not reset after deleting items

        // delete old columns
        while (*this_).ColumnCount != 0 {
            (*this_).ColumnCount -= 1;
            (*this_).DeleteColumn((*this_).ColumnCount);
        }

        if g_hDefaultPackageIcon.is_null() {
            ImageList_Destroy((*this_).m_hImageListView);
            g_IconSize = GetSystemMetrics(if ((SettingsInfo.bSmallIcons) != 0) { (SM_CXSMICON as i32) } else { (SM_CXICON as i32) }) as UINT;
            g_IconSize = max!(g_IconSize, 8);
            let ilc: UINT = (GetSystemColorDepth() | (ILC_MASK as i32)) as u32;
            (*this_).m_hImageListView = ImageList_Create((g_IconSize) as i32, (g_IconSize) as i32, ilc, 0, 1);
            (*this_).__base0.__base2.SetImageList((*this_).m_hImageListView, (LVSIL_SMALL as i32));
            (*this_).__base0.__base2.SetImageList((*this_).m_hImageListView, (LVSIL_NORMAL as i32));
            g_hDefaultPackageIcon = LoadImageW(hInst, MAKEINTRESOURCEW!(IDI_MAIN),
                                                      IMAGE_ICON, g_IconSize as i32, g_IconSize as i32, LR_SHARED) as HICON;
        }
        ImageList_RemoveAll!((*this_).m_hImageListView);

        g_DefaultPackageIconILIdx = ImageList_AddIcon!((*this_).m_hImageListView, g_hDefaultPackageIcon);
        if g_DefaultPackageIconILIdx == -1 {
            g_DefaultPackageIconILIdx = I_IMAGENONE;
        }

        // add new columns
        let mut szText: CStringW = CStringW::new();
        match AppType {
            AppViewTypeInstalledApps => {

                /* Add columns to ListView */
                szText.LoadStringW(IDS_APP_NAME);
                let idx = (*this_).ColumnCount;
                (*this_).ColumnCount += 1;
                (*this_).AddColumn(idx, &mut szText, 368, (LVCFMT_LEFT as i32));

                szText.LoadStringW(IDS_APP_INST_VERSION);
                let idx = (*this_).ColumnCount;
                (*this_).ColumnCount += 1;
                (*this_).AddColumn(idx, &mut szText, 90, (LVCFMT_RIGHT as i32));

                // Disable checkboxes
                (*this_).ShowCheckboxes(false);
            }

            AppViewTypeAvailableApps => {

                /* Add columns to ListView */
                szText.LoadStringW(IDS_APP_NAME);
                let idx = (*this_).ColumnCount;
                (*this_).ColumnCount += 1;
                (*this_).AddColumn(idx, &mut szText, 250, (LVCFMT_LEFT as i32));

                szText.LoadStringW(IDS_APP_INST_VERSION);
                let idx = (*this_).ColumnCount;
                (*this_).ColumnCount += 1;
                (*this_).AddColumn(idx, &mut szText, 90, (LVCFMT_RIGHT as i32));

                szText.LoadStringW(IDS_APP_DESCRIPTION);
                let idx = (*this_).ColumnCount;
                (*this_).ColumnCount += 1;
                (*this_).AddColumn(idx, &mut szText, 300, (LVCFMT_LEFT as i32));

                // Enable checkboxes
                (*this_).ShowCheckboxes(true);
            }

            _ => {}
        }

        return (TRUE) as BOOL;
    }
    pub unsafe fn SetViewMode(&mut self, ViewMode: DWORD) -> BOOL {
        let this_: *mut Self = self;
        return ((*this_).__base0.__base2.__base0.SendMessageW__0(LVM_SETVIEW, ViewMode as WPARAM, 0) == 1) as BOOL;
    }
    pub unsafe fn AddApplication(&mut self, AppInfo: *mut CAppInfo, InitialCheckState: BOOL) -> BOOL {
        let this_: *mut Self = self;
        if AppInfo.is_null() {
            CAsyncLoadIcon::StartTasks();
            return (TRUE) as BOOL;
        }

        let mut IconIndex: INT = g_DefaultPackageIconILIdx;
        if (*this_).ApplicationViewType == AppViewTypeInstalledApps {
            let Index: INT = (*this_).AddItem((*this_).ItemCount, IconIndex, (*AppInfo).szDisplayName.GetString(), AppInfo as LPARAM);
            if Index == -1 {
                return (FALSE) as BOOL;
            }
            CAsyncLoadIcon::Queue((*this_).__base0.__base2.__base0.m_hWnd, *AppInfo, true);

            (*this_).__base0.__base2.SetItemText(Index, 1, if (*AppInfo).szDisplayVersion.IsEmpty() { CPP_COM_WSTR_D4C31F2B2950.as_ptr() } else { (*AppInfo).szDisplayVersion.GetString() });
            (*this_).__base0.__base2.SetItemText(Index, 2, if (*AppInfo).szComments.IsEmpty() { CPP_COM_WSTR_D4C31F2B2950.as_ptr() } else { (*AppInfo).szComments.GetString() });

            (*this_).ItemCount += 1;
            return (TRUE) as BOOL;
        } else if (*this_).ApplicationViewType == AppViewTypeAvailableApps {
            let Index: INT = (*this_).AddItem((*this_).ItemCount, IconIndex, (*AppInfo).szDisplayName.GetString(), AppInfo as LPARAM);
            if Index == -1 {
                return (FALSE) as BOOL;
            }
            CAsyncLoadIcon::Queue((*this_).__base0.__base2.__base0.m_hWnd, *AppInfo, false);

            if InitialCheckState != 0 {
                (*this_).SetCheckState(Index, (TRUE as i32));
            }

            (*this_).__base0.__base2.SetItemText(Index, 1, (*AppInfo).szDisplayVersion.GetString());
            (*this_).__base0.__base2.SetItemText(Index, 2, (*AppInfo).szComments.GetString());

            (*this_).ItemCount += 1;
            return (TRUE) as BOOL;
        } else {
            return (FALSE) as BOOL;
        }
    }
    pub unsafe fn ItemCheckStateNotify(&mut self, iItem: i32, bCheck: BOOL) {
        let this_: *mut Self = self;
        if bCheck != 0 {
            (*this_).CheckedItemCount += 1;
        } else {
            (*this_).CheckedItemCount -= 1;
        }
    }
}

impl CMainToolbar {
    pub unsafe fn AddImageToImageList(&mut self, hImageList: HIMAGELIST, ImageIndex: UINT) {
        let this_: *mut Self = self;
        let mut hImage: HICON;

        hImage = LoadImageW(hInst, MAKEINTRESOURCE!(ImageIndex), IMAGE_ICON, (*this_).m_iToolbarHeight, (*this_).m_iToolbarHeight, 0) as HICON;
        if hImage.is_null() {
            return;
        }

        ImageList_AddIcon!(hImageList, hImage);
        DeleteObject(hImage as HGDIOBJ);
    }
    pub unsafe fn InitImageList(&mut self) -> HIMAGELIST {
        let this_: *mut Self = self;
        let mut hImageList: HIMAGELIST;

        /* Create the toolbar icon image list */
        hImageList = ImageList_Create((*this_).m_iToolbarHeight, (*this_).m_iToolbarHeight, ILC_MASK | (GetSystemColorDepth()) as u32, 1, 1);
        if hImageList.is_null()
        {
            return core::ptr::null_mut();
        }

        (*this_).AddImageToImageList(hImageList, IDI_INSTALL);
        (*this_).AddImageToImageList(hImageList, IDI_UNINSTALL);
        (*this_).AddImageToImageList(hImageList, IDI_MODIFY);
        (*this_).AddImageToImageList(hImageList, IDI_CHECK_ALL);
        (*this_).AddImageToImageList(hImageList, IDI_REFRESH);
        (*this_).AddImageToImageList(hImageList, IDI_UPDATE_DB);
        (*this_).AddImageToImageList(hImageList, IDI_SETTINGS);
        (*this_).AddImageToImageList(hImageList, IDI_EXIT);

        return hImageList;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).m_iToolbarHeight = 24;
        (*this_).m_dButtonsWidthMax = 0;
        {
        }
    }
    pub unsafe fn OnGetDispInfo(&mut self, lpttt: LPNMTTDISPINFOW) {
        let this_: *mut Self = self;
        let idButton: UINT = (*lpttt).hdr.idFrom as UINT;

        match idButton {
            ID_EXIT => {
                (*lpttt).lpszText = MAKEINTRESOURCEW!(IDS_TOOLTIP_EXIT);
            }

            ID_INSTALL => {
                (*lpttt).lpszText = MAKEINTRESOURCEW!(IDS_TOOLTIP_INSTALL);
            }

            ID_UNINSTALL => {
                (*lpttt).lpszText = MAKEINTRESOURCEW!(IDS_TOOLTIP_UNINSTALL);
            }

            ID_MODIFY => {
                (*lpttt).lpszText = MAKEINTRESOURCEW!(IDS_TOOLTIP_MODIFY);
            }

            ID_SETTINGS => {
                (*lpttt).lpszText = MAKEINTRESOURCEW!(IDS_TOOLTIP_SETTINGS);
            }

            ID_REFRESH => {
                (*lpttt).lpszText = MAKEINTRESOURCEW!(IDS_TOOLTIP_REFRESH);
            }

            ID_RESETDB => {
                (*lpttt).lpszText = MAKEINTRESOURCEW!(IDS_TOOLTIP_UPDATE_DB);
            }

            _ => {}
        }
    }
    pub unsafe fn Create(&mut self, hwndParent: HWND) -> HWND {
        let this_: *mut Self = self;
        let mut szInstallBtn = CStringW::new();
        let mut szUninstallBtn = CStringW::new();
        let mut szModifyBtn = CStringW::new();
        let mut szSelectAllBtn = CStringW::new();
        let mut szRefreshBtn = CStringW::new();
        let mut szUpdateDbBtn = CStringW::new();

        /* Load tooltip strings */
        szInstallBtn.LoadStringW(IDS_TOOLTIP_INSTALL);
        szUninstallBtn.LoadStringW(IDS_TOOLTIP_UNINSTALL);
        szModifyBtn.LoadStringW(IDS_TOOLTIP_MODIFY);
        szSelectAllBtn.LoadStringW(IDS_TOOLTIP_SELECT_ALL);
        szRefreshBtn.LoadStringW(IDS_TOOLTIP_REFRESH);
        szUpdateDbBtn.LoadStringW(IDS_TOOLTIP_UPDATE_DB);

        /* Create buttons */
        let Buttons: [TBBUTTON; 7] = [
            TBBUTTON {
                iBitmap: 0,
                idCommand: (ID_INSTALL as i32),
                fsState: (TBSTATE_ENABLED as u8),
                fsStyle: (BTNS_BUTTON | BTNS_AUTOSIZE) as u8,
                bReserved: [0],
                dwData: 0,
                iString: szInstallBtn.GetString() as INT_PTR,
            },
            TBBUTTON {
                iBitmap: 1,
                idCommand: (ID_UNINSTALL as i32),
                fsState: (TBSTATE_ENABLED as u8),
                fsStyle: (BTNS_BUTTON | BTNS_AUTOSIZE) as u8,
                bReserved: [0],
                dwData: 0,
                iString: szUninstallBtn.GetString() as INT_PTR,
            },
            TBBUTTON {
                iBitmap: 2,
                idCommand: (ID_MODIFY as i32),
                fsState: (TBSTATE_ENABLED as u8),
                fsStyle: (BTNS_BUTTON | BTNS_AUTOSIZE) as u8,
                bReserved: [0],
                dwData: 0,
                iString: szModifyBtn.GetString() as INT_PTR,
            },
            TBBUTTON {
                iBitmap: 3,
                idCommand: (ID_CHECK_ALL as i32),
                fsState: (TBSTATE_ENABLED as u8),
                fsStyle: (BTNS_BUTTON | BTNS_AUTOSIZE) as u8,
                bReserved: [0],
                dwData: 0,
                iString: szSelectAllBtn.GetString() as INT_PTR,
            },
            TBBUTTON {
                iBitmap: -1,
                idCommand: 0,
                fsState: (TBSTATE_ENABLED as u8),
                fsStyle: (BTNS_SEP as u8),
                bReserved: [0],
                dwData: 0,
                iString: 0,
            },
            TBBUTTON {
                iBitmap: 4,
                idCommand: (ID_REFRESH as i32),
                fsState: (TBSTATE_ENABLED as u8),
                fsStyle: (BTNS_BUTTON | BTNS_AUTOSIZE) as u8,
                bReserved: [0],
                dwData: 0,
                iString: szRefreshBtn.GetString() as INT_PTR,
            },
            TBBUTTON {
                iBitmap: 5,
                idCommand: (ID_RESETDB as i32),
                fsState: (TBSTATE_ENABLED as u8),
                fsStyle: (BTNS_BUTTON | BTNS_AUTOSIZE) as u8,
                bReserved: [0],
                dwData: 0,
                iString: szUpdateDbBtn.GetString() as INT_PTR,
            },
        ];

        (*this_).__base0.__base2.__base0.m_hWnd = CreateWindowExW(
            0,
            TOOLBARCLASSNAMEW,
            core::ptr::null_mut(),
            WS_CHILD | WS_VISIBLE | TBSTYLE_FLAT | TBSTYLE_TOOLTIPS | TBSTYLE_LIST,
            0,
            0,
            0,
            0,
            hwndParent,
            core::ptr::null_mut(),
            hInst,
            core::ptr::null_mut(),
        );

        if (*this_).__base0.__base2.__base0.m_hWnd.is_null() {
            return core::ptr::null_mut();
        }

        (*this_).__base0.__base2.__base0.SendMessageW__0(TB_SETEXTENDEDSTYLE, 0, (TBSTYLE_EX_HIDECLIPPEDBUTTONS) as LPARAM);
        (*this_).__base0.__base2.SetButtonStructSize();

        /* Set image list */
        let hImageList: HIMAGELIST = (*this_).InitImageList();

        if !hImageList.is_null() {
            ImageList_Destroy((*this_).__base0.__base2.SetImageList(hImageList));
        }

        (*this_).__base0.__base2.AddButtons((_countof!(Buttons)) as i32, Buttons.as_ptr() as *mut TBBUTTON);

        /* Remember the ideal width to use as a max width of buttons */
        (*this_).UpdateMaxButtonsWidth();

        (*this_).__base0.__base2.__base0.m_hWnd
    }
    pub unsafe fn ShowButtonCaption(&mut self, bShow: bool) {
        let this_: *mut Self = self;
        let dCurrentExStyle: DWORD = (*this_).__base0.__base2.__base0.SendMessageW__0(TB_GETEXTENDEDSTYLE, 0, 0) as DWORD;
        if bShow {
            (*this_).__base0.__base2.__base0.SendMessageW__0(TB_SETEXTENDEDSTYLE, 0, (dCurrentExStyle & !TBSTYLE_EX_MIXEDBUTTONS) as LPARAM);
        } else {
            (*this_).__base0.__base2.__base0.SendMessageW__0(TB_SETEXTENDEDSTYLE, 0, (dCurrentExStyle | TBSTYLE_EX_MIXEDBUTTONS) as LPARAM);
        }
    }
    pub unsafe fn UpdateMaxButtonsWidth(&mut self) {
        let this_: *mut Self = self;
        let mut size: SIZE = core::mem::zeroed();
        (*this_).__base0.__base2.GetIdealSize((FALSE) as BOOL, (core::ptr::addr_of_mut!(size)).cast());
        (*this_).m_dButtonsWidthMax = size.cx as DWORD;
    }
    pub unsafe fn GetMaxButtonsWidth(&mut self) -> DWORD {
        let this_: *mut Self = self;
        (*this_).m_dButtonsWidthMax
    }
}

impl CSearchBar {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).m_Width = 180;
        (*this_).m_Height = 22;
        {
        }
    }
    pub unsafe fn SetText(&mut self, lpszText: LPCWSTR) {
        let this_: *mut Self = self;
        (*this_).__base0.SendMessageW__0((SB_SETTEXT) as UINT, (SBT_NOBORDERS) as WPARAM, lpszText as LPARAM);
    }
    pub unsafe fn Create(&mut self, hwndParent: HWND) -> HWND {
        let this_: *mut Self = self;
        let mut szBuf = CStringW::new();
        (*this_).__base0.m_hWnd = CreateWindowExW(
            WS_EX_CLIENTEDGE, CPP_COM_WSTR_2742CD8F7DE1.as_ptr(), core::ptr::null_mut(), WS_CHILD | WS_VISIBLE | ES_LEFT | ES_AUTOHSCROLL, 0, 0, (*this_).m_Width, (*this_).m_Height,
            hwndParent, core::ptr::null_mut() as HMENU, hInst, core::ptr::null_mut());

        (*this_).__base0.SendMessageW__0(WM_SETFONT, GetStockObject((DEFAULT_GUI_FONT as i32)) as WPARAM, 0);
        szBuf.LoadStringW(IDS_SEARCH_TEXT);
        (*this_).__base0.SetWindowTextW(szBuf.GetString());
        return (*this_).__base0.m_hWnd;
    }
}

impl CComboBox {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).m_TypeStringID = [IDS_APP_DISPLAY_DETAILS, IDS_APP_DISPLAY_LIST, IDS_APP_DISPLAY_TILE];
        (*this_).m_DefaultSelectType = (m_AppDisplayTypeDetails) as i32;
        (*this_).m_Width = 80;
        (*this_).m_Height = 22;
    }
    pub unsafe fn Create(&mut self, hwndParent: HWND) -> HWND {
        let this_: *mut Self = self;
        (*this_).__base0.m_hWnd = CreateWindowW!(
            WC_COMBOBOX, CPP_COM_WSTR_96A296D224F2.as_ptr(), CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_CHILD | WS_OVERLAPPED | WS_VISIBLE, 0, 0, (*this_).m_Width,
            (*this_).m_Height, hwndParent, core::ptr::null_mut(), 0, core::ptr::null_mut());

        (*this_).__base0.SendMessageW__0(WM_SETFONT, GetStockObject((DEFAULT_GUI_FONT as i32)) as WPARAM, 0);

        let mut i: i32 = 0;
        while i < (_countof!((*this_).m_TypeStringID) as i32) {
            let mut szBuf = CStringW::new();
            szBuf.LoadStringW((*this_).m_TypeStringID[i as usize]);
            (*this_).__base0.SendMessageW__0(CB_ADDSTRING, 0, szBuf.GetString() as LPARAM);
            i += 1;
        }

        (*this_).__base0.SendMessageW__0(CB_SETCURSEL, (*this_).m_DefaultSelectType as WPARAM, 0); // select the first item

        return (*this_).__base0.m_hWnd;
    }
}

impl CApplicationView {
    pub unsafe fn ProcessWindowMessage(&mut self, hwnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM, theResult: &mut LRESULT, dwMapId: DWORD) -> BOOL {
        let this_: *mut Self = self;
        *theResult = 0;
        match message {
            WM_CREATE => {
                let mut bSuccess: BOOL = (TRUE as i32);
                (*this_).m_Panel = CUiPanel__new_0();
                (*(*this_).m_Panel).__base1.m_VerticalAlignment = UiAlign_Stretch;
                (*(*this_).m_Panel).__base1.m_HorizontalAlignment = UiAlign_Stretch;

                bSuccess &= (*this_).CreateToolbar();
                bSuccess &= (*this_).CreateSearchBar();
                bSuccess &= (*this_).CreateComboBox();
                bSuccess &= (*this_).CreateHSplitter();
                bSuccess &= (*this_).CreateListView();
                bSuccess &= (*this_).CreateAppInfoDisplay();

                /* APPWIZ-mode: Remove the unneeded menu items and toolbar buttons */
                if (*(*this_).m_MainWindow).m_bAppwizMode != 0 {
                    let mut hMenu: HMENU = core::ptr::null_mut();

                    /* Delete the "Settings" item in the "File" sub-menu */
                    hMenu = GetSubMenu((*(*this_).m_MainWindow).__base0.GetMenu(), 0);
                    DeleteMenuEx(hMenu, ID_SETTINGS, MF_BYCOMMAND);

                    /* Remove the menu items: ID_INSTALL, ID_RESETDB */
                    hMenu = (*this_).__base0.__base2.GetMenu();
                    DeleteMenuEx(hMenu, ID_INSTALL, MF_BYCOMMAND);
                    DeleteMenuEx(hMenu, ID_RESETDB, MF_BYCOMMAND);

                    /* Remove the toolbar buttons:
                     * ID_INSTALL, ID_CHECK_ALL, ID_RESETDB
                     * We only keep:
                     * ID_UNINSTALL, ID_MODIFY, ID_REFRESH */
                    let mut info: TBBUTTONINFOW = core::mem::zeroed();
                    info.cbSize = core::mem::size_of::<TBBUTTONINFOW>() as u32;
                    info.dwMask = 0;
                    let mut index: i32;

                    index = ((*(*this_).m_Toolbar).__base0.__base2.GetButtonInfo((ID_INSTALL as i32), core::ptr::addr_of_mut!(info))) as i32;
                    if index >= 0 {
                        (*(*this_).m_Toolbar).__base0.__base2.DeleteButton(index);
                    }

                    index = ((*(*this_).m_Toolbar).__base0.__base2.GetButtonInfo((ID_CHECK_ALL as i32), core::ptr::addr_of_mut!(info))) as i32;
                    if index >= 0 {
                        (*(*this_).m_Toolbar).__base0.__base2.DeleteButton(index);
                    }

                    index = ((*(*this_).m_Toolbar).__base0.__base2.GetButtonInfo((ID_RESETDB as i32), core::ptr::addr_of_mut!(info))) as i32;
                    if index >= 0 {
                        (*(*this_).m_Toolbar).__base0.__base2.DeleteButton(index);
                    }

                    /* Update the ideal width to use as a max width of buttons */
                    (*(*this_).m_Toolbar).UpdateMaxButtonsWidth();
                }

                /* Resize the toolbar */
                (*(*this_).m_Toolbar).__base0.__base2.AutoSize();

                let mut rTop: RECT = core::mem::zeroed();

                GetWindowRect((*(*this_).m_Toolbar).__base0.__base2.__base0.m_hWnd, core::ptr::addr_of_mut!(rTop));
                (*(*this_).m_HSplitter).__base1.m_Margin.top = rTop.bottom - rTop.top;
                if bSuccess == 0 {
                    return -1; // creation failure
                }
            }

            WM_NOTIFY => {
                let pNotifyHeader: LPNMHDR = lParam as usize as LPNMHDR;
                if (*pNotifyHeader).hwndFrom == ((*(*this_).m_ListView).__base0.GetWindow()).cast() {
                    'inner: {
                        match (((*pNotifyHeader).code) as i32) as u32 {
                            __c if __c == LVN_ITEMCHANGED as u32 => {
                                let pnic: LPNMLISTVIEW = lParam as usize as LPNMLISTVIEW;

                                /* Check if this is a valid item
                                 * (technically, it can be also an unselect) */
                                let ItemIndex: INT = (*pnic).iItem;
                                if ItemIndex == -1 || ItemIndex >= ListView_GetItemCount!((*pnic).hdr.hwndFrom) {
                                    break 'inner;
                                }

                                /* Check if the focus has been moved to another item */
                                if ((*pnic).uChanged & LVIF_STATE) != 0
                                    && ((*pnic).uNewState & LVIS_FOCUSED) != 0
                                    && !((*pnic).uOldState & LVIS_FOCUSED) != 0
                                {
                                    (*this_).ItemGetFocus((*pnic).lParam as usize as LPVOID);
                                }

                                /* Check if the item is checked/unchecked */
                                if ((*pnic).uChanged & LVIF_STATE) != 0 {
                                    let iOldState = STATEIMAGETOINDEX!((*pnic).uOldState);
                                    let iNewState = STATEIMAGETOINDEX!((*pnic).uNewState);

                                    if iOldState == STATEIMAGE_UNCHECKED && iNewState == STATEIMAGE_CHECKED {
                                        // this item is just checked
                                        (*(*this_).m_ListView).ItemCheckStateNotify((*pnic).iItem, (TRUE) as BOOL);
                                        (*this_).ItemCheckStateChanged((TRUE) as BOOL, (*pnic).lParam as usize as LPVOID);
                                    } else if iOldState == STATEIMAGE_CHECKED && iNewState == STATEIMAGE_UNCHECKED {
                                        // this item is just unchecked
                                        (*(*this_).m_ListView).ItemCheckStateNotify((*pnic).iItem, (FALSE) as BOOL);
                                        (*this_).ItemCheckStateChanged((FALSE) as BOOL, (*pnic).lParam as usize as LPVOID);
                                    }
                                }

                                /* Ensure that if there are any items still focused/selected,
                                 * the ID_INSTALL menu item and toolbar button stay enabled */
                                if ((*pnic).uChanged & LVIF_STATE) != 0 && (*(*this_).m_MainWindow).bUpdating == 0 {
                                    (*this_)._UpdateInstallBtn();
                                }
                            }

                            __c if __c == LVN_COLUMNCLICK as u32 => {
                                let pnmv: LPNMLISTVIEW = lParam as usize as LPNMLISTVIEW;

                                (*(*this_).m_ListView).ColumnClick(pnmv);
                            }

                            NM_DBLCLK => {
                                let Item: LPNMITEMACTIVATE = lParam as usize as LPNMITEMACTIVATE;
                                if (*Item).iItem != -1 {
                                    /* this won't do anything if the program is already installed */

                                    if (*this_).ApplicationViewType == AppViewTypeAvailableApps {
                                        (*(*this_).m_MainWindow).InstallApplication(
                                            (*(*this_).m_ListView).__base0.__base2.GetItemData((*Item).iItem) as *mut CAppInfo,
                                        );
                                    }
                                }
                            }

                            _ => {}
                        }
                    }
                } else if pNotifyHeader_is_toolbar(pNotifyHeader, this_) {
                    'inner2: {
                        match (*pNotifyHeader).code {
                            TTN_GETDISPINFO => {
                                (*(*this_).m_Toolbar).OnGetDispInfo(lParam as usize as LPTOOLTIPTEXT);
                            }
                            _ => {}
                        }
                    }
                }
            }

            WM_SYSCOLORCHANGE => {
                /* Forward WM_SYSCOLORCHANGE to common controls */
                (*(*this_).m_ListView).__base0.__base2.__base0.SendMessageW__0(WM_SYSCOLORCHANGE, wParam, lParam);
                (*(*this_).m_AppsInfo).__base0.__base2.SendMessageW__0(WM_SYSCOLORCHANGE, wParam, lParam);
                (*(*this_).m_Toolbar).__base0.__base2.__base0.SendMessageW__0(WM_SYSCOLORCHANGE, wParam, lParam);
                (*(*this_).m_ComboBox).__base2.__base0.SendMessageW__0(WM_SYSCOLORCHANGE, wParam, lParam);
            }

            WM_SIZE => {
                (*this_).OnSize(hwnd, wParam, lParam);
            }

            WM_COMMAND => {
                (*this_).OnCommand(wParam, lParam);
            }

            WM_CONTEXTMENU => {
                let kbd: bool = -1 == (lParam as isize as i32);
                if wParam as HWND == (*(*this_).m_ListView).__base0.__base2.__base0.m_hWnd {
                    let item: i32 = (*(*this_).m_ListView).__base0.__base2.GetNextItem(-1, (LVNI_FOCUSED | LVNI_SELECTED) as u16);
                    if item != -1 {
                        let mut ppt: *mut POINT = core::ptr::null_mut();
                        let mut pt: POINT = core::mem::zeroed();
                        if kbd {
                            let mut r: RECT = core::mem::zeroed();
                            ListView_GetItemRect!(wParam as HWND, item, core::ptr::addr_of_mut!(r), LVIR_LABEL);
                            pt.x = r.left + (r.right - r.left) / 2;
                            pt.y = r.top + (r.bottom - r.top) / 2;
                            ppt = core::ptr::addr_of_mut!(pt);
                            ClientToScreen(wParam as HWND, (ppt).cast());
                        }
                        ShowPopupMenuEx((*this_).__base0.__base2.m_hWnd, (*this_).__base0.__base2.m_hWnd, 0, ID_INSTALL, ppt);
                        return (TRUE) as BOOL;
                    }
                }
            }

            _ => {}
        }
        return (FALSE) as BOOL;
    }
    pub unsafe fn CreateToolbar(&mut self) -> i32 {
        let this_: *mut Self = self;
        (*this_).m_Toolbar = CMainToolbar__new_0();
        (*(*this_).m_Toolbar).__base0.__base1.m_VerticalAlignment = UiAlign_LeftTop;
        (*(*this_).m_Toolbar).__base0.__base1.m_HorizontalAlignment = UiAlign_Stretch;
        (*(*(*this_).m_Panel).__base2.Children()).__base0.Append(core::ptr::addr_of_mut!((*(*this_).m_Toolbar).__base0.__base0));

        ((*(*this_).m_Toolbar).Create((*this_).__base0.__base2.m_hWnd) != core::ptr::null_mut()) as i32
    }
    pub unsafe fn CreateSearchBar(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_SearchBar = CUiWindow_CSearchBar__new_0::<CSearchBar>();
        (*(*this_).m_SearchBar).__base1.m_VerticalAlignment = UiAlign_LeftTop;
        (*(*this_).m_SearchBar).__base1.m_HorizontalAlignment = UiAlign_RightBtm;
        (*(*this_).m_SearchBar).__base1.m_Margin.top = 4;
        (*(*this_).m_SearchBar).__base1.m_Margin.right = (TOOLBAR_PADDING as i32);

        return ((*(*this_).m_SearchBar).__base2.Create((*(*this_).m_Toolbar).__base0.__base2.__base0.m_hWnd) != core::ptr::null_mut()) as BOOL;
    }
    pub unsafe fn CreateComboBox(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_ComboBox = CUiWindow_CComboBox__new_0::<CComboBox>();
        (*(*this_).m_ComboBox).__base1.m_VerticalAlignment = UiAlign_LeftTop;
        (*(*this_).m_ComboBox).__base1.m_HorizontalAlignment = UiAlign_RightBtm;
        (*(*this_).m_ComboBox).__base1.m_Margin.top = 4;

        ((*(*this_).m_ComboBox).__base2.Create((*(*(*this_).m_Toolbar).__base0.__base2.__base0).m_hWnd) != core::ptr::null_mut()) as BOOL
    }
    pub unsafe fn CreateHSplitter(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_HSplitter = CUiSplitPanel__new_0();
        (*(*this_).m_HSplitter).__base1.m_VerticalAlignment = UiAlign_Stretch;
        (*(*this_).m_HSplitter).__base1.m_HorizontalAlignment = UiAlign_Stretch;
        (*(*this_).m_HSplitter).m_DynamicFirst = (TRUE as i32);
        (*(*this_).m_HSplitter).m_Horizontal = (TRUE as i32);
        (*(*this_).m_HSplitter).m_Pos = (INT_MAX as i32); // set INT_MAX to use lowest possible position (m_MinSecond)
        (*(*this_).m_HSplitter).m_MinFirst = 10;
        (*(*this_).m_HSplitter).m_MinSecond = 140;
        (*(*(*this_).m_Panel).__base2.Children()).__base0.Append(core::ptr::addr_of_mut!((*(*this_).m_HSplitter).__base0));

        ((*(*this_).m_HSplitter).Create((*this_).__base0.__base2.m_hWnd) != core::ptr::null_mut()) as BOOL
    }
    pub unsafe fn CreateListView(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_ListView = CAppsListView__new_0();
        (*(*(*this_).m_ListView).__base0.__base1).m_VerticalAlignment = UiAlign_Stretch;
        (*(*(*this_).m_ListView).__base0.__base1).m_HorizontalAlignment = UiAlign_Stretch;
        (*(*this_).m_HSplitter).First().__base0.Append(core::ptr::addr_of_mut!((*(*this_).m_ListView).__base0.__base0));

        return ((*(*this_).m_ListView).Create((*this_).__base0.__base2.m_hWnd) != core::ptr::null_mut()) as BOOL;
    }
    pub unsafe fn CreateAppInfoDisplay(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_AppsInfo = CAppInfoDisplay__new_0();
        (*(*this_).m_AppsInfo).__base0.__base1.m_VerticalAlignment = UiAlign_Stretch;
        (*(*this_).m_AppsInfo).__base0.__base1.m_HorizontalAlignment = UiAlign_Stretch;
        (*(*this_).m_HSplitter).Second().__base0.Append(core::ptr::addr_of_mut!((*(*this_).m_AppsInfo).__base0.__base0));

        ((*(*this_).m_AppsInfo).Create((*this_).__base0.__base2.m_hWnd) != core::ptr::null_mut()) as BOOL
    }
    pub unsafe fn OnSize(&mut self, hwnd: HWND, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        if wParam == (SIZE_MINIMIZED as _) {
            return;
        }

        /* Resize the toolbar */
        (*(*this_).m_Toolbar).__base0.__base2.AutoSize();

        /* Automatically hide captions */
        let dToolbarTreshold: DWORD = (*(*this_).m_Toolbar).GetMaxButtonsWidth();
        let dSearchbarMargin: DWORD = (LOWORD!(lParam) as i32
            - (*(*this_).m_SearchBar).__base2.m_Width
            - (*(*this_).m_ComboBox).__base2.m_Width
            - (TOOLBAR_PADDING * 2) as i32) as DWORD;

        if dSearchbarMargin > dToolbarTreshold {
            (*(*this_).m_Toolbar).ShowButtonCaption(true);
        } else if dSearchbarMargin < dToolbarTreshold {
            (*(*this_).m_Toolbar).ShowButtonCaption(false);
        }

        let r: RECT = RECT {
            left: 0,
            top: 0,
            right: LOWORD!(lParam) as i32,
            bottom: HIWORD!(lParam) as i32,
        };
        let mut hdwp: HDWP = core::ptr::null_mut();
        let mut count: INT = ((*(*this_).m_Panel).CountSizableChildren()) as i32;

        hdwp = BeginDeferWindowPos(count);
        if !hdwp.is_null() {
            hdwp = (*(*this_).m_Panel).OnParentSize(r, hdwp);
            if !hdwp.is_null() {
                EndDeferWindowPos(hdwp);
            }
        }

        count = ((*(*this_).m_SearchBar).CountSizableChildren()) as i32;
        hdwp = BeginDeferWindowPos(count);
        if !hdwp.is_null() {
            hdwp = (*(*this_).m_SearchBar).OnParentSize(r, hdwp);
            if !hdwp.is_null() {
                EndDeferWindowPos(hdwp);
            }
        }

        (*(*this_).m_ComboBox).__base1.m_Margin.right = (*(*this_).m_SearchBar).__base2.m_Width
            + (*(*this_).m_SearchBar).__base1.m_Margin.right
            + (TOOLBAR_PADDING as i32);
        count = ((*(*this_).m_ComboBox).CountSizableChildren()) as i32;
        hdwp = BeginDeferWindowPos(count);
        if !hdwp.is_null() {
            hdwp = (*(*this_).m_ComboBox).OnParentSize(r, hdwp);
            if !hdwp.is_null() {
                EndDeferWindowPos(hdwp);
            }
        }
    }
    pub unsafe fn OnCommand(&mut self, wParam: WPARAM, lParam: LPARAM) {
        let this_: *mut Self = self;
        if lParam != 0 {
            if (lParam as HWND) == (*(*this_).m_SearchBar).GetWindow() {
                let mut szBuf: CStringW = CStringW::new();
                match (HIWORD!(wParam)) as u32 {
                    EN_SETFOCUS => {
                        let mut szWndText: CStringW = CStringW::new();

                        szBuf.LoadStringW(IDS_SEARCH_TEXT);
                        (*(*this_).m_SearchBar).GetWindowTextW(&mut szWndText);
                        if szBuf.Compare(szWndText.GetString()) == 0 {
                            (*(*this_).m_SearchBar).__base2.__base0.SetWindowTextW(CPP_COM_WSTR_96A296D224F2.as_ptr());
                        }
                    }
                    EN_KILLFOCUS => {
                        (*(*this_).m_SearchBar).GetWindowTextW(&mut szBuf);
                        if szBuf.IsEmpty() {
                            szBuf.LoadStringW(IDS_SEARCH_TEXT);
                            (*(*this_).m_SearchBar).__base2.__base0.SetWindowTextW(szBuf.GetString());
                        }
                    }
                    EN_CHANGE => {
                        let mut szWndText: CStringW = CStringW::new();

                        szBuf.LoadStringW(IDS_SEARCH_TEXT);
                        (*(*this_).m_SearchBar).GetWindowTextW(&mut szWndText);
                        if szBuf.Compare(szWndText.GetString()) == 0 {
                            szWndText = CStringW::from_ptr(CPP_COM_WSTR_96A296D224F2.as_ptr());
                            (*(*this_).m_MainWindow).SearchTextChanged(&szWndText);
                        } else {
                            (*(*this_).m_MainWindow).SearchTextChanged(&szWndText);
                        }
                    }
                    _ => {}
                }

                return;
            } else if (lParam as HWND) == (*(*this_).m_ComboBox).GetWindow() {
                let NotifyCode: i32 = HIWORD!(wParam) as i32;
                match (NotifyCode) as u32 {
                    CBN_SELCHANGE => {
                        let CurrSelection: i32 = ((*(*this_).m_ComboBox).__base2.__base0.SendMessageW__0(
                            CB_GETCURSEL,
                            0,
                            0,
                        )) as i32;

                        let ViewModeList: [i32; 3] = [(LV_VIEW_DETAILS as i32), (LV_VIEW_LIST as i32), (LV_VIEW_TILE as i32)];
                        ATLASSERT!(CurrSelection < (_countof!(ViewModeList) as i32));
                        if (*(*this_).m_ListView).SetViewMode((ViewModeList[CurrSelection as usize]) as DWORD) == 0 {
                            (*this_).__base0.__base2.MessageBoxW(
                                CPP_COM_WSTR_00FB3B10D34F.as_ptr(),
                                core::ptr::null_mut(),
                                MB_OK,
                            );
                        }
                    }
                    _ => {}
                }

                return;
            } else if (lParam as HWND) == (*(*this_).m_Toolbar).__base0.GetWindow() {
                // the message is sent from Toolbar. fall down to continue process
            } else {
                return;
            }
        }

        // the LOWORD of wParam contains a Menu or Control ID
        let wCommand: WORD = LOWORD!(wParam);

        match (wCommand) as u32 {
            ID_INSTALL
            | ID_UNINSTALL
            | ID_MODIFY
            | ID_REGREMOVE
            | ID_REFRESH
            | ID_RESETDB
            | ID_CHECK_ALL => {
                (*(*this_).m_MainWindow).__base0.SendMessageW__0(WM_COMMAND, wCommand as WPARAM, 0);
            }
            _ => {}
        }
    }
    pub unsafe fn _UpdateInstallBtn(&mut self) {
        let this_: *mut Self = self;
        if (*this_).ApplicationViewType == AppViewTypeInstalledApps {
            EnableMenuItem((*this_).__base0.__base2.GetMenu(), ID_INSTALL, MF_GRAYED);
            (*(*this_).m_Toolbar).__base0.__base2.__base0.SendMessageW__0(TB_ENABLEBUTTON, (ID_INSTALL) as WPARAM, (FALSE) as LPARAM);
        } else if (*this_).ApplicationViewType == AppViewTypeAvailableApps {
            // We shouldn't get there in APPWIZ-mode.
            ATLASSERT!(!(*(*this_).m_MainWindow).m_bAppwizMode);

            /* Even if no ListView item is focused at this point, enable
             * or disable ID_INSTALL if there are selected applications. */
            let mut CanInstall: bool = !(*(*this_).m_MainWindow).m_Selected.IsEmpty();
            CanInstall = CanInstall || ((*(*this_).m_ListView).__base0.__base2.GetSelectedCount() > 0);
            EnableMenuItem((*this_).__base0.__base2.GetMenu(), ID_INSTALL, if CanInstall { MF_ENABLED } else { MF_GRAYED });
            (*(*this_).m_Toolbar).__base0.__base2.__base0.SendMessageW__0(TB_ENABLEBUTTON, (ID_INSTALL) as WPARAM, (CanInstall as BOOL) as LPARAM);
        }
    }
    pub unsafe fn __ctor_0(&mut self, MainWindow: *mut CMainWindow) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).m_Panel = core::ptr::null_mut();
        (*this_).m_Toolbar = core::ptr::null_mut();
        (*this_).m_ComboBox = core::ptr::null_mut();
        (*this_).m_SearchBar = core::ptr::null_mut();
        (*this_).m_ListView = core::ptr::null_mut();
        (*this_).m_AppsInfo = core::ptr::null_mut();
        (*this_).m_HSplitter = core::ptr::null_mut();
        (*this_).m_MainWindow = MainWindow;
        (*this_).ApplicationViewType = AppViewTypeAvailableApps;
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        CMainToolbar__delete((*this_).m_Toolbar);
        CUiWindow_CSearchBar__delete((*this_).m_SearchBar);
        CAppsListView__delete((*this_).m_ListView);
        CAppInfoDisplay__delete((*this_).m_AppsInfo);
        CUiSplitPanel__delete((*this_).m_HSplitter);
    }
    pub unsafe fn GetWndClassInfo() -> *mut CWndClassInfo {
        let csStyle: DWORD = CS_VREDRAW | CS_HREDRAW;
        static mut wc: CWndClassInfo = unsafe { core::mem::zeroed() };
        static mut wc_inited: bool = false;
        if !wc_inited {
            wc = CWndClassInfo {
                m_wc: WNDCLASSEX {
                    cbSize: core::mem::size_of::<WNDCLASSEX>() as UINT,
                    style: CS_VREDRAW | CS_HREDRAW,
                    lpfnWndProc: Some(StartWindowProc),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: core::ptr::null_mut(),
                    hIcon: core::ptr::null_mut(),
                    hCursor: core::ptr::null_mut(),
                    hbrBackground: core::ptr::null_mut(),
                    lpszMenuName: core::ptr::null(),
                    lpszClassName: CPP_COM_WSTR_7AFA86ACD8E9.as_ptr(),
                    hIconSm: core::ptr::null_mut(),
                },
                m_lpszOrigName: core::ptr::null(),
                pWndProc: None,
                m_lpszCursorID: IDC_ARROW,
                m_bSystemCursor: (TRUE as i32),
                m_atom: 0,
                m_szAutoName: CPP_COM_WSTR_96A296D224F2.as_ptr(),
            };
            // hbrBackground = (HBRUSH)(COLOR_BTNFACE + 1); integer-to-pointer cast is not const-evaluable, so set at runtime.
            wc.m_wc.hbrBackground = (COLOR_BTNFACE + 1) as usize as HBRUSH;
            wc_inited = true;
        }
        let _ = csStyle;
        return core::ptr::addr_of_mut!(wc);
    }
    pub unsafe fn Create(&mut self, hwndParent: HWND) -> HWND {
        let this_: *mut Self = self;
        let mut r: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

        // Pick the "Programs" sub-menu for building the context menu.
        let hMenu: HMENU = GetSubMenu((*(*this_).m_MainWindow).__base0.GetMenu(), 1);

        return (*this_).CWindowImpl_Create(hwndParent, &mut r, CPP_COM_WSTR_96A296D224F2.as_ptr(), WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS, 0, hMenu, );
    }
    pub unsafe fn SetRedraw(&mut self, bRedraw: BOOL) {
        let this_: *mut Self = self;
        (*this_).__base0.__base2.SetRedraw(bRedraw);
        (*(*this_).m_ListView).__base0.__base2.SetRedraw(bRedraw);
    }
    pub unsafe fn RefreshAvailableItem(&mut self, PackageName: PCWSTR) {
        let this_: *mut Self = self;
        if (*this_).ApplicationViewType != AppViewTypeAvailableApps || PackageName.is_null() {
            return;
        }
        let mut pApp: *mut CAppInfo;
        let mut i: UINT = 0;
        loop {
            pApp = (*(*this_).m_ListView).__base0.__base2.GetItemData((i) as i32) as *mut CAppInfo;
            if pApp.is_null() {
                break;
            }
            if (*pApp).szIdentifier.CompareNoCase(PackageName) == 0 {
                (*this_).RefreshDetailsPane(&mut *pApp, true);
                break;
            }
            i += 1;
        }
    }
    pub unsafe fn SetFocusOnSearchBar(&mut self) {
        let this_: *mut Self = self;
        (*(*this_).m_SearchBar).__base2.__base0.SetFocus();
    }
    pub unsafe fn SetDisplayAppType(&mut self, AppType: APPLICATION_VIEW_TYPE) -> BOOL {
        let this_: *mut Self = self;
        if (*(*this_).m_ListView).SetDisplayAppType(AppType) == 0 {
            return (FALSE) as BOOL;
        }

        (*this_).ApplicationViewType = AppType;
        (*(*this_).m_AppsInfo).SetWelcomeText((((*(*this_).m_MainWindow).m_bAppwizMode) != 0));

        let hMenu: HMENU = (*this_).__base0.__base2.GetMenu();
        match AppType {
            AppViewTypeInstalledApps => {
                EnableMenuItem(hMenu, ID_INSTALL, MF_GRAYED);
                (*(*this_).m_Toolbar).__base0.__base2.__base0.SendMessageW__0(TB_ENABLEBUTTON, (ID_INSTALL) as WPARAM, (FALSE) as LPARAM);

                (*(*this_).m_Toolbar).__base0.__base2.__base0.SendMessageW__0(TB_ENABLEBUTTON, (ID_CHECK_ALL) as WPARAM, (FALSE) as LPARAM);
            }

            AppViewTypeAvailableApps => {
                // We shouldn't get there in APPWIZ-mode.
                ATLASSERT!(((*(*this_).m_MainWindow).m_bAppwizMode == 0) as BOOL);

                /* Even if no ListView item is focused at this point, enable
                 * or disable ID_INSTALL if there are selected applications. */
                (*this_)._UpdateInstallBtn();

                (*(*this_).m_Toolbar).__base0.__base2.__base0.SendMessageW__0(TB_ENABLEBUTTON, (ID_CHECK_ALL) as WPARAM, (TRUE) as LPARAM);
            }
            _ => {}
        }

        /* Always disable these items by default */
        EnableMenuItem(hMenu, ID_UNINSTALL, MF_GRAYED);
        EnableMenuItem(hMenu, ID_MODIFY, MF_GRAYED);
        EnableMenuItem(hMenu, ID_REGREMOVE, MF_GRAYED);
        (*(*this_).m_Toolbar).__base0.__base2.__base0.SendMessageW__0(TB_ENABLEBUTTON, (ID_UNINSTALL) as WPARAM, (FALSE) as LPARAM);
        (*(*this_).m_Toolbar).__base0.__base2.__base0.SendMessageW__0(TB_ENABLEBUTTON, (ID_MODIFY) as WPARAM, (FALSE) as LPARAM);

        return (TRUE) as BOOL;
    }
    pub unsafe fn AddApplication(&mut self, AppInfo: *mut CAppInfo, InitialCheckState: BOOL) -> BOOL {
        let this_: *mut Self = self;
        return (*(*this_).m_ListView).AddApplication(AppInfo, InitialCheckState);
    }
    pub unsafe fn SetWatermark(&mut self, Text: &CStringW) {
        let this_: *mut Self = self;
        (*(*this_).m_ListView).SetWatermark(Text);
    }
    pub unsafe fn CheckAll(&mut self) {
        let this_: *mut Self = self;
        (*(*this_).m_ListView).CheckAll();
    }
    pub unsafe fn GetFocusedItemData(&mut self) -> PVOID {
        let this_: *mut Self = self;
        return (*(*this_).m_ListView).GetFocusedItemData();
    }
    pub unsafe fn GetItemCount(&mut self) -> i32 {
        let this_: *mut Self = self;
        (*(*this_).m_ListView).__base0.__base2.GetItemCount()
    }
    pub unsafe fn AppendTabOrderWindow__body(&mut self, Direction: i32, TabOrderList: &mut CSimpleArray<HWND>) {
        let this_: *mut Self = self;
        (*(*this_).m_Toolbar).__base0.AppendTabOrderWindow(Direction, TabOrderList);
        (*(*this_).m_ComboBox).AppendTabOrderWindow(Direction, TabOrderList);
        (*(*this_).m_SearchBar).AppendTabOrderWindow(Direction, TabOrderList);
        (*(*this_).m_ListView).__base0.AppendTabOrderWindow(Direction, TabOrderList);
        (*(*this_).m_AppsInfo).__base0.AppendTabOrderWindow(Direction, TabOrderList);
    }
    pub unsafe fn GetRestoreListSelectionData(&mut self, Restore: &mut RESTORELISTSELECTION) {
        let this_: *mut Self = self;
        let Item: &mut LVITEMW = &mut Restore.Item;
        Item.mask = LVIF_TEXT | LVIF_STATE;
        Item.iItem = -1;
        Item.iSubItem = 0;
        Item.stateMask = LVIS_FOCUSED | LVIS_SELECTED;
        Item.pszText = Restore.Name.as_mut_ptr();
        Item.cchTextMax = _countof!(Restore.Name) as _;

        let hList: HWND = if !(*this_).m_ListView.is_null() {
            (*(*this_).m_ListView).__base0.__base2.__base0.m_hWnd
        } else {
            core::ptr::null_mut()
        };
        if !hList.is_null() {
            Item.iItem = ListView_GetNextItem!(hList, -1, LVNI_FOCUSED);
            ListView_GetItem!(hList, Item as *mut LVITEMW);
        }
    }
    pub unsafe fn RestoreListSelection(&mut self, Restore: &RESTORELISTSELECTION) {
        let this_: *mut Self = self;
        let Item: &LVITEMW = &Restore.Item;
        let mut index: i32 = Item.iItem;
        if index != -1 { // Was there a selected item?
            let mut fi: LVFINDINFOW = core::mem::zeroed::<LVFINDINFOW>();
            fi.flags = LVFI_STRING;
            fi.psz = Item.pszText;
            index = ListView_FindItem!((*(*this_).m_ListView).__base0.__base2.__base0.m_hWnd, -1, core::ptr::addr_of_mut!(fi));
        }
        if index != -1 { // Is it still in the list?
            ListView_SetItemState!((*(*this_).m_ListView).__base0.__base2.__base0.m_hWnd, index, Item.state, Item.stateMask);
        }
    }
    pub unsafe fn RefreshDetailsPane(&mut self, Info: &mut CAppInfo, OnlyUpdateText: bool) {
        let this_: *mut Self = self;
        (*(*this_).m_AppsInfo).ShowAppInfo(Info, OnlyUpdateText);
    }
    pub unsafe fn ItemGetFocus(&mut self, CallbackParam: LPVOID) {
        let this_: *mut Self = self;
        if CallbackParam.is_null() {
            return;
        }

        let Info: *mut CAppInfo = CallbackParam as *mut CAppInfo;
        (*this_).RefreshDetailsPane(&mut *Info, false);

        let hMenu: HMENU = (*this_).__base0.__base2.GetMenu();
        if (*this_).ApplicationViewType == AppViewTypeInstalledApps {
            /* ID_INSTALL is left disabled */

            EnableMenuItem(hMenu, ID_UNINSTALL, MF_ENABLED);
            (*(*this_).m_Toolbar).__base0.__base2.__base0.SendMessageW__0(TB_ENABLEBUTTON, (ID_UNINSTALL) as WPARAM, (TRUE) as LPARAM);

            let CanModify: BOOL = (*Info).CanModify();
            EnableMenuItem(hMenu, ID_MODIFY, if CanModify != 0 { MF_ENABLED } else { MF_GRAYED });
            (*(*this_).m_Toolbar).__base0.__base2.__base0.SendMessageW__0(TB_ENABLEBUTTON, (ID_MODIFY) as WPARAM, (CanModify) as LPARAM);

            EnableMenuItem(hMenu, ID_REGREMOVE, MF_ENABLED);
        } else if (*this_).ApplicationViewType == AppViewTypeAvailableApps {
            // We shouldn't get there in APPWIZ-mode.
            ATLASSERT!(((*(*this_).m_MainWindow).m_bAppwizMode == 0) as i32);

            EnableMenuItem(hMenu, ID_INSTALL, MF_ENABLED);
            (*(*this_).m_Toolbar).__base0.__base2.__base0.SendMessageW__0(TB_ENABLEBUTTON, (ID_INSTALL) as WPARAM, (TRUE) as LPARAM);

            /* ID_UNINSTALL, ID_MODIFY and ID_REGREMOVE are left disabled */
            // TODO: When we are able to detect whether this selected available
            // application is already installed (could be an older version),
            // do also what's done in the AppViewTypeInstalledApps case above.
        }
    }
    pub unsafe fn ItemCheckStateChanged(&mut self, bChecked: BOOL, CallbackParam: LPVOID) {
        let this_: *mut Self = self;
        (*(*this_).m_MainWindow).ItemCheckStateChanged(bChecked, CallbackParam);
    }
}

impl CAsyncLoadIcon {
    pub unsafe fn Free(&mut self) {
        let this_: *mut Self = self;
        free((this_).cast());
    }
    pub unsafe fn Queue(hAppsList: HWND, AppInfo: &mut CAppInfo, Parse: bool) -> *mut CAsyncLoadIcon {
        ATLASSERT!(GetCurrentThreadId() == GetWindowThreadProcessId(hAppsList, core::ptr::null_mut()));
        let mut szIconPath: CStringW = CStringW::new();
        if AppInfo.RetrieveIcon(&mut szIconPath) == 0 {
            return core::ptr::null_mut();
        }
        let cbstr: SIZE_T = ((szIconPath.GetLength() + 1) as SIZE_T) * (core::mem::size_of::<WCHAR>()) as _;
        let task: *mut CAsyncLoadIcon = malloc(core::mem::size_of::<CAsyncLoadIcon>() + (cbstr) as _) as *mut CAsyncLoadIcon;
        if task.is_null() {
            return core::ptr::null_mut();
        }
        (*task).hAppsList = hAppsList;
        (*task).AppInfo = AppInfo as *mut CAppInfo;
        (*task).TaskId = g_AsyncIconTaskId;
        (*task).Parse = Parse;
        CopyMemory!(
            (*task).Location.as_mut_ptr() as *mut core::ffi::c_void,
            szIconPath.GetBuffer(0) as *const core::ffi::c_void,
            cbstr,
        );
        szIconPath.ReleaseBuffer();
        (*task).pNext = g_AsyncIconTasks;
        g_AsyncIconTasks = task;
        return task;
    }
    pub unsafe fn StartTasks() {
        let tasks: *mut CAsyncLoadIcon = g_AsyncIconTasks;
        g_AsyncIconTasks = core::ptr::null_mut();
        let hThread = CreateThread(core::ptr::null_mut(), 0, Some(AsyncLoadIconProc), tasks as LPVOID, 0, core::ptr::null_mut());
        if !hThread.is_null() {
            CloseHandle(hThread);
        } else {
            AsyncLoadIconProc((tasks).cast()); // Insist so we at least free the tasks
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
