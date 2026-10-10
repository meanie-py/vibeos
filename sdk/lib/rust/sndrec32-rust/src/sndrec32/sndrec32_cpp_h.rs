use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut hInst: HINSTANCE = unsafe { core::mem::zeroed() };
pub static mut szTitle: [TCHAR; 100] = unsafe { core::mem::zeroed() };
pub static mut szWindowClass: [TCHAR; 100] = unsafe { core::mem::zeroed() };
pub static mut win_first: BOOL = unsafe { core::mem::zeroed() };
pub static mut wout_first: BOOL = unsafe { core::mem::zeroed() };
pub static mut main_win: HWND = unsafe { core::mem::zeroed() };
pub static mut wave_win: HWND = unsafe { core::mem::zeroed() };
pub static mut slider: HWND = unsafe { core::mem::zeroed() };
pub static mut buttons: [HWND; 5] = unsafe { core::mem::zeroed() };
pub static mut butbmps: [HBITMAP; 5] = unsafe { core::mem::zeroed() };
pub static mut butbmps_dis: [HBITMAP; 5] = unsafe { core::mem::zeroed() };
pub static mut buttons_std_proc: WNDPROC = unsafe { core::mem::zeroed() };
pub static mut butdisabled: [BOOL; 5] = unsafe { core::mem::zeroed() };
pub static mut stopped_flag: BOOL = unsafe { core::mem::zeroed() };
pub static mut isnew: BOOL = unsafe { core::mem::zeroed() };
pub static mut display_dur: BOOL = unsafe { core::mem::zeroed() };
pub static mut slider_pos: DWORD = unsafe { core::mem::zeroed() };
pub static mut slider_min: WORD = unsafe { core::mem::zeroed() };
pub static mut slider_max: WORD = unsafe { core::mem::zeroed() };
pub static mut samples_max: DWORD = unsafe { core::mem::zeroed() };
pub static mut ofn: OPENFILENAME = unsafe { core::mem::zeroed() };
pub static mut file_path: [TCHAR; 260] = unsafe { core::mem::zeroed() };
pub static mut str_pos: [TCHAR; 100] = unsafe { core::mem::zeroed() };
pub static mut str_dur: [TCHAR; 100] = unsafe { core::mem::zeroed() };
pub static mut str_buf: [TCHAR; 100] = unsafe { core::mem::zeroed() };
pub static mut str_fmt: [TCHAR; 100] = unsafe { core::mem::zeroed() };
pub static mut str_chan: [TCHAR; 100] = unsafe { core::mem::zeroed() };
pub static mut str_mono: [TCHAR; 10] = unsafe { core::mem::zeroed() };
pub static mut str_stereo: [TCHAR; 10] = unsafe { core::mem::zeroed() };
pub static mut path_set: BOOL = unsafe { core::mem::zeroed() };
pub static mut AUD_BUF: *mut audio_membuffer = unsafe { core::mem::zeroed() };
pub static mut AUD_OUT: *mut audio_waveout = unsafe { core::mem::zeroed() };
pub static mut AUD_IN: *mut audio_wavein = unsafe { core::mem::zeroed() };
pub static mut s_recording: BOOL = unsafe { core::mem::zeroed() };
pub static mut s_info: NONCLIENTMETRICS = unsafe { core::mem::zeroed() };
pub static mut text_rect: RECT = unsafe { core::mem::zeroed() };
pub static mut text2_rect: RECT = unsafe { core::mem::zeroed() };
pub static mut cli: RECT = unsafe { core::mem::zeroed() };

#[no_mangle]
pub unsafe extern "system" fn wWinMain(hInstance: HINSTANCE, hPrevInstance: HINSTANCE, lpCmdLine: LPTSTR, nCmdShow: i32) -> i32 {
    __cxx_global_init();
    let _ = hPrevInstance; // UNREFERENCED_PARAMETER(hPrevInstance)
    let _ = lpCmdLine; // UNREFERENCED_PARAMETER(lpCmdLine)

    let mut msg: MSG = core::mem::zeroed();
    let mut hAccelTable: HACCEL = core::mem::zeroed();

    s_info.cbSize = core::mem::size_of::<NONCLIENTMETRICS>() as _;

    InitCommonControls();

    if GetUserDefaultUILanguage() == MAKELANGID!(LANG_HEBREW, SUBLANG_DEFAULT) {
        SetProcessDefaultLayout(LAYOUT_RTL);
    }

    win_first = (FALSE) as i32;
    wout_first = win_first;

    text_rect.left = (REFRESHA_X as i32);
    text_rect.top = (REFRESHA_Y as i32);
    text_rect.right = (REFRESHA_CX as i32);
    text_rect.bottom = (REFRESHA_CY as i32);

    text2_rect.left = (REFRESHB_X as i32);
    text2_rect.top = (REFRESHB_Y as i32);
    text2_rect.right = (REFRESHB_CX as i32);
    text2_rect.bottom = (REFRESHB_CY as i32);

    /* Retrieving default system font, and others system informations */
    SystemParametersInfo(
        SPI_GETNONCLIENTMETRICS,
        core::mem::size_of::<NONCLIENTMETRICS>() as _,
        (core::ptr::addr_of_mut!(s_info)).cast(),
        0,
    );

    /* Set font size */
    s_info.lfMenuFont.lfHeight = 14;

    /* Inits buttons bitmaps */

    butbmps[0] = LoadBitmap(hInstance, MAKEINTRESOURCE!(IDB_BITMAP2_START));
    butbmps[1] = LoadBitmap(hInstance, MAKEINTRESOURCE!(IDB_BITMAP2_END));
    butbmps[2] = LoadBitmap(hInstance, MAKEINTRESOURCE!(IDB_BITMAP2_PLAY));
    butbmps[3] = LoadBitmap(hInstance, MAKEINTRESOURCE!(IDB_BITMAP2_STOP));
    butbmps[4] = LoadBitmap(hInstance, MAKEINTRESOURCE!(IDB_BITMAP2_REC));

    butbmps_dis[0] = LoadBitmap(hInstance, MAKEINTRESOURCE!(IDB_BITMAP2_START_DIS));
    butbmps_dis[1] = LoadBitmap(hInstance, MAKEINTRESOURCE!(IDB_BITMAP2_END_DIS));
    butbmps_dis[2] = LoadBitmap(hInstance, MAKEINTRESOURCE!(IDB_BITMAP2_PLAY_DIS));
    butbmps_dis[3] = LoadBitmap(hInstance, MAKEINTRESOURCE!(IDB_BITMAP2_STOP_DIS));
    butbmps_dis[4] = LoadBitmap(hInstance, MAKEINTRESOURCE!(IDB_BITMAP2_REC_DIS));

    /* Inits audio devices and buffers */

    let mut AUD_buffer: audio_membuffer = audio_membuffer__zeroed();
    AUD_buffer.__ctor_1(A44100_16BIT_STEREO);
    let mut AUD_waveout: audio_waveout = audio_waveout__make_0(&A44100_16BIT_STEREO, &mut (AUD_buffer).__base1);
    let mut AUD_wavein: audio_wavein = audio_wavein__make_0(&A44100_16BIT_STEREO, &mut (AUD_buffer).__base0);

    AUD_buffer.__base1.play_finished = Some(l_play_finished);
    AUD_buffer.audio_arrival = Some(l_audio_arrival);
    AUD_buffer.buffer_resized = Some(l_buffer_resized);

    AUD_buffer.alloc_seconds__1(INITIAL_BUFREC_SECONDS as f32);

    AUD_IN = core::ptr::addr_of_mut!(AUD_wavein);
    AUD_OUT = core::ptr::addr_of_mut!(AUD_waveout);
    AUD_BUF = core::ptr::addr_of_mut!(AUD_buffer);

    /* Inits slider default parameters */

    slider_pos = 0;
    slider_min = 0;
    slider_max = (SLIDER_W as u16);

    stopped_flag = (FALSE) as i32;
    path_set = (FALSE) as i32;
    isnew = (TRUE) as i32;
    display_dur = (TRUE) as i32;

    samples_max = AUD_buffer.total_samples();

    s_recording = (false) as i32;

    /* Inits strings */
    LoadString(hInstance, IDS_APP_TITLE, (szTitle).as_mut_ptr(), (MAX_LOADSTRING as i32));
    LoadString(hInstance, IDC_REACTOS_SNDREC32, (szWindowClass).as_mut_ptr(), (MAX_LOADSTRING as i32));
    LoadString(hInstance, IDS_STRPOS, (str_pos).as_mut_ptr(), (MAX_LOADSTRING as i32));
    LoadString(hInstance, IDS_STRDUR, (str_dur).as_mut_ptr(), (MAX_LOADSTRING as i32));
    LoadString(hInstance, IDS_STRBUF, (str_buf).as_mut_ptr(), (MAX_LOADSTRING as i32));
    LoadString(hInstance, IDS_STRFMT, (str_fmt).as_mut_ptr(), (MAX_LOADSTRING as i32));
    LoadString(hInstance, IDS_STRCHAN, (str_chan).as_mut_ptr(), (MAX_LOADSTRING as i32));
    LoadString(hInstance, IDS_STRMONO, (str_mono).as_mut_ptr(), 10);
    LoadString(hInstance, IDS_STRSTEREO, (str_stereo).as_mut_ptr(), 10);

    /* Registers sndrec32 window class */
    MyRegisterClass(hInstance);
    MyRegisterClass_wave(hInstance);

    if InitInstance(hInstance, nCmdShow) == 0 {
        MessageBox(
            core::ptr::null_mut(),
            (['C' as u16, 'r' as u16, 'e' as u16, 'a' as u16, 't' as u16, 'e' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, '(' as u16, ')' as u16, ' ' as u16, 'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16, 'r' as u16, '!' as u16, 0].as_ptr()),
            (['E' as u16, 'R' as u16, 'R' as u16, 'O' as u16, 'R' as u16, 0].as_ptr()),
            MB_ICONERROR,
        );
        return (FALSE) as i32;
    }

    /* Loads key accelerators */
    hAccelTable = LoadAccelerators(hInstance, MAKEINTRESOURCE!(IDC_REACTOS_SNDREC32));

    /* Starts main loop */
    while GetMessage(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0) != 0 {
        if TranslateAccelerator(main_win, hAccelTable, core::ptr::addr_of_mut!(msg)) == 0 {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessage(core::ptr::addr_of_mut!(msg));
        }
    }

    if wout_first != 0 {
        AUD_waveout.close();
    }

    if win_first != 0 {
        AUD_wavein.close();
    }

    AUD_buffer.clear();

    return msg.wParam as i32;
}
#[no_mangle]
pub unsafe extern "C" fn MyRegisterClass(hInstance: HINSTANCE) -> ATOM {
    let mut wcex: WNDCLASSEX = core::mem::zeroed();

    wcex.cbSize = core::mem::size_of::<WNDCLASSEX>() as u32;

    wcex.style = CS_HREDRAW | CS_VREDRAW;
    wcex.lpfnWndProc = Some(WndProc);
    wcex.cbClsExtra = 0;
    wcex.cbWndExtra = 0;
    wcex.hInstance = hInstance;
    wcex.hIcon = LoadIcon(hInstance, MAKEINTRESOURCE!(IDI_SNDREC32));
    wcex.hCursor = LoadCursor(core::ptr::null_mut(), IDC_ARROW);
    wcex.hbrBackground = GetSysColorBrush((COLOR_BTNFACE as i32));
    wcex.lpszMenuName = MAKEINTRESOURCE!(IDR_MENU1);
    wcex.lpszClassName = (szWindowClass).as_ptr();
    wcex.hIconSm = LoadIcon(wcex.hInstance, MAKEINTRESOURCE!(IDI_SNDREC32));

    return RegisterClassEx(core::ptr::addr_of_mut!(wcex));
}
#[no_mangle]
pub unsafe extern "C" fn InitInstance(hInstance: HINSTANCE, nCmdShow: i32) -> BOOL {
    let mut hWnd: HWND;

    hInst = hInstance;

    hWnd = CreateWindow!(szWindowClass.as_ptr(),
                        szTitle.as_ptr(),
                        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        MAINWINDOW_W,
                        MAINWINDOW_H,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        hInstance,
                        core::ptr::null_mut());
    if hWnd.is_null() {
        return (FALSE) as BOOL;
    }

    ShowWindow(hWnd, nCmdShow);
    UpdateWindow(hWnd);

    main_win = hWnd;

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn MyRegisterClass_wave(hInstance: HINSTANCE) -> ATOM {
    let mut wcex: WNDCLASSEX = core::mem::zeroed();

    wcex.cbSize = core::mem::size_of::<WNDCLASSEX>() as u32;

    wcex.style = CS_HREDRAW | CS_VREDRAW;
    wcex.lpfnWndProc = Some(WndProc_wave);
    wcex.cbClsExtra = 0;
    wcex.cbWndExtra = 0;
    wcex.hInstance = hInstance;
    wcex.hIcon = core::ptr::null_mut();
    wcex.hCursor = LoadCursor(core::ptr::null_mut(), IDC_ARROW);
    wcex.hbrBackground = GetStockObject((BLACK_BRUSH as i32)) as HBRUSH;
    wcex.lpszMenuName = core::ptr::null();
    wcex.lpszClassName = (&[0x0073, 0x006e, 0x0064, 0x0072, 0x0065, 0x0063, 0x0033, 0x0032, 0x005f, 0x0077, 0x0061, 0x0076, 0x0065, 0x0000u16]).as_ptr() /* TEXT "sndrec32_wave" */;
    wcex.hIconSm = core::ptr::null_mut();

    return RegisterClassEx(core::ptr::addr_of_mut!(wcex));
}
#[no_mangle]
pub unsafe extern "C" fn InitInstance_wave(f: HWND, hInstance: HINSTANCE, nCmdShow: i32) -> BOOL {
    let mut hWnd: HWND;

    hInst = hInstance;

    hWnd = CreateWindowEx(WS_EX_STATICEDGE,
                          (&[0x0073, 0x006e, 0x0064, 0x0072, 0x0065, 0x0063, 0x0033, 0x0032, 0x005f, 0x0077, 0x0061, 0x0076, 0x0065, 0x0000u16]).as_ptr() /* TEXT "sndrec32_wave" */,
                          (&[0x0000u16]).as_ptr() /* TEXT "" */,
                          WS_VISIBLE | WS_CHILD,
                          (WAVEBAR_X as i32),
                          (WAVEBAR_Y as i32),
                          (WAVEBAR_CX as i32),
                          (WAVEBAR_CY as i32),
                          f,
                          (8 as HMENU),
                          hInstance,
                          core::ptr::null_mut());

    if hWnd.is_null() {
        return (FALSE) as BOOL;
    }

    ShowWindow(hWnd, nCmdShow);
    UpdateWindow(hWnd);

    wave_win = hWnd;

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn WndProc_wave(hWnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut hdc: HDC;
    let mut pen: HPEN;
    let mut oldpen: HPEN;

    let max_h: u32 = (cli.bottom / 2) as u32;
    let mut samples: u32;
    let mut x: u32;
    let mut line_h: u32;

    match message {
        WM_CREATE => {
            GetClientRect(hWnd, core::ptr::addr_of_mut!(cli));
        }

        WM_PAINT => {
            /* Initialize hdc objects */
            hdc = BeginPaint(hWnd, core::ptr::addr_of_mut!(ps));
            pen = CreatePen((PS_SOLID as i32), 1, WAVEBAR_COLOR) as HPEN;
            oldpen = SelectObject(hdc, (pen as HBRUSH).cast()) as HPEN;
            if (*AUD_OUT).current_status() == WAVEOUT_PLAYING {
                samples = (*AUD_OUT).tot_samples_buf();
                let mut i: u32 = 0;
                while i < WAVEBAR_CX {
                    x = (i * samples) / WAVEBAR_CX;
                    line_h = ((*AUD_OUT).nsample(x) * max_h) / (*AUD_OUT).samplevalue_max();
                    if line_h != 0 {
                        MoveToEx(hdc, (i) as i32, (max_h) as i32, core::ptr::null_mut());
                        LineTo(hdc, (i) as i32, (max_h - (line_h * 2)) as i32);
                        LineTo(hdc, (i) as i32, (max_h + (line_h * 2)) as i32);
                    } else {
                        SetPixel(hdc, (i) as i32, (max_h) as i32, WAVEBAR_COLOR);
                    }
                    i += 1;
                }
            } else if (*AUD_IN).current_status() == WAVEIN_RECORDING {
                samples = (*AUD_IN).tot_samples_buf();
                let mut i: u32 = 0;
                while i < WAVEBAR_CX {
                    x = (i * samples) / WAVEBAR_CX;
                    line_h = ((*AUD_IN).nsample(x) * max_h) / (*AUD_IN).samplevalue_max();
                    if line_h != 0 {
                        MoveToEx(hdc, (i) as i32, (max_h) as i32, core::ptr::null_mut());
                        LineTo(hdc, (i) as i32, (max_h - (line_h * 2)) as i32);
                        LineTo(hdc, (i) as i32, (max_h + (line_h * 2)) as i32);
                    } else {
                        SetPixel(hdc, (i) as i32, (max_h) as i32, WAVEBAR_COLOR);
                    }
                    i += 1;
                }
            } else {
                /* In standby mode draw a simple line */
                MoveToEx(hdc, 0, cli.bottom / 2, core::ptr::null_mut());
                LineTo(hdc, (WAVEBAR_CX as i32), cli.bottom / 2);
            }

            SelectObject(hdc, (oldpen).cast());
            DeleteObject((pen).cast());
            EndPaint(hWnd, core::ptr::addr_of_mut!(ps));
        }

        WM_USER => {
        }

        _ => {
            return DefWindowProc(hWnd, message, wParam, lParam);
        }
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn WndProc(hWnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut wmId: i32 = 0;
    let mut str_tmp: [TCHAR; MAX_LOADSTRING as usize] = [0; MAX_LOADSTRING as usize];
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut font: HFONT = core::ptr::null_mut();
    let mut oldfont: HFONT = core::ptr::null_mut();
    let mut slid_samp: i64 = 0 as i64;
    let mut szAppName: [WCHAR; 100 as usize] = [0; 100 as usize];

    /* Checking for global pointers to buffer and io audio devices */
    if (AUD_IN).is_null() || (AUD_OUT).is_null() || (AUD_BUF).is_null()
    {
        MessageBox(core::ptr::null_mut(), (&[0x0042, 0x0075, 0x0066, 0x0066, 0x0065, 0x0072, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x0000u16]).as_ptr() /* TEXT "Buffer Error" */, core::ptr::null(), 0);
        return 1;
    }

    'sw_msg: {
        match message {
            v if v == WM_CREATE => {
                /* Creating the wave bar */
                if InitInstance_wave(hWnd, hInst, (SW_SHOWNORMAL as i32)) == 0
                {
                    MessageBox(core::ptr::null_mut(), (&[0x0049, 0x006e, 0x0069, 0x0074, 0x0049, 0x006e, 0x0073, 0x0074, 0x0061, 0x006e, 0x0063, 0x0065, 0x005f, 0x0077, 0x0061, 0x0076, 0x0065, 0x0028, 0x0029, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x0021, 0x0000u16]).as_ptr() /* TEXT "InitInstance_wave() Error!" */, (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* TEXT "ERROR" */, MB_ICONERROR);
                    return (FALSE) as LRESULT;
                }

                /* Creating ALL the buttons */
                let mut i: i32 = 0 as i32;
                while i < 5 as i32
                {
                    buttons[i as usize] = CreateWindow!((&[0x0062, 0x0075, 0x0074, 0x0074, 0x006f, 0x006e, 0x0000u16]).as_ptr() /* TEXT "button" */,
                                              (&[0x0000u16]).as_ptr() /* TEXT "" */,
                                              WS_CHILD | WS_VISIBLE | BS_BITMAP,
                                              BUTTONS_CX + ((i * ((BUTTONS_W + (if i == 0 { 0 } else { BUTTONS_SPACE }))) as i32)) as u32,
                                              BUTTONS_CY,
                                              BUTTONS_W,
                                              BUTTONS_H,
                                              hWnd,
                                              UlongToPtr!(i as u32) as HMENU,
                                              hInst,
                                              0);
                    if buttons[i as usize].is_null()
                    {
                        MessageBox(core::ptr::null_mut(), core::ptr::null(), (&[0x0043, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0028, 0x0029, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x0021, 0x0000u16]).as_ptr() /* TEXT "CreateWindow() Error!" */, 0);
                        return (FALSE) as LRESULT;
                    }

                    /* Realize the button bmp image */
                    SendMessage(buttons[i as usize], BM_SETIMAGE, IMAGE_BITMAP as WPARAM, butbmps[i as usize] as LPARAM);
                    UpdateWindow(buttons[i as usize]);
                    disable_but((i) as DWORD);
                    i += 1;
                }

                /* Creating the SLIDER window */
                slider = CreateWindow!(TRACKBAR_CLASS,
                                      (&[0x0000u16]).as_ptr() /* TEXT "" */,
                                      WS_CHILD | WS_VISIBLE | TBS_NOTICKS | TBS_HORZ | TBS_ENABLESELRANGE,
                                      SLIDER_CX,
                                      SLIDER_CY,
                                      SLIDER_W,
                                      SLIDER_H,
                                      hWnd,
                                      SLIDER_ID as usize as HMENU,
                                      hInst,
                                      0);
                if (slider).is_null()
                {
                    MessageBox(core::ptr::null_mut(), core::ptr::null(), (&[0x0043, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0028, 0x0029, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x0021, 0x0000u16]).as_ptr() /* TEXT "CreateWindow() Error!" */, 0);
                    return (FALSE) as LRESULT;
                }

                /* Sets slider limits */
                SendMessage(slider,
                            TBM_SETRANGE,
                            TRUE as WPARAM,
                            MAKELONG!(slider_min, slider_max) as LPARAM);

                UpdateWindow(slider);
                enable_but(BUTREC_ID);
                EnableWindow(slider, (FALSE) as BOOL);
                break 'sw_msg;
            }

            /* Implements slider logic */
            v if v == WM_HSCROLL => {
                'sw_hs: {
                    match (LOWORD!(wParam)) as u32 {
                        w if (w) as u32 == SB_ENDSCROLL => {
                            break 'sw_hs;
                        }
                        w if (w) as u32 == SB_PAGERIGHT || (w) as u32 == SB_PAGELEFT || (w) as u32 == TB_THUMBTRACK => {
                            /* If the user touch the slider bar, set the
                               audio start position properly */
                            slider_pos = (SendMessage(slider, TBM_GETPOS, 0, 0)) as u32;
                            slid_samp = (slider_pos as i64) * (samples_max as i64);
                            (*AUD_BUF).__base1.set_position((*AUD_BUF).audinfo().bytes_in_samples(((slid_samp / (slider_max as i64)) as u32)));
                            InvalidateRect(hWnd, core::ptr::addr_of_mut!(text_rect), (TRUE) as BOOL);
                            break 'sw_hs;
                        }
                        _ => {}
                    }
                }
                break 'sw_msg;
            }

            v if v == WM_COMMAND => {
                wmId = LOWORD!(wParam) as i32;
                if (wmId >= 0 as i32) && (wmId < 5 as i32) && ((butdisabled[wmId as usize]) as u32 != FALSE)
                {
                    break 'sw_msg;
                }

                'sw_cmd: {
                    match (wmId) as u32 {
                        v2 if (v2) as u32 == ID_FILE_NEW => {
                            if ((isnew) == 0)
                            {
                                if (*AUD_IN).current_status() == WAVEIN_RECORDING
                                {
                                    (*AUD_IN).stop_recording();
                                }

                                if ((*AUD_OUT).current_status() == WAVEOUT_PLAYING) ||
                                    ((*AUD_OUT).current_status() == WAVEOUT_PAUSED)
                                {
                                    (*AUD_OUT).stop();
                                }

                                (*AUD_BUF).reset();

                                enable_but(BUTREC_ID);
                                disable_but(BUTSTART_ID);
                                disable_but(BUTEND_ID);
                                disable_but(BUTSTOP_ID);
                                disable_but(BUTPLAY_ID);

                                samples_max = (*AUD_BUF).total_samples();
                                slider_pos = 0;

                                SendMessage(slider, TBM_SETPOS, TRUE as WPARAM, slider_pos as LPARAM);

                                EnableMenuItem(GetMenu(hWnd), ID_FILE_SAVEAS, MF_GRAYED);
                                EnableMenuItem(GetMenu(hWnd), ID_FILE_SAVE, MF_GRAYED);

                                isnew = (TRUE) as i32;
                                display_dur = (TRUE) as i32;

                                ZeroMemory!(file_path.as_mut_ptr() as *mut _, (MAX_PATH as usize) * ((core::mem::size_of::<TCHAR>()) as usize));

                                EnableWindow(slider, (FALSE) as BOOL);

                                InvalidateRect(hWnd, core::ptr::addr_of_mut!(text_rect), (TRUE) as BOOL);
                                InvalidateRect(hWnd, core::ptr::addr_of_mut!(text2_rect), (TRUE) as BOOL);
                            }
                            break 'sw_cmd;
                        }

                        v2 if (v2) as u32 == ID_FILE_OPEN => {
                            ZeroMemory!(core::ptr::addr_of_mut!(ofn) as *mut _, core::mem::size_of::<OPENFILENAME>());

                            ofn.lStructSize = core::mem::size_of::<OPENFILENAME>() as u32;
                            ofn.hwndOwner = hWnd;
                            ofn.lpstrFilter = (&[0x0041, 0x0075, 0x0064, 0x0069, 0x006f, 0x0020, 0x0046, 0x0069, 0x006c, 0x0065, 0x0073, 0x0020, 0x0028, 0x002a, 0x002e, 0x0077, 0x0061, 0x0076, 0x0029, 0x0000, 0x002a, 0x002e, 0x0077, 0x0061, 0x0076, 0x0000, 0x0041, 0x006c, 0x006c, 0x0020, 0x0046, 0x0069, 0x006c, 0x0065, 0x0073, 0x0020, 0x0028, 0x002a, 0x002e, 0x002a, 0x0029, 0x0000, 0x002a, 0x002e, 0x002a, 0x0000, 0x0000u16]).as_ptr() /* TEXT "Audio Files (*.wav)\0*.wav\0All Files (*.*)\0*.*\0" */;
                            ofn.lpstrFile = file_path.as_mut_ptr();
                            ofn.nMaxFile = MAX_PATH;
                            ofn.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_HIDEREADONLY;
                            ofn.lpstrDefExt = (&[0x0077, 0x0061, 0x0076, 0x0000u16]).as_ptr() /* TEXT "wav" */;

                            if GetOpenFileName(core::ptr::addr_of_mut!(ofn)) != 0
                            {
                                open_wav(file_path.as_mut_ptr());
                                EnableMenuItem(GetMenu(hWnd), ID_FILE_SAVE, MF_ENABLED);
                                EnableMenuItem(GetMenu(hWnd), ID_FILE_SAVEAS, MF_ENABLED);

                                EnableWindow(slider, (TRUE) as BOOL);
                            }

                            InvalidateRect(hWnd, core::ptr::addr_of_mut!(text_rect), (TRUE) as BOOL);
                            InvalidateRect(hWnd, core::ptr::addr_of_mut!(text2_rect), (TRUE) as BOOL);
                            break 'sw_cmd;
                        }

                        v2 if (v2) as u32 == ID_ABOUT => {
                            LoadStringW(hInst, IDS_APP_TITLE, szAppName.as_mut_ptr(), (_countof!(szAppName)) as i32);
                            ShellAboutW(hWnd, szAppName.as_ptr(), core::ptr::null_mut(),
                                        LoadIconW(hInst, MAKEINTRESOURCEW!(IDI_REACTOS_SNDREC32)));
                            break 'sw_cmd;
                        }

                        v2 if (v2) as u32 == ID_FILE_SAVEAS => {
                            ZeroMemory!(core::ptr::addr_of_mut!(ofn) as *mut _, core::mem::size_of::<OPENFILENAME>());

                            ofn.lStructSize = core::mem::size_of::<OPENFILENAME>() as u32;
                            ofn.hwndOwner = hWnd ;
                            ofn.Flags = OFN_OVERWRITEPROMPT;
                            ofn.lpstrFilter = (&[0x0041, 0x0075, 0x0064, 0x0069, 0x006f, 0x0020, 0x0046, 0x0069, 0x006c, 0x0065, 0x0073, 0x0020, 0x0028, 0x002a, 0x002e, 0x0077, 0x0061, 0x0076, 0x0029, 0x0000, 0x002a, 0x002e, 0x0077, 0x0061, 0x0076, 0x0000, 0x0041, 0x006c, 0x006c, 0x0020, 0x0046, 0x0069, 0x006c, 0x0065, 0x0073, 0x0020, 0x0028, 0x002a, 0x002e, 0x002a, 0x0029, 0x0000, 0x002a, 0x002e, 0x002a, 0x0000, 0x0000u16]).as_ptr() /* TEXT "Audio Files (*.wav)\0*.wav\0All Files (*.*)\0*.*\0" */;
                            ofn.lpstrFile = file_path.as_mut_ptr();
                            ofn.nMaxFile = MAX_PATH;

                            ofn.lpstrDefExt = (&[0x0077, 0x0061, 0x0076, 0x0000u16]).as_ptr() /* TEXT "wav" */;

                            if GetSaveFileName (core::ptr::addr_of_mut!(ofn)) != 0
                            {
                                write_wav(file_path.as_mut_ptr());
                                EnableMenuItem(GetMenu(hWnd), ID_FILE_SAVE, MF_ENABLED);
                            }
                            break 'sw_cmd;
                        }

                        v2 if (v2) as u32 == ID_EDIT_AUDIOPROPS => {
                            ShellExecute(core::ptr::null_mut(), core::ptr::null_mut(), (&[0x0072, 0x0075, 0x006e, 0x0064, 0x006c, 0x006c, 0x0033, 0x0032, 0x002e, 0x0065, 0x0078, 0x0065, 0x0000u16]).as_ptr() /* _T "rundll32.exe" */, (&[0x0073, 0x0068, 0x0065, 0x006c, 0x006c, 0x0033, 0x0032, 0x002e, 0x0064, 0x006c, 0x006c, 0x002c, 0x0043, 0x006f, 0x006e, 0x0074, 0x0072, 0x006f, 0x006c, 0x005f, 0x0052, 0x0075, 0x006e, 0x0044, 0x004c, 0x004c, 0x0020, 0x006d, 0x006d, 0x0073, 0x0079, 0x0073, 0x002e, 0x0063, 0x0070, 0x006c, 0x002c, 0x0053, 0x0068, 0x006f, 0x0077, 0x0041, 0x0075, 0x0064, 0x0069, 0x006f, 0x0050, 0x0072, 0x006f, 0x0070, 0x0065, 0x0072, 0x0074, 0x0079, 0x0053, 0x0068, 0x0065, 0x0065, 0x0074, 0x0000u16]).as_ptr() /* _T "shell32.dll,Control_RunDLL mmsys.cpl,ShowAudioPropertySheet" */, core::ptr::null_mut(), (SW_SHOWNORMAL as i32));
                            break 'sw_cmd;
                        }

                        v2 if (v2) as u32 == ID_FILE_EXIT => {
                            DestroyWindow(hWnd);
                            break 'sw_cmd;
                        }

                        /* Sndrec32 buttons routines */
                        v2 if (v2) as u32 == BUTSTART_ID => {
                            (*AUD_BUF).__base1.set_position_start();
                            slider_pos = 0;
                            SendMessage(slider, TBM_SETPOS, TRUE as WPARAM, slider_pos as LPARAM);
                            break 'sw_cmd;
                        }

                        v2 if (v2) as u32 == BUTEND_ID => {
                            DestroyWindow(hWnd);
                            break 'sw_cmd;
                        }

                        v2 if (v2) as u32 == BUTPLAY_ID => {
                            if wout_first == (false) as i32
                            {
                                (*AUD_OUT).open();
                                wout_first = (true) as i32;
                            }

                            (*AUD_OUT).play();

                            disable_but(BUTSTART_ID);
                            disable_but(BUTEND_ID);
                            disable_but(BUTREC_ID);
                            disable_but(BUTPLAY_ID);

                            SetTimer(hWnd, 1 as UINT_PTR, 250, None);
                            SetTimer(hWnd, WAVEBAR_TIMERID as UINT_PTR, WAVEBAR_TIMERTIME, None);

                            break 'sw_cmd;
                        }

                        v2 if (v2) as u32 == BUTSTOP_ID => {
                            if ((s_recording) != 0)
                            {
                                s_recording = (FALSE) as i32;

                                (*AUD_IN).stop_recording();

                                /* Resetting slider position */
                                slider_pos = 0;
                                SendMessage(slider, TBM_SETPOS, TRUE as WPARAM, slider_pos as LPARAM);

                                samples_max = (*AUD_BUF).samples_received();

                                EnableMenuItem(IDR_MENU1 as usize as HMENU, ID_FILE_SAVEAS, MF_ENABLED);

                                enable_but(BUTSTART_ID);
                                enable_but(BUTEND_ID);
                                enable_but(BUTREC_ID);
                                enable_but(BUTPLAY_ID);

                                EnableMenuItem(GetMenu(hWnd), ID_FILE_SAVEAS, MF_ENABLED);
                                EnableWindow(slider, (TRUE) as BOOL);

                                display_dur = (FALSE) as i32;

                                (*AUD_BUF).truncate();

                                InvalidateRect(hWnd, core::ptr::addr_of_mut!(text_rect), (TRUE) as BOOL);
                                InvalidateRect(wave_win, core::ptr::null(), (TRUE) as BOOL);

                            }
                            else
                            {
                                (*AUD_OUT).pause();

                                enable_but(BUTSTART_ID);
                                enable_but(BUTEND_ID);
                                enable_but(BUTREC_ID);
                                enable_but(BUTPLAY_ID);

                            }

                            KillTimer(hWnd, 1);
                            KillTimer(hWnd, (WAVEBAR_TIMERID) as UINT_PTR);

                            InvalidateRect(hWnd, core::ptr::addr_of_mut!(text_rect), (TRUE) as BOOL);

                            break 'sw_cmd;
                        }

                        v2 if (v2) as u32 == BUTREC_ID => {
                            if win_first == (false) as i32
                            {
                                (*AUD_IN).open();
                                win_first = (true) as i32;
                            }

                            s_recording = (TRUE) as i32;

                            samples_max = (*AUD_BUF).total_samples();

                            (*AUD_IN).start_recording();

                            enable_but(BUTSTOP_ID);

                            disable_but(BUTSTART_ID);
                            disable_but(BUTEND_ID);
                            disable_but(BUTREC_ID);
                            disable_but(BUTPLAY_ID);

                            isnew = (FALSE) as i32;

                            EnableWindow(slider, (FALSE) as BOOL);

                            SetTimer(hWnd, 1 as UINT_PTR, 150, None);
                            SetTimer(hWnd, WAVEBAR_TIMERID as UINT_PTR, WAVEBAR_TIMERTIME, None);

                            break 'sw_cmd;
                        }

                        _ => {
                            return DefWindowProc(hWnd, message, wParam, lParam);
                        }
                    }
                }
                break 'sw_msg;
            }

            v if v == WM_TIMER => {
                'sw_tm: {
                    match wParam {
                        w if w == 1 => {
                            if ((stopped_flag) != 0)
                            {
                                KillTimer(hWnd, 1);
                                KillTimer(hWnd, (WAVEBAR_TIMERID) as UINT_PTR);
                                slider_pos = 0;
                                enable_but(BUTPLAY_ID);
                                stopped_flag = (FALSE) as i32;
                            }

                            SendMessage(slider, TBM_SETPOS, TRUE as WPARAM, slider_pos as LPARAM);
                            InvalidateRect(hWnd, core::ptr::addr_of_mut!(text_rect), (TRUE) as BOOL);
                            break 'sw_tm;
                        }

                        w if w == WAVEBAR_TIMERID as UINT_PTR => {
                            InvalidateRect(wave_win, core::ptr::null(), (TRUE) as BOOL);
                            SendMessage(wave_win, WM_USER, 0, 0);
                            break 'sw_tm;
                        }

                        _ => {}
                    }
                }
                break 'sw_msg;
            }

            v if v == WM_PAINT => {
                hdc = BeginPaint(hWnd, core::ptr::addr_of_mut!(ps));
                font = CreateFontIndirect(core::ptr::addr_of_mut!(s_info.lfMenuFont));
                oldfont = SelectObject(hdc, (font).cast()) as HFONT;
                SetBkMode(hdc, (TRANSPARENT as i32));

                if (*AUD_IN).current_status() == WAVEIN_RECORDING
                {
                    gprintf(str_tmp.as_mut_ptr(),
                            (MAX_LOADSTRING as usize),
                            (str_pos).as_ptr(),
                            (((((*AUD_BUF).bytes_recorded() as f32) / ((*AUD_BUF).audinfo().byte_rate() as f32))) as f32) as core::ffi::c_double);

                }
                else if (*AUD_OUT).current_status() == WAVEOUT_PLAYING
                {
                    gprintf(str_tmp.as_mut_ptr(),
                            (MAX_LOADSTRING as usize),
                            (str_pos).as_ptr(),
                            (((((*AUD_BUF).__base1.bytes_played() as f32) / ((*AUD_BUF).audinfo().byte_rate() as f32))) as f32) as core::ffi::c_double);
                }
                else
                {
                    gprintf(str_tmp.as_mut_ptr(),
                            (MAX_LOADSTRING as usize),
                            (str_pos).as_ptr(),
                            ((((((slider_pos as f32) * (samples_max as f32)) / (slider_max as f32)) / ((*AUD_BUF).audinfo().sample_rate() as f32))) as f32) as core::ffi::c_double);
                }

                ExtTextOut(hdc,
                           STRPOS_X as c_int,
                           STRPOS_Y as c_int,
                           0,
                           core::ptr::null(),
                           str_tmp.as_ptr(),
                           _tcslen(str_tmp.as_ptr()) as UINT,
                           core::ptr::null());

                if ((display_dur) != 0)
                {
                    gprintf(str_tmp.as_mut_ptr(),
                            (MAX_LOADSTRING as usize),
                            (str_dur).as_ptr(),
                            (*AUD_BUF).fseconds_total() as core::ffi::c_double);
                }
                else
                {
                    gprintf(str_tmp.as_mut_ptr(),
                            (MAX_LOADSTRING as usize),
                            (str_dur).as_ptr(),
                            (*AUD_BUF).fseconds_recorded() as core::ffi::c_double);
                }

                ExtTextOut(hdc,
                           STRDUR_X as c_int,
                           STRDUR_Y as c_int,
                           0,
                           core::ptr::null(),
                           str_tmp.as_ptr(),
                           _tcslen(str_tmp.as_ptr()) as UINT,
                           core::ptr::null());

                gprintf(str_tmp.as_mut_ptr(),
                        (MAX_LOADSTRING as usize),
                        (str_buf).as_ptr(),
                        (((*AUD_BUF).mem_size() as f32 / (1024 as f32)) as f32) as core::ffi::c_double);

                ExtTextOut(hdc,
                           STRBUF_X as c_int,
                           STRBUF_Y as c_int,
                           0,
                           core::ptr::null(),
                           str_tmp.as_ptr(),
                           _tcslen(str_tmp.as_ptr()) as UINT,
                           core::ptr::null());

                gprintf(str_tmp.as_mut_ptr(),
                        (MAX_LOADSTRING as usize),
                        (str_fmt).as_ptr(),
                        (((*AUD_BUF).audinfo().sample_rate() as f32 / (1000 as f32)) as f32) as core::ffi::c_double,
                        (*AUD_BUF).audinfo().bits() as core::ffi::c_uint,
                        // A C array argument DECAYS to its address: passed by value into the
                        // variadic _snwprintf, i386 copied the 20 bytes and `%s` read "st" as a
                        // pointer (64-bit passes the aggregate by hidden reference, so it worked
                        // there by accident). Diverges from the C deliberately: sndrec32.cpp:850
                        // shows "mono" for 2 channels here while :864 shows "stereo" for the same
                        // condition; 2 channels is stereo.
                        if (*AUD_BUF).audinfo().channels() == 2 { str_stereo.as_ptr() } else { str_mono.as_ptr() });

                ExtTextOut(hdc,
                           STRFMT_X as c_int,
                           STRFMT_Y as c_int,
                           0,
                           core::ptr::null(),
                           str_tmp.as_ptr(),
                           _tcslen(str_tmp.as_ptr()) as UINT,
                           core::ptr::null());

                gprintf(str_tmp.as_mut_ptr(),
                        (MAX_LOADSTRING as usize),
                        (str_chan).as_ptr(),
                        if (*AUD_BUF).audinfo().channels() == 2 { str_stereo.as_ptr() } else { str_mono.as_ptr() });

                ExtTextOut(hdc,
                           STRCHAN_X as c_int,
                           STRCHAN_Y as c_int,
                           0,
                           core::ptr::null(),
                           str_tmp.as_ptr(),
                           _tcslen(str_tmp.as_ptr()) as UINT,
                           core::ptr::null());

                SelectObject(hdc, (oldfont).cast());
                DeleteObject((font).cast());
                EndPaint(hWnd, core::ptr::addr_of_mut!(ps));
                break 'sw_msg;
            }

            v if v == WM_DESTROY => {
                PostQuitMessage(0);
                break 'sw_msg;
            }
            _ => {
                return DefWindowProc(hWnd, message, wParam, lParam);
            }
        }
    }

    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn l_play_finished() {
    stopped_flag = (true) as i32;

    enable_but(BUTSTART_ID);
    enable_but(BUTEND_ID);
    enable_but(BUTREC_ID);
    enable_but(BUTPLAY_ID);

    InvalidateRect(wave_win, core::ptr::null(), (TRUE) as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn l_audio_arrival(samples_arrival: u32) {
    slider_pos = slider_pos.wrapping_add(
        (slider_max.wrapping_mul((samples_arrival) as u16) / (samples_max) as u16) as DWORD,
    );
}

#[no_mangle]
pub unsafe extern "C" fn l_buffer_resized(new_size: u32) {}
#[no_mangle]
pub unsafe extern "C" fn enable_but(id: DWORD) {
    butdisabled[id as usize] = (FALSE) as i32;
    SendMessage(buttons[id as usize], BM_SETIMAGE, IMAGE_BITMAP as WPARAM, butbmps[id as usize] as LPARAM);
}
#[no_mangle]
pub unsafe extern "C" fn disable_but(id: DWORD) {
    butdisabled[id as usize] = (TRUE) as i32;
    SendMessage(buttons[id as usize], BM_SETIMAGE, IMAGE_BITMAP as WPARAM, butbmps_dis[id as usize] as LPARAM);
}
#[no_mangle]
pub unsafe extern "C" fn open_wav(f: *mut TCHAR) -> BOOL {
    let mut file: HANDLE;

    let mut r = riff_hdr__make_0();
    let mut w = wave_hdr__make_0();
    let mut d = data_chunk__make_0();

    let mut b: BOOL;

    let mut bytes_recorded_in_wav: DWORD = 0;
    let mut is_read: DWORD = 0;

    file = CreateFile(f,
                      GENERIC_READ,
                      0,
                      core::ptr::null_mut(),
                      OPEN_EXISTING,
                      FILE_ATTRIBUTE_NORMAL,
                      core::ptr::null_mut());
    if file.is_null()
    {
        MessageBox(main_win,
                   (&[0x0043, 0x0061, 0x006e, 0x006e, 0x006f, 0x0074, 0x0020, 0x006f, 0x0070, 0x0065, 0x006e, 0x0020, 0x0066, 0x0069, 0x006c, 0x0065, 0x002e, 0x0020, 0x0043, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0046, 0x0069, 0x006c, 0x0065, 0x0028, 0x0029, 0x0020, 0x0065, 0x0072, 0x0072, 0x006f, 0x0072, 0x002e, 0x0000u16]).as_ptr() /* TEXT "Cannot open file. CreateFile() error." */,
                   (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* TEXT "ERROR" */,
                   MB_OK | MB_ICONERROR);

        return (FALSE) as BOOL;
    }

    b = ReadFile(file, core::ptr::addr_of_mut!(r) as LPVOID, core::mem::size_of::<riff_hdr>() as DWORD, core::ptr::addr_of_mut!(is_read), core::ptr::null_mut());
    if b == 0
    {
        MessageBox(main_win,
                   (&[0x0043, 0x0061, 0x006e, 0x006e, 0x006f, 0x0074, 0x0020, 0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x0052, 0x0049, 0x0046, 0x0046, 0x0020, 0x0068, 0x0065, 0x0061, 0x0064, 0x0065, 0x0072, 0x002e, 0x0000u16]).as_ptr() /* TEXT "Cannot read RIFF header." */,
                   (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* TEXT "ERROR" */,
                   MB_OK | MB_ICONERROR);

        CloseHandle(file);
        return (FALSE) as BOOL;
    }

    b = ReadFile(file, core::ptr::addr_of_mut!(w) as LPVOID, core::mem::size_of::<wave_hdr>() as DWORD, core::ptr::addr_of_mut!(is_read), core::ptr::null_mut());
    if b == 0
    {
        MessageBox(main_win,
                   (&[0x0043, 0x0061, 0x006e, 0x006e, 0x006f, 0x0074, 0x0020, 0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x0057, 0x0041, 0x0056, 0x0045, 0x0020, 0x0068, 0x0065, 0x0061, 0x0064, 0x0065, 0x0072, 0x002e, 0x0000u16]).as_ptr() /* TEXT "Cannot read WAVE header." */,
                   (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* TEXT "ERROR" */,
                   MB_OK | MB_ICONERROR);

        CloseHandle(file);
        return (FALSE) as BOOL;
    }

    b = ReadFile(file, core::ptr::addr_of_mut!(d) as LPVOID, core::mem::size_of::<data_chunk>() as DWORD, core::ptr::addr_of_mut!(is_read), core::ptr::null_mut());
    if b == 0
    {
        MessageBox(main_win,
                   (&[0x0043, 0x0061, 0x006e, 0x006e, 0x006f, 0x0074, 0x0020, 0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x0057, 0x0041, 0x0056, 0x0045, 0x0020, 0x0073, 0x0075, 0x0062, 0x0063, 0x0068, 0x0075, 0x006e, 0x006b, 0x002e, 0x0000u16]).as_ptr() /* TEXT "Cannot read WAVE subchunk." */,
                   (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* TEXT "ERROR" */,
                   MB_OK | MB_ICONERROR);

        CloseHandle(file);
        return (FALSE) as BOOL;
    }

    bytes_recorded_in_wav = r.chunksize - 36;
    if bytes_recorded_in_wav == 0
    {
        MessageBox(main_win,
                   (&[0x0043, 0x0061, 0x006e, 0x006e, 0x006f, 0x0074, 0x0020, 0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x0066, 0x0069, 0x006c, 0x0065, 0x002e, 0x0020, 0x004e, 0x006f, 0x0020, 0x0061, 0x0075, 0x0064, 0x0069, 0x006f, 0x0020, 0x0064, 0x0061, 0x0074, 0x0061, 0x002e, 0x0000u16]).as_ptr() /* TEXT "Cannot read file. No audio data." */,
                   (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* TEXT "ERROR" */,
                   MB_OK | MB_ICONERROR);

        CloseHandle(file);
        return (FALSE) as BOOL;
    }

    let mut openfmt = audio_format__make_0(w.SampleRate, w.BitsPerSample, w.NumChannels);

    (*AUD_BUF).clear();
    (*AUD_BUF).alloc_bytes(bytes_recorded_in_wav);

    b = ReadFile(file,
                 (*AUD_BUF).audio_buffer() as LPVOID,
                 bytes_recorded_in_wav,
                 core::ptr::addr_of_mut!(is_read),
                 core::ptr::null_mut());

    (*AUD_BUF).__base0.set_b_received(bytes_recorded_in_wav);

    if (b == 0) || (is_read != bytes_recorded_in_wav)
    {
        MessageBox(main_win,
                   (&[0x0043, 0x0061, 0x006e, 0x006e, 0x006f, 0x0074, 0x0020, 0x0072, 0x0065, 0x0061, 0x0064, 0x0020, 0x0066, 0x0069, 0x006c, 0x0065, 0x002e, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x0020, 0x0072, 0x0065, 0x0061, 0x0064, 0x0069, 0x006e, 0x0067, 0x0020, 0x0061, 0x0075, 0x0064, 0x0069, 0x006f, 0x0020, 0x0064, 0x0061, 0x0074, 0x0061, 0x002e, 0x0000u16]).as_ptr() /* TEXT "Cannot read file. Error reading audio data." */,
                   (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* TEXT "ERROR" */,
                   MB_OK | MB_ICONERROR);

        CloseHandle(file);

        (*AUD_BUF).reset();
        return (FALSE) as BOOL;
    }

    CloseHandle(file);

    enable_but(BUTPLAY_ID);
    enable_but(BUTSTOP_ID);
    enable_but(BUTSTART_ID);
    enable_but(BUTEND_ID);
    enable_but(BUTREC_ID);

    samples_max = (*AUD_BUF).samples_received();

    isnew = (FALSE) as i32;

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn write_wav(f: *mut TCHAR) -> BOOL {
    let mut file: HANDLE = core::ptr::null_mut();
    let mut written: DWORD = 0;
    let mut is_writ: BOOL = 0;
    let mut i: i32 = 0;
    let mut r: riff_hdr = riff_hdr__make_0();
    let mut w: wave_hdr = wave_hdr__make_0();
    let mut d: data_chunk = data_chunk__make_0();

    file = CreateFile(f, GENERIC_WRITE, 0, core::ptr::null_mut(), CREATE_NEW, FILE_ATTRIBUTE_NORMAL, core::ptr::null_mut());
    if file.is_null() {
        i = MessageBox(main_win,
                       (&[0x0046, 0x0069, 0x006c, 0x0065, 0x0020, 0x0061, 0x006c, 0x0072, 0x0065, 0x0061, 0x0064, 0x0079, 0x0020, 0x0065, 0x0078, 0x0069, 0x0073, 0x0074, 0x002e, 0x0020, 0x004f, 0x0076, 0x0065, 0x0072, 0x0077, 0x0072, 0x0069, 0x0074, 0x0065, 0x0020, 0x0069, 0x0074, 0x003f, 0x0000u16]).as_ptr() /* TEXT "File already exist. Overwrite it?" */,
                       (&[0x0057, 0x0061, 0x0072, 0x006e, 0x0069, 0x006e, 0x0067, 0x0000u16]).as_ptr() /* TEXT "Warning" */,
                       MB_YESNO | MB_ICONQUESTION);

        if i == (IDYES as i32) {
            file = CreateFile(f, GENERIC_WRITE, 0, core::ptr::null_mut(), CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, core::ptr::null_mut());
            if file.is_null() {
                MessageBox(main_win,
                           (&[0x0046, 0x0069, 0x006c, 0x0065, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x002c, 0x0020, 0x0043, 0x0072, 0x0065, 0x0061, 0x0074, 0x0065, 0x0046, 0x0069, 0x006c, 0x0065, 0x0028, 0x0029, 0x0020, 0x0066, 0x0061, 0x0069, 0x006c, 0x0065, 0x0064, 0x002e, 0x0000u16]).as_ptr() /* TEXT "File Error, CreateFile() failed." */,
                           (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* TEXT "ERROR" */,
                           MB_OK | MB_ICONERROR);

                return (FALSE) as BOOL;
            }

        } else {
            return (FALSE) as BOOL;
        }
    }

    r.magic = 0x46464952;
    r.format = 0x45564157;
    r.chunksize = 36 + (*AUD_BUF).bytes_recorded();

    w.Subchunkid = 0x20746d66;
    w.Subchunk1Size = 16;
    w.AudioFormat = 1;
    w.NumChannels = (*AUD_BUF).audinfo().channels();
    w.SampleRate = (*AUD_BUF).audinfo().sample_rate();
    w.ByteRate = (*AUD_BUF).audinfo().byte_rate();
    w.BlockAlign = ((*AUD_BUF).audinfo().block_align()) as u16;
    w.BitsPerSample = (*AUD_BUF).audinfo().bits();

    d.subc = 0x61746164;
    d.subc_size = (*AUD_BUF).bytes_recorded();

    /* Writing headers */
    is_writ = WriteFile(file, core::ptr::addr_of!(r) as LPCVOID, core::mem::size_of::<riff_hdr>() as DWORD, core::ptr::addr_of_mut!(written), core::ptr::null_mut());
    if is_writ == 0 {
        MessageBox(main_win,
                   (&[0x0046, 0x0069, 0x006c, 0x0065, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x002c, 0x0020, 0x0057, 0x0072, 0x0069, 0x0074, 0x0065, 0x0046, 0x0069, 0x006c, 0x0065, 0x0028, 0x0029, 0x0020, 0x0066, 0x0061, 0x0069, 0x006c, 0x0065, 0x0064, 0x002e, 0x0000u16]).as_ptr() /* TEXT "File Error, WriteFile() failed." */,
                   (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* TEXT "ERROR" */,
                   MB_OK | MB_ICONERROR);

        CloseHandle(file);
        return (FALSE) as BOOL;
    }

    is_writ = WriteFile(file, core::ptr::addr_of!(w) as LPCVOID, core::mem::size_of::<wave_hdr>() as DWORD, core::ptr::addr_of_mut!(written), core::ptr::null_mut());
    if is_writ == 0 {
        MessageBox(main_win,
                   (&[0x0046, 0x0069, 0x006c, 0x0065, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x002c, 0x0020, 0x0057, 0x0072, 0x0069, 0x0074, 0x0065, 0x0046, 0x0069, 0x006c, 0x0065, 0x0028, 0x0029, 0x0020, 0x0066, 0x0061, 0x0069, 0x006c, 0x0065, 0x0064, 0x002e, 0x0000u16]).as_ptr() /* TEXT "File Error, WriteFile() failed." */,
                   (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* TEXT "ERROR" */,
                   MB_OK | MB_ICONERROR);

        CloseHandle(file);
        return (FALSE) as BOOL;
    }

    is_writ = WriteFile(file, core::ptr::addr_of!(d) as LPCVOID, core::mem::size_of::<data_chunk>() as DWORD, core::ptr::addr_of_mut!(written), core::ptr::null_mut());
    if is_writ == 0 {
        MessageBox(main_win,
                   (&[0x0046, 0x0069, 0x006c, 0x0065, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x002c, 0x0020, 0x0057, 0x0072, 0x0069, 0x0074, 0x0065, 0x0046, 0x0069, 0x006c, 0x0065, 0x0028, 0x0029, 0x0020, 0x0066, 0x0061, 0x0069, 0x006c, 0x0065, 0x0064, 0x002e, 0x0000u16]).as_ptr() /* TEXT "File Error, WriteFile() failed." */,
                   (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* TEXT "ERROR" */,
                   MB_OK | MB_ICONERROR);

        CloseHandle(file);
        return (FALSE) as BOOL;
    }

    is_writ = WriteFile(file,
                        (*AUD_BUF).audio_buffer() as LPCVOID,
                        (*AUD_BUF).bytes_recorded(),
                        core::ptr::addr_of_mut!(written),
                        core::ptr::null_mut());
    if is_writ == 0 {
        MessageBox(main_win,
                   (&[0x0046, 0x0069, 0x006c, 0x0065, 0x0020, 0x0045, 0x0072, 0x0072, 0x006f, 0x0072, 0x002c, 0x0020, 0x0057, 0x0072, 0x0069, 0x0074, 0x0065, 0x0046, 0x0069, 0x006c, 0x0065, 0x0028, 0x0029, 0x0020, 0x0066, 0x0061, 0x0069, 0x006c, 0x0065, 0x0064, 0x002e, 0x0000u16]).as_ptr() /* TEXT "File Error, WriteFile() failed." */,
                   (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* TEXT "ERROR" */,
                   MB_OK | MB_ICONERROR);

        CloseHandle(file);
        return (FALSE) as BOOL;
    }

    CloseHandle(file);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn __cxx_global_init() {
    UNKNOWN_FORMAT.__ctor_0(0, 0, 0);
    A44100_16BIT_STEREO.__ctor_0(44100, 16, 2);
    A44100_16BIT_MONO.__ctor_0(44100, 16, 1);
}
